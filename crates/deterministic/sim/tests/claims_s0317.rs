//! Claim tests for debris pieces with voxel models (S-0317) on synthetic
//! data: invented piece types, animations, a smoke system and a resource,
//! every name and number our own. The drawing side is `glasnost-app`'s
//! `mapview::piece_tests`; the ledger is `docs/m8-claims/c5-S-0317.md`.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    Decimal, LandRow, LandType, OverlayType, ParticleSystemType, ParticleType, VoxelAnimationType,
    WarheadType,
};
use glasnost_rules::sim_econ::TiberiumType;
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::pieces::{self, Piece};
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{save, testkit};

const N: i32 = 24;

fn dec(s: &str) -> Option<Decimal> {
    Some(Decimal(s.into()))
}

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn add(r: &mut Ruleset, name: &str, t: VoxelAnimationType) {
    register(r, "voxel_animations", name);
    r.sim.voxel_animations.insert(name.into(), t);
}

/// The rules: testkit's combat rules plus the piece types ROCK (a falling
/// piece with a bounce, expiry and trailer animation, damage 40 in a radius
/// of 200 with the warhead HE), BARE (no fields: every default), STAR (a
/// meteor piece that spawns two-at-most ROCKs per draw), SMOKY (a falling
/// piece with the smoke system PUFFS and the start sound Hum) and ORE1 (a
/// falling piece with the resource flag).
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    add(
        &mut r,
        "ROCK",
        VoxelAnimationType {
            duration: Some(40),
            damage: Some(40),
            damage_radius: Some(200),
            warhead: Some("HE".into()),
            bounce_anim: Some("Bump".into()),
            expire_anim: Some("Boom".into()),
            trailer_anim: Some("Trail".into()),
            elasticity: dec("0.5"),
            ..Default::default()
        },
    );
    add(&mut r, "BARE", VoxelAnimationType::default());
    add(
        &mut r,
        "STAR",
        VoxelAnimationType {
            is_meteor: Some(true),
            duration: Some(30),
            min_z_vel: dec("-10"),
            max_xy_vel: dec("6"),
            spawns: Some("ROCK".into()),
            spawn_count: Some(2),
            ..Default::default()
        },
    );
    register(&mut r, "particles", "SMOKEP");
    r.sim.particles.insert(
        "SMOKEP".into(),
        ParticleType {
            behaves_like: Some("smoke".into()),
            max_ec: Some(1000),
            end_state_ai: Some(100),
            state_ai_advance: Some(4),
            ..Default::default()
        },
    );
    register(&mut r, "particle_systems", "PUFFS");
    r.sim.particle_systems.insert(
        "PUFFS".into(),
        ParticleSystemType {
            behaves_like: Some("smoke".into()),
            holds_what: Some("SMOKEP".into()),
            spawn_frames: Some(1000),
            particle_cap: Some(4),
            ..Default::default()
        },
    );
    add(
        &mut r,
        "SMOKY",
        VoxelAnimationType {
            attached_system: Some("PUFFS".into()),
            start_sound: Some("Hum".into()),
            ..Default::default()
        },
    );
    add(
        &mut r,
        "METSMOKE",
        VoxelAnimationType {
            is_meteor: Some(true),
            min_z_vel: dec("-10"),
            attached_system: Some("PUFFS".into()),
            ..Default::default()
        },
    );
    add(
        &mut r,
        "ORE1",
        VoxelAnimationType {
            is_tiberium: Some(true),
            ..Default::default()
        },
    );
    add(
        &mut r,
        "OREM",
        VoxelAnimationType {
            is_tiberium: Some(true),
            is_meteor: Some(true),
            ..Default::default()
        },
    );
    // CLIFF: a falling piece whose elasticity 30% is the double product 0.3,
    // a double that single precision does not hold, so that ε and e scale
    // apart (F8, F9; S-0003 claim 6).
    add(
        &mut r,
        "CLIFF",
        VoxelAnimationType {
            elasticity: dec("30%"),
            ..Default::default()
        },
    );
    r.sim.general.wake = Some("Wake".into());
    r.sim.general.splash_list = Some(vec!["Spl1".into(), "Spl2".into(), "Spl3".into()]);
    r
}

/// [`rules`] with 128 resource overlays and one tiberium, so that the
/// overlay type at position 127 is a resource overlay (claim 12).
fn resource_rules() -> Ruleset {
    let mut r = rules();
    r.sim.land_types.insert(
        LandType::Clear,
        LandRow {
            buildable: Some(true),
            ..Default::default()
        },
    );
    r.sim
        .land_types
        .insert(LandType::Tiberium, LandRow::default());
    for i in 0..128 {
        let n = format!("OV{i:03}");
        register(&mut r, "overlays", &n);
        r.sim.overlays.insert(
            n,
            OverlayType {
                tiberium: Some(true),
                ..Default::default()
            },
        );
    }
    register(&mut r, "tiberiums", "ORE");
    r.sim.tiberiums.insert(
        "ORE".into(),
        TiberiumType {
            value: Some(25),
            image_run: Some([0, 12, 8]),
            ..Default::default()
        },
    );
    r
}

fn world(r: &Ruleset) -> World {
    testkit::game_with(r, N, 7)
}

fn ti(w: &World, name: &str) -> u32 {
    w.rules().pieces.index(name).expect("piece type")
}

/// Makes a piece of the type `name` at `at` with no house.
fn make(w: &mut World, name: &str, at: Pos) -> ObjectId {
    let t = ti(w, name);
    pieces::create(w, t, at, None).expect("piece made")
}

fn mid(dx: i32, dy: i32) -> CellXY {
    testkit::mid(N, dx, dy)
}

fn centre(c: CellXY, z: i32) -> Pos {
    Pos::centre(c, z)
}

fn mode(w: &World) -> Rounding {
    w.rules().mode
}

fn d(x: i64, m: Rounding) -> F64 {
    F64::from_i64(x, m)
}

fn s(text: &str) -> F64 {
    glasnost_num::decimal::nearest_double(text)
}

/// 1.4 in single precision, widened (claim 5).
fn g() -> F64 {
    s("1.4").to_f32(Rounding::Nearest).to_f64()
}

fn piece(w: &World, id: ObjectId) -> Piece {
    w.pieces.get(&id).cloned().expect("piece in play")
}

/// Puts a piece's flight state.
fn set(w: &mut World, id: ObjectId, flight: [i32; 3], v: [&str; 3]) {
    let m = mode(w);
    let p = w.pieces.get_mut(&id).unwrap();
    p.flight = flight.map(|c| d(i64::from(c), m).to_f32(m));
    p.velocity = v.map(|x| s(x).to_f32(m));
    p.pos = Pos::new(flight[0], flight[1], flight[2]);
}

fn anims(w: &World) -> Vec<(String, Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim { name, pos, .. } => Some((name.clone(), *pos)),
            _ => None,
        })
        .collect()
}

fn health(w: &World, id: ObjectId) -> i32 {
    w.obj(id).map_or(0, |o| o.health)
}

fn set_land(w: &mut World, c: CellXY, l: u8) {
    w.cells.grid.get_mut(c).expect("cell").land = l;
}

fn mag_rem(r: u32, s: i32) -> i64 {
    i64::from(glasnost_num::int::rem((r as i32).wrapping_abs(), s))
}

fn set_quirks(w: &mut World, q: glasnost_rules::quirks::Quirks) {
    let mut rules = (*w.rules_arc()).clone();
    rules.quirks = q;
    w.rules = Some(std::sync::Arc::new(rules));
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let back = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&back), save::state_hash(w));
    assert_eq!(back.pieces, w.pieces);
}

/// spec: S-0317 claim 3 and F1 (a falling piece: four raw values for the spin, vertical, y and x speeds, then three ranged draws R(−65535, 65535) for the axis (a₃, a₂, a₁); the velocity of F1 in single precision; the start position 10 leptons above the creation point; the position the creation point) and claim 2 (the counter starts at the duration); S-0202 claim 10 (revision 36: the bouncing row's group of three raw values and three ranged draws is for sprite animations only; a voxel-model debris piece takes S-0317's draws, not that group)
#[test]
fn claim_3_falling_piece_creation() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let mut probe: Rng = w.rng.clone();
    let at = centre(mid(0, 0), 0);
    let id = make(&mut w, "BARE", at);
    let half = w.rules().quirks.ranged_draw_half_span;
    let (r1, r2, r3, r4) = (probe.raw(), probe.raw(), probe.raw(), probe.raw());
    let a1 = probe.ranged(-65535, 65535, half);
    let a2 = probe.ranged(-65535, 65535, half);
    let a3 = probe.ranged(-65535, 65535, half);
    assert_eq!(
        w.rng, probe,
        "exactly four raw values and three ranged draws"
    );
    let p = piece(&w, id);
    // The defaults: M 15 (S_xy 30), Z 3.5 to 5.0 (S_z 2), A 0 to 10 degrees (S_a 1).
    let mm = s("15.0");
    let vx = d(mag_rem(r4, 30), m).sub(mm, m).to_f32(m);
    let vy = d(mag_rem(r3, 30), m).sub(mm, m).to_f32(m);
    let vz = d(mag_rem(r2, 2), m).add(s("3.5"), m).to_f32(m);
    assert_eq!(p.velocity, [vx, vy, vz]);
    assert_eq!(p.axis, [a3, a2, a1]);
    let amin = d(0, m);
    assert_eq!(p.spin, d(mag_rem(r1, 1), m).add(amin, m));
    assert_eq!(p.pos, at);
    assert_eq!(p.flight_point(), Pos::new(at.x, at.y, at.z + 10));
    assert_eq!(p.counter, 30, "the duration default 30");
    assert!(w.update.contains(id), "in the update order");
    round_trip(&w);
}

/// spec: S-0317 claim 4 and F2 (a meteor piece: raw values for the y speed, the x speed, the lifetime cut and the spin, then the three axis draws; v_z = Z_min; a sign change when v_x < −v_y; the counter cut by |q₃ rem 20|; the start trunc(c − T·v), which is its position)
#[test]
fn claim_4_meteor_piece_creation() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let half = w.rules().quirks.ranged_draw_half_span;
    let at = centre(mid(0, 0), 0);
    for _ in 0..12 {
        let mut probe: Rng = w.rng.clone();
        let id = make(&mut w, "STAR", at);
        let (q1, q2, q3, _q4) = (probe.raw(), probe.raw(), probe.raw(), probe.raw());
        for _ in 0..3 {
            probe.ranged(-65535, 65535, half);
        }
        assert_eq!(w.rng, probe, "four raw values and three ranged draws");
        let p = piece(&w, id);
        let mm = s("6");
        let mut vy = d(mag_rem(q1, 12), m).sub(mm, m);
        let mut vx = d(mag_rem(q2, 12), m).sub(mm, m);
        if vx.lt(vy.negated()) {
            vx = vx.negated();
            vy = vy.negated();
        }
        let vz = s("-10");
        assert_eq!(p.velocity, [vx.to_f32(m), vy.to_f32(m), vz.to_f32(m)]);
        assert!(
            vx.add(vy, m).ge(d(0, m)),
            "the sum of the horizontal speeds is never negative"
        );
        let cut = glasnost_num::int::rem(q3 as i32, 20).abs();
        assert_eq!(p.counter, 30 - cut);
        let t = d(i64::from(30 - cut), m);
        let back = |c: i32, v: F64| d(i64::from(c), m).sub(t.mul(v, m), m).to_i32_common();
        let start = Pos::new(back(at.x, vx), back(at.y, vy), back(at.z, vz));
        assert_eq!(p.pos, start);
        assert_eq!(p.flight_point(), start);
    }
}

/// spec: S-0317 claim 5 and F3 (gravity 1.4 off the vertical speed, then the move by the whole velocity, each stored in single precision; a meteor piece gets g back after the move) and claim 11 (after a flying turn the position is the flight point)
#[test]
fn claim_5_flight_step() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let at = centre(mid(0, 0), 2000);
    let id = make(&mut w, "BARE", at);
    let before = piece(&w, id);
    pieces::turn(&mut w, id);
    let p = piece(&w, id);
    let vz = before.velocity[2].to_f64().sub(g(), m).to_f32(m);
    assert_eq!(p.velocity, [before.velocity[0], before.velocity[1], vz]);
    let want: [F32; 3] = [0, 1, 2].map(|i| {
        before.flight[i]
            .to_f64()
            .add(p.velocity[i].to_f64(), m)
            .to_f32(m)
    });
    assert_eq!(p.flight, want);
    assert_eq!(p.pos, p.flight_point());
    assert_eq!(p.counter, 29);
    // A meteor piece: the vertical speed gets g added back.
    let id = make(&mut w, "STAR", centre(mid(0, 0), 5000));
    let before = piece(&w, id);
    pieces::turn(&mut w, id);
    let p = piece(&w, id);
    let down = before.velocity[2].to_f64().sub(g(), m).to_f32(m);
    assert_eq!(p.velocity[2], down.to_f64().add(g(), m).to_f32(m));
    assert_eq!(
        p.flight[2],
        before.flight[2].to_f64().add(down.to_f64(), m).to_f32(m),
        "the move uses the speed with g subtracted"
    );
}

/// spec: S-0317 claim 6 (a counter T ≥ 1: T − 1 flying turns, the end in the T-th turn, then out of play; a counter of 0 or less at creation ends in the first turn without flight; −2147483648 wraps to 2147483647 and flies)
#[test]
fn claim_6_lifetime_in_turns() {
    let mut r = rules();
    r.sim.voxel_animations.get_mut("BARE").unwrap().duration = Some(4);
    let mut w = world(&r);
    let at = centre(mid(0, 0), 100_000);
    let id = make(&mut w, "BARE", at);
    for k in 1..4 {
        pieces::turn(&mut w, id);
        assert!(w.pieces.contains_key(&id), "turn {k} flies");
    }
    pieces::turn(&mut w, id);
    assert!(!w.pieces.contains_key(&id), "ends in the fourth turn");
    assert!(!w.update.contains(id), "and leaves play");
    for (dur, flies) in [(0, false), (-5, false), (i32::MIN, true)] {
        let mut r = rules();
        r.sim.voxel_animations.get_mut("BARE").unwrap().duration = Some(dur);
        let mut w = world(&r);
        let id = make(&mut w, "BARE", at);
        pieces::turn(&mut w, id);
        assert_eq!(w.pieces.contains_key(&id), flies, "duration {dur}");
        if flies {
            assert_eq!(piece(&w, id).counter, i32::MAX);
        }
    }
}

/// spec: S-0317 claim 6 (a piece takes its turns in the update order: the tick pipeline turns it)
#[test]
fn claim_6_turns_in_the_tick() {
    let r = rules();
    let mut w = world(&r);
    let id = make(&mut w, "BARE", centre(mid(0, 0), 50_000));
    testkit::run(&mut w, 3);
    assert_eq!(piece(&w, id).counter, 27);
}

/// spec: S-0317 claim 6 and F7 (on an even tick, before the move, the trailer at (trunc x, trunc y, conv(6.967 z + 0.5)) of the flight position; none on an odd tick) and Edge cases (quirks.voxel_debris_trailer_height off: at the flight point)
#[test]
fn claim_6_f7_trailer() {
    for quirk in [true, false] {
        let r = rules();
        let mut w = world(&r);
        let mut q = w.rules().quirks;
        q.voxel_debris_trailer_height = quirk;
        set_quirks(&mut w, q);
        let m = mode(&w);
        let at = centre(mid(0, 0), 300);
        let id = make(&mut w, "ROCK", at);
        w.tick = 10;
        w.effects.clear();
        let fp = piece(&w, id).flight;
        pieces::turn(&mut w, id);
        let z = if quirk {
            s("6.967")
                .mul(fp[2].to_f64(), m)
                .add(s("0.5"), m)
                .to_i32_common()
        } else {
            fp[2].to_f64().to_i32_common()
        };
        assert_eq!(
            anims(&w),
            vec![("Trail".to_owned(), Pos::new(at.x, at.y, z))]
        );
        assert_eq!(z, if quirk { 2160 } else { 310 });
        w.tick = 11;
        w.effects.clear();
        pieces::turn(&mut w, id);
        assert!(anims(&w).is_empty(), "no trailer on an odd tick");
    }
}

/// spec: S-0317 claims 7 and 10 (a ground contact less than 100 leptons below the ground: set to the ground, the velocity rebounds as e(v_x, v_y, −v_z); the bounce animation at the position before the move, then a hit on each object of the new position's cell within the radius; none in a neighbouring cell)
#[test]
fn claim_7_ground_impact() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let c = mid(0, 0);
    let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
    let next = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, mid(1, 0), 0);
    let (h0, h1) = (health(&w, near), health(&w, next));
    let at = centre(c, 0);
    let id = make(&mut w, "ROCK", at);
    set(&mut w, id, [at.x + 120, at.y, 5], ["0", "0", "-10"]);
    w.tick = 1;
    w.effects.clear();
    pieces::turn(&mut w, id);
    let p = piece(&w, id);
    assert_eq!(p.flight[2], d(0, m).to_f32(m), "set to the ground");
    let vz = s("-10").to_f32(m).to_f64().sub(g(), m).to_f32(m);
    let e = s("0.5").to_f32(m).to_f64();
    assert_eq!(p.velocity[2], vz.to_f64().negated().mul(e, m).to_f32(m));
    assert_eq!(
        anims(&w),
        vec![("Bump".to_owned(), Pos::new(at.x + 120, at.y, 5))]
    );
    assert!(
        health(&w, near) < h0,
        "the object of the impact cell is hit"
    );
    assert_eq!(health(&w, next), h1, "the neighbouring cell is not");
    // Out of the radius in the same cell: no hit.
    let far = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, mid(0, 3), 0);
    let hf = health(&w, far);
    let at = centre(mid(0, 3), 0);
    set(&mut w, id, [at.x - 127, at.y - 100, 5], ["0", "0", "-10"]);
    pieces::turn(&mut w, id);
    assert_eq!(
        health(&w, far),
        hf,
        "a sum of 227 is beyond the radius of 200"
    );
}

/// spec: S-0317 claim 8 (a contact whose position before the move is in a water cell: no animation and no hit; the counter 0) and claim 11 (a falling piece's end on water: the wake at the position, the first splash 10 leptons higher)
#[test]
fn claim_8_11_water() {
    let r = rules();
    let mut w = world(&r);
    let c = mid(0, 0);
    set_land(&mut w, c, land::WATER);
    let at = centre(c, 0);
    let id = make(&mut w, "ROCK", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    w.tick = 1;
    w.effects.clear();
    pieces::turn(&mut w, id);
    assert!(anims(&w).is_empty());
    assert_eq!(piece(&w, id).counter, 0);
    let pos = piece(&w, id).pos;
    pieces::turn(&mut w, id);
    assert!(!w.pieces.contains_key(&id));
    assert_eq!(
        anims(&w),
        vec![
            ("Wake".to_owned(), pos),
            ("Spl1".to_owned(), Pos::new(pos.x, pos.y, pos.z + 10))
        ]
    );
}

/// spec: S-0317 claim 11 (a meteor piece's end on water: the last splash-list entry 5 leptons higher; at deck height on water: the land row)
#[test]
fn claim_11_meteor_on_water() {
    let r = rules();
    let mut w = world(&r);
    let c = mid(0, 0);
    set_land(&mut w, c, land::WATER);
    let id = make(&mut w, "STAR", centre(c, 0));
    set(
        &mut w,
        id,
        [centre(c, 0).x, centre(c, 0).y, 3],
        ["0", "0", "0"],
    );
    w.pieces.get_mut(&id).unwrap().counter = 1;
    w.effects.clear();
    let mut probe = w.rng.clone();
    pieces::turn(&mut w, id);
    assert_eq!(
        anims(&w),
        vec![(
            "Spl3".to_owned(),
            Pos::new(c.x * 256 + 128, c.y * 256 + 128, 8)
        )]
    );
    assert_eq!(w.rng.raw(), probe.raw(), "no spawn draws on water");
    // At deck height (416 above the ground) over water: the land row, with its spawns.
    let id = make(&mut w, "STAR", centre(c, 0));
    set(
        &mut w,
        id,
        [centre(c, 0).x, centre(c, 0).y, 416],
        ["0", "0", "0"],
    );
    w.pieces.get_mut(&id).unwrap().counter = 1;
    let before = w.pieces.len();
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let n = probe.ranged(0, 2, half) + probe.ranged(0, 2, half);
    w.effects.clear();
    pieces::turn(&mut w, id);
    assert!(anims(&w).is_empty(), "no splash at deck height");
    assert_eq!(
        w.pieces.len(),
        before - 1 + n as usize,
        "the spawns of the land row"
    );
}

/// spec: S-0317 claim 9 and F6 (a contact that leaves S ≤ 2 is a stop: the counter 0, the rebound made, no bounce animation and no hit)
#[test]
fn claim_9_stop() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let c = mid(0, 0);
    let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
    let h0 = health(&w, near);
    let at = centre(c, 0);
    let id = make(&mut w, "ROCK", at);
    set(&mut w, id, [at.x, at.y, 0], ["0", "0", "0"]);
    w.tick = 1;
    w.effects.clear();
    pieces::turn(&mut w, id);
    let p = piece(&w, id);
    assert_eq!(p.counter, 0);
    let e = s("0.5").to_f32(m).to_f64();
    assert_eq!(
        p.velocity[2],
        g().to_f32(m).to_f64().mul(e, m).to_f32(m),
        "the rebound is still made"
    );
    assert!(anims(&w).is_empty());
    assert_eq!(health(&w, near), h0);
    // A piece high above the ground with no speed is not stopped: h·g counts as motion.
    set(&mut w, id, [at.x, at.y, 400], ["0", "0", "0"]);
    w.pieces.get_mut(&id).unwrap().counter = 5;
    pieces::turn(&mut w, id);
    assert_eq!(piece(&w, id).counter, 4);
}

/// spec: S-0317 claim 10 (a move from at or above the deck height to below it puts the piece on the deck; one from below to at or above it puts it 20 below the deck; the hits take the ground layer) and Edge cases (quirks.voxel_debris_deck_impact_layer)
#[test]
fn claim_10_high_bridge_deck() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let c = mid(0, 0);
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    let at = centre(c, 0);
    let id = make(&mut w, "BARE", at);
    set(&mut w, id, [at.x, at.y, 420], ["0", "0", "-10"]);
    pieces::turn(&mut w, id);
    assert_eq!(piece(&w, id).flight[2], d(416, m).to_f32(m), "on the deck");
    set(&mut w, id, [at.x, at.y, 410], ["0", "0", "20"]);
    pieces::turn(&mut w, id);
    assert_eq!(
        piece(&w, id).flight[2],
        d(396, m).to_f32(m),
        "under the deck"
    );
}

/// spec: S-0317 claim 10 (a new height 100 leptons or more below the ground is left where it is; a height from the ground up to 150 above it in a cell that holds a building is a contact at the ground height)
#[test]
fn claim_10_deep_move_and_building_cell() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let c = mid(0, 0);
    let at = centre(c, 0);
    let id = make(&mut w, "BARE", at);
    set(&mut w, id, [at.x, at.y, -50], ["0", "0", "-60"]);
    pieces::turn(&mut w, id);
    let z = piece(&w, id).flight[2];
    assert!(z.to_f64().lt(d(-100, m)), "left below the ground");
    let b = mid(3, 3);
    testkit::spawn(&mut w, Kind::Building, "TOWER", 1, b, 0);
    let at = centre(b, 0);
    set(&mut w, id, [at.x, at.y, 120], ["0", "0", "0"]);
    pieces::turn(&mut w, id);
    assert_eq!(
        piece(&w, id).flight[2],
        d(0, m).to_f32(m),
        "a building contact"
    );
}

/// spec: S-0317 claim 10 (the cliff test: a contact in a cell at least 2 height levels above the old cell that would put a descending piece higher than it was is undone: the flight position and velocity of the start of the turn, the hits at the position before the move)
#[test]
fn claim_10_cliff_undone() {
    let r = rules();
    let mut w = world(&r);
    let low = mid(0, 0);
    let high = mid(1, 0);
    w.cells.grid.get_mut(high).unwrap().level = 2;
    let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, low, 0);
    let h0 = health(&w, near);
    let at = centre(low, 0);
    let id = make(&mut w, "ROCK", at);
    // Close to the east edge of the low cell, descending, moving east into the high cell.
    set(&mut w, id, [at.x + 120, at.y, 150], ["30", "0", "-5"]);
    let before = piece(&w, id);
    w.tick = 1;
    w.effects.clear();
    pieces::turn(&mut w, id);
    let p = piece(&w, id);
    assert_eq!(p.flight, before.flight, "the move is undone");
    assert_eq!(p.pos.cell(), low);
    assert!(
        p.velocity[0].to_f64().lt(d(0, w.rules().mode)),
        "turned off the east face (claim 18, F9)"
    );
    assert!(
        health(&w, near) < h0,
        "the hits at the position before the move"
    );
}

/// spec: S-0317 claim 11 (not on water: the expiry animation at the flight point, then area damage there with the type's damage and warhead) and Edge cases (quirks.voxel_debris_expiry_needs_animation: no expiry animation, no area damage; with the switch off a nonzero damage deals it)
#[test]
fn claim_11_expiry() {
    let r = rules();
    let mut w = world(&r);
    let c = mid(0, 0);
    let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
    let h0 = health(&w, near);
    let at = centre(c, 0);
    let id = make(&mut w, "ROCK", at);
    w.pieces.get_mut(&id).unwrap().counter = 1;
    w.tick = 1;
    w.effects.clear();
    pieces::turn(&mut w, id);
    assert!(!w.pieces.contains_key(&id));
    // A falling piece that ends in its first turn: the expiry animation at its start position.
    assert_eq!(anims(&w)[0], ("Boom".to_owned(), Pos::new(at.x, at.y, 10)));
    assert!(health(&w, near) < h0, "the area damage");
    // No expiry animation: no area damage with the switch on.
    for quirk in [true, false] {
        let mut r = rules();
        let t = r.sim.voxel_animations.get_mut("ROCK").unwrap();
        t.expire_anim = None;
        let mut w = world(&r);
        let mut q = w.rules().quirks;
        q.voxel_debris_expiry_needs_animation = quirk;
        set_quirks(&mut w, q);
        let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
        let h0 = health(&w, near);
        let id = make(&mut w, "ROCK", at);
        w.pieces.get_mut(&id).unwrap().counter = 1;
        pieces::turn(&mut w, id);
        assert_eq!(health(&w, near) < h0, !quirk, "switch {quirk}");
    }
}

/// spec: S-0303 claim 23 (revision 25: the end of a debris piece is a producer of the area damage of claims 1 to 20 and of none of the projectile-only effects of claims 21 and 22: the warhead's particle system appears, and no explosion animation of the warhead's list and no warhead debris; also at a damage amount of 0, when the area damage has no effect, claim 2: no hit and no particle system) and S-0317 claim 11
#[test]
fn s0303_claim_23_a_pieces_end_is_area_damage_only() {
    for damage in [40, 0] {
        let mut r = rules();
        r.sim.warheads.insert(
            "SHOWY".into(),
            WarheadType {
                verses: Some(vec![dec("1").unwrap(); 11]),
                cell_spread: dec("0.75"),
                anim_list: Some(vec!["Kaboom".into()]),
                max_debris: Some(3),
                min_debris: Some(3),
                debris_types: Some(vec!["BARE".into()]),
                debris_maximums: Some(vec![3]),
                particle: Some("PUFFS".into()),
                combat_light_size: dec("0.5"),
                ..Default::default()
            },
        );
        let t = r.sim.voxel_animations.get_mut("ROCK").unwrap();
        t.warhead = Some("SHOWY".into());
        t.damage = Some(damage);
        let mut w = world(&r);
        let c = mid(0, 0);
        let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
        let h0 = health(&w, near);
        let id = make(&mut w, "ROCK", centre(c, 0));
        w.pieces.get_mut(&id).unwrap().counter = 1;
        w.effects.clear();
        pieces::turn(&mut w, id);
        assert!(
            w.pieces.is_empty(),
            "no warhead debris (claim 22), damage {damage}"
        );
        let names: Vec<String> = anims(&w).into_iter().map(|a| a.0).collect();
        assert_eq!(
            names,
            ["Boom"],
            "only the expiry animation, damage {damage}"
        );
        let puffs = w.rules().gas.system("PUFFS").unwrap();
        let systems = w
            .gas
            .systems
            .values()
            .filter(|s| s.type_index == puffs)
            .count();
        if damage > 0 {
            assert!(health(&w, near) < h0, "the area damage");
            assert_eq!(systems, 1, "the warhead's particle system (claim 20)");
        } else {
            assert_eq!(health(&w, near), h0, "no hit (claim 2)");
            assert_eq!(systems, 0, "no particle system (claim 2)");
        }
    }
}

/// spec: S-0317 claim 12 (a meteor piece's end: n = R(0, c) + R(0, c) further pieces of the spawn type at its flight point, with no house)
#[test]
fn claim_12_spawns() {
    let r = rules();
    let mut w = world(&r);
    let at = centre(mid(0, 0), 0);
    let id = make(&mut w, "STAR", at);
    set(&mut w, id, [at.x, at.y, 200], ["0", "0", "0"]);
    w.pieces.get_mut(&id).unwrap().counter = 1;
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let n = probe.ranged(0, 2, half) + probe.ranged(0, 2, half);
    pieces::turn(&mut w, id);
    let rock = ti(&w, "ROCK");
    let made: Vec<&Piece> = w.pieces.values().collect();
    assert_eq!(made.len(), n as usize);
    for p in made {
        assert_eq!(p.type_index, rock);
        assert_eq!(p.pos, Pos::new(at.x, at.y, 200));
        assert_eq!(p.house, None);
    }
}

/// spec: S-0317 claim 12 (the resource flag: a falling piece adds a resource cell of the tiberium of overlay type 127 at stage 0 in its own cell, a meteor piece in each of the 8 neighbours; without a resource overlay at position 127, none and no draws, a departure)
#[test]
fn claim_12_resources() {
    let r = resource_rules();
    let mut w = world(&r);
    let c = mid(0, 0);
    let id = make(&mut w, "ORE1", centre(c, 0));
    w.pieces.get_mut(&id).unwrap().counter = 1;
    pieces::turn(&mut w, id);
    assert_eq!(
        glasnost_sim::econ::resource::resource_at(&w, c),
        Some((0, 0))
    );
    let c = mid(5, 5);
    let id = make(&mut w, "OREM", centre(c, 0));
    set(
        &mut w,
        id,
        [centre(c, 0).x, centre(c, 0).y, 0],
        ["0", "0", "0"],
    );
    w.pieces.get_mut(&id).unwrap().counter = 1;
    pieces::turn(&mut w, id);
    assert_eq!(glasnost_sim::econ::resource::resource_at(&w, c), None);
    for (dx, dy) in glasnost_sim::regions::STEPS {
        let n = CellXY::new(c.x + dx, c.y + dy);
        assert_eq!(
            glasnost_sim::econ::resource::resource_at(&w, n),
            Some((0, 0))
        );
    }
    // Without overlay position 127: no cell and no draws.
    let r = rules();
    let mut w = world(&r);
    let id = make(&mut w, "ORE1", centre(mid(0, 0), 0));
    w.pieces.get_mut(&id).unwrap().counter = 1;
    let mut probe = w.rng.clone();
    pieces::turn(&mut w, id);
    assert_eq!(w.rng.raw(), probe.raw());
}

/// spec: S-0317 claim 13 (the attached system at the creation point, owned by the piece, no owner house, directly after the piece in the update order; a falling piece's displacement zero, a meteor piece's the whole flight; marked when the piece leaves play) and claim 14 (the start sound at the creation point); S-0616 claim 1 (revision 20: a voxel-model animation's system is made at the piece's creation point, not its start position, and keeps claim 5's displacement from it: zero for a falling piece, the flight from the start position to the creation point for a meteor piece)
#[test]
fn claim_13_14_system_and_sound() {
    let r = rules();
    let mut w = world(&r);
    let at = centre(mid(0, 0), 600);
    w.effects.clear();
    let id = make(&mut w, "SMOKY", at);
    let (&sid, sys) = w
        .gas
        .systems
        .iter()
        .find(|(_, s)| s.owner_object == Some(id))
        .expect("the attached system");
    assert_eq!(sys.pos, at);
    assert_eq!(sys.owner, None);
    assert_eq!(sys.owner_offset, [0, 0, 0]);
    let order = &w.update.order;
    let k = order.iter().position(|&x| x == id).unwrap();
    assert_eq!(order[k + 1], sid, "directly after its piece");
    assert!(w.effects.iter().any(|e| matches!(
        e,
        Effect::PieceSound { id: p, sound, pos } if *p == id && sound == "Hum" && *pos == at
    )));
    // The system follows the piece (S-0616 claim 5).
    pieces::turn(&mut w, id);
    glasnost_sim::particles::smoke_turn(&mut w, sid);
    assert_eq!(w.gas.systems[&sid].pos, piece(&w, id).pos);
    // A meteor piece: the displacement is the whole flight back to the creation point.
    let id = make(&mut w, "METSMOKE", at);
    let start = piece(&w, id).pos;
    let sys = w
        .gas
        .systems
        .values()
        .find(|s| s.owner_object == Some(id))
        .unwrap();
    assert_eq!(
        sys.owner_offset,
        [at.x - start.x, at.y - start.y, at.z - start.z]
    );
    // The end marks the system.
    w.pieces.get_mut(&id).unwrap().counter = 1;
    pieces::turn(&mut w, id);
    assert!(
        w.gas
            .systems
            .values()
            .any(|s| s.ending && s.owner_object.is_none())
    );
}

/// spec: S-0317 Changes for other specs (S-0303 claim 22, S-0312 claim 4: the debris types are dealt as pieces, created with claim 3's draws)
#[test]
fn producers_make_pieces() {
    let r = rules();
    let mut w = world(&r);
    let at = centre(mid(0, 0), 0);
    glasnost_sim::combat::death::deal_debris(
        &mut w,
        3,
        &["ROCK".to_owned()],
        &[5],
        at,
        Some(glasnost_sim::world::HouseId(1)),
    );
    assert!(!w.pieces.is_empty());
    assert!(
        w.pieces
            .values()
            .all(|p| p.pos == at && p.house == Some(glasnost_sim::world::HouseId(1)))
    );
    round_trip(&w);
}

/// spec: S-0317 claim 1 (the object of play: the hits of its impact and of its end carry no attacker and no house, even for a piece that carries its object's house; a victim they destroy credits no kill to any house, and counts one loss for its own)
#[test]
fn claim_1_hits_carry_no_attacker_and_no_house() {
    use glasnost_sim::econ::stats;
    use glasnost_sim::world::HouseId;
    for ending in [false, true] {
        let r = rules();
        let mut w = world(&r);
        let c = mid(0, 0);
        let victim = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
        w.obj_mut(victim).unwrap().health = 1;
        let kills: Vec<i32> = (0..2).map(|h| stats::of(&w, HouseId(h)).kills).collect();
        let losses = stats::of(&w, HouseId(1)).losses;
        let at = centre(c, 0);
        let t = ti(&w, "ROCK");
        // A piece carrying house 0 (as a death's pieces carry their object's).
        let id = pieces::create(&mut w, t, at, Some(HouseId(0))).expect("piece made");
        if ending {
            w.pieces.get_mut(&id).unwrap().counter = 1;
        } else {
            set(&mut w, id, [at.x + 20, at.y, 5], ["0", "0", "-10"]);
        }
        w.tick = 1;
        pieces::turn(&mut w, id);
        testkit::run(&mut w, 3);
        assert!(
            w.obj(victim).is_none_or(|o| o.health <= 0),
            "ending {ending}: destroyed"
        );
        let after: Vec<i32> = (0..2).map(|h| stats::of(&w, HouseId(h)).kills).collect();
        assert_eq!(after, kills, "ending {ending}: no kill credited");
        assert_eq!(
            stats::of(&w, HouseId(1)).losses,
            losses + 1,
            "ending {ending}: its own loss"
        );
    }
}

/// Puts a piece's flight state from decimal texts (the flight position may
/// be fractional); the position is the flight position truncated.
fn set_exact(w: &mut World, id: ObjectId, flight: [&str; 3], v: [&str; 3]) {
    let m = mode(w);
    let p = w.pieces.get_mut(&id).unwrap();
    p.flight = flight.map(|x| s(x).to_f32(m));
    p.velocity = v.map(|x| s(x).to_f32(m));
    let [x, y, z] = p.flight.map(|c| c.to_f64().to_i32_common());
    p.pos = Pos::new(x, y, z);
}

fn single(x: F64, m: Rounding) -> F32 {
    x.to_f32(m)
}

/// Numeric equality per component (+0 equals −0: the spec's matrix sums
/// fix no sign for a zero).
fn same(a: [F32; 3], b: [F32; 3]) -> bool {
    (0..3).all(|i| a[i].to_f64().eq_num(b[i].to_f64()))
}

/// m · v grouped as S-0613 F3: (a_y + a_z) + a_x for x, (a_y + a_x) + a_z
/// for y and z, each component stored as a single.
fn mat(m3: &[[F32; 3]; 3], v: [F32; 3], m: Rounding) -> [F32; 3] {
    let row = |r: &[F32; 3]| {
        let a: Vec<F64> = (0..3)
            .map(|j| r[j].to_f64().mul(v[j].to_f64(), m))
            .collect();
        (a[0], a[1], a[2])
    };
    let (x0, x1, x2) = row(&m3[0]);
    let (y0, y1, y2) = row(&m3[1]);
    let (z0, z1, z2) = row(&m3[2]);
    [
        single(x1.add(x2, m).add(x0, m), m),
        single(y1.add(y0, m).add(y2, m), m),
        single(z1.add(z0, m).add(z2, m), m),
    ]
}

/// F8 as the spec writes it, for the test: v″ = S · M · Z · (ε (Mᵀ · S · v)).
fn f8(m3: &[[F32; 3]; 3], v: [F32; 3], eps: F32, m: Rounding) -> [F32; 3] {
    let flip_y = |a: [F32; 3]| [a[0], a[1].to_f64().negated().to_f32(m), a[2]];
    let mut t = *m3;
    for (i, r) in t.iter_mut().enumerate() {
        for (j, e) in r.iter_mut().enumerate() {
            *e = m3[j][i];
        }
    }
    let w = mat(&t, flip_y(v), m);
    let w = w.map(|c| single(c.to_f64().mul(eps.to_f64(), m), m));
    let w = [w[0], w[1], w[2].to_f64().negated().to_f32(m)];
    flip_y(mat(m3, w, m))
}

/// spec: S-0317 claim 10 and Constants (the cliff test's z₀ is the flight position's single-precision height; a vertical speed below −0.0002 undoes a contact that puts the piece higher, any other, 0 included, one that lifts it more than 1 lepton above z₀ + u)
#[test]
fn claim_10_cliff_line_and_flight_height() {
    let r = rules();
    // (flight height, position height, u, undone)
    let cases = [
        // u exactly 0: a lift of 2 above z₀ is undone, a lift of exactly 1 stands.
        ("206", None, "0", true),
        ("207", None, "0", false),
        // −0.0001 is not below the line: z₁ = 208 ≤ 207.5 − 0.0001 + 1, so the move stands.
        ("207.5", None, "-0.0001", false),
        // −0.001 is below it: z₁ = 208 > 207.5, undone.
        ("207.5", None, "-0.001", true),
        // A rising piece whose position is far below its flight position: the
        // flight height decides, 208 ≤ 205 + 3 + 1, and the piece climbs.
        ("205", Some(150), "3", false),
    ];
    for (z0, posz, u, undone) in cases {
        let mut w = world(&r);
        let low = mid(0, 0);
        let high = mid(1, 0);
        w.cells.grid.get_mut(high).unwrap().level = 2;
        let at = centre(low, 0);
        let id = make(&mut w, "ROCK", at);
        let x = format!("{}", at.x + 125);
        let y = format!("{}", at.y);
        set_exact(&mut w, id, [&x, &y, z0], ["10", "0", u]);
        if let Some(z) = posz {
            w.pieces.get_mut(&id).unwrap().pos.z = z;
        }
        let before = piece(&w, id);
        pieces::turn(&mut w, id);
        let p = piece(&w, id);
        assert_eq!(p.flight == before.flight, undone, "z₀ {z0}, u {u}");
        assert_eq!(p.pos.cell() == low, undone, "z₀ {z0}, u {u}");
    }
}

/// spec: S-0317 claim 17 and F8 (a contact on a sloped cell reflects the velocity in the plane of S-0106 C4's transform and scales it by ε, each step stored in single precision; the flat shape is claim 10's flat rule) and Edge cases (quirks.voxel_debris_deck_rebound_slope: a deck contact over sloped ground rebounds off the slope; off, by the flat rule)
#[test]
fn claim_17_f8_rebound_on_a_slope() {
    let r = rules();
    let ramps = glasnost_sim::gas::ramps();
    for shape in [0u8, 1, 6, 14, 18] {
        let mut w = world(&r);
        let m = mode(&w);
        let c = mid(0, 0);
        w.cells.grid.get_mut(c).unwrap().slope = shape;
        let at = centre(c, 0);
        let f = w.surface_at(at.x, at.y);
        let id = make(&mut w, "CLIFF", at);
        let z = format!("{}", f + 5);
        let (x, y) = (format!("{}", at.x), format!("{}", at.y));
        set_exact(&mut w, id, [&x, &y, &z], ["2", "-3", "-10"]);
        pieces::turn(&mut w, id);
        let p = piece(&w, id);
        let v = [
            s("2").to_f32(m),
            s("-3").to_f32(m),
            single(s("-10").to_f32(m).to_f64().sub(g(), m), m),
        ];
        let eps = e30(m).to_f32(m);
        let want = f8(&ramps[usize::from(shape)], v, eps, m);
        assert!(
            same(p.velocity, want),
            "shape {shape}: {:?} {want:?}",
            p.velocity
        );
        // The flat rule: (εvₓ, εv_y, −εv_z), each product stored.
        let flat = [
            single(v[0].to_f64().mul(eps.to_f64(), m), m),
            single(v[1].to_f64().mul(eps.to_f64(), m), m),
            single(v[2].to_f64().mul(eps.to_f64(), m), m)
                .to_f64()
                .negated()
                .to_f32(m),
        ];
        if shape == 0 || shape == 18 {
            assert_eq!(p.velocity, flat, "shape {shape}: the flat rule");
        } else {
            assert_ne!(p.velocity, flat, "shape {shape}: a tilted plane");
        }
    }
    // A deck contact over a sloped cell.
    for slope in [true, false] {
        let mut w = world(&r);
        let m = mode(&w);
        let mut q = w.rules().quirks;
        q.voxel_debris_deck_rebound_slope = slope;
        set_quirks(&mut w, q);
        let c = mid(0, 0);
        w.cells.grid.get_mut(c).unwrap().slope = 1;
        let i = w.cells.idx(c).unwrap();
        w.cells.deck[i] = true;
        let at = centre(c, 0);
        let deck = w.surface_at(at.x, at.y) + 416;
        let id = make(&mut w, "CLIFF", at);
        let z = format!("{}", deck + 4);
        let (x, y) = (format!("{}", at.x), format!("{}", at.y));
        set_exact(&mut w, id, [&x, &y, &z], ["0", "0", "-10"]);
        pieces::turn(&mut w, id);
        let p = piece(&w, id);
        assert_eq!(p.flight[2], d(i64::from(deck), m).to_f32(m), "on the deck");
        let v = [
            F32(0),
            F32(0),
            single(s("-10").to_f32(m).to_f64().sub(g(), m), m),
        ];
        let shape = if slope { 1 } else { 0 };
        let want = f8(&ramps[shape], v, e30(m).to_f32(m), m);
        assert!(
            same(p.velocity, want),
            "deck rebound slope {slope}: {:?} {want:?}",
            p.velocity
        );
    }
}

/// Claim 18's case: a fresh world whose old cell is `mid(0, 0)` at level 0,
/// whose new cell (dx, dy) from it and the cells `highs` (offsets from the
/// old cell) are at level 2; a CLIFF piece crosses into the new cell from
/// near the matching corner, descending. Returns the velocity the cliff
/// turn leaves and the velocity at the start of the turn.
fn cliff_case(dx: i32, dy: i32, highs: &[(i32, i32)], faults: bool) -> ([F32; 3], [F32; 3]) {
    let r = rules();
    let mut w = world(&r);
    let mut q = w.rules().quirks;
    q.voxel_debris_cliff_step_faults = faults;
    set_quirks(&mut w, q);
    let low = mid(0, 0);
    for &(ox, oy) in std::iter::once(&(dx, dy)).chain(highs) {
        w.cells.grid.get_mut(mid(ox, oy)).unwrap().level = 2;
    }
    let base = Pos::centre(low, 0);
    let edge = |k: i32, c: i32| match k {
        1 => c - 128 + 250,
        -1 => c - 128 + 5,
        _ => c,
    };
    let (x, y) = (edge(dx, base.x), edge(dy, base.y));
    let ux = if dx == 0 {
        "3"
    } else if dx > 0 {
        "12"
    } else {
        "-12"
    };
    let uy = if dy == 0 {
        "7"
    } else if dy > 0 {
        "10"
    } else {
        "-10"
    };
    let id = make(&mut w, "CLIFF", base);
    let (xs, ys) = (format!("{x}"), format!("{y}"));
    set_exact(&mut w, id, [&xs, &ys, "150"], [ux, uy, "-5"]);
    let before = piece(&w, id);
    pieces::turn(&mut w, id);
    let p = piece(&w, id);
    assert_eq!(p.flight, before.flight, "({dx}, {dy}) {highs:?}: undone");
    (p.velocity, before.velocity)
}

/// spec: S-0317 claim 18 and F9 (the cliff turn by step direction: across the step for a straight step; for a diagonal step the face of the one high neighbour of F9's table, else diagonal; each component scaled by e as imported, not narrowed, and stored) and Edge cases (quirks.voxel_debris_cliff_step_faults: the north-east and south-west steps' mirrored cells and the north-west turn (c, uₓ, u_y); off, the cells beside the corner and across y)
#[test]
fn claim_18_f9_cliff_turns() {
    let m = mode(&world(&rules()));
    let e = e30(m);
    assert_ne!(
        e,
        e.to_f32(m).to_f64(),
        "0.3 is not held in single precision"
    );
    let scale = |a: [F32; 3]| a.map(|c| c.to_f64().mul(e, m).to_f32(m));
    let n = |c: F32| c.to_f64().negated().to_f32(m);
    let across_x = |u: [F32; 3]| scale([n(u[0]), u[1], u[2]]);
    let across_y = |u: [F32; 3]| scale([u[0], n(u[1]), u[2]]);
    let diag_15 = |u: [F32; 3]| scale([u[1], u[0], u[2]]);
    let diag_37 = |u: [F32; 3]| scale([n(u[1]), n(u[0]), u[2]]);
    let nw = |u: [F32; 3]| scale([s("2.3771235942840576").to_f32(m), u[0], u[1]]);
    let check = |dx: i32,
                 dy: i32,
                 highs: &[(i32, i32)],
                 faults: bool,
                 want: &dyn Fn([F32; 3]) -> [F32; 3],
                 what: &str| {
        let (v, u) = cliff_case(dx, dy, highs, faults);
        assert_eq!(v, want(u), "({dx}, {dy}) {highs:?} faults {faults}: {what}");
    };
    for faults in [true, false] {
        // Straight steps: d = 0, 4 across y; d = 2, 6 across x.
        check(0, -1, &[], faults, &across_y, "d 0");
        check(0, 1, &[], faults, &across_y, "d 4");
        check(1, 0, &[], faults, &across_x, "d 2");
        check(-1, 0, &[], faults, &across_x, "d 6");
        // d = 3 (south-east): P = (x + 1, y), Q = (x, y + 1).
        check(1, 1, &[(1, 0)], faults, &across_x, "d 3, P");
        check(1, 1, &[(0, 1)], faults, &across_y, "d 3, Q");
        check(1, 1, &[], faults, &diag_37, "d 3, neither");
        check(1, 1, &[(1, 0), (0, 1)], faults, &diag_37, "d 3, both");
        // d = 7 (north-west): P = (x − 1, y) across x.
        check(-1, -1, &[(-1, 0)], faults, &across_x, "d 7, P");
        check(-1, -1, &[], faults, &diag_37, "d 7, neither");
    }
    // The faults: d = 1 and 5 examine the cells behind the move, swapped.
    check(1, -1, &[(-1, 0)], true, &across_y, "d 1, (x − 1, y)");
    check(1, -1, &[(0, 1)], true, &across_x, "d 1, (x, y + 1)");
    check(
        1,
        -1,
        &[(1, 0)],
        true,
        &diag_15,
        "d 1, (x + 1, y) is not examined",
    );
    check(-1, 1, &[(1, 0)], true, &across_y, "d 5, (x + 1, y)");
    check(-1, 1, &[(0, -1)], true, &across_x, "d 5, (x, y − 1)");
    check(-1, 1, &[], true, &diag_15, "d 5, neither");
    check(-1, -1, &[(0, -1)], true, &nw, "d 7, the north-west turn");
    // With the switch off: the cells beside the corner, and across y.
    check(1, -1, &[(1, 0)], false, &across_x, "d 1, (x + 1, y)");
    check(1, -1, &[(0, -1)], false, &across_y, "d 1, (x, y − 1)");
    check(
        1,
        -1,
        &[(-1, 0)],
        false,
        &diag_15,
        "d 1, (x − 1, y) is not examined",
    );
    check(-1, 1, &[(-1, 0)], false, &across_x, "d 5, (x − 1, y)");
    check(-1, 1, &[(0, 1)], false, &across_y, "d 5, (x, y + 1)");
    check(-1, -1, &[(0, -1)], false, &across_y, "d 7, across y");
}

/// spec: S-0317 Import mapping (Elasticity absent: the single-precision 0.8, widened; a given value kept as the double it reads as) and F8 (ε narrowed once)
#[test]
fn elasticity_as_imported_and_narrowed() {
    let r = rules();
    let w = world(&r);
    let m = mode(&w);
    let types = &w.rules().pieces.types;
    let bare = &types[ti(&w, "BARE") as usize];
    // The single-precision 0.8 is the single nearest to 0.8, a constant.
    let single8 = s("0.8").to_f32(Rounding::Nearest);
    assert_eq!(bare.elasticity, single8.to_f64());
    assert_ne!(bare.elasticity, s("0.8"), "not the double 0.8");
    assert_eq!(bare.epsilon, single8);
    let cliff = &types[ti(&w, "CLIFF") as usize];
    assert_eq!(cliff.elasticity, e30(m), "the double 30% reads as");
    assert_eq!(cliff.epsilon, e30(m).to_f32(m), "narrowed once");
    assert_ne!(cliff.elasticity, cliff.epsilon.to_f64());
}

/// The CLIFF type's elasticity as imported: 30%, a double (S-0003 claim 6).
fn e30(m: Rounding) -> F64 {
    glasnost_num::decimal::parse("30%", glasnost_num::decimal::Field::Decimal, m).unwrap()
}

/// spec: S-0317 claim 19 (the first building listed in the ground layer counts unless it is a laser-fence section, every section inactive in our engine, or a vehicle-like building; with no building listed, a wall overlay counts)
#[test]
fn claim_19_cells_that_hold_a_building() {
    use glasnost_rules::sim::BuildingType;
    let mut r = rules();
    let mut building = |name: &str, f: &dyn Fn(&mut BuildingType)| {
        let mut t = BuildingType {
            strength: Some(1000),
            foundation: Some([1, 1]),
            ..Default::default()
        };
        f(&mut t);
        register(&mut r, "buildings", name);
        r.sim.buildings.insert(name.into(), t);
    };
    building("FENCE", &|b| b.laser_fence = Some(true));
    building("HUT", &|_| {});
    building("MOBILE", &|b| b.undeploys_into = Some("BUGGY".into()));
    // Returns whether a move to 120 above the ground of cell c is a contact.
    let contact = |set_up: &dyn Fn(&mut World, CellXY)| {
        let mut w = world(&r);
        let m = mode(&w);
        let c = mid(0, 0);
        set_up(&mut w, c);
        let at = centre(c, 0);
        let id = make(&mut w, "BARE", at);
        set(&mut w, id, [at.x, at.y, 120], ["0", "0", "0"]);
        pieces::turn(&mut w, id);
        piece(&w, id).flight[2] == d(0, m).to_f32(m)
    };
    let spawn = |w: &mut World, c: CellXY, name: &str| {
        testkit::spawn(w, Kind::Building, name, 1, c, 0);
    };
    assert!(!contact(&|_, _| {}), "an empty cell");
    assert!(contact(&|w, c| spawn(w, c, "HUT")), "a building");
    assert!(
        !contact(&|w, c| spawn(w, c, "FENCE")),
        "a laser-fence section"
    );
    assert!(
        !contact(&|w, c| spawn(w, c, "MOBILE")),
        "a vehicle-like building"
    );
    for wall in [2u8, 26, 243] {
        assert!(
            contact(&|w, c| w.cells.grid.get_mut(c).unwrap().overlay = wall),
            "wall overlay {wall}"
        );
    }
    assert!(
        !contact(&|w, c| w.cells.grid.get_mut(c).unwrap().overlay = 3),
        "another overlay"
    );
    assert!(
        !contact(&|w, c| {
            w.cells.grid.get_mut(c).unwrap().overlay = 2;
            spawn(w, c, "FENCE");
        }),
        "a laser fence listed first hides the wall overlay"
    );
    // Two buildings in one cell: the first listed decides.
    for (a, b) in [("FENCE", "HUT"), ("HUT", "FENCE")] {
        let mut first_is_hut = false;
        let got = contact(&|w, c| {
            spawn(w, c, a);
            spawn(w, c, b);
        });
        {
            let mut w = world(&r);
            let c = mid(0, 0);
            spawn(&mut w, c, a);
            spawn(&mut w, c, b);
            let firstb = w
                .cells
                .list(c, glasnost_sim::world::Layer::Ground)
                .iter()
                .copied()
                .find(|&id| w.obj(id).is_some_and(|o| o.kind == Kind::Building))
                .unwrap();
            let hut = w.rules().index_of(Kind::Building, "HUT").unwrap();
            first_is_hut |= w.obj(firstb).unwrap().type_index == hut;
        }
        assert_eq!(got, first_is_hut, "{a} then {b}");
    }
}

/// spec: S-0317 claim 20 (a piece is never removed for leaving the legal map: off it, it meets the off-map cell, flat at height 0 and clear, rebounds there by the flat rule and keeps its turns)
#[test]
fn claim_20_off_the_legal_map() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let at = Pos::new(-2000, -2000, 0);
    assert!(w.cells.cell(at.cell()).is_none(), "off the legal map");
    let id = make(&mut w, "CLIFF", centre(mid(0, 0), 0));
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    w.effects.clear();
    pieces::turn(&mut w, id);
    let p = piece(&w, id);
    assert_eq!(p.flight[2], d(0, m).to_f32(m), "the ground at height 0");
    let vz = single(s("-10").to_f32(m).to_f64().sub(g(), m), m);
    let eps = e30(m).to_f32(m);
    assert_eq!(
        p.velocity[2],
        single(vz.to_f64().mul(eps.to_f64(), m), m)
            .to_f64()
            .negated()
            .to_f32(m),
        "the flat rebound"
    );
    assert!(p.counter > 0, "not on water: no end set");
    // Far off the map, high above the off-map ground: it keeps flying.
    set(&mut w, id, [-20000, -20000, 3000], ["-50", "-50", "0"]);
    for _ in 0..5 {
        pieces::turn(&mut w, id);
    }
    assert!(
        w.pieces.contains_key(&id),
        "never removed for leaving the map"
    );
}

/// spec: S-0317 claim 21 and Constants (a piece whose placement point is exactly (0, 0, 0) takes its creation draws, makes its system and plays its start sound, but is not placed: it never enters the update order and never takes a turn)
#[test]
fn claim_21_no_placement_at_the_origin() {
    let r = rules();
    let mut w = world(&r);
    let mut other = world(&r);
    let origin = Pos::new(0, 0, 0);
    w.effects.clear();
    let systems = w.gas.systems.len();
    let id = make(&mut w, "SMOKY", origin);
    make(&mut other, "SMOKY", Pos::new(1, 0, 0));
    assert_eq!(w.rng, other.rng, "every creation draw is taken");
    assert!(!w.pieces.contains_key(&id), "not placed");
    assert!(!w.update.contains(id), "never in the update order");
    assert_eq!(w.gas.systems.len(), systems + 1, "its system is made");
    let sid = *w
        .gas
        .systems
        .iter()
        .find(|(_, s)| s.owner_object == Some(id))
        .expect("the system keeps it as its owner")
        .0;
    assert!(w.effects.iter().any(|e| matches!(
        e,
        Effect::PieceSound { id: p, sound, pos } if *p == id && sound == "Hum" && *pos == origin
    )));
    testkit::run(&mut w, 1);
    let sys = &w.gas.systems[&sid];
    assert_eq!(sys.owner_object, Some(id), "its owner never leaves play");
    assert_eq!(sys.pos, origin, "nor does it move");
    round_trip(&w);
}

/// spec: S-0317 claim 22 and F10 (the orientation is the identity at creation; each flying turn turns once in the current sense; every contact, an undone one and one whose turn stops the piece included, reverses the sense before that turn's rotation; the net count is n₊ − n₋)
#[test]
fn claim_22_net_turns_and_reversal() {
    let r = rules();
    let mut w = world(&r);
    let c = mid(0, 0);
    let at = centre(c, 0);
    let id = make(&mut w, "ROCK", at);
    let fresh = piece(&w, id);
    assert_eq!(
        (fresh.turns, fresh.reversed),
        (0, false),
        "the identity at creation"
    );
    let fly = |w: &mut World, z: i32, v: [&str; 3]| {
        set(w, id, [at.x, at.y, z], v);
        w.pieces.get_mut(&id).unwrap().counter = 100;
        pieces::turn(w, id);
        let p = piece(w, id);
        (p.turns, p.reversed)
    };
    // Three free turns high above the ground.
    for n in 1..=3 {
        assert_eq!(fly(&mut w, 2000, ["0", "0", "0"]), (n, false));
    }
    // A ground contact: the contact turn already turns in the reversed sense.
    assert_eq!(fly(&mut w, 5, ["0", "0", "-20"]), (2, true));
    // Two free turns in the reversed sense: net 0.
    assert_eq!(fly(&mut w, 2000, ["0", "0", "0"]), (1, true));
    assert_eq!(fly(&mut w, 2000, ["0", "0", "0"]), (0, true));
    // A contact whose turn stops the piece reverses the sense too.
    assert_eq!(fly(&mut w, 0, ["0", "0", "0"]), (1, false));
    assert_eq!(piece(&w, id).counter, 0, "stopped");
    // The ending turn (claim 6) leaves play before any flight, so it has no pose to change.
    // A contact that the cliff test undoes reverses the sense (claim 10's undone case).
    let mut w = world(&r);
    let low = mid(0, 0);
    w.cells.grid.get_mut(mid(1, 0)).unwrap().level = 2;
    let at = centre(low, 0);
    let id = make(&mut w, "ROCK", at);
    set(&mut w, id, [at.x + 120, at.y, 150], ["30", "0", "-5"]);
    let before = piece(&w, id);
    pieces::turn(&mut w, id);
    let p = piece(&w, id);
    assert_eq!(p.flight, before.flight, "undone");
    assert_eq!((p.turns, p.reversed), (-1, true));
}
