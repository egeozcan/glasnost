//! Converter tests on invented data (every name and number here is made up).

use glasnost_ini::{Document, Quirks};

use crate::inputs::{InputFile, Inputs};
use crate::kinds::Kind;
use crate::layers::{self, GameKind, Layer, LayerQuirks, Outcome, Role, What};
use crate::{ImportOptions, convert};

fn doc(text: &str) -> Document {
    Document::parse(text.as_bytes(), Quirks::default(), &mut Vec::new())
}

fn layer(role: Role, text: &str) -> Layer {
    Layer {
        role,
        file: format!("{}.ini", role.label()),
        doc: doc(text),
    }
}

fn apply_with(layers: &[Layer], art: &str, game: GameKind, q: LayerQuirks) -> Outcome {
    layers::apply(layers, &doc(art), "art.ini", game, &q)
}

fn apply(layers: &[Layer], art: &str) -> Outcome {
    apply_with(layers, art, GameKind::Skirmish, LayerQuirks::default())
}

fn names(o: &Outcome, kind: Kind) -> Vec<String> {
    o.registries[&kind]
        .iter()
        .map(|t| String::from_utf8_lossy(&t.name).into_owned())
        .collect()
}

fn ty<'a>(o: &'a Outcome, kind: Kind, name: &str) -> &'a layers::TypeRec {
    o.registries[&kind]
        .iter()
        .find(|t| t.name == name.as_bytes())
        .unwrap_or_else(|| panic!("{name} not registered"))
}

fn val(o: &Outcome, kind: Kind, name: &str, key: &str) -> Option<String> {
    ty(o, kind, name)
        .keys
        .get(key.as_bytes())
        .and_then(|k| k.value.as_ref())
        .map(|v| String::from_utf8_lossy(v).into_owned())
}

/// spec: S-0004 claims 2, 5 and 8 (scenarios A1, A2, C1 to C3)
#[test]
fn type_lists_register_in_entry_order() {
    let long = "Abcdefghijklmnopqrstuvwxyz12"; // 28 bytes
    let base = format!(
        "[BuildingTypes]\n1=Zeta\n2=Alpha\n3=ZETA\n4=none\n5=<NONE>\n6={long}\n7={long}\n\
         8=Exactly24BytesLongName24\n9=Exactly24BytesLongName24       X\n"
    );
    let o = apply(&[layer(Role::Base, &base)], "");
    assert_eq!(
        names(&o, Kind::Buildings),
        [
            "Zeta",
            "Alpha",
            "Abcdefghijklmnopqrstuvwx",
            "Abcdefghijklmnopqrstuvwx",
            "Exactly24BytesLongName24"
        ]
    );
    // With the switch off, all references to a long name resolve to one type.
    let q = LayerQuirks {
        long_name_duplicates: false,
        ..LayerQuirks::default()
    };
    let o = apply_with(&[layer(Role::Base, &base)], "", GameKind::Skirmish, q);
    assert_eq!(names(&o, Kind::Buildings).len(), 4);
}

/// spec: S-0004 claim 2 (a repeated key gives the winning occurrence's value
/// at every one of its positions, S-0002 claim 4)
#[test]
fn repeated_list_key_uses_the_winning_value() {
    let base = "[VehicleTypes]\n1=Ant\n2=Bee\n1=Cat\n";
    let d = doc(base);
    let winner = d
        .value(b"VehicleTypes", b"1")
        .map(|v| String::from_utf8_lossy(v).into_owned())
        .expect("key 1");
    let loser = if winner == "Ant" { "Cat" } else { "Ant" };
    let o = apply(&[layer(Role::Base, base)], "");
    // Positions 1 and 3 both take the winner's value; the other line's
    // value names no type.
    assert_eq!(names(&o, Kind::Vehicles), [winner.as_str(), "Bee"]);
    assert!(!names(&o, Kind::Vehicles).iter().any(|n| n == loser));
}

/// spec: S-0004 claim 7 (a type reads the section that shares its name hash
/// while `quirks.ini_hash_identity` is on, S-0002 claim 3)
#[test]
fn type_section_found_by_shared_name_hash() {
    // "ZdxeLZgd" and "Cost" share a name hash (S-0002 fixture 3).
    let text = "[VehicleTypes]\n1=Cost\n[ZdxeLZgd]\nStrength=5\n";
    let with = |identity: bool| {
        let q = Quirks {
            ini_hash_identity: identity,
            ..Quirks::default()
        };
        let l = Layer {
            role: Role::Base,
            file: "base.ini".to_owned(),
            doc: Document::parse(text.as_bytes(), q, &mut Vec::new()),
        };
        apply(&[l], "")
    };
    assert_eq!(
        val(&with(true), Kind::Vehicles, "Cost", "Strength").as_deref(),
        Some("5")
    );
    assert_eq!(val(&with(false), Kind::Vehicles, "Cost", "Strength"), None);
}

/// spec: S-0004 claims 4, 6 and 7; S-0006 claim 7 (scenario B1)
#[test]
fn referenced_types_follow_rank_and_layer_rules() {
    let base = "[VehicleTypes]\n1=Truck\n[BuildingTypes]\n1=Depot\n\
                [Truck]\nDeploysInto=Shed\n[Shed]\nStrength=5\n\
                [Warheads]\n1=Boom\n";
    let map = "[Shed]\nStrength=9\n";
    let o = apply(
        &[
            layer(Role::Base, base),
            layer(Role::Mode, ""),
            layer(Role::Map, map),
        ],
        "",
    );
    assert_eq!(names(&o, Kind::Buildings), ["Depot", "Shed"]);
    let shed = ty(&o, Kind::Buildings, "Shed");
    // Created during the vehicles' pass (rank 6 > 4): the base section is never read.
    assert_eq!(shed.reads, [2]);
    assert_eq!(
        val(&o, Kind::Buildings, "Shed", "Strength").as_deref(),
        Some("9")
    );
    // Switch off: the creating layer is read too.
    let q = LayerQuirks {
        unlisted_reference_unread: false,
        ..LayerQuirks::default()
    };
    let o = apply_with(&[layer(Role::Base, base)], "", GameKind::Skirmish, q);
    assert_eq!(ty(&o, Kind::Buildings, "Shed").reads, [0]);
}

/// spec: S-0004 claim 6 (a particle that only a particle system names is not read)
#[test]
fn particles_named_by_particle_systems_are_unread() {
    let base = "[ParticleSystems]\n1=Smoke\n[Smoke]\nHoldsWhat=Puff\n[Puff]\nX=1\n";
    let o = apply(&[layer(Role::Base, base)], "");
    assert!(ty(&o, Kind::Particles, "Puff").reads.is_empty());
    // Listed, it is read.
    let listed = format!("{base}[Particles]\n1=Puff\n");
    let o = apply(&[layer(Role::Base, &listed)], "");
    assert_eq!(ty(&o, Kind::Particles, "Puff").reads, [0]);
    assert_eq!(names(&o, Kind::Particles), ["Puff"]);
}

/// spec: S-0004 claims 4 and 6 (crate overlays, row 4; scenario G)
#[test]
fn crate_overlays_come_after_listed_ones() {
    let base = "[CrateRules]\nCrateImg=Box\n[OverlayTypes]\n1=Rock\n2=Tree\n[Box]\nX=1\n";
    let map = "[OverlayTypes]\n1=Fern\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    assert_eq!(names(&o, Kind::Overlays), ["Rock", "Tree", "Box", "Fern"]);
    assert_eq!(ty(&o, Kind::Overlays, "Box").reads, Vec::<usize>::new());
}

/// spec: S-0004 claim 10 and Formulas (scenarios D1 to D3)
#[test]
fn tiberiums_by_key_number() {
    let base = "[Tiberiums]\n0=Ore\n1=Gem\n[Ore]\nValue=7\n[Gem]\nValue=9\n";
    let map = "[Tiberiums]\n0abc=Fake\n5=Novel\n-1=Bad\n[Ore]\nValue=8\n[Fake]\nValue=1\n[Novel]\nValue=3\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    assert_eq!(names(&o, Kind::Tiberiums), ["Ore", "Gem", "Novel"]);
    // Key 0 picks the existing first tiberium; the map's list entry lets it read its section.
    assert_eq!(
        val(&o, Kind::Tiberiums, "Ore", "Value").as_deref(),
        Some("8")
    );
    assert!(o.warnings.iter().any(|w| w.message.contains("negative")));
    // Without a list entry in the layer, a tiberium does not read it (D2).
    let o = apply(
        &[
            layer(Role::Base, base),
            layer(Role::Map, "[Ore]\nValue=8\n"),
        ],
        "",
    );
    assert_eq!(
        val(&o, Kind::Tiberiums, "Ore", "Value").as_deref(),
        Some("7")
    );
    assert_eq!(layers::tiberium_key(b"  +12x"), 12);
    assert_eq!(layers::tiberium_key(b"4294967297"), 1);
}

/// spec: S-0004 claims 1 and 9 (scenarios E1, E2, E4)
#[test]
fn sides_from_lists_and_country_fields() {
    // The side list names the second side first; the countries name the first first.
    let base = "[Countries]\n1=Red\n2=Blue\n3=Green\n\
                [Sides]\nSouth=Blue\nNorth=Red,Green\n\
                [Red]\nSide=North\n[Blue]\nSide=South\n[Green]\nSide=North\n";
    let skirmish = apply(&[layer(Role::Base, base)], "");
    let side_names: Vec<String> = skirmish
        .sides
        .iter()
        .map(|s| String::from_utf8_lossy(&s.name).into())
        .collect();
    assert_eq!(side_names, ["North", "South"]);
    let campaign = apply_with(
        &[layer(Role::Base, base)],
        "",
        GameKind::Campaign,
        LayerQuirks::default(),
    );
    let side_names: Vec<String> = campaign
        .sides
        .iter()
        .map(|s| String::from_utf8_lossy(&s.name).into())
        .collect();
    assert_eq!(side_names, ["South", "North"]);
    // A map that moves Green to the South side.
    let map = "[Green]\nSide=South\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    assert_eq!(ty(&o, Kind::Countries, "Green").side, Some(1));
    // A later side list that leaves a country out does not take its side away.
    let map = "[Sides]\nNorth=Red\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    assert_eq!(ty(&o, Kind::Countries, "Green").side, Some(0));
}

/// spec: S-0104 claim 12, S-0006 claim 5 (the value in force in the converted tree)
#[test]
fn computed_light_values_replace_their_text() {
    let inputs = Inputs {
        base: Some(file(
            "rulesmd.ini",
            "[BuildingTypes]\n1=Lamp\n[Lamp]\nLightIntensity=-3.0\n[AudioVisual]\nExtraUnitLight=0.75\n",
        )),
        map: Some(file("map.map", "[Lamp]\nStrength=2\n[AudioVisual]\nX=1\n")),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[]).unwrap();
    let lamp = &r.ruleset.types["buildings"]["Lamp"];
    assert!(!lamp.ini.contains_key("LightIntensity"));
    assert_eq!(lamp.derived["LightIntensity"], -2999 + 1000);
    assert!(!r.ruleset.ini_globals["AudioVisual"].contains_key("ExtraUnitLight"));
    assert_eq!(
        r.ruleset.ini_globals_derived["AudioVisual"]["ExtraUnitLight"],
        0
    );
    let path = glasnost_rules::explain::find_path(&r.tree, "Lamp.derived.LightIntensity").unwrap();
    let text = glasnost_rules::explain::explain(&r.tree, &r.provenance, &path);
    assert!(text.contains("truncated to -1999"), "{text}");
}

/// spec: S-0006 claims 3, 4 and 5 (scenarios A, C, D1, D3)
#[test]
fn layering_keeps_replaces_and_resets() {
    let base = "[Warheads]\n1=Boom\n[Boom]\nVerses=50%,50%\nSpread=3\n\
                [Countries]\n1=Red\n[Red]\nParentCountry=Blue\nOwner=A,B\n\
                [ParticleSystems]\n1=Smoke\n[Smoke]\nHoldsWhat=Puff\nBehavesLike=Gas\n\
                [Normal]\nFirePower=2\nCost=3\n";
    let map = "[Boom]\nSpread=4\n[Red]\nOwner=C\n[Smoke]\nLifetime=9\n[Normal]\nCost=5\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    // Absent keys keep, given keys replace, lists are replaced whole.
    assert_eq!(
        val(&o, Kind::Warheads, "Boom", "Spread").as_deref(),
        Some("4")
    );
    assert_eq!(
        val(&o, Kind::Countries, "Red", "Owner").as_deref(),
        Some("C")
    );
    // Reset on override.
    let verses = &ty(&o, Kind::Warheads, "Boom").keys[&b"Verses"[..]];
    assert!(verses.reset && verses.value.is_none());
    assert!(ty(&o, Kind::Countries, "Red").keys[&b"ParentCountry"[..]].reset);
    assert!(ty(&o, Kind::ParticleSystems, "Smoke").keys[&b"BehavesLike"[..]].reset);
    // The omitted emitted particle is the empty-named particle, created by the map layer.
    assert_eq!(names(&o, Kind::Particles), ["Puff", ""]);
    let normal = &o.globals[&b"Normal"[..]];
    assert!(normal[&b"FirePower"[..]].reset);
    assert_eq!(normal[&b"Cost"[..]].value.as_deref(), Some(&b"5"[..]));
    // Switch off: every key keeps its held value.
    let q = LayerQuirks {
        override_resets_keys: false,
        ..LayerQuirks::default()
    };
    let o = apply_with(
        &[layer(Role::Base, base), layer(Role::Map, map)],
        "",
        GameKind::Skirmish,
        q,
    );
    assert_eq!(
        val(&o, Kind::Warheads, "Boom", "Verses").as_deref(),
        Some("50%,50%")
    );
    assert_eq!(names(&o, Kind::Particles), ["Puff"]);
}

/// spec: S-0006 claim 5 (particle systems in the base rules), S-0004 claim 2 (`none` names a particle)
#[test]
fn empty_and_none_particles() {
    let base =
        "[ParticleSystems]\n1=A\n2=B\n3=C\n[A]\nX=1\n[B]\nHoldsWhat=none\n[C]\nHoldsWhat=<none>\n";
    let o = apply(&[layer(Role::Base, base)], "");
    assert_eq!(names(&o, Kind::Particles), ["", "none"]);
}

/// spec: S-0006 claim 5 (with quirks.override_resets_keys off, a reset key that the layer creating
/// the type omits holds the engine default, never its reset value: no reset is recorded and an omitted
/// emitted particle creates no empty-named particle)
#[test]
fn switch_off_creating_layer_holds_engine_defaults() {
    let base = "[Warheads]\n1=Boom\n[Boom]\nSpread=3\n\
                [ParticleSystems]\n1=Smoke\n[Smoke]\nLifetime=9\n\
                [Normal]\nCost=5\n";
    let q = LayerQuirks {
        override_resets_keys: false,
        ..LayerQuirks::default()
    };
    let o = apply_with(&[layer(Role::Base, base)], "", GameKind::Skirmish, q);
    let boom = ty(&o, Kind::Warheads, "Boom");
    assert!(
        boom.keys
            .get(&b"Verses"[..])
            .is_none_or(|k| !k.reset && k.value.is_none())
    );
    let smoke = ty(&o, Kind::ParticleSystems, "Smoke");
    assert!(smoke.keys.get(&b"BehavesLike"[..]).is_none_or(|k| !k.reset));
    assert!(names(&o, Kind::Particles).is_empty());
    let normal = &o.globals[&b"Normal"[..]];
    assert!(normal.get(&b"FirePower"[..]).is_none_or(|k| !k.reset));
    // Switch on: the same layer resets them.
    let o = apply(&[layer(Role::Base, base)], "");
    assert!(ty(&o, Kind::Warheads, "Boom").keys[&b"Verses"[..]].reset);
    assert_eq!(names(&o, Kind::Particles), [""]);
}

/// spec: S-0006 claims 2 and 3 (base-only maximum players; recharge time 0 not given)
#[test]
fn base_only_and_not_given() {
    let base = "[Maximums]\nPlayers=8\n[SuperWeaponTypes]\n1=Bolt\n[Bolt]\nRechargeTime=5\n";
    let map = "[Maximums]\nPlayers=2\n[Bolt]\nRechargeTime=0.0\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    let players = &o.globals[&b"Maximums"[..]][&b"Players"[..]];
    assert_eq!(players.value.as_deref(), Some(&b"8"[..]));
    assert!(matches!(players.history[1].what, What::Ignored(_)));
    assert_eq!(
        val(&o, Kind::Superweapons, "Bolt", "RechargeTime").as_deref(),
        Some("5")
    );
}

/// spec: S-0006 claim 5 (the twelve land-type sections reset omitted keys in the layered game-wide data)
#[test]
fn land_type_sections_reset_in_the_layering() {
    let base = "[Clear]\nFoot=50%\nBuildable=yes\n[Other]\nFoot=50%\n";
    let map = "[Clear]\nTrack=1\n[Other]\nTrack=1\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    let clear = &o.globals[&b"Clear"[..]];
    for k in [
        "Foot",
        "Wheel",
        "Hover",
        "Float",
        "Amphibious",
        "FloatBeach",
        "Buildable",
    ] {
        assert!(clear[k.as_bytes()].reset, "{k}");
        assert!(clear[k.as_bytes()].value.is_none(), "{k}");
    }
    assert_eq!(clear[&b"Track"[..]].value.as_deref(), Some(&b"1"[..]));
    // A section that is not a land type keeps per-key layering.
    let other = &o.globals[&b"Other"[..]];
    assert_eq!(other[&b"Foot"[..]].value.as_deref(), Some(&b"50%"[..]));
    let q = LayerQuirks {
        override_resets_keys: false,
        ..LayerQuirks::default()
    };
    let o = apply_with(
        &[layer(Role::Base, base), layer(Role::Map, map)],
        "",
        GameKind::Skirmish,
        q,
    );
    let clear = &o.globals[&b"Clear"[..]];
    assert_eq!(clear[&b"Foot"[..]].value.as_deref(), Some(&b"50%"[..]));
    assert!(!clear[&b"Buildable"[..]].reset);
}

/// spec: S-0006 claim 3 (the "not given" values of the pitch and roll angles and of a voxel
/// animation's angular velocities)
#[test]
fn angle_keys_not_given() {
    let base = "[VehicleTypes]\n1=Cart\n[Cart]\nPitchAngle=5\nRollAngle=6\n\
                [VoxelAnims]\n1=Spin\n[Spin]\nMinAngularVelocity=2\nMaxAngularVelocity=3\n";
    let map = "[Cart]\nPitchAngle=-100%\nRollAngle=-1.0\n\
               [Spin]\nMinAngularVelocity=0\nMaxAngularVelocity=0.0\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    assert_eq!(
        val(&o, Kind::Vehicles, "Cart", "PitchAngle").as_deref(),
        Some("5")
    );
    assert_eq!(
        val(&o, Kind::Vehicles, "Cart", "RollAngle").as_deref(),
        Some("6")
    );
    assert_eq!(
        val(&o, Kind::VoxelAnimations, "Spin", "MinAngularVelocity").as_deref(),
        Some("2")
    );
    assert_eq!(
        val(&o, Kind::VoxelAnimations, "Spin", "MaxAngularVelocity").as_deref(),
        Some("3")
    );
}

/// spec: S-0105 claim 27 and `quirks.barrel_recoil_layer_reset`
#[test]
fn barrel_values_follow_the_turret_per_layer() {
    let base = "[BuildingTypes]\n1=Tower\n[Tower]\nTurretTravel=3\nBarrelTravel=7\n";
    let map = "[Tower]\nTurretTravel=5\n";
    let o = apply(&[layer(Role::Base, base), layer(Role::Map, map)], "");
    assert_eq!(
        val(&o, Kind::Buildings, "Tower", "BarrelTravel").as_deref(),
        Some("5")
    );
    assert_eq!(val(&o, Kind::Buildings, "Tower", "BarrelHoldFrames"), None);
    let q = LayerQuirks {
        barrel_recoil_layer_reset: false,
        ..LayerQuirks::default()
    };
    let o = apply_with(
        &[layer(Role::Base, base), layer(Role::Map, map)],
        "",
        GameKind::Skirmish,
        q,
    );
    assert_eq!(
        val(&o, Kind::Buildings, "Tower", "BarrelTravel").as_deref(),
        Some("7")
    );
}

/// spec: S-0104 claim 12, S-0006 claim 5 (light values truncated on re-read; scenario P)
#[test]
fn light_values_truncate_on_re_read() {
    let base = "[BuildingTypes]\n1=Lamp\n2=Dim\n[Lamp]\nLightIntensity=-3.0\n[Dim]\nLightIntensity=0.5\n\
                [AudioVisual]\nExtraUnitLight=0.75\n";
    let mode = "[Lamp]\nStrength=1\n[Dim]\nStrength=1\n[AudioVisual]\nX=1\n";
    let map = "[Lamp]\nStrength=2\n";
    let o = apply(
        &[
            layer(Role::Base, base),
            layer(Role::Mode, mode),
            layer(Role::Map, map),
        ],
        "",
    );
    let lamp = &ty(&o, Kind::Buildings, "Lamp").keys[&b"LightIntensity"[..]];
    let steps: Vec<i64> = lamp
        .history
        .iter()
        .filter_map(|e| match e.what {
            What::Truncated(t) => Some(t),
            _ => None,
        })
        .collect();
    assert_eq!(steps, [-1999, -999]);
    assert_eq!(lamp.derived, Some(-999));
    assert_eq!(
        ty(&o, Kind::Buildings, "Dim").keys[&b"LightIntensity"[..]].derived,
        Some(0)
    );
    let extra = &o.globals[&b"AudioVisual"[..]][&b"ExtraUnitLight"[..]];
    assert_eq!(extra.derived, Some(0));
    assert_eq!(layers::building_light_thousandths(b"0.5"), Some(500));
    assert_eq!(layers::building_light_thousandths(b"-3.0"), Some(-2999));
}

/// spec: S-0104 claim 13, Edge cases (quirks.extra_light_truncation: 0.57
/// becomes 569; off, round to nearest)
#[test]
fn extra_light_truncation_switch() {
    assert_eq!(layers::extra_light_thousandths(b"0.57", true), Some(569));
    assert_eq!(layers::extra_light_thousandths(b"0.57", false), Some(570));
    assert_eq!(layers::extra_light_thousandths(b"-0.57", true), Some(-569));
    assert_eq!(layers::extra_light_thousandths(b"-0.57", false), Some(-570));
    assert_eq!(layers::extra_light_thousandths(b"0.2504", false), Some(250));
    assert_eq!(layers::extra_light_thousandths(b"0.0026", false), Some(3));
    assert_eq!(layers::extra_light_thousandths(b"x", false), None);
    let base = "[AudioVisual]\nExtraUnitLight=0.57\n";
    let derived = |q: LayerQuirks| {
        let o = apply_with(&[layer(Role::Base, base)], "", GameKind::Skirmish, q);
        o.globals[&b"AudioVisual"[..]][&b"ExtraUnitLight"[..]].derived
    };
    assert_eq!(derived(LayerQuirks::default()), Some(569));
    let off = LayerQuirks {
        extra_light_truncation: false,
        ..LayerQuirks::default()
    };
    assert_eq!(derived(off), Some(570));
}

/// spec: S-0006 claim 8, S-0007 Terms (animations read the art file in every layer)
#[test]
fn animations_read_art_in_every_layer() {
    let base = "[Animations]\n1=Spark\n[Spark]\nRate=1\n";
    let art = "[Spark]\nRate=5\n";
    let o = apply(
        &[
            layer(Role::Base, base),
            layer(Role::Mode, ""),
            layer(Role::Map, "[Spark]\nRate=9\n"),
        ],
        art,
    );
    let spark = ty(&o, Kind::Animations, "Spark");
    assert_eq!(spark.reads, [0, 1, 2]);
    assert_eq!(
        val(&o, Kind::Animations, "Spark", "Rate").as_deref(),
        Some("5")
    );
    let o = apply_with(
        &[layer(Role::Base, base), layer(Role::Map, "")],
        art,
        GameKind::Campaign,
        LayerQuirks::default(),
    );
    assert_eq!(ty(&o, Kind::Animations, "Spark").reads.len(), 2);
}

/// spec: S-0103 claims 4 and 5; S-0006 claim 2 (colour definitions and the colour-adjustment table)
#[test]
fn colours_and_colour_add() {
    let base = "[Colors]\nRuby=0,200,200\nJade=90,200,200\n[ColorAdd]\nNone=0,0,0\nTint=10,20,30\n";
    let lang = "[Colors]\nruby=1,1,1\nOnyx=0,0,10\n[ColorAdd]\nA=1,1,1\nB=2,2,2\nC=3,3,3\n";
    let map = "[ColorAdd]\nX=9,9,9\n";
    let o = apply(
        &[
            layer(Role::Base, base),
            layer(Role::Language, lang),
            layer(Role::Map, map),
        ],
        "",
    );
    let defs: Vec<(String, [u8; 3])> = o
        .colours
        .iter()
        .map(|c| (String::from_utf8_lossy(&c.name).into(), c.hsv))
        .collect();
    assert_eq!(
        defs,
        [
            ("Ruby".into(), [0, 200, 200]),
            ("Jade".into(), [90, 200, 200]),
            ("Onyx".into(), [0, 0, 10])
        ]
    );
    assert_eq!(o.colour_add[0].0, [9, 9, 9]);
    assert_eq!(o.colour_add[1].0, [10, 20, 30]);
    assert_eq!(o.colour_add[2].0, [3, 3, 3]);
    assert_eq!(o.colour_add[3].0, [0, 0, 0]);
}

fn file(name: &str, text: &str) -> InputFile {
    InputFile {
        name: name.into(),
        origin: "test".into(),
        bytes: text.as_bytes().to_vec(),
    }
}

/// spec: S-0006 claims 1 and 13 (a skirmish plays the selected listed mode's file; no listed mode, an empty layer)
#[test]
fn skirmish_mode_from_the_mode_list() {
    use crate::modes::ModeChoice;
    let list = file(
        "mpmodesmd.ini",
        "[Battle]\n1=N,T,first.ini,F\n2=N,T,second.ini,F\n[Siege]\n2=N,T,third.ini,F\n",
    );
    let pick = |mode, first_wins| {
        let mut o = ImportOptions {
            mode,
            ..ImportOptions::default()
        };
        o.quirks.mode_number_first_wins = first_wins;
        crate::mode_from_list(Some(&list), &o).0
    };
    assert_eq!(
        pick(ModeChoice::Preselected, true).as_deref(),
        Some("first.ini")
    );
    assert_eq!(
        pick(ModeChoice::Remembered(2), true).as_deref(),
        Some("second.ini")
    );
    assert_eq!(
        pick(ModeChoice::Listed(2), true).as_deref(),
        Some("second.ini")
    );
    assert_eq!(
        pick(ModeChoice::Listed(2), false).as_deref(),
        Some("third.ini")
    );
    let (none, w) = crate::mode_from_list(None, &ImportOptions::default());
    assert_eq!(none, None);
    assert!(w[0].message.contains("cannot be resolved"));
    let empty = file("mpmodesmd.ini", "[battle]\n1=N,T,first.ini,F\n");
    let (none, w) = crate::mode_from_list(Some(&empty), &ImportOptions::default());
    assert_eq!(none, None);
    assert!(w.iter().any(|w| w.message.contains("lists no mode")));
}

/// The converted tree resolves through the rules pipeline, explains every
/// value back to its INI line, and keeps high bytes.
#[test]
fn converted_tree_resolves_and_explains() {
    let inputs = Inputs {
        base: Some(file(
            "rulesmd.ini",
            "[VehicleTypes]\n1=Truck\n[Truck]\nCost=100\nImage=TRK\n[Hi\u{e9}]\nK=v\n",
        )),
        map: Some(file("test.map", "[Truck]\nCost=150\n")),
        art: Some(file("artmd.ini", "[TRK]\nVoxel=yes\n")),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let resolved = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &[],
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let truck = &resolved.ruleset.types["vehicles"]["Truck"];
    assert_eq!(truck.ini["Cost"], "150");
    assert_eq!(truck.reads, ["base rules", "map"]);
    let path = glasnost_rules::explain::find_path(&resolved.tree, "Truck.ini.Cost").unwrap();
    let text = glasnost_rules::explain::explain(&resolved.tree, &resolved.provenance, &path);
    assert!(
        text.contains("rulesmd.ini (test) line 4 | set \"100\""),
        "{text}"
    );
    assert!(
        text.contains("test.map (test) line 2 | set \"150\""),
        "{text}"
    );
    // A mod patch on top.
    let patched = glasnost_rules::resolve_ruleset(
        Some((import.tree.clone(), import.provenance.clone())),
        &[glasnost_rules::resolve::Source {
            layer: "mod 1",
            file: "mod.jsonc",
            text: r#"{"types": {"vehicles": {"Truck": {"ini": {"Cost": "175"}}}}}"#,
        }],
    )
    .unwrap();
    assert_eq!(
        patched.ruleset.types["vehicles"]["Truck"].ini["Cost"],
        "175"
    );
    assert_ne!(patched.sim_rules_hash, resolved.sim_rules_hash);
    // The "Hi\u{e9}" section's bytes survive the round trip.
    let utf8_name = "Hi\u{e9}".as_bytes();
    let key = crate::names::encode(utf8_name);
    assert!(resolved.ruleset.ini_globals.contains_key(&key));
    assert_eq!(crate::names::decode(&key).unwrap(), utf8_name);
    // Writing the tree and parsing it back gives the same hash.
    let text = glasnost_rules::jsonc::write(&import.tree);
    let again = glasnost_rules::jsonc::parse(&text).unwrap();
    assert_eq!(
        glasnost_rules::hash::sim_rules_hash(&again),
        glasnost_rules::hash::sim_rules_hash(&import.tree)
    );
    // Coverage rows exist for every key, names only.
    assert!(
        import
            .coverage
            .iter()
            .any(|r| r.category == "vehicles" && r.key == "Image")
    );
    assert!(
        import
            .coverage
            .iter()
            .any(|r| r.category == "art" && r.key == "Voxel")
    );
    assert!(crate::coverage::csv(&import.coverage).starts_with("category,key,outcome,spec,note\n"));
}

/// spec: S-0001 claim 13 and Edge cases (case-only key warning)
#[test]
fn case_only_keys_warn() {
    let inputs = Inputs {
        base: Some(file(
            "rulesmd.ini",
            "[General]\nMaxdebris=3\n[VehicleTypes]\n1=V\n[V]\nimage=X\n",
        )),
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    let case: Vec<&str> = import
        .warnings
        .iter()
        .filter(|w| w.message.contains("letter case"))
        .map(|w| w.message.as_str())
        .collect();
    assert_eq!(case.len(), 2, "{case:?}");
}

/// spec: S-0006 claim 12 and `quirks.bad_language_rules_skips_mode`
#[test]
fn failing_language_rules_skip_the_mode_layer() {
    let inputs = || Inputs {
        base: Some(file(
            "rulesmd.ini",
            "[ColorAdd]\nA=1,2,3\nB=1,1,1\nC=5,5,5\n[Animations]\n1=Spark\n[VehicleTypes]\n1=V\n[V]\nCost=1\n",
        )),
        language: Some(file("langrule.ini", "[V]\nCost=2\n")),
        mode: Some(file("mode.ini", "[V]\nCost=3\n")),
        map: Some(file("map.map", "[ColorAdd]\nA=4,5,6\nB=7,8,9\n")),
        art: Some(file("artmd.ini", "[Spark]\nRate=1\n")),
        ..Inputs::default()
    };
    let on = convert(&inputs(), &ImportOptions::default());
    let v = &on.outcome.registries[&Kind::Vehicles][0];
    assert_eq!(v.keys[&b"Cost"[..]].value.as_deref(), Some(&b"1"[..]));
    // Base and map only: two reads of the art database.
    assert_eq!(on.outcome.registries[&Kind::Animations][0].reads.len(), 2);
    // Only the map's entries; position 3 stays zero although the base rules give it.
    assert_eq!(on.outcome.colour_add[0].0, [4, 5, 6]);
    assert_eq!(on.outcome.colour_add[1].0, [7, 8, 9]);
    assert_eq!(on.outcome.colour_add[2].0, [0, 0, 0]);
    assert!(
        on.warnings
            .iter()
            .any(|w| w.message.contains("game-mode layer is skipped"))
    );
    let options = ImportOptions {
        quirks: LayerQuirks {
            bad_language_rules_skips_mode: false,
            ..LayerQuirks::default()
        },
        ..ImportOptions::default()
    };
    let off = convert(&inputs(), &options);
    let v = &off.outcome.registries[&Kind::Vehicles][0];
    assert_eq!(v.keys[&b"Cost"[..]].value.as_deref(), Some(&b"3"[..]));
    assert_eq!(off.outcome.registries[&Kind::Animations][0].reads.len(), 3);
    assert_eq!(off.outcome.colour_add[2].0, [5, 5, 5]);
}

/// spec: S-0005 claim 16, S-0006 claim 12
#[test]
fn import_reports_errors_and_departures() {
    let inputs = Inputs {
        base: None,
        language: Some(file("langrule.ini", "[A]\nk=v\n")),
        missing_databases: vec!["soundmd.ini".into()],
        case_clashes: vec![("rulesmd.ini".into(), "RULESMD.INI".into())],
        ..Inputs::default()
    };
    let import = convert(&inputs, &ImportOptions::default());
    assert_eq!(import.errors.len(), 2);
    assert!(import.errors.iter().any(|e| e.contains("letter case")));
    assert!(
        import
            .warnings
            .iter()
            .any(|w| w.message.contains("treated as empty"))
    );
    assert!(
        import
            .warnings
            .iter()
            .any(|w| w.message.contains("digest is missing"))
    );
}
