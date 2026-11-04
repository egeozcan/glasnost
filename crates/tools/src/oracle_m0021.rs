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
//! M-0021's British observer owns one building whose type, cell, facing
//! and mission the record keeps `local`: the type is our choice
//! ([`observer_building`]), on the observer's start position's cell. The
//! rest of the setup follows the record's answers to Q-2516: the creation
//! order ([`F_FIRST`]), the gattling type's numbered weapons
//! ([`test_ruleset`]) and the run's end ([`END_EXTRA`]).
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

/// Whether F's group is created and placed first, on whichever side it
/// stands (measure: M-0021, Setup, Houses: creation order, F's 16 vehicles,
/// then G's 16, whichever side F stands on, each in increasing index).
pub const F_FIRST: bool = true;

/// The ticks the game plays on after the tick limit in parts A and A0 and
/// the control (measure: M-0021, Parts and seeds: every run's recording
/// ends at tick 608, the limit being 600).
pub const END_EXTRA: i32 = 8;

/// Leptons in a cell (measure: M-0021, Result, First shots: F's range of 6
/// cells is 1536 leptons, G's of 5 cells 1280).
pub const LEPTONS_PER_CELL: i64 = 256;

/// The type of the observer's one building: `GLASNOST_M0021_OBSERVER`, else
/// the first unarmed building type of the rules with a 2 by 2 foundation
/// that is no factory or gate (a 1 by 1 type may be a wall), else the first
/// unarmed one. The record keeps the type `local` and the
/// building takes no part in the fight (measure: M-0021, Setup, Houses).
pub fn observer_building(r: &glasnost_sim::rules::SimRules) -> Option<String> {
    if let Ok(n) = std::env::var("GLASNOST_M0021_OBSERVER") {
        return Some(n);
    }
    let unarmed = |b: &&glasnost_sim::rules::BuildingRules| {
        b.combat.slots.iter().all(Option::is_none) && b.combat.occupy_weapon.is_none()
    };
    r.buildings
        .iter()
        .filter(unarmed)
        .find(|b| b.foundation == (2, 2) && b.factory.is_none() && !b.gate)
        .or_else(|| r.buildings.iter().find(unarmed))
        .map(|b| b.name.clone())
}

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
/// slot 1 filled; the record's file put the stage 0 weapon in slots 0 and
/// 1 and the stage 1 weapon in slots 2 and 3, with stage 0's weapon as
/// primary and secondary (measure: M-0021, Test types). The new types go at the end of the vehicle
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
    /// The firer's position as the shot's own row gives it (measure:
    /// M-0021, Procedure: the positions in leptons as the shot's own row
    /// gives them). Our shot event holds no positions, so the recorder
    /// derives them ([`shot_positions`]); `None` when an object had left
    /// the world.
    pub firer_pos: Option<Pos>,
    /// The target's position as the shot's own row gives it (derived like
    /// the firer's); `None` for a cell target.
    pub target_pos: Option<Pos>,
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
    /// Each tick's row of every vehicle that is still an object of the
    /// world, dead or alive: its position and health (measure: M-0021,
    /// Procedure: every vehicle's position and health each tick, stamped
    /// *t* = read after tick *t* has run). The record's reading-point rows
    /// are read from these.
    pub rows: BTreeMap<i32, BTreeMap<ObjectId, (Pos, i32)>>,
    /// The last tick played. Its per-tick rows are not kept in
    /// `positions` and `spins` (measure: M-0021, Parts and seeds: the
    /// per-tick rows end one tick before the recording's last tick).
    pub end: i32,
    /// The vehicles in play at the last tick, kept only to read deaths at
    /// that tick.
    pub last_in_play: Option<BTreeSet<ObjectId>>,
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

/// The positions of a shot's firer and target as the shot's own row gives
/// them (measure: M-0021, Procedure: the firer's and the target's positions
/// as the shot's own row gives them), derived from our state, since our shot
/// event holds none and the tool cannot stop inside a tick.
///
/// - The firer: its position after the tick. A vehicle's firing step comes
///   after its movement in its own turn and nothing moves it afterwards
///   (`tick::object_turn`; spec: S-0310 claim 9, the vehicle's own firing
///   step each tick it has a target, and S-0215 claim 11, the late start
///   "after this update's movement, before its separate firing").
/// - The target: it moves only in its own turn. When that turn comes before
///   the firer's in the update order read at the start of the tick
///   (`order`; spec: S-0203 claim 3), it has moved when the firer fires, so
///   its position after the tick; otherwise its position before the tick
///   (`before`).
///
/// The lookup is by the order's own indices for every shot, never by group.
/// So the reading-point rows of the shots test our update order and the
/// movement-before-firing rule, not an independent capture; the check that
/// every derived distance is within the weapon's range tests the
/// derivation against the engine's own firing test ([`Findings::reach`]).
///
/// A vehicle destroyed in the tick has left the world (S-0203 claim 6 (a)),
/// so its position after its move is the one its last hit of the tick
/// records (`gone`: `Effect::Hit`'s position, the hit object's own position
/// at the hit, read in `combat::damage` before the hit's effects).
fn shot_positions(
    w: &World,
    order: &[ObjectId],
    before: &BTreeMap<ObjectId, Pos>,
    gone: &BTreeMap<ObjectId, Pos>,
    firer: ObjectId,
    target: Target,
) -> (Option<Pos>, Option<Pos>) {
    let after = |id: ObjectId| w.obj(id).map(|o| o.pos).or_else(|| gone.get(&id).copied());
    let firer_pos = after(firer);
    let Target::Object(x) = target else {
        return (firer_pos, None);
    };
    let turn = |id: ObjectId| order.iter().position(|&o| o == id);
    let moved = matches!((turn(x), turn(firer)), (Some(tx), Some(tf)) if tx < tf);
    let target_pos = if moved {
        after(x)
    } else {
        before.get(&x).copied()
    };
    (firer_pos, target_pos)
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
        // measure: M-0021 (Setup, Houses: the British observer owns one building, its type local)
        observer_building: observer_building(&rules),
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
        let rows = all
            .iter()
            .filter_map(|&id| w.obj(id).map(|o| (id, (o.pos, o.health))))
            .collect();
        rec.rows.insert(t, rows);
    };
    snap(&w, &mut rec);
    let alive = |w: &World, ids: &[ObjectId]| ids.iter().any(|&id| in_play(w, id).is_some());
    // measure: M-0021 (Parts and seeds: in parts A and A0 and the control the game plays on for 8 ticks after the tick limit; part B ends after one side has no vehicle left)
    let stop = if part == Part::B {
        limit
    } else {
        limit + END_EXTRA
    };
    while w.tick < stop {
        // The update order and the positions at the start of the tick, for the shots' derived positions ([`shot_positions`]).
        let order = w.update.order.clone();
        let before: BTreeMap<ObjectId, Pos> = all
            .iter()
            .filter_map(|&id| w.obj(id).map(|o| (id, o.pos)))
            .collect();
        glasnost_sim::tick::step(&mut w);
        let t = w.tick;
        let effects = std::mem::take(&mut w.effects);
        // Where a vehicle was when it was last hit in the tick, for one that left the world in it.
        let gone: BTreeMap<ObjectId, Pos> = effects
            .iter()
            .filter_map(|e| match e {
                Effect::Hit { id, pos, .. } if all.contains(id) => Some((*id, *pos)),
                _ => None,
            })
            .collect();
        for e in effects {
            match e {
                Effect::Shot {
                    firer,
                    weapon,
                    target,
                    ..
                } if all.contains(&firer) => {
                    let (firer_pos, target_pos) =
                        shot_positions(&w, &order, &before, &gone, firer, target);
                    rec.shots.push(ShotRec {
                        tick: t,
                        firer,
                        target,
                        weapon,
                        firer_pos,
                        target_pos,
                    });
                }
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
    // measure: M-0021 (Parts and seeds: the per-tick rows end one tick before the recording's last tick)
    if rec.end > 0 {
        let last = rec.positions.remove(&rec.end).unwrap_or_default();
        rec.spins.remove(&rec.end);
        rec.rows.remove(&rec.end);
        rec.last_in_play = Some(last.into_keys().collect());
    }
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

/// One firer of a side's first-shot tick (Procedure step 6). Distances are
/// squared leptons.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shooter {
    /// The firer.
    pub firer: ObjectId,
    /// The centre distance to its target from the shot row's positions.
    pub shot: Option<i64>,
    /// The same from the per-tick rows stamped *t* − 1 and *t*.
    pub rows: (Option<i64>, Option<i64>),
    /// The weapon's range in leptons, squared.
    pub range2: i64,
}

/// A side's first shot: its tick and every firer of that tick.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FirstShot {
    /// The tick.
    pub tick: i32,
    /// Each firer of that tick with a target object.
    pub shooters: Vec<Shooter>,
}

/// A vehicle's own first shot: the distance at the reading points (an F
/// shot: the firer's position stamped *t* to the target's stamped *t* − 1; a
/// G shot: both stamped *t*) against the weapon's range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnFirst {
    /// A vehicle of F's group.
    pub f: bool,
    /// The tick.
    pub tick: i32,
    /// The squared distance, if both rows exist.
    pub d2: Option<i64>,
    /// The weapon's range in leptons, squared.
    pub range2: i64,
}

/// For one object of a shot, which per-tick row holds the position the shot's
/// row gives (Procedure step 11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PointCounts {
    /// The row stamped *t*, where the rows stamped *t* − 1 and *t* differ.
    pub at_t: usize,
    /// The row stamped *t* − 1, where they differ.
    pub at_before: usize,
    /// Neither row.
    pub neither: usize,
    /// The two rows are equal (the object did not move during *t*).
    pub unchanged: usize,
    /// A row or a position is missing.
    pub unread: usize,
}

/// The reading points (Procedure step 11).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reading {
    /// Hits that took health and have a per-tick row at their tick, per
    /// (tick, object) the last of them (the only one whose health after a
    /// row can show): the row that first shows its health after is the
    /// row stamped *t*, or the one stamped *t* − 1.
    pub hit_at_t: usize,
    /// See `hit_at_t`: the row stamped *t* − 1 already shows it.
    pub hit_at_before: usize,
    /// No row of *t* − 1 or *t* shows it.
    pub hit_unseen: usize,
    /// Hits followed by another hit on the same object in the same tick.
    pub hit_superseded: usize,
    /// Hits for which the row stamped *t* − 1 does not show the first hit's
    /// health before.
    pub hit_before_mismatch: usize,
    /// Hits that took health but have no per-tick row of their object at
    /// their tick (not covered by the record's row).
    pub hit_no_row: usize,
    /// Vehicles destroyed by a hit at a tick with per-tick rows.
    pub deaths: usize,
    /// Of those, the last row of the vehicle is the row stamped *t*, with no
    /// health left.
    pub deaths_last_at_t: usize,
    /// The others: what their last row is.
    pub deaths_other: Vec<String>,
    /// Destroyed by a hit at a tick without per-tick rows (the last tick).
    pub deaths_unread: usize,
    /// An F shot's firer, an F shot's target, a G shot's firer, a G shot's
    /// target.
    pub shot_rows: [PointCounts; 4],
}

/// The check of the derived shot positions against the engine's own firing
/// test (our check, not a row of the record).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reach {
    /// Shots with an object target and both positions derived.
    pub shots: usize,
    /// Of those, shots whose derived centre distance is beyond the weapon's
    /// range *R* (the engine fired, so its range test passed). That test is
    /// "at most *R* + *E*₁ + β" with the spatial distance of S-0307's table
    /// square root (spec: S-0307 range table), not the exact distance, so a
    /// distance a fraction of a lepton beyond *R* is no evidence against the
    /// derivation.
    pub beyond: usize,
    /// Local report only: the first of them: its tick, whether F fired it,
    /// the squared distance and the squared range.
    pub beyond_first: Option<(i32, bool, i64, i64)>,
    /// Of the shots with a row of the firer and of the target at *t* − 1 and
    /// at *t*, their number.
    pub combo_shots: usize,
    /// Of those, the shots within range if the firer is read at *t* − 1 or
    /// *t* and the target at *t* − 1 or *t* (index 2 × firer + target, 0 for
    /// the row stamped *t* − 1).
    pub combos: [usize; 4],
    /// Shots with a missing derived position.
    pub unread: usize,
}

/// What one run shows (Procedure steps 1 to 9).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Findings {
    /// F fired a weapon other than its primary (step 1: the run is void).
    pub void: bool,
    /// F's shots not judged (step 3).
    pub not_judged: usize,
    /// The judged shots.
    pub judged: Vec<Judged>,
    /// Every fragment-warhead hit in the run, and those whose attacker is
    /// not an F vehicle or that name none.
    pub fragment_hits: (usize, usize),
    /// The first shot of each group (F, G): its tick and each firer of that
    /// tick (Procedure step 6).
    pub first: [Option<FirstShot>; 2],
    /// Every vehicle's own first shot (measure: M-0021, Result, First shots:
    /// the distance at the reading points is within the weapon's range).
    pub own_first: Vec<OwnFirst>,
    /// The reading points (Procedure step 11).
    pub reading: Reading,
    /// Our own check of the derived shot positions against the engine's
    /// firing test ([`shot_positions`]).
    pub reach: Reach,
    /// The shortest gap in ticks 0 to 299 between two shots of one vehicle
    /// with one weapon: F's shot weapon, G's stage 0 and stage 1 weapons.
    pub gaps: [Option<i32>; 3],
    /// Local report only: (tick, squared centre distance) of F's first shooter to its target over the 4 ticks up to its first shot.
    pub f_approach: Vec<(i32, i64)>,
    /// Local report only: F's longest gap between two shots in ticks 0 to 299.
    pub f_longest_gap: Option<i32>,
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

/// Every vehicle's death: the first tick at which it is no longer in play
/// (a vehicle in play at tick t − 1 and not at t died at t), over the
/// per-tick rows and the last tick's vehicles.
pub fn deaths(rec: &Recording) -> BTreeMap<ObjectId, i32> {
    let mut died: BTreeMap<ObjectId, i32> = BTreeMap::new();
    let mut prev: Option<BTreeSet<ObjectId>> = None;
    let rows = rec
        .positions
        .iter()
        .map(|(&t, ps)| (t, ps.keys().copied().collect::<BTreeSet<_>>()))
        .chain(rec.last_in_play.clone().map(|s| (rec.end, s)));
    for (t, ps) in rows {
        if let Some(p) = &prev {
            for id in p {
                if !ps.contains(id) {
                    died.entry(*id).or_insert(t);
                }
            }
        }
        prev = Some(ps);
    }
    died
}

/// The squared centre distance in leptons.
fn dist2(a: Pos, b: Pos) -> i64 {
    let (dx, dy, dz) = (
        i64::from(a.x - b.x),
        i64::from(a.y - b.y),
        i64::from(a.z - b.z),
    );
    dx * dx + dy * dy + dz * dz
}

/// The range in leptons of one of the shot weapons (measure: M-0021, Setup,
/// Test types: F's primary 6 cells, G's stage weapons 5 cells; spec: S-0307
/// claim 1, the range in leptons), or `None` for any other weapon.
fn range_leptons(rec: &Recording, weapon: WeaponId) -> Option<i64> {
    let cells = if weapon == rec.weapons[0] {
        F_WEAPON.2
    } else if weapon == rec.weapons[2] {
        G_WEAPONS[0].2
    } else if weapon == rec.weapons[3] {
        G_WEAPONS[1].2
    } else {
        return None;
    };
    Some(i64::from(cells) * LEPTONS_PER_CELL)
}

/// The reading points of every shot and hit (Procedure step 11) and the
/// check of the derived shot positions.
fn reading_points(rec: &Recording) -> (Reading, Reach) {
    let mut out = Reading::default();
    let mut reach = Reach::default();
    let row = |t: i32, id: ObjectId| rec.rows.get(&t).and_then(|r| r.get(&id)).copied();
    // Hits: group by (tick, object); the last of a group is the one whose health after a row can show.
    let mut groups: BTreeMap<(i32, ObjectId), Vec<&HitRec>> = BTreeMap::new();
    for h in &rec.hits {
        // A hit on a vehicle that already has no health left (still a row) is no new hit that took health.
        if h.health.0 > 0 && h.health.1 < h.health.0 {
            groups.entry((h.tick, h.id)).or_default().push(h);
        }
    }
    let mut last_row: BTreeMap<ObjectId, i32> = BTreeMap::new();
    for (&t, r) in &rec.rows {
        for &id in r.keys() {
            last_row.insert(id, t);
        }
    }
    for (&(t, id), hs) in &groups {
        out.hit_superseded += hs.len() - 1;
        let (first, last) = (hs[0], hs[hs.len() - 1]);
        let destroyed = last.health.1 <= 0;
        if destroyed {
            // measure: M-0021 (Result, Reading points: a vehicle destroyed by a hit at t, its last per-tick row is the row stamped t with no health left)
            if !rec.rows.contains_key(&t) {
                out.deaths_unread += 1;
            } else {
                out.deaths += 1;
                match (last_row.get(&id), row(t, id)) {
                    (Some(&r), Some((_, h))) if r == t && h <= 0 => out.deaths_last_at_t += 1,
                    (r, at) => out.deaths_other.push(format!(
                        "object {}: last row at t plus {:?}, health at row stamped t {:?}",
                        id.0,
                        r.map(|r| r - t),
                        at.map(|(_, h)| h)
                    )),
                }
            }
        }
        // measure: M-0021 (Result, Reading points: the row that first shows a hit's health after is the row stamped t, never t − 1; every hit with a per-tick row at its tick)
        let Some((_, at)) = row(t, id) else {
            out.hit_no_row += 1;
            continue;
        };
        // A row shows the health after when it holds it (a destroyed vehicle: no health left).
        let shows = |h: i32| h == last.health.1 || (destroyed && h <= 0);
        if let Some((_, b)) = row(t - 1, id) {
            if b != first.health.0 {
                out.hit_before_mismatch += 1;
            }
            if shows(b) {
                out.hit_at_before += 1;
                continue;
            }
        }
        if shows(at) {
            out.hit_at_t += 1;
        } else {
            out.hit_unseen += 1;
        }
    }
    // Shots: which row holds the position the shot's row gives.
    for s in &rec.shots {
        let Target::Object(x) = s.target else {
            continue;
        };
        let by_f = rec.f.contains(&s.firer);
        let by_g = rec.g.contains(&s.firer);
        if !(by_f || by_g) {
            continue;
        }
        let k = if by_f { 0 } else { 2 };
        let t = s.tick;
        if !rec.rows.contains_key(&t) {
            // The run's last tick has no per-tick rows (Parts and seeds): nothing to read there.
            continue;
        }
        let mut point = |slot: usize, id: ObjectId, p: Option<Pos>| {
            let c = &mut out.shot_rows[slot];
            let (Some((a, _)), Some((b, _)), Some(p)) = (row(t - 1, id), row(t, id), p) else {
                c.unread += 1;
                return;
            };
            if a == b {
                if p == a {
                    c.unchanged += 1;
                } else {
                    c.neither += 1;
                }
            } else if p == b {
                c.at_t += 1;
            } else if p == a {
                c.at_before += 1;
            } else {
                c.neither += 1;
            }
        };
        point(k, s.firer, s.firer_pos);
        point(k + 1, x, s.target_pos);
        // Our check: the derived distance is within the weapon's range; and the four readings.
        let Some(range) = range_leptons(rec, s.weapon) else {
            continue;
        };
        let (Some(fp), Some(tp)) = (s.firer_pos, s.target_pos) else {
            reach.unread += 1;
            continue;
        };
        reach.shots += 1;
        if dist2(fp, tp) > range * range {
            reach.beyond += 1;
            reach
                .beyond_first
                .get_or_insert((t, by_f, dist2(fp, tp), range * range));
        }
        let rows4 = (
            row(t - 1, s.firer),
            row(t, s.firer),
            row(t - 1, x),
            row(t, x),
        );
        // A vehicle that left the world in the tick has no row stamped t: the four readings need all four rows.
        let (Some(f0), Some(f1), Some(x0), Some(x1)) = rows4 else {
            continue;
        };
        reach.combo_shots += 1;
        for (i, (a, b)) in [(f0, x0), (f0, x1), (f1, x0), (f1, x1)]
            .into_iter()
            .enumerate()
        {
            if dist2(a.0, b.0) <= range * range {
                reach.combos[i] += 1;
            }
        }
    }
    (out, reach)
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
    let died = deaths(rec);
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
            let shooters = rec
                .shots
                .iter()
                .filter(|s| s.tick == t && group.contains(&s.firer))
                .filter_map(|s| {
                    let Target::Object(x) = s.target else {
                        return None;
                    };
                    let range = range_leptons(rec, s.weapon)?;
                    // measure: M-0021 (Procedure step 6: the distance from the positions in the shot's own row, and whether the target is within range by the per-tick rows stamped t − 1 and t)
                    let rows = |tt: i32| {
                        let ps = rec.positions.get(&tt)?;
                        Some(dist2(*ps.get(&s.firer)?, *ps.get(&x)?))
                    };
                    Some(Shooter {
                        firer: s.firer,
                        shot: s.firer_pos.zip(s.target_pos).map(|(a, b)| dist2(a, b)),
                        rows: (rows(t - 1), rows(t)),
                        range2: range * range,
                    })
                })
                .collect();
            FirstShot { tick: t, shooters }
        });
    }
    // Every vehicle's own first shot, at the reading points (measure: M-0021, Result, First shots: an F shot from the firer's position stamped t to the target's stamped t − 1, a G shot both stamped t).
    for &v in rec.f.iter().chain(&rec.g) {
        let Some(s) = rec.shots.iter().find(|s| s.firer == v) else {
            continue;
        };
        let (Target::Object(x), Some(range)) = (s.target, range_leptons(rec, s.weapon)) else {
            continue;
        };
        let is_f = rec.f.contains(&v);
        let target_tick = if is_f { s.tick - 1 } else { s.tick };
        let d2 = rec
            .positions
            .get(&s.tick)
            .and_then(|p| p.get(&v))
            .zip(rec.positions.get(&target_tick).and_then(|p| p.get(&x)))
            .map(|(a, b)| dist2(*a, *b));
        out.own_first.push(OwnFirst {
            f: is_f,
            tick: s.tick,
            d2,
            range2: range * range,
        });
    }
    (out.reading, out.reach) = reading_points(rec);
    // Local report only: F's first shooter's centre distance to its target at the 4 ticks up to its first shot, and F's longest gap.
    if let Some(s) = rec.shots.iter().find(|s| rec.f.contains(&s.firer))
        && let Target::Object(x) = s.target
    {
        out.f_approach = (s.tick - 3..=s.tick)
            .filter_map(|t| {
                let ps = rec.positions.get(&t)?;
                let (a, b) = (ps.get(&s.firer)?, ps.get(&x)?);
                let (dx, dy, dz) = (
                    i64::from(a.x - b.x),
                    i64::from(a.y - b.y),
                    i64::from(a.z - b.z),
                );
                Some((t, dx * dx + dy * dy + dz * dz))
            })
            .collect();
    }
    out.f_longest_gap = rec
        .f
        .iter()
        .flat_map(|&v| {
            let t: Vec<i32> = rec
                .shots
                .iter()
                .filter(|s| s.firer == v && s.weapon == f_shot && s.tick < GAP_TICKS)
                .map(|s| s.tick)
                .collect();
            t.windows(2).map(|p| p[1] - p[0]).collect::<Vec<_>>()
        })
        .max();
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
// The seeds (measure: M-0021, Procedure step 12)

/// A shot as the runs are compared: the firer's place in the creation order
/// (F's vehicles, then G's), the target (a vehicle by place, a cell, or any
/// other object by number) and the weapon.
type ShotKey = (usize, (u8, i64, i64), WeaponId);
/// A hit as the runs are compared: the object hit by place, the warhead, the
/// attacker by place, the health before and after.
type HitKey = (usize, Option<WarheadId>, Option<usize>, (i32, i32));
/// A vehicle's per-tick row as the runs are compared: place, position,
/// health.
type RowKey = (usize, (i32, i32, i32), i32);
/// A vehicle's per-tick spin row as the runs are compared: place, spin value
/// and stage.
type SpinKey = (usize, i32, i32);
/// A vehicle's first shot in a run: tick, target and weapon.
type OwnFirstShot = (i32, (u8, i64, i64), WeaponId);

/// A shot fired at the first differing tick in some of the runs and not in
/// all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedShot {
    /// The firer's place in the creation order.
    pub firer: usize,
    /// A G vehicle.
    pub g: bool,
    /// Which shot of that vehicle it is in a run that fires it (1 = first).
    pub ordinal: usize,
    /// For each run that does not fire it then: the ticks to that vehicle's
    /// next shot in that run, and whether that shot has the same target and
    /// weapon; `None` when the vehicle never fires again.
    pub later: Vec<Option<(i32, bool)>>,
}

/// What the runs of one part and side with different seeds show
/// (Procedure step 12).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SeedDiff {
    /// The first tick at which the runs differ in a shot, a hit or a
    /// per-tick row (vehicles matched by their place in the creation
    /// order); `None` when they agree in every tick.
    pub first: Option<i32>,
    /// At that tick: whether the shots, the hits and the per-tick rows
    /// differ.
    pub differs: (bool, bool, bool),
    /// The shots fired then in some runs only.
    pub extras: Vec<SeedShot>,
    /// The latest tick at which any vehicle of any run fires its first shot.
    pub latest_first_shot: Option<i32>,
    /// Each side's first shot (F, G), the latest over the runs.
    pub side_first: [Option<i32>; 2],
    /// The first death of any vehicle in any run.
    pub first_death: Option<i32>,
    /// The runs' deaths (which vehicle, at which tick) are not all the same.
    pub deaths_differ: bool,
    /// Every vehicle's first shot (tick, target, weapon) is the same in all
    /// the runs.
    pub first_shots_same: bool,
    /// Local report only: how many F and how many G vehicles had fired their
    /// first shot (in the first run) before the first difference.
    pub fired_before: (usize, usize),
    /// Local report only: when the per-tick rows differ at the first
    /// difference, the place of the first vehicle whose row differs (or
    /// `usize::MAX`) and the field.
    pub first_row_diff: Option<(usize, &'static str)>,
    /// Local report only: the first tick at which a hit differs between the
    /// runs.
    pub first_hit_diff: Option<i32>,
    /// Local report only: the first tick of a death that is not the same in
    /// every run.
    pub first_death_diff: Option<i32>,
}

/// One part and side with its seeds compared.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeedGroup {
    /// The part.
    pub part: Part,
    /// F's side.
    pub side: Side,
    /// What the seeds show.
    pub diff: SeedDiff,
}

fn all_equal<T: PartialEq>(v: &[T]) -> bool {
    v.windows(2).all(|w| w[0] == w[1])
}

/// Compares runs of one part and side that differ only in their seed
/// (measure: M-0021, Procedure step 12): the first tick at which they differ
/// in a shot (tick, firer, target, weapon), a hit (tick, object hit,
/// warhead, attacker, health before and after) or a per-tick row (position,
/// health, spin value and stage), the vehicles matched by their place in the
/// creation order.
pub fn compare_seeds(recs: &[&Recording]) -> SeedDiff {
    let n = recs.len();
    let ids: Vec<Vec<ObjectId>> = recs
        .iter()
        .map(|r| r.f.iter().chain(&r.g).copied().collect())
        .collect();
    let places: Vec<BTreeMap<ObjectId, usize>> = ids
        .iter()
        .map(|v| v.iter().enumerate().map(|(i, &id)| (id, i)).collect())
        .collect();
    let target_key = |i: usize, t: Target| -> (u8, i64, i64) {
        match t {
            Target::Object(x) => match places[i].get(&x) {
                Some(&p) => (0, p as i64, 0),
                None => (2, i64::from(x.0), 0),
            },
            Target::Cell(c) => (1, i64::from(c.x), i64::from(c.y)),
        }
    };
    let mut shots: Vec<BTreeMap<i32, Vec<ShotKey>>> = vec![BTreeMap::new(); n];
    let mut hits: Vec<BTreeMap<i32, Vec<HitKey>>> = vec![BTreeMap::new(); n];
    for (i, r) in recs.iter().enumerate() {
        for s in &r.shots {
            if let Some(&p) = places[i].get(&s.firer) {
                shots[i]
                    .entry(s.tick)
                    .or_default()
                    .push((p, target_key(i, s.target), s.weapon));
            }
        }
        for h in &r.hits {
            if let Some(&p) = places[i].get(&h.id) {
                let attacker = h.attacker.and_then(|a| places[i].get(&a).copied());
                hits[i]
                    .entry(h.tick)
                    .or_default()
                    .push((p, h.warhead, attacker, h.health));
            }
        }
        for v in shots[i].values_mut() {
            v.sort();
        }
        for v in hits[i].values_mut() {
            v.sort();
        }
    }
    let rows_at = |i: usize, t: i32| -> Option<(Vec<RowKey>, Vec<SpinKey>)> {
        let rows = recs[i].rows.get(&t)?;
        let mut a: Vec<RowKey> = rows
            .iter()
            .filter_map(|(id, &(p, h))| Some((*places[i].get(id)?, (p.x, p.y, p.z), h)))
            .collect();
        a.sort();
        let mut s: Vec<SpinKey> = recs[i]
            .spins
            .get(&t)
            .map(|m| {
                m.iter()
                    .filter_map(|(id, &(v, st))| Some((*places[i].get(id)?, v, st)))
                    .collect()
            })
            .unwrap_or_default();
        s.sort();
        Some((a, s))
    };
    let mut out = SeedDiff::default();
    let end = recs.iter().map(|r| r.end).max().unwrap_or(0);
    let none_s: Vec<ShotKey> = Vec::new();
    let none_h: Vec<HitKey> = Vec::new();
    for t in 0..=end {
        let s: Vec<&Vec<ShotKey>> = (0..n)
            .map(|i| shots[i].get(&t).unwrap_or(&none_s))
            .collect();
        let h: Vec<&Vec<HitKey>> = (0..n).map(|i| hits[i].get(&t).unwrap_or(&none_h)).collect();
        let r: Vec<_> = (0..n).map(|i| rows_at(i, t)).collect();
        let d = (!all_equal(&s), !all_equal(&h), !all_equal(&r));
        if !(d.0 || d.1 || d.2) {
            continue;
        }
        out.first = Some(t);
        out.differs = d;
        if d.2 {
            // Local report only: which vehicle's row differs first, and in which field.
            out.first_row_diff = 'rows: {
                for other in r.iter().skip(1) {
                    let (Some(a), Some(b)) = (&r[0], other) else {
                        break 'rows Some((usize::MAX, "a run has no row"));
                    };
                    if a.0.len() != b.0.len() {
                        break 'rows Some((usize::MAX, "vehicles in play"));
                    }
                    for (x, y) in a.0.iter().zip(&b.0) {
                        if x != y {
                            let field = if x.1 != y.1 { "position" } else { "health" };
                            break 'rows Some((x.0, field));
                        }
                    }
                    for (x, y) in a.1.iter().zip(&b.1) {
                        if x != y {
                            break 'rows Some((x.0, "spin"));
                        }
                    }
                }
                None
            };
        }
        let keys: BTreeSet<ShotKey> = s.iter().flat_map(|v| v.iter().copied()).collect();
        for k in keys {
            let have: Vec<bool> = s.iter().map(|v| v.contains(&k)).collect();
            let Some(i) = have.iter().position(|&x| x) else {
                continue;
            };
            if have.iter().all(|&x| x) {
                continue;
            }
            let ordinal = recs[i]
                .shots
                .iter()
                .filter(|x| ids[i].get(k.0) == Some(&x.firer) && x.tick <= t)
                .count();
            let later = (0..n)
                .filter(|&j| !have[j])
                .map(|j| {
                    recs[j]
                        .shots
                        .iter()
                        .filter(|x| ids[j].get(k.0) == Some(&x.firer) && x.tick > t)
                        .min_by_key(|x| x.tick)
                        .map(|x| {
                            (
                                x.tick - t,
                                (target_key(j, x.target), x.weapon) == (k.1, k.2),
                            )
                        })
                })
                .collect();
            out.extras.push(SeedShot {
                firer: k.0,
                g: k.0 >= recs[i].f.len(),
                ordinal,
                later,
            });
        }
        break;
    }
    // The first shots: each vehicle's and each side's, the latest over the runs.
    for r in recs {
        for (k, group) in [&r.f, &r.g].into_iter().enumerate() {
            let first_of = |v: Option<ObjectId>| {
                r.shots
                    .iter()
                    .filter(|s| v.is_none_or(|v| s.firer == v) && group.contains(&s.firer))
                    .map(|s| s.tick)
                    .min()
            };
            if let Some(m) = first_of(None) {
                out.side_first[k] = Some(out.side_first[k].map_or(m, |o| o.max(m)));
            }
            for &v in group {
                if let Some(f) = first_of(Some(v)) {
                    out.latest_first_shot = Some(out.latest_first_shot.map_or(f, |o| o.max(f)));
                }
            }
        }
    }
    // Each vehicle's first shot in each run (tick, target, weapon), by place.
    let firsts: Vec<BTreeMap<usize, OwnFirstShot>> = recs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let mut m = BTreeMap::new();
            for s in &r.shots {
                if let Some(&p) = places[i].get(&s.firer) {
                    m.entry(p)
                        .or_insert((s.tick, target_key(i, s.target), s.weapon));
                }
            }
            m
        })
        .collect();
    out.first_shots_same = all_equal(&firsts);
    if let (Some(f), Some(first_run)) = (out.first, firsts.first()) {
        let f_len = recs[0].f.len();
        let before = |in_f: bool| {
            first_run
                .iter()
                .filter(|&(&p, &(t, _, _))| (p < f_len) == in_f && t < f)
                .count()
        };
        out.fired_before = (before(true), before(false));
    }
    // The deaths.
    let dmaps: Vec<BTreeMap<usize, i32>> = recs
        .iter()
        .enumerate()
        .map(|(i, r)| {
            deaths(r)
                .into_iter()
                .filter_map(|(id, t)| places[i].get(&id).map(|&p| (p, t)))
                .collect()
        })
        .collect();
    out.first_death = dmaps.iter().flat_map(|m| m.values().copied()).min();
    out.deaths_differ = !all_equal(&dmaps);
    // Local report only: the first tick at which a hit differs, and the first tick of a death that is not in every run at that tick.
    out.first_hit_diff = (0..=end).find(|t| {
        let h: Vec<&Vec<HitKey>> = (0..n).map(|i| hits[i].get(t).unwrap_or(&none_h)).collect();
        !all_equal(&h)
    });
    let places_dead: BTreeSet<usize> = dmaps.iter().flat_map(|m| m.keys().copied()).collect();
    out.first_death_diff = places_dead
        .iter()
        .filter_map(|p| {
            let ticks: Vec<Option<i32>> = dmaps.iter().map(|m| m.get(p).copied()).collect();
            if all_equal(&ticks) {
                None
            } else {
                ticks.iter().flatten().min().copied()
            }
        })
        .min();
    out
}

/// The seed rows of the record's Result, over the parts and sides played.
pub fn seed_verdicts(groups: &[SeedGroup]) -> Vec<(String, Verdict)> {
    let mut out = Vec::new();
    if groups.is_empty() {
        out.push((
            "seeds".into(),
            Verdict::NotCheckable("fewer than two seeds of a part and side were played".into()),
        ));
        return out;
    }
    let check = |ok: bool, why: String| {
        if ok {
            Verdict::Pass
        } else {
            Verdict::Fail(why)
        }
    };
    let label = |g: &SeedGroup| format!("part {} F on {}", g.part.text(), g.side.text());
    let bad = |f: &dyn Fn(&SeedGroup) -> bool| -> Vec<String> {
        groups.iter().filter(|g| f(g)).map(label).collect()
    };
    // measure: M-0021 (Result, Seeds: no run of one part and side agrees with the others in every tick; the three seeds differ, exact)
    let same = bad(&|g| g.diff.first.is_none());
    out.push((
        "seeds: the three seeds of a part and side differ".into(),
        check(
            same.is_empty(),
            format!("the seeds agree in every tick in: {}", same.join(", ")),
        ),
    ));
    // measure: M-0021 (Result, Seeds: before the first difference everything agrees, including every vehicle's first shot, exact)
    let late = bad(&|g| {
        g.diff
            .first
            .zip(g.diff.latest_first_shot)
            .is_some_and(|(f, l)| l >= f)
    });
    out.push((
        "seeds: every vehicle's first shot falls before the first difference".into(),
        check(
            late.is_empty(),
            format!(
                "a first shot at or after the first difference in: {}",
                late.join(", ")
            ),
        ),
    ));
    // measure: M-0021 (Result, Seeds: what agrees in the three seeds includes every vehicle's first shot: the other reading of that row, the first shot of each vehicle is the same in the seeds wherever it falls)
    // Our own supplementary reading, not a row of the record: shown, not counted in the rows failing.
    let moved = bad(&|g| !g.diff.first_shots_same);
    out.push((
        "seeds (our reading): every vehicle's first shot (tick, target, weapon) is the same in the three seeds"
            .into(),
        Verdict::Shown(if moved.is_empty() {
            "holds in every part and side".into()
        } else {
            format!("does not hold in: {}", moved.join(", "))
        }),
    ));
    // measure: M-0021 (Result, Seeds: the first difference is the tick of some G vehicles' second shots, exact)
    let kind = bad(&|g| {
        !g.diff.differs.0
            || g.diff.extras.is_empty()
            || g.diff.extras.iter().any(|e| !e.g || e.ordinal != 2)
    });
    out.push((
        "seeds: the first difference is the tick of some G vehicles' second shots".into(),
        check(
            kind.is_empty(),
            format!("otherwise in: {}", kind.join(", ")),
        ),
    ));
    // measure: M-0021 (Result, Seeds: each shot fired then in one seed only is fired by the same vehicle at the same target a few ticks later in the others)
    let later = bad(&|g| {
        g.diff.extras.iter().any(|e| {
            e.later
                .iter()
                .any(|l| !matches!(l, Some((dt, true)) if *dt > 0))
        })
    });
    out.push((
        "seeds: a shot fired in one seed only is fired by the same vehicle at the same target later in the others".into(),
        check(
            later.is_empty(),
            format!("otherwise in: {}", later.join(", ")),
        ),
    ));
    // measure: M-0021 (Result, Seeds: the first difference falls in the same tick in parts A and B and on both sides; the tick is local, so the report file holds ours)
    let firsts: BTreeSet<i32> = groups.iter().filter_map(|g| g.diff.first).collect();
    out.push((
        "seeds: the first difference falls in the same tick in every part and side".into(),
        check(
            firsts.len() <= 1,
            format!(
                "ours: {} different ticks over the parts and sides (the ticks are in the local report)",
                firsts.len()
            ),
        ),
    ));
    // measure: M-0021 (Result, Seeds: the first difference falls after each side's first shots and before any vehicle's death)
    let order = bad(&|g| {
        let Some(f) = g.diff.first else { return true };
        g.diff.side_first.iter().any(|s| s.is_none_or(|s| s >= f))
            || g.diff.first_death.is_some_and(|d| d <= f)
    });
    out.push((
        "seeds: the first difference falls after each side's first shots and before any death"
            .into(),
        check(
            order.is_empty(),
            format!("otherwise in: {}", order.join(", ")),
        ),
    ));
    // measure: M-0021 (Result, Seeds: part B's deaths differ between the seeds later, after further shots have moved apart)
    let part_b: Vec<&SeedGroup> = groups.iter().filter(|g| g.part == Part::B).collect();
    out.push((
        "seeds: part B's deaths differ between the seeds, after the first difference".into(),
        if part_b.is_empty() {
            Verdict::NotCheckable("no part B runs were played".into())
        } else {
            let wrong: Vec<String> = part_b
                .iter()
                .filter(|g| {
                    !(g.diff.deaths_differ
                        && g.diff
                            .first
                            .zip(g.diff.first_death)
                            .is_some_and(|(f, d)| d > f))
                })
                .map(|g| label(g))
                .collect();
            check(
                wrong.is_empty(),
                format!("otherwise in: {}", wrong.join(", ")),
            )
        },
    ));
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
pub fn verdicts(
    runs: &[((Part, Side, u32), Findings)],
    seeds: &[SeedGroup],
) -> Vec<(String, Verdict)> {
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
            (Some(a), Some(b)) => a.tick >= b.tick,
            (None, _) => true,
            _ => false,
        })
        .map(|(k, f)| {
            format!(
                "{}: F {:?}, G {:?}",
                name(k),
                f.first[0].as_ref().map(|x| x.tick),
                f.first[1].as_ref().map(|x| x.tick)
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
        let mut unread = 0;
        for (_, f) in runs {
            if let Some(first) = &f.first[k] {
                for sh in &first.shooters {
                    match sh.shot {
                        Some(d2) => {
                            seen.insert(d2);
                        }
                        None => unread += 1,
                    }
                }
            }
        }
        let shown: Vec<String> = seen.iter().map(|&d2| lepton_text(d2)).collect();
        // measure: M-0021 (centre distance at F's first shot 1527 leptons, at G's 1280, at equal heights, from the shot row's positions, exact)
        let ok = !seen.is_empty() && unread == 0 && seen.iter().all(|&d2| rounds_to(d2, want));
        out.push((
            format!(
                "first shots: centre distance at {label}'s first shot, from the shot row's positions, {want} leptons"
            ),
            check(
                ok,
                format!(
                    "ours (leptons, over the runs): {}{}",
                    shown.join(", "),
                    if unread > 0 {
                        format!("; {unread} shooters without a derived position")
                    } else {
                        String::new()
                    }
                ),
            ),
        ));
    }
    // The first shot's target within range by the per-tick rows stamped t − 1 and t.
    // measure: M-0021 (F's first shot: the target within F's range by the per-tick rows stamped t − 1 and t: yes, yes; G's: no, yes, exact)
    for (k, label, want) in [(0usize, "F", (true, true)), (1, "G", (false, true))] {
        let mut seen: BTreeSet<(Option<bool>, Option<bool>)> = BTreeSet::new();
        for (_, f) in runs {
            if let Some(first) = &f.first[k] {
                for sh in &first.shooters {
                    let within = |d: Option<i64>| d.map(|d2| d2 <= sh.range2);
                    seen.insert((within(sh.rows.0), within(sh.rows.1)));
                }
            }
        }
        let wanted = (Some(want.0), Some(want.1));
        out.push((
            format!(
                "first shots: {label}'s target within {label}'s range by the per-tick rows stamped t − 1, t: {}, {}",
                if want.0 { "yes" } else { "no" },
                if want.1 { "yes" } else { "no" }
            ),
            check(
                !seen.is_empty() && seen.iter().all(|&s| s == wanted),
                format!("ours (t − 1, t): {seen:?}"),
            ),
        ));
    }
    // measure: M-0021 (G's first shot: the per-tick distance stamped t is 1280, equal to the shot row's, exact)
    let mut g_rows: BTreeSet<(Option<i64>, Option<i64>)> = BTreeSet::new();
    for (_, f) in runs {
        if let Some(first) = &f.first[1] {
            for sh in &first.shooters {
                g_rows.insert((sh.rows.1, sh.shot));
            }
        }
    }
    out.push((
        "first shots: G's per-tick distance stamped t is 1280, equal to the shot row's".into(),
        check(
            !g_rows.is_empty()
                && g_rows.iter().all(|&(row, shot)| {
                    row.is_some_and(|d2| rounds_to(d2, FIRST_SHOT_DISTANCE[1]) && Some(d2) == shot)
                }),
            format!(
                "ours (per-tick, shot row; leptons): {:?}",
                g_rows
                    .iter()
                    .map(|&(a, b)| (a.map(lepton_text), b.map(lepton_text)))
                    .collect::<Vec<_>>()
            ),
        ),
    ));
    // measure: M-0021 (every vehicle's own first shot falls in a tick in which the distance from the firer's position stamped t to its target's stamped t − 1 (an F shot) or t (a G shot) is within the weapon's range)
    let far: Vec<String> = runs
        .iter()
        .flat_map(|(key, f)| {
            f.own_first
                .iter()
                .filter(|o| !o.d2.is_some_and(|d2| d2 <= o.range2))
                .map(move |o| format!("{}: {:?}", name(key), o.d2.map(lepton_text)))
        })
        .collect();
    let first_count: usize = runs.iter().map(|(_, f)| f.own_first.len()).sum();
    out.push((
        "first shots: every vehicle's own first shot is within range at the reading points".into(),
        if first_count == 0 {
            Verdict::NotCheckable("no vehicle fired".into())
        } else {
            check(
                far.is_empty(),
                format!(
                    "{} first shots: {}",
                    far.len(),
                    far.iter().take(6).cloned().collect::<Vec<_>>().join("; ")
                ),
            )
        },
    ));
    // Reading points (measure: M-0021, Result, Reading points).
    let sum =
        |f: &dyn Fn(&Reading) -> usize| -> usize { runs.iter().map(|(_, r)| f(&r.reading)).sum() };
    let (at_t, at_before, unseen, mism) = (
        sum(&|r| r.hit_at_t),
        sum(&|r| r.hit_at_before),
        sum(&|r| r.hit_unseen),
        sum(&|r| r.hit_before_mismatch),
    );
    out.push((
        "reading points: the row that first shows a hit's health after is the row stamped t, never t − 1".into(),
        if at_t + at_before + unseen == 0 {
            Verdict::NotCheckable("no hit had a per-tick row at its tick".into())
        } else {
            check(
                at_before == 0 && unseen == 0,
                format!(
                    "{at_t} hits first shown at t, {at_before} at t − 1, {unseen} in neither; {mism} hits whose t − 1 row does not show the health before; {} not covered (no row of the object at its tick, so the object was gone from the world), {} followed by another hit on the object in the same tick",
                    sum(&|r| r.hit_no_row),
                    sum(&|r| r.hit_superseded),
                ),
            )
        },
    ));
    let (deaths, last_t) = (sum(&|r| r.deaths), sum(&|r| r.deaths_last_at_t));
    let other: Vec<String> = runs
        .iter()
        .flat_map(|(key, r)| {
            r.reading
                .deaths_other
                .iter()
                .map(move |s| format!("{}: {s}", name(key)))
        })
        .collect();
    out.push((
        "reading points: a vehicle destroyed by a hit at t has its last per-tick row stamped t, with no health left".into(),
        if deaths == 0 {
            Verdict::NotCheckable("no vehicle was destroyed at a tick with per-tick rows".into())
        } else {
            check(
                last_t == deaths,
                format!(
                    "{last_t} of {deaths} destroyed vehicles have it; ours removes a destroyed vehicle at the hit (S-0203 claim 6 (a), S-0312 claim 15; Q-2596); the others: {} (first: {})",
                    deaths - last_t,
                    other.first().cloned().unwrap_or_default()
                ),
            )
        },
    ));
    for (slot, label, want) in [
        (0usize, "F shot: the firer", "t"),
        (1, "F shot: the target, a G vehicle", "t − 1"),
        (2, "G shot: the firer", "t"),
        (3, "G shot: the target, an F vehicle", "t"),
    ] {
        let mut c = PointCounts::default();
        for (_, f) in runs {
            let p = f.reading.shot_rows[slot];
            c.at_t += p.at_t;
            c.at_before += p.at_before;
            c.neither += p.neither;
            c.unchanged += p.unchanged;
            c.unread += p.unread;
        }
        let (right, wrong) = if want == "t" {
            (c.at_t, c.at_before)
        } else {
            (c.at_before, c.at_t)
        };
        // measure: M-0021 (Result, Reading points: an F shot's firer is read from the row stamped t and its target from t − 1; a G shot's firer and target from t; where the two rows differ, exact)
        out.push((
            format!("reading points: {label}: the per-tick row whose position equals the shot row's is {want}"),
            if right + wrong + c.neither == 0 {
                Verdict::NotCheckable("no such shot".into())
            } else {
                check(
                    wrong == 0 && c.neither == 0 && right > 0,
                    format!(
                        "ours: {} at t, {} at t − 1, {} neither ({} did not move, {} unread)",
                        c.at_t, c.at_before, c.neither, c.unchanged, c.unread
                    ),
                )
            },
        ));
    }
    // Our check of the derivation (not a row of the record).
    let (shots, beyond, unread, combo_shots) = (
        runs.iter().map(|(_, f)| f.reach.shots).sum::<usize>(),
        runs.iter().map(|(_, f)| f.reach.beyond).sum::<usize>(),
        runs.iter().map(|(_, f)| f.reach.unread).sum::<usize>(),
        runs.iter().map(|(_, f)| f.reach.combo_shots).sum::<usize>(),
    );
    let mut combos = [0usize; 4];
    for (_, f) in runs {
        for (c, n) in combos.iter_mut().zip(f.reach.combos) {
            *c += n;
        }
    }
    // Our own check, not a row of the record: shown, not counted in the rows failing. The counts of shots are local (the report file holds them).
    let how = |n: usize, of: usize| match n {
        0 => "none",
        n if n == of => "all",
        _ => "some",
    };
    out.push((
        "our check: every shot's derived centre distance is within its weapon's range".into(),
        Verdict::Shown(format!(
            "ours: shots beyond range: {}; shots without a derived position: {}",
            how(beyond, shots),
            how(unread, shots + unread)
        )),
    ));
    out.push((
        "our check: shots within range if the firer and the target are read at (t − 1, t − 1), (t − 1, t), (t, t − 1), (t, t)".into(),
        Verdict::Shown(format!(
            "ours: {}, {}, {}, {}",
            how(combos[0], combo_shots),
            how(combos[1], combo_shots),
            how(combos[2], combo_shots),
            how(combos[3], combo_shots)
        )),
    ));
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
    out.extend(seed_verdicts(seeds));
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
pub fn report(
    built: &BTreeMap<Part, Built>,
    runs: &[((Part, Side, u32), Findings)],
    seeds: &[SeedGroup],
) -> String {
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
            "  part {} F on {} seed {seed}: void {}, judged {}, not judged {}, fragment hits {} ({} not by F), first shots {:?}, gaps {:?} expected {:?}, stage changes {}, largest {}, dead F {} G {}, first death {:?}, F approach (tick, leptons) {:?}, F longest gap {:?}",
            p.text(),
            s.text(),
            f.void,
            f.judged.len(),
            f.not_judged,
            f.fragment_hits.0,
            f.fragment_hits.1,
            f.first
                .iter()
                .map(|x| x.as_ref().map(|first| (
                    first.tick,
                    first
                        .shooters
                        .iter()
                        .map(|sh| (
                            sh.shot.map(lepton_text),
                            sh.rows.0.map(lepton_text),
                            sh.rows.1.map(lepton_text),
                            lepton_text(sh.range2)
                        ))
                        .collect::<Vec<_>>()
                )))
                .collect::<Vec<_>>(),
            f.gaps,
            f.expected_gap,
            f.stage_changes.len(),
            f.largest,
            f.dead.0,
            f.dead.1,
            f.first_death,
            f.f_approach
                .iter()
                .map(|&(t, d)| (t, lepton_text(d)))
                .collect::<Vec<_>>(),
            f.f_longest_gap
        );
    }
    for ((p, s, seed), f) in runs {
        let _ = writeln!(
            out,
            "  part {} F on {} seed {seed}: derived shot positions {:?}, reading points {:?}",
            p.text(),
            s.text(),
            f.reach,
            f.reading
        );
    }
    for g in seeds {
        let d = &g.diff;
        let _ = writeln!(
            out,
            "  seeds, part {} F on {}: first difference at {:?} (shots, hits, rows differ: {:?}), shots fired then in some runs only {}: {:?}, latest first shot {:?}, each side's first shot {:?}, first death {:?}, deaths differ {}, first hit difference {:?}, first death difference {:?}, first row difference {:?}, vehicles that had fired before the first difference (F, G) {:?}, every vehicle's first shot the same in the seeds {}",
            g.part.text(),
            g.side.text(),
            d.first,
            d.differs,
            d.extras.len(),
            d.extras
                .iter()
                .map(|e| (e.firer, e.g, e.ordinal, e.later.clone()))
                .collect::<Vec<_>>(),
            d.latest_first_shot,
            d.side_first,
            d.first_death,
            d.deaths_differ,
            d.first_hit_diff,
            d.first_death_diff,
            d.first_row_diff,
            d.fired_before,
            d.first_shots_same,
        );
    }
    out.push_str(&verdicts_text(runs, seeds));
    out
}

/// The verdicts, one line per row, and the count of rows failing (the line
/// `M-0021: rows failing: N of M`).
pub fn verdicts_text(runs: &[((Part, Side, u32), Findings)], seeds: &[SeedGroup]) -> String {
    let mut out = String::new();
    let v = verdicts(runs, seeds);
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
    let mut recs: Vec<((Part, Side, u32), Recording)> = Vec::new();
    let mut errors = String::new();
    for (&(p, s, seed), r) in jobs.iter().zip(results) {
        match r {
            Ok(rec) => {
                judged.push(((p, s, seed), judge(&rec, p)));
                recs.push(((p, s, seed), rec));
            }
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
    // measure: M-0021 (Procedure step 12: parts A and B, each side, the three seeds compared)
    let mut seeds = Vec::new();
    for part in [Part::A, Part::B] {
        for side in [Side::A, Side::B] {
            let group: Vec<&Recording> = recs
                .iter()
                .filter(|((p, s, _), _)| *p == part && *s == side)
                .map(|(_, r)| r)
                .collect();
            if group.len() >= 2 {
                seeds.push(SeedGroup {
                    part,
                    side,
                    diff: compare_seeds(&group),
                });
            }
        }
    }
    let summary = format!("{errors}{}", verdicts_text(&judged, &seeds));
    let observer = grounds.values().next().map(|g| observer_building(&g.rules));
    let full = format!(
        "{errors}observer's building type (local): {observer:?}\n{}",
        report(&built, &judged, &seeds)
    );
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
                observer_building: None,
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

    // measure: M-0021 (Setup, Houses: the observer's starting construction vehicle is off the map at tick 0 and gone from tick 1 on; creation order, F's group then G's whichever side F stands on)
    #[test]
    fn the_observer_keeps_no_vehicle_and_f_is_created_first() {
        let g = ground(Part::A);
        let opts = |building: Option<&str>| SetupOptions {
            count: Some(GROUP),
            clear_neutral: true,
            observer_building: building.map(str::to_owned),
            ..Default::default()
        };
        let kept = crate::oracle::setup_with(&g, [names::F, names::G], Side::A, 0, &opts(None))
            .expect("set up");
        let human = |a: &crate::oracle::Arena| {
            a.world
                .objects
                .values()
                .filter(|o| o.owner == Some(crate::oracle::HUMAN))
                .count()
        };
        assert_eq!(human(&kept), 1);
        // A building type the rules lack is an error, not a silent skip.
        assert!(
            crate::oracle::setup_with(&g, [names::F, names::G], Side::A, 0, &opts(Some("X")))
                .is_err()
        );
        // With a building type (the synthetic rules have none), the observer owns that building only.
        if let Some(b) = observer_building(&g.rules) {
            let off =
                crate::oracle::setup_with(&g, [names::F, names::G], Side::A, 0, &opts(Some(&b)))
                    .expect("set up");
            assert_eq!(human(&off), 1);
        }
        // Side B's group stands first in the numbers when F stands on side B.
        let fb = crate::oracle::setup_with(&g, [names::G, names::F], Side::B, 0, &opts(None))
            .expect("set up");
        assert!(fb.units[1][0] < fb.units[0][0]);
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
        // measure: M-0021 (Parts and seeds: the per-tick rows end one tick before the recording's last tick)
        for t in 0..40 {
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
            firer_pos: None,
            target_pos: None,
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
        let v = verdicts(&[((Part::A, Side::A, 0), f.clone())], &[]);
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
        for t in 15..40 {
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

    // measure: M-0021 (Procedure step 2: the last shot's window runs to the run's end, and step 3: a shot fired at the run's last tick has no per-tick row and is not judged)
    #[test]
    fn a_shot_at_the_runs_last_tick_is_not_judged() {
        // A last shot just before the end is judged with its hits, even late ones.
        let mut late = hand_made();
        late.shots[2].tick = 39;
        late.hits.retain(|h| h.tick < 30);
        late.hits.push(HitRec {
            tick: 40,
            id: ObjectId(2),
            warhead: Some(0),
            attacker: Some(ObjectId(1)),
            health: (100, 93),
        });
        let f = judge(&late, Part::A);
        assert_eq!((f.judged.len(), f.not_judged), (2, 1));
        // A shot at the last tick (40) is not judged.
        let mut last = hand_made();
        last.shots[2].tick = 40;
        let f = judge(&last, Part::A);
        assert_eq!((f.judged.len(), f.not_judged), (1, 2));
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
        // The synthetic run is cut at tick 300 + 8, and a shot in its last
        // ticks owns fewer hits: only the shots up to the limit are checked.
        let full: Vec<&Judged> = f
            .judged
            .iter()
            .filter(|j| j.c >= 3 && j.tick <= 300)
            .collect();
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

    fn pos(x: i32) -> Pos {
        Pos { x, y: 0, z: 0 }
    }

    // spec: S-0203 claim 3 and S-0310 claim 9 (measure: M-0021, Procedure and Notes, Reading points: the shot rows' positions are those at the shot, the firer after its own movement, a target that has not had its turn before the tick's move)
    #[test]
    fn the_shot_rows_positions_follow_the_update_order() {
        let rec = play_to(&ground(Part::A), Part::A, Side::A, 0, 200).expect("plays");
        assert!(!rec.shots.is_empty());
        for s in &rec.shots {
            let t = s.tick;
            if !rec.rows.contains_key(&t) {
                continue; // the run's last tick has no per-tick rows
            }
            assert_eq!(s.firer_pos, Some(rec.rows[&t][&s.firer].0), "the firer");
            if let Target::Object(x) = s.target
                && (rec.f.contains(&x) || rec.g.contains(&x))
            {
                let tp = s.target_pos.expect("an object target has a position");
                assert!(
                    tp == rec.rows[&(t - 1)][&x].0 || tp == rec.rows[&t][&x].0,
                    "the target's position is a per-tick row's"
                );
            }
        }
        let f = judge(&rec, Part::A);
        // The derived distances pass the engine's own firing test.
        assert!(f.reach.shots > 0);
        assert_eq!((f.reach.beyond, f.reach.unread), (0, 0));
        // F's group is created first, so it takes its turns first: nobody reads a firer at t − 1, and a G
        // shot reads its F target at t.
        let r = f.reading.shot_rows;
        assert_eq!((r[0].at_before, r[0].neither), (0, 0));
        assert_eq!((r[1].at_t, r[1].neither), (0, 0));
        assert_eq!((r[2].at_before, r[2].neither), (0, 0));
        assert_eq!((r[3].at_before, r[3].neither), (0, 0));
    }

    // spec: S-0203 claim 6 (a) (a destroyed vehicle leaves play at the hit; measure: M-0021, Procedure: the positions as the shot's own row gives them): part B's shots at a vehicle that the shot or a later hit of the tick destroys still have both positions
    #[test]
    fn a_destroyed_vehicles_shot_still_has_its_positions() {
        let rec = play_to(&ground(Part::B), Part::B, Side::A, 0, 900).expect("plays");
        let f = judge(&rec, Part::B);
        assert!(f.dead.0 + f.dead.1 > 0, "some vehicle died in the run");
        assert!(!rec.shots.is_empty());
        for s in &rec.shots {
            assert!(s.firer_pos.is_some(), "tick {}", s.tick);
            if let Target::Object(x) = s.target
                && (rec.f.contains(&x) || rec.g.contains(&x))
            {
                assert!(s.target_pos.is_some(), "tick {}", s.tick);
            }
        }
        // The destroyed vehicle has no row at the tick of its death (S-0203 claim 6 (a)): its last row is the one before.
        assert_eq!(f.reading.deaths_last_at_t, 0);
        assert_eq!(f.reading.deaths, f.reading.deaths_other.len());
    }

    // measure: M-0021 (Procedure step 11 and Result, Reading points: the row that first shows a hit's health after, a destroyed vehicle's last row, the row that holds a shot row's position)
    #[test]
    fn the_reading_points_are_read_from_the_rows() {
        let (f, x, dead, gone) = (ObjectId(1), ObjectId(2), ObjectId(3), ObjectId(4));
        let mut rec = Recording {
            f: vec![f],
            g: vec![x, dead, gone],
            weapons: [0, 1, 2, 3],
            warheads: [0, 1],
            end: 5,
            ..Default::default()
        };
        // The target moves during tick 2 (1500 to 1450), the firer during tick 2 as well (0 to 40).
        for t in 0..5 {
            let mut r = BTreeMap::new();
            let moved = t >= 2;
            r.insert(f, (pos(if moved { 40 } else { 0 }), 100));
            r.insert(
                x,
                (
                    pos(if moved { 1450 } else { 1500 }),
                    if t >= 2 { 86 } else { 100 },
                ),
            );
            // `dead` is destroyed at tick 2 and still a row at 2 with no health; `gone` is destroyed at 2 and has no row at 2.
            if t <= 2 {
                r.insert(dead, (pos(900), if t == 2 { 0 } else { 50 }));
            }
            if t <= 1 {
                r.insert(gone, (pos(800), 50));
            }
            rec.positions
                .insert(t, BTreeMap::from([(f, r[&f].0), (x, r[&x].0)]));
            rec.rows.insert(t, r);
        }
        let hit = |id, health| HitRec {
            tick: 2,
            id,
            warhead: Some(0),
            attacker: Some(f),
            health,
        };
        // Two hits on x in the tick: only the last one's health after can show in a row.
        rec.hits = vec![
            hit(x, (100, 93)),
            hit(x, (93, 86)),
            hit(dead, (50, 0)),
            hit(gone, (50, 0)),
        ];
        // An F shot at tick 2: the firer as the row stamped 2 has it, the target as the row stamped 1.
        rec.shots = vec![ShotRec {
            tick: 2,
            firer: f,
            target: Target::Object(x),
            weapon: 0,
            firer_pos: Some(pos(40)),
            target_pos: Some(pos(1500)),
        }];
        let j = judge(&rec, Part::B);
        let r = &j.reading;
        assert_eq!((r.hit_at_t, r.hit_at_before, r.hit_unseen), (2, 0, 0));
        assert_eq!(
            (r.hit_superseded, r.hit_before_mismatch, r.hit_no_row),
            (1, 0, 1)
        );
        assert_eq!((r.deaths, r.deaths_last_at_t), (2, 1));
        assert_eq!(r.deaths_other.len(), 1);
        assert_eq!(r.shot_rows[0].at_t, 1);
        assert_eq!(r.shot_rows[1].at_before, 1);
        // The distance 1460 from the shot row's positions is within F's range of 1536; the per-tick distances 1500 and 1410 are too.
        let first = j.first[0].as_ref().expect("F fired");
        assert_eq!(first.shooters.len(), 1);
        let s = &first.shooters[0];
        assert_eq!(s.shot, Some(1460 * 1460));
        assert_eq!(s.rows, (Some(1500 * 1500), Some(1410 * 1410)));
        assert_eq!(s.range2, 1536 * 1536);
    }

    // measure: M-0021 (Result, First shots: the first-shot rows read the shot row's positions and the per-tick rows stamped t − 1 and t)
    #[test]
    fn the_first_shot_rows_read_the_shot_rows_positions() {
        let (f, x) = (ObjectId(1), ObjectId(2));
        let mut rec = Recording {
            f: vec![f],
            g: vec![x],
            weapons: [0, 1, 2, 3],
            warheads: [0, 1],
            end: 9,
            ..Default::default()
        };
        // The per-tick distance stamped 4 is 1530 and stamped 5 is 1520: the shot row's (1527) is neither.
        for (t, d) in [(4, 1530), (5, 1520)] {
            rec.positions
                .insert(t, BTreeMap::from([(f, pos(0)), (x, pos(d))]));
        }
        rec.shots = vec![ShotRec {
            tick: 5,
            firer: f,
            target: Target::Object(x),
            weapon: 0,
            firer_pos: Some(pos(0)),
            target_pos: Some(pos(1527)),
        }];
        let j = judge(&rec, Part::A);
        let v = verdicts(&[((Part::A, Side::A, 0), j)], &[]);
        let row = |s: &str| {
            v.iter()
                .find(|(r, _)| r.starts_with(s))
                .map(|(_, v)| v.clone())
        };
        assert_eq!(
            row("first shots: centre distance at F's first shot"),
            Some(Verdict::Pass)
        );
        assert_eq!(
            row("first shots: F's target within F's range"),
            Some(Verdict::Pass)
        );
        // A distance from the per-tick rows stamped t alone (1520) would not be the record's 1527.
        assert!(!rounds_to(1520 * 1520, FIRST_SHOT_DISTANCE[0]));
    }

    // measure: M-0021 (Procedure step 12 and Result, Seeds: the first tick at which the seeds' runs differ, the vehicles matched by creation order)
    #[test]
    fn the_seeds_first_difference_is_found_by_creation_order() {
        let (f, g0, g1) = (ObjectId(1), ObjectId(2), ObjectId(3));
        // Three runs whose vehicles carry other numbers in each (matching is by place).
        let run = |shift: u32, second: i32| {
            let (f, g0, g1) = (
                ObjectId(f.0 + shift),
                ObjectId(g0.0 + shift),
                ObjectId(g1.0 + shift),
            );
            let shot = |tick, firer, target| ShotRec {
                tick,
                firer,
                target: Target::Object(target),
                weapon: 2,
                firer_pos: None,
                target_pos: None,
            };
            Recording {
                f: vec![f],
                g: vec![g0, g1],
                shots: vec![
                    shot(3, f, g0),
                    shot(4, g0, f),
                    shot(5, g1, f),
                    shot(second, g0, f),
                ],
                end: 12,
                ..Default::default()
            }
        };
        let (a, b, c) = (run(0, 9), run(10, 9), run(20, 11));
        let same = compare_seeds(&[&a, &b]);
        assert_eq!(same.first, None);
        let d = compare_seeds(&[&a, &b, &c]);
        assert_eq!(d.first, Some(9));
        assert_eq!(d.differs, (true, false, false));
        assert_eq!(d.extras.len(), 1);
        let e = &d.extras[0];
        assert_eq!((e.firer, e.g, e.ordinal), (1, true, 2));
        assert_eq!(e.later, vec![Some((2, true))]);
        assert_eq!(d.latest_first_shot, Some(5));
        assert_eq!(d.side_first, [Some(3), Some(4)]);
        assert_eq!((d.first_death, d.deaths_differ), (None, false));
        assert!(d.first_shots_same);
        assert_eq!(d.fired_before, (1, 2));
        // The seed rows pass for such a difference, and fail for runs that agree.
        let group = |diff| SeedGroup {
            part: Part::A,
            side: Side::A,
            diff,
        };
        let rows = seed_verdicts(&[group(d)]);
        for (name, v) in &rows {
            if name.starts_with("seeds: part B") {
                continue;
            }
            assert!(!matches!(v, Verdict::Fail(_)), "{name}: {v:?}");
        }
        let rows = seed_verdicts(&[group(same)]);
        assert!(matches!(&rows[0].1, Verdict::Fail(_)));
    }

    // measure: M-0021 (Procedure step 12: a difference in a hit's health or in a per-tick row counts as well as one in a shot)
    #[test]
    fn the_seeds_first_difference_can_be_a_hit_or_a_row() {
        let (f, g) = (ObjectId(1), ObjectId(2));
        let base = |shift: u32| Recording {
            f: vec![ObjectId(f.0 + shift)],
            g: vec![ObjectId(g.0 + shift)],
            end: 8,
            ..Default::default()
        };
        let with_hit = |shift: u32, health: (i32, i32)| {
            let mut r = base(shift);
            r.hits.push(HitRec {
                tick: 6,
                id: ObjectId(g.0 + shift),
                warhead: Some(0),
                attacker: Some(ObjectId(f.0 + shift)),
                health,
            });
            r
        };
        let d = compare_seeds(&[&with_hit(0, (100, 93)), &with_hit(10, (100, 90))]);
        assert_eq!((d.first, d.differs), (Some(6), (false, true, false)));
        // A per-tick row that differs from tick 4 on.
        let with_row = |shift: u32, x: i32| {
            let mut r = base(shift);
            for t in 0..8 {
                let at = if t >= 4 { x } else { 0 };
                r.rows
                    .insert(t, BTreeMap::from([(ObjectId(g.0 + shift), (pos(at), 100))]));
            }
            r
        };
        let d = compare_seeds(&[&with_row(0, 5), &with_row(10, 6)]);
        assert_eq!((d.first, d.differs), (Some(4), (false, false, true)));
        assert!(d.extras.is_empty());
    }
}
