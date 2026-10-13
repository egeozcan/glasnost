//! Types created by reference (S-0004 claim 13 and its Import mapping
//! tables, with claims 2, 4, 6 and 8), on invented data: every name and
//! number here is made up. The expected kinds and orders are written out
//! by hand from the spec text, never taken from `refs`.

use glasnost_ini::{Document, Quirks};

use crate::kinds::Kind;
use crate::layers::{self, GameKind, How, Layer, LayerQuirks, Outcome, Role};

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

fn run_with(layers: &[(Role, &str)], art: &str, q: LayerQuirks) -> Outcome {
    let layers: Vec<Layer> = layers.iter().map(|(r, t)| layer(*r, t)).collect();
    layers::apply(&layers, &doc(art), "art.ini", GameKind::Skirmish, &q)
}

fn run(layers: &[(Role, &str)], art: &str) -> Outcome {
    run_with(layers, art, LayerQuirks::default())
}

fn base(text: &str) -> Outcome {
    run(&[(Role::Base, text)], "")
}

fn names(o: &Outcome, kind: Kind) -> Vec<String> {
    o.registries[&kind]
        .iter()
        .map(|t| String::from_utf8_lossy(&t.name).into_owned())
        .collect()
}

fn reads(o: &Outcome, kind: Kind, name: &str) -> Vec<usize> {
    o.registries[&kind]
        .iter()
        .find(|t| t.name == name.as_bytes())
        .unwrap_or_else(|| panic!("{name} not registered"))
        .reads
        .clone()
}

fn warned(o: &Outcome, text: &str) -> bool {
    o.warnings.iter().any(|w| w.message.contains(text))
}

/// Particle systems without the one created at game start (S-0004 claim 3).
fn systems(o: &Outcome) -> Vec<String> {
    let mut n = names(o, Kind::ParticleSystems);
    assert_eq!(n.pop().as_deref(), Some("GasCloudSys"));
    n
}

/// spec: S-0004 claim 13 (the kinds each section can create; creating single and list fields), claim 6 (which of them read their section)
#[test]
fn each_section_creates_its_kinds() {
    let o = base(
        "[Countries]\n1=Land\n[Land]\nVeteranUnits=Vu1,Vu2\n\
         [SuperWeaponTypes]\n1=Sw\n[Sw]\nAuxBuilding=Baux\nWeaponType=Wsw\n\
         [InfantryTypes]\n1=Inf\n[Inf]\nDeathAnims=Ad1,Ad2\nOccupyWeapon=Wocc\nEnslaves=Islave\n\
         [OverlayTypes]\n1=Ov\n[Ov]\nCellAnim=Acell\n\
         [VoxelAnims]\n1=Vxa\n[Vxa]\nSpawns=Vxb\nAttachedSystem=Psv\nWarhead=Hvx\n\
         [Warheads]\n1=Hw\n[Hw]\nParticle=Phw\nDebrisTypes=Vxd\n\
         [CombatDamage]\nScorches=Sm1,Sm2\nDeathWeapon=Wcd\n\
         [Vu1]\nStrength=1\n[Baux]\nStrength=1\n[Islave]\nStrength=1\n[Hvx]\nDamage=1\n[Wcd]\nDamage=1\n",
    );
    assert_eq!(names(&o, Kind::Vehicles), ["Vu1", "Vu2"]);
    assert_eq!(names(&o, Kind::Buildings), ["Baux"]);
    // Superweapons' pass (rank 2), then the infantry pass (rank 7), then the late combat damage.
    assert_eq!(names(&o, Kind::Weapons), ["Wsw", "Wocc", "Wcd"]);
    assert_eq!(names(&o, Kind::Animations), ["Ad1", "Ad2", "Acell"]);
    assert_eq!(names(&o, Kind::Infantry), ["Inf", "Islave"]);
    // The warheads' pass (rank 10) comes before the voxel animations' pass (rank 16).
    assert_eq!(names(&o, Kind::VoxelAnimations), ["Vxa", "Vxd", "Vxb"]);
    assert_eq!(systems(&o), ["Phw", "Psv"]);
    assert_eq!(names(&o, Kind::Warheads), ["Hw", "Hvx"]);
    assert_eq!(names(&o, Kind::Smudges), ["Sm1", "Sm2"]);
    // Named by a country or a superweapon: always read (claim 6).
    assert_eq!(reads(&o, Kind::Vehicles, "Vu1"), [0]);
    assert_eq!(reads(&o, Kind::Buildings, "Baux"), [0]);
    // Infantry named by infantry: created during its own kind's pass, read in it.
    assert_eq!(reads(&o, Kind::Infantry, "Islave"), [0]);
    // A warhead only a voxel animation names, no special-weapon settings: unread.
    assert!(reads(&o, Kind::Warheads, "Hvx").is_empty());
    // A weapon only the combat damage names: unread.
    assert!(reads(&o, Kind::Weapons, "Wcd").is_empty());
    match &o.registries[&Kind::Smudges][0].how {
        How::GameWide { section, key } => {
            assert_eq!((*section, *key), ("CombatDamage", "Scorches"))
        }
        other => panic!("{other:?}"),
    }
}

/// spec: S-0004 claims 6 and 13 (scenarios B2 and B3: weapons and projectiles by reference; a weapon only a projectile names is unread)
#[test]
fn weapons_and_projectiles_by_reference() {
    let text = "[VehicleTypes]\n1=Tank\n[Tank]\nPrimary=Gun\n\
                [Gun]\nProjectile=Shell\nWarhead=Hit\nDamage=1\n\
                [Shell]\nShrapnelWeapon=Frag\n[Frag]\nDamage=9\n[Hit]\nVerses=1\n";
    let o = base(text);
    assert_eq!(names(&o, Kind::Weapons), ["Gun", "Frag"]);
    assert_eq!(names(&o, Kind::Projectiles), ["Shell"]);
    assert_eq!(names(&o, Kind::Warheads), ["Hit"]);
    assert_eq!(reads(&o, Kind::Weapons, "Gun"), [0]);
    assert_eq!(reads(&o, Kind::Projectiles, "Shell"), [0]);
    assert_eq!(reads(&o, Kind::Warheads, "Hit"), [0]);
    assert!(reads(&o, Kind::Weapons, "Frag").is_empty());
    // Control: a vehicle also names it, so it exists before the weapons' pass.
    let o = base(&text.replace("Primary=Gun\n", "Primary=Gun\nSecondary=Frag\n"));
    assert_eq!(names(&o, Kind::Weapons), ["Gun", "Frag"]);
    assert_eq!(reads(&o, Kind::Weapons, "Frag"), [0]);
}

/// spec: S-0004 claim 13 (find-only fields create nothing; scenario I1, B4)
#[test]
fn find_only_fields_create_nothing() {
    let o = base(
        "[BuildingTypes]\n1=Tower\n[Tower]\nPrerequisite=Qqq\nPrerequisiteOverride=Rrr\n\
         Owner=Newland\nRequiredHouses=Otherland\nSuperWeapon=Nosuch\nSuperWeapon2=Nosuch2\n\
         [VoxelAnims]\n1=Vox\n[Vox]\nShareSource=Zzz\n\
         [General]\nPrerequisitePower=Ppp\n",
    );
    assert_eq!(names(&o, Kind::Buildings), ["Tower"]);
    assert!(names(&o, Kind::Countries).is_empty());
    assert!(names(&o, Kind::Superweapons).is_empty());
    for k in [Kind::Infantry, Kind::Vehicles, Kind::Aircraft] {
        assert!(names(&o, k).is_empty());
    }
    assert!(warned(
        &o,
        "SuperWeapon names the unknown superweapon \"Nosuch\""
    ));
    assert!(warned(
        &o,
        "SuperWeapon2 names the unknown superweapon \"Nosuch2\""
    ));
}

/// spec: S-0004 claims 2 and 13 (`none` and `<none>`; list items in order, items meaning no type skipped; scenario I6)
#[test]
fn list_items_in_order_and_reserved_names() {
    let o = base(
        "[BuildingTypes]\n1=Plant\n[Plant]\nDamageParticleSystems=none,Sss,<NONE>,Rrr,sss\n\
         DeploysInto=NONE\nDock=Ba, Bc\n",
    );
    assert_eq!(systems(&o), ["Sss", "Rrr"]);
    // S-0003 claim 12: items are not trimmed; the importer warns.
    assert_eq!(names(&o, Kind::Buildings), ["Plant", "Ba", " Bc"]);
    assert!(warned(&o, "begins or ends with whitespace"));
    // Particle systems come after buildings: read in the same layer.
    assert!(
        reads(&o, Kind::ParticleSystems, "Sss").is_empty(),
        "no section"
    );
}

fn commas(n: usize) -> String {
    ",".repeat(n)
}

/// spec: S-0004 claim 13 (text limits: the 127-byte cut of a creating list field, scenario I7)
#[test]
fn list_field_cut_at_127_bytes() {
    let o = |n: usize| {
        base(&format!(
            "[BuildingTypes]\n1=Plant\n[Plant]\nDamageParticleSystems={}Abcd\n",
            commas(n)
        ))
    };
    // Run 1: the name starts at byte 128.
    assert!(systems(&o(127)).is_empty());
    // Run 2: its first two letters are bytes 126 and 127.
    assert_eq!(systems(&o(125)), ["Ab"]);
    // Run 3: it ends at byte 127.
    assert_eq!(systems(&o(123)), ["Abcd"]);
}

/// spec: S-0004 claims 8 and 13 (the 127-byte cut of a single field, scenario I8)
#[test]
fn single_field_cut_at_127_bytes() {
    let o = |spaces: usize| {
        base(&format!(
            "[VehicleTypes]\n1=Eve\n[BuildingTypes]\n1=Plant\n[Plant]\nFreeUnit=Eve{}X\n",
            " ".repeat(spaces)
        ))
    };
    // Run 1: the letter is byte 129; the cut removes it and the spaces go too.
    assert_eq!(names(&o(125), Kind::Vehicles), ["Eve"]);
    // Run 2: the letter is byte 127; a new vehicle with that (truncated) name.
    let v = names(&o(123), Kind::Vehicles);
    assert_eq!(v.len(), 2);
    assert_eq!(v[1], format!("Eve{}", " ".repeat(21)));
}

/// spec: S-0004 claims 2 and 13 (the 63-byte cut of the emitted particle type, scenario I9)
#[test]
fn emitted_particle_cut_at_63_bytes() {
    let o = |spaces: usize| {
        base(&format!(
            "[Particles]\n1=Pp\n[ParticleSystems]\n1=Sys\n[Sys]\nHoldsWhat=Pp{}Z\n",
            " ".repeat(spaces)
        ))
    };
    // Run 1: the letter is byte 65.
    assert_eq!(names(&o(62), Kind::Particles), ["Pp"]);
    // Run 2: the letter is byte 63: a new particle, created during the particle systems' pass, unread.
    let p = o(60);
    assert_eq!(names(&p, Kind::Particles).len(), 2);
    assert!(p.registries[&Kind::Particles][1].reads.is_empty());
}

/// spec: S-0004 claims 2 and 13 (the 31-byte cut and `none` of the next particle type; the spawned particle type of an animation)
#[test]
fn next_and_spawned_particles() {
    let text = |spaces: usize| {
        format!(
            "[Particles]\n1=Pa\n2=Pb\n3=Pc\n[Pa]\nNextParticle=none\n[Pb]\nNextParticle=<NONE>\n\
             [Pc]\nNextParticle=Pa{}Q\nWarhead=Hp\n[Hp]\nDamage=1\n",
            " ".repeat(spaces)
        )
    };
    // The letter at byte 33: cut away; the value finds Pa.
    let o = base(&text(30));
    assert_eq!(names(&o, Kind::Particles), ["Pa", "Pb", "Pc", "none"]);
    // The letter at byte 31: a new particle.
    let o = base(&text(28));
    assert_eq!(names(&o, Kind::Particles).len(), 5);
    // A warhead that only a particle type names is not read without the special-weapon settings.
    assert!(reads(&o, Kind::Warheads, "Hp").is_empty());
    // quirks.particle_none_is_a_name off: `none` names no type.
    let q = LayerQuirks {
        particle_none_is_a_name: false,
        ..LayerQuirks::default()
    };
    let o = run_with(&[(Role::Base, &text(30))], "", q);
    assert_eq!(names(&o, Kind::Particles), ["Pa", "Pb", "Pc"]);
    // The spawned particle type, read from the art file.
    let o = run(
        &[(Role::Base, "[Animations]\n1=Boom\n")],
        "[Boom]\nSpawnsParticle=none\n",
    );
    assert_eq!(names(&o, Kind::Particles), ["none"]);
    let o = run(
        &[(Role::Base, "[Animations]\n1=Boom\n")],
        "[Boom]\nSpawnsParticle=<none>\n",
    );
    assert!(names(&o, Kind::Particles).is_empty());
}

/// spec: S-0004 claims 2, 8 and 13 (the attached particle system: `none`, `<none>`, the 19-byte limit; scenarios I4 and I5)
#[test]
fn attached_particle_system() {
    let text = "[ParticleSystems]\n1=Abcdefghijklmnopqrstuv\n\
                [VehicleTypes]\n1=Tank\n[Tank]\nPrimary=G1\nSecondary=G2\nElitePrimary=G3\n\
                [G1]\nAttachedParticleSystem=none\n[G2]\nAttachedParticleSystem=<none>\n\
                [G3]\nAttachedParticleSystem=Abcdefghijklmnopqrstuv\n";
    let o = base(text);
    assert_eq!(
        systems(&o),
        ["Abcdefghijklmnopqrstuv", "none", "Abcdefghijklmnopqrs"]
    );
    // spec: S-0004 Edge cases (quirks.attached_system_none_keeps: `<none>` has no effect; the importer warns)
    assert!(warned(&o, "keeps the attached particle system"));
    // quirks.long_name_duplicates off: the field reads 24 bytes and finds the listed type.
    let q = LayerQuirks {
        long_name_duplicates: false,
        ..LayerQuirks::default()
    };
    let o = run_with(&[(Role::Base, text)], "", q);
    assert_eq!(systems(&o), ["Abcdefghijklmnopqrstuv", "none"]);
}

fn weapons_of(text: &str) -> Vec<String> {
    names(&base(text), Kind::Weapons)
}

/// spec: S-0004 claim 13 (weapon slots: which fields take effect and the index order of the weapons they create), claim 4 (within one section)
#[test]
fn weapon_slot_fields() {
    // Turret count below 1, flag not set: the four standard fields, in their fixed order.
    let w = weapons_of(
        "[VehicleTypes]\n1=Va\n[Va]\nEliteSecondary=W4\nElitePrimary=W3\nSecondary=W2\n\
         Primary=W1\nWeapon1=Wx\nDeathWeapon=Wd\n",
    );
    let at = |n: &str| w.iter().position(|x| x == n);
    assert!(at("W1") < at("W2") && at("W2") < at("W3") && at("W3") < at("W4"));
    assert!(at("W1").is_some() && at("Wd").is_some() && at("Wx").is_none());
    // Turret count below 1, flag set: none.
    assert!(weapons_of("[VehicleTypes]\n1=Vb\n[Vb]\nClearAllWeapons=yes\nPrimary=P1\n").is_empty());
    // Turret count 1 or more: numbered fields 1 to c, weapon then elite weapon, flag or not.
    assert_eq!(
        weapons_of(
            "[VehicleTypes]\n1=Vc\n[Vc]\nWeapon2=N2\nPrimary=Px\nEliteWeapon1=E1\nWeapon3=N3\n\
             TurretCount=1\nWeaponCount=2\nWeapon1=N1\nClearAllWeapons=yes\n"
        ),
        ["N1", "E1", "N2"]
    );
    // A weapon count of 1 or more and no numbered field of that form.
    assert_eq!(
        weapons_of(
            "[BuildingTypes]\n1=Vd\n[Vd]\nTurretCount=2\nWeaponCount=2147483647\nWeapon7=N7\nWeapon07=Bad\n"
        ),
        ["N7"]
    );
    // Weapon count below 1: no field at all.
    assert!(
        weapons_of("[AircraftTypes]\n1=Ve\n[Ve]\nTurretCount=1\nWeapon1=N1\nPrimary=P\n")
            .is_empty()
    );
    // Infantry have the same fields, plus the occupy weapons.
    assert_eq!(
        weapons_of("[InfantryTypes]\n1=Guy\n[Guy]\nPrimary=Pistol\nOccupyWeapon=Rifle\n").len(),
        2
    );
}

/// spec: S-0004 claim 13 (the deciding fields act with the values the type holds once the section has given them; scenario I10 runs a, b and d)
#[test]
fn weapon_slot_values_held_and_given() {
    let two = |b: &str, m: &str| names(&run(&[(Role::Base, b), (Role::Map, m)], ""), Kind::Weapons);
    // Run a: the map's own turret count decides, wherever it sits.
    assert_eq!(
        two(
            "[VehicleTypes]\n1=Va\n[Va]\nPrimary=Pa\n",
            "[Va]\nWeapon1=Wn\nTurretCount=1\nWeaponCount=1\n"
        ),
        ["Pa", "Wn"]
    );
    // Run b: a turret count of 0 below the primary field.
    assert_eq!(
        two(
            "[VehicleTypes]\n1=Vb\n[Vb]\nTurretCount=1\nWeaponCount=1\nWeapon1=Nb\n",
            "[Vb]\nPrimary=Wb\nTurretCount=0\n"
        ),
        ["Nb", "Wb"]
    );
    // Run d: a flag held from the base rules keeps the primary field from taking effect.
    assert!(
        two(
            "[VehicleTypes]\n1=Vd\n[Vd]\nClearAllWeapons=yes\n",
            "[Vd]\nPrimary=Wd\n"
        )
        .is_empty()
    );
    assert_eq!(
        two(
            "[VehicleTypes]\n1=Vd\n[Vd]\nStrength=1\n",
            "[Vd]\nPrimary=Wd\n"
        ),
        ["Wd"]
    );
}

/// spec: S-0004 claim 4 (rows 1 to 6 within one layer; row 2: AI settings before general settings)
#[test]
fn rows_of_one_layer() {
    let o = run(
        &[(
            Role::Base,
            "[Animations]\n1=Alist\n[General]\nWake=Agen\nShipyard=Bgen\n[AudioVisual]\nOnFire=Alate\n\
             [Warheads]\n1=Hlist\n[Hlist]\nAnimList=Apass\n\
             [SpecialWeapons]\nNukeWarhead=Hspec\n[Hspec]\nAnimList=Asecond\n\
             [Tiberiums]\n0=Ore\n[Ore]\nDebris=Atib\n\
             [AI]\nBuildConst=Bai\n[BuildingTypes]\n1=Blist\n[Bgen]\nStrength=1\n",
        )],
        "[Agen]\nRate=1\n[Apass]\nRate=1\n[Alate]\nRate=1\n",
    );
    assert_eq!(
        names(&o, Kind::Animations),
        ["Alist", "Agen", "Apass", "Alate", "Asecond", "Atib"]
    );
    assert_eq!(names(&o, Kind::Buildings), ["Blist", "Bai", "Bgen"]);
    // Row 2 reads its section in the layer; rows 3 (after the animations' pass) and 4 do not.
    assert_eq!(reads(&o, Kind::Animations, "Agen"), [0]);
    assert_eq!(reads(&o, Kind::Buildings, "Bgen"), [0]);
    assert!(reads(&o, Kind::Animations, "Apass").is_empty());
    assert!(reads(&o, Kind::Animations, "Alate").is_empty());
    // A warhead first named by the special weapons is read in the second reading (claim 6).
    assert_eq!(reads(&o, Kind::Warheads, "Hspec"), [0]);
}

/// spec: S-0004 claims 4 and 13 (the crate overlays: wood crate, crate, water crate, whatever the file order; scenario G2)
#[test]
fn crate_overlays_in_fixed_order() {
    let o = base(
        "[CrateRules]\nWaterCrateImg=Y3\nCrateImg=Y2\nWoodCrateImg=Y1\nUnitCrateType=Vc\n[Vc]\nStrength=1\n",
    );
    assert_eq!(names(&o, Kind::Overlays), ["Y1", "Y2", "Y3"]);
    // A vehicle only the crate rules name is not read (claim 6).
    assert!(reads(&o, Kind::Vehicles, "Vc").is_empty());
    // Switch off: it reads the creating layer.
    let q = LayerQuirks {
        unlisted_reference_unread: false,
        ..LayerQuirks::default()
    };
    let o = run_with(
        &[(
            Role::Base,
            "[CrateRules]\nUnitCrateType=Vc\n[Vc]\nStrength=1\n",
        )],
        "",
        q,
    );
    assert_eq!(reads(&o, Kind::Vehicles, "Vc"), [0]);
}

/// spec: S-0004 claims 4 and 13 (fields read from the art file: the convert-to-overlay field and the animation's spawned overlay; scenarios G3 and G4)
#[test]
fn overlays_from_the_art_file() {
    let rules = "[Animations]\n1=Ann\n[BuildingTypes]\n1=Bb\n2=Ba\n3=Bz\n4=Bi\n\
                 [Bb]\nStrength=1\n[Ba]\nStrength=1\n[Bi]\nImage=Bpic\n\
                 [Ow1]\nX=1\n[Ow2]\nX=1\n[Ow3]\nX=1\n";
    let art = "[Ba]\nToOverlay=Ow3\n[Ann]\nTiberiumSpawnType=Ow1\n[Bb]\nToOverlay=Ow2\n\
               [Bz]\nToOverlay=Ow9\n[Bpic]\nToOverlay=Ow4\n";
    let o = run(&[(Role::Base, rules)], art);
    // The animations' pass first, then buildings by index; Bz has no rules section, so its art field is not read.
    assert_eq!(names(&o, Kind::Overlays), ["Ow1", "Ow2", "Ow3", "Ow4"]);
    for n in ["Ow1", "Ow2", "Ow3"] {
        assert_eq!(reads(&o, Kind::Overlays, n), [0], "{n}");
    }
    // Row 3 between rows 1 and 4 (G3).
    let o = run(
        &[(
            Role::Base,
            "[CrateRules]\nCrateImg=Yc\n[OverlayTypes]\n1=X1\n[BuildingTypes]\n1=Bb\n[Bb]\nStrength=1\n",
        )],
        "[Bb]\nToOverlay=Wo\n",
    );
    assert_eq!(names(&o, Kind::Overlays), ["X1", "Wo", "Yc"]);
}

/// spec: S-0004 claims 6 and 13 (an animation created during the weapons' pass reads its art in the next layer; scenario J2)
#[test]
fn animation_art_in_the_next_layer() {
    let rules = "[VehicleTypes]\n1=Tank\n[Tank]\nPrimary=Gun\n[Gun]\nAnim=An\n";
    let art = "[An]\nRate=1\nWarhead=Hn\n";
    // Setup 1: created in the base rules, read in the next layer.
    let o = run(&[(Role::Base, rules), (Role::Map, "")], art);
    assert_eq!(reads(&o, Kind::Animations, "An"), [1]);
    // The warhead its art names is created in the layer that reads it, not the art database's.
    let h = &o.registries[&Kind::Warheads][0];
    assert_eq!((h.name.as_slice(), h.created_in), (&b"Hn"[..], 1));
    // Setup 2: created in the last layer: never read.
    let o = run(&[(Role::Base, ""), (Role::Map, rules)], art);
    assert!(reads(&o, Kind::Animations, "An").is_empty());
    // Setup 3: listed, so read in the layer.
    let o = run(
        &[
            (Role::Base, ""),
            (Role::Map, &format!("{rules}[Animations]\n1=An\n")),
        ],
        art,
    );
    assert_eq!(reads(&o, Kind::Animations, "An"), [1]);
}

/// spec: S-0004 claim 6 (the second warhead reading needs the special-weapon settings section; scenario J1)
#[test]
fn second_warhead_reading() {
    let rules = "[Radiation]\nRadSiteWarhead=Nw\n[Nw]\nVerses=0%\n";
    let o = base(rules);
    assert!(reads(&o, Kind::Warheads, "Nw").is_empty());
    let o = base(&format!("{rules}[SpecialWeapons]\nNukeDown=Pd\n"));
    assert_eq!(reads(&o, Kind::Warheads, "Nw"), [0]);
    assert_eq!(names(&o, Kind::Projectiles), ["Pd"]);
    assert!(reads(&o, Kind::Projectiles, "Pd").is_empty());
}

/// spec: S-0004 Edge cases (the audio-visual Smoke key feeds two creating fields), claim 8
#[test]
fn smoke_feeds_two_fields() {
    let o = base("[AudioVisual]\nSmoke=Puffy\n");
    assert_eq!(names(&o, Kind::Animations), ["Puffy"]);
    let o = base("[AudioVisual]\nSmoke=Abcdefghijklmnopqrstuvwxyz\n");
    assert_eq!(names(&o, Kind::Animations).len(), 2);
    let q = LayerQuirks {
        long_name_duplicates: false,
        ..LayerQuirks::default()
    };
    let o = run_with(
        &[(
            Role::Base,
            "[AudioVisual]\nSmoke=Abcdefghijklmnopqrstuvwxyz\n",
        )],
        "",
        q,
    );
    assert_eq!(names(&o, Kind::Animations).len(), 1);
}

/// spec: S-0004 claim 3 (the fixed particle system is created at game start when missing, and reads no section)
#[test]
fn game_start_particle_system() {
    let o = base("[GasCloudSys]\nLifetime=3\n");
    let t = &o.registries[&Kind::ParticleSystems][0];
    assert_eq!(t.name, b"GasCloudSys");
    assert_eq!(t.how, How::GameStart);
    assert!(t.reads.is_empty());
    let o = base("[ParticleSystems]\n1=gascloudsys\n");
    assert_eq!(names(&o, Kind::ParticleSystems), ["gascloudsys"]);
}

/// spec: S-0004 claim 8 (a long name that names its own kind would create types without end; our departure stops it, Q-0083)
#[test]
fn self_naming_long_names_stop() {
    let long24 = "Abcdefghijklmnopqrstuvwx";
    let o = run(
        &[(Role::Base, &format!("[Animations]\n1={long24}\n"))],
        &format!("[{long24}]\nNext={long24}yz\n"),
    );
    assert_eq!(names(&o, Kind::Animations), [long24, long24]);
    assert!(warned(&o, "Q-0083"));
    // A long name met once more in another layer creates one more copy there (claim 8).
    let o = run(
        &[
            (
                Role::Base,
                &format!("[VehicleTypes]\n1=Tank\n[Tank]\nUnloadingClass={long24}yz\n"),
            ),
            (
                Role::Map,
                &format!("[{long24}]\nUnloadingClass={long24}yz\n"),
            ),
        ],
        "",
    );
    assert_eq!(names(&o, Kind::Vehicles).len(), 3);
}

/// spec: S-0308 claim 16 (numbered references above 18 register weapons up to the weapon count, normal before elite at each number and lower numbers first; a weapon so created reads its section and creates its projectile)
#[test]
fn numbered_weapons_above_eighteen_register() {
    let text = "[VehicleTypes]\n1=Vf\n[Vf]\nTurretCount=1\nWeaponCount=20\nEliteWeapon20=E20\n\
                Weapon20=N20\nWeapon19=N19\nWeapon21=N21\n[N20]\nProjectile=Far\n";
    assert_eq!(weapons_of(text), ["N19", "N20", "E20"]);
    let o = run(&[(Role::Base, text)], "");
    assert!(names(&o, Kind::Projectiles).iter().any(|x| x == "Far"));
}
