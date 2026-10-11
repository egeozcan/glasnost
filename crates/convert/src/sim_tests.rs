//! Tests of the `sim` section on invented data (every name and number here
//! is made up).

use std::collections::BTreeMap;

use glasnost_ini::binfloat::Format;
use glasnost_ini::value::{self, Decimal as IniDecimal, DecimalField};
use glasnost_num::Rounding;
use glasnost_num::decimal::{self as num_decimal, Field};
use glasnost_rules::sim::{LandType, LocomotionClass, MovementZone, SimData};

use super::*;
use crate::inputs::{InputFile, Inputs};
use crate::layers::Warning;
use crate::{ImportOptions, convert};

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

/// Converts base rules, map and art and resolves the tree; returns the
/// typed `sim` data and the import's warnings.
fn sim_with(base: &str, map: &str, art: &str, options: &ImportOptions) -> (SimData, Vec<Warning>) {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        map: Some(file("test.map", map)),
        art: Some(file("art.ini", art)),
        ..Inputs::default()
    };
    let import = convert(&inputs, options);
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset.sim, import.warnings)
}

fn sim(base: &str, map: &str, art: &str) -> (SimData, Vec<Warning>) {
    sim_with(base, map, art, &ImportOptions::default())
}

fn dec(d: &Option<glasnost_rules::sim::Decimal>) -> Option<&str> {
    d.as_ref().map(|d| d.0.as_str())
}

fn warned(w: &[Warning], text: &str) -> bool {
    w.iter().any(|w| w.message.contains(text))
}

/// The bits S-0003 gives a stored decimal, as a double.
fn ini_bits(stored: &str, field: DecimalField) -> Option<u64> {
    match value::decimal(stored.as_bytes(), field) {
        IniDecimal::Value { value, .. } => Some(value.convert_rz(Format::Double).bits),
        IniDecimal::NoNumber => None,
    }
}

/// spec: S-0003 claims 6 and 7 (our decimal text reads as the stored text does)
#[test]
fn decimal_text_reproduces_the_stored_value() {
    let cases = [
        "0.1",
        ".5",
        "5.",
        "-0",
        "1e2",
        "1e",
        "16777217",
        "3.4028235e38",
        "1e39",
        "1e-45",
        "1e-50",
        "50%",
        "50 %",
        "100%%",
        "0.5%",
        "-100%",
        "1,5%",
        "  +2.25x",
        "-7.125e-3",
        "0.0000001",
        "123456789012345678901234567890",
        "1.5E+3",
        "10%",
    ];
    for stored in cases {
        let ours = decimal_text(stored.as_bytes()).unwrap_or_else(|| panic!("{stored}"));
        for (ini_field, num_field) in [
            (DecimalField::Double, Field::Decimal),
            (DecimalField::Single, Field::Single),
        ] {
            let want = ini_bits(stored, ini_field).unwrap();
            let got = num_decimal::parse(&ours, num_field, Rounding::TowardZero)
                .unwrap_or_else(|e| panic!("{stored} -> {ours}: {e}"));
            assert_eq!(got.0, want, "{stored:?} -> {ours:?} ({num_field:?})");
        }
    }
    for none in ["abc", "%5", ".", "-"] {
        assert_eq!(decimal_text(none.as_bytes()), None, "{none}");
    }
}

/// spec: S-0209 Import mapping, S-0210 F2, S-0215 Import mapping (−1 keeps the held value); S-0003 claim 18; S-0211 Edge cases
#[test]
fn game_wide_fields_and_not_given_values() {
    let base = "[General]\nCloseEnough=2.25\nGuardModeStray=1.5\nCliffBackImpassability=258\n\
                NormalTargetingDelay=$1F\nEngineerCaptureLevel=40%\n\
                [CombatDamage]\nCrush=1.75\nPlayerScatter=yes\nPlayerReturnFire=maybe\n\
                [AI]\nBlockagePathDelay=12\nPathDelay=0.5\n[IQ]\nScatter=2\nMaxIQLevels=4\n\
                [AudioVisual]\nConditionYellow=50\nConditionRed=25%\nIdleActionFrequency=0.25\n\
                [MultiplayerDialogSettings]\nGameSpeed=3\n";
    let map = "[General]\nCloseEnough=-1.0\nGuardModeStray=-100%\n[CombatDamage]\nCrush=-1\n";
    let (s, w) = sim(base, map, "");
    let g = &s.general;
    assert_eq!(dec(&g.close_enough), Some("225e-2"));
    assert_eq!(dec(&g.guard_mode_stray), Some("15e-1"));
    assert_eq!(dec(&g.crush_range), Some("175e-2"));
    assert_eq!(g.cliff_back_impassability, Some(258));
    assert_eq!(g.normal_targeting_delay, Some(31));
    assert_eq!(dec(&g.engineer_capture_level), Some("40%"));
    assert_eq!(g.player_scatter, Some(true));
    // spec: S-0003 claim 2 (an unrecognised boolean keeps the value in force: here the default)
    assert_eq!(g.player_return_fire, None);
    assert_eq!(g.blockage_path_delay, Some(12));
    assert_eq!(dec(&g.path_delay), Some("5e-1"));
    assert_eq!((g.iq_scatter, g.iq_max), (Some(2), Some(4)));
    assert_eq!(dec(&g.condition_yellow), Some("50"));
    assert_eq!(dec(&g.condition_red), Some("25%"));
    assert_eq!(dec(&g.idle_action_frequency), Some("25e-2"));
    assert_eq!(g.lobby_game_speed, Some(3));
    assert_eq!(g.guard_area_targeting_delay, None);
    assert!(warned(&w, "CliffBackImpassability 258"));
    assert!(warned(&w, "ConditionYellow \"50\" has no `%`"));
    assert!(!warned(&w, "ConditionRed"));
    // Another number replaces the held value.
    let (s, _) = sim(base, "[General]\nCloseEnough=-1.5\n", "");
    assert_eq!(dec(&s.general.close_enough), Some("-15e-1"));
    // spec: S-0003 claim 8 (no number: the value in force is kept, with a warning)
    let (s, w) = sim(base, "[AI]\nPathDelay=soon\n", "");
    assert_eq!(dec(&s.general.path_delay), Some("5e-1"));
    assert!(warned(&w, "PathDelay \"soon\" has no number"));
}

/// spec: S-0207 claim 1, S-0208 Import mapping (Speed −1 keeps; warnings), S-0215 Import mapping (GuardRange −1)
#[test]
fn speed_and_guard_range() {
    let base = "[VehicleTypes]\n1=Cart\n[InfantryTypes]\n1=Walker\n[AircraftTypes]\n1=Kite\n\
                [Cart]\nSpeed=7\nGuardRange=3.5\n[Walker]\nSpeed=140\n[Kite]\nSpeed=9\n";
    let map = "[Cart]\nSpeed=$FFFFFFFF\nGuardRange=-1\n[Walker]\nSpeed=-1\n[Kite]\nSpeed=-1\n";
    let (s, w) = sim(base, map, "");
    assert_eq!(s.vehicles["Cart"].speed, Some(7));
    assert_eq!(dec(&s.vehicles["Cart"].guard_range), Some("35e-1"));
    assert_eq!(s.infantry["Walker"].speed, Some(140));
    // No spec gives aircraft a "not given" speed: −1 is a value.
    assert_eq!(s.aircraft["Kite"].speed, Some(-1));
    assert!(warned(&w, "Walker Speed 140"));
    assert!(!warned(&w, "Speed -1"));
    let (_, w) = sim(base, "[Cart]\nSpeed=-2\n", "");
    assert!(warned(&w, "Cart Speed -2"));
}

/// spec: S-0211 claim 2 (whole rows per layer, start-up row warning, negative multipliers)
#[test]
fn land_type_rows() {
    let base = "[Clear]\nFoot=50%\nTrack=0.75\nBuildable=yes\n[Road]\nWheel=1.5\n\
                [Water]\nFloat=1\n[Rock]\nFoot=0\n[Wall]\nX=1\n[Tiberium]\nX=1\n[Beach]\nX=1\n\
                [Rough]\nX=1\n[Ice]\nX=1\n[Railroad]\nX=1\n[Tunnel]\nX=1\n";
    let map = "[Road]\nHover=-0.5\n[Weeds]\nBuildable=yes\n";
    let (s, w) = sim(base, map, "");
    let clear = &s.land_types[&LandType::Clear];
    assert_eq!(dec(&clear.foot), Some("50%"));
    assert_eq!(dec(&clear.track), Some("75e-2"));
    assert_eq!(clear.buildable, Some(true));
    assert_eq!(clear.wheel, None);
    // The map's [Road] resets the base's Wheel (claim 2: the layer sets the whole row).
    let road = &s.land_types[&LandType::Road];
    assert_eq!(dec(&road.hover), Some("-5e-1"));
    assert_eq!(road.wheel, None);
    assert!(warned(&w, "[Road] Hover is negative"));
    // A section with only unread keys still sets the row.
    assert_eq!(s.land_types[&LandType::Wall], Default::default());
    // Weeds: given by the map only, and the base lacks it.
    assert_eq!(s.land_types[&LandType::Weeds].buildable, Some(true));
    assert!(warned(&w, "lack the land-type section [Weeds]"));
    assert!(!warned(&w, "lack the land-type section [Clear]"));
    // A land type no layer gives has no row (the start-up row).
    let (s, _) = sim("[Clear]\nFoot=1\n", "", "");
    assert!(!s.land_types.contains_key(&LandType::Water));
}

/// spec: S-0215 Import mapping (behaviour sections and keys), claim 21 (`quirks.anti_air_rate_layer_reset`)
#[test]
fn behaviours_and_anti_air_rate() {
    let base = "[Area Guard]\nRate=0.5\nAARate=0.25\nParalyzed=yes\n\
                [QMove]\nNoThreat=true\nScatter=no\n[Spyplane Approach]\nZombie=1\n\
                [Guard]\nRate=0.5\nAARate=0.125\n[Sticky]\nAARate=0.75\n";
    let map = "[Area Guard]\nRetaliate=no\n[Guard]\nRate=2\nAARate=0\n[Sticky]\nRecruitable=no\n";
    let (s, _) = sim(base, map, "");
    let b = &s.behaviours;
    let area = &b["area_guard"];
    assert_eq!(area.paralysed, Some(true));
    assert_eq!(area.retaliate, Some(false));
    assert_eq!(dec(&area.rate), Some("5e-1"));
    // The map's section gives no anti-air rate: it becomes the rate after that layer.
    assert_eq!(dec(&area.aa_rate), Some("5e-1"));
    assert_eq!(b["queued_move"].no_threat, Some(true));
    assert_eq!(b["queued_move"].scatter, Some(false));
    assert_eq!(b["spy_plane_approach"].zombie, Some(true));
    // An explicit 0 becomes the rate.
    assert_eq!(dec(&b["guard"].aa_rate), Some("2"));
    // No rate given: absent, which the simulation reads as the rate.
    assert_eq!(b["sticky"].aa_rate, None);
    assert!(!b.contains_key("move"));
    // Switch off: an absent anti-air rate keeps the held value.
    let options = ImportOptions {
        quirks: LayerQuirks {
            anti_air_rate_layer_reset: false,
            ..LayerQuirks::default()
        },
        ..ImportOptions::default()
    };
    let (s, _) = sim_with(base, map, "", &options);
    assert_eq!(dec(&s.behaviours["area_guard"].aa_rate), Some("25e-2"));
    assert_eq!(dec(&s.behaviours["sticky"].aa_rate), Some("75e-2"));
    assert_eq!(s.behaviours["guard"].aa_rate, None);
}

/// spec: S-0207 Import mapping, S-0208 Import mapping (country fields)
#[test]
fn countries() {
    let base = "[Countries]\n1=Ruritania\n[Ruritania]\nSpeedUnitsMult=1.25\nSpeedInfantryMult=90%\nMultiplayPassive=yes\n";
    let (s, _) = sim(base, "", "");
    let c = &s.countries["Ruritania"];
    assert_eq!(dec(&c.speed_units_mult), Some("125e-2"));
    assert_eq!(dec(&c.speed_infantry_mult), Some("90%"));
    assert_eq!(c.multiplay_passive, Some(true));
}

/// spec: S-0211 claim 3 (the locomotion class table and its departure)
#[test]
fn locomotion_classes() {
    let base = "[VehicleTypes]\n1=Roller\n2=Rover\n3=Skiff\n4=Odd\n[InfantryTypes]\n1=Walker\n\
                [AircraftTypes]\n1=Kite\n\
                [Roller]\nCrusher=yes\n[Rover]\nX=1\n[Skiff]\nSpeedType=floatbeach\n\
                [Odd]\nSpeedType=Wheel\n[Walker]\nX=1\n";
    // The map sets Rover's crusher flag: its class stays wheel. Odd's new value names no class.
    let map = "[Rover]\nCrusher=yes\n[Odd]\nSpeedType=Tracked\n";
    let (s, w) = sim(base, map, "");
    assert_eq!(
        s.vehicles["Roller"].locomotion_class,
        Some(LocomotionClass::Track)
    );
    assert_eq!(
        s.vehicles["Rover"].locomotion_class,
        Some(LocomotionClass::Wheel)
    );
    assert_eq!(
        s.vehicles["Skiff"].locomotion_class,
        Some(LocomotionClass::FloatBeach)
    );
    assert_eq!(
        s.vehicles["Odd"].locomotion_class,
        Some(LocomotionClass::Wheel)
    );
    assert!(warned(
        &w,
        "SpeedType \"Tracked\" names no locomotion class"
    ));
    assert_eq!(
        s.infantry["Walker"].locomotion_class,
        Some(LocomotionClass::Foot)
    );
    assert_eq!(
        s.aircraft["Kite"].locomotion_class,
        Some(LocomotionClass::Winged)
    );
    // A vehicle type whose first reading names no class gets the default.
    let (s, _) = sim("[VehicleTypes]\n1=Odd\n[Odd]\nSpeedType=Hoover\n", "", "");
    assert_eq!(
        s.vehicles["Odd"].locomotion_class,
        Some(LocomotionClass::Wheel)
    );
}

/// spec: S-0207 claim 14 and table T3 (class identifiers: case, blanks, ignored values, unknown identifiers, layering)
#[test]
fn locomotor_field() {
    let drive = "{4A582741-9839-11D1-B709-00A024DDAFD1}";
    let walk = "{4A582744-9839-11D1-B709-00A024DDAFD1}";
    let ship = "{2BEA74E1-7CCA-11D3-BE14-00104B62A16C}";
    let unknown = "{00000000-0000-0000-0000-000000000001}";
    let base = format!(
        "[VehicleTypes]\n1=A\n2=B\n3=C\n4=S\n5=U\n6=Boat\n[InfantryTypes]\n1=I\n2=J\n\
         [AircraftTypes]\n1=P\n\
         [A]\nX=1\n[B]\nLocomotor={}\n[C]\nLocomotor=drive\n[S]\nLocomotor={drive}\n\
         [U]\nLocomotor={drive}\n[Boat]\nLocomotor={ship}\n[I]\nX=1\n[J]\nLocomotor={walk}\n[P]\nX=1\n",
        drive.to_ascii_lowercase()
    );
    // A later layer: an ignored value keeps S's locomotor, an unknown one keeps U's.
    let map =
        format!("[S]\nLocomotor=drive\n[U]\nLocomotor={unknown}\n[Boat]\nLocomotor=  {drive}  \n");
    let (s, w) = sim(&base, &map, "");
    use glasnost_rules::sim::Locomotor as L;
    assert_eq!(
        s.vehicles["A"].locomotor, None,
        "absent: no field, the teleporting default"
    );
    assert_eq!(
        s.vehicles["B"].locomotor,
        Some(L::Drive),
        "lower-case digits"
    );
    assert_eq!(s.vehicles["C"].locomotor, None, "a name is ignored");
    assert_eq!(
        s.vehicles["S"].locomotor,
        Some(L::Drive),
        "ignored value keeps the layer's locomotor"
    );
    assert_eq!(
        s.vehicles["U"].locomotor,
        Some(L::Drive),
        "unknown identifier: kept (departure)"
    );
    assert_eq!(
        s.vehicles["Boat"].locomotor,
        Some(L::Drive),
        "blanks around the value"
    );
    assert_eq!(s.infantry["I"].locomotor, None);
    assert_eq!(s.infantry["J"].locomotor, Some(L::Walk));
    assert_eq!(s.aircraft["P"].locomotor, None);
    assert!(warned(
        &w,
        "C Locomotor \"drive\" is not a class identifier"
    ));
    assert!(warned(
        &w,
        "S Locomotor \"drive\" is not a class identifier"
    ));
    assert!(warned(&w, "U Locomotor"));
    assert!(warned(&w, "names no known locomotor"));
    for bad in [
        "{4A582741983911D1B70900A024DDAFD1}",
        "4A582741-9839-11D1-B709-00A024DDAFD1",
        "{4A582741-9839-11D1-B709-00A024DDAFD}",
        "{4A58274G-9839-11D1-B709-00A024DDAFD1}",
        "",
    ] {
        assert_eq!(class_identifier(bad.as_bytes()), None, "{bad}");
    }
    assert_eq!(
        class_identifier(format!("{drive}x").as_bytes()),
        None,
        "characters after the brace"
    );
}

/// spec: S-0210 claim 13 and Import mapping (DamageLevels from the art data's image section, Strength from the rules section), claim 14 (the wall-owner key)
#[test]
fn wall_data_and_wall_owner_flag() {
    let base = "[OverlayTypes]\n0=Fence\n1=Pale\n2=Bare\n[Countries]\n0=Land\n1=Isle\n\
                [Fence]\nWall=yes\nStrength=700\nImage=FENCEART\n\
                [Pale]\nWall=yes\nDamageLevels=3\n[Bare]\nWall=yes\n\
                [Land]\nWallOwner=no\n[Isle]\nX=1\n";
    let art = "[FENCEART]\nDamageLevels=4\n[Fence]\nDamageLevels=9\n[Pale]\nX=1\n";
    let (s, _) = sim(base, "", art);
    assert_eq!(
        s.overlays["Fence"].damage_levels,
        Some(4),
        "the image section's key"
    );
    assert_eq!(s.overlays["Fence"].strength, Some(700));
    assert_eq!(
        s.overlays["Pale"].damage_levels, None,
        "the rules section's key is not read"
    );
    assert_eq!(s.overlays["Bare"].damage_levels, None);
    assert_eq!(s.overlays["Bare"].strength, None);
    assert_eq!(s.countries["Land"].wall_owner, Some(false));
    assert!(
        s.countries
            .get("Isle")
            .is_none_or(|c| c.wall_owner.is_none())
    );
}

/// spec: S-0203 Import mapping (SpawnsTiberium, a terrain type's spawns-resources flag)
#[test]
fn terrain_spawns_resources_flag() {
    let base = "[TerrainTypes]\n0=Tree\n1=Grower\n[Tree]\nSpawnsTiberium=no\n[Grower]\nSpawnsTiberium=yes\n";
    let (s, _) = sim(base, "", "");
    assert_eq!(s.terrain_objects["Tree"].spawns_tiberium, Some(false));
    assert_eq!(s.terrain_objects["Grower"].spawns_tiberium, Some(true));
}

/// spec: S-0211 claim 4 and Import mapping (zone names, restricted land types)
#[test]
fn zones_and_restrictions() {
    let base = "[VehicleTypes]\n1=Mole\n2=Barge\n3=Boat\n4=Tram\n\
                [Mole]\nMovementZone=subterannean\n[Barge]\nMovementZone=AMPHIBIOUSDESTROYER\n\
                MovementRestrictedTo=water\n[Boat]\nMovementZone=WaterBeach\nMovementRestrictedTo=Sea\n\
                [Tram]\nMovementZone=Crusher\nMovementRestrictedTo=Railroad\n";
    let map = "[Tram]\nMovementZone=Subterranean\nMovementRestrictedTo=<none>\n";
    let (s, w) = sim(base, map, "");
    let v = &s.vehicles;
    assert_eq!(v["Mole"].movement_zone, Some(MovementZone::Subterranean));
    assert_eq!(
        v["Barge"].movement_zone,
        Some(MovementZone::AmphibiousDestroyer)
    );
    assert_eq!(v["Barge"].movement_restricted_to, Some(LandType::Water));
    assert_eq!(v["Boat"].movement_zone, Some(MovementZone::WaterBeach));
    assert_eq!(v["Boat"].movement_restricted_to, None);
    assert!(warned(&w, "MovementRestrictedTo \"Sea\""));
    // The two-r spelling is not a zone: the held zone stays.
    assert_eq!(v["Tram"].movement_zone, Some(MovementZone::Crusher));
    assert!(warned(
        &w,
        "MovementZone \"Subterranean\" is not a movement zone"
    ));
    assert_eq!(v["Tram"].movement_restricted_to, None);
    assert!(!warned(&w, "<none>"));
}

/// spec: S-0211 claims 4 and 5 (overlay land types; ore and gems get tiberium)
#[test]
fn overlay_land_types() {
    let base = "[OverlayTypes]\n1=Ore\n2=Gem\n3=Fence\n4=Pebble\n5=Mud\n\
                [Ore]\nTiberium=yes\n[Gem]\nTiberium=yes\nLand=<none>\n\
                [Fence]\nLand=Wall\nWall=yes\nIsARock=no\n[Pebble]\nLand=Gravel\n[Mud]\nLand=rough\n";
    let (s, w) = sim(base, "", "");
    let o = &s.overlays;
    assert_eq!(o["Ore"].land, Some(LandType::Tiberium));
    assert_eq!(o["Gem"].land, Some(LandType::Tiberium));
    assert_eq!(o["Fence"].land, Some(LandType::Wall));
    assert_eq!(o["Fence"].wall, Some(true));
    assert_eq!(o["Fence"].rock, Some(false));
    assert_eq!(o["Pebble"].land, Some(LandType::Clear));
    assert_eq!(o["Mud"].land, Some(LandType::Rough));
    assert!(warned(&w, "Gem Land \"<none>\" gives no land type"));
    assert!(warned(&w, "Pebble Land \"Gravel\""));
    // Without the art lookup, has_art stays absent.
    assert_eq!(o["Ore"].has_art, None);
}

/// spec: S-0009 claim 12 (has_art from the art lookup)
#[test]
fn overlay_art_flag() {
    let base = "[OverlayTypes]\n1=Ore\n2=Ghost\n";
    let layers = [crate::layers::Layer {
        role: crate::layers::Role::Base,
        file: "rules.ini".into(),
        doc: Document::parse(base.as_bytes(), Default::default(), &mut Vec::new()),
    }];
    let art = Document::empty(Default::default());
    let outcome = crate::layers::apply(
        &layers,
        &art,
        "art.ini",
        Default::default(),
        &LayerQuirks::default(),
    );
    let arts = BTreeMap::from([
        (
            (Kind::Overlays, 0),
            TypeArt {
                sprite: Some(b"ore.tem".to_vec()),
                ..TypeArt::default()
            },
        ),
        ((Kind::Overlays, 1), TypeArt::default()),
    ]);
    let (node, _) = build(SimInputs {
        outcome: &outcome,
        art: &art,
        arts: Some(&arts),
        quirks: &LayerQuirks::default(),
        art_quirks: &ArtQuirks::default(),
        ini_quirks: &Default::default(),
        sim_quirks: &SimQuirks::default(),
    });
    let parsed: SimData = glasnost_rules::de::from_node(&node).unwrap();
    assert_eq!(parsed.overlays["Ore"].has_art, Some(true));
    assert_eq!(parsed.overlays["Ghost"].has_art, Some(false));
}

/// spec: S-0003 claim 12 (lists), S-0107 Import mapping (art fields), S-0107 claim 22
#[test]
fn infantry_art_and_lists() {
    let base = "[InfantryTypes]\n1=Walker\n2=Runner\n[Walker]\nImage=WLK\nVeteranAbilities=faster,,Scatter\n\
                [Runner]\nX=1\n";
    let map = "[Walker]\nEliteAbilities=crusher\n";
    let long = format!("1,2,{}3,N", " ".repeat(30));
    let art = format!(
        "[WLK]\nCrawls=no\nFireUp=3\nSecondaryProne=5\nSequence=WalkSeq\n\
         [WalkSeq]\nReady=0,1,1\nWalk=8,6,6,E\nFireUp=1,2\nIdle1=4,x,9\nDie1=1,2,3,ne\n\
         Cheer=5,6,7,SW\nPanic={long}\nDeployedFire=1,2,3,4,5\n\
         [Runner]\nCrawls=yes\n"
    );
    let (s, w) = sim(base, map, &art);
    let wk = &s.infantry["Walker"];
    assert_eq!(wk.veteran_abilities, ["FASTER", "SCATTER"]);
    assert_eq!(wk.elite_abilities, ["CRUSHER"]);
    assert_eq!(wk.crawls, Some(false));
    assert_eq!(wk.fire_frames, Some([3, 0, 0, 5]));
    let q = &wk.sequences;
    let e = |k: &str| (q[k].start, q[k].count, q[k].multiplier, q[k].direction);
    assert_eq!(e("stand"), (0, 1, 1, None));
    assert_eq!(e("walk"), (8, 6, 6, Some(2)));
    // A missing field keeps it and every later one at 0.
    assert_eq!(e("fire_standing"), (1, 2, 0, None));
    assert_eq!(e("idle_a"), (4, 0, 0, None));
    // Lower case: no declared direction.
    assert_eq!(e("die_1"), (1, 2, 3, None));
    assert_eq!(e("cheer"), (5, 6, 7, Some(5)));
    // Cut at 31 characters: the multiplier lies beyond the cut.
    assert_eq!(e("panic"), (1, 2, 0, None));
    assert_eq!(e("deployed_fire"), (1, 2, 3, None));
    assert!(!q.contains_key("guard"));
    assert!(warned(
        &w,
        "Idle1=\"4,x,9\": the count field does not parse"
    ));
    assert!(warned(
        &w,
        "FireUp=\"1,2\": the multiplier field is missing"
    ));
    assert!(warned(&w, "Die1=\"1,2,3,ne\": the direction"));
    assert!(warned(&w, "longer than 31 characters"));
    assert!(!warned(&w, "Ready="));
    assert_eq!(s.infantry["Runner"].crawls, Some(true));
    assert!(s.infantry["Runner"].sequences.is_empty());
    // Switches off: no cut, any case.
    let options = ImportOptions {
        art: ArtQuirks {
            sequence_value_length_limit: false,
            case_sensitive_direction: false,
            ..ArtQuirks::default()
        },
        ..ImportOptions::default()
    };
    let (s, _) = sim_with(base, map, &art, &options);
    let q = &s.infantry["Walker"].sequences;
    assert_eq!(q["die_1"].direction, Some(1));
    assert_eq!((q["panic"].multiplier, q["panic"].direction), (3, Some(0)));
}

/// spec: S-0107 claim 17 and Edge cases (the importer warns about an infantry type whose die-1 or die-2 entry has no image frames, with quirks.refused_death_keeps_unit on or off)
#[test]
fn empty_death_entries_warn() {
    let base =
        "[InfantryTypes]\n1=Full\n2=NoDie2\n3=Bare\n[Full]\nX=1\n[NoDie2]\nX=1\n[Bare]\nX=1\n";
    let art = "[Full]\nSequence=FullSeq\n[FullSeq]\nDie1=1,2,3\nDie2=4,5,6\n\
               [NoDie2]\nSequence=HalfSeq\n[HalfSeq]\nDie1=1,2,3\nDie2=4,0,6\n";
    let (_, w) = sim(base, "", art);
    let names: Vec<&str> = ["Full", "NoDie2", "Bare"]
        .into_iter()
        .filter(|n| warned(&w, &format!("infantry {n}: its die-1 or die-2")))
        .collect();
    // A die-2 entry with count 0, and the all-zero table of a type without a sequence section.
    assert_eq!(names, ["NoDie2", "Bare"]);
}

/// spec: S-0108 Import mapping (Foundation: vocabulary, fallback, image and id sections)
#[test]
fn building_foundations() {
    let base = "[BuildingTypes]\n1=Hut\n2=Barn\n3=Silo\n4=Shed\n5=Tent\n\
                [Hut]\nImage=HUTART\nStrength=40\nNaval=yes\n[Barn]\nImage=BARNART\n\
                [Silo]\nX=1\n[Shed]\nImage=SHEDART\n[Tent]\nX=1\n";
    let art = "[HUTART]\nFoundation=3x3refinery\n[Barn]\nFoundation=1x1\n[BARNART]\nFoundation=2X3\n\
               [Silo]\nFoundation=Huge\n[SHEDART]\nFoundation=2x2\n[Shed]\nFoundation=0x0\n";
    let (s, w) = sim(base, "[Silo]\nY=2\n", art);
    let b = &s.buildings;
    assert_eq!(b["Hut"].foundation, Some([3, 3]));
    assert_eq!(b["Hut"].strength, Some(40));
    assert_eq!(b["Hut"].naval, Some(true));
    // The id's section cannot set the size back to 1x1.
    assert_eq!(b["Barn"].foundation, Some([2, 3]));
    assert_eq!(b["Silo"].foundation, Some([1, 1]));
    // Read in two layers, from one section that is both the image's and the id's: one warning.
    assert_eq!(
        w.iter()
            .filter(|w| w.message.contains("Foundation \"Huge\""))
            .count(),
        1
    );
    // The id's section decides with another size.
    assert_eq!(b["Shed"].foundation, Some([0, 0]));
    assert!(!b.contains_key("Tent"));
}

/// Every record kind with many fields resolves through `resolve_ruleset`
/// (`deny_unknown_fields` catches any misnamed field).
/// spec: S-0004 (registries), S-0006 claim 3
#[test]
fn every_record_kind_resolves() {
    let unit_keys: String = UNIT_FIELDS
        .iter()
        .map(|(_, k, f)| match f {
            Form::Int => format!("{k}=3\n"),
            Form::Bool => format!("{k}=yes\n"),
            Form::Dec => format!("{k}=0.5\n"),
            Form::Text | Form::Ref(_) | Form::Refs(_) => format!("{k}=Name{k}\n"),
            Form::Ints => format!("{k}=1,2\n"),
        })
        .collect();
    let section = |name: &str, table: &[(&str, &str, Form)]| -> String {
        let mut s = format!("[{name}]\n");
        for (_, k, f) in table {
            s.push_str(&match f {
                Form::Int => format!("{k}=2\n"),
                Form::Bool => format!("{k}=no\n"),
                Form::Dec => format!("{k}=25%\n"),
                Form::Text | Form::Ref(_) | Form::Refs(_) => format!("{k}=T{k}\n"),
                Form::Ints => format!("{k}=3,4\n"),
            });
        }
        s
    };
    let mut base = String::from(
        "[InfantryTypes]\n1=Walker\n[VehicleTypes]\n1=Cart\n[AircraftTypes]\n1=Kite\n\
         [BuildingTypes]\n1=Hut\n[OverlayTypes]\n1=Ore\n[TerrainTypes]\n1=Elm\n\
         [Warheads]\n1=Pow\n[Countries]\n1=Ruritania\n\
         [Hut]\nPrimary=Zap\n[Zap]\nProjectile=Dart\nWarhead=Pow\n",
    );
    base.push_str(&format!(
        "[Walker]\n{unit_keys}SpeedType=Foot\nMovementZone=Infantry\n\
         [Cart]\n{unit_keys}MovementRestrictedTo=Road\n[Kite]\n{unit_keys}"
    ));
    base.push_str(&section("Ruritania", COUNTRY_FIELDS));
    base.push_str(&section("Ore", OVERLAY_FIELDS));
    base.push_str("Land=Clear\n");
    base.push_str(&section("Elm", TERRAIN_OBJECT_FIELDS));
    base.push_str(&section("Pow", WARHEAD_FIELDS));
    let mut sections: BTreeMap<String, String> = BTreeMap::new();
    for (_, sec, key, f) in GENERAL_FIELDS {
        let v = match f {
            Form::Int => "1",
            Form::Bool => "yes",
            _ => "0.5",
        };
        sections
            .entry(show(sec))
            .or_default()
            .push_str(&format!("{key}={v}\n"));
    }
    for (sec, keys) in sections {
        base.push_str(&format!("[{sec}]\n{keys}"));
    }
    let map = section("Hut", BUILDING_FIELDS);
    let (s, _) = sim(&base, &map, "[Hut]\nFoundation=2x2\n");
    assert_eq!(s.infantry["Walker"].sight, Some(3));
    assert_eq!(
        s.vehicles["Cart"].movement_restricted_to,
        Some(LandType::Road)
    );
    assert_eq!(dec(&s.aircraft["Kite"].guard_range), Some("5e-1"));
    assert_eq!(s.buildings["Hut"].tech_level, Some(2));
    // spec: S-0311 F1, S-0314 claim 5 (the building cost field)
    assert_eq!(s.buildings["Hut"].cost, Some(2));
    assert_eq!(s.buildings["Hut"].foundation, Some([2, 2]));
    assert_eq!(s.overlays["Ore"].crate_, Some(false));
    assert_eq!(s.terrain_objects["Elm"].snow_occupation_bits, Some(2));
    assert_eq!(s.warheads["Pow"].inf_death, Some(2));
    assert_eq!(dec(&s.countries["Ruritania"].speed_units_mult), Some("25%"));
    assert_eq!(s.general.iq_auto_crush, Some(1));
    assert_eq!(s.general.player_auto_crush, Some(true));
    // Weapons and projectiles exist by reference only (S-0004 claims 3 and
    // 13): the building's primary weapon and that weapon's projectile.
    assert_eq!(s.weapons.keys().collect::<Vec<_>>(), ["Zap"]);
    assert_eq!(s.projectiles.keys().collect::<Vec<_>>(), ["Dart"]);
}

/// Weapon and projectile records carry valid field names (the record is
/// put in both registries by hand, so that one section feeds both).
/// spec: S-0004 claim 3
#[test]
fn weapon_and_projectile_fields_resolve() {
    let text = "[Warheads]\n1=Zap\n[Zap]\nDamage=5\nProjectile=Dart\nWarhead=Pow\nOmniFire=yes\n\
                AreaFire=no\nRadLevel=2\nAmbientDamage=1\nROT=3\nAG=yes\n";
    let layers = [crate::layers::Layer {
        role: crate::layers::Role::Base,
        file: "rules.ini".into(),
        doc: Document::parse(text.as_bytes(), Default::default(), &mut Vec::new()),
    }];
    let art = Document::empty(Default::default());
    let mut outcome = crate::layers::apply(
        &layers,
        &art,
        "art.ini",
        Default::default(),
        &LayerQuirks::default(),
    );
    let zap = outcome.registries[&Kind::Warheads][0].clone();
    outcome.registries.insert(Kind::Weapons, vec![zap.clone()]);
    outcome.registries.insert(Kind::Projectiles, vec![zap]);
    let (node, _) = build(SimInputs {
        outcome: &outcome,
        art: &art,
        arts: None,
        quirks: &LayerQuirks::default(),
        art_quirks: &ArtQuirks::default(),
        ini_quirks: &Default::default(),
        sim_quirks: &SimQuirks::default(),
    });
    let parsed: SimData = glasnost_rules::de::from_node(&node).unwrap();
    let w = &parsed.weapons["Zap"];
    assert_eq!(
        (w.damage, w.ambient_damage, w.rad_level),
        (Some(5), Some(1), Some(2))
    );
    assert_eq!(w.projectile.as_deref(), Some("Dart"));
    assert_eq!((w.omni_fire, w.area_fire), (Some(true), Some(false)));
    assert_eq!(
        (parsed.projectiles["Zap"].rot, parsed.projectiles["Zap"].ag),
        (Some(3), Some(true))
    );
}

/// spec: S-0215 claim 36 (b) and Questions for the project lead (decided: no layer gives the guard stray distance: the importer warns)
#[test]
fn guard_stray_absent_warns() {
    let (s, w) = sim("[General]\nCloseEnough=1\n", "", "");
    assert_eq!(s.general.guard_mode_stray, None);
    assert!(warned(&w, "GuardModeStray is given by no rules layer"));
    let (_, w) = sim("[General]\nGuardModeStray=2\n", "", "");
    assert!(!warned(&w, "GuardModeStray"));
}

/// spec: S-0214 Import mapping (Teleporter); S-0215 Import mapping (DeployTime and GateCloseDelay, game minutes)
#[test]
fn teleporter_and_gate_times() {
    let base = "[VehicleTypes]\n1=Cart\n[BuildingTypes]\n1=Door\n\
                [Cart]\nTeleporter=yes\n[Door]\nGate=yes\nDeployTime=0.1\nGateCloseDelay=0.2\n";
    let (s, _) = sim(base, "", "");
    assert_eq!(s.vehicles["Cart"].teleporter, Some(true));
    let d = &s.buildings["Door"];
    assert_eq!(d.gate, Some(true));
    assert_eq!(dec(&d.deploy_time), Some("1e-1"));
    assert_eq!(dec(&d.gate_close_delay), Some("2e-1"));
}

/// spec: S-0009 Import mapping and claims 33 and 45 (starting-unit fields; the game's tech level and the early phase's passive field from the base rules only); S-0112 Import mapping (gate and tower types); S-0202 Import mapping (IsVeins)
#[test]
fn starting_unit_gate_and_vein_fields() {
    let base = "[Countries]\n0=Alpha\n1=Beta\n[VehicleTypes]\n0=TRUCK\n1=MCV\n[InfantryTypes]\n0=GUY\n\
                [BuildingTypes]\n0=GATEA\n1=TOWER\n[OverlayTypes]\n0=VEIN\n\
                [General]\nBaseUnit=mcv,NOPE\nGDIGateOne=gatea\nWallTower=TOWER\n\
                [MultiplayerDialogSettings]\nTechLevel=7\n\
                [Alpha]\nMultiplayPassive=no\n[Beta]\n\
                [TRUCK]\nCost=600\nTechLevel=3\nOwner=beta,Alpha, Beta,Gamma\nLargeVisceroid=yes\n\
                [GUY]\nAllowedToStartInMultiplayer=no\n[VEIN]\nIsVeins=yes\n";
    let map = "[MultiplayerDialogSettings]\nTechLevel=2\n[Alpha]\nMultiplayPassive=yes\n";
    let (s, _) = sim(base, map, "");
    let g = &s.general;
    // spec: S-0004 claim 13 ([General] BaseUnit is a creating list field)
    assert_eq!(
        g.base_unit,
        Some(vec!["MCV".to_string(), "NOPE".to_string()])
    );
    assert_eq!(
        g.multiplayer_tech_level,
        Some(7),
        "the base rules' value only"
    );
    assert_eq!(g.allied_gate_one.as_deref(), Some("GATEA"));
    assert_eq!(g.wall_tower.as_deref(), Some("TOWER"));
    assert_eq!(g.soviet_gate_one, None);
    let truck = &s.vehicles["TRUCK"];
    assert_eq!((truck.cost, truck.tech_level), (Some(600), Some(3)));
    assert_eq!(truck.large_visceroid, Some(true));
    // S-0003 claim 12: items are not trimmed, so " Beta" names no country.
    assert_eq!(
        truck.owner,
        Some(vec!["Beta".to_string(), "Alpha".to_string()])
    );
    assert_eq!(
        s.infantry["GUY"].allowed_to_start_in_multiplayer,
        Some(false)
    );
    let alpha = &s.countries["Alpha"];
    assert_eq!(alpha.multiplay_passive, Some(true));
    assert_eq!(alpha.multiplay_passive_base, Some(false));
    assert_eq!(s.overlays["VEIN"].is_veins, Some(true));
}

/// spec: S-0109 Import mapping and claim 3 (an animation type's random rate pair, −1 keeping that end; the bouncing flags, the chain reaction and the middle-event effects)
#[test]
fn animation_fields() {
    let base = "[Animations]\n0=SPARK\n1=ROCK\n";
    let art = "[SPARK]\nRandomRate=-1,30\nTiberiumChainReaction=yes\n[ROCK]\nIsMeteor=yes\nSpawnsParticle=none\n";
    let (s, _) = sim(base, "", art);
    let spark = &s.animations["SPARK"];
    assert_eq!(spark.random_rate, Some([0, 30]));
    assert_eq!(spark.tiberium_chain_reaction, Some(true));
    let rock = &s.animations["ROCK"];
    assert_eq!(rock.is_meteor, Some(true));
    assert_eq!(rock.spawns_particle, None);
}
