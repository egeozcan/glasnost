//! The three plain AI duels of the gate (`duel`, `duel-diagonal`,
//! `duel-naval`): the own AI against itself on the synthetic maps of
//! `glasnost_ai::fixture`. The run counts what the gate requires of the
//! plain duels (M6a and M6b: transports loaded and let out, a superweapon
//! charged and launched, rockets and aircraft in flight), so that a sweep
//! over seeds can give each counter's margins (M8 step B4). The counters
//! are not part of the golden hashes.

use std::sync::Arc;

use glasnost_ai::{fixture, game::Game};
use glasnost_sim::rules::SimRules;
use glasnost_sim::world::{ObjectId, World};

/// What the simulation showed during a run of a plain duel.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exercise {
    /// Passengers that went aboard a transport (S-0507).
    pub loads: u32,
    /// Passengers let out beside a transport that still stands (S-0507;
    /// not the escape from a destroyed one).
    pub unloads: u32,
    /// Ticks in which a house held a granted superweapon that was still
    /// charging (S-0512 claims 3 and 5).
    pub charging_ticks: u32,
    /// Ticks whose effects held a superweapon launch (S-0513 claim 5).
    pub launches: u32,
    /// Ticks in which a rocket was in flight (S-0506).
    pub rocket_ticks: u32,
    /// Ticks in which an aircraft that is no rocket was in the air (S-0501).
    pub plane_ticks: u32,
}

/// Which of the plain duels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Plain {
    /// `duel`: the first synthetic map.
    Straight,
    /// `duel-diagonal`: the diagonal map.
    Diagonal,
    /// `duel-naval`: the water map.
    Naval,
}

impl Plain {
    /// Every plain duel, in gate order.
    pub const ALL: [Plain; 3] = [Plain::Straight, Plain::Diagonal, Plain::Naval];

    /// The duel's map.
    pub fn map(self) -> glasnost_map::Map {
        match self {
            Plain::Straight => fixture::map(),
            Plain::Diagonal => fixture::map_diagonal(),
            Plain::Naval => fixture::map_naval(),
        }
    }

    /// The duel's map name.
    pub fn map_name(self) -> &'static str {
        match self {
            Plain::Straight => "synthetic-duel",
            Plain::Diagonal => "synthetic-duel-diagonal",
            Plain::Naval => "synthetic-duel-naval",
        }
    }

    /// The gate's scenario name.
    pub fn name(self) -> &'static str {
        match self {
            Plain::Straight => "duel",
            Plain::Diagonal => "duel-diagonal",
            Plain::Naval => "duel-naval",
        }
    }

    /// The gate's seed for the duel.
    pub fn seed(self) -> u32 {
        match self {
            Plain::Straight | Plain::Naval => crate::DUEL_SEED,
            Plain::Diagonal => crate::DUEL_DIAGONAL_SEED,
        }
    }

    /// Whether the full run must charge and launch a superweapon (only the
    /// straight duel; see `hashes` for why the others are not required to).
    pub fn launches(self) -> bool {
        self == Plain::Straight
    }
}

impl Exercise {
    /// Every counter, by name.
    pub fn counters(&self) -> [(&'static str, u32); 6] {
        [
            ("transport loads (S-0507)", self.loads),
            ("transport unloads (S-0507)", self.unloads),
            (
                "superweapon charging ticks (S-0512 claims 3, 5)",
                self.charging_ticks,
            ),
            ("superweapon launches (S-0513 claim 5)", self.launches),
            ("rocket flight ticks (S-0506)", self.rocket_ticks),
            ("aircraft flight ticks (S-0501)", self.plane_ticks),
        ]
    }

    /// The counters the full run of `duel` asserts on: loads and unloads
    /// for every plain duel, and the superweapon's charge and launch for
    /// the straight one.
    pub fn required(&self, duel: Plain) -> Vec<(&'static str, u32)> {
        let c = self.counters();
        let n = if duel.launches() { 4 } else { 2 };
        c[..n].to_vec()
    }

    /// The required counters that a full run left at zero.
    pub fn missing(&self, duel: Plain) -> Vec<&'static str> {
        self.required(duel)
            .into_iter()
            .filter(|&(_, n)| n == 0)
            .map(|(name, _)| name)
            .collect()
    }
}

/// The credits setting of the plain duels (S-0412 claim 2), test data of
/// our own: the gate's margin setup (M8 step B4). On the rules' default
/// both houses spend their last credit by about tick 6800 and live from
/// two harvesters after, so that one transport bought (S-0507) in place of
/// a gun vehicle lost a house the game before it had loaded a passenger
/// (the straight duel's seed 7 and 29 of the 32-seed sweep).
pub const CREDITS: i32 = 20_000;

/// The lobby setup of the plain duels: [`fixture::setup`] with the credits
/// setting [`CREDITS`].
pub fn setup(seed: u32) -> glasnost_sim::world::GameSetup {
    let mut s = fixture::setup(seed);
    s.skirmish.credits = Some(CREDITS);
    s
}

/// Whether a rocket is in flight, and whether an aircraft that is no rocket
/// is in the air.
fn in_flight(w: &World) -> (bool, bool) {
    use glasnost_sim::rules::Kind;
    let rocket = w
        .objects
        .values()
        .any(|o| o.rocket.as_ref().is_some_and(|x| x.phase.is_some()));
    let plane = w.objects.iter().any(|(&id, o)| {
        o.kind == Kind::Aircraft
            && o.rocket.is_none()
            && o.on_map
            && glasnost_sim::air::height(w, id) > 0
    });
    (rocket, plane)
}

/// The objects that ride in a transport now, with their transports
/// (S-0507), in id order.
fn riders(w: &World) -> Vec<(ObjectId, ObjectId)> {
    w.objects
        .values()
        .filter_map(|o| Some((o.id, o.transport.riding?)))
        .collect()
}

/// Whether a house holds a superweapon record that is granted and still
/// charging (S-0512 claims 3 and 5).
fn charging(w: &World) -> bool {
    w.sw.houses
        .iter()
        .any(|h| h.records.iter().any(|r| r.granted && !r.ready))
}

/// Whether the tick just run launched a superweapon (S-0513 claim 5: the
/// launch event).
fn launched(w: &World) -> bool {
    w.effects.iter().any(|e| {
        matches!(
            e,
            glasnost_sim::combat::effects::Effect::Superweapon {
                event: glasnost_sim::superweapons::SwEvent::Launch,
                ..
            }
        )
    })
}

/// Plays a plain duel with `rules` for `ticks` ticks (it ends earlier when
/// the game is over); saved and loaded halfway, at the first tick with a
/// rocket in flight, with an aircraft in the air, with a passenger aboard a
/// transport and with a superweapon charging (panics when the loaded game
/// hashes differently). Returns the state hash after every tick, the final
/// game and what the run exercised.
pub fn play_with(
    rules: Arc<SimRules>,
    duel: Plain,
    seed: u32,
    ticks: i32,
) -> (Vec<String>, Game, Exercise) {
    let catalogue = fixture::catalogue();
    let name = duel.map_name();
    let mut g = Game::new(
        rules.clone(),
        &duel.map(),
        name,
        catalogue.clone(),
        setup(seed),
    )
    .expect("the synthetic duel loads");
    let mut out = Vec::new();
    let mut ex = Exercise::default();
    let (mut rocket_saved, mut plane_saved) = (false, false);
    let (mut aboard_saved, mut charge_saved) = (false, false);
    let mut before = riders(&g.world);
    while g.world.tick < ticks {
        let (rocket, plane) = in_flight(&g.world);
        let first_rocket = rocket && !rocket_saved;
        let first_plane = plane && !plane_saved;
        let first_aboard = !before.is_empty() && !aboard_saved;
        rocket_saved |= rocket;
        plane_saved |= plane;
        aboard_saved |= !before.is_empty();
        let charge = charging(&g.world);
        let first_charge = !charge_saved && charge;
        charge_saved |= first_charge;
        if g.world.tick == ticks / 2 || first_rocket || first_plane || first_aboard || first_charge
        {
            let loaded = Game::load(&g.save(), rules.clone(), catalogue.clone())
                .expect("the duel's save loads");
            assert_eq!(
                loaded.state_hash(),
                g.state_hash(),
                "{name}: a save and load keeps the state"
            );
            g = loaded;
        }
        g.step();
        out.push(g.state_hash());
        // Effects are presentation only (neither saved nor hashed).
        ex.launches += u32::from(launched(&g.world));
        g.world.effects.clear();
        let (rocket, plane) = in_flight(&g.world);
        ex.rocket_ticks += u32::from(rocket);
        ex.plane_ticks += u32::from(plane);
        ex.charging_ticks += u32::from(charging(&g.world));
        let now = riders(&g.world);
        let alive = |id| g.world.obj(id).is_some_and(|o| o.on_map && o.health > 0);
        ex.loads +=
            u32::try_from(now.iter().filter(|r| !before.contains(r)).count()).unwrap_or(u32::MAX);
        // Let out: on the map again beside a transport that still stands
        // (not the escape from a destroyed one).
        ex.unloads += u32::try_from(
            before
                .iter()
                .filter(|&&(p, t)| !now.contains(&(p, t)) && alive(p) && alive(t))
                .count(),
        )
        .unwrap_or(u32::MAX);
        before = now;
        // As the skirmish: stop after the tick in which the game ends.
        if g.outcome().is_some() {
            break;
        }
    }
    (out, g, ex)
}

/// [`play_with`] with the fixture's rules.
pub fn play(duel: Plain, seed: u32, ticks: i32) -> (Vec<String>, Game, Exercise) {
    play_with(fixture::rules(), duel, seed, ticks)
}
