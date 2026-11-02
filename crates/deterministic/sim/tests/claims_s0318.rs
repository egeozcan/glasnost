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
    setup_seed(r, with_t, 11)
}

fn setup_seed(r: &Ruleset, with_t: bool, seed: u32) -> S {
    let mut w = testkit::game_with_non_players(r, N, seed);
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
// spec: S-0318 claim 5 (a fragment takes its first turn in the tick of the burst unless an earlier removal costs it that turn), S-0203 claim 5 (revision 40: the third fragment loses its first turn in the tick of the burst and again in the next, so its first turn is at n + 2) and S-0306 State and ordering (revision 38) and S-0203 claim 10 (removal-skip switch on: the shot leaving play in its own turn costs the first fragment, which entered during that turn, its turn; the second fragment leaving costs the third its turn, and in the next tick the first fragment leaving costs it again)
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

// spec: S-0318 r27 claims 1 and 5 (placed at the burst point, raised as S-0306 claim 8 gives: a fragment whose burst point is not above the terrain starts 1 lepton above it, at the same x and y, so a burst at height 0 gives fragments that start at height 1) and S-0306 claim 8 (revision 45, Changes for other specs: a fragment whose burst point is not above the terrain starts 1 lepton above it, at the same x and y; a burst above the terrain is kept; an invisible fragment's move to K is not raised; the walk's X₁ is the burst point)
#[test]
fn s0306_claim8_fragments_start_above_the_ground() {
    let mut r = base(1);
    r.sim.projectiles.get_mut("Bit").unwrap().inviso = None;
    for dz in [-150, 0, 1, 90] {
        let mut s = setup(&r, true);
        put(&mut s, 1, 2, 0);
        let pr = shoot_t(&mut s);
        let ground = s.w.surface_at(pr.pos.x, pr.pos.y);
        let at = Pos::new(pr.pos.x, pr.pos.y, ground + dz);
        let rules = s.w.rules_arc();
        let p = rules.combat.projectile_of(pr.weapon).unwrap().clone();
        shrapnel::burst(&mut s.w, &pr, &p, at, None);
        let f = &frags(&s)[0];
        let want = if dz > 0 { ground + dz } else { ground + 1 };
        assert_eq!(f.placed_at, Pos::new(at.x, at.y, want), "dz {dz}");
        assert_eq!(f.fire_point, at, "dz {dz}");
    }
    // Invisible fragments are moved to K at once; that move is not raised.
    let mut s = setup(&base(1), true);
    let v = put(&mut s, 1, 2, 0);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    let f = &frags(&s)[0];
    assert_eq!(f.target, Some(Target::Object(v)));
    assert_eq!(f.pos, f.stored, "at K");
    assert_eq!(
        f.pos.z,
        s.w.surface_at(f.pos.x, f.pos.y),
        "on the ground, not raised"
    );
}

// spec: S-0318 r27 claims 5 and 6 (a fragment starts from its starting position, claim 5, and is launched towards its aim point's reference point, climbing at about 45°; the direction is taken from the burst point as it is, not from the raised starting position) and S-0306 claim 8 (revision 45: a burst point at the surface gives a start 1 lepton above it; Changes for other specs)
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
    assert_eq!(
        pr.pos.z,
        s.w.surface_at(pr.pos.x, pr.pos.y),
        "a burst at the surface"
    );
    assert_eq!(
        f.pos,
        Pos::new(pr.pos.x, pr.pos.y, pr.pos.z + 1),
        "placed at the burst point, raised to height 1"
    );
    assert_eq!(
        f.fire_point, pr.pos,
        "the walk's X₁ is the burst point itself"
    );
    let [x, y, z] = f.vel;
    assert!(x.gt(glasnost_num::F64::ZERO), "towards +x");
    assert!(y.to_i32_common().abs() <= 1, "no sideways part");
    let (xi, zi) = (x.to_i32_common(), z.to_i32_common());
    assert!(zi > 0 && (xi - zi).abs() <= 1, "45° elevation: {xi} {zi}");
}

// spec: S-0306 claims 1, 2 and 7 (revision 38: a fragment is not a shot; its speed is the fragment weapon's weapon speed S_w, without claim 1's shot speed or claim 2's cap at half the distance to the impact point, and a steering fragment does not start at 1, which S-0306 leaves open and our engine does not do) and S-0318 claims 5 and 6
#[test]
fn s0306_fragment_speed_is_the_weapon_speed_uncapped() {
    let mut r = base(1);
    let bit = r.sim.projectiles.get_mut("Bit").unwrap();
    bit.inviso = None;
    // Steering (ρ > 0): S_w is the imported speed field, here the largest.
    bit.rot = Some(12);
    frag(&mut r).speed = Some(100);
    let mut s = setup(&r, true);
    let v = put(&mut s, 1, 1, 0);
    let pr = shoot_t(&mut s);
    burst(&mut s, &pr);
    let f = &frags(&s)[0];
    assert_eq!(f.target, Some(Target::Object(v)));
    let sw = s.w.rules().combat.weapon(f.weapon).speed;
    assert_eq!(sw, 255, "S_w: the stored speed of the field 100");
    let len2: i64 = f
        .vel
        .iter()
        .map(|c| i64::from(c.to_i32_common()).pow(2))
        .sum();
    let len = len2.isqrt() as i32;
    assert!((len - sw).abs() <= 2, "length S_w: {len}");
    let tp = s.w.obj(v).unwrap().pos;
    let d2 = i64::from(tp.x - f.pos.x).pow(2)
        + i64::from(tp.y - f.pos.y).pow(2)
        + i64::from(tp.z - f.pos.z).pow(2);
    assert!(
        len > d2.isqrt() as i32 / 2 + 2,
        "not capped at half the distance"
    );
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

// ---------------------------------------------------------------- S-0316 claim 2 (revision 35): fragments and the line-effect flags

/// Bursts the test shot at T with a fragment weapon of the given
/// line-effect flags (laser, electric bolt, radiation beam, magnetic beam),
/// the attacker still in play, its own weapon without line-effect flags and
/// no live wave of its own.
fn flags_scene(laser: bool, bolt: bool, rad: bool, mag: bool) -> (S, Projectile, Vec<ObjectId>) {
    let mut r = base(3);
    let f = frag(&mut r);
    f.is_laser = Some(laser);
    f.laser_duration = Some(15);
    f.is_electric_bolt = Some(bolt);
    f.is_rad_beam = Some(rad);
    f.is_mag_beam = Some(mag);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let pr = shoot_t(&mut s);
    assert_eq!(glasnost_sim::lift::wave_of(&s.w, s.a), None, "no live wave");
    s.w.effects.clear();
    burst(&mut s, &pr);
    (s, pr, v)
}

/// Whether the burst made a radiation beam or a magnetic wave: a shot's
/// line effect (whose radiation-beam flash the drawing reads from the
/// weapon), a wave, or a live wave of the attacker.
fn beam_or_wave(s: &S) -> bool {
    s.w.effects
        .iter()
        .any(|e| matches!(e, Effect::Line { .. } | Effect::Wave { .. }))
        || !s.w.waves.is_empty()
        || glasnost_sim::lift::wave_of(&s.w, s.a).is_some()
}

// spec: S-0316 claim 2 (revision 35, scenario EB6 part c runs 1 and 2: neither the radiation-beam nor the magnetic-beam flag suppresses a fragment's bolt, and a fragment of a weapon with either or both makes no radiation beam and no magnetic wave, while its projectile has its attacker and the attacker has no live wave) and S-0318 claim 8
#[test]
fn eb6c_bolt_with_radiation_or_magnetic_beam_flags() {
    for (rad, mag) in [(true, false), (false, true), (true, true)] {
        let (s, pr, v) = flags_scene(false, true, rad, mag);
        assert_eq!(bolts(&s), ends(&s, &pr, &v[..3]), "rad {rad} mag {mag}");
        assert!(lines(&s).is_empty());
        assert!(!beam_or_wave(&s), "rad {rad} mag {mag}");
    }
}

// spec: S-0316 claim 2 (revision 35, scenario EB6 part c runs 3 to 5: a fragment of a weapon without the electric-bolt flag that has both or one of the radiation-beam and magnetic-beam flags makes no radiation beam and no magnetic wave) and S-0318 claim 8 (no bolt without the electric-bolt flag)
#[test]
fn eb6c_radiation_or_magnetic_beam_flags_alone() {
    for (rad, mag) in [(true, true), (true, false), (false, true)] {
        let (s, _, _) = flags_scene(false, false, rad, mag);
        assert!(bolts(&s).is_empty(), "rad {rad} mag {mag}");
        assert!(lines(&s).is_empty());
        assert!(!beam_or_wave(&s), "rad {rad} mag {mag}");
        assert_eq!(frags(&s).len(), 3, "the fragments are still launched");
    }
}

// spec: S-0316 claim 2 (revision 35, scenario EB6 part c run 6: a fragment of a weapon with the laser flag and both of those flags makes no radiation beam and no magnetic wave; its laser beam is S-0318 claim 8's) and S-0318 claim 8
#[test]
fn eb6c_laser_with_both_beam_flags() {
    let (s, pr, v) = flags_scene(true, false, true, true);
    assert_eq!(lines(&s), ends(&s, &pr, &v[..3]));
    assert!(bolts(&s).is_empty());
    assert!(!beam_or_wave(&s));
}

// spec: S-0316 claim 2 (revision 35: the ranking of the line-effect flags is for shots; a fragment of a weapon with every line-effect flag makes its laser beam and its bolt, and no radiation beam or magnetic wave) and claims 8 and 10 (the shrapnel weapon's own alternate-colour flag gives a fragment's bolt its colour: the bolt names the fragment weapon, and its firer is the projectile's attacker)
#[test]
fn eb6c_every_flag_and_the_bolt_weapon() {
    let (s, pr, v) = flags_scene(true, true, true, true);
    assert_eq!(lines(&s), ends(&s, &pr, &v[..3]));
    assert_eq!(bolts(&s), ends(&s, &pr, &v[..3]));
    assert!(!beam_or_wave(&s));
    let fw = wp(&s.w, "Frag");
    assert!(s.w.effects.iter().all(|e| match e {
        Effect::Bolt { weapon, firer, .. } => *weapon == fw && *firer == s.a,
        _ => true,
    }));
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

// ---------------------------------------------------------------- seams sweep 1 (M8): parts of SH1, SH5 and SH8 not yet tested

/// Registers a wall overlay type of our own, of the given strength.
fn wall_type(r: &mut Ruleset, strength: i32) {
    r.registries
        .entry("overlays".into())
        .or_default()
        .push("RAMPART".into());
    r.sim.overlays.insert(
        "RAMPART".into(),
        glasnost_rules::sim::OverlayType {
            has_art: Some(true),
            wall: Some(true),
            strength: Some(strength),
            damage_levels: Some(3),
            armor: Some("steel".into()),
            ..Default::default()
        },
    );
}

// spec: S-0318 claim 1 and State and ordering, scenario SH1 part p (a wall hit in the shot's own area damage takes its draw after the burst's six ranged draws: the 3 random-cell fragments land where draws 1 to 6 give, and the wall's outcome agrees with the 7th draw R(0, T_w))
#[test]
fn sh1p_wall_hit_draw_follows_the_burst_draws() {
    let mut r = base(8);
    wall_type(&mut r, 100);
    {
        let h = r.sim.warheads.get_mut("Pin").unwrap();
        h.wall = Some(true);
        h.cell_spread = Some(d("1"));
    }
    let mut told_apart = 0;
    for seed in 1..=40 {
        let mut s = setup_seed(&r, true, seed);
        around(&mut s);
        let wall = cell(0, 1);
        let ov =
            s.w.rules()
                .overlays
                .iter()
                .position(|o| o.name == "RAMPART")
                .unwrap() as u8;
        {
            let c = s.w.cells.grid.get_mut(wall).unwrap();
            c.overlay = ov;
            c.overlay_data = 0;
        }
        let h = half(&s.w);
        let shot = wp(&s.w, "Shot");
        let p = s.w.obj(s.a).unwrap().pos;
        let t = s.t.unwrap();
        let id = launch(&mut s.w, s.a, shot, Target::Object(t), p, SHOT).unwrap();
        // The draws come from the state at the shot's detonation; the shot
        // takes its turn alone (the tick's other objects draw as well).
        let first = s.w.rng.clone().ranged(0, 100, h);
        let (_, mut after_six) = drawn(&s.w, 3);
        let seventh = after_six.ranged(0, 100, h);
        glasnost_sim::combat::projectile::turn(&mut s.w, id);
        let hit = s.w.cells.cell(wall).unwrap().overlay_data != 0;
        assert_eq!(hit, SHOT > seventh, "seed {seed}: the 7th draw");
        if (SHOT > first) != (SHOT > seventh) {
            told_apart += 1;
        }
    }
    assert!(told_apart > 0, "the seeds tell the two orders apart");
}

/// `base(count)` with the shot's projectile an airburst one that names a
/// test airburst weapon of its own (S-0306 claim 21).
fn airburst_rules(count: i32) -> Ruleset {
    let mut r = base(count);
    ball(&mut r).airburst = Some(true);
    ball(&mut r).airburst_weapon = Some("Burst".into());
    r.sim.weapons.insert(
        "Burst".into(),
        WeaponType {
            damage: Some(9),
            range: Some(d("6")),
            speed: Some(77),
            projectile: Some("Pellet".into()),
            warhead: Some("Prick".into()),
            ..Default::default()
        },
    );
    r.sim.projectiles.insert(
        "Pellet".into(),
        ProjectileType {
            inviso: Some(true),
            ..Default::default()
        },
    );
    r
}

// spec: S-0318 claim 1 and State and ordering, scenario SH5 part c (an airburst projectile bursts once, before its area damage and its nine children: the 5 surrounding vehicles are candidates, the 6 ranged draws R(-2, 2) come first, then the draws of the detonation, then the nine draws R(0, 32), and no draw R(256, 512))
#[test]
fn sh5c_airburst_bursts_before_its_area_damage_and_children() {
    let r = airburst_rules(8);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let id = {
        let shot = wp(&s.w, "Shot");
        let p = s.w.obj(s.a).unwrap().pos;
        let t = s.t.unwrap();
        launch(&mut s.w, s.a, shot, Target::Object(t), p, SHOT).unwrap()
    };
    let (cells, after_six) = drawn(&s.w, 3);
    // The control: the same shot with no fragments, from the state the six
    // draws leave. Its draws are the detonation's, then the nine children's.
    let mut c = setup(&airburst_rules(0), true);
    around(&mut c);
    let shot = wp(&c.w, "Shot");
    let p = c.w.obj(c.a).unwrap().pos;
    let ct = c.t.unwrap();
    let cid = launch(&mut c.w, c.a, shot, Target::Object(ct), p, SHOT).unwrap();
    c.w.rng = after_six;
    glasnost_sim::combat::projectile::turn(&mut c.w, cid);
    glasnost_sim::combat::projectile::turn(&mut s.w, id);
    let mut want = objs(&v);
    want.extend(cells);
    let got: Vec<Target> = frags(&s)
        .iter()
        .filter(|p| p.chain == 1)
        .filter_map(|p| p.target)
        .collect();
    assert_eq!(got, want, "the 5 candidates, then the 3 drawn cells");
    assert_eq!(s.w.rng, c.w.rng, "six ranged draws, then the rest in order");
    let kids = s.w.projectiles.values().filter(|p| p.chain == 0).count();
    assert_eq!(kids, 9, "the nine children are launched");
    assert!(
        s.w.projectiles
            .values()
            .all(|p| p.chain >= 1 || p.damage == 9),
        "the children carry the airburst weapon's damage"
    );
}

/// Fires a visible, slow, straight shot at `target` and runs until it is
/// gone (at most 400 ticks).
fn fly(s: &mut S, target: Target) {
    let shot = wp(&s.w, "Shot");
    let mut p = s.w.obj(s.a).unwrap().pos;
    p.z += 20;
    // The aim is the firer's current target (S-0306 claim 3 and its quirk).
    s.w.obj_mut(s.a).unwrap().target = Some(target);
    let id = launch(&mut s.w, s.a, shot, target, p, SHOT).expect("a shot");
    for _ in 0..400 {
        if !s.w.projectiles.contains_key(&id) {
            // The fragments take their turns in the ticks after.
            testkit::run(&mut s.w, 5);
            return;
        }
        testkit::run(&mut s.w, 1);
    }
    panic!("the shot never arrived");
}

// spec: S-0318 claim 1 (the burst is judged at the burst point, not at the detonation point that S-0306 claim 20 (c) may move to a building's aim shift) and Terms, scenario SH1 part r (run 1: a building at T's cell with an aim shift of 256 towards (1, 0) where an enemy vehicle stands: the burst cell is the building's, first-listed there, so no object loses the fragment's damage; run 2, control: the vehicle at T gives the fragments of part a)
#[test]
fn sh1r_burst_point_against_detonation_point() {
    let visible = |r: &mut Ruleset| {
        ball(r).inviso = None;
        r.sim.general.gravity = Some(0);
    };
    // Run 1.
    let mut r = base(3);
    visible(&mut r);
    r.sim
        .buildings
        .get_mut("TOWER")
        .unwrap()
        .target_coord_offset = Some([256, 0, 0]);
    let mut s = setup(&r, false);
    let v = around(&mut s);
    let b = testkit::spawn(&mut s.w, Kind::Building, "TOWER", 1, cell(0, 0), 0);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    fly(&mut s, Target::Object(b));
    let got = lost(&s, &v, &before);
    assert!(
        got.iter().all(|&l| l == 0 || l == SHOT),
        "no object loses the fragment's damage: {got:?}"
    );
    assert!(
        got[1] == SHOT,
        "the detonation point did move to the target point in (1, 0): {got:?}"
    );
    // Run 2, the control.
    let mut r = base(3);
    visible(&mut r);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
    let t = s.t.unwrap();
    fly(&mut s, Target::Object(t));
    assert_eq!(lost(&s, &v, &before), vec![FRAG, FRAG, FRAG, 0, 0]);
}

// spec: S-0318 claim 3 and Edge cases, scenario SH8 part d (the scan ignores the fragment projectile's anti-air and anti-ground flags: run 1 anti-ground off and anti-air on, run 2 both off; the three fragments still go to (0, -1), (1, 0) and (0, 2) and no draw R(-2, 2) is taken)
#[test]
fn sh8d_flags_of_the_fragment_projectile_are_ignored() {
    for (ag, aa) in [(false, true), (false, false)] {
        let mut r = base(3);
        r.sim.projectiles.get_mut("Bit").unwrap().ag = Some(ag);
        r.sim.projectiles.get_mut("Bit").unwrap().aa = Some(aa);
        let mut s = setup(&r, true);
        let v = around(&mut s);
        let pr = shoot_t(&mut s);
        let rng = s.w.rng.clone();
        burst(&mut s, &pr);
        assert_eq!(targets(&s), objs(&v[..3]), "ag {ag} aa {aa}");
        assert_eq!(s.w.rng, rng, "ag {ag} aa {aa}: no draw");
    }
}

// spec: S-0318 claim 3 and Edge cases, scenario SH8 part e (the scan lists the ground layer only: an aircraft in the air over (0, -1) is unhurt and passed over, the fragments go to (1, 0), (0, 2) and (3, 0), and no draw R(-2, 2) is taken)
#[test]
fn sh8e_an_aircraft_in_the_air_is_not_a_candidate() {
    let mut r = base(3);
    r.registries.insert("aircraft".into(), vec!["PLANE".into()]);
    r.sim.aircraft.insert(
        "PLANE".into(),
        glasnost_rules::sim::UnitType {
            strength: Some(5000),
            locomotor: Some(glasnost_rules::sim::Locomotor::Fly),
            ..Default::default()
        },
    );
    let mut s = setup(&r, true);
    let v: Vec<ObjectId> = AROUND[1..]
        .iter()
        .map(|&(x, y)| put(&mut s, 1, x, y))
        .collect();
    let plane = testkit::spawn(&mut s.w, Kind::Aircraft, "PLANE", 1, cell(0, -1), 0);
    s.w.obj_mut(plane).unwrap().pos.z += 1000;
    // Climbing removes the listing (S-0209 claim 13).
    glasnost_sim::air::sync_listing(&mut s.w, plane);
    assert!(
        !s.w.cells
            .list(cell(0, -1), glasnost_sim::world::Layer::Ground)
            .contains(&plane),
        "an aircraft at positive listing height is not listed"
    );
    assert!(
        glasnost_sim::combat::weapon::in_air(&s.w, plane),
        "void unless the aircraft is in the air over (0, -1)"
    );
    assert_eq!(s.w.obj(plane).unwrap().pos.cell(), cell(0, -1));
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]), "(1, 0), (0, 2) and (3, 0)");
    assert_eq!(s.w.rng, rng, "no draw");
    assert_eq!(health(&s, plane), s.w.rules().aircraft[0].strength);
}

/// `base(count)` with a cloaking vehicle type GHOST of house B's (an
/// unarmed copy of BUGGY that cloaks by itself, S-0516 claims 2 and 5).
fn cloak_rules(count: i32) -> Ruleset {
    let mut r = base(count);
    {
        let g = &mut r.sim.general;
        g.cloaking_stages = Some(8);
        g.cloak_delay = Some(d("0.01"));
    }
    let mut ghost = r.sim.vehicles["BUGGY"].clone();
    ghost.combat.cloakable = Some(true);
    ghost.combat.cloaking_speed = Some(2);
    r.sim.vehicles.insert("GHOST".into(), ghost);
    r.registries
        .get_mut("vehicles")
        .expect("vehicle registry")
        .push("GHOST".into());
    r
}

// spec: S-0318 claim 3 and Edge cases, scenario SH8 part a (the scan has no visibility test: a fully cloaked vehicle at (0, -1), with no sensor of house A anywhere, receives the first fragment, the others go to (1, 0) and (0, 2), (3, 0) is unhurt and no draw R(-2, 2) is taken)
#[test]
fn sh8a_a_cloaked_vehicle_is_a_candidate() {
    let r = cloak_rules(3);
    let mut s = setup(&r, true);
    let ghost = testkit::spawn(&mut s.w, Kind::Vehicle, "GHOST", 1, cell(0, -1), 0);
    let mut v = vec![ghost];
    v.extend(AROUND[1..].iter().map(|&(x, y)| put(&mut s, 1, x, y)));
    testkit::run(&mut s.w, 60);
    assert_eq!(
        glasnost_sim::cloak::state(&s.w, ghost),
        glasnost_sim::cloak::CloakState::Cloaked,
        "void unless fully cloaked at the shot"
    );
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]));
    assert_eq!(s.w.rng, rng, "no draw");
}

// spec: S-0318 claim 3 and Edge cases, scenario SH8 part c (the alliance test reads the object's true owner, so a disguise plays no part: a house-B infantry unit disguised with house A as its disguise house, at (0, -1), receives the first fragment, and (3, 0) is unhurt)
#[test]
fn sh8c_a_disguise_plays_no_part() {
    use glasnost_sim::cloak::{Disguise, DisguiseAs};
    let mut s = setup(&base(3), true);
    let inf = testkit::spawn(&mut s.w, Kind::Infantry, "WALKER", 1, cell(0, -1), 0);
    let walker = s.w.rules().index_of(Kind::Infantry, "WALKER").unwrap();
    let tick = s.w.tick;
    s.w.obj_mut(inf).unwrap().cloak.disguise = Some(Disguise {
        what: Some(DisguiseAs::Infantry(walker)),
        house: Some(HouseId(0)),
        since: tick,
    });
    let mut v = vec![inf];
    v.extend(AROUND[1..].iter().map(|&(x, y)| put(&mut s, 1, x, y)));
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..3]));
    assert_eq!(s.w.rng, rng, "no draw");
}

// spec: S-0318 claim 3 and Edge cases, scenario SH8 part b (the scan has no visibility test: with the shroud on, fragment count 4 and house A never having seen the cell (3, 0), the fourth fragment goes to the vehicle in (3, 0) and no draw R(-2, 2) is taken)
#[test]
fn sh8b_a_shrouded_vehicle_is_a_candidate() {
    use glasnost_sim::econ::shroud;
    let mut r = base(4);
    r.sim.vehicles.get_mut("CART").unwrap().sight = Some(7);
    let mut s = setup(&r, true);
    let v = around(&mut s);
    testkit::run(&mut s.w, 3);
    let a = HouseId(0);
    assert!(
        !shroud::seen(&s.w, a, cell(3, 0)) && !shroud::explored(&s.w, a, cell(3, 0)),
        "void if house A has seen (3, 0)"
    );
    assert!(
        shroud::explored(&s.w, a, cell(-6, 0)),
        "the shroud is on and the test vehicle sees"
    );
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert_eq!(targets(&s), objs(&v[..4]), "the fourth is in (3, 0)");
    assert_eq!(s.w.rng, rng, "no draw");
}

/// Puts `id` on the bridge deck of its cell: out of the ground layer, into
/// the deck layer, 416 leptons up (S-0209 claim 15, S-0307 Constants).
fn to_deck(s: &mut S, id: ObjectId) {
    let c = s.w.obj(id).unwrap().pos.cell();
    s.w.cells.leave(c, glasnost_sim::world::Layer::Ground, id);
    s.w.cells
        .enter(c, glasnost_sim::world::Layer::Deck, id, false);
    let o = s.w.obj_mut(id).unwrap();
    o.layer = glasnost_sim::world::Layer::Deck;
    o.pos.z += 416;
}

// spec: S-0318 claim 1 (b), claim 3 and Edge cases (high bridges), scenario SH8 part f run 1 (T on the bridge deck over a cell whose ground layer lists nothing: no object is first-listed in the burst cell, so no fragment is launched and no draw R(-2, 2) is taken, though enemy vehicles stand on land within 3 cells)
#[test]
fn sh8f_run_1_a_target_on_the_deck_launches_no_fragments() {
    let mut s = setup(&base(3), true);
    around(&mut s);
    let t = s.t.unwrap();
    to_deck(&mut s, t);
    assert!(
        s.w.cells
            .list(cell(0, 0), glasnost_sim::world::Layer::Ground)
            .is_empty()
    );
    let pr = shoot_t(&mut s);
    let rng = s.w.rng.clone();
    burst(&mut s, &pr);
    assert!(frags(&s).is_empty(), "no fragments");
    assert_eq!(s.w.rng, rng, "no draw");
}

// spec: S-0318 claim 3 and Edge cases (high bridges), scenario SH8 part f run 2 (T on land beside the bridge, fragment count 1, the only other house-B vehicle on the deck over a cell of ring 1: the scan gives it no fragment, so the fragment goes by claim 4 with 2 draws R(-2, 2))
#[test]
fn sh8f_run_2_a_vehicle_on_the_deck_is_not_a_candidate() {
    let mut s = setup(&base(1), true);
    let deck = put(&mut s, 1, 0, -1);
    to_deck(&mut s, deck);
    let pr = shoot_t(&mut s);
    let (cells, after) = drawn(&s.w, 1);
    burst(&mut s, &pr);
    assert_eq!(targets(&s), cells, "the fragment goes to a drawn cell");
    assert_eq!(s.w.rng, after, "2 draws");
}

// spec: S-0318 Edge cases (a fragment range above 11 cells: the scan stops at ring 11, S-0303 table T1's last ring, and what remains goes by claim 4) and F2 (k = floor(R / 256))
#[test]
fn range_above_eleven_cells_scans_to_ring_eleven_only() {
    let mut r = base(2);
    frag(&mut r).range = Some(d("14"));
    let mut s = setup(&r, true);
    let near = put(&mut s, 1, 0, -11);
    let far = put(&mut s, 1, 0, -12);
    let pr = shoot_t(&mut s);
    let (cells, after) = drawn(&s.w, 1);
    burst(&mut s, &pr);
    let mut want = objs(&[near]);
    want.extend(cells);
    assert_eq!(
        targets(&s),
        want,
        "ring 11 is the last; ring 12 is not visited"
    );
    assert_eq!(s.w.rng, after, "one drawn pair for the second fragment");
    assert!(!targets(&s).contains(&Target::Object(far)));
}

// spec: S-0318 claim 1 (a) (a detonation that selects any special hit launches no fragments, whichever row of S-0514 claim 20's table it is; the rows other than bomb planting are tested here one by one, through the shot's own detonation)
#[test]
fn sh1n_every_special_hit_row_stops_the_burst() {
    type Set = fn(&mut WarheadType);
    let rows: [(&str, Set); 11] = [
        ("takeover", |h| h.mind_control = Some(true)),
        ("bomb", |h| h.ivan_bomb = Some(true)),
        ("charge", |h| h.electric_assault = Some(true)),
        ("parasite", |h| h.parasite = Some(true)),
        ("erase", |h| h.temporal = Some(true)),
        ("locomotor", |h| h.is_locomotor = Some(true)),
        ("airstrike", |h| h.airstrike = Some(true)),
        ("direct rock", |h| h.direct_rocker = Some(true)),
        ("disarm", |h| h.bomb_disarm = Some(true)),
        ("disguise", |h| h.makes_disguise = Some(true)),
        ("nuke", |h| h.nuke_maker = Some(true)),
    ];
    for (name, set) in rows {
        let mut r = base(3);
        set(r.sim.warheads.get_mut("Pin").unwrap());
        let mut s = setup(&r, true);
        let v = around(&mut s);
        let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
        let id = {
            let shot = wp(&s.w, "Shot");
            let p = s.w.obj(s.a).unwrap().pos;
            let t = s.t.unwrap();
            launch(&mut s.w, s.a, shot, Target::Object(t), p, SHOT).unwrap()
        };
        glasnost_sim::combat::projectile::turn(&mut s.w, id);
        testkit::run(&mut s.w, 3);
        assert!(frags(&s).is_empty(), "{name}: no fragments in play");
        assert_eq!(
            lost(&s, &v, &before),
            vec![0; 5],
            "{name}: no fragment damage"
        );
    }
}

// spec: S-0318 claim 1 (scenario SH1 part s run 2: on a map whose inert flag is set the detonation still bursts, the 6 ranged draws R(−2, 2) are still taken at the burst, and no object loses health, the fragments' own area damage doing nothing either); the control without the flag lets T and the fragments hit
#[test]
fn sh1s_run_2_inert_map_still_bursts_and_nothing_loses_health() {
    for inert in [false, true] {
        let mut s = setup(&base(8), true);
        s.w.map_flags.inert = inert;
        let v = around(&mut s);
        let t = s.t.unwrap();
        let t0 = health(&s, t);
        let before: Vec<i32> = v.iter().map(|&x| health(&s, x)).collect();
        let (_, after) = drawn(&s.w, 3);
        let pr = shoot_t(&mut s);
        burst(&mut s, &pr);
        assert_eq!(frags(&s).len(), 8, "the burst launched its fragments");
        assert_eq!(
            s.w.rng, after,
            "the 6 ranged draws were taken, inert or not"
        );
        testkit::run(&mut s.w, 4);
        if inert {
            assert_eq!(health(&s, t), t0, "T loses nothing");
            assert_eq!(lost(&s, &v, &before), vec![0; 5], "no fragment damage");
        } else {
            assert!(health(&s, t) < t0, "the control: T takes the shot");
            assert!(lost(&s, &v, &before).iter().any(|&x| x > 0));
        }
    }
}
