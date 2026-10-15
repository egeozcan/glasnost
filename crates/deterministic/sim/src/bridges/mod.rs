//! Bridge spans and links (S-0010 claim 25; S-0212 Terms, claims 3 and 4).
//!
//! Once at game start the map's bridge spans are found from the tiles of
//! the bridge and wooden-bridge roles. Each span is a bridge link of the
//! path search (enabled when the span is high); each low span gives the
//! cells of its band the bridge-body mark and its bridge axis. The links,
//! marks and axes are state ([`crate::cells::Cells`]); regions and sectors
//! read them.
//!
//! In play, bridges are damaged, collapse and are repaired (S-0511): every
//! destroy attempt goes through [`attempt`] (claim 2's dispatch), whatever
//! makes it: area damage ([`area_attempts`], claim 1), the trigger actions'
//! direct attempts ([`cell_attempts`]) and hut demolition ([`hut`]).

pub mod blow;
pub mod high;
pub mod hut;
pub mod low;
pub mod pieces;

use glasnost_map::CellXY;
use glasnost_map::terrain::{Roles, land};

use crate::cells::{Axis, BridgeLink};
use crate::world::{ObjectId, Target, World};
use blow::Ctx;
use pieces::{Class, Family, Piece};

/// Tiles per bridge role (S-0010 claim 25 and table R).
// spec: S-0010 claim 25 (16 tiles each)
const ROLE_TILES: i32 = 16;

/// Whether `tile` is one of the tiles of the bridge or wooden-bridge role
/// (S-0010 claim 25). An unset role has no tiles.
fn in_roles(roles: &Roles, tile: i32) -> bool {
    // spec: S-0010 claim 25 (an unset role has no tiles here)
    [roles.bridge, roles.wooden_bridge]
        .into_iter()
        .any(|s| s >= 0 && (s..s + ROLE_TILES).contains(&tile))
}

/// Whether `c` meets the condition of a bridge end (S-0212 Terms, *bridge
/// ends*): its tile is one of the 16 tiles of the bridge or wooden-bridge
/// role of its theater and its land type is not rock.
pub fn end_condition(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let Some(cat) = w.catalogue.as_ref() else {
        return false;
    };
    // spec: S-0212 Terms (bridge ends: a tile of the 16 of the bridge or wooden-bridge role, and land that is not rock)
    in_roles(&cat.roles, cell.tile) && cell.land != land::ROCK
}

/// The two bridge ends of the bridge cell `x` (S-0212 Terms), first the one
/// towards x + 1 (along cell x) or y + 1 (along cell y), then the other;
/// `None` for a direction without one.
// nocite: the two directions along an axis, not a behavioural number
pub fn ends(w: &World, x: CellXY) -> [Option<CellXY>; 2] {
    let (dx, dy) = match w.cells.axis(x) {
        Axis::X => (1, 0),
        Axis::Y => (0, 1),
        Axis::None => return [None, None],
    };
    let walk = |sx: i32, sy: i32| {
        let mut c = x;
        loop {
            c = CellXY::new(c.x + sx, c.y + sy);
            w.cells.cell(c)?;
            if !w.cells.is_bridge(c) {
                // spec: S-0212 Terms (the first cell that is not a bridge cell, when it meets the condition)
                return end_condition(w, c).then_some(c);
            }
        }
    };
    [walk(dx, dy), walk(-dx, -dy)]
}

/// Finds the map's bridge spans (S-0010 claim 25, through
/// [`glasnost_map::bridges::find_spans`]), in the visiting order `order` of
/// their start cells, and makes them the bridge links (S-0212 claim 3);
/// low spans set the bridge-body mark and the bridge axis on their bands.
pub fn find_spans(w: &mut World, order: &[CellXY]) {
    let roles = w.catalogue_arc().roles;
    let cells = &w.cells;
    let deck = |c: CellXY| cells.is_bridge(c);
    let mut spans = glasnost_map::bridges::find_spans(&cells.grid, &roles, deck);
    // spec: S-0212 claim 3 (links in the visiting order of the start cells)
    let rank: std::collections::BTreeMap<CellXY, usize> =
        order.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    spans.sort_by_key(|s| rank.get(&s.start).copied().unwrap_or(usize::MAX));
    // spec: S-0010 claim 25 (where bands overlap, the span whose start cell comes first in the visiting order decides the axis; so the spans are applied last to first)
    for s in spans.iter().rev().filter(|s| !s.high) {
        let axis = if s.east {
            // spec: S-0010 claim 25, S-0212 Terms (each band cell gets the axis: along cell x for a span running east, along cell y for one running south)
            Axis::X
        } else {
            Axis::Y
        };
        let band = glasnost_map::bridges::body_cells(&w.cells.grid, std::slice::from_ref(s), |c| {
            w.cells.is_bridge(c)
        });
        for m in band {
            if let Some(i) = w.cells.idx(m) {
                w.cells.bridge_body[i] = true;
                w.cells.bridge_axis[i] = axis;
            }
        }
    }
    // spec: S-0212 claim 3 (one link per span; enabled when the span is high)
    w.cells.links = spans
        .iter()
        .map(|s| BridgeLink {
            first: s.start,
            second: s.end,
            along_x: s.east,
            enabled: s.high,
        })
        .collect();
}

/// The first bridge link, in creation order, that `c` lies within `k`
/// cells of (S-0212 Terms).
pub fn link_within(w: &World, c: CellXY, k: i32) -> Option<BridgeLink> {
    w.cells.links.iter().copied().find(|l| l.within(c, k))
}

/// Going from the bridge cell `x` one cell at a time east along cell x (a
/// link along cell x) or south along cell y, the first cell that is not a
/// bridge cell (S-0212 claims 4 and 13b), if any.
pub fn first_off_bridge(w: &World, x: CellXY, along_x: bool) -> Option<CellXY> {
    let (sx, sy) = if along_x { (1, 0) } else { (0, 1) };
    let mut c = x;
    loop {
        c = CellXY::new(c.x + sx, c.y + sy);
        w.cells.cell(c)?;
        if !w.cells.is_bridge(c) {
            return Some(c);
        }
    }
}

/// One destroy attempt at cell `c` (S-0511 claim 2): a low piece that is
/// not destroyed, else the section whose overlay marks a high-bridge cell,
/// else a pier in state 0 to 3 at one of its cells without a deck. Then,
/// for a success on a high bridge, the span check (claim 7) and the named
/// cell (claim 12); then the heading hits (claim 11) and the dropped
/// objects' marks (Q-0712). Returns whether it succeeded.
pub fn attempt(w: &mut World, c: CellXY) -> bool {
    let mut ctx = Ctx::default();
    let ok = attempt_in(w, c, &mut ctx);
    // spec: S-0511 claim 11 (after an attempt has made its blow-ups, and after the connectivity change)
    blow::heading_hits(w, &ctx);
    blow::transfer_marks(w, &ctx);
    ok
}

fn attempt_in(w: &mut World, c: CellXY, ctx: &mut Ctx) -> bool {
    // spec: S-0511 claim 2 (a low piece that is not destroyed: the low strip)
    if let Some(l) = pieces::low_at(w, c)
        && l.role.class() != Class::Destroyed
    {
        return low::attempt(w, c, l);
    }
    // spec: S-0511 claim 2 (a high-bridge cell: the section whose overlay marks it)
    if w.cells.is_bridge(c) {
        let Some(s) = pieces::section_marking(w, c) else {
            return false;
        };
        let ok = high::attempt_section(w, s, ctx);
        if ok {
            // spec: S-0511 claim 7 (the span check of the span that held the lost section)
            high::span_check(w, s.cell, s.line, ctx);
            // spec: S-0511 claim 12 (the lost section's section cell is named)
            high::name_cell(w, s.cell, false);
        }
        return ok;
    }
    // spec: S-0511 claim 2 (a pier in state 0 to 3 at a sub-tile even along x or at most 4 along y)
    if let Some((family, Piece::Pier(line, k))) = pieces::piece_at(w, c)
        && k <= 3
        && pieces::sub(w, c).is_some_and(|s| pieces::pier_body_sub(line, s))
        && let Some(r) = pieces::pier_reference(w, c, line)
    {
        let ok = high::attempt_pier(w, r, family, line, k, ctx);
        if ok {
            let f = line.step();
            // Q-0710 (interim: the far-side span is checked before the start-side span)
            high::span_check_from(w, pieces::at(r, f, 1), line, ctx);
            high::span_check(w, pieces::at(r, f, -1), line, ctx);
            // spec: S-0511 claim 12 (a collapsed pier's reference cell is named)
            high::name_cell(w, r, false);
        }
        return ok;
    }
    // spec: S-0511 claim 2 (none of the above: nothing happens and the attempt fails)
    false
}

/// Up to `n` attempts at `c`, stopping at the first success (S-0511 claims
/// 1 and 16). Returns whether one succeeded.
pub fn attempts(w: &mut World, c: CellXY, n: i32) -> bool {
    (0..n).any(|_| attempt(w, c))
}

/// Seam: the direct attempts of the trigger action that destroys attached
/// objects (S-0011 claim 27, code 32) and of the demolition-blast action
/// (S-0524 claim 5, code 63) at cell `c`: up to 4 attempts, stopping at the
/// first success, whatever the bridge-destruction setting (S-0511 claim 1).
/// Neither action is built yet; tests and those actions call this.
pub fn cell_attempts(w: &mut World, c: CellXY) -> bool {
    // spec: S-0511 claim 1 and Constants (trigger attempts: up to 4, stopping at the first success; no gate)
    attempts(w, c, 4)
}

/// Whether the high test of `family` accepts centre cell `c` (S-0511 claim
/// 1): `Some(true)` for a high-bridge cell (which imposes the height
/// window), `Some(false)` for a pier cell.
fn high_test(w: &World, c: CellXY, family: Family) -> Option<bool> {
    if w.cells.is_bridge(c) {
        // spec: S-0511 claim 1 (a high-bridge cell whose section overlay belongs to the family)
        return pieces::section_marking(w, c)
            .filter(|s| s.family == family)
            .map(|_| true);
    }
    // spec: S-0511 claim 1 (or the centre cell's tile is one of the family's piers in state 0 to 3)
    match pieces::piece_at(w, c) {
        Some((f, Piece::Pier(_, k))) if f == family && k <= 3 => Some(false),
        _ => None,
    }
}

/// The bridge attempts of an area-damage detonation at centre cell `c`
/// with point height `z`, damage `damage` and warhead `warhead` (S-0303
/// claim 18; S-0511 claim 1). The caller has checked the game option and
/// the warhead's wall flag.
pub fn area_attempts(
    w: &mut World,
    c: CellXY,
    z: i32,
    damage: i32,
    warhead: crate::combat::data::WarheadId,
) {
    let g = w.rules().combat.general.clone();
    let ion = g.ion_cannon_warhead == Some(warhead);
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut success = false;
    // spec: S-0511 claim 1 (test order: high bridge-role, high wooden, low wooden, low bridge-role)
    for (high, family) in [
        (true, Family::Bridge),
        (true, Family::Wooden),
        (false, Family::Wooden),
        (false, Family::Bridge),
    ] {
        let ok = if high {
            match high_test(w, c, family) {
                None => false,
                Some(false) => true,
                Some(true) => {
                    let h = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
                    // spec: S-0303 claim 18 (the height window of a high-bridge cell: above (h − 2) × 104 + 416, at or below (h + 1) × 104 + 416)
                    z > (h - 2) * 104 + 416 && z <= (h + 1) * 104 + 416
                }
            }
        } else {
            // spec: S-0511 claim 1 (the low test: one of the family's low pieces that is not destroyed)
            pieces::low_at(w, c)
                .is_some_and(|l| l.family == family && l.role.class() != Class::Destroyed)
        };
        if !ok {
            continue;
        }
        let made = if ion {
            // spec: S-0303 claim 18 (the ion-cannon warhead bypasses the chance draw; a high test makes up to four attempts, stopping at the first success; a low test makes one)
            attempts(w, c, if high { 4 } else { 1 })
        } else {
            // spec: S-0303 claim 18 (a ranged draw r over 1 to the bridge strength; an attempt exactly when r < D)
            let r = w.rng.ranged(1, g.bridge_strength, half);
            r < damage && attempt(w, c)
        };
        success |= made;
    }
    if success {
        drop_cell_targets(w, c);
    }
}

/// After a successful attempt from area damage, every object whose target
/// is the centre cell itself drops that target (S-0511 claim 1; Q-0719:
/// the attack target only).
fn drop_cell_targets(w: &mut World, c: CellXY) {
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.target == Some(Target::Cell(c)))
        .map(|o| o.id)
        .collect();
    for id in ids {
        if let Some(o) = w.obj_mut(id) {
            // Q-0719 (interim: only the attack target is cleared)
            o.target = None;
        }
    }
}
