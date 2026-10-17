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
