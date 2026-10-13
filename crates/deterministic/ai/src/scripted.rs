//! A scripted house: a fixed build plan played through orders a human
//! could issue, for the headless skirmish and the determinism gate.
//!
//! Each think looks only at the house's own objects, its production lines
//! and the cells its shroud shows, and returns orders:
//!
//! 1. deploy the MCV (any vehicle that deploys into a building);
//! 2. build the plan's buildings in order — power plant, refinery, then each
//!    factory — and place each one when it is done, on the first legal cell
//!    of a square spiral around the construction yard;
//! 3. build another power plant whenever the drain reaches the output;
//! 4. keep the plan's harvester count, and send idle harvesters to harvest;
//! 5. build army units round-robin, and when a wave is ready attack-move it
//!    to the next enemy start position (lobby knowledge: the map's start
//!    waypoints), attacking enemies it sees on the way.
//!
//! The numbers here (think period, spiral size, wave size) are tuning of our
//! own opponent, not game behaviour.

use glasnost_map::CellXY;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::{deploy, placement, production, shroud, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, FactoryKind, Kind};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};

/// A build plan, by type name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Plan {
    /// The power plant.
    pub power: String,
    /// The refinery.
    pub refinery: String,
    /// Factories to build after the refinery, in order.
    pub factories: Vec<String>,
    /// The harvester.
    pub harvester: String,
    /// Harvesters to keep.
    pub harvesters: usize,
    /// Army types, built round-robin.
    pub army: Vec<String>,
    /// Idle army units that make a wave.
    pub wave: usize,
}

/// Ticks between two thinks.
pub const THINK_PERIOD: i32 = 15; // nocite: tuning of our own opponent

/// Cells within which an idle army unit attacks a seen enemy.
const ENGAGE: i64 = 10; // nocite: tuning of our own opponent

/// The largest ring of the placement spiral around the construction yard.
const SPIRAL_RINGS: i32 = 12; // nocite: tuning of our own opponent

/// A plan resolved against the rules.
#[derive(Clone, Debug)]
struct Types {
    power: TypeRef,
    refinery: TypeRef,
    factories: Vec<TypeRef>,
    harvester: TypeRef,
    army: Vec<TypeRef>,
}

/// A scripted house.
#[derive(Clone, Debug)]
pub struct Script {
    /// The house.
    pub house: HouseId,
    plan: Plan,
    types: Option<Types>,
    /// Enemy start cells, in visiting order.
    targets: Vec<CellXY>,
    /// The next target's index.
    next_target: usize,
    /// The next army type's index.
    next_army: usize,
}

fn type_ref(w: &World, name: &str) -> Option<TypeRef> {
    for kind in [Kind::Building, Kind::Vehicle, Kind::Infantry] {
        if let Some(index) = w.rules().index_of(kind, name) {
            return Some(TypeRef { kind, index });
        }
    }
    None
}

/// Whether an object is a construction yard (a factory of buildings).
fn is_yard(w: &World, o: &glasnost_sim::world::Object) -> bool {
    o.kind == Kind::Building
        && w.rules()
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|b| b.factory == Some(FactoryKind::Building))
}

fn idle(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        o.dest.is_none()
            && o.target.is_none()
            && matches!(
                o.beh.current,
                None | Some(Behaviour::Guard) | Some(Behaviour::AreaGuard) | Some(Behaviour::Stop)
            )
    })
}

/// The squared distance between two cells.
fn dist2(a: CellXY, b: CellXY) -> i64 {
    let dx = i64::from(a.x - b.x);
    let dy = i64::from(a.y - b.y);
    dx * dx + dy * dy
}

impl Script {
    /// A script for `house` playing `plan`; [`Script::set_starts`] gives
    /// it the cells to attack.
    pub fn new(house: HouseId, plan: Plan) -> Self {
        Self {
            house,
            plan,
            types: None,
            targets: Vec::new(),
            next_target: 0,
            next_army: 0,
        }
    }

    /// Sets the enemy start cells to visit: every start cell but the one
    /// nearest the house's first objects.
    pub fn set_starts(&mut self, w: &World, starts: &[CellXY]) {
        let home = self.home(w);
        self.targets = starts
            .iter()
            .copied()
            .filter(|&c| home.is_none_or(|h| dist2(c, h) > 25)) // nocite: tuning (own start within 5 cells)
            .collect();
        if let Some(h) = home {
            self.targets.sort_by_key(|&c| dist2(c, h));
        }
    }

    fn own(&self, w: &World) -> Vec<ObjectId> {
        w.objects
            .values()
            .filter(|o| o.owner == Some(self.house) && o.on_map && o.health > 0)
            .map(|o| o.id)
            .collect()
    }

    fn of_type(&self, w: &World, t: TypeRef) -> Vec<ObjectId> {
        self.own(w)
            .into_iter()
            .filter(|&id| w.obj(id).is_some_and(|o| tech::type_of(o) == t))
            .collect()
    }

    /// The house's construction yard or, before one exists, its first
    /// object's cell.
    fn home(&self, w: &World) -> Option<CellXY> {
        let own = self.own(w);
        own.iter()
            .find(|&&id| w.obj(id).is_some_and(|o| is_yard(w, o)))
            .or(own.first())
            .and_then(|&id| w.obj(id))
            .map(|o| o.pos.cell())
    }

    fn resolve(&mut self, w: &World) -> Option<Types> {
        if self.types.is_none() {
            let p = &self.plan;
            self.types = Some(Types {
                power: type_ref(w, &p.power)?,
                refinery: type_ref(w, &p.refinery)?,
                factories: p
                    .factories
                    .iter()
                    .map(|n| type_ref(w, n))
                    .collect::<Option<_>>()?,
                harvester: type_ref(w, &p.harvester)?,
                army: p
                    .army
                    .iter()
                    .map(|n| type_ref(w, n))
                    .collect::<Option<_>>()?,
            });
        }
        self.types.clone()
    }

    /// The orders of one think.
    pub fn think(&mut self, w: &World) -> Vec<Order> {
        let mut out = Vec::new();
        let Some(types) = self.resolve(w) else {
            return out;
        };
        let house = self.house;
        let own = self.own(w);

        // 1. Deploy the MCV.
        let has_yard = own
            .iter()
            .any(|&id| w.obj(id).is_some_and(|o| is_yard(w, o)));
        for &id in &own {
            let Some(o) = w.obj(id) else { continue };
            if o.kind == Kind::Vehicle
                && deploy::deploys_into(w, o.kind, o.type_index).is_some()
                && !has_yard
                && idle(w, id)
            {
                out.push(Order::Deploy { actor: id });
            }
        }

        // 2 and 3. Buildings.
        let p = &w.houses[usize::from(house.0)].econ.power;
        let want_power = p.output <= p.drain;
        let mut wanted: Option<TypeRef> = None;
        if want_power || self.of_type(w, types.power).is_empty() {
            wanted = Some(types.power);
        } else if self.of_type(w, types.refinery).is_empty() {
            wanted = Some(types.refinery);
        } else if let Some(&f) = types
            .factories
            .iter()
            .find(|&&f| self.of_type(w, f).is_empty())
        {
            wanted = Some(f);
        }
        // A done building waits to be placed, whatever is wanted now.
        let building_item = types
            .factories
            .iter()
            .chain([&types.power, &types.refinery])
            .find_map(|&t| production::line_of(w, house, t).and_then(|l| l.item.clone()));
        match building_item {
            Some(item) if item.done_tick.is_some() => {
                if let Some(cell) = self.place_cell(w, item.t) {
                    out.push(Order::Place {
                        what: item.t,
                        cell: Some(cell),
                    });
                }
            }
            Some(_) => {}
            None => {
                if let Some(t) = wanted.filter(|&t| tech::buildable(w, house, t)) {
                    out.push(Order::Build { what: t });
                }
            }
        }

        // 4. Harvesters.
        let harvesters = self.of_type(w, types.harvester);
        let refineries = self.of_type(w, types.refinery);
        if !refineries.is_empty() {
            for &h in &harvesters {
                if idle(w, h) {
                    out.push(Order::Behaviour {
                        actor: h,
                        behaviour: Behaviour::Harvest,
                        attack_move: false,
                        target: None,
                        dest: None,
                    });
                }
            }
        }
        let mut unit_wanted: Vec<TypeRef> = Vec::new();
        if !refineries.is_empty() && harvesters.len() < self.plan.harvesters {
            unit_wanted.push(types.harvester);
        }
        if !types.army.is_empty() {
            let t = types.army[self.next_army % types.army.len()];
            unit_wanted.push(t);
        }
        for t in unit_wanted {
            if !tech::buildable(w, house, t) {
                continue;
            }
            let busy = production::line_of(w, house, t)
                .is_some_and(|l| l.item.is_some() || !l.queue.is_empty());
            if !busy {
                out.push(Order::Build { what: t });
                if types.army.contains(&t) {
                    self.next_army += 1;
                }
            }
        }

        // 5. The army.
        let army: Vec<ObjectId> = own
            .iter()
            .copied()
            .filter(|&id| {
                w.obj(id)
                    .is_some_and(|o| types.army.contains(&tech::type_of(o)))
            })
            .collect();
        let enemies = self.seen_enemies(w);
        let mut waiting: Vec<ObjectId> = Vec::new();
        for id in army.iter().copied().filter(|&id| idle(w, id)) {
            let Some(at) = w.obj(id).map(|o| o.pos.cell()) else {
                continue;
            };
            let near = enemies
                .iter()
                .filter(|&&(_, c)| dist2(c, at) <= ENGAGE * ENGAGE)
                .min_by_key(|&&(_, c)| dist2(c, at));
            if let Some(&(e, _)) = near {
                out.push(Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::Attack,
                    attack_move: false,
                    target: Some(Target::Object(e)),
                    dest: None,
                });
            } else {
                waiting.push(id);
            }
        }
        if waiting.len() >= self.plan.wave {
            // The nearest seen enemy, else the next enemy start cell.
            let home = self.home(w);
            let goal = home
                .and_then(|h| enemies.iter().min_by_key(|&&(_, c)| dist2(c, h)))
                .map(|&(_, c)| c)
                .or_else(|| {
                    (!self.targets.is_empty()).then(|| {
                        let c = self.targets[self.next_target % self.targets.len()];
                        self.next_target += 1;
                        c
                    })
                });
            if let Some(c) = goal {
                for &id in &waiting {
                    out.push(Order::Behaviour {
                        actor: id,
                        behaviour: Behaviour::Move,
                        attack_move: true,
                        target: None,
                        dest: Some(Target::Cell(c)),
                    });
                }
            }
        }
        out
    }

    /// Enemy objects on cells the house sees, with their cells.
    fn seen_enemies(&self, w: &World) -> Vec<(ObjectId, CellXY)> {
        let me = &w.houses[usize::from(self.house.0)];
        w.objects
            .values()
            .filter(|o| o.on_map && o.health > 0)
            .filter_map(|o| {
                let owner = o.owner?;
                let h = w.houses.get(usize::from(owner.0))?;
                let enemy = owner != self.house
                    && !me.allies.contains(&owner)
                    && h.role == glasnost_sim::world::HouseRole::Player;
                let c = o.pos.cell();
                (enemy && shroud::seen(w, self.house, c)).then_some((o.id, c))
            })
            .collect()
    }

    /// The first cell of a square spiral around the construction yard where
    /// a place order for `t` is legal and the foundation is free.
    fn place_cell(&self, w: &World, t: TypeRef) -> Option<CellXY> {
        let home = self.home(w)?;
        for ring in 1..=SPIRAL_RINGS {
            for dy in -ring..=ring {
                for dx in -ring..=ring {
                    if dx.abs() != ring && dy.abs() != ring {
                        continue;
                    }
                    let c = CellXY::new(home.x + dx, home.y + dy);
                    if placement::place_legal(w, self.house, t, c) && self.free(w, t, c) {
                        return Some(c);
                    }
                }
            }
        }
        None
    }

    /// Whether the foundation's cells hold no object and keep a one-cell
    /// gap to other buildings, so that units can leave the factories.
    fn free(&self, w: &World, t: TypeRef, c: CellXY) -> bool {
        let cells = placement::foundation_cells(w, t.index, c);
        w.objects.values().filter(|o| o.on_map).all(|o| {
            // A building's position is its top-left cell's centre.
            let tl = o.pos.cell();
            if o.kind == Kind::Building {
                let (fw, fh) = o.foundation;
                cells
                    .iter()
                    .all(|k| k.x < tl.x - 1 || k.x > tl.x + fw || k.y < tl.y - 1 || k.y > tl.y + fh)
            } else {
                !cells.contains(&tl)
            }
        })
    }
}
