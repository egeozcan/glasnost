//! S-0522 claim 9 with S-0208 claims 2 and 3 and S-0208 Edge cases
//! (`quirks.top_position_counts_as_centre`): a paratrooper's standing
//! position is chosen by the position class of claim 2 whatever the switch
//! (#276). Synthetic rules from `tests/sw_common`.

mod sw_common;

use glasnost_rules::sim::SequenceEntry;
use glasnost_sim::world::Pos;
use glasnost_sim::{paradrop, superweapons::planes, transport, walk};
use sw_common::*;

/// One drop whose drop point is the top standing point (64, 64) of a clear
/// cell, with `quirks.top_position_counts_as_centre` set to `quirk`.
/// Returns (position within the cell the paratrooper got, whether exactly
/// one R(0, 3) was taken).
fn drop_on_top_point(quirk: bool) -> ((i32, i32), bool) {
    let mut r = ruleset();
    r.globals.quirks.top_position_counts_as_centre = quirk;
    let walker = r.sim.infantry.get_mut("WALKER").unwrap();
    for k in ["stand", "walk", "parachute"] {
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
    let mut w = game(&r);
    assert!(w.rules().superweapons.parachute.is_none());
    let p = planes::paradrop(&mut w, ALPHA, at(0, 0), false)[0];
    let c = at(1, 1);
    let (px, py) = (256 * c.x - 64, 256 * c.y + 64);
    let t = w.tick;
    {
        let o = w.obj_mut(p).unwrap();
        o.pos = Pos::new(px, py, o.pos.z);
        o.body.set(0, t, false);
        o.combat.ammo = 3;
    }
    let drop = walk::step_point(px, py, 16383, 128, true, w.rules().mode);
    assert_eq!(drop, (256 * c.x + 64, 256 * c.y + 64), "the drop point");
    let first = transport::passengers(&w, p)[0];
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut expected = w.rng.clone();
    expected.ranged(0, 3, half);
    assert!(paradrop::attempt(&mut w, p));
    let o = w.obj(first).unwrap();
    assert_eq!(o.pos.cell(), c);
    (
        (o.pos.x.rem_euclid(256), o.pos.y.rem_euclid(256)),
        w.rng == expected,
    )
}

// spec: S-0522 claim 9 (the paratrooper's standing position by S-0208 claim 3, the drop point the desired point) and claim 16 (one R(0, 3) for a centre-class drop point)
#[test]
fn issue_276_switch_on_takes_the_centre_class_draw_and_a_side_position() {
    let (at, one_draw) = drop_on_top_point(true);
    assert!(one_draw, "one R(0, 3)");
    assert_ne!(at, (64, 64));
    assert_ne!(at, (128, 128));
}

// spec: S-0208 Edge cases (quirks.top_position_counts_as_centre off: the class of every point is still claim 2's, so claim 3 chooses and draws exactly as with the switch on); S-0522 claim 9
#[test]
fn issue_276_switch_off_chooses_exactly_as_with_the_switch_on() {
    let (at, one_draw) = drop_on_top_point(false);
    assert_ne!(
        at,
        (64, 64),
        "the top position is never given (S-0208 claim 3)"
    );
    assert!(one_draw, "one R(0, 3)");
}
