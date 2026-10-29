//! Claim tests for bouncing animations (S-0319) on synthetic data: invented
//! animation types, warheads, overlays and a resource, every name and
//! number our own. The ledgers are `docs/m8-claims/intake15a-S-0319.md` and
//! `docs/m8-claims/intake33-S-0319.md`.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::{F32, F64, Rounding};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Decimal, LandRow, LandType, OverlayType};
use glasnost_sim::anims::{self, Anim, Spawn};
use glasnost_sim::bounce::Flight;
use glasnost_sim::combat::effects::Effect;
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

fn add(r: &mut Ruleset, name: &str, t: AnimationType) {
    register(r, "animations", name);
    r.sim.animations.insert(name.into(), t);
}

/// Two hundred frames at rate 900 (one update per frame), one pass: an
/// animation that plays long enough to land.
fn long() -> AnimationType {
    AnimationType {
        frame_count: Some(200),
        rate: Some(900),
        ..Default::default()
    }
}

/// The rules: testkit's combat rules plus the animation types
/// - DROP: a falling type (bouncer flag) with every S-0319 default;
/// - ROCK: a falling type with a bounce, expiry and trailer animation,
///   damage 40 in a radius of 200 with the warhead HE, elasticity 0.5;
/// - STAR: a meteor type (with the bouncer flag too), M 6, Z_min −10,
///   spawning up to two-per-draw PEBBLEs;
/// - PEBBLE, Bump, Boom, Trail, Wake, Spl1 to Spl3: plain long types;
/// - SLOW: four frames, a falling type, follow-up NEXTP (no flags);
/// - QUIET: a type with neither flag whose follow-up DROP has one.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    add(
        &mut r,
        "DROP",
        AnimationType {
            bouncer: Some(true),
            ..long()
        },
    );
    add(
        &mut r,
        "ROCK",
        AnimationType {
            bouncer: Some(true),
            damage: dec("40.7"),
            damage_radius: Some(200),
            warhead: Some("HE".into()),
            bounce_anim: Some("Bump".into()),
            expire_anim: Some("Boom".into()),
            elasticity: dec("0.5"),
            ..long()
        },
    );
    add(
        &mut r,
        "STAR",
        AnimationType {
            bouncer: Some(true),
            is_meteor: Some(true),
            min_z_vel: dec("-10"),
            max_xy_vel: dec("6"),
            spawns: Some("PEBBLE".into()),
            spawn_count: Some(2),
            ..long()
        },
    );
    for n in [
        "PEBBLE", "Bump", "Boom", "Trail", "Wake", "Spl1", "Spl2", "Spl3", "NEXTP",
    ] {
        add(&mut r, n, long());
    }
    add(
        &mut r,
        "SLOW",
        AnimationType {
            bouncer: Some(true),
            frame_count: Some(4),
            rate: Some(900),
            next: Some("NEXTP".into()),
            ..Default::default()
        },
    );
    add(
        &mut r,
        "QUIET",
        AnimationType {
            frame_count: Some(2),
            rate: Some(900),
            next: Some("DROP".into()),
            ..Default::default()
        },
    );
    r.sim.general.wake = Some("Wake".into());
    r.sim.general.splash_list = Some(vec!["Spl1".into(), "Spl2".into(), "Spl3".into()]);
    r
}

/// [`rules`] with buildable clear land, eight resource overlays OV0 to OV7
/// and the falling type GROW with the resource flag, the spawn overlay
/// type OV2 and the spread radius `radius`.
fn resource_rules(radius: i32) -> Ruleset {
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
    for i in 0..8 {
        let n = format!("OV{i}");
        register(&mut r, "overlays", &n);
        r.sim.overlays.insert(
            n,
            OverlayType {
                tiberium: Some(true),
                ..Default::default()
            },
        );
    }
    add(
        &mut r,
        "GROW",
        AnimationType {
            bouncer: Some(true),
            is_tiberium: Some(true),
            tiberium_spawn_type: Some("ov2".into()),
            tiberium_spread_radius: Some(radius),
            ..long()
        },
    );
    r
}

fn world(r: &Ruleset) -> World {
    testkit::game_with(r, N, 7)
}

fn make(w: &mut World, name: &str, at: Pos) -> ObjectId {
    anims::spawn(w, name, Some(at), Spawn::default())
        .id
        .expect("animation made")
}

fn get(w: &World, id: ObjectId) -> Anim {
    w.anims.anims.get(&id).cloned().expect("animation in play")
}

fn flight(w: &World, id: ObjectId) -> Flight {
    get(w, id).flight.expect("a flight")
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

/// 1.4 in single precision, widened (claim 4).
fn g() -> F64 {
    s("1.4").to_f32(Rounding::Nearest).to_f64()
}

/// Puts an animation's flight state and its position in play.
fn set(w: &mut World, id: ObjectId, p: [i32; 3], v: [&str; 3]) {
    let m = mode(w);
    let a = w.anims.anims.get_mut(&id).unwrap();
    a.flight = Some(Flight {
        p: p.map(|c| d(i64::from(c), m).to_f32(m)),
        v: v.map(|x| s(x).to_f32(m)),
    });
    a.pos = Some(Pos::new(p[0], p[1], p[2]));
}

fn anims_made(w: &World) -> Vec<(String, Pos)> {
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
    if s == 0 {
        return 0;
    }
    i64::from(glasnost_num::int::rem((r as i32).wrapping_abs(), s))
}

fn set_quirks(w: &mut World, f: impl Fn(&mut glasnost_rules::quirks::Quirks)) {
    let mut rules = (*w.rules_arc()).clone();
    f(&mut rules.quirks);
    w.rules = Some(std::sync::Arc::new(rules));
}

fn gone(w: &World, id: ObjectId) -> bool {
    !w.anims.anims.contains_key(&id) && !w.update.contains(id)
}

fn round_trip(w: &World) {
    let bytes = save::save(w);
    let back = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&back), save::state_hash(w));
    assert_eq!(back.anims, w.anims);
}

/// spec: S-0319 claim 2 and F1 (a falling animation: three raw values r₁ vertical, r₂ second horizontal, r₃ first horizontal, then three ranged draws R(−65535, 65535); v by F1 with W_xy = trunc(2M), W_z = trunc(3.5 − Z_min + 1); placed at the creation point, its flight position 10 leptons above) and Import mapping (M 15.0, Z_min 3.5 when absent); S-0202 claim 10 (the bouncing row's draws)
#[test]
fn claim_2_falling_creation() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let half = w.rules().quirks.ranged_draw_half_span;
    let at = centre(mid(0, 0), 0);
    let mut probe: Rng = w.rng.clone();
    let id = make(&mut w, "DROP", at);
    let (r1, r2, r3) = (probe.raw(), probe.raw(), probe.raw());
    for _ in 0..3 {
        probe.ranged(-65535, 65535, half);
    }
    assert_eq!(w.rng, probe, "three raw values and three ranged draws");
    let f = flight(&w, id);
    let mm = s("15.0");
    let vx = d(mag_rem(r3, 30), m).sub(mm, m).to_f32(m);
    let vy = d(mag_rem(r2, 30), m).sub(mm, m).to_f32(m);
    // W_z = trunc(3.5 − 3.5 + 1) = 1: the vertical speed is Z_min exactly.
    let vz = d(mag_rem(r1, 1), m).add(s("3.5"), m).to_f32(m);
    assert_eq!(f.v, [vx, vy, vz]);
    assert_eq!(vz, s("3.5").to_f32(m));
    assert_eq!(get(&w, id).pos, Some(at), "placed at the creation point");
    assert_eq!(
        f.p,
        [at.x, at.y, at.z + 10].map(|c| d(i64::from(c), m).to_f32(m))
    );
    assert!(w.update.contains(id));
    round_trip(&w);
}

/// spec: S-0319 claim 2, F1 and Edge cases (quirks.anim_fixed_upper_vertical_speed: the upper limit is 3.5 whatever MaxZVel says; off: the type's MaxZVel, absent 3.5)
#[test]
fn claim_2_upper_vertical_limit_switch() {
    for fixed in [true, false] {
        let mut r = rules();
        let t = r.sim.animations.get_mut("DROP").unwrap();
        t.min_z_vel = dec("1");
        t.max_z_vel = dec("9");
        // The span is formed when the rules are built, under the ruleset's switch.
        r.globals.quirks.anim_fixed_upper_vertical_speed = fixed;
        let mut w = world(&r);
        let b = w.rules().animation("DROP").unwrap().bounce.clone();
        // Fixed: trunc(3.5 − 1 + 1) = 3; off: trunc(9 − 1 + 1) = 9.
        assert_eq!(b.wz, if fixed { 3 } else { 9 }, "switch {fixed}");
        let m = mode(&w);
        let mut probe = w.rng.clone();
        let id = make(&mut w, "DROP", centre(mid(0, 0), 0));
        let r1 = probe.raw();
        let vz = d(mag_rem(r1, b.wz), m).add(s("1"), m).to_f32(m);
        assert_eq!(flight(&w, id).v[2], vz);
    }
}

/// spec: S-0319 claim 1 and claim 3, F2 (a type with the meteor flag, with the bouncer flag too, makes a meteor animation: q₁ the y speed, q₂ the x speed, v_z = Z_min; both horizontal speeds change sign when v_x < −v_y; T = 70 − |q₃ rem 20|; placed at the start trunc(c − T v), its flight position there; three ranged draws follow)
#[test]
fn claim_3_meteor_creation() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let half = w.rules().quirks.ranged_draw_half_span;
    let at = centre(mid(0, 0), 0);
    for _ in 0..16 {
        let mut probe: Rng = w.rng.clone();
        let id = make(&mut w, "STAR", at);
        let (q1, q2, q3) = (probe.raw(), probe.raw(), probe.raw());
        for _ in 0..3 {
            probe.ranged(-65535, 65535, half);
        }
        assert_eq!(w.rng, probe);
        let f = flight(&w, id);
        let mm = s("6");
        let mut vy = d(mag_rem(q1, 12), m).sub(mm, m).to_f32(m);
        let mut vx = d(mag_rem(q2, 12), m).sub(mm, m).to_f32(m);
        if vx.to_f64().lt(vy.to_f64().negated()) {
            vx = vx.to_f64().negated().to_f32(m);
            vy = vy.to_f64().negated().to_f32(m);
        }
        let vz = s("-10").to_f32(m);
        assert_eq!(f.v, [vx, vy, vz]);
        assert!(vx.to_f64().add(vy.to_f64(), m).ge(d(0, m)));
        let t = 70 - glasnost_num::int::rem(q3 as i32, 20).abs();
        assert!((51..=70).contains(&t));
        let back = |c: i32, v: F32| {
            d(i64::from(c), m)
                .sub(d(i64::from(t), m).mul(v.to_f64(), m), m)
                .to_i32_common()
        };
        let start = Pos::new(back(at.x, vx), back(at.y, vy), back(at.z, vz));
        assert_eq!(get(&w, id).pos, Some(start), "placed at the start position");
        assert_eq!(
            f.p,
            [start.x, start.y, start.z].map(|c| d(i64::from(c), m).to_f32(m))
        );
    }
}

/// spec: S-0319 claim 1 (a type with neither flag at creation never moves, even after a switch to a flagged follow-up type) and claim 11 (a falling animation's follow-up type with neither flag keeps the flight)
#[test]
fn claim_1_11_who_moves() {
    let r = rules();
    let mut w = world(&r);
    let at = centre(mid(0, 0), 5000);
    let quiet = make(&mut w, "QUIET", at);
    assert!(get(&w, quiet).flight.is_none());
    testkit::run(&mut w, 10);
    let a = get(&w, quiet);
    assert_eq!(
        a.anim,
        w.rules()
            .animations
            .iter()
            .position(|t| t.name == "DROP")
            .map(|i| i as u32),
        "switched to the flagged follow-up type"
    );
    assert!(a.flight.is_none(), "and still does not move");
    assert_eq!(a.pos, Some(at));
    let slow = make(&mut w, "SLOW", centre(mid(2, 2), 50_000));
    testkit::run(&mut w, 10);
    let a = get(&w, slow);
    assert_eq!(w.rules().animations[a.anim.unwrap() as usize].name, "NEXTP");
    assert!(a.flight.is_some(), "the follow-up keeps the flight");
    assert!(a.pos.unwrap().z < 50_000, "and keeps falling");
}

/// spec: S-0319 claim 4 and F3 (v′ = (v_x, v_y, RZ₂₄(v_z − 1.4)), p′ = RZ₂₄(p + v′), the position in play trunc(p′); a meteor animation has the 1.4 added back after the move) and claim 5 (the stage keeps playing)
#[test]
fn claim_4_motion_step() {
    let r = rules();
    let mut w = world(&r);
    let m = mode(&w);
    let id = make(&mut w, "DROP", centre(mid(0, 0), 3000));
    let before = flight(&w, id);
    anims::turn(&mut w, id);
    let a = get(&w, id);
    let f = a.flight.unwrap();
    let vz = before.v[2].to_f64().sub(g(), m).to_f32(m);
    assert_eq!(f.v, [before.v[0], before.v[1], vz]);
    let want: [F32; 3] = [0, 1, 2].map(|i| before.p[i].to_f64().add(f.v[i].to_f64(), m).to_f32(m));
    assert_eq!(f.p, want);
    let [x, y, z] = f.p.map(|c| c.to_f64().to_i32_common());
    assert_eq!(a.pos, Some(Pos::new(x, y, z)));
    let id = make(&mut w, "STAR", centre(mid(0, 0), 90_000));
    let before = flight(&w, id);
    anims::turn(&mut w, id);
    let f = flight(&w, id);
    let down = before.v[2].to_f64().sub(g(), m).to_f32(m);
    assert_eq!(f.p[2], before.p[2].to_f64().add(down.to_f64(), m).to_f32(m));
    assert_eq!(f.v[2], down.to_f64().add(g(), m).to_f32(m));
}

/// spec: S-0319 claim 4 (the motion step in every turn the animation receives in the update order) and S-0203 claims 2 and 3
#[test]
fn claim_4_turns_in_the_tick() {
    let r = rules();
    let mut w = world(&r);
    let id = make(&mut w, "DROP", centre(mid(0, 0), 90_000));
    let z0 = get(&w, id).pos.unwrap().z;
    testkit::run(&mut w, 3);
    assert_ne!(
        get(&w, id).pos.unwrap().z,
        z0,
        "it moves in the tick's turns"
    );
}

/// spec: S-0319 claims 6 and 7 (a ground contact: set to the ground; the vertical speed reversed and the velocity scaled by e; with S ≥ 2.5 an impact landing: the bounce animation at the position in play before the move, then a hit of trunc(δ) on each ground-layer object of the landing point's cell within the radius; none in a neighbouring cell) and claim 8 (then the expiry animation and area damage at the landing point; the animation leaves play in the same turn)
#[test]
fn claim_6_7_8_ground_impact() {
    let r = rules();
    let mut w = world(&r);
    let c = mid(0, 0);
    let next = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, mid(1, 0), 0);
    let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
    let (h0, h1) = (health(&w, near), health(&w, next));
    let at = centre(c, 0);
    let id = make(&mut w, "ROCK", at);
    set(&mut w, id, [at.x + 120, at.y, 5], ["0", "0", "-10"]);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(gone(&w, id), "ends in its landing turn");
    let made = anims_made(&w);
    assert_eq!(made[0], ("Bump".to_owned(), Pos::new(at.x + 120, at.y, 5)));
    assert_eq!(made[1], ("Boom".to_owned(), Pos::new(at.x + 120, at.y, 0)));
    assert_eq!(made.len(), 2);
    assert!(health(&w, near) < h0, "hit");
    let _ = h1;
    round_trip(&w);
}

/// spec: S-0319 claim 7 (the hits: |Δx| + |Δy| at most the radius; trunc(δ) with the warhead; only the landing point's cell) and S-0317 claim 7 (the same hits as a piece's)
#[test]
fn claim_7_hit_radius_and_cell() {
    let r = rules();
    for (offset, hit) in [(120, true), (227, false)] {
        let mut w = world(&r);
        let c = mid(0, 3);
        let far = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
        let other = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, mid(5, 5), 0);
        let (hf, ho) = (health(&w, far), health(&w, other));
        let at = centre(c, 0);
        let id = make(&mut w, "ROCK", at);
        // A landing point offset horizontally within the cell by `offset` in all.
        set(
            &mut w,
            id,
            [at.x - offset.min(127), at.y - (offset - offset.min(127)), 5],
            ["0", "0", "-10"],
        );
        // The expiry area damage would also hit: take away the expiry animation for this check.
        let mut rules = (*w.rules_arc()).clone();
        rules
            .animations
            .iter_mut()
            .find(|a| a.name == "ROCK")
            .unwrap()
            .bounce
            .expire = None;
        w.rules = Some(std::sync::Arc::new(rules));
        anims::turn(&mut w, id);
        assert_eq!(health(&w, far) < hf, hit, "offset {offset}");
        assert_eq!(health(&w, other), ho);
    }
}

/// spec: S-0319 claim 7 (with an elasticity of 0 a contact at the ground height leaves S = 0: a stopping landing, no bounce animation and no hits; the end of claim 8 follows)
#[test]
fn claim_7_stopping_landing_at_the_ground() {
    let mut r = rules();
    r.sim.animations.get_mut("ROCK").unwrap().elasticity = dec("0");
    let mut w = world(&r);
    let c = mid(0, 0);
    let at = centre(c, 0);
    let id = make(&mut w, "ROCK", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(gone(&w, id));
    let names: Vec<String> = anims_made(&w).into_iter().map(|a| a.0).collect();
    assert_eq!(names, ["Boom"], "no bounce animation, the expiry animation");
}

/// spec: S-0319 claim 7 (a step without a contact whose motion measure S of S-0317 F6 is below 2.5 is a stopping landing in the air)
#[test]
fn claim_7_stop_without_contact() {
    let r = rules();
    let mut w = world(&r);
    let at = centre(mid(0, 0), 0);
    let id = make(&mut w, "ROCK", at);
    // 30 above the ground, v_z −16.1: after the step v_z ≈ −17.5 at a height of 12, trunc(12 × 1.4 − 17.5) = 0.
    set(&mut w, id, [at.x, at.y, 30], ["0", "0", "-16.1"]);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(gone(&w, id), "a stopping landing with no contact");
    let made = anims_made(&w);
    assert_eq!(made, vec![("Boom".to_owned(), Pos::new(at.x, at.y, 12))]);
}

/// spec: S-0319 claim 6 (a high-bridge cell: from at or above the deck height to below it, on the deck; from below to at or above it, 20 below the deck) and claim 8 (an end at deck height takes the dry row even on water)
#[test]
fn claim_6_high_bridge_deck() {
    let r = rules();
    let mut w = world(&r);
    let c = mid(0, 0);
    let i = w.cells.idx(c).unwrap();
    w.cells.deck[i] = true;
    set_land(&mut w, c, land::WATER);
    let at = centre(c, 0);
    let id = make(&mut w, "ROCK", at);
    set(&mut w, id, [at.x, at.y, 420], ["0", "0", "-10"]);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(gone(&w, id));
    let names: Vec<(String, Pos)> = anims_made(&w);
    assert_eq!(
        names.last().unwrap(),
        &("Boom".to_owned(), Pos::new(at.x, at.y, 416)),
        "on the deck: the dry row at deck height over water"
    );
    let id = make(&mut w, "DROP", at);
    set(&mut w, id, [at.x, at.y, 410], ["0", "0", "20"]);
    anims::turn(&mut w, id);
    // Under the deck: S counts the height 396 from the ground, so an impact landing; it ends on water below deck height: the wake and the first splash.
    assert!(gone(&w, id));
}

/// spec: S-0319 claim 6 and Edge cases (quirks.bouncing_anim_deep_landing: a move ending 100 or more below the ground stays there; off: put at the ground height)
#[test]
fn claim_6_deep_landing_switch() {
    for deep in [true, false] {
        let r = rules();
        let mut w = world(&r);
        set_quirks(&mut w, |q| q.bouncing_anim_deep_landing = deep);
        let at = centre(mid(0, 0), 0);
        let id = make(&mut w, "ROCK", at);
        set(&mut w, id, [at.x, at.y, -50], ["0", "0", "-60"]);
        w.effects.clear();
        anims::turn(&mut w, id);
        assert!(gone(&w, id));
        let boom = anims_made(&w)
            .into_iter()
            .find(|a| a.0 == "Boom")
            .expect("the expiry animation");
        if deep {
            assert!(boom.1.z <= -100, "below the ground: {}", boom.1.z);
        } else {
            assert_eq!(boom.1.z, 0, "put on the ground");
        }
    }
}

/// spec: S-0319 claim 6 (a height from the ground up to 150 above it in a cell holding a building: a contact at the ground height; at 150 or more, none)
#[test]
fn claim_6_building_cell() {
    let r = rules();
    let mut w = world(&r);
    let b = mid(3, 3);
    testkit::spawn(&mut w, Kind::Building, "TOWER", 1, b, 0);
    let at = centre(b, 0);
    let id = make(&mut w, "DROP", at);
    set(&mut w, id, [at.x, at.y, 300], ["0", "0", "0"]);
    anims::turn(&mut w, id);
    assert!(!gone(&w, id), "298.6 is above the band");
    set(&mut w, id, [at.x, at.y, 120], ["0", "0", "0"]);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(gone(&w, id), "a building contact");
}

/// spec: S-0319 claim 6 (the cliff test: a contact in a cell 2 or more levels above the old cell that would put a descending animation higher than the flight height z₀ before the move is undone; the landing point is the restored flight position truncated; in a falling animation's first turn that is 10 leptons above the creation point) and claim 7 (the hits at that point)
#[test]
fn claim_6_cliff_undone_first_turn() {
    let r = rules();
    let mut w = world(&r);
    let low = mid(0, 0);
    let high = mid(1, 0);
    w.cells.grid.get_mut(high).unwrap().level = 2;
    let at = centre(low, 0);
    let id = make(&mut w, "ROCK", Pos::new(at.x + 127, at.y, 150));
    // First turn: the flight position is 160, the position in play 150.
    let m = mode(&w);
    w.anims
        .anims
        .get_mut(&id)
        .unwrap()
        .flight
        .as_mut()
        .unwrap()
        .v = ["30", "0", "-5"].map(|x| s(x).to_f32(m));
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(gone(&w, id));
    let boom = anims_made(&w)
        .into_iter()
        .find(|a| a.0 == "Boom")
        .expect("expiry");
    assert_eq!(
        boom.1,
        Pos::new(at.x + 127, at.y, 160),
        "the landing point: 10 above the creation point"
    );
}

/// spec: S-0319 claim 7 and Edge cases (quirks.bouncing_anim_water_impact_hits: an impact on water makes the bounce animation and hits before the splash; off: neither) and claim 8 (a falling animation's end on water: the wake at the landing point, then the first splash 3 leptons higher)
#[test]
fn claim_7_8_water() {
    for hits in [true, false] {
        let r = rules();
        let mut w = world(&r);
        set_quirks(&mut w, |q| q.bouncing_anim_water_impact_hits = hits);
        let c = mid(0, 0);
        set_land(&mut w, c, land::WATER);
        let at = centre(c, 0);
        let id = make(&mut w, "ROCK", at);
        set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
        w.effects.clear();
        anims::turn(&mut w, id);
        let names: Vec<(String, Pos)> = anims_made(&w);
        let mut want = vec![];
        if hits {
            want.push(("Bump".to_owned(), Pos::new(at.x, at.y, 5)));
        }
        want.push(("Wake".to_owned(), at));
        want.push(("Spl1".to_owned(), Pos::new(at.x, at.y, 3)));
        assert_eq!(names, want, "switch {hits}");
    }
}

/// spec: S-0319 claims 8 and 11 (a meteor animation on water: the last splash 3 higher, no wake; the meteor test reads the type at the landing) and claim 8's departure (an empty splash list: no splash)
#[test]
fn claim_8_meteor_on_water_and_empty_list() {
    let r = rules();
    let mut w = world(&r);
    let c = mid(0, 0);
    set_land(&mut w, c, land::WATER);
    let at = centre(c, 0);
    let id = make(&mut w, "STAR", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(gone(&w, id));
    assert_eq!(
        anims_made(&w),
        vec![("Spl3".to_owned(), Pos::new(at.x, at.y, 3))]
    );
    let mut r = rules();
    r.sim.general.splash_list = Some(vec![]);
    r.sim.general.wake = None;
    let mut w = world(&r);
    set_land(&mut w, c, land::WATER);
    let id = make(&mut w, "STAR", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(anims_made(&w).is_empty());
}

/// spec: S-0319 claim 8 and Edge cases (quirks.bouncing_anim_expiry_needs_animation: with no expiry animation no area damage; off: also when δ ≠ 0)
#[test]
fn claim_8_expiry_needs_animation_switch() {
    for needs in [true, false] {
        let mut r = rules();
        let t = r.sim.animations.get_mut("ROCK").unwrap();
        t.expire_anim = None;
        t.bounce_anim = None;
        t.damage_radius = Some(-1);
        let mut w = world(&r);
        set_quirks(&mut w, |q| q.bouncing_anim_expiry_needs_animation = needs);
        let c = mid(0, 0);
        let near = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
        let h0 = health(&w, near);
        let at = centre(c, 0);
        let id = make(&mut w, "ROCK", at);
        set(&mut w, id, [at.x + 60, at.y, 5], ["0", "0", "-10"]);
        anims::turn(&mut w, id);
        assert!(gone(&w, id));
        assert_eq!(health(&w, near) < h0, !needs, "switch {needs}");
    }
}

/// spec: S-0319 claim 7 and Edge cases (quirks.bouncing_anim_deck_impact_layer: an impact on a high-bridge deck hits the ground layer; off: the deck layer)
#[test]
fn claim_7_deck_impact_layer_switch() {
    for ground in [true, false] {
        let r = rules();
        let mut w = world(&r);
        set_quirks(&mut w, |q| q.bouncing_anim_deck_impact_layer = ground);
        let c = mid(0, 0);
        let under = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
        let i = w.cells.idx(c).unwrap();
        w.cells.deck[i] = true;
        let h0 = health(&w, under);
        let mut rules = (*w.rules_arc()).clone();
        rules
            .animations
            .iter_mut()
            .find(|a| a.name == "ROCK")
            .unwrap()
            .bounce
            .expire = None;
        w.rules = Some(std::sync::Arc::new(rules));
        let at = centre(c, 0);
        let id = make(&mut w, "ROCK", at);
        set(&mut w, id, [at.x, at.y, 420], ["0", "0", "-10"]);
        anims::turn(&mut w, id);
        assert!(gone(&w, id));
        assert_eq!(
            health(&w, under) < h0,
            ground,
            "the object under the deck, switch {ground}"
        );
    }
}

/// spec: S-0319 claim 9 and F4 (n = R(0, c) + R(0, c) animations of the spawn type at the landing point, both draws before the first creation; a meteor animation's dry end) and claim 13 (the expiry's draws, then the spawn draws, then each spawn's creation draws)
#[test]
fn claim_9_spawns() {
    let r = rules();
    let mut w = world(&r);
    let at = centre(mid(0, 0), 0);
    let id = make(&mut w, "STAR", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let n = probe.ranged(0, 2, half) + probe.ranged(0, 2, half);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert_eq!(
        w.rng, probe,
        "the two spawn draws, the plain spawns take none"
    );
    let made = anims_made(&w);
    assert_eq!(made.len(), n as usize);
    for a in made {
        assert_eq!(a, ("PEBBLE".to_owned(), Pos::new(at.x, at.y, 0)));
    }
}

/// spec: S-0319 claim 9 (a spawn count of 0 or less, or no spawn type: no draw)
#[test]
fn claim_9_no_spawn_count_no_draws() {
    let mut r = rules();
    r.sim.animations.get_mut("STAR").unwrap().spawn_count = Some(0);
    let mut w = world(&r);
    let at = centre(mid(0, 0), 0);
    let id = make(&mut w, "STAR", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    let probe = w.rng.clone();
    anims::turn(&mut w, id);
    assert_eq!(w.rng, probe);
}

fn overlay_at(w: &World, c: CellXY) -> (u8, u8) {
    let cell = w.cells.cell(c).unwrap();
    (cell.overlay, cell.overlay_data)
}

/// spec: S-0319 claim 10 and F5 (radius 1: the pairs with i of −1, 0, 1 offer the neighbours in step directions 7, 0 and 1; each germinable cell takes R(0, 3), receives the overlay that many places after the spawn overlay type, then takes R(0, 2) for its stage; the landing cell receives none) and claim 13 (that order)
#[test]
fn claim_10_resources_neighbours() {
    let r = resource_rules(1);
    let mut w = world(&r);
    let c = mid(0, 0);
    let at = centre(c, 0);
    let id = make(&mut w, "GROW", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let ov2 = w
        .rules()
        .overlays
        .iter()
        .position(|o| o.name == "OV2")
        .unwrap();
    let mut want = vec![];
    for dir in [7, 0, 1] {
        let k = probe.ranged(0, 3, half);
        let stage = probe.ranged(0, 2, half);
        let (dx, dy) = glasnost_sim::regions::STEPS[dir];
        want.push((
            CellXY::new(c.x + dx, c.y + dy),
            (ov2 as i32 + k) as u8,
            stage as u8,
        ));
    }
    anims::turn(&mut w, id);
    assert!(gone(&w, id));
    assert_eq!(w.rng, probe, "three cells' draws");
    for (cell, o, stage) in want {
        assert_eq!(overlay_at(&w, cell), (o, stage), "cell {cell:?}");
    }
    assert_eq!(
        overlay_at(&w, c).0,
        glasnost_map::NO_OVERLAY,
        "not the landing cell"
    );
}

/// spec: S-0319 Edge cases (quirks.anim_resource_spread_neighbours_only off: the pair (i, j) offers (X + i, Y + j), pairs with trunc(√(i² + j²)) ≤ R) and claim 10 (a type with no spawn overlay type: no resource and no draws)
#[test]
fn claim_10_resources_switch_and_none() {
    let r = resource_rules(1);
    let mut w = world(&r);
    set_quirks(&mut w, |q| q.anim_resource_spread_neighbours_only = false);
    let c = mid(0, 0);
    let at = centre(c, 0);
    let id = make(&mut w, "GROW", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    anims::turn(&mut w, id);
    // trunc(√2) = 1 ≤ 1: all nine cells of the square, the landing cell included.
    for i in -1..=1 {
        for j in -1..=1 {
            let cell = CellXY::new(c.x + i, c.y + j);
            assert_ne!(overlay_at(&w, cell).0, glasnost_map::NO_OVERLAY, "{i},{j}");
        }
    }
    let mut r = resource_rules(1);
    r.sim
        .animations
        .get_mut("GROW")
        .unwrap()
        .tiberium_spawn_type = None;
    let mut w = world(&r);
    let id = make(&mut w, "GROW", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    let probe = w.rng.clone();
    anims::turn(&mut w, id);
    assert_eq!(w.rng, probe, "no draws");
    assert_eq!(overlay_at(&w, mid(0, -1)).0, glasnost_map::NO_OVERLAY);
}

/// spec: S-0319 claim 10 and F5 (a large radius still offers only the eight neighbours, each once)
#[test]
fn claim_10_large_radius_neighbours() {
    let r = resource_rules(1_000_000_000);
    let mut w = world(&r);
    let c = mid(0, 0);
    let at = centre(c, 0);
    let id = make(&mut w, "GROW", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    for _ in 0..8 {
        probe.ranged(0, 3, half);
        probe.ranged(0, 2, half);
    }
    anims::turn(&mut w, id);
    assert_eq!(w.rng, probe, "eight cells' draws");
    for (dx, dy) in glasnost_sim::regions::STEPS {
        assert_ne!(
            overlay_at(&w, CellXY::new(c.x + dx, c.y + dy)).0,
            glasnost_map::NO_OVERLAY
        );
    }
}

/// spec: S-0319 claim 10 and F5 (the overlay draw counts places after the spawn overlay type; a place past the end of the overlay list is a departure: no overlay, both draws taken, and the cell stays germinable, so each further pair that offers it takes both draws again; with radius 1 each of the directions 7, 0 and 1 has three pairs)
#[test]
fn claim_10_place_past_the_overlay_list() {
    let mut r = resource_rules(1);
    let t = r.sim.animations.get_mut("GROW").unwrap();
    t.tiberium_spawn_type = Some("ov7".into());
    let mut w = world(&r);
    let last = w.rules().overlays.len() - 1;
    assert_eq!(
        w.rules().overlays[last].name,
        "OV7",
        "the spawn type is the list's last"
    );
    let c = mid(0, 0);
    let at = centre(c, 0);
    let id = make(&mut w, "GROW", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut want = vec![];
    let mut repeated = false;
    for dir in [7, 0, 1] {
        let (dx, dy) = glasnost_sim::regions::STEPS[dir];
        let cell = CellXY::new(c.x + dx, c.y + dy);
        let mut placed = None;
        // trunc(√(i² + j²)) ≤ 1 for j of −1, 0 and 1 whatever i: three pairs.
        for pair in 0..3 {
            let k = probe.ranged(0, 3, half);
            let stage = probe.ranged(0, 2, half);
            if k == 0 {
                placed = Some((last as u8, stage as u8));
                break;
            }
            repeated |= pair > 0;
        }
        want.push((cell, placed));
    }
    assert!(
        repeated,
        "the seed offers a cell again after a place past the list"
    );
    anims::turn(&mut w, id);
    assert!(gone(&w, id));
    assert_eq!(
        w.rng, probe,
        "each pair's two draws until the cell receives an overlay"
    );
    for (cell, placed) in want {
        match placed {
            Some(p) => assert_eq!(overlay_at(&w, cell), p, "{cell:?}"),
            None => assert_eq!(overlay_at(&w, cell).0, glasnost_map::NO_OVERLAY, "{cell:?}"),
        }
    }
}

/// spec: S-0319 claim 11 (the last pass with no follow-up type ends the animation by S-0109 claim 16 with none of claim 8's effects)
#[test]
fn claim_11_pass_end_without_landing() {
    let mut r = rules();
    let t = r.sim.animations.get_mut("ROCK").unwrap();
    t.frame_count = Some(3);
    let mut w = world(&r);
    let id = make(&mut w, "ROCK", centre(mid(0, 0), 80_000));
    w.effects.clear();
    testkit::run(&mut w, 10);
    assert!(gone(&w, id));
    assert!(anims_made(&w).is_empty(), "no landing effects");
}

/// spec: S-0319 claim 12 (a trailer on ticks divisible by s, at the position in play after the motion step, before the stage update; a non-bouncing animation too; a separation of 0 acts as 1, a departure; none in a landing turn)
#[test]
fn claim_12_trailer() {
    let mut r = rules();
    let t = r.sim.animations.get_mut("DROP").unwrap();
    t.trailer_anim = Some("Trail".into());
    t.trailer_separation = Some(2);
    let mut p = long();
    p.trailer_anim = Some("Trail".into());
    add(&mut r, "STILL", p);
    let mut w = world(&r);
    let at = centre(mid(0, 0), 50_000);
    let id = make(&mut w, "DROP", at);
    w.tick = 10;
    w.effects.clear();
    anims::turn(&mut w, id);
    let pos = get(&w, id).pos.unwrap();
    assert_eq!(anims_made(&w), vec![("Trail".to_owned(), pos)]);
    w.tick = 11;
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(anims_made(&w).is_empty(), "11 is not divisible by 2");
    let still = make(&mut w, "STILL", at);
    w.tick = 13;
    w.effects.clear();
    anims::turn(&mut w, still);
    assert_eq!(
        anims_made(&w),
        vec![("Trail".to_owned(), at)],
        "separation 0 acts as 1"
    );
    // No trailer in a landing turn.
    w.tick = 12;
    let c = centre(mid(3, 3), 0);
    set(&mut w, id, [c.x, c.y, 5], ["0", "0", "-10"]);
    w.effects.clear();
    anims::turn(&mut w, id);
    assert!(gone(&w, id));
    assert!(anims_made(&w).iter().all(|a| a.0 != "Trail"));
}

/// spec: S-0319 claim 14 (a span of 0 is a departure: the remainder is 0, so every animation of the type has the same speed: W_xy = trunc(2 × 0.3) = 0 gives v = −M; W_z = trunc(3.5 − 4 + 1) = 0 gives v_z = Z_min)
#[test]
fn claim_14_zero_spans() {
    let mut r = rules();
    let t = r.sim.animations.get_mut("DROP").unwrap();
    t.max_xy_vel = dec("0.3");
    t.min_z_vel = dec("4");
    let mut w = world(&r);
    let m = mode(&w);
    let b = &w.rules().animation("DROP").unwrap().bounce;
    assert_eq!((b.wxy, b.wz), (0, 0));
    for _ in 0..4 {
        let id = make(&mut w, "DROP", centre(mid(0, 0), 0));
        let v = flight(&w, id).v;
        let mm = Decimal("0.3".into()).value(m).negated().to_f32(m);
        assert_eq!(v, [mm, mm, s("4").to_f32(m)]);
    }
}

/// spec: S-0319 State and ordering (all flight state is saved: the state hash survives a round trip mid-flight, and the flight goes on the same)
#[test]
fn flight_round_trip() {
    let r = rules();
    let mut w = world(&r);
    make(&mut w, "STAR", centre(mid(0, 0), 2000));
    make(&mut w, "DROP", centre(mid(1, 1), 2000));
    testkit::run(&mut w, 5);
    round_trip(&w);
    let bytes = save::save(&w);
    let mut back = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    testkit::run(&mut w, 20);
    testkit::run(&mut back, 20);
    assert_eq!(save::state_hash(&back), save::state_hash(&w));
}

/// Puts an animation's flight state in play from decimal strings (the
/// flight position may be fractional); the position in play is the flight
/// position truncated.
fn set_exact(w: &mut World, id: ObjectId, p: [&str; 3], v: [&str; 3]) {
    let m = mode(w);
    let a = w.anims.anims.get_mut(&id).unwrap();
    let f = Flight {
        p: p.map(|x| s(x).to_f32(m)),
        v: v.map(|x| s(x).to_f32(m)),
    };
    let [x, y, z] = f.p.map(|c| c.to_f64().to_i32_common());
    a.flight = Some(f);
    a.pos = Some(Pos::new(x, y, z));
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
/// for y and z, each component stored as a single (the helper of
/// `claims_s0317.rs`).
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

/// F6 as the spec writes it: w = Mᵀ · D · v, w′ = Z · (ε w), v″ = D · M · w′.
fn f6(m3: &[[F32; 3]; 3], v: [F32; 3], eps: F32, m: Rounding) -> [F32; 3] {
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

/// [`rules`] with SOFT: a falling type of elasticity 30% (0.3 read from a
/// percentage, a double that single precision does not hold).
fn soft_rules() -> Ruleset {
    let mut r = rules();
    add(
        &mut r,
        "SOFT",
        AnimationType {
            bouncer: Some(true),
            elasticity: dec("30%"),
            ..long()
        },
    );
    r
}

fn e03(m: Rounding) -> F64 {
    Decimal("30%".into()).value(m)
}

/// spec: S-0319 claim 11 and Edge cases (a bouncing animation whose placement point is (0, 0, 0) is not placed: its creation draws are taken, it takes no turn; design rule) and State and ordering (placed, except at (0, 0, 0))
#[test]
fn claim_11_not_placed_at_the_origin() {
    let r = rules();
    let mut w = world(&r);
    let mut other = world(&r);
    let id = anims::spawn(&mut w, "ROCK", Some(Pos::new(0, 0, 0)), Spawn::default())
        .id
        .expect("a number");
    make(&mut other, "ROCK", Pos::new(1, 0, 0));
    assert_eq!(w.rng, other.rng, "the same creation draws");
    assert!(gone(&w, id), "not in play");
    testkit::run(&mut w, 3);
    assert!(gone(&w, id), "it never takes a turn");
    // A plain animation at the origin is placed as before.
    let plain = make(&mut w, "PEBBLE", Pos::new(0, 0, 0));
    assert!(!gone(&w, plain));
}

/// spec: S-0319 claim 11 (no placement test: an animation is placed at its placement point on the map or off it; a meteor animation's start position off the legal map is kept)
#[test]
fn claim_11_meteor_start_off_the_map() {
    let r = rules();
    let mut w = world(&r);
    let c = Pos::new(-5000, -5000, 0);
    let id = make(&mut w, "STAR", c);
    let a = get(&w, id);
    let p = a.pos.unwrap();
    assert_ne!(p, c, "the start position, not the creation point");
    assert!(w.cells.cell(p.cell()).is_none(), "off the legal map");
    anims::turn(&mut w, id);
    assert!(!gone(&w, id), "it keeps its turns off the map");
}

/// spec: S-0319 claim 6 (the building cell: the first building listed in the ground layer counts unless an inactive laser-fence section or a vehicle-like building; with none listed, one of the three wall overlays; no other overlay) and S-0317 claim 19 (every section counts as inactive in our engine)
#[test]
fn claim_6_kinds_of_building_cell() {
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
    // Whether a move to 118.6 above the ground of the cell is a landing.
    let contact = |set_up: &dyn Fn(&mut World, CellXY)| {
        let mut w = world(&r);
        let c = mid(0, 0);
        set_up(&mut w, c);
        let at = centre(c, 0);
        let id = make(&mut w, "DROP", at);
        set(&mut w, id, [at.x, at.y, 120], ["0", "0", "0"]);
        glasnost_sim::bounce::step_for_test(&w, id).2.is_some()
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
}

/// spec: S-0319 claim 6 and Constants (the cliff test: u_z below −0.0002, compared in double precision, undoes a contact that puts the animation higher than z₀; any other u_z, 0 included, one that lifts it more than 1 lepton above z₀ + u_z; z₀ is the flight position's height, not the position in play's)
#[test]
fn claim_6_cliff_line_and_flight_height() {
    let r = rules();
    // (flight height, position height, u, undone)
    let cases = [
        // u exactly 0: a lift to 208, more than 1 above 206, is undone; exactly 1 above 207 stands.
        ("206", None, "0", true),
        ("207", None, "0", false),
        // −0.0001 is not below the line: 208 ≤ 207.5 − 0.0001 + 1, so the move stands.
        ("207.5", None, "-0.0001", false),
        // −0.001 is below it: 208 > 207.5, undone.
        ("207.5", None, "-0.001", true),
        // A rising animation whose position in play stands far below its flight position: the
        // flight height decides, 208 ≤ 205 + 3 + 1, and it climbs.
        ("205", Some(150), "3", false),
    ];
    for (z0, posz, u, undone) in cases {
        let mut w = world(&r);
        let low = mid(0, 0);
        let high = mid(1, 0);
        w.cells.grid.get_mut(high).unwrap().level = 2;
        let at = centre(low, 0);
        let id = make(&mut w, "DROP", at);
        let x = format!("{}", at.x + 125);
        let y = format!("{}", at.y);
        set_exact(&mut w, id, [&x, &y, z0], ["10", "0", u]);
        if let Some(z) = posz {
            w.anims.anims.get_mut(&id).unwrap().pos.as_mut().unwrap().z = z;
        }
        let before = flight(&w, id);
        let (f, p, landed) = glasnost_sim::bounce::step_for_test(&w, id);
        assert!(landed.is_some(), "a contact either way");
        assert_eq!(f.p == before.p, undone, "z₀ {z0}, u {u}");
        assert_eq!(p.cell() == low, undone, "z₀ {z0}, u {u}");
    }
}

/// spec: S-0319 claim 7 and F6 (a contact reflects the velocity in the plane of S-0106 C4's transform for the new cell's slope shape and scales it by ε, e narrowed once to single precision, each step stored in single precision; shapes 0 and 17 to 20 give the flat rule) and Edge cases (quirks.bouncing_anim_rebound_faults: a deck contact over sloped ground rebounds off the slope; off, by the flat rule)
#[test]
fn claim_7_f6_rebound_on_a_slope() {
    let r = soft_rules();
    let ramps = glasnost_sim::gas::ramps();
    for shape in [0u8, 1, 6, 14, 18] {
        let mut w = world(&r);
        let m = mode(&w);
        let c = mid(0, 0);
        w.cells.grid.get_mut(c).unwrap().slope = shape;
        let at = centre(c, 0);
        let f = w.surface_at(at.x, at.y);
        let id = make(&mut w, "SOFT", at);
        let z = format!("{}", f + 5);
        let (x, y) = (format!("{}", at.x), format!("{}", at.y));
        set_exact(&mut w, id, [&x, &y, &z], ["2", "-3", "-10"]);
        let (fl, _, landed) = glasnost_sim::bounce::step_for_test(&w, id);
        assert!(landed.is_some(), "shape {shape}: a ground contact");
        let v = [
            s("2").to_f32(m),
            s("-3").to_f32(m),
            single(s("-10").to_f32(m).to_f64().sub(g(), m), m),
        ];
        let eps = e03(m).to_f32(m);
        let want = f6(&ramps[usize::from(shape)], v, eps, m);
        assert!(same(fl.v, want), "shape {shape}: {:?} {want:?}", fl.v);
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
            assert_eq!(fl.v, flat, "shape {shape}: the flat rule");
        } else {
            assert_ne!(fl.v, flat, "shape {shape}: a tilted plane");
        }
    }
    // A deck contact over a sloped cell.
    for faults in [true, false] {
        let mut w = world(&r);
        let m = mode(&w);
        set_quirks(&mut w, |q| q.bouncing_anim_rebound_faults = faults);
        let c = mid(0, 0);
        w.cells.grid.get_mut(c).unwrap().slope = 1;
        let i = w.cells.idx(c).unwrap();
        w.cells.deck[i] = true;
        let at = centre(c, 0);
        let deck = w.surface_at(at.x, at.y) + 416;
        let id = make(&mut w, "SOFT", at);
        let z = format!("{}", deck + 4);
        let (x, y) = (format!("{}", at.x), format!("{}", at.y));
        set_exact(&mut w, id, [&x, &y, &z], ["0", "0", "-10"]);
        let (fl, _, landed) = glasnost_sim::bounce::step_for_test(&w, id);
        assert!(landed.is_some());
        assert_eq!(fl.p[2], d(i64::from(deck), m).to_f32(m), "on the deck");
        let v = [
            F32(0),
            F32(0),
            single(s("-10").to_f32(m).to_f64().sub(g(), m), m),
        ];
        let shape = if faults { 1 } else { 0 };
        let want = f6(&ramps[shape], v, e03(m).to_f32(m), m);
        assert!(
            same(fl.v, want),
            "deck rebound, faults {faults}: {:?} {want:?}",
            fl.v
        );
    }
}

/// A SOFT animation in a fresh world whose old cell `mid(0, 0)` is at level
/// 0 and whose new cell (dx, dy) from it and the cells `highs` (offsets
/// from the old cell) are at level 2, crossing into the new cell from near
/// the matching corner, descending: the velocity the undone move leaves and
/// the velocity at the start of the turn.
fn cliff_case(dx: i32, dy: i32, highs: &[(i32, i32)], faults: bool) -> ([F32; 3], [F32; 3]) {
    let r = soft_rules();
    let mut w = world(&r);
    set_quirks(&mut w, |q| q.bouncing_anim_rebound_faults = faults);
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
    let ux = match dx {
        0 => "3",
        1 => "12",
        _ => "-12",
    };
    let uy = match dy {
        0 => "7",
        1 => "10",
        _ => "-10",
    };
    let id = make(&mut w, "SOFT", base);
    let (xs, ys) = (format!("{x}"), format!("{y}"));
    set_exact(&mut w, id, [&xs, &ys, "150"], [ux, uy, "-5"]);
    let before = flight(&w, id);
    let (f, _, landed) = glasnost_sim::bounce::step_for_test(&w, id);
    assert!(landed.is_some(), "({dx}, {dy}) {highs:?}: a landing");
    assert_eq!(f.p, before.p, "({dx}, {dy}) {highs:?}: undone");
    (f.v, before.v)
}

/// spec: S-0319 claim 7 and F7 (the cliff turn of the velocity u at the start of the turn, every face exchanging or reversing the horizontal components; each component multiplied by e as imported, in double precision, not narrowed first, and stored) and Edge cases (quirks.bouncing_anim_rebound_faults: the north-west turn (c, uₓ, u_y) when of (x − 1, y) and (x, y − 1) only (x, y − 1) is high; off, across the second axis)
#[test]
fn claim_7_f7_cliff_turns() {
    let m = mode(&world(&rules()));
    let e = e03(m);
    assert_ne!(
        e,
        e.to_f32(m).to_f64(),
        "30% is not held in single precision"
    );
    let scale = |a: [F32; 3]| a.map(|c| c.to_f64().mul(e, m).to_f32(m));
    let n = |c: F32| c.to_f64().negated().to_f32(m);
    for faults in [true, false] {
        // Straight steps: d = 0 across y, d = 2 across x.
        let (v, u) = cliff_case(0, -1, &[], faults);
        assert_eq!(v, scale([u[0], n(u[1]), u[2]]), "d 0");
        let (v, u) = cliff_case(1, 0, &[], faults);
        assert_eq!(v, scale([n(u[0]), u[1], u[2]]), "d 2");
        // d = 7 with only (x − 1, y) high: across x.
        let (v, u) = cliff_case(-1, -1, &[(-1, 0)], faults);
        assert_eq!(v, scale([n(u[0]), u[1], u[2]]), "d 7, (x − 1, y)");
        // d = 7 with only (x, y − 1) high: the north-west turn, or across y.
        let (v, u) = cliff_case(-1, -1, &[(0, -1)], faults);
        let want = if faults {
            scale([s("2.3771235942840576").to_f32(m), u[0], u[1]])
        } else {
            scale([u[0], n(u[1]), u[2]])
        };
        assert_eq!(v, want, "d 7, (x, y − 1), faults {faults}");
    }
}

/// spec: S-0319 claim 7 (after an undone descending move between flat cells, the height counts from the lower ground: with 0 ≤ e ≤ 1 and a face turn the landing is an impact landing) and F7 (after the north-west turn the vertical speed is the scaled old second-axis speed)
#[test]
fn claim_7_measure_after_a_cliff_turn() {
    for faults in [true, false] {
        let r = soft_rules();
        let mut w = world(&r);
        set_quirks(&mut w, |q| q.bouncing_anim_rebound_faults = faults);
        for (ox, oy) in [(-1, -1), (0, -1)] {
            w.cells.grid.get_mut(mid(ox, oy)).unwrap().level = 2;
        }
        let base = Pos::centre(mid(0, 0), 0);
        let id = make(&mut w, "SOFT", base);
        let (x, y) = (format!("{}", base.x - 123), format!("{}", base.y - 123));
        set_exact(&mut w, id, [&x, &y, "150"], ["-12", "-10", "-5"]);
        let (_, p, landed) = glasnost_sim::bounce::step_for_test(&w, id);
        assert_eq!(p.z, 150, "the restored height");
        assert_eq!(landed, Some(true), "an impact landing, faults {faults}");
    }
}

/// spec: S-0319 claim 7 and Edge cases (quirks.bouncing_anim_impact_hits_walk: the hits follow the cell's listing as it changes, so a hit that takes its own object out of the ground layer ends the hits and no object listed after it is hit; off: every object listed when the hits begin receives its hit)
#[test]
fn claim_7_impact_hits_walk_switch() {
    for walk in [true, false] {
        let r = rules();
        let mut w = world(&r);
        set_quirks(&mut w, |q| q.bouncing_anim_impact_hits_walk = walk);
        let c = mid(0, 0);
        let tower = testkit::spawn(&mut w, Kind::Building, "TOWER", 1, c, 0);
        let buggy = testkit::spawn(&mut w, Kind::Vehicle, "BUGGY", 1, c, 0);
        let list = w.cells.list(c, glasnost_sim::world::Layer::Ground).to_vec();
        let pos = |id| list.iter().position(|&x| x == id).unwrap();
        assert!(pos(buggy) < pos(tower), "the vehicle is listed first");
        w.obj_mut(buggy).unwrap().health = 1;
        let h0 = health(&w, tower);
        let mut rules = (*w.rules_arc()).clone();
        rules
            .animations
            .iter_mut()
            .find(|a| a.name == "ROCK")
            .unwrap()
            .bounce
            .expire = None;
        w.rules = Some(std::sync::Arc::new(rules));
        let at = centre(c, 0);
        let id = make(&mut w, "ROCK", at);
        set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
        anims::turn(&mut w, id);
        assert!(gone(&w, id));
        assert!(w.obj(buggy).is_none(), "the vehicle is destroyed");
        assert_eq!(
            health(&w, tower) < h0,
            !walk,
            "the building after it, walk {walk}"
        );
    }
}

/// spec: S-0319 claim 10 and F5 (a germinable neighbour that holds a terrain object receives no overlay; both draws are still taken, and it stays germinable, so each further pair that offers it takes both draws again)
#[test]
fn claim_10_terrain_object_neighbour() {
    use glasnost_rules::sim::TerrainObjectType;
    let mut r = resource_rules(1);
    register(&mut r, "terrain_objects", "TREE");
    r.sim.terrain_objects.insert(
        "TREE".into(),
        TerrainObjectType {
            strength: Some(100),
            ..Default::default()
        },
    );
    let mut w = world(&r);
    let c = mid(0, 0);
    let (dx, dy) = glasnost_sim::regions::STEPS[0];
    let treed = CellXY::new(c.x + dx, c.y + dy);
    let ty = w.rules().index_of(Kind::Terrain, "TREE").unwrap();
    let tree = glasnost_sim::objects::create(&mut w, Kind::Terrain, ty, None);
    glasnost_sim::objects::place(&mut w, tree, Pos::centre(treed, 0), 0);
    assert!(glasnost_sim::econ::resource::germinable(&w, treed));
    let at = centre(c, 0);
    let id = make(&mut w, "GROW", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    // Direction 7 once, direction 0 (the tree's) three times, direction 1 once.
    for n in [1, 3, 1] {
        for _ in 0..n {
            probe.ranged(0, 3, half);
            probe.ranged(0, 2, half);
        }
    }
    anims::turn(&mut w, id);
    assert_eq!(
        w.rng, probe,
        "the tree's cell takes both draws at each of its pairs"
    );
    assert_eq!(overlay_at(&w, treed).0, glasnost_map::NO_OVERLAY);
}

/// spec: S-0319 claim 10 and S-0401 claim 10 (quirks.spread_step_lost_on_terrain_object off: a cell holding a terrain object is not germinable, so the offer takes no draws there)
#[test]
fn claim_10_terrain_object_neighbour_switch_off() {
    use glasnost_rules::sim::TerrainObjectType;
    let mut r = resource_rules(1);
    r.globals.quirks.spread_step_lost_on_terrain_object = false;
    register(&mut r, "terrain_objects", "TREE");
    r.sim.terrain_objects.insert(
        "TREE".into(),
        TerrainObjectType {
            strength: Some(100),
            ..Default::default()
        },
    );
    let mut w = world(&r);
    let c = mid(0, 0);
    let (dx, dy) = glasnost_sim::regions::STEPS[0];
    let treed = CellXY::new(c.x + dx, c.y + dy);
    let ty = w.rules().index_of(Kind::Terrain, "TREE").unwrap();
    let tree = glasnost_sim::objects::create(&mut w, Kind::Terrain, ty, None);
    glasnost_sim::objects::place(&mut w, tree, Pos::centre(treed, 0), 0);
    assert!(!glasnost_sim::econ::resource::germinable(&w, treed));
    let at = centre(c, 0);
    let id = make(&mut w, "GROW", at);
    set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
    let mut probe = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    // Direction 7 once and direction 1 once; the tree's three pairs take none.
    for _ in 0..2 {
        probe.ranged(0, 3, half);
        probe.ranged(0, 2, half);
    }
    anims::turn(&mut w, id);
    assert_eq!(w.rng, probe, "the tree's cell takes no draws");
    assert_eq!(overlay_at(&w, treed).0, glasnost_map::NO_OVERLAY);
}

/// spec: S-0319 claim 10 and Edge cases (quirks.anim_resource_no_growth_entry: a new resource cell joins no growth list and takes no growth-list draw; off: it is offered to its tiberium's growth list with S-0401 claim 7's raw draw, after the placement and before the stage draw)
#[test]
fn claim_10_growth_entry_switch() {
    use glasnost_rules::sim_econ::TiberiumType;
    for none in [true, false] {
        let mut r = resource_rules(0);
        register(&mut r, "tiberiums", "ORE");
        r.sim.tiberiums.insert(
            "ORE".into(),
            TiberiumType {
                value: Some(10),
                image_run: Some([0, 4, 4]),
                ..Default::default()
            },
        );
        let mut w = world(&r);
        set_quirks(&mut w, |q| q.anim_resource_no_growth_entry = none);
        let c = mid(0, 0);
        let at = centre(c, 0);
        let id = make(&mut w, "GROW", at);
        set(&mut w, id, [at.x, at.y, 5], ["0", "0", "-10"]);
        let mut probe = w.rng.clone();
        let half = w.rules().quirks.ranged_draw_half_span;
        let entries = |w: &World| {
            w.econ
                .resources
                .tiberiums
                .first()
                .map_or(0, |t| t.growth.count)
        };
        let e0 = entries(&w);
        // Radius 0: the one pair (0, 0) offers the neighbour in step direction 0.
        probe.ranged(0, 3, half);
        if !none {
            probe.raw();
        }
        let stage = probe.ranged(0, 2, half);
        anims::turn(&mut w, id);
        assert_eq!(w.rng, probe, "switch {none}");
        let (dx, dy) = glasnost_sim::regions::STEPS[0];
        let cell = CellXY::new(c.x + dx, c.y + dy);
        assert_eq!(
            overlay_at(&w, cell).1,
            stage as u8,
            "the stage draw's stage"
        );
        assert_eq!(
            entries(&w) - e0,
            i64::from(!none),
            "growth entries, switch {none}"
        );
    }
}
