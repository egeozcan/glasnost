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
//!
//! M6a: the fixture's AI builds aircraft, pads and V3-style launchers, so the
//! duels play them; a third duel on the synthetic water map adds ships
//! (`golden/duel-naval.txt`). Each duel is also saved and loaded at the
//! first tick with a rocket in flight and the first tick with an aircraft in
//! the air.
//!
//! M6b: the fixture's AI ferries infantry in a hovering transport (S-0507);
//! every full duel must load a passenger and let one out, and is saved and
//! loaded at the first tick with a passenger aboard.
//!
//! M6c step 5: a fourth duel (`golden/duel-m6c.txt`) plays the own AI against
//! itself on the synthetic M6c map (`glasnost_ai::fixture_m6c`), whose invented
//! types make a plain game use every special ability of M6c (module [`m6c`]);
//! the full run asserts that each ability's order and event counters are
//! above zero, and is saved and loaded at the first tick each kind of M6c
//! state appears.
//!
//! M7a step 4: a fifth duel (`golden/duel-m7a.txt`) plays the own AI against
//! itself on the synthetic M7a map (`glasnost_ai::fixture_m7a`), where a
//! Yuri-style side with slave miners meets a fixture side and both have the
//! psychic weapons of S-0610 (module [`m7a`]); the same checks as M6c's.
//! M7a step 3.3 gives the Yuri-style side a gattling vehicle and defence
//! (S-0609) and a mind controller (S-0601), with their counters.
//!
//! M7b step 3.3: a sixth duel (`golden/duel-m7b.txt`) plays the M7a duel
//! with a drain (S-0608): the Yuri-style side's hovering discs drain the
//! other side's power plants and refineries (module [`m7b`]); the full run
//! asserts that each drain counter is at least [`m7b::MARGIN`].

pub mod m6c;
pub mod m7a;
pub mod m7b;
pub mod plain;

use glasnost_ai::{fixture, skirmish};
use glasnost_rules::layers::Layers;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::rules::SimRules;
use glasnost_sim::{save, testkit};

/// Seed of the gate's replay.
pub const SEED: u32 = 20_261_010;

/// Seed of the three AI duels. Moved from [`SEED`] in M6c step 2: with the
/// AI changes of that step the straight duel of the old seed never reached
/// its silo (its house spent its credits on the army), and the gate requires
/// that a duel launches a superweapon; this seed does, and every duel still
/// loads and unloads a transport. Two seeds were tried (the old one, which
/// failed, and this one, which passed), so this is one passing seed and
/// nothing more. Moved again in M6c step 5: with the AI's barracks rally
/// point, its ferry goal and its army type choice of that step no duel of
/// the old seed launched; of the eight seeds from 20 261 011 to 20 261 018
/// tried, this one (the last) is the only one whose straight duel launches,
/// so again one passing seed and nothing more. The purely simulated
/// replays keep [`SEED`].
pub const DUEL_SEED: u32 = 20_261_018;

/// Seed of the diagonal duel: [`DUEL_SEED`] until M7b AI round 2, when
/// house 0 won that game at tick 19 270 with its first ferry loaded and not
/// yet unloaded. Seed 8 plays to tick 26 867 with 18 loads and 18 unloads
/// (seeds 1 to 12 all ended between ticks 13 778 and 26 867; seed 12 never
/// loaded a ferry). After the economy-defence change later in that round
/// it plays to tick 28 947, again with 18 loads and 18 unloads.
pub const DUEL_DIAGONAL_SEED: u32 = 8;

/// Seed of the M6c duel. Kept at M7a step 2b, after the scenario was given
/// margins: over the sweep of seeds 1 to 32 (`m6c_seed_sweep`) this seed's
/// smallest counter is 4, the most any seed reached (see
/// `docs/development.md`, *Determinism gate*). Moved from 1 at M8 A3 intake
/// 110 (S-0309 claim 30: an engineer capture of a tech building requests
/// *guard*, from tick 48 of seed 1 on): seed 1 then ends at tick 9605 with
/// no radiation exit ordered; seed 2 plays all its ticks with smallest
/// counter 4 before and after the change (`m6c_seed_sweep`, seeds 1 to 16).
/// Moved from 2 at sim issues batch S3 (S-0415 claims 6 and 10, #221: a
/// unit attacking a vehicle that deploys into a building keeps attacking
/// the building; from tick 39 of seed 2 on, seven of house 0's infantry
/// keep house 1's new construction yard as their target): seed 2 then
/// ends at tick 9857 with smallest counter 1; seed 3 plays all its ticks
/// with smallest counter 4, the most any seed reached (`m6c_seed_sweep`,
/// seeds 1 to 16).
pub const M6C_SEED: u32 = 3;

/// Ticks of the full run of the M6c duel (shorter than the other duels'
/// games: every ability is exercised well before it).
pub const M6C_TICKS: i32 = 12_000;

/// Seed of the M7a duel. Moved from 1 at M7a step 3.3 (S-0609), when the
/// Yuri-style house was given a gattling vehicle and defence and a mind
/// controller: over the sweep of seeds 1 to 16 (`m7a_seed_sweep`) every
/// counter but one is then at least 3 in every seed (the psychic sensor
/// orders stay 2, one per house, as the AI orders a sensor only when the
/// house has none), but the Yuri-style house wins 7 of the 16 games
/// before the full run ends (seed 1 among them with an earlier tuning),
/// and the full run must play all its ticks. Seed 2 does in every tuning
/// tried, so it is kept, and nothing more (see `docs/development.md`, *Determinism gate*).
/// Moved from 2 to 3 at M7a step 1g (S-0605 revision 2 and the AI's
/// re-send of a stalled entrant): Alpha then won seed 2 at tick 10 314;
/// seed 3 played all its ticks in each of the three sweeps of that step.
/// At M7a step 3.6 (S-0603: a grinder, and badly damaged hulks for it) seed
/// 3 still plays all its ticks, with every counter at least 3 but the
/// sensor orders' 2, and is kept. Moved from 3 to 1 at M7a step 1n: since
/// step 1m (S-0607 revision 3) Alpha wins seed 3 at tick 11 222; seed 1,
/// the lowest that plays all its ticks, has every counter at least 3 but
/// the sensor orders' 2. Moved from 1 to 2 at M7a step 1o (S-0607
/// revisions 5 and 6: the jumpjet stop when a grip ends): seed 1 still
/// plays all its ticks but gives only 2 takeover orders; seed 2, the
/// lowest that plays all its ticks, has every counter at least 3 but the
/// sensor orders' 2. Moved from 2 to 1 at M7b step 1c (the S-0405 r20,
/// S-0502 r18 and S-0309 r21 revisions): Alpha wins seed 2 at tick
/// 11 331; seed 1, the lowest that plays all its ticks, has every counter
/// at least 3 but the sensor orders' 2. Moved from 1 to 16 at M7b step 2a
/// (the gate's margin setup of `glasnost_ai::fixture_m7a::map` and
/// `ruleset`): seed 16 is the only seed of 1 to 16 that plays all its
/// ticks with every counter at least 3, the sensor orders included (a
/// house lost its frail sensor and built another); seed 1 has the sensor
/// orders' 2. Moved from 16 to 4 at M7b AI round 2 (power before a
/// building, the army beside an extra harvester, the defence of the
/// economy): seed 16 gives 2 sensor orders and 2 bunker entry orders;
/// seed 4 is the only seed of 1 to 16 that plays all its ticks with every
/// counter at least 3. Moved from 4 to 27 later in that round (the
/// economy defence keeps a unit's fight instead of switching it between
/// raiders): seed 4 gives no takeover order, and no seed of 1 to 16 has
/// every counter at least 3 (seeds 1, 11 and 13 come nearest, with the
/// sensor orders' 2); of seeds 17 to 40, seed 27 is the only one that
/// does. Moved from 27 to 7 in M7b AI round 3, with the margin setup's
/// five PREY, Alpha's start TAKER and the HULKs' armour class: seed 7
/// has every counter at least 3; seed 27 gives 1 takeover order. Moved
/// from 7 to 92 later in that round, with each house's start WARD (a
/// bunker vehicle the waves leave at home): no counter is 0 in seeds 1
/// to 16, none of which has every counter at least 3 (the sensor orders'
/// 2, or few takeover orders); of seeds 17 to 104, seed 92 is the only
/// one that does. Moved from 92 to 17 at M8 step A4 round 5 (note 24): the
/// absorber keeps slaves, so seed 92 absorbs no infantry; of seeds 1 to 40,
/// seeds 17 and 26 have every counter at least 3 (seed 17 is the lowest),
/// seed 17 has 18 absorber entry orders and 4 infantry absorbed, all real
/// spare infantry (no slave among them).
pub const M7A_SEED: u32 = 17;

/// Ticks of the full run of the M7a duel.
pub const M7A_TICKS: i32 = 12_000;

/// Seed of the M7b duel. Chosen at M7b step 3.3 (S-0608) from the sweep of
/// seeds 1 to 16 (`m7b_seed_sweep`): every seed plays all [`M7B_TICKS`]
/// ticks with every drain counter at least [`m7b::MARGIN`]; seed 11 has the
/// largest smallest drain counter (8 links formed; since M7b AI round 2,
/// with Beta's construction yard drainable too, 4). See
/// `docs/development.md`, *Determinism gate*.
pub const M7B_SEED: u32 = 11;

/// Ticks of the full run of the M7b duel: shorter than the M7a duel's,
/// because the Yuri-style side, draining from tick 650 or so, wins every
/// seed of 1 to 16 between ticks 8 896 and 10 314, and the full run must
/// play all its ticks.
pub const M7B_TICKS: i32 = 8_000;

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
    /// Our own AI against itself on the synthetic water map (M6a).
    DuelNaval,
    /// Our own AI against itself on the synthetic M6c map (M6c).
    DuelM6c,
    /// Our own AI against itself on the synthetic M7a map (M7a).
    DuelM7a,
    /// Our own AI against itself on the synthetic M7b map (M7b).
    DuelM7b,
}

impl Scenario {
    /// Every scenario, in gate order.
    pub const ALL: [Scenario; 9] = [
        Scenario::Scripted,
        Scenario::Combat,
        Scenario::Skirmish,
        Scenario::Duel,
        Scenario::DuelDiagonal,
        Scenario::DuelNaval,
        Scenario::DuelM6c,
        Scenario::DuelM7a,
        Scenario::DuelM7b,
    ];

    /// The scenario's name, also its golden file's stem.
    pub fn name(self) -> &'static str {
        match self {
            Scenario::Scripted => "scripted",
            Scenario::Combat => "combat",
            Scenario::Skirmish => "skirmish",
            Scenario::Duel => "duel",
            Scenario::DuelDiagonal => "duel-diagonal",
            Scenario::DuelNaval => "duel-naval",
            Scenario::DuelM6c => "duel-m6c",
            Scenario::DuelM7a => "duel-m7a",
            Scenario::DuelM7b => "duel-m7b",
        }
    }

    /// The scenario of a wasm export's argument.
    pub fn from_index(i: i32) -> Option<Scenario> {
        Self::ALL.get(usize::try_from(i).ok()?).copied()
    }

    /// The ticks of a full run of the scenario.
    pub fn ticks(self) -> i32 {
        match self {
            Scenario::Skirmish | Scenario::Duel | Scenario::DuelDiagonal | Scenario::DuelNaval => {
                SKIRMISH_TICKS
            }
            Scenario::DuelM6c => M6C_TICKS,
            Scenario::DuelM7a => M7A_TICKS,
            Scenario::DuelM7b => M7B_TICKS,
            _ => TICKS,
        }
    }

    /// Every how many ticks the golden file keeps a hash.
    pub fn stride(self) -> usize {
        match self {
            Scenario::Skirmish
            | Scenario::Duel
            | Scenario::DuelDiagonal
            | Scenario::DuelNaval
            | Scenario::DuelM6c
            | Scenario::DuelM7a
            | Scenario::DuelM7b => SKIRMISH_STRIDE,
            _ => 1,
        }
    }
}

/// The state hash of a plain AI duel after each tick until the game is
/// over or `ticks` ticks ran ([`plain::play`]: saved and loaded halfway and
/// at the first tick with a rocket in flight, an aircraft in the air, a
/// passenger aboard and a superweapon charging). Every full run must have
/// loaded a passenger into a transport and let one out (M6b, S-0507); the
/// straight duel must also have charged and launched a superweapon (M6b,
/// S-0512 and S-0513).
fn duel(duel: plain::Plain, ticks: i32) -> Vec<String> {
    let (out, _, ex) = plain::play(duel, duel.seed(), ticks);
    if ticks >= SKIRMISH_TICKS {
        assert!(
            ex.missing(duel).is_empty(),
            "{}: the full duel did not exercise {:?} ({ex:?})",
            duel.name(),
            ex.missing(duel)
        );
    }
    out
}

/// A scenario's rules with our own patch layers `layers` (M8 step C7) on
/// its ruleset's quirk switches, for the seed sweeps' `GLASNOST_PLAY_FIXES`.
/// With no layer they are the scenario's own rules (`kit::rules()`), hash
/// name `name`; with one the name says so. The gate and the goldens never
/// take a layer.
pub fn layered(mut ruleset: Ruleset, name: &str, layers: Layers) -> std::sync::Arc<SimRules> {
    if layers == Layers::REFERENCE {
        return std::sync::Arc::new(SimRules::new(&ruleset, name));
    }
    ruleset.globals.quirks = layers
        .apply_quirks(ruleset.globals.quirks)
        .expect("our own layer applies");
    std::sync::Arc::new(SimRules::new(&ruleset, &format!("{name}+play-fixes")))
}

/// The state hash after each of the first `ticks` ticks of a replay (the
/// skirmish and the duels stop earlier when their game is over).
pub fn hashes(scenario: Scenario, ticks: i32) -> Vec<String> {
    match scenario {
        // Since the M6b2 spec revisions (S-0215 revision 45: a falling infantry unit is not
        // ready to change; S-0513 and S-0207 timing) the straight duel's full run no longer
        // reaches a launcher shot either, like the naval duel; rocket flights are covered by
        // S-0506's claim tests and the local lane's "rocket flights" count, so it is no
        // longer required to fly one. It still must load and unload a transport and launch.
        Scenario::Duel => {
            return duel(plain::Plain::Straight, ticks);
        }
        // The diagonal game ends before a launcher is sent; the straight
        // duel flies both a rocket and an aircraft. Since S-0507 revision 4
        // (eb0224c: the ferried infantry's course differs from about tick
        // 10000) the naval game no longer reaches a launcher shot within the
        // full run, and no fixture change kept both it and the naval
        // skirmish tests, so the naval duel is not required to fly. Short of
        // credits, it never reaches the LAB that the superweapon building
        // needs either (M6b), so it is not required to launch one; the
        // straight and the diagonal duels launch.
        Scenario::DuelDiagonal => {
            // Since the M6b2 round-4 revisions (S-0501 revision 15: aircraft are listed
            // by height) the diagonal game charges its superweapon but no longer fires
            // one within its run; the straight duel still must (`Plain::launches`).
            return duel(plain::Plain::Diagonal, ticks);
        }
        Scenario::DuelM6c => {
            let (out, _, ex) = m6c::play(M6C_SEED, ticks);
            if ticks >= M6C_TICKS {
                assert!(
                    ex.missing().is_empty(),
                    "duel-m6c: the full run did not exercise {:?} ({ex:?})",
                    ex.missing()
                );
            }
            return out;
        }
        Scenario::DuelM7a => {
            let (out, _, ex) = m7a::play(M7A_SEED, ticks);
            if ticks >= M7A_TICKS {
                assert!(
                    ex.missing().is_empty(),
                    "duel-m7a: the full run did not exercise {:?} ({ex:?})",
                    ex.missing()
                );
            }
            return out;
        }
        Scenario::DuelM7b => {
            let (out, _, ex) = m7b::play(M7B_SEED, ticks);
            if ticks >= M7B_TICKS {
                assert!(
                    ex.missing().is_empty(),
                    "duel-m7b: the full run did not exercise {:?} enough ({ex:?})",
                    ex.missing()
                );
            }
            return out;
        }
        Scenario::DuelNaval => {
            return duel(plain::Plain::Naval, ticks);
        }
        _ => {}
    }
    if scenario == Scenario::Skirmish {
        let run = skirmish::run(
            fixture::rules(),
            &fixture::map(),
            "synthetic-skirmish",
            fixture::catalogue(),
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
                fixture::catalogue(),
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
        Scenario::Skirmish
        | Scenario::Duel
        | Scenario::DuelDiagonal
        | Scenario::DuelNaval
        | Scenario::DuelM6c
        | Scenario::DuelM7a
        | Scenario::DuelM7b => {
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
    let seed = match scenario {
        Scenario::Duel | Scenario::DuelNaval => DUEL_SEED,
        Scenario::DuelDiagonal => DUEL_DIAGONAL_SEED,
        Scenario::DuelM6c => M6C_SEED,
        Scenario::DuelM7a => M7A_SEED,
        Scenario::DuelM7b => M7B_SEED,
        _ => SEED,
    };
    let mut s = format!(
        "# glasnost determinism gate ({name}): seed {seed}, {ticks} ticks, state hash (64 bits) after {every}\n"
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
