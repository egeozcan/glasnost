//! The client model headless: a person's house against an AI house on the
//! synthetic skirmish (`glasnost_ai::fixture`), selecting, moving,
//! deploying, building and placing through the same calls the window
//! makes, with no GPU.

use super::*;
use glasnost_ai::fixture;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::Target;
use glasnost_sim::testkit;

fn start() -> Play {
    let rules = fixture::rules();
    let players = [
        Player {
            name: "Person".into(),
            country: "Alphaland".into(),
            ..Default::default()
        },
        Player {
            name: "Machine".into(),
            country: "Betaland".into(),
            ai: true,
            ..Default::default()
        },
    ];
    let s = setup::skirmish(&rules, &players, Settings::default(), 0, 7);
    let local = local_house(&s).expect("one person's house");
    let game = Game::new(
        rules,
        &fixture::map(),
        "synthetic-skirmish",
        testkit::catalogue(),
        s,
    )
    .expect("the synthetic skirmish loads");
    assert_eq!(game.ai_houses().len(), 1, "only the machine is an AI house");
    Play::new(game, local)
}

fn own(p: &Play, kind: Kind) -> Vec<ObjectId> {
    p.world()
        .objects
        .values()
        .filter(|o| o.owner == Some(p.local) && o.kind == kind && o.on_map)
        .map(|o| o.id)
        .collect()
}

fn step_until(p: &mut Play, limit: i32, done: impl Fn(&Play) -> bool) {
    let end = p.world().tick + limit;
    while !done(p) {
        assert!(p.world().tick < end, "gave up at tick {}", p.world().tick);
        assert!(p.failed.is_none(), "{:?}", p.failed);
        p.step();
    }
}

/// spec: S-0214 claim 12 (a click selects), S-0213 claim 10 (deploy),
/// S-0201 claim 14 (no orders before tick 4), S-0411 claims 4 and 10
/// (build and place from the sidebar)
#[test]
fn select_move_deploy_build_and_place_through_the_client_model() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    // A left click on the unit selects it and orders nothing.
    let cell = p.world().obj(mcv).unwrap().pos.cell();
    p.click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    assert_eq!(p.selection, [mcv]);
    assert!(p.game.replay().orders.is_empty());
    // spec: S-0701 claim 17 (the newly selected object gives the selection response)
    assert_eq!(std::mem::take(&mut p.selected), [mcv]);
    // A left click on the ground clears it.
    p.click_map(Button::Left, None, cell, Modifiers::default());
    assert!(p.selection.is_empty());
    p.click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    // A right click on a cell moves it.
    let to = CellXY::new(cell.x, cell.y + 2);
    p.click_map(Button::Right, None, to, Modifiers::default());
    assert_eq!(p.game.replay().orders.len(), 1, "the move is in the replay");
    // spec: S-0701 claim 18 (the click's orders and selection go to the order response); the order selects nothing new
    assert_eq!(p.acks.len(), 1);
    assert_eq!(p.acks[0].0, [mcv]);
    assert_eq!(std::mem::take(&mut p.selected), [mcv], "only the second left click");
    p.step();
    assert_eq!(p.world().obj(mcv).unwrap().dest, Some(Target::Cell(to)));
    step_until(&mut p, 400, |p| {
        p.world().obj(mcv).is_some_and(|o| o.pos.cell() == to)
    });
    // Deploy (D) turns it into the base building.
    p.key(KeyCommand::Deploy);
    // spec: S-0701 claim 19 (a key command that issued orders)
    assert_eq!(p.keys, [KeyCommand::Deploy]);
    step_until(&mut p, 400, |p| !own(p, Kind::Building).is_empty());
    let first_buildings = own(&p, Kind::Building).len();
    // A building icon appears; a left click starts it.
    step_until(&mut p, 50, |p| {
        !p.world().houses[usize::from(p.local.0)].econ.tech.icons[0].is_empty()
    });
    p.tab_click(0);
    let t = p.world().houses[usize::from(p.local.0)].econ.tech.icons[0][0].t;
    p.cameo_click(0, Button::Left, false);
    assert!(
        p.cues.contains(&UiCue::BuildStarted),
        "the click started the item: {:?}",
        p.cues
    );
    step_until(&mut p, 4000, |p| icon_ready(p.world(), p.local, t));
    assert!(p.cues.contains(&UiCue::ConstructionComplete));
    // A left click on the ready icon enters placement mode.
    p.cameo_click(0, Button::Left, false);
    assert_eq!(p.tool, Tool::Place(t));
    // Find a cell the preview shows green, then place there.
    let yard = p.world().obj(own(&p, Kind::Building)[0]).unwrap().pos.cell();
    let spot = (-8..=8)
        .flat_map(|dy| (-8..=8).map(move |dx| CellXY::new(yard.x + dx, yard.y + dy)))
        .find(|&c| p.preview(c).is_some_and(|(_, ok)| ok))
        .expect("a legal cell near the base");
    p.click_map(Button::Left, None, spot, Modifiers::default());
    assert_eq!(p.tool, Tool::Select, "placement mode ends");
    // spec: S-0701 claim 23 (the placement sound)
    assert!(p.cues.contains(&UiCue::BuildingPlaced));
    step_until(&mut p, 10, |p| {
        own(p, Kind::Building).len() > first_buildings
    });
    // Every order went through the replay.
    let orders: Vec<&Order> = p.game.replay().orders.iter().map(|(_, _, o)| o).collect();
    assert!(orders.iter().any(|o| matches!(o, Order::Deploy { .. })));
    assert!(orders.iter().any(|o| matches!(o, Order::Build { .. })));
    assert!(
        orders
            .iter()
            .any(|o| matches!(o, Order::Place { cell: Some(_), .. }))
    );
    // The end screen is not there while the game runs.
    assert!(p.end(60).is_none());
}

/// Interim (Q-0278): a box takes only the local house's mobile objects.
#[test]
fn box_selection_takes_own_mobile_objects() {
    let mut p = start();
    let all: Vec<ObjectId> = p.world().objects.keys().copied().collect();
    p.select_box(&all, false);
    assert!(!p.selection.is_empty());
    // spec: S-0701 claim 17 (one input selecting several objects: only the first speaks)
    assert_eq!(p.selected, [p.selection[0]]);
    for id in &p.selection {
        let o = p.world().obj(*id).unwrap();
        assert_eq!(o.owner, Some(p.local));
        assert!(o.kind.is_mobile());
    }
    p.select_box(&[], false);
    assert!(p.selection.is_empty());
}

/// spec: S-0701 claim 24 (the strip scrolled past its end: the refusal sound); interim: Q-0339 (a click that cannot move the strip)
#[test]
fn scrolling_past_the_strip_end_cues_the_refusal_sound() {
    let mut p = start();
    p.cues.clear();
    p.scroll_icons(-1, 4);
    assert_eq!(p.scroll, 0);
    assert_eq!(p.cues, [UiCue::StripEnd]);
    p.cues.clear();
    p.scroll_icons(0, 4);
    assert!(p.cues.is_empty());
}

/// spec: S-0414 claim 11 (surrender), S-0414 (the outcome ends the game)
#[test]
fn surrender_ends_the_game_with_an_end_screen() {
    let mut p = start();
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.surrender();
    step_until(&mut p, 2000, |p| p.outcome.is_some());
    let end = p.end(60).expect("an end screen");
    assert_eq!(end.headline(), "DEFEAT");
    assert!(!p.running());
}
