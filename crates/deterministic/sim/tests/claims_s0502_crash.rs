//! Claim tests of S-0502 (movement of jumpjets): the crash motion, sinking,
//! the firing states, the importer defaults and the save format.

mod jump_common;

use glasnost_num::F64;
use glasnost_sim::combat::check::{Answer, check};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::jump::{self, JumpState, Phase};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use jump_common::*;

fn j(w: &World, id: ObjectId) -> &JumpState {
    w.obj(id).unwrap().jump.as_ref().unwrap()
}

fn js(w: &mut World, id: ObjectId) -> &mut JumpState {
    w.obj_mut(id).unwrap().jump.as_mut().unwrap()
}

fn kill(w: &mut World, id: ObjectId) {
    let health = w.obj(id).unwrap().health;
    damage::apply(
        w,
        id,
        Hit {
            amount: health * 10,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
}

/// A crashable vehicle cruising at 500 above flat ground.
fn flier(w: &mut World) -> ObjectId {
    let u = put(w, "JV", Kind::Vehicle, 0, -4, 0);
    go(w, u, at(10, 0));
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    jump::start_in_flight(w, u, Phase::Cruising, 500);
    js(w, u).vc = F64::from_i64(14, w.rules().mode);
    u
}

// spec: S-0502 claim 18 and S-0312 claims 15 and 16 (a killed crashable jumpjet above the terrain starts to crash and stays in play: three tumbling draws for a vehicle, none for infantry; its next update includes the extra drop k and enters crashing)
#[test]
fn claim_18_crash_start() {
    let mut w = game();
    let u = flier(&mut w);
    let mut r = w.rng.clone();
    let _ = r.unit(w.rules().mode);
    let half = w.rules().quirks.ranged_draw_half_span;
    let _ = r.ranged(0, 1, half);
    let _ = r.unit(w.rules().mode);
    kill(&mut w, u);
    assert!(w.obj(u).is_some(), "it stays in play");
    assert!(jump::crashing(&w, u));
    assert_eq!(
        phase(&w, u),
        Phase::Cruising,
        "the state still is the old one"
    );
    assert_eq!(w.rng.taken(), r.taken(), "the three tumbling draws");
    // Infantry with the crash flag: no tumbling draws.
    let mut w = game();
    let i = put(&mut w, "JJCRASH", Kind::Infantry, 0, -4, 0);
    go(&mut w, i, at(10, 0));
    jump::start_in_flight(&mut w, i, Phase::Cruising, 500);
    let before = w.rng.taken();
    kill(&mut w, i);
    assert!(w.obj(i).is_some() && jump::crashing(&w, i));
    assert_eq!(w.rng.taken(), before, "no tumbling draw for infantry");
    // On the ground, or without the crash flag, it leaves play at once.
    let mut w = game();
    let g = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    kill(&mut w, g);
    assert!(w.obj(g).is_none());
    let plain = put(&mut w, "JJ", Kind::Infantry, 0, 2, 0);
    go(&mut w, plain, at(10, 0));
    jump::start_in_flight(&mut w, plain, Phase::Cruising, 500);
    kill(&mut w, plain);
    assert!(
        w.obj(plain).is_none(),
        "a type without the crash flag vanishes"
    );
}

// spec: S-0312 claim 15 (r66, Q-2793, scenario D11 variant h: a destroyed vehicle whose type can crash starts to crash only at a positive height above the surface, measured from the deck on a bridge deck, so one standing on a deck 416 leptons above the terrain starts no crash, takes no tumbling draws and leaves play; the control, 416 leptons above the terrain off any deck, crashes, as in variant c)
#[test]
fn s0312_claim_15_d11_h_no_crash_from_a_deck() {
    for deck in [true, false] {
        let mut w = game();
        let g = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
        let c = pos(&w, g).cell();
        if deck {
            let i = w.cells.idx(c).unwrap();
            w.cells.deck[i] = true;
            w.cells.deck_entry[i] = true;
            w.cells.bridge_axis[i] = glasnost_sim::cells::Axis::X;
            w.cells.leave(c, glasnost_sim::world::Layer::Ground, g);
            w.cells.enter(c, glasnost_sim::world::Layer::Deck, g, false);
            w.obj_mut(g).unwrap().layer = glasnost_sim::world::Layer::Deck;
        }
        w.obj_mut(g).unwrap().pos.z += 416;
        let above = w.height_above_surface(g);
        assert_eq!(above, if deck { 0 } else { 416 });
        let before = w.rng.taken();
        kill(&mut w, g);
        if deck {
            assert!(w.obj(g).is_none(), "it leaves play");
            assert_eq!(w.rng.taken(), before, "no tumbling draws");
        } else {
            assert!(
                w.obj(g).is_some() && jump::crashing(&w, g),
                "the control crashes"
            );
        }
    }
}

// spec: S-0502 claim 18, F4 and scenario J12 (the fall: the update that starts the crash takes the ordinary height step of the old state then the extra drop; from then on, away from the surface, the ordinary descent by c and the extra drop k, each truncated: 11 leptons per tick; the heading's target 11520 ahead; the end below 1 lepton and the impact of S-0312 claim 16)
#[test]
fn claim_18_crash_motion() {
    let mut w = game();
    let u = flier(&mut w);
    kill(&mut w, u);
    let z0 = pos(&w, u).z;
    let mut zs = Vec::new();
    let mut targets = Vec::new();
    let mut prev_heading = j(&w, u).heading.value(w.tick);
    for _ in 0..80 {
        step(&mut w);
        let Some(o) = w.obj(u) else { break };
        zs.push(o.pos.z);
        let s = o.jump.as_ref().unwrap();
        targets.push((prev_heading, s.heading.target, s.phase, s.ht));
        prev_heading = s.heading.value(w.tick);
    }
    assert!(w.obj(u).is_none(), "it leaves play at the impact");
    // The entering update: the ordinary step of the cruising state (wobble raises the height towards 502) then the drop of 6.
    assert_eq!(zs[0], 496, "ordinary step then the extra drop (from {z0})");
    assert_eq!(targets[0].2, Phase::Crashing);
    assert_eq!(targets[0].3, -5, "target height −5");
    // 11 leptons per tick afterwards.
    for k in 1..zs.len() - 1 {
        assert_eq!(
            zs[k - 1] - zs[k],
            11,
            "tick {k}: {} to {}",
            zs[k - 1],
            zs[k]
        );
    }
    // The spin: from the first crashing tick on, the heading's target is 11520 clockwise of the heading used.
    for (used, target, phase, _) in targets.iter().skip(1) {
        if *phase == Phase::Crashing {
            assert_eq!(
                *target,
                ((i32::from(*used) + 11520).rem_euclid(65536)) as u16
            );
        }
    }
}

// spec: S-0502 claim 18 (crossing a deck: over a high-bridge cell the extra drop ends the fall when its height before it is at or above the deck and after it below, without snapping back to the deck)
#[test]
fn claim_18_crash_crossing_a_deck() {
    let mut w = game();
    let c = at(2, 0);
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 2, 2);
    let t = w.tick;
    w.obj_mut(u).unwrap().beh.force(Behaviour::Move, t, true);
    go(&mut w, u, at(2, 2));
    jump::start_in_flight(&mut w, u, Phase::Crashing, 421);
    w.obj_mut(u).unwrap().pos = Pos::new(Pos::centre(c, 0).x, Pos::centre(c, 0).y, 421);
    js(&mut w, u).ht = -5;
    // Not dead: the alive crash, so that the impact is the destruction of S-0312 claim 16.
    step(&mut w);
    // z 421: the ordinary step reaches 416, the drop 410, below the deck: the fall ends there.
    assert!(
        w.obj(u).is_none_or(|o| o.health <= 0),
        "the crash ended at the deck"
    );
}

// spec: S-0309 claim 27 (a destroyed object is dropped at the killing hit, also when it stays in play after that hit, a crashing object; every vehicle and building holder, no owner exception; no holder fires at it or its place) with S-0502 claim 18 (the killed crashable jumpjet stays in play) (M8 seam sweep 5a)
#[test]
fn claim_27_a_crashing_jumpjet_is_dropped_at_its_killing_hit() {
    use glasnost_sim::combat::effects::Effect;
    let mut w = game();
    let u = flier(&mut w);
    let holders = [
        put(&mut w, "CART", Kind::Vehicle, 1, -4, 3),
        put(&mut w, "LOW", Kind::Building, 1, 2, 4),
        put(&mut w, "CART", Kind::Vehicle, 0, -6, -3),
    ];
    for &x in &holders {
        w.obj_mut(x).unwrap().target = Some(Target::Object(u));
    }
    kill(&mut w, u);
    assert!(w.obj(u).is_some(), "it stays in play");
    assert!(jump::crashing(&w, u));
    for &x in &holders {
        assert_eq!(w.obj(x).unwrap().target, None, "dropped at the killing hit");
    }
    w.effects.clear();
    let mut shots = 0;
    for _ in 0..10 {
        step(&mut w);
        shots += w
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::Shot { firer, .. } if holders.contains(firer)))
            .count();
        w.effects.clear();
        if w.obj(u).is_none() {
            break;
        }
        for &x in &holders {
            assert_ne!(w.obj(x).unwrap().target, Some(Target::Object(u)));
        }
    }
    assert_eq!(
        shots, 0,
        "no holder fires at the crashing unit or its place"
    );
}

// spec: S-0502 claim 18 and S-0312 claim 16 (a crash started without a kill ends in the destruction of the unit by damage equal to its health, ignoring defences, with no attacker: no kill is credited)
#[test]
fn claim_18_living_crash_credits_nobody() {
    let mut w = game();
    let u = flier(&mut w);
    let kills = (w.houses[0].combat.kills, w.houses[1].combat.kills);
    assert!(jump::start_crash(&mut w, u, None));
    for _ in 0..80 {
        step(&mut w);
        if w.obj(u).is_none() {
            break;
        }
    }
    assert!(w.obj(u).is_none(), "destroyed and removed");
    assert_eq!(kills, (w.houses[0].combat.kills, w.houses[1].combat.kills));
}

// spec: S-0502 claim 3 and S-0312 claim 16 (safety departure, engine design rule: a pending aerial crash start on a jumpjet hovering with no move order guarantees its next update, so a dead hovering unit does not stay suspended; answers Q-0979)
#[test]
fn claim_3_crash_start_wakes_an_idle_hover() {
    let mut w = game();
    let u = put(&mut w, "JV", Kind::Vehicle, 0, 0, 0);
    jump::start_in_flight(&mut w, u, Phase::Hovering, 500);
    assert!(!jump::moving(&w, u));
    kill(&mut w, u);
    for _ in 0..80 {
        step(&mut w);
        if w.obj(u).is_none() {
            break;
        }
    }
    assert!(w.obj(u).is_none(), "it fell and was removed");
}

// spec: S-0502 claim 19 (an ordinary jumpjet that was shot down is never set sinking by its movement, whatever cell it falls on)
#[test]
fn claim_19_shot_down_over_water_never_sinks() {
    let mut w = game();
    let u = flier(&mut w);
    // Water under the whole flight line.
    let cells: Vec<glasnost_map::CellXY> = (-6..12).map(|x| at(x, 0)).collect();
    kill(&mut w, u);
    for c in cells {
        w.cells.grid.get_mut(c).unwrap().land = glasnost_map::terrain::land::WATER;
    }
    for _ in 0..80 {
        assert!(!glasnost_sim::naval::sinking(&w, u));
        step(&mut w);
        if w.obj(u).is_none() {
            break;
        }
    }
    assert!(w.obj(u).is_none());
}

fn answer(w: &World, firer: ObjectId, target: ObjectId) -> Answer {
    check(w, firer, Some(Target::Object(target)), 0, false)
}

// spec: S-0502 claim 20 and scenario J14 (a balloon-hover vehicle whose weapon lacks fire-while-moving is refused with *moving* in every jumpjet state except grounded and hovering, because the check reads moving now, not a destination)
#[test]
fn claim_20_balloon_hover_firing_states() {
    let mut w = game();
    let t = put(&mut w, "CART", Kind::Vehicle, 1, 2, 0);
    let b = put(&mut w, "BALLOON", Kind::Vehicle, 0, 0, 0);
    // Grounded: no moving refusal, even with a destination.
    go(&mut w, b, at(6, 0));
    assert_ne!(answer(&w, b, t), Answer::Moving, "grounded");
    for (phase, refused) in [
        (Phase::Ascending, true),
        (Phase::Cruising, true),
        (Phase::Descending, true),
        (Phase::Hovering, false),
    ] {
        let mut c = w.clone();
        jump::start_in_flight(&mut c, b, phase, 500);
        assert_eq!(answer(&c, b, t) == Answer::Moving, refused, "{phase:?}");
    }
}

// spec: S-0502 claim 20, S-0310 claim 8 and scenario J14 (an infantry jumpjet with the jumpjet-turn flag is refused while moving now; without the flag only while its speed fraction is above 0.1)
#[test]
fn claim_20_infantry_firing_states() {
    let mut w = game();
    let t = put(&mut w, "CART", Kind::Vehicle, 1, 2, 0);
    let a = put(&mut w, "JJTURN", Kind::Infantry, 0, 0, 0);
    let b = put(&mut w, "JJ", Kind::Infantry, 0, 0, 2);
    for u in [a, b] {
        go(&mut w, u, at(6, 0));
        jump::start_in_flight(&mut w, u, Phase::Ascending, 100);
        js(&mut w, u).fraction = F64::ZERO;
    }
    assert_eq!(answer(&w, a, t), Answer::Moving, "the flag: moving now");
    assert_ne!(answer(&w, b, t), Answer::Moving, "no flag, fraction 0");
    js(&mut w, b).fraction = F64::from_i64(1, w.rules().mode);
    assert_eq!(answer(&w, b, t), Answer::Moving, "fraction above 0.1");
    // Hovering and grounded do not trigger the flight refusal.
    jump::start_in_flight(&mut w, a, Phase::Hovering, 500);
    js(&mut w, a).fraction = F64::ZERO;
    assert_ne!(answer(&w, a, t), Answer::Moving, "hovering");
}

// spec: S-0502 Edge cases and claim 2 (a state that is saved is hashed and restored: a game with jumpjets in every state saves, loads and goes on alike)
#[test]
fn state_is_saved_and_hashed() {
    let mut w = game();
    let a = put(&mut w, "JJ", Kind::Infantry, 0, -8, 0);
    go(&mut w, a, at(8, 0));
    let b = put(&mut w, "JV", Kind::Vehicle, 0, -8, 4);
    jump::start_in_flight(&mut w, b, Phase::Hovering, 500);
    let c = put(&mut w, "JJ", Kind::Infantry, 0, 8, 4);
    go(&mut w, c, at(8, 4));
    jump::start_in_flight(&mut w, c, Phase::Descending, 50);
    testkit::run(&mut w, 30);
    let bytes = glasnost_sim::save::save(&w);
    let mut l = glasnost_sim::save::load(&bytes, w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(
        glasnost_sim::save::state_hash(&l),
        glasnost_sim::save::state_hash(&w)
    );
    assert_eq!(l.jump_slots, w.jump_slots);
    testkit::run(&mut w, 120);
    testkit::run(&mut l, 120);
    assert_eq!(
        glasnost_sim::save::state_hash(&l),
        glasnost_sim::save::state_hash(&w)
    );
    assert!(w.obj(a).is_some_and(|o| o.jump.is_some()));
    // The flight state is part of the hash.
    let mut m = w.clone();
    m.obj_mut(a).unwrap().jump.as_mut().unwrap().ht += 1;
    assert_ne!(
        glasnost_sim::save::state_hash(&m),
        glasnost_sim::save::state_hash(&w)
    );
}
