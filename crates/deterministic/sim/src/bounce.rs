//! Bouncing animations (S-0319): the motion of a sprite animation whose
//! type is a bouncing type at its creation (S-0109 Terms), its landing, the
//! effects of the landing and of the end that follows at once, and the
//! trailer animation that any animation type can name (claim 12).
//!
//! A bouncing animation is an ordinary animation in play (`crate::anims`)
//! that carries a [`Flight`] from its creation (claim 1). In each of its
//! turns the motion step comes first (claim 4); a landing ends the
//! animation in the same turn (claims 7 and 8), and otherwise the trailer
//! (claim 12) and the stage update of S-0109 follow (State and ordering).
//!
//! The flight and contact rule is shared with the debris pieces of S-0317
//! (`crate::pieces`), apart from the differences claims 6 and 7 name; each
//! spec keeps its own switches.
//!
//! Interims (requests): which buildings count for the building band, the
//! turn off a cliff face, the rebound on a sloped cell, and an animation
//! off the map (Q-1846, reusing S-0317's Q-1834 and Q-1836 interims); how a
//! new resource cell enters the resource lists, and a spawn overlay type
//! whose place plus the overlay draw lies past the overlay list (Q-1847).

use glasnost_map::CellXY;
use glasnost_num::{F32, F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::combat::data::WarheadId;
use crate::pieces::{DECK, gravity, ground, level, stopped, water};
use crate::rules::AnimRules;
use crate::world::{Layer, Pos, World};

/// The raw values a bouncing animation's creation takes (claims 2 and 3).
pub(crate) type Raws = [u32; RAW_DRAWS];

/// How many raw values the creation takes.
pub(crate) const RAW_DRAWS: usize = 3; // spec: S-0202 Constants (animation bounce draws: 3 raw values) and S-0319 claims 2 and 3

/// The step directions F5 offers (S-0204's 0 to 7).
const DIRECTIONS: usize = 8; // spec: S-0319 F5 (step directions 0 to 7 of S-0204)

/// The S-0319 fields of an animation type (Import mapping), with the spans
/// of F1 and F2 formed once.
#[derive(Clone, Debug)]
pub struct BounceType {
    /// The meteor flag (claims 1, 3, 4, 8 and 11).
    pub meteor: bool,
    /// The horizontal speed limit *M* (F1, F2), a double.
    pub m: F64,
    /// The lower vertical limit *Z*ₘᵢₙ (F1, F2), a double.
    pub zmin: F64,
    /// *W*ₓᵧ = trunc(2*M*) (F1, F2).
    pub wxy: i32,
    /// *W*_z = trunc(*Z*ₘₐₓ − *Z*ₘᵢₙ + 1) (F1).
    pub wz: i32,
    /// The elasticity *e* (claim 7): a single widened to a double.
    pub elasticity: F64,
    /// The damage radius in leptons (claim 7).
    pub radius: i32,
    /// The warhead's name as given (claims 7 and 8).
    pub warhead_name: Option<String>,
    /// The warhead by index (claims 7 and 8); `None` for none, or for a name
    /// no warhead is registered under.
    pub warhead: Option<WarheadId>,
    /// The bounce animation (claim 7).
    pub bounce: Option<String>,
    /// The expiry animation (claim 8).
    pub expire: Option<String>,
    /// The trailer animation (claim 12).
    pub trailer: Option<String>,
    /// The trailer separation *s* in ticks (claim 12), as written.
    pub separation: i32,
    /// The spawn type (claim 9).
    pub spawns: Option<String>,
    /// The spawn count *c* (claim 9).
    pub spawn_count: i32,
    /// The resource flag (claim 10).
    pub tiberium: bool,
    /// The resource spawn overlay type's place in the overlay list (claim
    /// 10); `None` for none, or for a name the list does not hold.
    pub spawn_overlay: Option<usize>,
    /// The resource spread radius *R* in cells (claim 10, F5).
    pub spread: i32,
}

impl Default for BounceType {
    fn default() -> Self {
        Self::new(
            &glasnost_rules::sim::AnimationType::default(),
            &[],
            &glasnost_rules::quirks::Quirks::default(),
            Rounding::default(),
        )
    }
}

impl BounceType {
    /// The S-0319 fields of animation type `t`; `overlays` are the overlay
    /// list's names in order.
    pub fn new(
        t: &glasnost_rules::sim::AnimationType,
        overlays: &[String],
        q: &glasnost_rules::quirks::Quirks,
        mode: Rounding,
    ) -> Self {
        let num = |s: &str| glasnost_num::decimal::nearest_double(s);
        let dec = |d: &Option<glasnost_rules::sim::Decimal>, default: &str| {
            d.as_ref().map_or_else(|| num(default), |d| d.value(mode))
        };
        // spec: S-0319 Import mapping (MaxXYVel absent 15.0, MinZVel absent 3.5; doubles)
        let m = dec(&t.max_xy_vel, "15.0");
        let zmin = dec(&t.min_z_vel, "3.5");
        let zmax = if q.anim_fixed_upper_vertical_speed {
            // spec: S-0319 claim 2, F1 and Edge cases (the upper vertical limit is fixed at 3.5; the type's key is not read)
            num("3.5")
        } else {
            // spec: S-0319 Edge cases and Import mapping (quirks.anim_fixed_upper_vertical_speed off: the type's MaxZVel, absent 3.5)
            dec(&t.max_z_vel, "3.5")
        };
        let one = F64::from_i64(1, mode);
        // spec: S-0319 F1 (W_xy = trunc(2M), W_z = trunc(Z_max − Z_min + 1), formed in double precision; S-0205 claim 7's conversion)
        let wxy = m.mul(F64::from_i64(2, mode), mode).to_i32_common();
        let wz = zmax.sub(zmin, mode).add(one, mode).to_i32_common();
        Self {
            // spec: S-0319 Import mapping (IsMeteor absent no)
            meteor: t.is_meteor.unwrap_or(false),
            m,
            zmin,
            wxy,
            wz,
            // spec: S-0319 Import mapping (Elasticity absent 0.8, the single-precision value widened)
            elasticity: t.elasticity.as_ref().map_or_else(
                || crate::gas::nearest_single("0.8").to_f64(),
                |d| d.single_value(mode).to_f32(mode).to_f64(),
            ),
            // spec: S-0319 Import mapping (DamageRadius absent 0; Warhead absent none)
            radius: t.damage_radius.unwrap_or(0),
            warhead_name: t.warhead.clone(),
            warhead: None,
            // spec: S-0319 Import mapping (BounceAnim, ExpireAnim, TrailerAnim absent none)
            bounce: t.bounce_anim.clone(),
            expire: t.expire_anim.clone(),
            trailer: t.trailer_anim.clone(),
            // spec: S-0319 Import mapping and claim 12 (TrailerSeperation absent 0, which acts as 1)
            separation: t.trailer_separation.unwrap_or(0),
            // spec: S-0319 Import mapping (Spawns absent none, SpawnCount absent 0)
            spawns: t.spawns.clone(),
            spawn_count: t.spawn_count.unwrap_or(0),
            // spec: S-0319 Import mapping (IsTiberium absent no, TiberiumSpawnType absent none, TiberiumSpreadRadius absent 0)
            tiberium: t.is_tiberium.unwrap_or(false),
            // spec: S-0004 claim 13 (a type name compared without regard to letter case)
            spawn_overlay: t
                .tiberium_spawn_type
                .as_ref()
                .and_then(|n| overlays.iter().position(|o| o.eq_ignore_ascii_case(n))),
            spread: t.tiberium_spread_radius.unwrap_or(0),
        }
    }

    /// Resolves the warhead by name against the warheads in index order
    /// (claims 7 and 8).
    pub fn resolve(&mut self, warheads: &[String]) {
        // spec: S-0319 Import mapping (Warhead: a warhead name)
        self.warhead = self
            .warhead_name
            .as_ref()
            .and_then(|h| warheads.iter().position(|w| w == h))
            .and_then(|i| u32::try_from(i).ok());
    }
}

/// A bouncing animation's flight (S-0319 Terms): its flight position and
/// its velocity, single precision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flight {
    /// The flight position in leptons.
    pub p: [F32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The velocity in leptons per tick.
    pub v: [F32; 3], // nocite: vector and matrix dimensions (three axes)
}

/// |r| rem W for a raw value r read as signed (S-0319 Formulas, S-0202
/// claim 6), with claim 14's departure for a span of 0.
fn mag_rem(raw: u32, span: i32, overflow: bool) -> i64 {
    if span == 0 {
        // spec: S-0319 claim 14 (a span of 0 is a departure: the remainder is taken as 0)
        return 0;
    }
    let r = raw as i32;
    if r == i32::MIN && !overflow {
        // spec: S-0319 Formulas (quirks.random_magnitude_overflow off: |−2147483648| is 2147483648)
        return i64::from(r).abs() % i64::from(span);
    }
    // spec: S-0319 Formulas (the "magnitude first" remainder, truncated toward zero with the dividend's sign; −2147483648 keeps its sign under quirks.random_magnitude_overflow) and claim 14 (a span of −1 with that value: S-0205 claim 3's result, 0)
    i64::from(glasnost_num::int::rem(r.wrapping_abs(), span))
}

/// The creation of a bouncing animation of type `a` at the creation point
/// `at` from the three raw values of its creation draws (claims 2 and 3):
/// the position it is placed at and its flight.
pub(crate) fn create(w: &World, a: &AnimRules, raws: Raws, at: Pos) -> (Pos, Flight) {
    let rules = w.rules();
    let mode = rules.mode;
    let ov = rules.quirks.random_magnitude_overflow;
    let t = &a.bounce;
    let d = |x: i64| F64::from_i64(x, mode);
    let [x1, x2, x3] = raws;
    if t.meteor {
        // spec: S-0319 claim 1 (a type with the meteor flag makes a meteor animation, with the bouncer flag or without) and claim 3, F2 (v_y = (|q₁| rem W_xy) − M, v_x = (|q₂| rem W_xy) − M, stored in single precision; v_z = Z_min in single precision)
        let mut vy = d(mag_rem(x1, t.wxy, ov)).sub(t.m, mode).to_f32(mode);
        let mut vx = d(mag_rem(x2, t.wxy, ov)).sub(t.m, mode).to_f32(mode);
        let vz = t.zmin.to_f32(mode);
        // spec: S-0319 claim 3 and F2 (when v_x < −v_y both horizontal speeds change sign)
        if vx.to_f64().lt(vy.to_f64().negated()) {
            vx = vx.to_f64().negated().to_f32(mode);
            vy = vy.to_f64().negated().to_f32(mode);
        }
        // spec: S-0319 F2 and Constants (T = 70 − |q₃ rem 20|, the remainder before the absolute value: 51 to 70 turns)
        let tt = 70 - glasnost_num::int::rem(x3 as i32, 20).abs();
        let tf = d(i64::from(tt));
        // spec: S-0319 F2 (start = trunc(c − T v) per axis, formed in double precision from the integer point, the integer T and the stored speed, truncated once)
        let back = |c: i32, v: F32| {
            d(i64::from(c))
                .sub(tf.mul(v.to_f64(), mode), mode)
                .to_i32_common()
        };
        let start = Pos::new(back(at.x, vx), back(at.y, vy), back(at.z, vz));
        // spec: S-0319 claim 3 (placed at its start position; its flight position starts there); a start position off the map is placed too (interim, Q-1846)
        let p = [start.x, start.y, start.z].map(|c| d(i64::from(c)).to_f32(mode));
        (start, Flight { p, v: [vx, vy, vz] })
    } else {
        // spec: S-0319 claim 2 and F1 (r₁ the vertical speed, r₂ the second horizontal axis, r₃ the first: v_x = (|r₃| rem W_xy) − M, v_y = (|r₂| rem W_xy) − M, v_z = (|r₁| rem W_z) + Z_min; formed in double precision, stored in single precision)
        let vx = d(mag_rem(x3, t.wxy, ov)).sub(t.m, mode).to_f32(mode);
        let vy = d(mag_rem(x2, t.wxy, ov)).sub(t.m, mode).to_f32(mode);
        let vz = d(mag_rem(x1, t.wz, ov)).add(t.zmin, mode).to_f32(mode);
        let lift = 10; // spec: S-0319 claim 2, F1 and Constants (the flight position starts 10 leptons above the creation point)
        let p = [
            d(i64::from(at.x)).to_f32(mode),
            d(i64::from(at.y)).to_f32(mode),
            d(i64::from(at.z) + lift).to_f32(mode),
        ];
        // spec: S-0319 claim 2 (placed at its creation point)
        (at, Flight { p, v: [vx, vy, vz] })
    }
}

/// What a contact of claim 6 was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Contact {
    /// No contact.
    None,
    /// A ground, building or under-deck contact.
    Ground,
    /// A deck crossing from above that put the animation on the deck.
    OnDeck,
}

/// A landing (claim 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Landing {
    /// A stopping landing.
    Stopping,
    /// An impact landing: the position in play before the move, and
    /// whether claim 6 put it at the deck height.
    Impact { old: Pos, on_deck: bool },
}

/// The motion step of claim 4 and F3, with the contacts of claim 6 and the
/// stop test of claim 7, for an animation whose current type is `t` (the
/// meteor test of claim 4 and the elasticity read it, claim 11). Updates
/// `flight` and `pos`, the position in play; returns the landing, if any.
pub(crate) fn step(
    w: &World,
    flight: &mut Flight,
    pos: &mut Pos,
    t: &AnimRules,
) -> Option<Landing> {
    let rules = w.rules();
    let mode = rules.mode;
    let q = rules.quirks;
    let g = gravity(); // spec: S-0319 claim 4, F3 and Constants (g = 1.4, the single-precision value widened, whatever the game-wide gravity)
    let p0 = flight.p;
    let v0 = flight.v;
    let old = *pos;
    // spec: S-0319 claim 6 (u: the vertical speed at the start of the turn, before the 1.4 is taken off)
    let u = v0[2].to_f64();
    // spec: S-0319 claim 4 and F3 (v′ = (v_x, v_y, RZ₂₄(v_z − g)), p′ = RZ₂₄(p + v′) per axis)
    let vz = u.sub(g, mode).to_f32(mode);
    let mut v = [v0[0], v0[1], vz];
    let mut p = [0, 1, 2].map(|i| p0[i].to_f64().add(v[i].to_f64(), mode).to_f32(mode)); // nocite: vector and matrix dimensions (three axes)
    // nocite: vector and matrix dimensions (three axes)
    let trunc = |p: &[F32; 3]| {
        let [x, y, z] = p.map(|c| c.to_f64().to_i32_common());
        Pos::new(x, y, z)
    };
    // spec: S-0319 claim 6 (the ground and deck heights are taken at the new position, truncated toward zero)
    let new = trunc(&p);
    let (oc, nc) = (old.cell(), new.cell());
    // Interim (Q-1846, as S-0317's Q-1836): off the map the ground height and height level are 0, the land type is not water and there is no high bridge; the animation is not removed for being there.
    let f = ground(w, new);
    let fd = F64::from_i64(i64::from(f), mode);
    let z0f = p0[2].to_f64(); // nocite: vector and matrix dimensions (three axes)
    let z1f = p[2].to_f64(); // nocite: vector and matrix dimensions (three axes)
    let at = |h: i32| F64::from_i64(i64::from(h), mode).to_f32(mode);
    let mut contact = Contact::None;
    // spec: S-0319 Terms and claim 6 (the deck height: 416 leptons above the ground height)
    let deck = f.wrapping_add(DECK);
    let dd = F64::from_i64(i64::from(deck), mode);
    // spec: S-0319 claim 6 (the cell of the new position or of the old one is a high-bridge cell: one an intact high bridge marks)
    let bridge = w.high_bridge(nc) || w.high_bridge(oc);
    if bridge && z0f.ge(dd) && z1f.lt(dd) {
        // spec: S-0319 claim 6 (from at or above the deck height to below it: the height is put at the deck height)
        p[2] = at(deck);
        contact = Contact::OnDeck;
    } else if bridge && z0f.lt(dd) && z1f.ge(dd) {
        // spec: S-0319 claim 6 and Constants (from below the deck height to at or above it: 20 leptons below the deck height)
        p[2] = at(deck - 20);
        contact = Contact::Ground;
    } else if z1f.lt(fd) {
        // spec: S-0319 claim 6 and Constants (below the ground height: put at it when less than 100 leptons below); Edge cases (quirks.bouncing_anim_deep_landing off: put at it however far below)
        if fd.sub(z1f, mode).lt(F64::from_i64(100, mode)) || !q.bouncing_anim_deep_landing {
            p[2] = at(f);
        }
        contact = Contact::Ground;
    } else if z1f.lt(F64::from_i64(i64::from(f) + 150, mode)) // spec: S-0319 claim 6 and Constants (less than 150 leptons above the ground height)
        && w.cells
            .occ(nc, Layer::Ground)
            .is_some_and(|o| o.buildings > 0)
    {
        // spec: S-0319 claim 6 (a building cell: the height is put at the ground height); which buildings and overlays count is interim, every ground-layer building and no overlay (Q-1846)
        p[2] = at(f);
        contact = Contact::Ground;
    }
    let mut undone = false;
    if contact != Contact::None {
        let z1 = p[2].to_f64(); // nocite: vector and matrix dimensions (three axes)
        // spec: S-0319 claim 6 (z₀: the flight position's height before the move, not the position in play's)
        let z0 = z0f;
        let one = F64::from_i64(1, mode);
        let zero = F64::ZERO;
        // spec: S-0319 claim 6 and Constants (the cliff test: the new cell at least 2 height levels above the old one; undone when u < 0 and z₁ > z₀, or u > 0 and z₁ > z₀ + u + 1; u = 0 meets neither)
        let cliff = level(w, nc) >= level(w, oc) + 2;
        if cliff
            && ((u.lt(zero) && z1.gt(z0)) || (u.gt(zero) && z1.gt(z0.add(u, mode).add(one, mode))))
        {
            undone = true;
            // spec: S-0319 claim 6 (the flight position and the velocity become those before the move)
            p = p0;
            v = v0;
            // Interim (Q-1846, as S-0317's Q-1834): the turn off the cliff face reverses each horizontal component along which the cell changed.
            if nc.x != oc.x {
                v[0] = v[0].to_f64().negated().to_f32(mode);
            }
            if nc.y != oc.y {
                v[1] = v[1].to_f64().negated().to_f32(mode);
            }
        } else {
            // spec: S-0319 claim 7 (the rebound of S-0317 claim 10: on a flat cell the vertical speed reversed); a sloped cell's rebound is interim, the flat one (Q-1846)
            v[2] = v[2].to_f64().negated().to_f32(mode);
        }
        // spec: S-0319 claim 7 (the whole velocity scaled by the elasticity e, each component stored in single precision)
        v = v.map(|c| c.to_f64().mul(t.bounce.elasticity, mode).to_f32(mode));
    }
    // spec: S-0319 claim 7 (the motion measure S of S-0317 F6 from the velocity the step has left and the height, before a meteor's addition of 1.4; stopping when S < 2.5)
    // Interim (Q-1846): S-0317 F6 as `pieces::stopped` computes it, with or without a contact and whatever the rebounded velocity.
    let stop = stopped(w, &p, &v, g, mode, q.table_root_distance);
    if t.bounce.meteor {
        // spec: S-0319 claims 4 and 11 and F3 (the current type's meteor flag: v_z = RZ₂₄(w_z + g) after the tests)
        v[2] = v[2].to_f64().add(g, mode).to_f32(mode);
    }
    flight.p = p;
    flight.v = v;
    // spec: S-0319 claim 4, F3 and Terms (the position in play: the flight position truncated toward zero in each axis, after the tests; after an undone move, the restored one)
    *pos = trunc(&p);
    if stop {
        // spec: S-0319 claim 7 (S < 2.5: a stopping landing, with or without a contact)
        return Some(Landing::Stopping);
    }
    match contact {
        Contact::None => None,
        // spec: S-0319 claim 7 (otherwise a step with a contact is an impact landing)
        c => Some(Landing::Impact {
            old,
            on_deck: c == Contact::OnDeck && !undone,
        }),
    }
}

/// Claim 8's water and deck-height tests at the landing point `at`.
fn tests(w: &World, at: Pos) -> (bool, bool) {
    // spec: S-0319 claim 8 (on water: the cell has the water land type; at deck height: at least the deck height above the ground height, at any cell)
    let on_water = water(w, at.cell());
    let at_deck = at.z.wrapping_sub(ground(w, at)) >= DECK;
    (on_water, at_deck)
}

/// The effects of a landing at the landing point `at` for an animation
/// whose current type is `t` (claims 7 to 10), in claim 13's order. The
/// caller removes the animation afterwards (claim 8).
pub(crate) fn land(w: &mut World, t: &AnimRules, at: Pos, landing: Landing) {
    let rules = w.rules_arc();
    let q = rules.quirks;
    let b = &t.bounce;
    // spec: S-0319 claims 7 and 8 (the type's damage amount truncated toward zero)
    let damage = t.damage.to_i32_common();
    let (on_water, at_deck) = tests(w, at);
    if let Landing::Impact { old, on_deck } = landing
        // spec: S-0319 claim 7 and Edge cases (an impact on water makes its bounce animation and hits; quirks.bouncing_anim_water_impact_hits off: not on water and not at deck height only)
        && (q.bouncing_anim_water_impact_hits || !on_water || at_deck)
    {
        if let Some(name) = b.bounce.clone() {
            // spec: S-0319 claim 7 (the bounce animation at the position in play before the move, no loop delay, repeat factor 1, no reverse request, before the hits)
            crate::combat::effects::anim(w, &name, old);
        }
        let cell = at.cell();
        // spec: S-0319 claim 7 and Edge cases (the ground layer whatever the height; quirks.bouncing_anim_deck_impact_layer off: a landing put at the deck height hits the deck layer of a high-bridge cell)
        let layer = if on_deck && !q.bouncing_anim_deck_impact_layer && w.high_bridge(cell) {
            Layer::Deck
        } else {
            Layer::Ground
        };
        // spec: S-0319 claim 7 (the hits of S-0317 claim 7 in the landing point's cell: |Δx| + |Δy| at most the radius, the lifted distance, no attacker, no house, defences not ignored, no no-escape marker; listing order; no skip for an object with no health left, as S-0317's reading; interim for a listing a hit changes, Q-1846)
        crate::pieces::ground_hits(w, cell, layer, at, b.radius, damage, b.warhead);
    }
    if !on_water || at_deck {
        // spec: S-0319 claim 8 and Edge cases (the area damage only when the type names an expiry animation; quirks.bouncing_anim_expiry_needs_animation off: also when its damage amount is not 0)
        let area = b.expire.is_some()
            || (!q.bouncing_anim_expiry_needs_animation && !t.damage.eq_num(F64::ZERO));
        if let Some(name) = b.expire.clone() {
            // spec: S-0319 claim 8 (the expiry animation at the landing point, no loop delay, repeat factor 1; its depth adjustment is drawing only)
            crate::combat::effects::anim(w, &name, at);
        }
        if area {
            // spec: S-0319 claim 8 (a detonation's area damage at the landing point, S-0303 claims 1 to 20: the damage amount, the warhead, no source object, no house, resources affected; the light flash is drawing only)
            crate::combat::area::detonate(
                w,
                crate::combat::area::Detonation {
                    point: at,
                    damage,
                    warhead: b.warhead,
                    source: None,
                    house: None,
                    resources: true,
                },
            );
        }
        if let Some(name) = b.spawns.clone()
            && b.spawn_count > 0
        {
            let half = q.ranged_draw_half_span;
            // spec: S-0319 claim 9 and F4 (n = R(0, c) + R(0, c), both draws before the first creation, a signed 32-bit sum that wraps; n ≤ 0 makes none)
            let first = w.rng.ranged(0, b.spawn_count, half);
            let n = first.wrapping_add(w.rng.ranged(0, b.spawn_count, half));
            for _ in 0..n.max(0) {
                // spec: S-0319 claim 9 (each at the landing point with no loop delay, repeat factor 1, no reverse request, its own creation draws in turn)
                crate::combat::effects::anim(w, &name, at);
            }
        }
    } else {
        let list = rules.combat.general.splash_list.clone();
        let lifted = Pos::new(at.x, at.y, at.z.wrapping_add(3)); // spec: S-0319 claim 8 and Constants (the splash 3 leptons higher)
        // spec: S-0319 claims 8 and 11 (the meteor test of the water rows reads the type the animation has at the landing)
        if b.meteor {
            // spec: S-0319 claim 8 (a meteor animation on water: the last splash-list entry; none with an empty list, a departure)
            if let Some(s) = list.last() {
                crate::combat::effects::anim(w, s, lifted);
            }
        } else {
            // spec: S-0319 claim 8 (a falling animation on water: the game-wide wake animation at the landing point)
            if let Some(wake) = rules.wake.clone() {
                crate::combat::effects::anim(w, &wake, at);
            }
            // spec: S-0319 claim 8 (then the first splash-list entry; none with an empty list, a departure)
            if let Some(s) = list.first() {
                crate::combat::effects::anim(w, s, lifted);
            }
        }
    }
    if b.tiberium && !on_water && !at_deck {
        resources(w, b, at.cell());
    }
}

/// Claim 10 and F5: new resource around the landing cell `c`.
fn resources(w: &mut World, b: &BounceType, c: CellXY) {
    // spec: S-0319 claim 10 (a type with the resource flag and no spawn overlay type makes no resource and takes no draws)
    let Some(spawn) = b.spawn_overlay else {
        return;
    };
    let r = i64::from(b.spread);
    // spec: S-0319 F5 (pairs with −R ≤ i, j ≤ R; none when R < 0)
    if r < 0 {
        return;
    }
    // spec: S-0319 F5 (a pair is used when trunc(√(i² + j²)) ≤ R, so exactly when i² + j² < (R + 1)²)
    let used = |i: i64, j: i64| {
        let (i, j, r) = (i128::from(i), i128::from(j), i128::from(r));
        i * i + j * j < (r + 1) * (r + 1)
    };
    if w.rules().quirks.anim_resource_spread_neighbours_only {
        // spec: S-0319 F5 and Edge cases (the pair offers the neighbour in step direction i mod 8, least non-negative residue; j does not change the cell)
        // The pairs of one i offer one cell, which after its first offer is not germinable or has received resource and is no longer (F5), so one offer per i is the whole of its pairs; and eight successive i that take no draws cover every direction without changing the world, so the offers after them repeat theirs and change nothing.
        // Interim (Q-1847): a cell whose offer took the draws and placed nothing (a place past the overlay list) takes no further offer in this landing.
        let mut barred = [false; DIRECTIONS];
        let mut idle = 0;
        let mut i = -r;
        while i <= r && idle < DIRECTIONS {
            debug_assert!(used(i, 0));
            let d = i.rem_euclid(DIRECTIONS as i64) as usize; // spec: S-0319 F5 (i mod 8, least non-negative residue)
            let (dx, dy) = crate::regions::STEPS[d]; // spec: S-0319 F5 (step directions 0 to 7 of S-0204)
            let took = if barred[d] {
                Offer::NoDraws
            } else {
                offer(w, spawn, CellXY::new(c.x + dx, c.y + dy))
            };
            match took {
                Offer::NoDraws => idle += 1,
                Offer::Placed => idle = 0,
                Offer::Nothing => {
                    barred[d] = true;
                    idle = 0;
                }
            }
            i += 1;
        }
    } else {
        // spec: S-0319 Edge cases (quirks.anim_resource_spread_neighbours_only off: the pair offers the cell (X + i, Y + j), in the same order)
        // A cell outside the coordinate square is not on the map, so it is not germinable and takes no draws: the pairs are clamped to the square.
        let side = i64::from(w.cells.grid.footprint().coordinate_side());
        let (x, y) = (i64::from(c.x), i64::from(c.y));
        let (i0, i1) = ((-r).max(-x), r.min(side - 1 - x));
        let (j0, j1) = ((-r).max(-y), r.min(side - 1 - y));
        for i in i0..=i1 {
            for j in j0..=j1 {
                if used(i, j) {
                    offer(w, spawn, CellXY::new((x + i) as i32, (y + j) as i32));
                }
            }
        }
    }
}

/// One offered cell of claim 10: when germinable, the overlay draw, the
/// placement and the stage draw.
fn offer(w: &mut World, spawn: usize, c: CellXY) -> Offer {
    // spec: S-0319 claim 10 (each offered cell that is germinable, S-0401 claim 10; others take no draws)
    if !crate::econ::resource::germinable(w, c) {
        return Offer::NoDraws;
    }
    let half = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0319 claim 10 and Constants (the overlay draw R(0, 3) first: that many places after the spawn overlay type)
    let k = w.rng.ranged(0, 3, half);
    let place = spawn + k as usize;
    // Interim (Q-1847): a place past the overlay list (or past the last index a cell can hold) places nothing; the stage draw is still taken.
    let o = u8::try_from(place)
        .ok()
        .filter(|&o| place < w.rules().overlays.len() && o != glasnost_map::NO_OVERLAY);
    if let Some(o) = o {
        // spec: S-0319 claim 10 (the overlay is placed with the effects and draws of an overlay placement, S-0202 claim 10)
        crate::overlays::place_overlay(w, c, o);
    }
    // spec: S-0319 claim 10 and Constants (the stage draw R(0, 2) last)
    let stage = w.rng.ranged(0, 2, half);
    if o.is_some() {
        if let Some(cell) = w.cells.grid.get_mut(c) {
            // spec: S-0319 claim 10 (the stage it gives is the cell's stage whatever the placement set)
            cell.overlay_data = stage as u8;
        }
        // spec: S-0401 claim 5 (a cell whose overlay changed has its land type worked out again; the placement recalculated it) and S-0212 claim 5 (its category recomputed)
        crate::regions::cell_changed(w, c);
        // Interim (Q-1847): the new resource cell is not offered to S-0401's growth or spread lists and takes no growth-list draw.
        Offer::Placed
    } else {
        Offer::Nothing
    }
}

/// What one offer of claim 10 did.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Offer {
    /// The cell was not germinable: no draws.
    NoDraws,
    /// The draws were taken and the overlay placed.
    Placed,
    /// The draws were taken and nothing placed (Q-1847's interim).
    Nothing,
}

/// Whether an animation of current type `t` makes a trailer on tick
/// `tick` (claim 12), and of which type.
pub(crate) fn trailer_due(t: &AnimRules, tick: i32) -> Option<String> {
    let b = &t.bounce;
    let name = b.trailer.clone()?;
    // spec: S-0319 claim 12 (a negative separation acts as its magnitude; a separation of 0 acts as 1, a departure)
    let s = i64::from(b.separation.unsigned_abs().max(1));
    // spec: S-0319 claim 12 (on ticks whose tick number is divisible by s)
    (i64::from(tick).rem_euclid(s) == 0).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0319 Formulas (the magnitude first, −2147483648 keeping its sign under the switch) and claim 14 (a span of 0 gives 0; a span of −1 with −2147483648 gives 0)
    #[test]
    fn magnitude_remainders() {
        assert_eq!(mag_rem(7, 5, true), 2);
        assert_eq!(mag_rem((-7i32) as u32, 5, true), 2);
        assert_eq!(mag_rem((-7i32) as u32, -5, true), 2);
        assert_eq!(mag_rem(i32::MIN as u32, 30, true), -2147483648 % 30);
        assert_eq!(mag_rem(i32::MIN as u32, 30, false), 2147483648 % 30);
        assert_eq!(mag_rem(9, 0, true), 0);
        assert_eq!(mag_rem(i32::MIN as u32, 0, false), 0);
        assert_eq!(mag_rem(i32::MIN as u32, -1, true), 0);
    }

    /// spec: S-0319 claim 12 (s divides the tick number; 0 acts as 1; a negative s as its magnitude)
    #[test]
    fn trailer_cadence() {
        let mut t = AnimRules::default();
        assert_eq!(trailer_due(&t, 4), None);
        t.bounce.trailer = Some("T".into());
        t.bounce.separation = 0;
        assert!(trailer_due(&t, 7).is_some());
        t.bounce.separation = 3;
        assert!(trailer_due(&t, 6).is_some());
        assert!(trailer_due(&t, 7).is_none());
        t.bounce.separation = -3;
        assert!(trailer_due(&t, 9).is_some());
        assert!(trailer_due(&t, 10).is_none());
        t.bounce.separation = i32::MIN;
        assert!(trailer_due(&t, 0).is_some());
        assert!(trailer_due(&t, 5).is_none());
    }
}
