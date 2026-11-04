//! M-0021's test pairing (M8 step A3 intake 93, Q-1714): a two-group fight
//! with the shape of Q-1714's pairing, played in our engine with the
//! record's setup and judged by its Procedure, one verdict per row of its
//! Result.
//!
//! # The setup (measure: M-0021, Setup)
//!
//! - **Ground**: the arena map of M-0001 with M-0001's sha256 (the same
//!   ground [`oracle::stock_inputs`] reads), the oracle arena's game start
//!   ([`oracle::setup_with`]): a skirmish at the fastest speed, 0 credits,
//!   superweapons, crates and the short game off, bases on, two hard
//!   computer houses (the easy row through the handicap, S-0412 claims 4
//!   and 5) that are not allied, and the idle human player house.
//! - **Houses**: F is always the house of the Americans and G that of the
//!   Russians ([`oracle::COUNTRIES`]); *F on side B* puts F's group on the
//!   side-B cells, facing west, and G's on the side-A cells, so the houses
//!   do not change with the side ([`oracle::SetupOptions::houses`]).
//! - **Groups**: [`GROUP`] vehicles a side on M-0001's cells and facings
//!   for 16 units, each in the hunt mission, no order given. Every vehicle
//!   that the human player house does not own and every object of the
//!   neutral house are taken off before the groups.
//! - **Test types** ([`test_ruleset`]): both copies of one tank type of our
//!   own choice ([`TestBase`]: the record keeps its own base `local`, and
//!   its Notes say a comparison builds the types on a tank type of its own
//!   choice), with the fields the record sets; every weapon a copy of the
//!   base tank's primary weapon, every warhead a copy of that weapon's
//!   warhead, every projectile a copy of the first invisible projectile
//!   type in name order. The names of our test types are our own.
//! - **Parts** ([`Part`]): A (fragment count 3, strength 60000, seeds 0 to
//!   2, both sides, [`LIMIT`] ticks), A0 (the control without fragments,
//!   seed 0), B (strength 400, seeds 0 to 2, until one side has no vehicle
//!   left, at most [`LIMIT_B`] ticks), and the control set (A with the easy
//!   row's rate-of-fire factor set to 1, seed 0). Seed numbers 0 to 2 are
//!   the record's game seeds 1000 to 1002 ([`oracle::game_seed`]).
//!
//! # What we cannot reproduce (docs/development.md, *Arena mode*)
//!
//! The human player is the arena's idle house with its base unit, not
//! M-0021's British observer with one building; which group is created
//! first is not in the record ([`F_FIRST`], our choice); the gattling
//! type's numbered weapons beyond the two stage weapons are not in the
//! record ([`test_ruleset`]). No row the record marks `exact` for the
//! per-shot and stage results depends on these, by the record's Notes;
//! the first-shot distances and part B's outcome may (they follow the
//! vehicles' movement).
//!
//! # Local values
//!
//! The record keeps every count of shots, every hit's health and the
//! houses' factors `local` (Notes). The report holds them, so it goes only
//! to the git-ignored local lane; never commit it.
//!
//! Tooling only: the recorder reads state and presentation effects after
//! each tick and touches no simulation state.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::data::{WarheadId, WeaponId};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::SimRules;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

use crate::oracle::{self, Ground, SIDE_HOUSES, SetupOptions, Side};

/// The record's ID.
pub const ID: &str = "M-0021";

/// The arena record whose map M-0021 plays on (measure: M-0021, Setup,
/// Map: the arena map of M-0001, with M-0001's sha256).
pub const MAP_RECORD: &str = "M-0001";

/// Vehicles a side (measure: M-0021, Setup, Groups: 16 vehicles a side).
pub const GROUP: u32 = 16;

/// The seed numbers of the record's game seeds 1000 to 1002 (measure:
/// M-0021, Setup, Parts and seeds; our arena seeds the stream with 1000 +
/// the seed number, S-0216 claim 8).
pub const SEEDS: [u32; 3] = [0, 1, 2];

/// Parts A, A0 and the control: the tick limit (measure: M-0021, Setup,
/// Parts and seeds: a tick limit of 600).
pub const LIMIT: i32 = 600;

/// Part B: the tick limit, which the record's runs did not reach (measure:
/// M-0021, Setup, Parts and seeds: 3000).
pub const LIMIT_B: i32 = 3000;

/// The end of the shot-gap window: ticks 0 to 299 (measure: M-0021,
/// Procedure step 7).
pub const GAP_TICKS: i32 = 300;

/// The strength of both types in parts A, A0 and the control, and in part
/// B (measure: M-0021, Setup, Test types).
pub const STRENGTH: [i32; 2] = [60000, 400];

/// Speed, turn rate and sight of both types (measure: M-0021, Setup, Test
/// types: speed 6, turn rate 8, sight 8 cells).
pub const SPEED: i32 = 6;
/// The turn rate (measure: M-0021, Setup, Test types).
pub const TURN_RATE: i32 = 8;
/// Sight in cells (measure: M-0021, Setup, Test types).
pub const SIGHT: i32 = 8;

/// Every test weapon's speed (measure: M-0021, Setup, Test types: a weapon
/// speed of 100).
pub const WEAPON_SPEED: i32 = 100;

/// F's shot weapon: damage, rate of fire in ticks, range in cells (measure:
/// M-0021, Setup, Test types: damage 20, rate of fire 60 ticks, range 6).
pub const F_WEAPON: (i32, i32, i32) = (20, 60, 6);

/// The fragment count of F's projectile (measure: M-0021, Setup, Test
/// types: a fragment count of 3).
pub const FRAGMENTS: i32 = 3;

/// The fragment weapon: damage and range in cells; its rate of fire is the
/// copied weapon's (measure: M-0021, Setup, Test types: damage 7, range 3).
pub const FRAGMENT_WEAPON: (i32, i32) = (7, 3);

/// G's two stage thresholds, normal and elite alike (measure: M-0021,
/// Setup, Test types: thresholds 30 and 60).
pub const THRESHOLDS: [i32; 2] = [30, 60];

/// G's spin-up and spin-down amounts (measure: M-0021, Setup, Test types:
/// a spin-up rate of 1 and a spin-down rate of 1).
pub const RATES: (i32, i32) = (1, 1);

/// G's turret count (measure: M-0021, Setup, Test types: a turret count
/// of 1).
pub const G_TURRETS: i32 = 1;

/// G's stage 0 and stage 1 weapons: damage, rate of fire in ticks, range in
/// cells (measure: M-0021, Setup, Test types).
pub const G_WEAPONS: [(i32, i32, i32); 2] = [(5, 20, 5), (10, 10, 5)];

/// The control set's easy-row rate-of-fire factor (measure: M-0021, Setup,
/// Parts and seeds: set to 1, a test value).
pub const CONTROL_ROF: &str = "1";

/// The rings around the target's cell that the candidate cells are read
/// from: rings 1 to 3, the fragment range *k* of 3 (measure: M-0021,
/// Procedure step 4; S-0318 claim 3).
pub const RINGS: i32 = 3;

/// Whether F's group is created and placed first. The record does not say
/// (M-0021, Setup, Groups); our choice, as the row type of the oracle
/// arena goes first (S-0216 claim 8, *Order*).
pub const F_FIRST: bool = true;

// The record's values (measure: M-0021, Result).

/// The centre distance in leptons at F's first shot and at G's (measure:
/// M-0021, Result, First shots: 1527 and 1280, exact).
pub const FIRST_SHOT_DISTANCE: [i64; 2] = [1527, 1280];

/// The values read in the tick before and the tick of a rise from stage 0
/// to stage 1 (measure: M-0021, Result, Stages: 30, then 31).
pub const RISE: (i32, i32) = (30, 31);

/// The largest value read (measure: M-0021, Result, Stages: 60).
pub const LARGEST: i32 = 60;

/// The values read in the tick before and the tick of a fall from stage 1
/// to stage 0 (measure: M-0021, Result, Stages: part B, 30, then 29).
pub const FALL: (i32, i32) = (30, 29);

/// Part B's G vehicles dead per run, by F's side and seed number (measure:
/// M-0021, Result, Part B: every run left G with vehicles and all 16 F
/// vehicles dead).
pub const PART_B_G_DEAD: [(Side, u32, u32); 6] = [
    (Side::A, 0, 4),
    (Side::A, 1, 4),
    (Side::A, 2, 7),
    (Side::B, 0, 7),
    (Side::B, 1, 6),
    (Side::B, 2, 7),
];

/// Part B: the ticks by which every G vehicle first read stage 1 before
/// the run's first death, over all 96 (measure: M-0021, Result, Part B:
/// 54 to 182).
pub const STAGE_LEAD: (i32, i32) = (54, 182);

/// Our names of the test types, weapons, projectiles and warheads.
pub mod names {
    /// The fragment side's type.
    pub const F: &str = "M21F";
    /// The gattling side's type.
    pub const G: &str = "M21G";
    /// F's shot weapon.
    pub const F_SHOT: &str = "M21FSHOT";
    /// The fragment weapon.
    pub const FRAGMENT: &str = "M21FRAG";
    /// G's stage weapons.
    pub const G_STAGE: [&str; 2] = ["M21G0", "M21G1"];
    /// F's shot projectile.
    pub const F_PROJECTILE: &str = "M21FPROJ";
    /// The other weapons' projectile.
    pub const PROJECTILE: &str = "M21PROJ";
    /// F's shot warhead.
    pub const F_WARHEAD: &str = "M21FWH";
    /// The fragment warhead.
    pub const FRAGMENT_WARHEAD: &str = "M21SWH";
    /// G's warhead.
    pub const G_WARHEAD: &str = "M21GWH";
}

/// The base types our test types copy (our choice; M-0021 Notes,
/// *Reproducing it*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestBase {
    /// The tank type both test types copy.
    pub tank: String,
}

/// Our default base tank on the install's rules: a tank of M-0003's
/// pairings with one cannon (our choice). `GLASNOST_M0021_BASE` overrides it.
pub const DEFAULT_BASE: &str = "MTNK";

/// A part of the record (measure: M-0021, Setup, Parts and seeds).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Part {
    /// Fragment count 3, nothing dies.
    A,
    /// The control without fragments.
    A0,
    /// Strength 400, to the end.
    B,
    /// Part A with the easy row's rate-of-fire factor at 1.
    Control,
}

impl Part {
    /// Every part, in the record's order.
    pub const ALL: [Part; 4] = [Part::A, Part::A0, Part::B, Part::Control];

    /// The part's name in the report.
    pub fn text(self) -> &'static str {
        match self {
            Part::A => "A",
            Part::A0 => "A0",
            Part::B => "B",
            Part::Control => "control",
        }
    }

    /// The seed numbers it is played with (measure: M-0021, Setup, Parts
    /// and seeds: A and B seeds 1000 to 1002, A0 and the control seed 1000).
    pub fn seeds(self) -> &'static [u32] {
        match self {
            Part::A | Part::B => &SEEDS,
            Part::A0 | Part::Control => &SEEDS[..1],
        }
    }

    /// The tick limit.
    pub fn limit(self) -> i32 {
        match self {
            Part::B => LIMIT_B,
            _ => LIMIT,
        }
    }

    /// The strength of both types.
    pub fn strength(self) -> i32 {
        match self {
            Part::B => STRENGTH[1],
            _ => STRENGTH[0],
        }
    }
}

/// The runs of the record: each part with F on side A and on side B, over
/// its seeds (16 runs).
pub fn runs() -> Vec<(Part, Side, u32)> {
    let mut out = Vec::new();
    for p in Part::ALL {
        for side in [Side::A, Side::B] {
            for &s in p.seeds() {
                out.push((p, side, s));
            }
        }
    }
    out
}

fn d(s: impl Into<String>) -> Decimal {
    Decimal(s.into())
}

/// A copy of `base` as a test warhead (measure: M-0021, Setup, Test types:
/// every armour class at 100 %, no cell spread, no explosion animation, no
/// wall or wood damage).
fn test_warhead(base: &WarheadType) -> WarheadType {
    WarheadType {
        // measure: M-0021 (every armour class at 100 %: eleven versus items of 1, S-0305 claim 1)
        verses: Some(vec![d("1"); 11]),
        // measure: M-0021 (no cell spread: it damages a single object)
        cell_spread: Some(d("0")),
        // measure: M-0021 (no explosion animation)
        anim_list: Some(Vec::new()),
        // measure: M-0021 (no wall or wood damage)
        wall: Some(false),
        wood: Some(false),
        ..base.clone()
    }
}

/// A copy of `base` as a test projectile, naming `fragments` (the fragment
/// weapon and count) or none.
fn test_projectile(base: &ProjectileType, fragments: Option<(&str, i32)>) -> ProjectileType {
    ProjectileType {
        // measure: M-0021 (F's projectile names the fragment weapon with a fragment count of 3; every other projectile, and F's in part A0, names none)
        shrapnel_weapon: fragments.map(|(w, _)| w.to_owned()),
        shrapnel_count: fragments.map(|(_, n)| n),
        ..base.clone()
    }
}

/// A copy of `base` as a test weapon: damage, rate of fire, range in
/// cells, projectile and warhead set, weapon speed [`WEAPON_SPEED`] and no
/// fire animation (measure: M-0021, Setup, Test types).
fn test_weapon(
    base: &WeaponType,
    (damage, rof, range): (i32, Option<i32>, i32),
    projectile: &str,
    warhead: &str,
) -> WeaponType {
    WeaponType {
        damage: Some(damage),
        rof: rof.or(base.rof),
        range: Some(d(range.to_string())),
        projectile: Some(projectile.to_owned()),
        warhead: Some(warhead.to_owned()),
        // measure: M-0021 (a weapon speed of 100)
        speed: Some(WEAPON_SPEED),
        // measure: M-0021 (no fire animation)
        anim: Some(Vec::new()),
        ..base.clone()
    }
}

/// A copy of `base` as a test vehicle type with the fields both test types
/// share (measure: M-0021, Setup, Test types: strength, speed 6, turn rate
/// 8, no acceleration, sight 8 cells, opportunity fire off, no veteran or
/// elite abilities, no debris).
fn test_vehicle(base: &UnitType, strength: i32) -> UnitType {
    let mut u = base.clone();
    u.strength = Some(strength);
    u.speed = Some(SPEED);
    u.rot = Some(TURN_RATE);
    // measure: M-0021 (no acceleration)
    u.accelerates = Some(false);
    u.sight = Some(SIGHT);
    // measure: M-0021 (opportunity fire off)
    u.opportunity_fire = Some(false);
    // measure: M-0021 (no veteran or elite abilities)
    u.veteran_abilities = Vec::new();
    u.elite_abilities = Vec::new();
    // measure: M-0021 (no debris)
    u.combat.max_debris = Some(0);
    u.combat.min_debris = Some(0);
    u.combat.debris_types = None;
    u
}

/// The first invisible projectile type of `r` in name order that can hit
/// ground targets (its anti-ground flag not off), has neither the
/// anti-air nor the flak-scatter flag, and names no fragment, airburst or
/// cluster weapon (measure: M-0021, Setup, Test types: a copy of a stock
/// invisible projectile type, S-0306 claim 9; which one is our choice, as
/// the record keeps its own `local`). `GLASNOST_M0021_PROJECTILE` names
/// another.
fn invisible_projectile(r: &Ruleset) -> Option<(&String, &ProjectileType)> {
    if let Ok(name) = std::env::var("GLASNOST_M0021_PROJECTILE") {
        return r.sim.projectiles.get_key_value(&name);
    }
    r.sim.projectiles.iter().find(|(_, p)| {
        p.inviso == Some(true)
            && p.ag != Some(false)
            && p.aa != Some(true)
            && p.flak_scatter != Some(true)
            && p.shrapnel_weapon.is_none()
            && p.airburst_weapon.is_none()
            && p.cluster.unwrap_or(0) <= 1
    })
}

/// What the test types were built from, for the local report.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Built {
    /// The base tank, its primary weapon, that weapon's warhead and the
    /// invisible projectile copied.
    pub from: [String; 4],
    /// The copied weapon's burst field (absent: 1).
    pub burst: Option<i32>,
    /// The base tank's turret count.
    pub turrets: Option<i32>,
}

/// `base` with M-0021's test types for `part` added (measure: M-0021,
/// Setup, Test types and Parts). F carries its shot weapon as its primary
/// and elite primary weapon and the fragment weapon as its secondary and
/// elite secondary, only so that its section is read; F keeps the base
/// tank's turret count and fills slots 0 and 1 from those fields. G is a
/// gattling type with two stages, a turret count of 1 and numbered slots
/// 0 to 3 holding the stage 0, stage 0, stage 1 and stage 1 weapons: stage
/// *s* fires slot 2*s* (S-0308 claim 7), and S-0308 table P row 3 needs
/// slot 1 filled; which weapons the record's file put in slots 1 and 3 it
/// does not say (our choice). The new types go at the end of the vehicle
/// registry, so no other type's index moves.
pub fn test_ruleset(
    base: &Ruleset,
    from: &TestBase,
    part: Part,
) -> Result<(Ruleset, Built), String> {
    let mut r = base.clone();
    let tank = r
        .sim
        .vehicles
        .get(&from.tank)
        .cloned()
        .ok_or_else(|| format!("no vehicle type {}", from.tank))?;
    let wname = tank
        .primary
        .clone()
        .or_else(|| {
            tank.combat
                .weapon_slots
                .as_ref()
                .and_then(|s| s.first().cloned())
        })
        .filter(|n| !n.is_empty())
        .ok_or_else(|| format!("{} has no primary weapon", from.tank))?;
    let weapon = r
        .sim
        .weapons
        .get(&wname)
        .cloned()
        .ok_or_else(|| format!("no weapon {wname}"))?;
    let whname = weapon
        .warhead
        .clone()
        .ok_or_else(|| format!("{wname} has no warhead"))?;
    let warhead = r
        .sim
        .warheads
        .get(&whname)
        .cloned()
        .ok_or_else(|| format!("no warhead {whname}"))?;
    let (pname, projectile) = invisible_projectile(&r)
        .map(|(n, p)| (n.clone(), p.clone()))
        .ok_or("no invisible projectile type")?;
    let built = Built {
        from: [from.tank.clone(), wname, whname, pname],
        burst: weapon.burst,
        turrets: tank.turret_count,
    };
    let s = &mut r.sim;
    for n in [names::F_WARHEAD, names::FRAGMENT_WARHEAD, names::G_WARHEAD] {
        s.warheads.insert(n.into(), test_warhead(&warhead));
    }
    // measure: M-0021 (part A0: F's projectile names no fragment weapon and no count)
    let fragments = (part != Part::A0).then_some((names::FRAGMENT, FRAGMENTS));
    s.projectiles.insert(
        names::F_PROJECTILE.into(),
        test_projectile(&projectile, fragments),
    );
    s.projectiles
        .insert(names::PROJECTILE.into(), test_projectile(&projectile, None));
    // measure: M-0021 (F: damage 20, rate of fire 60 ticks, range 6 cells)
    s.weapons.insert(
        names::F_SHOT.into(),
        test_weapon(
            &weapon,
            (F_WEAPON.0, Some(F_WEAPON.1), F_WEAPON.2),
            names::F_PROJECTILE,
            names::F_WARHEAD,
        ),
    );
    // measure: M-0021 (the fragment weapon: damage 7, range 3 cells, on an invisible projectile that names no fragment weapon)
    s.weapons.insert(
        names::FRAGMENT.into(),
        test_weapon(
            &weapon,
            (FRAGMENT_WEAPON.0, None, FRAGMENT_WEAPON.1),
            names::PROJECTILE,
            names::FRAGMENT_WARHEAD,
        ),
    );
    for (i, (dmg, rof, range)) in G_WEAPONS.into_iter().enumerate() {
        // measure: M-0021 (stage 0: damage 5, rate of fire 20 ticks, range 5 cells; stage 1: damage 10, rate of fire 10 ticks, range 5 cells)
        s.weapons.insert(
            names::G_STAGE[i].into(),
            test_weapon(
                &weapon,
                (dmg, Some(rof), range),
                names::PROJECTILE,
                names::G_WARHEAD,
            ),
        );
    }
    let mut f = test_vehicle(&tank, part.strength());
    // measure: M-0021 (F: primary and elite primary weapon its shot weapon, the fragment weapon as its secondary and elite secondary)
    f.primary = Some(names::F_SHOT.into());
    f.elite_primary = Some(names::F_SHOT.into());
    f.secondary = Some(names::FRAGMENT.into());
    f.elite_secondary = Some(names::FRAGMENT.into());
    f.combat.weapon_slots = None;
    f.combat.elite_weapon_slots = None;
    f.combat.is_gattling = Some(false);
    let mut g = test_vehicle(&tank, part.strength());
    let [g0, g1] = names::G_STAGE.map(String::from);
    g.primary = Some(g0.clone());
    g.elite_primary = Some(g0.clone());
    g.secondary = Some(g0.clone());
    g.elite_secondary = Some(g0.clone());
    let slots = vec![g0.clone(), g0, g1.clone(), g1];
    g.combat.weapon_slots = Some(slots.clone());
    g.combat.elite_weapon_slots = Some(slots);
    // measure: M-0021 (G: two firing stages, thresholds 30 and 60 normal and elite alike, spin-up and spin-down rates of 1, a turret count of 1)
    g.turret_count = Some(G_TURRETS);
    g.combat.is_gattling = Some(true);
    g.combat.weapon_stages = Some(THRESHOLDS.len() as i32);
    g.combat.stage_thresholds = Some(THRESHOLDS.to_vec());
    g.combat.elite_stage_thresholds = Some(THRESHOLDS.to_vec());
    g.combat.rate_up = Some(RATES.0);
    g.combat.rate_down = Some(RATES.1);
    for (n, u) in [(names::F, f), (names::G, g)] {
        s.vehicles.insert(n.into(), u);
        r.registries
            .entry("vehicles".into())
            .or_default()
            .push(n.into());
    }
    if part == Part::Control {
        // measure: M-0021 (the control set: the easy difficulty row's rate-of-fire factor set to 1)
        r.sim.difficulty.entry("easy".into()).or_default().rof = Some(d(CONTROL_ROF));
    }
    Ok((r, built))
}

/// The ground of one part: `ground` with the test types of `part` in its
/// rules.
pub fn part_ground(
    ground: &Ground,
    base: &Ruleset,
    hash: &str,
    from: &TestBase,
    part: Part,
) -> Result<(Ground, Built), String> {
    let (r, built) = test_ruleset(base, from, part)?;
    let mut g = ground.clone();
    g.rules = Arc::new(SimRules::new(&r, &format!("{hash}+{ID}-{}", part.text())));
    Ok((g, built))
}

// ---------------------------------------------------------------------------
// The recording (measure: M-0021, Procedure: every shot, every hit, every
// vehicle's position each tick, each G vehicle's spin value and stage)

/// One shot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShotRec {
    /// The tick.
    pub tick: i32,
    /// The firer.
    pub firer: ObjectId,
    /// Its target.
    pub target: Target,
    /// The weapon.
    pub weapon: WeaponId,
}

/// One hit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitRec {
    /// The tick.
    pub tick: i32,
    /// The object hit.
    pub id: ObjectId,
    /// Its warhead.
    pub warhead: Option<WarheadId>,
    /// Its attacker.
    pub attacker: Option<ObjectId>,
    /// Health before and after.
    pub health: (i32, i32),
}

/// One run's recording.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    /// F's vehicles and G's, in creation order.
    pub f: Vec<ObjectId>,
    /// G's vehicles.
    pub g: Vec<ObjectId>,
    /// Every shot of the two groups.
    pub shots: Vec<ShotRec>,
    /// Every hit on a vehicle of the two groups.
    pub hits: Vec<HitRec>,
    /// Each tick's positions of the vehicles in play (alive and on the
    /// map), from tick 0 (the state before the first tick).
    pub positions: BTreeMap<i32, BTreeMap<ObjectId, Pos>>,
    /// Each tick's spin value and stage of every G vehicle in play.
    pub spins: BTreeMap<i32, BTreeMap<ObjectId, (i32, i32)>>,
    /// The last tick played.
    pub end: i32,
    /// The firers whose projectiles (a shot's or a fragment's) are still in
    /// flight at the run's end.
    pub in_flight: BTreeSet<ObjectId>,
    /// F's shot weapon, the fragment weapon and G's stage weapons.
    pub weapons: [WeaponId; 4],
    /// F's shot warhead and the fragment warhead.
    pub warheads: [WarheadId; 2],
    /// The full delay's base with its draw at 0, trunc(RZ₅₃(*R* × *h*)), of
    /// F's shot weapon and G's two stage weapons, with the firing house's
    /// rate-of-fire factor *h* (S-0301 claim 2 and F1).
    pub expected_gap: [i32; 3],
}

fn in_play(w: &World, id: ObjectId) -> Option<Pos> {
    w.obj(id)
        .filter(|o| o.on_map && o.health > 0)
        .map(|o| o.pos)
}

/// Sets up and plays one run of `part` with F's group on `f_side` and seed
/// number `seed`, recording it.
pub fn play(ground: &Ground, part: Part, f_side: Side, seed: u32) -> Result<Recording, String> {
    play_to(ground, part, f_side, seed, part.limit())
}

/// [`play`] to tick `limit` in place of the part's own limit (the CI tests'
/// shorter runs).
pub fn play_to(
    ground: &Ground,
    part: Part,
    f_side: Side,
    seed: u32,
    limit: i32,
) -> Result<Recording, String> {
    let rules = ground.rules.clone();
    let weapon = |n: &str| {
        rules
            .combat
            .weapon_index
            .get(n)
            .copied()
            .ok_or_else(|| format!("no weapon {n}"))
    };
    let weapons = [
        weapon(names::F_SHOT)?,
        weapon(names::FRAGMENT)?,
        weapon(names::G_STAGE[0])?,
        weapon(names::G_STAGE[1])?,
    ];
    let warhead_of = |w: WeaponId| {
        rules
            .combat
            .weapon(w)
            .warhead
            .ok_or_else(|| "a test weapon without a warhead".to_owned())
    };
    let warheads = [warhead_of(weapons[0])?, warhead_of(weapons[1])?];
    // F is the Americans' house and G the Russians', whichever side F's group stands on.
    // measure: M-0021 (Setup, Houses: the houses do not change with the side)
    let g_side = match f_side {
        Side::A => Side::B,
        Side::B => Side::A,
    };
    let mut houses = SIDE_HOUSES;
    houses[f_side.index()] = SIDE_HOUSES[0];
    houses[g_side.index()] = SIDE_HOUSES[1];
    let mut types = [names::F; 2];
    types[g_side.index()] = names::G;
    let first = if F_FIRST { f_side } else { g_side };
    let opts = SetupOptions {
        count: Some(GROUP),
        houses,
        clear_neutral: true,
    };
    let a = oracle::setup_with(ground, types, first, seed, &opts)?;
    let mut w = a.world;
    let f = a.units[f_side.index()].clone();
    let g = a.units[g_side.index()].clone();
    let mode = rules.mode;
    let gap = |house: HouseId, rof: i32| {
        let h = w.houses[usize::from(house.0)].combat.rof_factor;
        // spec: S-0301 F1 (the full delay's base with its draw at 0: trunc(RZ₅₃(R × h)))
        F64::from_i64(i64::from(rof), mode)
            .mul(h, mode)
            .to_i32_common()
    };
    let expected_gap = [
        gap(SIDE_HOUSES[0], F_WEAPON.1),
        gap(SIDE_HOUSES[1], G_WEAPONS[0].1),
        gap(SIDE_HOUSES[1], G_WEAPONS[1].1),
    ];
    let all: Vec<ObjectId> = f.iter().chain(&g).copied().collect();
    let mut rec = Recording {
        f: f.clone(),
        g: g.clone(),
        weapons,
        warheads,
        expected_gap,
        ..Default::default()
    };
    let snap = |w: &World, rec: &mut Recording| {
        let t = w.tick;
        let pos = all
            .iter()
            .filter_map(|&id| in_play(w, id).map(|p| (id, p)))
            .collect();
        rec.positions.insert(t, pos);
        let spins = g
            .iter()
            .filter(|&&id| in_play(w, id).is_some())
            .map(|&id| {
                let s = glasnost_sim::gattling::spin(w, id);
                (id, (s.value, s.stage))
            })
            .collect();
        rec.spins.insert(t, spins);
    };
    snap(&w, &mut rec);
    let alive = |w: &World, ids: &[ObjectId]| ids.iter().any(|&id| in_play(w, id).is_some());
    while w.tick < limit {
        glasnost_sim::tick::step(&mut w);
        let t = w.tick;
        for e in std::mem::take(&mut w.effects) {
            match e {
                Effect::Shot {
                    firer,
                    weapon,
                    target,
                    ..
                } if all.contains(&firer) => rec.shots.push(ShotRec {
                    tick: t,
                    firer,
                    target,
                    weapon,
                }),
                Effect::Hit {
                    id,
                    attacker,
                    warhead,
                    health,
                    ..
                } if all.contains(&id) => rec.hits.push(HitRec {
                    tick: t,
                    id,
                    warhead,
                    attacker,
                    health,
                }),
                _ => {}
            }
        }
        snap(&w, &mut rec);
        rec.end = t;
        // measure: M-0021 (part B: each run until one side has no vehicle left)
        if part == Part::B && (!alive(&w, &f) || !alive(&w, &g)) {
            break;
        }
    }
    rec.in_flight = w
        .projectiles
        .values()
        .filter_map(|p| p.firer.or(p.attacker))
        .collect();
    Ok(rec)
}

/// Plays `jobs` on up to `threads` threads (at most 4 games at once is the
/// caller's limit to set).
pub fn play_all(
    grounds: &BTreeMap<Part, Ground>,
    jobs: &[(Part, Side, u32)],
    threads: usize,
) -> Vec<Result<Recording, String>> {
    let next = AtomicUsize::new(0);
    let results: Vec<std::sync::Mutex<Option<Result<Recording, String>>>> =
        jobs.iter().map(|_| std::sync::Mutex::new(None)).collect();
    std::thread::scope(|s| {
        for _ in 0..threads.max(1) {
            s.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(&(part, side, seed)) = jobs.get(i) else {
                        break;
                    };
                    let r = match grounds.get(&part) {
                        Some(g) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            play(g, part, side, seed)
                        }))
                        .unwrap_or_else(|_| Err("panic".into())),
                        None => Err(format!("no ground for part {}", part.text())),
                    };
                    if let Ok(mut slot) = results[i].lock() {
                        *slot = Some(r);
                    }
                }
            });
        }
    });
    results
        .into_iter()
        .map(|m| {
            m.into_inner()
                .ok()
                .flatten()
                .unwrap_or_else(|| Err("not run".into()))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The judging (measure: M-0021, Procedure steps 1 to 9)

/// One judged F shot (Procedure steps 4 and 5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Judged {
    /// The tick and the target.
    pub tick: i32,
    /// The target *X*.
    pub target: ObjectId,
    /// *c*: the candidate cells in rings 1 to 3 around *X*'s cell.
    pub c: usize,
    /// A cell at Chebyshev distance 1 to 3 from *X*'s cell held both an F
    /// and a G vehicle.
    pub mixed_cell: bool,
    /// Hits by F's shot warhead in the window naming the firer: on *X*,
    /// and on any other object.
    pub shot_hits: (usize, usize),
    /// The fragment-warhead hits in the window naming the firer: the
    /// object hit and the health it took.
    pub fragment_hits: Vec<(ObjectId, i32)>,
    /// Of those, the hits on F vehicles.
    pub fragments_on_f: usize,
    /// The distinct G vehicles other than *X* those hits are on.
    pub fragment_g: BTreeSet<ObjectId>,
    /// Objects damaged in the window by hits naming the firer.
    pub damaged: BTreeSet<ObjectId>,
    /// The cells of the G vehicles its fragments hit match the first 3
    /// candidate cells in T1's order: read at *t*, and at any of *t* − 1,
    /// *t* and *t* + 1.
    pub first_three: (bool, bool),
}

/// What one run shows (Procedure steps 1 to 9).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Findings {
    /// F fired a weapon other than its primary (step 1: the run is void).
    pub void: bool,
    /// F's shots not judged (step 3).
    pub not_judged: usize,
    /// F's last shots whose window the run's end cuts short (a shot in the
    /// run's last 2 ticks, or with a projectile of its firer still in
    /// flight at the end), not judged by Q-2516 question 4's interim.
    pub cut_short: usize,
    /// The judged shots.
    pub judged: Vec<Judged>,
    /// Every fragment-warhead hit in the run, and those whose attacker is
    /// not an F vehicle or that name none.
    pub fragment_hits: (usize, usize),
    /// The first shot of each group (F, G): its tick and the centre
    /// distances, squared, from each firer of that tick to its target.
    pub first: [Option<(i32, Vec<i64>)>; 2],
    /// The shortest gap in ticks 0 to 299 between two shots of one vehicle
    /// with one weapon: F's shot weapon, G's stage 0 and stage 1 weapons.
    pub gaps: [Option<i32>; 3],
    /// The full delay's base of each (S-0301 claim 2, draw at 0).
    pub expected_gap: [i32; 3],
    /// Every stage change of a G vehicle: (from stage, value the tick
    /// before, value at the change).
    pub stage_changes: Vec<(i32, i32, i32)>,
    /// The largest spin value read.
    pub largest: i32,
    /// The vehicles dead at the end (F, G), and the run's first death.
    pub dead: (usize, usize),
    /// The tick of the run's first death of either side.
    pub first_death: Option<i32>,
    /// Each G vehicle's first tick at stage 1.
    pub first_stage_one: Vec<Option<i32>>,
}

fn cell_of(p: Pos) -> CellXY {
    // measure: M-0021 (Procedure: the cell is the position divided by 256, truncated; S-0204 claim 2)
    p.cell()
}

/// The cells holding an F vehicle and the cells holding a G vehicle other
/// than `except`, at tick `t`.
fn occupied(
    rec: &Recording,
    t: i32,
    except: ObjectId,
) -> (BTreeSet<CellXY>, BTreeMap<CellXY, Vec<ObjectId>>) {
    let mut f = BTreeSet::new();
    let mut g: BTreeMap<CellXY, Vec<ObjectId>> = BTreeMap::new();
    if let Some(ps) = rec.positions.get(&t) {
        for (&id, &p) in ps {
            if rec.f.contains(&id) {
                f.insert(cell_of(p));
            } else if rec.g.contains(&id) && id != except {
                g.entry(cell_of(p)).or_default().push(id);
            }
        }
    }
    (f, g)
}

/// The candidate cells around `x` at tick `t` in S-0303 table T1's order,
/// rings 1 to 3 (Procedure step 4), and whether a cell at Chebyshev
/// distance 1 to 3 holds both an F and a G vehicle.
pub fn candidates(rec: &Recording, t: i32, x: ObjectId) -> Option<(Vec<CellXY>, bool)> {
    let xc = cell_of(*rec.positions.get(&t)?.get(&x)?);
    let (f, g) = occupied(rec, t, x);
    let mut out = Vec::new();
    for r in 1..=RINGS {
        // spec: S-0303 table T1 (the ring order S-0318 claim 3 visits); rings 1 to 3 hold no ring-11 cell
        for (dx, dy) in glasnost_sim::combat::area::ring(r, false) {
            let c = CellXY::new(xc.x + dx, xc.y + dy);
            if g.contains_key(&c) && !f.contains(&c) {
                out.push(c);
            }
        }
    }
    let mixed = f.iter().any(|c| {
        let cheb = (c.x - xc.x).abs().max((c.y - xc.y).abs());
        (1..=RINGS).contains(&cheb) && g.contains_key(c)
    });
    Some((out, mixed))
}

/// Judges one run (Procedure steps 1 to 9).
pub fn judge(rec: &Recording, part: Part) -> Findings {
    let [f_shot, _, g0, g1] = rec.weapons;
    let [f_wh, s_wh] = rec.warheads;
    let mut out = Findings {
        expected_gap: rec.expected_gap,
        ..Default::default()
    };
    // Step 1: a run in which F fires any weapon other than its primary is void.
    out.void = rec
        .shots
        .iter()
        .any(|s| rec.f.contains(&s.firer) && s.weapon != f_shot);
    // Deaths: a vehicle in play at tick t − 1 and not at t died at t.
    let mut died: BTreeMap<ObjectId, i32> = BTreeMap::new();
    let mut prev: Option<&BTreeMap<ObjectId, Pos>> = None;
    for (&t, ps) in &rec.positions {
        if let Some(p) = prev {
            for id in p.keys() {
                if !ps.contains_key(id) {
                    died.entry(*id).or_insert(t);
                }
            }
        }
        prev = Some(ps);
    }
    out.dead = (
        rec.f.iter().filter(|id| died.contains_key(id)).count(),
        rec.g.iter().filter(|id| died.contains_key(id)).count(),
    );
    out.first_death = died.values().copied().min();
    // Step 2: windows per F firer.
    for &firer in &rec.f {
        let shots: Vec<&ShotRec> = rec.shots.iter().filter(|s| s.firer == firer).collect();
        for (i, s) in shots.iter().enumerate() {
            let end = shots.get(i + 1).map_or(rec.end + 1, |n| n.tick);
            let in_window = |t: i32| t >= s.tick && t < end;
            let x = match s.target {
                Target::Object(x) => Some(x),
                Target::Cell(_) => None,
            };
            // Step 3: judged unless the target is not a G vehicle in play at t, or (part B) the window holds a death.
            let target_ok = x.is_some_and(|x| {
                rec.g.contains(&x)
                    && rec
                        .positions
                        .get(&s.tick)
                        .is_some_and(|p| p.contains_key(&x))
            });
            let death = part == Part::B && died.values().any(|&t| in_window(t));
            let (Some(x), true, false) = (x, target_ok, death) else {
                out.not_judged += 1;
                continue;
            };
            // The firer's last shot, in the run's last 2 ticks or with a
            // projectile of the firer still in flight at the run's end: the
            // run's end cuts its window short of the hits that follow the
            // shot, so it is not judged (Q-2516 question 4's interim).
            if i + 1 == shots.len() && (rec.end - s.tick < 2 || rec.in_flight.contains(&s.firer)) {
                out.cut_short += 1;
                continue;
            }
            let owned: Vec<&HitRec> = rec
                .hits
                .iter()
                .filter(|h| in_window(h.tick) && h.attacker == Some(firer))
                .collect();
            let mut j = Judged {
                tick: s.tick,
                target: x,
                c: 0,
                mixed_cell: false,
                shot_hits: (0, 0),
                fragment_hits: Vec::new(),
                fragments_on_f: 0,
                fragment_g: BTreeSet::new(),
                damaged: BTreeSet::new(),
                first_three: (false, false),
            };
            for h in &owned {
                if h.warhead == Some(f_wh) {
                    if h.id == x {
                        j.shot_hits.0 += 1;
                    } else {
                        j.shot_hits.1 += 1;
                    }
                } else if h.warhead == Some(s_wh) {
                    j.fragment_hits.push((h.id, h.health.0 - h.health.1));
                    if rec.f.contains(&h.id) {
                        j.fragments_on_f += 1;
                    } else if rec.g.contains(&h.id) && h.id != x {
                        j.fragment_g.insert(h.id);
                    }
                }
                if h.health.1 < h.health.0 {
                    j.damaged.insert(h.id);
                }
            }
            if let Some((cands, mixed)) = candidates(rec, s.tick, x) {
                j.c = cands.len();
                j.mixed_cell = mixed;
            }
            // Step 5: the cells of the G vehicles its fragments hit against the first 3 candidate cells.
            let hit_g: Vec<ObjectId> = j
                .fragment_hits
                .iter()
                .map(|&(id, _)| id)
                .filter(|id| rec.g.contains(id))
                .collect();
            let matches = |t: i32| {
                let Some((cands, _)) = candidates(rec, t, x) else {
                    return false;
                };
                let first: BTreeSet<CellXY> =
                    cands.iter().take(FRAGMENTS as usize).copied().collect();
                let cells: Option<BTreeSet<CellXY>> = hit_g
                    .iter()
                    .map(|id| rec.positions.get(&t)?.get(id).map(|&p| cell_of(p)))
                    .collect();
                cells.is_some_and(|c| c == first)
            };
            j.first_three = (matches(s.tick), (s.tick - 1..=s.tick + 1).any(matches));
            out.judged.push(j);
        }
    }
    out.fragment_hits = (
        rec.hits.iter().filter(|h| h.warhead == Some(s_wh)).count(),
        rec.hits
            .iter()
            .filter(|h| h.warhead == Some(s_wh) && !h.attacker.is_some_and(|a| rec.f.contains(&a)))
            .count(),
    );
    // Step 6: who fires first, and the centre distance at that tick.
    for (k, group) in [&rec.f, &rec.g].into_iter().enumerate() {
        let first = rec
            .shots
            .iter()
            .filter(|s| group.contains(&s.firer))
            .map(|s| s.tick)
            .min();
        out.first[k] = first.map(|t| {
            let ps = rec.positions.get(&t);
            let ds = rec
                .shots
                .iter()
                .filter(|s| s.tick == t && group.contains(&s.firer))
                .filter_map(|s| {
                    let Target::Object(x) = s.target else {
                        return None;
                    };
                    let (a, b) = (ps?.get(&s.firer)?, ps?.get(&x)?);
                    let (dx, dy, dz) = (
                        i64::from(a.x - b.x),
                        i64::from(a.y - b.y),
                        i64::from(a.z - b.z),
                    );
                    Some(dx * dx + dy * dy + dz * dz)
                })
                .collect();
            (t, ds)
        });
    }
    // Step 7: the shortest gap per vehicle and weapon in ticks 0 to 299.
    for (k, weapon) in [f_shot, g0, g1].into_iter().enumerate() {
        let mut best: Option<i32> = None;
        for &v in rec.f.iter().chain(&rec.g) {
            let ticks: Vec<i32> = rec
                .shots
                .iter()
                .filter(|s| s.firer == v && s.weapon == weapon && s.tick < GAP_TICKS)
                .map(|s| s.tick)
                .collect();
            for p in ticks.windows(2) {
                let gap = p[1] - p[0];
                best = Some(best.map_or(gap, |b| b.min(gap)));
            }
        }
        out.gaps[k] = best;
    }
    // Steps 8 and 9: the stage changes, the largest value, each G vehicle's first stage 1.
    let mut last: BTreeMap<ObjectId, (i32, i32)> = BTreeMap::new();
    let mut first_one: BTreeMap<ObjectId, i32> = BTreeMap::new();
    for (&t, spins) in &rec.spins {
        for (&id, &(v, s)) in spins {
            out.largest = out.largest.max(v);
            if let Some(&(pv, ps)) = last.get(&id)
                && ps != s
            {
                out.stage_changes.push((ps, pv, v));
            }
            if s >= 1 {
                first_one.entry(id).or_insert(t);
            }
            last.insert(id, (v, s));
        }
    }
    out.first_stage_one = rec.g.iter().map(|id| first_one.get(id).copied()).collect();
    out
}

// ---------------------------------------------------------------------------
// The report: one verdict per row of the record's Result

/// A row's verdict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Holds in every run it covers.
    Pass,
    /// Does not hold; what ours shows.
    Fail(String),
    /// Ours beside the record's, with no verdict (a row the record marks
    /// not for tests).
    Shown(String),
    /// Not checkable here, and why.
    NotCheckable(String),
}

impl Verdict {
    fn text(&self) -> String {
        match self {
            Verdict::Pass => "pass".into(),
            Verdict::Fail(s) => format!("FAIL: {s}"),
            Verdict::Shown(s) => format!("shown (not for tests): {s}"),
            Verdict::NotCheckable(s) => format!("not checkable: {s}"),
        }
    }
}

/// The verdicts of every row, in the record's order: (row, verdict).
pub fn verdicts(runs: &[((Part, Side, u32), Findings)]) -> Vec<(String, Verdict)> {
    let mut out = Vec::new();
    let of = |p: Part| runs.iter().filter(move |((q, _, _), _)| *q == p);
    let check = |ok: bool, why: String| {
        if ok {
            Verdict::Pass
        } else {
            Verdict::Fail(why)
        }
    };
    let name =
        |(p, s, seed): &(Part, Side, u32)| format!("{} F on {} seed {}", p.text(), s.text(), seed);
    let void: Vec<String> = runs
        .iter()
        .filter(|(_, f)| f.void)
        .map(|(k, _)| name(k))
        .collect();
    out.push((
        "void check: F fires only its primary weapon".into(),
        check(void.is_empty(), format!("void runs: {}", void.join(", "))),
    ));
    let missing = |p: Part| {
        (of(p).count() == 0)
            .then(|| Verdict::NotCheckable(format!("no run of part {} was played", p.text())))
    };
    let per_shot = |p: Part, label: &str, out: &mut Vec<(String, Verdict)>| {
        if let Some(v) = missing(p) {
            out.push((format!("{label}: the per-shot rows"), v));
            return;
        }
        let mut bad_c = Vec::new();
        let mut bad_hits = Vec::new();
        let mut bad_f = Vec::new();
        let mut bad_window = Vec::new();
        let mut none = Vec::new();
        for (k, f) in of(p) {
            if f.judged.is_empty() {
                none.push(name(k));
            }
            for j in &f.judged {
                let ids: BTreeSet<ObjectId> = j.fragment_hits.iter().map(|&(id, _)| id).collect();
                let on_f = j.fragments_on_f;
                if j.c < FRAGMENTS as usize {
                    bad_c.push(format!("{} t {}: c {}", name(k), j.tick, j.c));
                }
                let distinct_g = j.fragment_g.len() == FRAGMENTS as usize;
                if j.shot_hits != (1, 0)
                    || j.fragment_hits.len() != FRAGMENTS as usize
                    || !distinct_g
                {
                    bad_hits.push(format!(
                        "{} t {}: shot-warhead hits on X {} (others {}), fragment hits {} on {} objects{}",
                        name(k),
                        j.tick,
                        j.shot_hits.0,
                        j.shot_hits.1,
                        j.fragment_hits.len(),
                        ids.len(),
                        if ids.contains(&j.target) { ", one on X" } else { "" },
                    ));
                }
                if on_f > 0 {
                    bad_f.push(format!("{} t {}: {on_f}", name(k), j.tick));
                }
                if j.fragment_hits.len() != FRAGMENTS as usize {
                    bad_window.push(format!(
                        "{} t {}: {}",
                        name(k),
                        j.tick,
                        j.fragment_hits.len(),
                    ));
                }
            }
        }
        let tail = |v: &[String]| {
            let mut s = format!("{} shots: ", v.len());
            s.push_str(&v.iter().take(6).cloned().collect::<Vec<_>>().join("; "));
            s
        };
        if !none.is_empty() {
            out.push((
                format!("{label}: judged shots"),
                Verdict::Fail(format!("no judged shot in {}", none.join(", "))),
            ));
        }
        // measure: M-0021 (c ≥ 3 in every judged shot, exact)
        out.push((
            format!("{label}: c ≥ 3 in every judged shot"),
            check(bad_c.is_empty(), tail(&bad_c)),
        ));
        // measure: M-0021 (1 shot-warhead hit on X and 3 fragment-warhead hits on 3 distinct G vehicles other than X, exact)
        out.push((
            format!("{label}: hits a judged shot owns: 1 on X, 3 fragments on 3 distinct G vehicles other than X"),
            check(bad_hits.is_empty(), tail(&bad_hits)),
        ));
        if p == Part::A {
            // measure: M-0021 (F vehicles hit by a judged shot's fragments: 0, exact)
            out.push((
                format!("{label}: F vehicles hit by a judged shot's fragments: 0"),
                check(bad_f.is_empty(), tail(&bad_f)),
            ));
            // measure: M-0021 (fragment-warhead hits in a judged shot's window naming its firer: 3, exact)
            out.push((
                format!("{label}: fragment-warhead hits in a judged window naming its firer: 3"),
                check(bad_window.is_empty(), tail(&bad_window)),
            ));
        }
    };
    per_shot(Part::A, "part A", &mut out);
    // measure: M-0021 (part A: the health one fragment hit took is the same in every hit, exact; the value is local)
    let lost: BTreeSet<i32> = of(Part::A)
        .flat_map(|(_, f)| {
            f.judged
                .iter()
                .flat_map(|j| j.fragment_hits.iter().map(|&(_, l)| l))
        })
        .collect();
    out.push((
        "part A: health one fragment hit took, the same in every hit".into(),
        missing(Part::A)
            .unwrap_or_else(|| check(lost.len() == 1, format!("{} distinct amounts", lost.len()))),
    ));
    // measure: M-0021 (part A: fragment-warhead hits whose attacker is not an F vehicle, or that name none: 0, exact)
    let stray: usize = of(Part::A).map(|(_, f)| f.fragment_hits.1).sum();
    out.push((
        "part A: fragment-warhead hits whose attacker is not an F vehicle or that name none: 0"
            .into(),
        missing(Part::A).unwrap_or_else(|| check(stray == 0, format!("{stray} hits"))),
    ));
    // measure: M-0021 (part A0: fragment-warhead hits 0; objects damaged by one F shot: 1, its target, exact)
    let a0_frag: usize = of(Part::A0).map(|(_, f)| f.fragment_hits.0).sum();
    out.push((
        "part A0: fragment-warhead hits: 0".into(),
        missing(Part::A0).unwrap_or_else(|| check(a0_frag == 0, format!("{a0_frag} hits"))),
    ));
    let a0_bad: Vec<String> = of(Part::A0)
        .flat_map(|(k, f)| {
            f.judged
                .iter()
                .filter(|j| j.damaged.len() != 1 || !j.damaged.contains(&j.target))
                .map(move |j| format!("{} t {}: {} objects", name(k), j.tick, j.damaged.len()))
        })
        .collect();
    let a0_unjudged: usize = of(Part::A0).map(|(_, f)| f.not_judged).sum();
    out.push((
        "part A0: objects damaged by one F shot: 1, its target".into(),
        if let Some(v) = missing(Part::A0) {
            v
        } else if a0_unjudged > 0 {
            Verdict::Fail(format!("{a0_unjudged} F shots whose target was not a G vehicle in play (the record: none in A0); {}", a0_bad.join("; ")))
        } else {
            check(a0_bad.is_empty(), a0_bad.join("; "))
        },
    ));
    per_shot(Part::B, "part B", &mut out);
    per_shot(Part::Control, "control", &mut out);
    // Not for tests: the first 3 candidate cells.
    let judged: Vec<&Judged> = runs
        .iter()
        .filter(|((p, _, _), _)| *p != Part::A0)
        .flat_map(|(_, f)| f.judged.iter())
        .collect();
    let at_t = judged.iter().filter(|j| !j.first_three.0).count();
    let around = judged.iter().filter(|j| !j.first_three.1).count();
    // measure: M-0021 (judged shots whose 3 G vehicles hit stand in the first 3 candidate cells: all but 13 at t, all but 6 at t − 1, t or t + 1; not for tests)
    out.push((
        "the 3 G vehicles hit stand in the first 3 candidate cells of T1's order".into(),
        Verdict::Shown(format!("ours: all but {at_t} at t, all but {around} at t − 1, t or t + 1; the record: all but 13 and all but 6")),
    ));
    // First shots.
    let all_runs = runs.iter();
    let f_first: Vec<String> = all_runs
        .clone()
        .filter(|(_, f)| match (&f.first[0], &f.first[1]) {
            (Some((a, _)), Some((b, _))) => a >= b,
            (None, _) => true,
            _ => false,
        })
        .map(|(k, f)| {
            format!(
                "{}: F {:?}, G {:?}",
                name(k),
                f.first[0].as_ref().map(|x| x.0),
                f.first[1].as_ref().map(|x| x.0)
            )
        })
        .collect();
    // measure: M-0021 (the side that fires first: F in every run, exact)
    out.push((
        "first shots: F fires first in every run".into(),
        check(f_first.is_empty(), f_first.join("; ")),
    ));
    for (k, label) in [(0usize, "F"), (1, "G")] {
        let want = FIRST_SHOT_DISTANCE[k];
        let mut seen: BTreeSet<i64> = BTreeSet::new();
        for (_, f) in runs {
            if let Some((_, ds)) = &f.first[k] {
                for &d2 in ds {
                    seen.insert(d2);
                }
            }
        }
        let shown: Vec<String> = seen.iter().map(|&d2| lepton_text(d2)).collect();
        // measure: M-0021 (centre distance at F's first shot 1527 leptons, at G's 1280, at equal heights, exact)
        let ok = !seen.is_empty() && seen.iter().all(|&d2| rounds_to(d2, want));
        out.push((
            format!("first shots: centre distance at {label}'s first shot {want} leptons"),
            check(
                ok,
                format!("ours (leptons, over the runs): {}", shown.join(", ")),
            ),
        ));
    }
    // Shot gaps.
    for (k, label) in [
        "F's shot weapon",
        "G's stage 0 weapon",
        "G's stage 1 weapon",
    ]
    .into_iter()
    .enumerate()
    {
        let mut bad = Vec::new();
        for (key, f) in runs {
            match f.gaps[k] {
                None => bad.push(format!("{}: no two shots", name(key))),
                Some(g) if g != f.expected_gap[k] => bad.push(format!(
                    "{}: shortest gap differs from trunc(R × h) by {:+}",
                    name(key),
                    g - f.expected_gap[k]
                )),
                _ => {}
            }
        }
        // measure: M-0021 (the shortest gap is R × h with the draw at 0, converted to whole ticks, in every run, exact; S-0301 claim 2)
        out.push((
            format!("shot gaps: {label}'s shortest gap in ticks 0 to 299 is trunc(R × h)"),
            check(bad.is_empty(), bad.join("; ")),
        ));
    }
    // Stages.
    let rises: BTreeSet<(i32, i32)> = runs
        .iter()
        .flat_map(|(_, f)| {
            f.stage_changes
                .iter()
                .filter(|c| c.0 == 0)
                .map(|c| (c.1, c.2))
        })
        .collect();
    // measure: M-0021 (a change from stage 0 to 1 reads 30, then 31, every such change, exact)
    out.push((
        "stages: a rise from stage 0 to 1 reads 30, then 31".into(),
        check(
            rises.len() == 1 && rises.contains(&RISE),
            format!("ours: {rises:?}"),
        ),
    ));
    let largest: BTreeSet<i32> = runs.iter().map(|(_, f)| f.largest).collect();
    // measure: M-0021 (the largest value read: 60 in every run, exact)
    out.push((
        "stages: the largest value read is 60 in every run".into(),
        check(
            largest.len() == 1 && largest.contains(&LARGEST),
            format!("ours: {largest:?}"),
        ),
    ));
    let falls: BTreeSet<(i32, i32)> = of(Part::B)
        .flat_map(|(_, f)| {
            f.stage_changes
                .iter()
                .filter(|c| c.0 == 1)
                .map(|c| (c.1, c.2))
        })
        .collect();
    let other_falls: usize = runs
        .iter()
        .filter(|((p, _, _), _)| *p != Part::B)
        .map(|(_, f)| f.stage_changes.iter().filter(|c| c.0 == 1).count())
        .sum();
    // measure: M-0021 (part B: a change from stage 1 to 0 reads 30, then 29; parts A, A0 and the control have none, exact)
    out.push((
        "stages: a fall from stage 1 to 0 reads 30, then 29 (part B; none in the other parts)"
            .into(),
        missing(Part::B).unwrap_or_else(|| {
            check(
                falls.len() == 1 && falls.contains(&FALL) && other_falls == 0,
                format!("ours: part B {falls:?}, falls in the other parts {other_falls}"),
            )
        }),
    ));
    // Part B's table.
    let mut table = Vec::new();
    let mut ok = missing(Part::B).is_none();
    for &(side, seed, g_dead) in &PART_B_G_DEAD {
        let Some((_, f)) = runs.iter().find(|(k, _)| *k == (Part::B, side, seed)) else {
            ok = false;
            table.push(format!("{} {seed}: not played", side.text()));
            continue;
        };
        let left = match (f.dead.0 < GROUP as usize, f.dead.1 < GROUP as usize) {
            (true, false) => "F",
            (false, true) => "G",
            (true, true) => "both",
            (false, false) => "none",
        };
        // measure: M-0021 (part B: G left with vehicles, 16 F vehicles dead, G's dead per run)
        let this = left == "G" && f.dead.0 == GROUP as usize && f.dead.1 == g_dead as usize;
        ok &= this;
        table.push(format!(
            "{} {seed}: left {left}, F dead {}, G dead {} (record G, 16, {g_dead})",
            side.text(),
            f.dead.0,
            f.dead.1
        ));
    }
    out.push((
        "part B: the side left, F and G vehicles dead per run".into(),
        if ok {
            Verdict::Pass
        } else {
            Verdict::Fail(table.join("; "))
        },
    ));
    let mut leads = Vec::new();
    let mut late = 0;
    for (_, f) in of(Part::B) {
        for s1 in &f.first_stage_one {
            match (s1, f.first_death) {
                (Some(a), Some(d)) => leads.push(d - a),
                _ => late += 1,
            }
        }
    }
    let (lo, hi) = (leads.iter().min().copied(), leads.iter().max().copied());
    // measure: M-0021 (part B: every G vehicle first read stage 1 before the run's first death, by 54 to 182 ticks over all 96)
    out.push((
        "part B: every G vehicle reads stage 1 before the first death (the record: by 54 to 182 ticks)".into(),
        missing(Part::B).unwrap_or_else(|| {
            check(
                late == 0 && lo.is_some_and(|l| l > 0),
                format!("ours: leads {lo:?} to {hi:?}, {late} vehicles without a lead"),
            )
        }),
    ));
    out.push((
        "part B: the lead's range, 54 to 182 ticks".into(),
        Verdict::Shown(format!(
            "ours {lo:?} to {hi:?}; the record {} to {}",
            STAGE_LEAD.0, STAGE_LEAD.1
        )),
    ));
    out
}

/// Whether the square root of `d2` rounds to `want` (half up).
fn rounds_to(d2: i64, want: i64) -> bool {
    // (want − ½)² ≤ d2 < (want + ½)², times 4 to stay in integers.
    let lo = (2 * want - 1) * (2 * want - 1);
    let hi = (2 * want + 1) * (2 * want + 1);
    (lo..hi).contains(&(4 * d2))
}

fn lepton_text(d2: i64) -> String {
    let r = (d2.max(0) as u64).isqrt();
    format!("{r}{}", if r * r == d2 as u64 { "" } else { "+" })
}

/// The whole report: per-run lines (local values: counts of shots and
/// judged shots) and the verdicts.
pub fn report(built: &BTreeMap<Part, Built>, runs: &[((Part, Side, u32), Findings)]) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "{ID} test pairing (local values: never commit this report)"
    );
    for (p, b) in built {
        let _ = writeln!(
            out,
            "  part {}: built from tank {}, weapon {}, warhead {}, projectile {}; the weapon's burst {:?}, the tank's turret count {:?}",
            p.text(),
            b.from[0],
            b.from[1],
            b.from[2],
            b.from[3],
            b.burst,
            b.turrets
        );
    }
    for ((p, s, seed), f) in runs {
        let _ = writeln!(
            out,
            "  part {} F on {} seed {seed}: void {}, judged {}, not judged {}, cut short by the end {}, fragment hits {} ({} not by F), first shots {:?}, gaps {:?} expected {:?}, stage changes {}, largest {}, dead F {} G {}, first death {:?}",
            p.text(),
            s.text(),
            f.void,
            f.judged.len(),
            f.not_judged,
            f.cut_short,
            f.fragment_hits.0,
            f.fragment_hits.1,
            f.first
                .iter()
                .map(|x| x
                    .as_ref()
                    .map(|(t, ds)| (t, ds.iter().map(|&d| lepton_text(d)).collect::<Vec<_>>())))
                .collect::<Vec<_>>(),
            f.gaps,
            f.expected_gap,
            f.stage_changes.len(),
            f.largest,
            f.dead.0,
            f.dead.1,
            f.first_death
        );
    }
    out.push_str(&verdicts_text(runs));
    out
}

/// The verdicts, one line per row, and the count of rows failing (the line
/// `M-0021: rows failing: N of M`).
pub fn verdicts_text(runs: &[((Part, Side, u32), Findings)]) -> String {
    let mut out = String::new();
    let v = verdicts(runs);
    let failing = v
        .iter()
        .filter(|(_, v)| matches!(v, Verdict::Fail(_)))
        .count();
    let judged = v
        .iter()
        .filter(|(_, v)| !matches!(v, Verdict::Shown(_)))
        .count();
    let _ = writeln!(out, "{ID} rows:");
    for (row, verdict) in &v {
        let _ = writeln!(out, "  {row}: {}", verdict.text());
    }
    let _ = writeln!(out, "{ID}: rows failing: {failing} of {judged}");
    out
}

/// The base tank our test types copy: `GLASNOST_M0021_BASE`, else
/// `default` (our choice).
pub fn base_from_env(default: &str) -> TestBase {
    TestBase {
        tank: std::env::var("GLASNOST_M0021_BASE").unwrap_or_else(|_| default.to_owned()),
    }
}

/// Builds the four parts' grounds on `ground` (whose rules are `base`, of
/// hash `hash`), plays the record's 16 runs on up to `threads` threads and
/// judges them. Returns the verdicts (with any run that failed to play) and
/// the whole report with its local values.
pub fn run(
    ground: &Ground,
    base: &Ruleset,
    hash: &str,
    from: &TestBase,
    threads: usize,
) -> Result<(String, String), String> {
    let mut grounds = BTreeMap::new();
    let mut built = BTreeMap::new();
    for p in Part::ALL {
        let (g, b) = part_ground(ground, base, hash, from, p)?;
        grounds.insert(p, g);
        built.insert(p, b);
    }
    let jobs = runs();
    let results = play_all(&grounds, &jobs, threads);
    let mut judged = Vec::new();
    let mut errors = String::new();
    for (&(p, s, seed), r) in jobs.iter().zip(results) {
        match r {
            Ok(rec) => judged.push(((p, s, seed), judge(&rec, p))),
            Err(e) => {
                let _ = writeln!(
                    errors,
                    "{ID}: part {} F on {} seed {seed}: {e}",
                    p.text(),
                    s.text()
                );
            }
        }
    }
    let summary = format!("{errors}{}", verdicts_text(&judged));
    let full = format!("{errors}{}", report(&built, &judged));
    Ok((summary, full))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oracle::{synthetic_ground, synthetic_ruleset};

    /// Our synthetic map's size: room for both groups of 16 and the rings.
    const SIZE: i32 = 48;

    fn base() -> TestBase {
        // Our synthetic ruleset's one armed tank type.
        TestBase {
            tank: "CRUSHER".into(),
        }
    }

    fn ground(part: Part) -> Ground {
        let g = synthetic_ground(SIZE);
        part_ground(&g, &synthetic_ruleset(), "synthetic-arena", &base(), part)
            .expect("the test types build")
            .0
    }

    fn other(s: Side) -> Side {
        match s {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }

    // measure: M-0021 (Setup, Test types and Parts)
    #[test]
    fn the_test_types_carry_the_records_fields() {
        let (r, built) = test_ruleset(&synthetic_ruleset(), &base(), Part::A).expect("built");
        assert_eq!(built.from[0], "CRUSHER");
        let f = &r.sim.vehicles[names::F];
        let g = &r.sim.vehicles[names::G];
        for u in [f, g] {
            assert_eq!(u.strength, Some(60000));
            assert_eq!((u.speed, u.rot, u.sight), (Some(6), Some(8), Some(8)));
            assert_eq!(
                (u.accelerates, u.opportunity_fire),
                (Some(false), Some(false))
            );
            assert!(u.veteran_abilities.is_empty() && u.elite_abilities.is_empty());
            assert_eq!(
                (u.combat.max_debris, u.combat.min_debris),
                (Some(0), Some(0))
            );
        }
        assert_eq!(f.primary.as_deref(), Some(names::F_SHOT));
        assert_eq!(f.elite_primary.as_deref(), Some(names::F_SHOT));
        assert_eq!(f.secondary.as_deref(), Some(names::FRAGMENT));
        let shot = &r.sim.weapons[names::F_SHOT];
        assert_eq!(
            (shot.damage, shot.rof, shot.speed),
            (Some(20), Some(60), Some(100))
        );
        assert_eq!(shot.range, Some(Decimal("6".into())));
        let frag = &r.sim.weapons[names::FRAGMENT];
        assert_eq!(
            (frag.damage, frag.range.clone()),
            (Some(7), Some(Decimal("3".into())))
        );
        let fp = &r.sim.projectiles[names::F_PROJECTILE];
        assert_eq!(fp.shrapnel_weapon.as_deref(), Some(names::FRAGMENT));
        assert_eq!((fp.shrapnel_count, fp.inviso), (Some(3), Some(true)));
        assert_ne!(shot.warhead, frag.warhead);
        for n in [names::F_WARHEAD, names::FRAGMENT_WARHEAD, names::G_WARHEAD] {
            let wh = &r.sim.warheads[n];
            assert_eq!(wh.verses, Some(vec![Decimal("1".into()); 11]));
            assert_eq!((wh.wall, wh.wood), (Some(false), Some(false)));
        }
        assert_eq!(
            g.combat.weapon_slots.as_deref(),
            Some(&["M21G0", "M21G0", "M21G1", "M21G1"].map(String::from)[..])
        );
        assert_eq!(g.combat.stage_thresholds, Some(vec![30, 60]));
        assert_eq!(g.turret_count, Some(1));
        let rules = SimRules::new(&r, "synthetic");
        let gi = rules
            .index_of(glasnost_sim::rules::Kind::Vehicle, names::G)
            .expect("G");
        let tc = &rules.vehicles[gi as usize].combat;
        assert!(tc.gattling);
        assert_eq!((tc.spin.stages, tc.spin.up, tc.spin.down), (2, 1, 1));
        // Part A0: no fragment weapon or count; part B: strength 400; the
        // control: the easy row's factor 1.
        let (a0, _) = test_ruleset(&synthetic_ruleset(), &base(), Part::A0).expect("A0");
        let p = &a0.sim.projectiles[names::F_PROJECTILE];
        assert_eq!(
            (p.shrapnel_weapon.as_deref(), p.shrapnel_count),
            (None, None)
        );
        let (b, _) = test_ruleset(&synthetic_ruleset(), &base(), Part::B).expect("B");
        assert_eq!(b.sim.vehicles[names::G].strength, Some(400));
        let (c, _) = test_ruleset(&synthetic_ruleset(), &base(), Part::Control).expect("control");
        assert_eq!(c.sim.difficulty["easy"].rof, Some(Decimal("1".into())));
        assert_eq!(runs().len(), 16);
    }

    // measure: M-0021 (Setup, Houses and Groups: F the Americans' house and G the Russians', whichever side F stands on; 16 vehicles a side; the neutral house's objects removed)
    #[test]
    fn the_houses_do_not_change_with_the_side() {
        let g = ground(Part::A);
        for f_side in [Side::A, Side::B] {
            let rec = play_to(&g, Part::A, f_side, 0, 0).expect("set up");
            assert_eq!((rec.f.len(), rec.g.len()), (16, 16));
            let ps = &rec.positions[&0];
            for (ids, side) in [(&rec.f, f_side), (&rec.g, other(f_side))] {
                for id in ids {
                    let x = ps[id].cell().x;
                    assert_eq!(x < g.w.x, side == Side::A, "a group on its side's cells");
                }
            }
        }
        let a = crate::oracle::setup_with(
            &g,
            [names::G, names::F],
            Side::B,
            0,
            &SetupOptions {
                count: Some(GROUP),
                houses: [SIDE_HOUSES[1], SIDE_HOUSES[0]],
                clear_neutral: true,
            },
        )
        .expect("set up");
        for id in &a.units[1] {
            assert_eq!(a.world.obj(*id).and_then(|o| o.owner), Some(SIDE_HOUSES[0]));
        }
        for id in &a.units[0] {
            assert_eq!(a.world.obj(*id).and_then(|o| o.owner), Some(SIDE_HOUSES[1]));
        }
        let neutral = a.world.objects.values().any(|o| {
            o.owner.is_some_and(|h| {
                a.world.houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Neutral
            })
        });
        assert!(!neutral);
    }

    fn at(c: CellXY) -> Pos {
        Pos::centre(c, 0)
    }

    /// A hand-made recording: one F firer and four G vehicles, the target
    /// X with three G vehicles in ring 1 around it.
    fn hand_made() -> Recording {
        let (f, x) = (ObjectId(1), ObjectId(2));
        let gs = [ObjectId(3), ObjectId(4), ObjectId(5)];
        let xc = CellXY::new(20, 20);
        let ring1 = glasnost_sim::combat::area::ring(1, false);
        let cell = |k: usize| CellXY::new(xc.x + ring1[k].0, xc.y + ring1[k].1);
        let mut rec = Recording {
            f: vec![f],
            g: vec![x, gs[0], gs[1], gs[2]],
            weapons: [0, 1, 2, 3],
            warheads: [0, 1],
            expected_gap: [60, 20, 10],
            end: 40,
            ..Default::default()
        };
        for t in 0..=40 {
            let mut ps = BTreeMap::new();
            ps.insert(f, at(CellXY::new(14, 20)));
            ps.insert(x, at(xc));
            for (k, id) in gs.into_iter().enumerate() {
                ps.insert(id, at(cell(k)));
            }
            rec.positions.insert(t, ps);
        }
        let shot = |tick, target| ShotRec {
            tick,
            firer: f,
            target,
            weapon: 0,
        };
        rec.shots = vec![
            shot(10, Target::Object(x)),
            shot(20, Target::Cell(xc)),
            shot(30, Target::Object(x)),
        ];
        let hit = |tick, id, warhead| HitRec {
            tick,
            id,
            warhead: Some(warhead),
            attacker: Some(f),
            health: (100, 93),
        };
        rec.hits = vec![
            hit(10, x, 0),
            hit(10, gs[0], 1),
            hit(10, gs[1], 1),
            hit(11, gs[2], 1),
            hit(30, x, 0),
            hit(30, gs[0], 1),
        ];
        rec
    }

    // measure: M-0021 (Procedure steps 1 to 5 and 9: the void check, the window, the judged shots, c, the first 3 candidate cells, the deaths)
    #[test]
    fn the_procedure_judges_a_hand_made_recording() {
        let rec = hand_made();
        let f = judge(&rec, Part::A);
        assert!(!f.void);
        // The shot at a cell is not judged; the other two are.
        assert_eq!((f.judged.len(), f.not_judged), (2, 1));
        let j = &f.judged[0];
        assert_eq!((j.tick, j.c, j.mixed_cell), (10, 3, false));
        assert_eq!(j.shot_hits, (1, 0));
        // The fragment hit at tick 11 is in the window from 10 up to 20.
        assert_eq!((j.fragment_hits.len(), j.fragment_g.len()), (3, 3));
        assert_eq!(j.first_three, (true, true));
        // The last shot's window runs to the run's end: one fragment hit.
        assert_eq!(f.judged[1].fragment_hits.len(), 1);
        let v = verdicts(&[((Part::A, Side::A, 0), f.clone())]);
        let row = |s: &str| {
            v.iter()
                .find(|(r, _)| r.starts_with(s))
                .map(|(_, v)| v.clone())
        };
        assert_eq!(row("part A: c ≥ 3"), Some(Verdict::Pass));
        assert!(matches!(
            row("part A: hits a judged shot owns"),
            Some(Verdict::Fail(_))
        ));
        // Part B: a death in the window leaves the shot out.
        let mut b = hand_made();
        for t in 15..=40 {
            b.positions.get_mut(&t).expect("tick").remove(&ObjectId(5));
        }
        let fb = judge(&b, Part::B);
        assert_eq!((fb.dead, fb.first_death), ((0, 1), Some(15)));
        assert_eq!(fb.judged.len(), 1);
        // Step 1: F firing another weapon voids the run.
        let mut v1 = hand_made();
        v1.shots[1].weapon = 1;
        assert!(judge(&v1, Part::A).void);
    }

    // measure: M-0021 (Procedure step 2: the last shot's window runs to the run's end); Q-2516 question 4's interim: a last shot whose window the end cuts short is not judged
    #[test]
    fn a_last_shot_cut_short_by_the_end_is_not_judged() {
        // Its firer still has a projectile in flight at the end.
        let mut rec = hand_made();
        rec.in_flight.insert(ObjectId(1));
        let f = judge(&rec, Part::A);
        assert_eq!((f.judged.len(), f.not_judged, f.cut_short), (1, 1, 1));
        assert_eq!(f.judged[0].tick, 10);
        // A shot in the run's last 2 ticks.
        let mut late = hand_made();
        late.shots[2].tick = 39;
        let f = judge(&late, Part::A);
        assert_eq!((f.judged.len(), f.cut_short), (1, 1));
        // Neither: the last shot is judged with the hits its window holds.
        let f = judge(&hand_made(), Part::A);
        assert_eq!((f.judged.len(), f.cut_short), (2, 0));
    }

    // measure: M-0021 (Procedure step 4: a cell holding an F vehicle is no candidate, and the Chebyshev check sees it)
    #[test]
    fn a_cell_holding_an_f_vehicle_is_not_a_candidate() {
        let mut rec = hand_made();
        let g1 = rec.positions[&10][&ObjectId(3)];
        rec.positions
            .get_mut(&10)
            .expect("tick")
            .insert(ObjectId(1), g1);
        let (cands, mixed) = candidates(&rec, 10, ObjectId(2)).expect("X recorded");
        assert_eq!(cands.len(), 2);
        assert!(mixed);
    }

    #[test]
    fn a_distance_rounds_to_whole_leptons() {
        assert!(rounds_to(1527 * 1527, 1527));
        assert!(rounds_to(1527 * 1527 + 1527, 1527));
        assert!(!rounds_to(1527 * 1527 + 1528, 1527));
        assert!(!rounds_to(1526 * 1526, 1527));
    }

    /// One synthetic run of a part, judged.
    fn synthetic(part: Part, limit: i32) -> Findings {
        let rec = play_to(&ground(part), part, Side::A, 0, limit).expect("plays");
        judge(&rec, part)
    }

    // spec: S-0318 claims 1, 2, 3 and 5 (measure: M-0021, part A: a judged shot with 3 or more candidates bursts into 3 fragments, on 3 distinct G vehicles other than X, none on F, each naming the firer)
    #[test]
    fn synthetic_part_a_every_judged_burst_hits_three_candidates() {
        let f = synthetic(Part::A, 300);
        assert!(!f.void);
        // The shots whose window the run's end cuts own fewer hits.
        let full: Vec<&Judged> = f.judged.iter().filter(|j| j.c >= 3).collect();
        assert!(!full.is_empty(), "some judged shot with 3 candidates");
        for j in full {
            assert_eq!(j.shot_hits, (1, 0), "tick {}", j.tick);
            assert_eq!(j.fragment_hits.len(), 3, "tick {}", j.tick);
            assert_eq!(j.fragment_g.len(), 3, "tick {}", j.tick);
            assert_eq!(j.fragments_on_f, 0, "tick {}", j.tick);
            let lost: BTreeSet<i32> = j.fragment_hits.iter().map(|&(_, l)| l).collect();
            assert_eq!(lost.len(), 1, "every fragment hit takes the same health");
        }
        assert_eq!(
            f.fragment_hits.1, 0,
            "every fragment hit names an F vehicle"
        );
    }

    // spec: S-0318 claim 1 (measure: M-0021, part A0: a projectile naming no fragment weapon launches none)
    #[test]
    fn synthetic_part_a0_has_no_fragments() {
        let f = synthetic(Part::A0, 300);
        assert!(!f.judged.is_empty());
        assert_eq!(f.fragment_hits.0, 0);
        for j in &f.judged {
            assert_eq!(
                j.damaged.iter().copied().collect::<Vec<_>>(),
                vec![j.target]
            );
        }
    }

    // spec: S-0609 claims 2 and 3 (measure: M-0021: a rise from stage 0 to 1 reads 30, then 31; the cap 60 is the largest value read; a fall from 1 to 0 reads 30, then 29) and S-0301 claims 2 and 3 (measure: M-0021: the shortest gap is trunc(R × h) with the draw at 0)
    #[test]
    fn synthetic_part_a_stages_and_gaps() {
        let f = synthetic(Part::A, 600);
        let changes = |from: i32| -> BTreeSet<(i32, i32)> {
            f.stage_changes
                .iter()
                .filter(|c| c.0 == from)
                .map(|c| (c.1, c.2))
                .collect()
        };
        assert_eq!(changes(0), BTreeSet::from([RISE]));
        assert_eq!(f.largest, LARGEST);
        // Our synthetic map is not M-0001's: its G vehicles also lose their
        // targets in part A, and every fall reads as part B's do.
        assert_eq!(changes(1), BTreeSet::from([FALL]));
        for k in 0..3 {
            assert_eq!(f.gaps[k], Some(f.expected_gap[k]), "weapon {k}");
        }
    }
}
