//! Particle systems of the smoke, fire, spark and railgun kinds (S-0616).
//!
//! The systems live with the gas systems in [`crate::gas::GasWorld`]; a
//! system's turn dispatches on its kind ([`crate::gas::system_turn`]), and
//! each particle updates and moves by its own kind (claim 3), whichever
//! system holds it. Only what changes play is kept in the simulation state:
//! the speed *S*, the flight direction *u*, a spark's velocity, a fire
//! particle's step, the emission timer *T*, the spark counter *K*, the target
//! point and the owner object, and the state the drawing reads: each
//! particle's translucency, colour state, creation position and target
//! point (claims 26 and 29, F15, F16). Every draw that S-0616 lists is
//! taken; the light, helix and laser values are discarded.
//!
//! Creators built here: a warhead's system ([`crate::gas::warhead_system`]),
//! the damage smoke of S-0302 claim 13 ([`damage_smoke`]), the overload
//! sparks of S-0601 claim 16 and the drone delivery sparks of S-0614 claim
//! 9 ([`spark_system`]). Claim 1's other creators, the emitter places and
//! S-0301 claim 5's gate are in [`crate::emitters`]; the flame sweep F5,
//! swing F6 and insertion F7 of a fire emitter are here ([`fire_turn`]).

use glasnost_num::{F32, F64, Rounding};

use crate::gas::{Behaviour, Particle, ParticleSystem, ParticleType};
use crate::rules::Kind;
use crate::world::{HouseId, Layer, ObjectId, Pos, World};

/// The smoke wind steps (Δₓ, Δᵧ) by direction 0 … 7 (F2).
// spec: S-0616 Constants (smoke wind steps Δx 0, 2, 2, 2, 0, −2, −2, −2 and Δy −2, −2, 0, 2, 2, 2, 0, −2)
pub const SMOKE_WIND: [(i32, i32); 8] = [
    (0, -2),
    (2, -2),
    (2, 0),
    (2, 2),
    (0, 2),
    (-2, 2),
    (-2, 0),
    (-2, -2),
];

/// The smoke velocity limit, leptons per tick (claim 6).
const SMOKE_LIMIT: i32 = 5; // spec: S-0616 Constants (smoke velocity limit ±5 leptons per tick)

/// The smoke emission height, leptons (F4).
const SMOKE_EMIT_HEIGHT: i32 = 10; // spec: S-0616 Constants (smoke emission height 10 leptons)

/// The bridge deck height above the terrain, leptons (F2, F11, claim 14).
const DECK: i32 = 416; // spec: S-0616 Constants (bridge deck height 416 leptons)

/// The smoke deck margin below the deck, leptons (F2).
const SMOKE_DECK_MARGIN: i32 = 260; // spec: S-0616 Constants (smoke deck margin 260 leptons)

/// The successor radius divisor (F3).
const RADIUS_DIVISOR: i32 = 8; // spec: S-0616 Constants (successor radius divisor 8)

/// The fire distance divisor (F9).
const FIRE_DISTANCE_DIVISOR: i32 = 10; // spec: S-0616 Constants (fire distance divisor 10)

/// The height level step, leptons (claim 14: height level × 104).
const LEVEL: i32 = 104; // spec: S-0616 claim 14 (the cell's ground level is its height level times 104 leptons)

/// The spark wall and building height above *H*, leptons (F11).
const SPARK_BAND: i32 = 150; // spec: S-0616 Constants (spark wall and building height 150 leptons)

/// The spark under-deck placement, leptons (F11).
const SPARK_UNDER_DECK: i32 = 20; // spec: S-0616 Constants (spark under-deck placement 20 leptons)

/// The spark ground snap depth, leptons (F11).
const SPARK_SNAP: i32 = 100; // spec: S-0616 Constants (spark ground snap depth 100 leptons)

/// The top of a ranged draw over 0 … 2147483646 (claims 1, 15 to 18).
const DRAW_TOP: i32 = 2147483646; // spec: S-0616 claims 15 to 18 (ranged draws R(0, 2147483646))

/// The smoke translucency step, percentage points (claims 8, 9 and 26).
const SMOKE_TRANSLUCENCY_STEP: u8 = 25; // spec: S-0616 Constants (smoke translucency step 25)

/// The fire translucency values its stage fields set (claim 26).
const FIRE_TRANSLUCENCY: (u8, u8) = (25, 50); // spec: S-0616 claim 26 (the translucent-25 stage sets 25, the translucent-50 stage sets 50)

/// Railgun line draws before a particle's creation draws (claim 17).
const RAIL_BEFORE: usize = 3; // spec: S-0616 claim 17 (three ranged draws, then the creation draws)

/// Railgun line draws after a particle's creation draws (claim 17).
const RAIL_AFTER: usize = 4; // spec: S-0616 claim 17 (then four more ranged draws)

/// Spark burst raw values taken and discarded after the count (F10).
const SPARK_DISCARDED: usize = 3; // spec: S-0616 F10 (three unused raw values follow the count)

/// The most particles one spark burst or one railgun line makes (#198):
/// a particle cap or a particles-per-lepton field from rules data could
/// otherwise ask for billions in one turn. Counts up to it are unchanged:
/// the count raw value and the three discarded values are still taken, and
/// each particle made keeps its own draws.
pub const BURST_CEILING: i32 = 4096; // spec: S-0616 r29 claim 20a

fn mode(w: &World) -> Rounding {
    w.rules().mode
}

fn d(v: i64, m: Rounding) -> F64 {
    F64::from_i64(v, m)
}

#[cfg(test)]
fn c(text: &str) -> F64 {
    crate::rules::nearest_double(text)
}

/// ⌊x⌋ of a finite double as an integer (0 for no integer form).
fn floor_i32(x: F64) -> i32 {
    let t = x.to_i32_common();
    if x.lt(F64::from_i64(i64::from(t), Rounding::Nearest)) {
        t.wrapping_sub(1)
    } else {
        t
    }
}

/// The system type of system `sys`, borrowed from `r` (the world's rules,
/// held as `World::rules_arc` so the borrow outlives changes to the world).
fn sys_type<'r>(
    r: &'r crate::rules::SimRules,
    w: &World,
    sys: ObjectId,
) -> Option<&'r crate::gas::SystemType> {
    let s = w.gas.systems.get(&sys)?;
    r.gas.systems.get(s.type_index as usize)
}

/// The particle type of `p`, borrowed from `r` as in [`sys_type`].
fn ptype<'r>(r: &'r crate::rules::SimRules, p: &Particle) -> Option<&'r ParticleType> {
    r.gas.particles.get(p.type_index as usize)
}

/// Marks a system to end (S-0613 claim 17, S-0616 Terms *marked*).
pub fn mark(w: &mut World, sys: ObjectId) {
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        s.ending = true;
    }
}

// ---------------------------------------------------------------------------
// Creation (claim 1, F1, F13)

/// F1: the flight direction from `at` to `target`.
// nocite: vector and matrix dimensions (three axes)
pub fn heading(w: &World, at: Pos, target: Pos) -> [F32; 3] {
    let m = mode(w);
    let v = [target.x - at.x, target.y - at.y, target.z - at.z];
    // spec: S-0616 F1 (v = Q − P per component in single precision; m = r(vₓ² + vᵧ² + v_z²), the sum in double precision)
    let vs = v.map(|x| d(i64::from(x), m).to_f32(m));
    let sum = vs
        .iter()
        .fold(d(0, m), |a, x| a.add(x.to_f64().mul(x.to_f64(), m), m));
    let table = w.rules().quirks.table_root_distance;
    let root = glasnost_num::geom::table_root_real(sum, table, m);
    if root.eq_num(d(0, m)) {
        // spec: S-0616 F1 (u = (0, 0, 0) when m = 0)
        return [F32(0); 3]; // nocite: vector and matrix dimensions (three axes)
    }
    // spec: S-0616 F1 (u = v / m in single precision)
    vs.map(|x| x.to_f64().div(root, m).to_f32(m))
}

/// Claim 19, F13: a particle's own stage-advance value.
// nocite: vector and matrix dimensions (three axes)
pub fn own_advance(w: &World, t: &ParticleType, at: Pos, target: Pos, u: [F32; 3], s: F32) -> i8 {
    if !t.normalized {
        // spec: S-0616 claim 19 (the type's stage-advance field)
        return t.advance;
    }
    let m = mode(w);
    let single = |x: F64| x.to_f32(m).to_f64();
    // spec: S-0616 F13 (aₓ = |trunc(uₓ × S)|, a_y likewise, single precision)
    let a = |ui: F32| {
        let prod = single(ui.to_f64().mul(s.to_f64(), m));
        d(i64::from(prod.to_i32_common()), m).abs()
    };
    let (ax, ay) = (a(u[0]), a(u[1]));
    let threshold = crate::consts::MILLIONTH; // spec: S-0616 Constants (normalised speed threshold 10⁻⁶)
    let none = d(9999, m); // spec: S-0616 Constants (normalised no-motion time 9999.0 ticks)
    let dist = |p: i32, q: i32| single(d(i64::from(p), m).sub(d(i64::from(q), m), m).abs());
    // spec: S-0616 F13 (tₓ = fl(|Pₓ − Qₓ| / aₓ) when aₓ > 10⁻⁶; t_y likewise; t from the four cases)
    let tx = ax
        .gt(threshold)
        .then(|| single(dist(at.x, target.x).div(ax, m)));
    let ty = ay
        .gt(threshold)
        .then(|| single(dist(at.y, target.y).div(ay, m)));
    let t_ = match (tx, ty) {
        (None, None) => none,
        (Some(x), None) => x,
        (None, Some(y)) => y.min(none),
        (Some(x), Some(y)) => x.min(y),
    };
    // spec: S-0616 F13 (value = trunc(t / (δ_F + 1) + 1.0), single precision; a quotient with no integer form gives 0; reduced modulo 256 into −128 … 127)
    let q = single(t_.div(d(i64::from(t.final_stage) + 1, m), m));
    let v = single(q.add(d(1, m), m));
    v.to_i32_common() as i8
}

/// A new system of type `ti` at `pos`, entered in the update order.
pub(crate) fn create(w: &mut World, ti: u32, pos: Pos, house: Option<HouseId>) -> Option<ObjectId> {
    let rules = w.rules_arc();
    let t = rules.gas.systems.get(ti as usize)?;
    let id = w.new_id();
    w.gas
        .systems
        .insert(id, ParticleSystem::new(ti, t, pos, house, rules.mode));
    // spec: S-0616 claim 1 (making a system takes no draw) and S-0203 claim 2 (a new object enters the update order)
    w.update.enter(id);
    Some(id)
}

/// One system of the game-wide spark type at `pos`, with no owner object,
/// no owner house and no first particle (claim 1: the overload and drone
/// delivery rows). Without a listed game-wide spark type nothing is made.
pub fn spark_system(w: &mut World, pos: Pos) {
    let Some(ti) = w.rules().gas.default_spark else {
        if w.rules().gas.default_spark_given {
            w.note("S-0616 Import mapping: DefaultSparkSystem names no listed particle-system type; no spark system is made");
        }
        return;
    };
    // spec: S-0616 claim 1 (overload and drone delivery: no owner object, target point unused, no first particle)
    create(w, ti, pos, None);
}

/// The damage smoke of S-0302 claim 13 (S-0616 claim 1): a system of the
/// picked type named `name` at the object's position plus `offset`, owned
/// by the object and held in its damage-smoke place.
// nocite: vector and matrix dimensions (three axes)
pub fn damage_smoke(w: &mut World, id: ObjectId, name: &str, offset: [i32; 3]) {
    let Some(ti) = w.rules().gas.system(name) else {
        w.note(format!(
            "S-0302 claim 13: the damage particle system {name} is not a listed type; no system is made"
        ));
        return;
    };
    let Some(o) = w.obj(id) else { return };
    // spec: S-0616 claim 1 (damage smoke position: the object's position plus the type's damage-smoke displacement, used as given)
    let pos = Pos::new(
        o.pos.x.wrapping_add(offset[0]),
        o.pos.y.wrapping_add(offset[1]),
        o.pos.z.wrapping_add(offset[2]),
    );
    let Some(sys) = create(w, ti, pos, None) else {
        return;
    };
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        // spec: S-0616 claim 1 (owner object the damaged object; target point the world origin, unused; no first particle)
        s.owner_object = Some(id);
        s.owner_offset = offset;
    }
    if let Some(ob) = w.obj_mut(id) {
        ob.combat.damage_smoke = Some(sys);
    }
}

/// Claim 4: object `id` is removed. Every system it owns loses its owner
/// object and is marked.
pub fn owner_removed(w: &mut World, id: ObjectId) {
    for s in w.gas.systems.values_mut() {
        if s.owner_object == Some(id) {
            // spec: S-0616 claim 4 (every system whose owner object is removed loses it and is marked, whatever its kind)
            s.owner_object = None;
            s.ending = true;
        }
        if s.target_object == Some(id) {
            // spec: S-0616 claim 4 (a removed object is cleared from every system that names it as its target object)
            s.target_object = None;
        }
    }
}

// ---------------------------------------------------------------------------
// One particle (claim 3)

/// Claim 3: particle `i` of system `sys` updates by its own kind, then its
/// lifetime goes down by 1 modulo 65536 and 0 gives it the dead mark.
pub fn update_one(w: &mut World, sys: ObjectId, i: usize) {
    let Some(p) = crate::gas::particle(w, sys, i) else {
        return;
    };
    let kind = ptype(w.rules(), &p).and_then(|t| t.kind);
    // spec: S-0616 claim 3 (the particle's own behaviour kind, not its system's, chooses the update; a particle with no kind has none)
    match kind {
        Some(Behaviour::Gas) => crate::gas::gas_update(w, sys, i),
        Some(Behaviour::Smoke) => smoke_update(w, sys, i),
        Some(Behaviour::Fire) => fire_update(w, sys, i),
        Some(Behaviour::Spark) => spark_update(w, sys, i),
        Some(Behaviour::Railgun) => railgun_update(w, sys, i),
        None => {}
    }
    let Some(mut p) = crate::gas::particle(w, sys, i) else {
        return;
    };
    // spec: S-0616 claim 3 and S-0613 claim 6 (then the remaining lifetime goes down by 1 modulo 65536; 0 gives the dead mark)
    p.life = p.life.wrapping_sub(1);
    if p.life == 0 {
        p.dead = true;
    }
    crate::gas::put(w, sys, i, p);
}

/// Claim 3: a live particle's move by its own kind in a turn that moves
/// (gas, smoke and fire turns). Returns `true` when a smoke particle is
/// stopped at a bridge deck (claim 7).
pub fn move_one(w: &World, p: &mut Particle) -> bool {
    let Some(t) = ptype(w.rules(), p) else {
        return false;
    };
    // spec: S-0616 claim 3 (gas: S-0613 F5; smoke: claim 7; fire: claim 13; spark, railgun and no kind: no move here)
    match t.kind {
        Some(Behaviour::Gas) => {
            crate::gas::move_particle(w, t, p);
            false
        }
        Some(Behaviour::Smoke) => smoke_move(w, t, p),
        Some(Behaviour::Fire) => {
            fire_move(w, p);
            false
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Smoke (claims 5 to 9)

/// Claim 5: a smoke system's turn.
pub fn smoke_turn(w: &mut World, sys: ObjectId) {
    // spec: S-0616 claim 5 (the position follows an owner object that is not a building, at the displacement it had at creation)
    let owner = w
        .gas
        .systems
        .get(&sys)
        .and_then(|s| s.owner_object.map(|o| (o, s.owner_offset)));
    // spec: S-0317 claim 13 (a debris piece as owner object: the system follows the piece's position)
    let followed = owner.and_then(|(o, off)| match w.obj(o) {
        Some(ob) if ob.kind != Kind::Building => Some((ob.pos, off)),
        Some(_) => None,
        None => w.pieces.get(&o).map(|p| (p.pos, off)),
    });
    if let Some((op, off)) = followed {
        let at = Pos::new(
            op.x.wrapping_add(off[0]),
            op.y.wrapping_add(off[1]),
            op.z.wrapping_add(off[2]), // nocite: vector and matrix dimensions (three axes)
        );
        if let Some(s) = w.gas.systems.get_mut(&sys) {
            s.pos = at;
        }
    }
    // spec: S-0616 claim 5 (every particle takes its update draws, oldest first)
    let n = w.gas.systems.get(&sys).map_or(0, |s| s.particles.len());
    for i in 0..n {
        update_one(w, sys, i);
    }
    // spec: S-0616 claim 5 (the removal pass, newest first, before any move: a dead particle splits into successors, claim 8, and is removed) and claim 8 (revision 27: also in a marked system, so the pass never reads the mark)
    removal_pass(w, sys);
    // spec: S-0616 claims 5 and 7 (then each particle without the dead mark moves)
    let Some(s) = w.gas.systems.get(&sys) else {
        return;
    };
    let mut list = s.particles.clone();
    let mut stopped = Vec::new();
    for (i, p) in list.iter_mut().enumerate() {
        if !p.dead && move_one(w, p) {
            stopped.push(i);
        }
    }
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        s.particles = list;
    }
    if !w.rules().quirks.smoke_deck_extra_update && !stopped.is_empty() {
        // spec: S-0616 Edge cases (quirks.smoke_deck_extra_update off: a particle stopped at a deck is removed, with its successors, in the pass that stops it)
        for &i in stopped.iter().rev() {
            split_and_remove(w, sys, i);
        }
    }
    smoke_emission(w, sys);
}

/// The removal pass of claim 5: newest first, each particle with the dead
/// mark at the start of the pass splits (claim 8) and is removed.
fn removal_pass(w: &mut World, sys: ObjectId) {
    let Some(s) = w.gas.systems.get(&sys) else {
        return;
    };
    let dead: Vec<usize> = s
        .particles
        .iter()
        .enumerate()
        .filter(|(_, p)| p.dead)
        .map(|(i, _)| i)
        .collect();
    for i in dead.into_iter().rev() {
        split_and_remove(w, sys, i);
    }
}

/// Claim 8: particle `i` of a smoke system splits into two successors
/// when its type names a next type, whatever its own kind (claim 3), then
/// it is removed. The successors go at the end of the list.
fn split_and_remove(w: &mut World, sys: ObjectId, i: usize) {
    let Some(p) = crate::gas::particle(w, sys, i) else {
        return;
    };
    // spec: S-0616 claims 3 and 8 (successors follow the system's kind, not the particle's: a dead particle of any kind in a smoke system whose type names a next type splits, d taken from the dead particle's type)
    let marked = w
        .gas
        .systems
        .get(&sys)
        .is_some_and(|s| s.looped.contains(&p.id));
    // spec: S-0616 r29 claim 8a (a marked successor does not split, and takes no draw, through a next-type link on a loop of particle types)
    let rules = w.rules_arc();
    if let Some(t) = ptype(&rules, &p)
        && let Some(next) = t.next
        && let Some(looped) =
            w.rules()
                .loops
                .particles
                .link(Some(p.type_index as usize), marked, Some(next as usize))
    {
        // spec: S-0616 F3 (d = ⌊ρ / 8⌋, rounding down; q = r rem d; o = q + d when q ≥ 1, else q − d; S-0205 claim 3 for d = 0)
        let dv = t.radius.div_euclid(RADIUS_DIVISOR);
        let offset = |r: i32| {
            let q = glasnost_num::int::rem(r, dv);
            if q >= 1 {
                q.wrapping_add(dv)
            } else {
                q.wrapping_sub(dv)
            }
        };
        // spec: S-0616 claim 8 (two raw values give oₓ, then o_y)
        let ox = offset(w.rng.raw() as i32);
        let oy = offset(w.rng.raw() as i32);
        let origin = Pos::default();
        for sign in [1, -1] {
            // spec: S-0616 claim 8 (successors at (x ± oₓ, y ± o_y, z), target point (0, 0, 0), with their creation draws)
            let at = Pos::new(
                p.pos.x.wrapping_add(sign * ox),
                p.pos.y.wrapping_add(sign * oy),
                p.pos.z,
            );
            let added = crate::gas::add_particle(w, sys, next, at, origin);
            let made = added.is_some();
            if looped
                && let Some(q) = added
                && let Some(s) = w.gas.systems.get_mut(&sys)
            {
                s.looped.insert(q);
            }
            if made
                && let Some(q) = w
                    .gas
                    .systems
                    .get_mut(&sys)
                    .and_then(|s| s.particles.last_mut())
            {
                // spec: S-0616 claim 8 (each successor's speed is the dead particle's; its direction and velocity are its own)
                q.speed = p.speed;
            }
            // spec: S-0616 claim 8 (then one raw value r for its translucency)
            let r = w.rng.raw() as i32;
            if made
                && let Some(q) = w
                    .gas
                    .systems
                    .get_mut(&sys)
                    .and_then(|s| s.particles.last_mut())
            {
                q.translucency = successor_translucency(p.translucency, r);
            }
        }
    }
    // spec: S-0616 claim 8 (the dead particle is removed after both successors)
    if let Some(s) = w.gas.systems.get_mut(&sys)
        && i < s.particles.len()
    {
        s.particles.remove(i);
        s.looped.remove(&p.id);
    }
}

/// The translucency of a successor made in a smoke system (claims 8 and
/// 26) whose dead particle had `parent`, with its translucency raw value
/// `r`.
pub fn successor_translucency(parent: u8, r: i32) -> u8 {
    // spec: S-0616 claims 8 and 26 (the dead particle's value replaces the type's, plus 25 when r rem 6 ≠ 0, kept modulo 256)
    if glasnost_num::int::rem(r, 6) != 0 {
        parent.wrapping_add(SMOKE_TRANSLUCENCY_STEP)
    } else {
        parent
    }
}

/// Claim 6: a smoke particle's update.
fn smoke_update(w: &mut World, sys: ObjectId, i: usize) {
    let Some(mut p) = crate::gas::particle(w, sys, i) else {
        return;
    };
    let rules = w.rules_arc();
    let Some(t) = ptype(&rules, &p) else { return };
    let m = mode(w);
    // spec: S-0616 claim 6 (draws on odd ticks only)
    if w.tick.rem_euclid(2) == 1 {
        let r1 = w.rng.raw() as i32;
        // spec: S-0616 claim 6 and Formulas (|r| wraps at −2³¹; |r₁| a multiple of 4: two more raw values)
        if glasnost_num::int::rem(r1.wrapping_abs(), 4) == 0 {
            let r2 = w.rng.raw() as i32;
            let r3 = w.rng.raw() as i32;
            // spec: S-0616 claim 6 (the change (|r₃| rem 3) − 1; −3 for r₃ = −2³¹)
            let k = glasnost_num::int::rem(r3.wrapping_abs(), 3) - 1;
            // spec: S-0616 claim 6 (the parity of |r₂|: even x, odd y)
            if r2.wrapping_abs() & 1 == 0 {
                p.vel[0] = p.vel[0].wrapping_add(k);
            } else {
                p.vel[1] = p.vel[1].wrapping_add(k);
            }
        }
        // spec: S-0616 claim 6 (both horizontal components limited to −5 … 5; the vertical velocity set to 0 on every odd tick)
        p.vel[0] = p.vel[0].clamp(-SMOKE_LIMIT, SMOKE_LIMIT);
        p.vel[1] = p.vel[1].clamp(-SMOKE_LIMIT, SMOKE_LIMIT);
        p.vel[2] = 0;
    }
    // spec: S-0616 claim 6 (below the end stage the stage advances by S-0613 F8 with the type's field; at the end stage with the delete flag the dead mark; never past it)
    if p.stage < t.end_stage && crate::gas::stage_due(t.max_ec, t.advance, &p) {
        p.stage = p.stage.wrapping_add(1);
    }
    if p.stage == t.end_stage && t.delete_at_limit {
        p.dead = true;
    }
    // spec: S-0616 claim 6 and Constants (when S > 3.0, S goes down by the slowing field, single precision)
    if p.speed.to_f64().gt(d(3, m)) {
        p.speed = p.speed.to_f64().sub(t.deacc.to_f64(), m).to_f32(m);
    }
    crate::gas::put(w, sys, i, p);
}

/// Claim 7, F2: a live smoke particle's move. Returns `true` when it is
/// stopped at a bridge deck (and marked dead).
fn smoke_move(w: &World, t: &ParticleType, p: &mut Particle) -> bool {
    let (x, y, z) = (p.pos.x, p.pos.y, p.pos.z);
    // spec: S-0616 claim 7 and F2 (smoke's own wind table; a wind effect of 0 is unaffected; outside 0 … 7 no push, a departure)
    let (dx, dy) = if t.wind_effect == 0 {
        (0, 0)
    } else {
        w.rules().gas.smoke_wind.unwrap_or((0, 0))
    };
    // spec: S-0616 F2 (x′ = x + Δₓ(ω) × W + vₓ, y′ likewise, z′ = z + trunc(S) + v_z, integer arithmetic)
    let x2 = x
        .wrapping_add(dx.wrapping_mul(t.wind_effect))
        .wrapping_add(p.vel[0]);
    let y2 = y
        .wrapping_add(dy.wrapping_mul(t.wind_effect))
        .wrapping_add(p.vel[1]);
    let z2 = z
        .wrapping_add(p.speed.to_f64().to_i32_common())
        .wrapping_add(p.vel[2]); // nocite: vector and matrix dimensions (three axes)
    let h0 = w.surface_at(x, y);
    // spec: S-0616 F2 (stopped and marked when the cell at (x, y) is a bridge cell, z < H₀ + 416 and z′ ≥ H₀ + 416 − 260)
    if w.high_bridge(p.pos.cell()) && z < h0 + DECK && z2 >= h0 + DECK - SMOKE_DECK_MARGIN {
        p.dead = true;
        return true;
    }
    p.pos = Pos::new(x2, y2, z2);
    false
}

/// Claim 9, F4: a smoke system's emission and timer growth.
fn smoke_emission(w: &mut World, sys: ObjectId) {
    let rules = w.rules_arc();
    let Some(t) = sys_type(&rules, w, sys) else {
        return;
    };
    let Some(s) = w.gas.systems.get(&sys) else {
        return;
    };
    // The system as its emission starts.
    let (s_timer, s_ending, s_len, s_pos, s_target) =
        (s.timer, s.ending, s.particles.len(), s.pos, s.target);
    let m = mode(w);
    let tick = w.tick;
    // spec: S-0616 claim 9 (an owner that is an infantry, vehicle or aircraft object inside a tunnel stops emission; tunnels, S-0212, are not built, so no owner is inside one)
    let owner_ok = true;
    // spec: S-0616 claim 9 (not marked, and the tick number a multiple of ⌊T⌋; S-0205 claim 3 for ⌊T⌋ = 0, claim 20)
    let period = floor_i32(s_timer.to_f64());
    if !s_ending && owner_ok && glasnost_num::int::rem(tick, period) == 0 {
        // spec: S-0616 claim 9 and F4 (two raw values r₁, r₂, taken even without a held type; D = σ + 1, S-0205 claim 3 for D = 0)
        let dd = t.spawn_radius.wrapping_add(1);
        let r1 = w.rng.raw() as i32;
        let r2 = w.rng.raw() as i32;
        let capped = !w.rules().quirks.smoke_ignores_particle_cap
            && i32::try_from(s_len).unwrap_or(i32::MAX) >= t.cap;
        if let Some(held) = t.holds
            && !capped
        {
            // spec: S-0616 F4 (at (xₛ + (r₁ rem D), yₛ + (r₂ rem D), zₛ + 10), with the terrain floor; its target point the system's)
            let at = Pos::new(
                s_pos.x.wrapping_add(glasnost_num::int::rem(r1, dd)),
                s_pos.y.wrapping_add(glasnost_num::int::rem(r2, dd)),
                s_pos.z.wrapping_add(SMOKE_EMIT_HEIGHT),
            );
            if crate::gas::add_particle(w, sys, held, at, s_target).is_some()
                && let Some(p) = w
                    .gas
                    .systems
                    .get_mut(&sys)
                    .and_then(|x| x.particles.last_mut())
            {
                // spec: S-0616 F4 and Constants (S = S − (T − T₀) × 0.35 in double precision, stored in single, raised to 2.0 below 2.0)
                let loss = s_timer
                    .to_f64()
                    .sub(d(i64::from(t.spawn_frames), m), m)
                    .mul(crate::consts::SMOKE_SPEED_LOSS, m);
                let mut v = p.speed.to_f64().sub(loss, m).to_f32(m);
                // spec: S-0616 F4 and Constants (raised to 2.0 below 2.0)
                if v.to_f64().lt(d(2, m)) {
                    v = d(2, m).to_f32(m); // spec: S-0616 F4 and Constants (raised to 2.0)
                }
                p.speed = v;
                // spec: S-0616 claim 9, F4 and claim 26 (its translucency rises by 25 when T > τ, compared in single precision)
                if s_timer.to_f64().gt(t.translucency_cutoff.to_f64()) {
                    p.translucency = p.translucency.wrapping_add(SMOKE_TRANSLUCENCY_STEP);
                }
            }
        }
    }
    // spec: S-0616 claim 9 (after the emission step T grows by the slow-down field, single precision; above the spawn cut-off the system is marked)
    if let Some(x) = w.gas.systems.get_mut(&sys) {
        x.timer = x.timer.to_f64().add(t.slowdown.to_f64(), m).to_f32(m);
        if x.timer.to_f64().gt(t.cutoff.to_f64()) {
            x.ending = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Fire (claims 10 to 14)

/// Claim 10: a fire system's turn.
pub fn fire_turn(w: &mut World, sys: ObjectId) {
    // spec: S-0616 claim 10 (every particle takes its update draws, newest first, then moves)
    let n = w.gas.systems.get(&sys).map_or(0, |s| s.particles.len());
    for i in (0..n).rev() {
        update_one(w, sys, i);
    }
    let Some(s) = w.gas.systems.get(&sys) else {
        return;
    };
    let mut list = s.particles.clone();
    for p in list.iter_mut() {
        // spec: S-0616 claim 10 (a particle marked dead in its update still moves in that turn)
        move_one(w, p);
    }
    // spec: S-0616 claim 10 (particles with the dead mark are removed in a second pass, before the emission; no successors)
    list.retain(|p| !p.dead);
    let owner = w.gas.systems.get(&sys).and_then(|s| s.owner_object);
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        s.particles = list;
    }
    // spec: S-0616 claim 10 (no owner object, one that is not an infantry, vehicle, aircraft or building object, or one no longer alive: marked)
    let alive = owner.and_then(|o| w.obj(o)).is_some_and(|o| {
        matches!(
            o.kind,
            Kind::Infantry | Kind::Vehicle | Kind::Aircraft | Kind::Building
        ) && o.health > 0
    });
    if !alive {
        mark(w, sys);
        if let Some(o) = owner
            && crate::emitters::held(w, o, crate::emitters::Place::Fire) == Some(sys)
        {
            // spec: S-0616 claim 1 (a fire emitter is marked and let go when its firer dies) and S-0301 claim 5, place-and-kind rule (revision 63: only the fire place is let go; a fire-kind system held in the spark or railgun place is marked by its own turn and frees that place when it leaves play)
            crate::emitters::let_go_fire(w, o);
        }
    }
    let mut swept = false;
    if alive && let Some(o) = owner {
        // The retarget let-go of S-0616 claim 1 is not made here but at the
        // change of the owner's target (`crate::emitters::retarget_let_go`).
        // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: the test is made at the change of the firer's target and not in the turn of the system the fire place holds, for every kind) and S-0616 claim 10 (the fire turn's owner step has no retarget test)
        // spec: S-0616 Terms (a system with an owner object takes the owner's current target as its target object)
        let to = match w.obj(o).and_then(|x| x.target) {
            Some(crate::world::Target::Object(x)) if w.obj(x).is_some() => Some(x),
            _ => None,
        };
        if let Some(s) = w.gas.systems.get_mut(&sys) {
            s.target_object = to;
        }
        let turning = w.obj(o).is_some_and(|x| x.body.turning(w.tick));
        if let Some(g) = to
            && turning
        {
            // spec: S-0616 claim 10 (an alive owner with a target object whose body facing is turning makes a flame sweep, before the emission)
            flame_sweep(w, sys, o, g);
            swept = true;
        }
    }
    let Some(s) = w.gas.systems.get(&sys) else {
        return;
    };
    if s.ending {
        // spec: S-0616 claim 10 (the emission takes place unless the system is marked)
        return;
    }
    let Some(spawn_frames) = sys_type(w.rules(), w, sys).map(|t| t.spawn_frames) else {
        return;
    };
    let tick = w.tick;
    // spec: S-0616 claim 11 (emits when the tick is a multiple of the spawn interval, or a multiple of 3 in a turn that made a sweep)
    let due = (spawn_frames != 0 && glasnost_num::int::rem(tick, spawn_frames) == 0)
        || (swept && glasnost_num::int::rem(tick, SWEEP_CADENCE) == 0);
    if due {
        fire_emission(w, sys);
    }
}

/// The sweep emission cadence, ticks (claim 11).
const SWEEP_CADENCE: i32 = 3; // spec: S-0616 claim 11 (a multiple of 3 when this same turn made a sweep)

/// The fire swing period, ticks (F6).
const SWING_PERIOD: i32 = 500; // spec: S-0616 F6 (k = t rem 500; a table of 500 entries)

/// The fire swing near distance, leptons (F6).
const SWING_NEAR: i64 = 200; // spec: S-0616 F6 (m = 3 when ℓ < 200.0, otherwise 1)

/// The fire insertion window (F7).
const INSERT_WINDOW: usize = 4; // spec: S-0616 F7 (c = min(4, N))

/// The fire side table *e*ₓ, *e*ᵧ by δ = 0 … 7 (F6).
// spec: S-0616 Constants (fire side table eₓ 1, 1, 0, −1, −1, −1, 0, 1 and eᵧ 0, 1, 1, 1, 0, −1, −1, −1)
const SIDE: [(i32, i32); 8] = [
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
    (0, -1),
    (1, -1),
];

/// Claim 11, F5: the flame sweep of fire system `sys` with owner `o` and
/// its target object `g`.
pub fn flame_sweep(w: &mut World, sys: ObjectId, o: ObjectId, g: ObjectId) {
    let (Some(ob), Some(gb)) = (w.obj(o), w.obj(g)) else {
        return;
    };
    let m = mode(w);
    let (op, gp) = (ob.pos, gb.pos);
    // spec: S-0616 F5 (f from 32768 up is taken as f − 65536)
    let f = i64::from(ob.body.value(w.tick) as i16);
    let table = w.rules().quirks.table_root_distance;
    // spec: S-0616 F5 (d = trunc(r((Gₓ − Oₓ)² + (Gᵧ − Oᵧ)² + (G_z − O_z)²)), the table root of S-0204 F1)
    let sum = [gp.x - op.x, gp.y - op.y, gp.z - op.z]
        .iter()
        .fold(d(0, m), |a, &v| {
            let x = d(i64::from(v), m);
            a.add(x.mul(x, m), m)
        });
    let dist = d(
        i64::from(glasnost_num::geom::table_root_real(sum, table, m).to_i32_common()),
        m,
    );
    // spec: S-0616 F5 and Edge cases (θ = (f − 16383) × (−2π / 65534) in double precision; quirks.flame_sweep_facing off: 16384 and 65536)
    let (quarter, full) = if w.rules().quirks.flame_sweep_facing {
        (16383, 65534)
    } else {
        (16384, 65536)
    };
    let theta = d(f - quarter, m).mul(crate::consts::TWO_PI.negated().div(d(full, m), m), m);
    let (cos, sin) = (
        crate::table_trig::cos_q(w, theta, m),
        crate::table_trig::sin_q(w, theta, m),
    );
    // spec: S-0616 F5 (target point = (trunc(Oₓ + cosₜ(θ) × d), trunc(Oᵧ − sinₜ(θ) × d), O_z), double precision throughout)
    let tx = d(i64::from(op.x), m)
        .add(cos.mul(dist, m), m)
        .to_i32_common();
    let ty = d(i64::from(op.y), m)
        .sub(sin.mul(dist, m), m)
        .to_i32_common();
    // spec: S-0616 claim 11 (the system's position becomes the owner's fire point for its first weapon, with no extra displacement; where the fire point alternates between bursts, the burst counter is the owner's current one at the sweep, not the shot's and not 0)
    let k = w.obj(o).map_or(0, |x| x.combat.burst);
    let fp = crate::combat::position::fire_point(w, o, 0, k);
    if let Some(s) = w.gas.systems.get_mut(&sys) {
        s.target = Pos::new(tx, ty, op.z);
        s.pos = fp;
    }
}

/// F6: the swing table entry *V*[*i*].
pub fn swing(i: i32, m: Rounding) -> i32 {
    // spec: S-0616 F6 (V[i] = trunc(sinₜ(0.125 × i) × 12.0 + 0.49), argument and sum in double precision)
    let x = crate::consts::EIGHTH.mul(d(i64::from(i), m), m);
    // spec: S-0105 F1 (the argument is never negative here, so quirks.table_trig_negative_bias changes nothing)
    glasnost_num::trig::sin_t(x, m)
        .mul(d(12, m), m)
        .add(crate::consts::SWING_BIAS, m)
        .to_i32_common()
}

/// F6: the 8-way step direction δ from `p` to `q`.
pub fn step_direction(p: Pos, q: Pos, m: Rounding) -> usize {
    let up = p.y - q.y;
    let ahead = || if up > 0 { 0 } else { 4 }; // spec: S-0616 F6 (Qₓ = Pₓ: δ = 0 when Pᵧ − Qᵧ > 0, else 4)
    if q.x == p.x {
        // spec: S-0616 F6 (Qₓ = Pₓ: δ = 0 when Pᵧ − Qᵧ > 0, else 4)
        return ahead();
    }
    let right = q.x > p.x;
    // spec: S-0616 F6 (q = (Pᵧ − Qᵧ) / (Qₓ − Pₓ) and t₈ = sinₜ(π/8) / cosₜ(π/8), double precision)
    let q_ = d(i64::from(up), m).div(d(i64::from(q.x) - i64::from(p.x), m), m);
    let eighth = crate::consts::TWO_PI.div(d(16, m), m); // spec: S-0616 F6 (π/8 = 2π / 16)
    // spec: S-0105 F1 (π/8 is positive, so quirks.table_trig_negative_bias changes nothing)
    let t8 = glasnost_num::trig::sin_t(eighth, m).div(glasnost_num::trig::cos_t(eighth, m), m);
    let inv = d(1, m).div(t8, m);
    let pick = |a: usize, b: usize| if right { a } else { b };
    // spec: S-0616 F6 (the five ranges of q)
    if q_.ge(inv) || q_.lt(inv.negated()) {
        ahead()
    } else if q_.ge(t8) {
        pick(1, 5)
    } else if q_.ge(t8.negated()) {
        pick(2, 6)
    } else {
        pick(3, 7)
    }
}

/// Claim 11, F6, F7: a fire emission.
fn fire_emission(w: &mut World, sys: ObjectId) {
    let Some((p, q)) = w.gas.systems.get(&sys).map(|s| (s.pos, s.target)) else {
        return;
    };
    let Some(ti) = sys_type(w.rules(), w, sys).and_then(|t| t.holds) else {
        // spec: S-0616 claim 11 (with no held type nothing is made and nothing is drawn)
        return;
    };
    let m = mode(w);
    // spec: S-0616 F6 (k = t rem 500)
    let k = glasnost_num::int::rem(w.tick, SWING_PERIOD);
    let v = swing(k, m);
    // spec: S-0616 F6 (ℓ = r(sum of squares in 32-bit integers), compared with 200.0 as a floating-point number)
    let sum = [p.x - q.x, p.y - q.y, p.z - q.z]
        .iter()
        .fold(0i32, |a, &x| a.wrapping_add(x.wrapping_mul(x)));
    let table = w.rules().quirks.table_root_distance;
    let l = glasnost_num::geom::table_root_real(d(i64::from(sum), m), table, m);
    let near = if l.lt(d(SWING_NEAR, m)) { 3 } else { 1 }; // spec: S-0616 F6 (m = 3 when ℓ < 200.0, otherwise 1)
    // spec: S-0616 F6 (a = trunc(12 × V[k] / (3 × m)), the quotient in double precision)
    let a = d(12 * i64::from(v), m)
        .div(d(3 * near, m), m)
        .to_i32_common();
    let (ex, ey) = SIDE[step_direction(p, q, m)];
    // spec: S-0616 F6 (the particle aims at Q + (a × eₓ(δ), a × eᵧ(δ), 0))
    let aim = Pos::new(q.x.wrapping_add(a * ex), q.y.wrapping_add(a * ey), q.z);
    // spec: S-0616 claim 11 (made 1 lepton above the system's position, with its creation draws)
    let at = Pos::new(p.x, p.y, p.z.wrapping_add(1));
    if crate::gas::add_particle(w, sys, ti, at, aim).is_none() {
        return;
    }
    // spec: S-0616 F7 (c = min(4, N), one raw value r also when c = 1, j = |r| rem c; a negative j is taken as 0, our departure)
    let n = w.gas.systems.get(&sys).map_or(0, |s| s.particles.len());
    let window = n.min(INSERT_WINDOW);
    let r = w.rng.raw() as i32;
    let j = glasnost_num::int::rem(r.wrapping_abs(), window as i32).max(0) as usize;
    if let Some(s) = w.gas.systems.get_mut(&sys)
        && j > 0
    {
        // spec: S-0616 F7 (the new particle then has exactly j particles of the list newer than it)
        let p = s.particles.remove(n - 1);
        s.particles.insert(n - 1 - j, p);
    }
}

/// Claim 12, F8: a fire particle's update.
fn fire_update(w: &mut World, sys: ObjectId, i: usize) {
    let Some(mut p) = crate::gas::particle(w, sys, i) else {
        return;
    };
    let rules = w.rules_arc();
    let Some(t) = ptype(&rules, &p) else { return };
    let m = mode(w);
    let zero = d(0, m);
    // spec: S-0616 claim 12 (S ≤ 0: S becomes 0 and the particle gets the dead mark; no draw)
    if p.speed.to_f64().le(zero) {
        p.speed = F32(0);
        p.dead = true;
        crate::gas::put(w, sys, i, p);
        return;
    }
    // spec: S-0616 F8 (one raw value r: k = (r rem 10) − 5, φ = k × 0.02 + 1.0 in double precision)
    let r = w.rng.raw() as i32;
    let k = glasnost_num::int::rem(r, 10) - 5; // spec: S-0616 F8 (k = (r rem 10) − 5)
    let phi = d(i64::from(k), m)
        .mul(crate::consts::FIFTIETH, m)
        .add(d(1, m), m);
    let sp = p.speed.to_f64();
    // spec: S-0616 F8 (step = (trunc(fl(φ × uₓ) × S), trunc(fl(φ × u_y) × S), trunc(φ × u_z × S)))
    let xy = |u: F32| {
        phi.mul(u.to_f64(), m)
            .to_f32(m)
            .to_f64()
            .mul(sp, m)
            .to_i32_common()
    };
    p.step = [
        xy(p.dir[0]),
        xy(p.dir[1]),
        phi.mul(p.dir[2].to_f64(), m).mul(sp, m).to_i32_common(), // nocite: vector and matrix dimensions (three axes)
    ];
    // spec: S-0616 claim 12 (below the end stage the stage advances by S-0613 F8 with the particle's own value, claim 19)
    if p.stage < t.end_stage && crate::gas::stage_due(t.max_ec, p.advance, &p) {
        p.stage = p.stage.wrapping_add(1);
        // spec: S-0616 claim 26 (at an advance: the translucent-25 stage field sets 25, the translucent-50 field sets 50, 50 winning when both name the stage; set, not added; fields of 128 or more never match a signed stage)
        let stage = i32::from(p.stage);
        if stage == i32::from(t.look.stage50) {
            p.translucency = FIRE_TRANSLUCENCY.1;
        } else if stage == i32::from(t.look.stage25) {
            p.translucency = FIRE_TRANSLUCENCY.0;
        }
    }
    if p.stage == t.end_stage && t.delete_at_limit {
        // spec: S-0616 claim 12 (at the end stage with the delete flag: the dead mark; the step still applies, no slowing, no damage)
        p.dead = true;
        crate::gas::put(w, sys, i, p);
        return;
    }
    // spec: S-0616 claim 12 (S goes down by the slowing field, single precision; the countdown goes down by 1 modulo 65536)
    p.speed = sp.sub(t.deacc.to_f64(), m).to_f32(m);
    p.countdown = p.countdown.wrapping_sub(1);
    // spec: S-0616 claim 12 (at 0 with a nonzero amount and the stage at or below the last damaging stage, signed: the countdown restarts and the particle hits; otherwise no restart)
    let due = p.countdown == 0 && t.damage != 0 && p.stage <= t.final_stage;
    if due {
        p.countdown = t.max_dc;
    }
    let pos = p.pos;
    crate::gas::put(w, sys, i, p);
    if due {
        fire_hit(w, sys, t, pos);
    }
}

/// Claim 13: a fire particle's move.
fn fire_move(w: &World, p: &mut Particle) {
    // spec: S-0616 claim 13 (when S ≤ 0 at the move it does not move)
    if !p.speed.to_f64().gt(d(0, mode(w))) {
        return;
    }
    let old = w.surface_at(p.pos.x, p.pos.y);
    // spec: S-0616 claim 13 (the position changes by the step; rising terrain gives the dead mark, the particle still placed at the new position)
    let at = Pos::new(
        p.pos.x.wrapping_add(p.step[0]),
        p.pos.y.wrapping_add(p.step[1]),
        p.pos.z.wrapping_add(p.step[2]),
    );
    if w.surface_at(at.x, at.y) > old {
        p.dead = true;
    }
    p.pos = at;
}

/// Claim 14, F9: a fire particle's due hit at `pos`.
fn fire_hit(w: &mut World, sys: ObjectId, t: &ParticleType, pos: Pos) {
    let cell = pos.cell();
    let level = w.cells.cell(cell).map_or(0, |c| i32::from(c.level));
    // spec: S-0616 claim 14 (the bridge-deck layer when the cell is a bridge cell and z ≥ (height level + 4) × 104, else the ground layer)
    let layer = if w.high_bridge(cell) && pos.z >= level * LEVEL + DECK {
        Layer::Deck
    } else {
        Layer::Ground
    };
    let (owner, house) = w
        .gas
        .systems
        .get(&sys)
        .map_or((None, None), |s| (s.owner_object, s.owner));
    // spec: S-0616 claim 14 and Edge cases (no attacking house, quirks.fire_particle_no_house; off: the system's owner house)
    let house = if w.rules().quirks.fire_particle_no_house {
        None
    } else {
        house
    };
    let m = mode(w);
    let table = w.rules().quirks.table_root_distance;
    let list: Vec<ObjectId> = w.cells.list(cell, layer).to_vec();
    for target in list {
        // spec: S-0616 claim 14 (every listed object alive with health above 0, not the system's owner object, in listing order)
        if Some(target) == owner {
            continue;
        }
        let Some(o) = w.obj(target) else { continue };
        // spec: S-0504 claim 10 (a sinking vessel with health left is not immune; #172)
        let dead = crate::combat::damage::dead(o);
        if o.health <= 0 || !o.on_map || dead {
            continue;
        }
        let Some(warhead) = t.warhead else {
            // spec: S-0616 claim 20 (a non-zero amount with no warhead: our engine's hits deal no damage; the importer warns)
            continue;
        };
        // spec: S-0616 F9 (R = trunc(r(Δₓ² + Δᵧ² + Δ_z²)); the hit's distance trunc(R / 10))
        let dl = [o.pos.x - pos.x, o.pos.y - pos.y, o.pos.z - pos.z];
        let sum: u64 = dl
            .iter()
            .map(|&v| (i64::from(v) * i64::from(v)) as u64)
            .sum();
        let r = glasnost_num::geom::root_floor(sum, table, m) as i32;
        let hit = crate::combat::damage::Hit {
            amount: t.damage,
            warhead: Some(warhead),
            distance: r / FIRE_DISTANCE_DIVISOR,
            // spec: S-0616 claim 14 (no attacker; defences not ignored; passengers may escape)
            attacker: None,
            house,
            ignore_defences: false,
            no_escape: false,
        };
        // spec: S-0616 claim 14 (ordinary damage, not a detonation; a removed object does not stop the hits on those after it)
        crate::combat::damage::apply(w, target, hit);
    }
}

// ---------------------------------------------------------------------------
// Spark (claims 15 and 16)

/// Claim 15, F10: a spark system's turn.
pub fn spark_turn(w: &mut World, sys: ObjectId) {
    let rules = w.rules_arc();
    let Some(t) = sys_type(&rules, w, sys) else {
        return;
    };
    let Some((k, s_pos)) = w.gas.systems.get(&sys).map(|s| (s.sparks, s.pos)) else {
        return;
    };
    let half = w.rules().quirks.ranged_draw_half_span;
    let m = mode(w);
    // spec: S-0616 claim 15 (revision 27: the table reads only K, not whether the system is marked: a marked spark system whose K is above 0 still bursts)
    if t.spark_turns <= 0 {
        // spec: S-0616 claims 15 and 20 (a type's spark-turns field of 0 or less is bad data: our engine marks the system in its first turn, with no burst and no draw of claim 15; its particles still update; a fix with no switch, lead decision)
        mark(w, sys);
    } else if k > 0 {
        // spec: S-0616 claim 15 (K = 1: no decision draw; K > 1: the decision draw R(0, 2147483646), passing when r × 4.656612877414201 × 10⁻¹⁰ ≤ χ in double precision)
        let burst = if k == 1 {
            true
        } else {
            let r = w.rng.ranged(0, DRAW_TOP, half);
            let scale = crate::consts::DRAW_SCALE; // spec: S-0616 Constants (spark chance scale 4.656612877414201 × 10⁻¹⁰)
            d(i64::from(r), m).mul(scale, m).le(t.chance)
        };
        match t.holds {
            Some(held) => {
                if burst {
                    spark_burst(w, sys, t, held, s_pos);
                }
                // spec: S-0616 claim 15 (the light: one ranged draw in every turn that starts with K > 0, after the burst's draws; drawing only)
                let _light = w.rng.ranged(0, DRAW_TOP, half);
            }
            None => {
                // spec: S-0616 claim 20 (no held type: the decision draw, if any, and nothing else; the counter still counts down)
            }
        }
        // spec: S-0616 claim 15 (K goes down by 1; the system is marked when K is below 1)
        if let Some(x) = w.gas.systems.get_mut(&sys) {
            x.sparks = k - 1;
            if x.sparks < 1 {
                x.ending = true;
            }
        }
    }
    // spec: S-0616 claim 15 (whatever K is, each particle then updates, oldest first, the new sparks included; the dead are removed)
    let n = w.gas.systems.get(&sys).map_or(0, |s| s.particles.len());
    for i in 0..n {
        update_one(w, sys, i);
    }
    if let Some(x) = w.gas.systems.get_mut(&sys) {
        x.particles.retain(|p| !p.dead);
    }
}

/// F10: one burst of sparks of held type `held` at `at`.
fn spark_burst(w: &mut World, sys: ObjectId, t: &crate::gas::SystemType, held: u32, at: Pos) {
    let m = mode(w);
    // spec: S-0616 F10 (h = trunc(κ / 2); the count raw value r₀)
    let h = t.cap / 2;
    let r0 = w.rng.raw() as i32;
    if h == 0 {
        // spec: S-0616 claim 20 (h = 0: our engine takes the count raw value only and makes no spark)
        return;
    }
    // spec: S-0616 F10 (c = (|r₀| rem h) + h)
    let count = glasnost_num::int::rem(r0.wrapping_abs(), h).wrapping_add(h);
    for _ in 0..SPARK_DISCARDED {
        // spec: S-0616 F10 (three raw values taken and discarded)
        let _ = w.rng.raw();
    }
    let Some([xv, yv, zmin, zrange]) = w
        .rules()
        .gas
        .particles
        .get(held as usize)
        .map(|pt| pt.spark)
    else {
        return;
    };
    let table = w.rules().quirks.table_root_distance;
    // spec: S-0616 r29 claim 20a (at most BURST_CEILING sparks, with their draws, in one burst)
    for _ in 0..count.clamp(0, BURST_CEILING) {
        // spec: S-0616 claim 15 (each spark: the creation draws of one particle at the system's position, the system's position its target point)
        if crate::gas::add_particle(w, sys, held, at, at).is_none() {
            break;
        }
        // spec: S-0616 F10 (three raw values rₓ, r_y, r_z: w = (rₓ rem X, r_y rem Y, (|r_z| rem Zᵣ) + Zₘ) in single precision)
        let rx = w.rng.raw() as i32;
        let ry = w.rng.raw() as i32;
        let rz = w.rng.raw() as i32;
        let wv = [
            glasnost_num::int::rem(rx, xv),
            glasnost_num::int::rem(ry, yv),
            glasnost_num::int::rem(rz.wrapping_abs(), zrange).wrapping_add(zmin),
        ]
        .map(|v| d(i64::from(v), m).to_f32(m));
        // nocite: vector and matrix dimensions (three axes)
        let sq = |v: &[F32; 3]| {
            v.iter()
                .fold(d(0, m), |a, x| a.add(x.to_f64().mul(x.to_f64(), m), m))
        };
        // spec: S-0616 F10 (μ = r(|w|²), w′ = w + s in single precision, μ′ = r(|w′|²), velocity = μ × w′ / μ′ in double precision stored in single; w′ unchanged when μ′ = 0)
        let mu = glasnost_num::geom::table_root_real(sq(&wv), table, m);
        let mut w2 = wv;
        for (a, s) in w2.iter_mut().zip(t.direction) {
            *a = a.to_f64().add(s.to_f64(), m).to_f32(m);
        }
        let mu2 = glasnost_num::geom::table_root_real(sq(&w2), table, m);
        let vel = if mu2.eq_num(d(0, m)) {
            w2
        } else {
            w2.map(|x| mu.mul(x.to_f64(), m).div(mu2, m).to_f32(m))
        };
        if let Some(p) = w
            .gas
            .systems
            .get_mut(&sys)
            .and_then(|s| s.particles.last_mut())
        {
            p.fvel = vel;
        }
    }
}

/// Claim 16, F11: a spark's update.
fn spark_update(w: &mut World, sys: ObjectId, i: usize) {
    let Some(mut p) = crate::gas::particle(w, sys, i) else {
        return;
    };
    let m = mode(w);
    let half = w.rules().quirks.ranged_draw_half_span;
    let g = d(i64::from(w.rules().gas.gravity), m);
    let a = p.pos;
    // spec: S-0616 F11 (v′_z = fl(v_z − G), the stored vertical velocity)
    let vz2 = p.fvel[2].to_f64().sub(g, m).to_f32(m);
    // spec: S-0616 F11 and Edge cases (the vertical step v′_z − G, gravity a second time, quirks.spark_double_gravity; off: v′_z)
    let step_z = if w.rules().quirks.spark_double_gravity {
        vz2.to_f64().sub(g, m)
    } else {
        vz2.to_f64()
    };
    // spec: S-0616 F11 (p = fl(A + (vₓ, v_y, step)) per component; B = trunc(p))
    let px = d(i64::from(a.x), m)
        .add(p.fvel[0].to_f64(), m)
        .to_f32(m)
        .to_f64();
    let py = d(i64::from(a.y), m)
        .add(p.fvel[1].to_f64(), m)
        .to_f32(m)
        .to_f64();
    let pz = d(i64::from(a.z), m).add(step_z, m).to_f32(m).to_f64();
    let b = Pos::new(px.to_i32_common(), py.to_i32_common(), pz.to_i32_common());
    let f = w.surface_at(b.x, b.y);
    let e = f + DECK;
    let bridge = w.high_bridge(a.cell()) || w.high_bridge(b.cell());
    // spec: S-0616 F11 (the contacts by priority: a deck crossing, then the ground, then a building or wall)
    let height = if bridge && b.z < e && e <= a.z {
        // spec: S-0616 F11 (contact from above: the height becomes E)
        Some(d(i64::from(e), m))
    } else if bridge && a.z < e && e <= b.z {
        // spec: S-0616 F11 (contact from below: the height becomes E − 20)
        Some(d(i64::from(e - SPARK_UNDER_DECK), m))
    } else if pz.lt(d(i64::from(f), m)) {
        // spec: S-0616 F11 (ground contact: the height becomes F when p_z > F − 100, otherwise it stays p_z)
        if pz.gt(d(i64::from(f - SPARK_SNAP), m)) {
            Some(d(i64::from(f), m))
        } else {
            Some(pz)
        }
    } else if pz
        .sub(d(i64::from(SPARK_BAND), m), m)
        .lt(d(i64::from(f), m))
        && stops_spark(w, b)
    {
        // spec: S-0616 F11 (F ≤ p_z < F + 150, tested as p_z − 150.0 < F, in a cell holding a building or a wall: the height becomes F)
        Some(d(i64::from(f), m))
    } else {
        None
    };
    match height {
        Some(hz) => {
            // spec: S-0616 F11 (on contact the dead mark; the final position (trunc(pₓ), trunc(p_y), trunc(height)))
            p.dead = true;
            p.pos = Pos::new(px.to_i32_common(), py.to_i32_common(), hz.to_i32_common());
        }
        None => p.pos = b,
    }
    p.fvel[2] = vz2; // nocite: vector and matrix dimensions (three axes)
    // spec: S-0616 claim 16 (the colour draw after any contact: one ranged draw R(0, 2147483646)); claim 29 and F16 (it moves the colour progress)
    let r = w.rng.ranged(0, DRAW_TOP, half);
    if let Some(t) = ptype(w.rules(), &p) {
        p.colour.advance(&t.look, r, m);
    }
    crate::gas::put(w, sys, i, p);
}

/// Whether the cell at `b` stops a spark (F11): it holds a building (every
/// building counts), or no building and a wall overlay (Terms *wall*).
fn stops_spark(w: &World, b: Pos) -> bool {
    let cell = b.cell();
    let building = w
        .cells
        .list(cell, Layer::Ground)
        .iter()
        .any(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Building));
    // spec: S-0616 Terms (wall: a wall overlay, the overlay of S-0112 claim 7) and F11 (every building counts)
    building
        || w.cells
            .cell(cell)
            .is_some_and(|c| crate::walls::is_wall(w, c.overlay))
}

// ---------------------------------------------------------------------------
// Railgun (claims 17 and 18)

/// Claim 17, F12: a railgun system's turn.
pub fn railgun_turn(w: &mut World, sys: ObjectId) {
    let rules = w.rules_arc();
    let Some(t) = sys_type(&rules, w, sys) else {
        return;
    };
    let Some(s) = w.gas.systems.get(&sys) else {
        return;
    };
    // The system as its turn starts.
    let (s_ending, s_empty, s_pos, s_target, s_type_index) = (
        s.ending,
        s.particles.is_empty(),
        s.pos,
        s.target,
        s.type_index,
    );
    let half = w.rules().quirks.ranged_draw_half_span;
    let m = mode(w);
    if !s_ending && s_empty {
        // spec: S-0616 F12 (L = trunc(r(Δₓ² + Δᵧ² + Δ_z²)), n = trunc(L × p) in double precision)
        let dl = [
            s_target.x - s_pos.x,
            s_target.y - s_pos.y,
            s_target.z - s_pos.z,
        ];
        let sum: u64 = dl
            .iter()
            .map(|&v| (i64::from(v) * i64::from(v)) as u64)
            .sum();
        let table = w.rules().quirks.table_root_distance;
        let l = glasnost_num::geom::root_floor(sum, table, m);
        let n = d(l as i64, m).mul(t.per_coord, m).to_i32_common();
        // spec: S-0616 claim 31 (the laser beam, from the system's position to its target point, is made in every line turn whatever n, with or without a held type; a detonation railgun's is of zero length) and claim 20 (with n ≥ 1 and no held type our engine makes it too, a departure)
        w.effects.push(crate::combat::effects::Effect::RailLine {
            system_type: s_type_index,
            from: s_pos,
            to: s_target,
        });
        match t.holds {
            Some(held) => {
                // spec: S-0616 claim 17 (n ≤ 0: no particle and no ranged draw)
                // spec: S-0616 r29 claim 20a (at most BURST_CEILING particles, with their draws, in one line)
                for _ in 0..n.clamp(0, BURST_CEILING) {
                    // spec: S-0616 claim 17 (per particle: three ranged draws, its creation draws, four more ranged draws; the line's shape is drawing only, so each particle is kept at the system's position)
                    for _ in 0..RAIL_BEFORE {
                        let _ = w.rng.ranged(0, DRAW_TOP, half);
                    }
                    crate::gas::add_particle(w, sys, held, s_pos, s_target);
                    for _ in 0..RAIL_AFTER {
                        let _ = w.rng.ranged(0, DRAW_TOP, half);
                    }
                }
            }
            None => {
                // spec: S-0616 claim 20 (no held type: no ranged draw at all and no particle, whatever n; the system is still marked)
            }
        }
        // spec: S-0616 claim 17 (then the system is marked, also when n ≤ 0)
        mark(w, sys);
    }
    // spec: S-0616 claim 17 (after the emission step each particle updates, oldest first; the dead are removed)
    let n = w.gas.systems.get(&sys).map_or(0, |s| s.particles.len());
    for i in 0..n {
        update_one(w, sys, i);
    }
    if let Some(x) = w.gas.systems.get_mut(&sys) {
        x.particles.retain(|p| !p.dead);
    }
}

/// Claim 18: a railgun particle's update: two ranged draws, its speed
/// variation and its colour. Its motion along its flight direction never
/// reaches play (claim 2), so its position is not advanced.
fn railgun_update(w: &mut World, sys: ObjectId, i: usize) {
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0616 claim 18 (one ranged draw R(0, 2147483646) that varies its speed, then one for its colour: two per update)
    let _speed = w.rng.ranged(0, DRAW_TOP, half);
    let r = w.rng.ranged(0, DRAW_TOP, half);
    let m = mode(w);
    if let Some(mut p) = crate::gas::particle(w, sys, i)
        && let Some(t) = ptype(w.rules(), &p)
    {
        // spec: S-0616 claim 29 and F16 (the update's colour draw moves the colour progress)
        p.colour.advance(&t.look, r, m);
        crate::gas::put(w, sys, i, p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0616 Constants (the smoke wind table: direction 3 steps 2 in x, unlike the gas table)
    #[test]
    fn smoke_wind_direction_three_is_full() {
        assert_eq!(SMOKE_WIND[3], (2, 2));
        assert_eq!(SMOKE_WIND[5], (-2, 2));
    }

    /// spec: S-0616 Formulas (⌊·⌋ on a negative real rounds down)
    #[test]
    fn floor_rounds_down() {
        let m = Rounding::Nearest;
        assert_eq!(floor_i32(c("10.95")), 10);
        assert_eq!(floor_i32(c("-0.5")), -1);
        assert_eq!(floor_i32(F64::from_i64(-3, m)), -3);
    }
}
