//! The sidebar's tips (S-0411 claims 20 to 22, S-0111 claim 20's
//! exception over the build-icon slots) through the windowless client on
//! synthetic data: the invented rules and map of `glasnost_ai::fixture`, an
//! invented rules text for the labels and an invented string table. Every
//! label, format and text here is made up.

use super::access_tests::{client, deploy_base, session, session_mut, start_by_keyboard};
use super::*;
use crate::game::play::Button;
use crate::game::sidebar_tip;
use crate::game::strings::StringTable;
use crate::game::{name_tip, status_font};
use crate::mapview::live;
use glasnost_convert::inputs::{InputFile, Inputs};
use glasnost_convert::{ImportOptions, convert};
use glasnost_sim::econ::TypeRef;
use glasnost_sim::rules::Kind;
use glasnost_ui::hud::HudHit;

fn u(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// The icon types of every tab of the local house, in tab and icon order.
fn icon_types(c: &Client) -> Vec<TypeRef> {
    let p = &session(c).play;
    p.world().houses[usize::from(p.me().0)]
        .econ
        .tech
        .icons
        .iter()
        .flatten()
        .map(|i| i.t)
        .collect()
}

/// An import of the icon types `types` (of `c`'s rules), each with the
/// label `N:<name>`.
fn import(c: &Client, types: &[TypeRef]) -> glasnost_convert::Import {
    let rules = session(c).play.world().rules_arc();
    let mut text = String::new();
    let mut sections = String::new();
    for (list, kind) in [
        ("BuildingTypes", Kind::Building),
        ("InfantryTypes", Kind::Infantry),
        ("VehicleTypes", Kind::Vehicle),
        ("AircraftTypes", Kind::Aircraft),
    ] {
        text += &format!("[{list}]\n");
        for (i, t) in types.iter().filter(|t| t.kind == kind).enumerate() {
            let name = live::type_name(&rules, t.kind, t.index);
            text += &format!("{i}={name}\n");
            sections += &format!("[{name}]\nUIName=N:{name}\n");
        }
    }
    text += &sections;
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

/// The display name the table gives type `name`: two words.
fn display(name: &str) -> String {
    format!("the {}", name.to_lowercase())
}

/// A string table with a two-word display name for each of `names`, an
/// invented money format, power format and labels for the controls.
fn table(names: &[String]) -> StringTable {
    let mut entries: Vec<(Vec<u8>, Vec<u16>)> = names
        .iter()
        .map(|n| (format!("N:{n}").into_bytes(), u(&display(n))))
        .collect();
    for (l, t) in [
        ("TXT_MONEY_FORMAT_2", "<%s|%d>"),
        ("TXT_POWER_DRAIN", "pow %d of %d"),
        ("TXT_REPAIR_MODE", "fix it"),
        ("TXT_SELL_MODE", "sell it"),
        ("Tip:Tab1", "tab one"),
        ("Tip:Tab2", "tab two"),
        ("Tip:Tab3", "tab three"),
        ("Tip:Tab4", "tab four"),
        ("Tip:ScrollUp", "go up"),
        ("Tip:ScrollDown", "go down"),
        ("Tip:Disabled", "not now"),
        ("Tip:OptionsButton", "the menu"),
    ] {
        entries.push((l.as_bytes().to_vec(), u(t)));
    }
    StringTable::new(entries)
}

/// A game at `size` and interface `scale` with the base deployed, a font,
/// the import and table above, and the order block of the first ticks over.
fn bar_game(size: (i32, i32), scale: u16) -> Client {
    let mut c = client(size, scale);
    start_by_keyboard(&mut c);
    deploy_base(&mut c);
    let types = icon_types(&c);
    assert!(!types.is_empty(), "building icons after the deploy");
    let import = import(&c, &types);
    let rules = session(&c).play.world().rules_arc();
    let names: Vec<String> = types
        .iter()
        .map(|t| live::type_name(&rules, t.kind, t.index))
        .collect();
    let s = session_mut(&mut c);
    s.loaded.import = import;
    s.strings = Some(table(&names));
    let font = glasnost_fmt::fnt::synthetic_font();
    s.play.status_font = Some(status_font::StatusFont::new(font).unwrap());
    while s.play.game.world.tick <= s.play.game.world.orders_blocked_through {
        s.step();
    }
    let _ = c.frame();
    c
}

/// One frame at real time `*clock + ms`.
fn frame_at(c: &mut Client, clock: &mut i64, ms: i64) {
    *clock += ms;
    c.update(*clock);
    let _ = c.frame();
}

fn shown(c: &Client) -> Option<name_tip::Shown> {
    session(c).name_tip_shown().cloned()
}

/// The screen rectangle of logical rectangle `r`.
fn screen(c: &Client, r: Rect) -> Rect {
    session(c).ui_scale().rect(r)
}

/// The middle of logical rectangle `r` on the screen.
fn mid(c: &Client, r: Rect) -> Point {
    let r = screen(c, r);
    Point::new(r.x + r.w / 2, r.y + r.h / 2)
}

/// The shown slots' icon types: slot k's, or `None` for an empty slot.
fn slots(c: &Client) -> Vec<Option<TypeRef>> {
    let s = session(c);
    let local = s.play.me();
    let es = glasnost_ui::hud::entries(s.play.world(), local, s.play.shown_tab());
    let rows = s.layout().rows();
    let first = s
        .play
        .scroll
        .min(glasnost_ui::hud::max_scroll(es.len(), rows))
        * 2;
    (0..rows * 2)
        .map(|k| match es.get(first + k) {
            Some(glasnost_ui::hud::Entry::Type(t)) => Some(*t),
            _ => None,
        })
        .collect()
}

/// The tip text claim 20 gives icon type `t` with the invented format.
fn icon_text(c: &Client, t: TypeRef) -> Vec<u16> {
    let s = session(c);
    let w = s.play.world();
    let name = live::type_name(w.rules(), t.kind, t.index);
    let price = glasnost_sim::econ::production::price(w, s.play.me(), t);
    u(&format!("<{}|{price}>", display(&name)).replace(' ', "\n"))
}

/// The screen tip area and sidebar.
fn area(c: &Client) -> (Rect, Rect) {
    let s = session(c);
    let m = s.map_rect();
    let bar = Rect::new(m.right(), 0, s.screen.0 - m.right(), s.screen.1);
    (bar, screen(c, s.layout().tip_area()).intersect(&bar))
}

/// spec: S-0411 claim 20 (Over the slots, no wait: the movement onto the slots waits 1000 ms;
/// once a drawn frame has passed there, a movement ending on a slot shows its tip at once, the
/// lifetime from that movement; one ending on an empty slot hides it; Text: the money format
/// with the display name and the price P, every space a line feed) and claim 21 (placed by the
/// slot in the tip area, cut at the sidebar) and S-0111 claim 20 (the exception)
#[test]
fn claims_20_and_21_an_icon_tip() {
    let mut c = bar_game((800, 600), 100);
    let l = session(&c).layout();
    let t0 = slots(&c)[0].expect("an icon in the first slot");
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    let p = mid(&c, l.cameo(0));
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_none(), "the movement onto the slots waits");
    frame_at(&mut c, &mut t, 983);
    assert!(shown(&c).is_none());
    frame_at(&mut c, &mut t, 1);
    let s = shown(&c).expect("the tip after the wait");
    assert_eq!(s.text, icon_text(&c, t0));
    assert_eq!(
        s.until,
        t + name_tip::LIFETIME_MS,
        "from the end of the wait"
    );
    let (bar, a) = area(&c);
    assert_eq!(s.clip, bar);
    let size = (s.rect.w, s.rect.h);
    let slot = screen(&c, l.cameo(0));
    assert_eq!((s.rect.x, s.rect.y), sidebar_tip::slot_place(slot, size, a));
    // A movement within the icon: at once, the lifetime started again.
    t += 500;
    c.update(t);
    c.pointer_moved(Point::new(p.x + 1, p.y));
    let moved = t;
    frame_at(&mut c, &mut t, 16);
    let again = shown(&c).expect("shown at once");
    assert_eq!(again.text, s.text);
    assert_eq!(again.rect, s.rect, "placed by the slot, not the pointer");
    assert_eq!(again.until, moved + name_tip::LIFETIME_MS);
    // A movement onto the next slot: its tip at once, or none for an empty slot.
    c.pointer_moved(mid(&c, l.cameo(1)));
    frame_at(&mut c, &mut t, 16);
    match slots(&c)[1] {
        Some(t1) => assert_eq!(shown(&c).expect("at once").text, icon_text(&c, t1)),
        None => assert!(shown(&c).is_none(), "an empty slot hides it"),
    }
    // The gap between two icons is on the slots: still no wait.
    let c0 = l.cameo(0);
    c.pointer_moved(screen(&c, Rect::new(c0.right() + 1, c0.y, 1, 1)).centre());
    frame_at(&mut c, &mut t, 16);
    assert_eq!(shown(&c).map(|s| s.text), Some(icon_text(&c, t0)));
    // The lifetime ends; the next movement there shows it again at once.
    frame_at(&mut c, &mut t, name_tip::LIFETIME_MS);
    assert!(shown(&c).is_none(), "the lifetime ended");
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_some(), "again at once");
    // Off the slots onto the battlefield: an ordinary movement (Edge cases: ours waits).
    let m = session(&c).map_rect();
    c.pointer_moved(Point::new(m.right() - 40, m.y + m.h / 2));
    frame_at(&mut c, &mut t, 16);
    assert!(
        shown(&c).is_none(),
        "the movement off the slots hides it and waits"
    );
    // Back onto the slots: ordinary again.
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 16);
    assert!(
        shown(&c).is_none(),
        "the movement onto the slots waits again"
    );
}

/// spec: S-0411 claim 20 (Text: the same text whatever the item's state; here idle, building and
/// on hold, S-0404 claims 3 and 5) (seam sweep 7a: intake 115's row)
#[test]
fn seams7a_claim_20_the_icon_tip_is_the_same_building_and_on_hold() {
    use glasnost_sim::econ::production::{self, Hold};
    use glasnost_sim::orders::Order;
    let mut c = bar_game((800, 600), 100);
    let l = session(&c).layout();
    let t0 = slots(&c)[0].expect("an icon in the first slot");
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    let p = mid(&c, l.cameo(0));
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 16);
    frame_at(&mut c, &mut t, 1000);
    let idle = shown(&c).expect("the tip after the wait").text;
    let me = session(&c).play.me();
    let item = |c: &Client| {
        production::line_of(session(c).play.world(), me, t0)
            .and_then(|l| l.item.as_ref())
            .filter(|i| i.t == t0)
            .map(|i| i.hold)
    };
    // The tip again after a movement within the icon, in state `what`.
    let again = |c: &mut Client, t: &mut i64, dx: i32, what: &str| {
        c.pointer_moved(Point::new(p.x + dx, p.y));
        frame_at(c, t, 16);
        let s = shown(c).unwrap_or_else(|| panic!("{what}: shown at once"));
        assert_eq!(s.text, idle, "{what}: the same text");
    };
    session_mut(&mut c).play.game.world.houses[usize::from(me.0)].credits = 100_000;
    assert_eq!(
        session_mut(&mut c)
            .play
            .issue(vec![Order::Build { what: t0 }]),
        1
    );
    for _ in 0..5 {
        session_mut(&mut c).step();
    }
    assert_eq!(item(&c), Some(Hold::None), "set-up: building");
    again(&mut c, &mut t, 1, "building");
    assert_eq!(
        session_mut(&mut c)
            .play
            .issue(vec![Order::Hold { what: t0 }]),
        1
    );
    for _ in 0..3 {
        session_mut(&mut c).step();
    }
    assert_eq!(item(&c), Some(Hold::Player), "set-up: on hold");
    again(&mut c, &mut t, 0, "on hold");
}

/// spec: S-0411 claim 20 (no drawn frame between the movement onto the slots and the next one:
/// that one is ordinary too)
#[test]
fn claim_20_two_movements_in_one_frame() {
    let mut c = bar_game((800, 600), 100);
    let l = session(&c).layout();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    let p = mid(&c, l.cameo(0));
    c.pointer_moved(p);
    c.pointer_moved(Point::new(p.x + 1, p.y));
    frame_at(&mut c, &mut t, 16);
    assert!(
        shown(&c).is_none(),
        "no drawn frame on a slot yet: the wait runs"
    );
    frame_at(&mut c, &mut t, 984);
    assert!(shown(&c).is_some());
}

/// spec: S-0411 claim 20 (Buttons and blocks: a press or release hides the tip, and over the
/// slots the next movement shows it again at once, also with a button held; the order block
/// blocks sidebar tips; a moving battlefield view does not; the options menu hides it and stops
/// the wait)
#[test]
fn claim_20_buttons_and_blocks() {
    let mut c = bar_game((800, 600), 100);
    let l = session(&c).layout();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    let p = mid(&c, l.cameo(0));
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 1000);
    assert!(shown(&c).is_some());
    // The middle button: hidden until the pointer moves, then at once.
    session_mut(&mut c).middle_button(p);
    assert!(shown(&c).is_none(), "a press hides it");
    frame_at(&mut c, &mut t, 2000);
    assert!(shown(&c).is_none(), "until the pointer moves");
    c.pointer_moved(Point::new(p.x + 1, p.y));
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_some(), "the next movement shows it at once");
    // A right press held over the slots does not stop it.
    c.pointer_down(Button::Right, Point::new(p.x + 1, p.y));
    assert!(shown(&c).is_none());
    c.pointer_moved(Point::new(p.x + 2, p.y));
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_some(), "a held button does not stop it");
    c.pointer_up(
        Button::Right,
        Point::new(p.x + 2, p.y),
        Modifiers::default(),
    );
    // A moving battlefield view blocks no sidebar tip.
    c.pointer_moved(p);
    let (x, y) = session(&c).centre();
    session_mut(&mut c).set_centre((x + 60, y + 60));
    assert_ne!(session(&c).centre(), (x, y), "the view moves");
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_some(), "the moving view does not block it");
    // An order block (the hidden cursor) blocks it.
    let s = session_mut(&mut c);
    s.play.game.world.orders_blocked_through = s.play.game.world.tick + 100;
    c.pointer_moved(Point::new(p.x + 1, p.y));
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_none(), "the order block blocks it");
    let s = session_mut(&mut c);
    s.play.game.world.orders_blocked_through = s.play.game.world.tick - 1;
    frame_at(&mut c, &mut t, 2000);
    assert!(shown(&c).is_none(), "the block ending brings nothing back");
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_some());
    // The options menu hides it and stops the wait.
    assert!(session_mut(&mut c).open_options());
    assert!(shown(&c).is_none());
    session_mut(&mut c).close_options();
    frame_at(&mut c, &mut t, 3000);
    assert!(
        shown(&c).is_none(),
        "a tip hidden by the menu does not come back"
    );
    // The option off: none.
    c.settings.tool_tips = false;
    c.pointer_moved(Point::new(p.x + 1, p.y));
    frame_at(&mut c, &mut t, 2000);
    assert!(shown(&c).is_none(), "the option is off");
}

/// spec: S-0411 claim 20 (Chosen again in every drawn frame: another text replaces it with the
/// lifetime unchanged, placed again by the slot; a slot that becomes empty hides it, and nothing
/// shows until the pointer moves again)
#[test]
fn claim_20_chosen_again() {
    let mut c = bar_game((800, 600), 100);
    let l = session(&c).layout();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    let p = mid(&c, l.cameo(0));
    c.pointer_moved(p);
    frame_at(&mut c, &mut t, 1000);
    let first = shown(&c).expect("a tip");
    // Another format under the showing tip: replaced, the lifetime kept.
    let rules = session(&c).play.world().rules_arc();
    let names: Vec<String> = icon_types(&c)
        .iter()
        .map(|ty| live::type_name(&rules, ty.kind, ty.index))
        .collect();
    let mut entries: Vec<(Vec<u8>, Vec<u16>)> = names
        .iter()
        .map(|n| (format!("N:{n}").into_bytes(), u(&display(n))))
        .collect();
    entries.push((
        b"TXT_MONEY_FORMAT_2".to_vec(),
        u("%d=%s, a much longer text"),
    ));
    session_mut(&mut c).strings = Some(StringTable::new(entries));
    frame_at(&mut c, &mut t, 100);
    let second = shown(&c).expect("another text");
    assert_ne!(second.text, first.text);
    assert_eq!(
        second.until, first.until,
        "the lifetime does not start again"
    );
    let (_, a) = area(&c);
    let slot = screen(&c, l.cameo(0));
    let size = (second.rect.w, second.rect.h);
    assert_eq!(
        (second.rect.x, second.rect.y),
        sidebar_tip::slot_place(slot, size, a)
    );
    // The slot becomes empty: hidden, and nothing until the pointer moves.
    let s = session_mut(&mut c);
    let me = usize::from(s.play.me().0);
    let icons = std::mem::take(&mut s.play.game.world.houses[me].econ.tech.icons);
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_none(), "an empty slot hides it");
    session_mut(&mut c).play.game.world.houses[me]
        .econ
        .tech
        .icons = icons;
    frame_at(&mut c, &mut t, 3000);
    assert!(
        shown(&c).is_none(),
        "nothing until the pointer moves (Edge cases)"
    );
    c.pointer_moved(Point::new(p.x + 1, p.y));
    frame_at(&mut c, &mut t, 16);
    assert!(shown(&c).is_some());
}

/// spec: S-0411 claim 22 (the controls' tips: the 1000 ms wait, placed by the pointer in the tip
/// area, the spaces kept; a tab without icons and the scroll buttons while all icons fit add the
/// disabled line; the power meter's output and drain; nothing elsewhere on the sidebar) and
/// Import mapping (the entries; a missing entry shows the placeholder)
#[test]
fn claim_22_the_controls() {
    let mut c = bar_game((800, 600), 100);
    let l = session(&c).layout();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    let mut rest = |c: &mut Client, r: Rect| {
        let p = mid(c, r);
        c.pointer_moved(p);
        frame_at(c, &mut t, 999);
        assert!(shown(c).is_none(), "the wait runs");
        frame_at(c, &mut t, 1);
        (p, shown(c))
    };
    let (p, s) = rest(&mut c, l.power);
    let s = s.expect("the power meter's tip");
    let me = session(&c).play.me();
    let (o, d) = glasnost_sim::econ::power::totals(session(&c).play.world(), me);
    assert_eq!(s.text, u(&format!("pow {o} of {d}")), "spaces kept");
    let (bar, a) = area(&c);
    assert_eq!(s.clip, bar);
    let size = (s.rect.w, s.rect.h);
    assert_eq!(
        (s.rect.x, s.rect.y),
        name_tip::place((p.x, p.y), size, (a.right(), a.bottom()))
    );
    let (_, s) = rest(&mut c, l.modes[0]);
    assert_eq!(s.unwrap().text, u("sell it"));
    let (_, s) = rest(&mut c, l.modes[1]);
    assert_eq!(s.unwrap().text, u("fix it"));
    let (_, s) = rest(&mut c, l.options);
    assert_eq!(s.unwrap().text, u("the menu"));
    let (_, s) = rest(&mut c, l.modes[2]);
    assert_eq!(s.unwrap().text, u(sidebar_tip::POWER_MODE_TIP));
    let empty_tab = (0..4)
        .find(|&i| {
            let s = session(&c);
            glasnost_ui::hud::entries(s.play.world(), me, i).is_empty()
        })
        .expect("a tab without icons");
    let (_, s) = rest(&mut c, l.tabs[empty_tab]);
    let want = ["tab one", "tab two", "tab three", "tab four"][empty_tab];
    assert_eq!(s.unwrap().text, u(&format!("{want}\n(not now)")));
    let full_tab = session(&c).play.shown_tab();
    let (_, s) = rest(&mut c, l.tabs[full_tab]);
    assert_eq!(
        s.unwrap().text,
        u(["tab one", "tab two", "tab three", "tab four"][full_tab])
    );
    let n = glasnost_ui::hud::entries(session(&c).play.world(), me, full_tab).len();
    let fits = !glasnost_ui::hud::overflows(n, l.rows());
    let (_, s) = rest(&mut c, l.scroll[0]);
    let up = if fits { "go up\n(not now)" } else { "go up" };
    assert_eq!(s.unwrap().text, u(up));
    // Nothing on the radar or the credits counter.
    let (_, s) = rest(&mut c, l.radar);
    assert!(s.is_none());
    let (_, s) = rest(&mut c, l.credits);
    assert!(s.is_none());
    // A missing entry: the placeholder.
    session_mut(&mut c).strings = Some(StringTable::new(Vec::new()));
    let (_, s) = rest(&mut c, l.modes[0]);
    assert_eq!(s.unwrap().text, u("MISSING:'TXT_SELL_MODE'"));
    let (_, s) = rest(&mut c, l.power);
    assert_eq!(s.unwrap().text, u("MISSING:'TXT_POWER_DRAIN'"));
}

/// spec: S-0411 claim 21 (the tip area and the slot in screen pixels at an interface scale other
/// than 100: the box placed by the slot, cut at the sidebar)
#[test]
fn claim_21_at_another_scale() {
    let mut c = bar_game((1024, 768), 125);
    assert_ne!(session(&c).ui_scale().percent, 100);
    let l = session(&c).layout();
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    let k = l.rows() * 2 - 1;
    c.pointer_moved(mid(&c, l.cameo(k)));
    frame_at(&mut c, &mut t, 1000);
    match slots(&c)[k] {
        None => assert!(shown(&c).is_none(), "an empty slot"),
        Some(ty) => {
            let s = shown(&c).expect("a tip");
            assert_eq!(s.text, icon_text(&c, ty));
            let (bar, a) = area(&c);
            let slot = screen(&c, l.cameo(k));
            assert_eq!(
                (s.rect.x, s.rect.y),
                sidebar_tip::slot_place(slot, (s.rect.w, s.rect.h), a)
            );
            assert!(bar.intersect(&s.rect) == s.rect || s.clip == bar);
        }
    }
    // The first slot, whose tip reaches right and down into the area.
    c.pointer_moved(mid(&c, l.cameo(0)));
    frame_at(&mut c, &mut t, 16);
    let s = shown(&c).expect("at once: a frame passed on the slots");
    let slot = screen(&c, l.cameo(0));
    assert_eq!(s.rect.x, slot.right() - sidebar_tip::OVERLAP);
    assert_eq!(s.rect.y, slot.bottom() - sidebar_tip::OVERLAP);
}

/// design: our keyboard focus on an icon shows that icon's tip while no pointer tip shows
#[test]
fn design_the_keyboard_focus_tip() {
    let mut c = bar_game((800, 600), 100);
    let mut t = 10_000;
    frame_at(&mut c, &mut t, 0);
    assert!(session(&c).focus_tip_shown().is_none());
    c.key(Key::PageDown, true);
    c.key(Key::PageDown, false);
    frame_at(&mut c, &mut t, 16);
    assert_eq!(session(&c).sidebar_focus(), Some(HudHit::Cameo(0)));
    let t0 = slots(&c)[0].expect("an icon");
    let f = session(&c)
        .focus_tip_shown()
        .cloned()
        .expect("the focus tip");
    assert_eq!(f.text, icon_text(&c, t0));
    // The pointer moves but no pointer tip shows yet: the focus tip stays.
    let m = session(&c).map_rect();
    c.pointer_moved(Point::new(m.x + 40, m.y + m.h / 2));
    frame_at(&mut c, &mut t, 16);
    assert!(
        session(&c).focus_tip_shown().is_some(),
        "no pointer tip yet"
    );
    // A pointer tip shows instead.
    let l = session(&c).layout();
    c.pointer_moved(mid(&c, l.cameo(0)));
    frame_at(&mut c, &mut t, 1000);
    assert!(shown(&c).is_some(), "the pointer tip after the wait");
    assert!(
        session(&c).focus_tip_shown().is_none(),
        "no focus tip while a pointer tip shows"
    );
}
