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
use glasnost_render::compose::{ComposeQuirks, Frame};
use glasnost_render::projection::{
    ProjectionQuirks, TERRAIN_LIFT, cell_top, cursor_cell, limit_centre, project_lifted,
    view_origin,
};
use glasnost_render::scene::{Filter, Group, Scene, View};
use glasnost_render::shroud::ShroudQuirks;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::rules::Kind as SimKind;
use glasnost_sim::targeting::{KeyCommand, Modifiers};
use glasnost_sim::world::{GameSetup, ObjectId};
use glasnost_ui::draw::{DrawList, colour};
use glasnost_ui::font;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::hud::{self, HudHit, Layout, PauseHit};
use glasnost_ui::widget::Targets;
use glasnost_vfs::{FileRef, Vfs};

use super::paint::Painter;
use super::play::{Button, Play, Tool, local_house};
use super::scroll::{self, EdgeScroll};
use crate::mapview::build::{self, GameMode, Inputs, Options};
use crate::mapview::cover;
use crate::mapview::live::{self, Live};
use crate::mapview::load::{self, Loaded};
use crate::mapview::radar::Radar;

/// Image id of the battlefield in the client's draw lists.
pub const MAP_IMAGE: u32 = 1;
/// Image id of the radar map.
pub const RADAR_IMAGE: u32 = 2;
/// First sprite id of the sidebar icons.
const ICON_SPRITES: u32 = 1000;
/// A left-button drag longer than this (in either direction) is a box
/// selection. Interim (Q-0278).
pub const DRAG_START: i32 = 4;

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
    centre: (i32, i32),
    reference: i32,
    filter: Filter,
    /// Decoded icon frames by type (S-0007 claims 22 and 25).
    icons: BTreeMap<TypeRef, IndexedImage>,
    /// Sprite ids of the registered icons.
    icon_ids: BTreeMap<TypeRef, u32>,
    /// The icon palette's RGBA (interim, Q-0275).
    icon_palette: Option<[u8; 1024]>,
    pointer: Option<Point>,
    /// Edge momentum and its retained direction (S-0101 F9).
    edge: EdgeScroll,
    press: Option<(Point, Button)>,
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
        let icon_palette = loaded
            .assets
            .get(crate::mapview::combat::OVERLAY_PALETTE)
            .and_then(|b| glasnost_fmt::pal::Palette6::parse(b).ok())
            .map(|p| {
                // spec: S-0103 claim 2 (a palette file's components through the load rule)
                let pal = glasnost_render::palette::palette_from_file(
                    &p,
                    &glasnost_render::palette::PaletteQuirks::default(),
                );
                pal.to_rgba8()
            });
        let scene = std::mem::take(&mut loaded.built.scene);
        let mut problems = std::mem::take(&mut loaded.problems);
        let Some(live) = loaded.live.as_mut() else {
            return Err(problems
                .pop()
                .unwrap_or_else(|| "the game did not start".into()));
        };
        live.clear_scripts();
        let setup: GameSetup = live.setup.clone();
        let catalogue = live
            .world
            .catalogue
            .clone()
            .ok_or("the game has no tile catalogue")?;
        let game = Game::new(live.rules.clone(), &loaded.map, &map_id, catalogue, setup.clone())
            .map_err(|e| format!("the game did not start: {e}"))?;
        debug_assert_eq!(
            glasnost_sim::save::state_hash(&game.world),
            glasnost_sim::save::state_hash(&live.world),
            "the drawn copy and the game start alike"
        );
        let local = local_house(&setup).ok_or("the setup has no person's house")?;
        let radar = Radar::new(&game.world, loaded.map.theater, &loaded.tile_colours);
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
            centre: (0, 0),
            reference: Frame::reference_at_reset(0, &ComposeQuirks::default()),
            filter: Filter {
                groups: Group::ALL.to_vec(),
                provisional: true,
            },
            icons,
            icon_ids: BTreeMap::new(),
            icon_palette,
            pointer: None,
            edge: EdgeScroll::default(),
            press: None,
            targets: Targets::new(),
            pause_targets: Targets::new(),
            problems: Vec::new(),
        };
        problems.append(&mut s.loaded.problems);
        s.problems = problems;
        s.set_centre(start.unwrap_or((0, 0)), true);
        s.play.say("Game started.");
        Ok(s)
    }

    /// Calls `f` with the live drawing state holding the canonical world.
    pub fn with_live<R>(&mut self, f: impl FnOnce(&mut Live, &Inputs, &mut Scene) -> R) -> Option<R> {
        let (inputs, live) = self.loaded.split();
        let live = live?;
        std::mem::swap(&mut live.world, &mut self.play.game.world);
        let r = f(live, &inputs, &mut self.scene);
        std::mem::swap(&mut live.world, &mut self.play.game.world);
        Some(r)
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

    /// The battlefield's rectangle on the screen.
    pub fn map_rect(&self) -> Rect {
        Layout::new(self.screen).map
    }

    /// Sets the window size.
    pub fn resize(&mut self, size: (i32, i32)) {
        if size != self.screen {
            self.screen = size;
            let c = self.centre;
            self.set_centre(c, false);
        }
    }

    fn view_size(&self) -> (i32, i32) {
        let r = self.map_rect();
        (r.w.max(1), r.h.max(1))
    }

    /// Sets the view centre, limited by S-0101 F5; a jump resets the depth
    /// reference (S-0102 F1).
    pub fn set_centre(&mut self, requested: (i32, i32), jump: bool) {
        let f = self.loaded.map.footprint;
        let size = self.view_size();
        // spec: S-0101 claim 8 (every request is limited by F5; the stored centre is the limited one)
        let c = limit_centre(requested, size, f.width, f.local, &ProjectionQuirks::default());
        let (dx, dy) = (c.0 - self.centre.0, c.1 - self.centre.1);
        // spec: S-0102 Terms (a view jump moves by more than the view's width across or its height up or down)
        let big = dx.abs() > size.0 || dy.abs() > size.1;
        self.centre = c;
        if jump || big {
            // spec: S-0102 F1 (at a reset, C = 32768 + the world row of the view's top; quirks.depth_reference_reset_on_jump)
            self.reference =
                Frame::reference_at_reset(view_origin(c, size).1, &ComposeQuirks::default());
        }
    }

    /// Scrolls the view by `d` pixels.
    pub fn scroll(&mut self, d: (i32, i32)) {
        if d != (0, 0) {
            let c = (self.centre.0 + d.0, self.centre.1 + d.1);
            self.set_centre(c, false);
        }
    }

    /// One unfrozen cycle's scrolling (S-0101 claims 12 to 14): the edge
    /// request of F9 plus the held arrows' (left, right, up, down), added
    /// before F5 limits the centre. `edge` is false while edge scrolling is
    /// suppressed (no focus).
    pub fn scroll_cycle(&mut self, now_ms: u64, held: [bool; 4], edge: bool) {
        let pointer = self.pointer.filter(|_| edge).map(|p| (p.x, p.y));
        let f = self.loaded.map.footprint;
        let size = self.view_size();
        let centre = self.centre;
        // spec: S-0101 F9 (a one-pixel request in the direction blocked by F5)
        let blocked = |d: (i32, i32)| {
            limit_centre(
                (centre.0 + d.0, centre.1 + d.1),
                size,
                f.width,
                f.local,
                &ProjectionQuirks::default(),
            ) == centre
        };
        let e = self.edge.cycle(
            pointer,
            self.screen,
            scroll::DEFAULT_SCROLL_INDEX,
            self.loaded.scroll_multiplier,
            now_ms,
            blocked,
        );
        let a = scroll::arrows(held);
        // spec: S-0101 claim 14 (mouse and arrow requests add before the next view position, subject to F5)
        self.scroll((e.0 + a.0, e.1 + a.1));
    }

    /// The map view.
    pub fn view(&self) -> View {
        let size = self.view_size();
        View {
            origin: view_origin(self.centre, size),
            size: (size.0 as u32, size.1 as u32),
            reference: self.reference,
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
        self.icons.get(&t)
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
        let code = self.scene.object_at(qx, qy, &self.filter);
        let id = code.and_then(|c| self.with_live(|live, _, _| live.object_of_code(c)).flatten());
        (CellXY::new(cell.0, cell.1), id)
    }

    /// The pointer moved.
    pub fn pointer_moved(&mut self, p: Point) {
        self.pointer = Some(p);
    }

    /// The pointer left the window.
    pub fn pointer_left(&mut self) {
        self.pointer = None;
        self.press = None;
    }

    /// A button went down.
    pub fn pointer_down(&mut self, b: Button, p: Point) {
        self.pointer = Some(p);
        self.press = Some((p, b));
    }

    /// A button went up: a click or the end of a drag.
    pub fn pointer_up(&mut self, b: Button, p: Point, mods: Modifiers) -> SessionEvent {
        self.pointer = Some(p);
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
        let dragged = (p.x - start.x).abs() > DRAG_START || (p.y - start.y).abs() > DRAG_START;
        match self.targets.hit(start) {
            Some(HudHit::Map) if dragged && b == Button::Left && self.play.tool == Tool::Select => {
                let inside = self.objects_in(Rect::spanning(start, p));
                self.play.select_box(&inside, mods.force_select);
            }
            Some(HudHit::Map) if !dragged => {
                let (cell, object) = self.under(p);
                self.play.click_map(b, object, cell, mods);
            }
            Some(HudHit::Radar) if b == Button::Left => {
                let l = Layout::new(self.screen);
                if let Some(cell) = self.radar.as_ref().and_then(|r| {
                    r.cell_at(&self.play.game.world, p.x - l.radar.x, p.y - l.radar.y)
                }) {
                    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
                    self.set_centre(project_lifted(x, y, 0), true);
                }
            }
            Some(HudHit::Tab(i)) => self.play.tab_click(i),
            Some(HudHit::Cameo(n)) => self.play.cameo_click(n, b, mods.force_select),
            Some(HudHit::Scroll(d)) => {
                let rows = Layout::new(self.screen).rows();
                self.play.scroll_icons(d, rows);
            }
            Some(HudHit::Mode(m)) => self.play.mode_click(m),
            Some(HudHit::Options) => self.paused = true,
            _ => {}
        }
        SessionEvent::None
    }

    /// The local player's objects whose reference point projects inside
    /// `r` (screen pixels of the battlefield).
    fn objects_in(&self, r: Rect) -> Vec<ObjectId> {
        let w = &self.play.game.world;
        let v = self.view();
        let m = self.map_rect();
        w.objects
            .values()
            .filter(|o| o.owner == Some(self.play.local) && o.on_map)
            .filter(|o| {
                let p = w.reference_point(o.id);
                let (qx, qy) = project_lifted(p.x, p.y, p.z);
                r.contains(Point::new(qx - v.origin.0 + m.x, qy - v.origin.1 + m.y))
            })
            .map(|o| o.id)
            .collect()
    }

    /// A key command (S-0213 table T2).
    pub fn key(&mut self, cmd: KeyCommand) {
        if !self.paused {
            self.play.key(cmd);
        }
    }

    /// The options menu key: opens or closes the menu (S-0201 claim 8).
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Whether ticks may run now.
    pub fn ticking(&self) -> bool {
        !self.paused && self.play.running()
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
        let world_to_screen = |x: i32, y: i32| (x - view.origin.0 + m.x, y - view.origin.1 + m.y);
        // Placement preview (S-0411 claim 10, S-0406): green where the
        // building would be placed, red elsewhere.
        if let Some(p) = self.pointer.filter(|&p| m.contains(p)) {
            let (cell, _) = self.under(p);
            if let Some((cells, ok)) = self.play.preview(cell) {
                let fill = if ok { [60, 220, 80, 110] } else { [230, 50, 40, 110] };
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
                Some(l) => ids.into_iter().filter(|&id| !l.bracket_drawn(assets, id)).collect(),
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
        if let (Some((start, Button::Left)), Some(p)) = (self.press, self.pointer)
            && m.contains(start)
            && self.play.tool == Tool::Select
            && ((p.x - start.x).abs() > DRAG_START || (p.y - start.y).abs() > DRAG_START)
        {
            d.frame(Rect::spanning(start, p), colour::ACCENT);
        }
        let mut y = m.y + 4;
        for msg in &self.play.messages {
            let wdt = font::text_width(&msg.text) + 8;
            d.fill(Rect::new(m.x + 4, y, wdt, 13), [0, 0, 0, 150]);
            d.text(m.x + 8, y + 2, msg.text.clone(), colour::TEXT, m);
            y += 15;
        }
        if let Some(f) = &self.play.failed {
            d.fill(Rect::new(m.x, m.bottom() - 16, m.w, 16), [0, 0, 0, 200]);
            d.text(m.x + 4, m.bottom() - 13, f.clone(), colour::BAD, m);
        }
        let rules = self.play.game.world.rules().clone();
        let name = |t: TypeRef| live::type_name(&rules, t.kind, t.index);
        let ids = &self.icon_ids;
        let icons = &self.icons;
        let art = |t: TypeRef| {
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
        d
    }
}

/// A filled cell diamond with its top corner at (x, y): 60 × 30 pixels
/// (S-0101 F1), drawn as one-pixel rows, clipped to `clip`.
fn diamond(d: &mut DrawList, x: i32, y: i32, rgba: [u8; 4], clip: Rect) {
    // spec: S-0101 F1 (a cell's diamond: 60 × 30 pixels below its top corner)
    for row in 0..30 {
        let half = if row < 15 { 2 * row + 2 } else { 2 * (29 - row) + 2 };
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

/// Reads and decodes frame 0 of every buildable type's sidebar icon
/// (S-0007 claims 22 and 25: the main icon; the alternate one needs
/// infiltration, which no skirmish of M5b has).
async fn read_icons(vfs: &Vfs, loaded: &Loaded) -> BTreeMap<TypeRef, IndexedImage> {
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
                out.insert(TypeRef { kind, index }, img);
            }
        }
    }
    out
}
