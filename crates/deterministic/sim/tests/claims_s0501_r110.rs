//! Claim tests for S-0501 r110 (Q-2436, Q-2437, Q-2438): the pad's load
//! request and claim 24's second rule, the docking stage that tests no
//! position (AC31 part h), the run-away notice of a pad that starts selling,
//! claim 8's same-cell exception (AC4's same-cell part), claim 29 at cruise
//! height, the kept ready mark of claim 31 and the two quirk switches
//! `aircraft_grounded_enter_stall` and `aircraft_ready_mark_outlives_enter`.
//! Synthetic data through the real tick path; every type, weapon and number
//! is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready, switch};
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::econ::release::Exit;
use glasnost_sim::orders::Order;
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

fn pad_type(docks: i32, offsets: Option<Vec<[i32; 3]>>) -> glasnost_rules::sim::BuildingType {
    let mut pad = factory("aircraft");
    // A build-up sprite, so that a sell order sells it (S-0408 claim 4).
    pad.econ.buildup = Some(true);
    pad.econ.buildup_frames = Some(10);
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(docks);
    pad.econ.docking_offsets = offsets;
    pad
}

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with `docks` docking places and the displacements `offsets`, a 2 × 2
/// foundation); JET (armed, landable, not pad-bound, 2 rounds of the weapon
/// Agm, docking at APAD); BOUND (as JET, pad-bound); TANK (a ground vehicle
/// of the test house, for an occupant F7 refuses).
fn rules_with(docks: i32, offsets: Option<Vec<[i32; 3]>>) -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    push(&mut r, "buildings", "APAD");
    r.sim
        .buildings
        .insert("APAD".into(), pad_type(docks, offsets));
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
    let jet = || {
        let mut u = UnitType {
            strength: Some(200),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            tech_level: Some(1),
            owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
            cost: Some(1000),
            primary: Some("Agm".into()),
            ..Default::default()
        };
        u.combat.ammo = Some(2);
        u.econ.landable = Some(true);
        u.econ.dock = Some(vec!["APAD".into()]);
        u
    };
    push(&mut r, "aircraft", "JET");
    r.sim.aircraft.insert("JET".into(), jet());
    let mut bound = jet();
    bound.econ.airport_bound = Some(true);
    push(&mut r, "aircraft", "BOUND");
    r.sim.aircraft.insert("BOUND".into(), bound);
    r
}

/// AC31 part g's pad: one docking place whose displacement is zero, so its
/// docking position is its foundation centre (S-0404 claim 15).
fn rules() -> Ruleset {
    rules_with(1, Some(vec![[0, 0, 0]]))
}

/// The displacement of AC31 part h (1)'s pad (test value): from the 2 × 2
/// foundation's centre, a cell corner, to the centre of the cell two cells
/// east of the foundation's east column, in its south row, at height 0. One
/// cell of open ground lies between that cell and the foundation.
const OFF_PAD: [i32; 3] = [640, 128, 0];

/// AC31 part h (1)'s pad (S-0404 claim 15: one docking place, its sole
/// displacement).
fn off_rules() -> Ruleset {
    rules_with(1, Some(vec![OFF_PAD]))
}

fn put(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Aircraft, name, 0, testkit::mid(SIZE, dx, dy), 0)
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn describe(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    format!(
        "t={} pos={:?} h={} beh={:?}/{:?} last={:?} dest={:?} contact={:?} ammo={} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.beh.current,
        o.beh.queued,
        o.beh.last,
        o.dest,
        o.contact,
        o.combat.ammo,
        o.air
    )
}

fn foundation(w: &World, k: ObjectId) -> Vec<CellXY> {
    let b = w.obj(k).unwrap();
    glasnost_sim::econ::placement::foundation_cells(w, b.type_index, b.pos.cell())
}

fn in_foundation(w: &World, k: ObjectId, c: CellXY) -> bool {
    foundation(w, k).contains(&c)
}

/// Whether cell `c` is cell `d` or one of its eight neighbours.
fn near(c: CellXY, d: CellXY) -> bool {
    (c.x - d.x).abs() <= 1 && (c.y - d.y).abs() <= 1
}

fn service(w: &World, k: ObjectId) -> Option<i32> {
    w.obj(k).unwrap().air.as_ref().and_then(|s| s.service)
}

/// AC31 part f's start: a pad *P* of house 0 and a JET of house 0 on open
/// ground 8 cells from it, its ammunition spent and no contact, given by
/// claim 18's search after T2's ground row *P* as destination and *enter*
/// with no contact. Returns (world, P, A).
fn without_contact(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, name, 8, 0);
    step(&mut w);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    assert!(air::on_ground(&w, a) && w.obj(a).unwrap().contact.is_none());
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
    assert_eq!(o.beh.current, Some(Behaviour::Enter), "{}", describe(&w, a));
    (w, k, a)
}

/// Steps until the next processing of `a`; returns its tick and delay.
fn next_processing(w: &mut World, a: ObjectId) -> (i32, i32) {
    let before = w.obj(a).unwrap().beh.last;
    for _ in 0..50 {
        step(w);
        let last = w.obj(a).unwrap().beh.last;
        if last != before {
            return last.unwrap();
        }
    }
    panic!("no processing: {}", describe(w, a));
}

/// Steps until a processing of `a` is due in the coming tick, then calls
/// `f` and runs that tick; returns its tick and delay.
fn at_next_processing(w: &mut World, a: ObjectId, f: impl FnOnce(&mut World)) -> (i32, i32) {
    for _ in 0..50 {
        if w.obj(a).unwrap().beh.due(w.tick) {
            f(w);
            let t = w.tick;
            step(w);
            let last = w.obj(a).unwrap().beh.last.unwrap();
            assert_eq!(last.0, t, "processed: {}", describe(w, a));
            return last;
        }
        step(w);
    }
    panic!("no processing due: {}", describe(w, a));
}

/// Runs `without_contact` to the approaching stage in contact (the
/// second processing's fall-back made the contact).
fn approaching(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let (mut w, k, a) = without_contact(r, "JET");
    next_processing(&mut w, a);
    next_processing(&mut w, a);
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
    (w, k, a)
}

/// Runs an approaching aircraft in contact until the processing of *enter*
/// that moves it on to the docking stage (3 ticks); returns that
/// processing's tick.
fn to_docking_stage(w: &mut World, a: ObjectId) -> i32 {
    for _ in 0..1500 {
        let before = w.obj(a).unwrap().beh.last;
        step(w);
        let o = w.obj(a).unwrap();
        if o.beh.last != before
            && o.beh.current == Some(Behaviour::Enter)
            && st(w, a).enter_docking
            && o.beh.last.is_some_and(|(_, d)| d == 3)
        {
            return o.beh.last.unwrap().0;
        }
    }
    panic!("no docking stage: {}", describe(w, a));
}

/// Gives a sell order on building `k` of house 0 and runs until the tick at
/// whose end its sale has begun; returns that tick.
fn sell_order(w: &mut World, k: ObjectId) -> i32 {
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(issue(w, 0, Order::Sell { actor: k }), "the sell order");
    for _ in 0..20 {
        step(w);
        if w.obj(k)
            .is_none_or(|b| b.beh.current == Some(Behaviour::Selling))
        {
            return w.tick - 1;
        }
    }
    panic!("no sale");
}

/// A JET of house 0 on the ground on open ground 8 cells from the pad *P*,
/// in contact with it, no destination, *enter* current (just started, so
/// its next processing is its first) and nothing queued: the approaching
/// stage's grounded aircraft (claim 24). `name` is the type. Returns
/// (world, P, A).
fn grounded_in_contact(r: &Ruleset, name: &str) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, name, 8, 0);
    step(&mut w);
    assert!(air::on_ground(&w, a), "{}", describe(&w, a));
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.combat.ammo = 0;
        o.contact = Some(k);
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: false,
            place: 0,
        });
        o.dest = None;
        o.beh.force(Behaviour::Enter, t, true);
        o.beh.queued = None;
        let s = o.air.as_mut().unwrap();
        s.flying_to = None;
    }
    (w, k, a)
}

// ---------------------------------------------------------------------------
// AC31 part h (1): docking off the foundation

// spec: S-0501 claim 24 (r110, Q-2436: the docking stage tests no position: the landing processings move the aircraft toward its docking position, and after touchdown the next processing reports entry wherever it touched down; the report leaves sleep queued and the aircraft queues guard, which replaces it, so guard is current at the end of that processing's tick with nothing or sleep queued) and claim 26 (the pad starts servicing in the report's tick or the next; the first service step gives no round) and F7 (the open cell is accepted); AC31 part h (1)
#[test]
fn ac31h1_docking_off_the_foundation_reports_entry_and_is_serviced() {
    let (mut w, k, a) = approaching(&off_rules());
    let dp = glasnost_sim::econ::release::pad_position(&w, k, a).0;
    let dc = dp.cell();
    assert!(
        !in_foundation(&w, k, dc),
        "the docking position is off the pad"
    );
    assert!(
        foundation(&w, k).iter().all(|f| !near(*f, dc)),
        "one open cell between it and the foundation"
    );
    // Until the touchdown (u).
    let mut u = None;
    for _ in 0..2000 {
        step(&mut w);
        let s = st(&w, a);
        if air::height(&w, a) == 0 && !s.landing && s.ground {
            u = Some(w.tick - 1);
            break;
        }
    }
    let u = u.unwrap_or_else(|| panic!("no touchdown: {}", describe(&w, a)));
    let o = w.obj(a).unwrap();
    assert!(
        near(o.pos.cell(), dc) && !in_foundation(&w, k, o.pos.cell()),
        "touched down by the docking position, off the pad: {}",
        describe(&w, a)
    );
    assert_eq!(o.contact, Some(k), "{}", describe(&w, a));
    assert_eq!(o.dest, None, "{}", describe(&w, a));
    assert_eq!(o.beh.current, Some(Behaviour::Enter));
    let at = o.pos;
    // r: the next processing after u reports entry.
    let (r, _) = next_processing(&mut w, a);
    assert!(r > u);
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Guard),
        "the entered report: {}",
        describe(&w, a)
    );
    assert!(
        matches!(o.beh.queued, None | Some(Behaviour::Sleep)),
        "nothing or sleep queued: {}",
        describe(&w, a)
    );
    // q: the pad's servicing starts in r or r + 1.
    if service(&w, k).is_none() {
        step(&mut w);
    }
    let q = w.tick - 1;
    assert!(
        service(&w, k).is_some() && (q == r || q == r + 1),
        "servicing from r or r + 1: {}",
        describe(&w, a)
    );
    let mut slept = false;
    let mut rounds = None;
    for _ in 0..1500 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert_ne!(
            o.beh.current,
            Some(Behaviour::Enter),
            "no enter again: {}",
            describe(&w, a)
        );
        assert_ne!(o.beh.queued, Some(Behaviour::Enter));
        assert_ne!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
        assert_eq!(
            o.pos,
            at,
            "stays where it touched down: {}",
            describe(&w, a)
        );
        assert_eq!(o.contact, Some(k));
        slept |= o.beh.current == Some(Behaviour::Sleep);
        if o.combat.ammo == 1 && rounds.is_none() {
            rounds = Some(w.tick - 1);
        }
        if o.combat.ammo == 2 {
            break;
        }
    }
    assert!(slept, "sleep current: {}", describe(&w, a));
    let first = rounds.unwrap_or_else(|| panic!("no round: {}", describe(&w, a)));
    assert!(first > q, "the first service step gives no round");
}

// ---------------------------------------------------------------------------
// AC31 part h (2): the sale while docking

// spec: S-0501 claim 24 (r110, Q-2437: a pad that starts selling sends every aircraft in contact the run-away notice: it ends the contact, gives a landing spot, F8, as destination, which ends the landing at once, claim 8, queues move and, in enter, guard after it, which replaces the move) and claim 8 (the new destination makes it take off); AC31 part h (2)
#[test]
fn ac31h2_the_sale_while_docking_ends_the_contact_and_the_landing() {
    let (mut w, k, a) = approaching(&rules());
    let mut l = None;
    for _ in 0..2000 {
        step(&mut w);
        if st(&w, a).landing && w.obj(a).unwrap().contact == Some(k) {
            l = Some(w.tick - 1);
            break;
        }
    }
    let l = l.unwrap_or_else(|| panic!("no landing: {}", describe(&w, a)));
    while w.tick <= l + 3 {
        step(&mut w);
    }
    assert!(
        st(&w, a).landing && st(&w, a).enter_docking,
        "{}",
        describe(&w, a)
    );
    let h0 = air::height(&w, a);
    assert!(h0 > 0, "not yet down: {}", describe(&w, a));
    let s = sell_order(&mut w, k);
    let o = w.obj(a).unwrap();
    assert_eq!(o.contact, None, "the notice ends the contact");
    let Some(Target::Cell(spot)) = o.dest else {
        panic!("a landing spot: {}", describe(&w, a))
    };
    assert!(!in_foundation(&w, k, spot), "not the pad");
    assert!(
        o.beh.queued == Some(Behaviour::Guard) || o.beh.current == Some(Behaviour::Guard),
        "guard queued or current: {}",
        describe(&w, a)
    );
    let pad = foundation(&w, k);
    let mut rose = false;
    for _ in 0..3 {
        let h = air::height(&w, a);
        step(&mut w);
        rose |= air::height(&w, a) > h;
    }
    assert!(rose, "its height rises: {}", describe(&w, a));
    // The pad is removed when its sale ends; its foundation is read above.
    for _ in 0..400 {
        let Some(o) = w.obj(a) else { break };
        assert!(
            !(air::height(&w, a) == 0 && pad.contains(&o.pos.cell())),
            "never touches down on the pad: {}",
            describe(&w, a)
        );
        step(&mut w);
    }
    assert!(s > l + 3);
}

// ---------------------------------------------------------------------------
// AC31 part h (3): the pad-bound aircraft at a docking position off the pad

// spec: S-0501 claim 6 (a pad-bound aircraft whose cell holds no building it is in contact with does not land) and claim 24 (r110, Q-2436: it holds the contact with enter current and the pad as destination above the docking position, and the pad never services it); AC31 part h (3)
#[test]
fn ac31h3_a_pad_bound_aircraft_holds_above_a_docking_position_off_the_pad() {
    let (mut w, k, c) = without_contact(&off_rules(), "BOUND");
    let dc = glasnost_sim::econ::release::pad_position(&w, k, c).0.cell();
    let mut a = None;
    for _ in 0..3000 {
        step(&mut w);
        if near(w.obj(c).unwrap().pos.cell(), dc) && w.obj(c).unwrap().contact == Some(k) {
            a = Some(w.tick - 1);
            break;
        }
    }
    let a = a.unwrap_or_else(|| panic!("never near the docking position: {}", describe(&w, c)));
    while w.tick <= a + 600 {
        let o = w.obj(c).unwrap();
        assert_eq!(o.contact, Some(k), "{}", describe(&w, c));
        assert_eq!(o.beh.current, Some(Behaviour::Enter), "{}", describe(&w, c));
        assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, c));
        assert!(!st(&w, c).landing, "no landing: {}", describe(&w, c));
        assert!(
            air::height(&w, c) > 0,
            "above the ground: {}",
            describe(&w, c)
        );
        assert!(near(o.pos.cell(), dc), "{}", describe(&w, c));
        assert_eq!(service(&w, k), None, "never serviced");
        step(&mut w);
    }
}

// ---------------------------------------------------------------------------
// The second rule and the load request

// spec: S-0501 claim 24 (r110, Q-2437: the second rule: an aircraft in flight with no destination asks its contact to be loaded; without a contact nobody confirms it, so it falls back to rest at once, whatever is queued, and the processing returns 1 tick) and claim 31 (that fall-back leaves the ready mark as it is)
#[test]
fn claim_24_the_second_rule_in_flight_without_a_destination_falls_back() {
    let (mut w, _k, a) = without_contact(&rules(), "JET");
    // In flight.
    for _ in 0..200 {
        if air::height(&w, a) > 100 {
            break;
        }
        step(&mut w);
    }
    assert!(!air::on_ground(&w, a), "{}", describe(&w, a));
    let (t, d) = at_next_processing(&mut w, a, |w| {
        let o = w.obj_mut(a).unwrap();
        o.dest = None;
        o.contact = None;
        o.beh.queue(Some(Behaviour::AreaGuard));
    });
    assert_eq!(d, 1, "{}", describe(&w, a));
    assert_eq!(
        w.obj(a).unwrap().beh.last_common_effects,
        Some(t),
        "a fall-back at once, whatever is queued: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 24 (r110, Q-2436: an aircraft in flight with no destination asks its contact to be loaded; a pad that is switched on confirms it and, when the aircraft has settled, no destination counting as settled, and the pad accepts it, gives itself as destination and has it queue enter with an immediate start, replacing a queued behaviour, so entry starts again; a switched-off pad refuses it, which ends the contact, and the aircraft falls back to rest at once)
#[test]
fn claim_24_the_load_request_confirmed_and_refused() {
    for on in [true, false] {
        let (mut w, k, a) = approaching(&rules());
        for _ in 0..200 {
            if air::height(&w, a) > 100 && w.obj(a).unwrap().beh.due(w.tick + 1) {
                break;
            }
            step(&mut w);
        }
        assert!(!air::on_ground(&w, a), "{}", describe(&w, a));
        let (t, d) = at_next_processing(&mut w, a, |w| {
            if !on {
                switch(w, k, false);
            }
            let o = w.obj_mut(a).unwrap();
            o.dest = None;
            o.beh.queue(Some(Behaviour::AreaGuard));
        });
        let o = w.obj(a).unwrap();
        if on {
            assert_eq!(o.contact, Some(k), "{}", describe(&w, a));
            assert_eq!(o.dest, Some(Target::Object(k)), "the pad as destination");
            assert_eq!(o.beh.current, Some(Behaviour::Enter));
            assert_eq!(o.beh.queued, None, "the queued behaviour replaced and lost");
            assert_eq!(o.beh.start_tick, t, "entry starts again");
            assert_ne!(o.beh.last_common_effects, Some(t), "no fall-back");
            // spec: S-0501 claim 24 (r134, Q-2650: entry's beginning, with its 1-tick delay, is made within that processing, in the approaching stage as in the docking stage, so the next processing is the approaching stage's)
            assert_eq!(d, 1, "entry's beginning in that processing");
            let (t2, d2) = next_processing(&mut w, a);
            assert_eq!((t2, d2), (t + 1, 3), "{}", describe(&w, a));
        } else {
            assert_eq!(d, 1);
            assert_eq!(o.contact, None, "the refusal ends the contact");
            assert_eq!(
                o.beh.last_common_effects,
                Some(t),
                "a fall-back at once: {}",
                describe(&w, a)
            );
        }
    }
}

// spec: S-0501 claim 24 (r110, Q-2436: the load request is asked in flight with no destination and for a pad-bound aircraft in flight in the approaching stage only; an aircraft that is not pad-bound and has a destination asks nothing, so a switched-off pad does not end its contact)
#[test]
fn claim_24_no_load_request_with_a_destination_unless_pad_bound() {
    for bound in [false, true] {
        let name = if bound { "BOUND" } else { "JET" };
        let (mut w, k, a) = without_contact(&rules(), name);
        if bound {
            // In flight before its first processing: on the ground away
            // from the pad the pad-bound rule would end its contact.
            let o = w.obj_mut(a).unwrap();
            o.pos.z += 300;
            o.air.as_mut().unwrap().ground = false;
        }
        next_processing(&mut w, a);
        next_processing(&mut w, a);
        assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
        for _ in 0..200 {
            if air::height(&w, a) > 100 {
                break;
            }
            step(&mut w);
        }
        assert!(!air::on_ground(&w, a) && !st(&w, a).landing);
        at_next_processing(&mut w, a, |w| switch(w, k, false));
        let o = w.obj(a).unwrap();
        assert_eq!(
            o.contact.is_none(),
            bound,
            "bound {bound}: {}",
            describe(&w, a)
        );
    }
}

// ---------------------------------------------------------------------------
// The approaching stage on the ground and quirks.aircraft_grounded_enter_stall

// spec: S-0501 claim 24 (r110, Q-2436: a processing in contact in the approaching stage that finds a pad-bound aircraft on the ground and not landing moves it on to the docking stage, and the next processing reports entry wherever it stands; one that is not pad-bound stays: each processing returns 3 ticks and changes nothing, and it is never serviced) and Edge cases (quirks.aircraft_grounded_enter_stall: on, the original, it stays; off, it is moved on as a pad-bound one is, so it reports entry at its next processing and is serviced)
#[test]
fn claim_24_grounded_in_contact_stall_switch_both_positions() {
    for (name, stall) in [("JET", true), ("JET", false), ("BOUND", true)] {
        let mut r = rules();
        r.globals.quirks.aircraft_grounded_enter_stall = stall;
        let (mut w, k, a) = grounded_in_contact(&r, name);
        // Entry's first processing: the 1-tick delay.
        let (_, d) = next_processing(&mut w, a);
        assert_eq!(d, 1);
        let (_, d) = next_processing(&mut w, a);
        assert_eq!(d, 3, "{name} stall {stall}: {}", describe(&w, a));
        let moved = name == "BOUND" || !stall;
        assert_eq!(
            st(&w, a).enter_docking,
            moved,
            "{name} stall {stall}: {}",
            describe(&w, a)
        );
        if moved {
            let (_, d) = next_processing(&mut w, a);
            assert_eq!(d, 1);
            let o = w.obj(a).unwrap();
            assert_eq!(
                o.beh.current,
                Some(Behaviour::Guard),
                "{name}: the entered report: {}",
                describe(&w, a)
            );
            step(&mut w);
            assert!(service(&w, k).is_some(), "{name}: serviced");
        } else {
            for _ in 0..10 {
                let (_, d) = next_processing(&mut w, a);
                assert_eq!(d, 3, "{}", describe(&w, a));
                let o = w.obj(a).unwrap();
                assert_eq!(o.beh.current, Some(Behaviour::Enter));
                assert_eq!(o.contact, Some(k));
                assert!(!st(&w, a).enter_docking);
            }
            assert_eq!(service(&w, k), None, "never serviced");
        }
    }
}

// ---------------------------------------------------------------------------
// Claim 31's kept ready mark and quirks.aircraft_ready_mark_outlives_enter

// spec: S-0501 claim 31 (r110, Q-2438: the move on to the docking stage clears the ready mark, a kept mark; every change of behaviour, forced or from the queue, leaves it as it is, so an aircraft forced into guard below cruise height stays not ready there and a queued behaviour waits) and Edge cases (quirks.aircraft_ready_mark_outlives_enter: on, the original; off, the cleared mark holds the aircraft not ready only while enter is current, so the queued behaviour becomes current at claim 31's next point)
#[test]
fn claim_31_the_ready_mark_after_a_forced_change_switch_both_positions() {
    for outlives in [true, false] {
        let mut r = rules();
        r.globals.quirks.aircraft_ready_mark_outlives_enter = outlives;
        let (mut w, _k, a) = approaching(&r);
        to_docking_stage(&mut w, a);
        assert!(st(&w, a).mark_cleared, "{}", describe(&w, a));
        let t = w.tick;
        {
            let o = w.obj_mut(a).unwrap();
            o.combat.ammo = 2;
            o.beh.force(Behaviour::Guard, t, true);
            o.beh.queue(Some(Behaviour::AreaGuard));
        }
        let cruise = air::height(&w, a).max(1);
        let mut started = None;
        for i in 0..12 {
            step(&mut w);
            let o = w.obj(a).unwrap();
            if o.beh.current == Some(Behaviour::AreaGuard) {
                started = Some(i);
                break;
            }
            assert!(air::height(&w, a) <= cruise.max(400), "below cruise height");
        }
        if outlives {
            assert_eq!(started, None, "the queue held: {}", describe(&w, a));
            assert!(!process::ready(&w, a), "not ready in guard");
        } else {
            assert!(
                started.is_some_and(|i| i <= 1),
                "at claim 31's next point: {started:?} {}",
                describe(&w, a)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC4's same-cell part: claim 8's exception

// spec: S-0501 claim 8 (r110, Q-2437: a landing aircraft given a destination in the cell of the point it is landing at ignores it: no take-off, it goes on landing to a touchdown in that cell; a destination two cells away makes it take off; r120, Q-2507: that point is its flight point, which landing where it is puts in the cell claim 40's selection gives at the order, and this run asserts the touchdown cell; the flight point's cell apart from the cell it is over is `claims_s0501_r120::ac4_runs_2_and_3_…`) and claim 6 (no destination in flight: it lands where it is); AC4 same-cell part
#[test]
fn ac4_same_cell_a_landing_aircraft_ignores_a_destination_in_its_cell() {
    for far in [false, true] {
        let mut w = game(&rules());
        ready(&mut w);
        let a = put(&mut w, "JET", 0, 0);
        step(&mut w);
        let c = w.obj(a).unwrap().pos.cell();
        let goal = CellXY::new(c.x + 8, c.y);
        assert!(issue(
            &mut w,
            0,
            Order::Behaviour {
                actor: a,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(goal)),
            }
        ));
        for _ in 0..200 {
            if air::height(&w, a) > 200 {
                break;
            }
            step(&mut w);
        }
        assert!(issue(&mut w, 0, Order::Stop { actor: a }));
        // A landing where it is, and once it no longer moves across the
        // ground: while it still slows down it can leave the cell (Q-2507).
        let mut started = false;
        for _ in 0..300 {
            let p = w.obj(a).unwrap().pos;
            step(&mut w);
            let q = w.obj(a).unwrap().pos;
            if st(&w, a).landing && air::height(&w, a) > 0 && (p.x, p.y) == (q.x, q.y) {
                started = true;
                break;
            }
        }
        assert!(started, "a landing where it is: {}", describe(&w, a));
        let here = w.obj(a).unwrap().pos.cell();
        let to = if far {
            CellXY::new(here.x, here.y + 2)
        } else {
            here
        };
        assert!(issue(
            &mut w,
            0,
            Order::Behaviour {
                actor: a,
                behaviour: Behaviour::Move,
                attack_move: false,
                target: None,
                dest: Some(Target::Cell(to)),
            }
        ));
        let mut h = air::height(&w, a);
        let mut took_off = false;
        for _ in 0..600 {
            step(&mut w);
            let s = st(&w, a);
            let nh = air::height(&w, a);
            if s.taking_off || nh > h {
                took_off = true;
                break;
            }
            h = nh;
            if nh == 0 && !s.landing {
                break;
            }
        }
        if far {
            assert!(took_off, "a take-off at the order: {}", describe(&w, a));
        } else {
            assert!(!took_off, "no take-off (given {to:?}): {}", describe(&w, a));
            assert_eq!(air::height(&w, a), 0, "{}", describe(&w, a));
            assert_eq!(
                w.obj(a).unwrap().pos.cell(),
                here,
                "a touchdown in that cell"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Claim 29 at cruise height, the heading-pad record and T2's enter row

// spec: S-0501 claim 29 (r110, Q-2437: at exactly cruise height, outside a team, an armed aircraft falls back to rest whatever its destination, and that processing sets the ready mark, claim 31; an unarmed one moves to its own cell)
#[test]
fn claim_29_guard_at_cruise_height_outside_a_team() {
    let (mut w, _k, a) = without_contact(&rules(), "JET");
    let mut cruise = false;
    for _ in 0..400 {
        step(&mut w);
        let s = st(&w, a);
        if !s.taking_off && !s.landing && air::height(&w, a) > 0 && s.held == air::height(&w, a) {
            cruise = true;
            break;
        }
    }
    assert!(cruise, "{}", describe(&w, a));
    let h = air::height(&w, a);
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.combat.ammo = 2;
        o.contact = None;
        o.beh.force(Behaviour::Guard, t, true);
        o.air.as_mut().unwrap().mark_cleared = true;
    }
    let (t1, _) = next_processing(&mut w, a);
    assert_eq!(air::height(&w, a), h, "at cruise height");
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.last_common_effects,
        Some(t1),
        "armed: a fall-back whatever its destination: {}",
        describe(&w, a)
    );
    assert!(!st(&w, a).mark_cleared, "that processing sets the mark");
}

// spec: S-0501 claim 24 (r103: the pad-bound rule records the destination as the pad it was last heading for; r110, Q-2436: any other destination, such as a cell, is recorded as no pad, and the earlier record is lost)
#[test]
fn claim_24_the_pad_bound_rule_records_a_cell_destination_as_no_pad() {
    let (mut w, k, a) = grounded_in_contact(&rules(), "BOUND");
    {
        let o = w.obj_mut(a).unwrap();
        let c = o.pos.cell();
        o.dest = Some(Target::Cell(CellXY::new(c.x + 3, c.y)));
        o.air.as_mut().unwrap().heading_pad = Some(k);
        o.air.as_mut().unwrap().flying_to = o.dest;
    }
    let (_, d) = next_processing(&mut w, a);
    assert_eq!(d, 1);
    let o = w.obj(a).unwrap();
    assert_eq!(o.contact, None, "{}", describe(&w, a));
    assert_eq!(st(&w, a).heading_pad, None, "recorded as no pad");
    assert!(
        o.beh.current == Some(Behaviour::Move) || o.beh.queued == Some(Behaviour::Move),
        "{}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 46 and T2 (r110: the row of an armed aircraft in flight already flying to its enter destination needs only that destination: a fall-back in enter that keeps a destination keeps it and gives guard, without a pad search) and claim 24 (r110: the docking stage without a contact does not forget the destination before that fall-back)
#[test]
fn t2_the_enter_row_keeps_the_destination_and_gives_guard() {
    let (mut w, _k, a) = without_contact(&rules(), "JET");
    for _ in 0..200 {
        if air::height(&w, a) > 100 {
            break;
        }
        step(&mut w);
    }
    let spot = CellXY::new(3, 3);
    {
        let o = w.obj_mut(a).unwrap();
        o.combat.ammo = 1;
        o.contact = None;
        o.dest = Some(Target::Cell(spot));
    }
    let t = w.tick;
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Cell(spot)), "{}", describe(&w, a));
    assert!(
        o.beh.current == Some(Behaviour::Guard) || o.beh.queued == Some(Behaviour::Guard),
        "guard: {}",
        describe(&w, a)
    );
    assert_eq!(w.tick, t);
}
