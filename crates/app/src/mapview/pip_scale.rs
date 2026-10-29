//! The pip scale's rows and self-heal marker (S-0111 claims 10 to 16):
//! which image frame each pip of a row shows, and where each pip is drawn
//! from the pip-scale origin. Pure functions of the object's state; the
//! live view gathers that state from the simulation ([`super::live`]).

use glasnost_ini::value;
use glasnost_num::{F64, Rounding};

use super::combat::{Pip, Sprite};

/// A type's pip-scale kind (S-0111 Import mapping, `PipScale`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipKind {
    /// No row (`none`).
    None,
    /// Ammunition (`ammo`, claim 15).
    Ammo,
    /// Resource (`tiberium`, claim 14).
    Resource,
    /// Passengers (`passengers`, claim 12).
    Passengers,
    /// Power (`power`): no row of its own (claim 11).
    Power,
    /// Mind control (`mindcontrol`, claim 13).
    MindControl,
}

/// The vocabulary of `PipScale`, in [`PipKind`] order (S-0111 Import
/// mapping).
const PIP_SCALE_WORDS: [&str; 6] = [
    "none",
    "ammo",
    "tiberium",
    "passengers",
    "power",
    "mindcontrol",
]; // spec: S-0111 Import mapping (PipScale: none, ammo, tiberium, passengers, power, mindcontrol)

/// The text limit of the pip fields' named values. S-0003 claim 16 gives
/// 31 or 127 bytes per field without naming these fields; every word of
/// their vocabularies is shorter than 31 bytes, so either limit gives the
/// same match.
const NAMED_LIMIT: usize = 31; // spec: S-0003 claim 16 (named values read as text cut to the field's limit, 31 or 127 bytes in the cases checked)

/// The pip-scale kind a stored `PipScale` value gives.
pub fn pip_kind(stored: Option<&[u8]>) -> PipKind {
    // spec: S-0111 Import mapping (PipScale, absent none; compared ignoring case, S-0003 claim 16)
    let Some(v) = stored else {
        return PipKind::None;
    };
    // design: a word that matches none of PipScale's vocabulary gives none; S-0111 states no fallback for this field (Q-2071)
    match value::named(v, NAMED_LIMIT, &PIP_SCALE_WORDS) {
        Some(1) => PipKind::Ammo,
        Some(2) => PipKind::Resource,
        Some(3) => PipKind::Passengers,
        Some(4) => PipKind::Power,
        Some(5) => PipKind::MindControl,
        _ => PipKind::None,
    }
}

/// The vocabulary of `Pip` and `OccupyPip` with the image frame each word
/// selects (S-0111 Import mapping).
const PIP_WORDS: [(&str, usize); 11] = [
    ("green", 1),
    ("yellow", 2),
    ("white", 3),
    ("red", 4),
    ("blue", 5),
    ("persongreen", 7),
    ("personyellow", 8),
    ("personwhite", 9),
    ("personred", 10),
    ("personblue", 11),
    ("personpurple", 12),
]; // spec: S-0111 Import mapping (vocabulary of Pip and OccupyPip with the image frame each word selects)

/// The passenger pip colour before any reading (S-0111 claim 12).
pub const PIP_INITIAL: usize = 1; // spec: S-0111 claim 12 and Import mapping (Pip: an absent read keeps the held value, initially image frame 1)
/// The occupant pip image frame before any reading (S-0111 claim 12).
pub const OCCUPY_PIP_INITIAL: usize = 7; // spec: S-0111 claim 12 and Import mapping (OccupyPip: an absent read keeps the held value, initially image frame 7)
/// The image frame of a word that matches none (S-0111 Import mapping).
const PIP_UNKNOWN: usize = 1; // spec: S-0111 Import mapping (a word that matches none gives 1, with an importer warning)

/// The image frame a layered `Pip` or `OccupyPip` value gives. The layered
/// value is the last one a layer gave, so a layer that omits the key keeps
/// the held value; absent from every layer it is `initial` (S-0111 claim
/// 12, S-0003 claim 16, S-0006 claim 5).
pub fn pip_frame(stored: Option<&[u8]>, initial: usize) -> usize {
    let Some(v) = stored else {
        return initial;
    };
    let words: Vec<&str> = PIP_WORDS.iter().map(|(w, _)| *w).collect();
    value::named(v, NAMED_LIMIT, &words).map_or(PIP_UNKNOWN, |i| PIP_WORDS[i].1)
}

/// Which pip row an object's kind draws (S-0111 claim 10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    /// A vehicle or an aircraft.
    Vehicle,
    /// An infantry unit.
    Infantry,
    /// A building.
    Building,
}

/// Pip `j` of a row, image frame `frame`, from the pip-scale origin `o`
/// (S-0111 claim 10); `ammo` for the ammunition row.
pub fn place(body: Body, o: (i32, i32), j: i32, frame: usize, ammo: bool) -> Pip {
    // spec: S-0111 claim 10 (table: vehicle or aircraft o + (−5 + 4j, 0) second pip sprite; infantry o + (6 + 4j, 0) second pip sprite; building o + (6 + 4j, −1 + 2j) pip sprite)
    let (at, sprite) = match body {
        Body::Vehicle => ((o.0 - 5 + 4 * j, o.1), Sprite::Pips2),
        Body::Infantry => ((o.0 + 6 + 4 * j, o.1), Sprite::Pips2),
        Body::Building => ((o.0 + 6 + 4 * j, o.1 - 1 + 2 * j), Sprite::Pips),
    };
    if ammo {
        // spec: S-0111 claim 10 (ammunition pips 3 pixels higher than the row, always from the second pip sprite)
        Pip::image(Sprite::Pips2, frame, (at.0, at.1 - 3))
    } else {
        Pip::image(sprite, frame, at)
    }
}

/// A row's pips from pip 0, drawn at the places of claim 10.
pub fn places(body: Body, o: (i32, i32), frames: &[usize]) -> Vec<Pip> {
    frames
        .iter()
        .zip(0..)
        .map(|(&f, j)| place(body, o, j, f, false))
        .collect()
}

/// The power kind's row length (S-0111 Constants).
const POWER_ROW: i32 = 10; // spec: S-0111 Constants (power pip-scale length 10 pips) and T2
/// The resource row length of an infantry type (S-0111 claim 14).
const RESOURCE_ROW_INFANTRY: i32 = 3; // spec: S-0111 Constants (resource pips 3 for infantry)
/// The resource row length of any other type (S-0111 claim 14).
const RESOURCE_ROW: i32 = 5; // spec: S-0111 Constants (resource pips 5 for others)

/// The row length *R* a pip-scale kind gives (S-0111 Constants T2):
/// `ammo` the ammunition capacity, `capacity` the passenger capacity,
/// `primary` the primary weapon's damage if the type has a primary weapon.
pub fn row_length(
    kind: PipKind,
    infantry: bool,
    ammo: i32,
    capacity: i32,
    primary: Option<i32>,
) -> i32 {
    // spec: S-0111 T2 (row length by pip-scale kind)
    match kind {
        PipKind::None => 0,
        PipKind::Ammo => ammo,
        PipKind::Resource if infantry => RESOURCE_ROW_INFANTRY,
        PipKind::Resource => RESOURCE_ROW,
        PipKind::Passengers => capacity,
        PipKind::Power => POWER_ROW,
        // spec: S-0111 claim 13 and T2 (the primary weapon's damage; 0 when the type has none, a departure)
        PipKind::MindControl => primary.unwrap_or(0),
    }
}

/// A passenger as its pip sees it (S-0111 claim 12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passenger {
    /// An infantry passenger, with its type's passenger pip colour.
    Infantry(usize),
    /// A vehicle passenger.
    Vehicle,
    /// Any other passenger.
    Other,
}

/// The empty pip (S-0111 T1).
const EMPTY: usize = 0; // spec: S-0111 T1 (image frame 0: empty pip)
/// A passenger's extra pip (S-0111 claim 12).
const EXTRA_SPACE: usize = 3; // spec: S-0111 claim 12 (a block's other pips show image frame 3)
/// A vehicle passenger's colour pip (S-0111 claim 12).
const VEHICLE_PASSENGER: usize = 5; // spec: S-0111 claim 12 (image frame 5 for a vehicle)
/// Any other passenger's colour pip (S-0111 claim 12).
const OTHER_PASSENGER: usize = 1; // spec: S-0111 claim 12 (image frame 1 for any other passenger)
/// A vehicle's pip in an absorbing building (S-0111 claim 12).
const ABSORBED_VEHICLE: usize = 4; // spec: S-0111 claim 12 (absorbing buildings: a vehicle image frame 4)

fn trunc(v: F64) -> i64 {
    // design: a value beyond the 64-bit range (not reachable from imported sizes) truncates to 0
    v.trunc_i128()
        .and_then(|t| i64::try_from(t).ok())
        .unwrap_or(0)
}

/// The image frames of a transport's passenger row of length `r` (S-0111
/// claim 12), pip 0 first. `entered` holds the passengers in the order
/// they entered, each with its size *z* (binary64); `mode` is the rounding
/// mode of S-0205.
pub fn passenger_row(r: i32, entered: &[(Passenger, F64)], mode: Rounding) -> Vec<usize> {
    let r = r.max(0) as usize;
    // spec: S-0111 claim 12 (total size: Z_{m+1} = 0, Z_k = trunc(Z_{k+1} + z_k) for k = m … 1, each sum in binary64 under the rounding mode of S-0205)
    let z = entered.iter().rev().fold(0i64, |acc, &(_, size)| {
        trunc(F64::from_i64(acc, mode).add(size, mode))
    });
    let mut frames = vec![EMPTY; r];
    let one = F64::from_i64(1, mode);
    // spec: S-0111 claim 12 (blocks stacked downwards from pip Z − 1 in order of entry, the most recent passenger's block highest)
    let mut top = z;
    for &(p, size) in entered.iter().rev() {
        // spec: S-0111 claim 12 (b = 1 + max(0, trunc(z − 1)) for infantry or vehicle, z − 1 in binary64 with the S-0205 mode; b = 1 for any other)
        let b = match p {
            Passenger::Infantry(_) | Passenger::Vehicle => 1 + trunc(size.sub(one, mode)).max(0),
            Passenger::Other => 1,
        };
        let s = top - b;
        // spec: S-0111 claim 12 (the lowest pip shows its colour pip: the infantry type's passenger pip colour, 5 for a vehicle, 1 for any other; its other pips image frame 3)
        let colour = match p {
            Passenger::Infantry(c) => c,
            Passenger::Vehicle => VEHICLE_PASSENGER,
            Passenger::Other => OTHER_PASSENGER,
        };
        for j in s..top {
            // spec: S-0111 claim 12 (only pips 0 ≤ j < R are drawn: a block reaching below pip 0 shows nothing there, and Z > R clips; a departure)
            if let Ok(i) = usize::try_from(j)
                && i < r
            {
                frames[i] = if j == s { colour } else { EXTRA_SPACE };
            }
        }
        top = s;
    }
    frames
}

/// The image frames of an absorbing building's passenger row of length
/// `r` (S-0111 claim 12), pip 0 first; `recent_first` holds the passengers,
/// the most recent first.
pub fn absorbed_row(r: i32, recent_first: &[Passenger]) -> Vec<usize> {
    let r = r.max(0) as usize;
    // spec: S-0111 claim 12 (absorbing buildings: one pip per passenger upwards from pip 0, the most recent at pip 0; infantry its pip colour, a vehicle 4, anything else 1; the rest 0; more than R passengers: only the R most recent, our engine)
    (0..r)
        .map(|j| match recent_first.get(j) {
            Some(Passenger::Infantry(c)) => *c,
            Some(Passenger::Vehicle) => ABSORBED_VEHICLE,
            Some(Passenger::Other) => OTHER_PASSENGER,
            None => EMPTY,
        })
        .collect()
}

/// The place index of passenger pip `j` (S-0111 claim 12): on a gunner
/// type each pip from 1 on is drawn at the place of pip `j` + 2.
pub fn passenger_place(gunner: bool, j: i32) -> i32 {
    // spec: S-0111 claim 12 (gunner gap: pip 0 at its place, pip j ≥ 1 at the place of pip j + 2)
    if gunner && j >= 1 { j + 2 } else { j }
}

/// A full spawn slot's pip (S-0111 claim 11).
const SPAWN_FULL: usize = 1; // spec: S-0111 claim 11 (spawned aircraft: image frame 1 for each aircraft docked or ready, 0 for the rest)

/// The image frames of the spawned-aircraft row (S-0111 claim 11): one per
/// spawn slot, `full` when its aircraft is docked or ready.
pub fn spawn_row(full: &[bool]) -> Vec<usize> {
    full.iter()
        .map(|&f| if f { SPAWN_FULL } else { EMPTY })
        .collect()
}

/// An empty occupant slot's pip (S-0111 claim 11).
const OCCUPANT_EMPTY: usize = 6; // spec: S-0111 claim 11 and T1 (image frame 6 for an empty occupant slot)

/// The image frames of the occupant row of `slots` slots (S-0111 claim
/// 11): each occupant's occupant pip image frame, then empty slots.
pub fn occupant_row(slots: i32, occupants: &[usize]) -> Vec<usize> {
    // design: occupant k (the first the one inside longest, S-0508 Terms) fills slot k; S-0111 gives no slot order (Q-2072)
    (0..slots.max(0) as usize)
        .map(|k| occupants.get(k).copied().unwrap_or(OCCUPANT_EMPTY))
        .collect()
}

/// A controlled pip (S-0111 claim 13).
const CONTROLLED: usize = 1; // spec: S-0111 claim 13 (as many pips as the unit controls show image frame 1)
/// The overload pip after a pulse (S-0111 claim 13).
const OVERLOAD_HIGHLIGHT: usize = 3; // spec: S-0111 claim 13 (the overload pip shows image frame 3 for 10 ticks from each damaging pulse)
/// The overload pip otherwise (S-0111 claim 13).
const OVERLOAD: usize = 4; // spec: S-0111 claim 13 (otherwise image frame 4)

/// The mind-control row (S-0111 claim 13) as (pip index, image frame):
/// `m` pips, `count` controlled, and with `overload` (its highlight on or
/// off) one more pip at index `m`.
pub fn mind_row(m: i32, count: usize, overload: Option<bool>) -> Vec<(i32, usize)> {
    let mut out: Vec<(i32, usize)> = (0..m.max(0))
        .map(|j| {
            let controlled = usize::try_from(j).is_ok_and(|j| j < count);
            (j, if controlled { CONTROLLED } else { EMPTY })
        })
        .collect();
    if let Some(lit) = overload {
        // spec: S-0111 claim 13 (one more pip at index M, image frame 3 while highlighted, else 4)
        out.push((m, if lit { OVERLOAD_HIGHLIGHT } else { OVERLOAD }));
    }
    out
}

/// A resource pip at registration index 1 (S-0111 claim 14).
const RESOURCE_G: usize = 5; // spec: S-0111 claim 14 (the first n_g pips image frame 5)
/// A resource pip of the other resources (S-0111 claim 14).
const RESOURCE_Q: usize = 2; // spec: S-0111 claim 14 (the next n_q pips image frame 2)

/// The image frames of a resource row of length `r` (S-0111 claim 14,
/// F2) for the load `load` (one amount per registration index) and
/// storage capacity `c`; `mode` is the rounding mode of S-0205. No pips
/// for `c` = 0.
pub fn resource_row(r: i32, load: &[i32], c: i32, mode: Rounding) -> Vec<usize> {
    // spec: S-0111 F2 (C = 0: our engine draws no resource pips; the importer warns)
    if c == 0 {
        return Vec::new();
    }
    // spec: S-0111 claim 14 (each amount a single-precision number, widened exactly to binary64)
    let amount = |i: usize| {
        let a = load.get(i).copied().unwrap_or(0);
        F64::from_i64(i64::from(a), mode).to_f32(mode).to_f64()
    };
    // spec: S-0111 claim 14 (g: the amount at registration index 1; q = (a3 + a2) + a0 in binary64 under the S-0205 mode)
    let g = amount(1);
    let q = amount(3).add(amount(2), mode).add(amount(0), mode);
    let (cf, rf) = (
        F64::from_i64(i64::from(c), mode),
        F64::from_i64(i64::from(r), mode),
    );
    let half = F64::from_parts(false, 1, -1, mode);
    // spec: S-0111 F2 (n = trunc((ℓ / C) × R + 0.5), each operation rounded by the S-0205 mode)
    let n = |l: F64| trunc(l.div(cf, mode).mul(rf, mode).add(half, mode));
    let (ng, nq) = (n(g), n(q));
    // spec: S-0111 claim 14 (the first n_g pips image frame 5, the next n_q image frame 2, the rest 0; pips beyond R are not drawn)
    (0..i64::from(r.max(0)))
        .map(|j| {
            if j < ng {
                RESOURCE_G
            } else if j < ng.saturating_add(nq.max(0)) {
                RESOURCE_Q
            } else {
                EMPTY
            }
        })
        .collect()
}

/// An unwrapped ammunition pip (S-0111 claim 15).
const AMMO: usize = 13; // spec: S-0111 claim 15 (no wrap: image frame 13 while j < a)
/// An empty wrapped ammunition pip (S-0111 F3).
const AMMO_WRAP_EMPTY: usize = 14; // spec: S-0111 F3 (14 when no λ qualifies)
/// The first wrapped ammunition layer's pip (S-0111 F3).
const AMMO_WRAP_FIRST: i32 = 15; // spec: S-0111 F3 (image frame 15 + λ)

/// The ammunition row (S-0111 claim 15, F3) as (pip index, image frame)
/// for capacity `cap`, current ammunition `a` and pip wrap `w`.
pub fn ammo_row(cap: i32, a: i32, w: i32) -> Vec<(i32, usize)> {
    if w <= 0 {
        // spec: S-0111 claim 15 (w = 0: pip j < A drawn in image frame 13 while j < a, not drawn otherwise); design: a negative wrap, which S-0111 does not cover, reads as no wrap (Q-2071)
        return (0..cap.min(a)).map(|j| (j, AMMO)).collect();
    }
    let layers = cap / w;
    (0..w)
        .map(|j| {
            // spec: S-0111 F3 (the largest λ in 0 … ⌊A / w⌋ − 1 with λ·w + j < a gives 15 + λ; none gives 14)
            let below = a - 1 - j;
            let lambda = if below < 0 {
                -1
            } else {
                (below / w).min(layers - 1)
            };
            let frame = if lambda >= 0 {
                (AMMO_WRAP_FIRST + lambda) as usize
            } else {
                AMMO_WRAP_EMPTY
            };
            (j, frame)
        })
        .collect()
}

/// The self-heal ability that applies to a unit (S-0111 claim 16).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heal {
    /// Infantry healing (infantry and organic vehicles).
    Infantry,
    /// Unit repair (other vehicles).
    Unit,
}

/// The infantry-healing marker's image frame (S-0111 claim 16).
const HEAL_INFANTRY: usize = 13; // spec: S-0111 claim 16 (image frame 13 for infantry healing)
/// The unit-repair marker's image frame (S-0111 claim 16).
const HEAL_UNIT: usize = 20; // spec: S-0111 claim 16 (image frame 20 for unit repair)
/// The blink's ticks per period (S-0111 Constants).
const BLINK_TICKS: i64 = 6; // spec: S-0111 Constants (self-heal blink 6 ticks per period: t mod T ≤ 5)

/// The self-heal marker of a vehicle (`infantry` false) or infantry unit
/// with the ability `heal`, from the pip-scale origin `o` (S-0111 claim
/// 16), at tick `t` with the ability's interval `period`; `damaged` while
/// its health is below its full strength.
pub fn self_heal(
    infantry: bool,
    heal: Heal,
    o: (i32, i32),
    damaged: bool,
    t: i32,
    period: i32,
) -> Pip {
    let frame = match heal {
        Heal::Infantry => HEAL_INFANTRY,
        Heal::Unit => HEAL_UNIT,
    };
    // spec: S-0111 claim 16 ((33, −32) from o for a vehicle, (25, −35) for infantry)
    let at = if infantry {
        (o.0 + 25, o.1 - 35)
    } else {
        (o.0 + 33, o.1 - 32)
    };
    // spec: S-0111 claim 16 (while damaged and t mod |T| ≤ 5 a shadow; a negative T blinks with |T|; T = 0 never blinks, a departure)
    let p = i64::from(period).abs();
    let shadow = damaged && p != 0 && i64::from(t).rem_euclid(p) < BLINK_TICKS;
    Pip {
        // spec: S-0111 claim 10 (the self-heal marker always comes from the pip sprite)
        sprite: Sprite::Pips,
        frame,
        at,
        shadow,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RZ: Rounding = Rounding::TowardZero;

    fn size(n: i64, d: i64) -> F64 {
        F64::from_i64(n, RZ).div(F64::from_i64(d, RZ), RZ)
    }

    /// spec: S-0111 Import mapping (PipScale vocabulary ignoring case, absent none; Pip and OccupyPip vocabulary, unknown 1, absent the held value)
    #[test]
    fn named_values() {
        assert_eq!(pip_kind(None), PipKind::None);
        assert_eq!(pip_kind(Some(b"Passengers")), PipKind::Passengers);
        assert_eq!(pip_kind(Some(b"MINDCONTROL")), PipKind::MindControl);
        assert_eq!(pip_kind(Some(b"tiberium")), PipKind::Resource);
        assert_eq!(pip_kind(Some(b"Ammo")), PipKind::Ammo);
        assert_eq!(pip_kind(Some(b"power")), PipKind::Power);
        assert_eq!(pip_kind(Some(b"sideways")), PipKind::None);
        assert_eq!(pip_frame(None, PIP_INITIAL), 1);
        assert_eq!(pip_frame(None, OCCUPY_PIP_INITIAL), 7);
        assert_eq!(pip_frame(Some(b"Yellow"), PIP_INITIAL), 2);
        assert_eq!(pip_frame(Some(b"personPurple"), OCCUPY_PIP_INITIAL), 12);
        assert_eq!(pip_frame(Some(b"plaid"), OCCUPY_PIP_INITIAL), 1);
    }

    /// spec: S-0111 claim 10 (pip places and sprites by kind; ammunition 3 pixels higher from the second pip sprite)
    #[test]
    fn claim_10_places() {
        let o = (-10, 10);
        assert_eq!(
            place(Body::Vehicle, o, 2, 1, false),
            Pip::image(Sprite::Pips2, 1, (-7, 10))
        );
        assert_eq!(
            place(Body::Infantry, o, 0, 1, false),
            Pip::image(Sprite::Pips2, 1, (-4, 10))
        );
        assert_eq!(
            place(Body::Building, (-60, 0), 3, 6, false),
            Pip::image(Sprite::Pips, 6, (-42, 5))
        );
        assert_eq!(
            place(Body::Building, (-60, 0), 1, 13, true),
            Pip::image(Sprite::Pips2, 13, (-50, -2))
        );
    }

    /// spec: S-0111 T2 (row length by kind; mind control without a primary 0)
    #[test]
    fn t2_row_lengths() {
        assert_eq!(row_length(PipKind::None, false, 5, 5, Some(3)), 0);
        assert_eq!(row_length(PipKind::Ammo, false, 4, 5, None), 4);
        assert_eq!(row_length(PipKind::Resource, true, 0, 0, None), 3);
        assert_eq!(row_length(PipKind::Resource, false, 0, 0, None), 5);
        assert_eq!(row_length(PipKind::Passengers, false, 0, 6, None), 6);
        assert_eq!(row_length(PipKind::Power, false, 0, 6, None), 10);
        assert_eq!(row_length(PipKind::MindControl, false, 0, 6, Some(3)), 3);
        assert_eq!(row_length(PipKind::MindControl, false, 0, 6, None), 0);
    }

    /// spec: S-0111 claim 12 (blocks in order of entry from pip 0, colour pip then extra pips, then empty pips)
    #[test]
    fn claim_12_blocks_read_in_order_of_entry() {
        let one = size(1, 1);
        let row = passenger_row(
            6,
            &[
                (Passenger::Infantry(7), one),
                (Passenger::Vehicle, size(5, 2)),
                (Passenger::Infantry(9), one),
            ],
            RZ,
        );
        // Z = 1 + 2 + 1 = 4: infantry 7 at 0, vehicle (b = 2) at 1 and 2, infantry 9 at 3.
        assert_eq!(row, [7, 5, 3, 9, 0, 0]);
    }

    /// spec: S-0111 claim 12 (a negative size: Z by the stated recursion; the example −0.5 then 2.5 gives Z = 1)
    #[test]
    fn claim_12_total_size_with_a_negative_size() {
        let row = passenger_row(
            4,
            &[
                (Passenger::Infantry(7), size(-1, 2)),
                (Passenger::Infantry(8), size(5, 2)),
            ],
            RZ,
        );
        // Z = trunc(trunc(0 + 2.5) − 0.5) = 1. The second's block (b = 2) is
        // pips −1 and 0: its colour pip falls below pip 0, pip 0 is extra
        // space; the first's block (b = 1) is pip −2, not drawn.
        assert_eq!(row, [3, 0, 0, 0]);
    }

    /// spec: S-0111 claim 12 (Z > R clips to the row; R = 0 no pips; other passengers b = 1 and the lowest pips stay empty)
    #[test]
    fn claim_12_clipping_and_other_passengers() {
        let two = size(2, 1);
        let full = passenger_row(
            3,
            &[(Passenger::Vehicle, two), (Passenger::Vehicle, two)],
            RZ,
        );
        // Z = 4: first at 0–1, second at 2–3; pip 3 is clipped.
        assert_eq!(full, [5, 3, 5]);
        assert!(passenger_row(0, &[(Passenger::Vehicle, two)], RZ).is_empty());
        let other = passenger_row(4, &[(Passenger::Other, size(3, 1))], RZ);
        assert_eq!(other, [0, 0, 1, 0]);
    }

    /// spec: S-0111 claim 12 (gunner gap; absorbing buildings fill upwards from the most recent, only the R most recent)
    #[test]
    fn claim_12_gunner_gap_and_absorbing_buildings() {
        assert_eq!(passenger_place(true, 0), 0);
        assert_eq!(passenger_place(true, 1), 3);
        assert_eq!(passenger_place(false, 1), 1);
        let row = absorbed_row(
            3,
            &[
                Passenger::Infantry(8),
                Passenger::Vehicle,
                Passenger::Other,
                Passenger::Infantry(9),
            ],
        );
        assert_eq!(row, [8, 4, 1]);
        assert_eq!(absorbed_row(3, &[Passenger::Infantry(8)]), [8, 0, 0]);
    }

    /// spec: S-0111 claim 11 (spawn row: 1 per docked or ready aircraft, 0 for the rest; occupant row: the occupant pip image frame or 6)
    #[test]
    fn claim_11_spawn_and_occupant_rows() {
        assert_eq!(spawn_row(&[true, false, true]), [1, 0, 1]);
        assert_eq!(occupant_row(4, &[7, 12]), [7, 12, 6, 6]);
        assert!(occupant_row(0, &[7]).is_empty());
    }

    /// spec: S-0111 claim 13 (M pips, the controlled ones 1; the overload pip at index M, 3 highlighted, else 4; M = 0 puts it at 0)
    #[test]
    fn claim_13_mind_control_row() {
        assert_eq!(mind_row(3, 2, None), [(0, 1), (1, 1), (2, 0)]);
        assert_eq!(mind_row(2, 3, Some(true)), [(0, 1), (1, 1), (2, 3)]);
        assert_eq!(mind_row(2, 3, Some(false))[2], (2, 4));
        assert_eq!(mind_row(0, 1, Some(false)), [(0, 4)]);
    }

    /// spec: S-0111 claim 14 and F2 (each part rounds half up separately; infantry 3 pips; C = 0 no pips)
    #[test]
    fn claim_14_resource_row() {
        // C = 16, R = 5: g = 8 → 2.5 + 0.5 → 3; q = (2 + 1) + 1 = 4 → 1.25 + 0.5 → 1.
        assert_eq!(resource_row(5, &[1, 8, 1, 2], 16, RZ), [5, 5, 5, 2, 0]);
        // C = 32: g = 4 → 0.625 + 0.5 → 1 and q = 4 → 1, two pips, where
        // the whole load 8 → 1.25 + 0.5 would give one.
        assert_eq!(resource_row(5, &[4, 4, 0, 0], 32, RZ), [5, 2, 0, 0, 0]);
        // Rounding toward zero: 1 / 10 × 5 + 0.5 stays below 1, no pip.
        assert_eq!(resource_row(5, &[0, 1, 0, 0], 10, RZ), [0; 5]);
        // Infantry's 3 pips; past the row nothing is drawn.
        assert_eq!(resource_row(3, &[10, 10, 0, 0], 10, RZ), [5, 5, 5]);
        assert!(resource_row(5, &[1, 1, 0, 0], 0, RZ).is_empty());
    }

    /// spec: S-0111 claim 15 and F3 (no wrap: 13 while j < a; wrap w: 15 + λ or 14)
    #[test]
    fn claim_15_ammo_row() {
        assert_eq!(ammo_row(5, 3, 0), [(0, 13), (1, 13), (2, 13)]);
        assert!(ammo_row(5, 0, 0).is_empty());
        // A = 6, w = 2: λ up to 2. a = 5: pip 0 → λ = 2 (4 < 5), pip 1 → λ = 1 (3 < 5; 5 < 5 fails).
        assert_eq!(ammo_row(6, 5, 2), [(0, 17), (1, 16)]);
        assert_eq!(ammo_row(6, 0, 2), [(0, 14), (1, 14)]);
        // ⌊A / w⌋ = 0: no λ qualifies.
        assert_eq!(ammo_row(1, 1, 2), [(0, 14), (1, 14)]);
    }

    /// spec: S-0111 claim 16 (image frame by ability, place by kind, the shadow blink while damaged and t mod |T| ≤ 5; T = 0 never blinks)
    #[test]
    fn claim_16_self_heal_marker() {
        let o = (-10, 10);
        let v = self_heal(false, Heal::Unit, o, false, 0, 100);
        assert_eq!(v, Pip::image(Sprite::Pips, 20, (23, -22)));
        let i = self_heal(true, Heal::Infantry, o, true, 105, 100);
        assert_eq!((i.frame, i.at, i.shadow), (13, (15, -25), true));
        assert!(!self_heal(true, Heal::Infantry, o, true, 106, 100).shadow);
        assert!(self_heal(true, Heal::Infantry, o, true, 205, -100).shadow);
        assert!(!self_heal(true, Heal::Infantry, o, true, 0, 0).shadow);
    }
}
