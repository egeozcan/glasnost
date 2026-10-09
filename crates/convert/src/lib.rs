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

pub mod coverage;
pub mod emit;
pub mod inputs;
pub mod kinds;
pub mod layers;
pub mod names;

#[cfg(test)]
mod tests;

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
    /// The game-mode file, named directly (Q-0006: the mode list is not
    /// read). Skirmish games without one get an empty game-mode layer.
    pub mode_file: Option<String>,
    /// The map layer, if any.
    pub map: Option<InputFile>,
    /// Theater for art lookup.
    pub theater: Option<Theater>,
    /// INI quirk switches (S-0001, S-0002).
    pub ini: IniQuirks,
    /// Registration and layering quirk switches.
    pub quirks: LayerQuirks,
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
}

/// Imports the rules and art of the install mounted in `vfs`, whose search
/// order must already be built (`Vfs::mount_startup`).
pub async fn import(vfs: &Vfs, options: &ImportOptions) -> Import {
    let inputs = inputs::read_inputs(
        vfs,
        options.mode_file.as_deref(),
        options.map.clone(),
        options.ini,
    )
    .await;
    convert(&inputs, options)
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
    let mut warnings = Vec::new();
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
            bad => warnings.push(Warning {
                file: f.label(),
                line: 0,
                message: format!(
                    "the language rules file's integrity digest is {}; the original refuses to start, \
                     this import ignores the file (S-0006 claim 12, a departure)",
                    if bad == DigestCheck::Missing { "missing" } else { "wrong" }
                ),
            }),
        }
    }
    // spec: S-0006 claim 1 (the game-mode layer in every skirmish or multiplayer game, empty if it has no file)
    if options.game == GameKind::Skirmish {
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
                if let Some(name) = &options.mode_file {
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

    // spec: S-0001 Edge cases (warning: a key that differs from a read key only in letter case)
    for l in &layers {
        warnings.extend(coverage::case_warnings(&l.doc, &l.file));
    }
    warnings.extend(coverage::case_warnings(&art, &art_label));
    let outcome = layers::apply(&layers, &art, &art_label, options.game, &options.quirks);
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

    let (tree, provenance) = emit::emit(&outcome, &record, &art);
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
    }
}

fn q_named(q: &IniQuirks) -> [(&'static str, bool); 4] {
    [
        (
            "ini_final_line_needs_newline",
            q.ini_final_line_needs_newline,
        ),
        ("ini_trim_defect", q.ini_trim_defect),
        ("ini_duplicate_hash_order", q.ini_duplicate_hash_order),
        ("ini_hash_identity", q.ini_hash_identity),
    ]
}
