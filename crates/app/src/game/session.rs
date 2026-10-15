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

use std::collections::BTreeMap;

use glasnost_ai::game::Game;
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
use glasnost_sim::econ::TypeRef;
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{GameSetup, ObjectId};
use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::hud::{self, HudHit, Layout, PauseHit};
use glasnost_ui::modes::{self, Mode};
use glasnost_ui::widget::Targets;
use glasnost_vfs::{FileRef, Vfs};

use super::band;
use super::cursor::{self, Offer};
use super::paint::Painter;
use super::play::{Button, Play, Tool, local_house};
use super::scroll::{self, EdgeScroll};
use super::status_font;
use crate::mapview::build::{self, GameMode, Inputs, Options};
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
/// The 64-bit FNV-1a hash of a map file: the map's name in replays, so
/// that no map is named by its title (our own identifier).
pub fn content_hash(bytes: &[u8]) -> String {
    // nocite: FNV-1a 64-bit offset basis and prime (a public hash, our own identifier choice)
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("map-{h:016x}")
}

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
    /// The paced game-speed index (S-0201 claims 7 and 8).
    pub speed: u8,
    /// The options menu is open (S-0201 claim 8: no ticks run).
    pub paused: bool,
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
    /// The icon palette's RGBA (S-0103 claim 13).
    icon_palette: Option<[u8; 1024]>,
    pointer: Option<Point>,
    /// Edge momentum and its retained direction (S-0101 F9).
    edge: EdgeScroll,
    /// Whether the last scroll cycle allowed edge scrolling (focus, no
    /// order block: S-0201 claim 14).
    edge_live: bool,
    press: Option<(Point, Button)>,
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
    /// The interface's real-time clock, milliseconds (S-0214 claim 22: not
    /// simulation ticks).
    pub clock_ms: i64,
    /// The last drawn sidebar's targets.
    targets: Targets<HudHit>,
    pause_targets: Targets<PauseHit>,
    /// Problems met while loading (shown in the log).
    pub problems: Vec<String>,
}

/// What the client must do after a gesture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionEvent {
    /// Nothing else.
    None,
    /// Leave the game for the main menu.
    Leave,
}

impl Session {
    /// Loads the map file `file` of the mounted install and starts the game
    /// `make` sets up from the map's rules. `local_slot_names` are not used:
    /// colours come from the resolved setup.
    pub async fn start(
        vfs: &mut Vfs,
        file: &FileRef,
        make: live::MakeSetup<'_>,
    ) -> Result<Self, String> {
        let bytes = vfs
            .read(file)
            .await
            .map_err(|e| format!("the map could not be read: {e}"))?;
        let map_id = content_hash(&bytes);
        let options = Options {
            skirmish: true,
            game: GameMode::Skirmish,
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
        )
        .await;
        let icons = read_icons(vfs, &loaded).await;
        let cursor_art = read_cursor(vfs).await;
        let status_font = status_font::read(vfs).await;
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
        let local = local_house(&setup).ok_or("the setup has no person's house")?;
        if let Some(l) = loaded.live.as_mut() {
            // spec: S-0516 claims 28 and 29 (the battlefield drawn through the local player's eyes)
            l.set_viewer(Some(local));
        }
        let radar = Radar::new(
            &game.world,
            loaded.map.theater,
            &loaded.tile_colours,
            loaded.house_radar_colours(&game.world),
            loaded.neutral_radar_colour(),
        );
        let speed = game.world.game_speed;
        let start = game
            .world
            .objects
            .values()
            .find(|o| o.owner == Some(local) && o.on_map)
            .map(|o| project_lifted(o.pos.x, o.pos.y, o.pos.z));
        let mut s = Self {
            loaded,
            scene,
            play: Play::new(game, local),
            radar,
            map_id,
            speed,
            paused: false,
            screen: (1024, 768),
            sized: false,
            centre: (0, 0),
            filter: Filter {
                groups: Group::ALL.to_vec(),
                provisional: true,
            },
            icons,
            icon_ids: BTreeMap::new(),
            icon_palette,
            pointer: None,
            edge: EdgeScroll::default(),
            edge_live: true,
            press: None,
            drag: None,
            right: None,
            drag_refused: None,
            cursor_art,
            cursor_row: None,
            status_row: None,
            animator: cursor::Animator::default(),
            cursor_shown: None,
            mods: Modifiers::default(),
            clock_ms: 0,
            targets: Targets::new(),
            pause_targets: Targets::new(),
            problems: Vec::new(),
        };
        problems.append(&mut s.loaded.problems);
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
        let (inputs, live) = self.loaded.split();
        let live = live?;
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

    /// Runs one tick and redraws the live objects.
    pub fn step(&mut self) {
        if !self.play.running() {
            return;
        }
        self.play.step();
        if self.play.failed.is_some() {
            return;
        }
        let effects = self.play.effects.clone();
        self.with_live(|live, inputs, scene| {
            live.world.effects = effects;
            live.after_tick();
            live.redraw(inputs, scene);
        });
        let house = self.play.local;
        if let Some(r) = self.radar.as_mut() {
            r.observe(&self.play.game.world, house);
        }
    }

    /// The radar map's top-left pixel on the screen (S-0410 F9's origin
    /// before the image's own offset).
    pub fn radar_origin(&self) -> (i32, i32) {
        let r = Layout::new(self.screen).radar;
        (r.x, r.y)
    }

    /// The battlefield's rectangle on the screen.
    pub fn map_rect(&self) -> Rect {
        Layout::new(self.screen).map
    }

    /// Sets the window size. The client's first call, as the game starts,
    /// sets the starting size; a later change is an in-game change of the
    /// battlefield view's dimensions (S-0213 claim 25).
    pub fn resize(&mut self, size: (i32, i32)) {
        let first = !self.sized;
        self.sized = true;
        if size != self.screen {
            self.screen = size;
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
                scroll::DEFAULT_SCROLL_INDEX,
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
            scroll::DEFAULT_SCROLL_INDEX,
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
    /// covers (S-0110).
    pub fn light_alpha(&self) -> Vec<u8> {
        cover::light_alpha(
            &self.play.game.world,
            self.play.local,
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
        let Some(pal) = self.icon_palette else {
            return;
        };
        let row = painter.add_palette(pal);
        for (next, (t, img)) in (ICON_SPRITES..).zip(&self.icons) {
            if painter.add_sprite(next, 0, img, row) {
                self.icon_ids.insert(*t, next);
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

    /// The pointer moved.
    pub fn pointer_moved(&mut self, p: Point) {
        self.pointer = Some(p);
        let mode = self.play.game.world.rules().mode;
        if let Some(d) = self.drag.as_mut() {
            d.motion(p, mode);
        }
        if let Some(r) = self.right.as_mut() {
            r.motion(p);
        }
    }

    /// The pointer left the window.
    pub fn pointer_left(&mut self) {
        self.pointer = None;
        self.press = None;
        self.drag = None;
        self.right = None;
    }

    /// A mouse-wheel event, anywhere over the game (S-0411 claim 18).
    pub fn wheel(&mut self, delta: i32) {
        if self.paused {
            return;
        }
        let rows = Layout::new(self.screen).rows();
        self.play.wheel_icons(delta, rows);
    }

    /// A button went down.
    pub fn pointer_down(&mut self, b: Button, p: Point) {
        self.pointer = Some(p);
        self.press = Some((p, b));
        // spec: S-0214 claim 18 (a left-button press in the battlefield can start a selection drag when sell, repair, power, building-placement and superweapon targeting modes are off)
        self.drag = (b == Button::Left
            && !self.paused
            && self.play.tool == Tool::Select
            && self.targets.hit(p) == Some(HudHit::Map))
        .then(|| band::Drag::armed(p));
        // spec: S-0214 claim 27 (an accepted right-button press in the battlefield anchors a possible scroll drag at that pixel)
        self.right =
            (b == Button::Right && !self.paused && self.targets.hit(p) == Some(HudHit::Map))
                .then(|| band::RightDrag::armed(p));
    }

    /// A button went up: a click or the end of a drag.
    pub fn pointer_up(&mut self, b: Button, p: Point, mods: Modifiers) -> SessionEvent {
        self.pointer = Some(p);
        let drag = self.drag.take();
        let right = self.right.take();
        let Some((start, pb)) = self.press.take() else {
            return SessionEvent::None;
        };
        if pb != b {
            return SessionEvent::None;
        }
        if self.paused {
            return match self.pause_targets.hit(p) {
                Some(PauseHit::Resume) => {
                    self.paused = false;
                    SessionEvent::None
                }
                Some(PauseHit::Speed(d)) => {
                    // spec: S-0201 claim 8 (the options menu's speed control)
                    self.speed = hud::step_speed(self.speed, d);
                    SessionEvent::None
                }
                Some(PauseHit::Surrender) => {
                    self.paused = false;
                    self.play.surrender();
                    SessionEvent::None
                }
                Some(PauseHit::Leave) => SessionEvent::Leave,
                None => SessionEvent::None,
            };
        }
        let dragged = drag.is_some_and(|d| d.active) && b == Button::Left;
        let scrolled = right.is_some_and(|r| r.scrolling) && b == Button::Right;
        match self.targets.hit(start) {
            // spec: S-0214 claim 27 (a scroll drag's release neither clears the selection, nor cancels a targeting mode, nor gives an order)
            // spec: S-0411 claim 10 (a classified right drag and release keep placement mode and the ready item: no cancellation, no refund)
            Some(HudHit::Map) if scrolled => {}
            Some(HudHit::Map) if dragged && self.play.tool == Tool::Select => {
                let end = band::limit(p, self.map_rect());
                let inside = self.box_candidates(start, end);
                // spec: S-0214 claim 19 (an unmodified box without a live candidate continues as the ordinary release click at the final pixel)
                if self.play.select_box(&inside, mods.force_select) {
                    let (cell, object) = self.under(end);
                    self.play.click_map(b, object, cell, mods);
                    self.show_markers();
                }
            }
            Some(HudHit::Map) if !dragged => {
                let (cell, object) = self.under(p);
                self.play.click_map(b, object, cell, mods);
                self.show_markers();
            }
            Some(HudHit::Radar) if b == Button::Left => {
                let l = Layout::new(self.screen);
                if let Some(cell) = self.radar.as_ref().and_then(|r| {
                    r.cell_at(&self.play.game.world, p.x - l.radar.x, p.y - l.radar.y)
                }) && !self.play.click_radar(cell)
                {
                    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
                    self.set_centre(project_lifted(x, y, 0));
                }
            }
            Some(HudHit::Tab(i)) => self.play.tab_click(i),
            Some(HudHit::Cameo(n)) => self.play.cameo_click(n, b, mods.force_select),
            // spec: S-0411 claim 18 (a page button acts on release over the button after a press there)
            Some(HudHit::Scroll(d)) if self.targets.hit(p) == Some(HudHit::Scroll(d)) => {
                let rows = Layout::new(self.screen).rows();
                self.play.scroll_icons(d, rows);
            }
            Some(HudHit::Mode(m)) => self.play.mode_click(m),
            Some(HudHit::Options) => self.paused = true,
            _ => {}
        }
        SessionEvent::None
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
        let admitted = band::admit(buildings, mobile);
        let (press, end) = (
            Point::new(press.x - m.x, press.y - m.y),
            Point::new(end.x - m.x, end.y - m.y),
        );
        admitted
            .into_iter()
            .filter(|&(_, q)| band::inside(press, end, q))
            .map(|(id, _)| id)
            .collect()
    }

    /// A key command (S-0213 table T2).
    pub fn key(&mut self, cmd: KeyCommand) {
        if !self.paused {
            self.play.key(cmd);
        }
    }

    /// The options menu key opens the menu (S-0213 claim 23; S-0201
    /// claim 8: no ticks run while it is open).
    pub fn open_options(&mut self) {
        self.paused = true;
    }

    /// Whether ticks may run now.
    pub fn ticking(&self) -> bool {
        !self.paused && self.play.running()
    }

    /// What the pointer offers at `p` for table C, and whether the target
    /// is shrouded; `None` keeps the system pointer.
    fn offer(&mut self, p: Point) -> Option<(Offer, bool)> {
        let (cell, object) = self.under(p);
        let on = self.play.target(object, cell);
        let w = &self.play.game.world;
        let local = self.play.local;
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
            // S-0214 claim 31's table CS keys the aim images by aim action,
            // and the action words are still interim (Q-0731): the system
            // pointer stays.
            Tool::Superweapon { .. } => return None,
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
                    Offer::Action {
                        action: targeting::cursor(w, local, sel, on, mods),
                        in_range: targeting::attack_cursor_in_range(w, sel, on),
                        shroud_moves: targeting::first_may_move_into_shroud(w, sel),
                    }
                }
            }
        };
        Some((offer, shrouded))
    }

    /// Registers the game-font glyphs the status lines draw with the
    /// painter ([`status_font::register`]).
    pub fn update_status_glyphs(&mut self, painter: &mut Painter) {
        if let Some(font) = self.play.status_font.as_ref() {
            status_font::register(painter, font, &self.play.messages, &mut self.status_row);
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
                    scroll::DEFAULT_SCROLL_INDEX,
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
        let selection = self.play.selection.clone();
        self.with_live(|live, inputs, scene| {
            if live.set_marks(&selection, hovered) {
                live.redraw_status(inputs, scene);
            }
        });
    }

    /// The interface over the battlefield image: the image, the
    /// placement preview, our selection markers, the box, messages, the
    /// sidebar and the options menu.
    pub fn draw(&mut self) -> DrawList {
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
        // building would be placed, red elsewhere.
        if let Some(p) = self.pointer.filter(|&p| m.contains(p)) {
            let (cell, _) = self.under(p);
            if let Some((cells, ok)) = self.play.preview(cell) {
                let fill = if ok {
                    [60, 220, 80, 110]
                } else {
                    [230, 50, 40, 110]
                };
                for c in cells {
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
        if let (Some(drag), Some(p)) = (self.drag, self.pointer)
            && drag.active
        {
            d.frame(
                Rect::spanning(drag.press, band::limit(p, m)),
                colour::ACCENT,
            );
        }
        if let Some(font) = &self.play.status_font {
            status_font::draw(&mut d, font, &self.play.messages, m);
        }
        if let Some(f) = &self.play.failed {
            d.fill(Rect::new(m.x, m.bottom() - 16, m.w, 16), [0, 0, 0, 200]);
            d.text(m.x + 4, m.bottom() - 13, f.clone(), colour::BAD, m);
        }
        let rules = self.play.game.world.rules().clone();
        let name = |t: TypeRef| live::type_name(&rules, t.kind, t.index);
        let ids = &self.icon_ids;
        let icons = &self.icons;
        let art = |t: hud::Entry| {
            let id = *ids.get(&t)?;
            let img = icons.get(&t)?;
            Some((id, 0, img.width() as i32, img.height() as i32))
        };
        let model = self.play.hud_model(&name, &art);
        let pointer = self.pointer;
        let (hud_list, targets) = hud::view(&model, size, Some(RADAR_IMAGE), pointer);
        d.extend(hud_list);
        self.targets = targets;
        if self.paused {
            let (p, t) = hud::pause_view(size, self.speed, pointer);
            d.extend(p);
            self.pause_targets = t;
        }
        if let (Some((frame, (hx, hy))), Some(p)) = (self.cursor_shown, pointer) {
            // spec: S-0214 claim 22 (the hot spot coincides with the pointer's input pixel)
            let all = Rect::new(0, 0, size.0, size.1);
            d.sprite(CURSOR_SPRITE, u32::from(frame), p.x - hx, p.y - hy, all);
        }
        d
    }
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

/// Reads the cursor image and its palette; `None` when either is missing
/// (no replacement is specified, so the system pointer stays).
async fn read_cursor(vfs: &Vfs) -> Option<CursorArt> {
    // spec: S-0214 Import mapping (cursor image MOUSE.SHA, resolved by S-0005, a sprite image; cursor palette MOUSEPAL.PAL, S-0103)
    let r = vfs.resolve("MOUSE.SHA")?;
    let bytes = vfs.read_resolved(&r).await.ok()?;
    let shp = Shp::parse(&bytes).ok()?;
    let size = (i32::from(shp.width), i32::from(shp.height));
    let pr = vfs.resolve("MOUSEPAL.PAL")?;
    let pb = vfs.read_resolved(&pr).await.ok()?;
    let p = glasnost_fmt::pal::Palette6::parse(&pb).ok()?;
    // spec: S-0103 claim 2 (a palette file's components through the load rule)
    let pal = glasnost_render::palette::palette_from_file(
        &p,
        &glasnost_render::palette::PaletteQuirks::default(),
    );
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

async fn read_icons(vfs: &Vfs, loaded: &Loaded) -> BTreeMap<hud::Entry, IndexedImage> {
    let mut out = BTreeMap::new();
    let Some(rules) = loaded.rules.as_deref() else {
        return out;
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
    out
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
