//! S-0613 revision 24, claim 19, on a high bridge (#424, sim batch S28): gas
//! made on and under a deck, with claims 5, 9 to 11 and 13, F4 and F5,
//! against measurement M-0026's deck and under rows. The synthetic bridge
//! and rules are `r24_deck_common`'s; `quirks.gas_through_bridge_deck` is
//! at its default, on.

mod r24_deck_common;

use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::gas::{self, Particle};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{Layer, ObjectId, Pos, World};
use r24_deck_common::*;

fn shared(w: &World) -> ObjectId {
    w.gas.shared.expect("a shared gas system")
}

fn list(w: &World) -> Vec<Particle> {
    w.gas.systems[&shared(w)].particles.clone()
}

/// Adds one FOG particle to the shared system at `at` (as a middle event
/// does) and returns its index in the list.
fn puff(w: &mut World, at: Pos) -> usize {
    let ti = w
        .rules()
        .gas
        .particles
        .iter()
        .position(|p| p.name == "FOG")
        .unwrap();
    let draw = w.rules().gas.particles[ti].draw;
    gas::middle_event_particle(w, draw, Some((at.x, at.y, at.z)));
    list(w).len() - 1
}

/// One update of the shared system in the tick after the current one,
/// with particle `i` held at the centre of the test cell (no drift, so it
/// stays in its cell; the ground is flat, so its height rules read the
/// same terrain everywhere). Returns its height and vertical speed after
/// the update and whether the update hit `ground` and `deck`.
fn update(w: &mut World, i: usize, ground: ObjectId, deck: ObjectId) -> (i32, i32, bool, bool) {
    let s = shared(w);
    let at = Pos::centre(test_cell(), 0);
    {
        let p = &mut w.gas.systems.get_mut(&s).unwrap().particles[i];
        p.vel[0] = 0;
        p.vel[1] = 0;
        p.pos = Pos::new(at.x, at.y, p.pos.z);
    }
    w.tick += 1;
    w.effects.clear();
    gas::system_turn(w, s);
    let mut hit = (false, false);
    for e in &w.effects {
        if let Effect::GasHit { id, .. } = e {
            hit.0 |= *id == ground;
            hit.1 |= *id == deck;
        }
    }
    let p = &list(w)[i];
    (p.pos.z, p.vel[2], hit.0, hit.1)
}

/// F4 and F5 over flat ground at terrain height 0, with no wind and no
/// deflection: the height and vertical speed after the update on tick `t`.
fn f4_f5(z: i32, vz: i32, t: i32) -> (i32, i32) {
    if t % 2 == 0 {
        // spec: S-0613 F4 (even ticks: v_z ← max(v_z − 1, −5) when a > 5, else max(v_z, 0)); F5 (even ticks: the height is unchanged)
        let vz = if z > 5 { (vz - 1).max(-5) } else { vz.max(0) };
        (z, vz)
    } else {
        // spec: S-0613 F5 (odd ticks: z₁ = z − clamp(z − h₁ − 5, 0, 2), z′ = max(z₁ + v_z, H + 5))
        let z1 = z - (z - 5).clamp(0, 2);
        ((z1 + vz).max(5), vz)
    }
}

/// A world with the bridge, a crusher of the other house on the ground
/// under the test cell and another on the deck there, with the current
/// tick odd, so that a particle made now has its first update, in the
/// next tick, on an even tick.
fn gas_world() -> (World, ObjectId, ObjectId) {
    let mut w = game();
    bridge(&mut w);
    let ground = testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", THEIRS, test_cell(), 0);
    let deck = testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", THEIRS, east_cell(), 0);
    w.cells.leave(east_cell(), Layer::Ground, deck);
    w.cells.enter(test_cell(), Layer::Deck, deck, false);
    {
        let o = w.obj_mut(deck).unwrap();
        o.layer = Layer::Deck;
        o.pos = Pos::centre(test_cell(), DECK);
    }
    if w.tick % 2 == 0 {
        testkit::run(&mut w, 1);
    }
    (w, ground, deck)
}

/// spec: S-0613 claim 19 (a gas particle made at a high bridge's deck height with a vertical speed of zero sinks through the deck: below the deck's height after its first odd-tick move, then toward 5 leptons above the terrain under the bridge, by claims 9 and 11, F4 and F5; its hits use the ground layer, so an object on the deck is not hit) and claim 13 (the hits list the ground layer of its cell)
/// measure: M-0026 (deck setup, first update on an even tick: −3 from the deck after the first odd-tick move; 7 leptons per two ticks once the vertical speed is −5; 122 ticks from creation to 5 above the ground with the deck 416 above it; 5 above the ground from then on; no row differs from F4 and F5; hits fall due in the cell with B there, and B on the deck is never hit)
#[test]
fn claim_19_gas_made_on_the_deck_follows_m0026() {
    let (mut w, ground, deck) = gas_world();
    let created = w.tick;
    assert_eq!((created + 1) % 2, 0, "the first update is on an even tick");
    let i = puff(&mut w, Pos::centre(test_cell(), DECK));
    let p = list(&w)[i].clone();
    assert_eq!(
        p.pos,
        Pos::centre(test_cell(), DECK),
        "made at the deck height"
    );
    assert_eq!(p.vel[2], 0, "vertical speed 0 at creation");
    let (mut z, mut vz) = (DECK, 0);
    // (ticks since creation, height, vertical speed) after each update.
    let mut rows = Vec::new();
    let mut ground_hits = 0;
    let mut deck_hits = 0;
    for _ in 0..300 {
        let (hz, hvz, g, d) = update(&mut w, i, ground, deck);
        (z, vz) = f4_f5(z, vz, w.tick);
        assert_eq!((hz, hvz), (z, vz), "tick {}: F4 and F5", w.tick);
        rows.push((w.tick - created, hz, hvz));
        ground_hits += i32::from(g);
        deck_hits += i32::from(d);
    }
    // The first odd-tick move is the second update.
    assert_eq!(rows[1].1, DECK - 3, "−3 from the deck");
    // From one odd-tick move to the next, once the vertical speed is −5 and
    // before the floor.
    let fast: Vec<i32> = rows
        .windows(3)
        .filter(|x| x[0].0 % 2 == 0 && x[0].2 == -5 && x[2].1 > 5)
        .map(|x| x[0].1 - x[2].1)
        .collect();
    assert!(!fast.is_empty());
    assert!(fast.iter().all(|&d| d == 7), "7 per two ticks: {fast:?}");
    let settled = rows.iter().find(|x| x.1 == 5).map(|x| x.0);
    assert_eq!(settled, Some(122), "122 ticks to 5 above the ground");
    assert!(
        rows.iter().filter(|x| x.0 >= 122).all(|x| x.1 == 5),
        "5 above the ground from then on"
    );
    assert!(ground_hits > 0, "hits fell due in the cell");
    assert_eq!(deck_hits, 0, "the object on the deck is never hit");
}

/// spec: S-0613 claim 19 (r24: a gas particle made by claim 5 on the ground under a high bridge, with vertical speed zero, ignores the deck: it settles 5 leptons above the terrain there, stays under the deck, and hits the ground layer of its cell, never the objects on the deck, claims 9, 11 and 13) and claim 5 (a height at or below H is raised to H)
/// measure: M-0026 (under setup: made at the ground; 5 above the ground after the first odd-tick move and to the end of the run; no row differs from F4 and F5; hits fall due in the cell with B on the deck above, and B is never hit)
#[test]
fn claim_19_gas_made_under_the_deck_stays_under_it() {
    let (mut w, ground, deck) = gas_world();
    let i = puff(&mut w, Pos::centre(test_cell(), 0));
    assert_eq!(list(&w)[i].pos.z, 0, "made at the ground");
    assert_eq!(list(&w)[i].vel[2], 0, "vertical speed 0 at creation");
    let (mut z, mut vz) = (0, 0);
    let mut ground_hits = 0;
    let mut deck_hits = 0;
    for n in 1..=300 {
        let (hz, hvz, g, d) = update(&mut w, i, ground, deck);
        (z, vz) = f4_f5(z, vz, w.tick);
        assert_eq!((hz, hvz), (z, vz), "tick {}: F4 and F5", w.tick);
        if n >= 2 {
            assert_eq!(hz, 5, "5 above the ground from the first odd-tick move");
        }
        ground_hits += i32::from(g);
        deck_hits += i32::from(d);
    }
    assert!(ground_hits > 0, "it hits the ground layer of its cell");
    assert_eq!(deck_hits, 0, "never the object on the deck");
}

/// spec: S-0613 claims 2 and 19 (the clouds of an infantry unit that dies a virus death on the deck start at its position, its height included, with a vertical speed of zero)
/// measure: M-0026 (deck setup: each particle's position at its creation tick equals the dead unit's last position, at the deck height; vertical speed 0 at the creation tick)
#[test]
fn claim_19_a_virus_death_on_the_deck_makes_its_cloud_at_the_deck_height() {
    let mut w = game();
    bridge(&mut w);
    let a = testkit::spawn(&mut w, Kind::Infantry, "MAN", THEIRS, test_cell(), 0);
    onto_deck(&mut w, a);
    let last = w.obj(a).unwrap().pos;
    assert_eq!(last.z, DECK);
    let venom = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "VENOM")
        .unwrap() as u32;
    damage::apply(
        &mut w,
        a,
        Hit {
            amount: 10_000,
            warhead: Some(venom),
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
    let mut made = Vec::new();
    for _ in 0..60 {
        let before = list(&w).len();
        testkit::run(&mut w, 1);
        made.extend(list(&w)[before..].iter().cloned());
        if !made.is_empty() {
            break;
        }
    }
    assert_eq!(made.len(), 3, "the middle event's particles");
    for p in &made {
        assert_eq!(p.pos, last, "at the dead unit's last position");
        assert_eq!(p.vel[2], 0, "vertical speed 0");
    }
}
