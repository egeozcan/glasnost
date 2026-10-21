//! The desync tool (M9 N4, `glasnost-headless desync <bundle A> <bundle
//! B>`): reads two peers' desync reports ([`glasnost_net::desync::Bundle`]),
//! checks that they start from the same values and the same snapshot (the
//! older snapshot is replayed forward to the newer), replays both sides
//! tick by tick with their merged orders, and prints:
//!
//! - the first tick whose state hashes, as each peer recorded them, differ;
//! - whether each peer's own hashes are what this build computes from its
//!   snapshot and orders (a peer whose run departs from its replay changed
//!   outside the rules this build computes: another build, a platform
//!   difference, a memory fault, or a test perturbation it did not record);
//! - whether the merged orders differ (they never should: a lockstep fault);
//! - a field-by-field diff of the two games at the first tick where they
//!   can be compared: the replayed states where the replays differ (exactly
//!   the first differing tick when a bundle records the perturbation that
//!   caused it), else each peer's own snapshot at its mismatched
//!   checkpoint. Objects first (id, kind, type, owner, then each differing
//!   field with both values), then the houses, the game-wide state and our
//!   AI's state.
//!
//! [`compare`] does the work and says whether anything differs; the binary
//! turns that into its exit code. The report text, the order of its checks
//! and the diff are our own tool (`// nocite: our own netcode`); the rules
//! and the tile catalogue come from the synthetic fixture or the install,
//! never from the bundle.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::Arc;

use glasnost_ai::game::Game;
use glasnost_map::terrain::TileCatalogue;
use glasnost_net::desync::{Bundle, Perturb, Snapshot, TickRecord};
use glasnost_net::sim::{hash_from_hex, hex};
use glasnost_net::start::content_hash;
use glasnost_net::wire::Hash;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{HouseId, World};

use crate::inspect::{self, FieldDiff};

/// The most differing fields shown per object, house or AI.
// nocite: our own netcode (a readable report)
const FIELDS_SHOWN: usize = 48;

/// The most differing objects listed.
// nocite: our own netcode (a readable report)
const OBJECTS_SHOWN: usize = 64;

/// What the games need besides a bundle: the rules and the theater's tile
/// catalogue.
#[derive(Clone)]
pub struct Inputs {
    /// The rules the game ran under.
    pub rules: Arc<SimRules>,
    /// The map's theater catalogue.
    pub catalogue: Arc<TileCatalogue>,
}

impl Inputs {
    /// The synthetic fixture's (`glasnost_ai::fixture`), when its rules are
    /// the ones the report names.
    pub fn synthetic(rules_hash: &str) -> Option<Self> {
        let rules = glasnost_ai::fixture::rules();
        (rules.hash == rules_hash).then(|| Self {
            rules,
            catalogue: glasnost_ai::fixture::catalogue(),
        })
    }

    /// The install's, for the map whose content hash the report names
    /// (local only: the install's data never leaves the machine).
    pub fn install(dir: &Path, map_hash: &str, rules_hash: &str) -> Result<Self, String> {
        pollster::block_on(async {
            let mut vfs = crate::local_lane::mount(dir)
                .await
                .map_err(|e| format!("the install could not be read ({:?})", e.kind()))?;
            for listing in glasnost_convert::find::find_maps(&vfs).await {
                let Ok(bytes) = vfs.read(&listing.file).await else {
                    continue;
                };
                if content_hash(&bytes) != map_hash {
                    continue;
                }
                let inputs = glasnost_convert::game::load(&mut vfs, bytes, "install".into()).await;
                let resolved = inputs
                    .resolved
                    .map_err(|e| format!("the map's rules could not be resolved: {e}"))?;
                let tiles = inputs
                    .tiles
                    .ok_or("the map's theater has no tile catalogue in the install")?;
                let rules = SimRules::new(&resolved.ruleset, &resolved.sim_rules_hash);
                if rules.hash != rules_hash {
                    return Err(format!(
                        "the install's rules for that map hash to {}, the report's to {rules_hash}",
                        rules.hash
                    ));
                }
                return Ok(Self {
                    rules: Arc::new(rules),
                    catalogue: Arc::new(tiles),
                });
            }
            Err(format!(
                "no map of the install has the report's map hash {map_hash}"
            ))
        })
    }
}

/// Which bundle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// The first.
    A,
    /// The second.
    B,
}

/// An object that differs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectDiff {
    /// Its id.
    pub id: u32,
    /// Its kind.
    pub kind: String,
    /// Its type's name in the rules (or its index).
    pub type_name: String,
    /// Its owner's house index.
    pub owner: Option<u8>,
    /// The side it exists on, when only one has it.
    pub only: Option<Side>,
    /// Its differing fields (A's value, then B's).
    pub fields: Vec<FieldDiff>,
}

/// A house (or its AI) that differs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HouseDiff {
    /// Its house index.
    pub house: u8,
    /// Its differing fields.
    pub fields: Vec<FieldDiff>,
}

/// Two games compared field by field.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StateDiff {
    /// The objects that differ, by id.
    pub objects: Vec<ObjectDiff>,
    /// How many objects differ (more than are listed, perhaps).
    pub objects_differing: usize,
    /// The houses that differ.
    pub houses: Vec<HouseDiff>,
    /// The game-wide fields that differ (outside the objects and houses).
    pub game: Vec<FieldDiff>,
    /// Our AI's houses whose state differs.
    pub ai: Vec<HouseDiff>,
}

impl StateDiff {
    /// Whether nothing differs.
    pub fn is_empty(&self) -> bool {
        self.objects_differing == 0
            && self.houses.is_empty()
            && self.game.is_empty()
            && self.ai.is_empty()
    }
}

/// What the comparison found.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The report as text.
    pub text: String,
    /// Whether the two sides differ anywhere.
    pub differs: bool,
    /// The first tick after which the peers' own hashes differ.
    pub first_tick: Option<u32>,
    /// The first tick after which the replayed states differ.
    pub replay_tick: Option<u32>,
    /// The first tick whose merged orders differ.
    pub orders_tick: Option<u32>,
    /// Per side, the first tick whose recorded hash its replay does not
    /// reproduce.
    pub unreproduced: [Option<u32>; 2],
    /// The tick after which the diff shows the games.
    pub diff_tick: Option<u32>,
    /// The diff.
    pub diff: StateDiff,
}

fn hash_of(g: &Game) -> Hash {
    hash_from_hex(&g.state_hash()).expect("the game's state hash is 64 hex digits")
}

fn short(h: &Hash) -> String {
    hex(h)[..12].to_owned() // nocite: our own netcode (a short digest for people)
}

fn load(s: &Snapshot, inputs: &Inputs, side: char) -> Result<Game, String> {
    Game::load(&s.bytes, inputs.rules.clone(), inputs.catalogue.clone())
        .map_err(|e| format!("bundle {side}'s snapshot does not load: {e}"))
}

/// Runs one recorded tick: the recorded perturbation first when it is due
/// (as the peer made it, before the tick), then the merged orders in their
/// order, then the step, as `glasnost_net::Session::step` does.
fn run_tick(g: &mut Game, rec: &TickRecord, perturb: Option<Perturb>) {
    if let Some(p) = perturb
        && p.tick() == rec.tick
    {
        p.apply(&mut g.world);
    }
    for i in &rec.orders {
        g.issue(HouseId(i.house), i.order.clone());
    }
    g.step();
}

/// Type names by kind and registration index.
fn type_names(rules: &SimRules) -> BTreeMap<(Kind, u32), String> {
    rules
        .names
        .iter()
        .map(|((k, name), &i)| ((*k, i), name.clone()))
        .collect()
}

fn fields_of<T: serde::Serialize>(a: &T, b: &T) -> Vec<FieldDiff> {
    let (Ok(va), Ok(vb)) = (inspect::view(a), inspect::view(b)) else {
        return vec![FieldDiff {
            path: "(value)".into(),
            a: "(not viewable)".into(),
            b: "(not viewable)".into(),
        }];
    };
    let mut out = Vec::new();
    inspect::diff("", &va, &vb, FIELDS_SHOWN, &mut out);
    out
}

fn same_bytes<T: serde::Serialize>(a: &T, b: &T) -> bool {
    postcard::to_allocvec(a).ok() == postcard::to_allocvec(b).ok()
}

/// Two games compared field by field: objects, houses, the game-wide
/// state and our AI's state.
pub fn diff_games(ga: &Game, gb: &Game, rules: &SimRules) -> StateDiff {
    let names = type_names(rules);
    let (wa, wb) = (&ga.world, &gb.world);
    let mut d = StateDiff::default();
    let ids: BTreeSet<_> = wa
        .objects
        .keys()
        .chain(wb.objects.keys())
        .copied()
        .collect();
    for id in ids {
        let (oa, ob) = (wa.objects.get(&id), wb.objects.get(&id));
        let (o, only, fields) = match (oa, ob) {
            (Some(x), Some(y)) => {
                if same_bytes(x, y) {
                    continue;
                }
                (x, None, fields_of(x, y))
            }
            (Some(x), None) => (x, Some(Side::A), Vec::new()),
            (None, Some(y)) => (y, Some(Side::B), Vec::new()),
            (None, None) => continue,
        };
        d.objects_differing += 1;
        if d.objects.len() >= OBJECTS_SHOWN {
            continue;
        }
        d.objects.push(ObjectDiff {
            id: id.0,
            kind: format!("{:?}", o.kind),
            type_name: names
                .get(&(o.kind, o.type_index))
                .cloned()
                .unwrap_or_else(|| format!("type {}", o.type_index)),
            owner: o.owner.map(|h| h.0),
            only,
            fields,
        });
    }
    for i in 0..wa.houses.len().max(wb.houses.len()) {
        match (wa.houses.get(i), wb.houses.get(i)) {
            (Some(x), Some(y)) if !same_bytes(x, y) => d.houses.push(HouseDiff {
                house: i as u8,
                fields: fields_of(x, y),
            }),
            (Some(_), Some(_)) => {}
            (x, _) => d.houses.push(HouseDiff {
                house: i as u8,
                fields: vec![FieldDiff {
                    path: "(house)".into(),
                    a: if x.is_some() { "present" } else { "(none)" }.into(),
                    b: if x.is_some() { "(none)" } else { "present" }.into(),
                }],
            }),
        }
    }
    d.game = game_wide(wa, wb);
    let (ba, bb) = (ga.brains(), gb.brains());
    for i in 0..ba.len().max(bb.len()) {
        match (ba.get(i), bb.get(i)) {
            (Some(x), Some(y)) if !same_bytes(x, y) => d.ai.push(HouseDiff {
                house: x.house().0,
                fields: fields_of(x, y),
            }),
            (Some(_), Some(_)) => {}
            (x, y) => d.ai.push(HouseDiff {
                house: x.or(y).map_or(0, |b| b.house().0),
                fields: vec![FieldDiff {
                    path: "(brain)".into(),
                    a: if x.is_some() { "present" } else { "(none)" }.into(),
                    b: if y.is_some() { "present" } else { "(none)" }.into(),
                }],
            }),
        }
    }
    d
}

/// The world's fields other than the objects and houses that differ.
fn game_wide(wa: &World, wb: &World) -> Vec<FieldDiff> {
    let (Ok(va), Ok(vb)) = (inspect::view(wa), inspect::view(wb)) else {
        return Vec::new();
    };
    let (inspect::Value::Struct(fa), inspect::Value::Struct(fb)) = (&va, &vb) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for ((k, x), (_, y)) in fa.iter().zip(fb) {
        if *k == "objects" || *k == "houses" {
            continue;
        }
        inspect::diff(k, x, y, FIELDS_SHOWN, &mut out);
    }
    out
}

fn write_fields(out: &mut String, fields: &[FieldDiff]) {
    for f in fields {
        let _ = writeln!(out, "    {}: {} / {}", f.path, f.a, f.b);
    }
}

/// The diff as text (values: A's, then B's).
pub fn write_diff(out: &mut String, d: &StateDiff, slot_of_house: &dyn Fn(u8) -> Option<u8>) {
    if d.objects_differing == 0 {
        let _ = writeln!(out, "Objects: the same.");
    } else {
        let _ = writeln!(
            out,
            "Objects ({} differ; values A / B):",
            d.objects_differing
        );
        for o in &d.objects {
            let owner = o
                .owner
                .map_or_else(|| "no owner".into(), |h| format!("owner house {h}"));
            let _ = write!(
                out,
                "  object {}: {} {:?}, {owner}",
                o.id, o.kind, o.type_name
            );
            match o.only {
                Some(s) => {
                    let _ = writeln!(out, ": only in {s:?}");
                }
                None => {
                    let _ = writeln!(out);
                    write_fields(out, &o.fields);
                }
            }
        }
        if d.objects.len() < d.objects_differing {
            let _ = writeln!(
                out,
                "  ({} more objects differ)",
                d.objects_differing - d.objects.len()
            );
        }
    }
    let houses = |out: &mut String, title: &str, list: &[HouseDiff]| {
        if list.is_empty() {
            let _ = writeln!(out, "{title}: the same.");
            return;
        }
        let _ = writeln!(out, "{title} ({} differ; values A / B):", list.len());
        for h in list {
            let slot = slot_of_house(h.house)
                .map_or_else(String::new, |s| format!(" (slot {})", u32::from(s) + 1));
            let _ = writeln!(out, "  house {}{slot}:", h.house);
            write_fields(out, &h.fields);
        }
    };
    houses(out, "Houses", &d.houses);
    if d.game.is_empty() {
        let _ = writeln!(out, "Game-wide state: the same.");
    } else {
        let _ = writeln!(
            out,
            "Game-wide state ({} fields differ; values A / B):",
            d.game.len()
        );
        for f in &d.game {
            let _ = writeln!(out, "  {}: {} / {}", f.path, f.a, f.b);
        }
    }
    houses(out, "Our AI's state", &d.ai);
}

fn describe(out: &mut String, side: char, b: &Bundle) {
    let _ = writeln!(
        out,
        "{side}: slot {} of {}, build {:?}, ticks {} to {} from a snapshot after {} ticks",
        u32::from(b.writer) + 1,
        b.start.slots.len(),
        b.build,
        b.base.ran,
        b.end().saturating_sub(1),
        b.base.ran
    );
    match b.mismatch {
        Some(m) => {
            let _ = writeln!(
                out,
                "   reports a mismatch with slot {} after tick {}: its hash {}, theirs {}",
                u32::from(m.slot) + 1,
                m.tick,
                short(&m.mine),
                short(&m.theirs)
            );
        }
        None => {
            let _ = writeln!(out, "   reports no mismatch (made on demand)");
        }
    }
    if let Some(p) = b.perturb {
        let _ = writeln!(
            out,
            "   applied a test perturbation: {p}; the replay applies it too"
        );
    }
}

/// Compares two bundles; see the module's documentation.
pub fn compare(a: &Bundle, b: &Bundle, inputs: &Inputs) -> Result<Report, String> {
    let mut r = Report::default();
    let mut out = String::new();
    describe(&mut out, 'A', a);
    describe(&mut out, 'B', b);
    if let Some(f) = a.start.mismatch(&b.start) {
        let _ = writeln!(
            out,
            "The start values differ: the {f}. Nothing more compares."
        );
        r.differs = true;
        r.text = out;
        return Ok(r);
    }
    let slots = a.start.slots.clone();
    let slot_of_house = move |h: u8| slots.iter().position(|s| s.house == h).map(|i| i as u8);
    let mut ga = load(&a.base, inputs, 'A')?;
    let mut gb = load(&b.base, inputs, 'B')?;
    for (g, s, side) in [(&ga, &a.base, 'A'), (&gb, &b.base, 'B')] {
        if hash_of(g) != s.hash {
            let _ = writeln!(
                out,
                "{side}'s snapshot loads to another hash than the one it was taken with: this build reads it otherwise."
            );
        }
    }
    // Bring the older snapshot forward to the newer.
    let start = a.base.ran.max(b.base.ran);
    for (g, x, side) in [(&mut ga, a, 'A'), (&mut gb, b, 'B')] {
        for t in x.base.ran..start {
            let rec = x
                .tick(t)
                .ok_or_else(|| format!("bundle {side} lacks tick {t}"))?;
            run_tick(g, rec, x.perturb);
        }
    }
    if a.base.ran != b.base.ran {
        let _ = writeln!(
            out,
            "The older snapshot is replayed forward to the newer: both from tick {start}."
        );
    }
    let p = |side: Side| match side {
        Side::A => a.perturb,
        Side::B => b.perturb,
    };
    if hash_of(&ga) != hash_of(&gb) {
        let _ = writeln!(
            out,
            "The games differ already before tick {start}, where both bundles begin; the state then, field by field:"
        );
        r.differs = true;
        r.diff_tick = start.checked_sub(1);
        r.diff = diff_games(&ga, &gb, &inputs.rules);
        write_diff(&mut out, &r.diff, &slot_of_house);
        r.text = out;
        return Ok(r);
    }
    let end = a.end().min(b.end());
    let mut replay_diff = None;
    for t in start..end {
        let (Some(ra), Some(rb)) = (a.tick(t), b.tick(t)) else {
            break;
        };
        let orders = |x: &TickRecord| -> Vec<(u8, u8, glasnost_sim::orders::Order)> {
            x.orders
                .iter()
                .map(|i| (i.slot, i.house, i.order.clone()))
                .collect()
        };
        if r.orders_tick.is_none() && orders(ra) != orders(rb) {
            r.orders_tick = Some(t);
        }
        if r.first_tick.is_none() && ra.hash != rb.hash {
            r.first_tick = Some(t);
        }
        if r.replay_tick.is_none() {
            run_tick(&mut ga, ra, p(Side::A));
            run_tick(&mut gb, rb, p(Side::B));
            let (ha, hb) = (hash_of(&ga), hash_of(&gb));
            if r.unreproduced[0].is_none() && ha != ra.hash {
                r.unreproduced[0] = Some(t);
            }
            if r.unreproduced[1].is_none() && hb != rb.hash {
                r.unreproduced[1] = Some(t);
            }
            if ha != hb {
                r.replay_tick = Some(t);
                replay_diff = Some(diff_games(&ga, &gb, &inputs.rules));
            }
        }
    }
    let last = end.saturating_sub(1);
    match r.first_tick {
        Some(t) => {
            let _ = writeln!(
                out,
                "The first differing tick: {t}. The peers' own state hashes after tick {t} differ{}.",
                if t > start {
                    format!("; they agree through tick {}", t - 1)
                } else {
                    String::new()
                }
            );
        }
        None => {
            let _ = writeln!(
                out,
                "The peers' own state hashes agree on every tick both bundles hold ({start} to {last})."
            );
        }
    }
    if let Some(t) = r.orders_tick {
        let _ = writeln!(
            out,
            "The merged orders differ first at tick {t}: every peer must issue the same orders (a lockstep fault)."
        );
    }
    let checked = r.replay_tick.unwrap_or(last);
    for (i, side) in ['A', 'B'].into_iter().enumerate() {
        match r.unreproduced[i] {
            Some(t) => {
                let _ = writeln!(
                    out,
                    "{side}'s own hash after tick {t} is not what this build computes from its snapshot and orders: its game changed outside the rules this build runs (another build, a platform difference, a memory fault)."
                );
            }
            None => {
                let _ = writeln!(
                    out,
                    "{side}'s own hashes are what this build computes from its snapshot and orders, through tick {checked}."
                );
            }
        }
    }
    if let (Some(t), Some(d)) = (r.replay_tick, replay_diff) {
        let _ = writeln!(
            out,
            "The replays differ after tick {t}{}. The games after tick {t}, field by field:",
            if r.first_tick == Some(t) {
                ", the first differing tick"
            } else {
                ""
            }
        );
        r.diff_tick = Some(t);
        r.diff = d;
        write_diff(&mut out, &r.diff, &slot_of_house);
    } else if r.first_tick.is_some() {
        match (&a.after, &b.after) {
            (Some(sa), Some(sb)) if sa.ran == sb.ran => {
                let (xa, xb) = (load(sa, inputs, 'A')?, load(sb, inputs, 'B')?);
                let t = sa.ran - 1;
                let _ = writeln!(
                    out,
                    "The replays agree, so the inputs do not explain the difference. Each peer's own game after tick {t} (its snapshot at the mismatched checkpoint), field by field:"
                );
                r.diff_tick = Some(t);
                r.diff = diff_games(&xa, &xb, &inputs.rules);
                write_diff(&mut out, &r.diff, &slot_of_house);
            }
            _ => {
                let _ = writeln!(
                    out,
                    "The replays agree, and the bundles hold no snapshots of both peers' own games at one checkpoint: no field diff."
                );
            }
        }
    }
    r.differs = r.first_tick.is_some() || r.replay_tick.is_some() || r.orders_tick.is_some();
    if !r.differs {
        let _ = writeln!(out, "No difference.");
    }
    r.text = out;
    Ok(r)
}

/// The inputs for a bundle: the synthetic fixture's when its rules are the
/// report's, else the install's in `game_dir` (local only).
pub fn inputs_for(b: &Bundle, game_dir: Option<&Path>) -> Result<Inputs, String> {
    if let Some(i) = Inputs::synthetic(&b.start.rules_hash) {
        return Ok(i);
    }
    match game_dir {
        Some(dir) => Inputs::install(dir, &b.start.map_hash, &b.start.rules_hash),
        None => Err(format!(
            "the report's rules ({}) are not the synthetic fixture's: set {} to the install it was played with",
            b.start.rules_hash,
            crate::local_lane::GAME_DIR_VAR
        )),
    }
}

/// Reads a bundle file, with its digest.
pub fn read(path: &Path) -> Result<(Bundle, Hash), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let b = Bundle::from_bytes(&bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((b, Bundle::digest(&bytes)))
}

/// `glasnost-headless desync A B`: the report's text and whether the two
/// differ.
pub fn run(a: &Path, b: &Path, game_dir: Option<&Path>) -> Result<(String, bool), String> {
    let (ba, da) = read(a)?;
    let (bb, db) = read(b)?;
    let inputs = inputs_for(&ba, game_dir)?;
    let r = compare(&ba, &bb, &inputs)?;
    let mut text = format!(
        "A = {} (digest {})\nB = {} (digest {})\n",
        a.display(),
        short(&da),
        b.display(),
        short(&db)
    );
    text.push_str(&r.text);
    Ok((text, r.differs))
}
