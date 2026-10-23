//! Tests of the voxel-animation (debris piece) fields of the `sim` section
//! (S-0317 Import mapping) on invented data (every name and number here is
//! made up).

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

fn import(base: &str, map: &str) -> crate::Import {
    let inputs = Inputs {
        base: Some(file("rules.ini", base)),
        map: Some(file("test.map", map)),
        art: Some(file("art.ini", "")),
        ..Inputs::default()
    };
    convert(&inputs, &ImportOptions::default())
}

fn sim(base: &str, map: &str) -> (SimData, Vec<Warning>) {
    let import = import(base, map);
    let r = glasnost_rules::resolve_ruleset(Some((import.tree, import.provenance)), &[])
        .unwrap_or_else(|e| panic!("{e}"));
    (r.ruleset.sim, import.warnings)
}

fn warned(w: &[Warning], text: &str) -> bool {
    w.iter().any(|w| w.message.contains(text))
}

/// spec: S-0317 Import mapping (every key of a voxel-animation type section;
/// absent keys give no value, so the Import mapping's defaults apply)
#[test]
fn piece_fields_present_and_absent() {
    let base = "[VoxelAnims]\n1=Chunk\n2=Bare\n3=Rock\n[Animations]\n1=Puff\n2=Bang\n3=Trail\n\
                [Warheads]\n1=Thud\n[ParticleSystems]\n1=Smk\n\
                [Chunk]\nDuration=44\nIsMeteor=yes\nMaxXYVel=7.5\nMinZVel=-3\nMaxZVel=2.25\n\
                Elasticity=0.5\nDamage=12\nDamageRadius=90\nWarhead=Thud\nBounceAnim=Puff\n\
                ExpireAnim=Bang\nTrailerAnim=Trail\nSpawns=Rock\nSpawnCount=3\nIsTiberium=no\n\
                AttachedSystem=Smk\nStartSound=Whoosh\nStopSound=Clunk\n\
                MinAngularVelocity=2\nMaxAngularVelocity=6\n\
                [Bare]\nX=1\n";
    let (s, _) = sim(base, "");
    let c = &s.voxel_animations["Chunk"];
    assert_eq!(c.duration, Some(44));
    assert_eq!(c.is_meteor, Some(true));
    assert_eq!(c.max_xy_vel.as_ref().map(|d| d.0.as_str()), Some("75e-1"));
    assert_eq!(c.min_z_vel.as_ref().map(|d| d.0.as_str()), Some("-3"));
    assert_eq!(c.max_z_vel.as_ref().map(|d| d.0.as_str()), Some("225e-2"));
    assert_eq!(c.elasticity.as_ref().map(|d| d.0.as_str()), Some("5e-1"));
    assert_eq!(c.damage, Some(12));
    assert_eq!(c.damage_radius, Some(90));
    assert_eq!(c.warhead.as_deref(), Some("Thud"));
    assert_eq!(c.bounce_anim.as_deref(), Some("Puff"));
    assert_eq!(c.expire_anim.as_deref(), Some("Bang"));
    assert_eq!(c.trailer_anim.as_deref(), Some("Trail"));
    assert_eq!(c.spawns.as_deref(), Some("Rock"));
    assert_eq!(c.spawn_count, Some(3));
    assert_eq!(c.is_tiberium, Some(false));
    assert_eq!(c.attached_system.as_deref(), Some("Smk"));
    assert_eq!(c.start_sound.as_deref(), Some("Whoosh"));
    assert_eq!(c.min_angular_velocity.as_ref().map(|d| d.0.as_str()), Some("2"));
    assert_eq!(c.max_angular_velocity.as_ref().map(|d| d.0.as_str()), Some("6"));
    let b = &s.voxel_animations["Bare"];
    assert_eq!(b.duration, None);
    assert_eq!(b.warhead, None);
    assert_eq!(b.spawn_count, None);
    // A type with no section read still exists with the defaults (S-0004 claims 6 and 13).
    assert!(s.voxel_animations.contains_key("Rock"));
}

/// spec: S-0317 Import mapping and S-0006 claims 3 and 5 (the angular
/// velocities: a section that omits them resets them to −1; a value of 0
/// keeps the held value; a type that reads no section keeps the engine
/// defaults, left absent here)
#[test]
fn angular_velocities_reset_and_keep() {
    let base = "[VoxelAnims]\n1=Spin\n2=Omit\n3=Zero\n4=Never\n\
                [Spin]\nMinAngularVelocity=2\nMaxAngularVelocity=3\n\
                [Omit]\nDuration=5\n\
                [Zero]\nMinAngularVelocity=0\nMaxAngularVelocity=0.0\n\
                [Never]\n";
    let map = "[Spin]\nMinAngularVelocity=0\nMaxAngularVelocity=0\n";
    let (s, _) = sim(base, map);
    let text = |n: &str, min: bool| {
        let t = &s.voxel_animations[n];
        let d = if min {
            &t.min_angular_velocity
        } else {
            &t.max_angular_velocity
        };
        d.as_ref().map(|d| d.0.clone())
    };
    // The map gives 0: the base rules' values are kept.
    assert_eq!(text("Spin", true).as_deref(), Some("2"));
    assert_eq!(text("Spin", false).as_deref(), Some("3"));
    // A section without the keys: −1 each.
    assert_eq!(text("Omit", true).as_deref(), Some("-1"));
    assert_eq!(text("Omit", false).as_deref(), Some("-1"));
    // Explicit 0 in the only section: the engine defaults stay (absent here).
    assert_eq!(text("Zero", true), None);
    assert_eq!(text("Zero", false), None);
}

/// spec: S-0317 claim 16 (a zero span is a departure: the importer warns
/// for the horizontal, the vertical span of a falling piece and the spin
/// span; a meteor piece's vertical span is not used)
#[test]
fn zero_spans_warn() {
    let base = "[VoxelAnims]\n1=Flat\n2=Rocky\n3=Spun\n4=Fine\n\
                [Flat]\nMaxXYVel=0.25\n\
                [Rocky]\nIsMeteor=yes\nMinZVel=5\nMaxZVel=4\n\
                [Spun]\nMinZVel=5\nMaxZVel=4\nMinAngularVelocity=30\nMaxAngularVelocity=1\n\
                [Fine]\nMaxXYVel=-0.5\n";
    let (_, w) = sim(base, "");
    assert!(warned(&w, "Flat: its horizontal span is 0"));
    assert!(!warned(&w, "Rocky: its vertical span"));
    assert!(warned(&w, "Spun: its vertical span is 0"));
    assert!(warned(&w, "Spun: its spin span is 0"));
    assert!(!warned(&w, "Fine: its horizontal span"));
}

/// spec: S-0317 claim 12 (SpawnCount 2147483647 warns; IsTiberium without a
/// resource overlay at position 127 warns, a departure) and claim 11 (an
/// empty splash list warns)
#[test]
fn spawn_count_and_resource_warnings() {
    let base = "[VoxelAnims]\n1=Meteor\n2=Ore\n\
                [Meteor]\nIsMeteor=yes\nSpawns=Ore\nSpawnCount=2147483647\n\
                [Ore]\nIsTiberium=yes\n";
    let (_, w) = sim(base, "");
    assert!(warned(&w, "Meteor: SpawnCount 2147483647"));
    assert!(warned(&w, "the list has fewer than 128 entries"));
    assert!(warned(&w, "neither does a debris piece that ends in water"));
}

/// spec: S-0317 Import mapping (the voxel-animation keys appear as fields
/// in the coverage table)
#[test]
fn piece_keys_are_covered() {
    let i = import(
        "[VoxelAnims]\n1=Chunk\n[Chunk]\nDuration=3\nElasticity=0.5\nStopSound=Clunk\nMinAngularVelocity=1\n",
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
    for k in ["Duration", "Elasticity", "StopSound", "MinAngularVelocity"] {
        assert_eq!(row("voxel_animations", k).0, Field, "{k}");
    }
}
