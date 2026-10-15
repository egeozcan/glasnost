//! What a click or key command means (S-0214, and S-0213 table T2): the
//! input rules that turn a house's selection and a target into orders. A
//! person's clicks and our AI houses use the same functions (S-0214 Terms,
//! *acting house*), and the orders they return go through
//! [`crate::orders::issue`] like any other.
//!
//! Selection is input-side state, not game state (S-0213 Terms): the caller
//! keeps it and passes it in, in selection order. Call these functions
//! between ticks (they read the regions that every tick leaves current).
//!
//! The tables are implemented row by row. Rows whose conditions need state
//! our engine does not have yet evaluate as that state makes them: nothing is
//! berserk, melee-linked, being erased, phasing in, disabled, falling,
//! disguised, bomb-carrying or a loaner; there is no shroud, fog of war or
//! route mode; no docking answer is given (no contact exchange exists yet,
//! so the answer is neither); weapon ranges, damage and warhead flags are
//! weapon choice, ranges, healing weapons, veterancy abilities and the
//! firing rules' legality answer come from the combat module (M4), with its
//! own seams (garrison occupants, upgrades); the weapon cursor flags of rows
//! I17 and I18 are not imported; vehicles have no deploy state; the
//! factory kinds of rows B9 to B12 are not applied yet; a carrier carries nothing. The rows that read these states are in place
//! behind small predicates, so that the states plug in when their topics
//! arrive. The nearby-cell search of claim 11 always uses the actor's reach
//! region and own zone (S-0212 table T5's rows for a free mover's no-move
//! and for zone *fly* on an object are not in `path::nearby` yet).

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_num::F64;
use glasnost_num::geom::root_floor;
use glasnost_rules::sim::MovementZone;

use crate::combat::check::{self, Answer};
use crate::combat::data::{Ability, WeaponId};
use crate::combat::{range, veterancy, weapon};
use crate::infantry::Seq;
use crate::occupancy::{self, Outcome};
use crate::orders::Order;
use crate::path::{self, NearbyUse};
use crate::regions::{self, Reach};
use crate::rules::{Behaviour, BuildingRules, Kind, UnitRules};
use crate::world::{HouseId, Layer, ObjectId, Pos, Target, World};

/// What was clicked.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClickOn {
    /// An object.
    Object(ObjectId),
    /// A cell (no object under the cursor).
    Cell(CellXY),
}

/// The modifier roles held while clicking (S-0214 Terms).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Force-move.
    pub force_move: bool,
    /// Force-fire.
    pub force_fire: bool,
    /// Force-select.
    pub force_select: bool,
    /// Attack-move mode is on (S-0214 claim 5).
    pub attack_move_mode: bool,
}

impl Modifiers {
    /// The attack-move gesture: force-fire and force-select together, or
    /// attack-move mode (S-0214 claim 5).
    pub fn attack_move(self) -> bool {
        (self.force_fire && self.force_select) || self.attack_move_mode
    }

    /// The counted modifiers: force-fire and force-select held together
    /// count as neither (S-0214 Terms).
    pub fn counted(self) -> Self {
        if self.force_fire && self.force_select {
            Self {
                force_fire: false,
                force_select: false,
                ..self
            }
        } else {
            self
        }
    }
}

/// An action (S-0214 table A). Sidebar-mode actions arrive with the economy
/// topic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Action {
    /// Nothing.
    Nothing,
    /// Select.
    Select,
    /// Toggle select.
    ToggleSelect,
    /// Move.
    Move,
    /// A move shown as not possible.
    NoMove,
    /// Attack.
    Attack,
    /// Enter.
    Enter,
    /// Enter, refused.
    NoEnter,
    /// Deploy.
    Deploy,
    /// Deploy, refused.
    NoDeploy,
    /// Area guard (escort an object or guard a cell).
    AreaGuard,
    /// Patrol (route mode).
    Patrol,
    /// Capture, garrison, infiltrate or hijack.
    Capture,
    /// Damaging entry.
    DamagingEntry,
    /// Repair building.
    RepairBuilding,
    /// Repair, refused.
    NoRepair,
    /// Recycle.
    Recycle,
    /// Heal.
    Heal,
    /// Repair vehicle.
    RepairVehicle,
    /// Sabotage.
    Sabotage,
    /// Bomb.
    Bomb,
    /// Bomb, refused.
    NoBomb,
    /// Detonate.
    Detonate,
    /// Disarm bomb.
    DisarmBomb,
    /// Tote.
    Tote,
    /// Harvest.
    Harvest,
    /// Enter tunnel.
    EnterTunnel,
    /// Enter tunnel, refused.
    NoEnterTunnel,
    /// Area attack.
    AreaAttack,
    /// Demolish.
    Demolish,
    /// Airstrike.
    Airstrike,
    /// Idle.
    Idle,
}

/// The result of a click.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClickResult {
    /// Orders, in selection order, for the acting house to issue.
    pub orders: Vec<Order>,
    /// The new selection, when the click changed it.
    pub selection: Option<Vec<ObjectId>>,
    /// The object that speaks the acknowledgement: the first object of the
    /// selection, when it got an order from the click (S-0214 claim 14).
    /// Sound only, never game state; AI houses give none.
    pub acknowledge: Option<ObjectId>,
    /// Whether a move-target marker is drawn for the click (S-0214 claim
    /// 30): the first object of the selection, a mobile actor, got an order
    /// from the action *move* on a cell. Drawing only; placed by
    /// [`move_marker_at`].
    pub marker: bool,
}

/// A key command (S-0213 table T2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyCommand {
    /// Stop.
    Stop,
    /// Scatter.
    Scatter,
    /// Guard.
    Guard,
    /// Deploy.
    Deploy,
}

// ---------------------------------------------------------------------------
// Properties (S-0214 Terms)

fn unit(w: &World, id: ObjectId) -> Option<&UnitRules> {
    let o = w.obj(id)?;
    w.rules().unit(o.kind, o.type_index)
}

fn building(w: &World, id: ObjectId) -> Option<&BuildingRules> {
    let o = w.obj(id)?;
    if o.kind != Kind::Building {
        return None;
    }
    w.rules().buildings.get(o.type_index as usize)
}

fn kind(w: &World, id: ObjectId) -> Option<Kind> {
    w.obj(id).map(|o| o.kind)
}

/// An own object of the acting house (S-0214 Terms).
fn own(w: &World, house: HouseId, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.owner == Some(house))
}

/// A unit or building (S-0213 Terms).
fn unit_or_building(w: &World, id: ObjectId) -> bool {
    kind(w, id).is_some_and(|k| k != Kind::Terrain)
}

/// Allied with the actor (S-0214 Terms, *Allied, enemy*): judged from the
/// actor's side, with the one-way alliance: the actor's house allies the
/// target's house (a house allies itself). A target without a house is an
/// enemy.
fn allied(w: &World, actor: ObjectId, target: ObjectId) -> bool {
    let a = w.obj(actor).and_then(|o| o.owner);
    let t = w.obj(target).and_then(|o| o.owner);
    w.allied(a, t)
}

/// Whether the target belongs to another house than the actor.
fn other_house(w: &World, actor: ObjectId, target: ObjectId) -> bool {
    let a = w.obj(actor).and_then(|o| o.owner);
    let t = w.obj(target).and_then(|o| o.owner);
    t.is_some() && t != a
}

/// Whether the object can be selected (S-0214 Terms, *can be selected*):
/// a unit or building in play on the map whose type's selection flag
/// permits it, but not a building whose type is invisible in the game.
/// Zero health does not exclude a building still in play (awaiting
/// removal, S-0214 claim 16); being erased does (no erasing weapon yet).
/// Whether a unit below the surface or a cloaked one can be selected is an
/// open question of S-0214; neither state exists yet, so such units count
/// as selectable.
pub fn selectable(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 Terms (can be selected: units and buildings; a building whose type is invisible in the game is believed not to be, low)
    // spec: S-0214 Terms and Import mapping (the type's selection flag must permit selection; absent: yes)
    let flag = match kind(w, id) {
        Some(Kind::Building) => building(w, id).is_none_or(|b| b.raw.selectable.unwrap_or(true)),
        _ => unit(w, id).is_none_or(|u| u.raw.selectable.unwrap_or(true)),
    };
    // spec: S-0214 claim 16 (zero health does not exclude a building still in play awaiting removal; being erased does)
    let alive = w.obj(id).is_some_and(|o| {
        o.on_map && (o.health > 0 || (o.kind == Kind::Building && o.combat.killed))
    });
    unit_or_building(w, id) && !invisible(w, id) && flag && alive && !being_erased(w, id)
}

/// A vehicle-like building: it undeploys into a vehicle and has a 1 × 1
/// foundation (S-0214 Terms).
fn vehicle_like(w: &World, id: ObjectId) -> bool {
    building(w, id).is_some_and(|b| b.raw.undeploys_into.is_some() && b.foundation == (1, 1))
}

/// Whether an ordinary band box can add the object to house `house`'s
/// selection (S-0214 claim 19): an own selectable infantry unit, vehicle,
/// aircraft or vehicle-like building in play, not already selected, not
/// tethered to a building that stands on its cell. Deactivated objects,
/// slaves controlled by a master and units held in a tank bunker are
/// excluded too; none of those states exists yet, so no object is one.
pub fn box_eligible(w: &World, house: HouseId, selection: &[ObjectId], id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    // spec: S-0214 claim 19 (own infantry, vehicles and aircraft; ordinary buildings excluded, vehicle-like buildings taken)
    let kind_ok = match o.kind {
        Kind::Infantry | Kind::Vehicle | Kind::Aircraft => true,
        Kind::Building => vehicle_like(w, id),
        Kind::Terrain => false,
    };
    // spec: S-0214 claim 19 (a unit tethered to a building is excluded while its cell holds that building)
    let tether = crate::econ::tethered(w, id)
        && o.contact.and_then(|k| w.obj(k)).is_some_and(|b| {
            b.kind == Kind::Building
                && crate::objects::foundation_cells(b.pos.cell(), b.foundation)
                    .contains(&o.pos.cell())
        });
    // spec: S-0214 claim 19 (in play, selectable, not being erased, not already selected; enemy and allied objects never added)
    kind_ok && own(w, house, id) && selectable(w, id) && !selection.contains(&id) && !tether
}

/// Rally-capable (S-0214 Terms): a factory of infantry or vehicles, a
/// cloning building or a repair depot.
fn rally_capable(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 Terms (rally-capable) and claim 17 (a non-producing category alone does not grant it, nor cancels cloning or repair)
    building(w, id).is_some_and(|b| {
        matches!(
            b.factory,
            Some(crate::rules::FactoryKind::Infantry | crate::rules::FactoryKind::Vehicle)
        ) || b.raw.cloning.unwrap_or(false)
            || b.raw.unit_repair.unwrap_or(false)
    })
}

/// A factory of infantry, vehicles or aircraft (S-0213 claim 20 (c)).
fn makes_units(w: &World, id: ObjectId) -> bool {
    use crate::rules::FactoryKind as F;
    building(w, id)
        .is_some_and(|b| matches!(b.factory, Some(F::Infantry | F::Vehicle | F::Aircraft)))
}

/// The guard-spot cell of a building's click around `centre` (S-0213 claim
/// 20; S-0212 claim 25, table T5's rally-point row), chosen at the click.
fn rally_cell(w: &World, id: ObjectId, centre: CellXY) -> Option<CellXY> {
    use glasnost_rules::sim::LocomotionClass as L;
    let b = building(w, id)?;
    // spec: S-0213 claim 20 (the zone by the building's type: naval, whatever it builds; not naval and a factory of aircraft; any other) and S-0212 table T5 (rally point: the matching locomotion class)
    let (class, zone) = if b.raw.naval.unwrap_or(false) {
        (L::Amphibious, MovementZone::AmphibiousCrusher)
    } else if b.factory == Some(crate::rules::FactoryKind::Aircraft) {
        (L::Winged, MovementZone::Fly)
    } else {
        (L::Foot, MovementZone::Normal)
    };
    // spec: S-0212 table T5 (rally point: the reach region of the building's own cell for that zone)
    let reach = regions::reach_in(w, w.obj(id)?.pos.cell(), zone);
    // spec: S-0212 table T5 (rally point: in deck mode when the centre is a bridge cell)
    let deck = w.cells.idx(centre).is_some_and(|i| w.cells.deck[i]);
    // spec: S-0212 table T5 (rally point: no height condition, no further test, no reference cell)
    path::nearby(
        w,
        centre,
        NearbyUse::Settings(path::NearbySettings {
            class: Some(class),
            zone,
            reach: Some(reach),
            deck,
            height: false,
            reference: None,
            no_bridges: false,
            no_overlay: false,
        }),
    )
}

/// Factory-marked (S-0214 Terms): any imported factory category other than
/// no factory, the recognised non-producing ones included (claim 17).
fn factory_marked(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 Terms (factory-marked) and claim 17
    building(w, id).is_some_and(|b| b.factory_category.is_some())
}

/// Marked primary (S-0213 claim 13).
fn primary(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.econ.building.primary)
}

/// Movable (S-0214 Terms).
pub fn movable(w: &World, house: HouseId, id: ObjectId) -> bool {
    if !own(w, house, id) {
        return false;
    }
    // spec: S-0214 Terms (movable) and S-0506 claim 10 (a spawner whose launching count lies strictly between 0 and N is not movable)
    if crate::spawn::partly_launched(w, id) {
        return false;
    }
    match kind(w, id) {
        // spec: S-0214 Terms (movable mobile object: not disabled, not a spawned aircraft type, not a slave unit, …)
        // spec: S-0214 Terms and S-0614 claim 8 (not held)
        Some(Kind::Infantry | Kind::Vehicle) => !crate::parasite::held(w, id),
        Some(Kind::Aircraft) => {
            !unit(w, id).is_some_and(|u| u.raw.spawned.unwrap_or(false))
                && !crate::parasite::held(w, id)
        }
        // spec: S-0214 Terms (a building: rally-capable or undeploys into a vehicle; a construction yard also needs the lobby option, which our setup lacks)
        Some(Kind::Building) => {
            let b = building(w, id);
            let yard = b.is_some_and(|b| b.raw.construction_yard.unwrap_or(false));
            rally_capable(w, id) || (b.is_some_and(|b| b.raw.undeploys_into.is_some()) && !yard)
        }
        _ => false,
    }
}

/// Armed (S-0214 Terms): a weapon in the weapon slot it currently uses
/// (S-0308's choice with no target). Garrison occupants that may fire from
/// inside are a seam (no garrisons yet).
pub fn armed(w: &World, id: ObjectId) -> bool {
    match kind(w, id) {
        Some(Kind::Terrain) | None => false,
        // spec: S-0214 Terms (armed: a weapon in the slot it currently uses)
        _ => weapon::armed(w, id),
    }
}

/// The weapon an object would use by default (S-0214 Terms, healing actor).
fn default_weapon(w: &World, id: ObjectId) -> Option<WeaponId> {
    weapon::would_use(w, id, None).1
}

/// The weapon in an object's first slot.
fn first_weapon(w: &World, id: ObjectId) -> Option<WeaponId> {
    weapon::slot_weapon(w, id, 0)
}

fn heals(w: &World, x: Option<WeaponId>) -> bool {
    // spec: S-0214 Terms (a weapon that heals does negative damage)
    x.is_some_and(|x| w.rules().combat.weapon(x).damage < 0)
}

/// The target is in range of the weapon the actor would use against it
/// (S-0214 claim 6 (d), row GC4; the range test of S-0307).
fn in_range_of(w: &World, actor: ObjectId, t: Target) -> bool {
    let (_, x) = weapon::would_use(w, actor, Some(t));
    range::in_range(w, actor, x, Some(t), None)
}

/// Whether the attack cursor is the in-range one (S-0214 table C): the
/// selection is one unit or building, whose weapon is in range of the
/// target.
pub fn attack_cursor_in_range(w: &World, selection: &[ObjectId], on: ClickOn) -> bool {
    let t = match on {
        ClickOn::Object(t) => Target::Object(t),
        ClickOn::Cell(c) => Target::Cell(c),
    };
    // spec: S-0214 table C (attack: 53–57 only with one selected unit or building whose weapon is in range of the target)
    matches!(selection, [id] if unit_or_building(w, *id) && in_range_of(w, *id, t))
}

/// Whether the first selected object's type may move into shroud (S-0214
/// Import mapping, `MoveToShroud`), for table C's shrouded *no-move* row.
pub fn first_may_move_into_shroud(w: &World, selection: &[ObjectId]) -> bool {
    // spec: S-0214 table C (no-move on a shrouded target: the move frames if the first selected object's type permits shroud moves)
    selection
        .first()
        .and_then(|&id| unit(w, id))
        .is_some_and(|u| u.move_to_shroud)
}

/// Able to fire (S-0214 Terms): own, armed, not disabled, not falling
/// (nothing falls before the aircraft topic).
pub fn able_to_fire(w: &World, house: HouseId, id: ObjectId) -> bool {
    own(w, house, id) && armed(w, id) && !disabled(w, id)
}

/// A healing actor (S-0214 Terms): an armed object whose default weapon
/// does negative damage.
fn healing_actor(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 Terms (healing actor: armed, and the weapon it would use by default does negative damage)
    armed(w, id) && heals(w, default_weapon(w, id))
}

/// An infiltrating infantry type facing a building it can enter (S-0214
/// claim 6 (e)): a spy and a spyable building, another infiltrator and a
/// capturable building, or a charge planter and a building that accepts
/// charges.
fn infiltrator_entry(w: &World, actor: ObjectId, t: ObjectId) -> bool {
    if kind(w, actor) != Some(Kind::Infantry)
        || !uflag(w, actor, |u| u.raw.infiltrate.unwrap_or(false))
    {
        return false;
    }
    let Some(b) = building(w, t) else {
        return false;
    };
    // spec: S-0214 Import mapping (Agent, Capturable, Spyable, C4 absent: no; CanC4 absent: yes)
    let spy = uflag(w, actor, |u| u.raw.agent.unwrap_or(false));
    let charges = uflag(w, actor, |u| u.raw.c4.unwrap_or(false));
    (spy && b.raw.spyable.unwrap_or(false))
        || (!spy && b.raw.capturable.unwrap_or(false))
        || (charges && b.raw.can_c4.unwrap_or(true))
}

// The states below have no place in our engine before combat (M4) and the
// special-abilities topics: these seams keep the rows that read them in
// place, and evaluate as the state is never present.

/// Berserk (S-0213 Terms): no berserk-causing warhead before M4.
fn berserk(_w: &World, _id: ObjectId) -> bool {
    false
}

/// Melee-linked while the type lacks the keep-acting flag (S-0214 Terms,
/// rows G1 and GC1): the link needs warhead hits (M4).
fn melee_held(_w: &World, _id: ObjectId) -> bool {
    false
}

/// Being erased (S-0214 Terms): no erasing weapon before M4.
fn being_erased(_w: &World, _id: ObjectId) -> bool {
    false
}

/// Mind-controlled (S-0214 Terms; the combat topic owns mind control).
/// Seam: no object is under mind control in our engine yet.
fn mind_controlled(_w: &World, _id: ObjectId) -> bool {
    false
}

/// The occupation test of S-0508 claim 1, row I8's "can garrison": the
/// infantry unit `actor` may garrison building `t`.
fn may_garrison(w: &World, actor: ObjectId, t: ObjectId) -> bool {
    let (Some(a), Some(b), Some(tb)) = (w.obj(actor), w.obj(t), building(w, t)) else {
        return false;
    };
    // spec: S-0508 claim 1 (the building's type is garrisonable)
    if !tb.raw.can_be_occupied.unwrap_or(false) {
        return false;
    }
    // spec: S-0508 claim 1 (not being built up and not being sold, S-0408: the construction or selling behaviour, current or queued, or a sale under way)
    if matches!(
        b.beh.seen(),
        Some(Behaviour::Construction | Behaviour::Selling)
    ) || b.econ.sale.is_some()
    {
        return false;
    }
    // spec: S-0508 claim 1 (the building's centre lies in the usable area)
    if !w.cells.usable(w.reference_point(t).cell()) {
        return false;
    }
    // spec: S-0508 claim 1 (not being erased)
    if being_erased(w, t) {
        return false;
    }
    // spec: S-0508 claim 1 (the unit's type is an occupier type)
    if !uflag(w, actor, |u| u.raw.occupier.unwrap_or(false)) {
        return false;
    }
    // spec: S-0508 claim 1 (the building belongs to the unit's own house or to a passive house; alliance plays no part)
    let owner_ok = match b.owner {
        Some(h) => b.owner == a.owner || crate::econ::setup::passive(w, h),
        None => false,
    };
    if !owner_ok {
        return false;
    }
    // spec: S-0508 claim 1 (the occupant count is not equal to the occupant capacity: equality only, quirk)
    if crate::econ::power::occupants(w, t) == tb.building.max_occupants {
        return false;
    }
    // spec: S-0508 claim 1 (not both positive health and badly damaged: h ≤ the red ratio, S-0108 Terms and F3)
    if b.health > 0
        && tb.strength != 0
        && health_cmp(w, t, w.rules().general.condition_red) != std::cmp::Ordering::Greater
    {
        return false;
    }
    // spec: S-0508 claim 1 (the unit is not mind-controlled)
    !mind_controlled(w, actor)
}

/// Phasing in after a teleporting jump (S-0214 Terms, S-0207 claim 20 f).
fn phasing_in(w: &World, id: ObjectId) -> bool {
    crate::teleport::phasing_in(w, id)
}

/// Carries an active planted bomb (S-0213 Terms): no bombs before M4.
fn planted_bomb(_w: &World, _id: ObjectId) -> bool {
    false
}

/// Disabled (S-0214 Terms): paralysed, or a vehicle playing its death
/// sequence; neither exists before M4.
fn disabled(_w: &World, _id: ObjectId) -> bool {
    false
}

/// A carrier aircraft (S-0214 Import mapping, `Carryall`).
fn carrier(w: &World, id: ObjectId) -> bool {
    kind(w, id) == Some(Kind::Aircraft) && uflag(w, id, |u| u.raw.carryall.unwrap_or(false))
}

/// Whether a cell holds a vehicle factory (S-0214 row AC2): a building
/// with the vehicle-factory flag in its ground layer.
fn vehicle_factory_cell(w: &World, c: CellXY) -> bool {
    w.cells
        .list(c, Layer::Ground)
        .iter()
        .any(|&b| building(w, b).is_some_and(|r| r.raw.weapons_factory.unwrap_or(false)))
}

/// Whether an object stands on a cell that holds a building.
fn on_building_cell(w: &World, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| {
        w.cells
            .list(o.pos.cell(), Layer::Ground)
            .iter()
            .any(|&b| b != id && kind(w, b) == Some(Kind::Building))
    })
}

/// A self-teleporting type (S-0214 Terms, Import mapping `Teleporter`).
fn teleporter(w: &World, id: ObjectId) -> bool {
    uflag(w, id, |u| u.raw.teleporter.unwrap_or(false))
}

/// A free mover (S-0214 Terms): an aircraft, an underground type (zone
/// *subterranean*) or a jumpjet infantry type. The jumpjet counts for a
/// click on a cell only while no ion storm is active, and our engine has no
/// ion storm. Nothing of another kind is in the air before the aircraft
/// topic.
fn free_mover(w: &World, id: ObjectId) -> bool {
    match kind(w, id) {
        Some(Kind::Aircraft) => true,
        Some(Kind::Infantry) if uflag(w, id, |u| u.jumpjet) => true,
        Some(Kind::Infantry | Kind::Vehicle) => {
            uflag(w, id, |u| u.zone == MovementZone::Subterranean)
        }
        _ => false,
    }
}

fn uflag(w: &World, id: ObjectId, f: impl Fn(&UnitRules) -> bool) -> bool {
    unit(w, id).is_some_and(f)
}

fn engineer(w: &World, id: ObjectId) -> bool {
    kind(w, id) == Some(Kind::Infantry) && uflag(w, id, |u| u.engineer)
}

/// An in-place deployer (S-0214 Terms).
fn in_place_deployer(w: &World, id: ObjectId) -> bool {
    match kind(w, id) {
        // spec: S-0214 Import mapping (infantry Deployer, vehicle IsSimpleDeployer)
        Some(Kind::Infantry) => uflag(w, id, |u| u.deployer),
        Some(Kind::Vehicle) => uflag(w, id, |u| u.simple_deployer),
        _ => false,
    }
}

fn seq(w: &World, id: ObjectId) -> Option<Seq> {
    w.obj(id)
        .and_then(|o| o.infantry.as_ref())
        .and_then(|s| s.seq)
}

/// An in-place deployer in its deployed state (vehicles have no deploy
/// state yet).
fn deployed_ipd(w: &World, id: ObjectId) -> bool {
    in_place_deployer(w, id) && seq(w, id).is_some_and(|q| q == Seq::Deployed)
}

fn has_undeploy_delay(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 Import mapping (UndeployDelay: present when 0 or more)
    unit(w, id).is_some_and(|u| u.raw.undeploy_delay.is_some_and(|d| d >= 0))
}

fn passengers(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 Import mapping (Passengers: a transport when more than 0)
    unit(w, id).is_some_and(|u| u.passengers > 0)
}

fn immune(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 Import mapping (Immune absent: no)
    unit(w, id).is_some_and(|u| u.raw.immune.unwrap_or(false))
}

fn legal_target(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 Import mapping (LegalTarget absent: yes)
    unit(w, id).is_none_or(|u| u.raw.legal_target.unwrap_or(true))
}

fn invisible(w: &World, id: ObjectId) -> bool {
    building(w, id).is_some_and(|b| b.raw.invisible_in_game.unwrap_or(false))
}

/// The health ratio compared with a value (S-0214 F2): −1, 0 or 1.
fn health_cmp(w: &World, id: ObjectId, c: F64) -> std::cmp::Ordering {
    let Some(o) = w.obj(id) else {
        return std::cmp::Ordering::Equal;
    };
    let strength = match o.kind {
        Kind::Building => building(w, id).map_or(0, |b| b.strength),
        _ => unit(w, id).map_or(0, |u| u.strength),
    };
    if strength == 0 {
        return std::cmp::Ordering::Equal;
    }
    let mode = w.rules().mode;
    // spec: S-0214 F2 (h = p / P in double precision)
    let h = F64::from_i64(i64::from(o.health), mode)
        .div(F64::from_i64(i64::from(strength), mode), mode);
    if h.lt(c) {
        std::cmp::Ordering::Less
    } else if h.gt(c) {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

fn below_full(w: &World, id: ObjectId) -> bool {
    let mode = w.rules().mode;
    // spec: S-0214 Constants (full health: a health ratio of exactly 1)
    health_cmp(w, id, F64::from_i64(1, mode)) == std::cmp::Ordering::Less
}

/// Counts as a vehicle (S-0214 Terms).
pub fn counts_as_vehicle(w: &World, id: ObjectId) -> bool {
    match kind(w, id) {
        Some(Kind::Vehicle) => !uflag(w, id, |u| u.raw.non_vehicle.unwrap_or(false)),
        // Aircraft stand on the ground in our engine so far.
        Some(Kind::Aircraft) => true,
        Some(Kind::Building) => vehicle_like(w, id),
        _ => false,
    }
}

/// The height above the surface (S-0214 F3).
fn height_above_surface(w: &World, id: ObjectId) -> i32 {
    w.obj(id)
        .map_or(0, |o| o.pos.z - w.surface_at(o.pos.x, o.pos.y))
}

fn attackable_height(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 F3 (z_s > −20)
    height_above_surface(w, id) > -20
}

fn moving(w: &World, id: ObjectId) -> bool {
    occupancy::in_progress(w, id)
}

/// The scenery check and fallback move (S-0214 Terms); nothing is
/// disguised in our engine yet, so the scenery check gives *no-move*.
fn fallback_move(w: &World, house: HouseId, actor: ObjectId) -> Action {
    if movable(w, house, actor) {
        Action::NoMove
    } else {
        Action::Nothing
    }
}

/// Engineer-friendly building (S-0214 table I).
fn engineer_friendly(w: &World, actor: ObjectId, t: ObjectId) -> bool {
    let Some(b) = building(w, t) else {
        return false;
    };
    // spec: S-0214 Import mapping (Repairable absent: yes)
    let repairable = b.raw.repairable.unwrap_or(true) && !vehicle_like(w, t);
    let passive_neutral = w
        .obj(t)
        .and_then(|o| o.owner)
        .and_then(|h| w.houses.get(usize::from(h.0)))
        .and_then(|h| w.rules().countries.get(h.country as usize))
        .is_some_and(|c| c.passive);
    // spec: S-0214 table I (repairable, not vehicle-like, allied; or of a passive neutral house and garrisonable)
    (repairable && allied(w, actor, t))
        || (passive_neutral && b.raw.can_be_occupied.unwrap_or(false))
}

// ---------------------------------------------------------------------------
// Table G and the attack rule (claims 4 and 6)

/// The attack rule (S-0214 claim 6).
fn attack_rule(w: &World, house: HouseId, actor: ObjectId, t: ObjectId, mods: Modifiers) -> bool {
    let enemy = !allied(w, actor, t);
    let u = unit(w, actor);
    // spec: S-0214 claim 6 (may attack as a friend: the attack-friendlies or attack-cursor-on-friendlies flag)
    let friend_ok = u.is_some_and(|u| {
        u.raw.attack_friendlies.unwrap_or(false)
            || u.raw.attack_cursor_on_friendlies.unwrap_or(false)
    }) || sonic_on_parasite_carrier(w, actor, t);
    // spec: S-0214 claim 6 (a) (force-fire; or enemy or friend-attack, the disguise rule, and a legal target)
    let a = mods.force_fire || ((enemy || friend_ok) && legal_target(w, t));
    // spec: S-0214 claim 6 (b) (attackable at all: F3)
    let b = attackable_height(w, t);
    // spec: S-0214 claim 6 (c) (armed, or an infiltrating infantry type)
    let c = armed(w, actor)
        || (kind(w, actor) == Some(Kind::Infantry)
            && uflag(w, actor, |u| u.raw.infiltrate.unwrap_or(false)));
    // spec: S-0214 claim 6 (d) (movable, or the target already in range of the weapon it would use)
    let d = movable(w, house, actor) || in_range_of(w, actor, Target::Object(t));
    // spec: S-0214 claim 6 (e) (unless force-fire or an infiltrator's entry, the firing rules do not rule the target illegal)
    let e = mods.force_fire || infiltrator_entry(w, actor, t) || {
        let (slot, x) = weapon::would_use(w, actor, Some(Target::Object(t)));
        // Without a weapon there is no firing rule to rule the target illegal.
        x.is_none()
            || check::check(w, actor, Some(Target::Object(t)), slot, false) != Answer::Illegal
    };
    a && b && c && d && e
}

/// Whether the weapon `actor` would use has a sonic warhead and `t` is a
/// mobile unit with a parasite on it (S-0214 claim 6, *may attack as a friend*).
fn sonic_on_parasite_carrier(w: &World, actor: ObjectId, t: ObjectId) -> bool {
    // spec: S-0214 claim 6 (that warhead is sonic and the target is a mobile unit with a parasite on it)
    crate::parasite::carries_parasite(w, t) && {
        let (_, x) = weapon::would_use(w, actor, Some(Target::Object(t)));
        x.and_then(|wp| w.rules().combat.warhead_of(wp))
            .is_some_and(|h| h.sonic)
    }
}

/// The generic result g on an object (S-0214 table G).
fn generic(
    w: &World,
    house: HouseId,
    sel: &[ObjectId],
    actor: ObjectId,
    t: ObjectId,
    mods: Modifiers,
) -> Action {
    let is_own = own(w, house, actor);
    let only = sel.len() == 1 && sel[0] == actor;
    // No loaners exist before the topics that lend units: "not an own loaner" always holds.
    let selectable_new = selectable(w, t) && !sel.contains(&t);
    // spec: S-0214 table G row G1 (berserk, or melee-linked without the keep-acting flag: nothing)
    if berserk(w, actor) || melee_held(w, actor) {
        return Action::Nothing;
    }
    // spec: S-0214 table G row G2 (being erased: select unless armed, own and the target another house's)
    if being_erased(w, actor) {
        let armed_own_other = armed(w, actor) && is_own && other_house(w, actor, t);
        return if selectable_new && !armed_own_other {
            Action::Select
        } else {
            Action::Nothing
        };
    }
    // spec: S-0214 table G row G3 (the only selected own object, clicked on itself, carrying a bomb it may set off)
    if t == actor && only && is_own && planted_bomb(w, actor) {
        return Action::Detonate;
    }
    // spec: S-0214 table G row G4 (force-select, own actor, not a loaner)
    if mods.force_select && is_own {
        return Action::ToggleSelect;
    }
    // spec: S-0214 table G row G5 (the actor itself, the only one selected, own)
    if t == actor && only && is_own {
        return Action::Deploy;
    }
    // spec: S-0214 table G row G6 (force-move and force-fire on an allied object: escort)
    if mods.force_move
        && mods.force_fire
        && allied(w, actor, t)
        && movable(w, house, actor)
        && kind(w, actor) != Some(Kind::Aircraft)
    {
        return Action::AreaGuard;
    }
    // spec: S-0214 table G row G7 (force-move, own and movable)
    if mods.force_move && movable(w, house, actor) {
        return Action::Move;
    }
    // spec: S-0214 table G row G8 (the attack rule)
    if is_own && attack_rule(w, house, actor, t, mods) {
        return Action::Attack;
    }
    // spec: S-0214 table G row G9 (armed and own, the target another house's)
    if is_own && armed(w, actor) && other_house(w, actor, t) {
        return Action::Nothing;
    }
    // spec: S-0214 table G row G10 (selectable, not selected yet, not being erased)
    if selectable_new && !being_erased(w, t) {
        return Action::Select;
    }
    // spec: S-0214 table G row G11
    Action::Nothing
}

// ---------------------------------------------------------------------------
// Kind tables on an object (claims 7 and 8)

/// Infantry on an object (S-0214 table I).
fn infantry_on(
    w: &World,
    house: HouseId,
    actor: ObjectId,
    t: ObjectId,
    g: Action,
    mods: Modifiers,
) -> Action {
    let r = w.rules();
    let tb = building(w, t);
    let is_bld = tb.is_some();
    // spec: S-0214 table I value e (row e4: a hospital, no force-fire, the actor at full health; armories and docking answers need contact exchanges)
    let e = (is_bld
        && allied(w, actor, t)
        && tb.is_some_and(|b| b.raw.hospital.unwrap_or(false))
        && !mods.force_fire
        && !below_full(w, actor))
    .then_some(Action::NoEnter);
    // spec: S-0214 table I (g₁ = e where defined, else g)
    let g1 = e.unwrap_or(g);
    // spec: S-0214 table I value v (deploy: area attack, deploys in place, else nothing)
    let v = if g1 != Action::Deploy {
        g1
    } else if uflag(w, actor, |u| u.deployer) {
        Action::Deploy
    } else {
        Action::Nothing
    };
    // spec: S-0214 table I value x (rows x1 and x2; x3 and x4 need a docking answer)
    let x = (unit_or_building(w, t)
        && allied(w, actor, t)
        && passengers(w, t)
        && movable(w, house, actor)
        && !matches!(v, Action::Attack | Action::AreaGuard | Action::NoEnter))
    .then(|| {
        // spec: S-0214 table I rows x1 and x2 (a balloon-hovering type; a moving mobile target)
        if uflag(w, actor, |u| u.raw.balloon_hover.unwrap_or(false))
            || (kind(w, t).is_some_and(Kind::is_mobile) && moving(w, t))
        {
            Some(Action::NoEnter)
        } else {
            None
        }
    })
    .flatten();
    // spec: S-0214 table I (g₂ = x where defined, else v)
    let g2 = x.unwrap_or(v);
    let eng = engineer(w, actor);
    let target_building_repairable =
        tb.is_some_and(|b| b.raw.repairable.unwrap_or(true)) && !vehicle_like(w, t);
    // spec: S-0214 table I row I1
    if g == Action::ToggleSelect {
        return Action::ToggleSelect;
    }
    // spec: S-0214 table I row I2 (an engineer over a target carrying an active planted bomb: disarm bomb)
    if eng && planted_bomb(w, t) {
        return Action::DisarmBomb;
    }
    // spec: S-0214 table I row I3 (a bridge repair hut: no bridge is destroyed in our engine yet)
    if eng
        && target_building_repairable
        && tb.is_some_and(|b| b.raw.bridge_repair_hut.unwrap_or(false))
    {
        return Action::NoRepair;
    }
    let friendly = eng && engineer_friendly(w, actor, t);
    // spec: S-0214 table I row I4 (an engineer below full health; an engineer-friendly hospital)
    if friendly && below_full(w, actor) && tb.is_some_and(|b| b.raw.hospital.unwrap_or(false)) {
        return Action::Enter;
    }
    // spec: S-0214 table I row I5 (an engineer-friendly building at full health)
    if friendly && !below_full(w, t) {
        return if tb.is_some_and(|b| b.raw.grinding.unwrap_or(false)) {
            Action::Recycle
        } else {
            Action::NoRepair
        };
    }
    // spec: S-0214 table I row I6
    if friendly {
        return Action::RepairBuilding;
    }
    // spec: S-0214 table I row I7 (capture if h ≤ c, else damaging entry)
    if eng && target_building_repairable && tb.is_some_and(|b| b.raw.capturable.unwrap_or(false)) {
        return if health_cmp(w, t, r.general.engineer_capture) != std::cmp::Ordering::Greater {
            Action::Capture
        } else {
            Action::DamagingEntry
        };
    }
    // spec: S-0214 table I row I8 (a building this infantry can garrison under S-0508 claim 1; no force-fire)
    if !mods.force_fire && may_garrison(w, actor, t) {
        return Action::Capture;
    }
    // spec: S-0214 table I row I9
    if eng && g == Action::Attack {
        return Action::NoMove;
    }
    if healing_actor(w, actor) {
        // spec: S-0214 table I row I10 (a healing actor over an enemy: idle)
        if !allied(w, actor, t) {
            return Action::Idle;
        }
        // spec: S-0214 table I row I11 (over itself: area guard)
        if t == actor {
            return Action::AreaGuard;
        }
        // spec: S-0214 table I row I12 (infantry below full health: heal)
        if kind(w, t) == Some(Kind::Infantry) && below_full(w, t) {
            return Action::Heal;
        }
        // spec: S-0214 table I row I13 (not a unit or building, or no passengers: g if move or area guard, else select)
        if !unit_or_building(w, t) || !passengers(w, t) {
            return if matches!(g, Action::Move | Action::AreaGuard) {
                g
            } else {
                Action::Select
            };
        }
    }
    let thief = uflag(w, actor, |u| u.vehicle_thief);
    // spec: S-0214 table I row I14 (a vehicle thief on a train)
    if thief && counts_as_vehicle(w, t) && uflag(w, t, |u| u.train) {
        return Action::Select;
    }
    // spec: S-0214 table I row I15 (hijack a vehicle of another house; quirks.hijack_truce_harvester_check)
    if thief && counts_as_vehicle(w, t) && other_house(w, actor, t) {
        return Action::Capture;
    }
    // spec: S-0214 table I row I16 (a charge planter: the type flag or the explosive-charges ability)
    let planter = uflag(w, actor, |u| u.raw.c4.unwrap_or(false))
        || veterancy::has_ability(w, actor, Ability::C4);
    if planter && g1 == Action::Attack && building(w, t).is_some() && !vehicle_like(w, t) {
        // spec: S-0214 table I row I16 (sabotage if it accepts charges and is not invisible in the game, else attack)
        let accepts = building(w, t).is_some_and(|b| b.raw.can_c4.unwrap_or(true));
        return if accepts && !invisible(w, t) {
            Action::Sabotage
        } else {
            Action::Attack
        };
    }
    // Rows I17 and I18 need the weapon cursor flags, which are not imported.
    // spec: S-0214 table I row I19 (a bomb planter: bomb if the type accepts bombs and carries none)
    if uflag(w, actor, |u| u.raw.ivan.unwrap_or(false)) && g1 == Action::Attack {
        // spec: S-0214 Import mapping (Bombable absent: yes)
        let bombable = unit(w, t).is_none_or(|u| u.raw.bombable.unwrap_or(true));
        return if bombable {
            Action::Bomb
        } else {
            Action::NoBomb
        };
    }
    let dipd = deployed_ipd(w, actor);
    // spec: S-0214 table I row I20
    if dipd && g1 == Action::Move {
        return Action::NoMove;
    }
    // spec: S-0214 table I row I21 (does not fire while deployed, or the target out of range of its first weapon)
    if dipd
        && g1 == Action::Attack
        && (!uflag(w, actor, |u| u.raw.deploy_fire.unwrap_or(false))
            || !range::in_range(
                w,
                actor,
                first_weapon(w, actor),
                Some(Target::Object(t)),
                None,
            ))
    {
        return Action::NoMove;
    }
    // spec: S-0214 table I row I22 (deployed, with an undeploy delay)
    if dipd && g1 == Action::Deploy && has_undeploy_delay(w, actor) {
        return Action::NoDeploy;
    }
    // spec: S-0214 table I row I23 (undeploying, with an undeploy delay)
    if in_place_deployer(w, actor)
        && has_undeploy_delay(w, actor)
        && seq(w, actor) == Some(Seq::Undeploy)
        && g1 == Action::Deploy
    {
        return Action::NoDeploy;
    }
    let infiltrator = uflag(w, actor, |u| u.raw.infiltrate.unwrap_or(false));
    // spec: S-0214 table I row I24 (infiltrate an enemy spyable or capturable building)
    if infiltrator && g2 == Action::Attack && is_bld && !allied(w, actor, t) {
        let spy = uflag(w, actor, |u| u.raw.agent.unwrap_or(false));
        let ok = tb.is_some_and(|b| {
            if spy {
                b.raw.spyable.unwrap_or(false)
            } else {
                b.raw.capturable.unwrap_or(false)
            }
        });
        if ok {
            return Action::Capture;
        }
    }
    // spec: S-0214 table I row I25
    if infiltrator && !armed(w, actor) && g2 == Action::Attack {
        return fallback_move(w, house, actor);
    }
    // spec: S-0214 table I row I26 (a recycling building)
    if g2 == Action::Select
        && tb.is_some_and(|b| b.raw.grinding.unwrap_or(false))
        && !vehicle_like(w, t)
    {
        return if own(w, house, t) {
            Action::Recycle
        } else {
            Action::NoEnter
        };
    }
    // spec: S-0214 table I row I27
    if g2 == Action::Attack && immune(w, t) {
        return Action::NoMove;
    }
    // spec: S-0214 table I row I28
    if g2 == Action::Nothing {
        return fallback_move(w, house, actor);
    }
    // spec: S-0214 table I row I29
    g2
}

/// A vehicle clicked on itself (S-0214 table VD).
fn vehicle_deploy(w: &World, actor: ObjectId) -> Action {
    let on_tunnel = w
        .obj(actor)
        .and_then(|o| w.cells.cell(o.pos.cell()))
        .is_some_and(|c| c.land == land::TUNNEL);
    // spec: S-0214 Terms (deploy readiness: never on a tunnel cell)
    let ready = !on_tunnel;
    let deploy_weapon = uflag(w, actor, |u| u.raw.deploy_fire.unwrap_or(false));
    // spec: S-0214 table VD row VD3 (deploys in place)
    if uflag(w, actor, |u| u.simple_deployer) {
        return if ready {
            Action::Deploy
        } else {
            Action::NoDeploy
        };
    }
    // spec: S-0214 table VD row VD4 (no passengers, no deploy weapon, not in a bunker)
    if !passengers(w, actor) && !deploy_weapon {
        return Action::Nothing;
    }
    // spec: S-0214 table VD row VD5 (no passengers and a deploy weapon)
    if !passengers(w, actor) && deploy_weapon {
        return if ready {
            Action::Deploy
        } else {
            Action::NoDeploy
        };
    }
    // spec: S-0214 table VD row VD6 (a transport that carries no passengers now; transports hold none yet)
    Action::NoDeploy
}

/// A vehicle on an object (S-0214 table V).
fn vehicle_on(w: &World, house: HouseId, actor: ObjectId, t: ObjectId, g: Action) -> Action {
    let fire = able_to_fire(w, house, actor);
    // spec: S-0214 table V row k1 (it can crush: the type flag, or the crusher ability at its current rank)
    let crush = (occupancy::is_crusher(w, actor)
        || veterancy::has_ability(w, actor, Ability::Crusher))
        && occupancy::crushable_by(w, actor, t);
    // spec: S-0214 table V value k
    let k = match g {
        Action::Attack if !fire && crush => Action::Move,
        Action::Attack if !fire => Action::Select,
        Action::Deploy => vehicle_deploy(w, actor),
        _ => g,
    };
    let ally = allied(w, actor, t);
    // spec: S-0214 table V value b (rows b1 to b3 for a healing vehicle with g other than deploy; b4 otherwise k)
    let b = if healing_actor(w, actor) && g != Action::Deploy {
        let tb = building(w, t).is_some();
        if !ally {
            // spec: S-0214 table V row b1 (an enemy: nothing)
            Action::Nothing
        } else if !tb && (!counts_as_vehicle(w, t) || t == actor || !attackable_height(w, t)) {
            // spec: S-0214 table V row b2 (allied, not a building; not a vehicle, itself or not attackable: select)
            Action::Select
        } else if counts_as_vehicle(w, t)
            && t != actor
            && attackable_height(w, t)
            && below_full(w, t)
            && !(kind(w, t) == Some(Kind::Aircraft) && on_building_cell(w, t))
        {
            // spec: S-0214 table V row b3 (an allied vehicle below full health, not an aircraft on a building's cell: repair vehicle)
            Action::RepairVehicle
        } else {
            k
        }
    } else {
        k
    };
    // spec: S-0214 table V value n (rows n1 to n3 need a docking answer accepts, which no contact exchange gives yet: n4)
    let n = b;
    // spec: S-0214 table V value q (row q1: a moving transport refuses)
    let q = if !matches!(
        n,
        Action::Attack | Action::RepairVehicle | Action::AreaGuard | Action::NoEnter
    ) && unit_or_building(w, t)
        && allied(w, actor, t)
        && t != actor
        && passengers(w, t)
        && kind(w, t).is_some_and(Kind::is_mobile)
        && moving(w, t)
    {
        Action::NoEnter
    } else {
        n
    };
    // spec: S-0214 table V row V1
    if g == Action::ToggleSelect {
        return Action::ToggleSelect;
    }
    // spec: S-0214 table V row V4
    if q == Action::Nothing {
        return fallback_move(w, house, actor);
    }
    // spec: S-0214 table V row V6
    if q == Action::Attack && immune(w, t) {
        return Action::NoMove;
    }
    // spec: S-0214 table V row V7
    q
}

/// An aircraft on an object (S-0214 table AC).
fn aircraft_on(
    w: &World,
    house: HouseId,
    sel: &[ObjectId],
    actor: ObjectId,
    t: ObjectId,
    g: Action,
    mods: Modifiers,
) -> Action {
    // spec: S-0214 table AC rows AC2 and AC3 (a carrier carrying nothing, g select or nothing, an allied vehicle: tote, unless it stands on a vehicle factory's cell)
    // A carrier carries nothing before the aircraft topic.
    // spec: S-0214 claim 15 (a berserk carrier shows tote only with quirks.berserk_carrier_tote)
    if carrier(w, actor)
        && (!berserk(w, actor) || w.rules().quirks.berserk_carrier_tote)
        && matches!(g, Action::Select | Action::Nothing)
        && kind(w, t) == Some(Kind::Vehicle)
        && allied(w, actor, t)
    {
        let on_factory = w
            .obj(t)
            .is_some_and(|o| vehicle_factory_cell(w, o.pos.cell()));
        return if on_factory {
            Action::Nothing
        } else {
            Action::Tote
        };
    }
    match g {
        // spec: S-0214 table AC row AC1
        Action::ToggleSelect => Action::ToggleSelect,
        // spec: S-0214 table AC rows AC5 and AC6 (the aircraft's action on the target's cell)
        Action::Nothing => {
            let Some(c) = w.obj(t).map(|o| o.pos.cell()) else {
                return Action::Nothing;
            };
            let a = cell_action(w, house, sel, actor, c, mods);
            if a == Action::Attack && immune(w, t) {
                Action::NoMove
            } else {
                a
            }
        }
        // spec: S-0214 table AC row AC7
        Action::Deploy => {
            if passengers(w, actor) {
                Action::Deploy
            } else {
                Action::Nothing
            }
        }
        // spec: S-0214 table AC row AC8
        Action::Attack if first_weapon(w, actor).is_none() => Action::Nothing,
        // spec: S-0214 table AC row AC9
        Action::Attack if immune(w, t) => Action::NoMove,
        // spec: S-0214 table AC row AC10 (a repair depot: the dock list is not imported yet, so no-enter)
        Action::Select if building(w, t).is_some_and(|b| b.raw.unit_repair.unwrap_or(false)) => {
            Action::NoEnter
        }
        // spec: S-0214 table AC row AC11
        other => other,
    }
}

/// A building on an object (S-0214 table B).
fn building_on(w: &World, actor: ObjectId, t: ObjectId, g: Action) -> Action {
    // spec: S-0214 table B row B1
    if invisible(w, actor) || invisible(w, t) {
        return Action::Nothing;
    }
    match g {
        // spec: S-0214 table B row B2
        Action::ToggleSelect => Action::ToggleSelect,
        // spec: S-0214 table B row B3 (factory-marked and already primary: nothing)
        Action::Deploy if factory_marked(w, actor) && primary(w, actor) => Action::Nothing,
        // spec: S-0214 table B row B5 (a factory-marked building: deploy; absorbed units and garrison occupants are not modelled)
        Action::Deploy if factory_marked(w, actor) => Action::Deploy,
        // spec: S-0214 table B rows B4 and B6 (the gap generator's power and radius states are not modelled: nothing)
        Action::Deploy => Action::Nothing,
        // spec: S-0214 table B row B7 (a pulse cannon)
        Action::Attack if building(w, actor).is_some_and(|b| b.raw.emp_cannon.unwrap_or(false)) => {
            Action::Nothing
        }
        // spec: S-0214 table B row B8
        Action::Attack if immune(w, t) => Action::NoMove,
        // spec: S-0214 table B row B13
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Cells (claim 9)

/// The generic cell result c (S-0214 table GC).
fn generic_cell(w: &World, house: HouseId, actor: ObjectId, c: CellXY, mods: Modifiers) -> Action {
    // spec: S-0214 table GC row GC1 (berserk, melee-linked without keep-acting, being erased or phasing in: nothing)
    if berserk(w, actor) || melee_held(w, actor) || being_erased(w, actor) || phasing_in(w, actor) {
        return Action::Nothing;
    }
    // spec: S-0214 table GC row GC2
    if !own(w, house, actor) {
        return Action::Nothing;
    }
    let mv = movable(w, house, actor);
    let rally = rally_capable(w, actor);
    // spec: S-0214 table GC row GC3 (force-move and force-fire: area guard; no route points yet)
    if mods.force_move
        && mods.force_fire
        && mv
        && (able_to_fire(w, house, actor) || engineer(w, actor))
    {
        return Action::AreaGuard;
    }
    // spec: S-0214 table GC row GC4 (a weapon, force-fire or an attackable cell, movable or the cell centre in range)
    if weapon::current_weapon(w, actor).is_some()
        && (mods.force_fire || attackable_cell(w, c))
        && (mv || in_range_of(w, actor, Target::Cell(c)))
    {
        return Action::Attack;
    }
    // spec: S-0214 table GC row GC6 (outside the usable area)
    if (mv || rally) && !w.cells.usable(c) {
        return Action::NoMove;
    }
    // spec: S-0214 table GC row GC7
    if (mv || rally) && mods.force_select {
        return Action::Move;
    }
    // spec: S-0214 table GC row GC8
    if rally && mods.force_move {
        return Action::Move;
    }
    if mv {
        let reference = occupancy::reference_level(w, actor);
        let out = match kind(w, actor) {
            Some(Kind::Infantry) => occupancy::infantry_outcome(w, actor, c, reference),
            Some(Kind::Aircraft) => Outcome::Clear,
            _ => occupancy::vehicle_outcome(w, actor, c, reference),
        };
        // spec: S-0214 table GC row GC9 (entry outcome clear or cloaked blocker)
        if matches!(out, Outcome::Clear | Outcome::CloakedBlocker) {
            return Action::Move;
        }
    }
    // spec: S-0214 table GC row GC11
    if mv || rally {
        return Action::NoMove;
    }
    // spec: S-0214 table GC row GC12
    Action::Nothing
}

/// An attackable cell without force-fire (S-0214 table GC): a
/// destroyable-cliff cell without an overlay (warhead flags are not imported,
/// and walls have no owner yet).
fn attackable_cell(w: &World, c: CellXY) -> bool {
    let Some(cell) = w.cells.cell(c) else {
        return false;
    };
    let cat = w.catalogue.as_deref();
    let (s, n) = cat.map_or((0, 0), |k| (k.roles.destroyable_cliff, k.tile_count()));
    // spec: S-0214 Terms (destroyable-cliff cell: s ≤ t ≤ s + 1 and 0 ≤ t < N)
    let cliff = cell.tile >= s && cell.tile <= s + 1 && cell.tile >= 0 && cell.tile < n;
    cliff && cell.overlay == glasnost_map::NO_OVERLAY
}

/// The final action on a cell by kind (S-0214 tables IC, VC, ACC and BC);
/// no shroud, so c′ = c.
fn cell_action(
    w: &World,
    house: HouseId,
    _sel: &[ObjectId],
    actor: ObjectId,
    c: CellXY,
    mods: Modifiers,
) -> Action {
    let g = generic_cell(w, house, actor, c, mods);
    let tunnel = w.cells.cell(c).is_some_and(|x| x.land == land::TUNNEL);
    match kind(w, actor) {
        Some(Kind::Infantry) => {
            let jumpjet = uflag(w, actor, |u| u.jumpjet);
            let dipd = deployed_ipd(w, actor);
            // spec: S-0214 table IC row IC5 (a jumpjet type on a tunnel cell)
            if jumpjet && matches!(g, Action::Move | Action::NoMove) && tunnel {
                return Action::Nothing;
            }
            // spec: S-0214 table IC row IC6
            if dipd && g == Action::Move {
                return Action::NoMove;
            }
            // spec: S-0214 table IC row IC7
            if dipd
                && g == Action::Attack
                && !uflag(w, actor, |u| u.raw.deploy_fire.unwrap_or(false))
            {
                return Action::NoMove;
            }
            // spec: S-0214 table IC row IC8
            if g == Action::Nothing && !jumpjet && movable(w, house, actor) {
                return Action::NoMove;
            }
            // spec: S-0214 table IC row IC9
            g
        }
        Some(Kind::Vehicle) => {
            let cell = w.cells.cell(c);
            // spec: S-0214 table VC row VC2 (ore land and a harvester, or weed land and a weeder)
            if g == Action::Move
                && cell.is_some_and(|x| {
                    (x.land == land::TIBERIUM && uflag(w, actor, |u| u.harvester))
                        || (x.land == land::WEEDS
                            && uflag(w, actor, |u| u.raw.weeder.unwrap_or(false)))
                })
            {
                return Action::Harvest;
            }
            // spec: S-0214 table VC row VC7
            g
        }
        Some(Kind::Aircraft) => {
            // spec: S-0214 table ACC (attack without a weapon in the first slot: nothing)
            if g == Action::Attack && first_weapon(w, actor).is_none() {
                Action::Nothing
            } else {
                g
            }
        }
        Some(Kind::Building) => building_cell(
            g,
            BuildingCell {
                invisible: invisible(w, actor),
                first_slot_weapon: first_weapon(w, actor).is_some(),
                // spec: S-0214 table BC row BC4 (its projectile hits ground targets, not a pulse cannon, not being sold)
                fires_on_ground: weapon::would_use(w, actor, Some(Target::Cell(c)))
                    .1
                    .and_then(|x| w.rules().combat.projectile_of(x))
                    .is_some_and(|p| p.ag)
                    && !building(w, actor).is_some_and(|b| b.raw.emp_cannon.unwrap_or(false))
                    && w.obj(actor)
                        .is_none_or(|o| o.beh.seen() != Some(Behaviour::Selling)),
                own_rally: own(w, house, actor) && rally_capable(w, actor),
            },
            w.rules().quirks.building_cell_attack_no_weapon,
        ),
        _ => Action::Nothing,
    }
}

/// The inputs of table BC (S-0214) for one building.
#[derive(Clone, Copy, Debug, Default)]
pub struct BuildingCell {
    /// Its type is invisible in the game (row BC1).
    pub invisible: bool,
    /// A weapon is in its first slot (rows BC3).
    pub first_slot_weapon: bool,
    /// Its projectile can hit ground targets, it is not a pulse cannon and
    /// is not being sold (row BC4).
    pub fires_on_ground: bool,
    /// It is an own rally-capable building (row BC5).
    pub own_rally: bool,
}

/// A building on a cell (S-0214 table BC), from its generic cell result
/// `c` (no shroud change for buildings). Row BC2 needs the resource
/// gatherer flags, which are not imported yet (economy topic).
pub fn building_cell(c: Action, b: BuildingCell, no_weapon_attack: bool) -> Action {
    // spec: S-0214 table BC row BC1 (invisible in the game: nothing)
    if b.invisible {
        return Action::Nothing;
    }
    // spec: S-0214 table BC row BC3 (attack, no weapon in the first slot: attack; quirks.building_cell_attack_no_weapon off: nothing)
    if c == Action::Attack && !b.first_slot_weapon {
        return if no_weapon_attack {
            Action::Attack
        } else {
            Action::Nothing
        };
    }
    // spec: S-0214 table BC row BC4 (attack it can fire on the ground: attack)
    if c == Action::Attack && b.fires_on_ground {
        return Action::Attack;
    }
    // spec: S-0214 table BC row BC5 (an own rally-capable building: move, its rally point)
    if b.own_rally {
        return Action::Move;
    }
    // spec: S-0214 table BC row BC6 (attack: nothing)
    if c == Action::Attack {
        return Action::Nothing;
    }
    // spec: S-0214 table BC row BC7 (otherwise c)
    c
}

/// The action of one selected actor on a target, with the whole selection
/// in view (S-0214 claims 4 to 9).
fn action_in(
    w: &World,
    house: HouseId,
    sel: &[ObjectId],
    actor: ObjectId,
    on: ClickOn,
    mods: Modifiers,
) -> Action {
    let mods = mods.counted();
    match on {
        ClickOn::Object(t) => {
            let g = generic(w, house, sel, actor, t, mods);
            // spec: S-0214 claim 4 (an actor that is not own: g is the final action)
            if !own(w, house, actor) {
                return g;
            }
            match kind(w, actor) {
                Some(Kind::Infantry) => infantry_on(w, house, actor, t, g, mods),
                Some(Kind::Vehicle) => vehicle_on(w, house, actor, t, g),
                Some(Kind::Aircraft) => aircraft_on(w, house, sel, actor, t, g, mods),
                Some(Kind::Building) => building_on(w, actor, t, g),
                _ => Action::Nothing,
            }
        }
        ClickOn::Cell(c) => cell_action(w, house, sel, actor, c, mods),
    }
}

/// The action of one actor on a target, judged as if it were the only
/// selected object.
pub fn action(w: &World, house: HouseId, actor: ObjectId, on: ClickOn, mods: Modifiers) -> Action {
    action_in(w, house, &[actor], actor, on, mods)
}

// ---------------------------------------------------------------------------
// The representative, the click and table M (claims 1 to 3, 5, 10 to 12)

/// The representative rank (S-0214 table R).
fn rank(w: &World, id: ObjectId) -> u8 {
    // spec: S-0214 table R (0 not a unit or building, or berserk; 1 disabled; 2 not armed; 3 an armed building; 4 or 5 an armed unit)
    if !unit_or_building(w, id) || berserk(w, id) {
        0
    } else if disabled(w, id) {
        1
    } else if !armed(w, id) {
        2
    } else if kind(w, id) == Some(Kind::Building) {
        3
    } else {
        let damage = default_weapon(w, id).map(|x| w.rules().combat.weapon(x).damage);
        // spec: S-0214 table R (a default weapon doing zero or negative damage, or without damage: rank 4)
        if damage.is_some_and(|d| d > 0) { 5 } else { 4 }
    }
}

/// The representative distance (S-0214 F1).
fn rep_distance(w: &World, id: ObjectId, on: ClickOn) -> u64 {
    let a = w.reference_point(id);
    let b = match on {
        ClickOn::Object(t) => w.reference_point(t),
        // spec: S-0214 F1 (a cell target: its centre at height 0)
        ClickOn::Cell(c) => Pos::centre(c, 0),
    };
    let dx = i64::from(a.x - b.x);
    let dy = i64::from(a.y - b.y);
    let dz = i64::from(a.z - b.z);
    // spec: S-0214 F1 (d = ⌊r(n)⌋, the table square root)
    root_floor((dx * dx + dy * dy + dz * dz) as u64, true, w.rules().mode)
}

/// The representative (S-0214 claim 1): highest rank, then nearest to the
/// target, then the first in selection order (claim 12).
fn representative(w: &World, sel: &[ObjectId], on: ClickOn) -> Option<ObjectId> {
    let mut best: Option<(u8, u64, ObjectId)> = None;
    for &id in sel.iter().filter(|&&id| w.obj(id).is_some()) {
        let key = (rank(w, id), rep_distance(w, id, on));
        let better = match best {
            None => true,
            Some((r, d, _)) => key.0 > r || (key.0 == r && key.1 < d),
        };
        if better {
            best = Some((key.0, key.1, id));
        }
    }
    best.map(|b| b.2)
}

/// The action the cursor shows (S-0214 claim 1). With an empty selection:
/// *select* over a selectable object.
pub fn cursor(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    on: ClickOn,
    mods: Modifiers,
) -> Action {
    match representative(w, selection, on) {
        Some(id) => action_in(w, house, selection, id, on, mods),
        None => match on {
            ClickOn::Object(t) if selectable(w, t) => Action::Select,
            _ => Action::Nothing,
        },
    }
}

/// Whether the battlefield cursor shows the ordinary arrow instead of
/// table C's image for the representative's action (S-0214 claim 15,
/// claim 21 and Edge cases): `quirks.berserk_action_cursor` is off and the
/// representative is berserk infantry or a berserk vehicle. Presentation
/// only: [`cursor`] and [`click`] are unchanged.
pub fn cursor_shows_arrow(w: &World, selection: &[ObjectId], on: ClickOn) -> bool {
    representative(w, selection, on).is_some_and(|id| {
        berserk_cursor_arrow(
            kind(w, id),
            berserk(w, id),
            w.rules().quirks.berserk_action_cursor,
        )
    })
}

/// The rule of [`cursor_shows_arrow`] for a representative of kind `kind`.
/// `show_action` is `quirks.berserk_action_cursor`.
pub fn berserk_cursor_arrow(kind: Option<Kind>, berserk: bool, show_action: bool) -> bool {
    // spec: S-0214 Edge cases (quirks.berserk_action_cursor off: a berserk infantry or vehicle representative shows the ordinary arrow; aircraft and buildings are unaffected)
    !show_action && berserk && matches!(kind, Some(Kind::Infantry | Kind::Vehicle))
}

/// Attack-move eligibility (S-0214 claim 5).
fn eligible(w: &World, id: ObjectId) -> bool {
    matches!(kind(w, id), Some(Kind::Infantry | Kind::Vehicle))
        && armed(w, id)
        && !uflag(w, id, |u| u.raw.prevent_attack_move.unwrap_or(false))
}

/// Whether the cursor shows an attack-move (S-0214 table C, claim 5): the
/// gesture is on for the selection and the representative's order would
/// be *move* or *attack*, so the click gives an attack-move order.
pub fn shows_attack_move(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    on: ClickOn,
    mods: Modifiers,
) -> bool {
    // spec: S-0214 claim 5 (the gesture only when every selected unit or building is eligible; it changes a move or attack order, not the action)
    let am = mods.attack_move()
        && selection
            .iter()
            .all(|&id| !unit_or_building(w, id) || eligible(w, id));
    let Some(id) = am.then(|| representative(w, selection, on)).flatten() else {
        return false;
    };
    let a = action_in(w, house, selection, id, on, mods);
    matches!(
        order(w, house, id, a, on, mods, true),
        Some(Order::Behaviour {
            attack_move: true,
            ..
        })
    )
}

/// A click: each selected object resolves it itself (S-0214 claims 2, 3,
/// 5, 9 to 12 and 14).
pub fn click(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    on: ClickOn,
    mods: Modifiers,
) -> ClickResult {
    let shown = cursor(w, house, selection, on, mods);
    let first_own = selection.first().is_some_and(|&f| own(w, house, f));
    let reselect = |s: Vec<ObjectId>| ClickResult {
        selection: Some(s),
        ..ClickResult::default()
    };
    match (shown, on) {
        // spec: S-0214 claim 12 (toggle select acts only when the first selected object is own)
        (Action::ToggleSelect, ClickOn::Object(t)) if first_own => {
            let mut s: Vec<ObjectId> = selection.to_vec();
            if let Some(i) = s.iter().position(|&x| x == t) {
                s.remove(i);
            } else if selectable(w, t) {
                // spec: S-0214 Terms (can be selected: claim 12 adds only an object that can be selected) and claim 12 (the selection order of an addition; a removal keeps the others' order)
                add_to_selection(w, &mut s, t);
            } else {
                return ClickResult::default();
            }
            return reselect(s);
        }
        // spec: S-0214 claim 12 (select replaces the selection; a toggle that does not act is a select) and Terms (only with an object that can be selected)
        (Action::Select | Action::ToggleSelect, ClickOn::Object(t)) => {
            if !selectable(w, t) {
                return ClickResult::default();
            }
            return reselect(vec![t]);
        }
        // spec: S-0214 claim 3 (nothing over a selectable object not selected yet acts as select; over a selected one, nothing)
        (Action::Nothing, ClickOn::Object(t)) => {
            if selectable(w, t) && !selection.contains(&t) {
                return reselect(vec![t]);
            }
            return ClickResult::default();
        }
        // spec: S-0214 claim 3 (nothing over a cell does nothing)
        (Action::Nothing, ClickOn::Cell(_)) => return ClickResult::default(),
        _ => {}
    }
    // spec: S-0214 claim 5 (the gesture only when every selected unit or building is eligible)
    let am = mods.attack_move()
        && selection
            .iter()
            .all(|&id| !unit_or_building(w, id) || eligible(w, id));
    let mut result = ClickResult::default();
    // spec: S-0214 claim 2 (every selected object in selection order resolves the click itself, all against the state at the click)
    for (i, &id) in selection.iter().enumerate() {
        if w.obj(id).is_none() {
            continue;
        }
        let a = action_in(w, house, selection, id, on, mods);
        let o = order(w, house, id, a, on, mods, am);
        // spec: S-0214 claim 14 (only the first object of the selection gives the acknowledgement, and only when it gets an order)
        // spec: S-0214 claim 30 (one move-target marker, from the first object only: a mobile actor handling a cell click
        // with action move, a force-move or a move converted to attack-move to a place included; never a building's
        // click, a no-move click, an object click or a click refused before dispatch)
        if i == 0 && o.is_some() {
            result.acknowledge = Some(id);
            result.marker = a == Action::Move
                && matches!(on, ClickOn::Cell(_))
                && w.obj(id).is_some_and(|x| x.kind != Kind::Building);
        }
        result.orders.extend(o);
    }
    result
}

/// Where the move-target marker of a cell click stands (S-0214 claim 30):
/// the clicked cell's centre at the terrain surface height there, plus 416
/// leptons on an intact high-bridge deck cell, whether or not the cell
/// carries the deck-entry mark (the original's unmarked-deck drawing defect
/// is corrected, PLAN D6). Drawing only.
pub fn move_marker_at(w: &World, c: CellXY) -> Pos {
    // spec: S-0214 claim 30 (the cell centre (256x + 128, 256y + 128), exact integers)
    let (x, y) = (256 * c.x + 128, 256 * c.y + 128);
    // spec: S-0214 claim 30 (the terrain surface height at the centre, S-0106 claim 2, plus 416 on an intact high-bridge deck cell)
    let deck = if w.high_bridge(c) { 416 } else { 0 };
    Pos::new(x, y, w.surface_at(x, y) + deck)
}

/// The order response that acknowledges an order (S-0214 claim 14, F5):
/// entry `raw mod responses` of the applicable order-response list, where
/// `raw` is the next raw value of the unsynchronised random stream. For an
/// order kind without its own response kind that list is the type's
/// special-attack list, never its selection list (S-0701 claim 18 and F6,
/// which the audio crate applies). `None`, and no value taken, for an
/// empty list. Sound only, never game state.
pub fn order_response(raw: u32, responses: u32) -> Option<u32> {
    // spec: S-0214 F5 (i = x mod n, unsigned remainder; none for n = 0)
    raw.checked_rem(responses)
}

/// Whether an object's type has a primary weapon doing positive damage
/// (S-0214 claim 12): the type's normal first weapon, not the weapon the
/// object currently chooses, its veterancy or its ability to fire.
pub fn positive_primary(w: &World, id: ObjectId) -> bool {
    // spec: S-0214 claim 12 and Import mapping (the type's primary weapon and its Damage; no primary weapon or 0 damage: not positive)
    veterancy::type_combat(w, id)
        .and_then(|tc| tc.slots[0])
        .is_some_and(|x| w.rules().combat.weapon(x).damage > 0)
}

/// Adds an object to a selection (S-0214 claim 12, S-0213 Terms): at the
/// front when its type's primary weapon does positive damage, otherwise at
/// the end. An object already selected stays where it is.
pub fn add_to_selection(w: &World, selection: &mut Vec<ObjectId>, id: ObjectId) {
    if selection.contains(&id) {
        return;
    }
    // spec: S-0214 claim 12 (positive-primary-damage objects at the front, the others at the end)
    if positive_primary(w, id) {
        selection.insert(0, id);
    } else {
        selection.push(id);
    }
}

/// The move destination of claim 11. No cell or object is shrouded before
/// the visibility topic, so the shroud rules of the claim never apply.
fn move_destination(
    w: &World,
    actor: ObjectId,
    on: ClickOn,
    a: Action,
    mods: Modifiers,
) -> Option<Target> {
    let o = w.obj(actor)?;
    let zone = unit(w, actor).map_or(MovementZone::Normal, |u| u.zone);
    let fm = free_mover(w, actor);
    // spec: S-0214 claim 11 (in reach: in the same reach region of the actor's movement zone as the actor's own cell)
    let in_reach = |c: CellXY| {
        let r = regions::reach_in(w, c, zone);
        matches!(r, Reach::Region(_)) && r == regions::reach_in(w, o.pos.cell(), zone)
    };
    // The search of S-0212 table T5: a free mover's no-move searches without a reach region, and a
    // click on an object searches zone fly as infantry.
    let search = |c: CellXY, on_object: bool| {
        path::nearby(
            w,
            c,
            NearbyUse::MoveOrder {
                unit: actor,
                on_object,
                free_no_move: fm && a == Action::NoMove,
            },
        )
    };
    match on {
        ClickOn::Cell(c) => {
            let inside = w.cells.usable(c);
            // spec: S-0214 claim 11 (the clicked cell itself, no search: inside, and a free mover's or self-teleporting type's move, or in reach unless a free mover's no-move)
            let itself = inside
                && ((a == Action::Move && (fm || teleporter(w, actor)))
                    || (!(fm && a == Action::NoMove) && in_reach(c)));
            if itself {
                return Some(Target::Cell(c));
            }
            // spec: S-0214 claim 11 (otherwise a cell found by the search from the clicked cell; none found, no order)
            search(c, false).map(Target::Cell)
        }
        ClickOn::Object(t) => {
            let tc = w.obj(t)?.pos.cell();
            let infantry_force = o.kind == Kind::Infantry && mods.force_move;
            // spec: S-0214 claim 11 (the object itself: a move, its cell inside, and a free mover, or in reach without an infantry actor's force-move)
            if a == Action::Move && w.cells.usable(tc) && (fm || (in_reach(tc) && !infantry_force))
            {
                return Some(Target::Object(t));
            }
            // spec: S-0214 claim 11 (otherwise a cell found by the search from the object's cell, and the object itself when none is found)
            Some(search(tc, true).map_or(Target::Object(t), Target::Cell))
        }
    }
}

/// Part 1 of table M for an actor that is berserk or melee-linked while
/// its type lacks the keep-acting flag: whether its action gives no order
/// (S-0214 claim 15). The exception is *tote* for a berserk carrier
/// aircraft, under `quirks.berserk_carrier_tote`.
pub fn held_without_order(
    berserk: bool,
    melee_held: bool,
    carrier_aircraft: bool,
    action: Action,
    berserk_carrier_tote: bool,
) -> bool {
    // spec: S-0214 table M part 1 (berserk or melee-linked: no order, except tote for a berserk carrier aircraft)
    let tote = berserk && carrier_aircraft && action == Action::Tote && berserk_carrier_tote;
    (berserk || melee_held) && !tote
}

/// The order that one actor's action gives (S-0214 table M, claim 10),
/// with claim 9's rule for cells outside the usable area; `attack_move`
/// asks for the attack-move translation of claim 5.
pub fn order(
    w: &World,
    house: HouseId,
    id: ObjectId,
    a: Action,
    on: ClickOn,
    mods: Modifiers,
    attack_move: bool,
) -> Option<Order> {
    let k = kind(w, id)?;
    let q = w.rules().quirks;
    let target = match on {
        ClickOn::Object(t) => Target::Object(t),
        ClickOn::Cell(c) => Target::Cell(c),
    };
    let on_self = on == ClickOn::Object(id);
    let behaviour = |b: Behaviour, target: Option<Target>, dest: Option<Target>| {
        Some(Order::Behaviour {
            actor: id,
            behaviour: b,
            attack_move: attack_move && matches!(b, Behaviour::Move | Behaviour::Attack),
            target,
            dest,
        })
    };
    if let ClickOn::Cell(c) = on
        && k.is_mobile()
        && !w.cells.usable(c)
        && a != Action::NoMove
    {
        // spec: S-0214 claim 9 (outside the usable area only a no-move gives an order)
        return None;
    }
    let mv = movable(w, house, id);
    let fire = able_to_fire(w, house, id);
    if k == Kind::Building {
        // spec: S-0214 table M part 3 (buildings)
        return match (a, on) {
            (Action::Attack, _) => behaviour(Behaviour::Attack, Some(target), None),
            // spec: S-0214 table M part 3 (deploy: a primary order if the building is factory-marked; the unload orders of gap generators, absorbing buildings and garrisons are not modelled)
            (Action::Deploy, _) if factory_marked(w, id) => Some(Order::Primary { actor: id }),
            (Action::Move, _) | (Action::Harvest, ClickOn::Cell(_)) if rally_capable(w, id) => {
                let spot = match on {
                    // spec: S-0213 claim 20 (a) (a cell: the cell the nearby-cell search finds around it when the click is made; none found, no order)
                    ClickOn::Cell(c) => Target::Cell(rally_cell(w, id, c)?),
                    // spec: S-0213 claim 20 (c) (a factory of infantry, vehicles or aircraft: a cell searched around the clicked object's cell; any other building: the object itself)
                    ClickOn::Object(t) if makes_units(w, id) => {
                        Target::Cell(rally_cell(w, id, w.obj(t)?.pos.cell())?)
                    }
                    ClickOn::Object(_) => target,
                };
                Some(Order::GuardSpot { actor: id, spot })
            }
            _ => None,
        };
    }
    // spec: S-0214 table M part 1 (berserk or melee-linked: no order, except tote for a berserk carrier aircraft; quirks.berserk_carrier_tote)
    if held_without_order(
        berserk(w, id),
        melee_held(w, id),
        carrier(w, id),
        a,
        q.berserk_carrier_tote,
    ) {
        return None;
    }
    // spec: S-0214 table M part 1 (no order on the actor itself for attack, heal, disarm, and for a vehicle also no-move and repair)
    let self_none = match k {
        Kind::Infantry => matches!(a, Action::Attack | Action::Heal | Action::DisarmBomb),
        Kind::Vehicle => matches!(
            a,
            Action::Attack | Action::NoMove | Action::Heal | Action::RepairVehicle
        ),
        _ => false,
    };
    if on_self && self_none {
        return None;
    }
    // spec: S-0214 table M part 1 (a deployed in-place deployer's no-move on a cell gives no order)
    if deployed_ipd(w, id) && a == Action::NoMove && matches!(on, ClickOn::Cell(_)) {
        return None;
    }
    if k == Kind::Aircraft && matches!(on, ClickOn::Object(_)) {
        // spec: S-0214 table M part 1 (aircraft on an object: no-move none; enter, deploy and tote)
        match a {
            Action::NoMove => return None,
            Action::Enter => return behaviour(Behaviour::Enter, None, Some(target)),
            Action::Deploy => return behaviour(Behaviour::Unload, None, None),
            Action::Tote => return behaviour(Behaviour::Move, None, Some(target)),
            _ => {}
        }
    }
    // spec: S-0214 Edge cases (quirks.disarm_bomb_needs_weapon off: disarm bomb needs only a movable actor, and gives the attack order on the object)
    if k == Kind::Infantry && a == Action::DisarmBomb && !q.disarm_bomb_needs_weapon {
        return if mv {
            behaviour(Behaviour::Attack, Some(target), None)
        } else {
            None
        };
    }
    // spec: S-0214 table M part 1 (the infantry and vehicle "as for" rows: part 2 then applies in full, its needs included)
    let a = match (k, a) {
        (Kind::Infantry, Action::DamagingEntry | Action::RepairBuilding) => Action::Capture,
        (Kind::Infantry, Action::Heal | Action::DisarmBomb)
        | (Kind::Vehicle, Action::Heal | Action::RepairVehicle) => Action::Attack,
        _ => a,
    };
    // spec: S-0214 table M part 2 (infantry, vehicles and aircraft, with their needs)
    match a {
        Action::Move | Action::NoMove if mv => {
            let dest = move_destination(w, id, on, a, mods)?;
            behaviour(Behaviour::Move, None, Some(dest))
        }
        Action::Attack | Action::Bomb | Action::Demolish | Action::Airstrike if fire => {
            behaviour(Behaviour::Attack, Some(target), None)
        }
        Action::Enter if mv => behaviour(Behaviour::Enter, None, Some(target)),
        Action::Capture if mv => behaviour(Behaviour::Capture, None, Some(target)),
        Action::Recycle if mv => behaviour(Behaviour::Eaten, None, Some(target)),
        Action::Sabotage => behaviour(Behaviour::Sabotage, None, Some(target)),
        Action::AreaGuard => match on {
            ClickOn::Object(_) if fire && mv => behaviour(Behaviour::AreaGuard, Some(target), None),
            ClickOn::Cell(_) if mv && (armed(w, id) || engineer(w, id)) => {
                behaviour(Behaviour::AreaGuard, Some(target), None)
            }
            _ => None,
        },
        Action::Deploy | Action::AreaAttack => behaviour(Behaviour::Unload, None, None),
        Action::Harvest => behaviour(Behaviour::Harvest, None, Some(target)),
        // spec: S-0214 Part 2 (idle: guard, or area guard if the actor's first weapon heals; the actor itself)
        Action::Idle if heals(w, first_weapon(w, id)) => {
            behaviour(Behaviour::AreaGuard, Some(Target::Object(id)), None)
        }
        Action::Idle => behaviour(Behaviour::Guard, Some(Target::Object(id)), None),
        _ => None,
    }
}

/// The last accepted cheer, kept by the input side across the games of a
/// session (S-0213 claim 19).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CheerMemory {
    /// Tick of the last accepted cheer.
    pub last: i32,
}

impl Default for CheerMemory {
    fn default() -> Self {
        // spec: S-0213 claim 19 (at program start the last-cheer tick is −100)
        Self { last: -100 }
    }
}

impl CheerMemory {
    /// A game starts: with `quirks.cheer_memory_spans_session` off the
    /// memory is reset (S-0213 claim 19).
    pub fn game_start(&mut self, w: &World) {
        if !w.rules().quirks.cheer_memory_spans_session {
            *self = Self::default();
        }
    }

    /// The cheer command (S-0213 table T2, claim 19): the order to issue,
    /// when the rate limit lets it through. An accepted press is recorded
    /// even if the pending list then drops the order.
    pub fn cheer(&mut self, w: &World) -> Option<Order> {
        // spec: S-0213 Formulas (accepted ⇔ t > L + R)
        if w.tick > self.last.wrapping_add(w.rules().general.cheer_rate) {
            self.last = w.tick;
            Some(Order::Cheer)
        } else {
            None
        }
    }
}

/// A key command for the selection (S-0213 table T2 and claim 18): each
/// selected object in selection order gets at most one order.
pub fn key_command(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    cmd: KeyCommand,
) -> Vec<Order> {
    let mut out = Vec::new();
    for &id in selection {
        let Some(o) = w.obj(id) else { continue };
        let mv = movable(w, house, id);
        let fire = able_to_fire(w, house, id);
        let order = match cmd {
            // spec: S-0213 table T2 (stop: movable or able to fire)
            KeyCommand::Stop if mv || fire => Some(Order::Stop { actor: id }),
            // spec: S-0213 table T2 (scatter: movable)
            KeyCommand::Scatter if mv => Some(Order::Scatter { actor: id }),
            KeyCommand::Guard => {
                let harvester = o.kind == Kind::Vehicle && unit(w, id).is_some_and(|u| u.harvester);
                // spec: S-0213 table T2 (guard: movable, able to fire and not a building; or a harvester vehicle)
                ((mv && fire && o.kind != Kind::Building) || harvester).then(|| Order::Behaviour {
                    actor: id,
                    behaviour: Behaviour::AreaGuard,
                    attack_move: false,
                    target: Some(Target::Cell(o.pos.cell())),
                    dest: None,
                })
            }
            // spec: S-0213 table T2 (deploy: units that are movable or able to fire; deploy readiness waits for the deploy specs)
            KeyCommand::Deploy if (mv || fire) && o.kind != Kind::Building => {
                Some(Order::Deploy { actor: id })
            }
            _ => None,
        };
        out.extend(order);
    }
    out
}
