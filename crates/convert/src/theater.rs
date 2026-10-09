//! A theater's tile catalogue: which file holds each tile number, the fill
//! base tiles and the tile number translation.
//!
//! S-0009 gives the base tiles (claim 6), the translation (claim 8,
//! Formulas) and the keys they read; the glossary numbers tiles
//! consecutively across tilesets in declaration order. Which file holds the
//! theater's data, which tileset sections exist and how tile files are
//! named belong to the planned theater spec and are asked in Q-0009. Until it is
//! answered this module uses the loader layout of OpenRA's
//! `OpenRA.Mods.Cnc/UtilityCommands/LegacyTilesetImporter.cs` (Copyright (c)
//! The OpenRA Developers and Contributors, GPL-3.0-or-later): sections
//! `TileSet0000`, `TileSet0001`, … read until the first missing number,
//! `TilesInSet` defaulting to 1, and tile *i* (1-based) of a set in the file
//! `FileName` ‖ two-digit *i* ‖ `.` ‖ extension. Everything taken from there
//! is marked `provisional (Q-0009)`.

use glasnost_ini::Document;
use glasnost_ini::value;
use glasnost_vfs::theater::Theater;

/// Theater index whose water base tile is always unset (S-0009 claim 6).
const NO_WATER_THEATER: Theater = Theater::Lunar; // spec: S-0009 Constants (theater 5)

/// The unset base tile (S-0009 claim 6).
pub const UNSET_BASE_TILE: i32 = -1; // spec: S-0009 Constants (unset base tile)

/// One tileset of a theater.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tileset {
    /// Section number (`TileSetNNNN`).
    pub number: u32,
    /// The tileset's file-name key (provisional, Q-0009).
    pub file_name: Vec<u8>,
    /// The tileset's name key, for display only.
    pub set_name: Vec<u8>,
    /// Actual tile count C (S-0009 Import mapping, `TilesInSet`).
    pub count: i32,
    /// Authored tile count A, if given (`LastTilesInSet`).
    pub authored: Option<i32>,
    /// First tile number of the set in the theater's order.
    pub first: i32,
}

/// A theater's catalogue.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Catalogue {
    /// Tilesets in declaration order.
    pub tilesets: Vec<Tileset>,
    /// Total tile count of the theater.
    pub tile_count: i32,
    /// Clear base tile (S-0009 claim 6).
    pub clear_base: i32,
    /// Water base tile (S-0009 claim 6).
    pub water_base: i32,
    /// Tileset number of the clear tileset, if named.
    pub clear_set: Option<i32>,
    /// Translation pairs (t_i, δ_i) of S-0009 Formulas.
    pub translation: Vec<(i32, i32)>,
    /// Extension of tile files.
    pub extension: String,
}

/// The theater data file's name candidates (provisional, Q-0009): the
/// control name of S-0005 Table 2 with `MD.INI`, then with `.INI`.
pub fn data_file_names(theater: Theater) -> [String; 2] {
    let c = theater.control_name();
    [format!("{c}MD.INI"), format!("{c}.INI")]
}

/// Highest tileset section number tried (a bound of our own against
/// hostile files; provisional Q-0009 reading stops at the first gap anyway).
const MAX_TILESETS: u32 = 10_000; // nocite: bound of our own design

impl Catalogue {
    /// Reads the catalogue from the theater's data document.
    pub fn read(doc: &Document, theater: Theater) -> Self {
        let mut tilesets = Vec::new();
        let mut first = 0i32;
        let mut authored_first = 0i32;
        let mut translation = Vec::new();
        for number in 0..MAX_TILESETS {
            // provisional (Q-0009): sections are read until the first missing number
            let name = format!("TileSet{number:04}");
            let Some(section) = doc.section(name.as_bytes()) else {
                break;
            };
            // provisional (Q-0009): `TilesInSet` absent counts 1
            let count = section
                .value(b"TilesInSet")
                .and_then(value::integer)
                .unwrap_or(1);
            let authored = section.value(b"LastTilesInSet").and_then(value::integer);
            // spec: S-0009 Formulas (tile translation: pairs where A differs from C)
            if let Some(a) = authored
                && a != count
            {
                translation.push((authored_first.saturating_add(a), count.saturating_sub(a)));
            }
            authored_first = authored_first.saturating_add(authored.unwrap_or(count));
            tilesets.push(Tileset {
                number,
                file_name: section.value(b"FileName").unwrap_or_default().to_vec(),
                set_name: section.value(b"SetName").unwrap_or_default().to_vec(),
                count,
                authored,
                first,
            });
            first = first.saturating_add(count.max(0));
        }
        let general = |key: &[u8]| doc.value(b"General", key).and_then(value::integer);
        let first_of = |set: Option<i32>| {
            set.and_then(|n| usize::try_from(n).ok())
                .and_then(|n| tilesets.get(n))
                .map_or(UNSET_BASE_TILE, |t| t.first)
        };
        let clear_set = general(b"ClearTile");
        // spec: S-0009 claim 6 (first tile of the named tileset; absent or unknown: −1)
        let clear_base = first_of(clear_set);
        // spec: S-0009 claim 6 (theater 5: the water base tile is always −1)
        let water_base = if theater == NO_WATER_THEATER {
            UNSET_BASE_TILE
        } else {
            first_of(general(b"WaterSet"))
        };
        Self {
            tile_count: first,
            clear_base,
            water_base,
            clear_set,
            translation,
            extension: theater.extension().to_owned(),
            tilesets,
        }
    }

    /// Translates a map tile number into the theater's order (S-0009 claim 8,
    /// Formulas). 65535 is kept.
    pub fn translate(&self, m: i32) -> i32 {
        if m == glasnost_map::NO_TILE {
            return m;
        }
        // spec: S-0009 Formulas (m' = m + δ_1 + … + δ_j over the leading pairs with t_i ≤ m)
        let mut out = m;
        for &(t, delta) in &self.translation {
            if t > m {
                break;
            }
            out = out.saturating_add(delta);
        }
        out
    }

    /// The tile number whose artwork a cell's stored tile number shows
    /// (S-0009 claim 8): the marker 65535 and numbers at or beyond the tile
    /// count show the clear tile; a negative number is treated like the
    /// marker (our engine, a departure). `None` when the theater has no clear
    /// tile either.
    pub fn art_tile(&self, stored: i32) -> Option<i32> {
        // spec: S-0009 claim 8 (65535, numbers ≥ tile count and, in our engine, negatives: the clear tile)
        if stored < 0 || stored >= self.tile_count {
            (self.clear_base >= 0).then_some(self.clear_base)
        } else {
            Some(stored)
        }
    }

    /// The tileset holding tile `t` and the tile's 1-based position in it.
    pub fn locate(&self, t: i32) -> Option<(&Tileset, i32)> {
        let i = self
            .tilesets
            .partition_point(|s| s.first <= t)
            .checked_sub(1)?;
        let set = self.tilesets.get(i)?;
        let pos = t - set.first;
        (pos < set.count).then_some((set, pos + 1))
    }

    /// The file name of tile `t` (provisional, Q-0009).
    pub fn file_of(&self, t: i32) -> Option<Vec<u8>> {
        let (set, ordinal) = self.locate(t)?;
        // provisional (Q-0009): FileName ‖ two-digit ordinal ‖ . ‖ extension
        let mut name = set.file_name.clone();
        name.extend_from_slice(format!("{ordinal:02}.{}", self.extension).as_bytes());
        Some(name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_ini::Quirks;

    fn doc(text: &str) -> Document {
        Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new())
    }

    /// spec: S-0009 claim 6 (base tiles), claim 8 (identity translation, clear-tile rule)
    #[test]
    fn base_tiles_and_clear_rule() {
        let d = doc("[General]\nClearTile=1\nWaterSet=2\n\
             [TileSet0000]\nFileName=aa\nTilesInSet=3\n\
             [TileSet0001]\nFileName=bb\nTilesInSet=2\n\
             [TileSet0002]\nFileName=cc\nTilesInSet=4\n");
        let c = Catalogue::read(&d, Theater::Temperate);
        assert_eq!(c.tile_count, 9);
        assert_eq!((c.clear_base, c.water_base), (3, 5));
        assert_eq!(c.translate(7), 7);
        assert_eq!(c.art_tile(65535), Some(3));
        assert_eq!(c.art_tile(9), Some(3));
        assert_eq!(c.art_tile(-1), Some(3));
        assert_eq!(c.art_tile(8), Some(8));
        assert_eq!(c.file_of(4), Some(b"bb02.TEM".to_vec()));
        // spec: S-0009 claim 6 (theater 5: water base tile −1)
        assert_eq!(Catalogue::read(&d, Theater::Lunar).water_base, -1);
        let none = Catalogue::read(&doc("[TileSet0000]\nFileName=aa\n"), Theater::Snow);
        assert_eq!((none.clear_base, none.water_base), (-1, -1));
    }

    /// spec: S-0009 Formulas (tile number translation)
    #[test]
    fn translation() {
        // Set 0: C = 3; set 1: C = 5 but authored A = 2; set 2: C = 4.
        let d = doc(
            "[TileSet0000]\nTilesInSet=3\n[TileSet0001]\nTilesInSet=5\nLastTilesInSet=2\n\
             [TileSet0002]\nTilesInSet=4\n",
        );
        let c = Catalogue::read(&d, Theater::Temperate);
        // Authoring space: set 1 starts at 3, set 2 at 5; t_1 = 3 + 2 = 5, δ_1 = 3.
        assert_eq!(c.translation, vec![(5, 3)]);
        assert_eq!(c.translate(4), 4);
        assert_eq!(c.translate(5), 8);
        assert_eq!(c.translate(65535), 65535);
    }
}
