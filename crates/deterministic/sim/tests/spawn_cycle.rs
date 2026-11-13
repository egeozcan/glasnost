//! A debris piece type or bouncing animation type whose spawn type leads
//! back to itself cannot keep the turn pass from ending or grow without end
//! (#174). Synthetic data only.

// The watchdog thread and its channel belong to the test harness, not to the
// simulation, which runs on that thread alone.
#![allow(clippy::disallowed_methods, clippy::disallowed_types)]

use std::sync::mpsc;
use std::time::Duration;

use glasnost_num::F64;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, VoxelAnimationType};
use glasnost_sim::anims::{self, Spawn};
use glasnost_sim::bounce::Flight;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{pieces, testkit};

const N: i32 = 24;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// A meteor piece type of duration 1 that spawns `spawns`, four per draw.
fn piece(spawns: &str) -> VoxelAnimationType {
    VoxelAnimationType {
        is_meteor: Some(true),
        duration: Some(1),
        min_z_vel: dec("-10"),
        max_xy_vel: dec("6"),
        spawns: Some(spawns.into()),
        spawn_count: Some(4),
        ..Default::default()
    }
}

/// Piece types by name and spawn type.
fn piece_rules(types: &[(&str, &str)]) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for &(name, spawns) in types {
        register(&mut r, "voxel_animations", name);
        r.sim.voxel_animations.insert(name.into(), piece(spawns));
    }
    register(&mut r, "voxel_animations", "PLAIN");
    r.sim.voxel_animations.insert(
        "PLAIN".into(),
        VoxelAnimationType {
            is_meteor: Some(true),
            duration: Some(1),
            ..Default::default()
        },
    );
    r
}

fn world_with_piece(types: &[(&str, &str)], first: &str) -> World {
    let mut w = testkit::game_with(&piece_rules(types), N, 7);
    let t = w.rules().pieces.index(first).expect("piece type");
    let at = Pos::centre(testkit::mid(N, 0, 0), 0);
    pieces::create(&mut w, t, at, None).expect("piece made");
    w
}

/// One turn pass of the pieces by hand, stopped after 5000 piece turns.
/// Returns whether it ended and the pieces made in it (the pieces in play
/// and those that have ended), counted by object number.
fn piece_pass(w: &mut World) -> (bool, u32) {
    let first = w.next_id;
    w.update.begin_pass();
    let mut turns = 0;
    let mut ended = false;
    while turns < 5000 {
        let Some(id) = w.update.next_turn() else {
            ended = true;
            break;
        };
        if w.pieces.contains_key(&id) {
            pieces::turn(w, id);
            turns += 1;
        }
    }
    w.update.end_pass();
    (ended, w.next_id - first)
}

// spec: S-0317 claim 12 (an ending meteor piece makes n = R(0, c) + R(0, c) pieces of its spawn type) and claim 6 (they enter play at once): a chain without a loop is unchanged
#[test]
fn a_spawn_chain_without_a_loop_ends() {
    let mut w = world_with_piece(&[("A", "B"), ("B", "PLAIN")], "A");
    let (ended, _) = piece_pass(&mut w);
    assert!(ended);
}

// spec: S-0317 claim 29 (a piece made through a spawn link on a loop of piece types makes no spawns through such a link, so at most 2c pieces come of one)
#[test]
fn a_self_spawning_meteor_piece_ends_the_pass() {
    let mut w = world_with_piece(&[("LOOP", "LOOP")], "LOOP");
    let (ended, made) = piece_pass(&mut w);
    assert!(ended, "the pass did not end: {} pieces", w.pieces.len());
    assert!(made <= 8, "{made} objects made from one piece");
}

// spec: S-0317 claim 29 (the same for a loop of two types)
#[test]
fn a_spawn_loop_of_two_ends_the_pass() {
    let mut w = world_with_piece(&[("X", "Y"), ("Y", "X")], "X");
    let (ended, made) = piece_pass(&mut w);
    assert!(ended, "the pass did not end: {} pieces", w.pieces.len());
    assert!(made <= 8, "{made} objects made from one piece");
}

// spec: S-0317 claim 29 (one real tick returns)
#[test]
fn a_self_spawning_meteor_piece_tick_returns() {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut w = world_with_piece(&[("LOOP", "LOOP")], "LOOP");
        testkit::run(&mut w, 1);
        let _ = tx.send(w.pieces.len());
    });
    let got = rx.recv_timeout(Duration::from_secs(10)).ok();
    assert!(got.is_some(), "one tick did not return within 10 seconds");
}

/// A falling animation type SELF (bouncer flag) that spawns `spawns`, four
/// per draw, and a plain long type PLAIN.
fn anim_rules(spawns: &str) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let long = || AnimationType {
        frame_count: Some(200),
        rate: Some(900),
        ..Default::default()
    };
    register(&mut r, "animations", "SELF");
    r.sim.animations.insert(
        "SELF".into(),
        AnimationType {
            bouncer: Some(true),
            spawns: Some(spawns.into()),
            spawn_count: Some(4),
            ..long()
        },
    );
    register(&mut r, "animations", "PLAIN");
    r.sim.animations.insert("PLAIN".into(), long());
    r
}

/// Puts an animation on the ground's doorstep: 5 leptons up, falling 10
/// leptons a tick, so it lands in its next turn.
fn about_to_land(w: &mut World, id: ObjectId, at: Pos) {
    let m = w.rules().mode;
    let d = |x: i32| F64::from_i64(i64::from(x), m).to_f32(m);
    let a = w.anims.anims.get_mut(&id).unwrap();
    a.flight = Some(Flight {
        p: [d(at.x), d(at.y), d(5)],
        v: [d(0), d(0), d(-10)],
    });
    a.pos = Some(Pos::new(at.x, at.y, 5));
}

/// Lands animation `id` in one turn; returns the animations it made.
fn land(w: &mut World, id: ObjectId, at: Pos) -> Vec<ObjectId> {
    about_to_land(w, id, at);
    let first = w.next_id;
    w.effects.clear();
    anims::turn(w, id);
    assert!(!w.anims.anims.contains_key(&id), "it landed and ended");
    let made = w
        .effects
        .iter()
        .filter(|e| matches!(e, Effect::Anim { name, .. } if name == "SELF" || name == "PLAIN"))
        .count();
    let ids: Vec<ObjectId> = w
        .anims
        .anims
        .keys()
        .copied()
        .filter(|id| id.0 >= first)
        .collect();
    assert_eq!(ids.len(), made);
    ids
}

// spec: S-0319 claim 9 (a landing makes n = R(0, c) + R(0, c) animations of the spawn type): the spawns of a type that is not in a loop land with their own spawns
#[test]
fn a_bounce_spawn_without_a_loop_is_unchanged() {
    let mut w = testkit::game_with(&anim_rules("PLAIN"), N, 7);
    let at = Pos::centre(testkit::mid(N, 0, 0), 0);
    let id = anims::spawn(&mut w, "SELF", Some(at), Spawn::default())
        .id
        .unwrap();
    let made = land(&mut w, id, at);
    assert!(!made.is_empty(), "this seed draws at least one spawn");
}

// spec: S-0319 claim 15 (an animation made through a spawn link on a loop of types makes no spawns through such a link when it lands)
#[test]
fn a_self_spawning_bouncing_animation_spawns_once() {
    let mut w = testkit::game_with(&anim_rules("SELF"), N, 7);
    let at = Pos::centre(testkit::mid(N, 0, 0), 0);
    let id = anims::spawn(&mut w, "SELF", Some(at), Spawn::default())
        .id
        .unwrap();
    let made = land(&mut w, id, at);
    assert!(!made.is_empty(), "the first one spawns");
    for m in made {
        assert!(
            land(&mut w, m, at).is_empty(),
            "a spawn of the loop spawns again"
        );
    }
}
