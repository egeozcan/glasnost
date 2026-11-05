//! Claim tests for S-0501 r103 (Q-2366): the wait of a processing of
//! *enter* without a contact, the docking stage without a contact, the
//! touchdown on the pad, the pad-bound rule that comes before every other
//! rule of claim 24, the heading point of a pad without a contact, the
//! processing-cleared ready mark of claim 31 and AC31 part g. Synthetic data
//! through the real tick path; every type, weapon and number is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready};
use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::combat::flying;
use glasnost_sim::econ::release::Exit;
use glasnost_sim::orders::Order;
use glasnost_sim::process;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

fn pad_type(docks: i32, offsets: Option<Vec<[i32; 3]>>) -> glasnost_rules::sim::BuildingType {
    let mut pad = factory("aircraft");
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(docks);
    pad.econ.docking_offsets = offsets;
    pad
}

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with `docks` docking places and the displacements `offsets`); QPAD (the
/// same with one docking place and a zero displacement, a dock type of no
/// aircraft here); JET (armed, landable, not pad-bound, 2 rounds of the
/// weapon Agm, docking at APAD); BOUND (as JET, pad-bound).
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
    push(&mut r, "buildings", "QPAD");
    r.sim
        .buildings
        .insert("QPAD".into(), pad_type(1, Some(vec![[0, 0, 0]])));
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

fn foundation(w: &World, k: ObjectId) -> Vec<CellXY> {
    let b = w.obj(k).unwrap();
    glasnost_sim::econ::placement::foundation_cells(w, b.type_index, b.pos.cell())
}

fn in_foundation(w: &World, k: ObjectId, c: CellXY) -> bool {
    foundation(w, k).contains(&c)
}

/// The aircraft that hold a contact with building `k`.
fn holders(w: &World, k: ObjectId) -> Vec<ObjectId> {
    w.objects
        .values()
        .filter(|o| o.contact == Some(k))
        .map(|o| o.id)
        .collect()
}

/// AC31 part f's start: a pad *P* of house 0 and a JET of house 0 on open
/// ground 8 cells from it, its ammunition spent and no contact, given by
/// claim 18's search after T2's ground row *P* as destination and *enter*
/// with no contact. Returns (world, P, A).
fn without_contact(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, "JET", 8, 0);
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

/// Ends `a`'s contact (as claim 32's end of a contact would; r110 asks the
/// pad no request while a destination is held, so a switched-off pad no
/// longer ends it in these stages).
fn end_contact(w: &mut World, a: ObjectId) {
    let o = w.obj_mut(a).unwrap();
    o.contact = None;
    o.econ.exit = None;
}

/// Runs `without_contact` to the approaching stage in contact (the
/// second processing's fall-back made the contact).
fn approaching(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let (mut w, k, a) = without_contact(r);
    next_processing(&mut w, a);
    next_processing(&mut w, a);
    assert_eq!(w.obj(a).unwrap().contact, Some(k), "{}", describe(&w, a));
    (w, k, a)
}

/// Runs an approaching aircraft in contact until the processing of *enter*
/// that finds it landing over its pad and moves it on to the docking stage
/// (3 ticks); returns that processing's tick.
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

// ---------------------------------------------------------------------------
// AC31 part g (1): the wait under the guard command

// spec: S-0501 claim 24 (r103, Q-2366: with a behaviour queued, a processing of enter after its first that finds no contact waits; that processing leaves the aircraft ready to change and in enter it carries no run lock, so the queued behaviour becomes current at the end of that tick, claim 31's third point: the wait lasts that one processing; the wait is the same when the order that queued the behaviour also gave the aircraft a destination other than the pad, such as a cell) and S-0213 table T2 (the guard command: a behaviour order area guard with the cell the object stands in as its target) and S-0215 claim 12 (it queues area guard without an immediate start; the order's cell as guard spot and destination); AC31 part g (1)
#[test]
fn claim_24_ac31g1_the_guard_commands_area_guard_is_current_after_one_processing() {
    let (mut w, k, a) = without_contact(&rules());
    let start = w.obj(a).unwrap().beh.start_tick;
    let mut cell = None;
    // In the cycle of t1, A's first processing of enter, the guard command;
    // the order takes effect after every object's turn (S-0203 claim 7).
    let (t1, d1) = at_next_processing(&mut w, a, |w| {
        assert_eq!(
            w.obj(a).unwrap().beh.queued,
            None,
            "nothing queued before the order"
        );
        let c = w.obj(a).unwrap().pos.cell();
        cell = Some(c);
        assert!(issue(
            w,
            0,
            Order::Behaviour {
                actor: a,
                behaviour: Behaviour::AreaGuard,
                attack_move: false,
                target: Some(Target::Cell(c)),
                dest: None,
            }
        ));
    });
    let cell = cell.unwrap();
    assert_eq!(d1, 1, "entry begins with a 1-tick delay");
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.start_tick, start,
        "the first processing of this enter"
    );
    assert_eq!(o.beh.current, Some(Behaviour::Enter), "{}", describe(&w, a));
    assert_eq!(
        o.beh.queued,
        Some(Behaviour::AreaGuard),
        "{}",
        describe(&w, a)
    );
    assert_eq!(o.contact, None, "{}", describe(&w, a));
    assert_eq!(
        o.dest,
        Some(Target::Cell(cell)),
        "the order's cell: {}",
        describe(&w, a)
    );
    let places = holders(&w, k);
    let (t2, d2) = next_processing(&mut w, a);
    assert_eq!(t2, t1 + 1, "{}", describe(&w, a));
    assert_eq!(d2, 1, "a processing that finds no contact returns 1 tick");
    let o = w.obj(a).unwrap();
    assert_ne!(o.beh.last_common_effects, Some(t2), "a wait, no fall-back");
    assert_eq!(
        o.beh.current,
        Some(Behaviour::AreaGuard),
        "area guard current at the end of t1 + 1: {}",
        describe(&w, a)
    );
    assert_eq!(o.beh.queued, None, "{}", describe(&w, a));
    assert_eq!(o.contact, None, "no pad asked: {}", describe(&w, a));
    assert_eq!(holders(&w, k), places, "P's contact places unchanged");
}

// ---------------------------------------------------------------------------
// AC31 part g (2) and (3): a pad's cell refused to an aircraft without a contact

// spec: S-0501 F7 (r103: every cell of a building's foundation holds that building in its ground layer, so a lander that holds no contact is refused on every such cell; its own contact is accepted) and claim 40 (the move's first processing seeks a replacement cell when F7 refuses the destination's cell; a cell F7 accepts is kept); AC31 part g (2) and its control run (3)
#[test]
fn ac31g2_a_move_onto_a_pads_cell_is_replaced_and_the_control_keeps_its_cell() {
    for control in [false, true] {
        let r = rules();
        let mut w = game(&r);
        let q = building(&mut w, "QPAD", 0, -3, 0);
        ready(&mut w);
        let a = put(&mut w, "JET", 3, 0);
        // A rests in guard after its placement (claim 1).
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert!(air::on_ground(&w, a) && o.contact.is_none() && o.target.is_none());
        let qc = w.reference_point(q).cell();
        assert!(in_foundation(&w, q, qc));
        if !control {
            for c in foundation(&w, q) {
                assert!(
                    !air::landing_test(&w, a, Target::Cell(c)),
                    "{c:?} refused without a contact"
                );
            }
            w.obj_mut(a).unwrap().contact = Some(q);
            for c in foundation(&w, q) {
                assert!(
                    air::landing_test(&w, a, Target::Cell(c)),
                    "{c:?} accepted to its contact"
                );
            }
            w.obj_mut(a).unwrap().contact = None;
        }
        let ac = w.obj(a).unwrap().pos.cell();
        let to = if control {
            CellXY::new(ac.x, ac.y + (ac.x - qc.x))
        } else {
            qc
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
        step(&mut w);
        assert_eq!(
            w.obj(a).unwrap().beh.queued,
            Some(Behaviour::Move),
            "{}",
            describe(&w, a)
        );
        // A's first processing of move makes claim 40's selection.
        let mut chosen = None;
        let mut rested = None;
        let mut flew = false;
        for _ in 0..2000 {
            step(&mut w);
            let o = w.obj(a).unwrap();
            assert_ne!(o.contact, Some(q), "no contact with Q");
            assert!(
                o.beh.current != Some(Behaviour::Enter) && o.beh.queued != Some(Behaviour::Enter)
            );
            let selected = o.beh.current == Some(Behaviour::Move)
                && st(&w, a).first_moved == Some(o.beh.start_tick);
            if selected && chosen.is_none() {
                chosen = o.dest;
            }
            flew |= chosen.is_some() && !st(&w, a).ground;
            if flew && !st(&w, a).landing && st(&w, a).ground {
                rested = Some(o.pos.cell());
                break;
            }
            if flew && let Some(Target::Cell(c)) = o.dest {
                assert_eq!(
                    Some(Target::Cell(c)),
                    chosen,
                    "the destination until it lands"
                );
            }
        }
        let Some(Target::Cell(chosen)) = chosen else {
            panic!("no move destination: {}", describe(&w, a))
        };
        let rested = rested.unwrap_or_else(|| panic!("never rests: {}", describe(&w, a)));
        if control {
            assert_eq!(chosen, to, "the control keeps its cell");
            assert_eq!(rested, to, "{}", describe(&w, a));
        } else {
            assert!(
                !in_foundation(&w, q, chosen),
                "a replacement outside Q: {chosen:?}"
            );
            assert!(
                !in_foundation(&w, q, rested),
                "never rests on Q: {}",
                describe(&w, a)
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC31 part g (4): the touchdown on the pad

// spec: S-0501 claim 7 (r103: an accepted touchdown forgets the aircraft's destination in that same movement update; it makes no entry report itself: for an aircraft in enter its next processing makes the docking stage's step of claim 24, which reports entry for one in contact) and F7 (its contact's cell is accepted, so there is no rise of 10 leptons); AC31 part g (4)
#[test]
fn ac31g4_the_touchdown_on_the_pad_forgets_the_destination_and_keeps_the_contact() {
    let (mut w, k, a) = approaching(&rules());
    let mut down = false;
    let mut landing = false;
    for _ in 0..2000 {
        step(&mut w);
        let h = air::height(&w, a);
        let s = st(&w, a);
        landing |= s.landing;
        if landing {
            assert!(
                !s.taking_off,
                "no refusal and rise of 10 leptons: {}",
                describe(&w, a)
            );
        }
        if h == 0 && !s.landing && s.ground {
            down = true;
            break;
        }
    }
    assert!(down, "{}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.dest,
        None,
        "the destination forgotten: {}",
        describe(&w, a)
    );
    assert_eq!(o.contact, Some(k), "{}", describe(&w, a));
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Enter),
        "no entry report at the touchdown"
    );
    assert!(in_foundation(&w, k, o.pos.cell()), "{}", describe(&w, a));
    let service = |w: &World| w.obj(k).unwrap().air.as_ref().and_then(|s| s.service);
    assert_eq!(service(&w), None, "no entry report at the touchdown");
    // spec: S-0501 claim 24 (the next processing, in the docking stage, reports that it has entered: the pad starts servicing, claim 26; r110, Q-2438: the pad has it queue sleep and it then queues guard, which replaces the sleep, and that processing sets the ready mark, so claim 31's third point makes guard current at the end of that tick)
    let (t, _) = next_processing(&mut w, a);
    let o = w.obj(a).unwrap();
    assert!(
        service(&w).is_some(),
        "the entered report: {}",
        describe(&w, a)
    );
    assert_eq!(o.beh.current, Some(Behaviour::Guard), "{}", describe(&w, a));
    assert_eq!(o.beh.start_tick, t);
}

// ---------------------------------------------------------------------------
// The docking stage without a contact

// spec: S-0501 claim 24 (r103: in the docking stage without a contact, the first processing that finds the aircraft no longer landing makes it fall back to rest, whatever is queued; that is the processing after an accepted touchdown, whose own movement update makes no fall-back, claim 7; a contact ended once it is in the docking stage leaves it to that stage's rules without a contact; r110: the test ends it, since no request is made there, and the fall-back's pad search may grant a contact again)
#[test]
fn claim_24_after_an_accepted_touchdown_the_docking_stage_without_a_contact_falls_back() {
    let (mut w, k, a) = approaching(&rules());
    to_docking_stage(&mut w, a);
    for _ in 0..2000 {
        if st(&w, a).ground {
            break;
        }
        step(&mut w);
    }
    let o = w.obj(a).unwrap();
    assert!(
        st(&w, a).ground && o.contact == Some(k),
        "{}",
        describe(&w, a)
    );
    assert_eq!(o.beh.current, Some(Behaviour::Enter));
    assert_ne!(
        o.beh.last_common_effects,
        Some(w.tick - 1),
        "the touchdown made no fall-back"
    );
    let (t, d) = at_next_processing(&mut w, a, |w| {
        // The contact ends before the next processing (r110 asks the pad
        // nothing in the docking stage while a destination is held, so the
        // test ends it, as claim 32's end of a contact would); a behaviour
        // is queued.
        end_contact(w, a);
        w.obj_mut(a).unwrap().beh.queue(Some(Behaviour::AreaGuard));
    });
    assert_eq!(d, 1, "{}", describe(&w, a));
    let o = w.obj(a).unwrap();
    // The fall-back's pad search (T2's after-the-row entry, its ammunition
    // spent) may give the switched-on pad back with a new contact.
    assert_eq!(
        o.beh.last_common_effects,
        Some(t),
        "a fall-back whatever is queued"
    );
    assert_ne!(
        o.beh.current,
        Some(Behaviour::Sleep),
        "no entry report: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 24 (r103: in the docking stage without a contact, each processing, 1 tick apart, makes no test of contact or of a queued behaviour; while it is landing it is moved as one in contact, toward the docking position without a contact; the first processing that finds it no longer landing makes it fall back to rest: after a refused touchdown, since F7 refuses every cell of the pad's foundation to an aircraft with no contact, so it rose by 10 leptons and started to take off again, claim 7) and claim 47 (r103: a refused touchdown makes no overlap: the aircraft rises by 10 leptons before it would be listed again, so it is not listed in its cell layer at its positive height while it stays tracked) and Edge cases (r132: a refused touchdown comes only with quirks.aircraft_descent_touchdown_untested off; on, the touchdown is untested and the case is claim_24_after_an_accepted_touchdown_the_docking_stage_without_a_contact_falls_back's)
#[test]
fn claim_24_after_a_refused_touchdown_the_docking_stage_without_a_contact_falls_back() {
    let mut r = rules();
    r.globals.quirks.aircraft_descent_touchdown_untested = false;
    let (mut w, k, a) = approaching(&r);
    to_docking_stage(&mut w, a);
    // Before the next processing, in the docking stage, the contact ends
    // (set by the test, r110 asking no request there while a destination is
    // held): the stage's rules without a contact follow.
    let (t, d) = at_next_processing(&mut w, a, |w| end_contact(w, a));
    assert_eq!(d, 1, "{}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(o.contact, None);
    assert_ne!(
        o.beh.last_common_effects,
        Some(t),
        "still landing: no fall-back"
    );
    assert!(st(&w, a).landing, "{}", describe(&w, a));
    assert_eq!(o.beh.current, Some(Behaviour::Enter));
    // The touchdown is refused on the pad: it rises by 10 leptons.
    let mut refused = None;
    for _ in 0..200 {
        let before = w.obj(a).unwrap().beh.last;
        step(&mut w);
        let o = w.obj(a).unwrap();
        let s = st(&w, a);
        if o.beh.last != before && o.beh.last_common_effects == Some(w.tick - 1) {
            panic!("fell back before the touchdown: {}", describe(&w, a));
        }
        if !s.landing && s.taking_off && (1..=10).contains(&air::height(&w, a)) {
            refused = Some(w.tick - 1);
            break;
        }
        assert!(s.landing, "{}", describe(&w, a));
    }
    let refused = refused.unwrap_or_else(|| panic!("no refusal: {}", describe(&w, a)));
    let o = w.obj(a).unwrap();
    assert!(
        in_foundation(&w, k, o.pos.cell()),
        "refused over the pad: {}",
        describe(&w, a)
    );
    assert!(
        !w.cells.list(o.pos.cell(), Layer::Ground).contains(&a),
        "not listed in its cell layer"
    );
    assert!(flying::tracked(&w, a), "still tracked");
    assert_eq!(
        o.beh.current,
        Some(Behaviour::Enter),
        "no fall-back at the touchdown"
    );
    assert_ne!(o.beh.last_common_effects, Some(refused));
    let (t2, d2) = next_processing(&mut w, a);
    assert_eq!(t2, refused + 1, "{}", describe(&w, a));
    assert_eq!(d2, 1);
    assert_eq!(
        w.obj(a).unwrap().beh.last_common_effects,
        Some(t2),
        "the first processing that finds it no longer landing falls back: {}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 24 (r103: a processing without a contact that finds the aircraft landing also moves it on to the docking stage, but does not move the aircraft itself; it then waits or falls back as above, so a waiting aircraft is never moved toward the pad: its wait has ended by the end of that tick)
#[test]
fn claim_24_a_processing_without_a_contact_that_finds_it_landing_moves_it_on_to_the_docking_stage()
{
    let (mut w, k, a) = approaching(&rules());
    // Until the aircraft is landing over the pad with a processing due that
    // has not yet moved it on to the docking stage.
    let mut found = false;
    for _ in 0..2000 {
        let s = st(&w, a);
        if s.landing && !s.enter_docking && w.obj(a).unwrap().beh.due(w.tick) {
            found = true;
            break;
        }
        step(&mut w);
    }
    assert!(found, "{}", describe(&w, a));
    let (t, d) = at_next_processing(&mut w, a, |w| {
        end_contact(w, a);
        w.obj_mut(a).unwrap().beh.queue(Some(Behaviour::AreaGuard));
    });
    assert_eq!(d, 1, "{}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(o.contact, None);
    assert!(w.obj(k).is_some());
    assert!(st(&w, a).enter_docking, "moved on to the docking stage");
    assert_ne!(o.beh.last_common_effects, Some(t), "a wait");
    assert_eq!(
        o.beh.current,
        Some(Behaviour::AreaGuard),
        "the wait ended in that tick"
    );
}

// ---------------------------------------------------------------------------
// Claim 31: the processing-cleared ready mark

// spec: S-0501 claim 24 (r103: the processing that moves an aircraft in contact on to the docking stage leaves it not ready to change until its next processing, 3 ticks later; each processing of the docking stage leaves it ready to change, so a behaviour queued meanwhile becomes current at the end of that processing's tick, which ends enter) and claim 31 (an aircraft is not ready when its processing has cleared its ready mark)
#[test]
fn claim_31_the_move_on_to_the_docking_stage_clears_the_ready_mark_until_the_next_processing() {
    let (mut w, _k, a) = approaching(&rules());
    let t = to_docking_stage(&mut w, a);
    assert_eq!(w.tick, t + 1);
    assert!(
        !process::ready(&w, a),
        "not ready after the move on: {}",
        describe(&w, a)
    );
    w.obj_mut(a).unwrap().beh.queue(Some(Behaviour::AreaGuard));
    // t + 1 and t + 2: no processing, the queued behaviour waits.
    for _ in 0..2 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        assert_eq!(o.beh.current, Some(Behaviour::Enter), "{}", describe(&w, a));
        assert_eq!(o.beh.queued, Some(Behaviour::AreaGuard));
    }
    // t + 3: the docking stage's processing leaves it ready.
    let (t3, d3) = next_processing(&mut w, a);
    assert_eq!((t3, d3), (t + 3, 1), "{}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.beh.current,
        Some(Behaviour::AreaGuard),
        "{}",
        describe(&w, a)
    );
    assert_eq!(o.beh.queued, None);
}

// spec: S-0501 claim 24 (r103: each processing of the docking stage leaves the aircraft ready to change, so a behaviour queued meanwhile becomes current at the end of the next tick's update, claim 31's third point)
#[test]
fn claim_31_a_docking_stage_processing_leaves_the_aircraft_ready() {
    let (mut w, _k, a) = approaching(&rules());
    let t = to_docking_stage(&mut w, a);
    let (t3, d3) = next_processing(&mut w, a);
    assert_eq!((t3, d3), (t + 3, 1), "{}", describe(&w, a));
    assert_eq!(w.obj(a).unwrap().beh.current, Some(Behaviour::Enter));
    assert!(process::ready(&w, a), "{}", describe(&w, a));
}

// ---------------------------------------------------------------------------
// The pad-bound rule before every other rule of claim 24

/// A pad-bound BOUND of house 0 on the ground on open ground 8 cells from
/// the pad *P*, in contact with it, *P* its destination and *enter* current
/// (just started, so this is its first processing), *guard* queued, the
/// destination already given to its flight. Returns (world, P, A).
fn bound_away(r: &Ruleset) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let k = building(&mut w, "APAD", 0, 0, 0);
    ready(&mut w);
    let a = put(&mut w, "BOUND", 8, 0);
    step(&mut w);
    assert!(air::on_ground(&w, a), "{}", describe(&w, a));
    let t = w.tick;
    {
        let o = w.obj_mut(a).unwrap();
        o.contact = Some(k);
        o.econ.exit = Some(Exit {
            factory: k,
            tethered: false,
            place: 0,
        });
        o.dest = Some(Target::Object(k));
        o.beh.force(Behaviour::Enter, t, true);
        o.beh.queue(Some(Behaviour::Guard));
        let s = o.air.as_mut().unwrap();
        s.flying_to = Some(Target::Object(k));
        s.heading_pad = None;
    }
    (w, k, a)
}

// spec: S-0501 claim 24 (r103, Q-2366: one rule comes before every other rule of this claim, at every processing of enter, its first included: a pad-bound aircraft on the ground by its height with a destination other than the building in its own cell leaves any contact it holds and records that destination as the pad it was last heading for, then queues move with an immediate start, which replaces any queued behaviour; that processing returns 1 tick and does nothing else; once move is current its first processing chooses its destination by claim 40: F7 refuses the pad's own cell, since the pad stands in it and the aircraft holds no contact, so claim 40 seeks a replacement cell and the aircraft takes off and flies to it; over that cell it falls back to rest instead of landing, claim 6, and that fall-back's search asks the pad it was last heading for first, claims 18 and 21)
#[test]
fn claim_24_a_pad_bound_aircraft_on_the_ground_away_from_its_pad_queues_move() {
    let (mut w, k, a) = bound_away(&rules());
    let (t, d) = next_processing(&mut w, a);
    assert_eq!(d, 1, "{}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(
        o.contact,
        None,
        "it leaves its contact: {}",
        describe(&w, a)
    );
    assert_eq!(
        st(&w, a).heading_pad,
        Some(k),
        "the pad it was last heading for"
    );
    assert_eq!(o.beh.current, Some(Behaviour::Move), "{}", describe(&w, a));
    assert_eq!(o.beh.queued, None, "move replaced the queued guard");
    assert_eq!(o.beh.start_tick, t, "started in that processing");
    assert_ne!(o.beh.last_common_effects, Some(t), "no fall-back");
    assert_eq!(
        o.dest,
        Some(Target::Object(k)),
        "nothing else: {}",
        describe(&w, a)
    );
    // The move's first processing: claim 40's replacement cell.
    let (t2, _) = next_processing(&mut w, a);
    assert_eq!(t2, t + 1);
    let o = w.obj(a).unwrap();
    let Some(Target::Cell(c)) = o.dest else {
        panic!("a replacement cell: {}", describe(&w, a))
    };
    assert!(!in_foundation(&w, k, c), "outside the pad: {c:?}");
    // Over that cell it falls back instead of landing; the fall-back's
    // search gives it P back with a contact, and it docks.
    let mut fell = false;
    for _ in 0..3000 {
        step(&mut w);
        let o = w.obj(a).unwrap();
        if !fell {
            assert!(
                !st(&w, a).landing || o.contact == Some(k),
                "no landing off the pad: {}",
                describe(&w, a)
            );
        }
        if !fell && o.beh.current == Some(Behaviour::Enter) && o.contact == Some(k) {
            fell = true;
            assert_eq!(o.dest, Some(Target::Object(k)), "{}", describe(&w, a));
        }
        if w.obj(k)
            .unwrap()
            .air
            .as_ref()
            .and_then(|s| s.service)
            .is_some()
        {
            break;
        }
    }
    assert!(fell, "{}", describe(&w, a));
    let o = w.obj(a).unwrap();
    assert_eq!(o.contact, Some(k), "docked: {}", describe(&w, a));
    assert!(
        air::on_ground(&w, a) && in_foundation(&w, k, o.pos.cell()),
        "{}",
        describe(&w, a)
    );
}

// spec: S-0501 claim 24 (r103: the pad-bound rule applies to an aircraft on the ground by its height only; one in its destination pad's cell, or with no destination, takes the other rules of the claim: here the first processing's 1-tick entry delay)
#[test]
fn claim_24_the_pad_bound_rule_needs_the_ground_and_a_destination_elsewhere() {
    for case in 0..2 {
        let (mut w, _k, a) = bound_away(&rules());
        // Nothing queued, so that enter stays current after the entry delay.
        w.obj_mut(a).unwrap().beh.queued = None;
        if case == 0 {
            w.obj_mut(a).unwrap().dest = None;
        } else {
            // In flight.
            let o = w.obj_mut(a).unwrap();
            o.pos.z += 300;
            o.air.as_mut().unwrap().ground = false;
        }
        let (_, d) = next_processing(&mut w, a);
        assert_eq!(d, 1, "case {case}: {}", describe(&w, a));
        let o = w.obj(a).unwrap();
        assert_eq!(
            o.beh.current,
            Some(Behaviour::Enter),
            "case {case}: {}",
            describe(&w, a)
        );
        assert!(o.contact.is_some(), "case {case}: the contact kept");
    }
}

// spec: S-0501 claim 24 (r103: when every radius fails claim 40 keeps the pad itself as destination, so the aircraft flies toward the pad's heading point with no contact; for an aircraft without a contact that point is the docking position of its one docking place, height included; if that point lies more than 120 leptons above the terrain there, the aircraft is marked as holding height, claim 6: neither landing trigger applies, its move never reports stopped, claim 40, and it stays above the point without the fall-back; otherwise over the point's cell it falls back to rest instead of landing, claim 6, and the fall-back's search asks the pad it was last heading for first)
#[test]
fn claim_24_a_pad_kept_as_move_destination_without_a_contact_holds_above_a_high_heading_point() {
    for high in [true, false] {
        let z = if high { 200 } else { 0 };
        let (mut w, k, a) = bound_away(&rules_with(1, Some(vec![[0, 0, z]])));
        next_processing(&mut w, a);
        next_processing(&mut w, a);
        assert!(matches!(w.obj(a).unwrap().dest, Some(Target::Cell(_))));
        // As if every radius had failed: the pad itself is kept.
        w.obj_mut(a).unwrap().dest = Some(Target::Object(k));
        let c = w.reference_point(k);
        let p = Pos::new(c.x, c.y, c.z + z);
        assert_eq!(
            air::flight_point(&w, a, Target::Object(k)),
            p,
            "the heading point"
        );
        let falls = w.obj(a).unwrap().beh.last_common_effects;
        let mut fell = false;
        for _ in 0..1500 {
            step(&mut w);
            let o = w.obj(a).unwrap();
            if o.beh.last_common_effects != falls {
                fell = true;
                break;
            }
            assert!(
                !st(&w, a).landing,
                "high {high}: no landing: {}",
                describe(&w, a)
            );
        }
        let o = w.obj(a).unwrap();
        if high {
            assert!(!fell, "no fall-back: {}", describe(&w, a));
            assert!(st(&w, a).holding, "holding height: {}", describe(&w, a));
            assert!(!air::stopped(&w, a), "move never reports stopped");
            assert_eq!(o.beh.current, Some(Behaviour::Move));
            assert_eq!(o.dest, Some(Target::Object(k)));
            let d = (o.pos.x - p.x).abs().max((o.pos.y - p.y).abs());
            assert!(d < 128, "above the point: {}", describe(&w, a));
            assert_eq!(
                air::height(&w, a),
                z,
                "at the point's height: {}",
                describe(&w, a)
            );
        } else {
            assert!(
                fell,
                "a fall-back over the point's cell: {}",
                describe(&w, a)
            );
            assert_eq!(o.contact, Some(k), "the pad last heading for, asked first");
            assert_eq!(o.beh.current, Some(Behaviour::Enter));
        }
    }
}

// ---------------------------------------------------------------------------
// The heading point of a pad for an aircraft without a contact

// spec: S-0501 claim 24 (r103: for an aircraft without a contact, the pad's heading point is the docking position of its one docking place, height included, or its foundation centre when it has several places or none, S-0404 claim 15 and S-0506 claim 14) and claim 40 (pad guidance replaces a building point with the contacting aircraft's docking position, its horizontal location only)
#[test]
fn claim_24_the_heading_point_of_a_pad_without_a_contact() {
    let offsets = Some(vec![[40, -60, 150], [-40, 60, 90]]);
    for docks in [1, 2] {
        let r = rules_with(docks, offsets.clone());
        let mut w = game(&r);
        let k = building(&mut w, "APAD", 0, 0, 0);
        ready(&mut w);
        let a = put(&mut w, "JET", 8, 0);
        step(&mut w);
        let c = w.reference_point(k);
        let want = if docks == 1 {
            Pos::new(c.x + 40, c.y - 60, c.z + 150)
        } else {
            c
        };
        assert_eq!(
            air::flight_point(&w, a, Target::Object(k)),
            want,
            "docks {docks}"
        );
        assert!(air::request_contact(&mut w, a, k));
        let q = glasnost_sim::econ::release::pad_position(&w, k, a).0;
        assert_eq!(
            air::flight_point(&w, a, Target::Object(k)),
            Pos::new(q.x, q.y, c.z),
            "docks {docks}: in contact, the docking position's horizontal location"
        );
    }
}
