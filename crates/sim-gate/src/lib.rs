//! The determinism gate: runs the synthetic replays of `glasnost_sim::testkit`
//! (the scripted movement replay and the combat replay) and the scripted
//! skirmish of `glasnost_ai` (M5a: base building, harvesting, production and
//! a game played to the last side), and gives the state hash after every
//! tick (every [`SKIRMISH_STRIDE`]th tick for the long skirmish). The same
//! code runs natively (tests and `cargo xtask gate`) and as a bare wasm32
//! module under node; both must reproduce the committed golden files
//! `golden/scripted.txt`, `golden/combat.txt` and `golden/skirmish.txt`
//! exactly. The skirmish's replay is recorded by running its scripts, then
//! replayed without them.
//!
//! The two AI duels (M5b) play our own AI against itself on the two
//! synthetic maps of `glasnost_ai::fixture` through `glasnost_ai::game::Game`,
//! whose hash covers the world and the AI's state; halfway through each run
//! the game is saved and loaded, the loaded game must hash alike, and the
//! run goes on from the loaded game (`golden/duel.txt`, `golden/duel-diagonal.txt`).

use glasnost_ai::{fixture, game::Game, skirmish};
use glasnost_sim::{save, testkit};

/// Seed of the gate's replay.
pub const SEED: u32 = 20_261_010;

/// Ticks of the full gate run.
pub const TICKS: i32 = 3000;

/// Ticks checked in debug builds by `cargo test` (debug arithmetic panics on
/// overflow, which release builds would wrap silently).
pub const DEBUG_TICKS: i32 = 300;

/// Ticks the skirmish may run at most (it stops when the game is over).
pub const SKIRMISH_TICKS: i32 = 40_000;

/// The skirmish's golden file keeps every this many ticks' hash.
pub const SKIRMISH_STRIDE: usize = 10;

/// The gate's replays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    /// Group moves, crossing paths, crushing, scatters and area guards.
    Scripted,
    /// Two sides fighting under the synthetic combat rules.
    Combat,
    /// The scripted skirmish (M5a).
    Skirmish,
    /// Our own AI against itself on the first synthetic map (M5b).
    Duel,
    /// Our own AI against itself on the diagonal synthetic map (M5b).
    DuelDiagonal,
}

impl Scenario {
    /// Every scenario, in gate order.
    pub const ALL: [Scenario; 5] = [
        Scenario::Scripted,
        Scenario::Combat,
        Scenario::Skirmish,
        Scenario::Duel,
        Scenario::DuelDiagonal,
    ];

    /// The scenario's name, also its golden file's stem.
    pub fn name(self) -> &'static str {
        match self {
            Scenario::Scripted => "scripted",
            Scenario::Combat => "combat",
            Scenario::Skirmish => "skirmish",
            Scenario::Duel => "duel",
            Scenario::DuelDiagonal => "duel-diagonal",
        }
    }

    /// The scenario of a wasm export's argument.
    pub fn from_index(i: i32) -> Option<Scenario> {
        Self::ALL.get(usize::try_from(i).ok()?).copied()
    }

    /// The ticks of a full run of the scenario.
    pub fn ticks(self) -> i32 {
        match self {
            Scenario::Skirmish | Scenario::Duel | Scenario::DuelDiagonal => SKIRMISH_TICKS,
            _ => TICKS,
        }
    }

    /// Every how many ticks the golden file keeps a hash.
    pub fn stride(self) -> usize {
        match self {
            Scenario::Skirmish | Scenario::Duel | Scenario::DuelDiagonal => SKIRMISH_STRIDE,
            _ => 1,
        }
    }
}

/// The state hash of an AI duel after each tick until the game is over or
/// `ticks` ticks ran; saved and loaded halfway (panics when the loaded game
/// hashes differently).
fn duel(map: &glasnost_map::Map, name: &str, ticks: i32) -> Vec<String> {
    let mut g = Game::new(
        fixture::rules(),
        map,
        name,
        testkit::catalogue(),
        fixture::setup(SEED),
    )
    .expect("the synthetic duel loads");
    let mut out = Vec::new();
    while g.world.tick < ticks {
        if g.world.tick == ticks / 2 {
            let loaded = Game::load(&g.save(), fixture::rules(), testkit::catalogue())
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
        // As the skirmish: stop after the tick in which the game ends.
        if g.outcome().is_some() {
            break;
        }
    }
    out
}

/// The state hash after each of the first `ticks` ticks of a replay (the
/// skirmish and the duels stop earlier when their game is over).
pub fn hashes(scenario: Scenario, ticks: i32) -> Vec<String> {
    match scenario {
        Scenario::Duel => return duel(&fixture::map(), "synthetic-duel", ticks),
        Scenario::DuelDiagonal => {
            return duel(&fixture::map_diagonal(), "synthetic-duel-diagonal", ticks);
        }
        _ => {}
    }
    if scenario == Scenario::Skirmish {
        let run = skirmish::run(
            fixture::rules(),
            &fixture::map(),
            "synthetic-skirmish",
            testkit::catalogue(),
            fixture::setup(SEED),
            &fixture::plan(),
            ticks,
        )
        .expect("the synthetic skirmish loads");
        let mut out = Vec::new();
        run.replay
            .run(
                fixture::rules(),
                &fixture::map(),
                testkit::catalogue(),
                |w| {
                    out.push(save::state_hash(w));
                },
            )
            .expect("the skirmish replay runs");
        return out;
    }
    let (rep, rules) = match scenario {
        Scenario::Scripted => (testkit::scripted_replay(SEED, ticks), testkit::rules()),
        Scenario::Combat => (testkit::combat_replay(SEED, ticks), testkit::combat_rules()),
        Scenario::Skirmish | Scenario::Duel | Scenario::DuelDiagonal => {
            unreachable!("handled above")
        }
    };
    let mut out = Vec::with_capacity(usize::try_from(ticks).unwrap_or(0));
    rep.run(rules, &testkit::map(), testkit::catalogue(), |w| {
        out.push(save::state_hash(w));
    })
    .expect("the synthetic replay runs");
    out
}

/// The golden file's text for `ticks` ticks: one header line, then the
/// first 16 hexadecimal digits (64 bits) of the state hash after each tick
/// (after every [`Scenario::stride`]th tick: ticks stride − 1, 2·stride − 1,
/// and so on).
pub fn golden_text(scenario: Scenario, ticks: i32) -> String {
    let name = scenario.name();
    let stride = scenario.stride();
    let every = if stride == 1 {
        "each tick".to_string()
    } else {
        format!("every {stride}th tick")
    };
    let mut s = format!(
        "# glasnost determinism gate ({name}): seed {SEED}, {ticks} ticks, state hash (64 bits) after {every}\n"
    );
    for h in hashes(scenario, ticks)
        .into_iter()
        .skip(stride - 1)
        .step_by(stride)
    {
        s.push_str(&h[..16]);
        s.push('\n');
    }
    s
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    //! Bare exports for node: `gate_run(scenario, ticks)` computes the golden
    //! text and returns its length; `gate_ptr()` gives its address in linear
    //! memory.

    use std::sync::Mutex;

    static OUT: Mutex<Vec<u8>> = Mutex::new(Vec::new());

    /// Runs the gate for `ticks` ticks; returns the length of the text.
    #[allow(unsafe_code)] // `no_mangle` is an unsafe attribute; the export is plain data
    #[unsafe(no_mangle)]
    pub extern "C" fn gate_run(scenario: i32, ticks: i32) -> u32 {
        let Some(scenario) = super::Scenario::from_index(scenario) else {
            return 0;
        };
        let text = super::golden_text(scenario, ticks).into_bytes();
        let len = u32::try_from(text.len()).unwrap_or(0);
        if let Ok(mut out) = OUT.lock() {
            *out = text;
        }
        len
    }

    /// Address of the text that the last `gate_run` produced.
    #[allow(unsafe_code)] // `no_mangle` is an unsafe attribute; the export is plain data
    #[unsafe(no_mangle)]
    pub extern "C" fn gate_ptr() -> u32 {
        OUT.lock().map_or(0, |o| o.as_ptr() as usize as u32)
    }
}
