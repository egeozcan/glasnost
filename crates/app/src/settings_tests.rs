//! The settings format, the key bindings and the stores. The settings are
//! our own client choices (nocite); only the defaults that a spec fixes cite
//! it.

use super::*;
use std::cell::RefCell;
use std::collections::BTreeMap;

fn custom() -> Settings {
    let mut b = Bindings::default();
    b.rebind(Action::Stop, 'q').unwrap();
    b.rebind(Action::Cheer, '7').unwrap();
    Settings {
        sound: 30,
        voices: 0,
        music: 70,
        scroll_index: 0,
        ui_scale: 150,
        captions: false,
        colour_blind: true,
        bindings: b,
        relay: "ws://relay.example:9000/".into(),
        rejoin: Some(Rejoin {
            relay: "ws://relay.example:9000/a b".into(),
            code: "ABCD2345".into(),
            slot: 3,
            ticket: [0xa5; 16],
        }),
        direct: false,
        ice: "stun:stun.example:3478 turn:turn.example:3478".into(),
        tool_tips: false,
        touchpad_scroll: false,
    }
}

/// spec: S-0101 Import mapping (a missing saved scroll setting gives 3), S-0213 claim 22
/// (fresh-profile keys), S-0701 F3 and claim 35 (volume options at full)
#[test]
fn defaults_are_the_specified_ones() {
    let s = Settings::default();
    assert_eq!(s.scroll_index, 3);
    assert_eq!((s.sound, s.voices), (100, 100));
    for (a, c) in ACTIONS.iter().zip(['s', 'x', 'g', 'd', 'c']) {
        assert_eq!(s.bindings.key(*a), c);
    }
    // Our own diagnostics keys (M8 A4), none of claim 22's.
    assert_eq!(s.bindings.key(Action::PathReport), 'p');
    assert_eq!(s.bindings.key(Action::Mark), 'm');
    // Our key for S-0214's type-select role, whose key is the player's.
    assert_eq!(s.bindings.key(Action::TypeSelect), 't');
    // Our key for S-0213 claim 16's route mode, whose key is the player's.
    assert_eq!(s.bindings.key(Action::RouteMode), 'z');
    assert_eq!(Settings::mixer_level(100), 16384);
    assert_eq!(Settings::mixer_level(0), 0);
}

/// Direct links (M9 N7): on, with no STUN or TURN server, unless the
/// settings say otherwise; the default text has neither line.
#[test]
fn direct_links_are_on_with_no_servers_by_default() {
    let s = Settings::default();
    assert!(s.direct);
    assert!(s.ice.is_empty());
    let text = s.to_text();
    assert!(
        !text.contains("direct=") && !text.contains("ice="),
        "{text}"
    );
    let off = Settings::from_text("version=1\ndirect=0\nice= stun:a:1  \n");
    assert!(!off.direct);
    assert_eq!(off.ice, "stun:a:1");
    assert!(Settings::from_text("direct=yes").direct);
}

/// spec: S-0111 claim 20 and Import mapping (the tooltip option is on unless the player's
/// settings turn it off: a settings text without the key gives it on)
#[test]
fn the_tooltip_option_is_on_unless_turned_off() {
    assert!(Settings::default().tool_tips);
    assert!(Settings::from_text("version=1\n").tool_tips);
    assert!(!Settings::from_text("version=1\ntool_tips=0\n").tool_tips);
    assert!(Settings::from_text("version=1\ntool_tips=1\n").tool_tips);
}

/// Touchpad scrolling (design: touch and touchpad scrolling (lead's
/// request 2026-11-10)): on by default and in a file without the key
/// (every file written before it); the default text has no line. The
/// setting's first key, `touch`, still turns it off.
#[test]
fn touchpad_scrolling_is_on_unless_turned_off() {
    let s = Settings::default();
    assert!(s.touchpad_scroll);
    assert!(!s.to_text().contains("touch"), "{}", s.to_text());
    assert!(Settings::from_text("version=1\n").touchpad_scroll);
    let off = Settings::from_text("version=1\ntouchpad=0\n");
    assert!(!off.touchpad_scroll);
    assert!(off.to_text().contains("touchpad=0\n"));
    let old = Settings::from_text("version=1\ntouch=0\n");
    assert!(!old.touchpad_scroll, "the first key's meaning is kept");
    assert!(old.to_text().contains("touchpad=0\n"));
    assert!(Settings::from_text("touchpad=maybe").touchpad_scroll);
}

#[test]
fn text_round_trips() {
    for s in [Settings::default(), custom()] {
        assert_eq!(Settings::from_text(&s.to_text()), s);
    }
    assert!(custom().to_text().starts_with("version=1\n"));
}

/// An older file lacking fields, a newer one with more, and garbage all load.
#[test]
fn missing_unknown_and_wrong_fields_take_defaults() {
    let s = Settings::from_text("version=1\nsound=40\n");
    assert_eq!(s.sound, 40);
    assert_eq!(s.music, 100);
    assert_eq!(s.bindings, Bindings::default());
    let s = Settings::from_text("version=9\nsound=41\nfuture=1\nrelay=ws://x/\n");
    assert_eq!((s.sound, s.relay.as_str()), (41, "ws://x/"));
    assert_eq!(
        Settings::from_text("\u{0}garbage\n===\nsound=abc"),
        Settings::default()
    );
    assert_eq!(Settings::from_text(""), Settings::default());
    // A game to rejoin (M9 N5) that is not whole is no game to rejoin.
    let hex = "00112233445566778899aabbccddeeff";
    let s = Settings::from_text(&format!("rejoin=ABCD2345 1 {hex} ws://r/\n"));
    let r = s.rejoin.expect("a game to rejoin");
    assert_eq!(
        (r.code.as_str(), r.slot, r.relay.as_str()),
        ("ABCD2345", 1, "ws://r/")
    );
    assert_eq!(r.ticket[15], 0xff);
    for bad in [
        format!("rejoin=ABCD2345 1 {hex}"),
        format!("rejoin=ABCD2345 x {hex} ws://r/"),
        format!("rejoin=ABCD2345 1 {} ws://r/", &hex[..30]),
        "rejoin=ABCD2345 1 zz112233445566778899aabbccddeeff ws://r/".into(),
        format!("rejoin=AB-D 1 {hex} ws://r/"),
    ] {
        assert_eq!(Settings::from_text(&bad).rejoin, None, "{bad}");
    }
    // Out-of-range values clamp; an unoffered scale is ignored.
    let s = Settings::from_text("sound=900\nmusic=-5\nscroll=99\nscale=77\n");
    assert_eq!(
        (s.sound, s.music, s.scroll_index, s.ui_scale),
        (100, 0, 6, 100)
    );
}

#[test]
fn bindings_refuse_conflicts_and_bad_keys() {
    let mut b = Bindings::default();
    assert_eq!(
        b.rebind(Action::Stop, 'x'),
        Err(BindError::Taken(Action::Scatter))
    );
    assert_eq!(b.key(Action::Stop), 's', "a refused key changes nothing");
    assert_eq!(b.rebind(Action::Stop, ' '), Err(BindError::NotBindable));
    assert_eq!(
        b.rebind(Action::Stop, 'S'),
        Ok(()),
        "own key again, any case"
    );
    assert_eq!(b.rebind(Action::Stop, 'q'), Ok(()));
    assert_eq!(b.action('q'), Some(Action::Stop));
    assert_eq!(b.action('s'), None);
}

#[test]
fn a_file_with_clashing_keys_falls_back_to_the_default_keys() {
    let s = Settings::from_text("key.stop=x\nkey.scatter=x\nsound=20\n");
    assert_eq!(s.bindings, Bindings::default());
    assert_eq!(s.sound, 20, "the other fields still load");
    let s = Settings::from_text("key.stop=!\n");
    assert_eq!(s.bindings, Bindings::default());
}

/// A file written before the pathing-report keys (M8 A4) that gives P to
/// a command: the command keeps P and the report key moves to a free key;
/// one that leaves P and M alone gets them.
#[test]
fn a_file_older_than_the_report_keys_keeps_its_own() {
    let s =
        Settings::from_text("key.stop=p\nkey.scatter=x\nkey.guard=g\nkey.deploy=d\nkey.cheer=c\n");
    assert_eq!(s.bindings.key(Action::Stop), 'p');
    assert_eq!(s.bindings.key(Action::PathReport), 'a');
    assert_eq!(s.bindings.key(Action::Mark), 'm');
    let s = Settings::from_text("key.stop=q\n");
    assert_eq!(s.bindings.key(Action::PathReport), 'p');
    assert_eq!(s.bindings.key(Action::Mark), 'm');
    assert_eq!(s.bindings.key(Action::TypeSelect), 't');
    // A file older than the type-select key that gives T to a command.
    let s = Settings::from_text("key.stop=t\nkey.report=a\n");
    assert_eq!(s.bindings.key(Action::Stop), 't');
    assert_eq!(s.bindings.key(Action::TypeSelect), 'b');
    assert_eq!(s.bindings.key(Action::PathReport), 'a');
    // A written type-select key comes back.
    let s = Settings::from_text("key.typeselect=y\n");
    assert_eq!(s.bindings.key(Action::TypeSelect), 'y');
    assert_eq!(Settings::from_text(&s.to_text()).bindings, s.bindings);
}

/// A file written before the make-group key (M8 A4 round 5 seams, our
/// design) keeps N for it unless the file gives N to another action; it
/// then takes the first free key. A written make-group key comes back.
#[test]
fn a_file_older_than_the_make_group_key_keeps_its_own() {
    let s = Settings::from_text("key.stop=q\n");
    assert_eq!(s.bindings.key(Action::MakeGroup), 'n');
    let s = Settings::from_text("key.stop=n\n");
    assert_eq!(s.bindings.key(Action::Stop), 'n');
    assert_eq!(s.bindings.key(Action::MakeGroup), 'a');
    let s = Settings::from_text("key.group=j\n");
    assert_eq!(s.bindings.key(Action::MakeGroup), 'j');
    assert!(s.to_text().contains("key.group=j"));
    assert_eq!(Settings::from_text(&s.to_text()).bindings, s.bindings);
}

#[test]
fn steps_stay_in_range() {
    let mut s = Settings::default();
    s.step_scroll(10);
    assert_eq!(s.scroll_index, 0);
    assert_eq!(s.scroll_label(), "7 of 7");
    s.step_scroll(-10);
    assert_eq!(s.scroll_label(), "1 of 7");
    s.step_scale(-1);
    assert_eq!(s.ui_scale, 100);
    s.step_scale(1);
    assert_eq!(s.ui_scale, 125);
    for _ in 0..9 {
        s.step_scale(1);
    }
    assert_eq!(s.ui_scale, 300);
    assert_eq!(step_volume(95, 1), 100);
    assert_eq!(step_volume(5, -1), 0);
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("glasnost-settings-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// The native file: written, read back, replaced, with its folder created.
#[test]
fn the_native_file_round_trips() {
    let dir = scratch("file");
    let store = FileStore(dir.join("glasnost").join("settings.txt"));
    assert_eq!(store.read(), None, "no file: defaults");
    store.write(&custom().to_text()).unwrap();
    assert_eq!(Settings::from_text(&store.read().unwrap()), custom());
    store.write(&Settings::default().to_text()).unwrap();
    assert_eq!(
        Settings::from_text(&store.read().unwrap()),
        Settings::default()
    );
    assert!(!dir.join("glasnost").join("settings.tmp").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_unwritable_place_reports_instead_of_panicking() {
    let dir = scratch("blocked");
    std::fs::create_dir_all(&dir).unwrap();
    // A file where the folder should be.
    std::fs::write(dir.join("glasnost"), "x").unwrap();
    let store = FileStore(dir.join("glasnost").join("settings.txt"));
    assert!(store.write("version=1\n").is_err());
    assert_eq!(store.read(), None);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_config_folder_follows_the_platform() {
    let env = |pairs: &'static [(&str, &str)]| {
        move |k: &str| {
            pairs
                .iter()
                .find(|(n, _)| *n == k)
                .map(|(_, v)| (*v).to_owned())
        }
    };
    let tail = |p: Option<PathBuf>| p.map(|p| p.to_string_lossy().replace('\\', "/"));
    assert_eq!(
        tail(config_path(
            Host::Unix,
            env(&[("XDG_CONFIG_HOME", "/c"), ("HOME", "/h")])
        )),
        Some("/c/glasnost/settings.txt".into())
    );
    assert_eq!(
        tail(config_path(Host::Unix, env(&[("HOME", "/h")]))),
        Some("/h/.config/glasnost/settings.txt".into())
    );
    // A relative XDG folder is not used (the spec of the variable says so).
    assert_eq!(
        tail(config_path(
            Host::Unix,
            env(&[("XDG_CONFIG_HOME", "rel"), ("HOME", "/h")])
        )),
        Some("/h/.config/glasnost/settings.txt".into())
    );
    assert_eq!(
        tail(config_path(Host::Mac, env(&[("HOME", "/machome")]))),
        Some("/machome/Library/Application Support/glasnost/settings.txt".into())
    );
    assert_eq!(config_path(Host::Unix, env(&[])), None);
    #[cfg(windows)]
    assert_eq!(
        tail(config_path(
            Host::Windows,
            env(&[("APPDATA", "C:\\wuser\\AppData\\Roaming")])
        )),
        Some("C:/wuser/AppData/Roaming/glasnost/settings.txt".into())
    );
}

/// A browser-like storage that may refuse.
struct Mem {
    items: RefCell<BTreeMap<String, String>>,
    blocked: bool,
}

impl KeyValue for Mem {
    fn get(&self, key: &str) -> Result<Option<String>, String> {
        if self.blocked {
            return Err("SecurityError".into());
        }
        Ok(self.items.borrow().get(key).cloned())
    }

    fn set(&self, key: &str, value: &str) -> Result<(), String> {
        if self.blocked {
            return Err("QuotaExceededError".into());
        }
        self.items.borrow_mut().insert(key.into(), value.into());
        Ok(())
    }
}

/// The web layer's logic: round trip under its key, and a blocked storage
/// reads as empty (defaults) and reports on writing.
#[test]
fn the_web_storage_layer_round_trips_and_survives_a_block() {
    let open = KvStore(Mem {
        items: RefCell::default(),
        blocked: false,
    });
    assert_eq!(open.read(), None);
    open.write(&custom().to_text()).unwrap();
    assert_eq!(Settings::from_text(&open.read().unwrap()), custom());
    assert!(open.0.items.borrow().contains_key(WEB_KEY));

    let blocked = KvStore(Mem {
        items: RefCell::default(),
        blocked: true,
    });
    assert_eq!(blocked.read(), None);
    assert!(blocked.write("x").is_err());
    assert_eq!(
        Settings::from_text(&blocked.read().unwrap_or_default()),
        Settings::default()
    );
}
