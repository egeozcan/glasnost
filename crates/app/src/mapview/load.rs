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
use glasnost_map::Map;
use glasnost_render::palette::{ANIM_PALETTE_FILE, PaletteQuirks, PaletteState};
use glasnost_vfs::Vfs;
use glasnost_vfs::theater::Theater;

use super::build::{self, Assets, Built, Inputs, Options};
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
    /// Problems that kept parts of the map from being drawn, or the game
    /// from starting.
    pub problems: Vec<String>,
    /// The live sandbox game, when it started.
    pub live: Option<Live>,
}

impl Loaded {
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
/// places them all), whatever `options.skirmish` says.
pub async fn load(
    vfs: &mut Vfs,
    bytes: Vec<u8>,
    origin: String,
    options: &Options,
    play: bool,
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
    let mut assets = Assets::default();
    for name in build::wanted_files(&map, &import, &catalogue) {
        let Ok(text) = std::str::from_utf8(&name) else {
            continue;
        };
        if let Some(b) = read_named(vfs, text).await {
            assets.insert(&name, b);
        }
    }
    let started = if play {
        match (&resolved, tiles) {
            (Err(e), _) => Err(format!("the rules did not resolve: {e}")),
            (Ok(_), None) => Err("no tile catalogue for the simulation".into()),
            (Ok(r), Some(t)) => live::start(&map, r, t),
        }
    } else {
        Err("the game was not started".into())
    };
    let mut options = options.clone();
    if started.is_ok() {
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
    };
    let (mut built, painter) = build::build_parts(&inputs, started.is_ok());
    let live = match started {
        Ok(s) => Some(Live::new(&inputs, &mut built, painter, s)),
        Err(e) => {
            if play {
                problems.push(format!("the sandbox game did not start: {e}"));
            }
            None
        }
    };
    let minimap = build::minimap(&map, &catalogue, &assets);
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
        problems,
        live,
    }
}
