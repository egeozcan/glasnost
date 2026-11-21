//! The object under the cursor (S-0214 claim 35): the pointing distance of
//! F10, the nearest candidate in reach, and the exclusions of claim 35 (d)
//! that apply to an object the viewer does not own.

use glasnost_sim::rules::Kind;
use glasnost_sim::world::{HouseId, ObjectId, World};
use glasnost_ui::geom::Point;

/// A candidate is in reach when its pointing distance is below this.
pub const POINTING_REACH: i64 = 200; // spec: S-0214 Constants (pointing reach 200, truncated pointing distance) and claim 35 (a)

/// F10's pointing distance *d* from the pointer's pixel to a candidate's
/// point, both in the same screen coordinates.
pub fn distance(point: Point, pointer: Point) -> i64 {
    let dx = i64::from(point.x) - i64::from(pointer.x);
    let dy = i64::from(point.y) - i64::from(pointer.y);
    // spec: S-0214 F10 (d = Δx² + ⌊Δy²/2⌋, Δ = point − pointer; the vertical weight 1/2, exact in integers)
    dx * dx + (dy * dy) / 2
}

/// The candidate with the smallest pointing distance below the reach, the
/// first in `candidates`' order between equal distances (claim 35 (a)),
/// passing over those `skipped` gives (claim 35 (b)).
pub fn nearest(
    candidates: &[(ObjectId, Point)],
    pointer: Point,
    skipped: impl Fn(ObjectId) -> bool,
) -> Option<ObjectId> {
    let mut best: Option<(i64, ObjectId)> = None;
    for &(id, point) in candidates {
        if skipped(id) {
            continue;
        }
        let d = distance(point, pointer);
        // spec: S-0214 claim 35 (a) (in reach strictly below 200; on a tie the one first in claim 25's order is kept: only a strictly smaller d replaces it)
        if d < POINTING_REACH && best.is_none_or(|(b, _)| d < b) {
            best = Some((d, id));
        }
    }
    best.map(|(_, id)| id)
}

/// Whether viewer `v` can hover and click the object as far as its cloak
/// goes (S-0516 claim 26): an object `v` owns always, another only where
/// [`glasnost_sim::cloak::view::clickable`] allows it.
pub fn cloak_clickable(w: &World, id: ObjectId, v: Option<HouseId>) -> bool {
    // spec: S-0516 r36 claim 26 (none of the refusals applies to an object that V owns: V hovers and clicks its own building at fade level 15, and an order whose target is it takes it as its target)
    let owner = v.is_some() && w.obj(id).is_some_and(|o| o.owner == v);
    owner || glasnost_sim::cloak::view::clickable(w, id, v)
}

/// Whether a viewer that does not own the chosen object can find it
/// (S-0214 claim 35 (d)): not fully cloaked and unsensed, nor a building at
/// fade level 15 unsensed (S-0516 claim 26), nor of an invisible type, nor
/// a building of a type invisible in the game.
pub fn findable(w: &World, id: ObjectId, viewer: Option<HouseId>) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0214 claim 35 (d) and S-0516 claim 26 (fully cloaked and not sensed; a building at fade level 15 not sensed)
    if !glasnost_sim::cloak::view::clickable(w, id, viewer) {
        return false;
    }
    let r = w.rules();
    // spec: S-0214 Import mapping (`Invisible` on an infantry, vehicle, aircraft or building type: invisible type, absent no; `InvisibleInGame` on a building type: invisible in the game, absent no)
    let (invisible, in_game) = match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .map_or((false, false), |b| {
                (
                    b.raw.econ.invisible.unwrap_or(false),
                    b.raw.invisible_in_game.unwrap_or(false),
                )
            }),
        Kind::Terrain => (false, false),
        _ => r.unit(o.kind, o.type_index).map_or((false, false), |u| {
            (u.raw.econ.invisible.unwrap_or(false), false)
        }),
    };
    !invisible && !in_game
}
