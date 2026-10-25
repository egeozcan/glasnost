//! Tests of the special-hit fields of the `sim` section (S-0615 Import
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

fn resolved(base: &str, map: &str, art: &str) -> (glasnost_rules::schema::Ruleset, Vec<Warning>) {
    let import = import(base, map, art);
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset, import.warnings)
}

fn sim(base: &str, map: &str, art: &str) -> (SimData, Vec<Warning>) {
    let (r, w) = resolved(base, map, art);
    (r.sim, w)
}

fn warned(w: &[Warning], text: &str) -> bool {
    w.iter().any(|w| w.message.contains(text))
}

/// spec: S-0615 Import mapping (FlamingInfantry, InfantryHeadPop,
/// InfantryNuked and InfantryBrute of [General], DiskLaserChargeUp of
/// [AudioVisual]; absent: none) and S-0109 Import mapping (FlameDamage2 of
/// [CombatDamage]; absent: none)
#[test]
fn game_wide_keys_present_and_absent() {
    let base = "[Animations]\n0=Blaze\n1=Pop\n2=Glow\n3=Smash\n[Warheads]\n1=Scorch\n\
                [General]\nFlamingInfantry=Blaze\nInfantryHeadPop=Pop\nInfantryNuked=Glow\n\
                InfantryBrute=Smash\nInfantryExplode=Pop\n\
                [AudioVisual]\nDiskLaserChargeUp=Hum\n[CombatDamage]\nFlameDamage2=Scorch\n";
    let (s, _) = sim(base, "", "");
    let g = &s.general;
    assert_eq!(g.flaming_infantry.as_deref(), Some("Blaze"));
    assert_eq!(g.infantry_head_pop.as_deref(), Some("Pop"));
    assert_eq!(g.infantry_nuked.as_deref(), Some("Glow"));
    assert_eq!(g.infantry_brute.as_deref(), Some("Smash"));
    assert_eq!(g.infantry_explode.as_deref(), Some("Pop"));
    assert_eq!(g.disk_laser_charge_up.as_deref(), Some("Hum"));
    assert_eq!(g.flame_damage2.as_deref(), Some("Scorch"));
    let (s, _) = sim("[General]\nName=x\n", "", "");
    let g = &s.general;
    assert_eq!(g.flaming_infantry, None);
    assert_eq!(g.infantry_head_pop, None);
    assert_eq!(g.infantry_nuked, None);
    assert_eq!(g.infantry_brute, None);
    assert_eq!(g.disk_laser_charge_up, None);
    assert_eq!(g.flame_damage2, None);
}

/// spec: S-0615 Import mapping (LaserDuration, an integer, absent 10 by the
/// simulation's default; LaserInnerColor, LaserOuterColor and
/// LaserOuterSpread, three integer channels each, absent (0, 0, 0);
/// IsHouseColor, a boolean, absent no)
#[test]
fn weapon_laser_keys_present_and_absent() {
    let base = "[VehicleTypes]\n1=Saucer\n[Saucer]\nPrimary=Ring\nSecondary=Plain\n\
                [Ring]\nDiskLaser=yes\nLaserDuration=12\nLaserInnerColor=200,100,50\n\
                LaserOuterColor=10,20,30\nLaserOuterSpread=1,2\nIsHouseColor=yes\n\
                [Plain]\nDamage=1\n";
    let (s, w) = sim(base, "", "");
    let ring = &s.weapons["Ring"];
    assert_eq!(ring.disk_laser, Some(true));
    assert_eq!(ring.laser_duration, Some(12));
    assert_eq!(ring.laser_inner_color, Some([200, 100, 50]));
    assert_eq!(ring.laser_outer_color, Some([10, 20, 30]));
    // One component unread: it keeps the value in force (0), with a warning.
    assert_eq!(ring.laser_outer_spread, Some([1, 2, 0]));
    assert!(warned(&w, "LaserOuterSpread"), "{w:?}");
    assert_eq!(ring.is_house_color, Some(true));
    let plain = &s.weapons["Plain"];
    assert_eq!(plain.disk_laser, None);
    assert_eq!(plain.laser_duration, None);
    assert_eq!(plain.laser_inner_color, None);
    assert_eq!(plain.laser_outer_color, None);
    assert_eq!(plain.laser_outer_spread, None);
    assert_eq!(plain.is_house_color, None);
}

/// spec: S-0615 Import mapping (IsFlamingGuy and RunningFrames of the art
/// data; absent no and 0) and S-0109 Import mapping (Damage, a decimal of
/// the art data); a walker without a positive running-frame count warns
/// (S-0615 Open questions)
#[test]
fn art_walker_keys_present_and_absent() {
    let base = "[Animations]\n0=Blaze\n1=Puff\n2=Odd\n";
    let art = "[Blaze]\nIsFlamingGuy=yes\nRunningFrames=15\nDamage=0.25\n\
               [Puff]\nRate=100\n[Odd]\nIsFlamingGuy=yes\n";
    let (s, w) = sim(base, "", art);
    let blaze = &s.animations["Blaze"];
    assert_eq!(blaze.is_flaming_guy, Some(true));
    assert_eq!(blaze.running_frames, Some(15));
    // The decimal text in the importer's normal form.
    assert_eq!(blaze.damage.as_ref().map(|d| d.0.as_str()), Some("25e-2"));
    let puff = &s.animations["Puff"];
    assert_eq!(puff.is_flaming_guy, None);
    assert_eq!(puff.running_frames, None);
    assert_eq!(puff.damage, None);
    assert!(warned(&w, "animations Odd: a burning walker"), "{w:?}");
    assert!(!warned(&w, "animations Blaze: a burning walker"), "{w:?}");
}

/// spec: S-0615 claim 1 and Edge cases (kind 5 uses the animation type at
/// registration index 1, S-0004 claim 4: registration order, which the
/// importer records; a type first named by a [General] reference after the
/// list is registered after it and does not take index 1)
#[test]
fn registration_index_one_follows_reference_order() {
    let (r, _) = resolved("[Animations]\n0=First\n1=Second\n", "", "");
    assert_eq!(r.registries["animations"][1], "Second");
    let (r, _) = resolved(
        "[General]\nFlamingInfantry=Early\n[Animations]\n0=First\n1=Second\n",
        "",
        "",
    );
    let names = &r.registries["animations"];
    assert_eq!(names[1], "Second", "{names:?}");
    assert_eq!(
        names.iter().position(|n| n == "Early"),
        Some(2),
        "{names:?}"
    );
}

/// spec: S-0109 Import mapping (Q-1717: the built-in invisibility-animation
/// type is found by an exact comparison with its stored name, the spelling
/// it was first registered with, S-0004 claims 2 and 7: a later mention in
/// other letter case finds the same type and keeps that spelling; the
/// `C4Warhead` key feeds S-0109 claim 17 too)
#[test]
fn an_animation_keeps_its_first_registered_spelling() {
    let base =
        "[Animations]\n0=Inviso\n1=Mark\n[CombatDamage]\nC4Warhead=Bang\n[Warheads]\n0=Bang\n";
    let art = "[Mark]\nNext=INVISO\n[Inviso]\nDamage=1\n";
    let (r, _) = resolved(base, "", art);
    let names = &r.registries["animations"];
    assert_eq!(names, &["Inviso", "Mark"], "no second type for INVISO");
    assert_eq!(r.sim.animations["Mark"].next.as_deref(), Some("INVISO"));
    let (r, _) = resolved(
        "[Animations]\n0=INVISO\n1=Mark\n",
        "",
        "[Mark]\nNext=inviso\n[INVISO]\nDamage=1\n",
    );
    assert_eq!(r.registries["animations"], ["INVISO", "Mark"]);
    let i = import(base, "", art);
    let row = i
        .coverage
        .iter()
        .find(|r| r.category == "global:CombatDamage" && r.key == "C4Warhead")
        .expect("C4Warhead is covered");
    assert_eq!(row.outcome, crate::coverage::Outcome::Field);
    assert!(row.spec.contains("S-0109"), "{}", row.spec);
}

/// spec: S-0615 Import mapping (the special-hit keys are converted fields)
#[test]
fn special_hit_coverage() {
    let i = import(
        "[VehicleTypes]\n1=Saucer\n[Saucer]\nPrimary=Ring\n[Ring]\nDiskLaser=yes\n\
         LaserDuration=3\nLaserInnerColor=1,2,3\nLaserOuterColor=1,2,3\nLaserOuterSpread=1,2,3\n\
         IsHouseColor=no\n[Animations]\n0=Blaze\n\
         [General]\nFlamingInfantry=Blaze\nInfantryHeadPop=Blaze\nInfantryNuked=Blaze\n\
         InfantryBrute=Blaze\n[AudioVisual]\nDiskLaserChargeUp=Hum\n\
         [CombatDamage]\nFlameDamage2=Scorch\n[Warheads]\n0=Scorch\n",
        "",
        "[Blaze]\nIsFlamingGuy=yes\nRunningFrames=4\nDamage=1\n",
    );
    let row = |cat: &str, key: &str| {
        i.coverage
            .iter()
            .find(|r| r.category == cat && r.key == key)
            .map(|r| (r.outcome, r.spec.clone()))
            .unwrap_or_else(|| panic!("{cat} {key}"))
    };
    use crate::coverage::Outcome::Field;
    for key in [
        "DiskLaser",
        "LaserDuration",
        "LaserInnerColor",
        "LaserOuterColor",
        "LaserOuterSpread",
        "IsHouseColor",
    ] {
        let (o, spec) = row("weapons", key);
        assert_eq!(o, Field, "{key}");
        assert!(spec.contains("S-0615"), "{key}: {spec}");
    }
    for key in [
        "FlamingInfantry",
        "InfantryHeadPop",
        "InfantryNuked",
        "InfantryBrute",
    ] {
        assert_eq!(row("global:General", key).0, Field, "{key}");
    }
    assert_eq!(row("global:AudioVisual", "DiskLaserChargeUp").0, Field);
    assert_eq!(row("global:CombatDamage", "FlameDamage2").0, Field);
    for key in ["IsFlamingGuy", "RunningFrames", "Damage"] {
        assert_eq!(row("art:animations", key).0, Field, "{key}");
    }
}
