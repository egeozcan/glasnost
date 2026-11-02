//! An aircraft released above its pad (S-0501 claim 48, revision 52,
//! answering Q-1841; its object-present flag by claim 48 (e), revision 55,
//! answering Q-1875), docking by landing that moves only horizontally
//! (claim 24) and pad guidance by the docking position's horizontal
//! location (claim 40), on synthetic data through the real order and tick
//! path. The scenario shapes follow AC30 part k and AC14.
//!
//! Every type, weapon and number here is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready, set_credits, t};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::combat::flying;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, Target, World};

const ALPHA: HouseId = HouseId(0);

/// The pads: (name, docking displacement).
const PADS: [(&str, [i32; 3]); 9] = [
    ("HIPAD", [0, 0, 100]),
    ("LOWPAD", [0, 0, 15]),
    ("TENPAD", [0, 0, 10]),
    ("MIDPAD", [128, 128, 10]),
    ("SIDEPAD", [128, 28, 10]),
    ("OUTPAD", [384, 128, 10]),
    ("WESTPAD", [-640, 128, 10]),
    ("OFFPAD", [64, 64, 0]),
    ("FLATPAD", [0, 0, 0]),
];

fn owners() -> Option<Vec<String>> {
    Some(vec!["Alphaland".into(), "Betaland".into()])
}

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

fn flyer(weapon: Option<&str>) -> UnitType {
    let mut u = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(1000),
        primary: weapon.map(Into::into),
        ..Default::default()
    };
    u.combat.ammo = Some(if weapon.is_some() { 2 } else { 0 });
    u.econ.landable = Some(true);
    u.econ.dock = Some(PADS.iter().map(|(n, _)| (*n).into()).collect());
    u
}

/// The rules: the economy ruleset with the combat ruleset's weapons and
/// warheads, and
/// - the pads of [`PADS`] (TENPAD is AC30 part k (7)'s second pad; MIDPAD
///   docks at 10 leptons above the middle of a foundation cell, so that a
///   destination straight ahead can be a cell whose centre it passes over), each an aircraft factory, helipad and reload pad
///   with one docking place and the given displacement;
/// - JET (armed with Agm, 2 rounds, pad-bound), as AC30 part k's type;
/// - NEAR (JET with a slowdown distance of 2000 leptons, AC30 part k (4));
/// - SCOUT (unarmed, not pad-bound), the control that lands on open ground;
/// - SLOW (JET with a speed of 5 leptons per tick, so that a forward step
///   at a speed fraction below 0.2 is 0 leptons and only the docking
///   approach moves it horizontally while it lands, claim 4);
/// - DRIFT (SCOUT with a slowdown distance of 2000 leptons: a target
///   fraction below 1 near its destination, and a landing at the trigger
///   since it is not pad-bound);
/// - NEAR6 (JET with a slowdown distance of 600 leptons, below the 768 of
///   claim 5's widest distance band).
///
/// SIDEPAD docks 100 leptons beside the middle row of a foundation cell
/// (inside the 128 of claim 5's first band, outside the 86 of its halving
/// zone). OUTPAD docks at the middle of the cell east of its foundation,
/// and WESTPAD two cells west of it, so that their docking positions lie
/// outside the foundation (claim 48 (c), Open question 26).
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    for (n, d) in PADS {
        let mut pad = factory("aircraft");
        pad.helipad = Some(true);
        pad.econ.unit_reload = Some(true);
        pad.econ.number_of_docks = Some(1);
        pad.econ.docking_offsets = Some(vec![d]);
        push(&mut r, "buildings", n);
        r.sim.buildings.insert(n.into(), pad);
    }
    r.sim.weapons.insert(
        "Agm".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: econ_common::dec("6"),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    let mut jet = flyer(Some("Agm"));
    jet.econ.airport_bound = Some(true);
    let mut near = jet.clone();
    near.slowdown_distance = Some(2000);
    let mut slow = jet.clone();
    slow.speed = Some(5);
    let mut drift = flyer(None);
    drift.slowdown_distance = Some(2000);
    let mut near6 = jet.clone();
    near6.slowdown_distance = Some(600);
    for (n, u) in [
        ("NEAR6", near6),
        ("JET", jet),
        ("NEAR", near),
        ("SCOUT", flyer(None)),
        ("SLOW", slow),
        ("DRIFT", drift),
    ] {
        push(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    r
}

/// AC30 part k (7)'s named test rules: [`rules`] with the game-wide pose
/// direction (S-0404 Import mapping) set to 64, so that every release onto
/// a pad faces 64 × 256 = 16384 facing units, map east (S-0404 claim 13).
fn rules_k7() -> Ruleset {
    let mut r = rules();
    // spec: S-0501 AC30 part k (7) and Import mapping (PoseDir: the facing of a release onto a pad, set in part k (7)'s named test rules) and S-0404 claim 13 (v × 256 facing units)
    r.sim.general.economy.pose_dir = Some(64);
    r
}

/// The released facing of AC30 part k (7): map east.
const EAST: u16 = 16384; // spec: S-0501 AC30 part k (7) (every release from either pad faces 16384, map east)

fn open(w: &mut World) {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
}

/// A person's move order to `cell`.
fn move_to(w: &mut World, a: ObjectId, cell: glasnost_map::CellXY) {
    open(w);
    assert!(orders::issue(
        w,
        ALPHA,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(cell)),
        }
    ));
}

/// A person's order sending `a` into the pad `k`.
fn enter(w: &mut World, a: ObjectId, k: ObjectId) {
    open(w);
    assert!(orders::issue(
        w,
        ALPHA,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(k)),
        }
    ));
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn pos(w: &World, id: ObjectId) -> Pos {
    w.obj(id).unwrap().pos
}

fn tracked(w: &World, id: ObjectId) -> bool {
    flying::tracked(w, id)
}

/// Whether `id` is listed in either cell layer of its cell (S-0209 claim
/// 13).
fn listed(w: &World, id: ObjectId) -> bool {
    let c = pos(w, id).cell();
    [Layer::Ground, Layer::Deck]
        .iter()
        .any(|&l| w.cells.list(c, l).contains(&id))
}

fn describe(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    format!(
        "t={} pos={:?} h={} beh={:?}/{:?} dest={:?} contact={:?} tracked={} listed={} hp={} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.beh.current,
        o.beh.queued,
        o.dest,
        o.contact,
        tracked(w, id),
        listed(w, id),
        o.health,
        o.air
    )
}

fn run_until(w: &mut World, max: i32, mut done: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        step(w);
    }
    done(w)
}

/// Builds `name` at a fresh `pad` of house 0, in the middle of the map;
/// returns (world, pad, aircraft) in the tick of the release.
fn built_at(r: &Ruleset, pad: &str, name: &str) -> (World, ObjectId, ObjectId) {
    built_rally(r, pad, name, None)
}

/// [`built_at`] with the pad's rally point at `rally` (S-0213 claim 20,
/// S-0404 claim 13).
fn built_rally(
    r: &Ruleset,
    pad: &str,
    name: &str,
    rally: Option<glasnost_map::CellXY>,
) -> (World, ObjectId, ObjectId) {
    built_watch(r, pad, name, rally, |_, _| {}, |_, _| {})
}

/// [`built_rally`] with `prep` run once on the world and the pad before the
/// build order, and `watch` on every tick before the release.
fn built_watch(
    r: &Ruleset,
    pad: &str,
    name: &str,
    rally: Option<glasnost_map::CellXY>,
    prep: impl FnOnce(&mut World, ObjectId),
    mut watch: impl FnMut(&World, ObjectId),
) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, pad, 0, 0, 0);
    w.obj_mut(k).unwrap().guard_spot = rally.map(Target::Cell);
    prep(&mut w, k);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let what = t(&w, Kind::Aircraft, name);
    assert!(issue(&mut w, 0, Order::Build { what }));
    let ti = what.index;
    let found = run_until(&mut w, 3000, |w| {
        let out = w
            .objects
            .values()
            .any(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map);
        if !out {
            watch(w, k);
        }
        out
    });
    assert!(found, "the aircraft is built");
    let a = w
        .objects
        .values()
        .find(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map)
        .unwrap()
        .id;
    (w, k, a)
}

/// The index of the synthetic warhead HE.
fn he() -> u32 {
    rules().sim.warheads.keys().position(|k| k == "HE").unwrap() as u32
}

/// Area damage (S-0303 claim 6) of HE at the terrain under `id`; returns
/// whether `id` lost health.
fn blast_under(w: &mut World, id: ObjectId) -> bool {
    let p = pos(w, id);
    let at = Pos::new(p.x, p.y, w.surface_at(p.x, p.y));
    let before = w.obj(id).unwrap().health;
    let wh = he();
    glasnost_sim::combat::projectile::detonate_area(w, wh, 100, at, None, None, false);
    w.obj(id).unwrap().health < before
}

/// A SCOUT of house 0 landed on open ground at offset (dx, dy) from the
/// middle after a move order, the positive control of AC30 part i (1).
fn landed_control(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let c = testkit::spawn(
        w,
        Kind::Aircraft,
        "SCOUT",
        0,
        testkit::mid(SIZE, dx - 4, dy),
        0,
    );
    move_to(w, c, testkit::mid(SIZE, dx, dy));
    assert!(run_until(w, 50, |w| !st(w, c).ground));
    let down = run_until(w, 2000, |w| st(w, c).ground);
    assert!(down, "the control lands: {}", describe(w, c));
    c
}

/// The released aircraft at rest: at the release height, not settled, not
/// landing and with no take-off, its held height 0, in neither cell layer,
/// not tracked and filed in the ground render layer, in contact with its
/// pad (claim 48 (a) to (d)).
fn assert_resting(w: &World, k: ObjectId, a: ObjectId, h: i32) {
    let s = st(w, a);
    assert_eq!(air::height(w, a), h, "{}", describe(w, a));
    assert!(!air::on_ground(w, a), "in flight by its height");
    assert!(
        !s.ground && !s.landing && !s.taking_off,
        "{}",
        describe(w, a)
    );
    assert_eq!(s.held, 0, "the landing height held from its placement");
    assert!(s.released && air::ground_filed(w, a), "{}", describe(w, a));
    assert!(!listed(w, a), "in neither cell layer: {}", describe(w, a));
    assert!(!tracked(w, a), "not tracked: {}", describe(w, a));
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(w, a));
}

// spec: S-0501 claim 48 (a) to (d) and AC30 part k (2) (released at its docking position, height included, it rests at 100 leptons above level ground with no order: its height and position unchanged, in neither cell layer, not tracked, filed in the ground render layer, in contact with its pad and in guard; area damage passes over it while it reaches the landed control); S-0404 claim 15 (the docking position, height included)
#[test]
fn claim_48_release_rests_above_its_pad() {
    let r = rules();
    let (mut w, k, a) = built_at(&r, "HIPAD", "JET");
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    assert_eq!(pos(&w, a), p, "at the docking position, height included");
    assert_resting(&w, k, a, 100);
    for _ in 0..300 {
        step(&mut w);
        assert_eq!(
            pos(&w, a),
            p,
            "unchanged with no order: {}",
            describe(&w, a)
        );
        assert_resting(&w, k, a, 100);
    }
    // spec: S-0501 claim 48 (d) (its placement on the ground gave it guard, T2's ground row for an aircraft that is not visiting; the release gives it no other behaviour)
    assert_eq!(
        w.obj(a).unwrap().beh.current,
        Some(Behaviour::Guard),
        "{}",
        describe(&w, a)
    );
    let c = landed_control(&mut w, 0, 6);
    assert!(blast_under(&mut w, c), "the control takes a hit");
    assert!(
        !blast_under(&mut w, a),
        "area damage passes over it: {}",
        describe(&w, a)
    );
    assert_resting(&w, k, a, 100);
}

// spec: S-0501 claim 48 (d) (an unarmed aircraft compares its height with its cruise height for equality only: at the release height it rests as well)
#[test]
fn claim_48_an_unarmed_release_rests_too() {
    let r = rules();
    let (mut w, k, a) = built_at(&r, "HIPAD", "SCOUT");
    let p = pos(&w, a);
    for _ in 0..200 {
        step(&mut w);
        assert_eq!(pos(&w, a), p, "{}", describe(&w, a));
        assert_resting(&w, k, a, 100);
    }
}

// spec: S-0501 Edge cases (quirks.aircraft_released_above_pad off: the release puts the aircraft at the docking position's horizontal location at the landing height above the terrain there, settled and listed in its cell layer like an aircraft docked by landing; area damage reaches it, and it leaves by an ordinary take-off, tracked from it)
#[test]
fn claim_48_quirk_off_settles_at_the_landing_height() {
    let mut r = rules();
    r.globals.quirks.aircraft_released_above_pad = false;
    let (mut w, k, a) = built_at(&r, "HIPAD", "JET");
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    let q = pos(&w, a);
    assert_eq!((q.x, q.y), (p.x, p.y));
    assert_eq!(air::height(&w, a), 0, "{}", describe(&w, a));
    let s = st(&w, a);
    assert!(s.ground && !s.released && !air::ground_filed(&w, a));
    assert!(listed(&w, a), "listed in its cell layer");
    assert!(!tracked(&w, a));
    // spec: S-0501 claim 48 (e) and Edge cases (off: it holds the object-present flag in the ground record of the docking position's cell, as an aircraft docked by landing does, and in no other recorded cell)
    let dock = p.cell();
    assert_eq!(st(&w, a).mark, Some(dock), "{}", describe(&w, a));
    assert_eq!(flags(&w, dock), (true, false));
    assert_flags_only(&w, &around(&w, k), Some(dock));
    testkit::run(&mut w, 20);
    assert_eq!(flags(&w, dock), (true, false), "kept at rest");
    assert!(blast_under(&mut w, a), "area damage reaches it");
    move_to(&mut w, a, testkit::mid(SIZE, 10, 0));
    let off = run_until(&mut w, 50, |w| st(w, a).taking_off);
    assert!(off, "an ordinary take-off: {}", describe(&w, a));
    assert!(tracked(&w, a), "tracked from the take-off (claim 47 (a))");
    assert!(!air::ground_filed(&w, a));
    // spec: S-0501 claim 44 (the take-off clears its flag)
    step(&mut w);
    assert_flags_only(&w, &around(&w, k), None);
}

// spec: S-0501 Terms and claim 8 (the take-off test reads height: an aircraft at height 0 that is neither settled nor landing is on the ground, and a new destination makes it take off)
#[test]
fn claim_8_take_off_test_reads_height() {
    let r = rules();
    let (mut w, k, a) = built_at(&r, "FLATPAD", "SCOUT");
    // spec: S-0501 claim 44 (released at its docking height 0, outside claim 48: it rests on its pad holding its flag in its cell's ground record)
    let c = pos(&w, a).cell();
    assert_flags_only(&w, &around(&w, k), Some(c));
    testkit::run(&mut w, 10);
    assert_eq!(air::height(&w, a), 0, "{}", describe(&w, a));
    move_to(&mut w, a, testkit::mid(SIZE, 10, 0));
    // The order is applied; its movement update has not taken the
    // destination yet.
    step(&mut w);
    assert!(w.obj(a).unwrap().dest.is_some() && st(&w, a).flying_to.is_none());
    // Neither settled nor landing at height 0, and off its pad's contact
    // so that no pad departure starts the take-off.
    if let Some(o) = w.obj_mut(a) {
        o.contact = None;
        o.econ.exit = None;
        if let Some(s) = o.air.as_mut() {
            s.ground = false;
            s.landing = false;
        }
    }
    let off = run_until(&mut w, 20, |w| {
        // spec: S-0501 claim 8 (the movement update that takes the destination takes off: no landing comes first)
        assert!(!st(w, a).landing, "{}", describe(w, a));
        st(w, a).taking_off
    });
    assert!(off, "it takes off: {}", describe(&w, a));
    assert!(tracked(&w, a), "tracked from the take-off (claim 47 (a))");
}

/// Per-tick samples of `a` from the order on: (height, position, current
/// and target fraction, tracked, listed, ground-filed).
#[derive(Debug)]
struct Sample {
    h: i32,
    p: Pos,
    cur: glasnost_num::F64,
    tgt: glasnost_num::F64,
    tracked: bool,
    listed: bool,
    filed: bool,
}

fn sample(w: &World, a: ObjectId) -> Sample {
    Sample {
        h: air::height(w, a),
        p: pos(w, a),
        cur: st(w, a).cur,
        tgt: st(w, a).tgt,
        tracked: tracked(w, a),
        listed: listed(w, a),
        filed: air::ground_filed(w, a),
    }
}

/// Gives the released `a` a move order to `dest` and steps until the first
/// tick whose sample differs from the rest; returns the samples of the
/// first and second movement updates (one tick apart) and the rest
/// before them.
fn leave(w: &mut World, a: ObjectId, dest: glasnost_map::CellXY) -> (Sample, Sample, Sample) {
    let rest = sample(w, a);
    move_to(w, a, dest);
    let mut moved = false;
    for _ in 0..20 {
        step(w);
        let s = sample(w, a);
        if s.h != rest.h || s.p != rest.p {
            moved = true;
            break;
        }
        // spec: S-0501 claim 48 (c) (a destination given while it rests starts no take-off, and it is not tracked at the order)
        assert!(!st(w, a).taking_off && !tracked(w, a), "{}", describe(w, a));
    }
    assert!(moved, "it leaves: {}", describe(w, a));
    let first = sample(w, a);
    step(w);
    let second = sample(w, a);
    (rest, first, second)
}

// spec: S-0501 claim 48 (b) and (c) and AC30 part k (3) (leaving far: in its first movement update no forward step, a descent of 20 leptons by F2 to 80 and only claim 4's 5-lepton descent nudge toward the destination cell's centre, its current fraction becoming 0.1; it is not tracked at the order; in the second it is tracked, takes a forward step and climbs by F1 toward its cruise height; it stays filed in the ground render layer while it leaves); claim 48 (c) pad contact (its contact ends at the order)
#[test]
fn claim_48_leaving_far() {
    let r = rules();
    let (mut w, k, a) = built_at(&r, "HIPAD", "JET");
    testkit::run(&mut w, 10);
    let dest = testkit::mid(SIZE, 10, 0);
    let (rest, first, second) = leave(&mut w, a, dest);
    assert_ne!(w.obj(a).unwrap().contact, Some(k), "the contact ended");
    // spec: S-0501 F2 (δ = min(50, max(20, ⌊(h − FLₕ) / 20⌋)): 20 from 100 to the held height 0)
    assert_eq!(first.h, 80, "a descent of 20 to 80");
    let to = Pos::centre(dest, 0);
    let (dx, dy) = (first.p.x - rest.p.x, first.p.y - rest.p.y);
    assert!(
        dx.abs() <= 5 && dy.abs() <= 5 && (dx, dy) != (0, 0),
        "only the 5-lepton nudge: ({dx}, {dy})"
    );
    assert!(
        (to.x - first.p.x).abs() <= (to.x - rest.p.x).abs()
            && (to.y - first.p.y).abs() <= (to.y - rest.p.y).abs(),
        "toward the destination cell's centre"
    );
    // spec: S-0501 claim 5 (the current fraction goes 0 to 0.1 in the first update)
    assert_eq!(first.cur, glasnost_sim::rules::nearest_double("0.1"));
    assert!(!first.tracked && !first.listed && first.filed);
    // spec: S-0501 claim 48 (b) (above the terrain after that descent: the held height of ordinary flight at the end of the same update)
    assert_eq!(st(&w, a).held, 500, "{}", describe(&w, a));
    // spec: S-0501 claim 47 (c) and claim 48 (c) (tracked in the second update; a forward step with the carried 0.1; a climb by F1)
    assert!(second.tracked, "{}", describe(&w, a));
    // spec: S-0501 F1 (a climb step of 20 without passengers)
    assert_eq!(second.h, 100, "a climb by F1");
    assert!(
        (second.p.x - first.p.x, second.p.y - first.p.y) != (0, 0),
        "a forward step"
    );
    assert!(second.filed && !second.listed);
    // spec: S-0501 claim 48 (c) (nothing re-files it as it leaves: the ground render layer until its landing starts)
    let mut landing = false;
    for _ in 0..400 {
        step(&mut w);
        if st(&w, a).landing || air::on_ground(&w, a) {
            landing = true;
            break;
        }
        assert!(air::ground_filed(&w, a), "{}", describe(&w, a));
    }
    assert!(landing, "it lands somewhere: {}", describe(&w, a));
}

// spec: S-0501 claim 48 (b) and AC30 part k (4) (leaving near: a destination 512 to 767 leptons away and nearer than the slowdown distance; the same as far, the climb in the second update included, although the target fraction stays below 1)
#[test]
fn claim_48_leaving_near() {
    let r = rules();
    let (mut w, _k, a) = built_at(&r, "HIPAD", "NEAR");
    testkit::run(&mut w, 10);
    let p = pos(&w, a);
    let dest = (1..4)
        .map(|d| glasnost_map::CellXY::new(p.cell().x + d, p.cell().y))
        .find(|&c| {
            let q = Pos::centre(c, 0);
            let d = (q.x - p.x).abs().max((q.y - p.y).abs());
            (512..768).contains(&d)
        })
        .expect("a cell 512 to 767 leptons away");
    let (_, first, second) = leave(&mut w, a, dest);
    assert_eq!(first.h, 80, "{}", describe(&w, a));
    // spec: S-0501 claim 5 (the approach target replacement: 0.75 from 512 to below 768 leptons) and AC30 part k (4) (a target fraction of 0.75 at the end of the first update, which ends at 80 leptons, above the terrain, Q-1941)
    assert!(
        first
            .tgt
            .eq_num(glasnost_sim::rules::nearest_double("0.75")),
        "{first:?}"
    );
    assert!(
        st(&w, a).tgt.lt(glasnost_sim::rules::nearest_double("1")),
        "target fraction below 1"
    );
    assert_eq!(second.h, 100, "the climb in the second update");
    assert!(second.tracked);
}

// spec: S-0501 claim 48 (c) (the low release: released at most 20 leptons up, the first update's descent reaches 0 and it is listed in its cell's ground layer at the end of that update, (b)'s end-of-update rule not applying, not yet tracked; Low release on a move: it takes off at the move's first processing, before its second movement update, tracked at that take-off, claim 47 (a), its held height its cruise height; in its second update it climbs by F1 and leaves the ground listing, and that update, its first after the take-off starts, re-files it into the top render layer, S-0102 claim 4, Q-1942)
#[test]
fn claim_48_low_release() {
    let r = rules();
    let (mut w, k, a) = built_at(&r, "LOWPAD", "JET");
    assert_resting(&w, k, a, 15);
    testkit::run(&mut w, 10);
    let (_, first, second) = leave(&mut w, a, testkit::mid(SIZE, 10, 0));
    assert_eq!(first.h, 0, "{}", describe(&w, a));
    assert!(first.listed && !first.tracked, "listed, not yet tracked");
    assert!(first.filed, "still filed in the ground render layer");
    assert!(second.tracked, "tracked at its take-off");
    // spec: S-0501 F1 (a climb step of 20 without passengers)
    assert_eq!(second.h, 20, "{}", describe(&w, a));
    assert!(!second.listed, "it left the ground listing");
    assert!(!second.filed, "re-filed into the top render layer");
    assert!(st(&w, a).taking_off, "{}", describe(&w, a));
    assert_eq!(st(&w, a).held, air::cruise_height(&w, a));
}

// spec: S-0501 claim 48 (c) and (d) and S-0404 claim 13 (with a rally point, a type that need not return to a dock gets a move order to it at the release, which is not a behaviour order: the release gives the rally point as its destination and queues move, which becomes current in its first movement update; its docking position lies in a cell of the pad's foundation, so claim 8's rule ends its contact as it takes that destination; no take-off; its first movement update descends from 100 to 80 by F2, and its second climbs)
#[test]
fn claim_48_rally_point_at_the_release() {
    let r = rules();
    let dest = testkit::mid(SIZE, 10, 0);
    let (mut w, k, a) = built_rally(&r, "HIPAD", "SCOUT", Some(dest));
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(dest)), "{}", describe(&w, a));
    assert_ne!(o.contact, Some(k), "the contact ended at the release");
    assert_eq!(o.beh.queued, Some(Behaviour::Move), "{}", describe(&w, a));
    assert_ne!(o.beh.current, Some(Behaviour::Move), "{}", describe(&w, a));
    assert_eq!(air::height(&w, a), 100);
    assert!(!st(&w, a).taking_off && !tracked(&w, a));
    let mut hs = Vec::new();
    let moved = run_until(&mut w, 20, |w| {
        hs.push(air::height(w, a));
        air::height(w, a) != 100
    });
    assert!(moved, "{hs:?} {}", describe(&w, a));
    assert_eq!(air::height(&w, a), 80, "{}", describe(&w, a));
    assert!(!st(&w, a).taking_off, "no take-off: {}", describe(&w, a));
    // spec: S-0501 claim 31 (the queued move becomes current at the first start opportunity of its first movement update)
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Move));
    step(&mut w);
    // spec: S-0501 F1 (a climb step of 20 without passengers)
    assert_eq!(air::height(&w, a), 100, "{}", describe(&w, a));
    assert!(tracked(&w, a), "{}", describe(&w, a));
}

// spec: S-0501 claim 24 and AC30 part k (1) (docking by landing at the raised pad: the height component of the docking displacement is never applied; it rests at the landing height 0 and area damage reaches it)
#[test]
fn claim_24_docking_at_a_raised_pad_rests_at_the_landing_height() {
    let r = rules();
    let (mut w, k, a) = built_at(&r, "HIPAD", "JET");
    testkit::run(&mut w, 10);
    move_to(&mut w, a, testkit::mid(SIZE, 8, 0));
    let away = run_until(&mut w, 400, |w| {
        let o = w.obj(a).unwrap();
        o.contact.is_none() && air::height(w, a) >= 400
    });
    assert!(away, "{}", describe(&w, a));
    enter(&mut w, a, k);
    let docked = run_until(&mut w, 3000, |w| {
        st(w, a).ground && w.obj(a).unwrap().beh.current != Some(Behaviour::Enter)
    });
    assert!(docked, "docks: {}", describe(&w, a));
    assert_eq!(air::height(&w, a), 0, "{}", describe(&w, a));
    assert!(st(&w, a).ground && !st(&w, a).released);
    assert!(!air::ground_filed(&w, a), "re-filed by the landing");
    assert!(listed(&w, a));
    testkit::run(&mut w, 30);
    assert_eq!(air::height(&w, a), 0, "at rest: {}", describe(&w, a));
    assert!(blast_under(&mut w, a), "area damage reaches it");
}

// spec: S-0501 claim 24 and AC14 (the processing of enter that finds it landing in contact advances to the docking stage; from the next processing, 3 ticks later, until touchdown, each processing 1 tick after the one before moves it toward the docking position by the difference along each axis limited to −5 to +5 leptons, each axis separately, its height unchanged by the move: no step in the first 2 ticks after the landing starts, the first 3 to 6 ticks after it; it rests at the landing height). A pad-bound type at 5 leptons per tick makes no forward step at the low speed fraction of its landing (claim 4), so each tick's horizontal change is the docking step alone
#[test]
fn claim_24_docking_steps_are_clamped() {
    let r = rules();
    let (mut w, k, a) = built_at(&r, "OFFPAD", "SLOW");
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    assert_eq!(air::height(&w, a), 0, "the displacement has no height");
    testkit::run(&mut w, 10);
    // Sent south-east and ordered back to its pad once in the air (claim 24).
    // spec: S-0501 claim 50 (r93: a type of 10 ≤ V < 20, here 12, can hover short of its destination with a positive fraction at claim 5's stalling facings, as on this south-easterly heading, and claim 6's exact-cell trigger reads the ordinary target, so no refused landing returns it; the order does)
    let to = testkit::mid(SIZE, 3, 2);
    move_to(&mut w, a, to);
    let away = run_until(&mut w, 1000, |w| {
        let o = w.obj(a).unwrap();
        let c = o.pos.cell();
        o.contact.is_none() && c.x >= to.x - 1 && c.y >= to.y - 1
    });
    assert!(away, "{}", describe(&w, a));
    enter(&mut w, a, k);
    let back = run_until(&mut w, 3000, |w| {
        let o = w.obj(a).unwrap();
        o.contact == Some(k) && o.beh.current == Some(Behaviour::Enter) && st(w, a).landing
    });
    assert!(back, "{}", describe(&w, a));
    let start = w.tick;
    let mut last = pos(&w, a);
    // AC14's void condition: more than 5 leptons to go along an axis at the landing's start.
    assert!((last.x - p.x).abs() > 5 || (last.y - p.y).abs() > 5);
    let mut steps = Vec::new();
    let down = run_until(&mut w, 400, |w| {
        let q = pos(w, a);
        if w.tick > start {
            steps.push((
                w.tick - start,
                q.x - last.x,
                q.y - last.y,
                last.x - p.x,
                last.y - p.y,
            ));
        }
        last = q;
        st(w, a).ground
    });
    assert!(down, "{}", describe(&w, a));
    // spec: S-0501 claim 24 (each axis by the remaining difference limited to −5 … 5 leptons, separately)
    let clamp = |d: i32| (-d).clamp(-5, 5);
    let docking = |s: &(i32, i32, i32, i32, i32)| {
        (s.1, s.2) != (0, 0) && (s.1, s.2) == (clamp(s.3), clamp(s.4))
    };
    let first = steps
        .iter()
        .position(docking)
        .expect("docking steps before touchdown");
    // spec: S-0501 claim 24 and AC14 (no step in the first 2 ticks after the landing starts; the first 3 to 6 ticks after it, the docking stage starting 3 ticks after the processing that finds it landing)
    assert!(
        (3..=6).contains(&steps[first].0),
        "first step at +{}: {steps:?}",
        steps[first].0
    );
    // spec: S-0501 claim 24 (from then on every processing, 1 tick apart, until touchdown)
    for s in &steps[first..] {
        assert_eq!((s.1, s.2), (clamp(s.3), clamp(s.4)), "+{}: {steps:?}", s.0);
    }
    assert!(
        steps[first..].iter().any(|s| (s.1, s.2) == (-5, -5)),
        "a step limited on both axes: {steps:?}"
    );
    assert!(
        steps[first..]
            .iter()
            .any(|s| (s.1, s.2) == (-5, 0) && s.4 == 0),
        "an axis already used up stays: {steps:?}"
    );
    let q = pos(&w, a);
    assert_eq!(
        (q.x, q.y),
        (p.x, p.y),
        "the difference used up before touchdown"
    );
    assert_eq!(air::height(&w, a), 0, "{}", describe(&w, a));
}

// spec: S-0501 claim 48 (a) and Changes for S-0309 claim 7 (a map-wide search's ground-layer pass takes the objects filed in the ground render layer's list, whatever their height, so it takes the released aircraft; once a re-file has moved it out of that list, only the aircraft pass would consider it, and a searcher with ground weapons runs no aircraft pass). The searcher is a vehicle, whose connected-region test for this aircraft is Open question 25; both stand in one region of open level ground here, so the test does not depend on it. The pad is removed first so that the released aircraft is the only enemy object
#[test]
fn claim_48_map_wide_ground_pass_takes_the_released_aircraft() {
    use glasnost_sim::combat::acquire::{SearchContext, SearchKind, search};
    let mut r = rules();
    r.sim.vehicles.get_mut("CRUSHER").unwrap().primary = Some("Gun".into());
    let (mut w, k, a) = built_at(&r, "HIPAD", "JET");
    glasnost_sim::objects::remove(&mut w, k);
    let hunter = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CRUSHER",
        1,
        testkit::mid(SIZE, -8, -8),
        0,
    );
    assert!(air::ground_filed(&w, a) && air::height(&w, a) == 100);
    let found = search(&mut w, hunter, SearchKind::MapWide, SearchContext::Hunt);
    assert_eq!(found, Some(a), "{}", describe(&w, a));
    w.obj_mut(hunter).unwrap().target = None;
    w.obj_mut(a).unwrap().air.as_mut().unwrap().ground_filed = false;
    let found = search(&mut w, hunter, SearchKind::MapWide, SearchContext::Hunt);
    assert_eq!(found, None, "{}", describe(&w, a));
}

// ---------------------------------------------------------------------------
// Claim 48 (e): the object-present flag (revision 55, Q-1875), AC30 part k
// (6) and (7). The recorder reads, without writing, the flag of the ground
// and the deck record and both occupant lists.

/// The object-present flags (S-0209 claim 1) of `c`'s ground and deck
/// records.
fn flags(w: &World, c: glasnost_map::CellXY) -> (bool, bool) {
    let f = |l| w.cells.occ(c, l).is_some_and(|o| o.others > 0);
    (f(Layer::Ground), f(Layer::Deck))
}

/// The recorded cells: the pad's foundation and every cell next to it (a
/// square of 7 by 7 cells around the pad's position, which covers its 2 by
/// 2 foundation and its neighbours whatever the anchor).
fn around(w: &World, k: ObjectId) -> Vec<glasnost_map::CellXY> {
    let c = pos(w, k).cell();
    (-3..=3)
        .flat_map(|dy| (-3..=3).map(move |dx| glasnost_map::CellXY::new(c.x + dx, c.y + dy)))
        .collect()
}

/// Every cell of `cells` has both object-present flags clear, except
/// `only`, whose ground flag is set and whose deck flag is clear.
fn assert_flags_only(
    w: &World,
    cells: &[glasnost_map::CellXY],
    only: Option<glasnost_map::CellXY>,
) {
    for &c in cells.iter().chain(only.iter()) {
        let want = (only == Some(c), false);
        assert_eq!(flags(w, c), want, "t={} cell {c:?}", w.tick);
    }
}

/// `a` is in neither occupant list of any cell of `cells`.
fn assert_unlisted(w: &World, cells: &[glasnost_map::CellXY], a: ObjectId) {
    for &c in cells {
        for l in [Layer::Ground, Layer::Deck] {
            assert!(!w.cells.list(c, l).contains(&a), "t={} {c:?} {l:?}", w.tick);
        }
    }
}

// spec: S-0501 claim 48 (e) and AC30 part k (6) (from the release tick on, while it rests and through the first three movement updates of a far move order, both object-present flags are clear and the aircraft is in neither list in every cell of the pad's foundation and next to it and in each cell it is in: no flag of its ground placement is kept, on the docking position's cell or another)
#[test]
fn claim_48_e_no_flag_from_the_release_on() {
    let r = rules();
    let (mut w, k, a) = built_watch(
        &r,
        "HIPAD",
        "JET",
        None,
        |_, _| {},
        |w, k| {
            // The tick before the release, and every one before it.
            assert_flags_only(w, &around(w, k), None);
        },
    );
    let mut cells = around(&w, k);
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    assert!(
        cells.contains(&p.cell()),
        "the docking position's cell is recorded"
    );
    let check = |w: &World, cells: &mut Vec<glasnost_map::CellXY>| {
        let c = pos(w, a).cell();
        if !cells.contains(&c) {
            cells.push(c);
        }
        assert_flags_only(w, cells, None);
        assert_unlisted(w, cells, a);
        assert_eq!(st(w, a).mark, None, "{}", describe(w, a));
    };
    // The release tick, then at rest.
    check(&w, &mut cells);
    for _ in 0..20 {
        step(&mut w);
        check(&w, &mut cells);
    }
    assert_resting(&w, k, a, 100);
    // The move order, and its first three movement updates.
    move_to(&mut w, a, testkit::mid(SIZE, 10, 0));
    check(&w, &mut cells);
    let rest = pos(&w, a);
    let mut updates = 0;
    for _ in 0..30 {
        step(&mut w);
        check(&w, &mut cells);
        if updates > 0 || pos(&w, a) != rest {
            updates += 1;
        }
        if updates == 3 {
            break;
        }
    }
    assert_eq!(updates, 3, "{}", describe(&w, a));
    assert!(air::height(&w, a) > 0, "{}", describe(&w, a));
}

// spec: S-0501 claim 48 (e) (after a release, the ground record of the cell holding the docking position's horizontal location has no object-present flag, even if another object had contributed one to that record before the release: S-0209 claim 1, the flags are not counts). The other contribution is written straight into the record before the build, standing for another object's; it is still set on every tick before the release
#[test]
fn claim_48_e_release_clears_the_docking_cells_ground_flag() {
    let r = rules();
    let dock = std::cell::Cell::new(None);
    let (w, k, a) = built_watch(
        &r,
        "HIPAD",
        "JET",
        None,
        |w, k| {
            let c = glasnost_sim::econ::release::pad_position_at(w, k, 0).cell();
            w.cells.occ_mut(c, Layer::Ground).unwrap().others = 1;
            dock.set(Some(c));
        },
        |w, _| assert_eq!(flags(w, dock.get().unwrap()), (true, false), "t={}", w.tick),
    );
    let (p, _) = glasnost_sim::econ::release::pad_position(&w, k, a);
    assert_eq!(Some(p.cell()), dock.get());
    assert_eq!(flags(&w, p.cell()), (false, false), "{}", describe(&w, a));
    assert_resting(&w, k, a, 100);
}

/// AC30 part k (7)'s per-update record of the low release: the height,
/// the cell, and whether it is listed in that cell's ground layer.
fn low_update(
    w: &World,
    a: ObjectId,
    cells: &mut Vec<glasnost_map::CellXY>,
) -> (i32, glasnost_map::CellXY, bool) {
    let c = pos(w, a).cell();
    if !cells.contains(&c) {
        cells.push(c);
    }
    (
        air::height(w, a),
        c,
        w.cells.list(c, Layer::Ground).contains(&a),
    )
}

// spec: S-0501 claim 48 (c) and (e) and AC30 part k (7) (a) (the low release at 10 leptons with a far move order: at the end of its first movement update at height 0, listed in its cell's ground layer and holding the ground flag there and in no other recorded cell; it takes off at the move's first processing, and at the end of its second update, which climbs, a positive height and no flag and no listing in any recorded cell, as after claim 44's take-off)
#[test]
fn claim_48_e_low_release_climbs_and_clears() {
    let r = rules_k7();
    let (mut w, k, a) = built_at(&r, "TENPAD", "JET");
    assert_resting(&w, k, a, 10);
    let mut cells = around(&w, k);
    assert_flags_only(&w, &cells, None);
    testkit::run(&mut w, 10);
    assert_flags_only(&w, &cells, None);
    let rest = pos(&w, a);
    move_to(&mut w, a, testkit::mid(SIZE, 10, 0));
    let moved = run_until(&mut w, 20, |w| pos(w, a) != rest);
    assert!(moved, "{}", describe(&w, a));
    let (h, c, listed) = low_update(&w, a, &mut cells);
    assert_eq!(h, 0, "{}", describe(&w, a));
    assert!(listed, "in its cell's ground layer: {}", describe(&w, a));
    assert_flags_only(&w, &cells, Some(c));
    step(&mut w);
    let (h, _, _) = low_update(&w, a, &mut cells);
    assert!(h > 0, "it climbs: {}", describe(&w, a));
    assert_flags_only(&w, &cells, None);
    assert_unlisted(&w, &cells, a);
    assert_eq!(st(&w, a).mark, None);
}

/// The cell `dist` leptons straight ahead of `a` along its body facing.
fn ahead(w: &World, a: ObjectId, dist: i32) -> glasnost_map::CellXY {
    let o = w.obj(a).unwrap();
    let (x, y) = glasnost_sim::walk::step_point(
        o.pos.x,
        o.pos.y,
        o.body.value(w.tick),
        dist,
        true,
        w.rules().mode,
    );
    Pos::new(x, y, 0).cell()
}

/// The planar distance of S-0204 claim 5 (the truncated table square root
/// of the squared planar difference).
fn dist(w: &World, p: Pos, q: Pos) -> i32 {
    let (dx, dy) = (i64::from(p.x - q.x), i64::from(p.y - q.y));
    glasnost_num::geom::root_floor((dx * dx + dy * dy) as u64, true, w.rules().mode) as i32
}

/// The cells of pad `k`'s foundation.
fn pad_feet(w: &World, k: ObjectId) -> Vec<glasnost_map::CellXY> {
    let b = w.obj(k).unwrap();
    glasnost_sim::econ::placement::foundation_cells(w, b.type_index, b.pos.cell())
}

/// The centre of `c` at height 0.
fn centre(c: glasnost_map::CellXY) -> Pos {
    Pos::centre(c, 0)
}

/// One sample of AC30 part k (7)'s recorder, at the end of a movement
/// update: height, held height, horizontal position, body facing, speed
/// fractions, behaviour (current, queued), destination and cell, with its
/// listing in that cell's ground layer and its render layer (whether it is
/// still filed in the ground render layer).
#[derive(Clone, Debug)]
struct Up {
    h: i32,
    held: i32,
    pos: Pos,
    cell: glasnost_map::CellXY,
    listed: bool,
    filed: bool,
    facing: u16,
    tgt: glasnost_num::F64,
    beh: (Option<Behaviour>, Option<Behaviour>),
    dest: Option<Target>,
    contact: Option<ObjectId>,
    landing: bool,
    taking_off: bool,
    ground: bool,
}

fn sample_up(w: &World, a: ObjectId, cells: &mut Vec<glasnost_map::CellXY>) -> Up {
    let (h, cell, listed) = low_update(w, a, cells);
    let o = w.obj(a).unwrap();
    let s = st(w, a);
    Up {
        h,
        held: s.held,
        pos: o.pos,
        cell,
        listed,
        filed: air::ground_filed(w, a),
        facing: o.body.value(w.tick),
        tgt: s.tgt,
        beh: (o.beh.current, o.beh.queued),
        dest: o.dest,
        contact: o.contact,
        landing: s.landing,
        taking_off: s.taking_off,
        ground: s.ground,
    }
}

/// The result of a low-release run: the world, the pad, the aircraft, the
/// destination, the recorded cells and the samples.
type Run = (
    World,
    ObjectId,
    ObjectId,
    glasnost_map::CellXY,
    Vec<glasnost_map::CellXY>,
    Vec<Up>,
);

/// A low release of `name` at `pad` under `r`, given the destination `dest`
/// by `give` after 10 ticks at rest. From its first movement update on,
/// while `go` holds (at most 600 updates), it samples each update with
/// [`sample_up`] and checks the recorder: at height 0 the flag is in the
/// cell it is in and in no other recorded cell; above it, in none. Every
/// cell it is in is usable (AC30 part k (7): void if it reaches the map's
/// edge).
fn low_run_by(
    r: &Ruleset,
    pad: &str,
    name: &str,
    dest: impl Fn(&World, ObjectId) -> glasnost_map::CellXY,
    give: impl FnOnce(&mut World, ObjectId, glasnost_map::CellXY),
    mut go: impl FnMut(&World, ObjectId, &[Up]) -> bool,
) -> Run {
    let (mut w, k, a) = built_at(r, pad, name);
    assert_resting(&w, k, a, 10);
    // spec: S-0501 AC30 part k (7) (void unless the released facing is 16384) and S-0404 claim 13
    assert_eq!(w.obj(a).unwrap().body.value(w.tick), EAST);
    testkit::run(&mut w, 10);
    let p = pos(&w, a);
    let to = dest(&w, a);
    let mut cells = around(&w, k);
    give(&mut w, a, to);
    let moved = run_until(&mut w, 20, |w| pos(w, a) != p);
    assert!(moved, "{}", describe(&w, a));
    let mut samples = Vec::new();
    loop {
        let s = sample_up(&w, a, &mut cells);
        assert_flags_only(&w, &cells, (s.h == 0).then_some(s.cell));
        assert!(w.cells.usable(s.cell), "void: the map's edge: {s:?}");
        samples.push(s);
        if !go(&w, a, &samples) || samples.len() >= 600 {
            break;
        }
        step(&mut w);
    }
    (w, k, a, to, cells, samples)
}

/// A destination given without a person's order, as a behaviour of the
/// aircraft's own gives it: straight to the locomotor (S-0501 claim 48 (c),
/// *Pad contact*, reads only the cell; S-0213 table T3 (b) applies only to
/// an order).
fn assign(w: &mut World, a: ObjectId, to: glasnost_map::CellXY) {
    glasnost_sim::econ::assign_destination(w, a, Some(Target::Cell(to)));
}

/// A destination cell whose centre lies at a right angle to the released
/// facing (map east), `rows` rows north of the aircraft's cell.
fn abeam(w: &World, a: ObjectId, rows: i32) -> glasnost_map::CellXY {
    let c = pos(w, a).cell();
    glasnost_map::CellXY::new(c.x, c.y - rows)
}

/// AC30 part k (7)'s expectations common to every run, on the samples of
/// [`low_run_by`] (index 0 the end of the first movement update):
/// - at the end of the first update, height 0 and listed in its cell's
///   ground layer, still on the released facing (its flag there is checked
///   by the recorder);
/// - at the end of the second, after the take-off at the move's first
///   processing (claim 8): a positive height, its cruise height as held
///   height, taking off, in neither cell layer of any recorded cell and no
///   longer filed in the ground render layer (S-0102 claim 4);
/// - no change of body facing up to and including the first update that
///   ends above trunc(*H* / 2).
///
/// Returns the index of that update.
fn k7_expect(w: &World, a: ObjectId, cells: &[glasnost_map::CellXY], samples: &[Up]) -> usize {
    let first = &samples[0];
    // spec: S-0501 claim 48 (c) and (e) and AC30 part k (7) (height 0 at the end of the first movement update, listed in the ground layer of its cell, with that cell's ground flag set)
    assert_eq!(first.h, 0, "{first:?}");
    assert!(first.listed, "in its cell's ground layer: {first:?}");
    assert!(!first.taking_off && !first.landing, "{first:?}");
    assert_eq!(first.facing, EAST, "{first:?}");
    let second = samples
        .get(1)
        .unwrap_or_else(|| panic!("a second update: {first:?}"));
    let cruise = air::cruise_height(w, a);
    // spec: S-0501 claim 48 (c) (Low release on a move: the move's first processing gives the destination again, kept or replaced, claim 40, and an aircraft on the ground by its height takes off, claim 8: held height its cruise height, tracked at that take-off; in the second update it climbs and leaves the ground listing) and AC30 part k (7) (falsified by height 0 or a held height of 0 at the end of the second update, kept or replaced)
    assert!(second.h > 0, "it climbs: {second:?}");
    assert_eq!(second.held, cruise, "{second:?}");
    assert!(second.taking_off, "a take-off: {second:?}");
    assert!(!second.listed, "{second:?}");
    assert_unlisted(w, cells, a);
    // spec: S-0501 claim 48 (c) (for a landable type the second movement update is its first after the take-off starts and re-files it into the top render layer, S-0102 claim 4) and AC30 part k (7) (falsified by the ground render layer at the end of the second update)
    assert!(!second.filed, "re-filed: {second:?}");
    // spec: S-0501 claim 8 (at or below trunc(H / 2) no new heading request: it goes on along the released facing, which the release also requested) and AC30 part k (7) (falsified by a change of body facing before the first update that ends above trunc(H / 2))
    let half = cruise / 2;
    let n = samples
        .iter()
        .position(|s| s.h > half)
        .unwrap_or_else(|| panic!("above trunc(H / 2): {:?}", samples.last()));
    for (u, s) in samples.iter().take(n + 1).enumerate() {
        assert_eq!(s.facing, EAST, "update {u}: {s:?}");
    }
    n
}

/// Runs [`low_run_by`] until `extra` updates after the first that ends
/// above trunc(*H* / 2).
fn low_run_k7(
    pad: &str,
    dest: impl Fn(&World, ObjectId) -> glasnost_map::CellXY,
    give: impl FnOnce(&mut World, ObjectId, glasnost_map::CellXY),
    extra: usize,
) -> Run {
    let r = rules_k7();
    let half = std::cell::Cell::new(None);
    low_run_by(&r, pad, "NEAR", dest, give, |w, a, s| {
        let n = s.len() - 1;
        if half.get().is_none() && s[n].h > air::cruise_height(w, a) / 2 {
            half.set(Some(n));
        }
        half.get().is_none_or(|m| n < m + extra)
    })
}

/// The bearing from a sample's position to the centre of `to`.
fn bearing(s: &Up, to: glasnost_map::CellXY) -> u16 {
    glasnost_num::geom::direction(
        i64::from(centre(to).x - s.pos.x),
        i64::from(centre(to).y - s.pos.y),
        true,
    )
}

/// After the update `n`, the body turns toward `to`: the last sample's
/// facing is nearer the bearing than the facing at `n` (S-0206 claim 1).
fn turns_toward(samples: &[Up], n: usize, to: glasnost_map::CellXY) {
    let off = |s: &Up| glasnost_num::geom::delta(s.facing, bearing(s, to)).abs();
    let (at, last) = (&samples[n], samples.last().unwrap());
    // spec: S-0501 claim 8 (above trunc(H / 2), up to and including the upper threshold, its body is requested to face the destination) and AC30 part k (7) (from then on a turn toward the destination)
    assert!(
        last.facing != EAST && off(last) < off(at),
        "{at:?} {last:?}"
    );
}

// spec: S-0501 claim 48 (c) and (e) and AC30 part k (7) (a) (a move order as in (3): the low release takes off at the move's first processing, before its second movement update; at the end of that update a positive height, no flag and no listing in any recorded cell, as after claim 44's take-off)
#[test]
fn ac30_k7_a_far_move_takes_off_at_the_first_processing() {
    let (w, _, a, to, cells, samples) =
        low_run_k7("TENPAD", |_, _| testkit::mid(SIZE, 10, 0), move_to, 5);
    k7_expect(&w, a, &cells, &samples);
    assert_flags_only(&w, &cells, None);
    assert_eq!(st(&w, a).mark, None);
    assert_eq!(
        samples[1].dest,
        Some(Target::Cell(to)),
        "kept: {:?}",
        samples[1]
    );
    // spec: S-0501 claim 47 (a) (tracked at the take-off)
    assert!(tracked(&w, a), "{}", describe(&w, a));
}

// spec: S-0501 claims 5 and 42 and AC30 part k (7) (b) (a move order as in (4), 512 to 767 leptons straight ahead along the released facing: at the end of the first update, at height 0, claim 5's ordinary target fraction, no approach target replacement, since guidance runs only above the terrain, Q-1941; then the take-off of every run)
#[test]
fn ac30_k7_b_straight_ahead_takes_off_with_the_ordinary_fraction() {
    let (w, _, a, to, cells, samples) = low_run_k7(
        "MIDPAD",
        |w, a| {
            let p = pos(w, a);
            let to = ahead(w, a, 512);
            let m = centre(to);
            // spec: S-0501 AC30 part k (7) (b) (a cell whose centre lies within 20 leptons of the straight line ahead along that facing, 512 to 767 leptons away)
            assert!((m.y - p.y).abs() <= 20 && (512..768).contains(&dist(w, p, m)));
            to
        },
        move_to,
        5,
    );
    let first = &samples[0];
    let mode = w.rules().mode;
    let s = rules_k7().sim.aircraft["NEAR"].slowdown_distance.unwrap();
    let d = dist(&w, first.pos, centre(to));
    // spec: S-0501 claim 5 (τ = min(1, d / S) in ordinary slowing) and AC30 part k (4) (the named type's slowdown distance chosen so that d / S differs from 0.75 and 0.5 by at least 0.05)
    let ordinary = glasnost_num::F64::from_i64(i64::from(d), mode)
        .div(glasnost_num::F64::from_i64(i64::from(s), mode), mode);
    assert!(first.tgt.eq_num(ordinary), "{first:?} d {d} S {s}");
    for x in ["0.75", "0.5"] {
        assert!(
            !first.tgt.eq_num(glasnost_sim::rules::nearest_double(x)),
            "{first:?}"
        );
    }
    k7_expect(&w, a, &cells, &samples);
}

// spec: S-0501 claim 48 (c) and AC30 part k (7) (c) (a move order as in (4) at a right angle to the released facing: the ordinary target fraction at the end of the first update, the take-off at the move's first processing, the released facing kept up to trunc(H / 2) and then a turn toward the destination; the take-off does not depend on the destination's direction)
#[test]
fn ac30_k7_c_right_angle_takes_off_then_turns() {
    let (w, _, a, to, cells, samples) = low_run_k7("MIDPAD", |w, a| abeam(w, a, 2), move_to, 12);
    let first = &samples[0];
    for x in ["0.75", "0.5"] {
        assert!(
            !first.tgt.eq_num(glasnost_sim::rules::nearest_double(x)),
            "{first:?}"
        );
    }
    let n = k7_expect(&w, a, &cells, &samples);
    turns_toward(&samples, n, to);
}

// spec: S-0501 claim 40 and AC30 part k (7) (d) (a move order naming a cell of the pad's own foundation next to the docking position's cell, which claim 40 replaces at the move's first processing: the replaced destination is given to the aircraft at that processing, and it takes off as with a kept one, turning toward the replacement cell above trunc(H / 2)); void if the destination after that processing is the order's cell. MIDPAD docks above the middle of a foundation cell, so that claim 4's descent nudge keeps it in the docking cell: from TENPAD's corner docking position the nudge can carry it into the destination cell, which F7 then accepts as the cell that holds the lander itself
#[test]
fn ac30_k7_d_replaced_destination_takes_off_too() {
    let (w, k, a, to, cells, samples) = low_run_k7(
        "MIDPAD",
        |w, a| {
            let c = pos(w, a).cell();
            let o = w.obj(a).unwrap();
            let feet = glasnost_sim::econ::placement::foundation_cells(
                w,
                w.obj(o.contact.unwrap()).unwrap().type_index,
                w.obj(o.contact.unwrap()).unwrap().pos.cell(),
            );
            *feet
                .iter()
                .find(|q| **q != c && (q.x - c.x).abs() <= 1 && (q.y - c.y).abs() <= 1)
                .expect("a foundation cell next to the docking cell")
        },
        move_to,
        12,
    );
    assert!(pad_feet(&w, k).contains(&to));
    // spec: S-0501 AC30 part k (7) (d) (void if the recorded behaviour after the order is not move, or if the recorded destination after that processing is the order's cell)
    assert_eq!(samples[0].beh.0, Some(Behaviour::Move), "{:?}", samples[0]);
    let Some(Target::Cell(got)) = samples[1].dest else {
        panic!("a cell destination: {:?}", samples[1]);
    };
    assert_ne!(
        got,
        to,
        "void: claim 40 kept the order's cell: {:?} {}",
        &samples[..3],
        describe(&w, a)
    );
    let n = k7_expect(&w, a, &cells, &samples);
    if bearing(&samples[n], got) != EAST {
        turns_toward(&samples, n, got);
    }
}

// spec: S-0501 claim 40 (the move's first processing gives the selected destination to the aircraft, and the move's next processing gives its flight point to the flight once more)
#[test]
fn claim_40_the_next_processing_gives_the_flight_point_once_more() {
    let r = rules_k7();
    let start = std::cell::Cell::new(None);
    let (w, _, a, to, _, samples) = low_run_by(
        &r,
        "TENPAD",
        "NEAR",
        |_, _| testkit::mid(SIZE, 10, 0),
        move_to,
        |w, a, _| {
            let o = w.obj(a).unwrap();
            match (start.get(), st(w, a).regive) {
                (None, Some(t)) => {
                    // spec: S-0501 claim 40 (given at the move's first processing: the move's start)
                    assert_eq!(o.beh.current, Some(Behaviour::Move), "{}", describe(w, a));
                    assert_eq!(t, o.beh.start_tick, "{}", describe(w, a));
                    start.set(Some(t));
                    true
                }
                (Some(t), Some(u)) => {
                    assert_eq!(t, u);
                    true
                }
                (Some(_), None) => false,
                (None, None) => true,
            }
        },
    );
    let t = start
        .get()
        .unwrap_or_else(|| panic!("no first processing: {samples:?}"));
    let o = w.obj(a).unwrap();
    // spec: S-0501 claim 40 (the next processing of the same move gives the point once more: the flight's point is the destination's, an unfinished flight)
    assert_eq!(o.beh.current, Some(Behaviour::Move), "{}", describe(&w, a));
    assert_eq!(o.beh.start_tick, t, "the same move: {}", describe(&w, a));
    assert_eq!(st(&w, a).flying_to, Some(Target::Cell(to)));
    assert!(st(&w, a).unfinished);
    assert_eq!(
        st(&w, a).regive,
        None,
        "a later processing: {}",
        describe(&w, a)
    );
    assert!(
        (2..600).contains(&samples.len()),
        "a later processing: {samples:?}"
    );
}

// spec: S-0501 claim 48 (c) (Pad contact: for a pad-bound type a behaviour order ends the contact at the order wherever the docking position lies, S-0213 table T3 (b), the type refusing the release's tether, S-0417 claim 11, Q-1940) and AC30 part k (7) (e) (from a docking position outside the foundation, a person's move order as in (3): contact and no tether at the release; no contact at the first sample after the order, on either object; then the take-off of every run)
#[test]
fn ac30_k7_e_person_order_ends_a_pad_bound_contact_outside_the_foundation() {
    let r = rules_k7();
    let (w, k, a) = built_at(&r, "OUTPAD", "NEAR");
    // spec: S-0417 claim 11 (docked at K: contact; a type that must return to a dock refuses the tether)
    assert_eq!(w.obj(a).unwrap().contact, Some(k));
    assert!(!glasnost_sim::econ::release::tethered(&w, a));
    assert!(!pad_feet(&w, k).contains(&pos(&w, a).cell()));
    drop(w);
    let (w, k, a, _, cells, samples) =
        low_run_k7("OUTPAD", |_, _| testkit::mid(SIZE, 10, 0), move_to, 5);
    assert!(!pad_feet(&w, k).contains(&samples[0].cell));
    // spec: S-0501 AC30 part k (7) (e) (no contact at the first sample after the order, on either object: the pad has no aircraft in contact)
    assert!(
        samples.iter().all(|s| s.contact.is_none()),
        "{:?}",
        samples[0]
    );
    assert!(w.objects.values().all(|o| o.contact != Some(k)));
    k7_expect(&w, a, &cells, &samples);
}

// spec: S-0501 claim 48 (c) and (d) (the rally point's move at the release is not a behaviour order; a type that need not return to a dock is tethered, and its contact ends only by claim 8's rule, which reads the cell of its horizontal position: from a docking position outside the foundation it keeps its contact as it takes off at the move's first processing) and S-0417 claim 11 (the tether ends when it leaves the ground)
#[test]
fn claim_48_c_tethered_type_keeps_its_contact_outside_the_foundation() {
    let r = rules_k7();
    let dest = testkit::mid(SIZE, 10, 0);
    let (mut w, k, a) = built_rally(&r, "OUTPAD", "SCOUT", Some(dest));
    assert_resting(&w, k, a, 10);
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
    assert!(glasnost_sim::econ::release::tethered(&w, a));
    let p = pos(&w, a);
    let moved = run_until(&mut w, 20, |w| pos(w, a) != p);
    assert!(moved, "{}", describe(&w, a));
    assert_eq!(air::height(&w, a), 0, "{}", describe(&w, a));
    step(&mut w);
    assert!(air::height(&w, a) > 0, "{}", describe(&w, a));
    assert!(st(&w, a).taking_off, "{}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
    assert!(!glasnost_sim::econ::release::tethered(&w, a));
}

// design: S-0501 claim 48 (c) and Open question 26 (whether a low release given its destination by a behaviour other than move takes off as one on a move does is open). Our design: it takes off at the start of its next movement update at height 0 with that destination, so no low release makes a movement update at height 0 with a destination but without a take-off. The destination is given straight to the locomotor, without a behaviour order or a move (as the attack of claim 48 (d) gives one); a pad-bound type from a docking position outside the foundation keeps its contact as it takes off (Pad contact)
#[test]
fn open_question_26_destination_without_a_move_takes_off_too() {
    let (w, k, a, _, cells, samples) =
        low_run_k7("OUTPAD", |_, _| testkit::mid(SIZE, 10, 0), assign, 5);
    assert_ne!(samples[0].beh.0, Some(Behaviour::Move), "{:?}", samples[0]);
    k7_expect(&w, a, &cells, &samples);
    for s in &samples {
        // spec: S-0501 claim 48 (c) (Pad contact: a docking position outside the pad's foundation leaves the contact held after a destination without a behaviour order)
        assert_eq!(s.contact, Some(k), "{s:?}");
    }
}

// design: S-0501 claim 48 (c) and Open question 26 (a pad-bound aircraft that holds its contact in flight after a destination without a behaviour order, with a destination cell of that pad: claim 6's pad-bound restriction does not hold there). Our design: claim 6 as written, so it starts to land (claim 7), and its touchdown in a cell of the pad it is in contact with docks it there, outside enter (claim 24)
#[test]
fn open_question_26_destination_on_its_pad_lands_and_docks() {
    let (mut w, k, a, to, _, samples) = low_run_by(
        &rules_k7(),
        "WESTPAD",
        "NEAR",
        |w, a| ahead(w, a, 512),
        assign,
        |_, _, s| s.last().is_some_and(|s| !s.landing && !s.ground),
    );
    // The destination is a cell of the pad's foundation; the docking position's cell is not.
    let feet = pad_feet(&w, k);
    assert!(feet.contains(&to) && !feet.contains(&samples[0].cell));
    assert!(samples[1].taking_off, "{:?}", samples[1]);
    let z = samples
        .iter()
        .position(|s| s.landing)
        .unwrap_or_else(|| panic!("its landing starts: {}", describe(&w, a)));
    assert_eq!(samples[z].contact, Some(k));
    let down = run_until(&mut w, 400, |w| st(w, a).ground);
    assert!(down, "{}", describe(&w, a));
    assert_eq!(air::height(&w, a), 0);
    assert!(feet.contains(&pos(&w, a).cell()), "{}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
}

// spec: S-0501 Edge cases (quirks.aircraft_released_above_pad off: the release settles it at the landing height, and it takes off at the order, as any aircraft on the ground does, claim 8, which steers it toward its destination; the low release on a move would take off only at the move's first processing, after a first movement update that descends). MIDPAD, facing 16384, a destination at a right angle: it climbs in its first movement update and its body turns toward the destination during the take-off
#[test]
fn claim_48_quirk_off_takes_off_toward_a_right_angle_destination() {
    let mut r = rules_k7();
    r.globals.quirks.aircraft_released_above_pad = false;
    let (mut w, _, a) = built_at(&r, "MIDPAD", "NEAR");
    assert_eq!(air::height(&w, a), 0);
    assert!(st(&w, a).ground);
    assert_eq!(w.obj(a).unwrap().body.value(w.tick), EAST);
    testkit::run(&mut w, 10);
    let p = pos(&w, a);
    let to = abeam(&w, a, 2);
    move_to(&mut w, a, to);
    let moved = run_until(&mut w, 20, |w| pos(w, a) != p);
    assert!(moved, "{}", describe(&w, a));
    // spec: S-0501 claim 8 (take-off: the held height becomes the cruise height and it climbs by claim 2)
    assert!(air::height(&w, a) > 0, "{}", describe(&w, a));
    let turned = run_until(&mut w, 200, |w| {
        w.obj(a).unwrap().body.value(w.tick) != EAST
    });
    assert!(turned, "{}", describe(&w, a));
    // spec: S-0501 claim 8 (above trunc(H / 2) its body is requested to face the destination)
    let o = w.obj(a).unwrap();
    let m = centre(to);
    let b = glasnost_num::geom::direction(i64::from(m.x - o.pos.x), i64::from(m.y - o.pos.y), true);
    let off = glasnost_num::geom::delta(o.body.value(w.tick), b).abs();
    assert!(
        off < i32::from(EAST),
        "toward the destination: {}",
        describe(&w, a)
    );
}

// ---- Seams sweep 2 (docs/m8-claims/seams-2.md): the searches and the
// airborne scan against a released aircraft (claim 48 (a), AC30 part k (5)).

/// [`rules`] plus the anti-air-only weapon `Sam` (range 8 cells, the
/// anti-air projectile `Flak`), the defence `SAMS` (1 × 1, primary `Sam`)
/// and the vehicle `SAMV` (primary `Sam`), as in `claims_s0309_airborne`.
fn rules_scan() -> Ruleset {
    use glasnost_rules::sim::{ArmourClass, BuildingType, ProjectileType};
    let mut r = rules();
    r.sim.projectiles.insert(
        "Flak".into(),
        ProjectileType {
            aa: Some(true),
            ag: Some(false),
            inviso: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Sam".into(),
        WeaponType {
            damage: Some(5),
            rof: Some(30),
            range: econ_common::dec("8"),
            speed: Some(100),
            projectile: Some("Flak".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    let mut sams = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        primary: Some("Sam".into()),
        ..Default::default()
    };
    sams.combat.armor = Some(ArmourClass::Concrete);
    push(&mut r, "buildings", "SAMS");
    r.sim.buildings.insert("SAMS".into(), sams);
    let mut samv = econ_common::wheel(500);
    samv.primary = Some("Sam".into());
    push(&mut r, "vehicles", "SAMV");
    r.sim.vehicles.insert("SAMV".into(), samv);
    r
}

// spec: S-0501 claim 48 (a) (resting above its pad, a released aircraft is in neither cell layer and not a tracked flying object, so a range search passes over it: no cell offers it, S-0309 claim 10, and the airborne scan, S-0309 claim 9, reads only the tracked flying objects); the control: the same searcher acquires an aircraft of the same type in ordinary flight
#[test]
fn claim_48_a_a_range_search_passes_over_the_resting_release() {
    let r = rules_scan();
    let (mut w, k, a) = built_at(&r, "HIPAD", "SCOUT");
    // An enemy anti-air unit 3 cells away, well inside its 8-cell range.
    let v = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "SAMV",
        1,
        testkit::mid(SIZE, 0, 3),
        0,
    );
    for _ in 0..300 {
        step(&mut w);
        assert_ne!(
            w.obj(v).unwrap().target,
            Some(Target::Object(a)),
            "{}",
            describe(&w, a)
        );
        assert_resting(&w, k, a, 100);
    }
    // The control: an aircraft of the same type taking off beside it.
    let c = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "SCOUT",
        0,
        testkit::mid(SIZE, 2, 3),
        0,
    );
    move_to(&mut w, c, testkit::mid(SIZE, 2, -6));
    let seen = run_until(&mut w, 300, |w| {
        w.obj(v).unwrap().target == Some(Target::Object(c))
    });
    assert!(seen, "the control is acquired: {}", describe(&w, c));
    assert_ne!(w.obj(v).unwrap().target, Some(Target::Object(a)));
}

// spec: S-0501 AC30 part k (5) and claim 48 (c) (airborne scan: a fresh release leaving on a move order stays filed in the ground render layer, is tracked from its second movement update, and the enemy defence beside its path, 11 cells from the pad, acquires it at its first search after it is within range in flight: the scan reads the height's layer, not the filed list) and S-0309 claim 9
#[test]
fn ac30_k5_the_airborne_scan_acquires_the_leaving_release() {
    let r = rules_scan();
    let (mut w, _, a) = built_at(&r, "HIPAD", "JET");
    let d = building(&mut w, "SAMS", 1, 11, 2);
    testkit::run(&mut w, 10);
    assert_eq!(
        w.obj(d).unwrap().target,
        None,
        "the pad is outside its range"
    );
    move_to(&mut w, a, testkit::mid(SIZE, 16, 0));
    let mut tracked_since = None;
    let mut acquired = None;
    for _ in 0..600 {
        step(&mut w);
        if tracked_since.is_none() && tracked(&w, a) {
            tracked_since = Some(w.tick);
        }
        if w.obj(d).unwrap().target == Some(Target::Object(a)) {
            acquired = Some(w.tick);
            assert!(!air::on_ground(&w, a), "in flight: {}", describe(&w, a));
            assert!(tracked(&w, a), "{}", describe(&w, a));
            assert!(
                air::ground_filed(&w, a),
                "still filed in the ground render layer: {}",
                describe(&w, a)
            );
            break;
        }
    }
    assert!(tracked_since.is_some(), "{}", describe(&w, a));
    assert!(
        acquired.is_some(),
        "the defence acquires it: {}",
        describe(&w, a)
    );
}

/// A JET of house 0 flying on a person's order (`order`, with `target`)
/// and two TANKs of house 1 for the loss tests of claim 32.
fn loss_fixture(order: Behaviour, target: bool) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut w = game(&rules());
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "JET",
        0,
        testkit::mid(SIZE, -6, 0),
        0,
    );
    let x = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "TANK",
        1,
        testkit::mid(SIZE, 4, 0),
        0,
    );
    let y = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "TANK",
        1,
        testkit::mid(SIZE, 4, 4),
        0,
    );
    open(&mut w);
    assert!(orders::issue(
        &mut w,
        ALPHA,
        Order::Behaviour {
            actor: a,
            behaviour: order,
            attack_move: false,
            target: target.then_some(Target::Object(x)),
            dest: (!target).then_some(Target::Cell(testkit::mid(SIZE, 6, -6))),
        }
    ));
    for _ in 0..3 {
        step(&mut w);
    }
    assert_eq!(w.obj(a).unwrap().beh.current, Some(order));
    (w, a, x, y)
}

// spec: S-0501 claim 32, A target that leaves the map (an aircraft whose behaviour has been overridden is reverted at the moment its target boards a transport, right after the loss) and S-0215 claim 10 (the revert restores the suspended behaviour, target and destination)
#[test]
fn claim_32_the_loss_reverts_an_overridden_aircraft() {
    let (mut w, a, _, y) = loss_fixture(Behaviour::Move, false);
    let before = w.obj(a).unwrap().dest;
    assert!(before.is_some());
    glasnost_sim::combat::acquire::override_attack(&mut w, a, y);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
    assert_eq!(o.target, Some(Target::Object(y)));
    glasnost_sim::objects::take_inside(&mut w, y);
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.current, Some(Behaviour::Move), "reverted at the loss");
    assert_eq!(o.beh.suspended, None);
    assert_eq!(o.target, None, "the target put aside: none");
    assert_eq!(o.dest, before, "the destination put aside");
}

// spec: S-0501 claim 32, A target that leaves the map (the revert brings back the target the override put aside, even when it is the very object that left the map; a removal is a loss and a revert cause too). The target a revert at a removal leaves is not stated by the claim, so it is not asserted here.
// The override and the loss are invoked directly (`override_attack`, `take_inside`, `remove`): the fixture has no transport type.
#[test]
fn claim_32_the_revert_brings_back_the_object_that_left() {
    for removed in [false, true] {
        let (mut w, a, x, _) = loss_fixture(Behaviour::Attack, true);
        assert_eq!(w.obj(a).unwrap().target, Some(Target::Object(x)));
        glasnost_sim::combat::acquire::override_attack(&mut w, a, x);
        assert!(w.obj(a).unwrap().beh.suspended.is_some());
        if removed {
            glasnost_sim::objects::remove(&mut w, x);
        } else {
            glasnost_sim::objects::take_inside(&mut w, x);
        }
        let o = w.obj(a).unwrap();
        assert_eq!(o.beh.suspended, None, "reverted (removed: {removed})");
        assert_eq!(o.beh.current, Some(Behaviour::Attack));
        if !removed {
            assert_eq!(o.target, Some(Target::Object(x)), "the object that left");
        }
    }
}
