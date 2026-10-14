//! The game client's model of a running game, without drawing: the
//! [`Game`] (world and AI), the local player's selection, tool, sidebar
//! tab and scroll, the credit counter, status messages and the sounds the
//! interface asks for. Every order goes through [`Game::issue`] in the
//! local house's name, so the replay holds all of them.
//!
//! The rules of each input come from the simulation (`targeting` for
//! clicks and keys, `econ::sidebar` for the icons, `econ::placement` for
//! placement); this module only routes them. Tests drive it headless.

use glasnost_ai::game::Game;
use glasnost_audio::events::UiCue;
use glasnost_map::CellXY;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::econ::sidebar::Click;
use glasnost_sim::econ::victory::Outcome;
use glasnost_sim::econ::{TypeRef, production, radar};
use glasnost_sim::orders::Order;
use glasnost_sim::targeting::{self, CheerMemory, ClickOn, ClickResult, KeyCommand, Modifiers};
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, ObjectId, World};
use glasnost_ui::end::End;
use glasnost_ui::hud::{self, HudModel};
use glasnost_ui::modes::{self, Mode};

use crate::mapview::credits::{CreditCounter, Tick};
use crate::mapview::live::guarded;
use crate::mapview::sidebar::{self, Action};

/// How long a status message stays, in ticks. Interim (Q-0282).
pub const MESSAGE_TICKS: i32 = 90;
/// The most status messages shown at once. Interim (Q-0282).
pub const MESSAGES_SHOWN: usize = 4;

/// What a left click on the battlefield does.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    /// Select, and order with the right button (S-0214).
    #[default]
    Select,
    /// Placement mode for a done building (S-0411 claim 10).
    Place(TypeRef),
    /// Sell, repair or power mode (S-0214 claim 13).
    Mode(Mode),
}

/// A status message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// Its text.
    pub text: String,
    /// The tick after which it goes.
    pub until: i32,
}

/// Which mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    /// The left (primary) button.
    Left,
    /// The right (secondary) button.
    Right,
}

/// The local player's game.
pub struct Play {
    /// The game.
    pub game: Game,
    /// The local player's house.
    pub local: HouseId,
    /// The selection, in selection order.
    pub selection: Vec<ObjectId>,
    /// What a battlefield click does.
    pub tool: Tool,
    /// The sidebar tab asked for (S-0411 claim 2 picks the one shown).
    pub tab: usize,
    /// The icon grid's scroll position, in rows.
    pub scroll: usize,
    /// The credit counter (S-0416).
    pub credits: CreditCounter,
    /// Status messages, oldest first.
    pub messages: Vec<Message>,
    /// Why the game stopped, when a tick failed.
    pub failed: Option<String>,
    /// The cheer command's rate limit (S-0213 claim 19).
    pub cheer: CheerMemory,
    /// The last tick's presentation events, for sound.
    pub effects: Vec<Effect>,
    /// Interface sounds asked for since the client last took them.
    pub cues: Vec<UiCue>,
    /// Battlefield clicks that gave orders, with the selection they were
    /// given to in selection order, for the order response (S-0701 claim 18).
    pub acks: Vec<(Vec<ObjectId>, ClickResult)>,
    /// The first object each selecting input selected, for the selection
    /// response (S-0701 claim 17).
    pub selected: Vec<ObjectId>,
    /// Key commands that issued orders, for their sounds (S-0701 claim 19).
    pub keys: Vec<KeyCommand>,
    /// The game's result, once it is over (S-0414).
    pub outcome: Option<Outcome>,
    radar_on: bool,
    waiting: bool,
}

fn icon_ready(w: &World, house: HouseId, t: TypeRef) -> bool {
    glasnost_sim::econ::sidebar::icon_state(w, house, t).is_some_and(|s| s.ready)
}

/// The person's house of a setup: the first player house without an AI.
pub fn local_house(setup: &GameSetup) -> Option<HouseId> {
    setup
        .houses
        .iter()
        .position(|h| !h.ai && h.role == HouseRole::Player)
        .map(|i| HouseId(i as u8))
}

impl Play {
    /// Wraps a started game for the local player's `house`.
    pub fn new(game: Game, local: HouseId) -> Self {
        let mut cheer = CheerMemory::default();
        cheer.game_start(&game.world);
        let radar_on = radar::available(&game.world, local);
        Self {
            game,
            local,
            selection: Vec::new(),
            tool: Tool::Select,
            tab: 0,
            scroll: 0,
            credits: CreditCounter::default(),
            messages: Vec::new(),
            failed: None,
            cheer,
            effects: Vec::new(),
            cues: Vec::new(),
            acks: Vec::new(),
            selected: Vec::new(),
            keys: Vec::new(),
            outcome: None,
            radar_on,
            waiting: false,
        }
    }

    /// The world.
    pub fn world(&self) -> &World {
        &self.game.world
    }

    /// Whether ticks may run: no failure and the game not over.
    pub fn running(&self) -> bool {
        self.failed.is_none() && self.outcome.is_none()
    }

    /// Shows a status message.
    pub fn say(&mut self, text: impl Into<String>) {
        let until = self.game.world.tick + MESSAGE_TICKS;
        self.messages.push(Message {
            text: text.into(),
            until,
        });
        let n = self.messages.len();
        if n > MESSAGES_SHOWN {
            self.messages.drain(..n - MESSAGES_SHOWN);
        }
    }

    /// Runs one tick: the AI thinks and the simulation steps
    /// ([`Game::step`]); then the interface follows the new state.
    pub fn step(&mut self) {
        if !self.running() {
            return;
        }
        let game = &mut self.game;
        if let Err(p) = guarded(|| game.step()) {
            self.failed = Some(format!("the simulation failed in a tick: {p}"));
            return;
        }
        self.effects = std::mem::take(&mut self.game.world.effects);
        let w = &self.game.world;
        let house = self.local;
        // spec: S-0416 claim 1 (one counter step per cycle towards the available credits)
        if let Some(t) = self
            .credits
            .update(production::available_credits(w, house))
        {
            // spec: S-0416 claim 4 (a tick sound when the counter moves)
            self.cues.push(UiCue::CreditTick {
                rising: t == Tick::Rising,
            });
        }
        let on = radar::available(w, house);
        if on != self.radar_on {
            // spec: S-0410 claim 3 (the radar's on and off sounds)
            self.cues.push(if on { UiCue::RadarOn } else { UiCue::RadarOff });
            self.radar_on = on;
        }
        let model = self.hud_model(&|_| String::new(), &|_| None);
        let waiting = model.tabs.iter().any(|t| t.waiting);
        if waiting && !self.waiting {
            // spec: S-0411 claim 9 (a building item done: the construction-complete cue)
            self.cues.push(UiCue::ConstructionComplete);
        }
        self.waiting = waiting;
        if let Tool::Place(t) = self.tool
            && !icon_ready(w, house, t)
        {
            self.tool = Tool::Select;
        }
        self.selection
            .retain(|&id| targeting::selectable(&self.game.world, id));
        let tick = self.game.world.tick;
        self.messages.retain(|m| m.until > tick);
        self.outcome = self.game.outcome();
    }

    /// Issues orders in the local house's name; returns how many the
    /// simulation took.
    pub fn issue(&mut self, orders: Vec<Order>) -> usize {
        let local = self.local;
        orders
            .into_iter()
            .filter(|o| self.game.issue(local, o.clone()))
            .count()
    }

    /// What a click targets: the selectable object under the pointer, else
    /// the cell.
    pub fn target(&self, object: Option<ObjectId>, cell: CellXY) -> ClickOn {
        match object.filter(|&id| targeting::selectable(&self.game.world, id)) {
            Some(id) => ClickOn::Object(id),
            None => ClickOn::Cell(cell),
        }
    }

    /// A click on the battlefield at `cell`, with `object` under the
    /// pointer.
    pub fn click_map(
        &mut self,
        button: Button,
        object: Option<ObjectId>,
        cell: CellXY,
        mods: Modifiers,
    ) {
        let on = self.target(object, cell);
        match (self.tool, button) {
            (Tool::Place(t), Button::Left) => {
                let w = &self.game.world;
                match sidebar::place(w, self.local, t, cell) {
                    Some(o) => {
                        if self.issue(vec![o]) > 0 {
                            self.tool = Tool::Select;
                            // spec: S-0701 claim 23 (the local player's house places a building: the placement sound)
                            self.cues.push(UiCue::BuildingPlaced);
                        }
                    }
                    None => {
                        // spec: S-0411 claim 10 (a refused placement plays a sound; placement mode resumes)
                        self.cues.push(UiCue::PlacementRefused);
                        self.say("Cannot place there.");
                    }
                }
            }
            (Tool::Place(_) | Tool::Mode(_), Button::Right) => self.tool = Tool::Select,
            (Tool::Mode(m), Button::Left) => {
                match modes::order(&self.game.world, self.local, m, on) {
                    Some(o) => {
                        self.issue(vec![o]);
                    }
                    None => self.say("Nothing to do there."),
                }
            }
            (Tool::Select, Button::Left) => match on {
                ClickOn::Object(_) => {
                    // A left click selects: without force-select it acts as
                    // if nothing were selected, so it never orders (S-0214
                    // claim 12).
                    let sel = if mods.force_select {
                        std::mem::take(&mut self.selection)
                    } else {
                        Vec::new()
                    };
                    self.apply_click(sel, on, mods);
                }
                ClickOn::Cell(_) => self.selection.clear(),
            },
            (Tool::Select, Button::Right) => {
                let sel = std::mem::take(&mut self.selection);
                self.apply_click(sel, on, mods);
            }
        }
    }

    fn apply_click(&mut self, mut sel: Vec<ObjectId>, on: ClickOn, mods: Modifiers) {
        let w = &self.game.world;
        sel.retain(|&id| targeting::selectable(w, id));
        let r = targeting::click(w, self.local, &sel, on, mods);
        // spec: S-0701 claim 17 (the first object the input selects speaks; one already in the selection the click acted on does not)
        if let Some(new) = &r.selection
            && let Some(&first) = new.iter().find(|id| !sel.contains(id))
        {
            self.selected.push(first);
        }
        self.selection = r.selection.clone().unwrap_or_else(|| sel.clone());
        self.issue(r.orders.clone());
        if !r.orders.is_empty() {
            self.acks.push((sel, r));
        }
    }

    /// A box selection (interim, Q-0278): `inside` are the objects whose
    /// reference point projects inside the box; the local house's
    /// selectable mobile ones are taken, replacing the selection, or added
    /// to it with `add`.
    pub fn select_box(&mut self, inside: &[ObjectId], add: bool) {
        let w = &self.game.world;
        let local = self.local;
        let taken: Vec<ObjectId> = inside
            .iter()
            .copied()
            .filter(|&id| {
                targeting::selectable(w, id)
                    && w.obj(id)
                        .is_some_and(|o| o.owner == Some(local) && o.kind.is_mobile())
            })
            .collect();
        if !add {
            self.selection.clear();
        }
        let mut first = None;
        for id in taken {
            if !self.selection.contains(&id) {
                self.selection.push(id);
                first = first.or(Some(id));
            }
        }
        // spec: S-0701 claim 17 (several objects selected by one input: only the first speaks)
        self.selected.extend(first);
    }

    /// A key command for the selection (S-0213 table T2).
    pub fn key(&mut self, cmd: KeyCommand) {
        let w = &self.game.world;
        self.selection.retain(|&id| targeting::selectable(w, id));
        let list = targeting::key_command(w, self.local, &self.selection, cmd);
        // interim: Q-0336 (a key command sounds once per press, and only when it issued orders)
        if self.issue(list) > 0 {
            self.keys.push(cmd);
        }
    }

    /// The cheer command (S-0213 table T2, claim 19).
    pub fn cheer(&mut self) {
        if let Some(o) = self.cheer.cheer(&self.game.world) {
            self.issue(vec![o]);
        }
    }

    /// Surrender (S-0414 claim 11).
    pub fn surrender(&mut self) {
        self.issue(vec![Order::Surrender]);
    }

    /// The tab shown (S-0411 claim 2).
    pub fn shown_tab(&self) -> usize {
        hud::shown_tab(&self.game.world, self.local, self.tab)
    }

    /// The sidebar model.
    pub fn hud_model(
        &self,
        name: &dyn Fn(TypeRef) -> String,
        art: &dyn Fn(TypeRef) -> Option<(u32, u32, i32, i32)>,
    ) -> HudModel {
        let mut m = hud::model(
            &self.game.world,
            self.local,
            self.tab,
            self.credits.shown,
            name,
            art,
        );
        m.scroll = self.scroll;
        m.mode = match self.tool {
            Tool::Mode(x) => Some(x),
            _ => None,
        };
        m
    }

    /// A click on a tab button.
    pub fn tab_click(&mut self, i: usize) {
        if i != self.tab {
            self.tab = i;
            self.scroll = 0;
        }
    }

    /// Scrolls the icon grid by `d` rows, `rows` being shown.
    pub fn scroll_icons(&mut self, d: i32, rows: usize) {
        let n = self
            .game
            .world
            .houses
            .get(usize::from(self.local.0))
            .and_then(|h| h.econ.tech.icons.get(self.shown_tab()))
            .map_or(0, Vec::len);
        let max = hud::max_scroll(n, rows) as i32;
        self.scroll = (self.scroll as i32 + d).clamp(0, max) as usize;
    }

    /// A click on a mode button: the mode, or back to selecting when it is
    /// on already.
    pub fn mode_click(&mut self, m: Mode) {
        self.tool = if self.tool == Tool::Mode(m) {
            Tool::Select
        } else {
            Tool::Mode(m)
        };
    }

    /// A click on the `n`-th icon of the shown tab (S-0411 claims 4 and 5).
    pub fn cameo_click(&mut self, n: usize, button: Button, shift: bool) {
        let w = &self.game.world;
        let Some(c) = w
            .houses
            .get(usize::from(self.local.0))
            .and_then(|h| h.econ.tech.icons.get(self.shown_tab()))
            .and_then(|v| v.get(n))
        else {
            return;
        };
        let t = c.t;
        let click = match button {
            Button::Left => Click::Left,
            Button::Right => Click::Right,
        };
        let held = glasnost_sim::econ::sidebar::icon_state(w, self.local, t)
            .is_some_and(|s| s.on_hold);
        match sidebar::click(w, self.local, t, click, shift) {
            Action::Order(o) => {
                // spec: S-0411 claim 12 (start, queue, resume, hold and cancel clicks play interface sounds)
                let cue = match &o {
                    Order::Build { .. } if held => Some(UiCue::BuildResumed),
                    Order::Build { .. } => Some(UiCue::BuildStarted),
                    Order::Hold { .. } => Some(UiCue::BuildHeld),
                    Order::Cancel { .. } | Order::CancelAll { .. } => Some(UiCue::BuildCancelled),
                    _ => None,
                };
                if self.issue(vec![o]) > 0 {
                    self.cues.extend(cue);
                }
            }
            Action::Place(t) => self.tool = Tool::Place(t),
            Action::Refused => {
                // spec: S-0411 claims 4 and 12 (a refused build click plays a refusal sound)
                self.cues.push(UiCue::BuildRefused);
                self.say("Cannot build that now.");
            }
            Action::Nothing => {}
        }
    }

    /// The placement preview at `cell`: the foundation and whether the
    /// building would be placed there.
    pub fn preview(&self, cell: CellXY) -> Option<(Vec<CellXY>, bool)> {
        match self.tool {
            Tool::Place(t) => Some(sidebar::preview(&self.game.world, self.local, t, cell)),
            _ => None,
        }
    }

    /// The end screen, once the game is over (S-0414). `period_ms` is the
    /// paced period of a tick (S-0201 claim 4).
    pub fn end(&self, period_ms: i64) -> Option<End> {
        self.outcome
            .as_ref()
            .map(|o| End::from_world(&self.game.world, o, Some(self.local), period_ms))
    }
}

#[cfg(test)]
#[path = "play_tests.rs"]
mod tests;
