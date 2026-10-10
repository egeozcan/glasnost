//! What a click or key command means (S-0214, and S-0213 table T2): the
//! input rules that turn a house's selection and a target into orders. A
//! person's clicks and our AI houses use the same functions (S-0214 Terms,
//! *acting house*), and the orders they return go through
//! [`crate::orders::issue`] like any other.
//!
//! Selection is input-side state, not game state (S-0213 Terms): the caller
//! keeps it and passes it in, in selection order.

use glasnost_map::CellXY;

use crate::orders::Order;
use crate::rules::{Behaviour, Kind};
use crate::world::{HouseId, ObjectId, Target, World};

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

/// An action (S-0214 table A).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    /// Area guard.
    AreaGuard,
    /// Capture.
    Capture,
}

/// The result of a click.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClickResult {
    /// Orders, in selection order, for the acting house to issue.
    pub orders: Vec<Order>,
    /// The new selection, when the click changed it.
    pub selection: Option<Vec<ObjectId>>,
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

/// Whether `id` is an own object of `house` (S-0214 Terms).
fn own(w: &World, house: HouseId, id: ObjectId) -> bool {
    w.obj(id).is_some_and(|o| o.owner == Some(house))
}

/// Whether the acting house can select the object: a live unit or building
/// on the map.
pub fn selectable(w: &World, id: ObjectId) -> bool {
    w.obj(id)
        .is_some_and(|o| o.kind != Kind::Terrain && o.on_map && o.health > 0)
}

/// Movable (S-0214 Terms), for the states our engine has so far: an own,
/// live, on-map infantry unit or vehicle. Buildings are not movable until
/// rally points exist.
pub fn movable(w: &World, house: HouseId, id: ObjectId) -> bool {
    own(w, house, id)
        && w.obj(id)
            .is_some_and(|o| matches!(o.kind, Kind::Infantry | Kind::Vehicle) && o.on_map)
}

/// Armed (S-0214 Terms): a weapon in its first slot.
pub fn armed(w: &World, id: ObjectId) -> bool {
    let Some(o) = w.obj(id) else { return false };
    let r = w.rules();
    match o.kind {
        Kind::Building => r
            .buildings
            .get(o.type_index as usize)
            .is_some_and(|b| b.raw.primary.is_some()),
        Kind::Terrain => false,
        _ => r
            .unit(o.kind, o.type_index)
            .is_some_and(|u| u.primary.is_some()),
    }
}

/// Able to fire (S-0214 Terms): own and armed.
pub fn able_to_fire(w: &World, house: HouseId, id: ObjectId) -> bool {
    own(w, house, id) && armed(w, id)
}

/// The action of one selected actor on a target (S-0214 claims 4 to 9).
///
/// First version: own units move to cells and to allied objects, attack
/// enemy units and buildings, and select own objects; the full tables G, I,
/// V, AC, B, GC, IC, VC, ACC and BC replace it.
pub fn action(w: &World, house: HouseId, actor: ObjectId, on: ClickOn, mods: Modifiers) -> Action {
    let mods = mods.counted();
    if !own(w, house, actor) {
        return match on {
            ClickOn::Object(t) if selectable(w, t) => Action::Select,
            _ => Action::Nothing,
        };
    }
    match on {
        ClickOn::Cell(_) => {
            if mods.force_fire && able_to_fire(w, house, actor) {
                Action::Attack
            } else if movable(w, house, actor) {
                Action::Move
            } else {
                Action::Nothing
            }
        }
        ClickOn::Object(t) => {
            if mods.force_select {
                return Action::ToggleSelect;
            }
            let enemy = w
                .obj(t)
                .is_some_and(|x| x.owner.is_some() && !w.allied(Some(house), x.owner));
            if (mods.force_fire || enemy) && able_to_fire(w, house, actor) && t != actor {
                Action::Attack
            } else if mods.force_move && movable(w, house, actor) {
                Action::Move
            } else if selectable(w, t) && own(w, house, t) {
                Action::Select
            } else if movable(w, house, actor) {
                Action::Move
            } else {
                Action::Nothing
            }
        }
    }
}

/// The representative's action, which the cursor shows (S-0214 claim 1).
/// With an empty selection: *select* over a selectable object.
pub fn cursor(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    on: ClickOn,
    mods: Modifiers,
) -> Action {
    let rep = selection.iter().copied().find(|&id| w.obj(id).is_some());
    match rep {
        Some(id) => action(w, house, id, on, mods),
        None => match on {
            ClickOn::Object(t) if selectable(w, t) => Action::Select,
            _ => Action::Nothing,
        },
    }
}

/// A click: each selected object resolves it itself (S-0214 claims 2, 3,
/// 10 and 12).
pub fn click(
    w: &World,
    house: HouseId,
    selection: &[ObjectId],
    on: ClickOn,
    mods: Modifiers,
) -> ClickResult {
    let shown = cursor(w, house, selection, on, mods);
    match (shown, on) {
        // spec: S-0214 claim 12 (select replaces the selection with the clicked object)
        (Action::Select, ClickOn::Object(t)) => {
            return ClickResult {
                orders: Vec::new(),
                selection: Some(vec![t]),
            };
        }
        // spec: S-0214 claim 12 (toggle select adds or removes the clicked object)
        (Action::ToggleSelect, ClickOn::Object(t)) => {
            let mut s: Vec<ObjectId> = selection.to_vec();
            if let Some(i) = s.iter().position(|&x| x == t) {
                s.remove(i);
            } else {
                s.push(t);
            }
            return ClickResult {
                orders: Vec::new(),
                selection: Some(s),
            };
        }
        // spec: S-0214 claim 3 (nothing over a cell does nothing)
        (Action::Nothing, _) => return ClickResult::default(),
        _ => {}
    }
    let am = mods.attack_move()
        && selection.iter().all(|&id| {
            w.obj(id)
                .is_some_and(|o| matches!(o.kind, Kind::Infantry | Kind::Vehicle))
                && armed(w, id)
        });
    let mut orders = Vec::new();
    // spec: S-0214 claim 2 (every selected object in selection order resolves the click itself)
    for &id in selection {
        let a = action(w, house, id, on, mods);
        let target = match on {
            ClickOn::Object(t) => Target::Object(t),
            ClickOn::Cell(c) => Target::Cell(c),
        };
        // spec: S-0214 table M (move: behaviour move with the destination; attack: behaviour attack with the target)
        let order = match a {
            Action::Move => Some(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Move,
                attack_move: am,
                target: None,
                dest: Some(target),
            }),
            Action::Attack => Some(Order::Behaviour {
                actor: id,
                behaviour: Behaviour::Attack,
                attack_move: am,
                target: Some(target),
                dest: None,
            }),
            _ => None,
        };
        orders.extend(order);
    }
    ClickResult {
        orders,
        selection: None,
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
                let harvester = o.kind == Kind::Vehicle
                    && w.rules()
                        .unit(o.kind, o.type_index)
                        .is_some_and(|u| u.harvester);
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
