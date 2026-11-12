//! Claim tests for special-hit effects (S-0615) on synthetic data: invented
//! animation, warhead, weapon and unit types, every number our own. The
//! ledger is `docs/m7-claims/S-0615.md`.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, BuildingType, Decimal, UnitType, WarheadType, WeaponType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::{DiscEvent, Effect};
use glasnost_sim::disc;
use glasnost_sim::infantry::Seq;
use glasnost_sim::rules::Kind;
use glasnost_sim::walker::{self, Walker};
use glasnost_sim::world::{ObjectId, Pos, Target, World};
use glasnost_sim::{save, testkit};

const SIZE: i32 = 20;
/// The invented burning walker's running frames *R* and sprite frames *N*.
const R: i32 = 4;
const N: i32 = 80;
/// The invented disc weapon's damage and range (cells).
const DISC_DAMAGE: i32 = 40;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn verses() -> Option<Vec<Decimal>> {
    Some((0..11).map(|_| Decimal("1".into())).collect())
}

/// The rules: testkit's combat rules plus
/// - animations FIRST (registration index 0), FENCEDIE (index 1), BOOM,
///   BURN (a burning walker, R = 4, N = 80, damage 0.25), POP, GLOW, SMASH
///   and SPLASH, each of 10 frames unless said;
/// - the game-wide death animations BOOM, BURN, POP, GLOW and SMASH, the
///   splash list SPLASH, OTHER and the flame warhead FLAME;
/// - warheads DK-1 to DK11 of infantry-death kinds −1 to 11, and ZAP and
///   FLAME (versus 1, no spread);
/// - the disc weapon DISC (damage 40, range 6 cells, warhead ZAP, colours
///   (200, 100, 50), (100, 50, 25), (10, 10, 10)) on the vehicle SAUCER;
/// - FENCE, a laser-fence building; a wall overlay WALLY.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let plain = || AnimationType {
        frame_count: Some(10),
        ..Default::default()
    };
    for n in [
        "FIRST", "FENCEDIE", "BOOM", "POP", "GLOW", "SMASH", "SPLASH",
    ] {
        register(&mut r, "animations", n);
        r.sim.animations.insert(n.into(), plain());
    }
    register(&mut r, "animations", "BURN");
    r.sim.animations.insert(
        "BURN".into(),
        AnimationType {
            frame_count: Some(N),
            is_flaming_guy: Some(true),
            running_frames: Some(R),
            damage: Some(Decimal("0.25".into())),
            ..Default::default()
        },
    );
    let g = &mut r.sim.general;
    g.infantry_explode = Some("BOOM".into());
    g.flaming_infantry = Some("BURN".into());
    g.infantry_head_pop = Some("POP".into());
    g.infantry_nuked = Some("GLOW".into());
    g.infantry_brute = Some("SMASH".into());
    g.flame_damage2 = Some("FLAME".into());
    g.splash_list = Some(vec!["SPLASH".into(), "OTHER".into()]);
    g.disk_laser_charge_up = Some("HUM".into());
    for k in -1..=11 {
        let n = format!("DK{k}");
        register(&mut r, "warheads", &n);
        r.sim.warheads.insert(
            n,
            WarheadType {
                verses: verses(),
                inf_death: Some(k),
                ..Default::default()
            },
        );
    }
    for n in ["ZAP", "FLAME"] {
        register(&mut r, "warheads", n);
        r.sim.warheads.insert(
            n.into(),
            WarheadType {
                verses: verses(),
                ..Default::default()
            },
        );
    }
    r.sim.weapons.insert(
        "DISC".into(),
        WeaponType {
            damage: Some(DISC_DAMAGE),
            rof: Some(100),
            range: Some(Decimal("6".into())),
            speed: Some(100),
            projectile: Some("Bullet".into()),
            warhead: Some("ZAP".into()),
            disk_laser: Some(true),
            laser_inner_color: Some([200, 100, 50]),
            laser_outer_color: Some([100, 50, 25]),
            laser_outer_spread: Some([10, 10, 10]),
            ..Default::default()
        },
    );
    let mut saucer = UnitType {
        primary: Some("DISC".into()),
        ..r.sim.vehicles["CART"].clone()
    };
    // A muzzle point straight above the position: the ring centre does not depend on facing.
    saucer.combat.primary_fire_flh = Some([0, 0, 70]);
    register(&mut r, "vehicles", "SAUCER");
    r.sim.vehicles.insert("SAUCER".into(), saucer);
    register(&mut r, "buildings", "FENCE");
    r.sim.buildings.insert(
        "FENCE".into(),
        BuildingType {
            strength: Some(1000),
            foundation: Some([1, 1]),
            laser_fence: Some(true),
            ..Default::default()
        },
    );
    register(&mut r, "buildings", "BIG");
    r.sim.buildings.insert(
        "BIG".into(),
        BuildingType {
            strength: Some(5000),
            foundation: Some([2, 3]),
            ..Default::default()
        },
    );
    register(&mut r, "overlays", "WALLY");
    r.sim.overlays.insert(
        "WALLY".into(),
        glasnost_rules::sim::OverlayType {
            wall: Some(true),
            ..Default::default()
        },
    );
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn warhead(w: &World, name: &str) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap_or_else(|| panic!("no warhead {name}")) as u32
}

fn anim_index(w: &World, name: &str) -> u32 {
    w.rules()
        .animations
        .iter()
        .position(|a| a.name == name)
        .unwrap_or_else(|| panic!("no animation {name}")) as u32
}

fn kill(w: &mut World, id: ObjectId, wh: &str, attacker: Option<ObjectId>) {
    let h = warhead(w, wh);
    damage::apply(
        w,
        id,
        Hit {
            amount: 10_000,
            warhead: Some(h),
            distance: 0,
            attacker,
            house: None,
            ignore_defences: false,
            no_escape: false,
        },
    );
}

/// The animation names and walkers the effects show.
fn made(w: &World) -> (Vec<String>, Vec<String>) {
    let mut anims = Vec::new();
    let mut walkers = Vec::new();
    for e in &w.effects {
        match e {
            Effect::Anim { name, .. } => anims.push(name.clone()),
            Effect::Walker {
                name, made: true, ..
            } => walkers.push(name.clone()),
            _ => {}
        }
    }
    (anims, walkers)
}

fn set_land(w: &mut World, cell: CellXY, l: u8) {
    w.cells.grid.get_mut(cell).expect("legal cell").land = l;
}

fn set_deck(w: &mut World, cell: CellXY, on: bool) {
    let i = w.cells.idx(cell).expect("legal cell");
    w.cells.deck[i] = on;
}

fn the_walker(w: &World) -> (ObjectId, Walker) {
    let (id, wk) = w.walkers.iter().next().expect("a walker");
    (*id, wk.clone())
}

/// A walker of BURN made directly at the centre of `cell`, height `z`.
fn walker_at(w: &mut World, cell: CellXY, z: i32) -> ObjectId {
    let a = anim_index(w, "BURN");
    let p = Pos::centre(cell, z);
    walker::create(w, a, p);
    *w.walkers.keys().last().expect("made")
}

// ---------------------------------------------------------------------------
// Death kinds (claims 1 to 3)

// spec: S-0615 claim 1 (the death-kind table: 0 and 11 none, 1 and 2 the unit's sequences, 3 explode, 4 burning, 5 index 1, 6 head pop, 7 nuked, 8 S-0613, 9 S-0611, 10 brute) and claim 3 (kinds without an animation are removed at once)
#[test]
fn death_kind_table() {
    let r = rules();
    let expect: [(i32, &[&str], bool, bool); 13] = [
        (-1, &[], false, false),
        (0, &[], false, false),
        (1, &[], false, true),
        (2, &[], false, true),
        (3, &["BOOM"], false, false),
        (4, &[], true, false),
        (5, &["FENCEDIE"], false, false),
        (6, &["POP"], false, false),
        (7, &["GLOW"], false, false),
        (8, &[], false, false),
        (9, &[], false, false),
        (10, &["SMASH"], false, false),
        (11, &[], false, false),
    ];
    for (k, anims, walks, stays) in expect {
        let mut w = game(&r);
        let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
        w.effects.clear();
        kill(&mut w, id, &format!("DK{k}"), None);
        let (a, wk) = made(&w);
        assert_eq!(a, anims, "kind {k}");
        assert_eq!(wk.len(), usize::from(walks), "kind {k}");
        // Kinds 1 and 2 request the unit's own death sequence: our WALKER has none, so with
        // quirks.refused_death_keeps_unit the unit stays (S-0107 claim 17); the others are removed.
        // Kind 9 converts on this open ground (S-0611 claims 6 and 7): the unit leaves play at
        // once and no death animation of this table is made.
        assert_eq!(w.obj(id).is_some(), stays, "kind {k}");
    }
}

// spec: S-0615 claim 1 (S-0107 claim 17 sends a laser-fence kill to kind 5 and a parachuting unit to kind 3, so they make those animations)
#[test]
fn laser_fence_and_parachute_kinds() {
    let r = rules();
    let mut w = game(&r);
    let fence = testkit::spawn(&mut w, Kind::Building, "FENCE", 1, c(4, 0), 0);
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
    w.effects.clear();
    kill(&mut w, id, "DK6", Some(fence));
    assert_eq!(made(&w).0, ["FENCEDIE"]);
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 2), 0);
    w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Parachute);
    w.effects.clear();
    kill(&mut w, id, "DK6", None);
    assert_eq!(made(&w).0, ["BOOM"]);
    assert!(w.obj(id).is_none());
}

// spec: S-0615 claim 1 and Edge cases (kind 5 by registration index 1: changing index 1 changes the animation; a change at another index does not)
#[test]
fn kind_five_follows_registration_index_one() {
    let mut r = rules();
    let list = r.registries.get_mut("animations").unwrap();
    // The control: swap indices 2 and 3; index 1 is still FENCEDIE.
    list.swap(2, 3);
    let mut w = game(&r);
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
    w.effects.clear();
    kill(&mut w, id, "DK5", None);
    assert_eq!(made(&w).0, ["FENCEDIE"]);
    // Index 1 changed to POP.
    let list = r.registries.get_mut("animations").unwrap();
    let pop = list.iter().position(|n| n == "POP").unwrap();
    list.swap(1, pop);
    let mut w = game(&r);
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
    w.effects.clear();
    kill(&mut w, id, "DK5", None);
    assert_eq!(made(&w).0, ["POP"]);
}

// spec: S-0615 claim 2 (one animation at the unit's position; the dispatch adds no draw; the creation draws are the usual conditional ones)
#[test]
fn death_animation_position_and_draws() {
    let mut r = rules();
    // BOOM bounces: its creation takes S-0202 claim 10's three raw values and three ranged draws.
    r.sim.animations.get_mut("BOOM").unwrap().bouncer = Some(true);
    let mut w = game(&r);
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(1, 2), 0);
    let at = w.obj(id).unwrap().pos;
    // POP: no draws at creation.
    w.effects.clear();
    let before = w.rng.clone();
    kill(&mut w, id, "DK6", None);
    assert_eq!(
        w.rng, before,
        "the dispatch and an undrawn creation take no draw"
    );
    let pos: Vec<Pos> = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { pos, .. } => Some(*pos),
            _ => None,
        })
        .collect();
    assert_eq!(pos, [at]);
    // BOOM: exactly the creation's draws.
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(1, 4), 0);
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut expected = w.rng.clone();
    for _ in 0..3 {
        let _ = expected.raw();
    }
    for _ in 0..3 {
        let _ = expected.ranged(-65535, 65535, half);
    }
    kill(&mut w, id, "DK3", None);
    assert_eq!(w.rng, expected, "only BOOM's creation draws");
}

// spec: S-0615 claim 2 (a kind whose type the rules data does not name, or names but does not list, still removes the unit with no animation)
#[test]
fn missing_death_type_still_removes() {
    let mut r = rules();
    r.sim.general.infantry_nuked = None;
    r.sim.general.infantry_brute = Some("NOWHERE".into());
    let mut w = game(&r);
    for k in ["DK7", "DK10"] {
        let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
        w.effects.clear();
        kill(&mut w, id, k, None);
        assert_eq!(made(&w), (vec![], vec![]), "{k}");
        assert!(w.obj(id).is_none(), "{k}");
    }
}

// spec: S-0615 claim 3 (kinds 0 and 11 and above: no animation, removed at once, no dispatch draw; a negative kind likewise, S-0615 r3)
#[test]
fn kinds_without_animation() {
    let r = rules();
    let mut w = game(&r);
    for k in ["DK0", "DK11", "DK-1"] {
        let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
        w.effects.clear();
        let before = w.rng.clone();
        kill(&mut w, id, k, None);
        assert_eq!(w.rng, before, "{k}");
        assert_eq!(made(&w), (vec![], vec![]), "{k}");
        assert!(w.obj(id).is_none(), "{k}");
    }
}

// ---------------------------------------------------------------------------
// The burning walker (claims 4 to 11)

// spec: S-0615 claim 1 and claim 4 (kind 4 with a flagged type makes a burning walker in the update order; its first update chooses a destination without moving; its stage stays frozen while it runs)
#[test]
fn walker_first_update_and_frozen_stage() {
    let r = rules();
    let mut w = game(&r);
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
    let at = w.obj(id).unwrap().pos;
    kill(&mut w, id, "DK4", None);
    let (wid, wk) = the_walker(&w);
    assert_eq!(wk.pos, at);
    assert_eq!(wk.dest, None);
    testkit::run(&mut w, 1);
    let wk = w.walkers[&wid].clone();
    assert_eq!(wk.pos, at, "no movement in the first update");
    assert!(wk.dest.is_some(), "a destination was chosen");
    for _ in 0..30 {
        testkit::run(&mut w, 1);
        let Some(wk) = w.walkers.get(&wid) else { break };
        if !wk.expiring {
            assert_eq!(wk.stage, 0, "frozen while running");
        }
    }
}

// spec: S-0615 claim 4 and F1 (frame = R (7 − d) + (⌊t / 3⌋ mod R); d = 7 with no destination; north d = 0 uses cycle 7)
#[test]
fn walker_frame_formula() {
    let mut wk = Walker {
        anim: 0,
        pos: Pos::new(1000, 1000, 0),
        dest: None,
        retries: 0,
        expiring: false,
        stage: 0,
        falling: false,
        rate: 0,
        acc: glasnost_num::F64::ZERO,
    };
    // No destination: cycle 0.
    assert_eq!(walker::frame(&wk, R, 0, true), 0);
    assert_eq!(walker::frame(&wk, R, 3, true), 1);
    assert_eq!(walker::frame(&wk, R, 11, true), 3);
    assert_eq!(walker::frame(&wk, R, 12, true), 0);
    // North (smaller y): d = 0, cycle 7.
    wk.dest = Some(Pos::new(1000, 500, 0));
    assert_eq!(walker::frame(&wk, R, 6, true), 7 * R + 2);
    // East: d = 2, cycle 5.
    wk.dest = Some(Pos::new(1500, 1000, 0));
    assert_eq!(walker::frame(&wk, R, 0, true), 5 * R);
    // South-east: d = 3, cycle 4.
    wk.dest = Some(Pos::new(1500, 1500, 0));
    assert_eq!(walker::frame(&wk, R, 0, true), 4 * R);
    // Expiring: the stage.
    wk.expiring = true;
    wk.stage = 33;
    assert_eq!(walker::frame(&wk, R, 0, true), 33);
}

// spec: S-0615 claim 4 (a first choice of none starts expiry at once) and claim 6 (no passable neighbour in the fallback: none)
#[test]
fn walker_boxed_in_expires_at_once() {
    let r = rules();
    let mut w = game(&r);
    for (dx, dy) in glasnost_sim::regions::STEPS {
        set_land(&mut w, c(dx, dy), land::ROCK);
    }
    let wid = walker_at(&mut w, c(0, 0), 0);
    testkit::run(&mut w, 1);
    let wk = w.walkers[&wid].clone();
    assert!(wk.expiring);
    assert_eq!(wk.dest, None);
    assert_eq!(wk.stage, 8 * R + 1);
}

// spec: S-0615 claim 5 and F2 (the search square −5 … +4; water only; the south-east bias; ties to the first in scan order, x outer)
#[test]
fn water_search_square_bias_and_ties() {
    let r = rules();
    let found = |water: &[(i32, i32)]| {
        let mut w = game(&r);
        for &(dx, dy) in water {
            set_land(&mut w, c(dx, dy), land::WATER);
        }
        walker::water_search(&w, c(0, 0)).map(|b| (b.x - c(0, 0).x, b.y - c(0, 0).y))
    };
    assert_eq!(found(&[]), None);
    // Boundaries: −5 and +4 are in the square, +5 is not.
    assert_eq!(found(&[(-5, 0)]), Some((-5, 0)));
    assert_eq!(found(&[(0, -5)]), Some((0, -5)));
    assert_eq!(found(&[(4, 0)]), Some((4, 0)));
    assert_eq!(found(&[(5, 0)]), None);
    assert_eq!(found(&[(0, 5)]), None);
    // The bias: (2, 0) scores 2 − 3 = −1 and beats (−1, 0) at 1.
    assert_eq!(found(&[(-1, 0), (2, 0)]), Some((2, 0)));
    // Own cell: score 0 beats (0, −1) at 1.
    assert_eq!(found(&[(0, -1), (0, 0)]), Some((0, 0)));
    // A tie at −1: (−2, 1) and (1, −2) (√5 truncates to 2); x outer, so (−2, 1) first.
    assert_eq!(found(&[(1, -2), (-2, 1)]), Some((-2, 1)));
    // Not water: beach.
    let mut w = game(&r);
    set_land(&mut w, c(1, 0), land::BEACH);
    assert_eq!(walker::water_search(&w, c(0, 0)), None);
}

// spec: S-0615 claim 5 (water under a bridge deck, or with a deck at the neighbour in step direction 6 or 0, is not a candidate; other neighbours do not matter)
#[test]
fn water_search_excludes_decks() {
    let r = rules();
    for (deck, ok) in [
        ((2, 0), false),
        ((1, 0), false),
        ((2, -1), false),
        ((3, 0), true),
        ((2, 1), true),
    ] {
        let mut w = game(&r);
        set_land(&mut w, c(2, 0), land::WATER);
        set_deck(&mut w, c(deck.0, deck.1), true);
        assert_eq!(
            walker::water_search(&w, c(0, 0)).is_some(),
            ok,
            "deck at {deck:?}"
        );
    }
}

// spec: S-0615 claim 6 (the diagonal step towards the best cell, then x only, then y only; the destination is the centre at height 0; no fallback draw when one is passable)
#[test]
fn step_towards_the_best_cell() {
    let r = rules();
    let mut w = game(&r);
    set_land(&mut w, c(3, 3), land::WATER);
    let at = Pos::centre(c(0, 0), 0);
    let before = w.rng.clone();
    assert_eq!(walker::choose(&mut w, at), Some(Pos::centre(c(1, 1), 0)));
    set_land(&mut w, c(1, 1), land::ROCK);
    assert_eq!(walker::choose(&mut w, at), Some(Pos::centre(c(1, 0), 0)));
    set_land(&mut w, c(1, 0), land::TUNNEL);
    assert_eq!(walker::choose(&mut w, at), Some(Pos::centre(c(0, 1), 0)));
    assert_eq!(w.rng, before, "no draw while a step is passable");
    // All three blocked: the fallback's one draw.
    set_land(&mut w, c(0, 1), land::WALL);
    let mut expect = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let rr = expect.ranged(0, 7, half);
    let got = walker::choose(&mut w, at).expect("a neighbour");
    assert_eq!(w.rng, expect);
    let first = (0..8)
        .map(|i| {
            let (dx, dy) = glasnost_sim::regions::STEPS[((rr + i) % 8) as usize];
            c(dx, dy)
        })
        .find(|&n| walker::passable(&w, n, 0))
        .unwrap();
    assert_eq!(got, Pos::centre(first, 0));
    // The best cell is the walker's own: all three candidates are that cell.
    let mut w = game(&r);
    set_land(&mut w, c(0, 0), land::WATER);
    let before = w.rng.clone();
    assert_eq!(walker::choose(&mut w, at), Some(Pos::centre(c(0, 0), 0)));
    assert_eq!(w.rng, before);
}

// spec: S-0615 claim 6 (the fallback: one ranged draw r in [0, 7], then neighbours in cyclic order r, r + 1, … mod 8)
#[test]
fn fallback_cyclic_order() {
    let r = rules();
    let mut w = game(&r);
    let at = Pos::centre(c(0, 0), 0);
    let half = w.rules().quirks.ranged_draw_half_span;
    for _ in 0..12 {
        let mut probe = w.rng.clone();
        let rr = probe.ranged(0, 7, half);
        let (dx, dy) = glasnost_sim::regions::STEPS[rr as usize];
        assert_eq!(walker::choose(&mut w, at), Some(Pos::centre(c(dx, dy), 0)));
        // Block the cell the draw gives next time, so the order is exercised.
        let mut probe = w.rng.clone();
        let nr = probe.ranged(0, 7, half);
        let (bx, by) = glasnost_sim::regions::STEPS[nr as usize];
        let (nx, ny) = glasnost_sim::regions::STEPS[((nr + 1) % 8) as usize];
        let mut w2 = w.clone();
        set_land(&mut w2, c(bx, by), land::ROCK);
        assert_eq!(walker::choose(&mut w2, at), Some(Pos::centre(c(nx, ny), 0)));
        let _ = walker::choose(&mut w, at);
    }
}

// spec: S-0615 claim 7 (passable: not rock, tunnel or wall land; no wall overlay; no vehicle, aircraft or building on the ground layer, infantry does not block; 104 L − z ≤ 208; on a deck when z − 104 L > 208, only the deck layer's occupants count)
#[test]
fn passable_cells() {
    let r = rules();
    let mut w = game(&r);
    let p = |w: &World, dx, dy, z| walker::passable(w, c(dx, dy), z);
    assert!(p(&w, 1, 0, 0));
    for (i, l) in [land::ROCK, land::TUNNEL, land::WALL]
        .into_iter()
        .enumerate()
    {
        set_land(&mut w, c(1, i as i32), l);
        assert!(!p(&w, 1, i as i32, 0));
    }
    set_land(&mut w, c(1, 3), land::WATER);
    assert!(p(&w, 1, 3, 0));
    let wall = w
        .rules()
        .overlays
        .iter()
        .position(|o| o.name == "WALLY")
        .unwrap() as u8;
    w.cells.grid.get_mut(c(2, 0)).unwrap().overlay = wall;
    assert!(!p(&w, 2, 0, 0));
    testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(3, 0), 0);
    assert!(p(&w, 3, 0, 0), "infantry does not block");
    testkit::spawn(&mut w, Kind::Vehicle, "CART", 0, c(3, 1), 0);
    assert!(!p(&w, 3, 1, 0));
    testkit::spawn(&mut w, Kind::Building, "FENCE", 1, c(3, 2), 0);
    assert!(!p(&w, 3, 2, 0));
    // Height: level 2 is reachable from 0 (208 ≤ 208), level 3 is not.
    w.cells.grid.get_mut(c(4, 0)).unwrap().level = 2;
    w.cells.grid.get_mut(c(4, 1)).unwrap().level = 3;
    assert!(p(&w, 4, 0, 0));
    assert!(!p(&w, 4, 1, 0));
    assert!(p(&w, 4, 1, 104));
    // On a deck: a ground vehicle under it does not block a walker at deck height.
    set_deck(&mut w, c(3, 1), true);
    assert!(p(&w, 3, 1, 416));
    assert!(!p(&w, 3, 1, 208), "208 above level 0 is not on the deck");
    // Rock under a deck does not matter to a walker on it.
    set_deck(&mut w, c(1, 0), true);
    assert!(p(&w, 1, 0, 416));
}

// spec: S-0615 claim 8 and F3 (a step of S-0208 F3 with s = 18 while the table-root distance is 19 or more; the height is the terrain's, or 416 above it when the old height was more than 208 above it)
#[test]
fn running_step_and_height() {
    let r = rules();
    let mut w = game(&r);
    set_land(&mut w, c(3, 1), land::WATER);
    let wid = walker_at(&mut w, c(0, 0), 0);
    testkit::run(&mut w, 1);
    let wk = w.walkers[&wid].clone();
    let d = wk.dest.expect("a destination");
    assert_eq!(d, Pos::centre(c(1, 1), 0));
    testkit::run(&mut w, 1);
    let q = w.rules().quirks;
    let f = glasnost_num::geom::direction(
        i64::from(d.x - wk.pos.x),
        i64::from(d.y - wk.pos.y),
        q.table_direction,
    );
    let (x, y) = glasnost_sim::walk::step_point(wk.pos.x, wk.pos.y, f, 18, true, w.rules().mode);
    assert_eq!(w.walkers[&wid].pos, Pos::new(x, y, 0));
    // The height rule on a deck: a walker 416 above keeps to deck height.
    let mut w = game(&r);
    for dx in -2..=2 {
        for dy in -2..=2 {
            set_deck(&mut w, c(dx, dy), true);
        }
    }
    w.cells.grid.get_mut(c(0, 0)).unwrap().level = 1;
    let wid = walker_at(&mut w, c(0, 0), 104 + 416);
    testkit::run(&mut w, 2);
    let wk = w.walkers[&wid].clone();
    assert!(!wk.expiring);
    let t = w.surface_at(wk.pos.x, wk.pos.y);
    assert_eq!(wk.pos.z, t + 416);
}

// spec: S-0615 claim 9 (on arrival: a new choice; the retry count rises and the walker is placed on the old destination; it expires at its eighth destination, after at most nine fallback draws)
#[test]
fn arrivals_and_retry_limit() {
    let r = rules();
    let mut w = game(&r);
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut expect = w.rng.clone();
    let wid = walker_at(&mut w, c(0, 0), 0);
    let mut dests = Vec::new();
    for _ in 0..1000 {
        testkit::run(&mut w, 1);
        let wk = w.walkers[&wid].clone();
        if let Some(d) = wk.dest
            && dests.last() != Some(&d)
        {
            if let Some(&old) = dests.last() {
                assert_eq!(wk.pos.x, old.x, "placed on the old destination");
                assert_eq!(wk.pos.y, old.y);
            }
            dests.push(d);
        }
        if wk.expiring {
            assert_eq!(wk.retries, 7);
            assert_eq!(wk.dest, None, "expiry clears the destination");
            break;
        }
    }
    assert_eq!(dests.len(), 8, "eight destinations in all");
    for _ in 0..9 {
        let _ = expect.ranged(0, 7, half);
    }
    assert_eq!(w.rng, expect, "nine fallback draws, nothing else");
}

// spec: S-0615 claim 9 (arrival at a destination whose cell is water or beach, at most 104 above the terrain: expiry without moving)
#[test]
fn arrival_on_water_or_beach_expires() {
    let r = rules();
    let mut w = game(&r);
    set_land(&mut w, c(1, 0), land::WATER);
    let wid = walker_at(&mut w, c(0, 0), 0);
    let mut last = None;
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        let wk = w.walkers[&wid].clone();
        if wk.expiring {
            assert_eq!(wk.retries, 0);
            assert_eq!(Some(wk.pos), last, "no move in the expiring update");
            break;
        }
        last = Some(wk.pos);
    }
    assert!(w.walkers[&wid].expiring);
    // Beach: every neighbour is beach, so the fallback picks one.
    let mut w = game(&r);
    for (dx, dy) in glasnost_sim::regions::STEPS {
        set_land(&mut w, c(dx, dy), land::BEACH);
    }
    let wid = walker_at(&mut w, c(0, 0), 0);
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        if w.walkers[&wid].expiring {
            break;
        }
    }
    assert!(w.walkers[&wid].expiring);
    assert_eq!(w.walkers[&wid].retries, 0);
}

/// Decks over a square round the middle; the walker on them at deck
/// height.
fn decked(r: &Ruleset) -> (World, ObjectId) {
    let mut w = game(r);
    for dx in -4..=4 {
        for dy in -4..=4 {
            set_deck(&mut w, c(dx, dy), true);
            set_land(&mut w, c(dx, dy), land::WATER);
        }
    }
    let wid = walker_at(&mut w, c(0, 0), 416);
    testkit::run(&mut w, 1);
    assert!(w.walkers[&wid].dest.is_some());
    (w, wid)
}

// spec: S-0615 claim 10 (a walker at least 416 above the terrain with no deck at its cell or neighbours 6 and 0 starts to fall after a step; it falls by S-0511, keeps running steps without choosing, and landing starts expiry; on water the first splash animation 3 above)
#[test]
fn lost_deck_fall_and_splash() {
    let r = rules();
    let (mut w, wid) = decked(&r);
    for dx in -4..=4 {
        for dy in -4..=4 {
            set_deck(&mut w, c(dx, dy), false);
        }
    }
    let dest = w.walkers[&wid].dest;
    let z0 = w.walkers[&wid].pos.z;
    testkit::run(&mut w, 1);
    let wk = w.walkers[&wid].clone();
    assert!(wk.falling, "falls after the step");
    // spec: S-0615 claim 10 (revision 3, First descent and Starting rate: the descent runs in the update that starts the fall, from rate 0, so the height keeps the deck height and only the rate follows S-0511 F3; the first drop is in the next update)
    assert_eq!(wk.pos.z, z0, "no drop in the start update");
    let v1 = glasnost_sim::fall::next_rate(&w, 0);
    assert_eq!(wk.rate, v1, "the rate already updated");
    let x = w.walkers[&wid].pos.z;
    testkit::run(&mut w, 1);
    if w.walkers[&wid].falling {
        assert_eq!(
            w.walkers[&wid].pos.z,
            x + v1,
            "the first drop by the updated rate"
        );
    }
    let mut landed = false;
    for _ in 0..200 {
        w.effects.clear();
        let before = w.walkers[&wid].clone();
        testkit::run(&mut w, 1);
        let wk = w.walkers[&wid].clone();
        if wk.falling {
            assert_eq!(wk.dest, dest, "no new destination while falling");
            assert!(wk.rate <= before.rate);
            continue;
        }
        assert!(wk.expiring, "landing starts expiry");
        assert_eq!(wk.pos.z, 0);
        let splash: Vec<Pos> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Anim { name, pos, .. } if name == "SPLASH" => Some(*pos),
                _ => None,
            })
            .collect();
        assert_eq!(splash, [Pos::new(wk.pos.x, wk.pos.y, 3)]);
        landed = true;
        break;
    }
    assert!(landed);
}

// spec: S-0615 claim 10 (descent follows S-0511 claim 10 and F3: z changes by v, then v follows the recurrence) and claim 8 (no running step near the destination: uninterrupted descent)
#[test]
fn descent_follows_s0511() {
    let r = rules();
    let (mut w, wid) = decked(&r);
    // Put the walker on its destination's point, so no running step follows.
    let d = w.walkers[&wid].dest.unwrap();
    w.walkers.get_mut(&wid).unwrap().pos = Pos::new(d.x, d.y, 416);
    for dx in -4..=4 {
        for dy in -4..=4 {
            set_deck(&mut w, c(dx, dy), false);
        }
    }
    w.walkers.get_mut(&wid).unwrap().falling = true;
    let mut z = 416;
    let mut v = 0;
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        let wk = w.walkers[&wid].clone();
        z += v;
        v = glasnost_sim::fall::next_rate(&w, v);
        if z < 1 {
            assert!(wk.expiring);
            assert_eq!(wk.pos.z, 0);
            assert_eq!(wk.dest, Some(d), "expiry on landing keeps the destination");
            return;
        }
        assert_eq!((wk.pos.z, wk.rate), (z, v));
    }
    panic!("never landed");
}

// spec: S-0615 claim 11 and S-0109 claim 17, F5 (expiry: stage 8R + 1, one stage per update from a later update, removed at the start of an update with stage ≥ ⌊N / 2⌋ − 1; damage over time on expiring advances only, δ = 0.25 gives 1, 0, 0, 1, …; FlameDamage2, no attacker)
#[test]
fn expiry_lifetime_and_damage() {
    let r = rules();
    let mut w = game(&r);
    for (dx, dy) in glasnost_sim::regions::STEPS {
        set_land(&mut w, c(dx, dy), land::ROCK);
    }
    let wid = walker_at(&mut w, c(0, 0), 0);
    let mut stages = Vec::new();
    let mut damage = Vec::new();
    for _ in 0..20 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        let dealt: Vec<i32> = w
            .effects
            .iter()
            .filter_map(|e| match e {
                Effect::Detonation { damage, .. } => Some(*damage),
                _ => None,
            })
            .collect();
        match w.walkers.get(&wid) {
            Some(wk) => {
                stages.push(wk.stage);
                damage.push(dealt.first().copied().unwrap_or(0));
            }
            None => {
                assert!(dealt.is_empty());
                assert!(
                    w.effects
                        .iter()
                        .any(|e| matches!(e, Effect::Walker { made: false, .. }))
                );
                break;
            }
        }
    }
    let s0 = 8 * R + 1;
    assert_eq!(stages, (s0..=N / 2 - 1).collect::<Vec<_>>());
    // The first update starts expiry with no advance and no damage.
    assert_eq!(damage, [0, 1, 0, 0, 1, 0, 0]);
    // quirks.anim_damage_starts_at_one off: A₀ = 0, so the first delivery is on the fourth advance.
    let mut r2 = rules();
    r2.globals.quirks.anim_damage_starts_at_one = false;
    let mut w = game(&r2);
    for (dx, dy) in glasnost_sim::regions::STEPS {
        set_land(&mut w, c(dx, dy), land::ROCK);
    }
    let wid = walker_at(&mut w, c(0, 0), 0);
    let mut damage = Vec::new();
    while w.walkers.contains_key(&wid) {
        w.effects.clear();
        testkit::run(&mut w, 1);
        let n = w
            .effects
            .iter()
            .filter(|e| matches!(e, Effect::Detonation { .. }))
            .count();
        damage.push(n);
    }
    assert_eq!(damage, [0, 0, 0, 0, 1, 0, 0, 0]);
}

// spec: S-0615 claim 11 (a running walker delivers no damage over time) and S-0109 claim 17
#[test]
fn running_walker_deals_no_damage() {
    let r = rules();
    let mut w = game(&r);
    let wid = walker_at(&mut w, c(0, 0), 0);
    for _ in 0..60 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        let wk = w.walkers[&wid].clone();
        if wk.expiring {
            break;
        }
        assert!(
            !w.effects
                .iter()
                .any(|e| matches!(e, Effect::Detonation { .. }))
        );
    }
}

// ---------------------------------------------------------------------------
// The disc laser (claims 12 to 18)

/// A SAUCER of house 0 at the middle and a target of `kind`/`ty` of house 1
/// `dx` cells east.
fn duel(r: &Ruleset, kind: Kind, ty: &str, dx: i32) -> (World, ObjectId, ObjectId) {
    let mut w = game(r);
    let f = testkit::spawn(&mut w, Kind::Vehicle, "SAUCER", 0, c(0, 0), 0);
    let t = testkit::spawn(&mut w, kind, ty, 1, c(dx, 0), 0);
    (w, f, t)
}

fn shoot(w: &mut World, f: ObjectId, t: ObjectId) {
    assert!(glasnost_sim::combat::fire::shot(w, f, 0, Target::Object(t)));
}

fn discs(w: &World) -> Vec<&DiscEvent> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Disc { event, .. } => Some(event),
            _ => None,
        })
        .collect()
}

fn beams(w: &World) -> Vec<(Pos, Pos, i32, bool)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Beam {
                from,
                to,
                duration,
                fade_out,
                ..
            } => Some((*from, *to, *duration, *fade_out)),
            _ => None,
        })
        .collect()
}

// spec: S-0615 claim 12 (a disc-laser shot makes a disc laser instead of a projectile: no report at the shot, no ammunition used; it holds D_s and the start point of F4)
#[test]
fn disc_shot_makes_no_projectile() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    w.obj_mut(f).unwrap().combat.ammo = 3;
    w.effects.clear();
    shoot(&mut w, f, t);
    assert!(w.projectiles.is_empty());
    assert_eq!(w.discs.len(), 1);
    assert!(!w.effects.iter().any(|e| matches!(e, Effect::Shot { .. })));
    assert_eq!(discs(&w), [&DiscEvent::Shot]);
    assert_eq!(w.obj(f).unwrap().combat.ammo, 3);
    let d = &w.discs[0];
    assert_eq!(
        d.damage,
        glasnost_sim::combat::damage::shot_damage(&w, f, d.weapon)
    );
    // Target due east: the 16-step facing 4, so i₀ = 12 (west of the firer).
    assert_eq!(d.start, 12);
}

// spec: S-0615 F4 (i₀ = (⌊(⌊f / 2048⌋ + 1) / 2⌋ mod 16 + 8) mod 16)
#[test]
fn start_point_formula() {
    assert_eq!(disc::start_point(0), 8);
    assert_eq!(disc::start_point(16384), 12);
    assert_eq!(disc::start_point(32768), 0);
    assert_eq!(disc::start_point(49152), 4);
    // Half-way rounds up: 2048 is the boundary between 0 and 1.
    assert_eq!(disc::start_point(2047), 8);
    assert_eq!(disc::start_point(2048), 9);
    // Near a full turn: back to 0.
    assert_eq!(disc::start_point(65535), 8);
}

// spec: S-0615 claim 13 and F5 (steps in T + 1, + 3, …, + 15, the final step with the damage in T + 17, removal in T + 18; nothing in the other updates)
#[test]
fn disc_schedule() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    let h0 = w.obj(t).unwrap().health;
    shoot(&mut w, f, t);
    for n in 1..=18 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        let b = beams(&w);
        let ev = discs(&w);
        if n <= 15 && n % 2 == 1 {
            assert_eq!(b.len(), 2, "update {n}");
            assert_eq!(
                ev.len(),
                usize::from(n == 1),
                "update {n}: the charge-up at k = 0 only"
            );
            assert_eq!(w.obj(t).unwrap().health, h0, "update {n}");
        } else if n == 17 {
            assert_eq!(b.len(), 1);
            assert!(matches!(ev[..], [DiscEvent::Hit { .. }]));
            assert!(w.obj(t).unwrap().health < h0, "the damage lands in T + 17");
            assert_eq!(w.discs.len(), 1, "removed at its next update");
        } else {
            assert!(b.is_empty() && ev.is_empty(), "update {n}");
        }
    }
    assert!(w.discs.is_empty(), "removed in T + 18");
}

// spec: S-0615 claim 13 and S-0203 row 12 (disc lasers update newest first)
#[test]
fn newest_disc_first() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    let g = testkit::spawn(&mut w, Kind::Vehicle, "SAUCER", 0, c(0, 2), 0);
    shoot(&mut w, f, t);
    shoot(&mut w, g, t);
    assert_eq!(w.discs[0].firer, g);
    w.effects.clear();
    testkit::run(&mut w, 1);
    let firers: Vec<ObjectId> = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Disc { firer, .. } => Some(*firer),
            _ => None,
        })
        .collect();
    assert_eq!(firers, [g, f]);
}

// spec: S-0615 claim 14 (at a step, a distance over the range field cancels with no damage and nothing drawn; an excursion between steps does not; equality passes)
#[test]
fn range_recheck_cancels() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    let h0 = w.obj(t).unwrap().health;
    shoot(&mut w, f, t);
    testkit::run(&mut w, 1);
    // Out of range during the even update 2 only: no effect.
    let home = w.obj(t).unwrap().pos;
    w.obj_mut(t).unwrap().pos.x += 256 * 10;
    testkit::run(&mut w, 1);
    w.obj_mut(t).unwrap().pos = home;
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert_eq!(
        beams(&w).len(),
        2,
        "the excursion between steps did not cancel"
    );
    // Out of range at step 2 (update 5).
    w.obj_mut(t).unwrap().pos.x += 256 * 10;
    testkit::run(&mut w, 1);
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert!(beams(&w).is_empty());
    assert_eq!(discs(&w), [&DiscEvent::Cancelled]);
    assert!(w.discs.is_empty());
    w.obj_mut(t).unwrap().pos = home;
    testkit::run(&mut w, 15);
    assert_eq!(w.obj(t).unwrap().health, h0);
    // Equality passes: the range field is 6 cells = 1536 leptons.
    let range = w
        .rules()
        .combat
        .weapon(
            w.rules()
                .combat
                .weapons
                .iter()
                .position(|x| x.name == "DISC")
                .unwrap() as u32,
        )
        .range;
    assert_eq!(range, 1536);
}

// spec: S-0615 claim 14 (revision 3, Points measured: the three-dimensional table-root distance from the firer's position to the target's position; the vertical difference is 0 for a firer airborne in S-0308's sense; a building target less 64 (W + H), clamped at 0)
#[test]
fn recheck_distance_forms() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    let tt = Target::Object(t);
    let flat = disc::recheck_distance(&w, f, tt);
    assert_eq!(flat, 768);
    // Height: the target 300 up counts for a grounded firer.
    w.obj_mut(t).unwrap().pos.z = 300;
    let up = disc::recheck_distance(&w, f, tt);
    assert!(up > flat);
    // An ordinary vehicle firer below 208 leptons is not airborne in S-0308's sense: it counts the vertical difference.
    w.obj_mut(f).unwrap().pos.z = 100;
    w.obj_mut(t).unwrap().pos.z = 0;
    assert!(disc::recheck_distance(&w, f, tt) > flat);
    // spec: S-0308 Terms (Airborne: for an object that is not an aircraft, S-0307's height test; #178) — the same vehicle 600 up is airborne: the vertical difference is 0.
    w.obj_mut(f).unwrap().pos.z = 600;
    assert_eq!(disc::recheck_distance(&w, f, tt), flat);
    // A jumpjet firer 600 up is in the air (S-0502 claim 30): the vertical difference is 0.
    let mut rj = r.clone();
    rj.sim.vehicles.get_mut("SAUCER").unwrap().locomotor =
        Some(glasnost_rules::sim::Locomotor::Jumpjet);
    let (mut w, f, t) = duel(&rj, Kind::Vehicle, "CART", 3);
    assert!(glasnost_sim::jump::is_jumpjet(&w, f));
    w.obj_mut(f).unwrap().pos.z = 600;
    assert_eq!(disc::recheck_distance(&w, f, Target::Object(t)), flat);
    // A 2 × 3 building, measured at its position (not its reference point): 64 × 5 = 320 less.
    let (w, f, b) = duel(&r, Kind::Building, "BIG", 4);
    assert_ne!(w.reference_point(b), w.obj(b).unwrap().pos);
    let raw = {
        let s = w.obj(f).unwrap().pos;
        let m = w.obj(b).unwrap().pos;
        let n = (i64::from(s.x - m.x).pow(2)
            + i64::from(s.y - m.y).pow(2)
            + i64::from(s.z - m.z).pow(2)) as u64;
        glasnost_num::geom::root_floor(n, w.rules().quirks.table_root_distance, w.rules().mode)
            as i32
    };
    assert_eq!(disc::recheck_distance(&w, f, Target::Object(b)), raw - 320);
    // Clamped at 0 next to it.
    let (mut w, f, b) = duel(&r, Kind::Building, "BIG", 1);
    let m = w.obj(b).unwrap().pos;
    w.obj_mut(f).unwrap().pos = Pos::new(m.x + 100, m.y, w.obj(f).unwrap().pos.z);
    assert_eq!(disc::recheck_distance(&w, f, Target::Object(b)), 0);
}

// spec: S-0615 claim 14 (cancelled without damage when the firer or the target leaves play, or when the firer has begun a crash)
#[test]
fn loss_and_crash_cancel() {
    let r = rules();
    // The target leaves play.
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    testkit::run(&mut w, 4);
    glasnost_sim::objects::remove(&mut w, t);
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert!(w.discs.is_empty());
    assert_eq!(discs(&w), [&DiscEvent::Cancelled]);
    // The firer leaves the map.
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    let h0 = w.obj(t).unwrap().health;
    shoot(&mut w, f, t);
    testkit::run(&mut w, 4);
    w.obj_mut(f).unwrap().on_map = false;
    testkit::run(&mut w, 1);
    assert!(w.discs.is_empty());
    w.obj_mut(f).unwrap().on_map = true;
    testkit::run(&mut w, 14);
    assert_eq!(w.obj(t).unwrap().health, h0);
    // The firer begins a crash: cancelled at the next due step.
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    testkit::run(&mut w, 1);
    let mut j = glasnost_sim::jump::JumpState::new();
    j.crash_started = true;
    w.obj_mut(f).unwrap().jump = Some(j);
    testkit::run(&mut w, 1);
    assert_eq!(w.discs.len(), 1, "an even update makes no step");
    testkit::run(&mut w, 1);
    assert!(w.discs.is_empty());
}

// spec: S-0615 claim 15 (arc step k: beams from a = i₀ + k to a + 1 and from b = i₀ − k to b − 1, lasting 8 − k and fading to half; the weapon's colours; the ring at the slot-0 fire point at the step) and claim 17 (the ring points)
#[test]
fn arc_beams() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    let i0 = i32::from(w.discs[0].start);
    let mode = w.rules().mode;
    for k in 0..8 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        let burst = w.obj(f).unwrap().combat.burst;
        let centre = glasnost_sim::combat::position::fire_point(&w, f, 0, burst);
        let at = |i: i32| {
            let (dx, dy) = disc::ring_point(i, mode);
            Pos::new(centre.x + dx, centre.y + dy, centre.z)
        };
        let a = (i0 + k).rem_euclid(16);
        let b = (i0 - k).rem_euclid(16);
        assert_eq!(
            beams(&w),
            [
                (at(a), at((a + 1) % 16), 8 - k, false),
                (at(b), at((b + 15) % 16), 8 - k, false)
            ],
            "k {k}"
        );
        for e in &w.effects {
            if let Effect::Beam { colours, house, .. } = e {
                assert_eq!(*colours, [[200, 100, 50], [100, 50, 25], [10, 10, 10]]);
                assert_eq!(*house, None);
            }
        }
        testkit::run(&mut w, 1);
    }
    assert_eq!(disc::arc_pairs(12, 0), [(12, 13), (12, 11)]);
    assert_eq!(disc::arc_pairs(12, 7), [(3, 4), (5, 4)]);
}

// spec: S-0615 claim 15 (the charge-up sound at k = 0 only; with the house-colour flag the firer's house laser colour, the outer colour halved toward zero, the spread kept)
#[test]
fn charge_up_and_house_colours() {
    let mut r = rules();
    r.sim.weapons.get_mut("DISC").unwrap().is_house_color = Some(true);
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert_eq!(
        discs(&w),
        [&DiscEvent::ChargeUp {
            sound: Some("HUM".into())
        }]
    );
    for e in &w.effects {
        if let Effect::Beam { house, .. } = e {
            assert_eq!(*house, w.obj(f).unwrap().owner);
        }
    }
    assert_eq!(
        disc::house_colours([201, 99, 0], [10, 10, 10]),
        [[201, 99, 0], [100, 49, 0], [10, 10, 10]]
    );
}

// spec: S-0615 claim 16 (the final step: a beam from ring point i₀ + 8 to the target point lasting LaserDuration (absent 10), fading to none; a detonation of D_s with the weapon's warhead at the target point, the firer the attacker; the report sound with it)
#[test]
fn final_step() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    testkit::run(&mut w, 16);
    let i0 = i32::from(w.discs[0].start);
    let mode = w.rules().mode;
    w.effects.clear();
    testkit::run(&mut w, 1);
    let burst = w.obj(f).unwrap().combat.burst;
    let centre = glasnost_sim::combat::position::fire_point(&w, f, 0, burst);
    let (dx, dy) = disc::ring_point(i0 + 8, mode);
    let tp = glasnost_sim::combat::range::target_point(&w, Target::Object(t));
    assert_eq!(
        beams(&w),
        [(
            Pos::new(centre.x + dx, centre.y + dy, centre.z),
            tp,
            10,
            true
        )]
    );
    let report = w.obj(f).unwrap().report_value;
    assert_eq!(discs(&w), [&DiscEvent::Hit { report }]);
    let det: Vec<(Pos, i32)> = w
        .effects
        .iter()
        .filter_map(|e| match e {
            Effect::Detonation { pos, damage, .. } => Some((*pos, *damage)),
            _ => None,
        })
        .collect();
    assert_eq!(det, [(tp, DISC_DAMAGE)]);
    // The firer is the attacker: the target's hit names it.
    assert!(
        w.effects
            .iter()
            .any(|e| matches!(e, Effect::Hit { attacker: Some(a), .. } if *a == f))
    );
}

// spec: S-0615 claim 16 and Edge cases (quirks.disc_laser_no_house on: the detonation names no attacking house; off: the firer's owner), and its import default of 10 for LaserDuration
#[test]
fn disc_laser_no_house_both_positions() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    let d = w.discs[0].clone();
    let on = disc::detonation(&w, &d);
    assert_eq!(on.house, None);
    assert_eq!(on.source, Some(f));
    assert!(on.resources);
    assert_eq!(on.damage, DISC_DAMAGE);
    let mut r2 = rules();
    r2.globals.quirks.disc_laser_no_house = false;
    let (mut w, f, t) = duel(&r2, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    let d = w.discs[0].clone();
    let off = disc::detonation(&w, &d);
    assert_eq!(off.house, w.obj(f).unwrap().owner);
    assert!(off.house.is_some());
    assert_eq!(w.rules().combat.weapon(d.weapon).laser_duration, 10);
}

// spec: S-0615 claim 17 and F6 (16 points on a 240-lepton circle, point 0 due north, clockwise, whole-degree angles ⌊22.5 i⌋; points 14 and 15 at 225° and 247°)
#[test]
fn ring_points() {
    let mode = glasnost_num::Rounding::TowardZero;
    let pts: Vec<(i32, i32)> = (0..16).map(|i| disc::ring_point(i, mode)).collect();
    assert_eq!(pts[0], (0, -240));
    assert_eq!(pts[4], (240, 0));
    assert_eq!(pts[8], (0, 240));
    assert_eq!(pts[12], (-240, 0));
    // 292°: (trunc(240 cos 292°), trunc(240 sin 292°)) = (89, −222).
    assert_eq!(pts[1], (89, -222));
    // 225° and 247°.
    assert_eq!(pts[14], (-169, -169));
    assert_eq!(pts[15], (-93, -220));
    assert_eq!(disc::ring_point(16, mode), pts[0]);
}

// spec: S-0615 claim 18 (a disc laser draws nothing from either random stream itself)
#[test]
fn disc_takes_no_draws() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Building, "BIG", 4);
    // A control world whose disc is removed right after the shot: the random stream must match
    // it through T + 16 (the firer's rearm is the same in both).
    shoot(&mut w, f, t);
    let mut control = w.clone();
    control.discs.clear();
    testkit::run(&mut w, 16);
    testkit::run(&mut control, 16);
    assert_eq!(w.rng, control.rng);
    assert_eq!(w.discs.len(), 1);
}

// spec: S-0615 claims 4 and 12 (walkers and disc lasers are game state: a save and load keeps them and the state hash)
#[test]
fn save_load_round_trip() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    walker_at(&mut w, c(-3, -3), 0);
    testkit::run(&mut w, 5);
    assert!(!w.discs.is_empty() && !w.walkers.is_empty());
    let bytes = save::save(&w);
    let back = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&back), save::state_hash(&w));
    assert_eq!(back.discs, w.discs);
    assert_eq!(back.walkers, w.walkers);
    let mut a = w.clone();
    let mut b = back;
    testkit::run(&mut a, 20);
    testkit::run(&mut b, 20);
    assert_eq!(save::state_hash(&a), save::state_hash(&b));
}

// spec: S-0615 Edge cases (the start point is fixed at the shot: a target that moves round the firer still gets the arcs closing on the old side, and the final beam runs to its current target point; the ring follows the firer's current fire point; a laser duration over 127 is kept as given)
#[test]
fn edge_start_fixed_ring_follows_long_duration() {
    let mut r = rules();
    r.sim.weapons.get_mut("DISC").unwrap().laser_duration = Some(200);
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    shoot(&mut w, f, t);
    let i0 = i32::from(w.discs[0].start);
    // The target goes round to the west of the firer; the firer shifts north.
    w.obj_mut(t).unwrap().pos = Pos::centre(c(-3, 0), 0);
    w.obj_mut(f).unwrap().pos.y -= 100;
    let mode = w.rules().mode;
    for k in 0..8 {
        w.effects.clear();
        testkit::run(&mut w, 1);
        let p = w.obj(f).unwrap().pos;
        let (dx, dy) = disc::ring_point((i0 + k).rem_euclid(16), mode);
        assert_eq!(
            beams(&w)[0].0,
            Pos::new(p.x + dx, p.y + dy, p.z + 70),
            "k {k}"
        );
        testkit::run(&mut w, 1);
    }
    w.effects.clear();
    testkit::run(&mut w, 1);
    let p = w.obj(f).unwrap().pos;
    let (dx, dy) = disc::ring_point(i0 + 8, mode);
    let tp = glasnost_sim::combat::range::target_point(&w, Target::Object(t));
    assert_eq!(
        beams(&w),
        [(Pos::new(p.x + dx, p.y + dy, p.z + 70), tp, 200, true)]
    );
}

/// A disc shot at an adjacent CART with the weapon range field `range`,
/// run for the whole schedule: whether the target took damage and the disc
/// events seen in its first update.
fn disc_with_range(r: &Ruleset, range: &str) -> (bool, Vec<DiscEvent>) {
    let mut r = r.clone();
    r.sim.weapons.get_mut("DISC").unwrap().range = Some(Decimal(range.into()));
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    let h0 = w.obj(t).unwrap().health;
    shoot(&mut w, f, t);
    w.effects.clear();
    testkit::run(&mut w, 1);
    let first = discs(&w).into_iter().cloned().collect();
    testkit::run(&mut w, 18);
    (w.obj(t).is_none_or(|o| o.health < h0), first)
}

// spec: S-0615 claim 14 and Edge cases (revision 3, Q-1426 answered: with quirks.disc_laser_unlimited_range on, the unlimited range −512 is compared as it is, so the disc laser is cancelled at its first step and never deals damage; off, unlimited range never cancels on distance)
#[test]
fn unlimited_range_both_positions() {
    let r = rules();
    let (hit, first) = disc_with_range(&r, "-2");
    assert!(!hit);
    assert_eq!(first, [DiscEvent::Cancelled]);
    let mut off = r.clone();
    off.globals.quirks.disc_laser_unlimited_range = false;
    let (hit, first) = disc_with_range(&off, "-2");
    assert!(
        hit,
        "never cancelled on distance: the final detonation hits"
    );
    assert!(!first.contains(&DiscEvent::Cancelled));
}

// spec: S-0615 claim 14 (Other negative ranges: every distance exceeds them, so the disc laser is cancelled at its first step with no damage; with quirks.negative_range_fields off the range is 0, and a distance above 0 still cancels)
#[test]
fn other_negative_ranges_cancel() {
    let r = rules();
    let (hit, first) = disc_with_range(&r, "-1.5");
    assert!(!hit);
    assert_eq!(first, [DiscEvent::Cancelled]);
    let mut off = r.clone();
    off.globals.quirks.negative_range_fields = false;
    let (hit, first) = disc_with_range(&off, "-1.5");
    assert!(!hit);
    assert_eq!(first, [DiscEvent::Cancelled]);
}

// spec: S-0615 claim 14 (revision 3, Q-1422 answered: When the loss applies: at the moment it happens, also between two steps, and for good; a target that leaves the map and comes back before the next step has still cancelled the disc laser)
#[test]
fn loss_between_steps_is_permanent() {
    let r = rules();
    let (mut w, f, t) = duel(&r, Kind::Vehicle, "CART", 3);
    let h0 = w.obj(t).unwrap().health;
    shoot(&mut w, f, t);
    // Updates 1 (step 0) and 2 (no step): the next step is in update 3.
    testkit::run(&mut w, 2);
    let pos = w.obj(t).unwrap().pos;
    let facing = w.body_facing(t);
    glasnost_sim::objects::take_inside(&mut w, t);
    assert!(w.discs[0].lost, "lost at the moment it leaves the map");
    glasnost_sim::objects::place(&mut w, t, pos, facing);
    assert!(w.obj(t).unwrap().on_map, "back before the next step");
    w.effects.clear();
    testkit::run(&mut w, 1);
    assert!(w.discs.is_empty());
    assert_eq!(discs(&w), [&DiscEvent::Cancelled]);
    testkit::run(&mut w, 16);
    assert_eq!(w.obj(t).unwrap().health, h0);
}

// spec: S-0615 claim 14 (a cell target is measured at its centre at the terrain height, a bridge deck not counted) and claim 16 (revision 3, Q-1420 and Q-1422 answered: the final beam and detonation of a cell target are at its centre at the terrain height, raised by 416 when the cell carries a bridge deck)
#[test]
fn cell_target_over_a_bridge_deck() {
    let r = rules();
    let mut w = game(&r);
    let f = testkit::spawn(&mut w, Kind::Vehicle, "SAUCER", 0, c(0, 0), 0);
    let cell = c(3, 0);
    set_deck(&mut w, cell, true);
    let ground = glasnost_sim::combat::range::target_point(&w, Target::Cell(cell));
    let flat = {
        let s = w.obj(f).unwrap().pos;
        let n = (i64::from(s.x - ground.x).pow(2)
            + i64::from(s.y - ground.y).pow(2)
            + i64::from(s.z - ground.z).pow(2)) as u64;
        glasnost_num::geom::root_floor(n, w.rules().quirks.table_root_distance, w.rules().mode)
            as i32
    };
    assert_eq!(
        disc::recheck_distance(&w, f, Target::Cell(cell)),
        flat,
        "no deck in the re-check"
    );
    let raised = disc::final_point(&w, Target::Cell(cell));
    assert_eq!(raised, Pos::new(ground.x, ground.y, ground.z + 416));
    set_deck(&mut w, cell, false);
    assert_eq!(disc::final_point(&w, Target::Cell(cell)), ground);
}

// ---------------------------------------------------------------------------
// S-0107 claim 17: row 3 (the type's own death animations) and the corpse
// pick (M8 step A1, docs/m8-claims/a1-arena-spread.md)

/// `rules()` plus the animations ROW0, ROW2, OWNBODY, BODYA and BODYB
/// (10 frames, no creation draws), the general corpse list BODYA, BODYB and
/// the infantry type DYER: WALKER with die-1, die-2 and wet-die-1 entries
/// of 2 frames.
fn corpse_rules() -> Ruleset {
    let mut r = rules();
    for n in ["ROW0", "ROW2", "OWNBODY", "BODYA", "BODYB"] {
        register(&mut r, "animations", n);
        r.sim.animations.insert(
            n.into(),
            AnimationType {
                frame_count: Some(10),
                ..Default::default()
            },
        );
    }
    r.sim.general.dead_bodies = Some(vec!["BODYA".into(), "BODYB".into()]);
    let mut dyer = r.sim.infantry["WALKER"].clone();
    for k in ["die_1", "die_2", "wet_die_1"] {
        dyer.sequences.insert(
            k.into(),
            glasnost_rules::sim::SequenceEntry {
                start: 0,
                count: 2,
                multiplier: 0,
                direction: None,
            },
        );
    }
    register(&mut r, "infantry", "DYER");
    r.sim.infantry.insert("DYER".into(), dyer);
    r
}

/// Kills a DYER with `wh`, lets its death sequence complete, and gives the
/// animations of the completion, the stream before it, the world and the
/// unit. `wet`: the sequence is wet-die-1 instead.
fn completed_death(
    r: &Ruleset,
    wh: &str,
    wet: bool,
) -> (Vec<String>, glasnost_sim::rng::Rng, World, ObjectId) {
    let mut w = game(r);
    let id = testkit::spawn(&mut w, Kind::Infantry, "DYER", 0, c(0, 0), 0);
    kill(&mut w, id, wh, None);
    let st = w.obj_mut(id).unwrap().infantry.as_mut().unwrap();
    assert!(
        matches!(st.seq, Some(Seq::Die1 | Seq::Die2)),
        "the death sequence started"
    );
    if wet {
        st.seq = Some(Seq::WetDie1);
    }
    w.tick += 100;
    w.effects.clear();
    let before = w.rng.clone();
    glasnost_sim::combat::death::infantry_death_turn(&mut w, id);
    (made(&w).0, before, w, id)
}

/// The entry of BODYA, BODYB that one raw value of `x` picks.
fn general_pick(x: &mut glasnost_sim::rng::Rng) -> &'static str {
    if x.raw().is_multiple_of(2) {
        "BODYA"
    } else {
        "BODYB"
    }
}

// spec: S-0107 claim 17 (die-1 and die-2 complete: a corpse from the general list when the type's own list is empty, picked by one draw read as an unsigned 32-bit number modulo the list length; the unit is removed on the same tick) and Import mapping ([General] DeadBodies)
#[test]
fn s0107_claim_17_corpse_from_the_general_list_by_one_draw() {
    let r = corpse_rules();
    for wh in ["DK1", "DK2"] {
        let (anims, before, w, id) = completed_death(&r, wh, false);
        let mut x = before.clone();
        let pick = general_pick(&mut x);
        assert_eq!(anims, [pick], "{wh}");
        assert_eq!(w.rng, x, "{wh}: exactly the pick's raw value");
        assert!(w.obj(id).is_none(), "{wh}");
    }
}

// spec: S-0107 claim 17 (the type's own corpse list when it is not empty, the pick still one draw; a type marked "not human" with no list of its own leaves no corpse and takes no draw; wet-die-1 completes with no corpse) and Import mapping (DeadBodies of the type)
#[test]
fn s0107_claim_17_own_corpse_list_not_human_and_wet() {
    let mut r = corpse_rules();
    r.sim.infantry.get_mut("DYER").unwrap().dead_bodies = Some(vec!["OWNBODY".into()]);
    let (anims, before, w, _) = completed_death(&r, "DK1", false);
    let mut x = before.clone();
    let _ = x.raw();
    assert_eq!(anims, ["OWNBODY"]);
    assert_eq!(w.rng, x);
    let (anims, before, w, id) = completed_death(&r, "DK1", true);
    assert!(anims.is_empty(), "wet-die-1: no corpse");
    assert_eq!(w.rng, before);
    assert!(w.obj(id).is_none());
    let mut r = corpse_rules();
    r.sim.infantry.get_mut("DYER").unwrap().not_human = Some(true);
    let (anims, before, w, id) = completed_death(&r, "DK1", false);
    assert!(anims.is_empty(), "not human: no corpse");
    assert_eq!(w.rng, before);
    assert!(w.obj(id).is_none());
}

// spec: S-0107 claim 17 (quirks.refused_death_keeps_unit off: a die-1 request refused because its entry has C = 0 is treated as completed on the tick of its death, with its corpse)
#[test]
fn s0107_claim_17_refused_death_with_the_quirk_off_leaves_a_corpse() {
    let mut r = corpse_rules();
    r.globals.quirks.refused_death_keeps_unit = false;
    let mut w = game(&r);
    // WALKER has no death sequences: the die-1 request is refused by its C = 0.
    let id = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, c(0, 0), 0);
    w.effects.clear();
    let mut x = w.rng.clone();
    kill(&mut w, id, "DK1", None);
    let pick = general_pick(&mut x);
    assert_eq!(made(&w).0, [pick]);
    assert_eq!(w.rng, x);
    assert!(w.obj(id).is_none());
}

// spec: S-0107 claim 17 row 3 (revision 22, Q-1715 answered: a type with its own list of death animations with at least one entry shows entry w, entry 0 when w is negative or not less than the list's length, with no sequence, and is removed at once; it takes precedence over rows 4 to 8, so no die-1 request, no conversion and no game-wide death animation) and Import mapping (DeathAnims: no-type items are left out, so A, none, B imports as A, B and death kind 1 shows B)
#[test]
fn s0107_claim_17_row_3_own_death_animations() {
    let mut r = corpse_rules();
    // The imported shape of `ROW0, none, ROW2`: two entries.
    r.sim.infantry.get_mut("DYER").unwrap().death_anims = Some(vec!["ROW0".into(), "ROW2".into()]);
    for (wh, expect) in [
        ("DK1", "ROW2"),
        ("DK2", "ROW0"),
        ("DK7", "ROW0"),
        ("DK9", "ROW0"),
        ("DK-1", "ROW0"),
    ] {
        let mut w = game(&r);
        let id = testkit::spawn(&mut w, Kind::Infantry, "DYER", 0, c(0, 0), 0);
        w.effects.clear();
        let before = w.rng.clone();
        kill(&mut w, id, wh, None);
        assert_eq!(made(&w).0, [expect], "{wh}");
        assert!(w.obj(id).is_none(), "{wh}: removed at once");
        assert_eq!(w.rng, before, "{wh}: no corpse pick, no dispatch draw");
    }
    // A "not human" type with its own list: entry 0 for kind 8, no die-1 request.
    r.sim.infantry.get_mut("DYER").unwrap().not_human = Some(true);
    let mut w = game(&r);
    let id = testkit::spawn(&mut w, Kind::Infantry, "DYER", 0, c(0, 0), 0);
    w.effects.clear();
    kill(&mut w, id, "DK8", None);
    assert_eq!(made(&w).0, ["ROW0"]);
    assert!(w.obj(id).is_none());
}

// spec: S-0107 claim 17 (revision 22: a death-animation list whose every item was left out has no entries, so the type has no list for row 3 and the later rows decide exactly as for a type that never had one)
#[test]
fn s0107_claim_17_a_list_without_entries_is_no_row_3_list() {
    let outcome = |list: Option<Vec<String>>, wh: &str| {
        let mut r = corpse_rules();
        r.sim.infantry.get_mut("DYER").unwrap().death_anims = list;
        let mut w = game(&r);
        let id = testkit::spawn(&mut w, Kind::Infantry, "DYER", 0, c(0, 0), 0);
        w.effects.clear();
        kill(&mut w, id, wh, None);
        testkit::run(&mut w, 40);
        (made(&w).0, w.rng.clone(), w.obj(id).is_some())
    };
    for wh in ["DK1", "DK2", "DK8"] {
        assert_eq!(outcome(Some(Vec::new()), wh), outcome(None, wh), "{wh}");
    }
}
