//! Claim tests for gas clouds (S-0613) on synthetic data: invented particle,
//! particle-system, warhead and animation types, every number our own. The
//! ledger is `docs/m7-claims/S-0613.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{
    AnimationType, Decimal, ParticleSystemType, ParticleType, UnitType, WarheadType,
};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::gas::{self, Particle};
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, Pos, World};
use glasnost_sim::{save, testkit};

const SIZE: i32 = 20;
/// The invented virus-death animation's middle frame μ and frame count.
const MU: i32 = 2;
const FRAMES: i32 = 6;
/// The invented particle count of its middle event.
const COUNT: i32 = 3;

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
/// - FOG, a gas particle (lifetime 30000, damage interval 5, damage 7,
///   warhead VENOM, stage advance 4, end stage 100);
/// - WISP, a gas particle with no damage that FOG's successor field does
///   not name (tests set it);
/// - SMOG, a smoke particle;
/// - the shared gas system GasCloudSys (gas, holding FOG), CLOUD (gas,
///   holding FOG, lifetime −1), SOOT (smoke, holding SMOG), BARE (gas, no
///   held type);
/// - VENOM, a warhead of infantry-death kind 8; MIST, a warhead naming
///   CLOUD; SMOKER, one naming SOOT; EMPTY, one naming BARE;
/// - PLAGUE, the virus-death animation (μ = 2, 6 frames, 3 FOG particles);
/// - DRONE, a WALKER marked not human.
fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let gasp = |damage: i32| ParticleType {
        behaves_like: Some("gas".into()),
        max_ec: Some(30000),
        max_dc: Some(5),
        damage: Some(damage),
        warhead: Some("VENOM".into()),
        state_ai_advance: Some(4),
        end_state_ai: Some(100),
        ..Default::default()
    };
    for (n, p) in [
        ("FOG", gasp(7)),
        ("WISP", gasp(0)),
        (
            "SMOG",
            ParticleType {
                behaves_like: Some("smoke".into()),
                max_ec: Some(40),
                ..Default::default()
            },
        ),
    ] {
        register(&mut r, "particles", n);
        r.sim.particles.insert(n.into(), p);
    }
    let sys = |kind: &str, holds: Option<&str>| ParticleSystemType {
        behaves_like: Some(kind.into()),
        holds_what: holds.map(Into::into),
        lifetime: None,
    };
    for (n, s) in [
        ("GasCloudSys", sys("gas", Some("FOG"))),
        ("CLOUD", sys("gas", Some("FOG"))),
        ("SOOT", sys("smoke", Some("SMOG"))),
        ("BARE", sys("gas", None)),
    ] {
        register(&mut r, "particle_systems", n);
        r.sim.particle_systems.insert(n.into(), s);
    }
    let wh = |inf: i32, particle: Option<&str>| WarheadType {
        verses: verses(),
        inf_death: Some(inf),
        particle: particle.map(Into::into),
        ..Default::default()
    };
    for (n, h) in [
        ("VENOM", wh(8, None)),
        ("MIST", wh(0, Some("CLOUD"))),
        ("SMOKER", wh(0, Some("SOOT"))),
        ("EMPTY", wh(0, Some("BARE"))),
    ] {
        register(&mut r, "warheads", n);
        r.sim.warheads.insert(n.into(), h);
    }
    register(&mut r, "animations", "PLAGUE");
    r.sim.animations.insert(
        "PLAGUE".into(),
        AnimationType {
            middle_frame: Some(MU),
            frame_count: Some(FRAMES),
            spawns_particle: Some("FOG".into()),
            num_particles: Some(COUNT),
            ..Default::default()
        },
    );
    r.sim.general.infantry_virus = Some("PLAGUE".into());
    let drone = UnitType {
        not_human: Some(true),
        ..r.sim.infantry["WALKER"].clone()
    };
    register(&mut r, "infantry", "DRONE");
    r.sim.infantry.insert("DRONE".into(), drone);
    r
}

fn game(r: &Ruleset) -> World {
    testkit::game_with(r, SIZE, 7)
}

fn warhead(w: &World, name: &str) -> u32 {
    w.rules()
        .combat
        .warheads
        .iter()
        .position(|h| h.name == name)
        .unwrap_or_else(|| panic!("no warhead {name}")) as u32
}

fn ptype(w: &World, name: &str) -> u32 {
    w.rules()
        .gas
        .particles
        .iter()
        .position(|p| p.name == name)
        .unwrap_or_else(|| panic!("no particle {name}")) as u32
}

fn shared(w: &World) -> ObjectId {
    w.gas.shared.expect("a shared gas system")
}

fn list(w: &World, sys: ObjectId) -> Vec<Particle> {
    w.gas.systems[&sys].particles.clone()
}

/// Kills `id` with a hit of the VENOM warhead (infantry-death kind 8).
fn venom(w: &mut World, id: ObjectId, attacker: Option<ObjectId>, house: Option<HouseId>) {
    let h = warhead(w, "VENOM");
    damage::apply(
        w,
        id,
        Hit {
            amount: 10_000,
            warhead: Some(h),
            distance: 0,
            attacker,
            house,
            ignore_defences: false,
            no_escape: false,
        },
    );
}

/// Adds one particle of `name` to the shared system at `at` (as a middle
/// event does) and returns its index in the list.
fn puff(w: &mut World, name: &str, at: Pos) -> usize {
    let ti = ptype(w, name);
    let draw = w.rules().gas.particles[ti as usize].draw;
    gas::middle_event_particle(w, draw, Some((at.x, at.y, at.z)));
    list(w, shared(w)).len() - 1
}

/// Sets particle `i` of the shared system with `f`.
fn edit(w: &mut World, i: usize, f: impl FnOnce(&mut Particle)) {
    let s = shared(w);
    f(&mut w.gas.systems.get_mut(&s).unwrap().particles[i]);
}

fn centre(dx: i32, dy: i32) -> Pos {
    Pos::centre(testkit::mid(SIZE, dx, dy), 0)
}

/// Brings the world to an even tick.
fn to_even(w: &mut World) {
    if w.tick % 2 != 0 {
        testkit::run(w, 1);
    }
}

/// spec: S-0613 claim 1 (the shared gas system exists from game start, with no owner, empty, in the update order, and stays when empty with lifetime −1)
#[test]
fn claim_1_shared_system_at_game_start() {
    let mut w = game(&rules());
    let s = shared(&w);
    let sys = &w.gas.systems[&s];
    assert_eq!(sys.owner, None);
    assert!(sys.particles.is_empty());
    assert_eq!(sys.lifetime, -1);
    // Q-1240: it is first in the update order.
    assert_eq!(w.update.order.first(), Some(&s));
    testkit::run(&mut w, 30);
    assert_eq!(w.gas.shared, Some(s), "never removed for being empty");
}

/// spec: S-0613 claim 1 and claim 6 (a positive system lifetime marks the shared system to end after that many turns; a marked empty system is removed)
#[test]
fn claim_1_finite_shared_lifetime() {
    let mut r = rules();
    r.sim
        .particle_systems
        .get_mut("GasCloudSys")
        .unwrap()
        .lifetime = Some(3);
    let mut w = game(&r);
    let s = shared(&w);
    testkit::run(&mut w, 2);
    assert!(w.gas.systems.contains_key(&s));
    testkit::run(&mut w, 1);
    assert!(!w.gas.systems.contains_key(&s));
    assert!(!w.update.contains(s));
    assert_eq!(w.gas.shared, None);
}

/// spec: S-0613 claim 1 (a shared system type without a behaviour kind neither updates nor moves the particles delivered into it)
#[test]
fn claim_1_kindless_shared_system_is_inert() {
    let mut r = rules();
    r.sim
        .particle_systems
        .insert("GasCloudSys".into(), ParticleSystemType::default());
    let mut w = game(&r);
    let i = puff(&mut w, "FOG", centre(0, 0));
    let before = list(&w, shared(&w))[i].clone();
    testkit::run(&mut w, 20);
    assert_eq!(list(&w, shared(&w))[i], before);
}

/// spec: S-0613 claim 2 and S-0107 claim 17 row 8 (a human infantry unit's virus death creates the animation at its position; its middle event at a + k = μ puts its particle count of particles into the shared system at that position; it ends after E advances)
#[test]
fn claim_2_virus_animation_delivers_its_count_later() {
    let mut w = game(&rules());
    let c = testkit::mid(SIZE, 0, 0);
    let victim = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, c, 0);
    let killer = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 0, testkit::mid(SIZE, 3, 0), 0);
    let at = w.obj(victim).unwrap().pos;
    venom(&mut w, victim, Some(killer), None);
    assert!(w.obj(victim).is_none_or(|o| !o.on_map), "removed at once");
    assert_eq!(w.gas.anims.len(), 1);
    let (&anim, v) = w.gas.anims.iter().next().unwrap();
    assert_eq!(v.pos, at);
    assert_eq!(v.owner, Some(HouseId(0)), "the attacker's house");
    assert!(list(&w, shared(&w)).is_empty(), "no particle before the middle event");
    // First update: no advance; then one advance per tick (d = 1): k = μ after μ + 1 updates.
    testkit::run(&mut w, MU as i32);
    assert!(list(&w, shared(&w)).is_empty());
    testkit::run(&mut w, 1);
    let l = list(&w, shared(&w));
    assert_eq!(l.len(), COUNT as usize);
    assert!(l.iter().all(|p| (p.pos.x, p.pos.y) == (at.x, at.y)));
    testkit::run(&mut w, FRAMES - MU - 1);
    assert!(w.gas.anims.contains_key(&anim));
    testkit::run(&mut w, 1);
    assert!(!w.gas.anims.contains_key(&anim), "removed at k = E");
    assert!(!w.update.contains(anim));
}

/// spec: S-0613 claim 2 (without an attacker object the animation takes the hit's attacking house)
#[test]
fn claim_2_owner_without_attacker() {
    let mut w = game(&rules());
    let victim = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, testkit::mid(SIZE, 0, 0), 0);
    venom(&mut w, victim, None, Some(HouseId(1)));
    assert_eq!(w.gas.anims.values().next().unwrap().owner, Some(HouseId(1)));
}

/// spec: S-0613 claim 3 and S-0107 claim 17 row 4 (a not-human type's virus death: one particle at once, the animation removed in the same step)
#[test]
fn claim_3_not_human_leaves_one_cloud() {
    let mut w = game(&rules());
    let victim = testkit::spawn(&mut w, Kind::Infantry, "DRONE", 1, testkit::mid(SIZE, 0, 0), 0);
    let at = w.obj(victim).unwrap().pos;
    venom(&mut w, victim, None, None);
    assert!(w.gas.anims.is_empty());
    let l = list(&w, shared(&w));
    assert_eq!(l.len(), 1);
    assert_eq!((l[0].pos.x, l[0].pos.y), (at.x, at.y));
    testkit::run(&mut w, 10);
    assert_eq!(list(&w, shared(&w)).len(), 1, "no later middle event");
}

/// spec: S-0613 claim 3 (with μ = 0 the creation middle event's particles come first, then the one extra)
#[test]
fn claim_3_mu_zero_adds_the_count_then_one() {
    let mut r = rules();
    r.sim.animations.get_mut("PLAGUE").unwrap().middle_frame = Some(0);
    let mut w = game(&r);
    let victim = testkit::spawn(&mut w, Kind::Infantry, "DRONE", 1, testkit::mid(SIZE, 0, 0), 0);
    venom(&mut w, victim, None, None);
    assert_eq!(list(&w, shared(&w)).len(), COUNT as usize + 1);
}

/// spec: S-0613 Edge cases (a virus-death animation without a particle type: claim 3 creates no particle)
#[test]
fn edge_virus_animation_without_particle_type() {
    let mut r = rules();
    r.sim.animations.get_mut("PLAGUE").unwrap().spawns_particle = None;
    let mut w = game(&r);
    let victim = testkit::spawn(&mut w, Kind::Infantry, "DRONE", 1, testkit::mid(SIZE, 0, 0), 0);
    venom(&mut w, victim, None, None);
    assert!(list(&w, shared(&w)).is_empty());
}

/// spec: S-0613 claim 4 and S-0303 claim 20 (a warhead's gas system at P, owned by the source house, lifetime −1, with one particle of its held type at P; without a held type no particle; a non-gas kind is not kept, Q-1243)
#[test]
fn claim_4_warhead_system_and_first_particle() {
    let mut w = game(&rules());
    let p = Pos::new(centre(0, 0).x + 10, centre(0, 0).y - 20, 0);
    let before: Vec<ObjectId> = w.gas.systems.keys().copied().collect();
    gas::warhead_system(&mut w, "CLOUD", p, Some(HouseId(1)));
    let (&id, s) = w
        .gas
        .systems
        .iter()
        .find(|(k, _)| !before.contains(k))
        .unwrap();
    assert_eq!(s.owner, Some(HouseId(1)));
    assert_eq!(s.lifetime, -1);
    assert_eq!(s.particles.len(), 1);
    assert_eq!(s.particles[0].pos, p);
    assert!(w.update.contains(id));
    let n = w.gas.systems.len();
    gas::warhead_system(&mut w, "BARE", p, None);
    assert_eq!(w.gas.systems.len(), n + 1);
    assert!(w.gas.systems.values().any(|s| s.particles.is_empty() && s.owner.is_none() && s.type_index == w.rules().gas.system("BARE").unwrap()));
    let rng = w.rng.clone();
    gas::warhead_system(&mut w, "SOOT", p, None);
    assert_eq!(w.gas.systems.len(), n + 1, "a smoke system is not kept");
    assert_ne!(w.rng, rng, "its first particle's creation draws are taken");
    // A gas system with lifetime −1 stays, inert, after its last particle is gone.
    w.gas.systems.get_mut(&id).unwrap().particles[0].life = 1;
    testkit::run(&mut w, 2);
    assert!(w.gas.systems[&id].particles.is_empty());
    assert!(w.gas.systems.contains_key(&id));
}

/// spec: S-0613 claim 4 and S-0303 claim 20 (a detonation of a warhead naming a particle system creates it at the end of the detonation)
#[test]
fn claim_4_detonation_creates_the_system() {
    use glasnost_sim::combat::area::{Detonation, detonate};
    let mut w = game(&rules());
    let h = warhead(&w, "MIST");
    let p = centre(0, 0);
    let n = w.gas.systems.len();
    detonate(
        &mut w,
        Detonation {
            point: p,
            warhead: Some(h),
            // S-0303 claim 2: a damage amount of 0 is no detonation at all.
            damage: 10,
            source: None,
            house: Some(HouseId(0)),
            resources: false,
        },
    );
    assert_eq!(w.gas.systems.len(), n + 1);
    let s = w.gas.systems.values().find(|s| s.owner == Some(HouseId(0))).unwrap();
    assert_eq!(s.particles.len(), 1);
}

/// spec: S-0613 claim 5 and S-0202 claim 15 (E is the drawn lifetime variation m(r rem MaxEC) modulo 65536, Q-1242; C the damage interval; the stage the start stage; a height at or below H raised to H; zero velocity and deflection)
#[test]
fn claim_5_particle_creation() {
    let mut w = game(&rules());
    let mut rng: Rng = w.rng.clone();
    let i = puff(&mut w, "FOG", Pos::new(centre(0, 0).x, centre(0, 0).y, -50));
    let r = rng.raw() as i32;
    let p = &list(&w, shared(&w))[i];
    assert_eq!(i32::from(p.life), (r % 30000).wrapping_abs());
    assert_eq!(p.countdown, 5);
    assert_eq!(p.stage, 0);
    assert_eq!(p.pos.z, 0, "raised to the terrain");
    assert_eq!(p.vel, [0, 0, 0]);
    assert_eq!(w.rng, rng, "one raw value for a gas particle without colours");
}

/// spec: S-0613 claim 6 (the update pass runs newest particle first; the lifetime goes down by one each turn and 0 marks the particle dead and removes it in that turn)
#[test]
fn claim_6_lifetime_and_order() {
    let mut w = game(&rules());
    to_even(&mut w);
    let a = puff(&mut w, "WISP", centre(0, 0));
    let b = puff(&mut w, "WISP", centre(2, 0));
    edit(&mut w, a, |p| p.life = 3);
    edit(&mut w, b, |p| p.life = 100);
    testkit::run(&mut w, 2);
    assert_eq!(list(&w, shared(&w))[a].life, 1);
    testkit::run(&mut w, 1);
    let l = list(&w, shared(&w));
    assert_eq!(l.len(), 1, "the particle whose lifetime reached 0 is gone");
    assert_eq!(l[0].life, 97);
}

/// spec: S-0613 claim 7 (a dead particle's successor at its position plus the displacement, with its velocity, created in the second pass and not moved that tick)
#[test]
fn claim_7_successor() {
    let mut r = rules();
    let wisp = r.sim.particles.get_mut("FOG").unwrap();
    wisp.next_particle = Some("WISP".into());
    wisp.next_particle_offset = Some([10, -20, 30]);
    let mut w = game(&r);
    to_even(&mut w);
    let i = puff(&mut w, "FOG", centre(0, 0));
    edit(&mut w, i, |p| {
        p.life = 1;
        p.vel = [1, -1, 0];
    });
    let at = list(&w, shared(&w))[i].pos;
    testkit::run(&mut w, 1);
    let l = list(&w, shared(&w));
    assert_eq!(l.len(), 1);
    assert_eq!(l[0].type_index, ptype(&w, "WISP"));
    assert_eq!(l[0].pos, Pos::new(at.x + 10, at.y - 20, at.z + 30));
    assert_eq!(l[0].vel, [1, -1, 0]);
}

/// spec: S-0613 claim 8 and F2 (one raw value on an even tick, none on an odd one; three when the low bits of the first are zero)
#[test]
fn claim_8_drift_draws() {
    let mut w = game(&rules());
    to_even(&mut w);
    let i = puff(&mut w, "WISP", centre(0, 0));
    edit(&mut w, i, |p| p.life = 1000);
    // Measure the turn's draws by running the system's turn alone.
    let s = shared(&w);
    let mut rng = w.rng.clone();
    gas::system_turn(&mut w, s);
    let r1 = rng.raw() as i32;
    if r1.wrapping_abs() & 7 != 0 {
        assert_eq!(w.rng, rng, "one raw value");
    } else {
        rng.raw();
        rng.raw();
        assert_eq!(w.rng, rng, "three raw values");
    }
    w.tick += 1;
    let before = w.rng.clone();
    gas::system_turn(&mut w, s);
    assert_eq!(w.rng, before, "no draw on an odd tick");
    let v = list(&w, s)[i].vel;
    assert!(v[0].abs() <= 2 && v[1].abs() <= 2);
}

/// spec: S-0613 claim 9 and F4 (above 5 leptons over the terrain v_z goes down by 1 on even ticks, not below −5)
#[test]
fn claim_9_vertical_speed() {
    let mut w = game(&rules());
    to_even(&mut w);
    let i = puff(&mut w, "WISP", Pos::new(centre(0, 0).x, centre(0, 0).y, 200));
    edit(&mut w, i, |p| p.life = 1000);
    let s = shared(&w);
    gas::system_turn(&mut w, s);
    assert_eq!(list(&w, s)[i].vel[2], -1);
    for _ in 0..12 {
        w.tick += 1;
        gas::system_turn(&mut w, s);
    }
    assert_eq!(list(&w, s)[i].vel[2], -5);
}

/// spec: S-0613 claim 10 and F3 (zero deflection over flat ground; on a ramp the deflection points downhill)
#[test]
fn claim_10_deflection_downhill() {
    let mut w = game(&rules());
    let p = centre(0, 0);
    assert_eq!(gas::deflection(&w, Pos::new(p.x, p.y, 5)), [glasnost_num::F32(0); 2]);
    // Shape 2 rises toward +y (S-0106 table C2: b = 1), so downhill is −y.
    let c = p.cell();
    if let Some(cell) = w.cells.grid.get_mut(c) {
        cell.slope = 2;
    }
    let h = w.surface_at(p.x, p.y);
    let d = gas::deflection(&w, Pos::new(p.x, p.y, h));
    let zero = glasnost_num::F64::ZERO;
    assert!(d[1].to_f64().lt(zero), "downhill is −y: {d:?}");
    // High above the terrain nothing deflects it.
    assert_eq!(gas::deflection(&w, Pos::new(p.x, p.y, h + 300)), [glasnost_num::F32(0); 2]);
}

/// spec: S-0613 claim 11 and F5 (after an odd-tick move the height is never below 5 above the terrain; on even ticks the height is unchanged)
#[test]
fn claim_11_movement_heights() {
    let mut w = game(&rules());
    to_even(&mut w);
    let i = puff(&mut w, "WISP", centre(0, 0));
    edit(&mut w, i, |p| p.life = 1000);
    let s = shared(&w);
    gas::system_turn(&mut w, s);
    assert_eq!(list(&w, s)[i].pos.z, 0, "even tick: height unchanged");
    w.tick += 1;
    gas::system_turn(&mut w, s);
    let p = list(&w, s)[i].pos;
    assert!(p.z >= w.surface_at(p.x, p.y) + 5);
}

/// spec: S-0613 claim 12, F6 and Edge cases (a positive wind effect pushes on ticks that are multiples of ⌊10 / W⌋; direction 3 pushes 1 lepton in x with quirks.wind_southeast_short on, 2 off)
#[test]
fn claim_12_wind_and_short_step() {
    for (quirk, dx) in [(true, 1), (false, 2)] {
        let mut r = rules();
        r.sim.general.wind_direction = Some(3);
        r.sim.particles.get_mut("WISP").unwrap().wind_effect = Some(5);
        r.globals.quirks.wind_southeast_short = quirk;
        let mut w = game(&r);
        assert_eq!(w.rules().gas.wind, Some((dx, 2)));
        to_even(&mut w);
        let i = puff(&mut w, "WISP", centre(0, 0));
        edit(&mut w, i, |p| p.life = 1000);
        let s = shared(&w);
        // Make the tick a multiple of ⌊10 / 5⌋ = 2 and even.
        let before = list(&w, s)[i].pos;
        gas::system_turn(&mut w, s);
        let after = list(&w, s)[i].pos;
        assert_eq!((after.x - before.x, after.y - before.y), (dx, 2));
    }
    // Outside 0 … 7: no push.
    let mut r = rules();
    r.sim.general.wind_direction = Some(9);
    let w = game(&r);
    assert_eq!(w.rules().gas.wind, None);
}

/// spec: S-0613 claim 13 and F7 (C counts down every update; at 0 with nonzero damage it hits the ground-layer objects of its cell with no attacker and the system's house, then restarts at the interval)
#[test]
fn claim_13_damage_cadence() {
    let mut w = game(&rules());
    let c = testkit::mid(SIZE, 0, 0);
    let target = testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, c, 0);
    let h0 = w.obj(target).unwrap().health;
    let i = puff(&mut w, "FOG", Pos::centre(c, 0));
    edit(&mut w, i, |p| {
        p.life = 1000;
        p.vel = [0, 0, 0];
    });
    let s = shared(&w);
    let mut hits = Vec::new();
    for n in 1..=10 {
        // Keep the cloud in its cell: no drift.
        edit(&mut w, i, |p| p.vel = [0, 0, 0]);
        let at = list(&w, s)[i].pos;
        edit(&mut w, i, |p| p.pos = Pos::new(Pos::centre(c, 0).x, Pos::centre(c, 0).y, at.z));
        w.effects.clear();
        gas::system_turn(&mut w, s);
        w.tick += 1;
        if w.effects.iter().any(|e| matches!(e, Effect::GasHit { id, .. } if *id == target)) {
            hits.push(n);
        }
    }
    assert_eq!(hits, [5, 10], "every 5 updates");
    assert!(w.obj(target).unwrap().health < h0);
}

/// spec: S-0613 claim 13 (a damage interval of 0 wraps: no early hit)
#[test]
fn claim_13_zero_interval_wraps() {
    let mut r = rules();
    r.sim.particles.get_mut("FOG").unwrap().max_dc = Some(0);
    let mut w = game(&r);
    let i = puff(&mut w, "FOG", centre(0, 0));
    assert_eq!(list(&w, shared(&w))[i].countdown, 0);
    edit(&mut w, i, |p| p.life = 1000);
    let s = shared(&w);
    gas::system_turn(&mut w, s);
    assert_eq!(list(&w, s)[i].countdown, 65535);
}

/// spec: S-0613 claim 14 (the shared system credits no house; a warhead-made system credits the detonation's house)
#[test]
fn claim_14_credit() {
    let mut w = game(&rules());
    assert_eq!(w.gas.systems[&shared(&w)].owner, None);
    gas::warhead_system(&mut w, "CLOUD", centre(0, 0), Some(HouseId(0)));
    assert!(w.gas.systems.values().any(|s| s.owner == Some(HouseId(0))));
}

/// spec: S-0613 claim 15 and F8 (the stage advances when (n + E_max − E) is divisible by (n mod 2) + A; at the end stage it resets to 0, or the particle dies with the delete flag)
#[test]
fn claim_15_stage_and_limit() {
    let mut r = rules();
    let wisp = r.sim.particles.get_mut("WISP").unwrap();
    wisp.end_state_ai = Some(2);
    wisp.delete_on_state_limit = Some(true);
    let mut w = game(&r);
    let i = puff(&mut w, "WISP", centre(0, 0));
    edit(&mut w, i, |p| p.life = 1000);
    let s = shared(&w);
    let n = list(&w, s)[i].id.0 as i32;
    let period = n % 2 + 4;
    let mut ticks = 0;
    while w.gas.systems[&s].particles.len() == 1 && ticks < 40 {
        gas::system_turn(&mut w, s);
        w.tick += 1;
        ticks += 1;
    }
    assert!(list(&w, s).is_empty(), "dead at the end stage");
    assert!(ticks <= 2 * period);
}

/// spec: S-0613 claim 13 and S-0302 (a gas hit is ordinary damage: a cloud kills a human infantry unit with its kind-8 warhead, whose death leaves a new animation, claim 2)
#[test]
fn chain_death_from_a_cloud() {
    let mut r = rules();
    r.sim.particles.get_mut("FOG").unwrap().damage = Some(10_000);
    let mut w = game(&r);
    let c = testkit::mid(SIZE, 0, 0);
    let victim = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, c, 0);
    let i = puff(&mut w, "FOG", Pos::centre(c, 0));
    edit(&mut w, i, |p| {
        p.life = 1000;
        p.countdown = 1;
    });
    let s = shared(&w);
    gas::system_turn(&mut w, s);
    assert!(w.obj(victim).is_none_or(|o| !o.on_map));
    assert_eq!(w.gas.anims.len(), 1);
}

/// spec: S-0613 State and ordering (gas state survives a save and load: the state hash is unchanged)
#[test]
fn save_load_round_trip() {
    let mut w = game(&rules());
    let victim = testkit::spawn(&mut w, Kind::Infantry, "WALKER", 1, testkit::mid(SIZE, 0, 0), 0);
    venom(&mut w, victim, None, None);
    testkit::run(&mut w, 5);
    assert!(!list(&w, shared(&w)).is_empty());
    let bytes = save::save(&w);
    let back = save::load(&bytes, w.rules_arc(), testkit::catalogue()).expect("loads");
    assert_eq!(save::state_hash(&back), save::state_hash(&w));
    assert_eq!(back.gas, w.gas);
}

/// spec: S-0613 claim 13, S-0302 claim 10 and S-0610 claim 4 (a cloud whose warhead is a berserk warhead sends its target berserk through ordinary damage resolution)
#[test]
fn berserk_cloud() {
    let mut r = rules();
    let venom = r.sim.warheads.get_mut("VENOM").unwrap();
    venom.psychedelic = Some(true);
    venom.inf_death = Some(0);
    let mut w = game(&r);
    let c = testkit::mid(SIZE, 0, 0);
    let target = testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", 1, c, 0);
    let h0 = w.obj(target).unwrap().health;
    let i = puff(&mut w, "FOG", Pos::centre(c, 0));
    edit(&mut w, i, |p| {
        p.life = 1000;
        p.countdown = 1;
    });
    let s = shared(&w);
    gas::system_turn(&mut w, s);
    let o = w.obj(target).unwrap();
    assert!(o.combat.berserk.is_some());
    assert_eq!(o.health, h0, "a berserk warhead changes no health");
}

/// F2 applied to `vel` with raw values from `rng` (a test-side copy of the formula).
fn drift_with(rng: &mut Rng, vel: [i32; 3]) -> [i32; 3] {
    let m = |r: i32| r.wrapping_abs();
    let r1 = rng.raw() as i32;
    if m(r1) & 7 != 0 {
        return vel;
    }
    let r2 = rng.raw() as i32;
    let r3 = rng.raw() as i32;
    let k = (m(r3) % 3) - 1;
    let mut v = vel;
    if m(r2) & 1 == 0 {
        v[0] = (v[0] + k).clamp(-2, 2);
    } else {
        v[1] = (v[1] + k).clamp(-2, 2);
    }
    v[0] = v[0].clamp(-2, 2);
    v[1] = v[1].clamp(-2, 2);
    v
}

/// spec: S-0613 claim 6 and F2 (the newer particle's update draws precede the older particle's)
#[test]
fn claim_6_newest_draws_first() {
    let mut w = game(&rules());
    to_even(&mut w);
    let old = puff(&mut w, "WISP", centre(0, 0));
    let new = puff(&mut w, "WISP", centre(3, 0));
    for i in [old, new] {
        edit(&mut w, i, |p| p.life = 30000);
    }
    let s = shared(&w);
    let mut told_apart = false;
    for _ in 0..400 {
        let (vo, vn) = (list(&w, s)[old].vel, list(&w, s)[new].vel);
        let mut a = w.rng.clone();
        let newer_first = {
            let n = drift_with(&mut a, vn);
            (drift_with(&mut a, vo), n)
        };
        let mut b = w.rng.clone();
        let older_first = {
            let o = drift_with(&mut b, vo);
            (o, drift_with(&mut b, vn))
        };
        gas::system_turn(&mut w, s);
        let got = (list(&w, s)[old].vel, list(&w, s)[new].vel);
        let got = ([got.0[0], got.0[1]], [got.1[0], got.1[1]]);
        let xy = |v: ([i32; 3], [i32; 3])| ([v.0[0], v.0[1]], [v.1[0], v.1[1]]);
        assert_eq!(got, xy(newer_first));
        told_apart |= xy(newer_first) != xy(older_first);
        // Keep the next turn even and the particles in place.
        w.tick += 2;
        for i in [old, new] {
            edit(&mut w, i, |p| p.vel[2] = 0);
        }
    }
    assert!(told_apart, "the run separated the two orders at least once");
}
