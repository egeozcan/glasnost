//! The viewer's rules pane: the imported type registries (S-0004), each
//! type's layered keys and looked-up art (S-0006, S-0007), and `rules
//! explain` for any value (PLAN.md 4.6).

use std::collections::BTreeMap;

use egui::{Color32, RichText, Ui};
use glasnost_fmt::pal::Palette6;
use glasnost_render::palette::{PaletteQuirks, PaletteState, SLOT_POSITIONS};
use glasnost_rules::Resolved;
use glasnost_rules::explain::{explain, find_path};
use glasnost_rules::jsonc::{Node, Value};
use glasnost_vfs::theater::Theater;

/// A colour definition shown by a lobby slot: its name and (H, S, V).
pub type SlotDefinition = (String, [u8; 3]);

/// What the pane asks the browser to do.
pub enum RulesAction {
    /// Import the rules for the pane's theater.
    Import,
    /// Open a file by name in the preview.
    Open(String),
}

/// The result of an import, as the pane keeps it.
pub struct RulesData {
    resolved: Resolved,
    warnings: Vec<String>,
    errors: Vec<String>,
    coverage: Vec<(String, usize)>,
    palettes: PaletteState,
    theater: Theater,
}

impl RulesData {
    /// The resolved ruleset.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn resolved(&self) -> &Resolved {
        &self.resolved
    }

    /// Builds the pane's data from an import.
    pub fn new(
        resolved: Resolved,
        import: &glasnost_convert::Import,
        palette_files: &BTreeMap<String, Option<Palette6>>,
        theater: Theater,
    ) -> Self {
        let q = PaletteQuirks::default();
        let file = |n: &str| {
            palette_files
                .get(&n.to_ascii_uppercase())
                .cloned()
                .flatten()
        };
        // S-0103 State and ordering: the program starts with the snow unit palette and the anim palette.
        let mut palettes = PaletteState::at_program_start(
            file(&Theater::Snow.unit_palette()).as_ref(),
            file(glasnost_render::palette::ANIM_PALETTE_FILE).as_ref(),
            q,
        );
        palettes.start_game(theater, |n| file(n));
        let coverage = glasnost_convert::coverage::summary(&import.coverage)
            .into_iter()
            .map(|(o, n)| (o.word().to_owned(), n))
            .collect();
        Self {
            resolved,
            warnings: import
                .warnings
                .iter()
                .map(|w| {
                    if w.line > 0 {
                        format!("{}:{}: {}", w.file, w.line, w.message)
                    } else {
                        format!("{}: {}", w.file, w.message)
                    }
                })
                .collect(),
            errors: import.errors.clone(),
            coverage,
            palettes,
            theater,
        }
    }
}

/// The rules pane.
pub struct RulesPane {
    /// Theater to import for.
    pub theater: Theater,
    data: Option<RulesData>,
    error: Option<String>,
    kind: String,
    filter: String,
    selected: Option<String>,
    query: String,
    explained: String,
}

impl Default for RulesPane {
    fn default() -> Self {
        Self {
            theater: Theater::Temperate,
            data: None,
            error: None,
            kind: "buildings".into(),
            filter: String::new(),
            selected: None,
            query: String::new(),
            explained: String::new(),
        }
    }
}

fn text(n: Option<&Node>) -> String {
    match n.map(|n| &n.value) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Integer(i)) => i.text().to_owned(),
        Some(Value::Bool(b)) => b.to_string(),
        _ => String::new(),
    }
}

impl RulesPane {
    /// Stores an import's result.
    pub fn set(&mut self, data: Result<RulesData, String>) {
        match data {
            Ok(d) => {
                self.data = Some(d);
                self.error = None;
            }
            Err(e) => {
                self.data = None;
                self.error = Some(e);
            }
        }
        self.selected = None;
        self.explained.clear();
    }

    /// The imported data, if any.
    pub fn data(&self) -> Option<&RulesData> {
        self.data.as_ref()
    }

    /// The palettes in force for the imported game (S-0103 claims 1 to 3).
    pub fn palettes(&self) -> Option<&PaletteState> {
        self.data.as_ref().map(|d| &d.palettes)
    }

    /// The colour definitions that lobby slots 0 to 7 show (S-0103 claim
    /// 10), with their names; `None` for a slot whose position has no
    /// definition.
    pub fn slot_definitions(&self) -> Option<Vec<Option<SlotDefinition>>> {
        let colours = &self.data.as_ref()?.resolved.ruleset.colours;
        Some(
            SLOT_POSITIONS
                .iter()
                .map(|&p| colours.get(p).map(|c| (c.name.clone(), c.hsv)))
                .collect(),
        )
    }

    /// Runs `rules explain` for a dotted query and keeps the text for the
    /// details panel.
    pub fn explain_query(&mut self, query: &str) -> &str {
        self.query = query.to_owned();
        self.explained = match &self.data {
            Some(d) => match find_path(&d.resolved.tree, query.trim()) {
                Some(p) => explain(&d.resolved.tree, &d.resolved.provenance, &p),
                None => format!("{} is not in the ruleset", query.trim()),
            },
            None => "no rules imported".to_owned(),
        };
        &self.explained
    }

    /// Selects a type of a kind, as a click in the type list does.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn select_type(&mut self, kind: &str, name: &str) {
        self.kind = kind.to_owned();
        self.selected = Some(name.to_owned());
    }

    /// The left panel: import controls, summary and the type list.
    pub fn list_ui(&mut self, ui: &mut Ui, busy: bool) -> Option<RulesAction> {
        let mut action = None;
        ui.heading("Rules");
        ui.horizontal(|ui| {
            egui::ComboBox::from_id_salt("theater")
                .selected_text(self.theater.identifier())
                .show_ui(ui, |ui| {
                    for t in Theater::ALL {
                        ui.selectable_value(&mut self.theater, t, t.identifier());
                    }
                });
            if ui.add_enabled(!busy, egui::Button::new("Import")).clicked() {
                action = Some(RulesAction::Import);
            }
        });
        if let Some(e) = &self.error {
            ui.colored_label(Color32::LIGHT_RED, e);
        }
        let Some(data) = &self.data else {
            ui.label("Import the mounted install's rules (base rules, empty game mode, no map) to list its types.");
            return action;
        };
        let r = &data.resolved;
        ui.label(
            RichText::new(format!(
                "{} theater · sim_rules_hash {}…",
                data.theater.identifier(),
                &r.sim_rules_hash[..12]
            ))
            .small(),
        );
        for e in &data.errors {
            ui.colored_label(Color32::LIGHT_RED, e);
        }
        egui::CollapsingHeader::new(format!("{} importer warnings", data.warnings.len()))
            .id_salt("warnings")
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .show(ui, |ui| {
                        for w in &data.warnings {
                            ui.label(RichText::new(w).small());
                        }
                    });
            });
        let coverage: Vec<String> = data
            .coverage
            .iter()
            .map(|(o, n)| format!("{o} {n}"))
            .collect();
        ui.label(RichText::new(format!("coverage: {}", coverage.join(", "))).small());
        ui.separator();
        let kinds: Vec<(String, usize)> = r
            .ruleset
            .registries
            .iter()
            .map(|(k, v)| (k.clone(), v.len()))
            .collect();
        egui::ComboBox::from_id_salt("kind")
            .selected_text(&self.kind)
            .show_ui(ui, |ui| {
                for (k, n) in &kinds {
                    ui.selectable_value(&mut self.kind, k.clone(), format!("{k} ({n})"));
                }
            });
        if let Some(incomplete) = r.ruleset.import.as_ref().map(|i| &i.incomplete_registries)
            && incomplete.contains(&self.kind)
        {
            ui.label(
                RichText::new("May lack types created by references no spec lists yet (Q-0005).")
                    .small()
                    .weak(),
            );
        }
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.filter);
        });
        let names = r
            .ruleset
            .registries
            .get(&self.kind)
            .cloned()
            .unwrap_or_default();
        let filter = self.filter.to_ascii_lowercase();
        let shown: Vec<(usize, &String)> = names
            .iter()
            .enumerate()
            .filter(|(_, n)| filter.is_empty() || n.to_ascii_lowercase().contains(&filter))
            .collect();
        let h = ui.text_style_height(&egui::TextStyle::Body);
        let mut chosen = None;
        egui::ScrollArea::vertical().show_rows(ui, h, shown.len(), |ui, range| {
            for &(i, n) in &shown[range] {
                let on = self.selected.as_deref() == Some(n.as_str());
                if ui.selectable_label(on, format!("{i:>4}  {n}")).clicked() {
                    chosen = Some(n.clone());
                }
            }
        });
        if let Some(n) = chosen {
            self.query = format!("types.{}.{n}", self.kind);
            self.selected = Some(n);
            self.explained.clear();
        }
        action
    }

    /// The second panel: the selected type's keys and art, and `rules
    /// explain`.
    pub fn details_ui(&mut self, ui: &mut Ui) -> Option<RulesAction> {
        let mut action = None;
        if self.data.is_none() {
            ui.label("No rules imported.");
            return None;
        }
        let mut explain_now = false;
        ui.heading("rules explain");
        ui.horizontal(|ui| {
            let field = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("HTNK.ini.Cost")
                    .desired_width(220.0),
            );
            if ui.button("Explain").clicked()
                || (field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
            {
                explain_now = true;
            }
        });
        if explain_now {
            let q = self.query.clone();
            self.explain_query(&q);
        }
        let Some(data) = &self.data else {
            return None;
        };
        let r = &data.resolved;
        if !self.explained.is_empty() {
            egui::ScrollArea::vertical()
                .id_salt("explained")
                .max_height(220.0)
                .show(ui, |ui| {
                    ui.monospace(&self.explained);
                });
        }
        ui.separator();
        let Some(name) = &self.selected else {
            ui.label("Select a type.");
            return action;
        };
        let Some(node) = r
            .tree
            .get("types")
            .and_then(|t| t.get(&self.kind))
            .and_then(|k| k.get(name))
        else {
            return action;
        };
        ui.label(RichText::new(name).strong());
        ui.label(format!(
            "index {} · created by {} · reads {}",
            text(node.get("index")),
            text(node.get("created")),
            match node.get("reads").map(|n| &n.value) {
                Some(Value::Array(a)) if !a.is_empty() => a
                    .iter()
                    .map(|x| text(Some(x)))
                    .collect::<Vec<_>>()
                    .join(", "),
                _ => "nothing".into(),
            }
        ));
        if let Some(art) = node.get("art") {
            egui::CollapsingHeader::new("Art (S-0007)")
                .id_salt("art")
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!("image {}", text(art.get("image"))));
                    let mut files: Vec<(String, String)> = Vec::new();
                    for field in ["sprite", "icon", "alt_icon", "translucent"] {
                        if let Some(f) = art.get(field) {
                            files.push((field.to_owned(), text(Some(f))));
                        }
                    }
                    for group in ["voxels", "building_sprites"] {
                        for m in art.get(group).and_then(Node::members).unwrap_or(&[]) {
                            files.push((m.key.clone(), text(Some(&m.value))));
                        }
                    }
                    for (role, file) in files {
                        ui.horizontal(|ui| {
                            ui.label(format!("{role}: {file}"));
                            if ui.small_button("open").clicked() {
                                action = Some(RulesAction::Open(file.clone()));
                            }
                        });
                    }
                    if let Some(Value::Array(notes)) = art.get("notes").map(|n| &n.value) {
                        for n in notes {
                            ui.label(RichText::new(text(Some(n))).small().weak());
                        }
                    }
                });
        }
        let mut explain_key = None;
        egui::CollapsingHeader::new("Keys (layered, as stored)")
            .id_salt("keys")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("keys-scroll")
                    .show(ui, |ui| {
                        for m in node.get("ini").and_then(Node::members).unwrap_or(&[]) {
                            ui.horizontal(|ui| {
                                if ui.small_button("?").clicked() {
                                    explain_key = Some(m.key.clone());
                                }
                                ui.label(format!("{} = {}", m.key, text(Some(&m.value))));
                            });
                        }
                        if let Some(Value::Array(reset)) = node.get("reset").map(|n| &n.value) {
                            for k in reset {
                                ui.label(
                                    RichText::new(format!(
                                        "{} = (reset, S-0006 claim 5)",
                                        text(Some(k))
                                    ))
                                    .weak(),
                                );
                            }
                        }
                    });
            });
        if let Some(k) = explain_key {
            let path = vec![
                "types".to_owned(),
                self.kind.clone(),
                name.clone(),
                "ini".to_owned(),
                k,
            ];
            self.query = path.join(".");
            self.explained = explain(&r.tree, &r.provenance, &path);
        }
        action
    }
}
