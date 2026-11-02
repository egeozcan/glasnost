//! The watch game through the windowless client on synthetic data
//! (invented rules and map, `glasnost_ai::fixture`): a skirmish whose rows
//! are all computers', started from the setup screen and watched to its
//! end with no house played.
//!
//! design: watch mode (lead request 2026-11-02, an extra not in the
//! original); the specs cited are the ones the rules come from.

use super::access_tests::{click, client, run_start, session, session_mut};
use super::*;
use glasnost_ai::fixture;
use glasnost_ai::game::Game;
use glasnost_map::CellXY;
use glasnost_sim::orders::Order;
use glasnost_sim::world::{HouseId, HouseRole, ObjectId};
use glasnost_ui::hud::{HudHit, PauseHit};
use glasnost_ui::setup::{SetupHit, SlotKind};

use super::super::session::GroupCommand;

/// Opens the setup, turns the first row into a computer's and starts.
fn start_watch(c: &mut Client) {
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(c, r);
    let _ = c.frame();
    let r = c
        .setup_targets
        .rect_of(&SetupHit::Kind(0, 1))
        .expect("the first row's kind is offered");
    click(c, r);
    let _ = c.frame();
    assert_eq!(c.shell.setup.as_ref().unwrap().slots[0].kind, SlotKind::Ai);
    let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    c.next_seed = 7;
    click(c, r);
    run_start(c);
}

/// The player houses, in house order.
fn players(s: &Session) -> Vec<HouseId> {
    let w = s.play.world();
    (0..w.houses.len())
        .filter(|&i| w.houses[i].role == HouseRole::Player)
        .map(|i| HouseId(i as u8))
        .collect()
}

/// An object of house `h` on the map.
fn object_of(s: &Session, h: HouseId) -> ObjectId {
    s.play
        .world()
        .objects
        .values()
        .find(|o| o.owner == Some(h) && o.on_map)
        .map(|o| o.id)
        .expect("the house has an object on the map")
}

// spec: S-0412 Terms (no human participant: every player house a computer's) and claim 12 (AI houses take the computer slots)
#[test]
fn the_setup_starts_a_game_with_no_played_house() {
    let mut c = client((1024, 768), 100);
    start_watch(&mut c);
    let s = session(&c);
    assert_eq!(s.play.local, None);
    assert!(s.play.watching());
    let w = s.play.world();
    let ps = players(s);
    assert_eq!(ps.len(), 2);
    for &h in &ps {
        let setup = &w.houses[usize::from(h.0)].econ.setup;
        assert!(setup.ai && !setup.person_played && !setup.observer, "{h:?}");
    }
    assert_eq!(s.play.game.ai_houses(), ps, "every player house thinks");
}

// design: watch mode (lead request 2026-11-02, an extra not in the original): no shroud and no fog anywhere, the whole radar map with every house's objects, and the watch panel in the sidebar's place
#[test]
fn the_watch_view_draws_no_shroud_and_the_whole_radar() {
    let mut c = client((1024, 768), 100);
    start_watch(&mut c);
    let s = session_mut(&mut c);
    for _ in 0..30 {
        s.step();
    }
    assert!(s.shroud_covers().is_empty(), "no shroud or fog cover");
    // Every house's objects on the radar, each in its own colour.
    let radar = s.radar_rgba().expect("a radar map");
    let ps = players(s);
    let w = s.play.world();
    let r = s.radar.as_ref().unwrap();
    let pixels = r.watch_pixels(w);
    let (_, _, iw, _) = r.image_rect();
    for &h in &ps {
        let o = w.obj(object_of(s, h)).unwrap();
        let colour = glasnost_sim::econ::minimap::radar_colour(
            s.house_colours[usize::from(h.0)],
            crate::mapview::radar::DISPLAY_BITS,
        );
        assert!(
            pixels.contains(&colour),
            "{h:?}'s colour {colour:?} on the radar ({:?} at {:?})",
            o.kind,
            o.pos
        );
    }
    assert_eq!(pixels.len() as i32 % iw, 0);
    assert!(radar.chunks(4).any(|p| p[..3] != [0, 0, 0]), "not black");
    // The watch panel: every house, and no order's target.
    let m = s.watch_model();
    let names: Vec<&str> = m.houses.iter().map(|r| r.name.as_str()).collect();
    let world_names: Vec<&str> = ps
        .iter()
        .map(|h| w.houses[usize::from(h.0)].name.as_str())
        .collect();
    assert_eq!(names, world_names);
    let _ = c.frame();
    for (_, hit) in &session(&c).hud_targets().list {
        assert!(
            matches!(hit, HudHit::Map | HudHit::Radar | HudHit::Options),
            "{hit:?}"
        );
    }
}

// design: watch mode (lead request 2026-11-02, an extra not in the original): clicks, keys, the sidebar, groups and route mode give no order; a left click selects any house's object for looking
#[test]
fn the_watch_game_takes_no_orders() {
    let mut c = client((1024, 768), 100);
    start_watch(&mut c);
    let s = session_mut(&mut c);
    let ps = players(s);
    let id = object_of(s, ps[1]);
    let cell = s.play.world().obj(id).unwrap().pos.cell();
    s.play
        .click_map(Button::Left, Some(id), cell, Modifiers::default());
    assert_eq!(s.play.selection, vec![id], "selected for looking");
    let m = s.watch_model();
    assert_eq!(m.selected.len(), 3, "{:?}", m.selected);
    assert!(m.selected[1] == s.play.world().houses[usize::from(ps[1].0)].name);
    // A left click on the ground with an object selected gives no move.
    s.play.click_map(
        Button::Left,
        None,
        CellXY::new(cell.x + 2, cell.y + 2),
        Modifiers::default(),
    );
    assert!(s.play.selection.is_empty());
    s.play
        .click_map(Button::Left, Some(id), cell, Modifiers::default());
    for k in [
        KeyCommand::Stop,
        KeyCommand::Guard,
        KeyCommand::Scatter,
        KeyCommand::Deploy,
    ] {
        s.key(k);
    }
    s.play.cameo_click(0, Button::Left, false);
    s.group_key(1, GroupCommand::Make);
    s.route_key(true);
    assert!(!s.route_held(), "route mode is never entered");
    s.route_key(false);
    s.delete_key();
    for _ in 0..10 {
        s.step();
    }
    assert!(s.play.game.replay().orders.is_empty(), "no order given");
    assert_eq!(s.play.selection, vec![id], "the selection is kept");
    // The options menu: the leave only, and the speed control.
    assert!(s.menu_allowed());
    assert!(s.speed_offered());
    assert_eq!(s.leave_choices(), vec![PauseHit::Leave]);
}

// spec: S-0414 claim 6 (the end test needs no person-played house) and claim 7 (every player house's result)
// design: watch mode (lead request 2026-11-02, an extra not in the original): the end screen names the winner and tells no one they won or lost; a watch game saves, loads and replays like any skirmish
#[test]
fn a_watch_game_ends_with_the_winner_named_and_replays() {
    let mut c = client((1024, 768), 100);
    start_watch(&mut c);
    let s = session_mut(&mut c);
    let ps = players(s);
    for _ in 0..60 {
        s.step();
    }
    // The menu's speed change is an order in the first player house's name.
    // spec: S-0201 claim 17 (the speed order of whoever sent it)
    assert!(s.play.change_speed(2));
    let orders = &s.play.game.replay().orders;
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0].1, ps[0]);
    for _ in 0..20 {
        s.step();
    }
    // A save of the watch game loads to the same state.
    let rules = fixture::rules();
    let bytes = s.play.game.save();
    let loaded = Game::load(&bytes, rules.clone(), fixture::catalogue()).expect("it loads");
    assert_eq!(loaded.state_hash(), s.play.game.state_hash());
    // The second house gives up; the first is the winner.
    // spec: S-0414 claim 11 (a surrender defeats the house)
    assert!(s.play.game.issue(ps[1], Order::Surrender));
    // Each tick's state while the game runs (the interface reads the
    // statistics into the world once play stops, S-0414 claim 19, which a
    // replay does not).
    let mut hashes = Vec::new();
    let mut n = 0;
    loop {
        s.step();
        if !s.play.running() {
            break;
        }
        hashes.push((s.play.world().tick, s.play.game.state_hash()));
        n += 1;
        assert!(n < 2_000, "the game ends");
    }
    let replay = s.play.game.replay().clone();
    let winner = s.play.world().houses[usize::from(ps[0].0)].name.clone();
    let end = s.play.end().expect("the end screen's model");
    assert!(end.rows.iter().all(|r| !r.local), "no local row");
    assert_eq!(end.result_line(), format!("Winner: {winner}"));
    let (d, _) = end.view((1024, 768), None);
    let texts = d.texts();
    for told in ["VICTORY", "DEFEAT"] {
        assert!(
            !texts.contains(&told),
            "{told} on a watch game's end screen"
        );
    }
    // The client shows it, and the way back is the menu.
    c.update(100_000);
    assert!(c.end().is_some(), "the end screen");
    c.end_press();
    assert_eq!(c.shell.screen, Screen::Menu);
    // The replay plays the same game, tick for tick.
    let mut replayed = std::collections::BTreeMap::new();
    let again = Game::run_replay(&replay, rules, &fixture::map(), fixture::catalogue(), |g| {
        // spec: S-0201 claim 1 (the final tick leaves the tick number at itself: the first state at a tick number is the running game's)
        replayed
            .entry(g.world.tick)
            .or_insert_with(|| g.state_hash());
    })
    .expect("it replays");
    assert!(again.outcome().is_some(), "the replayed game ends too");
    assert!(!hashes.is_empty());
    for (tick, hash) in &hashes {
        assert_eq!(replayed.get(tick), Some(hash), "tick {tick}");
    }
}
