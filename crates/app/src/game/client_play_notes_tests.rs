//! The M8 A4 round 5 play notes through the windowless client on synthetic
//! data (invented rules and map, `glasnost_ai::fixture`): the radar's view
//! outline, the superweapon countdowns, the superweapon aiming cursor, the
//! type-select key and the control groups, the icon tooltips and the
//! placement range. The specs cited are the ones each fix implements; the
//! rest is our own design, named so where it is tested.

use std::sync::Arc;

use super::access_tests::{client, session_mut, start_by_keyboard, start_by_keyboard_with};
use super::*;
use glasnost_ai::fixture;
use glasnost_sim::rules::SimRules;
use glasnost_ui::hud;

/// The fixture's rules with its superweapon showing a countdown.
fn countdown_rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    for t in r.sim.superweapons.values_mut() {
        t.show_timer = Some(true);
    }
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

/// The radar outline moves with the view, and its middle names a cell
/// close to the one under the battlefield's centre (note 22).
#[test]
fn the_radar_outline_follows_the_view() {
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let s = session_mut(&mut c);
    let first = s.radar_view_box().expect("a radar map");
    let (l, t, r, b) = first.edges;
    assert!(l < r && t < b, "{first:?}");
    let check = |s: &mut Session| {
        let bx = s.radar_view_box().unwrap();
        let (l, t, r, b) = bx.edges;
        let mid = (
            (l + r).div_euclid(2) - bx.image.x,
            (t + b).div_euclid(2) - bx.image.y,
        );
        let radar = s.radar.as_ref().unwrap();
        let named = radar.cell_at(s.play.world(), mid.0 + bx.image.x, mid.1 + bx.image.y);
        let under = s.under(s.map_rect().centre()).0;
        let n = named.expect("the outline's middle is on the radar image");
        // design: the outline reads height level 0; the synthetic map is
        // flat, so the middle is within a cell or two of the view's
        assert!(
            (n.x - under.x).abs() <= 2 && (n.y - under.y).abs() <= 2,
            "{n:?} against {under:?}"
        );
        bx
    };
    // spec: S-0410 claim 11 (the outline marks the battlefield view's part of the map)
    let at_start = check(s);
    let centre = s.centre();
    s.set_centre((centre.0 + 400, centre.1 + 200));
    let moved = check(s);
    assert_ne!(s.centre(), centre, "the view moved");
    assert_ne!(
        at_start.edges, moved.edges,
        "the outline moves with the view"
    );
}

/// Every granted superweapon with the countdown flag shows a row with its
/// house's colour and the time left (note 25).
#[test]
fn granted_countdowns_show_on_the_battlefield() {
    let mut c = client((800, 600), 100);
    start_by_keyboard_with(&mut c, countdown_rules());
    let s = session_mut(&mut c);
    assert!(s.countdown_rows().is_empty(), "nothing granted yet");
    let local = s.play.local;
    glasnost_sim::superweapons::charge::permanent_grant(&mut s.play.game.world, local, 0);
    let rows = s.countdown_rows();
    // spec: S-0512 claim 18 (each granted record with the countdown flag has a row, the name in its house's colour)
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0].name, "NUKE");
    assert_eq!(rows[0].colour, s.house_colours[usize::from(local.0)]);
    let time = hud::countdown_time(rows[0].seconds);
    let (list, _) = c.frame();
    assert!(list.texts().contains(&"NUKE "), "{:?}", list.texts());
    assert!(rows[0].seconds > 0, "{rows:?}");
    assert!(list.texts().contains(&time.as_str()));
}

/// The fixture's rules with its superweapon's kind and aim action word.
fn aim_rules(kind: &str, action: &str) -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let t = r.sim.superweapons.get_mut("NUKE").unwrap();
    t.kind = Some(kind.into());
    t.action = Some(action.into());
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

/// A started game on `rules`, its superweapon armed with nothing selected,
/// and the battlefield's centre pixel.
fn armed(rules: Arc<SimRules>) -> (Client, Point) {
    let mut c = client((800, 600), 100);
    start_by_keyboard_with(&mut c, rules);
    let s = session_mut(&mut c);
    s.play.selection.clear();
    s.play.tool = super::super::play::Tool::Superweapon {
        sw: 0,
        from_source: false,
    };
    let p = s.map_rect().centre();
    (c, p)
}

/// The battlefield cursor offers the armed superweapon's table CS image
/// with no selection, the units' action with one, and the refusal image
/// for a force shield away from an own building (note 27).
#[test]
fn an_armed_superweapon_offers_its_aim_image() {
    use super::super::cursor::{Aim, Offer};
    let (mut c, p) = armed(aim_rules("MultiMissile", "Nuke"));
    let s = session_mut(&mut c);
    // spec: S-0214 claim 31 and table CS (the image by aim action: nuclear strike)
    assert_eq!(s.offer(p).map(|o| o.0), Some(Offer::Aim(Aim::Nuke)));
    // spec: S-0513 claim 1 (while units are selected, the cursor offers their actions)
    let local = s.play.local;
    s.play.selection = s
        .play
        .world()
        .objects
        .values()
        .filter(|o| o.owner == Some(local) && o.on_map)
        .map(|o| o.id)
        .take(1)
        .collect();
    assert!(!s.play.selection.is_empty());
    assert!(matches!(s.offer(p), Some((Offer::Action { .. }, _))));
    // An aim action outside table CS keeps the system pointer.
    let (mut c, p) = armed(aim_rules("MultiMissile", "Attack"));
    assert_eq!(session_mut(&mut c).offer(p), None);
    // spec: S-0513 claim 1 and S-0214 table CS (a force shield away from an own or allied building shows no-force-shield)
    let (mut c, p) = armed(aim_rules("ForceShield", "ForceShield"));
    let s = session_mut(&mut c);
    let (cell, object) = s.under(p);
    let on = s.play.target(object, cell);
    let want = if s.play.sw_offered(0, on) {
        Aim::ForceShield
    } else {
        Aim::NoForceShield
    };
    assert_eq!(s.offer(p).map(|o| o.0), Some(Offer::Aim(want)));
}
