//! The name tip's text (S-0111 claims 20 to 23) and the display names it
//! shows (S-0004 claim 17) through the windowless client on synthetic data:
//! the invented rules and map of `glasnost_ai::fixture`, an invented rules
//! text for the labels and an invented string table. Every label and text
//! here is made up.

use std::sync::Arc;

use super::access_tests::{client, own, session_mut, start_by_keyboard_with};
use super::*;
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
/// label resolves to; empty for a type with no label; a missing entry and no table read give our
/// stand-in, Q-2686)
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
    // A missing entry, and no table read: our stand-ins (Q-2686).
    s.strings = Some(StringTable::new(Vec::new()));
    assert_eq!(s.name_tip_text(cm, Some(minr)), u("MINR"));
    assert_eq!(s.name_tip_text(dark, None), u("TXT_SHADOW"));
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
