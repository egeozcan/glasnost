//! The own AI's spies disguise before they infiltrate (design: own AI,
//! #454; PLAN §4.7).
//!
//! A spy whose type has a weapon with the disguise-flag warhead (S-0516
//! claim 31: an infantry firer copies the infantry it hits, that unit's
//! disguise or else its type and owner) is a *disguiser*. A person sends
//! one in disguised: an enemy's units and defences then take it for one of
//! their own (claim 35: it appears disguised to house H when its disguise
//! house is H, none, or a house H allies, and no detector coverage of H
//! lies on its centre cell). The brain does the same with each disguiser
//! of the house, once [`Special::spies`] has chosen the building it goes
//! for, as before:
//! - **Not fooling the building's owner** (its disguise does not pass the
//!   house condition of claim 35 for the owner: none, or its own house's,
//!   as a permanent disguiser's side default is, claim 32): it is sent to
//!   attack the nearest seen infantry unit of the owner, or of a house the
//!   owner allies, within [`COPY_REACH`] cells. It keeps that attack for up
//!   to [`COPY_TIME`] ticks; a unit whose hit gave no disguise that fools
//!   the owner (it may have been disguised itself) is not tried again until
//!   the spy fools the owner.
//! - **No such infantry in sight:** it goes in undisguised, as before (a
//!   person does not keep a spy waiting at the enemy base for infantry that
//!   may never come). A spy on its way in without a disguise that fools the
//!   owner is turned to disguise as soon as such a unit comes into sight,
//!   which is also how a spy whose disguise a hit cleared (claim 34 (a))
//!   disguises again.
//! - **Fooling the owner:** it goes in. When the straight way from the spy
//!   to the building's centre crosses the sensor disc of a seen disguise
//!   detector building (claim 40: the disc of its detection range, around
//!   its centre cell) of the owner or a house it allies, it first walks to
//!   a cell [`MARGIN`] cells beyond such a disc, the one from which the two
//!   straight legs cross the fewest covered cells, when that is fewer than
//!   the straight way crosses; once per disguise. Claim 40 has a detector
//!   that was built, deployed or placed add no coverage (claim 23's
//!   activation rules); the house cannot tell which seen detectors cover,
//!   so it avoids them all. A detector unit gives no coverage (claim 40)
//!   and is not avoided.
//!
//! A spy with no disguise weapon goes in at once, as before. The module
//! reads only the house's own units (its spies' disguises), the enemy
//! objects it sees ([`crate::in_view`]) and the rules a person may read
//! (which types detect disguises and how far); never another house's
//! disguises or detector coverage. It draws no random value. The counters
//! [`crate::special::Uses::disguises`] and
//! [`crate::special::Uses::spy_detours`] count its orders.

use std::collections::BTreeMap;

use glasnost_map::CellXY;
use glasnost_sim::cloak;
use glasnost_sim::combat::data::SLOTS;
use glasnost_sim::combat::{veterancy, weapon};
use glasnost_sim::regions::{self, Reach};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

use crate::scripted::{self, dist2};

/// Cells from the spy within which an infantry unit to copy is looked for.
pub const COPY_REACH: i64 = 12; // nocite: tuning of our own opponent

/// The longest a spy is kept on its attack on one unit to copy.
pub const COPY_TIME: i32 = 300; // nocite: tuning of our own opponent

/// Cells beyond a detector's disc a spy going round it walks to.
pub const MARGIN: i32 = 2; // nocite: tuning of our own opponent

/// The longest a spy is kept on its walk round a detector.
pub const ROUND_TIME: i32 = 450; // nocite: tuning of our own opponent

/// Cells within which a spy has reached the cell it walks to.
const ARRIVED: i64 = 2; // nocite: tuning of our own opponent

/// The eight ways from a cell, clockwise from north, and their lengths in
/// sevenths of a step (a diagonal step is about √2 long: 5/7 of a step on
/// each axis keeps the cell about as far).
// nocite: the eight neighbour directions and a rounding of our own, not behaviour
const WAYS: [(i32, i32, i32); 8] = [
    (0, -1, 7),
    (1, -1, 5),
    (1, 0, 7),
    (1, 1, 5),
    (0, 1, 7),
    (-1, 1, 5),
    (-1, 0, 7),
    (-1, -1, 5),
];

/// What a disguiser does now.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Step {
    /// Attacking this unit to copy it.
    Copy(ObjectId),
    /// Walking to this cell, round a detector.
    Round(CellXY),
    /// Going in (or sent in).
    Enter,
}

/// One disguiser's job.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Job {
    /// The building it goes for.
    pub(crate) building: ObjectId,
    /// What it does now.
    pub(crate) step: Step,
    /// The tick of the step's order.
    pub(crate) at: i32,
    /// The units whose hit gave no disguise that fools the owner.
    pub(crate) tried: Vec<ObjectId>,
    /// Whether it went round a detector on this disguise.
    pub(crate) rounded: bool,
}

/// What the module remembers between thinks: the house's disguisers'
/// jobs.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Watch {
    pub(crate) jobs: BTreeMap<ObjectId, Job>,
}

impl Watch {
    /// Forgets the jobs of spies no longer the house's on the map, and of
    /// buildings gone or no longer seen.
    pub(crate) fn prune(&mut self, w: &World, house: HouseId) {
        self.jobs.retain(|&id, j| {
            w.obj(id).is_some_and(|o| o.owner == Some(house) && o.on_map)
                && w.obj(j.building).is_some_and(|b| b.on_map && b.health > 0)
                && crate::in_view(w, house, j.building)
        });
    }

    /// The buildings the house's disguisers go for.
    pub(crate) fn buildings(&self) -> Vec<ObjectId> {
        self.jobs.values().map(|j| j.building).collect()
    }
}

/// Whether spy `id` is a disguiser: a weapon of its type has the
/// disguise-flag warhead (S-0516 claim 31).
pub fn disguiser(w: &World, id: ObjectId) -> bool {
    // spec: S-0516 claim 31 (a projectile whose warhead has the disguise flag disguises its infantry firer)
    (0..SLOTS).any(|i| {
        weapon::slot_weapon(w, id, i)
            .and_then(|wp| w.rules().combat.warhead_of(wp))
            .is_some_and(|h| h.makes_disguise)
    })
}

/// Whether the disguise of the house's unit `id` passes the house
/// condition of S-0516 claim 35 for `victim`: its disguise house is none,
/// the victim, or a house the victim allies. Its own unit's disguise is
/// state the house knows.
pub fn fools(w: &World, id: ObjectId, victim: HouseId) -> bool {
    let Some(d) = w.obj(id).and_then(|o| o.cloak.disguise) else {
        return false;
    };
    // spec: S-0516 claim 35 (disguise house H, none, or one H allies)
    d.house.is_none() || d.house == Some(victim) || w.allied(Some(victim), d.house)
}

/// Whether the house the spy's disguise must fool is `h` or is fooled with
/// it: `victim` or a house it allies, not the spy's own house.
fn fooled_with(w: &World, house: HouseId, victim: HouseId, h: HouseId) -> bool {
    h != house && (h == victim || w.allied(Some(victim), Some(h)))
}

/// The nearest seen infantry unit of `victim` or a house it allies within
/// [`COPY_REACH`] cells of spy `id`, not in `tried` and not refused by
/// `ok`.
pub(crate) fn source(
    w: &World,
    house: HouseId,
    id: ObjectId,
    victim: HouseId,
    tried: &[ObjectId],
    ok: impl Fn(ObjectId) -> bool,
) -> Option<ObjectId> {
    let at = w.obj(id)?.pos.cell();
    w.objects
        .values()
        .filter(|o| {
            // spec: S-0516 claim 31 (an infantry firer copies only infantry)
            o.kind == Kind::Infantry
                && o.on_map
                && o.health > 0
                && o.owner.is_some_and(|h| fooled_with(w, house, victim, h))
                && dist2(o.pos.cell(), at) <= COPY_REACH * COPY_REACH
                && !tried.contains(&o.id)
        })
        .map(|o| o.id)
        .filter(|&x| crate::in_view(w, house, x) && ok(x))
        .min_by_key(|&x| (w.obj(x).map_or(i64::MAX, |o| dist2(o.pos.cell(), at)), x))
}

/// Whether spy `id` holds its attack on `src`.
pub(crate) fn copying(w: &World, id: ObjectId, src: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Attack) && o.target == Some(Target::Object(src))
    }) && w.obj(src).is_some_and(|o| o.on_map && o.health > 0)
}

/// Whether spy `id` is on its way to `c`.
pub(crate) fn walking(w: &World, id: ObjectId, c: CellXY) -> bool {
    w.obj(id).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Move)
            && o.dest == Some(Target::Cell(c))
            && dist2(o.pos.cell(), c) > ARRIVED * ARRIVED
    })
}

/// Whether spy `id` is on its way into building `b`.
pub(crate) fn entering(w: &World, id: ObjectId, b: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.beh.current == Some(Behaviour::Capture) && o.dest == Some(Target::Object(b))
    })
}

/// Whether cell `c` lies in the sensor disc of radius `r` around `k`
/// (S-0516 Terms).
fn in_disc(k: CellXY, r: i32, c: CellXY) -> bool {
    let (dx, dy) = (c.x - k.x, c.y - k.y);
    // spec: S-0516 Terms (sensor disc: −R ≤ dx ≤ R − 1, −R ≤ dy ≤ R − 1, dx² + dy² < R²)
    (-r..r).contains(&dx)
        && (-r..r).contains(&dy)
        && i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy)
            < i64::from(r) * i64::from(r)
}

/// The cells of the straight line from `a` to `b`, both ends included:
/// one a step along the longer axis.
fn line(a: CellXY, b: CellXY) -> impl Iterator<Item = CellXY> {
    let n = (b.x - a.x).abs().max((b.y - a.y).abs()).max(1);
    (0..=n).map(move |i| CellXY::new(a.x + (b.x - a.x) * i / n, a.y + (b.y - a.y) * i / n))
}

/// Steps from `a` to `b` along a straight line.
fn steps(a: CellXY, b: CellXY) -> i32 {
    (b.x - a.x).abs().max((b.y - a.y).abs())
}

/// The seen disguise detector buildings whose coverage would see through
/// a spy going into `victim`'s building: their centre cells and detection
/// ranges (S-0516 claim 40).
fn detectors(w: &World, house: HouseId, victim: HouseId) -> Vec<(CellXY, i32)> {
    w.objects
        .values()
        .filter(|o| {
            o.kind == Kind::Building
                && o.on_map
                && o.health > 0
                && o.owner.is_some_and(|h| fooled_with(w, house, victim, h))
        })
        .filter(|o| crate::in_view(w, house, o.id))
        .filter_map(|o| {
            // spec: S-0516 Import mapping (DetectDisguise absent: no; DetectDisguiseRange absent: 0)
            let detects = veterancy::type_combat(w, o.id).is_some_and(|t| t.detect_disguise);
            let r = w
                .rules()
                .buildings
                .get(o.type_index as usize)
                .map_or(0, |b| b.raw.detect_disguise_range.unwrap_or(0));
            (detects && r > 0).then(|| (cloak::centre_cell(w, o.id), r))
        })
        .collect()
}

/// The cell disguised spy `id` walks to before it goes into building `b`
/// of `victim`, round the seen detectors' discs, if going round crosses
/// fewer covered cells than the straight way.
pub(crate) fn round_cell(
    w: &World,
    house: HouseId,
    id: ObjectId,
    b: ObjectId,
    victim: HouseId,
) -> Option<CellXY> {
    let dets = detectors(w, house, victim);
    if dets.is_empty() {
        return None;
    }
    let o = w.obj(id)?;
    let from = o.pos.cell();
    let to = cloak::centre_cell(w, b);
    let covered = |c: CellXY| dets.iter().any(|&(k, r)| in_disc(k, r, c));
    let cost = |a: CellXY, z: CellXY| line(a, z).filter(|&c| covered(c)).count();
    let straight = cost(from, to);
    if straight == 0 {
        return None;
    }
    let zone = w.rules().unit(o.kind, o.type_index).map(|u| u.zone)?;
    let region = regions::reach_in(w, from, zone);
    if !matches!(region, Reach::Region(_)) {
        return None;
    }
    dets.iter()
        .filter(|&&(k, r)| line(from, to).any(|c| in_disc(k, r, c)))
        .flat_map(|&(k, r)| {
            WAYS.iter().map(move |&(dx, dy, len)| {
                // nocite: sevenths of a step (WAYS), rounded up on the diagonals
                let d = (r + MARGIN) * len / 7 + i32::from(len < 7);
                CellXY::new(k.x + dx * d, k.y + dy * d)
            })
        })
        .filter(|&c| {
            !covered(c) && w.cells.usable(c) && regions::reach_in(w, c, zone) == region
        })
        .map(|c| (cost(from, c) + cost(c, to), steps(from, c) + steps(c, to), c))
        .filter(|&(n, _, _)| n < straight)
        .min_by_key(|&(n, len, c)| (n, len, c.y, c.x))
        .map(|(_, _, c)| c)
}

/// Whether spy `id` may be given a building: idle, or standing on its own
/// in guard (an armed spy acquires targets by itself, S-0309) with no
/// destination.
pub(crate) fn free(w: &World, id: ObjectId) -> bool {
    scripted::idle(w, id)
        || w.obj(id).is_some_and(|o| {
            o.dest.is_none()
                && matches!(
                    o.beh.current,
                    None | Some(Behaviour::Guard) | Some(Behaviour::AreaGuard)
                )
        })
}
