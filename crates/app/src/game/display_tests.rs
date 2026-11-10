//! The display scale's factor, conversions and upscaling (design: readable
//! text and display scale (lead's request 2026-11-10)).

use super::*;

/// A window large enough for every factor (a 4K screen).
const BIG: (i32, i32) = (3840, 2400);

#[test]
fn the_factor_follows_the_host_scale_under_auto_and_the_setting_otherwise() {
    let scales = [1.0, 1.25, 1.5, 2.0, 2.25, 3.0];
    let auto = [1, 1, 2, 2, 2, 3];
    for (s, want) in scales.iter().zip(auto) {
        assert_eq!(factor(AUTO, *s, BIG), want, "Auto at {s}");
        for fixed in [1u8, 2, 3] {
            assert_eq!(factor(fixed, *s, BIG), i32::from(fixed), "{fixed}x at {s}");
        }
    }
    // A host that reports nothing usable, or more than the largest factor.
    assert_eq!(factor(AUTO, 0.0, BIG), 1);
    assert_eq!(factor(AUTO, f64::NAN, BIG), 1);
    assert_eq!(factor(AUTO, 4.0, BIG), MAX_FACTOR);
    // A setting from a newer or damaged file stays in range.
    assert_eq!(factor(9, 1.0, BIG), MAX_FACTOR);
}

#[test]
fn a_factor_too_large_for_the_window_is_lowered() {
    // 2x on 1280 x 960 keeps 640 x 480; on 1280 x 900 it would not.
    assert_eq!(factor(2, 1.0, (1280, 960)), 2);
    assert_eq!(factor(2, 1.0, (1280, 900)), 1);
    assert_eq!(factor(3, 1.0, (1920, 1200)), 2);
    assert_eq!(factor(AUTO, 3.0, (1920, 1440)), 3);
    assert_eq!(factor(AUTO, 2.0, (600, 400)), 1);
    let d = Display::new((1920, 1200), 1.0, 3);
    assert_eq!(d.label(3), "3x (2x fits)");
    assert_eq!(d.label(AUTO), "Auto (2x)");
    assert_eq!(Display::new(BIG, 1.0, 2).label(2), "2x");
}

#[test]
fn a_two_times_display_shows_the_logical_size_in_view_pixels() {
    let d = Display::new((2560, 1600), 2.0, AUTO);
    assert_eq!(d.factor, 2);
    assert_eq!(d.view(), (1280, 800));
    assert_eq!(d.shown(), (2560, 1600));
    // An odd physical size leaves a remainder of less than one view pixel.
    let d = Display::new((2561, 1601), 2.0, AUTO);
    assert_eq!(d.view(), (1280, 800));
    assert_eq!(d.shown(), (2560, 1600));
    // 1x is the window as it is.
    let d = Display::new((1280, 800), 1.0, AUTO);
    assert_eq!(
        (d.factor, d.view(), d.shown()),
        (1, (1280, 800), (1280, 800))
    );
}

#[test]
fn pointer_and_touch_positions_become_view_pixels_at_two_times() {
    let d = Display::new((2561, 1601), 2.0, AUTO);
    assert_eq!(d.point(0.0, 0.0), Point::new(0, 0));
    assert_eq!(d.point(1.9, 3.0), Point::new(0, 1));
    assert_eq!(d.point(201.0, 99.5), Point::new(100, 49));
    assert_eq!(d.point(2559.0, 1599.0), Point::new(1279, 799));
    // The black remainder and positions just outside reach the edge pixels.
    assert_eq!(d.point(2560.5, 1600.0), Point::new(1279, 799));
    assert_eq!(d.point(-3.0, -0.5), Point::new(0, 0));
    assert_eq!(d.point(f64::NAN, 4.0), Point::new(0, 2));
    // At 1x positions are the pixels they fall in.
    let one = Display::one((800, 600));
    assert_eq!(one.point(10.7, 20.2), Point::new(10, 20));
}

#[test]
fn wheel_pixels_become_view_pixels_and_lines_stay() {
    let d = Display::new((2560, 1600), 2.0, AUTO);
    assert_eq!(d.delta((240.0, -60.0)), (120.0, -30.0));
    assert_eq!(
        d.wheel(WheelDelta::Pixels(240.0, -60.0)),
        WheelDelta::Pixels(120.0, -30.0)
    );
    assert_eq!(d.wheel(WheelDelta::Lines(-1.0)), WheelDelta::Lines(-1.0));
    // A browser's notch at 2x is still told from a swipe in CSS pixels
    // before the conversion, then stays a line step.
    let notch = WheelDelta::from_pixels((0.0, 240.0), d.host_scale, true);
    assert_eq!(d.wheel(notch), WheelDelta::Lines(240.0));
}

#[test]
fn upscaling_repeats_each_pixel_as_a_square() {
    let px = [1, 2, 3, 4, 5, 6, 7, 8];
    let up = upscale(&px, (2, 1), 2);
    assert_eq!(up.len(), 4 * 2 * 4);
    let at = |x: usize, y: usize| &up[(y * 4 + x) * 4..(y * 4 + x) * 4 + 4];
    for y in 0..2 {
        assert_eq!(at(0, y), [1, 2, 3, 4]);
        assert_eq!(at(1, y), [1, 2, 3, 4]);
        assert_eq!(at(2, y), [5, 6, 7, 8]);
        assert_eq!(at(3, y), [5, 6, 7, 8]);
    }
    assert_eq!(upscale(&px, (2, 1), 1), px);
}
