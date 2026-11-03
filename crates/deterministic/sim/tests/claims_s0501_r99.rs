//! Claim tests for S-0501 r99 (Q-2326): what a processing of *enter*
//! does for an aircraft with a pad destination and no contact (claim 24),
//! after claim 18's search after the row, after a refusal that ends the
//! contact, and after a sale; AC31 part f. Synthetic data through the real
//! tick path; every type, weapon and number is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready, switch};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::econ::release::Exit;
use glasnost_sim::orders::Order;
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with `docks` docking places and the displacements `offsets`); JET
/// (armed, landable, not pad-bound, 2 rounds of the weapon Agm, docking at
/// APAD).
fn rules_with(docks: i32, offsets: Option<Vec<[i32; 3]>>) -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    let mut pad = factory("aircraft");
    // A build-up sprite, so that a sell order sells it (S-0408 claim 4).
    pad.econ.buildup = Some(true);
    pad.econ.buildup_frames = Some(10);
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(docks);
    pad.econ.docking_offsets = offsets;
    push(&mut r, "buildings", "APAD");
    r.sim.buildings.insert("APAD".into(), pad);
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
    push(&mut r, "aircraft", "JET");
    r.sim.aircraft.insert("JET".into(), u);
    r
}

fn rules() -> Ruleset {
    rules_with(1, None)
}

fn put(w: &mut World, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Aircraft, "JET", 0, testkit::mid(SIZE, dx, dy), 0)
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
        "t={} pos={:?} h={} beh={:?}/{:?} last={:?} dest={:?} contact={:?} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.beh.current,
        o.beh.queued,
        o.beh.last,
        o.dest,
        o.contact,
        o.air
    )
}

fn has(w: &World, id: ObjectId, b: Behaviour) -> bool {
    let o = w.obj(id).unwrap();
    o.beh.current == Some(b) || o.beh.queued == Some(b)
}

/// AC31 part f's start: a pad *P* of house 0 and a JET of house 0 on open
/// ground 8 cells from it, its ammunition spent and no contact, given by
/// claim 18's search after T2's ground row *P* as destination and *enter*
/// with no contact. Returns (world, P, A).
fn without_contact(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, 8, 0);
    // A tick after the placement fall-back.
    step(&mut w);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    assert!(air::on_ground(&w, a) && w.obj(a).unwrap().contact.is_none());
    process::fall_back(&mut w, a, false);
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
    assert_eq!(o.beh.current, Some(Behaviour::Enter), "{}", describe(&w, a));
    assert_eq!(o.contact, None, "the search asks for no contact");
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

/// Queues `b` for `a` without an immediate start (S-0215 claim 12's
/// queue, before the aircraft's processing).
fn queue(w: &mut World, a: ObjectId, b: Behaviour) {
    w.obj_mut(a).unwrap().beh.queue(Some(b));
}

// ---------------------------------------------------------------------------
// AC31 part f: the processing without a contact, nothing queued

// spec: S-0501 claim 24 (r99, Q-2326: the first processing of enter only starts entry, 1 tick, and makes no contact; the second, 1 tick later, finds no contact with nothing queued: it forgets the destination and falls back to rest, claim 18, whose pad search gives the pad back and the contact within that processing; the fall-back's queued enter changes nothing and starts nothing, so the next processing, 1 tick later, finds the contact and goes on in the approaching stage, 3 ticks per processing) and claim 18 (r99: queueing enter while it is current with nothing queued, S-0215 claims 7 and 8); AC31 part f
#[test]
fn claim_24_ac31f_the_second_processing_falls_back_and_entry_is_not_restarted() {
    let (mut w, k, a) = without_contact(&rules());
    let start = w.obj(a).unwrap().beh.start_tick;
    let (t1, d1) = next_processing(&mut w, a);
    assert_eq!(d1, 1, "entry begins with a 1-tick delay");
    let o = w.obj(a).unwrap();
    assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
    assert_eq!(o.contact, None, "the first processing makes no contact");
    let (t2, d2) = next_processing(&mut w, a);
    assert_eq!(t2, t1 + 1, "{}", describe(&w, a));
    assert_eq!(d2, 1, "a processing that finds no contact returns 1 tick");
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.last_common_effects, Some(t2), "a fall-back to rest");
    assert_eq!(o.contact, Some(k), "{}", describe(&w, a));
    assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
    assert_eq!(o.beh.current, Some(Behaviour::Enter));
    assert_eq!(o.beh.start_tick, start, "entry is not started again");
    let (t3, d3) = next_processing(&mut w, a);
    assert_eq!(t3, t1 + 2, "{}", describe(&w, a));
    assert_eq!(d3, 3, "in contact, approaching: {}", describe(&w, a));
    let (t4, _) = next_processing(&mut w, a);
    assert_eq!(t4, t1 + 5, "none in t1 + 3 or t1 + 4: {}", describe(&w, a));
    // spec: S-0501 claim 24 (it then flies to the pad and docks: sleep)
    let mut docked = false;
    for _ in 0..1500 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        if o.beh.current == Some(Behaviour::Sleep) && o.contact == Some(k) {
            docked = true;
            break;
        }
    }
    assert!(docked, "{}", describe(&w, a));
}

// spec: S-0501 claim 24 (r99: a pad with no free contact place is not found again, claim 21: the fall-back at the second processing takes claim 18's outcome when no pad grants contact) and T2 (here the take-off has begun on the ground: the ground row forgets the destination, guard; after the row the new pad search finds none and the aircraft is not pad-bound)
#[test]
fn claim_24_with_the_pad_full_the_fall_back_takes_claim_18s_outcome() {
    let (mut w, k, a) = without_contact(&rules());
    let (t1, _) = next_processing(&mut w, a);
    // Another aircraft takes P's one contact place between the two processings.
    let b = put(&mut w, -8, 0);
    {
        let o = w.obj_mut(b).unwrap();
        o.contact = Some(k);
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: false,
            place: 0,
        });
    }
    let (t2, d2) = next_processing(&mut w, a);
    assert_eq!((t2, d2), (t1 + 1, 1), "{}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.last_common_effects, Some(t2), "a fall-back to rest");
    assert_eq!(o.contact, None, "{}", describe(&w, a));
    assert_eq!(o.dest, None, "the ground row: {}", describe(&w, a));
    assert!(has(&w, a, Behaviour::Guard), "{}", describe(&w, a));
}

// ---------------------------------------------------------------------------
// The processing without a contact, a behaviour queued

// spec: S-0501 claim 24 (r99, Q-2326: with a behaviour queued, a processing of enter after its first that finds no contact keeps the destination and enter, asks no pad and waits, returning 1 tick, whether or not the pad has a free contact place; a behaviour queued without an immediate start before the aircraft's processing, S-0215 claim 12, is still queued at it) and claim 31 (the third start opportunity then makes the queued behaviour current at the end of the update)
#[test]
fn claim_24_with_a_behaviour_queued_it_keeps_the_pad_and_waits() {
    for full in [false, true] {
        let (mut w, k, a) = without_contact(&rules());
        let (t1, _) = next_processing(&mut w, a);
        if full {
            let b = put(&mut w, -8, 0);
            let o = w.obj_mut(b).unwrap();
            o.contact = Some(k);
            o.econ.exit = Some(Exit {
                factory: k,
                tethered: false,
                place: 0,
            });
        }
        let (t2, d2) = at_next_processing(&mut w, a, |w| queue(w, a, Behaviour::Guard));
        assert_eq!((t2, d2), (t1 + 1, 1), "full {full}: {}", describe(&w, a));
        let o = w.obj(a).unwrap();
        assert_ne!(o.beh.last_common_effects, Some(t2), "no fall-back");
        assert_eq!(o.contact, None, "asks no pad: {}", describe(&w, a));
        assert_eq!(
            o.dest,
            Some(Target::Object(k)),
            "keeps the destination: {}",
            describe(&w, a)
        );
        assert_eq!(
            o.beh.current,
            Some(Behaviour::Guard),
            "the queued behaviour became current after the processing: {}",
            describe(&w, a)
        );
    }
}

// ---------------------------------------------------------------------------
// A refusal and a sale during the approach

/// Runs `without_contact` to the approaching stage in contact (the
/// second processing's fall-back made the contact).
fn approaching(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let (mut w, k, a) = without_contact(r);
    next_processing(&mut w, a);
    next_processing(&mut w, a);
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
    (w, k, a)
}

// spec: S-0501 claim 24 (r99: a refusal that ends the contact while the pad stays the destination leaves it to the processing without a contact: with a behaviour queued it keeps the pad and enter and waits; with none it forgets the destination and falls back to rest) and claim 46 (a refusal clears the destination when no behaviour is queued); Open question 19 (the refusal itself stays open: our engine meets it at enter's request to be loaded, here a pad switched off, S-0501 claim 22, and the processing without a contact follows in that processing)
#[test]
fn claim_24_a_refusal_ends_the_contact_and_the_processing_without_one_follows() {
    for queued in [false, true] {
        let (mut w, k, a) = approaching(&rules());
        let (t, d) = at_next_processing(&mut w, a, |w| {
            switch(w, k, false);
            if queued {
                queue(w, a, Behaviour::Guard);
            }
        });
        assert_eq!(d, 1, "queued {queued}: {}", describe(&w, a));
        let o = w.obj(a).unwrap();
        assert_eq!(o.contact, None, "the contact ended: {}", describe(&w, a));
        if queued {
            assert_ne!(o.beh.last_common_effects, Some(t), "no fall-back");
            assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
        } else {
            assert_eq!(o.beh.last_common_effects, Some(t), "a fall-back");
            // spec: S-0501 T2 (the take-off has begun on the ground: the ground row forgets the destination, guard; the switched-off pad grants no contact after the row)
            assert_eq!(o.dest, None, "{}", describe(&w, a));
            assert!(has(&w, a, Behaviour::Guard), "{}", describe(&w, a));
        }
    }
}

// spec: S-0501 claim 24 (r99: if the pad stops being its destination while it approaches, a sale, claim 32, the aircraft falls back to rest, even with a behaviour queued) and claim 46 (a sale removes the pad as the aircraft's destination before it falls back)
#[test]
fn claim_24_a_sale_during_the_approach_falls_back_with_a_behaviour_queued() {
    let (mut w, k, a) = approaching(&rules());
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(issue(&mut w, 0, Order::Sell { actor: k }));
    let mut selling = false;
    for _ in 0..20 {
        step(&mut w);
        let b = w.obj(k).unwrap();
        if b.econ.sale.is_some() || b.beh.current == Some(Behaviour::Selling) {
            selling = true;
            break;
        }
    }
    assert!(selling, "the sale started");
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
    let (t, d) = at_next_processing(&mut w, a, |w| queue(w, a, Behaviour::Guard));
    assert_eq!(d, 1, "{}", describe(&w, a));
    assert!(w.obj(k).is_some(), "while the pad is being sold");
    let o = w.obj(a).unwrap();
    assert_eq!(o.beh.last_common_effects, Some(t), "a fall-back");
    assert_eq!(o.contact, None);
    assert_ne!(
        o.dest,
        Some(Target::Object(k)),
        "the pad is no longer its destination: {}",
        describe(&w, a)
    );
}

// ---------------------------------------------------------------------------
// S-0404 r23 claim 15: the released position follows the contact place

// spec: S-0404 claim 15 (r23: an aircraft not in contact takes the first empty place, which is the place a new contact takes, so the released position follows the contact place; an aircraft already in contact keeps its place) and S-0501 claim 23 (a granted contact holds the pad's place)
#[test]
fn s0404_claim_15_the_release_place_is_the_place_a_new_contact_takes() {
    let r = rules_with(3, Some(vec![[10, 0, 0], [20, 0, 0], [30, 0, 0]]));
    let mut w = game(&r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, 8, 0);
    let b = put(&mut w, -8, 0);
    let c = put(&mut w, 0, 8);
    let d = put(&mut w, 0, -8);
    // b holds place 1, so place 0 is the first empty place and 2 the next.
    {
        let o = w.obj_mut(b).unwrap();
        o.contact = Some(k);
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: false,
            place: 1,
        });
    }
    let place_of = |w: &World, id: ObjectId| w.obj(id).unwrap().econ.exit.unwrap().place;
    for (u, want) in [(a, 0), (c, 2)] {
        let (pos, release) = glasnost_sim::econ::release::pad_position(&w, k, u);
        assert_eq!(release, want, "the release's place");
        assert!(air::request_contact(&mut w, u, k), "contact granted");
        assert_eq!(place_of(&w, u), release, "the contact's place");
        // The aircraft now in contact keeps that place and that position.
        assert_eq!(
            glasnost_sim::econ::release::pad_position(&w, k, u),
            (pos, release)
        );
    }
    // Every place held: the release has none below the count, and no contact
    // is granted.
    assert_eq!(glasnost_sim::econ::release::pad_position(&w, k, d).1, 3);
    assert!(!air::request_contact(&mut w, d, k));
}

// ---------------------------------------------------------------------------
// A waiting aircraft landing over its pad

// spec: S-0501 claim 24 (r99: a waiting aircraft that lands over its destination pad is moved while landing as the docking stage moves one in contact, toward a docking position that does not depend on whether the pad has a free contact place: with one docking place that place's docking position, otherwise the pad's foundation centre with no displacement) and S-0404 claim 15 (r23: the same docking position for an aircraft that lands over the pad without a contact); S-0501 claim 24 (each axis by the difference limited to −5 … +5 leptons, the height unchanged; so does an aircraft whose contact ends during the docking stage)
#[test]
fn claim_24_a_waiting_aircraft_is_moved_toward_its_docking_position_without_a_contact() {
    let offsets = Some(vec![[0, -30, 0], [30, 0, 0]]);
    for docks in [1, 2] {
        let (mut w, k, a) = approaching(&rules_with(docks, offsets.clone()));
        let c = w.reference_point(k);
        let p0 = Pos::new(c.x, c.y - 30, 0);
        // In contact through place 0, the docking stage moves it until it
        // stands over that place's docking position, still landing.
        let mut docking = false;
        for _ in 0..1500 {
            let s = st(&w, a);
            let p = w.obj(a).unwrap().pos;
            if s.landing && s.enter_docking && air::height(&w, a) > 20 && (p.x, p.y) == (p0.x, p0.y)
            {
                docking = true;
                break;
            }
            step(&mut w);
        }
        assert!(docking, "docks {docks}: {}", describe(&w, a));
        let mut before = w.obj(a).unwrap().pos;
        let mut height = air::height(&w, a);
        let (_, d) = at_next_processing(&mut w, a, |w| {
            before = w.obj(a).unwrap().pos;
            height = air::height(w, a);
            switch(w, k, false);
            queue(w, a, Behaviour::Guard);
        });
        assert_eq!(d, 1, "docks {docks}: {}", describe(&w, a));
        let o = w.obj(a).unwrap();
        assert_eq!(o.contact, None, "{}", describe(&w, a));
        assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
        assert!(height > 0 && st(&w, a).landing, "{}", describe(&w, a));
        let clamp = |d: i32| d.clamp(-5, 5);
        assert_ne!(
            clamp(p0.y - before.y),
            clamp(c.y - before.y),
            "the set-up tells the two positions apart: {}",
            describe(&w, a)
        );
        let to = if docks == 1 {
            // One docking place: its displacement, though no place is held.
            p0
        } else {
            // Several: the foundation centre with no displacement.
            c
        };
        assert_eq!(
            (o.pos.x - before.x, o.pos.y - before.y),
            (clamp(to.x - before.x), clamp(to.y - before.y)),
            "docks {docks}: {}",
            describe(&w, a)
        );
    }
}
