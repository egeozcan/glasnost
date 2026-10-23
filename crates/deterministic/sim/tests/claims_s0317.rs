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

/// spec: S-0317 claim 3 and F1 (a falling piece: four raw values for the spin, vertical, y and x speeds, then three ranged draws R(−65535, 65535) for the axis (a₃, a₂, a₁); the velocity of F1 in single precision; the start position 10 leptons above the creation point; the position the creation point) and claim 2 (the counter starts at the duration)
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
        "turned off the face (interim, Q-1834)"
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

/// spec: S-0317 claim 13 (the attached system at the creation point, owned by the piece, no owner house, directly after the piece in the update order; a falling piece's displacement zero, a meteor piece's the whole flight; marked when the piece leaves play) and claim 14 (the start sound at the creation point)
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
