//! The bridge repair hut (S-0511 claims 16 to 21): demolition, the
//! engineer's entry and the repair it makes, and the repair cursor's test.

use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};

use super::high;
use super::low;
use super::pieces::{self, Class, Family, Line, Low, Piece, Role, at};
use crate::combat::effects::{Effect, Sound};
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, Target, World};

/// The cell of a hut that the bridge searches start from: the cell of its
/// position, its top-left foundation cell (S-0511 Terms).
pub fn hut_cell(w: &World, hut: ObjectId) -> Option<CellXY> {
    // spec: S-0511 Terms (the hut cell: the cell containing its actual position, its top-left foundation cell, S-0406 claim 1)
    w.obj(hut).map(|o| o.pos.cell())
}

/// Whether `id` is a bridge repair hut.
pub fn is_hut(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.kind == Kind::Building
            && w.rules()
                .buildings
                .get(o.type_index as usize)
                .is_some_and(|b| b.building.bridge_repair_hut)
    })
}

/// The 5 × 5 block around `c` in claim 16's order: smallest x first, ties
/// by smallest y.
fn block(c: CellXY) -> Vec<CellXY> {
    let mut out = Vec::new();
    // spec: S-0511 Constants (hut search block 5 × 5); claim 16 (smallest x, ties by smallest y)
    for dx in -2..=2 {
        for dy in -2..=2 {
            out.push(CellXY::new(c.x + dx, c.y + dy));
        }
    }
    out
}

/// The family filter of claims 16 and 18 from the block around `c`:
/// `Some` with `quirks.bridge_family_by_block`, `None` (the family of
/// whatever bridge the search finds) without it.
fn family_filter(w: &World, c: CellXY) -> Option<Family> {
    if !w.rules().quirks.bridge_family_by_block {
        // spec: S-0511 Edge cases (quirks.bridge_family_by_block off: the family is taken from the bridge the search finds)
        return None;
    }
    // spec: S-0511 claim 16 (wooden when any block cell has a tile of the wooden-bridge role or a wooden low piece; the bridge-role family otherwise)
    let wooden = block(c).into_iter().any(|p| {
        w.cells
            .cell(p)
            .and_then(|x| pieces::tile_q(w, x.tile))
            .is_some_and(|(f, _)| f == Family::Wooden)
            || pieces::low_of_family(w, p, Family::Wooden)
    });
    Some(if wooden {
        Family::Wooden
    } else {
        Family::Bridge
    })
}

fn accepts(filter: Option<Family>, f: Family) -> bool {
    filter.is_none_or(|x| x == f)
}

/// The first low piece of the filter's family in the block around `c`
/// (claims 16 and 18).
fn first_low(w: &World, c: CellXY, filter: Option<Family>) -> Option<(CellXY, Low)> {
    block(c).into_iter().find_map(|p| {
        let l = pieces::low_at(w, p)?;
        accepts(filter, l.family).then_some((p, l))
    })
}

/// Claim 16's bridge cell from `c`: `c` itself if it is a high-bridge cell
/// or has the bridge-body mark, else the first such cell in directions N,
/// NE, E, SE, S, SW, W, NW at distances 1, 2, 3.
fn bridge_cell(w: &World, c: CellXY) -> Option<CellXY> {
    let is = |p: CellXY| w.cells.is_bridge(p) || w.cells.body(p);
    if is(c) {
        return Some(c);
    }
    // spec: S-0511 claim 16 (directions N, NE, E, SE, S, SW, W, NW in turn, distances 1, 2, 3 in each)
    for d in crate::regions::STEPS {
        for k in 1..=3 {
            let p = at(c, d, k);
            if is(p) {
                return Some(p);
            }
        }
    }
    None
}

/// The section cell and line of bridge cell `b` (claim 16).
fn section_cell(
    w: &World,
    b: CellXY,
    filter: Option<Family>,
) -> Option<(CellXY, Line, Option<Family>)> {
    let (c, line, fam) = pieces::section_of(w, b)?;
    // spec: S-0511 claim 16 (the family filter applies to a high-bridge cell's section)
    fam.is_none_or(|f| accepts(filter, f))
        .then_some((c, line, fam))
}

/// The family of a bridge tile in `c` that passes the filter.
fn tile_family(w: &World, c: CellXY, filter: Option<Family>) -> Option<Family> {
    let (f, _) = pieces::piece_at(w, c)?;
    accepts(filter, f).then_some(f)
}

// ---------------------------------------------------------------------
// Demolition (claim 16).

/// Seam and entry point: a bridge repair hut demolishes its bridge (S-0511
/// claim 16). Called when its delayed-kill countdown expires (S-0302
/// claim 16) and, when a timed bomb (S-0519 claim 5) on
/// it detonates. No gate applies.
pub fn demolish(w: &mut World, hut: ObjectId) {
    let Some(c) = hut_cell(w, hut) else { return };
    let filter = family_filter(w, c);
    if let Some((p, l)) = first_low(w, c, filter) {
        demolish_low(w, p, l);
        return;
    }
    // spec: S-0511 claim 16 (a high bridge: the bridge cell from the table; none: nothing more happens)
    let Some(b) = bridge_cell(w, c) else { return };
    let Some((sc, line, fam)) = section_cell(w, b, filter) else {
        return;
    };
    let filter = filter.or(fam);
    let f = line.step();
    // spec: S-0511 claim 16 (S: the nearest start piece on the start-side ray from the section cell within the map: a start ramp, or a pier at sub-tile 4 or 2)
    let start = high::walk_back(w, sc, line, |x| {
        tile_family(w, x, filter).is_some()
            && match pieces::piece_at(w, x) {
                Some((_, Piece::StartRamp(l))) => l == line,
                Some((_, Piece::Pier(l, _))) => {
                    l == line && pieces::sub(w, x) == Some(pieces::reference_sub(line))
                }
                _ => false,
            }
    });
    let Some(start) = start else { return };
    // spec: S-0511 claim 16 (the far endpoint: the nearest far piece on the opposite ray from S, at sub-tile 4 or 2)
    let mut k = 0;
    let end = loop {
        k += 1;
        let x = at(start, f, k);
        if w.cells.cell(x).is_none() {
            break None;
        }
        if high::is_far_piece(w, x, line, false) {
            break Some(x);
        }
    };
    // spec: S-0511 claim 16 and Constants (up to three attempts at S, then up to three at the cell before the endpoint)
    super::attempts(w, start, 3);
    if let Some(e) = end {
        // spec: S-0511 claim 16 and Constants (up to three attempts at the cell before the endpoint)
        super::attempts(w, at(e, f, -1), 3);
    }
}

/// The low form of claim 16.
fn demolish_low(w: &mut World, p: CellXY, l: Low) {
    let line = l.line;
    let family = l.family;
    let centre = pieces::strip_centre(w, p, line, family);
    let f = line.step();
    let count = |dir: i32| {
        let mut n = 0;
        while pieces::low_of_family(w, at(centre, f, dir * (n + 1)), family) {
            n += 1;
        }
        n
    };
    // spec: S-0511 claim 16 (w and e: 1 plus the counts of consecutive centre-row low pieces toward decreasing and increasing coordinate)
    let wc = 1 + count(-1);
    let ec = 1 + count(1);
    // spec: S-0511 claim 16 (the first strip displaced by −trunc((w − e) / 2), exact integers truncated toward zero)
    let first = at(centre, f, -((wc - ec) / 2));
    // spec: S-0511 claim 16 (toward decreasing coordinate when e < w, increasing otherwise)
    let dir = if ec < wc { -1 } else { 1 };
    let (a, b) = low::run(w, centre, line, family);
    let (lo, hi) = (pieces::along(line, a), pieces::along(line, b));
    // spec: S-0511 Constants (hut demolition strips: 4)
    for k in 0..4 {
        let s = at(first, f, dir * k);
        // spec: S-0511 claim 16 (bounded by the contiguous low-piece run)
        if !(lo..=hi).contains(&pieces::along(line, s)) {
            break;
        }
        // spec: S-0511 claim 16 (an already-destroyed strip: no animations or effective attempt)
        if pieces::low_at(w, s).is_none_or(|x| x.role.class() == Class::Destroyed) {
            continue;
        }
        // spec: S-0511 claim 16 (three F2 low-form animations, increasing cross coordinate, before its attempts)
        for x in pieces::strip(line, s) {
            super::blow::low_form(w, x);
        }
        // spec: S-0511 claim 16 and Constants (up to three attempts, ending at the first success)
        super::attempts(w, s, 3);
    }
}

// ---------------------------------------------------------------------
// The engineer's entry and repair (claims 17 to 20).

/// Seam: an engineer enters a bridge repair hut (S-0511 claim 17), at once,
/// in the step in which it enters. Entering buildings is S-0509's
/// (`structures::entry`), which calls this. Returns whether the entry is
/// accepted: with `quirks.repair_consumes_engineer_always` off, an entry
/// that would repair nothing is refused and nothing happens.
pub fn engineer_entered_hut(w: &mut World, engineer: ObjectId, hut: ObjectId) -> bool {
    let Some(e) = w.obj(engineer).cloned() else {
        return false;
    };
    let engineer_type = e.kind == Kind::Infantry
        && w.rules()
            .unit(e.kind, e.type_index)
            .is_some_and(|u| u.engineer);
    if !engineer_type || !is_hut(w, hut) {
        return false;
    }
    let cell = e.pos.cell();
    if !w.rules().quirks.repair_consumes_engineer_always && !would_repair(w, cell) {
        // spec: S-0511 Edge cases (quirks.repair_consumes_engineer_always off: refused when claim 18 would repair nothing)
        return false;
    }
    // spec: S-0511 claim 17 (the hut's entered-by raise first: its actions see the unrepaired bridge)
    crate::triggers::building_entered(w, hut, engineer);
    let hut_pos = w.obj(hut).map(|o| o.pos);
    // spec: S-0511 claim 17 (the local player's engineer: the bridge-repaired announcement, when a radar event can be created at its cell; which player is local is the presentation's)
    w.effects.push(Effect::Sound {
        sound: Sound::BridgeRepaired,
        of: None,
        house: e.owner,
        pos: Some(e.pos),
    });
    // spec: S-0511 claim 17 (the game-wide repair sound, if set, at the hut for every player; before repair-caused deaths)
    if w.rules().combat.general.repair_bridge_sound.is_some() {
        w.effects.push(Effect::Sound {
            sound: Sound::RepairBridge,
            of: None,
            house: None,
            pos: hut_pos,
        });
    }
    repair(w, cell);
    forget_and_scatter(w, engineer, hut);
    // spec: S-0511 claim 17 (the engineer's destroyed-by-anything raise sees the repaired bridge and the scatter orders; it leaves play without a combat death or kill credit)
    crate::triggers::destroyed_by_anything(w, engineer);
    crate::objects::remove(w, engineer);
    true
}

/// Every infantry unit forgets the hut as a target or destination;
/// infantry in the hut's cells with no destination, or with the hut as
/// destination, receive scatter orders (claim 17).
fn forget_and_scatter(w: &mut World, engineer: ObjectId, hut: ObjectId) {
    let Some(h) = w.obj(hut).cloned() else { return };
    let cells = crate::econ::placement::foundation_cells(w, h.type_index, h.pos.cell());
    let mut scatter = Vec::new();
    for c in &cells {
        for &id in w.cells.list(*c, Layer::Ground) {
            if id == engineer {
                continue;
            }
            if let Some(o) = w.obj(id)
                && o.kind == Kind::Infantry
                && (o.dest.is_none() || o.dest == Some(Target::Object(hut)))
                && !scatter.contains(&id)
            {
                scatter.push(id);
            }
        }
    }
    let ids: Vec<ObjectId> = w
        .objects
        .values()
        .filter(|o| o.kind == Kind::Infantry)
        .map(|o| o.id)
        .collect();
    for id in ids {
        if let Some(o) = w.obj_mut(id) {
            // spec: S-0511 claim 17 (every infantry unit forgets the hut as a target or destination)
            if o.target == Some(Target::Object(hut)) {
                o.target = None;
                crate::emitters::cleared(o);
            }
            if o.dest == Some(Target::Object(hut)) {
                o.dest = None;
            }
        }
    }
    for id in scatter {
        // spec: S-0511 claim 17 (scatter orders, S-0208; their draws follow the repair's)
        crate::scatter::request(
            w,
            id,
            crate::scatter::ScatterRequest {
                threat: None,
                forced: false,
                anywhere: false,
            },
        );
    }
}

/// Whether the repair from engineer cell `c` would change any cell (for
/// `quirks.repair_consumes_engineer_always` off).
fn would_repair(w: &World, c: CellXY) -> bool {
    let mut trial = w.clone();
    trial.effects.clear();
    repair(&mut trial, c);
    trial.cells.grid != w.cells.grid || trial.cells.alternate != w.cells.alternate
}

/// The repair from engineer cell `c` (claims 18 to 20).
pub fn repair(w: &mut World, c: CellXY) {
    let filter = family_filter(w, c);
    // spec: S-0511 claim 18 (a low piece of the family in the block, any class: the first in claim 16's order; the low repair runs from it)
    if let Some((p, l)) = first_low(w, c, filter) {
        repair_low(w, p, l);
        return;
    }
    // spec: S-0511 claim 18 (otherwise the bridge cell and section cell as in claim 16; none: nothing is repaired)
    let Some(b) = bridge_cell(w, c) else { return };
    let Some((sc, line, fam)) = section_cell(w, b, filter) else {
        return;
    };
    repair_from(w, sc, line, filter.or(fam));
    // spec: S-0511 claim 19 (then connectivity is updated)
    w.regions.dirty = true;
}

/// Claim 18's start piece from `c` toward the start side, `c` included,
/// and the repair it selects.
fn repair_from(w: &mut World, c: CellXY, line: Line, filter: Option<Family>) {
    let unbounded = w.rules().quirks.repair_walk_unbounded;
    let f = line.step();
    let mut x = c;
    let start = loop {
        if w.cells.cell(x).is_none() {
            return;
        }
        // spec: S-0511 claim 18 (a start ramp at sub-tile 8 or 12, or a pier in any state at sub-tile 5 or 7)
        if high::is_start_piece(w, x, line) && tile_family(w, x, filter).is_some() {
            break x;
        }
        // spec: S-0511 Edge cases (quirks.repair_walk_unbounded off: the walk stops at the first cell that is neither a high-bridge cell nor bridge-body marked)
        if !unbounded && !w.cells.is_bridge(x) && !w.cells.body(x) {
            return;
        }
        x = at(x, f, -1);
    };
    let Some((family, piece)) = pieces::piece_at(w, start) else {
        return;
    };
    let family = filter.unwrap_or(family);
    match piece {
        Piece::StartRamp(_) => {
            // spec: S-0511 claim 18 (a selected start ramp loses its alternate look; its span has claim 19's repaired state)
            for t in pieces::tile_cells(w, start) {
                w.cells.alternate.remove(&t);
            }
            repair_span(w, start, line, family);
        }
        Piece::Pier(..) => {
            let Some(r) = pieces::pier_reference(w, start, line) else {
                return;
            };
            // spec: S-0511 claim 18 (a selected pier is restored, with the bridge selected from the cell two steps toward its start side, before its own far-side span receives sections)
            restore_pier(w, r, line);
            high::name_cell(w, r, true);
            // spec: S-0511 claim 18 (the bridge selected from the cell two steps toward the start side)
            repair_from(w, at(start, f, -2), line, filter);
            repair_span(w, start, line, family);
        }
        Piece::FarRamp(_) => {}
    }
}

/// Restores the pier with reference cell `r` (claim 19).
fn restore_pier(w: &mut World, r: CellXY, line: Line) {
    // spec: S-0511 Terms (span repair uses its selected axis)
    let Some((family, k)) = pieces::pier_along(w, r, line) else {
        return;
    };
    // spec: S-0511 claim 19 (the pier's tile goes back to state 0 on all its cells, whatever state it was in)
    let cells = high::set_pier(w, r, family, line, 0);
    // spec: S-0511 claim 19 (a collapsed pier, state 4, also regains its height)
    if k == 4 {
        if w.rules().quirks.pier_repair_partial_height {
            // spec: S-0511 claim 19 and Constants (the reference cell and its two neighbours across each rise by 4 height levels)
            for c in pieces::strip(line, r) {
                if let Some(cell) = w.cells.grid.get_mut(c) {
                    cell.level = cell.level.saturating_add(4);
                }
            }
        } else {
            // spec: S-0511 Edge cases (quirks.pier_repair_partial_height off: every cell of the pier's tile goes back to its level before the collapse)
            for &c in &cells {
                if let Some(l) = w.cells.pier_levels.get(&c).copied()
                    && let Some(cell) = w.cells.grid.get_mut(c)
                {
                    cell.level = l;
                }
            }
        }
        let mut touched = cells.clone();
        touched.extend(pieces::strip(line, r));
        crate::load::recalc_cells(w, &touched);
        for c in touched {
            crate::regions::cell_changed(w, c);
        }
    }
    for c in &cells {
        w.cells.pier_levels.remove(c);
    }
}

/// The high-bridge repair of the span starting at `start` (claim 19).
fn repair_span(w: &mut World, start: CellXY, line: Line, family: Family) {
    let f = line.step();
    // spec: S-0511 claim 19 (the endpoint: the nearest far ramp at sub-tile 4 or 2 within 29 cell steps)
    let end = high::far_from(w, start, line, true);
    let reach = match end {
        Some(e) => pieces::along(line, e) - pieces::along(line, start),
        // spec: S-0511 claim 19 (no endpoint within 29 steps: the piers within that distance are still restored)
        None => high::span_limit(w).map_or(0, |l| l + 1),
    };
    let mut named = Vec::new();
    for k in 1..reach {
        let x = at(start, f, k);
        // spec: S-0511 claim 19 (every pier before the endpoint, at sub-tile 4 or 2, is restored)
        if pieces::pier_along(w, x, line).is_some()
            && pieces::sub(w, x) == Some(pieces::reference_sub(line))
        {
            restore_pier(w, x, line);
            named.push(x);
        }
    }
    if let Some(e) = end {
        // spec: S-0511 claim 19 (the endpoint ramp loses its alternate look)
        for t in pieces::tile_cells(w, e) {
            w.cells.alternate.remove(&t);
        }
        for k in 0..reach {
            place_section(w, at(start, f, k), line, family);
        }
        // spec: S-0511 claim 19 (the far ramp and the cell one step before it name cells)
        named.push(e);
        named.push(at(e, f, -1));
    }
    for c in named {
        high::name_cell(w, c, true);
    }
}

/// Places an intact section of `family` along `line` in `c` (claim 19),
/// unless `c` is a pier cell without a deck or its slope shape is above 4.
fn place_section(w: &mut World, c: CellXY, line: Line, family: Family) {
    let Some(cell) = w.cells.cell(c) else { return };
    // spec: S-0511 claim 19 (the ineligible pier cells: even sub-tiles along x, sub-tiles at most 4 along y)
    if let Some((_, Piece::Pier(l, _))) = pieces::piece_at(w, c)
        && l == line
        && pieces::pier_body_sub(line, cell.sub_tile)
    {
        return;
    }
    // spec: S-0511 claim 19 (a cell whose slope shape is greater than 4 receives no overlay)
    if cell.slope > 4 {
        return;
    }
    let o = pieces::high_overlay(family, line);
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.overlay = o;
        // spec: S-0511 claim 19 (data 0 along x, 9 along y)
        cell.overlay_data = pieces::data_of(line, pieces::State::Intact);
    }
    let Some(s) = pieces::section_in(w, c) else {
        return;
    };
    // spec: S-0511 claim 19 (each placement marks its four cells, S-0211 claim 15, and clears their bridge-body mark)
    w.cells.clear_body(&s.cells());
    w.cells.mark_high_bridge(c, o);
    crate::load::recalc_cells(w, &[c]);
    for x in s.cells() {
        crate::regions::cell_changed(w, x);
    }
}

/// F4: one accepted value r = trunc(g × 4 × c) of the map-generator
/// stream, 0 to 3 (S-0202 claim 13).
pub fn f4(w: &mut World) -> u8 {
    let mode = Rounding::TowardZero;
    loop {
        let Some(g) = w.map_rng.as_mut().map(|r| r.raw()) else {
            return 0;
        };
        // spec: S-0511 F4 and S-0202 claim 13 (r = trunc(g × 4 × c), both multiplications in double precision, round-toward-zero mode)
        let r = F64::from_i64(i64::from(g), mode)
            .mul(F64::from_i64(4, mode), mode)
            .mul(crate::consts::MAP_GENERATOR_C, mode)
            .trunc_i128()
            .unwrap_or(i128::MAX);
        // spec: S-0511 F4 (r > 3: another raw value is taken)
        if (0..=3).contains(&r) {
            return r as u8;
        }
    }
}

/// The low-bridge repair from piece `p` (claim 20).
fn repair_low(w: &mut World, p: CellXY, l: Low) {
    let line = l.line;
    let centre = pieces::strip_centre(w, p, line, l.family);
    let (a, b) = low::run(w, centre, line, l.family);
    let f = line.step();
    let n = pieces::along(line, b) - pieces::along(line, a);
    // spec: S-0511 claim 20 (the whole contiguous run, no cap, in increasing axis coordinate)
    for k in 0..=n {
        let x = at(a, f, k);
        let Some(piece) = pieces::low_at(w, x) else {
            continue;
        };
        let new = match piece.role {
            // spec: S-0511 claim 20 (damaged, destroyed, I₄ or I₅: Iᵣ with r from the map-generator stream, F4)
            Role::D(_) | Role::X | Role::I(4) | Role::I(5) => Role::I(f4(w)),
            // spec: S-0511 claim 20 (a damaged end piece becomes its whole position)
            Role::E2 => Role::E,
            Role::S2 => Role::S,
            // spec: S-0511 claim 20 (any other piece stays)
            _ => continue,
        };
        // spec: S-0511 claim 20 (all three cells of its strip get the new position, recalculated and kill-checked)
        let cells = low::write_strip(w, x, piece.family, piece.line, new);
        for c in cells {
            low::kill_check(w, c, false);
        }
    }
    // spec: S-0511 claim 20 (afterwards connectivity is updated)
    w.regions.dirty = true;
}

// ---------------------------------------------------------------------
// The repair cursor (claim 21).

/// What the repair cursor's candidate cell holds (claim 21).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Candidate {
    /// A tile of either bridge role, with its zero-based position within
    /// the role.
    High(i32),
    /// A low piece of either family.
    Low(Low),
}

/// A tile's zero-based position within a bridge role for the cursor test
/// (claim 21): when both role ranges contain the tile, the wooden-bridge
/// role supplies it.
fn cursor_position(w: &World, tile: i32) -> Option<i32> {
    let roles = w.catalogue.as_ref()?.roles;
    // spec: S-0511 claim 21 (all 16 positions of either role; the wooden-bridge role first when both contain the tile); S-0010 claim 25 (16 tiles per role)
    [roles.wooden_bridge, roles.bridge]
        .into_iter()
        .find(|&s| s >= 0 && (s..s + 16).contains(&tile))
        .map(|s| tile - s)
}

/// The candidate of claim 21 in the 5 × 5 block around hut cell `c`: the
/// last qualifying cell in increasing y, then increasing x order.
fn cursor_candidate(w: &World, c: CellXY) -> Option<(CellXY, Candidate)> {
    let mut found = None;
    // spec: S-0511 claim 21 (the hut cell's 5 × 5 block, increasing y, then increasing x; the last qualifying cell)
    for dy in -2..=2 {
        for dx in -2..=2 {
            let p = CellXY::new(c.x + dx, c.y + dy);
            let Some(cell) = w.cells.cell(p) else {
                continue;
            };
            // spec: S-0511 claim 21 (the high tile test takes precedence over the cell's low overlay)
            if let Some(q) = cursor_position(w, cell.tile) {
                found = Some((p, Candidate::High(q)));
            } else if let Some(l) = pieces::low_at(w, p) {
                // spec: S-0511 claim 21 (otherwise a low piece of either family, destroyed included)
                found = Some((p, Candidate::Low(l)));
            }
        }
    }
    found
}

/// The probe of a high candidate (claim 21's table): its displacement from
/// the placed tile's origin and the direction of travel; `None` for
/// positions 6 to 15.
// spec: S-0511 claim 21 (the probe table)
fn probe_of(position: i32) -> Option<((i32, i32), (i32, i32))> {
    match position {
        // spec: S-0511 claim 21 (positions 0 and 1: probe (1, 2), increasing x)
        0 | 1 => Some(((1, 2), (1, 0))),
        // spec: S-0511 claim 21 (position 2: probe (0, 2), decreasing x)
        2 => Some(((0, 2), (-1, 0))),
        // spec: S-0511 claim 21 (positions 3 and 4: probe (2, 1), increasing y)
        3 | 4 => Some(((2, 1), (0, 1))),
        // spec: S-0511 claim 21 (position 5: probe (2, 0), decreasing y)
        5 => Some(((2, 0), (0, -1))),
        // spec: S-0511 claim 21 (positions 6 to 15: no probe, no repair cursor)
        _ => None,
    }
}

/// Whether the engineer's cursor offers a repair over hut `hut` (S-0511
/// claim 21; S-0214 rows I3 and IC2): the last qualifying candidate of the
/// hut cell's block, then its low run or its high candidate's link
/// traversal. Separate from claim 18's repair selection.
pub fn repair_offered(w: &World, hut: ObjectId) -> bool {
    let Some(c) = hut_cell(w, hut) else {
        return false;
    };
    // spec: S-0511 claim 21 (no candidate: no repair cursor)
    let Some((p, cand)) = cursor_candidate(w, c) else {
        return false;
    };
    match cand {
        Candidate::Low(l) => {
            // spec: S-0511 claim 21 (a low candidate: its own row's contiguous run of the family's low pieces along its axis, both directions, candidate included, must hold a destroyed piece)
            let (a, b) = low::run(w, p, l.line, l.family);
            let f = l.line.step();
            let n = pieces::along(l.line, b) - pieces::along(l.line, a);
            (0..=n).any(|k| {
                pieces::low_at(w, at(a, f, k)).is_some_and(|x| x.role.class() == Class::Destroyed)
            })
        }
        Candidate::High(q) => high_cursor(w, p, q),
    }
}

/// Claim 21's link traversal for the high candidate in `p` at zero-based
/// role position `q`.
fn high_cursor(w: &World, p: CellXY, q: i32) -> bool {
    let Some(cell) = w.cells.cell(p) else {
        return false;
    };
    let Some(((ox, oy), d)) = probe_of(q) else {
        return false;
    };
    let Some((tw, _)) = w.tile_size(cell.tile) else {
        return false;
    };
    let tw = i32::from(tw.max(1));
    let s = i32::from(cell.sub_tile);
    // spec: S-0307 claim 23 (the placed tile's origin: x₀ = x − (s mod W), y₀ = y − ⌊s / W⌋)
    let origin = CellXY::new(p.x - s % tw, p.y - s / tw);
    let unrelated = w.rules().quirks.bridge_cursor_unrelated_link;
    // spec: S-0511 claim 21 (the initial traversal cell equals the initial probe)
    let mut probe = CellXY::new(origin.x + ox, origin.y + oy);
    let mut traversal = probe;
    let mut seen = std::collections::BTreeSet::new();
    loop {
        // spec: S-0511 Edge cases (a repeated traversal cell refuses repair rather than searching without end)
        // The warning is not written here: the cursor test is a read-only
        // query of the world, and notes are written only by the step.
        if !seen.insert(traversal) {
            return false;
        }
        let ends = |l: &crate::cells::BridgeLink| l.first == traversal || l.second == traversal;
        // spec: S-0511 claim 21 (the first bridge link in creation order within 3 cells of the probe, S-0212 Terms; low links included; links are not rebuilt)
        let link = if unrelated {
            super::link_within(w, probe, 3)
        } else {
            // spec: S-0511 Edge cases (quirks.bridge_cursor_unrelated_link off: links without an endpoint at the traversal cell are skipped)
            w.cells
                .links
                .iter()
                .copied()
                .find(|l| l.within(probe, 3) && ends(l))
        };
        // spec: S-0511 claim 21 (no such link: no repair cursor)
        let Some(l) = link else { return false };
        // spec: S-0511 claim 21 (a disabled link offers repair)
        if !l.enabled {
            return true;
        }
        // spec: S-0511 claim 21 and Edge cases (quirks.bridge_cursor_unrelated_link: an enabled link with neither endpoint at the traversal cell offers repair)
        if !ends(&l) {
            return true;
        }
        // spec: S-0511 claim 21 (the traversal cell becomes the opposite endpoint; the next probe is one cell from there in the table's direction)
        traversal = if l.first == traversal {
            l.second
        } else {
            l.first
        };
        probe = at(traversal, d, 1);
    }
}
