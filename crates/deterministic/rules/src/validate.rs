//! Semantic validation (PLAN.md 4.6, step 5) and ID interning (step 6).

use std::collections::{BTreeMap, BTreeSet};

use crate::jsonc::Node;
use crate::schema::{Ruleset, SCHEMA_VERSION};

/// A semantic error: a path and what is wrong there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Problem {
    /// Dotted path.
    pub path: String,
    /// What is wrong.
    pub message: String,
}

impl std::fmt::Display for Problem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Number of entries of the colour-adjustment table (S-0006 claim 2).
const COLOUR_ADD_LEN: usize = 16; // spec: S-0006 claim 2 (16 entries)

/// Checks references and ranges.
pub fn validate(r: &Ruleset) -> Vec<Problem> {
    let mut out = Vec::new();
    let mut problem = |path: String, message: String| out.push(Problem { path, message });
    if r.schema != SCHEMA_VERSION {
        problem(
            "schema".into(),
            format!(
                "schema {} is not supported (this build reads {SCHEMA_VERSION})",
                r.schema
            ),
        );
    }
    for (id, o) in &r.objects {
        if let Some(a) = &o.armor
            && !r.armor_classes.contains_key(a)
        {
            problem(
                format!("objects.{id}.armor"),
                format!("unknown armour class {a:?}"),
            );
        }
        for owner in &o.owners {
            if !r.owners.contains_key(owner) {
                problem(
                    format!("objects.{id}.owners"),
                    format!("unknown owner {owner:?}"),
                );
            }
        }
        for p in &o.prerequisites {
            if !r.objects.contains_key(p) {
                problem(
                    format!("objects.{id}.prerequisites"),
                    format!("unknown or abstract object {p:?}"),
                );
            }
        }
        for (slot, a) in &o.armament {
            if !r.weapons.contains_key(&a.weapon) {
                problem(
                    format!("objects.{id}.armament.{slot}.weapon"),
                    format!("unknown weapon {:?}", a.weapon),
                );
            }
        }
    }
    for (id, w) in &r.weapons {
        if let Some(wh) = &w.warhead
            && !r.warheads.contains_key(wh)
        {
            problem(
                format!("weapons.{id}.warhead"),
                format!("unknown warhead {wh:?}"),
            );
        }
    }
    for (id, wh) in &r.warheads {
        for class in wh.versus.keys() {
            if !r.armor_classes.contains_key(class) {
                problem(
                    format!("warheads.{id}.versus"),
                    format!("unknown armour class {class:?}"),
                );
            }
        }
    }
    for (kind, names) in &r.registries {
        let types = r.types.get(kind);
        let mut seen = BTreeSet::new();
        for (i, name) in names.iter().enumerate() {
            if !seen.insert(name) {
                problem(
                    format!("registries.{kind}"),
                    format!("{name:?} listed twice"),
                );
            }
            match types.and_then(|t| t.get(name)) {
                None => problem(
                    format!("registries.{kind}"),
                    format!("{name:?} has no entry in types.{kind}"),
                ),
                Some(t) if t.index as usize != i => problem(
                    format!("types.{kind}.{name}.index"),
                    format!("index {} but registry position {i}", t.index),
                ),
                Some(_) => {}
            }
        }
    }
    for (kind, types) in &r.types {
        let listed: BTreeSet<&String> = r
            .registries
            .get(kind)
            .map(|v| v.iter().collect())
            .unwrap_or_default();
        for name in types.keys() {
            if !listed.contains(name) {
                problem(
                    format!("types.{kind}.{name}"),
                    "not in the kind's registry".into(),
                );
            }
        }
    }
    // nocite: our own rules format (PLAN.md 4.6, #293): the importer writes each versus item as the exact decimal of its double, after S-0003 claim 13's `%` readings; a `%` item has no reading in our format
    for (id, wh) in &r.sim.warheads {
        for (i, d) in wh.verses.iter().flatten().enumerate() {
            if glasnost_num::decimal::exact_double(&d.0).is_err() {
                problem(
                    format!("sim.warheads.{id}.verses"),
                    format!(
                        "item {i} is {:?}: a versus item is a plain decimal (the exact decimal of a double), without `%`",
                        d.0
                    ),
                );
            }
        }
    }
    // spec: S-0214 and S-0404 Import mapping (one `Cloning` key); S-0308 Import mapping (`Naval`); S-0303 and S-0513 Import mapping (`LightningWarhead`, `WeatherConBoltExplosion`); S-0302, S-0303, S-0414 and S-0513 Import mapping (`C4Warhead`): one key each, stored in two fields that different consumers read, so the copies must agree (#323)
    let mut agree = |a: String, b: String, same: bool| {
        if !same {
            problem(
                a.clone(),
                format!("differs from {b}: both hold one imported key and must be set alike"),
            );
        }
    };
    for (id, b) in &r.sim.buildings {
        agree(
            format!("sim.buildings.{id}.cloning"),
            format!("sim.buildings.{id}.econ.cloning"),
            b.cloning == b.econ.cloning,
        );
        agree(
            format!("sim.buildings.{id}.naval"),
            format!("sim.buildings.{id}.combat.naval"),
            b.naval == b.combat.naval,
        );
    }
    let g = &r.sim.general;
    agree(
        "sim.general.lightning_warhead".into(),
        "sim.general.superweapons.lightning_warhead".into(),
        g.lightning_warhead == g.superweapons.lightning_warhead,
    );
    agree(
        "sim.general.weather_con_bolt_explosion".into(),
        "sim.general.superweapons.weather_con_bolt_explosion".into(),
        g.weather_con_bolt_explosion == g.superweapons.weather_con_bolt_explosion,
    );
    agree(
        "sim.general.c4_warhead".into(),
        "sim.general.economy.c4_warhead".into(),
        g.c4_warhead == g.economy.c4_warhead,
    );
    if !r.colour_add.is_empty() && r.colour_add.len() != COLOUR_ADD_LEN {
        problem(
            "colour_add".into(),
            format!("{} entries, expected {COLOUR_ADD_LEN}", r.colour_add.len()),
        );
    }
    out
}

/// Interned IDs: every kind's names in a defined order.
///
/// The imported registries and our own collections are two name spaces:
/// the importer writes a registry per kind (`weapons`, `warheads`, …) and
/// the own format has top-level collections that can share those names, so
/// each keeps its own map.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ids {
    /// Imported names per kind, in registration order (S-0004); the
    /// position is the ID.
    pub kinds: BTreeMap<String, Vec<String>>,
    /// Our own collections' names (`objects`, `weapons`, `warheads`,
    /// `owners`), in the order their instances first appear across the
    /// layers; the position is the ID.
    ///
    /// That order is not covered by `sim_rules_hash`, which encodes object
    /// members sorted by key: two rulesets whose layers first name the same
    /// instances in a different order hash the same and number them
    /// differently. These IDs must not enter simulation state, saves or the
    /// wire until they are derived from an order the hash covers.
    pub own: BTreeMap<String, Vec<String>>,
}

impl Ids {
    /// The ID of `name` in the imported registry of `kind`.
    pub fn get(&self, kind: &str, name: &str) -> Option<usize> {
        self.kinds.get(kind)?.iter().position(|n| n == name)
    }

    /// The ID of `name` in our own collection `collection` (see
    /// [`Ids::own`] for what it may be used for).
    pub fn get_own(&self, collection: &str, name: &str) -> Option<usize> {
        self.own.get(collection)?.iter().position(|n| n == name)
    }
}

/// Interns IDs (PLAN.md 4.6, step 6): imported kinds in registration order
/// (S-0004); our own collections, separately, in the order their instances
/// first appear across the layers (import order, then mods).
pub fn intern(r: &Ruleset, tree: &Node) -> Ids {
    let mut own = BTreeMap::new();
    for collection in crate::resolve::COLLECTIONS {
        let names: Vec<String> = tree
            .get(collection)
            .and_then(Node::members)
            .map(|m| m.iter().map(|x| x.key.clone()).collect())
            .unwrap_or_default();
        if !names.is_empty() {
            own.insert(collection.to_owned(), names);
        }
    }
    Ids {
        kinds: r.registries.clone(),
        own,
    }
}
