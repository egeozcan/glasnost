//! Tests of the combat fields of the `sim` section on invented data (every
//! name and number here is made up).

use glasnost_ini::binfloat::{self, Binary, Format};
use glasnost_ini::value::{self, DecimalField};
use glasnost_num::Rounding;
use glasnost_num::decimal::nearest_double;
use glasnost_rules::sim::{ArmourClass, SimData};

use super::*;
use crate::inputs::{InputFile, Inputs};
use crate::layers::{Layer, Role, Warning};
use crate::{ImportOptions, convert};

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

/// Converts base rules, a game-mode layer, a map and art, and resolves the
/// tree; returns the typed `sim` data, the import's warnings and the
/// recorded quirk switches.
fn import(
    base: &str,
    mode: &str,
    map: &str,
    art: &str,
    options: &ImportOptions,
) -> (SimData, Vec<Warning>, BTreeMap<String, bool>) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        mode: Some(file("mode.ini", mode)),
        map: Some(file("test.map", map)),
        art: Some(file("art.ini", art)),
        ..Inputs::default()
    };
    let import = convert(&inputs, options);
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    let quirks = r.ruleset.import.map(|i| i.quirks).unwrap_or_default();
    (r.ruleset.sim, import.warnings, quirks)
}

fn sim(base: &str, map: &str, art: &str) -> (SimData, Vec<Warning>) {
    let (s, w, _) = import(base, "", map, art, &ImportOptions::default());
    (s, w)
}

/// Builds the `sim` data from layers whose `[Warheads]` sections named in
/// `weapons` and `projectiles` are moved to those registries (weapons and
/// projectiles exist by reference only, S-0004 claim 3; listing them as
/// warheads keeps each test's fixture short).
fn direct(
    layers: &[&str],
    art: &str,
    weapons: &[&str],
    projectiles: &[&str],
    ini: IniQuirks,
) -> (SimData, Vec<Warning>) {
    let roles = [Role::Base, Role::Mode, Role::Map];
    let layers: Vec<Layer> = layers
        .iter()
        .zip(roles)
        .map(|(text, role)| Layer {
            role,
            file: format!("{role:?}.ini"),
            doc: Document::parse(text.as_bytes(), ini, &mut Vec::new()),
        })
        .collect();
    let art = Document::parse(art.as_bytes(), ini, &mut Vec::new());
    let mut outcome = crate::layers::apply(
        &layers,
        &art,
        "art.ini",
        Default::default(),
        &LayerQuirks::default(),
    );
    let warheads = outcome.registries[&Kind::Warheads].clone();
    let take = |names: &[&str]| -> Vec<TypeRec> {
        warheads
            .iter()
            .filter(|t| names.iter().any(|n| n.as_bytes() == t.name.as_slice()))
            .cloned()
            .collect()
    };
    outcome.registries.insert(Kind::Weapons, take(weapons));
    outcome
        .registries
        .insert(Kind::Projectiles, take(projectiles));
    if let Some(w) = outcome.registries.get_mut(&Kind::Warheads) {
        w.retain(|t| {
            !weapons
                .iter()
                .chain(projectiles)
                .any(|n| n.as_bytes() == t.name.as_slice())
        });
    }
    let (node, warnings) = build(SimInputs {
        outcome: &outcome,
        art: &art,
        arts: None,
        quirks: &LayerQuirks::default(),
        art_quirks: &ArtQuirks::default(),
        ini_quirks: &ini,
        sim_quirks: &SimQuirks::default(),
    });
    let parsed: SimData = glasnost_rules::de::from_node(&node).unwrap_or_else(|e| panic!("{e}"));
    (parsed, warnings)
}

fn dec(d: &Option<glasnost_rules::sim::Decimal>) -> Option<&str> {
    d.as_ref().map(|d| d.0.as_str())
}

fn texts(v: &Option<Vec<String>>) -> Vec<&str> {
    v.iter().flatten().map(String::as_str).collect()
}

fn warned(w: &[Warning], text: &str) -> bool {
    w.iter().any(|w| w.message.contains(text))
}

/// spec: S-0301 Import mapping, S-0306 Import mapping, S-0307 claim 1 and Import mapping (weapon fields; Speed and the lepton ranges: −1 keeps the held value)
#[test]
fn weapon_fields_and_not_given_values() {
    let base = "[Warheads]\n1=Zap\n2=Boom\n3=Dart\n\
                [Zap]\nROF=40\nBurst=2\nSpeed=30\nRange=5.5\nMinimumRange=1\nCellRangefinding=yes\n\
                IsSonic=yes\nUseSparkParticles=no\nUseFireParticles=1\nIsRailgun=yes\nIsMagBeam=no\n\
                Temporal=yes\nSpawner=no\nDiskLaser=yes\nDecloakToFire=no\nLobber=yes\nNeverUse=no\n\
                DrainWeapon=yes\nFireWhileMoving=no\nSuicide=yes\nSupress=yes\nFireOnce=no\nRevealOnFire=yes\n\
                Warhead=Boom\nProjectile=Dart\nDamage=10\n[Boom]\nCellSpread=1\n[Dart]\nROT=0\n";
    let (s, w) = direct(&[base], "", &["Zap"], &["Dart"], IniQuirks::default());
    let z = &s.weapons["Zap"];
    assert_eq!((z.rof, z.burst, z.speed), (Some(40), Some(2), Some(30)));
    assert_eq!(dec(&z.range), Some("55e-1"));
    assert_eq!(dec(&z.minimum_range), Some("1"));
    assert_eq!(z.cell_rangefinding, Some(true));
    assert_eq!(
        (z.is_sonic, z.use_spark_particles),
        (Some(true), Some(false))
    );
    assert_eq!(
        (z.use_fire_particles, z.is_railgun),
        (Some(true), Some(true))
    );
    assert_eq!(
        (z.is_mag_beam, z.temporal, z.spawner),
        (Some(false), Some(true), Some(false))
    );
    assert_eq!((z.disk_laser, z.decloak_to_fire), (Some(true), Some(false)));
    assert_eq!(
        (z.lobber, z.never_use, z.drain_weapon),
        (Some(true), Some(false), Some(true))
    );
    assert_eq!((z.fire_while_moving, z.suicide), (Some(false), Some(true)));
    assert_eq!((z.supress, z.fire_once), (Some(true), Some(false)));
    // spec: S-0409 Import mapping (RevealOnFire, a boolean)
    assert_eq!(z.reveal_on_fire, Some(true));
    assert!(!warned(&w, "Zap Range"));
    // Another negative value replaces the held one, with a warning; −2 is unlimited and quiet.
    let (s, w) = direct(
        &[base, "[Zap]\nRange=-3\nMinimumRange=-2\n"],
        "",
        &["Zap"],
        &["Dart"],
        IniQuirks::default(),
    );
    assert_eq!(dec(&s.weapons["Zap"].range), Some("-3"));
    assert!(warned(&w, "Zap Range -3 is negative"));
    assert!(!warned(&w, "MinimumRange -2"));
    // The "not given" values of the layering (the layering's own test of a stored value is
    // checked here directly).
    use crate::layers::{global_value_not_given, type_value_not_given};
    assert!(type_value_not_given(Kind::Weapons, b"Speed", b"$FFFFFFFF"));
    assert!(!type_value_not_given(Kind::Weapons, b"Speed", b"-2"));
    for key in [&b"Range"[..], b"MinimumRange"] {
        assert!(type_value_not_given(Kind::Weapons, key, b"-1.0"));
        assert!(type_value_not_given(Kind::Weapons, key, b"-100%"));
        assert!(!type_value_not_given(Kind::Weapons, key, b"-1.5"));
    }
    assert!(type_value_not_given(
        Kind::Buildings,
        b"AirRangeBonus",
        b"-1"
    ));
    assert!(global_value_not_given(COMBAT_DAMAGE, b"FireSupress", b"-1"));
    // End to end: AirRangeBonus and BallisticScatter given as −1 keep the held value.
    let (s, _) = sim(
        "[VehicleTypes]\n1=Cart\n[Cart]\nAirRangeBonus=2.5\n[CombatDamage]\nBallisticScatter=1.5\n",
        "[Cart]\nAirRangeBonus=-1\n[CombatDamage]\nBallisticScatter=-1\n",
        "",
    );
    assert_eq!(
        dec(&s.vehicles["Cart"].combat.air_range_bonus),
        Some("25e-1")
    );
    assert_eq!(dec(&s.general.ballistic_scatter), Some("15e-1"));
}

/// spec: S-0306 Import mapping (projectile fields; art Voxel of the projectile image)
#[test]
fn projectile_fields() {
    let base = "[Warheads]\n1=Dart\n2=Zap\n\
                [Dart]\nAA=yes\nArcing=no\nVertical=no\nInviso=yes\nInaccurate=yes\nFlakScatter=no\n\
                Dropping=no\nAirburst=yes\nVeryHigh=no\nLevel=yes\nFloater=no\nSubjectToCliffs=yes\n\
                SubjectToWalls=no\nSubjectToElevation=yes\nRanged=no\nDegenerates=yes\nScalable=no\n\
                Acceleration=4\nCourseLockDuration=2\nArm=3\nDetonationAltitude=600\nCluster=5\n\
                AirburstWeapon=zap\nROT=8\nImage=DARTART\n\
                [ElevationModel]\nElevationIncrement=2\n";
    let art = "[DARTART]\nVoxel=yes\nTrailer=Puff\nSpawnDelay=6\n";
    let (s, _) = direct(&[base], art, &["Zap"], &["Dart"], IniQuirks::default());
    let p = &s.projectiles["Dart"];
    assert_eq!(
        (p.aa, p.arcing, p.vertical, p.inviso),
        (Some(true), Some(false), Some(false), Some(true))
    );
    assert_eq!(
        (p.inaccurate, p.flak_scatter, p.dropping),
        (Some(true), Some(false), Some(false))
    );
    assert_eq!(
        (p.airburst, p.very_high, p.level, p.floater),
        (Some(true), Some(false), Some(true), Some(false))
    );
    assert_eq!(
        (
            p.subject_to_cliffs,
            p.subject_to_walls,
            p.subject_to_elevation
        ),
        (Some(true), Some(false), Some(true))
    );
    assert_eq!(
        (p.ranged, p.degenerates, p.scalable),
        (Some(false), Some(true), Some(false))
    );
    assert_eq!(
        (p.acceleration, p.course_lock_duration, p.arm),
        (Some(4), Some(2), Some(3))
    );
    assert_eq!(
        (p.detonation_altitude, p.cluster, p.spawn_delay),
        (Some(600), Some(5), Some(6))
    );
    // spec: S-0003 claim 14 (a type name resolves to the type's key whatever its letter case)
    assert_eq!(p.airburst_weapon.as_deref(), Some("Zap"));
    // An animation no registry holds keeps its name as written.
    assert_eq!(p.trailer.as_deref(), Some("Puff"));
    assert_eq!(p.voxel, Some(true));
}

/// spec: S-0301 Edge cases, S-0302 Edge cases, S-0303 Edge cases, S-0306 Edge cases, S-0307 Edge cases (importer warnings of weapons and projectiles)
#[test]
fn weapon_and_projectile_warnings() {
    let base = "[Warheads]\n1=Zap\n2=Boom\n3=Dart\n4=Lone\n5=Lob\n6=Flak\n7=Mad\n\
                [Zap]\nBurst=0\nROF=-3\nDamage=-20000000\nWarhead=Boom\nProjectile=Dart\nSpeed=0\n\
                [Boom]\nAnimList=Pop,Bang\n[Dart]\nROT=-1\nCluster=0\nCourseLockDuration=-1\n\
                [Lob]\nProjectile=Flak\nWarhead=Mad\nDamage=0\n\
                [Flak]\nFlakScatter=yes\nInaccurate=yes\nArcing=yes\nVertical=yes\nImage=FlakArt\n\
                SubjectToElevation=yes\n[Mad]\nPsychedelic=yes\n[Lone]\nDamage=1\n\
                [ElevationModel]\nElevationBonusCap=-150\n";
    let (_, w) = direct(
        &[base],
        "[FlakArt]\nTrailer=Puff\nSpawnDelay=0\n",
        &["Zap", "Lob", "Lone"],
        &["Dart", "Flak"],
        IniQuirks::default(),
    );
    for text in [
        "Zap Burst 0 is below 1",
        "Zap ROF -3 is negative",
        "Zap Damage -20000000 is above 2^24",
        "Zap has negative damage and its warhead an animation list",
        "Zap has speed 0 and a projectile with a turn rate",
        "Dart ROT -1 is negative",
        "Dart Cluster 0 is not above 0",
        "Dart CourseLockDuration -1 is negative",
        "Flak is vertical with a detonation altitude of 0",
        "Flak is arcing and also steering or vertical",
        "Flak has a trailer and SpawnDelay 0",
        "Lob is a flak-scattering inaccurate arcing weapon with range 0",
        "Lob has a berserk warhead and damage 0",
        "Lone has no warhead",
        "ElevationIncrement is not above 0",
        "ElevationBonusCap is negative",
        "ElevationBonusCap is 148 or more",
    ] {
        assert!(warned(&w, text), "{text}: {w:#?}");
    }
}

/// spec: S-0003 claim 13 (exact doubles; `quirks.percent_list_integer_prefix` on and off; the warnings), S-0305 claim 1 (class order), S-0006 claim 5 (Verses resets on override)
#[test]
fn verses_are_exact_doubles() {
    let base = "[Warheads]\n1=Pow\n2=Bam\n[Pow]\nVerses=50%,12.5%,0.1,1e400,-0,abc,2,100%,25%,0%,7%,9%\n\
                [Bam]\nVerses=10%\n";
    let (s, w) = direct(&[base], "", &[], &[], IniQuirks::default());
    let pow: Vec<String> = s.warheads["Pow"]
        .verses
        .clone()
        .unwrap()
        .into_iter()
        .map(|d| d.0)
        .collect();
    let c = binfloat::percent_factor();
    let pct =
        |i: u128| Binary::round_toward_zero(Format::Double, false, i, 0).mul_rz(&c, Format::Double);
    let (want, _) = value::percent_list(
        b"50%,12.5%,0.1,1e400,-0,abc,2,100%,25%,0%,7%,9%",
        value::ARMOUR_CLASS_COUNT,
        true,
        true,
    );
    assert_eq!(pow.len(), 11);
    assert_eq!(pow[0], "0.5");
    assert_eq!(pow[1], pct(12).to_exact_decimal());
    assert_eq!(pow[3], "1e400");
    assert_eq!(pow[4], "-0");
    assert_eq!(pow[5], "0");
    assert_eq!(pow[6], "2");
    // Every item reads back to the same double at 53 bits (glasnost_num's nearest double of the text).
    // (`nearest_double` gives +0 for "-0": the two zeros are equal values.)
    for (text, b) in pow.iter().zip(&want) {
        let zero = text == "-0";
        assert_eq!(
            nearest_double(text).0,
            if zero { 0 } else { b.bits },
            "{text}"
        );
    }
    // Items that are exactly singles also read back through Decimal::value (which reads at 24 bits).
    for i in [0, 4, 5, 6, 7] {
        let d = glasnost_rules::sim::Decimal(pow[i].clone());
        assert_eq!(d.value(Rounding::TowardZero).0, want[i].bits, "{}", pow[i]);
    }
    assert!(warned(&w, "Pow Verses \"50%,12.5%"));
    assert!(warned(&w, "1 items beyond 11"));
    assert!(warned(&w, "1 items have no number"));
    assert!(warned(&w, "1 items with `%` use only their integer part"));
    assert!(warned(&w, "2 items without `%` are above 1"));
    // Bam: one item, ten filled with 100 %.
    let bam: Vec<String> = s.warheads["Bam"]
        .verses
        .clone()
        .unwrap()
        .into_iter()
        .map(|d| d.0)
        .collect();
    assert_eq!(bam[1..], vec!["1".to_string(); 10][..]);
    assert!(warned(&w, "Bam Verses \"10%\" has 1 of 11 items"));
    // Switch off: `12.5%` is 0.125 exactly, and no integer-prefix warning.
    let off = IniQuirks {
        percent_list_integer_prefix: false,
        ..IniQuirks::default()
    };
    let (s, w) = direct(&[base], "", &[], &[], off);
    assert_eq!(s.warheads["Pow"].verses.as_ref().unwrap()[1].0, "0.125");
    assert!(!warned(&w, "use only their integer part"));
    // A later layer's section without the key resets the list (absent: 100 % each).
    let (s, _) = direct(
        &[base, "[Pow]\nCellSpread=1\n"],
        "",
        &[],
        &[],
        IniQuirks::default(),
    );
    assert_eq!(s.warheads["Pow"].verses, None);
    // The recorded switch.
    let (_, _, q) = import(base, "", "", "", &ImportOptions::default());
    assert_eq!(q.get("percent_list_integer_prefix"), Some(&true));
}

/// spec: S-0003 claim 13 and Formulas (a `%` item of Verses is Q₅₃(i × c): RZ₅₃ with quirks.chop_rounding on, RN₅₃ off; the switch is recorded)
#[test]
fn verses_percent_rounding_follows_chop() {
    let base = "[Warheads]\n1=Pow\n[Pow]\nVerses=10%\n";
    let (s, _, q) = import(base, "", "", "", &ImportOptions::default());
    let on = s.warheads["Pow"].verses.clone().unwrap();
    assert_eq!(nearest_double(&on[0].0).0, 0x3FB9_9999_9999_9999);
    assert_eq!(q.get("chop_rounding"), Some(&true));
    let options = ImportOptions {
        sim: SimQuirks {
            chop_rounding: false,
            ..SimQuirks::default()
        },
        ..ImportOptions::default()
    };
    let (s, _, q) = import(base, "", "", "", &options);
    let off = s.warheads["Pow"].verses.clone().unwrap();
    assert_eq!(nearest_double(&off[0].0).0, 0x3FB9_9999_9999_999A);
    assert_eq!(q.get("chop_rounding"), Some(&false));
}

/// spec: S-0302 Import mapping, S-0303 Import mapping, S-0308 Import mapping (warhead fields), S-0303 and S-0312 Edge cases (warhead warnings)
#[test]
fn warhead_fields_and_warnings() {
    let base = "[Warheads]\n1=Pow\n2=Fizz\n3=Hush\n[ParticleSystems]\n1=Smog\n\
                [Pow]\nCellSpread=1.5\nPercentAtMax=50%\nProneDamage=0.25\nPenetratesBunker=yes\n\
                Radiation=no\nPsychicDamage=yes\nPoison=no\nAffectsAllies=no\nPsychedelic=yes\nSparky=no\n\
                CausesDelayKill=yes\nDelayKillFrames=9\nDelayKillAtMax=0.5\nWallAbsoluteDestroyer=yes\n\
                Tiberium=no\nRocker=yes\nParticle=SMOG\nConventional=yes\nEMEffect=no\nAnimList=Pop,none,Bang\n\
                MaxDebris=3\nMinDebris=1\nDebrisTypes=Bit,Bob\nDebrisMaximums=0,0\nAirstrike=no\nIsLocomotor=yes\n\
                [Fizz]\nCellSpread=12\nMaxDebris=2\n[Hush]\nCausesDelayKill=yes\nCellSpread=0.5\n";
    let (s, w) = direct(&[base], "", &[], &[], IniQuirks::default());
    let h = &s.warheads["Pow"];
    assert_eq!(dec(&h.cell_spread), Some("15e-1"));
    assert_eq!(dec(&h.percent_at_max), Some("50%"));
    assert_eq!(dec(&h.prone_damage), Some("25e-2"));
    assert_eq!(
        (h.penetrates_bunker, h.radiation, h.psychic_damage),
        (Some(true), Some(false), Some(true))
    );
    assert_eq!(
        (h.poison, h.affects_allies, h.psychedelic, h.sparky),
        (Some(false), Some(false), Some(true), Some(false))
    );
    assert_eq!(
        (h.causes_delay_kill, h.delay_kill_frames),
        (Some(true), Some(9))
    );
    assert_eq!(dec(&h.delay_kill_at_max), Some("5e-1"));
    assert_eq!(
        (h.wall_absolute_destroyer, h.tiberium, h.rocker),
        (Some(true), Some(false), Some(true))
    );
    assert_eq!(h.particle.as_deref(), Some("Smog"));
    assert_eq!((h.conventional, h.em_effect), (Some(true), Some(false)));
    // spec: S-0003 claim 14 (`none` in a name list names no type)
    assert_eq!(texts(&h.anim_list), ["Pop", "Bang"]);
    assert_eq!((h.max_debris, h.min_debris), (Some(3), Some(1)));
    assert_eq!(texts(&h.debris_types), ["Bit", "Bob"]);
    assert_eq!(h.debris_maximums, Some(vec![0, 0]));
    assert_eq!((h.airstrike, h.is_locomotor), (Some(false), Some(true)));
    // spec: S-0003 claims 19 and 20 (scenarios V14 and V15)
    let lists = "[Warheads]\n1=Pow\n[Pow]\nAnimList=none,Pop,<NONE>,,Bang,Pop\nDebrisTypes=Bit\nDebrisMaximums=2,,abc,$10,10h\n";
    let (s, _) = direct(&[lists], "", &[], &[], IniQuirks::default());
    let h = &s.warheads["Pow"];
    assert_eq!(texts(&h.anim_list), ["Pop", "Bang", "Pop"]);
    assert_eq!(h.debris_maximums, Some(vec![2, 0, 0, 10]));
    // A list of no-type items alone replaces the held list with an empty one; an empty value keeps it.
    let over = "[Pow]\nAnimList=none,<none>\nDebrisTypes=\n";
    let (s, _) = direct(&[lists, over], "", &[], &[], IniQuirks::default());
    let h = &s.warheads["Pow"];
    assert!(texts(&h.anim_list).is_empty());
    assert_eq!(texts(&h.debris_types), ["Bit"]);
    for text in [
        "Fizz CellSpread 12 is outside 0 to 11",
        "Hush causes delayed kills with a spread below one cell",
        "Fizz has MaxDebris 2 but no DebrisTypes and the game-wide MetallicDebris list is empty",
        "Pow has MaxDebris 3 but every debris maximum is 0",
    ] {
        assert!(warned(&w, text), "{text}");
    }
    assert!(!warned(&w, "Pow causes delayed kills"));
}

/// spec: S-0305 claims 3 to 5 and Import mapping (armour words, an unknown word gives none, an absent key keeps the class, resource overlays)
#[test]
fn armour_classes() {
    let base = "[VehicleTypes]\n1=Cart\n2=Tank\n[BuildingTypes]\n1=Hut\n[TerrainTypes]\n1=Elm\n\
                [OverlayTypes]\n1=Ore\n2=Fence\n\
                [Cart]\nArmor=HEAVY\n[Tank]\nArmor=special_2\n[Hut]\nArmor=Concrete\n[Elm]\nArmor=wood\n\
                [Ore]\nTiberium=yes\nArmor=steel\n[Fence]\nArmor=woody\n";
    let map = "[Cart]\nArmor=heavy2\n[Tank]\nSpeed=4\n";
    let (s, w) = sim(base, map, "");
    assert_eq!(s.vehicles["Cart"].combat.armor, Some(ArmourClass::None));
    assert!(warned(
        &w,
        "vehicles Cart Armor \"heavy2\" is not an armour word"
    ));
    assert_eq!(s.vehicles["Tank"].combat.armor, Some(ArmourClass::Special2));
    assert_eq!(s.buildings["Hut"].combat.armor, Some(ArmourClass::Concrete));
    assert_eq!(s.terrain_objects["Elm"].armor, Some(ArmourClass::Wood));
    // Overlays keep the word as written; the warnings are the importer's.
    assert_eq!(s.overlays["Ore"].armor.as_deref(), Some("steel"));
    assert!(warned(
        &w,
        "overlays Ore is a resource overlay: its Armor \"steel\" is ignored"
    ));
    assert!(warned(
        &w,
        "overlays Fence Armor \"woody\" is not an armour word"
    ));
    // A type without the key has no class field (the simulation's none).
    let (s, _) = sim("[VehicleTypes]\n1=Cart\n[Cart]\nSpeed=2\n", "", "");
    assert_eq!(s.vehicles["Cart"].combat.armor, None);
}

/// spec: S-0308 claim 1 and Edge cases (weapon slots of both forms; the clear-weapons flag; `quirks.clear_weapons_any_turret_count` on and off; a weapon count above 18)
#[test]
fn weapon_slots_and_clear_weapons() {
    let base = "[VehicleTypes]\n1=Cart\n2=Multi\n3=Plain\n[BuildingTypes]\n1=Hut\n\
                [Cart]\nPrimary=Zap\nSecondary=Pop\nElitePrimary=Zip\n\
                [Multi]\nTurretCount=2\nWeaponCount=3\nWeapon1=A\nWeapon2=B\nWeapon3=C\nWeapon4=D\n\
                EliteWeapon2=EB\nPrimary=Ignored\n\
                [Plain]\nPrimary=Zap\nSecondary=none\n[Hut]\nPrimary=Gun\nElitePrimary=BigGun\n";
    let mode = "[Cart]\nClearAllWeapons=yes\nPrimary=New\n[Multi]\nClearAllWeapons=yes\n";
    let map = "[Cart]\nSecondary=Late\nClearAllWeapons=no\n";
    let (s, w, q) = import(base, mode, map, "", &ImportOptions::default());
    let cart = &s.vehicles["Cart"];
    // The mode layer's clear empties slots 0 and 1 even though it names a weapon; the map turns
    // the flag off and refills slot 1.
    assert_eq!(
        (cart.primary.as_deref(), cart.secondary.as_deref()),
        (None, Some("Late"))
    );
    assert_eq!(cart.elite_primary, None);
    assert_eq!(cart.combat.weapon_slots, None);
    let multi = &s.vehicles["Multi"];
    assert_eq!(
        multi.combat.weapon_slots,
        Some(vec![String::new(), String::new(), "C".into()])
    );
    assert_eq!(multi.combat.elite_weapon_slots, Some(vec![]));
    assert_eq!(multi.primary, None);
    let plain = &s.vehicles["Plain"];
    assert_eq!(
        (plain.primary.as_deref(), plain.secondary.as_deref()),
        (Some("Zap"), None)
    );
    let hut = &s.buildings["Hut"];
    assert_eq!(hut.primary.as_deref(), Some("Gun"));
    assert_eq!(
        hut.combat.elite_weapon_slots,
        Some(vec!["BigGun".to_string()])
    );
    assert_eq!(q.get("clear_weapons_any_turret_count"), Some(&true));
    assert!(!warned(&w, "WeaponCount"));
    // Switch off: the flag leaves a multi-weapon type alone.
    let options = ImportOptions {
        sim: SimQuirks {
            clear_weapons_any_turret_count: false,
            ..SimQuirks::default()
        },
        ..ImportOptions::default()
    };
    let (s, _, q) = import(base, mode, map, "", &options);
    assert_eq!(
        s.vehicles["Multi"].combat.weapon_slots,
        Some(vec!["A".into(), "B".into(), "C".into()])
    );
    assert_eq!(s.vehicles["Multi"].primary.as_deref(), Some("A"));
    assert_eq!(
        s.vehicles["Multi"].combat.elite_weapon_slots,
        Some(vec![String::new(), "EB".into()])
    );
    assert_eq!(s.vehicles["Cart"].primary, None);
    assert_eq!(q.get("clear_weapons_any_turret_count"), Some(&false));
    // A later section that leaves the flag set (by omitting it) clears again.
    let (s, _, _) = import(
        base,
        mode,
        "[Cart]\nSecondary=Late\n",
        "",
        &ImportOptions::default(),
    );
    assert_eq!(s.vehicles["Cart"].secondary, None);
    let (_, w) = sim(
        "[VehicleTypes]\n1=Cart\n[Cart]\nTurretCount=1\nWeaponCount=20\n",
        "",
        "",
    );
    assert!(warned(&w, "Cart WeaponCount 20 is above 18"));
}

/// spec: S-0309 claim 26, Import mapping and Edge cases (each coefficient starts at 0; `quirks.coefficient_zero_replaced` on and off; the default as it stands when the type is read; the warnings)
#[test]
fn threat_coefficients() {
    let base = "[VehicleTypes]\n1=Cart\n2=Tank\n3=Jeep\n\
                [General]\nMyEffectivenessCoefficientDefault=2\nTargetDistanceCoefficientDefault=0.5\n\
                [Cart]\nMyEffectivenessCoefficient=0\nTargetStrengthCoefficient=2000000\n[Tank]\nSpeed=1\n\
                [Jeep]\nSpeed=1\n";
    let mode =
        "[Cart]\nSpeed=3\n[General]\nTargetDistanceCoefficientDefault=0.25\n[Jeep]\nSpeed=2\n";
    let (s, w, q) = import(base, mode, "", "", &ImportOptions::default());
    let cart = s.vehicles["Cart"].combat.coefficients.clone().unwrap();
    let cart: Vec<&str> = cart.iter().map(|d| d.0.as_str()).collect();
    // The 0 holds after the base, then the mode layer's reading without the key takes the default 2.
    // The distance coefficient was copied in the base (0.5) and keeps it.
    assert_eq!(cart, ["2", "0", "0", "2000000", "5e-1"]);
    // Tank and Jeep were read in the base: their distance coefficient is the base's default, not the final one.
    let tank = s.vehicles["Tank"].combat.coefficients.clone().unwrap();
    assert_eq!(tank[4].0, "5e-1");
    assert_eq!(tank[0].0, "2");
    assert_eq!(
        dec(&s
            .general
            .coefficient_defaults
            .as_ref()
            .map(|d| d[4].clone())),
        Some("25e-2")
    );
    assert!(warned(
        &w,
        "Cart MyEffectivenessCoefficient is written as \"0\""
    ));
    assert!(warned(
        &w,
        "Cart TargetStrengthCoefficient 2000000 is above 10^6"
    ));
    assert_eq!(q.get("coefficient_zero_replaced"), Some(&true));
    // A type read only after the last change of the defaults takes no field.
    let (s, _, _) = import(
        "[VehicleTypes]\n1=Cart\n[General]\nMyEffectivenessCoefficientDefault=2\n[Cart]\nSpeed=1\n",
        "",
        "",
        "",
        &ImportOptions::default(),
    );
    assert_eq!(s.vehicles["Cart"].combat.coefficients, None);
    // Switch off: an absent key keeps the held 0.
    let options = ImportOptions {
        sim: SimQuirks {
            coefficient_zero_replaced: false,
            ..SimQuirks::default()
        },
        ..ImportOptions::default()
    };
    let (s, w, q) = import(base, mode, "", "", &options);
    let cart = s.vehicles["Cart"].combat.coefficients.clone().unwrap();
    assert_eq!(cart[0].0, "0");
    assert!(warned(
        &w,
        "Cart MyEffectivenessCoefficient is written as \"0\""
    ));
    assert_eq!(q.get("coefficient_zero_replaced"), Some(&false));
}

/// spec: S-0301, S-0302, S-0306, S-0307, S-0308, S-0309, S-0311, S-0312, S-0314, S-0315 Import mapping (every combat key of a mobile type and a building resolves; spot checks)
#[test]
fn every_combat_key_resolves() {
    fn keys(table: &[(&str, &str, Form, &str)], int: &str, dec: &str) -> String {
        table
            .iter()
            .map(|(_, k, f, _)| match f {
                Form::Int => format!("{k}={int}\n"),
                Form::Bool => format!("{k}=yes\n"),
                Form::Dec => format!("{k}={dec}\n"),
                Form::Text | Form::Ref(_) => format!("{k}=N{k}\n"),
                Form::Refs(_) | Form::Texts => format!("{k}=A{k},B{k}\n"),
                Form::Maxima => format!("{k}=1,2\n"),
            })
            .collect()
    }
    let shared = keys(combat::COMBAT_FIELDS, "3", "0.5");
    let unit = keys(combat::UNIT_COMBAT, "2", "1.5");
    let set_below = [
        "IFVMode",
        "OpenTransportWeapon",
        "DeployFireWeapon",
        "SuppressionThreshold",
    ];
    let cart_unit: String = unit
        .lines()
        .filter(|l| !set_below.iter().any(|k| l.starts_with(&format!("{k}="))))
        .map(|l| format!("{l}\n"))
        .collect();
    let base = format!(
        "[InfantryTypes]\n1=Walker\n[VehicleTypes]\n1=Cart\n[AircraftTypes]\n1=Kite\n\
         [BuildingTypes]\n1=Hut\n[ParticleSystems]\n1=Smog\n2=Glow\n[Countries]\n1=Ruritania\n\
         [Smog]\nBehavesLike=smoke\n[Glow]\nBehavesLike=Fire\n\
         [Walker]\n{shared}{unit}JumpJetTurn=yes\nOccupyWeapon=Gun\nEliteOccupyWeapon=BigGun\n\
         VeteranAbilities=ROF,faster\nEliteAbilities=ROF, faster\nStrength=50\nTargetEffectivenessCoefficient=1.5\n\
         [Cart]\n{shared}{cart_unit}HunterSeeker=yes\nGunner=no\nDeployToFire=yes\nBurstDelay1=4\nBurstDelay3=-2\n\
         DamageParticleSystems=glow,SMOG,Nope\nIFVMode=18\nOpenTransportWeapon=-2\nDeployFire=yes\nDeployFireWeapon=18\n\
         SuppressionThreshold=-1\nStrength=0\n\
         [Kite]\n{shared}{unit}\n\
         [Hut]\n{shared}BuildCat=Combat\nEligibleForDelayKill=yes\nLeaveRubble=no\nHelipad=yes\nArtillary=no\n\
         MaxNumberOccupants=11\nInfantryGainSelfHeal=-1\nUnitsGainSelfHeal=2\nStrength=3\nCost=-5\n\
         [Ruritania]\nFirepower=1.25\nROF=-0.5\nArmorInfantryMult=0.75\nArmorUnitsMult=1\nArmorAircraftMult=2\n\
         ArmorBuildingsMult=0.5\nArmorDefensesMult=25%\nArmor=1.5\nVeteranInfantry=walker,Nobody\n\
         VeteranUnits=Cart\nVeteranAircraft=KITE\n"
    );
    let (s, w) = sim(&base, "", "");
    let walker = &s.infantry["Walker"];
    assert_eq!(walker.combat.ammo, Some(3));
    assert_eq!(dec(&walker.combat.air_range_bonus), Some("5e-1"));
    assert_eq!(
        texts(&walker.combat.explosion),
        ["AExplosion", "BExplosion"]
    );
    assert_eq!(walker.combat.debris_maximums, Some(vec![1, 2]));
    assert_eq!(walker.combat.vhp_scan.as_deref(), Some("NVHPScan"));
    assert_eq!(walker.combat.death_weapon.as_deref(), Some("NDeathWeapon"));
    assert_eq!(walker.ifv_mode, Some(2));
    assert_eq!(dec(&walker.weight), Some("15e-1"));
    assert_eq!(
        (walker.jump_jet_turn, walker.occupy_weapon.as_deref()),
        (Some(true), Some("Gun"))
    );
    assert_eq!(walker.elite_occupy_weapon.as_deref(), Some("BigGun"));
    let coeff = walker.combat.coefficients.clone().unwrap();
    assert_eq!(
        coeff.iter().map(|d| d.0.as_str()).collect::<Vec<_>>(),
        ["0", "15e-1", "0", "0", "0"]
    );
    let cart = &s.vehicles["Cart"];
    assert_eq!(
        (cart.hunter_seeker, cart.gunner, cart.deploy_to_fire),
        (Some(true), Some(false), Some(true))
    );
    assert_eq!(cart.burst_delay, Some([-1, 4, -1, -2]));
    // spec: S-0302 Import mapping (only particle systems that behave like smoke, letter case ignored)
    assert_eq!(
        cart.combat.damage_smoke_systems,
        Some(vec!["Smog".to_string()])
    );
    assert_eq!(s.aircraft["Kite"].combat.is_gattling, Some(true));
    let hut = &s.buildings["Hut"];
    assert_eq!(hut.defense, Some(true));
    assert_eq!(hut.combat.trainable, Some(true));
    assert_eq!(
        (hut.eligible_for_delay_kill, hut.leave_rubble, hut.helipad),
        (Some(true), Some(false), Some(true))
    );
    assert_eq!(
        (
            hut.max_number_occupants,
            hut.infantry_gain_self_heal,
            hut.units_gain_self_heal
        ),
        (Some(11), Some(-1), Some(2))
    );
    let r = &s.countries["Ruritania"];
    assert_eq!(dec(&r.firepower), Some("125e-2"));
    assert_eq!(dec(&r.armor_defenses_mult), Some("25%"));
    // spec: S-0004 claim 13 (a country's veteran lists create the types they name)
    assert_eq!(
        r.veteran_infantry,
        Some(vec!["Walker".to_string(), "Nobody".to_string()])
    );
    assert_eq!(r.veteran_aircraft, Some(vec!["Kite".to_string()]));
    for text in [
        "infantry Walker EliteAbilities: \" faster\" names no ability",
        "vehicles Cart BurstDelay3 -2 is below −1",
        "vehicles Cart IFVMode 18 is outside 0 … 17",
        "vehicles Cart OpenTransportWeapon -2 is outside −1 … 17",
        "vehicles Cart DeployFireWeapon 18 is outside 0 … 17",
        "vehicles Cart SuppressionThreshold -1 is below 0",
        "vehicles Cart has DamageReducesReadiness and a strength of 0",
        "buildings Hut MaxNumberOccupants 11 is above 10",
        "buildings Hut InfantryGainSelfHeal -1 is negative",
        "buildings Hut Strength 3 is below the repair step 5",
        "buildings Hut Cost -5 is negative",
        "countries Ruritania Armor 1.5: the original ignores",
        "countries Ruritania ROF -0.5 is negative",
    ] {
        assert!(warned(&w, text), "{text}: {w:#?}");
    }
    assert!(!warned(&w, "Walker VeteranAbilities"));
}

/// spec: S-0313 Import mapping (muzzle positions and turret offset from the type's image section), S-0301 Import mapping (firing frames; building delayed fire), S-0306 Import mapping (aim shift, height), S-0310 Import mapping (voxel turret, pixel offsets); S-0003 claim 9 (an unread component keeps the value in force, with a warning)
#[test]
fn art_combat_fields() {
    let base = "[VehicleTypes]\n1=Cart\n2=Solo\n[BuildingTypes]\n1=Hut\n\
                [Cart]\nImage=CARTART\nTurretCount=1\nWeaponCount=2\n[Solo]\nImage=CARTART\n[Hut]\nSpeed=0\n";
    let art = "[CARTART]\nPrimaryFireFLH=100,-20,30\nEliteSecondaryFireFLH=5\nWeapon2FLH=1,2,3\n\
               TurretOffset=-40\nFiringFrames=3\nFiringSyncFrame1=2\n\
               [Hut]\nTargetCoordOffset=0,0,50\nHeight=4\nTurretAnimIsVoxel=yes\nBarrelAnimIsVoxel=no\n\
               PrimaryFirePixelOffset=0,-12\nPrimaryFireDualOffset=yes\nTurretAnimY=7\n\
               IsAnimDelayedFire=yes\nDelayedFireDelay=6\n";
    let (s, w) = sim(base, "", art);
    let c = &s.vehicles["Cart"];
    assert_eq!(c.combat.primary_fire_flh, Some([100, -20, 30]));
    assert_eq!(c.combat.elite_secondary_fire_flh, Some([5, 0, 0]));
    assert!(warned(
        &w,
        "art EliteSecondaryFireFLH \"5\" has fewer than 3 readable integers"
    ));
    assert_eq!(c.combat.weapon_flh, Some(vec![[0, 0, 0], [1, 2, 3]]));
    // A type with the primary and secondary pair keeps no numbered muzzle positions.
    let solo = &s.vehicles["Solo"];
    assert_eq!(solo.combat.weapon_flh, None);
    assert_eq!(solo.combat.primary_fire_flh, Some([100, -20, 30]));
    assert_eq!(c.combat.turret_offset, Some(-40));
    assert_eq!(
        (c.firing_frames, c.firing_sync_frames),
        (Some(3), Some([-1, 2]))
    );
    let h = &s.buildings["Hut"];
    assert_eq!(
        (h.target_coord_offset, h.art_height),
        (Some([0, 0, 50]), Some(4))
    );
    assert_eq!(
        (h.turret_anim_is_voxel, h.barrel_anim_is_voxel),
        (Some(true), Some(false))
    );
    assert_eq!(
        (h.primary_fire_pixel_offset, h.primary_fire_dual_offset),
        (Some([0, -12]), Some(true))
    );
    assert_eq!(h.turret_anim_xy, Some([0, 7]));
    assert_eq!(
        (h.is_anim_delayed_fire, h.delayed_fire_delay),
        (Some(true), Some(6))
    );
}

/// spec: S-0301 to S-0314 Import mapping (game-wide combat values, the difficulty rows, the overlay and terrain-object fields), S-0006 claim 5 (difficulty keys reset), the game-wide warnings of S-0301, S-0302, S-0306, S-0309, S-0311, S-0312 and S-0314 Edge cases
#[test]
fn game_wide_combat_values() {
    let base = "[VehicleTypes]\n1=Hauler\n[InfantryTypes]\n1=Crewman\n[Animations]\n1=Fire3\n\
                [OverlayTypes]\n1=Barrel\n[TerrainTypes]\n1=Elm\n\
                [Barrel]\nExplodes=yes\nChainReaction=no\n[Elm]\nStrength=80\nImmune=yes\n\
                [CombatDamage]\nMaxDamage=0\nMinDamage=1\nOccupyROFMultiplier=-1.5\nBunkerROFMultiplier=2\n\
                OccupyDamageMultiplier=0.5\nBunkerDamageMultiplier=0.25\nOpenToppedDamageMultiplier=1.5\n\
                C4Warhead=Blast\nCrushWarhead=Squash\nIonCannonWarhead=Ion\nBridgeStrength=900\nAmmoCrateDamage=20\n\
                SplashList=Drip\nBallisticScatter=1.5\nOccupyWeaponRange=6\nBunkerWeaponRangeBonus=1\n\
                OpenToppedRangeBonus=3\nFireSupress=0.75\nDeathWeapon=Pop\n\
                [General]\nVeteranROF=0.5\nVeteranCombat=1.5\nVeteranArmor=2\nVeteranSight=1.25\nVeteranRatio=-3\n\
                VeteranCap=2\nDamageFireTypes=F1,F2\nInfantryExplode=Splat\nMissileROTVar=0.5\n\
                MissileSafetyAltitude=400\nHarvesterUnit=hauler\nBarrelExplode=Kaboom\nBarrelDebris=Bit\n\
                BarrelParticle=Smoke\nMetallicDebris=M1,M2\nWeaponNullifyAnim=Null\nLightningWarhead=Zot\n\
                WeatherConBoltExplosion=Flash\nCrewEscape=50\nAlliedCrew=crewman\nSovietCrew=Ghost\n\
                AlliedSurvivorDivisor=100\nSovietSurvivorDivisor=50\nThirdSurvivorDivisor=0\nShipSinkingWeight=1\n\
                RepairRate=0.0001\nURepairRate=0.02\nIRepairRate=0.03\nRepairStep=0\nIRepairStep=2\n\
                RepairPercent=25%\nSelfHealInfantryFrames=0\nSelfHealUnitFrames=-5\nSelfHealInfantryAmount=-1\n\
                SelfHealUnitAmount=2\nThreatPerOccupant=7\nEnemyHouseThreatBonus=0.5\n\
                TargetSpecialThreatCoefficientDefault=3\nDisabledDisguiseDetectionPercent=40\n\
                [AudioVisual]\nGravity=0\n[WallModel]\nAlliedWallTransparency=yes\n\
                [ElevationModel]\nElevationIncrement=4\nElevationIncrementBonus=0.5\nElevationBonusCap=2\n\
                [IQ]\nRepairSell=2\n\
                [Easy]\nROF=1.5\nFirePower=0.5\nDestroyWalls=no\nArmor=1.0\n\
                [Normal]\nROF=-1\nArmor=0.5\n[Difficult]\nFirePower=2\n";
    let map = "[Difficult]\nDestroyWalls=yes\n";
    let (s, w) = sim(base, map, "");
    let g = &s.general;
    assert_eq!(g.max_damage, Some(0));
    assert_eq!(dec(&g.occupy_rof_multiplier), Some("-15e-1"));
    assert_eq!(dec(&g.open_topped_damage_multiplier), Some("15e-1"));
    assert_eq!(
        (g.c4_warhead.as_deref(), g.crush_warhead.as_deref()),
        (Some("Blast"), Some("Squash"))
    );
    assert_eq!(
        (g.bridge_strength, g.ammo_crate_damage),
        (Some(900), Some(20))
    );
    assert_eq!(texts(&g.splash_list), ["Drip"]);
    assert_eq!(dec(&g.ballistic_scatter), Some("15e-1"));
    assert_eq!(
        (
            g.occupy_weapon_range,
            g.bunker_weapon_range_bonus,
            g.open_topped_range_bonus
        ),
        (Some(6), Some(1), Some(3))
    );
    assert_eq!(dec(&g.fire_supress), Some("75e-2"));
    assert_eq!(g.death_weapon.as_deref(), Some("Pop"));
    assert_eq!(dec(&g.veteran_sight), Some("125e-2"));
    assert_eq!(texts(&g.damage_fire_types), ["F1", "F2"]);
    assert_eq!(g.harvester_unit, Some(vec!["Hauler".to_string()]));
    assert_eq!(texts(&g.metallic_debris), ["M1", "M2"]);
    assert_eq!(g.allied_crew.as_deref(), Some("Crewman"));
    // spec: S-0004 claim 13 ([General] crew fields create the infantry they name)
    assert_eq!(g.soviet_crew.as_deref(), Some("Ghost"));
    assert_eq!(
        (g.allied_survivor_divisor, g.repair_step, g.i_repair_step),
        (Some(100), Some(0), Some(2))
    );
    assert_eq!(dec(&g.repair_percent), Some("25%"));
    assert_eq!(
        (g.self_heal_unit_amount, g.threat_per_occupant),
        (Some(2), Some(7))
    );
    let defaults: Vec<&str> = g
        .coefficient_defaults
        .as_ref()
        .unwrap()
        .iter()
        .map(|d| d.0.as_str())
        .collect();
    assert_eq!(defaults, ["0", "0", "3", "0", "0"]);
    assert_eq!(g.disguise_detection_percent, Some(vec![40]));
    assert_eq!(
        (g.gravity, g.allied_wall_transparency),
        (Some(0), Some(true))
    );
    assert_eq!(g.elevation_increment, Some(4));
    assert_eq!(dec(&g.elevation_bonus_cap), Some("2"));
    assert_eq!(g.iq_repair_sell, Some(2));
    let easy = &s.difficulty["easy"];
    assert_eq!(
        (dec(&easy.rof), dec(&easy.firepower), easy.destroy_walls),
        (Some("15e-1"), Some("5e-1"), Some(false))
    );
    // The map's [Difficult] resets FirePower (S-0006 claim 5).
    let hard = &s.difficulty["difficult"];
    assert_eq!(
        (hard.firepower.as_ref(), hard.destroy_walls),
        (None, Some(true))
    );
    assert_eq!(
        (
            s.overlays["Barrel"].explodes,
            s.overlays["Barrel"].chain_reaction
        ),
        (Some(true), Some(false))
    );
    assert_eq!(
        (
            s.terrain_objects["Elm"].strength,
            s.terrain_objects["Elm"].immune
        ),
        (Some(80), Some(true))
    );
    for text in [
        "[CombatDamage] MinDamage 1: the original ignores",
        "[CombatDamage] MaxDamage 0 is below 1",
        "[CombatDamage] OccupyROFMultiplier -1.5 is negative",
        "[General] VeteranRatio -3 is negative",
        "DisabledDisguiseDetectionPercent \"40\" has no second item",
        "DamageFireTypes has fewer than three animations",
        "[AudioVisual] Gravity is 0",
        "[General] RepairStep 0 is below 1",
        "[General] SelfHealInfantryFrames 0 is not above 0",
        "[General] SelfHealUnitFrames -5 is not above 0",
        "[General] SelfHealInfantryAmount -1 is negative",
        "[General] RepairRate 0.0001 gives a building repair period below one tick",
        "[General] CrewEscape \"50\" has no `%`",
        "[Normal] Armor 0.5: the original ignores",
        "[Normal] ROF -1 is negative",
    ] {
        assert!(warned(&w, text), "{text}: {w:#?}");
    }
    assert!(!warned(&w, "[Easy] Armor"));
    assert!(!warned(&w, "ThirdCrew"));
    assert!(!warned(&w, "SplashList is empty"));
    assert!(!warned(&w, "named FIRE3"));
    // No animation type named FIRE3 with an exploding overlay: a warning (S-0312 Import mapping).
    let (_, w) = sim("[OverlayTypes]\n1=Barrel\n[Barrel]\nExplodes=yes\n", "", "");
    assert!(warned(&w, "no animation type is named FIRE3"));
    assert!(warned(&w, "SplashList is empty"));
}

/// spec: S-0003 claim 13 (the cross-check: a percent-list item's exact text reads back at 53 bits as the importer's double)
#[test]
fn verses_text_round_trips_through_num() {
    for item in [
        "12%", "0.1", "-7.25", "33%", "1e-320", "3.4e38", "abc", "0%",
    ] {
        let (vals, _) = value::percent_list(item.as_bytes(), 1, true, true);
        let text = combat::exact_text(&vals[0]);
        assert!(glasnost_num::decimal::parse_text(&text).is_ok(), "{text}");
        assert_eq!(nearest_double(&text).0, vals[0].bits, "{item} -> {text}");
    }
    // A decimal field of S-0003 claim 6 reads at 24 bits, so only singles round-trip there.
    let half = glasnost_rules::sim::Decimal("0.5".into());
    let Some(b) = (match value::decimal(b"0.5", DecimalField::Double) {
        value::Decimal::Value { value, .. } => Some(value.convert_rz(Format::Double)),
        value::Decimal::NoNumber => None,
    }) else {
        panic!()
    };
    assert_eq!(half.value(Rounding::TowardZero).0, b.bits);
}

/// spec: S-0301 to S-0315 Import mapping (coverage outcomes of the converted combat keys and of the keys the combat specs name but this converter does not convert)
#[test]
fn combat_coverage() {
    let inputs = Inputs {
        base: Some(file(
            "rules.ini",
            "[VehicleTypes]\n1=Cart\n[Warheads]\n1=Pow\n\
             [Cart]\nArmor=heavy\nWeapon3=Zap\nAmmo=2\nBurstDelay0=3\nWake=x\nPrimary=Gun\n\
             [Gun]\nROF=5\n[BuildingTypes]\n1=Hut\n[Hut]\nCost=5\n\
             [Pow]\nVerses=50%\nCellSpread=1\nShakeXlo=2\n\
             [CombatDamage]\nMaxDamage=500\nMinDamage=1\n[General]\nCrewEscape=50%\n\
             DumbMyEffectivenessCoefficient=1\n",
        )),
        art: Some(file(
            "art.ini",
            "[Cart]\nPrimaryFireFLH=1,2,3\nWeapon2FLH=1,1,1\nMuzzleFlash0=1,1\n",
        )),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let row = |cat: &str, key: &str| {
        import
            .coverage
            .iter()
            .find(|r| r.category == cat && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{cat} {key}"))
    };
    use crate::coverage::Outcome::{Field, Ignored, Unsupported};
    assert_eq!(row("vehicles", "Armor"), (Field, "S-0305".into()));
    assert_eq!(row("vehicles", "Weapon3"), (Field, "S-0308".into()));
    assert_eq!(row("vehicles", "Ammo"), (Field, "S-0301".into()));
    assert_eq!(row("vehicles", "BurstDelay0"), (Field, "S-0301".into()));
    assert_eq!(row("vehicles", "Wake"), (Unsupported, "S-0312".into()));
    assert_eq!(row("warheads", "Verses").0, Field);
    assert_eq!(row("warheads", "CellSpread").0, Field);
    assert_eq!(row("warheads", "ShakeXlo"), (Unsupported, "S-0303".into()));
    assert_eq!(
        row("global:CombatDamage", "MaxDamage"),
        (Field, "S-0302".into())
    );
    assert_eq!(
        row("global:CombatDamage", "MinDamage"),
        (Ignored, "S-0302".into())
    );
    assert_eq!(
        row("global:General", "CrewEscape"),
        (Field, "S-0312".into())
    );
    assert_eq!(
        row("global:General", "DumbMyEffectivenessCoefficient").0,
        Ignored
    );
    assert_eq!(row("art", "PrimaryFireFLH"), (Field, "S-0313".into()));
    assert_eq!(row("art", "Weapon2FLH"), (Field, "S-0313".into()));
    assert_eq!(row("art", "MuzzleFlash0"), (Unsupported, "S-0313".into()));
    // spec: S-0311 F1, S-0314 claim 5 (the building cost field)
    assert_eq!(row("buildings", "Cost"), (Field, "S-0311, S-0314".into()));
    // spec: S-0004 claim 13 (a weapon created by a vehicle's primary field: its section is a weapon's)
    assert_eq!(row("weapons", "ROF"), (Field, "S-0301".into()));
}

/// spec: S-0004 claim 11 and S-0302 Import mapping and claim 5 (the sidebar tab and the defence factor share the category; BuildCat: the first 31 bytes, trimmed, a whole word ignoring A–Z case; only Combat makes a defence building; an unknown nonempty word is not one, not the held value; missing or empty keeps the held category)
#[test]
fn build_category_word() {
    let defence = |base: &str, mode: &str| {
        let text = format!("[BuildingTypes]\n1=Hut\n[Hut]\n{base}");
        let (s, _, _) = import(&text, mode, "", "", &ImportOptions::default());
        s.buildings["Hut"].defense
    };
    assert_eq!(defence("BuildCat=  cOmBaT  \n", ""), Some(true));
    assert_eq!(defence("BuildCat=Power\n", ""), Some(false));
    assert_eq!(defence("BuildCat=Combatx\n", ""), Some(false));
    // An unknown word after Combat gives DontCare, not the held Combat.
    assert_eq!(
        defence("BuildCat=Combat\n", "[Hut]\nBuildCat=Bunker\n"),
        Some(false)
    );
    // An empty value keeps the held category.
    assert_eq!(
        defence("BuildCat=Combat\n", "[Hut]\nBuildCat=\n"),
        Some(true)
    );
    // Never given: not a defence.
    assert_ne!(defence("Strength=1\n", ""), Some(true));
    // Only the first 31 bytes count: text after them is not seen.
    let padded = format!("BuildCat=Combat{}x\n", " ".repeat(25));
    assert_eq!(defence(&padded, ""), Some(true));
    let padded = format!("BuildCat=Combat{}x\n", " ".repeat(24));
    assert_eq!(defence(&padded, ""), Some(false));
}

/// spec: S-0303 claims 25 and 26 and S-0312 claim 28 (no-type items are left out of the animation and debris lists and hold no place; a value of only no-type items empties the list; an empty value keeps it; the maxima are read independently of the types)
#[test]
fn no_type_list_items_and_maxima() {
    let base = "[Warheads]\n1=Pow\n2=Bam\n\
                [Pow]\nAnimList=none,Alpha,<NONE>,Beta\nDebrisTypes=<none>,Chunk\nDebrisMaximums=2,abc,3\n\
                [Bam]\nAnimList=Gamma\n";
    let mode = "[Bam]\nAnimList=\n";
    let (s, _, _) = import(base, mode, "", "", &ImportOptions::default());
    let pow = &s.warheads["Pow"];
    assert_eq!(
        pow.anim_list.as_deref(),
        Some(&["Alpha".to_owned(), "Beta".to_owned()][..])
    );
    assert_eq!(pow.debris_types.as_deref(), Some(&["Chunk".to_owned()][..]));
    assert_eq!(pow.debris_maximums.as_deref(), Some(&[2, 0, 3][..]));
    // An empty value keeps the list in force.
    assert_eq!(
        s.warheads["Bam"].anim_list.as_deref(),
        Some(&["Gamma".to_owned()][..])
    );
    // Only no-type items: an empty list.
    let (s, _, _) = import(
        base,
        "[Bam]\nAnimList=none,<none>\n",
        "",
        "",
        &ImportOptions::default(),
    );
    assert_eq!(s.warheads["Bam"].anim_list.as_deref(), Some(&[][..]));
}

/// spec: S-0007 claim 33, S-0004 claim 14, S-0306 Import mapping (a projectile's trailer and trailer delay come from the art section of each read's non-empty rules image; rules keys have no effect; absent or empty keeps, a no-type name clears the trailer)
#[test]
fn projectile_trailer_settings_from_art() {
    let art = "[Pa]\nTrailer=Puff\nSpawnDelay=6\n[Pb]\nTrailer=\n[Pc]\nTrailer=none\nSpawnDelay=2\n\
               [Dart]\nTrailer=Other\nSpawnDelay=9\n";
    let get = |layers: &[&str]| {
        let (s, _) = direct(layers, art, &[], &["Dart"], IniQuirks::default());
        let p = &s.projectiles["Dart"];
        (p.trailer.clone(), p.spawn_delay)
    };
    let base = "[Warheads]\n1=Dart\n";
    // The rules keys have no effect, and there is no fallback to the type name.
    assert_eq!(
        get(&[&format!("{base}[Dart]\nTrailer=Rt\nSpawnDelay=4\n")]),
        (None, None)
    );
    let a = format!("{base}[Dart]\nImage=Pa\n");
    assert_eq!(get(&[&a]), (Some("Puff".into()), Some(6)));
    // A later read without an image reads neither setting: both are kept.
    assert_eq!(
        get(&[&a, "[Dart]\nImage=\n"]),
        (Some("Puff".into()), Some(6))
    );
    // An empty trailer and a missing delay keep the held values.
    assert_eq!(
        get(&[&a, "[Dart]\nImage=Pb\n"]),
        (Some("Puff".into()), Some(6))
    );
    // A no-type name clears the trailer.
    assert_eq!(get(&[&a, "[Dart]\nImage=Pc\n"]), (None, Some(2)));
}

/// spec: S-0003 claim 12 and Edge cases, S-0312 claim 28 (quirks.list_items_untrimmed: on, the importer warns for a name item with whitespace at either end; off, items are trimmed, a whitespace-only item is skipped, and there is no warning)
#[test]
fn list_items_untrimmed_switch() {
    let base = "[Warheads]\n1=Pow\n[Pow]\nAnimList=Alpha , ,Beta\n";
    let (_, w) = direct(&[base], "", &[], &[], IniQuirks::default());
    assert!(warned(&w, "quirks.list_items_untrimmed"));
    let off = IniQuirks {
        list_items_untrimmed: false,
        ..IniQuirks::default()
    };
    let (s, w) = direct(&[base], "", &[], &[], off);
    assert_eq!(
        s.warheads["Pow"].anim_list.as_deref(),
        Some(&["Alpha".to_owned(), "Beta".to_owned()][..])
    );
    assert!(!warned(&w, "quirks.list_items_untrimmed"));
}

/// spec: S-0311 Import mapping and Edge cases (revision 2: quirks.list_items_untrimmed on, `firepower ` names no ability and both warnings are given; off, it is trimmed and matches; an unknown non-empty item warns in either setting)
#[test]
fn ability_lists_follow_list_items_untrimmed() {
    let base = "[InfantryTypes]\n1=Walker\n[Walker]\nVeteranAbilities=firepower ,sight,bogus\n";
    let (s, w) = direct(&[base], "", &[], &[], IniQuirks::default());
    assert_eq!(
        s.infantry["Walker"].veteran_abilities,
        ["FIREPOWER ", "SIGHT", "BOGUS"]
    );
    assert!(warned(&w, "VeteranAbilities: a list item with whitespace"));
    assert!(warned(&w, "\"firepower \" names no ability"));
    assert!(warned(&w, "\"bogus\" names no ability"));
    let off = IniQuirks {
        list_items_untrimmed: false,
        ..IniQuirks::default()
    };
    let (s, w) = direct(&[base], "", &[], &[], off);
    assert_eq!(
        s.infantry["Walker"].veteran_abilities,
        ["FIREPOWER", "SIGHT", "BOGUS"]
    );
    assert!(!warned(&w, "a list item with whitespace"));
    assert!(!warned(&w, "\"firepower\" names no ability"));
    assert!(warned(&w, "\"bogus\" names no ability"));
}

/// spec: S-0003 claim 19 and Import mapping row C18, S-0312 Import mapping (DestroyParticleSystems: a particle-system name list; `none` and `<none>` take no place)
#[test]
fn destroy_particle_systems_omit_no_type_items() {
    let base = "[VehicleTypes]\n1=Cart\n[ParticleSystems]\n1=Smog\n\
                [Cart]\nDestroyParticleSystems=none,Smog,<NONE>,Glow,Smog\n";
    let (s, _, _) = import(base, "", "", "", &ImportOptions::default());
    assert_eq!(
        s.vehicles["Cart"]
            .combat
            .destroy_particle_systems
            .as_deref(),
        Some(&["Smog".to_owned(), "Glow".to_owned(), "Smog".to_owned()][..])
    );
}

/// spec: S-0506 Import mapping (the spawn keys of an object type, the three rocket parameter sets of `[General]`, their warheads in the combat-damage section), S-0004 claim 14 (an aircraft's art trailer)
#[test]
fn missile_fields() {
    let base = "[VehicleTypes]\n1=Lobber\n[AircraftTypes]\n1=Dart\n2=Glider\n[Animations]\n1=Puff\n\
                [Lobber]\nSpawns=dart\nSpawnsNumber=2\nSpawnRegenRate=150\nSpawnReloadRate=40\nMissileSpawn=yes\n\
                [Dart]\nSpawned=yes\nMissileSpawn=yes\nImage=DARTART\n[Glider]\nSpawned=yes\n\
                [General]\nV3RocketType=dart\nV3RocketPauseFrames=3\nV3RocketTiltFrames=12\n\
                V3RocketPitchInitial=0.25\nV3RocketPitchFinal=0.75\nV3RocketTurnRate=0.125\n\
                V3RocketRaiseRate=2.5\nV3RocketAcceleration=0.5\nV3RocketAltitude=300\n\
                V3RocketDamage=77\nV3RocketEliteDamage=99\nV3RocketBodyLength=64\nV3RocketLazyCurve=no\n\
                DMislType=Glider\nDMislLazyCurve=yes\nCMislType=none\nCMislDamage=5\n\
                [CombatDamage]\nV3Warhead=Boom\nV3EliteWarhead=BigBoom\nDMislWarhead=Pow\nCMislEliteWarhead=Zap\n";
    let art = "[DARTART]\nTrailer=Puff\n[Glider]\nTrailer=none\n";
    let (s, _) = sim(base, "", art);
    let l = &s.vehicles["Lobber"];
    assert_eq!(l.combat.spawns.as_deref(), Some("Dart"));
    assert_eq!(
        (
            l.combat.spawns_number,
            l.combat.spawn_regen_rate,
            l.combat.spawn_reload_rate
        ),
        (Some(2), Some(150), Some(40))
    );
    assert_eq!(l.missile_spawn, Some(true));
    let d = &s.aircraft["Dart"];
    assert_eq!((d.spawned, d.missile_spawn), (Some(true), Some(true)));
    assert_eq!(d.trailer.as_deref(), Some("Puff"));
    assert_eq!(s.aircraft["Glider"].trailer, None);
    let g = &s.general;
    assert_eq!(g.v3_rocket_type.as_deref(), Some("Dart"));
    assert_eq!(
        (g.v3_rocket_pause_frames, g.v3_rocket_tilt_frames),
        (Some(3), Some(12))
    );
    assert_eq!(dec(&g.v3_rocket_pitch_initial), Some("25e-2"));
    assert_eq!(dec(&g.v3_rocket_pitch_final), Some("75e-2"));
    assert_eq!(dec(&g.v3_rocket_turn_rate), Some("125e-3"));
    assert_eq!(dec(&g.v3_rocket_raise_rate), Some("25e-1"));
    assert_eq!(dec(&g.v3_rocket_acceleration), Some("5e-1"));
    assert_eq!(
        (
            g.v3_rocket_altitude,
            g.v3_rocket_damage,
            g.v3_rocket_elite_damage,
            g.v3_rocket_body_length,
            g.v3_rocket_lazy_curve
        ),
        (Some(300), Some(77), Some(99), Some(64), Some(false))
    );
    assert_eq!(g.d_misl_type.as_deref(), Some("Glider"));
    assert_eq!(g.d_misl_lazy_curve, Some(true));
    // `none` names no type; an absent key gives no value (the engine applies the defaults).
    assert_eq!(g.c_misl_type, None);
    assert_eq!(g.c_misl_damage, Some(5));
    assert_eq!(g.d_misl_pause_frames, None);
    assert_eq!(
        (g.v3_warhead.as_deref(), g.v3_elite_warhead.as_deref()),
        (Some("Boom"), Some("BigBoom"))
    );
    assert_eq!(g.d_misl_warhead.as_deref(), Some("Pow"));
    assert_eq!(g.d_misl_elite_warhead, None);
    assert_eq!(g.c_misl_elite_warhead.as_deref(), Some("Zap"));
}
