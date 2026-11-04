//! Tests of the game's sound events on synthetic worlds, entries and
//! samples (S-0701 claims 4, 6, 16 to 24 and 26 to 29).

use super::*;
use crate::catalog::SoundEntry;
use crate::output::NullOutput;
use crate::speech::Line;
use glasnost_fmt::pcm::Pcm;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::testkit;
use glasnost_sim::world::Target;

const RATE: u32 = 8000;
const ME: HouseId = HouseId(0);
const SIZE: i32 = 30;

fn centre(_: i32, _: i32, _: i32) -> (i32, i32) {
    (400, 300)
}

/// A view that sees every position at its centre.
fn view() -> View {
    View {
        origin: (0, 0),
        size: (800, 600),
        project: centre,
    }
}

/// The synthetic combat rules with an infantry factory `BARR` and a
/// negative-damage weapon on `BUGGY`; `CART` is an unarmed harvester.
fn ruleset() -> Ruleset {
    use glasnost_rules::sim::{BuildingType, Decimal, WeaponType};
    let mut r = testkit::combat_ruleset();
    r.registries
        .get_mut("buildings")
        .expect("buildings")
        .push("BARR".into());
    r.sim.buildings.insert(
        "BARR".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([1, 1]),
            factory: Some("infantry".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Mend".into(),
        WeaponType {
            damage: Some(-20),
            rof: Some(40),
            range: Some(Decimal("5".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    if let Some(v) = r.sim.vehicles.get_mut("BUGGY") {
        v.primary = Some("Mend".into());
    }
    if let Some(v) = r.sim.vehicles.get_mut("CART") {
        v.harvester = Some(true);
        v.primary = None;
    }
    r
}

fn world() -> World {
    testkit::game_with(&ruleset(), SIZE, 5)
}

fn spawn(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

fn index(w: &World, kind: Kind, name: &str) -> u32 {
    w.rules().index_of(kind, name).expect("type")
}

fn tref(w: &World, kind: Kind, name: &str) -> TypeRef {
    TypeRef {
        kind,
        index: index(w, kind, name),
    }
}

/// Audio for house 0 of `w` with one entry and one sample of four frames
/// per name.
fn audio_with(w: &World, names: &[&str], q: Quirks) -> Audio {
    let mut a = Audio::new(Box::new(NullOutput::new(RATE, "test")));
    a.set_quirks(q);
    let mut bank = SoundBank::new();
    let mut cat = SoundCatalog::new();
    for n in names {
        bank.add_pcm(
            n,
            Pcm {
                sample_rate: RATE,
                channels: 1,
                samples: vec![1000; 4],
            },
        );
        cat.insert(SoundEntry::new(n, &[n]));
    }
    a.set_bank(bank);
    a.set_catalog(cat);
    a.start(w, Some(ME));
    a.set_view(view());
    a
}

fn audio(w: &World, names: &[&str]) -> Audio {
    audio_with(w, names, Quirks::default())
}

/// The entries of every live instance after a pass, in channel order.
fn heard(a: &mut Audio, now: i64) -> Vec<String> {
    a.service(now, &mut Rng::new(1));
    let on: Vec<EntryId> = a.mixer().channel_entries().into_iter().flatten().collect();
    on.iter()
        .map(|e| a.catalog().entry(*e).name.clone())
        .collect()
}

fn render(a: &mut Audio, frames: usize) {
    let mut b = vec![0i16; frames * 2];
    a.mixer_mut().render(&mut b);
}

fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| (*s).to_owned()).collect()
}

fn mv(actor: ObjectId) -> Order {
    Order::Behaviour {
        actor,
        behaviour: Behaviour::Move,
        attack_move: false,
        target: None,
        dest: Some(Target::Cell(testkit::mid(SIZE, 3, 3))),
    }
}

fn click(orders: Vec<Order>) -> ClickResult {
    ClickResult {
        orders,
        selection: None,
        acknowledge: None,
        marker: false,
    }
}

// spec: S-0701 claim 4 (quirks.missing_sample_silences_entry: one missing sample silences the whole entry; off: it is left out)
#[test]
fn missing_sample_silences_entry() {
    let w = world();
    for quirk in [true, false] {
        let q = Quirks {
            missing_sample_silences_entry: quirk,
            ..Quirks::default()
        };
        let mut a = audio_with(&w, &["Here"], q);
        let mut cat = SoundCatalog::new();
        let e = cat.insert(SoundEntry::new("Both", &["Here", "Gone"]));
        a.set_catalog(cat);
        assert_eq!(a.prepare(e).is_none(), quirk);
        if !quirk {
            assert_eq!(a.prepare(e).map(|p| p.clips.len()), Some(1));
        }
    }
}

// spec: S-0701 Edge cases (quirks.attack_decay_without_body: an attack or decay entry with no body sample is silent)
#[test]
fn attack_decay_without_body_is_silent() {
    let w = world();
    let mut a = audio(&w, &["A"]);
    let mut cat = SoundCatalog::new();
    let mut e = SoundEntry::new("Rise", &["A"]);
    e.attack = 1;
    e.control.attack = true;
    let e = cat.insert(e);
    a.set_catalog(cat);
    assert!(a.prepare(e).is_none());
}

// spec: S-0701 claim 6 and Edge cases (quirks.shroud_sound_corner_cell: a shroud sound at cell (0, 0) is never heard; unexplored cells are silent)
#[test]
fn shroud_sounds() {
    let w = world();
    let mut a = audio(&w, &["Hush"]);
    let mut cat = SoundCatalog::new();
    let mut e = SoundEntry::new("Hush", &["Hush"]);
    e.placement.shroud = true;
    let e = cat.insert(e);
    a.set_catalog(cat);
    let corner = Pos::new(128, 128, 0);
    assert_eq!(corner.cell().x, 0);
    assert!(a.request(Some(&w), e, Where::Fixed(corner), FULL).is_none());
    let mut a2 = audio_with(
        &w,
        &["Hush"],
        Quirks {
            shroud_sound_corner_cell: false,
            ..Quirks::default()
        },
    );
    let mut cat = SoundCatalog::new();
    let mut e = SoundEntry::new("Hush", &["Hush"]);
    e.placement.shroud = true;
    let e = cat.insert(e);
    a2.set_catalog(cat);
    let explored = glasnost_sim::econ::shroud::explored(&w, ME, corner.cell());
    assert_eq!(
        a2.request(Some(&w), e, Where::Fixed(corner), FULL)
            .is_some(),
        explored
    );
    // A screen sound is never shrouded.
    assert!(a.request(Some(&w), e, Where::Screen, FULL).is_some());
}

/// A world with a local `CRUSHER` and the response names of `t`.
fn speaker(t: TypeSounds) -> (World, ObjectId, Audio) {
    let mut w = world();
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
    let mut a = audio(
        &w,
        &[
            "Yes0", "Yes1", "Yes2", "Go", "Hit", "Special", "In", "Take", "Dig", "Up", "Mend",
            "Second",
        ],
    );
    let mut n = SoundNames::default();
    n.set_type(Kind::Vehicle, index(&w, Kind::Vehicle, "CRUSHER"), t);
    n.voice_ifv_repair = Some("Mend".into());
    a.set_names(n);
    (w, tank, a)
}

// spec: S-0701 claim 16 (a waiting response is spoken as a screen sound if not speaking; the same entry is dropped; a different one waits)
#[test]
fn response_slot() {
    let (w, tank, mut a) = speaker(TypeSounds {
        voice_select: strings(&["Yes0"]),
        voice_move: strings(&["Go"]),
        ..TypeSounds::default()
    });
    let mut u = Rng::new(3);
    a.selected(&w, tank, &mut u);
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 0), ["Yes0"]);
    assert_eq!(
        a.mixer().placement(InstanceId(0)),
        Some((FULL, crate::mixer::CENTRE))
    );
    // The same entry again while it is spoken: dropped, not restarted.
    a.selected(&w, tank, &mut u);
    a.tick(&w, &[], &mut u);
    assert_eq!(a.mixer().alive(), 1);
    render(&mut a, 100);
    a.tick(&w, &[], &mut u);
    assert_eq!(a.mixer().alive(), 0);
    // A different one waits for the current one.
    a.selected(&w, tank, &mut u);
    a.tick(&w, &[], &mut u);
    heard(&mut a, 100);
    a.ordered(&w, &[tank], &click(vec![mv(tank)]), &mut u);
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 200), ["Yes0"]);
    render(&mut a, 100);
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 300), ["Go"]);
}

// spec: S-0701 claim 16 (only the local player's objects respond); claim 17 (one raw value, entry x mod n) and 31 (the draw is taken first)
#[test]
fn selection_response_draw() {
    let (mut w, tank, mut a) = speaker(TypeSounds {
        voice_select: strings(&["Yes0", "Yes1", "Yes2"]),
        ..TypeSounds::default()
    });
    let theirs = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, 2, 0);
    let mut u = Rng::new(11);
    let mut r = u.clone();
    a.selected(&w, tank, &mut u);
    let x = r.raw();
    assert_eq!(u, r);
    a.tick(&w, &[], &mut u);
    let want = format!("Yes{}", x % 3);
    assert_eq!(heard(&mut a, 0), [want]);
    a.mixer_mut().clear();
    // spec: S-0701 claim 17 and scenario D3 (an ordinary other house's object: no draw, no response)
    assert!(!glasnost_sim::econ::setup::passive(
        &w,
        w.obj(theirs).unwrap().owner.unwrap()
    ));
    a.selected(&w, theirs, &mut u);
    assert_eq!(u, r);
    a.tick(&w, &[], &mut u);
    assert!(heard(&mut a, 100).is_empty());
}

// spec: S-0701 claim 18 (the responder: the most recently selected positive-primary-damage object, else the earliest; no other responds)
#[test]
fn order_responder() {
    let mut w = world();
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, 0, 0);
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, 1, 0);
    let tank2 = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, 2, 0);
    let buggy = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, 3, 0);
    let mut a = audio(&w, &["CartGo", "TankGo", "BuggyGo"]);
    let mut n = SoundNames::default();
    for (t, s) in [
        ("CART", "CartGo"),
        ("CRUSHER", "TankGo"),
        ("BUGGY", "BuggyGo"),
    ] {
        n.set_type(
            Kind::Vehicle,
            index(&w, Kind::Vehicle, t),
            TypeSounds {
                voice_move: strings(&[s]),
                ..TypeSounds::default()
            },
        );
    }
    a.set_names(n);
    let mut u = Rng::new(1);
    let all = |s: &[ObjectId]| click(s.iter().map(|&x| mv(x)).collect());
    let mut said = |a: &mut Audio, sel: &[ObjectId], c: ClickResult, now: i64| {
        a.mixer_mut().clear();
        a.ordered(&w, sel, &c, &mut u);
        a.tick(&w, &[], &mut u);
        heard(a, now)
    };
    // spec: S-0701 claim 18 (the selection order from selection times)
    assert_eq!(
        said(&mut a, &[cart, tank, buggy], all(&[cart, tank, buggy]), 0),
        ["TankGo"]
    );
    assert_eq!(
        said(&mut a, &[tank, cart, tank2], all(&[tank, cart, tank2]), 100),
        ["TankGo"]
    );
    assert_eq!(
        said(&mut a, &[cart, buggy], all(&[cart, buggy]), 200),
        ["CartGo"]
    );
    // The responder got no order: silence.
    let none: Vec<String> = Vec::new();
    assert_eq!(said(&mut a, &[tank, cart], all(&[cart]), 300), none);
}

// spec: S-0701 F6 (the response of each order kind and its fallbacks; a single response takes no draw)
#[test]
fn order_response_table() {
    let full = TypeSounds {
        voice_move: strings(&["Go"]),
        voice_attack: strings(&["Hit"]),
        voice_special_attack: strings(&["Special"]),
        voice_enter: Some("In".into()),
        voice_capture: Some("Take".into()),
        voice_harvest: Some("Dig".into()),
        voice_deploy: Some("Up".into()),
        voice_secondary_attack: Some("Second".into()),
        ..TypeSounds::default()
    };
    let (mut w, tank, mut a) = speaker(full.clone());
    let other = spawn(&mut w, Kind::Vehicle, "CART", 1, 2, 0);
    let mut u = Rng::new(1);
    let b = |behaviour: Behaviour, attack_move: bool| Order::Behaviour {
        actor: tank,
        behaviour,
        attack_move,
        target: Some(Target::Object(other)),
        dest: None,
    };
    let mut now = 0;
    let mut say = |a: &mut Audio, w: &World, o: Order, u: &mut Rng| {
        a.mixer_mut().clear();
        a.ordered(w, &[tank], &click(vec![o]), u);
        a.tick(w, &[], u);
        now += 100;
        heard(a, now)
    };
    assert_eq!(say(&mut a, &w, b(Behaviour::Move, false), &mut u), ["Go"]);
    assert_eq!(say(&mut a, &w, b(Behaviour::Attack, true), &mut u), ["Go"]);
    // spec: S-0701 F6 (weapon 0 chosen: not the secondary-weapon response; no primary response: the attack list)
    assert_eq!(
        say(&mut a, &w, b(Behaviour::Attack, false), &mut u),
        ["Hit"]
    );
    let before = u.taken();
    assert_eq!(say(&mut a, &w, b(Behaviour::Enter, false), &mut u), ["In"]);
    assert_eq!(
        say(&mut a, &w, b(Behaviour::Capture, false), &mut u),
        ["Take"]
    );
    assert_eq!(
        say(&mut a, &w, b(Behaviour::Harvest, false), &mut u),
        ["Dig"]
    );
    assert_eq!(
        say(&mut a, &w, Order::Deploy { actor: tank }, &mut u),
        ["Up"]
    );
    assert_eq!(say(&mut a, &w, b(Behaviour::Unload, false), &mut u), ["Up"]);
    assert_eq!(u.taken(), before);
    assert_eq!(
        say(&mut a, &w, Order::Stop { actor: tank }, &mut u),
        ["Special"]
    );
    assert_eq!(
        say(&mut a, &w, b(Behaviour::Hunt, false), &mut u),
        ["Special"]
    );
    // Fallbacks: capture to enter to move; enter and harvest to move.
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(&w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            voice_enter: None,
            voice_capture: None,
            voice_harvest: None,
            voice_deploy: None,
            ..full.clone()
        },
    );
    a.set_names(n);
    assert_eq!(
        say(&mut a, &w, b(Behaviour::Capture, false), &mut u),
        ["Go"]
    );
    assert_eq!(say(&mut a, &w, b(Behaviour::Enter, false), &mut u), ["Go"]);
    assert_eq!(
        say(&mut a, &w, b(Behaviour::Harvest, false), &mut u),
        ["Go"]
    );
    let none: Vec<String> = Vec::new();
    assert_eq!(say(&mut a, &w, Order::Deploy { actor: tank }, &mut u), none);
    // spec: S-0701 F6 and Import mapping (`Voice` + `PrimaryWeaponAttack`: weapon 0, not elite, the single response before the attack list)
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(&w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            voice_primary_attack: Some("Mend".into()),
            ..full.clone()
        },
    );
    a.set_names(n);
    let before = u.taken();
    assert_eq!(
        say(&mut a, &w, b(Behaviour::Attack, false), &mut u),
        ["Mend"]
    );
    assert_eq!(u.taken(), before, "a single response takes no draw");
}

// spec: S-0701 F6 (attack by the repair-response vehicle with a weapon of negative damage: the game-wide repair-vehicle response)
#[test]
fn repair_vehicle_response() {
    let mut w = world();
    let buggy = spawn(&mut w, Kind::Vehicle, "BUGGY", 0, 0, 0);
    let cart = spawn(&mut w, Kind::Vehicle, "CART", 0, 1, 0);
    let mut a = audio(&w, &["Hit", "Mend"]);
    let mut n = SoundNames::default();
    let t = TypeSounds {
        voice_attack: strings(&["Hit"]),
        repair_vehicle: true,
        ..TypeSounds::default()
    };
    n.set_type(Kind::Vehicle, index(&w, Kind::Vehicle, "BUGGY"), t);
    n.voice_ifv_repair = Some("Mend".into());
    a.set_names(n);
    let o = Order::Behaviour {
        actor: buggy,
        behaviour: Behaviour::Attack,
        attack_move: false,
        target: Some(Target::Object(cart)),
        dest: None,
    };
    let mut u = Rng::new(1);
    let (_, wid) = weapon::would_use(&w, buggy, Some(Target::Object(cart)));
    assert!(wid.is_some_and(|x| w.rules().combat.weapon(x).damage < 0));
    a.ordered(&w, &[buggy], &click(vec![o]), &mut u);
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 0), ["Mend"]);
}

// spec: S-0701 claim 19 (guard, stop and scatter: their game-wide sounds as screen sounds; quirks.deploy_command_silent)
#[test]
fn key_command_sounds() {
    let w = world();
    for quirk in [true, false] {
        let q = Quirks {
            deploy_command_silent: quirk,
            ..Quirks::default()
        };
        let mut a = audio_with(&w, &["G", "S", "X", "D"], q);
        let mut n = SoundNames::default();
        n.guard = Some("G".into());
        n.stop = Some("S".into());
        n.scatter = Some("X".into());
        n.deploy_command = Some("D".into());
        a.set_names(n);
        let mut now = 0;
        for (k, want) in [
            (KeyCommand::Guard, Some("G")),
            (KeyCommand::Stop, Some("S")),
            (KeyCommand::Scatter, Some("X")),
            (KeyCommand::Deploy, (!quirk).then_some("D")),
        ] {
            a.mixer_mut().clear();
            a.key(k);
            now += 100;
            let got = heard(&mut a, now);
            assert_eq!(got.first().map(String::as_str), want);
        }
    }
}

fn hit(w: &World, of: TypeRef, owner: u8, result: HitResult, attacker: Option<ObjectId>) -> Effect {
    let _ = w;
    Effect::Hit {
        id: ObjectId(999),
        of,
        owner: Some(HouseId(owner)),
        pos: Pos::centre(testkit::mid(SIZE, 0, 0), 0),
        result,
        attacker,
        warhead: None,
        health: (0, 0),
    }
}

fn damage_names(w: &World) -> SoundNames {
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            damage_sound: Some("Ouch".into()),
            voice_feedback: strings(&["Help"]),
            die_sound: strings(&["Boom0", "Boom1"]),
            ..TypeSounds::default()
        },
    );
    n
}

// spec: S-0701 claim 20 (unchanged or beyond now dead: the damage sound, positional, any owner; exactly one instance per damage-sound event) and Edge cases (damage-sound departure, no switch)
#[test]
fn damage_sound() {
    let w = world();
    let t = tref(&w, Kind::Vehicle, "CRUSHER");
    {
        let mut a = audio_with(&w, &["Ouch"], Quirks::default());
        a.set_names(damage_names(&w));
        let mut u = Rng::new(1);
        for r in [HitResult::Unchanged, HitResult::PostMortem] {
            a.mixer_mut().clear();
            a.tick(&w, &[hit(&w, t, 1, r, None)], &mut u);
            assert_eq!(a.mixer().alive(), 1);
        }
        // Unaffected and now red: nothing, no draws.
        a.mixer_mut().clear();
        let before = u.taken();
        a.tick(
            &w,
            &[
                hit(&w, t, 0, HitResult::Unaffected, None),
                hit(&w, t, 0, HitResult::NowRed, None),
            ],
            &mut u,
        );
        assert_eq!(a.mixer().alive(), 0);
        assert_eq!(u.taken(), before);
    }
}

// spec: S-0701 claim 20 and 31 (now yellow: R(0, 99) for every owner; below 30 and local: one feedback response, positional)
#[test]
fn feedback_chance() {
    let w = world();
    let t = tref(&w, Kind::Vehicle, "CRUSHER");
    let mut a = audio(&w, &["Help"]);
    a.set_names(damage_names(&w));
    let (mut spoke, mut quiet) = (0, 0);
    for seed in 0..40 {
        let mut u = Rng::new(seed);
        let mut r = u.clone();
        a.mixer_mut().clear();
        a.tick(&w, &[hit(&w, t, 0, HitResult::NowYellow, None)], &mut u);
        let x = r.ranged(0, 99, true);
        if x < 30 {
            r.raw();
            spoke += 1;
        } else {
            quiet += 1;
        }
        assert_eq!(u, r);
        assert_eq!(a.mixer().alive(), usize::from(x < 30));
        assert!(
            !matches!(a.mixer().placement(InstanceId(0)), Some((_, p)) if p != crate::mixer::CENTRE)
        );
        // Another owner's object: the draw, never a response.
        let mut u = Rng::new(seed);
        let mut r = u.clone();
        a.mixer_mut().clear();
        a.tick(&w, &[hit(&w, t, 1, HitResult::NowYellow, None)], &mut u);
        r.ranged(0, 99, true);
        assert_eq!(u, r);
        assert_eq!(a.mixer().alive(), 0);
    }
    assert!(spoke > 0 && quiet > 0);
}

// spec: S-0701 claim 20 (now dead: for every owner one death sound chosen by F5, positional)
#[test]
fn death_sound() {
    let w = world();
    let t = tref(&w, Kind::Vehicle, "CRUSHER");
    let mut a = audio(&w, &["Boom0", "Boom1"]);
    a.set_names(damage_names(&w));
    let mut u = Rng::new(9);
    let mut r = u.clone();
    a.tick(&w, &[hit(&w, t, 1, HitResult::NowDead, None)], &mut u);
    let x = r.raw();
    assert_eq!(u, r);
    assert_eq!(heard(&mut a, 0), [format!("Boom{}", x % 2)]);
}

// spec: S-0701 claim 20 and Import mapping (`Voice` + `Die`: the local player's death response, chosen before the any-owner death sound)
#[test]
fn local_death_response() {
    let w = world();
    let t = tref(&w, Kind::Vehicle, "CRUSHER");
    let mut a = audio(&w, &["Boom0", "Last"]);
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(&w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            voice_die: strings(&["Last"]),
            die_sound: strings(&["Boom0"]),
            ..TypeSounds::default()
        },
    );
    a.set_names(n);
    let mut u = Rng::new(9);
    let before = u.taken();
    a.tick(&w, &[hit(&w, t, 0, HitResult::NowDead, None)], &mut u);
    assert_eq!(u.taken(), before + 2, "one choice for each list");
    let mut got = heard(&mut a, 0);
    got.sort();
    assert_eq!(got, ["Boom0", "Last"]);
    a.mixer_mut().clear();
    let before = u.taken();
    a.tick(&w, &[hit(&w, t, 1, HitResult::NowDead, None)], &mut u);
    assert_eq!(u.taken(), before + 1, "another house: the death sound only");
    assert_eq!(heard(&mut a, 100), ["Boom0"]);
}

// spec: S-0701 claim 37 (a building without its own damage sound: the game-wide one at now yellow and now red; with an empty death list: the game-wide death sound)
#[test]
fn building_fallback_sounds() {
    let w = world();
    let t = tref(&w, Kind::Building, "TOWER");
    let mut a = audio(&w, &["BDamage", "BDie", "Own", "OwnDie"]);
    let mut n = SoundNames::default();
    n.building_damage = Some("BDamage".into());
    n.building_die = Some("BDie".into());
    a.set_names(n.clone());
    let mut u = Rng::new(1);
    let mut now = 0;
    for (r, want) in [
        (HitResult::NowYellow, vec!["BDamage"]),
        (HitResult::NowRed, vec!["BDamage"]),
        (HitResult::NowDead, vec!["BDie"]),
        (HitResult::Unchanged, vec![]),
        (HitResult::Unaffected, vec![]),
    ] {
        a.mixer_mut().clear();
        a.tick(&w, &[hit(&w, t, 1, r, None)], &mut u);
        now += 100;
        assert_eq!(heard(&mut a, now), want, "{r:?}");
    }
    // With its own damage sound and death list, neither fallback plays.
    n.set_type(
        Kind::Building,
        t.index,
        TypeSounds {
            damage_sound: Some("Own".into()),
            die_sound: strings(&["OwnDie"]),
            ..TypeSounds::default()
        },
    );
    a.set_names(n);
    for (r, want) in [
        (HitResult::NowYellow, vec![]),
        (HitResult::NowRed, vec![]),
        (HitResult::NowDead, vec!["OwnDie"]),
    ] {
        a.mixer_mut().clear();
        a.tick(&w, &[hit(&w, t, 1, r, None)], &mut u);
        now += 100;
        assert_eq!(heard(&mut a, now), want, "{r:?}");
    }
    // A vehicle never takes the building fallbacks.
    let v = tref(&w, Kind::Vehicle, "CRUSHER");
    a.mixer_mut().clear();
    a.tick(&w, &[hit(&w, v, 1, HitResult::NowRed, None)], &mut u);
    assert!(heard(&mut a, now + 100).is_empty());
}

// spec: S-0701 claims 21 and 32 (the report at the firing position, any owner, no draw: entry r mod n, r the firer's report value)
#[test]
fn weapon_report() {
    let mut w = world();
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, 0, 0);
    let gun = veterancy::type_combat(&w, tank)
        .and_then(|c| c.slots[0])
        .expect("gun");
    let name = w.rules().combat.weapon(gun).name.clone();
    // r mod 2 picks the entry: 4 the first, 65535 the second.
    for (r, want) in [(4u16, "Bang0"), (65535, "Bang1")] {
        let mut a = audio(&w, &["Bang0", "Bang1"]);
        let mut n = SoundNames::default();
        n.set_report(&name, &["Bang0", "Bang1"]);
        a.set_names(n);
        let mut u = Rng::new(1);
        let fx = Effect::Shot {
            firer: tank,
            weapon: gun,
            from: Pos::centre(testkit::mid(SIZE, 0, 0), 0),
            report: r,
            gattling: false,
            target: Target::Object(ObjectId(999)),
        };
        a.tick(&w, &[fx], &mut u);
        assert_eq!(u.taken(), 0);
        assert_eq!(heard(&mut a, 0), [want], "r = {r}");
    }
}

// spec: S-0701 claim 22 (an animation's start sound, positional, any owner)
#[test]
fn animation_start_sound() {
    let w = world();
    let mut a = audio(&w, &["Poof"]);
    let mut n = SoundNames::default();
    n.set_anim_report("Puff", "Poof");
    a.set_names(n);
    let mut u = Rng::new(1);
    let fx = [
        Effect::Anim {
            name: "PUFF".into(),
            pos: Pos::new(0, 0, 0),
            delay: None,
            reverse: false,
            z_adjust: None,
            id: None,
        },
        Effect::Anim {
            name: "Silent".into(),
            pos: Pos::new(0, 0, 0),
            delay: None,
            reverse: false,
            z_adjust: None,
            id: None,
        },
    ];
    a.tick(&w, &fx, &mut u);
    assert_eq!(heard(&mut a, 0), ["Poof"]);
}

/// Screen points one sixteenth of the leptons: positions far apart are
/// placed apart.
fn linear(x: i32, y: i32, _: i32) -> (i32, i32) {
    (x / 16, y / 16)
}

// spec: S-0319 claim 5 (a bouncing animation's start sound, while it plays, follows its position in play; presentation only); design: once the animation has left play the sound plays on where it was
#[test]
fn bouncing_animation_start_sound_follows_it() {
    use glasnost_rules::sim::AnimationType;
    let mut r = ruleset();
    r.registries
        .entry("animations".into())
        .or_default()
        .push("DROP".into());
    r.sim.animations.insert(
        "DROP".into(),
        AnimationType {
            bouncer: Some(true),
            frame_count: Some(200),
            rate: Some(900),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, SIZE, 5);
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 2000);
    let id = glasnost_sim::anims::spawn(&mut w, "DROP", Some(at), Default::default())
        .id
        .expect("in play");
    let mut a = audio(&w, &["Whoosh"]);
    let mut cat = SoundCatalog::new();
    let mut e = SoundEntry::new("Whoosh", &["Whoosh"]);
    e.control.looped = true;
    cat.insert(e);
    a.set_catalog(cat);
    a.set_view(View {
        origin: (0, 0),
        size: (800, 600),
        project: linear,
    });
    let mut n = SoundNames::default();
    n.set_anim_report("DROP", "Whoosh");
    a.set_names(n);
    let mut u = Rng::new(1);
    let start = Effect::Anim {
        name: "DROP".into(),
        pos: at,
        delay: None,
        reverse: false,
        z_adjust: None,
        id: Some(id),
    };
    a.tick(&w, &[start], &mut u);
    assert_eq!(heard(&mut a, 0), ["Whoosh"]);
    let far = Pos::new(100_000, at.y, at.z);
    w.anims.anims.get_mut(&id).expect("in play").pos = Some(far);
    a.tick(&w, &[], &mut u);
    assert_eq!(a.mixer().alive(), 1);
    assert_eq!(a.mixer().busy_channels(), 0, "moved off the screen with it");
    w.anims.anims.get_mut(&id).expect("in play").pos = Some(at);
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 100), ["Whoosh"], "back with it");
    w.anims.anims.remove(&id);
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 200), ["Whoosh"], "it plays on where it was");
}

// spec: S-0701 claim 23 (movement: one entry by F5 as an attached sound; grace 3; then finish looping)
#[test]
fn movement_sound_episode() {
    let mut w = world();
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, 0, 0);
    let mut a = audio(&w, &["Rumble"]);
    let mut cat = SoundCatalog::new();
    let mut e = SoundEntry::new("Rumble", &["Rumble"]);
    e.control.looped = true;
    cat.insert(e);
    a.set_catalog(cat);
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(&w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            move_sound: strings(&["Rumble"]),
            ..TypeSounds::default()
        },
    );
    a.set_names(n);
    let mut u = Rng::new(1);
    a.tick(&w, &[], &mut u);
    assert_eq!(a.mixer().alive(), 0);
    w.obj_mut(tank).expect("tank").pos.x += 10;
    a.tick(&w, &[], &mut u);
    assert_eq!(u.taken(), 1);
    assert_eq!(heard(&mut a, 0), ["Rumble"]);
    w.obj_mut(tank).expect("tank").pos.x += 10;
    a.tick(&w, &[], &mut u);
    // Three updates count the grace down; the loop goes on.
    for _ in 0..3 {
        a.tick(&w, &[], &mut u);
    }
    render(&mut a, 100);
    assert_eq!(a.mixer().alive(), 1);
    // The following one asks it to finish looping.
    a.tick(&w, &[], &mut u);
    render(&mut a, 100);
    assert_eq!(a.mixer().alive(), 0);
    assert_eq!(u.taken(), 1);
}

// spec: S-0701 claim 23 (the rotation sound as an attached sound while the turret turns)
#[test]
fn turret_sound() {
    let mut w = world();
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, 0, 0);
    let mut a = audio(&w, &["Whirr"]);
    let mut cat = SoundCatalog::new();
    let mut e = SoundEntry::new("Whirr", &["Whirr"]);
    e.control.looped = true;
    cat.insert(e);
    a.set_catalog(cat);
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(&w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            turret_rotate: Some("Whirr".into()),
            ..TypeSounds::default()
        },
    );
    a.set_names(n);
    let mut u = Rng::new(1);
    a.tick(&w, &[], &mut u);
    assert_eq!(a.mixer().alive(), 0);
    // spec: S-0206 claim 17 (the sound update reads the latest rotation sample)
    w.obj_mut(tank).expect("tank").rotation_sample = true;
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 0), ["Whirr"]);
    // spec: S-0701 claim 23 (successive turning samples keep the existing instance)
    a.tick(&w, &[], &mut u);
    assert_eq!(a.mixer().alive(), 1);
    // spec: S-0701 claim 23 and scenario F12 (a non-turning sample asks it to end looping, no grace, no cut)
    w.obj_mut(tank).expect("tank").rotation_sample = false;
    w.tick += 10_000;
    a.tick(&w, &[], &mut u);
    assert_eq!(a.mixer().alive(), 1, "still finishing its pass");
    // spec: S-0701 claim 23 (a later turning sample starts a fresh instance; the finishing one is not cut)
    w.obj_mut(tank).expect("tank").rotation_sample = true;
    a.tick(&w, &[], &mut u);
    assert_eq!(a.mixer().alive(), 2);
    w.obj_mut(tank).expect("tank").rotation_sample = false;
    w.tick += 10_000;
    a.tick(&w, &[], &mut u);
    render(&mut a, 100);
    assert_eq!(a.mixer().alive(), 0);
}

// spec: S-0701 claim 23 (unit ready: the type's creation sound, else the game-wide one of its kind, any owner)
#[test]
fn unit_ready_sound() {
    let w = world();
    let mut a = audio(&w, &["Mine", "Wheels", "Boots"]);
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(&w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            create: Some("Mine".into()),
            ..TypeSounds::default()
        },
    );
    n.create_unit = Some("Wheels".into());
    n.create_infantry = Some("Boots".into());
    a.set_names(n);
    let mut u = Rng::new(1);
    let ready = |t: TypeRef, h: u8| Effect::Sound {
        sound: Sound::UnitReady,
        of: Some(t),
        house: Some(HouseId(h)),
        pos: Some(Pos::centre(testkit::mid(SIZE, 0, 0), 0)),
    };
    let fx = [
        ready(tref(&w, Kind::Vehicle, "CRUSHER"), 1),
        ready(tref(&w, Kind::Vehicle, "CART"), 1),
        ready(tref(&w, Kind::Infantry, "WALKER"), 0),
    ];
    a.tick(&w, &fx, &mut u);
    assert_eq!(heard(&mut a, 0), ["Mine", "Wheels", "Boots"]);
}

// spec: S-0701 claim 23 (building placement and sale sounds; promotion for the local player's objects)
#[test]
fn building_and_promotion_sounds() {
    let mut w = world();
    let mine = spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, 0, 0);
    let theirs = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, 2, 0);
    let mut a = audio(&w, &["Slam", "Cash", "Vet", "Elite"]);
    let mut n = SoundNames::default();
    n.building_slam = Some("Slam".into());
    n.sell = Some("Cash".into());
    n.upgrade_veteran = Some("Vet".into());
    n.upgrade_elite = Some("Elite".into());
    a.set_names(n);
    a.cue(UiCue::BuildingPlaced);
    assert_eq!(heard(&mut a, 0), ["Slam"]);
    a.mixer_mut().clear();
    let mut u = Rng::new(1);
    let sell = |h: u8, pos: Option<Pos>| Effect::Sound {
        sound: Sound::Sell,
        of: None,
        house: Some(HouseId(h)),
        pos,
    };
    a.tick(&w, &[sell(1, None), sell(0, None)], &mut u);
    assert_eq!(heard(&mut a, 100), ["Cash"]);
    a.mixer_mut().clear();
    for id in [mine, theirs] {
        w.obj_mut(id).expect("object").combat.experience.0 = 0x3F80_0000; // 1.0
    }
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 200), ["Vet"]);
    a.mixer_mut().clear();
    w.obj_mut(mine).expect("object").combat.experience.0 = 0x4000_0000; // 2.0
    a.tick(&w, &[], &mut u);
    assert_eq!(heard(&mut a, 300), ["Elite"]);
}

// spec: S-0701 claim 24 and S-0411 claim 18 (production-feedback clicks: the build click; a page request at the strip's end: the refusal sound; an accepted one: the tab-scroll sound); S-0416 claim 4 and S-0701 claim 35 (credit ticks at request level 8192)
#[test]
fn interface_sounds() {
    let w = world();
    let mut a = audio(&w, &["Click", "Nope", "Up", "Down", "Tab"]);
    let mut n = SoundNames::default();
    n.gui_build = Some("Click".into());
    n.scold = Some("Nope".into());
    n.gui_tab = Some("Tab".into());
    n.credit_ticks = strings(&["Up", "Down"]);
    a.set_names(n);
    let mut now = 0;
    for (c, want) in [
        (UiCue::BuildStarted { infantry: false }, "Click"),
        (UiCue::BuildQueued, "Click"),
        (UiCue::BuildResumed { infantry: true }, "Click"),
        (UiCue::BuildHeld, "Click"),
        (UiCue::BuildCancelled, "Click"),
        (UiCue::StripEnd, "Nope"),
        (UiCue::StripScroll, "Tab"),
        (UiCue::CreditTick { rising: true }, "Up"),
        (UiCue::CreditTick { rising: false }, "Down"),
    ] {
        a.mixer_mut().clear();
        let i = a.cue(c).expect("plays");
        now += 100;
        assert_eq!(heard(&mut a, now), [want]);
        let level = if matches!(c, UiCue::CreditTick { .. }) {
            FULL / 2
        } else {
            FULL
        };
        assert_eq!(a.mixer().placement(i), Some((level, crate::mixer::CENTRE)));
    }
    assert_eq!(CREDIT_TICK_LEVEL, 8192);
}

// spec: S-0701 claim 24 and S-0404 claim 3 (a production request refused as it is applied: the game-wide refusal sound, the local player's house only)
#[test]
fn queue_refusal_at_application() {
    let w = world();
    let mut a = audio(&w, &["Nope"]);
    let mut n = SoundNames::default();
    n.scold = Some("Nope".into());
    a.set_names(n);
    let mut u = Rng::new(1);
    let refused = |h: u8| Effect::Sound {
        sound: Sound::QueueRefused,
        of: None,
        house: Some(HouseId(h)),
        pos: None,
    };
    a.tick(&w, &[refused(1)], &mut u);
    assert!(heard(&mut a, 100).is_empty());
    a.tick(&w, &[refused(0)], &mut u);
    assert_eq!(heard(&mut a, 200), ["Nope"]);
}

// spec: S-0701 claim 24 (a busy line: only unable to comply; a refused placement: only cannot deploy here; no interface sound)
#[test]
fn refusal_cues_are_announcements_only() {
    let w = world();
    let mut a = announcing(
        &w,
        &[
            (Announcement::UnableToComply, "Nope"),
            (Announcement::CannotDeploy, "NoDep"),
        ],
    );
    let mut n = a.names().clone();
    n.scold = Some("Scold".into());
    n.gui_build = Some("Click".into());
    a.set_names(n);
    a.bank_mut().add_pcm(
        "Scold",
        Pcm {
            sample_rate: RATE,
            channels: 1,
            samples: vec![1000; 4],
        },
    );
    let mut now = 1000;
    for c in [UiCue::BuildRefused, UiCue::PlacementRefused] {
        a.mixer_mut().clear();
        assert_eq!(a.cue(c), None);
        assert!(heard(&mut a, now).is_empty());
        assert!(a.mixer().speaking());
        render(&mut a, 100);
        now += 1000;
    }
}

// spec: S-0701 claim 28 and Import mapping (rally point set: the local player's action requests its announcement, with no interface sound)
#[test]
fn rally_point_set_is_announced() {
    let w = world();
    let mut a = announcing(&w, &[(Announcement::RallySet, "Rally")]);
    assert_eq!(a.cue(UiCue::RallySet), None);
    assert!(heard(&mut a, 1000).is_empty());
    assert!(a.mixer().speaking());
}

// spec: S-0512 claim 25 and Import mapping (a superweapon icon click: the game-wide build click; arming: the select-target announcement, with no interface sound of its own)
#[test]
fn superweapon_icon_cues() {
    let w = world();
    let mut a = audio(&w, &["Click"]);
    add_voices(&mut a, &strings(&["Aim0", "Aim1", "Aim2"]));
    let mut s = SpeechCatalog::default();
    s.insert(Line {
        name: "Aim".into(),
        samples: strings(&["Aim0", "Aim1", "Aim2"]).try_into().unwrap(),
        ..Line::default()
    });
    a.set_speech(s);
    let mut n = SoundNames::default();
    n.gui_build = Some("Click".into());
    n.set_announcement(Announcement::SelectTarget, "Aim");
    a.set_names(n);
    assert!(a.cue(UiCue::SuperweaponClick).is_some());
    assert_eq!(heard(&mut a, 1000), ["Click"]);
    assert!(!a.mixer().speaking());
    a.mixer_mut().clear();
    assert_eq!(a.cue(UiCue::SelectTarget), None);
    assert!(heard(&mut a, 2000).is_empty());
    assert!(a.mixer().speaking());
}

/// Adds announcement files of four frames for `samples` (S-0701 claim 34:
/// the eight-character name plus `.wav`, apart from the sample bank).
fn add_voices(a: &mut Audio, samples: &[String]) {
    for n in samples {
        a.voices_mut().add_pcm(
            &crate::speech::file_name(n),
            Pcm {
                sample_rate: RATE,
                channels: 1,
                samples: vec![1000; 4],
            },
        );
    }
}

// spec: S-0701 claim 34 (announcements are separate `.wav` files, never bank samples; sound entries never play those files)
#[test]
fn announcement_files_are_apart_from_the_bank() {
    let w = world();
    let mut a = audio(&w, &["Bld0", "Bld1", "Bld2"]);
    let mut s = SpeechCatalog::default();
    s.insert(Line {
        name: "Bld".into(),
        samples: ["Bld0".into(), "Bld1".into(), "Bld2".into()],
        ..Line::default()
    });
    let mut names = SoundNames::default();
    names.set_announcement(Announcement::Building, "Bld");
    a.set_speech(s);
    a.set_names(names);
    a.announce(Announcement::Building);
    a.service(1000, &mut Rng::new(1));
    assert!(
        !a.mixer().speaking(),
        "a bank sample is not an announcement file"
    );
    add_voices(&mut a, &strings(&["Bld0", "Bld1", "Bld2"]));
    a.announce(Announcement::Building);
    a.service(2000, &mut Rng::new(1));
    assert!(a.mixer().speaking());
    assert_eq!(crate::speech::file_name("Bld0"), "Bld0.wav");
}

/// Audio whose announcements are entries named after their role, each with
/// one sample per voice set.
fn announcing(w: &World, roles: &[(Announcement, &str)]) -> Audio {
    let mut samples: Vec<String> = Vec::new();
    for (_, n) in roles {
        for v in 0..3 {
            samples.push(format!("{n}{v}"));
        }
    }
    let mut a = audio(w, &[]);
    add_voices(&mut a, &samples);
    let mut s = SpeechCatalog::default();
    let mut names = SoundNames::default();
    for (role, n) in roles {
        s.insert(Line {
            name: (*n).into(),
            samples: [format!("{n}0"), format!("{n}1"), format!("{n}2")],
            ..Line::default()
        });
        names.set_announcement(*role, n);
    }
    a.set_speech(s);
    a.set_names(names);
    a
}

// spec: S-0701 claim 24 and claim 28 and scenario F11; S-0411 claim 12 (start and resume request training for infantry, building for every other kind; a queued copy on an active line requests neither)
#[test]
fn production_announcement_kinds() {
    let w = world();
    let mut now = 0;
    for (cue, role, want) in [
        (
            UiCue::BuildStarted { infantry: true },
            Announcement::Training,
            true,
        ),
        (
            UiCue::BuildStarted { infantry: true },
            Announcement::Building,
            false,
        ),
        (
            UiCue::BuildResumed { infantry: false },
            Announcement::Building,
            true,
        ),
        (
            UiCue::BuildResumed { infantry: false },
            Announcement::Training,
            false,
        ),
        (UiCue::BuildQueued, Announcement::Building, false),
        (UiCue::BuildQueued, Announcement::Training, false),
    ] {
        let mut a = announcing(&w, &[(role, "Say")]);
        a.cue(cue);
        now += 1000;
        assert_eq!(announced(&mut a, now), want, "{cue:?} with only {role:?}");
    }
}

// spec: S-0701 claim 27 (one at a time, more than 500 ms after the previous start); claim 26 (the local player's voice set)
#[test]
fn announcements_one_at_a_time() {
    let w = world();
    let mut a = announcing(
        &w,
        &[
            (Announcement::Building, "Bld"),
            (Announcement::OnHold, "Hold"),
        ],
    );
    let set = a.voice_set();
    assert!(set < 3);
    a.cue(UiCue::BuildStarted { infantry: false });
    a.service(1000, &mut Rng::new(1));
    assert!(a.mixer().speaking());
    // A standard request while one plays and the place is empty waits.
    a.cue(UiCue::BuildHeld);
    render(&mut a, 100);
    assert!(!a.mixer().speaking());
    a.service(1400, &mut Rng::new(1));
    assert!(!a.mixer().speaking());
    a.service(1500, &mut Rng::new(1));
    assert!(!a.mixer().speaking());
    a.service(1501, &mut Rng::new(1));
    assert!(a.mixer().speaking());
}

// spec: S-0701 claim 27 (one at a time, more than 500 ms after the previous start): the names kept for captions (M8 step C3) follow the starts, not the requests, and a missing file still counts
#[test]
fn started_announcements_are_named_for_captions() {
    let w = world();
    let mut a = announcing(
        &w,
        &[
            (Announcement::Building, "Bld"),
            (Announcement::OnHold, "Hold"),
        ],
    );
    a.cue(UiCue::BuildStarted { infantry: false });
    a.service(1000, &mut Rng::new(1));
    assert_eq!(a.take_started(), ["Bld"]);
    a.cue(UiCue::BuildHeld);
    render(&mut a, 100);
    a.service(1500, &mut Rng::new(1));
    assert!(a.take_started().is_empty(), "the second waits its turn");
    a.service(1501, &mut Rng::new(1));
    assert_eq!(a.take_started(), ["Hold"]);
    assert!(a.take_started().is_empty());
    // Without the announcement's file nothing is heard, but it is named.
    let mut mute = audio(&w, &[]);
    mute.set_speech(speech_of(&["Mute"]));
    let mut names = SoundNames::default();
    names.set_announcement(Announcement::Building, "Mute");
    mute.set_names(names);
    mute.cue(UiCue::BuildStarted { infantry: false });
    mute.service(1000, &mut Rng::new(1));
    assert!(!mute.mixer().speaking());
    assert_eq!(mute.take_started(), ["Mute"]);
}

// spec: S-0701 claim 26 (the caption of a started announcement is its entry's text key when present with a nonempty value, otherwise the entry's name)
#[test]
fn started_announcement_captions_come_from_the_text_key() {
    let w = world();
    let mut a = audio(&w, &[]);
    let mut s = SpeechCatalog::default();
    for (n, text) in [("Bld", Some("Made-up words")), ("Hold", None)] {
        s.insert(Line {
            name: n.into(),
            text: text.map(Into::into),
            ..Line::default()
        });
    }
    let mut names = SoundNames::default();
    names.set_announcement(Announcement::Building, "Bld");
    names.set_announcement(Announcement::OnHold, "Hold");
    a.set_speech(s);
    a.set_names(names);
    a.cue(UiCue::BuildStarted { infantry: false });
    a.service(1000, &mut Rng::new(1));
    assert_eq!(a.take_started(), ["Made-up words"]);
    a.cue(UiCue::BuildHeld);
    a.service(1501, &mut Rng::new(1));
    assert_eq!(a.take_started(), ["Hold"], "no text key: the entry's name");
}

fn base_names(a: &mut Audio) {
    let mut n = SoundNames::default();
    for (r, s) in [
        (Announcement::BaseUnderAttack, "Base"),
        (Announcement::UnitLost, "Lost"),
        (Announcement::HarvesterUnderAttack, "Harv"),
        (Announcement::LowFunds, "Funds"),
        (Announcement::LowPower, "Power"),
    ] {
        n.set_announcement(r, s);
    }
    n.base_under_attack = Some("Siren".into());
    a.set_names(n);
}

fn speech_of(roles: &[&str]) -> SpeechCatalog {
    let mut s = SpeechCatalog::default();
    for n in roles {
        s.insert(Line {
            name: (*n).into(),
            samples: [format!("{n}0"), format!("{n}1"), format!("{n}2")],
            ..Line::default()
        });
    }
    s
}

/// Whether an announcement started at `now` after an earlier one ended.
fn announced(a: &mut Audio, now: i64) -> bool {
    a.service(now, &mut Rng::new(1));
    let s = a.mixer().speaking();
    render(a, 100);
    s
}

const ROLES: [&str; 5] = ["Base", "Lost", "Harv", "Funds", "Power"];

fn base_audio(w: &World) -> Audio {
    let mut samples = Vec::new();
    for n in ROLES {
        for v in 0..3 {
            samples.push(format!("{n}{v}"));
        }
    }
    let mut a = audio(w, &["Siren"]);
    add_voices(&mut a, &samples);
    a.set_speech(speech_of(&ROLES));
    base_names(&mut a);
    a
}

// spec: S-0701 claim 28 and F7 (base under attack with the attack-warning sound; suppressed near a recent one)
#[test]
fn base_under_attack() {
    let mut w = world();
    let tower = tref(&w, Kind::Building, "TOWER");
    let foe = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, 5, 5);
    let mut a = base_audio(&w);
    let mut u = Rng::new(1);
    let hit_by = |dx: i32, attacker: Option<ObjectId>| Effect::Hit {
        id: ObjectId(999),
        of: tower,
        owner: Some(ME),
        pos: Pos::centre(testkit::mid(SIZE, dx, 0), 0),
        result: HitResult::Unchanged,
        attacker,
        warhead: None,
        health: (0, 0),
    };
    let at = |dx: i32| hit_by(dx, Some(foe));
    a.tick(&w, &[at(0)], &mut u);
    assert_eq!(heard(&mut a, 0), ["Siren"]);
    assert!(announced(&mut a, 1000));
    // Within 8 cells: suppressed; 8 cells away: raised.
    a.tick(&w, &[at(7)], &mut u);
    assert!(!announced(&mut a, 2000));
    a.tick(&w, &[at(8)], &mut u);
    assert!(announced(&mut a, 3000));
    // A hit without an attacker raises nothing; no draws at all.
    a.tick(&w, &[hit_by(20, None)], &mut u);
    assert!(!announced(&mut a, 4000));
    assert_eq!(u.taken(), 0);
    // spec: S-0410 claims 14 and 15 (without drawn frames the zoom never completes: the event stays)
    w.tick += 600;
    a.tick(&w, &[at(0)], &mut u);
    assert!(!announced(&mut a, 4500));
    // Drawn frames finish the zoom; 600 ticks from that frame's tick, the place is free again.
    zoom_out(&mut a, &w);
    w.tick += 599;
    a.tick(&w, &[at(0)], &mut u);
    assert!(!announced(&mut a, 4800));
    w.tick += 1;
    a.tick(&w, &[at(0)], &mut u);
    assert!(announced(&mut a, 5000));
}

/// Enough drawn frames at one tick (as in a freeze) to finish every zoom.
fn zoom_out(a: &mut Audio, w: &World) {
    for _ in 0..1000 {
        a.radar_frame(w, (0, 0));
    }
}

/// The drawn frames an event at `cell` takes to finish its zoom under the
/// world's rules (S-0410 F9), with the radar map at the screen origin.
fn zoom_frames(w: &World, cell: (i32, i32)) -> i32 {
    let g = minimap::grid(w).expect("grid");
    let img = minimap::image(&g);
    let r = w.rules().econ.radar.zoom;
    let c = minimap::event_centre(&g, &img, (img.left, img.top), cell.0, cell.1);
    let mut z = Zoom::new(&r, c, img.width, img.height);
    let mut n = 1;
    while !z.advance(&r) {
        n += 1;
    }
    n
}

// spec: S-0410 claims 14 and 15, S-0314 claim 6, S-0701 F7 (the post-zoom lifetime starts at the tick of the zoom-completion frame; frames advance zoom without ticks)
#[test]
fn radar_event_lifetime_follows_zoom() {
    let mut w = world();
    let tank = tref(&w, Kind::Vehicle, "CRUSHER");
    let mut a = base_audio(&w);
    let mut u = Rng::new(1);
    let lost = |w: &World| hit(w, tank, 0, HitResult::NowDead, None);
    a.tick(&w, &[lost(&w)], &mut u);
    assert!(announced(&mut a, 1000));
    let c = testkit::mid(SIZE, 0, 0);
    let d = zoom_frames(&w, (c.x, c.y));
    assert!(d > 1, "the zoom takes several drawn frames");
    // One drawn frame per tick until the frame before completion.
    for _ in 0..d - 1 {
        a.radar_frame(&w, (0, 0));
        w.tick += 1;
    }
    // Still zooming: no expiry however many ticks pass without a frame.
    w.tick += 10_000;
    a.tick(&w, &[lost(&w)], &mut u);
    assert!(!announced(&mut a, 2000));
    // The completion frame: the 200-tick tail starts at this tick.
    a.radar_frame(&w, (0, 0));
    let t0 = w.tick;
    w.tick = t0 + 199;
    a.tick(&w, &[lost(&w)], &mut u);
    assert!(!announced(&mut a, 3000));
    w.tick = t0 + 200;
    a.tick(&w, &[lost(&w)], &mut u);
    assert!(announced(&mut a, 4000));
}

// spec: S-0701 claim 28 (unit lost and harvester under attack, F7)
#[test]
fn unit_lost_and_harvester() {
    let w = world();
    let mut a = base_audio(&w);
    let mut u = Rng::new(1);
    let tank = tref(&w, Kind::Vehicle, "CRUSHER");
    let cart = tref(&w, Kind::Vehicle, "CART");
    a.tick(&w, &[hit(&w, tank, 0, HitResult::NowDead, None)], &mut u);
    assert!(announced(&mut a, 1000));
    a.tick(&w, &[hit(&w, tank, 0, HitResult::NowDead, None)], &mut u);
    assert!(!announced(&mut a, 2000));
    a.tick(&w, &[hit(&w, tank, 1, HitResult::NowDead, None)], &mut u);
    assert!(!announced(&mut a, 2500));
    a.tick(&w, &[hit(&w, cart, 0, HitResult::Unchanged, None)], &mut u);
    assert!(announced(&mut a, 3000));
}

// spec: S-0701 claim 28 (low funds: below 100 credits with a factory; the countdown N(⌊k × 900⌋))
#[test]
fn low_funds() {
    let mut w = world();
    let mut a = base_audio(&w);
    let mut u = Rng::new(1);
    w.houses[0].credits = 50;
    // No factory: nothing.
    a.tick(&w, &[], &mut u);
    assert!(!announced(&mut a, 1000));
    spawn(&mut w, Kind::Building, "BARR", 0, 0, 0);
    a.tick(&w, &[], &mut u);
    assert!(announced(&mut a, 2000));
    // spec: S-0701 claim 38 and scenario A5 (no speak delay: initially 2 game minutes, a countdown of N(1800) ticks)
    a.tick(&w, &[], &mut u);
    assert!(!announced(&mut a, 3000));
    let initial = glasnost_sim::infantry::normalised(1800, w.game_speed);
    let t0 = w.tick;
    w.tick = t0 + initial - 1;
    a.tick(&w, &[], &mut u);
    assert!(!announced(&mut a, 3500));
    w.tick = t0 + initial;
    a.tick(&w, &[], &mut u);
    assert!(announced(&mut a, 4000));
    w.tick = t0;
    // With a speak delay of one minute the countdown restarts at N(900).
    let mut b = base_audio(&w);
    let mut n = SoundNames::default();
    n.set_announcement(Announcement::LowFunds, "Funds");
    n.speak_delay = Some(glasnost_ini::binfloat::Binary::from_i64(
        glasnost_ini::binfloat::Format::Double,
        1,
    ));
    b.set_names(n);
    b.tick(&w, &[], &mut u);
    assert!(announced(&mut b, 1000));
    let wait = glasnost_sim::infantry::normalised(900, w.game_speed);
    w.tick += wait - 1;
    b.tick(&w, &[], &mut u);
    assert!(!announced(&mut b, 2000));
    w.tick += 1;
    b.tick(&w, &[], &mut u);
    assert!(announced(&mut b, 3000));
    // Enough credits: nothing.
    w.houses[0].credits = 100;
    w.tick += wait;
    b.tick(&w, &[], &mut u);
    assert!(!announced(&mut b, 4000));
}

// spec: S-0701 claim 28 (low power once per episode, with one of the first three construction-building types)
#[test]
fn low_power_once_per_episode() {
    let mut w = world();
    let mut a = base_audio(&w);
    let mut n = SoundNames::default();
    n.set_announcement(Announcement::LowPower, "Power");
    n.build_const = vec![index(&w, Kind::Building, "BARR")];
    a.set_names(n);
    let mut u = Rng::new(1);
    w.houses[0].credits = 10_000;
    w.houses[0].econ.power.drain = 50;
    w.houses[0].econ.power.output = 10;
    a.tick(&w, &[], &mut u);
    assert!(!announced(&mut a, 1000));
    spawn(&mut w, Kind::Building, "BARR", 0, 0, 0);
    w.houses[0].econ.power.drain = 50;
    w.houses[0].econ.power.output = 10;
    a.tick(&w, &[], &mut u);
    assert!(announced(&mut a, 2000));
    a.tick(&w, &[], &mut u);
    assert!(!announced(&mut a, 3000));
    w.houses[0].econ.power.output = 60;
    a.tick(&w, &[], &mut u);
    w.houses[0].econ.power.output = 10;
    a.tick(&w, &[], &mut u);
    assert!(announced(&mut a, 4000));
}

// spec: S-0701 claim 29 (nothing in the game reads sound: the state hash is the same with and without audio)
#[test]
fn state_hash_without_audio() {
    let run = |audio: bool| {
        let mut w = testkit::combat_game(21);
        let rep = testkit::combat_replay(21, 300);
        let mut a = audio.then(|| {
            let mut a = audio_with(&w, &["Bang", "Boom", "Ouch", "Yes"], Quirks::default());
            let mut n = SoundNames::default();
            for kind in [Kind::Vehicle, Kind::Infantry] {
                for i in 0..3 {
                    n.set_type(
                        kind,
                        i,
                        TypeSounds {
                            damage_sound: Some("Ouch".into()),
                            die_sound: strings(&["Boom"]),
                            voice_feedback: strings(&["Yes"]),
                            move_sound: strings(&["Ouch"]),
                            ..TypeSounds::default()
                        },
                    );
                }
            }
            a.set_names(n);
            a
        });
        let mut u = Rng::new(21);
        for t in 0..300 {
            rep.issue_due(&mut w);
            glasnost_sim::tick::step(&mut w);
            let fx = std::mem::take(&mut w.effects);
            if let Some(a) = a.as_mut() {
                a.tick(&w, &fx, &mut u);
                a.service(i64::from(t) * 67, &mut u);
                render(a, 64);
            }
        }
        glasnost_sim::save::state_hash(&w)
    };
    assert_eq!(run(true), run(false));
}

// spec: S-0701 claim 30 (the per-instance draws take the caller's unsynchronised stream; no separate presentation generator)
#[test]
fn instance_draws_take_the_shared_stream() {
    let w = world();
    let mut a = audio(&w, &["Wobble"]);
    let mut e = SoundEntry::new("Wobble", &["Wobble"]);
    e.fshift = (-5, 7);
    let mut cat = SoundCatalog::new();
    cat.insert(e);
    a.set_catalog(cat);
    let own = a.mixer().rng().clone();
    a.play_named(Some("Wobble"), Where::Screen)
        .expect("requested");
    let mut u = Rng::new(5);
    let before = u.taken();
    a.service(100, &mut u);
    assert!(u.taken() > before, "the pass drew from the shared stream");
    assert_eq!(a.mixer().rng(), &own, "the mixer's own stream is untouched");
}

// spec: S-0314 claim 6; S-0701 claim 28 and F7 (repair complete: the depot owner's announcement when it is the local player, through the unit repaired radar event, 2 cells and 400 ticks; a nearby event suppresses it)
#[test]
fn depot_repair_complete_announcement() {
    let w = world();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let fx = |h: u8| Effect::Sound {
        sound: Sound::UnitRepaired,
        of: None,
        house: Some(HouseId(h)),
        pos: Some(at),
    };
    let mut a = announcing(&w, &[(Announcement::RepairComplete, "Fixed")]);
    let mut u = Rng::new(1);
    a.tick(&w, &[fx(1)], &mut u);
    assert!(!announced(&mut a, 1000), "another house's depot: nothing");
    a.tick(&w, &[fx(0)], &mut u);
    assert!(
        announced(&mut a, 2000),
        "the local player's depot announces"
    );
    a.mixer_mut().clear();
    a.tick(&w, &[fx(0)], &mut u);
    assert!(
        !announced(&mut a, 3000),
        "a nearby unit repaired event suppresses it"
    );
}

// spec: S-0314 claim 6 (the depot's low-funds announcement: the local player's only, no suppression)
#[test]
fn depot_low_funds_announcement() {
    let w = world();
    let fx = |h: u8| Effect::Sound {
        sound: Sound::LowFunds,
        of: None,
        house: Some(HouseId(h)),
        pos: None,
    };
    let mut a = announcing(&w, &[(Announcement::LowFunds, "Funds")]);
    let mut u = Rng::new(1);
    a.tick(&w, &[fx(1)], &mut u);
    assert!(!announced(&mut a, 1000));
    a.tick(&w, &[fx(0)], &mut u);
    assert!(announced(&mut a, 2000));
}

// spec: S-0210 claim 4 (the crushed type's crush sound at the crusher); S-0210 claim 8 (the crushed overlay's crush sound); S-0210 Import mapping (`CrushSound` of a type or an overlay)
#[test]
fn crush_sounds() {
    let w = world();
    let mut a = audio(&w, &["Squish", "Crunch"]);
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Infantry,
        index(&w, Kind::Infantry, "WALKER"),
        TypeSounds {
            crush: Some("Squish".into()),
            ..TypeSounds::default()
        },
    );
    n.set_overlay_crush(3, "Crunch");
    a.set_names(n);
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let fx = [
        Effect::Sound {
            sound: Sound::Crush,
            of: Some(tref(&w, Kind::Infantry, "WALKER")),
            house: None,
            pos: Some(at),
        },
        Effect::OverlayCrushed {
            overlay: 3,
            pos: at,
        },
        // An overlay without a crush sound: nothing.
        Effect::OverlayCrushed {
            overlay: 4,
            pos: at,
        },
    ];
    a.tick(&w, &fx, &mut Rng::new(1));
    assert_eq!(heard(&mut a, 0), ["Squish", "Crunch"]);
}

// spec: S-0207 claim 20 (e) (a teleporting jump's departure sound at the old point and arrival sound at L; the type's sounds, which the names fall back to the game-wide ones for)
#[test]
fn chrono_sounds() {
    let w = world();
    let mut a = audio(&w, &["Away", "Back"]);
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(&w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            chrono_out: Some("Away".into()),
            chrono_in: Some("Back".into()),
            ..TypeSounds::default()
        },
    );
    a.set_names(n);
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let jump = |sound| Effect::Sound {
        sound,
        of: Some(tref(&w, Kind::Vehicle, "CRUSHER")),
        house: None,
        pos: Some(at),
    };
    let fx = [jump(Sound::ChronoOut), jump(Sound::ChronoIn)];
    a.tick(&w, &fx, &mut Rng::new(1));
    assert_eq!(heard(&mut a, 0), ["Away", "Back"]);
}

// spec: S-0508 claim 6; S-0701 claim 28 (structure garrisoned: the garrisoned sound and the announcement for the local player's entering unit, whoever owns the building; nothing for another house's unit)
#[test]
fn garrisoned_feedback_is_the_entering_units() {
    let w = world();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let fx = |h: u8| Effect::Sound {
        sound: Sound::Garrisoned,
        of: None,
        house: Some(HouseId(h)),
        pos: Some(at),
    };
    let mut a = announcing(&w, &[(Announcement::StructureGarrisoned, "Garr")]);
    let mut n = SoundNames::default();
    n.set_announcement(Announcement::StructureGarrisoned, "Garr");
    n.building_garrisoned = Some("GarrSound".into());
    a.set_names(n);
    let mut u = Rng::new(1);
    a.tick(&w, &[fx(1)], &mut u);
    assert!(!announced(&mut a, 1000), "another house's unit: nothing");
    a.tick(&w, &[fx(0)], &mut u);
    assert!(announced(&mut a, 2000), "the local player's entering unit");
}

// spec: S-0508 claim 8; S-0701 claim 28 and F7 (structure abandoned: the announcement for the owner that lost the building when it is the local player, only when the garrison abandoned radar event is created: 8 cells, 400 ticks, suppressing)
#[test]
fn abandoned_announcement_follows_the_radar_event() {
    let w = world();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let fx = |h: u8| Effect::Sound {
        sound: Sound::Abandoned,
        of: None,
        house: Some(HouseId(h)),
        pos: Some(at),
    };
    let mut a = announcing(&w, &[(Announcement::StructureAbandoned, "Aband")]);
    let mut u = Rng::new(1);
    a.tick(&w, &[fx(1)], &mut u);
    assert!(!announced(&mut a, 1000), "another house lost it: nothing");
    a.tick(&w, &[fx(0)], &mut u);
    assert!(announced(&mut a, 2000), "the local player lost it");
    a.mixer_mut().clear();
    a.tick(&w, &[fx(0)], &mut u);
    assert!(
        !announced(&mut a, 3000),
        "a nearby garrison abandoned event suppresses the announcement"
    );
}

// spec: S-0509 claim 12 and S-0701 claim 39 (the capture and loss announcements: eligible when the cause requests one, the local player is the old or new owner and the new owner's country is not passive; an ordinary building uses the captured entry for both)
#[test]
fn capture_announcements() {
    use glasnost_sim::econ::TypeRef;
    let w = world();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let ty = TypeRef {
        kind: Kind::Vehicle,
        index: 0,
    };
    let fx = |old: u8, new: u8, announce: bool| Effect::Captured {
        building: ObjectId(1),
        of: ty,
        old: Some(HouseId(old)),
        new: Some(HouseId(new)),
        pos: at,
        announce,
        after_slots: announce,
    };
    let mut a = announcing(&w, &[(Announcement::BuildingCaptured, "Capt")]);
    let mut u = Rng::new(1);
    a.tick(&w, &[fx(1, 0, true)], &mut u);
    assert!(announced(&mut a, 1000), "the local player gained it");
    a.mixer_mut().clear();
    a.tick(&w, &[fx(0, 1, true)], &mut u);
    assert!(
        announced(&mut a, 2000),
        "the local player lost it: the same entry for an ordinary building"
    );
    a.mixer_mut().clear();
    a.tick(&w, &[fx(1, 0, false)], &mut u);
    assert!(
        !announced(&mut a, 3000),
        "a cause that requests none: nothing"
    );
    a.tick(&w, &[fx(1, 1, true)], &mut u);
    assert!(
        !announced(&mut a, 4000),
        "the local player is neither owner"
    );
}

// spec: S-0509 claim 5 (the game-wide building-repaired sound plays at the building's centre, for every player)
#[test]
fn building_repaired_sound() {
    let w = world();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let mut a = audio(&w, &["Fixed"]);
    let mut n = SoundNames::default();
    n.building_repaired = Some("Fixed".into());
    a.set_names(n);
    let fx = Effect::Sound {
        sound: Sound::BuildingRepaired,
        of: None,
        house: Some(HouseId(1)),
        pos: Some(at),
    };
    a.tick(&w, &[fx], &mut Rng::new(1));
    assert_eq!(heard(&mut a, 0), ["Fixed"]);
}

/// spec: S-0605 claims 7 and 13 (the rules-wide entry sound at the entrant, the exit sound at the building; none set: silent)
#[test]
fn bio_reactor_sounds() {
    let w = world();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let mut n = SoundNames::default();
    n.enter_bio_reactor = Some("Gulp".into());
    n.leave_bio_reactor = Some("Burp".into());
    let fx = |sound| Effect::Sound {
        sound,
        of: None,
        house: Some(HouseId(1)),
        pos: Some(at),
    };
    for (sound, name) in [
        (Sound::EnterBioReactor, "Gulp"),
        (Sound::LeaveBioReactor, "Burp"),
    ] {
        let mut a = audio(&w, &["Gulp", "Burp"]);
        a.set_names(n.clone());
        a.tick(&w, &[fx(sound)], &mut Rng::new(1));
        assert_eq!(heard(&mut a, 0), [name]);
    }
    let mut b = audio(&w, &["Gulp"]);
    b.set_names(SoundNames::default());
    b.tick(&w, &[fx(Sound::EnterBioReactor)], &mut Rng::new(1));
    assert!(heard(&mut b, 0).is_empty());
}

/// spec: S-0606 claims 7, 12 and 14 (the imported walls-up and walls-down sounds at the bunker's position, for every player; none set: silent)
#[test]
fn bunker_walls_sounds() {
    let w = world();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let mut n = SoundNames::default();
    n.bunker_walls_up = Some("Up".into());
    n.bunker_walls_down = Some("Down".into());
    let fx = |sound| Effect::Sound {
        sound,
        of: None,
        house: Some(HouseId(1)),
        pos: Some(at),
    };
    for (sound, name) in [
        (Sound::BunkerWallsUp, "Up"),
        (Sound::BunkerWallsDown, "Down"),
    ] {
        let mut a = audio(&w, &["Up", "Down"]);
        a.set_names(n.clone());
        a.tick(&w, &[fx(sound)], &mut Rng::new(1));
        assert_eq!(heard(&mut a, 0), [name]);
    }
    let mut b = audio(&w, &["Up"]);
    b.set_names(SoundNames::default());
    b.tick(&w, &[fx(Sound::BunkerWallsUp)], &mut Rng::new(1));
    assert!(heard(&mut b, 0).is_empty());
}

fn infiltration(
    rows: Vec<glasnost_sim::combat::effects::InfiltrationRow>,
    satellite: bool,
    infiltrator: u8,
    victim: u8,
) -> Effect {
    Effect::Infiltrated {
        building: ObjectId(1),
        infiltrator: Some(HouseId(infiltrator)),
        victim: Some(HouseId(victim)),
        rows,
        satellite,
        pos: Pos::centre(testkit::mid(SIZE, 0, 0), 0),
    }
}

// spec: S-0510 claim 11 (the lines by row and by role: the victim hears the sabotage or theft line, the infiltrator hears building infiltrated and then the row's line, one line for radar and cash; a radar row with an active spy satellite is silent; a house that is neither hears nothing)
#[test]
fn infiltration_announcement_lines() {
    use glasnost_sim::combat::effects::InfiltrationRow as R;
    let w = world();
    // The local player (house 0) is the victim of a power infiltration.
    let mut a = announcing(&w, &[(Announcement::PowerSabotaged, "Pwr")]);
    a.tick(
        &w,
        &[infiltration(vec![R::Power], false, 1, 0)],
        &mut Rng::new(1),
    );
    assert!(announced(&mut a, 1000), "the victim hears power sabotaged");
    // The local player is the infiltrator: not the victim's line.
    let mut a = announcing(&w, &[(Announcement::PowerSabotaged, "Pwr")]);
    a.tick(
        &w,
        &[infiltration(vec![R::Power], false, 0, 1)],
        &mut Rng::new(1),
    );
    assert!(
        !announced(&mut a, 1000),
        "the infiltrator does not hear the victim's line"
    );
    let mut a = announcing(
        &w,
        &[
            (Announcement::BuildingInfiltrated, "Inf"),
            (Announcement::EnemyBasePoweredDown, "Down"),
        ],
    );
    a.tick(
        &w,
        &[infiltration(vec![R::Power], false, 0, 1)],
        &mut Rng::new(1),
    );
    assert!(announced(&mut a, 1000), "the infiltrator's lines");
    // The radar row: one line for the infiltrator, none with a satellite.
    for (satellite, want) in [(false, true), (true, false)] {
        let mut a = announcing(&w, &[(Announcement::BuildingInfRadarSabotaged, "Both")]);
        a.tick(
            &w,
            &[infiltration(vec![R::Radar], satellite, 0, 1)],
            &mut Rng::new(1),
        );
        assert_eq!(announced(&mut a, 1000), want, "satellite {satellite}");
    }
    // Cash: the victim hears cash stolen; the infiltrator the one line.
    let mut a = announcing(&w, &[(Announcement::CashStolen, "Cash")]);
    a.tick(
        &w,
        &[infiltration(vec![R::Cash], false, 1, 0)],
        &mut Rng::new(1),
    );
    assert!(announced(&mut a, 1000));
    let mut a = announcing(&w, &[(Announcement::BuildingInfCashStolen, "Both")]);
    a.tick(
        &w,
        &[infiltration(vec![R::Cash], false, 0, 1)],
        &mut Rng::new(1),
    );
    assert!(announced(&mut a, 1000));
    // Tech rows: technology stolen for the victim.
    let mut a = announcing(&w, &[(Announcement::TechnologyStolen, "Tech")]);
    a.tick(
        &w,
        &[infiltration(vec![R::VehicleFactory], false, 1, 0)],
        &mut Rng::new(1),
    );
    assert!(announced(&mut a, 1000));
    // The same house on both sides, or neither the local player's: nothing.
    let mut a = announcing(
        &w,
        &[
            (Announcement::PowerSabotaged, "Pwr"),
            (Announcement::BuildingInfiltrated, "Inf"),
        ],
    );
    a.tick(
        &w,
        &[infiltration(vec![R::Power], false, 1, 1)],
        &mut Rng::new(1),
    );
    assert!(!announced(&mut a, 1000));
}

// spec: S-0610 claim 13 (the psychic reveal's activation sound plays at the target cell's centre for every player, the launching house's or not)
#[test]
fn psychic_reveal_sound() {
    let mut r = ruleset();
    r.sim.general.superweapons.psychic_reveal_activate_sound = Some("Whisper".into());
    let w = testkit::game_with(&r, SIZE, 5);
    let mut a = audio(&w, &["Whisper"]);
    let mut u = Rng::new(1);
    let fx = [Effect::Superweapon {
        event: SwEvent::RevealSound,
        house: Some(HouseId(1)),
        sw: None,
        pos: Some(Pos::centre(testkit::mid(SIZE, 0, 0), 0)),
    }];
    a.tick(&w, &fx, &mut u);
    assert_eq!(heard(&mut a, 0), ["Whisper"]);
}

/// A spin effect of `id`.
fn spin_fx(id: ObjectId, event: glasnost_sim::gattling::SpinEvent) -> Effect {
    Effect::Spin {
        id,
        event,
        pos: Some(Pos::centre(testkit::mid(SIZE, 0, 0), 0)),
    }
}

// spec: S-0609 claim 13 (one draw per spin sound start: at a spin-up with the mark clear, again at a stage rise and after a spin-down or leaving the map; none at a spin-up that raises nothing while the mark is set, none for an empty report list or an empty slot, which leave the mark clear; no per-shot report)
#[test]
fn spin_sound_draws() {
    use glasnost_sim::gattling::SpinEvent;
    let mut w = world();
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, 0, 0);
    let gun = veterancy::type_combat(&w, tank)
        .and_then(|c| c.slots[0])
        .expect("gun");
    let name = w.rules().combat.weapon(gun).name.clone();
    let mut a = audio(&w, &["Spin0", "Spin1"]);
    let mut n = SoundNames::default();
    n.set_report(&name, &["Spin0", "Spin1"]);
    a.set_names(n);
    let mut u = Rng::new(1);
    let up = |raised| SpinEvent::Up {
        raised,
        weapon: Some(gun),
    };
    // A gattling firer's shot: no report, no draw.
    let shot = Effect::Shot {
        firer: tank,
        weapon: gun,
        from: Pos::centre(testkit::mid(SIZE, 0, 0), 0),
        report: 4,
        gattling: true,
        target: Target::Object(ObjectId(999)),
    };
    a.tick(&w, &[shot], &mut u);
    assert_eq!(u.taken(), 0);
    assert!(heard(&mut a, 0).is_empty());
    let steps: [(SpinEvent, u64); 9] = [
        (up(false), 1),
        (up(false), 1),
        (up(true), 2),
        (up(false), 2),
        (SpinEvent::Down, 2),
        (up(false), 3),
        (SpinEvent::Left, 3),
        (up(false), 4),
        (up(false), 4),
    ];
    for (k, (e, want)) in steps.into_iter().enumerate() {
        a.tick(&w, &[spin_fx(tank, e)], &mut u);
        assert_eq!(u.taken(), want, "step {k}: {e:?}");
    }
    assert_eq!(heard(&mut a, 100).len(), 1, "one spin sound at a time");
    // An empty slot: no draw, and the mark stays clear.
    a.tick(&w, &[spin_fx(tank, SpinEvent::Down)], &mut u);
    let empty = SpinEvent::Up {
        raised: false,
        weapon: None,
    };
    a.tick(&w, &[spin_fx(tank, empty)], &mut u);
    assert_eq!(u.taken(), 4);
    a.tick(&w, &[spin_fx(tank, up(false))], &mut u);
    assert_eq!(u.taken(), 5, "the mark was left clear");
    // A weapon without a report list: likewise.
    let mut a = audio(&w, &["Spin0"]);
    let mut u = Rng::new(1);
    a.tick(&w, &[spin_fx(tank, up(false))], &mut u);
    a.tick(&w, &[spin_fx(tank, up(true))], &mut u);
    assert_eq!(u.taken(), 0);
    assert!(heard(&mut a, 0).is_empty());
}

/// spec: S-0603 claim 7 (a consumption: the local player's entrant gives a death response and then a death sound, one raw draw each; another house's only the death sound; the enter-grinder sound at the entrant when the building grinds)
#[test]
fn grinder_consumption_sounds() {
    let w = world();
    let t = tref(&w, Kind::Vehicle, "CRUSHER");
    let mut n = SoundNames::default();
    n.set_type(
        Kind::Vehicle,
        index(&w, Kind::Vehicle, "CRUSHER"),
        TypeSounds {
            voice_die: strings(&["Last"]),
            die_sound: strings(&["Boom0"]),
            ..TypeSounds::default()
        },
    );
    n.enter_grinder = Some("Crunch".into());
    let fx = |owner: u8, grinding: bool| Effect::Consumed {
        of: t,
        owner: Some(HouseId(owner)),
        pos: Pos::centre(testkit::mid(SIZE, 0, 0), 0),
        grinding,
    };
    let mut a = audio(&w, &["Boom0", "Last", "Crunch"]);
    a.set_names(n.clone());
    let mut u = Rng::new(9);
    let before = u.taken();
    a.tick(&w, &[fx(0, true)], &mut u);
    assert_eq!(u.taken(), before + 2, "one choice for each list");
    let mut got = heard(&mut a, 0);
    got.sort();
    assert_eq!(got, ["Boom0", "Crunch", "Last"]);
    let mut b = audio(&w, &["Boom0", "Last", "Crunch"]);
    b.set_names(n);
    let before = u.taken();
    b.tick(&w, &[fx(1, false)], &mut u);
    assert_eq!(u.taken(), before + 1, "another house: the death sound only");
    assert_eq!(
        heard(&mut b, 0),
        ["Boom0"],
        "no enter-grinder sound without the flag"
    );
}

// spec: S-0615 claims 12, 14, 15 and 16 (a disc laser: silent at the shot and at a cancellation; the charge-up sound at its first arc step, if given; the weapon's report with the damage, entry r mod n from the firer's own value, no draw) and S-0701 claim 21
#[test]
fn disc_laser_sounds() {
    use glasnost_sim::combat::effects::DiscEvent;
    let mut w = world();
    let tank = spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, 0, 0);
    let gun = veterancy::type_combat(&w, tank)
        .and_then(|c| c.slots[0])
        .expect("gun");
    let name = w.rules().combat.weapon(gun).name.clone();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let fx = |event| Effect::Disc {
        firer: tank,
        weapon: gun,
        event,
        pos: at,
    };
    let names = || {
        let mut n = SoundNames::default();
        n.set_report(&name, &["Zap0", "Zap1"]);
        n
    };
    for (event, want) in [
        (DiscEvent::Shot, vec![]),
        (DiscEvent::Cancelled, vec![]),
        (
            DiscEvent::ChargeUp {
                sound: Some("Hum".into()),
            },
            vec!["Hum"],
        ),
        (DiscEvent::ChargeUp { sound: None }, vec![]),
        (DiscEvent::Hit { report: 4 }, vec!["Zap0"]),
        (DiscEvent::Hit { report: 65535 }, vec!["Zap1"]),
    ] {
        let mut a = audio(&w, &["Hum", "Zap0", "Zap1"]);
        a.set_names(names());
        let mut u = Rng::new(1);
        a.tick(&w, &[fx(event.clone())], &mut u);
        assert_eq!(u.taken(), 0);
        assert_eq!(heard(&mut a, 0), want, "{event:?}");
    }
}

// spec: S-0615 Terms (a beam is drawing only: no sound) and claim 2 with S-0701 claim 22 (a burning walker's creation plays its type's start sound)
#[test]
fn walker_and_beam_sounds() {
    let w = world();
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let mut a = audio(&w, &["Roar"]);
    let mut n = SoundNames::default();
    n.set_anim_report("Burn", "Roar");
    a.set_names(n);
    let fx = [
        Effect::Beam {
            from: at,
            to: at,
            duration: 8,
            fade_out: false,
            colours: [[1, 2, 3]; 3],
            house: None,
        },
        Effect::Walker {
            id: ObjectId(9),
            name: "BURN".into(),
            pos: at,
            made: true,
        },
        Effect::Walker {
            id: ObjectId(9),
            name: "BURN".into(),
            pos: at,
            made: false,
        },
    ];
    a.tick(&w, &fx, &mut Rng::new(1));
    assert_eq!(heard(&mut a, 0), ["Roar"]);
}

// spec: S-0611 claim 2 (the genetic mutation's activation sound, positional at the mutation point) and claim 8 with S-0109 claim 13 (an infantry-making animation plays its type's start sound at its creation and at a follow-up type's start; its other events are silent)
#[test]
fn genetic_mutation_sounds() {
    use glasnost_sim::mutation::MakerEvent;
    let mut r = ruleset();
    r.sim.general.superweapons.genetic_mutator_activate_sound = Some("Hum".into());
    let w = testkit::game_with(&r, SIZE, 5);
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let mut a = audio(&w, &["Hum", "Grow"]);
    let mut n = SoundNames::default();
    n.set_anim_report("Bloom", "Grow");
    a.set_names(n);
    let maker = |event| Effect::Maker {
        id: ObjectId(9),
        name: "BLOOM".into(),
        pos: at,
        owner: Some(HouseId(1)),
        frame: 0,
        event,
    };
    let fx = [
        Effect::Superweapon {
            event: SwEvent::GeneticSound,
            house: Some(HouseId(1)),
            sw: None,
            pos: Some(at),
        },
        maker(MakerEvent::Made),
        maker(MakerEvent::Owner),
        maker(MakerEvent::Refused),
        maker(MakerEvent::Ended {
            unit: None,
            conversion: true,
        }),
    ];
    a.tick(&w, &fx, &mut Rng::new(1));
    assert_eq!(heard(&mut a, 0), ["Hum", "Grow"]);
}

// spec: S-0612 claim 1 (the psychic domination's activation sound at the centre of the target cell, positional) and claim 8 (the blast ring and the domination's own events are silent)
#[test]
fn dominator_sounds() {
    use glasnost_sim::domination::DomEvent;
    let mut r = ruleset();
    r.sim.general.superweapons.psychic_dominator_activate_sound = Some("Psi".into());
    let w = testkit::game_with(&r, SIZE, 5);
    let at = Pos::centre(testkit::mid(SIZE, 0, 0), 0);
    let mut a = audio(&w, &["Psi"]);
    let dom = |event| Effect::Domination {
        event,
        house: Some(HouseId(1)),
        pos: at,
    };
    let fx = [
        Effect::Superweapon {
            event: SwEvent::DominatorSound,
            house: Some(HouseId(1)),
            sw: None,
            pos: Some(at),
        },
        dom(DomEvent::Started),
        dom(DomEvent::Sweep),
        dom(DomEvent::Taken(ObjectId(9))),
        dom(DomEvent::Final),
        dom(DomEvent::Over),
        dom(DomEvent::RingEnded),
    ];
    a.tick(&w, &fx, &mut Rng::new(1));
    assert_eq!(heard(&mut a, 0), ["Psi"]);
}
