//! A scripted house: a simple build plan played through orders a human
//! could issue, for the headless skirmish and the determinism gate.
//!
//! Each think looks only at the house's own objects, its production lines,
//! what its sidebar offers and what stands on the cells its shroud shows
//! (every explored cell: fog of war is off in a skirmish, S-0409 claim 19),
//! and returns orders:
//!
//! 1. deploy the MCV (any vehicle that deploys into a building) while the
//!    house has no construction yard;
//! 2. build a power plant, a refinery, then a factory of infantry and one of
//!    vehicles (skipping, for now, a role the house cannot build a type for
//!    yet), and place each one when it is done, on the first cell of a
//!    square spiral around the construction yard that passes the place
//!    order's conditions and the cell test;
//! 3. build another power plant whenever the drain reaches the output;
//! 4. keep the plan's harvester count, and send idle or resting harvesters
//!    to harvest;
//! 5. once the harvesters are kept, build army units round-robin, and when
//!    a wave is ready attack-move it to the nearest seen enemy or the next enemy start position (lobby
//!    knowledge: the map's start waypoints), and once every start position
//!    has been visited, to the nearest cell of a coarse search grid that
//!    the house does not see now; idle units attack seen enemies near them.
//!    A smaller wave goes too once units have waited long enough, so a house
//!    that cannot afford a full wave still attacks.
//!
//! Types are found by role from the rules (a building with power output, the
//! refinery flag, a factory kind; a harvester; a unit with a primary weapon)
//! among those the house can build now, the cheapest first; a plan can name
//! the types to use instead.
//!
//! The numbers here (think period, spiral size, engage distance, wave patience) are tuning
//! of our own opponent, not game behaviour.

use glasnost_map::CellXY;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::{deploy, placement, production, shroud, tech};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::{Behaviour, FactoryKind, Kind, Locomotor};
use glasnost_sim::world::{HouseId, ObjectId, Target, World};
use serde::{Deserialize, Serialize};

/// A build plan.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Plan {
    /// Type names to use instead of finding types by role; an empty list
    /// leaves that role to the rules.
    pub names: Names,
    /// Harvesters to keep.
    pub harvesters: usize,
    /// Idle army units that make a wave.
    pub wave: usize,
}

/// Type names of a plan, by role.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Names {
    /// Power plants.
    pub power: Vec<String>,
    /// Refineries.
    pub refinery: Vec<String>,
    /// Factories.
    pub factories: Vec<String>,
    /// Harvesters.
    pub harvester: Vec<String>,
    /// Army units.
    pub army: Vec<String>,
}

/// Ticks between two thinks.
pub const THINK_PERIOD: i32 = 15; // nocite: tuning of our own opponent

/// Cells within which an idle army unit attacks a seen enemy.
const ENGAGE: i64 = 10; // nocite: tuning of our own opponent

/// Ticks after which idle army units go as a wave even when fewer than the
/// plan's wave size.
const WAVE_PATIENCE: i32 = 3000; // nocite: tuning of our own opponent

/// Spacing of the search grid that waves sweep once every enemy start
/// position has been visited.
const SEARCH_STEP: i32 = 8; // nocite: tuning of our own opponent

/// Free cells kept between the house's buildings.
const GAP: i32 = 2; // nocite: tuning of our own opponent

/// The largest ring of the placement spiral around the construction yard.
pub(crate) const SPIRAL_RINGS: i32 = 12; // nocite: tuning of our own opponent

/// What a type is for in the plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// A construction yard (a factory of buildings).
    Yard,
    /// A power plant.
    Power,
    /// A refinery.
    Refinery,
    /// A factory of infantry or vehicles.
    Factory(FactoryKind),
    /// A harvester.
    Harvester,
    /// An armed ground unit.
    Army,
}

/// The factories the plan builds, in order.
const FACTORIES: [FactoryKind; 2] = [FactoryKind::Infantry, FactoryKind::Vehicle]; // nocite: the script's own build order, not game behaviour

/// Every building, vehicle and infantry type.
pub fn all_types(w: &World) -> Vec<TypeRef> {
    let r = w.rules();
    let n = |k: Kind| match k {
        Kind::Building => r.buildings.len(),
        Kind::Vehicle => r.vehicles.len(),
        _ => r.infantry.len(),
    };
    [Kind::Building, Kind::Vehicle, Kind::Infantry]
        .into_iter()
        .flat_map(|kind| (0..n(kind) as u32).map(move |index| TypeRef { kind, index }))
        .collect()
}

/// A type's name.
fn name_of(w: &World, t: TypeRef) -> &str {
    let r = w.rules();
    match t.kind {
        Kind::Building => r.buildings.get(t.index as usize).map(|b| b.name.as_str()),
        Kind::Vehicle => r.vehicles.get(t.index as usize).map(|u| u.name.as_str()),
        Kind::Infantry => r.infantry.get(t.index as usize).map(|u| u.name.as_str()),
        _ => None,
    }
    .unwrap_or("")
}

/// The role type `t` plays in the plan, if any (the first that fits).
pub fn role_of(w: &World, t: TypeRef) -> Option<Role> {
    let roles = [Role::Yard, Role::Power, Role::Refinery]
        .into_iter()
        .chain(FACTORIES.map(Role::Factory))
        .chain([Role::Harvester, Role::Army]);
    roles.into_iter().find(|&r| plays(w, t, r))
}

/// Whether type `t` plays `role`.
fn plays(w: &World, t: TypeRef, role: Role) -> bool {
    let r = w.rules();
    match t.kind {
        Kind::Building => {
            let Some(b) = r.buildings.get(t.index as usize) else {
                return false;
            };
            match role {
                Role::Power => b.factory.is_none() && b.raw.econ.power.unwrap_or(0) > 0,
                // The refinery flag, or a member of the refinery group
                // (S-0407 Import mapping, PROC).
                Role::Refinery => {
                    b.raw.econ.refinery == Some(true)
                        || r.econ.tech.groups[tech::Group::Proc as usize].contains(&t.index)
                }
                Role::Factory(k) => b.factory == Some(k) && !b.naval,
                Role::Yard => b.factory == Some(FactoryKind::Building),
                _ => false,
            }
        }
        Kind::Vehicle | Kind::Infantry => {
            let list = if t.kind == Kind::Vehicle {
                &r.vehicles
            } else {
                &r.infantry
            };
            let Some(u) = list.get(t.index as usize) else {
                return false;
            };
            let ground = matches!(t.kind, Kind::Infantry)
                || deploy::deploys_into(w, t.kind, t.index).is_none();
            // Only types the simulation moves: driving, walking and
            // teleporting vehicles and infantry (an aircraft type with the
            // teleporting locomotor does not move, Q-0232).
            let moves = u.locomotor != Locomotor::None;
            match role {
                Role::Harvester => u.harvester && moves,
                Role::Army => u.primary.is_some() && !u.harvester && ground && moves,
                _ => false,
            }
        }
        _ => false,
    }
}

/// Whether an object is a construction yard (a factory of buildings).
pub(crate) fn is_yard(w: &World, o: &glasnost_sim::world::Object) -> bool {
    o.kind == Kind::Building
        && w.rules()
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|b| b.factory == Some(FactoryKind::Building))
}

/// Cells within which a unit moving to a cell counts as arrived, so that a
/// crowd jammed around its goal is free for the next wave.
const ARRIVED: i64 = 2; // nocite: tuning of our own opponent

/// Whether a unit has nothing to do: no destination (or a cell destination
/// it has as good as reached), no target, and a resting behaviour (or a
/// move that has arrived).
pub(crate) fn idle(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        let arrived = match o.dest {
            None => true,
            Some(Target::Cell(c)) => {
                o.beh.current == Some(Behaviour::Move)
                    && dist2(c, o.pos.cell()) <= ARRIVED * ARRIVED
            }
            Some(_) => false,
        };
        arrived
            && o.target.is_none()
            && matches!(
                o.beh.current,
                None | Some(Behaviour::Guard)
                    | Some(Behaviour::AreaGuard)
                    | Some(Behaviour::Stop)
                    | Some(Behaviour::Move)
            )
    })
}

/// The squared distance between two cells.
pub(crate) fn dist2(a: CellXY, b: CellXY) -> i64 {
    let dx = i64::from(a.x - b.x);
    let dy = i64::from(a.y - b.y);
    dx * dx + dy * dy
}

/// A scripted house.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Script {
    /// The house.
    pub house: HouseId,
    plan: Plan,
    /// Enemy start cells, in visiting order.
    targets: Vec<CellXY>,
    /// The next target's index.
    next_target: usize,
    /// The next army type's index.
    next_army: usize,
    /// The tick since which idle army units have been waiting for a wave.
    waiting_since: Option<i32>,
}

impl Script {
    /// A script for `house` playing `plan`; [`Script::set_starts`] gives
    /// it the cells to attack.
    pub fn new(house: HouseId, plan: Plan) -> Self {
        Self {
            house,
            plan,
            targets: Vec::new(),
            next_target: 0,
            next_army: 0,
            waiting_since: None,
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

    /// Whether `t` may be used for `role` under the plan's names.
    fn allowed(&self, w: &World, t: TypeRef, role: Role) -> bool {
        let n = &self.plan.names;
        let list = match role {
            Role::Power => &n.power,
            Role::Refinery => &n.refinery,
            Role::Yard | Role::Factory(_) => &n.factories,
            Role::Harvester => &n.harvester,
            Role::Army => &n.army,
        };
        plays(w, t, role) && (list.is_empty() || list.iter().any(|x| x == name_of(w, t)))
    }

    /// The types the house can build now for `role`, cheapest first.
    pub(crate) fn candidates(&self, w: &World, role: Role) -> Vec<TypeRef> {
        let mut v: Vec<TypeRef> = all_types(w)
            .into_iter()
            .filter(|&t| self.allowed(w, t, role) && tech::buildable(w, self.house, t))
            .collect();
        v.sort_by_key(|&t| (production::cost_field(w, t), t.kind as u8, t.index));
        v
    }

    pub(crate) fn own(&self, w: &World) -> Vec<ObjectId> {
        w.objects
            .values()
            .filter(|o| o.owner == Some(self.house) && o.on_map && o.health > 0)
            .map(|o| o.id)
            .collect()
    }

    /// The house's objects on the map that play `role`.
    pub(crate) fn of_role(&self, w: &World, role: Role) -> Vec<ObjectId> {
        self.own(w)
            .into_iter()
            .filter(|&id| {
                w.obj(id)
                    .is_some_and(|o| self.allowed(w, tech::type_of(o), role))
            })
            .collect()
    }

    /// The house's construction yard or, before one exists, its first
    /// object's cell.
    pub(crate) fn home(&self, w: &World) -> Option<CellXY> {
        let own = self.own(w);
        own.iter()
            .find(|&&id| w.obj(id).is_some_and(|o| is_yard(w, o)))
            .or(own.first())
            .and_then(|&id| w.obj(id))
            .map(|o| o.pos.cell())
    }

    /// Whether the line of `t` holds an item or a queue.
    pub(crate) fn busy(&self, w: &World, t: TypeRef) -> bool {
        production::line_of(w, self.house, t)
            .is_some_and(|l| l.item.is_some() || !l.queue.is_empty())
    }

    /// Orders a build of the cheapest type the house can build for `role`
    /// when its line is free; returns the type ordered.
    fn build(&self, w: &World, role: Role, out: &mut Vec<Order>) -> Option<TypeRef> {
        let t = *self.candidates(w, role).first()?;
        if self.busy(w, t) {
            return None;
        }
        out.push(Order::Build { what: t });
        Some(t)
    }

    /// The orders of one think.
    pub fn think(&mut self, w: &World) -> Vec<Order> {
        let mut out = Vec::new();
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

        // 2 and 3. Buildings: a done building is placed first; otherwise the
        // next wanted one is ordered.
        let done = w.econ.lines.lines.iter().find_map(|l| {
            let item = l.item.as_ref()?;
            (l.house == house && item.t.kind == Kind::Building && item.done_tick.is_some())
                .then_some(item.t)
        });
        if let Some(t) = done {
            if let Some(cell) = self.place_cell(w, t) {
                out.push(Order::Place {
                    what: t,
                    cell: Some(cell),
                });
            }
        } else {
            let p = &w.houses[usize::from(house.0)].econ.power;
            // The first missing role the house can build a type for now
            // (a role it cannot build yet, such as a refinery that needs a
            // factory first, does not hold up the others).
            let low_power = p.output <= p.drain || self.of_role(w, Role::Power).is_empty();
            let wanted = low_power
                .then_some(Role::Power)
                .into_iter()
                .chain(
                    self.of_role(w, Role::Refinery)
                        .is_empty()
                        .then_some(Role::Refinery),
                )
                .chain(
                    FACTORIES
                        .into_iter()
                        .map(Role::Factory)
                        .filter(|&f| self.of_role(w, f).is_empty()),
                )
                .find(|&r| !self.candidates(w, r).is_empty());
            if let Some(role) = wanted {
                self.build(w, role, &mut out);
            }
        }

        // 4. Harvesters.
        let harvesters = self.of_role(w, Role::Harvester);
        let refineries = self.of_role(w, Role::Refinery);
        if !refineries.is_empty() {
            // A harvester that rests (whatever destination it kept) or is
            // idle is sent to harvest.
            for &h in &harvesters {
                let resting = w.obj(h).is_some_and(|o| {
                    matches!(
                        o.beh.current,
                        None | Some(Behaviour::Guard)
                            | Some(Behaviour::AreaGuard)
                            | Some(Behaviour::Stop)
                    )
                });
                if resting || idle(w, h) {
                    out.push(Order::Behaviour {
                        actor: h,
                        behaviour: Behaviour::Harvest,
                        attack_move: false,
                        target: None,
                        dest: None,
                    });
                }
            }
            if harvesters.len() < self.plan.harvesters {
                self.build(w, Role::Harvester, &mut out);
            }
        }

        // The army: one type round-robin per kind of factory, once the
        // harvesters are kept (while a harvester can still be had: a
        // harvester type to build now, or a vehicle factory to build while
        // the house has none).
        let vehicle_factory = Role::Factory(FactoryKind::Vehicle);
        let economy_first = harvesters.len() < self.plan.harvesters
            && (!self.candidates(w, Role::Harvester).is_empty()
                || (self.of_role(w, vehicle_factory).is_empty()
                    && !self.candidates(w, vehicle_factory).is_empty()));
        let army_types = if economy_first {
            Vec::new()
        } else {
            self.candidates(w, Role::Army)
        };
        for kind in [Kind::Vehicle, Kind::Infantry] {
            let of_kind: Vec<TypeRef> = army_types
                .iter()
                .copied()
                .filter(|t| t.kind == kind)
                .collect();
            if of_kind.is_empty() {
                continue;
            }
            let t = of_kind[self.next_army % of_kind.len()];
            if !self.busy(w, t) {
                out.push(Order::Build { what: t });
                self.next_army += 1;
            }
        }

        // 5. Army units.
        let army = self.of_role(w, Role::Army);
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
        let since = *self.waiting_since.get_or_insert(w.tick);
        if waiting.is_empty() {
            self.waiting_since = None;
        }
        let patient = !waiting.is_empty() && w.tick - since >= WAVE_PATIENCE;
        if waiting.len() >= self.plan.wave.max(1) || patient {
            self.waiting_since = None;
            // The nearest seen enemy, else the next enemy start cell.
            let home = self.home(w);
            let from = w.obj(waiting[0]).map(|o| o.pos.cell());
            let goal = home
                .and_then(|h| enemies.iter().min_by_key(|&&(_, c)| dist2(c, h)))
                .map(|&(_, c)| c)
                .or_else(|| {
                    (self.next_target < self.targets.len()).then(|| {
                        let c = self.targets[self.next_target];
                        self.next_target += 1;
                        c
                    })
                })
                .or_else(|| from.and_then(|f| self.unseen_cell(w, f)))
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

    /// The nearest cell of the search grid, from `from`, that is in the
    /// usable area and that the house does not see now; the first in cell
    /// order on equal distance.
    fn unseen_cell(&self, w: &World, from: CellXY) -> Option<CellXY> {
        w.cells
            .grid
            .iter()
            .map(|(c, _)| c)
            .filter(|c| c.x % SEARCH_STEP == 0 && c.y % SEARCH_STEP == 0)
            .filter(|&c| w.cells.usable(c) && !shroud::seen(w, self.house, c))
            .min_by_key(|&c| (dist2(c, from), c.y, c.x))
    }

    /// Enemy objects on cells the house has explored, with their cells: fog
    /// of war is off in a skirmish (S-0409 claim 19), so a person in the
    /// house's seat sees what stands on every explored cell.
    pub(crate) fn seen_enemies(&self, w: &World) -> Vec<(ObjectId, CellXY)> {
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
                (enemy && shroud::explored(w, self.house, c)).then_some((o.id, c))
            })
            .collect()
    }

    /// The first cell of a square spiral around the construction yard where
    /// a place order for `t` is legal, the cell test passes and the
    /// foundation is free.
    pub(crate) fn place_cell(&self, w: &World, t: TypeRef) -> Option<CellXY> {
        let home = self.home(w)?;
        // The place order's conditions and the cell test (S-0406 claims 3 to 6, 8 and 12); an upgrade needs only the former.
        let upgrade = placement::place_type(w, t.index).upgrade_of.is_some();
        for ring in 1..=SPIRAL_RINGS {
            for dy in -ring..=ring {
                for dx in -ring..=ring {
                    if dx.abs() != ring && dy.abs() != ring {
                        continue;
                    }
                    let c = CellXY::new(home.x + dx, home.y + dy);
                    if placement::place_legal(w, self.house, t, c)
                        && (upgrade
                            || placement::cell_test(
                                w,
                                t.index,
                                c,
                                &placement::Test::order(self.house),
                            ))
                        && self.free(w, t, c)
                    {
                        return Some(c);
                    }
                }
            }
        }
        None
    }

    /// Whether the foundation's cells hold no object and keep a gap of
    /// [`GAP`] cells to other buildings, so that units can leave the
    /// factories and reach the docks.
    pub(crate) fn free(&self, w: &World, t: TypeRef, c: CellXY) -> bool {
        self.free_but(w, t, c, None)
    }

    /// [`Script::free`], not counting object `but`.
    pub(crate) fn free_but(&self, w: &World, t: TypeRef, c: CellXY, but: Option<ObjectId>) -> bool {
        let cells = placement::foundation_cells(w, t.index, c);
        // Only what the house knows: its own objects and those on cells it
        // has explored (the AI sees only its own house's shroud; fog is off
        // in a skirmish, S-0409 claim 19).
        w.objects
            .values()
            .filter(|o| {
                o.on_map
                    && Some(o.id) != but
                    && (o.owner == Some(self.house)
                        || shroud::explored(w, self.house, o.pos.cell()))
            })
            .all(|o| {
                // A building's position is its top-left cell's centre.
                let tl = o.pos.cell();
                if o.kind == Kind::Building {
                    let (fw, fh) = o.foundation;
                    cells.iter().all(|k| {
                        k.x < tl.x - GAP
                            || k.x > tl.x + fw - 1 + GAP
                            || k.y < tl.y - GAP
                            || k.y > tl.y + fh - 1 + GAP
                    })
                } else {
                    !cells.contains(&tl)
                }
            })
    }
}
