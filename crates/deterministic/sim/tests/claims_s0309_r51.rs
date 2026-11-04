//! Claim tests of S-0309 revision 51, claim 27 (a held target that is
//! destroyed, goes off the map or is removed; Q-2006), on the testkit's
//! synthetic combat ruleset (CART, CRUSHER, BUGGY, WALKER, TOWER: every
//! name and number invented, no game data). The ledger is
//! `docs/m8-claims/intake52b-S-0309-S-0405-M-0005-M-0006-M-0016.md`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{Decimal, SequenceEntry, WarheadType};
use glasnost_sim::combat::check::{self, Answer};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::held;
use glasnost_sim::objects;
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Target, World};

const SIZE: i32 = 24;

fn hit(amount: i32) -> Hit {
    Hit {
        amount,
        warhead: None,
        distance: 0,
        attacker: None,
        house: None,
        ignore_defences: true,
        no_escape: false,
    }
}

fn game() -> World {
    testkit::game_with(&testkit::combat_ruleset(), SIZE, 7)
}

/// The combat ruleset with WALKER given its infantry sequences (4 stages
/// each) and a warhead DIE1 of death kind 1 (S-0107 claim 17: a die-1
/// sequence), so that a killed WALKER stays in play while it plays.
fn dying_rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let s = &mut r.sim;
    let walker = s.infantry.get_mut("WALKER").unwrap();
    for k in [
        "stand", "walk", "go_prone", "prone", "crawl", "get_up", "die_1", "die_2",
    ] {
        walker.sequences.insert(
            k.into(),
            SequenceEntry {
                start: 0,
                count: 4,
                multiplier: 0,
                direction: None,
            },
        );
    }
    s.warheads.insert(
        "DIE1".into(),
        WarheadType {
            verses: Some(vec![Decimal("1".into()); 11]),
            inf_death: Some(1),
            ..Default::default()
        },
    );
    r
}

fn put(w: &mut World, kind: Kind, name: &str, house: u8, dx: i32, dy: i32) -> ObjectId {
    testkit::spawn(w, kind, name, house, testkit::mid(SIZE, dx, dy), 0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Leave {
    Kill,
    Inside,
    Remove,
}

fn leave(w: &mut World, v: ObjectId, how: Leave) {
    match how {
        Leave::Kill => {
            damage::apply(w, v, hit(1_000_000));
        }
        Leave::Inside => objects::take_inside(w, v),
        Leave::Remove => objects::remove(w, v),
    }
}

/// Gives `x` an override whose put-aside target is `aside` (S-0215 claim
/// 10): the suspended behaviour is *guard*, the current one *attack*.
fn overridden(w: &mut World, x: ObjectId, aside: ObjectId, current: ObjectId) {
    let o = w.obj_mut(x).unwrap();
    o.beh.current = Some(Behaviour::Attack);
    o.beh.suspended = Some(Behaviour::Guard);
    o.beh.aside = Some((Some(Target::Object(aside)), None));
    o.target = Some(Target::Object(current));
}

fn shots_by(w: &World, x: ObjectId) -> usize {
    w.projectiles
        .values()
        .filter(|p| p.attacker == Some(x))
        .count()
}

// spec: S-0309 claim 27 (every infantry unit, vehicle, building and aircraft holding it drops it at the killing hit, when it goes off the map and when it is removed, inside that action; no owner exception; a holder with more than 10 ticks left on its targeting timer restarts it with one ranged draw over 4 to 8 ticks, none with at most 10; several holders: key order, interim, S-0309 Open questions)
#[test]
fn claim_27_every_holder_drops_with_the_timer_reset() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
    let holders = [
        put(&mut w, Kind::Vehicle, "CART", 0, -4, 0),
        put(&mut w, Kind::Infantry, "WALKER", 0, -3, 2),
        put(&mut w, Kind::Building, "TOWER", 0, 4, 0),
        // The victim's own house: no owner exception.
        put(&mut w, Kind::Vehicle, "CART", 1, 0, 4),
    ];
    let half = w.rules().quirks.ranged_draw_half_span;
    // The draws the holders take, in key order, from the stream as it stands.
    let mut e = w.rng.clone();
    let drawn: Vec<i32> = holders.iter().map(|_| e.ranged(4, 8, half)).collect();
    let raws = e.taken() - w.rng.taken();
    for how in [Leave::Kill, Leave::Inside, Leave::Remove] {
        let taken = |left: i32| {
            let mut c = w.clone();
            let tick = c.tick;
            for &x in &holders {
                let o = c.obj_mut(x).unwrap();
                o.target = Some(Target::Object(v));
                o.targeting_timer = tick + left;
            }
            let before = c.rng.taken();
            leave(&mut c, v, how);
            let timers: Vec<i32> = holders
                .iter()
                .map(|&x| {
                    let o = c.obj(x).unwrap();
                    assert_eq!(o.target, None, "{how:?}: dropped at that moment");
                    o.targeting_timer - tick
                })
                .collect();
            (c.rng.taken() - before, timers)
        };
        let (base, kept) = taken(10);
        assert_eq!(kept, vec![10; 4], "{how:?}: 10 ticks left, no restart");
        assert_eq!(taken(5), (base, vec![5; 4]), "{how:?}: 5 ticks left");
        for left in [11, 50] {
            let (n, timers) = taken(left);
            assert_eq!(
                timers, drawn,
                "{how:?}: {left} left, one draw each in key order"
            );
            if how == Leave::Kill {
                assert!(n >= base + raws, "{how:?}: the draws taken");
            } else {
                assert_eq!(n, base + raws, "{how:?}: one draw per holder");
            }
        }
    }
}

// spec: S-0309 claim 27 (Consequences: a holder whose turn comes after the drop has no target in that turn and fires nothing; its rearm countdown runs on unchanged) (M8 seam sweep 5a)
#[test]
fn claim_27_the_holders_rearm_countdown_runs_on() {
    for how in [Leave::Kill, Leave::Inside, Leave::Remove] {
        let mut w = game();
        let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
        let a = put(&mut w, Kind::Vehicle, "CART", 0, -4, 0);
        let t = put(&mut w, Kind::Building, "TOWER", 0, 4, 0);
        let tick = w.tick;
        for (x, left) in [(a, 25), (t, 7)] {
            let o = w.obj_mut(x).unwrap();
            o.target = Some(Target::Object(v));
            o.combat.rearm_until = tick + left;
        }
        leave(&mut w, v, how);
        for (x, left) in [(a, 25), (t, 7)] {
            let o = w.obj(x).unwrap();
            assert_eq!(o.target, None, "{how:?}: dropped");
            assert_eq!(o.combat.rearm_until, tick + left, "{how:?}: unchanged");
        }
        // Its next turns: no target, no shot, and the countdown is not restarted.
        for _ in 0..5 {
            glasnost_sim::tick::step(&mut w);
        }
        for (x, left) in [(a, 25), (t, 7)] {
            let o = w.obj(x).unwrap();
            assert_eq!(o.target, None, "{how:?}: no target in its turns");
            assert_eq!(o.combat.rearm_until, tick + left, "{how:?}: runs on");
            assert_eq!(shots_by(&w, x), 0, "{how:?}: no shot");
        }
    }
}

// spec: S-0309 claim 27 (the drop ends an override, S-0215 claim 10: the revert can give the object off the map back; DECIDED: no target in the firing step, dropped at the start of the holder's next turn with no draw and no revert; the removal drops it again, the drops taking at most one draw)
#[test]
fn claim_27_a_revert_gives_back_an_object_off_the_map() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
    let a = put(&mut w, Kind::Vehicle, "CART", 0, -3, 0);
    overridden(&mut w, a, v, v);
    let tick = w.tick;
    w.obj_mut(a).unwrap().targeting_timer = tick + 50;
    let mut e = w.rng.clone();
    let drawn = e.ranged(4, 8, w.rules().quirks.ranged_draw_half_span);
    objects::take_inside(&mut w, v);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, Some(Target::Object(v)), "the revert gave it back");
    assert_eq!(o.beh.suspended, None, "reverted");
    assert_eq!(o.beh.current, Some(Behaviour::Guard));
    assert_eq!(o.targeting_timer - tick, drawn, "one draw");
    assert_eq!(
        held::firing_target(&w, w.obj(a).unwrap()),
        None,
        "no target in the firing step"
    );
    // The removal drops it again, with no second draw.
    let mut c = w.clone();
    objects::remove(&mut c, v);
    let o = c.obj(a).unwrap();
    assert_eq!(o.target, None, "dropped again at the removal");
    assert_eq!(o.targeting_timer - tick, drawn, "no second draw");
    // Without the removal: dropped at the start of its next turn, no shot.
    glasnost_sim::tick::step(&mut w);
    let o = w.obj(a).unwrap();
    assert_eq!(o.target, None, "dropped at the start of its turn");
    assert_eq!(o.beh.suspended, None, "no revert");
    assert_eq!(shots_by(&w, a), 0, "no shot at it or at its place");
}

// spec: S-0309 claim 27 (a destroyed object is dropped at the killing hit, also an infantry unit in its death sequence that stays in play; a holder's later firing step has no target and fires nothing; a revert that gives it back: no target in the firing step, dropped at the start of the next turn, DECIDED)
#[test]
fn claim_27_killed_infantry_in_its_death_sequence() {
    let r = dying_rules();
    let die1 = r.sim.warheads.keys().position(|k| k == "DIE1").unwrap() as u32;
    let mut w = testkit::game_with(&r, SIZE, 7);
    let v = put(&mut w, Kind::Infantry, "WALKER", 1, 0, 0);
    let a = put(&mut w, Kind::Vehicle, "CART", 0, -3, 0);
    let b = put(&mut w, Kind::Vehicle, "CART", 0, -3, 2);
    w.obj_mut(a).unwrap().target = Some(Target::Object(v));
    overridden(&mut w, b, v, v);
    damage::apply(
        &mut w,
        v,
        Hit {
            warhead: Some(die1),
            ignore_defences: false,
            ..hit(1_000_000)
        },
    );
    let dying = w.obj(v).expect("still in play, in its death sequence");
    assert!(damage::dead(dying));
    assert_eq!(w.obj(a).unwrap().target, None, "dropped at the killing hit");
    assert_eq!(
        w.obj(b).unwrap().target,
        Some(Target::Object(v)),
        "the revert gave it back"
    );
    assert_eq!(held::firing_target(&w, w.obj(b).unwrap()), None);
    glasnost_sim::tick::step(&mut w);
    for x in [a, b] {
        assert_eq!(w.obj(x).unwrap().target, None, "no target in its turn");
        assert_eq!(shots_by(&w, x), 0, "no shot at the dying unit or its place");
    }
}

// spec: S-0309 claim 27 (a building whose animation-delayed shot is pending makes, in the turn that ends the delay, no firing check and no shot if it then holds no target, and the shot is no longer pending; S-0301 claim 14)
#[test]
fn claim_27_a_pending_delayed_shot_without_a_target() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
    let t = put(&mut w, Kind::Building, "TOWER", 0, 4, 0);
    let tick = w.tick;
    {
        let o = w.obj_mut(t).unwrap();
        o.target = Some(Target::Object(v));
        o.combat.delayed_shot = Some(tick + 1);
        o.combat.delayed_slot = 0;
    }
    objects::remove(&mut w, v);
    assert_eq!(w.obj(t).unwrap().target, None);
    testkit::run(&mut w, 2);
    let o = w.obj(t).unwrap();
    assert_eq!(o.combat.delayed_shot, None, "no longer pending");
    assert_eq!(shots_by(&w, t), 0, "no shot");
}

/// One tick of `w`: whether `x` made a shot in it (its shot effects).
fn step_shot(w: &mut World, x: ObjectId) -> bool {
    use glasnost_sim::combat::effects::Effect;
    w.effects.clear();
    glasnost_sim::tick::step(w);
    w.effects
        .iter()
        .any(|e| matches!(e, Effect::Shot { firer, .. } if *firer == x))
}

// spec: S-0309 claim 27 (consequences: a target destroyed earlier in the tick by another firer's shot, S-0306 scenario P27, is dropped before the firing step of a holder whose turn comes later, S-0203 claim 3; that step asks no firing check and fires nothing; the drop restarts a timer with more than 10 ticks left); S-0203 claim 10 (the object right after a removed projectile misses its turn, so a filler unit stands between)
#[test]
fn claim_27_a_target_killed_earlier_in_the_tick_by_another_firer() {
    let mut w = game();
    let v = put(&mut w, Kind::Vehicle, "BUGGY", 1, 0, 0);
    let a = put(&mut w, Kind::Vehicle, "CRUSHER", 0, -4, 0);
    w.obj_mut(a).unwrap().target = Some(Target::Object(v));
    // A's first shot, through its own firing step.
    let mut fired = false;
    for _ in 0..200 {
        if step_shot(&mut w, a) {
            fired = true;
            break;
        }
    }
    assert!(fired, "A fired at V");
    w.obj_mut(v).unwrap().health = 1;
    // B is created while the shell flies, so its turn comes after the
    // shell's in every tick (key order). A unit created between them takes
    // the missed turn that the shell's removal gives the object right after
    // it (S-0203 claim 10), so B keeps its turn in the kill's tick.
    put(&mut w, Kind::Vehicle, "BUGGY", 0, 6, 6);
    let b = put(&mut w, Kind::Vehicle, "CRUSHER", 0, -4, 2);
    {
        let tick = w.tick;
        let o = w.obj_mut(b).unwrap();
        o.target = Some(Target::Object(v));
        o.targeting_timer = tick + 500;
    }
    let gone = |w: &World| w.obj(v).is_none_or(damage::dead);
    // The tick of the kill, in a copy.
    let mut c = w.clone();
    let mut kill = None;
    for _ in 0..200 {
        glasnost_sim::tick::step(&mut c);
        if gone(&c) {
            kill = Some(c.tick);
            break;
        }
    }
    let kill = kill.expect("the shell kills V");
    // `kill` is the counter after the kill's step, so that step runs tick
    // `kill - 1`. B's countdown runs below `rearm_until` (S-0301 claim 1),
    // so B holds its fire before that tick and is ready in it.
    w.obj_mut(b).unwrap().combat.rearm_until = kill - 1;
    // The control: with V kept alive, B fires in that tick.
    let mut control = w.clone();
    control.obj_mut(v).unwrap().health = 1_000_000;
    let mut shot_in = None;
    while control.tick < kill + 1 {
        let t = control.tick;
        if step_shot(&mut control, b) {
            shot_in.get_or_insert(t);
        }
    }
    assert_eq!(
        shot_in,
        Some(kill - 1),
        "control: B fires in the kill's tick"
    );
    let mut b_shots = 0;
    while w.tick < kill - 1 {
        b_shots += usize::from(step_shot(&mut w, b));
    }
    assert!(!gone(&w), "V is still in play when the kill's tick starts");
    assert_eq!(
        check::check(&w, b, Some(Target::Object(v)), 0, true),
        Answer::CanFire,
        "B's firing check against V allows the shot when that tick starts"
    );
    b_shots += usize::from(step_shot(&mut w, b));
    assert!(gone(&w), "killed in that tick");
    assert_eq!(b_shots, 0, "B fires nothing in or before that tick");
    let o = w.obj(b).unwrap();
    assert_eq!(o.target, None, "dropped before B's firing step");
    let left = o.targeting_timer - w.tick;
    assert!((3..=8).contains(&left), "one draw over 4 to 8: {left} left");
}
