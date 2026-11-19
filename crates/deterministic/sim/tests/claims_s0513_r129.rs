//! Claim tests of S-0513 r129 claim 11 (Q-3253, #441): the nuclear warning
//! marker's place, its tie to its missile, its place in the update order and
//! its removal at once inside the missile's removal (scenario SW4 parts (b)
//! and (c), and S-0513's change for S-0203 claim 10). Synthetic rules
//! (`tests/sw_common`, with an endless warning-marker type); every name and
//! number is invented.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::superweapons::{self, nuke};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use sw_common::{ALPHA, STRIKE, ruleset};

/// The map size of these tests: large enough for three vehicles to drive 30
/// cells far from the target (SW4 part (c)).
const BIG: i32 = 40;

fn cell(dx: i32, dy: i32) -> CellXY {
    testkit::mid(BIG, dx, dy)
}

/// The superweapon rules with a warning-marker type that loops for ever
/// (S-0109 claim 7: loop count −1), so that only its missile ends it, and
/// an upward projectile that bursts high enough (20 000 leptons) for the
/// test vehicles to be moving before it does. `skip` sets
/// `quirks.update_order_removal_skip`.
fn rules(skip: bool) -> Ruleset {
    let mut r = ruleset();
    r.sim.animations.insert(
        superweapons::WARNING_MARKER.into(),
        glasnost_rules::sim::AnimationType {
            // spec: S-0109 claim 7 (loop count −1: endless)
            loop_count: Some(-1),
            ..Default::default()
        },
    );
    r.registries
        .entry("animations".into())
        .or_default()
        .push(superweapons::WARNING_MARKER.into());
    r.sim
        .projectiles
        .get_mut("UpShot")
        .unwrap()
        .detonation_altitude = Some(20_000);
    r.globals.quirks.update_order_removal_skip = skip;
    r
}

fn game(r: &Ruleset) -> World {
    let mut w = testkit::game_with(r, BIG, 1);
    while orders::blocked(&w) {
        glasnost_sim::tick::step(&mut w);
    }
    w
}

/// The warning markers in play.
fn markers(w: &World) -> Vec<ObjectId> {
    w.anims
        .anims
        .iter()
        .filter(|(_, a)| a.player.name == superweapons::WARNING_MARKER.as_bytes())
        .map(|(&id, _)| id)
        .collect()
}

/// The projectiles of weapon `name` in play.
fn missiles(w: &World, name: &str) -> Vec<ObjectId> {
    let wp = *w.rules().combat.weapon_index.get(name).unwrap();
    w.projectiles
        .values()
        .filter(|p| p.weapon == wp)
        .map(|p| p.id)
        .collect()
}

fn index(w: &World, id: ObjectId) -> Option<usize> {
    w.update.order.iter().position(|&o| o == id)
}

/// A silo of Alpha at `cell(dx, dy)`, its strike record ready, launched at
/// `target`; run until the upward missile is in play. Returns the missile.
fn silo_strike(w: &mut World, dx: i32, dy: i32, target: CellXY) -> ObjectId {
    let silo = testkit::spawn(w, Kind::Building, "SILO", ALPHA.0, cell(dx, dy), 0);
    sw_common::ready_tick(w, ALPHA, STRIKE, 200);
    nuke::silo_launch(w, ALPHA, STRIKE, target, Some(silo));
    sw_common::until(w, 10, |w| !missiles(w, "Lift").is_empty());
    missiles(w, "Lift")[0]
}

/// A one-time strike of Alpha at `target`, made now. Returns the falling
/// missile.
fn one_time(w: &mut World, target: CellXY) -> ObjectId {
    nuke::one_time(w, Some(ALPHA), STRIKE, target);
    missiles(w, "NukePayload")[0]
}

/// The point claim 11 gives the marker: the target cell's centre at the
/// terrain surface height there, with no bridge lift.
fn surface_centre(w: &World, c: CellXY) -> Pos {
    let p = Pos::centre(c, 0);
    Pos::new(p.x, p.y, w.surface_at(p.x, p.y))
}

// spec: S-0513 claim 11 (r129, Q-3253: the marker enters play just before its missile, in the action or the silo update that creates that missile, so it stands directly before the missile in the update order; in a silo launch it is made in the silo's update in which the upward missile leaves, claim 8) and S-0203 claim 3 (entering play appends)
#[test]
fn claim11_r129_the_marker_enters_play_just_before_its_missile() {
    let r = rules(true);
    // A one-time strike.
    let mut w = game(&r);
    let fall = one_time(&mut w, cell(0, 0));
    let m = markers(&w);
    assert_eq!(m.len(), 1, "one marker");
    assert_eq!(
        index(&w, m[0]).map(|i| i + 1),
        index(&w, fall),
        "one-time strike: the marker directly before its missile: {:?}",
        w.update.order
    );
    // A silo strike: made in the silo's update in which the upward missile leaves.
    let mut w = game(&r);
    let up = silo_strike(&mut w, -5, 0, cell(0, 0));
    let m = markers(&w);
    assert_eq!(m.len(), 1, "made in the tick the upward missile leaves");
    assert_eq!(
        index(&w, m[0]).map(|i| i + 1),
        index(&w, up),
        "silo strike: the marker directly before its missile: {:?}",
        w.update.order
    );
}

/// Steps until `missile` leaves play, checking in every tick that the one
/// marker stands at `expected`. Returns the marker.
fn follow_to_the_end(w: &mut World, missile: ObjectId, expected: Pos) -> ObjectId {
    let m = markers(w);
    assert_eq!(m.len(), 1);
    let m = m[0];
    for t in 0..2000 {
        assert_eq!(
            w.anims.anims.get(&m).and_then(|a| a.pos),
            Some(expected),
            "tick {t}: the marker keeps its place"
        );
        glasnost_sim::tick::step(w);
        if !w.projectiles.contains_key(&missile) {
            return m;
        }
    }
    panic!("the missile did not leave play");
}

// spec: S-0513 claim 11 (r129, Q-3253, scenario SW4 part (b): the marker is created at the target cell's centre, at the terrain surface height there with no bridge lift, and keeps that position for its whole life, not following its missile; it is gone at the end of the tick in which its missile leaves play: it leaves at once, inside the missile's removal); runs (i) a one-time strike on open ground and (ii) a silo strike from a silo five cells from the target
#[test]
fn sw4_b_the_marker_keeps_its_place_and_is_gone_with_its_missile() {
    let r = rules(true);
    // (i)
    let mut w = game(&r);
    let target = cell(3, 2);
    let fall = one_time(&mut w, target);
    let expected = surface_centre(&w, target);
    let m = follow_to_the_end(&mut w, fall, expected);
    assert!(
        !w.anims.anims.contains_key(&m) && !w.update.order.contains(&m),
        "(i): gone at the end of the tick its missile left play"
    );
    // (ii)
    let mut w = game(&r);
    let target = cell(0, 0);
    let up = silo_strike(&mut w, -5, 0, target);
    let expected = surface_centre(&w, target);
    let m = follow_to_the_end(&mut w, up, expected);
    assert!(
        !w.anims.anims.contains_key(&m) && !w.update.order.contains(&m),
        "(ii): gone at the end of the tick its upward missile burst"
    );
    assert!(markers(&w).is_empty(), "the falling missile has none");
}

// spec: S-0513 claim 11 (r129, scenario SW4 part (b) run (iii): a one-time strike at a high-bridge cell: the marker stands at the terrain surface height, below the deck, with no bridge lift; the missile itself starts with the lift, claim 10 and F2)
#[test]
fn sw4_b_iii_on_a_high_bridge_the_marker_stands_below_the_deck() {
    let r = rules(true);
    let mut w = game(&r);
    let target = cell(3, 2);
    let i = w.cells.idx(target).unwrap();
    w.cells.deck[i] = true;
    assert!(w.high_bridge(target), "the target is a high-bridge cell");
    let fall = one_time(&mut w, target);
    let expected = surface_centre(&w, target);
    let lifted = superweapons::cell_point(&w, target, true);
    assert!(
        expected.z < lifted.z,
        "the deck is above the terrain surface"
    );
    let m = follow_to_the_end(&mut w, fall, expected);
    assert!(!w.anims.anims.contains_key(&m));
}

/// One tick of SW4 part (c)'s recorder: the update order at its start, and
/// for each test vehicle whether it changed its position in the tick.
struct Tick {
    order: Vec<ObjectId>,
    after: Vec<ObjectId>,
    moved: [bool; 3],
}

/// SW4 part (c): three vehicles created after `missile` entered play,
/// ordered 30 cells east across open flat ground far from the target; the
/// recorder runs until the tick after the missile leaves play. Returns the
/// vehicles, the log and the index of the tick in which the missile left.
fn removal_run(w: &mut World, missile: ObjectId) -> ([ObjectId; 3], Vec<Tick>, usize) {
    let vehicles = [-12, -10, -8].map(|dy| {
        let v = testkit::spawn(w, Kind::Vehicle, "BLOB", ALPHA.0, cell(-15, dy), 0);
        let order = Order::Behaviour {
            actor: v,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(cell(15, dy))),
        };
        assert!(orders::issue(w, ALPHA, order), "move order taken");
        v
    });
    let pos = |w: &World| vehicles.map(|v| w.obj(v).map(|o| o.pos));
    let mut log = Vec::new();
    let mut left = None;
    for _ in 0..2000 {
        let order = w.update.order.clone();
        let before = pos(w);
        let there = w.projectiles.contains_key(&missile);
        glasnost_sim::tick::step(w);
        let after = pos(w);
        let moved = [0, 1, 2].map(|k| before[k] != after[k]);
        log.push(Tick {
            order,
            after: w.update.order.clone(),
            moved,
        });
        if let Some(m) = markers(w).first()
            && w.projectiles.contains_key(&missile)
        {
            assert_eq!(
                index(w, *m).map(|i| i + 1),
                index(w, missile),
                "tick {}: the marker stands directly before its missile",
                w.tick
            );
        }
        if there && !w.projectiles.contains_key(&missile) {
            left = Some(log.len() - 1);
        } else if left.is_some() {
            break;
        }
    }
    let left = left.expect("the missile left play");
    (vehicles, log, left)
}

/// Checks SW4 part (c)'s void conditions and returns the vehicles' turns
/// in the tick in which the missile left play.
fn missed_turns(w: &mut World, missile: ObjectId, marker: ObjectId) -> [bool; 3] {
    let (vehicles, log, t) = removal_run(w, missile);
    let tick = &log[t];
    let at = tick
        .order
        .iter()
        .position(|&o| o == missile)
        .expect("the missile was in play");
    assert_eq!(
        tick.order.get(at.wrapping_sub(1)),
        Some(&marker),
        "the marker directly before its missile"
    );
    assert_eq!(
        &tick.order[at + 1..at + 4],
        &vehicles,
        "void unless the three objects directly after the missile are the test vehicles"
    );
    let gone: Vec<ObjectId> = tick
        .order
        .iter()
        .filter(|o| !tick.after.contains(o))
        .copied()
        .collect();
    let mut both = vec![marker, missile];
    both.sort_unstable();
    assert_eq!(
        gone, both,
        "void unless only the marker and the missile leave play in that tick"
    );
    assert_eq!(
        log[t - 1].moved,
        [true; 3],
        "void unless each moved in the tick before"
    );
    assert_eq!(
        log[t + 1].moved,
        [true; 3],
        "void unless each moved in the tick after"
    );
    tick.moved
}

// spec: S-0513 claim 11 (r129, Q-3253, scenario SW4 part (c): the marker stands before its missile in every tick; when the missile leaves play in its own turn the marker leaves at once inside that removal, and both removals count, so the two objects that directly follow the missile miss their turns in that tick and the third takes its turn) and S-0203 claim 10 (k counting removals, k objects directly after miss their turns; S-0513's change for its list of causes), with quirks.update_order_removal_skip on; a one-time strike and a silo strike
#[test]
fn sw4_c_both_removals_count_with_the_switch_on() {
    let r = rules(true);
    let mut w = game(&r);
    let fall = one_time(&mut w, cell(0, 15));
    let m = markers(&w)[0];
    assert_eq!(
        missed_turns(&mut w, fall, m),
        [false, false, true],
        "one-time strike: two missed turns, the third vehicle moves"
    );
    let mut w = game(&r);
    let up = silo_strike(&mut w, -20, 12, cell(0, 15));
    let m = markers(&w)[0];
    assert_eq!(
        missed_turns(&mut w, up, m),
        [false, false, true],
        "silo strike: two missed turns at the upward missile's burst"
    );
}

// spec: S-0513 claim 11 (r129: quirks.update_order_removal_skip governs both removals; with it off neither costs a turn) and S-0203 claim 10 (the switch off: a removal costs nobody a turn)
#[test]
fn sw4_c_neither_removal_costs_a_turn_with_the_switch_off() {
    let r = rules(false);
    let mut w = game(&r);
    let fall = one_time(&mut w, cell(0, 15));
    let m = markers(&w)[0];
    assert_eq!(missed_turns(&mut w, fall, m), [true; 3], "one-time strike");
    let mut w = game(&r);
    let up = silo_strike(&mut w, -20, 12, cell(0, 15));
    let m = markers(&w)[0];
    assert_eq!(missed_turns(&mut w, up, m), [true; 3], "silo strike");
}

// spec: S-0513 claim 11 (r129: the tie is a link of lifetime only; the marker is game state, so the tie is saved and hashed: a game saved while the marker is in play and loaded again keeps it, and the marker still leaves with its missile) and S-0109 claim 28 (an animation's state is simulation state, saved and hashed)
#[test]
fn claim11_r129_the_tie_survives_a_save_and_load() {
    let r = rules(true);
    let mut w = game(&r);
    let fall = one_time(&mut w, cell(3, 2));
    sw_common::run(&mut w, 3);
    let bytes = glasnost_sim::save::save(&w);
    let mut back = glasnost_sim::save::load(&bytes, w.rules_arc(), w.catalogue_arc()).expect("loads");
    assert_eq!(
        glasnost_sim::save::state_hash(&w),
        glasnost_sim::save::state_hash(&back)
    );
    let expected = surface_centre(&back, cell(3, 2));
    let m = follow_to_the_end(&mut back, fall, expected);
    assert!(!back.anims.anims.contains_key(&m));
}
