//! Blow-ups of bridge cells (S-0511 claim 9, F2), deck drops, the heading
//! hits of claim 11 and the ground-layer hit shared by claims 9, 11 and 15.

use glasnost_map::CellXY;
use glasnost_num::F64;

use crate::combat::damage::{self, Hit};
use crate::rules::{Kind, Locomotor};
use crate::world::{Layer, ObjectId, Pos, Target, World};

/// What one destroy attempt (or demolition, or repair) has done so far that
/// later steps of it read.
#[derive(Clone, Debug, Default)]
pub struct Ctx {
    /// The blown-up cells, in blow-up order (claim 11).
    pub blown: Vec<CellXY>,
    /// Objects dropped from a deck, whose marks move to the ground record
    /// after the attempt's heading hits (Q-0712).
    pub dropped: Vec<ObjectId>,
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
        drop_from_deck(w, c, id, ctx);
    }
    explosions(w, c, level);
}

/// An object listed in `c`'s deck layer leaves the deck (claim 9): it moves
/// to the ground layer of the same cell at its current position, its
/// locomotion stops and it starts falling with the bomb mark (claim 10).
fn drop_from_deck(w: &mut World, c: CellXY, id: ObjectId, ctx: &mut Ctx) {
    let Some(o) = w.obj(id) else { return };
    let building = o.kind == Kind::Building;
    // spec: S-0511 claim 9 (it moves to the ground layer of the same cell at its current position)
    w.cells.leave(c, Layer::Deck, id);
    w.cells.enter(c, Layer::Ground, id, building);
    if let Some(ob) = w.obj_mut(id) {
        ob.layer = Layer::Ground;
        // Q-0713 (interim: the destination, queued destinations and path are cleared and the movement in progress is reset; the target is kept)
        ob.dest = None;
        ob.queued_dests.clear();
        ob.path.steps.clear();
        if let Some(d) = ob.drive.as_mut() {
            d.track = None;
            d.turn = None;
            d.turning_in_place = false;
        }
    }
    // spec: S-0511 claims 9 and 10 (it starts falling with the bomb mark)
    crate::fall::start(w, id, true);
    ctx.dropped.push(id);
}

/// The marks a dropped object left in the deck record move to the ground
/// record of the same cells (Q-0712), after the attempt's heading hits.
pub fn transfer_marks(w: &mut World, ctx: &Ctx) {
    let q = crate::objects::occ_quirks(w);
    for &id in &ctx.dropped {
        let Some(o) = w.obj(id).cloned() else {
            continue;
        };
        // Q-0712 (interim: the dropped object's deck marks move to the ground record once the attempt's heading hits are done)
        match o.kind {
            Kind::Infantry => {
                let mut cells = vec![o.pos.cell()];
                if let Some(t) = o.walk.as_ref().and_then(|s| s.target) {
                    cells.push(t.cell());
                }
                for c in cells {
                    let Some(rec) = w.cells.occ(c, Layer::Deck).copied() else {
                        continue;
                    };
                    for (p, m) in rec.marks.iter().enumerate() {
                        if let Some(crate::cells::Mark {
                            by: crate::cells::Marker::Infantry(who, house),
                            ..
                        }) = m
                            && *who == id
                        {
                            w.cells.clear_position(c, Layer::Deck, p as u8, q);
                            w.cells.mark_infantry(c, Layer::Ground, p as u8, id, *house);
                        }
                    }
                }
            }
            Kind::Vehicle => {
                if let Some(h) = o.hover.as_ref() {
                    let mut moved = h.claims.clone();
                    for (c, l) in moved.iter_mut() {
                        if *l == Layer::Deck {
                            w.cells.claim_vehicle(*c, Layer::Deck, false);
                            w.cells.claim_vehicle(*c, Layer::Ground, true);
                            *l = Layer::Ground;
                        }
                    }
                    if let Some(hs) = w.obj_mut(id).and_then(|ob| ob.hover.as_mut()) {
                        hs.claims = moved;
                    }
                } else if let Some(d) = o.drive.as_ref() {
                    // The drive claims carry no layer: they were made on the deck.
                    for &c in &d.claims {
                        w.cells.claim_vehicle(c, Layer::Deck, false);
                        w.cells.claim_vehicle(c, Layer::Ground, true);
                    }
                }
            }
            _ => {}
        }
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

/// The destination cell that a unit's locomotion heads to for claims 11
/// and 15: drive and ship use the end of the track they follow, else the
/// destination; walk and hover use the destination; an object destination
/// does not count; other locomotion never heads (Q-0718).
pub fn heading_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    let o = w.obj(id)?;
    let u = w.rules().unit(o.kind, o.type_index)?;
    let dest = match o.dest {
        Some(Target::Cell(c)) => Some(c),
        // Q-0718 (interim: an object destination does not count)
        _ => None,
    };
    match u.locomotor {
        // spec: S-0511 claim 11 (drive and ship: the end of the track it is following, if any, else its destination)
        Locomotor::Drive | Locomotor::Ship => o
            .drive
            .as_ref()
            .and_then(|d| d.track.as_ref())
            .map(|t| t.target_cell)
            .or(dest),
        // spec: S-0511 claim 11 (walk and hover: their destination)
        Locomotor::Walk | Locomotor::Hover => dest,
        // spec: S-0511 claim 11 (drop-pod, fly, jumpjet, rocket, teleport and tunnel locomotion do not count)
        _ => None,
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
        for n in block_others(c) {
            for layer in [Layer::Deck, Layer::Ground] {
                let ids: Vec<ObjectId> = w.cells.list(n, layer).to_vec();
                for id in ids {
                    // Q-0718 (interim: the height condition, within 104 leptons of P's height, is taken as met)
                    if heading_cell(w, id) == Some(c) {
                        ground_hit(w, id);
                    }
                }
            }
        }
    }
}
