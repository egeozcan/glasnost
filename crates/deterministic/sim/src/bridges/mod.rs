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
    sort_spans(&mut spans, order);
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

/// Puts spans in link creation order (S-0212 claims 3 and 13b): the
/// visiting order `order` of their start cells.
pub(crate) fn sort_spans(spans: &mut [glasnost_map::bridges::Span], order: &[CellXY]) {
    // spec: S-0212 claim 3 (links in the visiting order of the start cells)
    let rank: std::collections::BTreeMap<CellXY, usize> =
        order.iter().enumerate().map(|(i, &c)| (c, i)).collect();
    spans.sort_by_key(|s| rank.get(&s.start).copied().unwrap_or(usize::MAX));
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
/// cell (claim 12); then the heading hits (claim 11). Returns whether it
/// succeeded.
pub fn attempt(w: &mut World, c: CellXY) -> bool {
    let mut ctx = Ctx::default();
    let ok = attempt_in(w, c, &mut ctx);
    // spec: S-0511 claim 11 (after an attempt has made its blow-ups, and after the connectivity change)
    blow::heading_hits(w, &ctx);
    ok
}

fn attempt_in(w: &mut World, c: CellXY, ctx: &mut Ctx) -> bool {
    // spec: S-0511 claim 2 (a low piece that is not destroyed: the low strip)
    if let Some(l) = pieces::low_at(w, c)
        && l.role.class() != Class::Destroyed
    {
        return low::attempt(w, c, l);
    }
    // spec: S-0511 claim 2 (a high-bridge cell and a family qualifies: its responsible section)
    if w.cells.is_bridge(c) {
        // spec: S-0511 claim 2 (a retained state outside the table, or no qualifying family on a deck cell: no change, not a no-deck pier attempt)
        let Some(s) = qualifying_section(w, c) else {
            return false;
        };
        let ok = high::attempt_section(w, s, ctx);
        if ok {
            // spec: S-0511 claim 7 (the span check from the named loss cell, the lost section's cell)
            high::span_check(w, s.cell, ctx);
            // spec: S-0511 claim 12 (the lost section's section cell is named)
            high::name_cell(w, s.cell, false);
        }
        return ok;
    }
    // spec: S-0511 claim 2 (a pier in state 0 to 3 at a sub-tile even along x or at most 4 along y)
    if let Some((family, Piece::Pier(line, k))) = pieces::piece_at(w, c)
        && k <= 3
        && pieces::sub(w, c).is_some_and(|s| pieces::pier_body_sub(line, s))
    {
        let Some(r) = pieces::pier_reference_of(w, c, line) else {
            // spec: S-0511 claim 6 (r13, Q-3046, A reference cell that does not hold the pier: the attempt changes nothing, fails and sends no notice, and our engine warns)
            w.note("S-0511 claim 2: a pier cell whose reference cell does not hold its tile; the attempt does nothing (Q-3046)");
            return false;
        };
        let ok = high::attempt_pier(w, r, family, line, k, ctx);
        if ok {
            let f = line.step();
            // spec: S-0511 claims 6 and 7 (after the tile replacement and both notices, searches from the reference cell's start-side neighbour, then its far-side neighbour; the first check completes before the second, and both can select the same combined extent)
            high::span_check(w, pieces::at(r, f, -1), ctx);
            high::span_check(w, pieces::at(r, f, 1), ctx);
            // spec: S-0511 claim 12 (a collapsed pier's reference cell is named)
            high::name_cell(w, r, false);
        }
        return ok;
    }
    // spec: S-0511 claim 2 (none of the above: nothing happens and the attempt fails)
    false
}

/// The section that an attempt at high-bridge cell `c` acts on (S-0511
/// claim 2), when a family qualifies: the bridge-role family first, then
/// the wooden one, each by its responsible overlay or by a non-collapsed
/// pier tile of the family at `c`. With a pier supplying the family over a
/// replaced responsible overlay, the section is the responsible cell's
/// retained state, and a state outside the section-state table gives no
/// section.
fn qualifying_section(w: &mut World, c: CellXY) -> Option<pieces::Section> {
    let Some(r) = w.cells.responsible(c) else {
        // spec: S-0511 claim 2 and Edge cases (a missing responsible section is malformed input: nothing happens, with a warning)
        w.note("S-0511 claim 2: a high-bridge cell has no responsible section; the attempt does nothing (a departure)");
        return None;
    };
    let own = pieces::section_in(w, r);
    let pier = match pieces::piece_at(w, c) {
        // spec: S-0511 claim 2 (a non-collapsed pier tile: states 0 to 3)
        Some((f, Piece::Pier(line, k))) if k <= 3 => Some((f, line)),
        _ => None,
    };
    // spec: S-0511 claim 2 (family selection: the bridge-role family's responsible overlay or matching non-collapsed pier tile at the attempted cell, then the wooden family's)
    for family in [Family::Bridge, Family::Wooden] {
        if let Some(s) = own.filter(|s| s.family == family) {
            return Some(s);
        }
        if let Some((_, line)) = pier.filter(|p| p.0 == family) {
            // spec: S-0511 claim 2 (a pier supplying the family over a replaced responsible overlay: claim 3 on the responsible cell's retained section state; a state outside the table gives no change)
            let data = w.cells.cell(r).map_or(0, |x| x.overlay_data);
            let line = own.map_or(line, |s| s.line);
            pieces::state_in_table(line, data)?;
            return Some(pieces::Section {
                cell: r,
                family,
                line,
                d: pieces::marking_direction(line),
            });
        }
    }
    // spec: S-0511 claim 2 (without a recognised responsible overlay or such a pier tile, the attempt fails)
    None
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
    let deck = w.cells.is_bridge(c);
    // spec: S-0511 claim 1 (a high-bridge cell whose section overlay belongs to the family); S-0511 claim 2 (its responsible section)
    let section = deck && pieces::responsible_section(w, c).is_some_and(|s| s.family == family);
    // spec: S-0511 claim 1 (or the centre cell's tile is one of the family's piers in state 0 to 3)
    let pier =
        matches!(pieces::piece_at(w, c), Some((f, Piece::Pier(_, k))) if f == family && k <= 3);
    // spec: S-0511 claim 1 (the height window applies only when the centre cell is a high-bridge cell)
    (section || pier).then_some(deck)
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
    let g = &w.rules().combat.general;
    let (ion, bridge_strength) = (g.ion_cannon_warhead == Some(warhead), g.bridge_strength);
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
            let r = w.rng.ranged(1, bridge_strength, half);
            r < damage && attempt(w, c)
        };
        success |= made;
    }
    if success {
        drop_cell_targets(w, c);
    }
}

/// After a successful attempt from area damage, references to the centre
/// cell itself are lost (S-0511 claims 1 and 22): a mobile object whose
/// attack target is that cell first reverts a suspended behaviour (S-0215
/// claim 10), a still-equal attack target is cleared, an aircraft in
/// *attack* restarts its run, and projectiles aimed at the cell lose it.
fn drop_cell_targets(w: &mut World, c: CellXY) {
    let cell = Some(Target::Cell(c));
    for p in w.projectiles.values_mut() {
        if p.target == cell {
            // spec: S-0511 claim 22 (a projectile's target equal to the centre cell is cleared; its recorded firer is an object in our engine, never a cell)
            p.target = None;
        }
    }
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.target == cell)
        .map(|o| o.id)
        .collect();
    for id in ids {
        let aircraft = crate::air::is_aircraft(w, id);
        let Some(ob) = w.obj_mut(id) else { continue };
        let mut restored = None;
        // spec: S-0511 claim 22 (a mobile object first reverts a suspended behaviour, restoring the target and destination its override put aside; S-0215 claim 10)
        if ob.kind.is_mobile()
            && let Some((t, d)) = ob.beh.revert()
        {
            crate::spawn::set_own_target(ob, t);
            restored = Some(ob.dest);
            ob.dest = d;
            // spec: S-0301 claim 5, Moment of a retarget's let-go (revision 67, Q-2246: at the change of the firer's target)
            crate::emitters::retarget_let_go(w, id);
            crate::combat::held::note_revert(w, id);
        }
        let Some(ob) = w.obj_mut(id) else { continue };
        let lost = ob.target == cell;
        if lost {
            // spec: S-0511 claim 22 (an attack target still equal to the centre cell is cleared; no new behaviour, no attack-move memory)
            crate::spawn::clear_own_target(ob);
            ob.combat.burst = 0; // spec: S-0301 claim 4 (a cleared target resets the burst counter)
        }
        if lost && aircraft && ob.beh.current == Some(crate::rules::Behaviour::Attack) {
            // spec: S-0511 claim 22 (an aircraft in attack restarts the attack-run substate when its target is lost, keeping the behaviour)
            crate::air::restart_run(w, id);
        }
        // spec: S-0107 r46 claim 11, Changes of target (the loss of a bridge cell target, or the earlier target put back when the override ends, S-0215 claim 10, ends an infantry unit's attack state and makes its request at once)
        crate::infantry::watch_test(w, id);
        if let Some(before) = restored {
            // spec: S-0107 claim 15 (a destination brought back is assigned: the destination-driven get-up)
            crate::infantry::destination_assigned(w, id, before);
        }
    }
}
