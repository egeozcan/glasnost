//! An animation type whose trailer leads back to itself cannot keep the
//! turn pass from ending (#224). Synthetic data only.

use std::sync::mpsc;
use std::time::Duration;

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::AnimationType;
use glasnost_sim::anims::{self, Spawn};
use glasnost_sim::testkit;
use glasnost_sim::world::{Pos, World};

const N: i32 = 24;

/// Animation types by name and trailer.
fn rules(types: &[(&str, Option<&str>)]) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for &(name, tr) in types {
        let l = r.registries.entry("animations".into()).or_default();
        if !l.iter().any(|n| n == name) {
            l.push(name.into());
        }
        r.sim.animations.insert(
            name.into(),
            AnimationType {
                frame_count: Some(200),
                rate: Some(900),
                trailer_anim: tr.map(Into::into),
                trailer_separation: tr.map(|_| 1),
                ..Default::default()
            },
        );
    }
    r
}

/// A game with one animation of type `first` at the middle cell.
fn world(types: &[(&str, Option<&str>)], first: &str) -> World {
    let mut w = testkit::game_with(&rules(types), N, 7);
    let at = Pos::centre(testkit::mid(N, 0, 0), 0);
    anims::spawn(&mut w, first, Some(at), Spawn::default())
        .id
        .expect("animation made");
    w
}

/// One turn pass by hand, stopped after 5000 animation turns. Returns
/// whether it ended and the animations in play afterwards.
fn pass(w: &mut World) -> (bool, usize) {
    w.update.begin_pass();
    let mut turns = 0;
    let mut ended = false;
    while turns < 5000 {
        let Some(id) = w.update.next_turn() else {
            ended = true;
            break;
        };
        if w.anims.anims.contains_key(&id) {
            anims::turn(w, id);
            turns += 1;
        }
    }
    w.update.end_pass();
    (ended, w.anims.anims.len())
}

// spec: S-0319 claim 12 (every animation whose type names a trailer makes it on a due tick, its first turn included) and S-0203 claim 4 (a new object takes its first turn later in the same pass): a chain without a loop is unchanged
#[test]
fn a_trailer_chain_without_a_loop_is_unchanged() {
    let types = [("A", Some("B")), ("B", Some("C")), ("C", Some("PLAIN")), ("PLAIN", None)];
    let mut w = world(&types, "A");
    assert_eq!(pass(&mut w), (true, 4), "A, B, C and PLAIN");
}

// nocite: our own input validation (#224): an animation made through a trailer link that closes a loop of types makes no trailer through such a link, so the pass ends
#[test]
fn a_self_trailer_ends_the_pass() {
    let mut w = world(&[("LOOPX", Some("LOOPX"))], "LOOPX");
    let (ended, n) = pass(&mut w);
    assert!(ended, "the pass did not end: {n} animations");
    assert_eq!(n, 2, "the first and its trailer");
    let (ended, n) = pass(&mut w);
    assert!(ended);
    assert_eq!(n, 3, "only the first makes a trailer in each tick");
}

// nocite: our own input validation (#224): the same for a loop of two types and for a chain that enters a loop
#[test]
fn a_trailer_loop_of_two_and_a_chain_into_it_end_the_pass() {
    let types = [("X", Some("Y")), ("Y", Some("X")), ("Z", Some("X"))];
    let mut w = world(&types, "X");
    assert_eq!(pass(&mut w), (true, 2), "X and its trailer Y");
    let mut w = world(&types, "Z");
    assert_eq!(pass(&mut w), (true, 3), "Z, X (no loop yet) and Y");
}

// nocite: our own input validation (#224): one real tick returns
#[test]
fn a_self_trailer_tick_returns() {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut w = world(&[("LOOPX", Some("LOOPX"))], "LOOPX");
        testkit::run(&mut w, 1);
        let _ = tx.send(w.anims.anims.len());
    });
    let got = rx.recv_timeout(Duration::from_secs(10)).ok();
    assert!(got.is_some(), "one tick did not return within 10 seconds");
}
