//! Shrapnel fragments (S-0318): which detonations burst into fragments,
//! how many, where each goes, and what a fragment is.

use glasnost_map::CellXY;
use glasnost_num::F64;
use glasnost_num::geom::{direction, direction_constants};

use crate::combat::data::{ProjectileRules, WeaponId};
use crate::combat::effects::Effect;
use crate::combat::projectile::{Special, angle_facing, d3, initial_velocity};
use crate::combat::range::target_point;
use crate::combat::state::Projectile;
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// The chain bound of our engine: a projectile of chain level d bursts only
/// when d < 16 (S-0318 Edge cases, a provisional design).
// spec: S-0318 Constants (fragment chain bound 16, our engine)
pub const CHAIN_BOUND: u8 = 16;

/// The projectile's attacker, while it still has one (S-0302 claim 1): an
/// attacker removed from the game since the shot is gone.
fn attacker_present(w: &World, pr: &Projectile) -> Option<ObjectId> {
    // spec: S-0318 claims 2 and 3 (the projectile still has its attacker, S-0302 claim 1)
    pr.attacker.filter(|&a| w.obj(a).is_some())
}

/// The fragment count n of F1 (S-0318 claim 2), from the burst point.
pub fn count(w: &World, pr: &Projectile, c: i32, burst: Pos) -> i64 {
    // spec: S-0318 claim 2 and F1 (c ≥ 0: n = c)
    if c >= 0 {
        return i64::from(c);
    }
    match attacker_present(w, pr) {
        Some(a) => {
            // spec: S-0318 F1 (d = ⌊r(Δx² + Δy² + Δz²)⌋ to the attacker's reference point, the table root of S-0204, no building adjustment)
            let d = d3(w, burst, target_point(w, Target::Object(a)));
            // spec: S-0318 F1 (n = |c| − ⌊d / 256⌋)
            i64::from(c.unsigned_abs()) - i64::from(d / 256)
        }
        // spec: S-0318 claim 2 and Constants (the attacker gone: n = 3)
        None => 3,
    }
}

/// Whether the burst cell stops a burst (S-0318 claim 1 b): no first-listed
/// object, or a building first-listed.
fn burst_cell_stops(w: &World, c: CellXY) -> bool {
    // spec: S-0318 Terms and claim 1 (b) (the first-listed object of the burst cell's ground layer, S-0209 claim 13; absent or a building: no burst)
    match w.cells.list(c, Layer::Ground).first() {
        None => true,
        Some(&x) => w.obj(x).is_none_or(|o| o.kind == Kind::Building),
    }
}

/// The candidate of one visited cell (S-0318 claim 3 and Edge cases), or
/// `None` when the cell gives no fragment.
fn candidate(w: &World, c: CellXY, attacker: ObjectId, given: &[ObjectId]) -> Option<ObjectId> {
    let q = w.rules().quirks;
    let view = w.obj(attacker).and_then(|o| o.owner);
    let passes = |x: ObjectId| {
        let Some(o) = w.obj(x) else { return false };
        // spec: S-0318 claim 3 (not the attacker itself)
        if x == attacker {
            return false;
        }
        // spec: S-0318 claim 3 and Terms (not allied from the attacker's house at the burst; an object without an owning house is never allied)
        if w.allied(view, o.owner) {
            return false;
        }
        // spec: S-0318 Edge cases (quirks.shrapnel_hits_terrain off: an object without an owning house is not a candidate)
        if !q.shrapnel_hits_terrain && o.owner.is_none() {
            return false;
        }
        // spec: S-0318 Edge cases (quirks.shrapnel_building_per_cell off: an object already given a fragment in this scan fails the test)
        q.shrapnel_building_per_cell || !given.contains(&x)
    };
    let list = w.cells.list(c, Layer::Ground);
    if q.shrapnel_first_listed_only {
        // spec: S-0318 claim 3 (only the first-listed object of a visited cell can be a candidate)
        list.first().copied().filter(|&x| passes(x))
    } else {
        // spec: S-0318 Edge cases (quirks.shrapnel_first_listed_only off: the first object in listing order that passes the test)
        list.iter().copied().find(|&x| passes(x))
    }
}

/// The fragments of one detonation (S-0318 claims 1 to 8), launched after
/// the warhead's radiation effect and before the area damage (S-0318 State
/// and ordering). `burst` is the burst point (S-0318 Terms), `special` the
/// special hit the detonation selects (S-0514 claim 20).
pub fn burst(
    w: &mut World,
    pr: &Projectile,
    p: &ProjectileRules,
    burst: Pos,
    special: Option<Special>,
) {
    let Some(fw) = p.shrapnel_weapon else {
        // spec: S-0318 Terms (a projectile type without a fragment weapon launches no fragments)
        return;
    };
    // spec: S-0318 Edge cases (our engine: a projectile of chain level d bursts only when d < 16)
    if pr.chain >= CHAIN_BOUND {
        return;
    }
    // spec: S-0318 claim 1 (a) (a selected special hit stops the burst)
    if special.is_some() {
        return;
    }
    let bc = burst.cell();
    // spec: S-0318 claim 1 (b)
    if burst_cell_stops(w, bc) {
        return;
    }
    let n = count(w, pr, p.shrapnel_count, burst);
    // spec: S-0318 claim 1 (c) (n ≤ 0: no burst, no draw)
    if n <= 0 {
        return;
    }
    let rules = w.rules_arc();
    let q = rules.quirks;
    let half = q.ranged_draw_half_span;
    let attacker = attacker_present(w, pr);
    let mut launched: i64 = 0;
    if let Some(a) = attacker {
        // spec: S-0318 F2 (k = ⌊R / 256⌋, truncated toward zero) and Edge cases (rings 1 to min(k, 11), a departure for k > 11)
        let k = (rules.combat.weapon(fw).range / 256).min(11);
        let mut given: Vec<ObjectId> = Vec::new();
        // spec: S-0318 claim 3 (rings 1 to k in S-0303 table T1's scan order, ring 0 not visited; quirks.spread_ring_eleven governs ring 11)
        'scan: for r in 1..=k {
            for (dx, dy) in crate::combat::area::ring(r, q.spread_ring_eleven) {
                if launched >= n {
                    break 'scan;
                }
                let c = CellXY::new(bc.x + dx, bc.y + dy);
                if let Some(x) = candidate(w, c, a, &given) {
                    // spec: S-0318 claim 3 (each candidate receives one fragment, at once and in visiting order)
                    fragment(w, pr, fw, burst, Target::Object(x), Some(a));
                    given.push(x);
                    launched += 1;
                }
            }
        }
    }
    // spec: S-0318 claim 4 (each remaining fragment: Δx = R(−2, 2), then Δy = R(−2, 2), aimed at that cell itself)
    while launched < n {
        let dx = w.rng.ranged(-2, 2, half);
        let dy = w.rng.ranged(-2, 2, half);
        let c = CellXY::new(bc.x + dx, bc.y + dy);
        fragment(w, pr, fw, burst, Target::Cell(c), attacker);
        launched += 1;
    }
}

/// Launches one fragment of `fw` from the burst point at `target` (S-0318
/// claims 5, 6 and 8). `attacker` is the projectile's attacker while it is
/// still present: only then does the fragment make a beam or a bolt.
fn fragment(
    w: &mut World,
    pr: &Projectile,
    fw: WeaponId,
    burst: Pos,
    target: Target,
    attacker: Option<ObjectId>,
) {
    let rules = w.rules_arc();
    let mode = rules.mode;
    let wr = rules.combat.weapon(fw);
    let Some(pid) = wr.projectile else {
        // A weapon without a projectile launches nothing (the importer warns).
        return;
    };
    let fp = &rules.combat.projectiles[pid as usize];
    // spec: S-0318 r27 claim 6 (towards the aim point's reference point; the direction is taken from the burst point as it is, not from the raised starting position) and S-0306 claims 3 and 5 (revision 38: a fragment is excepted from the shot's aim point, lead and launch direction)
    let tp = target_point(w, target);
    let yaw = i32::from(direction(
        i64::from(tp.x - burst.x),
        i64::from(tp.y - burst.y),
        rules.quirks.table_direction,
    ));
    // spec: S-0318 claim 6 and Constants (a fixed elevation of about an eighth of a turn, π/4) and S-0306 claim 6 (revision 38: every projectile except a fragment takes its pitch from (a) to (c); a fragment climbs at this fixed elevation instead)
    let (c1, _) = direction_constants();
    // spec: S-0318 claim 6 and Constants (π/4: half of S-0204 F3's c₁ = π/2)
    let pitch = angle_facing(c1.div(F64::from_i64(2, mode), mode), mode);
    // spec: S-0318 claim 5 (the speed is the fragment weapon's weapon speed S_w of S-0306 claim 1, without the shot speed or its cap) and claim 6 (length S_w, 100 at a speed of 0); S-0306 claims 1, 2 and 7 and Terms (revision 38: a fragment is not a shot, its speed S_w, uncapped; a steering or vertical fragment's start speed 1 is left open there, so S_w is kept)
    let speed = wr.speed;
    let mut vel = initial_velocity(w, yaw, pitch, speed);
    // spec: S-0306 claim 9 (the stored target point K; the arm time: 0 against an aircraft)
    let stored = tp;
    let arm = if matches!(target, Target::Object(t) if w.obj(t).is_some_and(|x| x.kind == Kind::Aircraft))
    {
        0
    } else {
        fp.arm
    };
    // spec: S-0318 r27 claims 1 and 5 (placed at the burst point, raised in height only, as S-0306 claim 8 gives, when it is not above the terrain) and S-0306 claims 5 and 8 (revision 38: a fragment starts at the burst point, not the fire point; revision 45: raised to 1 lepton above the terrain when the burst point is not above it, S-0306 Changes for other specs)
    let mut pos = crate::combat::projectile::start_point(w, burst);
    if fp.inviso {
        // spec: S-0318 claims 5 and 6 and S-0306 claim 9 (an invisible fragment is moved at once to K, or the first blocking cell's centre; no launch scatter draw, S-0318 Open questions)
        pos = match crate::combat::lof::walk(w, fp, pr.house, burst, stored) {
            Some(c) => {
                let cc = Pos::centre(c, 0);
                Pos::new(cc.x, cc.y, w.surface_at(cc.x, cc.y))
            }
            None => stored,
        };
        // spec: S-0318 claim 6 and S-0306 claim 9 (moved at once: no velocity in any of the three axes)
        vel = [F64::ZERO; 3];
    }
    let id = w.new_id();
    let frag = Projectile {
        id,
        weapon: fw,
        // spec: S-0318 claim 5 (the projectile's attacker as its attacker)
        attacker: pr.attacker,
        house: pr.house,
        firer: pr.firer,
        firer_house: pr.firer_house,
        pos,
        vel,
        target: Some(target),
        stored,
        placed_at: pos,
        arm,
        placed: w.tick,
        // spec: S-0318 claim 5 (the fragment weapon's damage field, unscaled)
        damage: wr.damage,
        // spec: S-0306 T1 (revision 45: Ω starts as the full spatial distance from the start point of claim 8, or an invisible fragment's moved position, to K)
        omega: d3(w, pos, stored),
        top_speed: speed,
        turns: 0,
        unlocked: 0,
        locked: fp.course_lock != 1,
        closing: F64::ZERO,
        prev_cell: pos.cell(),
        // spec: S-0306 claims 8 and 22 (revision 45: the line-of-fire walk's X₁, the burst point as it is, not raised)
        fire_point: burst,
        nuke_wait: None,
        vehicle_scalable: false,
        airburst_child: false,
        multiplier: 256, // spec: S-0518 claim 9 (every projectile starts with the multiplier 256)
        // spec: S-0318 Terms (a fragment of a projectile of level d has level d + 1)
        chain: pr.chain + 1,
        looped: false,
    };
    w.projectiles.insert(id, frag);
    // spec: S-0318 claim 5 and S-0306 claim 8 (the fragment enters play at once; its first turn is in the tick of the burst) and S-0203 claim 5 (revision 40: unless removals cost it that turn, then in the first later tick in which none does; S-0306 State and ordering, revision 38: n + 1 or later)
    w.update.enter_new(id);
    let Some(a) = attacker else {
        // spec: S-0318 claim 8 (a fragment launched after the attacker has gone makes no beam and no bolt)
        return;
    };
    // spec: S-0318 claim 8 (the beam and the bolt end at the fragment's target: an object's target point, a cell's centre at its height)
    let to = target_point(w, target);
    // spec: S-0318 claim 8 (the laser and electric-bolt flags are tested separately for each fragment)
    if wr.laser {
        w.effects.push(Effect::FragmentLine {
            attacker: a,
            weapon: fw,
            from: burst,
            to,
        });
    }
    if wr.electric_bolt {
        // spec: S-0316 claim 8 (a building attacker's fragment bolt takes the building-source adjustment from the burst point; never attached)
        let building = w.obj(a).filter(|o| o.kind == Kind::Building).map(|o| o.pos);
        w.effects.push(Effect::Bolt {
            firer: a,
            weapon: fw,
            slot: 0,
            burst: 0,
            from: burst,
            to,
            building,
            vehicle_on_map: false,
        });
        // spec: S-0318 claim 8 and S-0316 claim 8 (each fragment's bolt has its spark system at the bolt end)
        crate::emitters::electric_bolt_sparks(w, to);
    }
}
