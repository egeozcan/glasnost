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
use glasnost_sim::superweapons::{SwEvent, SwKind, charge};
use glasnost_sim::targeting::{self, CheerMemory, ClickOn, ClickResult, KeyCommand, Modifiers};
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, ObjectId, World};
use glasnost_ui::end::End;
use glasnost_ui::hud::{self, HudModel};
use glasnost_ui::modes::{self, Mode};
use glasnost_ui::status;

use crate::game::status_font::StatusFont;
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
    /// Select and order with the left button; the right button deselects
    /// (S-0214 claim 26).
    #[default]
    Select,
    /// Placement mode for a done building (S-0411 claim 10).
    Place(TypeRef),
    /// Sell, repair or power mode (S-0214 claim 13).
    Mode(Mode),
    /// Superweapon targeting (S-0512 claim 16, S-0513 claim 1): the armed
    /// type by its position in the type list; `from_source` when a source
    /// pick armed it (S-0513 claim 33) rather than its icon.
    Superweapon {
        /// The armed type.
        sw: u32,
        /// Armed by a source pick.
        from_source: bool,
    },
}

/// A status message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Message {
    /// Its text: the line's 16-bit text units (S-0213 claim 25).
    pub units: Vec<u16>,
    /// Its deadline: the presentation clock count when added plus
    /// [`MESSAGE_CLOCKS`]; it goes at the first display update whose count
    /// is greater.
    pub until: i64,
}

impl Message {
    /// The line as a string, for logs and tests (an unpaired surrogate
    /// shows as U+FFFD; drawing uses [`Self::units`]).
    pub fn text(&self) -> String {
        String::from_utf16_lossy(&self.units)
    }
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
    /// The status lines' width budget in pixels (S-0213 claim 25); set
    /// by [`Self::status_view`], unlimited until then.
    pub status_width: i32,
    /// Status texts left partly out because they could not fit (S-0213
    /// Edge cases), for the client's log.
    pub status_problems: Vec<String>,
    /// The game font that measures and draws status text (S-0213 claim
    /// 25); without it status text is omitted (Import mapping).
    pub status_font: Option<StatusFont>,
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
    /// Move-target markers to show on this screen only (S-0214 claim 30),
    /// taken by the session. Drawing only.
    pub markers: Vec<glasnost_sim::world::Pos>,
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
    /// In a network game, the person's orders not yet handed to the
    /// lockstep session; `None` in a skirmish, where orders go straight
    /// into the game.
    pub outbox: Option<Vec<Order>>,
    /// The pathing report's ring of snapshots, marks and clicks (M8 A4,
    /// our own diagnostics; nothing of it reaches the simulation).
    pub capture: crate::game::capture::Ring,
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
        let capture = crate::game::capture::Ring::new(&game);
        Self {
            capture,
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
            status_width: i32::MAX,
            status_problems: Vec::new(),
            status_font: None,
            failed: None,
            cheer,
            effects: Vec::new(),
            cues: Vec::new(),
            acks: Vec::new(),
            markers: Vec::new(),
            selected: Vec::new(),
            keys: Vec::new(),
            outcome: None,
            radar_on,
            waiting: false,
            flash: [None; hud::TABS],
            clock_units: 0,
            clock_last: None,
            outbox: None,
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

    /// Sets the status lines' width budget from the battlefield view's
    /// width `view_w` (S-0213 claim 25): at game start the width minus 22;
    /// after an in-game change of the view's dimensions (`resized`) the
    /// width minus 16, and the existing lines are cleared, not split again.
    pub fn status_view(&mut self, view_w: i32, resized: bool) {
        if resized {
            // spec: S-0213 claim 25 (a video-mode change clears the existing status lines rather than re-splitting them)
            self.messages.clear();
            // spec: S-0213 claim 25 (after an in-game change of the view's dimensions, the new width minus 16)
            self.status_width = view_w - status::RESIZED_MARGIN;
        } else {
            // spec: S-0213 claim 25 (the view's width minus 22 pixels after game start; the sidebar is not included)
            self.status_width = view_w - status::START_MARGIN;
        }
    }

    /// Shows a status message on the shared display (S-0213 claim 25):
    /// it is split into lines within the width budget with the game font's
    /// glyph widths, each appended with its own lifetime and counting
    /// against the capacity, never refreshing an older copy; a line added
    /// at capacity removes the oldest. Without a loaded game font the text
    /// is omitted (the session reported the missing font when it started).
    pub fn say(&mut self, text: impl Into<String>) {
        let text = text.into();
        // spec: S-0213 Import mapping (no status text without a loaded font: our engine omits the text)
        let Some(font) = self.status_font.as_ref() else {
            return;
        };
        let units: Vec<u16> = text.encode_utf16().collect();
        // spec: S-0213 claim 25 (Line splitting and font: the imported game font's glyph widths)
        let split = status::split(&units, self.status_width, |c| font.width(c));
        if split.unfittable {
            // spec: S-0213 Edge cases (Unfittable status text: the remainder is omitted with a warning for the offending text or font)
            self.status_problems.push(format!(
                "status text {text:?} has a remainder that cannot fit {} pixels in the game font; it is left out",
                self.status_width
            ));
        }
        for units in split.lines {
            // spec: S-0213 claim 25 (each line's deadline = clock count when added + 480, exact integers)
            let until = self.status_clock + MESSAGE_CLOCKS;
            self.messages.push(Message { units, until });
            let n = self.messages.len();
            // spec: S-0213 claim 25 (a new line at capacity removes the oldest line; wrapped lines count against the same capacity)
            if n > MESSAGES_SHOWN {
                self.messages.drain(..n - MESSAGES_SHOWN);
            }
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
        self.step_by(|g| {
            g.step();
            true
        });
    }

    /// Runs `tick` on the game, which steps it at most once and says
    /// whether it did (in a network game the lockstep session applies the
    /// tick's orders and steps); when it did, the interface follows the new
    /// state. Returns whether a tick ran.
    pub fn step_by(&mut self, tick: impl FnOnce(&mut Game) -> bool) -> bool {
        if !self.running() {
            return false;
        }
        let game = &mut self.game;
        match guarded(|| tick(game)) {
            Err(p) => {
                self.failed = Some(format!("the simulation failed in a tick: {p}"));
                return false;
            }
            Ok(false) => return false,
            Ok(true) => {}
        }
        self.effects = std::mem::take(&mut self.game.world.effects);
        // The pathing report's ring (M8 A4) after the tick, before any
        // order of the next.
        self.capture.after_tick(&self.game);
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
            *f = match (*f, tv.flashes()) {
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
        // spec: S-0512 claim 16 (the icon offers targeting only while granted, not suspended and ready, or for a drain-mode type ready or active)
        if let Tool::Superweapon {
            sw,
            from_source: false,
        } = self.tool
            && !charge::offers_targeting(w, house, sw)
        {
            self.tool = Tool::Select;
        }
        if let Some(sw) = self.source_aim() {
            self.tool = Tool::Superweapon {
                sw,
                from_source: true,
            };
        }
        let local = Some(self.local);
        for e in &self.effects {
            if let Effect::Cloak { id, event } = *e
                && glasnost_sim::cloak::view::deselects(&self.game.world, id, event, local)
            {
                // spec: S-0516 claim 26 (a cloak start or re-evaluation takes the object out of a non-owner's selection)
                self.selection.retain(|&x| x != id);
            }
            if let Effect::Bunkered { vehicle, .. } = *e {
                // spec: S-0606 claim 7 (the bunkered vehicle is removed from the selection; the client has no control groups yet)
                self.selection.retain(|&x| x != vehicle);
            }
            if let Effect::Seized { unit } = *e {
                // spec: S-0607 claim 5 (the seized unit is deselected)
                self.selection.retain(|&x| x != unit);
            }
            if let Effect::Deactivated { unit } = *e {
                // spec: S-0405 claim 20 (a vehicle disabled by the loss of its control centre is removed from the selection)
                self.selection.retain(|&x| x != unit);
            }
            if let Effect::Dropped { unit } = *e {
                // spec: S-0607 claim 13 (the stop on dropping deselects the dropped unit)
                self.selection.retain(|&x| x != unit);
            }
        }
        self.captured_yard_cancels_placement();
        self.selection
            .retain(|&id| targeting::selectable_by(&self.game.world, id, local));
        self.outcome = self.game.outcome();
        if let Some(o) = &self.outcome {
            let listed: Vec<HouseId> = o.results.iter().map(|r| r.0).collect();
            // spec: S-0414 claims 19 and 23, S-0202 claim 16 (the displayed scores, once, after the simulation has stopped)
            glasnost_sim::econ::stats::finish(&mut self.game.world, &listed);
        }
        true
    }

    /// When the new owner of a changed building is the local player's house
    /// and the building is a construction yard, the local player's building
    /// placement in progress is cancelled (S-0509 claim 15).
    fn captured_yard_cancels_placement(&mut self) {
        // spec: S-0509 claim 15 (the new owner is the local player's house and the building a construction yard: the placement in progress is cancelled)
        let gained_yard = self.effects.iter().any(|e| {
            matches!(e, Effect::Captured { of, new, .. }
                if *new == Some(self.local)
                    && of.kind == glasnost_sim::rules::Kind::Building
                    && self.game.world.rules().buildings.get(of.index as usize)
                        .is_some_and(|b| b.raw.construction_yard.unwrap_or(false)))
        });
        if gained_yard && matches!(self.tool, Tool::Place(_)) {
            self.tool = Tool::Select;
        }
    }

    /// The type a source pick of the local player in the last tick arms
    /// (S-0513 claim 33): the type at the warp kind's position in the type
    /// list under `quirks.superweapon_kind_as_list_position`, else the
    /// first type of the warp kind; `None` when no pick happened or no
    /// such type exists.
    fn source_aim(&self) -> Option<u32> {
        let picked = self.effects.iter().any(|e| {
            matches!(e, Effect::Superweapon { event: SwEvent::SourceMarker, house, .. }
                if *house == Some(self.local))
        });
        if !picked {
            return None;
        }
        let sw = &self.game.world.rules().superweapons;
        let i = if self
            .game
            .world
            .rules()
            .quirks
            .superweapon_kind_as_list_position
        {
            // spec: S-0513 claim 33 (the launching player aims the type at registration index 4, the warp kind's number)
            SwKind::ChronoWarp.number()
        } else {
            // spec: S-0513 Edge cases (quirks.superweapon_kind_as_list_position off: the type of the warp kind)
            sw.first_of_kind(SwKind::ChronoWarp)?
        };
        ((i as usize) < sw.types.len()).then_some(i)
    }

    /// Issues orders in the local house's name; returns how many the
    /// simulation took.
    pub fn issue(&mut self, orders: Vec<Order>) -> usize {
        orders.into_iter().filter(|o| self.give(o.clone())).count()
    }

    /// Gives one order in the local house's name: straight to the game, or
    /// in a network game into the outbox, which the client hands to the
    /// lockstep session; the order then runs on every peer at a later tick,
    /// and is taken as given here (the simulation judges it when it runs).
    fn give(&mut self, o: Order) -> bool {
        match self.outbox.as_mut() {
            Some(out) => {
                out.push(o);
                true
            }
            None => self.game.issue(self.local, o),
        }
    }

    /// What a click targets: the selectable object under the pointer, else
    /// the cell.
    pub fn target(&self, object: Option<ObjectId>, cell: CellXY) -> ClickOn {
        // spec: S-0516 claims 26 and 36 (a cloaked or scenery-disguised object the player cannot pick is not clicked)
        match object.filter(|&id| targeting::selectable_by(&self.game.world, id, Some(self.local)))
        {
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
        self.capture.click(crate::game::capture::Click {
            tick: self.game.world.tick,
            right: button == Button::Right,
            cell,
            object: object.map(|o| o.0),
        });
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
            // spec: S-0214 claim 28 (a right click cancels placement, repair, sell and power-switch mode with no order and no ordinary deselection; the placement item stays ready)
            (Tool::Place(_) | Tool::Mode(_) | Tool::Superweapon { .. }, Button::Right) => {
                self.tool = Tool::Select;
            }
            // spec: S-0513 claim 1 (a non-empty unit selection takes precedence: the click acts for the units)
            (Tool::Superweapon { .. }, Button::Left) if !self.selection.is_empty() => {
                let sel = std::mem::take(&mut self.selection);
                self.apply_click(sel, on, mods);
            }
            (Tool::Superweapon { sw, .. }, Button::Left) => self.aim(sw, on, cell),
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
            // spec: S-0214 claim 26 (a left-button release resolves the action at the release pixel: it selects or toggles selection, or gives the orders of claims 2 to 11 and table M)
            (Tool::Select, Button::Left) => {
                let sel = std::mem::take(&mut self.selection);
                self.apply_click(sel, on, mods);
            }
            // spec: S-0214 claim 26 (a right-button release without a scroll drag clears the selection in ordinary mode, over an object or the ground; it never gives an order)
            (Tool::Select, Button::Right) => self.selection.clear(),
        }
    }

    /// Whether superweapon `sw` is offered at a click on `on` (S-0513
    /// claim 1): a force shield only over a building of the local house or
    /// of a house allied with it, every other kind anywhere.
    pub fn sw_offered(&self, sw: u32, on: ClickOn) -> bool {
        let w = &self.game.world;
        let shield = w
            .rules()
            .superweapons
            .types
            .get(sw as usize)
            .is_some_and(|t| t.kind == Some(SwKind::ForceShield));
        if !shield {
            // spec: S-0513 claim 1 (every other kind is offered at every cell the cursor can point at)
            return true;
        }
        // spec: S-0513 claim 1 (a force shield is offered only over a building owned by the player's own house or a house allied with it)
        let ClickOn::Object(id) = on else {
            return false;
        };
        w.obj(id).is_some_and(|o| {
            o.kind == glasnost_sim::rules::Kind::Building
                && o.owner
                    .is_some_and(|h| h == self.local || w.allied(Some(self.local), Some(h)))
        })
    }

    /// Whether the armed superweapon has an explicit radar offer (S-0410
    /// claim 12, table C1): its row is chosen by its list position under
    /// `quirks.superweapon_kind_as_list_position`, else by its kind.
    pub fn radar_aims(&self) -> Option<u32> {
        let Tool::Superweapon { sw, .. } = self.tool else {
            return None;
        };
        let w = &self.game.world;
        let row = if w.rules().quirks.superweapon_kind_as_list_position {
            // spec: S-0410 claim 12 (the selected type's zero-based list position chooses the row; C1's positions are the kind numbers)
            SwKind::from_number(sw)
        } else {
            // spec: S-0410 claim 12 (switch off: its actual kind chooses the row)
            w.rules()
                .superweapons
                .types
                .get(sw as usize)
                .and_then(|t| t.kind)
        };
        // spec: S-0410 claim 12 (nuclear strike, weather storm, either paradrop kind, psychic domination, spy plane and psychic reveal have explicit offers independent of selection)
        matches!(
            row,
            Some(
                SwKind::Missile
                    | SwKind::Storm
                    | SwKind::Paradrop
                    | SwKind::AmerParadrop
                    | SwKind::Dominator
                    | SwKind::SpyPlane
                    | SwKind::PsychicReveal
            )
        )
        .then_some(sw)
    }

    /// A left click on the radar at `cell`: fires the armed superweapon
    /// when it has an explicit radar offer (S-0410 claim 12); returns
    /// whether it did, else the radar moves the view.
    pub fn click_radar(&mut self, cell: CellXY) -> bool {
        let Some(sw) = self.radar_aims() else {
            return false;
        };
        // spec: S-0513 claim 2 (the order names the selected type; the simulation applies quirks.superweapon_order_first_type_of_action)
        if self.issue(vec![Order::FireSuperweapon { sw, cell }]) > 0 {
            self.tool = Tool::Select;
        }
        true
    }

    /// An aimed click of superweapon `sw` (S-0513 claims 1 and 2): the
    /// fire order at the clicked cell when offered there, and back to
    /// selecting; otherwise no order, and the aim stays.
    fn aim(&mut self, sw: u32, on: ClickOn, cell: CellXY) {
        if !self.sw_offered(sw, on) {
            // spec: S-0513 claim 1 (the no-force-shield action: a click sends no order and keeps the charge)
            return;
        }
        // spec: S-0513 claim 2 (the order names the selected type; the simulation applies quirks.superweapon_order_first_type_of_action)
        if self.issue(vec![Order::FireSuperweapon { sw, cell }]) > 0 {
            self.tool = Tool::Select;
        }
    }

    fn apply_click(&mut self, mut sel: Vec<ObjectId>, on: ClickOn, mods: Modifiers) {
        let w = &self.game.world;
        sel.retain(|&id| targeting::selectable_by(w, id, Some(self.local)));
        let r = targeting::click(w, self.local, &sel, on, mods);
        // spec: S-0701 claim 17 (the first object the input selects speaks; one already in the selection the click acted on does not)
        if let Some(new) = &r.selection
            && let Some(&first) = new.iter().find(|id| !sel.contains(id))
        {
            self.selected.push(first);
        }
        self.selection = r.selection.clone().unwrap_or_else(|| sel.clone());
        // spec: S-0701 claim 28 (rally point set: the local player's matching action, a building's guard-spot order of S-0213 claim 20)
        let w = &self.game.world;
        let rally = r.orders.iter().any(|o| {
            matches!(o, Order::GuardSpot { actor, .. }
                if w.obj(*actor).is_some_and(|b| b.kind == glasnost_sim::rules::Kind::Building))
        });
        if self.issue(r.orders.clone()) > 0 && rally {
            self.cues.push(UiCue::RallySet);
        }
        // spec: S-0214 claim 30 (the marker is click feedback on the clicking person's screen, at the input cell, not the destination claim 11 may choose)
        if r.marker
            && let ClickOn::Cell(c) = on
        {
            self.markers
                .push(targeting::move_marker_at(&self.game.world, c));
        }
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
        // spec: S-0214 claim 29 (successful additions keep the admitted candidate order of claim 25 restricted to the box and claim 19's eligibility, whether the box replaces or adds and whatever the drag direction; no reversal or re-sort at release)
        for &id in candidates {
            if targeting::box_eligible(w, local, &self.selection, id) {
                // spec: S-0214 claims 12 and 20 (additions use the selection order; no count limit on the selection)
                targeting::add_to_selection(w, &mut self.selection, id);
                first = first.or(Some(id));
            }
        }
        // spec: S-0701 claim 17 and S-0214 claims 20 and 29 (only the earliest successful addition in candidate order speaks, not the eventual first selected object)
        self.selected.extend(first);
        false
    }

    /// A key command for the selection (S-0213 table T2).
    pub fn key(&mut self, cmd: KeyCommand) {
        let w = &self.game.world;
        let local = Some(self.local);
        self.selection
            .retain(|&id| targeting::selectable_by(w, id, local));
        let list = targeting::key_command(w, self.local, &self.selection, cmd);
        // spec: S-0701 claim 19 (an empty selection requests no command sound)
        if self.selection.is_empty() {
            return;
        }
        for o in list {
            // spec: S-0701 claim 19 (a failed issuance exits the command without its sound, even after earlier orders)
            if !self.give(o) {
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

    /// The options menu's speed change, as an order of the local house
    /// (S-0201 claims 8 and 17); returns whether it was taken or sent.
    pub fn change_speed(&mut self, speed: u8) -> bool {
        self.give(Order::GameSpeed { speed })
    }

    /// The tab shown (S-0411 claim 2).
    pub fn shown_tab(&self) -> usize {
        hud::shown_tab(&self.game.world, self.local, self.tab)
    }

    /// The sidebar model.
    pub fn hud_model(
        &self,
        name: &dyn Fn(TypeRef) -> String,
        art: &dyn Fn(hud::Entry) -> Option<(u32, u32, i32, i32)>,
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
        hud::entries(&self.game.world, self.local, self.shown_tab()).len()
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

    /// A click on the `n`-th icon of the shown tab (S-0411 claims 4 and 5;
    /// a superweapon icon, S-0512 claim 16).
    pub fn cameo_click(&mut self, n: usize, button: Button, shift: bool) {
        let w = &self.game.world;
        let t = match hud::entries(w, self.local, self.shown_tab()).get(n) {
            Some(hud::Entry::Type(t)) => *t,
            Some(hud::Entry::Superweapon(sw)) => {
                // spec: S-0512 claim 16 (selecting an icon that offers targeting arms that type, by its registration index)
                if button == Button::Left && charge::offers_targeting(w, self.local, *sw) {
                    let sw = *sw;
                    let no_action = w
                        .rules()
                        .superweapons
                        .types
                        .get(sw as usize)
                        .is_some_and(|t| t.action.is_none());
                    if no_action {
                        // spec: S-0512 claim 23 (an otherwise fireable icon with no aim action issues that type's fire order at cell (0, 0), without entering aiming)
                        self.issue(vec![Order::FireSuperweapon {
                            sw,
                            cell: CellXY::new(0, 0),
                        }]);
                    } else {
                        // spec: S-0512 claim 23 (any recognised nonzero action enters aiming; this alone does not spend the charge)
                        self.tool = Tool::Superweapon {
                            sw,
                            from_source: false,
                        };
                    }
                }
                return;
            }
            None => return,
        };
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
