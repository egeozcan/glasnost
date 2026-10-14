//! Battlefield cursor images (S-0214 claims 21 and 22, table C): which
//! frames of the imported cursor image show for the hovered action, their
//! hot spot and their animation on the interface's real-time clock.
//! Presentation only: no cursor state reaches the simulation.

use glasnost_sim::targeting::Action;

/// A cursor image: an inclusive range of frames of the imported cursor
/// image (one frame for a static row).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frames {
    /// The first frame.
    pub first: u16,
    /// The last frame, inclusive.
    pub last: u16,
}

impl Frames {
    const fn range(first: u16, last: u16) -> Self {
        Self { first, last }
    }

    const fn one(frame: u16) -> Self {
        Self::range(frame, frame)
    }

    /// The ordinary arrow: static frame 0.
    pub const ARROW: Frames = Frames::one(0);

    /// Whether this is the ordinary arrow, whose hot spot is (0, 0).
    pub fn is_arrow(self) -> bool {
        self == Self::ARROW
    }
}

/// What the pointer offers, for table C: a battlefield action of table A,
/// an attack-move, or a sidebar-mode offer (claim 13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offer {
    /// A battlefield action with its table C context.
    Action {
        /// The action.
        action: Action,
        /// Attack only: the selection is one unit or building whose weapon
        /// is in range of the target.
        in_range: bool,
        /// No-move only: the first selected object's type may move into
        /// shroud.
        shroud_moves: bool,
    },
    /// Attack-move to a place or on a target.
    AttackMove,
    /// Sell mode over a building it can sell.
    Sell,
    /// Sell mode elsewhere.
    NoSell,
    /// Repair mode over an object it can repair: the repair switch.
    RepairSwitch,
    /// Repair mode elsewhere.
    NoRepair,
    /// Power mode, whether or not it offers a switch.
    Power,
}

/// The cursor choice for an offer: an image, or the previous image kept.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Choice {
    /// Show these frames.
    Show(Frames),
    /// Keep the previous cursor image.
    Retain,
}

/// Table C (claim 21): the frames for an offer over a revealed or shrouded
/// target. `retain_damaging_entry` is `quirks.damaging_entry_cursor_retained`.
pub fn table_c(offer: Offer, shrouded: bool, retain_damaging_entry: bool) -> Choice {
    use Action as A;
    let arrow = Frames::ARROW;
    let pick = |revealed: Frames, under: Frames| Choice::Show(if shrouded { under } else { revealed });
    // spec: S-0214 table C (move frames 31–40)
    let mv = Frames::range(31, 40);
    // spec: S-0214 table C (no-move 41)
    let no_move = Frames::one(41);
    match offer {
        Offer::Action {
            action,
            in_range,
            shroud_moves,
        } => match action {
            // spec: S-0214 table C (nothing, idle: arrow)
            A::Nothing | A::Idle => Choice::Show(arrow),
            // spec: S-0214 table C (select, toggle select: 18–30; shrouded: arrow)
            A::Select | A::ToggleSelect => pick(Frames::range(18, 30), arrow),
            A::Move => pick(mv, mv),
            // spec: S-0214 table C (no-move: 41; shrouded: 31–40 if the first selected object's type permits shroud moves, otherwise 41)
            A::NoMove => pick(no_move, if shroud_moves { mv } else { no_move }),
            // spec: S-0214 table C (attack: 53–57 only with one selected unit or building in range, otherwise 58–62; shrouded: 31–40)
            A::Attack => pick(
                if in_range {
                    Frames::range(53, 57)
                } else {
                    Frames::range(58, 62)
                },
                mv,
            ),
            // spec: S-0214 table C (harvest: 58–62; shrouded: arrow)
            A::Harvest => pick(Frames::range(58, 62), arrow),
            // spec: S-0214 table C (enter, capture, recycle, enter tunnel: 89–98; shrouded: arrow)
            A::Enter | A::Capture | A::Recycle | A::EnterTunnel => {
                pick(Frames::range(89, 98), arrow)
            }
            // spec: S-0214 table C (deploy, area attack: 110–118; shrouded: arrow)
            A::Deploy | A::AreaAttack => pick(Frames::range(110, 118), arrow),
            // spec: S-0214 table C (no-deploy: 119 either way)
            A::NoDeploy => Choice::Show(Frames::one(119)),
            // spec: S-0214 table C (no-enter, no-enter tunnel: 99 either way)
            A::NoEnter | A::NoEnterTunnel => Choice::Show(Frames::one(99)),
            // spec: S-0214 table C (repair building, repair vehicle: 150–169; shrouded: arrow)
            A::RepairBuilding | A::RepairVehicle => pick(Frames::range(150, 169), arrow),
            // spec: S-0214 table C (no-repair: 190 either way)
            A::NoRepair => Choice::Show(Frames::one(190)),
            // spec: S-0214 table C (heal, patrol, tote: 384 either way)
            A::Heal | A::Patrol | A::Tote => Choice::Show(Frames::one(384)),
            // spec: S-0214 table C (sabotage, demolish, detonate: 309–318; shrouded: arrow)
            A::Sabotage | A::Demolish | A::Detonate => pick(Frames::range(309, 318), arrow),
            // spec: S-0214 table C (area guard: 68–72 either way)
            A::AreaGuard => Choice::Show(Frames::range(68, 72)),
            // spec: S-0214 table C (bomb: 204 static; shrouded: arrow)
            A::Bomb => pick(Frames::one(204), arrow),
            // spec: S-0214 table C (no-bomb: 41; shrouded: arrow)
            A::NoBomb => pick(no_move, arrow),
            // spec: S-0214 table C (disarm bomb: 369 static either way)
            A::DisarmBomb => Choice::Show(Frames::one(369)),
            // spec: S-0214 table C (airstrike: 480–487; shrouded: arrow)
            A::Airstrike => pick(Frames::range(480, 487), arrow),
            A::DamagingEntry => {
                if shrouded {
                    // spec: S-0214 table C (damaging entry, shrouded: arrow)
                    Choice::Show(arrow)
                } else if retain_damaging_entry {
                    // spec: S-0214 claim 21 and Edge cases (quirks.damaging_entry_cursor_retained: the previous cursor image stays)
                    Choice::Retain
                } else {
                    // spec: S-0214 Edge cases (switch off: the capture cursor of table C)
                    Choice::Show(Frames::range(89, 98))
                }
            }
            // Every action of table A is listed above; the enum is
            // non-exhaustive, and an action added later shows the arrow.
            _ => Choice::Show(arrow),
        },
        // spec: S-0214 table C (attack-move to a place or on a target: 404–412 either way)
        Offer::AttackMove => Choice::Show(Frames::range(404, 412)),
        // spec: S-0214 table C (sell: 129–138; shrouded: 149)
        Offer::Sell => pick(Frames::range(129, 138), Frames::one(149)),
        // spec: S-0214 table C (no-sell: 149 either way)
        Offer::NoSell => Choice::Show(Frames::one(149)),
        // spec: S-0214 table C (repair switch: 170–189; shrouded: 190)
        Offer::RepairSwitch => pick(Frames::range(170, 189), Frames::one(190)),
        // spec: S-0214 table C (no-repair: 190 either way)
        Offer::NoRepair => Choice::Show(Frames::one(190)),
        // spec: S-0214 table C (power switch, no-power: 384 either way)
        Offer::Power => Choice::Show(Frames::one(384)),
    }
}

/// The hot spot of a cursor image in its full canvas of `w` × `h` pixels:
/// the centre (⌊w / 2⌋, ⌊h / 2⌋), or (0, 0) for the ordinary arrow.
pub fn hot_spot(frames: Frames, w: i32, h: i32) -> (i32, i32) {
    if frames.is_arrow() {
        // spec: S-0214 claim 22 (the ordinary arrow's hot spot is (0, 0))
        (0, 0)
    } else {
        // spec: S-0214 claim 22 (the centre hot spot (⌊width/2⌋, ⌊height/2⌋) of the full canvas)
        (w.div_euclid(2), h.div_euclid(2))
    }
}

/// Milliseconds per unit of the presentation clock.
// spec: S-0214 claim 22 (the 16-millisecond presentation clock)
const CLOCK_MS: i64 = 16;
/// Clock units between frame advances.
// spec: S-0214 claim 22 and Constants (an advance once the clock has advanced at least 4 units)
const INTERVAL: i64 = 4;

/// The cursor's animation state (claim 22).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Animator {
    shown: Option<Frames>,
    frame: u16,
    last: i64,
}

impl Animator {
    /// The frame to show at real time `now_ms` for the choice `c`: a
    /// changed image starts at its first frame and restarts the interval;
    /// the same image advances one frame, wrapping, once 4 clock units
    /// have passed since the previous advance, never catching up.
    pub fn frame(&mut self, c: Choice, now_ms: i64) -> (Frames, u16) {
        let clock = now_ms.div_euclid(CLOCK_MS);
        let frames = match (c, self.shown) {
            (Choice::Show(f), _) => f,
            (Choice::Retain, Some(f)) => f,
            (Choice::Retain, None) => Frames::ARROW,
        };
        if self.shown != Some(frames) {
            // spec: S-0214 claim 22 (changing cursor kind starts at its first frame and restarts the interval)
            *self = Self {
                shown: Some(frames),
                frame: frames.first,
                last: clock,
            };
        } else if clock - self.last >= INTERVAL {
            // spec: S-0214 claim 22 (one update advances at most one frame, wrapping to the first; no catch-up)
            self.frame = if self.frame >= frames.last {
                frames.first
            } else {
                self.frame + 1
            };
            self.last = clock;
        }
        (frames, self.frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn act(action: Action) -> Offer {
        Offer::Action {
            action,
            in_range: false,
            shroud_moves: false,
        }
    }

    // spec: S-0214 claim 21 and table C (frame assignments, revealed and shrouded; nothing is the arrow, an illegal move its own refusal image)
    #[test]
    fn claim_21_table_c() {
        let show = |o, s| table_c(o, s, true);
        assert_eq!(show(act(Action::Nothing), false), Choice::Show(Frames::ARROW));
        assert_eq!(show(act(Action::Move), true), Choice::Show(Frames::range(31, 40)));
        assert_eq!(show(act(Action::NoMove), false), Choice::Show(Frames::one(41)));
        assert_eq!(show(act(Action::Select), true), Choice::Show(Frames::ARROW));
        let shroud_mover = Offer::Action {
            action: Action::NoMove,
            in_range: false,
            shroud_moves: true,
        };
        assert_eq!(show(shroud_mover, true), Choice::Show(Frames::range(31, 40)));
        assert_eq!(show(shroud_mover, false), Choice::Show(Frames::one(41)));
        let attack = |in_range| Offer::Action {
            action: Action::Attack,
            in_range,
            shroud_moves: false,
        };
        assert_eq!(show(attack(true), false), Choice::Show(Frames::range(53, 57)));
        assert_eq!(show(attack(false), false), Choice::Show(Frames::range(58, 62)));
        assert_eq!(show(attack(true), true), Choice::Show(Frames::range(31, 40)));
        assert_eq!(show(Offer::Sell, true), Choice::Show(Frames::one(149)));
        assert_eq!(show(Offer::RepairSwitch, false), Choice::Show(Frames::range(170, 189)));
        assert_eq!(show(Offer::AttackMove, true), Choice::Show(Frames::range(404, 412)));
        assert_eq!(show(act(Action::Bomb), false), Choice::Show(Frames::one(204)));
    }

    // spec: S-0214 claim 21 and Edge cases (damaging entry keeps the previous image under quirks.damaging_entry_cursor_retained; off, the capture frames; shrouded, the arrow)
    #[test]
    fn claim_21_damaging_entry_retention() {
        let de = act(Action::DamagingEntry);
        assert_eq!(table_c(de, false, true), Choice::Retain);
        assert_eq!(table_c(de, false, false), Choice::Show(Frames::range(89, 98)));
        assert_eq!(table_c(de, true, true), Choice::Show(Frames::ARROW));
        let mut a = Animator::default();
        let (f, _) = a.frame(table_c(act(Action::Move), false, true), 0);
        assert_eq!(a.frame(table_c(de, false, true), 16), (f, f.first));
    }

    // spec: S-0214 claim 22 (centre hot spot of the full canvas, floor halves; the arrow at (0, 0))
    #[test]
    fn claim_22_hot_spot() {
        assert_eq!(hot_spot(Frames::range(31, 40), 55, 44), (27, 22));
        assert_eq!(hot_spot(Frames::ARROW, 55, 44), (0, 0));
    }

    // spec: S-0214 claim 22 (one frame per 4 units of the 16 ms clock, wrapping, no catch-up, a kind change restarting, the same kind not)
    #[test]
    fn claim_22_animation() {
        let mv = Choice::Show(Frames::range(31, 33));
        let mut a = Animator::default();
        assert_eq!(a.frame(mv, 0).1, 31);
        assert_eq!(a.frame(mv, 63).1, 31, "3 clock units");
        assert_eq!(a.frame(mv, 64).1, 32, "4 clock units");
        // A long delay advances one frame only.
        assert_eq!(a.frame(mv, 10_000).1, 33);
        assert_eq!(a.frame(mv, 10_064).1, 31, "wraps");
        // Another kind starts at its first frame and restarts the interval.
        let sel = Choice::Show(Frames::range(18, 30));
        assert_eq!(a.frame(sel, 10_070).1, 18);
        assert_eq!(a.frame(sel, 10_120).1, 18);
        assert_eq!(a.frame(sel, 10_134).1, 19);
        // A static row stays on its frame.
        let still = Choice::Show(Frames::one(41));
        assert_eq!(a.frame(still, 20_000).1, 41);
        assert_eq!(a.frame(still, 30_000).1, 41);
    }
}
