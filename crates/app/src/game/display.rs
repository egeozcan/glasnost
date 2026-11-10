//! The display scale: how the game's view pixels reach the screen (our own
//! client design; design: readable text and display scale (lead's request
//! 2026-11-10)).
//!
//! The whole game picture, battlefield and interface, is drawn in *view
//! pixels*, the pixels every spec'd size, place and metric is given in.
//! The window then shows each view pixel as a square of `factor` ×
//! `factor` screen pixels, nearest-neighbour, from the window's top left
//! corner. A display with a scale factor of 2 (a HiDPI laptop, a Retina
//! Mac, a browser page with `devicePixelRatio` 2) so shows the same game
//! area as a display with a scale factor of 1 and the same logical size,
//! and every sprite and glyph stays crisp.
//!
//! [`Display`] keeps the window's physical size, the host's scale factor
//! and the factor chosen, and converts the host's pointer, touch and wheel
//! coordinates to view pixels; the window layer (`super::window`) converts
//! nothing else. Nothing here reaches the simulation.

use glasnost_ui::geom::{MIN_UI, Point};

use super::touch::WheelDelta;

/// The display-scale setting meaning "follow the display": the host's
/// scale factor, rounded.
// design: readable text and display scale (lead's request 2026-11-10): Auto is the setting's default
pub const AUTO: u8 = 0;

/// The display-scale settings offered, in the order the settings screen
/// steps through them: Auto, then the fixed factors 1, 2 and 3.
// design: readable text and display scale (lead's request 2026-11-10): Auto, 1x, 2x, 3x
pub const CHOICES: [u8; 4] = [AUTO, 1, 2, 3];

/// The largest factor (the last of [`CHOICES`]); Auto never goes past it.
// design: readable text and display scale (lead's request 2026-11-10): at most 3x
pub const MAX_FACTOR: i32 = 3;

/// The factor a setting asks for on a display with the host's
/// `host_scale` (device pixels per logical pixel), before the window's
/// size is taken into account: a fixed setting is its own factor; Auto is
/// `host_scale` rounded to the nearest whole number (halves up), at least
/// 1 and at most [`MAX_FACTOR`].
pub fn wanted(setting: u8, host_scale: f64) -> i32 {
    if setting != AUTO {
        return i32::from(setting).clamp(1, MAX_FACTOR);
    }
    if !host_scale.is_finite() || host_scale <= 0.0 {
        return 1;
    }
    // design: readable text and display scale (lead's request 2026-11-10): Auto rounds the scale factor (1.25 gives 1, 1.5 gives 2, 2.25 gives 2)
    (host_scale.round() as i32).clamp(1, MAX_FACTOR)
}

/// The factor used on a window of `physical` pixels: [`wanted`], lowered
/// (never below 1) until the view keeps at least the smallest interface
/// size the screens are laid out for, `glasnost_ui::geom::MIN_UI`.
pub fn factor(setting: u8, host_scale: f64, physical: (i32, i32)) -> i32 {
    let mut f = wanted(setting, host_scale);
    // design: readable text and display scale (lead's request 2026-11-10): a factor is lowered until the view keeps at least the smallest interface (640 x 480)
    while f > 1 && (physical.0 / f < MIN_UI.0 || physical.1 / f < MIN_UI.1) {
        f -= 1;
    }
    f
}

/// The window as the game sees it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Display {
    /// The window's drawable size in physical (device) pixels.
    pub physical: (i32, i32),
    /// The host's scale factor (device pixels per logical pixel; the
    /// browser's `devicePixelRatio`).
    pub host_scale: f64,
    /// Screen pixels per view pixel, along each axis.
    pub factor: i32,
}

impl Display {
    /// A window of `physical` pixels at the host's `host_scale`, under the
    /// display-scale `setting`.
    pub fn new(physical: (i32, i32), host_scale: f64, setting: u8) -> Self {
        let physical = (physical.0.max(1), physical.1.max(1));
        Self {
            physical,
            host_scale,
            factor: factor(setting, host_scale, physical),
        }
    }

    /// One view pixel per screen pixel (tests, the benchmark).
    pub fn one(physical: (i32, i32)) -> Self {
        Self {
            physical: (physical.0.max(1), physical.1.max(1)),
            host_scale: 1.0,
            factor: 1,
        }
    }

    /// The view's size in view pixels: the whole view pixels that fit.
    pub fn view(&self) -> (i32, i32) {
        (
            (self.physical.0 / self.factor).max(1),
            (self.physical.1 / self.factor).max(1),
        )
    }

    /// The screen rectangle the picture covers, from the top left corner,
    /// in physical pixels; the rest of the window (fewer than `factor`
    /// pixels on the right and at the bottom) stays black.
    pub fn shown(&self) -> (i32, i32) {
        let v = self.view();
        (
            (v.0 * self.factor).min(self.physical.0),
            (v.1 * self.factor).min(self.physical.1),
        )
    }

    /// The view pixel under a host position in physical pixels (a mouse
    /// pointer or a touch): the pixel whose screen square holds it. A
    /// position on the black remainder, or just outside the window, counts
    /// as the nearest edge pixel, so the outermost screen pixels still
    /// reach the view's edge (edge scrolling, S-0101 claim 12, is at the
    /// view's outermost pixel).
    pub fn point(&self, x: f64, y: f64) -> Point {
        let (w, h) = self.view();
        let f = f64::from(self.factor);
        let to = |v: f64, n: i32| {
            if v.is_finite() {
                ((v / f).floor() as i32).clamp(0, n - 1)
            } else {
                0
            }
        };
        Point::new(to(x, w), to(y, h))
    }

    /// A host movement in physical pixels (a touchpad's pixel wheel delta)
    /// in view pixels, so a swipe moves the content as far as the fingers
    /// on the screen.
    pub fn delta(&self, d: (f64, f64)) -> (f64, f64) {
        let f = f64::from(self.factor);
        (d.0 / f, d.1 / f)
    }

    /// A host wheel event in view pixels: a pixel delta through
    /// [`Display::delta`], a line step as it is. The window first tells a
    /// browser's notch from a swipe in CSS pixels
    /// ([`WheelDelta::from_pixels`]), then converts.
    pub fn wheel(&self, d: WheelDelta) -> WheelDelta {
        match d {
            WheelDelta::Pixels(x, y) => {
                let (x, y) = self.delta((x, y));
                WheelDelta::Pixels(x, y)
            }
            lines => lines,
        }
    }

    /// The setting's label on the settings screen: `Auto (2x)` names the
    /// factor Auto chose, and a fixed factor the window lowered says so.
    pub fn label(&self, setting: u8) -> String {
        if setting == AUTO {
            return format!("Auto ({}x)", self.factor);
        }
        if i32::from(setting) != self.factor {
            return format!("{setting}x ({}x fits)", self.factor);
        }
        format!("{setting}x")
    }
}

/// Upscales an RGBA picture of `size` view pixels by `factor`, nearest
/// neighbour: what the window shows (the CPU snapshots of the local lane;
/// the window does the same on the GPU with a viewport).
pub fn upscale(rgba: &[u8], size: (u32, u32), factor: u32) -> Vec<u8> {
    let (w, h) = (size.0 as usize, size.1 as usize);
    let f = factor.max(1) as usize;
    let mut out = vec![0u8; w * f * h * f * 4];
    for y in 0..h * f {
        for x in 0..w * f {
            let from = ((y / f) * w + x / f) * 4;
            let to = (y * w * f + x) * 4;
            out[to..to + 4].copy_from_slice(&rgba[from..from + 4]);
        }
    }
    out
}

#[cfg(test)]
#[path = "display_tests.rs"]
mod tests;
