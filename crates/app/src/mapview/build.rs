//! Builds the [`Scene`] of a decoded map: every tile, overlay, terrain
//! object, building and unit placed in world pixels in draw order, with
//! its depth (S-0102), palette row (S-0103, S-0104) and image frame
//! (S-0107, S-0108, S-0105).
//!
//! The map's initial state is drawn: no tick has run, so buildings are in
//! their idle state at stage 0 and infantry show the *stand* entry at stage
//! 0. Choices that no approved spec makes are marked provisional with their
//! request, and listed in [`Built::notes`].

use std::collections::{BTreeMap, BTreeSet};

use glasnost_convert::Import;
use glasnost_convert::kinds::Kind;
use glasnost_convert::theater::Catalogue;
use glasnost_fmt::shp::Shp;
use glasnost_fmt::tmp::Tmp;
use glasnost_ini::Document;
use glasnost_ini::value;
use glasnost_map::{CellXY, Map, NO_OVERLAY};
use glasnost_render::compose::{DepthTest, Paint, Profile, row565};
use glasnost_render::light::{self, DetailLevel, LightQuirks, LightSource, TileTable};
use glasnost_render::palette::{PaletteState, TINT_EXEMPT, house_palette};
use glasnost_render::projection::{
    LEVEL_LEPTONS, TERRAIN_LIFT, cell_centre, cell_top, height_lift, project_lifted,
};
use glasnost_render::scene::{Group, Item, ItemDepth, Scene};

/// File bytes read for a scene, by upper-case name.
#[derive(Clone, Debug, Default)]
pub struct Assets {
    files: BTreeMap<Vec<u8>, Vec<u8>>,
}

impl Assets {
    /// Adds a file.
    pub fn insert(&mut self, name: &[u8], bytes: Vec<u8>) {
        self.files.insert(name.to_ascii_uppercase(), bytes);
    }

    /// A file by name, ignoring case.
    pub fn get(&self, name: &[u8]) -> Option<&[u8]> {
        self.files
            .get(&name.to_ascii_uppercase())
            .map(Vec::as_slice)
    }

    /// Number of files.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether no file is held.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Viewer choices for a scene (none of them is game data).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// Detail level (S-0104): building lights need 2.
    pub detail: DetailLevel,
    /// Skirmish view: drop objects whose owner no skirmish house carries
    /// (S-0009 claim 27).
    pub skirmish: bool,
    /// Lobby colour slot of the human placeholder house (S-0103 claim 10).
    pub human_slot: u8,
    /// Lobby colour slot of the computer placeholder house.
    pub computer_slot: u8,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            detail: DetailLevel::High,
            skirmish: true,
            human_slot: 0,
            computer_slot: 1,
        }
    }
}

/// What the viewer shows about one object.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectInfo {
    /// Kind label.
    pub kind: &'static str,
    /// Type name.
    pub type_name: String,
    /// Owner as written.
    pub owner: String,
    /// Cell.
    pub cell: CellXY,
    /// The map entry as written, or the overlay's stream values.
    pub entry: String,
    /// Notes (provisional choices that apply to it).
    pub notes: Vec<String>,
}

/// A built scene.
#[derive(Clone, Debug, Default)]
pub struct Built {
    /// The scene.
    pub scene: Scene,
    /// Objects, indexed by [`Item::object`].
    pub objects: Vec<ObjectInfo>,
    /// Provisional choices and gaps that this map met.
    pub notes: BTreeSet<String>,
    /// Objects dropped (unknown type, no art, or an owner no skirmish house
    /// carries).
    pub dropped: BTreeMap<String, usize>,
}

/// The fixed house names of S-0009 Import mapping (claim 27).
const NEUTRAL: &[u8] = b"Neutral"; // spec: S-0009 Import mapping (house names)
const SPECIAL: &[u8] = b"Special"; // spec: S-0009 Import mapping (house names)
const HUMAN: &[u8] = b"<human player>"; // spec: S-0009 Import mapping (house names)
const COMPUTER: &[u8] = b"Computer"; // spec: S-0009 Import mapping (house names)
/// The colour name both non-player houses show (S-0103 claim 10).
const LIGHT_GREY: &[u8] = b"LightGrey"; // spec: S-0103 Import mapping
/// Rows of a house-colour palette's shade table (S-0103 claim 5).
const HOUSE_ROWS: usize = 53; // spec: S-0103 Constants (shade counts)
/// White tint in per-mille (S-0104 claim 3).
const WHITE: [i32; 3] = [1000, 1000, 1000]; // spec: S-0104 claim 3 (house-colour palettes are shaded with white light)
/// Ore and gem overlays are drawn at brightness 1000 (S-0104 claim 13).
const NEUTRAL_BRIGHTNESS: i32 = 1000; // spec: S-0104 claim 13 (ore and gem overlays)
/// The veinhole marker overlay index (S-0009 claim 14).
const VEINHOLE_MARKER: u8 = 126; // spec: S-0009 Constants (veinhole marker overlay index)
/// Bridge deck height in levels (S-0102 claim 15).
const SITUATIONAL: i32 = -1; // spec: S-0102 claim 14 (s = −1 in the ordinary case)
/// A cliff cell is more than this many levels higher (S-0102 claim 14).
const CLIFF_LEVELS: i32 = 3; // spec: S-0102 Constants (cliff fudge test)
/// Absent-key default of the cliff fudge (S-0102 claim 14).
const DEFAULT_CLIFF_FUDGE: i32 = 10; // spec: S-0102 claim 14 (absent fudge values: cliff 10)
/// Default light radius of building lights (S-0104 claim 12).
const DEFAULT_LIGHT_RADIUS: i32 = 5000; // spec: S-0104 Constants (building light radius default)
/// Building light tint default, 1000.0 as stored (S-0104 claim 12).
const DEFAULT_LIGHT_TINT: i32 = 1_000_000; // spec: S-0104 claim 12 (tint default 1000.0, stored 1 000 000 ‰)
/// Voxel lighting table file (S-0105 Import mapping).
pub const VPL_FILE: &[u8] = b"voxels.vpl"; // spec: S-0105 Import mapping

/// The provisional notes, by request.
mod notes {
    pub const TILES: &str = "Q-0009: tile files, sub-tile cells and tile placement are provisional";
    pub const OBJECTS: &str =
        "Q-0010: object list fields, infantry positions and facings are provisional";
    pub const PLACEMENT: &str =
        "Q-0011: sprites are centred on their object's point by their canvas (provisional)";
    pub const OVERLAY: &str = "Q-0012: overlay and smudge frames and palettes are provisional";
    pub const TILE_LIGHT: &str = "Q-0013: tiles are lit by the scaled ground level (provisional)";
    pub const SHADOW: &str = "Q-0015: shadows halve the pixels beneath (provisional)";
    pub const VOXEL_SHADOW_POINT: &str = "voxel shadows are placed at the object's ground point (the locomotor's shadow point has no spec)";
    pub const NO_SLOT_ANIMS: &str = "building slot animations (S-0108) need animation playback, which has no spec yet: not drawn";
    pub const NO_COMPANION: &str =
        "building depth images (S-0102 F5): the shared file is not named by a spec: not used";
    pub const HEALTH: &str =
        "Q-0010: the health scale is unknown, so every building is drawn undamaged";
    pub const WATER_FILL: &str = "water-fill variants need the scenario random stream (no spec yet): the base water tile is drawn";
}

fn upper(name: &[u8]) -> Vec<u8> {
    name.to_ascii_uppercase()
}

/// The registration index of `name` in `kind` (names compared as S-0003
/// compares type names).
pub fn find_type(import: &Import, kind: Kind, name: &[u8]) -> Option<usize> {
    import
        .outcome
        .registries
        .get(&kind)?
        .iter()
        .position(|t| value::type_name_eq(&t.name, name))
}

/// A rules key of a type, as layered.
fn rule<'a>(import: &'a Import, kind: Kind, idx: usize, key: &[u8]) -> Option<&'a [u8]> {
    import
        .outcome
        .registries
        .get(&kind)?
        .get(idx)?
        .keys
        .get(key)?
        .value
        .as_deref()
}

fn rule_bool(import: &Import, kind: Kind, idx: usize, key: &[u8], default: bool) -> bool {
    rule(import, kind, idx, key)
        .and_then(value::boolean)
        .unwrap_or(default)
}

fn rule_int(import: &Import, kind: Kind, idx: usize, key: &[u8], default: i32) -> i32 {
    rule(import, kind, idx, key)
        .and_then(value::integer)
        .unwrap_or(default)
}

fn derived(import: &Import, kind: Kind, idx: usize, key: &[u8]) -> Option<i64> {
    import
        .outcome
        .registries
        .get(&kind)?
        .get(idx)?
        .keys
        .get(key)?
        .derived
}

/// Every file the scene of `map` may read: tile files, sprites, voxel
/// models with their motion files, and the voxel lighting table.
pub fn wanted_files(map: &Map, import: &Import, catalogue: &Catalogue) -> BTreeSet<Vec<u8>> {
    let mut out = BTreeSet::new();
    for (_, cell) in map.cells.iter() {
        if let Some(t) = catalogue.art_tile(cell.tile)
            && let Some(f) = catalogue.file_of(t)
        {
            out.insert(upper(&f));
        }
    }
    let mut add_art = |kind: Kind, idx: usize| {
        if let Some(a) = import.arts.get(&(kind, idx)) {
            out.extend(a.sprite.iter().map(|s| upper(s)));
            for v in a.voxels.values() {
                out.insert(upper(v));
                out.insert(upper(&hva_name(v)));
            }
            out.extend(a.building_sprites.values().map(|s| upper(s)));
        }
    };
    let used_overlays: BTreeSet<u8> = map.cells.iter().map(|(_, c)| c.overlay).collect();
    for o in used_overlays {
        if o != NO_OVERLAY {
            add_art(Kind::Overlays, usize::from(o));
        }
    }
    let objects = &map.objects;
    let names = objects
        .units
        .iter()
        .map(|u| (Kind::Vehicles, &u.placed.type_name))
        .chain(
            objects
                .infantry
                .iter()
                .map(|u| (Kind::Infantry, &u.placed.type_name)),
        )
        .chain(
            objects
                .aircraft
                .iter()
                .map(|u| (Kind::Aircraft, &u.placed.type_name)),
        )
        .chain(
            objects
                .structures
                .iter()
                .map(|u| (Kind::Buildings, &u.placed.type_name)),
        )
        .chain(
            objects
                .terrain
                .iter()
                .map(|t| (Kind::TerrainObjects, &t.type_name)),
        )
        .chain(
            objects
                .smudges
                .iter()
                .map(|t| (Kind::Smudges, &t.type_name)),
        );
    for (kind, name) in names {
        if let Some(i) = find_type(import, kind, name) {
            add_art(kind, i);
        }
    }
    out.insert(upper(VPL_FILE));
    out
}

/// The motion file of a model file (S-0105 glossary *voxel model*: same
/// base name).
pub fn hva_name(vxl: &[u8]) -> Vec<u8> {
    let base = vxl
        .iter()
        .rposition(|&b| b == b'.')
        .map_or(vxl, |i| &vxl[..i]);
    let mut n = base.to_vec();
    n.extend_from_slice(b".hva");
    n
}

/// Per-cell light (S-0104 claim 9).
struct Lights {
    side: i32,
    cells: Vec<Option<light::CellLight>>,
}

impl Lights {
    fn get(&self, c: CellXY) -> Option<&light::CellLight> {
        if c.x < 0 || c.y < 0 || c.x >= self.side || c.y >= self.side {
            return None;
        }
        self.cells[(c.y * self.side + c.x) as usize].as_ref()
    }
}

/// Palette rows added to the scene, by table.
#[derive(Default)]
struct Rows {
    tiles: BTreeMap<TileTable, (usize, usize)>,
    houses: BTreeMap<[u8; 3], (usize, usize)>,
}

struct Builder<'a> {
    map: &'a Map,
    import: &'a Import,
    catalogue: &'a Catalogue,
    palettes: &'a PaletteState,
    assets: &'a Assets,
    art: Document,
    options: &'a Options,
    lights: Lights,
    rows: Rows,
    built: Built,
    images: BTreeMap<(Vec<u8>, usize, u8), Option<u32>>,
}

/// A decoded tile cell: diamond image, its depth data and template
/// position, and the extra image with its depth and offset from the
/// diamond.
type TileImages = (
    u32,
    Option<u32>,
    i32,
    i32,
    Option<(u32, Option<u32>, i32, i32)>,
);

/// A sprite frame: image index, the frame's offset on the canvas, and the
/// canvas size.
type SpriteFrame = (u32, (i32, i32), (i32, i32));

/// One entry of the object pass's ground layer (S-0102 claims 5 and 6).
struct GroundEntry {
    key: i64,
    items: Vec<Item>,
}

/// Builds the scene of `map`.
#[allow(clippy::too_many_arguments)]
pub fn build(
    map: &Map,
    import: &Import,
    catalogue: &Catalogue,
    palettes: &PaletteState,
    assets: &Assets,
    options: &Options,
) -> Built {
    let art = match &import.art_file {
        Some(f) => Document::parse(&f.bytes, Default::default(), &mut Vec::new()),
        None => Document::empty(Default::default()),
    };
    let mut b = Builder {
        map,
        import,
        catalogue,
        palettes,
        assets,
        art,
        options,
        lights: Lights {
            side: 0,
            cells: Vec::new(),
        },
        rows: Rows::default(),
        built: Built::default(),
        images: BTreeMap::new(),
    };
    b.light_cells();
    b.tiles();
    b.overlays();
    b.smudges();
    b.terrain_objects();
    let mut ground = Vec::new();
    b.structures(&mut ground);
    b.built.scene.cell_pass_len = b.built.scene.items.len();
    b.mobiles(&mut ground);
    // spec: S-0102 claim 6 (an entering object goes before the first strictly greater key: a stable sort)
    ground.sort_by_key(|g| g.key);
    for g in ground {
        b.built.scene.items.extend(g.items);
    }
    b.built.notes.insert(notes::NO_SLOT_ANIMS.into());
    b.built
}

impl Builder<'_> {
    fn note(&mut self, n: &str) {
        self.built.notes.insert(n.to_owned());
    }

    fn drop_object(&mut self, why: &str) {
        *self.built.dropped.entry(why.to_owned()).or_default() += 1;
    }

    /// Full recompute of every cell's light (S-0104 claim 9) with the
    /// building light sources (claim 12).
    fn light_cells(&mut self) {
        let l = self.map.lighting.normal;
        let mut sources = Vec::new();
        for s in &self.map.objects.structures {
            let Some(i) = find_type(self.import, Kind::Buildings, &s.placed.type_name) else {
                continue;
            };
            if !self.owner_kept(&s.placed.owner) {
                continue;
            }
            let intensity =
                derived(self.import, Kind::Buildings, i, b"LightIntensity").unwrap_or(0);
            if intensity == 0 {
                continue;
            }
            let (w, h) = self.foundation(i);
            let tint = |k: &[u8]| {
                derived(self.import, Kind::Buildings, i, k).map_or(DEFAULT_LIGHT_TINT, |v| v as i32)
            };
            let c = s.placed.cell;
            // spec: S-0104 claim 12 (one light at the building's centre, counting at detail 2)
            sources.push(LightSource::building(
                (c.x, c.y),
                (w, h),
                rule_int(
                    self.import,
                    Kind::Buildings,
                    i,
                    b"LightVisibility",
                    DEFAULT_LIGHT_RADIUS,
                ),
                intensity as i32,
                [
                    tint(b"LightRedTint"),
                    tint(b"LightGreenTint"),
                    tint(b"LightBlueTint"),
                ],
            ));
        }
        let side = i32::from(self.map.cells.footprint().coordinate_side());
        let mut cells = vec![None; (side * side) as usize];
        let q = LightQuirks::default();
        // spec: S-0104 claim 1 (the map's normal set; current, base and target ambient start equal; tint in hundredths)
        let tint = l.tint;
        for (c, cell) in self.map.cells.iter() {
            cells[(c.y * side + c.x) as usize] = Some(light::full_recompute(
                l.ambient,
                l.level,
                l.ground,
                tint,
                (c.x, c.y),
                i32::from(cell.level),
                &sources,
                self.options.detail,
                &q,
            ));
        }
        self.lights = Lights { side, cells };
    }

    fn tile_row(&mut self, table: TileTable, brightness: i32) -> usize {
        let (base, n) = match self.rows.tiles.get(&table) {
            Some(r) => *r,
            None => {
                // spec: S-0103 claim 8 (terrain tiles use the tile palette, shaded by their cell's table)
                let t = light::shade_table(&self.palettes.tile, table.tint(), table.rows(), None);
                let base = self.built.scene.rows.len();
                for row in &t {
                    self.built.scene.add_row(row565(row));
                }
                self.rows.tiles.insert(table, (base, t.len()));
                (base, t.len())
            }
        };
        base + light::brightness_row(brightness, n, light::DEFAULT_LIGHT_ALPHA)
    }

    fn house_row(&mut self, hsv: [u8; 3], brightness: i32) -> usize {
        let (base, n) = match self.rows.houses.get(&hsv) {
            Some(r) => *r,
            None => {
                let pal = house_palette(&self.palettes.unit, hsv);
                // spec: S-0103 claim 7 (indices 240 … 254 are tint-exempt in house-colour palettes)
                let t = light::shade_table(&pal, WHITE, HOUSE_ROWS, Some(TINT_EXEMPT));
                let base = self.built.scene.rows.len();
                for row in &t {
                    self.built.scene.add_row(row565(row));
                }
                self.rows.houses.insert(hsv, (base, t.len()));
                (base, t.len())
            }
        };
        base + light::brightness_row(brightness, n, light::DEFAULT_LIGHT_ALPHA)
    }

    /// The colour definition an owner shows (S-0103 claim 10), and whether
    /// the choice is ours (an owner no skirmish house carries).
    fn owner_colour(&self, owner: &[u8]) -> ([u8; 3], bool) {
        let colours = &self.import.outcome.colours;
        let by_name = |n: &[u8]| {
            colours
                .iter()
                .find(|c| c.name.eq_ignore_ascii_case(n))
                .map(|c| c.hsv)
        };
        let by_slot = |s: u8| {
            glasnost_render::palette::slot_position(s)
                .and_then(|p| colours.get(p))
                .map(|c| c.hsv)
        };
        let grey = by_name(LIGHT_GREY).unwrap_or([0, 0, 200]);
        if owner == NEUTRAL || owner == SPECIAL {
            (grey, false)
        } else if owner == HUMAN {
            (by_slot(self.options.human_slot).unwrap_or(grey), false)
        } else if owner == COMPUTER {
            (by_slot(self.options.computer_slot).unwrap_or(grey), false)
        } else {
            (grey, true)
        }
    }

    /// Whether an owner's objects appear (S-0009 claim 27, in the skirmish
    /// view).
    fn owner_kept(&self, owner: &[u8]) -> bool {
        // spec: S-0009 claim 27 (only the neutral and special houses and the player placeholders match)
        !self.options.skirmish || [NEUTRAL, SPECIAL, HUMAN, COMPUTER].contains(&owner)
    }

    /// An image of a sprite file's frame, decoded once, with its frame
    /// rectangle's offset on the canvas and the canvas size.
    fn sprite_frame(&mut self, file: &[u8], frame: usize) -> Option<SpriteFrame> {
        let bytes = self.assets.get(file)?;
        let shp = Shp::parse(bytes).ok()?;
        let f = *shp.frames.get(frame)?;
        let canvas = (i32::from(shp.width), i32::from(shp.height));
        let key = (upper(file), frame, 0u8);
        let id = match self.images.get(&key) {
            Some(id) => *id,
            None => {
                let id = shp
                    .decode_frame(frame)
                    .ok()
                    .filter(|i| i.width() > 0 && i.height() > 0)
                    .map(|i| self.built.scene.add_image(i));
                self.images.insert(key, id);
                id
            }
        };
        Some((id?, (i32::from(f.x), i32::from(f.y)), canvas))
    }

    fn sprite_frames(&self, file: &[u8]) -> Option<usize> {
        Shp::parse(self.assets.get(file)?)
            .ok()
            .map(|s| s.frames.len())
    }

    /// Places a sprite frame with its canvas centred on world pixel `p`
    /// (provisional, Q-0011; S-0108 claim 8 for buildings).
    #[allow(clippy::too_many_arguments)]
    fn place_sprite(
        &mut self,
        file: &[u8],
        frame: usize,
        p: (i32, i32),
        test: DepthTest,
        depth: ItemDepth,
        paint: Paint,
        group: Group,
        provisional: bool,
        object: Option<u32>,
    ) -> Option<Item> {
        let (image, (fx, fy), (cw, ch)) = self.sprite_frame(file, frame)?;
        Some(Item {
            image,
            x: p.0 - cw / 2 + fx,
            y: p.1 - ch / 2 + fy,
            test,
            depth,
            paint,
            group,
            provisional,
            object,
        })
    }

    fn add_object(&mut self, info: ObjectInfo) -> u32 {
        self.built.objects.push(info);
        (self.built.objects.len() - 1) as u32
    }

    /// Terrain tiles (S-0101 claim 4, S-0102 claim 9 and F3; Q-0009).
    fn tiles(&mut self) {
        self.note(notes::TILES);
        self.note(notes::TILE_LIGHT);
        if self.map.fill_draws > 0 {
            self.note(notes::WATER_FILL);
        }
        let mut tmps: BTreeMap<Vec<u8>, Option<Vec<Option<TileImages>>>> = BTreeMap::new();
        let cells: Vec<(CellXY, glasnost_map::Cell)> = self
            .map
            .cells
            .visiting_order(true)
            .into_iter()
            .filter_map(|c| Some((c, *self.map.cells.get(c)?)))
            .collect();
        // Cells the visiting order misses (W = 1) keep their blank state; draw them too.
        let mut seen: BTreeSet<CellXY> = cells.iter().map(|(c, _)| *c).collect();
        let mut all = cells;
        for (c, cell) in self.map.cells.iter() {
            if seen.insert(c) {
                all.push((c, *cell));
            }
        }
        for (c, cell) in all {
            let Some(t) = self.catalogue.art_tile(cell.tile) else {
                continue;
            };
            let Some(file) = self.catalogue.file_of(t).map(|f| upper(&f)) else {
                continue;
            };
            if !tmps.contains_key(&file) {
                let decoded = self.assets.get(&file).and_then(|bytes| {
                    let tmp = Tmp::parse(bytes).ok()?;
                    let mut out = Vec::new();
                    for i in 0..tmp.tiles.len() {
                        out.push(tmp.decode_tile(i).ok().map(|d| {
                            let dia = &d.diamond;
                            let img = self.built.scene.add_image(dia.image.clone());
                            let dep = dia.depth.clone().map(|i| self.built.scene.add_image(i));
                            let extra = d.extra.as_ref().map(|e| {
                                let ei = self.built.scene.add_image(e.image.clone());
                                let ed = e.depth.clone().map(|i| self.built.scene.add_image(i));
                                (ei, ed, e.x - dia.x, e.y - dia.y)
                            });
                            (img, dep, dia.x, dia.y, extra)
                        }));
                    }
                    Some(out)
                });
                tmps.insert(file.clone(), decoded);
            }
            // provisional (Q-0009): sub-tile n is the n-th template cell in row-major order
            let Some(Some(Some((img, dep, _, _, extra)))) = tmps.get(&file).map(|v| {
                v.as_ref()
                    .map(|v| v.get(usize::from(cell.sub_tile)).copied().flatten())
            }) else {
                continue;
            };
            let level = i32::from(cell.level);
            let (qx, qy) = cell_top(c.x, c.y);
            // spec: S-0101 claim 4 (terrain is lifted 15 pixels per height level)
            let top = qy - TERRAIN_LIFT * level;
            let w = self.built.scene.images[img as usize].width() as i32;
            let left = qx - w / 2;
            // spec: S-0102 F3 (t = the tile's top row as drawn plus 15 × its height level)
            let reference_row = top + TERRAIN_LIFT * level;
            let (table, b) = match self.lights.get(c) {
                // provisional (Q-0013): the scaled ground level
                Some(l) => (l.table, l.scaled_ground),
                None => (TileTable::Neutral, NEUTRAL_BRIGHTNESS),
            };
            let row = self.tile_row(table, b);
            let mut push = |image: u32, depth: Option<u32>, x: i32, y: i32| {
                self.built.scene.items.push(Item {
                    image,
                    x,
                    y,
                    test: DepthTest::Tile,
                    depth: ItemDepth::Tile {
                        depth,
                        reference_row,
                    },
                    paint: Paint::Row(row),
                    group: Group::Tiles,
                    provisional: false,
                    object: None,
                });
            };
            push(img, dep, left, top);
            if let Some((ei, ed, dx, dy)) = extra {
                push(ei, ed, left + dx, top + dy);
            }
        }
    }

    /// Overlays (S-0009 claim 12, S-0102 claims 12 and 13, S-0104 claim 13;
    /// frames and palettes provisional, Q-0012).
    fn overlays(&mut self) {
        let list_len = self
            .import
            .outcome
            .registries
            .get(&Kind::Overlays)
            .map_or(0, Vec::len);
        let cells: Vec<(CellXY, glasnost_map::Cell)> =
            self.map.cells.iter().map(|(c, cell)| (c, *cell)).collect();
        let mut beyond = 0usize;
        for (c, cell) in cells {
            let o = cell.overlay;
            // spec: S-0009 claim 14 (veinhole markers are cleared)
            if o == NO_OVERLAY || o == VEINHOLE_MARKER {
                continue;
            }
            let idx = usize::from(o);
            if idx >= list_len {
                // spec: S-0009 Edge cases (an index beyond the list: no overlay, and the importer warns)
                beyond += 1;
                continue;
            }
            // spec: S-0009 claim 12 (crate overlays never appear in skirmish or multiplayer)
            if self.options.skirmish && rule_bool(self.import, Kind::Overlays, idx, b"Crate", false)
            {
                continue;
            }
            let Some(sprite) = self
                .import
                .arts
                .get(&(Kind::Overlays, idx))
                .and_then(|a| a.sprite.clone())
            else {
                // spec: S-0009 claim 12 (an overlay without artwork or cell animation is not created)
                continue;
            };
            let k = (Kind::Overlays, idx);
            let tiberium = rule_bool(self.import, k.0, idx, b"Tiberium", false);
            let wall = rule_bool(self.import, k.0, idx, b"Wall", false);
            let rock = rule_bool(self.import, k.0, idx, b"IsARock", false);
            // spec: S-0102 claim 12 (drawn flat unless the type says otherwise; walls always upright)
            let flat = !wall && (tiberium || rule_bool(self.import, k.0, idx, b"DrawFlat", true));
            let level = i32::from(cell.level);
            // spec: S-0102 claim 13 (overlay biases: −2 − 15ℓ, or −17 − 15ℓ for other upright types)
            let bias = if flat || wall || rock || tiberium {
                -2 - TERRAIN_LIFT * level
            } else {
                -17 - TERRAIN_LIFT * level
            };
            let light = self.lights.get(c);
            // spec: S-0104 claim 13 (ore and gems 1000; walls U; others the scaled ground level)
            let b = if tiberium {
                NEUTRAL_BRIGHTNESS
            } else if wall {
                light.map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground)
            } else {
                light.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground)
            };
            let table = light.map_or(TileTable::Neutral, |l| l.table);
            // provisional (Q-0012): overlays use their cell's tile shade table
            let row = self.tile_row(table, b);
            let frames = self.sprite_frames(&sprite).unwrap_or(0);
            // provisional (Q-0012): ore shows the frame of its data value, other overlays frame 0
            let frame = if tiberium && usize::from(cell.overlay_data) < frames / 2 {
                usize::from(cell.overlay_data)
            } else {
                0
            };
            let (qx, qy) = cell_top(c.x, c.y);
            // provisional (Q-0011): centred on the cell centre; spec: S-0101 claim 11 (lift 15 per level)
            let p = (qx, qy + 15 - TERRAIN_LIFT * level);
            let name = self.import.outcome.registries[&Kind::Overlays][idx]
                .name
                .clone();
            let object = self.add_object(ObjectInfo {
                kind: "overlay",
                type_name: String::from_utf8_lossy(&name).into_owned(),
                owner: String::new(),
                cell: c,
                entry: format!("type index {o}, data {}", cell.overlay_data),
                notes: vec![notes::OVERLAY.into(), notes::PLACEMENT.into()],
            });
            if let Some(item) = self.place_sprite(
                &sprite,
                frame,
                p,
                DepthTest::Writer,
                ItemDepth::Profile {
                    profile: if flat {
                        Profile::Flat
                    } else {
                        Profile::Upright
                    },
                    bias,
                    companion: None,
                },
                Paint::Row(row),
                Group::Overlays,
                true,
                Some(object),
            ) {
                self.built.scene.items.push(item);
                self.note(notes::OVERLAY);
                self.note(notes::PLACEMENT);
            }
        }
        if beyond > 0 {
            *self
                .built
                .dropped
                .entry("overlay index beyond the overlay list".into())
                .or_default() += beyond;
        }
    }

    /// Smudges (provisional throughout: no spec draws them yet).
    fn smudges(&mut self) {
        let smudges = self.map.objects.smudges.clone();
        for s in smudges {
            let Some(i) = find_type(self.import, Kind::Smudges, &s.type_name) else {
                self.drop_object("unknown smudge type");
                continue;
            };
            let Some(sprite) = self
                .import
                .arts
                .get(&(Kind::Smudges, i))
                .and_then(|a| a.sprite.clone())
            else {
                self.drop_object("smudge without art");
                continue;
            };
            let Some(cell) = self.map.cells.get(s.cell).copied() else {
                continue;
            };
            let level = i32::from(cell.level);
            let light = self.lights.get(s.cell);
            let row = self.tile_row(
                light.map_or(TileTable::Neutral, |l| l.table),
                light.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground),
            );
            let (qx, qy) = cell_top(s.cell.x, s.cell.y);
            let object = self.add_object(ObjectInfo {
                kind: "smudge",
                type_name: String::from_utf8_lossy(&s.type_name).into_owned(),
                owner: String::new(),
                cell: s.cell,
                entry: String::new(),
                notes: vec![notes::OVERLAY.into(), notes::OBJECTS.into()],
            });
            if let Some(item) = self.place_sprite(
                &sprite,
                0,
                (qx, qy + 15 - TERRAIN_LIFT * level),
                DepthTest::Writer,
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: -2 - TERRAIN_LIFT * level,
                    companion: None,
                },
                Paint::Row(row),
                Group::Smudges,
                true,
                Some(object),
            ) {
                self.built.scene.items.push(item);
            }
        }
    }

    /// Terrain objects (S-0102 claim 13 rows, S-0104 claim 13; frame and
    /// placement provisional, Q-0011).
    fn terrain_objects(&mut self) {
        let list = self.map.objects.terrain.clone();
        for t in list {
            let Some(i) = find_type(self.import, Kind::TerrainObjects, &t.type_name) else {
                self.drop_object("unknown terrain object type");
                continue;
            };
            let Some(sprite) = self
                .import
                .arts
                .get(&(Kind::TerrainObjects, i))
                .and_then(|a| a.sprite.clone())
            else {
                self.drop_object("terrain object without art");
                continue;
            };
            let Some(cell) = self.map.cells.get(t.cell).copied() else {
                continue;
            };
            let z = LEVEL_LEPTONS * i32::from(cell.level);
            let light = self.lights.get(t.cell);
            // spec: S-0104 claim 13 (a plain terrain object: scaled ground level, its cell's tile table)
            let row = self.tile_row(
                light.map_or(TileTable::Neutral, |l| l.table),
                light.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground),
            );
            let (x, y) = cell_centre(t.cell.x, t.cell.y);
            let p = project_lifted(x, y, z);
            let frames = self.sprite_frames(&sprite).unwrap_or(0);
            let object = self.add_object(ObjectInfo {
                kind: "terrain object",
                type_name: String::from_utf8_lossy(&t.type_name).into_owned(),
                owner: String::new(),
                cell: t.cell,
                entry: String::new(),
                notes: vec![notes::PLACEMENT.into()],
            });
            let lz = height_lift(z);
            // spec: S-0102 claim 13 (terrain object: −L(z) − 12, upright, writes)
            if let Some(item) = self.place_sprite(
                &sprite,
                0,
                p,
                DepthTest::Writer,
                ItemDepth::Profile {
                    profile: Profile::Upright,
                    bias: -lz - 12,
                    companion: None,
                },
                Paint::Row(row),
                Group::TerrainObjects,
                true,
                Some(object),
            ) {
                self.built.scene.items.push(item);
                self.note(notes::PLACEMENT);
            }
            // spec: S-0102 claim 13 (its shadow: −L(z) − 3, flat, writes); provisional frame (Q-0011)
            if frames > 1
                && let Some(item) = self.place_sprite(
                    &sprite,
                    frames / 2,
                    p,
                    DepthTest::Writer,
                    ItemDepth::Profile {
                        profile: Profile::Flat,
                        bias: -lz - 3,
                        companion: None,
                    },
                    Paint::Shadow,
                    Group::Shadows,
                    true,
                    Some(object),
                )
            {
                self.built.scene.items.push(item);
                self.note(notes::SHADOW);
            }
        }
    }

    /// The foundation W × H of a building (S-0108 Import mapping).
    fn foundation(&self, idx: usize) -> (i32, i32) {
        const WORDS: [(&str, (i32, i32)); 22] = [
            ("1x1", (1, 1)),
            ("2x1", (2, 1)),
            ("1x2", (1, 2)),
            ("2x2", (2, 2)),
            ("2x3", (2, 3)),
            ("3x2", (3, 2)),
            ("3x3", (3, 3)),
            ("3x5", (3, 5)),
            ("4x2", (4, 2)),
            ("3x3Refinery", (3, 3)),
            ("1x3", (1, 3)),
            ("3x1", (3, 1)),
            ("4x3", (4, 3)),
            ("1x4", (1, 4)),
            ("1x5", (1, 5)),
            ("2x6", (2, 6)),
            ("2x5", (2, 5)),
            ("5x3", (5, 3)),
            ("4x4", (4, 4)),
            ("3x4", (3, 4)),
            ("6x4", (6, 4)),
            ("0x0", (0, 0)),
        ]; // spec: S-0108 Import mapping (foundation vocabulary)
        let read = |section: &[u8]| {
            let v = self.art.value(section, b"Foundation")?;
            // spec: S-0108 Import mapping (text cut to 31 characters, compared ignoring case)
            let v = value::text(v, 31);
            Some(
                WORDS
                    .iter()
                    .find(|(w, _)| w.as_bytes().eq_ignore_ascii_case(v))
                    .map_or((1, 1), |(_, s)| *s),
            )
        };
        let name = &self.import.outcome.registries[&Kind::Buildings][idx].name;
        let image = self
            .import
            .arts
            .get(&(Kind::Buildings, idx))
            .map_or(name.clone(), |a| a.image.clone());
        // spec: S-0108 Import mapping (the id's section decides when it gives a size other than 1 × 1)
        match read(name) {
            Some(s) if s != (1, 1) => s,
            _ => read(&image).unwrap_or((1, 1)),
        }
    }

    fn art_value(&self, kind: Kind, idx: usize, key: &[u8]) -> Option<&[u8]> {
        let image = self.import.arts.get(&(kind, idx))?.image.clone();
        let doc: &Document = &self.art;
        doc.value(&image, key)
    }

    /// Buildings in the cell pass (S-0108, S-0102 claims 13 and 16).
    fn structures(&mut self, ground: &mut Vec<GroundEntry>) {
        let list = self.map.objects.structures.clone();
        let mut any = false;
        for s in list {
            let p = &s.placed;
            if !self.owner_kept(&p.owner) {
                self.drop_object("owner no skirmish house carries");
                continue;
            }
            let Some(i) = find_type(self.import, Kind::Buildings, &p.type_name) else {
                self.drop_object("unknown building type");
                continue;
            };
            let Some(art) = self.import.arts.get(&(Kind::Buildings, i)).cloned() else {
                self.drop_object("building without art");
                continue;
            };
            let Some(cell) = self.map.cells.get(p.cell).copied() else {
                self.drop_object("building on a non-legal cell");
                continue;
            };
            any = true;
            let (hsv, ours) = self.owner_colour(&p.owner);
            let object = self.add_object(ObjectInfo {
                kind: "building",
                type_name: String::from_utf8_lossy(&p.type_name).into_owned(),
                owner: String::from_utf8_lossy(&p.owner).into_owned(),
                cell: p.cell,
                entry: p
                    .raw
                    .iter()
                    .map(|f| String::from_utf8_lossy(f).into_owned())
                    .collect::<Vec<_>>()
                    .join(","),
                notes: vec![notes::HEALTH.into(), notes::PLACEMENT.into()],
            });
            let z = LEVEL_LEPTONS * i32::from(cell.level);
            let lz = height_lift(z);
            // spec: S-0108 claim 8 (centred on the foundation's top corner, 128 leptons before the top-left cell's centre)
            let anchor = project_lifted(256 * p.cell.x, 256 * p.cell.y, z);
            let u = self
                .lights
                .get(p.cell)
                .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground);
            let terrain_palette = self
                .art_value(Kind::Buildings, i, b"TerrainPalette")
                .and_then(value::boolean)
                .unwrap_or(false);
            let row = if terrain_palette {
                // spec: S-0103 claim 8, S-0104 claim 13 (terrain-palette building: the centre cell's table and scaled ground)
                let (w, h) = self.foundation(i);
                let (cx, cy) = light::building_centre_cell((p.cell.x, p.cell.y), (w, h));
                let centre = CellXY::new(cx, cy);
                let l = self.lights.get(centre);
                self.tile_row(
                    l.map_or(TileTable::Neutral, |l| l.table),
                    l.map_or(NEUTRAL_BRIGHTNESS, |l| l.scaled_ground),
                )
            } else {
                // spec: S-0104 claim 13 (an ordinary building: U of its top-left foundation cell)
                self.house_row(hsv, u)
            };
            let _ = ours;
            let Some(sprite) = art.sprite.clone() else {
                continue;
            };
            let n = self.sprite_frames(&sprite).unwrap_or(0);
            // spec: S-0108 claim 6 (game set-up: immediate change to idle; stage = a of the idle range)
            let idle = self
                .art_value(Kind::Buildings, i, b"AnimIdle")
                .map(|v| value::int_tuple_or::<3>(v, [0, 1, 0]).0)
                .unwrap_or([0, 1, 0]);
            let k = idle[0].max(0) as usize;
            // spec: S-0108 claim 2 (occupiable: 2·[occupied] + [hurt] = 0; gate: 0; others: k when not damaged)
            let occupiable = rule_bool(self.import, Kind::Buildings, i, b"CanBeOccupied", false);
            let gate = rule_bool(self.import, Kind::Buildings, i, b"Gate", false);
            let f = if occupiable || gate { 0 } else { k };
            self.note(notes::HEALTH);
            // spec: S-0108 claim 3 (drawn frame min(f, ⌊N/2⌋))
            let body_frame = f.min(n / 2);
            let normal_z = self
                .art_value(Kind::Buildings, i, b"NormalZAdjust")
                .and_then(value::integer)
                .unwrap_or(0);
            // spec: S-0102 claim 16 (each building's group: body, then shadow, then bib)
            // spec: S-0102 claim 13 (body: −L(z) − 2 + normal depth adjustment, upright, writes)
            if let Some(item) = self.place_sprite(
                &sprite,
                body_frame,
                anchor,
                DepthTest::Writer,
                ItemDepth::Profile {
                    profile: Profile::Upright,
                    bias: -lz - 2 + normal_z,
                    companion: None,
                },
                Paint::Row(row),
                Group::Buildings,
                true,
                Some(object),
            ) {
                self.built.scene.items.push(item);
            }
            // spec: S-0108 claim 9 (shadow frame F + ⌊N/2⌋ unless the type suppresses shadows; none past the sprite)
            let shadow = body_frame + n / 2;
            if !rule_bool(self.import, Kind::Buildings, i, b"NoShadow", false)
                && shadow < n
                && let Some(item) = self.place_sprite(
                    &sprite,
                    shadow,
                    anchor,
                    DepthTest::Writer,
                    // spec: S-0102 claim 13 (building shadow: −L(z) − 4, flat, writes)
                    ItemDepth::Profile {
                        profile: Profile::Flat,
                        bias: -lz - 4,
                        companion: None,
                    },
                    Paint::Shadow,
                    Group::Shadows,
                    true,
                    Some(object),
                )
            {
                self.built.scene.items.push(item);
                self.note(notes::SHADOW);
            }
            // spec: S-0108 claim 7 (bib: same frame f, no clamp, outside the construction state)
            if let Some(bib) = art.building_sprites.get("bib").cloned()
                && let Some(item) = self.place_sprite(
                    &bib,
                    f,
                    anchor,
                    DepthTest::Writer,
                    // spec: S-0102 claim 13 (bib: −L(z) − 3, flat, writes)
                    ItemDepth::Profile {
                        profile: Profile::Flat,
                        bias: -lz - 3,
                        companion: None,
                    },
                    Paint::Row(row),
                    Group::Bibs,
                    true,
                    Some(object),
                )
            {
                self.built.scene.items.push(item);
            }
            self.note(notes::PLACEMENT);
            // Voxel turret or barrel in the object pass (S-0105 claims 5, 9, 37; S-0102 claims 5, 13).
            let turret_voxel =
                rule_bool(self.import, Kind::Buildings, i, b"TurretAnimIsVoxel", false);
            if turret_voxel || art.voxels.contains_key("barrel") {
                let items = self.building_voxels(i, &art, p, anchor, lz, u, hsv, object);
                // spec: S-0102 F4 (key = 256·(c_x + c_y) + 32 for a voxel turret − 16 for a gate)
                let key = i64::from(256 * (p.cell.x + p.cell.y))
                    + if turret_voxel { 32 } else { 0 }
                    - if gate { 16 } else { 0 };
                if !items.is_empty() {
                    ground.push(GroundEntry { key, items });
                }
            }
        }
        if any {
            self.note(notes::NO_COMPANION);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn building_voxels(
        &mut self,
        idx: usize,
        art: &glasnost_convert::art::TypeArt,
        p: &glasnost_map::objects::Placed,
        anchor: (i32, i32),
        lz: i32,
        brightness: i32,
        hsv: [u8; 3],
        object: u32,
    ) -> Vec<Item> {
        let _ = (idx, art, p, anchor, lz, brightness, hsv, object);
        Vec::new()
    }

    /// Vehicles, infantry and aircraft in the ground layer (S-0102 claims 3
    /// and 5): on the ground, every one of them is in the ground layer.
    fn mobiles(&mut self, ground: &mut Vec<GroundEntry>) {
        self.note(notes::OBJECTS);
        let units = self.map.objects.units.clone();
        for u in &units {
            if let Some(g) = self.vehicle(Kind::Vehicles, &u.placed) {
                ground.push(g);
            }
        }
        let infantry = self.map.objects.infantry.clone();
        for i in &infantry {
            if let Some(g) = self.infantry(i) {
                ground.push(g);
            }
        }
        let aircraft = self.map.objects.aircraft.clone();
        for a in &aircraft {
            if let Some(g) = self.vehicle(Kind::Aircraft, &a.placed) {
                ground.push(g);
            }
        }
    }

    /// Common checks for a placed mobile: owner, type, art and cell.
    fn mobile_start(
        &mut self,
        kind: Kind,
        p: &glasnost_map::objects::Placed,
    ) -> Option<(usize, glasnost_convert::art::TypeArt, glasnost_map::Cell)> {
        if !self.owner_kept(&p.owner) {
            self.drop_object("owner no skirmish house carries");
            return None;
        }
        let Some(i) = find_type(self.import, kind, &p.type_name) else {
            self.drop_object("unknown unit type");
            return None;
        };
        let Some(art) = self.import.arts.get(&(kind, i)).cloned() else {
            self.drop_object("unit without art");
            return None;
        };
        let Some(cell) = self.map.cells.get(p.cell).copied() else {
            self.drop_object("unit on a non-legal cell");
            return None;
        };
        Some((i, art, cell))
    }

    /// The unit depth adjustment *U* = −L(z) + s + F (S-0102 claim 14) for a
    /// unit of `kind`/`idx` standing on cell `c` at world height `z`. Only
    /// the cliff fudge can be judged: bridges, columns and tunnels need
    /// specs not written yet, so their factors are 0.
    fn unit_depth(&self, kind: Kind, idx: usize, c: CellXY, z: i32) -> i32 {
        let level = |dx: i32| {
            self.map
                .cells
                .get(CellXY::new(c.x + dx, c.y + dx))
                .map(|cell| i32::from(cell.level))
        };
        let own = level(0).unwrap_or(0);
        // spec: S-0102 claim 14 (a cell is high when more than 3 levels above the unit's cell)
        let high = |dx: i32| level(dx).is_some_and(|l| l - own > CLIFF_LEVELS);
        // spec: S-0102 claim 14 (cliff factor: two-step high 1, else one-step high 2, else 0)
        let factor = if high(2) {
            1
        } else if high(1) {
            2
        } else {
            0
        };
        let fudge = rule_int(self.import, kind, idx, b"ZFudgeCliff", DEFAULT_CLIFF_FUDGE);
        // spec: S-0102 claim 14 (U = −L(z) + s + F, F the largest fudge)
        -height_lift(z) + SITUATIONAL + (factor * fudge).max(0)
    }

    /// The rules' extra-light value in ‰ (S-0104 claim 13), 0 when absent.
    fn extra_light(&self, key: &[u8]) -> i32 {
        self.import
            .outcome
            .globals
            .get(&b"AudioVisual"[..])
            .and_then(|s| s.get(key))
            .and_then(|k| k.derived)
            .map_or(0, |v| v as i32)
    }

    /// An infantry unit in its initial state (S-0107 claims 1 to 3, 19).
    fn infantry(&mut self, inf: &glasnost_map::Infantry) -> Option<GroundEntry> {
        let p = &inf.placed;
        let (i, art, cell) = self.mobile_start(Kind::Infantry, p)?;
        let sprite = art.sprite.clone()?;
        let n = self.sprite_frames(&sprite).unwrap_or(0);
        // provisional (Q-0010): the facing value on the scale of glossary claim G1 (× 256)
        let facing = (p.facing & 0xFF) * 256;
        // spec: S-0107 claim 2 (no current sequence: the stand entry; water cells need land types, Q-0009)
        let (start, count, mult) = self.sequence_entry(i, b"Ready");
        // spec: S-0107 F2 (g = (7 − ⌊(φ + 5120) / 8192⌋) mod 8)
        let g = (7 - (facing + 5120) / 8192).rem_euclid(8);
        // spec: S-0107 F1 (frame = S + (k mod max(C, 1)) + [M > 0]·M·g, stage k = 0)
        let k = 0;
        let frame = start + k % count.max(1) + if mult > 0 { mult * g } else { 0 };
        let frame = usize::try_from(frame).ok()?;
        let (x, y) = cell_centre(p.cell.x, p.cell.y);
        let z = LEVEL_LEPTONS * i32::from(cell.level);
        let point = project_lifted(x, y, z);
        let u = self.unit_depth(Kind::Infantry, i, p.cell, z);
        let (hsv, _) = self.owner_colour(&p.owner);
        // spec: S-0104 claim 13 (infantry: U + extra infantry light; shadowed cells need the theater spec)
        let b = self
            .lights
            .get(p.cell)
            .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground)
            + self.extra_light(b"ExtraInfantryLight");
        let row = self.house_row(hsv, b);
        let object = self.add_object(ObjectInfo {
            kind: "infantry",
            type_name: String::from_utf8_lossy(&p.type_name).into_owned(),
            owner: String::from_utf8_lossy(&p.owner).into_owned(),
            cell: p.cell,
            entry: p
                .raw
                .iter()
                .map(|f| String::from_utf8_lossy(f).into_owned())
                .collect::<Vec<_>>()
                .join(","),
            notes: vec![notes::OBJECTS.into(), notes::PLACEMENT.into()],
        });
        let mut items = Vec::new();
        // spec: S-0107 claim 19 (on the ground: shadow frame + ⌊L/2⌋ at the same position, unless suppressed)
        if !rule_bool(self.import, Kind::Infantry, i, b"NoShadow", false)
            && let Some(item) = self.place_sprite(
                &sprite,
                frame + n / 2,
                point,
                DepthTest::Reader,
                // spec: S-0102 claim 13 (infantry shadow: U − 2, flat)
                ItemDepth::Profile {
                    profile: Profile::Flat,
                    bias: u - 2,
                    companion: None,
                },
                Paint::Shadow,
                Group::Shadows,
                true,
                Some(object),
            )
        {
            items.push(item);
        }
        // spec: S-0102 claim 13 (infantry drawn as a sprite: U − 2, upright, reader)
        if let Some(item) = self.place_sprite(
            &sprite,
            frame,
            point,
            DepthTest::Reader,
            ItemDepth::Profile {
                profile: Profile::Upright,
                bias: u - 2,
                companion: None,
            },
            Paint::Row(row),
            Group::Infantry,
            true,
            Some(object),
        ) {
            items.push(item);
        }
        self.note(notes::SHADOW);
        self.note(notes::PLACEMENT);
        // spec: S-0102 claim 5 (sort key X + Y of the position)
        Some(GroundEntry {
            key: i64::from(x) + i64::from(y),
            items,
        })
    }

    /// The sequence entry (start, count, multiplier) of `key` for infantry
    /// type `idx` (S-0107 Import mapping; claim 1: absent fields are 0).
    fn sequence_entry(&self, idx: usize, key: &[u8]) -> (i32, i32, i32) {
        // spec: S-0107 Import mapping (values read as at most 31 characters)
        const VALUE_LIMIT: usize = 31; // spec: S-0107 Constants (value length limit)
        let Some(seq) = self.art_value(Kind::Infantry, idx, b"Sequence") else {
            // spec: S-0107 claim 1 (no sequence table: every entry is zero)
            return (0, 0, 0);
        };
        let seq = value::text(seq, VALUE_LIMIT).to_vec();
        let Some(v) = self.art.value(&seq, key) else {
            return (0, 0, 0);
        };
        let v = &v[..v.len().min(VALUE_LIMIT)];
        // spec: S-0107 Import mapping (a missing or unparseable field keeps it and every later field at 0)
        let [s, c, m] = value::int_tuple::<3>(v);
        match (s, c, m) {
            (Some(s), Some(c), Some(m)) => (s, c, m),
            (Some(s), Some(c), None) => (s, c, 0),
            (Some(s), None, _) => (s, 0, 0),
            _ => (0, 0, 0),
        }
    }

    /// A vehicle or aircraft on the ground (S-0105 for voxel models; a
    /// sprite-drawn one shows image frame 0, provisional Q-0011).
    fn vehicle(&mut self, kind: Kind, p: &glasnost_map::objects::Placed) -> Option<GroundEntry> {
        let (i, art, cell) = self.mobile_start(kind, p)?;
        let (x, y) = cell_centre(p.cell.x, p.cell.y);
        let z = LEVEL_LEPTONS * i32::from(cell.level);
        let point = project_lifted(x, y, z);
        let u = self.unit_depth(kind, i, p.cell, z);
        let (hsv, _) = self.owner_colour(&p.owner);
        let extra = if kind == Kind::Aircraft {
            self.extra_light(b"ExtraAircraftLight")
        } else {
            self.extra_light(b"ExtraUnitLight")
        };
        // spec: S-0104 claim 13 (vehicles: U + extra unit light; aircraft on the ground: U + extra aircraft light)
        let b = self
            .lights
            .get(p.cell)
            .map_or(NEUTRAL_BRIGHTNESS, |l| l.unscaled_ground)
            + extra;
        let row = self.house_row(hsv, b);
        let object = self.add_object(ObjectInfo {
            kind: if kind == Kind::Aircraft {
                "aircraft"
            } else {
                "vehicle"
            },
            type_name: String::from_utf8_lossy(&p.type_name).into_owned(),
            owner: String::from_utf8_lossy(&p.owner).into_owned(),
            cell: p.cell,
            entry: p
                .raw
                .iter()
                .map(|f| String::from_utf8_lossy(f).into_owned())
                .collect::<Vec<_>>()
                .join(","),
            notes: vec![notes::OBJECTS.into()],
        });
        let group = if kind == Kind::Aircraft {
            Group::Aircraft
        } else {
            Group::Vehicles
        };
        let items = if art.voxels.contains_key("body") {
            self.vehicle_voxels(kind, i, &art, p, point, u, row, group, object)
        } else if let Some(sprite) = art.sprite.clone() {
            self.note(notes::PLACEMENT);
            // spec: S-0102 claim 13 (a unit drawn as a sprite: U − 2, upright, reader)
            self.place_sprite(
                &sprite,
                0,
                point,
                DepthTest::Reader,
                ItemDepth::Profile {
                    profile: Profile::Upright,
                    bias: u - 2,
                    companion: None,
                },
                Paint::Row(row),
                group,
                true,
                Some(object),
            )
            .into_iter()
            .collect()
        } else {
            Vec::new()
        };
        // spec: S-0102 claim 5 (sort key X + Y of the position)
        Some(GroundEntry {
            key: i64::from(x) + i64::from(y),
            items,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn vehicle_voxels(
        &mut self,
        kind: Kind,
        idx: usize,
        art: &glasnost_convert::art::TypeArt,
        p: &glasnost_map::objects::Placed,
        point: (i32, i32),
        u: i32,
        row: usize,
        group: Group,
        object: u32,
    ) -> Vec<Item> {
        let _ = (kind, idx, art, p, point, u, row, group, object);
        self.note(notes::VOXEL_SHADOW_POINT);
        Vec::new()
    }
}

/// A greyscale-free minimap: each cell as two pixels with its tile's radar
/// colours (the tile file's own data), cells outside the usable area of
/// S-0009 claim 4 dimmed. Size 2W × 2H. Our own viewer design.
pub fn minimap(map: &Map, catalogue: &Catalogue, assets: &Assets) -> (u32, u32, Vec<u8>) {
    let f = map.footprint;
    let (w, h) = (f.width.max(0) as u32 * 2, f.height.max(0) as u32 * 2);
    let mut px = vec![0u8; (w * h * 4) as usize];
    let mut radar: BTreeMap<Vec<u8>, Option<Tmp>> = BTreeMap::new();
    for (c, cell) in map.cells.iter() {
        let colours = catalogue.art_tile(cell.tile).and_then(|t| {
            let file = upper(&catalogue.file_of(t)?);
            let tmp = radar
                .entry(file.clone())
                .or_insert_with(|| assets.get(&file).and_then(|b| Tmp::parse(b).ok()))
                .as_ref()?;
            let tile = tmp
                .tiles
                .get(usize::from(cell.sub_tile))
                .copied()
                .flatten()?;
            Some((tile.radar_left, tile.radar_right))
        });
        let ((l, r), usable) = (
            colours.unwrap_or(([0, 0, 0], [0, 0, 0])),
            // spec: S-0009 claim 5a (the height-aware test limits the minimap; slopes need the theater spec)
            f.is_usable(c, i32::from(cell.level), false),
        );
        let x = c.d() + f.width - 1;
        let y = c.s() - f.width - 1;
        for (dx, col) in [(0, l), (1, r)] {
            let (px_x, px_y) = (x + dx, y);
            if px_x < 0 || px_y < 0 || px_x as u32 >= w || px_y as u32 >= h {
                continue;
            }
            let at = ((px_y as u32 * w + px_x as u32) * 4) as usize;
            let dim = |v: u8| if usable { v } else { v / 3 };
            px[at..at + 4].copy_from_slice(&[dim(col[0]), dim(col[1]), dim(col[2]), 255]);
        }
    }
    (w, h, px)
}
