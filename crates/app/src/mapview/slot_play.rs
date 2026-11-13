//! The slot animations of the live game's buildings: which slots run, as
//! the building's state starts and stops them (S-0108 Terms and claims 11,
//! 12, 15, 17 and 20; S-0606 claims 6, 12 and 16 through the simulation's
//! `Effect::SpecialAnims`). Each running slot animation is played by
//! S-0109's playback, which the caller supplies through [`Make`] and drives
//! with [`Slots::finished`].
//!
//! This is presentation state the simulation never reads. A repair
//! depot's *production* and *special-1* to *special-3* slots are the
//! simulation's (`glasnost_sim::depot_anim`, S-0108 claim 20: their
//! lifetime is game state); the drawing shows them from there and plays
//! the depot's *idle* and *active-1* as the simulation's
//! `Effect::DepotSlot` reports ([`Slots::depot`]).
//!
//! The power rules of claim 13 are played for these slots: a
//! *freeze-unpowered* slot is frozen (held, S-0109 claim 23) by a loss of
//! operation of a type that needs and draws power, by a disabling (a
//! switch-off, among them the map's placement of a building switched off,
//! claim 28; a pulse's start; and the map's placement of a building that
//! needs an engineer and is not manned, claim 26), and when it starts
//! frozen; it resumes at a power return or a re-enabling (a switch-on, a
//! pulse's end, and an ownership change of a building whose type needs an
//! engineer, claim 26; an ownership change of any other building is
//! neither). The *light-unpowered* and *effect-unpowered* columns stop and
//! start slots. The superweapon slots *super-1* to *super-4* and
//! *super-low-power* are played: the grand opening's start (claim 11), the
//! charge cycle (claim 18) and the power rows (claim 13). Not covered here:
//! claim 14's special outage and the *low-power* slot, silo fill (claim
//! 16), the turret slot (claims 11 and 19), the upgrade slots and the
//! recycling building's row of claim 20.

use std::collections::{BTreeMap, BTreeSet};

use super::slots::{self, Counting, SlotDef};

/// The slots this module starts or stops (S-0108 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slot {
    /// *idle*.
    Idle,
    /// *active-1*.
    Active1,
    /// *active-2*.
    Active2,
    /// *active-3*.
    Active3,
    /// *active-4*.
    Active4,
    /// *pre-production*: started only by the simulation (a docking
    /// animation, S-0402 claim 11).
    PreProduction,
    /// *production*.
    Production,
    /// *special-1*.
    Special1,
    /// *special-2*.
    Special2,
    /// *special-3*.
    Special3,
    /// *special-4*.
    Special4,
    /// *super-1*: charging.
    Super1,
    /// *super-2*: opening.
    Super2,
    /// *super-3*: charged.
    Super3,
    /// *super-4*: closing.
    Super4,
    /// *super-low-power*.
    SuperLowPower,
}

impl Slot {
    /// This module's slots in S-0108 claim 12's restart order: *active-1*
    /// to *active-4*, *pre-production*, *production*, *special-1* to
    /// *special-4*, *super-1* to *super-4*, *idle*, then *super-low-power*
    /// (the upgrade, turret and low-power slots of that order are not
    /// played here).
    // spec: S-0108 claim 12 (the running slots are restarted one after another in the fixed order upgrade-1 to upgrade-3, active-1 to active-4, pre-production, production, turret, special-1 to special-4, super-1 to super-4, idle, low-power, super-low-power)
    pub const RESTART: [Slot; 16] = [
        Slot::Active1,
        Slot::Active2,
        Slot::Active3,
        Slot::Active4,
        Slot::PreProduction,
        Slot::Production,
        Slot::Special1,
        Slot::Special2,
        Slot::Special3,
        Slot::Special4,
        Slot::Super1,
        Slot::Super2,
        Slot::Super3,
        Slot::Super4,
        Slot::Idle,
        Slot::SuperLowPower,
    ];

    /// Every slot of this module, in our own order (also the drawing
    /// order of a building's slot animations: our placeholder).
    pub const ALL: [Slot; 16] = [
        Slot::Idle,
        Slot::Active1,
        Slot::Active2,
        Slot::Active3,
        Slot::Active4,
        Slot::PreProduction,
        Slot::Production,
        Slot::Special1,
        Slot::Special2,
        Slot::Special3,
        Slot::Special4,
        Slot::Super1,
        Slot::Super2,
        Slot::Super3,
        Slot::Super4,
        Slot::SuperLowPower,
    ];

    /// The slot's base art key.
    pub fn base(self) -> &'static [u8] {
        // spec: S-0108 Import mapping (slot base keys)
        match self {
            Slot::Idle => b"IdleAnim",
            Slot::Active1 => b"ActiveAnim",
            Slot::Active2 => b"ActiveAnimTwo",
            Slot::Active3 => b"ActiveAnimThree",
            Slot::Active4 => b"ActiveAnimFour",
            Slot::PreProduction => b"PreProductionAnim",
            Slot::Production => b"ProductionAnim",
            Slot::Special1 => b"SpecialAnim",
            Slot::Special2 => b"SpecialAnimTwo",
            Slot::Special3 => b"SpecialAnimThree",
            Slot::Special4 => b"SpecialAnimFour",
            Slot::Super1 => b"SuperAnim",
            Slot::Super2 => b"SuperAnimTwo",
            Slot::Super3 => b"SuperAnimThree",
            Slot::Super4 => b"SuperAnimFour",
            Slot::SuperLowPower => b"SuperLowPower",
        }
    }

    /// *special-n* for n from 1 to 4 (S-0606 Terms: the walls' slots).
    pub fn special(n: u8) -> Option<Slot> {
        match n {
            1 => Some(Slot::Special1),
            2 => Some(Slot::Special2),
            3 => Some(Slot::Special3),
            4 => Some(Slot::Special4),
            _ => None,
        }
    }
}

/// A type's slot definitions, by slot.
pub type Defs = BTreeMap<Slot, SlotDef>;

/// Reads a building type's slot definitions from its art section `image`
/// (S-0108 Import mapping).
pub fn read_defs(art: &glasnost_ini::Document, image: &[u8]) -> Defs {
    Slot::ALL
        .iter()
        .map(|&s| (s, slots::read_slot(art, image, s.base())))
        .collect()
}

/// What a building type's slots react to (S-0108 Terms and Import
/// mapping).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TypeKind {
    /// The refinery flag (claims 11 and 15).
    pub refinery: bool,
    /// Absorbs infantry and gives a positive extra power per held
    /// infantry (claim 17).
    pub absorbs: bool,
    /// The unit-repair flag: a repair depot (claim 20), whose special
    /// slots the simulation plays.
    pub depot: bool,
    /// A delayed-fire type (claim 20).
    pub delayed_fire: bool,
    /// The type needs power (claim 13: only then does a
    /// *freeze-unpowered* slot start frozen).
    pub needs_power: bool,
    /// The type needs and draws power (Terms): claim 13's power-loss and
    /// power-return columns apply.
    pub draws_power: bool,
    /// The type needs an engineer (claim 26): only then is an ownership
    /// change a re-enabling (claim 13).
    pub needs_engineer: bool,
    /// The type's first superweapon field names a superweapon (claims 11
    /// and 18: its first superweapon only).
    pub superweapon: bool,
    /// The type's charged-animation time *t_c* in game minutes, the
    /// single-precision field widened exactly to binary64 (claim 18, F6);
    /// `None` when the data does not set the key.
    pub charged_anim_time: Option<glasnost_num::F64>,
}

/// The grand opening's "charged" test: a remaining charge below this many
/// ticks starts *super-3* (S-0108 claim 11).
pub const CHARGED_TICKS: i32 = 15; // spec: S-0108 Constants (grand-opening "charged" test: 15 ticks)

/// The charge cycle runs only for a charged-animation time of at most this
/// many game minutes (S-0108 claim 18).
pub const CHARGE_CYCLE_CUTOFF: i64 = 990; // spec: S-0108 claim 18 (the check runs only when t_c is at most 990)

/// The charged-animation time when the data does not set the key, in game
/// minutes (S-0108 Import mapping).
pub const CHARGED_ANIM_TIME_DEFAULT: i64 = 999; // spec: S-0108 Import mapping (ChargedAnimTime absent: 999)

/// S-0108 F6: the remaining charge *t_r* in game minutes for `r` remaining
/// ticks, r × κ formed in binary64 under rounding `mode`, with κ =
/// 2 386 093 / 2³¹.
pub fn charge_minutes(r: i32, mode: glasnost_num::Rounding) -> glasnost_num::F64 {
    use glasnost_num::F64;
    // spec: S-0108 F6 and Constants (κ = 2 386 093 / 2³¹, the single-precision value nearest 0.0011111111)
    let kappa = F64::from_parts(false, 2_386_093, -31, mode);
    // spec: S-0108 F6 (the product formed in binary64, rounded by the program-wide mode)
    F64::from_i64(i64::from(r), mode).mul(kappa, mode)
}

/// A building's state as the slot rules read it, each tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// It has completed and is not in the *construction* state (claims 11
    /// and 17).
    pub completed: bool,
    /// It was placed during the game's set-up (claim 11).
    pub setup: bool,
    /// Damaged (S-0108 Terms, F3).
    pub damaged: bool,
    /// Occupied: at least one garrison occupant (S-0108 Terms).
    pub occupied: bool,
    /// The held infantry's total size *T* (claim 17, F9).
    pub held_size: i32,
    /// The owner, as a house index (claim 26: every ownership change of a
    /// building whose type needs an engineer re-enables it).
    pub owner: Option<u8>,
    /// The in-operation result of S-0405 claim 8: the powered state of
    /// claim 13 (Terms, claim 28).
    pub in_operation: bool,
    /// Its power switch is off (claim 28's switched-on row).
    pub switched_off: bool,
    /// It is under a pulse (claim 28's not-under-a-pulse row).
    pub pulsed: bool,
    /// Its ownership change of this tick, if any, was made in the turn of an
    /// object that stands after the building's slots (an engineer that
    /// entered play after them), so it comes after their turns; `false` for
    /// a change made before the turn pass or by an object before the slots
    /// (claim 26, Q-2186).
    pub change_after_slots: bool,
    /// Its type needs an engineer and it is not manned (claim 26).
    pub unmanned: bool,
    /// It is being sold (claim 18: the charge cycle is not checked then).
    pub selling: bool,
    /// The remaining charge in ticks of its owner's record of its type's
    /// first superweapon (S-0512 F2), read whether or not the record is
    /// granted with `quirks.ungranted_super_reads_clock` on and only when it
    /// is granted with the switch off (claims 11 and 18); `None` when there
    /// is no record to read.
    pub charge_remaining: Option<i32>,
    /// That remaining charge in game minutes (F6).
    pub charge_minutes: Option<glasnost_num::F64>,
}

/// Makes a slot's animation of a type name.
pub trait Make<A> {
    /// A new animation of type `name`, keeping the stage of `from` when it
    /// replaces one, with its own passes (S-0108 claim 25); `None` when
    /// the name names no animation type.
    fn make(&mut self, name: &[u8], from: Option<&A>) -> Option<A>;
}

/// A running slot animation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Running<A> {
    /// The animation name it was started with.
    pub name: Vec<u8>,
    /// Its playback.
    pub anim: A,
    /// Frozen by claim 13: it holds its image frame (S-0109 claim 23).
    pub frozen: bool,
}

/// What a building's last update saw of the causes that disable and
/// re-enable it (claims 13, 26 and 28).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Seen {
    owner: Option<u8>,
    switched_off: bool,
    pulsed: bool,
}

impl Seen {
    fn of(st: &State) -> Self {
        Self {
            owner: st.owner,
            switched_off: st.switched_off,
            pulsed: st.pulsed,
        }
    }
}

/// One building's slots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slots<A> {
    /// The running slot animations.
    pub running: BTreeMap<Slot, Running<A>>,
    /// The slot condition is *damaged* (S-0108 Terms).
    condition_damaged: bool,
    /// The grand opening has been run (claim 11).
    opened: bool,
    /// Disabled in claim 13's sense: by a switch-off (the map's placement
    /// switched off among them), a pulse or, for a building that needs an
    /// engineer, the map's placement while it is not manned (claims 26 and
    /// 28); cleared by a re-enabling.
    disabled: bool,
    /// What the last update saw (claims 13, 26 and 28); `None` before the
    /// first.
    seen: Option<Seen>,
    /// The powered state (the in-operation result) the last update saw
    /// since the grand opening (claim 13); `None` before it.
    powered: Option<bool>,
    /// The slots stopped by the outage (claim 13, *effect-unpowered*).
    outage: BTreeSet<Slot>,
    /// The type needs power, as of the tick's [`Slots::prepare`].
    needs_power: bool,
    /// The building is in operation, as of the tick's [`Slots::prepare`].
    in_operation: bool,
    /// `quirks.enable_unfreezes_unpowered`, as of the tick's
    /// [`Slots::prepare`].
    enable_unfreezes: bool,
    /// The tick's [`Slots::prepare`] has applied the changes that come
    /// before the slots play; [`Slots::update`] applies the rest.
    early_done: bool,
}

impl<A> Default for Slots<A> {
    fn default() -> Self {
        Self {
            running: BTreeMap::new(),
            condition_damaged: false,
            opened: false,
            disabled: false,
            seen: None,
            powered: None,
            outage: BTreeSet::new(),
            needs_power: false,
            in_operation: true,
            // spec: S-0108 Edge cases (quirks.enable_unfreezes_unpowered defaults on)
            enable_unfreezes: true,
            early_done: false,
        }
    }
}

impl<A> Slots<A> {
    /// Whether `slot` holds a running animation.
    pub fn runs(&self, slot: Slot) -> bool {
        self.running.contains_key(&slot)
    }

    /// Whether `slot` holds a running animation that is frozen (claim 13).
    pub fn frozen(&self, slot: Slot) -> bool {
        self.running.get(&slot).is_some_and(|r| r.frozen)
    }

    /// Whether the building is disabled in claim 13's sense (tests).
    pub fn disabled(&self) -> bool {
        self.disabled
    }

    /// Takes in the building's state at the start of its turn in a tick,
    /// before its slots play and before any start: whether its type needs
    /// power, whether it is in operation, and
    /// `quirks.enable_unfreezes_unpowered` as `enable_unfreezes`. Then the
    /// re-enablings that come before the slots' turns in the tick act
    /// ([`Slots::early_enablings`]).
    pub fn prepare(&mut self, kind: &TypeKind, st: &State, enable_unfreezes: bool) {
        self.needs_power = kind.needs_power;
        self.in_operation = st.in_operation;
        self.enable_unfreezes = enable_unfreezes;
        if let Some(last) = self.seen {
            self.early_enablings(kind, st, last);
        }
        self.early_done = true;
    }

    /// Whether a slot of definition `def` started now starts frozen.
    fn starts_frozen(&self, def: &SlotDef) -> bool {
        // spec: S-0108 claim 13 (a freeze-unpowered slot that starts while the building is disabled or not in operation, any row of S-0405 claim 8, starts frozen; only when the type itself needs power) and claim 26 (slot starts while not manned)
        def.power.counting() == Counting::Freeze
            && self.needs_power
            && (self.disabled || !self.in_operation)
    }

    /// A disabling (claim 13): every running *freeze-unpowered* slot
    /// freezes, whether or not the type needs power.
    fn disable(&mut self, defs: &Defs) {
        self.disabled = true;
        for (slot, r) in &mut self.running {
            // spec: S-0108 claim 13 (disabling a building freezes every running freeze-unpowered slot, whether or not the type needs power)
            if defs
                .get(slot)
                .is_some_and(|d| d.power.counting() == Counting::Freeze)
            {
                r.frozen = true;
            }
        }
    }

    /// A re-enabling (claims 13 and 26): the frozen slots resume, even
    /// when the building is not in operation
    /// (`quirks.enable_unfreezes_unpowered`; off: only when it is in
    /// operation right after it).
    fn enable(&mut self, st: &State) {
        self.disabled = false;
        // spec: S-0108 claim 13 and Edge cases (quirks.enable_unfreezes_unpowered: re-enabling unfreezes the freeze-unpowered slots even when the building has no power; off: only when it is in operation right after it)
        if self.enable_unfreezes || st.in_operation {
            for r in self.running.values_mut() {
                // spec: S-0108 claim 26 (each frozen slot resumes from the image frame it held: no slot is started, stopped or restarted, so it keeps its variant)
                r.frozen = false;
            }
        }
    }

    /// The re-enablings since the last update that come before the slots'
    /// turns in the tick, and so before the tick's advances (claim 13,
    /// S-0203; S-0109 claims 5 and 23: a released slot advances at its first
    /// eligible update): an ownership change by a trigger, which acts before
    /// any object's turn, and a pulse's end.
    fn early_enablings(&mut self, kind: &TypeKind, st: &State, last: Seen) {
        // spec: S-0108 claim 26 (as part of every ownership change of a building whose type needs an engineer it is re-enabled, whether or not it is in operation after the change) and claim 13 (an ownership change of any other building is neither a disabling nor a re-enabling; a polled trigger's action acts before any object's turn, S-0203 row 1) and claim 28 (an ownership change while switched off: for a type that does not need an engineer the held slots stay held)
        // measure: M-0014 (b22n-tm1: the trigger's change of owner at 300 re-enables W7 and its held slots advance 0 to 1 in 300)
        if last.owner != st.owner && kind.needs_engineer && !st.change_after_slots {
            self.enable(st);
        }
        // spec: S-0108 claims 13 and 28 (a pulse's end re-enables the building whatever its power switch; it acts before the orders of its tick)
        // measure: M-0014 (e: in the tick in which the pulse's hold reaches 0, 289, W8 is enabled again and its held slots advance 2 to 3 in that tick; b22o-ep1: a switch-off order applied after that tick's turns holds them at 3)
        if last.pulsed && !st.pulsed {
            self.enable(st);
        }
    }

    /// The disablings and re-enablings since the last update that come
    /// after the slots' turns: an ownership change in the turn pass (a
    /// capture), a pulse's start, then the switch, each acting at once, in
    /// their places in the tick (claim 13, S-0203). The building's own
    /// reaction to a change of its in-operation result follows them
    /// ([`Slots::power`]).
    fn enablings(&mut self, kind: &TypeKind, defs: &Defs, st: &State, last: Seen) {
        // spec: S-0108 claim 26 (Q-2186: a change made in the turn of an object that stands after the building's slots, such as an engineer that entered play after them, acts after the slots' turns of its tick, so the first advance is in the next tick; the simulation says which in `Effect::Captured::after_slots`, and a change made before the turn pass or by an object before the slots acts early, in `early_enablings`)
        // measure: M-0014 (the captures at 133 and 134: the first advance in the next tick, an engineer that stands after the slots)
        // spec: S-0108 claims 13 and 28 (a pulse's start acts in the turn pass, after the slots' turns); pulses do not reach the simulation yet (`econ::power::under_pulse`)
        // spec: S-0108 claim 26 (as part of every ownership change of a building whose type needs an engineer it is re-enabled, whether or not it is in operation after the change)
        if last.owner != st.owner && kind.needs_engineer && st.change_after_slots {
            self.enable(st);
        }
        // spec: S-0108 claims 13 and 28 (a pulse's start disables the building, whatever its power switch)
        if !last.pulsed && st.pulsed {
            self.disable(defs);
        }
        // spec: S-0108 claim 13 (a power order acts after the turn pass, S-0203 row 8 and claim 7, so after an ownership change and a pulse's start or end of its tick) and claim 28 (switched-on row: switching off disables the building; switching on re-enables it, whether or not it is then in operation)
        // measure: M-0014 (an order applied before tick F's logic, the cycle of F - 1, holds the advance due in F: b22n-tm1 at 299, b22o-ep3 at 292, b22o-ep6 at 295)
        match (last.switched_off, st.switched_off) {
            (false, true) => self.disable(defs),
            (true, false) => self.enable(st),
            _ => {}
        }
    }

    /// The slots whose power rules this module plays for a type: all of
    /// them except a repair depot's *production* and *special-1* to
    /// *special-3*, which are the simulation's (claim 20).
    fn power_slots(kind: &TypeKind) -> Vec<Slot> {
        Slot::ALL
            .into_iter()
            .filter(|s| {
                !(kind.depot
                    && matches!(
                        s,
                        Slot::Production | Slot::Special1 | Slot::Special2 | Slot::Special3
                    ))
            })
            .collect()
    }

    /// Claim 13's power-loss column.
    fn power_lost(&mut self, kind: &TypeKind, defs: &Defs, st: &State, mk: &mut dyn Make<A>) {
        // design: the column acts on the slots running at the loss; a slot it starts itself (active-1 of a delayed-fire type, super-low-power) is not then put through the column again
        let was_running: BTreeSet<Slot> = self.running.keys().copied().collect();
        for slot in Self::power_slots(kind) {
            let Some(def) = defs.get(&slot) else { continue };
            if !self.runs(slot) || !was_running.contains(&slot) {
                continue;
            }
            match def.power.counting() {
                Counting::Freeze => {
                    if let Some(r) = self.running.get_mut(&slot) {
                        // spec: S-0108 claim 13 (freeze-unpowered, power lost: a running slot holds its image frame and stops advancing)
                        r.frozen = true;
                    }
                }
                Counting::Light => {
                    // spec: S-0108 claim 13 (light-unpowered, power lost: a running slot is stopped)
                    self.stop(slot);
                    // spec: S-0108 claim 13 (special-1 of a delayed-fire type whose active-1 has freeze-unpowered: active-1 starts, and being unpowered starts frozen; normal or damaged variant)
                    if slot == Slot::Special1
                        && kind.delayed_fire
                        && defs
                            .get(&Slot::Active1)
                            .is_some_and(|d| d.power.counting() == Counting::Freeze)
                    {
                        self.start_variant(Slot::Active1, defs, st.damaged, mk);
                    }
                }
                Counting::Effect => {
                    // spec: S-0108 claim 13 (effect-unpowered, power lost: a running slot is stopped and marked as stopped by the outage)
                    self.stop(slot);
                    self.outage.insert(slot);
                    // spec: S-0108 claim 13 (if it is super-3, super-low-power starts; normal or damaged variant)
                    if slot == Slot::Super3 {
                        self.start_variant(Slot::SuperLowPower, defs, st.damaged, mk);
                    }
                }
                Counting::Nothing => {}
            }
        }
    }

    /// Claim 13's power-return column.
    fn power_back(&mut self, kind: &TypeKind, defs: &Defs, st: &State, mk: &mut dyn Make<A>) {
        // spec: S-0108 claim 13 (any counting flag, power comes back: super-low-power stops)
        self.stop(Slot::SuperLowPower);
        for slot in Self::power_slots(kind) {
            // design: super-low-power, stopped by the row for any flag, is not started again by its own flag's row at the same return (claim 13 gives no order between the rows; ours lets the stop stand)
            if slot == Slot::SuperLowPower {
                self.outage.remove(&slot);
                continue;
            }
            let Some(def) = defs.get(&slot) else { continue };
            match def.power.counting() {
                Counting::Freeze => {
                    if let Some(r) = self.running.get_mut(&slot) {
                        // spec: S-0108 claim 13 (freeze-unpowered, power comes back: it resumes)
                        r.frozen = false;
                    }
                }
                Counting::Light => {
                    // spec: S-0108 claim 13 (light-unpowered, power comes back: the slot starts if it is not running and the building has completed; not the special-1 slot of a delayed-fire type; normal or damaged variant)
                    if !self.runs(slot)
                        && st.completed
                        && !(slot == Slot::Special1 && kind.delayed_fire)
                    {
                        self.start_variant(slot, defs, st.damaged, mk);
                    }
                }
                Counting::Effect => {
                    // spec: S-0108 claim 13 (effect-unpowered, power comes back: marked slots start and are unmarked; normal or damaged variant)
                    if self.outage.remove(&slot) {
                        self.start_variant(slot, defs, st.damaged, mk);
                    }
                }
                Counting::Nothing => {}
            }
        }
    }

    /// Claim 13's reaction to a change of the powered state since the last
    /// update, once to each change, for a type that needs and draws power.
    fn power(&mut self, kind: &TypeKind, defs: &Defs, st: &State, mk: &mut dyn Make<A>) {
        let was = self.powered.replace(st.in_operation);
        // spec: S-0108 claim 13 (both columns apply only to a building that needs and draws power)
        if !kind.draws_power {
            return;
        }
        // spec: S-0108 claim 13 (a building reacts to a change of its in-operation result in its next update, once to each change) and claim 27 (a building not powered since it was placed or completed has had no change from powered to unpowered)
        match (was, st.in_operation) {
            (Some(true), false) => self.power_lost(kind, defs, st, mk),
            (Some(false), true) => self.power_back(kind, defs, st, mk),
            _ => {}
        }
    }

    /// The names of the running slots, in slot order (tests and lanes).
    pub fn names(&self) -> Vec<(Slot, Vec<u8>)> {
        self.running
            .iter()
            .map(|(s, r)| (*s, r.name.clone()))
            .collect()
    }

    /// Starts `slot` in condition `damaged`, with the occupied variant when
    /// `occupied_variant` (S-0108 Terms, *starting a slot*, *variant
    /// choice*; claim 12).
    fn start(
        &mut self,
        slot: Slot,
        defs: &Defs,
        damaged: bool,
        occupied_variant: bool,
        mk: &mut dyn Make<A>,
    ) {
        // spec: S-0108 claim 12 (a start in the condition opposite to the slot condition swaps every
        // running slot first; a start with the occupied variant counts as normal, and the occupied
        // variant is chosen only when not damaged)
        if damaged != self.condition_damaged {
            self.swap(damaged, defs, mk);
        }
        let Some(def) = defs.get(&slot) else {
            return;
        };
        // spec: S-0108 Terms (variant choice: damaged name; occupied name when allowed; else normal)
        let name = slots::variant(def, damaged, occupied_variant, true).to_vec();
        // spec: S-0108 Terms (nothing happens when the chosen name is empty or names no animation type)
        if name.is_empty() {
            return;
        }
        let from = self.running.get(&slot).map(|r| &r.anim);
        // spec: S-0108 Terms and claim 25 (a running slot is replaced; the new one keeps its stage and takes its passes afresh)
        if let Some(anim) = mk.make(&name, from) {
            let frozen = self.starts_frozen(def);
            self.running.insert(slot, Running { name, anim, frozen });
        }
    }

    /// Starts `slot` with the normal or damaged variant (S-0108 claims 13,
    /// 15 and 20; S-0606 claims 6 and 12): the damaged name when
    /// `damaged`.
    pub fn start_variant(&mut self, slot: Slot, defs: &Defs, damaged: bool, mk: &mut dyn Make<A>) {
        self.start(slot, defs, damaged, false, mk);
    }

    /// Starts `slot` with occupied variants allowed (S-0108 claims 11, 17
    /// and 20).
    fn start_occupied(&mut self, slot: Slot, defs: &Defs, st: &State, mk: &mut dyn Make<A>) {
        self.start(slot, defs, st.damaged, st.occupied, mk);
    }

    /// Stops `slot` (S-0108 Terms, *stopping a slot*).
    pub fn stop(&mut self, slot: Slot) {
        self.running.remove(&slot);
    }

    /// The condition swap of S-0108 claim 12: the slot condition becomes
    /// `damaged` and every running slot restarts with the name for it, in
    /// [`Slot::RESTART`]'s order, keeping its stage (claim 25); a slot
    /// whose name for it is empty keeps its animation.
    fn swap(&mut self, damaged: bool, defs: &Defs, mk: &mut dyn Make<A>) {
        self.condition_damaged = damaged;
        // spec: S-0108 claim 12 (the restarts come one after another in the fixed slot order, each taking its creation draws as it is restarted)
        let slots: Vec<Slot> = Slot::RESTART
            .into_iter()
            .filter(|s| self.running.contains_key(s))
            .collect();
        for slot in slots {
            let Some(def) = defs.get(&slot) else {
                continue;
            };
            // spec: S-0108 claim 12 (the damaged name when now damaged, the normal name when not;
            // never the occupied variant, quirks.damage_swap_drops_occupied)
            let name = if damaged { &def.damaged } else { &def.normal };
            // spec: S-0108 claim 12 (a slot whose name for the new condition is empty keeps its animation)
            if name.is_empty() {
                continue;
            }
            let from = self.running.get(&slot).map(|r| &r.anim);
            if let Some(anim) = mk.make(name, from) {
                // spec: S-0108 claim 26 (a restart by the swap is a start: frozen only by claim 13's start rule, so a type that does not need power restarts running and unfrozen although still disabled)
                let frozen = self.starts_frozen(def);
                self.running.insert(
                    slot,
                    Running {
                        name: name.clone(),
                        anim,
                        frozen,
                    },
                );
            }
        }
    }

    /// The grand opening's starts (S-0108 claim 11) for a building
    /// completed in play.
    fn grand_opening(&mut self, kind: &TypeKind, defs: &Defs, st: &State, mk: &mut dyn Make<A>) {
        if st.setup {
            // spec: S-0108 claim 11 (a building placed during the set-up starts idle and active-1 …
            // active-4 when its body state is set, all with occupied variants allowed, then runs the
            // grand opening)
            for slot in [
                Slot::Idle,
                Slot::Active1,
                Slot::Active2,
                Slot::Active3,
                Slot::Active4,
            ] {
                self.start_occupied(slot, defs, st, mk);
            }
        }
        if kind.refinery {
            // spec: S-0108 claim 11 (a refinery: only the active slot of its fill level, no occupied
            // variant) and claim 15 (no stored ore, m = 0: L = 0 selects active-1; no skirmish path
            // fills a building's own storage)
            self.start_variant(Slot::Active1, defs, st.damaged, mk);
        } else {
            // spec: S-0108 claim 11 (idle and active-1 with occupied variants allowed; active-2,
            // active-3 and active-4 with no occupied variant)
            self.start_occupied(Slot::Idle, defs, st, mk);
            self.start_occupied(Slot::Active1, defs, st, mk);
            for slot in [Slot::Active2, Slot::Active3, Slot::Active4] {
                self.start_variant(slot, defs, st.damaged, mk);
            }
        }
        if kind.superweapon {
            // spec: S-0108 claim 11 (a building with a superweapon: super-3 when the weapon's remaining charge time is below 15 ticks, otherwise super-1; both allow occupied variants)
            // spec: S-0108 claim 11 and Edge cases (quirks.ungranted_super_reads_clock: on, a record not granted is read by its clock, never granted reads 0 and gives super-3; off, it counts as not charged and gives super-1; the caller leaves charge_remaining empty then)
            let charged = st.charge_remaining.is_some_and(|r| r < CHARGED_TICKS);
            let slot = if charged { Slot::Super3 } else { Slot::Super1 };
            self.start_occupied(slot, defs, st, mk);
        }
    }

    /// S-0108 claim 18's charge cycle check, for a type with a superweapon
    /// whose charged-animation time is at most 990 game minutes.
    fn charge_cycle(&mut self, kind: &TypeKind, defs: &Defs, st: &State, mk: &mut dyn Make<A>) {
        use glasnost_num::{F64, Rounding};
        // spec: S-0108 claim 18 (a building with a superweapon, its first only; not while it is being built or sold)
        if !kind.superweapon || !st.completed || st.selling {
            return;
        }
        // spec: S-0108 Import mapping (ChargedAnimTime absent: 999, above 990, so claim 18 is off)
        let Some(tc) = kind.charged_anim_time else {
            return;
        };
        // spec: S-0108 claim 18 (the check runs only when t_c is at most 990); 990 is exact in binary64 in either mode
        if !tc.le(F64::from_i64(CHARGE_CYCLE_CUTOFF, Rounding::TowardZero)) {
            return;
        }
        // spec: S-0108 claim 18 (quirks.ungranted_super_reads_clock off: while the record is not granted there is no t_r and the check waits for the grant; on, the caller reads the clock whatever the grant)
        let Some(tr) = st.charge_minutes else {
            return;
        };
        if tc.lt(tr) {
            // spec: S-0108 claim 18 (t_c < t_r and super-3 running: super-3 stops and super-4 starts; normal or damaged variant)
            if self.runs(Slot::Super3) {
                self.stop(Slot::Super3);
                self.start_variant(Slot::Super4, defs, st.damaged, mk);
            }
        } else if self.runs(Slot::Super1) {
            // spec: S-0108 claim 18 and F6 (t_c ≥ t_r, the charged branch, and super-1 running: super-1 stops and super-2 starts; normal or damaged variant)
            self.stop(Slot::Super1);
            self.start_variant(Slot::Super2, defs, st.damaged, mk);
        }
    }

    /// The building's update: its disablings and re-enablings, the grand
    /// opening once it has completed, the damage swap, the periodic checks
    /// of S-0108 claims 17 and 20, and the power rules of claim 13.
    pub fn update(&mut self, kind: &TypeKind, defs: &Defs, st: &State, mk: &mut dyn Make<A>) {
        let last = self.seen.replace(Seen::of(st));
        if let Some(last) = last {
            if !self.early_done {
                // A caller that did not prepare the tick: the early changes act here, first.
                self.early_enablings(kind, st, last);
            }
            self.enablings(kind, defs, st, last);
        }
        self.early_done = false;
        if st.completed && !self.opened {
            self.opened = true;
            self.grand_opening(kind, defs, st, mk);
            if st.setup && st.unmanned {
                // spec: S-0108 claim 26 (placed by the map's building list while not manned, whatever the owner: disabled in claim 13's sense from its placement on, so every freeze-unpowered slot its set-up starts holds the image frame it was created with, whether or not the type needs power)
                self.disable(defs);
            }
            if st.setup && st.switched_off {
                // spec: S-0108 claim 28 (placed switched off by the map, which S-0405 claim 18 allows only for a type with a base drain above 0 or the needs-power flag: switched off, and so disabled, at the end of its placement, after its set-up and grand-opening slot starts, so every freeze-unpowered slot holds the image frame it was created with, whether or not the type needs power)
                self.disable(defs);
            }
            // spec: S-0108 claim 13 (a newly placed or completed building that is not in operation has no power-loss reaction; one in operation has its power-return reaction in its first update) and claim 27 (one the map places switched off has had no change from powered to unpowered until it first comes into operation)
            self.powered = Some(false);
        }
        if !st.completed {
            return;
        }
        // spec: S-0108 claim 12 (the damaged state differs from the slot condition: the swap, on
        // the tick of the damage or repair)
        if st.damaged != self.condition_damaged {
            self.swap(st.damaged, defs, mk);
        }
        if kind.absorbs {
            // spec: S-0108 claim 17 (T < 1: active-2 stops, active-1 starts if absent; T ≥ 1: active-1
            // stops, active-2 starts if absent; both starts allow occupied variants)
            let (off, on) = if st.held_size < 1 {
                (Slot::Active2, Slot::Active1)
            } else {
                (Slot::Active1, Slot::Active2)
            };
            self.stop(off);
            if !self.runs(on) {
                self.start_occupied(on, defs, st, mk);
            }
        }
        // design: the charge cycle after the damage swap and the absorption check, before the power reaction (S-0108 gives no order among a building's own checks)
        self.charge_cycle(kind, defs, st, mk);
        self.power(kind, defs, st, mk);
    }

    /// A repair depot's *idle* or *active-1* start or stop that the
    /// simulation reported (S-0108 claim 20, `Effect::DepotSlot`), in the
    /// damaged variant when `damaged`, never the occupied one.
    pub fn depot(
        &mut self,
        active1: bool,
        start: bool,
        damaged: bool,
        defs: &Defs,
        mk: &mut dyn Make<A>,
    ) {
        let slot = if active1 { Slot::Active1 } else { Slot::Idle };
        if start {
            // spec: S-0108 claim 20 (idle or active-1 starts, normal or damaged variant)
            self.start_variant(slot, defs, damaged, mk);
        } else {
            // spec: S-0108 claim 20 (active-1 and idle stop at the working entry)
            self.stop(slot);
        }
    }

    /// A walls event the simulation reported (S-0606 claims 6, 12 and 16,
    /// `Effect::SpecialAnims`): the *special-n* slots of `pair` start, in
    /// the damaged variant when `damaged`, or stop.
    pub fn walls(
        &mut self,
        pair: [u8; 2],
        start: bool,
        damaged: bool,
        defs: &Defs,
        mk: &mut dyn Make<A>,
    ) {
        for slot in pair.iter().filter_map(|n| Slot::special(*n)) {
            if start {
                // spec: S-0606 claims 6 and 12 (the walls' slots start in their normal or damaged variant)
                self.start_variant(slot, defs, damaged, mk);
            } else {
                // spec: S-0606 claims 12 and 16 (the walls' slots stop)
                self.stop(slot);
            }
        }
    }

    /// `slot`'s animation finished on its own (S-0108 Terms): it is gone,
    /// and the rows of S-0108 claim 20 that react to it run.
    pub fn finished(
        &mut self,
        slot: Slot,
        kind: &TypeKind,
        defs: &Defs,
        st: &State,
        mk: &mut dyn Make<A>,
    ) {
        self.running.remove(&slot);
        match slot {
            // spec: S-0108 claim 20 (a delayed-fire type that is not a repair depot: special-1
            // finishes on its own, active-1 starts with occupied variants allowed; a depot's
            // special slots are the simulation's)
            Slot::Special1 if kind.delayed_fire && !kind.depot => {
                self.start_occupied(Slot::Active1, defs, st, mk);
            }
            // spec: S-0108 claim 18 (when super-2 finishes on its own, super-3 starts; normal or damaged variant)
            Slot::Super2 => self.start_variant(Slot::Super3, defs, st.damaged, mk),
            // spec: S-0108 claim 18 (when super-4 finishes on its own, super-1 starts; normal or damaged variant)
            Slot::Super4 => self.start_variant(Slot::Super1, defs, st.damaged, mk),
            // spec: S-0108 claim 20 (no other building reacts to special-1 or special-3 ending)
            _ => {}
        }
    }
}

/// The held infantry's total size *T* of S-0108 F9: T₀ = 0 and
/// Tᵢ = trunc(Tᵢ₋₁ + sᵢ) over the sizes `sizes` in turn, the addition in
/// binary64 under rounding `mode`.
pub fn held_size(sizes: &[glasnost_num::F64], mode: glasnost_num::Rounding) -> i32 {
    use glasnost_num::F64;
    let mut t: i64 = 0;
    for s in sizes {
        // spec: S-0108 F9 (the addition in binary64 rounded by the mode, then truncated toward zero)
        let sum = F64::from_i64(t, mode).add(*s, mode);
        t = sum.trunc_i128().map_or(0, |v| {
            v.clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i64
        });
    }
    t as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test animation: its name and stage.
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Fake {
        name: Vec<u8>,
        stage: i32,
    }

    /// Makes every name except `MISSING`; a new animation starts at stage 0
    /// or keeps the replaced one's stage.
    struct Maker;

    impl Make<Fake> for Maker {
        fn make(&mut self, name: &[u8], from: Option<&Fake>) -> Option<Fake> {
            (name != b"MISSING").then(|| Fake {
                name: name.to_vec(),
                stage: from.map_or(0, |f| f.stage),
            })
        }
    }

    /// Records the names it makes, in order.
    struct Recorder(Vec<String>);

    impl Make<Fake> for Recorder {
        fn make(&mut self, name: &[u8], from: Option<&Fake>) -> Option<Fake> {
            self.0.push(String::from_utf8(name.to_vec()).unwrap());
            Maker.make(name, from)
        }
    }

    fn def(normal: &str, damaged: &str, occupied: &str) -> SlotDef {
        SlotDef {
            normal: normal.as_bytes().to_vec(),
            damaged: damaged.as_bytes().to_vec(),
            occupied: occupied.as_bytes().to_vec(),
            ..SlotDef::default()
        }
    }

    /// Every slot named after itself: `I`, `A1` … `A4`, `P`, `S1` … `S4`,
    /// `U1` … `U4` (super-1 … super-4) and `L` (super-low-power), with
    /// damaged names `…D` and occupied names `…G`.
    fn defs() -> Defs {
        let names = [
            "I", "A1", "A2", "A3", "A4", "PP", "P", "S1", "S2", "S3", "S4", "U1", "U2", "U3", "U4",
            "L",
        ];
        Slot::ALL
            .iter()
            .zip(names)
            .map(|(s, n)| (*s, def(n, &format!("{n}D"), &format!("{n}G"))))
            .collect()
    }

    fn names(s: &Slots<Fake>) -> Vec<String> {
        s.running
            .values()
            .map(|r| String::from_utf8(r.name.clone()).unwrap())
            .collect()
    }

    fn done() -> State {
        State {
            completed: true,
            ..State::default()
        }
    }

    /// spec: S-0108 claim 11 (grand opening: idle and active-1 … active-4; idle and active-1 allow
    /// occupied variants, the others not; a refinery only its fill level's active slot, active-1
    /// with no stored ore by claim 15; nothing before completion; the set-up starts)
    #[test]
    fn grand_opening_starts() {
        let d = defs();
        let mut s = Slots::default();
        s.update(&TypeKind::default(), &d, &State::default(), &mut Maker);
        assert!(s.running.is_empty(), "not completed: no opening");
        let st = State {
            occupied: true,
            ..done()
        };
        s.update(&TypeKind::default(), &d, &st, &mut Maker);
        assert_eq!(names(&s), ["IG", "A1G", "A2", "A3", "A4"]);
        let mut r = Slots::default();
        let refinery = TypeKind {
            refinery: true,
            ..TypeKind::default()
        };
        r.update(&refinery, &d, &st, &mut Maker);
        assert_eq!(names(&r), ["A1"]);
        // Placed during the set-up: all five with occupied variants allowed, then the grand
        // opening restarts active-2 … active-4 (a refinery: active-1) with no occupied variant.
        let setup = State { setup: true, ..st };
        let mut u = Slots::default();
        u.update(&TypeKind::default(), &d, &setup, &mut Maker);
        assert_eq!(names(&u), ["IG", "A1G", "A2", "A3", "A4"]);
        let mut v = Slots::default();
        v.update(&refinery, &d, &setup, &mut Maker);
        assert_eq!(names(&v), ["IG", "A1", "A2G", "A3G", "A4G"]);
        // The opening runs once: a stopped slot is not restarted by a later update.
        s.stop(Slot::Active3);
        s.update(&TypeKind::default(), &d, &st, &mut Maker);
        assert!(!s.runs(Slot::Active3));
    }

    /// spec: S-0108 claims 12 and 25 (damage swaps every running slot to its damaged name keeping
    /// its stage, repair back to the normal name; the occupied variant is dropped; a slot whose
    /// name for the new condition is empty keeps its animation)
    #[test]
    fn damage_swaps_the_variants() {
        let mut d = defs();
        d.insert(Slot::Active4, def("A4", "", "A4"));
        let mut s = Slots::default();
        let st = State {
            occupied: true,
            ..done()
        };
        s.update(&TypeKind::default(), &d, &st, &mut Maker);
        s.running.get_mut(&Slot::Idle).unwrap().anim.stage = 5;
        let hurt = State {
            damaged: true,
            ..st
        };
        s.update(&TypeKind::default(), &d, &hurt, &mut Maker);
        assert_eq!(names(&s), ["ID", "A1D", "A2D", "A3D", "A4"]);
        assert_eq!(s.running[&Slot::Idle].anim.stage, 5, "the stage is kept");
        s.update(&TypeKind::default(), &d, &st, &mut Maker);
        assert_eq!(
            names(&s),
            ["I", "A1", "A2", "A3", "A4"],
            "no occupied art after a swap"
        );
    }

    /// spec: S-0108 claim 12 (a start in the opposite condition swaps every running slot on the
    /// tick of the start, in the fixed restart order with idle after the active and special
    /// slots, and the started slot's own animation is created after the swap; a started slot
    /// already running is restarted by the swap and replaced again, keeping its stage both times)
    #[test]
    fn opposite_condition_start_swaps() {
        let d = defs();
        let mut s = Slots::default();
        s.update(&TypeKind::default(), &d, &done(), &mut Maker);
        s.start_variant(Slot::Special1, &d, false, &mut Maker);
        s.running.get_mut(&Slot::Special1).unwrap().anim.stage = 3;
        let mut rec = Recorder(Vec::new());
        s.start_variant(Slot::Special1, &d, true, &mut rec);
        assert_eq!(names(&s), ["ID", "A1D", "A2D", "A3D", "A4D", "S1D"]);
        assert_eq!(rec.0, ["A1D", "A2D", "A3D", "A4D", "S1D", "ID", "S1D"]);
        assert_eq!(s.running[&Slot::Special1].anim.stage, 3);
    }

    /// spec: S-0108 Terms and claim 25 (starting a slot: nothing happens when the name is empty or
    /// names no animation type; a running slot is replaced and the new one keeps its stage)
    #[test]
    fn starting_a_slot() {
        let mut d = defs();
        d.insert(Slot::Special1, def("MISSING", "MISSING", "MISSING"));
        d.insert(Slot::Special2, SlotDef::default());
        let mut s: Slots<Fake> = Slots::default();
        s.start_variant(Slot::Special1, &d, false, &mut Maker);
        s.start_variant(Slot::Special2, &d, false, &mut Maker);
        assert!(s.running.is_empty());
        s.start_variant(Slot::Special3, &d, false, &mut Maker);
        s.running.get_mut(&Slot::Special3).unwrap().anim.stage = 7;
        s.start_variant(Slot::Special3, &d, false, &mut Maker);
        assert_eq!(s.running[&Slot::Special3].anim.stage, 7);
    }

    /// spec: S-0108 claim 17 (T < 1: active-1 and not active-2; T ≥ 1: active-2 and not active-1;
    /// back to active-1 when the held infantry leave)
    #[test]
    fn infantry_absorption_swaps_active_one_and_two() {
        let d = defs();
        let absorber = TypeKind {
            absorbs: true,
            ..TypeKind::default()
        };
        let mut s = Slots::default();
        s.update(&absorber, &d, &done(), &mut Maker);
        assert!(s.runs(Slot::Active1) && !s.runs(Slot::Active2));
        let held = State {
            held_size: 1,
            ..done()
        };
        s.update(&absorber, &d, &held, &mut Maker);
        assert!(!s.runs(Slot::Active1) && s.runs(Slot::Active2));
        s.update(&absorber, &d, &done(), &mut Maker);
        assert!(s.runs(Slot::Active1) && !s.runs(Slot::Active2));
        // Another type does not react to held infantry.
        let mut o = Slots::default();
        o.update(&TypeKind::default(), &d, &held, &mut Maker);
        assert!(o.runs(Slot::Active1) && o.runs(Slot::Active2));
    }

    /// spec: S-0108 F9 and claim 17 (each addition truncated: two size-0.75 infantry give T = 0,
    /// one size-1.5 gives T = 1, three size-1 give T = 3)
    #[test]
    fn held_size_truncates_each_addition() {
        use glasnost_num::{F64, Rounding};
        let m = Rounding::TowardZero;
        let q = F64::from_i64(3, m).div(F64::from_i64(4, m), m);
        let h = F64::from_i64(3, m).div(F64::from_i64(2, m), m);
        let one = F64::from_i64(1, m);
        assert_eq!(held_size(&[], m), 0);
        assert_eq!(held_size(&[q, q], m), 0);
        assert_eq!(held_size(&[h], m), 1);
        assert_eq!(held_size(&[one, one, one], m), 3);
    }

    /// spec: S-0108 claim 20 (a repair depot's idle and active-1 follow the simulation's reports:
    /// a start in the reported variant, a stop)
    #[test]
    fn repair_depot_reports() {
        let d = defs();
        let mut s = Slots::default();
        s.depot(true, true, false, &d, &mut Maker);
        s.depot(false, true, false, &d, &mut Maker);
        assert!(s.runs(Slot::Active1) && s.runs(Slot::Idle));
        s.depot(true, false, false, &d, &mut Maker);
        s.depot(false, false, false, &d, &mut Maker);
        assert!(!s.runs(Slot::Active1) && !s.runs(Slot::Idle));
        // The depot's special slots are the simulation's: the drawing runs no row of its own.
        let depot = TypeKind {
            depot: true,
            delayed_fire: true,
            ..TypeKind::default()
        };
        s.finished(Slot::Special1, &depot, &d, &done(), &mut Maker);
        s.finished(Slot::Special3, &depot, &d, &done(), &mut Maker);
        assert!(s.running.is_empty());
    }

    /// spec: S-0606 claims 6, 12 and 16 (walls up: special-1 and special-2 start in the reported
    /// variant; walls down: they stop and special-3 and special-4 start; an abandoned entry stops
    /// special-1 and special-2 only)
    #[test]
    fn walls_start_and_stop_the_special_slots() {
        let d = defs();
        let mut s = Slots::default();
        s.update(&TypeKind::default(), &d, &done(), &mut Maker);
        s.walls([1, 2], true, false, &d, &mut Maker);
        assert!(s.runs(Slot::Special1) && s.runs(Slot::Special2));
        s.walls([1, 2], false, false, &d, &mut Maker);
        s.walls([3, 4], true, false, &d, &mut Maker);
        assert!(!s.runs(Slot::Special1) && !s.runs(Slot::Special2));
        assert!(s.runs(Slot::Special3) && s.runs(Slot::Special4));
        let mut h = Slots::default();
        h.walls([1, 2], true, true, &d, &mut Maker);
        assert_eq!(names(&h), ["S1D", "S2D"]);
        h.walls([1, 2], false, true, &d, &mut Maker);
        assert!(h.running.is_empty());
    }

    /// spec: S-0108 claim 20 (a delayed-fire type that is not a depot restarts active-1, occupied
    /// variants allowed, when special-1 finishes; no other building reacts)
    #[test]
    fn delayed_fire_and_other_types() {
        let d = defs();
        let fire = TypeKind {
            delayed_fire: true,
            ..TypeKind::default()
        };
        let st = State {
            occupied: true,
            ..done()
        };
        let mut s = Slots::default();
        s.update(&fire, &d, &st, &mut Maker);
        s.stop(Slot::Active1);
        s.start_variant(Slot::Special1, &d, false, &mut Maker);
        s.finished(Slot::Special1, &fire, &d, &st, &mut Maker);
        assert_eq!(s.running[&Slot::Active1].name, b"A1G");
        let mut o = Slots::default();
        o.update(&TypeKind::default(), &d, &st, &mut Maker);
        o.stop(Slot::Active1);
        o.stop(Slot::Idle);
        o.start_variant(Slot::Special3, &d, false, &mut Maker);
        o.finished(Slot::Special3, &TypeKind::default(), &d, &st, &mut Maker);
        assert!(!o.runs(Slot::Active1) && !o.runs(Slot::Idle));
    }

    /// A slot named `n` whose art turns *freeze-unpowered* off and sets
    /// *light-unpowered* to `light` and *effect-unpowered* to `effect`.
    fn flagged(n: &str, light: bool, effect: bool) -> SlotDef {
        SlotDef {
            power: slots::PowerFlags {
                freeze: false,
                light,
                effect,
                special: false,
            },
            ..def(n, &format!("{n}D"), &format!("{n}G"))
        }
    }

    /// One building turn as the live view runs it: `prepare`, then
    /// `update`, with `unfreezes` as `quirks.enable_unfreezes_unpowered`.
    fn turn(s: &mut Slots<Fake>, kind: &TypeKind, d: &Defs, st: &State, unfreezes: bool) {
        s.prepare(kind, st, unfreezes);
        s.update(kind, d, st, &mut Maker);
    }

    /// The names of the frozen running slots, in slot order.
    fn frozen(s: &Slots<Fake>) -> Vec<String> {
        s.running
            .values()
            .filter(|r| r.frozen)
            .map(|r| String::from_utf8(r.name.clone()).unwrap())
            .collect()
    }

    fn on() -> State {
        State {
            in_operation: true,
            ..done()
        }
    }

    /// spec: S-0108 claims 13 and 28 (switching off disables the building: every running
    /// freeze-unpowered slot freezes, whether or not the type needs power, and a slot whose art
    /// turns freeze-unpowered off keeps playing; switching on re-enables it and the frozen slots
    /// resume, keeping their animation and stage)
    #[test]
    fn a_switch_off_freezes_and_a_switch_on_resumes() {
        let mut d = defs();
        d.insert(Slot::Active2, flagged("A2", true, false));
        let kind = TypeKind::default();
        let mut s = Slots::default();
        turn(&mut s, &kind, &d, &on(), true);
        assert!(frozen(&s).is_empty());
        s.running.get_mut(&Slot::Idle).unwrap().anim.stage = 3;
        let off = State {
            switched_off: true,
            in_operation: false,
            ..on()
        };
        turn(&mut s, &kind, &d, &off, true);
        assert!(s.disabled());
        assert_eq!(frozen(&s), ["I", "A1", "A3", "A4"]);
        assert!(s.runs(Slot::Active2) && !s.frozen(Slot::Active2));
        turn(&mut s, &kind, &d, &off, true);
        assert_eq!(frozen(&s), ["I", "A1", "A3", "A4"], "still held");
        turn(&mut s, &kind, &d, &on(), true);
        assert!(!s.disabled() && frozen(&s).is_empty());
        assert_eq!(s.running[&Slot::Idle].anim.stage, 3);
        assert_eq!(names(&s), ["I", "A1", "A2", "A3", "A4"]);
    }

    /// spec: S-0108 claims 13 and 26 (a freeze-unpowered slot that starts while the building is
    /// disabled or not in operation starts frozen only when the type needs power; a damage swap
    /// is a start, so for a type that does not need power it restarts the slots running and
    /// unfrozen although the building is still disabled, and for one that needs power they
    /// restart frozen)
    #[test]
    fn starts_while_disabled_follow_the_start_rule() {
        let mut d = defs();
        d.insert(Slot::Active2, flagged("A2", true, false));
        let off = State {
            switched_off: true,
            in_operation: false,
            ..on()
        };
        for needs_power in [false, true] {
            let kind = TypeKind {
                needs_power,
                ..TypeKind::default()
            };
            let mut s = Slots::default();
            turn(&mut s, &kind, &d, &on(), true);
            turn(&mut s, &kind, &d, &off, true);
            s.start_variant(Slot::Special1, &d, false, &mut Maker);
            assert_eq!(s.frozen(Slot::Special1), needs_power);
            let hurt = State {
                damaged: true,
                ..off
            };
            turn(&mut s, &kind, &d, &hurt, true);
            assert_eq!(names(&s), ["ID", "A1D", "A2D", "A3D", "A4D", "S1D"]);
            assert!(s.disabled());
            let held: &[&str] = if needs_power {
                &["ID", "A1D", "A3D", "A4D", "S1D"]
            } else {
                &[]
            };
            assert_eq!(frozen(&s), held, "needs power: {needs_power}");
        }
        // Not in operation without a disabling (a type that needs power, its house short of
        // power): the start rule reads the in-operation result.
        let kind = TypeKind {
            needs_power: true,
            ..TypeKind::default()
        };
        let low = done();
        let mut s = Slots::default();
        turn(&mut s, &kind, &d, &low, true);
        assert!(!s.disabled());
        assert_eq!(frozen(&s), ["I", "A1", "A3", "A4"]);
    }

    /// spec: S-0108 claim 26 (placed by the map while not manned: disabled from its placement on,
    /// so every freeze-unpowered slot its set-up starts holds the frame it was created with,
    /// whether or not the type needs power, and a slot whose art turns freeze-unpowered off
    /// plays; at an ownership change it is re-enabled and the frozen slots resume from the frame
    /// they held, keeping their variant, whether or not it is then in operation)
    #[test]
    fn an_unmanned_map_building_holds_until_its_owner_changes() {
        let mut d = defs();
        d.insert(Slot::Active2, flagged("A2", true, false));
        for needs_power in [false, true] {
            let kind = TypeKind {
                needs_power,
                needs_engineer: true,
                ..TypeKind::default()
            };
            let placed = State {
                setup: true,
                unmanned: true,
                owner: Some(9),
                ..done()
            };
            let mut s = Slots::default();
            turn(&mut s, &kind, &d, &placed, true);
            assert!(s.disabled());
            assert_eq!(frozen(&s), ["I", "A1", "A3", "A4"]);
            assert!(s.runs(Slot::Active2));
            turn(&mut s, &kind, &d, &placed, true);
            assert_eq!(
                frozen(&s),
                ["I", "A1", "A3", "A4"],
                "a later update changes nothing"
            );
            s.running.get_mut(&Slot::Idle).unwrap().anim.stage = 2;
            // Captured: manned; for a type that needs power the new house is short of power
            // here, so it is not in operation after the change.
            let captured = State {
                owner: Some(0),
                unmanned: false,
                in_operation: !needs_power,
                ..placed
            };
            turn(&mut s, &kind, &d, &captured, true);
            assert!(!s.disabled() && frozen(&s).is_empty());
            assert_eq!(s.running[&Slot::Idle].anim.stage, 2);
            assert_eq!(names(&s), ["I", "A1", "A2", "A3", "A4"]);
        }
    }

    /// spec: S-0108 Edge cases (quirks.enable_unfreezes_unpowered, both positions: on, the end
    /// of a pulse on a building that is not manned and an ownership change that leaves it not in
    /// operation unfreeze its slots; off, a re-enabling or an ownership change unfreezes them
    /// only when the building is in operation right after it) and claim 26
    #[test]
    fn the_enable_quirk_in_both_positions() {
        let d = defs();
        let kind = TypeKind {
            needs_power: true,
            needs_engineer: true,
            ..TypeKind::default()
        };
        let placed = State {
            setup: true,
            unmanned: true,
            owner: Some(9),
            ..done()
        };
        let pulsed = State {
            pulsed: true,
            ..placed
        };
        // Captured into a house short of power: manned, not in operation.
        let short = State {
            owner: Some(0),
            unmanned: false,
            ..placed
        };
        // Captured again, into a house with enough power: in operation.
        let ample = State {
            owner: Some(1),
            in_operation: true,
            ..short
        };
        for unfreezes in [true, false] {
            let mut s = Slots::default();
            turn(&mut s, &kind, &d, &placed, unfreezes);
            assert_eq!(frozen(&s).len(), 5);
            turn(&mut s, &kind, &d, &pulsed, unfreezes);
            turn(&mut s, &kind, &d, &placed, unfreezes);
            assert_eq!(
                frozen(&s).is_empty(),
                unfreezes,
                "pulse end, quirk {unfreezes}"
            );
            turn(&mut s, &kind, &d, &short, unfreezes);
            assert_eq!(
                frozen(&s).is_empty(),
                unfreezes,
                "capture, quirk {unfreezes}"
            );
            turn(&mut s, &kind, &d, &ample, unfreezes);
            assert!(
                frozen(&s).is_empty(),
                "capture into operation, quirk {unfreezes}"
            );
        }
    }

    /// spec: S-0108 claims 13 and 28 (a pulse disables the building and its end re-enables it
    /// whatever its power switch: a building still switched off when its pulse ends has its
    /// frozen freeze-unpowered slots resume)
    #[test]
    fn a_pulse_end_resumes_a_switched_off_building() {
        let d = defs();
        let kind = TypeKind::default();
        let mut s = Slots::default();
        turn(&mut s, &kind, &d, &on(), true);
        let off = State {
            switched_off: true,
            in_operation: false,
            ..on()
        };
        turn(&mut s, &kind, &d, &off, true);
        let both = State {
            pulsed: true,
            ..off
        };
        turn(&mut s, &kind, &d, &both, true);
        assert_eq!(frozen(&s).len(), 5);
        turn(&mut s, &kind, &d, &off, true);
        assert!(frozen(&s).is_empty() && !s.disabled());
    }

    /// The state of a building of the map placed switched off for the
    /// neutral house (house 9), not in operation.
    fn placed_off() -> State {
        State {
            setup: true,
            switched_off: true,
            owner: Some(9),
            ..done()
        }
    }

    /// spec: S-0108 claim 28 (placed switched off by the map: disabled at the end of its placement,
    /// after its set-up and grand-opening starts, so every freeze-unpowered slot holds the frame it
    /// was created with, whether or not the type needs power, and a slot whose art turns
    /// freeze-unpowered off keeps playing; an ownership change of a type that does not need an
    /// engineer is no re-enabling, so the held slots stay held; the switch-on re-enables it at once,
    /// whether or not it is then in operation, and the held slots resume from the frame they held,
    /// with no start, stop or restart, keeping their variant; in a house short of power a type that
    /// needs and draws power stays not in operation, so its resumed slots go on advancing; a type
    /// with the needs-power flag and no base drain gets neither power column) and claim 27 (its
    /// light-unpowered slot keeps playing before and after the switch-on): S-0108 B22 parts (l) and
    /// (m), on test types D, E and a needs-power type with no drain
    #[test]
    fn a_building_placed_switched_off_holds_until_its_switch_on() {
        let mut d = defs();
        d.insert(Slot::Active2, flagged("A2", true, false));
        // D: a drain and no needs-power flag; E: both (needs and draws power); N: the needs-power
        // flag and no drain.
        let kinds = [
            ("D", TypeKind::default()),
            (
                "E",
                TypeKind {
                    needs_power: true,
                    draws_power: true,
                    ..TypeKind::default()
                },
            ),
            (
                "N",
                TypeKind {
                    needs_power: true,
                    ..TypeKind::default()
                },
            ),
        ];
        for (name, kind) in kinds {
            let mut s = Slots::default();
            turn(&mut s, &kind, &d, &placed_off(), true);
            assert!(s.disabled(), "{name}");
            assert_eq!(frozen(&s), ["I", "A1", "A3", "A4"], "{name}");
            assert!(s.runs(Slot::Active2) && !s.frozen(Slot::Active2), "{name}");
            turn(&mut s, &kind, &d, &placed_off(), true);
            assert_eq!(frozen(&s), ["I", "A1", "A3", "A4"], "{name}: still held");
            s.running.get_mut(&Slot::Idle).unwrap().anim.stage = 2;
            // Given to the player's house (a trigger action): no re-enabling.
            let given = State {
                owner: Some(0),
                ..placed_off()
            };
            turn(&mut s, &kind, &d, &given, true);
            assert!(
                s.disabled(),
                "{name}: the ownership change is no re-enabling"
            );
            assert_eq!(
                frozen(&s),
                ["I", "A1", "A3", "A4"],
                "{name}: held through it"
            );
            // Switched on. The house is short of power: E stays not in operation; D and N are in
            // operation (N never stops for low power, S-0405 claim 8).
            let on = State {
                switched_off: false,
                in_operation: name != "E",
                ..given
            };
            let mut rec = Recorder(Vec::new());
            s.prepare(&kind, &on, true);
            s.update(&kind, &d, &on, &mut rec);
            assert!(!s.disabled() && frozen(&s).is_empty(), "{name}: resumed");
            assert!(
                rec.0.is_empty(),
                "{name}: no slot started again: {:?}",
                rec.0
            );
            assert_eq!(
                s.running[&Slot::Idle].anim.stage,
                2,
                "{name}: from the held frame"
            );
            assert_eq!(
                names(&s),
                ["I", "A1", "A2", "A3", "A4"],
                "{name}: same variants"
            );
            for _ in 0..3 {
                turn(&mut s, &kind, &d, &on, true);
            }
            assert!(frozen(&s).is_empty(), "{name}: they go on advancing");
            assert!(
                s.runs(Slot::Active2),
                "{name}: claim 27, no power-loss reaction"
            );
        }
    }

    /// spec: S-0108 claim 28 (a building of a type that needs an engineer placed switched off and
    /// not manned: its ownership change is a re-enabling, claim 26, so its held slots resume
    /// although it stays switched off, `quirks.enable_unfreezes_unpowered`)
    #[test]
    fn a_needs_engineer_building_placed_switched_off_resumes_at_its_capture() {
        let d = defs();
        let kind = TypeKind {
            needs_engineer: true,
            ..TypeKind::default()
        };
        let placed = State {
            unmanned: true,
            ..placed_off()
        };
        let mut s = Slots::default();
        turn(&mut s, &kind, &d, &placed, true);
        assert_eq!(frozen(&s).len(), 5);
        let captured = State {
            owner: Some(0),
            unmanned: false,
            ..placed
        };
        turn(&mut s, &kind, &d, &captured, true);
        assert!(frozen(&s).is_empty() && !s.disabled());
    }

    /// spec: S-0108 claim 13 (the re-enablings are a switch-on, a pulse's end and an ownership
    /// change of a building whose type needs an engineer; an ownership change of any other building
    /// is neither: a building switched off in play stays held through it, and one frozen by a loss
    /// of operation stays frozen until power comes back)
    #[test]
    fn an_ownership_change_of_another_type_is_no_re_enabling() {
        let d = defs();
        let kind = TypeKind::default();
        let mut s = Slots::default();
        turn(&mut s, &kind, &d, &on(), true);
        let off = State {
            switched_off: true,
            in_operation: false,
            ..on()
        };
        turn(&mut s, &kind, &d, &off, true);
        let captured = State {
            owner: Some(1),
            ..off
        };
        turn(&mut s, &kind, &d, &captured, true);
        assert!(s.disabled() && frozen(&s).len() == 5);
        // A type that needs and draws power, frozen by a loss of operation, then captured into a
        // house still short of power.
        let draws = TypeKind {
            needs_power: true,
            draws_power: true,
            ..TypeKind::default()
        };
        let mut p = Slots::default();
        turn(&mut p, &draws, &d, &on(), true);
        turn(&mut p, &draws, &d, &done(), true);
        assert_eq!(frozen(&p).len(), 5);
        let taken = State {
            owner: Some(1),
            ..done()
        };
        turn(&mut p, &draws, &d, &taken, true);
        assert_eq!(frozen(&p).len(), 5, "still frozen");
        // The same with the needs-engineer flag: the change re-enables it.
        let tech = TypeKind {
            needs_engineer: true,
            ..draws
        };
        let mut t = Slots::default();
        turn(&mut t, &tech, &d, &on(), true);
        turn(&mut t, &tech, &d, &done(), true);
        turn(&mut t, &tech, &d, &taken, true);
        assert!(
            frozen(&t).is_empty(),
            "resumed by the needs-engineer change"
        );
    }

    /// spec: S-0108 claim 13 (several disablings and re-enablings in one tick: the slots end as the
    /// last of them leaves them, in their places in the tick; a switch-off order applied in the tick
    /// of an ownership change that gives the building to the order's house, or of a pulse's end,
    /// leaves them frozen; a pulse that starts in the tick of an ownership change made by a polled
    /// trigger's action leaves them frozen; a switch-on order refused under a pulse changes no slot;
    /// for a type that needs and draws power, its reaction to a change of its in-operation result is
    /// one more event, here the last)
    #[test]
    fn several_in_one_tick() {
        let d = defs();
        for needs_engineer in [false, true] {
            let kind = TypeKind {
                needs_engineer,
                ..TypeKind::default()
            };
            // An ownership change to house 0 and house 0's switch-off order in one tick.
            let mut s = Slots::default();
            let theirs = State {
                owner: Some(1),
                ..on()
            };
            turn(&mut s, &kind, &d, &theirs, true);
            let both = State {
                owner: Some(0),
                switched_off: true,
                in_operation: false,
                ..on()
            };
            turn(&mut s, &kind, &d, &both, true);
            assert_eq!(frozen(&s).len(), 5, "change then order: {needs_engineer}");
        }
        let kind = TypeKind::default();
        // A pulse's end and a switch-off order in one tick.
        let mut s = Slots::default();
        turn(&mut s, &kind, &d, &on(), true);
        let pulsed = State {
            pulsed: true,
            in_operation: false,
            ..on()
        };
        turn(&mut s, &kind, &d, &pulsed, true);
        let ended_off = State {
            switched_off: true,
            in_operation: false,
            ..on()
        };
        turn(&mut s, &kind, &d, &ended_off, true);
        assert_eq!(frozen(&s).len(), 5, "pulse end then order");
        // A switch-on order under a pulse is refused (S-0405 claim 10): the switch stays off.
        let both = State {
            pulsed: true,
            ..ended_off
        };
        turn(&mut s, &kind, &d, &both, true);
        turn(&mut s, &kind, &d, &both, true);
        assert_eq!(frozen(&s).len(), 5, "no change under the pulse");
        // A pulse that starts in the tick of an ownership change (a polled trigger's action) of a
        // needs-engineer building.
        let tech = TypeKind {
            needs_engineer: true,
            ..TypeKind::default()
        };
        let mut t = Slots::default();
        turn(&mut t, &tech, &d, &on(), true);
        let hit = State {
            owner: Some(1),
            pulsed: true,
            in_operation: false,
            ..on()
        };
        turn(&mut t, &tech, &d, &hit, true);
        assert_eq!(frozen(&t).len(), 5, "change then pulse");
        // The same for a type that needs and draws power in operation before the tick: its
        // reaction to the loss of operation is the last event.
        let draws = TypeKind {
            needs_power: true,
            draws_power: true,
            ..tech
        };
        let mut r = Slots::default();
        turn(&mut r, &draws, &d, &on(), true);
        turn(&mut r, &draws, &d, &hit, true);
        assert_eq!(frozen(&r).len(), 5, "the reaction last");
    }

    /// spec: S-0108 claim 13 (disablings and re-enablings close together: a switch-off order in the
    /// tick before an ownership change of a needs-engineer building leaves its slots playing after
    /// the change although it stays switched off; a switch-off under a pulse leaves them held to the
    /// pulse's end, which re-enables it although it stays switched off, and a switch-off after the
    /// pulse's end freezes them again): S-0108 B22 parts (n) and (o), on test type W
    #[test]
    fn a_switch_off_close_to_an_ownership_change_or_a_pulse_end() {
        let d = defs();
        let w = TypeKind {
            needs_engineer: true,
            ..TypeKind::default()
        };
        let mine = State {
            owner: Some(0),
            ..on()
        };
        let off = State {
            switched_off: true,
            in_operation: false,
            ..mine
        };
        // (n): the order in tick t - 1, the change of owner in tick t.
        let mut s = Slots::default();
        turn(&mut s, &w, &d, &mine, true);
        turn(&mut s, &w, &d, &off, true);
        assert_eq!(frozen(&s).len(), 5, "held from the order");
        let given = State {
            owner: Some(1),
            ..off
        };
        turn(&mut s, &w, &d, &given, true);
        assert!(
            frozen(&s).is_empty(),
            "playing after the change, still switched off"
        );
        // (o): the order under the pulse, then its end.
        let pulsed = State {
            pulsed: true,
            in_operation: false,
            ..mine
        };
        let mut p = Slots::default();
        turn(&mut p, &w, &d, &mine, true);
        turn(&mut p, &w, &d, &pulsed, true);
        let both = State {
            switched_off: true,
            ..pulsed
        };
        turn(&mut p, &w, &d, &both, true);
        assert_eq!(frozen(&p).len(), 5, "held to the pulse's end");
        turn(&mut p, &w, &d, &off, true);
        assert!(frozen(&p).is_empty(), "the pulse's end re-enables it");
        // (o): the order after the pulse's end.
        let mut q = Slots::default();
        turn(&mut q, &w, &d, &mine, true);
        turn(&mut q, &w, &d, &pulsed, true);
        turn(&mut q, &w, &d, &mine, true);
        turn(&mut q, &w, &d, &mine, true);
        assert!(frozen(&q).is_empty());
        turn(&mut q, &w, &d, &off, true);
        for _ in 0..3 {
            turn(&mut q, &w, &d, &off, true);
        }
        assert_eq!(frozen(&q).len(), 5, "held from the later order on");
    }

    /// spec: S-0108 claim 13 (a type that needs and draws power: at a loss of operation a running
    /// freeze-unpowered slot freezes, a light-unpowered one stops, an effect-unpowered one stops
    /// and is marked; when power comes back the frozen ones resume, a light-unpowered slot that is
    /// not running starts and the marked ones start, in the normal or damaged variant; once to
    /// each change; a type that needs but does not draw power has no reaction)
    #[test]
    fn the_power_columns() {
        let mut d = defs();
        d.insert(Slot::Active2, flagged("A2", true, false));
        d.insert(Slot::Active3, flagged("A3", false, true));
        let draws = TypeKind {
            needs_power: true,
            draws_power: true,
            ..TypeKind::default()
        };
        let low = done();
        let mut s = Slots::default();
        turn(&mut s, &draws, &d, &on(), true);
        assert_eq!(names(&s), ["I", "A1", "A2", "A3", "A4"]);
        turn(&mut s, &draws, &d, &low, true);
        assert_eq!(names(&s), ["I", "A1", "A4"]);
        assert_eq!(frozen(&s), ["I", "A1", "A4"]);
        turn(&mut s, &draws, &d, &low, true);
        assert_eq!(names(&s), ["I", "A1", "A4"], "once to each change");
        let hurt = State {
            damaged: true,
            occupied: true,
            ..on()
        };
        turn(&mut s, &draws, &d, &hurt, true);
        assert_eq!(names(&s), ["ID", "A1D", "A2D", "A3D", "A4D"]);
        assert!(frozen(&s).is_empty());
        let needs = TypeKind {
            needs_power: true,
            ..TypeKind::default()
        };
        let mut n = Slots::default();
        turn(&mut n, &needs, &d, &on(), true);
        turn(&mut n, &needs, &d, &low, true);
        assert_eq!(names(&n), ["I", "A1", "A2", "A3", "A4"]);
        assert!(frozen(&n).is_empty());
    }

    /// spec: S-0108 claim 27 (a building that needs and draws power and completes while not in
    /// operation has had no change from powered to unpowered: its light-unpowered and
    /// effect-unpowered slots keep playing, and claim 13's start rule freezes its
    /// freeze-unpowered ones) and claim 13 (one in operation when completed has its power-return
    /// reaction in its first update; power coming back later is a return)
    #[test]
    fn a_building_never_powered() {
        let mut d = defs();
        d.insert(Slot::Active2, flagged("A2", true, false));
        d.insert(Slot::Active3, flagged("A3", false, true));
        d.insert(Slot::Special2, flagged("S2", true, false));
        let draws = TypeKind {
            needs_power: true,
            draws_power: true,
            ..TypeKind::default()
        };
        let low = done();
        let mut s = Slots::default();
        turn(&mut s, &draws, &d, &low, true);
        turn(&mut s, &draws, &d, &low, true);
        assert_eq!(names(&s), ["I", "A1", "A2", "A3", "A4"]);
        assert_eq!(frozen(&s), ["I", "A1", "A4"]);
        let mut p = Slots::default();
        turn(&mut p, &draws, &d, &on(), true);
        assert!(
            p.runs(Slot::Special2),
            "the power return in the first update"
        );
        assert!(!s.runs(Slot::Special2));
        turn(&mut s, &draws, &d, &on(), true);
        assert!(frozen(&s).is_empty() && s.runs(Slot::Special2));
    }

    /// spec: S-0108 claim 13 (a light-unpowered special-1 of a delayed-fire type whose active-1
    /// has freeze-unpowered: at a loss it stops and active-1 starts, unpowered so frozen; at the
    /// return special-1 does not start)
    #[test]
    fn a_delayed_fire_outage() {
        let mut d = defs();
        d.insert(Slot::Special1, flagged("S1", true, false));
        let fire = TypeKind {
            delayed_fire: true,
            needs_power: true,
            draws_power: true,
            ..TypeKind::default()
        };
        let mut s = Slots::default();
        turn(&mut s, &fire, &d, &on(), true);
        assert!(
            !s.runs(Slot::Special1),
            "no start at the first update's return"
        );
        s.stop(Slot::Active1);
        s.start_variant(Slot::Special1, &d, false, &mut Maker);
        turn(&mut s, &fire, &d, &done(), true);
        assert!(!s.runs(Slot::Special1));
        assert!(s.frozen(Slot::Active1));
        turn(&mut s, &fire, &d, &on(), true);
        assert!(!s.runs(Slot::Special1) && !s.frozen(Slot::Active1));
    }

    fn sw() -> TypeKind {
        TypeKind {
            superweapon: true,
            ..TypeKind::default()
        }
    }

    /// A completed building whose owner's record has `r` ticks of charge
    /// left (`None`: no reading), with the F6 minutes.
    fn charging(r: Option<i32>) -> State {
        State {
            charge_remaining: r,
            charge_minutes: r.map(|r| charge_minutes(r, glasnost_num::Rounding::TowardZero)),
            ..done()
        }
    }

    /// spec: S-0108 claim 11 and Constants (a building with a superweapon: super-3 when the
    /// remaining charge is below 15 ticks, otherwise super-1, both with occupied variants allowed;
    /// alongside the other opening starts, a refinery's included; at a set-up placement too)
    #[test]
    fn grand_opening_starts_a_superweapon_slot() {
        let d = defs();
        let open = |kind: &TypeKind, st: &State| {
            let mut s = Slots::default();
            s.update(kind, &d, st, &mut Maker);
            names(&s)
        };
        assert_eq!(
            open(&sw(), &charging(Some(15))),
            ["I", "A1", "A2", "A3", "A4", "U1"]
        );
        assert_eq!(
            open(&sw(), &charging(Some(14))),
            ["I", "A1", "A2", "A3", "A4", "U3"]
        );
        assert_eq!(open(&sw(), &charging(Some(0))).last().unwrap(), "U3");
        // Occupied variants allowed.
        let occupied = State {
            occupied: true,
            ..charging(Some(900))
        };
        assert_eq!(open(&sw(), &occupied).last().unwrap(), "U1G");
        // No reading (quirks.ungranted_super_reads_clock off and the record not granted): not
        // charged.
        assert_eq!(open(&sw(), &charging(None)).last().unwrap(), "U1");
        // A refinery with a superweapon: its fill-level slot and the superweapon slot.
        let refinery = TypeKind {
            refinery: true,
            ..sw()
        };
        assert_eq!(open(&refinery, &charging(Some(900))), ["A1", "U1"]);
        // A set-up placement.
        let setup = State {
            setup: true,
            ..charging(Some(3))
        };
        assert_eq!(open(&sw(), &setup).last().unwrap(), "U3");
        // No superweapon: no superweapon slot.
        assert_eq!(
            open(&TypeKind::default(), &charging(Some(3))),
            ["I", "A1", "A2", "A3", "A4"]
        );
    }

    /// spec: S-0108 F6 and Constants (t_r = r × κ in binary64, κ = 2 386 093 / 2³¹ exactly, a
    /// little above 1/900)
    #[test]
    fn remaining_charge_in_game_minutes_f6() {
        use glasnost_num::{F64, Rounding};
        let m = Rounding::TowardZero;
        let kappa = F64::from_i64(2_386_093, m).div(F64::from_i64(1 << 31, m), m);
        assert_eq!(charge_minutes(1, m), kappa);
        assert_eq!(charge_minutes(0, m), F64::ZERO);
        // 900 ticks are a little more than one game minute.
        let one = F64::from_i64(1, m);
        assert!(one.lt(charge_minutes(900, m)));
        assert!(charge_minutes(899, m).lt(one));
    }

    /// spec: S-0108 claim 18 (t_c at most 990: when t_c ≥ t_r and super-1 runs, super-2 replaces it;
    /// super-2 finishing starts super-3; when t_c < t_r and super-3 runs, super-4 replaces it;
    /// super-4 finishing starts super-1; normal or damaged variant; not while being built or sold;
    /// t_c above 990, or absent with its default 999: no check)
    #[test]
    fn the_superweapon_charge_cycle() {
        use glasnost_num::{F64, Rounding};
        let d = defs();
        let tc = |m: i64| Some(F64::from_i64(m, Rounding::TowardZero));
        let kind = TypeKind {
            charged_anim_time: tc(30),
            ..sw()
        };
        let mut s = Slots::default();
        s.update(&kind, &d, &charging(Some(100_000)), &mut Maker);
        assert!(s.runs(Slot::Super1));
        // 27 000 ticks are a little more than 30 minutes (κ is above 1/900): still charging.
        s.update(&kind, &d, &charging(Some(27_000)), &mut Maker);
        assert!(s.runs(Slot::Super1) && !s.runs(Slot::Super2));
        // 26 999 ticks: t_c ≥ t_r, the charged branch.
        s.update(&kind, &d, &charging(Some(26_999)), &mut Maker);
        assert!(!s.runs(Slot::Super1) && s.runs(Slot::Super2));
        // Super-2 finishes on its own: super-3, here damaged.
        let hurt = State {
            damaged: true,
            ..charging(Some(10))
        };
        s.update(&kind, &d, &hurt, &mut Maker);
        s.finished(Slot::Super2, &kind, &d, &hurt, &mut Maker);
        assert_eq!(names(&s).last().unwrap(), "U3D");
        // Fired: the charge restarts, t_c < t_r: super-4 replaces super-3.
        s.update(&kind, &d, &charging(Some(100_000)), &mut Maker);
        assert!(!s.runs(Slot::Super3) && s.runs(Slot::Super4));
        s.finished(Slot::Super4, &kind, &d, &charging(Some(99_000)), &mut Maker);
        assert!(s.runs(Slot::Super1) && !s.runs(Slot::Super4));
        // Being sold: no check.
        let sold = State {
            selling: true,
            ..charging(Some(0))
        };
        s.update(&kind, &d, &sold, &mut Maker);
        assert!(s.runs(Slot::Super1));
        // t_c above 990, or not set (999): no check, super-1 stays.
        for k in [
            TypeKind {
                charged_anim_time: tc(991),
                ..sw()
            },
            sw(),
        ] {
            let mut s = Slots::default();
            s.update(&k, &d, &charging(Some(100_000)), &mut Maker);
            s.update(&k, &d, &charging(Some(0)), &mut Maker);
            assert!(s.runs(Slot::Super1) && !s.runs(Slot::Super2));
        }
        // At 990 the check runs.
        let at = TypeKind {
            charged_anim_time: tc(990),
            ..sw()
        };
        let mut s = Slots::default();
        s.update(&at, &d, &charging(Some(100_000)), &mut Maker);
        s.update(&at, &d, &charging(Some(0)), &mut Maker);
        assert!(s.runs(Slot::Super2));
        // No reading (the switch off and the record not granted): the cycle waits (claim 18).
        let mut s = Slots::default();
        s.update(&kind, &d, &charging(None), &mut Maker);
        s.update(&kind, &d, &charging(None), &mut Maker);
        assert!(s.runs(Slot::Super1) && !s.runs(Slot::Super2));
    }

    /// spec: S-0108 claim 13 (an effect-unpowered super-3: at a power loss it stops, is marked, and
    /// super-low-power starts; at the power return super-low-power stops and the marked super-3
    /// starts again), claim 12 (super-1 … super-4 restart before idle, super-low-power last)
    #[test]
    fn super_low_power_at_an_outage() {
        let mut d = defs();
        d.insert(Slot::Super3, flagged("U3", false, true));
        let kind = TypeKind {
            needs_power: true,
            draws_power: true,
            ..sw()
        };
        let powered = State {
            in_operation: true,
            ..charging(Some(0))
        };
        let mut s = Slots::default();
        turn(&mut s, &kind, &d, &powered, true);
        assert!(s.runs(Slot::Super3));
        turn(&mut s, &kind, &d, &charging(Some(0)), true);
        assert!(!s.runs(Slot::Super3) && s.runs(Slot::SuperLowPower));
        turn(&mut s, &kind, &d, &powered, true);
        assert!(s.runs(Slot::Super3) && !s.runs(Slot::SuperLowPower));
        // A freeze-unpowered super-1 is only frozen at a loss: no super-low-power.
        let mut t = Slots::default();
        turn(
            &mut t,
            &kind,
            &d,
            &State {
                in_operation: true,
                ..charging(Some(900))
            },
            true,
        );
        turn(&mut t, &kind, &d, &charging(Some(900)), true);
        assert!(t.frozen(Slot::Super1) && !t.runs(Slot::SuperLowPower));
        // The restart order of claim 12.
        let i = |x: Slot| Slot::RESTART.iter().position(|s| *s == x).unwrap();
        assert!(i(Slot::Special4) < i(Slot::Super1));
        assert!(i(Slot::Super1) < i(Slot::Super2) && i(Slot::Super3) < i(Slot::Super4));
        assert!(i(Slot::Super4) < i(Slot::Idle) && i(Slot::Idle) < i(Slot::SuperLowPower));
        assert_eq!(Slot::RESTART.len(), Slot::ALL.len());
    }

    /// The ticks `from..=to` as the live view plays them (`live::update_slots`):
    /// `prepare` with the state after the tick's logic, then each running slot's
    /// turn, then `update`. A slot's turn is a model of S-0109 claims 5 and 23
    /// for a stage delay of 3: *idle* advances when it is not frozen and at
    /// least 3 ticks have passed since its last advance. Returns the ticks of
    /// *idle*'s advances.
    fn play(
        s: &mut Slots<Fake>,
        kind: &TypeKind,
        d: &Defs,
        at: &dyn Fn(i32) -> State,
        ticks: std::ops::RangeInclusive<i32>,
        last: &mut i32,
    ) -> Vec<i32> {
        let mut out = Vec::new();
        for t in ticks {
            let st = at(t);
            s.prepare(kind, &st, true);
            // spec: S-0109 claim 23 (a held animation does not advance; released, it advances on its first eligible update once d ticks have passed since the last advance) and claim 5
            // measure: M-0014 (idle and active-1 advance every 3 ticks)
            if !s.frozen(Slot::Idle) && t - *last >= 3 {
                *last = t;
                s.running.get_mut(&Slot::Idle).unwrap().anim.stage += 1;
                out.push(t);
            }
            s.update(kind, d, &st, &mut Maker);
        }
        out
    }

    /// W8 of M-0014: running, its *idle* last advanced to frame 2 in 198,
    /// then a pulse from 200 whose hold reaches 0 at the end of 289, and a
    /// switch-off order applied before the logic of `off` (`None`: no order).
    /// In our tick the order applies at the end of the tick before, so the
    /// state after tick `off - 1` shows it.
    fn w8(off: Option<i32>) -> impl Fn(i32) -> State {
        move |t| {
            let switched_off = off.is_some_and(|o| t >= o - 1);
            let pulsed = (200..289).contains(&t);
            State {
                owner: Some(0),
                switched_off,
                pulsed,
                in_operation: !switched_off && !pulsed,
                ..done()
            }
        }
    }

    /// Plays W8 from 199 to 300, its slots set up before; returns *idle*'s
    /// advances from 286 on and its final frame.
    fn w8_run(off: Option<i32>) -> (Vec<i32>, i32) {
        let d = defs();
        let kind = TypeKind {
            needs_engineer: true,
            ..TypeKind::default()
        };
        let mut s = Slots::default();
        let at = w8(off);
        turn(&mut s, &kind, &d, &at(198), true);
        s.running.get_mut(&Slot::Idle).unwrap().anim.stage = 2;
        let mut last = 198;
        let adv = play(&mut s, &kind, &d, &at, 199..=300, &mut last);
        let late: Vec<i32> = adv.into_iter().filter(|t| *t >= 286).collect();
        (late, s.running[&Slot::Idle].anim.stage)
    }

    /// measure: M-0014 (the recording run: W8's idle holds frame 2 under the pulse and advances
    /// 2 to 3 in 289, the tick its hold reaches 0, then every 3 ticks; b22o-em2 and b22o-em3: a
    /// switch-off order before the pulse's end does not hold that advance; b22o-ep1: an order
    /// applied before 290 holds the slots at frame 3 from 290; b22o-ep2 at 291 and b22o-ep3 at
    /// 292: the advance due in 292 does not come; b22o-ep6 at 295: frame 4 from 292, the advance
    /// due in 295 does not come)
    /// spec: S-0108 claims 13 and 28 (a pulse's end re-enables the building whatever its power
    /// switch, before the orders of its tick; a power order acts after the turn pass); S-0109
    /// claim 23
    #[test]
    fn m0014_pulse_end_and_switch_off_timing() {
        assert_eq!(w8_run(None), (vec![289, 292, 295, 298], 6), "recording");
        assert_eq!(w8_run(Some(287)), (vec![289, 292, 295, 298], 6), "em2");
        assert_eq!(w8_run(Some(286)), (vec![289, 292, 295, 298], 6), "em3");
        assert_eq!(w8_run(Some(290)), (vec![289], 3), "ep1");
        assert_eq!(w8_run(Some(291)), (vec![289], 3), "ep2");
        assert_eq!(w8_run(Some(292)), (vec![289], 3), "ep3");
        assert_eq!(w8_run(Some(295)), (vec![289, 292], 4), "ep6");
    }

    /// W7 of M-0014 from 290 to 306: manned, its *idle* advancing on the
    /// grid 296, 299; a trigger gives it to the bystander at 300; a
    /// switch-off order applied before the logic of `off`.
    fn w7_run(off: i32) -> Vec<i32> {
        let d = defs();
        let kind = TypeKind {
            needs_engineer: true,
            ..TypeKind::default()
        };
        let at = move |t: i32| {
            let switched_off = t >= off - 1;
            State {
                owner: Some(if t >= 300 { 1 } else { 0 }),
                switched_off,
                in_operation: !switched_off,
                change_after_slots: false,
                ..done()
            }
        };
        let mut s = Slots::default();
        turn(&mut s, &kind, &d, &at(289), true);
        let mut last = 290;
        play(&mut s, &kind, &d, &at, 291..=306, &mut last)
    }

    /// measure: M-0014 (b22n-t: an order applied before the logic of 300, the tick of the
    /// trigger's change of owner: idle advances in 299 and 302; b22n-tm1: an order applied before
    /// 299 holds the advance due in 299, and the change of owner at 300 re-enables W7 and idle
    /// advances in 300, then every 3 ticks)
    /// spec: S-0108 claims 13 and 26 (an ownership change of a type that needs an engineer
    /// re-enables it; a polled trigger acts before any object's turn, S-0203 row 1)
    #[test]
    fn m0014_trigger_owner_change_timing() {
        assert_eq!(w7_run(300), [293, 296, 299, 302, 305], "b22n-t");
        assert_eq!(w7_run(299), [293, 296, 300, 303, 306], "b22n-tm1");
    }

    /// An engineer's capture (an announcing ownership change in the turn
    /// pass) re-enables the building after its slots' turns: a held slot
    /// whose clock is past due first advances in the tick after the capture.
    /// measure: M-0014 (the captures at 133 and 134, as the run's timeline only: idle and active-1
    /// advance in the next tick, then every 3 ticks)
    /// spec: S-0108 claims 13 and 26; Q-2186 (an engineer that stands after the slots)
    #[test]
    fn m0014_capture_releases_in_the_next_tick() {
        let d = defs();
        let kind = TypeKind {
            needs_engineer: true,
            ..TypeKind::default()
        };
        for (change_after_slots, first) in [(true, 134), (false, 133)] {
            let at = move |t: i32| State {
                setup: true,
                unmanned: t < 133,
                owner: Some(if t >= 133 { 0 } else { 9 }),
                change_after_slots: change_after_slots && t == 133,
                in_operation: t >= 133,
                ..done()
            };
            let mut s = Slots::default();
            turn(&mut s, &kind, &d, &at(0), true);
            assert!(s.frozen(Slot::Idle));
            let mut last = 0;
            let adv = play(&mut s, &kind, &d, &at, 1..=140, &mut last);
            assert_eq!(
                adv[0], first,
                "captured in the turn pass: {change_after_slots}"
            );
            assert_eq!(adv[1], first + 3);
        }
    }
}
