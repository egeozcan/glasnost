//! Blow-ups of bridge cells (S-0511 claim 9, F2), deck drops, the heading
//! hits of claim 11 and the ground-layer hit shared by claims 9, 11 and 15.

use glasnost_map::CellXY;
use glasnost_num::F64;

use crate::combat::damage::{self, Hit};
use crate::rules::{Kind, Locomotor};
use crate::world::{Layer, ObjectId, Pos, World};

/// What one destroy attempt (or demolition, or repair) has done so far that
/// later steps of it read.
#[derive(Clone, Debug, Default)]
pub struct Ctx {
    /// The blown-up cells, in blow-up order (claim 11).
    pub blown: Vec<CellXY>,
}

/// Claim 9's ground-layer hit: the object's current health, distance 0, the
/// demolition warhead, ignoring defences, with the no-escape marker, no
/// attacker and no attacking house.
pub fn ground_hit(w: &mut World, id: ObjectId) {
    let Some(o) = w.obj(id) else { return };
    // spec: S-0303 claim 9 (an object with no health or off the map takes no hit)
    if o.health <= 0 || !o.on_map {
        return;
    }
    let health = o.health;
    // spec: S-0511 claim 9 (a hit equal to its current health, distance 0, the demolition warhead, ignoring defences, the no-escape marker, no attacker and no attacking house)
    let hit = Hit {
        amount: health,
        warhead: w.rules().combat.general.c4_warhead,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: true,
    };
    damage::apply(w, id, hit);
}

/// The blow-up of cell `c` at height level `level` (claim 9): ground-layer
/// hits, then deck drops, then the explosion draws.
pub fn blow_up(w: &mut World, c: CellXY, level: i32, ctx: &mut Ctx) {
    ctx.blown.push(c);
    // spec: S-0511 claim 9 (ground-layer hits precede deck drops; both precede the explosion draws)
    let ground: Vec<ObjectId> = w.cells.list(c, Layer::Ground).to_vec();
    for id in ground {
        ground_hit(w, id);
    }
    let deck: Vec<ObjectId> = w.cells.list(c, Layer::Deck).to_vec();
    for id in deck {
        drop_from_deck(w, c, id);
    }
    explosions(w, c, level);
}

/// An object listed in `c`'s deck layer leaves the deck (claim 9): it moves
/// to the ground layer of the same cell at its current position, its marks
/// move at once, and it starts falling with the bomb mark (claim 10). Its
/// orders and movement state are kept.
fn drop_from_deck(w: &mut World, c: CellXY, id: ObjectId) {
    let Some(o) = w.obj(id).cloned() else { return };
    let building = o.kind == Kind::Building;
    // spec: S-0511 claim 9 (it moves to the ground layer of the same cell at its current position; no stop order: target, destination, queued destinations, path and movement in progress are kept)
    w.cells.leave(c, Layer::Deck, id);
    w.cells.enter(c, Layer::Ground, id, building);
    if let Some(ob) = w.obj_mut(id) {
        ob.layer = Layer::Ground;
    }
    move_marks(w, c, id, &o);
    // spec: S-0511 claims 9 and 10 (it starts falling with the bomb mark)
    crate::fall::start(w, id, true);
}

/// The marks of object `id` (state `o` before the drop) dropped from the
/// deck of `c` (claim 9), made at once: a stationary infantry unit's
/// standing-position mark and a stationary vehicle's present flag go from
/// the deck record to the ground record; a moving one's reservation at its
/// heading point is cleared at that point's height, with the deck flag its
/// listing change clears.
fn move_marks(w: &mut World, c: CellXY, id: ObjectId, o: &crate::world::Object) {
    let q = crate::objects::occ_quirks(w);
    match o.kind {
        Kind::Infantry => {
            if let Some(t) = o.walk.as_ref().and_then(|s| s.target) {
                // spec: S-0511 claim 9 (movement reservations are cleared at the locomotor's heading point, at that point's stored height)
                let layer = dropped_layer(w, t);
                w.cells.clear_infantry(t.cell(), layer, id, q);
                return;
            }
            let Some(rec) = w.cells.occ(c, Layer::Deck).copied() else {
                return;
            };
            for (p, m) in rec.marks.iter().enumerate() {
                if let Some(crate::cells::Mark {
                    by: crate::cells::Marker::Infantry(who, house),
                    ..
                }) = m
                    && *who == id
                {
                    // spec: S-0511 claim 9 (a stationary infantry unit's standing-position mark is cleared on deck and the corresponding ground mark is made immediately)
                    w.cells.clear_position(c, Layer::Deck, p as u8, q);
                    w.cells.mark_infantry(c, Layer::Ground, p as u8, id, *house);
                }
            }
        }
        Kind::Vehicle => {
            if let Some(h) = o.hover.as_ref() {
                let point = h.point;
                let mut kept = Vec::new();
                for &(cl, l) in &h.claims {
                    let at_point =
                        point.is_some_and(|p| (cl, l) == (p.cell(), dropped_layer(w, p)));
                    if at_point || (cl, l) == (c, Layer::Deck) {
                        // spec: S-0511 claim 9 (the reservation at the heading point, at its stored height, and the deck flag of the cell it leaves are cleared)
                        w.cells.claim_vehicle(cl, l, false);
                        if point.is_none() && (cl, l) == (c, Layer::Deck) {
                            // spec: S-0511 claim 9 (a stationary vehicle's present flag: the ground flag is made immediately)
                            w.cells.claim_vehicle(c, Layer::Ground, true);
                            kept.push((c, Layer::Ground));
                        }
                    } else {
                        // spec: S-0511 claim 9 (unrelated marks remain until their ordinary clearing event)
                        kept.push((cl, l));
                    }
                }
                if let Some(hs) = w.obj_mut(id).and_then(|ob| ob.hover.as_mut()) {
                    hs.claims = kept;
                }
            } else if let Some(d) = o.drive.as_ref() {
                for &(cl, l) in &d.claims {
                    // spec: S-0511 claim 9 and S-0209 claim 1 (the heading point's reservation, set in the layer of the position at that point, and the deck flag of the cell it leaves are cleared, each in the record it was set in)
                    w.cells.claim_vehicle(cl, l, false);
                }
                let stationary = d.track.is_none();
                if stationary {
                    // spec: S-0511 claim 9 (a stationary vehicle's present flag: cleared on deck, the ground flag made immediately)
                    w.cells.claim_vehicle(c, Layer::Ground, true);
                }
                if let Some(ds) = w.obj_mut(id).and_then(|ob| ob.drive.as_mut()) {
                    ds.claims = if stationary {
                        vec![(c, Layer::Ground)]
                    } else {
                        Vec::new()
                    };
                }
            }
        }
        _ => {}
    }
}

/// The record layer of a reservation at point `p` of a unit dropped from a
/// lost deck (claim 9): the deck when the point's stored height is at least
/// 416 leptons above the terrain surface there, in a cell whose deck is or
/// was there. Claim 5 has already taken the high-bridge mark off the lost
/// section's cells and given them the bridge-body mark, so the reservation
/// made on the deck is found by either mark.
fn dropped_layer(w: &World, p: Pos) -> Layer {
    let decked = w
        .cells
        .idx(p.cell())
        .is_some_and(|i| w.cells.deck[i] || w.cells.bridge_body[i]);
    // spec: S-0511 claims 5 and 9, S-0208 claim 19 and Constants (the stored height at least 416 above the terrain surface at the point, in a cell whose deck is or was there)
    if decked
        && p.z
            >= w.surface_at(p.x, p.y)
                .wrapping_add(crate::teleport::DECK_HEIGHT)
    {
        Layer::Deck
    } else {
        Layer::Ground
    }
}

/// The 0.5 of F2 and claim 9.
fn half(mode: glasnost_num::Rounding) -> F64 {
    // spec: S-0511 F2 and claim 9 (0.5)
    F64::from_parts(false, 1, -1, mode)
}

/// One coordinate of F2: trunc(256 x + 128 + (u − 0.5) × 50), in double
/// precision, truncated after the sum.
fn f2(cell: i32, u: F64, mode: glasnost_num::Rounding) -> i32 {
    // spec: S-0511 F2 (256 x + 128 + (u − 0.5) × 50, double precision, truncated toward zero after the sum)
    let base = F64::from_i64(i64::from(256 * cell + 128), mode);
    let fifty = F64::from_i64(50, mode); // spec: S-0511 Constants (explosion scatter 50 leptons)
    base.add(u.sub(half(mode), mode).mul(fifty, mode), mode)
        .to_i32_common()
}

/// The explosion draws of a blown-up cell (claim 9, F2).
fn explosions(w: &mut World, c: CellXY, level: i32) {
    let g = w.rules().combat.general.clone();
    // spec: S-0511 claim 9 (an empty bridge-explosion list: no draws)
    if g.bridge_explosions.is_empty() {
        return;
    }
    let mode = w.rules().mode;
    let half_span = w.rules().quirks.ranged_draw_half_span;
    let u1 = w.rng.unit(mode);
    // spec: S-0511 claim 9 (nothing more unless u₁ < 0.95)
    if !u1.lt(crate::consts::BRIDGE_EXPLOSION_CHANCE) {
        return;
    }
    let u2 = w.rng.unit(mode);
    let u3 = w.rng.unit(mode);
    // spec: S-0511 F2 (Z = 104 ℓ + 416)
    let at = Pos::new(f2(c.x, u2, mode), f2(c.y, u3, mode), 104 * level + 416);
    let u4 = w.rng.unit(mode);
    // spec: S-0511 claim 9 (u₄ < 0.5: a debris animation chosen by a ranged draw over the metallic-debris list)
    if u4.lt(half(mode)) {
        let n = g.metallic_debris.len() as i32;
        let k = w.rng.ranged(0, n - 1, half_span);
        // spec: S-0511 Edge cases (an empty metallic-debris list: the draw is taken, no debris animation)
        if let Some(name) = g.metallic_debris.get(k as usize).cloned() {
            crate::combat::effects::anim(w, &name, at);
        }
    }
    explosion_anim(w, &g.bridge_explosions, at);
}

/// An explosion animation at `at`: R(1, 5) for its loop delay, then a
/// ranged draw over the bridge-explosion list for its type (claim 9, F2).
fn explosion_anim(w: &mut World, list: &[String], at: Pos) {
    let half_span = w.rules().quirks.ranged_draw_half_span;
    // spec: S-0511 claim 9 and Constants (loop delay R(1, 5))
    let _delay = w.rng.ranged(1, 5, half_span);
    let n = list.len() as i32;
    let k = w.rng.ranged(0, n - 1, half_span);
    // spec: S-0511 Edge cases (an empty list in the low form: the draw is taken, nothing is created)
    if let Some(name) = list.get(k as usize) {
        crate::combat::effects::anim(w, name, at);
    }
}

/// F2's low form for one cell (hut demolition, claim 16): two unit-interval
/// draws for X and Y with Z = 104 ℓ, then R(1, 5) and the list choice.
pub fn low_form(w: &mut World, c: CellXY) {
    let mode = w.rules().mode;
    let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    let u2 = w.rng.unit(mode);
    let u3 = w.rng.unit(mode);
    // spec: S-0511 F2 low form (Z = 104 ℓ, no 0.95 or 0.5 tests, no debris)
    let at = Pos::new(f2(c.x, u2, mode), f2(c.y, u3, mode), 104 * level);
    let list = w.rules().combat.general.bridge_explosions.clone();
    explosion_anim(w, &list, at);
}

/// The heading tolerance of claim 11, leptons.
// spec: S-0511 claim 11 (a stored height within 104 leptons of P's)
const HEADING_TOLERANCE: i32 = 104;

/// Whether unit `id`'s locomotion is heading to the point at the centre of
/// `c` at height `z` (claims 11 and 15): walk and hover by their heading
/// point's cell and stored height; drive and ship by the same test or the
/// end cell of their current track with the unit's own height, both only
/// with a current heading point; other locomotion never.
pub fn heads_to(w: &World, id: ObjectId, c: CellXY, z: i32) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let Some(u) = w.rules().unit(o.kind, o.type_index) else {
        return false;
    };
    let near = |h: i32| (h - z).abs() <= HEADING_TOLERANCE;
    // spec: S-0511 claim 11 (walk and hover: the current heading point, its cell c and its stored world height within 104 leptons; an object destination is not excluded)
    let point = |p: Option<Pos>| p.is_some_and(|p| p.cell() == c && near(p.z));
    match u.locomotor {
        Locomotor::Walk => point(o.walk.as_ref().and_then(|s| s.target)),
        Locomotor::Hover => point(o.hover.as_ref().and_then(|h| h.point)),
        Locomotor::Drive | Locomotor::Ship => {
            // spec: S-0207 Terms (a drive or ship vehicle's heading point: the target of the track it is on)
            let Some(t) = o.drive.as_ref().and_then(|d| d.track.as_ref()) else {
                // spec: S-0511 claim 11 (both drive and ship tests require a current heading point)
                return false;
            };
            // spec: S-0511 claim 11 and S-0207 claim 24 (the heading-point test: its cell c and its stored height, the vehicle's height at the track's start, within 104 leptons; or the track test: the end cell of the unfinished current track with the unit's current world height)
            t.target_cell == c && (near(t.height) || near(o.pos.z))
        }
        // spec: S-0511 claim 11 (drop-pod, fly, jumpjet, rocket, teleport and tunnel locomotion do not count)
        _ => false,
    }
}

/// The 24 other cells of the 5 × 5 block around `c`, row by row.
pub fn block_others(c: CellXY) -> Vec<CellXY> {
    let mut out = Vec::new();
    // spec: S-0511 Constants (heading search block 5 × 5, the centre cell excluded)
    for dy in -2..=2 {
        for dx in -2..=2 {
            if (dx, dy) != (0, 0) {
                out.push(CellXY::new(c.x + dx, c.y + dy));
            }
        }
    }
    out
}

/// Claim 11: after an attempt's blow-ups, each blown-up cell whose deck
/// record is not empty hits the units of the other 24 cells of its block
/// whose locomotion heads to it, deck layer first, then ground layer.
pub fn heading_hits(w: &mut World, ctx: &Ctx) {
    for &c in &ctx.blown {
        // spec: S-0511 claim 11 (only a blown-up cell whose deck-layer occupancy record is not empty)
        if w.cells.occ(c, Layer::Deck).is_none_or(|o| o.empty()) {
            continue;
        }
        // spec: S-0511 claim 11 (P: the centre of c at its terrain floor + 416 leptons); S-0511 F2 (the floor of level ℓ is 104 ℓ)
        let z = floor(w, c) + crate::teleport::DECK_HEIGHT;
        for n in block_others(c) {
            for layer in [Layer::Deck, Layer::Ground] {
                let ids: Vec<ObjectId> = w.cells.list(n, layer).to_vec();
                for id in ids {
                    // spec: S-0511 claim 11 (each object whose locomotion is heading to P receives claim 9's ground-layer hit)
                    if heads_to(w, id, c, z) {
                        ground_hit(w, id);
                    }
                }
            }
        }
    }
}

/// The height of the terrain floor of `c`: 104 leptons per height level
/// (claims 11 and 15).
pub fn floor(w: &World, c: CellXY) -> i32 {
    // spec: S-0511 F2 (Z = 104 ℓ for the cell's height level ℓ)
    104 * w.cells.cell(c).map_or(0, |x| i32::from(x.level))
}
