//! The map viewer pane: map list, the battlefield view with scrolling
//! (S-0101 scroll limits), a minimap, layer toggles and object info with
//! `rules explain`; with a live sandbox game, the game's pacing controls,
//! the controlled house, selection and orders.
//!
//! Mouse and keys in the view, with a live game: a left click on an object
//! selects it through the input rules (with shift, force-select, it toggles
//! the object in the selection); a left click on the ground clears the
//! selection. A right click on a cell or object gives the selection's orders
//! through the same rules (shift: force-select, ctrl: force-fire, alt:
//! force-move). S, X, G and D are stop, scatter, guard and deploy, and C is
//! the house's cheer (S-0213 table T2). Dragging and the arrow keys scroll.
//!
//! The battlefield is drawn by the GPU map pass (`glasnost_render::map_gpu`)
//! into a texture the shell gives this pane each frame, or on the CPU by the
//! reference compositor when chosen.

use std::rc::Rc;

use egui::{Color32, RichText, TextureHandle, TextureOptions, Ui};
use glasnost_render::compose::{ComposeQuirks, Frame};
use glasnost_render::light::DetailLevel;
use glasnost_render::projection::{
    ProjectionQuirks, cell_centre, cursor_cell, limit_centre, project_lifted, view_origin,
};
use glasnost_render::scene::{Filter, Group, Scene, View};
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target};
use glasnost_vfs::FileRef;

use crate::mapview::build::{self, ObjectInfo};
use crate::mapview::find::Listing;
use crate::mapview::live::{self, InputReport, Live, Pacer};
use crate::mapview::load::Loaded;

/// What the pane asks the browser to do.
pub enum MapAction {
    /// Find the maps of the mounted install.
    Find,
    /// Load a map with these options.
    Load(FileRef, String, build::Options),
}

/// A request for the shell's GPU map pass.
pub struct GpuRequest {
    /// The scene (its generation tells the shell when to re-upload).
    pub scene: Rc<Scene>,
    /// Scene generation.
    pub generation: u64,
    /// View.
    pub view: View,
    /// Filter.
    pub filter: Filter,
}

/// A loaded map as the pane keeps it.
struct Shown {
    loaded: Loaded,
    scene: Rc<Scene>,
    generation: u64,
    minimap: Option<TextureHandle>,
    /// The controlled house's selection, in selection order (input state,
    /// S-0213 Terms).
    selection: Vec<ObjectId>,
    /// The house the user controls.
    house: HouseId,
    /// Tick pacing (S-0201).
    pacer: Pacer,
    /// Game-speed index for pacing (S-0201 claim 8: 0 fastest … 6 slowest).
    speed: u8,
    /// Paused (no ticks).
    paused: bool,
    /// One tick requested while paused.
    step_once: bool,
}

/// The pane.
pub struct MapPane {
    listings: Vec<Listing>,
    filter_text: String,
    multiplayer_only: bool,
    selected: Option<usize>,
    options: build::Options,
    shown: Option<Shown>,
    generation: u64,
    centre: (i32, i32),
    reference: i32,
    view_size: (u32, u32),
    groups: Vec<Group>,
    provisional: bool,
    use_gpu: bool,
    cpu_texture: Option<TextureHandle>,
    gpu_texture: Option<egui::TextureId>,
    hover: Option<(i32, i32, Option<u32>)>,
    pinned: Option<u32>,
    explain: String,
    pending_jump: Option<(i32, i32)>,
    /// The cheer rate limit, kept for the whole session (S-0213 claim 19).
    cheer: targeting::CheerMemory,
    /// Status line.
    pub status: String,
}

impl Default for MapPane {
    fn default() -> Self {
        Self {
            listings: Vec::new(),
            filter_text: String::new(),
            multiplayer_only: true,
            selected: None,
            options: build::Options::default(),
            shown: None,
            generation: 0,
            centre: (0, 0),
            reference: Frame::reference_at_reset(0, &ComposeQuirks::default()),
            view_size: (960, 600),
            groups: Group::ALL.to_vec(),
            provisional: true,
            use_gpu: true,
            cpu_texture: None,
            gpu_texture: None,
            hover: None,
            pinned: None,
            explain: String::new(),
            pending_jump: None,
            cheer: targeting::CheerMemory::default(),
            status: "Mount an install, then find its maps.".into(),
        }
    }
}

/// Pixels scrolled per arrow-key press (our own viewer choice).
const KEY_SCROLL: i32 = 60;

impl MapPane {
    /// The maps found.
    pub fn set_listings(&mut self, listings: Vec<Listing>) {
        self.status = format!(
            "{} maps found ({} multiplayer).",
            listings.len(),
            listings.iter().filter(|l| l.multiplayer).count()
        );
        self.listings = listings;
    }

    /// A map finished loading.
    pub fn set_loaded(&mut self, loaded: Loaded) {
        self.generation += 1;
        let map = &loaded.map;
        // spec: S-0009 claim 16 (the initial camera centres on the home waypoint cell)
        let home = map
            .waypoints
            .get(&map.home_waypoint)
            .copied()
            .unwrap_or((0, 0));
        let (lx, ly) = cell_centre(home.0.into(), home.1.into());
        let start = project_lifted(lx, ly, 0);
        self.status = format!(
            "{}: {} × {}, {} items, {} objects, {} map warnings, {} problems{}.",
            String::from_utf8_lossy(&map.basic.name),
            map.footprint.width,
            map.footprint.height,
            loaded.built.scene.items.len(),
            loaded.built.objects.len(),
            loaded.map_warnings.len(),
            loaded.problems.len(),
            match &loaded.live {
                Some(l) => format!(", sandbox game with {} houses", l.world.houses.len()),
                None => ", no game".into(),
            }
        );
        let mut loaded = loaded;
        if let Some(live) = &loaded.live {
            // spec: S-0213 claim 19 (a game start may reset the cheer memory)
            self.cheer.game_start(&live.world);
        }
        let scene = Rc::new(std::mem::take(&mut loaded.built.scene));
        let (house, speed) = loaded
            .live
            .as_ref()
            .map_or((HouseId(0), 0), |l| (l.default_house(), l.setup.game_speed));
        self.shown = Some(Shown {
            scene,
            generation: self.generation,
            loaded,
            minimap: None,
            selection: Vec::new(),
            house,
            pacer: Pacer::default(),
            speed,
            paused: false,
            step_once: false,
        });
        self.set_centre(start, true);
        self.pinned = None;
    }

    fn footprint(&self) -> Option<&glasnost_map::Footprint> {
        self.shown.as_ref().map(|s| &s.loaded.map.footprint)
    }

    /// Sets the view centre, limited by S-0101 F5. A jump resets the depth
    /// reference (S-0102 F1).
    fn set_centre(&mut self, requested: (i32, i32), jump: bool) {
        let Some(f) = self.footprint().copied() else {
            return;
        };
        let size = (self.view_size.0 as i32, self.view_size.1 as i32);
        // spec: S-0101 claim 8 (every request is limited by F5; the stored centre is the limited one)
        let c = limit_centre(
            requested,
            size,
            f.width,
            f.local,
            &ProjectionQuirks::default(),
        );
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

    fn view(&self) -> View {
        let size = (self.view_size.0 as i32, self.view_size.1 as i32);
        View {
            origin: view_origin(self.centre, size),
            size: self.view_size,
            reference: self.reference,
        }
    }

    fn filter(&self) -> Filter {
        Filter {
            groups: self.groups.clone(),
            provisional: self.provisional,
        }
    }

    /// What the shell should draw on the GPU this frame, if anything.
    pub fn gpu_request(&self) -> Option<GpuRequest> {
        let s = self.shown.as_ref()?;
        if !self.use_gpu {
            return None;
        }
        Some(GpuRequest {
            scene: s.scene.clone(),
            generation: s.generation,
            view: self.view(),
            filter: self.filter(),
        })
    }

    /// Chooses the GPU map pass or the CPU reference.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn set_use_gpu(&mut self, on: bool) {
        self.use_gpu = on;
    }

    /// The texture the shell drew this frame's GPU view into.
    pub fn set_gpu_texture(&mut self, id: Option<egui::TextureId>) {
        self.gpu_texture = id;
    }

    /// Draws the pane. Returns an action for the browser.
    pub fn ui(&mut self, ui: &mut Ui, mounted: bool, busy: bool) -> Option<MapAction> {
        let mut action = None;
        egui::Panel::left("maps")
            .resizable(true)
            .default_size(300.0)
            .show(ui, |ui| {
                action = self.list_ui(ui, mounted, busy);
            });
        egui::Panel::right("map info")
            .resizable(true)
            .default_size(320.0)
            .show(ui, |ui| self.info_ui(ui));
        egui::CentralPanel::default().show(ui, |ui| self.view_ui(ui));
        action
    }

    fn list_ui(&mut self, ui: &mut Ui, mounted: bool, busy: bool) -> Option<MapAction> {
        let mut action = None;
        ui.heading("Maps");
        if ui
            .add_enabled(mounted && !busy, egui::Button::new("Find maps"))
            .clicked()
        {
            action = Some(MapAction::Find);
        }
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter_text);
        });
        ui.checkbox(&mut self.multiplayer_only, "multiplayer maps only");
        ui.separator();
        ui.label(RichText::new("Viewer options (not game data)").strong());
        ui.checkbox(
            &mut self.options.skirmish,
            "skirmish view (S-0009 claim 27)",
        )
        .on_hover_text("When the sandbox game starts, every owner's objects are shown: the game places them all.");
        egui::ComboBox::from_label("detail level (S-0104)")
            .selected_text(format!("{:?}", self.options.detail))
            .show_ui(ui, |ui| {
                for d in [DetailLevel::Low, DetailLevel::Medium, DetailLevel::High] {
                    ui.selectable_value(&mut self.options.detail, d, format!("{d:?}"));
                }
            });
        ui.add(egui::Slider::new(&mut self.options.human_slot, 0..=7).text("human colour slot"));
        ui.add(
            egui::Slider::new(&mut self.options.computer_slot, 0..=7).text("computer colour slot"),
        );
        ui.separator();
        let needle = self.filter_text.to_ascii_lowercase();
        let mut load = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, l) in self.listings.iter().enumerate() {
                if self.multiplayer_only && !l.multiplayer {
                    continue;
                }
                let label = format!("{} ({}, {})", l.name, l.theater, l.archive);
                if !needle.is_empty() && !label.to_ascii_lowercase().contains(&needle) {
                    continue;
                }
                let r = ui.selectable_label(self.selected == Some(i), label);
                if r.clicked() {
                    self.selected = Some(i);
                }
                if r.double_clicked() {
                    load = Some(i);
                }
            }
        });
        if let Some(i) = load.or_else(|| {
            ui.add_enabled(!busy && self.selected.is_some(), egui::Button::new("Load"))
                .clicked()
                .then_some(self.selected)
                .flatten()
        }) && let Some(l) = self.listings.get(i)
        {
            action = Some(MapAction::Load(
                l.file,
                l.archive.clone(),
                self.options.clone(),
            ));
            self.status = format!("Loading {}…", l.name);
        }
        action
    }

    fn info_ui(&mut self, ui: &mut Ui) {
        ui.label(&self.status);
        ui.separator();
        ui.label(RichText::new("Layers").strong());
        for g in Group::ALL {
            let mut on = self.groups.contains(&g);
            if ui.checkbox(&mut on, g.label()).changed() {
                if on {
                    self.groups.push(g);
                } else {
                    self.groups.retain(|x| *x != g);
                }
            }
        }
        ui.checkbox(
            &mut self.provisional,
            "draw provisional items (spec requests)",
        );
        ui.checkbox(&mut self.use_gpu, "GPU map pass (off: CPU reference)");
        let Some(shown) = &mut self.shown else {
            return;
        };
        game_ui(ui, shown, &mut self.status);
        ui.separator();
        let (w, h, px) = &shown.loaded.minimap;
        if *w > 0 && *h > 0 {
            let tex = shown.minimap.get_or_insert_with(|| {
                ui.ctx().load_texture(
                    "minimap",
                    egui::ColorImage::from_rgba_unmultiplied([*w as usize, *h as usize], px),
                    TextureOptions::NEAREST,
                )
            });
            let scale = (300.0 / *w as f32).min(2.0);
            let r = ui.add(
                egui::Image::new((tex.id(), egui::vec2(*w as f32 * scale, *h as f32 * scale)))
                    .sense(egui::Sense::click_and_drag()),
            );
            if let Some(p) = r.interact_pointer_pos()
                && (r.clicked() || r.dragged())
            {
                // Minimap pixel (x, y) ↔ cell: x = d + W − 1, y = s − W − 1 (our layout).
                let f = shown.loaded.map.footprint;
                let mx = ((p.x - r.rect.min.x) / scale) as i32;
                let my = ((p.y - r.rect.min.y) / scale) as i32;
                let d = mx - f.width + 1;
                let s = my + f.width + 1;
                let (cx, cy) = ((s + d) / 2, (s - d) / 2);
                let (lx, ly) = cell_centre(cx, cy);
                let target = project_lifted(lx, ly, 0);
                self.pending_jump = Some(target);
            }
        }
        ui.separator();
        ui.label(RichText::new("Provisional choices and gaps").strong());
        for n in &shown.loaded.built.notes {
            ui.label(RichText::new(n).small());
        }
        for (k, v) in &shown.loaded.built.dropped {
            ui.label(RichText::new(format!("dropped: {k}: {v}")).small());
        }
        for p in &shown.loaded.problems {
            ui.colored_label(Color32::YELLOW, p);
        }
        ui.collapsing(
            format!("map warnings ({})", shown.loaded.map_warnings.len()),
            |ui| {
                for w in &shown.loaded.map_warnings {
                    ui.label(RichText::new(w).small());
                }
            },
        );
        ui.separator();
        ui.label(RichText::new("Object").strong());
        let hovered = self.hover.and_then(|h| h.2);
        if let Some((cx, cy, _)) = self.hover {
            ui.label(format!("cell under the cursor (S-0101 F8): ({cx}, {cy})"));
        }
        if let Some(live) = &shown.loaded.live {
            // The selection's first object, else the one under the cursor.
            let id = shown
                .selection
                .first()
                .copied()
                .or_else(|| hovered.and_then(|c| live.object_of_code(c)));
            if shown.selection.len() > 1 {
                ui.label(format!("{} objects selected", shown.selection.len()));
            }
            if let Some(id) = id {
                sim_object_ui(ui, live, id);
            }
        }
        let object = self.pinned.or(hovered);
        if let Some(o) = object
            .filter(|o| o & live::SIM_OBJECT == 0)
            .and_then(|o| shown.loaded.built.objects.get(o as usize))
        {
            object_ui(ui, o);
            if ui.button("rules explain").clicked() {
                self.explain = explain(&shown.loaded, &o.type_name);
            }
        } else if let Some(live) = &shown.loaded.live
            && let Some(o) = shown.selection.first().and_then(|&id| live.world.obj(id))
            && ui.button("rules explain").clicked()
        {
            self.explain = explain(
                &shown.loaded,
                &live::type_name(&live.rules, o.kind, o.type_index),
            );
        }
        if !self.explain.is_empty() {
            egui::ScrollArea::vertical()
                .max_height(240.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(&self.explain).monospace().small());
                });
        }
    }

    fn view_ui(&mut self, ui: &mut Ui) {
        if let Some(t) = self.pending_jump.take() {
            self.set_centre(t, true);
        }
        let avail = ui.available_size();
        let ppp = ui.ctx().pixels_per_point();
        let size = (
            ((avail.x * ppp) as u32).max(64),
            ((avail.y * ppp) as u32).max(64),
        );
        if size != self.view_size {
            self.view_size = size;
            let c = self.centre;
            self.set_centre(c, false);
        }
        if self.shown.is_none() {
            ui.centered_and_justified(|ui| ui.label("No map loaded."));
            return;
        }
        if let Some(shown) = self.shown.as_mut() {
            run_ticks(ui.ctx(), shown);
        }
        let Some(scene) = self.shown.as_ref().map(|s| s.scene.clone()) else {
            return;
        };
        let display = egui::vec2(size.0 as f32 / ppp, size.1 as f32 / ppp);
        let texture = if self.use_gpu {
            self.gpu_texture
        } else {
            let frame = scene.compose(&self.view(), &self.filter(), &ComposeQuirks::default());
            let img = egui::ColorImage::from_rgba_unmultiplied(
                [size.0 as usize, size.1 as usize],
                &frame.to_rgba(),
            );
            match &mut self.cpu_texture {
                Some(t) => t.set(img, TextureOptions::NEAREST),
                None => {
                    self.cpu_texture = Some(ui.ctx().load_texture(
                        "map view",
                        img,
                        TextureOptions::NEAREST,
                    ))
                }
            }
            self.cpu_texture.as_ref().map(TextureHandle::id)
        };
        let Some(texture) = texture else {
            ui.label("Waiting for the GPU view…");
            return;
        };
        let r = ui.add(egui::Image::new((texture, display)).sense(egui::Sense::click_and_drag()));
        let mut c = self.centre;
        if r.dragged() {
            let d = r.drag_delta() * ppp;
            c.0 -= d.x as i32;
            c.1 -= d.y as i32;
        }
        if r.hovered() {
            ui.input(|i| {
                if i.key_down(egui::Key::ArrowLeft) {
                    c.0 -= KEY_SCROLL / 4;
                }
                if i.key_down(egui::Key::ArrowRight) {
                    c.0 += KEY_SCROLL / 4;
                }
                if i.key_down(egui::Key::ArrowUp) {
                    c.1 -= KEY_SCROLL / 4;
                }
                if i.key_down(egui::Key::ArrowDown) {
                    c.1 += KEY_SCROLL / 4;
                }
            });
        }
        if c != self.centre {
            self.set_centre(c, false);
            ui.ctx().request_repaint();
        }
        drop(scene);
        let view = self.view();
        let filter = self.filter();
        let Some(shown) = self.shown.as_mut() else {
            return;
        };
        let to_world = |p: egui::Pos2| {
            (
                view.origin.0 + ((p.x - r.rect.min.x) * ppp) as i32,
                view.origin.1 + ((p.y - r.rect.min.y) * ppp) as i32,
            )
        };
        self.hover = r.hover_pos().map(|p| {
            let (qx, qy) = to_world(p);
            let map = &shown.loaded.map;
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
                        .get(glasnost_map::CellXY::new(x, y))
                        .map_or(0, |c| i32::from(c.level))
                },
                max_level,
                &q,
            );
            (cell.0, cell.1, shown.scene.object_at(qx, qy, &filter))
        });
        let hover = self.hover;
        let mods = ui.input(|i| Modifiers {
            force_select: i.modifiers.shift,
            force_fire: i.modifiers.ctrl || i.modifiers.mac_cmd,
            force_move: i.modifiers.alt,
            attack_move_mode: false,
        });
        let house = shown.house;
        let mut report = None;
        if let Some(live) = shown.loaded.live.as_mut() {
            let target = hover.map(|(cx, cy, code)| {
                match code
                    .and_then(|c| live.object_of_code(c))
                    .filter(|&id| targeting::selectable(&live.world, id))
                {
                    Some(id) => ClickOn::Object(id),
                    None => ClickOn::Cell(glasnost_map::CellXY::new(cx, cy)),
                }
            });
            if r.clicked() {
                match target {
                    Some(on @ ClickOn::Object(_)) => {
                        // A left click selects: without force-select it acts
                        // as if nothing were selected, so it never orders.
                        let mut sel = if mods.force_select {
                            std::mem::take(&mut shown.selection)
                        } else {
                            Vec::new()
                        };
                        let rep = live.click(house, &mut sel, on, mods);
                        shown.selection = sel;
                        report = Some(rep);
                    }
                    _ => shown.selection.clear(),
                }
            }
            if r.secondary_clicked()
                && let Some(on) = target
            {
                report = Some(live.click(house, &mut shown.selection, on, mods));
            }
            if r.hovered() && !ui.ctx().egui_wants_keyboard_input() {
                let keys = [
                    (egui::Key::S, KeyCommand::Stop),
                    (egui::Key::X, KeyCommand::Scatter),
                    (egui::Key::G, KeyCommand::Guard),
                    (egui::Key::D, KeyCommand::Deploy),
                ];
                for (key, cmd) in keys {
                    if ui.input(|i| i.key_pressed(key)) {
                        report = Some(live.key(house, &mut shown.selection, cmd));
                    }
                }
                // The cheer command (S-0213 table T2, claim 19).
                if ui.input(|i| i.key_pressed(egui::Key::C)) {
                    let order = self.cheer.cheer(&live.world);
                    report = Some(live.issue_orders(house, order.into_iter().collect()));
                }
            }
            live.prune(&mut shown.selection);
            selection_markers(ui, live, &shown.selection, &view, r.rect, ppp);
        } else if r.clicked() {
            self.pinned = hover.and_then(|h| h.2);
            self.explain.clear();
        }
        if let Some(rep) = report {
            self.status = input_status(&rep);
        }
    }
}

/// The status line after an input.
fn input_status(rep: &InputReport) -> String {
    match (rep.issued, rep.refused) {
        (0, 0) if rep.selection_changed => "Selection changed.".into(),
        (0, 0) => "No order.".into(),
        (n, 0) => format!("{n} orders issued."),
        (n, m) => format!(
            "{n} orders issued, {m} refused (orders are blocked during ticks 0 to 3, S-0201 claim 14)."
        ),
    }
}

/// Runs at most one tick when one is due (S-0201 claims 2 to 4), and
/// redraws the mobile objects after it.
fn run_ticks(ctx: &egui::Context, shown: &mut Shown) {
    let Some(live) = shown.loaded.live.as_ref() else {
        return;
    };
    if live.failed.is_some() {
        return;
    }
    let now_ms = (crate::viewer::input::now_seconds() * 1000.0) as i64;
    let tick = if shown.paused {
        shown.pacer.reset();
        std::mem::take(&mut shown.step_once)
    } else {
        // Frames keep coming while the game runs; the pacer picks the ones
        // that run a tick.
        ctx.request_repaint();
        shown.pacer.due(now_ms, shown.speed)
    };
    if !tick {
        return;
    }
    let scene = Rc::make_mut(&mut shown.scene);
    let (inputs, live) = shown.loaded.split();
    if let Some(live) = live {
        live.step();
        live.redraw(&inputs, scene);
    }
}

/// The game panel: tick, pacing, the controlled house, notes.
fn game_ui(ui: &mut Ui, shown: &mut Shown, status: &mut String) {
    let Some(live) = shown.loaded.live.as_ref() else {
        return;
    };
    ui.separator();
    ui.label(RichText::new("Sandbox game").strong());
    ui.label(format!(
        "tick {}; pending orders {}",
        live.world.tick,
        live.world
            .houses
            .iter()
            .map(|h| h.pending.len())
            .sum::<usize>()
    ));
    if let Some(f) = &live.failed {
        ui.colored_label(Color32::RED, f);
    }
    ui.horizontal(|ui| {
        ui.checkbox(&mut shown.paused, "paused");
        if ui
            .add_enabled(shown.paused, egui::Button::new("one tick"))
            .clicked()
        {
            shown.step_once = true;
        }
    });
    ui.add(
        egui::Slider::new(&mut shown.speed, 0..=6)
            .text("game speed index (S-0201: 0 fastest, 6 slowest)"),
    );
    ui.label(
        RichText::new(format!(
            "paced period {} ms; the game's own speed setting stays at {}",
            live::period_ms(shown.speed),
            live.world.game_speed
        ))
        .small()
        .weak(),
    );
    let name = |h: HouseId| {
        live.world
            .houses
            .get(usize::from(h.0))
            .map_or_else(String::new, |x| x.name.clone())
    };
    let before = shown.house;
    egui::ComboBox::from_label("controlled house")
        .selected_text(name(shown.house))
        .show_ui(ui, |ui| {
            for i in 0..live.world.houses.len() {
                let h = HouseId(i as u8);
                ui.selectable_value(&mut shown.house, h, name(h));
            }
        });
    if shown.house != before {
        shown.selection.clear();
        *status = format!("You control {}.", name(shown.house));
    }
    if !live.substituted.is_empty() {
        ui.label(
            RichText::new(format!(
                "houses given the first country (no country of their name): {}",
                live.substituted.join(", ")
            ))
            .small(),
        );
    }
    if !live.undrawn_types.is_empty() {
        ui.label(
            RichText::new(format!(
                "simulation types with no import type of their name (not drawn): {}",
                live.undrawn_types
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
            .small(),
        );
    }
    let notes = live.world.notes.len() + live.notes.len();
    ui.collapsing(format!("game and drawing notes ({notes})"), |ui| {
        for n in &live.world.notes {
            ui.label(RichText::new(n).small());
        }
        for n in &live.notes {
            ui.label(RichText::new(n).small());
        }
    });
}

/// Info about a simulation object.
fn sim_object_ui(ui: &mut Ui, live: &Live, id: ObjectId) {
    let w = &live.world;
    let Some(o) = w.obj(id) else {
        return;
    };
    ui.label(format!(
        "{} {} (object {})",
        glasnost_sim::load::kind_name(o.kind),
        live::type_name(&live.rules, o.kind, o.type_index),
        id.0
    ));
    let owner = o
        .owner
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .map_or("none", |h| h.name.as_str());
    ui.label(format!("owner: {owner}"));
    let c = o.pos.cell();
    ui.label(format!(
        "cell: ({}, {}); position ({}, {}, {}) leptons",
        c.x, c.y, o.pos.x, o.pos.y, o.pos.z
    ));
    ui.label(format!(
        "behaviour: {}",
        o.beh.current.map_or("none", |b| b.key())
    ));
    let target = |t: Option<Target>| match t {
        Some(Target::Cell(c)) => format!("cell ({}, {})", c.x, c.y),
        Some(Target::Object(x)) => format!("object {}", x.0),
        None => "none".into(),
    };
    ui.label(format!("destination: {}", target(o.dest)));
    if o.target.is_some() {
        ui.label(format!("target: {}", target(o.target)));
    }
    ui.label(format!(
        "health {}; body facing {}",
        o.health,
        o.body.value(w.tick)
    ));
    if let Some(s) = &o.infantry {
        ui.label(format!("sequence: {}", s.seq.map_or("none", |q| q.key())));
    }
}

/// Our marker for selected objects: corner brackets around the object's
/// point (presentation only, no spec).
fn selection_markers(
    ui: &Ui,
    live: &Live,
    selection: &[ObjectId],
    view: &View,
    rect: egui::Rect,
    ppp: f32,
) {
    let painter = ui.painter_at(rect);
    let stroke = egui::Stroke::new(1.5, Color32::from_rgb(255, 230, 80));
    for &id in selection {
        let Some(o) = live.world.obj(id) else {
            continue;
        };
        let p = live.world.reference_point(id);
        let (qx, qy) = project_lifted(p.x, p.y, p.z);
        let (half_w, half_h) = match o.kind {
            glasnost_sim::rules::Kind::Infantry => (8.0, 14.0),
            glasnost_sim::rules::Kind::Building => {
                let (fw, fh) = o.foundation;
                (15.0 * (fw + fh) as f32, 8.0 * (fw + fh) as f32 + 20.0)
            }
            _ => (16.0, 14.0),
        };
        let centre = egui::pos2(
            rect.min.x + (qx - view.origin.0) as f32 / ppp,
            rect.min.y + (qy - view.origin.1 - 8) as f32 / ppp,
        );
        let (hw, hh) = (half_w / ppp, half_h / ppp);
        let arm = hw.min(hh) * 0.5;
        for (sx, sy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            let corner = egui::pos2(centre.x + sx * hw, centre.y + sy * hh);
            painter.line_segment([corner, egui::pos2(corner.x - sx * arm, corner.y)], stroke);
            painter.line_segment([corner, egui::pos2(corner.x, corner.y - sy * arm)], stroke);
        }
    }
}

fn object_ui(ui: &mut Ui, o: &ObjectInfo) {
    ui.label(format!("{} {}", o.kind, o.type_name));
    if !o.owner.is_empty() {
        ui.label(format!("owner: {}", o.owner));
    }
    ui.label(format!("cell: ({}, {})", o.cell.x, o.cell.y));
    if !o.entry.is_empty() {
        ui.label(RichText::new(format!("entry: {}", o.entry)).small());
    }
    for n in &o.notes {
        ui.label(RichText::new(n).small().weak());
    }
}

/// `rules explain` for a type: the history of every value of its type
/// entry in the converted tree.
fn explain(loaded: &Loaded, type_name: &str) -> String {
    let tree = &loaded.import.tree;
    let Some(path) = glasnost_rules::explain::find_path(tree, type_name) else {
        return format!("{type_name} is not in the converted rules");
    };
    glasnost_rules::explain::explain(tree, &loaded.import.provenance, &path)
}
