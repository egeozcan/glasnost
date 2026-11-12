//! Skirmish setup (S-0412) and country bonuses at game start (S-0413).
//!
//! # Starting a skirmish
//!
//! A skirmish is a [`GameSetup`] with `sandbox: false`, loaded by
//! [`crate::load::new_game`] (or recorded and replayed by
//! [`crate::save::Replay::new`], which takes the same setup):
//!
//! 1. Build the setup with [`skirmish`]: the player houses in house order
//!    ([`Player`]: country, start position, AI or person, lobby team,
//!    handicap, colour slot, observer), the game-wide [`Settings`] (credits,
//!    short game, superweapons, crates, build off ally, redeploy), the
//!    unit-count and bases settings, the seed and the game speed. The
//!    neutral and special houses are appended after the players.
//! 2. Optionally resolve "random" lobby choices with [`resolve_lobby`]
//!    (S-0412 claims 6 and 7). It draws from its own stream, never from the
//!    game's, so the resolved setup is what a replay records.
//! 3. [`crate::load::new_game`] checks the setup ([`check`]), creates the
//!    houses with their credits setting (claim 2) and their per-house
//!    values ([`HouseSetupState`]), loads the map, runs the unused-house
//!    cleanup, gives the starting units (S-0009 claims 45 and 46: the base
//!    unit, such as the construction vehicle, and the budget units) and then
//!    calls [`start_game`] for the rest of game start in the order of claim
//!    14: crates (a seam), the computer-only extra credits of claim 3 (only
//!    for non-passive neutral and special houses in our engine), the start
//!    alliances of claim 8, the wall-owner rewrite and colour fields of
//!    S-0413 claims 5 and 6, and the secret labs (S-0407 claim 14) last, before
//!    the game-start pass (S-0202 claim 14, S-0412 claim 14).
//!
//! The outcome of the game is [`crate::econ::victory::outcome`].
//!
//! The setup values live in the game state ([`crate::econ::WorldEcon`]'s
//! `settings`); the topics read them through the accessors here
//! ([`credits_setting`], [`short_game`], [`superweapons_allowed`],
//! [`crates`], [`build_off_ally`], [`redeploy`]).

use std::collections::BTreeMap;

use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use crate::rng::Rng;
use crate::rules::SimRules;
use crate::world::{GameSetup, HouseId, HouseRole, HouseSetup, World};

/// The side whose countries the start alliances and the alliance orders
/// skip (S-0414 Import mapping: the side named Civilian, an engine
/// constant).
pub const CIVILIAN_SIDE: &str = "Civilian"; // spec: S-0414 Import mapping (the civilian side is the side named Civilian)

/// A difficulty row (S-0412 claim 5, the handicap).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Row {
    /// The easy row.
    Easy,
    /// The normal row.
    Normal,
    /// The difficult row.
    Difficult,
}

/// One player's setup values (S-0412 Terms, *setup values*: country,
/// colour, start position and team; our engine's handicap, claim 5).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseValues {
    /// The lobby team, `None` for "no team" (S-0412 claim 8).
    #[serde(default)]
    pub team: Option<u8>,
    /// The handicap: the difficulty row this house uses instead of the
    /// normal row (S-0412 claim 5); `None`: off.
    #[serde(default)]
    pub handicap: Option<Row>,
    /// The colour slot 0 … 7 (S-0103 claim 10; S-0412 claim 7).
    #[serde(default)]
    pub colour: Option<u8>,
    /// The house of an observer (S-0009 claim 22; S-0414 Terms).
    #[serde(default)]
    pub observer: bool,
}

/// The game's setup values (S-0412 claim 1). A value left `None` takes the
/// default the rules offer (S-0412 Import mapping).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The credits setting (claim 2).
    #[serde(default)]
    pub credits: Option<i32>,
    /// The short-game setting (S-0414 claim 2).
    #[serde(default)]
    pub short_game: Option<bool>,
    /// The superweapon setting (claim 9; S-0407 claim 7).
    #[serde(default)]
    pub superweapons: Option<bool>,
    /// The crate setting (claim 10).
    #[serde(default)]
    pub crates: Option<bool>,
    /// The build-off-ally setting (S-0406 claim 4).
    #[serde(default)]
    pub build_off_ally: Option<bool>,
    /// The bridge-destruction game option (S-0511 claim 1).
    #[serde(default)]
    pub bridge_destruction: Option<bool>,
    /// The redeploy setting (S-0415).
    #[serde(default)]
    pub redeploy: Option<bool>,
    /// A game mode of the all-base-units family (claim 13). Seam: setting
    /// it refuses the setup until the mode is written.
    #[serde(default)]
    pub all_base_units: bool,
    /// A network game (S-0213 claim 15, Network games): every machine
    /// checks an order's actor's owner immediately before applying it,
    /// not when the order is issued. Off for a skirmish game, a replay of
    /// one and a test.
    #[serde(default)]
    pub network: bool,
    /// Per-house values in house order; a missing entry is the default
    /// (no team, no handicap, no colour, not an observer).
    #[serde(default)]
    pub houses: Vec<HouseValues>,
}

/// A house's setup values in play (S-0412 claims 4, 5 and 8; S-0414
/// Terms).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseSetupState {
    /// The lobby team (claim 8).
    pub team: Option<u8>,
    /// The handicap row (claim 5).
    pub handicap: Option<Row>,
    /// The colour slot (S-0103 claim 10).
    pub colour: Option<u8>,
    /// An observer's house (S-0414 Terms, *observer's house set aside*).
    pub observer: bool,
    /// A person-played house (S-0414 Terms): a player house of a human
    /// participant who is not an observer.
    pub person_played: bool,
    /// The difficulty index (S-0412 Terms; claims 4 and 5).
    pub difficulty: i32,
    /// An AI house (glossary *AI house*): counted as computer-controlled
    /// in a rebuild of threat records (S-0212 claim 16a).
    #[serde(default)]
    pub ai: bool,
}

/// Game-start values that change country data for one game (S-0413 claims
/// 5 and 6) and the crate count of S-0412 claim 10.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartState {
    /// The rewritten wall-owner flags by country (S-0413 claim 6).
    pub wall_owner: BTreeMap<u32, bool>,
    /// The colour fields that hold a lobby colour slot, by country (S-0413
    /// claim 5).
    pub colour_field: BTreeMap<u32, u8>,
    /// The crates game start places (S-0412 claim 10).
    pub crates: i32,
    /// The bases setting of the game (S-0009 claim 45; S-0520 claim 13).
    #[serde(default)]
    pub bases: bool,
}

/// The setup rules (S-0412 Import mapping).
#[derive(Clone, Debug, Default)]
pub struct Rules {
    /// The default credits setting offered (`Money`).
    pub money: i32,
    /// The bases setting of the base rules (`Bases`).
    pub bases: bool,
    /// The defaults offered for the short-game, crate and superweapon
    /// settings.
    pub short_game: bool,
    /// See `short_game`.
    pub crates: bool,
    /// See `short_game`.
    pub superweapons: bool,
    /// The default of the bridge-destruction game option (S-0511 Import
    /// mapping).
    pub bridge_destruction: bool,
    /// *C*<sub>min</sub>, *C*<sub>max</sub> (claim 10).
    pub crate_min: i32,
    /// See `crate_min`.
    pub crate_max: i32,
    /// The extra-credit percentage list (claim 3).
    pub aicm: Vec<i32>,
    /// *c*, the double nearest to 1/100 (claim 3).
    pub percent: F64,
    /// Per country: its side is the civilian side (S-0414 claim 12).
    pub civilian: Vec<bool>,
    /// Per country: the imported colour field, a colour definition
    /// position (S-0413 claim 5).
    pub colour_field: Vec<u32>,
}

impl Rules {
    /// The setup rules of a resolved ruleset.
    pub fn new(r: &Ruleset, mode: Rounding) -> Self {
        let g = &r.sim.general.economy;
        let names = r.registries.get("countries").cloned().unwrap_or_default();
        let colours: Vec<String> = r
            .colours
            .iter()
            .map(|c| c.name.to_ascii_lowercase())
            .collect();
        Self {
            // spec: S-0412 Import mapping (Money absent: 10000)
            money: g.money.unwrap_or(10000),
            // spec: S-0412 claim 1 (the bases setting's default: the base rules' Bases, or yes)
            bases: g.bases.unwrap_or(true),
            // spec: S-0412 Import mapping (ShortGame absent: no)
            short_game: g.short_game.unwrap_or(false),
            // spec: S-0412 Import mapping (Crates absent: yes)
            crates: g.crates.unwrap_or(true),
            // spec: S-0412 Import mapping (SuperWeaponsAllowed absent: yes)
            superweapons: g.super_weapons_allowed.unwrap_or(true),
            // spec: S-0511 Import mapping (BridgeDestruction absent: yes)
            bridge_destruction: g.bridge_destruction.unwrap_or(true),
            // spec: S-0412 claim 10 (crate minimum absent: 1)
            crate_min: g.crate_minimum.unwrap_or(1),
            // spec: S-0412 claim 10 (crate maximum absent: 255)
            crate_max: g.crate_maximum.unwrap_or(255),
            // spec: S-0412 Import mapping (MultiplayerAICM absent: no entries)
            aicm: g.multiplayer_aicm.clone().unwrap_or_default(),
            // spec: S-0412 claim 3 (c, the double nearest to 1/100)
            percent: {
                let _ = mode;
                crate::rules::nearest_double("0.01")
            },
            civilian: names
                .iter()
                .map(|n| {
                    r.sim
                        .countries
                        .get(n)
                        .and_then(|c| c.econ.side.as_deref())
                        .is_some_and(|s| s == CIVILIAN_SIDE)
                })
                .collect(),
            colour_field: names
                .iter()
                .map(|n| {
                    r.sim
                        .countries
                        .get(n)
                        .and_then(|c| c.econ.color.as_deref())
                        .and_then(|c| {
                            let c = c.to_ascii_lowercase();
                            colours.iter().position(|x| *x == c)
                        })
                        // spec: S-0413 Import mapping (Color absent: the colour definition at position 0)
                        .map_or(0, |p| p as u32)
                })
                .collect(),
        }
    }
}

// ---------------------------------------------------------------------------
// Setting accessors

fn rules(w: &World) -> &Rules {
    &w.rules().econ.setup
}

/// The credits setting (S-0412 claim 2).
pub fn credits_setting(w: &World) -> i32 {
    w.econ.settings.credits.unwrap_or(rules(w).money)
}

/// The short-game setting (S-0414 claim 2).
pub fn short_game(w: &World) -> bool {
    w.econ.settings.short_game.unwrap_or(rules(w).short_game)
}

/// The bridge-destruction game option (S-0511 claim 1): in a skirmish the
/// setup screens' option, whose default the rules offer; the map's own
/// setting has no effect.
pub fn bridge_destruction(w: &World) -> bool {
    // spec: S-0511 claim 1 and Import mapping (the game option; the map's DestroyableBridges is not used in a skirmish)
    w.econ
        .settings
        .bridge_destruction
        .unwrap_or(rules(w).bridge_destruction)
}

/// The game's superweapon setting (S-0412 claim 9; S-0407 claim 7 reads
/// it).
pub fn superweapons_allowed(w: &World) -> bool {
    w.econ
        .settings
        .superweapons
        .unwrap_or(rules(w).superweapons)
}

/// The bases setting of this game (S-0412 Import mapping, S-0520 claims 9
/// and 13).
pub fn bases(w: &World) -> bool {
    w.econ.start.bases
}

/// The crate setting (S-0412 claim 10).
pub fn crates(w: &World) -> bool {
    w.econ.settings.crates.unwrap_or(rules(w).crates)
}

/// The build-off-ally setting (S-0406 claim 4).
pub fn build_off_ally(w: &World) -> bool {
    w.econ
        .settings
        .build_off_ally
        .unwrap_or(w.rules().econ.placement.build_off_ally)
}

/// The redeploy setting (S-0415).
pub fn redeploy(w: &World) -> bool {
    w.econ
        .settings
        .redeploy
        .unwrap_or(w.rules().econ.placement.mcv_redeploys)
}

/// A house's setup state.
pub fn house(w: &World, h: HouseId) -> Option<&HouseSetupState> {
    w.houses.get(usize::from(h.0)).map(|x| &x.econ.setup)
}

/// Whether the house is an observer's house (S-0414 Terms).
pub fn observer(w: &World, h: HouseId) -> bool {
    house(w, h).is_some_and(|s| s.observer)
}

/// Whether the house is person-played (S-0414 Terms).
pub fn person_played(w: &World, h: HouseId) -> bool {
    house(w, h).is_some_and(|s| s.person_played)
}

/// Whether the house plays a country of the civilian side (S-0414 claim
/// 12).
pub fn civilian(w: &World, h: HouseId) -> bool {
    w.houses.get(usize::from(h.0)).is_some_and(|x| {
        rules(w)
            .civilian
            .get(x.country as usize)
            .copied()
            .unwrap_or(false)
    })
}

/// Whether the house's country is passive (S-0009 Terms).
pub fn passive(w: &World, h: HouseId) -> bool {
    w.houses.get(usize::from(h.0)).is_none_or(|x| {
        w.rules()
            .countries
            .get(x.country as usize)
            .is_none_or(|c| c.passive)
    })
}

/// A country's wall-owner flag in force (S-0413 claim 6; S-0210 claim 14).
pub fn wall_owner(w: &World, country: u32) -> bool {
    w.econ
        .start
        .wall_owner
        .get(&country)
        .copied()
        .unwrap_or_else(|| {
            w.rules()
                .countries
                .get(country as usize)
                .is_some_and(|c| c.wall_owner)
        })
}

/// A country's colour field (S-0413 claim 5): the lobby colour slot of the
/// last player house of the country, or the imported colour definition
/// position. Presentation only (the alliance messages).
pub fn colour_field(w: &World, country: u32) -> u32 {
    w.econ
        .start
        .colour_field
        .get(&country)
        .map(|&s| u32::from(s))
        .unwrap_or_else(|| {
            rules(w)
                .colour_field
                .get(country as usize)
                .copied()
                .unwrap_or(0)
        })
}

// ---------------------------------------------------------------------------
// Building a setup

/// One player of a skirmish setup ([`skirmish`]).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Player {
    /// The house's name.
    pub name: String,
    /// The country type name.
    pub country: String,
    /// The chosen start position, `None` for random (S-0009 claim 19).
    pub start: Option<u32>,
    /// An AI house (glossary *AI house*; S-0412 claim 12); otherwise a
    /// person's house.
    pub ai: bool,
    /// Lobby team, handicap, colour slot and observer.
    pub values: HouseValues,
}

/// A skirmish setup: the players in house order, then the neutral and
/// special houses when the rules have those countries (S-0009 claim 19).
/// Every player house counts as human-controlled (glossary *AI house*).
pub fn skirmish(
    rules: &SimRules,
    players: &[Player],
    settings: Settings,
    unit_count: i32,
    seed: u32,
) -> GameSetup {
    let mut houses: Vec<HouseSetup> = players
        .iter()
        .map(|p| HouseSetup {
            name: p.name.clone(),
            country: p.country.clone(),
            human: true,
            start: p.start,
            role: HouseRole::Player,
            ai: p.ai,
        })
        .collect();
    for (name, role) in [
        (crate::load::NEUTRAL, HouseRole::Neutral),
        (crate::load::SPECIAL, HouseRole::Special),
    ] {
        if rules.country_names.iter().any(|n| n == name) {
            houses.push(HouseSetup {
                name: name.into(),
                country: name.into(),
                human: false,
                start: None,
                role,
                ai: false,
            });
        }
    }
    let mut settings = settings;
    settings.houses = players.iter().map(|p| p.values.clone()).collect();
    GameSetup {
        seed,
        game_speed: rules.general.lobby_game_speed,
        houses,
        alliances: Vec::new(),
        sandbox: false,
        unit_count,
        bases: rules.econ.setup.bases,
        skirmish: settings,
    }
}

/// Why a skirmish setup is refused (S-0412).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetupError {
    /// The short-game setting is on and the bases setting off, with
    /// `quirks.short_game_without_bases` off (claim 11).
    ShortGameWithoutBases,
    /// A player on a random colour finds no free colour slot (claim 7, a
    /// departure).
    NoFreeColour(usize),
    /// A random country with no country to choose (claim 6).
    NoCountry(usize),
    /// A game mode of the all-base-units family (claim 13), not written.
    AllBaseUnitsMode,
}

impl std::fmt::Display for SetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ShortGameWithoutBases => write!(
                f,
                "short game with bases off is refused (S-0412 claim 11, quirks.short_game_without_bases off)"
            ),
            Self::NoFreeColour(h) => {
                write!(
                    f,
                    "house {h} on a random colour finds no free colour slot (S-0412 claim 7)"
                )
            }
            Self::NoCountry(h) => write!(
                f,
                "house {h} on a random country has no country to choose (S-0412 claim 6)"
            ),
            Self::AllBaseUnitsMode => {
                write!(
                    f,
                    "the all-base-units game mode is not implemented (S-0412 claim 13)"
                )
            }
        }
    }
}

/// Checks a skirmish setup before the game is created (S-0412 claims 11
/// and 13).
pub fn check(setup: &GameSetup, rules: &SimRules) -> Result<(), SetupError> {
    if setup.sandbox {
        return Ok(());
    }
    let short = setup
        .skirmish
        .short_game
        .unwrap_or(rules.econ.setup.short_game);
    // spec: S-0412 claim 11 (quirks.short_game_without_bases off: a setup with short game on and bases off is refused)
    if short && !setup.bases && !rules.quirks.short_game_without_bases {
        return Err(SetupError::ShortGameWithoutBases);
    }
    // S-0412 claim 13 (the all-base-units family) is a seam: such a mode is refused until it is written.
    if setup.skirmish.all_base_units {
        return Err(SetupError::AllBaseUnitsMode);
    }
    Ok(())
}

/// A player's random lobby choices (S-0412 claims 6 and 7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LobbyChoice {
    /// The country is on "random".
    pub country: bool,
    /// The colour is on "random".
    pub colour: bool,
}

/// Resolves the random lobby choices of the player houses (S-0412 claims
/// 6 and 7) before the game, from the lobby's own stream seeded with
/// `lobby_seed` (never the game's stream). `choices[i]` belongs to house
/// `i`; `offered` are the countries the setup screen offers, read only with
/// `quirks.random_country_fixed_range` off.
pub fn resolve_lobby(
    setup: &mut GameSetup,
    rules: &SimRules,
    choices: &[LobbyChoice],
    offered: &[String],
    lobby_seed: u32,
) -> Result<(), SetupError> {
    let half = rules.quirks.ranged_draw_half_span;
    let mut rng = Rng::new(lobby_seed);
    let n = setup.houses.len();
    if setup.skirmish.houses.len() < n {
        setup.skirmish.houses.resize(n, HouseValues::default());
    }
    // spec: S-0412 claim 7 (a slot is free when no player has it fixed and no player resolved earlier was given it)
    let mut taken: Vec<u8> = (0..n)
        .filter(|&i| !choices.get(i).is_some_and(|c| c.colour))
        .filter_map(|i| setup.skirmish.houses[i].colour)
        .collect();
    for i in 0..n {
        let c = choices.get(i).copied().unwrap_or_default();
        if setup.houses[i].role != HouseRole::Player {
            continue;
        }
        if c.country {
            let country = if rules.quirks.random_country_fixed_range {
                if rules.country_names.is_empty() {
                    return Err(SetupError::NoCountry(i));
                }
                // spec: S-0412 claim 6 (k = R(0, 9), quirks.random_country_fixed_range; an index with no country draws again, a departure)
                loop {
                    let k = rng.ranged(0, 9, half);
                    if let Some(name) = rules.country_names.get(k as usize) {
                        break name.clone();
                    }
                }
            } else {
                if offered.is_empty() {
                    return Err(SetupError::NoCountry(i));
                }
                // spec: S-0412 claim 6 (switch off: uniform among the countries the setup offers)
                let k = rng.ranged(0, offered.len() as i32 - 1, half);
                offered[k as usize].clone()
            };
            setup.houses[i].country = country;
        }
        // spec: S-0412 claim 7 (an observer gets no colour slot and takes no draw)
        if c.colour && !setup.skirmish.houses[i].observer {
            // spec: S-0412 claim 7 and Constants (the slots 0 … 7; no free slot: the setup is refused, a departure)
            let Some(s) = draw_colour(&mut rng, &taken, half) else {
                return Err(SetupError::NoFreeColour(i));
            };
            taken.push(s);
            setup.skirmish.houses[i].colour = Some(s);
        }
    }
    Ok(())
}

/// The colour slot of a player whose colour is on "random" (S-0412 claim
/// 7): the first of successive ranged draws R(0, 7) on `rng` that is not
/// in `taken`; `None`, with no draw, when every slot 0 … 7 is taken.
/// `half_span_quirk` is `quirks.ranged_draw_half_span`, which cannot change
/// a draw over 0 … 7 (S-0202 claim 5 needs a span of 2147483647).
pub fn draw_colour(rng: &mut Rng, taken: &[u8], half_span_quirk: bool) -> Option<u8> {
    // spec: S-0412 claim 7 and Constants (the slots 0 … 7)
    if (0..=7u8).all(|s| taken.contains(&s)) {
        return None;
    }
    loop {
        // spec: S-0412 claim 7 (the first of successive draws R(0, 7) that is free)
        let s = rng.ranged(0, 7, half_span_quirk) as u8;
        if !taken.contains(&s) {
            return Some(s);
        }
    }
}

// ---------------------------------------------------------------------------
// House creation and game start

/// The per-house setup state and starting balance when the houses are
/// created (S-0412 claims 2, 4 and 5). `sandbox`: our viewer's setup,
/// which keeps a balance of 0.
pub fn house_values(setup: &GameSetup, i: usize, rules: &SimRules) -> (HouseSetupState, i32) {
    let h = &setup.houses[i];
    let v = setup.skirmish.houses.get(i).cloned().unwrap_or_default();
    let player = h.role == HouseRole::Player;
    let state = HouseSetupState {
        team: v.team,
        handicap: v.handicap,
        colour: v.colour,
        observer: player && v.observer,
        // spec: S-0414 Terms (a person-played house: a player house of a human participant who is not an observer; an AI house never is)
        person_played: player && h.human && !h.ai && !v.observer,
        // spec: S-0412 claims 4 and 5 (index 1 for a human-controlled house, which an AI house counts as, and for the neutral and special houses)
        difficulty: 1,
        ai: h.ai,
    };
    let credits = if setup.sandbox || !player {
        // spec: S-0412 claim 2 (the neutral and special houses start with 0 credits)
        0
    } else {
        // spec: S-0412 claim 2 (every player house, observer included, starts with the credits setting)
        setup.skirmish.credits.unwrap_or(rules.econ.setup.money)
    };
    (state, credits)
}

/// The difficulty row a player house uses (S-0412 claim 5): the normal row
/// unless its handicap picks another.
pub fn row_of(setup: &GameSetup, i: usize) -> Row {
    setup
        .skirmish
        .houses
        .get(i)
        .and_then(|v| v.handicap)
        .unwrap_or(Row::Normal)
}

/// Game start after the starting units (S-0412 claim 14), for a skirmish:
/// the crates (claim 10), the extra credits (claim 3), the start alliances
/// (claim 8), the wall-owner flags (S-0413 claim 6), the colour fields
/// (S-0413 claim 5) and, last of this function's draws, the secret labs
/// (S-0407 claim 14); the game-start pass's draws come after them (S-0202
/// claim 14, S-0412 claim 14). Removed houses get none of it (claim 14).
pub fn start_game(w: &mut World) {
    // spec: S-0412 claim 14 (the crate draws follow the starting units' draws)
    place_crates(w);
    extra_credits(w);
    start_alliances(w);
    rewrite_wall_owners(w);
    colour_fields(w);
    // spec: S-0407 claim 14 and S-0202 claim 14 (the secret-lab choices: the last draws here, before the game-start pass of S-0010 claim 12)
    crate::econ::tech::assign_secret_labs(w);
    // spec: S-0414 claim 1 (in a skirmish every contender is tested)
    w.econ.end.active = true;
}

/// The live houses of game start: those the cleanup did not remove (S-0412
/// claim 14).
fn live(w: &World) -> Vec<HouseId> {
    (0..w.houses.len())
        .filter(|&i| !w.houses[i].removed)
        .map(|i| HouseId(i as u8))
        .collect()
}

/// S-0412 claim 10: the number of crates game start places, and their
/// placing with its draws (S-0520 claims 2 and 3).
fn place_crates(w: &mut World) {
    if !crates(w) {
        // spec: S-0412 claim 10 (crate setting off: no crate at game start)
        w.econ.start.crates = 0;
        return;
    }
    let humans_only = w.rules().quirks.crate_count_humans_only;
    let players: Vec<HouseId> = live(w)
        .into_iter()
        .filter(|&h| w.houses[usize::from(h.0)].role == HouseRole::Player)
        .collect();
    let h = if humans_only {
        // spec: S-0412 claim 10 (H: the human participants, an observer included; AI houses do not count, quirks.crate_count_humans_only)
        players
            .iter()
            .filter(|&&h| person_played(w, h) || observer(w, h))
            .count() as i32
    } else {
        // spec: S-0412 claim 10 (switch off: H is the number of player houses)
        players.len() as i32
    };
    let r = rules(w);
    // spec: S-0412 Formulas (n = min(max(C_min, H), C_max))
    let n = r.crate_min.max(h).min(r.crate_max);
    // spec: S-0412 Formulas (no crate when n ≤ 0)
    w.econ.start.crates = n.max(0);
    // spec: S-0520 claim 2 (each game-start crate is one random placement, in order, one complete draw group after the other)
    crate::crates::place_at_start(w, n.max(0));
}

/// S-0412 claim 3: the extra credits of a house that is neither
/// human-controlled nor passive. In our engine only a non-passive neutral
/// or special house gets them (AI houses count as human-controlled).
fn extra_credits(w: &mut World) {
    let mode = w.rules().mode;
    for h in live(w) {
        let i = usize::from(h.0);
        // spec: S-0412 claim 3 (neither human-controlled nor passive; AI houses count as human-controlled, glossary *AI house*)
        if w.houses[i].human || passive(w, h) {
            continue;
        }
        let idx = w.houses[i].econ.setup.difficulty;
        let r = rules(w);
        // spec: S-0412 Edge cases (a list too short for the index: 0 extra credits)
        let Some(&p) = usize::try_from(idx).ok().and_then(|k| r.aicm.get(k)) else {
            continue;
        };
        let m = w.houses[i].credits;
        let x = extra_amount(p, m, r.percent, mode);
        w.houses[i].credits = w.houses[i].credits.wrapping_add(x);
    }
}

/// The extra credits of S-0412 claim 3 for percentage `p`, available
/// credits `m` and the factor `percent` (*c*).
pub fn extra_amount(p: i32, m: i32, percent: F64, mode: Rounding) -> i32 {
    // spec: S-0412 Formulas (extra = trunc(RZ₅₃(RZ₅₃(p × c) × M)))
    F64::from_i64(i64::from(p), mode)
        .mul(percent, mode)
        .mul(F64::from_i64(i64::from(m), mode), mode)
        .to_i32_common()
}

/// S-0412 claim 8: the alliances every skirmish starts with.
fn start_alliances(w: &mut World) {
    let houses = live(w);
    let special = houses
        .iter()
        .copied()
        .find(|&h| w.houses[usize::from(h.0)].role == HouseRole::Special);
    let mut pairs: Vec<(HouseId, HouseId)> = Vec::new();
    if let Some(s) = special {
        for &h in &houses {
            if h == s {
                continue;
            }
            // spec: S-0412 claim 8 (every house other than the special house allies it, and it allies every such house)
            pairs.push((h, s));
            pairs.push((s, h));
        }
    }
    for &a in &houses {
        for &b in &houses {
            if a == b || passive(w, a) || passive(w, b) {
                continue;
            }
            let (ta, tb) = (
                w.houses[usize::from(a.0)].econ.setup.team,
                w.houses[usize::from(b.0)].econ.setup.team,
            );
            // spec: S-0412 claim 8 (two houses that are not passive with the same lobby team number ally each other)
            if ta.is_some() && ta == tb {
                pairs.push((a, b));
            }
        }
    }
    let mut made: Vec<HouseId> = Vec::new();
    for (a, b) in pairs {
        // spec: S-0412 claim 8 (a one-way alliance is made only when the house to be allied does not play a country of the civilian side)
        if civilian(w, b) {
            continue;
        }
        let allies = &mut w.houses[usize::from(a.0)].allies;
        if let Err(p) = allies.binary_search(&b) {
            allies.insert(p, b);
            if !made.contains(&a) {
                made.push(a);
            }
        }
    }
    made.sort_unstable();
    for h in made {
        // spec: S-0212 claim 16a (a house that makes the alliances of game start is rebuilt then, after the map's objects and the starting units are placed)
        crate::threat::rebuild(w, h);
    }
    // spec: S-0409 claim 2 (who reveals for whom follows the alliances in force)
    crate::econ::shroud::alliance_changed(w);
}

/// S-0413 claim 6: the wall-owner flags of the countries the game's houses
/// play.
fn rewrite_wall_owners(w: &mut World) {
    let mut out = BTreeMap::new();
    for h in live(w) {
        let c = w.houses[usize::from(h.0)].country;
        // spec: S-0413 claim 6 (played and not passive: on; played and passive: off; every house of the game counts, the neutral and special houses included)
        out.insert(c, !passive(w, h));
    }
    w.econ.start.wall_owner = out;
}

/// S-0413 claim 5: the colour field of each country a player house plays
/// holds the colour slot of the last such house in house order.
fn colour_fields(w: &mut World) {
    let mut out = BTreeMap::new();
    for h in &w.houses {
        if h.role != HouseRole::Player {
            continue;
        }
        if let Some(s) = h.econ.setup.colour {
            // spec: S-0413 claim 5 (the slot of the last player house of the country in house order)
            out.insert(h.country, s);
        }
    }
    w.econ.start.colour_field = out;
}

// ---------------------------------------------------------------------------
// Country bonuses (S-0413)

/// The whole speed of an aircraft in progress units per tick (S-0413 claim
/// 3, Formulas *Aircraft speed*): S-0207 F2 with the country's aircraft
/// factor. `rho` is the aircraft's speed fraction, which the flight rules
/// own (the M6 flight rules); nothing calls this before they exist.
pub fn aircraft_speed(w: &World, id: crate::world::ObjectId, rho: F64) -> i32 {
    let r = w.rules();
    let mode = r.mode;
    let Some(o) = w.obj(id) else { return 0 };
    let Some(u) = r.aircraft.get(o.type_index as usize) else {
        return 0;
    };
    // spec: S-0413 claim 3 (the factor of the owner's country for the object's own kind)
    let kappa = o
        .owner
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .and_then(|h| r.countries.get(h.country as usize))
        .map_or(F64::from_i64(1, mode), |c| c.speed_aircraft);
    // spec: S-0413 Formulas (s₁ = trunc(RZ₅₃(RZ₅₃(V × κ) × b)); s₂ with ν for a faster veteran; s = trunc(RZ₅₃(s₂ × ρ)))
    let s1 = F64::from_i64(i64::from(u.speed), mode)
        .mul(kappa, mode)
        .mul(o.speed_bonus, mode)
        .to_i32_common();
    let s2 = if crate::combat::veterancy::has_ability(w, id, crate::combat::data::Ability::Faster) {
        F64::from_i64(i64::from(s1), mode)
            .mul(r.general.veteran_speed, mode)
            .to_i32_common()
    } else {
        s1
    };
    F64::from_i64(i64::from(s2), mode)
        .mul(rho, mode)
        .to_i32_common()
}

/// The handicap row's index into the three rows (easy, normal, difficult),
/// for the combat factors (S-0412 claim 5).
pub fn row_index(r: Row) -> usize {
    match r {
        Row::Easy => 0,
        Row::Normal => 1,
        Row::Difficult => 2, // nocite: an array position
    }
}
