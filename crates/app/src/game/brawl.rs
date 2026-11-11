//! A staged battle for the benchmark's offscreen pictures (`brawl=N`): in
//! a running skirmish, two large mixed groups are created for the first
//! two player houses, between their bases, and kept attacking each other.
//!
//! Each house's group takes the house's own infantry and vehicle types
//! that are offered to it ([`glasnost_sim::econ::tech::offered`]), that
//! walk, drive or teleport and that carry a primary weapon, `N` of each
//! (at most [`MAX_PER_SIDE`] in all). The groups stand on the line between
//! the two houses' buildings, each [`GAP`] cells from the middle on its own
//! house's side, on the free cells nearest to its anchor. Every unit is
//! ordered (orders a person could give, through `orders::issue`) to
//! attack-move to the nearest unit of the other group, and so again every
//! [`REORDER`] ticks while it is idle, as in the arena tool.
//!
//! Our own tooling for pictures, like the benchmark: nothing here is game
//! behaviour, so no spec applies; the numbers are ours.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Behaviour, Kind, Locomotor};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, HouseRole, Layer, ObjectId, Pos, Target, World};

/// The most units in a group.
// nocite: our own tooling
pub const MAX_PER_SIDE: usize = 80;

/// Cells from the middle to each group's anchor.
// nocite: our own tooling
pub const GAP: i32 = 9;

/// Ticks between the re-orders of idle units.
// nocite: our own tooling
pub const REORDER: i32 = 15;

/// The two groups.
#[derive(Clone, Debug)]
pub struct Brawl {
    /// Each group's house.
    pub houses: [HouseId; 2],
    /// Each group's units, in creation order.
    pub units: [Vec<ObjectId>; 2],
    /// The types each group was made of, by name.
    pub types: [Vec<String>; 2],
    /// The middle cell.
    pub middle: CellXY,
    /// The tick of the last re-order.
    last: i32,
}

/// The mean cell of a house's buildings.
fn base_cell(w: &World, h: HouseId) -> Option<CellXY> {
    let (mut x, mut y, mut n) = (0i64, 0i64, 0i64);
    for o in w.objects.values() {
        if o.on_map && o.kind == Kind::Building && o.owner == Some(h) {
            let c = o.pos.cell();
            x += i64::from(c.x);
            y += i64::from(c.y);
            n += 1;
        }
    }
    (n > 0).then(|| CellXY::new((x / n) as i32, (y / n) as i32))
}

/// Whether a ground unit can be put on cell `c`: open land with nothing in
/// its ground layer and no bridge deck.
fn free(w: &World, c: CellXY) -> bool {
    if !w.cells.usable(c) || w.high_bridge(c) {
        return false;
    }
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    !matches!(
        cell.land,
        land::ROCK | land::TUNNEL | land::WALL | land::WATER | land::BEACH
    ) && !glasnost_sim::walls::is_wall(w, cell.overlay)
        && w.cells.list(c, Layer::Ground).is_empty()
}

/// The free cells nearest to `anchor`, ring by ring, `n` of them at most.
fn cells_near(w: &World, anchor: CellXY, n: usize) -> Vec<CellXY> {
    let mut out = Vec::new();
    for r in 0..40i32 {
        for dx in -r..=r {
            for dy in -r..=r {
                if dx.abs().max(dy.abs()) != r {
                    continue;
                }
                let c = CellXY::new(anchor.x + dx, anchor.y + dy);
                if free(w, c) && !out.contains(&c) {
                    out.push(c);
                    if out.len() >= n {
                        return out;
                    }
                }
            }
        }
    }
    out
}

/// The types of house `h`'s group.
fn types_of(w: &World, h: HouseId) -> Vec<(Kind, String)> {
    let r = w.rules();
    let mut out = Vec::new();
    for kind in [Kind::Infantry, Kind::Vehicle] {
        let list = if kind == Kind::Infantry {
            &r.infantry
        } else {
            &r.vehicles
        };
        for (i, u) in list.iter().enumerate() {
            let t = TypeRef {
                kind,
                index: i as u32,
            };
            if glasnost_sim::econ::tech::offered(w, h, t)
                && matches!(
                    u.locomotor,
                    Locomotor::Drive | Locomotor::Walk | Locomotor::Teleport
                )
                && u.primary.is_some()
            {
                out.push((kind, u.name.clone()));
            }
        }
    }
    out
}

/// The cell [`GAP`] cells from `middle` towards `(dx, dy)` (rounded down
/// on each axis; `middle` itself when the direction is zero).
pub fn anchor(middle: CellXY, (dx, dy): (i32, i32)) -> CellXY {
    let len = (i64::from(dx).pow(2) + i64::from(dy).pow(2)).isqrt().max(1);
    let step = |d: i32| (i64::from(d) * i64::from(GAP) / len) as i32;
    CellXY::new(middle.x + step(dx), middle.y + step(dy))
}

/// Creates the two groups; `None` when the game has no two player houses
/// with buildings.
pub fn stage(w: &mut World, per_type: usize) -> Option<Brawl> {
    let players: Vec<HouseId> = w
        .houses
        .iter()
        .enumerate()
        .filter(|(_, h)| h.role == HouseRole::Player)
        .map(|(i, _)| HouseId(i as u8))
        .filter(|&h| base_cell(w, h).is_some())
        .take(2)
        .collect();
    let [a, b] = players[..] else {
        return None;
    };
    let (ca, cb) = (base_cell(w, a)?, base_cell(w, b)?);
    let middle = CellXY::new((ca.x + cb.x) / 2, (ca.y + cb.y) / 2);
    let mut brawl = Brawl {
        houses: [a, b],
        middle,
        last: w.tick,
        units: [Vec::new(), Vec::new()],
        types: [Vec::new(), Vec::new()],
    };
    for (side, (h, home)) in [(a, ca), (b, cb)].into_iter().enumerate() {
        let (dx, dy) = (home.x - middle.x, home.y - middle.y);
        let anchor = anchor(middle, (dx, dy));
        let types = types_of(w, h);
        let mut roster = Vec::new();
        for _ in 0..per_type {
            for t in &types {
                if roster.len() < MAX_PER_SIDE {
                    roster.push(t.clone());
                }
            }
        }
        let cells = cells_near(w, anchor, roster.len());
        for ((kind, name), cell) in roster.iter().zip(cells) {
            let id = testkit::spawn(w, *kind, name, h.0, cell, 0);
            if w.obj(id).is_some_and(|o| o.on_map) {
                brawl.units[side].push(id);
            }
        }
        brawl.types[side] = types.into_iter().map(|(_, n)| n).collect();
    }
    brawl.reorder(w, true);
    Some(brawl)
}

impl Brawl {
    /// Group `side`'s units still there, alive and owned by its house.
    pub fn alive(&self, w: &World, side: usize) -> Vec<ObjectId> {
        self.units[side]
            .iter()
            .copied()
            .filter(|&id| {
                w.obj(id)
                    .is_some_and(|o| o.health > 0 && o.owner == Some(self.houses[side]))
            })
            .collect()
    }

    /// Orders the idle units (every unit with `all`) of both groups to
    /// attack-move to the nearest unit of the other group.
    pub fn reorder(&mut self, w: &mut World, all: bool) {
        self.last = w.tick;
        for side in 0..2 {
            let enemies: Vec<(ObjectId, Pos)> = self
                .alive(w, 1 - side)
                .into_iter()
                .filter_map(|id| w.obj(id).map(|o| (id, o.pos)))
                .collect();
            for id in self.alive(w, side) {
                let Some(o) = w.obj(id) else { continue };
                let idle = matches!(
                    o.beh.current,
                    None | Some(Behaviour::Guard) | Some(Behaviour::AreaGuard)
                ) && o.target.is_none()
                    && o.dest.is_none();
                if !(all || idle) {
                    continue;
                }
                let p = o.pos;
                let nearest = enemies.iter().min_by_key(|(eid, q)| {
                    let (dx, dy) = (i64::from(q.x - p.x), i64::from(q.y - p.y));
                    (dx * dx + dy * dy, *eid)
                });
                if let Some((_, q)) = nearest {
                    let order = Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Move,
                        attack_move: true,
                        target: None,
                        dest: Some(Target::Cell(q.cell())),
                    };
                    orders::issue(w, self.houses[side], order);
                }
            }
        }
    }

    /// Re-orders the idle units every [`REORDER`] ticks.
    pub fn tick(&mut self, w: &mut World) {
        if w.tick >= self.last + REORDER {
            self.reorder(w, false);
        }
    }

    /// The mean position of both groups' living units, if any.
    pub fn focus(&self, w: &World) -> Option<Pos> {
        let (mut x, mut y, mut z, mut n) = (0i64, 0i64, 0i64, 0i64);
        for side in 0..2 {
            for id in self.alive(w, side) {
                if let Some(o) = w.obj(id).filter(|o| o.on_map) {
                    x += i64::from(o.pos.x);
                    y += i64::from(o.pos.y);
                    z += i64::from(o.pos.z);
                    n += 1;
                }
            }
        }
        (n > 0).then(|| Pos::new((x / n) as i32, (y / n) as i32, (z / n) as i32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> World {
        testkit::game_with(&testkit::combat_ruleset(), 24, 1)
    }

    #[test]
    fn anchor_steps_gap_cells_towards_the_base() {
        let m = CellXY::new(50, 50);
        assert_eq!(anchor(m, (20, 0)), CellXY::new(50 + GAP, 50));
        assert_eq!(anchor(m, (0, -3)), CellXY::new(50, 50 - GAP));
        assert_eq!(anchor(m, (0, 0)), m);
        let a = anchor(m, (30, 30));
        assert_eq!(a.x - m.x, a.y - m.y);
        assert!((a.x - m.x) * (a.x - m.x) * 2 <= GAP * GAP);
    }

    #[test]
    fn no_brawl_without_two_bases() {
        let mut w = game();
        assert!(stage(&mut w, 2).is_none());
    }

    #[test]
    fn cells_near_skips_occupied_cells_and_grows_in_rings() {
        let mut w = game();
        let c = testkit::mid(24, 0, 0);
        testkit::spawn(&mut w, Kind::Vehicle, "CRUSHER", 0, c, 0);
        let cells = cells_near(&w, c, 9);
        assert_eq!(cells.len(), 9);
        assert!(!cells.contains(&c));
        assert!(
            cells
                .iter()
                .all(|d| (d.x - c.x).abs().max((d.y - c.y).abs()) <= 2)
        );
    }

    #[test]
    fn reorder_sends_both_groups_at_each_other() {
        let mut w = game();
        let a = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "CRUSHER",
            0,
            testkit::mid(24, -4, 0),
            0,
        );
        let b = testkit::spawn(
            &mut w,
            Kind::Vehicle,
            "CRUSHER",
            1,
            testkit::mid(24, 4, 0),
            0,
        );
        let mut brawl = Brawl {
            houses: [HouseId(0), HouseId(1)],
            units: [vec![a], vec![b]],
            types: [Vec::new(), Vec::new()],
            middle: testkit::mid(24, 0, 0),
            last: w.tick,
        };
        // Orders are taken only once the game lets them in (as in the
        // arena tool); a running skirmish is long past that.
        while orders::blocked(&w) {
            testkit::run(&mut w, 1);
        }
        let f = brawl.focus(&w).unwrap();
        let (pa, pb) = (w.obj(a).unwrap().pos, w.obj(b).unwrap().pos);
        assert_eq!(f.x, (pa.x + pb.x) / 2);
        brawl.reorder(&mut w, true);
        testkit::run(&mut w, 1);
        for id in [a, b] {
            let o = w.obj(id).unwrap();
            assert!(
                o.target.is_some() || o.dest.is_some(),
                "{id:?} was not sent: {:?}",
                o.beh.current
            );
        }
        assert_eq!(brawl.alive(&w, 0), vec![a]);
    }
}
