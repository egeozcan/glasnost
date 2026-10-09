//! The map viewer pane: map list, the battlefield view with scrolling
//! (S-0101 scroll limits), a minimap, layer toggles and object info with
//! `rules explain`.
//!
//! The battlefield is drawn by the GPU map pass (`glasnost_render::map_gpu`)
//! into a texture the shell gives this pane each frame, or on the CPU by the
//! reference compositor when chosen.

use std::rc::Rc;

use egui::{Color32, RichText, TextureHandle, TextureOptions, Ui};
use glasnost_render::compose::{ComposeQuirks, Frame};
use glasnost_render::light::DetailLevel;
use glasnost_render::projection::{
    ScrollQuirks, cell_centre, cursor_cell, limit_centre, project_lifted, view_origin,
};
use glasnost_render::scene::{Filter, Group, Scene, View};
use glasnost_vfs::FileRef;

use crate::mapview::build::{self, ObjectInfo};
use crate::mapview::find::Listing;
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
            reference: Frame::reset_reference(0),
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
            "{}: {} × {}, {} items, {} objects, {} map warnings, {} problems.",
            String::from_utf8_lossy(&map.basic.name),
            map.footprint.width,
            map.footprint.height,
            loaded.built.scene.items.len(),
            loaded.built.objects.len(),
            loaded.map_warnings.len(),
            loaded.problems.len(),
        );
        self.shown = Some(Shown {
            scene: Rc::new(loaded.built.scene.clone()),
            generation: self.generation,
            loaded,
            minimap: None,
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
        let c = limit_centre(requested, size, f.width, f.local, &ScrollQuirks::default());
        let (dx, dy) = (c.0 - self.centre.0, c.1 - self.centre.1);
        // spec: S-0102 Terms (a view jump moves by more than the view's width across or its height up or down)
        let big = dx.abs() > size.0 || dy.abs() > size.1;
        self.centre = c;
        if jump || big {
            // spec: S-0102 F1 (at a reset, C = 32768 + the world row of the view's top)
            self.reference = Frame::reset_reference(view_origin(c, size).1);
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
        );
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
        let object = self.pinned.or(self.hover.and_then(|h| h.2));
        if let Some((cx, cy, _)) = self.hover {
            ui.label(format!("cell under the cursor (S-0101 F8): ({cx}, {cy})"));
        }
        if let Some(o) = object.and_then(|o| shown.loaded.built.objects.get(o as usize)) {
            object_ui(ui, o);
            if ui.button("rules explain").clicked() {
                self.explain = explain(&shown.loaded, o);
            }
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
        let Some(scene) = self.shown.as_ref().map(|s| s.scene.clone()) else {
            ui.centered_and_justified(|ui| ui.label("No map loaded."));
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
        let Some(shown) = self.shown.as_ref() else {
            return;
        };
        self.hover = r.hover_pos().map(|p| {
            let o = self.view().origin;
            let qx = o.0 + ((p.x - r.rect.min.x) * ppp) as i32;
            let qy = o.1 + ((p.y - r.rect.min.y) * ppp) as i32;
            let map = &shown.loaded.map;
            let cell = cursor_cell(qx, qy, |x, y| {
                map.cells
                    .get(glasnost_map::CellXY::new(x, y))
                    .map_or(0, |c| i32::from(c.level))
            });
            (cell.0, cell.1, scene.object_at(qx, qy, &self.filter()))
        });
        if r.clicked() {
            self.pinned = self.hover.and_then(|h| h.2);
            self.explain.clear();
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

/// `rules explain` for an object's type: the history of every value of its
/// type entry in the converted tree.
fn explain(loaded: &Loaded, o: &ObjectInfo) -> String {
    let tree = &loaded.import.tree;
    let Some(path) = glasnost_rules::explain::find_path(tree, &o.type_name) else {
        return format!("{} is not in the converted rules", o.type_name);
    };
    glasnost_rules::explain::explain(tree, &loaded.import.provenance, &path)
}
