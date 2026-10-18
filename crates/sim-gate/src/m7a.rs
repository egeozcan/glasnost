//! The M7a AI-versus-AI game of the gate (`duel-m7a`): the own AI against
//! itself on the synthetic M7a map of `glasnost_ai::fixture_m7a`, where a
//! Yuri-style side (slave miners for its economy) meets a fixture side, and
//! both have the psychic weapons of S-0610; the Yuri-style side also has a
//! gattling vehicle and defence (S-0609) whose top-stage weapon leaves gas
//! clouds (S-0613) and a mind controller (S-0601), and its power plant
//! absorbs infantry (S-0605); it also builds a grinder (S-0603); both
//! sides build tank bunkers and send vehicles in (S-0606); the Yuri-style
//! side builds magnetic lifters that seize and drop the other side's
//! vehicles (S-0607).
//! The run counts, per ability, the
//! orders the AI gave (`glasnost_ai::special::Uses`) and the events the
//! simulation then showed ([`Exercise`]), so that the gate cannot go on
//! passing after a change that makes an ability fall out of the game. The
//! counters are not part of the golden hashes; the full run asserts that
//! none is zero.
//!
//! To add a counter: a field in [`Exercise`], its line in
//! [`Exercise::counters`] (and the array's length), and its arm in
//! `Exercise::observe`; a kind of state whose first appearance deserves a
//! save and load calls `first(..)` there with a new name.

use std::collections::{BTreeMap, BTreeSet};

use glasnost_ai::fixture_m7a as kit;
use glasnost_ai::game::Game;
use glasnost_ai::special::Uses;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::SwEvent;
use glasnost_sim::world::{HouseId, ObjectId, World};

/// What the simulation showed during a run, per ability of M7a.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exercise {
    /// The orders each ability was given, summed over the AI houses.
    pub uses: Uses,
    /// Units that entered the berserk state (S-0610 claim 4).
    pub berserk_hits: u32,
    /// Ticks in which a house saw a hostile-intent line through its
    /// psychic sensor (S-0610 claims 9 to 11).
    pub intent_ticks: u32,
    /// Psychic reveals fired (S-0610 claims 13 and 14; the activation
    /// sound).
    pub reveals: u32,
    /// Slave-miner buildings that appeared, placed or deployed (S-0402
    /// claim 14).
    pub slave_bases: u32,
    /// Ticks in which a slave was out of its master (S-0402 claim 14).
    pub slave_ticks: u32,
    /// Gattling stage rises (S-0609 claim 2).
    pub stage_rises: u32,
    /// Ticks in which a gattling stood at its top stage (S-0609 claim 2).
    pub top_ticks: u32,
    /// Ticks in which a gattling building had spun up (S-0609 claims 7
    /// and 8; only the AI builds one).
    pub building_spin_ticks: u32,
    /// Takeovers: control links that formed (S-0601 claim 6).
    pub takeovers: u32,
    /// Gas clouds that appeared: a warhead's particle system or a virus
    /// death's animation (S-0613 claims 2 to 4). The gas weapon is an
    /// ordinary weapon to the AI, which gives its carriers attack orders
    /// like any other, so there is no gas order to count in [`Uses`].
    pub gas_clouds: u32,
    /// Hits that gas clouds dealt (S-0613 claim 13).
    pub gas_hits: u32,
    /// Infantry that went into an absorbing building (S-0605 claims 1
    /// and 7; Alpha's power plant BIOP is one).
    pub absorbed: u32,
    /// Units a recycling building consumed (S-0603 claims 5 and 8; only
    /// Alpha builds one, GRND).
    pub ground: u32,
    /// Vehicles that became bunkered (S-0606 claim 7; both sides build the
    /// bunker BUNK).
    pub bunkered: u32,
    /// Vehicles that became seized (S-0607 claim 5; both sides build the
    /// lifter MAGN).
    pub seized: u32,
    /// Seized units that became dropped: their grip ended in the air
    /// (S-0607 claims 8, 12 and 13).
    pub drops: u32,
}

impl Exercise {
    /// Every counter the full run asserts on, by name.
    pub fn counters(&self) -> [(&'static str, u32); 26] {
        let u = &self.uses;
        [
            ("berserk weapon orders (S-0610)", u.berserks),
            ("units made berserk (S-0610 claim 4)", self.berserk_hits),
            ("psychic sensor orders (S-0610 claim 9)", u.sensors),
            (
                "hostile intent seen by a sensor (S-0610 claims 9 to 11)",
                self.intent_ticks,
            ),
            ("psychic reveal orders (S-0610 claim 13)", u.reveals),
            (
                "psychic reveals fired (S-0610 claims 13 and 14)",
                self.reveals,
            ),
            (
                "slave-miner buildings placed or deployed (S-0402 claim 14)",
                self.slave_bases,
            ),
            ("slaves at work (S-0402 claim 14)", self.slave_ticks),
            ("gattling stage rises (S-0609 claim 2)", self.stage_rises),
            (
                "gattling at its top stage (S-0609 claim 2)",
                self.top_ticks,
            ),
            ("takeover orders (S-0601)", u.takeovers),
            ("takeovers (S-0601 claim 6)", self.takeovers),
            ("gas clouds (S-0613 claims 2 to 4)", self.gas_clouds),
            ("gas hits (S-0613 claim 13)", self.gas_hits),
            ("absorber entry orders (S-0605)", u.absorbs),
            ("infantry absorbed (S-0605 claims 1 and 7)", self.absorbed),
            ("grinder entry orders (S-0603)", u.grinds),
            ("units ground (S-0603 claims 5 and 8)", self.ground),
            ("tank bunker build orders (S-0606)", u.bunker_builds),
            ("bunker entry orders (S-0606 claim 3)", u.bunker_entries),
            ("vehicles bunkered (S-0606 claim 7)", self.bunkered),
            ("bunker exit orders (S-0606 claim 12)", u.bunker_exits),
            ("lifter build orders (S-0607)", u.lifter_builds),
            ("lift orders (S-0607)", u.lift_orders),
            ("vehicles seized (S-0607 claim 5)", self.seized),
            ("seized units dropped (S-0607 claim 13)", self.drops),
        ]
    }

    /// The names of the counters that a full run left at zero (empty when
    /// the run exercised all of them).
    pub fn missing(&self) -> Vec<&'static str> {
        self.counters()
            .iter()
            .filter(|&&(_, n)| n == 0)
            .map(|&(name, _)| name)
            .collect()
    }
}

/// The per-tick observer of a run: what the last tick looked like, so that
/// a beginning can be told from a continuation, and which kinds of state
/// have been seen (a save and a load happens at the first tick each one
/// appears).
#[derive(Default)]
struct Watch {
    berserk: BTreeSet<ObjectId>,
    slave_bases: BTreeSet<ObjectId>,
    victims: BTreeSet<(ObjectId, ObjectId)>,
    stages: BTreeMap<ObjectId, i32>,
    clouds: BTreeSet<ObjectId>,
    absorbed: BTreeSet<ObjectId>,
    bunkered: BTreeSet<ObjectId>,
    seized: BTreeSet<ObjectId>,
    dropped: BTreeSet<ObjectId>,
    seen: BTreeSet<&'static str>,
}

impl Watch {
    /// Notes that a kind of state is present; returns whether it is new.
    fn first(&mut self, present: bool, kind: &'static str) -> bool {
        present && self.seen.insert(kind)
    }
}

/// The objects that newly meet `now` compared with `before`.
fn began(before: &BTreeSet<ObjectId>, now: &BTreeSet<ObjectId>) -> u32 {
    u32::try_from(now.difference(before).count()).unwrap_or(u32::MAX)
}

impl Exercise {
    /// Notes what the tick just run showed (the effects of the tick are
    /// still in the world) and returns whether a kind of state appeared for
    /// the first time.
    fn observe(&mut self, w: &World, watch: &mut Watch) -> bool {
        let mut first = false;
        // Psychic reveals: the activation sound of each launch.
        let fired = w
            .effects
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    Effect::Superweapon {
                        event: SwEvent::RevealSound,
                        ..
                    }
                )
            })
            .count();
        self.reveals += u32::try_from(fired).unwrap_or(u32::MAX);
        first |= watch.first(fired > 0, "reveal");
        // Berserk: units that entered the state.
        let berserk: BTreeSet<ObjectId> = w
            .objects
            .values()
            .filter(|o| o.on_map && o.combat.berserk.is_some())
            .map(|o| o.id)
            .collect();
        self.berserk_hits += began(&watch.berserk, &berserk);
        first |= watch.first(!berserk.is_empty(), "berserk");
        watch.berserk = berserk;
        // Psychic sensors: a house that sees a hostile-intent line.
        let houses = u8::try_from(w.houses.len()).unwrap_or(u8::MAX);
        let sensing = (0..houses).map(HouseId).any(|h| {
            !glasnost_sim::psychic::sensors(w, h).is_empty()
                && !glasnost_sim::psychic::intent_lines(w, h).is_empty()
        });
        self.intent_ticks += u32::from(sensing);
        first |= watch.first(sensing, "intent");
        // Slave miners: their buildings and their slaves at work.
        let smb = w.rules().index_of(Kind::Building, "SMB");
        let bases: BTreeSet<ObjectId> = w
            .objects
            .values()
            .filter(|o| o.on_map && o.kind == Kind::Building && Some(o.type_index) == smb)
            .map(|o| o.id)
            .collect();
        self.slave_bases += began(&watch.slave_bases, &bases);
        first |= watch.first(!bases.is_empty(), "slave base");
        watch.slave_bases = bases;
        let out = w.objects.values().any(|o| {
            o.on_map && o.kind == Kind::Infantry && glasnost_sim::econ::slaves::is_slave(w, o.id)
        });
        self.slave_ticks += u32::from(out);
        first |= watch.first(out, "slave out");
        // Gattlings: stage rises, a tick in which one stands at its top
        // stage, and a tick in which a gattling building has spun up.
        let stages: BTreeMap<ObjectId, i32> = w
            .objects
            .values()
            .filter(|o| o.on_map && glasnost_sim::gattling::flagged(w, o.id))
            .map(|o| (o.id, o.combat.gattling_stage))
            .collect();
        let rises = stages
            .iter()
            .filter(|&(id, &s)| s > watch.stages.get(id).copied().unwrap_or(0))
            .count();
        self.stage_rises += u32::try_from(rises).unwrap_or(u32::MAX);
        watch.stages = stages;
        let top = w
            .objects
            .values()
            .any(|o| o.on_map && glasnost_sim::gattling::at_top(w, o.id));
        self.top_ticks += u32::from(top);
        first |= watch.first(top, "gattling top");
        let building = w.objects.values().any(|o| {
            o.on_map
                && o.kind == Kind::Building
                && o.combat.spin > 0
                && glasnost_sim::gattling::flagged(w, o.id)
        });
        self.building_spin_ticks += u32::from(building);
        first |= watch.first(building, "gattling building");
        // Mind control: links that formed in this tick.
        let victims: BTreeSet<(ObjectId, ObjectId)> =
            w.mind.victims.iter().map(|(&v, r)| (v, r.controller)).collect();
        self.takeovers +=
            u32::try_from(victims.difference(&watch.victims).count()).unwrap_or(u32::MAX);
        first |= watch.first(!victims.is_empty(), "takeover");
        watch.victims = victims;
        // Gas clouds: warhead-made gas systems and virus-death animations
        // that appeared (the shared system is there from the start; systems
        // of other kinds are no clouds), and the hits clouds dealt.
        let clouds: BTreeSet<ObjectId> = w
            .gas
            .systems
            .keys()
            .copied()
            .filter(|&id| glasnost_sim::gas::is_cloud(w, id))
            .chain(w.gas.anims.keys().copied())
            .collect();
        self.gas_clouds += began(&watch.clouds, &clouds);
        watch.clouds = clouds;
        let hits = w
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::GasHit { .. }))
            .count();
        self.gas_hits += u32::try_from(hits).unwrap_or(u32::MAX);
        let floating = glasnost_sim::gas::particles(w).next().is_some();
        first |= watch.first(floating, "gas cloud");
        // Absorbing buildings: passengers that went in during this tick.
        let inside: BTreeSet<ObjectId> = w
            .objects
            .values()
            .filter(|o| glasnost_sim::structures::absorb::absorbed_in(w, o.id).is_some())
            .map(|o| o.id)
            .collect();
        self.absorbed += began(&watch.absorbed, &inside);
        first |= watch.first(!inside.is_empty(), "absorbed");
        watch.absorbed = inside;
        // Recycling buildings: units bound for one and units consumed.
        let ground = w
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::Consumed { .. }))
            .count();
        self.ground += u32::try_from(ground).unwrap_or(u32::MAX);
        first |= watch.first(ground > 0, "ground");
        let bound = w.objects.values().any(|o| {
            o.on_map
                && o.beh.current == Some(glasnost_sim::rules::Behaviour::Eaten)
                && matches!(o.dest, Some(glasnost_sim::world::Target::Object(b)) if glasnost_sim::structures::grinder::is_recycler(w, b))
        });
        first |= watch.first(bound, "bound for a grinder");
        // Tank bunkers: vehicles that became bunkered during this tick.
        let held: BTreeSet<ObjectId> = w
            .objects
            .values()
            .filter(|o| glasnost_sim::structures::bunker::bunkered(w, o.id))
            .map(|o| o.id)
            .collect();
        self.bunkered += began(&watch.bunkered, &held);
        first |= watch.first(!held.is_empty(), "bunkered");
        watch.bunkered = held;
        // Magnetic lift: units that became seized, and seized units that
        // became dropped, during this tick.
        let seized: BTreeSet<ObjectId> = w
            .objects
            .values()
            .filter(|o| glasnost_sim::lift::seized(w, o.id))
            .map(|o| o.id)
            .collect();
        self.seized += began(&watch.seized, &seized);
        first |= watch.first(!seized.is_empty(), "seized");
        watch.seized = seized;
        let dropped: BTreeSet<ObjectId> = w
            .objects
            .values()
            .filter(|o| glasnost_sim::lift::dropped(w, o.id))
            .map(|o| o.id)
            .collect();
        self.drops += began(&watch.dropped, &dropped);
        first |= watch.first(!dropped.is_empty(), "dropped");
        watch.dropped = dropped;
        first
    }
}

/// Plays the M7a duel for `ticks` ticks (it ends earlier when the game is
/// over). The game is saved and loaded halfway and at the first tick each
/// kind of M7a state appears (a psychic reveal, a berserk unit, a sensed
/// intent line, a slave-miner building, a slave at work, a gattling at its
/// top stage, a controlled object, a gas particle, an absorbed passenger, a unit bound for a grinder,
/// a consumption, a bunkered vehicle, a seized unit, a dropped unit); the loaded game
/// must hash alike and goes on from there. Returns the state hash after
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
    .expect("the synthetic M7a duel loads");
    let mut out = Vec::new();
    let mut ex = Exercise::default();
    let mut watch = Watch::default();
    let mut save_next = false;
    while g.world.tick < ticks {
        if g.world.tick == ticks / 2 || save_next {
            let loaded = Game::load(&g.save(), rules.clone(), catalogue.clone())
                .expect("the M7a duel's save loads");
            assert_eq!(
                loaded.state_hash(),
                g.state_hash(),
                "duel-m7a: a save and load keeps the state"
            );
            g = loaded;
        }
        g.step();
        out.push(g.state_hash());
        // Effects are presentation only (neither saved nor hashed).
        save_next = ex.observe(&g.world, &mut watch);
        g.world.effects.clear();
        if g.outcome().is_some() {
            break;
        }
    }
    ex.uses = crate::m6c::total(&g.special_uses());
    (out, g, ex)
}
