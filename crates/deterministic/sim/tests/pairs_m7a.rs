//! Interaction-pair scenario tests of M7a (PLAN, cross-component
//! interactions): each pair runs through orders and the fixed tick pipeline
//! on synthetic data (invented types, weapons and warheads, every number our
//! own). Houses Alpha (0) and Beta (1) of the testkit's sandbox game, not
//! allied.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{BuildingType, Decimal, UnitType, WarheadType, WeaponType};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::targeting::{self, Action, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use glasnost_sim::{gattling, mind, orders, save, testkit, tick, transport};

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
/// of 0.5 and Betaland's infantry cost factor 0.5.
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
    r.sim.infantry.get_mut("GRUNT").unwrap().cost = Some(400);
    r.sim.general.economy.refund_percent = Some(dec("0.5"));
    r.sim.countries.get_mut("Betaland").unwrap().econ.cost_infantry_mult = Some(dec("0.5"));
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

fn attack(w: &mut World, house: u8, actor: ObjectId, target: ObjectId) {
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
    );
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
    let clone = put(&mut w, Kind::Infantry, "CLONE", BETA, 0, 6);
    testkit::run(&mut w, 5);
    attack(&mut w, ALPHA, gat, tgt);
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
    assert_eq!(owner(&w, grunt), Some(ALPHA), "inside as its original owner's unit");
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
    assert_eq!(credits(&w, BETA) - cb, beta_value, "the controller's house is paid");
    assert_eq!(credits(&w, ALPHA), ca, "the original owner gets nothing");
    assert_eq!(stats(&w), s0, "no kill, no loss");
    assert!(mind::controller(&w, clone).is_none_or(|r| r.victims.is_empty()));
    assert!(
        !fx.iter().any(|e| matches!(e, Effect::Mind { sound: MindSound::Cleared, .. })),
        "no release sound"
    );
    assert!(fx.iter().any(|e| matches!(e, Effect::Consumed { grinding: true, .. })));
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
    assert_eq!(w.obj(civ).unwrap().dest, Some(Target::Object(grnd)), "the grinder fate");
    round_trip(&w);
    let c0 = credits(&w, 2);
    assert!(
        run_until(&mut w, 900, |w| w.obj(civ).is_none()),
        "consumed"
    );
    assert_eq!(credits(&w, 2) - c0, 400, "P for the neutral house");
}
