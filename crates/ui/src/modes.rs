//! The sidebar's sell, repair and power modes (S-0214 claim 13): which
//! action the cursor shows over an object and the order a click gives.

use glasnost_sim::econ::sell;
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn};
use glasnost_sim::world::{HouseId, World};

/// A sidebar mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Sell mode.
    Sell,
    /// Repair mode.
    Repair,
    /// Power mode.
    Power,
}

/// The order a click in `mode` on `on` gives, `None` for the refused
/// action (*no-sell*, *no-repair*, *no-power*). Wall cells give no sell
/// order yet (the wall branch of claim 13 needs the shroud and fog test of
/// the drawing, which is not wired; see docs/development.md).
pub fn order(w: &World, house: HouseId, mode: Mode, on: ClickOn) -> Option<Order> {
    let ClickOn::Object(id) = on else {
        return None;
    };
    let o = w.obj(id)?;
    // spec: S-0214 claim 13 (every mode acts on an own object only)
    if o.owner != Some(house) || o.kind != Kind::Building {
        return None;
    }
    let b = w.rules().buildings.get(o.type_index as usize)?;
    match mode {
        // spec: S-0214 claim 13 (sell: an own building that can be sold and is not vehicle-like)
        Mode::Sell => (sell::sellable(w, id) && !sell::vehicle_like(w, id))
            .then_some(Order::Sell { actor: id }),
        // spec: S-0214 claim 13 (repair switch over an own object that can be repaired; a repair order only on a building)
        Mode::Repair => sell::repairable(w, id).then_some(Order::Repair { actor: id }),
        Mode::Power => {
            let p = w.rules().econ.power.buildings.get(o.type_index as usize)?;
            // spec: S-0214 claim 13 (selectable, not vehicle-like, the type can be switched off, base drain above 0 or the needs-power flag; the type only)
            // spec: S-0214 Import mapping (TogglePower absent: yes)
            let ok = targeting::selectable(w, id)
                && !sell::vehicle_like(w, id)
                && b.raw.toggle_power.unwrap_or(true)
                && (p.drain > 0 || p.needs_power);
            // spec: S-0214 claim 13 (power-on for an offline building, power-off for an online one)
            let order = if o.econ.building.switched_off {
                Order::PowerOn { actor: id }
            } else {
                Order::PowerOff { actor: id }
            };
            ok.then_some(order)
        }
    }
}
