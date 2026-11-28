//! Pluggable AI scripts (#449): the contract every script of our own AI
//! keeps, the registry of the scripts compiled in, and the seat that holds
//! one AI house's script, its choice and its state.
//!
//! **The contract** ([`AiScript`]). A script is given, at the start of each
//! tick, the house's [`View`] and the think [`Schedule`], and the difficulty
//! its house was given; it returns orders, in the order to issue them. The
//! orders go through the same `orders::issue` as a person's, so a script
//! can only give orders a person could give (the legality rules and the
//! pending-order limit apply; a refused order is simply not applied). It
//! acts only on what a person in its seat sees: its own and its allies'
//! objects, and other objects where the house has explored and no cloak
//! hides them ([`View::sees`], [`crate::in_view`]). The view reaches the
//! world for the simulation's own answers (legality, placement, the rules,
//! the house's own money and lines); a script asks [`View::sees`] before it
//! acts on anything of another house. The shroud test of the AI suite
//! checks every registry entry at every difficulty for it: two worlds that
//! differ only by an enemy unit on a cell the house has not explored give
//! the same orders.
//!
//! **Difficulty** changes only how well a script plays. It never changes a
//! house's rules: every player house, a person's or an AI house, uses the
//! normal difficulty row unless its handicap picks another, and the
//! handicap is never set by AI difficulty (glossary *difficulty skill
//! block*; S-0412 claim 5; S-0301 claim 3; S-0302 Terms and claim 3).
//!
//! **State.** A script's state is serialised with the game and hashed with
//! the AI's state; it must be deterministic (integers only, ordered
//! containers, no I/O), as the rest of this crate.
//!
//! **The registry** ([`REGISTRY`]) is fixed, ordered and compiled in: each
//! entry has a stable id (stored in setups, saves and replays; never
//! reused), a key for the tools and a display name. Script 1 is the default
//! script, the brain of [`crate::brain`]. A setup naming an id the build
//! does not know is played by the default script (a setup made by a newer
//! build).

use std::sync::Arc;

use glasnost_map::Map;
use glasnost_sim::orders::Order;
use glasnost_sim::probe::Probe;
use glasnost_sim::world::{AiDifficulty, AiSeat, HouseId, ObjectId, World};
use serde::{Deserialize, Serialize};

use crate::brain::{self, Brain};
use crate::siege::{self, Siege};
use crate::vanguard::{self, Vanguard};

/// What a script is given of the world: the house's seat. See the module's
/// contract.
#[derive(Clone, Copy)]
pub struct View<'a> {
    world: &'a World,
    house: HouseId,
}

impl<'a> View<'a> {
    /// House `house`'s view of `world`.
    pub fn new(world: &'a World, house: HouseId) -> Self {
        Self { world, house }
    }

    /// The house.
    pub fn house(&self) -> HouseId {
        self.house
    }

    /// The tick about to run.
    pub fn tick(&self) -> i32 {
        self.world.tick
    }

    /// Whether the house sees object `id` where it stands, as a person in
    /// its seat does ([`crate::in_view`]).
    pub fn sees(&self, id: ObjectId) -> bool {
        crate::in_view(self.world, self.house, id)
    }

    /// Every object the house sees, in id order.
    pub fn seen(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.world
            .objects
            .keys()
            .copied()
            .filter(move |&id| self.sees(id))
    }

    /// The world, for the simulation's own answers (legality, placement,
    /// the rules, the house's own state). Anything of another house is
    /// acted on only after [`View::sees`].
    pub fn world(&self) -> &'a World {
        self.world
    }
}

/// The think schedule of a tick for one house: every script may give
/// orders in every tick; its plan thinks on the house's think ticks
/// ([`crate::scripted::thinks_at`]: every
/// [`THINK_PERIOD`](crate::scripted::THINK_PERIOD) ticks, at the house's
/// phase), so that the houses' thinks are spread over the period.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Schedule {
    /// The tick about to run.
    pub tick: i32,
    /// Whether it is one of the house's think ticks.
    pub thinks: bool,
}

impl Schedule {
    /// House `house`'s schedule in tick `tick`.
    pub fn of(house: HouseId, tick: i32) -> Self {
        Self {
            tick,
            thinks: crate::scripted::thinks_at(house, tick),
        }
    }
}

/// One AI script: see the module's contract.
pub trait AiScript {
    /// The house it plays.
    fn house(&self) -> HouseId;

    /// The orders for the tick about to run, in the order to issue them.
    /// `own` is the house's own objects when the caller has listed them
    /// (`scripted::own_by_house`), else the script finds them; `p` is told
    /// which part runs (the tools' benchmark) and changes nothing.
    fn think<P: Probe>(
        &mut self,
        view: &View<'_>,
        schedule: Schedule,
        level: AiDifficulty,
        own: Option<Arc<[ObjectId]>>,
        p: &mut P,
    ) -> Vec<Order>;
}

impl AiScript for Brain {
    fn house(&self) -> HouseId {
        Brain::house(self)
    }

    fn think<P: Probe>(
        &mut self,
        view: &View<'_>,
        schedule: Schedule,
        level: AiDifficulty,
        own: Option<Arc<[ObjectId]>>,
        p: &mut P,
    ) -> Vec<Order> {
        self.think_scheduled(view.world(), p, own, schedule.thinks, level)
    }
}

/// One entry of the registry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    /// The stable id, as setups, saves and replays store it.
    pub id: u8,
    /// The key the tools take (`ai-tournament`).
    pub key: &'static str,
    /// The name the setup screen shows.
    pub name: &'static str,
    /// One line on how it plays.
    pub about: &'static str,
}

/// The scripts compiled in, in the order the setup screen offers them.
// nocite: design: own AI (#449): the registry of our own scripts
pub const REGISTRY: &[Entry] = &[
    Entry {
        id: AiSeat::DEFAULT_SCRIPT,
        key: "default",
        name: "Default",
        about: "the brain of glasnost_ai::brain: a build plan by role, harvesters kept, waves sent at the nearest seen enemy, the special abilities",
    },
    Entry {
        id: siege::ID,
        key: "siege",
        name: "Siege",
        about: "glasnost_ai::siege: the default's base without superweapons, one army gathered and sent as a strike at the enemy's economy and base when it has the edge, called home when outmatched, then a hunt for the last units",
    },
    Entry {
        id: vanguard::ID,
        key: "vanguard",
        name: "Vanguard",
        about: "glasnost_ai::vanguard: rules-adaptive economy and vehicle scoring, active scouting, smart air sniping, forward staging, and sim-safe focus-fire, pullback and standoff micro",
    },
];

/// The registry entry of id `id`.
pub fn entry(id: u8) -> Option<&'static Entry> {
    REGISTRY.iter().find(|e| e.id == id)
}

/// The registry entry of key `key`.
pub fn by_key(key: &str) -> Option<&'static Entry> {
    REGISTRY.iter().find(|e| e.key.eq_ignore_ascii_case(key))
}

/// `seat` with a script id the build knows: an unknown id is the default
/// script's (see the module's registry note).
pub fn known(seat: AiSeat) -> AiSeat {
    AiSeat {
        script: entry(seat.script).map_or(AiSeat::DEFAULT_SCRIPT, |e| e.id),
        ..seat
    }
}

/// A script's state, by registry entry (the enum dispatch of [`AiScript`]).
#[derive(Clone, Debug, Serialize, Deserialize)]
// The default's brain stays unboxed so that its seats are laid out and run
// as before; the siege's state, which holds a brain and more, is boxed.
#[allow(clippy::large_enum_variant)]
pub enum ScriptState {
    /// Script 1, the default: [`Brain`].
    Default(Brain),
    /// Script 2, the siege: [`Siege`].
    Siege(Box<Siege>),
    /// Script 3, the vanguard: [`Vanguard`].
    Vanguard(Box<Vanguard>),
}

impl ScriptState {
    /// The state a script starts a game with for house `house`, at the
    /// seat's difficulty.
    pub fn start(seat: AiSeat, w: &World, map: &Map, house: HouseId) -> Self {
        // A new script adds its id's arm here; every id the build does not
        // know, and the default's, is the default script.
        debug_assert!(entry(known(seat).script).is_some());
        match known(seat).script {
            siege::ID => Self::Siege(Box::new(Siege::start(w, map, house, seat.difficulty))),
            vanguard::ID => {
                Self::Vanguard(Box::new(Vanguard::start(w, map, house, seat.difficulty)))
            }
            _ => Self::Default(Brain::with_plan(
                w,
                map,
                house,
                brain::plan_for(seat.difficulty),
            )),
        }
    }

    /// The script's registry id.
    pub fn id(&self) -> u8 {
        match self {
            Self::Default(_) => AiSeat::DEFAULT_SCRIPT,
            Self::Siege(_) => siege::ID,
            Self::Vanguard(_) => vanguard::ID,
        }
    }

    /// The default script's brain, when this is it.
    pub fn brain(&self) -> Option<&Brain> {
        match self {
            Self::Default(b) => Some(b),
            Self::Siege(_) | Self::Vanguard(_) => None,
        }
    }

    /// The siege script's state, when this is it.
    pub fn siege(&self) -> Option<&Siege> {
        match self {
            Self::Siege(s) => Some(s),
            Self::Default(_) | Self::Vanguard(_) => None,
        }
    }

    /// The vanguard script's state, when this is it.
    pub fn vanguard(&self) -> Option<&Vanguard> {
        match self {
            Self::Vanguard(v) => Some(v),
            Self::Default(_) | Self::Siege(_) => None,
        }
    }
}

impl AiScript for ScriptState {
    fn house(&self) -> HouseId {
        match self {
            Self::Default(b) => AiScript::house(b),
            Self::Siege(s) => AiScript::house(&**s),
            Self::Vanguard(v) => AiScript::house(&**v),
        }
    }

    fn think<P: Probe>(
        &mut self,
        view: &View<'_>,
        schedule: Schedule,
        level: AiDifficulty,
        own: Option<Arc<[ObjectId]>>,
        p: &mut P,
    ) -> Vec<Order> {
        match self {
            Self::Default(b) => AiScript::think(b, view, schedule, level, own, p),
            Self::Siege(s) => AiScript::think(&mut **s, view, schedule, level, own, p),
            Self::Vanguard(v) => AiScript::think(&mut **v, view, schedule, level, own, p),
        }
    }
}

/// One AI house's seat: its choice (script and difficulty) and its
/// script's state, saved and hashed with the game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Seat {
    choice: AiSeat,
    script: ScriptState,
}

impl Seat {
    /// The seat of house `house` at the start of a game, for `choice` (an
    /// unknown script id is the default script's).
    pub fn start(choice: AiSeat, w: &World, map: &Map, house: HouseId) -> Self {
        let choice = known(choice);
        Self {
            choice,
            script: ScriptState::start(choice, w, map, house),
        }
    }

    /// The house.
    pub fn house(&self) -> HouseId {
        self.script.house()
    }

    /// The choice the house plays with.
    pub fn choice(&self) -> AiSeat {
        self.choice
    }

    /// The script's state.
    pub fn script(&self) -> &ScriptState {
        &self.script
    }

    /// The orders for the tick about to run in `w`, at the seat's
    /// difficulty and on the house's schedule.
    pub fn think<P: Probe>(
        &mut self,
        w: &World,
        own: Option<Arc<[ObjectId]>>,
        p: &mut P,
    ) -> Vec<Order> {
        let h = self.house();
        let view = View::new(w, h);
        self.script.think(
            &view,
            Schedule::of(h, w.tick),
            self.choice.difficulty,
            own,
            p,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The registry: ids unique and stable, the default script first.
    #[test]
    fn the_registry_is_ordered_and_stable() {
        assert_eq!(REGISTRY[0].id, AiSeat::DEFAULT_SCRIPT);
        assert_eq!(REGISTRY[0].key, "default");
        for (i, a) in REGISTRY.iter().enumerate() {
            for b in &REGISTRY[i + 1..] {
                assert_ne!(a.id, b.id, "ids are unique");
                assert_ne!(a.key, b.key, "keys are unique");
            }
        }
        assert_eq!(by_key("DEFAULT").map(|e| e.id), Some(1));
        assert_eq!(
            known(AiSeat {
                script: 200,
                difficulty: AiDifficulty::Hard
            }),
            AiSeat {
                script: AiSeat::DEFAULT_SCRIPT,
                difficulty: AiDifficulty::Hard
            }
        );
    }
}
