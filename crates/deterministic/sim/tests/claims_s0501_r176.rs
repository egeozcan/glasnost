//! Claim tests for S-0501 r176 (Q-2866 and Q-2867 answered), M8 intake
//! 119: the pad last heading for (claim 21: every search records its
//! result, a contact records nothing, claim 24's in-flight record at enter,
//! a factory release at a pad, a destroyed or removed pad) and the moment
//! with no destination in T2's in-flight pad row (claim 18), with
//! `quirks.unarmed_pad_bound_hover` in both positions. Synthetic data
//! through the real tick path; every type, weapon, map and number is
//! invented. Ledger: `docs/m8-claims/intake119.md`.

mod econ_common;

use econ_common::{SIZE, building, factory, game, issue, ready};
use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Locomotor, UnitType, WeaponType};
use glasnost_sim::air;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::econ::release::{self, Outcome};
use glasnost_sim::objects;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// The ruleset: the pad APAD (an aircraft factory, helipad and reload pad
/// with one docking place at the foundation centre); JET (armed, landable,
/// not pad-bound, 2 rounds, docking at APAD); BOUND (JET, pad-bound); LIFT
/// (unarmed, landable, pad-bound, docking at APAD). econ_common's HUT is a
/// building of no dock type.
fn rules() -> Ruleset {
    let mut r = econ_common::ruleset();
    let c = testkit::combat_ruleset();
    r.sim.weapons.extend(c.sim.weapons.clone());
    r.sim.projectiles.extend(c.sim.projectiles.clone());
    r.sim.warheads.extend(c.sim.warheads.clone());
    push(&mut r, "buildings", "APAD");
    let mut pad = factory("aircraft");
    pad.helipad = Some(true);
    pad.econ.unit_reload = Some(true);
    pad.econ.number_of_docks = Some(1);
    pad.econ.docking_offsets = Some(vec![[0, 0, 0]]);
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
    let plane = |armed: bool, bound: bool| {
        let mut u = UnitType {
            strength: Some(200),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            tech_level: Some(1),
            owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
            cost: Some(1000),
            ..Default::default()
        };
        if armed {
            u.primary = Some("Agm".into());
            u.combat.ammo = Some(2);
        }
        u.econ.dock = Some(vec!["APAD".into()]);
        u.econ.landable = Some(true);
        u.econ.airport_bound = Some(bound);
        u
    };
    for (n, u) in [
        ("JET", plane(true, false)),
        ("BOUND", plane(true, true)),
        ("LIFT", plane(false, true)),
    ] {
        push(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    r
}

fn step(w: &mut World) {
    glasnost_sim::tick::step(w);
}

fn st(w: &World, id: ObjectId) -> air::AirState {
    w.obj(id).unwrap().air.clone().unwrap_or_default()
}

fn heading(w: &World, id: ObjectId) -> Option<ObjectId> {
    st(w, id).heading_pad
}

fn set_heading(w: &mut World, id: ObjectId, k: Option<ObjectId>) {
    w.obj_mut(id).unwrap().air.as_mut().unwrap().heading_pad = k;
}

fn put(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, Kind::Aircraft, name, 0, testkit::mid(SIZE, dx, dy), 0)
}

fn describe(w: &World, id: ObjectId) -> String {
    let Some(o) = w.obj(id) else {
        return format!("t={} gone", w.tick);
    };
    format!(
        "t={} pos={:?} h={} hp={} ammo={} beh={:?}/{:?} dest={:?} target={:?} contact={:?} air={:?}",
        w.tick,
        o.pos,
        air::height(w, id),
        o.health,
        o.combat.ammo,
        o.beh.current,
        o.beh.queued,
        o.dest,
        o.target,
        o.contact,
        o.air,
    )
}

/// A `name` aircraft of house 0 flying east at full speed in flight over
/// open ground, toward a cell 12 cells east of where it started; with
/// `pad`, an APAD of house 0 far north of its line. Returns (world,
/// aircraft, pad).
fn cruising(r: &Ruleset, name: &str, pad: bool) -> (World, ObjectId, Option<ObjectId>) {
    let mut w = game(r);
    let k = pad.then(|| building(&mut w, "APAD", 0, 0, -14));
    ready(&mut w);
    let a = put(&mut w, name, -8, 4);
    step(&mut w);
    let c = w.obj(a).unwrap().pos.cell();
    assert!(issue(
        &mut w,
        0,
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(CellXY::new(c.x + 12, c.y))),
        }
    ));
    let one = F64::from_i64(1, w.rules().mode);
    for _ in 0..600 {
        step(&mut w);
        let s = st(&w, a);
        let h = air::height(&w, a);
        if h > 0 && !s.taking_off && !s.landing && h == s.held && s.cur.eq_num(one) {
            return (w, a, k);
        }
    }
    panic!("never at full speed: {}", describe(&w, a));
}

/// Makes `beh` current for aircraft `a` with nothing queued, and `dest` its
/// destination.
fn set_beh(w: &mut World, a: ObjectId, beh: Behaviour, dest: Option<Target>) {
    let t = w.tick;
    let o = w.obj_mut(a).unwrap();
    o.beh.force(beh, t, true);
    o.beh.queued = None;
    o.dest = dest;
}

/// A second aircraft whose destination is the cell `a` is over: the
/// reservation test refuses that cell (claim 39), so claim 6's selection
/// around it takes claim 40's direction draws (Q-2507).
fn reserve_own_cell(w: &mut World, a: ObjectId) {
    let here = w.obj(a).unwrap().pos.cell();
    let b = put(w, "JET", -8, 12);
    w.obj_mut(b).unwrap().dest = Some(Target::Cell(here));
}

// ---------------------------------------------------------------------------
// Claim 21: the pad last heading for

// spec: S-0501 claim 21 (r176, Q-2866: every search records its result as the pad the aircraft was last heading for: the pad it chooses, or no pad when it finds none; an aircraft that is not pad-bound records it too; a pad-bound aircraft whose recorded pad no longer accepts it loses that record and the result replaces it)
#[test]
fn claim_21_every_search_records_its_result() {
    for name in ["JET", "BOUND"] {
        let mut w = game(&rules());
        let k = building(&mut w, "APAD", 0, 0, -10);
        // A building of the other house: it does not accept the aircraft
        // (claim 22).
        let hut = building(&mut w, "HUT", 1, 8, 8);
        ready(&mut w);
        let a = put(&mut w, name, 4, -10);
        step(&mut w);
        set_heading(&mut w, a, None);
        assert_eq!(air::pad_search(&mut w, a), Some(k), "{name}");
        assert_eq!(heading(&w, a), Some(k), "{name}: the pad it chooses");
        // A recorded building that does not accept it: the result replaces it.
        set_heading(&mut w, a, Some(hut));
        assert_eq!(air::pad_search(&mut w, a), Some(k), "{name}");
        assert_eq!(heading(&w, a), Some(k), "{name}: replaced");
        // No pad found: no pad recorded.
        w.obj_mut(a).unwrap().owner = Some(HouseId(1));
        assert_eq!(air::pad_search(&mut w, a), None, "{name}: void");
        assert_eq!(heading(&w, a), None, "{name}: none recorded");
        // find_pad itself records nothing.
        w.obj_mut(a).unwrap().owner = Some(HouseId(0));
        assert_eq!(air::find_pad(&w, a), Some(k), "{name}");
        assert_eq!(heading(&w, a), None, "{name}: find_pad is pure");
    }
}

// spec: S-0501 claim 21 (r176, Q-2866: only a pad-bound aircraft asks its recorded pad first, so it keeps that pad while the pad accepts it, even with a nearer free pad of its dock types; one that is not pad-bound chooses the nearer pad, and records that) and AC13's last check
#[test]
fn claim_21_a_pad_bound_aircraft_keeps_its_recorded_pad() {
    for (name, keeps) in [("BOUND", true), ("JET", false)] {
        let mut w = game(&rules());
        let far = building(&mut w, "APAD", 0, 0, -12);
        let near = building(&mut w, "APAD", 0, 0, 10);
        ready(&mut w);
        let a = put(&mut w, name, 4, -12);
        step(&mut w);
        assert_eq!(air::pad_search(&mut w, a), Some(far), "{name}: void");
        // The aircraft is now beside the other pad.
        let p = w.obj(near).unwrap().pos;
        w.obj_mut(a).unwrap().pos = glasnost_sim::world::Pos::new(p.x + 1024, p.y, p.z);
        let want = if keeps { far } else { near };
        assert_eq!(air::pad_search(&mut w, a), Some(want), "{name}");
        assert_eq!(heading(&w, a), Some(want), "{name}: recorded");
    }
}

// spec: S-0501 claim 21 (r176, Q-2866: a contact that a pad grants after the search records nothing) and claim 23
#[test]
fn claim_21_a_contact_records_nothing() {
    for name in ["JET", "BOUND"] {
        let mut w = game(&rules());
        let k = building(&mut w, "APAD", 0, 0, -10);
        ready(&mut w);
        let a = put(&mut w, name, 4, 6);
        step(&mut w);
        set_heading(&mut w, a, None);
        assert!(air::request_contact(&mut w, a, k), "{name}: void");
        assert_eq!(w.obj(a).unwrap().contact, Some(k), "{name}: void");
        assert_eq!(heading(&w, a), None, "{name}: no record");
    }
}

// spec: S-0501 claim 21 (r176, Q-2866: the record changes when the recorded building is destroyed or removed from the game, which leaves no pad recorded)
#[test]
fn claim_21_a_destroyed_or_removed_pad_leaves_no_record() {
    for destroy in [false, true] {
        let mut w = game(&rules());
        let k = building(&mut w, "APAD", 0, 0, -10);
        ready(&mut w);
        let a = put(&mut w, "BOUND", 4, 6);
        step(&mut w);
        assert_eq!(air::pad_search(&mut w, a), Some(k), "void");
        if destroy {
            damage::apply(
                &mut w,
                k,
                Hit {
                    amount: 1_000_000,
                    warhead: None,
                    distance: 0,
                    attacker: None,
                    house: None,
                    ignore_defences: true,
                    no_escape: false,
                },
            );
            assert!(
                w.obj(k).is_none_or(|b| b.health <= 0),
                "void: the pad destroyed"
            );
        } else {
            objects::remove(&mut w, k);
        }
        assert_eq!(heading(&w, a), None, "destroyed {destroy}");
    }
}

// spec: S-0501 claim 21 (r176, Q-2866: a factory release of the aircraft at a pad's docking position, S-0404 claims 13 and 15, records that pad whatever its height there)
#[test]
fn claim_21_a_factory_release_at_a_pad_records_the_pad() {
    for name in ["JET", "BOUND"] {
        let mut w = game(&rules());
        let k = building(&mut w, "APAD", 0, 0, -10);
        ready(&mut w);
        let ti = w.rules().index_of(Kind::Aircraft, name).unwrap();
        let a = objects::create(&mut w, Kind::Aircraft, ti, Some(HouseId(0)));
        assert_eq!(
            release::aircraft(&mut w, k, a, true),
            Outcome::Released,
            "{name}: void"
        );
        assert_eq!(heading(&w, a), Some(k), "{name}: the pad recorded");
    }
}

// spec: S-0501 claim 24 (r176, Q-2866: at every processing of enter, its first included, an aircraft in flight, pad-bound or not, whose destination is a building other than the pad it was last heading for records that building as that pad, of its dock types or not) and claim 21
#[test]
fn claim_24_enter_in_flight_records_its_destination_building() {
    for name in ["JET", "BOUND"] {
        let (mut w, a, _) = cruising(&rules(), name, false);
        let hut = building(&mut w, "HUT", 0, 10, -12);
        set_heading(&mut w, a, None);
        set_beh(&mut w, a, Behaviour::Enter, Some(Target::Object(hut)));
        let mut seen = false;
        for _ in 0..6 {
            step(&mut w);
            if heading(&w, a) == Some(hut) {
                seen = true;
                break;
            }
        }
        assert!(seen, "{name}: the building recorded: {}", describe(&w, a));
    }
}

// ---------------------------------------------------------------------------
// Claim 18: the moment with no destination

/// The behaviours that reach T2's in-flight pad row with nothing else
/// changed, and the destination each holds there.
fn pad_row_cases(w: &World, a: ObjectId) -> Vec<(Behaviour, Option<Target>)> {
    let d = w.obj(a).unwrap().dest;
    vec![
        (Behaviour::Guard, d),
        (Behaviour::Sticky, d),
        (Behaviour::Move, d),
        (Behaviour::Enter, None),
    ]
}

// spec: S-0501 claim 18, The moment with no destination (r176, Q-2867: every fall-back that reaches T2's in-flight pad search gives the aircraft no destination for a moment after that search, whatever its current behaviour, guard, sticky, move and enter with no destination alike, and whether the pad grants the contact; for an aircraft moving and not landing it has claim 6's effects: a new flight point around the cell it is over, with claim 40's direction draws when that cell is refused, Q-2507; the pad given right after it replaces that flight point) and T2 (r176: granted: none, then the pad; enter)
#[test]
fn claim_18_the_in_flight_pad_row_gives_no_destination_for_a_moment() {
    let (w0, a, k) = cruising(&rules(), "JET", true);
    let k = k.unwrap();
    for (beh, dest) in pad_row_cases(&w0, a) {
        for refused in [false, true] {
            let mut w = w0.clone();
            set_beh(&mut w, a, beh, dest);
            if refused {
                reserve_own_cell(&mut w, a);
            }
            let rng = w.rng.clone();
            air::fall_back_rows(&mut w, a);
            let o = w.obj(a).unwrap();
            let d = describe(&w, a);
            assert_eq!(o.dest, Some(Target::Object(k)), "{beh:?}: the pad: {d}");
            assert_eq!(o.contact, Some(k), "{beh:?}: the contact: {d}");
            assert!(
                o.beh.current == Some(Behaviour::Enter) || o.beh.queued == Some(Behaviour::Enter),
                "{beh:?}: enter: {d}"
            );
            assert_eq!(heading(&w, a), Some(k), "{beh:?}: the search's record");
            if refused {
                assert_ne!(w.rng, rng, "{beh:?}: the moment's draws: {d}");
            } else {
                assert_eq!(w.rng, rng, "{beh:?}: no draws: {d}");
            }
        }
    }
}

// spec: S-0501 claim 18 (r176, Q-2867: no other row has the moment, the after-the-row search included; enter current with a destination takes the guard row instead and has no such moment) and T2's after-the-row entries (no ammunition, armed, no contact: the pad as destination, enter)
#[test]
fn claim_18_the_after_the_row_search_has_no_moment() {
    let (mut w, a, k) = cruising(&rules(), "JET", true);
    let k = k.unwrap();
    let c = w.obj(a).unwrap().pos.cell();
    let d = Some(Target::Cell(CellXY::new(c.x + 3, c.y)));
    set_beh(&mut w, a, Behaviour::Enter, d);
    w.obj_mut(a).unwrap().combat.ammo = 0;
    reserve_own_cell(&mut w, a);
    let rng = w.rng.clone();
    air::fall_back_rows(&mut w, a);
    let o = w.obj(a).unwrap();
    let dd = describe(&w, a);
    assert_eq!(o.dest, Some(Target::Object(k)), "the pad: {dd}");
    assert_eq!(heading(&w, a), Some(k), "recorded: {dd}");
    assert_eq!(w.rng, rng, "no moment, so no draws: {dd}");
}

// spec: S-0501 claim 18 (r176, Q-2867: a pad-bound aircraft that crashes is given no destination after the moment, so its crash, claim 46, begins with the flight point and unfinished flight the moment left it: the cell it is over, not its old destination) and T2 (r176: pad-bound with no grant: none, then the crash) and claim 21 (the search finding none records no pad)
#[test]
fn claim_18_a_pad_bound_crash_begins_with_the_moments_flight_point() {
    let (mut w, a, _) = cruising(&rules(), "BOUND", false);
    let c = w.obj(a).unwrap().pos.cell();
    let old = Some(Target::Cell(CellXY::new(c.x + 6, c.y)));
    set_beh(&mut w, a, Behaviour::Guard, old);
    // A recorded building of the other house, which does not accept it.
    let hut = building(&mut w, "HUT", 1, 10, -12);
    set_heading(&mut w, a, Some(hut));
    air::fall_back_rows(&mut w, a);
    let d = describe(&w, a);
    assert!(air::crashing(&w, a), "it crashes: {d}");
    let s = st(&w, a);
    assert_eq!(
        s.flying_to,
        Some(Target::Cell(c)),
        "the moment's point: {d}"
    );
    assert!(s.own_point, "{d}");
    assert_eq!(w.obj(a).unwrap().dest, None, "{d}");
    assert_eq!(s.heading_pad, None, "none recorded: {d}");
}

// spec: S-0501 Edge cases (quirks.unarmed_pad_bound_hover in both positions, r176: off, an unarmed landable pad-bound aircraft with dock types falling back in flight with no contact looks for a pad and asks it as T2's in-flight pad row does, with that row's moment with no destination, claim 18; on, it takes T2's last in-flight row, the landing spot and move, with no search and no moment)
#[test]
fn quirk_unarmed_pad_bound_hover_off_has_the_moment() {
    for hover in [false, true] {
        let mut r = rules();
        r.globals.quirks.unarmed_pad_bound_hover = hover;
        let (w0, a, k) = cruising(&r, "LIFT", true);
        let k = k.unwrap();
        for refused in [false, true] {
            let mut w = w0.clone();
            let d0 = w.obj(a).unwrap().dest;
            set_beh(&mut w, a, Behaviour::Guard, d0);
            set_heading(&mut w, a, None);
            if refused {
                reserve_own_cell(&mut w, a);
            }
            let rng = w.rng.clone();
            air::fall_back_rows(&mut w, a);
            let o = w.obj(a).unwrap();
            let d = describe(&w, a);
            if hover {
                assert_ne!(o.dest, Some(Target::Object(k)), "{hover}: no pad: {d}");
                assert_eq!(o.contact, None, "{hover}: {d}");
                assert_eq!(heading(&w, a), None, "{hover}: no search: {d}");
                assert!(!air::crashing(&w, a), "{hover}: {d}");
                continue;
            }
            assert_eq!(o.dest, Some(Target::Object(k)), "{hover}: the pad: {d}");
            assert_eq!(o.contact, Some(k), "{hover}: {d}");
            assert_eq!(heading(&w, a), Some(k), "{hover}: recorded: {d}");
            if refused {
                assert_ne!(w.rng, rng, "{hover}: the moment's draws: {d}");
            } else {
                assert_eq!(w.rng, rng, "{hover}: no draws: {d}");
            }
        }
    }
}
