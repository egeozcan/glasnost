//! Touch and touchpad scrolling: our own input additions, not behaviour of
//! the original (design: touch and touchpad scrolling (lead's request
//! 2026-11-10)).
//!
//! - [`Touches`] turns a touch screen's contacts (an iPad, a touch laptop)
//!   into gestures: one finger dragged past [`SLOP`] pans the view with the
//!   finger, or on the sidebar scrolls the icon strip by rows; a touch that
//!   ends within the slop is a left click; two fingers tapped together are
//!   a right click at their midpoint.
//! - [`Strip`] turns pixels into icon-strip rows, so a swipe steps one row
//!   per row height rather than one row per event.
//! - [`Carry`] keeps the fraction of a touchpad's pixel deltas between
//!   events.
//!
//! The client ([`super::client::Client::touch`] and
//! [`super::client::Client::pixel_wheel`]) carries the gestures out: a pan
//! through `Session::pan` (S-0101 F5's limit, and no pan where edge
//! scrolling is suppressed, S-0201 claim 14), rows through S-0411 claim
//! 18's wheel step, clicks through the mouse's own path. Nothing here
//! reaches the simulation. With the setting off the window never calls in.

use glasnost_ui::geom::Point;

use super::play::Button;

/// How far, in view pixels, a finger may move from where it touched down
/// and still be a tap; past it, a drag.
// design: touch and touchpad scrolling (lead's request 2026-11-10): a slop of about 10 view pixels
pub const SLOP: i32 = 10;

/// The longest a two-finger tap may take, from the first finger down to
/// the last finger up, milliseconds.
// design: touch and touchpad scrolling (lead's request 2026-11-10): a two-finger tap within about 300 ms
pub const TWO_FINGER_TAP_MS: i64 = 300;

/// How long after the last touch event the window still drops mouse
/// events, milliseconds: a system that also sends mouse events made from
/// the touches (Windows does) must not click or move the pointer twice.
// design: touch and touchpad scrolling (lead's request 2026-11-10): emulated mouse events after a touch are dropped
pub const MOUSE_QUIET_MS: i64 = 500;

/// Where a gesture started.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    /// The battlefield view of a running game: a drag pans the view.
    Map,
    /// The game's sidebar: a vertical drag scrolls the icon strip.
    Bar,
    /// Anything else (the menus, the options menu): a drag does nothing,
    /// taps click.
    Other,
}

/// A touch event's phase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// A finger touched down.
    Start,
    /// A finger moved.
    Move,
    /// A finger lifted.
    End,
    /// The system took the touch away (no gesture follows).
    Cancel,
}

/// A host wheel event's delta, positive for content moving right and down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WheelDelta {
    /// A vertical delta in lines (a notched wheel).
    Lines(f64),
    /// A delta in screen pixels (a touchpad, or a smooth wheel).
    Pixels(f64, f64),
}

/// What a touch event asks of the client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gesture {
    /// Move the view centre by this many screen pixels (the content moves
    /// the opposite way, with the finger).
    Pan((i32, i32)),
    /// Scroll the icon strip by this many rows, signed as S-0411 claim
    /// 18's wheel delta: positive moves the strip's content down (toward
    /// its first row).
    Rows(i32),
    /// A press and release of a button at a screen pixel.
    Click(Button, Point),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Contact {
    id: u64,
    start: Point,
    at: Point,
    down: bool,
    down_ms: i64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// No finger down.
    Idle,
    /// One finger: a tap until it passes the slop, then a drag.
    One { region: Region, dragging: bool },
    /// Two fingers: a tap if both lift soon, within the slop.
    Two { since: i64, spoilt: bool },
    /// Nothing more comes of these fingers; wait until all are up.
    Spent,
}

/// The touch-gesture recogniser.
#[derive(Clone, Debug)]
pub struct Touches {
    contacts: Vec<Contact>,
    mode: Mode,
    strip: Strip,
    last_event: Option<i64>,
}

impl Default for Touches {
    fn default() -> Self {
        Self::new()
    }
}

fn past_slop(a: Point, b: Point) -> bool {
    let (dx, dy) = (i64::from(b.x - a.x), i64::from(b.y - a.y));
    let slop = i64::from(SLOP);
    dx * dx + dy * dy > slop * slop
}

impl Touches {
    /// No finger down.
    pub fn new() -> Self {
        Self {
            contacts: Vec::new(),
            mode: Mode::Idle,
            strip: Strip::default(),
            last_event: None,
        }
    }

    /// Whether a finger is down.
    pub fn active(&self) -> bool {
        self.contacts.iter().any(|c| c.down)
    }

    /// Whether mouse events at host time `now_ms` are dropped: a finger is
    /// down, or the last touch event was less than [`MOUSE_QUIET_MS`] ago.
    pub fn mouse_quiet(&self, now_ms: i64) -> bool {
        self.active() || self.last_event.is_some_and(|t| now_ms - t < MOUSE_QUIET_MS)
    }

    /// One touch event: contact `id` at screen pixel `p` at host time
    /// `now_ms`. `region` is where `p` lies; only a first finger's start
    /// reads it. `pitch` is the icon strip's row height in screen pixels.
    pub fn event(
        &mut self,
        phase: Phase,
        id: u64,
        p: Point,
        now_ms: i64,
        region: Region,
        pitch: i32,
    ) -> Option<Gesture> {
        self.last_event = Some(now_ms);
        match phase {
            Phase::Start => {
                self.start(id, p, now_ms, region);
                None
            }
            Phase::Move => self.moved(id, p, pitch),
            Phase::End => self.end(id, p, now_ms),
            Phase::Cancel => {
                self.contacts.clear();
                self.mode = Mode::Idle;
                None
            }
        }
    }

    fn start(&mut self, id: u64, p: Point, now_ms: i64, region: Region) {
        // A stale contact of the same id (a lost end) starts over.
        self.contacts.retain(|c| c.id != id);
        if !self.active() {
            self.contacts.clear();
        }
        self.contacts.push(Contact {
            id,
            start: p,
            at: p,
            down: true,
            down_ms: now_ms,
        });
        let first_ms = self.contacts.first().map_or(now_ms, |c| c.down_ms);
        self.mode = match (self.mode, self.contacts.len()) {
            (_, 1) => {
                self.strip = Strip::default();
                Mode::One {
                    region,
                    dragging: false,
                }
            }
            // A second finger on a first one that is still a tap: a
            // two-finger tap, timed from the first finger down.
            (
                Mode::One {
                    dragging: false, ..
                },
                2,
            ) => Mode::Two {
                since: first_ms,
                spoilt: false,
            },
            // A finger added to a drag is ignored; the drag goes on.
            (m @ Mode::One { dragging: true, .. }, _) => m,
            // A third finger, or one added to spent fingers: nothing.
            _ => Mode::Spent,
        };
    }

    fn moved(&mut self, id: u64, p: Point, pitch: i32) -> Option<Gesture> {
        let first = self.contacts.first().map(|c| c.id);
        let c = self.contacts.iter_mut().find(|c| c.id == id && c.down)?;
        let last = c.at;
        c.at = p;
        let start = c.start;
        match &mut self.mode {
            Mode::One { region, dragging } if first == Some(id) => {
                let d = if *dragging {
                    (p.x - last.x, p.y - last.y)
                } else if past_slop(start, p) {
                    // The whole way from the touch point, so that what was
                    // under the finger stays under it.
                    *dragging = true;
                    (p.x - start.x, p.y - start.y)
                } else {
                    return None;
                };
                match region {
                    // The content follows the finger: the view centre
                    // moves the other way.
                    Region::Map if d != (0, 0) => Some(Gesture::Pan((-d.0, -d.1))),
                    Region::Bar => match self.strip.add(d.1, pitch) {
                        0 => None,
                        n => Some(Gesture::Rows(n)),
                    },
                    _ => None,
                }
            }
            Mode::Two { spoilt, .. } => {
                if past_slop(start, p) {
                    *spoilt = true;
                }
                None
            }
            _ => None,
        }
    }

    fn end(&mut self, id: u64, p: Point, now_ms: i64) -> Option<Gesture> {
        let c = self.contacts.iter_mut().find(|c| c.id == id && c.down)?;
        c.at = p;
        c.down = false;
        let moved = past_slop(c.start, p);
        let start = c.start;
        let all_up = !self.active();
        let out = match self.mode {
            Mode::One {
                dragging: false, ..
            } if all_up && !moved => Some(Gesture::Click(Button::Left, start)),
            Mode::Two { since, spoilt } => {
                let spoilt = spoilt || moved || now_ms - since > TWO_FINGER_TAP_MS;
                self.mode = Mode::Two { since, spoilt };
                match (all_up, spoilt, self.contacts.as_slice()) {
                    (true, false, [a, b]) => Some(Gesture::Click(
                        Button::Right,
                        Point::new(
                            (a.start.x + b.start.x).div_euclid(2),
                            (a.start.y + b.start.y).div_euclid(2),
                        ),
                    )),
                    _ => None,
                }
            }
            _ => None,
        };
        if all_up {
            self.contacts.clear();
            self.mode = Mode::Idle;
        }
        out
    }
}

/// Pixels to icon-strip rows: one row per `pitch` pixels gathered in one
/// direction; turning back starts the count again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Strip {
    rest: i32,
}

impl Strip {
    /// Adds `dy` pixels (positive: the content moves down) and returns the
    /// whole rows they complete, signed as `dy`. `pitch` is the row height
    /// in screen pixels.
    pub fn add(&mut self, dy: i32, pitch: i32) -> i32 {
        let pitch = pitch.max(1);
        if dy.signum() == -self.rest.signum() {
            self.rest = 0;
        }
        self.rest = self.rest.saturating_add(dy);
        let n = self.rest / pitch;
        self.rest -= n * pitch;
        n
    }
}

/// The fraction of pixel deltas carried between events, so a slow swipe of
/// less than a pixel per event still moves.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Carry {
    rest: (f64, f64),
}

impl Carry {
    /// Adds a delta and returns its whole pixels (toward zero), keeping the
    /// rest for the next event.
    pub fn take(&mut self, d: (f64, f64)) -> (i32, i32) {
        let whole = |v: f64| {
            if v.is_finite() {
                v.trunc().clamp(f64::from(i32::MIN), f64::from(i32::MAX))
            } else {
                0.0
            }
        };
        let x = self.rest.0 + d.0;
        let y = self.rest.1 + d.1;
        let (wx, wy) = (whole(x), whole(y));
        self.rest = (
            if x.is_finite() { x - wx } else { 0.0 },
            if y.is_finite() { y - wy } else { 0.0 },
        );
        (wx as i32, wy as i32)
    }
}

#[cfg(test)]
#[path = "touch_tests.rs"]
mod tests;
