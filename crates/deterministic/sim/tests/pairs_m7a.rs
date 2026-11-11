//! Interaction-pair scenario tests of M7a (PLAN, cross-component
//! interactions): each pair runs through orders and the fixed tick pipeline
//! on synthetic data (invented types, weapons and warheads, every number our
//! own). Houses Alpha (0) and Beta (1) of the testkit's sandbox game, not
//! allied.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::{Effect, Sound};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::structures::bunker;
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{gattling, lift, mind, orders, save, testkit, tick, transport};

const SIZE: i32 = 24;
const ALPHA: u8 = 0;
const BETA: u8 = 1;

fn dec(s: &str) -> Decimal {
    Decimal(s.into())
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

/// The combat ruleset with the takeover warhead MIND, the weapon Ray3
/// (MIND, damage 3: room for three victims), infantry CLONE (Ray3) and
/// GRUNT (plain), vehicle CART (an open-topped transport for three), and
/// for the gattling pair GAT (a turreted gattling vehicle: six slots W0 …
/// W5 of damage 1 and rate 1, stage count 3, spin-up 7, spin-down 1,
/// thresholds 10, 30 and 60) and TGT (an unarmed vehicle of strength
/// 30000), and for the absorber pairs BIO (a 2 × 2 infantry absorber for
/// three, size limit 1, power 10, extra power 25), and for the grinder pairs
/// GRND (a 2 × 2 recycling building), GRUNT's cost 400, a refund percentage
/// of 0.5 and Betaland's infantry cost factor 0.5, and for the bunker pairs
/// BUNK (a 2 × 2 tank bunker) and GATT (GAT with the turret flag set,
/// which bunker eligibility reads), and for the magnetic-lift pairs MAGL
/// (a lifter vehicle) and DRONE (a parasite drone).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.warheads.insert(
        "MIND".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            mind_control: Some(true),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Ray3".into(),
        WeaponType {
            damage: Some(3),
            rof: Some(30),
            range: Some(dec("8")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("MIND".into()),
            ..Default::default()
        },
    );
    for (name, primary) in [("CLONE", Some("Ray3")), ("GRUNT", None)] {
        let mut t = r.sim.infantry["WALKER"].clone();
        t.strength = Some(1000);
        t.primary = primary.map(Into::into);
        register(&mut r, "infantry", name);
        r.sim.infantry.insert(name.into(), t);
    }
    let mut cart: UnitType = r.sim.vehicles["BUGGY"].clone();
    cart.strength = Some(1000);
    cart.primary = None;
    cart.passengers = Some(3);
    cart.size_limit = Some(dec("2"));
    cart.open_topped = Some(true);
    register(&mut r, "vehicles", "CART");
    r.sim.vehicles.insert("CART".into(), cart);
    let slots: Vec<String> = (0..6).map(|i| format!("W{i}")).collect();
    for n in &slots {
        r.sim.weapons.insert(
            n.clone(),
            WeaponType {
                damage: Some(1),
                rof: Some(1),
                range: Some(dec("10")),
                speed: Some(100),
                projectile: Some("Bullet".into()),
                warhead: Some("SA".into()),
                ..Default::default()
            },
        );
    }
    let mut tgt: UnitType = r.sim.vehicles["BUGGY"].clone();
    tgt.strength = Some(30000);
    tgt.primary = None;
    let mut gat = tgt.clone();
    gat.turret_count = Some(1);
    gat.weapon_count = Some(6);
    gat.combat.weapon_slots = Some(slots);
    gat.combat.is_gattling = Some(true);
    gat.combat.weapon_stages = Some(3);
    gat.combat.rate_up = Some(7);
    gat.combat.rate_down = Some(1);
    gat.combat.stage_thresholds = Some(vec![10, 30, 60]);
    for (n, t) in [("GAT", gat), ("TGT", tgt)] {
        register(&mut r, "vehicles", n);
        r.sim.vehicles.insert(n.into(), t);
    }
    let mut bio = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        infantry_absorb: Some(true),
        passengers: Some(3),
        size_limit: Some(dec("1")),
        ..Default::default()
    };
    bio.econ.power = Some(10);
    bio.econ.extra_power = Some(25);
    register(&mut r, "buildings", "BIO");
    r.sim.buildings.insert("BIO".into(), bio);
    let grnd = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        grinding: Some(true),
        ..Default::default()
    };
    register(&mut r, "buildings", "GRND");
    r.sim.buildings.insert("GRND".into(), grnd);
    // The bunker pairs (S-0606): BUNK, a 2 × 2 tank bunker with no
    // impassable rows, and GATT, GAT with the turret flag (claim 1).
    let bunk = BuildingType {
        strength: Some(1000),
        foundation: Some([2, 2]),
        bunker: Some(true),
        number_impassable_rows: Some(0),
        ..Default::default()
    };
    register(&mut r, "buildings", "BUNK");
    r.sim.buildings.insert("BUNK".into(), bunk);
    let mut gatt = r.sim.vehicles["GAT"].clone();
    gatt.turret = Some(true);
    register(&mut r, "vehicles", "GATT");
    r.sim.vehicles.insert("GATT".into(), gatt);
    // The magnetic-lift pairs (S-0607): MAGL, a vehicle whose weapon MagRay
    // (damage 50, a magnetic beam) has the seizing warhead LIFTWH (the
    // jumpjet locomotor), and the drone DRONE whose weapon Latch has the
    // parasite warhead Grip (S-0614).
    r.sim.warheads.insert(
        "LIFTWH".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            is_locomotor: Some(true),
            locomotor: Some("jumpjet".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "MagRay".into(),
        WeaponType {
            damage: Some(50),
            rof: Some(20),
            range: Some(dec("10")),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("LIFTWH".into()),
            is_mag_beam: Some(true),
            ..Default::default()
        },
    );
    let mut magl: UnitType = r.sim.vehicles["BUGGY"].clone();
    magl.strength = Some(1000);
    magl.primary = Some("MagRay".into());
    register(&mut r, "vehicles", "MAGL");
    r.sim.vehicles.insert("MAGL".into(), magl);
    r.sim
        .projectiles
        .insert("Hop".into(), glasnost_rules::sim::ProjectileType::default());
    r.sim.weapons.insert(
        "Latch".into(),
        WeaponType {
            damage: Some(10),
            rof: Some(15),
            range: Some(dec("6")),
            speed: Some(60),
            projectile: Some("Hop".into()),
            warhead: Some("Grip".into()),
            limbo_launch: Some(true),
            ..Default::default()
        },
    );
    r.sim.warheads.insert(
        "Grip".into(),
        WarheadType {
            verses: Some(vec![dec("1"); 11]),
            parasite: Some(true),
            paralyzes: Some(20),
            ..Default::default()
        },
    );
    let mut drone: UnitType = r.sim.vehicles["BUGGY"].clone();
    drone.strength = Some(100);
    drone.primary = Some("Latch".into());
    register(&mut r, "vehicles", "DRONE");
    r.sim.vehicles.insert("DRONE".into(), drone);
    r.sim.infantry.get_mut("GRUNT").unwrap().cost = Some(400);
    r.sim.general.economy.refund_percent = Some(dec("0.5"));
    r.sim
        .countries
        .get_mut("Betaland")
        .unwrap()
        .econ
        .cost_infantry_mult = Some(dec("0.5"));
    r
}

fn at(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, at(dx, dy), 0)
}

fn owner(w: &World, id: ObjectId) -> Option<u8> {
    w.obj(id).and_then(|o| o.owner).map(|h| h.0)
}

fn click(w: &mut World, house: u8, sel: &[ObjectId], on: ObjectId) {
    let r = targeting::click(
        w,
        HouseId(house),
        sel,
        ClickOn::Object(on),
        Modifiers::default(),
    );
    for o in r.orders {
        orders::issue(w, HouseId(house), o);
    }
}

fn run_until(w: &mut World, max: i32, done: impl Fn(&World) -> bool) -> bool {
    for _ in 0..max {
        if done(w) {
            return true;
        }
        tick::step(w);
    }
    done(w)
}

// Pair: a controlled unit and a transport.
// spec: S-0507 claims 4, 6 and 14 (a controller with no victims boards an open-topped transport; its passengers stay in play)
// spec: S-0213 table T3 (f) (an open-topped transport gives its target to its passengers)
// spec: S-0601 claims 5 and 10 and Terms (a takeover by passenger fire; a transport answers neither to a controlled unit or to a controller holding victims; the controller leaving the game frees its victim)
// spec: S-0312 claim 15 (a destroyed transport's passengers leave the game with it)
// spec: S-0601 State and ordering (the records survive a save/load round trip mid-scenario)
#[test]
fn a_controller_riding_an_open_transport_takes_and_loses_a_victim() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 0);
    let cart = put(&mut w, Kind::Vehicle, "CART", BETA, 2, 0);
    let grunt = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 9, 0);
    testkit::run(&mut w, 5);

    // The controller, holding no victims, boards by a click on the transport.
    let enter = targeting::cursor(
        &w,
        HouseId(BETA),
        &[clone],
        ClickOn::Object(cart),
        Modifiers::default(),
    );
    assert_eq!(enter, Action::Enter);
    click(&mut w, BETA, &[clone], cart);
    assert!(
        run_until(&mut w, 600, |w| transport::riding(w, clone) == Some(cart)),
        "the controller boards"
    );

    // The transport is ordered to attack; its passenger's takeover lands.
    orders::issue(
        &mut w,
        HouseId(BETA),
        orders::Order::Behaviour {
            actor: cart,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(grunt)),
            dest: None,
        },
    );
    assert!(
        run_until(&mut w, 900, |w| mind::mind_controlled(w, grunt)),
        "taken by passenger fire"
    );
    assert_eq!(owner(&w, grunt), Some(BETA));
    assert_eq!(
        mind::controller(&w, clone).map(|r| r.victims.clone()),
        Some(vec![grunt])
    );

    // Save and load with the controller inside and its victim held.
    let h = save::state_hash(&w);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);

    // The controlled unit may not board its new owner's transport.
    let on_cart = targeting::cursor(
        &w,
        HouseId(BETA),
        &[grunt],
        ClickOn::Object(cart),
        Modifiers::default(),
    );
    assert_ne!(on_cart, Action::Enter, "a controlled unit is refused");
    assert_eq!(
        transport::answer(&w, cart, grunt),
        transport::Answer::Neither
    );

    // The transport is destroyed with its passenger: the victim goes home.
    damage::apply(
        &mut w,
        cart,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    testkit::run(&mut w, 2);
    assert!(w.obj(cart).is_none() && w.obj(clone).is_none());
    assert_eq!(owner(&w, grunt), Some(ALPHA));
    assert!(!mind::mind_controlled(&w, grunt));
}

fn attack(w: &mut World, house: u8, actor: ObjectId, target: ObjectId) -> bool {
    orders::issue(
        w,
        HouseId(house),
        orders::Order::Behaviour {
            actor,
            behaviour: Behaviour::Attack,
            attack_move: false,
            target: Some(Target::Object(target)),
            dest: None,
        },
    )
}

// Pair: a gattling taken over by mind control while it fires.
// spec: S-0609 claims 1 and 4 and scenario G7 (d) (a gattling taken over part-way, its attack re-ordered at once: no drop of stage at the change of owner)
// spec: S-0601 claims 4, 5 and 6 (the controller's shot lands, the gattling changes owner and gets a guard reset)
#[test]
fn a_gattling_taken_over_while_firing_keeps_its_stage() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let gat = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    let tgt = put(&mut w, Kind::Vehicle, "TGT", BETA, 6, 0);
    let home = put(&mut w, Kind::Vehicle, "TGT", ALPHA, -6, 0);
    attack(&mut w, ALPHA, gat, tgt);
    assert!(
        run_until(&mut w, 600, |w| gattling::spin(w, gat).stage >= 1),
        "the gattling spins up"
    );

    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 6);
    attack(&mut w, BETA, clone, gat);
    let mut before = gattling::spin(&w, gat).stage;
    let mut taken = false;
    for _ in 0..900 {
        before = gattling::spin(&w, gat).stage;
        tick::step(&mut w);
        if mind::mind_controlled(&w, gat) {
            taken = true;
            break;
        }
    }
    assert!(taken, "the controller takes the gattling");
    assert_eq!(owner(&w, gat), Some(BETA));
    assert!(before >= 1);
    assert!(
        gattling::spin(&w, gat).stage >= before,
        "no drop of stage at the change of owner"
    );
    assert_ne!(
        w.obj(gat).unwrap().target,
        Some(Target::Object(tgt)),
        "the guard reset let go of its new house's object"
    );

    // Re-ordered at once against its former owner, it climbs to the top stage.
    attack(&mut w, BETA, gat, home);
    assert!(
        run_until(&mut w, 900, |w| gattling::spin(w, gat).stage == 2),
        "the top stage under the new owner"
    );
}

// Pair: a takeover in the tick of the former owner's order.
// spec: S-0213 claim 15 (a behaviour order is applied without an owner check: an actor taken over by mind control between the issue and the application still gets it)
// spec: S-0203 claim 7 (orders take effect after the turn pass)
// spec: S-0601 claim 6 (the takeover's guard reset comes first and the order then replaces it)
#[test]
fn an_order_of_the_former_owner_still_applies_in_the_takeover_tick() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let gat = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    let tgt = put(&mut w, Kind::Vehicle, "TGT", BETA, 6, 0);
    testkit::run(&mut w, 5);
    // spec: S-0213 claim 15 (a house's orders act only on its own objects: Alpha orders the gattling while it is Alpha's)
    assert!(
        attack(&mut w, ALPHA, gat, tgt),
        "Alpha orders its own object"
    );
    // The controller comes now, so that it has not taken the gattling by fire before the order.
    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 6);
    assert!(mind::take(&mut w, clone, gat));
    assert_eq!(w.obj(gat).unwrap().target, None, "the guard reset");
    tick::step(&mut w);
    assert_eq!(owner(&w, gat), Some(BETA));
    assert_eq!(
        w.obj(gat).unwrap().target,
        Some(Target::Object(tgt)),
        "Alpha's order reached Beta's new object"
    );
}

// Pair: a controlled unit and an absorbing building.
// spec: S-0605 claims 1, 6, 7, 10 and 11 to 13 (a controlled enemy is accepted and goes in; its controller releases it first, so it is inside as its original owner's unit; the building's owner gains the power; deploy lets it out as an enemy)
// spec: S-0601 claims 5 and 9 (the takeover by fire; the release on entry)
// spec: S-0214 table I row x3 and row B5 (enter cursor on the absorbing building; deploy on it with a passenger)
#[test]
fn a_controlled_enemy_goes_into_an_absorber_and_comes_out_an_enemy() {
    use glasnost_sim::structures::absorb;
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 0);
    let bio = put(&mut w, Kind::Building, "BIO", BETA, -4, -4);
    let grunt = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 6, 0);
    testkit::run(&mut w, 5);
    attack(&mut w, BETA, clone, grunt);
    assert!(
        run_until(&mut w, 900, |w| mind::mind_controlled(w, grunt)),
        "taken by fire"
    );
    assert_eq!(owner(&w, grunt), Some(BETA));
    let (out0, _) = glasnost_sim::econ::power::totals(&w, HouseId(BETA));

    // The controller's house sends its victim into its own absorber.
    let enter = targeting::cursor(
        &w,
        HouseId(BETA),
        &[grunt],
        ClickOn::Object(bio),
        Modifiers::default(),
    );
    assert_eq!(enter, Action::Enter);
    click(&mut w, BETA, &[grunt], bio);
    assert!(
        run_until(&mut w, 900, |w| absorb::absorbed_in(w, grunt) == Some(bio)),
        "goes in"
    );
    assert_eq!(
        owner(&w, grunt),
        Some(ALPHA),
        "inside as its original owner's unit"
    );
    assert!(!mind::mind_controlled(&w, grunt));
    assert!(mind::controller(&w, clone).is_none_or(|r| r.victims.is_empty()));
    testkit::run(&mut w, 3);
    let (out1, _) = glasnost_sim::econ::power::totals(&w, HouseId(BETA));
    assert_eq!(out1, out0 + 25, "the building's owner gains the power");

    // Save and load with the enemy inside.
    let h = save::state_hash(&w);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);

    // Deploy lets it out, still Alpha's.
    let deploy = targeting::cursor(
        &w,
        HouseId(BETA),
        &[bio],
        ClickOn::Object(bio),
        Modifiers::default(),
    );
    assert_eq!(deploy, Action::Deploy);
    click(&mut w, BETA, &[bio], bio);
    assert!(run_until(&mut w, 300, |w| absorb::count(w, bio) == 0));
    let o = w.obj(grunt).unwrap();
    assert!(o.on_map);
    assert_eq!(owner(&w, grunt), Some(ALPHA));
}

// Pair: the release fate and an absorbing building.
// spec: S-0601 claims 8 and 10 (the controller's destruction releases its victim; the neutral house's fate decision draws and selects the bio reactor)
// spec: S-0605 claims 1, 3, 6, 7 and 16 and S-0215 claim 24 (the released unit takes the enter-nearest instruction and enters its own owner's nearest accepting absorbing building)
#[test]
fn a_released_neutral_unit_takes_the_bio_reactor_fate() {
    use glasnost_sim::structures::absorb;
    let mut r = rules();
    let g = &mut r.sim.general;
    g.ai_capture_normal = Some(vec![0, 0, 100, 0]);
    g.ai_capture_wounded = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_power = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_money = Some(vec![0, 0, 0, 0]);
    // House 2 is the neutral house of `game_with_non_players`.
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, -6, 0);
    let bio = put(&mut w, Kind::Building, "BIO", 2, 6, -4);
    let civ = put(&mut w, Kind::Infantry, "GRUNT", 2, 1, 0);
    testkit::run(&mut w, 5);
    attack(&mut w, BETA, clone, civ);
    assert!(
        run_until(&mut w, 900, |w| mind::mind_controlled(w, civ)),
        "taken by fire"
    );
    // The controller dies: its victim goes home and the fate sends it in.
    damage::apply(
        &mut w,
        clone,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    testkit::run(&mut w, 2);
    assert_eq!(owner(&w, civ), Some(2));
    assert!(
        run_until(&mut w, 900, |w| absorb::absorbed_in(w, civ) == Some(bio)),
        "the bio-reactor fate"
    );
    let h = save::state_hash(&w);
    let l = save::load(&save::save(&w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
}

fn credits(w: &World, h: u8) -> i32 {
    w.houses[usize::from(h)].credits
}

fn round_trip(w: &World) {
    let h = save::state_hash(w);
    let l = save::load(&save::save(w), w.rules_arc(), testkit::catalogue()).unwrap();
    assert_eq!(save::state_hash(&l), h);
}

// Pair: a controlled unit and a recycling building.
// spec: S-0603 claims 1, 5, 6, 8, 9 and 11 and F1 (the controlling house recycle-clicks its victim into its own grinder: consumed as the controlling house's object, valued with that house's factors and paid to it; the original owner gets nothing back; removal, not death)
// spec: S-0601 claims 5 and 12 (the takeover by fire; a victim leaving the game leaves its controller's list silently, no release sound, the slot free for a new takeover)
// spec: S-0214 table I row I26 (recycle cursor on the house's own recycling building)
#[test]
fn a_controlled_enemy_is_ground_for_its_controllers_house() {
    use glasnost_sim::combat::effects::Effect;
    use glasnost_sim::econ::{TypeRef, sell};
    use glasnost_sim::mind::MindSound;
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 0);
    let grnd = put(&mut w, Kind::Building, "GRND", BETA, -5, -5);
    let grunt = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 6, 0);
    testkit::run(&mut w, 5);
    attack(&mut w, BETA, clone, grunt);
    assert!(
        run_until(&mut w, 900, |w| mind::mind_controlled(w, grunt)),
        "taken by fire"
    );
    assert_eq!(owner(&w, grunt), Some(BETA));
    let recycle = targeting::cursor(
        &w,
        HouseId(BETA),
        &[grunt],
        ClickOn::Object(grnd),
        Modifiers::default(),
    );
    assert_eq!(recycle, Action::Recycle);
    click(&mut w, BETA, &[grunt], grnd);
    testkit::run(&mut w, 2);
    assert_eq!(w.obj(grunt).unwrap().dest, Some(Target::Object(grnd)));
    // Save and load while it is bound for the grinder.
    round_trip(&w);
    let t = TypeRef {
        kind: Kind::Infantry,
        index: w.rules().index_of(Kind::Infantry, "GRUNT").unwrap(),
    };
    let beta_value = sell::refund(&w, HouseId(BETA), t);
    // Beta's infantry factor: trunc(trunc(400 × 0.5) × 0.5) = 100, not Alpha's 200.
    assert_eq!(beta_value, 100);
    assert_eq!(sell::refund(&w, HouseId(ALPHA), t), 200);
    let (ca, cb) = (credits(&w, ALPHA), credits(&w, BETA));
    let stats = |w: &World| {
        [ALPHA, BETA].map(|h| {
            let s = &w.houses[usize::from(h)].econ.result.stats;
            (s.kills, s.losses)
        })
    };
    let s0 = stats(&w);
    let mut fx = Vec::new();
    for _ in 0..900 {
        if w.obj(grunt).is_none() {
            break;
        }
        w.effects.clear();
        tick::step(&mut w);
        fx.extend(w.effects.iter().cloned());
    }
    assert!(w.obj(grunt).is_none(), "consumed");
    assert_eq!(
        credits(&w, BETA) - cb,
        beta_value,
        "the controller's house is paid"
    );
    assert_eq!(credits(&w, ALPHA), ca, "the original owner gets nothing");
    assert_eq!(stats(&w), s0, "no kill, no loss");
    assert!(mind::controller(&w, clone).is_none_or(|r| r.victims.is_empty()));
    assert!(
        !fx.iter().any(|e| matches!(
            e,
            Effect::Mind {
                sound: MindSound::Cleared,
                ..
            }
        )),
        "no release sound"
    );
    assert!(
        fx.iter()
            .any(|e| matches!(e, Effect::Consumed { grinding: true, .. }))
    );
    // The freed slot takes a new victim.
    let next = put(&mut w, Kind::Infantry, "GRUNT", ALPHA, 4, 3);
    testkit::run(&mut w, 2);
    attack(&mut w, BETA, clone, next);
    assert!(
        run_until(&mut w, 900, |w| mind::mind_controlled(w, next)),
        "takes again"
    );
    round_trip(&w);
}

// Pair: the release fate and a recycling building.
// spec: S-0601 claims 8 and 10 (the controller's destruction releases its victim; the neutral house's fate decision draws and selects the grinder)
// spec: S-0603 claims 3, 5, 8 and 14 and F1 (the released unit walks to its own owner's nearest recycling building and is consumed; the neutral house is paid P without the refund percentage)
#[test]
fn a_released_neutral_unit_takes_the_grinder_fate() {
    let mut r = rules();
    let g = &mut r.sim.general;
    g.ai_capture_normal = Some(vec![0, 100, 0, 0]);
    g.ai_capture_wounded = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_power = Some(vec![0, 0, 0, 0]);
    g.ai_capture_low_money = Some(vec![0, 0, 0, 0]);
    let mut w = testkit::game_with_non_players(&r, SIZE, 1);
    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, -6, 0);
    let grnd = put(&mut w, Kind::Building, "GRND", 2, 6, -4);
    let civ = put(&mut w, Kind::Infantry, "GRUNT", 2, 1, 0);
    testkit::run(&mut w, 5);
    attack(&mut w, BETA, clone, civ);
    assert!(
        run_until(&mut w, 900, |w| mind::mind_controlled(w, civ)),
        "taken by fire"
    );
    damage::apply(
        &mut w,
        clone,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    testkit::run(&mut w, 2);
    assert_eq!(owner(&w, civ), Some(2));
    assert_eq!(
        w.obj(civ).unwrap().dest,
        Some(Target::Object(grnd)),
        "the grinder fate"
    );
    round_trip(&w);
    let c0 = credits(&w, 2);
    assert!(run_until(&mut w, 900, |w| w.obj(civ).is_none()), "consumed");
    assert_eq!(credits(&w, 2) - c0, 400, "P for the neutral house");
}

// Pairs with a tank bunker (S-0606).

/// Sends vehicle `v` into bunker `b` with its owner's enter click and runs
/// until it is bunkered.
fn bunker_in(w: &mut World, v: ObjectId, b: ObjectId) {
    let house = owner(w, v).unwrap();
    assert_eq!(
        targeting::cursor(
            w,
            HouseId(house),
            &[v],
            ClickOn::Object(b),
            Modifiers::default()
        ),
        Action::Enter
    );
    testkit::run(w, 5);
    click(w, house, &[v], b);
    assert!(
        run_until(w, 1500, |w| bunker::bunkered_in(w, v) == Some(b)),
        "bunkered"
    );
}

/// The spin value's change in each tick from the first spin-up of `g`
/// (attacking a target in range) to its top stage.
fn climb(w: &mut World, g: ObjectId) -> Vec<i32> {
    let mut steps = Vec::new();
    let mut last = gattling::spin(w, g).value;
    for _ in 0..900 {
        tick::step(w);
        let s = gattling::spin(w, g);
        if s.value != last || !steps.is_empty() {
            steps.push(s.value - last);
        }
        last = s.value;
        if s.stage == 2 {
            return steps;
        }
    }
    panic!("no top stage");
}

// Pair: a gattling vehicle in a tank bunker.
// spec: S-0609 claim 5 (being bunkered adds no firing-step exclusion or delay: the same per-tick spin rule applies) and its bunker variant (spin updates compared with an open-ground control of the same type; shot ticks may differ)
// spec: S-0606 claims 1, 7 and 9 (a turreted, armed gattling vehicle is bunker-eligible; bunkered, it fires from its bunker at the bunker rate and range)
#[test]
fn a_gattling_in_a_bunker_spins_up_as_in_the_open() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let b = put(&mut w, Kind::Building, "BUNK", ALPHA, 0, 0);
    let g = put(&mut w, Kind::Vehicle, "GATT", ALPHA, 6, 1);
    bunker_in(&mut w, g, b);
    let tgt = put(&mut w, Kind::Vehicle, "TGT", BETA, -4, 1);
    attack(&mut w, ALPHA, g, tgt);
    let inside = climb(&mut w, g);
    assert_eq!(bunker::bunkered_in(&w, g), Some(b), "still bunkered");
    assert_eq!(w.obj(g).unwrap().target, Some(Target::Object(tgt)));
    round_trip(&w);

    // The open-ground control: the same type and distance, no bunker.
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let g = put(&mut w, Kind::Vehicle, "GATT", ALPHA, 1, 1);
    let tgt = put(&mut w, Kind::Vehicle, "TGT", BETA, -4, 1);
    testkit::run(&mut w, 5);
    attack(&mut w, ALPHA, g, tgt);
    let open = climb(&mut w, g);
    // One spin-up of 7 in every tick until the top stage, in the bunker as
    // in the open: no missing or delayed update (the starting value may
    // differ: the bunkered gattling may already have spun up in guard).
    for (name, steps) in [("bunker", &inside), ("open", &open)] {
        assert!(
            !steps.is_empty() && steps.iter().all(|&d| d == 7),
            "{name}: {steps:?}"
        );
    }
}

// Pair: a controlled vehicle goes into its controller's bunker.
// spec: S-0601 claims 3 and 6 (a takeover makes the vehicle the controlling house's; the bunker row of claim 3 refuses taking over a vehicle in a tank bunker, not one that goes in after the takeover)
// spec: S-0606 claims 2, 3 and 7 (the controlling house's enter click on its own bunker: permitted, the vehicle drives in and is bunkered)
#[test]
fn a_controlled_vehicle_goes_into_its_controllers_bunker() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let b = put(&mut w, Kind::Building, "BUNK", ALPHA, 0, 0);
    let g = put(&mut w, Kind::Vehicle, "GATT", BETA, 6, 1);
    let clone = put(&mut w, Kind::Infantry, "CLONE", ALPHA, 6, 4);
    testkit::run(&mut w, 5);
    assert!(mind::take(&mut w, clone, g));
    tick::step(&mut w);
    assert_eq!(owner(&w, g), Some(ALPHA));
    bunker_in(&mut w, g, b);
    assert!(mind::mind_controlled(&w, g), "still controlled inside");
    assert_eq!(owner(&w, g), Some(ALPHA));
    assert_eq!(bunker::occupant(&w, b), Some(g));
    round_trip(&w);
    // The bunker row of S-0601 claim 3 on an uncontrolled vehicle: Beta's
    // controller may take Alpha's vehicle outside, not once it is bunkered.
    let b2 = put(&mut w, Kind::Building, "BUNK", ALPHA, -6, -6);
    let plain = put(&mut w, Kind::Vehicle, "GATT", ALPHA, -6, -2);
    let other = put(&mut w, Kind::Infantry, "CLONE", BETA, -8, 10);
    assert!(mind::can_take(&w, other, plain), "outside: legal");
    bunker_in(&mut w, plain, b2);
    assert!(!mind::can_take(&w, other, plain), "bunkered: refused");
    round_trip(&w);
}

// Pair: a bunkered vehicle caught by a teleport warp.
// spec: S-0513 claim 34 (a bunkered vehicle joins the warp group: its tank-bunker link ends in place with the walls-down effects; the bunker goes back to clearing)
// spec: S-0606 claim 14 (the link ends in place: walls-down effects, the bunker at clearing with guard queued)
#[test]
fn a_warp_takes_a_bunkered_vehicle_out_of_its_bunker() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let b = put(&mut w, Kind::Building, "BUNK", ALPHA, 0, 0);
    let g = put(&mut w, Kind::Vehicle, "GATT", ALPHA, 6, 1);
    bunker_in(&mut w, g, b);
    let from = w.obj(g).unwrap().pos.cell();
    w.effects.clear();
    glasnost_sim::superweapons::warp::launch(&mut w, HouseId(BETA), from, at(0, 8));
    assert!(!bunker::bunkered(&w, g), "the link ends in place");
    assert_eq!(bunker::occupant(&w, b), None);
    assert_eq!(
        w.obj(b).unwrap().structure.bunker_stage,
        bunker::Stage::Clearing
    );
    assert_eq!(w.obj(b).unwrap().beh.queued, Some(Behaviour::Guard));
    assert!(w.effects.iter().any(|e| matches!(
        e,
        Effect::Sound {
            sound: Sound::BunkerWallsDown,
            ..
        }
    )));
    assert!(w.obj(g).unwrap().sw.warp.is_some(), "in the warp group");
    round_trip(&w);
    assert!(run_until(&mut w, 600, |w| w.obj(g).is_some_and(|o| o
        .sw
        .warp
        .is_none()
        && o.on_map)));
    assert!(!bunker::bunkered(&w, g));
    round_trip(&w);
}

// Pair: a berserk weapon against a bunkered vehicle and its bunker.
// spec: S-0610 claim 3 (a berserk-causing weapon is illegal against a vehicle inside a tank bunker, or a tank bunker that holds a vehicle)
// spec: S-0606 Terms (the bunker link)
#[test]
fn a_berserk_weapon_is_refused_at_a_bunkered_vehicle_and_its_bunker() {
    // A bunker without psionics immunity (a building's default has it, S-0610
    // Import mapping), so that only the bunker link refuses.
    let mut r = rules();
    r.sim
        .buildings
        .get_mut("BUNK")
        .unwrap()
        .combat
        .immune_to_psionics = Some(false);
    let mut w = testkit::game_with(&r, SIZE, 1);
    let b = put(&mut w, Kind::Building, "BUNK", ALPHA, 0, 0);
    let g = put(&mut w, Kind::Vehicle, "GATT", ALPHA, 6, 1);
    let firer = put(&mut w, Kind::Infantry, "GRUNT", BETA, -6, 6);
    testkit::run(&mut w, 5);
    let refused = |w: &World, x: ObjectId| {
        glasnost_sim::psychic::firing_refused(w, firer, true, Target::Object(x))
    };
    assert!(
        !refused(&w, g) && !refused(&w, b),
        "outside, and an empty bunker"
    );
    bunker_in(&mut w, g, b);
    assert!(refused(&w, g) && refused(&w, b));
    round_trip(&w);
}

// Pairs with a magnetic lifter (S-0607).

/// Attack-orders lifter `l` at `v` and runs until `v` is seized.
fn lift_up(w: &mut World, l: ObjectId, v: ObjectId) {
    let house = owner(w, l).unwrap();
    attack(w, house, l, v);
    assert!(run_until(w, 600, |w| lift::seized(w, v)), "seized");
    assert!(lift::grips(w, l, v));
}

// Pair: a lifter and a mind-controlled vehicle.
// spec: S-0607 claims 3, 5, 12, 13 and 18 (a controlled vehicle is not immune; dropped over a vehicle when its lifter takes an order, its impact removes it)
// spec: S-0601 claim 12 (a victim leaving the game leaves its controller's list silently; the slot takes a new victim)
#[test]
fn a_lifted_controlled_vehicle_dies_and_frees_its_controllers_slot() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let v = put(&mut w, Kind::Vehicle, "TGT", BETA, 6, 0);
    let clone = put(&mut w, Kind::Infantry, "CLONE", ALPHA, 6, 4);
    let l = put(&mut w, Kind::Vehicle, "MAGL", BETA, -2, 0);
    testkit::run(&mut w, 5);
    assert!(mind::take(&mut w, clone, v));
    tick::step(&mut w);
    assert_eq!(owner(&w, v), Some(ALPHA));
    lift_up(&mut w, l, v);
    assert!(
        mind::mind_controlled(&w, v),
        "still controlled while gripped"
    );
    round_trip(&w);
    // The clone's own house: a blocker it will not take over.
    let blocker = put(&mut w, Kind::Vehicle, "TGT", ALPHA, 2, 0);
    let under = w.obj(blocker).unwrap().pos.cell();
    assert!(
        run_until(&mut w, 600, |w| w
            .obj(v)
            .is_some_and(|o| o.pos.cell() == under)
            && w.height_above_surface(v) > 300),
        "carried over the blocker"
    );
    let away = w.obj(l).unwrap().pos.cell();
    glasnost_sim::econ::assign_destination(
        &mut w,
        l,
        Some(Target::Cell(glasnost_map::CellXY::new(away.x, away.y + 2))),
    );
    assert!(
        run_until(&mut w, 900, |w| w.obj(v).is_none()),
        "dropped onto the blocker"
    );
    assert!(mind::controller(&w, clone).is_none_or(|r| r.victims.is_empty()));
    let next = put(&mut w, Kind::Infantry, "GRUNT", BETA, 6, 7);
    testkit::run(&mut w, 2);
    attack(&mut w, ALPHA, clone, next);
    assert!(
        run_until(&mut w, 900, |w| mind::mind_controlled(w, next)),
        "takes again"
    );
    round_trip(&w);
}

// Pair: a lifter and a bunkered vehicle.
// spec: S-0606 claim 9 and S-0309 claim 24 row 1 (a locomotor-warhead attack on a bunkered vehicle is redirected to its bunker: the hit seizes nothing)
// spec: S-0607 claims 1 and 3 (a building is never seized and the hit does no damage)
#[test]
fn a_lift_attack_on_a_bunkered_vehicle_goes_to_its_bunker() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let b = put(&mut w, Kind::Building, "BUNK", ALPHA, 0, 0);
    let g = put(&mut w, Kind::Vehicle, "GATT", ALPHA, 6, 1);
    bunker_in(&mut w, g, b);
    let l = put(&mut w, Kind::Vehicle, "MAGL", BETA, -6, 6);
    testkit::run(&mut w, 2);
    let (hb, hg) = (w.obj(b).unwrap().health, w.obj(g).unwrap().health);
    attack(&mut w, BETA, l, g);
    testkit::run(&mut w, 2);
    round_trip(&w);
    let mut fired = false;
    for _ in 0..200 {
        tick::step(&mut w);
        // A wave at a bunkered target ends at once (S-0607 claim 21: inside a bunker is out of play), so the shot itself is looked for.
        fired |= w.projectiles.values().any(|p| p.attacker == Some(l));
    }
    assert!(fired, "the lifter fired");
    assert!(!lift::seized(&w, g) && !lift::seized(&w, b));
    assert_eq!(bunker::bunkered_in(&w, g), Some(b), "still bunkered");
    assert_eq!(
        (w.obj(b).unwrap().health, w.obj(g).unwrap().health),
        (hb, hg),
        "no damage"
    );
}

// Pair: a gattling vehicle seized while spun up.
// spec: S-0607 claims 5 and 10 (the seized unit starts guard with no target; it gets illegal for every weapon)
// spec: S-0609 claim 5 (no target, or an answer of illegal: one spin-down update in each firing step)
#[test]
fn a_seized_gattling_spins_down() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let gat = put(&mut w, Kind::Vehicle, "GAT", ALPHA, 0, 0);
    let tgt = put(&mut w, Kind::Vehicle, "TGT", BETA, 6, 0);
    testkit::run(&mut w, 5);
    attack(&mut w, ALPHA, gat, tgt);
    climb(&mut w, gat);
    let l = put(&mut w, Kind::Vehicle, "MAGL", BETA, 0, 6);
    lift_up(&mut w, l, gat);
    round_trip(&w);
    let mut last = gattling::spin(&w, gat).value;
    let mut steps = Vec::new();
    for _ in 0..30 {
        tick::step(&mut w);
        if !lift::seized(&w, gat) {
            break;
        }
        let s = gattling::spin(&w, gat).value;
        steps.push(s - last);
        last = s;
    }
    assert!(
        steps.len() >= 10 && steps.iter().all(|&d| d == -1),
        "{steps:?}"
    );
}

// Pair: a seized vehicle and its owner's recycling building.
// spec: S-0607 claim 9 (a seized unit ignores every new destination, from orders too)
// spec: S-0603 claim 1 and S-0214 table I row I26 (the recycle click gives an enter order with the grinder as destination)
#[test]
fn a_seized_vehicle_ignores_its_owners_recycle_click() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let grnd = put(&mut w, Kind::Building, "GRND", ALPHA, -5, -5);
    let v = put(&mut w, Kind::Vehicle, "TGT", ALPHA, 6, 0);
    let l = put(&mut w, Kind::Vehicle, "MAGL", BETA, 6, 8);
    testkit::run(&mut w, 5);
    lift_up(&mut w, l, v);
    let bound = w.obj(v).unwrap().dest;
    assert!(matches!(bound, Some(Target::Cell(_))), "{bound:?}");
    click(&mut w, ALPHA, &[v], grnd);
    testkit::run(&mut w, 2);
    assert_eq!(
        w.obj(v).unwrap().dest,
        bound,
        "still bound for the cell of the movement point fixed at the seize"
    );
    round_trip(&w);
    let credits0 = credits(&w, ALPHA);
    testkit::run(&mut w, 60);
    assert!(
        lift::grips(&w, l, v)
            || w.obj(v)
                .is_none_or(|o| o.dest != Some(Target::Object(grnd)))
    );
    assert_eq!(credits(&w, ALPHA), credits0, "not ground");
}

// Pair: a lifter and a host carrying a parasite drone.
// spec: S-0614 claim 12 (an effect that replaces the host's locomotor: a drone stays attached)
// spec: S-0607 claim 5 (the seize releases a naval parasite only)
#[test]
fn a_drone_stays_on_a_seized_host() {
    let mut w = testkit::game_with(&rules(), SIZE, 1);
    let host = put(&mut w, Kind::Vehicle, "TGT", BETA, 2, 0);
    let drone = put(&mut w, Kind::Vehicle, "DRONE", ALPHA, 0, 0);
    let l = put(&mut w, Kind::Vehicle, "MAGL", ALPHA, 2, 7);
    testkit::run(&mut w, 2);
    attack(&mut w, ALPHA, drone, host);
    assert!(
        run_until(&mut w, 300, |w| glasnost_sim::parasite::host_of(w, drone)
            == Some(host)),
        "latched"
    );
    lift_up(&mut w, l, host);
    assert_eq!(
        glasnost_sim::parasite::host_of(&w, drone),
        Some(host),
        "stays attached"
    );
    round_trip(&w);
    testkit::run(&mut w, 10);
    if lift::seized(&w, host) {
        assert_eq!(glasnost_sim::parasite::host_of(&w, drone), Some(host));
    }
}
