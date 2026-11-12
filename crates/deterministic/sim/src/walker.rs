//! The burning walker (S-0615 claims 4 to 11): a burning-infantry death
//! animation whose type has the burning-walker flag. It runs from cell to
//! cell, preferring water to the south-east, for up to eight destinations,
//! then plays its dying frames and is removed.
//!
//! A walker takes its turns in the update order (S-0203), like any
//! animation (S-0109 claim 5). Its stage is frozen while it runs, so it has
//! no middle event and delivers damage over time only while it expires.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_num::geom::{direction, root_floor};
use serde::{Deserialize, Serialize};

use crate::combat::area::{self, Detonation};
use crate::combat::effects::Effect;
use crate::regions::STEPS;
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Pos, World};

/// A burning walker in play.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Walker {
    /// The animation type, by registration index.
    pub anim: u32,
    /// Its position.
    pub pos: Pos,
    /// Its walker destination (claim 6: a cell centre at height 0), or none.
    pub dest: Option<Pos>,
    /// The retry count (claim 9).
    pub retries: i32,
    /// It is expiring (claim 11).
    pub expiring: bool,
    /// Its stage (frozen at 0 while it runs, claim 4).
    pub stage: i32,
    /// It is falling (claim 10).
    pub falling: bool,
    /// Its fall rate *v* (S-0511 F3), leptons per update.
    pub rate: i32,
    /// The damage accumulator *A* of S-0109 F5.
    pub acc: F64,
}

/// The height level, leptons (glossary).
const LEVEL: i32 = 104; // spec: S-0615 Constants (height level 104 leptons)
/// The climb limit, leptons (claim 7).
const CLIMB: i32 = 208; // spec: S-0615 Constants (walker climb limit 208 leptons)
/// The deck lift, leptons (claim 8).
const DECK_LIFT: i32 = 416; // spec: S-0615 Constants (walker deck lift 416 leptons)
/// The nominal step, leptons per update (claim 8).
const STEP: i32 = 18; // spec: S-0615 Constants (walker step 18 leptons per update)
/// The arrival radius, leptons (claim 8).
const ARRIVAL: u64 = 19; // spec: S-0615 Constants (walker arrival radius 19 leptons)
/// The retry limit (claim 9).
const RETRY_LIMIT: i32 = 7; // spec: S-0615 Constants (walker retry limit 7)
/// The search start score (F2).
const NO_SCORE: i32 = 10000; // spec: S-0615 F2 and Constants (search start score 10000)
/// The south-east bias (F2).
const BIAS: i32 = 3; // spec: S-0615 F2 and Constants (south-east bias 3 cells of score)
/// The frame period, ticks per running frame (F1).
const FRAME_PERIOD: i32 = 3; // spec: S-0615 F1 and Constants (frame period 3 ticks)
/// The splash's height above the walker, leptons (claim 10).
const SPLASH_RISE: i32 = 3; // spec: S-0615 claim 10 (the splash at its position plus 3 leptons of height)

/// Creates a burning walker of animation type `anim` at `pos` (claims 2 and
/// 4): its creation and start-event draws (S-0202 claim 15, S-0109 claim
/// 13), no destination, a place at the end of the update order.
pub fn create(w: &mut World, anim: u32, pos: Pos) {
    let rules = w.rules_arc();
    let Some(a) = rules.animations.get(anim as usize) else {
        return;
    };
    // spec: S-0615 claim 2 (creation and immediate start-event effects have their usual conditional draws; the stage is frozen, so no later middle event, claim 4)
    crate::load::anim_created(w, &a.name, Some((pos.x, pos.y, pos.z)), true);
    // spec: S-0109 claim 17 and F5 (A₀ = 1.0; quirks.anim_damage_starts_at_one off: 0.0)
    let acc = if rules.quirks.anim_damage_starts_at_one {
        F64::from_i64(1, rules.mode)
    } else {
        F64::ZERO
    };
    let id = w.new_id();
    w.walkers.insert(
        id,
        Walker {
            anim,
            pos,
            dest: None,
            retries: 0,
            expiring: false,
            stage: 0,
            falling: false,
            // spec: S-0511 claim 10 (a fall rate starts from the value held: 0 for an object that has never fallen)
            rate: 0,
            acc,
        },
    );
    w.effects.push(Effect::Walker {
        id,
        name: a.name.clone(),
        pos,
        made: true,
    });
    // spec: S-0109 claim 5 (created during the turn pass: its first turn comes in that tick after every object already in the order)
    w.update.enter(id);
}

/// The image frame a walker shows (F1 while it runs; its stage while it
/// expires), with `r` the running frames *R* and `tick` the tick number.
pub fn frame(wk: &Walker, r: i32, tick: i32, table: bool) -> i32 {
    if wk.expiring {
        return wk.stage;
    }
    // spec: S-0615 Open questions (a walker needs a positive running-frame count; our engine shows frame 0, the importer warns)
    if r <= 0 {
        return 0;
    }
    // spec: S-0615 F1 (d = 7 with no destination)
    let d = wk.dest.map_or(7, |p| {
        let f = direction(i64::from(p.x - wk.pos.x), i64::from(p.y - wk.pos.y), table);
        // spec: S-0615 F1 (d = ⌊(⌊f / 4096⌋ + 1) / 2⌋ mod 8)
        ((i32::from(f) / 4096 + 1) / 2).rem_euclid(8)
    });
    // spec: S-0615 F1 (frame = R × (7 − d) + (⌊t / 3⌋ mod R))
    r * (7 - d) + (tick / FRAME_PERIOD).rem_euclid(r)
}

/// The cell next to `c` in step direction `d` (S-0204 Constants).
fn neighbour(c: CellXY, d: usize) -> CellXY {
    let (dx, dy) = STEPS[d % 8]; // spec: S-0204 Constants (eight step directions)
    CellXY::new(c.x + dx, c.y + dy)
}

/// Whether a cell layer holds a vehicle, aircraft or building (claim 7:
/// infantry does not block).
fn blocked(w: &World, c: CellXY, l: Layer) -> bool {
    w.cells.list(c, l).iter().any(|&x| {
        w.obj(x)
            .is_some_and(|o| matches!(o.kind, Kind::Vehicle | Kind::Aircraft | Kind::Building))
    })
}

/// Whether a cell is passable for a walker at height `z` (claim 7).
pub fn passable(w: &World, c: CellXY, z: i32) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let l = i32::from(cell.level);
    // spec: S-0615 claim 7 (on a bridge deck when the cell carries one and z − 104 L > 208: passable when the deck layer holds no vehicle, aircraft or building)
    if w.high_bridge(c) && z - LEVEL * l > CLIMB {
        return !blocked(w, c, Layer::Deck);
    }
    // spec: S-0615 claim 7 (otherwise: land not rock, tunnel or wall; no wall overlay; no vehicle, aircraft or building in the ground layer; 104 L − z ≤ 208)
    !matches!(cell.land, land::ROCK | land::TUNNEL | land::WALL)
        && !crate::walls::is_wall(w, cell.overlay)
        && !blocked(w, c, Layer::Ground)
        && LEVEL * l - z <= CLIMB
}

/// The water search of claim 5 and F2 from cell `c0`: the best cell.
pub fn water_search(w: &World, c0: CellXY) -> Option<CellXY> {
    let r = w.rules();
    let mut best = None;
    let mut score = NO_SCORE;
    // spec: S-0615 claim 5 (x₀ − 5 ≤ x ≤ x₀ + 4 and y₀ − 5 ≤ y ≤ y₀ + 4; x outer, y inner, each low to high)
    for x in c0.x - 5..=c0.x + 4 {
        for y in c0.y - 5..=c0.y + 4 {
            let c = CellXY::new(x, y);
            // spec: S-0615 claim 5 (candidates: usable, water, no bridge deck, no deck at the neighbours in step directions 6 and 0)
            let candidate = w.cells.usable(c)
                && w.cells.cell(c).is_some_and(|k| k.land == land::WATER)
                && !w.high_bridge(c)
                && !w.high_bridge(neighbour(c, 6))
                && !w.high_bridge(neighbour(c, 0));
            if !candidate {
                continue;
            }
            let (dx, dy) = (x - c0.x, y - c0.y);
            let s = if dx == 0 && dy == 0 {
                // spec: S-0615 F2 (score 0 for the walker's own cell)
                0
            } else {
                let n = u64::from((dx * dx + dy * dy).unsigned_abs());
                // spec: S-0615 F2 (trunc(√ₜ(Δx² + Δy²)), the table root of S-0204 F1)
                let root = root_floor(n, r.quirks.table_root_distance, r.mode) as i32;
                // spec: S-0615 F2 (3 less when Δx > 0 or Δy > 0)
                root - if dx > 0 || dy > 0 { BIAS } else { 0 }
            };
            // spec: S-0615 F2 (the lowest score wins; on a tie the first in scan order)
            if s < score {
                score = s;
                best = Some(c);
            }
        }
    }
    best
}

/// Chooses a destination from the walker's cell (claims 5 and 6), taking
/// the fallback's ranged draw when it comes to that.
pub fn choose(w: &mut World, pos: Pos) -> Option<Pos> {
    let c0 = pos.cell();
    let z = pos.z;
    if let Some(b) = water_search(w, c0) {
        let (sx, sy) = ((b.x - c0.x).signum(), (b.y - c0.y).signum());
        // spec: S-0615 claim 6 (the diagonal step towards the best cell, then x only, then y only; the first passable)
        let steps = [
            CellXY::new(c0.x + sx, c0.y + sy),
            CellXY::new(c0.x + sx, c0.y),
            CellXY::new(c0.x, c0.y + sy),
        ];
        if let Some(&c) = steps.iter().find(|&&c| passable(w, c, z)) {
            // spec: S-0615 claim 6 (the destination is the cell's centre at height 0)
            return Some(Pos::centre(c, 0));
        }
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0615 claim 6 (the fallback: one ranged draw r in [0, 7], the destination choice's only one)
    let r = w.rng.ranged(0, 7, half);
    // spec: S-0615 claim 6 (the neighbours in cyclic step-direction order r, r + 1, …, r + 7 mod 8; the first passable; none otherwise)
    (0..8)
        .map(|i| neighbour(c0, (r + i).rem_euclid(8) as usize))
        .find(|&c| passable(w, c, z))
        .map(|c| Pos::centre(c, 0))
}

/// Claim 8's height rule for a walker that was at height `old` and is now
/// at (x, y).
fn height(w: &World, x: i32, y: i32, old: i32) -> i32 {
    let t = w.surface_at(x, y);
    // spec: S-0615 claim 8 (the terrain height there, or 416 above it when the old height was more than 208 above it)
    if old - t > CLIMB { t + DECK_LIFT } else { t }
}

/// Claim 10's test after a step or a placement: start falling. A fall
/// that starts here has its first descent in the same update, straight
/// after the step or placement: from rate 0 its height changes by 0 and
/// only its rate follows S-0511's recurrence.
fn fall_check(w: &mut World, id: ObjectId, wk: &mut Walker) {
    if wk.falling {
        return;
    }
    let c = wk.pos.cell();
    let t = w.surface_at(wk.pos.x, wk.pos.y);
    // spec: S-0615 claim 10 (at least 416 above the terrain, and no deck at its cell nor at its neighbours in step directions 6 and 0)
    if wk.pos.z - t >= DECK_LIFT
        && !w.high_bridge(c)
        && !w.high_bridge(neighbour(c, 6))
        && !w.high_bridge(neighbour(c, 0))
    {
        wk.falling = true;
        // spec: S-0615 claim 10 (First descent: the descent processing runs in the same update as the step or placement that started the fall, straight after it; Starting rate: 0, the rate held since creation)
        descend(w, id, wk);
    }
}

/// Claim 11's removal test: an expiring walker whose stage is at least
/// ⌊*N* / 2⌋ − 1 is removed. Returns whether it was.
fn removal_test(w: &mut World, id: ObjectId, wk: &Walker) -> bool {
    let n = w
        .rules()
        .animations
        .get(wk.anim as usize)
        .and_then(|a| a.frames)
        // spec: S-0615 claim 11 and Import mapping (Unknown frame count: N = 0, as S-0109 F2 for a type with no sprite, so the threshold is −1; a departure with no switch, the importer warns)
        .unwrap_or(0);
    // spec: S-0615 claim 11 (an expiring walker whose stage is at least ⌊N / 2⌋ − 1 is removed; Short sprites: the formula stands)
    if wk.stage >= n / 2 - 1 {
        remove(w, id);
        return true;
    }
    false
}

/// Starts expiry (claim 11): stage 8 *R* + 1; from claims 4 or 9 the
/// destination is cleared.
fn expire(w: &World, wk: &mut Walker, clear: bool) {
    let r = w
        .rules()
        .animations
        .get(wk.anim as usize)
        .map_or(0, |a| a.running_frames);
    wk.expiring = true;
    // spec: S-0615 claim 11 (the stage is set to 8R + 1)
    wk.stage = 8 * r + 1;
    if clear {
        // spec: S-0615 claim 11 (expiry from claims 4 or 9 clears the destination; on landing it can be kept)
        wk.dest = None;
    }
}

/// The horizontal table-root distance of S-0204 claim 5.
fn distance(w: &World, a: Pos, b: Pos) -> u64 {
    let r = w.rules();
    let (dx, dy) = (i64::from(a.x - b.x), i64::from(a.y - b.y));
    root_floor(
        (dx * dx + dy * dy).unsigned_abs(),
        r.quirks.table_root_distance,
        r.mode,
    )
}

/// A running step of claim 8 and F3 towards `d`: an 18-lepton step and
/// the height rule.
fn run(w: &World, wk: &mut Walker, d: Pos) {
    let r = w.rules();
    let q = r.quirks;
    // spec: S-0615 F3 (f, the direction from the walker's position to its destination, S-0204 F3)
    let f = direction(
        i64::from(d.x - wk.pos.x),
        i64::from(d.y - wk.pos.y),
        q.table_direction,
    );
    // spec: S-0615 F3 (signed facing, θ = (s − 16383) × (−2π / 65534), x′ = trunc(x + 18 cos θ), y′ = trunc(y − 18 sin θ); S-0208 F3's step with s = 18)
    // spec: S-0105 F1 (quirks.table_trig_negative_bias)
    let (x, y) = crate::walk::step_point_b(
        wk.pos.x,
        wk.pos.y,
        f,
        STEP,
        true,
        r.mode,
        q.table_trig_negative_bias,
    );
    let z = height(w, x, y, wk.pos.z);
    wk.pos = Pos::new(x, y, z);
}

/// Removes a walker (claim 11).
fn remove(w: &mut World, id: ObjectId) {
    let Some(wk) = w.walkers.remove(&id) else {
        return;
    };
    let name = w
        .rules()
        .animations
        .get(wk.anim as usize)
        .map_or_else(String::new, |a| a.name.clone());
    w.effects.push(Effect::Walker {
        id,
        name,
        pos: wk.pos,
        made: false,
    });
    let skip = w.rules().quirks.update_order_removal_skip;
    // spec: S-0203 claim 10 (a removal from the update order, counting during the turn pass)
    w.update.leave(id, skip);
}

/// The damage over time of one expiring advance (S-0109 claim 17 and F5):
/// δ added to the accumulator; its whole part, when 1 or more, delivered
/// at the walker's position with the game-wide flame warhead.
fn advance_damage(w: &mut World, wk: &mut Walker) {
    let rules = w.rules_arc();
    let Some(a) = rules.animations.get(wk.anim as usize) else {
        return;
    };
    let mode = rules.mode;
    // spec: S-0109 claim 17 (a type with δ > 0 that is not bouncing)
    if a.bouncing || !a.damage.gt(F64::ZERO) {
        return;
    }
    let one = F64::from_i64(1, mode);
    // spec: S-0109 F5 (B = A + cδ, c = 1 for an unattached animation; binary64 toward zero)
    let b = wk.acc.add(a.damage, mode);
    if !b.ge(one) {
        wk.acc = b;
        return;
    }
    // spec: S-0109 F5 (n = trunc(B) delivered, A = B − n)
    let n = b.to_i32_common();
    wk.acc = b.sub(F64::from_i64(i64::from(n), mode), mode);
    // spec: S-0109 claim 17 and Import mapping (C4Warhead for the built-in invisibility-animation type by the current type's name, FlameDamage2 for every other type; absent: the amount is taken, no damage)
    let warhead = crate::anims::damage_warhead(&rules, a);
    // spec: S-0302 claim 1 (a damaging animation's area detonation: no attacker, no attacking house)
    // spec: S-0109 claim 17 (each delivery is an area detonation with resource reduction allowed, no attacker and no attacking house, the expiring advances of a burning walker included; S-0303 claim 17's gates)
    area::detonate(
        w,
        Detonation {
            point: wk.pos,
            damage: n,
            warhead,
            source: None,
            house: None,
            resources: true,
        },
    );
}

/// A burning walker's turn (claims 4 to 11; the state table).
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(mut wk) = w.walkers.get(&id).cloned() else {
        return;
    };
    if wk.expiring {
        // spec: S-0615 claim 11 (at the start of each update the removal test; Removed before advancing: no advance and no damage over time in the removing update)
        if removal_test(w, id, &wk) {
            return;
        }
        // spec: S-0615 claim 11 (a one-update stage delay: one stage per update, the first on a later update than expiry's start)
        wk.stage += 1;
        // spec: S-0615 claim 11 (expiring advances deliver ordinary damage over time, S-0109 claim 17)
        advance_damage(w, &mut wk);
        w.walkers.insert(id, wk);
        return;
    }
    if wk.falling {
        // spec: S-0615 claim 10 (while falling it still takes running steps, with claim 8's height placement before descent, but handles no arrival and makes no choice)
        if let Some(d) = wk.dest
            && distance(w, wk.pos, d) >= ARRIVAL
        {
            run(w, &mut wk, d);
        }
        descend(w, id, &mut wk);
        w.walkers.insert(id, wk);
        return;
    }
    let Some(d) = wk.dest else {
        // spec: S-0615 claim 4 (the first update chooses without the retry and water tests and does not move; none: expiry at once)
        match choose(w, wk.pos) {
            Some(p) => wk.dest = Some(p),
            None => {
                expire(w, &mut wk, true);
                // spec: S-0615 claim 11 (the removal test is also made at the end of the update in which a first-update choice of none started expiry)
                if removal_test(w, id, &wk) {
                    return;
                }
            }
        }
        w.walkers.insert(id, wk);
        return;
    };
    // spec: S-0615 claim 8 (a step when the horizontal table-root distance is 19 leptons or more)
    if distance(w, wk.pos, d) >= ARRIVAL {
        run(w, &mut wk, d);
        fall_check(w, id, &mut wk);
        w.walkers.insert(id, wk);
        return;
    }
    // spec: S-0615 claim 9 (on arrival: choose a new destination from the cell it is in first)
    let next = choose(w, wk.pos);
    let old_land = w.cells.cell(d.cell()).map(|c| c.land);
    let above = wk.pos.z - w.surface_at(wk.pos.x, wk.pos.y);
    // spec: S-0615 claim 9 (expire when the new destination is none, the retry count is 7 or more, or the old destination's cell is water or beach and the height above the terrain is at most 104)
    let wet = matches!(old_land, Some(land::WATER | land::BEACH)) && above <= LEVEL;
    match next {
        Some(p) if wk.retries < RETRY_LIMIT && !wet => {
            // spec: S-0615 claim 9 (retry count + 1; placed exactly on the old destination with claim 8's height rule; the new destination becomes current)
            wk.retries += 1;
            let z = height(w, d.x, d.y, wk.pos.z);
            wk.pos = Pos::new(d.x, d.y, z);
            wk.dest = Some(p);
            fall_check(w, id, &mut wk);
        }
        _ => expire(w, &mut wk, true),
    }
    w.walkers.insert(id, wk);
}

/// One descent of a falling walker (claim 10; S-0511 claim 10 and F3):
/// the height changes by the rate, the rate follows F3, and a height less
/// than 1 lepton above the terrain lands. Landing starts expiry; on water
/// it also makes the first splash animation 3 leptons above.
fn descend(w: &mut World, id: ObjectId, wk: &mut Walker) {
    let v = wk.rate;
    // spec: S-0511 F3 (h_{n+1} = h_n + v_n, then v_{n+1} = max(trunc(v_n − 1.4), v_min))
    wk.pos.z += v;
    wk.rate = crate::fall::next_rate(w, v);
    let ground = w.surface_at(wk.pos.x, wk.pos.y);
    // spec: S-0511 claim 10 (a height above the terrain of 0 or less after the move: height 0, the fall ends)
    if wk.pos.z - ground >= 1 {
        return;
    }
    wk.pos.z = ground;
    wk.falling = false;
    // spec: S-0615 claim 10 (landing starts expiry, which can keep the destination)
    expire(w, wk, false);
    let water = w
        .cells
        .cell(wk.pos.cell())
        .is_some_and(|c| c.land == land::WATER);
    if water && let Some(name) = w.rules().combat.general.splash_list.first().cloned() {
        // spec: S-0615 claim 10 (landing on water creates the first game-wide splash animation at its position plus 3 leptons of height)
        let at = Pos::new(wk.pos.x, wk.pos.y, wk.pos.z + SPLASH_RISE);
        crate::combat::effects::anim(w, &name, at);
    }
    let _ = id;
}
