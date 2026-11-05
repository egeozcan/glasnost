//! The AI lane's counters of transport loads, hovering transports and
//! superweapon launches by kind (M6c step 2b and 2c), on the synthetic
//! skirmish (no game data).

use glasnost_ai::{fixture, game::Game};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::charge;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, World};
use glasnost_tools::skirmish_lane::Tally;

fn of(w: &World, h: u8, name: &str) -> Vec<glasnost_sim::world::ObjectId> {
    w.objects
        .values()
        .filter(|o| o.owner == Some(HouseId(h)) && o.on_map && o.health > 0)
        .filter(|o| {
            let r = w.rules();
            match o.kind {
                Kind::Building => r.buildings[o.type_index as usize].name == name,
                Kind::Vehicle => r.vehicles[o.type_index as usize].name == name,
                _ => false,
            }
        })
        .map(|o| o.id)
        .collect()
}

// Riflemen carried over the strait by a hovering transport are counted as
// loads onto a hovering transport, and their unloading as unloads
// (S-0507, S-0503).
#[test]
fn counts_the_loads_onto_a_hovering_transport() {
    let mut g = Game::new(
        fixture::rules(),
        &fixture::map_strait(),
        "synthetic-strait",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the strait skirmish loads");
    let mut tally = Tally::default();
    while of(&g.world, 0, "YARD").is_empty() && g.world.tick < 3000 {
        g.step();
        tally.observe(&g.world);
    }
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let free = |w: &World, c: glasnost_map::CellXY| {
        w.cells.usable(c) && w.objects.values().all(|o| o.pos.cell() != c)
    };
    let cells: Vec<glasnost_map::CellXY> = (0..12)
        .map(|d| glasnost_map::CellXY::new(yard.x, yard.y + 5 + d))
        .filter(|&c| free(&g.world, c))
        .take(4)
        .collect();
    assert_eq!(cells.len(), 4);
    testkit::spawn(&mut g.world, Kind::Vehicle, "LIFT", 0, cells[0], 0);
    for &c in &cells[1..] {
        testkit::spawn(&mut g.world, Kind::Infantry, "WALKER", 0, c, 0);
    }
    let limit = g.world.tick + 6000;
    while g.world.tick < limit && tally.unloads == 0 {
        g.step();
        tally.observe(&g.world);
    }
    assert!(tally.loads > 0, "riflemen boarded: {tally:?}");
    assert_eq!(
        tally.hover_loads, tally.loads,
        "onto the hovering transport"
    );
    assert!(tally.unloads > 0, "they were let out: {tally:?}");
}

// A launch is counted when the ready mark falls, by the kind of the type
// (S-0512, S-0513).
#[test]
fn counts_a_launch_by_kind() {
    let mut g = Game::new(
        fixture::rules(),
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    while of(&g.world, 0, "YARD").is_empty() && g.world.tick < 3000 {
        g.step();
        tally.observe(&g.world);
    }
    let yard = g.world.obj(of(&g.world, 0, "YARD")[0]).unwrap().pos.cell();
    let at = (2..12)
        .map(|d| glasnost_map::CellXY::new(yard.x - d, yard.y))
        .find(|&c| g.world.cells.usable(c) && g.world.objects.values().all(|o| o.pos.cell() != c))
        .expect("a free cell for the silo");
    testkit::spawn(&mut g.world, Kind::Building, "SILO", 0, at, 0);
    let limit = g.world.tick + 3000;
    while g.world.tick < limit && !charge::get(&g.world, HouseId(0), 0).is_some_and(|r| r.ready) {
        g.step();
        tally.observe(&g.world);
    }
    assert!(charge::get(&g.world, HouseId(0), 0).is_some_and(|r| r.ready));
    assert_eq!(tally.launches, 0);
    // The fire order a person gives, at the enemy's start.
    let enemy = fixture::at(fixture::START_DX, 0);
    assert!(g.issue(HouseId(0), Order::FireSuperweapon { sw: 0, cell: enemy }));
    for _ in 0..3 {
        g.step();
        tally.observe(&g.world);
    }
    assert_eq!(tally.launches, 1, "{tally:?}");
    assert_eq!(tally.launches_by_kind.get("Missile"), Some(&1));
}

// Units that go berserk are counted once each, when they go berserk, and
// the house-ticks with a hostile-intent line are counted (S-0610 claims 4
// and 10).
#[test]
fn counts_berserk_units_and_intent_lines() {
    let mut r = fixture::ruleset();
    let mut eye = fixture::building(1, 1, 150, 0);
    eye.econ.psychic_detection_radius = Some(5);
    eye.tech_level = Some(99);
    r.sim.buildings.insert("EYE".into(), eye);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("EYE".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    tally.observe(&g.world);
    assert_eq!((tally.berserked, tally.intent_ticks), (0, 0));
    let unit = g
        .world
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(1)) && o.kind != Kind::Building && o.on_map)
        .map(|o| o.id)
        .expect("a unit of house 1");
    glasnost_sim::psychic::enter(&mut g.world, unit, 10_000);
    let near = fixture::at(4, 4);
    let eye = testkit::spawn(&mut g.world, Kind::Building, "EYE", 0, near, 0);
    g.world.obj_mut(unit).unwrap().dest = Some(glasnost_sim::world::Target::Object(eye));
    for _ in 0..3 {
        tally.observe(&g.world);
    }
    assert_eq!(tally.berserked, 1, "{tally:?}");
    assert_eq!(tally.intent_ticks, 3, "{tally:?}");
}

// A takeover is counted once, when the control link forms; the most
// objects a house holds at once and its controllers are kept per house
// (S-0601 claims 1, 5 and 6).
#[test]
fn counts_takeovers_and_controllers() {
    use glasnost_rules::sim::{Decimal, WarheadType, WeaponType};
    let mut r = fixture::ruleset();
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            mind_control: Some(true),
            verses: Some(vec![Decimal("1".into()); 11]),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "RAY".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(40),
            range: Some(Decimal("2".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    let mut c = r.sim.vehicles["GUNR"].clone();
    c.primary = Some("RAY".into());
    c.secondary = None;
    c.tech_level = Some(99);
    r.sim.vehicles.insert("CTRL".into(), c);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("CTRL".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    let ctrl = testkit::spawn(&mut g.world, Kind::Vehicle, "CTRL", 0, fixture::at(4, 4), 0);
    tally.observe(&g.world);
    assert_eq!(tally.taken, 0);
    assert_eq!(tally.controllers[0].len(), 1, "{tally:?}");
    let unit = g
        .world
        .objects
        .values()
        .find(|o| o.owner == Some(HouseId(1)) && o.kind != Kind::Building && o.on_map)
        .map(|o| o.id)
        .expect("a unit of house 1");
    assert!(glasnost_sim::mind::take(&mut g.world, ctrl, unit));
    for _ in 0..3 {
        tally.observe(&g.world);
    }
    assert_eq!(tally.taken, 1, "{tally:?}");
    assert_eq!(tally.controlled_max[0], 1, "{tally:?}");
    assert_eq!(tally.controlled_max[1], 0, "{tally:?}");
}

// A house's gattling types are kept per house, and a tick counts once per
// house however many of its gattlings stand at the top stage (S-0609
// Terms and claim 2).
#[test]
fn counts_gattling_types_and_top_stage_ticks() {
    let mut r = fixture::ruleset();
    let mut c = r.sim.vehicles["GUNR"].clone();
    c.primary = None;
    c.tech_level = Some(99);
    c.turret_count = Some(1);
    c.weapon_count = Some(6);
    c.combat.weapon_slots = Some(vec!["Gun".into(); 6]);
    c.combat.is_gattling = Some(true);
    c.combat.weapon_stages = Some(3);
    c.combat.rate_up = Some(5);
    c.combat.rate_down = Some(1);
    c.combat.stage_thresholds = Some(vec![10, 30, 60]);
    r.sim.vehicles.insert("GATG".into(), c);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("GATG".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    let a = testkit::spawn(&mut g.world, Kind::Vehicle, "GATG", 0, fixture::at(4, 4), 0);
    let b = testkit::spawn(&mut g.world, Kind::Vehicle, "GATG", 0, fixture::at(5, 4), 0);
    tally.observe(&g.world);
    assert_eq!(tally.gattling_types[0].len(), 1, "{tally:?}");
    assert_eq!(tally.gattling_types[1].len(), 0, "{tally:?}");
    assert_eq!(tally.top_ticks, [0, 0]);
    for id in [a, b] {
        let o = g.world.obj_mut(id).unwrap();
        o.combat.spin = 60;
        o.combat.gattling_stage = 2;
    }
    tally.observe(&g.world);
    tally.observe(&g.world);
    assert_eq!(tally.top_ticks, [2, 0], "{tally:?}");
}

// spec: S-0613 claims 2 to 4 (gas clouds: a warhead's particle system and a virus death's animation appear), S-0107 claim 17 (the virus death, infantry-death kind 8): the lane counts virus units per house, clouds that appear and the most gas particles at once.
#[test]
fn counts_virus_units_and_gas_clouds() {
    use glasnost_ai::fixture_m7a as kit;
    // A rifleman whose weapon fires the duel's virus-death warhead.
    let mut r = kit::ruleset();
    let mut gun = r.sim.weapons["SPIN2"].clone();
    gun.range = Some(glasnost_rules::sim::Decimal("4".into()));
    r.sim.weapons.insert("VENOMGUN".into(), gun);
    let mut virus = r.sim.infantry["WALKER"].clone();
    virus.primary = Some("VENOMGUN".into());
    r.sim.infantry.insert("VIRUS".into(), virus);
    r.registries
        .entry("infantry".into())
        .or_default()
        .push("VIRUS".into());
    let mut g = Game::new(
        std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, kit::MAP_NAME)),
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(2),
    )
    .expect("the synthetic M7a duel loads");
    let mut tally = Tally::default();
    g.step();
    tally.observe(&g.world);
    assert_eq!(tally.clouds, 0, "the shared gas system is not a new cloud");
    assert_eq!(tally.virus_armed[0].len(), 0);
    testkit::spawn(
        &mut g.world,
        Kind::Infantry,
        "VIRUS",
        0,
        fixture::at(4, 4),
        0,
    );
    let p = glasnost_sim::world::Pos::centre(fixture::at(6, 6), 0);
    glasnost_sim::gas::warhead_system(&mut g.world, "VAPOUR", p, Some(HouseId(0)));
    tally.observe(&g.world);
    tally.observe(&g.world);
    assert_eq!(tally.virus_armed[0].len(), 1, "{tally:?}");
    assert_eq!(tally.virus_armed[1].len(), 0, "{tally:?}");
    assert_eq!(tally.clouds, 1, "{tally:?}");
    assert_eq!(tally.gas_max, 1, "{tally:?}");
}

// An entry into an absorbing building is counted once, when the passenger
// goes in; a passenger let out beside the standing building counts as a
// release; the most passengers a house holds at once and its absorbing
// buildings are kept per house (S-0605 claims 7, 11 and 13).
#[test]
fn counts_absorber_entries_and_releases() {
    use glasnost_rules::sim::Decimal;
    let mut r = fixture::ruleset();
    let mut b = fixture::building(2, 2, 600, 50);
    b.infantry_absorb = Some(true);
    b.passengers = Some(3);
    b.size_limit = Some(Decimal("1".into()));
    b.econ.extra_power = Some(20);
    b.tech_level = Some(99);
    r.sim.buildings.insert("BIOR".into(), b);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("BIOR".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    let bio = testkit::spawn(
        &mut g.world,
        Kind::Building,
        "BIOR",
        0,
        fixture::at(0, 6),
        0,
    );
    let u = testkit::spawn(
        &mut g.world,
        Kind::Infantry,
        "WALKER",
        0,
        fixture::at(4, 6),
        0,
    );
    for _ in 0..3 {
        g.step();
    }
    tally.observe(&g.world);
    assert_eq!(tally.absorbers[0].len(), 1, "{tally:?}");
    g.world.houses[0].pending.clear();
    glasnost_sim::orders::issue(
        &mut g.world,
        HouseId(0),
        Order::Behaviour {
            actor: u,
            behaviour: glasnost_sim::rules::Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Object(bio)),
        },
    );
    for _ in 0..900 {
        glasnost_sim::tick::step(&mut g.world);
        tally.observe(&g.world);
        if glasnost_sim::structures::absorb::absorbed_in(&g.world, u).is_some() {
            break;
        }
    }
    tally.observe(&g.world);
    assert_eq!(tally.absorbed, 1, "{tally:?}");
    assert_eq!(tally.passengers_max[0], 1, "{tally:?}");
    glasnost_sim::orders::issue(&mut g.world, HouseId(0), Order::Deploy { actor: bio });
    for _ in 0..300 {
        glasnost_sim::tick::step(&mut g.world);
        tally.observe(&g.world);
    }
    assert_eq!(tally.absorbed, 1, "{tally:?}");
    assert_eq!(tally.absorber_releases, 1, "{tally:?}");
}

// A recycling building is kept per house once seen (S-0603 Terms: a
// building whose type has the grinding flag).
#[test]
fn counts_recycling_buildings() {
    let mut r = fixture::ruleset();
    let mut b = fixture::building(2, 2, 400, 0);
    b.grinding = Some(true);
    b.tech_level = Some(99);
    r.sim.buildings.insert("GRND".into(), b);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("GRND".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    tally.observe(&g.world);
    assert!(tally.grinders[0].is_empty(), "{tally:?}");
    testkit::spawn(
        &mut g.world,
        Kind::Building,
        "GRND",
        0,
        fixture::at(0, 6),
        0,
    );
    g.step();
    tally.observe(&g.world);
    assert_eq!(tally.grinders[0].len(), 1, "{tally:?}");
    assert!(tally.grinders[1].is_empty(), "{tally:?}");
}

// A vehicle that becomes bunkered is counted once; a link that ends with
// the vehicle and the bunker standing counts as a release; tank bunkers are
// kept per house (S-0606 claims 7 and 12).
#[test]
fn counts_bunkered_vehicles_and_releases() {
    let mut r = fixture::ruleset();
    let mut b = fixture::building(2, 2, 500, 0);
    b.bunker = Some(true);
    b.number_impassable_rows = Some(0);
    b.tech_level = Some(99);
    r.sim.buildings.insert("BUNK".into(), b);
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("BUNK".into());
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    let bunk = testkit::spawn(
        &mut g.world,
        Kind::Building,
        "BUNK",
        0,
        fixture::at(0, 6),
        0,
    );
    let v = testkit::spawn(&mut g.world, Kind::Vehicle, "GUNR", 0, fixture::at(5, 7), 0);
    for _ in 0..3 {
        g.step();
    }
    tally.observe(&g.world);
    assert_eq!(tally.bunkers[0].len(), 1, "{tally:?}");
    g.world.houses[0].pending.clear();
    glasnost_sim::orders::issue(
        &mut g.world,
        HouseId(0),
        Order::Behaviour {
            actor: v,
            behaviour: glasnost_sim::rules::Behaviour::Enter,
            attack_move: false,
            target: None,
            dest: Some(glasnost_sim::world::Target::Object(bunk)),
        },
    );
    for _ in 0..1500 {
        glasnost_sim::tick::step(&mut g.world);
        tally.observe(&g.world);
        if glasnost_sim::structures::bunker::bunkered(&g.world, v) {
            break;
        }
    }
    tally.observe(&g.world);
    assert_eq!(tally.bunkered, 1, "{tally:?}");
    glasnost_sim::orders::issue(&mut g.world, HouseId(0), Order::Deploy { actor: v });
    for _ in 0..100 {
        glasnost_sim::tick::step(&mut g.world);
        tally.observe(&g.world);
    }
    assert_eq!(tally.bunkered, 1, "{tally:?}");
    assert_eq!(tally.bunker_releases, 1, "{tally:?}");
}

// A unit that becomes seized is counted once, and once more when it is
// dropped (S-0607 claims 5 and 13), on the synthetic M7a skirmish's lifter
// (the base scenario: the gate's start lifter and prey would seize too).
#[test]
fn counts_seized_and_dropped_units() {
    use glasnost_ai::fixture_m7a as kit;
    use glasnost_sim::world::Target;
    let mut g = Game::new(
        kit::base_rules(),
        &kit::base_map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(5),
    )
    .expect("the M7a skirmish loads");
    g.step();
    let mut tally = Tally::default();
    let l = testkit::spawn(&mut g.world, Kind::Vehicle, "MAGN", 0, fixture::at(0, 6), 0);
    let v = testkit::spawn(&mut g.world, Kind::Vehicle, "GUNR", 1, fixture::at(5, 6), 0);
    g.world.houses[0].pending.clear();
    glasnost_sim::orders::issue(
        &mut g.world,
        HouseId(0),
        Order::Behaviour {
            actor: l,
            behaviour: glasnost_sim::rules::Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(v)),
            dest: None,
        },
    );
    let mut lifted = false;
    for _ in 0..600 {
        glasnost_sim::tick::step(&mut g.world);
        tally.observe(&g.world);
        if glasnost_sim::lift::grips(&g.world, l, v) && g.world.height_above_surface(v) > 200 {
            lifted = true;
            break;
        }
    }
    assert!(lifted, "{tally:?}");
    assert_eq!((tally.seized, tally.dropped), (1, 0), "{tally:?}");
    let c = g.world.obj(l).unwrap().pos.cell();
    glasnost_sim::orders::issue(
        &mut g.world,
        HouseId(0),
        Order::Behaviour {
            actor: l,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(glasnost_map::CellXY::new(c.x, c.y + 1))),
        },
    );
    for _ in 0..5 {
        glasnost_sim::tick::step(&mut g.world);
        tally.observe(&g.world);
    }
    assert_eq!((tally.seized, tally.dropped), (1, 1), "{tally:?}");
}

// Particle systems that appear are counted by behaviour kind, and an
// effect weapon's emitter by its place (S-0616 claim 1).
#[test]
fn counts_particle_systems_by_kind_and_emitters() {
    use glasnost_rules::sim::{ParticleSystemType, ParticleType, WeaponType};
    use glasnost_sim::world::{Pos, Target};
    let mut r = testkit::combat_ruleset();
    let reg = |r: &mut glasnost_rules::schema::Ruleset, l: &str, n: &str| {
        r.registries.entry(l.into()).or_default().push(n.into());
    };
    reg(&mut r, "particles", "EMBER");
    r.sim.particles.insert(
        "EMBER".into(),
        ParticleType {
            behaves_like: Some("fire".into()),
            max_ec: Some(100),
            ..Default::default()
        },
    );
    for (n, k) in [("BLAZE", "fire"), ("PLUME", "smoke")] {
        reg(&mut r, "particle_systems", n);
        r.sim.particle_systems.insert(
            n.into(),
            ParticleSystemType {
                behaves_like: Some(k.into()),
                holds_what: Some("EMBER".into()),
                ..Default::default()
            },
        );
    }
    r.sim.weapons.insert(
        "TORCH".into(),
        WeaponType {
            damage: Some(1),
            rof: Some(30),
            range: Some(glasnost_rules::sim::Decimal("10".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("SA".into()),
            use_fire_particles: Some(true),
            attached_particle_system: Some("BLAZE".into()),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("CART").unwrap().primary = Some("TORCH".into());
    let mut w = testkit::game_with(&r, 24, 7);
    let mut tally = Tally::default();
    let a = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "CART",
        0,
        testkit::mid(24, -2, 0),
        16384,
    );
    let b = testkit::spawn(
        &mut w,
        Kind::Vehicle,
        "BUGGY",
        1,
        testkit::mid(24, 2, 0),
        49152,
    );
    tally.observe(&w);
    glasnost_sim::gas::warhead_system(
        &mut w,
        "PLUME",
        Pos::centre(testkit::mid(24, 0, 3), 0),
        None,
    );
    assert!(glasnost_sim::combat::fire::shot(
        &mut w,
        a,
        0,
        Target::Object(b)
    ));
    tally.observe(&w);
    assert_eq!(tally.systems_by_kind.get("smoke"), Some(&1), "{tally:?}");
    assert_eq!(tally.systems_by_kind.get("fire"), Some(&1), "{tally:?}");
    assert_eq!(tally.emitters, [1, 0, 0]);
    assert!(tally.particles_max >= 1);
}

// spec: S-0615 claims 4, 12, 14 and 16 (the lane counts burning walkers as they appear, and disc-laser shots, final-step hits and cancellations from their events)
#[test]
fn counts_burning_walkers_and_disc_lasers() {
    use glasnost_sim::combat::effects::{DiscEvent, Effect};
    let mut r = fixture::ruleset();
    r.registries
        .entry("animations".into())
        .or_default()
        .push("BURN".into());
    r.sim.animations.insert(
        "BURN".into(),
        glasnost_rules::sim::AnimationType {
            frame_count: Some(80),
            is_flaming_guy: Some(true),
            running_frames: Some(4),
            ..Default::default()
        },
    );
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&r, "synthetic-tally"));
    let mut g = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        fixture::catalogue(),
        fixture::setup(5),
    )
    .expect("the synthetic skirmish loads");
    let mut tally = Tally::default();
    g.step();
    tally.observe(&g.world);
    assert_eq!(tally.walkers, 0);
    let anim = g
        .world
        .rules()
        .animations
        .iter()
        .position(|a| a.name == "BURN")
        .unwrap() as u32;
    let at = glasnost_sim::world::Pos::centre(fixture::at(3, 3), 0);
    glasnost_sim::walker::create(&mut g.world, anim, at);
    glasnost_sim::walker::create(&mut g.world, anim, at);
    tally.observe(&g.world);
    tally.observe(&g.world);
    assert_eq!(tally.walkers, 2, "each walker once");
    let disc = |event| Effect::Disc {
        firer: glasnost_sim::world::ObjectId(1),
        weapon: 0,
        event,
        pos: at,
    };
    for e in [
        disc(DiscEvent::Shot),
        disc(DiscEvent::ChargeUp { sound: None }),
        disc(DiscEvent::Shot),
        disc(DiscEvent::Cancelled),
        disc(DiscEvent::Hit { report: 0 }),
    ] {
        tally.effect(&e);
    }
    assert_eq!(
        (tally.disc_shots, tally.disc_hits, tally.disc_cancelled),
        (2, 1, 1)
    );
}

// spec: S-0608 claims 1, 5, 7 and 14 (the lane counts drain links as they form and end, the credits payouts move, and the drained buildings per tick)
#[test]
fn counts_drain_links_and_credits_drained() {
    use glasnost_ai::fixture_m7b as kit;
    let mut g = Game::new(
        kit::rules(),
        &kit::map(),
        kit::MAP_NAME,
        fixture::catalogue(),
        kit::setup(11),
    )
    .expect("the M7b skirmish loads");
    let mut tally = Tally::default();
    let mut paid = 0;
    // The AI's opening holds its first wave back (M8 step A4 round 8), so
    // the first link forms later than tick 3000.
    while g.world.tick < 12000 && paid < 3 {
        g.step();
        for e in &g.world.effects {
            tally.effect(e);
            if let glasnost_sim::combat::effects::Effect::Drain {
                event: glasnost_sim::drain::DrainEvent::Paid { .. },
                ..
            } = e
            {
                paid += 1;
            }
        }
        g.world.effects.clear();
        tally.observe(&g.world);
    }
    assert!(tally.drain_links >= 1, "a disc linked");
    assert!(tally.drained_ticks >= 1);
    assert_eq!(paid, 3, "three payouts within the run");
    assert!(tally.credits_drained > 0, "the payouts moved credits");
}

// spec: S-0611 claims 2, 8 and 9 to 12 (the lane counts each launch by its activation sound, each infantry-making animation made, and each unit a conversion animation's end made)
#[test]
fn counts_mutator_launches_and_infantry_mutated() {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::mutation::MakerEvent;
    use glasnost_sim::superweapons::SwEvent;
    use glasnost_sim::world::{ObjectId, Pos};
    let mut tally = Tally::default();
    let maker = |event| Effect::Maker {
        id: ObjectId(7),
        name: "MUTANIM".into(),
        pos: Pos::default(),
        owner: Some(HouseId(0)),
        frame: 0,
        event,
    };
    for e in [
        Effect::Superweapon {
            event: SwEvent::GeneticSound,
            house: Some(HouseId(0)),
            sw: Some(0),
            pos: None,
        },
        Effect::Superweapon {
            event: SwEvent::Launch,
            house: Some(HouseId(0)),
            sw: Some(0),
            pos: None,
        },
        maker(MakerEvent::Made),
        maker(MakerEvent::Made),
        maker(MakerEvent::Refused),
        maker(MakerEvent::Ended {
            unit: Some(ObjectId(9)),
            conversion: true,
        }),
        maker(MakerEvent::Ended {
            unit: None,
            conversion: true,
        }),
        maker(MakerEvent::Ended {
            unit: Some(ObjectId(10)),
            conversion: false,
        }),
    ] {
        tally.effect(&e);
    }
    assert_eq!(
        (
            tally.mutator_launches,
            tally.conversions,
            tally.infantry_mutated
        ),
        (1, 2, 1)
    );
}

// spec: S-0612 claims 1, 9 and 14 (the lane counts each launch by its activation sound, each object a sweep takes over, and the objects the sweep's area damage hurt and destroyed)
#[test]
fn counts_dominator_launches_takeovers_and_damage() {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::domination::DomEvent;
    use glasnost_sim::superweapons::SwEvent;
    use glasnost_sim::world::{ObjectId, Pos};
    let mut tally = Tally::default();
    let dom = |event| Effect::Domination {
        event,
        house: Some(HouseId(0)),
        pos: Pos::default(),
    };
    for e in [
        Effect::Superweapon {
            event: SwEvent::DominatorSound,
            house: Some(HouseId(0)),
            sw: Some(0),
            pos: None,
        },
        Effect::Superweapon {
            event: SwEvent::Launch,
            house: Some(HouseId(0)),
            sw: Some(0),
            pos: None,
        },
        dom(DomEvent::Started),
        dom(DomEvent::Sweep),
        dom(DomEvent::Damage {
            damaged: 4,
            killed: 2,
        }),
        dom(DomEvent::Taken(ObjectId(5))),
        dom(DomEvent::Taken(ObjectId(6))),
        dom(DomEvent::Taken(ObjectId(7))),
        dom(DomEvent::Final),
        dom(DomEvent::Over),
    ] {
        tally.effect(&e);
    }
    assert_eq!(
        (
            tally.dominator_launches,
            tally.dominated,
            tally.dominator_damaged,
            tally.dominator_killed
        ),
        (1, 3, 4, 2)
    );
    // One sweep; with no owners observed before it, no takeover counts as an owner change (S-0612 claim 14).
    assert_eq!(tally.dominator_sweeps, vec![0]);
}
