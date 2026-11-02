//! The client model headless: a person's house against an AI house on the
//! synthetic skirmish (`glasnost_ai::fixture`), selecting, moving,
//! deploying, building and placing through the same calls the window
//! makes, with no GPU.

use super::*;
use crate::game::status_font::StatusFont;
use glasnost_ai::fixture;
use glasnost_sim::econ::setup::{self, Player, Settings};
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::Target;

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
        fixture::catalogue(),
        s,
    )
    .expect("the synthetic skirmish loads");
    assert_eq!(game.ai_houses().len(), 1, "only the machine is an AI house");
    let mut p = Play::new(game, Some(local));
    p.status_font = Some(test_font());
    p
}

/// A synthetic game font (S-0014) for status text: printable ASCII mapped,
/// a space 2 pixels wide, `W` 9, the capital X 6 and every other glyph 4;
/// other units are missing.
fn test_font() -> StatusFont {
    test_font_with_pitch(9)
}

/// [`test_font`] with line pitch `pitch`.
fn test_font_with_pitch(pitch: u32) -> StatusFont {
    let mut f = glasnost_fmt::fnt::Font::empty(2, 7, pitch);
    for c in 32u16..=126 {
        let w = match c {
            32 => 2,
            87 => 9,
            88 => 6,
            _ => 4,
        };
        let j = f.push_glyph(w, &["#########", "#", "#"]);
        f.map_unit(c, Some(j));
    }
    StatusFont::new(f).expect("the capital X is mapped")
}

fn own(p: &Play, kind: Kind) -> Vec<ObjectId> {
    p.world()
        .objects
        .values()
        .filter(|o| o.owner == Some(p.me()) && o.kind == kind && o.on_map)
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

/// spec: S-0214 claim 12 (a click selects), claim 26 (a left release
/// selects or orders, a right release clears the selection), claim 30 (the
/// move marker), S-0213 claim 10 (deploy),
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
    // A right click clears it, over the unit too.
    p.click_map(Button::Right, Some(mcv), cell, Modifiers::default());
    assert!(p.selection.is_empty());
    assert!(p.game.replay().orders.is_empty());
    p.click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    // A left click on a cell moves it.
    let to = CellXY::new(cell.x, cell.y + 2);
    p.click_map(Button::Left, None, to, Modifiers::default());
    assert_eq!(p.game.replay().orders.len(), 1, "the move is in the replay");
    // spec: S-0701 claim 18 (the click's orders and selection go to the order response); the order selects nothing new
    assert_eq!(p.acks.len(), 1);
    assert_eq!(p.acks[0].0, [mcv]);
    // spec: S-0214 claim 30 (one move-target marker at the clicked cell's centre, for this screen only)
    assert_eq!(
        std::mem::take(&mut p.markers),
        [glasnost_sim::targeting::move_marker_at(p.world(), to)]
    );
    assert_eq!(
        std::mem::take(&mut p.selected),
        [mcv],
        "only the second left click"
    );
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
        !p.world().houses[usize::from(p.me().0)].econ.tech.icons[0].is_empty()
    });
    p.tab_click(0);
    let t = p.world().houses[usize::from(p.me().0)].econ.tech.icons[0][0].t;
    p.cameo_click(0, Button::Left, false);
    assert!(
        p.cues.contains(&UiCue::BuildStarted { infantry: false }),
        "the click started the item: {:?}",
        p.cues
    );
    step_until(&mut p, 4000, |p| icon_ready(p.world(), p.me(), t));
    assert!(p.cues.contains(&UiCue::ConstructionComplete));
    // A left click on the ready icon enters placement mode.
    p.cameo_click(0, Button::Left, false);
    assert_eq!(p.tool, Tool::Place(t));
    // spec: S-0214 claim 28 (a battlefield right click ends placement mode with no order and no deselection; the item stays ready and can be taken up again)
    // spec: S-0411 claim 10 (a battlefield right click without a scroll drag ends placement mode and keeps the completed item ready: no place or cancel order, no refund; clicking the ready icon again re-enters placement mode)
    let orders = p.game.replay().orders.len();
    let selection = p.selection.clone();
    let credits = p.world().houses[usize::from(p.me().0)].credits;
    let yard_cell = p
        .world()
        .obj(own(&p, Kind::Building)[0])
        .unwrap()
        .pos
        .cell();
    p.click_map(Button::Right, None, yard_cell, Modifiers::default());
    assert_eq!(p.tool, Tool::Select);
    assert_eq!(p.game.replay().orders.len(), orders);
    assert_eq!(p.selection, selection);
    p.step();
    assert_eq!(p.world().houses[usize::from(p.me().0)].credits, credits);
    assert!(icon_ready(p.world(), p.me(), t));
    p.cameo_click(0, Button::Left, false);
    assert_eq!(p.tool, Tool::Place(t));
    // Find a cell the preview shows green, then place there.
    let yard = p
        .world()
        .obj(own(&p, Kind::Building)[0])
        .unwrap()
        .pos
        .cell();
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
        fixture::catalogue(),
        s,
    )
    .expect("the synthetic skirmish loads");
    Play::new(game, Some(local))
}

fn others(p: &Play) -> Vec<ObjectId> {
    p.world()
        .objects
        .values()
        .filter(|o| o.owner != Some(p.me()) && o.on_map && o.kind != Kind::Terrain)
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
        assert_eq!(o.owner, Some(p.me()));
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

/// spec: S-0214 claim 28 (a right click in sell or repair mode ends the
/// mode instead of selling or repairing the building under the pointer, and
/// does not also clear the selection); claim 26 (in ordinary mode the right
/// click clears the selection and gives no order)
#[test]
fn right_click_cancels_a_mode_before_deselecting() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    let cell = p.world().obj(mcv).unwrap().pos.cell();
    p.click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.key(KeyCommand::Deploy);
    step_until(&mut p, 400, |p| !own(p, Kind::Building).is_empty());
    let yard = own(&p, Kind::Building)[0];
    let at = p.world().obj(yard).unwrap().pos.cell();
    p.selection = vec![yard];
    for m in [Mode::Sell, Mode::Repair] {
        p.mode_click(m);
        assert_eq!(p.tool, Tool::Mode(m));
        let orders = p.game.replay().orders.len();
        p.click_map(Button::Right, Some(yard), at, Modifiers::default());
        assert_eq!(p.tool, Tool::Select, "the mode ends");
        assert_eq!(
            p.game.replay().orders.len(),
            orders,
            "no sell or repair order"
        );
        assert_eq!(p.selection, [yard], "no ordinary deselection on mode exit");
    }
    let orders = p.game.replay().orders.len();
    p.click_map(Button::Right, Some(yard), at, Modifiers::default());
    assert!(p.selection.is_empty());
    assert_eq!(p.game.replay().orders.len(), orders);
}

/// spec: S-0411 claim 19 (Mode buttons: a click on the sell or repair button while its mode is
/// active ends that mode, as a click while inactive enters it, S-0214 claim 13; Mode lighting:
/// our engine shows frame 1 in every frame drawn while the mode is active, however it was
/// entered, and frame 0 otherwise)
#[test]
fn s0411_claim19_a_mode_buttons_click_ends_its_active_mode() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    let cell = p.world().obj(mcv).unwrap().pos.cell();
    p.click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.key(KeyCommand::Deploy);
    step_until(&mut p, 400, |p| !own(p, Kind::Building).is_empty());
    let frame = |p: &Play| {
        let m = p.hud_model(&|_| String::new(), &|_| None);
        [Mode::Sell, Mode::Repair].map(|x| hud::mode_frame(m.mode == Some(x)))
    };
    assert_eq!(frame(&p), [0, 0]);
    p.mode_click(Mode::Sell);
    assert_eq!(p.tool, Tool::Mode(Mode::Sell));
    assert_eq!(frame(&p), [1, 0]);
    p.mode_click(Mode::Repair);
    assert_eq!(
        p.tool,
        Tool::Mode(Mode::Repair),
        "the other mode replaces it"
    );
    assert_eq!(frame(&p), [0, 1]);
    p.mode_click(Mode::Repair);
    assert_eq!(
        p.tool,
        Tool::Select,
        "a click on the active mode's button ends it"
    );
    assert_eq!(frame(&p), [0, 0]);
    // A mode entered another way shows frame 1 too.
    p.tool = Tool::Mode(Mode::Sell);
    assert_eq!(frame(&p), [1, 0]);
    p.mode_click(Mode::Sell);
    assert_eq!(p.tool, Tool::Select);
    assert_eq!(frame(&p), [0, 0]);
}

/// spec: S-0214 claim 19 (with force-select eligible objects are added, not toggled: a selected one inside stays selected, and ineligible candidates clear nothing)
#[test]
fn claim_19_force_select_adds() {
    let mut p = start();
    let units: Vec<ObjectId> = p
        .world()
        .objects
        .values()
        .filter(|o| o.owner == Some(p.me()) && o.on_map && o.kind.is_mobile())
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

/// Two GUNR (armed) and two MINR (unarmed) of the local house beside its
/// base unit, in creation order GUNR, MINR, GUNR, MINR, then the base unit
/// and these as the own mobiles.
fn mixed_mobiles(p: &mut Play) -> Vec<ObjectId> {
    let mut mobiles = own(p, Kind::Vehicle);
    let local = p.me();
    let w = &mut p.game.world;
    let base = w.obj(mobiles[0]).unwrap().pos.cell();
    for (i, n) in ["GUNR", "MINR", "GUNR", "MINR"].into_iter().enumerate() {
        let t = w.rules().index_of(Kind::Vehicle, n).expect("fixture type");
        let id = glasnost_sim::objects::create(w, Kind::Vehicle, t, Some(local));
        let c = CellXY::new(base.x + 2 + i as i32, base.y + 3);
        let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
        glasnost_sim::objects::place(w, id, glasnost_sim::world::Pos::centre(c, z), 0);
        mobiles.push(id);
    }
    mobiles
}

/// The selection claim 12 builds from `eligible` added one by one in that
/// order: positive-primary-damage objects in reverse order at the front,
/// the others in order at the end.
fn claim_12_order(p: &Play, eligible: &[ObjectId]) -> Vec<ObjectId> {
    let w = p.world();
    let mut front: Vec<ObjectId> = eligible
        .iter()
        .copied()
        .filter(|&id| targeting::positive_primary(w, id))
        .collect();
    front.reverse();
    front.extend(
        eligible
            .iter()
            .copied()
            .filter(|&id| !targeting::positive_primary(w, id)),
    );
    front
}

/// spec: S-0214 claim 29 (an ordinary box adds its eligible candidates in candidate order, ineligible ones skipped, through claim 12's front and end rule, whichever order the candidates come in; S-0701 claim 17's response belongs to the earliest successful addition in candidate order)
#[test]
fn claim_29_box_addition_order() {
    let mut p = start();
    let mobiles = mixed_mobiles(&mut p);
    let armed = |id: &ObjectId| targeting::positive_primary(p.world(), *id);
    assert!(mobiles.iter().any(armed) && !mobiles.iter().all(armed));
    let theirs = others(&p);
    for reversed in [false, true] {
        let mut candidates = mobiles.clone();
        if reversed {
            candidates.reverse();
        }
        // An ineligible enemy candidate between the own ones is skipped.
        candidates.splice(1..1, theirs.iter().copied().take(1));
        p.selection.clear();
        p.selected.clear();
        assert!(!p.select_box(&candidates, false));
        let eligible: Vec<ObjectId> = candidates
            .iter()
            .copied()
            .filter(|id| mobiles.contains(id))
            .collect();
        assert_eq!(p.selection, claim_12_order(&p, &eligible));
        assert_eq!(p.selected, [eligible[0]], "the earliest addition speaks");
    }
}

/// spec: S-0214 claim 29 (in an additive box an already-selected object is neither added again nor moved, and does not speak; the others are added in candidate order)
#[test]
fn claim_29_additive_box_keeps_the_selected_in_place() {
    let mut p = start();
    let mobiles = mixed_mobiles(&mut p);
    let kept = mobiles[mobiles.len() - 1];
    p.selection = vec![kept];
    p.selected.clear();
    assert!(!p.select_box(&mobiles, true));
    let added: Vec<ObjectId> = mobiles.iter().copied().filter(|&id| id != kept).collect();
    let w = p.world();
    let mut expected = vec![kept];
    for &id in &added {
        if targeting::positive_primary(w, id) {
            expected.insert(0, id);
        } else {
            expected.push(id);
        }
    }
    assert_eq!(p.selection, expected);
    assert_eq!(p.selected, [added[0]]);
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
    let rules = std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &rs,
        "synthetic-skirmish",
    ));
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
        !p.world().houses[usize::from(p.me().0)].econ.tech.icons[0].is_empty()
    });
    p.tab_click(0);
    // Five icons on the shown tab, one row visible: N = 5, R = 1, E = 2.
    let icons = &mut p.game.world.houses[usize::from(p.local.unwrap().0)]
        .econ
        .tech
        .icons[0];
    let first = icons[0];
    while icons.len() < 5 {
        icons.push(first);
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
    let local = p.me();
    assert!(
        glasnost_sim::econ::stats::of(p.world(), local)
            .shown
            .is_some()
    );
}

/// spec: S-0414 claim 23 and Formulas (the active clock counts ⌊v/16⌋ − ⌊u/16⌋ per active interval; an inactive interval, the skirmish's open options menu, and time after play stops are not counted)
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
    let texts: Vec<String> = p.messages.iter().map(|m| m.text()).collect();
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

/// spec: S-0213 claim 25 (Line splitting: the budget is the view width minus 22 after game start and minus 16 after a change of the view's dimensions, which clears the lines; every stored line, explicit breaks included, takes a place and its own 480-unit lifetime) and Edge cases (Unfittable status text)
#[test]
fn s0213_claim_25_split_lines_share_capacity_and_lifetime() {
    let mut p = start();
    p.messages.clear();
    p.status_view(500, false);
    assert_eq!(p.status_width, 478);
    p.update_messages(16 * 100);
    p.say("one\ntwo\r\nthree");
    let texts: Vec<String> = p.messages.iter().map(|m| m.text()).collect();
    assert_eq!(texts, ["one\n", "two\r", "three"]);
    assert!(p.messages.iter().all(|m| m.until == 580));
    // Lines said at a later count expire later.
    p.update_messages(16 * 101);
    p.say("four\nfive\nsix\nseven");
    let texts: Vec<String> = p.messages.iter().map(|m| m.text()).collect();
    assert_eq!(
        texts,
        ["two\r", "three", "four\n", "five\n", "six\n", "seven"]
    );
    p.update_messages(16 * 581);
    let texts: Vec<String> = p.messages.iter().map(|m| m.text()).collect();
    assert_eq!(texts, ["four\n", "five\n", "six\n", "seven"]);
    // A long word wraps inside itself at the budget; each piece is a line.
    p.messages.clear();
    let word = "W".repeat(200);
    p.say(word.clone());
    assert!(p.messages.len() > 1 && p.messages.len() <= 6);
    let joined: String = p.messages.iter().map(|m| m.text()).collect();
    assert_eq!(joined, word);
    // A change of the view's dimensions clears the lines and sets W − 16.
    p.status_view(300, true);
    assert!(p.messages.is_empty());
    assert_eq!(p.status_width, 284);
    // An unfittable remainder is left out with a warning.
    p.status_view(22, false);
    p.say("x");
    assert!(p.messages.is_empty());
    assert_eq!(p.status_problems.len(), 1);
}

/// spec: S-0213 claim 25 (a missing unit measures as the generated fallback, the capital X's width plus one pixel; a mapped unit by its own width; the zero unit ends the text)
#[test]
fn s0213_claim_25_game_font_widths() {
    let mut p = start();
    p.messages.clear();
    // Budget 7: one missing unit (6 + 1) fits exactly, two do not.
    p.status_view(22 + 7, false);
    p.say("\u{e9}\u{e9}");
    let texts: Vec<String> = p.messages.iter().map(|m| m.text()).collect();
    assert_eq!(texts, ["\u{e9}", "\u{e9}"]);
    p.messages.clear();
    // "ab" is 5 + 5 = 10 pixels and fits; the zero unit ends the text.
    p.status_view(22 + 10, false);
    p.say("ab\0cd");
    let texts: Vec<String> = p.messages.iter().map(|m| m.text()).collect();
    assert_eq!(texts, ["ab"]);
}

/// spec: S-0213 claim 25, Vertical metrics, and Scenario O21 d (changing the font's line pitch,
/// zero, below, equal to or above the bitmap height, leaves splitting, wrapping, capacity and
/// lifetimes unchanged)
#[test]
fn s0213_claim_25_pitch_does_not_change_lines() {
    let said = |pitch: u32| {
        let mut p = start();
        p.status_font = Some(test_font_with_pitch(pitch));
        p.messages.clear();
        p.status_view(22 + 60, false);
        p.update_messages(16 * 100);
        p.say(
            "one
two",
        );
        p.say("a long line of words that wraps");
        p.say("WWWWWWWWWWWWWWWW");
        p.messages.clone()
    };
    let reference = said(9);
    assert_eq!(reference.len(), 6, "capacity reached");
    for pitch in [0, 3, 7, 40] {
        assert_eq!(said(pitch), reference, "pitch {pitch}");
    }
}

/// spec: S-0213 Import mapping (without a loaded game font there is no status text; the session reports the missing font)
#[test]
fn s0213_claim_25_no_font_no_status_text() {
    let mut p = start();
    p.status_font = None;
    p.messages.clear();
    p.say("hello");
    assert!(p.messages.is_empty());
    assert!(p.status_problems.is_empty());
}

/// spec: S-0411 claim 17 (starting, holding, resuming and cancelling production and its completion add no battlefield status line; the sidebar labels are not status messages)
#[test]
fn s0411_claim_17_production_adds_no_status_line() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    let cell = p.world().obj(mcv).unwrap().pos.cell();
    p.click_map(Button::Left, Some(mcv), cell, Modifiers::default());
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    let to = CellXY::new(cell.x, cell.y + 2);
    p.click_map(Button::Left, None, to, Modifiers::default());
    step_until(&mut p, 400, |p| {
        p.world().obj(mcv).is_some_and(|o| o.pos.cell() == to)
    });
    p.key(KeyCommand::Deploy);
    step_until(&mut p, 400, |p| !own(p, Kind::Building).is_empty());
    step_until(&mut p, 50, |p| {
        !p.world().houses[usize::from(p.me().0)].econ.tech.icons[0].is_empty()
    });
    p.tab_click(0);
    let t = p.world().houses[usize::from(p.me().0)].econ.tech.icons[0][0].t;
    p.messages.clear();
    for b in [Button::Left, Button::Right, Button::Left] {
        p.cameo_click(0, b, false);
        p.step();
    }
    step_until(&mut p, 4000, |p| icon_ready(p.world(), p.me(), t));
    p.cameo_click(0, Button::Right, false);
    p.step();
    for cue in [
        UiCue::BuildStarted { infantry: false },
        UiCue::BuildHeld,
        UiCue::BuildResumed { infantry: false },
        UiCue::ConstructionComplete,
        UiCue::BuildCancelled,
    ] {
        assert!(p.cues.contains(&cue), "{cue:?} in {:?}", p.cues);
    }
    assert!(p.messages.is_empty(), "{:?}", p.messages);
}

/// Diagnostic: a factory selected, a left click on a cell.
// spec: S-0214 claim 26 and row BC5 (a left release on a cell with an own rally-capable factory selected gives the guard-spot order)
// spec: S-0213 claim 20 (a) (the factory's guard spot is its rally point)
// spec: S-0417 claim 8 (a vehicle factory's run-away reply sends the released vehicle to the rally point of a human house)
// spec: S-0701 claim 28 (rally point set: the local player's announcement)
#[test]
fn a_left_click_sets_a_rally_point_that_new_vehicles_drive_to() {
    let mut r = fixture::ruleset();
    // spec: S-0417 Import mapping (`WeaponsFactory`: the vehicle factory kind)
    r.sim.buildings.get_mut("FACT").unwrap().weapons_factory = Some(true);
    let mut p = start_with(std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &r, "x",
    )));
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    let c = p.world().obj(mcv).unwrap().pos.cell();
    let at = CellXY::new(c.x - 6, c.y - 6);
    let fact = testkit::spawn(
        &mut p.game.world,
        Kind::Building,
        "FACT",
        p.local.unwrap().0,
        at,
        0,
    );
    p.click_map(Button::Left, Some(fact), at, Modifiers::default());
    assert_eq!(p.selection, vec![fact]);
    p.cues.clear();
    let to = CellXY::new(at.x + 5, at.y + 8);
    p.click_map(Button::Left, None, to, Modifiers::default());
    assert_eq!(p.selection, vec![fact], "the order keeps the selection");
    assert!(p.cues.contains(&UiCue::RallySet));
    p.step();
    assert_eq!(
        p.world().obj(fact).unwrap().guard_spot,
        Some(Target::Cell(to))
    );
    let gi = p.world().rules().index_of(Kind::Vehicle, "GUNR").unwrap();
    p.game.world.houses[usize::from(p.local.unwrap().0)].credits = 100_000;
    let what = glasnost_sim::econ::TypeRef {
        kind: Kind::Vehicle,
        index: gi as u32,
    };
    assert_eq!(p.issue(vec![Order::Build { what }]), 1);
    // The new vehicle reaches the rally point.
    step_until(&mut p, 3000, |p| {
        p.world().objects.values().any(|o| {
            o.type_index == gi && o.kind == Kind::Vehicle && o.on_map && o.pos.cell() == to
        })
    });
}

/// The synthetic skirmish with the NUKE type's kind word set to `kind`,
/// the types `extra` (name, kind word) added after it, and
/// `quirks.superweapon_kind_as_list_position` set to `list_position`.
fn sw_start(kind: &str, extra: &[(&str, &str)], list_position: bool) -> Play {
    let mut r = fixture::ruleset();
    r.sim.superweapons.get_mut("NUKE").unwrap().kind = Some(kind.into());
    for (name, k) in extra {
        r.sim.superweapons.insert(
            (*name).into(),
            glasnost_rules::sim_econ::SuperweaponType {
                kind: Some((*k).into()),
                ..Default::default()
            },
        );
        r.registries
            .get_mut("superweapons")
            .unwrap()
            .push((*name).into());
    }
    r.globals.quirks.superweapon_kind_as_list_position = list_position;
    start_with(std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &r,
        "synthetic-skirmish",
    )))
}

/// Gives the local house a powered missile silo and charges its record
/// fully; returns the silo.
fn ready_silo(p: &mut Play) -> ObjectId {
    let silo = granted_silo(p);
    charge::set_charge(&mut p.game.world, p.local.unwrap(), 0, 100);
    step_until(p, 10, |p| charge::offers_targeting(p.world(), p.me(), 0));
    silo
}

/// Gives the local house a powered missile silo whose record is granted
/// and still charging; returns the silo.
fn granted_silo(p: &mut Play) -> ObjectId {
    step_until(p, 10, |p| p.world().tick >= 4);
    let mcv = *own(p, Kind::Vehicle).first().expect("a base unit");
    let c = p.world().obj(mcv).unwrap().pos.cell();
    let local = p.me().0;
    testkit::spawn(
        &mut p.game.world,
        Kind::Building,
        "POWR",
        local,
        CellXY::new(c.x + 4, c.y - 6),
        0,
    );
    let silo = testkit::spawn(
        &mut p.game.world,
        Kind::Building,
        "SILO",
        local,
        CellXY::new(c.x - 6, c.y - 6),
        0,
    );
    step_until(p, 10, |p| {
        charge::get(p.world(), p.me(), 0).is_some_and(|r| r.granted)
    });
    silo
}

fn fired(p: &Play) -> Vec<Order> {
    p.game
        .replay()
        .orders
        .iter()
        .filter(|(_, h, o)| *h == p.me() && matches!(o, Order::FireSuperweapon { .. }))
        .map(|(_, _, o)| o.clone())
        .collect()
}

/// spec: S-0512 claim 16 (a ready icon arms its type); S-0513 claims 1 and 2 (a click fires it at the cell; the order names the selected type); S-0004 claim 11 (superweapons in the 2nd tab)
#[test]
fn s0512_claim_16_icon_arms_and_a_click_fires() {
    let mut p = sw_start("MultiMissile", &[], true);
    ready_silo(&mut p);
    let entries = hud::entries(p.world(), p.me(), hud::SW_TAB);
    let n = entries
        .iter()
        .position(|e| *e == hud::Entry::Superweapon(0))
        .expect("the silo's superweapon in the 2nd tab");
    p.tab = hud::SW_TAB;
    p.selection.clear();
    p.cameo_click(n, Button::Left, false);
    let armed = Tool::Superweapon {
        sw: 0,
        from_source: false,
    };
    assert_eq!(p.tool, armed);
    // spec: S-0410 claim 12 (a nuclear strike has an explicit radar offer)
    assert_eq!(p.radar_aims(), Some(0));
    let at = CellXY::new(10, 10);
    p.click_map(Button::Left, None, at, Modifiers::default());
    assert_eq!(p.tool, Tool::Select);
    assert_eq!(fired(&p), vec![Order::FireSuperweapon { sw: 0, cell: at }]);
}

/// spec: S-0512 claim 23 (an otherwise fireable icon with no aim action issues that type's fire order at cell (0, 0), without entering aiming; a recognised nonzero action enters aiming)
#[test]
fn a_superweapon_icon_without_an_aim_action_fires_at_cell_zero() {
    let mut r = fixture::ruleset();
    r.sim.superweapons.get_mut("NUKE").unwrap().action = None;
    let mut p = start_with(std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &r,
        "synthetic-skirmish",
    )));
    ready_silo(&mut p);
    let entries = hud::entries(p.world(), p.me(), hud::SW_TAB);
    let n = entries
        .iter()
        .position(|e| *e == hud::Entry::Superweapon(0))
        .expect("the silo's superweapon in the 2nd tab");
    p.tab = hud::SW_TAB;
    // spec: S-0512 claim 25 (an icon with no aim action fires at once and changes nothing else: the selection stays)
    p.selection = own(&p, Kind::Vehicle);
    let kept = p.selection.clone();
    p.cues.clear();
    p.cameo_click(n, Button::Left, false);
    assert_eq!(p.selection, kept);
    assert_eq!(p.tool, Tool::Select, "no aiming");
    // spec: S-0512 claim 25 (the no-aim input requests the click sound but no select-target announcement)
    assert_eq!(p.cues, vec![UiCue::SuperweaponClick]);
    assert_eq!(
        fired(&p),
        vec![Order::FireSuperweapon {
            sw: 0,
            cell: CellXY::new(0, 0)
        }]
    );
}

/// spec: S-0512 claim 25 (every left click on a superweapon icon requests the build click first, whatever the icon's state; the arming input then requests the select-target announcement; an icon that does not offer targeting changes nothing but the click sound request)
#[test]
fn s0512_claim_25_icon_clicks_request_the_click_then_the_announcement() {
    let mut p = sw_start("MultiMissile", &[], true);
    granted_silo(&mut p);
    let entries = hud::entries(p.world(), p.me(), hud::SW_TAB);
    let n = entries
        .iter()
        .position(|e| *e == hud::Entry::Superweapon(0))
        .expect("the silo's superweapon in the 2nd tab");
    p.tab = hud::SW_TAB;
    // A charging icon: the click sound only, no arming, no selection change.
    assert!(!charge::offers_targeting(p.world(), p.me(), 0));
    p.selection = own(&p, Kind::Vehicle);
    let kept = p.selection.clone();
    p.cues.clear();
    p.cameo_click(n, Button::Left, false);
    assert_eq!(p.cues, vec![UiCue::SuperweaponClick]);
    assert_eq!(p.tool, Tool::Select);
    assert_eq!(p.selection, kept);
    // A ready icon with an aim action: the click, then the announcement.
    charge::set_charge(&mut p.game.world, p.local.unwrap(), 0, 100);
    step_until(&mut p, 10, |p| {
        charge::offers_targeting(p.world(), p.me(), 0)
    });
    p.cues.clear();
    p.cameo_click(n, Button::Left, false);
    assert!(matches!(p.tool, Tool::Superweapon { sw: 0, .. }));
    assert_eq!(
        p.cues,
        vec![UiCue::SuperweaponClick, UiCue::SelectTarget],
        "the click sound is the input's first request"
    );
}

/// spec: S-0214 claim 28 and S-0513 claim 1 (a right click cancels the aim with no order)
#[test]
fn s0513_claim_1_right_click_cancels_the_aim() {
    let mut p = sw_start("MultiMissile", &[], true);
    ready_silo(&mut p);
    p.selection.clear();
    p.tool = Tool::Superweapon {
        sw: 0,
        from_source: false,
    };
    p.click_map(
        Button::Right,
        None,
        CellXY::new(10, 10),
        Modifiers::default(),
    );
    assert_eq!(p.tool, Tool::Select);
    assert!(fired(&p).is_empty());
}

/// spec: S-0513 claim 1 (a force shield is offered only over an own or allied building; elsewhere a click sends no order and keeps the charge)
#[test]
fn s0513_claim_1_force_shield_only_over_own_buildings() {
    let mut p = sw_start("ForceShield", &[], true);
    let silo = ready_silo(&mut p);
    p.selection.clear();
    let armed = Tool::Superweapon {
        sw: 0,
        from_source: false,
    };
    p.tool = armed;
    // spec: S-0410 claim 12 (under the switch the list position 0, the nuclear strike's row, picks the offer, not the kind)
    assert_eq!(p.radar_aims(), Some(0));
    p.click_map(
        Button::Left,
        None,
        CellXY::new(10, 10),
        Modifiers::default(),
    );
    assert_eq!(p.tool, armed, "the aim stays");
    assert!(fired(&p).is_empty(), "no order");
    let at = p.world().obj(silo).unwrap().pos.cell();
    p.click_map(Button::Left, Some(silo), at, Modifiers::default());
    assert_eq!(p.tool, Tool::Select);
    assert_eq!(fired(&p), vec![Order::FireSuperweapon { sw: 0, cell: at }]);
}

/// spec: S-0513 claim 1 (a non-empty unit selection takes precedence over the aim)
#[test]
fn s0513_claim_1_a_selection_takes_precedence() {
    let mut p = sw_start("MultiMissile", &[], true);
    ready_silo(&mut p);
    p.selection = own(&p, Kind::Vehicle);
    assert!(!p.selection.is_empty());
    p.tool = Tool::Superweapon {
        sw: 0,
        from_source: false,
    };
    let selected = p.selection.clone();
    p.click_map(
        Button::Left,
        None,
        CellXY::new(10, 10),
        Modifiers::default(),
    );
    assert!(fired(&p).is_empty());
    assert_eq!(p.selection, selected, "the units stay selected");
    assert!(
        matches!(p.tool, Tool::Superweapon { sw: 0, .. }),
        "the aim stays"
    );
}

/// spec: S-0512 claim 25 (arming from an icon with an aim action empties
/// the selection in the same input and ends sell, repair and power mode), so
/// the next click fires (S-0513 claim 1).
#[test]
fn arming_from_the_icon_clears_the_selection_and_the_next_click_fires() {
    let mut p = sw_start("MultiMissile", &[], true);
    ready_silo(&mut p);
    let entries = hud::entries(p.world(), p.me(), hud::SW_TAB);
    let n = entries
        .iter()
        .position(|e| *e == hud::Entry::Superweapon(0))
        .expect("the silo's superweapon in the 2nd tab");
    p.tab = hud::SW_TAB;
    p.selection = own(&p, Kind::Vehicle);
    assert!(!p.selection.is_empty());
    p.tool = Tool::Mode(Mode::Sell);
    p.cameo_click(n, Button::Left, false);
    assert!(matches!(p.tool, Tool::Superweapon { sw: 0, .. }));
    assert!(p.selection.is_empty(), "arming deselects");
    assert!(p.sel_reset, "a clearing resets the type-select reach");
    let at = CellXY::new(10, 10);
    p.click_map(Button::Left, None, at, Modifiers::default());
    assert_eq!(fired(&p), vec![Order::FireSuperweapon { sw: 0, cell: at }]);
}

/// Placing an armed building offers its weapon's range in leptons for the
/// indicator; an unarmed one and other tools offer none (our design, M8 A4
/// round 5 note 34). spec: S-0307 claim 1 (R₀ in leptons)
#[test]
fn placing_an_armed_building_offers_its_weapon_range() {
    let mut r = fixture::ruleset();
    r.sim.buildings.get_mut("PILL").unwrap().primary = Some("Gun".into());
    let mut p = start_with(std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &r,
        "synthetic-skirmish",
    )));
    let rules = p.world().rules_arc();
    let place = |name: &str| {
        Tool::Place(glasnost_sim::econ::TypeRef {
            kind: Kind::Building,
            index: rules.index_of(Kind::Building, name).unwrap() as u32,
        })
    };
    let gun = rules.combat.weapon_index["Gun"];
    let want = rules.combat.weapon(gun).range;
    assert!(want > 0);
    p.tool = place("PILL");
    assert_eq!(p.placement_range(), Some(want));
    p.tool = place("POWR");
    assert_eq!(p.placement_range(), None);
    p.tool = Tool::Select;
    assert_eq!(p.placement_range(), None);
}

fn source_pick(p: &mut Play, house: HouseId) {
    p.effects = vec![Effect::Superweapon {
        event: SwEvent::SourceMarker,
        house: Some(house),
        sw: None,
        pos: None,
    }];
}

const FOUR_MORE: [(&str, &str); 4] = [
    ("SWA", "ChronoWarp"),
    ("SWB", "LightningStorm"),
    ("SWC", "IronCurtain"),
    ("SWD", "MultiMissile"),
];

/// spec: S-0513 claim 33 (a source pick arms the type at list position 4, the warp kind's number)
#[test]
fn s0513_claim_33_source_pick_arms_position_4() {
    let mut p = sw_start("Chronosphere", &FOUR_MORE, true);
    let local = p.me();
    source_pick(&mut p, local);
    assert_eq!(p.source_aim(), Some(4));
    source_pick(&mut p, HouseId(1 - local.0));
    assert_eq!(p.source_aim(), None, "another house's pick arms nothing");
}

/// spec: S-0513 Edge cases (quirks.superweapon_kind_as_list_position off: a source pick arms the warp kind's type)
#[test]
fn s0513_claim_33_source_pick_switch_off() {
    let mut p = sw_start("Chronosphere", &FOUR_MORE, false);
    let local = p.me();
    source_pick(&mut p, local);
    assert_eq!(p.source_aim(), Some(1));
}

/// spec: S-0509 claim 15 (when the new owner of a changed building is the local player's house and the building is a construction yard, the local player's building placement in progress is cancelled; another building, another house or no placement does nothing)
#[test]
fn a_gained_construction_yard_cancels_the_placement_in_progress() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.click_map(
        Button::Left,
        Some(mcv),
        p.world().obj(mcv).unwrap().pos.cell(),
        Modifiers::default(),
    );
    p.key(KeyCommand::Deploy);
    step_until(&mut p, 400, |p| !own(p, Kind::Building).is_empty());
    step_until(&mut p, 50, |p| {
        !p.world().houses[usize::from(p.me().0)].econ.tech.icons[0].is_empty()
    });
    p.tab_click(0);
    let t = p.world().houses[usize::from(p.me().0)].econ.tech.icons[0][0].t;
    p.cameo_click(0, Button::Left, false);
    step_until(&mut p, 4000, |p| icon_ready(p.world(), p.me(), t));
    p.cameo_click(0, Button::Left, false);
    assert_eq!(p.tool, Tool::Place(t));
    // The fixture's yard carries no construction-yard flag: set it (S-0214 Terms).
    let mut rules = (*p.game.world.rules_arc()).clone();
    let yard = rules.index_of(Kind::Building, "YARD").unwrap() as usize;
    rules.buildings[yard].raw.construction_yard = Some(true);
    p.game.world.rules = Some(std::sync::Arc::new(rules));
    let captured = |p: &Play, kind: &str, new: u8| Effect::Captured {
        building: ObjectId(1),
        of: glasnost_sim::econ::TypeRef {
            kind: Kind::Building,
            index: p.world().rules().index_of(Kind::Building, kind).unwrap(),
        },
        old: None,
        new: Some(glasnost_sim::world::HouseId(new)),
        pos: glasnost_sim::world::Pos::default(),
        announce: true,
        after_slots: true,
    };
    // A construction yard gained by another house, or another building gained by the local one: nothing.
    p.effects = vec![captured(&p, "YARD", 1 - p.me().0)];
    p.captured_yard_cancels_placement();
    assert_eq!(p.tool, Tool::Place(t));
    p.effects = vec![captured(&p, "POWR", p.me().0)];
    p.captured_yard_cancels_placement();
    assert_eq!(p.tool, Tool::Place(t));
    // A construction yard gained by the local house cancels it.
    p.effects = vec![captured(&p, "YARD", p.me().0)];
    p.captured_yard_cancels_placement();
    assert_eq!(p.tool, Tool::Select);
}

/// spec: S-0214 Terms (clicked object; can be selected; a fully cloaked object the viewer neither
/// owns nor senses cannot be clicked) and S-0516 claims 26, 35 and 36 (the lead's note 17 of M8
/// step A4, round 4): a click finds an enemy unit disguised as scenery, which cannot be selected,
/// so the kind tables and the disguise rule decide; a fully cloaked enemy is not found
#[test]
fn a_click_finds_an_enemy_disguised_as_scenery_but_not_a_cloaked_one() {
    use glasnost_sim::cloak::{CloakState, Disguise, DisguiseAs};
    let mut p = start();
    let e = others(&p)
        .into_iter()
        .find(|&id| p.world().obj(id).is_some_and(|o| o.kind == Kind::Vehicle))
        .expect("an enemy vehicle");
    let cell = p.world().obj(e).unwrap().pos.cell();
    assert_eq!(p.target(Some(e), cell), ClickOn::Object(e));
    let tick = p.world().tick;
    p.game.world.obj_mut(e).unwrap().cloak.disguise = Some(Disguise {
        what: Some(DisguiseAs::Terrain(0)),
        house: None,
        since: tick,
    });
    assert!(glasnost_sim::cloak::appears_disguised(
        p.world(),
        e,
        Some(p.me())
    ));
    assert!(!targeting::selectable_by(p.world(), e, Some(p.me())));
    assert_eq!(p.target(Some(e), cell), ClickOn::Object(e), "still clicked");
    p.game.world.obj_mut(e).unwrap().cloak.disguise = None;
    p.game.world.obj_mut(e).unwrap().cloak.state = CloakState::Cloaked;
    assert_eq!(p.target(Some(e), cell), ClickOn::Cell(cell), "cloaked");
}

/// A GUNR of the other player's house beside the local base unit.
fn their_gunr(p: &mut Play) -> ObjectId {
    let other = others(p)
        .into_iter()
        .find_map(|id| p.world().obj(id).and_then(|o| o.owner))
        .expect("another house's object");
    let base = own(p, Kind::Vehicle)[0];
    let w = &mut p.game.world;
    let at = w.obj(base).unwrap().pos.cell();
    let t = w
        .rules()
        .index_of(Kind::Vehicle, "GUNR")
        .expect("fixture type");
    let id = glasnost_sim::objects::create(w, Kind::Vehicle, t, Some(other));
    let c = CellXY::new(at.x + 2, at.y + 5);
    let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
    glasnost_sim::objects::place(w, id, glasnost_sim::world::Pos::centre(c, z), 0);
    id
}

/// spec: S-0214 claim 12 (type selection adds every object of the clicked object's type through the front/end rule, or removes them) and claim 29 (types in first-encounter order, within a type the pool's order, already-selected objects not added again)
#[test]
fn type_expansion_takes_the_seed_types_from_the_pool() {
    let mut p = start();
    let m = mixed_mobiles(&mut p);
    let (g1, n1, g2, n2) = (m[1], m[2], m[3], m[4]);
    let theirs = their_gunr(&mut p);
    let mut pool = m.clone();
    pool.push(theirs);
    p.selection = vec![g1];
    p.expand_types(&[g1], &pool, false);
    assert_eq!(p.selection, claim_12_order(&p, &[g1, g2]));
    // design: the same type of another house is not taken (Q-1987)
    assert!(!p.selection.contains(&theirs));
    // Two types come in seed order; the pool decides the order within one.
    p.selection = vec![n2];
    p.expand_types(&[n2, g2], &[n2, n1, g2, g1], false);
    assert_eq!(p.selection, claim_12_order(&p, &[n2, n1, g2, g1]));
    // A pool without the other GUNR leaves it out (the screen's pool).
    p.selection = vec![g1];
    p.expand_types(&[g1], &[g1, n1], false);
    assert_eq!(p.selection, vec![g1]);
    // Removing takes the type out and keeps the rest's order.
    p.selection = vec![g2, g1, n1, n2];
    p.expand_types(&[g1], &pool, true);
    assert_eq!(p.selection, vec![n1, n2]);
}

/// The selection `sel` grows to by S-0214 claim 12's rule when `add` joins
/// it one by one, skipping objects already in it.
fn added_in_order(p: &Play, sel: &[ObjectId], add: &[ObjectId]) -> Vec<ObjectId> {
    let mut out = sel.to_vec();
    for &id in add {
        if !out.contains(&id) {
            targeting::add_to_selection(p.world(), &mut out, id);
        }
    }
    out
}

/// spec: S-0214 claim 33 (make group: every own object in the group leaves it, then every selected own object joins it and leaves any other group; the selection does not change) and Terms (members are own objects on the map, in creation order)
#[test]
fn claim_33_make_group() {
    let mut p = start();
    let m = mixed_mobiles(&mut p);
    let theirs = their_gunr(&mut p);
    p.selection = vec![m[2], m[1], theirs];
    p.make_group(1);
    assert_eq!(p.group(1), vec![m[1], m[2]], "own only, creation order");
    assert_eq!(p.selection, vec![m[2], m[1], theirs], "the selection stays");
    // Making the group again replaces its members.
    p.selection = vec![m[3]];
    p.make_group(1);
    assert_eq!(p.group(1), vec![m[3]]);
    // An object is in one group at most.
    p.selection = vec![m[1], m[2]];
    p.make_group(2);
    p.selection = vec![m[2]];
    p.make_group(3);
    assert_eq!(p.group(2), vec![m[1]]);
    assert_eq!(p.group(3), vec![m[2]]);
    // Making a group of an empty selection empties it.
    p.selection.clear();
    p.make_group(2);
    assert!(p.group(2).is_empty());
    // spec: S-0214 claim 33 (make group ends no mode)
    p.tool = Tool::Mode(Mode::Sell);
    p.selection = vec![m[1]];
    p.make_group(4);
    assert_eq!(p.tool, Tool::Mode(Mode::Sell));
}

/// spec: S-0214 claim 33 (select group clears the selection, even for an empty group, and adds every member that can be selected in creation order, the first one added giving the only response; add group does so without the clearing, and a selection holding another house's object is replaced; both end sell, repair and power mode) and S-0701 claim 17
#[test]
fn claim_33_select_and_add_group() {
    let mut p = start();
    let m = mixed_mobiles(&mut p);
    let theirs = their_gunr(&mut p);
    p.selection = vec![m[3], m[1]];
    p.make_group(2);
    p.selection = vec![m[4]];
    p.selected.clear();
    p.tool = Tool::Mode(Mode::Repair);
    p.select_group(2);
    assert_eq!(p.selection, added_in_order(&p, &[], &[m[1], m[3]]));
    assert_eq!(p.selected, vec![m[1]], "the first member in creation order");
    assert_eq!(p.tool, Tool::Select, "the mode ends");
    assert!(p.sel_reset, "an addition resets the type-select reach");
    // An empty group leaves the selection empty.
    p.select_group(5);
    assert!(p.selection.is_empty());
    // Add group keeps the selection and adds the members after it.
    p.selection = vec![m[4]];
    p.selected.clear();
    p.tool = Tool::Mode(Mode::Power);
    p.add_group(2);
    assert_eq!(p.selection, added_in_order(&p, &[m[4]], &[m[1], m[3]]));
    assert_eq!(p.selected, vec![m[1]]);
    assert_eq!(p.tool, Tool::Select);
    // Members already selected add nothing and give no response.
    p.selected.clear();
    let before = p.selection.clone();
    p.add_group(2);
    assert_eq!(p.selection, before);
    assert!(p.selected.is_empty());
    // Another house's object in the selection is replaced.
    p.selection = vec![theirs];
    p.add_group(2);
    assert_eq!(p.selection, added_in_order(&p, &[], &[m[1], m[3]]));
    // A member given to another house is no member while it is theirs.
    let other = p.world().obj(theirs).unwrap().owner;
    p.game.world.obj_mut(m[1]).unwrap().owner = other;
    p.select_group(2);
    assert_eq!(p.selection, vec![m[3]]);
}

/// spec: S-0214 claim 33 (centre group: the selection is cleared when its first object is not in the group, then the members are added; the group centre point of F9 is returned) and Edge cases (quirks.group_center_keeps_extra_selection on: other selected objects stay when the first one is a member)
#[test]
fn claim_33_centre_group_keeps_extra_selection() {
    let mut p = start();
    let m = mixed_mobiles(&mut p);
    p.selection = vec![m[1], m[2]];
    p.make_group(6);
    // The first selected object is a member: the extra one stays.
    p.selection = vec![m[1], m[4]];
    let c = p.centre_group(6).expect("a selection");
    assert_eq!(p.selection, added_in_order(&p, &[m[1], m[4]], &[m[2]]));
    assert_eq!(Some(c), targeting::group_centre(p.world(), &p.selection));
    // The first selected object is no member: the selection is cleared.
    p.selection = vec![m[4], m[1]];
    p.centre_group(6);
    assert_eq!(p.selection, added_in_order(&p, &[], &[m[1], m[2]]));
    // An empty group: no centre and an empty selection.
    p.selection = vec![m[4]];
    assert_eq!(p.centre_group(7), None);
    assert!(p.selection.is_empty());
}

fn start_with_quirks(f: impl FnOnce(&mut glasnost_rules::quirks::Quirks)) -> Play {
    let mut rs = fixture::ruleset();
    f(&mut rs.globals.quirks);
    start_with(std::sync::Arc::new(glasnost_sim::rules::SimRules::new(
        &rs,
        "synthetic-skirmish",
    )))
}

/// spec: S-0214 Edge cases (quirks.group_center_keeps_extra_selection off: centre group replaces the selection by the group's members)
#[test]
fn claim_33_centre_group_switch_off() {
    let mut p = start_with_quirks(|q| q.group_center_keeps_extra_selection = false);
    let m = mixed_mobiles(&mut p);
    p.selection = vec![m[1], m[2]];
    p.make_group(6);
    p.selection = vec![m[1], m[4]];
    p.centre_group(6);
    assert_eq!(p.selection, added_in_order(&p, &[], &[m[1], m[2]]));
}

/// spec: S-0214 Edge cases (quirks.group_kept_on_owner_change on: the number stays with an object given away, which is a member again when it comes back; off: make group by the house removes its earlier members wherever they are)
#[test]
fn claim_33_group_kept_on_owner_change_both_ways() {
    for kept in [true, false] {
        let mut p = start_with_quirks(|q| q.group_kept_on_owner_change = kept);
        let m = mixed_mobiles(&mut p);
        let theirs = their_gunr(&mut p);
        let other = p.world().obj(theirs).unwrap().owner;
        let local = Some(p.me());
        p.selection = vec![m[1], m[2]];
        p.make_group(1);
        p.game.world.obj_mut(m[1]).unwrap().owner = other;
        assert_eq!(p.group(1), vec![m[2]], "not a member while theirs");
        p.selection = vec![m[3]];
        p.make_group(1);
        p.game.world.obj_mut(m[1]).unwrap().owner = local;
        let want = if kept { vec![m[1], m[3]] } else { vec![m[3]] };
        assert_eq!(p.group(1), want, "kept {kept}");
    }
}

/// The screen-candidate pool of a type-select tap given by hand, the
/// fixture drawing none: a tap of `p` with `screen` and `reach`.
fn tap(p: &mut Play, screen: &[ObjectId], reach: Reach) -> Reach {
    p.messages.clear();
    p.type_tap(screen, reach)
}

fn said(p: &Play) -> String {
    p.messages
        .iter()
        .map(|m| String::from_utf16_lossy(&m.units))
        .collect()
}

/// An own GUNR placed `dx` cells east of the base unit's row of
/// [`mixed_mobiles`].
fn own_gunr(p: &mut Play, dx: i32) -> ObjectId {
    let base = own(p, Kind::Vehicle)[0];
    let local = p.me();
    let w = &mut p.game.world;
    let at = w.obj(base).unwrap().pos.cell();
    let t = w.rules().index_of(Kind::Vehicle, "GUNR").unwrap();
    let id = glasnost_sim::objects::create(w, Kind::Vehicle, t, Some(local));
    let c = CellXY::new(at.x + dx, at.y + 6);
    let z = w.surface_at(256 * c.x + 128, 256 * c.y + 128);
    glasnost_sim::objects::place(w, id, glasnost_sim::world::Pos::centre(c, z), 0);
    id
}

/// spec: S-0214 claim 32 (with reach screen and an unselected screen candidate, the tap adds the screen candidates in candidate order and the reach stays screen; otherwise the reach becomes map and every candidate is added in creation order; own objects only; one response, from the first object added; the messages by the reach and an empty selection)
#[test]
fn claim_32_a_tap_widens_from_the_screen_to_the_map() {
    let mut p = start();
    let m = mixed_mobiles(&mut p);
    let (g1, g2) = (m[1], m[3]);
    let far = own_gunr(&mut p, 9);
    let theirs = their_gunr(&mut p);
    p.selection = vec![g1];
    p.selected.clear();
    // The screen holds g2 (and the selected g1, another type and theirs).
    let r = tap(&mut p, &[g1, m[2], theirs, g2], Reach::Screen);
    assert_eq!(r, Reach::Screen);
    assert_eq!(p.selection, added_in_order(&p, &[g1], &[g2]));
    assert_eq!(p.selected, vec![g2]);
    assert_eq!(said(&p), "MSG:SelAcrossScreen");
    assert!(!p.sel_reset, "the tap keeps its reach");
    // Nothing unselected on the screen: the reach becomes map.
    p.selected.clear();
    let r = tap(&mut p, &[g1, g2], Reach::Screen);
    assert_eq!(r, Reach::Map);
    assert!(p.selection.contains(&far) && !p.selection.contains(&theirs));
    assert_eq!(p.selected, vec![far]);
    assert_eq!(said(&p), "MSG:SelAcrossMap");
    // A tap at map adding nothing still says across the map.
    p.selected.clear();
    let before = p.selection.clone();
    assert_eq!(tap(&mut p, &[], Reach::Map), Reach::Map);
    assert_eq!(p.selection, before);
    assert!(p.selected.is_empty(), "nothing added, no response");
    assert_eq!(said(&p), "MSG:SelAcrossMap");
    // An empty selection: nothing selected.
    p.clear_selection();
    assert!(p.sel_reset, "a clearing resets the reach");
    tap(&mut p, &[g1], Reach::Screen);
    assert!(p.selection.is_empty());
    assert_eq!(said(&p), "MSG:NothingSelected");
}

/// spec: S-0214 claim 32 (if the selection is a single object that is not own, the tap first removes it)
#[test]
fn claim_32_a_single_foreign_object_is_removed_first() {
    let mut p = start();
    mixed_mobiles(&mut p);
    let theirs = their_gunr(&mut p);
    p.selection = vec![theirs];
    tap(&mut p, &[], Reach::Screen);
    assert!(p.selection.is_empty());
    assert_eq!(said(&p), "MSG:NothingSelected");
}

/// spec: S-0214 claim 32 (held drags: no claim 19 candidate tests; every live box candidate inside, of any house and kind, buildings included, stands for its exact type and adds the own objects of that type that can be selected within the reach, inside the box or not; a deactivated vehicle and a building that is not vehicle-like join; no deploys-into types; claim 19's clearing and empty-box rules still apply; silent) and claim 29 (types in first in-box encounter order, within a type the pool's order)
#[test]
fn claim_32_a_held_drag_seeds_every_candidate_in_the_box() {
    let mut p = start();
    let m = mixed_mobiles(&mut p);
    let (mcv, g1, n1, g2, n2) = (m[0], m[1], m[2], m[3], m[4]);
    let far = own_gunr(&mut p, 9);
    let theirs = their_gunr(&mut p);
    let at = p.world().obj(mcv).unwrap().pos.cell();
    let local = p.me().0;
    let w = &mut p.game.world;
    let pw1 = testkit::spawn(
        w,
        Kind::Building,
        "POWR",
        local,
        CellXY::new(at.x + 4, at.y - 6),
        0,
    );
    let pw2 = testkit::spawn(
        w,
        Kind::Building,
        "POWR",
        local,
        CellXY::new(at.x - 6, at.y - 6),
        0,
    );
    let yard = testkit::spawn(
        w,
        Kind::Building,
        "YARD",
        local,
        CellXY::new(at.x - 6, at.y + 8),
        0,
    );
    let map: Vec<ObjectId> = p.world().objects.keys().copied().collect();
    // An enemy GUNR alone in the box: the player's own GUNRs across the
    // reach, none of theirs; the old selection is cleared.
    p.selection = vec![n1];
    p.selected.clear();
    assert!(!p.type_drag(&[theirs], &map, false));
    assert_eq!(p.selection, added_in_order(&p, &[], &[g1, g2, far]));
    assert!(p.selected.is_empty(), "the drag is silent");
    assert!(p.sel_reset);
    // The screen reach: GUNRs outside the box but on the screen join, the
    // one off the screen does not.
    p.type_drag(&[theirs], &[g1, theirs, g2], false);
    assert_eq!(p.selection, added_in_order(&p, &[], &[g1, g2]));
    // A deactivated own GUNR in the box joins with the others of its type.
    p.game
        .world
        .obj_mut(g2)
        .unwrap()
        .econ
        .building
        .unit_disabled = true;
    assert!(!targeting::box_eligible(p.world(), p.me(), &[], g2));
    p.type_drag(&[g2], &map, false);
    assert_eq!(p.selection, added_in_order(&p, &[], &[g1, g2, far]));
    // A building that is not vehicle-like seeds its type: both plants.
    p.type_drag(&[pw2], &map, false);
    assert_eq!(p.selection, added_in_order(&p, &[], &[pw1, pw2]));
    // The base unit seeds its own type only, not the yard it deploys into.
    p.type_drag(&[mcv], &map, false);
    assert_eq!(p.selection, vec![mcv]);
    assert!(!p.selection.contains(&yard));
    // Types in first in-box encounter order; force-select keeps the old
    // selection and adds.
    p.selection = vec![pw1];
    p.type_drag(&[n2, theirs, n1], &map, true);
    assert_eq!(
        p.selection,
        added_in_order(&p, &[pw1], &[n1, n2, g1, g2, far])
    );
    // The empty box keeps claim 19's rule: unmodified, the release click
    // (the quirk's default); with force-select, nothing.
    let before = p.selection.clone();
    assert!(p.type_drag(&[], &map, false));
    assert!(!p.type_drag(&[], &map, true));
    assert_eq!(p.selection, before);
}

/// spec: S-0415 claim 6 (the deployed building takes over the deploying
/// vehicle's control group and its selection: a selected vehicle gives a
/// selected building), S-0111 claim 17 (it then shows the group's digit)
#[test]
fn a_deploy_hands_the_control_group_and_the_selection_to_the_building() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    let m = mixed_mobiles(&mut p);
    // The base unit in group 3 with two others; selected with one of them.
    p.selection = vec![m[1], mcv, m[2]];
    p.make_group(3);
    p.selection = vec![m[1], mcv];
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.key(KeyCommand::Deploy);
    step_until(&mut p, 400, |p| !own(p, Kind::Building).is_empty());
    let yard = own(&p, Kind::Building)[0];
    assert!(p.world().obj(mcv).is_none(), "the vehicle is gone");
    // spec: S-0214 claim 33 (membership passes on deploy; members in creation order); design: it takes the vehicle's place in the selection's order
    assert_eq!(p.group(3), vec![m[1], m[2], yard]);
    assert_eq!(p.selection, vec![m[1], yard]);
    // spec: S-0111 claim 17 (key 3's group shows 3)
    assert_eq!(p.group_digits().get(&yard), Some(&3));
    // Recalling the group selects the building with the others.
    p.selection.clear();
    p.select_group(3);
    assert!(p.selection.contains(&yard));
}

/// spec: S-0415 claim 10 (the vehicle that appears takes over the packing
/// building's control group and selection). The synthetic skirmish's yard
/// does not pack up, so the pack-up's report (tested in the simulation,
/// `claims_s0415::claim_10_pack_up_reports_the_hand_over`) is given to the
/// client's effect loop by hand, with a vehicle put where it would appear.
#[test]
fn a_pack_up_hands_the_control_group_and_the_selection_to_the_vehicle() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    step_until(&mut p, 10, |p| p.world().tick >= 4);
    p.selection = vec![mcv];
    p.key(KeyCommand::Deploy);
    step_until(&mut p, 400, |p| !own(p, Kind::Building).is_empty());
    let yard = own(&p, Kind::Building)[0];
    p.selection = vec![yard];
    p.make_group(0);
    // spec: S-0111 claim 17 (the tenth group, key 0, shows 0)
    assert_eq!(p.group_digits().get(&yard), Some(&0));
    let local = p.me();
    let cell = p.world().obj(yard).unwrap().pos.cell();
    let vehicle = {
        let w = &mut p.game.world;
        let t = w
            .rules()
            .index_of(Kind::Vehicle, "GUNR")
            .expect("fixture type");
        let id = glasnost_sim::objects::create(w, Kind::Vehicle, t, Some(local));
        glasnost_sim::objects::remove(w, yard);
        let z = w.surface_at(256 * cell.x + 128, 256 * cell.y + 128);
        glasnost_sim::objects::place(w, id, glasnost_sim::world::Pos::centre(cell, z), 0);
        id
    };
    p.step_by(|g| {
        g.step();
        g.world.effects.push(Effect::Converted {
            old: yard,
            new: vehicle,
        });
        true
    });
    assert_eq!(p.group(0), vec![vehicle]);
    assert_eq!(p.selection, vec![vehicle]);
    assert_eq!(p.group_digits().get(&vehicle), Some(&0));
}

/// spec: S-0415 claims 6 and 10 (the take-over replaces the old object in
/// place); an object in no group and not selected changes nothing
#[test]
fn a_hand_over_keeps_the_order() {
    let (a, b, c, d) = (ObjectId(10), ObjectId(11), ObjectId(12), ObjectId(13));
    let mut selection = vec![a, b];
    let mark = GroupMark {
        group: 0,
        by: HouseId(0),
    };
    let mut groups = BTreeMap::from([(c, mark), (b, mark)]);
    converted(&mut selection, &mut groups, b, d);
    assert_eq!(selection, vec![a, d]);
    assert_eq!(groups, BTreeMap::from([(c, mark), (d, mark)]));
    converted(&mut selection, &mut groups, ObjectId(99), ObjectId(100));
    assert_eq!(selection, vec![a, d]);
    assert_eq!(groups, BTreeMap::from([(c, mark), (d, mark)]));
}

/// spec: S-0214 claim 30 (an accepted cell click in route mode uses the move-target marker rule of a click outside it: a move on a cell gives the marker at the clicked cell, a click on an object none) and S-0213 claim 16a (the click's orders are point adds)
#[test]
fn an_accepted_route_mode_cell_click_shows_the_move_marker() {
    let mut p = start();
    let mcv = *own(&p, Kind::Vehicle).first().expect("a base unit");
    step_until(&mut p, 10, |p| !glasnost_sim::orders::blocked(p.world()));
    p.selection = vec![mcv];
    let cell = p.world().obj(mcv).unwrap().pos.cell();
    let to = CellXY::new(cell.x, cell.y + 2);
    let on = ClickOn::Cell(to);
    let glasnost_sim::routes::RouteClick::Orders(o) =
        glasnost_sim::routes::click(p.world(), p.me(), &p.selection, on, Modifiers::default())
    else {
        panic!("a point add");
    };
    assert!(matches!(o.as_slice(), [Order::RoutePoint { .. }]));
    p.markers.clear();
    p.issue_route_click(o, on, Modifiers::default());
    assert_eq!(
        std::mem::take(&mut p.markers),
        [glasnost_sim::targeting::move_marker_at(p.world(), to)]
    );
    assert!(
        p.game
            .replay()
            .orders
            .iter()
            .any(|(_, _, o)| matches!(o, Order::RoutePoint { .. })),
        "the point add is in the replay"
    );
}
