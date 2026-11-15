//! S-0314 claim 12 (sim batch S13, #256): a building being erased does not
//! reach the automatic repair and sale work: no automatic repair start and
//! no sale draw while the erasure lock holds. Synthetic data (an invented
//! building type, a repair-and-sell threshold of 0); the lock is inserted
//! directly, as `claims_s0207_r36.rs` does.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::BuildingType;
use glasnost_sim::rules::Kind;
use glasnost_sim::temporal::{self, Lock};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, World};

const SIZE: i32 = 24;

fn rules() -> Ruleset {
    let mut r = testkit::ruleset();
    r.registries
        .insert("buildings".into(), vec!["HUT".to_string()]);
    let mut b = BuildingType {
        strength: Some(400),
        cost: Some(1000),
        foundation: Some([1, 1]),
        ..Default::default()
    };
    b.econ.buildup = Some(true);
    b.econ.buildup_frames = Some(10);
    r.sim.buildings.insert("HUT".into(), b);
    // Synthetic: the gate passes for every level-0 house; a sale threshold of 0.
    r.sim.general.iq_repair_sell = Some(0);
    r.sim.general.iq_sell_back = Some(0);
    r.sim.general.credit_reserve = Some(100);
    r
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

/// A damaged HUT of house 0 with `credits`, off a repair-period multiple,
/// with an erasure lock when `erase`.
fn setup(erase: bool, credits: i32) -> (World, ObjectId) {
    let r = rules();
    let mut w = testkit::game_with_non_players(&r, SIZE, 13);
    testkit::run(&mut w, 4);
    for h in &mut w.houses {
        h.credits = credits;
    }
    let pb = w.rules().combat.general.repair_period;
    while (w.tick + 1) % pb == 0 || w.tick % pb == 0 {
        glasnost_sim::tick::step(&mut w);
    }
    let hut = testkit::spawn(&mut w, Kind::Building, "HUT", 0, c(0, 0), 0);
    w.obj_mut(hut).unwrap().health = 200;
    if erase {
        let leader = testkit::spawn(&mut w, Kind::Vehicle, "CART", 1, c(-6, -6), 0);
        w.temporal.locks.insert(
            hut,
            Lock {
                leader,
                joiners: Vec::new(),
                amount: 1_000_000,
                was_off: false,
                suspended: false,
            },
        );
    }
    (w, hut)
}

// spec: S-0314 claim 12 (buildings being erased do not reach this work) and scenario H13 part f (no automatic restart during erasure)
#[test]
fn an_erased_building_does_not_start_automatic_repair() {
    let (mut w, hut) = setup(false, 10_000);
    glasnost_sim::tick::step(&mut w);
    assert!(
        w.obj(hut).unwrap().econ.building.repairing,
        "control: it starts"
    );

    let (mut w, hut) = setup(true, 10_000);
    assert!(temporal::erased(&w, hut));
    glasnost_sim::tick::step(&mut w);
    assert!(temporal::erased(&w, hut), "still being erased");
    assert!(
        !w.obj(hut).unwrap().econ.building.repairing,
        "no automatic repair start while being erased"
    );
    assert!(!w.houses[0].combat.repair_hold, "and no hold");
}

// spec: S-0314 claim 8 (Being erased, Q-3113: a building being erased takes no step at a multiple of P_b, nothing is paid and health does not rise; the state does not turn repair off, and the steps resume at the first multiple after it ends)
#[test]
fn issue_256_an_erased_building_takes_no_paid_repair_step() {
    let pb = |w: &World| w.rules().combat.general.repair_period;
    // Runs to just past the next multiple of P_b.
    let past_a_multiple = |w: &mut World| {
        let p = pb(w);
        glasnost_sim::tick::step(w);
        while w.tick % p != 1 {
            glasnost_sim::tick::step(w);
        }
    };
    // Control: the repair, on by order, takes a paid step.
    let (mut w, hut) = setup(false, 10_000);
    w.obj_mut(hut).unwrap().econ.building.repairing = true;
    past_a_multiple(&mut w);
    assert!(w.houses[0].credits < 10_000, "control: the step is paid");
    assert!(w.obj(hut).unwrap().health > 200, "control: health rises");

    let (mut w, hut) = setup(true, 10_000);
    w.obj_mut(hut).unwrap().econ.building.repairing = true;
    past_a_multiple(&mut w);
    assert!(temporal::erased(&w, hut), "still being erased");
    assert_eq!(w.houses[0].credits, 10_000, "nothing is paid");
    assert!(w.obj(hut).unwrap().health <= 200, "health does not rise");
    assert!(
        w.obj(hut).unwrap().econ.building.repairing,
        "the erasure does not turn repair off"
    );
    // The lock ends: the steps resume at the next multiple.
    w.temporal.locks.remove(&hut);
    assert!(!temporal::erased(&w, hut));
    let health = w.obj(hut).unwrap().health;
    past_a_multiple(&mut w);
    assert!(w.houses[0].credits < 10_000, "the step is paid again");
    assert!(w.obj(hut).unwrap().health > health, "health rises again");
}

// spec: S-0314 claim 12 (buildings being erased do not reach this work) and claim 14 (the R(0, 50) draw on each eligible turn)
#[test]
fn an_erased_building_takes_no_sale_draw() {
    // Below the reserve with the hostile-hit mark: the sale branch.
    let draws = |erase: bool| {
        let (mut w, hut) = setup(erase, 0);
        w.obj_mut(hut).unwrap().combat.hostile_hit = true;
        let (mut a, mut b) = (w.clone(), w);
        a.obj_mut(hut).unwrap().combat.hostile_hit = false;
        glasnost_sim::tick::step(&mut a);
        glasnost_sim::tick::step(&mut b);
        // The same tick with and without the mark: equal streams mean no sale draw.
        a.rng == b.rng
    };
    assert!(!draws(false), "control: the marked building draws");
    assert!(draws(true), "no sale draw while being erased");
}
