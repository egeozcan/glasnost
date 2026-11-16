//! A battlefield press held while the pointer leaves the game's window
//! (S-0214 claim 34, #150), through the windowless client on synthetic
//! data. The host here is one that keeps reporting the pointer and the
//! release outside the window, as the window layer's hosts do (claim 34
//! (f)): points outside the window reach the client as they are.

use super::access_tests::{client, own, session, session_mut, start_by_keyboard_with, steps};
use super::*;
use crate::game::capture::Click;
use glasnost_ai::fixture;
use glasnost_map::CellXY;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::ObjectId;
use std::sync::Arc;

const SIZE: (i32, i32) = (1024, 768);

/// A client in a skirmish on `rules`, past its starting order block.
fn game_with(rules: Arc<SimRules>) -> Client {
    let mut c = client(SIZE, 100);
    start_by_keyboard_with(&mut c, rules);
    steps(&mut c, 400, |c| {
        let w = &session(c).play.game.world;
        w.tick > w.orders_blocked_through
    });
    c
}

/// A client in a skirmish on the synthetic rules, past its order block.
fn game() -> Client {
    game_with(fixture::rules())
}

/// The synthetic rules with the off-view release quirk off.
fn origin_off() -> Arc<SimRules> {
    let mut rs = fixture::ruleset();
    rs.globals.quirks.offview_release_origin_cell = false;
    Arc::new(SimRules::new(&rs, "synthetic-skirmish"))
}

/// The local base unit, selected as a click on it selects it.
fn selected(c: &mut Client) -> ObjectId {
    let mcv = *own(c, Kind::Vehicle).first().expect("a base unit");
    let cell = session(c).play.world().obj(mcv).unwrap().pos.cell();
    session_mut(c)
        .play
        .click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    assert_eq!(session(c).play.selection, vec![mcv]);
    mcv
}

/// The battlefield clicks the game noted so far.
fn clicks(c: &Client) -> Vec<Click> {
    session(c).play.capture.clicks().copied().collect()
}

/// The last click noted, as (right button, cell, object).
fn last(c: &Client) -> Option<(bool, CellXY, Option<u32>)> {
    clicks(c).last().map(|k| (k.right, k.cell, k.object))
}

fn up(c: &mut Client, b: Button, p: Point) {
    c.pointer_up(b, p, Modifiers::default());
}

/// spec: S-0214 claim 34 (a) (the pointer leaving or re-entering the window is not an event of its
/// own: the press keeps its anchor, and a release inside ends it as usual)
#[test]
fn claim_34_a_a_press_survives_the_pointer_leaving_and_coming_back() {
    let mut c = game();
    selected(&mut c);
    let n = clicks(&c).len();
    let p = Point::new(400, 300);
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_left();
    let back = Point::new(401, 300);
    c.pointer_moved(back);
    let cell = session_mut(&mut c).under(back).0;
    up(&mut c, Button::Left, back);
    assert_eq!(clicks(&c).len(), n + 1, "the click is applied");
    assert_eq!(last(&c).map(|k| k.1), Some(cell));
}

/// spec: S-0214 claim 34 (c) and Edge cases (quirks.offview_release_origin_cell on, the original: a
/// click released outside the window to the left of or above the view on either axis, beyond its
/// upper-right corner included, acts on the cell at map cell coordinates (0, 0) with no object)
#[test]
fn claim_34_c_a_click_released_left_of_or_above_the_view_acts_on_cell_0_0() {
    let m = session(&game()).map_rect();
    let cases = [
        // Pressed by the left edge, sampled outside, released there: the
        // limited pointer stays within 4 pixels of the press.
        (
            Point::new(2, 300),
            Some(Point::new(-30, 300)),
            Point::new(-30, 300),
        ),
        (
            Point::new(300, 2),
            Some(Point::new(300, -20)),
            Point::new(300, -20),
        ),
        // Released beyond the upper-right corner with no sample outside.
        (
            Point::new(m.right() - 3, 2),
            None,
            Point::new(SIZE.0 + 20, -20),
        ),
    ];
    for (press, outside, release) in cases {
        let mut c = game();
        selected(&mut c);
        let n = clicks(&c).len();
        c.pointer_moved(press);
        c.pointer_down(Button::Left, press);
        c.pointer_left();
        if let Some(o) = outside {
            c.pointer_moved(o);
        }
        up(&mut c, Button::Left, release);
        assert_eq!(clicks(&c).len(), n + 1, "{release:?}: one click");
        assert_eq!(
            last(&c),
            Some((false, CellXY::new(0, 0), None)),
            "{release:?}"
        );
    }
}

/// spec: S-0214 Edge cases (quirks.offview_release_origin_cell off: such a release gives no order and
/// does not change the selection)
#[test]
fn claim_34_c_with_the_switch_off_such_a_release_does_nothing() {
    let mut c = game_with(origin_off());
    let mcv = selected(&mut c);
    let n = clicks(&c).len();
    let orders = session(&c).play.game.replay().orders.len();
    let press = Point::new(2, 300);
    c.pointer_moved(press);
    c.pointer_down(Button::Left, press);
    c.pointer_left();
    c.pointer_moved(Point::new(-30, 300));
    up(&mut c, Button::Left, Point::new(-30, 300));
    assert_eq!(clicks(&c).len(), n, "no click");
    assert_eq!(session(&c).play.selection, vec![mcv], "the selection stays");
    assert_eq!(session(&c).play.game.replay().orders.len(), orders);
}

/// spec: S-0214 claim 34 (c) (a click released outside the window neither to the left of nor above
/// the view resolves at the release pixel, projected beyond the view's edge, with no object under it)
#[test]
fn claim_34_c_a_click_released_right_of_or_below_the_window_acts_at_its_pixel() {
    for release in [Point::new(500, SIZE.1 + 40), Point::new(SIZE.0 + 50, 400)] {
        let mut c = game();
        selected(&mut c);
        let n = clicks(&c).len();
        let press = Point::new(500, 400);
        c.pointer_moved(press);
        c.pointer_down(Button::Left, press);
        c.pointer_left();
        let cell = session_mut(&mut c).under(release).0;
        up(&mut c, Button::Left, release);
        assert_eq!(clicks(&c).len(), n + 1, "{release:?}: one click");
        assert_eq!(last(&c), Some((false, cell, None)), "{release:?}");
    }
}

/// spec: S-0214 claim 34 (b) (a band box released outside the window is applied under claim 19; an
/// unmodified box with no live box candidate continues as the ordinary release click at the release
/// pixel, which resolves as (c) gives) and claim 19 with the empty-box quirk on. The synthetic game
/// draws no mobile object, so every box here has no live candidate.
#[test]
fn claim_34_b_a_band_box_released_outside_continues_as_its_click() {
    let press = Point::new(400, 300);
    let drag = |c: &mut Client| {
        selected(c);
        c.pointer_moved(press);
        c.pointer_down(Button::Left, press);
        c.pointer_moved(Point::new(380, 300));
    };
    // Inside the window, the empty box is a click at its release pixel.
    let mut c = game();
    drag(&mut c);
    let n = clicks(&c).len();
    let end = Point::new(380, 300);
    let cell = session_mut(&mut c).under(end).0;
    up(&mut c, Button::Left, end);
    assert_eq!(clicks(&c).len(), n + 1, "the box inside");
    assert_eq!(last(&c).map(|k| k.1), Some(cell));
    // Dragged out of the window and released there, left of the view.
    let mut c = game();
    drag(&mut c);
    let n = clicks(&c).len();
    c.pointer_left();
    c.pointer_moved(Point::new(-50, 300));
    up(&mut c, Button::Left, Point::new(-50, 300));
    assert_eq!(clicks(&c).len(), n + 1, "the box outside");
    assert_eq!(last(&c), Some((false, CellXY::new(0, 0), None)));
    // Released below the window.
    let mut c = game();
    drag(&mut c);
    let n = clicks(&c).len();
    c.pointer_left();
    let below = Point::new(380, SIZE.1 + 30);
    c.pointer_moved(below);
    let cell = session_mut(&mut c).under(below).0;
    up(&mut c, Button::Left, below);
    assert_eq!(clicks(&c).len(), n + 1, "the box below");
    assert_eq!(last(&c), Some((false, cell, None)));
}

/// spec: S-0214 claim 34 (d) (a right release outside the window without a scroll drag is the right
/// click of claim 26: it clears the selection, whatever the release pixel)
#[test]
fn claim_34_d_a_right_release_outside_is_a_right_click() {
    let mut c = game();
    selected(&mut c);
    let press = Point::new(400, 300);
    c.pointer_moved(press);
    c.pointer_down(Button::Right, press);
    c.pointer_left();
    up(&mut c, Button::Right, Point::new(-30, 300));
    assert!(
        session(&c).play.selection.is_empty(),
        "the selection is cleared"
    );
}

/// Puts the view centre in the middle of F5's limits.
fn middle(c: &mut Client) -> (i32, i32) {
    let s = session_mut(c);
    s.set_centre((-1_000_000, -1_000_000));
    let low = s.centre();
    s.set_centre((1_000_000, 1_000_000));
    let high = s.centre();
    let start = ((low.0 + high.0) / 2, (low.1 + high.1) / 2);
    s.set_centre(start);
    start
}

/// spec: S-0214 claim 34 (a) and (d) (an active scroll drag goes on with the limited pointer as its
/// sampled pointer: the pointer's pixel limited to the client area)
#[test]
fn claim_34_d_a_scroll_drag_samples_the_limited_pointer() {
    let drag = |leave: bool| {
        let mut c = game();
        let start = middle(&mut c);
        let press = Point::new(400, 300);
        c.pointer_moved(press);
        c.pointer_down(Button::Right, press);
        c.pointer_moved(Point::new(380, 300));
        if leave {
            c.pointer_left();
            c.pointer_moved(Point::new(-100, 300));
        } else {
            c.pointer_moved(Point::new(0, 300));
        }
        session_mut(&mut c).scroll_cycle(0, [false; 4], true);
        (start, session(&c).centre())
    };
    let (start, inside) = drag(false);
    assert_ne!(inside, start, "the drag scrolls");
    assert_eq!(
        drag(true),
        (start, inside),
        "outside, at the limited pointer"
    );
}

/// spec: S-0214 claim 34 (f) (with either host, the game's window losing the input focus while a
/// press is held abandons it: nothing is applied and the later release does nothing)
#[test]
fn claim_34_f_losing_the_focus_abandons_the_press() {
    let mut c = game();
    let mcv = selected(&mut c);
    let n = clicks(&c).len();
    let p = Point::new(400, 300);
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.focus(false);
    c.focus(true);
    up(&mut c, Button::Left, p);
    assert_eq!(clicks(&c).len(), n, "no click");
    assert_eq!(session(&c).play.selection, vec![mcv]);
}
