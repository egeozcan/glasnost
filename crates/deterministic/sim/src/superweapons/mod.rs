//! Superweapons: charge and readiness (S-0512) and the effects of the
//! Allied and Soviet kinds (S-0513).
//!
//! Each house has one [`Record`] per superweapon type ([`HouseSw`], kept in
//! `World::sw`). The charge rules run in the house update
//! ([`charge::house_update`] and [`charge::after_tech`], called from
//! `econ::houses`); a fire order ([`crate::orders::Order::FireSuperweapon`])
//! is carried out by [`launch::fire`]. The effects are in [`protect`]
//! (invulnerability and the force shield), [`nuke`], [`storm`], [`warp`]
//! and [`planes`] (spy plane and the paradrop launch; the delivery is
//! [`crate::paradrop`]).
//!
//! Seams to specs that are not built (each named where it is used):
//! - S-0525 (radiation fields): [`nuke::radiation_seam`];
//! - S-0514 (chrono weapons; units that cannot be teleported): the erasing
//!   half of [`being_erased`] and [`warp::chrono_seam`];
//! - S-0610, S-0611 and S-0612 (psychic reveal, genetic mutation, psychic
//!   domination): [`launch::yuri_seam`];
//! - S-0510 (spy infiltration resetting the first superweapon):
//!   [`charge::spy_reset`];
//! - S-0601 (mind control refused by protection): [`protected`] is the
//!   test it reads;
//! - S-0607 (locomotor seizing) and S-0519 (C4 planting): [`protected`].

pub mod charge;
pub mod launch;
pub mod nuke;
pub mod planes;
pub mod protect;
pub mod storm;
pub mod warp;

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_num::{F64, Rounding};
use glasnost_rules::schema::Ruleset;
use serde::{Deserialize, Serialize};

use crate::combat::data::{CombatRules, WarheadId, WeaponId};
use crate::rules::Kind;
use crate::world::{HouseId, ObjectId, Pos, World};

/// The superweapon kinds, in kind-number order 0 to 11 (S-0512 Import
/// mapping).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SwKind {
    /// The nuclear strike (missile kind).
    Missile,
    /// Invulnerability (iron curtain).
    IronCurtain,
    /// The weather storm.
    Storm,
    /// The teleport source (chronosphere first stage).
    Chrono,
    /// The teleport warp (chronosphere second stage).
    ChronoWarp,
    /// Paradrop.
    Paradrop,
    /// American paradrop.
    AmerParadrop,
    /// Psychic domination (S-0612; not built).
    Dominator,
    /// Spy plane.
    SpyPlane,
    /// Genetic mutation (S-0611; not built).
    Genetic,
    /// Force shield.
    ForceShield,
    /// Psychic reveal (S-0610; not built).
    PsychicReveal,
}

impl SwKind {
    /// The kind number (S-0512 Import mapping: 0 to 11 in table order).
    pub fn number(self) -> u32 {
        self as u32
    }

    /// The kind of a kind number.
    pub fn from_number(n: u32) -> Option<SwKind> {
        KINDS.get(n as usize).map(|&(_, k)| k)
    }
}

/// The kind words (Q-0730 interim): the table words of S-0512 Import
/// mapping with "kind" dropped and hyphens and spaces stripped, matched
/// without regard to case. Our engine's provisional spelling, not the
/// data's: Q-0730 asks for the words the rules files use.
// spec: S-0512 Import mapping (kind words in kind-number order, 0 to 11; matched without regard to case)
const KINDS: [(&str, SwKind); 12] = [
    ("multimissile", SwKind::Missile),
    ("ironcurtain", SwKind::IronCurtain),
    ("lightningstorm", SwKind::Storm),
    ("chronosphere", SwKind::Chrono),
    ("chronospherewarp", SwKind::ChronoWarp),
    ("parachutedrop", SwKind::Paradrop),
    ("americanparachutedrop", SwKind::AmerParadrop),
    ("psychicdominator", SwKind::Dominator),
    ("spyaircraft", SwKind::SpyPlane),
    ("geneticconversion", SwKind::Genetic),
    ("forceshield", SwKind::ForceShield),
    ("psychicreveal", SwKind::PsychicReveal),
];

/// A kind word's kind (Q-0730 interim normalisation).
pub fn kind_of_word(word: &str) -> Option<SwKind> {
    let w: String = word
        .chars()
        .filter(|c| *c != '-' && *c != ' ' && *c != '_')
        .flat_map(char::to_lowercase)
        .collect();
    KINDS.iter().find(|(k, _)| *k == w).map(|&(_, k)| k)
}

/// One superweapon type, resolved (S-0512 and S-0513 Import mapping).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwType {
    /// Type name.
    pub name: String,
    /// The kind; `None` for no kind.
    pub kind: Option<SwKind>,
    /// The charge time *T* in ticks (F1).
    pub charge: i32,
    /// Power-dependent.
    pub powered: bool,
    /// Disableable from the setup screen.
    pub disableable: bool,
    /// The countdown flag (claim 18).
    pub show_timer: bool,
    /// The flash window *F* in ticks, −1 unlimited (claim 17).
    pub flash: i32,
    /// First-stage flag.
    pub pre_click: bool,
    /// Second-stage flag.
    pub post_click: bool,
    /// The second stage's link, a kind number (claim 11).
    pub pre_dependent: Option<u32>,
    /// The aim action word, lower case (claim 23; Q-0731 interim).
    pub action: Option<String>,
    /// Drain mode (claim 19).
    pub drain: bool,
    /// Manual charge (claim 20).
    pub manual: bool,
    /// The auxiliary building type, by building registration index.
    pub aux: Option<u32>,
    /// The display-name label (claim 18).
    pub ui_name: Option<String>,
    /// The sidebar icon art name.
    pub sidebar_image: Option<String>,
    /// The silo's upward weapon (S-0513 claim 8).
    pub weapon: Option<WeaponId>,
    /// The force-shield start sound (S-0513 claim 32).
    pub start_sound: Option<String>,
    /// The force-shield fading sound (S-0513 claim 32).
    pub special_sound: Option<String>,
}

/// The storm's game-wide values (S-0513 Import mapping).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StormRules {
    /// Countdown *L*.
    pub deferment: i32,
    /// Duration *D* (−1 endless).
    pub duration: i32,
    /// Hit period.
    pub hit: i32,
    /// Scatter period.
    pub scatter: i32,
    /// Cell spread *s*.
    pub spread: i32,
    /// Separation.
    pub separation: i32,
    /// Storm damage.
    pub damage: i32,
    /// Storm warhead.
    pub warhead: Option<WarheadId>,
    /// The storm-message flag.
    pub print_text: bool,
    /// Cloud animations.
    pub clouds: Vec<String>,
    /// Bolt animations.
    pub bolts: Vec<String>,
    /// The storm explosion animation.
    pub explosion: Option<String>,
    /// The storm sound.
    pub sound: Option<String>,
    /// Lightning sounds.
    pub sounds: Vec<String>,
}

/// The superweapon rules (S-0512, S-0513).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SwRules {
    /// Import warnings (diagnostics; the loader turns them into notes).
    pub warnings: Vec<String>,
    /// The types in list order (S-0004).
    pub types: Vec<SwType>,
    /// ρ, the charge-to-drain ratio (F5).
    pub ratio: F64,
    /// Each building type's first and second superweapon, by building
    /// registration index.
    pub building_sw: Vec<[Option<u32>; 2]>, // spec: S-0512 Terms (a building type's first and second superweapon fields)
    /// Each building type's silo flag.
    pub silo: Vec<bool>,
    /// Each building type's weapons-factory flag (S-0513 claim 34).
    pub weapons_factory: Vec<bool>,
    /// The storm.
    pub storm: StormRules,
    /// Curtain duration.
    pub curtain_duration: i32,
    /// Curtain invoke animation.
    pub curtain_anim: Option<String>,
    /// Force-shield invoke animation.
    pub shield_anim: Option<String>,
    /// Force-shield radius *R* in cells.
    pub shield_radius: i32,
    /// Force-shield duration *D*.
    pub shield_duration: i32,
    /// Force-shield blackout length.
    pub shield_blackout: i32,
    /// Force-shield fade-sound lead time *P*.
    pub shield_fade: i32,
    /// The source marker.
    pub chrono_placement: Option<String>,
    /// The blasts at the source and destination.
    pub chrono_blast: Option<String>,
    /// The blast at the destination.
    pub chrono_blast_dest: Option<String>,
    /// The teleport delay (claim 40).
    pub chrono_delay: i32,
    /// The payload weapon `[NukePayload]`.
    pub payload: Option<WeaponId>,
    /// The silo take-off animation.
    pub nuke_take_off: Option<String>,
    /// The nuclear siren.
    pub dig_sound: Option<String>,
    /// The spy plane type `SPYP`, by aircraft registration index.
    pub spy_plane: Option<u32>,
    /// The first side group's paradrop infantry and count list lengths
    /// (claim 43).
    pub ally_para: (usize, usize),
    /// The camera sound.
    pub camera_sound: Option<String>,
    /// The paradrop plane type `PDPLANE`, by aircraft registration index.
    pub para_plane: Option<u32>,
    /// The payload lists (S-0522 claim 2) of the first, second and third
    /// side groups and of the American paradrop, in that order: each
    /// entry one plane carrying `count` units of an infantry type; `None`
    /// sends no plane at all.
    pub payloads: [Option<Vec<(u32, i32)>>; 4], // spec: S-0522 Import mapping (four payload lists)
    /// The paradrop radius, leptons (S-0522 claims 6 and 7).
    pub para_radius: i32,
    /// The parachute animation type (S-0522 claim 13), when the rules name
    /// one.
    pub parachute: Option<String>,
    /// The parachute sound (S-0522 claim 9), when the rules name one.
    pub chute_sound: Option<String>,
}

/// The fixed paradrop plane type name (S-0522 Import mapping).
// spec: S-0522 Import mapping (aircraft type ID `PDPLANE`, a name fixed in the original)
pub const PARA_PLANE: &str = "PDPLANE";

/// One payload list (S-0522 claims 2 and 3). `partial` is the second side
/// group's list, whose shorter count list sends the entries that have a
/// count.
fn payload_list(
    label: &str,
    inf: &Option<Vec<String>>,
    num: &Option<Vec<i32>>,
    partial: bool,
    names: &BTreeMap<(Kind, String), u32>,
    warnings: &mut Vec<String>,
) -> Option<Vec<(u32, i32)>> {
    // spec: S-0522 Import mapping (absent lists: empty, no plane)
    let inf = inf.clone().unwrap_or_default();
    let num = num.clone().unwrap_or_default();
    if inf.len() != num.len() {
        if !partial || num.len() > inf.len() {
            if !partial {
                // spec: S-0522 claim 2 (the American, first and third side groups' lists of different lengths: no plane at all)
                warnings.push(format!("S-0522 claim 2: paradrop list {label} has {} types and {} counts; it sends no plane", inf.len(), num.len()));
                return None;
            }
        } else {
            // spec: S-0522 claim 2 (the second side group's shorter count list: planes for the entries that have a count; the importer warns, departure)
            warnings.push(format!("S-0522 claim 2: paradrop list {label} has fewer counts than types; the entries without a count send no plane"));
        }
    }
    let mut out = Vec::new();
    // spec: S-0522 claim 2 (a count list longer than the infantry list has its extra counts ignored)
    for (t, &n) in inf.iter().zip(num.iter()) {
        let Some(&ti) = names.get(&(Kind::Infantry, t.clone())) else {
            // Q-0736 interim: an entry naming no infantry type sends no plane; the importer warns.
            warnings.push(format!("S-0522 claim 2 (Q-0736): paradrop list {label} names {t}, which is no infantry type; that entry sends no plane"));
            continue;
        };
        let n = if n < 0 {
            // spec: S-0522 claim 3 (a negative count is treated as 0, with an importer warning; design)
            warnings.push(format!("S-0522 claim 3: paradrop list {label} has the negative count {n} for {t}; treated as 0"));
            0
        } else {
            n
        };
        out.push((ti, n));
    }
    Some(out)
}

/// The fixed payload weapon name (S-0513 Import mapping).
// spec: S-0513 Import mapping (weapon section `[NukePayload]`, a fixed name)
pub const PAYLOAD: &str = "NukePayload";
/// The fixed spy plane type name (S-0513 Import mapping).
// spec: S-0513 Import mapping (aircraft type `SPYP`, a fixed name)
pub const SPY_PLANE: &str = "SPYP";
/// The fixed warning-marker animation name (S-0513 claim 11).
// spec: S-0513 Import mapping (animation type `PSIWARN`, a fixed name)
pub const WARNING_MARKER: &str = "PSIWARN";

impl SwRules {
    /// Resolves the superweapon types and the game-wide values.
    pub fn new(
        r: &Ruleset,
        names: &BTreeMap<(Kind, String), u32>,
        combat: &CombatRules,
        mode: Rounding,
    ) -> Self {
        let s = &r.sim;
        let g = &s.general.superweapons;
        let mut warnings = Vec::new();
        let list = r
            .registries
            .get("superweapons")
            .cloned()
            .unwrap_or_default();
        let find_sw = |n: &str| list.iter().position(|x| x.eq_ignore_ascii_case(n));
        let weapon =
            |n: &Option<String>| n.as_ref().and_then(|n| combat.weapon_index.get(n).copied());
        let mut types = Vec::new();
        for n in &list {
            let t = s
                .superweapons
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(n))
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            // spec: S-0512 F1 (T = trunc(Z(v × 900)), v the single-precision value widened; the 900-tick factor is exact)
            let converted = t.recharge_time.as_ref().map(|v| {
                v.single_value(mode)
                    .mul(F64::from_i64(900, mode), mode)
                    .to_i32_common()
            });
            let charge = match converted {
                // spec: S-0512 claim 21 and Constants (absent: the engine default 4500 ticks)
                None => 4500,
                Some(0) => {
                    // spec: S-0512 claim 21 (0 keeps the previous value; one merged layer here, so the engine default)
                    warnings.push(format!("S-0512 claim 21: superweapon {n} has charge time 0, which keeps the previous value; the merged ruleset has none, so the engine default 4500 ticks applies"));
                    4500 // spec: S-0512 Constants (the engine default charge time 4500 ticks)
                }
                Some(v) if v < 0 => {
                    // spec: S-0512 Edge cases (our importer refuses non-positive charge times with a warning; departure)
                    warnings.push(format!("S-0512 Edge cases: superweapon {n} has a negative charge time; refused, the engine default 4500 ticks applies"));
                    4500 // spec: S-0512 Constants (the engine default charge time 4500 ticks)
                }
                Some(v) => v,
            };
            let kind = match t.kind.as_deref() {
                None => None,
                Some(word) => {
                    let k = kind_of_word(word);
                    if k.is_none() {
                        // spec: S-0512 Import mapping (an unknown kind word keeps the previous kind; one merged layer: none)
                        warnings.push(format!("S-0512 Import mapping and Q-0730: superweapon {n} has an unknown kind word {word}; no kind"));
                    }
                    k
                }
            };
            let pre_dependent = t.pre_dependent.as_deref().and_then(|word| {
                // spec: S-0512 Import mapping (the link is a kind word as for `Type`; its kind number)
                let k = kind_of_word(word).map(SwKind::number);
                if k.is_none() {
                    warnings.push(format!("S-0512 claim 11: superweapon {n} links to an unknown kind word {word}; no first stage"));
                }
                k
            });
            let action = t.action.as_deref().and_then(|a| {
                let a = a.to_ascii_lowercase();
                // spec: S-0512 claim 23 and Import mapping (`None` maps to no action; case-insensitive; Q-0731 interim: an empty word is no action too)
                (a != "none" && !a.is_empty()).then_some(a)
            });
            let aux = t
                .aux_building
                .as_ref()
                .and_then(|a| names.get(&(Kind::Building, a.clone())).copied());
            types.push(SwType {
                name: n.clone(),
                kind,
                charge,
                // spec: S-0512 Import mapping (IsPowered absent: yes)
                powered: t.is_powered.unwrap_or(true),
                // spec: S-0512 Import mapping (DisableableFromShell absent: no)
                disableable: t.disableable_from_shell.unwrap_or(false),
                // spec: S-0512 Import mapping (ShowTimer absent: no)
                show_timer: t.show_timer.unwrap_or(false),
                // spec: S-0512 Import mapping (FlashSidebarTabFrames absent: −1)
                flash: t.flash_sidebar_tab_frames.unwrap_or(-1),
                // spec: S-0512 Import mapping (PreClick, PostClick absent: no)
                pre_click: t.pre_click.unwrap_or(false),
                post_click: t.post_click.unwrap_or(false),
                pre_dependent,
                action,
                // spec: S-0512 Import mapping (UseChargeDrain, ManualControl absent: no)
                drain: t.use_charge_drain.unwrap_or(false),
                manual: t.manual_control.unwrap_or(false),
                aux,
                ui_name: t.ui_name.clone(),
                sidebar_image: t.sidebar_image.clone(),
                // spec: S-0513 Import mapping (WeaponType absent: none)
                weapon: weapon(&t.weapon_type),
                // spec: S-0513 Import mapping (StartSound, SpecialSound absent: none)
                start_sound: t.start_sound.clone(),
                special_sound: t.special_sound.clone(),
            });
        }
        // spec: S-0512 Constants (default charge-to-drain ratio 3)
        let three = F64::from_i64(3, mode);
        let ratio = match g.charge_to_drain_ratio.as_ref().map(|d| d.value(mode)) {
            None => three,
            Some(v) if v.gt(F64::ZERO) => v,
            Some(_) => {
                // spec: S-0512 Edge cases (non-positive drain ratios refused with a warning; departure)
                warnings.push(
                    "S-0512 Edge cases: a non-positive charge-to-drain ratio is refused; 3 applies"
                        .to_owned(),
                );
                three
            }
        };
        let buildings = r.registries.get("buildings").cloned().unwrap_or_default();
        let mut building_sw = Vec::new();
        let mut silo = Vec::new();
        let mut weapons_factory = Vec::new();
        for n in &buildings {
            let b = s.buildings.get(n).cloned().unwrap_or_default();
            let f = |x: &Option<String>| x.as_deref().and_then(find_sw).map(|i| i as u32);
            building_sw.push([f(&b.econ.super_weapon), f(&b.econ.super_weapon2)]);
            // spec: S-0513 Import mapping (NukeSilo absent: no)
            silo.push(b.econ.nuke_silo.unwrap_or(false));
            // spec: S-0513 Import mapping (WeaponsFactory absent: no)
            weapons_factory.push(b.weapons_factory.unwrap_or(false));
        }
        let warhead = |n: &Option<String>| {
            n.as_ref().and_then(|n| {
                combat
                    .warheads
                    .iter()
                    .position(|h| h.name.eq_ignore_ascii_case(n))
                    .map(|i| i as WarheadId)
            })
        };
        let storm = StormRules {
            // spec: S-0513 Import mapping (LightningDeferment absent: 250 ticks)
            deferment: g.lightning_deferment.unwrap_or(250),
            // spec: S-0513 Import mapping (LightningStormDuration absent: 900 ticks)
            duration: g.lightning_storm_duration.unwrap_or(900),
            // spec: S-0513 Import mapping (LightningHitDelay absent: 90 ticks)
            hit: g.lightning_hit_delay.unwrap_or(90),
            // spec: S-0513 Import mapping (LightningScatterDelay absent: 10 ticks)
            scatter: g.lightning_scatter_delay.unwrap_or(10),
            // spec: S-0513 Import mapping (LightningCellSpread absent: 10 cells)
            spread: g.lightning_cell_spread.unwrap_or(10),
            // spec: S-0513 Import mapping (LightningSeparation absent: 3 cells)
            separation: g.lightning_separation.unwrap_or(3),
            // spec: S-0513 Import mapping (LightningDamage absent: 200 health points)
            damage: g.lightning_damage.unwrap_or(200),
            // spec: S-0513 Import mapping (LightningWarhead absent: none, claim 21)
            warhead: warhead(&g.lightning_warhead),
            // spec: S-0513 Import mapping (LightningPrintText absent: yes)
            print_text: g.lightning_print_text.unwrap_or(true),
            // spec: S-0513 Import mapping (WeatherConClouds, WeatherConBolts absent: empty)
            clouds: g.weather_con_clouds.clone().unwrap_or_default(),
            bolts: g.weather_con_bolts.clone().unwrap_or_default(),
            explosion: g.weather_con_bolt_explosion.clone(),
            sound: g.storm_sound.clone(),
            // spec: S-0513 Import mapping (LightningSounds absent: empty)
            sounds: g.lightning_sounds.clone().unwrap_or_default(),
        };
        if storm.clouds.is_empty() {
            // spec: S-0513 claim 21 (an empty cloud list: no cloud is made; the importer warns)
            warnings.push(
                "S-0513 claim 21: the cloud animation list is empty; no storm cloud is made"
                    .to_owned(),
            );
        }
        if storm.warhead.is_none() {
            // spec: S-0513 claim 21 (no storm warhead: strikes deliver no damage; the importer warns)
            warnings
                .push("S-0513 claim 21: no storm warhead; strikes deliver no damage".to_owned());
        }
        let payload = combat.weapon_index.get(PAYLOAD).copied();
        if payload.is_none() {
            // spec: S-0513 Edge cases (a missing payload weapon: no projectile; the importer warns)
            warnings.push("S-0513 Edge cases: no weapon NukePayload; nuclear strikes create no falling missile".to_owned());
        }
        let mut w2 = Vec::new();
        let payloads = [
            payload_list(
                "Ally",
                &g.ally_para_drop_inf,
                &g.ally_para_drop_num,
                false,
                names,
                &mut w2,
            ),
            payload_list(
                "Sov",
                &g.sov_para_drop_inf,
                &g.sov_para_drop_num,
                true,
                names,
                &mut w2,
            ),
            payload_list(
                "Yuri",
                &g.yuri_para_drop_inf,
                &g.yuri_para_drop_num,
                false,
                names,
                &mut w2,
            ),
            payload_list(
                "Amer",
                &g.amer_para_drop_inf,
                &g.amer_para_drop_num,
                false,
                names,
                &mut w2,
            ),
        ];
        warnings.extend(w2);
        let para_inf = g.ally_para_drop_inf.as_ref().map_or(0, Vec::len);
        let para_num = g.ally_para_drop_num.as_ref().map_or(0, Vec::len);
        Self {
            warnings,
            types,
            ratio,
            building_sw,
            silo,
            weapons_factory,
            storm,
            // spec: S-0513 Import mapping (IronCurtainDuration absent: 0 ticks)
            curtain_duration: g.iron_curtain_duration.unwrap_or(0),
            curtain_anim: g.iron_curtain_invoke_anim.clone(),
            shield_anim: g.force_shield_invoke_anim.clone(),
            // spec: S-0513 Import mapping (ForceShieldRadius absent: 10 cells)
            shield_radius: g.force_shield_radius.unwrap_or(10),
            // spec: S-0513 Import mapping (ForceShieldDuration absent: 400 ticks)
            shield_duration: g.force_shield_duration.unwrap_or(400),
            // spec: S-0513 Import mapping (ForceShieldBlackoutDuration absent: 800 ticks)
            shield_blackout: g.force_shield_blackout_duration.unwrap_or(800),
            // spec: S-0513 Import mapping (ForceShieldPlayFadeSoundTime absent: 50 ticks)
            shield_fade: g.force_shield_play_fade_sound_time.unwrap_or(50),
            chrono_placement: g.chrono_placement.clone(),
            chrono_blast: g.chrono_blast.clone(),
            chrono_blast_dest: g.chrono_blast_dest.clone(),
            // spec: S-0513 Import mapping (ChronoDelay absent: 60 ticks)
            chrono_delay: g.chrono_delay.unwrap_or(60),
            payload,
            nuke_take_off: g.nuke_take_off.clone(),
            dig_sound: g.dig_sound.clone(),
            spy_plane: names.get(&(Kind::Aircraft, SPY_PLANE.to_owned())).copied(),
            ally_para: (para_inf, para_num),
            camera_sound: g.spy_plane_camera.clone(),
            para_plane: names.get(&(Kind::Aircraft, PARA_PLANE.to_owned())).copied(),
            payloads,
            // spec: S-0522 Import mapping and Constants (ParadropRadius absent: 1024 leptons)
            para_radius: g.paradrop_radius.unwrap_or(1024),
            // spec: S-0522 Import mapping (Parachute absent: none; Open questions 2, Q-0752)
            parachute: g.parachute.clone(),
            // spec: S-0522 Import mapping (ChuteSound absent: no sound)
            chute_sound: g.chute_sound.clone(),
        }
    }

    /// The first type in list order of a kind.
    pub fn first_of_kind(&self, k: SwKind) -> Option<u32> {
        self.types
            .iter()
            .position(|t| t.kind == Some(k))
            .map(|i| i as u32)
    }
}

// ---------------------------------------------------------------------------
// State

/// The drain-mode states (S-0512 claim 19).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrainState {
    /// Charging.
    #[default]
    Charging,
    /// Ready.
    Ready,
    /// Active.
    Active,
}

/// One house's record of one superweapon type (S-0512 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    /// Granted.
    pub granted: bool,
    /// One-time (claim 12).
    pub one_time: bool,
    /// A permanent trigger grant (claim 13).
    pub permanent: bool,
    /// Ready.
    pub ready: bool,
    /// Suspended.
    pub suspended: bool,
    /// The clock runs.
    pub running: bool,
    /// The tick the running clock started.
    pub start: i32,
    /// The remaining time when the clock started or stopped.
    pub remain: i32,
    /// The ready tick (claim 17).
    pub ready_tick: i32,
    /// The last one-time grant tick, −100 initially (claim 17).
    pub one_time_tick: i32,
    /// The drain state (claim 19).
    pub drain: DrainState,
    /// The grant order, for the countdown rows (claim 18).
    pub seq: u64,
    /// The next completion is a source pick's: no announcement, ready tick
    /// unchanged (claim 10, our engine).
    pub silent: bool,
    /// The last picked source (claim 10; S-0513 claim 33).
    pub source: Option<CellXY>,
    /// The source has not been used by a second stage yet (claim 10, the
    /// quirk's off state).
    pub source_fresh: bool,
}

impl Default for Record {
    fn default() -> Self {
        Self {
            granted: false,
            one_time: false,
            permanent: false,
            ready: false,
            suspended: false,
            running: false,
            start: 0,
            remain: 0,
            ready_tick: 0,
            // spec: S-0512 Constants (initial flash reference tick −100)
            one_time_tick: -100,
            drain: DrainState::Charging,
            seq: 0,
            silent: false,
            source: None,
            source_fresh: false,
        }
    }
}

impl Record {
    /// The remaining time in tick `t` (F2).
    pub fn remaining(&self, t: i32) -> i32 {
        if self.running {
            // spec: S-0512 F2 (max(0, R − (n − s)), exact integers)
            (self.remain - (t - self.start)).max(0)
        } else {
            self.remain
        }
    }

    /// Stops the clock, keeping the remaining time (F2).
    pub fn stop(&mut self, t: i32) {
        self.remain = self.remaining(t);
        self.running = false;
    }

    /// Restarts the clock with remaining time `r` from tick `t`.
    pub fn restart(&mut self, t: i32, r: i32, running: bool) {
        self.remain = r;
        self.start = t;
        self.running = running;
    }
}

/// A house's superweapon state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HouseSw {
    /// Records by type list position.
    pub records: Vec<Record>,
    /// The target cell recorded by the last silo launch (S-0513 claim 8).
    pub nuke_target: Option<CellXY>,
    /// The force-shield fading sound: house updates left and the target
    /// point (S-0513 claim 32).
    pub fade: Option<(i32, Pos, u32)>,
}

/// The storm states (S-0513 T1).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StormState {
    /// Idle.
    #[default]
    Idle,
    /// Counting down.
    Counting,
    /// Raging.
    Raging,
    /// Clearing.
    Clearing,
}

/// The single game-wide storm (S-0513 Terms).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Storm {
    /// The state.
    pub state: StormState,
    /// The centre cell.
    pub centre: CellXY,
    /// The owner; none for no house.
    pub owner: Option<HouseId>,
    /// The duration.
    pub duration: i32,
    /// The countdown remainder while counting down.
    pub countdown: i32,
    /// The start tick.
    pub start: i32,
    /// The tick of the weather update in which clearing began.
    pub clearing_since: i32,
    /// The tick of the latest weather update (whether a stop request came
    /// before or after the update of its tick).
    pub last_update: i32,
}

impl Default for Storm {
    fn default() -> Self {
        Self {
            state: StormState::Idle,
            centre: CellXY::new(0, 0),
            owner: None,
            duration: 0,
            countdown: 0,
            start: 0,
            clearing_since: 0,
            last_update: -1,
        }
    }
}

/// A storm cloud (S-0513 claims 16 to 18).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cloud {
    /// The cell.
    pub cell: CellXY,
    /// The animation type.
    pub anim: String,
    /// Where it was placed.
    pub pos: Pos,
    /// The creation tick.
    pub created: i32,
    /// The stage delay (Q-0732 interim playback).
    pub delay: i32,
    /// Still pending its strike.
    pub pending: bool,
}

/// The game-wide superweapon state (`World::sw`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwWorld {
    /// Per house, in house order.
    pub houses: Vec<HouseSw>,
    /// The grant counter (claim 18's row order).
    pub seq: u64,
    /// The storm.
    pub storm: Storm,
    /// Present clouds, oldest first.
    pub clouds: Vec<Cloud>,
}

/// The protection kinds (S-0513 Terms).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProtectKind {
    /// Curtain kind.
    Curtain,
    /// Force-shield kind.
    ForceShield,
}

/// An object's protection timer (S-0513 claims 25 and 26).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protection {
    /// The start tick *t*₀.
    pub start: i32,
    /// The duration *D*.
    pub duration: i32,
    /// The kind.
    pub kind: ProtectKind,
    /// The tick of the tint's pulsing-phase draw, while it is due (claim 29).
    pub tint_due: Option<i32>,
}

/// An object's superweapon state.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjSw {
    /// The protection timer.
    pub protection: Option<Protection>,
    /// A warp in progress (S-0513 claims 34 to 40).
    pub warp: Option<warp::Member>,
    /// The held delay *H* (claim 40); 0 at creation.
    pub held_delay: i32,
    /// A silo's accepted launch: the target recorded when it accepted
    /// (S-0513 claim 13) and the type list position fired.
    pub silo_launch: Option<(Option<CellXY>, u32)>,
}

// ---------------------------------------------------------------------------
// Shared queries

/// Whether an object is protected now (S-0513 claim 26, F7).
pub fn protected(w: &World, id: ObjectId) -> bool {
    protection_kind(w, id).is_some()
}

/// The kind of an object's protection in force now, if any (F7).
pub fn protection_kind(w: &World, id: ObjectId) -> Option<ProtectKind> {
    let p = w.obj(id)?.sw.protection?;
    let t = w.tick;
    // spec: S-0513 F7 (protected exactly when t ≥ t₀, t − t₀ < D and D > 0)
    (t >= p.start && t - p.start < p.duration && p.duration > 0).then_some(p.kind)
}

/// Whether an object is being erased (glossary; S-0513 claim 39: a warp
/// member in its leaving stage). The chrono weapons' temporal beam is
/// S-0514's, not built: that half is a seam.
pub fn being_erased(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.sw.warp.as_ref())
        .is_some_and(|m| m.leaving())
}

/// Whether an object has a warp pending or is in its leaving stage (S-0513
/// claims 22 and 34).
pub fn warp_pending_or_leaving(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .and_then(|o| o.sw.warp.as_ref())
        .is_some_and(|m| m.pending_or_leaving())
}

/// The cells of an effect block in F1's order, off-map cells skipped.
pub fn block(w: &World, c: CellXY) -> Vec<(usize, CellXY)> {
    // spec: S-0513 F1 (the offsets (0, 0), (1, 0), (1, −1), (0, −1), (−1, −1), (−1, 0), (−1, 1), (0, 1), (1, 1))
    const OFFSETS: [(i32, i32); 9] = [
        (0, 0),
        (1, 0),
        (1, -1),
        (0, -1),
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
    ];
    OFFSETS
        .iter()
        .enumerate()
        .map(|(i, &(a, b))| (i, CellXY::new(c.x + a, c.y + b)))
        // spec: S-0513 F1 (a cell that is not on the map is skipped)
        .filter(|(_, c)| w.cells.cell(*c).is_some())
        .collect()
}

/// The swept layers of a block cell (claim 4), in recipient order.
pub fn swept_layers(w: &World, c: CellXY) -> Vec<crate::world::Layer> {
    use crate::world::Layer;
    if w.high_bridge(c) {
        if w.rules().quirks.superweapon_block_one_layer {
            // spec: S-0513 claim 4 (a high-bridge cell: the deck only; quirks.superweapon_block_one_layer)
            vec![Layer::Deck]
        } else {
            // spec: S-0513 claim 4 (quirk off: both layers, deck first)
            vec![Layer::Deck, Layer::Ground]
        }
    } else {
        // spec: S-0513 claim 4 (every other cell: the ground layer)
        vec![Layer::Ground]
    }
}

/// The recipients of an effect block: each swept layer's listed objects
/// in F1's cell order and S-0209 claim 13's listing order. With
/// `quirks.superweapon_list_cut_by_removal`, the caller re-reads the list
/// after each effect; this returns the lists as they stand.
pub fn listed(w: &World, c: CellXY, l: crate::world::Layer) -> Vec<ObjectId> {
    w.cells.list(c, l).to_vec()
}

/// Applies `f` to each object listed in cell `c`, layer `l`, in listing
/// order (S-0209 claim 13), honouring the list-cut quirk (S-0513 State
/// and ordering): with it on, an object that leaves the cell during its
/// own effect ends the walk of that cell.
pub fn walk_list(
    w: &mut World,
    c: CellXY,
    l: crate::world::Layer,
    f: &mut dyn FnMut(&mut World, ObjectId),
) {
    let cut = w.rules().quirks.superweapon_list_cut_by_removal;
    let list = listed(w, c, l);
    for id in list {
        if cut && !w.cells.list(c, l).contains(&id) {
            // spec: S-0513 State and ordering (quirks.superweapon_list_cut_by_removal: an earlier recipient removed at once cuts the walk)
            break;
        }
        f(w, id);
        if cut && !w.cells.list(c, l).contains(&id) {
            // spec: S-0513 State and ordering (later listed objects in that cell are not reached)
            break;
        }
    }
}

/// The point at a cell's centre on its terrain surface, plus the bridge
/// lift on a high-bridge cell when `lift` (S-0513 F2).
pub fn cell_point(w: &World, c: CellXY, lift: bool) -> Pos {
    let cc = Pos::centre(c, 0);
    let mut z = w.surface_at(cc.x, cc.y);
    if lift && w.high_bridge(c) {
        // spec: S-0513 F2 (b = 416 on a high-bridge cell)
        z += crate::teleport::DECK_HEIGHT;
    }
    Pos::new(cc.x, cc.y, z)
}

/// A whole-strength demolition hit (S-0513 claims 23, 24, 35 and 37).
pub fn demolish(w: &mut World, id: ObjectId, ignore: bool, house: Option<HouseId>) {
    let strength = crate::combat::damage::strength(w, id);
    let warhead = w.rules().combat.general.c4_warhead;
    // spec: S-0513 claims 23 and 35 (a hit of its type's whole strength, distance 0, the demolition warhead, no attacker object)
    let hit = crate::combat::damage::Hit {
        amount: strength,
        warhead,
        distance: 0,
        attacker: None,
        house,
        ignore_defences: ignore,
        no_escape: false,
    };
    crate::combat::damage::apply(w, id, hit);
}

/// The house's state, created on first use.
pub fn house_mut(w: &mut World, h: HouseId) -> &mut HouseSw {
    let n = w.rules().superweapons.types.len();
    let hi = usize::from(h.0);
    if w.sw.houses.len() <= hi {
        w.sw.houses.resize_with(hi + 1, HouseSw::default);
    }
    let hs = &mut w.sw.houses[hi];
    if hs.records.len() < n {
        hs.records.resize_with(n, Record::default);
    }
    hs
}

/// A house's record of a type, if any.
pub fn record(w: &World, h: HouseId, i: u32) -> Option<&Record> {
    w.sw.houses
        .get(usize::from(h.0))
        .and_then(|x| x.records.get(i as usize))
}

/// A house's record of a type, for writing.
pub fn record_mut(w: &mut World, h: HouseId, i: u32) -> Option<&mut Record> {
    let n = w.rules().superweapons.types.len();
    if (i as usize) >= n {
        return None;
    }
    house_mut(w, h).records.get_mut(i as usize)
}

/// A presentation event of a superweapon (announcements, sounds, messages
/// and markers; S-0512 claim 5, S-0513 claim 5). Not game state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwEvent {
    /// The ready announcement of the record's kind (S-0512 claim 5), for
    /// its own player.
    Ready,
    /// A launch announcement (S-0513 claim 5).
    Launch,
    /// An order refused (storm pending or raging), for its own player.
    Refused,
    /// The nuclear siren at the target.
    Siren,
    /// The storm-approach announcement and message.
    StormApproach,
    /// The storm sound and start message.
    StormStart,
    /// A lightning sound, by list position.
    Lightning(u32),
    /// The force shield's start sound.
    ShieldStart,
    /// The force shield's fading sound.
    ShieldFade,
    /// A radar event at the cell.
    Radar,
    /// The spy plane's camera sound.
    Camera,
    /// The teleport source marker over a picked source, for the launching
    /// player only (S-0513 claim 33).
    SourceMarker,
    /// The source marker ends (S-0513 claim 34).
    MarkerEnd,
    /// The parachute sound of a successful drop, at the plane's position
    /// (S-0522 claim 9).
    Chute,
    /// A parachute animation was attached to a dropped paratrooper (S-0522
    /// claim 13): the drawing shows it on the unit while the unit's
    /// parachute is attached. `delay` is the creation's stage delay (S-0109
    /// claim 3), when a draw gave one.
    Parachute {
        /// The paratrooper.
        trooper: ObjectId,
        /// The stage delay of the creation.
        delay: Option<i32>,
    },
}

/// Pushes a superweapon presentation event.
pub fn event(
    w: &mut World,
    ev: SwEvent,
    house: Option<HouseId>,
    sw: Option<u32>,
    pos: Option<Pos>,
) {
    w.effects.push(crate::combat::effects::Effect::Superweapon {
        event: ev,
        house,
        sw,
        pos,
    });
}
