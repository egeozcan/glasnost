//! The converted tree (schema 1 of `glasnost_rules`) and the history of
//! every value in it.

use std::collections::BTreeSet;

use glasnost_rules::jsonc::{self, Node};
use glasnost_rules::resolve::{Event as RulesEvent, Origin, Path, Provenance};
use glasnost_rules::schema::SCHEMA_VERSION;

use crate::kinds::Kind;
use crate::layers::{Event, KeyState, Outcome, What};
use crate::names::{encode, show};

/// Converter name and version recorded in every import.
pub const CONVERTER: &str = concat!("glasnost-convert ", env!("CARGO_PKG_VERSION"));

/// Facts about the import recorded under `import`.
#[derive(Clone, Debug, Default)]
pub struct ImportRecord {
    /// `skirmish` or `campaign`.
    pub game: String,
    /// The theater art was looked up for, if any.
    pub theater: Option<String>,
    /// (role, file, origin, sha256) per input, in layer order.
    pub inputs: Vec<(String, String, String, String)>,
    /// Quirk switches by name.
    pub quirks: Vec<(String, bool)>,
}

fn s(v: &str) -> Node {
    jsonc::string(v)
}

fn int(v: i128) -> Node {
    jsonc::integer(v)
}

/// The unique key of each type of a kind: its stored name, or for a
/// second type with the same stored name (S-0004 claim 8) the name with
/// `~` and its index.
pub fn type_keys(reg: &[crate::layers::TypeRec]) -> Vec<String> {
    let mut used = BTreeSet::new();
    reg.iter()
        .enumerate()
        .map(|(i, t)| {
            let base = encode(&t.name);
            if used.insert(base.clone()) {
                base
            } else {
                let k = format!("{base}~{i}");
                used.insert(k.clone());
                k
            }
        })
        .collect()
}

struct Emitter<'a> {
    outcome: &'a Outcome,
    provenance: Provenance,
}

impl Emitter<'_> {
    fn file(&self, layer: usize) -> (String, String) {
        self.outcome
            .files
            .get(layer)
            .cloned()
            .unwrap_or_else(|| ("?".into(), "?".into()))
    }

    fn origin(&self, e: &Event) -> Origin {
        let (layer, file) = self.file(e.layer);
        let detail = match &e.what {
            What::Set(v) => format!("set {:?}", show(v)),
            What::NotGiven(v) => format!("{:?} counts as not given; the held value stays (S-0006 claim 3)", show(v)),
            What::Reset => "the layer has the section but not the key: reset to the engine default (S-0006 claim 5)".into(),
            What::FromTurret(v) => format!(
                "the layer has the section but not the key: takes the turret's value {} (S-0105 claim 27)",
                v.as_ref().map_or_else(|| "(none: the engine default)".to_owned(), |v| format!("{:?}", show(v)))
            ),
            What::Truncated(t) => format!("re-read without the key: truncated to {t} thousandths (S-0104 claim 12)"),
            What::Ignored(v) => format!("{:?} ignored: read from the base rules only (S-0006 claim 2)", show(v)),
        };
        Origin {
            layer,
            file,
            line: e.line,
            event: RulesEvent::Imported,
            detail,
        }
    }

    fn record_key(&mut self, path: &Path, state: &KeyState) {
        for e in &state.history {
            let o = self.origin(e);
            self.provenance.record(path.clone(), o);
        }
    }

    fn note(&mut self, path: Path, layer: usize, line: u32, detail: String) {
        let (l, file) = self.file(layer);
        self.provenance.record(
            path,
            Origin {
                layer: l,
                file,
                line,
                event: RulesEvent::Imported,
                detail,
            },
        );
    }
}

/// Builds the converted tree and its history.
pub fn emit(
    outcome: &Outcome,
    record: &ImportRecord,
    art_sections: &glasnost_ini::Document,
) -> (Node, Provenance) {
    let mut em = Emitter {
        outcome,
        provenance: Provenance::default(),
    };
    let mut root: Vec<(String, Node)> = vec![("schema".into(), int(i128::from(SCHEMA_VERSION)))];

    // import
    let incomplete: Vec<Node> = Kind::all()
        .filter(|k| !k.registry_complete())
        .map(|k| s(k.key()))
        .collect();
    let import = jsonc::object([
        ("converter".to_owned(), s(CONVERTER)),
        ("game".to_owned(), s(&record.game)),
        (
            "theater".to_owned(),
            record.theater.as_deref().map_or_else(
                || Node {
                    value: jsonc::Value::Null,
                    span: Default::default(),
                },
                s,
            ),
        ),
        (
            "inputs".to_owned(),
            jsonc::array(
                record
                    .inputs
                    .iter()
                    .map(|(role, file, origin, sha)| {
                        jsonc::object([
                            ("role".to_owned(), s(role)),
                            ("file".to_owned(), s(file)),
                            ("origin".to_owned(), s(origin)),
                            ("sha256".to_owned(), s(sha)),
                        ])
                    })
                    .collect(),
            ),
        ),
        (
            "quirks".to_owned(),
            jsonc::object(
                record
                    .quirks
                    .iter()
                    .map(|(k, v)| (k.clone(), jsonc::boolean(*v))),
            ),
        ),
        ("incomplete_registries".to_owned(), jsonc::array(incomplete)),
    ]);
    root.push(("import".into(), strip_nulls(import)));

    // registries and types
    let mut registries = Vec::new();
    let mut types = Vec::new();
    for kind in Kind::all() {
        let reg = &outcome.registries[&kind];
        let keys = type_keys(reg);
        registries.push((
            kind.key().to_owned(),
            jsonc::array(keys.iter().map(|k| s(k)).collect()),
        ));
        let mut entries = Vec::new();
        for (i, (t, key)) in reg.iter().zip(&keys).enumerate() {
            let base: Path = vec!["types".into(), kind.key().into(), key.clone()];
            let created = match &t.how {
                crate::layers::How::List => "list".to_owned(),
                crate::layers::How::Reference { kind, name, key } => {
                    format!("reference: {}.{} of {}", kind.key(), key, show(name))
                }
                crate::layers::How::GameWide { section, key } => {
                    format!("reference: [{section}] {key}")
                }
                crate::layers::How::EmptyParticle => {
                    "reference: a particle system without its emitted particle (S-0006 claim 5)"
                        .to_owned()
                }
                crate::layers::How::SideField => "side field".to_owned(),
                crate::layers::How::GameStart => {
                    "game start: the fixed particle system (S-0004 claim 3)".to_owned()
                }
            };
            let mut p = base.clone();
            p.push("created".into());
            em.note(
                p,
                t.created_in,
                0,
                format!("created ({created}) at registration index {i} (S-0004)"),
            );
            let mut ini = Vec::new();
            let mut reset = Vec::new();
            let mut derived = Vec::new();
            for (k, state) in &t.keys {
                let ek = encode(k);
                let mut p = base.clone();
                p.push("ini".into());
                p.push(ek.clone());
                em.record_key(&p, state);
                // A computed value is the value in force (S-0104 claim 12); the
                // stored text it came from stays in the history only.
                if let Some(v) = state.value.as_ref().filter(|_| state.derived.is_none()) {
                    ini.push((ek.clone(), s(&encode(v))));
                }
                if state.reset {
                    reset.push(s(&ek));
                }
                if let Some(d) = state.derived {
                    let mut dp = base.clone();
                    dp.push("derived".into());
                    dp.push(ek.clone());
                    em.record_key(&dp, state);
                    derived.push((ek.clone(), int(i128::from(d))));
                }
            }
            let reads: Vec<Node> = t.reads.iter().map(|&l| s(&em.file(l).0)).collect();
            let mut members = vec![
                ("index".to_owned(), int(i as i128)),
                ("created".to_owned(), s(&created)),
                ("reads".to_owned(), jsonc::array(reads)),
                ("ini".to_owned(), jsonc::object(ini)),
            ];
            if !reset.is_empty() {
                members.push(("reset".to_owned(), jsonc::array(reset)));
            }
            if !derived.is_empty() {
                members.push(("derived".to_owned(), jsonc::object(derived)));
            }
            if let Some(side) = t.side {
                members.push(("side".to_owned(), int(side as i128)));
            }
            entries.push((key.clone(), jsonc::object(members)));
        }
        types.push((kind.key().to_owned(), jsonc::object(entries)));
    }
    root.push(("registries".into(), jsonc::object(registries)));
    root.push(("types".into(), jsonc::object(types)));

    // art database sections as stored (the section a lookup finds, S-0002 claim 2)
    let mut art = Vec::new();
    let art_layer = outcome.files.len().saturating_sub(1);
    let mut seen = BTreeSet::new();
    for (i, section) in art_sections.sections().iter().enumerate() {
        if art_sections.section_index(section.name()) != Some(i)
            || !seen.insert(section.name().to_vec())
        {
            continue;
        }
        let name = encode(section.name());
        let mut keys = Vec::new();
        let mut done = BTreeSet::new();
        for e in section.entries() {
            if !done.insert(e.key.clone()) {
                continue;
            }
            let w = section.get(&e.key).unwrap_or(e);
            let k = encode(&e.key);
            em.note(
                vec!["art_sections".into(), name.clone(), k.clone()],
                art_layer,
                w.line,
                format!("set {:?}", show(&w.value)),
            );
            keys.push((k, s(&encode(&w.value))));
        }
        art.push((name, jsonc::object(keys)));
    }
    root.push(("art_sections".into(), jsonc::object(art)));

    // game-wide sections
    let mut globals = Vec::new();
    let mut globals_derived = Vec::new();
    for (section, keys) in &outcome.globals {
        let sname = encode(section);
        let mut members = Vec::new();
        let mut derived = Vec::new();
        for (k, state) in keys {
            let ek = encode(k);
            em.record_key(
                &vec!["ini_globals".into(), sname.clone(), ek.clone()],
                state,
            );
            match state.derived {
                Some(d) => {
                    em.record_key(
                        &vec!["ini_globals_derived".into(), sname.clone(), ek.clone()],
                        state,
                    );
                    derived.push((ek, int(i128::from(d))));
                }
                None => {
                    if let Some(v) = &state.value {
                        members.push((ek, s(&encode(v))));
                    }
                }
            }
        }
        globals.push((sname.clone(), jsonc::object(members)));
        if !derived.is_empty() {
            globals_derived.push((sname, jsonc::object(derived)));
        }
    }
    root.push(("ini_globals".into(), jsonc::object(globals)));
    root.push(("ini_globals_derived".into(), jsonc::object(globals_derived)));

    // sides
    let countries = &outcome.registries[&Kind::Countries];
    let country_keys = type_keys(countries);
    let sides: Vec<Node> = outcome
        .sides
        .iter()
        .map(|side| {
            jsonc::object([
                ("name".to_owned(), s(&encode(&side.name))),
                (
                    "members".to_owned(),
                    jsonc::array(
                        side.members
                            .iter()
                            .filter_map(|&c| country_keys.get(c).map(|k| s(k)))
                            .collect(),
                    ),
                ),
            ])
        })
        .collect();
    root.push(("sides".into(), jsonc::array(sides)));

    // colours
    let colours: Vec<Node> = outcome
        .colours
        .iter()
        .enumerate()
        .map(|(i, c)| {
            em.note(
                vec!["colours".into(), i.to_string()],
                c.layer,
                c.line,
                format!(
                    "definition position {i}: {} (S-0103 claims 4 and 5)",
                    show(&c.name)
                ),
            );
            jsonc::object([
                ("name".to_owned(), s(&encode(&c.name))),
                (
                    "hsv".to_owned(),
                    jsonc::array(c.hsv.iter().map(|&b| int(i128::from(b))).collect()),
                ),
            ])
        })
        .collect();
    root.push(("colours".into(), jsonc::array(colours)));

    let colour_add: Vec<Node> = outcome
        .colour_add
        .iter()
        .enumerate()
        .map(|(i, (rgb, from))| {
            if let Some((layer, line)) = from {
                em.note(
                    vec!["colour_add".into(), i.to_string()],
                    *layer,
                    *line,
                    format!("position {} set (S-0006 claim 2)", i + 1),
                );
            }
            jsonc::array(rgb.iter().map(|&b| int(i128::from(b))).collect())
        })
        .collect();
    root.push(("colour_add".into(), jsonc::array(colour_add)));

    (jsonc::object(root), em.provenance)
}

fn strip_nulls(node: Node) -> Node {
    match node.value {
        jsonc::Value::Object(members) => Node {
            value: jsonc::Value::Object(
                members
                    .into_iter()
                    .filter(|m| !matches!(m.value.value, jsonc::Value::Null))
                    .collect(),
            ),
            span: node.span,
        },
        other => Node {
            value: other,
            span: node.span,
        },
    }
}

/// Adds each type's looked-up art (S-0007) to the converted tree.
pub fn attach_art(
    tree: &mut Node,
    provenance: &mut Provenance,
    outcome: &Outcome,
    arts: &std::collections::BTreeMap<(Kind, usize), crate::art::TypeArt>,
    theater: glasnost_vfs::theater::Theater,
) {
    let jsonc::Value::Object(root) = &mut tree.value else {
        return;
    };
    let Some(types) = root.iter_mut().find(|m| m.key == "types") else {
        return;
    };
    let jsonc::Value::Object(kinds) = &mut types.value.value else {
        return;
    };
    for ((kind, i), a) in arts {
        let keys = type_keys(&outcome.registries[kind]);
        let Some(key) = keys.get(*i) else { continue };
        let Some(k) = kinds.iter_mut().find(|m| m.key == kind.key()) else {
            continue;
        };
        let jsonc::Value::Object(entries) = &mut k.value.value else {
            continue;
        };
        let Some(entry) = entries.iter_mut().find(|m| &m.key == key) else {
            continue;
        };
        let jsonc::Value::Object(members) = &mut entry.value.value else {
            continue;
        };
        let name = |v: &[u8]| s(&encode(v));
        let mut art = vec![
            ("image".to_owned(), name(&a.image)),
            (
                "flags".to_owned(),
                jsonc::array(a.flags.iter().map(|f| s(f)).collect()),
            ),
        ];
        if let Some(v) = &a.sprite {
            art.push(("sprite".to_owned(), name(v)));
        }
        if !a.voxels.is_empty() {
            art.push((
                "voxels".to_owned(),
                jsonc::object(a.voxels.iter().map(|(r, v)| (r.clone(), name(v)))),
            ));
        }
        if !a.building_sprites.is_empty() {
            art.push((
                "building_sprites".to_owned(),
                jsonc::object(
                    a.building_sprites
                        .iter()
                        .map(|(r, v)| ((*r).to_owned(), name(v))),
                ),
            ));
        }
        for (field, v) in [
            ("icon", &a.icon),
            ("alt_icon", &a.alt_icon),
            ("translucent", &a.translucent),
        ] {
            if let Some(v) = v {
                art.push((field.to_owned(), name(v)));
            }
        }
        if !a.notes.is_empty() {
            art.push((
                "notes".to_owned(),
                jsonc::array(a.notes.iter().map(|n| s(n)).collect()),
            ));
        }
        provenance.record(
            vec!["types".into(), kind.key().into(), key.clone(), "art".into()],
            Origin {
                layer: "art lookup".into(),
                file: "artmd.ini".into(),
                line: 0,
                event: RulesEvent::Imported,
                detail: format!(
                    "looked up for the {} theater (S-0007)",
                    theater.identifier()
                ),
            },
        );
        members.retain(|m| m.key != "art");
        members.push(glasnost_rules::jsonc::Member {
            key: "art".into(),
            key_span: Default::default(),
            value: jsonc::object(art),
        });
    }
}
