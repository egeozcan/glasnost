//! The touch-gesture recogniser without a window (design: touch and
//! touchpad scrolling (lead's request 2026-11-10)).

use super::*;

const PITCH: i32 = 56;

fn at(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

/// Feeds one event in `region` at time `t`.
fn ev(
    t: &mut Touches,
    phase: Phase,
    id: u64,
    p: Point,
    ms: i64,
    region: Region,
) -> Option<Gesture> {
    t.event(phase, id, p, ms, region, PITCH)
}

/// A touch that lifts within the slop is a left click where it touched
/// down; one just past the slop is not.
#[test]
fn a_touch_within_the_slop_is_a_left_click() {
    let mut t = Touches::new();
    assert_eq!(
        ev(&mut t, Phase::Start, 1, at(100, 100), 0, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(106, 108), 10, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::End, 1, at(106, 108), 900, Region::Map),
        Some(Gesture::Click(Button::Left, at(100, 100)))
    );
    assert!(!t.active());
    // A long press still clicks; taps click on every region.
    ev(&mut t, Phase::Start, 2, at(5, 5), 0, Region::Other);
    assert_eq!(
        ev(&mut t, Phase::End, 2, at(5, 5), 5000, Region::Other),
        Some(Gesture::Click(Button::Left, at(5, 5)))
    );
    // Past the slop (11 pixels) and back: a drag, so no click.
    ev(&mut t, Phase::Start, 3, at(100, 100), 0, Region::Other);
    assert_eq!(
        ev(&mut t, Phase::Move, 3, at(111, 100), 10, Region::Other),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::Move, 3, at(100, 100), 20, Region::Other),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::End, 3, at(100, 100), 30, Region::Other),
        None
    );
    // An end past the slop with no move before it is no click either.
    ev(&mut t, Phase::Start, 4, at(100, 100), 0, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::End, 4, at(100, 120), 30, Region::Map),
        None
    );
}

/// The slop is a distance of 10 view pixels: (6, 8) is exactly 10 and
/// still a tap; (8, 7) is past it.
#[test]
fn the_slop_is_ten_pixels_of_distance() {
    assert_eq!(SLOP, 10);
    assert!(!past_slop(at(0, 0), at(6, 8)));
    assert!(!past_slop(at(0, 0), at(-10, 0)));
    assert!(past_slop(at(0, 0), at(8, 7)));
    assert!(past_slop(at(0, 0), at(0, 11)));
}

/// A drag on the map pans with the finger: once past the slop, by the
/// whole way from the touch point, then by each move; the view centre
/// moves against the finger, so the content stays under it.
#[test]
fn a_drag_on_the_map_pans_with_the_finger() {
    let mut t = Touches::new();
    ev(&mut t, Phase::Start, 7, at(300, 200), 0, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::Move, 7, at(305, 200), 5, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::Move, 7, at(320, 190), 10, Region::Map),
        Some(Gesture::Pan((-20, 10)))
    );
    assert_eq!(
        ev(&mut t, Phase::Move, 7, at(330, 150), 20, Region::Map),
        Some(Gesture::Pan((-10, 40)))
    );
    // No move, no pan.
    assert_eq!(
        ev(&mut t, Phase::Move, 7, at(330, 150), 25, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::End, 7, at(330, 150), 30, Region::Map),
        None
    );
    // The pans add up to the touch point less the end point.
    let mut sum = (0, 0);
    ev(&mut t, Phase::Start, 8, at(50, 60), 0, Region::Map);
    for (i, p) in [at(40, 70), at(10, 90), at(-30, 150), at(0, 100)]
        .into_iter()
        .enumerate()
    {
        if let Some(Gesture::Pan(d)) = ev(&mut t, Phase::Move, 8, p, i as i64, Region::Map) {
            sum = (sum.0 + d.0, sum.1 + d.1);
        }
    }
    assert_eq!(sum, (50 - 0, 60 - 100));
}

/// The region is the first finger's at its touch point: a drag that
/// starts on the map and runs over the sidebar still pans.
#[test]
fn the_region_is_taken_at_the_touch_point() {
    let mut t = Touches::new();
    ev(&mut t, Phase::Start, 1, at(300, 200), 0, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(340, 200), 5, Region::Bar),
        Some(Gesture::Pan((-40, 0)))
    );
    // A drag on the other screens does nothing.
    ev(&mut t, Phase::End, 1, at(340, 200), 9, Region::Bar);
    ev(&mut t, Phase::Start, 2, at(300, 200), 0, Region::Other);
    assert_eq!(
        ev(&mut t, Phase::Move, 2, at(300, 300), 5, Region::Other),
        None
    );
}

/// A vertical drag on the sidebar scrolls the strip one row per row
/// height, the content following the finger; sideways movement does not.
#[test]
fn a_drag_on_the_sidebar_steps_rows() {
    let mut t = Touches::new();
    ev(&mut t, Phase::Start, 1, at(900, 400), 0, Region::Bar);
    // Past the slop: 12 pixels down, no whole row yet.
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(900, 412), 1, Region::Bar),
        None
    );
    // 12 + 50 = 62: one row down (toward the first row: a positive wheel).
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(900, 462), 2, Region::Bar),
        Some(Gesture::Rows(1))
    );
    // Two rows at once for a fast move (6 + 112 = 118).
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(900, 574), 3, Region::Bar),
        Some(Gesture::Rows(2))
    );
    // Sideways: nothing.
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(800, 574), 4, Region::Bar),
        None
    );
    // Up: the count turns, and a row up is a negative wheel.
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(800, 530), 5, Region::Bar),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(800, 510), 6, Region::Bar),
        Some(Gesture::Rows(-1))
    );
}

/// Pixel deltas gather into rows: one row per pitch in one direction,
/// the rest kept, a turn starting again.
#[test]
fn pixels_gather_into_rows() {
    let mut s = Strip::default();
    let steps: Vec<i32> = [20, 20, 20, 20, 20, 20]
        .iter()
        .map(|d| s.add(*d, 56))
        .collect();
    assert_eq!(steps, [0, 0, 1, 0, 0, 1]);
    assert_eq!(s.add(-20, 56), 0, "a turn drops the gathered rest");
    assert_eq!(s.add(-40, 56), -1);
    assert_eq!(s.add(-300, 56), -5);
    // A pitch of 0 counts as 1.
    assert_eq!(Strip::default().add(3, 0), 3);
}

/// A touchpad's fractional deltas carry over, so a slow swipe moves.
#[test]
fn fractions_carry_between_events() {
    let mut c = Carry::default();
    let got: Vec<(i32, i32)> = (0..4).map(|_| c.take((0.4, -0.75))).collect();
    assert_eq!(got, [(0, 0), (0, -1), (1, -1), (0, -1)]);
    assert_eq!(c.take((f64::NAN, 2.5)).0, 0);
    assert_eq!(c.take((0.0, 0.0)), (0, 0));
}

/// Two fingers down and up together within the slop and 300 ms: a right
/// click at the midpoint of their touch points.
#[test]
fn a_two_finger_tap_is_a_right_click_at_the_midpoint() {
    let mut t = Touches::new();
    ev(&mut t, Phase::Start, 1, at(100, 100), 1000, Region::Map);
    ev(&mut t, Phase::Start, 2, at(201, 140), 1040, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::Move, 2, at(205, 143), 1100, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::End, 1, at(101, 100), 1200, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::End, 2, at(205, 143), 1300, Region::Map),
        Some(Gesture::Click(Button::Right, at(150, 120)))
    );
    assert!(!t.active());
}

/// A two-finger touch that lasts too long, or that moves past the slop,
/// is no tap; nor is a three-finger one.
#[test]
fn a_slow_moved_or_three_finger_touch_is_no_tap() {
    let mut t = Touches::new();
    ev(&mut t, Phase::Start, 1, at(100, 100), 0, Region::Map);
    ev(&mut t, Phase::Start, 2, at(200, 100), 10, Region::Map);
    ev(&mut t, Phase::End, 1, at(100, 100), 200, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::End, 2, at(200, 100), 311, Region::Map),
        None
    );

    ev(&mut t, Phase::Start, 1, at(100, 100), 0, Region::Map);
    ev(&mut t, Phase::Start, 2, at(200, 100), 10, Region::Map);
    // A pinch: no pan and no tap.
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(80, 100), 20, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(100, 100), 30, Region::Map),
        None
    );
    ev(&mut t, Phase::End, 1, at(100, 100), 40, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::End, 2, at(200, 100), 50, Region::Map),
        None
    );

    ev(&mut t, Phase::Start, 1, at(100, 100), 0, Region::Map);
    ev(&mut t, Phase::Start, 2, at(200, 100), 10, Region::Map);
    ev(&mut t, Phase::Start, 3, at(300, 100), 20, Region::Map);
    ev(&mut t, Phase::End, 1, at(100, 100), 30, Region::Map);
    ev(&mut t, Phase::End, 2, at(200, 100), 40, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::End, 3, at(300, 100), 50, Region::Map),
        None
    );
    // The next touch is new again.
    ev(&mut t, Phase::Start, 4, at(1, 2), 60, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::End, 4, at(1, 2), 70, Region::Map),
        Some(Gesture::Click(Button::Left, at(1, 2)))
    );
}

/// A finger added to a running drag is ignored: the first finger keeps
/// panning, and nothing clicks when they lift.
#[test]
fn a_finger_added_to_a_drag_is_ignored() {
    let mut t = Touches::new();
    ev(&mut t, Phase::Start, 1, at(100, 100), 0, Region::Map);
    ev(&mut t, Phase::Move, 1, at(130, 100), 5, Region::Map);
    ev(&mut t, Phase::Start, 2, at(400, 100), 10, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::Move, 2, at(450, 100), 15, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(140, 100), 20, Region::Map),
        Some(Gesture::Pan((-10, 0)))
    );
    assert_eq!(
        ev(&mut t, Phase::End, 1, at(140, 100), 25, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::End, 2, at(450, 100), 30, Region::Map),
        None
    );
}

/// A cancelled touch gives nothing, and the next touch starts afresh.
#[test]
fn a_cancel_ends_the_gesture() {
    let mut t = Touches::new();
    ev(&mut t, Phase::Start, 1, at(100, 100), 0, Region::Map);
    assert_eq!(
        ev(&mut t, Phase::Cancel, 1, at(100, 100), 5, Region::Map),
        None
    );
    assert!(!t.active());
    assert_eq!(
        ev(&mut t, Phase::End, 1, at(100, 100), 6, Region::Map),
        None
    );
    assert_eq!(
        ev(&mut t, Phase::Move, 1, at(200, 100), 7, Region::Map),
        None
    );
}

/// Mouse events are dropped while a finger is down and for a while after
/// the last touch event, and not before any touch.
#[test]
fn mouse_events_are_quiet_around_touches() {
    let mut t = Touches::new();
    assert!(!t.mouse_quiet(0));
    ev(&mut t, Phase::Start, 1, at(1, 1), 1000, Region::Other);
    assert!(t.mouse_quiet(99_999), "a finger is down");
    ev(&mut t, Phase::End, 1, at(1, 1), 2000, Region::Other);
    assert!(t.mouse_quiet(2000 + MOUSE_QUIET_MS - 1));
    assert!(!t.mouse_quiet(2000 + MOUSE_QUIET_MS));
}
