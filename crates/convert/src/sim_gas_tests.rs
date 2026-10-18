//! Tests of the gas-cloud fields of the `sim` section (S-0613 Import
//! mapping) on invented data (every name and number here is made up).

use glasnost_rules::sim::SimData;

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

fn import(base: &str, map: &str, art: &str) -> crate::Import {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        map: Some(file("test.map", map)),
        art: Some(file("art.ini", art)),
        ..Inputs::default()
    };
    convert(&inputs, &ImportOptions::default())
}

fn sim(base: &str, map: &str, art: &str) -> (SimData, Vec<Warning>) {
    let import = import(base, map, art);
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset.sim, import.warnings)
}

fn warned(w: &[Warning], text: &str) -> bool {
    w.iter().any(|w| w.message.contains(text))
}

/// spec: S-0613 Import mapping (MaxDC, Damage, Warhead, StartFrame,
/// StartStateAI, EndStateAI, StateAIAdvance, DeleteOnStateLimit,
/// NextParticle, NextParticleOffset, WindEffect and Translucency of a
/// particle type; absent keys give no value, so the engine defaults apply)
#[test]
fn particle_gas_fields_present_and_absent() {
    let base = "[Particles]\n0=Puff\n1=Wisp\n2=Bare\n[Warheads]\n1=Sting\n\
                [Puff]\nBehavesLike=Gas\nMaxEC=40\nMaxDC=6\nDamage=9\nWarhead=Sting\n\
                StartFrame=2\nStartStateAI=1\nEndStateAI=300\nStateAIAdvance=5\n\
                DeleteOnStateLimit=yes\nNextParticle=Wisp\nNextParticleOffset=1,-2,3\n\
                WindEffect=2\nTranslucency=50\n\
                [Wisp]\nBehavesLike=gas\nNextParticle=<none>\nNextParticleOffset=4\n";
    let (s, w) = sim(base, "", "");
    let p = &s.particles["Puff"];
    assert_eq!(p.max_ec, Some(40));
    assert_eq!(p.max_dc, Some(6));
    assert_eq!(p.damage, Some(9));
    assert_eq!(p.warhead.as_deref(), Some("Sting"));
    assert_eq!(p.start_frame, Some(2));
    assert_eq!(p.start_state_ai, Some(1));
    // The importer keeps the value as written; the simulation reduces it to a signed byte.
    assert_eq!(p.end_state_ai, Some(300));
    assert_eq!(p.state_ai_advance, Some(5));
    assert_eq!(p.delete_on_state_limit, Some(true));
    assert_eq!(p.next_particle.as_deref(), Some("Wisp"));
    assert_eq!(p.next_particle_offset, Some([1, -2, 3]));
    assert_eq!(p.wind_effect, Some(2));
    assert_eq!(p.translucency, Some(50));
    let q = &s.particles["Wisp"];
    assert_eq!(q.behaves_like.as_deref(), Some("gas"));
    assert_eq!(q.next_particle, None, "<none> names no type");
    // One readable component: the others keep the value in force (0, 0).
    assert_eq!(q.next_particle_offset, Some([4, 0, 0]));
    assert!(warned(&w, "particles Wisp NextParticleOffset"), "{w:?}");
    let bare = &s.particles["Bare"];
    assert_eq!(bare.max_dc, None);
    assert_eq!(bare.damage, None);
    assert_eq!(bare.warhead, None);
    assert_eq!(bare.state_ai_advance, None);
    assert_eq!(bare.delete_on_state_limit, None);
    assert_eq!(bare.next_particle, None);
    assert_eq!(bare.next_particle_offset, None);
    assert_eq!(bare.wind_effect, None);
    assert_eq!(bare.translucency, None);
}

/// spec: S-0613 Import mapping and claim 1 (a particle-system type's
/// BehavesLike, HoldsWhat and Lifetime; the shared gas system exists with no
/// fields when no section names it)
#[test]
fn particle_system_fields_and_shared_gas_system() {
    let base = "[Particles]\n0=Puff\n[ParticleSystems]\n0=Cloud\n1=Still\n\
                [Puff]\nBehavesLike=Gas\n\
                [Cloud]\nBehavesLike=GAS\nHoldsWhat=Puff\nLifetime=90\n[Still]\nHoldsWhat=<none>\n";
    let (s, _) = sim(base, "", "");
    let c = &s.particle_systems["Cloud"];
    assert_eq!(c.behaves_like.as_deref(), Some("gas"));
    assert_eq!(c.holds_what.as_deref(), Some("Puff"));
    assert_eq!(c.lifetime, Some(90));
    let still = &s.particle_systems["Still"];
    assert_eq!(still.behaves_like, None);
    assert_eq!(still.holds_what, None);
    assert_eq!(still.lifetime, None, "absent: the default −1 applies");
    let gas = &s.particle_systems["GasCloudSys"];
    assert_eq!(gas.behaves_like, None);
    assert_eq!(gas.holds_what, None);
    assert_eq!(gas.lifetime, None);
}

/// spec: S-0613 Terms (empty-named type), claim 4 and Import mapping
/// (HoldsWhat absent from a section that is read: the empty-named type;
/// a system type never read: none) and S-0004 claim 13 (the empty-named
/// particle type registered by reference, at the end of the list)
#[test]
fn absent_holds_what_in_a_read_section_names_the_empty_type() {
    let base = "[Particles]\n0=Puff\n[ParticleSystems]\n0=Plain\n1=Unread\n\
                [Puff]\nBehavesLike=Gas\n[Plain]\nBehavesLike=Smoke\nLifetime=5\n";
    let import = import(base, "", "");
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"))
        .ruleset;
    let (names, s) = (&r.registries["particles"], &r.sim);
    let empty = names.last().expect("a particle type");
    assert_eq!(names.len(), 2, "Puff, then the empty-named type: {names:?}");
    assert_ne!(empty, "Puff");
    let plain = &s.particle_systems["Plain"];
    assert_eq!(plain.holds_what.as_ref(), Some(empty), "read without HoldsWhat");
    let unread = s.particle_systems.get("Unread").cloned().unwrap_or_default();
    assert_eq!(unread.holds_what, None, "never read");
    assert_eq!(s.particle_systems["GasCloudSys"].holds_what, None, "never read");
}

/// spec: S-0613 Import mapping ([General] InfantryVirus an animation name,
/// WindDirection an integer; absent: none and −1)
#[test]
fn general_gas_keys() {
    let base = "[Animations]\n1=Fume\n[General]\nInfantryVirus=Fume\nWindDirection=3\n";
    let art = "[Fume]\nSpawnsParticle=Puff\nNumParticles=2\n";
    let (s, w) = sim(base, "", art);
    assert_eq!(s.general.infantry_virus.as_deref(), Some("Fume"));
    assert_eq!(s.general.wind_direction, Some(3));
    assert!(!warned(&w, "InfantryVirus"), "{w:?}");
    let (s, _) = sim("[Animations]\n1=Fume\n", "", "");
    assert_eq!(s.general.infantry_virus, None);
    assert_eq!(s.general.wind_direction, None);
}

/// spec: S-0613 Edge cases (the importer warns for a virus-death animation
/// without a particle type and for a wind direction outside 0–7 while a type
/// has a wind effect) and F6, F8 with S-0205 claim 3 (zero divisors warn)
#[test]
fn gas_warnings() {
    let art = "[Fume]\nNumParticles=2\n";
    let (_, w) = sim(
        "[Animations]\n1=Fume\n[General]\nInfantryVirus=Fume\n",
        "",
        art,
    );
    assert!(warned(&w, "[General] InfantryVirus Fume"), "{w:?}");
    let windy = "[Particles]\n0=Puff\n[Puff]\nWindEffect=2\n";
    let (_, w) = sim(windy, "", "");
    assert!(warned(&w, "[General] WindDirection -1"), "{w:?}");
    let (_, w) = sim(&format!("{windy}[General]\nWindDirection=8\n"), "", "");
    assert!(warned(&w, "[General] WindDirection 8"), "{w:?}");
    let (_, w) = sim(&format!("{windy}[General]\nWindDirection=7\n"), "", "");
    assert!(!warned(&w, "WindDirection"), "{w:?}");
    let (_, w) = sim("[Particles]\n0=Puff\n[Puff]\nWindEffect=0\n", "", "");
    assert!(!warned(&w, "WindDirection"), "no wind effect: no warning");
    let (_, w) = sim(
        "[Particles]\n0=Puff\n1=Gust\n[Puff]\nWindEffect=11\nStateAIAdvance=0\n\
         [Gust]\nStateAIAdvance=255\n[General]\nWindDirection=0\n",
        "",
        "",
    );
    assert!(warned(&w, "particle Puff WindEffect 11"), "{w:?}");
    assert!(warned(&w, "particle Puff: StateAIAdvance 0"), "{w:?}");
    // 255 reduces to −1 as a signed byte.
    assert!(warned(&w, "particle Gust: StateAIAdvance 255"), "{w:?}");
}

/// spec: S-0613 Import mapping (the gas-cloud keys are converted fields)
#[test]
fn gas_coverage() {
    let i = import(
        "[Particles]\n0=Puff\n[Puff]\nMaxDC=3\nWindEffect=1\n[ParticleSystems]\n0=Cloud\n\
         [Cloud]\nLifetime=4\n[General]\nWindDirection=1\nInfantryVirus=Fume\n",
        "",
        "",
    );
    let row = |cat: &str, key: &str| {
        i.coverage
            .iter()
            .find(|r| r.category == cat && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{cat} {key}"))
    };
    use crate::coverage::Outcome::Field;
    assert_eq!(row("particles", "MaxDC").0, Field);
    assert_eq!(row("particles", "WindEffect").0, Field);
    assert_eq!(row("particle_systems", "Lifetime").0, Field);
    assert_eq!(row("global:General", "WindDirection").0, Field);
    assert_eq!(row("global:General", "InfantryVirus").0, Field);
}

/// spec: S-0613 claim 17 (the shared gas system leaves play after a positive
/// lifetime and is not created again, a departure: the importer warns when
/// its lifetime is positive; none for −1 or 0)
#[test]
fn shared_system_positive_lifetime_warns() {
    let with = |l: &str| format!("[ParticleSystems]\n0=GasCloudSys\n[GasCloudSys]\nLifetime={l}\n");
    let (_, w) = sim(&with("3"), "", "");
    assert!(warned(&w, "particle system GasCloudSys Lifetime 3"), "{w:?}");
    for l in ["-1", "0"] {
        let (_, w) = sim(&with(l), "", "");
        assert!(!warned(&w, "particle system GasCloudSys Lifetime"), "{l}: {w:?}");
    }
    // Another system's positive lifetime is ordinary.
    let (_, w) = sim("[ParticleSystems]\n0=Cloud\n[Cloud]\nLifetime=3\n", "", "");
    assert!(!warned(&w, "Lifetime 3"), "{w:?}");
}

/// spec: S-0613 Edge cases (a cell animation with μ = 0 that requests
/// particles is created while the map loads, before the shared gas system
/// exists: the importer warns; none when μ is not 0 or no particle is
/// requested)
#[test]
fn loading_time_particle_request_warns() {
    use crate::art::TypeArt;
    use crate::kinds::Kind;
    use crate::layers::{Layer, Role, apply};
    use glasnost_ini::Document;
    use std::collections::BTreeMap;
    let base = "[Animations]\n0=Seep\n1=Late\n2=Dry\n[Particles]\n0=Puff\n\
                [OverlayTypes]\n0=Vent\n1=Slow\n2=Bare\n\
                [Vent]\nCellAnim=Seep\n[Slow]\nCellAnim=Late\n[Bare]\nCellAnim=Dry\n";
    let art_text = "[Seep]\nSpawnsParticle=Puff\nNumParticles=2\n\
                    [Late]\nSpawnsParticle=Puff\nNumParticles=2\n[Dry]\nNumParticles=2\n";
    let layers = [Layer {
        role: Role::Base,
        file: "rules.ini".into(),
        doc: Document::parse(base.as_bytes(), Default::default(), &mut Vec::new()),
    }];
    let art = Document::parse(art_text.as_bytes(), Default::default(), &mut Vec::new());
    let outcome = apply(&layers, &art, "art.ini", Default::default(), &Default::default());
    // μ: Seep and Dry found no middle sprite (μ = 0); Late's has 8 frames (μ = 4).
    let late = TypeArt {
        middle_sprite: Some(b"late.shp".to_vec()),
        middle_sprite_frames: Some(8),
        ..TypeArt::default()
    };
    let arts = BTreeMap::from([
        ((Kind::Animations, 0), TypeArt::default()),
        ((Kind::Animations, 1), late),
        ((Kind::Animations, 2), TypeArt::default()),
    ]);
    let (_, w) = super::build(super::SimInputs {
        outcome: &outcome,
        art: &art,
        arts: Some(&arts),
        quirks: &Default::default(),
        art_quirks: &Default::default(),
        ini_quirks: &Default::default(),
        sim_quirks: &Default::default(),
    });
    assert!(warned(&w, "overlay Vent CellAnim Seep"), "{w:?}");
    assert!(!warned(&w, "CellAnim Late"), "μ ≠ 0: {w:?}");
    assert!(!warned(&w, "CellAnim Dry"), "no particle type: {w:?}");
}

fn dec(v: &Option<glasnost_rules::sim::Decimal>) -> Option<&str> {
    v.as_ref().map(|d| d.0.as_str())
}

/// spec: S-0616 Import mapping (Velocity, Deacc, FinalDamageState, Normalized, Radius,
/// XVelocity, YVelocity, MinZVelocity and ZVelocityRange of a particle type; SpawnFrames,
/// Slowdown, SpawnCutoff, SpawnRadius, ParticleCap, SparkSpawnFrames, SpawnSparkPercentage,
/// SpawnDirection and ParticlesPerCoord of a particle-system type; DamageSmokeOffset of a unit
/// type; absent keys give no value, so the engine defaults apply)
#[test]
fn particle_system_kind_fields() {
    let base = "[Particles]\n0=Soot\n1=Ember\n[ParticleSystems]\n0=Plume\n1=Burst\n\
                [VehicleTypes]\n1=Hulk\n2=Plain\n\
                [Soot]\nBehavesLike=Smoke\nVelocity=4.5\nDeacc=0.25\nRadius=40\n\
                [Ember]\nBehavesLike=Spark\nFinalDamageState=3\nNormalized=yes\nXVelocity=7\n\
                YVelocity=-6\nMinZVelocity=2\nZVelocityRange=9\n\
                [Plume]\nBehavesLike=Smoke\nHoldsWhat=Soot\nSpawnFrames=3\nSlowdown=0.5\n\
                SpawnCutoff=12.5\nSpawnRadius=20\nParticleCap=6\nParticlesPerCoord=0.25\n\
                [Burst]\nBehavesLike=Spark\nHoldsWhat=Ember\nSparkSpawnFrames=4\n\
                SpawnSparkPercentage=0.75\nSpawnDirection=1.5,-2\n\
                [Hulk]\nDamageSmokeOffset=10,-20,30\n[Plain]\nStrength=5\n";
    let (s, w) = sim(base, "", "");
    let soot = &s.particles["Soot"];
    assert_eq!((dec(&soot.velocity), dec(&soot.deacc)), (Some("45e-1"), Some("25e-2")));
    assert_eq!(soot.radius, Some(40));
    assert_eq!((soot.final_damage_state, soot.normalized), (None, None));
    let ember = &s.particles["Ember"];
    assert_eq!((ember.final_damage_state, ember.normalized), (Some(3), Some(true)));
    assert_eq!(
        (ember.x_velocity, ember.y_velocity, ember.min_z_velocity, ember.z_velocity_range),
        (Some(7), Some(-6), Some(2), Some(9))
    );
    assert_eq!((ember.velocity.as_ref(), ember.radius), (None, None));
    let plume = &s.particle_systems["Plume"];
    assert_eq!(plume.spawn_frames, Some(3));
    assert_eq!((dec(&plume.slowdown), dec(&plume.spawn_cutoff)), (Some("5e-1"), Some("125e-1")));
    assert_eq!((plume.spawn_radius, plume.particle_cap), (Some(20), Some(6)));
    assert_eq!(dec(&plume.particles_per_coord), Some("25e-2"));
    assert_eq!(plume.spawn_direction, None);
    let burst = &s.particle_systems["Burst"];
    assert_eq!(burst.spark_spawn_frames, Some(4));
    assert_eq!(dec(&burst.spawn_spark_percentage), Some("75e-2"));
    // A value with two numbers keeps the value in force, 0, for the third (S-0003 claim 9).
    let dir: Vec<&str> = burst.spawn_direction.iter().flatten().map(|d| d.0.as_str()).collect();
    assert_eq!(dir, ["15e-1", "-2", "0"]);
    assert_eq!(s.vehicles["Hulk"].combat.damage_smoke_offset, Some([10, -20, 30]));
    assert_eq!(s.vehicles["Plain"].combat.damage_smoke_offset, None);
    assert!(!warned(&w, "S-0616 claim 20"), "{w:?}");
}

/// spec: S-0616 claim 20 (the data cases the importer warns for: a smoke type with a next type
/// and a radius below 8, spark velocity fields of 0, a fire type with damage and no warhead, a
/// smoke spawn interval of 0, a negative smoke spawn radius, a fire spawn interval of 0 and a
/// spark particle cap of −1, 0 or 1) and claim 1 (DamSmkOffScrnRel on: used as given, warned)
#[test]
fn particle_system_kind_warnings() {
    let base = "[Particles]\n0=Soot\n1=Ash\n2=Ember\n3=Flame\n\
                [ParticleSystems]\n0=Plume\n1=Burst\n2=Blaze\n\
                [VehicleTypes]\n1=Hulk\n\
                [Soot]\nBehavesLike=Smoke\nNextParticle=Ash\nRadius=7\n[Ash]\nBehavesLike=Smoke\n\
                [Ember]\nBehavesLike=Spark\nXVelocity=0\nZVelocityRange=0\n\
                [Flame]\nBehavesLike=Fire\nDamage=5\n\
                [Plume]\nBehavesLike=Smoke\nSpawnFrames=0\nSpawnRadius=-1\n\
                [Burst]\nBehavesLike=Spark\nParticleCap=1\n\
                [Blaze]\nBehavesLike=Fire\nSpawnFrames=0\n\
                [Hulk]\nDamSmkOffScrnRel=yes\n";
    let (_, w) = sim(base, "", "");
    for text in [
        "particle Soot: a smoke type with a next type and a radius below 8",
        "particle Ember: XVelocity 0",
        "particle Ember: ZVelocityRange 0",
        "particle Flame: a fire type with damage and no warhead",
        "particle system Plume: a smoke spawn interval of 0",
        "particle system Plume: a smoke spawn radius below 0",
        "particle system Burst: a spark particle cap",
        "particle system Blaze: a fire spawn interval of 0",
        "DamSmkOffScrnRel is on",
    ] {
        assert!(warned(&w, text), "{text}: {w:?}");
    }
    assert!(!warned(&w, "particle Ember: YVelocity"), "{w:?}");
}

/// spec: S-0601 Import mapping (DefaultSparkSystem absent: no game-wide spark systems and no
/// overload placement draws; the importer warns) and S-0616 Import mapping
#[test]
fn default_spark_system_absent_warns() {
    let (s, w) = sim("[ParticleSystems]\n0=Fizz\n[Fizz]\nBehavesLike=Spark\n", "", "");
    assert_eq!(s.general.default_spark_system, None);
    assert!(warned(&w, "DefaultSparkSystem is absent"), "{w:?}");
    let (s, w) = sim(
        "[ParticleSystems]\n0=Fizz\n[Fizz]\nBehavesLike=Spark\n[CombatDamage]\nDefaultSparkSystem=Fizz\n",
        "",
        "",
    );
    assert_eq!(s.general.default_spark_system.as_deref(), Some("Fizz"));
    assert!(!warned(&w, "DefaultSparkSystem is absent"), "{w:?}");
}

/// spec: S-0616 F5 and Edge cases (quirks.flame_sweep_facing: absent the original's position,
/// on; an imported ruleset with a patch layer setting it loads either position. The switch is
/// wired for the effect-weapon emitter seam and changes nothing until that seam is filled.)
/// Every other S-0616 switch loads the same way.
#[test]
fn particle_quirks_load_in_both_positions() {
    let base = "[ParticleSystems]\n0=Blaze\n[Blaze]\nBehavesLike=Fire\n";
    let load = |layer: Option<&str>| {
        let i = import(base, "", "");
        let sources: Vec<glasnost_rules::resolve::Source> = layer
            .iter()
            .map(|text| glasnost_rules::resolve::Source {
                layer: "patch",
                file: "patch.jsonc",
                text,
            })
            .collect();
        glasnost_rules::resolve_ruleset(Some((i.tree, i.provenance)), &sources)
            .unwrap_or_else(|e| panic!("{e}"))
            .ruleset
            .globals
            .quirks
    };
    let q = load(None);
    assert!(q.flame_sweep_facing, "absent: on");
    assert!(q.fire_particle_no_house && q.smoke_ignores_particle_cap);
    assert!(q.spark_double_gravity && q.smoke_deck_extra_update);
    for on in [false, true] {
        let text = format!(
            "{{\"globals\": {{\"quirks\": {{\"flame_sweep_facing\": {on}, \"fire_particle_no_house\": {on}, \
             \"smoke_ignores_particle_cap\": {on}, \"spark_double_gravity\": {on}, \"smoke_deck_extra_update\": {on}}}}}}}"
        );
        let q = load(Some(&text));
        assert_eq!(
            (
                q.flame_sweep_facing,
                q.fire_particle_no_house,
                q.smoke_ignores_particle_cap,
                q.spark_double_gravity,
                q.smoke_deck_extra_update
            ),
            (on, on, on, on, on)
        );
    }
}

/// spec: S-0212 claim 13 and Edge cases (quirks.corridor_retry_no_fallback: absent the
/// original's position, on; an imported ruleset with a patch layer setting it loads either
/// position).
#[test]
fn corridor_retry_quirk_loads_in_both_positions() {
    let load = |layer: Option<&str>| {
        let i = import("", "", "");
        let sources: Vec<glasnost_rules::resolve::Source> = layer
            .iter()
            .map(|text| glasnost_rules::resolve::Source {
                layer: "patch",
                file: "patch.jsonc",
                text,
            })
            .collect();
        glasnost_rules::resolve_ruleset(Some((i.tree, i.provenance)), &sources)
            .unwrap_or_else(|e| panic!("{e}"))
            .ruleset
            .globals
            .quirks
    };
    assert!(load(None).corridor_retry_no_fallback, "absent: on");
    for on in [false, true] {
        let text = format!("{{\"globals\": {{\"quirks\": {{\"corridor_retry_no_fallback\": {on}}}}}}}");
        assert_eq!(load(Some(&text)).corridor_retry_no_fallback, on);
    }
}
