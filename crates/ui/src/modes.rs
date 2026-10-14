//! The sidebar's sell, repair and power modes (S-0214 claim 13): which
//! action the cursor shows over an object and the order a click gives.

use glasnost_sim::econ::{sell, shroud};
use glasnost_sim::orders::Order;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn};
use glasnost_sim::walls;
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
/// action (*no-sell*, *no-repair*, *no-power*). A click that picks no
/// object in sell mode gives a sell-wall order for the cursor cell when it
/// holds a wall of the house that the house has explored.
pub fn order(w: &World, house: HouseId, mode: Mode, on: ClickOn) -> Option<Order> {
    let id = match on {
        ClickOn::Object(id) => id,
        ClickOn::Cell(_) if mode == Mode::Sell => return wall_sale(w, house, on),
        ClickOn::Cell(_) => return None,
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

/// The sell-wall order of a sell-mode click that picks no object (S-0214
/// claim 13, S-0408 claim 12a): the cursor cell holds a wall overlay owned
/// by the acting house and is not shrouded for it. Our engine has no fog
/// of war, so no cell is fogged. The builder and the unsellable flag are
/// not tested here; the order's application tests them.
fn wall_sale(w: &World, house: HouseId, on: ClickOn) -> Option<Order> {
    let ClickOn::Cell(cell) = on else {
        return None;
    };
    let i = w.cells.idx(cell)?;
    let overlay = w.cells.cell(cell)?.overlay;
    // spec: S-0214 claim 13 (a wall cell owned by the acting house)
    let own_wall = walls::is_wall(w, overlay) && w.cells.wall_owner[i] == Some(house);
    // spec: S-0214 claim 13 (neither shrouded nor fogged for the house; S-0214 Open questions: no skirmish setting turns fog of war on)
    let visible = shroud::explored(w, house, cell);
    // spec: S-0214 claim 13 (only when no individual object was picked; the target is the cursor cell, S-0101 claims 9 and 10)
    (own_wall && visible).then_some(Order::SellWall { cell })
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_rules::sim::OverlayType;
    use glasnost_sim::testkit;

    const SIZE: i32 = 40;

    // spec: S-0214 claim 13 (sell mode with no picked object: a sell-wall order for the cursor cell when it holds an own wall that is not shrouded; others' walls, ownerless walls, other overlays and shrouded cells give none)
    #[test]
    fn claim_13_sell_wall_at_the_cursor_cell() {
        let mut r = testkit::ruleset();
        r.registries
            .insert("overlays".into(), vec!["O0".into(), "O1".into()]);
        r.sim.overlays.insert(
            "O0".into(),
            OverlayType {
                wall: Some(true),
                ..Default::default()
            },
        );
        r.sim.overlays.insert("O1".into(), OverlayType::default());
        let mut w = testkit::game_with_non_players(&r, SIZE, 13);
        let (a, b) = (HouseId(0), HouseId(1));
        let c = |dx| testkit::mid(SIZE, dx, 0);
        let put = |w: &mut World, cell, overlay: u8, owner| {
            w.cells.grid.get_mut(cell).unwrap().overlay = overlay;
            let i = w.cells.idx(cell).unwrap();
            w.cells.wall_owner[i] = owner;
        };
        put(&mut w, c(0), 0, Some(a));
        put(&mut w, c(2), 0, Some(b));
        put(&mut w, c(4), 0, None);
        put(&mut w, c(6), 1, Some(a));
        let sell = |w: &World, cell| order(w, a, Mode::Sell, ClickOn::Cell(cell));
        assert!(!shroud::explored(&w, a, c(0)), "starts shrouded");
        assert_eq!(sell(&w, c(0)), None, "a shrouded own wall gives none");
        shroud::defeat_reveal(&mut w, a);
        assert!(shroud::explored(&w, a, c(0)));
        assert_eq!(sell(&w, c(0)), Some(Order::SellWall { cell: c(0) }));
        assert_eq!(sell(&w, c(2)), None, "another house's wall");
        assert_eq!(sell(&w, c(4)), None, "an ownerless wall");
        assert_eq!(sell(&w, c(6)), None, "not a wall overlay");
        assert_eq!(sell(&w, c(8)), None, "an empty cell");
        let other = |m| order(&w, a, m, ClickOn::Cell(c(0)));
        assert_eq!(other(Mode::Repair), None);
        assert_eq!(other(Mode::Power), None);
    }
}
