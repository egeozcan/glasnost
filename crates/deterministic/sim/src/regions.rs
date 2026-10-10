//! Connectivity: regions and reach regions (S-0212 claims 1 to 5).
//!
//! Regions are rebuilt from the passability categories whenever a category
//! changes (claim 5). The rebuild follows the rule of claim 3 (our engine's
//! departure D4 from the original's labelling order).

use glasnost_map::CellXY;
use glasnost_rules::sim::MovementZone;

use crate::passability::{self, Answer, category};
use crate::world::World;

// spec: S-0211 Terms (the thirteen movement zones)
/// The thirteen movement zones in a fixed order.
pub const ZONES: [MovementZone; 13] = [
    MovementZone::Normal,
    MovementZone::Crusher,
    MovementZone::Destroyer,
    MovementZone::AmphibiousDestroyer,
    MovementZone::AmphibiousCrusher,
    MovementZone::Amphibious,
    MovementZone::Subterranean,
    MovementZone::Infantry,
    MovementZone::InfantryDestroyer,
    MovementZone::Fly,
    MovementZone::Water,
    MovementZone::WaterBeach,
    MovementZone::CrusherAll,
];

fn zone_index(z: MovementZone) -> usize {
    ZONES.iter().position(|x| *x == z).unwrap_or(0)
}

/// Connectivity derived from the categories (not saved; rebuilt on load).
#[derive(Clone, Debug, Default)]
pub struct Regions {
    /// Region label by dense cell index (`u32::MAX`: off-map or illegal).
    pub region: Vec<u32>,
    /// Reach-region label per zone, by region (`u32::MAX`: no reach region).
    pub reach: Vec<[u32; 13]>, // spec: S-0212 claim 4 (a reach region per movement zone)
    /// The categories changed since the last rebuild.
    pub dirty: bool,
}

/// Where a cell lies for one zone, as claim 4 compares them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// An off-map cell.
    OffMap,
    /// A cell of no reach region.
    None,
    /// A reach region.
    Region(u32),
}

struct Dsu(Vec<u32>);

impl Dsu {
    fn new(n: usize) -> Self {
        Self((0..n as u32).collect())
    }
    fn find(&mut self, x: u32) -> u32 {
        let mut r = x;
        while self.0[r as usize] != r {
            r = self.0[r as usize];
        }
        let mut x = x;
        while self.0[x as usize] != r {
            let n = self.0[x as usize];
            self.0[x as usize] = r;
            x = n;
        }
        r
    }
    fn union(&mut self, a: u32, b: u32) {
        let (ra, rb) = (self.find(a), self.find(b));
        if ra != rb {
            // Deterministic: the smaller label wins.
            let (lo, hi) = if ra < rb { (ra, rb) } else { (rb, ra) };
            self.0[hi as usize] = lo;
        }
    }
}

/// The eight neighbour steps (S-0204 claim 9; S-0212 Terms *step direction*).
// spec: S-0204 Constants (neighbour steps of 8-directions 0 … 7)
pub const STEPS: [(i32, i32); 8] = [
    (0, -1),
    (1, -1),
    (1, 0),
    (1, 1),
    (0, 1),
    (-1, 1),
    (-1, 0),
    (-1, -1),
];

/// Rebuilds the regions from the categories.
pub fn rebuild(w: &mut World) {
    let q = w.rules().quirks;
    let n = w.cells.grid.len();
    let cells: Vec<CellXY> = w.cells.grid.iter().map(|(c, _)| c).collect();
    let mut dsu = Dsu::new(n);
    let level = |w: &World, c: CellXY| w.cells.cell(c).map_or(0, |x| i32::from(x.level));
    for &c in &cells {
        let Some(i) = w.cells.idx(c) else { continue };
        let cat = w.cells.category[i];
        // spec: S-0212 claim 3 (off-map cells belong to no region)
        if cat == category::OFF_MAP {
            continue;
        }
        for (d, (dx, dy)) in STEPS.iter().enumerate() {
            let nc = CellXY::new(c.x + dx, c.y + dy);
            let Some(j) = w.cells.idx(nc) else { continue };
            if w.cells.category[j] != cat {
                continue;
            }
            let diff = (level(w, c) - level(w, nc)).abs();
            // spec: S-0212 claim 3 (same category joins within 1 level; along x also 2 or 3 with quirks.region_join_height_asymmetry)
            let joins =
                diff <= 1 || (q.region_join_height_asymmetry && (d == 2 || d == 6) && diff <= 3);
            if joins {
                dsu.union(i as u32, j as u32);
            }
        }
    }
    // Label regions densely in index order.
    let mut region = vec![u32::MAX; n];
    let mut labels: std::collections::BTreeMap<u32, u32> = Default::default();
    let mut cats = Vec::new();
    for &c in &cells {
        let Some(i) = w.cells.idx(c) else { continue };
        if w.cells.category[i] == category::OFF_MAP {
            continue;
        }
        let root = dsu.find(i as u32);
        let next = labels.len() as u32;
        let l = *labels.entry(root).or_insert_with(|| {
            cats.push(w.cells.category[i]);
            next
        });
        region[i] = l;
    }
    // Region adjacency (S-0212 claim 3).
    let nr = labels.len();
    let mut adj: Vec<(u32, u32)> = Vec::new();
    for &c in &cells {
        let Some(i) = w.cells.idx(c) else { continue };
        let ri = region[i];
        if ri == u32::MAX {
            continue;
        }
        for (dx, dy) in STEPS {
            let nc = CellXY::new(c.x + dx, c.y + dy);
            let Some(j) = w.cells.idx(nc) else { continue };
            let rj = region[j];
            if rj == u32::MAX || rj == ri {
                continue;
            }
            let diff = (level(w, c) - level(w, nc)).abs();
            let blocked =
                cats[ri as usize] == category::BLOCKED || cats[rj as usize] == category::BLOCKED;
            // spec: S-0212 claim 3 (adjacent within 1 level, or whatever the heights when one region is blocked)
            if diff <= 1 || blocked {
                adj.push((ri.min(rj), ri.max(rj)));
            }
        }
    }
    adj.sort_unstable();
    adj.dedup();
    let mut reach = vec![[u32::MAX; 13]; nr]; // spec: S-0212 claim 4 (a reach region per movement zone)
    for (zi, &z) in ZONES.iter().enumerate() {
        let mut d = Dsu::new(nr);
        let yes = |r: u32| passability::zone_table(z, cats[r as usize]) == Answer::Yes;
        for &(a, b) in &adj {
            // spec: S-0212 claim 4 (yes regions adjacent to one another form one reach region)
            if yes(a) && yes(b) {
                d.union(a, b);
            }
        }
        for r in 0..nr as u32 {
            if yes(r) {
                reach[r as usize][zi] = d.find(r);
            }
        }
    }
    w.regions = Regions {
        region,
        reach,
        dirty: false,
    };
}

/// Recomputes a cell's category after a change; marks the regions dirty
/// when it changed (S-0212 claim 5).
pub fn cell_changed(w: &mut World, c: CellXY) {
    let cat = passability::compute_category(w, c);
    if let Some(i) = w.cells.idx(c)
        && w.cells.category[i] != cat
    {
        w.cells.category[i] = cat;
        w.regions.dirty = true;
    }
}

/// Where a cell lies for a zone (S-0212 claim 4), rebuilding first if dirty.
pub fn reach(w: &mut World, c: CellXY, z: MovementZone) -> Reach {
    ensure(w);
    reach_in(w, c, z)
}

/// Rebuilds the regions if a category changed since the last rebuild.
pub fn ensure(w: &mut World) {
    if w.regions.dirty || w.regions.region.is_empty() {
        rebuild(w);
    }
}

/// Where a cell lies for a zone, from regions that [`ensure`] has brought
/// up to date (every tick ends with them up to date).
pub fn reach_in(w: &World, c: CellXY, z: MovementZone) -> Reach {
    debug_assert!(!w.regions.dirty, "regions queried while out of date");
    let Some(i) = w.cells.idx(c) else {
        return Reach::OffMap;
    };
    if w.cells.category[i] == category::OFF_MAP {
        return Reach::OffMap;
    }
    let r = w.regions.region[i];
    match w.regions.reach.get(r as usize).map(|x| x[zone_index(z)]) {
        Some(x) if x != u32::MAX => Reach::Region(x),
        _ => Reach::None,
    }
}
