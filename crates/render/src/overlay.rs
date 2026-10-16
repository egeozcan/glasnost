//! Overlay and terrain object drawing (S-0112): which row of the overlay
//! drawing table a cell meets, the tiberium image runs and the picture they
//! choose, bridge deck frames, shadow frames, and the overlay draw point
//! shifts of S-0102 claim 24. Pure integer rules; the scene builder applies
//! them.

/// Overlay list positions of the veinhole markers (S-0112 Constants).
pub const VEINHOLE_MARKERS: [u8; 2] = [167, 178]; // spec: S-0112 Constants (veinhole marker positions 167, 178)

/// The vein overlay's list position (S-0112 Constants, S-0009 Terms).
pub const VEIN: u8 = 126; // spec: S-0112 Constants (vein overlay position 126)

/// Bridge positions whose placement makes a deck cell (S-0112 claim 1).
pub const DECK_BRIDGES: [u8; 4] = [24, 25, 237, 238]; // spec: S-0112 Constants (bridge positions that make deck cells)

/// The overlay type that takes the −15 shift off deck cells (S-0102 claim
/// 24).
pub const SHIFT_239: u8 = 239; // spec: S-0102 claim 24 (the overlay type with index 239)

/// Deck variation table T(x mod 4, y mod 4), by row y mod 4 (S-0112 claim 6).
// spec: S-0112 Constants (deck variation table T)
pub const DECK_T: [[u8; 4]; 4] = [[0, 1, 2, 3], [3, 2, 1, 0], [2, 3, 0, 1], [1, 0, 3, 2]];

/// Deck data values that vary by position (S-0112 claim 6).
pub const DECK_VARYING: [u8; 2] = [0, 9]; // spec: S-0112 Constants (deck data values that vary: 0, 9)

/// Overlay drawing switches of S-0112. Each defaults to the original.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OverlayQuirks {
    /// `quirks.wall_local_player_colour` (claim 1 row 4).
    pub wall_local_player_colour: bool,
    /// `quirks.resource_palettes_from_start` (claim 2).
    pub resource_palettes_from_start: bool,
    /// `quirks.overlay_shadow_from_stored_type` (claim 8).
    pub overlay_shadow_from_stored_type: bool,
    /// `quirks.image_two_keeps_sloped_run` (claim 10).
    pub image_two_keeps_sloped_run: bool,
    /// `quirks.crate_data_value_ignored` (claim 1 row 6, claim 8).
    pub crate_data_value_ignored: bool,
}

impl Default for OverlayQuirks {
    fn default() -> Self {
        // spec: S-0112 Edge cases and quirks (every switch defaults on, the original)
        Self {
            wall_local_player_colour: true,
            resource_palettes_from_start: true,
            overlay_shadow_from_stored_type: true,
            image_two_keeps_sloped_run: true,
            crate_data_value_ignored: true, // spec: S-0112 Edge cases and quirks (crates ignore their data value)
        }
    }
}

impl OverlayQuirks {
    /// The image frame of a crate's body (claim 1 row 6): 0 with
    /// `quirks.crate_data_value_ignored` on, else the overlay data value.
    pub fn crate_body_frame(&self, data: u8) -> usize {
        // spec: S-0112 claim 1 row 6 (data value 0 with the switch on; the overlay data value with it off)
        if self.crate_data_value_ignored {
            0
        } else {
            usize::from(data)
        }
    }

    /// The image frame of a crate's shadow (claim 8 precedence 1) for a
    /// sprite of `n` frames: ⌊*N*/2⌋ with the switch on, else the data value
    /// plus ⌊*N*/2⌋.
    pub fn crate_shadow_frame(&self, data: u8, n: usize) -> usize {
        // spec: S-0112 claim 8 precedence 1 (N/2 with the switch on; the data value + N/2 with it off)
        if self.crate_data_value_ignored {
            n / 2
        } else {
            usize::from(data) + n / 2
        }
    }

    /// The switches by name.
    pub fn named(&self) -> [(&'static str, bool); 5] {
        [
            ("wall_local_player_colour", self.wall_local_player_colour),
            (
                "resource_palettes_from_start",
                self.resource_palettes_from_start,
            ),
            (
                "overlay_shadow_from_stored_type",
                self.overlay_shadow_from_stored_type,
            ),
            (
                "image_two_keeps_sloped_run",
                self.image_two_keeps_sloped_run,
            ),
            ("crate_data_value_ignored", self.crate_data_value_ignored),
        ]
    }
}

/// The row of the overlay drawing table a cell meets (S-0112 claim 1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    /// 1: a veinhole marker: the body is not drawn.
    Marker,
    /// 2: a deck cell: tile shade table, scaled raised level, deck frame.
    Deck,
    /// 3: a tiberium overlay: start-up resource table, 1000, claim 4.
    Tiberium,
    /// 4: a wall: the local player's house-colour palette, U.
    Wall,
    /// 5: the vein overlay: start-up resource table, scaled ground.
    Vein,
    /// 6: a crate: tile shade table, scaled ground, frame 0.
    Crate,
    /// 7: rubble: tile shade table, scaled ground, claim 5.
    Rubble,
    /// 8: any other overlay: tile shade table, scaled ground, data value.
    Other,
}

/// The flags of an overlay type that the drawing table reads (Import
/// mapping).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    /// `Tiberium`.
    pub tiberium: bool,
    /// `Wall`.
    pub wall: bool,
    /// `Crate`.
    pub crate_: bool,
    /// `IsRubble`.
    pub rubble: bool,
    /// The overlay's land type is railroad (S-0102 claim 24).
    pub railroad: bool,
}

/// The first row of the drawing table that a cell holding overlay type
/// `position` with `flags` meets (S-0112 claim 1).
pub fn table_row(position: u8, flags: Flags, deck: bool) -> Row {
    // spec: S-0112 claim 1 (rows in precedence order 1 to 8)
    if VEINHOLE_MARKERS.contains(&position) {
        Row::Marker
    } else if deck {
        Row::Deck
    } else if flags.tiberium {
        Row::Tiberium
    } else if flags.wall {
        Row::Wall
    } else if position == VEIN {
        Row::Vein
    } else if flags.crate_ {
        Row::Crate
    } else if flags.rubble {
        Row::Rubble
    } else {
        Row::Other
    }
}

/// A tiberium's image run (S-0112 Terms): a flat run of `f` positions from
/// `p`, then a sloped run of `s`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Run {
    /// First position *P*.
    pub p: i32,
    /// Flat run length *F*.
    pub f: i32,
    /// Sloped run length *S* (0 or 8).
    pub s: i32,
}

impl Run {
    /// Whether overlay list position `position` lies in the flat or the
    /// sloped run.
    pub fn contains(&self, position: i32) -> bool {
        position >= self.p && position < self.p + self.f + self.s
    }
}

/// Applies one image number `v` given by a rules layer to a tiberium's run
/// (S-0112 claim 10, Constants table).
pub fn apply_image(run: Option<Run>, v: i32, q: &OverlayQuirks) -> Option<Run> {
    // spec: S-0112 Constants (image runs: 2 → 27/12/keeps S; 3 → 127/12/8; 4 → 147/12/8; other → 102/12/8)
    const FLAT: i32 = 12; // spec: S-0112 Constants (F is 12 for every image number)
    const SLOPED: i32 = 8; // spec: S-0112 Constants (S is 8 for every value other than 2 and −1)
    match v {
        // spec: S-0112 claim 10 (−1 changes nothing)
        -1 => run,
        // spec: S-0112 claim 10 (2 sets P and F only; quirks.image_two_keeps_sloped_run)
        2 => Some(Run {
            p: 27,
            f: FLAT,
            s: if q.image_two_keeps_sloped_run {
                run.map_or(0, |r| r.s)
            } else {
                0
            },
        }),
        3 => Some(Run {
            p: 127,
            f: FLAT,
            s: SLOPED,
        }),
        4 => Some(Run {
            p: 147,
            f: FLAT,
            s: SLOPED,
        }),
        _ => Some(Run {
            p: 102,
            f: FLAT,
            s: SLOPED,
        }),
    }
}

/// The tiberium of a tiberium overlay at `position` (S-0112 Terms): the
/// first tiberium in list order whose run contains it, else the first
/// tiberium. `None` when the list is empty.
pub fn tiberium_of(position: i32, runs: &[Option<Run>]) -> Option<usize> {
    if runs.is_empty() {
        return None;
    }
    // spec: S-0112 Terms (the first tiberium whose run contains the position; else the first of the list)
    Some(
        runs.iter()
            .position(|r| r.is_some_and(|r| r.contains(position)))
            .unwrap_or(0),
    )
}

/// The overlay list position whose sprite a tiberium overlay's body shows on
/// cell (x, y) of slope shape `slope` (S-0112 claim 4, Formulas). A run
/// without a sloped run gives the flat choice on slopes (claim 11, a
/// departure).
pub fn tiberium_body(run: Run, (x, y): (i32, i32), slope: u8) -> i32 {
    let xy = i64::from(x) * i64::from(y);
    let k = run.s / 4; // spec: S-0112 Formulas (k = ⌊S/4⌋)
    if (1..=4).contains(&slope) && k > 0 {
        // spec: S-0112 Formulas (p = P + F + (s − 1)·k + ((x·y) mod k))
        run.p + run.f + (i32::from(slope) - 1) * k + xy.rem_euclid(i64::from(k)) as i32
    } else {
        // spec: S-0112 Formulas (flat: p = P + ((x·y) mod F)); claim 11 (S = 0 on a slope: the flat choice)
        run.p + xy.rem_euclid(i64::from(run.f.max(1))) as i32
    }
}

/// The body image frame of a deck cell (S-0112 claim 6, Formulas).
pub fn deck_frame(d: u8, (x, y): (i32, i32)) -> usize {
    // spec: S-0112 claim 6 (d + T(x mod 4, y mod 4) for d = 0 and 9, else d)
    if DECK_VARYING.contains(&d) {
        usize::from(d) + usize::from(DECK_T[y.rem_euclid(4) as usize][x.rem_euclid(4) as usize])
    } else {
        usize::from(d)
    }
}

/// The vertical shift *s* of an overlay's draw point and the displacement
/// of its shadow's draw point (S-0102 claim 24).
pub fn draw_shift(position: u8, flags: Flags, deck: bool, data: u8) -> (i32, (i32, i32)) {
    // spec: S-0102 claim 24 (s = s_type + s_rail + s_cell, three independent terms)
    let s_type = if position == VEIN {
        -13 // spec: S-0102 claim 24 (s_type: the overlay type with index 126)
    } else if flags.tiberium || flags.wall || flags.crate_ {
        -12 // spec: S-0102 claim 24 (s_type: otherwise ore or gems, a wall or a crate)
    } else {
        0
    };
    // spec: S-0102 claim 24 (s_rail: a railroad land type)
    let s_rail = if flags.railroad { -1 } else { 0 };
    let deck_mid = deck && (9..=17).contains(&data); // spec: S-0102 claim 24 (data 9 to 17)
    let s_cell = if deck_mid {
        -31 // spec: S-0102 claim 24 (s_cell: a bridge-deck cell with data 9 to 17)
    } else if deck {
        -16 // spec: S-0102 claim 24 (s_cell: otherwise a bridge-deck cell)
    } else if position == SHIFT_239 {
        -15 // spec: S-0102 claim 24 (s_cell: no bridge deck, the overlay type with index 239)
    } else {
        0
    };
    // spec: S-0102 claim 24 (the shadow 15 left and 7 down on a deck cell with data 9 to 17)
    let shadow = if deck_mid { (-15, 7) } else { (0, 0) };
    (s_type + s_rail + s_cell, shadow)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// spec: S-0112 claim 1 (the first row a cell meets)
    #[test]
    fn claim_1_rows() {
        let t = Flags {
            tiberium: true,
            wall: true,
            ..Flags::default()
        };
        assert_eq!(table_row(167, t, true), Row::Marker);
        assert_eq!(table_row(24, t, true), Row::Deck);
        assert_eq!(table_row(5, t, false), Row::Tiberium);
        let w = Flags {
            wall: true,
            crate_: true,
            ..Flags::default()
        };
        assert_eq!(table_row(VEIN, w, false), Row::Wall);
        assert_eq!(table_row(VEIN, Flags::default(), false), Row::Vein);
        let c = Flags {
            crate_: true,
            rubble: true,
            ..Flags::default()
        };
        assert_eq!(table_row(3, c, false), Row::Crate);
        let r = Flags {
            rubble: true,
            ..Flags::default()
        };
        assert_eq!(table_row(3, r, false), Row::Rubble);
        assert_eq!(table_row(3, Flags::default(), false), Row::Other);
    }

    /// spec: S-0112 claims 4 and 10 to 12 (runs by image number, the tiberium of an overlay, the body type)
    #[test]
    fn claims_4_10_runs() {
        let q = OverlayQuirks::default();
        let a = apply_image(None, 1, &q).unwrap();
        assert_eq!(
            a,
            Run {
                p: 102,
                f: 12,
                s: 8
            }
        );
        // 2 after another value keeps S = 8; 2 alone has S = 0; −1 keeps the run.
        let b = apply_image(Some(a), 2, &q).unwrap();
        assert_eq!(b, Run { p: 27, f: 12, s: 8 });
        assert_eq!(apply_image(None, 2, &q).unwrap().s, 0);
        assert_eq!(apply_image(Some(b), -1, &q), Some(b));
        assert_eq!(apply_image(None, -1, &q), None);
        let off = OverlayQuirks {
            image_two_keeps_sloped_run: false,
            ..q
        };
        assert_eq!(apply_image(Some(a), 2, &off).unwrap().s, 0);
        // Flat: P + (x·y mod F); slope s: P + F + (s − 1)·k + (x·y mod k).
        assert_eq!(tiberium_body(a, (3, 5), 0), 102 + 15 % 12);
        assert_eq!(tiberium_body(a, (3, 5), 3), 102 + 12 + 2 * 2 + 1);
        // Claim 10's example: image 2 kept S = 8: positions 39 to 46 on slopes.
        assert_eq!(tiberium_body(b, (2, 3), 1), 39);
        // Claim 11: no sloped run → the flat choice.
        let flat_only = Run { p: 27, f: 12, s: 0 };
        assert_eq!(tiberium_body(flat_only, (3, 5), 2), 27 + 3);
        // The tiberium of an overlay: the first whose run holds it, else the first.
        let runs = [None, Some(a), Some(b)];
        assert_eq!(tiberium_of(110, &runs), Some(1));
        assert_eq!(tiberium_of(30, &runs), Some(2));
        assert_eq!(tiberium_of(5, &runs), Some(0));
    }

    /// spec: S-0112 claim 1 row 6 and claim 8 (the crate switch: frame 0 and N/2 with it on, the data value and data + N/2 with it off)
    #[test]
    fn crate_frames_follow_the_switch() {
        let on = OverlayQuirks::default();
        assert!(on.crate_data_value_ignored);
        assert_eq!(
            (on.crate_body_frame(5), on.crate_shadow_frame(5, 4)),
            (0, 2)
        );
        let off = OverlayQuirks {
            crate_data_value_ignored: false,
            ..on
        };
        assert_eq!(
            (off.crate_body_frame(5), off.crate_shadow_frame(5, 4)),
            (5, 7)
        );
    }

    /// spec: S-0112 claim 6 (deck frames vary for data 0 and 9)
    #[test]
    fn claim_6_deck_frames() {
        assert_eq!(deck_frame(0, (1, 0)), 1);
        assert_eq!(deck_frame(9, (0, 1)), 12);
        assert_eq!(deck_frame(5, (1, 1)), 5);
    }

    /// spec: S-0102 claim 24 (overlay draw point shifts and the deck shadow displacement)
    #[test]
    fn claim_24_shifts() {
        let ore = Flags {
            tiberium: true,
            ..Flags::default()
        };
        assert_eq!(draw_shift(5, ore, false, 0), (-12, (0, 0)));
        let rail = Flags {
            railroad: true,
            ..Flags::default()
        };
        assert_eq!(draw_shift(VEIN, rail, false, 0).0, -14);
        assert_eq!(draw_shift(VEIN, Flags::default(), false, 0).0, -13);
        assert_eq!(draw_shift(24, Flags::default(), true, 10), (-31, (-15, 7)));
        assert_eq!(draw_shift(SHIFT_239, Flags::default(), false, 0).0, -15);
        assert_eq!(draw_shift(SHIFT_239, Flags::default(), true, 0).0, -16);
    }
}
