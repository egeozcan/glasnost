//! Debris pieces with voxel models (S-0317): their creation, flight,
//! contacts and rebounds, impacts, stops, end of life and removal, with
//! their attached particle systems, trailers and start sounds.
//!
//! A piece is an object of play (claim 1): it has its own entry in the
//! update order and takes one turn per tick there (claim 6,
//! `crate::tick`). The pieces of a destroyed object (S-0312 claim 4, F1),
//! of a projectile's warhead (S-0303 claim 22, F6) and the barrel debris of
//! an exploding overlay (S-0303 claim 19) are made here.
//!
//! The rebound on a sloped cell (claim 17, F8), the turn off a cliff face
//! (claim 18, F9), the cells that count as holding a building (claim 19)
//! and a piece off the legal map (claim 20) follow S-0317 r57.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::{F32, F64, Rounding};
use serde::{Deserialize, Serialize};

use crate::combat::data::WarheadId;
use crate::rules::Kind;
use crate::world::{HouseId, Layer, ObjectId, Pos, World};

/// The gravity *g* of every piece (S-0317 claim 5, F3, Constants): the
/// single-precision 1.4 widened to a double.
pub(crate) fn gravity() -> F64 {
    crate::gas::nearest_single("1.4").to_f64() // spec: S-0317 F3 and Constants (g = 1.4 leptons per tick per tick, the single-precision value widened)
}

/// The deck height above the ground (S-0317 claims 10 and 11, Constants).
pub(crate) const DECK: i32 = 416; // spec: S-0317 claim 10 and Constants (the deck height: the ground height plus 416 leptons, 4 height levels)

/// The height levels above the old cell from which a cell is high (S-0317
/// claim 10's cliff test, claim 18, Terms, Constants).
const HIGH: i32 = 2; // spec: S-0317 Terms and Constants (high cell: 2 height levels above the old cell)

/// One piece type (a voxel-animation type, S-0317 Import mapping), with the
/// spans of F1, F2 and F5 formed once.
#[derive(Clone, Debug, Default)]
pub struct PieceType {
    /// Type name.
    pub name: String,
    /// The lifetime counter at creation (claim 2).
    pub duration: i32,
    /// A meteor piece (claim 4).
    pub meteor: bool,
    /// The horizontal speed limit *M* (F1), a double.
    pub m: F64,
    /// The lower vertical limit *Z*ₘᵢₙ (F1, F2), a double.
    pub zmin: F64,
    /// *S*ₓᵧ = trunc(2*M*) (F1).
    pub sxy: i32,
    /// *S*_z = trunc(*Z*ₘₐₓ − *Z*ₘᵢₙ + 1) (F1).
    pub sz: i32,
    /// *S*ₐ = trunc(*A*ₘₐₓ − *A*ₘᵢₙ + 1) (F5).
    pub sa: i32,
    /// The lower spin limit *A*ₘᵢₙ in radians per tick (F5).
    pub amin: F64,
    /// The elasticity *e* as imported (claim 10), a double: F9's cliff
    /// turn scales by it.
    pub elasticity: F64,
    /// *ε*, the elasticity narrowed once to single precision (F8): the
    /// rebound scales by it.
    pub epsilon: F32,
    /// The damage amount (claims 7 and 11).
    pub damage: i32,
    /// The damage radius in leptons (claim 7).
    pub radius: i32,
    /// The warhead (claims 7 and 11).
    pub warhead: Option<WarheadId>,
    /// The bounce animation (claim 7).
    pub bounce: Option<String>,
    /// The expiry animation (claim 11).
    pub expire: Option<String>,
    /// The trailer animation (claim 6).
    pub trailer: Option<String>,
    /// The spawn type (claim 12), by registration index.
    pub spawns: Option<u32>,
    /// The spawn count *c* (claim 12).
    pub spawn_count: i32,
    /// The resource flag (claim 12).
    pub tiberium: bool,
    /// The attached particle system (claim 13), by index.
    pub system: Option<u32>,
    /// The start sound (claim 14).
    pub start_sound: Option<String>,
}

/// The piece types by registration index.
#[derive(Clone, Debug, Default)]
pub struct PieceRules {
    /// Types by registration index.
    pub types: Vec<PieceType>,
}

impl PieceRules {
    /// The piece rules of a ruleset: `warheads` the warhead names by index,
    /// `gas` the particle-system types.
    pub fn new(
        r: &glasnost_rules::schema::Ruleset,
        warheads: &[String],
        gas: &crate::gas::GasRules,
        mode: Rounding,
    ) -> Self {
        let names = r
            .registries
            .get("voxel_animations")
            .cloned()
            .unwrap_or_default();
        let index_of = |n: &Option<String>| -> Option<u32> {
            let n = n.as_ref()?;
            // spec: S-0004 claim 13 (a type name compared without regard to letter case)
            names
                .iter()
                .position(|x| x.eq_ignore_ascii_case(n))
                .and_then(|i| u32::try_from(i).ok())
        };
        let num = |t: &str| glasnost_num::decimal::nearest_double(t);
        let types = names
            .iter()
            .map(|n| {
                let t = r.sim.voxel_animations.get(n).cloned().unwrap_or_default();
                let dec = |d: &Option<glasnost_rules::sim::Decimal>, default: &str| {
                    d.as_ref().map_or_else(|| num(default), |d| d.value(mode))
                };
                // spec: S-0317 Import mapping (MaxXYVel 15.0, MinZVel 3.5, MaxZVel 5.0 when absent; doubles)
                let m = dec(&t.max_xy_vel, "15.0");
                let zmin = dec(&t.min_z_vel, "3.5");
                let zmax = dec(&t.max_z_vel, "5.0");
                // spec: S-0317 Import mapping and Constants (an angular velocity in degrees times 0.01745277777777778; absent from every read the creation defaults 0 and 10 degrees)
                let to_rad = num("0.01745277777777778");
                let amin = dec(&t.min_angular_velocity, "0").mul(to_rad, mode);
                let amax = dec(&t.max_angular_velocity, "10").mul(to_rad, mode);
                let one = F64::from_i64(1, mode);
                // spec: S-0317 F1 (S_xy = trunc(2M), S_z = trunc(Z_max − Z_min + 1)) and F5 (S_a = trunc(A_max − A_min + 1)), S-0205 claim 7's conversion
                let sxy = m.mul(F64::from_i64(2, mode), mode).to_i32_common();
                let sz = zmax.sub(zmin, mode).add(one, mode).to_i32_common();
                let sa = amax.sub(amin, mode).add(one, mode).to_i32_common();
                // spec: S-0317 Import mapping (Elasticity absent 0.8 in single precision, widened; a double field otherwise) and claim 10 (e as imported)
                let elasticity = t.elasticity.as_ref().map_or_else(
                    || crate::gas::nearest_single("0.8").to_f64(),
                    |d| d.value(mode),
                );
                PieceType {
                    name: n.clone(),
                    // spec: S-0317 Import mapping (Duration absent 30)
                    duration: t.duration.unwrap_or(30),
                    // spec: S-0317 Import mapping (IsMeteor absent no)
                    meteor: t.is_meteor.unwrap_or(false),
                    m,
                    zmin,
                    sxy,
                    sz,
                    sa,
                    amin,
                    elasticity,
                    // spec: S-0317 F8 (ε: the imported elasticity narrowed once to single precision)
                    epsilon: single(elasticity, mode),
                    // spec: S-0317 Import mapping (Damage and DamageRadius absent 0, no warhead)
                    damage: t.damage.unwrap_or(0),
                    radius: t.damage_radius.unwrap_or(0),
                    warhead: t
                        .warhead
                        .as_ref()
                        .and_then(|h| warheads.iter().position(|w| w == h))
                        .and_then(|i| u32::try_from(i).ok()),
                    bounce: t.bounce_anim.clone(),
                    expire: t.expire_anim.clone(),
                    trailer: t.trailer_anim.clone(),
                    spawns: index_of(&t.spawns),
                    // spec: S-0317 Import mapping (SpawnCount absent 0)
                    spawn_count: t.spawn_count.unwrap_or(0),
                    // spec: S-0317 Import mapping (IsTiberium absent no)
                    tiberium: t.is_tiberium.unwrap_or(false),
                    system: t.attached_system.as_deref().and_then(|s| gas.system(s)),
                    start_sound: t.start_sound.clone(),
                }
            })
            .collect();
        Self { types }
    }

    /// A piece type's index by name (S-0004 claim 13: without regard to
    /// letter case).
    pub fn index(&self, name: &str) -> Option<u32> {
        // spec: S-0004 claim 13 (a type name compared without regard to letter case)
        self.types
            .iter()
            .position(|t| t.name.eq_ignore_ascii_case(name))
            .and_then(|i| u32::try_from(i).ok())
    }
}

/// One piece in play (S-0317 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    /// The piece type's registration index.
    pub type_index: u32,
    /// The house its producer gives it (claim 1: it changes no outcome).
    pub house: Option<HouseId>,
    /// The lifetime counter (claims 2 and 6).
    pub counter: i32,
    /// Its position: the creation point (a falling piece) or the start
    /// position (a meteor piece) until its first flying turn, then the
    /// flight point (claims 3, 4 and 11).
    pub pos: Pos,
    /// The flight position, single precision (F3).
    pub flight: [F32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The velocity, single precision, leptons per tick (F1 to F3).
    pub velocity: [F32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The spin rate in radians per tick, a double (F5); drawing only.
    pub spin: F64,
    /// The spin axis (*a*₃, *a*₂, *a*₁) before normalisation (F5); drawing
    /// only.
    pub axis: [i32; 3], // nocite: vector and matrix dimensions (three axes)
    /// The net count *n*₊ − *n*₋ of its flying turns (claim 22, F10): the
    /// drawing turns the model about its axis by the spin rate times this.
    /// Wraps as an `i32`; drawing only.
    pub turns: i32,
    /// Whether the sense of the turn is reversed (claim 22: each contact
    /// reverses it); drawing only.
    pub reversed: bool,
    /// Made through a spawn link on a loop of piece types: it makes no
    /// piece through such a link itself (#174; [`crate::loops`]).
    #[serde(default)]
    pub looped: bool,
}

impl Piece {
    /// The flight point: the flight position truncated toward zero in each
    /// axis (claim 11; S-0205 claim 7's conversion).
    pub fn flight_point(&self) -> Pos {
        let [x, y, z] = self.flight.map(|c| c.to_f64().to_i32_common());
        Pos::new(x, y, z)
    }
}

/// |r| rem s for a raw value r read as signed (S-0317 Formulas preamble,
/// S-0202 claim 6), as an integer that converts exactly to a double.
fn mag_rem(raw: u32, s: i32, overflow: bool) -> i64 {
    let r = raw as i32;
    if r == i32::MIN && !overflow {
        // spec: S-0317 Formulas (quirks.random_magnitude_overflow off: |−2147483648| is 2147483648) and claim 16 (a span of 0: the remainder is the dividend, S-0205 claim 3)
        let a = i64::from(r).abs();
        return if s == 0 { a } else { a % i64::from(s) };
    }
    // spec: S-0317 Formulas (the "magnitude first" remainder, truncated toward zero; −2147483648 keeps its sign under quirks.random_magnitude_overflow) and claim 16 (a span of 0: the remainder is the dividend, S-0205 claim 3)
    i64::from(glasnost_num::int::rem(r.wrapping_abs(), s))
}

/// A single-precision store (S-0317 Terms: RZ₂₄ under
/// `quirks.chop_rounding`, RN₂₄ with it off).
fn single(x: F64, mode: Rounding) -> F32 {
    x.to_f32(mode)
}

/// Makes a piece of type `ti` at the creation point `at` for `house`
/// (claims 2 to 4, 13 and 14): its draws, its entry in the update order,
/// its attached system and its start sound. Returns the piece.
pub fn create(w: &mut World, ti: u32, at: Pos, house: Option<HouseId>) -> Option<ObjectId> {
    create_marked(w, ti, at, house, false)
}

/// [`create`] for a piece made through a spawn link on a loop of piece
/// types when `looped` ([`Piece::looped`]).
fn create_marked(
    w: &mut World,
    ti: u32,
    at: Pos,
    house: Option<HouseId>,
    looped: bool,
) -> Option<ObjectId> {
    let rules = w.rules_arc();
    let t = rules.pieces.types.get(ti as usize)?;
    let mode = rules.mode;
    let q = rules.quirks;
    let half = q.ranged_draw_half_span;
    let ov = q.random_magnitude_overflow;
    let d = |x: i64| F64::from_i64(x, mode);
    // spec: S-0202 claim 10 (revision 36: a voxel-model debris piece is not an animation of S-0109's sprite kind, so the creation-draw table's bouncing row does not apply; its creation draws are S-0317 claims 3 and 4)
    let (pos, flight, velocity, counter, u) = if t.meteor {
        // spec: S-0317 claim 4 (a raw value for the y speed, one for the x speed, one that shortens the lifetime, one for the spin rate)
        let q1 = w.rng.raw();
        let q2 = w.rng.raw();
        let q3 = w.rng.raw();
        let q4 = w.rng.raw();
        // spec: S-0317 F2 (v_y = (|q₁| rem S_xy) − M, v_x = (|q₂| rem S_xy) − M, each stored in single precision; v_z = Z_min stored in single precision)
        let mut vy = d(mag_rem(q1, t.sxy, ov)).sub(t.m, mode).to_f32(mode);
        let mut vx = d(mag_rem(q2, t.sxy, ov)).sub(t.m, mode).to_f32(mode);
        let vz = single(t.zmin, mode);
        // spec: S-0317 claim 4 and F2 (when v_x < −v_y both horizontal components change sign)
        if vx.to_f64().lt(vy.to_f64().negated()) {
            vx = vx.to_f64().negated().to_f32(mode);
            vy = vy.to_f64().negated().to_f32(mode);
        }
        // spec: S-0317 F2 (the counter T is reduced by |q₃ rem 20|, the remainder before the absolute value, a signed 32-bit wrap, S-0205)
        let cut = glasnost_num::int::rem(q3 as i32, 20).abs(); // spec: S-0317 F2 and Constants (meteor lifetime cut 0 to 19)
        let counter = t.duration.wrapping_sub(cut);
        let tt = d(i64::from(counter));
        // spec: S-0317 F2 (start = trunc(c − T·v) per axis, formed in double precision from T, the stored speeds and the creation point)
        let back = |c: i32, v: F32| {
            d(i64::from(c))
                .sub(tt.mul(v.to_f64(), mode), mode)
                .to_i32_common()
        };
        let start = Pos::new(back(at.x, vx), back(at.y, vy), back(at.z, vz));
        let flight = [start.x, start.y, start.z].map(|c| d(i64::from(c)).to_f32(mode));
        // spec: S-0317 claim 6 (a meteor piece is at its start position until its first turn)
        (start, flight, [vx, vy, vz], counter, q4)
    } else {
        // spec: S-0317 claim 3 (four raw values: the spin rate, the vertical speed, the y speed, the x speed)
        let r1 = w.rng.raw();
        let r2 = w.rng.raw();
        let r3 = w.rng.raw();
        let r4 = w.rng.raw();
        // spec: S-0317 F1 (v_x = (|r₄| rem S_xy) − M, v_y = (|r₃| rem S_xy) − M, v_z = (|r₂| rem S_z) + Z_min; formed in double precision, stored in single precision)
        let vx = d(mag_rem(r4, t.sxy, ov)).sub(t.m, mode).to_f32(mode);
        let vy = d(mag_rem(r3, t.sxy, ov)).sub(t.m, mode).to_f32(mode);
        let vz = d(mag_rem(r2, t.sz, ov)).add(t.zmin, mode).to_f32(mode);
        // spec: S-0317 F1 and Constants (the start position is the creation point plus (0, 0, 10) leptons, in single precision)
        let lift = 10; // spec: S-0317 Constants (start lift of a falling piece 10 leptons)
        let flight = [
            d(i64::from(at.x)).to_f32(mode),
            d(i64::from(at.y)).to_f32(mode),
            d(i64::from(at.z) + lift).to_f32(mode),
        ];
        // spec: S-0317 claims 2 and 3 (the counter starts at the duration; the position is the creation point until the first turn)
        (at, flight, [vx, vy, vz], t.duration, r1)
    };
    // spec: S-0317 F5 and Constants (three ranged draws R(−65535, 65535); the axis is (a₃, a₂, a₁))
    let a1 = w.rng.ranged(-65535, 65535, half);
    let a2 = w.rng.ranged(-65535, 65535, half); // spec: S-0317 F5 and Constants (R(−65535, 65535))
    let a3 = w.rng.ranged(-65535, 65535, half); // spec: S-0317 F5 and Constants (R(−65535, 65535))
    // spec: S-0317 F5 (spin rate = (|u| rem S_a) + A_min, kept in double precision)
    let spin = d(mag_rem(u, t.sa, ov)).add(t.amin, mode);
    let id = w.new_id();
    // spec: S-0317 claim 21 and Constants (a piece whose placement point is exactly (0, 0, 0) is not placed: it never enters the update order, never takes a turn and is never drawn; its system and start sound are still made)
    // Our engine keeps no record of a piece that is not placed (design):
    // nothing reads it again, its system keeps it as an owner that never
    // moves and is never marked (S-0616 claims 4 and 5), and its start sound
    // stays at the creation point. The returned number is still its own.
    if pos != Pos::new(0, 0, 0) {
        w.pieces.insert(
            id,
            Piece {
                type_index: ti,
                house,
                counter,
                pos,
                flight,
                velocity,
                spin,
                axis: [a3, a2, a1],
                // spec: S-0317 claim 22 (the identity at creation)
                turns: 0,
                reversed: false,
                looped,
            },
        );
        // spec: S-0317 claims 1 and 6 (a piece enters play during its creation) and S-0203 claim 3 (appended to the update order)
        w.update.enter_new(id);
    }
    if let Some(si) = t.system {
        crate::emitters::voxel_animation_system(w, si, id, at, pos);
    }
    if let Some(s) = t.start_sound.clone() {
        // spec: S-0317 claim 14 (the start sound at the creation point, then moving with the piece)
        w.effects.push(crate::combat::effects::Effect::PieceSound {
            id,
            sound: s,
            pos: at,
        });
    }
    Some(id)
}

/// Makes a piece of the type named `name` (the producers of S-0303 claims
/// 19 and 22 and S-0312 F1); a name that is no piece type makes none.
pub fn create_named(w: &mut World, name: &str, at: Pos, house: Option<HouseId>) {
    let Some(ti) = w.rules().pieces.index(name) else {
        w.note(format!(
            "S-0317 claim 1: the debris type {name} is not a listed voxel-animation type; no piece is made"
        ));
        return;
    };
    create(w, ti, at, house);
}

/// The ground height at a position (S-0317 claim 10: S-0106 claim 2's
/// surface height at the position truncated toward zero). Off the legal map
/// it is the off-map cell's 0 (S-0317 claim 20).
pub(crate) fn ground(w: &World, p: Pos) -> i32 {
    // spec: S-0317 claim 20 and Constants (the off-map cell: ground height 0)
    w.surface_at(p.x, p.y)
}

/// A cell's height level; the off-map cell's 0 for a cell that is not
/// legal (S-0317 claim 20).
pub(crate) fn level(w: &World, c: CellXY) -> i32 {
    // spec: S-0317 claim 20 and Constants (the off-map cell: height level 0)
    w.cells.cell(c).map_or(0, |x| i32::from(x.level))
}

/// Whether a cell has the water land type (claims 8 and 11); the off-map
/// cell is of the clear land type (S-0317 claim 20).
pub(crate) fn water(w: &World, c: CellXY) -> bool {
    // spec: S-0317 claim 20 (the off-map cell is clear, not water)
    w.cells.cell(c).is_some_and(|x| x.land == land::WATER)
}

/// The slope shape of a cell (S-0106); the off-map cell's 0 (S-0317 claim
/// 20).
pub(crate) fn slope(w: &World, c: CellXY) -> usize {
    // spec: S-0317 claims 17 and 20 (the slope shape of the cell read; the off-map cell is flat, shape 0)
    w.cells.cell(c).map_or(0, |x| usize::from(x.slope))
}

/// Whether the cell of a new position counts as holding a building for
/// claim 10's building contact (S-0317 claim 19).
pub(crate) fn building_cell(w: &World, c: CellXY) -> bool {
    let rules = w.rules();
    // spec: S-0317 claim 19 (the first building listed in the cell's ground layer, S-0209 claim 13's listing order, decides)
    let first = w
        .cells
        .list(c, Layer::Ground)
        .iter()
        .copied()
        .find(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Building));
    match first {
        Some(id) => {
            let Some(o) = w.obj(id) else { return false };
            // spec: S-0317 claim 19 (an inactive laser-fence section does not count; section states are not modelled, so every section counts as inactive, as for S-0613 claim 10)
            let fence = rules
                .buildings
                .get(o.type_index as usize)
                .is_some_and(|b| b.building.laser_fence);
            // spec: S-0317 claim 19 (a vehicle-like building, S-0214 Terms, does not count)
            !fence && !crate::targeting::vehicle_like(w, id)
        }
        // spec: S-0317 claim 19 (no building listed: the cell counts when its overlay is one of S-0306's three wall overlays)
        None => w
            .cells
            .cell(c)
            .is_some_and(|cell| crate::gas::WALLS.contains(&cell.overlay)),
    }
}

/// What a move met (claim 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Contact {
    /// No contact.
    None,
    /// A ground or building contact, or a deck crossing from below.
    Ground,
    /// A deck crossing from above that put the piece on the deck.
    OnDeck,
}

/// One piece's turn (claim 6).
pub fn turn(w: &mut World, id: ObjectId) {
    let Some(mut pc) = w.pieces.get(&id).cloned() else {
        return;
    };
    let rules = w.rules_arc();
    let Some(t) = rules.pieces.types.get(pc.type_index as usize) else {
        return;
    };
    // spec: S-0317 claim 6 (the counter is reduced by 1 unless it is 0; −2147483648 wraps to 2147483647, S-0205)
    if pc.counter != 0 {
        pc.counter = pc.counter.wrapping_sub(1);
    }
    if pc.counter <= 0 {
        // spec: S-0317 claim 6 (a counter of 0 or less ends the piece, which leaves play in that turn)
        w.pieces.insert(id, pc);
        end(w, id);
        return;
    }
    // spec: S-0317 claim 6 and Constants (on a tick with an even tick number, the trailer before the move)
    if w.tick.rem_euclid(2) == 0
        && let Some(name) = t.trailer.clone()
    {
        let at = trailer_point(
            &pc,
            w.rules().quirks.voxel_debris_trailer_height,
            rules.mode,
        );
        // spec: S-0317 claim 6 (an initial loop delay of 1 update, repeat factor 1, no reverse request: S-0109 claim 29's projectile trailer)
        crate::combat::effects::trailer(w, &name, at);
    }
    // spec: S-0317 claim 7 (the position before the move: the creation point before a falling piece's first flight, else the previous turn's flight point)
    let old = pc.pos;
    let impact = fly(w, &mut pc, t);
    w.pieces.insert(id, pc);
    if let Some((cell, at, deck)) = impact {
        land_impact(w, t, old, cell, at, deck);
    }
}

/// F7: the trailer point from the flight position before the move.
fn trailer_point(pc: &Piece, too_high: bool, mode: Rounding) -> Pos {
    let [x, y, z] = pc.flight;
    let (tx, ty) = (x.to_f64().to_i32_common(), y.to_f64().to_i32_common());
    let h = if too_high {
        // spec: S-0317 F7 and Constants (h = conv(k × z + 0.5), formed in double precision, k = 6.967, our engine's value)
        let k = crate::rules::nearest_double("6.967");
        let half = crate::rules::nearest_double("0.5"); // spec: S-0317 F7 (+ 0.5)
        k.mul(z.to_f64(), mode).add(half, mode).to_i32_common()
    } else {
        // spec: S-0317 Edge cases (quirks.voxel_debris_trailer_height off: h = trunc z, the flight point)
        z.to_f64().to_i32_common()
    };
    Pos::new(tx, ty, h)
}

/// Claim 5, F3, claims 9 and 10: the flight of one turn, its contact,
/// rebound and stop. Returns the land impact to make (the cell whose
/// objects are hit, the position the distances are taken from, and whether
/// the deck test put the piece on the deck), or `None`.
fn fly(w: &mut World, pc: &mut Piece, t: &PieceType) -> Option<(CellXY, Pos, bool)> {
    let rules = w.rules_arc();
    let mode = rules.mode;
    let q = rules.quirks;
    let g = gravity();
    let p0 = pc.flight;
    let v0 = pc.velocity;
    let old = pc.pos;
    // spec: S-0317 claim 10 (u: the vertical speed at the start of the turn, before gravity)
    let u = v0[2].to_f64();
    // spec: S-0317 claim 5 and F3 (v′ = (v_x, v_y, RZ₂₄(v_z − g)), p′ = p + v′ per axis, each stored in single precision)
    let vz = single(u.sub(g, mode), mode);
    let mut v = [v0[0], v0[1], vz];
    let mut p = [0, 1, 2].map(|i| single(p0[i].to_f64().add(v[i].to_f64(), mode), mode)); // nocite: vector and matrix dimensions (three axes)
    // nocite: vector and matrix dimensions (three axes)
    let trunc = |p: &[F32; 3]| {
        let [x, y, z] = p.map(|c| c.to_f64().to_i32_common());
        Pos::new(x, y, z)
    };
    let new = trunc(&p);
    let (oc, nc) = (old.cell(), new.cell());
    let f = ground(w, new);
    let fd = F64::from_i64(i64::from(f), mode);
    let z0f = p0[2].to_f64(); // nocite: vector and matrix dimensions (three axes)
    let z1f = p[2].to_f64(); // nocite: vector and matrix dimensions (three axes)
    let at = |h: i32| F64::from_i64(i64::from(h), mode).to_f32(mode);
    let mut contact = Contact::None;
    // spec: S-0317 claim 10 (the deck height: the ground height plus 416 leptons at the new position)
    let deck = f.wrapping_add(DECK);
    let dd = F64::from_i64(i64::from(deck), mode);
    let bridge = w.high_bridge(nc) || w.high_bridge(oc);
    if bridge && z0f.ge(dd) && z1f.lt(dd) {
        // spec: S-0317 claim 10 (a high-bridge cell: a move from at or above the deck height to below it puts the height at the deck height)
        p[2] = at(deck);
        contact = Contact::OnDeck;
    } else if bridge && z0f.lt(dd) && z1f.ge(dd) {
        // spec: S-0317 claim 10 (a move from below the deck height to at or above it puts the height 20 leptons below the deck height)
        p[2] = at(deck - 20); // spec: S-0317 claim 10 (20 leptons below the deck height)
        contact = Contact::Ground;
    } else if z1f.lt(fd) {
        // spec: S-0317 claim 10 (the new height below the ground height reaches the ground; set to it when less than 100 leptons below, else left where it is)
        if fd.sub(z1f, mode).lt(F64::from_i64(100, mode)) {
            p[2] = at(f);
        }
        contact = Contact::Ground;
    } else if z1f.lt(F64::from_i64(i64::from(f) + 150, mode)) // spec: S-0317 claim 10 and Constants (less than 150 leptons above the ground height: the building band)
        && building_cell(w, nc)
    {
        // spec: S-0317 claims 10 and 19 (at least the ground height and less than 150 leptons above it, in a cell that counts as holding a building: a contact at the ground height)
        p[2] = at(f);
        contact = Contact::Ground;
    }
    let mut impact_at = new;
    let mut undone = false;
    if contact != Contact::None {
        let z1 = p[2].to_f64(); // nocite: vector and matrix dimensions (three axes)
        // spec: S-0317 claim 10 (z₀: the single-precision height of the flight position before the move, not of the position)
        let z0 = z0f;
        let one = F64::from_i64(1, mode);
        // spec: S-0317 claim 10 and Constants (the cliff test: the new cell at least 2 height levels above the old one)
        let cliff = level(w, nc) >= level(w, oc) + HIGH;
        // spec: S-0317 claim 10 and Constants (the cliff speed line −0.0002, compared in double precision with u widened exactly)
        let line = crate::rules::nearest_double("-0.0002");
        // spec: S-0317 claim 10 (undone when u < −0.0002 and z₁ > z₀; for any other u, 0 included, when z₁ > z₀ + u + 1)
        let lifted = if u.lt(line) {
            z1.gt(z0)
        } else {
            z1.gt(z0.add(u, mode).add(one, mode))
        };
        if cliff && lifted {
            undone = true;
            // spec: S-0317 claim 10 (the move undone: the flight position and the velocity become those at the start of the turn; the impact's position is the position before the move)
            p = p0;
            // spec: S-0317 claim 18 and F9 (the velocity of the start of the turn turned off the cliff face and scaled by e as imported)
            v = cliff_turn(
                w,
                v0,
                oc,
                nc,
                t.elasticity,
                q.voxel_debris_cliff_step_faults,
                mode,
            );
            impact_at = old;
        } else {
            // spec: S-0317 claim 17 (the slope shape of the cell of the new position; at a deck contact the ground under the deck, quirks.voxel_debris_deck_rebound_slope; off: the flat rule there)
            let shape = if contact == Contact::OnDeck && !q.voxel_debris_deck_rebound_slope {
                0
            } else {
                slope(w, nc)
            };
            v = rebound(&rules.gas.ramps, shape, v, t.epsilon, mode);
        }
    }
    // spec: S-0317 claim 9 and F6 (the stop test after the move and any rebound, before a meteor piece's addition of g)
    let stop = stopped(w, &p, &v, g, mode, q.table_root_distance);
    if t.meteor {
        // spec: S-0317 claim 5 and F3 (a meteor piece: v_z = RZ₂₄(w_z + g) after the move and any rebound)
        v[2] = single(v[2].to_f64().add(g, mode), mode);
    }
    // spec: S-0317 claim 22 (every contact, one the cliff test undoes included, reverses the sense of the turn before this flying turn's rotation; every flying turn then turns once in the current sense)
    if contact != Contact::None {
        pc.reversed = !pc.reversed;
    }
    // spec: S-0317 claim 22 and F10 (the net count n₊ − n₋ of flying turns)
    pc.turns = pc.turns.wrapping_add(if pc.reversed { -1 } else { 1 });
    pc.flight = p;
    pc.velocity = v;
    // spec: S-0317 F3 and claim 11 (the position after the move: p′ truncated toward zero in each axis)
    pc.pos = trunc(&p);
    if stop {
        // spec: S-0317 claim 9 (a stop sets the counter to 0; a stop in a contact turn has no bounce animation, hits or water test)
        pc.counter = 0;
        return None;
    }
    if contact == Contact::None {
        return None;
    }
    let on_deck = contact == Contact::OnDeck && !undone;
    let over_water = water(w, oc);
    // spec: S-0317 claim 8 and Edge cases (a contact whose position before the move is in a water cell: no animation, no hit, the counter 0; quirks.voxel_debris_deck_over_water off: one the deck test puts on the deck is on land)
    if over_water && !(on_deck && !q.voxel_debris_deck_over_water) {
        pc.counter = 0;
        return None;
    }
    // spec: S-0317 claim 7 (the hits in the one cell that holds the new position; the position before the move when a cliff undoes the move, claim 10)
    Some((impact_at.cell(), impact_at, on_deck))
}

/// F8: the rebound of the velocity `v` that the move leaves, off a cell of
/// slope shape `shape`, scaled by *ε*.
// nocite: vector and matrix dimensions (three axes)
pub(crate) fn rebound(
    ramps: &[[[F32; 3]; 3]],
    shape: usize,
    v: [F32; 3],
    eps: F32,
    mode: Rounding,
) -> [F32; 3] {
    use crate::gas::{diagonal, identity, transform, transposed};
    // spec: S-0317 F8 (M: S-0106 table C4 for the slope shape, rows as C4 lists them; shapes 0 and 17 to 20 give the identity)
    let m = ramps.get(shape).copied().unwrap_or_else(identity);
    // spec: S-0317 F8 (S = diag(1, −1, 1), Z = diag(1, 1, −1))
    let s = diagonal("1", "-1", "1");
    let z = diagonal("1", "1", "-1");
    // spec: S-0317 F8 (w = Mᵀ · S · v, each completed component stored in single precision, the row sums grouped as S-0613 F3; a change of sign is exact)
    let w = transform(&transposed(&m), transform(&s, v, mode), mode);
    // spec: S-0317 F8 (w′ = Z · (ε w), each product ε wᵢ stored in single precision)
    let scaled = w.map(|c| single(c.to_f64().mul(eps.to_f64(), mode), mode));
    let w = transform(&z, scaled, mode);
    // spec: S-0317 F8 (v″ = S · M · w′, each completed component stored in single precision)
    transform(&s, transform(&m, w, mode), mode)
}

/// The face of F9's table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Face {
    /// Across *x*: (−*u*ₓ, *u*ᵧ, *u*_z).
    X,
    /// Across *y*: (*u*ₓ, −*u*ᵧ, *u*_z).
    Y,
    /// Diagonal.
    Diagonal,
    /// The north-west turn (*c*, *u*ₓ, *u*ᵧ).
    NorthWest,
}

/// F9: the cliff turn of the velocity `u` of the start of the turn, for a
/// move from the cell `oc` to `nc`, scaled by *e* as imported.
// nocite: vector and matrix dimensions (three axes)
pub(crate) fn cliff_turn(
    w: &World,
    u: [F32; 3],
    oc: CellXY,
    nc: CellXY,
    e: F64,
    faults: bool,
    mode: Rounding,
) -> [F32; 3] {
    // spec: S-0317 claim 18 (the step direction d from the old cell to the new, glossary); a move of more than one cell is a departure: design, the step direction of the signs of the two cell changes
    let dx = nc.x.wrapping_sub(oc.x).signum();
    let dy = nc.y.wrapping_sub(oc.y).signum();
    let base = level(w, oc);
    // spec: S-0317 Terms and Constants (a high cell: a height level at least the old cell's plus 2) and claim 20 (a neighbour that names no legal cell is the off-map cell, level 0)
    let high = |ox: i32, oy: i32| {
        let c = CellXY::new(oc.x.wrapping_add(ox), oc.y.wrapping_add(oy));
        level(w, c) >= base + HIGH
    };
    // spec: S-0317 F9 (only P high gives the first face, only Q high the second, otherwise diagonal)
    let pick = |p: (i32, i32), q: (i32, i32), on_p: Face, on_q: Face| match (
        high(p.0, p.1),
        high(q.0, q.1),
    ) {
        (true, false) => on_p,
        (false, true) => on_q,
        _ => Face::Diagonal,
    };
    let face = match (dx, dy) {
        // spec: S-0317 F9 (d = 0 and 4: across y)
        (0, _) => Face::Y,
        // spec: S-0317 F9 (d = 2 and 6: across x)
        (_, 0) => Face::X,
        // spec: S-0317 F9 (d = 1, quirks.voxel_debris_cliff_step_faults: P = (x − 1, y) across y, Q = (x, y + 1) across x)
        (1, -1) if faults => pick((-1, 0), (0, 1), Face::Y, Face::X),
        // spec: S-0317 F9 (d = 1 with the switch off: (x + 1, y) across x, (x, y − 1) across y)
        (1, -1) => pick((1, 0), (0, -1), Face::X, Face::Y),
        // spec: S-0317 F9 (d = 3: P = (x + 1, y) across x, Q = (x, y + 1) across y)
        (1, _) => pick((1, 0), (0, 1), Face::X, Face::Y),
        // spec: S-0317 F9 (d = 5, quirks.voxel_debris_cliff_step_faults: P = (x + 1, y) across y, Q = (x, y − 1) across x)
        (-1, 1) if faults => pick((1, 0), (0, -1), Face::Y, Face::X),
        // spec: S-0317 F9 (d = 5 with the switch off: (x − 1, y) across x, (x, y + 1) across y)
        (-1, 1) => pick((-1, 0), (0, 1), Face::X, Face::Y),
        // spec: S-0317 F9 (d = 7: P = (x − 1, y) across x, Q = (x, y − 1) the north-west turn; with the switch off across y)
        _ => pick(
            (-1, 0),
            (0, -1),
            Face::X,
            if faults { Face::NorthWest } else { Face::Y },
        ),
    };
    let neg = |c: F32| c.to_f64().negated().to_f32(mode);
    let [ux, uy, uz] = u;
    let turned = match face {
        // spec: S-0317 F9 (across x: (−uₓ, u_y, u_z))
        Face::X => [neg(ux), uy, uz],
        // spec: S-0317 F9 (across y: (uₓ, −u_y, u_z))
        Face::Y => [ux, neg(uy), uz],
        // spec: S-0317 F9 (diagonal, d = 1 or 5: (u_y, uₓ, u_z))
        Face::Diagonal if dx * dy < 0 => [uy, ux, uz],
        // spec: S-0317 F9 (diagonal, d = 3 or 7: (−u_y, −uₓ, u_z))
        Face::Diagonal => [neg(uy), neg(ux), uz],
        // spec: S-0317 F9 and Constants (the north-west turn: (c, uₓ, u_y), c the single-precision 2.3771235942840576)
        Face::NorthWest => [crate::gas::nearest_single("2.3771235942840576"), ux, uy],
    };
    // spec: S-0317 F9 (each component multiplied by e as imported, in double precision, not narrowed first, and stored in single precision)
    turned.map(|c| single(c.to_f64().mul(e, mode), mode))
}

/// F6: whether the piece stops.
// nocite: vector and matrix dimensions (three axes)
pub(crate) fn stopped(
    w: &World,
    p: &[F32; 3],
    v: &[F32; 3],
    g: F64,
    mode: Rounding,
    table: bool,
) -> bool {
    let fp = {
        let [x, y, z] = p.map(|c| c.to_f64().to_i32_common());
        Pos::new(x, y, z)
    };
    let f = F64::from_i64(i64::from(ground(w, fp)), mode);
    // spec: S-0317 F6 (h = trunc(z − f), f the ground height at the flight point)
    let mut h = p[2].to_f64().sub(f, mode).to_i32_common();
    if w.high_bridge(fp.cell()) && h >= DECK {
        // spec: S-0317 F6 (in a high-bridge cell, h of at least 416 is counted from the deck: reduced by 416)
        h -= DECK;
    }
    // spec: S-0317 F6 (S = trunc(ρ(trunc(v_x)² + trunc(v_y)² + trunc(h·g + v_z)²)), ρ the table root of S-0204 F1 under quirks.table_root_distance)
    let a = i64::from(v[0].to_f64().to_i32_common());
    let b = i64::from(v[1].to_f64().to_i32_common());
    let c = i64::from(
        F64::from_i64(i64::from(h), mode)
            .mul(g, mode)
            .add(v[2].to_f64(), mode) // nocite: vector and matrix dimensions (three axes)
            .to_i32_common(),
    );
    let sum = (a * a) as u64 + (b * b) as u64 + (c * c) as u64;
    let s = glasnost_num::geom::root_floor(sum, table, mode);
    // spec: S-0317 F6 and Constants (stopped when S < 2.5, so exactly when S ≤ 2)
    s <= 2
}

/// Claim 7: a ground impact on land: the bounce animation at the position
/// before the move, then the hits.
fn land_impact(w: &mut World, t: &PieceType, old: Pos, cell: CellXY, at: Pos, on_deck: bool) {
    if let Some(a) = t.bounce.clone() {
        // spec: S-0317 claim 7 (the bounce animation at the piece's position before the move, with its creation draws, before the first hit)
        crate::combat::effects::anim(w, &a, old);
    }
    // spec: S-0317 claim 7 and Edge cases (the ground layer whatever the height; quirks.voxel_debris_deck_impact_layer off: an impact the deck test puts on a high-bridge deck hits the deck layer)
    let layer =
        if on_deck && !w.rules().quirks.voxel_debris_deck_impact_layer && w.high_bridge(cell) {
            Layer::Deck
        } else {
            Layer::Ground
        };
    // S-0319 claim 7's listing walk is stated for sprite animations only (S-0317 Open questions leave a piece's open): pieces keep the listing as it stands when the hits begin.
    ground_hits(w, cell, layer, at, t.radius, t.damage, t.warhead, false);
}

/// The hits of a ground impact (S-0317 claim 7 and F4), which a bouncing
/// animation's impact landing shares (S-0319 claim 7): each object listed
/// in `layer` of `cell` within `radius` of `at` by |Δx| + |Δy| receives a
/// hit of `damage` with `warhead`, at the lifted distance. With `walk` the
/// hits go through the listing as it changes (S-0319 claim 7,
/// `quirks.bouncing_anim_impact_hits_walk`); without it through the listing
/// as it stands when the hits begin.
#[allow(clippy::too_many_arguments)]
pub(crate) fn ground_hits(
    w: &mut World,
    cell: CellXY,
    layer: Layer,
    at: Pos,
    radius: i32,
    damage: i32,
    warhead: Option<WarheadId>,
    walk: bool,
) {
    if !walk {
        // spec: S-0317 claim 7 and S-0209 claim 13 (the objects listed in the cell's layer, in listing order; the list as it stands when the hits begin) and S-0319 Edge cases (quirks.bouncing_anim_impact_hits_walk off: each object listed when the hits begin, unless an earlier hit took it out of play; an object entering receives none)
        let list: Vec<ObjectId> = w.cells.list(cell, layer).to_vec();
        for target in list {
            ground_hit(w, target, at, radius, damage, warhead);
        }
        return;
    }
    // spec: S-0319 claim 7 (the hits go through the cell's listing in its order as it changes, S-0209 claim 13)
    let mut next = w.cells.list(cell, layer).first().copied();
    while let Some(target) = next {
        ground_hit(w, target, at, radius, damage, warhead);
        let live = w.cells.list(cell, layer);
        // spec: S-0319 claim 7 (a hit that takes its own object out of the cell's layer ends the hits; the design rule for infantry removed at once and a building that leaves at once: any object no longer listed after its hit ends them)
        let Some(i) = live.iter().position(|&x| x == target) else {
            break;
        };
        // spec: S-0319 claim 7 (the next object is the one now listed after the object just hit: one taken out before it is reached is not hit, one that entered is hit only when listed after it)
        next = live.get(i + 1).copied();
    }
}

/// One hit of a ground impact on `target` (S-0317 claim 7 and F4; S-0319
/// claim 7), its radius test and distance taken from its position now.
fn ground_hit(
    w: &mut World,
    target: ObjectId,
    at: Pos,
    radius: i32,
    damage: i32,
    warhead: Option<WarheadId>,
) {
    let mode = w.rules().mode;
    // spec: S-0319 Edge cases (an object an earlier hit took out of play receives no hit)
    let Some(o) = w.obj(target) else { return };
    // spec: S-0317 F4 (the objects whose horizontal sum |Δx| + |Δy| before the lift is at most the damage radius) and S-0319 claim 7 (taken at the object's own hit, from its position then, with the walk switch on or off)
    let sum = (o.pos.x.wrapping_sub(at.x))
        .wrapping_abs()
        .wrapping_add(o.pos.y.wrapping_sub(at.y).wrapping_abs());
    if sum > radius {
        return;
    }
    // spec: S-0317 F4 (D = L(|xₒ − x| + |yₒ − y|), S-0409 F2's lift applied to the horizontal sum, as S-0613 F7)
    let distance = crate::econ::shroud::height_lift(sum, mode);
    let hit = crate::combat::damage::Hit {
        amount: damage,
        warhead,
        distance,
        // spec: S-0317 claims 1 and 7 and S-0312 claim 4 (no attacker, no attacking house whatever the piece's own house, defences not ignored, no no-escape marker; S-0302 claim 1's table row)
        attacker: None,
        house: None,
        ignore_defences: false,
        no_escape: false,
    };
    // spec: S-0317 claim 7 and S-0319 claim 7 (ordinary damage, S-0302; whatever the amount; no skip for an object with no health left)
    crate::combat::damage::apply(w, target, hit);
}

/// Claim 11: the end of a piece, then its removal.
fn end(w: &mut World, id: ObjectId) {
    let Some(pc) = w.pieces.get(&id).cloned() else {
        return;
    };
    let rules = w.rules_arc();
    let q = rules.quirks;
    let half = q.ranged_draw_half_span;
    if let Some(t) = rules.pieces.types.get(pc.type_index as usize) {
        let pos = pc.pos;
        let fp = pc.flight_point();
        // spec: S-0317 claim 11 (on water: the cell of the position has the water land type; at deck height: at least 416 leptons above the cell's ground height)
        let on_water = water(w, pos.cell());
        let at_deck = pos.z.wrapping_sub(ground(w, pos)) >= DECK;
        if !on_water || at_deck {
            // spec: S-0317 claim 11 and Edge cases (the area damage when an expiry animation is named; quirks.voxel_debris_expiry_needs_animation off: also when the damage amount is not 0)
            let area = if q.voxel_debris_expiry_needs_animation {
                t.expire.is_some()
            } else {
                t.expire.is_some() || t.damage != 0
            };
            if let Some(a) = t.expire.clone() {
                // spec: S-0317 claim 11 (the expiry animation at the flight point)
                crate::combat::effects::anim(w, &a, fp);
            }
            if area {
                // spec: S-0317 claim 11 (a detonation's area damage at the flight point, S-0303 claims 1 to 20: the type's damage and warhead, no source object, no house, resources affected; the light flash is drawing only)
                crate::combat::area::detonate(
                    w,
                    crate::combat::area::Detonation {
                        point: fp,
                        damage: t.damage,
                        warhead: t.warhead,
                        source: None,
                        house: None,
                        resources: true,
                    },
                );
            }
            // spec: S-0317 r59 claim 29 (a marked piece makes no piece, and takes no draw, through a spawn link on a loop of piece types)
            if t.meteor
                && let Some(s) = t.spawns
                && t.spawn_count > 0
                && let Some(looped) = rules.loops.pieces.link(
                    Some(pc.type_index as usize),
                    pc.looped,
                    Some(s as usize),
                )
            {
                // spec: S-0317 claim 12 (n = R(0, c) + R(0, c), the first draw first, a signed 32-bit sum that wraps; a negative sum makes no piece)
                let a = w.rng.ranged(0, t.spawn_count, half);
                let b = w.rng.ranged(0, t.spawn_count, half);
                let n = a.wrapping_add(b);
                for _ in 0..n.max(0) {
                    // spec: S-0317 claim 12 (each further piece at the flight point, with no house, created as claims 3 and 4 say, in turn)
                    create_marked(w, s, fp, None, looped);
                }
            }
        } else {
            let list = rules.combat.general.splash_list.clone();
            if t.meteor {
                // spec: S-0317 claim 11 and Constants (a meteor piece on water: the last splash-list entry 5 leptons higher; none with an empty list, a departure)
                if let Some(s) = list.last() {
                    crate::combat::effects::anim(w, s, Pos::new(pos.x, pos.y, pos.z + 5));
                }
            } else {
                // spec: S-0317 claim 11 (a falling piece on water: the game-wide wake animation at the position)
                if let Some(wake) = rules.wake.clone() {
                    crate::combat::effects::anim(w, &wake, pos);
                }
                // spec: S-0317 claim 11 and Constants (then the first splash-list entry 10 leptons higher; none with an empty list, a departure)
                if let Some(s) = list.first() {
                    crate::combat::effects::anim(w, s, Pos::new(pos.x, pos.y, pos.z + 10));
                }
            }
        }
        if t.tiberium && !on_water && !at_deck {
            resources(w, t.meteor, fp);
        }
    }
    w.pieces.remove(&id);
    // spec: S-0317 claim 6 and S-0203 claim 10 (the piece leaves play: a counting removal under quirks.update_order_removal_skip)
    w.update.leave(id, q.update_order_removal_skip);
    // spec: S-0317 claim 13 and S-0616 claim 4 (its attached system is marked, as for the removal of an owner object)
    crate::particles::owner_removed(w, id);
}

/// Claim 12: new resource at the end of a piece with the resource flag.
fn resources(w: &mut World, meteor: bool, at: Pos) {
    // spec: S-0317 claim 12 (the tiberium of the overlay type at position 127; not a resource overlay or a shorter list: no cell and no draws, a departure)
    let Some(tib) = crate::econ::resource::tiberium_of_overlay(w, 127) else {
        return;
    };
    let c = at.cell();
    let cells: Vec<CellXY> = if meteor {
        // spec: S-0317 claim 12 (a meteor piece: each of the 8 neighbouring cells in step-direction order 0 to 7)
        crate::regions::STEPS
            .iter()
            .map(|&(dx, dy)| CellXY::new(c.x + dx, c.y + dy))
            .collect()
    } else {
        // spec: S-0317 claim 12 (a falling piece: its own cell)
        vec![c]
    };
    for cell in cells {
        // spec: S-0317 claim 12 and S-0401 claims 10 and 11 (each germinable cell becomes a new resource cell at stage 0, cell by cell; others take no draw)
        if crate::econ::resource::germinable(w, cell) {
            crate::econ::resource::add_resource_cell(w, cell, tib, 0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0317 Formulas (the magnitude first, −2147483648 keeping its sign under the switch) and claim 16 (a span of 0 gives the dividend)
    #[test]
    fn magnitude_remainders() {
        assert_eq!(mag_rem(7, 5, true), 2);
        assert_eq!(mag_rem((-7i32) as u32, 5, true), 2);
        assert_eq!(mag_rem(i32::MIN as u32, 30, true), -2147483648 % 30);
        assert_eq!(mag_rem(i32::MIN as u32, 30, false), 2147483648 % 30);
        assert_eq!(mag_rem(9, 0, true), 9);
        assert_eq!(mag_rem(i32::MIN as u32, 0, false), 2147483648);
        assert_eq!(mag_rem(i32::MIN as u32, -1, true), 0);
    }
}
