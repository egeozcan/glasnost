//! The clock's quirk switches reach the simulation (S-0109 Edge cases and
//! quirks; M8 seam sweep 3, `docs/m8-claims/seams-3.md`). Each test plays
//! one synthetic animation type through the real order and tick path in
//! both positions of one switch (`globals.quirks.anim_*`, the ruleset's
//! quirks): the default plays as the original, the other position as the
//! spec's "off" text says. Every number is our own.

use glasnost_map::CellXY;
use glasnost_rules::quirks::Quirks;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::AnimationType;
use glasnost_sim::anims::{self, Spawn};
use glasnost_sim::world::{ObjectId, Pos, World};
use glasnost_sim::{testkit, tick};

const SIZE: i32 = 16;

/// Four frames at rate 900 (d = 1), one pass.
fn four() -> AnimationType {
    AnimationType {
        frame_count: Some(4),
        rate: Some(900),
        ..Default::default()
    }
}

fn rules(types: &[(&str, AnimationType)], set: impl Fn(&mut Quirks)) -> Ruleset {
    let mut r = testkit::combat_ruleset();
    for (name, t) in types {
        let l = r.registries.entry("animations".into()).or_default();
        l.push((*name).into());
        r.sim.animations.insert((*name).into(), t.clone());
    }
    set(&mut r.globals.quirks);
    r
}

fn ground(w: &World, c: CellXY) -> Pos {
    let (x, y) = (c.x * 256 + 128, c.y * 256 + 128);
    Pos::new(x, y, w.surface_at(x, y))
}

/// The image frame the animation shows after each of `n` ticks, `None`
/// once removed.
fn frames(r: &Ruleset, name: &str, n: usize) -> Vec<Option<i32>> {
    let mut w = testkit::game_with(r, SIZE, 3);
    let at = ground(&w, testkit::mid(SIZE, 0, 0));
    let id: ObjectId = anims::spawn(&mut w, name, Some(at), Spawn::default())
        .id
        .expect("a listed type is in play");
    (0..n)
        .map(|_| {
            tick::step(&mut w);
            anims::shown(&w, id).map(|s| s.1)
        })
        .collect()
}

fn some(v: &[i32]) -> Vec<Option<i32>> {
    v.iter().map(|f| Some(*f)).collect()
}

#[test]
fn quirk_anim_loop_count_wrap_in_both_positions() {
    // spec: S-0109 claim 7 and Edge cases (a loop count of 256 is 256 mod 256 = 0, raised to one pass; off: 256 passes of four ticks)
    let t = AnimationType {
        loop_count: Some(256),
        ..four()
    };
    let on = rules(&[("BIG", t.clone())], |_| {});
    assert_eq!(
        frames(&on, "BIG", 6),
        [Some(0), Some(1), Some(2), Some(3), None, None]
    );
    let off = rules(&[("BIG", t)], |q| q.anim_loop_count_wrap = false);
    let got = frames(&off, "BIG", 1030);
    assert_eq!(got.iter().take_while(|f| f.is_some()).count(), 1024);
    assert_eq!(got[1023], Some(3));
    assert_eq!(got[1024], None);
}

#[test]
fn quirk_anim_reverse_raw_loop_end_in_both_positions() {
    // spec: S-0109 claim 8 and Edge cases (a reversed pass starts at λₑ − 1 and ends after an advance to k <= 0, so frame a is never drawn; off: it ends when k < 0 and shows down to a)
    let t = AnimationType {
        reverse: Some(true),
        ..four()
    };
    let on = rules(&[("BACK", t.clone())], |_| {});
    assert_eq!(
        frames(&on, "BACK", 5),
        [Some(3), Some(2), Some(1), None, None]
    );
    let off = rules(&[("BACK", t)], |q| q.anim_reverse_raw_loop_end = false);
    assert_eq!(
        frames(&off, "BACK", 5),
        [Some(3), Some(2), Some(1), Some(0), None]
    );
}

#[test]
fn quirk_anim_loop_end_from_sprite_in_both_positions() {
    // spec: S-0109 claims 6 and 25 and Edge cases (an absent loop end takes the sprite end 4, so the first of two passes shows 0 to 3 and the last stops at the end field 2; off: it takes the end field, so both passes show 0 and 1)
    let t = AnimationType {
        end: Some(2),
        loop_count: Some(2),
        ..four()
    };
    let on = rules(&[("SHORT", t.clone())], |_| {});
    let mut want = some(&[0, 1, 2, 3, 0, 1]);
    want.extend([None, None]);
    assert_eq!(frames(&on, "SHORT", 8), want);
    let off = rules(&[("SHORT", t)], |q| q.anim_loop_end_from_sprite = false);
    let mut want = some(&[0, 1, 0, 1]);
    want.extend([None, None]);
    assert_eq!(frames(&off, "SHORT", 6), want);
}

#[test]
fn quirk_anim_followup_start_doubled_in_both_positions() {
    // spec: S-0109 claim 15 and Edge cases (the follow-up starts at k = a′, so with a′ = 1 its first image frame is 2; off: at k₀ = 0, its first image frame a′ = 1)
    let types = [
        (
            "ONE",
            AnimationType {
                next: Some("TWO".into()),
                ..four()
            },
        ),
        (
            "TWO",
            AnimationType {
                start: Some(1),
                ..four()
            },
        ),
    ];
    let on = frames(&rules(&types, |_| {}), "ONE", 12);
    let off = frames(
        &rules(&types, |q| q.anim_followup_start_doubled = false),
        "ONE",
        12,
    );
    // The first type's pass shows 0 to 3; the follow-up's first frame is the switch tick's.
    assert_eq!(&on[..4], &some(&[0, 1, 2, 3])[..]);
    assert_eq!(&off[..4], &some(&[0, 1, 2, 3])[..]);
    assert_eq!(on[4], Some(2), "doubled start: a′ + a′");
    assert_eq!(off[4], Some(1), "k₀ = 0: a′");
}

#[test]
fn quirk_anim_followup_keeps_step_in_both_positions() {
    // spec: S-0109 claim 15 and Edge cases (an animation reversed by its old type keeps σ = −1 across the switch and runs down for ever, invisible; off: σ is set afresh, +1 for a follow-up type that is not reversed, so the follow-up plays forward and ends)
    let types = [
        (
            "DOWN",
            AnimationType {
                reverse: Some(true),
                next: Some("UP".into()),
                ..four()
            },
        ),
        ("UP", four()),
    ];
    let on = frames(&rules(&types, |_| {}), "DOWN", 60);
    assert!(
        on.iter().all(|f| f.is_some()),
        "kept σ = −1: the animation never ends: {on:?}"
    );
    let off = frames(
        &rules(&types, |q| q.anim_followup_keeps_step = false),
        "DOWN",
        60,
    );
    assert!(
        off.last().is_some_and(|f| f.is_none()),
        "σ = +1: the follow-up plays forward and ends: {off:?}"
    );
}

#[test]
fn quirk_anim_pingpong_lower_turn_in_both_positions() {
    // spec: S-0109 claim 9 and Edge cases (while r >= 2 and a = 1 the turn needs k = a exactly, so the animation turns at frame 2, comes down past a and runs below 0, drawing nothing; off: the lower turn is at k <= λₛ − a, so it turns back at frame 0)
    let t = AnimationType {
        ping_pong: Some(true),
        start: Some(1),
        loop_count: Some(2),
        ..four()
    };
    let on = frames(&rules(&[("SWING", t.clone())], |_| {}), "SWING", 14);
    let off = frames(
        &rules(&[("SWING", t)], |q| q.anim_pingpong_lower_turn = false),
        "SWING",
        14,
    );
    let low = |v: &[Option<i32>]| v.iter().flatten().copied().min();
    assert!(on.iter().all(|f| f.is_some()), "{on:?}");
    assert_eq!(low(&on), Some(-10), "never turns back: {on:?}");
    assert_eq!(low(&off), Some(0), "turns back at frame 0: {off:?}");
    assert!(off.iter().skip(5).any(|f| *f == Some(2)), "{off:?}");
}
