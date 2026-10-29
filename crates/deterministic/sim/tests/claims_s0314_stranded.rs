//! The stranded hovering vehicle of S-0314 (claim 22, Edge cases *Hovering
//! vehicle stranded*, `quirks.depot_loss_strands_hover`) in both positions:
//! the reference ruleset (the switch on, the original) and our play-fixes
//! layer, which sets it off (M8 step C7 part 2, `data/layers/play-fixes.jsonc`).
//! Synthetic data: invented types, rates and costs, no game data.

use glasnost_rules::layers::Layers;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, LocomotionClass, Locomotor, MovementZone, UnitType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};
use glasnost_sim::{depot, hover, testkit, tick};

const SIZE: i32 = 30;
const ALPHA: HouseId = HouseId(0);

/// The two positions: the reference ruleset and the play-fixes layer.
const BOTH: [Layers; 2] = [Layers::REFERENCE, Layers::PLAY_FIXES];

/// How the synthetic depot is built.
#[derive(Clone, Copy)]
enum Depot {
    /// A 1 × 1 depot: its docking point lies in its only cell, so a hover
    /// that moves while it sinks there soon leaves the depot's cells.
    Small,
    /// A 5 × 5 depot with two places 512 leptons either side of its centre:
    /// S-0312 claim 17 destroys a contact closer than 256 leptons to the
    /// destroyed building's reference point, so a hover docked at the
    /// centre would die with the depot instead of being stranded.
    Wide,
}

/// The combat ruleset plus a hovering vehicle SKIMMER (strength 600) and a
/// repair depot DEPOT of the given build (repair step 10, 25 %), with the
/// quirk switches the layers set, read from the layer file itself.
fn rules(layers: Layers, build: Depot) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let mut push = |reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push("vehicles", "SKIMMER");
    push("buildings", "DEPOT");
    r.sim.vehicles.insert(
        "SKIMMER".into(),
        UnitType {
            strength: Some(600),
            speed: Some(8),
            rot: Some(8),
            cost: Some(900),
            locomotion_class: Some(LocomotionClass::Hover),
            locomotor: Some(Locomotor::Hover),
            movement_zone: Some(MovementZone::Amphibious),
            ..Default::default()
        },
    );
    let mut d = BuildingType {
        strength: Some(800),
        unit_repair: Some(true),
        number_impassable_rows: Some(0),
        cost: Some(800),
        ..Default::default()
    };
    // A build-up sprite, so that a sale runs (S-0408 claim 4).
    d.econ.buildup = Some(true);
    match build {
        Depot::Small => d.foundation = Some([1, 1]),
        Depot::Wide => {
            d.foundation = Some([5, 5]);
            d.econ.number_of_docks = Some(2);
            d.econ.docking_offsets = Some(vec![[-512, 0, 0], [512, 0, 0]]);
        }
    }
    r.sim.buildings.insert("DEPOT".into(), d);
    r.sim.general.repair_step = Some(10);
    r.sim.general.repair_percent = Some(glasnost_rules::sim::Decimal("25%".into()));
    // No path-retry delay: a move order given while the hover's delay runs
    // waits out the rest of it (S-0212 claim 21 and its state table, row
    // "destination changes"; S-0503 H14, a retarget while a positive retry
    // delay runs), and these tests are about S-0314's settling.
    r.sim.general.path_delay = Some(glasnost_rules::sim::Decimal("0".into()));
    r.globals.quirks = layers
        .apply_quirks(r.globals.quirks)
        .expect("the layer reads");
    r
}

fn mv(actor: ObjectId, dx: i32, dy: i32) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(testkit::mid(SIZE, dx, dy))),
    }
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

fn ground(w: &World, v: ObjectId) -> (i32, i32) {
    let o = w.obj(v).unwrap();
    (o.pos.x, o.pos.y)
}

fn down(w: &World, v: ObjectId) -> bool {
    hover::hover_height(w, v) <= 0
}

/// Whether a building occupies vehicle `v`'s cell (here only the depot).
fn on_the_depot(w: &World, v: ObjectId) -> bool {
    let c = w.obj(v).unwrap().pos.cell();
    w.cells
        .list(c, Layer::Ground)
        .iter()
        .any(|&x| w.obj(x).is_some_and(|o| o.kind == Kind::Building))
}

/// Whether `v` stays where it is for `n` ticks under the move order to
/// cell (`dx`, `dy`) given first.
fn stays_put(w: &mut World, v: ObjectId, dx: i32, dy: i32, n: i32) -> bool {
    let at = ground(w, v);
    assert!(orders::issue(w, ALPHA, mv(v, dx, dy)));
    for _ in 0..n {
        tick::step(w);
    }
    ground(w, v) == at
}

/// Whether `v` reaches cell (`dx`, `dy`) within `n` ticks of a move order
/// there.
fn reaches(w: &mut World, v: ObjectId, dx: i32, dy: i32, n: i32) -> bool {
    assert!(orders::issue(w, ALPHA, mv(v, dx, dy)));
    let to = testkit::mid(SIZE, dx, dy);
    run_until(w, n, |w| w.obj(v).unwrap().pos.cell() == to)
}

/// A damaged SKIMMER clicked onto an own DEPOT and run to its movement
/// arrival: in contact, settled at a depot (S-0314 claim 17, Terms),
/// switched off and still above the ground (S-0503 claim 10). Returns the
/// world, the depot and the vehicle.
fn arrived(layers: Layers, build: Depot) -> (World, ObjectId, ObjectId) {
    let r = rules(layers, build);
    let mut w = testkit::game_with(&r, SIZE, 5);
    for h in &mut w.houses {
        h.credits = 10_000;
    }
    let d = testkit::spawn(
        &mut w,
        Kind::Building,
        "DEPOT",
        0,
        testkit::mid(SIZE, 0, 0),
        0,
    );
    let v = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "SKIMMER",
        0,
        testkit::mid(SIZE, 6, 1),
        0,
    );
    w.obj_mut(v).unwrap().health = 300;
    while w.tick < 4 {
        tick::step(&mut w);
    }
    let click = targeting::click(&w, ALPHA, &[v], ClickOn::Object(d), Modifiers::default());
    assert!(!click.orders.is_empty(), "the click gives an enter order");
    for o in click.orders {
        assert!(orders::issue(&mut w, ALPHA, o));
    }
    assert!(run_until(&mut w, 900, |w| depot::settled(w, v)));
    assert_eq!(w.obj(v).unwrap().contact, Some(d));
    assert!(depot::settled_at_depot(&w, v));
    assert!(hover::switched_off(&w, v) && !down(&w, v), "sinking");
    (w, d, v)
}

fn lethal() -> Hit {
    Hit {
        amount: 100_000,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    }
}

// spec: S-0314 claim 22 and Edge cases (Hovering vehicle stranded: with quirks.depot_loss_strands_hover on, a hover whose depot is destroyed after it became settled stays settled and, down on the ground, never moves again; off, as the play-fixes layer sets it, losing contact with the depot ends being settled, so it is switched on again and moves); S-0312 claim 17 (a contact 256 leptons or more from the reference point is only released)
#[test]
fn a_hover_whose_depot_is_destroyed_moves_only_with_the_play_fixes() {
    for layers in BOTH {
        let fixed = layers.play_fixes;
        let (mut w, d, v) = arrived(layers, Depot::Wide);
        assert!(
            run_until(&mut w, 900, |w| w.obj(v).unwrap().health > 300
                && down(w, v)),
            "down, and the settling step taken"
        );
        assert!(depot::settled_at_depot(&w, v));
        damage::apply(&mut w, d, lethal());
        tick::step(&mut w);
        assert!(w.obj(d).is_none_or(|b| !b.on_map), "the depot is gone");
        assert!(w.obj(v).is_some_and(|o| o.health > 0), "the hover survives");
        assert_eq!(w.obj(v).unwrap().contact, None);
        assert_eq!(depot::settled(&w, v), !fixed, "fixes {fixed}");
        if fixed {
            assert!(!hover::switched_off(&w, v), "switched on again");
            assert!(reaches(&mut w, v, 10, 10, 900), "and it moves");
        } else {
            assert!(stays_put(&mut w, v, 10, 10, 300), "stranded");
            assert!(stays_put(&mut w, v, -8, 6, 300), "under every later order");
            assert!(depot::settled(&w, v));
        }
    }
}

// spec: S-0314 claim 22 (Settled vehicles: a move order in the sinking window ends nothing; down, a settled hover cannot move) and Terms (What ends it: a destination ends being settled only while a depot or tank bunker occupies the cell; Away from a depot's reasoning: outside every depot's cells no later order ends it), with quirks.depot_loss_strands_hover on; S-0314 Edge cases and claim 20 (off, as the play-fixes layer sets it: the order releases the approaching vehicle's contact at once, which ends being settled, so it is switched on again and leaves)
#[test]
fn a_hover_ordered_off_its_depot_while_sinking_moves_only_with_the_play_fixes() {
    for layers in BOTH {
        let fixed = layers.play_fixes;
        let (mut w, _, v) = arrived(layers, Depot::Small);
        assert!(!depot::tethered(&w, v), "an ordinary depot: no tether");
        assert!(orders::issue(&mut w, ALPHA, mv(v, -9, 0)));
        tick::step(&mut w);
        assert_eq!(w.obj(v).unwrap().contact, None, "the order releases it");
        assert_eq!(depot::settled(&w, v), !fixed, "fixes {fixed}");
        if fixed {
            assert!(!hover::switched_off(&w, v), "switched on again");
            let to = testkit::mid(SIZE, -9, 0);
            assert!(
                run_until(&mut w, 900, |w| w.obj(v).unwrap().pos.cell() == to),
                "it goes where it was sent"
            );
            assert!(!depot::settled(&w, v));
        } else {
            assert!(run_until(&mut w, 300, |w| down(w, v)));
            assert!(
                !on_the_depot(&w, v),
                "it came down outside the depot's cells"
            );
            assert!(depot::settled(&w, v), "down and still settled");
            assert!(stays_put(&mut w, v, -9, 3, 300), "a new cell ends nothing");
            assert!(stays_put(&mut w, v, 6, 6, 300), "nor any later order");
            assert!(depot::settled(&w, v));
        }
    }
}

// spec: S-0314 claim 20 (a docked vehicle keeps its contact under a move order) and claim 21 (the depot's checks go on wherever it stands; its service ends the contact); claim 22 and Edge cases (with quirks.depot_loss_strands_hover on it stays settled after that and, down outside the depot's cells, never moves again; off, as the play-fixes layer sets it, the contact's end at the end of service ends being settled)
#[test]
fn a_docked_hover_ordered_off_while_sinking_is_freed_at_its_service_end_by_the_play_fixes() {
    for layers in BOTH {
        let fixed = layers.play_fixes;
        let (mut w, d, v) = arrived(layers, Depot::Small);
        // Our synthetic depot's arrival sets no tether (claim 17's pad
        // exchange does not arise here), so the test sets it, as
        // claims_s0314_depot.rs does.
        w.obj_mut(v).unwrap().depot.link.as_mut().unwrap().tethered = true;
        assert!(depot::tethered(&w, v));
        assert!(orders::issue(&mut w, ALPHA, mv(v, -9, 0)));
        assert!(run_until(&mut w, 300, |w| down(w, v)));
        assert!(!on_the_depot(&w, v), "down outside the depot's cells");
        assert_eq!(w.obj(v).unwrap().contact, Some(d), "docked: contact kept");
        assert!(depot::settled(&w, v), "fixes {fixed}: still settled");
        assert!(
            run_until(&mut w, 3000, |w| w.obj(v).unwrap().contact.is_none()),
            "the service ends"
        );
        assert_eq!(w.obj(v).unwrap().health, 600, "repaired where it stands");
        assert_eq!(depot::settled(&w, v), !fixed, "fixes {fixed}");
        if fixed {
            assert!(reaches(&mut w, v, -9, 0, 900), "it leaves");
        } else {
            assert!(stays_put(&mut w, v, -9, 0, 300), "stranded");
            assert!(stays_put(&mut w, v, 6, 6, 300), "under every later order");
        }
    }
}

// spec: S-0314 Edge cases and Terms (quirks.depot_loss_strands_hover off: losing contact with a depot ends being settled; on, nothing but Terms' ends does) and S-0213 claim 10 (a deploy order breaks off the contact); design: the engine sees a contact broken off outside the depot's own code at the vehicle's next turn
#[test]
fn a_deploy_order_breaking_a_sinking_hovers_contact_frees_it_only_with_the_play_fixes() {
    for layers in BOTH {
        let fixed = layers.play_fixes;
        let (mut w, _, v) = arrived(layers, Depot::Small);
        assert!(orders::issue(&mut w, ALPHA, Order::Deploy { actor: v }));
        tick::step(&mut w);
        tick::step(&mut w);
        assert_eq!(
            w.obj(v).unwrap().contact,
            None,
            "the deploy order breaks it"
        );
        assert_eq!(depot::settled(&w, v), !fixed, "fixes {fixed}");
        assert_eq!(hover::switched_off(&w, v), !fixed);
    }
}
