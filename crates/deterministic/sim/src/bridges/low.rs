//! Low bridges (S-0511 claims 13 to 15): direct attempts on strips,
//! neighbour re-evaluation, destruction with its band raises, and the kill
//! check of changed cells.

use glasnost_map::CellXY;

use super::blow;
use super::pieces::{self, Class, Family, Line, Low, Role, at};
use crate::rules::Kind;
use crate::world::{Layer, ObjectId, World};

/// Writes `role` of `family` and `line` on the strip across `line` centred
/// on `centre` (claims 13 and 14) and recalculates the changed cells. The
/// cells are written whatever they held before. Returns the cells.
pub fn write_strip(
    w: &mut World,
    centre: CellXY,
    family: Family,
    line: Line,
    role: Role,
) -> Vec<CellXY> {
    let o = pieces::low_overlay(family, line, role);
    let mut cells = Vec::new();
    for (place, c) in pieces::strip(line, centre).into_iter().enumerate() {
        if let Some(cell) = w.cells.grid.get_mut(c) {
            // spec: S-0511 claim 13 (the three cells are written whatever they held before); S-0202 claim 10 (data: the place across)
            cell.overlay = o;
            cell.overlay_data = place as u8;
            cells.push(c);
        }
    }
    // spec: S-0511 claim 13 (every changed cell is recalculated: its land type from the new overlay when it overrides the tile's)
    crate::load::recalc_cells(w, &cells);
    for &c in &cells {
        crate::regions::cell_changed(w, c);
    }
    cells
}

/// Whether unit `id` could not enter `c` as it now is (claim 15): its own
/// kind's entry outcome of S-0209 claims 4 and 5 is *blocked*.
fn blocked(w: &World, id: ObjectId, c: CellXY) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0511 claim 15 (no reference height level: a low strip carries no high bridge, so its ground layer and record count, S-0209 Terms)
    let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    // spec: S-0511 claim 15 (the object's own-kind entry outcome of S-0209 claims 4 and 5, occupants and overlays included, the locomotor's cell admission included; only blocked, outcome 7, counts)
    let outcome = match o.kind {
        Kind::Infantry => crate::occupancy::infantry_outcome(w, id, c, level),
        _ => crate::occupancy::vehicle_outcome(w, id, c, level),
    };
    outcome == crate::occupancy::Outcome::Blocked
}

/// The kill check of claim 15 for one changed cell; `destroyed` when it
/// became destroyed.
pub fn kill_check(w: &mut World, c: CellXY, destroyed: bool) {
    let ids: Vec<ObjectId> = w.cells.list(c, Layer::Ground).to_vec();
    for id in ids {
        let Some(o) = w.obj(id) else { continue };
        let kind = o.kind;
        let jumpjet = w
            .rules()
            .unit(kind, o.type_index)
            .is_some_and(|u| u.jumpjet_locomotor);
        let unit = matches!(kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft);
        // spec: S-0511 claim 15 (it could not enter the cell as it now is; the cell became destroyed and the type has jumpjet locomotion; or it is an aircraft)
        let hit = kind == Kind::Aircraft || (unit && blocked(w, id, c)) || (destroyed && jumpjet);
        if hit {
            blow::ground_hit(w, id);
        }
    }
    // spec: S-0511 claim 15 (then every unit in the ground layer of the other 24 cells of the block heading to the cell and unable to enter it)
    // spec: S-0511 claim 15 (heading to the cell at its floor height, claim 11's test)
    let z = blow::floor(w, c);
    for n in blow::block_others(c) {
        let ids: Vec<ObjectId> = w.cells.list(n, Layer::Ground).to_vec();
        for id in ids {
            let unit = w
                .obj(id)
                .is_some_and(|o| matches!(o.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft));
            if unit && blow::heads_to(w, id, c, z) && blocked(w, id, c) {
                blow::ground_hit(w, id);
            }
        }
    }
}

/// The run of low pieces of `family` along `line` through `c` on its row:
/// the first and last cells (claims 8, 16 and 20). Destroyed pieces count.
pub fn run(w: &World, c: CellXY, line: Line, family: Family) -> (CellXY, CellXY) {
    let f = line.step();
    let mut a = c;
    while pieces::low_of_family(w, at(a, f, -1), family) {
        a = at(a, f, -1);
    }
    let mut b = c;
    while pieces::low_of_family(w, at(b, f, 1), family) {
        b = at(b, f, 1);
    }
    (a, b)
}

/// A direct attempt on the low strip holding piece `p` in `c` (claims 13
/// and 15). Returns whether it destroyed the strip.
pub fn attempt(w: &mut World, c: CellXY, p: Low) -> bool {
    let line = p.line;
    let centre = pieces::strip_centre(w, c, line, p.family);
    let Some(cp) = pieces::low_at(w, centre) else {
        return false;
    };
    let family = cp.family;
    let f = line.step();
    let (dec, inc) = (at(centre, f, -1), at(centre, f, 1));
    match cp.role.class() {
        Class::Intact => {
            // spec: S-0511 claim 13 (intact: all three cells D₀; both neighbours re-evaluated; fails)
            let cells = write_strip(w, centre, family, line, Role::D(0));
            reevaluate(w, dec, line);
            reevaluate(w, inc, line);
            for c in cells {
                kill_check(w, c, false);
            }
            false
        }
        Class::Damaged => {
            // spec: S-0511 claim 13 (damaged: destroyed; both neighbours, then claim 15; succeeds)
            let cells = write_strip(w, centre, family, line, Role::X);
            // spec: S-0511 claim 15 (the neighbours' deaths, the decreasing side first, precede the trigger actions)
            reevaluate(w, dec, line);
            reevaluate(w, inc, line);
            // spec: S-0511 claim 8 (one raise per destroyed low strip; the band's ends are the first and last low pieces of the run along the centre row)
            let (first, last) = run(w, centre, line, family);
            super::high::band(w, first, last, line);
            // spec: S-0511 claim 15 (this strip's own kill checks follow the trigger actions)
            for c in cells {
                kill_check(w, c, true);
            }
            // spec: S-0511 claim 12 (a low-bridge change names no cell; connectivity is brought up to date)
            w.regions.dirty = true;
            true
        }
        Class::FarWhole => {
            // spec: S-0511 claim 13 (a far-side end piece: its damaged position; re-evaluates decreasing x along x, increasing y along y)
            let cells = write_strip(w, centre, family, line, Role::E2);
            let n = match line {
                Line::X => dec,
                Line::Y => inc,
            };
            reevaluate(w, n, line);
            for c in cells {
                kill_check(w, c, false);
            }
            false
        }
        Class::StartWhole => {
            // spec: S-0511 claim 13 (a start-side end piece: its damaged position; re-evaluates increasing x along x, decreasing y along y)
            let cells = write_strip(w, centre, family, line, Role::S2);
            let n = match line {
                Line::X => inc,
                Line::Y => dec,
            };
            reevaluate(w, n, line);
            for c in cells {
                kill_check(w, c, false);
            }
            false
        }
        // spec: S-0511 claim 13 (any other class: no change; fails)
        _ => false,
    }
}

/// The role in `c` when it is a piece of `family` bound to `line`.
fn role_in(w: &World, c: CellXY, family: Family, line: Line) -> Option<Role> {
    pieces::low_at(w, c)
        .filter(|l| l.family == family && l.line == line)
        .map(|l| l.role)
}

/// Re-evaluates the strip whose centre is `n`, one step along `line` from
/// a changed strip (claim 14).
pub fn reevaluate(w: &mut World, n: CellXY, line: Line) {
    // spec: S-0511 claim 14 (re-evaluated only when its centre cell holds a low piece)
    let Some(np) = pieces::low_at(w, n) else {
        return;
    };
    // spec: S-0511 Edge cases (quirks.low_bridge_wrong_axis_recheck off: a strip of the other axis is not re-evaluated)
    if np.line != line && !w.rules().quirks.low_bridge_wrong_axis_recheck {
        return;
    }
    let family = np.family;
    let f = line.step();
    // spec: S-0511 claim 14 (along x, a from increasing x and b from decreasing x; along y, a from decreasing y and b from increasing y)
    let (pa, pb) = match line {
        Line::X => (at(n, f, 1), at(n, f, -1)),
        Line::Y => (at(n, f, -1), at(n, f, 1)),
    };
    // spec: S-0511 claim 14 (contributions: a = 1 for I₄, D₀, D₂, E′; a = 2 for D₁, X; b = 4 for I₅, D₀, D₁, S′; b = 8 for D₂, X)
    let a = match role_in(w, pa, family, line) {
        Some(Role::I(4) | Role::D(0) | Role::D(2) | Role::E2) => 1,
        Some(Role::D(1) | Role::X) => 2,
        _ => 0,
    };
    // spec: S-0511 claim 14 (b = 4 for I₅, D₀, D₁, S′; b = 8 for D₂, X)
    let b = match role_in(w, pb, family, line) {
        Some(Role::I(5) | Role::D(0) | Role::D(1) | Role::S2) => 4,
        Some(Role::D(2) | Role::X) => 8,
        _ => 0,
    };
    let s = a + b;
    // spec: S-0511 claim 14 (s = 0: nothing changes)
    if s == 0 {
        return;
    }
    // spec: S-0511 claim 14 (new piece: 1 I₅, 2 D₂, 4 I₄, 5 D₀, 6 D₂, 8 D₁, 9 D₁, 10 X)
    let new = match s {
        1 => Role::I(5),
        2 | 6 => Role::D(2),
        4 => Role::I(4),
        5 => Role::D(0),
        8 | 9 => Role::D(1),
        _ => Role::X,
    };
    match np.role.class() {
        Class::Intact | Class::Damaged => {
            // spec: S-0511 Edge cases (quirks.low_strip_regress off: a damaged strip never moves to the intact class)
            if np.role.class() == Class::Damaged
                && new.class() == Class::Intact
                && !w.rules().quirks.low_strip_regress
            {
                return;
            }
            let o = pieces::low_overlay(family, line, new);
            // spec: S-0511 claim 14 (the same position: nothing more happens)
            if w.cells.cell(n).is_some_and(|x| x.overlay == o) {
                return;
            }
            let cells = write_strip(w, n, family, line, new);
            for c in cells {
                kill_check(w, c, new == Role::X);
            }
        }
        Class::FarWhole | Class::StartWhole => {
            // spec: S-0511 claim 14 (a whole end piece becomes its damaged position, wherever s points)
            let role = if np.role.class() == Class::FarWhole {
                Role::E2
            } else {
                Role::S2
            };
            let cells = write_strip(w, n, family, line, role);
            for c in cells {
                kill_check(w, c, false);
            }
        }
        // spec: S-0511 claim 14 (any other class stays)
        _ => {}
    }
}
