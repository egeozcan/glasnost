//! Touch and touchpad scrolling through the windowless client on synthetic
//! data (design: touch and touchpad scrolling (lead's request
//! 2026-11-10)): pans of the view, the icon strip's rows, taps as clicks,
//! the edge-scroll exclusion and the setting. The specs cited are the
//! paths the gestures go through unchanged.

use super::access_tests::{
    client, deploy_base, key, own, session, session_mut, start_by_keyboard, steps,
};
use super::touch::{MOUSE_QUIET_MS, Phase};
use super::*;
use glasnost_sim::rules::Kind;

const SIZE: (i32, i32) = (1024, 768);

/// A client in a skirmish, the view on its start.
fn game() -> Client {
    let mut c = client(SIZE, 100);
    start_by_keyboard(&mut c);
    c
}

/// A client in a skirmish past its starting order block, with the view in
/// the middle of F5's limits.
fn panning() -> Client {
    let mut c = game();
    unblocked(&mut c);
    middle(&mut c);
    c
}

/// Puts the view centre in the middle of F5's limits, where a small move in
/// any direction is free.
fn middle(c: &mut Client) {
    let s = session_mut(c);
    s.set_centre((-1_000_000, -1_000_000));
    let low = s.centre();
    s.set_centre((1_000_000, 1_000_000));
    let high = s.centre();
    let start = ((low.0 + high.0) / 2, (low.1 + high.1) / 2);
    s.set_centre(start);
    // A pan of 50 pixels each way must be unlimited here.
    for d in [(50, 0), (-50, 0), (0, 50), (0, -50)] {
        s.set_centre((start.0 + d.0, start.1 + d.1));
        assert_eq!(
            s.centre(),
            (start.0 + d.0, start.1 + d.1),
            "the synthetic start view is away from the limits"
        );
    }
    s.set_centre(start);
}

fn touch(c: &mut Client, phase: Phase, id: u64, p: Point, ms: i64) {
    c.touch(phase, id, p, ms, Modifiers::default());
}

/// Steps the game past its starting order block, then selects the local
/// base unit as a click on it does.
fn selected(c: &mut Client) {
    unblocked(c);
    let mcv = *own(c, Kind::Vehicle).first().expect("a base unit");
    let cell = session(c).play.world().obj(mcv).unwrap().pos.cell();
    session_mut(c)
        .play
        .click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    assert_eq!(session(c).play.selection, vec![mcv]);
}

/// Steps the game until orders are no longer blocked (S-0201 claim 14).
fn unblocked(c: &mut Client) {
    steps(c, 400, |c| {
        let w = &session(c).play.game.world;
        w.tick > w.orders_blocked_through
    });
}

/// The local house's pending orders.
fn pending(c: &Client) -> Vec<glasnost_sim::orders::Order> {
    let p = &session(c).play;
    p.world().houses[usize::from(p.me().0)].pending.clone()
}

/// spec: S-0101 claim 8 (every request is limited by F5) — a one-finger drag past the slop pans the view
/// with the finger: the centre moves by the touch point less the finger's point.
#[test]
fn a_touch_drag_on_the_map_pans_with_the_finger() {
    let mut c = panning();
    let before = session(&c).centre();
    let p = session(&c).map_rect().centre();
    touch(&mut c, Phase::Start, 1, p, 0);
    touch(&mut c, Phase::Move, 1, Point::new(p.x + 4, p.y + 3), 10);
    assert_eq!(session(&c).centre(), before, "within the slop");
    touch(&mut c, Phase::Move, 1, Point::new(p.x + 30, p.y - 20), 20);
    touch(&mut c, Phase::Move, 1, Point::new(p.x + 40, p.y - 25), 30);
    assert_eq!(session(&c).centre(), (before.0 - 40, before.1 + 25));
    touch(&mut c, Phase::End, 1, Point::new(p.x + 40, p.y - 25), 40);
    assert!(session(&c).play.selection.is_empty(), "a drag is no click");
    // F5 limits a pan as any request: a huge drag ends where set_centre puts it.
    let want = {
        let s = session_mut(&mut c);
        let now = s.centre();
        s.set_centre((now.0 + 100_000, now.1));
        let limited = s.centre();
        s.set_centre(now);
        limited
    };
    touch(&mut c, Phase::Start, 2, Point::new(500, 300), 100);
    touch(&mut c, Phase::Move, 2, Point::new(-99_500, 300), 110);
    assert_eq!(session(&c).centre(), want);
}

/// spec: S-0201 claim 14 (edge scrolling is suppressed during an order block) — our pans follow the
/// edge's rule: none during an order block, none while the options menu is open.
#[test]
fn pans_are_held_to_the_order_block_and_the_menu() {
    let mut c = panning();
    let before = session(&c).centre();
    let tick = session(&c).play.world().tick;
    session_mut(&mut c).play.game.world.orders_blocked_through = tick + 10;
    let p = session(&c).map_rect().centre();
    touch(&mut c, Phase::Start, 1, p, 0);
    touch(&mut c, Phase::Move, 1, Point::new(p.x + 30, p.y), 10);
    touch(&mut c, Phase::End, 1, Point::new(p.x + 30, p.y), 20);
    c.pointer_moved(p);
    c.pixel_wheel((25.0, 0.0));
    assert_eq!(session(&c).centre(), before, "no pan in an order block");
    session_mut(&mut c).play.game.world.orders_blocked_through = tick - 1;
    c.pixel_wheel((25.0, 0.0));
    assert_eq!(session(&c).centre(), (before.0 - 25, before.1));
    // The options menu: no pan.
    key(&mut c, Key::Escape);
    assert!(session(&c).paused);
    c.pixel_wheel((25.0, 0.0));
    assert_eq!(session(&c).centre(), (before.0 - 25, before.1));
}

/// A touchpad swipe over the battlefield pans with the fingers (winit's
/// delta is the content's movement); its fractions carry over.
#[test]
fn a_touchpad_swipe_over_the_map_pans() {
    let mut c = panning();
    let before = session(&c).centre();
    c.pointer_moved(session(&c).map_rect().centre());
    c.pixel_wheel((-12.5, 30.0));
    assert_eq!(session(&c).centre(), (before.0 + 12, before.1 - 30));
    c.pixel_wheel((-0.5, 0.0));
    assert_eq!(session(&c).centre(), (before.0 + 13, before.1 - 30));
    assert_eq!(session(&c).play.scroll, 0, "the strip stays");
}

/// Pads the shown tab's strip with copies of its first icon until it
/// overflows, as `page_requests_and_wheel_on_an_overflowing_strip` does.
fn long_strip(c: &mut Client) {
    deploy_base(c);
    let s = session_mut(c);
    s.play.tab_click(0);
    let me = s.play.me();
    let icons = &mut s.play.game.world.houses[usize::from(me.0)].econ.tech.icons[0];
    let first = icons[0];
    while icons.len() < 60 {
        icons.push(first);
    }
}

/// spec: S-0411 claim 18 (a wheel event moves the strip one row, upward for a positive delta) — a
/// swipe over the sidebar steps one row per row height gathered, and a
/// finger's vertical drag on the sidebar does the same, the content
/// following the finger.
#[test]
fn swipes_and_drags_on_the_sidebar_step_the_strip_by_rows() {
    let mut c = game();
    long_strip(&mut c);
    let pitch = session(&c).strip_pitch();
    assert_eq!(pitch, 56, "our layout's row with its gap at 100%");
    let bar = Point::new(SIZE.0 - 40, SIZE.1 / 2);
    c.pointer_moved(bar);
    session_mut(&mut c).play.scroll = 5;
    // Up by two and a half rows: two rows down the strip.
    c.pixel_wheel((0.0, -2.5 * f64::from(pitch)));
    assert_eq!(session(&c).play.scroll, 7);
    // Down by half a row: the count turns and starts again.
    c.pixel_wheel((0.0, f64::from(pitch) / 2.0));
    assert_eq!(session(&c).play.scroll, 7);
    c.pixel_wheel((0.0, f64::from(pitch) / 2.0));
    assert_eq!(session(&c).play.scroll, 6);
    let centre = session(&c).centre();
    // A finger dragged up the sidebar by three rows and a bit.
    touch(&mut c, Phase::Start, 1, bar, 0);
    touch(
        &mut c,
        Phase::Move,
        1,
        Point::new(bar.x, bar.y - 3 * pitch - 20),
        10,
    );
    assert_eq!(session(&c).play.scroll, 9);
    touch(&mut c, Phase::Move, 1, Point::new(bar.x, bar.y - pitch), 20);
    assert_eq!(session(&c).play.scroll, 7);
    touch(&mut c, Phase::End, 1, Point::new(bar.x, bar.y - pitch), 30);
    assert_eq!(session(&c).centre(), centre, "the view stays");
}

/// A tap is a left click at its point and a two-finger tap a right click at
/// the fingers' midpoint: each leaves the game as the mouse's click at that
/// pixel does, on a twin game.
#[test]
fn taps_click() {
    let mut c = game();
    selected(&mut c);
    let mut twin = game();
    selected(&mut twin);
    let ground = session(&c).map_rect().centre();
    let ground = Point::new(ground.x - 60, ground.y + 40);
    // A tap that wobbles within the slop.
    touch(&mut c, Phase::Start, 1, ground, 0);
    touch(
        &mut c,
        Phase::Move,
        1,
        Point::new(ground.x + 3, ground.y + 2),
        50,
    );
    touch(
        &mut c,
        Phase::End,
        1,
        Point::new(ground.x + 3, ground.y + 2),
        120,
    );
    super::access_tests::click_at(&mut twin, ground);
    assert!(
        !pending(&twin).is_empty(),
        "the mouse's click gave an order"
    );
    assert_eq!(pending(&c), pending(&twin));
    assert_eq!(session(&c).play.selection, session(&twin).play.selection);
    // Two fingers tapped either side of a pixel, and a right click there.
    let q = Point::new(ground.x + 50, ground.y - 30);
    let before = (pending(&twin), session(&twin).play.selection.clone());
    touch(&mut c, Phase::Start, 2, Point::new(q.x - 20, q.y - 5), 1000);
    touch(&mut c, Phase::Start, 3, Point::new(q.x + 20, q.y + 5), 1030);
    touch(&mut c, Phase::End, 2, Point::new(q.x - 20, q.y - 5), 1150);
    touch(&mut c, Phase::End, 3, Point::new(q.x + 20, q.y + 5), 1200);
    twin.pointer_moved(q);
    twin.pointer_down(Button::Right, q);
    twin.pointer_up(Button::Right, q, Modifiers::default());
    let after = (pending(&twin), session(&twin).play.selection.clone());
    assert_ne!(before, after, "the right click changed the game");
    assert_eq!((pending(&c), session(&c).play.selection.clone()), after);
    // A tap on the main menu presses its control.
    let mut m = client(SIZE, 100);
    let r = m
        .ui_scale()
        .rect(m.menu_targets.rect_of(&MenuHit::Settings).unwrap());
    touch(&mut m, Phase::Start, 1, r.centre(), 0);
    touch(&mut m, Phase::End, 1, r.centre(), 80);
    assert_eq!(m.shell.screen, Screen::Settings);
}

/// Edge scrolling (S-0101 claim 12) never starts from a touch: after a tap
/// at the screen's edge no pointer stays there, and a finger held at the
/// edge during a drag moves the view by the drag alone; a mouse pointer at
/// the same pixel does scroll.
/// spec: S-0101 claim 12 (the outermost client column activates edge scrolling)
#[test]
fn no_edge_scroll_starts_from_a_touch() {
    let edge = Point::new(0, SIZE.1 / 2);
    // The control: a mouse resting there scrolls.
    let mut mouse = panning();
    let before = session(&mouse).centre();
    mouse.pointer_moved(edge);
    for k in 1..=20 {
        mouse.update(1_000 + 100 * k);
    }
    assert!(session(&mouse).play.world().tick > 0);
    assert_ne!(
        session(&mouse).centre(),
        before,
        "a mouse at the edge scrolls"
    );

    let mut c = panning();
    let before = session(&c).centre();
    touch(&mut c, Phase::Start, 1, edge, 0);
    touch(&mut c, Phase::End, 1, edge, 50);
    for k in 1..=20 {
        c.update(1_000 + 100 * k);
    }
    assert_eq!(session(&c).centre(), before, "a tap at the edge");
    // A drag that ends with the finger held at the edge.
    let p = Point::new(40, edge.y);
    let to = edge;
    touch(&mut c, Phase::Start, 2, p, 3_000);
    touch(&mut c, Phase::Move, 2, to, 3_010);
    let panned = session(&c).centre();
    assert_eq!(panned, (before.0 + (p.x - to.x), before.1));
    for k in 1..=20 {
        c.update(3_100 + 100 * k);
    }
    assert_eq!(session(&c).centre(), panned, "a finger held at the edge");
    touch(&mut c, Phase::End, 2, to, 6_000);
}

/// spec: S-0411 claim 18 (a wheel event moves the strip one row, upward for a positive delta) — a line
/// delta is one row with the setting on or off, and with the setting off a
/// pixel delta is one row too, wherever the pointer is.
#[test]
fn line_deltas_and_the_setting_off_keep_one_row_per_event() {
    let mut c = game();
    long_strip(&mut c);
    let map = session(&c).map_rect().centre();
    let bar = Point::new(SIZE.0 - 40, SIZE.1 / 2);
    session_mut(&mut c).play.scroll = 5;
    let centre = session(&c).centre();
    c.pointer_moved(map);
    c.wheel_event(WheelDelta::Lines(-3.0));
    assert_eq!(
        session(&c).play.scroll,
        6,
        "a line delta over the map: one row"
    );
    c.pointer_moved(bar);
    c.wheel_event(WheelDelta::Lines(0.25));
    assert_eq!(session(&c).play.scroll, 5);
    c.settings.touch_scroll = false;
    c.pointer_moved(map);
    c.wheel_event(WheelDelta::Pixels(40.0, -200.0));
    assert_eq!(session(&c).play.scroll, 6, "off: a pixel delta is one row");
    assert_eq!(session(&c).centre(), centre, "off: no pan");
    c.pointer_moved(bar);
    c.wheel_event(WheelDelta::Pixels(0.0, 3.0));
    assert_eq!(session(&c).play.scroll, 5);
    c.wheel_event(WheelDelta::Pixels(0.0, 0.0));
    assert_eq!(session(&c).play.scroll, 5, "no delta, no row");
}

/// The window drops mouse events around touches only with the setting on;
/// with it off, touches are ignored as they were.
#[test]
fn the_setting_off_keeps_todays_paths() {
    let mut c = game();
    let p = session(&c).map_rect().centre();
    touch(&mut c, Phase::Start, 1, p, 1_000);
    assert!(
        c.mouse_quiet(1_000 + MOUSE_QUIET_MS + 5_000),
        "a finger is down"
    );
    touch(&mut c, Phase::End, 1, p, 1_100);
    assert!(c.mouse_quiet(1_100 + MOUSE_QUIET_MS - 1));
    assert!(!c.mouse_quiet(1_100 + MOUSE_QUIET_MS));
    c.settings.touch_scroll = false;
    touch(&mut c, Phase::Start, 2, p, 2_000);
    assert!(!c.mouse_quiet(2_000), "off: no mouse event is dropped");
    touch(&mut c, Phase::End, 2, p, 2_050);
    // Off: a drag does not pan and a tap does not click.
    let centre = session(&c).centre();
    touch(&mut c, Phase::Start, 3, p, 3_000);
    touch(&mut c, Phase::Move, 3, Point::new(p.x + 60, p.y), 3_010);
    touch(&mut c, Phase::End, 3, Point::new(p.x + 60, p.y), 3_020);
    assert_eq!(session(&c).centre(), centre);
    selected(&mut c);
    touch(
        &mut c,
        Phase::Start,
        4,
        Point::new(p.x - 60, p.y + 40),
        4_000,
    );
    touch(&mut c, Phase::End, 4, Point::new(p.x - 60, p.y + 40), 4_050);
    assert!(pending(&c).is_empty(), "off: a tap is no click");
    // The setting is on by default and is a row of the settings screen.
    assert!(Settings::default().touch_scroll);
    let mut s = client(SIZE, 100);
    let r = s.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    super::access_tests::click(&mut s, r);
    let _ = s.frame();
    let step = s
        .settings_targets
        .rect_of(&SettingsHit::Step(glasnost_ui::settings::Row::Touch, 1))
        .unwrap();
    super::access_tests::click(&mut s, step);
    assert!(!s.settings.touch_scroll);
}
