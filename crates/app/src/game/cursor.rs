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
    /// The ordinary arrow in place of the representative's action image:
    /// a berserk infantry or vehicle representative with
    /// `quirks.berserk_action_cursor` off (S-0214 Edge cases).
    BerserkArrow,
    /// A superweapon aim action of table CS (claim 31).
    Aim(Aim),
}

/// The superweapon aim actions with a battlefield image in table CS
/// (S-0214 claim 31).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Aim {
    /// Nuclear strike.
    Nuke,
    /// Lightning storm.
    LightningStorm,
    /// Iron curtain (invulnerability).
    IronCurtain,
    /// Chronosphere source pick or destination: one image for both.
    Chronosphere,
    /// Force shield.
    ForceShield,
    /// No-force-shield: refused force-shield aiming (S-0513 claim 1).
    NoForceShield,
    /// Paradrop, either kind.
    Paradrop,
    /// Spy plane.
    SpyPlane,
    /// Psychic domination.
    PsychicDominator,
    /// Genetic mutation.
    GeneticConverter,
    /// Psychic reveal.
    PsychicReveal,
}

impl Aim {
    /// The aim action of a superweapon type's action word (S-0512 claim 23
    /// and Import mapping, lower case), when table CS has an image for it;
    /// `None` for no action and for a word outside table CS.
    pub fn of_word(word: &str) -> Option<Self> {
        // spec: S-0214 table CS (the aim action rows) and S-0512 Import mapping (the action words)
        Some(match word {
            "nuke" => Self::Nuke,
            "lightningstorm" => Self::LightningStorm,
            "ironcurtain" => Self::IronCurtain,
            "chronosphere" | "chronowarp" => Self::Chronosphere,
            "forceshield" => Self::ForceShield,
            "noforceshield" => Self::NoForceShield,
            "paradrop" | "amerparadrop" => Self::Paradrop,
            "spyplane" => Self::SpyPlane,
            "psychicdominator" => Self::PsychicDominator,
            "geneticconverter" => Self::GeneticConverter,
            "psychicreveal" => Self::PsychicReveal,
            _ => return None,
        })
    }

    /// Its battlefield image, revealed or shrouded alike (table CS).
    pub fn frames(self) -> Frames {
        match self {
            // spec: S-0214 table CS (nuclear strike 319–328)
            Self::Nuke => Frames::range(319, 328),
            // spec: S-0214 table CS (lightning storm 279–298)
            Self::LightningStorm => Frames::range(279, 298),
            // spec: S-0214 table CS (iron curtain 346)
            Self::IronCurtain => Frames::one(346),
            // spec: S-0214 table CS and claim 31 (chronosphere source pick and destination: the same static 357)
            Self::Chronosphere => Frames::one(357),
            // spec: S-0214 table CS (force shield 450–459)
            Self::ForceShield => Frames::range(450, 459),
            // spec: S-0214 table CS (no-force-shield 460–469)
            Self::NoForceShield => Frames::range(460, 469),
            // spec: S-0214 table CS (paradrop, either kind, 259)
            Self::Paradrop => Frames::one(259),
            // spec: S-0214 table CS (spy plane 504–511)
            Self::SpyPlane => Frames::range(504, 511),
            // spec: S-0214 table CS (psychic domination 488–495)
            Self::PsychicDominator => Frames::range(488, 495),
            // spec: S-0214 table CS (genetic mutation 470–479)
            Self::GeneticConverter => Frames::range(470, 479),
            // spec: S-0214 table CS (psychic reveal 496–503)
            Self::PsychicReveal => Frames::range(496, 503),
        }
    }
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
    let pick =
        |revealed: Frames, under: Frames| Choice::Show(if shrouded { under } else { revealed });
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
        // spec: S-0214 claim 21 and Edge cases (with quirks.berserk_action_cursor off the arrow replacement takes precedence over table C, revealed or shrouded)
        Offer::BerserkArrow => Choice::Show(arrow),
        // spec: S-0214 claim 31 (shrouded and revealed targets use the same superweapon image)
        Offer::Aim(a) => Choice::Show(a.frames()),
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

/// One row of table E: direction, allowed frame, blocked frame, hot-spot
/// places.
type EdgeRow = ((i32, i32), u16, u16, (u8, u8));

/// The edge-scroll cursor's screen directions in table E's order (screen x
/// right, y down) with the allowed frame, the blocked frame and the hot
/// spot's horizontal and vertical place: 0 left or top, 1 centre, 2 right
/// or bottom.
// spec: S-0213 table E (Up 2/10 centre-top, UR 3/11, R 4/12, LR 5/13, Down 6/14, LL 7/15, L 8/16, UL 9/17)
const EDGE: [EdgeRow; 8] = [
    ((0, -1), 2, 10, (1, 0)),
    ((1, -1), 3, 11, (2, 0)),
    ((1, 0), 4, 12, (2, 1)),
    ((1, 1), 5, 13, (2, 2)),
    ((0, 1), 6, 14, (1, 2)),
    ((-1, 1), 7, 15, (0, 2)),
    ((-1, 0), 8, 16, (0, 1)),
    ((-1, -1), 9, 17, (0, 0)),
];

/// The edge-scroll cursor (S-0213 claim 24, table E) for a requested
/// screen direction: its static frame, the allowed or the blocked variant,
/// and its hot spot in a full canvas of `w` × `h` pixels. `None` for a
/// direction that is not one of the eight.
pub fn edge(dir: (i32, i32), blocked: bool, w: i32, h: i32) -> Option<(Frames, (i32, i32))> {
    let &(_, allowed, refused, (hx, hy)) = EDGE.iter().find(|e| e.0 == dir)?;
    // spec: S-0213 claim 24 (the blocked variant when a one-pixel trial scroll cannot move the view)
    let f = if blocked { refused } else { allowed };
    // spec: S-0213 table E (centre ⌊w/2⌋ and ⌊h/2⌋; right and bottom use w and h, not the last pixel)
    let place = |k: u8, size: i32| match k {
        0 => 0,
        1 => size.div_euclid(2),
        _ => size,
    };
    Some((Frames::one(f), (place(hx, w), place(hy, h))))
}

/// The right-drag cursor (S-0101 claim 17): its static full-size frame for
/// the refused one-pixel screen directions `[up, right, down, left]`, and
/// its hot spot, the centre of a full canvas of `w` × `h` pixels.
pub fn drag(refused: [bool; 4], w: i32, h: i32) -> (Frames, (i32, i32)) {
    // spec: S-0101 claim 17 (the frame by the refused screen directions, zero-based: none 385, Up 386, Up and Right 387, Right 388, Right and Down 389, Down 390, Down and Left 391, Left 392, Left and Up 393, any other combination 385)
    let frame = match refused {
        [true, false, false, false] => 386,
        [true, true, false, false] => 387,
        [false, true, false, false] => 388,
        [false, true, true, false] => 389,
        [false, false, true, false] => 390,
        [false, false, true, true] => 391,
        [false, false, false, true] => 392,
        [true, false, false, true] => 393,
        _ => 385,
    };
    // spec: S-0101 claim 17 (the hot spot is the full canvas centre (⌊w/2⌋, ⌊h/2⌋))
    (Frames::one(frame), (w.div_euclid(2), h.div_euclid(2)))
}

/// The one-pixel screen directions S-0101 claim 17 tries: up, right, down,
/// left.
pub const DRAG_TRIALS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)]; // spec: S-0101 claim 17 (a one-pixel view-centre request in each of the four screen directions)

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
    /// The hot spot of an image shown outside the animation (the edge-scroll
    /// and drag cursors, which have their own hot spots), kept with it.
    hot: Option<(i32, i32)>,
}

impl Animator {
    /// Records an image shown instead of an animated one: the edge-scroll
    /// or drag cursor with its hot spot, or `None` when no battlefield
    /// cursor is drawn (the system pointer shows). A later image is then a
    /// change of kind, and a kept image (`Choice::Retain`) is this one.
    pub fn other(&mut self, shown: Option<(Frames, (i32, i32))>) {
        match shown {
            // spec: S-0214 claim 22 (changing cursor kind starts at its first frame; the edge-scroll and drag images are static); #85
            Some((frames, hot)) => {
                if self.shown != Some(frames) {
                    self.shown = Some(frames);
                    self.frame = frames.first;
                }
                self.hot = Some(hot);
            }
            // design: render (#85) with no battlefield cursor drawn nothing is kept: the next image starts at its first frame, and a kept image is the ordinary arrow
            None => *self = Self::default(),
        }
    }

    /// The hot spot of the image the last [`Animator::frame`] gave, in a
    /// full canvas of `w` × `h` pixels: that of a kept edge-scroll or drag
    /// image, otherwise [`hot_spot`].
    pub fn hot(&self, frames: Frames, w: i32, h: i32) -> (i32, i32) {
        self.hot.unwrap_or_else(|| hot_spot(frames, w, h))
    }

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
                hot: None,
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

    // spec: S-0214 claim 31 and table CS (the battlefield aim images by aim action word, revealed or shrouded alike; both teleport stages share 357; the centre hot spot of claim 22)
    #[test]
    fn claim_31_table_cs() {
        let rows: [(&str, Frames); 13] = [
            ("nuke", Frames::range(319, 328)),
            ("lightningstorm", Frames::range(279, 298)),
            ("ironcurtain", Frames::one(346)),
            ("chronosphere", Frames::one(357)),
            ("chronowarp", Frames::one(357)),
            ("forceshield", Frames::range(450, 459)),
            ("noforceshield", Frames::range(460, 469)),
            ("paradrop", Frames::one(259)),
            ("amerparadrop", Frames::one(259)),
            ("spyplane", Frames::range(504, 511)),
            ("psychicdominator", Frames::range(488, 495)),
            ("geneticconverter", Frames::range(470, 479)),
            ("psychicreveal", Frames::range(496, 503)),
        ];
        for (word, frames) in rows {
            let aim = Aim::of_word(word).expect(word);
            for shrouded in [false, true] {
                assert_eq!(
                    table_c(Offer::Aim(aim), shrouded, true),
                    Choice::Show(frames),
                    "{word}"
                );
            }
            assert_eq!(hot_spot(frames, 55, 43), (27, 21), "{word}");
        }
        for word in ["none", "attack", "airstrike", ""] {
            assert_eq!(Aim::of_word(word), None, "{word}");
        }
    }

    // spec: S-0214 claim 21 and Edge cases (quirks.berserk_action_cursor off: the ordinary arrow, static frame 0 with hot spot (0, 0), whatever the target)
    #[test]
    fn berserk_arrow_replaces_table_c() {
        for shrouded in [false, true] {
            assert_eq!(
                table_c(Offer::BerserkArrow, shrouded, true),
                Choice::Show(Frames::ARROW)
            );
        }
        assert_eq!(hot_spot(Frames::ARROW, 55, 43), (0, 0));
    }

    // spec: S-0214 claim 21 and table C (frame assignments, revealed and shrouded; nothing is the arrow, an illegal move its own refusal image)
    #[test]
    fn claim_21_table_c() {
        let show = |o, s| table_c(o, s, true);
        assert_eq!(
            show(act(Action::Nothing), false),
            Choice::Show(Frames::ARROW)
        );
        assert_eq!(
            show(act(Action::Move), true),
            Choice::Show(Frames::range(31, 40))
        );
        assert_eq!(
            show(act(Action::NoMove), false),
            Choice::Show(Frames::one(41))
        );
        assert_eq!(show(act(Action::Select), true), Choice::Show(Frames::ARROW));
        let shroud_mover = Offer::Action {
            action: Action::NoMove,
            in_range: false,
            shroud_moves: true,
        };
        assert_eq!(
            show(shroud_mover, true),
            Choice::Show(Frames::range(31, 40))
        );
        assert_eq!(show(shroud_mover, false), Choice::Show(Frames::one(41)));
        let attack = |in_range| Offer::Action {
            action: Action::Attack,
            in_range,
            shroud_moves: false,
        };
        assert_eq!(
            show(attack(true), false),
            Choice::Show(Frames::range(53, 57))
        );
        assert_eq!(
            show(attack(false), false),
            Choice::Show(Frames::range(58, 62))
        );
        assert_eq!(
            show(attack(true), true),
            Choice::Show(Frames::range(31, 40))
        );
        assert_eq!(show(Offer::Sell, true), Choice::Show(Frames::one(149)));
        assert_eq!(
            show(Offer::RepairSwitch, false),
            Choice::Show(Frames::range(170, 189))
        );
        assert_eq!(
            show(Offer::AttackMove, true),
            Choice::Show(Frames::range(404, 412))
        );
        assert_eq!(
            show(act(Action::Bomb), false),
            Choice::Show(Frames::one(204))
        );
    }

    // spec: S-0214 claim 21 and Edge cases (damaging entry keeps the previous image under quirks.damaging_entry_cursor_retained; off, the capture frames; shrouded, the arrow)
    #[test]
    fn claim_21_damaging_entry_retention() {
        let de = act(Action::DamagingEntry);
        assert_eq!(table_c(de, false, true), Choice::Retain);
        assert_eq!(
            table_c(de, false, false),
            Choice::Show(Frames::range(89, 98))
        );
        assert_eq!(table_c(de, true, true), Choice::Show(Frames::ARROW));
        let mut a = Animator::default();
        let (f, _) = a.frame(table_c(act(Action::Move), false, true), 0);
        assert_eq!(a.frame(table_c(de, false, true), 16), (f, f.first));
    }

    // spec: S-0213 claim 24 and table E (allowed and blocked frames per screen direction; hot spots at 0, the floor half or the full canvas size)
    #[test]
    fn s0213_claim_24_edge_cursor() {
        let (w, h) = (55, 44);
        let rows = [
            ((0, -1), 2, 10, (27, 0)),
            ((1, -1), 3, 11, (55, 0)),
            ((1, 0), 4, 12, (55, 22)),
            ((1, 1), 5, 13, (55, 44)),
            ((0, 1), 6, 14, (27, 44)),
            ((-1, 1), 7, 15, (0, 44)),
            ((-1, 0), 8, 16, (0, 22)),
            ((-1, -1), 9, 17, (0, 0)),
        ];
        for (d, a, b, hot) in rows {
            assert_eq!(edge(d, false, w, h), Some((Frames::one(a), hot)), "{d:?}");
            assert_eq!(edge(d, true, w, h), Some((Frames::one(b), hot)), "{d:?}");
        }
        assert_eq!(edge((0, 0), false, w, h), None);
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

    /// #85. spec: S-0214 claim 22 (changing cursor kind starts at its first frame and restarts the
    /// interval) and claim 21 (damaging entry keeps the previous cursor image)
    #[test]
    fn claim_22_an_image_shown_outside_the_animation() {
        let mv = Choice::Show(Frames::range(31, 40));
        let up = edge((0, -1), false, 55, 43).unwrap();
        let mut a = Animator::default();
        assert_eq!(a.frame(mv, 0).1, 31);
        assert_eq!(a.frame(mv, 64).1, 32);
        // The edge-scroll cursor, then the same action: its first frame.
        a.other(Some(up));
        assert_eq!(a.frame(mv, 80).1, 31);
        assert_eq!(a.hot(Frames::range(31, 40), 55, 43), (27, 21));
        // A kept image after the edge-scroll cursor is that image, with
        // its own hot spot.
        a.other(Some(up));
        let (frames, frame) = a.frame(Choice::Retain, 96);
        assert_eq!((frames, frame), (up.0, up.0.first));
        assert_eq!(a.hot(frames, 55, 43), up.1);
        // The system pointer: the action starts again, and a kept image is
        // the ordinary arrow.
        assert_eq!(a.frame(mv, 112).1, 31);
        assert_eq!(a.frame(mv, 176).1, 32);
        a.other(None);
        assert_eq!(a.frame(mv, 192).1, 31);
        a.other(None);
        let (frames, _) = a.frame(Choice::Retain, 208);
        assert_eq!(frames, Frames::ARROW);
        assert_eq!(a.hot(frames, 55, 43), (0, 0));
    }

    /// spec: S-0101 claim 17 (the drag-scroll frame by refused directions, 385 to 393, any other combination 385; the hot spot the canvas centre)
    #[test]
    fn s0101_claim_17_drag_cursor() {
        let f = |r: [bool; 4]| drag(r, 55, 43).0.first;
        assert_eq!(f([false; 4]), 385);
        assert_eq!(f([true, false, false, false]), 386);
        assert_eq!(f([true, true, false, false]), 387);
        assert_eq!(f([false, true, false, false]), 388);
        assert_eq!(f([false, true, true, false]), 389);
        assert_eq!(f([false, false, true, false]), 390);
        assert_eq!(f([false, false, true, true]), 391);
        assert_eq!(f([false, false, false, true]), 392);
        assert_eq!(f([true, false, false, true]), 393);
        assert_eq!(f([true, false, true, false]), 385);
        assert_eq!(f([true; 4]), 385);
        assert_eq!(drag([false; 4], 55, 43).1, (27, 21));
        assert_eq!(DRAG_TRIALS, [(0, -1), (1, 0), (0, 1), (-1, 0)]);
    }
}
