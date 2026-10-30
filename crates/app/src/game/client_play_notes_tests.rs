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

/// A tap of the type-select key is a release at most 500 ms after the
/// press; a key repeat does not restart the press (S-0214 claim 32). The
/// bare session imports no art, so it draws no mobile object and its screen
/// holds no candidate: the tap's reach becomes the map at once and it adds
/// the type wherever it is.
#[test]
fn the_type_select_key_taps_on_a_release_within_500_ms() {
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let [a, b, far] = three_gunrs(&mut c);
    session_mut(&mut c).play.selection = vec![a];
    let hold = |c: &mut Client, down: i64, up: i64| {
        session_mut(c).clock_ms = down;
        c.key(Key::Char('t'), true);
        session_mut(c).clock_ms = up;
        c.key(Key::Char('t'), false);
        sorted(session(c).play.selection.clone())
    };
    // spec: S-0214 claim 32 and Constants (a release 501 ms after the press is no tap and does nothing)
    assert_eq!(hold(&mut c, 1000, 1501), vec![a]);
    // spec: S-0214 claim 32 (the key's automatic repeat while held does not restart the press time)
    session_mut(&mut c).clock_ms = 3000;
    c.key(Key::Char('t'), true);
    session_mut(&mut c).clock_ms = 3400;
    c.key(Key::Char('t'), true);
    session_mut(&mut c).clock_ms = 3600;
    c.key(Key::Char('t'), false);
    assert_eq!(sorted(session(&c).play.selection.clone()), vec![a]);
    // spec: S-0214 claim 32 and Constants (500 ms, inclusive, is a tap; with no screen candidate the reach becomes map)
    let wide = hold(&mut c, 5000, 5500);
    assert!([a, b, far].iter().all(|id| wide.contains(id)), "{wide:?}");
    assert_eq!(
        session(&c).type_select_reach(),
        crate::game::play::Reach::Map
    );
    // Every own GUNR, the fixture's own ones too, and nothing else.
    let w = session(&c).play.world();
    let ty = w.obj(a).unwrap().type_index;
    let gunrs: Vec<ObjectId> = own(&c, Kind::Vehicle)
        .into_iter()
        .filter(|&id| w.obj(id).unwrap().type_index == ty)
        .collect();
    assert_eq!(wide, sorted(gunrs));
    assert!(!session(&c).type_select_held(), "released");
    // spec: S-0214 claim 32 (a clearing since the last tap sets the reach to screen before the next one)
    session_mut(&mut c).play.clear_selection();
    session_mut(&mut c).paused = true;
    hold(&mut c, 7000, 7100);
    assert_eq!(
        session(&c).type_select_reach(),
        crate::game::play::Reach::Map,
        "paused: no tap"
    );
    session_mut(&mut c).paused = false;
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

fn with(c: &mut Client, mods: Modifiers, d: char) {
    c.modifiers(mods);
    key(c, Key::Char(d));
    c.modifiers(Modifiers::default());
}

const CTRL: Modifiers = Modifiers {
    force_fire: true,
    force_move: false,
    force_select: false,
    attack_move_mode: false,
};
const SHIFT: Modifiers = Modifiers {
    force_fire: false,
    force_move: false,
    force_select: true,
    attack_move_mode: false,
};
const ALT: Modifiers = Modifiers {
    force_fire: false,
    force_move: true,
    force_select: false,
    attack_move_mode: false,
};

/// The control-group keys (S-0213 claim 22, S-0214 claim 33): Ctrl+digit
/// makes a group, Shift+digit adds the group to the selection, Alt+digit
/// centres on it, and the digit selects it; a second press less than 800 ms
/// after the last select that was not a repeat centres the view.
#[test]
fn control_group_keys_make_add_select_and_centre() {
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let [a, b, far] = three_gunrs(&mut c);
    session_mut(&mut c).play.selection = vec![a, b];
    with(&mut c, CTRL, '3');
    assert_eq!(session(&c).play.group(3), vec![a, b]);
    // spec: S-0213 claim 22 and S-0214 claim 33 (Shift with the digit adds the group to the selection)
    session_mut(&mut c).play.selection = vec![far];
    with(&mut c, SHIFT, '3');
    assert_eq!(
        sorted(session(&c).play.selection.clone()),
        sorted(vec![a, b, far])
    );
    assert_eq!(session(&c).play.group(3), vec![a, b], "the group stays");
    // The digit alone selects the group, the view stays.
    session_mut(&mut c).play.selection = vec![far];
    let away = (session(&c).centre().0 + 300, session(&c).centre().1);
    session_mut(&mut c).set_centre(away);
    let start = session(&c).centre();
    session_mut(&mut c).clock_ms = 5000;
    with(&mut c, Modifiers::default(), '3');
    assert_eq!(sorted(session(&c).play.selection.clone()), vec![a, b]);
    assert_eq!(session(&c).centre(), start, "one press only selects");
    // spec: S-0214 claim 33 and Constants (a repeat less than 800 ms later centres the view)
    session_mut(&mut c).clock_ms = 5799;
    with(&mut c, Modifiers::default(), '3');
    let centred = session(&c).centre();
    assert_ne!(centred, start, "the repeat goes to it");
    // spec: S-0214 claim 33 (a repeat does not restart the 800 ms: 801 ms after the select is a new select, which does not move the view)
    session_mut(&mut c).set_centre(away);
    session_mut(&mut c).clock_ms = 5801;
    with(&mut c, Modifiers::default(), '3');
    assert_eq!(session(&c).centre(), start, "no repeat");
    // spec: S-0214 Constants (800 ms exactly is no repeat)
    session_mut(&mut c).clock_ms = 6601;
    with(&mut c, Modifiers::default(), '3');
    assert_eq!(session(&c).centre(), start, "800 ms is too late");
    // spec: S-0214 claim 33 (a repeat needs the selection's own objects to be exactly the members)
    session_mut(&mut c).play.selection = vec![a];
    session_mut(&mut c).clock_ms = 6700;
    with(&mut c, Modifiers::default(), '3');
    assert_eq!(session(&c).centre(), start, "a select, not a repeat");
    assert_eq!(sorted(session(&c).play.selection.clone()), vec![a, b]);
    // spec: S-0213 claim 22 and S-0214 claim 33 (Alt with the digit: the selection becomes the group's and the view is centred on it)
    session_mut(&mut c).play.selection = vec![far];
    with(&mut c, ALT, '3');
    assert_eq!(sorted(session(&c).play.selection.clone()), vec![a, b]);
    assert_eq!(session(&c).centre(), centred, "centred as by the repeat");
    // spec: S-0214 claim 33 (selecting a group with no member leaves the selection empty)
    session_mut(&mut c).clock_ms = 9000;
    with(&mut c, Modifiers::default(), '4');
    assert!(session(&c).play.selection.is_empty());
    // spec: S-0214 claim 33 (while the options menu is open the group keys do nothing)
    session_mut(&mut c).play.selection = vec![far];
    session_mut(&mut c).paused = true;
    session_mut(&mut c).group_key(3, crate::game::session::GroupCommand::Select);
    assert_eq!(session(&c).play.selection, vec![far]);
    session_mut(&mut c).paused = false;
}

/// Route mode through the client (S-0213 claims 16, 16a and table T2):
/// the bound key held, a battlefield click gives the selection's
/// behaviour order as a route point; T2's commands are refused with their
/// message and no order; the key's release, or the focus going, sends one
/// commit.
#[test]
fn route_mode_adds_points_and_commits_on_release() {
    use glasnost_sim::orders::Order;
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let [a, _, _] = three_gunrs(&mut c);
    // Past the opening order block (S-0201 claim 14).
    while glasnost_sim::orders::blocked(session(&c).play.world()) {
        session_mut(&mut c).step();
    }
    let orders = |c: &Client| -> Vec<Order> {
        let s = session(c);
        s.play
            .game
            .replay()
            .orders
            .iter()
            .filter(|(_, h, _)| *h == s.play.local)
            .map(|(_, _, o)| o.clone())
            .collect()
    };
    let before = orders(&c).len();
    assert_eq!(c.settings.bindings.key(Action::RouteMode), 'z');
    session_mut(&mut c).play.selection = vec![a];
    c.key(Key::Char('z'), true);
    assert!(session(&c).route_held());
    // spec: S-0213 table T2 (guard in route mode: no order, refused with its message and the refusal sound; the bare session loads no font, so the message line is not kept)
    session_mut(&mut c).play.cues.clear();
    c.key(Key::Char('g'), true);
    c.key(Key::Char('g'), false);
    assert_eq!(
        session(&c).play.cues,
        vec![glasnost_audio::events::UiCue::StripEnd]
    );
    assert_eq!(orders(&c).len(), before, "no order");
    // spec: S-0213 claim 16a (a click gives the behaviour order as a route point)
    let at = session(&c).map_rect().centre();
    super::access_tests::click_at(&mut c, at);
    let new = orders(&c)[before..].to_vec();
    assert!(
        matches!(new.as_slice(), [Order::RoutePoint { actor, .. }] if *actor == a),
        "{new:?}"
    );
    // spec: S-0213 claim 16a (the release sends one commit)
    c.key(Key::Char('z'), false);
    assert!(!session(&c).route_held());
    assert!(matches!(orders(&c).last(), Some(Order::RouteCommit)));
    let n = orders(&c).len();
    c.key(Key::Char('z'), false);
    assert_eq!(orders(&c).len(), n, "a second release sends nothing");
    // design: the focus going lets go of the key, which commits
    c.key(Key::Char('z'), true);
    c.focus(false);
    c.focus(true);
    assert!(!session(&c).route_held());
    assert_eq!(orders(&c).len(), n + 1);
    assert!(matches!(orders(&c).last(), Some(Order::RouteCommit)));
}

/// Route-mode refusals share one cue per tick (S-0213 claim 16, S-0701
/// claim 24): a second refusal before the next tick completes adds no cue;
/// after a completed tick a new one does.
#[test]
fn route_mode_refusals_share_one_cue_per_tick() {
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let [a, _, _] = three_gunrs(&mut c);
    while glasnost_sim::orders::blocked(session(&c).play.world()) {
        session_mut(&mut c).step();
    }
    session_mut(&mut c).play.selection = vec![a];
    c.key(Key::Char('z'), true);
    session_mut(&mut c).play.cues.clear();
    let refuse = |c: &mut Client| {
        c.key(Key::Char('g'), true);
        c.key(Key::Char('g'), false);
    };
    // spec: S-0701 claim 24 (refusal notices before the next tick completes share one cue)
    refuse(&mut c);
    refuse(&mut c);
    assert_eq!(
        session(&c).play.cues,
        vec![glasnost_audio::events::UiCue::StripEnd]
    );
    // spec: S-0701 claim 24 (a completed tick permits a new one)
    session_mut(&mut c).step();
    session_mut(&mut c).play.cues.clear();
    refuse(&mut c);
    assert_eq!(
        session(&c).play.cues,
        vec![glasnost_audio::events::UiCue::StripEnd]
    );
}

/// The web stand-in for Ctrl+digit (M8 A4 round 5 seams, our design):
/// with the fallback on, the held make-group key (N by default, rebindable)
/// and a digit make that group; its release, or the focus going, ends it.
/// With the fallback off (the native build) the key does nothing and a
/// digit only selects; Ctrl+digit makes the group either way.
#[test]
fn the_make_group_key_stands_in_for_ctrl_on_the_web() {
    let mut c = client((800, 600), 100);
    start_by_keyboard(&mut c);
    let [a, b, far] = three_gunrs(&mut c);
    assert_eq!(c.settings.bindings.key(Action::MakeGroup), 'n');
    // Off, as natively: N held and 5 make no group.
    c.group_fallback = false;
    session_mut(&mut c).play.selection = vec![a, b];
    c.key(Key::Char('n'), true);
    key(&mut c, Key::Char('5'));
    c.key(Key::Char('n'), false);
    assert!(session(&c).play.group(5).is_empty());
    // On: N held and 5 make group 5 of the selection.
    c.group_fallback = true;
    session_mut(&mut c).play.selection = vec![a, b];
    c.key(Key::Char('n'), true);
    key(&mut c, Key::Char('5'));
    assert_eq!(session(&c).play.group(5), vec![a, b]);
    // Released, 5 selects the group again.
    c.key(Key::Char('n'), false);
    session_mut(&mut c).play.selection = vec![far];
    key(&mut c, Key::Char('5'));
    assert_eq!(
        sorted(session(&c).play.selection.clone()),
        sorted(vec![a, b])
    );
    // The focus going lets go of it.
    c.key(Key::Char('n'), true);
    c.focus(false);
    c.focus(true);
    session_mut(&mut c).play.selection = vec![far];
    key(&mut c, Key::Char('5'));
    assert_eq!(
        sorted(session(&c).play.selection.clone()),
        sorted(vec![a, b])
    );
    // Rebound to J, N no longer holds it.
    c.settings
        .bindings
        .rebind(Action::MakeGroup, 'j')
        .expect("J is free");
    session_mut(&mut c).play.selection = vec![far];
    c.key(Key::Char('n'), true);
    key(&mut c, Key::Char('6'));
    c.key(Key::Char('n'), false);
    assert!(session(&c).play.group(6).is_empty());
    session_mut(&mut c).play.selection = vec![far];
    c.key(Key::Char('j'), true);
    key(&mut c, Key::Char('6'));
    c.key(Key::Char('j'), false);
    assert_eq!(session(&c).play.group(6), vec![far]);
}
