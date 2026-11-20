//! A game with its AI houses: the simulation's world plus the state of our
//! own AI for every AI house of the setup (glossary *AI house*), stepped,
//! saved and hashed together.
//!
//! The AI runs inside the tick: [`Game::step`] lets every AI house that is
//! still in the game think at the tick boundary, after the person's orders
//! for that tick were issued and before the tick runs, and issues its
//! orders through the same `orders::issue` as a person's (the legality
//! rules and the pending-order limit apply, and a refused order is simply
//! not applied). Its state is part of the saved game and of the state
//! hash, so a save in mid-game, loaded and continued, plays on alike, and
//! a replay holds only the setup and the person's orders: replaying it runs
//! the AI again.

use std::sync::Arc;

use glasnost_map::Map;
use glasnost_map::terrain::TileCatalogue;
use glasnost_sim::econ::victory;
use glasnost_sim::load::{LoadError, new_game};
use glasnost_sim::orders::{self, Order};
use glasnost_sim::probe::{Phase, Probe};
use glasnost_sim::rules::SimRules;
use glasnost_sim::save::{self, Replay, SaveError};
use glasnost_sim::tick;
use glasnost_sim::world::{GameSetup, HouseId, HouseRole, World};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::brain::Brain;
use crate::scripts::Seat;

/// Format version of a game save (the world's own save inside it carries
/// the simulation's version).
// nocite: version number of our own save format
pub const GAME_FORMAT_VERSION: u32 = 147;

/// Bytes reserved beyond the world's save for the rest of a game save: a
/// first guess at the brains and the replay, not a limit.
// nocite: buffer size of our own save writer
const SAVE_ROOM: usize = 64 * 1024;

/// A running game.
pub struct Game {
    /// The simulation state.
    pub world: World,
    /// One seat per AI house, in house order: its script and difficulty
    /// and its script's state (#449).
    seats: Vec<Seat>,
    /// The setup and the person's orders.
    replay: Replay,
}

/// The saved form of the AI's part.
#[derive(Serialize, Deserialize)]
struct SavedGame {
    version: u32,
    world: Vec<u8>,
    seats: Vec<Seat>,
    replay: Replay,
}

/// [`SavedGame`] written from borrows: the same fields in the same order,
/// so the same bytes, without copying the seats and the replay (#51).
#[derive(Serialize)]
struct SavedGameRef<'a> {
    version: u32,
    world: Bytes<'a>,
    seats: &'a [Seat],
    replay: &'a Replay,
}

/// A byte string written with the serializer's byte form. In postcard that
/// is the length and the bytes, the same encoding as a sequence of `u8`, so
/// [`SavedGame`] reads it back as its `Vec<u8>`; it is written in one copy
/// instead of one call per byte (#131).
struct Bytes<'a>(&'a [u8]);

impl Serialize for Bytes<'_> {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_bytes(self.0)
    }
}

/// The AI houses of a setup.
fn ai_houses(setup: &GameSetup) -> Vec<HouseId> {
    setup
        .houses
        .iter()
        .enumerate()
        .filter(|(_, h)| h.ai && h.role == HouseRole::Player)
        .map(|(i, _)| HouseId(i as u8))
        .collect()
}

impl Game {
    /// Starts a game: the world from the setup, a seat for every AI house
    /// with the script and difficulty the setup gives it
    /// ([`GameSetup::ai_seat`]; #449).
    pub fn new(
        rules: Arc<SimRules>,
        map: &Map,
        map_name: &str,
        catalogue: Arc<TileCatalogue>,
        setup: GameSetup,
    ) -> Result<Self, LoadError> {
        let world = new_game(rules.clone(), map, catalogue, &setup)?;
        let seats = ai_houses(&setup)
            .into_iter()
            .filter(|&h| !world.houses[usize::from(h.0)].removed)
            .map(|h| Seat::start(setup.ai_seat(usize::from(h.0)), &world, map, h))
            .collect();
        let replay = Replay::new(&rules, map_name, setup);
        Ok(Self {
            world,
            seats,
            replay,
        })
    }

    /// Issues a person's order in the current tick; it is recorded in the
    /// replay when the simulation takes it.
    pub fn issue(&mut self, house: HouseId, order: Order) -> bool {
        let taken = orders::issue(&mut self.world, house, order.clone());
        if taken {
            self.replay.record(self.world.tick, house, order);
        }
        taken
    }

    /// Runs one tick: the AI houses think, then the simulation steps.
    pub fn step(&mut self) {
        self.step_with(&mut ());
    }

    /// [`Game::step`], telling `p` where the tick is (the tools' benchmark,
    /// `glasnost_sim::probe`); `p` reads nothing and changes nothing. Once
    /// the game is over (S-0414) a step does nothing, so the state stays the
    /// one its replay ends on.
    pub fn step_with<P: Probe>(&mut self, p: &mut P) {
        // design: own AI (#51): the simulation runs its final tick again when
        // stepped after it (the tick number stays), and the AI would think
        // again, while the replay's tick count stays put
        if self.outcome().is_some() {
            return;
        }
        let on = p.on();
        // design: own AI (AP1's seam): every house's own list from one pass
        // over the objects, made when the first brain thinks; the brains'
        // orders do not change the objects (`scripted::own_by_house`). The
        // bench counts the pass in that first brain's AI time.
        let mut own: Option<Vec<Vec<glasnost_sim::world::ObjectId>>> = None;
        for b in &mut self.seats {
            let h = b.house();
            if victory::defeated(&self.world, h) || self.world.houses[usize::from(h.0)].removed {
                continue;
            }
            if on {
                p.mark(Phase::Think(h.0));
            }
            let lists = own.get_or_insert_with(|| {
                if on {
                    p.mark(Phase::Part("ai:own_by_house"));
                }
                crate::scripted::own_by_house(&self.world)
            });
            let list = lists
                .get_mut(usize::from(h.0))
                .map(std::mem::take)
                .unwrap_or_default();
            let orders = b.think(&self.world, Some(list.into()), p);
            if on {
                p.mark(Phase::Issue);
            }
            for o in orders {
                orders::issue(&mut self.world, h, o);
            }
        }
        tick::step_with(&mut self.world, p);
        self.replay.ticks = tick::ticks_run(&self.world);
    }

    /// The game's result once it is over (S-0414).
    pub fn outcome(&self) -> Option<victory::Outcome> {
        victory::outcome(&self.world)
    }

    /// The special-ability orders each AI house has given (M6c), in house
    /// order of the brains.
    pub fn special_uses(&self) -> Vec<(HouseId, crate::special::Uses)> {
        self.brains()
            .into_iter()
            .map(|b| (b.house(), b.special_uses()))
            .collect()
    }

    /// A counts-only diagnostic line per AI house (local-lane reports).
    pub fn diagnose(&self) -> Vec<(HouseId, String)> {
        self.brains()
            .into_iter()
            .map(|b| (b.house(), b.diagnose(&self.world)))
            .collect()
    }

    /// What each brain's anti-air step sees now, in the order of the
    /// brains (local-lane reports).
    pub fn antiair_probes(&self) -> Vec<(HouseId, crate::antiair::Probe)> {
        self.brains()
            .into_iter()
            .map(|b| (b.house(), b.antiair_probe(&self.world)))
            .collect()
    }

    /// What each brain's expansion step sees now, in the order of the
    /// brains (local-lane reports).
    pub fn expansion_probes(&self) -> Vec<(HouseId, crate::brain::ExpansionProbe)> {
        self.brains()
            .into_iter()
            .map(|b| (b.house(), b.expansion_probe(&self.world)))
            .collect()
    }

    /// What each brain's economy sees now (`Brain::economy_probe`), in the
    /// order of the brains (local-lane reports).
    pub fn economy_probes(&self) -> Vec<String> {
        self.brains()
            .into_iter()
            .map(|b| b.economy_probe(&self.world))
            .collect()
    }

    /// What each brain's broke house waits behind now
    /// (`Brain::broke_probe`, M8 step B10), in the order of the brains
    /// (local-lane reports).
    pub fn broke_probes(&self) -> Vec<(HouseId, crate::brain::BrokeProbe)> {
        self.brains()
            .into_iter()
            .map(|b| (b.house(), b.broke_probe(&self.world)))
            .collect()
    }

    /// The AI houses.
    pub fn ai_houses(&self) -> Vec<HouseId> {
        self.seats.iter().map(Seat::house).collect()
    }

    /// The AI houses' seats, in house order: script, difficulty and the
    /// script's state (read only: the desync tool's field-by-field view,
    /// M9 N4; #449).
    pub fn seats(&self) -> &[Seat] {
        &self.seats
    }

    /// The brains of the AI houses the default script plays, in house
    /// order (read only: the local-lane reports and the tests).
    pub fn brains(&self) -> Vec<&Brain> {
        self.seats
            .iter()
            .filter_map(|s| s.script().brain())
            .collect()
    }

    /// The AI's state as hashed: with every seat on the default script at
    /// normal, the brains alone, so that such a game hashes as it did
    /// before the seats existed (#449); else the seats with their choices.
    fn ai_state_bytes(&self) -> Vec<u8> {
        if self.seats.iter().all(|s| s.choice().is_default()) {
            postcard::to_allocvec(&self.brains())
        } else {
            postcard::to_allocvec(&self.seats)
        }
        .expect("the AI state serializes")
    }

    /// The replay so far: the setup and the person's orders.
    pub fn replay(&self) -> &Replay {
        &self.replay
    }

    /// The state hash of the world and the AI's state together.
    pub fn state_hash(&self) -> String {
        let mut h = Sha256::new();
        h.update(save::state_hash(&self.world).as_bytes());
        h.update(self.ai_state_bytes());
        h.finalize().iter().fold(String::new(), |mut s, b| {
            use std::fmt::Write as _;
            let _ = write!(s, "{b:02x}"); // nocite: hexadecimal text of a digest
            s
        })
    }

    /// Saves the game.
    pub fn save(&self) -> Vec<u8> {
        let world = save::save(&self.world);
        // The world's bytes and room for the rest, so the output grows
        // rarely (#131).
        let room = world.len() + SAVE_ROOM;
        postcard::to_extend(
            &SavedGameRef {
                version: GAME_FORMAT_VERSION,
                world: Bytes(&world),
                seats: &self.seats,
                replay: &self.replay,
            },
            Vec::with_capacity(room),
        )
        .expect("the game serializes")
    }

    /// Loads a game saved by [`Game::save`] under the same rules.
    pub fn load(
        bytes: &[u8],
        rules: Arc<SimRules>,
        catalogue: Arc<TileCatalogue>,
    ) -> Result<Self, SaveError> {
        let s: SavedGame =
            postcard::from_bytes(bytes).map_err(|e| SaveError::Format(e.to_string()))?;
        if s.version != GAME_FORMAT_VERSION {
            return Err(SaveError::Format(format!(
                "game format version {}",
                s.version
            )));
        }
        let world = save::load(&s.world, rules, catalogue)?;
        Ok(Self {
            world,
            seats: s.seats,
            replay: s.replay,
        })
    }

    /// Replays a game from its replay up to its last tick, calling `each`
    /// after every tick: the person's orders as recorded, the AI anew.
    pub fn run_replay(
        replay: &Replay,
        rules: Arc<SimRules>,
        map: &Map,
        catalogue: Arc<TileCatalogue>,
        mut each: impl FnMut(&Game),
    ) -> Result<Self, SaveError> {
        if replay.rules_hash != rules.hash {
            return Err(SaveError::RulesMismatch {
                saved: replay.rules_hash.clone(),
                given: rules.hash.clone(),
            });
        }
        let mut g = Self::new(rules, map, &replay.map, catalogue, replay.setup.clone())
            .map_err(SaveError::Load)?;
        // spec: S-0201 claim 1 (the final tick leaves the tick number at itself: count the ticks run)
        while tick::ticks_run(&g.world) < replay.ticks {
            g.issue_recorded(replay);
            g.step();
            each(&g);
        }
        Ok(g)
    }

    /// Issues the orders `replay` recorded for the current tick.
    fn issue_recorded(&mut self, replay: &Replay) {
        let tick = self.world.tick;
        let from = replay.orders.partition_point(|o| o.0 < tick);
        for (t, h, o) in &replay.orders[from..] {
            if *t != tick {
                break;
            }
            self.issue(*h, o.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixture;

    /// The save written before #131: the world's bytes as a `Vec<u8>`.
    #[derive(Serialize)]
    struct OldRef<'a> {
        version: u32,
        world: Vec<u8>,
        seats: &'a [Seat],
        replay: &'a Replay,
    }

    /// The cheaper save writes the same bytes as the old one, and they load.
    #[test]
    fn save_bytes_unchanged_by_the_byte_form() {
        let mut g = Game::new(
            fixture::rules(),
            &fixture::map(),
            "synthetic-skirmish",
            fixture::catalogue(),
            fixture::setup(5),
        )
        .expect("the synthetic skirmish loads");
        for _ in 0..200 {
            g.step();
        }
        let old = postcard::to_allocvec(&OldRef {
            version: GAME_FORMAT_VERSION,
            world: save::save(&g.world),
            seats: &g.seats,
            replay: &g.replay,
        })
        .expect("the game serializes");
        let new = g.save();
        assert!(old == new, "the save's bytes changed");
        let back = Game::load(&new, fixture::rules(), fixture::catalogue()).expect("it loads");
        assert_eq!(back.state_hash(), g.state_hash());
    }
}
