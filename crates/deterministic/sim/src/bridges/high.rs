//! High bridges (S-0511 claims 3 to 8 and 12): direct attempts on sections
//! and piers, notices, losses, the span check with its band raises, and the
//! bridge-link changes of a named cell.

use glasnost_map::{CellXY, NO_OVERLAY};

use super::blow::{self, Ctx};
use super::pieces::{self, Family, Line, Piece, Section, State, at};
use crate::cells::BridgeLink;
use crate::world::World;

/// A notice's kind (S-0511 claim 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Notice {
    /// A damage notice.
    Damage,
    /// A loss notice.
    Loss,
}

/// The side of a receiver on which a notice arrives.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    /// Its start side (the sender lies toward decreasing x or y).
    Start,
    /// Its far side.
    Far,
}

/// Sets a section's state.
fn set_state(w: &mut World, s: &Section, st: State) {
    if let Some(cell) = w.cells.grid.get_mut(s.cell) {
        cell.overlay_data = pieces::data_of(s.line, st);
    }
}

/// Loses a section (claim 5): its overlay is removed and its four cells
/// lose their high-bridge marking and gain the bridge-body mark.
fn lose(w: &mut World, s: &Section) {
    if let Some(cell) = w.cells.grid.get_mut(s.cell) {
        // spec: S-0511 claim 5 (its overlay is removed from its cell)
        cell.overlay = NO_OVERLAY;
        cell.overlay_data = 0;
    }
    let cells = s.cells();
    // spec: S-0511 claim 5 (its four cells lose their high-bridge marking and gain the bridge-body mark)
    w.cells.unmark_high_bridge(&cells);
    crate::load::recalc_cells(w, &[s.cell]);
    for c in cells {
        crate::regions::cell_changed(w, c);
    }
}

/// The blow-ups of a lost section's four cells (claim 5's order).
fn blow_section(w: &mut World, s: &Section, ctx: &mut Ctx) {
    for c in s.cells() {
        let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
        blow::blow_up(w, c, level, ctx);
    }
}

/// A direct attempt on section `s` (claim 3). Returns whether it
/// succeeded.
pub fn attempt_section(w: &mut World, s: Section, ctx: &mut Ctx) -> bool {
    let data = w.cells.cell(s.cell).map_or(0, |x| x.overlay_data);
    let st = pieces::state_of(s.line, data);
    let f = s.line.step();
    let far = at(s.cell, f, 1);
    let start = at(s.cell, f, -1);
    match st {
        State::Intact | State::ChippedStart | State::ChippedFar => {
            // spec: S-0511 claim 3 (intact or chipped: damaged; a damage notice to both neighbours; fails)
            set_state(w, &s, State::Damaged);
            notice(w, far, Notice::Damage, Side::Start, ctx);
            notice(w, start, Notice::Damage, Side::Far, ctx);
            false
        }
        State::Damaged | State::OpenStart | State::OpenFar => {
            lose(w, &s);
            // spec: S-0511 claim 3 (damaged: loss notices to the far-side neighbour, then the start-side one; open on one side: to the other side only)
            if st != State::OpenFar {
                notice(w, far, Notice::Loss, Side::Start, ctx);
            }
            if st != State::OpenStart {
                notice(w, start, Notice::Loss, Side::Far, ctx);
            }
            // spec: S-0511 claim 5 (for the attempted section, the notices and the losses they cause come before its own blow-ups)
            blow_section(w, &s, ctx);
            true
        }
    }
}

/// A notice of `kind` arriving at cell `c` on side `side` (claim 4). It
/// changes only a section, a pier or a ramp in that cell.
pub fn notice(w: &mut World, c: CellXY, kind: Notice, side: Side, ctx: &mut Ctx) {
    if let Some(s) = pieces::section_in(w, c) {
        section_notice(w, s, kind, side, ctx);
        return;
    }
    match pieces::piece_at(w, c) {
        Some((family, Piece::Pier(line, k))) => {
            // spec: S-0511 claim 6 (r13, Q-3046, A received notice uses no reference cell: it reads the state of the cell it arrives at and replaces that cell's component)
            pier_notice(w, c, family, line, k, kind, side, ctx);
        }
        Some((_, Piece::StartRamp(_) | Piece::FarRamp(_))) => {
            // spec: S-0511 claim 4 (either kind gives the ramp's whole tile its alternate look; drawing only)
            for t in pieces::tile_cells(w, c) {
                w.cells.alternate.insert(t);
            }
        }
        None => {}
    }
}

/// The cell beyond `c` on the side away from a sender that arrived on
/// `side`, and the side on which a passed-on notice arrives there.
fn onward(c: CellXY, line: Line, side: Side) -> CellXY {
    let f = line.step();
    match side {
        Side::Start => at(c, f, 1),
        Side::Far => at(c, f, -1),
    }
}

/// A notice to a section (claim 4).
fn section_notice(w: &mut World, s: Section, kind: Notice, side: Side, ctx: &mut Ctx) {
    let data = w.cells.cell(s.cell).map_or(0, |x| x.overlay_data);
    let st = pieces::state_of(s.line, data);
    let (chipped_here, chipped_other, open_here, open_other) = match side {
        Side::Start => (
            State::ChippedStart,
            State::ChippedFar,
            State::OpenStart,
            State::OpenFar,
        ),
        Side::Far => (
            State::ChippedFar,
            State::ChippedStart,
            State::OpenFar,
            State::OpenStart,
        ),
    };
    match kind {
        Notice::Damage => {
            // spec: S-0511 claim 4 (damage notice: intact → chipped on the arriving side; chipped on the other side → damaged; otherwise no change)
            if st == State::Intact {
                set_state(w, &s, chipped_here);
            } else if st == chipped_other {
                set_state(w, &s, State::Damaged);
            }
        }
        Notice::Loss => {
            if st == open_other {
                // spec: S-0511 claim 4 (open on the other side: lost at once, and the loss notice passes on beyond it)
                lose(w, &s);
                notice(w, onward(s.cell, s.line, side), Notice::Loss, side, ctx);
                // spec: S-0511 State and ordering (along a chain, the farthest collapse's effects come first)
                blow_section(w, &s, ctx);
            } else if st != open_here {
                // spec: S-0511 claim 4 (intact, chipped or damaged → open on the arriving side)
                set_state(w, &s, open_here);
            }
        }
    }
}

/// Sets a pier's state *k* on its affected tile cells (S-0511 Terms),
/// returning them as found before the change.
pub(super) fn set_pier(
    w: &mut World,
    r: CellXY,
    family: Family,
    line: Line,
    k: i32,
) -> Vec<CellXY> {
    let Some(b) = w.catalogue.as_ref().map(|c| c.bridge_positions) else {
        return Vec::new();
    };
    let m = match line {
        Line::X => b.middle[0],
        Line::Y => b.middle[1],
    };
    // spec: S-0511 Terms (pier state k: tile position m + k)
    let Some(tile) = pieces::tile_of(w, family, m + k) else {
        return Vec::new();
    };
    // spec: S-0511 Terms (the affected tile cells: the same-tile eight-neighbour component of the reference cell, before the change)
    let cells = pieces::pier_component(w, r);
    for &c in &cells {
        if let Some(cell) = w.cells.grid.get_mut(c) {
            // spec: S-0511 Terms (a state replacement replaces the tile on every affected cell)
            cell.tile = tile;
        }
    }
    crate::load::recalc_cells(w, &cells);
    for &c in &cells {
        crate::regions::cell_changed(w, c);
    }
    cells
}

/// A collapse lowers the pier's changed tile cells `cells` to cell `r`'s
/// pre-collapse level − 4 (claim 6), remembering their levels: the
/// reference cell for a direct attempt, the centre cell for a notice.
fn lower_pier(w: &mut World, r: CellXY, cells: &[CellXY]) {
    let Some(ref_level) = w.cells.cell(r).map(|x| x.level) else {
        return;
    };
    for &c in cells {
        let Some(old) = w.cells.cell(c).map(|x| x.level) else {
            continue;
        };
        w.cells.pier_levels.entry(c).or_insert(old);
        if let Some(cell) = w.cells.grid.get_mut(c) {
            // spec: S-0511 claim 6 and Constants (every cell of its tile has the reference cell's pre-collapse level minus 4)
            cell.level = ref_level.saturating_sub(4);
        }
    }
    crate::load::recalc_cells(w, cells);
    for &c in cells {
        crate::regions::cell_changed(w, c);
    }
}

/// The pier column (S-0511 Terms) in decreasing-cross-coordinate,
/// reference, increasing-cross-coordinate order.
// spec: S-0511 Terms (the pier column's three cells)
fn column(r: CellXY, line: Line) -> [CellXY; 3] {
    pieces::strip(line, r)
}

/// The pre-collapse levels of the pier column.
// spec: S-0511 Terms (the pier column's three cells)
fn column_levels(w: &World, r: CellXY, line: Line) -> [(CellXY, i32); 3] {
    column(r, line).map(|c| (c, w.cells.cell(c).map_or(0, |x| i32::from(x.level))))
}

/// A direct attempt on the pier with reference cell `r` in state `k`
/// (claim 6). Returns whether it collapsed.
pub fn attempt_pier(
    w: &mut World,
    r: CellXY,
    family: Family,
    line: Line,
    k: i32,
    ctx: &mut Ctx,
) -> bool {
    let f = line.step();
    // spec: S-0511 claim 6 (states 0 to 2 fail; state 3 collapses)
    if k < 3 {
        // spec: S-0511 claim 6 (states 0 to 2: state 3, a damage notice both ways; fails)
        let _ = set_pier(w, r, family, line, 3);
        from_pier(w, r, line, at(r, f, 1), Notice::Damage, Side::Start, ctx);
        from_pier(w, r, line, at(r, f, -1), Notice::Damage, Side::Far, ctx);
        return false;
    }
    // spec: S-0511 claim 6 (state 3: collapse; succeeds)
    let pre = column_levels(w, r, line);
    // spec: S-0511 claim 6 (a collapsed pier is in state 4)
    let cells = set_pier(w, r, family, line, 4);
    lower_pier(w, r, &cells);
    // spec: S-0511 claim 6 (the column's blow-ups at pre-collapse heights, decreasing cross coordinate, reference, increasing)
    for (c, level) in pre {
        blow::blow_up(w, c, level, ctx);
    }
    // spec: S-0511 claim 6 (these effects precede the losses of its loss notices; far-side losses precede start-side ones)
    from_pier(w, r, line, at(r, f, 1), Notice::Loss, Side::Start, ctx);
    from_pier(w, r, line, at(r, f, -1), Notice::Loss, Side::Far, ctx);
    true
}

/// A notice sent from cell `sender` of a pier to cell `c`. A cell of the
/// same pier (the same reference cell) without a section is not another
/// piece: the notice changes nothing there.
#[allow(clippy::too_many_arguments)]
fn from_pier(
    w: &mut World,
    sender: CellXY,
    line: Line,
    c: CellXY,
    kind: Notice,
    side: Side,
    ctx: &mut Ctx,
) {
    // spec: S-0511 claim 6 (a notice goes to the neighbouring piece; the pier's own cells are not one) and Terms (the pier's reference cell)
    let own = pieces::pier_reference(w, sender, line);
    if own.is_some()
        && pieces::section_in(w, c).is_none()
        && matches!(pieces::piece_at(w, c), Some((_, Piece::Pier(..))))
        && pieces::pier_reference(w, c, line) == own
    {
        return;
    }
    notice(w, c, kind, side, ctx);
}

/// A notice arriving at cell `c` of a pier, whose state there is `k`
/// (claim 6). It uses no reference cell (r13, Q-3046): the state read and
/// the component replaced are `c`'s.
#[allow(clippy::too_many_arguments)]
fn pier_notice(
    w: &mut World,
    c: CellXY,
    family: Family,
    line: Line,
    k: i32,
    kind: Notice,
    side: Side,
    ctx: &mut Ctx,
) {
    let skips = w.rules().quirks.pier_loss_notice_skips_chipped;
    // spec: S-0511 claim 6 (the notice table)
    let next = match (kind, side, k) {
        (_, _, 4) => return,
        (Notice::Damage, Side::Start, 0 | 1) => 1,
        (Notice::Damage, Side::Start, _) => 3,
        (Notice::Damage, Side::Far, 0 | 2) => 2,
        (Notice::Damage, Side::Far, _) => 3,
        // spec: S-0511 claim 6 (quirks.pier_loss_notice_skips_chipped: a pier chipped on the arriving side ignores a loss notice from it)
        (Notice::Loss, Side::Start, 1) if skips => 1,
        (Notice::Loss, Side::Far, 2) if skips => 2,
        (Notice::Loss, _, 0..=2) => 3,
        (Notice::Loss, _, _) => 4,
    };
    if next == k {
        return;
    }
    // spec: S-0511 claim 6 (states 1 to 3 are set; state 4 is a collapse; r13: on the arriving cell's component)
    if next < 4 {
        let _ = set_pier(w, c, family, line, next);
        return;
    }
    // spec: S-0511 claim 6 (r13, A received notice uses no reference cell: the collapse's centre cell is the arriving cell when its sub-tile is even along x or below 5 along y, otherwise the cell one step from it toward the start side)
    let centre = if pieces::sub(w, c).is_some_and(|s| pieces::pier_body_sub(line, s)) {
        c
    } else {
        at(c, line.step(), -1)
    };
    let pre = column_levels(w, centre, line);
    // spec: S-0511 claim 6 (collapse from a notice: the loss passes onward first, on the side away from the sender; r13: from the arriving cell)
    from_pier(w, c, line, onward(c, line, side), Notice::Loss, side, ctx);
    // spec: S-0511 claim 6 (then its blow-ups: centre, decreasing y, increasing y along x; decreasing x, centre, increasing x along y)
    let order = match line {
        Line::X => [pre[1], pre[0], pre[2]],
        Line::Y => pre,
    };
    for (c, level) in order {
        blow::blow_up(w, c, level, ctx);
    }
    // spec: S-0511 claim 6 (its tile and height change as for a direct collapse, with no span check or connectivity of its own; r13: the arriving cell's component takes the centre cell's level minus 4)
    let cells = set_pier(w, c, family, line, 4);
    lower_pier(w, centre, &cells);
}

// ---------------------------------------------------------------------
// The span check (claim 7) and the band (claim 8).

/// Whether `c` is a start piece of a span along `line` (claims 7 and 18):
/// a start ramp at sub-tile 8 or 12, or a pier in any state at sub-tile 5
/// or 7, read along `line`.
pub fn is_start_piece(w: &World, c: CellXY, line: Line) -> bool {
    let Some(s) = pieces::sub(w, c) else {
        return false;
    };
    if let Some((_, Piece::StartRamp(l))) = pieces::piece_at(w, c)
        && l == line
        && s == pieces::ramp_start_sub(line)
    {
        return true;
    }
    // spec: S-0511 Terms (repair's start-piece selection requires the axis-specific sub-tile)
    pieces::pier_along(w, c, line).is_some() && s == pieces::pier_start_sub(line)
}

/// Whether `c` is a far piece along `line` (claims 7 and 19): a far ramp or
/// a pier at sub-tile 4 or 2.
pub fn is_far_piece(w: &World, c: CellXY, line: Line, ramps_only: bool) -> bool {
    let Some(s) = pieces::sub(w, c) else {
        return false;
    };
    let at_ref = s == pieces::reference_sub(line);
    if let Some((_, Piece::FarRamp(l))) = pieces::piece_at(w, c) {
        return l == line && at_ref;
    }
    // spec: S-0511 Terms (span repair uses its selected axis)
    !ramps_only && at_ref && pieces::pier_along(w, c, line).is_some()
}

/// The nearest cell from `from` toward the start side, `from` included,
/// that satisfies `is`, within the map.
pub fn walk_back(
    w: &World,
    from: CellXY,
    line: Line,
    is: impl Fn(CellXY) -> bool,
) -> Option<CellXY> {
    let f = line.step();
    let mut c = from;
    loop {
        w.cells.cell(c)?;
        if is(c) {
            return Some(c);
        }
        c = at(c, f, -1);
    }
}

/// The span limit of claims 7 and 19 (`quirks.bridge_span_cap`).
pub fn span_limit(w: &World) -> Option<i32> {
    // spec: S-0511 Constants (span limit 29 cell steps; quirks.bridge_span_cap off: no limit)
    w.rules().quirks.bridge_span_cap.then_some(29)
}

/// The nearest far piece after `start` along `line` within the span limit.
pub fn far_from(w: &World, start: CellXY, line: Line, ramps_only: bool) -> Option<CellXY> {
    far_where(w, start, line, |c| is_far_piece(w, c, line, ramps_only))
}

/// The nearest cell after `start` along `line` within the span limit that
/// satisfies `is`.
fn far_where(w: &World, start: CellXY, line: Line, is: impl Fn(CellXY) -> bool) -> Option<CellXY> {
    let f = line.step();
    let limit = span_limit(w);
    let mut k = 0;
    loop {
        k += 1;
        if limit.is_some_and(|l| k > l) {
            return None;
        }
        let c = at(start, f, k);
        w.cells.cell(c)?;
        if is(c) {
            return Some(c);
        }
    }
}

/// Raises claim 8's band from `first` to `last` along `line`.
pub fn band(w: &mut World, first: CellXY, last: CellXY, line: Line) {
    let f = line.step();
    let a = line.across();
    let n = pieces::along(line, last) - pieces::along(line, first);
    // spec: S-0511 claim 8 (quirks.bridge_event_band_short: the last cell along the band is never processed)
    let end = if w.rules().quirks.bridge_event_band_short {
        n - 1
    } else {
        n
    };
    for k in 0..=end {
        let c = at(first, f, k);
        // spec: S-0511 claim 8 (the cell, one across toward increasing, then one and two across toward decreasing)
        for j in [0, 1, -1, -2] {
            crate::triggers::bridge_destroyed(w, at(c, a, j));
        }
    }
}

/// Whether a pier piece at `c` may end a claim 7 span: a non-collapsed pier,
/// or any pier with `quirks.collapsed_pier_span_search` off.
fn span_end_pier_ok(w: &World, c: CellXY, line: Line) -> bool {
    // spec: S-0511 claim 7 and Edge cases (quirks.collapsed_pier_span_search: a collapsed pier, state 4, never qualifies as a start or far piece; off: it stays eligible at the same sub-tiles as a live pier)
    !w.rules().quirks.collapsed_pier_span_search
        || pieces::pier_along(w, c, line).is_none_or(|(_, k)| k < 4)
}

/// The span check of claim 7 for named loss cell `named`, searched on the
/// tiles and marks as they stand after the change and its notices.
pub fn span_check(w: &mut World, named: CellXY, ctx: &mut Ctx) {
    // spec: S-0511 claim 7 (the first neighbour of the named loss cell in N, NE, E, SE, S, SW, W, NW order carrying a deck or bridge-body mark supplies the bridge)
    let Some(b) = crate::regions::STEPS
        .iter()
        .map(|&d| at(named, d, 1))
        .find(|&p| w.cells.is_bridge(p) || w.cells.body(p))
    else {
        return;
    };
    // spec: S-0511 claim 7 (its responsible section, or its body-mark section by claim 16's body rule; none: no span check)
    let Some((sc, line, _)) = pieces::section_of(w, b) else {
        return;
    };
    // spec: S-0511 claim 7 (from that section cell toward the start side up to the legal-map boundary, passing over non-bridge cells: the nearest start ramp, or non-collapsed pier, at its start sub-tile; no exception for a direct collapse's far-side named cell; none: no span check)
    let start = walk_back(w, sc, line, |c| {
        is_start_piece(w, c, line) && span_end_pier_ok(w, c, line)
    });
    let Some(start) = start else {
        return;
    };
    span_check_from(w, start, line, ctx);
}

/// The span check of claim 7 for the span starting at `start`.
pub fn span_check_from(w: &mut World, start: CellXY, line: Line, ctx: &mut Ctx) {
    // spec: S-0511 claim 7 (the nearest far ramp, or non-collapsed pier, within 29 cells; none: the check does nothing)
    let Some(far) = far_where(w, start, line, |c| {
        is_far_piece(w, c, line, false) && span_end_pier_ok(w, c, line)
    }) else {
        return;
    };
    let f = line.step();
    let len = pieces::along(line, far) - pieces::along(line, start);
    let cells: Vec<CellXY> = (0..len).map(|k| at(start, f, k)).collect();
    let has: Vec<bool> = cells
        .iter()
        .map(|&c| pieces::section_in(w, c).is_some())
        .collect();
    // spec: S-0511 claim 7 (a span with no missing section raises nothing)
    if has.iter().all(|&h| h) {
        return;
    }
    // Isolated runs: runs of section cells touching neither the first nor the last cell.
    let mut runs: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < has.len() {
        if has[i] {
            let j0 = i;
            while i < has.len() && has[i] {
                i += 1;
            }
            // spec: S-0511 claim 7 (runs that include the first or the last cell stay)
            if j0 != 0 && i != has.len() {
                runs.push((j0, i));
            }
        } else {
            i += 1;
        }
    }
    let first = at(start, f, 1);
    // spec: S-0511 claim 7 (the band raises come before the first isolated loss, between losses and after the last)
    band(w, first, far, line);
    // spec: S-0511 claim 7 (the run nearest the start side first; within a run, from its far-side end toward its start side)
    for (a, b) in runs {
        for k in (a..b).rev() {
            if let Some(s) = pieces::section_in(w, cells[k]) {
                // spec: S-0511 claim 7 (sections lost here send no notices and name no cell)
                lose(w, &s);
                blow_section(w, &s, ctx);
                band(w, first, far, line);
            }
        }
    }
}

// ---------------------------------------------------------------------
// Connectivity (claim 12; S-0212 claim 3).

/// The bridge links' change for named cell `c` (S-0212 claim 3): the links
/// within 3 cells stop being enabled (`enable` false, a destruction) or
/// become enabled (a repair); with none, the spans are found again.
pub fn name_cell(w: &mut World, c: CellXY, enable: bool) {
    // spec: S-0212 Terms and claim 3 (links within 3 cells of the named cell)
    let within = |l: &BridgeLink| l.within(c, 3);
    if w.cells.links.iter().any(within) {
        // spec: S-0212 Terms and claim 3 (links within 3 cells of the named cell)
        for l in w.cells.links.iter_mut().filter(|l| l.within(c, 3)) {
            l.enabled = enable;
        }
    } else {
        // spec: S-0212 claim 3 (none: the links are replaced by the spans found on the map as it stands, without the low-span bands, S-0010 claim 25)
        let roles = w.catalogue_arc().roles;
        let order = w
            .cells
            .footprint()
            .visiting_order(w.rules().quirks.narrow_map_partial_pass);
        let cells = &w.cells;
        let mut spans =
            glasnost_map::bridges::find_spans(&cells.grid, &roles, |x| cells.is_bridge(x));
        super::sort_spans(&mut spans, &order);
        let mut links: Vec<BridgeLink> = spans
            .iter()
            .map(|s| BridgeLink {
                first: s.start,
                second: s.end,
                along_x: s.east,
                enabled: s.high,
            })
            .collect();
        // spec: S-0212 claim 3 (when no new link lies within 3 cells, no link changes)
        if links.iter().any(within) {
            // spec: S-0212 Terms and claim 3 (links within 3 cells of the named cell)
            for l in links.iter_mut().filter(|l| l.within(c, 3)) {
                l.enabled = enable;
            }
            w.cells.links = links;
        }
    }
    // spec: S-0212 claim 5 (connectivity brought up to date after a bridge destroyed or repaired, whether or not a link changed)
    w.regions.dirty = true;
}
