//! Loading a map for viewing: the game inputs (`convert::game`: the map
//! decoded per S-0009, the rules import with the map as the map layer per
//! S-0006, the resolved ruleset and the theater's tile catalogues per
//! S-0010), the palettes (S-0103) and every file the scene needs; then the
//! sandbox game of the simulation and the scene.

use glasnost_convert::Import;
use glasnost_convert::game::GameInputs;
use glasnost_convert::map::to_vfs_theater;
use glasnost_convert::theater::Catalogue;
use glasnost_fmt::pal::Palette6;
use glasnost_rules::layers::Layers;
use std::sync::Arc;

use glasnost_map::Map;
use glasnost_sim::rules::SimRules;

use glasnost_render::palette::{ANIM_PALETTE_FILE, PaletteQuirks, PaletteState};
use glasnost_render::voxel::VoxelCanvas;
use glasnost_vfs::Vfs;
use glasnost_vfs::theater::Theater;

use super::build::{self, Assets, Built, Inputs, Options};
use super::cover::{self, CoverSprites};
use super::live::{self, Live};

/// A loaded map.
pub struct Loaded {
    /// The decoded map.
    pub map: Map,
    /// Importer warnings about the map file.
    pub map_warnings: Vec<String>,
    /// The rules import with the map layer.
    pub import: Import,
    /// The theater's tile catalogue.
    pub catalogue: Catalogue,
    /// The palettes in force.
    pub palettes: PaletteState,
    /// Files read for the scene.
    pub assets: Assets,
    /// The options the scene was built with.
    pub options: Options,
    /// The scene.
    pub built: Built,
    /// Minimap (width, height, RGBA).
    pub minimap: (u32, u32, Vec<u8>),
    /// Each cell's (left, right) tile radar colours after loading, when the
    /// game started (for its radar map).
    pub tile_colours: std::collections::BTreeMap<glasnost_map::CellXY, [u8; 3]>,
    /// Problems that kept parts of the map from being drawn, or the game
    /// from starting.
    pub problems: Vec<String>,
    /// The live sandbox game, when it started.
    pub live: Option<Live>,
    /// The typed rules, when the ruleset resolved.
    pub rules: Option<Arc<SimRules>>,
    /// The edge-scroll multiplier μ (S-0101 F9 and Import mapping:
    /// `[AudioVisual] ScrollMultiplier`, 1.0 when absent), presentation only.
    pub scroll_multiplier: glasnost_num::F64,
    /// The cells after loading, when the game was set up.
    pub cells: Option<glasnost_map::CellGrid>,
    /// The shroud and fog sprites of a skirmish game (S-0110 claim 3).
    pub covers: CoverSprites,
    /// The map's stored `[Basic]` `Theme` value, which names its map theme
    /// (S-0703 claim 11 and Import mapping); presentation only.
    pub theme: Option<Vec<u8>>,
    /// The voxel buffer the scene's fresh voxel draws left, when no live
    /// game holds it (S-0105 claim 34, S2).
    canvas: VoxelCanvas,
}

impl Loaded {
    /// The voxel buffer and depth-buffered mode as this map's drawing left
    /// them, for the next map loaded in the program: S-0105 claim 34 and
    /// S2 keep both across games.
    pub fn voxel_canvas(&self) -> VoxelCanvas {
        match &self.live {
            Some(live) => live.voxel_canvas().clone(),
            None => self.canvas.clone(),
        }
    }

    /// The scene inputs and the live game, borrowed apart.
    pub fn split(&mut self) -> (Inputs<'_>, Option<&mut Live>) {
        (
            Inputs {
                map: &self.map,
                import: &self.import,
                catalogue: &self.catalogue,
                palettes: &self.palettes,
                assets: &self.assets,
                options: &self.options,
                rules: self.rules.as_deref(),
                cells: self.cells.as_ref(),
            },
            self.live.as_mut(),
        )
    }

    /// Each house's radar colour by house index: its house display RGB
    /// for the radar's 5-6-5 display (S-0410 claim 8, S-0103 claim 15).
    pub fn house_radar_colours(&self, w: &glasnost_sim::world::World) -> Vec<[u8; 3]> {
        w.houses
            .iter()
            .map(|h| {
                build::owner_radar_colour(
                    &self.import,
                    &self.options,
                    h.name.as_bytes(),
                    super::radar::DISPLAY_BITS.0,
                )
            })
            .collect()
    }

    /// The neutral radar fallback colour for the radar's 5-6-5 display
    /// (S-0410 claim 8, S-0516 claim 38).
    pub fn neutral_radar_colour(&self) -> [u8; 3] {
        build::neutral_radar_colour(&self.import, super::radar::DISPLAY_BITS.0)
    }

    /// Runs one tick of the live game and redraws its objects into
    /// [`Built::scene`] (headless use; the viewer keeps its scene apart).
    pub fn tick(&mut self) {
        let mut scene = std::mem::take(&mut self.built.scene);
        let (inputs, live) = self.split();
        if let Some(live) = live {
            live.step();
            live.redraw(&inputs, &mut scene);
        }
        self.built.scene = scene;
    }
}

impl Loaded {
    /// A map loaded without an install: the decoded `map` with its game
    /// started from `setup` on `rules` and `tiles`, no art, no palettes and
    /// no files (the synthetic games of the client's tests; nothing is
    /// drawn, the rest runs as in a loaded game).
    pub fn bare(
        map: Map,
        rules: Arc<SimRules>,
        tiles: Arc<glasnost_map::terrain::TileCatalogue>,
        setup: &glasnost_sim::world::GameSetup,
    ) -> Result<Loaded, String> {
        let world = live::guarded(|| {
            glasnost_sim::load::new_game(rules.clone(), &map, tiles, setup)
                .map_err(|e| e.to_string())
        })
        .map_err(|p| format!("the simulation failed while setting up the game: {p}"))??;
        let import = glasnost_convert::convert(
            &glasnost_convert::inputs::Inputs::default(),
            &glasnost_convert::ImportOptions::default(),
        );
        let catalogue = Catalogue::default();
        let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
        let assets = Assets::default();
        let options = Options {
            game: build::GameMode::Skirmish,
            ..Options::default()
        };
        let cells = Some(world.cells.grid.clone());
        let inputs = Inputs {
            map: &map,
            import: &import,
            catalogue: &catalogue,
            palettes: &palettes,
            assets: &assets,
            options: &options,
            rules: Some(&rules),
            cells: cells.as_ref(),
        };
        let (mut built, painter) = build::build_parts_with(&inputs, true, VoxelCanvas::new());
        let live = Live::new(
            &inputs,
            &mut built,
            painter,
            (world, rules.clone(), setup.clone(), Vec::new()),
        );
        let tile_colours = build::tile_radar_colours(&built.shown_tiles, &assets);
        let minimap = build::minimap(&map, &tile_colours);
        Ok(Loaded {
            map,
            map_warnings: Vec::new(),
            import,
            catalogue,
            palettes,
            assets,
            options,
            built,
            minimap,
            tile_colours,
            problems: Vec::new(),
            live: Some(live),
            rules: Some(rules),
            // spec: S-0101 Import mapping (ScrollMultiplier absent: 1.0)
            scroll_multiplier: glasnost_num::F64::from_i64(1, glasnost_num::Rounding::TowardZero),
            cells,
            covers: CoverSprites::default(),
            theme: None,
            canvas: VoxelCanvas::new(),
        })
    }
}

async fn read_named(vfs: &Vfs, name: &str) -> Option<Vec<u8>> {
    let r = vfs.resolve(name)?;
    vfs.read_resolved(&r).await.ok()
}

/// Loads `bytes` (a map file found at `origin`) from the install mounted in
/// `vfs`, whose search order is built. With `play`, the sandbox game starts
/// too; when it does, the scene shows every owner's objects (the sandbox
/// places them all), whatever `options.skirmish` says. `canvas` is the
/// voxel buffer earlier games of the program left ([`Loaded::voxel_canvas`];
/// [`VoxelCanvas::new`] at program start), which S-0105 claim 34 and S2 keep
/// across games.
pub async fn load(
    vfs: &mut Vfs,
    bytes: Vec<u8>,
    origin: String,
    options: &Options,
    play: bool,
    canvas: VoxelCanvas,
) -> Loaded {
    load_with(
        vfs,
        bytes,
        origin,
        options,
        play,
        canvas,
        None,
        Layers::REFERENCE,
    )
    .await
}

/// The lobby colour slot of the local house that `choice` picks in
/// `setup` (S-0112 claim 1 row 4: the walls take the local player's house
/// colour; S-0103 claim 10: a house's colour is its lobby slot's), `None`
/// without a local house or without a slot.
pub(super) fn local_slot(
    setup: &glasnost_sim::world::GameSetup,
    choice: build::LocalChoice,
) -> Option<u8> {
    let h = live::local_of(setup, choice).ok()??;
    setup.skirmish.houses.get(usize::from(h.0))?.colour
}

/// [`load`] with the game's setup made by `make` from the map's typed
/// rules (the game client's skirmish); `options.game` should then be
/// [`build::GameMode::Skirmish`] so the skirmish files are read. The rules
/// are resolved with our own patch layers `layers` (M8 step C7).
#[allow(clippy::too_many_arguments)]
pub async fn load_with(
    vfs: &mut Vfs,
    bytes: Vec<u8>,
    origin: String,
    options: &Options,
    play: bool,
    canvas: VoxelCanvas,
    make: Option<live::MakeSetup<'_>>,
    layers: Layers,
) -> Loaded {
    let map_ini = glasnost_ini::Document::parse(&bytes, Default::default(), &mut Vec::new());
    // spec: S-0703 Import mapping ([Basic] Theme, the map theme; the music player cuts and matches it); S-0009 Import mapping ([Basic] Theme: read and kept as map data, absent: no map theme)
    let theme = map_ini.value(b"Basic", b"Theme").map(<[u8]>::to_vec);
    let GameInputs {
        map,
        mut map_warnings,
        import,
        resolved,
        catalogue,
        tiles,
        mut problems,
    } = glasnost_convert::game::load(vfs, bytes, origin, layers).await;
    let theater: Theater = to_vfs_theater(map.theater);
    let pal = |b: Option<Vec<u8>>| b.and_then(|b| Palette6::parse(&b).ok());
    // spec: S-0103 State and ordering (program start: the snow unit palette and the anim palette)
    let mut palettes = PaletteState::at_program_start(
        pal(read_named(vfs, &Theater::Snow.unit_palette()).await).as_ref(),
        pal(read_named(vfs, ANIM_PALETTE_FILE).await).as_ref(),
        PaletteQuirks::default(),
    );
    // spec: S-0112 claim 2 (the start-up resource table: the temperate theater palette, read at program start);
    // S-0103 Terms and claim 8 (ore and gem overlays use it; later theater loads leave it)
    palettes.set_startup_resource(
        pal(read_named(vfs, &Theater::Temperate.theater_palette()).await).as_ref(),
    );
    let mut files = std::collections::BTreeMap::new();
    for name in [
        theater.unit_palette(),
        theater.tile_palette(),
        theater.theater_palette(),
    ] {
        files.insert(name.clone(), pal(read_named(vfs, &name).await));
    }
    // spec: S-0103 claim 3 (a game in a new theater loads its palettes)
    palettes.start_game(theater, |n| files.get(n).cloned().flatten());
    let started = if play {
        match (&resolved, tiles) {
            (Err(e), _) => Err(format!("the rules did not resolve: {e}")),
            (Ok(_), None) => Err("no tile catalogue for the simulation".into()),
            (Ok(r), Some(t)) => live::start_with(&map, r, t, options.game, make),
        }
    } else {
        Err("the game was not started".into())
    };
    // The cells after loading (S-0010 claim 12), which the tiles are drawn from.
    let cells = started.as_ref().ok().map(|(w, ..)| w.cells.grid.clone());
    let mut assets = Assets::default();
    let exists = |n: &[u8]| vfs.resolve_bytes(n).is_some();
    let mut wanted = build::wanted_files(
        &map,
        cells.as_ref().unwrap_or(&map.cells),
        &import,
        &catalogue,
        &exists,
    );
    if started.is_ok() {
        // The animations a game creates in play (S-0109).
        wanted.extend(build::anim_files(&import));
    }
    // spec: S-0103 claim 17 (each named palette file for the current theater is read with the game's palettes)
    wanted.extend(build::named_palette_files(&import, theater));
    let skirmish = started.is_ok() && options.game == build::GameMode::Skirmish;
    if skirmish {
        wanted.extend(build::game_files(&import));
        wanted.insert(cover::SHROUD_FILE.to_vec());
        wanted.insert(cover::FOG_FILE.to_vec());
    }
    for name in wanted {
        let Ok(text) = std::str::from_utf8(&name) else {
            continue;
        };
        if let Some(b) = read_named(vfs, text).await {
            assets.insert(&name, b);
        }
    }
    let rules = resolved
        .as_ref()
        .ok()
        .map(|r| Arc::new(SimRules::new(&r.ruleset, &r.sim_rules_hash)));
    // spec: S-0101 Import mapping (ScrollMultiplier: a double; absent: 1.0)
    let scroll_multiplier = resolved
        .as_ref()
        .ok()
        .and_then(|r| r.ruleset.sim.general.scroll_multiplier.as_ref())
        .map_or(
            glasnost_num::F64::from_i64(1, glasnost_num::Rounding::TowardZero),
            |d| d.value(glasnost_num::Rounding::TowardZero),
        );
    let mut options = options.clone();
    if started.is_ok() && options.game == build::GameMode::Sandbox {
        // The sandbox game places every owner's objects (S-0009 claim 27 in
        // the viewer's sandbox, glasnost_sim::load::sandbox_setup).
        options.skirmish = false;
    }
    if make.is_some()
        && options.slots.is_empty()
        && let Ok((_, _, setup, _)) = &started
    {
        // spec: S-0103 claim 10, S-0412 claim 7 (each player house's colour slot as the lobby resolved it)
        // Keyed by the name's bytes as the drawing passes owners (`h.name.as_bytes()`), not
        // by the importer's type-name form (`live::name_bytes`), which differs for a name with
        // characters in U+F780 … U+F7FF and would draw that house in the fallback grey.
        options.slots = setup
            .houses
            .iter()
            .zip(&setup.skirmish.houses)
            .filter_map(|(h, v)| Some((h.name.as_bytes().to_vec(), v.colour?)))
            .collect();
    }
    if make.is_some()
        && let Some(choice) = options.local
        && let Ok((_, _, setup, _)) = &started
        && let Some(slot) = local_slot(setup, choice)
    {
        // design: render (#94): the walls' placeholder human house carries the local house's slot
        options.human_slot = slot;
    }
    if make.is_none() && !options.skirmish && options.slots.is_empty() {
        // spec: S-0103 claim 14 (outside a skirmish the map's own owners show their imported colours)
        let owners = build::map_owner_colours(&map, &import, &map_ini);
        options.map_colours = owners.colours;
        options.unresolved_owners = owners.unresolved;
        map_warnings.extend(owners.warnings);
    }
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: rules.as_deref(),
        cells: cells.as_ref(),
    };
    let (mut built, painter) = build::build_parts_with(&inputs, started.is_ok(), canvas);
    let mut canvas = VoxelCanvas::new();
    let live = match started {
        Ok(s) => Some(Live::new(&inputs, &mut built, painter, s)),
        Err(e) => {
            if play {
                problems.push(format!("the sandbox game did not start: {e}"));
            }
            canvas = painter.canvas().clone();
            None
        }
    };
    let covers = if skirmish {
        let (c, warnings) = CoverSprites::load(&assets);
        problems.extend(warnings);
        c
    } else {
        CoverSprites::default()
    };
    let tile_colours = build::tile_radar_colours(&built.shown_tiles, &assets);
    let minimap = build::minimap(&map, &tile_colours);
    Loaded {
        map,
        map_warnings,
        import,
        catalogue,
        palettes,
        assets,
        options,
        built,
        minimap,
        tile_colours,
        problems,
        live,
        rules,
        scroll_multiplier,
        cells,
        canvas,
        covers,
        theme,
    }
}
