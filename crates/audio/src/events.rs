//! What the game client calls: [`Audio`] turns simulation effects, the
//! world's state, selections, orders, key commands and interface cues into
//! sound instances and announcements (S-0701 claims 16 to 28).
//!
//! Sound never feeds back into the simulation (S-0701 claim 29): [`Audio`]
//! reads effects and the world. The game-side choices of claims 17, 18, 20
//! and 23 and the per-instance draws of claims 13 and 14 all take the one
//! *unsynchronised* stream (S-0202 claim 11, S-0701 claim 30), which the
//! caller owns and passes in; the scenario random stream is never touched.

use std::collections::BTreeMap;

use glasnost_rules::quirks::Quirks;
use glasnost_sim::combat::damage::HitResult;
use glasnost_sim::combat::effects::{Effect, InfiltrationRow, Sound};
use glasnost_sim::combat::veterancy::{self, Rank};
use glasnost_sim::combat::weapon;
use glasnost_sim::econ::minimap::{self, Grid, Image, Zoom};
use glasnost_sim::econ::{TypeRef, power, production};
use glasnost_sim::orders::Order;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, FactoryKind, Kind};
use glasnost_sim::superweapons::SwEvent;
use glasnost_sim::targeting::{ClickResult, KeyCommand};
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};

use crate::bank::SoundBank;
use crate::catalog::{EntryId, FULL, SoundCatalog};
use crate::mixer::{InstanceId, MixQuirks, Mixer, Prepared, View, Where};
use crate::names::{Announcement, SoundNames, TypeSounds};
use crate::output::{Output, OutputState};
use crate::speech::{Announcer, Requested, SpeechCatalog, voice_set};

/// The feedback chance: a draw below this of R(0, 99) (S-0701 Constants).
const FEEDBACK_CHANCE: i32 = 30; // spec: S-0701 claim 20 and Constants (30 in 100)
/// The movement sound's grace counter (S-0701 Constants).
const MOVE_GRACE: i32 = 3; // spec: S-0701 claim 23 and Constants (3 ticks)
/// Low funds threshold (S-0701 Constants).
const LOW_FUNDS: i32 = 100; // spec: S-0701 claim 28 and Constants (below 100 credits)
/// Ticks per minute of the speak delay: N(⌊k × 900⌋) (S-0701 Constants).
const SPEAK_DELAY_TICKS: i64 = 900; // spec: S-0701 claim 28 and Constants (N(⌊k × 900⌋) ticks)
const SPEAK_DELAY_INITIAL_TICKS: i32 = 1800; // spec: S-0701 claim 38 (the low-funds repeat initially 2 game minutes: N(1800) ticks)
/// The level of a credit tick's request: one half (S-0416 Constants).
const CREDIT_TICK_LEVEL: i32 = FULL / 2; // spec: S-0416 claim 4; S-0701 claim 35 (half volume: request level 8192)
/// Anchors of turret sounds are offset from those of movement sounds.
const TURRET_ANCHOR: u64 = 1 << 40;

/// Interface cues the client reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiCue {
    /// The credit counter moved this cycle (S-0416 claim 4).
    CreditTick {
        /// Rising (true) or falling.
        rising: bool,
    },
    /// The local player's radar became available (S-0410 claim 3).
    RadarOn,
    /// The local player's radar became unavailable (S-0410 claim 3).
    RadarOff,
    /// A sidebar click started an item on an idle line (S-0411 claim 12).
    BuildStarted {
        /// The item is infantry.
        infantry: bool,
    },
    /// A sidebar click resumed an item (S-0411 claim 12).
    BuildResumed {
        /// The item is infantry.
        infantry: bool,
    },
    /// A sidebar click added a queued copy to an already active line
    /// (S-0411 claim 12).
    BuildQueued,
    /// A sidebar click held an item (S-0411 claim 12).
    BuildHeld,
    /// A sidebar click cancelled an item (S-0411 claim 12).
    BuildCancelled,
    /// A build click refused for a busy line (S-0411 claims 4 and 12).
    BuildRefused,
    /// A placement click refused by its tests (S-0411 claim 10, S-0406).
    PlacementRefused,
    /// A page request at the sidebar strip's requested end (S-0701 claim
    /// 24, S-0411 claim 18).
    StripEnd,
    /// An accepted page request of the sidebar strip (S-0701 claim 24,
    /// S-0411 claim 18).
    StripScroll,
    /// A building item became done (S-0411 claims 9 and 12).
    ConstructionComplete,
    /// The local player's house placed a building (S-0701 claim 23).
    BuildingPlaced,
    /// The local player set a building's rally point (S-0701 claim 28).
    RallySet,
}

/// The F7 radar-event kinds that announcements pass through.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RadarKind {
    BaseAttacked,
    HarvesterAttacked,
    UnitProduced,
    UnitLost,
    AllyAttacked,
    UnitRepaired,
    GarrisonAbandoned,
    Captured,
    Infiltrated,
}

impl RadarKind {
    /// (*r* cells, *t* ticks) of F7's table.
    fn table(self) -> (i64, i32) {
        match self {
            // spec: S-0701 F7 (base attacked 8, 600; harvester attacked 8, 400; unit produced 2, 200; unit lost 8, 200; ally attacked 8, 600)
            RadarKind::BaseAttacked => (8, 600),
            RadarKind::HarvesterAttacked => (8, 400),
            RadarKind::UnitProduced => (2, 200),
            RadarKind::UnitLost => (8, 200),
            RadarKind::AllyAttacked => (8, 600),
            // spec: S-0701 F7 (unit repaired 2, 400)
            RadarKind::UnitRepaired => (2, 400),
            // spec: S-0701 F7 (garrison abandoned 8, 400, suppresses)
            RadarKind::GarrisonAbandoned => (8, 400),
            // spec: S-0701 F7 (captured: no suppression, 100 ticks)
            RadarKind::Captured => (0, 100),
            // spec: S-0701 F7 (infiltrated: no suppression, 400 ticks)
            RadarKind::Infiltrated => (0, 400),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RadarEvent {
    kind: RadarKind,
    cell: (i32, i32),
    /// The zoom square (S-0410 claim 14); `None` before the first drawn
    /// frame after acceptance.
    zoom: Option<Zoom>,
    /// The end of the post-zoom lifetime (S-0701 F7), once zoom completed.
    until: Option<i32>,
}

/// An object's response slot (S-0701 claim 16).
#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    waiting: Option<EntryId>,
    speaking: Option<(InstanceId, EntryId)>,
}

/// An object's movement-sound episode (S-0701 claim 23).
#[derive(Clone, Copy, Debug)]
struct Motion {
    last: Pos,
    grace: i32,
    begun: bool,
}

/// The audio of a game.
pub struct Audio {
    mixer: Mixer,
    output: Box<dyn Output>,
    bank: SoundBank,
    voices: SoundBank,
    catalog: SoundCatalog,
    speech: SpeechCatalog,
    names: SoundNames,
    quirks: Quirks,
    announcer: Announcer,
    voice: usize,
    local: Option<HouseId>,
    slots: BTreeMap<ObjectId, Slot>,
    motion: BTreeMap<ObjectId, Motion>,
    turrets: BTreeMap<ObjectId, InstanceId>,
    ranks: BTreeMap<ObjectId, Rank>,
    radar: Vec<RadarEvent>,
    radar_map: Option<(Grid, Image)>,
    low_funds_after: Option<i32>,
    low_power: bool,
    now_ms: i64,
}

/// The voice set of the local player's house (0 Allied, 1 Soviet, 2 Yuri;
/// S-0701 claim 26), for choosing which announcement files to load.
pub fn local_voice_set(w: &World, local: Option<HouseId>) -> usize {
    // spec: S-0701 claim 26 (the voice set follows the local player's side: first, second, any other)
    local
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .and_then(|h| {
            w.rules()
                .econ
                .tech
                .country_side
                .get(h.country as usize)
                .copied()
                .flatten()
        })
        .map_or(2, |s| voice_set(s.max(0) as usize))
}

fn mix_quirks(q: &Quirks) -> MixQuirks {
    MixQuirks {
        sound_order_startup_draw: q.sound_order_startup_draw,
        ranged_draw_half_span: q.ranged_draw_half_span,
    }
}

impl Audio {
    /// Audio on an output, with no sound data, an empty bank and no sound
    /// names.
    pub fn new(output: Box<dyn Output>) -> Self {
        let quirks = Quirks::default();
        let mixer = Mixer::new(output.rate(), 1, mix_quirks(&quirks));
        Self {
            mixer,
            output,
            bank: SoundBank::new(),
            voices: SoundBank::new(),
            catalog: SoundCatalog::new(),
            speech: SpeechCatalog::default(),
            names: SoundNames::default(),
            quirks,
            announcer: Announcer::default(),
            voice: 0,
            local: None,
            slots: BTreeMap::new(),
            motion: BTreeMap::new(),
            turrets: BTreeMap::new(),
            ranks: BTreeMap::new(),
            radar: Vec::new(),
            radar_map: None,
            low_funds_after: None,
            low_power: false,
            now_ms: 0,
        }
    }

    /// Audio on the platform's default output
    /// ([`crate::output::platform_default`]).
    pub fn platform() -> Self {
        Self::new(crate::output::platform_default())
    }

    /// Sets the sample bank.
    pub fn set_bank(&mut self, bank: SoundBank) {
        self.bank = bank;
    }

    /// The sample bank.
    pub fn bank_mut(&mut self) -> &mut SoundBank {
        &mut self.bank
    }

    /// Sets the announcement files, each under its file name
    /// ([`crate::speech::file_name`]). Announcements are never taken from
    /// the sample bank, nor sound entries from these (S-0701 claim 34).
    pub fn set_voices(&mut self, voices: SoundBank) {
        self.voices = voices;
    }

    /// The announcement files.
    pub fn voices_mut(&mut self) -> &mut SoundBank {
        &mut self.voices
    }

    /// Sets the sound entries (S-0701 claims 1 to 4).
    pub fn set_catalog(&mut self, catalog: SoundCatalog) {
        self.catalog = catalog;
    }

    /// The sound entries.
    pub fn catalog(&self) -> &SoundCatalog {
        &self.catalog
    }

    /// Sets the announcement data (S-0701 claim 26).
    pub fn set_speech(&mut self, speech: SpeechCatalog) {
        self.speech = speech;
    }

    /// Sets the rules data's sound names (from
    /// [`SoundNames::from_ruleset`]).
    pub fn set_names(&mut self, names: SoundNames) {
        self.names = names;
    }

    /// The sound names.
    pub fn names(&self) -> &SoundNames {
        &self.names
    }

    /// Sets the quirk switches (the game's rules).
    pub fn set_quirks(&mut self, q: Quirks) {
        self.quirks = q;
        self.mixer = Mixer::new(self.output.rate(), 1, mix_quirks(&q));
    }

    /// The mixer (offline rendering, tests, volume).
    pub fn mixer_mut(&mut self) -> &mut Mixer {
        &mut self.mixer
    }

    /// The mixer.
    pub fn mixer(&self) -> &Mixer {
        &self.mixer
    }

    /// Starts a game: the local player's house and its voice set.
    pub fn start(&mut self, w: &World, local: Option<HouseId>) {
        self.mixer.clear();
        self.announcer.reset();
        self.local = local;
        self.voice = local_voice_set(w, local);
        self.slots.clear();
        self.motion.clear();
        self.turrets.clear();
        self.ranks.clear();
        self.radar.clear();
        self.radar_map = minimap::grid(w).map(|g| (g, minimap::image(&g)));
        self.low_funds_after = None;
        self.low_power = false;
    }

    /// The voice set in use (0 Allied, 1 Soviet, 2 Yuri).
    pub fn voice_set(&self) -> usize {
        self.voice
    }

    /// Sets the battlefield view. Call every frame.
    pub fn set_view(&mut self, v: View) {
        self.mixer.set_view(v);
    }

    /// Starts or resumes the output; call from a user-input handler.
    pub fn resume(&mut self) {
        self.output.resume();
    }

    /// Services the audio at host time `now_ms` (the audio clock): an audio
    /// pass when one is due (claim 12), the next announcement (claim 27),
    /// then the output. Call once per drawn frame, after the frame's events.
    /// The per-instance draws of the pass and of the rendered samples take
    /// `unsync`, the client's unsynchronised stream.
    pub fn frame(&mut self, now_ms: i64, unsync: &mut Rng) {
        // spec: S-0701 claim 30 (per-instance choices on the shared unsynchronised stream; no separate presentation generator)
        std::mem::swap(self.mixer.rng_mut(), unsync);
        self.serve(now_ms);
        self.output.pump(&mut self.mixer);
        std::mem::swap(self.mixer.rng_mut(), unsync);
    }

    /// [`Audio::frame`] without feeding the output (offline rendering).
    pub fn service(&mut self, now_ms: i64, unsync: &mut Rng) {
        // spec: S-0701 claim 30 (per-instance choices on the shared unsynchronised stream)
        std::mem::swap(self.mixer.rng_mut(), unsync);
        self.serve(now_ms);
        std::mem::swap(self.mixer.rng_mut(), unsync);
    }

    fn serve(&mut self, now_ms: i64) {
        self.now_ms = now_ms;
        self.mixer.service(now_ms);
        if let Some(id) = self.announcer.next(now_ms, self.mixer.speaking()) {
            let sample = &self.speech.line(id).samples[self.voice];
            // spec: S-0701 claim 34 (a separate file, the eight-character name plus `.wav`, never from the bank)
            let file = crate::speech::file_name(sample);
            // spec: S-0701 claim 26 (quirks.announcement_volume_ignored: the volume key has no effect)
            // spec: S-0701 claim 35 (neither the master level nor the sound volume option applies; the voice option is at full: our engine has no options screen)
            if let Some(clip) = self.voices.clip(&file).filter(|c| c.frames() > 0) {
                self.mixer.speak(clip);
            }
        }
    }

    /// The output's state.
    pub fn state(&self) -> OutputState {
        self.output.state()
    }

    /// A sound entry with its loaded samples, or `None` when it never
    /// plays (S-0701 claim 4 and Edge cases).
    pub fn prepare(&mut self, id: EntryId) -> Option<Prepared> {
        let entry = self.catalog.entry_arc(id);
        let mut clips = Vec::new();
        for s in &entry.samples {
            match self.bank.clip(s).filter(|c| c.frames() > 0) {
                Some(c) => clips.push(c),
                // spec: S-0701 claim 4 (quirks.missing_sample_silences_entry: one missing sample silences the whole entry; off: it is left out)
                None if self.quirks.missing_sample_silences_entry => return None,
                None => {}
            }
        }
        if clips.is_empty() {
            return None;
        }
        let n = clips.len() as i32;
        let a = entry.attack.clamp(0, n);
        let d = entry.decay.clamp(0, n - a);
        // spec: S-0701 Edge cases (quirks.attack_decay_without_body: an attack or decay entry with no body sample is silent; off: attack and decay play)
        if self.quirks.attack_decay_without_body
            && (entry.control.attack || entry.control.decay)
            && n - a - d <= 0
        {
            return None;
        }
        Some(Prepared { id, entry, clips })
    }

    /// Requests an entry at a placement; `w` gives the shroud for *shroud*
    /// entries (claim 6).
    fn request(
        &mut self,
        w: Option<&World>,
        id: EntryId,
        at: Where,
        scale: i32,
    ) -> Option<InstanceId> {
        let prep = self.prepare(id)?;
        if prep.entry.placement.shroud
            && let Some(p) = match at {
                Where::Screen => None,
                Where::Fixed(p) | Where::Follow(p) | Where::Attached { pos: p, .. } => Some(p),
            }
        {
            let cell = p.cell();
            // spec: S-0701 claim 6 and Edge cases (quirks.shroud_sound_corner_cell: a shroud sound at cell (0, 0) is never heard)
            if self.quirks.shroud_sound_corner_cell && cell.x == 0 && cell.y == 0 {
                return None;
            }
            // spec: S-0701 claim 6 (shroud: silent when the cell is not explored by the local player)
            if let (Some(w), Some(h)) = (w, self.local)
                && !glasnost_sim::econ::shroud::explored(w, h, cell)
            {
                return None;
            }
        }
        self.mixer.request(prep, at, scale).ok()
    }

    /// Plays a sound by name at a placement (unknown names play nothing,
    /// claim 1).
    pub fn play_named(&mut self, name: Option<&str>, at: Where) -> Option<InstanceId> {
        let id = self.catalog.resolve(name?)?;
        self.request(None, id, at, FULL)
    }

    fn play_in(&mut self, w: &World, name: Option<&str>, at: Where) -> Option<InstanceId> {
        let id = self.catalog.resolve(name?)?;
        self.request(Some(w), id, at, FULL)
    }

    /// F5: one entry of a list chosen by one raw value, `x mod n` over the
    /// list with unknown names dropped; no value when it is empty.
    fn choose(&self, list: &[String], unsync: &mut Rng) -> Option<EntryId> {
        // spec: S-0701 Import mapping (lists drop unknown names)
        let ids: Vec<EntryId> = list
            .iter()
            .filter_map(|n| self.catalog.resolve(n))
            .collect();
        if ids.is_empty() {
            // spec: S-0701 F5 (no raw value when n = 0)
            return None;
        }
        // spec: S-0701 F5 (i = x mod n, x the next raw value)
        let x = unsync.raw();
        Some(ids[(x % ids.len() as u32) as usize])
    }

    fn single(&self, name: Option<&String>) -> Option<EntryId> {
        self.catalog.resolve(name?)
    }

    fn sounds(&self, t: TypeRef) -> TypeSounds {
        self.names
            .of_type(t.kind, t.index)
            .cloned()
            .unwrap_or_default()
    }

    /// Puts a response in an object's slot (claim 16).
    fn respond(&mut self, id: ObjectId, e: Option<EntryId>) {
        if let Some(e) = e {
            // spec: S-0701 claim 16 (replacing any response already waiting)
            self.slots.entry(id).or_default().waiting = Some(e);
        }
    }

    /// Plays an announcement for the local player (claims 27 and 28).
    pub fn announce(&mut self, a: Announcement) {
        let Some(name) = self.names.announcement(a) else {
            return;
        };
        let Some(id) = self.speech.resolve(name) else {
            return;
        };
        if self
            .announcer
            .request(&self.speech, id, self.mixer.speaking())
            == Requested::StopCurrent
        {
            self.mixer.stop_speech();
        }
    }

    /// F7: tries to create a radar event; false when a nearby one of the
    /// same kind suppresses it.
    fn radar_event(&mut self, kind: RadarKind, cell: (i32, i32), tick: i32) -> bool {
        // spec: S-0410 claim 14, S-0701 F7 (an event lasts through its zoom, then t ticks from the completion frame's tick)
        self.radar.retain(|e| e.until.is_none_or(|u| u > tick));
        let (r, _) = kind.table();
        let near = self.radar.iter().any(|e| {
            let dx = i64::from(e.cell.0 - cell.0);
            let dy = i64::from(e.cell.1 - cell.1);
            // spec: S-0701 F7 (⌊√(Δx² + Δy²)⌋ < r cells, for a kind that suppresses)
            e.kind == kind && ((dx * dx + dy * dy) as u64).isqrt() < r as u64
        });
        if near {
            return false;
        }
        self.radar.push(RadarEvent {
            kind,
            cell,
            zoom: None,
            until: None,
        });
        true
    }

    /// Advances the zoom of every radar event for one drawn frame of the
    /// game (S-0410 claims 14 and 15, F9): once per cycle, frozen or not,
    /// with or without radar. `origin` is the radar map's top-left pixel
    /// on the screen.
    pub fn radar_frame(&mut self, w: &World, origin: (i32, i32)) {
        let Some((g, img)) = self.radar_map else {
            return;
        };
        let rules = w.rules().econ.radar.zoom;
        // spec: S-0410 F9 (Oₓ, Oᵧ: the image's top-left pixel in screen coordinates)
        let o = (origin.0 + img.left, origin.1 + img.top);
        for e in &mut self.radar {
            if e.until.is_some() {
                continue;
            }
            let z = e.zoom.get_or_insert_with(|| {
                let c = minimap::event_centre(&g, &img, o, e.cell.0, e.cell.1);
                Zoom::new(&rules, c, img.width, img.height)
            });
            // spec: S-0410 claim 14 (the first drawn frame after acceptance advances the initial square once)
            if z.advance(&rules) {
                // spec: S-0410 claim 14 (completion starts S-0701 F7's post-zoom lifetime at the tick number of that drawn frame)
                e.until = Some(w.tick + e.kind.table().1);
            }
        }
    }

    /// Handles one tick: its effects (claims 20 to 23 and 28), the response
    /// slots (claim 16), movement, turret and promotion sounds (claim 23) and
    /// the low funds and low power announcements (claim 28). Call after each
    /// simulation step with that step's effects.
    pub fn tick(&mut self, w: &World, effects: &[Effect], unsync: &mut Rng) {
        for e in effects {
            self.effect(w, e, unsync);
        }
        self.serve_slots(w);
        self.scan(w, unsync);
        self.house_announcements(w);
    }

    fn effect(&mut self, w: &World, e: &Effect, unsync: &mut Rng) {
        match e {
            // spec: S-0701 claim 22, S-0109 claim 13 (an animation's start sound, positional, any owner)
            Effect::Anim { name, pos, .. } => {
                let s = self.names.anim_report(name).map(str::to_owned);
                self.play_in(w, s.as_deref(), Where::Fixed(*pos));
            }
            Effect::Shot {
                weapon,
                from,
                report,
                ..
            } => {
                let name = w.rules().combat.weapon(*weapon).name.clone();
                // spec: S-0701 claim 21 (the report at the firing position, positional, any owner; no draw: entry r mod n of the list, r the firer's creation-derived report value)
                let list = self.names.report(&name);
                let entry = (!list.is_empty())
                    .then(|| &list[usize::from(*report) % list.len()])
                    .and_then(|n| self.catalog.resolve(n));
                if let Some(id) = entry {
                    self.request(Some(w), id, Where::Fixed(*from), FULL);
                }
            }
            Effect::Hit {
                id,
                of,
                owner,
                pos,
                result,
                attacker,
            } => self.hit(w, *id, *of, *owner, *pos, *result, *attacker, unsync),
            // A detonation's sounds are those of the animations it creates
            // (claim 22); a kill's are claim 20's, through Effect::Hit.
            Effect::Killed { .. } | Effect::Detonation { .. } => {}
            // spec: S-0011 claim 32 (play sound, and sound at a waypoint: the sound at the cell's centre)
            Effect::Trigger {
                code: 19 | 99,
                name,
                pos,
                ..
            } => {
                let at = pos.map_or(Where::Screen, Where::Fixed);
                self.play_in(w, Some(name), at);
            }
            Effect::Trigger { .. } => {}
            Effect::Sound {
                sound,
                of,
                house,
                pos,
            } => self.sound(w, *sound, *of, *house, *pos),
            Effect::Captured {
                of,
                old,
                new,
                pos,
                announce,
                ..
            } => self.captured(w, *of, *old, *new, *pos, *announce),
            Effect::Infiltrated {
                infiltrator,
                victim,
                rows,
                satellite,
                pos,
                ..
            } => self.infiltrated(w, *infiltrator, *victim, rows, *satellite, *pos),
            // spec: S-0210 claim 8 (the crushed overlay's crush sound at the crusher)
            // The cloak events are presentation-only (selection, S-0516 claim 26).
            Effect::Cloak { .. } => {}
            // spec: S-0518 claim 6 (the support beam is drawing only: no sound)
            Effect::SupportBeam { .. } => {}
            Effect::Superweapon { event, sw, pos, .. } => self.superweapon(w, *event, *sw, *pos),
            Effect::OverlayCrushed { overlay, pos } => {
                let s = self
                    .names
                    .overlay_crush(u32::from(*overlay))
                    .map(str::to_owned);
                self.play_in(w, s.as_deref(), Where::Fixed(*pos));
            }
        }
    }

    /// A building changed owner (S-0509 claim 12, S-0701 claim 39): eligible
    /// when the cause requests an announcement, the local player is the old
    /// or the new owner and the new owner's country is not passive. A tech
    /// building (the needs-engineer flag) uses its imported capture
    /// announcement when the local player gains it (silent without one) and
    /// the tech-building-lost entry when the local player loses it; any
    /// other building the ordinary captured entry. A *captured* radar event
    /// is created at its cell.
    fn captured(
        &mut self,
        w: &World,
        of: TypeRef,
        old: Option<HouseId>,
        new: Option<HouseId>,
        pos: Pos,
        announce: bool,
    ) {
        let local = self.local;
        let involved = local.is_some() && (local == old || local == new);
        if !announce || !involved || new.is_none_or(|h| glasnost_sim::econ::setup::passive(w, h)) {
            return;
        }
        let tech = of.kind == Kind::Building
            && w.rules()
                .buildings
                .get(of.index as usize)
                // spec: S-0701 claim 39 and S-0405 claim 8 (the needs-engineer flag)
                .is_some_and(|b| b.raw.needs_engineer.unwrap_or(false));
        let created = self.radar_event(RadarKind::Captured, cell_of(pos), w.tick);
        let _ = created;
        if !tech {
            // spec: S-0701 claim 39 (a building without the flag: the ordinary building-captured entry for the eligible announcement)
            self.announce(Announcement::BuildingCaptured);
        } else if local == new {
            // spec: S-0701 claim 39 (the imported capture announcement when the new owner is the local player; with none, silent, no fallback to the ordinary entry)
            let name = self.sounds(of).capture_announcement;
            self.announce_entry(name.as_deref());
        } else {
            // spec: S-0701 claim 39 (losing such a building as the local player: the tech-building-lost entry)
            self.announce(Announcement::TechBuildingLost);
        }
    }

    /// A spy infiltrated a building (S-0510 claim 11): when the local player
    /// is the infiltrating house or the victim house and the two differ, an
    /// *infiltrated* radar event is created at the building's cell (that
    /// kind never suppresses), and the lines of each row follow: the victim
    /// hears the sabotage or theft line, the infiltrator hears *building
    /// infiltrated* and then the row's line (one line for radar and cash).
    fn infiltrated(
        &mut self,
        w: &World,
        infiltrator: Option<HouseId>,
        victim: Option<HouseId>,
        rows: &[InfiltrationRow],
        satellite: bool,
        pos: Pos,
    ) {
        use Announcement as A;
        let local = self.local;
        if local.is_none() || infiltrator == victim || (local != infiltrator && local != victim) {
            return;
        }
        let _ = self.radar_event(RadarKind::Infiltrated, cell_of(pos), w.tick);
        for &row in rows {
            let lines: &[Announcement] = if local == victim {
                match row {
                    // spec: S-0510 claim 11 (row E1, the local player is the victim: radar sabotaged, unless the victim has an active spy satellite)
                    InfiltrationRow::Radar if satellite => &[],
                    InfiltrationRow::Radar => &[A::RadarSabotaged],
                    // spec: S-0510 claim 11 (row E2 victim: power sabotaged)
                    InfiltrationRow::Power => &[A::PowerSabotaged],
                    // spec: S-0510 claim 11 (rows E3, E6 and E7 victim: technology stolen)
                    InfiltrationRow::Tech
                    | InfiltrationRow::VehicleFactory
                    | InfiltrationRow::InfantryFactory => &[A::TechnologyStolen],
                    // spec: S-0510 claim 11 (row E4 victim: building infiltrated)
                    InfiltrationRow::Superweapon => &[A::BuildingInfiltrated],
                    // spec: S-0510 claim 11 (row E5 victim: cash stolen)
                    InfiltrationRow::Cash => &[A::CashStolen],
                }
            } else {
                match row {
                    // spec: S-0510 claim 11 (row E1, the local player is the infiltrator: building infiltrated, radar sabotaged as one line, unless the victim has an active spy satellite)
                    InfiltrationRow::Radar if satellite => &[],
                    InfiltrationRow::Radar => &[A::BuildingInfRadarSabotaged],
                    // spec: S-0510 claim 11 (row E2 infiltrator: building infiltrated, then enemy base powered down)
                    InfiltrationRow::Power => &[A::BuildingInfiltrated, A::EnemyBasePoweredDown],
                    // spec: S-0510 claim 11 (rows E3, E6 and E7 infiltrator: building infiltrated, then new technology acquired)
                    InfiltrationRow::Tech
                    | InfiltrationRow::VehicleFactory
                    | InfiltrationRow::InfantryFactory => {
                        &[A::BuildingInfiltrated, A::NewTechnologyAcquired]
                    }
                    // spec: S-0510 claim 11 (row E4 infiltrator: building infiltrated)
                    InfiltrationRow::Superweapon => &[A::BuildingInfiltrated],
                    // spec: S-0510 claim 11 (row E5 infiltrator: building infiltrated, cash stolen as one line)
                    InfiltrationRow::Cash => &[A::BuildingInfCashStolen],
                }
            };
            for &a in lines {
                self.announce(a);
            }
        }
    }

    /// Plays the announcement entry `name` for the local player (claim 27).
    fn announce_entry(&mut self, name: Option<&str>) {
        let Some(id) = name.and_then(|n| self.speech.resolve(n)) else {
            return;
        };
        if self
            .announcer
            .request(&self.speech, id, self.mixer.speaking())
            == Requested::StopCurrent
        {
            self.mixer.stop_speech();
        }
    }

    /// The positional sounds of superweapon events (S-0513 claims 5, 18,
    /// 32 and 44; S-0701 claim 25). The sound names come from the rules.
    fn superweapon(&mut self, w: &World, event: SwEvent, sw: Option<u32>, pos: Option<Pos>) {
        let at = pos.map_or(Where::Screen, Where::Fixed);
        let r = w.rules_arc();
        let s = &r.superweapons;
        let ty = sw.and_then(|i| s.types.get(i as usize));
        let name = match event {
            // spec: S-0513 claim 5 (the nuclear siren at the target point)
            SwEvent::Siren => s.dig_sound.clone(),
            // spec: S-0513 claim 15 (the storm sound at the storm's start)
            SwEvent::StormStart => s.storm.sound.clone(),
            // spec: S-0701 claim 25 (the lightning sound the simulation chose, at the strike point)
            SwEvent::Lightning(k) => s.storm.sounds.get(k as usize).cloned(),
            // spec: S-0513 claim 32 (the type's start and special sounds at the target)
            SwEvent::ShieldStart => ty.and_then(|t| t.start_sound.clone()),
            SwEvent::ShieldFade => ty.and_then(|t| t.special_sound.clone()),
            // spec: S-0513 claim 44 (the camera sound at the plane)
            SwEvent::Camera => s.camera_sound.clone(),
            // spec: S-0610 claim 13 (the psychic reveal's activation sound at the target cell's centre, positionally, for every player)
            SwEvent::RevealSound => s.reveal_sound.clone(),
            // Announcements, messages, markers and radar events are the
            // interface's (S-0701 claim 28; S-0410).
            _ => None,
        };
        self.play_in(w, name.as_deref(), at);
    }

    fn sound(
        &mut self,
        w: &World,
        sound: Sound,
        of: Option<TypeRef>,
        house: Option<HouseId>,
        pos: Option<Pos>,
    ) {
        let ts = of.map(|t| self.sounds(t)).unwrap_or_default();
        let at = pos.map_or(Where::Screen, Where::Fixed);
        // A sound that concerns a house is heard only by that house's player
        // (S-0408 claims 6, 11 and 12; S-0415 claims 4 and 9).
        let mine = house.is_none() || house == self.local;
        match sound {
            // spec: S-0415 claims 6 and 9 (the type's deploy sound at its position)
            Sound::Deploy if mine => {
                self.play_in(w, ts.deploy.as_deref(), at);
            }
            // spec: S-0517 claim 12 (the vehicle type's undeploy sound at its position)
            Sound::Undeploy if mine => {
                self.play_in(w, ts.undeploy.as_deref(), at);
            }
            // spec: S-0415 claim 9; S-0701 claim 23 (the pack-up sound, positional at the build-down start)
            Sound::Packup if mine => {
                self.play_in(w, ts.packup.as_deref(), at);
            }
            // spec: S-0408 claims 6 and 12; S-0701 claim 23 (positional at the building; a wall sale's as a screen sound)
            Sound::Sell if mine => {
                let s = self.names.sell.clone();
                self.play_in(w, s.as_deref(), at);
            }
            // spec: S-0415 claim 4 (the refusal announcement)
            Sound::CannotDeploy if mine => self.announce(Announcement::CannotDeploy),
            // spec: S-0408 claim 11; S-0701 claim 28 (unit sold)
            Sound::UnitSold if mine => self.announce(Announcement::UnitSold),
            // spec: S-0701 claim 24; S-0404 claim 3 (a request refused at application: the game-wide refusal sound, the local player's house only)
            Sound::QueueRefused if house.is_some() && house == self.local => {
                let s = self.names.scold.clone();
                self.play_in(w, s.as_deref(), Where::Screen);
            }
            Sound::UnitReady => {
                let Some(t) = of else { return };
                // spec: S-0701 claim 23 (the type's creation sound, else the game-wide one of its kind; any owner)
                let kind_wide = match t.kind {
                    Kind::Infantry => self.names.create_infantry.clone(),
                    Kind::Aircraft => self.names.create_aircraft.clone(),
                    _ => self.names.create_unit.clone(),
                };
                let id = self
                    .single(ts.create.as_ref())
                    .or_else(|| self.single(kind_wide.as_ref()));
                if let Some(id) = id {
                    self.request(Some(w), id, at, FULL);
                }
                // spec: S-0701 claim 28 (unit ready for the local player, F7 kind unit produced at the factory)
                if house.is_some()
                    && house == self.local
                    && let Some(p) = pos
                    && self.radar_event(RadarKind::UnitProduced, cell_of(p), w.tick)
                {
                    self.announce(Announcement::UnitReady);
                }
            }
            // spec: S-0314 claim 6; S-0701 claim 28 (repair complete: the depot owner's, only when it is the local player, through the F7 kind unit repaired at the depot; suppressed with the event)
            Sound::UnitRepaired => {
                if house.is_some()
                    && house == self.local
                    && let Some(p) = pos
                    && self.radar_event(RadarKind::UnitRepaired, cell_of(p), w.tick)
                {
                    self.announce(Announcement::RepairComplete);
                }
            }
            // spec: S-0210 claim 4 (the crushed type's crush sound at the crusher's position, any owner)
            Sound::Crush => {
                self.play_in(w, ts.crush.as_deref(), at);
            }
            // spec: S-0207 claim 20 (e) (the departure sound at the old point; the type's sound overrides the game-wide one)
            Sound::ChronoOut => {
                self.play_in(w, ts.chrono_out.as_deref(), at);
            }
            // spec: S-0207 claim 20 (e) (the arrival sound at L; the type's sound overrides the game-wide one)
            Sound::ChronoIn => {
                self.play_in(w, ts.chrono_in.as_deref(), at);
            }
            // spec: S-0516 claim 15 (the game-wide cloak sound at the object; no type-specific sound)
            Sound::Cloak => {
                let s = w.rules().cloak.sound.clone();
                self.play_in(w, s.as_deref(), at);
            }
            // spec: S-0508 claim 6 and S-0701 claim 28 (structure garrisoned: the local player's entering unit, regardless of the building's owner; the garrisoned sound at the entering unit's last position)
            Sound::Garrisoned if house.is_some() && house == self.local => {
                let s = self.names.building_garrisoned.clone();
                self.play_in(w, s.as_deref(), at);
                self.announce(Announcement::StructureGarrisoned);
            }
            // spec: S-0508 claim 8 and S-0701 claim 28 (structure abandoned: the owner that lost the building, when it is the local player; the abandoned sound on the screen, unthrottled; the announcement only when F7 creates a garrison abandoned event at the top-left cell)
            Sound::Abandoned if house.is_some() && house == self.local => {
                let s = self.names.building_abandoned.clone();
                self.play_in(w, s.as_deref(), Where::Screen);
                if let Some(p) = pos
                    && self.radar_event(RadarKind::GarrisonAbandoned, cell_of(p), w.tick)
                {
                    self.announce(Announcement::StructureAbandoned);
                }
            }
            // spec: S-0509 claim 5 (the game-wide building-repaired sound at the building's centre, for every player)
            Sound::BuildingRepaired => {
                let s = self.names.building_repaired.clone();
                self.play_in(w, s.as_deref(), at);
            }
            // spec: S-0314 claim 6 (the depot's low-funds announcement, the local player's only, with no suppression)
            Sound::LowFunds if house.is_some() && house == self.local => {
                self.announce(Announcement::LowFunds);
            }
            _ => {}
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn hit(
        &mut self,
        w: &World,
        id: ObjectId,
        of: TypeRef,
        owner: Option<HouseId>,
        pos: Pos,
        result: HitResult,
        attacker: Option<ObjectId>,
        unsync: &mut Rng,
    ) {
        let ts = self.sounds(of);
        let local = owner.is_some() && owner == self.local;
        match result {
            // spec: S-0701 claim 20 (unchanged or any result beyond now dead: the damage sound at the object, positional, any owner)
            HitResult::Unchanged | HitResult::PostMortem => {
                if let Some(e) = self.single(ts.damage_sound.as_ref()) {
                    // spec: S-0701 claim 7 (the damage sound keeps its map position and is placed again at passes)
                    // spec: S-0701 claim 20 and Edge cases (exactly one instance per damage-sound event: the original's second instance is a departure without a switch)
                    self.request(Some(w), e, Where::Follow(pos), FULL);
                }
            }
            HitResult::NowYellow => {
                let has = ts
                    .voice_feedback
                    .iter()
                    .any(|n| self.catalog.resolve(n).is_some());
                if has {
                    // spec: S-0701 claim 20 and 31 (one R(0, 99) for every owner's object with a feedback list)
                    let r = unsync.ranged(0, 99, self.quirks.ranged_draw_half_span);
                    // spec: S-0701 claim 20 (below 30 and the local player's: one feedback response, positional, not through the slot)
                    if r < FEEDBACK_CHANCE
                        && local
                        && let Some(e) = self.choose(&ts.voice_feedback, unsync)
                    {
                        self.request(Some(w), e, Where::Fixed(pos), FULL);
                    }
                }
            }
            HitResult::NowDead => {
                // spec: S-0701 claim 20 (now dead: the local player's object first chooses one death response (F5), at the object, positional)
                if local && let Some(e) = self.choose(&ts.voice_die, unsync) {
                    self.request(Some(w), e, Where::Fixed(pos), FULL);
                }
                // spec: S-0701 claim 20 (then, for every owner, one death sound (F5), at the object, positional)
                if let Some(e) = self.choose(&ts.die_sound, unsync) {
                    self.request(Some(w), e, Where::Fixed(pos), FULL);
                }
            }
            // spec: S-0701 claim 20 (unaffected or now red: nothing)
            HitResult::Unaffected | HitResult::NowRed => {}
        }
        if of.kind == Kind::Building {
            self.building_fallback(w, id, of, &ts, pos, result);
        }
        self.attack_announcements(w, of, owner, pos, result, attacker);
    }

    /// The game-wide building damage and death sounds of a building type
    /// with none of its own (claim 37).
    fn building_fallback(
        &mut self,
        w: &World,
        id: ObjectId,
        of: TypeRef,
        ts: &TypeSounds,
        pos: Pos,
        result: HitResult,
    ) {
        let name = match result {
            // spec: S-0701 claim 37 (now yellow or now red, and the type names no damage sound of its own)
            HitResult::NowYellow | HitResult::NowRed
                if self.single(ts.damage_sound.as_ref()).is_none() =>
            {
                self.names.building_damage.clone()
            }
            // spec: S-0701 claim 37 (destroyed, and the type's death-sound list is empty; lists drop unknown names)
            HitResult::NowDead
                if ts
                    .die_sound
                    .iter()
                    .all(|n| self.catalog.resolve(n).is_none()) =>
            {
                self.names.building_die.clone()
            }
            _ => None,
        };
        let Some(e) = name.as_deref().and_then(|n| self.catalog.resolve(n)) else {
            return;
        };
        // spec: S-0701 claim 37 (at the building's centre); S-0204 claim 4 (its reference point: position plus 128 × (W − 1, H − 1))
        let centre = if w.obj(id).is_some() {
            w.reference_point(id)
        } else {
            let (fw, fh) = w
                .rules()
                .buildings
                .get(of.index as usize)
                .map_or((1, 1), |b| b.foundation);
            Pos::new(pos.x + 128 * (fw - 1), pos.y + 128 * (fh - 1), pos.z)
        };
        // spec: S-0701 claim 37 and claim 7 (the attached form that keeps its map position and follows scrolling)
        self.request(Some(w), e, Where::Follow(centre), FULL);
    }

    /// Claim 28's attack and loss announcements of one hit.
    fn attack_announcements(
        &mut self,
        w: &World,
        of: TypeRef,
        owner: Option<HouseId>,
        pos: Pos,
        result: HitResult,
        attacker: Option<ObjectId>,
    ) {
        let (Some(local), Some(owner)) = (self.local, owner) else {
            return;
        };
        if result == HitResult::Unaffected {
            return;
        }
        let cell = cell_of(pos);
        let tick = w.tick;
        let r = w.rules();
        if of.kind == Kind::Building && attacker.is_some() {
            // Walls are overlays in our engine, never hit as buildings; the exempt types' flag is an open question of S-0701.
            if owner == local {
                // spec: S-0701 claim 28 (base under attack: F7 kind base attacked; also the game-wide base under attack sound, a screen sound)
                if self.radar_event(RadarKind::BaseAttacked, cell, tick) {
                    self.announce(Announcement::BaseUnderAttack);
                    let s = self.names.base_under_attack.clone();
                    self.play_named(s.as_deref(), Where::Screen);
                }
            } else {
                let passive = w
                    .houses
                    .get(usize::from(owner.0))
                    .and_then(|h| r.countries.get(h.country as usize))
                    .is_some_and(|c| c.passive);
                // spec: S-0701 claim 28 (ally under attack: a non-passive house allied to the local player; F7 kind ally attacked; the same sound)
                if !passive
                    && w.allied(Some(local), Some(owner))
                    && self.radar_event(RadarKind::AllyAttacked, cell, tick)
                {
                    self.announce(Announcement::AllyUnderAttack);
                    let s = self.names.base_under_attack.clone();
                    self.play_named(s.as_deref(), Where::Screen);
                }
            }
        }
        if owner != local {
            return;
        }
        let harvester = r.unit(of.kind, of.index).is_some_and(|u| u.harvester);
        // spec: S-0701 claim 28 (harvester under attack: damage without dying; F7 kind harvester attacked)
        if harvester
            && result != HitResult::NowDead
            && self.radar_event(RadarKind::HarvesterAttacked, cell, tick)
        {
            self.announce(Announcement::HarvesterUnderAttack);
        }
        // spec: S-0701 claim 28 (unit lost: an infantry unit, vehicle or aircraft of the local player dies; F7 kind unit lost)
        if result == HitResult::NowDead
            && matches!(of.kind, Kind::Infantry | Kind::Vehicle | Kind::Aircraft)
            && self.radar_event(RadarKind::UnitLost, cell, tick)
        {
            self.announce(Announcement::UnitLost);
        }
    }

    /// The response slots (claim 16): at every tick a waiting response is
    /// spoken if the object is not speaking, dropped if it is the entry being
    /// spoken, and kept otherwise.
    fn serve_slots(&mut self, w: &World) {
        let ids: Vec<ObjectId> = self.slots.keys().copied().collect();
        for id in ids {
            if w.obj(id).is_none() {
                self.slots.remove(&id);
                continue;
            }
            let slot = self.slots[&id];
            let Some(r) = slot.waiting else { continue };
            let speaking = slot
                .speaking
                .filter(|(i, _)| self.mixer.is_alive(*i))
                .map(|(_, e)| e);
            let s = self.slots.get_mut(&id).expect("present");
            match speaking {
                // spec: S-0701 claim 16 (the same entry as the one being spoken is dropped)
                Some(e) if e == r => s.waiting = None,
                // spec: S-0701 claim 16 (a different one keeps waiting)
                Some(_) => {}
                None => {
                    s.waiting = None;
                    // spec: S-0701 claim 16 and 5 (spoken as a screen sound)
                    let inst = self.request(Some(w), r, Where::Screen, FULL);
                    let s = self.slots.get_mut(&id).expect("present");
                    s.speaking = inst.map(|i| (i, r));
                }
            }
        }
    }

    /// Movement, turret and promotion sounds from the world (claim 23).
    fn scan(&mut self, w: &World, unsync: &mut Rng) {
        let gone: Vec<ObjectId> = self
            .motion
            .keys()
            .chain(self.turrets.keys())
            .copied()
            .filter(|id| w.obj(*id).is_none())
            .collect();
        for id in gone {
            self.mixer.finish_anchor(u64::from(id.0));
            self.mixer.finish_anchor(TURRET_ANCHOR | u64::from(id.0));
            self.motion.remove(&id);
            self.turrets.remove(&id);
        }
        self.ranks.retain(|id, _| w.obj(*id).is_some());
        for (&id, o) in &w.objects {
            if !o.kind.is_mobile() && o.kind != Kind::Building {
                continue;
            }
            let ts = match self.names.of_type(o.kind, o.type_index) {
                Some(t) => t.clone(),
                None => TypeSounds::default(),
            };
            let anchor = u64::from(id.0);
            if o.kind.is_mobile() && !ts.move_sound.is_empty() {
                let m = self.motion.entry(id).or_insert(Motion {
                    last: o.pos,
                    grace: 0,
                    begun: false,
                });
                let moved = m.last != o.pos;
                m.last = o.pos;
                let mut m = *m;
                if moved {
                    if !m.begun {
                        // spec: S-0701 claim 23 (an episode start chooses one movement-list entry (F5), requested as an attached sound; marked begun even if inaudible)
                        m.begun = true;
                        if let Some(e) = self.choose(&ts.move_sound, unsync) {
                            self.request(Some(w), e, Where::Attached { anchor, pos: o.pos }, FULL);
                        }
                    }
                    // spec: S-0701 claim 23 (movement resets a grace counter to 3)
                    m.grace = MOVE_GRACE;
                    self.mixer.move_anchor(anchor, o.pos);
                } else if m.begun {
                    if m.grace > 0 {
                        m.grace -= 1;
                    } else {
                        // spec: S-0701 claim 23 (after three updates the following one asks the sound to finish looping)
                        self.mixer.finish_anchor(anchor);
                        m.begun = false;
                    }
                }
                self.motion.insert(id, m);
            }
            if let Some(turret) = self.single(ts.turret_rotate.as_ref()) {
                let t_anchor = TURRET_ANCHOR | anchor;
                // spec: S-0206 claim 17 (the sound update uses the latest rotation sample, not the facing again)
                let turning = o.rotation_sample;
                let live = self
                    .turrets
                    .get(&id)
                    .copied()
                    .filter(|i| self.mixer.is_alive(*i));
                match (turning, live) {
                    // spec: S-0701 claim 23 (the rotation sound as an attached sound while the turret turns)
                    // spec: S-0701 claim 23 (a turning sample with no instance of its own starts a fresh one, even while an earlier one still finishes; a non-turning sample asks it to end looping, no grace counter, no cut)
                    (true, None) => {
                        if let Some(i) = self.request(
                            Some(w),
                            turret,
                            Where::Attached {
                                anchor: t_anchor,
                                pos: o.pos,
                            },
                            FULL,
                        ) {
                            self.turrets.insert(id, i);
                        }
                    }
                    (true, Some(_)) => self.mixer.move_anchor(t_anchor, o.pos),
                    (false, Some(_)) => {
                        self.mixer.finish_anchor(t_anchor);
                        self.turrets.remove(&id);
                    }
                    (false, None) => {}
                }
            }
            if o.owner.is_some() && o.owner == self.local {
                let rank = veterancy::rank(w, id);
                let before = self.ranks.insert(id, rank);
                if let Some(b) = before
                    && rank_value(rank) > rank_value(b)
                {
                    // spec: S-0701 claim 23 (the veteran or elite promotion sound, for the local player's objects, positional)
                    let s = match rank {
                        Rank::Elite => self.names.upgrade_elite.clone(),
                        _ => self.names.upgrade_veteran.clone(),
                    };
                    self.play_in(w, s.as_deref(), Where::Fixed(o.pos));
                    // spec: S-0701 claim 36 (a promotion to either rank requests the one unit-promoted entry)
                    self.announce(Announcement::Promoted);
                }
            }
        }
    }

    /// Low funds and low power (claim 28).
    fn house_announcements(&mut self, w: &World) {
        let Some(local) = self.local else { return };
        let r = w.rules();
        let tick = w.tick;
        let owns = |f: &dyn Fn(u32) -> bool| {
            w.objects
                .values()
                .any(|o| o.owner == Some(local) && o.kind == Kind::Building && f(o.type_index))
        };
        // spec: S-0701 claim 28 (low funds: available credits below 100, the countdown expired, and an infantry, vehicle, building or naval factory owned)
        let low = production::available_credits(w, local) < LOW_FUNDS;
        if low
            && self.low_funds_after.is_none_or(|t| tick >= t)
            && owns(&|i| {
                r.buildings.get(i as usize).is_some_and(|b| {
                    matches!(
                        b.factory,
                        Some(FactoryKind::Infantry | FactoryKind::Vehicle | FactoryKind::Building)
                    )
                })
            })
        {
            self.announce(Announcement::LowFunds);
            let b = match self.names.speak_delay {
                // spec: S-0701 claim 28 (the countdown restarts at N(⌊k × 900⌋) ticks)
                Some(k) => k
                    .mul_rz(
                        &glasnost_ini::binfloat::Binary::from_i64(
                            glasnost_ini::binfloat::Format::Double,
                            SPEAK_DELAY_TICKS,
                        ),
                        glasnost_ini::binfloat::Format::Double,
                    )
                    .trunc_to_i64()
                    .unwrap_or(0)
                    .clamp(0, i64::from(i32::MAX)) as i32,
                // spec: S-0701 claim 38 (no SpeakDelay in any layer: initially 2 game minutes, so N(1800) ticks)
                None => SPEAK_DELAY_INITIAL_TICKS,
            };
            let n = glasnost_sim::infantry::normalised(b, w.game_speed);
            self.low_funds_after = Some(tick.saturating_add(n));
        }
        // spec: S-0701 claim 28 (low power: output below a positive drain, one of the first three construction-building types owned; once per episode)
        let lp = power::low_power(w, local);
        if lp && !self.low_power {
            let bc = self.names.build_const.clone();
            if owns(&|i| bc.contains(&i)) {
                self.low_power = true;
                self.announce(Announcement::LowPower);
            }
        } else if !lp {
            self.low_power = false;
        }
    }

    /// A selection response (claim 17): `id` became selected by the local
    /// player's input, the first object of that input to do so.
    pub fn selected(&mut self, w: &World, id: ObjectId, unsync: &mut Rng) {
        let Some(o) = w.obj(id) else { return };
        let ts = self.sounds(TypeRef {
            kind: o.kind,
            index: o.type_index,
        });
        // Our engine has no enslaved or deactivated objects yet, so the
        // normal list always applies (claim 17).
        let own = o.owner.is_some() && o.owner == self.local;
        let passive = o
            .owner
            .is_some_and(|h| glasnost_sim::econ::setup::passive(w, h));
        if own {
            // spec: S-0701 claims 17 and 31 (one raw value, entry x mod n; an empty list takes none)
            let e = self.choose(&ts.voice_select, unsync);
            // spec: S-0701 claim 16 (only the local player's objects respond)
            self.respond(id, e);
        } else if passive {
            // spec: S-0701 claim 17 (a passive house's object takes the raw value but gives no response)
            let _ = self.choose(&ts.voice_select, unsync);
        }
        // spec: S-0701 claim 17 (an ordinary other house's object, enemy or allied, takes no draw and gives no response)
    }

    /// The order response of a battlefield click (claim 18, F6).
    /// `selection` is the selection in the order its objects were selected.
    pub fn ordered(
        &mut self,
        w: &World,
        selection: &[ObjectId],
        click: &ClickResult,
        unsync: &mut Rng,
    ) {
        // spec: S-0701 claim 18 (the first object in selection order: positive-primary-damage objects go to the front when selected, others to the end; so the most recently selected such object, else the earliest selected; no order to it, no response)
        let positive = |id: ObjectId| {
            veterancy::type_combat(w, id)
                .and_then(|tc| tc.slots[0])
                .is_some_and(|wid| w.rules().combat.weapon(wid).damage > 0)
        };
        let Some(id) = selection
            .iter()
            .rev()
            .copied()
            .find(|&id| positive(id))
            .or_else(|| selection.first().copied())
        else {
            return;
        };
        let Some(o) = w.obj(id) else { return };
        // spec: S-0701 claim 16 (only the local player's objects respond)
        if o.owner.is_none() || o.owner != self.local {
            return;
        }
        let Some(order) = click.orders.iter().find(|x| actor(x) == Some(id)) else {
            // spec: S-0701 claim 18 (no other object responds instead)
            return;
        };
        let ts = self.sounds(TypeRef {
            kind: o.kind,
            index: o.type_index,
        });
        let e = match order {
            // spec: S-0701 F6 (move, attack move: the move list)
            Order::Behaviour {
                attack_move: true, ..
            }
            | Order::Behaviour {
                behaviour: Behaviour::Move | Behaviour::AttackMove,
                ..
            } => self.choose(&ts.voice_move, unsync),
            Order::Behaviour {
                behaviour: Behaviour::Attack,
                target,
                ..
            } => {
                // spec: S-0701 F6 (attack by the repair-response vehicle with a weapon of negative damage: the game-wide repair-vehicle response, then the attack list)
                let heal = ts.repair_vehicle
                    && weapon::would_use(w, id, *target)
                        .1
                        .is_some_and(|x| w.rules().combat.weapon(x).damage < 0);
                let single = if heal {
                    self.single(self.names.voice_ifv_repair.as_ref())
                } else {
                    // spec: S-0701 F6 (the single response of the weapon chosen against the target: weapon 0 primary, any other secondary; elite or not)
                    let slot = weapon::would_use(w, id, *target).0;
                    let elite = veterancy::rank(w, id) == Rank::Elite;
                    match (slot, elite) {
                        // spec: S-0701 Import mapping (`VoiceSecondaryWeaponAttack`, the normal secondary-weapon response)
                        (1.., false) => self.single(ts.voice_secondary_attack.as_ref()),
                        // spec: S-0701 Import mapping (`Voice` + `SecondaryEliteWeaponAttack`)
                        (1.., true) => self.single(ts.voice_secondary_elite_attack.as_ref()),
                        // spec: S-0701 Import mapping (`Voice` + `PrimaryWeaponAttack`)
                        (0, false) => self.single(ts.voice_primary_attack.as_ref()),
                        // spec: S-0701 Import mapping (`Voice` + `PrimaryEliteWeaponAttack`)
                        (0, true) => self.single(ts.voice_primary_elite_attack.as_ref()),
                    }
                };
                // spec: S-0701 F6 (then the attack list)
                single.or_else(|| self.choose(&ts.voice_attack, unsync))
            }
            // spec: S-0701 F6 (enter: enter response, then the move list)
            Order::Behaviour {
                behaviour: Behaviour::Enter,
                ..
            } => self
                .single(ts.voice_enter.as_ref())
                .or_else(|| self.choose(&ts.voice_move, unsync)),
            // spec: S-0701 F6 (capture: capture response, then enter response, then the move list)
            Order::Behaviour {
                behaviour: Behaviour::Capture,
                ..
            } => self
                .single(ts.voice_capture.as_ref())
                .or_else(|| self.single(ts.voice_enter.as_ref()))
                .or_else(|| self.choose(&ts.voice_move, unsync)),
            // spec: S-0701 F6 (harvest: harvest response, then the move list)
            Order::Behaviour {
                behaviour: Behaviour::Harvest,
                ..
            } => self
                .single(ts.voice_harvest.as_ref())
                .or_else(|| self.choose(&ts.voice_move, unsync)),
            // spec: S-0701 F6 (unload, deploy: deployed infantry's undeploy response, otherwise the deploy response; no fallback)
            Order::Behaviour {
                behaviour: Behaviour::Unload,
                ..
            }
            | Order::Deploy { .. } => {
                if o.kind == Kind::Infantry && deployed_infantry(o) {
                    self.single(ts.voice_undeploy.as_ref())
                } else {
                    self.single(ts.voice_deploy.as_ref())
                }
            }
            // spec: S-0701 F6 (any other order: the special-attack list)
            _ => self.choose(&ts.voice_special_attack, unsync),
        };
        self.respond(id, e);
    }

    /// A key command of the local player that issued orders (claim 19).
    pub fn key(&mut self, cmd: KeyCommand) {
        let s = match cmd {
            // spec: S-0701 claim 19 (guard, stop and scatter: their game-wide interface sounds, screen sounds)
            KeyCommand::Guard => self.names.guard.clone(),
            KeyCommand::Stop => self.names.stop.clone(),
            KeyCommand::Scatter => self.names.scatter.clone(),
            // spec: S-0701 claim 19 and Edge cases (quirks.deploy_command_silent: the deploy command is silent; off: `DeploySound`)
            KeyCommand::Deploy if !self.quirks.deploy_command_silent => {
                self.names.deploy_command.clone()
            }
            _ => None,
        };
        self.play_named(s.as_deref(), Where::Screen);
    }

    /// Plays an interface cue for the local player (claims 23, 24 and 28).
    pub fn cue(&mut self, cue: UiCue) -> Option<InstanceId> {
        let screen = |a: &mut Self, s: Option<String>| a.play_named(s.as_deref(), Where::Screen);
        match cue {
            UiCue::CreditTick { rising } => {
                let t = &self.names.credit_ticks;
                // spec: S-0416 claim 4 (fewer than two entries: no tick sound at all)
                if t.len() < 2 {
                    return None;
                }
                // spec: S-0416 claim 4 (first entry rising, second falling)
                let name = t[usize::from(!rising)].clone();
                let id = self.catalog.resolve(&name)?;
                // spec: S-0416 claim 4 (volume one half: the request level of a screen sound, F3)
                self.request(None, id, Where::Screen, CREDIT_TICK_LEVEL)
            }
            // spec: S-0410 claim 3 (the radar-on and radar-off sounds)
            UiCue::RadarOn => screen(self, self.names.radar_on.clone()),
            UiCue::RadarOff => screen(self, self.names.radar_off.clone()),
            // spec: S-0701 claim 28 (construction complete; no throttle)
            UiCue::ConstructionComplete => {
                self.announce(Announcement::ConstructionComplete);
                None
            }
            // spec: S-0701 claim 24 (the production-feedback clicks play the game-wide build click; their announcements, claim 28)
            UiCue::BuildStarted { infantry } | UiCue::BuildResumed { infantry } => {
                // spec: S-0701 claim 28 and S-0411 claim 12 (start and resume: training for infantry, building for every other kind)
                self.announce(if infantry {
                    Announcement::Training
                } else {
                    Announcement::Building
                });
                screen(self, self.names.gui_build.clone())
            }
            // spec: S-0701 claim 24 (all five production clicks: the build click; adding a queued copy to an active line requests no new start announcement), claim 28 and S-0411 claim 12
            UiCue::BuildQueued => screen(self, self.names.gui_build.clone()),
            UiCue::BuildHeld => {
                self.announce(Announcement::OnHold);
                screen(self, self.names.gui_build.clone())
            }
            UiCue::BuildCancelled => {
                self.announce(Announcement::Cancelled);
                screen(self, self.names.gui_build.clone())
            }
            // spec: S-0701 claim 24 (a busy production line: only the unable-to-comply announcement, no interface sound)
            UiCue::BuildRefused => {
                self.announce(Announcement::UnableToComply);
                None
            }
            // spec: S-0701 claim 24 (a refused placement: only the cannot-deploy-here announcement, no interface sound)
            UiCue::PlacementRefused => {
                self.announce(Announcement::CannotDeploy);
                None
            }
            // spec: S-0701 claim 24 (the sidebar strip scrolled past its end: the game-wide refusal sound)
            UiCue::StripEnd => screen(self, self.names.scold.clone()),
            // spec: S-0701 claim 24 and S-0411 claim 18 (an accepted page request: the tab-scroll sound)
            UiCue::StripScroll => screen(self, self.names.gui_tab.clone()),
            // spec: S-0701 claim 23 (the game-wide building placement sound, a screen sound)
            UiCue::BuildingPlaced => screen(self, self.names.building_slam.clone()),
            // spec: S-0701 claim 28 (rally point set: the local player's matching action; no throttle)
            UiCue::RallySet => {
                self.announce(Announcement::RallySet);
                None
            }
        }
    }
}

fn cell_of(p: Pos) -> (i32, i32) {
    let c = p.cell();
    (c.x, c.y)
}

/// An infantry unit in a deployed state (S-0701 F6: its unload or deploy
/// order is an undeploy).
fn deployed_infantry(o: &glasnost_sim::world::Object) -> bool {
    use glasnost_sim::infantry::Seq;
    o.infantry
        .as_ref()
        .and_then(|s| s.seq)
        .is_some_and(|q| matches!(q, Seq::Deployed | Seq::DeployedFire | Seq::DeployedIdle))
}

fn rank_value(r: Rank) -> u8 {
    match r {
        Rank::Rookie => 0,
        Rank::Veteran => 1,
        Rank::Elite => 2,
    }
}

/// The actor of an object order.
fn actor(o: &Order) -> Option<ObjectId> {
    match o {
        Order::Behaviour { actor, .. }
        | Order::Stop { actor }
        | Order::Scatter { actor }
        | Order::Deploy { actor }
        | Order::GuardSpot { actor, .. }
        | Order::Sell { actor }
        | Order::Repair { actor }
        | Order::PowerOff { actor }
        | Order::PowerOn { actor }
        | Order::Primary { actor } => Some(*actor),
        _ => None,
    }
}

#[cfg(test)]
#[path = "events_tests.rs"]
mod tests;
