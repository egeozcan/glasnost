//! The tracks of the drive locomotor (S-0207 tables T1 and T2).

/// One track point: displacement from the track target in leptons and a
/// facing in 1/256 of a turn.
pub type Point = (i32, i32, u8);

// spec: S-0207 table T1 (curve A, straight towards map north; 23 points)
const A: [Point; 23] = [
    (0, 245, 0), (0, 234, 0), (0, 223, 0), (0, 212, 0), (0, 201, 0), (0, 190, 0), (0, 179, 0),
    (0, 168, 0), (0, 157, 0), (0, 146, 0), (0, 135, 0), (0, 124, 0), (0, 113, 0), (0, 102, 0),
    (0, 91, 0), (0, 80, 0), (0, 69, 0), (0, 58, 0), (0, 47, 0), (0, 36, 0), (0, 25, 0), (0, 14, 0),
    (0, 3, 0),
];

// spec: S-0207 table T1 (curve B, diagonal towards map north-east; 31 points)
const B: [Point; 31] = [
    (-248, 248, 32), (-240, 240, 32), (-232, 232, 32), (-224, 224, 32), (-216, 216, 32),
    (-208, 208, 32), (-200, 200, 32), (-192, 192, 32), (-184, 184, 32), (-176, 176, 32),
    (-168, 168, 32), (-160, 160, 32), (-152, 152, 32), (-144, 144, 32), (-136, 136, 32),
    (-129, 129, 32), (-120, 120, 32), (-112, 112, 32), (-104, 104, 32), (-96, 96, 32),
    (-88, 88, 32), (-80, 80, 32), (-72, 72, 32), (-64, 64, 32), (-56, 56, 32), (-48, 48, 32),
    (-40, 40, 32), (-32, 32, 32), (-24, 24, 32), (-16, 16, 32), (-8, 8, 32),
];

// spec: S-0207 table T1 (curve C, N then NE; 54 points)
const C: [Point; 54] = [
    (-256, 501, 0), (-256, 490, 0), (-256, 479, 0), (-256, 468, 0), (-256, 457, 0), (-256, 446, 0),
    (-256, 435, 0), (-256, 424, 0), (-256, 413, 0), (-256, 402, 0), (-256, 391, 0), (-256, 383, 0),
    (-256, 373, 0), (-256, 363, 0), (-254, 352, 1), (-252, 341, 3), (-250, 332, 4), (-248, 321, 5),
    (-245, 311, 7), (-241, 302, 8), (-237, 292, 9), (-233, 282, 11), (-229, 272, 12),
    (-225, 263, 13), (-220, 252, 15), (-216, 243, 16), (-212, 236, 17), (-206, 224, 19),
    (-202, 215, 20), (-195, 207, 21), (-190, 198, 23), (-183, 186, 24), (-179, 176, 25),
    (-168, 168, 27), (-160, 160, 28), (-152, 152, 29), (-144, 144, 31), (-136, 136, 32),
    (-129, 129, 32), (-120, 120, 32), (-112, 112, 32), (-104, 104, 32), (-96, 96, 32), (-88, 88, 32),
    (-80, 80, 32), (-72, 72, 32), (-64, 64, 32), (-56, 56, 32), (-48, 48, 32), (-40, 40, 32),
    (-32, 32, 32), (-24, 24, 32), (-16, 16, 32), (-8, 8, 32),
];

// spec: S-0207 table T1 (curve D, N then E; 38 points)
const D: [Point; 38] = [
    (-256, 245, 0), (-256, 235, 0), (-256, 224, 0), (-256, 213, 0), (-255, 203, 0), (-253, 192, 0),
    (-251, 181, 1), (-249, 171, 1), (-246, 160, 2), (-243, 149, 3), (-240, 139, 4), (-236, 127, 5),
    (-232, 117, 8), (-228, 109, 12), (-222, 99, 16), (-219, 90, 20), (-213, 82, 23), (-206, 72, 27),
    (-201, 64, 32), (-195, 56, 36), (-186, 48, 39), (-177, 43, 43), (-168, 36, 47), (-160, 32, 51),
    (-147, 27, 54), (-135, 23, 57), (-126, 20, 60), (-113, 17, 62), (-104, 13, 63), (-94, 9, 64),
    (-84, 6, 64), (-75, 4, 66), (-64, 3, 64), (-53, 2, 64), (-43, 1, 64), (-32, 0, 64),
    (-21, 0, 64), (-11, 0, 64),
];

// spec: S-0207 table T1 (curve E, NE then SE; 61 points)
const E: [Point; 61] = [
    (-504, -8, 32), (-496, -16, 32), (-488, -24, 32), (-480, -32, 32), (-472, -40, 32),
    (-464, -48, 32), (-456, -56, 32), (-448, -64, 32), (-440, -72, 32), (-432, -80, 32),
    (-424, -88, 32), (-416, -96, 32), (-408, -104, 32), (-400, -112, 32), (-392, -120, 32),
    (-385, -127, 32), (-376, -136, 32), (-368, -143, 32), (-361, -150, 32), (-353, -158, 32),
    (-344, -166, 32), (-336, -173, 35), (-329, -181, 38), (-322, -188, 41), (-316, -194, 44),
    (-306, -199, 47), (-296, -204, 50), (-288, -208, 53), (-277, -211, 56), (-267, -212, 59),
    (-256, -213, 62), (-245, -212, 66), (-235, -211, 69), (-225, -208, 72), (-216, -204, 75),
    (-208, -199, 78), (-198, -194, 81), (-188, -188, 84), (-181, -181, 87), (-176, -173, 90),
    (-168, -166, 93), (-160, -158, 96), (-152, -150, 96), (-144, -143, 96), (-136, -136, 96),
    (-129, -129, 96), (-120, -120, 96), (-112, -112, 96), (-104, -104, 96), (-96, -96, 96),
    (-88, -88, 96), (-80, -80, 96), (-72, -72, 96), (-64, -64, 96), (-56, -56, 96), (-48, -48, 96),
    (-40, -40, 96), (-32, -32, 96), (-24, -24, 96), (-16, -16, 96), (-8, -8, 96),
];

// spec: S-0207 table T1 (curve F, NE then E; 56 points)
const F: [Point; 56] = [
    (-512, 256, 32), (-504, 248, 32), (-496, 240, 32), (-488, 232, 32), (-480, 224, 32),
    (-472, 216, 32), (-464, 208, 32), (-456, 200, 32), (-448, 192, 32), (-440, 184, 32),
    (-432, 176, 32), (-424, 168, 32), (-416, 160, 32), (-408, 152, 32), (-400, 144, 32),
    (-392, 136, 32), (-385, 129, 32), (-376, 120, 32), (-368, 112, 32), (-360, 104, 32),
    (-352, 96, 32), (-344, 88, 32), (-338, 85, 32), (-328, 78, 35), (-320, 72, 37), (-311, 66, 40),
    (-302, 59, 43), (-294, 55, 45), (-285, 50, 48), (-277, 43, 51), (-267, 38, 53), (-258, 34, 56),
    (-248, 28, 59), (-238, 25, 61), (-229, 21, 64), (-218, 17, 64), (-208, 14, 64), (-199, 11, 64),
    (-189, 9, 64), (-178, 7, 64), (-169, 5, 64), (-158, 3, 64), (-147, 1, 64), (-137, 0, 64),
    (-129, 0, 64), (-117, 0, 64), (-107, 0, 64), (-96, 0, 64), (-85, 0, 64), (-75, 0, 64),
    (-64, 0, 64), (-53, 0, 64), (-43, 0, 64), (-32, 0, 64), (-21, 0, 64), (-11, 0, 64),
];

/// A transform of table T2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tf {
    /// (x, y, f) → (y, x, (192 − f) mod 256).
    Mirror,
    /// (x, y, f) → (−x, y, (256 − f) mod 256).
    FlipX,
    /// (x, y, f) → (x, −y, (128 − f) mod 256).
    FlipY,
}

/// The curves of table T1 by letter index 0 … 5 (A … F).
fn curve(i: u8) -> &'static [Point] {
    match i {
        0 => &A,
        1 => &B,
        2 => &C, // spec: S-0207 table T1 (curve C)
        3 => &D, // spec: S-0207 table T1 (curve D)
        4 => &E, // spec: S-0207 table T1 (curve E)
        _ => &F,
    }
}

/// A table T2 entry: curve letter index, transforms, two-cell.
type Entry = Option<(u8, &'static [Tf], bool)>;

/// Table T2 by (current step d, next step e).
fn t2(d: u8, e: u8) -> Entry {
    use Tf::{FlipX as X, FlipY as Y, Mirror as M};
    // spec: S-0207 table T2 (track for each pair of steps)
    const ROWS: [[Entry; 8]; 8] = [
        // N
        [Some((0, &[], false)), Some((2, &[], true)), Some((3, &[], true)), None, None, None, Some((3, &[X], true)), Some((2, &[X], true))],
        // NE
        [Some((5, &[M, X, Y], true)), Some((1, &[], false)), Some((5, &[], true)), Some((4, &[], true)), None, None, None, Some((4, &[M, X, Y], true))],
        // E
        [Some((3, &[M, X, Y], true)), Some((2, &[M, X, Y], true)), Some((0, &[M, X], false)), Some((2, &[M, X], true)), Some((3, &[M, X], true)), None, None, None],
        // SE
        [None, Some((4, &[Y], true)), Some((5, &[Y], true)), Some((1, &[Y], false)), Some((5, &[M, X], true)), Some((4, &[M, X], true)), None, None],
        // S
        [None, None, Some((3, &[Y], true)), Some((2, &[Y], true)), Some((0, &[Y], false)), Some((2, &[X, Y], true)), Some((3, &[X, Y], true)), None],
        // SW
        [None, None, None, Some((4, &[M], true)), Some((5, &[M], true)), Some((1, &[M], false)), Some((5, &[X, Y], true)), Some((4, &[X, Y], true))],
        // W
        [Some((3, &[M, Y], true)), None, None, None, Some((3, &[M], true)), Some((2, &[M], true)), Some((0, &[M], false)), Some((2, &[M, Y], true))],
        // NW
        [Some((5, &[M, Y], true)), Some((4, &[M, Y], true)), None, None, None, Some((4, &[X], true)), Some((5, &[X], true)), Some((1, &[X], false))],
    ];
    ROWS[usize::from(d % 8)][usize::from(e % 8)] // spec: S-0207 table T2 (rows and columns by step direction)
}

fn apply(p: Point, t: Tf) -> Point {
    let (x, y, f) = p;
    let f = i32::from(f);
    // spec: S-0207 table T2 (transforms: mirror, flip x, flip y)
    match t {
        Tf::Mirror => (y, x, (192 - f).rem_euclid(256) as u8),
        Tf::FlipX => (-x, y, (256 - f).rem_euclid(256) as u8),
        Tf::FlipY => (x, -y, (128 - f).rem_euclid(256) as u8),
    }
}

/// The track for current step `d` and next step `e` (S-0207 claim 7): its
/// points and whether it is a two-cell curve. `e = None` (no next step, or
/// crush mode) gives the straight or diagonal track for `d`.
pub fn track(d: u8, e: Option<u8>) -> (Vec<Point>, bool) {
    let entry = match e {
        Some(e) => t2(d, e).or_else(|| t2(d, d)),
        None => t2(d, d),
    };
    let (c, tfs, two) = entry.unwrap_or((0, &[], false));
    let pts = curve(c)
        .iter()
        .map(|&p| tfs.iter().fold(p, |acc, &t| apply(acc, t)))
        .collect();
    (pts, two)
}

/// The chain point index of a two-cell curve (S-0207 table T1), if any.
pub fn chain_point(d: u8, e: u8) -> Option<usize> {
    let (c, _, two) = t2(d, e)?;
    if !two {
        return None;
    }
    // spec: S-0207 table T1 (chain points: C 37, D 26, E 45, F 44)
    match c {
        2 => Some(37),
        3 => Some(26),
        4 => Some(45),
        5 => Some(44),
        _ => None,
    }
}

/// The end correction trunc(7 × (1 − m/11)) of S-0207 claim 3 for a track.
pub fn end_correction(points: &[Point]) -> i32 {
    let (x, y, _) = points.last().copied().unwrap_or((0, 0, 0));
    let m = x.abs() + y.abs();
    // spec: S-0207 F3 (trunc(7 × (1 − m/11)), exact: 5, −3 or 0 for the end distances of T1)
    7 * (11 - m) / 11
}

#[cfg(test)]
mod tests {
    use super::*;

    // spec: S-0207 F3 (track costs: A 163, B 227, C 388, D 273, E 437, F 399)
    #[test]
    fn costs() {
        for (c, want) in [(0u8, 163), (1, 227), (2, 388), (3, 273), (4, 437), (5, 399)] {
            let pts = curve(c);
            let cost = 7 * (pts.len() as i32 + 1) - end_correction(pts);
            assert_eq!(cost, want, "curve {c}");
        }
    }

    // spec: S-0207 table T2 (every transformed track ends with the facing of its last step, behind its target)
    #[test]
    fn transformed_tracks_end_facing_the_last_step() {
        for d in 0..8u8 {
            for e in 0..8u8 {
                let Some((_, _, two)) = t2(d, e) else { continue };
                let (pts, two2) = track(d, Some(e));
                assert_eq!(two, two2);
                let last = pts.last().copied().unwrap();
                let want = if two { e } else { d };
                assert_eq!(u32::from(last.2), u32::from(want) * 32, "pair ({d}, {e})");
            }
        }
        // Straight north starts south of its target and faces north.
        let (pts, _) = track(0, None);
        assert_eq!(pts[0], (0, 245, 0));
        // Straight south (A flip y) faces south.
        let (pts, _) = track(4, None);
        assert_eq!(pts[0], (0, -245, 128));
    }
}
