//! The images a building draws while it unloads a unit (S-0108 claims 22
//! and 23, Formulas F8; depth rows of S-0102 claim 13): which sprite stands
//! in for the body, which under-door sprite is drawn under it, and the
//! door image's frame. Unloading itself, the door and its timing belong to
//! S-0417 and the simulation.

use std::collections::BTreeMap;

use glasnost_num::{F64, Rounding};
use glasnost_sim::behaviour::Door;

/// A building's door as the door image sees it (S-0108 Terms "Door state"
/// and "Door progress", F8): at rest, or moving with `m` the length in
/// ticks of the current movement and `r` the ticks it still has to run.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorView {
    /// Closed, at rest.
    Closed,
    /// Open, at rest.
    Open,
    /// Opening.
    Opening {
        /// Movement length in ticks.
        m: i32,
        /// Ticks still to run.
        r: i32,
    },
    /// Closing.
    Closing {
        /// Movement length in ticks.
        m: i32,
        /// Ticks still to run.
        r: i32,
    },
}

/// The door of the simulation at tick `now` (S-0417 Terms *door*, F2: a
/// movement takes its door time from the tick it began): the ticks still
/// to run are held to 0 … M, so a door whose time has passed but that has
/// not yet reached rest at its timer update shows q = 1.
pub fn door_view(door: Door, now: i32) -> DoorView {
    let left = |since: i32, ticks: i32| {
        let m = ticks.max(0);
        // The same elapsed time as the door's own update (S-0215 claim 35 (d)).
        (m, m.saturating_sub(now.wrapping_sub(since)).clamp(0, m))
    };
    match door {
        Door::Closed => DoorView::Closed,
        Door::Open => DoorView::Open,
        Door::Opening { since, ticks } => {
            let (m, r) = left(since, ticks);
            DoorView::Opening { m, r }
        }
        Door::Closing { since, ticks } => {
            let (m, r) = left(since, ticks);
            DoorView::Closing { m, r }
        }
    }
}

/// A live building that is unloading a unit (S-0417 Terms "Unloading":
/// its current behaviour is *unload*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unloading {
    /// Whether the unloaded unit's type is marked jumpjet or balloon-hover
    /// (S-0108 claim 22, Import mapping `JumpJet`, `BalloonHover`).
    pub roof: bool,
    /// The door.
    pub door: DoorView,
}

/// What stands in for a building's body while it unloads, and the
/// under-door sprite drawn with it (S-0108 claim 22).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnloadSprites<'a> {
    /// The sprite replacing the body sprite with its final depth bias
    /// before −L(z); `None` keeps the body sprite and its normal bias.
    pub body: Option<(&'a [u8], i32)>,
    /// The under-door sprite to draw, if any.
    pub under_door: Option<&'a [u8]>,
}

/// The unloading sprites of a building whose secondary sprites are
/// `sprites` (roles as the importer names them) for an unloaded unit
/// marked jumpjet or balloon-hover (`roof`) or not. `no_fallback` is
/// `quirks.roof_unload_no_fallback`.
pub fn unload_sprites<'a>(
    sprites: &'a BTreeMap<&'static str, Vec<u8>>,
    roof: bool,
    no_fallback: bool,
) -> UnloadSprites<'a> {
    let get = |role: &str| sprites.get(role).map(Vec::as_slice);
    // spec: S-0108 claim 22 (deploying sprite: depth adjustment −20 instead of the normal one);
    // S-0102 claim 13 (final bias −L(z) − 22, replacing the normal depth adjustment)
    let deploying = get("deploying").map(|s| (s, -22));
    // spec: S-0108 claim 22 (roof-deploying sprite: depth adjustment −40);
    // S-0102 claim 13 (final bias −L(z) − 42, replacing the normal depth adjustment)
    let roof_deploying = get("roof_deploying").map(|s| (s, -42));
    if !roof {
        // spec: S-0108 claim 22 (a unit not marked jumpjet or balloon-hover: the deploying sprite
        // replaces the body when the type has one; the under-door sprite)
        return UnloadSprites {
            body: deploying,
            under_door: get("under_door"),
        };
    }
    if no_fallback {
        // spec: S-0108 claim 22, Edge cases (quirks.roof_unload_no_fallback on: the roof-deploying
        // and under-roof-door sprites only; no fallback to the ordinary ones)
        UnloadSprites {
            body: roof_deploying,
            under_door: get("under_roof_door"),
        }
    } else {
        // spec: S-0108 Edge cases (quirks.roof_unload_no_fallback off: fall back to the ordinary sprites)
        UnloadSprites {
            body: roof_deploying.or(deploying),
            under_door: get("under_roof_door").or_else(|| get("under_door")),
        }
    }
}

/// The under-door image frame (S-0108 claim 22).
pub fn under_door_frame(damaged: bool) -> usize {
    // spec: S-0108 claim 22 (image frame 1 when damaged, 0 otherwise)
    usize::from(damaged)
}

/// The door progress q (S-0108 F8) in binary64 with rounding `mode`.
pub fn door_progress(door: DoorView, mode: Rounding) -> F64 {
    let one = F64::from_i64(1, mode);
    match door {
        DoorView::Opening { m, r } | DoorView::Closing { m, r } if m != 0 => {
            // spec: S-0108 F8 (q = (M − r) / M, both integers, divided in binary64 and rounded)
            let num = i64::from(m) - i64::from(r);
            F64::from_i64(num, mode).div(F64::from_i64(i64::from(m), mode), mode)
        }
        // spec: S-0108 F8 (q = 1 when the door is not moving or M = 0)
        _ => one,
    }
}

/// The door frame of S-0108 claim 23 and F8 for door-stage count `d`, a
/// building `damaged` or not, of a damaged-door type or not, with rounding
/// `mode`. A frame below 0 or at or past the door sprite's image-frame
/// count draws nothing (the caller's test).
pub fn door_frame(
    door: DoorView,
    d: i32,
    damaged: bool,
    damaged_door: bool,
    mode: Rounding,
) -> i64 {
    let d64 = i64::from(d);
    // spec: S-0108 F8 (q × D formed in binary64, rounded, then truncated toward zero)
    let stages = || {
        door_progress(door, mode)
            .mul(F64::from_i64(d64, mode), mode)
            .trunc_i128()
            .map_or(0, |t| {
                t.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
            })
    };
    // spec: S-0108 claim 23 (closed 0, opening trunc(q × D), open D − 1, closing D − trunc(q × D))
    let p0 = match door {
        DoorView::Closed => 0,
        DoorView::Opening { .. } => stages(),
        DoorView::Open => d64 - 1,
        DoorView::Closing { .. } => d64 - stages(),
    };
    // spec: S-0108 claim 23, F8 (clamped to 0 … D − 1; 0 when D ≤ 0)
    let p = p0.min(d64 - 1).max(0);
    // spec: S-0108 claim 23, F8 (+ D when damaged and of a damaged-door type)
    p + if damaged && damaged_door { d64 } else { 0 }
}

/// The body frame of a gate whose door is opening or closing (S-0108
/// claim 4): *p* when not damaged and *p* + *G* + 1 when damaged, with *p*
/// the door position clamped to 0 … *G* − 1 (no clamp of claim 3). `None`
/// for a door at rest, which claim 2's stationary rows draw. *p* itself is
/// the Q-0685 interim: claim 23's door-frame rule with *D* = *G*.
pub fn gate_frame(door: DoorView, g: i32, damaged: bool, mode: Rounding) -> Option<i64> {
    if !matches!(door, DoorView::Opening { .. } | DoorView::Closing { .. }) {
        return None;
    }
    // interim: Q-0685 (p from claim 23 and F8 with D = G, clamped to 0 … G − 1)
    let p = door_frame(door, g, false, false, mode);
    // spec: S-0108 claim 4 (p when not damaged, p + G + 1 when damaged)
    Some(p + if damaged { i64::from(g) + 1 } else { 0 })
}

/// The door-stage count *D* and the damaged-door flag of a building
/// (S-0108 claim 24) from the values of `DoorStages` and `DamagedDoor` in
/// the art section named by its image name; neither is inferred from the
/// door sprite.
pub fn door_inputs(stages: Option<&[u8]>, damaged: Option<&[u8]>) -> (i32, bool) {
    // spec: S-0108 claim 24 (a missing or empty key keeps the value in force: initially 0 image frames and false)
    let given = |v: &[u8]| !v.is_empty();
    // spec: S-0108 claim 24 and Import mapping (`DoorStages`: a signed 32-bit integer read by S-0003 claims 3 to 5)
    let d = stages
        .filter(|v| given(v))
        .and_then(glasnost_ini::value::integer)
        .unwrap_or(0);
    // spec: S-0108 claim 24 and Import mapping (`DamagedDoor`: a boolean read by S-0003 claim 2)
    let f = damaged
        .filter(|v| given(v))
        .and_then(glasnost_ini::value::boolean)
        .unwrap_or(false);
    (d, f)
}

/// The door image's frame index into a door sprite of `count` image
/// frames, if one is drawn (S-0108 claim 23: no clamp of claim 3; below 0
/// or at or past the count draws nothing).
pub fn door_image_frame(frame: i64, count: usize) -> Option<usize> {
    // spec: S-0108 claims 3 and 23 (a door frame below 0 or at or past the image-frame count draws nothing)
    usize::try_from(frame).ok().filter(|&f| f < count)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RZ: Rounding = Rounding::TowardZero;

    fn sprites(roles: &[&'static str]) -> BTreeMap<&'static str, Vec<u8>> {
        roles
            .iter()
            .map(|r| (*r, format!("{r}.shp").into_bytes()))
            .collect()
    }

    const ALL: [&str; 4] = [
        "deploying",
        "roof_deploying",
        "under_door",
        "under_roof_door",
    ];

    /// spec: S-0108 claim 22 (an unmarked unit: the deploying sprite with −20, the under-door
    /// sprite); S-0102 claim 13 (final −L(z) − 22)
    #[test]
    fn deploying_sprite_for_an_unmarked_unit() {
        let s = sprites(&ALL);
        let u = unload_sprites(&s, false, true);
        assert_eq!(u.body, Some((&b"deploying.shp"[..], -22)));
        assert_eq!(u.under_door, Some(&b"under_door.shp"[..]));
        // Without a deploying sprite the body sprite stays; without an under-door sprite none.
        let s = sprites(&["roof_deploying", "under_roof_door"]);
        let u = unload_sprites(&s, false, true);
        assert_eq!(u.body, None);
        assert_eq!(u.under_door, None);
    }

    /// spec: S-0108 claim 22 (a jumpjet or balloon-hover unit: the roof-deploying sprite with −40,
    /// the under-roof-door sprite); S-0102 claim 13 (final −L(z) − 42)
    #[test]
    fn roof_deploying_sprite_for_a_marked_unit() {
        let s = sprites(&ALL);
        let u = unload_sprites(&s, true, true);
        assert_eq!(u.body, Some((&b"roof_deploying.shp"[..], -42)));
        assert_eq!(u.under_door, Some(&b"under_roof_door.shp"[..]));
    }

    /// spec: S-0108 claim 22 and Edge cases (quirks.roof_unload_no_fallback: on, no fallback to the
    /// ordinary sprites; off, fall back to them); S-0102 claim 13 (which unloading image is drawn)
    #[test]
    fn roof_unload_has_no_fallback() {
        let s = sprites(&["deploying", "under_door"]);
        let on = unload_sprites(&s, true, true);
        assert_eq!(on.body, None);
        assert_eq!(on.under_door, None);
        let off = unload_sprites(&s, true, false);
        assert_eq!(off.body, Some((&b"deploying.shp"[..], -22)));
        assert_eq!(off.under_door, Some(&b"under_door.shp"[..]));
    }

    /// spec: S-0108 claim 22 (under-door image frame 1 when damaged, 0 otherwise)
    #[test]
    fn under_door_frame_by_damage() {
        assert_eq!(under_door_frame(false), 0);
        assert_eq!(under_door_frame(true), 1);
    }

    /// spec: S-0108 claim 23, F8 (D = 3, M = 3: opening 0, 0, 1, 2; open 2; closing 2, 2, 1, 0
    /// after the clamp; closed 0)
    #[test]
    fn door_frames_through_a_movement() {
        let f = |door| door_frame(door, 3, false, false, RZ);
        let opening: Vec<i64> = (0..=3)
            .rev()
            .map(|r| f(DoorView::Opening { m: 3, r }))
            .collect();
        // q = 0, 1/3, 2/3, 1 (each rounded toward zero): trunc(q × 3) = 0, 0, 1, 3 → clamp 2.
        assert_eq!(opening, [0, 0, 1, 2]);
        assert_eq!(f(DoorView::Open), 2);
        let closing: Vec<i64> = (0..=3)
            .rev()
            .map(|r| f(DoorView::Closing { m: 3, r }))
            .collect();
        // 3 − trunc(q × 3) = 3, 3, 2, 0 → clamp 2, 2, 2, 0.
        assert_eq!(closing, [2, 2, 2, 0]);
        assert_eq!(f(DoorView::Closed), 0);
    }

    /// spec: S-0108 F8 (q and q × D each rounded toward zero in binary64: with M = 3, r = 1,
    /// D = 3, q × D falls just below 2 and truncates to 1; round-to-nearest gives 2), Edge cases
    /// (round-toward-zero mode, S-0205's switch)
    #[test]
    fn door_frame_rounds_toward_zero() {
        let door = DoorView::Opening { m: 3, r: 1 };
        assert_eq!(door_frame(door, 3, false, false, RZ), 1);
        assert_eq!(door_frame(door, 3, false, false, Rounding::Nearest), 2);
    }

    /// spec: S-0108 F8 (q = 1 when M = 0 or the door is not moving)
    #[test]
    fn door_progress_at_rest_and_zero_length() {
        let one = F64::from_i64(1, RZ);
        assert_eq!(door_progress(DoorView::Opening { m: 0, r: 0 }, RZ), one);
        assert_eq!(door_progress(DoorView::Open, RZ), one);
        // Opening with M = 0: trunc(1 × 4) = 4 → clamp 3. Closing: 4 − 4 = 0.
        assert_eq!(
            door_frame(DoorView::Opening { m: 0, r: 0 }, 4, false, false, RZ),
            3
        );
        assert_eq!(
            door_frame(DoorView::Closing { m: 0, r: 0 }, 4, false, false, RZ),
            0
        );
    }

    /// spec: S-0108 claim 23, F8 (+ D when damaged and of a damaged-door type; unchanged otherwise)
    #[test]
    fn damaged_door_adds_the_stage_count() {
        let open = DoorView::Open;
        assert_eq!(door_frame(open, 4, true, true, RZ), 3 + 4);
        assert_eq!(door_frame(open, 4, true, false, RZ), 3);
        assert_eq!(door_frame(open, 4, false, true, RZ), 3);
        assert_eq!(door_frame(DoorView::Closed, 4, true, true, RZ), 4);
    }

    /// spec: S-0108 claim 23 (D = 0: frame 0 in every state, damaged or not; D < 0: 0, but a
    /// damaged damaged-door type requests D and draws nothing; D = 10 with 8 image frames: the open
    /// frame 9 draws nothing), claim 3 (no clamp; below 0 or at or past the count draws nothing)
    #[test]
    fn non_positive_and_high_door_frames() {
        let states = [
            DoorView::Closed,
            DoorView::Opening { m: 4, r: 1 },
            DoorView::Open,
            DoorView::Closing { m: 4, r: 3 },
        ];
        for s in states {
            for dmg in [false, true] {
                assert_eq!(door_frame(s, 0, dmg, true, RZ), 0);
            }
            assert_eq!(door_frame(s, -2, false, true, RZ), 0);
            assert_eq!(door_frame(s, -2, true, true, RZ), -2);
            assert_eq!(door_frame(s, -2, true, false, RZ), 0);
        }
        assert_eq!(door_image_frame(-2, 8), None);
        assert_eq!(door_image_frame(0, 8), Some(0));
        assert_eq!(door_image_frame(7, 8), Some(7));
        let open10 = door_frame(DoorView::Open, 10, false, false, RZ);
        assert_eq!(open10, 9);
        assert_eq!(door_image_frame(open10, 8), None);
        // Damaged damaged-door copy: 10 or more throughout.
        assert!(
            states
                .iter()
                .all(|&s| door_frame(s, 10, true, true, RZ) >= 10)
        );
    }

    /// spec: S-0417 F2 and Terms (door: closed, opening, open, closing; a movement takes the door
    /// time from its first tick), S-0108 F8 (M and r)
    #[test]
    fn door_view_of_the_simulation_door() {
        assert_eq!(door_view(Door::Closed, 9), DoorView::Closed);
        assert_eq!(door_view(Door::Open, 9), DoorView::Open);
        assert_eq!(
            door_view(
                Door::Opening {
                    since: 10,
                    ticks: 5
                },
                12
            ),
            DoorView::Opening { m: 5, r: 3 }
        );
        assert_eq!(
            door_view(
                Door::Closing {
                    since: 10,
                    ticks: 5
                },
                10
            ),
            DoorView::Closing { m: 5, r: 5 }
        );
        // Time passed before the door's update brings it to rest: r = 0.
        assert_eq!(
            door_view(
                Door::Opening {
                    since: 10,
                    ticks: 5
                },
                20
            ),
            DoorView::Opening { m: 5, r: 0 }
        );
    }

    // spec: S-0108 claim 24 and Import mapping (absent or empty keys: 0 and false; S-0003's partial forms: `4abc` is 4, `Yes please` true; a signed count)
    #[test]
    fn door_inputs_from_their_keys() {
        assert_eq!(door_inputs(None, None), (0, false));
        assert_eq!(door_inputs(Some(b""), Some(b"")), (0, false));
        assert_eq!(door_inputs(Some(b"3"), Some(b"no")), (3, false));
        assert_eq!(door_inputs(Some(b"4abc"), Some(b"Yes please")), (4, true));
        assert_eq!(door_inputs(Some(b"-2"), Some(b"1")), (-2, true));
    }

    // spec: S-0108 claim 4 (a moving gate: p undamaged, p + G + 1 damaged, p within 0 … G − 1; at rest: the stationary rows); interim: Q-0685 (p by claim 23's rule with D = G)
    #[test]
    fn moving_gate_frames() {
        let g = 9;
        assert_eq!(gate_frame(DoorView::Closed, g, false, RZ), None);
        assert_eq!(gate_frame(DoorView::Open, g, true, RZ), None);
        // Opening, 90 ticks: q = 0 gives 0, q = 1/2 gives trunc(4.5) = 4, q = 1 holds G − 1.
        let opening = |r| DoorView::Opening { m: 90, r };
        assert_eq!(gate_frame(opening(90), g, false, RZ), Some(0));
        assert_eq!(gate_frame(opening(45), g, false, RZ), Some(4));
        assert_eq!(gate_frame(opening(0), g, false, RZ), Some(8));
        // Closing runs backwards: q = 0 gives G, clamped to G − 1; q = 1 gives 0.
        let closing = |r| DoorView::Closing { m: 90, r };
        assert_eq!(gate_frame(closing(90), g, false, RZ), Some(8));
        assert_eq!(gate_frame(closing(0), g, false, RZ), Some(0));
        // Damaged: + G + 1, so G + 1 … 2G.
        assert_eq!(gate_frame(opening(90), g, true, RZ), Some(10));
        assert_eq!(gate_frame(opening(0), g, true, RZ), Some(18));
    }
}
