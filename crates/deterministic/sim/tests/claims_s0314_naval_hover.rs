//! Claim tests of S-0314 claim 21's *Over water, at a naval yard* and the
//! Edge case *Hovering vehicle lost at a naval yard* (revision 77,
//! `quirks.docked_hover_water_loss`), with S-0503 claims 10 and 11, on a
//! synthetic water map with invented types (every name and number here is
//! made up; no game data).

mod common;

use common::{Scene, cell, code_tile};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BuildingType, Decimal, LandRow, LandType, LocomotionClass, Locomotor, MovementZone, UnitType,
};
use glasnost_sim::econ::power;
use glasnost_sim::hover;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_sim::{depot, save, testkit, tick};

const ALPHA: HouseId = HouseId(0);
/// The terrain code of a water tile in the scene's catalogue (S-0211 Terms).
const WATER_CODE: u8 = 9;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

/// A land row: the float class only on water, the hover and amphibious
/// classes everywhere.
fn land_row(ground: &str, float: &str) -> LandRow {
    LandRow {
        foot: dec(ground),
        track: dec(ground),
        wheel: dec(ground),
        hover: dec("1"),
        float: dec(float),
        amphibious: dec("1"),
        float_beach: dec(float),
        buildable: None,
    }
}

fn owners() -> Option<Vec<String>> {
    Some(vec!["Alphaland".into(), "Betaland".into()])
}

/// The testkit's combat ruleset plus water rows, the hover fields of
/// [General] (invented values, no bob, so that heights are steady), and
/// - SKIMMER: a hovering vehicle of the plain amphibious zone (S-0504
///   claim 7: it may dock at a naval yard), with the powered-unit flag;
/// - NAVYARD: a 2 × 2 naval vehicle factory with the repair flag, its sole
///   docking displacement to the centre of the cell (2, 1) from its
///   top-left;
/// - PLANT (power output) and CENTRE (the control centre for SKIMMER);
/// - repair step 10 at 25 %.
fn rules(loss: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.globals.quirks.docked_hover_water_loss = loss;
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    let s = &mut r.sim;
    s.land_types.insert(LandType::Clear, land_row("1", "0"));
    s.land_types.insert(LandType::Road, land_row("1", "0"));
    s.land_types.insert(LandType::Water, land_row("0", "1"));
    s.general.hover_height = Some(100);
    s.general.hover_bob = dec("0");
    s.general.hover_boost = dec("1.5");
    s.general.hover_acceleration = dec("0.02");
    s.general.hover_brake = dec("0.01");
    s.general.hover_dampen = dec("0.75");
    s.general.repair_step = Some(10);
    s.general.repair_percent = dec("25%");
    let mut skimmer = UnitType {
        strength: Some(600),
        speed: Some(6),
        rot: Some(5),
        locomotion_class: Some(LocomotionClass::Hover),
        locomotor: Some(Locomotor::Hover),
        movement_zone: Some(MovementZone::Amphibious),
        cost: Some(900),
        ..Default::default()
    };
    skimmer.econ.powered_unit = Some(true);
    s.vehicles.insert("SKIMMER".into(), skimmer);
    push(&mut r, "vehicles", "SKIMMER");
    let mut yard = BuildingType {
        strength: Some(500),
        foundation: Some([2, 2]),
        factory: Some("vehicle".into()),
        naval: Some(true),
        unit_repair: Some(true),
        tech_level: Some(1),
        owner: owners(),
        cost: Some(1000),
        ..Default::default()
    };
    yard.econ.docking_offsets = Some(vec![[384, 128, 0]]);
    let small = |power: i32| {
        let mut b = BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        b.econ.power = Some(power);
        b
    };
    let mut centre = small(-10);
    centre.powered = Some(true);
    centre.econ.powers_unit = Some("SKIMMER".into());
    r.sim.buildings.insert("NAVYARD".into(), yard);
    r.sim.buildings.insert("PLANT".into(), small(120));
    r.sim.buildings.insert("CENTRE".into(), centre);
    for n in ["NAVYARD", "PLANT", "CENTRE"] {
        push(&mut r, "buildings", n);
    }
    r
}

/// Whether a cell offset lies in the lake: dx in −12 … 12, dy in −8 … 8.
fn lake(dx: i32, dy: i32) -> bool {
    (-12..=12).contains(&dx) && (-8..=8).contains(&dy)
}

/// A flat clear map with the lake of [`lake`] in water tiles; the power
/// plant and the control centre on land in its north-west corner.
fn world(loss: bool) -> (World, ObjectId) {
    let mut s = Scene::new();
    s.ruleset = rules(loss);
    for dx in -14..=14 {
        for dy in -10..=10 {
            if lake(dx, dy) {
                s.set_cell(cell(dx, dy), code_tile(WATER_CODE), 0);
            }
        }
    }
    let mut w = s.build();
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    testkit::spawn(&mut w, Kind::Building, "PLANT", 0, cell(-16, -16), 0);
    let k = testkit::spawn(&mut w, Kind::Building, "CENTRE", 0, cell(-14, -16), 0);
    (w, k)
}

fn at(w: &mut World, name: &str, dx: i32, dy: i32) -> ObjectId {
    let kind = if w.rules().index_of(Kind::Vehicle, name).is_some() {
        Kind::Vehicle
    } else {
        Kind::Building
    };
    testkit::spawn(w, kind, name, 0, cell(dx, dy), 0)
}

fn run_until(w: &mut World, n: i32, mut f: impl FnMut(&World) -> bool) -> bool {
    for _ in 0..n {
        if f(w) {
            return true;
        }
        tick::step(w);
    }
    f(w)
}

fn spin(w: &World, id: ObjectId) -> i32 {
    w.obj(id).and_then(|o| o.hover.as_ref()).map_or(0, |h| h.spin)
}

fn alive(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.on_map && o.health > 0)
}

fn land_of(w: &World, id: ObjectId) -> u8 {
    let c = w.obj(id).unwrap().pos.cell();
    w.cells.cell(c).unwrap().land
}

fn power_order(w: &mut World, k: ObjectId, on: bool) {
    let o = if on {
        Order::PowerOn { actor: k }
    } else {
        Order::PowerOff { actor: k }
    };
    assert!(orders::issue(w, ALPHA, o));
}

/// A damaged SKIMMER on the lake, the naval yard on the lake (its docking
/// cell a water cell), the SKIMMER sent in by a person's click and run
/// until the yard repairs it (S-0314 claims 6, 16 and 17).
fn docked(loss: bool) -> (World, ObjectId, ObjectId, ObjectId) {
    let (mut w, k) = world(loss);
    let yard = at(&mut w, "NAVYARD", -2, -2);
    let v = at(&mut w, "SKIMMER", -8, 3);
    w.obj_mut(v).unwrap().health = 200;
    common::open_orders(&mut w);
    let r = targeting::click(
        &mut w,
        ALPHA,
        &[v],
        ClickOn::Object(yard),
        Modifiers::default(),
    );
    assert!(!r.orders.is_empty(), "the click gives an enter order");
    for o in r.orders {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    // Whether the ordinary visit, with no disabling or enabling, takes the
    // draws is open in the spec (claim 21); our switch-offs follow the stated
    // rules: the arrival's in *enter*, the later ones in *sleep*, no draws.
    let mut spun = false;
    assert!(
        run_until(&mut w, 900, |w| {
            spun |= spin(w, v) != 0;
            w.obj(v).is_some_and(|o| o.health > 200)
        }),
        "the SKIMMER docks at the yard and is repaired"
    );
    assert!(!spun, "the ordinary visit started no spin");
    assert!(!hover::bomb_mark(&w, v));
    assert_eq!(
        land_of(&w, v),
        glasnost_map::terrain::land::WATER,
        "the docking cell of a yard on water is a water cell"
    );
    (w, k, yard, v)
}

/// The disabling, enabling and second disabling of S-0405 claim 20 at the
/// yard: the first disabling (down, switched off) takes no draws; the
/// enabling switches it on and it rises; the second disabling, while it is
/// above the water, takes S-0503 claim 10's two draws and starts a spin.
/// Returns once the spin has ended.
fn spin_at_the_yard(w: &mut World, k: ObjectId, v: ObjectId) {
    power_order(w, k, false);
    assert!(run_until(w, 30, |w| power::unit_disabled(w, v)));
    power_order(w, k, true);
    assert!(run_until(w, 30, |w| !power::unit_disabled(w, v)));
    assert!(
        run_until(w, 300, |w| hover::hover_height(w, v) >= 60),
        "switched on, it rises"
    );
    let before = w.rng.taken();
    power_order(w, k, false);
    assert!(run_until(w, 30, |w| power::unit_disabled(w, v)));
    assert!(
        w.rng.taken() > before && (spin(w, v) != 0 || hover::bomb_mark(w, v)),
        "the disabling above the water took the draws and started a spin"
    );
    assert!(
        run_until(w, 120, |w| spin(w, v) == 0),
        "the spin ends, on the tick it comes down at the latest"
    );
}

// spec: S-0314 claim 21 (Over water, at a naval yard: a hovering vehicle docked at a naval yard on water whose switch-off takes the draws is given the bomb mark at the spin's end below 104 leptons, falls and is lost, and the yard meets the loss of a vehicle in contact; the default of quirks.docked_hover_water_loss) and Edge cases; S-0503 claims 10 and 11
#[test]
fn claim_21_docked_hover_lost_at_a_naval_yard_by_default() {
    let (mut w, k, yard, v) = docked(true);
    assert!(w.rules().quirks.docked_hover_water_loss, "the default");
    spin_at_the_yard(&mut w, k, v);
    assert!(hover::bomb_mark(&w, v), "the bomb mark at the spin's end");
    assert!(
        run_until(&mut w, 600, |w| !alive(w, v)),
        "it falls and is lost"
    );
    assert!(alive(&w, yard));
    assert!(
        !depot::in_contact(&w, yard),
        "the yard holds no contact once the vehicle is lost"
    );
}

// spec: S-0314 claim 21 and Edge cases (quirks.docked_hover_water_loss off: a spin end that comes while the vehicle has contact with a depot or is settled makes no water test: no bomb mark, no fall); S-0503 claim 11 (changed by S-0314)
#[test]
fn quirk_docked_hover_water_loss_off_spares_the_docked_hover() {
    let (mut w, k, yard, v) = docked(false);
    let draws = |w: &World| w.rng.taken();
    let a = draws(&w);
    spin_at_the_yard(&mut w, k, v);
    assert!(draws(&w) > a, "the same draws are taken with the switch off");
    assert!(
        depot::link(&w, v).is_some() || depot::settled(&w, v),
        "the spin ended in contact or settled"
    );
    assert!(!hover::bomb_mark(&w, v), "no water test, no bomb mark");
    assert!(!glasnost_sim::fall::falling(&w, v), "it does not fall");
    tick::step(&mut w);
    assert!(alive(&w, v));
    assert_eq!(hover::hover_height(&w, v), 0, "it rests on the water");
    // The yard goes on with the vehicle: it is still in contact and, once
    // enabled again, it stays alive at the yard.
    assert_eq!(depot::link(&w, v).map(|l| l.depot), Some(yard));
    power_order(&mut w, k, true);
    glasnost_sim::testkit::run(&mut w, 300);
    assert!(alive(&w, v), "never lost");
    assert!(!hover::bomb_mark(&w, v));
    let h = save::state_hash(&w);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h, "the state survives a round trip");
}

/// A SKIMMER standing still on the open water of the lake, with no depot
/// anywhere near; run until its hover height is steady.
fn on_the_lake(loss: bool) -> (World, ObjectId, ObjectId) {
    let (mut w, k) = world(loss);
    let v = at(&mut w, "SKIMMER", 4, 4);
    glasnost_sim::testkit::run(&mut w, 300);
    assert_eq!(land_of(&w, v), glasnost_map::terrain::land::WATER);
    (w, k, v)
}

/// The SKIMMER switched off by `apply_disabled` (S-0503 claim 10: two
/// draws, a spin) and held 50 leptons above the water until the spin ends.
fn spin_low(w: &mut World, v: ObjectId, during: impl Fn(&mut World, i32)) {
    power::apply_disabled(w, v, true);
    assert_ne!(spin(w, v), 0, "the switch-off started a spin");
    for i in 0..40 {
        during(w, i);
        let p = w.obj(v).unwrap().pos;
        let s = w.surface_at(p.x, p.y);
        let o = w.obj_mut(v).unwrap();
        o.pos.z = s + 50;
        o.hover.as_mut().unwrap().v = glasnost_num::F64::ZERO;
        tick::step(w);
        if spin(w, v) == 0 {
            break;
        }
    }
    assert_eq!(spin(w, v), 0, "the spin has ended");
}

// spec: S-0314 Edge cases (quirks.docked_hover_water_loss off: a spin end while the vehicle is settled, its contact ended, makes no water test); S-0314 Terms (settled can outlast the contact, claim 22)
#[test]
fn quirk_off_settled_without_contact_is_spared() {
    for loss in [true, false] {
        let (mut w, _, v) = on_the_lake(loss);
        w.obj_mut(v).unwrap().depot.settled = true;
        spin_low(&mut w, v, |_, _| {});
        assert!(depot::link(&w, v).is_none());
        assert_eq!(
            hover::bomb_mark(&w, v),
            loss,
            "settled, no contact: loss {loss}"
        );
    }
}

// spec: S-0314 Edge cases (quirks.docked_hover_water_loss off: a spin end while the vehicle has contact with a depot (Terms: contact can exist before it reaches the building), not settled, makes no water test)
#[test]
fn quirk_off_contact_without_settled_is_spared() {
    for loss in [true, false] {
        let (mut w, _, v) = on_the_lake(loss);
        let yard = at(&mut w, "NAVYARD", -6, -6);
        assert!(depot::request(&mut w, v, yard, false), "contact taken");
        assert!(!depot::settled(&w, v));
        spin_low(&mut w, v, |_, _| {});
        assert_eq!(
            depot::link(&w, v).map(|l| l.depot),
            Some(yard),
            "still in contact at the spin's end"
        );
        assert!(!depot::settled(&w, v));
        assert_eq!(
            hover::bomb_mark(&w, v),
            loss,
            "contact, not settled: loss {loss}"
        );
    }
}

// spec: S-0314 Edge cases (quirks.docked_hover_water_loss off: once its contact has ended and it is not settled, a later spin end is tested as usual by S-0503 claim 11; the test belongs to the spin's end, not to its start)
#[test]
fn quirk_off_settled_ended_before_the_spin_end_is_tested() {
    let (mut w, _, v) = on_the_lake(false);
    w.obj_mut(v).unwrap().depot.settled = true;
    spin_low(&mut w, v, |w, i| {
        if i == 2 {
            w.obj_mut(v).unwrap().depot.settled = false;
        }
    });
    assert!(
        hover::bomb_mark(&w, v),
        "neither in contact nor settled at the spin's end: tested as usual"
    );
}

// spec: S-0314 Edge cases (quirks.docked_hover_water_loss off: everywhere else S-0503 claim 11's water test is unchanged); S-0503 claim 11 (a spin ending over water below 104 leptons gives the bomb mark and starts the fall)
#[test]
fn quirk_off_open_water_still_sinks() {
    let (mut w, _, v) = on_the_lake(false);
    spin_low(&mut w, v, |_, _| {});
    assert!(hover::bomb_mark(&w, v), "no contact, not settled: marked");
    assert!(glasnost_sim::fall::falling(&w, v), "and it falls");
}

// spec: S-0503 claims 10 and 11 (a switch-off in sleep takes no draws and starts no spin, so no water test is made in either position of quirks.docked_hover_water_loss: the vehicle rests on the water); S-0314 claim 21 (a switch-off that takes no draws: serviced as at a land depot)
#[test]
fn no_draw_switch_off_makes_no_test_in_either_position() {
    for loss in [true, false] {
        let (mut w, _, v) = on_the_lake(loss);
        let (t, stale) = (w.tick, w.rules().quirks.stale_suspended_revert);
        w.obj_mut(v).unwrap().beh.force(Behaviour::Sleep, t, stale);
        let before = w.rng.taken();
        power::apply_disabled(&mut w, v, true);
        assert_eq!(w.rng.taken(), before, "no draws in sleep");
        assert_eq!(spin(&w, v), 0, "no spin");
        glasnost_sim::testkit::run(&mut w, 300);
        assert_eq!(hover::hover_height(&w, v), 0, "it comes down on the water");
        assert!(!hover::bomb_mark(&w, v), "no test, no mark: loss {loss}");
        assert!(alive(&w, v));
    }
}

// spec: S-0314 Terms (the switch-off of a disabling of S-0405 claim 20 makes the vehicle settled, built for every disabling) and Edge cases (quirks.docked_hover_water_loss off); interim of Q-1876: a powered-unit hover disabled by the loss of its control centre over open water, far from any depot, is settled by the disabling and so, with the switch off, is not tested; with the switch on it is lost as anywhere
#[test]
fn q1876_interim_disabled_far_from_a_depot() {
    for loss in [true, false] {
        let (mut w, k, v) = on_the_lake(loss);
        assert!(hover::hover_height(&w, v) > 0, "it hovers");
        power_order(&mut w, k, false);
        assert!(run_until(&mut w, 30, |w| power::unit_disabled(w, v)));
        assert!(depot::settled(&w, v), "the disabling settles it");
        assert!(depot::link(&w, v).is_none(), "no depot contact");
        assert!(run_until(&mut w, 120, |w| spin(w, v) == 0));
        assert_eq!(hover::bomb_mark(&w, v), loss, "loss {loss}");
    }
}
