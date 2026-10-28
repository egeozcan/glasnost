//! The M8 A4 round 5 play notes through the windowless client on synthetic
//! data (invented rules and map, `glasnost_ai::fixture`): the radar's view
//! outline, the superweapon countdowns, the superweapon aiming cursor, the
//! type-select key and the control groups, the icon tooltips and the
//! placement range. The specs cited are the ones each fix implements; the
//! rest is our own design, named so where it is tested.

use std::sync::Arc;

use super::access_tests::{
    client, key, own, session, session_mut, start_by_keyboard, start_by_keyboard_with,
};
use super::*;
use glasnost_ai::fixture;
use glasnost_map::CellXY;
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::ObjectId;
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

/// Two GUNR of the local house beside its base unit and one far off the
/// screen, then a tick and a frame.
fn three_gunrs(c: &mut Client) -> [ObjectId; 3] {
    let base = *own(c, Kind::Vehicle).first().expect("a base unit");
    let s = session_mut(c);
    let local = s.play.local;
    let w = &mut s.play.game.world;
    let at = w.obj(base).unwrap().pos.cell();
    let far = if at.x < fixture::SIZE { 30 } else { -30 };
    let t = w
        .rules()
        .index_of(Kind::Vehicle, "GUNR")
        .expect("fixture type");
    let ids = [(2, 3), (3, 3), (far, 0)].map(|(dx, dy)| {
        let id = glasnost_sim::objects::create(w, Kind::Vehicle, t, Some(local));
        let cell = CellXY::new(at.x + dx, at.y + dy);
        let z = w.surface_at(256 * cell.x + 128, 256 * cell.y + 128);
        glasnost_sim::objects::place(w, id, glasnost_sim::world::Pos::centre(cell, z), 0);
        id
    });
    session_mut(c).step();
    let _ = c.frame();
    ids
}

fn sorted(mut v: Vec<ObjectId>) -> Vec<ObjectId> {
    v.sort();
    v
}

/// A press of the type-select key selects the selection's type on the
/// screen, and a second press at most 500 ms after the one before across
/// the map (note 31). The bare session imports no art, so it draws no
/// mobile object and its screen holds no candidate: a press on the screen
/// adds nothing here (the local lane shows it on the real data), the
/// map-wide press adds the type wherever it is.
#[test]
fn the_type_select_key_widens_to_the_map_on_a_quick_second_press() {
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let [a, b, far] = three_gunrs(&mut c);
    session_mut(&mut c).play.selection = vec![a];
    let press = |c: &mut Client, at: i64| {
        session_mut(c).clock_ms = at;
        key(c, Key::Char('t'));
        sorted(session(c).play.selection.clone())
    };
    // spec: S-0214 claim 12 (on the screen only; Q-1987: a press alone acts, scenario T2)
    assert_eq!(press(&mut c, 1000), vec![a]);
    // spec: S-0214 Constants (501 ms after the press before is too late for the double press)
    assert_eq!(press(&mut c, 1501), vec![a]);
    // spec: S-0214 claim 12 and Constants (500 ms, inclusive, widens it to the whole map)
    let wide = press(&mut c, 2001);
    assert!([a, b, far].iter().all(|id| wide.contains(id)), "{wide:?}");
    // Every own GUNR, the fixture's own ones too, and nothing else.
    let w = session(&c).play.world();
    let ty = w.obj(a).unwrap().type_index;
    let gunrs: Vec<ObjectId> = own(&c, Kind::Vehicle)
        .into_iter()
        .filter(|&id| w.obj(id).unwrap().type_index == ty)
        .collect();
    assert_eq!(wide, sorted(gunrs));
    assert!(!session(&c).type_select_held(), "released");
}

/// The type-select key is held from its press to its release, and losing
/// the window's focus lets go of it, since its release may not come back;
/// a key that is not the binding does not hold it (note 31).
#[test]
fn the_type_select_key_is_held_until_released_or_the_focus_goes() {
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    // spec: S-0214 Terms (the type-select role's key is the player's binding; ours defaults to T)
    c.key(Key::Char('t'), true);
    assert!(session(&c).type_select_held());
    c.key(Key::Char('y'), false);
    assert!(session(&c).type_select_held(), "another key's release");
    c.key(Key::Char('t'), false);
    assert!(!session(&c).type_select_held());
    c.key(Key::Char('t'), true);
    c.focus(false);
    assert!(!session(&c).type_select_held());
    c.focus(true);
    // Rebound to K, T is no longer the role's key.
    let k = Action::TypeSelect;
    c.settings.bindings.rebind(k, 'k').expect("K is free");
    c.key(Key::Char('t'), true);
    assert!(!session(&c).type_select_held());
    c.key(Key::Char('t'), false);
    c.key(Key::Char('k'), true);
    assert!(session(&c).type_select_held());
    c.key(Key::Char('k'), false);
    assert!(!session(&c).type_select_held());
}

/// The control-group keys: Ctrl+digit makes a group, Shift+digit adds to
/// it, the digit selects it, and a second press within the interval goes
/// to it (note 31, our design, Q-1988).
#[test]
fn control_group_keys_make_select_and_go_to_a_group() {
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let [a, b, far] = three_gunrs(&mut c);
    let with = |c: &mut Client, mods: Modifiers, d: char| {
        c.modifiers(mods);
        key(c, Key::Char(d));
        c.modifiers(Modifiers::default());
    };
    let ctrl = Modifiers {
        force_fire: true,
        ..Modifiers::default()
    };
    let shift = Modifiers {
        force_select: true,
        ..Modifiers::default()
    };
    session_mut(&mut c).play.selection = vec![a, b];
    with(&mut c, ctrl, '3');
    session_mut(&mut c).play.selection = vec![far];
    with(&mut c, shift, '3');
    assert_eq!(session(&c).play.groups[3], vec![a, b, far]);
    session_mut(&mut c).play.selection.clear();
    let start = (session(&c).centre().0 + 300, session(&c).centre().1);
    session_mut(&mut c).set_centre(start);
    let start = session(&c).centre();
    session_mut(&mut c).clock_ms = 5000;
    with(&mut c, Modifiers::default(), '3');
    assert_eq!(
        sorted(session(&c).play.selection.clone()),
        sorted(vec![a, b, far])
    );
    assert_eq!(session(&c).centre(), start, "one press only selects");
    session_mut(&mut c).clock_ms = 5000 + crate::game::session::GROUP_DOUBLE_MS;
    with(&mut c, Modifiers::default(), '3');
    assert_ne!(session(&c).centre(), start, "the second press goes to it");
    // Another group's digit with nothing in it keeps the selection.
    with(&mut c, Modifiers::default(), '4');
    assert_eq!(session(&c).play.selection.len(), 3);
}
