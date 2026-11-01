//! Claim tests for S-0501 r65 (Q-2237): the damage-smoke animation of
//! claim 37 and its place in the aircraft's turn (State and ordering), on
//! synthetic data through the real tick path. The fixed-named type is
//! registered here under another letter case with invented fields (a
//! descending random rate pair, so its creation takes one ranged draw, and
//! a loop count); every other type, weapon and number is invented.

mod common;
mod econ_common;

use econ_common::{SIZE, game, ready};
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, Locomotor, UnitType};
use glasnost_sim::air;
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId, Pos, Target, World};

/// The fixed name in another letter case (S-0004 claims 2 and 7).
const SMOKE: &str = "sgrysmk1";
/// The test smoke type's loop count *L*: with repeat factor 1 its
/// animation starts with *L* passes (S-0109 claim 7).
const LOOPS: i32 = 3;

fn push(r: &mut Ruleset, reg: &str, n: &str) {
    r.registries.entry(reg.into()).or_default().push(n.into());
}

/// The aircraft ruleset: the flying type SCOUT (strength 200), whose own
/// explosion, destruction and debris animation entries name the test
/// animation BOOM, and, when `smoke`, the damage-smoke type under [`SMOKE`].
fn rules(smoke: bool) -> Ruleset {
    let mut r = econ_common::ruleset();
    push(&mut r, "animations", "BOOM");
    r.sim
        .animations
        .insert("BOOM".into(), AnimationType::default());
    if smoke {
        push(&mut r, "animations", SMOKE);
        r.sim.animations.insert(
            SMOKE.into(),
            AnimationType {
                // A descending pair: d = 450 and 900, a random stage delay drawn at creation.
                random_rate: Some([2, 1]),
                loop_count: Some(LOOPS),
                ..Default::default()
            },
        );
    }
    let mut u = UnitType {
        strength: Some(200),
        speed: Some(30),
        rot: Some(12),
        locomotor: Some(Locomotor::Fly),
        tech_level: Some(1),
        owner: Some(vec!["Alphaland".into(), "Betaland".into()]),
        cost: Some(1000),
        ..Default::default()
    };
    // spec: S-0501 claim 37 and AC5 part d (no entry of the aircraft's type names the smoke type)
    u.combat.explosion = Some(vec!["BOOM".into()]);
    u.combat.destroy_anim = Some(vec!["BOOM".into()]);
    u.combat.debris_anims = Some(vec!["BOOM".into()]);
    push(&mut r, "aircraft", "SCOUT");
    r.sim.aircraft.insert("SCOUT".into(), u);
    r
}

/// The registration index of animation type `name`, letter case aside.
fn anim_index(w: &World, name: &str) -> u32 {
    w.rules()
        .animations
        .iter()
        .position(|a| a.name.eq_ignore_ascii_case(name))
        .expect("registered") as u32
}

/// The animations in play of type index `ti`: id, position and owner.
fn anims_of(w: &World, ti: u32) -> Vec<(ObjectId, Option<Pos>)> {
    w.anims
        .anims
        .iter()
        .filter(|(_, a)| a.anim == Some(ti))
        .map(|(id, a)| (*id, a.pos))
        .collect()
}

/// A SCOUT of house 0 in flight at height 500 or more, moving away on a
/// long move.
fn lifted(w: &mut World) -> ObjectId {
    let a = testkit::spawn(w, Kind::Aircraft, "SCOUT", 0, testkit::mid(SIZE, 8, 8), 0);
    let far = testkit::mid(SIZE, -12, -12);
    w.tick = w.tick.max(w.orders_blocked_through + 1);
    assert!(orders::issue(
        w,
        HouseId(0),
        Order::Behaviour {
            actor: a,
            behaviour: Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(far)),
        }
    ));
    for _ in 0..300 {
        if air::height(w, a) >= 500 {
            return a;
        }
        glasnost_sim::tick::step(w);
    }
    panic!("not lifted");
}

fn kill(w: &mut World, a: ObjectId) {
    damage::apply(
        w,
        a,
        Hit {
            amount: 1_000_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    assert!(air::crashing(w, a), "shot down in the air");
}

/// The `Effect::Anim` reports of this tick for animation `id`.
fn reported(w: &World, id: ObjectId) -> Vec<(String, Pos)> {
    w.effects
        .iter()
        .filter_map(|e| match e {
            Effect::Anim {
                name, pos, id: i, ..
            } if *i == Some(id) => Some((name.clone(), *pos)),
            _ => None,
        })
        .collect()
}

/// Checks a smoke animation `id` just created by aircraft `a`: the fixed
/// type, at `at` (the aircraft's position at the end of the tick), not
/// attached, with no loop delay, repeat factor 1 and no reverse request.
fn check_new_smoke(w: &World, id: ObjectId, at: Pos) {
    let s = &w.anims.anims[&id];
    assert_eq!(
        s.pos,
        Some(at),
        "where the tick's movement left the aircraft"
    );
    assert_eq!(s.owner, None, "not attached");
    assert_eq!(s.role, glasnost_sim::anims::Role::Ordinary);
    assert_eq!(s.player.loop_delay(), 0, "no initial loop delay");
    assert_eq!(
        s.player.remaining_passes(),
        Some(i64::from(LOOPS)),
        "repeat factor 1: L passes"
    );
    assert!(!s.player.play.reverse, "no reverse request");
    assert_eq!(reported(w, id).len(), 1, "one presentation report");
    assert_eq!(reported(w, id)[0].1, at);
}

// spec: S-0501 claim 37 (Q-2237: a living aircraft's shown smoke is one animation of the fixed type, found without regard to letter case, at the aircraft's position when u is taken, after its own movement in that tick, height included; not attached, so it stays where it was created while the aircraft flies on; no initial loop delay, repeat factor 1, no reverse request), State and ordering (the smoke draw follows the aircraft's own movement), AC5 parts b and d
#[test]
fn claim_37_a_living_aircrafts_smoke_is_the_fixed_type_where_its_movement_left_it() {
    let mut w = game(&rules(true));
    ready(&mut w);
    let a = lifted(&mut w);
    w.obj_mut(a).unwrap().health = 20;
    let (smoke, boom) = (anim_index(&w, SMOKE), anim_index(&w, "BOOM"));
    let mut first: Option<(ObjectId, Pos, i32)> = None;
    let mut made = 0;
    for _ in 0..600 {
        let start = w.obj(a).unwrap().pos;
        let before: Vec<ObjectId> = w.anims.anims.keys().copied().collect();
        glasnost_sim::tick::step(&mut w);
        let end = w.obj(a).unwrap().pos;
        for (id, _) in anims_of(&w, smoke) {
            if before.contains(&id) {
                continue;
            }
            made += 1;
            assert!(air::height(&w, a) > 0);
            assert_ne!(start, end, "the aircraft moved in this tick");
            check_new_smoke(&w, id, end);
            first.get_or_insert((id, end, w.tick));
        }
        assert!(
            anims_of(&w, boom).is_empty(),
            "the type's own entries make no smoke"
        );
        if let Some((id, at, t)) = first
            && w.tick >= t + 20
        {
            // The aircraft flew on; the smoke stayed.
            assert_ne!(w.obj(a).unwrap().pos, at);
            assert_eq!(w.anims.anims[&id].pos, Some(at), "not attached");
            break;
        }
    }
    assert!(made >= 1, "a living aircraft shows smoke");
    assert!(first.is_some());
}

// spec: S-0501 claim 37 (Q-2237: a dead aircraft's smoke is the same fixed type, point and settings as a living one's; only the chance differs), AC5 parts b and d
#[test]
fn claim_37_a_dead_aircrafts_smoke_is_the_same_type() {
    let mut w = game(&rules(true));
    ready(&mut w);
    let a = lifted(&mut w);
    let smoke = anim_index(&w, SMOKE);
    kill(&mut w, a);
    let mut made = 0;
    for _ in 0..200 {
        let before: Vec<ObjectId> = w.anims.anims.keys().copied().collect();
        glasnost_sim::tick::step(&mut w);
        let Some(end) = w.obj(a).map(|o| o.pos) else {
            break;
        };
        for (id, a_) in w.anims.anims.iter() {
            if before.contains(id) {
                continue;
            }
            assert_eq!(
                a_.anim,
                Some(smoke),
                "the only animation of the fall is the smoke"
            );
            made += 1;
            check_new_smoke(&w, *id, end);
        }
    }
    assert!(made >= 1, "a falling aircraft shows smoke");
}

// spec: S-0501 claim 37 (Q-2237: the creation's draws, S-0109 claim 3's stage-delay draw for this type, come right after u in the aircraft's own turn; a shown smoke takes u plus them, an unshown one u alone), AC5 part c
#[test]
fn claim_37_the_smoke_creation_draws_follow_u() {
    let mut w = game(&rules(true));
    ready(&mut w);
    let a = lifted(&mut w);
    kill(&mut w, a);
    let smoke = anim_index(&w, SMOKE);
    let half = w.rules().quirks.ranged_draw_half_span;
    let (mut shown, mut unshown) = (0, 0);
    for _ in 0..40 {
        let mut r = w.rng.clone();
        let u = r.ranged(0, 99, half);
        // spec: S-0501 claim 37 (a dead aircraft shows smoke when u < 80)
        let show = u < 80;
        if show {
            // spec: S-0109 claim 3 (the descending pair's converted range [450, 900])
            let _ = r.ranged(450, 900, half);
        }
        let n = anims_of(&w, smoke).len();
        w.effects.clear();
        air::end_turn(&mut w, a);
        assert_eq!(w.rng, r, "u, then the creation's draws exactly when shown");
        assert_eq!(anims_of(&w, smoke).len(), n + usize::from(show));
        if show {
            shown += 1;
        } else {
            unshown += 1;
        }
    }
    assert!(
        shown > 0 && unshown > 0,
        "both cases met: {shown} {unshown}"
    );
}

// spec: S-0501 claim 37 (Q-2237: with no animation type of the fixed name no smoke is created and u is still taken)
#[test]
fn claim_37_without_the_type_nothing_is_created_and_u_is_taken() {
    let mut w = game(&rules(false));
    ready(&mut w);
    let a = lifted(&mut w);
    kill(&mut w, a);
    let half = w.rules().quirks.ranged_draw_half_span;
    for _ in 0..20 {
        let mut r = w.rng.clone();
        let _ = r.ranged(0, 99, half);
        let n = w.anims.anims.len();
        w.effects.clear();
        air::end_turn(&mut w, a);
        assert_eq!(w.rng, r, "u alone");
        assert_eq!(w.anims.anims.len(), n);
        assert!(!w.effects.iter().any(|e| matches!(e, Effect::Anim { .. })));
    }
}

// spec: S-0501 claim 37 and Edge cases (quirks.aircraft_smoke_scenario_draw on: the draw and the smoke animation; off: no damage-smoke draw and no smoke animation)
#[test]
fn claim_37_the_smoke_animation_follows_the_scenario_draw_switch_both_ways() {
    for on in [true, false] {
        let mut r = rules(true);
        r.globals.quirks.aircraft_smoke_scenario_draw = on;
        let mut w = game(&r);
        ready(&mut w);
        let a = lifted(&mut w);
        kill(&mut w, a);
        let smoke = anim_index(&w, SMOKE);
        let (rng, n) = (w.rng.clone(), anims_of(&w, smoke).len());
        for _ in 0..20 {
            air::end_turn(&mut w, a);
        }
        if on {
            assert_ne!(w.rng, rng);
            assert!(anims_of(&w, smoke).len() > n, "on: smoke");
        } else {
            assert_eq!(w.rng, rng, "off: no draw");
            assert_eq!(anims_of(&w, smoke).len(), n, "off: no smoke");
        }
    }
}
