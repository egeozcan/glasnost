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
        p.cues.contains(&UiCue::BuildStarted { infantry: false }),
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
    // spec: S-0406 claims 6 and 20; S-0701 claim 24 (a refused click: the announcement cue, placement mode kept, no battlefield text)
    let messages = p.messages.len();
    let far = (-40..=40)
        .flat_map(|dy| (-40..=40).map(move |dx| CellXY::new(yard.x + dx, yard.y + dy)))
        .find(|&c| p.preview(c).is_some_and(|(_, ok)| !ok))
        .expect("a refused cell");
    p.click_map(Button::Left, None, far, Modifiers::default());
    assert!(p.cues.contains(&UiCue::PlacementRefused));
    assert_eq!(p.tool, Tool::Place(t));
    assert_eq!(p.messages.len(), messages);
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
    assert!(p.end().is_none());
}

fn start_with(rules: std::sync::Arc<glasnost_sim::rules::SimRules>) -> Play {
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
    Play::new(game, local)
}

fn others(p: &Play) -> Vec<ObjectId> {
    p.world()
        .objects
        .values()
        .filter(|o| o.owner != Some(p.local) && o.on_map && o.kind != Kind::Terrain)
        .map(|o| o.id)
        .collect()
}

/// spec: S-0214 claim 19 (only own selectable mobile objects are taken; enemy objects and ordinary buildings are not); S-0701 claim 17 (only the first addition speaks)
#[test]
fn claim_19_box_takes_own_mobile_objects() {
    let mut p = start();
    let all: Vec<ObjectId> = p.world().objects.keys().copied().collect();
    assert!(!p.select_box(&all, false));
    assert!(!p.selection.is_empty());
    assert_eq!(p.selected.len(), 1);
    for id in &p.selection {
        let o = p.world().obj(*id).unwrap();
        assert_eq!(o.owner, Some(p.local));
        assert!(o.kind.is_mobile());
    }
}

/// spec: S-0214 claim 19 (a box holding only an enemy or an ordinary building clears the old selection and adds nothing)
#[test]
fn claim_19_ineligible_candidates_still_clear() {
    let mut p = start();
    let mine = own(&p, Kind::Vehicle);
    p.selection = mine.clone();
    let mut theirs = others(&p);
    theirs.extend(own(&p, Kind::Building));
    assert!(!theirs.is_empty());
    assert!(!p.select_box(&theirs, false));
    assert!(p.selection.is_empty());
}

/// spec: S-0701 claim 19 and scenario D4 (no command sound for an empty selection; one per press for a nonempty one, whether or not anything was ordered)
#[test]
fn key_commands_sound_once_per_press_for_a_nonempty_selection() {
    let mut p = start();
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.key(KeyCommand::Stop);
    assert!(p.keys.is_empty());
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    let cell = p.world().obj(mcv).unwrap().pos.cell();
    p.click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    p.key(KeyCommand::Stop);
    p.key(KeyCommand::Guard);
    assert_eq!(p.keys, [KeyCommand::Stop, KeyCommand::Guard]);
}

/// spec: S-0214 claim 19 (with force-select eligible objects are added, not toggled: a selected one inside stays selected, and ineligible candidates clear nothing)
#[test]
fn claim_19_force_select_adds() {
    let mut p = start();
    let units: Vec<ObjectId> = p
        .world()
        .objects
        .values()
        .filter(|o| o.owner == Some(p.local) && o.on_map && o.kind.is_mobile())
        .map(|o| o.id)
        .collect();
    let mut inside = units.clone();
    inside.extend(others(&p));
    p.selection = units.clone();
    assert!(!p.select_box(&inside, true));
    assert_eq!(p.selection, units);
    p.selection.clear();
    assert!(!p.select_box(&inside, true));
    assert_eq!(p.selection, units);
}

/// spec: S-0214 claim 19 and Edge cases (an unmodified empty box falls through to the release click under quirks.empty_box_release_click; with force-select it is inert)
#[test]
fn claim_19_empty_box_release_click() {
    let mut p = start();
    let mine = own(&p, Kind::Vehicle);
    p.selection = mine.clone();
    assert!(p.select_box(&[], false), "continues as a click");
    assert_eq!(p.selection, mine);
    assert!(!p.select_box(&[], true), "no fallback with force-select");
    assert_eq!(p.selection, mine);
}

/// spec: S-0214 Edge cases (quirks.empty_box_release_click off: an unmodified empty box clears the selection and gives no order)
#[test]
fn claim_19_empty_box_switch_off() {
    let mut rs = fixture::ruleset();
    rs.globals.quirks.empty_box_release_click = false;
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(&rs, "synthetic-skirmish"));
    let mut p = start_with(rules);
    p.selection = own(&p, Kind::Vehicle);
    assert!(!p.select_box(&[], false));
    assert!(p.selection.is_empty());
    assert!(p.game.replay().orders.is_empty());
}

/// spec: S-0411 claim 18 and S-0701 claim 24 (all icons fit: the page buttons are disabled, no request and no sound)
#[test]
fn page_requests_on_a_fitting_strip_are_silent() {
    let mut p = start();
    p.cues.clear();
    p.scroll_icons(-1, 4);
    p.scroll_icons(1, 4);
    assert_eq!(p.scroll, 0);
    assert!(p.cues.is_empty());
}

/// spec: S-0411 claim 18 and S-0701 claim 24 (an overflowing strip: a request at its requested end gives the refusal sound; an accepted one moves a page, clamped at the end, with the tab-scroll sound; the wheel moves one row silently)
#[test]
fn page_requests_and_wheel_on_an_overflowing_strip() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    let cell = p.world().obj(mcv).unwrap().pos.cell();
    p.click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.key(KeyCommand::Deploy);
    step_until(&mut p, 400, |p| {
        !p.world().houses[usize::from(p.local.0)].econ.tech.icons[0].is_empty()
    });
    p.tab_click(0);
    // Five icons on the shown tab, one row visible: N = 5, R = 1, E = 2.
    let icons = &mut p.game.world.houses[usize::from(p.local.0)].econ.tech.icons[0];
    let first = icons[0].clone();
    while icons.len() < 5 {
        icons.push(first.clone());
    }
    p.cues.clear();
    p.scroll_icons(-1, 1);
    assert_eq!((p.scroll, p.cues.clone()), (0, vec![UiCue::StripEnd]));
    p.cues.clear();
    p.scroll_icons(1, 1);
    p.scroll_icons(1, 1);
    assert_eq!(p.scroll, 2);
    assert_eq!(p.cues, [UiCue::StripScroll, UiCue::StripScroll]);
    p.cues.clear();
    p.scroll_icons(1, 1);
    assert_eq!((p.scroll, p.cues.clone()), (2, vec![UiCue::StripEnd]));
    // Two rows visible from the top row 2 of 3: partial page accepted, clamped.
    p.scroll = 0;
    p.cues.clear();
    p.scroll_icons(1, 2);
    assert_eq!((p.scroll, p.cues.clone()), (1, vec![UiCue::StripScroll]));
    p.cues.clear();
    p.wheel_icons(-3, 1);
    assert_eq!(p.scroll, 2);
    p.wheel_icons(-1, 1);
    assert_eq!(p.scroll, 2);
    p.wheel_icons(7, 1);
    assert_eq!(p.scroll, 1);
    assert!(p.cues.is_empty(), "the wheel is silent");
}

/// spec: S-0414 claim 11 (surrender), S-0414 (the outcome ends the game)
#[test]
fn surrender_ends_the_game_with_an_end_screen() {
    let mut p = start();
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.surrender();
    step_until(&mut p, 2000, |p| p.outcome.is_some());
    let end = p.end().expect("an end screen");
    assert_eq!(end.headline(), "DEFEAT");
    assert!(!p.running());
    // spec: S-0414 claims 19 and 23 (the displayed scores are fixed once play has stopped)
    let local = p.local;
    assert!(glasnost_sim::econ::stats::of(p.world(), local).shown.is_some());
}

/// spec: S-0414 claim 23 and Formulas (the active clock counts ⌊v/16⌋ − ⌊u/16⌋ per active interval; paused or unfocused time and time after play stops are not counted)
#[test]
fn result_duration_counts_active_clock_units() {
    let mut p = start();
    p.clock(0, true);
    p.clock(959, true);
    assert_eq!(p.clock_units, 59);
    p.clock(2000, false);
    p.clock(5000, true);
    p.clock(5016, true);
    assert_eq!(p.clock_units, 60);
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.surrender();
    step_until(&mut p, 2000, |p| p.outcome.is_some());
    p.clock(9000, true);
    assert_eq!(p.clock_units, 60);
    assert_eq!(p.end().unwrap().duration(), (0, 0, 1));
}

/// spec: S-0213 claim 25 (status lines: deadline = clock count + 480 in 16 ms units, removed when the count is strictly greater; six shared lines, oldest first, the oldest evicted at capacity; a later message appends rather than refreshing; the clock runs on regardless of ticks)
#[test]
fn s0213_claim_25_status_lines() {
    let mut p = start();
    // A line said before the first display update is added at that update.
    p.messages.clear();
    p.say("early");
    p.update_messages(16 * 10_000);
    assert_eq!(p.messages[0].until, 10_480);
    let mut p = start();
    p.messages.clear();
    p.update_messages(16 * 100 + 15);
    assert_eq!(p.status_clock, 100);
    p.say("one");
    assert_eq!(p.messages[0].until, 580);
    p.update_messages(16 * 580);
    assert_eq!(p.messages.len(), 1, "kept at equality");
    p.update_messages(16 * 581 - 1);
    assert_eq!(p.messages.len(), 1, "still clock 580");
    p.say("one");
    assert_eq!(p.messages.len(), 2, "appended, not refreshed");
    p.update_messages(16 * 581);
    assert_eq!(p.messages.len(), 1, "gone once the count is greater");
    assert_eq!(p.messages[0].until, 580 + 480);
    for k in 0..7 {
        p.say(format!("line {k}"));
    }
    let texts: Vec<&str> = p.messages.iter().map(|m| m.text.as_str()).collect();
    assert_eq!(
        texts,
        ["line 1", "line 2", "line 3", "line 4", "line 5", "line 6"]
    );
    // No simulation tick runs; the presentation clock alone expires lines.
    let tick = p.world().tick;
    p.update_messages(16 * (581 + 481));
    assert!(p.messages.is_empty());
    assert_eq!(p.world().tick, tick);
}
