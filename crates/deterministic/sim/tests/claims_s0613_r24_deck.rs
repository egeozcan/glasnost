//! S-0613 revision 24, claim 20, on a high bridge (#424, sim batch S28): the
//! chaos drone's cloud on and under a deck, with S-0306 claim 3 (a) and
//! S-0303 claims 4 and 21, against measurement M-0026's drone rows. The
//! synthetic bridge and rules are `r24_deck_common`'s.

mod r24_deck_common;

use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, Layer, ObjectId, Pos, World};
use r24_deck_common::*;

fn berserk(w: &World, id: ObjectId) -> Option<i32> {
    w.obj(id).and_then(|o| o.combat.berserk)
}

/// Every particle in play, in any system.
fn particles(w: &World) -> usize {
    w.gas.systems.values().map(|s| s.particles.len()).sum()
}

/// What one run of the drone setup records.
#[derive(Default)]
struct DroneRun {
    /// Each pulse's tick and detonation point.
    pulses: Vec<(i32, Pos)>,
    /// Each HAZE animation's creation tick, number and position.
    hazes: Vec<(i32, ObjectId, Pos)>,
    /// B's remaining berserk time at the end of each tick, from the start.
    deck_berserk: Vec<(i32, Option<i32>)>,
    /// Whether C, on the ground, was ever berserk.
    ground_berserk: bool,
    /// The most particles in play at any tick.
    most_particles: usize,
    /// Animations whose position changed during their life, or that were
    /// outside the drone's cell or the deck height at some tick.
    moved: Vec<(ObjectId, Pos, Pos)>,
    /// Animations still in play at the end of the run.
    alive_at_end: usize,
    /// B and C at the end of the run: cell and layer.
    b_at_end: Option<(glasnost_map::CellXY, Layer)>,
    c_at_end: Option<(glasnost_map::CellXY, Layer)>,
}

/// The drone setup of M-0026 on the synthetic bridge: MADBOT stands in the
/// test cell (on the deck when `on_deck`, else on the ground under it),
/// infantry B stands on the deck in the east cell and infantry C on the
/// ground under it; MADBOT is ordered to deploy and the game runs `ticks`
/// ticks.
fn drone_run(on_deck: bool, ticks: i32) -> DroneRun {
    let mut w = game();
    bridge(&mut w);
    let drone = testkit::spawn(&mut w, Kind::Vehicle, "MADBOT", OURS, test_cell(), 0);
    if on_deck {
        onto_deck(&mut w, drone);
    }
    let b = testkit::spawn(&mut w, Kind::Infantry, "MAN", THEIRS, east_cell(), 0);
    onto_deck(&mut w, b);
    let c = testkit::spawn(&mut w, Kind::Infantry, "MAN", THEIRS, east_cell(), 0);
    orders::issue(&mut w, HouseId(OURS), Order::Deploy { actor: drone });
    let mut run = DroneRun::default();
    let mut live: Vec<(ObjectId, Pos)> = Vec::new();
    for _ in 0..ticks {
        w.effects.clear();
        testkit::run(&mut w, 1);
        let t = w.tick;
        for e in &w.effects {
            match e {
                Effect::Detonation { pos, .. } => run.pulses.push((t, *pos)),
                Effect::Anim {
                    name,
                    pos,
                    id: Some(id),
                    ..
                } if name == "HAZE" => {
                    run.hazes.push((t, *id, *pos));
                    live.push((*id, *pos));
                }
                _ => {}
            }
        }
        live.retain(|&(id, at)| {
            let Some(a) = w.anims.anims.get(&id) else {
                return false;
            };
            let now = a.pos.unwrap_or_default();
            if now != at || now.cell() != test_cell() || now.z != DECK {
                run.moved.push((id, at, now));
            }
            true
        });
        run.deck_berserk.push((t, berserk(&w, b)));
        run.ground_berserk |= berserk(&w, c).is_some();
        run.most_particles = run.most_particles.max(particles(&w));
    }
    run.alive_at_end = live.len();
    let place = |id: ObjectId| w.obj(id).map(|o| (o.pos.cell(), o.layer));
    run.b_at_end = place(b);
    run.c_at_end = place(c);
    run
}

/// The checks both setups share: the infantry stayed where they were put;
/// no particle; one animation per pulse, made in the pulse's tick in the
/// drone's cell at the deck height, which keeps its position for its whole
/// life; B made berserk at the first pulse, its time set anew at every
/// pulse and berserk to the end of the run; C never berserk.
fn common_checks(run: &DroneRun) {
    assert_eq!(run.b_at_end, Some((east_cell(), Layer::Deck)), "B stayed");
    assert_eq!(run.c_at_end, Some((east_cell(), Layer::Ground)), "C stayed");
    assert!(run.pulses.len() >= 3, "it pulsed: {:?}", run.pulses);
    assert_eq!(run.most_particles, 0, "no particle of any kind");
    assert_eq!(
        run.hazes.len(),
        run.pulses.len(),
        "one animation per pulse: {:?}",
        run.hazes
    );
    for (&(tp, _), &(ta, _, at)) in run.pulses.iter().zip(&run.hazes) {
        assert_eq!(tp, ta, "the animation in its pulse's tick");
        // spec: S-0303 claim 21 (an invisible projectile's animation at N(P, 32, b): within 32 leptons of P, at P's height)
        assert_eq!(at.cell(), test_cell(), "in the drone's cell");
        assert_eq!(at.z, DECK, "at the deck height");
    }
    assert!(
        run.moved.is_empty(),
        "no animation changed its position: {:?}",
        run.moved
    );
    assert!(
        run.alive_at_end < run.hazes.len(),
        "animations ended within the run, so their whole life was seen"
    );
    let first = run.pulses[0].0;
    for &(t, left) in &run.deck_berserk {
        assert_eq!(
            left.is_some(),
            t >= first,
            "tick {t}: B berserk from the first pulse on"
        );
    }
    // No pulse without a hit on B: at each pulse after the first, B's
    // remaining time is set anew, above what it was the tick before.
    for &(tp, _) in &run.pulses[1..] {
        let at = |t: i32| run.deck_berserk.iter().find(|x| x.0 == t).and_then(|x| x.1);
        assert!(
            at(tp) > at(tp - 1),
            "pulse at {tp} reached B: {:?} → {:?}",
            at(tp - 1),
            at(tp)
        );
    }
    // spec: S-0303 claim 4 (the deck layer only: the ground infantry under B is not hit)
    assert!(!run.ground_berserk, "C, on the ground, is not reached");
}

/// spec: S-0613 claim 20 (r24: an area berserk weapon whose data gives it no particle route makes no particle; its cloud is its animations, which claims 8 to 13 neither move nor make hit; for a firer on a high bridge's deck whose shot meets S-0306 claim 3 (a)'s conditions, the shot detonates at the centre of the firer's cell at the deck height, so its animations are made at that height and keep it, and its area damage looks at the bridge-deck layer, S-0303 claims 4 and 21) with S-0610 claim 8 (the area berserk weapon) and S-0306 claim 3 (a) (area fire on a high bridge: K is the centre of the firer's cell at the deck height; the invisible straight projectile placed there detonates in its first turn by T1's downward deck crossing)
/// measure: M-0026 (drone setup: no particle of any kind; each pulse's animation in the drone's cell at the deck's height, with the same position for its whole life; no pulse without a hit on B in the animation's first tick; B berserk from the first pulse to the end of the run)
#[test]
fn claim_20_a_drone_on_the_deck_pulses_at_the_deck_height() {
    let run = drone_run(true, 200);
    for &(t, p) in &run.pulses {
        assert_eq!(
            p,
            Pos::centre(test_cell(), DECK),
            "tick {t}: the centre of the firer's cell at the deck height"
        );
    }
    common_checks(&run);
}

/// spec: S-0613 claim 20 (r24: for a firer in a high-bridge cell under the deck whose shot meets S-0306 claim 3 (a)'s conditions, the same point: the centre of the firer's cell at the deck height, so its animations are made at that height and its area damage looks at the bridge-deck layer) with S-0306 claim 3 (a) (K is the centre of the firer's cell at the deck height, on the deck or under it) and S-0303 claims 4 and 21
#[test]
fn claim_20_a_drone_under_the_deck_pulses_at_the_deck_height() {
    let run = drone_run(false, 200);
    for &(t, p) in &run.pulses {
        assert_eq!(
            p,
            Pos::centre(test_cell(), DECK),
            "tick {t}: at the deck height, not under it"
        );
    }
    common_checks(&run);
}
