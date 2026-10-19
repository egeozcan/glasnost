//! The M7b AI-versus-AI game of the gate (`duel-m7b`): the own AI against
//! itself on the synthetic M7b map of `glasnost_ai::fixture_m7b`, which is
//! the M7a duel ([`crate::m7a`]) with a drain (S-0608): the Yuri-style side
//! starts with hovering discs whose draining weapon links them with the
//! other side's power plants and refineries, and builds more.
//!
//! The run counts the drain orders the AI gave and the drain events the
//! simulation then showed ([`Exercise`]), besides everything the M7a duel
//! counts. The counters are not part of the golden hashes; the full run
//! asserts that none of the drain counters is below [`MARGIN`] (the M7a
//! counters are reported, not asserted: `duel-m7a` asserts them).
//!
//! To add a counter: a field in [`Exercise`], its line in
//! [`Exercise::counters`] (and the array's length), and its arm in
//! `Exercise::observe`.

use glasnost_ai::fixture_m7b as kit;
use glasnost_ai::game::Game;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::drain::DrainEvent;
use glasnost_sim::world::World;

use crate::m7a;

/// The least value the full run requires of each drain counter.
pub const MARGIN: u32 = 3;

/// What the simulation showed during a run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exercise {
    /// Everything the M7a duel counts (its `uses` are the M7b duel's,
    /// drain orders included).
    pub m7a: m7a::Exercise,
    /// Drain links that formed (S-0608 claims 2 and 5).
    pub links: u32,
    /// Credits that payouts moved (S-0608 claims 7 to 10).
    pub credits_drained: u32,
    /// The sum over ticks of the buildings drained at the tick's end
    /// (S-0608 claim 1).
    pub drained_ticks: u32,
    /// Drain links that ended (S-0608 claim 14).
    pub ends: u32,
    /// The first tick at whose end a drain link existed.
    pub first_link: Option<i32>,
}

impl Exercise {
    /// Every drain counter the full run asserts on, by name.
    pub fn counters(&self) -> [(&'static str, u32); 4] {
        [
            ("drain orders (S-0608)", self.m7a.uses.drain_orders),
            ("drain links formed (S-0608 claims 2 and 5)", self.links),
            (
                "credits drained (S-0608 claims 7 to 10)",
                self.credits_drained,
            ),
            (
                "drained-building ticks (S-0608 claim 1)",
                self.drained_ticks,
            ),
        ]
    }

    /// The names of the drain counters that a full run left below
    /// [`MARGIN`] (empty when the run exercised all of them enough).
    pub fn missing(&self) -> Vec<&'static str> {
        self.counters()
            .iter()
            .filter(|&&(_, n)| n < MARGIN)
            .map(|&(name, _)| name)
            .collect()
    }

    /// Notes what the tick just run showed and returns whether a kind of
    /// state appeared for the first time.
    fn observe(&mut self, w: &World, watch: &mut m7a::Watch) -> bool {
        let mut first = self.m7a.observe(w, watch);
        for e in &w.effects {
            if let Effect::Drain { event, .. } = e {
                match event {
                    DrainEvent::Formed { .. } => self.links += 1,
                    DrainEvent::Ended => self.ends += 1,
                    DrainEvent::Paid { amount, .. } => {
                        self.credits_drained = self
                            .credits_drained
                            .saturating_add(u32::try_from(*amount).unwrap_or(0));
                    }
                }
            }
        }
        let drained = w.drain.links.len();
        self.drained_ticks = self
            .drained_ticks
            .saturating_add(u32::try_from(drained).unwrap_or(u32::MAX));
        if drained > 0 && self.first_link.is_none() {
            self.first_link = Some(w.tick);
        }
        first |= watch.first(drained > 0, "drain link");
        first
    }
}

/// Plays the M7b duel for `ticks` ticks (it ends earlier when the game is
/// over). The game is saved and loaded halfway, at the first tick a drain
/// link exists and at the first tick each kind of M7a state appears; the
/// loaded game must hash alike and goes on from there. Returns the state
/// hash after every tick, the final game and what the run exercised.
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
    .expect("the synthetic M7b duel loads");
    let mut out = Vec::new();
    let mut ex = Exercise::default();
    let mut watch = m7a::Watch::default();
    let mut save_next = false;
    while g.world.tick < ticks {
        if g.world.tick == ticks / 2 || save_next {
            let loaded = Game::load(&g.save(), rules.clone(), catalogue.clone())
                .expect("the M7b duel's save loads");
            assert_eq!(
                loaded.state_hash(),
                g.state_hash(),
                "duel-m7b: a save and load keeps the state"
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
    ex.m7a.uses = crate::m6c::total(&g.special_uses());
    (out, g, ex)
}
