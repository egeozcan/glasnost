//! A docking building whose build-up has not finished refuses an entrant,
//! also after an owner change took its queued *construction* away (#347):
//! the infantry absorber (S-0605 claim 1), the recycler (S-0603 claim 2),
//! the repair depot (S-0314 claim 16) and the reload pad (S-0501 claim 22),
//! as the bunker does (S-0606 claim 2 row P3). On synthetic data: invented
//! types and numbers, no game data.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, Locomotor, UnitType};
use glasnost_sim::econ::placement;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::ownership::{self, Cause};
use glasnost_sim::structures::{absorb, grinder};
use glasnost_sim::transport::Answer;
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{air, depot, objects, testkit, tick};

const SIZE: i32 = 30;
const A: HouseId = HouseId(0);
const B: HouseId = HouseId(1);

fn add(r: &mut Ruleset, name: &str, f: impl FnOnce(&mut BuildingType)) {
    let mut t = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        ..Default::default()
    };
    t.econ.buildup = Some(true);
    t.econ.buildup_frames = Some(20);
    f(&mut t);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push(name.into());
    r.sim.buildings.insert(name.into(), t);
}

/// The combat ruleset with BIO (an infantry absorber), GRIND (a
/// recycler), DEPOT (a repair depot), PAD (a reload pad with one docking
/// place) and FLYER (an aircraft that docks at PAD).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    add(&mut r, "BIO", |t| {
        t.infantry_absorb = Some(true);
        t.passengers = Some(3);
        t.size_limit = Some(Decimal("1".into()));
    });
    add(&mut r, "GRIND", |t| t.grinding = Some(true));
    add(&mut r, "DEPOT", |t| {
        t.foundation = Some([3, 3]);
        t.unit_repair = Some(true);
        t.number_impassable_rows = Some(0);
    });
    add(&mut r, "PAD", |t| {
        t.econ.unit_reload = Some(true);
        t.econ.number_of_docks = Some(1);
    });
    let mut u = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        ..Default::default()
    };
    u.econ.landable = Some(true);
    u.econ.dock = Some(vec!["PAD".into()]);
    r.sim.aircraft.insert("FLYER".into(), u);
    r.registries
        .entry("aircraft".into())
        .or_default()
        .push("FLYER".into());
    r
}

/// A building of Alpha's placed by order: it enters with *construction*
/// queued (S-0406 claim 17).
fn placed(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    let t = w.rules().index_of(Kind::Building, name).unwrap();
    let id = objects::create(w, Kind::Building, t, Some(A));
    placement::put_on_map(w, id, testkit::mid(SIZE, dx, dy), true);
    id
}

fn state(w: &World, b: ObjectId) -> (Option<Behaviour>, Option<Behaviour>, bool) {
    let o = w.obj(b).unwrap();
    (o.beh.current, o.beh.queued, o.econ.construction.building_up)
}

struct Scene {
    w: World,
    bio: ObjectId,
    grind: ObjectId,
    depot: ObjectId,
    pad: ObjectId,
    foot: ObjectId,
    car: ObjectId,
    fly: ObjectId,
}

/// The four buildings placed by Alpha and handed to Beta by a trigger
/// during their first build-up run, with an entrant of Beta's for each.
fn handed_over() -> Scene {
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    testkit::run(&mut w, 2);
    let bio = placed(&mut w, "BIO", -8, -6);
    let grind = placed(&mut w, "GRIND", 6, -6);
    let depot = placed(&mut w, "DEPOT", -8, 4);
    let pad = placed(&mut w, "PAD", 6, 4);
    let foot = testkit::spawn(
        &mut w,
        Kind::Infantry,
        "WALKER",
        1,
        testkit::mid(SIZE, -8, -2),
        0,
    );
    let car = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let fly = testkit::spawn(
        &mut w,
        Kind::Aircraft,
        "FLYER",
        1,
        testkit::mid(SIZE, 2, 9),
        0,
    );
    tick::step(&mut w);
    for b in [bio, grind, depot, pad] {
        assert_eq!(
            state(&w, b),
            (None, Some(Behaviour::Construction), true),
            "set-up: construction queued, build-up running"
        );
    }
    for b in [bio, grind, depot, pad] {
        ownership::change(&mut w, b, Some(B), Cause::Trigger);
    }
    for b in [bio, grind, depot, pad] {
        let (cur, queued, up) = state(&w, b);
        assert!(
            up && !matches!(
                cur.or(queued),
                Some(Behaviour::Construction | Behaviour::Selling)
            ),
            "set-up: guard replaced construction, the build-up state lasts ({cur:?}, {queued:?}, {up})"
        );
    }
    Scene {
        w,
        bio,
        grind,
        depot,
        pad,
        foot,
        car,
        fly,
    }
}

fn answers(s: &Scene) -> (Answer, Answer, bool, bool) {
    (
        absorb::answer(&s.w, s.bio, s.foot),
        grinder::answer(&s.w, s.grind, s.car),
        depot::accepts(&s.w, s.depot, s.car, true),
        air::accepts(&s.w, s.pad, s.fly),
    )
}

// spec: S-0605 claim 1, S-0603 claim 2, S-0314 claim 16 and S-0501 claim 22 (a building that has not finished its build-up refuses); S-0309 claim 31 (an owner change replaces the queued construction with guard)
#[test]
fn issue_347_unfinished_build_up_after_an_owner_change_refuses() {
    let s = handed_over();
    assert_eq!(
        answers(&s),
        (Answer::Refuses, Answer::Refuses, false, false),
        "(absorber, recycler, depot, pad) while the build-up is unfinished"
    );
}

// spec: S-0605 claim 1, S-0603 claim 2, S-0314 claim 16 and S-0501 claim 22 (the build-up ends at the building's next turn, after which each accepts)
#[test]
fn issue_347_control_the_next_turn_ends_the_build_up_and_each_accepts() {
    let mut s = handed_over();
    tick::step(&mut s.w);
    for b in [s.bio, s.grind, s.depot, s.pad] {
        assert!(!state(&s.w, b).2, "the build-up state has ended");
    }
    assert_eq!(
        answers(&s),
        (Answer::Accepts, Answer::Accepts, true, true),
        "(absorber, recycler, depot, pad) after the build-up"
    );
}
