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

/// How long a status line stays, in units of the presentation clock.
// spec: S-0213 claim 25 (each line expires 480 units of the 16 ms clock after it was added)
pub const MESSAGE_CLOCKS: i64 = 480;
/// The most status lines the shared display holds.
// spec: S-0213 claim 25 (at most six text lines, shared by every event's status messages)
pub const MESSAGES_SHOWN: usize = 6;
/// Milliseconds per unit of the presentation clock.
// spec: S-0213 claim 25 (the real-time presentation clock's unit is 16 milliseconds, independent of game speed)
pub const CLOCK_MS: i64 = 16;

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
    /// Its deadline: the presentation clock count when added plus
    /// [`MESSAGE_CLOCKS`]; it goes at the first display update whose count
    /// is greater.
    pub until: i64,
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
    /// Status lines, oldest first.
    pub messages: Vec<Message>,
    /// The presentation clock: host milliseconds over [`CLOCK_MS`], set
    /// at each display update (S-0213 claim 25).
    pub status_clock: i64,
    /// Whether a display update has set [`Self::status_clock`] yet.
    pub clock_started: bool,
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
    /// The tick that started each tab's flash (S-0411 claim 11).
    flash: [Option<i32>; hud::TABS],
    /// Active play time in 16-ms clock units, *E* of S-0414's result
    /// duration, and the host time of the active interval's last frame.
    pub clock_units: i64,
    clock_last: Option<i64>,
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
            status_clock: 0,
            clock_started: false,
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
            flash: [None; hud::TABS],
            clock_units: 0,
            clock_last: None,
        }
    }

    /// Advances the result duration's active clock to host time `now_ms`
    /// (S-0414 claim 23): `active` is false while the game is paused for the
    /// options menu or has lost focus. The clock stops when play stops.
    pub fn clock(&mut self, now_ms: i64, active: bool) {
        if !active || !self.running() {
            self.clock_last = None;
            return;
        }
        if let Some(u) = self.clock_last {
            // spec: S-0414 Formulas, result duration (an interval's contribution ⌊v/16⌋ − ⌊u/16⌋ clock units)
            self.clock_units += now_ms.div_euclid(16) - u.div_euclid(16);
        }
        self.clock_last = Some(now_ms);
    }

    /// The world.
    pub fn world(&self) -> &World {
        &self.game.world
    }

    /// Whether ticks may run: no failure and the game not over.
    pub fn running(&self) -> bool {
        self.failed.is_none() && self.outcome.is_none()
    }

    /// Shows a status message as a new line of the shared display (S-0213
    /// claim 25): it appends, never refreshing an older copy, and a line
    /// added at capacity removes the oldest.
    pub fn say(&mut self, text: impl Into<String>) {
        // spec: S-0213 claim 25 (deadline = clock count when added + 480, exact integers)
        let until = self.status_clock + MESSAGE_CLOCKS;
        self.messages.push(Message {
            text: text.into(),
            until,
        });
        let n = self.messages.len();
        // spec: S-0213 claim 25 (a new line at capacity removes the oldest line)
        if n > MESSAGES_SHOWN {
            self.messages.drain(..n - MESSAGES_SHOWN);
        }
    }

    /// One status-display update at host time `now_ms` (S-0213 claim 25):
    /// the clock advances, and lines whose deadline the count has passed
    /// go. The clock runs on through pauses and loss of focus.
    pub fn update_messages(&mut self, now_ms: i64) {
        // spec: S-0213 claim 25 (the 16 ms clock count)
        self.status_clock = now_ms.div_euclid(CLOCK_MS);
        let clock = self.status_clock;
        if !self.clock_started {
            // Lines said before the first update (the game's start) are
            // added at that update's clock count: the host clock is not
            // read before it, so this only fixes the count's time base.
            // spec: S-0213 claim 25 (deadline = clock count when added + 480)
            self.clock_started = true;
            for m in &mut self.messages {
                m.until += clock;
            }
        }
        // spec: S-0213 claim 25 (removed when the count is strictly greater than the deadline, not at equality)
        self.messages.retain(|m| clock <= m.until);
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
        if let Some(t) = self.credits.update(production::available_credits(w, house)) {
            // spec: S-0416 claim 4 (a tick sound when the counter moves)
            self.cues.push(UiCue::CreditTick {
                rising: t == Tick::Rising,
            });
        }
        let on = radar::available(w, house);
        if on != self.radar_on {
            // spec: S-0410 claim 3 (the radar's on and off sounds)
            self.cues
                .push(if on { UiCue::RadarOn } else { UiCue::RadarOff });
            self.radar_on = on;
        }
        let model = self.hud_model(&|_| String::new(), &|_| None);
        let waiting = model.tabs.iter().any(|t| t.waiting);
        if waiting && !self.waiting {
            // spec: S-0411 claim 9 (a building item done: the construction-complete cue)
            self.cues.push(UiCue::ConstructionComplete);
        }
        self.waiting = waiting;
        let tick = w.tick;
        for (f, tv) in self.flash.iter_mut().zip(model.tabs.iter()) {
            *f = match (*f, tv.waiting) {
                // spec: S-0411 claim 11 (with no ready subject on the tab the flash stops and clears)
                (_, false) => None,
                // spec: S-0411 claim 11 (a ready item found again does not restart an existing flash)
                (Some(s), true) => Some(s),
                // spec: S-0411 claim 11 (the tick whose sidebar update starts a new flash)
                (None, true) => Some(tick),
            };
        }
        if let Tool::Place(t) = self.tool
            && !icon_ready(w, house, t)
        {
            self.tool = Tool::Select;
        }
        self.selection
            .retain(|&id| targeting::selectable(&self.game.world, id));
        self.outcome = self.game.outcome();
        if let Some(o) = &self.outcome {
            let listed: Vec<HouseId> = o.results.iter().map(|r| r.0).collect();
            // spec: S-0414 claims 19 and 23, S-0202 claim 16 (the displayed scores, once, after the simulation has stopped)
            glasnost_sim::econ::stats::finish(&mut self.game.world, &listed);
        }
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
                        // spec: S-0411 claim 10 (placement mode resumes); S-0701 claim 24 (a refused placement: the cannot-deploy-here announcement only); S-0406 claim 20 (no battlefield text)
                        self.cues.push(UiCue::PlacementRefused);
                    }
                }
            }
            (Tool::Place(_) | Tool::Mode(_), Button::Right) => self.tool = Tool::Select,
            (Tool::Mode(m), Button::Left) => {
                match modes::order(&self.game.world, self.local, m, on) {
                    Some(o) => {
                        self.issue(vec![o]);
                    }
                    // spec: S-0408 claim 18 (a no-sell or no-repair click adds no battlefield text)
                    None if m != Mode::Power => {}
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

    /// An ordinary band box's release (S-0214 claims 19 and 20).
    /// `candidates` are the live box candidates whose box point is inside
    /// the box, of any owner and kind. `force_select` is read as held, not
    /// as a counted modifier. Returns whether the release continues as the
    /// ordinary release click at the final pixel.
    pub fn select_box(&mut self, candidates: &[ObjectId], force_select: bool) -> bool {
        let w = &self.game.world;
        let local = self.local;
        if candidates.is_empty() {
            // spec: S-0214 claim 19 (with force-select an empty box performs no fallback)
            if force_select {
                return false;
            }
            // spec: S-0214 Edge cases (quirks.empty_box_release_click: on, an unmodified empty box is a click at its release pixel; off, it clears the selection and gives no order)
            if w.rules().quirks.empty_box_release_click {
                return true;
            }
            self.selection.clear();
            return false;
        }
        // spec: S-0214 claim 19 (without force-select a box with a live candidate clears the old selection, even when nothing in it is eligible; with force-select eligible objects are added, not toggled)
        if !force_select {
            self.selection.clear();
        }
        let mut first = None;
        // Candidates are added in their admitted order: buildings, then
        // mobile objects in drawing order (interim, Q-0452).
        for &id in candidates {
            if targeting::box_eligible(w, local, &self.selection, id) {
                // spec: S-0214 claims 12 and 20 (additions use the selection order; no count limit on the selection)
                targeting::add_to_selection(w, &mut self.selection, id);
                first = first.or(Some(id));
            }
        }
        // spec: S-0701 claim 17 and S-0214 claim 20 (only the first successful addition speaks)
        self.selected.extend(first);
        false
    }

    /// A key command for the selection (S-0213 table T2).
    pub fn key(&mut self, cmd: KeyCommand) {
        let w = &self.game.world;
        self.selection.retain(|&id| targeting::selectable(w, id));
        let list = targeting::key_command(w, self.local, &self.selection, cmd);
        // spec: S-0701 claim 19 (an empty selection requests no command sound)
        if self.selection.is_empty() {
            return;
        }
        let local = self.local;
        for o in list {
            // spec: S-0701 claim 19 (a failed issuance exits the command without its sound, even after earlier orders)
            if !self.game.issue(local, o) {
                return;
            }
        }
        // spec: S-0701 claim 19 (one sound per press after a nonempty selection, however many objects were ordered, even none)
        self.keys.push(cmd);
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
        for (tv, f) in m.tabs.iter_mut().zip(self.flash) {
            tv.flash_start = f;
        }
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
        let n = self.shown_icon_count();
        // spec: S-0411 claim 18 (all icons fit: the buttons are disabled; no request and no sound)
        if d == 0 || !hud::overflows(n, rows) {
            return;
        }
        if hud::at_end(d, self.scroll, rows, n) {
            // spec: S-0411 claim 18 and S-0701 claim 24 (a request already at its requested end: the game-wide refusal sound)
            self.cues.push(UiCue::StripEnd);
            return;
        }
        // spec: S-0411 claim 18 (one page, the complete visible rows, in its direction; movement stops at the end with no further sound)
        let max = hud::max_scroll(n, rows) as i64;
        let page = rows as i64 * i64::from(d.signum());
        self.scroll = (self.scroll as i64 + page).clamp(0, max) as usize;
        // spec: S-0411 claim 18 and S-0701 claim 24 (an accepted request, even for less than a full page: the tab-scroll sound)
        self.cues.push(UiCue::StripScroll);
    }

    /// A mouse-wheel event over the game (S-0411 claim 18): one row on the
    /// shown tab, upward for a positive `delta`, with the same ends and no
    /// sound.
    pub fn wheel_icons(&mut self, delta: i32, rows: usize) {
        let n = self.shown_icon_count();
        let max = hud::max_scroll(n, rows) as i64;
        // spec: S-0411 claim 18 (one row per wheel event whatever the delta's magnitude; upward for a positive delta; no tab-scroll or refusal sound)
        let step = -i64::from(delta.signum());
        self.scroll = (self.scroll as i64 + step).clamp(0, max) as usize;
    }

    fn shown_icon_count(&self) -> usize {
        self.game
            .world
            .houses
            .get(usize::from(self.local.0))
            .and_then(|h| h.econ.tech.icons.get(self.shown_tab()))
            .map_or(0, Vec::len)
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
        let held =
            glasnost_sim::econ::sidebar::icon_state(w, self.local, t).is_some_and(|s| s.on_hold);
        // spec: S-0411 claim 12 (training for infantry, building for every other kind)
        let infantry = t.kind == glasnost_sim::rules::Kind::Infantry;
        // spec: S-0411 claim 12 (an already active line: the click adds a queued copy)
        let active = glasnost_sim::econ::production::line_of(w, self.local, t)
            .is_some_and(|l| l.item.is_some());
        match sidebar::click(w, self.local, t, click, shift) {
            Action::Order(o) => {
                // spec: S-0411 claim 12 (start, queue, resume, hold and cancel clicks play interface sounds)
                let cue = match &o {
                    Order::Build { .. } if held => Some(UiCue::BuildResumed { infantry }),
                    Order::Build { .. } if active => Some(UiCue::BuildQueued),
                    Order::Build { .. } => Some(UiCue::BuildStarted { infantry }),
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
                // spec: S-0411 claims 4 and 12; S-0701 claim 24 (a busy production line: the unable-to-comply announcement only)
                // spec: S-0411 claim 17 (no battlefield text for the refusal)
                self.cues.push(UiCue::BuildRefused);
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

    /// The end screen, once the game is over (S-0414), with the active
    /// clock's duration (claim 23).
    pub fn end(&self) -> Option<End> {
        self.outcome
            .as_ref()
            .map(|o| End::from_world(&self.game.world, o, Some(self.local), self.clock_units))
    }
}

#[cfg(test)]
#[path = "play_tests.rs"]
mod tests;
