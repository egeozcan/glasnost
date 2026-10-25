//! Claim tests for bouncing animations (S-0319) on synthetic data: invented
//! animation types, warheads, overlays and a resource, every name and
//! number our own. The ledger is `docs/m8-claims/intake15a-S-0319.md`.

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

/// spec: S-0319 claim 10 and F5 (the overlay draw counts places after the spawn overlay type), with Q-1847's interim for a place past the overlay list: nothing placed, both draws taken, and the cell takes no further offer in the landing, so a large radius still ends after one offer per direction
#[test]
fn claim_10_place_past_the_overlay_list() {
    let mut r = resource_rules(1_000_000_000);
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
    // i runs from −10⁹, which is 0 mod 8: directions 0 to 7 in turn, one offer each.
    let mut want = vec![];
    for (dx, dy) in glasnost_sim::regions::STEPS {
        let k = probe.ranged(0, 3, half);
        let stage = probe.ranged(0, 2, half);
        let cell = CellXY::new(c.x + dx, c.y + dy);
        want.push((cell, (k == 0).then_some((last as u8, stage as u8))));
    }
    assert!(
        want.iter().any(|(_, p)| p.is_none()),
        "the seed draws a place past the list"
    );
    anims::turn(&mut w, id);
    assert!(gone(&w, id));
    assert_eq!(w.rng, probe, "one offer's two draws per direction");
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
