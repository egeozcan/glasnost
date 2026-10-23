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
//! `Effect::DepotSlot` reports ([`Slots::depot`]). Not covered here: the
//! power rules of claims 13 and 14 (no slot freezes or lights with power),
//! silo fill (claim 16), the superweapon and turret slots (claims 11, 18
//! and 19), the upgrade slots and the recycling building's row of claim
//! 20.

use std::collections::BTreeMap;

use super::slots::{self, SlotDef};

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
}

impl Slot {
    /// This module's slots in S-0108 claim 12's restart order: *active-1*
    /// to *active-4*, *production*, *special-1* to *special-4*, then
    /// *idle* (the upgrade, pre-production, turret, super and low-power
    /// slots of that order are not played here).
    // spec: S-0108 claim 12 (the running slots are restarted one after another in the fixed order upgrade-1 to upgrade-3, active-1 to active-4, pre-production, production, turret, special-1 to special-4, super-1 to super-4, idle, low-power, super-low-power)
    pub const RESTART: [Slot; 10] = [
        Slot::Active1,
        Slot::Active2,
        Slot::Active3,
        Slot::Active4,
        Slot::Production,
        Slot::Special1,
        Slot::Special2,
        Slot::Special3,
        Slot::Special4,
        Slot::Idle,
    ];

    /// Every slot of this module, in our own order (also the drawing
    /// order of a building's slot animations: our placeholder).
    pub const ALL: [Slot; 10] = [
        Slot::Idle,
        Slot::Active1,
        Slot::Active2,
        Slot::Active3,
        Slot::Active4,
        Slot::Production,
        Slot::Special1,
        Slot::Special2,
        Slot::Special3,
        Slot::Special4,
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
            Slot::Production => b"ProductionAnim",
            Slot::Special1 => b"SpecialAnim",
            Slot::Special2 => b"SpecialAnimTwo",
            Slot::Special3 => b"SpecialAnimThree",
            Slot::Special4 => b"SpecialAnimFour",
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
}

impl<A> Default for Slots<A> {
    fn default() -> Self {
        Self {
            running: BTreeMap::new(),
            condition_damaged: false,
            opened: false,
        }
    }
}

impl<A> Slots<A> {
    /// Whether `slot` holds a running animation.
    pub fn runs(&self, slot: Slot) -> bool {
        self.running.contains_key(&slot)
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
            self.running.insert(slot, Running { name, anim });
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
                self.running.insert(
                    slot,
                    Running {
                        name: name.clone(),
                        anim,
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
            return;
        }
        // spec: S-0108 claim 11 (idle and active-1 with occupied variants allowed; active-2,
        // active-3 and active-4 with no occupied variant)
        self.start_occupied(Slot::Idle, defs, st, mk);
        self.start_occupied(Slot::Active1, defs, st, mk);
        for slot in [Slot::Active2, Slot::Active3, Slot::Active4] {
            self.start_variant(slot, defs, st.damaged, mk);
        }
    }

    /// The building's update: the grand opening once it has completed, the
    /// damage swap, and the periodic checks of S-0108 claims 17 and 20.
    pub fn update(&mut self, kind: &TypeKind, defs: &Defs, st: &State, mk: &mut dyn Make<A>) {
        if !st.completed {
            return;
        }
        if !self.opened {
            self.opened = true;
            self.grand_opening(kind, defs, st, mk);
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
    /// with damaged names `…D` and occupied names `…G`.
    fn defs() -> Defs {
        let names = ["I", "A1", "A2", "A3", "A4", "P", "S1", "S2", "S3", "S4"];
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
}
