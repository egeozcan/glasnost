//! Claim tests of S-0314 claim 21's *Over water, at a naval yard* and the
//! Edge case *Hovering vehicle lost at a naval yard* (revisions 77 and 88,
//! `quirks.docked_hover_water_loss`; Terms, *Settled at a depot* and *Away
//! from a depot*, Q-1876), with S-0503 claims 10 and 11, on a
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
    // BUNKER: a 1 × 1 tank bunker (S-0606), for S-0314 Terms' tank-bunker end.
    let bunker = BuildingType {
        strength: Some(500),
        foundation: Some([1, 1]),
        bunker: Some(true),
        number_impassable_rows: Some(0),
        ..Default::default()
    };
    r.sim.buildings.insert("BUNKER".into(), bunker);
    for n in ["NAVYARD", "PLANT", "CENTRE", "BUNKER"] {
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
    w.obj(id)
        .and_then(|o| o.hover.as_ref())
        .map_or(0, |h| h.spin)
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
    let r = targeting::click(&w, ALPHA, &[v], ClickOn::Object(yard), Modifiers::default());
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

// spec: S-0314 claim 21 and Edge cases (quirks.docked_hover_water_loss off: a spin end that comes while the vehicle has contact with a depot or is settled at a depot makes no water test: no bomb mark, no fall); S-0503 claim 11 (changed by S-0314)
#[test]
fn quirk_docked_hover_water_loss_off_spares_the_docked_hover() {
    let (mut w, k, yard, v) = docked(false);
    let draws = |w: &World| w.rng.taken();
    let a = draws(&w);
    spin_at_the_yard(&mut w, k, v);
    assert!(
        draws(&w) > a,
        "the same draws are taken with the switch off"
    );
    assert!(
        depot::link(&w, v).is_some() || depot::settled_at_depot(&w, v),
        "the spin ended in contact or settled at a depot"
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

// spec: S-0314 Edge cases (quirks.docked_hover_water_loss off: a spin end while the vehicle is settled at a depot, its contact ended, makes no water test); S-0314 Terms (settled at a depot can outlast the contact, claim 22)
#[test]
fn quirk_off_settled_without_contact_is_spared() {
    for loss in [true, false] {
        let (mut w, _, v) = on_the_lake(loss);
        w.obj_mut(v).unwrap().depot.settled = depot::Settled::AtDepot;
        spin_low(&mut w, v, |_, _| {});
        assert!(depot::link(&w, v).is_none());
        assert_eq!(
            hover::bomb_mark(&w, v),
            loss,
            "settled at a depot, no contact: loss {loss}"
        );
    }
}

// spec: S-0314 Terms (Settled at a depot: a vehicle settled away from a depot is settled but not settled at a depot) and Edge cases (quirks.docked_hover_water_loss off: only contact or being settled at a depot spares it; everywhere else S-0503 claim 11's water test is unchanged)
#[test]
fn quirk_off_settled_away_without_contact_is_tested() {
    for loss in [true, false] {
        let (mut w, _, v) = on_the_lake(loss);
        w.obj_mut(v).unwrap().depot.settled = depot::Settled::Away;
        spin_low(&mut w, v, |_, _| {});
        assert!(depot::settled(&w, v) && !depot::settled_at_depot(&w, v));
        assert!(
            hover::bomb_mark(&w, v),
            "settled away, no contact: tested in both positions (loss {loss})"
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

// spec: S-0314 Edge cases (quirks.docked_hover_water_loss off: once its contact has ended and it is not settled at a depot, a later spin end is tested as usual by S-0503 claim 11; the test belongs to the spin's end, not to its start)
#[test]
fn quirk_off_settled_ended_before_the_spin_end_is_tested() {
    let (mut w, _, v) = on_the_lake(false);
    w.obj_mut(v).unwrap().depot.settled = depot::Settled::AtDepot;
    spin_low(&mut w, v, |w, i| {
        if i == 2 {
            w.obj_mut(v).unwrap().depot.settled = depot::Settled::No;
        }
    });
    assert!(
        hover::bomb_mark(&w, v),
        "neither in contact nor settled at a depot at the spin's end: tested as usual"
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

// spec: S-0314 Terms (Away from a depot: a disabling of S-0405 claim 20 makes a vehicle settled wherever it is, and one with no contact in a cell no depot occupies is settled but not settled at a depot) and claim 21 (Over water, at a naval yard: a hovering vehicle disabled over open water away from any depot, for example when its house loses its last control centre, is subject to S-0503 claim 11's water test in both positions of quirks.docked_hover_water_loss; Q-1876 answered); scenario H4 part i run 7 in our engine (two draws at the disabling: S-0503 claim 10's two ranged draws and nothing else on that tick; the bomb mark and the loss at a spin end below 104 leptons)
#[test]
fn run_7_disabled_over_open_water_is_lost_in_both_positions() {
    for loss in [true, false] {
        let (mut w, k, v) = on_the_lake(loss);
        assert!(hover::hover_height(&w, v) > 0, "it hovers");
        power_order(&mut w, k, false);
        // The disabling adds to its tick exactly the raw values of the
        // switch-off's two ranged draws (S-0503 claim 10; a ranged draw may
        // reject a raw value and take another, S-0202 claim 4). The rest of
        // the tick is measured on a copy whose vehicle belongs to the other
        // house, so the centre's loss does not disable it.
        let mut alone = 0;
        for _ in 0..30 {
            if power::unit_disabled(&w, v) {
                break;
            }
            let mut probe = w.clone();
            let p0 = probe.rng.taken();
            power::apply_disabled(&mut probe, v, true);
            alone = probe.rng.taken() - p0;
            let mut rest = w.clone();
            rest.obj_mut(v).unwrap().owner = Some(HouseId(1));
            let r0 = rest.rng.taken();
            tick::step(&mut rest);
            let others = rest.rng.taken() - r0;
            let before = w.rng.taken();
            tick::step(&mut w);
            let taken = w.rng.taken() - before;
            if power::unit_disabled(&w, v) {
                assert_eq!(
                    taken,
                    others + alone,
                    "the disabling adds the switch-off's draws"
                );
            } else {
                assert_eq!(taken, others, "no draws of its own before it");
            }
        }
        assert!(power::unit_disabled(&w, v));
        assert!(alone >= 2, "two ranged draws");
        assert_ne!(
            spin(&w, v),
            0,
            "the disabling above the water started a spin"
        );
        assert!(depot::settled(&w, v), "the disabling settles it");
        assert!(!depot::settled_at_depot(&w, v), "but not at a depot");
        assert!(depot::link(&w, v).is_none(), "no depot contact");
        assert!(run_until(&mut w, 120, |w| spin(w, v) == 0));
        assert!(
            hover::bomb_mark(&w, v),
            "marked in both positions: loss {loss}"
        );
        assert!(
            run_until(&mut w, 600, |w| !alive(w, v)),
            "it falls and is lost: loss {loss}"
        );
    }
}

// spec: S-0314 Terms (Settled at a depot: a disabling while a depot occupied its cell makes it settled at a depot, contact or not) and Edge cases (quirks.docked_hover_water_loss off: no water test while settled at a depot; on, the loss)
#[test]
fn disabling_in_a_yard_cell_without_contact_is_settled_at_a_depot() {
    for loss in [true, false] {
        let (mut w, k, v) = on_the_lake(loss);
        // The yard's top-left cell is the vehicle's cell.
        let _yard = at(&mut w, "NAVYARD", 4, 4);
        assert!(depot::link(&w, v).is_none());
        power_order(&mut w, k, false);
        assert!(run_until(&mut w, 30, |w| power::unit_disabled(w, v)));
        assert!(depot::settled_at_depot(&w, v), "a depot occupies its cell");
        assert!(run_until(&mut w, 120, |w| spin(w, v) == 0));
        assert_eq!(hover::bomb_mark(&w, v), loss, "loss {loss}");
    }
}

// spec: S-0314 Terms (Settled at a depot: a disabling while it had contact with a depot makes it settled at a depot, however far away it is) and Edge cases (quirks.docked_hover_water_loss off: spared while in contact)
#[test]
fn disabling_in_contact_away_from_the_yard_is_settled_at_a_depot() {
    for loss in [true, false] {
        let (mut w, k, v) = on_the_lake(loss);
        let yard = at(&mut w, "NAVYARD", -6, -6);
        assert!(depot::request(&mut w, v, yard, false), "contact taken");
        power_order(&mut w, k, false);
        assert!(run_until(&mut w, 30, |w| power::unit_disabled(w, v)));
        assert!(
            depot::settled_at_depot(&w, v),
            "in contact at the disabling"
        );
        assert!(run_until(&mut w, 120, |w| spin(w, v) == 0));
        assert_eq!(hover::bomb_mark(&w, v), loss, "loss {loss}");
    }
}

// spec: S-0314 Terms (Settled at a depot: the event that last made it settled decides; a disabling anywhere else makes it settled but not settled at a depot, so one settled at a depot earlier, with no contact, becomes settled away) and Edge cases (quirks.docked_hover_water_loss off: then tested)
#[test]
fn settled_at_a_depot_then_disabled_away_is_tested() {
    let (mut w, k, v) = on_the_lake(false);
    w.obj_mut(v).unwrap().depot.settled = depot::Settled::AtDepot;
    power_order(&mut w, k, false);
    assert!(run_until(&mut w, 30, |w| power::unit_disabled(w, v)));
    assert!(depot::settled(&w, v) && !depot::settled_at_depot(&w, v));
    assert!(run_until(&mut w, 120, |w| spin(w, v) == 0));
    assert!(hover::bomb_mark(&w, v), "settled away at the spin's end");
}

// spec: S-0314 Terms (Settled away, then at a depot: a depot occupying its cell changes nothing by itself, so a vehicle settled away that stands in a yard's cell with no contact is still not settled at a depot) and claim 21 (Over water, at a naval yard: with quirks.docked_hover_water_loss off, its spin end is tested as in open water)
#[test]
fn settled_away_then_in_a_yard_cell_is_still_tested() {
    let (mut w, k, v) = on_the_lake(false);
    power_order(&mut w, k, false);
    assert!(run_until(&mut w, 30, |w| power::unit_disabled(w, v)));
    assert_ne!(
        spin(&w, v),
        0,
        "the disabling above the water started a spin"
    );
    assert!(!depot::settled_at_depot(&w, v));
    // The yard comes to stand in the vehicle's cell while the spin runs.
    let _yard = at(&mut w, "NAVYARD", 4, 4);
    tick::step(&mut w);
    assert!(
        depot::settled(&w, v) && !depot::settled_at_depot(&w, v),
        "the depot's cell changes nothing by itself"
    );
    assert!(run_until(&mut w, 120, |w| spin(w, v) == 0));
    assert!(depot::link(&w, v).is_none());
    assert!(hover::bomb_mark(&w, v), "no contact, settled away: tested");
}

// spec: S-0314 Edge cases (quirks.docked_hover_water_loss off: once the vehicle is no longer settled and has no contact, a later spin end is tested as usual by S-0503 claim 11); S-0405 claim 20 and S-0314 Terms (the enabling ends being settled); S-0503 claim 10 (switched on, an active spin is not cancelled) and claim 11 (the test belongs to the spin's end)
#[test]
fn quirk_off_a_spin_outliving_the_disabling_is_tested() {
    let (mut w, k, v) = on_the_lake(false);
    power_order(&mut w, k, false);
    assert!(run_until(&mut w, 30, |w| power::unit_disabled(w, v)));
    assert_ne!(
        spin(&w, v),
        0,
        "the disabling above the water started a spin"
    );
    assert!(depot::settled(&w, v));
    power_order(&mut w, k, true);
    assert!(run_until(&mut w, 30, |w| !power::unit_disabled(w, v)));
    assert_ne!(spin(&w, v), 0, "the spin outlives the disabling");
    assert!(!depot::settled(&w, v), "the enabling ends being settled");
    assert!(run_until(&mut w, 120, |w| spin(w, v) == 0));
    assert!(
        hover::bomb_mark(&w, v),
        "neither settled nor in contact at the spin's end: tested and marked"
    );
}

// spec: S-0314 Terms (Away from a depot: once a vehicle settled away stands in a cell that a tank bunker occupies, a destination that takes effect there while its locomotor counts as switched off ends its being settled, and a hovering one is switched on again at that end although it is still disabled, claim 21; What ends it: one received while it is still above the ground ends nothing, wherever it is)
#[test]
fn settled_away_in_a_tank_bunker_cell() {
    let (mut w, k) = world(true);
    // Land south of the lake: the bunker, and the SKIMMER standing in its cell.
    let bunker = at(&mut w, "BUNKER", 0, 9);
    let v = at(&mut w, "SKIMMER", 0, 9);
    glasnost_sim::testkit::run(&mut w, 300);
    assert_eq!(
        w.obj(v).unwrap().pos.cell(),
        w.obj(bunker).unwrap().pos.cell()
    );
    assert!(hover::hover_height(&w, v) > 0, "it hovers");
    power_order(&mut w, k, false);
    assert!(run_until(&mut w, 30, |w| power::unit_disabled(w, v)));
    assert!(depot::settled(&w, v) && !depot::settled_at_depot(&w, v));
    let to = Some(glasnost_sim::world::Target::Cell(cell(3, 9)));
    if hover::hover_height(&w, v) > 0 {
        glasnost_sim::econ::assign_destination(&mut w, v, to);
        glasnost_sim::econ::assign_destination(&mut w, v, None);
        assert!(
            depot::settled(&w, v) && hover::switched_off(&w, v),
            "above the ground: no end"
        );
    }
    assert!(run_until(&mut w, 300, |w| hover::hover_height(w, v) <= 0));
    assert_eq!(
        w.obj(v).unwrap().pos.cell(),
        w.obj(bunker).unwrap().pos.cell()
    );
    assert!(depot::settled(&w, v), "down, still settled away");
    glasnost_sim::econ::assign_destination(&mut w, v, to);
    assert!(!depot::settled(&w, v), "the bunker's cell: the end");
    assert!(!hover::switched_off(&w, v), "switched on again");
    assert!(power::unit_disabled(&w, v), "though still disabled");
}

// spec: S-0314 Terms (Away from a depot: in cells that neither a depot nor a tank bunker occupies, of the ends only the enabling fires; a destination received there, down, ends nothing)
#[test]
fn settled_away_on_open_ground_only_the_enabling_ends_it() {
    let (mut w, k) = world(true);
    let v = at(&mut w, "SKIMMER", 0, 9);
    glasnost_sim::testkit::run(&mut w, 300);
    power_order(&mut w, k, false);
    assert!(run_until(&mut w, 30, |w| power::unit_disabled(w, v)));
    assert!(run_until(&mut w, 300, |w| hover::hover_height(w, v) <= 0));
    glasnost_sim::econ::assign_destination(
        &mut w,
        v,
        Some(glasnost_sim::world::Target::Cell(cell(3, 9))),
    );
    assert!(depot::settled(&w, v), "no depot or bunker: no end");
    assert!(hover::switched_off(&w, v));
    power_order(&mut w, k, true);
    assert!(run_until(&mut w, 30, |w| !power::unit_disabled(w, v)));
    assert!(!depot::settled(&w, v), "the enabling ends it");
}
