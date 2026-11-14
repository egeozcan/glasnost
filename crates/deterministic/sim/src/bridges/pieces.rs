//! Bridge pieces (S-0511 Terms and Import mapping): families, tile
//! positions, ramps and piers, high-bridge sections and their states, and
//! the low pieces with their symbolic roles.

use glasnost_map::{CellXY, NO_OVERLAY};

use crate::world::World;

/// A bridge family (S-0511 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Family {
    /// The bridge-role family.
    Bridge,
    /// The wooden-bridge-role family.
    Wooden,
}

/// A bridge's line: along cell x or along cell y (S-0511 Terms, *axis*).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Line {
    /// Along cell x (east).
    X,
    /// Along cell y (south).
    Y,
}

impl Line {
    /// One step toward the far side (increasing x or y).
    pub fn step(self) -> (i32, i32) {
        match self {
            Line::X => (1, 0),
            Line::Y => (0, 1),
        }
    }

    /// One step across the line toward increasing y (along x) or x (along
    /// y).
    pub fn across(self) -> (i32, i32) {
        match self {
            Line::X => (0, 1),
            Line::Y => (1, 0),
        }
    }

    /// The other line.
    pub fn other(self) -> Line {
        match self {
            Line::X => Line::Y,
            Line::Y => Line::X,
        }
    }
}

/// `c` moved by `k` steps of `d`.
pub fn at(c: CellXY, d: (i32, i32), k: i32) -> CellXY {
    CellXY::new(c.x + k * d.0, c.y + k * d.1)
}

/// The coordinate of `c` along `line`.
pub fn along(line: Line, c: CellXY) -> i32 {
    match line {
        Line::X => c.x,
        Line::Y => c.y,
    }
}

/// The three cells of the strip across `line` centred on `c`, in increasing
/// cross coordinate (S-0511 Terms, *low strip*).
// spec: S-0511 Terms (a low strip has three cells)
pub fn strip(line: Line, c: CellXY) -> [CellXY; 3] {
    let d = line.across();
    // spec: S-0511 Terms (low strip: (x, y − 1), (x, y), (x, y + 1) along x; (x − 1, y), (x, y), (x + 1, y) along y)
    [at(c, d, -1), c, at(c, d, 1)]
}

/// Tiles per bridge role (S-0010 claim 25).
const ROLE_TILES: i32 = 16; // spec: S-0010 claim 25 (16 tiles each)

/// A tile's family and its position *q* (S-0511 Terms, F1): the bridge
/// role is read first (S-0010 claim 25); an unset role has no tiles.
pub fn tile_q(w: &World, tile: i32) -> Option<(Family, i32)> {
    let roles = w.catalogue.as_ref()?.roles;
    [
        (Family::Bridge, roles.bridge),
        (Family::Wooden, roles.wooden_bridge),
    ]
    .into_iter()
    .find(|&(_, s)| s >= 0 && (s..s + ROLE_TILES).contains(&tile))
    // spec: S-0511 F1 (q = tile number − role start + 1)
    .map(|(f, s)| (f, tile - s + 1))
}

/// The tile number of position `q` of `family`'s role.
pub fn tile_of(w: &World, family: Family, q: i32) -> Option<i32> {
    let roles = w.catalogue.as_ref()?.roles;
    let s = match family {
        Family::Bridge => roles.bridge,
        Family::Wooden => roles.wooden_bridge,
    };
    // spec: S-0511 F1 (tile number = role start + q − 1)
    (s >= 0).then_some(s + q - 1)
}

/// What a bridge tile is (S-0511 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    /// A start ramp of a bridge along `Line`.
    StartRamp(Line),
    /// A far ramp.
    FarRamp(Line),
    /// A pier in state *k* (0 to 4).
    Pier(Line, i32),
}

/// The piece of tile position `q` (S-0511 Terms; Import mapping). A
/// position in both non-collapsed pier ranges is a pier along x; an x
/// match only at collapsed state 4 does not override a non-collapsed y
/// match.
pub fn piece_of_q(w: &World, q: i32) -> Option<Piece> {
    let b = w.catalogue.as_ref()?.bridge_positions;
    // spec: S-0511 Terms (a position the theater leaves unset matches no tile)
    let is = |set: [i32; 2]| {
        set.iter()
            .any(|&p| p != glasnost_map::terrain::UNSET_POSITION && p == q)
    };
    // spec: S-0511 Terms (start ramps: top-left along x, top-right along y)
    if is(b.top_left) {
        return Some(Piece::StartRamp(Line::X));
    }
    if is(b.top_right) {
        return Some(Piece::StartRamp(Line::Y));
    }
    // spec: S-0511 Terms (far ramps: bottom-right along x, bottom-left along y)
    if is(b.bottom_right) {
        return Some(Piece::FarRamp(Line::X));
    }
    if is(b.bottom_left) {
        return Some(Piece::FarRamp(Line::Y));
    }
    let piers = [(Line::X, b.middle[0]), (Line::Y, b.middle[1])];
    // spec: S-0511 Terms (a pier: q = m + k, k = 0 to 4; test x states 0 to 3 before y states 0 to 3, and an x match only at collapsed state 4 does not override a non-collapsed y match)
    for states in [0..=3, 4..=4] {
        for (line, m) in piers {
            if m != glasnost_map::terrain::UNSET_POSITION && states.contains(&(q - m)) {
                return Some(Piece::Pier(line, q - m));
            }
        }
    }
    None
}

/// The family and state *k* of a pier tile in `c` read along `line` alone
/// (S-0511 Terms): repair's start-piece selection and the span searches use
/// the axis they are working on, not the direct-attempt interpretation of
/// a position in both pier ranges.
pub fn pier_along(w: &World, c: CellXY, line: Line) -> Option<(Family, i32)> {
    let cell = w.cells.cell(c)?;
    let (f, q) = tile_q(w, cell.tile)?;
    let b = w.catalogue.as_ref()?.bridge_positions;
    let m = match line {
        Line::X => b.middle[0],
        Line::Y => b.middle[1],
    };
    // spec: S-0511 Terms (a pier: q = m + k, k = 0 to 4, m = m1 along x and m2 along y; repair and span repair use the selected axis)
    (m != glasnost_map::terrain::UNSET_POSITION && (m..=m + 4).contains(&q)).then_some((f, q - m))
}

/// The family and piece of the tile in `c`.
pub fn piece_at(w: &World, c: CellXY) -> Option<(Family, Piece)> {
    let cell = w.cells.cell(c)?;
    let (f, q) = tile_q(w, cell.tile)?;
    Some((f, piece_of_q(w, q)?))
}

/// The sub-tile of `c`.
pub fn sub(w: &World, c: CellXY) -> Option<u8> {
    w.cells.cell(c).map(|x| x.sub_tile)
}

/// The pier's reference sub-tile (S-0511 Terms): 4 along x, 2 along y.
pub fn reference_sub(line: Line) -> u8 {
    match line {
        Line::X => 4, // spec: S-0511 Terms (reference cell: sub-tile 4 along x)
        Line::Y => 2, // spec: S-0511 Terms (reference cell: sub-tile 2 along y)
    }
}

/// The sub-tile of a pier cell that starts the next span (S-0511 claim 7):
/// 5 along x, 7 along y.
pub fn pier_start_sub(line: Line) -> u8 {
    match line {
        Line::X => 5, // spec: S-0511 claim 7 (a pier at sub-tile 5 starts a span along x)
        Line::Y => 7, // spec: S-0511 claim 7 (a pier at sub-tile 7 starts a span along y)
    }
}

/// The sub-tile of a start ramp's starting cell (S-0511 claims 7 and 18):
/// 8 along x, 12 along y.
pub fn ramp_start_sub(line: Line) -> u8 {
    match line {
        Line::X => 8,  // spec: S-0511 claim 7 (a start ramp at sub-tile 8 along x)
        Line::Y => 12, // spec: S-0511 claim 7 (a start ramp at sub-tile 12 along y)
    }
}

/// Whether a pier cell's sub-tile has no deck (S-0511 claims 2 and 19):
/// even along x, at most 4 along y.
pub fn pier_body_sub(line: Line, s: u8) -> bool {
    match line {
        // spec: S-0511 claim 2 (even sub-tiles along x)
        Line::X => s.is_multiple_of(2),
        // spec: S-0511 claim 2 (sub-tiles at most 4 along y)
        Line::Y => s <= 4,
    }
}

/// The cells of the placed tile that holds `c` (S-0307 claim 23's
/// rectangle from the sub-tile and the tile's width) that still hold that
/// tile at their own place in the rectangle. Row-major order. Used for a
/// ramp's whole-tile look (S-0511 claim 4); a pier's state replacement uses
/// [`pier_component`].
pub fn tile_cells(w: &World, c: CellXY) -> Vec<CellXY> {
    let Some(cell) = w.cells.cell(c) else {
        return Vec::new();
    };
    let tile = cell.tile;
    let Some((tw, th)) = w.tile_size(tile) else {
        return vec![c];
    };
    let (tw, th) = (i32::from(tw.max(1)), i32::from(th.max(1)));
    let s = i32::from(cell.sub_tile);
    // spec: S-0307 claim 23 (x₀ = x − (s mod W), y₀ = y − ⌊s / W⌋)
    let (x0, y0) = (c.x - s % tw, c.y - s / tw);
    let mut out = Vec::new();
    for j in 0..th {
        for i in 0..tw {
            let p = CellXY::new(x0 + i, y0 + j);
            // spec: S-0307 claim 23 (the placed tile's cells: the rectangle cells holding it at their own sub-tile); S-0511 claim 4 (the ramp's whole tile)
            if let Some(pc) = w.cells.cell(p)
                && pc.tile == tile
                && i32::from(pc.sub_tile) == j * tw + i
            {
                out.push(p);
            }
        }
    }
    out
}

/// A pier's affected tile cells for a state replacement (S-0511 Terms):
/// the eight-neighbour-connected component containing reference cell `r`
/// whose cells have exactly `r`'s tile number. Row-major order.
pub fn pier_component(w: &World, r: CellXY) -> Vec<CellXY> {
    let Some(tile) = w.cells.cell(r).map(|x| x.tile) else {
        return Vec::new();
    };
    let mut seen = std::collections::BTreeSet::from([(r.y, r.x)]);
    let mut todo = vec![r];
    while let Some(c) = todo.pop() {
        // spec: S-0511 Terms (eight-neighbour-connected; sub-tile numbers and placed-tile origins do not restrict the set)
        for d in crate::regions::STEPS {
            let n = at(c, d, 1);
            // spec: S-0511 Terms (cells with exactly the reference cell's pre-change tile number)
            if w.cells.cell(n).is_some_and(|x| x.tile == tile) && seen.insert((n.y, n.x)) {
                todo.push(n);
            }
        }
    }
    seen.into_iter().map(|(y, x)| CellXY::new(x, y)).collect()
}

/// The reference cell of the pier whose tile holds `c` (S-0511 Terms).
pub fn pier_reference(w: &World, c: CellXY, line: Line) -> Option<CellXY> {
    let cell = w.cells.cell(c)?;
    let (tw, _) = w.tile_size(cell.tile)?;
    let tw = i32::from(tw.max(1));
    let s = i32::from(cell.sub_tile);
    let r = i32::from(reference_sub(line));
    // spec: S-0307 claim 23 (the rectangle's origin from the sub-tile and width); S-0511 Terms (the reference cell's sub-tile)
    let x0 = c.x - s % tw;
    let y0 = c.y - s / tw;
    Some(CellXY::new(x0 + r % tw, y0 + r / tw))
}

/// The reference cell of the pier at `c` for a direct attempt (S-0511
/// claims 2 and 6): [`pier_reference`], taken only when that cell holds
/// `c`'s own tile number at the reference sub-tile. A cell painted over,
/// or a part of the pier cut off from it in another state, gives none
/// (r13, Q-3046). A received notice uses no reference cell.
pub fn pier_reference_of(w: &World, c: CellXY, line: Line) -> Option<CellXY> {
    let tile = w.cells.cell(c)?.tile;
    let r = pier_reference(w, c, line)?;
    let rc = w.cells.cell(r)?;
    // spec: S-0511 claim 6 (r13, Q-3046, A reference cell that does not hold the pier: a direct attempt acts only when its reference cell holds the attempted cell's own tile, at sub-tile r: the same family, axis and state)
    (rc.tile == tile && rc.sub_tile == reference_sub(line)).then_some(r)
}

// ---------------------------------------------------------------------
// High-bridge sections.

/// A family's high overlays Hₓ and Hᵧ (S-0511 Import mapping, *piece-role
/// binding*): 24 and 25 for the bridge-role family, 237 and 238 for the
/// wooden-bridge-role family, with marking directions map north (along x)
/// and map west (along y) (S-0211 claims 14 and 15).
// spec: S-0511 Import mapping (piece-role binding); S-0211 claim 15 (indices and marking directions)
pub const HIGH: [(u8, Family, Line, (i32, i32)); 4] = [
    (24, Family::Bridge, Line::X, (0, -1)),
    (25, Family::Bridge, Line::Y, (-1, 0)),
    (237, Family::Wooden, Line::X, (0, -1)),
    (238, Family::Wooden, Line::Y, (-1, 0)),
];

/// The high overlay of `family` along `line`.
pub fn high_overlay(family: Family, line: Line) -> u8 {
    HIGH.iter()
        .find(|h| h.1 == family && h.2 == line)
        .map_or(NO_OVERLAY, |h| h.0)
}

/// A high overlay's family, line and marking direction.
pub fn high_of(o: u8) -> Option<(Family, Line, (i32, i32))> {
    HIGH.iter().find(|h| h.0 == o).map(|h| (h.1, h.2, h.3))
}

/// A section (S-0511 Terms): its overlay's cell, family and line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Section {
    /// The section cell.
    pub cell: CellXY,
    /// Its family.
    pub family: Family,
    /// Its line.
    pub line: Line,
    /// Its marking direction d (S-0211 claim 15).
    pub d: (i32, i32),
}

impl Section {
    /// Its four marked cells in claim 5's blow-up order: the section cell,
    /// one and two steps along d, one step against it.
    // spec: S-0511 claim 5 (a section marks four cells)
    pub fn cells(&self) -> [CellXY; 4] {
        // spec: S-0511 claim 5 (section cell, one step along the marking direction, two steps along it, one step against it)
        [
            self.cell,
            at(self.cell, self.d, 1),
            at(self.cell, self.d, 2),
            at(self.cell, self.d, -1),
        ]
    }
}

/// The section whose overlay is in `c` itself.
pub fn section_in(w: &World, c: CellXY) -> Option<Section> {
    let cell = w.cells.cell(c)?;
    let (family, line, d) = high_of(cell.overlay)?;
    Some(Section {
        cell: c,
        family,
        line,
        d,
    })
}

/// The responsible section of high-bridge cell `c` when its overlay is a
/// section (S-0511 claim 2; S-0211 claim 15 (a)): no search for another
/// overlay when it is missing or replaced.
pub fn responsible_section(w: &World, c: CellXY) -> Option<Section> {
    // spec: S-0511 claim 2 (historical responsibility; no nearest-overlay or directional tie-break, no search for another section)
    section_in(w, w.cells.responsible(c)?)
}

/// The section cell and line of a cell carrying a deck or bridge-body mark
/// (S-0511 claims 7 and 16): a high-bridge cell gives its responsible
/// section; a body-marked cell gives the section cell of claim 16's body
/// rule, with no family.
pub fn section_of(w: &World, b: CellXY) -> Option<(CellXY, Line, Option<Family>)> {
    if w.cells.is_bridge(b) {
        // spec: S-0511 claim 16 (a high-bridge cell gives its section cell); claim 7 (its responsible section)
        let s = responsible_section(w, b)?;
        return Some((s.cell, s.line, Some(s.family)));
    }
    let line = match w.cells.axis(b) {
        crate::cells::Axis::X => Line::X,
        crate::cells::Axis::Y => Line::Y,
        crate::cells::Axis::None => return None,
    };
    let d = line.across();
    // spec: S-0511 claim 16 and Constants (t: the distance to the nearest cell without the body mark across, toward increasing y on an x bridge, increasing x on a y bridge; 1 ≤ t ≤ 4)
    let t = (1..=4).find(|&t| !w.cells.body(at(b, d, t)))?;
    // spec: S-0511 claim 16 (the section cell displaced by (t − 2) d)
    Some((at(b, d, t - 2), line, None))
}

/// The marking direction of a section of `line` (S-0211 claim 15): map
/// north along x, map west along y.
pub fn marking_direction(line: Line) -> (i32, i32) {
    // spec: S-0211 claim 15 (indices 24 and 237 mark along step direction 0, map north; 25 and 238 along step direction 6, map west)
    match line {
        Line::X => (0, -1),
        Line::Y => (-1, 0),
    }
}

/// A section's state (S-0511 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    /// Whole.
    Intact,
    /// Chipped where it meets its start-side neighbour.
    ChippedStart,
    /// Chipped on the far side.
    ChippedFar,
    /// Damaged.
    Damaged,
    /// Its start-side neighbour is gone.
    OpenStart,
    /// Its far-side neighbour is gone.
    OpenFar,
}

/// The state of a section of `line` with overlay data `v`, or `None` for a
/// value outside the section-state table (S-0511 Terms and claim 2).
pub fn state_in_table(line: Line, v: u8) -> Option<State> {
    // spec: S-0511 Terms (intact 0 to 3 along x, 9 to 12 along y; the other states as in state_of)
    let intact = match line {
        Line::X => (0..=3).contains(&v),
        Line::Y => (9..=12).contains(&v),
    };
    let st = state_of(line, v);
    (intact || st != State::Intact).then_some(st)
}

/// The state of a section of `line` with overlay data `v` (S-0511 Terms).
// spec: S-0511 Terms (the section states along x and along y)
pub fn state_of(line: Line, v: u8) -> State {
    match (line, v) {
        // spec: S-0511 Terms (along x: intact 0 to 3, 4 chipped start, 5 chipped far, 6 damaged, 7 open start, 8 open far)
        (Line::X, 4) => State::ChippedStart,
        (Line::X, 5) => State::ChippedFar,
        (Line::X, 6) => State::Damaged,
        (Line::X, 7) => State::OpenStart,
        (Line::X, 8) => State::OpenFar,
        // spec: S-0511 Terms (along y: intact 9 to 12, 14 chipped start, 13 chipped far, 15 damaged, 17 open start, 16 open far)
        (Line::Y, 14) => State::ChippedStart,
        (Line::Y, 13) => State::ChippedFar,
        (Line::Y, 15) => State::Damaged,
        (Line::Y, 17) => State::OpenStart,
        (Line::Y, 16) => State::OpenFar,
        _ => State::Intact,
    }
}

/// The overlay data of state `s` (S-0511 Terms); intact gives 0 along x and
/// 9 along y (claim 19).
// spec: S-0511 Terms and claim 19 (the overlay data of each section state)
pub fn data_of(line: Line, s: State) -> u8 {
    match (line, s) {
        // spec: S-0511 Terms and claim 19 (along x: intact 0, 4, 5, 6, 7, 8)
        (Line::X, State::Intact) => 0,
        (Line::X, State::ChippedStart) => 4,
        (Line::X, State::ChippedFar) => 5,
        (Line::X, State::Damaged) => 6,
        (Line::X, State::OpenStart) => 7,
        (Line::X, State::OpenFar) => 8,
        // spec: S-0511 Terms and claim 19 (along y: intact 9, 14, 13, 15, 17, 16)
        (Line::Y, State::Intact) => 9,
        (Line::Y, State::ChippedStart) => 14,
        (Line::Y, State::ChippedFar) => 13,
        (Line::Y, State::Damaged) => 15,
        (Line::Y, State::OpenStart) => 17,
        (Line::Y, State::OpenFar) => 16,
    }
}

// ---------------------------------------------------------------------
// Low pieces.

/// A low piece's symbolic role (S-0511 Terms, *strip class*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// I₀ to I₅.
    I(u8),
    /// D₀ to D₂.
    D(u8),
    /// The far-side end piece, whole.
    E,
    /// The far-side end piece, damaged.
    E2,
    /// The start-side end piece, whole.
    S,
    /// The start-side end piece, damaged.
    S2,
    /// Destroyed.
    X,
}

/// A strip class (S-0511 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Class {
    /// Intact.
    Intact,
    /// Damaged.
    Damaged,
    /// Far-side end piece, whole.
    FarWhole,
    /// Start-side end piece, whole.
    StartWhole,
    /// A damaged end piece.
    EndDamaged,
    /// Destroyed.
    Destroyed,
}

impl Role {
    /// The role's class.
    pub fn class(self) -> Class {
        match self {
            Role::I(_) => Class::Intact,
            Role::D(_) => Class::Damaged,
            Role::E => Class::FarWhole,
            Role::S => Class::StartWhole,
            Role::E2 | Role::S2 => Class::EndDamaged,
            Role::X => Class::Destroyed,
        }
    }
}

/// The low-piece base type *t*₀ of each family and line (S-0511 Import
/// mapping; S-0202 claim 10's bridge piece type): the lower of an axis's
/// two bases is the wooden-bridge-role palette.
// spec: S-0511 Import mapping (t₀ of S-0202 claim 10; the lower base is the wooden-bridge-role palette); S-0202 Formulas (bridge piece type bases 74, 83, 205, 214)
pub const LOW_BASES: [(Family, Line, u8); 4] = [
    (Family::Wooden, Line::X, 74),
    (Family::Wooden, Line::Y, 83),
    (Family::Bridge, Line::X, 205),
    (Family::Bridge, Line::Y, 214),
];

/// A role's displacement from *t*₀ along `line` (S-0511 Import mapping).
// spec: S-0511 Import mapping (each role's displacement from t₀)
pub fn displacement(line: Line, r: Role) -> u8 {
    match (r, line) {
        // spec: S-0511 Import mapping (I₀ to I₅: 0 to 5; D₀ to D₂: 6 to 8)
        (Role::I(k), _) => k,
        (Role::D(k), _) => 6 + k,
        // spec: S-0511 Import mapping (E, E′: 18, 19 along x; 13, 14 along y)
        (Role::E, Line::X) => 18,
        (Role::E2, Line::X) => 19,
        (Role::E, Line::Y) => 13,
        (Role::E2, Line::Y) => 14,
        // spec: S-0511 Import mapping (S, S′: 20, 21 along x; 15, 16 along y)
        (Role::S, Line::X) => 20,
        (Role::S2, Line::X) => 21,
        (Role::S, Line::Y) => 15,
        (Role::S2, Line::Y) => 16,
        // spec: S-0511 Import mapping (X: 26 along x, 18 along y)
        (Role::X, Line::X) => 26,
        (Role::X, Line::Y) => 18,
    }
}

/// The role of displacement `k` along `line`.
// spec: S-0511 Terms (strip classes I₀ to I₅ and D₀ to D₂)
fn role_of(line: Line, k: u8) -> Option<Role> {
    let all = [
        Role::I(0),
        Role::I(1),
        Role::I(2),
        Role::I(3),
        Role::I(4),
        Role::I(5),
        Role::D(0),
        Role::D(1),
        Role::D(2),
        Role::E,
        Role::E2,
        Role::S,
        Role::S2,
        Role::X,
    ];
    all.into_iter().find(|&r| displacement(line, r) == k)
}

/// A low piece.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Low {
    /// Its family.
    pub family: Family,
    /// Its line.
    pub line: Line,
    /// Its role.
    pub role: Role,
}

/// The low piece that overlay `o` is, if any.
pub fn low_of(o: u8) -> Option<Low> {
    if o == NO_OVERLAY {
        return None;
    }
    LOW_BASES.iter().find_map(|&(family, line, base)| {
        let k = o.checked_sub(base)?;
        let role = role_of(line, k)?;
        Some(Low { family, line, role })
    })
}

/// The overlay type of a low piece.
pub fn low_overlay(family: Family, line: Line, role: Role) -> u8 {
    let base = LOW_BASES
        .iter()
        .find(|b| b.0 == family && b.1 == line)
        .map_or(0, |b| b.2);
    base + displacement(line, role)
}

/// The low piece in `c`.
pub fn low_at(w: &World, c: CellXY) -> Option<Low> {
    low_of(w.cells.cell(c)?.overlay)
}

/// Whether `c` holds a low piece of `family`.
pub fn low_of_family(w: &World, c: CellXY, family: Family) -> bool {
    low_at(w, c).is_some_and(|l| l.family == family)
}

/// The strip centre selected for an attempt at `c` (S-0511 claim 13), with
/// membership meaning a low piece of `family`.
pub fn strip_centre(w: &World, c: CellXY, line: Line, family: Family) -> CellXY {
    let d = line.across();
    let low = |p: CellXY| low_of_family(w, p, family);
    // spec: S-0511 claim 13 (centre selection: (x, y − 1) not low → (x, y + 1); (x, y − 1) and (x, y − 2) low → (x, y − 1); otherwise (x, y))
    if !low(at(c, d, -1)) {
        at(c, d, 1)
    } else if low(at(c, d, -2)) {
        at(c, d, -1)
    } else {
        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0511 Import mapping (displacements bind distinct overlay types; no collisions between families and axes)
    #[test]
    fn low_roles_are_distinct() {
        let mut seen = std::collections::BTreeSet::new();
        for &(f, l, _) in &LOW_BASES {
            for k in 0..=40u8 {
                if let Some(r) = role_of(l, k) {
                    let o = low_overlay(f, l, r);
                    assert!(seen.insert(o), "overlay {o} bound twice");
                    assert_eq!(
                        low_of(o),
                        Some(Low {
                            family: f,
                            line: l,
                            role: r
                        })
                    );
                }
            }
        }
        assert_eq!(seen.len(), 4 * 14);
    }

    // spec: S-0511 Terms (section states round-trip through the data values)
    #[test]
    fn section_states() {
        for line in [Line::X, Line::Y] {
            for s in [
                State::Intact,
                State::ChippedStart,
                State::ChippedFar,
                State::Damaged,
                State::OpenStart,
                State::OpenFar,
            ] {
                assert_eq!(state_of(line, data_of(line, s)), s);
            }
        }
        assert_eq!(state_of(Line::X, 3), State::Intact);
        assert_eq!(state_of(Line::Y, 12), State::Intact);
    }
}
