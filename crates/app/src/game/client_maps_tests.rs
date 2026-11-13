//! The map picker in the client (design: map picker (lead's request
//! 2026-11-13)) on invented maps: the preview decoded off the frame and
//! shown (S-0009 claims 24 and 25), the keys, the wheel, and the search and
//! filters kept in the settings.

use std::cell::RefCell;
use std::rc::Rc;

use super::maps::{PREVIEW_IMAGE, preview_rgba};
use super::*;
use glasnost_convert::find::Listing;
use glasnost_convert::map::synth::MapText;
use glasnost_fmt::mix::MixEntry;
use glasnost_ini::Document;
use glasnost_ui::draw::Prim;
use glasnost_ui::mappick::FilterKind;
use glasnost_ui::setup::{MapEntry, SetupRules};
use glasnost_vfs::ArchiveId;

/// Invented maps: name, starts, the map text's own sections, and a preview
/// stream to pack.
type Made = (&'static str, u32, &'static str, Option<Vec<u8>>);

fn maps() -> Vec<Made> {
    vec![
        // A 2 × 1 preview: red, then blue; one header marker.
        (
            "Amber Ridge",
            2,
            "[Map]\nSize=0,0,60,40\nTheater=SNOW\n[Preview]\nSize=0,0,2,1\n\
             [Header]\nNumberStartingPoints=1\nStartX=0\nStartY=0\nWidth=2\nHeight=1\nWaypoint1=1,0\n",
            Some(vec![255, 0, 0, 0, 0, 255]),
        ),
        // No preview block.
        ("Basalt Flats", 4, "[Map]\nSize=0,0,80,80\n", None),
        // A preview block that does not expand: shown black (S-0009 claim 24).
        (
            "Cobalt Ridge",
            2,
            "[Map]\nSize=0,0,50,50\n[Preview]\nSize=0,0,3,2\n[PreviewPack]\n1=QUJDREVGR0hJSktM\n",
            None,
        ),
        // An invalid preview size: no preview.
        (
            "Dune Basin",
            6,
            "[Map]\nSize=0,0,90,70\n[Preview]\nSize=0,0,-3,2\n",
            Some(vec![9; 18]),
        ),
    ]
}

fn install() -> Install {
    let file = |id| FileRef {
        archive: ArchiveId(0),
        entry: MixEntry {
            id,
            offset: 0,
            size: 0,
        },
    };
    let made = maps();
    Install {
        setup: SetupRules {
            money: 5000,
            money_range: (1000, 9000, 500),
            units: 3,
            units_range: (1, 9),
            bases: true,
            flags: [false, true, true, true, true],
            game_speed: 3,
            countries: vec!["Alphaland".into(), "Betaland".into()],
            colours: vec![[200, 0, 0, 255], [0, 0, 200, 255]],
        },
        maps: made
            .iter()
            .enumerate()
            .map(|(i, (name, starts, ..))| {
                (
                    Listing {
                        file: file(i as u32 + 1),
                        archive: "synthetic".into(),
                        name: (*name).into(),
                        theater: "x".into(),
                        multiplayer: true,
                    },
                    MapEntry {
                        label: (*name).into(),
                        starts: *starts,
                    },
                )
            })
            .collect(),
        map_hashes: (0..made.len())
            .map(|i| format!("map-synthetic-{i}"))
            .collect(),
        map_info: made
            .iter()
            .map(|(_, _, ini, preview)| {
                let text = MapText {
                    ini: (*ini).into(),
                    preview: preview.clone(),
                    ..Default::default()
                }
                .text();
                install::map_info_of(&Document::parse(
                    text.as_bytes(),
                    Default::default(),
                    &mut Vec::new(),
                ))
            })
            .collect(),
        rules_hash: "rules-synthetic".into(),
        names: Default::default(),
        sounds: None,
        speech: None,
        bank: Default::default(),
        cursor_palette: Default::default(),
        strings: None,
        music: None,
        summary: "synthetic".into(),
    }
}

fn client() -> Client {
    let mut c = Client::new();
    c.resize((800, 600));
    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    let _ = c.frame();
    c.menu_press(MenuHit::Skirmish);
    c
}

fn uploads(images: &FrameImages) -> Vec<(u32, u32, Vec<u8>)> {
    images
        .changed
        .iter()
        .filter(|(id, ..)| *id == PREVIEW_IMAGE)
        .map(|(_, w, h, rgba)| (*w, *h, rgba.clone()))
        .collect()
}

fn image_rect(d: &DrawList) -> Option<Rect> {
    fn find(prims: &[Prim]) -> Option<Rect> {
        prims.iter().find_map(|p| match p {
            Prim::Image {
                id: PREVIEW_IMAGE,
                rect,
            } => Some(*rect),
            Prim::Layer { list, .. } => find(&list.prims),
            _ => None,
        })
    }
    find(&d.prims)
}

fn texts(d: &DrawList) -> Vec<String> {
    fn walk(prims: &[Prim], out: &mut Vec<String>) {
        for p in prims {
            match p {
                Prim::Text { text, .. } => out.push(text.clone()),
                Prim::Layer { list, .. } => walk(&list.prims, out),
                _ => {}
            }
        }
    }
    let mut out = Vec::new();
    walk(&d.prims, &mut out);
    out
}

/// The chosen map's preview is decoded in the update, not in the frame,
/// uploaded once, shown with its marker, and sent again to a new device.
// spec: S-0009 claim 24 (RGB, row by row) and claim 25 (the header's markers over the preview)
#[test]
fn the_preview_is_decoded_off_the_frame_and_uploaded_once() {
    let mut c = client();
    assert_eq!(c.shell.screen, Screen::Setup);
    let (d, images) = c.frame();
    assert!(uploads(&images).is_empty(), "nothing decoded yet");
    assert!(image_rect(&d).is_none());
    assert!(texts(&d).iter().any(|t| t == "..."), "pending");
    c.update(0);
    let (d, images) = c.frame();
    assert_eq!(
        uploads(&images),
        [(2, 1, vec![255, 0, 0, 255, 0, 0, 255, 255])]
    );
    assert!(image_rect(&d).is_some());
    assert!(texts(&d).iter().any(|t| t == "1"), "the header's marker");
    let (_, images) = c.frame();
    assert!(uploads(&images).is_empty(), "uploaded once");
    c.new_device();
    let (_, images) = c.frame();
    assert_eq!(uploads(&images).len(), 1, "a new device gets it again");
    // On the CPU path the image is drawn.
    c.update(0);
    let px = c.frame_rgba();
    assert_eq!(px.len(), (c.size.0 * c.size.1 * 4) as usize);
}

/// A map without a preview shows the placeholder; a block that does not
/// expand shows black (S-0009 claim 24: the pixels not decoded are black);
/// an invalid preview size gives no preview (S-0012 Edge cases).
#[test]
fn missing_broken_and_invalid_previews() {
    let mut c = client();
    c.setup_press(SetupHit::Map(1));
    c.update(0);
    let (d, images) = c.frame();
    assert!(uploads(&images).is_empty());
    assert!(image_rect(&d).is_none());
    assert!(texts(&d).iter().any(|t| t == "No preview"));
    c.setup_press(SetupHit::Map(1));
    c.update(0);
    let (d, images) = c.frame();
    assert_eq!(uploads(&images), [(3, 2, [0, 0, 0, 255].repeat(6))]);
    assert!(image_rect(&d).is_some());
    c.setup_press(SetupHit::Map(1));
    c.update(0);
    let (d, images) = c.frame();
    assert!(uploads(&images).is_empty());
    assert!(texts(&d).iter().any(|t| t == "No preview"));
    assert!(!c.shell.picker.open);
    // The picker says the map has none.
    c.setup_press(SetupHit::OpenMaps);
    let (d, _) = c.frame();
    assert!(texts(&d).iter().any(|t| t == "No embedded preview"));
}

/// The picker's keys: typing narrows the list (the chosen map follows),
/// Backspace widens it, Down and Up choose in the list, Escape closes it.
#[test]
fn the_picker_takes_typing_and_its_keys() {
    let mut c = client();
    c.setup_press(SetupHit::OpenMaps);
    assert!(c.typing());
    c.typed("RIDGE");
    assert_eq!(c.shell.picker.query.search, "RIDGE");
    assert_eq!(c.shell.setup.as_ref().unwrap().map, 0);
    c.key(Key::Down, true);
    assert_eq!(c.shell.setup.as_ref().unwrap().map, 2, "the next ridge");
    c.key(Key::Up, true);
    assert_eq!(c.shell.setup.as_ref().unwrap().map, 0);
    c.typed(" cob");
    assert_eq!(c.shell.setup.as_ref().unwrap().map, 2, "the first match");
    // A letter typed is no command, a Space is text.
    c.key(Key::Char('x'), true);
    c.key(Key::Space, true);
    for _ in 0..4 {
        c.key(Key::Backspace, true);
    }
    assert_eq!(c.shell.picker.query.search, "RIDGE");
    // Tab moves the focus to the search, then off it: typing stops.
    let _ = c.frame();
    c.key(Key::Tab, true);
    assert!(c.typing());
    c.key(Key::Tab, true);
    assert!(!c.typing());
    c.key(Key::Escape, true);
    assert!(!c.shell.picker.open);
    assert_eq!(
        c.shell.screen,
        Screen::Setup,
        "Escape closed only the picker"
    );
    c.key(Key::Escape, true);
    assert_eq!(c.shell.screen, Screen::Menu);
}

/// With no map matching, the picker and the setup say so and the game does
/// not start.
#[test]
fn no_match_keeps_start_off() {
    let mut c = client();
    c.setup_press(SetupHit::OpenMaps);
    c.typed("zz");
    let (d, _) = c.frame();
    assert!(
        texts(&d)
            .iter()
            .any(|t| t == "No map matches the search and filters")
    );
    c.key(Key::Enter, true);
    assert!(!c.shell.picker.open, "Enter closes the picker");
    let (d, _) = c.frame();
    assert!(
        texts(&d)
            .iter()
            .any(|t| t == "No map matches the search and filters")
    );
    assert!(c.setup_targets.rect_of(&SetupHit::Start_).is_none());
    c.setup_press(SetupHit::Start_);
    assert!(c.job.is_none());
}

/// The wheel and a touch drag scroll the open picker's list.
#[test]
fn the_wheel_scrolls_the_picker() {
    let mut c = client();
    // Too few maps to scroll: the wheel does nothing, and stays out of the
    // game.
    c.setup_press(SetupHit::OpenMaps);
    c.wheel(-1);
    assert_eq!(c.shell.picker.scroll, 0);
    let many: Vec<MapEntry> = (0..60)
        .map(|i| MapEntry {
            label: format!("Map {i:02}"),
            starts: 2,
        })
        .collect();
    let i = c.install.as_mut().unwrap();
    let first = i.maps[0].0.clone();
    i.maps = many.into_iter().map(|m| (first.clone(), m)).collect();
    i.map_info.clear();
    i.map_hashes = vec!["h".into(); 60];
    c.wheel(-1);
    assert_eq!(c.shell.picker.scroll, 3, "content up: later rows");
    c.wheel(1);
    assert_eq!(c.shell.picker.scroll, 0);
    c.picker_rows(-5);
    assert_eq!(c.shell.picker.scroll, 5);
}

/// The search and filters are kept in the settings as they change, and a
/// client started with them chooses the first match when the skirmish
/// screen opens.
#[test]
fn the_search_and_filters_are_kept() {
    #[derive(Clone, Default)]
    struct Mem(Rc<RefCell<Option<String>>>);
    impl Store for Mem {
        fn read(&self) -> Option<String> {
            self.0.borrow().clone()
        }
        fn write(&self, text: &str) -> Result<(), String> {
            *self.0.borrow_mut() = Some(text.into());
            Ok(())
        }
    }
    let store = Mem::default();
    let mut c = Client::new();
    c.attach_store(Box::new(store.clone()));
    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    c.menu_press(MenuHit::Skirmish);
    c.setup_press(SetupHit::OpenMaps);
    c.typed("basin");
    c.setup_press(SetupHit::Filter(FilterKind::Sort, 1));
    let text = store.read().unwrap();
    assert!(text.contains("maps.search=basin\n"), "{text}");
    assert!(text.contains("maps.sort=Name\n"), "{text}");
    let mut again = Client::new();
    again.attach_store(Box::new(store.clone()));
    assert_eq!(again.shell.picker.query, c.shell.picker.query);
    again.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    again.menu_press(MenuHit::Skirmish);
    assert_eq!(again.shell.setup.as_ref().unwrap().map, 3, "the only basin");
    // Reset on the settings screen keeps them.
    again.settings_click(SettingsHit::Reset);
    assert_eq!(again.settings.maps.search, "basin");
}

/// A large preview is uploaded at every n-th pixel, no side over 1024;
/// the cache keeps a bounded number of previews.
#[test]
fn large_previews_shrink_and_the_cache_is_bounded() {
    let (w, h) = (2500u32, 3);
    let rgb: Vec<u8> = (0..w * h).flat_map(|i| [(i % 251) as u8, 0, 0]).collect();
    let d = preview_rgba(w, h, &rgb);
    let (dw, dh, px) = (d.w, d.h, &d.rgba);
    assert_eq!((dw, dh), (834, 1), "every third pixel");
    assert_eq!(px[4], 3, "the second pixel is the fourth");
    assert_eq!(px.len(), (dw * dh * 4) as usize);
    let mut c = client();
    let n = c.install.as_ref().unwrap().maps.len();
    for _ in 0..40 {
        c.setup_press(SetupHit::Map(1));
        c.update(0);
    }
    assert!(c.previews.kept() <= n.min(32));
}
