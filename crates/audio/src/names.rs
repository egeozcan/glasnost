//! Sound names from the imported rules data, by role.
//!
//! Each key here is one a spec's Import mapping gives; the importer keeps
//! every type section's layered keys as text (`ImportedType::ini`), the
//! game-wide sections (`Ruleset::ini_globals`) and the art database's
//! sections (`Ruleset::art_sections`), so no converter change is needed.
//! Typed fields the converter already makes (`VoiceSelect`, the chrono
//! sounds) are taken from `Ruleset::sim`.
//!
//! Sounds and announcements that no Import mapping names resolve to
//! nothing: see [`Announcement`] and docs/development.md, "Audio (M5b)".

use std::collections::BTreeMap;

use glasnost_ini::value::{self, COMMON_TEXT_LIMIT};
use glasnost_rules::schema::{ImportedType, Ruleset};
use glasnost_sim::rules::Kind;

/// The sound names of one type.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypeSounds {
    /// `DeploySound` (S-0415 Import mapping, S-0107 Import mapping).
    pub deploy: Option<String>,
    /// `UndeploySound` (S-0107 Import mapping).
    pub undeploy: Option<String>,
    /// `PackupSound`, building types (S-0415 Import mapping).
    pub packup: Option<String>,
    /// `CrushSound` (S-0210 Import mapping).
    pub crush: Option<String>,
    /// `EnterWaterSound`, infantry (S-0107 Import mapping).
    pub enter_water: Option<String>,
    /// `LeaveWaterSound`, infantry (S-0107 Import mapping).
    pub leave_water: Option<String>,
    /// `VoiceComment`, infantry: the idle comments (S-0107 Import mapping).
    pub voice_comment: Vec<String>,
    /// `VoiceSelect`: the selection responses in the listed order (S-0214
    /// and S-0701 Import mappings).
    pub voice_select: Vec<String>,
    /// `VoiceSelectEnslaved` (S-0701 claim 17).
    pub voice_select_enslaved: Vec<String>,
    /// `VoiceSelectDeactivated` (S-0701 claim 17).
    pub voice_select_deactivated: Vec<String>,
    /// `VoiceMove` (S-0701 F6).
    pub voice_move: Vec<String>,
    /// `VoiceAttack` (S-0701 F6).
    pub voice_attack: Vec<String>,
    /// `VoiceSpecialAttack` (S-0701 F6).
    pub voice_special_attack: Vec<String>,
    /// `VoiceSecondaryWeaponAttack`, the normal secondary-weapon response
    /// (S-0701 F6).
    pub voice_secondary_attack: Option<String>,
    /// `VoiceEnter` (S-0701 F6).
    pub voice_enter: Option<String>,
    /// `VoiceCapture` (S-0701 F6).
    pub voice_capture: Option<String>,
    /// `VoiceHarvest` (S-0701 F6).
    pub voice_harvest: Option<String>,
    /// `VoiceDeploy` (S-0701 F6).
    pub voice_deploy: Option<String>,
    /// `VoiceFeedback` (S-0701 claim 20).
    pub voice_feedback: Vec<String>,
    /// `DieSound` (S-0701 claim 20).
    pub die_sound: Vec<String>,
    /// `DamageSound` (S-0701 claim 20).
    pub damage_sound: Option<String>,
    /// `MoveSound` (S-0701 claim 23).
    pub move_sound: Vec<String>,
    /// `TurretRotateSound` (S-0701 claim 23).
    pub turret_rotate: Option<String>,
    /// `CreateSound` (S-0701 claim 23).
    pub create: Option<String>,
    /// The type is the repair-response vehicle, `FV` (S-0701 F6, Import
    /// mapping).
    pub repair_vehicle: bool,
    /// `ChronoInSound`, falling back to the game-wide one (S-0207 Import
    /// mapping).
    pub chrono_in: Option<String>,
    /// `ChronoOutSound`, falling back to the game-wide one (S-0207 Import
    /// mapping).
    pub chrono_out: Option<String>,
}

/// Announcement roles of S-0701 claim 28 and the specs it cites.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Announcement {
    /// A deploy refused (S-0415 claim 4): entry `EVA_CannotDeployHere`.
    CannotDeploy,
    /// A unit sold (S-0408 claim 11).
    UnitSold,
    /// A building item done (S-0411 claims 9 and 12; S-0701 claim 28).
    ConstructionComplete,
    /// Base under attack (S-0701 claim 28).
    BaseUnderAttack,
    /// Ally under attack (S-0701 claim 28).
    AllyUnderAttack,
    /// Harvester under attack (S-0701 claim 28).
    HarvesterUnderAttack,
    /// Low funds (S-0701 claim 28).
    LowFunds,
    /// Low power (S-0701 claim 28).
    LowPower,
    /// Unit ready (S-0701 claim 28).
    UnitReady,
    /// Unit lost (S-0701 claim 28).
    UnitLost,
    /// Building, training (S-0411 claim 12; S-0701 claim 28).
    Building,
    /// On hold (S-0411 claim 12; S-0701 claim 28).
    OnHold,
    /// Cancelled (S-0411 claim 12; S-0701 claim 28).
    Cancelled,
    /// Unable to comply (S-0411 claim 12; S-0701 claim 28).
    UnableToComply,
    /// Repair complete (S-0314 claim 6).
    RepairComplete,
    /// A house defeated (S-0414 claim 5).
    Defeat,
    /// A unit promoted, to either rank (S-0701 claim 36).
    Promoted,
}

impl Announcement {
    /// The fixed announcement entry the event requests (S-0701 claim 36 and
    /// Import mapping, announcement requests).
    pub fn entry(self) -> &'static str {
        // spec: S-0701 Import mapping (announcement requests: fixed section names, looked up ignoring case)
        match self {
            Self::CannotDeploy => "EVA_CannotDeployHere",
            Self::UnitSold => "EVA_UnitSold",
            Self::ConstructionComplete => "EVA_ConstructionComplete",
            Self::BaseUnderAttack => "EVA_OurBaseIsUnderAttack",
            Self::AllyUnderAttack => "EVA_OurAllyIsUnderAttack",
            Self::HarvesterUnderAttack => "EVA_OreMinerUnderAttack",
            Self::LowFunds => "EVA_InsufficientFunds",
            Self::LowPower => "EVA_LowPower",
            Self::UnitReady => "EVA_UnitReady",
            Self::UnitLost => "EVA_UnitLost",
            Self::Building => "EVA_Building",
            Self::OnHold => "EVA_OnHold",
            Self::Cancelled => "EVA_Canceled",
            Self::UnableToComply => "EVA_UnableToComply",
            Self::RepairComplete => "EVA_UnitRepaired",
            Self::Defeat => "EVA_YouHaveLost",
            Self::Promoted => "EVA_UnitPromoted",
        }
    }
}

/// The rules data's sound names, by role.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoundNames {
    types: BTreeMap<(u8, u32), TypeSounds>,
    anim_start: BTreeMap<String, String>,
    anim_stop: BTreeMap<String, String>,
    reports: BTreeMap<String, Vec<String>>,
    announcements: BTreeMap<Announcement, String>,
    /// `[AudioVisual] SellSound` (S-0408, S-0701 Import mappings).
    pub sell: Option<String>,
    /// `[AudioVisual] RadarOn` (S-0410 Import mapping).
    pub radar_on: Option<String>,
    /// `[AudioVisual] RadarOff` (S-0410 Import mapping).
    pub radar_off: Option<String>,
    /// `[AudioVisual] CreditTicks`: first entry rising, second falling
    /// (S-0416 Import mapping).
    pub credit_ticks: Vec<String>,
    /// `CreateUnitSound` (S-0701 claim 23).
    pub create_unit: Option<String>,
    /// `CreateInfantrySound` (S-0701 claim 23).
    pub create_infantry: Option<String>,
    /// `CreateAircraftSound` (S-0701 claim 23).
    pub create_aircraft: Option<String>,
    /// `BuildingSlam` (S-0701 claim 23).
    pub building_slam: Option<String>,
    /// `BuildingDamageSound` (S-0701 claims 23 and 37).
    pub building_damage: Option<String>,
    /// `BuildingDieSound` (S-0701 claims 23 and 37).
    pub building_die: Option<String>,
    /// `UpgradeVeteranSound` (S-0701 claim 23).
    pub upgrade_veteran: Option<String>,
    /// `UpgradeEliteSound` (S-0701 claim 23).
    pub upgrade_elite: Option<String>,
    /// `BaseUnderAttackSound` (S-0701 claim 28).
    pub base_under_attack: Option<String>,
    /// `GuardSound` (S-0701 claim 19).
    pub guard: Option<String>,
    /// `StopSound` (S-0701 claim 19).
    pub stop: Option<String>,
    /// `ScatterSound` (S-0701 claim 19).
    pub scatter: Option<String>,
    /// `DeploySound`, read only with `quirks.deploy_command_silent` off
    /// (S-0701 claim 19).
    pub deploy_command: Option<String>,
    /// `ScoldSound` (S-0701 claim 24).
    pub scold: Option<String>,
    /// `GUIBuildSound` (S-0701 claim 24).
    pub gui_build: Option<String>,
    /// `VoiceIFVRepair` (S-0701 F6).
    pub voice_ifv_repair: Option<String>,
    /// `SpeakDelay`, minutes, as read (S-0701 claim 28); `None` when absent.
    pub speak_delay: Option<glasnost_ini::binfloat::Binary>,
    /// Building registration indices of the first three entries of
    /// `[AI] BuildConst` (S-0701 claim 28).
    pub build_const: Vec<u32>,
}

/// Our index of a simulation kind.
fn kind_code(k: Kind) -> u8 {
    match k {
        Kind::Infantry => 0,
        Kind::Vehicle => 1,
        Kind::Aircraft => 2,
        Kind::Building => 3,
        Kind::Terrain => 4,
    }
}

/// The importer's registry key of a simulation kind.
fn registry_key(k: Kind) -> &'static str {
    match k {
        Kind::Infantry => "infantry",
        Kind::Vehicle => "vehicles",
        Kind::Aircraft => "aircraft",
        Kind::Building => "buildings",
        Kind::Terrain => "terrain_objects",
    }
}

/// The bytes of a value in our rules format's reversible text form: each
/// character below 128 is that byte, and U+F780 to U+F7FF stand for bytes
/// 128 to 255 (the converter's `names::encode`).
fn bytes_of(text: &str) -> Vec<u8> {
    text.chars()
        .map(|c| {
            let u = u32::from(c);
            if u < 128 {
                u as u8
            } else if (0xF780..=0xF7FF).contains(&u) {
                (u - 0xF700) as u8
            } else {
                b'?'
            }
        })
        .collect()
}

/// The text form of name bytes (inverse of [`bytes_of`]).
fn text_of(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if b < 128 {
                char::from(b)
            } else {
                char::from_u32(0xF700 + u32::from(b)).unwrap_or('?')
            }
        })
        .collect()
}

/// A sound-name field: a text value (S-0003 claim 1); empty is none.
fn name_field(stored: Option<&String>) -> Option<String> {
    let b = bytes_of(stored?);
    let t = value::text(&b, COMMON_TEXT_LIMIT);
    (!t.is_empty()).then(|| text_of(t))
}

/// A list of sound names (S-0003 claim 12): split at commas, empty items
/// skipped, items untrimmed unless `quirks.list_items_untrimmed` is off.
fn list_field(stored: Option<&String>, untrimmed: bool) -> Vec<String> {
    let Some(s) = stored else { return Vec::new() };
    let b = bytes_of(s);
    value::list(&b, COMMON_TEXT_LIMIT)
        .into_iter()
        .map(|item| {
            if untrimmed {
                item
            } else {
                // spec: S-0003 claim 12 (switch off: name items trimmed; a whitespace-only item is skipped)
                glasnost_ini::lex::trim_full(item)
            }
        })
        .filter(|i| !i.is_empty())
        .map(text_of)
        .collect()
}

impl SoundNames {
    /// Collects the sound names of a resolved ruleset.
    pub fn from_ruleset(r: &Ruleset) -> Self {
        // spec: S-0003 claim 12 (quirks.list_items_untrimmed, default on)
        let untrimmed = r
            .import
            .as_ref()
            .and_then(|i| i.quirks.get("list_items_untrimmed").copied())
            .unwrap_or(true);
        let av = r.ini_globals.get("AudioVisual");
        let global = |k: &str| name_field(av.and_then(|s| s.get(k)));
        let g = &r.sim.general;
        let mut out = Self {
            // spec: S-0408 Import mapping (`[AudioVisual] SellSound`)
            sell: global("SellSound"),
            // spec: S-0410 Import mapping (`[AudioVisual] RadarOn`, `RadarOff`)
            radar_on: global("RadarOn"),
            radar_off: global("RadarOff"),
            // spec: S-0416 Import mapping (`CreditTicks`, a list of sound names)
            credit_ticks: list_field(av.and_then(|s| s.get("CreditTicks")), untrimmed),
            // spec: S-0701 Import mapping (`[AudioVisual]` sound names)
            create_unit: global("CreateUnitSound"),
            create_infantry: global("CreateInfantrySound"),
            create_aircraft: global("CreateAircraftSound"),
            building_slam: global("BuildingSlam"),
            building_damage: global("BuildingDamageSound"),
            building_die: global("BuildingDieSound"),
            upgrade_veteran: global("UpgradeVeteranSound"),
            upgrade_elite: global("UpgradeEliteSound"),
            base_under_attack: global("BaseUnderAttackSound"),
            guard: global("GuardSound"),
            stop: global("StopSound"),
            scatter: global("ScatterSound"),
            // spec: S-0701 claim 19 (`DeploySound`, used only with quirks.deploy_command_silent off)
            deploy_command: global("DeploySound"),
            scold: global("ScoldSound"),
            gui_build: global("GUIBuildSound"),
            voice_ifv_repair: global("VoiceIFVRepair"),
            // spec: S-0701 Import mapping (`SpeakDelay`, floating point, minutes)
            speak_delay: av.and_then(|s| s.get("SpeakDelay")).and_then(|v| {
                match value::decimal(&bytes_of(v), value::DecimalField::Double) {
                    value::Decimal::Value { value, .. } => Some(value),
                    value::Decimal::NoNumber => None,
                }
            }),
            ..Self::default()
        };
        // spec: S-0701 Import mapping (`[AI] BuildConst`: only its first three entries count)
        let bc = list_field(
            r.ini_globals.get("AI").and_then(|s| s.get("BuildConst")),
            untrimmed,
        );
        if let Some(reg) = r.registries.get("buildings") {
            out.build_const = bc
                .iter()
                .take(3)
                .filter_map(|n| reg.iter().position(|x| x.eq_ignore_ascii_case(n)))
                .map(|i| i as u32)
                .collect();
        }
        for kind in [
            Kind::Infantry,
            Kind::Vehicle,
            Kind::Aircraft,
            Kind::Building,
            Kind::Terrain,
        ] {
            let key = registry_key(kind);
            let Some(reg) = r.registries.get(key) else {
                continue;
            };
            let types = r.types.get(key);
            for (i, name) in reg.iter().enumerate() {
                let empty = ImportedType::default();
                let t = types.and_then(|m| m.get(name)).unwrap_or(&empty);
                let f = |k: &str| name_field(t.ini.get(k));
                let l = |k: &str| list_field(t.ini.get(k), untrimmed);
                let unit = match kind {
                    Kind::Infantry => r.sim.infantry.get(name),
                    Kind::Vehicle => r.sim.vehicles.get(name),
                    Kind::Aircraft => r.sim.aircraft.get(name),
                    _ => None,
                };
                let voice_select = match kind {
                    Kind::Building => r
                        .sim
                        .buildings
                        .get(name)
                        .and_then(|b| b.voice_select.clone()),
                    _ => unit.and_then(|u| u.voice_select.clone()),
                };
                let s = TypeSounds {
                    // spec: S-0415 Import mapping (`DeploySound`, `PackupSound`), S-0107 Import mapping (`DeploySound`, `UndeploySound`)
                    deploy: f("DeploySound"),
                    undeploy: f("UndeploySound"),
                    packup: if kind == Kind::Building {
                        f("PackupSound")
                    } else {
                        None
                    },
                    // spec: S-0210 Import mapping (`CrushSound`)
                    crush: f("CrushSound"),
                    // spec: S-0107 Import mapping (`EnterWaterSound`, `LeaveWaterSound`, `VoiceComment`: infantry)
                    enter_water: (kind == Kind::Infantry)
                        .then(|| f("EnterWaterSound"))
                        .flatten(),
                    leave_water: (kind == Kind::Infantry)
                        .then(|| f("LeaveWaterSound"))
                        .flatten(),
                    voice_comment: if kind == Kind::Infantry {
                        l("VoiceComment")
                    } else {
                        Vec::new()
                    },
                    // spec: S-0214 Import mapping (`VoiceSelect`, every entry kept in order; absent: empty)
                    voice_select: voice_select.unwrap_or_default(),
                    // spec: S-0701 Import mapping (the response lists and single responses of claims 17, 18 and 20)
                    voice_select_enslaved: l("VoiceSelectEnslaved"),
                    voice_select_deactivated: l("VoiceSelectDeactivated"),
                    voice_move: l("VoiceMove"),
                    voice_attack: l("VoiceAttack"),
                    voice_special_attack: l("VoiceSpecialAttack"),
                    voice_secondary_attack: f("VoiceSecondaryWeaponAttack"),
                    voice_enter: f("VoiceEnter"),
                    voice_capture: f("VoiceCapture"),
                    voice_harvest: f("VoiceHarvest"),
                    voice_deploy: f("VoiceDeploy"),
                    voice_feedback: l("VoiceFeedback"),
                    die_sound: l("DieSound"),
                    damage_sound: f("DamageSound"),
                    // spec: S-0701 Import mapping (`MoveSound` list, `TurretRotateSound`, `CreateSound`)
                    move_sound: l("MoveSound"),
                    turret_rotate: f("TurretRotateSound"),
                    create: f("CreateSound"),
                    // spec: S-0701 Import mapping (the repair-response vehicle is the vehicle type named `FV`, its type name compared ignoring case)
                    repair_vehicle: kind == Kind::Vehicle && name.eq_ignore_ascii_case("FV"),
                    // spec: S-0207 Import mapping (the type's chrono sounds, else the game-wide ones)
                    chrono_in: unit.and_then(|u| {
                        u.chrono_in_sound
                            .clone()
                            .or_else(|| g.chrono_in_sound.clone())
                    }),
                    chrono_out: unit.and_then(|u| {
                        u.chrono_out_sound
                            .clone()
                            .or_else(|| g.chrono_out_sound.clone())
                    }),
                };
                if s != TypeSounds::default() {
                    out.types.insert((kind_code(kind), i as u32), s);
                }
            }
        }
        // spec: S-0701 Import mapping (weapon sections: `Report`, a list)
        if let Some(ws) = r.types.get("weapons") {
            for (name, t) in ws {
                let list = list_field(t.ini.get("Report"), untrimmed);
                if !list.is_empty() {
                    out.reports.insert(name.to_ascii_uppercase(), list);
                }
            }
        }
        // spec: S-0109 Import mapping (`Report` in the animation's art section: its start sound); S-0701 Import mapping (`StartSound`, or `Report`, and `StopSound`)
        if let Some(reg) = r.registries.get("animations") {
            for name in reg {
                let section = r.art_sections.get(name).or_else(|| {
                    r.art_sections
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case(name))
                        .map(|(_, v)| v)
                });
                let rules = r.types.get("animations").and_then(|m| m.get(name));
                let key = |k: &str| {
                    name_field(section.and_then(|s| s.get(k)))
                        .or_else(|| name_field(rules.and_then(|t| t.ini.get(k))))
                };
                if let Some(s) = key("StartSound").or_else(|| key("Report")) {
                    out.anim_start.insert(name.to_ascii_uppercase(), s);
                }
                if let Some(s) = key("StopSound") {
                    out.anim_stop.insert(name.to_ascii_uppercase(), s);
                }
            }
        }
        out
    }

    /// The sound names of a type, by simulation kind and registration
    /// index (the index of `glasnost_sim::econ::TypeRef` and of an
    /// object's `type_index`).
    pub fn of_type(&self, kind: Kind, index: u32) -> Option<&TypeSounds> {
        self.types.get(&(kind_code(kind), index))
    }

    /// Sets the sound names of a type (tests and tools).
    pub fn set_type(&mut self, kind: Kind, index: u32, sounds: TypeSounds) {
        self.types.insert((kind_code(kind), index), sounds);
    }

    /// An animation type's start sound (S-0109 claim 13, S-0701 claim 22),
    /// by type name (ASCII case folded, S-0003 claim 14).
    pub fn anim_report(&self, anim: &str) -> Option<&str> {
        self.anim_start
            .get(&anim.to_ascii_uppercase())
            .map(String::as_str)
    }

    /// An animation type's stop sound (S-0701 claim 22).
    pub fn anim_stop(&self, anim: &str) -> Option<&str> {
        self.anim_stop
            .get(&anim.to_ascii_uppercase())
            .map(String::as_str)
    }

    /// Sets an animation type's start sound (tests and tools).
    pub fn set_anim_report(&mut self, anim: &str, sound: &str) {
        self.anim_start
            .insert(anim.to_ascii_uppercase(), sound.to_owned());
    }

    /// A weapon's report list (S-0701 claim 21), by weapon name.
    pub fn report(&self, weapon: &str) -> &[String] {
        self.reports
            .get(&weapon.to_ascii_uppercase())
            .map_or(&[], Vec::as_slice)
    }

    /// Sets a weapon's report list (tests and tools).
    pub fn set_report(&mut self, weapon: &str, list: &[&str]) {
        self.reports.insert(
            weapon.to_ascii_uppercase(),
            list.iter().map(|s| (*s).to_owned()).collect(),
        );
    }

    /// Sets the announcement data entry of a role (tests and tools).
    pub fn set_announcement(&mut self, a: Announcement, entry: &str) {
        self.announcements.insert(a, entry.to_owned());
    }

    /// The announcement data entry of a role ([`Announcement::entry`]),
    /// unless a test or tool set another with [`Self::set_announcement`].
    pub fn announcement(&self, a: Announcement) -> Option<&str> {
        Some(self.announcements.get(&a).map_or(a.entry(), String::as_str))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_rules::schema::Import;

    fn ruleset(untrimmed: Option<bool>) -> Ruleset {
        let mut r = Ruleset {
            schema: glasnost_rules::schema::SCHEMA_VERSION,
            ..Ruleset::default()
        };
        if let Some(u) = untrimmed {
            let mut i = Import::default();
            i.quirks.insert("list_items_untrimmed".into(), u);
            r.import = Some(i);
        }
        r.registries
            .insert("buildings".into(), vec!["Shed".into(), "Hut".into()]);
        r.registries.insert("infantry".into(), vec!["Grunt".into()]);
        r.registries
            .insert("animations".into(), vec!["Puff".into()]);
        let mut hut = ImportedType::default();
        hut.ini.insert("DeploySound".into(), "Whirr".into());
        hut.ini.insert("PackupSound".into(), "Clunk".into());
        hut.ini.insert("CrushSound".into(), String::new());
        r.types
            .entry("buildings".into())
            .or_default()
            .insert("Hut".into(), hut);
        let mut grunt = ImportedType::default();
        grunt
            .ini
            .insert("VoiceComment".into(), "Mutter, Grumble,,Hmm".into());
        grunt.ini.insert("EnterWaterSound".into(), "Splish".into());
        r.types
            .entry("infantry".into())
            .or_default()
            .insert("Grunt".into(), grunt);
        r.sim.infantry.insert(
            "Grunt".into(),
            glasnost_rules::sim::UnitType {
                voice_select: Some(vec!["Yes1".into(), "Yes2".into()]),
                ..Default::default()
            },
        );
        r.sim.general.chrono_in_sound = Some("Arrive".into());
        let mut av = BTreeMap::new();
        av.insert("SellSound".into(), "Kaching".into());
        av.insert("CreditTicks".into(), "Up,Down".into());
        av.insert("RadarOn".into(), "Bleep".into());
        r.ini_globals.insert("AudioVisual".into(), av);
        let mut puff = BTreeMap::new();
        puff.insert("Report".into(), "Poof".into());
        r.art_sections.insert("Puff".into(), puff);
        r
    }

    // spec: S-0408, S-0410, S-0416, S-0415, S-0210, S-0107, S-0214, S-0207, S-0109 Import mappings
    #[test]
    fn names_come_from_the_import_mapping_keys() {
        let n = SoundNames::from_ruleset(&ruleset(None));
        assert_eq!(n.sell.as_deref(), Some("Kaching"));
        assert_eq!(n.radar_on.as_deref(), Some("Bleep"));
        assert_eq!(n.radar_off, None);
        assert_eq!(n.credit_ticks, ["Up", "Down"]);
        assert!(n.of_type(Kind::Building, 0).is_none());
        let hut = n.of_type(Kind::Building, 1).unwrap();
        assert_eq!(hut.deploy.as_deref(), Some("Whirr"));
        assert_eq!(hut.packup.as_deref(), Some("Clunk"));
        assert_eq!(hut.crush, None, "an empty value is no sound");
        let grunt = n.of_type(Kind::Infantry, 0).unwrap();
        assert_eq!(grunt.voice_select, ["Yes1", "Yes2"]);
        assert_eq!(grunt.enter_water.as_deref(), Some("Splish"));
        assert_eq!(grunt.chrono_in.as_deref(), Some("Arrive"));
        assert_eq!(n.anim_report("PUFF"), Some("Poof"));
        assert_eq!(
            n.announcement(Announcement::CannotDeploy),
            Some("EVA_CannotDeployHere")
        );
        // spec: S-0701 Import mapping (announcement requests: the fixed entry of each event)
        assert_eq!(n.announcement(Announcement::UnitSold), Some("EVA_UnitSold"));
        assert_eq!(
            n.announcement(Announcement::LowFunds),
            Some("EVA_InsufficientFunds")
        );
        assert_eq!(
            n.announcement(Announcement::Promoted),
            Some("EVA_UnitPromoted")
        );
        let mut m = n.clone();
        m.set_announcement(Announcement::UnitSold, "Other");
        assert_eq!(m.announcement(Announcement::UnitSold), Some("Other"));
    }

    // spec: S-0701 Import mapping (the response, event and game-wide keys; `[AI] BuildConst` first three; weapon `Report`; animation `StartSound`, `StopSound`)
    #[test]
    fn s0701_keys_come_from_the_import_mapping() {
        let mut r = ruleset(None);
        r.registries
            .insert("vehicles".into(), vec!["Tank".into(), "fv".into()]);
        r.registries
            .insert("animations".into(), vec!["Puff".into(), "Flash".into()]);
        let mut tank = ImportedType::default();
        for (k, v) in [
            ("VoiceMove", "Go1,Go2"),
            ("VoiceAttack", "Hit"),
            ("VoiceSpecialAttack", "Odd"),
            ("VoiceEnter", "In"),
            ("VoiceCapture", "Take"),
            ("VoiceHarvest", "Dig"),
            ("VoiceDeploy", "Up"),
            ("VoiceFeedback", "Help"),
            ("DieSound", "Boom1,Boom2"),
            ("DamageSound", "Ouch"),
            ("MoveSound", "Rumble"),
            ("TurretRotateSound", "Whirr"),
            ("CreateSound", "Ready"),
        ] {
            tank.ini.insert(k.into(), v.into());
        }
        let vehicles = r.types.entry("vehicles".into()).or_default();
        vehicles.insert("Tank".into(), tank);
        let mut fv = ImportedType::default();
        fv.ini.insert("VoiceAttack".into(), "Zap".into());
        vehicles.insert("fv".into(), fv);
        let mut gun = ImportedType::default();
        gun.ini.insert("Report".into(), "Bang1,Bang2".into());
        r.types
            .entry("weapons".into())
            .or_default()
            .insert("Gun".into(), gun);
        let mut flash = BTreeMap::new();
        flash.insert("StartSound".into(), "Pop".into());
        flash.insert("Report".into(), "NotThis".into());
        flash.insert("StopSound".into(), "Fizzle".into());
        r.art_sections.insert("Flash".into(), flash);
        let av = r.ini_globals.get_mut("AudioVisual").unwrap();
        for (k, v) in [
            ("CreateUnitSound", "Wheels"),
            ("CreateInfantrySound", "Boots"),
            ("CreateAircraftSound", "Wings"),
            ("BuildingSlam", "Slam"),
            ("UpgradeVeteranSound", "Vet"),
            ("UpgradeEliteSound", "Elite"),
            ("BaseUnderAttackSound", "Siren"),
            ("GuardSound", "G"),
            ("StopSound", "S"),
            ("ScatterSound", "X"),
            ("DeploySound", "D"),
            ("ScoldSound", "Nope"),
            ("GUIBuildSound", "Click"),
            ("VoiceIFVRepair", "Mend"),
            ("SpeakDelay", "2.5"),
        ] {
            av.insert(k.into(), v.into());
        }
        let mut ai = BTreeMap::new();
        ai.insert("BuildConst".into(), "Hut,Nothing,Shed,Hut".into());
        r.ini_globals.insert("AI".into(), ai);
        let n = SoundNames::from_ruleset(&r);
        let t = n.of_type(Kind::Vehicle, 0).unwrap();
        assert_eq!(t.voice_move, ["Go1", "Go2"]);
        assert_eq!(t.voice_attack, ["Hit"]);
        assert_eq!(t.voice_special_attack, ["Odd"]);
        assert_eq!(t.voice_enter.as_deref(), Some("In"));
        assert_eq!(t.voice_capture.as_deref(), Some("Take"));
        assert_eq!(t.voice_harvest.as_deref(), Some("Dig"));
        assert_eq!(t.voice_deploy.as_deref(), Some("Up"));
        assert_eq!(t.voice_feedback, ["Help"]);
        assert_eq!(t.die_sound, ["Boom1", "Boom2"]);
        assert_eq!(t.damage_sound.as_deref(), Some("Ouch"));
        assert_eq!(t.move_sound, ["Rumble"]);
        assert_eq!(t.turret_rotate.as_deref(), Some("Whirr"));
        assert_eq!(t.create.as_deref(), Some("Ready"));
        assert!(!t.repair_vehicle);
        assert!(n.of_type(Kind::Vehicle, 1).unwrap().repair_vehicle);
        assert_eq!(n.report("GUN"), ["Bang1", "Bang2"]);
        assert!(n.report("Other").is_empty());
        assert_eq!(n.anim_report("Flash"), Some("Pop"));
        assert_eq!(n.anim_stop("flash"), Some("Fizzle"));
        assert_eq!(n.anim_report("Puff"), Some("Poof"));
        assert_eq!(n.create_unit.as_deref(), Some("Wheels"));
        assert_eq!(n.create_infantry.as_deref(), Some("Boots"));
        assert_eq!(n.create_aircraft.as_deref(), Some("Wings"));
        assert_eq!(n.building_slam.as_deref(), Some("Slam"));
        assert_eq!(n.upgrade_veteran.as_deref(), Some("Vet"));
        assert_eq!(n.upgrade_elite.as_deref(), Some("Elite"));
        assert_eq!(n.base_under_attack.as_deref(), Some("Siren"));
        assert_eq!(n.guard.as_deref(), Some("G"));
        assert_eq!(n.stop.as_deref(), Some("S"));
        assert_eq!(n.scatter.as_deref(), Some("X"));
        assert_eq!(n.deploy_command.as_deref(), Some("D"));
        assert_eq!(n.scold.as_deref(), Some("Nope"));
        assert_eq!(n.gui_build.as_deref(), Some("Click"));
        assert_eq!(n.voice_ifv_repair.as_deref(), Some("Mend"));
        assert!(n.speak_delay.is_some());
        // Only the first three entries count; unknown names are dropped.
        assert_eq!(n.build_const, [1, 0]);
    }

    // spec: S-0003 claim 12 (list items untrimmed with the switch on, trimmed with it off)
    #[test]
    fn list_items_follow_the_untrimmed_switch() {
        let on = SoundNames::from_ruleset(&ruleset(Some(true)));
        assert_eq!(
            on.of_type(Kind::Infantry, 0).unwrap().voice_comment,
            ["Mutter", " Grumble", "Hmm"]
        );
        let off = SoundNames::from_ruleset(&ruleset(Some(false)));
        assert_eq!(
            off.of_type(Kind::Infantry, 0).unwrap().voice_comment,
            ["Mutter", "Grumble", "Hmm"]
        );
    }

    #[test]
    fn high_bytes_survive_the_text_form() {
        let s = "A\u{F7E9}b".to_owned();
        assert_eq!(bytes_of(&s), [b'A', 0xE9, b'b']);
        assert_eq!(text_of(&bytes_of(&s)), s);
    }
}
