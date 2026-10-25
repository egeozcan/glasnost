//! An aircraft released above its pad (S-0501 claim 48, revision 52,
//! answering Q-1841), docking by landing that moves only horizontally
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
const PADS: [(&str, [i32; 3]); 4] = [
    ("HIPAD", [0, 0, 100]),
    ("LOWPAD", [0, 0, 15]),
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
/// - the pads of [`PADS`], each an aircraft factory, helipad and reload pad
///   with one docking place and the given displacement;
/// - JET (armed with Agm, 2 rounds, pad-bound), as AC30 part k's type;
/// - NEAR (JET with a slowdown distance of 2000 leptons, AC30 part k (4));
/// - SCOUT (unarmed, not pad-bound), the control that lands on open ground;
/// - SLOW (JET with a speed of 5 leptons per tick, so that a forward step
///   at a speed fraction below 0.2 is 0 leptons and only the docking
///   approach moves it horizontally while it lands, claim 4).
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
    for (n, u) in [
        ("JET", jet),
        ("NEAR", near),
        ("SCOUT", flyer(None)),
        ("SLOW", slow),
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
    let mut w = game(r);
    let k = building(&mut w, pad, 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let what = t(&w, Kind::Aircraft, name);
    assert!(issue(&mut w, 0, Order::Build { what }));
    let ti = what.index;
    let found = run_until(&mut w, 3000, |w| {
        w.objects
            .values()
            .any(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map)
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
    testkit::run(&mut w, 20);
    assert!(blast_under(&mut w, a), "area damage reaches it");
    move_to(&mut w, a, testkit::mid(SIZE, 10, 0));
    let off = run_until(&mut w, 50, |w| st(w, a).taking_off);
    assert!(off, "an ordinary take-off: {}", describe(&w, a));
    assert!(tracked(&w, a), "tracked from the take-off (claim 47 (a))");
    assert!(!air::ground_filed(&w, a));
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
