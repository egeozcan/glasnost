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
    /// `VoiceSelect`: the selection responses in the listed order, every
    /// entry kept (S-0214 Import mapping, F5).
    pub voice_select: Vec<String>,
    /// `ChronoInSound`, falling back to the game-wide one (S-0207 Import
    /// mapping).
    pub chrono_in: Option<String>,
    /// `ChronoOutSound`, falling back to the game-wide one (S-0207 Import
    /// mapping).
    pub chrono_out: Option<String>,
}

/// Announcements the specs mention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Announcement {
    /// A deploy refused (S-0415 claim 4): entry `EVA_CannotDeployHere`.
    CannotDeploy,
    /// A unit sold (S-0408 claim 11): no Import mapping (Q-0261).
    UnitSold,
    /// A building item done (S-0411 claims 9 and 12): no Import mapping
    /// (Q-0261).
    ConstructionComplete,
    /// Repair complete (S-0314 claim 6): no Import mapping (Q-0261).
    RepairComplete,
    /// Low funds (S-0314 claim 6): no Import mapping (Q-0261).
    LowFunds,
    /// A house defeated (S-0414 claim 5): no Import mapping (Q-0261).
    Defeat,
}

/// The rules data's sound names, by role.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SoundNames {
    types: BTreeMap<(u8, u32), TypeSounds>,
    anim_report: BTreeMap<String, String>,
    /// `[AudioVisual] SellSound` (S-0408 Import mapping).
    pub sell: Option<String>,
    /// `[AudioVisual] RadarOn` (S-0410 Import mapping).
    pub radar_on: Option<String>,
    /// `[AudioVisual] RadarOff` (S-0410 Import mapping).
    pub radar_off: Option<String>,
    /// `[AudioVisual] CreditTicks`: first entry rising, second falling
    /// (S-0416 Import mapping).
    pub credit_ticks: Vec<String>,
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
            ..Self::default()
        };
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
                        list_field(t.ini.get("VoiceComment"), untrimmed)
                    } else {
                        Vec::new()
                    },
                    // spec: S-0214 Import mapping (`VoiceSelect`, every entry kept in order; absent: empty)
                    voice_select: voice_select.unwrap_or_default(),
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
        // spec: S-0109 Import mapping (`Report` in the animation's art section: its start sound)
        if let Some(reg) = r.registries.get("animations") {
            for name in reg {
                let section = r.art_sections.get(name).or_else(|| {
                    r.art_sections
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case(name))
                        .map(|(_, v)| v)
                });
                if let Some(report) = name_field(section.and_then(|s| s.get("Report"))) {
                    out.anim_report.insert(name.to_ascii_uppercase(), report);
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

    /// An animation type's start sound (S-0109 claim 13), by type name
    /// (ASCII case folded, S-0003 claim 14).
    pub fn anim_report(&self, anim: &str) -> Option<&str> {
        self.anim_report
            .get(&anim.to_ascii_uppercase())
            .map(String::as_str)
    }

    /// Sets an animation type's start sound (tests and tools).
    pub fn set_anim_report(&mut self, anim: &str, sound: &str) {
        self.anim_report
            .insert(anim.to_ascii_uppercase(), sound.to_owned());
    }

    /// The speech database entry of an announcement, where a spec's Import
    /// mapping names one; `None` otherwise (Q-0261).
    pub fn announcement(&self, a: Announcement) -> Option<&'static str> {
        match a {
            // spec: S-0415 Import mapping (`EVA_CannotDeployHere`, the refusal announcement's entry)
            Announcement::CannotDeploy => Some("EVA_CannotDeployHere"),
            // No Import mapping gives these entries (Q-0261).
            Announcement::UnitSold
            | Announcement::ConstructionComplete
            | Announcement::RepairComplete
            | Announcement::LowFunds
            | Announcement::Defeat => None,
        }
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
        assert_eq!(n.announcement(Announcement::UnitSold), None);
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
