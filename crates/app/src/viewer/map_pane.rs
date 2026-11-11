//! The map viewer pane: map list, the battlefield view with scrolling
//! (S-0101 scroll limits), a minimap, layer toggles and object info with
//! `rules explain`; with a live sandbox game, the game's pacing controls,
//! the controlled house, selection and orders.
//!
//! Mouse and keys in the view, with a live game (S-0214 claim 26): a left
//! click on a cell or object selects or gives the selection's orders, as the
//! input rules decide (shift: force-select, ctrl: force-fire, alt:
//! force-move); a right click clears the selection. S, X, G and D are stop,
//! scatter, guard and deploy, and C is the house's cheer (S-0213 table T2).
//! Dragging with either button and the arrow keys scroll (the viewer's own;
//! the game client follows S-0214 claim 27).
//!
//! The battlefield is drawn by the GPU map pass (`glasnost_render::map_gpu`)
//! into a texture the shell gives this pane each frame, or on the CPU by the
//! reference compositor when chosen.

use std::rc::Rc;

use egui::{Color32, RichText, TextureHandle, TextureOptions, Ui};
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE};
use glasnost_render::light::DetailLevel;
use glasnost_render::projection::{
    ProjectionQuirks, cell_centre, cursor_cell, limit_centre, project_lifted, view_origin,
};
use glasnost_render::scene::{Filter, Group, Scene, View};
use glasnost_render::shroud::ShroudQuirks;
use glasnost_sim::targeting::{self, ClickOn, KeyCommand, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target};
use glasnost_vfs::FileRef;

use crate::mapview::build::{self, GameMode, ObjectInfo};
use crate::mapview::cover;
use crate::mapview::credits::CreditCounter;
use crate::mapview::find::Listing;
use crate::mapview::live::{self, InputReport, Live, Pacer};
use crate::mapview::load::Loaded;
use crate::mapview::radar::{self, Radar};
use crate::mapview::sidebar::{self, Action, Tab};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::sidebar::Click;

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
    /// The light-alpha values of the view after the shroud covers (S-0110),
    /// row by row; `None`: 127 everywhere.
    pub alpha: Option<Vec<u8>>,
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
    /// The sidebar's credits counter (S-0416), moved once per tick run.
    credits: CreditCounter,
    /// The sidebar tab shown.
    tab: Tab,
    /// The building type in placement mode (S-0411 claim 4).
    placing: Option<TypeRef>,
    /// The radar map of the live game (S-0410).
    radar: Option<Radar>,
    /// The radar map's texture.
    radar_texture: Option<TextureHandle>,
    /// The tick the radar texture shows.
    radar_tick: i32,
    /// Draw the shroud covers (S-0110) of the controlled house.
    shroud: bool,
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

    /// The voxel buffer the shown map's drawing left, for the next map
    /// (S-0105 claim 34, S2: kept across games; a fresh one at program
    /// start).
    pub fn voxel_canvas(&self) -> glasnost_render::voxel::VoxelCanvas {
        self.shown
            .as_ref()
            .map(|s| s.loaded.voxel_canvas())
            .unwrap_or_default()
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
                Some(l) => format!(
                    ", {} game with {} houses",
                    if l.setup.sandbox {
                        "sandbox"
                    } else {
                        "skirmish"
                    },
                    l.world.houses.len()
                ),
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
        let radar = loaded.live.as_ref().and_then(|l| {
            Radar::new(
                &l.world,
                map_theater(&loaded),
                &loaded.tile_colours,
                loaded.house_radar_colours(&l.world),
                loaded.neutral_radar_colour(),
            )
        });
        // In a skirmish the view starts on the controlled house's base unit.
        let start = loaded
            .live
            .as_ref()
            .filter(|l| !l.setup.sandbox)
            .and_then(|l| {
                l.world
                    .objects
                    .values()
                    .find(|o| o.owner == Some(house) && o.on_map)
            })
            .map_or(start, |o| project_lifted(o.pos.x, o.pos.y, o.pos.z));
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
            credits: CreditCounter::default(),
            tab: Tab::Buildings,
            placing: None,
            radar,
            radar_texture: None,
            radar_tick: -1,
            shroud: true,
        });
        self.set_centre(start);
        self.pinned = None;
    }

    fn footprint(&self) -> Option<&glasnost_map::Footprint> {
        self.shown.as_ref().map(|s| &s.loaded.map.footprint)
    }

    /// Sets the view centre, limited by S-0101 F5. The depth reference
    /// does not change (S-0102 F1: one C for the whole game).
    fn set_centre(&mut self, requested: (i32, i32)) {
        let Some(f) = self.footprint().copied() else {
            return;
        };
        let size = (self.view_size.0 as i32, self.view_size.1 as i32);
        // spec: S-0101 claim 8 (every request is limited by F5; the stored centre is the limited one)
        self.centre = limit_centre(requested, size, f.width, f.local);
    }

    fn view(&self) -> View {
        let size = (self.view_size.0 as i32, self.view_size.1 as i32);
        View {
            origin: view_origin(self.centre, size),
            size: self.view_size,
            // spec: S-0102 F1 (C = 32768 throughout the game, whatever the view)
            reference: DEPTH_REFERENCE,
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
            alpha: self.light_alpha(),
        })
    }

    /// The light-alpha values of the view after the controlled house's
    /// shroud covers (S-0110 claims 1 to 10), or `None` when no cover is
    /// drawn: no live game, the sandbox (every owner's objects, no single
    /// player), or the covers turned off.
    fn light_alpha(&self) -> Option<Vec<u8>> {
        let s = self.shown.as_ref()?;
        let live = s.loaded.live.as_ref()?;
        if !s.shroud || live.setup.sandbox {
            return None;
        }
        Some(cover::light_alpha(
            &live.world,
            s.house,
            &self.view(),
            &s.loaded.covers,
            &ShroudQuirks::default(),
        ))
    }

    /// Local snapshots: issues the deploy order of the controlled house's
    /// first mobile unit. Returns whether it was accepted.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn deploy_base(&mut self) -> bool {
        let Some(shown) = self.shown.as_mut() else {
            return false;
        };
        let house = shown.house;
        let Some(live) = shown.loaded.live.as_mut() else {
            return false;
        };
        // Orders are blocked during ticks 0 to 3 (S-0201 claim 14).
        while live.world.tick < 4 {
            live.step();
        }
        let Some(id) = live
            .world
            .objects
            .values()
            .find(|o| o.owner == Some(house) && o.kind.is_mobile() && o.on_map)
            .map(|o| o.id)
        else {
            return false;
        };
        let rep = live.issue_orders(
            house,
            vec![glasnost_sim::orders::Order::Deploy { actor: id }],
        );
        rep.issued > 0
    }

    /// Local snapshots: a left click on the first build icon of the first
    /// tab with icons. Returns whether it gave an order.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn build_first(&mut self) -> bool {
        let Some(shown) = self.shown.as_mut() else {
            return false;
        };
        let house = shown.house;
        let Some(live) = shown.loaded.live.as_mut() else {
            return false;
        };
        let m = sidebar::model(&live.world, house);
        let Some(c) = m.tabs.first().and_then(|(_, v)| v.first()) else {
            return false;
        };
        match sidebar::click(&live.world, house, c.t, Click::Left, false) {
            Action::Order(o) => live.issue_orders(house, vec![o]).issued > 0,
            _ => false,
        }
    }

    /// Local snapshots: runs `n` ticks at once, as the pacer would one by
    /// one, and redraws.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn advance(&mut self, n: i32) {
        let Some(shown) = self.shown.as_mut() else {
            return;
        };
        for _ in 0..n.max(0) {
            let scene = Rc::make_mut(&mut shown.scene);
            let (inputs, live) = shown.loaded.split();
            let Some(live) = live else {
                return;
            };
            live.step();
            live.redraw(&inputs, scene);
            shown
                .credits
                .update(glasnost_sim::econ::production::available_credits(
                    &live.world,
                    shown.house,
                ));
            if let Some(r) = shown.radar.as_mut() {
                r.observe(&live.world, Some(shown.house));
            }
        }
        shown.paused = true;
    }

    /// Local snapshots: drops every texture handle, for a new egui context.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn forget_textures(&mut self) {
        self.cpu_texture = None;
        if let Some(s) = self.shown.as_mut() {
            s.minimap = None;
            s.radar_texture = None;
        }
    }

    /// Chooses the GPU map pass or the CPU reference.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn set_use_gpu(&mut self, on: bool) {
        self.use_gpu = on;
    }

    /// The GPU map pass failed with `error`: the pane falls back on the CPU
    /// reference, so the pass is not built again on every frame; the GPU
    /// box turns it back on.
    ///
    /// design: render (#110): the game's drawer falls back on the CPU
    /// compositor in the same way.
    pub fn gpu_failed(&mut self, error: &str) {
        self.use_gpu = false;
        self.gpu_texture = None;
        self.status = format!("GPU map pass failed, showing the CPU reference: {error}");
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
        if self.shown.as_ref().is_some_and(|s| s.loaded.live.is_some()) {
            egui::Panel::right("sidebar")
                .resizable(false)
                .default_size(250.0)
                .show(ui, |ui| self.sidebar_ui(ui));
        }
        egui::Panel::right("map info")
            .resizable(true)
            .default_size(320.0)
            .show(ui, |ui| self.info_ui(ui));
        egui::CentralPanel::default().show(ui, |ui| self.view_ui(ui));
        action
    }

    /// The live game's sidebar (M5a placeholder layout, our own): credits,
    /// power, the radar map, the tabs with their build icons, and the
    /// placement mode.
    fn sidebar_ui(&mut self, ui: &mut Ui) {
        let Some(shown) = self.shown.as_mut() else {
            return;
        };
        let house = shown.house;
        let Some(live) = shown.loaded.live.as_mut() else {
            return;
        };
        let m = sidebar::model(&live.world, house);
        ui.label(
            RichText::new(format!("$ {}", shown.credits.shown))
                .strong()
                .size(20.0)
                .color(Color32::from_rgb(240, 220, 120)),
        );
        let (output, drain) = m.power;
        let power = format!("power {output} / drain {drain}");
        if drain > output {
            ui.colored_label(Color32::from_rgb(240, 80, 60), power);
        } else {
            ui.label(power);
        }
        // The radar map (S-0410), redrawn once per tick.
        if let Some(r) = &shown.radar {
            if shown.radar_tick != live.world.tick || shown.radar_texture.is_none() {
                let img = egui::ColorImage::from_rgba_unmultiplied(
                    [radar::SIZE.0 as usize, radar::SIZE.1 as usize],
                    &r.rgba(&live.world, Some(house)),
                );
                match &mut shown.radar_texture {
                    Some(t) => t.set(img, TextureOptions::NEAREST),
                    None => {
                        shown.radar_texture =
                            Some(ui.ctx().load_texture("radar", img, TextureOptions::NEAREST))
                    }
                }
                shown.radar_tick = live.world.tick;
            }
            if let Some(t) = &shown.radar_texture {
                let size = egui::vec2(radar::SIZE.0 as f32 * 1.6, radar::SIZE.1 as f32 * 1.6);
                let resp =
                    ui.add(egui::Image::new((t.id(), size)).sense(egui::Sense::click_and_drag()));
                // spec: S-0410 claim 1 (without radar the map stays dark)
                if !glasnost_sim::econ::radar::available(&live.world, house) {
                    ui.painter().text(
                        resp.rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "no radar",
                        egui::FontId::proportional(14.0),
                        Color32::GRAY,
                    );
                }
                if let Some(p) = resp.interact_pointer_pos()
                    && (resp.clicked() || resp.dragged())
                {
                    let x = ((p.x - resp.rect.min.x) / 1.6) as i32;
                    let y = ((p.y - resp.rect.min.y) / 1.6) as i32;
                    if let Some(c) = r.cell_at(&live.world, x, y) {
                        let (lx, ly) = cell_centre(c.x, c.y);
                        self.pending_jump = Some(project_lifted(lx, ly, 0));
                    }
                }
            }
        }
        ui.checkbox(&mut shown.shroud, "shroud");
        ui.separator();
        ui.horizontal_wrapped(|ui| {
            for t in Tab::ALL {
                let n = m.tab(t).len();
                let label = format!("{} {n}", t.label());
                if ui
                    .add_enabled(n > 0, egui::Button::selectable(shown.tab == t, label))
                    .clicked()
                {
                    shown.tab = t;
                }
            }
        });
        if m.tab(shown.tab).is_empty()
            && let Some((t, _)) = m.tabs.first()
        {
            shown.tab = *t;
        }
        let shift = ui.input(|i| i.modifiers.shift);
        let mut action = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("cameos").num_columns(2).show(ui, |ui| {
                for (i, c) in m.tab(shown.tab).iter().enumerate() {
                    if let Some(a) = cameo_ui(ui, c, shift, &live.world, house) {
                        action = Some(a);
                    }
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
        });
        if let Some(t) = shown.placing {
            ui.separator();
            ui.label(format!(
                "Placing {}: left click on the map to place, right click to stop.",
                crate::mapview::live::type_name(&live.rules, t.kind, t.index)
            ));
        }
        match action {
            Some(Action::Order(o)) => {
                let rep = live.issue_orders(house, vec![o]);
                self.status = input_status(&rep);
            }
            Some(Action::Place(t)) => {
                shown.placing = Some(t);
                self.status = "Placement mode.".into();
            }
            Some(Action::Refused) => self.status = "Refused: the line is busy.".into(),
            Some(Action::Nothing) | None => {}
        }
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
        egui::ComboBox::from_label("game on load")
            .selected_text(match self.options.game {
                GameMode::Sandbox => "sandbox",
                GameMode::Skirmish => "skirmish vs script",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.options.game, GameMode::Sandbox, "sandbox");
                ui.selectable_value(
                    &mut self.options.game,
                    GameMode::Skirmish,
                    "skirmish vs script",
                );
            });
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
            self.set_centre(t);
        }
        let avail = ui.available_size();
        let ppp = ui.ctx().pixels_per_point();
        let max_side = ui.ctx().input(|i| i.max_texture_side);
        let size = view_size(avail.x * ppp, avail.y * ppp, max_side);
        if size != self.view_size {
            self.view_size = size;
            let c = self.centre;
            self.set_centre(c);
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
            let frame = scene.compose_lit(
                &self.view(),
                &self.filter(),
                &ComposeQuirks::default(),
                self.light_alpha(),
            );
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
            self.set_centre(c);
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
            // spec: S-0101 F8 (the probe reads each cell's height level): a live game's cells, whose levels change in play, else the map's (#83)
            let cells = shown
                .loaded
                .live
                .as_ref()
                .map_or(&shown.loaded.map.cells, |l| &l.world.cells.grid);
            let q = ProjectionQuirks::default();
            // spec: S-0101 claim 10 (the highest level is read only with quirks.cursor_probe_depth_180 off)
            let max_level = if q.cursor_probe_depth_180 {
                0
            } else {
                cells
                    .iter()
                    .map(|(_, c)| i32::from(c.level))
                    .max()
                    .unwrap_or(0)
            };
            let cell = cursor_cell(
                qx,
                qy,
                |x, y| {
                    cells
                        .get(glasnost_map::CellXY::new(x, y))
                        .map_or(0, |c| i32::from(c.level))
                },
                max_level,
                &q,
            );
            (
                cell.0,
                cell.1,
                // The object drawn at the point (S-0102 claims 10 and 31; #68).
                shown
                    .scene
                    .object_at(qx, qy, &view, &filter, &ComposeQuirks::default()),
            )
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
        let mut marks_changed = false;
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
            // Placement mode (S-0411 claim 4; S-0406): the foundation under
            // the cursor, green where the building would be placed, red
            // elsewhere; a left click places, a right click stops.
            if let Some(t) = shown.placing {
                let still_ready = sidebar::model(&live.world, house)
                    .tabs
                    .iter()
                    .flat_map(|(_, v)| v)
                    .any(|c| c.t == t && c.ready);
                if !still_ready {
                    shown.placing = None;
                }
            }
            let placing = shown.placing;
            if let (Some(t), Some((cx, cy, _))) = (placing, hover) {
                let cursor = glasnost_map::CellXY::new(cx, cy);
                let (cells, ok) = sidebar::preview(&live.world, house, t, cursor);
                let painter = ui.painter_at(r.rect);
                let fill = if ok {
                    Color32::from_rgba_unmultiplied(60, 220, 80, 110)
                } else {
                    Color32::from_rgba_unmultiplied(230, 50, 40, 110)
                };
                for c in cells {
                    let level = live.world.cells.cell(c).map_or(0, |x| i32::from(x.level));
                    painter.add(egui::Shape::convex_polygon(
                        cell_diamond(c, level, &view, r.rect, ppp).to_vec(),
                        fill,
                        egui::Stroke::new(1.0, Color32::WHITE),
                    ));
                }
                if r.clicked() {
                    match sidebar::place(&live.world, house, t, cursor) {
                        Some(o) => {
                            let rep = live.issue_orders(house, vec![o]);
                            if rep.issued > 0 {
                                shown.placing = None;
                            }
                            report = Some(rep);
                        }
                        None => self.status = "Cannot place there.".into(),
                    }
                }
                if r.secondary_clicked() {
                    shown.placing = None;
                    self.status = "Placement stopped.".into();
                }
            } else if r.clicked()
                && let Some(on) = target
            {
                // spec: S-0214 claim 26 (a left release selects or gives the selection's orders, as the input rules decide)
                report = Some(live.click(house, &mut shown.selection, on, mods));
            }
            if placing.is_none() && r.secondary_clicked() {
                // spec: S-0214 claim 26 (a right release that was not a scroll drag clears the selection)
                shown.selection.clear();
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
            // S-0111 claim 3: the selected objects and the one under the
            // pointer show their health bars.
            let hovered = hover
                .and_then(|h| h.2)
                .and_then(|c| live.object_of_code(c))
                .filter(|&id| live.world.obj(id).is_some_and(|o| o.on_map));
            marks_changed = live.set_marks(&shown.selection, hovered);
            // Our own markers for selected objects S-0111's bracket image
            // does not mark (buildings; units without the bracket sprite).
            let plain: Vec<ObjectId> = shown
                .selection
                .iter()
                .copied()
                .filter(|&id| {
                    !crate::mapview::live::bracket_drawn(&live.world, &shown.loaded.assets, id)
                })
                .collect();
            selection_markers(ui, live, &plain, &view, r.rect, ppp);
        } else if r.clicked() {
            self.pinned = hover.and_then(|h| h.2);
            self.explain.clear();
        }
        if marks_changed {
            let scene = Rc::make_mut(&mut shown.scene);
            let (inputs, live) = shown.loaded.split();
            if let Some(live) = live {
                live.redraw_status(&inputs, scene);
            }
            ui.ctx().request_repaint();
        }
        if let Some(rep) = report {
            self.status = input_status(&rep);
        }
    }
}

/// One build icon of the placeholder sidebar: a button with the type's name
/// and price, its clock as a progress bar, and the ready, on-hold and
/// queue-count labels (S-0411 claims 3, 7 and 8). Returns what a click did.
fn cameo_ui(
    ui: &mut Ui,
    c: &sidebar::Cameo,
    shift: bool,
    w: &glasnost_sim::world::World,
    house: HouseId,
) -> Option<Action> {
    let mut action = None;
    ui.vertical(|ui| {
        ui.set_width(110.0);
        let mut text = RichText::new(format!("{}\n${}", c.name, c.price));
        if c.darkened {
            text = text.weak();
        }
        let b = ui.add_sized([110.0, 44.0], egui::Button::new(text));
        if b.clicked() {
            action = Some(sidebar::click(w, house, c.t, Click::Left, shift));
        }
        if b.secondary_clicked() {
            action = Some(sidebar::click(w, house, c.t, Click::Right, shift));
        }
        if let Some((stage, last)) = c.progress() {
            ui.add(
                egui::ProgressBar::new(stage as f32 / last as f32)
                    .desired_width(110.0)
                    .desired_height(6.0),
            );
        }
        let mut tags = Vec::new();
        if c.ready {
            tags.push("ready".to_string());
        }
        if c.on_hold {
            tags.push("on hold".to_string());
        }
        if let Some(q) = c.queued {
            tags.push(format!("× {q}"));
        }
        if !tags.is_empty() {
            ui.label(RichText::new(tags.join(" · ")).small().strong());
        }
    });
    action
}

/// The screen corners of cell `c` at height level `level` (top, right,
/// bottom, left): its diamond of S-0101 F1, lifted by the terrain lift.
fn cell_diamond(
    c: glasnost_map::CellXY,
    level: i32,
    view: &View,
    rect: egui::Rect,
    ppp: f32,
) -> [egui::Pos2; 4] {
    use glasnost_render::projection::{TERRAIN_LIFT, cell_top};
    let (tx, ty) = cell_top(c.x, c.y);
    let ty = ty - TERRAIN_LIFT * level;
    let at = |x: i32, y: i32| {
        egui::pos2(
            rect.min.x + (x - view.origin.0) as f32 / ppp,
            rect.min.y + (y - view.origin.1) as f32 / ppp,
        )
    };
    // spec: S-0101 F1 (a cell's diamond: 60 × 30 pixels below its top corner)
    [
        at(tx, ty),
        at(tx + 30, ty + 15),
        at(tx, ty + 30),
        at(tx - 30, ty + 15),
    ]
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
        // spec: S-0416 claim 2 (one step per cycle, aiming at the available credits)
        shown
            .credits
            .update(glasnost_sim::econ::production::available_credits(
                &live.world,
                shown.house,
            ));
        if let Some(r) = shown.radar.as_mut() {
            r.observe(&live.world, Some(shown.house));
        }
    }
}

/// The theater of a loaded map.
fn map_theater(loaded: &Loaded) -> glasnost_map::Theater {
    loaded.map.theater
}

/// The game panel: tick, pacing, the controlled house, notes.
fn game_ui(ui: &mut Ui, shown: &mut Shown, status: &mut String) {
    let Some(live) = shown.loaded.live.as_ref() else {
        return;
    };
    ui.separator();
    let title = if live.setup.sandbox {
        "Sandbox game"
    } else {
        "Skirmish game"
    };
    ui.label(RichText::new(title).strong());
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
    let k = &live.counts;
    ui.label(format!(
        "combat: {} shots, {} detonations, {} animations, {} killed; {} projectiles in flight",
        k.shots,
        k.detonations,
        k.anims,
        k.kills,
        live.world.projectiles.len()
    ));
    egui::Grid::new("house combat")
        .striped(true)
        .show(ui, |ui| {
            ui.label(RichText::new("house").small().strong());
            ui.label(RichText::new("kills").small().strong());
            ui.label(RichText::new("losses").small().strong());
            ui.label(RichText::new("score").small().strong());
            ui.end_row();
            for h in &live.world.houses {
                ui.label(RichText::new(&h.name).small());
                ui.label(RichText::new(h.combat.kills.to_string()).small());
                ui.label(RichText::new(h.combat.losses.to_string()).small());
                ui.label(RichText::new(h.combat.score.to_string()).small());
                ui.end_row();
            }
        });
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
    ui.label(format!("target: {}", target(o.target)));
    let strength = match o.kind {
        glasnost_sim::rules::Kind::Building => live
            .rules
            .buildings
            .get(o.type_index as usize)
            .map(|b| b.strength),
        glasnost_sim::rules::Kind::Terrain => None,
        k => live.rules.unit(k, o.type_index).map(|u| u.strength),
    };
    ui.label(format!(
        "health {} of {}; body facing {}",
        o.health,
        strength.map_or_else(|| "?".into(), |s| s.to_string()),
        o.body.value(w.tick)
    ));
    let c = &o.combat;
    // Ammunition −1 is unlimited (S-0301 claim 10).
    let ammo = if c.ammo < 0 {
        "unlimited".to_string()
    } else {
        c.ammo.to_string()
    };
    // The experience is a single (S-0311 claim 1); shown as a host float,
    // presentation only.
    let experience = f32::from_bits(c.experience.0);
    let rank = glasnost_sim::combat::veterancy::rank_of(c.experience);
    ui.label(format!(
        "ammunition {ammo}; experience {experience:.3} ({rank:?})"
    ));
    if c.killed {
        ui.label("killed (its death is under way)");
    }
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

/// The map view's size in pixels for an available area of `w × h`
/// physical pixels: at least 64 a side, and at most `max_side`, the
/// device's texture limit, which the view's textures must fit.
///
/// design: render (#109): a larger pane shows the view at the limit
/// rather than failing to make its textures.
fn view_size(w: f32, h: f32, max_side: usize) -> (u32, u32) {
    let max = u32::try_from(max_side).unwrap_or(u32::MAX).max(64);
    let side = |v: f32| (v as u32).clamp(64, max);
    (side(w), side(h))
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

#[cfg(test)]
mod tests {
    use super::*;

    /// design: render (#110): a failed GPU map pass falls back on the CPU reference and asks
    /// for no further GPU pass, so the renderer is not built again on every frame.
    #[test]
    fn a_failed_gpu_pass_falls_back_on_the_cpu_reference() {
        let mut pane = MapPane::default();
        assert!(pane.use_gpu);
        pane.gpu_failed("the atlas is full");
        assert!(!pane.use_gpu);
        assert!(pane.gpu_request().is_none());
        assert!(pane.status.contains("the atlas is full"), "{}", pane.status);
        // The GPU box turns the pass back on.
        pane.set_use_gpu(true);
        assert!(pane.use_gpu);
    }

    /// design: render (#109): the map view never asks for textures over the device's limit,
    /// and keeps its 64-pixel floor.
    #[test]
    fn the_view_size_stays_within_the_texture_limit() {
        assert_eq!(view_size(5000.0, 300.0, 4096), (4096, 300));
        assert_eq!(view_size(10.0, 9000.0, 2048), (64, 2048));
        assert_eq!(view_size(800.0, 600.0, usize::MAX), (800, 600));
    }
}
