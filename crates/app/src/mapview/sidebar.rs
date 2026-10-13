//! The live game's sidebar as the viewer shows it (M5a placeholder layout,
//! our own; the original-looking layout is M5b): the controlled house's
//! credits and power, its build icons in one tab per production line kind,
//! each icon's state, and what a click on an icon or, in placement mode, on
//! a cell does. The rules come from the simulation's `econ::sidebar`
//! (S-0411): this module only groups and reads them, so the viewer and the
//! tests see the same model without egui.

use glasnost_map::CellXY;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::econ::placement;
use glasnost_sim::econ::production::{self, LineKind};
use glasnost_sim::econ::sidebar::{self, Click, ClickOutcome};
use glasnost_sim::orders::Order;
use glasnost_sim::world::{HouseId, World};

/// A tab of the placeholder sidebar: one per production line kind
/// (S-0403 Terms), in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tab {
    /// Ordinary buildings.
    Buildings,
    /// Defences.
    Defences,
    /// Infantry.
    Infantry,
    /// Ground vehicles.
    Vehicles,
    /// Naval vehicles.
    Naval,
    /// Aircraft.
    Aircraft,
}

impl Tab {
    /// Every tab, in order.
    pub const ALL: [Tab; 6] = [
        Tab::Buildings,
        Tab::Defences,
        Tab::Infantry,
        Tab::Vehicles,
        Tab::Naval,
        Tab::Aircraft,
    ];

    /// The tab's label.
    pub fn label(self) -> &'static str {
        match self {
            Tab::Buildings => "Buildings",
            Tab::Defences => "Defences",
            Tab::Infantry => "Infantry",
            Tab::Vehicles => "Vehicles",
            Tab::Naval => "Naval",
            Tab::Aircraft => "Aircraft",
        }
    }

    fn of(k: LineKind) -> Tab {
        match k {
            LineKind::Building => Tab::Buildings,
            LineKind::Defence => Tab::Defences,
            LineKind::Infantry => Tab::Infantry,
            LineKind::Ground => Tab::Vehicles,
            LineKind::Naval => Tab::Naval,
            LineKind::Aircraft => Tab::Aircraft,
        }
    }
}

/// One build icon (cameo).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cameo {
    /// The type.
    pub t: TypeRef,
    /// The type's name.
    pub name: String,
    /// The price the house pays (S-0404).
    pub price: i32,
    /// Drawn darkened (S-0411 claim 3).
    pub darkened: bool,
    /// The clock stage, 1 to 55, while the line's item is of this type
    /// (S-0411 claim 8).
    pub stage: Option<i32>,
    /// The *ready* label (S-0411 claim 8).
    pub ready: bool,
    /// The *on hold* label (S-0411 claim 8).
    pub on_hold: bool,
    /// The queue count shown (S-0411 claim 7).
    pub queued: Option<i32>,
}

impl Cameo {
    /// The build progress from 0 to 1 for a progress bar: the clock stage
    /// over its last stage (S-0411 claim 8: stages 1 to 55).
    pub fn progress(&self) -> Option<(i32, i32)> {
        // spec: S-0411 Formulas (clock stage = progress + 1, progress 0 to 54: stages 1 to 55)
        self.stage.map(|s| (s, 55))
    }
}

/// The sidebar of one house.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
    /// Available credits (S-0404 claim 4).
    pub credits: i32,
    /// Power output and drain (S-0405 claim 4).
    pub power: (i32, i32),
    /// The tabs that have icons, in tab order, with their icons in the
    /// house's icon order (S-0411 claim 2).
    pub tabs: Vec<(Tab, Vec<Cameo>)>,
}

impl Model {
    /// The icons of a tab.
    pub fn tab(&self, tab: Tab) -> &[Cameo] {
        self.tabs
            .iter()
            .find(|(t, _)| *t == tab)
            .map_or(&[], |(_, c)| c.as_slice())
    }
}

/// The sidebar model of `house`.
pub fn model(w: &World, house: HouseId) -> Model {
    let mut tabs: Vec<(Tab, Vec<Cameo>)> = Vec::new();
    for t in sidebar::icons(w, house) {
        let Some(k) = production::line_kind(w, t) else {
            continue;
        };
        let state = sidebar::icon_state(w, house, t);
        let cameo = Cameo {
            t,
            name: super::live::type_name(w.rules(), t.kind, t.index),
            price: production::price(w, house, t),
            darkened: sidebar::darkened(w, house, t),
            stage: state.map(|s| s.stage),
            ready: state.is_some_and(|s| s.ready),
            on_hold: state.is_some_and(|s| s.on_hold),
            queued: sidebar::queue_count(w, house, t),
        };
        let tab = Tab::of(k);
        match tabs.iter_mut().find(|(x, _)| *x == tab) {
            Some((_, v)) => v.push(cameo),
            None => tabs.push((tab, vec![cameo])),
        }
    }
    tabs.sort_by_key(|(t, _)| *t);
    let (output, drain) = w
        .houses
        .get(usize::from(house.0))
        .map_or((0, 0), |h| (h.econ.power.output, h.econ.power.drain));
    Model {
        credits: production::available_credits(w, house),
        power: (output, drain),
        tabs,
    }
}

/// What a click on an icon leads to in the viewer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Issue this order.
    Order(Order),
    /// Enter placement mode for this building type.
    Place(TypeRef),
    /// Refused (S-0411 claim 4 row 6).
    Refused,
    /// Nothing.
    Nothing,
}

/// A click on the icon of `t` (S-0411 claims 4 and 5).
pub fn click(w: &World, house: HouseId, t: TypeRef, button: Click, shift: bool) -> Action {
    match sidebar::click(w, house, t, button, shift) {
        ClickOutcome::Order(o) => Action::Order(o),
        ClickOutcome::PlacementMode => Action::Place(t),
        ClickOutcome::Refused => Action::Refused,
        ClickOutcome::Nothing => Action::Nothing,
    }
}

/// The top-left cell a click on `cursor` names in placement mode (S-0406
/// claim 7, F2).
pub fn top_left(w: &World, t: TypeRef, cursor: CellXY) -> CellXY {
    placement::click_cell(w, t.index, cursor)
}

/// The placement preview with the cursor on `cursor`: the foundation cells
/// a click there names, and whether the building would be placed there
/// (the place order's conditions, S-0406 claims 4 to 6 and 12, and the
/// cell test, claims 3 and 8; an upgrade type needs only the former).
pub fn preview(w: &World, house: HouseId, t: TypeRef, cursor: CellXY) -> (Vec<CellXY>, bool) {
    let cell = top_left(w, t, cursor);
    let upgrade = placement::place_type(w, t.index).upgrade_of.is_some();
    let ok = placement::place_legal(w, house, t, cell)
        && (upgrade || placement::cell_test(w, t.index, cell, &placement::Test::order(house)));
    (placement::foundation_cells(w, t.index, cell), ok)
}

/// The place order of a click on `cursor` in placement mode, when the
/// order may be issued (S-0406 claims 4 to 6 and 12; a cell test that
/// fails when it is applied places nothing).
pub fn place(w: &World, house: HouseId, t: TypeRef, cursor: CellXY) -> Option<Order> {
    let cell = top_left(w, t, cursor);
    placement::place_legal(w, house, t, cell).then_some(Order::Place {
        what: t,
        cell: Some(cell),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use glasnost_ai::fixture;
    use glasnost_sim::{load, orders, testkit, tick};

    fn game() -> World {
        load::new_game(
            fixture::rules(),
            &fixture::map(),
            testkit::catalogue(),
            &fixture::setup(3),
        )
        .expect("the synthetic skirmish loads")
    }

    fn named(m: &Model, tab: Tab, name: &str) -> Cameo {
        m.tab(tab)
            .iter()
            .find(|c| c.name == name)
            .cloned()
            .unwrap_or_else(|| panic!("{name} in {tab:?}: {m:?}"))
    }

    fn run(w: &mut World, n: i32) {
        for _ in 0..n {
            tick::step(w);
        }
    }

    // The base unit deploys; the yard offers its buildings in the Buildings
    // tab; a left click builds, the icon counts up and becomes ready; a
    // click on the ready icon enters placement; a legal cell places it.
    // spec: S-0411 claims 2, 3, 4 and 8 (the icons, a build click, the clock and ready state, placement mode); S-0406 F2 (the click names the top-left cell)
    #[test]
    fn build_and_place_through_the_sidebar() {
        let mut w = game();
        let h = HouseId(0);
        run(&mut w, 5);
        let mcv = w
            .objects
            .values()
            .find(|o| o.owner == Some(h) && o.kind == glasnost_sim::rules::Kind::Vehicle)
            .map(|o| o.id)
            .expect("the base unit");
        assert!(orders::issue(&mut w, h, Order::Deploy { actor: mcv }));
        run(&mut w, 200);
        let m = model(&w, h);
        assert!(m.credits > 0);
        let powr = named(&m, Tab::Buildings, "POWR");
        assert!(!powr.darkened);
        assert_eq!(powr.stage, None);
        let Action::Order(o) = click(&w, h, powr.t, Click::Left, false) else {
            panic!("a build order");
        };
        assert!(orders::issue(&mut w, h, o));
        run(&mut w, 30);
        let m = model(&w, h);
        let powr = named(&m, Tab::Buildings, "POWR");
        let (stage, last) = powr.progress().expect("in progress");
        assert!((1..=last).contains(&stage));
        // Another building of the line is darkened while it has an item.
        assert!(named(&m, Tab::Buildings, "PROC").darkened);
        assert_eq!(
            click(
                &w,
                h,
                named(&m, Tab::Buildings, "PROC").t,
                Click::Left,
                false
            ),
            Action::Refused
        );
        let mut ticks = 0;
        while !named(&model(&w, h), Tab::Buildings, "POWR").ready {
            run(&mut w, 1);
            ticks += 1;
            assert!(ticks < 5000, "the item finishes");
        }
        assert_eq!(
            click(&w, h, powr.t, Click::Left, false),
            Action::Place(powr.t)
        );
        // On the yard the preview is red; beside it, green.
        let yard = w
            .objects
            .values()
            .find(|o| o.owner == Some(h) && o.kind == glasnost_sim::rules::Kind::Building)
            .map(|o| o.pos.cell())
            .expect("the yard");
        let (cells, ok) = preview(&w, h, powr.t, yard);
        assert_eq!(cells.len(), 4);
        assert!(!ok);
        let cursor = (0..6)
            .flat_map(|dy| (-6..7).map(move |dx| CellXY::new(yard.x + dx, yard.y + 4 + dy)))
            .find(|&c| preview(&w, h, powr.t, c).1)
            .expect("a legal cell near the yard");
        let cell = top_left(&w, powr.t, cursor);
        let before = w.objects.len();
        let o = place(&w, h, powr.t, cursor).expect("legal");
        assert!(orders::issue(&mut w, h, o));
        run(&mut w, 2);
        assert!(w.objects.len() >= before);
        assert!(
            w.objects.values().any(|o| o.owner == Some(h)
                && o.kind == glasnost_sim::rules::Kind::Building
                && o.pos.cell() == cell),
            "the power plant stands at the chosen cell"
        );
        let m = model(&w, h);
        assert!(m.power.0 > 0, "power output: {:?}", m.power);
    }

    // A right click on a running item holds it; the icon shows on hold.
    // spec: S-0411 claims 5 and 8 (hold, the on-hold label)
    #[test]
    fn right_click_holds() {
        let mut w = game();
        let h = HouseId(0);
        run(&mut w, 5);
        let mcv = w
            .objects
            .values()
            .find(|o| o.owner == Some(h) && o.kind == glasnost_sim::rules::Kind::Vehicle)
            .map(|o| o.id)
            .expect("the base unit");
        assert!(orders::issue(&mut w, h, Order::Deploy { actor: mcv }));
        run(&mut w, 200);
        let t = named(&model(&w, h), Tab::Buildings, "POWR").t;
        let Action::Order(o) = click(&w, h, t, Click::Left, false) else {
            panic!("a build order");
        };
        assert!(orders::issue(&mut w, h, o));
        run(&mut w, 10);
        let Action::Order(o) = click(&w, h, t, Click::Right, false) else {
            panic!("a hold order");
        };
        assert!(matches!(o, Order::Hold { .. }));
        assert!(orders::issue(&mut w, h, o));
        run(&mut w, 2);
        assert!(named(&model(&w, h), Tab::Buildings, "POWR").on_hold);
    }
}
