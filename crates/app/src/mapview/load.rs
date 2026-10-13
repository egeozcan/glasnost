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
    /// The cells after loading, when the game was set up.
    pub cells: Option<glasnost_map::CellGrid>,
    /// The shroud and fog sprites of a skirmish game (S-0110 claim 3).
    pub covers: CoverSprites,
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
    let GameInputs {
        map,
        map_warnings,
        import,
        resolved,
        catalogue,
        tiles,
        mut problems,
    } = glasnost_convert::game::load(vfs, bytes, origin).await;
    let theater: Theater = to_vfs_theater(map.theater);
    let pal = |b: Option<Vec<u8>>| b.and_then(|b| Palette6::parse(&b).ok());
    // spec: S-0103 State and ordering (program start: the snow unit palette and the anim palette)
    let mut palettes = PaletteState::at_program_start(
        pal(read_named(vfs, &Theater::Snow.unit_palette()).await).as_ref(),
        pal(read_named(vfs, ANIM_PALETTE_FILE).await).as_ref(),
        PaletteQuirks::default(),
    );
    // spec: S-0112 claim 2 (the start-up resource table: the temperate theater palette, read at program start)
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
            (Ok(r), Some(t)) => live::start(&map, r, t, options.game),
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
    let mut options = options.clone();
    if started.is_ok() && options.game == build::GameMode::Sandbox {
        // The sandbox game places every owner's objects (S-0009 claim 27 in
        // the viewer's sandbox, glasnost_sim::load::sandbox_setup).
        options.skirmish = false;
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
        cells,
        canvas,
        covers,
    }
}
