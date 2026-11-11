//! The name tip's text (S-0111 claims 20 to 23) and the display names it
//! shows (S-0004 claim 17) through the windowless client on synthetic data:
//! the invented rules and map of `glasnost_ai::fixture`, an invented rules
//! text for the labels and an invented string table. Every label and text
//! here is made up.

use std::sync::Arc;

use super::access_tests::{client, own, session_mut, start_by_keyboard_with};
use super::*;
use crate::game::play::Button;
use crate::game::strings::StringTable;
use crate::game::{name_tip, status_font};
use glasnost_ai::fixture;
use glasnost_convert::inputs::{InputFile, Inputs};
use glasnost_convert::turrets::EMPTY;
use glasnost_convert::{ImportOptions, convert};
use glasnost_map::CellXY;
use glasnost_sim::cloak::{CloakState, Disguise, DisguiseAs};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{HouseId, ObjectId};

fn u(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// The fixture's rules with TRUK a multi-turret gunner type of four
/// turrets (S-0105 claim 43), so that its index follows its weapon number.
fn rules() -> Arc<SimRules> {
    let mut r = fixture::ruleset();
    let t = r.sim.vehicles.get_mut("TRUK").expect("fixture type");
    t.turret = Some(true);
    t.turret_count = Some(4);
    t.weapon_count = Some(4);
    t.gunner = Some(true);
    Arc::new(SimRules::new(&r, "synthetic-skirmish"))
}

/// An import of the fixture's vehicle and infantry types in which each
/// type's section has the lines `extra` gives it and, unless `extra` gives
/// it `-`, the label `N:<name>`.
fn import(extra: &[(&str, &str)]) -> glasnost_convert::Import {
    let vehicles = ["MCV", "MINR", "GUNR", "LNCH", "TRUK"];
    let mut text = String::from("[VehicleTypes]\n");
    for (i, n) in vehicles.iter().enumerate() {
        text += &format!("{i}={n}\n");
    }
    text += "[InfantryTypes]\n0=WALKER\n[BuildingTypes]\n0=POWR\n";
    for n in vehicles.iter().chain(&["WALKER", "POWR"]) {
        text += &format!("[{n}]\n");
        let lines: Vec<&str> = extra
            .iter()
            .filter(|(s, _)| s == n)
            .map(|(_, l)| *l)
            .collect();
        if !lines.contains(&"-") {
            text += &format!("UIName=N:{n}\n");
        }
        for l in lines.iter().filter(|l| **l != "-") {
            text += &format!("{l}\n");
        }
    }
    let base = InputFile {
        name: "rules.ini".into(),
        origin: "test".into(),
        bytes: text.into_bytes(),
    };
    convert(
        &Inputs {
            base: Some(base),
            ..Default::default()
        },
        &ImportOptions::default(),
    )
}

/// A string table with a text for each type label, the shroud label and
/// the three turret-tip labels.
fn table() -> StringTable {
    let mut entries: Vec<(Vec<u8>, Vec<u16>)> =
        ["MCV", "MINR", "GUNR", "LNCH", "TRUK", "WALKER", "POWR"]
            .iter()
            .map(|n| {
                (
                    format!("N:{n}").into_bytes(),
                    u(&format!("the {}", n.to_lowercase())),
                )
            })
            .collect();
    for (l, t) in [
        ("TXT_SHADOW", "unknown ground"),
        ("Tip:Rocket", "label-a"),
        ("Tip:MachineGun", "label-b"),
        ("Tip:Repair", "label-c"),
    ] {
        entries.push((l.as_bytes().to_vec(), u(t)));
    }
    StringTable::new(entries)
}

/// A game on `rules` whose session has the import of `extra` and the
/// string table, with the cells around the local base unit explored (an
/// area reveal of radius 8, S-0409 claim 17) so that the objects the tests
/// place beside it stand on explored ground.
fn game(extra: &[(&str, &str)]) -> Client {
    let mut c = client((800, 600), 100);
    start_by_keyboard_with(&mut c, rules());
    let mcv = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    s.loaded.import = import(extra);
    s.strings = Some(table());
    let me = s.play.me();
    let at = s.play.world().obj(mcv).unwrap().pos;
    glasnost_sim::econ::shroud::area_reveal(&mut s.play.game.world, me, at, 8);
    c
}

/// The other player's house.
fn other(s: &Session) -> HouseId {
    let me = s.play.me();
    s.play
        .world()
        .objects
        .values()
        .find_map(|o| o.owner.filter(|&h| h != me))
        .expect("another house's object")
}

/// An object of type `name` for `house`, `dx` cells beside the cell of
/// the local base unit `base`, and its cell.
fn spawn(
    s: &mut Session,
    base: ObjectId,
    kind: Kind,
    name: &str,
    house: HouseId,
    dx: i32,
) -> (ObjectId, CellXY) {
    let b = cell_of(s, base);
    let at = CellXY::new(b.x + dx, b.y + 2);
    let id = testkit::spawn(&mut s.play.game.world, kind, name, house.0, at, 0);
    (id, cell_of(s, id))
}

/// The cell under an object.
fn cell_of(s: &Session, id: ObjectId) -> CellXY {
    s.play.world().obj(id).unwrap().pos.cell()
}

/// spec: S-0111 claim 21 (the shroud label on a cell the local house has not explored, whatever
/// it holds; nothing with no pointer object; the name for an own object, a cloaked one included;
/// for another house's: nothing when fully cloaked and not sensed, or for a type with the
/// invisible flag, a building's invisible-in-game key included) and S-0004 claim 17 (the text the
/// label resolves to; empty for a type with no label; a missing entry gives the placeholder,
/// S-0015 claim 8, and no table read our stand-in)
#[test]
fn claim_21_which_objects_get_a_tip() {
    let mut c = game(&[
        ("GUNR", "Invisible=yes"),
        ("POWR", "InvisibleInGame=yes"),
        ("LNCH", "-"),
    ]);
    let mcv = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    let me = s.play.me();
    let them = other(s);
    let at = cell_of(s, mcv);
    assert_eq!(s.name_tip_text(at, Some(mcv)), u("the mcv"), "own");
    // An unexplored cell gives the shroud label, whatever it holds.
    let w = s.play.world();
    let dark = s
        .loaded
        .map
        .cells
        .iter()
        .map(|(c, _)| c)
        .find(|&c| !glasnost_sim::econ::shroud::explored(w, me, c))
        .expect("an unexplored cell");
    assert_eq!(s.name_tip_text(dark, None), u("unknown ground"));
    assert_eq!(s.name_tip_text(dark, Some(mcv)), u("unknown ground"));
    // An explored cell without an object: nothing.
    assert!(s.name_tip_text(at, None).is_empty());
    // Another house's vehicle, and the invisible flag.
    let (minr, cm) = spawn(s, mcv, Kind::Vehicle, "MINR", them, 1);
    assert_eq!(s.name_tip_text(cm, Some(minr)), u("the minr"));
    let (gunr, cg) = spawn(s, mcv, Kind::Vehicle, "GUNR", them, 2);
    assert!(s.name_tip_text(cg, Some(gunr)).is_empty(), "invisible");
    let (my_gunr, cmg) = spawn(s, mcv, Kind::Vehicle, "GUNR", me, 3);
    assert_eq!(
        s.name_tip_text(cmg, Some(my_gunr)),
        u("the gunr"),
        "own: the flag does not apply"
    );
    let (powr, cp) = spawn(s, mcv, Kind::Building, "POWR", them, -4);
    assert!(
        s.name_tip_text(cp, Some(powr)).is_empty(),
        "invisible in game"
    );
    // Fully cloaked and not sensed: nothing; an own cloaked object: its name.
    s.play.game.world.obj_mut(minr).unwrap().cloak.state = CloakState::Cloaked;
    assert!(s.name_tip_text(cm, Some(minr)).is_empty(), "cloaked");
    s.play.game.world.obj_mut(minr).unwrap().cloak.state = CloakState::Cloaking;
    assert_eq!(s.name_tip_text(cm, Some(minr)), u("the minr"), "cloaking");
    s.play.game.world.obj_mut(mcv).unwrap().cloak.state = CloakState::Cloaked;
    assert_eq!(s.name_tip_text(at, Some(mcv)), u("the mcv"), "own cloaked");
    // A type with no label: empty text, so no tip.
    let (lnch, cl) = spawn(s, mcv, Kind::Vehicle, "LNCH", me, 5);
    assert!(s.name_tip_text(cl, Some(lnch)).is_empty(), "no label");
    // A missing entry: the placeholder (S-0015 claim 8). No table read: our stand-ins.
    s.strings = Some(StringTable::new(Vec::new()));
    assert_eq!(s.name_tip_text(cm, Some(minr)), u("MISSING:'N:MINR'"));
    assert_eq!(s.name_tip_text(dark, None), u("MISSING:'TXT_SHADOW'"));
    s.strings = None;
    assert_eq!(s.name_tip_text(cm, Some(minr)), u("MINR"));
}

/// spec: S-0111 claim 22 (an infantry unit shows its disguise's name when disguised and its owner
/// does not ally the local house, an own one its own; a vehicle that appears disguised, S-0516
/// claim 35, shows nothing for a disguise without a house and its disguise's name otherwise; a
/// building keeps its own name)
#[test]
fn claim_22_the_name_by_kind_and_disguise() {
    let mut c = game(&[]);
    let mcv = own(&c, Kind::Vehicle)[0];
    let s = session_mut(&mut c);
    let me = s.play.me();
    let them = other(s);
    let gunr = s
        .play
        .world()
        .rules()
        .index_of(Kind::Vehicle, "GUNR")
        .unwrap();
    let walker = s
        .play
        .world()
        .rules()
        .index_of(Kind::Infantry, "WALKER")
        .unwrap();
    let tick = s.play.world().tick;
    let disguise = |what, house| Disguise {
        what: Some(what),
        house,
        since: tick,
    };
    let (theirs, ct) = spawn(s, mcv, Kind::Infantry, "WALKER", them, 1);
    let (mine, cm) = spawn(s, mcv, Kind::Infantry, "WALKER", me, 2);
    for id in [theirs, mine] {
        s.play.game.world.obj_mut(id).unwrap().cloak.disguise =
            Some(disguise(DisguiseAs::Vehicle(gunr), Some(them)));
    }
    assert_eq!(
        s.name_tip_text(ct, Some(theirs)),
        u("the gunr"),
        "disguised enemy"
    );
    assert_eq!(s.name_tip_text(cm, Some(mine)), u("the walker"), "own");
    // A disguised enemy vehicle.
    let (car, cc) = spawn(s, mcv, Kind::Vehicle, "MINR", them, 3);
    s.play.game.world.obj_mut(car).unwrap().cloak.disguise =
        Some(disguise(DisguiseAs::Infantry(walker), None));
    assert!(glasnost_sim::cloak::appears_disguised(
        s.play.world(),
        car,
        Some(me)
    ));
    assert!(
        s.name_tip_text(cc, Some(car)).is_empty(),
        "a disguise without a house"
    );
    s.play.game.world.obj_mut(car).unwrap().cloak.disguise =
        Some(disguise(DisguiseAs::Infantry(walker), Some(me)));
    assert_eq!(s.name_tip_text(cc, Some(car)), u("the walker"));
    // A building's disguise does not change its name.
    let (powr, cp) = spawn(s, mcv, Kind::Building, "POWR", them, -4);
    s.play.game.world.obj_mut(powr).unwrap().cloak.disguise =
        Some(disguise(DisguiseAs::Vehicle(gunr), Some(me)));
    assert_eq!(s.name_tip_text(cp, Some(powr)), u("the powr"));
}

/// spec: S-0111 claim 23 (a turret-tip vehicle: the modifier, a space and its own type's name;
/// label A with no passenger; with an infantry P at position 1, the table by t and the own-name
/// flag u) and S-0105 claim 43 (a gunner type's t, the turret table entry for its weapon number)
#[test]
fn claim_23_the_turret_tip_modifier() {
    for own_name in [false, true] {
        let walker = if own_name {
            ("WALKER", "UseOwnName=yes")
        } else {
            ("WALKER", "Cost=1")
        };
        let mut c = game(&[("TRUK", "HasTurretTooltips=yes"), walker]);
        let mcv = own(&c, Kind::Vehicle)[0];
        let s = session_mut(&mut c);
        let me = s.play.me();
        // weapon number w → index w for 0 … 3, 4 → 7 (another value), 5 → none.
        let mut t = EMPTY;
        for (w, k) in [(0, 0), (1, 1), (2, 2), (3, 3), (4, 7)] {
            t[w] = Some(k);
        }
        let truk_idx = 4;
        s.loaded.import.turret_tables.insert(truk_idx, t);
        let (truk, ct) = spawn(s, mcv, Kind::Vehicle, "TRUK", me, 1);
        let text = |s: &Session| String::from_utf16(&s.name_tip_text(ct, Some(truk))).unwrap();
        assert_eq!(text(s), "label-a the truk", "no passenger");
        let (p, _) = spawn(s, mcv, Kind::Infantry, "WALKER", me, 2);
        let (q, _) = spawn(s, mcv, Kind::Vehicle, "MINR", me, 3);
        // A P that is not infantry: label A.
        s.play
            .game
            .world
            .obj_mut(truk)
            .unwrap()
            .transport
            .passengers = vec![q, p];
        assert_eq!(text(s), "label-a the truk", "P not infantry");
        s.play
            .game
            .world
            .obj_mut(truk)
            .unwrap()
            .transport
            .passengers = vec![p, q];
        let pn = if own_name { "the walker" } else { "label-a" };
        let b = if own_name { "the walker" } else { "label-b" };
        for (w, want) in [
            (0, "label-a"),
            (1, b),
            (2, "label-c"),
            (3, "the walker"),
            (4, pn),
            (5, pn),
        ] {
            glasnost_sim::combat::weapon::set_weapon_number(&mut s.play.game.world, truk, w);
            assert_eq!(
                text(s),
                format!("{want} the truk"),
                "weapon {w}, u {own_name}"
            );
        }
        // An empty own name still leaves the space.
        s.strings = Some(StringTable::new(vec![
            (b"N:TRUK".to_vec(), Vec::new()),
            (b"Tip:Repair".to_vec(), u("label-c")),
        ]));
        glasnost_sim::combat::weapon::set_weapon_number(&mut s.play.game.world, truk, 2);
        assert_eq!(text(s), "label-c ", "u {own_name}");
    }
}

/// spec: S-0111 claims 20, 24 and 25 (through the client: the wait on the battlefield view, the
/// box 16 pixels below the pointer, the tip colour by side index; a movement hides it) and
/// claim 20 (with the option off no tip shows)
#[test]
fn claims_20_24_25_a_tip_shows_after_the_wait() {
    let mut c = game(&[]);
    let font = glasnost_fmt::fnt::synthetic_font();
    session_mut(&mut c).play.status_font = Some(status_font::StatusFont::new(font).unwrap());
    let s = session_mut(&mut c);
    // The order block of a game's first ticks blocks the tip (claim 20, r40).
    while s.play.game.world.tick <= s.play.game.world.orders_blocked_through {
        s.step();
    }
    let m = s.map_rect();
    let p = m.centre();
    let (cell, id) = s.under(p);
    let want = s.name_tip_text(cell, id);
    let mut clock = 10_000;
    c.update(clock);
    c.pointer_moved(p);
    let _ = c.frame();
    clock += name_tip::WAIT_MS - 1;
    c.update(clock);
    let _ = c.frame();
    assert!(
        session_mut(&mut c).name_tip_shown().is_none(),
        "the wait runs"
    );
    clock += 1;
    c.update(clock);
    let _ = c.frame();
    let shown = session_mut(&mut c).name_tip_shown().cloned();
    if want.is_empty() {
        assert!(shown.is_none(), "an empty text shows nothing");
    } else {
        let s = shown.expect("the tip shows");
        assert_eq!(s.lines.concat(), want);
        assert_eq!((s.rect.x, s.rect.y), (p.x, p.y + name_tip::BELOW));
        assert!(s.ink == name_tip::INK_SIDE_0 || s.ink == name_tip::INK_OTHER);
    }
    c.pointer_moved(Point::new(p.x + 1, p.y));
    let _ = c.frame();
    assert!(
        session_mut(&mut c).name_tip_shown().is_none(),
        "a movement hides it"
    );
    // The option off: no tip after the wait.
    c.settings.tool_tips = false;
    clock += name_tip::WAIT_MS;
    c.update(clock);
    let _ = c.frame();
    assert!(
        session_mut(&mut c).name_tip_shown().is_none(),
        "the option is off"
    );
}

/// A game with a font where the order block of the first ticks is over, and
/// a point on the battlefield view whose pointer place has a tip text.
fn tip_game() -> (Client, Point) {
    let (mut c, p) = font_game();
    let s = session_mut(&mut c);
    while s.play.game.world.tick <= s.play.game.world.orders_blocked_through {
        s.step();
    }
    (c, p)
}

/// A game with a font, still inside the order block of its first ticks, and
/// a point on the battlefield view whose pointer place has a tip text.
fn font_game() -> (Client, Point) {
    let mut c = game(&[]);
    let font = glasnost_fmt::fnt::synthetic_font();
    let s = session_mut(&mut c);
    s.play.status_font = Some(status_font::StatusFont::new(font).unwrap());
    let m = s.map_rect();
    // Away from the view's edges, where the pointer would scroll it.
    let p = (60..m.h - 60)
        .step_by(7)
        .flat_map(|dy| {
            (60..m.w - 60)
                .step_by(7)
                .map(move |dx| Point::new(m.x + dx, m.y + dy))
        })
        .find(|&p| {
            let (cell, id) = s.under(p);
            !s.name_tip_text(cell, id).is_empty()
        })
        .expect("a point with a tip text");
    (c, p)
}

/// One frame at real time `*clock + ms`.
fn frame_at(c: &mut Client, clock: &mut i64, ms: i64) {
    *clock += ms;
    c.update(*clock);
    let _ = c.frame();
}

fn shown(c: &mut Client) -> Option<name_tip::Shown> {
    session_mut(c).name_tip_shown().cloned()
}

/// spec: S-0111 claim 20 (r40, Wait: the pointer leaving the window is not an event of its own;
/// the last movement restarts the wait as usual, and a wait that ends with the pointer outside the
/// window shows nothing and uses up the rest's choice)
#[test]
fn claim_20_the_pointer_leaving_the_window() {
    let (mut c, p) = tip_game();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 400);
    c.pointer_left();
    frame_at(&mut c, &mut t, 600);
    assert!(shown(&mut c).is_none(), "outside the window: nothing shows");
    frame_at(&mut c, &mut t, 3000);
    assert!(shown(&mut c).is_none(), "the rest's choice is used up");
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 999);
    assert!(shown(&mut c).is_none());
    frame_at(&mut c, &mut t, 1);
    assert!(shown(&mut c).is_some(), "back inside, moved and rested");
}

/// spec: S-0111 claim 20 (r40, Blocked: a left or right button pressed over the battlefield view
/// and held blocks the tip; the release brings nothing back until the pointer moves and rests
/// again; a middle button is no held button) and claim 26 (a press or release hides it)
#[test]
fn claim_20_a_held_button_blocks() {
    for b in [Button::Left, Button::Right] {
        let (mut c, p) = tip_game();
        let mut t = 10_000;
        frame_at(&mut c, &mut t, 0);
        c.pointer_down(b, p);
        c.pointer_moved(Point::new(p.x + 1, p.y));
        frame_at(&mut c, &mut t, 1000);
        assert!(shown(&mut c).is_none(), "{b:?} held: blocked");
        c.pointer_up(b, Point::new(p.x + 1, p.y), Default::default());
        frame_at(&mut c, &mut t, 3000);
        assert!(
            shown(&mut c).is_none(),
            "{b:?} released: nothing until a rest"
        );
        c.pointer_moved(p);
        frame_at(&mut c, &mut t, 1000);
        assert!(shown(&mut c).is_some(), "{b:?}: moved and rested");
    }
    let (mut c, p) = tip_game();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    c.middle_button(p);
    c.pointer_moved(Point::new(p.x + 1, p.y));
    frame_at(&mut c, &mut t, 1000);
    assert!(shown(&mut c).is_some(), "a middle button blocks nothing");
}

/// How a route-mode run of S-0111 scenario NT1 (h) presses: the button,
/// the selection and the press place.
#[derive(Clone, Copy, Debug)]
enum RoutePress {
    /// An ordinary left click outside every point's click area, a GUNR
    /// with no route selected (runs 1 and 3).
    OrdinaryLeft,
    /// The right button, the same GUNR selected (run 2).
    Right,
    /// A building selected, a left press outside every click area: refused
    /// for kind at the press (run 4).
    KindRefused,
    /// The route holder selected, a left press on its last point: refused
    /// by the point-click test (run 5).
    PointRefused,
    /// The GUNR with no route selected, a left press on the holder's first
    /// point: it takes the point, a join (run 6).
    TakesPoint,
    /// Nothing selected, a left press on the holder's second point: it
    /// takes the own point, which the click selects (S-0213 claim 16e).
    SelectsPoint,
    /// The route holder selected, a left press on its own first point: it
    /// takes the point, a loop close.
    LoopClose,
    /// The holder and a GUNR with no route selected, a left press outside
    /// every click area: an ordinary click, refused only at its release by
    /// the same-route-state test (S-0213 claim 16d).
    MixedAtRelease,
    /// A GUNR with no route selected and the house at the holder limit, a
    /// left press outside every click area: an ordinary click, refused
    /// only at its release by the holder limit (S-0213 claim 16).
    LimitAtRelease,
}

/// One run: a tip game with `routed`'s three GUNR (the first holding three
/// points) and the route-mode key held; the press of `how`, a movement
/// with the button held onto a place with a tip text (`p`, outside every
/// click area; one pixel from the press for a press there), a rest of
/// 1000 ms, the release there, a rest of 3000 ms, then a movement and a
/// rest of 1000 ms. Returns whether the tip shows at the end of each rest.
fn route_mode_run(how: RoutePress) -> [bool; 3] {
    let (mut c, p) = tip_game();
    let (ids, pts) = super::play_notes_tests::routed(&mut c);
    let s = session_mut(&mut c);
    let (cell, id) = s.under(p);
    assert!(!s.name_tip_text(cell, id).is_empty(), "set-up: a tip text");
    let m = s.map_rect();
    let screens: Vec<Point> = pts
        .iter()
        .map(|&n| s.route_point_screen(n).expect("set-up: a point"))
        .collect();
    for q in &screens {
        assert!(m.contains(*q), "set-up: the point is on the view");
        // S-0213 claim 16b, Click areas: 8 pixels right of a place is outside its area.
        assert!(
            (q.x - p.x).abs() > 16 || (q.y - p.y).abs() > 16,
            "set-up: the rest place is outside every click area"
        );
    }
    let (select, button, at) = match how {
        RoutePress::OrdinaryLeft => (vec![ids[1]], Button::Left, p),
        RoutePress::Right => (vec![ids[1]], Button::Right, p),
        RoutePress::KindRefused => {
            let local = s.play.me();
            let w = &mut s.play.game.world;
            let t = w
                .rules()
                .index_of(Kind::Building, "POWR")
                .expect("fixture type");
            let hut = glasnost_sim::objects::create(w, Kind::Building, t, Some(local));
            let cell = CellXY::new(4, 4);
            let z = w.surface_at(256 * cell.x + 128, 256 * cell.y + 128);
            glasnost_sim::objects::place(w, hut, glasnost_sim::world::Pos::centre(cell, z), 0);
            (vec![hut], Button::Left, p)
        }
        RoutePress::PointRefused => (vec![ids[0]], Button::Left, screens[2]),
        RoutePress::TakesPoint => (vec![ids[1]], Button::Left, screens[0]),
        RoutePress::SelectsPoint => (Vec::new(), Button::Left, screens[1]),
        RoutePress::LoopClose => (vec![ids[0]], Button::Left, screens[0]),
        RoutePress::MixedAtRelease => (vec![ids[0], ids[1]], Button::Left, p),
        RoutePress::LimitAtRelease => {
            let local = s.play.me();
            s.play
                .game
                .world
                .routes
                .counts
                .insert(local, glasnost_sim::routes::HOLDER_LIMIT);
            (vec![ids[1]], Button::Left, p)
        }
    };
    let s = session_mut(&mut c);
    s.play.selection = select;
    c.key(Key::Char('z'), true);
    assert!(session_mut(&mut c).route_held(), "set-up: route mode");
    let rest = if at == p { Point::new(p.x + 1, p.y) } else { p };
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    c.pointer_moved(at);
    c.pointer_down(button, at);
    c.pointer_moved(rest);
    frame_at(&mut c, &mut t, 1000);
    let held = shown(&mut c).is_some();
    c.pointer_up(button, rest, Default::default());
    assert!(shown(&mut c).is_none(), "{how:?}: the release hides it");
    frame_at(&mut c, &mut t, 3000);
    let released = shown(&mut c).is_some();
    c.pointer_moved(Point::new(rest.x, rest.y + 1));
    frame_at(&mut c, &mut t, 1000);
    let again = shown(&mut c).is_some();
    assert!(
        session_mut(&mut c).route_held(),
        "{how:?}: still route mode"
    );
    [held, released, again]
}

/// spec: S-0111 r49 claim 20 (Blocked, Q-2736 answered: in the route mode, S-0213 claim 16, a held
/// right button blocks the tip as outside it, and so does a held left button whose press is an
/// ordinary click outside every point's click area, S-0213 claim 16b; the block ends at the
/// button's release, in the route mode too) and scenario NT1 (h) runs 1 to 3
#[test]
fn claim_20_route_mode_an_ordinary_press_or_the_right_button_blocks() {
    for how in [RoutePress::OrdinaryLeft, RoutePress::Right] {
        assert_eq!(
            route_mode_run(how),
            [false, false, true],
            "{how:?}: blocked while held, nothing until a rest, the block ends at the release"
        );
    }
}

/// spec: S-0111 r49 claim 20 (Blocked, Q-2736 answered: a press that the route mode refuses only
/// at its release, by the same-route-state test or the holder limit, S-0213 claims 16 and 16d, is
/// an ordinary click outside every point's click area, so its held left button blocks the tip; the
/// block ends at the release) (seam sweep 6: intake 111's row)
#[test]
fn seams6_route_mode_a_press_refused_only_at_its_release_blocks() {
    for how in [RoutePress::MixedAtRelease, RoutePress::LimitAtRelease] {
        assert_eq!(
            route_mode_run(how),
            [false, false, true],
            "{how:?}: blocked while held, nothing until a rest, the block ends at the release"
        );
    }
}

/// spec: S-0111 r49 claim 20 (Blocked, Q-2736 answered: a held left button does not block the tip
/// when its press took a point, S-0213 claim 16b, here the selected holder's own first point, a
/// loop close; after a movement with the button held the tip shows once the pointer has rested
/// 1000 ms, and the release hides it) (seam sweep 6: intake 111's row)
#[test]
fn seams6_route_mode_a_loop_close_does_not_block() {
    assert_eq!(
        route_mode_run(RoutePress::LoopClose),
        [true, false, true],
        "a tip with the button held, hidden at the release"
    );
}

/// spec: S-0111 r49 claim 20 (Blocked, Q-2736 answered: a held left button does not block the tip
/// when the route mode refused its press at the press, for kind, S-0213 claim 16d, or by the
/// point-click test, S-0213 claim 16b, or when its press took a point, S-0213 claim 16b, or, with
/// no object selected, took an own point that the click selects, S-0213 claims 16b and 16e; after a
/// movement with the button held the tip shows once the pointer has rested 1000 ms, and the release
/// hides it) and scenario NT1 (h) runs 4 to 6
#[test]
fn claim_20_route_mode_a_press_refused_or_taking_a_point_does_not_block() {
    for how in [
        RoutePress::KindRefused,
        RoutePress::PointRefused,
        RoutePress::TakesPoint,
        RoutePress::SelectsPoint,
    ] {
        assert_eq!(
            route_mode_run(how),
            [true, false, true],
            "{how:?}: a tip with the button held, hidden at the release"
        );
    }
}

/// spec: S-0111 claim 20 (r40, Blocked: an order block locks the input and hides the cursor, as
/// the block of a game's first ticks, S-0201 claim 14: a wait that ends in it shows nothing, and
/// the block ending brings nothing back)
#[test]
fn claim_20_the_order_block() {
    let (mut c, p) = font_game();
    let s = session_mut(&mut c);
    assert!(s.play.game.world.tick <= s.play.game.world.orders_blocked_through);
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 1000);
    assert!(shown(&mut c).is_none(), "blocked at the end of the wait");
    let s = session_mut(&mut c);
    while s.play.game.world.tick <= s.play.game.world.orders_blocked_through {
        s.step();
    }
    frame_at(&mut c, &mut t, 3000);
    assert!(
        shown(&mut c).is_none(),
        "the block ending brings nothing back"
    );
    c.pointer_moved(Point::new(p.x + 1, p.y));
    frame_at(&mut c, &mut t, 1000);
    assert!(shown(&mut c).is_some(), "moved and rested after the block");
}

/// #84. spec: S-0111 claim 20 (Blocked: while the battlefield view is moving, as during a scroll)
/// and S-0101 claims 13 to 15 (scrolling is requested once per unfrozen cycle, not per drawn frame)
///
/// A held arrow scrolls the view at every third drawn frame; a wait that ends in a frame between
/// two cycles is blocked too.
#[test]
fn claim_20_a_scroll_blocks_between_its_cycles() {
    let (mut c, p) = tip_game();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    let s = session_mut(&mut c);
    // The held arrow with the most room to scroll from here.
    let start = s.centre();
    let held = (0..4)
        .map(|k| {
            let mut h = [false; 4];
            h[k] = true;
            h
        })
        .max_by_key(|&h| {
            s.set_centre(start);
            for _ in 0..40 {
                s.scroll_cycle(t as u64, h, false);
            }
            let d = s.centre();
            (d.0 - start.0).abs() + (d.1 - start.1).abs()
        })
        .unwrap();
    s.set_centre(start);
    s.update_marks();
    c.pointer_moved(p);
    let s = session_mut(&mut c);
    for k in 0..90 {
        t += 16;
        s.clock_ms = t;
        if k % 3 == 0 {
            let before = s.centre();
            s.scroll_cycle(t as u64, held, false);
            assert_ne!(s.centre(), before, "the view moves at frame {k}");
        }
        s.update_marks();
        assert!(
            s.name_tip_shown().is_none(),
            "frame {k}: the view is moving"
        );
    }
}

/// spec: S-0111 claim 20 (r40, Options menu: opening it hides a tip and stops the wait; while it
/// is open no wait runs; when it closes nothing shows until the pointer moves and rests again)
#[test]
fn claim_20_the_options_menu() {
    let (mut c, p) = tip_game();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 1000);
    assert!(shown(&mut c).is_some());
    assert!(session_mut(&mut c).open_options());
    assert!(shown(&mut c).is_none(), "opening hides it");
    frame_at(&mut c, &mut t, 3000);
    assert!(shown(&mut c).is_none(), "no wait runs while it is open");
    session_mut(&mut c).close_options();
    frame_at(&mut c, &mut t, 3000);
    assert!(
        shown(&mut c).is_none(),
        "a tip hidden by the menu does not come back"
    );
    c.pointer_moved(Point::new(p.x, p.y + 1));
    frame_at(&mut c, &mut t, 1000);
    assert!(shown(&mut c).is_some(), "moved and rested after it closed");
}

/// spec: S-0111 claim 26 (r40: while the tip shows, its text is chosen again in every drawn frame:
/// another text replaces it, laid out and placed again, with the lifetime unchanged; the same
/// text leaves it; no text hides it and nothing shows until the pointer moves and rests again)
/// and S-0015 claim 8 (a label no entry matches shows as the placeholder)
#[test]
fn claim_26_the_text_is_chosen_again() {
    let (mut c, p) = tip_game();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 1000);
    let first = shown(&mut c).expect("a tip");
    // The same text in the next frames: as it is.
    frame_at(&mut c, &mut t, 100);
    assert_eq!(shown(&mut c), Some(first.clone()));
    // The entries change under the resting pointer: another text.
    session_mut(&mut c).strings = Some(StringTable::new(Vec::new()));
    frame_at(&mut c, &mut t, 2000);
    let second = shown(&mut c).expect("the tip stays, with another text");
    assert_ne!(second.text, first.text);
    assert!(second.text.starts_with(&u("MISSING:'")));
    assert_eq!(
        second.until, first.until,
        "the lifetime does not start again"
    );
    assert_eq!((second.rect.x, second.rect.y), (p.x, p.y + name_tip::BELOW));
    // No text: it hides, and nothing shows until the pointer moves and rests again.
    let empty = table()
        .labels()
        .map(|l| (l.to_vec(), Vec::new()))
        .collect::<Vec<_>>();
    session_mut(&mut c).strings = Some(StringTable::new(empty));
    frame_at(&mut c, &mut t, 100);
    assert!(shown(&mut c).is_none(), "no text hides it");
    session_mut(&mut c).strings = Some(table());
    frame_at(&mut c, &mut t, 3000);
    assert!(shown(&mut c).is_none(), "nothing until a rest");
    c.pointer_moved(Point::new(p.x, p.y + 1));
    frame_at(&mut c, &mut t, 1000);
    assert!(shown(&mut c).is_some());
}
