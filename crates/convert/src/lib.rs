//! Converter from the user's install to our own formats (PLAN.md 4.3, 4.6).
//!
//! **Milestone M1** fills the rules side:
//!
//! - [`inputs`]: which copies of the rules, art and game-mode files are used
//!   (S-0005), the databases the game cannot start without, and the
//!   language rules file's integrity digest;
//! - [`layers`]: the rules layers of one game applied in order, with type
//!   registration (S-0004) and per-key layering (S-0006), colour
//!   definitions (S-0103 claim 5) and the colour-adjustment table;
//! - [`emit`]: the converted tree in schema 1 of `glasnost_rules`, with the
//!   history of every value for `rules explain`;
//! - [`coverage`] builds `coverage.csv`.
//!
//! **Milestone M2** adds map conversion. Conversion runs locally on the
//! user's machine; its output is never committed.

pub mod art;
pub mod coverage;
pub mod emit;
pub mod find;
pub mod game;
pub mod inputs;
pub mod kinds;
pub mod layers;
pub mod map;
pub mod modes;
pub mod names;
pub mod refs;
pub mod sim;
pub mod theater;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod refs_tests;

use glasnost_ini::{Document, Quirks as IniQuirks};
use glasnost_rules::jsonc::Node;
use glasnost_rules::resolve::Provenance;
use glasnost_vfs::Vfs;
use glasnost_vfs::theater::Theater;

use inputs::{DigestCheck, InputFile, Inputs};
use layers::{GameKind, Layer, LayerQuirks, Outcome, Role, Warning};

/// What to import.
#[derive(Clone, Debug, Default)]
pub struct ImportOptions {
    /// Kind of game: decides the game-mode layer and the side order.
    pub game: GameKind,
    /// The game-mode file, named directly. Without one a skirmish plays
    /// the mode `mode` picks from the mode list (S-0006 claim 13).
    pub mode_file: Option<String>,
    /// Which listed mode a skirmish plays when no game-mode file is named
    /// (S-0006 claim 13).
    pub mode: modes::ModeChoice,
    /// The map layer, if any.
    pub map: Option<InputFile>,
    /// Theater for art lookup.
    pub theater: Option<Theater>,
    /// INI quirk switches (S-0001, S-0002, S-0003).
    pub ini: IniQuirks,
    /// Registration and layering quirk switches.
    pub quirks: LayerQuirks,
    /// Art lookup quirk switches (S-0007).
    pub art: art::ArtQuirks,
    /// Quirk switches of the typed simulation fields (S-0308, S-0309).
    pub sim: sim::SimQuirks,
    /// The local player's country, which picks the side group mounted for
    /// art lookup (S-0005 claim 13a). Default: the first country that has
    /// a side.
    pub country: Option<String>,
}

/// The result of an import.
#[derive(Clone, Debug)]
pub struct Import {
    /// The layered data.
    pub outcome: Outcome,
    /// The converted tree (schema 1).
    pub tree: Node,
    /// History of every value in the tree.
    pub provenance: Provenance,
    /// Coverage rows of the base rules and the art database.
    pub coverage: Vec<coverage::Row>,
    /// Errors: conditions under which the original would not start.
    pub errors: Vec<String>,
    /// Importer warnings, with file and line.
    pub warnings: Vec<Warning>,
    /// Each type's art for the import's theater (S-0007), by kind and
    /// registration index; empty without a theater.
    pub arts: std::collections::BTreeMap<(kinds::Kind, usize), art::TypeArt>,
    /// The art database as read, for the art keys the renderer reads.
    pub art_file: Option<InputFile>,
}

/// Imports the rules and art of the install mounted in `vfs`, whose search
/// order must already be built (`Vfs::mount_startup`). With a theater, the
/// game's per-game archives are mounted (S-0005 claim 13) and every type's
/// art is looked up (S-0007); the loading archives are released afterwards.
pub async fn import(vfs: &mut Vfs, options: &ImportOptions) -> Import {
    let (mode_name, mode_warnings) = mode_file(vfs, options).await;
    let mut inputs =
        inputs::read_inputs(vfs, mode_name.as_deref(), options.map.clone(), options.ini).await;
    inputs.mode_name = mode_name;
    inputs.mode_warnings = mode_warnings;
    let mut import = convert(&inputs, options);
    if let Some(theater) = options.theater {
        look_up_art(vfs, options, theater, &mut import, &inputs).await;
    }
    import
}

/// The game-mode file of the import: the one the options name, or for a
/// skirmish the file of the mode the mode list gives (S-0006 claim 13),
/// with the mode list's warnings.
async fn mode_file(vfs: &Vfs, options: &ImportOptions) -> (Option<String>, Vec<Warning>) {
    if options.mode_file.is_some() || options.game != GameKind::Skirmish {
        return (options.mode_file.clone(), Vec::new());
    }
    mode_from_list(inputs::read(vfs, modes::MODE_LIST).await.as_ref(), options)
}

/// The game-mode file a skirmish plays from the mode list `list` (S-0006
/// claim 13), with the mode list's warnings.
pub fn mode_from_list(
    list: Option<&InputFile>,
    options: &ImportOptions,
) -> (Option<String>, Vec<Warning>) {
    let mut warnings = Vec::new();
    let Some(f) = list else {
        warnings.push(Warning {
            file: modes::MODE_LIST.into(),
            line: 0,
            message: "the mode list cannot be resolved, so no mode is listed; the game-mode layer is empty (S-0006 claim 13)".into(),
        });
        return (None, warnings);
    };
    let doc = inputs::parse(f, options.ini, &mut Vec::new());
    let (list, notes) = modes::read(&doc);
    for (line, message) in notes {
        warnings.push(Warning {
            file: f.label(),
            line,
            message,
        });
    }
    let Some(i) = modes::select(&list, options.mode, options.quirks.mode_number_first_wins) else {
        warnings.push(Warning {
            file: f.label(),
            line: 0,
            message: "the mode list lists no mode; the game-mode layer is empty (S-0006 claim 13)"
                .into(),
        });
        return (None, warnings);
    };
    (Some(list[i].file_name()), warnings)
}

/// Answers art file questions through the game's search order.
struct VfsFiles<'a> {
    vfs: &'a Vfs,
    /// Validity of model and motion files read so far; names not in the
    /// map count as valid and are recorded in `asked`.
    valid: std::collections::BTreeMap<Vec<u8>, bool>,
    asked: std::cell::RefCell<std::collections::BTreeSet<Vec<u8>>>,
}

impl art::ArtFiles for VfsFiles<'_> {
    fn exists(&self, name: &[u8]) -> bool {
        self.vfs.resolve_bytes(name).is_some()
    }

    fn valid(&self, name: &[u8]) -> bool {
        match self.valid.get(name) {
            Some(v) => *v,
            None => {
                self.asked.borrow_mut().insert(name.to_vec());
                true
            }
        }
    }
}

async fn look_up_art(
    vfs: &mut Vfs,
    options: &ImportOptions,
    theater: Theater,
    import: &mut Import,
    inputs: &Inputs,
) {
    let countries = &import.outcome.registries[&kinds::Kind::Countries];
    let chosen = options.country.as_ref().and_then(|c| {
        countries
            .iter()
            .find(|t| t.name.eq_ignore_ascii_case(c.as_bytes()))
    });
    let side = chosen
        .or_else(|| countries.iter().find(|t| t.side.is_some()))
        .and_then(|t| t.side)
        .map(|s| s as u32);
    // spec: S-0005 claim 13 (the per-game archives: loading, theater and side groups)
    let game = vfs.mount_game(theater, side).await;
    if game.never_reaches_play {
        import.errors.push(
            "the local player's side archive cannot be resolved: the game never reaches play \
             (S-0005 claims 13a and 16)"
                .into(),
        );
    }
    let art_doc = match &inputs.art {
        Some(f) => Document::parse(&f.bytes, options.ini, &mut Vec::new()),
        None => Document::empty(options.ini),
    };
    // First pass: which model and motion files are asked for.
    let mut files = VfsFiles {
        vfs,
        valid: Default::default(),
        asked: Default::default(),
    };
    let _ = art::lookup(&import.outcome, &art_doc, theater, &files, &options.art);
    let asked: Vec<Vec<u8>> = files.asked.borrow().iter().cloned().collect();
    for name in asked {
        let ok = match files.vfs.resolve_bytes(&name) {
            Some(r) => match files.vfs.read_resolved(&r).await {
                Ok(bytes) if name.to_ascii_lowercase().ends_with(b".vxl") => {
                    glasnost_fmt::vxl::Vxl::parse(&bytes).is_ok()
                }
                Ok(bytes) => glasnost_fmt::hva::Hva::parse(&bytes).is_ok(),
                Err(_) => false,
            },
            None => false,
        };
        files.valid.insert(name, ok);
    }
    // Second pass with the files' validity known.
    let (mut arts, notes) = art::lookup(&import.outcome, &art_doc, theater, &files, &options.art);
    // spec: S-0108 claim 5 (the construction range is set when the build-up sprite is loaded: n = ⌊N_b / 2⌋ from its image-frame count)
    for a in arts.values_mut() {
        let Some(f) = a.building_sprites.get("buildup") else {
            continue;
        };
        let Some(r) = vfs.resolve_bytes(f) else {
            continue;
        };
        if let Ok(bytes) = vfs.read_resolved(&r).await
            && let Ok(shp) = glasnost_fmt::shp::Shp::parse(&bytes)
        {
            a.buildup_frames = Some(i32::try_from(shp.frames.len()).unwrap_or(i32::MAX));
        }
    }
    // spec: S-0005 claim 13 (the loading archives are released before play)
    vfs.finish_loading();
    let in_play = |n: &[u8]| vfs.resolve_bytes(n).is_some();
    art::note_vanishing_sprites(&mut arts, &in_play, &options.art);
    let art_label = inputs
        .art
        .as_ref()
        .map_or_else(|| inputs::ART.to_owned(), InputFile::label);
    import
        .warnings
        .extend(notes.into_iter().map(|message| Warning {
            file: art_label.clone(),
            line: 0,
            message,
        }));
    emit::attach_art(
        &mut import.tree,
        &mut import.provenance,
        &import.outcome,
        &arts,
        theater,
    );
    // The art-dependent simulation fields, now with each type's art (S-0007).
    let (sim_node, sim_warnings) = sim::build(sim::SimInputs {
        outcome: &import.outcome,
        art: &art_doc,
        arts: Some(&arts),
        quirks: &options.quirks,
        art_quirks: &options.art,
        ini_quirks: &options.ini,
        sim_quirks: &options.sim,
    });
    sim::attach(&mut import.tree, sim_node);
    for w in sim_warnings {
        if !import.warnings.contains(&w) {
            import.warnings.push(w);
        }
    }
    import.arts = arts;
}

fn empty_layer(role: Role, file: &str, quirks: IniQuirks) -> Layer {
    Layer {
        role,
        file: file.to_owned(),
        doc: Document::empty(quirks),
    }
}

/// The synchronous part of [`import`]: layers, tree and coverage from
/// files already read.
pub fn convert(inputs: &Inputs, options: &ImportOptions) -> Import {
    let q = options.ini;
    let mut ini_warnings = Vec::new();
    let mut errors = Vec::new();
    let mut warnings = inputs.mode_warnings.clone();
    let mut record = emit::ImportRecord {
        game: match options.game {
            GameKind::Skirmish => "skirmish".into(),
            GameKind::Campaign => "campaign".into(),
        },
        theater: options.theater.map(|t| t.identifier().to_owned()),
        ..Default::default()
    };
    let mut note_input = |role: &str, f: &InputFile| {
        record.inputs.push((
            role.to_owned(),
            f.name.clone(),
            f.origin.clone(),
            f.sha256(),
        ));
    };

    let mut layers = Vec::new();
    let mut bad_language_rules = false;
    // spec: S-0006 claim 1 (the base rules first)
    match &inputs.base {
        Some(f) => {
            note_input("base rules", f);
            layers.push(Layer {
                role: Role::Base,
                file: f.label(),
                doc: inputs::parse(f, q, &mut ini_warnings),
            });
        }
        None => {
            // spec: S-0005 claim 16 (a missing base rules file is treated as empty)
            warnings.push(Warning {
                file: inputs::BASE_RULES.into(),
                line: 0,
                message: "the base rules file cannot be resolved; it is treated as empty (S-0005 claim 16)".into(),
            });
            layers.push(empty_layer(Role::Base, inputs::BASE_RULES, q));
        }
    }
    // spec: S-0006 claims 1 and 12 (the language rules file, only with a valid digest)
    if let Some(f) = &inputs.language {
        let doc = inputs::parse(f, q, &mut ini_warnings);
        match inputs::check_digest(&doc) {
            DigestCheck::Valid | DigestCheck::Empty => {
                note_input("language rules", f);
                layers.push(Layer {
                    role: Role::Language,
                    file: f.label(),
                    doc,
                });
            }
            bad => {
                // spec: S-0006 claim 12 (program start: a departure, our engine starts without the file)
                bad_language_rules = true;
                warnings.push(Warning {
                    file: f.label(),
                    line: 0,
                    message: format!(
                        "the language rules file's integrity digest is {}; the original refuses to start, \
                         this import ignores the file (S-0006 claim 12, a departure)",
                        if bad == DigestCheck::Missing { "missing" } else { "wrong" }
                    ),
                });
            }
        }
    }
    // spec: S-0006 claim 12 (game start: a failing language rules file skips the whole game-mode layer)
    let skip_mode = bad_language_rules && options.quirks.bad_language_rules_skips_mode;
    if skip_mode && options.game == GameKind::Skirmish {
        warnings.push(Warning {
            file: inputs::LANGUAGE_RULES.into(),
            line: 0,
            message: "because the language rules file fails its check, the game-mode layer is skipped and \
                      the colour-adjustment table takes only the map's entries (S-0006 claim 12)"
                .into(),
        });
    }
    // spec: S-0006 claim 1 (the game-mode layer in every skirmish or multiplayer game, empty if it has no file)
    if options.game == GameKind::Skirmish && !skip_mode {
        match &inputs.mode {
            Some(f) => {
                note_input("game mode", f);
                layers.push(Layer {
                    role: Role::Mode,
                    file: f.label(),
                    doc: inputs::parse(f, q, &mut ini_warnings),
                });
            }
            None => {
                if let Some(name) = &inputs.mode_name {
                    warnings.push(Warning {
                        file: name.clone(),
                        line: 0,
                        message: "the game-mode file cannot be resolved; the mode layer is empty (S-0005 claim 19)".into(),
                    });
                }
                layers.push(empty_layer(Role::Mode, "(no game-mode file)", q));
            }
        }
    }
    // spec: S-0006 claim 1 (the map layer last)
    match &inputs.map {
        Some(f) => {
            note_input("map", f);
            layers.push(Layer {
                role: Role::Map,
                file: f.label(),
                doc: inputs::parse(f, q, &mut ini_warnings),
            });
        }
        None => layers.push(empty_layer(Role::Map, "(no map)", q)),
    }

    let (art, art_label) = match &inputs.art {
        Some(f) => {
            note_input("art database", f);
            (inputs::parse(f, q, &mut ini_warnings), f.label())
        }
        None => (Document::empty(q), inputs::ART.to_owned()),
    };
    for (a, b) in &inputs.case_clashes {
        // spec: S-0005 Edge cases (an error at import until one of the two is removed)
        errors.push(format!(
            "the game directory holds {a} and {b}, whose names differ only in letter case; remove one \
             (S-0005 Edge cases)"
        ));
    }
    for name in &inputs.missing_databases {
        // spec: S-0005 claim 16 (required databases)
        errors.push(format!(
            "{name} cannot be resolved or holds no section: the original refuses to start (S-0005 claim 16)"
        ));
    }
    for name in &inputs.shadowing_loose {
        warnings.push(Warning {
            file: name.clone(),
            line: 0,
            message:
                "a loose file of this name would override the archive copy; it is not used because \
                      the mod layer is off (S-0005 claim 1, PLAN.md 4.3)"
                    .into(),
        });
    }

    for (k, v) in q_named(&options.ini) {
        record.quirks.push((k.to_owned(), v));
    }
    for (k, v) in options.quirks.named() {
        record.quirks.push((k.to_owned(), v));
    }
    for (k, v) in options.art.named() {
        record.quirks.push((k.to_owned(), v));
    }
    for (k, v) in options.sim.named() {
        record.quirks.push((k.to_owned(), v));
    }

    // spec: S-0001 Edge cases (warning: a key that differs from a read key only in letter case)
    for l in &layers {
        warnings.extend(coverage::case_warnings(&l.doc, &l.file));
    }
    warnings.extend(coverage::case_warnings(&art, &art_label));
    // spec: S-0004 claim 14 (the snow suffix selects the art section of building and aircraft references)
    let select = layers::ArtSelect {
        snow: options.theater == Some(Theater::Snow),
        quirks: options.art,
    };
    let outcome = layers::apply_in(
        &layers,
        &art,
        &art_label,
        options.game,
        &options.quirks,
        bad_language_rules,
        &select,
    );
    warnings.extend(ini_warnings.into_iter().map(|(file, w)| {
        Warning {
            file,
            line: w.line,
            message: w
                .to_string()
                .split_once(": ")
                .map_or_else(|| w.to_string(), |(_, m)| m.to_owned()),
        }
    }));
    warnings.extend(outcome.warnings.iter().cloned());

    let (mut tree, provenance) = emit::emit(&outcome, &record, &art);
    let (sim_node, sim_warnings) = sim::build(sim::SimInputs {
        outcome: &outcome,
        art: &art,
        arts: None,
        quirks: &options.quirks,
        art_quirks: &options.art,
        ini_quirks: &options.ini,
        sim_quirks: &options.sim,
    });
    sim::attach(&mut tree, sim_node);
    warnings.extend(sim_warnings);
    let base_doc = layers.first().map(|l| &l.doc);
    let coverage = base_doc
        .map(|b| coverage::rows(b, &art, &outcome))
        .unwrap_or_default();
    Import {
        outcome,
        tree,
        provenance,
        coverage,
        errors,
        warnings,
        arts: Default::default(),
        art_file: inputs.art.clone(),
    }
}

fn q_named(q: &IniQuirks) -> [(&'static str, bool); 6] {
    [
        (
            "ini_final_line_needs_newline",
            q.ini_final_line_needs_newline,
        ),
        ("ini_trim_defect", q.ini_trim_defect),
        ("ini_duplicate_hash_order", q.ini_duplicate_hash_order),
        ("ini_hash_identity", q.ini_hash_identity),
        ("percent_list_integer_prefix", q.percent_list_integer_prefix),
        ("list_items_untrimmed", q.list_items_untrimmed),
    ]
}
