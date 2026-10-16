//! The M6c AI-versus-AI game of the gate (`duel-m6c`): the own AI against
//! itself on the synthetic M6c map of `glasnost_ai::fixture_m6c`, whose
//! invented types make a plain game use every special ability of M6c. The
//! run counts, per ability, the orders the AI gave (`glasnost_ai::special::Uses`)
//! and the events the simulation then showed ([`Exercise`]), so that the
//! gate cannot go on passing after a change that makes an ability fall out of
//! the game. The counters are not part of the golden hashes; the full run
//! asserts that none is zero.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_ai::fixture_m6c as kit;
use glasnost_ai::game::Game;
use glasnost_ai::special::Uses;
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::world::{ObjectId, World};

/// What the simulation showed during a run, per ability of M6c.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exercise {
    /// The orders each ability was given, summed over the AI houses.
    pub uses: Uses,
    /// The most occupants that sat in garrisons at one tick (S-0508).
    pub occupants: u32,
    /// Buildings that changed owner by capture (S-0509; `Effect::Captured`).
    pub captures: u32,
    /// Infiltrations (S-0510; `Effect::Infiltrated`).
    pub infiltrations: u32,
    /// Income cycles of tech buildings held by a player house (S-0515).
    pub cash_cycles: u32,
    /// Ticks in which a jumpjet unit was above the ground (S-0502).
    pub flight_ticks: u32,
    /// Jumpjet units a factory built that rose into the air (the ascent
    /// that ends the factory contact, S-0417 claim 15).
    pub jumpjets_built: u32,
    /// Deploy and undeploy sounds, one per change of form (S-0517).
    pub deploy_changes: u32,
    /// Ticks in which some cell held radiation (S-0525).
    pub radiation_ticks: u32,
    /// Erasures begun (S-0514; a new lock on a target).
    pub erasures: u32,
    /// Support beams drawn from prism buildings (S-0518).
    pub support_beams: u32,
    /// Bombs planted (S-0519; a carrier that gained a bomb).
    pub bombs: u32,
    /// Air strikes called (S-0519; a designator whose strike began).
    pub strikes: u32,
    /// Crates a unit picked up before they expired (S-0520).
    pub crates: u32,
}

/// The sum of the orders of every AI house.
pub fn total(uses: &[(glasnost_sim::world::HouseId, Uses)]) -> Uses {
    let mut t = Uses::default();
    for (_, u) in uses {
        t.garrisons += u.garrisons;
        t.engineers += u.engineers;
        t.spies += u.spies;
        t.tech += u.tech;
        t.jumps += u.jumps;
        t.deploys += u.deploys;
        t.radiation += u.radiation;
        t.erasures += u.erasures;
        t.prisms += u.prisms;
        t.charges += u.charges;
        t.bombs += u.bombs;
        t.disarms += u.disarms;
        t.suicides += u.suicides;
        t.sabotages += u.sabotages;
        t.strikes += u.strikes;
        t.crates += u.crates;
    }
    t
}

impl Exercise {
    /// The names of the abilities that a full run did not exercise, with the
    /// counter that stayed zero (empty when the run exercised all of them).
    pub fn missing(&self) -> Vec<&'static str> {
        let u = &self.uses;
        let checks: [(&'static str, u32); 29] = [
            ("garrison orders (S-0508)", u.garrisons),
            ("garrison occupants inside (S-0508)", self.occupants),
            ("engineer orders (S-0509)", u.engineers),
            ("buildings captured (S-0509)", self.captures),
            ("spy orders (S-0510)", u.spies),
            ("infiltrations (S-0510)", self.infiltrations),
            ("tech building orders (S-0515)", u.tech),
            ("tech building income cycles (S-0515)", self.cash_cycles),
            ("jumpjet orders (S-0502)", u.jumps),
            ("jumpjet flight (S-0502)", self.flight_ticks),
            (
                "jumpjet units built and flown (S-0417 claim 15)",
                self.jumpjets_built,
            ),
            ("deploy orders (S-0517)", u.deploys),
            ("changes of deployed form (S-0517)", self.deploy_changes),
            ("radiation exits (S-0525)", u.radiation),
            ("radiation in the field (S-0525)", self.radiation_ticks),
            ("chrono erasure orders (S-0514)", u.erasures),
            ("erasures begun (S-0514)", self.erasures),
            ("prism buildings ordered (S-0518)", u.prisms),
            ("charger orders (S-0518)", u.charges),
            ("support beams (S-0518)", self.support_beams),
            ("bomb planter orders (S-0519)", u.bombs),
            ("bombs planted (S-0519)", self.bombs),
            ("disarm orders (S-0519)", u.disarms),
            ("suicide orders (S-0519)", u.suicides),
            ("sabotage orders (S-0519)", u.sabotages),
            ("air-strike orders (S-0519)", u.strikes),
            ("air strikes called (S-0519)", self.strikes),
            ("crate orders (S-0520)", u.crates),
            ("crates picked up (S-0520)", self.crates),
        ];
        checks
            .iter()
            .filter(|&&(_, n)| n == 0)
            .map(|&(name, _)| name)
            .collect()
    }
}

/// The per-tick observer of a run: remembers what the last tick looked like
/// so that it can tell a beginning from a continuation.
#[derive(Default)]
struct Watch {
    locks: BTreeSet<ObjectId>,
    bombed: BTreeSet<ObjectId>,
    striking: BTreeSet<ObjectId>,
    restarts: BTreeMap<ObjectId, (Option<glasnost_sim::world::HouseId>, Option<i32>)>,
    crates: BTreeMap<u16, (i32, i32)>,
    /// The objects of the first observed tick (the starting units).
    initial: Option<BTreeSet<ObjectId>>,
    /// Built jumpjet units seen in the air.
    flown: BTreeSet<ObjectId>,
}

/// Which kinds of M6c state have been seen so far in a run (a save and a
/// load happens at the first tick each one appears).
#[derive(Default)]
struct Seen {
    occupants: bool,
    flight: bool,
    radiation: bool,
    erasure: bool,
    bomb: bool,
    strike: bool,
    crate_place: bool,
}

impl Exercise {
    /// Notes what the tick just run showed (the effects of the tick are
    /// still in the world) and returns which kinds of state appeared for the
    /// first time.
    fn observe(&mut self, w: &World, watch: &mut Watch, seen: &mut Seen) -> bool {
        let mut first = false;
        for e in &w.effects {
            match e {
                Effect::Captured { .. } => self.captures += 1,
                Effect::Infiltrated { .. } => self.infiltrations += 1,
                Effect::SupportBeam { .. } => self.support_beams += 1,
                Effect::Sound {
                    sound: Sound::Deploy | Sound::Undeploy,
                    ..
                } => self.deploy_changes += 1,
                _ => {}
            }
        }
        // Garrisons: the most occupants inside at one tick.
        let inside: u32 = w
            .objects
            .values()
            .map(|o| u32::try_from(o.structure.occupants.len()).unwrap_or(u32::MAX))
            .sum();
        self.occupants = self.occupants.max(inside);
        if inside > 0 && !seen.occupants {
            seen.occupants = true;
            first = true;
        }
        // Jumpjets in the air.
        let flying = w.objects.values().any(|o| {
            o.on_map
                && glasnost_sim::jump::is_jumpjet(w, o.id)
                && glasnost_sim::jump::height_above(w, o.id) > 0
        });
        let initial = watch
            .initial
            .get_or_insert_with(|| w.objects.keys().copied().collect());
        for o in w.objects.values() {
            if o.on_map
                && !initial.contains(&o.id)
                && glasnost_sim::jump::is_jumpjet(w, o.id)
                && glasnost_sim::jump::height_above(w, o.id) > 0
                && watch.flown.insert(o.id)
            {
                self.jumpjets_built += 1;
            }
        }
        if flying {
            self.flight_ticks += 1;
            if !seen.flight {
                seen.flight = true;
                first = true;
            }
        }
        // Radiation in the field.
        if !w.radiation.cells.is_empty() {
            self.radiation_ticks += 1;
            if !seen.radiation {
                seen.radiation = true;
                first = true;
            }
        }
        // Erasures: a target that gained a lock.
        let locks: BTreeSet<ObjectId> = w.temporal.locks.keys().copied().collect();
        let begun = u32::try_from(locks.difference(&watch.locks).count()).unwrap_or(u32::MAX);
        self.erasures += begun;
        if begun > 0 && !seen.erasure {
            seen.erasure = true;
            first = true;
        }
        watch.locks = locks;
        // Bombs: a carrier that gained a bomb.
        let bombed: BTreeSet<ObjectId> = w
            .objects
            .values()
            .filter(|o| o.combat.bomb.is_some())
            .map(|o| o.id)
            .collect();
        let planted = u32::try_from(bombed.difference(&watch.bombed).count()).unwrap_or(u32::MAX);
        self.bombs += planted;
        if planted > 0 && !seen.bomb {
            seen.bomb = true;
            first = true;
        }
        watch.bombed = bombed;
        // Air strikes: a designator whose strike is in progress, newly.
        let striking: BTreeSet<ObjectId> = w
            .strike
            .strikes
            .iter()
            .filter(|(_, s)| s.progress)
            .map(|(&d, _)| d)
            .collect();
        let called =
            u32::try_from(striking.difference(&watch.striking).count()).unwrap_or(u32::MAX);
        self.strikes += called;
        if called > 0 && !seen.strike {
            seen.strike = true;
            first = true;
        }
        watch.striking = striking;
        // Tech buildings' income cycles: the countdown restarted without a
        // change of owner, in a building a player house holds.
        for o in w.objects.values() {
            if o.kind != glasnost_sim::rules::Kind::Building
                || !glasnost_sim::structures::tech::cash_producing(w, o.id)
            {
                continue;
            }
            let now = (o.owner, o.econ.building.income_restart);
            if let Some(&(owner, restart)) = watch.restarts.get(&o.id)
                && owner == now.0
                && restart != now.1
                && now
                    .0
                    .is_some_and(|h| !glasnost_sim::econ::setup::passive(w, h))
            {
                self.cash_cycles += 1;
            }
            watch.restarts.insert(o.id, now);
        }
        // Crates: a place that went before its time ran out was picked up.
        let places: BTreeMap<u16, (i32, i32)> = w
            .crates
            .places
            .iter()
            .map(|(&k, p)| (k, (p.start, p.life)))
            .collect();
        for (k, &(start, life)) in &watch.crates {
            // A place whose number is free or taken by a newer crate is gone;
            // an expiry is due when its lifetime has run out.
            if places.get(k) != Some(&(start, life)) && w.tick - start < life - 1 {
                self.crates += 1;
            }
        }
        if !places.is_empty() && !seen.crate_place {
            seen.crate_place = true;
            first = true;
        }
        watch.crates = places;
        first
    }
}

/// Plays the M6c duel for `ticks` ticks (it ends earlier when the game is
/// over). The game is saved and loaded halfway and at the first tick each
/// kind of M6c state appears (garrison occupants, a jumpjet in the air,
/// radiation, an erasure, a planted bomb, an air strike, a crate); the loaded
/// game must hash alike and goes on from there. Returns the state hash after
/// every tick, the final game and what the run exercised.
pub fn play(seed: u32, ticks: i32) -> (Vec<String>, Game, Exercise) {
    let rules = kit::rules();
    let catalogue = glasnost_ai::fixture::catalogue();
    let mut g = Game::new(
        rules.clone(),
        &kit::map(),
        kit::MAP_NAME,
        catalogue.clone(),
        kit::setup(seed),
    )
    .expect("the synthetic M6c duel loads");
    let mut out = Vec::new();
    let mut ex = Exercise::default();
    let (mut watch, mut seen) = (Watch::default(), Seen::default());
    let mut save_next = false;
    while g.world.tick < ticks {
        if g.world.tick == ticks / 2 || save_next {
            let loaded = Game::load(&g.save(), rules.clone(), catalogue.clone())
                .expect("the M6c duel's save loads");
            assert_eq!(
                loaded.state_hash(),
                g.state_hash(),
                "duel-m6c: a save and load keeps the state"
            );
            g = loaded;
        }
        g.step();
        out.push(g.state_hash());
        // Effects are presentation only (neither saved nor hashed).
        save_next = ex.observe(&g.world, &mut watch, &mut seen);
        g.world.effects.clear();
        if g.outcome().is_some() {
            break;
        }
    }
    ex.uses = total(&g.special_uses());
    (out, g, ex)
}
