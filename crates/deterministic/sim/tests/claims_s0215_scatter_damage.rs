//! S-0215 claim 19, *Scatter from damage*, and S-0208 claim 15 (requests A
//! and B): the scatter requests that a hit sends a mobile actor that takes
//! damage and survives it, in the order of S-0215 claim 16 (anger,
//! retaliation, request A or the scatter from damage, the no-threat test,
//! request B, fear). Every hit here is a real shot unless a test says
//! otherwise: the firer's weapon fires a projectile (`combat::fire::shot`)
//! and it detonates in the tick pipeline (`testkit::run`). Synthetic data
//! only: the combat ruleset of `testkit` with invented types (a long-range
//! buggy, an unarmed peon, a fraidycat civilian, an aircraft) and invented
//! behaviour files after scenarios S-0215 B6 and S-0208 W7 and W9; no stock
//! value. Ledger: `docs/m8-claims/scatter-damage.md`.

mod common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    BehaviourProps, Decimal, LocomotionClass, Locomotor, UnitType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::fire;
use glasnost_sim::regions::STEPS;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit::{self, mid, spawn};
use glasnost_sim::trace::{self, What};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

const SIZE: i32 = 40;

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// The combat ruleset with: the weapon LONG (range 10 cells, an arcing
/// shell) on BUGGY; sight 7 for CART and WALKER, so that F7's distance is
/// 7.5 cells; PEON (unarmed infantry), CIVVY (unarmed fraidycat infantry);
/// the aircraft JET (armed, weapon AGM, range 6) and SCOUT (unarmed); and
/// the player-scatter setting as given.
fn rules(player_scatter: bool) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let d = |s: &str| Decimal(s.into());
    r.sim.weapons.insert(
        "Long".into(),
        WeaponType {
            damage: Some(20),
            rof: Some(90),
            range: Some(d("10")),
            speed: Some(40),
            projectile: Some("Lob".into()),
            warhead: Some("SA".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Agm".into(),
        WeaponType {
            damage: Some(40),
            rof: Some(30),
            range: Some(d("6")),
            speed: Some(60),
            projectile: Some("Rocket".into()),
            warhead: Some("HE".into()),
            ..Default::default()
        },
    );
    r.sim.vehicles.get_mut("BUGGY").unwrap().primary = Some("Long".into());
    r.sim.vehicles.get_mut("CART").unwrap().sight = Some(7);
    r.sim.infantry.get_mut("WALKER").unwrap().sight = Some(7);
    let foot = |fraidycat: bool| UnitType {
        strength: Some(100),
        speed: Some(4),
        locomotion_class: Some(LocomotionClass::Foot),
        locomotor: Some(Locomotor::Walk),
        fraidycat: Some(fraidycat),
        ..Default::default()
    };
    for (n, u) in [("PEON", foot(false)), ("CIVVY", foot(true))] {
        push(&mut r, "infantry", n);
        r.sim.infantry.insert(n.into(), u);
    }
    let flyer = |weapon: Option<&str>| {
        let mut u = UnitType {
            strength: Some(200),
            speed: Some(30),
            rot: Some(12),
            locomotor: Some(Locomotor::Fly),
            primary: weapon.map(Into::into),
            ..Default::default()
        };
        u.combat.ammo = Some(2);
        u.econ.landable = Some(true);
        u
    };
    for (n, u) in [("JET", flyer(Some("Agm"))), ("SCOUT", flyer(None))] {
        push(&mut r, "aircraft", n);
        r.sim.aircraft.insert(n.into(), u);
    }
    r.sim.general.player_scatter = Some(player_scatter);
    r
}

/// Sets the scatter and retaliate properties of behaviour `b`.
fn props(r: &mut Ruleset, b: Behaviour, scatter: bool, retaliate: bool) {
    r.sim.behaviours.insert(
        b.key().into(),
        BehaviourProps {
            scatter: Some(scatter),
            retaliate: Some(retaliate),
            ..Default::default()
        },
    );
}

fn force(w: &mut World, id: ObjectId, b: Behaviour) {
    let t = w.tick;
    let stale = w.rules().quirks.stale_suspended_revert;
    w.obj_mut(id).unwrap().beh.force(b, t, stale);
}

/// Sets `id`'s experience to veteran (S-0311 claim 1).
fn veteran(w: &mut World, id: ObjectId) {
    let m = w.rules().mode;
    w.obj_mut(id).unwrap().combat.experience = glasnost_num::F64::from_i64(1, m).to_f32(m);
}

/// The attacker: a BUGGY of house `h` in *sleep*, so that it fires only
/// when told to.
fn attacker(w: &mut World, h: u8, dx: i32, dy: i32) -> ObjectId {
    let a = spawn(w, Kind::Vehicle, "BUGGY", h, mid(SIZE, dx, dy), 64);
    force(w, a, Behaviour::Sleep);
    a
}

/// `firer` fires its primary weapon at `target` and the shell detonates in
/// the tick pipeline; the target loses health and survives. Gives the tick
/// of the hit.
fn shoot(w: &mut World, firer: ObjectId, target: ObjectId) -> i32 {
    let h0 = health(w, target);
    w.obj_mut(firer).unwrap().target = Some(Target::Object(target));
    assert!(fire::shot(w, firer, 0, Target::Object(target)), "a shot");
    let t0 = w.tick;
    loop {
        // The tick number grows by 1 at the end of the tick (S-0203 claim 1).
        let tick = w.tick;
        testkit::run(w, 1);
        if w.projectiles.is_empty() {
            w.obj_mut(firer).unwrap().target = None;
            let h = health(w, target);
            assert!(h < h0 && h > 0, "a hit it survives: {h0} to {h}");
            return tick;
        }
        assert!(w.tick < t0 + 120, "the shot did not detonate");
    }
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

fn cell_of(w: &World, id: ObjectId) -> CellXY {
    w.obj(id).unwrap().pos.cell()
}

fn dest_cell(w: &World, id: ObjectId) -> Option<CellXY> {
    match w.obj(id).unwrap().dest {
        Some(Target::Cell(c)) => Some(c),
        _ => None,
    }
}

/// The scatter answers recorded for `id` on tick `t`: (accepted, destination).
fn answers(w: &World, id: ObjectId, t: i32) -> Vec<(bool, Option<CellXY>)> {
    trace::take(w)
        .into_iter()
        .filter(|e| e.unit == id && e.tick == t)
        .filter_map(|e| match e.what {
            What::Asked { accepted, dest } => Some((accepted, dest)),
            _ => None,
        })
        .collect()
}

fn neighbour(c: CellXY, d: i32) -> CellXY {
    let (dx, dy) = STEPS[d.rem_euclid(8) as usize];
    CellXY::new(c.x + dx, c.y + dy)
}

// ---------------------------------------------------------------------------
// Vehicles: S-0215 claim 19, scenario B6

/// A sandbox game like `testkit::game_with` (houses 0 and 1, both player
/// houses, human-controlled), with the neutral house 2 and the special
/// house 3 when `non_players` (`testkit::game_with_non_players`), and house
/// `ai` set up as an AI house (glossary *AI house*: human-controlled, our
/// own AI playing it).
fn game(r: &Ruleset, ai: Option<usize>, non_players: bool) -> World {
    use glasnost_sim::world::{HouseRole, HouseSetup};
    let mut s = testkit::setup(5);
    if non_players {
        for (name, country, role) in [
            ("Neutral", "Alphaland", HouseRole::Neutral),
            ("Special", "Betaland", HouseRole::Special),
        ] {
            s.houses.push(HouseSetup {
                name: name.into(),
                country: country.into(),
                human: false,
                start: None,
                role,
                ai: false,
            });
        }
    }
    if let Some(h) = ai {
        s.houses[h].ai = true;
    }
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(r, "synthetic-test"));
    glasnost_sim::load::new_game(rules, &testkit::empty_map(SIZE), testkit::catalogue(), &s)
        .expect("synthetic test game loads")
}

/// A CART of house 1 in *guard* at the middle and a BUGGY of house 0 `dx`
/// cells map west of it; house 1 an AI house when `ai`.
fn cart_scene_of(r: &Ruleset, dx: i32, ai: bool) -> (World, ObjectId, ObjectId) {
    let mut w = game(r, ai.then_some(1), false);
    assert!(
        w.houses[1].human,
        "a player house is human-controlled, AI or not"
    );
    let v = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 0), 0);
    let a = attacker(&mut w, 0, -dx, 0);
    testkit::run(&mut w, 4);
    (w, v, a)
}

/// [`cart_scene_of`] with no AI house.
fn cart_scene(r: &Ruleset, dx: i32) -> (World, ObjectId, ObjectId) {
    cart_scene_of(r, dx, false)
}

/// Whether the vehicle was sent the scatter from damage on the tick of the
/// hit: an accepted request on open ground gives it a destination there
/// (S-0207 claim 16), and nothing else in these scenes does.
fn scattered(w: &World, v: ObjectId) -> bool {
    w.obj(v).unwrap().dest.is_some()
}

// spec: S-0215 claim 19 case 1 (scenario B6 part d: a vehicle of a human-controlled owner, AI houses alike, in guard with no target and no destination, hit from beyond its weapon range and beyond F7's (S + 0.5) × 256 leptons, may retaliate but does not; with the player-scatter setting on it is sent the request, forced and without a threat point, and moves away; no override to attack, the suspended slot stays empty)
#[test]
fn claim_19_b6_d_case_1_a_vehicle_hit_from_beyond_f7_scatters() {
    let (mut w, v, a) = cart_scene(&rules(true), 9);
    let c0 = cell_of(&w, v);
    let h0 = health(&w, v);
    shoot(&mut w, a, v);
    assert!(health(&w, v) < h0 && health(&w, v) > 0, "a hit it survives");
    let o = w.obj(v).unwrap();
    assert_eq!(o.target, None, "no retaliation");
    assert_eq!(o.beh.suspended, None, "no override to attack");
    assert!(scattered(&w, v), "sent the scatter from damage");
    testkit::run(&mut w, 90);
    assert_ne!(cell_of(&w, v), c0, "it moves away to scatter");
}

// spec: S-0215 claim 19 case 1 (scenario B6 part d, control 1: the player-scatter setting off and no scatter veterancy ability, no request)
#[test]
fn claim_19_b6_d_control_1_player_scatter_off_sends_no_request() {
    let (mut w, v, a) = cart_scene(&rules(false), 9);
    let c0 = cell_of(&w, v);
    shoot(&mut w, a, v);
    assert!(!scattered(&w, v), "no request");
    testkit::run(&mut w, 90);
    assert_eq!(cell_of(&w, v), c0, "no move");
}

// spec: S-0215 claim 19 case 1 (the scatter veterancy ability stands in for the player-scatter setting; S-0311 claim 3, a veteran's list)
#[test]
fn claim_19_case_1_the_scatter_ability_stands_in_for_the_setting() {
    let mut r = rules(false);
    r.sim.vehicles.get_mut("CART").unwrap().veteran_abilities = vec!["SCATTER".into()];
    let (mut w, v, a) = cart_scene(&r, 9);
    veteran(&mut w, v);
    shoot(&mut w, a, v);
    assert!(scattered(&w, v), "the ability gives the request");
}

// spec: S-0215 claim 19 case 1 (scenario B6 part d, control 2: an attacker within F7's distance but beyond the weapon range; the vehicle retaliates, gets the attacker as target, and is never sent the request by case 1)
#[test]
fn claim_19_b6_d_control_2_a_retaliating_vehicle_is_never_sent_case_1() {
    let (mut w, v, a) = cart_scene(&rules(true), 7);
    shoot(&mut w, a, v);
    let o = w.obj(v).unwrap();
    assert_eq!(o.target, Some(Target::Object(a)), "the override to attack");
    assert_eq!(o.beh.current, Some(Behaviour::Attack));
}

/// B6 part b's behaviour file: *guard* with the scatter property as given
/// and no retaliate property, so that the vehicle may not retaliate.
fn b6_b(player_scatter: bool, scatter: bool) -> Ruleset {
    let mut r = rules(player_scatter);
    props(&mut r, Behaviour::Guard, scatter, false);
    r
}

// spec: S-0215 claim 19 case 2 (scenario B6 part b: a vehicle that may not retaliate, standing still with no target and no destination, in guard with the scatter property; with the player-scatter setting on it is sent the request and moves away; repeated with the property off, no request)
#[test]
fn claim_19_b6_b_case_2_needs_the_scatter_property() {
    for scatter in [true, false] {
        let (mut w, v, a) = cart_scene(&b6_b(true, scatter), 9);
        let c0 = cell_of(&w, v);
        shoot(&mut w, a, v);
        assert_eq!(scattered(&w, v), scatter, "scatter property {scatter}");
        testkit::run(&mut w, 90);
        assert_eq!(cell_of(&w, v) != c0, scatter, "scatter property {scatter}");
    }
}

// spec: S-0215 claim 19 case 2, In our engine (B6 part b's variant with the player-scatter setting off: the vehicle of a player house, AI houses included, is not sent the request; the original-only computer-controlled condition holds for the neutral and special houses and a test-only computer-controlled player house, S-0215 Terms)
#[test]
fn claim_19_b6_b_case_2_owner_rule_with_player_scatter_off() {
    let r = b6_b(false, true);
    // A player house, human or AI: no request.
    for ai in [false, true] {
        let (mut w, v, a) = cart_scene_of(&r, 9, ai);
        shoot(&mut w, a, v);
        assert!(
            !scattered(&w, v),
            "a player house (AI {ai}) is not sent case 2"
        );
    }
    // A test-only computer-controlled player house: the request.
    let (mut w, v, a) = cart_scene(&r, 9);
    w.houses[1].human = false;
    // spec: S-0217 claims 2 and 4 (the test holds the house's next check past its ticks: a house with no building would otherwise have hunt queued for the vehicle before the hit)
    w.houses[1].checks.next = i32::MAX;
    shoot(&mut w, a, v);
    assert!(
        scattered(&w, v),
        "a computer-controlled house is sent case 2"
    );
    // The neutral and special houses: the request.
    for h in [2u8, 3] {
        let mut w = testkit::game_with_non_players(&r, SIZE, 5);
        let v = spawn(&mut w, Kind::Vehicle, "CART", h, mid(SIZE, 0, 0), 0);
        let a = attacker(&mut w, 0, -9, 0);
        testkit::run(&mut w, 4);
        let h0 = health(&w, v);
        shoot(&mut w, a, v);
        assert!(health(&w, v) < h0, "house {h}: hit");
        assert!(scattered(&w, v), "house {h} is sent case 2");
    }
}

/// A hit on `v` from `a` entering at `damage::apply`, the entry a
/// detonation uses.
fn hit(w: &mut World, v: ObjectId, a: Option<ObjectId>) {
    let sa = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "SA")
        .unwrap() as u32;
    let h0 = health(w, v);
    damage::apply(
        w,
        v,
        Hit {
            amount: 20,
            warhead: Some(sa),
            distance: 0,
            attacker: a,
            house: a.and_then(|a| w.obj(a)).and_then(|o| o.owner),
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert!(health(w, v) < h0 && health(w, v) > 0, "a hit it survives");
}

// spec: S-0215 claim 19 case 2 (a hit without a known attacker: the actor may not retaliate against it, so case 2 decides; with the scatter property and the player-scatter setting on, the request)
#[test]
fn claim_19_case_2_damage_without_a_known_attacker() {
    let (mut w, v, _) = cart_scene(&rules(true), 9);
    hit(&mut w, v, None);
    assert!(scattered(&w, v), "case 2 with no known attacker");
    let (mut w, v, _) = cart_scene(&rules(false), 9);
    hit(&mut w, v, None);
    assert!(!scattered(&w, v), "a player house without the setting");
}

/// Scenario B6 parts f and g's behaviour file: *hunt* with the scatter and
/// the retaliate properties as given, the player-scatter setting off.
fn b6_fg(scatter: bool, retaliate: bool) -> Ruleset {
    let mut r = rules(false);
    props(&mut r, Behaviour::Hunt, scatter, retaliate);
    r
}

/// B6 parts f and g: *G*, a CART of the computer-controlled house 1, on
/// open flat ground at the middle, and the firer, a vehicle `name` of house
/// `h` 9 cells map west of it, beyond *G*'s weapon range, in *sleep*. After
/// the scene settles *G* is put in *hunt* and the hit lands before any
/// processing of *hunt*, so that *G* stands still and free (part f1's judged
/// hit).
fn hunt_scene(r: &Ruleset, name: &str, h: u8) -> (World, ObjectId, ObjectId) {
    let mut w = game(r, None, false);
    w.houses[1].human = false;
    // spec: S-0217 claims 2 and 4 (the test holds the house's next check past its ticks: a house with no building would otherwise have hunt queued for G at tick 0's check, processed before the judged hit)
    w.houses[1].checks.next = i32::MAX;
    let v = spawn(&mut w, Kind::Vehicle, "CART", 1, mid(SIZE, 0, 0), 0);
    let a = spawn(&mut w, Kind::Vehicle, name, h, mid(SIZE, -9, 0), 64);
    force(&mut w, a, Behaviour::Sleep);
    force(&mut w, v, Behaviour::Sleep);
    testkit::run(&mut w, 4);
    force(&mut w, v, Behaviour::Hunt);
    let o = w.obj(v).unwrap();
    assert!(o.target.is_none() && o.dest.is_none(), "free when hit");
    (w, v, a)
}

/// Whether `v` holds, after the hit, a destination in one of the 8 cells
/// next to its own and no target (B6 part f1's and part g's expectation).
fn next_cell_destination(w: &World, v: ObjectId) -> bool {
    let c = cell_of(w, v);
    w.obj(v).unwrap().target.is_none()
        && dest_cell(w, v)
            .is_some_and(|d| d != c && (d.x - c.x).abs() <= 1 && (d.y - c.y).abs() <= 1)
}

// spec: S-0215 r137 claim 19 case 2, Hunting vehicles in an arena fight, and Q-2782 (scenario B6 part f1: a computer-controlled house's vehicle in hunt with the scatter property and without the retaliate property, standing still and free, is sent the request at a hit by an enemy and at one that names no attacker; isolated on open flat ground it holds a destination in a cell next to its own and no target at the end of the hit's tick, whatever the player-scatter setting); M-0024
#[test]
fn claim_19_b6_f1_a_hunting_vehicle_of_a_computer_house_scatters() {
    let r = b6_fg(true, false);
    for named in [true, false] {
        let (mut w, v, a) = hunt_scene(&r, "BUGGY", 0);
        hit(&mut w, v, named.then_some(a));
        assert!(
            next_cell_destination(&w, v),
            "attacker named {named}: {:?}",
            w.obj(v).unwrap().dest
        );
    }
}

// spec: S-0215 r137 claim 19 case 2 (the property tested before the request is sent: scenario B6 part f3, hunt with neither the scatter nor the retaliate property, no request) and In our engine (a human-controlled owner with the player-scatter setting off is not sent case 2, the part f1 set-up with house 1 human-controlled); M-0024
#[test]
fn claim_19_b6_f3_control_needs_the_scatter_property() {
    let (mut w, v, a) = hunt_scene(&b6_fg(false, false), "BUGGY", 0);
    hit(&mut w, v, Some(a));
    assert!(!scattered(&w, v), "no scatter property, no request");
    let (mut w, v, a) = hunt_scene(&b6_fg(true, false), "BUGGY", 0);
    w.houses[1].human = true;
    hit(&mut w, v, Some(a));
    assert!(
        !scattered(&w, v),
        "a human-controlled owner without the setting"
    );
}

// spec: S-0215 r137 claim 19 case 2 and Q-2782 (scenario B6 parts g1 and g2: with the scatter and the retaliate properties, a hit by an attacker of the actor's own house or of a house allied both ways sends the request, since S-0309 claim 18 refuses retaliation only on the alliance; the control: the same firer of an enemy house within F7's distance is retaliated against, so every other condition of claim 18 holds); M-0025
#[test]
fn claim_19_b6_g1_g2_an_own_or_allied_attacker_gives_case_2() {
    let r = b6_fg(true, true);
    for allied in [false, true] {
        let h = if allied { 0 } else { 1 };
        let (mut w, v, a) = hunt_scene(&r, "BUGGY", h);
        if allied {
            w.houses[0].human = false;
            w.houses[0].allies.push(HouseId(1));
            w.houses[1].allies.push(HouseId(0));
        }
        hit(&mut w, v, Some(a));
        assert!(
            next_cell_destination(&w, v),
            "allied house {allied}: {:?} {:?}",
            w.obj(v).unwrap().target,
            w.obj(v).unwrap().dest
        );
    }
    let (mut w, v, _) = hunt_scene(&r, "BUGGY", 0);
    let a = attacker(&mut w, 0, -7, 0);
    hit(&mut w, v, Some(a));
    assert_eq!(
        w.obj(v).unwrap().target,
        Some(Target::Object(a)),
        "an enemy is retaliated against"
    );
}

// spec: S-0215 r137 claim 19 case 2 and Q-2782 (scenario B6 part g3: an enemy firer whose armour gives the actor's weapon a versus value of 0 may not be retaliated against, S-0304 claim 3 and S-0309 claim 18, so case 2 sends the request; the actor never takes it as target); M-0025
#[test]
fn claim_19_b6_g3_an_enemy_the_weapon_cannot_harm_gives_case_2() {
    let mut r = b6_fg(true, true);
    // CART's weapon (Gun, warhead HE) has a versus value of 0 against
    // CRUSHER's armour (Heavy, the sixth armour class).
    r.sim
        .warheads
        .get_mut("HE")
        .unwrap()
        .verses
        .as_mut()
        .unwrap()[5] = Decimal("0".into());
    r.sim.vehicles.get_mut("CRUSHER").unwrap().primary = Some("Long".into());
    let (mut w, v, a) = hunt_scene(&r, "CRUSHER", 0);
    hit(&mut w, v, Some(a));
    assert!(next_cell_destination(&w, v), "{:?}", w.obj(v).unwrap().dest);
    testkit::run(&mut w, 30);
    assert_ne!(
        w.obj(v).unwrap().target,
        Some(Target::Object(a)),
        "never its target"
    );
}

// spec: S-0215 claim 19 case 2 (refusals: a target, a destination with the locomotor moving, a tether link, each stops the request; the hits enter at damage::apply so that the state set just before stands at the hit)
#[test]
fn claim_19_case_2_a_target_a_destination_or_a_tether_stops_it() {
    let mut r = b6_b(true, true);
    // The driving cases below run in *move*: it too has the scatter property and no retaliate
    // property, so that only the condition under test differs.
    props(&mut r, Behaviour::Move, true, false);
    // A target (a cell far away), no destination.
    let (mut w, v, a) = cart_scene(&r, 9);
    let far = Target::Cell(mid(SIZE, 0, 12));
    w.obj_mut(v).unwrap().target = Some(far);
    hit(&mut w, v, Some(a));
    assert_eq!(w.obj(v).unwrap().dest, None, "a target: no request");
    // A destination, driving towards it: the destination stays.
    let (mut w, v, a) = cart_scene(&r, 9);
    let goal = mid(SIZE, 0, 12);
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(goal));
    force(&mut w, v, Behaviour::Move);
    testkit::run(&mut w, 8);
    assert!(glasnost_sim::occupancy::in_progress(&w, v), "moving");
    hit(&mut w, v, Some(a));
    assert_eq!(dest_cell(&w, v), Some(goal), "a destination: no request");
    // The locomotor moving with no destination (S-0208 Terms *locomotor moving*): the same drive,
    // its destination cleared just before the hit.
    let (mut w, v, a) = cart_scene(&r, 9);
    w.obj_mut(v).unwrap().dest = Some(Target::Cell(goal));
    force(&mut w, v, Behaviour::Move);
    testkit::run(&mut w, 8);
    w.obj_mut(v).unwrap().dest = None;
    assert!(glasnost_sim::occupancy::in_progress(&w, v), "still moving");
    hit(&mut w, v, Some(a));
    assert_eq!(
        w.obj(v).unwrap().dest,
        None,
        "the locomotor moving: no request"
    );
    // A tether link (a factory-release contact, S-0417 claim 10).
    let (mut w, v, a) = cart_scene(&r, 9);
    let k = spawn(&mut w, Kind::Building, "TOWER", 1, mid(SIZE, 3, 3), 0);
    {
        let o = w.obj_mut(v).unwrap();
        o.contact = Some(k);
        o.econ.exit = Some(glasnost_sim::econ::release::Exit {
            factory: k,
            tethered: true,
            place: 0,
        });
    }
    assert!(glasnost_sim::econ::tethered(&w, v));
    hit(&mut w, v, Some(a));
    assert_eq!(w.obj(v).unwrap().dest, None, "a tether link: no request");
    // The control: the same hit without any of them.
    let (mut w, v, a) = cart_scene(&r, 9);
    hit(&mut w, v, Some(a));
    assert!(scattered(&w, v), "the control is sent the request");
}

// ---------------------------------------------------------------------------
// Aircraft: S-0215 claim 19 case 1, S-0501 claim 49

/// A landed aircraft `name` of house 1 in *sleep*, a behaviour given the
/// scatter property as stated and the retaliate property, with the player
/// return-fire setting on so that the human owner's behaviour list does not
/// stop it; the BUGGY 9 cells map west.
fn air_scene(name: &str, player_scatter: bool, scatter: bool) -> (World, ObjectId, ObjectId) {
    let mut r = rules(player_scatter);
    props(&mut r, Behaviour::Sleep, scatter, true);
    r.sim.general.player_return_fire = Some(true);
    let mut w = testkit::game_with(&r, SIZE, 5);
    let j = spawn(&mut w, Kind::Aircraft, name, 1, mid(SIZE, 0, 0), 0);
    let a = attacker(&mut w, 0, -9, 0);
    testkit::run(&mut w, 4);
    {
        let t = w.tick;
        let o = w.obj_mut(j).unwrap();
        o.dest = None;
        o.target = None;
        o.beh.force(Behaviour::Sleep, t, false);
    }
    assert!(glasnost_sim::air::on_ground(&w, j), "landed");
    (w, j, a)
}

// spec: S-0215 claim 19 case 1 and S-0501 claim 49 (an aircraft that may retaliate and does not, with neither target nor destination and the player-scatter setting on, is sent the request; its current behaviour has the scatter property, so it falls back to rest on the tick of the hit; AC31 part e) and S-0215 claim 16 (no fall-back from the no-threat test here)
#[test]
fn claim_19_an_aircraft_answers_the_scatter_from_damage_by_a_fall_back() {
    for (player_scatter, scatter, falls) in [
        (true, true, true),
        (true, false, false),
        (false, true, false),
    ] {
        let (mut w, j, a) = air_scene("JET", player_scatter, scatter);
        let before = w.obj(j).unwrap().beh.last_common_effects;
        let t = shoot(&mut w, a, j);
        let o = w.obj(j).unwrap();
        assert_eq!(
            o.beh.last_common_effects == Some(t),
            falls,
            "setting {player_scatter}, property {scatter}: before {before:?}, beh {:?}",
            o.beh
        );
        assert_eq!(o.dest, None, "no nearby cell for an aircraft");
    }
}

// spec: S-0215 claim 19 case 2 (the actor is not an aircraft: an unarmed aircraft, which may not retaliate, is never sent the request, whatever the setting and the scatter property)
#[test]
fn claim_19_case_2_never_sends_an_aircraft_the_request() {
    let (mut w, j, a) = air_scene("SCOUT", true, true);
    let t = shoot(&mut w, a, j);
    assert_ne!(
        w.obj(j).unwrap().beh.last_common_effects,
        Some(t),
        "no request"
    );
}

// ---------------------------------------------------------------------------
// Infantry: S-0208 claim 15, scenarios W7 and W9

/// An infantry unit `name` of house `h` in *guard* at the middle, a BUGGY
/// of house `ah` `dx` cells map west of it, the trace on.
fn foot_scene(w: &mut World, name: &str, h: u8, ah: u8, dx: i32) -> (ObjectId, ObjectId) {
    let s = spawn(w, Kind::Infantry, name, h, mid(SIZE, 0, 0), 0);
    let a = attacker(w, ah, -dx, 0);
    testkit::run(w, 4);
    trace::switch_on(w);
    (s, a)
}

// spec: S-0208 claim 15, request A first case (scenario W7 part g: a soldier of a player house, not a fraidycat, that may retaliate, hit from beyond its sight so that it does not attack back; g1 the player-scatter rule on: request A accepted; g2 off: not sent; g3 off with the scatter veterancy ability: accepted) and request B (sent with a known attacker, refused by claim 11: not a fraidycat)
#[test]
fn claim_15_w7_g_first_case_of_request_a() {
    for (setting, ability, sent) in [
        (true, false, true),
        (false, false, false),
        (false, true, true),
    ] {
        let mut r = rules(setting);
        if ability {
            r.sim.infantry.get_mut("WALKER").unwrap().veteran_abilities = vec!["SCATTER".into()];
        }
        let mut w = testkit::game_with(&r, SIZE, 5);
        let (s, a) = foot_scene(&mut w, "WALKER", 1, 0, 9);
        if ability {
            veteran(&mut w, s);
        }
        let t = shoot(&mut w, a, s);
        assert_eq!(w.obj(s).unwrap().target, None, "it does not retaliate");
        let got = answers(&w, s, t);
        let accepted: Vec<bool> = got.iter().map(|x| x.0).collect();
        let want = if sent { vec![true, false] } else { vec![false] };
        assert_eq!(
            accepted, want,
            "setting {setting}, ability {ability}: {got:?}"
        );
        assert_eq!(dest_cell(&w, s).is_some(), sent);
    }
}

// spec: S-0208 claim 15 (retaliating always gives the unit a target, so a unit that retaliates is never sent request A by the first case: the attacker within F7's distance, beyond the rifle's range; only request B is asked, and refused) and S-0215 claim 18
#[test]
fn claim_15_a_retaliating_soldier_is_never_sent_request_a() {
    let mut w = testkit::game_with(&rules(true), SIZE, 5);
    let (s, a) = foot_scene(&mut w, "WALKER", 1, 0, 6);
    let t = shoot(&mut w, a, s);
    assert_eq!(
        w.obj(s).unwrap().target,
        Some(Target::Object(a)),
        "it retaliates"
    );
    assert_eq!(answers(&w, s, t), vec![(false, None)], "request B only");
}

// spec: S-0208 claim 15, request A second case, Original and our engine (scenario W7 part h: an unarmed unit, not a fraidycat, that may not retaliate, idle in guard with the scatter property, the player-scatter rule off; h6 a unit of the neutral house hit by a unit of an AI house is sent request A and accepts it; h2 a unit of the AI house hit by the neutral house's unit is not sent it, as for a human-controlled house; h4 with the rule on it is) and S-0215 Terms (houses in our engine)
#[test]
fn claim_15_w7_h_second_case_an_ai_house_and_the_neutral_house() {
    // h6: the neutral house's PEON, hit by an AI house's BUGGY.
    let mut w = game(&rules(false), Some(0), true);
    let (s, a) = foot_scene(&mut w, "PEON", 2, 0, 9);
    let t = shoot(&mut w, a, s);
    let got = answers(&w, s, t);
    assert_eq!(
        got.first().map(|x| x.0),
        Some(true),
        "neutral: request A: {got:?}"
    );
    assert_eq!(got.len(), 2, "then request B: {got:?}");
    assert!(dest_cell(&w, s).is_some());
    // h2 and h4: the AI house's PEON, hit by the neutral house's BUGGY.
    for setting in [false, true] {
        let mut w = game(&rules(setting), Some(1), true);
        assert!(w.houses[1].human, "an AI house is human-controlled");
        let (s, a) = foot_scene(&mut w, "PEON", 1, 2, 9);
        let t = shoot(&mut w, a, s);
        let got = answers(&w, s, t);
        let accepted: Vec<bool> = got.iter().map(|x| x.0).collect();
        let want = if setting {
            vec![true, false]
        } else {
            vec![false]
        };
        assert_eq!(accepted, want, "AI house, setting {setting}: {got:?}");
    }
}

// spec: S-0215 claim 16, Order within the hit (the no-threat test comes after the scatter from damage: request A of S-0208 claim 15 is decided on the target and destination the actor had before the no-threat fall-back, request B on the state that the fall-back leaves) and claim 13 (an infantry unit's fall-back takes its guard spot on another cell as its destination); the control without the no-threat property keeps request A's destination (seam sweep 7a: scatter-damage's row; the spec rates this order `unmeasured`, so it is tested synthetically)
#[test]
fn seams7a_claim_16_request_a_comes_before_the_no_threat_fall_back() {
    for no_threat in [false, true] {
        let mut r = rules(true);
        r.sim.behaviours.insert(
            Behaviour::Guard.key().into(),
            BehaviourProps {
                scatter: Some(true),
                no_threat: Some(no_threat),
                ..Default::default()
            },
        );
        let mut w = testkit::game_with(&r, SIZE, 5);
        // An unarmed PEON of a player house with the player-scatter rule on:
        // request A by its second case (W7 part h4).
        let (s, a) = foot_scene(&mut w, "PEON", 1, 0, 9);
        let spot = mid(SIZE, 4, 4);
        w.obj_mut(s).unwrap().guard_spot = Some(Target::Cell(spot));
        let t = w.tick;
        hit(&mut w, s, Some(a));
        let got = answers(&w, s, t);
        assert_eq!(
            got.first().map(|x| x.0),
            Some(true),
            "no-threat {no_threat}: request A, decided before the fall-back: {got:?}"
        );
        assert_eq!(got.len(), 2, "then request B: {got:?}");
        let a_dest = got[0].1;
        assert!(a_dest.is_some() && a_dest != Some(spot));
        if no_threat {
            assert_eq!(
                dest_cell(&w, s),
                Some(spot),
                "the fall-back after request A gives the guard spot"
            );
        } else {
            assert_eq!(dest_cell(&w, s), a_dest, "no fall-back: A's destination");
        }
    }
}

/// W7 part j's file: *guard* with the scatter property and no retaliate
/// property (the unit may not retaliate; request A by its second case,
/// request B accepted by a fraidycat); the player-scatter rule on, so that
/// the player-house fraidycat has scatter permission. `scatter` off gives
/// W9 part c's reference: neither request moves the unit.
fn w7_j(scatter: bool) -> Ruleset {
    let mut r = rules(true);
    props(&mut r, Behaviour::Guard, scatter, false);
    r
}

/// Replays on `start` `before` raw values of the background, then the draws
/// over `ranges`, then `after` raw values of the background; gives the end
/// state and the draws' values.
fn replay(
    start: &Rng,
    half: bool,
    ranges: &[(i32, i32)],
    before: u64,
    after: u64,
) -> (Rng, Vec<i32>) {
    let mut r = start.clone();
    for _ in 0..before {
        r.raw();
    }
    let js = ranges.iter().map(|&(a, b)| r.ranged(a, b, half)).collect();
    for _ in 0..after {
        r.raw();
    }
    (r, js)
}

/// Runs W7 part j for a CIVVY standing at the centre (`sub` None) or on
/// map sub-cell `sub`, and its W9 part c reference; checks the draws of the
/// tick of the hit and returns (A's destination, B's destination, the
/// walking target's cell, the end destination).
fn w7_j_run(sub: Option<i32>, ranges: &[(i32, i32)]) -> [Option<CellXY>; 4] {
    let build = |scatter: bool| {
        let mut sc = common::Scene::new();
        sc.ruleset = w7_j(scatter);
        let at = common::cell(0, 0);
        if let Some(k) = sub {
            sc.place_infantry(1, "CIVVY", at, k);
        }
        sc.place_unit(0, "BUGGY", common::cell(-9, 0), 64);
        let mut w = sc.build();
        let s = match sub {
            Some(_) => common::objects_of(&w, Kind::Infantry)[0],
            None => spawn(&mut w, Kind::Infantry, "CIVVY", 1, at, 0),
        };
        let a = common::objects_of(&w, Kind::Vehicle)[0];
        force(&mut w, a, Behaviour::Sleep);
        force(&mut w, s, Behaviour::Guard);
        testkit::run(&mut w, 4);
        trace::switch_on(&mut w);
        (w, s, a)
    };
    // The real shot through the tick path: both requests are asked and accepted on the tick of
    // the hit, and B's destination is the one that stands.
    {
        let (mut w, s, a) = build(true);
        let t = shoot(&mut w, a, s);
        let got = answers(&w, s, t);
        assert_eq!(got.len(), 2, "tick path: requests A then B: {got:?}");
        assert!(got[0].0 && got[1].0, "tick path: both accepted: {got:?}");
        assert_eq!(
            dest_cell(&w, s),
            got[1].1,
            "tick path: B's destination stands"
        );
    }
    // The draws, counted over the hit itself (`damage::apply`, the entry a detonation uses), so
    // that the rest of the tick, where a walking unit and a standing one differ, stays out.
    // W9 part c's reference: the same hit, where neither request moves the unit; its draws are
    // the background (S-0208 W9, *Background draws*).
    let (mut w0, s0, a0) = build(false);
    let r0 = w0.rng.clone();
    hit(&mut w0, s0, Some(a0));
    assert_eq!(dest_cell(&w0, s0), None, "the reference does not move");
    let background = w0.rng.taken() - r0.taken();
    let (mut w, s, a) = build(true);
    assert_eq!(w.rng, r0, "the same random state before the hit");
    let half = w.rules().quirks.ranged_draw_half_span;
    let t = w.tick;
    hit(&mut w, s, Some(a));
    // The hit takes the reference's background and, within it, the draws over `ranges` in their
    // order.
    let (_, js) = (0..=background)
        .map(|k| replay(&r0, half, ranges, k, background - k))
        .find(|(end, _)| *end == w.rng)
        .unwrap_or_else(|| {
            panic!(
                "the draws of the hit are not the background ({background} raw values) and \
                 {ranges:?}: {} raw values taken",
                w.rng.taken() - r0.taken()
            )
        });
    let got = answers(&w, s, t);
    assert_eq!(got.len(), 2, "requests A then B: {got:?}");
    assert!(got[0].0 && got[1].0, "both accepted: {got:?}");
    let (a_dest, b_dest) = (got[0].1, got[1].1);
    let o = w.obj(s).unwrap();
    let walk = o.walk.as_ref().and_then(|x| x.target).map(|p| p.cell());
    // spec: S-0208 claim 12 (the attacker due map west: d₀ = 2, so d = j for B's draw, the last; open ground: the neighbour in direction d of B's scatter centre cell)
    let jb = *js.last().unwrap();
    let centre = walk.expect("a walking target chosen at once (claim 6)");
    assert_eq!(
        b_dest,
        Some(neighbour(centre, jb)),
        "B's destination from B's draw"
    );
    [a_dest, b_dest, walk, dest_cell(&w, s)]
}

// spec: S-0208 claim 15 (scenario W7 part j and W9 part c: both requests accepted on the tick of the hit; on a side position, request A's draw then request B's, two draws; A's destination from the nearby-cell search gives the walker its walking target at once, claim 6, in A's destination cell, which is B's scatter centre cell; B's destination replaces A's) and S-0208 claims 12 and 13
#[test]
fn claim_15_w7_j_w9_c_both_requests_on_a_side_position_take_two_draws() {
    let [a_dest, b_dest, walk, end] = w7_j_run(Some(2), &[(0, 4), (0, 4)]);
    assert_eq!(
        walk, a_dest,
        "the walking target lies in A's destination cell"
    );
    assert_eq!(end, b_dest, "B's destination replaces A's");
    assert_ne!(end, a_dest);
}

// spec: S-0208 claim 15 (on a position of class centre the at-once choice of the walking target takes claim 3's draw between request A's and request B's: three draws on the tick of the hit) and claim 6
#[test]
fn claim_15_w7_j_on_the_centre_claim_3_draws_between_a_and_b() {
    let [a_dest, b_dest, walk, end] = w7_j_run(None, &[(0, 4), (0, 3), (0, 4)]);
    assert_eq!(walk, a_dest);
    assert_eq!(end, b_dest, "B's destination replaces A's");
}

// spec: S-0302 r43 claim 10 and Changes for other specs, S-0208 claim 15 (a berserk hit with a known attacker sends request B, not request A)
#[test]
fn claim_15_a_berserk_hit_sends_request_b_only() {
    let mut r = w7_j(true);
    r.sim.warheads.insert(
        "Mad".into(),
        glasnost_rules::sim::WarheadType {
            psychedelic: Some(true),
            verses: Some(vec![Decimal("1".into()); 11]),
            ..Default::default()
        },
    );
    let mut w = testkit::game_with(&r, SIZE, 5);
    let (s, a) = foot_scene(&mut w, "CIVVY", 1, 0, 9);
    let mad = w
        .rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == "Mad")
        .unwrap() as u32;
    let res = damage::apply(
        &mut w,
        s,
        Hit {
            amount: 20,
            warhead: Some(mad),
            distance: 0,
            attacker: Some(a),
            house: Some(HouseId(0)),
            ignore_defences: false,
            no_escape: false,
        },
    );
    assert_eq!(res, damage::HitResult::Unchanged, "berserk");
    assert!(
        w.obj(s).unwrap().combat.berserk.is_some(),
        "the hit took effect"
    );
    let got = answers(&w, s, w.tick);
    assert_eq!(got.len(), 1, "request B only: {got:?}");
    assert!(got[0].0, "the fraidycat accepts B: {got:?}");
}
