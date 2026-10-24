//! Claim tests for S-0318 (shrapnel fragments), on invented rules and a
//! flat synthetic map: synthetic analogues of scenarios SH1 to SH5 and SH7.
//! Every type, damage and range is our own. A shot of the test weapon `Shot`
//! (invisible projectile `Ball`, naming the fragment weapon `Frag`) is
//! launched through `combat::projectile::launch`; the burst itself is driven
//! either through a whole tick or directly through `combat::shrapnel::burst`
//! with the shot as it stands at its detonation.

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, ProjectileType, WarheadType, WeaponType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::combat::projectile::{Special, launch};
use glasnost_sim::combat::shrapnel;
use glasnost_sim::combat::state::Projectile;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

const N: i32 = 40;
/// The shot's damage and the fragment's (visibly smaller, as SH's setup).
const SHOT: i32 = 50;
const FRAG: i32 = 7;
/// SH1's surrounding vehicles, from T's cell, in S-0303 table T1 order.
const AROUND: [(i32, i32); 5] = [(0, -1), (1, 0), (0, 2), (3, 0), (2, 2)];

fn d(s: &str) -> Decimal {
    Decimal(s.into())
}

fn every(v: &str) -> Option<Vec<Decimal>> {
    Some(vec![d(v); 11])
}

/// The combat ruleset with the test weapon, the test fragment weapon (range
/// 3 cells, invisible projectile `Bit`) and single-target warheads; every
/// unit unarmed and strong.
fn base(count: i32) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.sim.weapons.insert(
        "Shot".into(),
        WeaponType {
            damage: Some(SHOT),
            rof: Some(100),
            range: Some(d("10")),
            speed: Some(50),
            projectile: Some("Ball".into()),
            warhead: Some("Pin".into()),
            ..Default::default()
        },
    );
    r.sim.weapons.insert(
        "Frag".into(),
        WeaponType {
            damage: Some(FRAG),
            rof: Some(100),
            range: Some(d("3")),
            speed: Some(40),
            projectile: Some("Bit".into()),
            warhead: Some("Prick".into()),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Ball".into(),
        ProjectileType {
            inviso: Some(true),
            shrapnel_weapon: Some("Frag".into()),
            shrapnel_count: Some(count),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Bit".into(),
        ProjectileType {
            inviso: Some(true),
            ..Default::default()
        },
    );
    for h in ["Pin", "Prick"] {
        r.sim.warheads.insert(
            h.into(),
            WarheadType {
                verses: every("1"),
                ..Default::default()
            },
        );
    }
    for v in ["CART", "BUGGY", "CRUSHER"] {
        let t = r.sim.vehicles.get_mut(v).unwrap();
        t.primary = None;
        t.secondary = None;
        t.strength = Some(5000);
    }
    let i = r.sim.infantry.get_mut("WALKER").unwrap();
    i.primary = None;
    i.strength = Some(5000);
    r
}

fn frag(r: &mut Ruleset) -> &mut WeaponType {
    r.sim.weapons.get_mut("Frag").unwrap()
}

fn ball(r: &mut Ruleset) -> &mut ProjectileType {
    r.sim.projectiles.get_mut("Ball").unwrap()
}

struct S {
    w: World,
    /// The test vehicle (house 0), 6 cells west of T.
    a: ObjectId,
    /// The target T (house 1) at the middle cell, if any.
    t: Option<ObjectId>,
}

fn cell(dx: i32, dy: i32) -> CellXY {
    testkit::mid(N, dx, dy)
}

/// The world: houses 0 and 1 are players, 2 neutral and 3 the special
/// house; the test vehicle at (−6, 0) from T's cell, T a buggy of house 1.
fn setup(r: &Ruleset, with_t: bool) -> S {
    let mut w = testkit::game_with_non_players(r, N, 11);
    let a = testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, cell(-6, 0), 0);
    let t = with_t.then(|| testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, cell(0, 0), 0));
    S { w, a, t }
}

/// An enemy buggy at (dx, dy) from T's cell, of `house`.
fn put(s: &mut S, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(&mut s.w, Kind::Vehicle, "BUGGY", house, cell(dx, dy), 0)
}

fn around(s: &mut S) -> Vec<ObjectId> {
    AROUND.iter().map(|&(x, y)| put(s, 1, x, y)).collect()
}

fn health(s: &S, id: ObjectId) -> i32 {
    s.w.obj(id).unwrap().health
}

fn wp(w: &World, name: &str) -> u32 {
    w.rules().combat.weapon_index[name]
}

/// Launches the test weapon's shot at `target` from the test vehicle;
/// returns the shot as it stands at K (an invisible projectile is moved
/// there at once, S-0306 claim 9).
fn shoot(s: &mut S, target: Target) -> Projectile {
    let shot = wp(&s.w, "Shot");
    let p = s.w.obj(s.a).unwrap().pos;
    let id = launch(&mut s.w, s.a, shot, target, p, SHOT).expect("a shot");
    s.w.projectiles[&id].clone()
}

/// The shot at T.
fn shoot_t(s: &mut S) -> Projectile {
    let t = Target::Object(s.t.expect("T"));
    shoot(s, t)
}

/// Bursts `pr` directly at its position with no special hit.
fn burst(s: &mut S, pr: &Projectile) {
    burst_with(s, pr, None);
}

fn burst_with(s: &mut S, pr: &Projectile, special: Option<Special>) {
    let rules = s.w.rules_arc();
    let p = rules.combat.projectile_of(pr.weapon).unwrap().clone();
    shrapnel::burst(&mut s.w, pr, &p, pr.pos, special);
}

/// The fragments in play (chain level ≥ 1), in launch order.
fn frags(s: &S) -> Vec<Projectile> {
    s.w.projectiles
        .values()
        .filter(|p| p.chain >= 1)
        .cloned()
        .collect()
}

fn targets(s: &S) -> Vec<Target> {
    frags(s).iter().filter_map(|p| p.target).collect()
}

fn objs(ids: &[ObjectId]) -> Vec<Target> {
    ids.iter().map(|&x| Target::Object(x)).collect()
}

fn half(w: &World) -> bool {
    w.rules().quirks.ranged_draw_half_span
}

/// The cells the next `m` fragment draw pairs give around T's cell.
fn drawn(w: &World, m: usize) -> (Vec<Target>, glasnost_sim::rng::Rng) {
    let mut c = w.rng.clone();
    let h = half(w);
    let mut out = Vec::new();
    for _ in 0..m {
        let dx = c.ranged(-2, 2, h);
        let dy = c.ranged(-2, 2, h);
        out.push(Target::Cell(cell(dx, dy)));
    }
    (out, c)
}

fn with_quirks(r: &Ruleset, f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) -> Ruleset {
    let mut r = r.clone();
    f(&mut r.globals.quirks);
    r
}

// ---------------------------------------------------------------- claims 1, 3 and 5: a whole tick

fn lost(s: &S, v: &[ObjectId], before: &[i32]) -> Vec<i32> {
    v.iter()
        .zip(before)
        .map(|(&x, &b)| b - health(s, x))
        .collect()
}

// spec: S-0318 claims 1, 3 and 5 and State and ordering (scenario SH1 part a: T takes the shot's damage and the first 3 of the surrounding vehicles in T1 order take the fragment's; with S-0203's removal-skip switch off, all in the tick of the shot)
#[test]
fn sh1a_three_fragments_in_the_tick_of_the_shot() {
    let r = with_quirks(&base(3), |q| q.update_order_removal_skip = false);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let t = s.t.unwrap();
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    let t0 = health(&s, t);
    shoot_t(&mut s);
    testkit::run(&mut s.w, 1);
    assert_eq!(health(&s, t), t0 - SHOT);
    assert_eq!(lost(&s, &v, &before), vec![FRAG, FRAG, FRAG, 0, 0]);
    assert!(
        frags(&s).is_empty(),
        "every invisible fragment detonated in the tick of the burst"
    );
}

// spec: S-0318 claim 5 and State and ordering (scenario SH1 part a run 1: the fragments join the end of the update order in launch order during the bursting projectile's turn; with S-0203's removal skip on, the projectile's removal costs the first fragment its turn and each fragment's removal costs the next waiting one its turn: T's loss and the vehicle at (1, 0)'s in the tick of the shot, (0, −1)'s one tick later, (0, 2)'s two ticks later)
#[test]
fn sh1a_removals_cost_fragments_their_first_turn() {
    let mut s = setup(&base(3), true);
    let v = around(&mut s);
    let t = s.t.unwrap();
    let t0 = health(&s, t);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    shoot_t(&mut s);
    testkit::run(&mut s.w, 1);
    assert_eq!(health(&s, t), t0 - SHOT, "T in the tick of the shot");
    assert_eq!(lost(&s, &v, &before), vec![0, FRAG, 0, 0, 0]);
    testkit::run(&mut s.w, 1);
    assert_eq!(lost(&s, &v, &before), vec![FRAG, FRAG, 0, 0, 0]);
    testkit::run(&mut s.w, 1);
    assert_eq!(lost(&s, &v, &before), vec![FRAG, FRAG, FRAG, 0, 0]);
    assert!(frags(&s).is_empty());
}

// spec: S-0318 claim 5 (scenario SH1 part a run 2: a shot animation that entered play directly after the projectile, S-0109 claim 29, misses the turn instead, so the pattern starts one place later: T's loss and the vehicles at (0, −1) and (0, 2) in the tick of the shot, the vehicle at (1, 0) one tick later)
#[test]
fn sh1a_run_2_shot_animation_shifts_the_pattern() {
    let mut r = base(3);
    r.registries
        .entry("animations".into())
        .or_default()
        .push("FLASH".into());
    r.sim.animations.insert(
        "FLASH".into(),
        glasnost_rules::sim::AnimationType {
            frame_count: Some(40),
            rate: Some(1),
            ..Default::default()
        },
    );
    r.sim.weapons.get_mut("Shot").unwrap().anim = Some(vec!["FLASH".into()]);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let t = s.t.unwrap();
    let t0 = health(&s, t);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    let shot = wp(&s.w, "Shot");
    let p = s.w.obj(s.a).unwrap().pos;
    launch(&mut s.w, s.a, shot, Target::Object(t), p, SHOT).expect("a shot");
    glasnost_sim::combat::shotanim::create(&mut s.w, s.a, shot, p);
    assert_eq!(s.w.anims.anims.len(), 1, "the shot animation entered play");
    testkit::run(&mut s.w, 1);
    assert_eq!(health(&s, t), t0 - SHOT, "T in the tick of the shot");
    assert_eq!(lost(&s, &v, &before), vec![FRAG, 0, FRAG, 0, 0]);
    testkit::run(&mut s.w, 1);
    assert_eq!(lost(&s, &v, &before), vec![FRAG, FRAG, FRAG, 0, 0]);
    assert!(frags(&s).is_empty());
    assert_eq!(
        s.w.anims.anims.len(),
        1,
        "the run is not void: the animation is still in play"
    );
}

// spec: S-0318 claim 5 (scenario SH4 part b: the vehicle at (0, −1) is destroyed by the first fragment, which detonates one tick after the shot; house A's count of enemy units destroyed rises by one then, and no other house's count changes)
#[test]
fn sh4b_fragment_kill_credited_in_the_tick_after_the_shot() {
    let mut s = setup(&base(3), true);
    let v = around(&mut s);
    s.w.obj_mut(v[0]).unwrap().health = FRAG;
    let kills = |s: &S| -> Vec<i32> {
        (0..2)
            .map(|h| glasnost_sim::econ::stats::of(&s.w, HouseId(h)).kills)
            .collect()
    };
    let k0 = kills(&s);
    shoot_t(&mut s);
    testkit::run(&mut s.w, 1);
    assert!(s.w.obj(v[0]).is_some(), "alive after the tick of the shot");
    assert_eq!(kills(&s), k0, "no kill in the tick of the shot");
    testkit::run(&mut s.w, 1);
    assert!(s.w.obj(v[0]).is_none_or(|o| o.health <= 0), "destroyed");
    assert_eq!(
        kills(&s),
        vec![k0[0] + 1, k0[1]],
        "house A's count, one tick later"
    );
}

// spec: S-0318 claim 1 (b) (scenario SH1 part b: a shot at a building of one-cell foundation, first-listed in the burst cell, launches no fragments)
#[test]
fn sh1b_building_first_listed_stops_the_burst() {
    let mut s = setup(&base(3), false);
    let v = around(&mut s);
    let b = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell(0, 0), 0);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    let rng = s.w.rng.clone();
    let pr = shoot(&mut s, Target::Object(b));
    assert_eq!(s.w.rng, rng);
    burst(&mut s, &pr);
    assert!(frags(&s).is_empty());
    assert_eq!(s.w.rng, rng, "no draw for fragments");
    let after: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    assert_eq!(before, after);
}

// spec: S-0318 claim 1 (b) (a burst cell with no listed object launches no fragments; scenario SH1 part l: a forced attack on an empty cell takes no claim 4 draw)
#[test]
fn sh1l_empty_burst_cell_takes_no_draw() {
    let mut s = setup(&base(3), false);
    around(&mut s);
    let pr = shoot(&mut s, Target::Cell(cell(0, 0)));
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert!(frags(&s).is_empty());
    assert_eq!(s.w.rng, rng);
}

// spec: S-0318 claim 1 (c) and claim 2 (scenario SH1 part o: a fragment count of 0 launches nothing and takes no draw)
#[test]
fn sh1o_count_zero() {
    let mut s = setup(&base(0), true);
    around(&mut s);
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert!(frags(&s).is_empty());
    assert_eq!(s.w.rng, rng);
}

// spec: S-0318 claim 1 (a) (scenario SH1 part n run 3: a detonation that selects bomb planting launches no fragments and takes no draw)
#[test]
fn sh1n_special_hit_stops_the_burst() {
    let mut s = setup(&base(8), true);
    around(&mut s);
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst_with(&mut s, &pr, Some(Special::Bomb));
    assert!(frags(&s).is_empty());
    assert_eq!(s.w.rng, rng);
}

// spec: S-0318 claim 1 (a) (scenario SH1 part n run 3, through the detonation: the bomb-planting warhead selects its row and the surrounding vehicles take no fragment)
#[test]
fn sh1n_bomb_planting_warhead_through_the_detonation() {
    let mut r = base(3);
    r.sim.warheads.get_mut("Pin").unwrap().ivan_bomb = Some(true);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    shoot_t(&mut s);
    testkit::run(&mut s.w, 3);
    let after: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    assert_eq!(before, after);
}

// spec: S-0318 claim 1 (a) (scenario SH1 part n run 1: the electromagnetic-pulse warhead does not stop fragments)
#[test]
fn sh1n_emp_warhead_still_bursts() {
    let mut r = base(3);
    r.sim.warheads.get_mut("Pin").unwrap().em_effect = Some(true);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    shoot_t(&mut s);
    testkit::run(&mut s.w, 3);
    let lost: Vec<i32> = v
        .iter()
        .zip(&before)
        .map(|(&x, &b)| b - health(&s, x))
        .collect();
    assert_eq!(lost, vec![FRAG, FRAG, FRAG, 0, 0]);
}

// spec: S-0318 claim 1 (a detonation whose area damage does nothing still bursts; scenario SH1 part s run 1: a damage field of 0)
#[test]
fn sh1s_zero_damage_shot_still_bursts() {
    let mut r = base(3);
    r.sim.weapons.get_mut("Shot").unwrap().damage = Some(0);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let t = s.t.unwrap();
    let t0 = health(&s, t);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    let shot = wp(&s.w, "Shot");
    let p = s.w.obj(s.a).unwrap().pos;
    launch(&mut s.w, s.a, shot, Target::Object(t), p, 0).unwrap();
    testkit::run(&mut s.w, 3);
    assert_eq!(health(&s, t), t0);
    let lost: Vec<i32> = v
        .iter()
        .zip(&before)
        .map(|(&x, &b)| b - health(&s, x))
        .collect();
    assert_eq!(lost, vec![FRAG, FRAG, FRAG, 0, 0]);
}

// ---------------------------------------------------------------- claims 3 and 4: targets and draws

// spec: S-0318 claim 3 (scenario SH1 part a: the first 3 candidates in T1 visiting order, at once, no draw)
#[test]
fn sh1a_targets_in_visiting_order() {
    let mut s = setup(&base(3), true);
    let v = around(&mut s);
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]));
    assert_eq!(s.w.rng, rng, "the scan takes no draw");
}

// spec: S-0318 claims 3 and 4 and State and ordering (scenario SH1 part c: count 8 gives the 5 candidates, then 3 random cells from 6 draws R(−2, 2) in Δx, Δy pairs)
#[test]
fn sh1c_random_cells_after_the_candidates() {
    let mut s = setup(&base(8), true);
    let v = around(&mut s);
    let pr = shoot_t(&mut s);
    let (cells, after) = drawn(&s.w, 3);
    burst(&mut s, &pr);
    let mut want = objs(&v);
    want.extend(cells);
    assert_eq!(targets(&s), want);
    assert_eq!(s.w.rng, after, "exactly 6 ranged draws");
}

// spec: S-0318 claim 3 (scenario SH1 part d: an own vehicle first-listed in a visited cell is passed over)
#[test]
fn sh1d_own_vehicle_is_not_a_candidate() {
    let mut s = setup(&base(3), true);
    let own = put(&mut s, 0, 0, -1);
    let v: Vec<ObjectId> = AROUND[1..]
        .iter()
        .map(|&(x, y)| put(&mut s, 1, x, y))
        .collect();
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]));
    assert!(!targets(&s).contains(&Target::Object(own)));
}

fn infantry_cell(r: &Ruleset) -> (S, ObjectId, ObjectId, Vec<ObjectId>) {
    let mut s = setup(r, true);
    let enemy = testkit::spawn(&mut s.w, Kind::Infantry, "WALKER", 1, cell(0, -1), 0);
    let own = testkit::spawn(&mut s.w, Kind::Infantry, "WALKER", 0, cell(0, -1), 0);
    let v: Vec<ObjectId> = AROUND[1..]
        .iter()
        .map(|&(x, y)| put(&mut s, 1, x, y))
        .collect();
    (s, enemy, own, v)
}

// spec: S-0318 claim 3 and Edge cases (quirks.shrapnel_first_listed_only on; scenario SH1 part e: an own unit listed first blocks the cell, the enemy behind it is passed over)
#[test]
fn sh1e_first_listed_only() {
    let (mut s, enemy, own, v) = infantry_cell(&base(3));
    let list =
        s.w.cells
            .list(cell(0, -1), glasnost_sim::world::Layer::Ground);
    assert_eq!(
        list.first(),
        Some(&own),
        "the most recent entrant is listed first (S-0209 claim 13)"
    );
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]));
    assert!(!targets(&s).contains(&Target::Object(enemy)));
}

// spec: S-0318 Edge cases (quirks.shrapnel_first_listed_only off: the first object in listing order that passes the test is the candidate)
#[test]
fn sh1e_first_listed_only_off() {
    let r = with_quirks(&base(3), |q| q.shrapnel_first_listed_only = false);
    let (mut s, enemy, _own, v) = infantry_cell(&r);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&[enemy, v[0], v[1]]));
}

fn tree(r: &mut Ruleset) {
    r.registries
        .entry("terrain_objects".into())
        .or_default()
        .push("Shrub".into());
    r.sim
        .terrain_objects
        .insert("Shrub".into(), Default::default());
}

fn tree_scene(r: &Ruleset) -> (S, ObjectId, Vec<ObjectId>) {
    let mut s = setup(r, true);
    let ty = s.w.rules().index_of(Kind::Terrain, "Shrub").unwrap();
    let t = glasnost_sim::objects::create(&mut s.w, Kind::Terrain, ty, None);
    glasnost_sim::objects::place(&mut s.w, t, Pos::centre(cell(0, -1), 0), Default::default());
    let v: Vec<ObjectId> = AROUND[1..]
        .iter()
        .map(|&(x, y)| put(&mut s, 1, x, y))
        .collect();
    (s, t, v)
}

// spec: S-0318 claim 3 and Edge cases (quirks.shrapnel_hits_terrain on; scenario SH1 part f: a terrain object, with no owning house, is a candidate and takes the first fragment)
#[test]
fn sh1f_terrain_takes_a_fragment() {
    let mut r = base(3);
    tree(&mut r);
    let (mut s, t, v) = tree_scene(&r);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&[t, v[0], v[1]]));
}

// spec: S-0318 Edge cases (quirks.shrapnel_hits_terrain off: objects without an owning house are not candidates)
#[test]
fn sh1f_terrain_quirk_off() {
    let mut r = base(3);
    tree(&mut r);
    let r = with_quirks(&r, |q| q.shrapnel_hits_terrain = false);
    let (mut s, _t, v) = tree_scene(&r);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]));
}

fn ally(w: &mut World, a: u8, b: u8) {
    let h = &mut w.houses[usize::from(a)];
    if !h.allies.contains(&HouseId(b)) {
        h.allies.push(HouseId(b));
        h.allies.sort();
    }
}

// spec: S-0318 claim 3 and Terms (the firer's view; scenario SH1 part g: the firer's house allies C, C does not ally it: C's vehicle is passed over)
#[test]
fn sh1g_allied_from_the_firers_view() {
    let mut s = setup(&base(3), true);
    let c = put(&mut s, 2, 0, -1);
    ally(&mut s.w, 0, 2);
    assert!(!s.w.allied(Some(HouseId(2)), Some(HouseId(0))));
    let v: Vec<ObjectId> = AROUND[1..]
        .iter()
        .map(|&(x, y)| put(&mut s, 1, x, y))
        .collect();
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]));
    assert!(!targets(&s).contains(&Target::Object(c)));
}

// spec: S-0318 claim 3 (scenario SH1 part h: the special house allies the firer's house, which does not ally it: its vehicle is a candidate)
#[test]
fn sh1h_one_way_alliance_towards_the_firer() {
    let mut s = setup(&base(3), true);
    let sp = put(&mut s, 3, 0, -1);
    ally(&mut s.w, 3, 0);
    assert!(!s.w.allied(Some(HouseId(0)), Some(HouseId(3))));
    let v: Vec<ObjectId> = AROUND[1..]
        .iter()
        .map(|&(x, y)| put(&mut s, 1, x, y))
        .collect();
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&[sp, v[0], v[1]]));
}

// spec: S-0318 F2 and claims 3 and 4 (scenario SH1 part i: a range of 2.75 cells gives k = 2, so ring 3 is not visited and the fourth fragment goes to a drawn cell)
#[test]
fn sh1i_fragment_range_truncates() {
    let mut r = base(4);
    frag(&mut r).range = Some(d("2.75"));
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let pr = shoot_t(&mut s);
    let (cells, after) = drawn(&s.w, 1);
    burst(&mut s, &pr);
    let mut want = objs(&v[..3]);
    want.extend(cells);
    assert_eq!(targets(&s), want);
    assert_eq!(s.w.rng, after);
}

// spec: S-0318 F2 (k < 1 visits no ring: every fragment goes by claim 4)
#[test]
fn f2_range_below_one_cell_visits_no_ring() {
    let mut r = base(2);
    frag(&mut r).range = Some(d("0.9"));
    let mut s = setup(&r, true);
    around(&mut s);
    let pr = shoot_t(&mut s);
    let (cells, after) = drawn(&s.w, 2);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), cells);
    assert_eq!(s.w.rng, after);
}

fn ring_eleven(r: &Ruleset) -> (S, ObjectId) {
    let mut r = r.clone();
    frag(&mut r).range = Some(d("11"));
    let mut s = setup(&r, true);
    let v = put(&mut s, 1, -3, 11);
    (s, v)
}

// spec: S-0318 claim 3 and Edge cases (quirks.spread_ring_eleven on; scenario SH1 part j: the cell T1 lists twice is visited twice, its candidate takes two fragments, no draw)
#[test]
fn sh1j_ring_eleven_cell_twice() {
    let (mut s, v) = ring_eleven(&base(2));
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&[v, v]));
    assert_eq!(s.w.rng, rng);
}

// spec: S-0318 Edge cases (quirks.spread_ring_eleven off: (−3, 11), of metric 12, is not visited, so V is unhurt and 4 draws are taken)
#[test]
fn sh1j_ring_eleven_quirk_off() {
    let r = with_quirks(&base(2), |q| q.spread_ring_eleven = false);
    let (mut s, _v) = ring_eleven(&r);
    let pr = shoot_t(&mut s);
    let (cells, after) = drawn(&s.w, 2);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), cells);
    assert_eq!(s.w.rng, after);
}

// spec: S-0318 Edge cases (quirks.shrapnel_building_per_cell off: the rule covers the whole scan, ring 11's second visit included: one fragment, not two)
#[test]
fn sh1j_building_per_cell_off_covers_the_second_visit() {
    let r = with_quirks(&base(2), |q| q.shrapnel_building_per_cell = false);
    let (mut s, v) = ring_eleven(&r);
    let pr = shoot_t(&mut s);
    let (cells, after) = drawn(&s.w, 1);
    burst(&mut s, &pr);
    let mut want = objs(&[v]);
    want.extend(cells);
    assert_eq!(targets(&s), want);
    assert_eq!(s.w.rng, after);
}

fn big_building(r: &mut Ruleset) {
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("BLOCK".into());
    r.sim.buildings.insert(
        "BLOCK".into(),
        glasnost_rules::sim::BuildingType {
            strength: Some(5000),
            foundation: Some([2, 2]),
            ..Default::default()
        },
    );
}

// spec: S-0318 claim 3 and Edge cases (quirks.shrapnel_building_per_cell on; scenario SH1 part k: a 2 × 2 building, listed in each visited foundation cell, takes four fragments, no draw)
#[test]
fn sh1k_one_building_several_fragments() {
    let mut r = base(4);
    big_building(&mut r);
    let mut s = setup(&r, true);
    let b = testkit::spawn(&mut s.w, Kind::Building, "BLOCK", 1, cell(1, 0), 0);
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&[b, b, b, b]));
    assert_eq!(s.w.rng, rng);
}

// spec: S-0318 Edge cases (quirks.shrapnel_building_per_cell off: each object takes at most one fragment from the scan; the rest go by claim 4)
#[test]
fn sh1k_building_per_cell_off() {
    let mut r = base(4);
    big_building(&mut r);
    let r = with_quirks(&r, |q| q.shrapnel_building_per_cell = false);
    let mut s = setup(&r, true);
    let b = testkit::spawn(&mut s.w, Kind::Building, "BLOCK", 1, cell(1, 0), 0);
    let pr = shoot_t(&mut s);
    let (cells, after) = drawn(&s.w, 3);
    burst(&mut s, &pr);
    let mut want = objs(&[b]);
    want.extend(cells);
    assert_eq!(targets(&s), want);
    assert_eq!(s.w.rng, after);
}

// spec: S-0318 claim 3 (scenario SH1 part q: a candidate whose armour the fragment warhead cannot damage still takes its fragment)
#[test]
fn sh1q_zero_versus_candidate_still_takes_a_fragment() {
    let mut r = base(3);
    r.sim.warheads.get_mut("Prick").unwrap().verses = every("0");
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]));
}

// ---------------------------------------------------------------- claim 2 and F1: the count

fn negative_at(dx: i32, dy: i32) -> usize {
    let mut r = base(-6);
    r.sim.weapons.get_mut("Shot").unwrap().range = Some(d("9"));
    let mut s = setup(&r, true);
    let a = s.a;
    let p = Pos::centre(cell(dx, dy), 0);
    let z = s.w.surface_at(p.x, p.y);
    glasnost_sim::objects::remove(&mut s.w, a);
    s.a = testkit::spawn(&mut s.w, Kind::Vehicle, "CART", 0, cell(dx, dy), 0);
    assert_eq!(s.w.obj(s.a).unwrap().pos, Pos::new(p.x, p.y, z));
    // Surrounding vehicles away from the firer's cell.
    for (x, y) in [(0, -1), (1, 0), (0, 2), (3, 0), (2, 2), (-1, 2)] {
        if (x, y) != (dx, dy) {
            put(&mut s, 1, x, y);
        }
    }
    let pr = shoot_t(&mut s);
    let rules = s.w.rules_arc();
    let n = shrapnel::count(
        &s.w,
        &pr,
        rules
            .combat
            .projectile_of(pr.weapon)
            .unwrap()
            .shrapnel_count,
        pr.pos,
    );
    burst(&mut s, &pr);
    let got = frags(&s).len();
    assert_eq!(i64::try_from(got).unwrap(), n.max(0));
    got
}

// spec: S-0318 claim 2 and F1 (scenario SH2 parts a to d: n = |c| − ⌊d / 256⌋ from the burst point to the attacker; 2 cells: 4, 5 cells: 1, 7 cells: 0, (−2, −2): 4)
#[test]
fn sh2_negative_count() {
    assert_eq!(negative_at(-2, 0), 4);
    assert_eq!(negative_at(-5, 0), 1);
    assert_eq!(negative_at(-7, 0), 0);
    assert_eq!(negative_at(-2, -2), 4);
}

// spec: S-0318 claim 3 (scenario SH2 part a: the attacker itself, inside the fragment range, is not a candidate)
#[test]
fn sh2a_attacker_is_not_a_candidate() {
    let mut r = base(-6);
    r.sim.weapons.get_mut("Shot").unwrap().range = Some(d("9"));
    let mut s = setup(&r, true);
    let a = s.a;
    glasnost_sim::objects::remove(&mut s.w, a);
    s.a = testkit::spawn(&mut s.w, Kind::Vehicle, "CART", 0, cell(-2, 0), 0);
    let v: Vec<ObjectId> = AROUND[..4]
        .iter()
        .map(|&(x, y)| put(&mut s, 1, x, y))
        .collect();
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v));
}

// ---------------------------------------------------------------- SH3: the attacker gone

// spec: S-0318 claims 2, 3, 4 and 8 and Edge cases (scenario SH3: the attacker gone, a negative count gives 3 fragments, all to drawn cells, no beam)
#[test]
fn sh3_attacker_gone() {
    let mut r = base(-6);
    frag(&mut r).is_laser = Some(true);
    frag(&mut r).laser_duration = Some(15);
    let mut s = setup(&r, true);
    for (x, y) in [(3, 0), (0, -3), (-3, 0)] {
        put(&mut s, 1, x, y);
    }
    let pr = shoot_t(&mut s);
    let a = s.a;
    glasnost_sim::objects::remove(&mut s.w, a);
    s.w.effects.clear();
    let (cells, after) = drawn(&s.w, 3);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), cells);
    assert_eq!(s.w.rng, after);
    assert!(
        !s.w.effects
            .iter()
            .any(|e| matches!(e, Effect::FragmentLine { .. })),
        "no beam after the attacker has gone"
    );
    // The fragments still carry the attacker (S-0302 claim 1).
    assert!(frags(&s).iter().all(|p| p.attacker == pr.attacker));
}

// spec: S-0318 claim 3 (scenario SH3 part c: the control, the attacker present, gives each surrounding vehicle one fragment and no draw)
#[test]
fn sh3c_control() {
    let mut s = setup(&base(3), true);
    let v: Vec<ObjectId> = [(3, 0), (0, -3), (-3, 0)]
        .iter()
        .map(|&(x, y)| put(&mut s, 1, x, y))
        .collect();
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    let mut got = targets(&s);
    got.sort_by_key(|t| match t {
        Target::Object(x) => x.0,
        _ => u32::MAX,
    });
    assert_eq!(got, objs(&v));
    assert_eq!(s.w.rng, rng);
}

// ---------------------------------------------------------------- claim 5: a fragment's fields

// spec: S-0318 claim 5 (scenario SH4: the fragment weapon's damage field unscaled, its warhead through its weapon, the projectile's attacker as attacker, placed at once in the update order; invisible: moved to its target at once)
#[test]
fn sh4_fragment_fields() {
    let mut s = setup(&base(1), true);
    let v = around(&mut s);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    let f = frags(&s);
    assert_eq!(f.len(), 1);
    let f = &f[0];
    assert_eq!(f.damage, FRAG);
    assert_eq!(f.weapon, wp(&s.w, "Frag"));
    assert_eq!(f.attacker, pr.attacker);
    assert_eq!(f.house, pr.house);
    assert_eq!(f.chain, 1);
    assert_eq!(f.fire_point, pr.pos);
    assert_eq!(f.multiplier, 256);
    assert_eq!(
        f.pos,
        s.w.obj(v[0]).unwrap().pos,
        "S-0306 claim 9: moved at once to K"
    );
    assert!(s.w.update.contains(f.id), "it enters play at once");
}

// spec: S-0318 claim 5 and S-0525 claim 1 (revision 4) (a fragment carries no weapon: its detonation leaves no radiation, whatever the fragment weapon's radiation level; scenario SH1 part n run 4), and S-0318 claim 1 (a) (the bursting projectile's radiation does not stop fragments; run 2)
#[test]
fn sh1n_radiation_of_shot_and_fragment() {
    let mut r = rad(base(3));
    frag(&mut r).rad_level = Some(100);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    shoot_t(&mut s);
    testkit::run(&mut s.w, 3);
    assert!(
        s.w.radiation.fields.is_empty(),
        "no fragment leaves radiation"
    );
    let lost: Vec<i32> = v
        .iter()
        .zip(&before)
        .map(|(&x, &b)| b - health(&s, x))
        .collect();
    assert_eq!(lost, vec![FRAG, FRAG, FRAG, 0, 0]);

    let mut r = rad(base(3));
    r.sim.weapons.get_mut("Shot").unwrap().rad_level = Some(100);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    shoot_t(&mut s);
    testkit::run(&mut s.w, 3);
    assert_eq!(
        s.w.radiation.fields.len(),
        1,
        "the shot leaves its radiation"
    );
    let lost: Vec<i32> = v
        .iter()
        .zip(&before)
        .map(|(&x, &b)| b - health(&s, x))
        .collect();
    assert_eq!(lost, vec![FRAG, FRAG, FRAG, 0, 0]);
}

// spec: S-0318 claim 6 (launched from the burst point towards the aim point's reference point, climbing at about 45°)
#[test]
fn claim6_visible_fragment_direction() {
    let mut r = base(1);
    r.sim.projectiles.get_mut("Bit").unwrap().inviso = None;
    let mut s = setup(&r, true);
    let v = put(&mut s, 1, 2, 0);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    let f = &frags(&s)[0];
    assert_eq!(f.target, Some(Target::Object(v)));
    assert_eq!(f.pos, pr.pos, "placed at the burst point");
    let [x, y, z] = f.vel;
    assert!(x.gt(glasnost_num::F64::ZERO), "towards +x");
    assert!(y.to_i32_common().abs() <= 1, "no sideways part");
    let (xi, zi) = (x.to_i32_common(), z.to_i32_common());
    assert!(zi > 0 && (xi - zi).abs() <= 1, "45° elevation: {xi} {zi}");
}

// ---------------------------------------------------------------- claim 7: chains

// spec: S-0318 claim 7 and Terms (scenario SH5 part a: a fragment whose projectile names a fragment weapon bursts again, one chain level below, with the original attacker)
#[test]
fn sh5a_fragments_of_fragments() {
    let mut r = base(1);
    r.sim.weapons.insert(
        "Splinter".into(),
        WeaponType {
            damage: Some(3),
            range: Some(d("3")),
            speed: Some(40),
            projectile: Some("Chip".into()),
            warhead: Some("Prick".into()),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Chip".into(),
        ProjectileType {
            inviso: Some(true),
            ..Default::default()
        },
    );
    let bit = r.sim.projectiles.get_mut("Bit").unwrap();
    bit.shrapnel_weapon = Some("Splinter".into());
    bit.shrapnel_count = Some(1);
    let mut s = setup(&r, true);
    let v1 = put(&mut s, 1, 0, -1);
    let v2 = put(&mut s, 1, 0, -3);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    let f1 = frags(&s)[0].clone();
    assert_eq!(f1.target, Some(Target::Object(v1)));
    let rules = s.w.rules_arc();
    let p1 = rules.combat.projectile_of(f1.weapon).unwrap().clone();
    shrapnel::burst(&mut s.w, &f1, &p1, f1.pos, None);
    let f2: Vec<Projectile> = frags(&s).into_iter().filter(|p| p.chain == 2).collect();
    assert_eq!(f2.len(), 1);
    assert_eq!(f2[0].weapon, wp(&s.w, "Splinter"));
    assert_eq!(f2[0].attacker, pr.attacker);
    // From v1's cell, ring 1 holds T's cell (T, first-listed, enemy): T, not v2 two rings out.
    assert_eq!(f2[0].target, Some(Target::Object(s.t.unwrap())));
    let _ = v2;
}

// spec: S-0318 claim 1 (scenario SH5 part b: each detonation of a cluster tests and bursts afresh from the same burst point, so with a cluster count of 2 the first 3 candidates each take two fragments)
#[test]
fn sh5b_each_cluster_detonation_bursts() {
    let mut r = with_quirks(&base(3), |q| q.update_order_removal_skip = false);
    ball(&mut r).cluster = Some(2);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    shoot_t(&mut s);
    testkit::run(&mut s.w, 1);
    assert_eq!(
        lost(&s, &v, &before),
        vec![2 * FRAG, 2 * FRAG, 2 * FRAG, 0, 0]
    );
}

// spec: S-0318 Edge cases and Constants (our engine's chain bound: a projectile of level d bursts only when d < 16)
#[test]
fn chain_bound_sixteen() {
    let mut s = setup(&base(3), true);
    around(&mut s);
    let mut pr = shoot_t(&mut s);
    pr.chain = shrapnel::CHAIN_BOUND;
    burst(&mut s, &pr);
    assert!(frags(&s).is_empty());
    pr.chain = shrapnel::CHAIN_BOUND - 1;
    burst(&mut s, &pr);
    assert_eq!(frags(&s).len(), 3);
    assert!(frags(&s).iter().all(|p| p.chain == shrapnel::CHAIN_BOUND));
}

// ---------------------------------------------------------------- claim 8: beams and bolts

fn effects_scene(laser: bool, bolt: bool) -> (S, Projectile, Vec<ObjectId>) {
    let mut r = base(3);
    frag(&mut r).is_laser = Some(laser);
    frag(&mut r).laser_duration = Some(15);
    frag(&mut r).is_electric_bolt = Some(bolt);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let pr = shoot_t(&mut s);
    s.w.effects.clear();
    burst(&mut s, &pr);
    (s, pr, v)
}

fn lines(s: &S) -> Vec<(Pos, Pos)> {
    s.w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::FragmentLine { from, to, .. } => Some((*from, *to)),
            _ => None,
        })
        .collect()
}

fn bolts(s: &S) -> Vec<(Pos, Pos)> {
    s.w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Bolt { from, to, .. } => Some((*from, *to)),
            _ => None,
        })
        .collect()
}

fn ends(s: &S, pr: &Projectile, v: &[ObjectId]) -> Vec<(Pos, Pos)> {
    v.iter()
        .map(|&x| (pr.pos, s.w.obj(x).unwrap().pos))
        .collect()
}

// spec: S-0318 claim 8 (scenario SH1 part m run 1: a laser fragment weapon gives one beam per fragment from the burst point to its target, none from the firer)
#[test]
fn sh1m_laser_lines() {
    let (s, pr, v) = effects_scene(true, false);
    assert_eq!(lines(&s), ends(&s, &pr, &v[..3]));
    assert!(bolts(&s).is_empty());
    let a = s.a;
    assert!(s.w.effects.iter().all(|e| match e {
        Effect::FragmentLine { attacker, .. } => *attacker == a,
        _ => true,
    }));
}

// spec: S-0318 claim 8 (scenario SH1 part m run 2: an electric-bolt fragment weapon gives one bolt per fragment from the burst point)
#[test]
fn sh1m_bolts() {
    let (s, pr, v) = effects_scene(false, true);
    assert_eq!(bolts(&s), ends(&s, &pr, &v[..3]));
    assert!(lines(&s).is_empty());
}

// spec: S-0318 claim 8 (scenario SH1 part m run 3: both flags are tested separately, so each fragment makes a beam and a bolt)
#[test]
fn sh1m_both_flags() {
    let (s, pr, v) = effects_scene(true, true);
    assert_eq!(lines(&s), ends(&s, &pr, &v[..3]));
    assert_eq!(bolts(&s), ends(&s, &pr, &v[..3]));
}

// ---------------------------------------------------------------- import

// spec: S-0318 Import mapping (ShrapnelWeapon absent: no fragments; ShrapnelCount absent: 0)
#[test]
fn import_defaults() {
    let mut r = base(3);
    let b = ball(&mut r);
    b.shrapnel_count = None;
    let s = setup(&r, true);
    let p = s.w.rules().combat.projectile_of(wp(&s.w, "Shot")).unwrap();
    assert_eq!(p.shrapnel_count, 0);
    assert_eq!(p.shrapnel_weapon, Some(wp(&s.w, "Frag")));
    let mut r = base(3);
    ball(&mut r).shrapnel_weapon = None;
    let s = setup(&r, true);
    let p = s.w.rules().combat.projectile_of(wp(&s.w, "Shot")).unwrap();
    assert_eq!(p.shrapnel_weapon, None);
}

/// Game-wide radiation values of our own (S-0525 Import mapping), so that a
/// radiation weapon's detonation makes a field that lasts.
fn rad(mut r: Ruleset) -> Ruleset {
    let g = &mut r.sim.general;
    g.rad_duration_multiple = Some(2);
    g.rad_application_delay = Some(15);
    g.rad_level_max = Some(10_000);
    g.rad_level_delay = Some(60);
    g.rad_light_delay = Some(60);
    g.rad_level_factor = Some(d("0.0625"));
    r
}
