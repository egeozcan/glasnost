//! A game on a loaded map, as the client shows it: the [`Play`] model, the
//! map's scene and the live drawing state (`mapview::live`), the camera,
//! the radar, the sidebar icons' art, and the pointer's gestures on the
//! battlefield and the sidebar. Drawing produces a [`DrawList`] over the
//! battlefield image; the battlefield itself is composed on the CPU here
//! ([`Session::map_rgba`]) or on the GPU by the client from the same scene.
//!
//! The canonical world is the one in `play.game`; the live drawing state
//! keeps a copy of the same start (both are `load::new_game` of one setup)
//! and borrows the canonical world by swapping it in for each call that
//! reads it ([`Session::with_live`]).

use glasnost_rules::layers::Layers;
use std::collections::BTreeMap;

use glasnost_ai::game::Game;
use glasnost_convert::kinds::Kind;
use glasnost_fmt::image::IndexedImage;
use glasnost_fmt::shp::Shp;
use glasnost_map::CellXY;
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE};
use glasnost_render::projection::{
    ProjectionQuirks, TERRAIN_LIFT, cell_top, cursor_cell, limit_centre, local_area_rect,
    project_lifted, view_origin,
};
use glasnost_render::scene::{Filter, Group, Scene, View};
use glasnost_render::shroud::ShroudQuirks;
use glasnost_sim::econ::{TypeRef, victory};
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, ObjectId};
use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::geom::{Point, Rect, Scale};
use glasnost_ui::hud::{self, HudHit, Layout, PauseHit};
use glasnost_ui::modes::{self, Mode};
use glasnost_ui::watch;
use glasnost_ui::widget::{Focus, Nav, Targets};
use glasnost_vfs::{FileRef, Vfs};

use super::band;
use super::cursor::{self, Offer};
use super::name_tip;
use super::paint::Painter;
use super::play::{Button, GROUPS, Play, Reach, Tool, local_house};
use super::scroll::{self, EdgeScroll};
use super::sidebar_art::{self, SidebarArt};
use super::status_font;
use super::strings;
use crate::mapview::build::{self, GameMode, Inputs, Options};
use crate::mapview::combat::CURSOR_PALETTE;
use crate::mapview::cover;
use crate::mapview::live::{self, Live};
use crate::mapview::load::{self, Loaded};
use crate::mapview::radar::Radar;

/// Image id of the battlefield in the client's draw lists.
pub const MAP_IMAGE: u32 = 1;
/// Image id of the radar map.
pub const RADAR_IMAGE: u32 = 2;
/// Sprite id of the battlefield cursor image's frames.
// nocite: our own sprite id, below the icons'
const CURSOR_SPRITE: u32 = 900;
/// First sprite id of the sidebar icons.
const ICON_SPRITES: u32 = 1000;
/// First sprite id of the alternate sidebar icons (S-0007 claim 25).
// nocite: our own sprite id, above the main icons'
const ALT_ICON_SPRITES: u32 = 6000;
/// The 64-bit FNV-1a hash of a map file: the map's name in replays, so
/// that no map is named by its title (our own identifier). It lives in
/// `glasnost-net` since M9 N4, where the desync tool finds a report's map
/// by it too.
pub use glasnost_net::start::content_hash;

/// A game on a loaded map.
pub struct Session {
    /// The map, its scene inputs and the live drawing state.
    pub loaded: Loaded,
    /// The scene (the live objects redrawn into it after every tick).
    pub scene: Scene,
    /// The game and the local player's interface state.
    pub play: Play,
    /// The radar map (S-0410).
    pub radar: Option<Radar>,
    /// The map's content hash.
    pub map_id: String,
    /// The paced game-speed index (S-0201 claims 7, 8 and 17): in a
    /// single-player game the index the options menu last set, from the
    /// next cycle on; in a network game the index in force, which follows
    /// the game's after every tick (a speed order carried out).
    pub speed: u8,
    /// The options menu's speed control while the menu is open (S-0201
    /// claims 8 and 17): it starts at the index in force and is sent as a
    /// speed order when the menu closes, if it differs.
    menu_speed: u8,
    /// The person resigned through the options menu: in a network game
    /// the menu then stays refused until the local house is defeated
    /// (S-0201 claim 15); in a skirmish it no longer offers the surrender.
    resigned: bool,
    /// A network game's "surrender and leave" was chosen: the client
    /// leaves once the lockstep has carried out the local surrender order
    /// (S-0201 claim 15).
    pub leave_after_surrender: bool,
    /// The options menu is open (S-0201 claim 8: in a single-player game no
    /// cycle runs; claim 15: in a network game ticks go on).
    pub paused: bool,
    /// The window has the input focus (S-0201 claims 9 and 16); the client
    /// sets it every update.
    pub focused: bool,
    /// A network game: the lockstep session paces the ticks and the
    /// person's orders, a speed change among them, go through it
    /// (`Play::outbox`; S-0201 claims 15 to 17).
    pub network: bool,
    /// The scroll-speed index of the person's settings (S-0101: 0 fastest to 6 slowest).
    pub scroll_index: i32,
    /// The window size.
    pub screen: (i32, i32),
    /// Whether the client has set the starting window size.
    sized: bool,
    centre: (i32, i32),
    filter: Filter,
    /// Decoded icon frames by type and superweapon (S-0007 claims 22, 25
    /// and 31).
    icons: BTreeMap<hud::Entry, IndexedImage>,
    /// Sprite ids of the registered icons.
    icon_ids: BTreeMap<hud::Entry, u32>,
    /// Decoded alternate icon frames by type (S-0007 claim 25).
    alt_icons: BTreeMap<TypeRef, IndexedImage>,
    /// Sprite ids of the registered alternate icons.
    alt_icon_ids: BTreeMap<TypeRef, u32>,
    /// The icon palette's RGBA (S-0103 claim 13).
    icon_palette: Option<[u8; 1024]>,
    /// The install's sidebar button images and palette, read at the game's
    /// start (S-0411 claim 19); `None` without the palette.
    sidebar_art: Option<SidebarArt>,
    /// The button images the sidebar draws and lays out (S-0411 claim 19).
    buttons: hud::ButtonArt,
    pointer: Option<Point>,
    /// The name tip's rest and the tip that shows (S-0111 claims 20 to 26).
    name_tip: name_tip::NameTip,
    /// The player's tooltip option (S-0111 claim 20), as the client's
    /// settings give it.
    pub tool_tips: bool,
    /// The string table (S-0004 claim 17, S-0015): the install's
    /// `ra2md.csf`, read once when the program starts and handed to each
    /// game by the client; none in a game started without an install.
    pub strings: Option<strings::StringTable>,
    /// The view's centre at the last name-tip update, to tell a moving view
    /// (S-0111 claim 20).
    tip_centre: (i32, i32),
    /// The name tip glyphs' painter rows, one per tip colour
    /// ([`name_tip::register`]).
    tip_rows: [Option<u32>; 2],
    /// Edge momentum and its retained direction (S-0101 F9).
    edge: EdgeScroll,
    /// Whether the last scroll cycle allowed edge scrolling (focus, no
    /// order block: S-0201 claim 14).
    edge_live: bool,
    press: Option<(Point, Button)>,
    /// Whether a left or right button pressed over the battlefield view is
    /// still held (blocks the name tip, S-0111 claim 20).
    view_button_held: bool,
    /// The armed band-box drag (S-0214 claim 18).
    drag: Option<band::Drag>,
    /// The right-button press in the battlefield and its scroll-drag
    /// classification (S-0214 claims 26 to 28, F8).
    right: Option<band::RightDrag>,
    /// The refused one-pixel directions (up, right, down, left) of the last
    /// permitted right-drag update, for its cursor (S-0101 claim 17); none
    /// when the last scroll cycle made no drag update.
    drag_refused: Option<[bool; 4]>,
    /// The imported cursor image (S-0214 claim 22), when the install has it.
    cursor_art: Option<CursorArt>,
    /// The cursor palette's painter row, once registered.
    cursor_row: Option<u32>,
    /// The status glyphs' painter row (index 1 the foreground), once
    /// registered.
    status_row: Option<u32>,
    /// The cursor's animation (S-0214 claim 22).
    animator: cursor::Animator,
    /// The cursor frame drawn this frame and its hot spot.
    cursor_shown: Option<(u16, (i32, i32))>,
    /// The modifiers held, for the hovered action.
    pub mods: Modifiers,
    /// The type-select role's key (S-0214 claim 12).
    type_select: TypeSelect,
    /// Route mode's key and point selection (S-0213 claims 16 and 16e).
    route: RouteMode,
    /// The last control group selected and when, for the double press
    /// that goes to it.
    last_group: Option<(usize, i64)>,
    /// The interface's real-time clock, milliseconds (S-0214 claim 22: not
    /// simulation ticks).
    pub clock_ms: i64,
    /// Each house's display colour by house index (the radar colours,
    /// S-0410 claim 8), for the psychic sensor's intent-line end squares.
    pub(crate) house_colours: Vec<[u8; 3]>,
    /// The last drawn sidebar's targets, in the interface's logical pixels.
    targets: Targets<HudHit>,
    pause_targets: Targets<PauseHit>,
    /// The interface scale the sidebar, the options menu and the client's
    /// notices are drawn at (M8 step C3); the battlefield keeps its pixels.
    ui: Scale,
    /// The keyboard focus on the sidebar.
    hud_focus: Focus<HudHit>,
    /// The keyboard focus on the options menu.
    pause_focus: Focus<PauseHit>,
    /// The keyboard's placement or aiming cell, while the keyboard aims.
    aim: Option<CellXY>,
    /// Problems met while loading (shown in the log).
    pub problems: Vec<String>,
    /// A host clock in milliseconds, set only by the performance benchmark
    /// (M8 step C4): each step then records its [`StepLaps`].
    pub lap_clock: Option<fn() -> f64>,
    /// The last step's laps, when [`Session::lap_clock`] is set.
    pub laps: StepLaps,
}

/// Host-clock times of one step's parts, in milliseconds, for the
/// performance benchmark (M8 step C4; our own tooling, never game state).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StepLaps {
    /// The game's tick (the AI's thinking and the simulation).
    pub tick: f64,
    /// The interface model after the tick ([`Play::step_by`] less the tick).
    pub play: f64,
    /// The live objects redrawn into the scene.
    pub scene: f64,
    /// The radar's observation.
    pub radar: f64,
}

/// Our own keys for the sidebar and keyboard aiming (M8 step C3; the
/// client maps keys to them).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideKey {
    /// The next or previous sidebar control (`true` forward).
    Step(bool),
    /// Press the focused control as a left click (`shift`: with the
    /// force-select modifier), or place or fire at the keyboard cell while
    /// it aims.
    Press {
        /// Shift was held.
        shift: bool,
    },
    /// The focused control as a right click (hold or cancel production),
    /// or end keyboard aiming as a battlefield right click does.
    Back,
    /// The next (+1) or previous (-1) sidebar tab with icons.
    Tab(i32),
    /// Move the keyboard cell one cell in a screen direction (−1, 0 or +1
    /// in x and y), while placing or aiming.
    Cell(i32, i32),
    /// Place or fire at the keyboard cell, while it aims.
    Place,
}

/// What the client must do after a gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionEvent {
    /// Nothing else.
    None,
    /// Leave the game for the main menu.
    Leave,
    /// Open the settings screen (the game stays paused behind it).
    Settings,
}

impl Session {
    /// Loads the map file `file` of the mounted install and starts the game
    /// `make` sets up from the map's rules, played from house `local` (a
    /// network game's slot), else from the first person's house, else, when
    /// the setup has no person's house, watched from no house (a watch
    /// game, [`Play::watching`]). `cursor`
    /// is the session's cursor palette keep (S-0111 claim 19a);
    /// `colour_blind` draws the player houses in our colour-blind colours
    /// (presentation only, M8 step C3). With `watch`, `local` may be an AI
    /// house, watched rather than played (the performance benchmark, M8
    /// step C4). The rules are resolved with our own patch layers `layers`
    /// (M8 step C7), which the game's pathing reports record.
    #[allow(clippy::too_many_arguments)]
    pub async fn start(
        vfs: &mut Vfs,
        file: &FileRef,
        make: live::MakeSetup<'_>,
        local: Option<HouseId>,
        cursor: &CursorPaletteKeep,
        colour_blind: bool,
        watch: bool,
        layers: Layers,
    ) -> Result<Self, String> {
        let bytes = vfs
            .read(file)
            .await
            .map_err(|e| format!("the map could not be read: {e}"))?;
        let map_id = content_hash(&bytes);
        let options = Options {
            skirmish: true,
            game: GameMode::Skirmish,
            colour_blind,
            ..Options::default()
        };
        let mut loaded = load::load_with(
            vfs,
            bytes,
            map_id.clone(),
            &options,
            true,
            glasnost_render::voxel::VoxelCanvas::new(),
            Some(make),
            layers,
        )
        .await;
        // spec: S-0111 claim 19a (one cursor palette for the cursor images, the wrench and the bomb icon: the session's kept copy with quirks.art_cache_never_invalidated on, this game's read with it off)
        let palette = cursor.for_game(vfs).await;
        loaded.assets.set(CURSOR_PALETTE, palette.clone());
        if let Some(l) = loaded.live.as_mut() {
            l.note_cursor_palette(&loaded.assets);
        }
        let (icons, alt_icons) = read_icons(vfs, &loaded).await;
        let cursor_art = read_cursor(vfs, palette.as_deref()).await;
        let status_font = status_font::read(vfs).await;
        let mut s = Self::assemble(
            loaded,
            map_id,
            (icons, alt_icons, cursor_art),
            status_font,
            local,
            watch,
        )?;
        s.play.capture.layers = layers;
        // spec: S-0411 claim 19 (the button images resolved with the local player's side group mounted, S-0005 claims 13 and 13a)
        if let Some(side) = s.local_side(watch) {
            let theater = glasnost_convert::map::to_vfs_theater(s.loaded.map.theater);
            sidebar_art::mount_side(vfs, theater, side).await;
        }
        s.set_sidebar_art(sidebar_art::read(vfs).await);
        Ok(s)
    }

    /// The side index whose group the game mounts for the local player
    /// (S-0005 claim 13a): that of the local house's country, or, for a
    /// watched game, of the country of registration index 0 (an observing
    /// local player). `None` for a country with no side.
    fn local_side(&self, watch: bool) -> Option<u32> {
        let w = &self.play.game.world;
        let country = match self.play.local {
            // spec: S-0005 claim 13a (the country the local player plays)
            Some(h) if !watch => usize::try_from(w.houses.get(usize::from(h.0))?.country).ok()?,
            // spec: S-0005 claim 13a (an observing local player counts as playing the country of registration index 0)
            // design: watch mode (lead request 2026-11-02, an extra not in the original): the watcher of a watch game is mounted as an observer is
            _ => 0,
        };
        let side = w
            .rules()
            .econ
            .tech
            .country_side
            .get(country)
            .copied()
            .flatten()?;
        u32::try_from(side).ok()
    }

    /// Takes the game's sidebar button images (S-0411 claim 19); every
    /// frame counts as drawable until the painter registers them.
    pub fn set_sidebar_art(&mut self, art: Option<SidebarArt>) {
        self.buttons = art
            .as_ref()
            .map(|a| sidebar_art::button_art(a, |_, _| true))
            .unwrap_or_default();
        self.sidebar_art = art;
    }

    /// A game on `loaded`, a map loaded without art (the synthetic
    /// scenarios of tests: nothing is drawn, everything else runs), played
    /// from house `local`, else from the first person's house, else watched
    /// from no house.
    pub fn bare(loaded: Loaded, map_id: String, local: Option<HouseId>) -> Result<Self, String> {
        Self::assemble(
            loaded,
            map_id,
            (BTreeMap::new(), BTreeMap::new(), None),
            Err("no game font in a bare game".into()),
            local,
            false,
        )
    }

    /// The game of a loaded map with its art.
    fn assemble(
        mut loaded: Loaded,
        map_id: String,
        art: Art,
        status_font: Result<status_font::StatusFont, String>,
        local: Option<HouseId>,
        watch: bool,
    ) -> Result<Self, String> {
        let (icons, alt_icons, cursor_art) = art;
        // spec: S-0103 claim 13 (the dedicated icon palette file, claim 1's conversion)
        let icon_file = loaded
            .assets
            .get(glasnost_render::palette::ICON_PALETTE_FILE)
            .and_then(|b| glasnost_fmt::pal::Palette6::parse(b).ok());
        let icon_missing = icon_file.is_none();
        let icon_palette = Some(icon_rgba(&glasnost_render::palette::icon_palette_565(
            icon_file.as_ref(),
            &glasnost_render::palette::PaletteQuirks::default(),
        )));
        let scene = std::mem::take(&mut loaded.built.scene);
        let mut problems = std::mem::take(&mut loaded.problems);
        let Some(live) = loaded.live.as_mut() else {
            return Err(problems
                .pop()
                .unwrap_or_else(|| "the game did not start".into()));
        };
        if icon_missing {
            // spec: S-0103 claim 13 (our engine's departure: black icons and an importer warning naming the file)
            problems.push(
                "CAMEO.PAL: the sidebar icon palette cannot be read; icons are drawn black (S-0103 claim 13)"
                    .into(),
            );
        }
        live.clear_scripts();
        // The game view draws art only: no debug markers for what has none.
        live.set_debug_markers(false);
        let setup: GameSetup = live.setup.clone();
        let catalogue = live
            .world
            .catalogue
            .clone()
            .ok_or("the game has no tile catalogue")?;
        let game = Game::new(
            live.rules.clone(),
            &loaded.map,
            &map_id,
            catalogue,
            setup.clone(),
        )
        .map_err(|e| format!("the game did not start: {e}"))?;
        debug_assert_eq!(
            glasnost_sim::save::state_hash(&game.world),
            glasnost_sim::save::state_hash(&live.world),
            "the drawn copy and the game start alike"
        );
        let local = match local {
            Some(h)
                if setup
                    .houses
                    .get(usize::from(h.0))
                    .is_some_and(|x| watch || !x.ai) =>
            {
                Some(h)
            }
            Some(h) => return Err(format!("house {} is no person's house", h.0)),
            None if watch => {
                // A watched game's first player house (M8 step C4).
                let first = setup
                    .houses
                    .iter()
                    .position(|h| h.role == HouseRole::Player);
                Some(
                    local_house(&setup)
                        .or(first.map(|i| HouseId(i as u8)))
                        .ok_or("the setup has no player house")?,
                )
            }
            // spec: S-0412 Terms (human participant: our setup may start a skirmish with none, every player house an AI house)
            // design: watch mode (lead request 2026-11-02, an extra not in the original): a setup with no person's house is watched from no house at all
            None => local_house(&setup),
        };
        if let Some(l) = loaded.live.as_mut() {
            // spec: S-0516 claims 28 and 29 (the battlefield drawn through the local player's eyes)
            // design: watch mode (lead request 2026-11-02, an extra not in the original): with no viewer every object is drawn as its own house sees it, with no shroud
            l.set_viewer(local);
        }
        let house_colours = loaded.house_radar_colours(&game.world);
        if let Some(l) = loaded.live.as_mut() {
            // spec: S-0111 claim 17 (the digit and its border in the owner's house display RGB of S-0103 claim 15, at display precision)
            l.set_house_colours(house_colours.clone());
        }
        let radar = Radar::new(
            &game.world,
            loaded.map.theater,
            &loaded.tile_colours,
            house_colours.clone(),
            loaded.neutral_radar_colour(),
        );
        let speed = game.world.game_speed;
        // design: watch mode (lead request 2026-11-02, an extra not in the original): the view starts over the first player house's objects
        let looked_at = local.or_else(|| {
            let first = setup
                .houses
                .iter()
                .position(|h| h.role == HouseRole::Player);
            first.map(|i| HouseId(i as u8))
        });
        let start = game
            .world
            .objects
            .values()
            .find(|o| o.owner.is_some() && o.owner == looked_at && o.on_map)
            .map(|o| project_lifted(o.pos.x, o.pos.y, o.pos.z));
        let mut s = Self {
            loaded,
            scene,
            play: Play::new(game, local),
            radar,
            map_id,
            speed,
            menu_speed: speed,
            resigned: false,
            leave_after_surrender: false,
            paused: false,
            focused: true,
            network: false,
            scroll_index: scroll::DEFAULT_SCROLL_INDEX,
            screen: (1024, 768),
            sized: false,
            centre: (0, 0),
            filter: Filter {
                groups: Group::ALL.to_vec(),
                provisional: true,
            },
            icons,
            icon_ids: BTreeMap::new(),
            alt_icons,
            alt_icon_ids: BTreeMap::new(),
            icon_palette,
            sidebar_art: None,
            buttons: hud::ButtonArt::default(),
            pointer: None,
            name_tip: name_tip::NameTip::default(),
            // spec: S-0111 claim 20 and Import mapping (the tooltip option is on unless the player's settings turn it off)
            tool_tips: true,
            // spec: S-0015 State and ordering (the table is read once, at start-up, by the install: `install::prepare`; the client hands it to the game)
            strings: None,
            tip_centre: (0, 0),
            tip_rows: [None; 2],
            edge: EdgeScroll::default(),
            edge_live: true,
            press: None,
            view_button_held: false,
            drag: None,
            right: None,
            drag_refused: None,
            cursor_art,
            cursor_row: None,
            status_row: None,
            animator: cursor::Animator::default(),
            cursor_shown: None,
            mods: Modifiers::default(),
            type_select: TypeSelect::default(),
            route: RouteMode::default(),
            last_group: None,
            clock_ms: 0,
            house_colours,
            targets: Targets::new(),
            pause_targets: Targets::new(),
            ui: Scale::one((1024, 768)),
            hud_focus: Focus::new(),
            pause_focus: Focus::new(),
            aim: None,
            problems: Vec::new(),
            lap_clock: None,
            laps: StepLaps::default(),
        };
        problems.append(&mut s.loaded.problems);
        if let Some(l) = s.loaded.live.as_mut() {
            // spec: S-0111 claim 17 and Import mapping (the group number uses the game font of S-0213 claim 25; without a usable font no number is drawn, S-0014 claim 6)
            let digits = status_font
                .as_ref()
                .ok()
                .and_then(|f| crate::mapview::combat::GroupFont::new(f.font()));
            l.set_group_font(digits);
        }
        match status_font {
            Ok(f) => s.play.status_font = Some(f),
            // spec: S-0213 Import mapping, S-0014 claim 6 (a missing or rejected font: our engine reports a missing-asset diagnostic naming the file and reason and omits the text)
            Err(e) => problems.push(e),
        }
        s.problems = problems;
        s.set_centre(start.unwrap_or((0, 0)));
        // spec: S-0213 claim 25 (the status width budget after game start)
        let view_w = s.map_rect().w;
        s.play.status_view(view_w, false);
        s.play.say("Game started.");
        Ok(s)
    }

    /// Calls `f` with the live drawing state holding the canonical world.
    pub fn with_live<R>(
        &mut self,
        f: impl FnOnce(&mut Live, &Inputs, &mut Scene) -> R,
    ) -> Option<R> {
        let view = self.view();
        let (inputs, live) = self.loaded.split();
        let live = live?;
        // spec: S-0316 claim 5 (the bolts' view test is on the battlefield view as drawn)
        live.battlefield = Some((view.origin, view.size));
        std::mem::swap(&mut live.world, &mut self.play.game.world);
        let r = f(live, &inputs, &mut self.scene);
        std::mem::swap(&mut live.world, &mut self.play.game.world);
        Some(r)
    }

    /// Hands the click's move-target markers to the live drawing state
    /// (S-0214 claim 30) and redraws, so they show at once, paused or not.
    fn show_markers(&mut self) {
        let markers = std::mem::take(&mut self.play.markers);
        if markers.is_empty() {
            return;
        }
        self.with_live(|live, inputs, scene| {
            for at in markers {
                live.move_marker(at);
            }
            live.redraw(inputs, scene);
        });
    }

    /// Puts `game` in place of the game (a rejoining peer's game as a live
    /// peer sent it, M9 N5) and brings the drawing and the radar up to it.
    pub fn replace_game(&mut self, game: Game) {
        self.play.game = game;
        self.catch_up();
    }

    /// Brings the drawing and the radar up to a game that moved on without
    /// them (a game put in place, or ticks the performance benchmark ran
    /// directly, M8 step C4).
    pub fn catch_up(&mut self) {
        // The pathing report's ring starts again from the game as it is,
        // with the game's layers (M8 step C7).
        let layers = self.play.capture.layers;
        self.play.capture = super::capture::Ring::new(&self.play.game);
        self.play.capture.layers = layers;
        self.with_live(|live, inputs, scene| {
            live.after_tick();
            live.redraw(inputs, scene);
        });
        let house = self.play.local;
        if let Some(r) = self.radar.as_mut() {
            r.observe(&self.play.game.world, house);
        }
    }

    /// Runs one tick and redraws the live objects.
    pub fn step(&mut self) {
        self.step_by(|g| {
            g.step();
            true
        });
    }

    /// Runs `tick` on the game, which steps it at most once and says
    /// whether it did (a network game's lockstep session), and when it did,
    /// the interface follows and the live objects are redrawn.
    pub fn step_by(&mut self, tick: impl FnOnce(&mut Game) -> bool) -> bool {
        self.play.capture.speed = self.speed;
        let clock = self.lap_clock;
        let lap = || clock.map_or(0.0, |c| c());
        let start = lap();
        let mut tick_ms = 0.0;
        let ran = self.play.step_by(|g| {
            let t = lap();
            let r = tick(g);
            tick_ms = lap() - t;
            r
        });
        if !ran {
            return false;
        }
        if self.network {
            // spec: S-0201 claim 17, Our engine (a network game is paced at the index in force from the cycle after a speed order is carried out)
            self.speed = self.play.game.world.game_speed;
        }
        self.route_after_tick();
        let played = lap();
        let effects = self.play.effects.clone();
        // spec: S-0201 claim 15 (in a network game the opener's machine draws no new frame of the battlefield while the options menu is open; the ticks go on) and claim 16 (nor while its window lacks the focus)
        let draw = !(self.network && (self.paused || !self.focused));
        self.with_live(|live, inputs, scene| {
            live.world.effects = effects;
            live.after_tick();
            if draw {
                live.redraw(inputs, scene);
            }
        });
        let drawn = lap();
        let house = self.play.local;
        if let Some(r) = self.radar.as_mut() {
            r.observe(&self.play.game.world, house);
        }
        if clock.is_some() {
            self.laps = StepLaps {
                tick: tick_ms,
                play: played - start - tick_ms,
                scene: drawn - played,
                radar: lap() - drawn,
            };
        }
        true
    }

    /// What the person looks at now, for a pathing report (M8 A4): the
    /// selection, the view and the cells under its corners, the cell
    /// under the pointer and the game speed. The ring adds the clicks.
    pub fn look(&mut self) -> super::capture::Look {
        let v = self.view();
        let m = self.map_rect();
        let mut corners = [None; 4];
        if !m.is_empty() {
            let points = [
                Point::new(m.x, m.y),
                Point::new(m.right() - 1, m.y),
                Point::new(m.x, m.bottom() - 1),
                Point::new(m.right() - 1, m.bottom() - 1),
            ];
            for (c, p) in corners.iter_mut().zip(points) {
                *c = Some(self.under(p).0);
            }
        }
        let cursor = self
            .pointer
            .filter(|p| m.contains(*p))
            .map(|p| self.under(p).0);
        super::capture::Look {
            local: self.play.local.map(|h| h.0),
            selection: self.play.selection.iter().map(|o| o.0).collect(),
            view: super::capture::ViewRect {
                origin: v.origin,
                size: v.size,
                corners,
            },
            cursor,
            clicks: Vec::new(),
            speed: self.speed,
        }
    }

    /// Makes the pathing report of now (M8 A4) and stores it with `save`;
    /// returns the status text shown.
    pub fn save_path_report(&mut self, save: Option<super::report::SaveReport>) -> String {
        let look = self.look();
        let (text, _) = super::capture::save_report(&mut self.play, self.network, look, save);
        log::info!("{text}");
        self.play.say(text.clone());
        text
    }

    /// Marks the present tick for the pathing report (M8 A4).
    pub fn mark(&mut self) {
        let tick = self.play.game.world.tick;
        self.play.capture.mark(tick);
        self.play
            .say(format!("Marked tick {tick} for the pathing report."));
    }

    /// The radar map's top-left pixel on the screen (S-0410 F9's origin
    /// before the image's own offset).
    pub fn radar_origin(&self) -> (i32, i32) {
        // The radar is drawn in the interface's logical pixels, so its
        // origin is given in them too (equal to the screen's at 100 %).
        let r = self.layout().radar;
        (r.x, r.y)
    }

    /// The superweapon countdown rows every player sees (S-0512 claim 18),
    /// in grant order, with each house's colour.
    pub fn countdown_rows(&self) -> Vec<hud::CountdownRow> {
        let w = &self.play.game.world;
        let types = &w.rules().superweapons.types;
        // spec: S-0512 claim 18 (every granted record with the countdown flag of a house that is not passive, seen by every player whatever the house and the alliances, in grant order)
        glasnost_sim::superweapons::charge::countdown_rows(w)
            .into_iter()
            .map(|r| hud::CountdownRow {
                // Seam (countdown display name): claim 18 shows the type's
                // display name, a string-table label (S-0512 Import mapping);
                // the row still shows the type's name, as the sidebar's icons
                // do: routing the superweapon's label through the table
                // (`strings::resolve`) is not part of the name tip's
                // intake 109 and stays a seam.
                name: types
                    .get(r.sw as usize)
                    .map_or_else(String::new, |t| t.name.clone()),
                // spec: S-0512 claim 18 (the name in its house's colour)
                colour: self
                    .house_colours
                    .get(usize::from(r.house.0))
                    .copied()
                    .unwrap_or([255, 255, 255]),
                seconds: r.seconds,
            })
            .collect()
    }

    /// The battlefield view's outline on the radar map (S-0410 claim 11),
    /// in radar map pixels; none without a radar map.
    pub fn radar_view_box(&self) -> Option<hud::ViewBox> {
        let r = self.radar.as_ref()?;
        let v = self.view();
        let (x, y, w, h) = r.image_rect();
        Some(hud::ViewBox {
            image: Rect::new(x, y, w, h),
            edges: r.view_outline(v.origin, (v.size.0 as i32, v.size.1 as i32)),
        })
    }

    /// The sidebar's layout in the interface's logical pixels.
    pub fn layout(&self) -> Layout {
        Layout::with_buttons(self.ui.ui, &self.buttons)
    }

    /// The scroll button (0 up, 1 down) whose press is in progress with
    /// the pointer over it (S-0411 claim 19, frame 1): a press that started
    /// on it with the pointer on it now, so exactly when a release would
    /// act (claim 18).
    pub(crate) fn scroll_pressed(&self) -> Option<usize> {
        let (start, _) = self.press?;
        let p = self.pointer?;
        match self.hud_at(start) {
            // spec: S-0411 claim 19 (a scroll button's press in progress, with the pointer over it) and claim 18 (a page button acts on release over the button after a press there)
            Some(HudHit::Scroll(d)) if self.hud_at(p) == Some(HudHit::Scroll(d)) => {
                Some(usize::from(d > 0))
            }
            _ => None,
        }
    }

    /// The interface scale.
    pub fn ui_scale(&self) -> Scale {
        self.ui
    }

    /// The battlefield's rectangle on the screen: the screen left of the
    /// scaled sidebar, at the battlefield's own pixel scale.
    pub fn map_rect(&self) -> Rect {
        let l = self.layout();
        Rect::new(0, 0, self.ui.x(l.map.right()).max(0), self.screen.1)
    }

    /// Sets the window size and the interface scale. The client's first
    /// call, as the game starts, sets the starting size; a later change of
    /// the battlefield's size (a new window size, or a scale that widens or
    /// narrows the sidebar) is an in-game change of the battlefield view's
    /// dimensions (S-0213 claim 25).
    pub fn resize(&mut self, size: (i32, i32), scale: Scale) {
        let first = !self.sized;
        self.sized = true;
        let before = self.map_rect();
        let resized = size != self.screen;
        self.ui = scale;
        self.screen = size;
        if resized || self.map_rect() != before {
            let c = self.centre;
            self.set_centre(c);
            let view_w = self.map_rect().w;
            // spec: S-0213 claim 25 (W − 22 after game start; after an in-game change of the view's dimensions W − 16 and the lines cleared)
            self.play.status_view(view_w, !first);
        }
    }

    fn view_size(&self) -> (i32, i32) {
        let r = self.map_rect();
        (r.w.max(1), r.h.max(1))
    }

    /// Sets the view centre, limited by S-0101 F5. The depth reference
    /// does not change (S-0102 F1: one C for the whole game).
    pub fn set_centre(&mut self, requested: (i32, i32)) {
        let f = self.loaded.map.footprint;
        let size = self.view_size();
        // spec: S-0101 claim 8 (every request is limited by F5; the stored centre is the limited one)
        self.centre = limit_centre(requested, size, f.width, f.local);
    }

    /// The view centre (limited by S-0101 F5).
    pub fn centre(&self) -> (i32, i32) {
        self.centre
    }

    /// Scrolls the view by `d` pixels.
    pub fn scroll(&mut self, d: (i32, i32)) {
        if d != (0, 0) {
            let c = (self.centre.0 + d.0, self.centre.1 + d.1);
            self.set_centre(c);
        }
    }

    /// Whether a view-arrow press or release made now is accepted (S-0213
    /// claim 23, S-0201 claim 14): not while orders are blocked, under
    /// `quirks.blocked_arrow_events`.
    pub fn arrow_event_accepted(&self) -> bool {
        let w = &self.play.game.world;
        scroll::arrow_event_accepted(
            glasnost_sim::orders::blocked(w),
            w.rules().quirks.blocked_arrow_events,
        )
    }

    /// One unfrozen cycle's scrolling (S-0101 claims 12 to 14): the edge
    /// request of F9 plus the held arrows' (left, right, up, down), added
    /// before F5 limits the centre. `edge` is false while edge scrolling is
    /// suppressed (no focus, or an order block: S-0201 claim 14).
    pub fn scroll_cycle(&mut self, now_ms: u64, held: [bool; 4], edge: bool) {
        let w = &self.play.game.world;
        // spec: S-0201 claim 14 (edge scrolling is suppressed during an order block; held arrows continue)
        let edge = scroll::edge_enabled(edge, w.tick, w.orders_blocked_through);
        self.edge_live = edge;
        let blocked = self.scroll_blocked();
        // spec: S-0101 claims 16 and 17 (each unfrozen cycle of an active right drag samples the pointer once; order blocks suppress drag updates as they do edge updates)
        let dragging = self.right.filter(|r| r.scrolling && edge).zip(self.pointer);
        self.drag_refused = dragging.map(|_| {
            // spec: S-0101 claim 17 (before the update's displacement, a one-pixel request in each screen direction is tried under claim 8)
            cursor::DRAG_TRIALS.map(&blocked)
        });
        let d = dragging.map_or((0, 0), |(r, p)| {
            let m = self.map_rect();
            // spec: S-0101 F10 (anchor and pointer in view pixels; the full client size; method 0, the default of the scrolling-method preference, which keeps the pointer displaced)
            scroll::drag_request(
                (r.press.x - m.x, r.press.y - m.y),
                (p.x - m.x, p.y - m.y),
                self.screen,
                self.scroll_index,
                scroll::DragMethod::Hold,
            )
            .displacement
        });
        // spec: S-0101 claim 12 (edge scrolling only without a mouse button held)
        let pointer = self
            .pointer
            .filter(|_| edge && self.press.is_none())
            .map(|p| (p.x, p.y));
        let e = self.edge.cycle(
            pointer,
            self.screen,
            self.scroll_index,
            self.loaded.scroll_multiplier,
            now_ms,
            blocked,
        );
        let a = scroll::arrows(held);
        // spec: S-0101 claims 14 and 16 (mouse and arrow requests add before the next view position, subject to F5)
        self.scroll((e.0 + d.0 + a.0, e.1 + d.1 + a.1));
    }

    /// Whether F5 blocks a one-pixel scroll in a screen direction from the
    /// current centre (S-0101 F9; S-0213 claim 24's trial).
    fn scroll_blocked(&self) -> impl Fn((i32, i32)) -> bool + use<> {
        let f = self.loaded.map.footprint;
        let size = self.view_size();
        let centre = self.centre;
        // spec: S-0101 F9 (a one-pixel request in the direction blocked by F5)
        // spec: S-0213 claim 24 (blocked when a one-pixel trial cannot move the view under S-0101's limits)
        move |d: (i32, i32)| {
            limit_centre((centre.0 + d.0, centre.1 + d.1), size, f.width, f.local) == centre
        }
    }

    /// The map view.
    pub fn view(&self) -> View {
        let size = self.view_size();
        View {
            origin: view_origin(self.centre, size),
            size: (size.0 as u32, size.1 as u32),
            // spec: S-0102 F1 (C = 32768 throughout the game, whatever the view)
            reference: DEPTH_REFERENCE,
        }
    }

    /// The scene filter.
    pub fn filter(&self) -> &Filter {
        &self.filter
    }

    /// The light-alpha values of the view after the local player's shroud
    /// covers (S-0110); in a watch game after none.
    pub fn light_alpha(&self) -> Vec<u8> {
        self.light_alpha_of(&self.shroud_covers())
    }

    /// The local player's shroud covers of the view (S-0110), from which
    /// [`Session::light_alpha_of`] gives the light-alpha values.
    pub fn shroud_covers(&self) -> Vec<glasnost_render::shroud::Cover> {
        let Some(house) = self.play.local else {
            // design: watch mode (lead request 2026-11-02, an extra not in the original): no house is played, so no shroud and no fog is drawn anywhere
            return Vec::new();
        };
        cover::covers(
            &self.play.game.world,
            house,
            &self.view(),
            &ShroudQuirks::default(),
        )
    }

    /// The light-alpha values of the view after `covers` (the
    /// [`Session::shroud_covers`] of this frame): [`Session::light_alpha`]
    /// in two parts, so that a caller can keep the values while the covers
    /// stay the same.
    pub fn light_alpha_of(&self, covers: &[glasnost_render::shroud::Cover]) -> Vec<u8> {
        cover::alpha_of(
            covers,
            &self.view(),
            &self.loaded.covers,
            &ShroudQuirks::default(),
        )
    }

    /// The battlefield composed on the CPU, RGBA.
    pub fn map_rgba(&self) -> Vec<u8> {
        self.scene
            .compose_lit(
                &self.view(),
                &self.filter,
                &ComposeQuirks::default(),
                Some(self.light_alpha()),
            )
            .to_rgba()
    }

    /// The radar image (S-0410), RGBA of `hud::RADAR`.
    pub fn radar_rgba(&self) -> Option<Vec<u8>> {
        self.radar
            .as_ref()
            .map(|r| r.rgba(&self.play.game.world, self.play.local))
    }

    /// The watch panel's model of a watch game ([`watch::view`]): each
    /// player house's row, the radar's view outline and what the selected
    /// object is.
    ///
    /// design: watch mode (lead request 2026-11-02, an extra not in the
    /// original): the selected object's lines are its type's name, its
    /// owner's name and its health against its strength.
    pub fn watch_model(&self) -> watch::WatchModel {
        let w = &self.play.game.world;
        let colours = &self.house_colours;
        let houses = watch::house_rows(w, |h| {
            colours
                .get(usize::from(h.0))
                .copied()
                .unwrap_or([255, 255, 255])
        });
        let selected = self
            .play
            .selection
            .first()
            .and_then(|&id| w.obj(id).map(|o| (id, o)))
            .map(|(id, o)| {
                let rules = w.rules();
                let owner = o
                    .owner
                    .and_then(|h| w.houses.get(usize::from(h.0)))
                    .map_or_else(|| "No owner".to_string(), |h| h.name.clone());
                vec![
                    live::type_name(rules, o.kind, o.type_index),
                    owner,
                    format!(
                        "Health {} / {}",
                        o.health,
                        glasnost_sim::combat::damage::strength(w, id)
                    ),
                ]
            })
            .unwrap_or_default();
        watch::WatchModel {
            houses,
            view_box: self.radar_view_box(),
            selected,
        }
    }

    /// The icon frame read for type `t` (palette indices), for checks.
    pub fn icon(&self, t: TypeRef) -> Option<&IndexedImage> {
        self.icons.get(&hud::Entry::Type(t))
    }

    /// The icon frame read for superweapon type `sw`, for checks.
    pub fn sw_icon(&self, sw: u32) -> Option<&IndexedImage> {
        self.icons.get(&hud::Entry::Superweapon(sw))
    }

    /// Registers the icon frames with the painter (once per game).
    pub fn register_art(&mut self, painter: &mut Painter) {
        if let Some(art) = &self.sidebar_art {
            // spec: S-0411 claim 19 (every one of these images drawn with the dedicated sidebar palette)
            let row = painter.add_palette(art.palette);
            let mut ok = std::collections::BTreeSet::new();
            for (k, f) in art.files.iter().enumerate() {
                let Some(f) = f else { continue };
                for (i, img) in f.frames.iter().enumerate() {
                    if let Some(img) = img {
                        let id = sidebar_art::BUTTON_SPRITES + k as u32;
                        if painter.add_sprite(id, i as u32, img, row) {
                            ok.insert((k, i));
                        }
                    }
                }
            }
            self.buttons = sidebar_art::button_art(art, |k, i| ok.contains(&(k, i)));
        }
        let Some(pal) = self.icon_palette else {
            return;
        };
        let row = painter.add_palette(pal);
        for (next, (t, img)) in (ICON_SPRITES..).zip(&self.icons) {
            if painter.add_sprite(next, 0, img, row) {
                self.icon_ids.insert(*t, next);
            }
        }
        for (next, (t, img)) in (ALT_ICON_SPRITES..).zip(&self.alt_icons) {
            if painter.add_sprite(next, 0, img, row) {
                self.alt_icon_ids.insert(*t, next);
            }
        }
    }

    fn world_point(&self, p: Point) -> (i32, i32) {
        let v = self.view();
        let m = self.map_rect();
        (v.origin.0 + p.x - m.x, v.origin.1 + p.y - m.y)
    }

    /// The cell and the selectable object under a screen point of the
    /// battlefield.
    pub fn under(&mut self, p: Point) -> (CellXY, Option<ObjectId>) {
        let (qx, qy) = self.world_point(p);
        let map = &self.loaded.map;
        let q = ProjectionQuirks::default();
        // spec: S-0101 claim 10 (the highest level is read only with quirks.cursor_probe_depth_180 off)
        let max_level = if q.cursor_probe_depth_180 {
            0
        } else {
            map.cells
                .iter()
                .map(|(_, c)| i32::from(c.level))
                .max()
                .unwrap_or(0)
        };
        let cell = cursor_cell(
            qx,
            qy,
            |x, y| {
                map.cells
                    .get(CellXY::new(x, y))
                    .map_or(0, |c| i32::from(c.level))
            },
            max_level,
            &q,
        );
        let top = self.view().origin.1;
        let code = self.scene.object_at(qx, qy, top, &self.filter);
        let id = code.and_then(|c| {
            self.with_live(|live, _, _| live.object_of_code(c))
                .flatten()
        });
        (CellXY::new(cell.0, cell.1), id)
    }

    /// Debug diagnostic: every drawn item covering the screen point `p`
    /// (see `Live::trace_point`); empty without a live game.
    pub fn trace_at(&mut self, p: Point) -> Vec<String> {
        let (qx, qy) = self.world_point(p);
        let top = self.view().origin.1;
        let statics = self.loaded.built.objects.clone();
        self.with_live(|live, _, scene| live.trace_point(scene, &statics, qx, qy, top))
            .unwrap_or_default()
    }

    /// The pointer moved.
    pub fn pointer_moved(&mut self, p: Point) {
        self.pointer_at(p);
        let mode = self.play.game.world.rules().mode;
        if let Some(d) = self.drag.as_mut() {
            d.motion(p, mode);
        }
        if let Some(r) = self.right.as_mut() {
            r.motion(p);
        }
    }

    /// The pointer is at `p`: a change of place is a movement, which hides
    /// a name tip and starts its wait again (S-0111 claim 20).
    fn pointer_at(&mut self, p: Point) {
        if self.pointer != Some(p) {
            // spec: S-0111 claim 20 (every pointer movement, of any distance, hides a showing tip and starts the wait again)
            self.name_tip.moved(self.clock_ms);
        }
        self.pointer = Some(p);
    }

    /// The pointer left the window.
    pub fn pointer_left(&mut self) {
        self.pointer = None;
        self.press = None;
        self.drag = None;
        self.right = None;
        self.view_button_held = false;
        // spec: S-0111 claim 20 (r40: the pointer leaving the window is not an event of its own; the last movement restarts the wait as usual, and a wait that ends with the pointer outside the window shows nothing and uses up the rest's choice)
    }

    /// The middle button went down or up: it does nothing in the game but
    /// hide the name tip and stop its wait (S-0111 claim 20).
    pub fn middle_button(&mut self, p: Point) {
        self.pointer_at(p);
        // spec: S-0111 claim 20 (pressing or releasing the middle mouse button hides the tip and stops the wait)
        self.name_tip.button();
    }

    /// A mouse-wheel event, anywhere over the game (S-0411 claim 18).
    pub fn wheel(&mut self, delta: i32) {
        if self.paused {
            return;
        }
        let rows = self.layout().rows();
        self.play.wheel_icons(delta, rows);
    }

    /// The sidebar target under a screen pixel.
    fn hud_at(&self, p: Point) -> Option<HudHit> {
        self.targets.hit(self.ui.to_ui(p))
    }

    /// A button went down.
    pub fn pointer_down(&mut self, b: Button, p: Point) {
        self.pointer_at(p);
        // spec: S-0111 claim 20 (pressing the left or right mouse button hides the tip and stops the wait)
        self.name_tip.button();
        self.press = Some((p, b));
        // The pointer takes over from the keyboard.
        self.hud_focus.shown = false;
        self.pause_focus.shown = false;
        let on_map = self.hud_at(p) == Some(HudHit::Map);
        // spec: S-0111 claim 20 (Blocked: a left or right button pressed over the battlefield view is held; a held middle button does not block)
        // design: Q-2736 (S-0111 Open questions leave a button pressed in the route mode and held undecided: it blocks like any other held left or right button pressed over the view)
        self.view_button_held = on_map && matches!(b, Button::Left | Button::Right);
        if on_map {
            self.aim = None;
        }
        // spec: S-0214 claim 18 (a left-button press in the battlefield can start a selection drag when sell, repair, power, building-placement and superweapon targeting modes are off)
        self.drag = (b == Button::Left && !self.paused && self.play.tool == Tool::Select && on_map)
            .then(|| band::Drag::armed(p));
        // spec: S-0214 claim 27 (an accepted right-button press in the battlefield anchors a possible scroll drag at that pixel)
        self.right =
            (b == Button::Right && !self.paused && on_map).then(|| band::RightDrag::armed(p));
        self.route.pressed = None;
        // spec: S-0213 claim 16d, When the tests are made (revision 84, Q-2193: only left-button presses are tested; a right press or right click is never tested for kind, whatever is selected, and acts as claim 16h gives) and claim 16h (a right click while the key is held acts as outside route mode)
        if self.route.held && b == Button::Left && !self.paused && on_map {
            self.route_press(p);
        }
    }

    /// A route-mode left press on the battlefield (S-0213 claims 16b, 16d
    /// and 16h): claim 16d's kind test first, then what the press does with
    /// the points whose click areas hold the cursor.
    fn route_press(&mut self, p: Point) {
        let Some(local) = self.play.local else {
            // design: watch mode (lead request 2026-11-02, an extra not in the original): no route is laid where no house is played
            return;
        };
        let under = self.points_under(p);
        let w = &self.play.game.world;
        let sel = self.play.selection.clone();
        if let Some(m) = glasnost_sim::routes::press_kind_refusal(w, local, &sel, &under) {
            // spec: S-0213 claim 16d, When the tests are made (the kind test is made when the button is pressed, whatever the click would do; a refused press ends the click: the selection is kept, and no order, point or band box follows)
            self.press = None;
            self.drag = None;
            self.route_refused(m);
            return;
        }
        let press = glasnost_sim::routes::press(w, local, &sel, &under);
        if press != glasnost_sim::routes::Press::Ordinary {
            // spec: S-0213 claim 16h (a press that takes a point does that and nothing else) and claim 16b (a press refused with the invalid-point message gives no order)
            self.drag = None;
        }
        self.route.pressed = Some(press);
    }

    /// The route points of any house whose click areas hold screen pixel
    /// `p` (S-0213 claim 16b, *Click areas*), in point order.
    fn points_under(&self, p: Point) -> Vec<u32> {
        let (qx, qy) = self.world_point(p);
        let w = &self.play.game.world;
        w.routes
            .points
            .iter()
            .filter(|(_, pt)| {
                let (sx, sy) = place_screen(w, pt);
                glasnost_sim::routes::in_click_area(qx - sx, qy - sy)
            })
            .map(|(&n, _)| n)
            .collect()
    }

    /// The window pixel at the screen position of route point `n`'s place
    /// (S-0213 claim 16b, *Click areas*).
    pub fn route_point_screen(&self, n: u32) -> Option<Point> {
        let w = &self.play.game.world;
        let (sx, sy) = place_screen(w, w.routes.points.get(&n)?);
        let v = self.view();
        let m = self.map_rect();
        Some(Point::new(sx - v.origin.0 + m.x, sy - v.origin.1 + m.y))
    }

    /// The point a route-mode press at the pointer would take, for its
    /// highlight (S-0213 claim 16b).
    fn route_hover(&self) -> Option<u32> {
        let p = self.pointer.filter(|&p| self.map_rect().contains(p))?;
        let local = self.play.local?;
        let under = self.points_under(p);
        glasnost_sim::routes::highlighted(
            &self.play.game.world,
            local,
            &self.play.selection,
            &under,
        )
    }

    /// A button went up: a click or the end of a drag.
    pub fn pointer_up(&mut self, b: Button, p: Point, mods: Modifiers) -> SessionEvent {
        self.pointer_at(p);
        // spec: S-0111 claim 20 (releasing the left or right mouse button hides the tip and stops the wait)
        self.name_tip.button();
        self.view_button_held = false;
        let drag = self.drag.take();
        let right = self.right.take();
        let Some((start, pb)) = self.press.take() else {
            return SessionEvent::None;
        };
        if pb != b {
            return SessionEvent::None;
        }
        if self.paused {
            return match self.pause_targets.hit(self.ui.to_ui(p)) {
                Some(hit) => self.pause_hit(hit),
                None => SessionEvent::None,
            };
        }
        let dragged = drag.is_some_and(|d| d.active) && b == Button::Left;
        let scrolled = right.is_some_and(|r| r.scrolling) && b == Button::Right;
        let pressed = self.route.pressed.take();
        if let Some(press) = pressed
            && self.route.held
            && b == Button::Left
            && !dragged
            && self.hud_at(start) == Some(HudHit::Map)
            && self.route_release(press, p, mods)
        {
            self.route_selection_changed();
            return SessionEvent::None;
        }
        // spec: S-0213 claim 16h (a right click made while the key is held and no object is selected clears the point selection, only when its release is less than 4 screen pixels from its press on each axis)
        let short = (p.x - start.x).abs() < ROUTE_RIGHT_CLICK_BOUND
            && (p.y - start.y).abs() < ROUTE_RIGHT_CLICK_BOUND;
        if b == Button::Right
            && !scrolled
            && self.route.held
            && self.play.selection.is_empty()
            && short
            && self.hud_at(start) == Some(HudHit::Map)
        {
            self.route.point = None;
        }
        match self.hud_at(start) {
            // spec: S-0214 claim 27 (a scroll drag's release neither clears the selection, nor cancels a targeting mode, nor gives an order)
            // spec: S-0411 claim 10 (a classified right drag and release keep placement mode and the ready item: no cancellation, no refund)
            Some(HudHit::Map) if scrolled => {}
            Some(HudHit::Map) if dragged && self.play.tool == Tool::Select => {
                let end = band::limit(p, self.map_rect());
                let inside = self.box_candidates(start, end);
                let click = if self.type_select.held {
                    // spec: S-0214 claims 29 and 32 (a held drag expands the types of every in-box candidate, of any house and kind, in first in-box encounter order, each type in the pool's order, matching objects outside the box included; it reads the reach the last tap left and does not change it) and S-0701 claim 17 (silent)
                    let pool = self.type_pool(self.type_select.reach == Reach::Map);
                    self.play.type_drag(&inside, &pool, mods.force_select)
                } else {
                    self.play.select_box(&inside, mods.force_select)
                };
                // spec: S-0214 claim 19 (an unmodified box without a live candidate continues as the ordinary release click at the final pixel)
                if click {
                    let (cell, object) = self.under(end);
                    self.play.click_map(b, object, cell, mods);
                    self.show_markers();
                }
            }
            Some(HudHit::Map) if !dragged => {
                let (cell, object) = self.under(p);
                match self.type_select_click(b, object, cell, mods) {
                    Some((t, held)) => {
                        // spec: S-0214 claims 12 and 32 (with the role held, select and toggle select act on the clicked object's type as a whole within the reach the last tap left; silent, and the reach is not changed)
                        let pool = self.type_pool(self.type_select.reach == Reach::Map);
                        match held {
                            HeldClick::Select => {
                                // spec: S-0214 claim 32 (select clears the selection, then adds every own object of exactly the clicked object's type that can be selected within the reach; the clicked object joins only if it is own)
                                self.play.clear_selection();
                                self.play.expand_types(&[t], &pool, false);
                            }
                            HeldClick::Add => {
                                // spec: S-0214 claim 32 (toggle select on an unselected object makes the same additions and keeps the selection)
                                self.play.expand_types(&[t], &pool, false);
                            }
                            HeldClick::Remove => {
                                // spec: S-0214 claim 32 (toggle select on a selected object removes every selected own object of that type within the reach; with reach screen, selected ones off the screen stay)
                                self.play.expand_types(&[t], &pool, true);
                            }
                        }
                    }
                    None => self.play.click_map(b, object, cell, mods),
                }
                self.show_markers();
            }
            Some(HudHit::Radar) if b == Button::Left => {
                let l = self.layout();
                // The radar's pixel, in the logical pixels it is drawn in.
                let u = self.ui.to_ui(p);
                if let Some(cell) = self.radar.as_ref().and_then(|r| {
                    r.cell_at(&self.play.game.world, u.x - l.radar.x, u.y - l.radar.y)
                }) && !self.play.click_radar(cell)
                {
                    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
                    self.set_centre(project_lifted(x, y, 0));
                }
            }
            // spec: S-0411 claim 18 (a page button acts on release over the button after a press there)
            Some(HudHit::Scroll(d)) if self.hud_at(p) == Some(HudHit::Scroll(d)) => {
                self.hud_hit(HudHit::Scroll(d), b, false);
            }
            Some(hit @ (HudHit::Tab(_) | HudHit::Cameo(_) | HudHit::Mode(_) | HudHit::Options)) => {
                self.hud_hit(hit, b, mods.force_select)
            }
            _ => {}
        }
        self.route_selection_changed();
        SessionEvent::None
    }

    /// Any selection of an object clears the point selection (S-0213
    /// claims 16e and 16h).
    fn route_selection_changed(&mut self) {
        // spec: S-0213 claim 16e, The delete key (any selection of an object, by any means, clears the point selection) and claim 16h (while any object is selected, no point is selected)
        if !self.play.selection.is_empty() {
            self.route.point = None;
        }
    }

    /// The clicked object of a left click whose action is *select* or
    /// *toggle select* while the type-select role is held, and what the
    /// held click does (S-0214 claims 12 and 32).
    fn type_select_click(
        &self,
        b: Button,
        object: Option<ObjectId>,
        cell: CellXY,
        mods: Modifiers,
    ) -> Option<(ObjectId, HeldClick)> {
        if b != Button::Left || !self.type_select.held || self.play.tool != Tool::Select {
            return None;
        }
        let on = self.play.target(object, cell);
        let ClickOn::Object(t) = on else {
            return None;
        };
        // design: watch mode (lead request 2026-11-02, an extra not in the original): the type-select role selects nothing more where no house is played
        let local = self.play.local?;
        let w = &self.play.game.world;
        let sel = &self.play.selection;
        // spec: S-0214 claim 12 (toggle select acts only when the first selected object is the acting house's own, and otherwise acts as select)
        let first_own = sel
            .first()
            .is_some_and(|&f| w.obj(f).is_some_and(|o| o.owner == Some(local)));
        match targeting::cursor(w, local, sel, on, mods) {
            targeting::Action::Select => Some((t, HeldClick::Select)),
            targeting::Action::ToggleSelect if !first_own => Some((t, HeldClick::Select)),
            targeting::Action::ToggleSelect if sel.contains(&t) => Some((t, HeldClick::Remove)),
            targeting::Action::ToggleSelect => Some((t, HeldClick::Add)),
            _ => None,
        }
    }

    /// A sidebar control pressed with button `b` (a click, or the
    /// keyboard's press of the focused control).
    fn hud_hit(&mut self, hit: HudHit, b: Button, shift: bool) {
        if self.play.watching() && hit != HudHit::Options {
            // design: watch mode (lead request 2026-11-02, an extra not in the original): the watch panel has no tab, cameo or mode button; only the options button acts
            return;
        }
        match hit {
            HudHit::Tab(i) => self.play.tab_click(i),
            HudHit::Cameo(n) => self.play.cameo_click(n, b, shift),
            HudHit::Scroll(d) => {
                let rows = self.layout().rows();
                self.play.scroll_icons(d, rows);
            }
            HudHit::Mode(m) => self.play.mode_click(m),
            HudHit::Options => {
                self.open_options();
            }
            HudHit::Map | HudHit::Radar => {}
        }
    }

    /// An options-menu control pressed (a click or the keyboard).
    fn pause_hit(&mut self, hit: PauseHit) -> SessionEvent {
        match hit {
            PauseHit::Resume => {
                self.close_options();
                SessionEvent::None
            }
            // spec: S-0201 claim 17 (no speed control once the local house is defeated, in a network game)
            PauseHit::Speed(_) if !self.speed_offered() => SessionEvent::None,
            PauseHit::Speed(d) => {
                // spec: S-0201 claims 8 and 17 (the options menu's speed control; the change is sent when the menu closes)
                self.menu_speed = hud::step_speed(self.menu_speed, d);
                SessionEvent::None
            }
            hit @ (PauseHit::Surrender | PauseHit::SurrenderLeave)
                if !self.leave_choices().contains(&hit) =>
            {
                SessionEvent::None
            }
            PauseHit::Surrender => {
                // spec: S-0201 claim 15 (in a network game, after resigning through the menu it does not open again until the house is defeated; the choice that keeps the player watching)
                self.resigned = true;
                self.close_options();
                self.play.surrender();
                SessionEvent::None
            }
            PauseHit::SurrenderLeave => {
                // spec: S-0201 claim 15 (the choice that sends the surrender order and then leaves the game)
                // design: the client leaves once the lockstep has carried its own surrender order out, so that every peer applies the surrender at once rather than at the drop's grace period
                self.resigned = true;
                self.leave_after_surrender = true;
                self.close_options();
                self.play.surrender();
                SessionEvent::None
            }
            PauseHit::Settings if !self.network => SessionEvent::Settings,
            PauseHit::Settings => SessionEvent::None,
            PauseHit::Leave => SessionEvent::Leave,
        }
    }

    /// A keyboard step on the open options menu (our own keys, M8 step
    /// C3).
    pub fn pause_key(&mut self, n: Nav) -> SessionEvent {
        match self.pause_focus.nav(&self.pause_targets, n) {
            Some(hit) => self.pause_hit(hit),
            None => SessionEvent::None,
        }
    }

    /// Whether the keyboard cell aims a placement or a superweapon now.
    pub fn aiming(&self) -> bool {
        self.aim.is_some() && matches!(self.play.tool, Tool::Place(_) | Tool::Superweapon { .. })
    }

    /// Whether the tool places a building or aims a superweapon, so that
    /// the keyboard cell can aim it.
    pub fn tool_aims(&self) -> bool {
        !self.paused && matches!(self.play.tool, Tool::Place(_) | Tool::Superweapon { .. })
    }

    /// The cell under the battlefield's centre.
    fn centre_cell(&mut self) -> CellXY {
        let c = self.map_rect().centre();
        self.under(c).0
    }

    /// Our own keyboard play of the sidebar and keyboard aiming (M8 step
    /// C3). Every press goes through the same paths as the pointer's
    /// clicks, so orders reach the replay alike.
    pub fn sidebar_key(&mut self, k: SideKey) {
        if self.paused {
            return;
        }
        if !self.aiming() {
            self.aim = None;
        }
        match k {
            SideKey::Step(forward) => self.sidebar_step(forward),
            SideKey::Press { .. } | SideKey::Place if self.aiming() => {
                let cell = self.aim.expect("aiming");
                self.play
                    .click_map(Button::Left, None, cell, Modifiers::default());
                self.show_markers();
                if !self.aiming() {
                    self.aim = None;
                }
            }
            SideKey::Place => {}
            SideKey::Press { shift } => {
                if let Some(hit) = self.hud_focus.nav(&self.targets, Nav::Activate) {
                    let before = self.play.tool;
                    self.hud_hit(hit, Button::Left, shift);
                    // A placement or an aim started from the keyboard aims
                    // with the keyboard cell.
                    let aims = matches!(self.play.tool, Tool::Place(_) | Tool::Superweapon { .. });
                    if aims && self.play.tool != before {
                        self.aim = Some(self.centre_cell());
                    }
                }
            }
            SideKey::Back if self.aiming() => {
                let cell = self.aim.take().expect("aiming");
                self.play
                    .click_map(Button::Right, None, cell, Modifiers::default());
            }
            SideKey::Back => {
                if let Some(hit @ HudHit::Cameo(_)) = self.hud_focus.target(&self.targets) {
                    self.hud_hit(hit, Button::Right, false);
                }
            }
            SideKey::Tab(d) => self.sidebar_tab(d),
            SideKey::Cell(dx, dy) => {
                if !matches!(self.play.tool, Tool::Place(_) | Tool::Superweapon { .. }) {
                    return;
                }
                let c = match self.aim {
                    Some(c) => c,
                    None => self.centre_cell(),
                };
                let to = aim_step(c, dx, dy);
                self.aim = Some(to);
                self.keep_in_view(to);
            }
        }
    }

    /// Moves the sidebar focus; past the last or first shown icon the grid
    /// moves one row, silently, as the wheel does (S-0411 claim 18).
    fn sidebar_step(&mut self, forward: bool) {
        if let Some(HudHit::Cameo(n)) = self.hud_focus.target(&self.targets)
            && let Some(local) = self.play.local
        {
            let w = &self.play.game.world;
            let count = hud::entries(w, local, self.play.shown_tab()).len();
            let next = if forward { n + 1 } else { n.wrapping_sub(1) };
            if next < count && self.targets.rect_of(&HudHit::Cameo(next)).is_none() {
                let rows = self.layout().rows();
                self.play.wheel_icons(if forward { -1 } else { 1 }, rows);
                self.hud_focus.at = Some(HudHit::Cameo(next));
                self.hud_focus.shown = true;
                return;
            }
        }
        let n = if forward { Nav::Next } else { Nav::Prev };
        self.hud_focus.nav(&self.targets, n);
    }

    /// Shows the next or previous tab that has icons and focuses its first
    /// icon.
    fn sidebar_tab(&mut self, d: i32) {
        // design: watch mode (lead request 2026-11-02, an extra not in the original): the watch panel has no tabs
        let Some(local) = self.play.local else {
            return;
        };
        let w = &self.play.game.world;
        let shown = self.play.shown_tab() as i32;
        let tabs = hud::TABS as i32;
        let found = (1..=tabs)
            .map(|k| (shown + d.signum() * k).rem_euclid(tabs) as usize)
            .find(|&i| !hud::entries(w, local, i).is_empty());
        if let Some(i) = found {
            self.play.tab_click(i);
            self.hud_focus.at = Some(HudHit::Cameo(0));
            self.hud_focus.shown = true;
        }
    }

    /// Centres the view on `cell` when its centre is not well inside the
    /// battlefield.
    fn keep_in_view(&mut self, cell: CellXY) {
        let m = self.map_rect();
        let v = self.view();
        let level = self
            .play
            .game
            .world
            .cells
            .cell(cell)
            .map_or(0, |x| i32::from(x.level));
        let (tx, ty) = cell_top(cell.x, cell.y);
        // The cell's centre: half a cell (S-0101 F1's 30 × 15 half sizes) below its top corner.
        let (wx, wy) = (tx, ty + 15 - TERRAIN_LIFT * level); // spec: S-0101 F1 (a cell's top corner projects to (30(x − y), 15(x + y)); the diamond is 60 × 30)
        let p = Point::new(wx - v.origin.0 + m.x, wy - v.origin.1 + m.y);
        // nocite: our own keyboard aiming (the cell stays a cell's width inside the view)
        if !m.inset(60).contains(p) {
            self.set_centre((wx, wy));
        }
    }

    /// Whether the sidebar's keyboard focus mark is on `hit` (tests).
    pub fn sidebar_focus(&self) -> Option<HudHit> {
        self.hud_focus.target(&self.targets)
    }

    /// The sidebar's targets of the last frame, in logical pixels (tests).
    pub fn hud_targets(&self) -> &Targets<HudHit> {
        &self.targets
    }

    /// The options menu's targets of the last frame, in logical pixels
    /// (tests).
    pub fn pause_targets(&self) -> &Targets<PauseHit> {
        &self.pause_targets
    }

    /// The options menu's keyboard focus (tests).
    pub fn pause_focus(&self) -> Option<PauseHit> {
        self.pause_focus.target(&self.pause_targets)
    }

    /// The keyboard cell, while it aims (tests and the drawing).
    pub fn aim_cell(&self) -> Option<CellXY> {
        self.aim.filter(|_| self.aiming())
    }

    /// The live box candidates whose box point is inside the box from
    /// `press` to `end` (S-0214 claim 19, Terms *box point*), out of the
    /// frame's admitted screen candidates (claims 20, 24 and 25, F7): the
    /// in-play buildings on the map whose type is not invisible in the game
    /// and whose point is in the view, in creation order, then the drawn
    /// mobile objects whose point is within 32 pixels of it, in drawing
    /// order, cut at 500. Owner and eligibility play no part here. Points
    /// are the position projected with height lift, aircraft at their
    /// lifted origin.
    fn box_candidates(&mut self, press: Point, end: Point) -> Vec<ObjectId> {
        let m = self.map_rect();
        let (press, end) = (
            Point::new(press.x - m.x, press.y - m.y),
            Point::new(end.x - m.x, end.y - m.y),
        );
        self.screen_candidates()
            .into_iter()
            .filter(|&(_, q)| band::inside(press, end, q))
            .map(|(id, _)| id)
            .collect()
    }

    /// The screen candidates of S-0214 claims 24 and 25, admitted by claim
    /// 20, in candidate order, each with its box point relative to the
    /// battlefield view's top-left corner.
    fn screen_candidates(&mut self) -> Vec<(ObjectId, Point)> {
        let drawn = self
            .with_live(|live, inputs, _| live.drawn_mobiles(inputs))
            .unwrap_or_default();
        let w = &self.play.game.world;
        let v = self.view();
        let m = self.map_rect();
        // spec: S-0214 Terms (box point: the drawing origin, the position projected with world-height lift; S-0101 F1) and F7 (relative to the battlefield view's top-left corner)
        let point = |o: &glasnost_sim::world::Object| {
            let (qx, qy) = project_lifted(o.pos.x, o.pos.y, o.pos.z);
            Point::new(qx - v.origin.0, qy - v.origin.1)
        };
        let invisible = |o: &glasnost_sim::world::Object| {
            w.rules()
                .buildings
                .get(o.type_index as usize)
                .is_some_and(|b| b.raw.invisible_in_game.unwrap_or(false))
        };
        // spec: S-0214 claim 24 (an in-play building on the map whose type is not invisible in the game, its point in the unexpanded view; not filtered by owner, eligibility, shroud or health; terrain objects are no candidates) and claim 25 (earlier-created buildings first)
        let buildings = w
            .objects
            .values()
            .filter(|o| o.kind == SimKind::Building && o.on_map && !invisible(o))
            .map(|o| (o.id, point(o)))
            .filter(|&(_, q)| band::building_point_in(q, m.w, m.h));
        // spec: S-0214 claim 24 (a mobile object taking part in the frame's body drawing, its point within F7's expanded view) and claim 25 (in the current drawing order)
        let mobile = drawn
            .iter()
            .filter_map(|&id| w.obj(id))
            .map(|o| (o.id, point(o)))
            .filter(|&(_, q)| band::mobile_point_in(q, m.w, m.h));
        // spec: S-0214 claims 20 and 25 (500 screen candidates, buildings before mobile objects, places consumed whether eligible or not)
        band::admit(buildings, mobile)
    }

    /// The objects a type selection draws from (S-0214 claims 12 and
    /// 29): the screen candidates in candidate order, or with `map` every
    /// object in creation order.
    fn type_pool(&mut self, map: bool) -> Vec<ObjectId> {
        if map {
            // spec: S-0214 claim 29 (the whole-map expansion follows creation order among surviving objects)
            self.play.game.world.objects.keys().copied().collect()
        } else {
            // spec: S-0214 claim 12 (every object of the type that is on the screen) and claim 29 (in the candidate order of claim 25)
            self.screen_candidates()
                .into_iter()
                .map(|(id, _)| id)
                .collect()
        }
    }

    /// The type-select role's key went down or up (S-0214 claim 32).
    /// A release at most [`TYPE_SELECT_TAP_MS`] after the press is a tap;
    /// a key repeat while held is no new press.
    pub fn type_select_key(&mut self, down: bool) {
        let now = self.clock_ms;
        if down {
            // spec: S-0214 claim 32 (the press time is that of the first press: a key's automatic repeat while it is held does not restart it)
            if self.type_select.held || self.paused {
                return;
            }
            self.type_select.held = true;
            self.type_select.pressed = now;
            return;
        }
        if !self.type_select.held {
            return;
        }
        self.type_select.held = false;
        // spec: S-0214 claim 32 and Constants (a tap: release time minus press time, whole milliseconds, at most 500; a longer hold does nothing at its release; after the acting player's defeat a tap does nothing)
        let tap = (0..=TYPE_SELECT_TAP_MS).contains(&(now - self.type_select.pressed));
        // design: watch mode (lead request 2026-11-02, an extra not in the original): no acting player, so a tap does nothing
        let Some(local) = self.play.local else {
            return;
        };
        if !tap || self.paused || victory::defeated(&self.play.game.world, local) {
            return;
        }
        // spec: S-0214 claim 32 (a tap keeps the reach only when nothing was added to the selection and it was not cleared since the last tap; otherwise it first sets it to screen)
        if self.play.sel_reset {
            self.type_select.reach = Reach::Screen;
        }
        let screen: Vec<ObjectId> = if self.type_select.reach == Reach::Screen {
            self.screen_candidates()
                .into_iter()
                .map(|(id, _)| id)
                .collect()
        } else {
            Vec::new()
        };
        self.type_select.reach = self.play.type_tap(&screen, self.type_select.reach);
        self.route_selection_changed();
    }

    /// The type-select role's key is let go without a release, as when
    /// the window loses the focus: no tap (our design; the release may
    /// never come).
    pub fn type_select_cancel(&mut self) {
        self.type_select.held = false;
    }

    /// Whether the type-select role is held (S-0214 claim 32).
    pub fn type_select_held(&self) -> bool {
        self.type_select.held
    }

    /// The type-select reach as the last tap left it (S-0214 claim 32).
    pub fn type_select_reach(&self) -> Reach {
        self.type_select.reach
    }

    /// A control-group key press (S-0214 claim 33) for group `g`.
    pub fn group_key(&mut self, g: usize, cmd: GroupCommand) {
        // spec: S-0214 claim 33 (while the options menu is open the group keys do nothing)
        if self.paused || g >= GROUPS {
            return;
        }
        // design: watch mode (lead request 2026-11-02, an extra not in the original): no house's groups, so the group keys do nothing
        let Some(local) = self.play.local else {
            return;
        };
        let now = self.clock_ms;
        let centre = match cmd {
            GroupCommand::Make => {
                // spec: S-0214 claim 33 (make group ends no mode and neither starts nor restarts the 800 ms)
                self.play.make_group(g);
                None
            }
            GroupCommand::Add => {
                self.play.add_group(g);
                None
            }
            GroupCommand::Centre => self.play.centre_group(g),
            GroupCommand::Select => {
                let members = self.play.group(g);
                let w = &self.play.game.world;
                let mut own: Vec<ObjectId> = self
                    .play
                    .selection
                    .iter()
                    .copied()
                    .filter(|&id| w.obj(id).is_some_and(|o| o.owner == Some(local)))
                    .collect();
                own.sort();
                // spec: S-0214 claim 33 (a repeat: the last select-group press that was not a repeat was for group N, came less than 800 ms before, and the selection's own objects are exactly the members)
                let repeat = self
                    .last_group
                    .is_some_and(|(h, t)| h == g && (0..GROUP_REPEAT_MS).contains(&(now - t)))
                    && own == members;
                if repeat {
                    // spec: S-0214 claim 33 (a repeat leaves the selection as it is, centres the view as centre group does and does not restart the 800 ms)
                    self.play.selection_centre()
                } else {
                    self.last_group = Some((g, now));
                    self.play.select_group(g);
                    None
                }
            }
        };
        // spec: S-0214 claim 33 and F9 (the view is centred on the group centre point; when the selection is empty the view stays)
        if let Some((x, y, z)) = centre {
            self.set_centre(project_lifted(x, y, z));
        }
        self.route_selection_changed();
    }

    /// A key command (S-0213 table T2).
    pub fn key(&mut self, cmd: KeyCommand) {
        if self.paused {
            return;
        }
        if self.route.held {
            // spec: S-0213 table T2 (while route mode is held, guard, stop, scatter and deploy issue no orders and add no route point; each is refused with its own message and a sound)
            let m = match cmd {
                KeyCommand::Guard => "MSG:PlanningModeNoGuardArea",
                KeyCommand::Stop => "MSG:PlanningModeNoStop",
                KeyCommand::Scatter => "MSG:PlanningModeNoScatter",
                KeyCommand::Deploy => "MSG:PlanningModeNoDeploy",
            };
            self.route_refused(m);
            return;
        }
        self.play.key(cmd);
    }

    /// The route-mode key went down or up (S-0213 claims 16 and 22):
    /// held, clicks build routes; released, the routes are committed.
    pub fn route_key(&mut self, down: bool) {
        if self.play.watching() {
            // design: watch mode (lead request 2026-11-02, an extra not in the original): no house is played, so route mode is never entered
            return;
        }
        if down {
            // spec: S-0213 claim 16h, End of route mode (after the local player's house is defeated, route mode cannot be entered again)
            if !self.paused && !self.route.ended {
                self.route.held = true;
                self.play.route_held = true;
            }
            return;
        }
        if !self.route.held {
            return;
        }
        self.route.held = false;
        self.play.route_held = false;
        self.route.pressed = None;
        // spec: S-0213 claim 16e, The delete key (releasing the key clears the point selection)
        self.route.point = None;
        // spec: S-0213 claim 16a (releasing the key sends one commit order for the local player's house)
        self.play
            .issue(vec![glasnost_sim::orders::Order::RouteCommit]);
    }

    /// Route mode after a tick (S-0213 claims 16e and 16h): the local
    /// house's defeat ends it for good, with the commit, and a point delete
    /// of the local house selects the point after the deleted one.
    fn route_after_tick(&mut self) {
        // design: watch mode (lead request 2026-11-02, an extra not in the original): route mode is never entered, and no house's route notes are shown
        let Some(local) = self.play.local else {
            return;
        };
        let w = &self.play.game.world;
        if !self.route.ended && victory::defeated(w, local) {
            // spec: S-0213 claim 16h, End of route mode (route mode ends, with the commit, when the local player's house is defeated, after which it cannot be entered again)
            self.route.ended = true;
            self.route_key(false);
        }
        let mut shown: Vec<&'static str> = Vec::new();
        for e in &self.play.effects {
            let glasnost_sim::combat::effects::Effect::Route { house, note } = e else {
                continue;
            };
            if *house != local {
                continue;
            }
            let label = match note {
                // spec: S-0213 claim 16b (for the local player's own orders a refused point add shows its case's message; an added terminating, continual or 16th point shows that case's message as information)
                glasnost_sim::routes::RouteNote::Refused(m)
                | glasnost_sim::routes::RouteNote::Added { info: Some(m) } => *m,
                // spec: S-0213 claim 16c and Import mapping (closing a loop shows the loop-made information message)
                glasnost_sim::routes::RouteNote::Looped => "MSG:PlanningModeLoopHelp",
                _ => continue,
            };
            // design: one message per label and tick; the orders of one click each give the note
            if !shown.contains(&label) {
                shown.push(label);
            }
        }
        for label in shown {
            self.play.notice(label);
        }
        let w = &self.play.game.world;
        let quirk = w.rules().quirks.delete_selects_after_release;
        let next = self.play.effects.iter().rev().find_map(|e| match e {
            glasnost_sim::combat::effects::Effect::Route {
                house,
                note: glasnost_sim::routes::RouteNote::Selected(n),
            } if *house == local => Some(*n),
            _ => None,
        });
        // spec: S-0213 claim 16e, The delete key (when a point-delete order of the local player's house is applied, the point after the deleted one becomes the selected point; a point delete applied after the key's release selects it too) and Edge cases (quirks.delete_selects_after_release off: one applied while the key is not held selects no point)
        if let Some(n) = next
            && self.play.selection.is_empty()
            && !self.route.ended
            && (self.route.held || quirk)
        {
            self.route.point = Some(n);
        }
    }

    /// Whether the route-mode key is held (S-0213 claim 16).
    pub fn route_held(&self) -> bool {
        self.route.held
    }

    /// The selected route point (S-0213 claim 16e).
    pub fn route_point(&self) -> Option<u32> {
        self.route.point
    }

    /// The delete key (S-0213 claim 16e): a point-delete order for the
    /// selected route point.
    pub fn delete_key(&mut self) {
        if self.paused {
            return;
        }
        if let Some(point) = self.route.point.take() {
            // spec: S-0213 claim 16e, The delete key (the delete key sends a point-delete order for the selected point whenever a point is selected, whether or not the route-mode key is held)
            self.play
                .issue(vec![glasnost_sim::orders::Order::RouteDelete { point }]);
        }
    }

    /// A route-mode refusal (S-0213 claim 16): its message and the
    /// game-wide refusal sound, one shared cue per tick.
    fn route_refused(&mut self, label: &str) {
        let tick = self.play.game.world.tick;
        // spec: S-0213 claim 16 and S-0701 claim 24 (refusal notices before the next tick completes share one message-and-sound cue; a completed tick permits a new one)
        if self.route.cue_tick == Some(tick) {
            return;
        }
        self.route.cue_tick = Some(tick);
        self.play.notice(label);
        self.play.cues.push(glasnost_audio::events::UiCue::StripEnd);
    }

    /// The release of a route-mode left press on the battlefield (S-0213
    /// claims 16b to 16e and 16h). Returns whether route mode handled it;
    /// otherwise it is the release as outside route mode: an ordinary
    /// click that selects, toggles the selection or does nothing, or a
    /// tool's click.
    fn route_release(
        &mut self,
        press: glasnost_sim::routes::Press,
        p: Point,
        mods: Modifiers,
    ) -> bool {
        use glasnost_sim::routes::{Press, RouteClick};
        // design: watch mode (lead request 2026-11-02, an extra not in the original): route mode is never entered, so the release is an ordinary one
        let Some(local) = self.play.local else {
            return false;
        };
        match press {
            Press::Take(n) if self.play.selection.is_empty() => {
                if self.play.game.world.routes.points.contains_key(&n) {
                    self.route.point = Some(n);
                    // spec: S-0213 claim 16e (with no object selected, a press that takes a point selects it and shows an information message)
                    self.play.notice("MSG:PlanningModeDeleteHelp");
                }
                true
            }
            Press::Take(n) => {
                let w = &self.play.game.world;
                let sel = self.play.selection.clone();
                // spec: S-0213 claims 16b and 16d (a press that takes a point sends its join and loop-close orders, which meet no test of claim 16d)
                let orders = glasnost_sim::routes::press_orders(w, local, &sel, n);
                self.play.issue(orders);
                true
            }
            Press::Refused => {
                // spec: S-0213 claim 16b, Click areas (the point-click test refuses the point: the press is refused with the invalid-point message)
                self.route_refused("MSG:PlanningModeInvalidNodeX");
                true
            }
            // spec: S-0213 claim 16h (a tool's click that passes the press rules does what the tool does outside route mode, at once; the release-time tests of claim 16d are not made for it)
            Press::Ordinary if self.play.tool != Tool::Select => false,
            Press::Ordinary => {
                let (cell, object) = self.under(p);
                let on = self.play.target(object, cell);
                let w = &self.play.game.world;
                let sel = self.play.selection.clone();
                match glasnost_sim::routes::click(w, local, &sel, on, mods) {
                    RouteClick::Orders(o) => {
                        self.play.issue_route_click(o, on, mods);
                        self.show_markers();
                        true
                    }
                    RouteClick::Refused(m) => {
                        self.route_refused(m);
                        true
                    }
                    RouteClick::Ignored => true,
                    // spec: S-0213 claim 16h (clicks that select act as outside route mode)
                    RouteClick::Ordinary => false,
                }
            }
        }
    }

    /// The options menu key opens the menu (S-0213 claim 23; S-0201
    /// claim 8: in a single-player game no cycle runs while it is open;
    /// claim 15: in a network game ticks go on). Returns whether the menu
    /// is open.
    pub fn open_options(&mut self) -> bool {
        if self.paused {
            return true;
        }
        if !self.menu_allowed() {
            return false;
        }
        // spec: S-0213 claim 16h, End of route mode (in our engine route mode ends, with the commit, when the options menu is opened while it is on)
        self.route_key(false);
        self.paused = true;
        // spec: S-0111 claim 20 (Options menu: opening it hides a tip that shows and stops the wait)
        self.name_tip.off();
        self.menu_speed = self.speed_in_force();
        true
    }

    /// The index the open options menu's speed control shows (S-0201
    /// claims 8 and 17).
    pub fn menu_speed(&self) -> u8 {
        self.menu_speed
    }

    /// Closes the options menu; a speed control set to an index other than
    /// the one in force sends the speed change (S-0201 claims 8 and 17).
    pub(super) fn close_options(&mut self) {
        self.paused = false;
        // spec: S-0111 claim 20 (Options menu: when it closes nothing shows until the pointer moves and rests again; a tip hidden by the menu does not come back)
        self.name_tip.off();
        if self.network {
            // spec: S-0201 claim 15 (closing the menu brings back drawing: the battlefield as the ticks left it, nothing to catch up)
            self.with_live(|live, inputs, scene| live.redraw(inputs, scene));
        }
        let want = self.menu_speed;
        // spec: S-0201 claim 17 (setting the index already in force sends nothing; no control once the house is defeated)
        if !self.speed_offered() || want == self.speed_in_force() {
            return;
        }
        // spec: S-0201 claim 17 (the change is sent when the menu closes, as an order of the player with the delay of the player's other orders, and carried out in one tick on every machine) and claim 8 (in a skirmish, through our order path: the index changes when the order is carried out, at the end of the next tick)
        self.play.change_speed(want);
        if !self.network {
            // spec: S-0201 claim 8 (in a single-player game the new paced period holds from the next cycle)
            self.speed = want;
        }
    }

    /// Whether the options menu may open now (S-0201 claim 15).
    pub fn menu_allowed(&self) -> bool {
        let w = &self.play.game.world;
        // spec: S-0201 claim 15 (in a skirmish and a network game alike the menu does not open from the end of the game on, S-0414 claim 6, and stays refused through the end delay, S-0414 claim 8)
        if victory::ended(w) {
            return false;
        }
        // spec: S-0201 claim 15 (in a network game, after resigning through the menu it does not open again until the player's house is defeated, S-0414 claims 5 and 11; a defeated player watching a network game can open it)
        // design: in a skirmish the menu opens again after our surrender (S-0201 claim 15 states the refusal for a network game only, the skirmish menu's abort being no resignation, S-0703 claim 13), so that the person watching on can still leave or change the speed
        !self.network || !self.resigned || self.play.local.is_some_and(|h| victory::defeated(w, h))
    }

    /// Whether the options menu offers the speed control (S-0201 claim 17):
    /// in a network game as long as the local house is not defeated; in a
    /// skirmish always, by our engine's rule also while the person watches
    /// on after the defeat.
    pub fn speed_offered(&self) -> bool {
        // spec: S-0201 claim 17 (in a network game the control is offered as long as the player's own house has not been defeated; in a skirmish watched on after the human player's defeat our engine shows it, and a change takes effect as in claim 8, by the project lead's ruling of 2026-10-24)
        // design: watch mode (lead request 2026-11-02, an extra not in the original): a watch game is a skirmish, so the control is offered
        !self.network
            || self
                .play
                .local
                .is_some_and(|h| !victory::defeated(&self.play.game.world, h))
    }

    /// The options menu's leave choices, in order (S-0201 claims 1 and 15).
    pub fn leave_choices(&self) -> Vec<PauseHit> {
        let w = &self.play.game.world;
        let Some(local) = self.play.local else {
            // design: watch mode (lead request 2026-11-02, an extra not in the original): no house to surrender, so the menu offers the leave only, which ends the game at once as a skirmish's does (S-0703 claim 13)
            return vec![PauseHit::Leave];
        };
        let undefeated = !victory::defeated(w, local);
        if self.network {
            // spec: S-0201 claim 15 (while at least two houses of human participants are undefeated, the player's own among them, and the player is not an observer: two choices that send the surrender order, one then leaving the game, one keeping the player watching; otherwise a leave only, sending no surrender order; our engine has no observers, S-0009 claim 28)
            let person = |h: usize| {
                w.houses
                    .get(h)
                    .is_some_and(|x| x.econ.setup.person_played && !x.econ.setup.observer)
            };
            let standing = (0..w.houses.len())
                .filter(|&h| person(h) && !victory::defeated(w, HouseId(h as u8)))
                .count();
            if person(usize::from(local.0)) && undefeated && standing >= 2 {
                return vec![PauseHit::Surrender, PauseHit::SurrenderLeave];
            }
            return vec![PauseHit::Leave];
        }
        // spec: S-0201 claim 1 and S-0703 claim 13 (a skirmish's options menu offers one way to leave, which ends the game at once and sends no order)
        // design: our engine's skirmish menu also offers S-0414 claim 11's surrender (the order is legal at any time in our engine, and a person who surrendered keeps watching; S-0703 claim 13 allows it "where our engine's interface offers one"), while the local house is undefeated and has not surrendered through the menu
        if undefeated && !self.resigned {
            vec![PauseHit::Surrender, PauseHit::Leave]
        } else {
            vec![PauseHit::Leave]
        }
    }

    /// The index the menu's control starts from and is compared with: the
    /// game's in a network game (the changer's own index changes only when
    /// the speed order is carried out, S-0201 claim 17), the paced one in a
    /// single-player game (the last one set, claim 8).
    fn speed_in_force(&self) -> u8 {
        if self.network {
            self.play.game.world.game_speed
        } else {
            self.speed
        }
    }

    /// Whether ticks may run now.
    pub fn ticking(&self) -> bool {
        !self.paused && self.play.running()
    }

    /// What the pointer offers at `p` for table C, and whether the target
    /// is shrouded; `None` keeps the system pointer.
    pub(crate) fn offer(&mut self, p: Point) -> Option<(Offer, bool)> {
        let (cell, object) = self.under(p);
        let on = self.play.target(object, cell);
        let w = &self.play.game.world;
        let Some(local) = self.play.local else {
            // design: watch mode (lead request 2026-11-02, an extra not in the original): no order is offered; over an object a click would select, the cursor offers select, elsewhere the ordinary arrow; nothing is shrouded
            let action = match on {
                ClickOn::Object(_) => targeting::Action::Select,
                ClickOn::Cell(_) => targeting::Action::Nothing,
            };
            let offer = Offer::Action {
                action,
                in_range: false,
                shroud_moves: false,
            };
            return Some((offer, false));
        };
        let tc = match on {
            ClickOn::Object(id) => w.obj(id).map_or(cell, |o| o.pos.cell()),
            ClickOn::Cell(c) => c,
        };
        // spec: S-0214 claim 21 (table C's shrouded column, for a shrouded target) and S-0409 Terms (shrouded: not explored by the house)
        let shrouded = !glasnost_sim::econ::shroud::explored(w, local, tc);
        let tool = match self.play.tool {
            // spec: S-0513 claim 1 (while units are selected, the cursor offers their actions, not the superweapon's)
            Tool::Superweapon { .. } if !self.play.selection.is_empty() => Tool::Select,
            t => t,
        };
        let offer = match tool {
            // Placement hover images belong to S-0406's placement display
            // (S-0214 Open questions): the system pointer stays.
            Tool::Place(_) => return None,
            Tool::Superweapon { sw, .. } => {
                if !self.play.sw_offered(sw, on) {
                    // spec: S-0513 claim 1 (over anything else the cursor shows the no-force-shield action) and S-0214 table CS (no-force-shield)
                    Offer::Aim(cursor::Aim::NoForceShield)
                } else {
                    // spec: S-0214 claim 31 (table CS keys the battlefield image by the type's aim action, S-0512 claim 23, not by its effect kind)
                    let word = w
                        .rules()
                        .superweapons
                        .types
                        .get(sw as usize)
                        .and_then(|t| t.action.as_deref());
                    // An aim action outside table CS has no image there: the
                    // system pointer stays.
                    Offer::Aim(word.and_then(cursor::Aim::of_word)?)
                }
            }
            Tool::Mode(m) => {
                // spec: S-0214 claims 13 and 21 (the sidebar modes' offers and their images in table C)
                let ok = modes::order(w, local, m, on).is_some();
                match (m, ok) {
                    (Mode::Sell, true) => Offer::Sell,
                    (Mode::Sell, false) => Offer::NoSell,
                    (Mode::Repair, true) => Offer::RepairSwitch,
                    (Mode::Repair, false) => Offer::NoRepair,
                    (Mode::Power, _) => Offer::Power,
                }
            }
            Tool::Select => {
                let sel = &self.play.selection;
                let mods = self.mods;
                if targeting::cursor_shows_arrow(w, sel, on) {
                    // spec: S-0214 claim 21 and Edge cases (quirks.berserk_action_cursor off: a berserk infantry or vehicle representative shows the arrow; the action and the click are unchanged)
                    Offer::BerserkArrow
                } else if targeting::shows_attack_move(w, local, sel, on, mods) {
                    Offer::AttackMove
                } else {
                    let action = targeting::cursor(w, local, sel, on, mods);
                    // spec: S-0214 table VD row VD1, table I rows v2 and I22 (route mode held: no-deploy); S-0213 claim 16b (revision 87: not a building's deploy)
                    let action = if self.route.held {
                        targeting::in_route_mode(w, sel, on, action)
                    } else {
                        action
                    };
                    Offer::Action {
                        action,
                        in_range: targeting::attack_cursor_in_range(w, sel, on),
                        shroud_moves: targeting::first_may_move_into_shroud(w, sel),
                    }
                }
            }
        };
        Some((offer, shrouded))
    }

    /// Registers the game-font glyphs the status lines and the name tip
    /// draw with the painter ([`status_font::register`],
    /// [`name_tip::register`]).
    pub fn update_status_glyphs(&mut self, painter: &mut Painter) {
        if let Some(font) = self.play.status_font.as_ref() {
            status_font::register(painter, font, &self.play.messages, &mut self.status_row);
            if let Some(s) = self.name_tip.shown.as_ref() {
                name_tip::register(painter, font, s, &mut self.tip_rows);
            }
        }
    }

    /// Chooses the battlefield cursor for this frame (S-0214 claims 21 and
    /// 22) and registers its frames with the painter. Without the cursor
    /// image, outside the battlefield, with the options menu open or while
    /// placing a building, the system pointer shows.
    pub fn update_cursor(&mut self, painter: &mut Painter) {
        self.cursor_shown = None;
        if self.cursor_art.is_none() || self.paused {
            return;
        }
        let Some(art) = self.cursor_art.as_ref() else {
            return;
        };
        let size = art.size;
        // spec: S-0101 claim 17 (each permitted active drag update shows the static drag-scroll image, not an edge-scroll image)
        let drag = self
            .drag_refused
            .filter(|_| self.right.is_some_and(|r| r.scrolling))
            .map(|r| cursor::drag(r, size.0, size.1));
        // spec: S-0213 claim 24 (during edge scrolling the cursor shows the requested screen direction; the whole outer edge, sidebar included, selects it)
        let edge = drag.or_else(|| {
            self.edge
                .shown_direction(
                    self.pointer.map(|p| (p.x, p.y)),
                    self.screen,
                    self.scroll_index,
                )
                .filter(|_| self.edge_live)
                .and_then(|d| cursor::edge(d, self.scroll_blocked()(d), size.0, size.1))
        });
        let (frames, frame, hot) = if let Some((frames, hot)) = edge {
            (frames, frames.first, hot)
        } else {
            let Some(p) = self.pointer.filter(|&p| self.map_rect().contains(p)) else {
                return;
            };
            let Some((offer, shrouded)) = self.offer(p) else {
                return;
            };
            let retain = self
                .play
                .game
                .world
                .rules()
                .quirks
                .damaging_entry_cursor_retained;
            let choice = cursor::table_c(offer, shrouded, retain);
            let (frames, frame) = self.animator.frame(choice, self.clock_ms);
            (frames, frame, cursor::hot_spot(frames, size.0, size.1))
        };
        let Some(art) = self.cursor_art.as_ref() else {
            return;
        };
        let row = *self
            .cursor_row
            .get_or_insert_with(|| painter.add_palette(art.palette));
        for f in frames.first..=frames.last {
            if !painter.has_sprite(CURSOR_SPRITE, u32::from(f))
                && let Some(img) = art.frame(f)
            {
                painter.add_sprite(CURSOR_SPRITE, u32::from(f), &img, row);
            }
        }
        if painter.has_sprite(CURSOR_SPRITE, u32::from(frame)) {
            self.cursor_shown = Some((frame, hot));
        }
    }

    /// Whether this frame draws its own battlefield cursor, so the system
    /// pointer is hidden.
    pub fn draws_cursor(&self) -> bool {
        self.cursor_shown.is_some()
    }

    /// The selection and hover marks of the live drawing (S-0111 claim 3),
    /// updated before a frame is drawn.
    pub fn update_marks(&mut self) {
        let hovered = self
            .pointer
            .filter(|&p| self.map_rect().contains(p))
            .and_then(|p| self.under(p).1);
        self.update_name_tip();
        let selection = self.play.selection.clone();
        let digits = self.play.group_digits();
        self.with_live(|live, inputs, scene| {
            let marks = live.set_marks(&selection, hovered);
            // Making or changing a group changes no mark, but its digits.
            let groups = live.set_group_digits(digits);
            if marks || groups {
                live.redraw_status(inputs, scene);
            }
        });
    }

    /// One frame of the name tip (S-0111 claims 20 and 26): the option, the
    /// view's movement, and the rest's one choice when the wait ends.
    fn update_name_tip(&mut self) {
        let moving = self.centre != self.tip_centre;
        self.tip_centre = self.centre;
        // spec: S-0111 claim 20 (Options menu: opening the in-game options menu hides a tip and stops the wait; while it is open no wait runs and no tip shows; when it closes nothing shows until the pointer moves and rests again) and claim 20 (the tooltip option off)
        if !self.tool_tips || self.paused {
            self.name_tip.off();
            return;
        }
        // spec: S-0111 claim 20 (Blocked: while the battlefield view is moving, as during a scroll): our test is a view centre changed since the last frame
        // spec: S-0111 claim 20 (Blocked: a left or right button pressed over the view and held)
        // spec: S-0111 claim 20 (Blocked: a hidden cursor or locked input; an order block locks the input and hides the cursor, S-0201 claims 11 and 14): the ticks up to the world's last blocked tick
        let w = &self.play.game.world;
        let order_block = w.tick <= w.orders_blocked_through;
        let blocked = moving || self.view_button_held || order_block;
        let now = self.clock_ms;
        let mut tip = std::mem::take(&mut self.name_tip);
        tip.update(now, blocked, |_| self.choose_name_tip(now));
        self.name_tip = tip;
    }

    /// The tip for the pointer's place at real time `now` (S-0111 claims 21
    /// to 25): `None` when it rests off the battlefield view or the text is
    /// empty. Used for the rest's choice and again in every drawn frame
    /// while the tip shows (claim 26).
    fn choose_name_tip(&mut self, now: i64) -> Option<name_tip::Shown> {
        let p = self.pointer?;
        let m = self.map_rect();
        // spec: S-0111 claim 20 (Where the pointer rests: over the battlefield view; over the sidebar, the radar included, no tip; outside the window nothing; the status message lines drawn on the view are not a place of their own)
        if !m.contains(p) || self.hud_at(p) != Some(HudHit::Map) {
            return None;
        }
        let (cell, id) = self.under(p);
        let text = self.name_tip_text(cell, id);
        // spec: S-0111 claim 25 (the tip colour by the local player's side index, S-0005 claim 13a); design: with no side index, the other colour
        let ink = self
            .local_side(self.play.watching())
            .map_or(name_tip::INK_OTHER, name_tip::ink);
        // design: S-0213 Import mapping (without the game font no text is drawn: no tip shows)
        let font = self.play.status_font.as_ref()?;
        let pitch = i32::try_from(font.line_pitch()).unwrap_or(i32::MAX);
        let width = |c: u16| font.width(c);
        name_tip::tip(&text, (p.x, p.y), m, (&width, pitch), ink, now)
    }

    /// The name tip that shows, if any: its lines and its box, for checks.
    pub fn name_tip_shown(&self) -> Option<&name_tip::Shown> {
        self.name_tip.shown.as_ref()
    }

    /// The name tip's text for pointer cell `cell` and pointer object `id`
    /// (S-0111 claims 21 to 23), in text units; empty for no tip.
    pub fn name_tip_text(&self, cell: CellXY, id: Option<ObjectId>) -> Vec<u16> {
        let w = &self.play.game.world;
        // design: watch mode (lead request 2026-11-02): the watcher has no house, so no cell is unexplored to it and no object is its own
        let local = self.play.local;
        if let Some(h) = local
            && !glasnost_sim::econ::shroud::explored(w, h, cell)
        {
            // spec: S-0111 claim 21 (the shroud label when the pointer cell is not explored by the local player's house, whatever the cell holds)
            return self.fixed_label(SHROUD_LABEL);
        }
        // spec: S-0111 claim 21 (nothing when there is no pointer object)
        let Some(o) = id.and_then(|id| w.obj(id)) else {
            return Vec::new();
        };
        let Some(kind) = live::import_kind(o.kind).filter(|_| o.kind != SimKind::Terrain) else {
            // spec: S-0111 claim 21 (any other pointer object, such as a terrain object: the display name of the resource kind of the pointer cell's resource overlay, S-0401; nothing without one)
            return glasnost_sim::econ::resource::resource_at(w, cell)
                .and_then(|(t, _)| w.rules().econ.resource.tiberiums.get(t))
                .map(|t| self.display_name(Kind::Tiberiums, &t.name))
                .unwrap_or_default();
        };
        let own = local.is_some() && o.owner == local;
        if !own {
            // spec: S-0111 claim 21 (another house's object: nothing when fully cloaked and not sensed by the local player's house, S-0516 Terms, also when its owner is allied)
            if glasnost_sim::cloak::fully_cloaked(w, o.id)
                && !glasnost_sim::cloak::senses(w, local, o.id)
            {
                return Vec::new();
            }
            let name = live::type_name(w.rules(), o.kind, o.type_index);
            let idx = build::find_type(&self.loaded.import, kind, &live::name_bytes(&name));
            let flag = |k: &[u8]| {
                idx.is_some_and(|i| build::rule_bool(&self.loaded.import, kind, i, k, false))
            };
            // spec: S-0111 claim 21 and Import mapping (nothing for a type with the invisible flag: Invisible, absent no; a building type whose InvisibleInGame is yes also gets it)
            if flag(b"Invisible") || (o.kind == SimKind::Building && flag(b"InvisibleInGame")) {
                return Vec::new();
            }
        }
        self.name_by_kind(o, own)
    }

    /// The name of claim 22 of S-0111 for object `o`, `own` when the local
    /// player owns it.
    fn name_by_kind(&self, o: &glasnost_sim::world::Object, own: bool) -> Vec<u16> {
        let w = &self.play.game.world;
        let local = self.play.local;
        let own_name = || {
            let kind = live::import_kind(o.kind).expect("every unit kind has an import kind");
            self.display_name(kind, &live::type_name(w.rules(), o.kind, o.type_index))
        };
        match o.kind {
            // spec: S-0111 claim 22 (a building or an aircraft: its type's display name; a disguise does not change it)
            SimKind::Building | SimKind::Aircraft | SimKind::Terrain => own_name(),
            SimKind::Infantry => {
                // spec: S-0111 claim 22 (the technician label only for a unit with the technician mark): our engine sets no mark yet, which releases set it being open (S-0111 Open questions)
                // spec: S-0111 claim 22 (the disguise's display name when the unit is disguised and its owner does not ally the local player's house; not S-0516 claim 35's test)
                let allies = own || w.allied(o.owner, local);
                match o.cloak.disguise {
                    Some(d) if !allies => self.disguise_name(d.what),
                    _ => own_name(),
                }
            }
            SimKind::Vehicle => {
                // spec: S-0111 claim 22 (a vehicle that appears disguised to the local player's house, S-0516 claim 35, with a disguise: nothing for a disguise without a house, else the disguise's display name)
                if let Some(d) = o.cloak.disguise
                    && glasnost_sim::cloak::disguise::appears_disguised(w, o.id, local)
                {
                    return if d.house.is_none() {
                        Vec::new()
                    } else {
                        self.disguise_name(d.what)
                    };
                }
                let name = live::type_name(w.rules(), o.kind, o.type_index);
                let import = &self.loaded.import;
                let idx = build::find_type(import, Kind::Vehicles, &live::name_bytes(&name));
                // spec: S-0111 Import mapping (HasTurretTooltips: the turret-tip flag, absent no) and claim 22 (any other turret-tip vehicle: claim 23)
                match idx.filter(|&i| {
                    build::rule_bool(import, Kind::Vehicles, i, b"HasTurretTooltips", false)
                }) {
                    Some(i) => self.turret_tip_text(o, i, own_name()),
                    // spec: S-0111 claim 22 (any other vehicle: its type's display name)
                    None => own_name(),
                }
            }
        }
    }

    /// The text of a turret-tip vehicle `o` of import type `idx` whose own
    /// type's display name is `name` (S-0111 claim 23).
    fn turret_tip_text(
        &self,
        o: &glasnost_sim::world::Object,
        idx: usize,
        name: Vec<u16>,
    ) -> Vec<u16> {
        let w = &self.play.game.world;
        let import = &self.loaded.import;
        // spec: S-0111 claim 23 (P is the passenger at position 1 of the passenger order, S-0507 claim 3)
        let p = o.transport.passengers.first().and_then(|&p| w.obj(p));
        let modifier = match p.filter(|p| p.kind == SimKind::Infantry) {
            // spec: S-0111 claim 23 (with no passenger, or a P that is not infantry, label A)
            None => self.fixed_label(TIP_LABEL_A),
            Some(p) => {
                let pname = live::type_name(w.rules(), p.kind, p.type_index);
                let pidx = build::find_type(import, Kind::Infantry, &live::name_bytes(&pname));
                // spec: S-0111 Import mapping (UseOwnName: the own-name flag, absent no)
                let u = pidx.is_some_and(|i| {
                    build::rule_bool(import, Kind::Infantry, i, b"UseOwnName", false)
                });
                let p_name = || self.display_name(Kind::Infantry, &pname);
                // spec: S-0111 claim 23 (the selected turret index t, S-0105 claim 43, and for a rotating-barrel type by the gunner rules)
                let shown = (w.tick - 1).max(0);
                let t = build::tip_turret_index(w, o.id, shown, import.turret_tables.get(&idx));
                // spec: S-0111 claim 23 (the modifier table: 0 label A; 1 P's name if u, else label B; 2 label C; 3 P's name; any other value, none included, P's name if u, else label A)
                match t {
                    Some(0) => self.fixed_label(TIP_LABEL_A),
                    Some(1) if u => p_name(),
                    Some(1) => self.fixed_label(TIP_LABEL_B),
                    Some(2) => self.fixed_label(TIP_LABEL_C),
                    Some(3) => p_name(),
                    _ if u => p_name(),
                    _ => self.fixed_label(TIP_LABEL_A),
                }
            }
        };
        // spec: S-0111 claim 23 (the modifier, one space, and the vehicle's own type's display name, never its disguise's; an empty modifier or name still leaves the space)
        let mut text = modifier;
        text.push(u16::from(b' '));
        text.extend(name);
        text
    }

    /// The display name of a disguise's type (S-0111 claim 22: that of the
    /// type it copies, which can be a terrain object's or an overlay's).
    fn disguise_name(&self, what: Option<glasnost_sim::cloak::disguise::DisguiseAs>) -> Vec<u16> {
        use glasnost_sim::cloak::disguise::DisguiseAs;
        let r = self.play.game.world.rules();
        let (kind, name) = match what {
            Some(DisguiseAs::Infantry(i)) => {
                (Kind::Infantry, live::type_name(r, SimKind::Infantry, i))
            }
            Some(DisguiseAs::Vehicle(i)) => {
                (Kind::Vehicles, live::type_name(r, SimKind::Vehicle, i))
            }
            Some(DisguiseAs::Terrain(i)) => (
                Kind::TerrainObjects,
                live::type_name(r, SimKind::Terrain, i),
            ),
            Some(DisguiseAs::Overlay(i)) => match r.overlays.get(usize::from(i)) {
                Some(ov) => (Kind::Overlays, ov.name.clone()),
                None => return Vec::new(),
            },
            // spec: S-0004 claim 17 (no type, so no label: empty text)
            None => return Vec::new(),
        };
        self.display_name(kind, &name)
    }

    /// The display name of the type of `kind` named `name` (S-0004 claim
    /// 17): the text its label in force resolves to; without a string
    /// table read, the type's name stands in.
    fn display_name(&self, kind: Kind, name: &str) -> Vec<u16> {
        let import = &self.loaded.import;
        let Some(i) = build::find_type(import, kind, &live::name_bytes(name)) else {
            // spec: S-0004 claim 17 (a type that never had a label has an empty display name)
            return Vec::new();
        };
        // spec: S-0004 claim 17 and S-0111 Import mapping (UIName: the display-name label, as layered)
        let label = strings::label(build::rule(import, kind, i, b"UIName"));
        strings::resolve(self.strings.as_ref(), label, name)
    }

    /// The text of a fixed string-table entry `label` (S-0004 claim 17);
    /// without a string table read, the label stands in.
    fn fixed_label(&self, label: &str) -> Vec<u16> {
        strings::resolve(self.strings.as_ref(), Some(label.as_bytes()), label)
    }

    /// The interface over the battlefield image: the image, the
    /// placement preview, our selection markers, the box, messages, the
    /// sidebar, the options menu and the client's `overlay` (notices in the
    /// interface's logical pixels, drawn with the sidebar at its scale).
    pub fn draw(&mut self, overlay: DrawList) -> DrawList {
        let size = self.screen;
        let m = self.map_rect();
        let mut d = DrawList::new();
        d.image(MAP_IMAGE, m);
        let view = self.view();
        let f = self.loaded.map.footprint;
        // spec: S-0101 claim 8 (black bands over the battlefield image, taking precedence over S-0110's cover and light; overlays, the cursor, messages and the sidebar drawn after may appear over them)
        for r in outside_local_area(m, view.origin, local_area_rect(f.width, f.local)) {
            d.fill(r, [0, 0, 0, 255]);
        }
        let world_to_screen = |x: i32, y: i32| (x - view.origin.0 + m.x, y - view.origin.1 + m.y);
        // Placement preview (S-0411 claim 10, S-0406): green where the
        // building would be placed, red elsewhere; at the keyboard cell
        // while the keyboard aims.
        let aim = self.aim_cell();
        let at = match aim {
            Some(c) => Some(c),
            None => self
                .pointer
                .filter(|&p| m.contains(p))
                .map(|p| self.under(p).0),
        };
        if let Some(c) = aim {
            // Our own mark of the keyboard cell (M8 step C3).
            let level = self
                .play
                .game
                .world
                .cells
                .cell(c)
                .map_or(0, |x| i32::from(x.level));
            let (tx, ty) = cell_top(c.x, c.y);
            let (sx, sy) = world_to_screen(tx, ty - TERRAIN_LIFT * level);
            diamond(&mut d, sx, sy, AIM_INK, m);
        }
        if self.route.held
            && let Some(local) = self.play.local
        {
            // spec: S-0213 claim 16g (route points and the routes through them are drawn only while the viewer holds the route-mode key, for the viewer's own house and allied houses; each point with a marker, each holder's route as lines from the holder through its points); design: their look is ours (not studied)
            let hover = self.route_hover();
            let w = &self.play.game.world;
            let at_cell = |c: CellXY| {
                let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
                let (qx, qy) = project_lifted(256 * c.x + 128, 256 * c.y + 128, 0);
                world_to_screen(qx, qy - TERRAIN_LIFT * level)
            };
            for (id, cells) in glasnost_sim::routes::drawn(w, local) {
                let Some(o) = w.obj(id) else { continue };
                let (qx, qy) = project_lifted(o.pos.x, o.pos.y, o.pos.z);
                let mut prev = world_to_screen(qx, qy);
                for c in cells {
                    let next = at_cell(c);
                    if m.contains(Point::new(prev.0, prev.1))
                        && m.contains(Point::new(next.0, next.1))
                    {
                        d.line(prev, next, ROUTE_INK);
                    }
                    prev = next;
                }
            }
            for (&n, p) in &w.routes.points {
                if p.house != local && !w.allied(Some(local), Some(p.house)) {
                    continue;
                }
                let c = glasnost_sim::routes::point_cell(w, p);
                let (tx, ty) = cell_top(c.x, c.y);
                let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
                let (sx, sy) = world_to_screen(tx, ty - TERRAIN_LIFT * level);
                let ink = if self.route.point == Some(n) {
                    ROUTE_SELECTED_INK
                } else if hover == Some(n) {
                    // spec: S-0213 claim 16b, Click areas (the candidate is highlighted only when a press would take it)
                    ROUTE_HOVER_INK
                } else {
                    ROUTE_POINT_INK
                };
                diamond(&mut d, sx, sy, ink, m);
            }
        }
        if let Some(cell) = at
            && let Some((cells, ok)) = self.play.preview(cell)
        {
            let fill = if ok {
                [60, 220, 80, 110]
            } else {
                [230, 50, 40, 110]
            };
            for &c in &cells {
                let level = self
                    .play
                    .game
                    .world
                    .cells
                    .cell(c)
                    .map_or(0, |x| i32::from(x.level));
                let (tx, ty) = cell_top(c.x, c.y);
                let (sx, sy) = world_to_screen(tx, ty - TERRAIN_LIFT * level);
                diamond(&mut d, sx, sy, fill, m);
            }
            // design: our placement range indicator (M8 A4 round 5 note
            // 34): the ground circle of the type's weapon range around the
            // foundation's centre, projected by S-0101 F1 to an ellipse, at
            // the height of the foundation's first cell.
            if let (Some(range), Some(&first)) = (self.play.placement_range(), cells.first()) {
                let (x0, x1) = cells
                    .iter()
                    .fold((i32::MAX, i32::MIN), |(a, b), c| (a.min(c.x), b.max(c.x)));
                let (y0, y1) = cells
                    .iter()
                    .fold((i32::MAX, i32::MIN), |(a, b), c| (a.min(c.y), b.max(c.y)));
                // spec: S-0307 claim 1 (256 leptons per cell)
                let (lx, ly) = ((x0 + x1 + 1) * 128, (y0 + y1 + 1) * 128);
                let level = self
                    .play
                    .game
                    .world
                    .cells
                    .cell(first)
                    .map_or(0, |x| i32::from(x.level));
                let (qx, qy) = project_lifted(lx, ly, 0);
                let (cx, cy) = world_to_screen(qx, qy - TERRAIN_LIFT * level);
                let (a, b) = range_axes(range);
                ellipse(&mut d, (cx, cy), (a, b), RANGE_INK, m);
            }
        }
        // Our markers for selected objects the S-0111 bracket does not mark.
        let w = &self.play.game.world;
        let plain: Vec<ObjectId> = {
            let ids = self.play.selection.clone();
            let assets = &self.loaded.assets;
            match self.loaded.live.as_ref() {
                Some(l) => ids
                    .into_iter()
                    .filter(|&id| !l.bracket_drawn(assets, id))
                    .collect(),
                None => ids,
            }
        };
        for id in plain {
            let Some(o) = w.obj(id) else { continue };
            let p = w.reference_point(id);
            let (qx, qy) = project_lifted(p.x, p.y, p.z);
            let (cx, cy) = world_to_screen(qx, qy - 8);
            let (hw, hh) = match o.kind {
                SimKind::Infantry => (8, 14),
                SimKind::Building => {
                    let (fw, fh) = o.foundation;
                    (15 * (fw + fh), 8 * (fw + fh) + 20)
                }
                _ => (16, 14),
            };
            brackets(&mut d, cx, cy, hw, hh);
        }
        // spec: S-0213 claim 20 (a factory's guard spot is its rally point); design: M8 A4 round 7 note 42 (no spec says what shows it: while one of our factories, cloning buildings or repair depots is selected, a dashed line from it to its rally point and a diamond on the point's cell)
        let rally_phase = (self.clock_ms / DASH_MS).rem_euclid(2 * DASH) as i32;
        for &id in &self.play.selection {
            let Some(o) = w.obj(id) else { continue };
            // design: watch mode (lead request 2026-11-02, an extra not in the original): a selected object's rally point shows as its own house would see it
            if o.owner != self.play.eye(id) || !targeting::holds_rally_point(w, id) {
                continue;
            }
            let (cell, to) = match o.guard_spot {
                Some(glasnost_sim::world::Target::Cell(c)) => (c, None),
                Some(glasnost_sim::world::Target::Object(t)) => match w.obj(t) {
                    Some(t) => (t.pos.cell(), Some(t.pos)),
                    None => continue,
                },
                None => continue,
            };
            let level = w.cells.cell(cell).map_or(0, |x| i32::from(x.level));
            let (tx, ty) = cell_top(cell.x, cell.y);
            let (sx, sy) = world_to_screen(tx, ty - TERRAIN_LIFT * level);
            diamond(&mut d, sx, sy, RALLY_POINT_INK, m);
            let end = match to {
                Some(p) => {
                    let (qx, qy) = project_lifted(p.x, p.y, p.z);
                    world_to_screen(qx, qy)
                }
                // The diamond's centre: 15 pixels below its top corner (S-0101 F1).
                None => (sx, sy + 15),
            };
            let p = w.reference_point(id);
            let (qx, qy) = project_lifted(p.x, p.y, p.z);
            dashed(
                &mut d,
                world_to_screen(qx, qy),
                end,
                rally_phase,
                RALLY_INK,
                m,
            );
        }
        // spec: S-0610 claims 10 and 11 (the hostile-intent lines the local player's psychic sensors show: from the unit's firing point to its target's aim point or to its destination, a 3 × 3 square in the unit owner's house colour at both ends, the dashes moving with real time)
        let phase = (self.clock_ms / DASH_MS).rem_euclid(2 * DASH) as i32;
        // design: watch mode (lead request 2026-11-02, an extra not in the original): no house's psychic sensors are the viewer's, so no intent line is shown
        let intents = self
            .play
            .local
            .map(|h| glasnost_sim::psychic::intent_lines(w, h))
            .unwrap_or_default();
        for l in intents {
            let screen = |p: glasnost_sim::world::Pos| {
                let (qx, qy) = project_lifted(p.x, p.y, p.z);
                world_to_screen(qx, qy)
            };
            let (a, b) = (screen(l.from), screen(l.to));
            let [r, g, bl] = l
                .owner
                .and_then(|h| self.house_colours.get(usize::from(h.0)))
                .copied()
                .unwrap_or([255, 255, 255]);
            dashed(&mut d, a, b, phase, INTENT_INK, m);
            for (x, y) in [a, b] {
                let sq = Rect::new(x - 1, y - 1, 3, 3).intersect(&m);
                if !sq.is_empty() {
                    d.fill(sq, [r, g, bl, 255]);
                }
            }
        }
        // spec: S-0601 claim 17 (a link line from each controller to each victim in the controller house's colour, from the alternate anchor k mod 5 to the victim raised by its link height; drawn when the controller, a transport carrying it or the victim is selected, or the victim's timer runs; selecting only the carrying transport shows it too, the drawing bug fixed)
        let selected = |id: ObjectId| self.play.selection.contains(&id);
        for l in glasnost_sim::mind::link_lines(w) {
            let carrier = glasnost_sim::transport::riding(w, l.controller);
            if !(l.timer
                || selected(l.controller)
                || selected(l.victim)
                || carrier.is_some_and(selected))
            {
                continue;
            }
            // Our own: no line into or out of cells the local player has not
            // explored (in a watch game, where nothing is shrouded, every line).
            let seen = |p: glasnost_sim::world::Pos| {
                self.play
                    .local
                    .is_none_or(|h| glasnost_sim::econ::shroud::explored(w, h, p.cell()))
            };
            if !(seen(l.from) && seen(l.to)) {
                continue;
            }
            let screen = |p: glasnost_sim::world::Pos| {
                let (qx, qy) = project_lifted(p.x, p.y, p.z);
                world_to_screen(qx, qy)
            };
            let [r, g, bl] = l
                .house
                .and_then(|h| self.house_colours.get(usize::from(h.0)))
                .copied()
                .unwrap_or([255, 255, 255]);
            let (a, b) = (screen(l.from), screen(l.to));
            if m.contains(Point::new(a.0, a.1)) && m.contains(Point::new(b.0, b.1)) {
                d.line(a, b, [r, g, bl, 255]);
            }
        }
        // spec: S-0601 claim 18 (the control ring on a controlled object, the permanent ring on a permanently controlled one, raised by the ring height; on a building in front of it). Seam: the ring animations' art is not drawn yet; a marker stands in at the ring's place.
        for o in w.objects.values() {
            // Our own: no ring on objects in cells the local player has not
            // explored (in a watch game, where nothing is shrouded, on every one).
            let seen = self
                .play
                .local
                .is_none_or(|h| glasnost_sim::econ::shroud::explored(w, h, o.pos.cell()));
            if !o.on_map || !seen {
                continue;
            }
            let Some(ring) = glasnost_sim::mind::ring(w, o.id) else {
                continue;
            };
            let (qx, qy) = project_lifted(ring.at.x, ring.at.y, ring.at.z);
            let (x, y) = world_to_screen(qx, qy);
            let ink = if ring.permanent {
                PERMA_RING_INK
            } else {
                RING_INK
            };
            ring_marker(&mut d, x, y, ink, m);
        }
        if let (Some(drag), Some(p)) = (self.drag, self.pointer)
            && drag.active
        {
            d.frame(
                Rect::spanning(drag.press, band::limit(p, m)),
                colour::ACCENT,
            );
        }
        if let Some(font) = &self.play.status_font {
            // The status lines keep the battlefield's pixels: S-0213 claim
            // 25 fixes their width budget and places in screen pixels.
            status_font::draw(&mut d, font, &self.play.messages, m);
        }
        // The interface over the battlefield, in logical pixels at the
        // interface scale.
        let mut ui = DrawList::new();
        let lm = self.layout().map;
        if let Some(f) = &self.play.failed {
            ui.fill(Rect::new(lm.x, lm.bottom() - 16, lm.w, 16), [0, 0, 0, 200]);
            ui.text(lm.x + 4, lm.bottom() - 13, f.clone(), colour::BAD, lm);
        }
        ui.extend(hud::countdown_view(
            &self.countdown_rows(),
            lm,
            self.clock_ms,
        ));
        // A shared handle: a copy of the rules each frame cost more than the
        // rest of the draw list (M8 step C4).
        let rules = self.play.game.world.rules_arc();
        let name = |t: TypeRef| live::type_name(&rules, t.kind, t.index);
        let ids = &self.icon_ids;
        let icons = &self.icons;
        let local = self.play.local;
        let world = &self.play.game.world;
        let (alt_ids, alt_icons) = (&self.alt_icon_ids, &self.alt_icons);
        let art = |t: hud::Entry| {
            // spec: S-0007 claim 25 (the alternate icon exactly when the type has one, is trainable and the local house has infiltrated the matching factory; chosen each time the sidebar shows the icon)
            if let hud::Entry::Type(ty) = t
                && local
                    .is_some_and(|h| glasnost_sim::structures::spy::alternate_icon(world, h, ty))
                && let (Some(&id), Some(img)) = (alt_ids.get(&ty), alt_icons.get(&ty))
            {
                return Some((id, 0, img.width() as i32, img.height() as i32));
            }
            let id = *ids.get(&t)?;
            let img = icons.get(&t)?;
            Some((id, 0, img.width() as i32, img.height() as i32))
        };
        let pointer = self.pointer;
        let hover = pointer.map(|p| self.ui.to_ui(p));
        let (hud_list, targets) = if self.play.watching() {
            // design: watch mode (lead request 2026-11-02, an extra not in the original): the watch panel in the sidebar's place
            let radar = self.radar.is_some().then_some(RADAR_IMAGE);
            watch::view(&self.watch_model(), self.ui.ui, radar, hover)
        } else {
            let mut model = self.play.hud_model(&name, &art);
            model.buttons = self.buttons;
            model.scroll_pressed = self.scroll_pressed();
            model.view_box = self.radar_view_box();
            model.focus_cameo = match self.hud_focus.at {
                Some(HudHit::Cameo(n)) if self.hud_focus.shown => Some(n),
                _ => None,
            };
            hud::view(&model, self.ui.ui, Some(RADAR_IMAGE), hover)
        };
        ui.extend(hud_list);
        self.targets = targets;
        if self.paused {
            let speed = self.speed_offered().then_some(self.menu_speed);
            let leaves = self.leave_choices();
            let (p, t) = hud::pause_view(self.ui.ui, speed, self.network, &leaves, hover);
            ui.extend(p);
            self.pause_targets = t;
            self.pause_focus.mark(&mut ui, &self.pause_targets);
        } else {
            self.hud_focus.mark(&mut ui, &self.targets);
        }
        ui.extend(overlay);
        d.scaled(self.ui, ui);
        // spec: S-0111 claim 25 (the name tip on top of everything else on the battlefield view, never over the sidebar: kept inside the view) and claim 20 (Options menu: no tip shows while the menu is open)
        if let (Some(s), Some(font), false) = (
            self.name_tip.shown.as_ref(),
            self.play.status_font.as_ref(),
            self.paused,
        ) {
            name_tip::draw(&mut d, font, s, m);
        }
        if let (Some((frame, (hx, hy))), Some(p)) = (self.cursor_shown, pointer) {
            // spec: S-0214 claim 22 (the hot spot coincides with the pointer's input pixel)
            let all = Rect::new(0, 0, size.0, size.1);
            d.sprite(CURSOR_SPRITE, u32::from(frame), p.x - hx, p.y - hy, all);
        }
        d
    }
}

/// The keyboard cell's step in a screen direction (`dx`, `dy`: one of
/// them −1 or +1): to the neighbouring cell half a diamond that way. S-0101
/// F1 puts cell (x, y)'s top corner at (30(x − y), 15(x + y)), so a step in
/// x moves (+30, +15) and a step in y (−30, +15); a screen direction
/// alternates between the two diagonal neighbours on its side, by the
/// parity of x + y, so that the steps reach every cell and each undoes the
/// opposite one.
fn aim_step(c: CellXY, dx: i32, dy: i32) -> CellXY {
    // spec: S-0101 F1 (a cell's top corner projects to (30(x − y), 15(x + y)))
    let even = (c.x + c.y).rem_euclid(2) == 0;
    let (x, y) = match (dx.signum(), dy.signum(), even) {
        // Right: down-right from an even cell, up-right from an odd one.
        (1, _, true) => (1, 0),
        (1, _, false) => (0, -1),
        // Left: down-left from an even cell, up-left from an odd one.
        (-1, _, true) => (0, 1),
        (-1, _, false) => (-1, 0),
        // Down: down-left from an even cell, down-right from an odd one.
        (0, 1, true) => (0, 1),
        (0, 1, false) => (1, 0),
        // Up: up-left from an even cell, up-right from an odd one.
        (0, -1, true) => (-1, 0),
        (0, -1, false) => (0, -1),
        _ => (0, 0),
    };
    CellXY::new(c.x + x, c.y + y)
}

/// A filled cell diamond with its top corner at (x, y): 60 × 30 pixels
/// (S-0101 F1), drawn as one-pixel rows, clipped to `clip`.
fn diamond(d: &mut DrawList, x: i32, y: i32, rgba: [u8; 4], clip: Rect) {
    // spec: S-0101 F1 (a cell's diamond: 60 × 30 pixels below its top corner)
    for row in 0..30 {
        let half = if row < 15 {
            2 * row + 2
        } else {
            2 * (29 - row) + 2
        };
        let r = Rect::new(x - half, y + row, 2 * half, 1).intersect(&clip);
        if !r.is_empty() {
            d.fill(r, rgba);
        }
    }
}

/// The intent line's dash and gap length, pixels (our own look; S-0610
/// claim 11 leaves the pattern open).
// nocite: our own presentation of S-0610 claim 11's dash pattern
const DASH: i64 = 4;
/// Real milliseconds per pixel of dash movement (our own look).
// nocite: our own presentation of S-0610 claim 11's real-time dash animation
const DASH_MS: i64 = 60;
/// The intent line's ink (our own; S-0610 open question on the line colour).
// nocite: our own line colour, S-0610 leaves its animation unspecified
const INTENT_INK: [u8; 4] = [230, 60, 60, 255];

/// The stand-in control ring's ink (our own; S-0601 claim 18's animation
/// art is not drawn yet).
// nocite: our own stand-in colour for the control ring
const RING_INK: [u8; 4] = [200, 80, 230, 255];
/// The stand-in permanent control ring's ink (our own).
// nocite: our own stand-in colour for the permanent control ring
const PERMA_RING_INK: [u8; 4] = [255, 120, 255, 255];
/// The keyboard cell's mark (our own; the interface's focus colour, veiled).
// nocite: our own client setting (keyboard aiming)
const AIM_INK: [u8; 4] = [96, 208, 255, 90];
/// The type-select tap limit: the longest hold that is a tap, whole
/// milliseconds.
pub const TYPE_SELECT_TAP_MS: i64 = 500; // spec: S-0214 Constants (type-select tap limit: 500 milliseconds; a release at most 500 after the press is a tap)
/// The name tip's shroud label: a fixed string-table entry identifier.
const SHROUD_LABEL: &str = "TXT_SHADOW"; // spec: S-0111 Import mapping (string table TXT_SHADOW: the shroud label of claim 21, a fixed entry identifier)
/// Label A of a turret-tip vehicle's modifier.
const TIP_LABEL_A: &str = "Tip:Rocket"; // spec: S-0111 Import mapping (string table Tip:Rocket: label A of claim 23)
/// Label B of a turret-tip vehicle's modifier.
const TIP_LABEL_B: &str = "Tip:MachineGun"; // spec: S-0111 Import mapping (string table Tip:MachineGun: label B of claim 23)
/// Label C of a turret-tip vehicle's modifier.
const TIP_LABEL_C: &str = "Tip:Repair"; // spec: S-0111 Import mapping (string table Tip:Repair: label C of claim 23)

/// The group repeat interval: a *select group* press less than this after
/// the last one that selected the group can be a repeat, milliseconds.
pub const GROUP_REPEAT_MS: i64 = 800; // spec: S-0214 Constants (group repeat interval: 800 milliseconds of real time, less than counts)

/// The type-select role's key (S-0214 claim 32): whether it is held, the
/// time of the press that holds it, and the reach the last tap left.
#[derive(Clone, Copy, Debug, Default)]
struct TypeSelect {
    held: bool,
    pressed: i64,
    reach: Reach,
}

/// Route mode's interface state (S-0213 claims 16, 16e and 16g): whether
/// its key is held, the selected route point, the tick of the last
/// refusal cue, what the left press of the click under way does with the
/// points under it, and whether the local house's defeat has ended route
/// mode for good.
#[derive(Clone, Copy, Debug, Default)]
struct RouteMode {
    held: bool,
    point: Option<u32>,
    cue_tick: Option<i32>,
    pressed: Option<glasnost_sim::routes::Press>,
    ended: bool,
}

/// What a click with the type-select role held does (S-0214 claim 32).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HeldClick {
    /// *Select*: the type replaces the selection.
    Select,
    /// *Toggle select* on an unselected object: the type joins.
    Add,
    /// *Toggle select* on a selected object: the type leaves.
    Remove,
}

/// A control-group command (S-0214 claim 33; its default keys, S-0213
/// claim 22).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupCommand {
    /// The digit alone: select the group.
    Select,
    /// Shift and the digit: add the group to the selection.
    Add,
    /// Ctrl and the digit: make the group from the selection.
    Make,
    /// Alt and the digit: select the group and centre the view on it.
    Centre,
}

/// Route lines' ink (our own look; S-0213 claim 16g leaves it open).
// nocite: our own look of route lines, not studied (S-0213 claim 16g)
const ROUTE_INK: [u8; 4] = [120, 230, 120, 230];
/// A route point marker's ink (our own look).
// nocite: our own look of route point markers, not studied (S-0213 claim 16g)
const ROUTE_POINT_INK: [u8; 4] = [120, 230, 120, 110];
/// The line from a selected factory to its rally point (our own look).
// nocite: our own presentation of a rally point (M8 A4 round 7 note 42; no spec draws it)
const RALLY_INK: [u8; 4] = [255, 230, 80, 255];
/// The rally point's cell (our own look).
// nocite: our own presentation of a rally point (M8 A4 round 7 note 42; no spec draws it)
pub(crate) const RALLY_POINT_INK: [u8; 4] = [255, 230, 80, 90];
/// The selected route point's ink (our own look).
// nocite: our own look of the selected route point, not studied (S-0213 claim 16g)
const ROUTE_SELECTED_INK: [u8; 4] = [255, 255, 140, 160];
/// The highlighted route point's ink: the one a press would take (our own
/// look).
// nocite: our own look of the highlighted route point, not studied (S-0213 claim 16g)
const ROUTE_HOVER_INK: [u8; 4] = [255, 255, 255, 200];

/// A right click while the route-mode key is held and nothing is selected
/// clears the point selection only when its release is less than this many
/// screen pixels from its press on each axis (S-0213 claim 16h).
const ROUTE_RIGHT_CLICK_BOUND: i32 = 4; // spec: S-0213 claim 16h (the point selection is cleared only by a right click whose release is less than 4 screen pixels from its press on each axis; a bound of its own, apart from the drag bound)

/// The placement range indicator's ink (our own).
// nocite: our own colour for the placement range indicator
const RANGE_INK: [u8; 4] = [255, 230, 80, 200];

/// The half-axes in pixels of the screen ellipse of a ground circle of
/// `range` leptons. S-0101 F1 puts a ground point (X, Y) at
/// (30(X − Y) / 256, 15(X + Y) / 256), so a circle of radius R becomes an
/// ellipse with half-axes 30·√2·R / 256 and 15·√2·R / 256.
fn range_axes(range: i32) -> (i32, i32) {
    let r = u128::from(range.max(0).unsigned_abs());
    // spec: S-0101 F1 (30 and 15 pixels per 256 leptons along the projected axes)
    let half = |k: u128| (glasnost_num::geom::isqrt(2 * (k * r) * (k * r)) / 256) as i32;
    (half(30), half(15))
}

/// An ellipse outline centred at `c` with half-axes `ab`, as short lines;
/// lines outside `clip` are left out.
fn ellipse(d: &mut DrawList, c: (i32, i32), ab: (i32, i32), rgba: [u8; 4], clip: Rect) {
    let (a, b) = ab;
    if a <= 0 || b <= 0 {
        return;
    }
    let y_at = |x: i32| {
        let (a2, x2) = (i128::from(a) * i128::from(a), i128::from(x) * i128::from(x));
        let s = glasnost_num::geom::isqrt((a2 - x2).max(0) as u128) as i64;
        (i64::from(b) * s / i64::from(a)) as i32
    };
    let step = (a / 24).max(1);
    let mut xs: Vec<i32> = (-a..a).step_by(step as usize).collect();
    xs.push(a);
    for sign in [-1, 1] {
        for pair in xs.windows(2) {
            let (p, q) = (
                (c.0 + pair[0], c.1 + sign * y_at(pair[0])),
                (c.0 + pair[1], c.1 + sign * y_at(pair[1])),
            );
            if clip.contains(Point::new(p.0, p.1)) && clip.contains(Point::new(q.0, q.1)) {
                d.line(p, q, rgba);
            }
        }
    }
}

/// A small flat ring (an ellipse outline, 12 × 6 pixels) centred at
/// `(x, y)`: the stand-in for the control ring animations (S-0601 claim
/// 18).
fn ring_marker(d: &mut DrawList, x: i32, y: i32, rgba: [u8; 4], clip: Rect) {
    // nocite: our own stand-in shape
    const PTS: [(i32, i32); 8] = [
        (6, 0),
        (4, 2),
        (0, 3),
        (-4, 2),
        (-6, 0),
        (-4, -2),
        (0, -3),
        (4, -2),
    ];
    for i in 0..PTS.len() {
        let (a, b) = (PTS[i], PTS[(i + 1) % PTS.len()]);
        let (p, q) = ((x + a.0, y + a.1), (x + b.0, y + b.1));
        if clip.contains(Point::new(p.0, p.1)) && clip.contains(Point::new(q.0, q.1)) {
            d.line(p, q, rgba);
        }
    }
}

/// A dashed line from `a` to `b`: dashes of [`DASH`] pixels with gaps of the
/// same length, shifted along the line by `phase` pixels; dashes outside
/// `clip` are left out.
fn dashed(d: &mut DrawList, a: (i32, i32), b: (i32, i32), phase: i32, rgba: [u8; 4], clip: Rect) {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let len = dx.abs().max(dy.abs());
    if len == 0 {
        return;
    }
    let at = |t: i32| (a.0 + dx * t / len, a.1 + dy * t / len);
    let step = 2 * DASH as i32;
    let mut t = phase - step;
    while t < len {
        let (s, e) = (t.max(0), (t + DASH as i32).min(len));
        if s < e {
            let (p, q) = (at(s), at(e));
            let inside = |(x, y): (i32, i32)| clip.contains(Point::new(x, y));
            if inside(p) && inside(q) {
                d.line(p, q, rgba);
            }
        }
        t += step;
    }
}

/// Our corner brackets around a point (presentation only, no spec).
fn brackets(d: &mut DrawList, cx: i32, cy: i32, hw: i32, hh: i32) {
    let ink = [255, 230, 80, 255];
    let arm = hw.min(hh) / 2;
    for (sx, sy) in [(-1, -1), (1, -1), (-1, 1), (1, 1)] {
        let (x, y) = (cx + sx * hw, cy + sy * hh);
        d.line((x, y), (x - sx * arm, y), ink);
        d.line((x, y), (x, y - sy * arm), ink);
    }
}

/// The sidebar icons, the alternate icons and the cursor art of a game.
type Art = (
    BTreeMap<hud::Entry, IndexedImage>,
    BTreeMap<TypeRef, IndexedImage>,
    Option<CursorArt>,
);

/// The imported cursor image and palette (S-0214 claim 22, Import
/// mapping).
struct CursorArt {
    bytes: Vec<u8>,
    /// The full canvas, pixels.
    size: (i32, i32),
    palette: [u8; 1024],
}

impl CursorArt {
    /// Frame `f` decoded on the full canvas.
    fn frame(&self, f: u16) -> Option<IndexedImage> {
        Shp::parse(&self.bytes)
            .ok()?
            .decode_on_canvas(usize::from(f))
            .ok()
    }
}

/// The cursor palette's file as the session keeps it (S-0111 claim 19a,
/// S-0005 claim 22): one copy, shared by the cursor images (S-0214), the
/// wrench and the bomb icon.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CursorPaletteKeep {
    /// `quirks.art_cache_never_invalidated` on: the copy read once when the
    /// install was mounted, before any game (`None`: not found then), used
    /// by every game of the session.
    Session(Option<Vec<u8>>),
    /// The switch off: resolved and read afresh at the start of each game.
    #[default]
    EachGame,
}

impl CursorPaletteKeep {
    /// The keep at program start, under the mounted install's switch.
    pub async fn at_program_start(vfs: &Vfs) -> Self {
        if vfs.game_options().quirks.art_cache_never_invalidated {
            // spec: S-0111 claim 19a, S-0005 claim 22 (switch on: read once when the program starts, before any game)
            Self::Session(read_file(vfs, CURSOR_PALETTE).await)
        } else {
            // spec: S-0111 claim 19a (switch off: nothing kept; each game reads it afresh)
            Self::EachGame
        }
    }

    /// The cursor palette file a game starting on `vfs` uses.
    pub async fn for_game(&self, vfs: &Vfs) -> Option<Vec<u8>> {
        match self {
            // spec: S-0111 claim 19a (switch on: the program-start copy, across games, a replaced file not followed)
            Self::Session(kept) => kept.clone(),
            // spec: S-0111 claim 19a, S-0005 claim 22 (switch off: resolved and read afresh at the start of each game)
            Self::EachGame => read_file(vfs, CURSOR_PALETTE).await,
        }
    }
}

/// A file resolved by S-0005 and read; `None` when it is not found or
/// cannot be read.
async fn read_file(vfs: &Vfs, name: &[u8]) -> Option<Vec<u8>> {
    let r = vfs.resolve_bytes(name)?;
    vfs.read_resolved(&r).await.ok()
}

/// Reads the cursor image; with the game's cursor palette file `palette`,
/// the cursor art. `None` when either is missing (no replacement is
/// specified, so the system pointer stays).
async fn read_cursor(vfs: &Vfs, palette: Option<&[u8]>) -> Option<CursorArt> {
    // spec: S-0214 Import mapping (cursor image MOUSE.SHA, resolved by S-0005, a sprite image; cursor palette MOUSEPAL.PAL, S-0103)
    let bytes = read_file(vfs, b"MOUSE.SHA").await?;
    let shp = Shp::parse(&bytes).ok()?;
    let size = (i32::from(shp.width), i32::from(shp.height));
    // spec: S-0111 claim 19a (the same cursor palette, through S-0103 claim 1's load rule)
    let pal = crate::mapview::combat::cursor_palette(
        palette?,
        &glasnost_render::palette::PaletteQuirks::default(),
    )?;
    Some(CursorArt {
        bytes,
        size,
        palette: pal.to_rgba8(),
    })
}

/// Reads and decodes frame 0 of every buildable type's sidebar icon
/// (S-0007 claims 22 and 25: the main icon; the alternate one needs
/// infiltration, which no skirmish of M5b has).
/// The icon palette's 5-6-5 pixels as the painter's RGBA (S-0103 claim 13:
/// what a packed pixel shows; entry 0 is transparent through the sprite).
fn icon_rgba(p: &[u16; 256]) -> [u8; 1024] {
    let mut out = [0u8; 1024];
    for (dst, &c) in out.as_chunks_mut::<4>().0.iter_mut().zip(p.iter()) {
        let (r, g, b) = glasnost_render::compose::expand565(c);
        *dst = [r, g, b, 255];
    }
    out
}

/// The first frame of an icon file on its canvas, when it decodes to a
/// non-empty image.
async fn icon_frame(vfs: &Vfs, file: &str) -> Option<IndexedImage> {
    let r = vfs.resolve(file)?;
    let bytes = vfs.read_resolved(&r).await.ok()?;
    // frame 0 on its canvas (interim, Q-0275)
    Shp::parse(&bytes)
        .ok()
        .and_then(|s| s.decode_on_canvas(0).ok())
        .filter(|i| i.width() > 0 && i.height() > 0)
}

async fn read_icons(
    vfs: &Vfs,
    loaded: &Loaded,
) -> (
    BTreeMap<hud::Entry, IndexedImage>,
    BTreeMap<TypeRef, IndexedImage>,
) {
    let mut out = BTreeMap::new();
    let mut alt = BTreeMap::new();
    let Some(rules) = loaded.rules.as_deref() else {
        return (out, alt);
    };
    for kind in [
        SimKind::Building,
        SimKind::Vehicle,
        SimKind::Infantry,
        SimKind::Aircraft,
    ] {
        let count = match kind {
            SimKind::Building => rules.buildings.len(),
            SimKind::Vehicle => rules.vehicles.len(),
            SimKind::Infantry => rules.infantry.len(),
            _ => rules.aircraft.len(),
        } as u32;
        for index in 0..count {
            let name = live::type_name(rules, kind, index);
            let Some(ik) = live::import_kind(kind) else {
                continue;
            };
            let Some(i) = build::find_type(&loaded.import, ik, &live::name_bytes(&name)) else {
                continue;
            };
            // spec: S-0007 claim 25 (the alternate icon: the file the art lookup resolved, frame 0 on its canvas, no placeholder)
            if let Some(file) = loaded
                .import
                .arts
                .get(&(ik, i))
                .and_then(|a| a.alt_icon.as_ref())
                && let Ok(text) = std::str::from_utf8(file)
                && let Some(img) = icon_frame(vfs, text).await
            {
                alt.insert(TypeRef { kind, index }, img);
            }
            let Some(file) = loaded
                .import
                .arts
                .get(&(ik, i))
                .and_then(|a| a.icon.as_ref())
            else {
                continue;
            };
            let Ok(text) = std::str::from_utf8(file) else {
                continue;
            };
            let Some(r) = vfs.resolve(text) else { continue };
            let Ok(bytes) = vfs.read_resolved(&r).await else {
                continue;
            };
            // spec: S-0007 claim 22 (the main icon file); frame 0 on its canvas (interim, Q-0275)
            if let Some(img) = Shp::parse(&bytes)
                .ok()
                .and_then(|s| s.decode_on_canvas(0).ok())
                .filter(|i| i.width() > 0 && i.height() > 0)
            {
                out.insert(hud::Entry::Type(TypeRef { kind, index }), img);
            }
        }
    }
    for (i, t) in rules.superweapons.types.iter().enumerate() {
        // spec: S-0007 claim 31 (the sidebar icon name, the type name by default, ‖ .shp; no theater handling or generic fallback)
        let name = t.sidebar_image.as_deref().unwrap_or(&t.name);
        // spec: S-0007 claim 31 (cut to 24 bytes)
        let cut = &name.as_bytes()[..name.len().min(24)];
        let file = format!("{}.shp", String::from_utf8_lossy(cut));
        // spec: S-0007 claim 31 (an unresolved file uses the placeholder XXICON.SHP, as for the main icon)
        let img = match icon_frame(vfs, &file).await {
            Some(img) => Some(img),
            None => icon_frame(vfs, "XXICON.SHP").await,
        };
        if let Some(img) = img {
            out.insert(hud::Entry::Superweapon(i as u32), img);
        }
    }
    (out, alt)
}

/// The parts of the battlefield rectangle `m`, showing world pixels from
/// `origin`, that S-0101 claim 8 makes black: on each oversized axis, the
/// pixels beyond that axis's interval of the local area's world-pixel
/// rectangle `local` (x, y, width, height). The bands of both axes form a
/// union, so the corners are covered when both are oversized.
fn outside_local_area(m: Rect, origin: (i32, i32), local: (i32, i32, i32, i32)) -> Vec<Rect> {
    let (lx, ly, lw, lh) = local;
    let mut out = Vec::new();
    // spec: S-0101 F5 (an axis is oversized when M < m: 60·V_w < W horizontally, 30·V_h + 135 < H vertically) and claim 8 (non-oversized axes gain no clipping)
    if lw < m.w {
        // spec: S-0101 claim 8, F5 (left inclusive, right exclusive)
        let x0 = (lx - origin.0 + m.x).clamp(m.x, m.right());
        let x1 = (lx + lw - origin.0 + m.x).clamp(m.x, m.right());
        out.push(Rect::new(m.x, m.y, x0 - m.x, m.h));
        out.push(Rect::new(x1, m.y, m.right() - x1, m.h));
    }
    if lh < m.h {
        // spec: S-0101 claim 8, F5 (top inclusive, bottom exclusive; the rectangle is not shifted to the oversized centre)
        let y0 = (ly - origin.1 + m.y).clamp(m.y, m.bottom());
        let y1 = (ly + lh - origin.1 + m.y).clamp(m.y, m.bottom());
        out.push(Rect::new(m.x, m.y, m.w, y0 - m.y));
        out.push(Rect::new(m.x, y1, m.w, m.bottom() - y1));
    }
    out.retain(|r| r.w > 0 && r.h > 0);
    out
}

#[cfg(test)]
mod local_area_tests {
    use super::*;

    // spec: S-0101 F5 (a limited view that is not oversized lies inside the local area: nothing is masked) and claim 8 (an oversized view centred on it: the margins on both sides are black)
    #[test]
    fn only_an_oversized_view_has_margins() {
        let m = Rect::new(0, 0, 640, 400);
        let local = (-780, 495, 1560, 795);
        assert!(outside_local_area(m, (-780, 495), local).is_empty());
        assert!(outside_local_area(m, (140, 890), local).is_empty());
        let wide = Rect::new(0, 0, 2000, 400);
        let margins = outside_local_area(wide, (-1000, 600), local);
        assert_eq!(
            margins,
            vec![Rect::new(0, 0, 220, 400), Rect::new(1780, 0, 220, 400)]
        );
    }

    // spec: S-0101 claim 8 (left and top inclusive, right and bottom exclusive: the first pixel outside each interval is black, the last inside is not)
    #[test]
    fn the_band_boundaries_follow_the_rectangle() {
        let local = (-780, 495, 1560, 795);
        let wide = Rect::new(10, 20, 2000, 400);
        let bands = outside_local_area(wide, (-1000, 600), local);
        let black = |x: i32, y: i32| bands.iter().any(|r| r.contains(Point { x, y }));
        // world column c is at screen x = c + 1000 + 10
        let screen = |c: i32| c + 1010;
        assert!(black(screen(-781), 100));
        assert!(!black(screen(-780), 100));
        assert!(!black(screen(-780 + 1560 - 1), 100));
        assert!(black(screen(-780 + 1560), 100));
    }

    // spec: S-0101 claim 8 (with both axes oversized the black region is the union of the horizontal and vertical bands, corners included; non-oversized axes gain no clipping)
    #[test]
    fn both_axes_give_the_union_and_one_axis_clips_only_itself() {
        let local = (-780, 495, 1560, 795);
        let big = Rect::new(0, 0, 2000, 1000);
        let bands = outside_local_area(big, (-1000, 400), local);
        let black = |x: i32, y: i32| bands.iter().any(|r| r.contains(Point { x, y }));
        // world (x, y) is at screen (x + 1000, y − 400)
        assert!(black(0, 0), "top-left corner");
        assert!(black(1999, 999), "bottom-right corner");
        assert!(black(1000 - 780, 495 - 400 - 1), "just above the rectangle");
        assert!(
            !black(1000 - 780, 495 - 400),
            "the rectangle's top-left pixel"
        );
        assert!(black(1000, 495 + 795 - 400), "just below the rectangle");
        // Wide but not tall: a view hanging past the rectangle's top is not clipped vertically.
        let wide = Rect::new(0, 0, 2000, 400);
        let bands = outside_local_area(wide, (-1000, 400), local);
        assert!(bands.iter().all(|r| r.h == 400), "{bands:?}");
    }
}

#[cfg(test)]
mod cursor_palette_tests {
    use std::rc::Rc;

    use glasnost_fmt::mix::{MixWriter, WriteLayout, id_hash};
    use glasnost_vfs::{GameOptions, GameQuirks, MemSource, PickedFile};

    use super::*;

    /// A mounted install whose main archive holds `palette` as the cursor
    /// palette (or none), searched under `quirks`. The bytes are invented.
    fn install(palette: Option<&[u8]>, quirks: GameQuirks) -> Vfs {
        let mut files = vec![(id_hash("filler.ini"), b"[Filler]\n".to_vec())];
        if let Some(p) = palette {
            files.push((id_hash("mousepal.pal"), p.to_vec()));
        }
        let main = MixWriter {
            layout: WriteLayout::Plain,
            files,
        }
        .write();
        let mut vfs = Vfs::default();
        pollster::block_on(vfs.mount_install(vec![PickedFile {
            path: "ra2md.mix".into(),
            source: Rc::new(MemSource(main)),
        }]));
        pollster::block_on(vfs.mount_startup(GameOptions {
            quirks,
            ..GameOptions::default()
        }));
        vfs
    }

    fn off() -> GameQuirks {
        GameQuirks {
            art_cache_never_invalidated: false,
            ..GameQuirks::default()
        }
    }

    /// spec: S-0111 claim 19a and scenario OV14 palette part c with its engine fixture (switch on:
    /// read once when the program starts, before any game, so a palette replaced between games is
    /// not followed; switch off: resolved and read afresh at the start of each game, so the second
    /// game takes the replacement), S-0005 claim 22
    #[test]
    fn cursor_palette_lifetime_follows_the_switch() {
        let first = vec![1u8; 768];
        let second = vec![2u8; 768];
        // Switch on (the default): both games take the program-start copy.
        let start = install(Some(&first), GameQuirks::default());
        let keep = pollster::block_on(CursorPaletteKeep::at_program_start(&start));
        assert_eq!(keep, CursorPaletteKeep::Session(Some(first.clone())));
        assert_eq!(
            pollster::block_on(keep.for_game(&start)),
            Some(first.clone())
        );
        let replaced = install(Some(&second), GameQuirks::default());
        assert_eq!(
            pollster::block_on(keep.for_game(&replaced)),
            Some(first.clone())
        );
        // Switch off: nothing kept; each game reads the file present at its start.
        let start = install(Some(&first), off());
        let keep = pollster::block_on(CursorPaletteKeep::at_program_start(&start));
        assert_eq!(keep, CursorPaletteKeep::EachGame);
        assert_eq!(pollster::block_on(keep.for_game(&start)), Some(first));
        let replaced = install(Some(&second), off());
        assert_eq!(pollster::block_on(keep.for_game(&replaced)), Some(second));
    }

    /// spec: S-0111 claim 19a and scenario OV14 palette part d (switch on: the read happens at
    /// start-up, so a palette that appears after it does not reach the games; with none found
    /// then, no game has one: the missing-file case), S-0111 Import mapping (MOUSEPAL.PAL,
    /// resolved by S-0005)
    #[test]
    fn cursor_palette_is_read_at_program_start() {
        let start = install(None, GameQuirks::default());
        let keep = pollster::block_on(CursorPaletteKeep::at_program_start(&start));
        assert_eq!(keep, CursorPaletteKeep::Session(None));
        let later = install(Some(&[7u8; 768]), GameQuirks::default());
        assert_eq!(pollster::block_on(keep.for_game(&later)), None);
        // With the switch off the later game finds it.
        let later = install(Some(&[7u8; 768]), off());
        assert_eq!(
            pollster::block_on(CursorPaletteKeep::EachGame.for_game(&later)),
            Some(vec![7u8; 768])
        );
    }
}

#[cfg(test)]
mod aim_tests {
    use super::*;

    /// A ground circle of one cell's radius projects to an ellipse of
    /// 30·√2 by 15·√2 pixels, and the drawn outline passes through its
    /// four extreme points. spec: S-0101 F1 (30 and 15 pixels per 256 leptons)
    #[test]
    fn the_range_ellipse_follows_the_projection() {
        assert_eq!(range_axes(256), (42, 21));
        assert_eq!(range_axes(2560), (424, 212));
        assert_eq!(range_axes(0), (0, 0));
        let mut d = DrawList::new();
        let clip = Rect::new(0, 0, 1000, 1000);
        ellipse(&mut d, (500, 500), (42, 21), RANGE_INK, clip);
        let ends: Vec<(i32, i32)> = d
            .prims
            .iter()
            .filter_map(|p| match p {
                glasnost_ui::draw::Prim::Line { from, to, .. } => Some([*from, *to]),
                _ => None,
            })
            .flatten()
            .collect();
        for p in [(458, 500), (542, 500), (500, 479), (500, 521)] {
            assert!(ends.contains(&p), "{p:?}");
        }
        assert!(
            ends.iter()
                .all(|&(x, y)| (458..=542).contains(&x) && (479..=521).contains(&y))
        );
    }

    /// The keyboard cell's steps go the screen's way, reach every cell and
    /// undo each other. spec: S-0101 F1 (a cell's top corner projects to (30(x − y), 15(x + y)))
    #[test]
    fn keyboard_cell_steps_follow_the_screen_and_reach_every_cell() {
        let top = |c: CellXY| cell_top(c.x, c.y);
        let dirs = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        for x in 0..4 {
            for y in 0..4 {
                let c = CellXY::new(x, y);
                let (sx, sy) = top(c);
                for (dx, dy) in dirs {
                    let to = aim_step(c, dx, dy);
                    let (tx, ty) = top(to);
                    // Half a diamond the arrow's way.
                    if dx != 0 {
                        assert_eq!(tx - sx, 30 * dx, "{c:?} {dx}");
                        assert_eq!((ty - sy).abs(), 15);
                    } else {
                        assert_eq!(ty - sy, 15 * dy, "{c:?} {dy}");
                        assert_eq!((tx - sx).abs(), 30);
                    }
                    assert_eq!(aim_step(to, -dx, -dy), c, "the opposite step undoes it");
                }
            }
        }
        // Two steps right or down move a whole diamond straight that way.
        let c = CellXY::new(5, 2);
        let r = aim_step(aim_step(c, 1, 0), 1, 0);
        assert_eq!(top(r), (top(c).0 + 60, top(c).1));
        let d = aim_step(aim_step(c, 0, 1), 0, 1);
        assert_eq!(top(d), (top(c).0, top(c).1 + 30));
        // Every neighbouring cell is at most three steps away, from a cell
        // of either parity.
        for c in [CellXY::new(5, 2), CellXY::new(5, 3)] {
            let mut reached = std::collections::BTreeSet::from([(c.x, c.y)]);
            for _ in 0..3 {
                let now: Vec<_> = reached.iter().copied().collect();
                for (x, y) in now {
                    for (dx, dy) in dirs {
                        let n = aim_step(CellXY::new(x, y), dx, dy);
                        reached.insert((n.x, n.y));
                    }
                }
            }
            for (dx, dy) in [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
            ] {
                assert!(reached.contains(&(c.x + dx, c.y + dy)), "{c:?} {dx} {dy}");
            }
        }
    }
}

/// The screen position of a route point's place, in battlefield pixels
/// (S-0213 claim 16b, *Click areas*): an object's projected position, or
/// a cell's centre lifted by its level.
fn place_screen(
    w: &glasnost_sim::world::World,
    p: &glasnost_sim::routes::RoutePoint,
) -> (i32, i32) {
    if let glasnost_sim::world::Target::Object(id) = p.place
        && let Some(o) = w.obj(id)
    {
        return project_lifted(o.pos.x, o.pos.y, o.pos.z);
    }
    let c = glasnost_sim::routes::point_cell(w, p);
    let level = w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    let (x, y) = glasnost_render::projection::cell_centre(c.x, c.y);
    let (qx, qy) = project_lifted(x, y, 0);
    (qx, qy - TERRAIN_LIFT * level)
}
