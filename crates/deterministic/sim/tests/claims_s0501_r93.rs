//! Claim tests for S-0501 r93 (Q-2251, Q-2252): a scatter request
//! received by an aircraft (claim 49), the contact made when a pad is
//! accepted with *enter* (claims 18 and 24), the unarmed pad-bound aircraft
//! away from its pad (claim 50, `quirks.unarmed_pad_bound_hover`) and the
//! exact-cell landing trigger's reading of the ordinary target (claims 5
//! and 6), on synthetic data through the real tick path. Every type,
//! weapon and number is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, building, dec, factory, game, issue, ready, set_credits, t};
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::orders::Order;
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::scatter::{self, ScatterRequest};
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// A flying, landable aircraft type with weapon `weapon` and `ammo`
/// rounds, docking at APAD, of raw speed `speed`.
fn flyer(weapon: Option<&str>, ammo: i32, speed: i32) -> UnitType {
    let mut u = UnitType {
        strength: Some(200),
        speed: Some(speed),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1000),
        primary: weapon.map(Into::into),
        ..Default::default()
    };
    u.combat.ammo = Some(ammo);
    u.econ.landable = Some(true);
    u.econ.dock = Some(vec!["APAD".into()]);
    u
}

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with one docking place); JET (armed, weapon Agm, 2 rounds, not
/// pad-bound); SCOUT (unarmed, not pad-bound); BOUND (unarmed, pad-bound,
/// raw speed 30, so a converted speed of 76 and no stall); SLOW (as BOUND
/// with raw speed 2, a converted speed of 5, and slowdown distance 600);
/// NOLAND (as BOUND but not landable, raw speed 8, a converted speed of
/// 20, slowdown distance 5000). The behaviours' scatter properties are set
/// per test.
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    let mut pad = factory("aircraft");
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(1);
    push(&mut r, "buildings", "APAD");
    r.sim.buildings.insert("APAD".into(), pad);
    r.sim.weapons.insert(
        "Agm".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: dec("6"),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    let bound = |speed: i32| {
        let mut u = flyer(None, 0, speed);
        u.econ.airport_bound = Some(true);
        u
    };
    for (n, u) in [
        ("JET", flyer(Some("Agm"), 2, 30)),
        ("SCOUT", flyer(None, 0, 30)),
        ("BOUND", bound(30)),
        ("SLOW", {
            let mut u = bound(2);
            u.slowdown_distance = Some(600);
            u
        }),
        ("NOLAND", {
            let mut u = bound(8);
            u.econ.landable = Some(false);
            u.slowdown_distance = Some(5000);
            u
        }),
    ] {
        push(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    r
}

/// Sets the scatter property of behaviours `bs` (S-0215 claim 19).
fn scatter_prop(r: &mut Ruleset, bs: &[Behaviour], on: bool) {
    for b in bs {
        r.sim.behaviours.entry(b.key().into()).or_default().scatter = Some(on);
    }
}

fn put(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Aircraft, name, 0, testkit::mid(SIZE, dx, dy), 0)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
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

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn describe(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    format!(
        "t={} pos={:?} h={} beh={:?}/{:?} dest={:?} target={:?} ammo={} contact={:?} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.beh.current,
        o.beh.queued,
        o.dest,
        o.target,
        o.combat.ammo,
        o.contact,
        o.air
    )
}

fn has(w: &World, id: ObjectId, b: Behaviour) -> bool {
    let o = w.obj(id).unwrap();
    o.beh.current == Some(b) || o.beh.queued == Some(b)
}

fn move_to(w: &mut World, a: ObjectId, c: CellXY) {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(issue(
        w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(c)),
        }
    ));
}

/// Builds `name` at a fresh APAD of house 0; returns (world, pad, aircraft).
fn built(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    set_credits(&mut w, 0, 100_000);
    let a = build_one(&mut w, name);
    (w, k, a)
}

/// Builds `name` at house 0's factory; returns the new aircraft.
fn build_one(w: &mut World, name: &str) -> ObjectId {
    let what = t(w, Kind::Aircraft, name);
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(issue(w, 0, Order::Build { what }));
    let ti = what.index;
    let new = |w: &World| {
        w.objects
            .values()
            .find(|o| o.kind == Kind::Aircraft && o.type_index == ti && o.on_map)
            .map(|o| o.id)
    };
    assert!(run_until(w, 3000, |w| new(w).is_some()), "built");
    new(w).unwrap()
}

/// The squared planar distance between `p` and cell `c`'s centre.
fn d2(p: Pos, c: CellXY) -> i64 {
    let q = Pos::centre(c, 0);
    let (dx, dy) = (i64::from(p.x - q.x), i64::from(p.y - q.y));
    dx * dx + dy * dy
}

/// A JET of house 0 standing on open ground at offset (dx, dy), with its
/// ammunition spent and no contact.
fn spent_jet(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    let a = put(w, "JET", dx, dy);
    // A tick after the placement fall-back.
    step(w);
    let o = w.obj_mut(a).unwrap();
    o.combat.ammo = 0;
    assert!(air::on_ground(w, a) && w.obj(a).unwrap().contact.is_none());
    a
}

// ---------------------------------------------------------------------------
// Claims 18 and 24: the contact at the pad's acceptance

// spec: S-0501 claim 24 (r93, Q-2251: an ordered enter on a pad of its dock types, with no contact held, makes the contact when the pad is accepted as destination, in the tick the order is applied) and claim 23 (a pad with a free place takes it into contact)
#[test]
fn claim_24_an_ordered_enter_makes_the_contact_at_the_acceptance() {
    let mut w = game(&rules());
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, "SCOUT", 8, 0);
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(Target::Object(k)),
        }
    ));
    assert!(run_until(&mut w, 10, |w| w.obj(a).unwrap().dest
        == Some(Target::Object(k))));
    assert_eq!(
        w.obj(a).unwrap().contact,
        Some(k),
        "in the tick the order applies: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 24 (r93: the contact needs enter current or queued at the acceptance; a move onto the pad's cell makes none)
#[test]
fn claim_24_a_move_onto_the_pad_makes_no_contact() {
    let mut w = game(&rules());
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, "SCOUT", 8, 0);
    let c = w.obj(k).unwrap().pos.cell();
    move_to(&mut w, a, c);
    for _ in 0..40 {
        step(&mut w);
        assert_eq!(w.obj(a).unwrap().contact, None, "{}", describe(&w, a));
    }
}

// spec: S-0501 claim 18 (r93, Q-2251: the search after the row asks the pad for no contact, and the pad is accepted before enter is queued, so the first fall-back makes none) and claim 49 (a second fall-back in that tick finds enter current or queued at the acceptance, so the pad takes it into contact, claim 24; T2's after-the-row entries give the pad as destination and enter)
#[test]
fn claims_18_and_24_the_second_fall_back_makes_the_contact() {
    let mut w = game(&rules());
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = spent_jet(&mut w, 8, 0);
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
    assert!(has(&w, a, Behaviour::Enter), "{}", describe(&w, a));
    assert_eq!(o.contact, None, "the first fall-back asks for no contact");
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
    assert!(has(&w, a, Behaviour::Enter), "{}", describe(&w, a));
    assert_eq!(o.contact, Some(k), "{}", describe(&w, a));
}

// ---------------------------------------------------------------------------
// Claim 49: a scatter request received

/// The behaviours a request's gate can read in these tests.
const GATED: [Behaviour; 6] = [
    Behaviour::Sleep,
    Behaviour::Guard,
    Behaviour::Enter,
    Behaviour::Capture,
    Behaviour::Sabotage,
    Behaviour::Eaten,
];

fn req(threat: Option<Pos>, forced: bool) -> ScatterRequest {
    ScatterRequest {
        threat,
        forced,
        anywhere: false,
    }
}

// spec: S-0501 claim 49 (r93, Q-2251: an aircraft answers a cell's request by a fall-back to rest when its current behaviour has the scatter property, otherwise not at all; the threat point and the forced option play no part; it is never given a nearby cell and never takes off) and AC31 part e runs 2S and 2C
#[test]
fn claim_49_a_cells_request_is_gated_by_the_current_behaviours_scatter_property() {
    for set in [true, false] {
        for (threat, forced) in [(None, false), (Some(()), true)] {
            let mut r = rules();
            r.sim.general.player_scatter = Some(true);
            scatter_prop(&mut r, &GATED, set);
            let mut w = game(&r);
            ready(&mut w);
            let a = put(&mut w, "JET", 8, 0);
            // A tick after the placement fall-back, so a new fall-back shows in the common-effects tick.
            step(&mut w);
            let t0 = w.tick;
            w.obj_mut(a).unwrap().beh.force(Behaviour::Sleep, t0, false);
            assert_ne!(w.obj(a).unwrap().beh.last_common_effects, Some(t0));
            let c = w.obj(a).unwrap().pos.cell();
            assert!(w.cells.list(c, Layer::Ground).contains(&a), "listed");
            let before = w.obj(a).unwrap().clone();
            let threat = threat.map(|()| Pos::centre(testkit::mid(SIZE, 9, 0), 0));
            scatter::cell_request(&mut w, c, Layer::Ground, req(threat, forced));
            let o = w.obj(a).unwrap();
            assert_eq!(o.dest, None, "no nearby cell: {}", describe(&w, a));
            assert_eq!(o.pos, before.pos);
            assert!(!st(&w, a).taking_off && air::on_ground(&w, a));
            if set {
                assert_eq!(o.beh.last_common_effects, Some(w.tick), "a fall-back");
                assert!(
                    has(&w, a, Behaviour::Guard),
                    "T2's ground row: {}",
                    describe(&w, a)
                );
            } else {
                assert_eq!(o.beh, before.beh, "no effect: {}", describe(&w, a));
            }
        }
    }
}

// spec: S-0501 claim 49 (r93: an aircraft with no current behaviour always falls back to rest, whatever is queued: the gate never reads the queued behaviour, an exception to S-0215 claim 1)
#[test]
fn claim_49_no_current_behaviour_always_falls_back() {
    let mut r = rules();
    scatter_prop(&mut r, &GATED, false);
    let mut w = game(&r);
    ready(&mut w);
    let a = put(&mut w, "JET", 8, 0);
    // A tick after the placement fall-back.
    step(&mut w);
    {
        let o = w.obj_mut(a).unwrap();
        o.beh.current = None;
        o.beh.queued = Some(Behaviour::Sleep);
    }
    assert_ne!(w.obj(a).unwrap().beh.last_common_effects, Some(w.tick));
    assert!(!scatter::request(&mut w, a, req(None, true)));
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.last_common_effects,
        Some(w.tick),
        "a fall-back: {}",
        describe(&w, a)
    );
    // spec: S-0501 T2 (on the ground otherwise: guard, in place of the queued sleep)
    assert!(has(&w, a, Behaviour::Guard), "{}", describe(&w, a));
    assert_ne!(o.beh.queued, Some(Behaviour::Sleep), "{}", describe(&w, a));
}

/// Places `name` of house 0 on the ground on a HUT's cell (A1) and on open
/// ground (A2), each forced into `b` with no destination and no target.
fn capture_pair(r: &Ruleset, name: &str, b: Behaviour) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let hut = building(&mut w, "HUT", 1, 6, 6);
    ready(&mut w);
    let hc = w.obj(hut).unwrap().pos.cell();
    let a1 = testkit::spawn(&mut w, Kind::Aircraft, name, 0, hc, 0);
    let a2 = put(&mut w, name, -8, -8);
    let c1 = w.obj(a1).unwrap().pos.cell();
    assert!(
        w.cells
            .list(c1, Layer::Ground)
            .iter()
            .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building)),
        "A1 stands on the building's cell"
    );
    let t0 = w.tick;
    for a in [a1, a2] {
        let o = w.obj_mut(a).unwrap();
        o.dest = None;
        o.target = None;
        o.beh.force(b, t0, false);
    }
    (w, a1, a2)
}

// spec: S-0501 claim 49 (r93, Q-2251: an aircraft in capture, sabotage or eaten with no destination falls back to rest and, on a cell holding a building of any owner, is sent the request; landed and unarmed, both fall-backs leave it with no destination and guard) and AC31 part a runs 1, 3 and 4 (A1 on a building's cell and A2 on open ground end alike); S-0215 claim 28 row 2
#[test]
fn claim_49_capture_sabotage_and_eaten_on_a_building_end_as_on_open_ground() {
    for set in [true, false] {
        for b in [Behaviour::Capture, Behaviour::Sabotage, Behaviour::Eaten] {
            let mut r = rules();
            scatter_prop(&mut r, &GATED, set);
            let (mut w, a1, a2) = capture_pair(&r, "SCOUT", b);
            let cells = [a1, a2].map(|a| w.obj(a).unwrap().pos.cell());
            let processed = run_until(&mut w, 200, |w| [a1, a2].iter().all(|&a| !has(w, a, b)));
            assert!(processed, "{}", describe(&w, a1));
            for _ in 0..300 {
                for (i, a) in [a1, a2].into_iter().enumerate() {
                    let o = w.obj(a).unwrap();
                    assert_eq!(o.dest, None, "{b:?} {set}: {}", describe(&w, a));
                    assert_eq!(air::height(&w, a), 0);
                    assert_eq!(o.pos.cell(), cells[i]);
                    assert!(!st(&w, a).taking_off);
                    assert!(has(&w, a, Behaviour::Guard), "{}", describe(&w, a));
                }
                let (o1, o2) = (w.obj(a1).unwrap(), w.obj(a2).unwrap());
                assert_eq!(o1.beh.current, o2.beh.current);
                assert_eq!(o1.beh.queued, o2.beh.queued);
                step(&mut w);
            }
        }
    }
}

// spec: S-0501 claim 49 (r93, Q-2251: landed, armed, ammunition spent and no contact, in capture on a building's cell: the first fall-back gives the pad as destination and enter with no contact; the request then makes a second fall-back when the deciding behaviour has the scatter property, whose acceptance with enter current or queued takes the aircraft into contact, claim 24; without the property the request has no effect and no contact is made)
#[test]
fn claim_49_the_request_on_a_building_makes_the_contact_only_through_the_scatter_property() {
    for set in [true, false] {
        let mut r = rules();
        scatter_prop(&mut r, &GATED, set);
        let mut w = game(&r);
        let k = building(&mut w, "APAD", 0, -8, 0);
        let hut = building(&mut w, "HUT", 1, 6, 6);
        ready(&mut w);
        let hc = w.obj(hut).unwrap().pos.cell();
        let a = testkit::spawn(&mut w, Kind::Aircraft, "JET", 0, hc, 0);
        let c = w.obj(a).unwrap().pos.cell();
        assert!(
            w.cells
                .list(c, Layer::Ground)
                .iter()
                .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building)),
            "it stands on the building's cell"
        );
        {
            let t0 = w.tick;
            let o = w.obj_mut(a).unwrap();
            o.combat.ammo = 0;
            o.dest = None;
            o.target = None;
            o.beh.force(Behaviour::Capture, t0, false);
        }
        assert!(w.obj(a).unwrap().contact.is_none());
        let took = run_until(&mut w, 200, |w| {
            w.obj(a).unwrap().dest == Some(Target::Object(k))
        });
        assert!(took, "{}", describe(&w, a));
        assert!(has(&w, a, Behaviour::Enter), "{}", describe(&w, a));
        let want = set.then_some(k);
        assert_eq!(
            w.obj(a).unwrap().contact,
            want,
            "{set}: {}",
            describe(&w, a)
        );
    }
}

// ---------------------------------------------------------------------------
// Claim 50 and quirks.unarmed_pad_bound_hover

/// Flies a BOUND built at the pad to a far cell P; with `occupy`, once it
/// has left the pad a second BOUND is built into the pad's only place.
/// Then follows it for `n` ticks after its first fall-back away from P and
/// returns (world, pad, aircraft, docked again).
fn far_flight(hover: bool, occupy: bool, n: i32) -> (World, ObjectId, ObjectId, bool) {
    let mut r = rules();
    r.globals.quirks.unarmed_pad_bound_hover = hover;
    let (mut w, k, a) = built(&r, "BOUND");
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "docked at the pad");
    let p = testkit::mid(SIZE, -14, 12);
    move_to(&mut w, a, p);
    assert!(run_until(&mut w, 300, |w| w
        .obj(a)
        .unwrap()
        .contact
        .is_none()));
    if occupy {
        // spec: S-0501 claim 23 (a second aircraft of the type takes the pad's only contact place)
        let b = put(&mut w, "BOUND", 3, 3);
        assert!(air::request_contact(&mut w, b, k));
        assert_eq!(
            w.obj(b).unwrap().contact,
            Some(k),
            "the second takes the place"
        );
    }
    // spec: S-0501 claim 6 (at P, a cell holding no building it is in contact with, the landing is refused and it falls back to rest)
    let away = run_until(&mut w, 4000, |w| {
        w.obj(a).unwrap().dest != Some(Target::Cell(p))
    });
    assert!(away, "it falls back at P: {}", describe(&w, a));
    let pad_cell = w.obj(k).unwrap().pos.cell();
    let mut docked = false;
    for _ in 0..n {
        let o = w.obj(a).expect("never crashes");
        assert!(!air::crashing(&w, a), "{}", describe(&w, a));
        if o.contact == Some(k) && st(&w, a).ground {
            docked = true;
            break;
        }
        if hover || occupy {
            assert_eq!(o.contact, None, "no contact: {}", describe(&w, a));
            assert!(!has(&w, a, Behaviour::Enter), "{}", describe(&w, a));
            assert!(!st(&w, a).landing, "no landing: {}", describe(&w, a));
            assert!(has(&w, a, Behaviour::Move), "{}", describe(&w, a));
            // spec: S-0501 F8 (a free spot near the pad, the quartered distance being the smallest)
            if let Some(Target::Cell(c)) = o.dest {
                let near = (c.x - pad_cell.x).abs() <= 6 && (c.y - pad_cell.y).abs() <= 6;
                assert!(near, "near the pad: {}", describe(&w, a));
            }
        }
        step(&mut w);
    }
    (w, k, a, docked)
}

// spec: S-0501 claim 50 (r93, Q-2252: an unarmed pad-bound aircraft with dock types makes no pad search; at a far cell its landing is refused and it takes an F8 spot near its pad with move, and stays in the air there with no contact, no landing and no crash, whether its pad's place is free or taken) and AC31 part b runs 1 and 3; Edge cases (quirks.unarmed_pad_bound_hover on)
#[test]
fn claim_50_an_unarmed_pad_bound_aircraft_hovers_near_its_pad() {
    for occupy in [false, true] {
        let (w, _, a, docked) = far_flight(true, occupy, 2000);
        assert!(!docked, "{}", describe(&w, a));
        assert!(air::height(&w, a) > 0, "in the air: {}", describe(&w, a));
    }
}

// spec: S-0501 Edge cases (quirks.unarmed_pad_bound_hover off: only T2's last in-flight row changes for a landable pad-bound type with dock types: it looks for a pad and asks it for contact; granted, the pad as destination and enter, and it docks, claim 24; otherwise the row's F8 spot and move, with no crash)
#[test]
fn quirk_unarmed_pad_bound_hover_off_docks_at_a_free_pad_and_hovers_at_a_taken_one() {
    let (w, k, a, docked) = far_flight(false, false, 3000);
    assert!(docked, "free pad: it docks: {}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().contact, Some(k));
    let (w, _, a, docked) = far_flight(false, true, 2000);
    assert!(!docked, "taken pad: {}", describe(&w, a));
    assert!(air::height(&w, a) > 0, "in the air: {}", describe(&w, a));
}

// spec: S-0501 Edge cases (quirks.unarmed_pad_bound_hover: the alternative applies only to a landable type; an armed aircraft is not this row's) and claim 50 (a type that is not landable takes the F8 spot and move in both settings)
#[test]
fn quirk_unarmed_pad_bound_hover_changes_nothing_for_a_type_that_is_not_landable() {
    for hover in [true, false] {
        let mut r = rules();
        r.globals.quirks.unarmed_pad_bound_hover = hover;
        let mut w = game(&r);
        building(&mut w, "APAD", 0, 0, 0);
        ready(&mut w);
        let a = put(&mut w, "NOLAND", 10, 0);
        move_to(&mut w, a, testkit::mid(SIZE, -10, 0));
        assert!(run_until(&mut w, 300, |w| air::height(w, a) >= 500));
        process::fall_back(&mut w, a, false);
        let o = w.obj(a).unwrap();
        assert!(has(&w, a, Behaviour::Move), "{hover}: {}", describe(&w, a));
        assert!(
            matches!(o.dest, Some(Target::Cell(_))),
            "{}",
            describe(&w, a)
        );
        assert_eq!(o.contact, None);
    }
}

// ---------------------------------------------------------------------------
// Claims 5 and 6: the exact-cell trigger reads the ordinary target

// spec: S-0501 claim 6 (r93, Q-2252: the exact-cell trigger reads the target the same tick's ordinary calculation gave, not claim 5's approach target replacement: a slow-speed stall within 128 leptons of the flight point, in its cell, starts no landing and makes no fall-back, although the target logged at the end of each tick there is 0) and claim 5 (the stall with V < 10, its current fraction at least 0.1); claim 50 (Slow-speed stalls) and AC31 part c
#[test]
fn claim_6_a_stall_in_the_flight_points_cell_starts_no_landing() {
    let mut w = game(&rules());
    building(&mut w, "APAD", 0, 8, 8);
    ready(&mut w);
    let a = put(&mut w, "SLOW", 4, 0);
    let p = testkit::mid(SIZE, -4, 0);
    move_to(&mut w, a, p);
    let tenth = glasnost_sim::rules::nearest_double("0.1");
    let mut stood = 0;
    for _ in 0..3000 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        let s = st(&w, a);
        assert!(!s.landing, "no landing starts: {}", describe(&w, a));
        assert_eq!(o.dest, Some(Target::Cell(p)), "{}", describe(&w, a));
        assert!(has(&w, a, Behaviour::Move), "{}", describe(&w, a));
        // spec: S-0501 claim 5 (within 128 leptons of the destination point: the approach replacement)
        let close = d2(o.pos, p) < 128 * 128;
        if close && o.pos.cell() == p && s.tgt.eq_num(F64::ZERO) && s.cur.gt(F64::ZERO) {
            assert!(!s.cur.lt(tenth), "the stall keeps at least 0.1");
            stood += 1;
        }
    }
    assert!(
        stood > 100,
        "it stands stalled in P's cell with a logged target of 0: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 50 (r93, Q-2252: a type that is not landable and that claim 5 does not hold at full speed is marked as holding height, so no landing is started or refused and no fall-back follows; within 768 leptons of its cell's flight point it descends to height 0; along map east on the point's row its y never changes, and with a converted speed of at least 20 it stops below 86 leptons short of the point, in its cell, and stays there with move and no contact) and AC31 part d
#[test]
fn claim_50_a_type_that_is_not_landable_stops_short_at_height_0() {
    let mut w = game(&rules());
    building(&mut w, "APAD", 0, 8, 8);
    ready(&mut w);
    // spec: S-0204 claim 8 (map east: facing 16384)
    let a = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "NOLAND",
        0,
        testkit::mid(SIZE, -12, 0),
        16384,
    );
    let p = testkit::mid(SIZE, 12, 0);
    let y = w.obj(a).unwrap().pos.y;
    assert_eq!(y, Pos::centre(p, 0).y, "on P's row");
    move_to(&mut w, a, p);
    let mut stop: Option<Pos> = None;
    // The take-off climb is exempt (AC31 part d): the checks of height 0 start once it has reached cruise height.
    let mut climbed = false;
    for _ in 0..3000 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        let s = st(&w, a);
        assert_eq!(o.pos.y, y, "the row is kept: {}", describe(&w, a));
        assert!(!s.landing, "no landing: {}", describe(&w, a));
        assert_eq!(o.dest, Some(Target::Cell(p)), "{}", describe(&w, a));
        assert!(has(&w, a, Behaviour::Move), "{}", describe(&w, a));
        assert_eq!(o.contact, None);
        assert!(o.pos.x <= Pos::centre(p, 0).x, "it does not pass P");
        climbed |= air::height(&w, a) >= w.rules().air.flight_level;
        if !climbed {
            continue;
        }
        assert!(!s.ground, "no touchdown: {}", describe(&w, a));
        if let Some(q) = stop {
            assert_eq!(o.pos, q, "it stays: {}", describe(&w, a));
            assert_eq!(air::height(&w, a), 0);
        } else if air::height(&w, a) == 0 && s.cur.eq_num(F64::ZERO) {
            stop = Some(o.pos);
        }
    }
    assert!(climbed, "it reaches cruise height");
    let q = stop.unwrap_or_else(|| panic!("it stops: {}", describe(&w, a)));
    assert_eq!(q.cell(), p, "in P's cell");
    // spec: S-0501 claim 50 and AC31 part d (it stops below 86 leptons short of P)
    assert!(Pos::centre(p, 0).x - q.x < 86, "{q:?}");
}
