//! S-0601 claim 11 with `quirks.mind_control_owner_change_keeps_victims`
//! off (#292): a controller that changes owner releases all its victims
//! first, as claim 10 does, so each release's fate decision (claim 8) reads
//! the house the controller had while it held them. Synthetic rules on the
//! test kit: the warhead MIND, the weapon Ray3, the vehicles CTRL3 (a
//! controller) and BOX; every name and number is made up.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
use glasnost_sim::mind;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, World};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;
const NEUTRAL: u8 = 2;

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![Decimal("1".into()); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Ray3".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(30),
            range: Some(Decimal("10".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    for (name, primary) in [("CTRL3", Some("Ray3")), ("BOX", None)] {
        let mut t = r.sim.vehicles["BUGGY"].clone();
        t.strength = Some(1000);
        t.primary = primary.map(Into::into);
        let l = r.registries.entry("vehicles".into()).or_default();
        if !l.iter().any(|n| n == name) {
            l.push(name.into());
        }
        r.sim.vehicles.insert(name.into(), t);
    }
    let g = &mut r.sim.general;
    // Normal table: always hunt. Low-money table: nothing.
    g.ai_capture_normal = Some(vec![0, 0, 0, 100]);
    g.ai_capture_wounded = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_power = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_money = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_money_mark = Some(1000);
    // spec: S-0601 claim 11 (the switch off: an owner change releases all victims first)
    r.globals.quirks.mind_control_owner_change_keeps_victims = false;
    r
}

fn veh(w: &mut World, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    let c: CellXY = testkit::mid(SIZE, dx, dy);
    testkit::spawn(w, Kind::Vehicle, name, house, c, 0)
}

/// BETA (poor) holds a neutral-house unit; ALPHA (rich) is the house the
/// controller changes to.
fn held() -> (World, ObjectId, ObjectId) {
    let mut w = testkit::game_with_non_players(&rules(), SIZE, 1);
    w.houses[usize::from(BETA)].credits = 0;
    w.houses[usize::from(ALPHA)].credits = 5000;
    let c = veh(&mut w, "CTRL3", BETA, 0, 0);
    let n = veh(&mut w, "BOX", NEUTRAL, 3, 0);
    assert!(mind::take(&mut w, c, n));
    w.obj_mut(n).unwrap().beh.queued = None;
    (w, c, n)
}

// spec: S-0601 claim 11 (switch off: the controller releases all its victims first, as claim 10 does) and claim 8 (money and power belong to the controller's house even on release) (#292)
#[test]
fn switch_off_release_reads_the_house_the_controller_had_before_its_change() {
    // Reference: the release made before the change (claim 10's trigger path), then the change.
    let (mut w, c, n) = held();
    mind::trigger_owner_change(&mut w, c);
    mind::set_owner(&mut w, c, Some(HouseId(ALPHA)));
    assert_eq!(w.obj(n).unwrap().owner, Some(HouseId(NEUTRAL)));
    let first = w.obj(n).unwrap().beh.queued;
    assert_ne!(
        first,
        Some(Behaviour::Hunt),
        "BETA is below the low-money mark: the low-money table, nothing"
    );

    // The switch-off path: an owner change by other means.
    let (mut w, c, n) = held();
    mind::set_owner(&mut w, c, Some(HouseId(ALPHA)));
    assert_eq!(w.obj(n).unwrap().owner, Some(HouseId(NEUTRAL)));
    assert!(mind::controller(&w, c).is_some_and(|r| r.victims.is_empty()));
    assert_eq!(
        w.obj(n).unwrap().beh.queued,
        first,
        "release first: the fate table is chosen from the money of the house the controller had while it held the victim"
    );
}

// spec: S-0601 claim 11 (switch on, the original: a controller that changes owner keeps its victims)
#[test]
fn switch_on_keeps_the_victims() {
    let mut r = rules();
    r.globals.quirks.mind_control_owner_change_keeps_victims = true;
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    let c = veh(&mut w, "CTRL3", BETA, 0, 0);
    let n = veh(&mut w, "BOX", NEUTRAL, 3, 0);
    assert!(mind::take(&mut w, c, n));
    mind::set_owner(&mut w, c, Some(HouseId(ALPHA)));
    assert!(mind::controller(&w, c).is_some_and(|r| r.victims == vec![n]));
}
