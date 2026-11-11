//! The asset browser: archive list, entry list, settings and preview.

use std::collections::{BTreeMap, BTreeSet};

use egui::{Color32, RichText, TextureHandle, TextureOptions, Ui};
use glasnost_fmt::aud::Aud;
use glasnost_fmt::csf::Csf;
use glasnost_fmt::hva::Hva;
use glasnost_fmt::idx::{Idx, IdxEntry};
use glasnost_fmt::image::Palette;
use glasnost_fmt::mix::{MixEntry, id_hash};
use glasnost_fmt::pal::Palette6;
use glasnost_fmt::pcx::Pcx;
use glasnost_fmt::shp::Shp;
use glasnost_fmt::sniff::{Kind, sniff};
use glasnost_fmt::tmp::Tmp;
use glasnost_fmt::vxl::Vxl;
use glasnost_render::palette::{PaletteQuirks, house_palette, palette_from_file, ramp};
use glasnost_vfs::names::candidates;
use glasnost_vfs::{ArchiveId, FileRef, Vfs};

use super::map_pane::{MapAction, MapPane};
use super::preview::{Rgba, fits, rgb, strip, voxel_preview};
use super::rules::{RulesAction, RulesData, RulesPane};
use super::tasks::{self, Purpose, Spawner, TaskResult};
use glasnost_render::palette::PaletteChoice;

/// Colour definitions (hue, saturation, value) offered for house-colour
/// previews before a rules file is imported: an invented set of our own,
/// easy to tell apart (not taken from the game). Their ramps follow S-0103.
const PREVIEW_DEFINITIONS: [(&str, [u8; 3]); 8] = [
    ("crimson", [2, 220, 230]),
    ("azure", [150, 200, 245]),
    ("jade", [95, 190, 200]),
    ("amber", [30, 230, 250]),
    ("violet", [190, 160, 220]),
    ("tangerine", [18, 235, 245]),
    ("teal", [125, 210, 190]),
    ("rose", [230, 150, 235]),
];

/// Side of the voxel preview image.
const VOXEL_PREVIEW: u32 = 256;
/// Largest text shown in full.
const MAX_TEXT_SHOWN: usize = 64 * 1024;

/// What the preview pane shows.
enum Preview {
    Nothing,
    Loading,
    Error(String),
    Shp(Vec<u8>),
    Tmp(Vec<u8>),
    Vxl(Box<Vxl>),
    Pcx(Box<Pcx>),
    Pal(Box<Palette6>),
    Csf(Vec<(String, String)>),
    Hva { frames: usize, limbs: usize },
    Text(String),
    Idx { idx: Idx, bag: Option<FileRef> },
    Aud(Box<glasnost_fmt::pcm::Pcm>),
    Other { kind: Kind, len: usize },
}

/// One row of the entry list.
struct Row {
    entry: MixEntry,
    label: String,
    kind: Option<Kind>,
    nested: Option<ArchiveId>,
    /// Which copy of the name the game uses (S-0005).
    used: Used,
}

/// Whether a row's copy is the one the game uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Used {
    /// This copy wins.
    Yes,
    /// Another archive's copy wins.
    Shadowed,
    /// No archive of the search order holds the name.
    NotSearched,
}

/// A palette of the imported game (S-0103).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GamePalette {
    Unit,
    Tile,
    Theater,
    Anim,
}

/// Viewer state.
pub struct Browser {
    vfs: Option<Vfs>,
    status: String,
    /// The file system is out in a task, or a mount is under way.
    busy: bool,
    /// The number of the current mount; results of tasks started for an
    /// earlier one are dropped (#89).
    mount: u64,
    /// Names the current mount's index harvested, waiting for the file
    /// system to come back from a task (#89).
    index: Option<BTreeSet<String>>,
    #[cfg(not(target_arch = "wasm32"))]
    folder: String,
    archive: Option<ArchiveId>,
    kinds: BTreeMap<(ArchiveId, u32), Kind>,
    rows: Vec<Row>,
    rows_dirty: bool,
    filter_kind: Option<Kind>,
    filter_text: String,
    lookup: String,
    selected: Option<FileRef>,
    preview: Preview,
    texture: Option<TextureHandle>,
    texture_dirty: bool,
    palettes: Vec<(String, FileRef)>,
    palette_choice: Option<usize>,
    palette6: Option<Palette6>,
    remap: bool,
    houses: [bool; PREVIEW_DEFINITIONS.len()],
    game_palette: Option<GamePalette>,
    /// Show one row with the first colour definition's ramp (S-0103 claim
    /// 8, row 4) instead of the slots' rows.
    first_definition_row: bool,
    rules: RulesPane,
    show_rules: bool,
    frame: usize,
    on_canvas: bool,
    zoom: u32,
    angle: i32,
    player: glasnost_audio::Player,
    sample_status: String,
    /// Map viewer mode (M2).
    pub map_mode: bool,
    /// The map viewer pane.
    pub map: MapPane,
    /// A map to open once the install is mounted.
    pub pending_map: Option<String>,
    /// The game the pending map starts (local snapshots).
    pub pending_game: crate::mapview::build::GameMode,
}

impl Browser {
    /// A browser with nothing mounted. `folder` pre-fills the native folder
    /// field.
    pub fn new(folder: Option<String>) -> Self {
        let _ = &folder;
        Self {
            vfs: None,
            status: "No install mounted.".into(),
            busy: false,
            mount: 0,
            index: None,
            #[cfg(not(target_arch = "wasm32"))]
            folder: folder.unwrap_or_default(),
            archive: None,
            kinds: BTreeMap::new(),
            rows: Vec::new(),
            rows_dirty: true,
            filter_kind: None,
            filter_text: String::new(),
            lookup: String::new(),
            selected: None,
            preview: Preview::Nothing,
            texture: None,
            texture_dirty: false,
            palettes: Vec::new(),
            palette_choice: None,
            palette6: None,
            remap: false,
            houses: [true, true, true, true, false, false, false, false],
            game_palette: None,
            first_definition_row: false,
            rules: RulesPane::default(),
            show_rules: false,
            frame: 0,
            on_canvas: false,
            zoom: 2,
            angle: 30,
            player: glasnost_audio::Player::new(),
            sample_status: String::new(),
            map_mode: false,
            map: MapPane::default(),
            pending_map: None,
            pending_game: Default::default(),
        }
    }

    /// Starts mounting `files` (native; the web picker mounts directly).
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub fn mount(&mut self, spawner: &dyn Spawner, files: Vec<glasnost_vfs::PickedFile>) {
        self.busy = true;
        self.status = format!("Mounting {} files…", files.len());
        spawner.spawn(tasks::mount(files));
    }

    /// Starts `task` for the current mount: its result is dropped if it
    /// comes back after another mount (#89).
    fn spawn(&self, spawner: &dyn Spawner, task: tasks::Task) {
        spawner.spawn(tasks::for_mount(self.mount, task));
    }

    /// Handles a finished task.
    pub fn on_task(&mut self, result: TaskResult, spawner: &dyn Spawner) {
        match result {
            TaskResult::ForMount { mount, result } => {
                // design: render (#89): a result of an earlier mount, its file system included, is
                // dropped; the current mount's file system and index stay
                if mount == self.mount {
                    self.on_task(*result, spawner);
                } else {
                    log::info!("dropped a task result of an earlier mount");
                }
            }
            TaskResult::Mounted(Ok(mounted)) => {
                let (vfs, summary) = *mounted;
                crate::status::publish_mount(Some(vfs.archives().len()));
                self.mount += 1;
                self.status = format!("{summary}. Indexing…");
                let jobs = tasks::index_jobs(&vfs);
                self.vfs = Some(vfs);
                self.busy = false;
                self.kinds.clear();
                self.index = None;
                self.archive = Some(ArchiveId(0));
                self.selected = None;
                self.preview = Preview::Nothing;
                self.rows_dirty = true;
                self.spawn(spawner, tasks::index(jobs));
                if self.pending_map.is_some()
                    && let Some(vfs) = self.vfs.take()
                {
                    self.map_mode = true;
                    self.busy = true;
                    self.spawn(spawner, tasks::find_maps(vfs));
                }
            }
            TaskResult::Imported(imported) => {
                let imported = *imported;
                self.busy = false;
                self.vfs = Some(imported.vfs);
                self.apply_index();
                self.rows_dirty = true;
                let data = imported.resolved.map(|resolved| {
                    RulesData::new(
                        resolved,
                        &imported.import,
                        &imported.palettes,
                        imported.theater,
                    )
                });
                self.status = match &data {
                    Ok(_) => format!(
                        "Rules imported for {}: {} warnings, {} errors.",
                        imported.theater.identifier(),
                        imported.import.warnings.len(),
                        imported.import.errors.len()
                    ),
                    Err(e) => format!("The imported rules did not resolve: {e}"),
                };
                if data.is_ok() {
                    self.game_palette = Some(GamePalette::Unit);
                    self.texture_dirty = true;
                }
                self.rules.set(data);
            }
            TaskResult::MapsFound(found) => {
                let (vfs, maps) = *found;
                log::info!("{} maps found", maps.len());
                self.vfs = Some(vfs);
                self.busy = false;
                self.apply_index();
                if let Some(want) = self.pending_map.take() {
                    let want = want.to_ascii_lowercase();
                    let hit = maps
                        .iter()
                        .find(|m| {
                            m.name.to_ascii_lowercase().contains(&want)
                                || m.archive.to_ascii_lowercase().contains(&want)
                        })
                        .map(|m| (m.file, m.archive.clone()));
                    log::info!("map to open: {:?}", hit.as_ref().map(|h| &h.1));
                    // The file system goes out only with a map to load.
                    if let Some((file, origin)) = hit
                        && let Some(vfs) = self.vfs.take()
                    {
                        self.busy = true;
                        let task = tasks::load_map(
                            vfs,
                            file,
                            origin,
                            crate::mapview::build::Options {
                                game: self.pending_game,
                                ..Default::default()
                            },
                            self.map.voxel_canvas(),
                        );
                        self.spawn(spawner, task);
                    }
                }
                self.map.set_listings(maps);
            }
            TaskResult::MapLoaded(loaded) => {
                let (vfs, loaded) = *loaded;
                log::info!(
                    "map loaded: {} items, {} problems",
                    loaded.built.scene.items.len(),
                    loaded.problems.len()
                );
                self.vfs = Some(vfs);
                self.busy = false;
                self.apply_index();
                self.map.set_loaded(loaded);
            }
            TaskResult::Mounted(Err(e)) => {
                crate::status::publish_mount(None);
                self.busy = false;
                self.status = e;
            }
            TaskResult::Indexed { kinds, tokens } => {
                // The busy flag stays: it is the file system's being out in a task.
                self.kinds = kinds;
                self.index = Some(tokens);
                self.apply_index();
                self.rows_dirty = true;
            }
            TaskResult::Loaded { purpose, bytes } => match (purpose, bytes) {
                (Purpose::Preview(file), Ok(bytes)) if self.selected == Some(file) => {
                    self.preview = self.decode(file, bytes);
                    self.frame = 0;
                    self.texture_dirty = true;
                }
                (Purpose::Preview(file), Err(e)) if self.selected == Some(file) => {
                    self.preview = Preview::Error(e);
                }
                (Purpose::Palette(file), Ok(bytes))
                    if self
                        .palette_choice
                        .and_then(|i| self.palettes.get(i))
                        .is_some_and(|(_, f)| *f == file) =>
                {
                    match Palette6::parse(&bytes) {
                        Ok(p) => self.palette6 = Some(p),
                        Err(e) => self.status = format!("palette: {e}"),
                    }
                    self.texture_dirty = true;
                }
                (Purpose::Sample(entry), Ok(bytes)) => {
                    let local = IdxEntry { offset: 0, ..entry };
                    self.sample_status = match local.decode(&bytes) {
                        Ok(pcm) => self.play(&pcm),
                        Err(e) => format!("cannot decode: {e}"),
                    };
                }
                (_, Err(e)) => self.status = e,
                _ => {}
            },
        }
    }

    /// Learns the names harvested by the index and lists the palettes, once
    /// the file system is here; the index may come back while a task holds
    /// it (#89).
    fn apply_index(&mut self) {
        let Some(vfs) = &mut self.vfs else { return };
        let Some(tokens) = self.index.take() else {
            return;
        };
        let before = vfs.names().len();
        learn_names(vfs, &tokens);
        self.status = format!(
            "{} archives, {} entries classified, {} names found in text entries.",
            vfs.archives().len(),
            self.kinds.len(),
            vfs.names().len() - before
        );
        self.palettes = self
            .kinds
            .iter()
            .filter(|(_, k)| **k == Kind::Pal)
            .map(|(&(a, id), _)| {
                let entry = vfs
                    .archive(a)
                    .and_then(|x| x.index.find(id))
                    .copied()
                    .unwrap_or(MixEntry {
                        id,
                        offset: 0,
                        size: 0,
                    });
                (entry_label(vfs, a, id), FileRef { archive: a, entry })
            })
            .collect();
        self.palettes.sort_by(|a, b| a.0.cmp(&b.0));
        self.rows_dirty = true;
    }

    fn play(&mut self, pcm: &glasnost_fmt::pcm::Pcm) -> String {
        match self
            .player
            .play(pcm.sample_rate, pcm.channels, &pcm.samples)
        {
            Ok(()) => format!(
                "playing {} frames at {} Hz, {} channel(s)",
                pcm.frames(),
                pcm.sample_rate,
                pcm.channels
            ),
            Err(e) => e,
        }
    }

    fn decode(&self, file: FileRef, bytes: Vec<u8>) -> Preview {
        let kind = self
            .kinds
            .get(&(file.archive, file.entry.id))
            .copied()
            .unwrap_or_else(|| sniff(&bytes, bytes.len() as u64));
        let err = |e: glasnost_fmt::Error| Preview::Error(e.to_string());
        match kind {
            Kind::Shp => match Shp::parse(&bytes) {
                Ok(_) => Preview::Shp(bytes),
                Err(e) => err(e),
            },
            Kind::Tmp => match Tmp::parse(&bytes).and_then(|t| t.render()) {
                Ok(_) => Preview::Tmp(bytes),
                Err(e) => err(e),
            },
            Kind::Vxl => Vxl::parse(&bytes).map_or_else(err, |v| Preview::Vxl(Box::new(v))),
            Kind::Pcx => Pcx::parse(&bytes).map_or_else(err, |p| Preview::Pcx(Box::new(p))),
            Kind::Pal => Palette6::parse(&bytes).map_or_else(err, |p| Preview::Pal(Box::new(p))),
            Kind::Csf => Csf::parse(&bytes).map_or_else(err, |c| {
                Preview::Csf(
                    c.labels
                        .iter()
                        .map(|l| {
                            (
                                String::from_utf8_lossy(&l.name).into_owned(),
                                l.strings.first().map(|s| s.text()).unwrap_or_default(),
                            )
                        })
                        .collect(),
                )
            }),
            Kind::Hva => Hva::parse(&bytes).map_or_else(err, |h| Preview::Hva {
                frames: h.frames(),
                limbs: h.limb_names.len(),
            }),
            Kind::Idx => Idx::parse(&bytes).map_or_else(err, |idx| {
                let bag = self.find_bag(file, &idx);
                Preview::Idx { idx, bag }
            }),
            Kind::Aud => Aud::parse(&bytes)
                .and_then(|a| a.decode())
                .map_or_else(err, |p| Preview::Aud(Box::new(p))),
            Kind::Text => {
                let text = String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_TEXT_SHOWN)]);
                if text.to_ascii_lowercase().contains("[privatekey]") {
                    Preview::Error("not shown: holds a private key section".into())
                } else {
                    Preview::Text(text.into_owned())
                }
            }
            kind => Preview::Other {
                kind,
                len: bytes.len(),
            },
        }
    }

    /// The BAG of an IDX: the entry of its archive whose size is exactly the
    /// end of the furthest sample.
    fn find_bag(&self, idx_file: FileRef, idx: &Idx) -> Option<FileRef> {
        let end = idx
            .entries
            .iter()
            .map(|e| u64::from(e.offset) + u64::from(e.size))
            .max()?;
        let archive = self.vfs.as_ref()?.archive(idx_file.archive)?;
        let mut found = archive
            .index
            .entries
            .iter()
            .filter(|e| e.id != idx_file.entry.id && u64::from(e.size) == end);
        let entry = *found.next()?;
        found.next().is_none().then_some(FileRef {
            archive: idx_file.archive,
            entry,
        })
    }

    fn select(&mut self, file: FileRef, spawner: &dyn Spawner) {
        let Some(vfs) = &self.vfs else { return };
        self.selected = Some(file);
        self.sample_status.clear();
        match vfs.locate(&file) {
            Ok(loc) => {
                if let Some(source) = vfs.source(loc.source) {
                    self.preview = Preview::Loading;
                    self.spawn(
                        spawner,
                        tasks::read(Purpose::Preview(file), source, loc, None),
                    );
                }
            }
            Err(e) => self.preview = Preview::Error(e.to_string()),
        }
    }

    fn base_palette(&self) -> Palette {
        if let (Some(g), Some(state)) = (self.game_palette, self.rules.palettes()) {
            return match g {
                GamePalette::Unit => state.unit.clone(),
                GamePalette::Tile => state.tile.clone(),
                GamePalette::Theater => state.theater.clone(),
                GamePalette::Anim => state.anim.clone(),
            };
        }
        match &self.palette6 {
            Some(p) if self.palette_choice.is_some() => {
                palette_from_file(p, &PaletteQuirks::default())
            }
            _ => glasnost_fmt::synth::ramp_palette(0),
        }
    }

    fn rows_for(&self, base: &Palette) -> Vec<Palette> {
        if !self.remap {
            return vec![base.clone()];
        }
        if self.first_definition_row
            && let Some(first) = self.rules.first_definition()
        {
            return vec![house_palette(base, first)];
        }
        let rows: Vec<Palette> = self
            .house_definitions()
            .into_iter()
            .zip(self.houses)
            .filter(|(d, on)| *on && d.is_some())
            .filter_map(|(d, _)| d.map(|(_, def)| house_palette(base, def)))
            .collect();
        if rows.is_empty() {
            vec![base.clone()]
        } else {
            rows
        }
    }

    /// The colour definitions of lobby slots 0 to 7: the imported ones at
    /// the slots' definition positions (S-0103 claim 10), or our invented
    /// preview set before an import.
    fn house_definitions(&self) -> Vec<Option<(String, [u8; 3])>> {
        self.rules.slot_definitions().unwrap_or_else(|| {
            PREVIEW_DEFINITIONS
                .iter()
                .map(|(n, d)| Some(((*n).to_owned(), *d)))
                .collect()
        })
    }

    /// The selected file's picture, its strips wrapped at `max_side`, the
    /// device's texture limit (#109).
    fn picture(&self, max_side: usize) -> Option<Result<Rgba, String>> {
        let base = self.base_palette();
        let rows = self.rows_for(&base);
        let text = |e: glasnost_fmt::Error| e.to_string();
        // Every strip below wraps at the limit.
        let strip = |img: &glasnost_fmt::image::IndexedImage, rows: &[_], zero: bool| {
            strip(img, rows, zero, max_side)
        };
        Some(match &self.preview {
            Preview::Shp(bytes) => Shp::parse(bytes)
                .and_then(|s| {
                    let i = self.frame.min(s.frames.len().saturating_sub(1));
                    if self.on_canvas {
                        s.decode_on_canvas(i)
                    } else {
                        s.decode_frame(i)
                    }
                })
                .map(|img| strip(&img, &rows, true))
                .map_err(text),
            Preview::Tmp(bytes) => Tmp::parse(bytes)
                .and_then(|t| t.render())
                .map(|(img, _)| strip(&img, &rows, true))
                .map_err(text),
            Preview::Vxl(model) => Ok(strip(
                &voxel_preview(model, self.angle, VOXEL_PREVIEW),
                &rows,
                true,
            )),
            Preview::Pcx(p) => Ok(match p.as_ref() {
                Pcx::Indexed { image, palette } => {
                    strip(image, std::slice::from_ref(palette), false)
                }
                Pcx::Rgb {
                    width,
                    height,
                    pixels,
                } => rgb(*width, *height, pixels),
            }),
            Preview::Pal(p) => {
                let swatches = glasnost_fmt::image::IndexedImage::new(
                    16 * 12,
                    16 * 12,
                    (0..16 * 12 * 16 * 12)
                        .map(|i| {
                            let (x, y) = (i % (16 * 12), i / (16 * 12));
                            ((y / 12) * 16 + x / 12) as u8
                        })
                        .collect(),
                )
                .unwrap_or_else(|_| glasnost_fmt::image::IndexedImage::empty());
                Ok(strip(
                    &swatches,
                    &[palette_from_file(p, &PaletteQuirks::default())],
                    false,
                ))
            }
            _ => return None,
        })
    }

    fn refresh_rows(&mut self) {
        self.rows.clear();
        let (Some(vfs), Some(aid)) = (&self.vfs, self.archive) else {
            return;
        };
        let Some(archive) = vfs.archive(aid) else {
            return;
        };
        let nested: BTreeMap<u32, ArchiveId> = vfs
            .archives()
            .iter()
            .enumerate()
            .filter_map(|(i, a)| match a.parent {
                Some((p, e)) if p == aid => Some((e, ArchiveId(i as u32))),
                _ => None,
            })
            .collect();
        let filter = self.filter_text.to_ascii_lowercase();
        for &entry in &archive.index.entries {
            let nested_id = nested.get(&entry.id).copied();
            let kind = if nested_id.is_some() {
                Some(Kind::Mix)
            } else {
                self.kinds.get(&(aid, entry.id)).copied()
            };
            if self.filter_kind.is_some() && kind != self.filter_kind {
                continue;
            }
            let label = vfs
                .names()
                .get(entry.id)
                .map_or_else(|| format!("{:08X}", entry.id), str::to_owned);
            if !filter.is_empty() && !label.contains(&filter) {
                continue;
            }
            // spec: S-0005 claim 7 (the first archive of the search order that holds the name wins)
            let used = match vfs.find_id(entry.id) {
                Some(f) if f.archive == aid => Used::Yes,
                Some(_) => Used::Shadowed,
                None => Used::NotSearched,
            };
            self.rows.push(Row {
                entry,
                label,
                kind,
                nested: nested_id,
                used,
            });
        }
        // Known names first, then hex IDs.
        let known = |r: &Row| vfs.names().get(r.entry.id).is_none();
        self.rows
            .sort_by(|a, b| (known(a), &a.label).cmp(&(known(b), &b.label)));
        self.rows_dirty = false;
    }

    /// Draws the whole interface.
    pub fn ui(&mut self, ui: &mut Ui, spawner: &dyn Spawner, backend: &str) {
        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.heading("Glasnost viewer");
                ui.selectable_value(&mut self.map_mode, false, "Assets");
                ui.selectable_value(&mut self.map_mode, true, "Map viewer");
                ui.separator();
                self.mount_controls(ui, spawner);
                ui.separator();
                if self.busy {
                    ui.spinner();
                }
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(backend).weak());
                });
            });
        });
        if self.map_mode {
            let action = self.map.ui(ui, self.vfs.is_some(), self.busy);
            match action {
                Some(MapAction::Find) => {
                    if let Some(vfs) = self.vfs.take() {
                        self.busy = true;
                        self.map.status = "Finding maps…".into();
                        self.spawn(spawner, tasks::find_maps(vfs));
                    }
                }
                Some(MapAction::Load(file, origin, options)) => {
                    if let Some(vfs) = self.vfs.take() {
                        self.busy = true;
                        let canvas = self.map.voxel_canvas();
                        self.spawn(
                            spawner,
                            tasks::load_map(vfs, file, origin, options, canvas),
                        );
                    }
                }
                None => {}
            }
            return;
        }
        let mut action = None;
        egui::Panel::left("archives")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.show_rules, false, "Archives");
                    ui.selectable_value(&mut self.show_rules, true, "Rules");
                });
                ui.separator();
                if self.show_rules {
                    action = self.rules.list_ui(ui, self.busy || self.vfs.is_none());
                } else {
                    self.archive_list(ui);
                }
            });
        egui::Panel::left("entries")
            .resizable(true)
            .default_size(300.0)
            .show(ui, |ui| {
                if self.show_rules {
                    action = action.take().or(self.rules.details_ui(ui));
                } else {
                    self.entry_list(ui, spawner);
                }
            });
        if let Some(a) = action {
            self.rules_action(a, spawner);
        }
        egui::Panel::right("settings")
            .resizable(true)
            .default_size(260.0)
            .show(ui, |ui| self.settings(ui, spawner));
        egui::CentralPanel::default().show(ui, |ui| self.preview_pane(ui, spawner));
    }

    fn mount_controls(&mut self, ui: &mut Ui, spawner: &dyn Spawner) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            ui.label("Install folder:");
            ui.add(egui::TextEdit::singleline(&mut self.folder).desired_width(320.0));
            if ui
                .add_enabled(!self.busy, egui::Button::new("Mount"))
                .clicked()
            {
                match tasks::native_files(std::path::Path::new(&self.folder)) {
                    Ok(files) => self.mount(spawner, files),
                    Err(e) => self.status = e,
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = spawner;
            if ui
                .add_enabled(!self.busy, egui::Button::new("Pick install folder…"))
                .clicked()
            {
                super::web::open_folder_picker();
            }
        }
    }

    fn archive_list(&mut self, ui: &mut Ui) {
        ui.heading("Archives");
        let Some(vfs) = &self.vfs else {
            ui.label("Mount an install to browse its archives.");
            return;
        };
        let mut chosen = None;
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (i, a) in vfs.archives().iter().enumerate() {
                let id = ArchiveId(i as u32);
                let indent = "   ".repeat(a.depth as usize);
                let rank = vfs
                    .search_order()
                    .iter()
                    .position(|r| r.archive == id)
                    .map_or_else(|| "–".to_owned(), |r| format!("#{}", r + 1));
                let text = format!("{indent}{rank} {} ({})", a.name, a.index.entries.len());
                if ui
                    .selectable_label(self.archive == Some(id), text)
                    .clicked()
                {
                    chosen = Some(id);
                }
            }
        });
        if let Some(id) = chosen {
            self.archive = Some(id);
            self.rows_dirty = true;
        }
    }

    fn entry_list(&mut self, ui: &mut Ui, spawner: &dyn Spawner) {
        ui.heading("Entries");
        ui.horizontal(|ui| {
            ui.label("Find file:");
            let field = ui.add(
                egui::TextEdit::singleline(&mut self.lookup)
                    .hint_text("name.ext")
                    .desired_width(150.0),
            );
            let go = ui.button("Go").clicked()
                || (field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
            if go {
                self.find_by_name(spawner);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Filter:");
            if ui
                .add(egui::TextEdit::singleline(&mut self.filter_text).desired_width(110.0))
                .changed()
            {
                self.rows_dirty = true;
            }
            let label = self.filter_kind.map_or("any kind", Kind::label);
            egui::ComboBox::from_id_salt("kind")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    let mut changed = ui
                        .selectable_value(&mut self.filter_kind, None, "any kind")
                        .changed();
                    for k in KINDS {
                        changed |= ui
                            .selectable_value(&mut self.filter_kind, Some(k), k.label())
                            .changed();
                    }
                    if changed {
                        self.rows_dirty = true;
                    }
                });
        });
        if self.rows_dirty {
            self.refresh_rows();
        }
        let mut clicked = None;
        let row_height = ui.text_style_height(&egui::TextStyle::Body);
        egui::ScrollArea::vertical().show_rows(ui, row_height, self.rows.len(), |ui, range| {
            for row in &self.rows[range] {
                let selected = self.selected.is_some_and(|s| {
                    s.archive == self.archive.unwrap_or(ArchiveId(0)) && s.entry == row.entry
                });
                let kind = row.kind.map_or("?", Kind::label);
                let used = match row.used {
                    Used::Yes => "",
                    Used::Shadowed => " · shadowed",
                    Used::NotSearched => " · not searched",
                };
                let text = format!("{}  [{kind}, {} B]{used}", row.label, row.entry.size);
                if ui.selectable_label(selected, text).clicked() {
                    clicked = Some((row.entry, row.nested));
                }
            }
        });
        if let (Some((entry, nested)), Some(aid)) = (clicked, self.archive) {
            match nested {
                Some(child) => {
                    self.archive = Some(child);
                    self.rows_dirty = true;
                }
                None => self.select(
                    FileRef {
                        archive: aid,
                        entry,
                    },
                    spawner,
                ),
            }
        }
    }

    /// Sets the preview's palette to the one S-0103 claim 8 gives an
    /// object's art (unshaded: lighting, S-0104, is not applied).
    fn use_object_palette(&mut self, p: PaletteChoice) {
        let (palette, remap, first) = match p {
            PaletteChoice::Tile => (GamePalette::Tile, false, false),
            PaletteChoice::Anim => (GamePalette::Anim, false, false),
            PaletteChoice::OwnerHouse
            | PaletteChoice::LocalPlayerHouse
            | PaletteChoice::OwnerSingleShade => (GamePalette::Unit, true, false),
            PaletteChoice::FirstDefinitionSingleShade => (GamePalette::Unit, true, true),
        };
        self.game_palette = Some(palette);
        self.remap = remap;
        self.first_definition_row = first;
        self.texture_dirty = true;
    }

    fn rules_action(&mut self, action: RulesAction, spawner: &dyn Spawner) {
        match action {
            RulesAction::Import => {
                let Some(vfs) = self.vfs.take() else {
                    return;
                };
                self.busy = true;
                self.status = format!(
                    "Importing the rules for {}…",
                    self.rules.theater.identifier()
                );
                self.spawn(spawner, tasks::import(vfs, self.rules.theater));
            }
            RulesAction::Open(name, palette) => {
                if let Some(p) = palette {
                    self.use_object_palette(p);
                }
                let Some(vfs) = &mut self.vfs else {
                    return;
                };
                vfs.learn_name(&name);
                let found = vfs.resolve(&name).map(|r| (vfs.describe(&r), r));
                match found {
                    Some((from, glasnost_vfs::Resolved::Member(file))) => {
                        self.archive = Some(file.archive);
                        self.rows_dirty = true;
                        self.select(file, spawner);
                        self.status = format!("{name}: the game uses the copy in {from}");
                    }
                    Some((from, _)) => {
                        self.status =
                            format!("{name}: the game uses {from}, which the viewer does not open");
                    }
                    None => self.status = format!("{name} does not resolve"),
                }
            }
        }
    }

    /// Learns the typed name (archives store only hashes) and opens the file
    /// or archive it names.
    fn find_by_name(&mut self, spawner: &dyn Spawner) {
        let name = self.lookup.trim().to_owned();
        let Some(vfs) = &mut self.vfs else { return };
        if name.is_empty() {
            return;
        }
        vfs.learn_name(&name);
        self.rows_dirty = true;
        let id = id_hash(&name);
        let nested = vfs
            .archives()
            .iter()
            .position(|a| a.parent.is_some_and(|(_, e)| e == id));
        if let Some(i) = nested {
            self.archive = Some(ArchiveId(i as u32));
            self.status = format!("opened archive {name}");
        } else if let Some(file) = vfs.find(&name) {
            // The copy the game uses (S-0005).
            let copies = vfs.find_all(id).len();
            self.archive = Some(file.archive);
            self.select(file, spawner);
            self.status = format!("found {name} ({copies} copies; the game uses this one)");
        } else if let Some(&file) = vfs.find_all(id).first() {
            self.archive = Some(file.archive);
            self.select(file, spawner);
            self.status = format!("found {name}, but only in archives the game never searches");
        } else {
            self.status = format!("{name} is not in any mounted archive");
        }
    }

    fn settings(&mut self, ui: &mut Ui, spawner: &dyn Spawner) {
        ui.heading("Palette");
        let game_label = |g: GamePalette| match g {
            GamePalette::Unit => "game: unit palette",
            GamePalette::Tile => "game: tile palette",
            GamePalette::Theater => "game: theater palette",
            GamePalette::Anim => "game: anim palette",
        };
        let current = match self
            .game_palette
            .filter(|_| self.rules.palettes().is_some())
        {
            Some(g) => game_label(g).to_owned(),
            None => self
                .palette_choice
                .and_then(|i| self.palettes.get(i))
                .map_or("built-in synthetic ramps", |(n, _)| n.as_str())
                .to_owned(),
        };
        let mut choice = self.palette_choice;
        let mut game = self.game_palette;
        egui::ComboBox::from_id_salt("palette")
            .selected_text(current)
            .width(220.0)
            .show_ui(ui, |ui| {
                if self.rules.palettes().is_some() {
                    for g in [
                        GamePalette::Unit,
                        GamePalette::Tile,
                        GamePalette::Theater,
                        GamePalette::Anim,
                    ] {
                        if ui
                            .selectable_label(game == Some(g), game_label(g))
                            .clicked()
                        {
                            game = Some(g);
                        }
                    }
                }
                if ui
                    .selectable_label(
                        game.is_none() && choice.is_none(),
                        "built-in synthetic ramps",
                    )
                    .clicked()
                {
                    game = None;
                    choice = None;
                }
                for (i, (name, _)) in self.palettes.iter().enumerate() {
                    if ui
                        .selectable_label(game.is_none() && choice == Some(i), name)
                        .clicked()
                    {
                        game = None;
                        choice = Some(i);
                    }
                }
            });
        if game != self.game_palette {
            self.game_palette = game;
            self.texture_dirty = true;
        }
        if choice != self.palette_choice {
            self.palette_choice = choice;
            self.texture_dirty = true;
            if let (Some(i), Some(vfs)) = (choice, &self.vfs)
                && let Some((_, file)) = self.palettes.get(i)
                && let (Ok(loc), Some(src)) = (vfs.locate(file), vfs.source(file_source(vfs, file)))
            {
                self.spawn(
                    spawner,
                    tasks::read(Purpose::Palette(*file), src, loc, None),
                );
            }
        }
        ui.label(
            RichText::new("Components ×4 in 8-bit arithmetic (S-0103 claim 1); unshaded.")
                .small()
                .weak(),
        );
        ui.separator();
        ui.heading("House colour");
        ui.label(
            RichText::new(
                "Indices 16–31 take the ramp of a colour definition (S-0103 claim 6). \
                 Lighting (S-0104) is not applied.",
            )
            .small()
            .weak(),
        );
        if ui.checkbox(&mut self.remap, "house colour rows").changed() {
            self.first_definition_row = false;
            self.texture_dirty = true;
        }
        if self.first_definition_row {
            ui.label(
                RichText::new("One row: the first colour definition, as animations with the alternative palette show it (S-0103 claim 8).")
                    .small()
                    .weak(),
            );
        }
        if self.rules.data().is_some() {
            ui.label(
                RichText::new("Slots 0–7 show the imported definitions at positions 1, 5, 10, 14, 6, 12, 8, 7 (S-0103 claim 10).")
                    .small()
                    .weak(),
            );
        }
        let defs = self.house_definitions();
        ui.horizontal_wrapped(|ui| {
            for (i, d) in defs.iter().enumerate() {
                let Some((name, def)) = d else {
                    ui.label(RichText::new(format!("slot {i}: none")).weak());
                    continue;
                };
                let c = ramp(*def)[0];
                let colour = Color32::from_rgb(c.r, c.g, c.b);
                if ui
                    .checkbox(
                        &mut self.houses[i],
                        RichText::new(format!("{i}: {name}")).color(colour),
                    )
                    .changed()
                {
                    self.texture_dirty = true;
                }
            }
        });
        ui.separator();
        ui.heading("View");
        ui.horizontal(|ui| {
            ui.label("zoom");
            if ui.add(egui::Slider::new(&mut self.zoom, 1..=8)).changed() {
                self.texture_dirty = true;
            }
        });
        if let Preview::Shp(bytes) = &self.preview {
            let frames = Shp::parse(bytes).map_or(0, |s| s.frames.len());
            if frames > 0 {
                let mut f = self.frame;
                ui.add(egui::Slider::new(&mut f, 0..=frames - 1).text("frame"));
                let mut c = self.on_canvas;
                ui.checkbox(&mut c, "place on canvas");
                if f != self.frame || c != self.on_canvas {
                    self.frame = f;
                    self.on_canvas = c;
                    self.texture_dirty = true;
                }
            }
        }
        if matches!(self.preview, Preview::Vxl(_)) {
            ui.label(
                RichText::new("Simple preview; the faithful voxel renderer waits for its spec.")
                    .small()
                    .weak(),
            );
            if ui
                .add(egui::Slider::new(&mut self.angle, 0..=359).text("turn (°)"))
                .changed()
            {
                self.texture_dirty = true;
            }
        }
    }

    fn preview_pane(&mut self, ui: &mut Ui, spawner: &dyn Spawner) {
        if self.texture_dirty {
            self.texture_dirty = false;
            let max_side = ui.ctx().input(|i| i.max_texture_side);
            // design: render (#109): a picture over the device's texture limit is not uploaded
            let picture = self
                .picture(max_side)
                .map(|r| r.and_then(|p| fits(&p, max_side).map(|()| p)));
            self.texture = match picture {
                Some(Ok(p)) if p.width > 0 && p.height > 0 => Some(ui.ctx().load_texture(
                    "preview",
                    egui::ColorImage::from_rgba_unmultiplied([p.width, p.height], &p.pixels),
                    TextureOptions::NEAREST,
                )),
                Some(Err(e)) => {
                    self.status = e;
                    None
                }
                _ => None,
            };
        }
        if let Some(file) = self.selected {
            let name = self
                .vfs
                .as_ref()
                .map(|v| entry_label(v, file.archive, file.entry.id))
                .unwrap_or_default();
            ui.label(RichText::new(name).strong());
            if let Some(vfs) = &self.vfs {
                // spec: S-0005 claim 7 (the first archive of the search order that holds the name wins)
                let winner = vfs.find_id(file.entry.id).map(|f| f.archive);
                let copies: Vec<String> = vfs
                    .find_all(file.entry.id)
                    .iter()
                    .map(|c| {
                        let path = vfs.archive(c.archive).map_or("?", |a| a.path.as_str());
                        if Some(c.archive) == winner {
                            format!("{path} (used by the game)")
                        } else {
                            path.to_owned()
                        }
                    })
                    .collect();
                if winner.is_none() {
                    ui.label(
                        RichText::new("Not in the game's search order (S-0005).")
                            .small()
                            .weak(),
                    );
                }
                if copies.len() > 1 || winner.is_some_and(|w| w != file.archive) {
                    ui.label(RichText::new(format!("copies: {}", copies.join(", "))).small());
                }
            }
        }
        match &self.preview {
            Preview::Nothing => {
                ui.label("Select an entry.");
            }
            Preview::Loading => {
                ui.spinner();
            }
            Preview::Error(e) => {
                ui.colored_label(Color32::LIGHT_RED, e);
            }
            Preview::Text(t) => {
                egui::ScrollArea::both().show(ui, |ui| {
                    ui.monospace(t.as_str());
                });
            }
            Preview::Csf(labels) => {
                ui.label(format!("{} labels", labels.len()));
                let h = ui.text_style_height(&egui::TextStyle::Body);
                egui::ScrollArea::vertical().show_rows(ui, h, labels.len(), |ui, range| {
                    for (name, text) in &labels[range] {
                        ui.label(format!("{name}: {text}"));
                    }
                });
            }
            Preview::Hva { frames, limbs } => {
                ui.label(format!("HVA animation: {frames} frames × {limbs} limbs"));
            }
            Preview::Other { kind, len } => {
                ui.label(format!("{} bytes, kind: {}", len, kind.label()));
            }
            Preview::Aud(pcm) => {
                ui.label(format!(
                    "{} frames, {} Hz, {} channel(s)",
                    pcm.frames(),
                    pcm.sample_rate,
                    pcm.channels
                ));
                if ui.button("▶ Play").clicked() {
                    let pcm = pcm.clone();
                    self.sample_status = self.play(&pcm);
                }
                ui.label(&self.sample_status);
            }
            Preview::Idx { idx, bag } => {
                let (idx, bag) = (idx.clone(), *bag);
                self.sound_bank(ui, &idx, bag, spawner);
            }
            _ => {
                if let Some(tex) = &self.texture {
                    let size = tex.size_vec2() * self.zoom as f32 / ui.ctx().pixels_per_point();
                    egui::ScrollArea::both().show(ui, |ui| {
                        ui.image((tex.id(), size));
                    });
                }
            }
        }
    }

    fn sound_bank(&mut self, ui: &mut Ui, idx: &Idx, bag: Option<FileRef>, spawner: &dyn Spawner) {
        ui.label(format!("{} samples", idx.entries.len()));
        let Some(bag) = bag else {
            ui.colored_label(Color32::LIGHT_RED, "No matching BAG found in this archive.");
            return;
        };
        if ui.button("■ Stop").clicked() {
            self.player.stop();
        }
        ui.label(&self.sample_status);
        let mut play = None;
        let h = ui.text_style_height(&egui::TextStyle::Body) + 6.0;
        egui::ScrollArea::vertical().show_rows(ui, h, idx.entries.len(), |ui, range| {
            for e in &idx.entries[range] {
                ui.horizontal(|ui| {
                    if ui.small_button("▶").clicked() {
                        play = Some(e.clone());
                    }
                    ui.label(format!(
                        "{}  {} Hz, {} ch, {} B",
                        e.name,
                        e.sample_rate,
                        e.channels(),
                        e.size
                    ));
                });
            }
        });
        if let (Some(entry), Some(vfs)) = (play, &self.vfs)
            && let (Ok(loc), Some(src)) = (vfs.locate(&bag), vfs.source(file_source(vfs, &bag)))
        {
            {
                let sample = glasnost_vfs::Location {
                    source: loc.source,
                    offset: loc.offset + u64::from(entry.offset),
                    len: u64::from(entry.size).min(loc.len.saturating_sub(u64::from(entry.offset))),
                };
                self.sample_status = "loading…".into();
                self.spawn(
                    spawner,
                    tasks::read(Purpose::Sample(entry), src, sample, None),
                );
            }
        }
    }
}

/// Kinds offered in the entry filter.
const KINDS: [Kind; 12] = [
    Kind::Shp,
    Kind::Tmp,
    Kind::Vxl,
    Kind::Hva,
    Kind::Pal,
    Kind::Pcx,
    Kind::Idx,
    Kind::Aud,
    Kind::Csf,
    Kind::Text,
    Kind::Mix,
    Kind::Unknown,
];

fn file_source(vfs: &Vfs, file: &FileRef) -> glasnost_vfs::SourceId {
    vfs.archive(file.archive)
        .map_or(glasnost_vfs::SourceId(u32::MAX), |a| a.location.source)
}

fn entry_label(vfs: &Vfs, archive: ArchiveId, id: u32) -> String {
    let name = vfs
        .names()
        .get(id)
        .map_or_else(|| format!("{id:08X}"), str::to_owned);
    match vfs.archive(archive) {
        Some(a) => format!("{}/{name}", a.path),
        None => name,
    }
}

fn learn_names(vfs: &mut Vfs, tokens: &BTreeSet<String>) {
    for token in tokens {
        for name in candidates(token) {
            if vfs.contains_id(id_hash(&name)) {
                vfs.learn_name(&name);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use glasnost_fmt::fixture::{generate, registry};
    use glasnost_fmt::idx::{IdxEncoding, IdxSound, IdxWriter};
    use glasnost_fmt::mix::{MixWriter, WriteLayout};
    use glasnost_fmt::pcm::Pcm;
    use glasnost_vfs::{MemSource, PickedFile};

    use super::*;
    use crate::viewer::tasks::Task;

    /// Runs tasks to completion and queues their results.
    #[derive(Default)]
    struct Queue(RefCell<Vec<TaskResult>>);

    impl Spawner for Queue {
        fn spawn(&self, task: Task) {
            self.0.borrow_mut().push(pollster::block_on(task));
        }
    }

    fn fixture(path: &str) -> Vec<u8> {
        generate(&registry())
            .unwrap()
            .into_iter()
            .find(|f| f.path == path)
            .unwrap()
            .bytes
    }

    /// A synthetic install: one plain archive holding sprites, tiles,
    /// voxels, palettes, sounds, a text entry that names the sprite, and a
    /// nested archive with an image.
    fn install() -> Vec<PickedFile> {
        let pcm = Pcm::synthetic_tone(11025, 1, 400, 16, 4000, 3);
        let (idx, bag) = IdxWriter {
            sounds: vec![IdxSound {
                name: "beep".into(),
                pcm,
                encoding: IdxEncoding::Ima,
            }],
        }
        .write();
        let inner = MixWriter {
            layout: WriteLayout::Plain,
            files: vec![(id_hash("picture.pcx"), fixture("pcx/checker-indexed.pcx"))],
        }
        .write();
        let files = vec![
            (id_hash("discs.shp"), fixture("shp/discs-seed1.shp")),
            (id_hash("grid.tmp"), fixture("tmp/grid2x2-24x12.tmp")),
            (id_hash("model.vxl"), fixture("vxl/box-and-orb.vxl")),
            (id_hash("ramps.pal"), fixture("pal/ramps-tint7.pal")),
            (id_hash("tone.aud"), fixture("aud/tone-ima.aud")),
            (id_hash("bank.idx"), idx),
            (id_hash("bank.bag"), bag),
            (
                id_hash("notes.ini"),
                b"[Demo]\nImage=discs\nArt=model\n".to_vec(),
            ),
            (id_hash("inner.mix"), inner),
        ];
        let top = MixWriter {
            layout: WriteLayout::Plain,
            files,
        }
        .write();
        vec![
            PickedFile {
                path: "demo.mix".into(),
                source: Rc::new(MemSource(top)),
            },
            PickedFile {
                path: "readme.txt".into(),
                source: Rc::new(glasnost_vfs::source::Unopened(3)),
            },
        ]
    }

    fn settle(browser: &mut Browser, queue: &Queue) {
        loop {
            let results: Vec<_> = queue.0.borrow_mut().drain(..).collect();
            if results.is_empty() {
                return;
            }
            for r in results {
                browser.on_task(r, queue);
            }
        }
    }

    fn frame(browser: &mut Browser, queue: &Queue, ctx: &egui::Context) {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            browser.ui(ui, queue, "test")
        })
        .drop_without_applying_deltas();
    }

    fn select(browser: &mut Browser, queue: &Queue, name: &str) {
        let archive = browser.archive.unwrap();
        let entry = *browser
            .vfs
            .as_ref()
            .unwrap()
            .archive(archive)
            .unwrap()
            .index
            .find(id_hash(name))
            .unwrap();
        browser.select(FileRef { archive, entry }, queue);
        settle(browser, queue);
    }

    /// An invented install whose searched archive holds rules, art, a
    /// palette and a sprite (S-0005 Table 1 names only; contents are ours).
    fn rules_install() -> Vec<PickedFile> {
        let rules = b"[InfantryTypes]\n1=Walker\n[Walker]\nCost=7\nImage=STRIDER\n\
                      [Colors]\nC0=0,0,0\nC1=10,200,200\nC2=90,200,200\n";
        let files = vec![
            (id_hash("rulesmd.ini"), rules.to_vec()),
            (
                id_hash("artmd.ini"),
                b"[STRIDER]\nCameo=STRIDERICON\n".to_vec(),
            ),
            (id_hash("strider.shp"), fixture("shp/discs-seed1.shp")),
            (id_hash("unittem.pal"), fixture("pal/ramps-tint7.pal")),
        ];
        let main = MixWriter {
            layout: WriteLayout::Plain,
            files,
        }
        .write();
        vec![PickedFile {
            path: "ra2md.mix".into(),
            source: Rc::new(MemSource(main)),
        }]
    }

    #[test]
    fn imports_rules_and_explains_them() {
        let queue = Queue::default();
        let ctx = egui::Context::default();
        let mut b = Browser::new(None);
        b.mount(&queue, rules_install());
        settle(&mut b, &queue);
        b.rules_action(RulesAction::Import, &queue);
        settle(&mut b, &queue);
        assert!(b.vfs.is_some(), "the file system comes back");
        let data = b.rules.data().expect("rules imported");
        let _ = data;
        assert!(b.rules.palettes().is_some());
        // Lobby slot 0 shows definition position 1 (S-0103 claim 10).
        let slots = b.rules.slot_definitions().unwrap();
        assert_eq!(slots[0].as_ref().map(|(n, _)| n.as_str()), Some("C1"));
        assert!(slots[1].is_none());
        let text = b.rules.explain_query("Walker.ini.Cost").to_owned();
        assert!(
            text.contains("rulesmd.ini (ra2md.mix) line 4 | set \"7\""),
            "{text}"
        );
        // The type's sprite opens from the archive the game uses.
        b.rules.select_type("infantry", "Walker");
        b.show_rules = true;
        frame(&mut b, &queue, &ctx);
        b.rules_action(
            RulesAction::Open("STRIDER.shp".into(), Some(PaletteChoice::OwnerHouse)),
            &queue,
        );
        settle(&mut b, &queue);
        assert!(matches!(b.preview, Preview::Shp(_)), "{}", b.status);
        // Infantry art: the unit palette with house rows (S-0103 claim 8).
        assert_eq!(b.game_palette, Some(GamePalette::Unit));
        assert!(b.remap);
        frame(&mut b, &queue, &ctx);
        assert!(b.texture.is_some());
        b.use_object_palette(PaletteChoice::FirstDefinitionSingleShade);
        let base = b.base_palette();
        assert_eq!(b.rows_for(&base).len(), 1);
        b.use_object_palette(PaletteChoice::Tile);
        assert_eq!(b.game_palette, Some(GamePalette::Tile));
        assert!(!b.remap);
    }

    /// #89: with a map to open, the index arrives while the map search
    /// holds the file system; its names and palettes still apply.
    #[test]
    fn a_map_to_open_keeps_the_index() {
        let queue = Queue::default();
        let mut b = Browser::new(None);
        b.pending_map = Some("no such map".into());
        b.mount(&queue, install());
        settle(&mut b, &queue);
        assert!(!b.busy, "{}", b.status);
        let vfs = b.vfs.as_ref().unwrap();
        assert_eq!(vfs.names().get(id_hash("discs.shp")), Some("discs.shp"));
        assert_eq!(b.palettes.len(), 1);
    }

    /// #89: a task of an earlier mount that comes back after a new mount
    /// leaves the new mount's file system alone.
    #[test]
    fn a_result_of_an_earlier_mount_is_dropped() {
        let queue = Queue::default();
        let mut b = Browser::new(None);
        b.mount(&queue, rules_install());
        settle(&mut b, &queue);
        b.rules_action(RulesAction::Import, &queue);
        let stale = queue.0.borrow_mut().pop().expect("the import's result");
        b.mount(&queue, install());
        settle(&mut b, &queue);
        b.on_task(stale, &queue);
        settle(&mut b, &queue);
        assert!(!b.busy, "{}", b.status);
        let vfs = b.vfs.as_ref().expect("the new mount's file system");
        assert_eq!(vfs.archives().len(), 2, "top-level plus nested");
        assert!(b.rules.data().is_none(), "the earlier mount's rules");
    }

    #[test]
    fn mounts_indexes_and_previews_a_synthetic_install() {
        let queue = Queue::default();
        let ctx = egui::Context::default();
        let mut b = Browser::new(None);
        b.mount(&queue, install());
        settle(&mut b, &queue);
        assert!(!b.busy, "{}", b.status);
        let vfs = b.vfs.as_ref().unwrap();
        assert_eq!(vfs.archives().len(), 2, "top-level plus nested");
        assert_eq!(vfs.loose_files().len(), 1);
        // Names come from the text entry plus the format extensions.
        assert_eq!(vfs.names().get(id_hash("discs.shp")), Some("discs.shp"));
        assert_eq!(vfs.names().get(id_hash("model.vxl")), Some("model.vxl"));
        assert_eq!(b.palettes.len(), 1);
        frame(&mut b, &queue, &ctx);
        assert!(b.rows.iter().any(|r| r.label == "discs.shp"));
        assert!(b.rows.iter().any(|r| r.nested.is_some()));

        select(&mut b, &queue, "discs.shp");
        assert!(matches!(b.preview, Preview::Shp(_)));
        frame(&mut b, &queue, &ctx);
        let single = b.texture.as_ref().unwrap().size();

        // Two house rows side by side once house colour is on.
        b.remap = true;
        b.houses = [true, true, false, false, false, false, false, false];
        b.texture_dirty = true;
        frame(&mut b, &queue, &ctx);
        let rows = b.texture.as_ref().unwrap().size();
        assert_eq!(rows[0], 2 * single[0] + 6);
        assert_eq!(rows[1], single[1]);

        for (name, check) in [
            (
                "grid.tmp",
                (|p: &Preview| matches!(p, Preview::Tmp(_))) as fn(&Preview) -> bool,
            ),
            ("model.vxl", |p| matches!(p, Preview::Vxl(_))),
            ("ramps.pal", |p| matches!(p, Preview::Pal(_))),
            ("tone.aud", |p| matches!(p, Preview::Aud(_))),
            ("notes.ini", |p| matches!(p, Preview::Text(_))),
        ] {
            select(&mut b, &queue, name);
            assert!(check(&b.preview), "{name}");
            frame(&mut b, &queue, &ctx);
        }
        select(&mut b, &queue, "bank.idx");
        match &b.preview {
            Preview::Idx { idx, bag } => {
                assert_eq!(idx.entries.len(), 1);
                assert!(bag.is_some(), "BAG paired by size");
            }
            _ => panic!("IDX preview expected"),
        }

        // Palette choice loads the archive's palette.
        b.palette_choice = Some(0);
        let file = b.palettes[0].1;
        let vfs = b.vfs.as_ref().unwrap();
        let loc = vfs.locate(&file).unwrap();
        queue.spawn(tasks::read(
            Purpose::Palette(file),
            vfs.source(loc.source).unwrap(),
            loc,
            None,
        ));
        settle(&mut b, &queue);
        assert!(b.palette6.is_some());

        // Typing a name opens the archive or file it names.
        b.lookup = "inner.mix".into();
        b.find_by_name(&queue);
        settle(&mut b, &queue);
        assert_eq!(b.vfs.as_ref().unwrap().archives()[1].name, "inner.mix");
        b.lookup = "discs.shp".into();
        b.find_by_name(&queue);
        settle(&mut b, &queue);
        assert!(matches!(b.preview, Preview::Shp(_)));

        // The nested archive opens and its image decodes.
        b.archive = b.rows.iter().find_map(|r| r.nested);
        b.rows_dirty = true;
        frame(&mut b, &queue, &ctx);
        let archive = b.archive.unwrap();
        let entry = b.rows[0].entry;
        b.select(FileRef { archive, entry }, &queue);
        settle(&mut b, &queue);
        assert!(matches!(b.preview, Preview::Pcx(_)));
        frame(&mut b, &queue, &ctx);
        assert!(b.texture.is_some());
    }

    /// Renders one egui frame offscreen and returns RGBA rows.
    fn render_offscreen(
        ctx: &egui::Context,
        size: [u32; 2],
        mut ui: impl FnMut(&mut Ui),
    ) -> Option<Vec<u8>> {
        use glasnost_render::wgpu;
        // The process's one shared device: these tests run on several
        // threads, and opening devices concurrently crashed (M7b step 2c).
        let glasnost_render::gpu::SharedDevice { device, queue, .. } =
            glasnost_render::gpu::shared_device().ok()?;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let mut renderer = egui_wgpu::Renderer::new(
            &device,
            format,
            egui_wgpu::RendererOptions {
                msaa_samples: 1,
                depth_stencil_format: None,
                dithering: false,
                predictable_texture_filtering: true,
            },
        );
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size[0] as f32, size[1] as f32),
            )),
            ..Default::default()
        };
        // Two passes: the first lays out, the second draws settled widths.
        // Textures from both must reach the renderer (egui sends the font
        // atlas once).
        let mut first = ctx.run_ui(raw.clone(), &mut ui);
        let out = ctx.run_ui(raw, &mut ui);
        let screen = egui_wgpu::ScreenDescriptor {
            size_in_pixels: size,
            pixels_per_point: 1.0,
        };
        for (id, deltas) in first
            .textures_delta
            .set
            .iter()
            .chain(&out.textures_delta.set)
        {
            for d in deltas {
                renderer.update_texture(&device, &queue, *id, d);
            }
        }
        first.textures_delta.clear();
        let jobs = ctx.tessellate(out.shapes, 1.0);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let row = (size[0] * 4).div_ceil(256) * 256;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: u64::from(row * size[1]),
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut enc = device.create_command_encoder(&Default::default());
        let extra = renderer.update_buffers(&device, &queue, &mut enc, &jobs, &screen);
        {
            let pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            renderer.render(&mut pass.forget_lifetime(), &jobs, &screen);
        }
        enc.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(size[1]),
                },
            },
            wgpu::Extent3d {
                width: size[0],
                height: size[1],
                depth_or_array_layers: 1,
            },
        );
        queue.submit(extra.into_iter().chain([enc.finish()]));
        buffer.slice(..).map_async(wgpu::MapMode::Read, |_| {});
        device.poll(wgpu::PollType::wait_indefinitely()).ok()?;
        let data = buffer.slice(..).get_mapped_range().ok()?;
        let mut rgba = Vec::with_capacity((size[0] * size[1] * 4) as usize);
        for y in 0..size[1] {
            let start = (y * row) as usize;
            rgba.extend_from_slice(&data[start..start + (size[0] * 4) as usize]);
        }
        Some(rgba)
    }

    /// A minimal PNG (stored deflate blocks), for local snapshots only.
    fn png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
        fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            let mut c = kind.to_vec();
            c.extend_from_slice(data);
            out.extend_from_slice(&c);
            out.extend_from_slice(&glasnost_fmt::mix::crc32(&c).to_be_bytes());
        }
        let mut raw = Vec::new();
        for y in 0..height as usize {
            raw.push(0);
            raw.extend_from_slice(&rgba[y * width as usize * 4..(y + 1) * width as usize * 4]);
        }
        let mut z = vec![0x78, 0x01];
        let blocks: Vec<&[u8]> = raw.chunks(65535).collect();
        for (i, b) in blocks.iter().enumerate() {
            z.push(u8::from(i + 1 == blocks.len()));
            z.extend_from_slice(&(b.len() as u16).to_le_bytes());
            z.extend_from_slice(&(!(b.len() as u16)).to_le_bytes());
            z.extend_from_slice(b);
        }
        let (mut a, mut b) = (1u32, 0u32);
        for &x in &raw {
            a = (a + u32::from(x)) % 65521;
            b = (b + a) % 65521;
        }
        z.extend_from_slice(&((b << 16) | a).to_be_bytes());
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&width.to_be_bytes());
        ihdr.extend_from_slice(&height.to_be_bytes());
        ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
        chunk(&mut out, b"IHDR", &ihdr);
        chunk(&mut out, b"IDAT", &z);
        chunk(&mut out, b"IEND", &[]);
        out
    }

    /// Local only: mounts the install named by `GLASNOST_GAME_DIR`, opens the
    /// map viewer on the map named by `GLASNOST_SNAPSHOT_MAP` (default
    /// "dustbowl", matched against names and archives) and writes a
    /// screenshot of the viewer, drawn with the CPU reference, to
    /// `local-lane/` (git-ignored). Never commit it.
    #[test]
    #[ignore = "local only: needs GLASNOST_GAME_DIR and a GPU"]
    fn snapshot_map_viewer() {
        let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
            return;
        };
        let queue = Queue::default();
        let mut b = Browser::new(None);
        b.pending_map =
            Some(std::env::var("GLASNOST_SNAPSHOT_MAP").unwrap_or_else(|_| "dustbowl".into()));
        b.mount(
            &queue,
            tasks::native_files(std::path::Path::new(&dir)).unwrap(),
        );
        settle(&mut b, &queue);
        b.map.set_use_gpu(false);
        let size = [1600, 900];
        let ctx = egui::Context::default();
        super::super::configure_fonts(&ctx);
        let Some(rgba) = render_offscreen(&ctx, size, |ui| b.ui(ui, &queue, "offscreen")) else {
            eprintln!("no GPU adapter; skipped");
            return;
        };
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane");
        std::fs::create_dir_all(&out).unwrap();
        let path = out.join("map-viewer-ui.png");
        std::fs::write(&path, png(size[0], size[1], &rgba)).unwrap();
        eprintln!("wrote {}", path.display());
    }

    /// Local only: mounts the install named by `GLASNOST_GAME_DIR`, starts
    /// a skirmish against the script on the map named by
    /// `GLASNOST_SNAPSHOT_MAP` (default "dustbowl"), deploys the controlled
    /// house's base unit, starts the first build the sidebar offers, runs
    /// `GLASNOST_SNAPSHOT_TICKS` ticks (default 600) and writes screenshots
    /// of the viewer (CPU reference) at the start and the end to
    /// `local-lane/skirmish-view/` (git-ignored). Never commit them.
    #[test]
    #[ignore = "local only: needs GLASNOST_GAME_DIR and a GPU"]
    fn snapshot_skirmish_view() {
        let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
            return;
        };
        let queue = Queue::default();
        let mut b = Browser::new(None);
        b.pending_map =
            Some(std::env::var("GLASNOST_SNAPSHOT_MAP").unwrap_or_else(|_| "dustbowl".into()));
        b.pending_game = crate::mapview::build::GameMode::Skirmish;
        b.mount(
            &queue,
            tasks::native_files(std::path::Path::new(&dir)).unwrap(),
        );
        settle(&mut b, &queue);
        b.map.set_use_gpu(false);
        let ticks: i32 = std::env::var("GLASNOST_SNAPSHOT_TICKS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);
        let size = [1600, 900];
        let out =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/skirmish-view");
        std::fs::create_dir_all(&out).unwrap();
        let shoot = |b: &mut Browser, name: &str| {
            // A fresh context per shot: each offscreen render has its own
            // renderer, so no texture may carry over.
            let ctx = egui::Context::default();
            super::super::configure_fonts(&ctx);
            b.map.forget_textures();
            b.texture = None;
            let Some(rgba) = render_offscreen(&ctx, size, |ui| b.ui(ui, &queue, "offscreen"))
            else {
                eprintln!("no GPU adapter; skipped");
                return false;
            };
            let path = out.join(name);
            std::fs::write(&path, png(size[0], size[1], &rgba)).unwrap();
            eprintln!("wrote {}", path.display());
            true
        };
        if !shoot(&mut b, "start.png") {
            return;
        }
        let deployed = b.map.deploy_base();
        b.map.advance(150);
        let building = b.map.build_first();
        b.map.advance(ticks - 150);
        eprintln!(
            "deployed: {deployed}; build started: {building}; {}",
            b.map.status
        );
        shoot(&mut b, "after.png");
    }

    /// Local only: mounts the install named by `GLASNOST_GAME_DIR`, opens a
    /// few assets and writes screenshots of the viewer to the temp dir (never
    /// to the repository). `GLASNOST_SNAPSHOT_REMAP=1` turns on the house
    /// colour rows (S-0103).
    #[test]
    #[ignore = "local only: needs GLASNOST_GAME_DIR and a GPU"]
    fn snapshot_real_install() {
        let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
            return;
        };
        let queue = Queue::default();
        let mut b = Browser::new(None);
        b.mount(
            &queue,
            tasks::native_files(std::path::Path::new(&dir)).unwrap(),
        );
        settle(&mut b, &queue);
        b.remap = std::env::var_os("GLASNOST_SNAPSHOT_REMAP").is_some();
        let shots = [
            (Kind::Shp, ""),
            (Kind::Vxl, ""),
            (Kind::Tmp, ""),
            (Kind::Idx, ""),
        ];
        for (n, (kind, archive_name)) in shots.into_iter().enumerate() {
            let vfs = b.vfs.as_ref().unwrap();
            let Some((&(aid, id), _)) = b.kinds.iter().find(|((a, _), k)| {
                **k == kind
                    && (archive_name.is_empty()
                        || vfs.archive(*a).is_some_and(|x| x.name == archive_name))
            }) else {
                continue;
            };
            let entry = *vfs.archive(aid).unwrap().index.find(id).unwrap();
            b.archive = Some(aid);
            b.rows_dirty = true;
            b.select(
                FileRef {
                    archive: aid,
                    entry,
                },
                &queue,
            );
            settle(&mut b, &queue);
            b.frame = 0;
            b.texture_dirty = true;
            let size = [1600, 900];
            // A fresh context per shot: each shot has its own renderer.
            let ctx = egui::Context::default();
            super::super::configure_fonts(&ctx);
            let Some(rgba) = render_offscreen(&ctx, size, |ui| b.ui(ui, &queue, "offscreen"))
            else {
                eprintln!("no GPU adapter; skipped");
                return;
            };
            let path =
                std::env::temp_dir().join(format!("glasnost-viewer-{n}-{}.png", kind.label()));
            std::fs::write(&path, png(size[0], size[1], &rgba)).unwrap();
            eprintln!("wrote {}", path.display());
        }
        // The rules view: import, pick the first vehicle with a voxel body
        // or a sprite, explain one of its keys and open its art.
        b.rules_action(RulesAction::Import, &queue);
        settle(&mut b, &queue);
        b.show_rules = true;
        b.remap = true;
        let Some(data) = b.rules.data() else {
            eprintln!("rules did not import: {}", b.status);
            return;
        };
        let _ = data;
        let tree = &b.rules.data().unwrap().resolved().tree;
        let pick = ["vehicles", "infantry", "buildings"]
            .iter()
            .find_map(|kind| {
                let types = tree.get("types")?.get(kind)?.members()?;
                types.iter().find_map(|t| {
                    let art = t.value.get("art")?;
                    let file = art
                        .get("sprite")
                        .or_else(|| art.get("voxels").and_then(|v| v.get("body")))?
                        .as_str()?
                        .to_owned();
                    let key = t.value.get("ini")?.members()?.first()?.key.clone();
                    Some(((*kind).to_owned(), t.key.clone(), key, file))
                })
            });
        if let Some((kind, name, key, file)) = pick {
            b.rules.select_type(&kind, &name);
            b.rules
                .explain_query(&format!("types.{kind}.{name}.ini.{key}"));
            b.rules_action(
                RulesAction::Open(file, Some(PaletteChoice::OwnerHouse)),
                &queue,
            );
            settle(&mut b, &queue);
        }
        b.texture_dirty = true;
        let ctx = egui::Context::default();
        super::super::configure_fonts(&ctx);
        if let Some(rgba) = render_offscreen(&ctx, [1600, 900], |ui| b.ui(ui, &queue, "offscreen"))
        {
            let path = std::env::temp_dir().join("glasnost-viewer-rules.png");
            std::fs::write(&path, png(1600, 900, &rgba)).unwrap();
            eprintln!("wrote {}", path.display());
        }
    }
}
