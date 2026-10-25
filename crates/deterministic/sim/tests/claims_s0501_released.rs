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
const PADS: [(&str, [i32; 3]); 6] = [
    ("HIPAD", [0, 0, 100]),
    ("LOWPAD", [0, 0, 15]),
    ("TENPAD", [0, 0, 10]),
    ("MIDPAD", [128, 128, 10]),
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
///   since it is not pad-bound).
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
    for (n, u) in [
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
/// fraction, tracked, listed, ground-filed).
struct Sample {
    h: i32,
    p: Pos,
    cur: glasnost_num::F64,
    tracked: bool,
    listed: bool,
    filed: bool,
}

fn sample(w: &World, a: ObjectId) -> Sample {
    Sample {
        h: air::height(w, a),
        p: pos(w, a),
        cur: st(w, a).cur,
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
    assert!(
        st(&w, a).tgt.lt(glasnost_sim::rules::nearest_double("1")),
        "target fraction below 1"
    );
    assert_eq!(second.h, 100, "the climb in the second update");
    assert!(second.tracked);
}

// spec: S-0501 claim 48 (c) (the low release: released at most 20 leptons up, the first update's descent reaches 0 and it is listed in its cell's ground layer at the end of that update, (b)'s end-of-update rule not applying; at the start of its second it is tracked at height 0 and, with a target fraction of 1, takes its cruise height as held height and climbs by F1, leaving the ground listing)
#[test]
fn claim_48_low_release() {
    let r = rules();
    let (mut w, k, a) = built_at(&r, "LOWPAD", "JET");
    assert_resting(&w, k, a, 15);
    testkit::run(&mut w, 10);
    let (_, first, second) = leave(&mut w, a, testkit::mid(SIZE, 10, 0));
    assert_eq!(first.h, 0, "{}", describe(&w, a));
    assert!(first.listed && !first.tracked, "listed, not yet tracked");
    assert!(second.tracked, "tracked in its second update");
    // spec: S-0501 F1 (a climb step of 20 without passengers)
    assert_eq!(second.h, 20, "{}", describe(&w, a));
    assert!(!second.listed, "it left the ground listing");
    assert!(second.filed, "still filed in the ground render layer");
    // spec: S-0501 claim 48 (c) (no landing has brought it to height 0: its second update climbs by (b) without a take-off, so nothing re-files it)
    assert!(!st(&w, a).taking_off, "{}", describe(&w, a));
}

// spec: S-0501 claim 48 (c) and (d) and S-0404 claim 13 (with a rally point, a type that need not return to a dock gets a move order to it at the release, which ends its contact with the pad in the same way as a person's order and starts no take-off; its first movement update descends from 100 to 80 by F2, and its second climbs)
#[test]
fn claim_48_rally_point_at_the_release() {
    let r = rules();
    let dest = testkit::mid(SIZE, 10, 0);
    let (mut w, k, a) = built_rally(&r, "HIPAD", "SCOUT", Some(dest));
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(dest)), "{}", describe(&w, a));
    assert_ne!(o.contact, Some(k), "the contact ended at the release");
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
    // A pad-bound aircraft that reaches its destination looks for a pad and returns (claims 6 and 18).
    move_to(&mut w, a, testkit::mid(SIZE, 3, 2));
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

// spec: S-0501 claim 48 (c) and (e) and AC30 part k (7) (a) (the low release at 10 leptons with a far move order: at the end of its first movement update at height 0, listed in its cell's ground layer and holding the ground flag there and in no other recorded cell; at the end of its second, which climbs with a target fraction of 1, a positive height and no flag and no listing in any recorded cell, as after claim 44's take-off)
#[test]
fn claim_48_e_low_release_climbs_and_clears() {
    let r = rules();
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

/// One sample per movement update.
type Update = (i32, glasnost_map::CellXY, bool);

/// A low release of DRIFT at `pad` given a person's move order to `dest`
/// after 10 ticks at rest. From its first movement update on, while `go`
/// holds (at most 400 updates), it samples each update with
/// [`low_update`] and checks the recorder: at height 0 the flag is in the
/// cell it is in and in no other recorded cell; above it, in none. Returns
/// the world, the aircraft, the recorded cells and the samples.
fn low_run(
    pad: &str,
    dest: impl Fn(&World, ObjectId) -> glasnost_map::CellXY,
    mut go: impl FnMut(&World, ObjectId) -> bool,
) -> (World, ObjectId, Vec<glasnost_map::CellXY>, Vec<Update>) {
    let r = rules();
    let (mut w, k, a) = built_at(&r, pad, "DRIFT");
    assert_resting(&w, k, a, 10);
    testkit::run(&mut w, 10);
    let p = pos(&w, a);
    let to = dest(&w, a);
    let mut cells = around(&w, k);
    move_to(&mut w, a, to);
    let moved = run_until(&mut w, 20, |w| pos(w, a) != p);
    assert!(moved, "{}", describe(&w, a));
    let mut samples = Vec::new();
    while go(&w, a) && samples.len() < 400 {
        let s = low_update(&w, a, &mut cells);
        assert_flags_only(&w, &cells, (s.0 == 0).then_some(s.1));
        samples.push(s);
        step(&mut w);
    }
    (w, a, cells, samples)
}

// spec: S-0501 claim 48 (c) and (e) and AC30 part k (7) (b) (the low release at 10 leptons with a near move order, a target fraction below 1: at height 0 at the end of every movement update until its landing starts, listed in the ground layer of the cell it is in at that tick and holding the ground flag there and in no other recorded cell, so that a cell it has left is clear; then it holds the flag at the end of its touchdown, in the cell that then holds it). The destination lies 640 leptons straight ahead along its body facing, since claim 42's guidance does not steer it at height 0 (Q-1905), and MIDPAD releases it above a cell's middle, so that it passes over the destination cell's centre and claim 6's trigger starts its landing near it; the run crosses at least one cell boundary at height 0
#[test]
fn claim_48_e_low_release_at_height_0_carries_its_flag() {
    let (mut w, a, mut cells, samples) = low_run(
        "MIDPAD",
        |w, a| ahead(w, a, 640),
        |w, a| !st(w, a).landing && !st(w, a).ground,
    );
    assert!(
        samples.len() < 400,
        "its landing starts: {}",
        describe(&w, a)
    );
    for (u, &(h, _, listed)) in samples.iter().enumerate() {
        assert_eq!(h, 0, "update {u}: {samples:?}");
        assert!(listed, "update {u}: in its cell's ground layer");
    }
    let mut seen: Vec<_> = samples.iter().map(|s| s.1).collect();
    seen.dedup();
    assert!(seen.len() > 1, "it crosses a cell at height 0: {seen:?}");
    // spec: S-0501 claim 48 (e) (it next holds the flag at the end of its touchdown, in the cell that then holds it)
    let down = run_until(&mut w, 400, |w| st(w, a).ground);
    assert!(down, "{}", describe(&w, a));
    let (h, c, _) = low_update(&w, a, &mut cells);
    assert_eq!(h, 0);
    assert_flags_only(&w, &cells, Some(c));
}

// spec: S-0501 claim 48 (b), (c) and (e) and claim 42 (interim of Q-1905: guidance only above the terrain, so the low release at height 0 with a destination off its heading goes straight on along its body facing; once claim 5 makes its target fraction 1 it climbs in that update by (b)'s third case, and at the end of that update it holds no flag and is in neither list). Until then the flag goes with it from cell to cell at height 0
#[test]
fn claim_48_e_low_release_off_its_heading_goes_straight_on() {
    let (w, a, cells, samples) = low_run(
        "TENPAD",
        |w, a| {
            let p = pos(w, a);
            let c = p.cell();
            // A destination 512 to 767 leptons away and not straight ahead.
            [(1, 0), (-1, 0), (0, 1), (0, -1)]
                .iter()
                .flat_map(|&(ux, uy)| (1..4).map(move |d| (ux * d, uy * d)))
                .map(|(dx, dy)| glasnost_map::CellXY::new(c.x + dx, c.y + dy))
                .find(|&q| {
                    let m = Pos::centre(q, 0);
                    let d = (m.x - p.x).abs().max((m.y - p.y).abs());
                    (512..768).contains(&d) && (1..4).all(|n| q != ahead(w, a, n * 256))
                })
                .expect("a destination off its heading")
        },
        |w, a| air::height(w, a) == 0,
    );
    assert!(samples.len() < 400, "it climbs: {}", describe(&w, a));
    let mut seen: Vec<_> = samples.iter().map(|s| s.1).collect();
    seen.dedup();
    assert!(seen.len() > 1, "it crosses a cell at height 0: {seen:?}");
    assert!(
        air::height(&w, a) > 0 && !st(&w, a).landing,
        "{}",
        describe(&w, a)
    );
    assert_flags_only(&w, &cells, None);
    assert_unlisted(&w, &cells, a);
    assert_eq!(st(&w, a).mark, None);
}
