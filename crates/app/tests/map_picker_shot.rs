//! Pictures of the skirmish setup screen and its map picker (design: map
//! picker (lead's request 2026-11-13)) on the CPU path, with synthetic data
//! only: invented maps whose previews are generated patterns packed by our
//! own writers; no install. Writes PNG files into the folder named by
//! `GLASNOST_SHOT_DIR` when it is set; otherwise it only checks that the
//! screens draw.

#![cfg(not(target_arch = "wasm32"))]

use glasnost_app::game::client::{Client, Done, Key};
use glasnost_app::game::install::{Install, MapInfo, map_info_of};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_convert::find::Listing;
use glasnost_convert::map::synth::MapText;
use glasnost_fmt::mix::MixEntry;
use glasnost_ini::Document;
use glasnost_sim::targeting::Modifiers;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::mappick::FilterKind;
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::{MapEntry, SetupHit, SetupRules};
use glasnost_vfs::{ArchiveId, FileRef, Vfs};

/// Invented map names, start counts, sizes, theaters and whether each has a
/// generated preview.
/// Name, starts, size, theater identifier and whether it has a preview.
type Made = (&'static str, u32, (i32, i32), &'static str, bool);

const MAPS: [Made; 24] = [
    ("Alpha Test", 2, (60, 60), "TEMPERATE", true),
    ("Bravo Ridge", 4, (100, 80), "SNOW", true),
    ("Charlie Test", 2, (50, 70), "DESERT", false),
    ("Delta Ridge", 6, (140, 120), "URBAN", true),
    ("Echo Basin", 8, (180, 160), "NEWURBAN", true),
    ("Foxtrot Test", 3, (70, 50), "LUNAR", true),
    ("Golf Flats", 2, (64, 64), "SNOW", false),
    ("Hotel Ridge", 4, (90, 90), "TEMPERATE", true),
    ("India Basin", 6, (120, 100), "DESERT", true),
    ("Juliet Test", 2, (40, 80), "URBAN", true),
    ("Kilo Flats", 4, (110, 70), "SNOW", true),
    ("Lima Ridge", 8, (200, 140), "TEMPERATE", false),
    ("Mike Basin", 2, (56, 56), "LUNAR", true),
    ("November Test", 4, (96, 96), "NEWURBAN", true),
    ("Oscar Flats", 6, (130, 130), "DESERT", true),
    ("Papa Ridge", 2, (60, 90), "SNOW", true),
    ("Quebec Basin", 4, (100, 100), "TEMPERATE", true),
    ("Romeo Test", 8, (160, 160), "URBAN", true),
    ("Sierra Flats", 3, (80, 60), "TEMPERATE", false),
    ("Tango Ridge", 2, (50, 50), "LUNAR", true),
    ("Uniform Basin", 4, (120, 80), "SNOW", true),
    ("Victor Test", 6, (140, 140), "DESERT", true),
    ("Whiskey Flats", 2, (70, 70), "NEWURBAN", true),
    ("Xray Ridge", 4, (90, 110), "TEMPERATE", true),
];

/// A generated preview of `w` × `h`: bands and a diagonal in colours of
/// the map's number (our own pattern).
fn pattern(n: usize, w: usize, h: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(w * h * 3);
    for y in 0..h {
        for x in 0..w {
            let band = ((x / 12 + y / 9 + n) % 4) as u8;
            let edge = (x + y) % 37 < 2;
            let (r, g, b) = if edge {
                (230, 220, 160)
            } else {
                (
                    30 + band * 25 + (n as u8 * 7) % 40,
                    70 + band * 30,
                    40 + ((x * 90 / w.max(1)) as u8),
                )
            };
            v.extend_from_slice(&[r, g, b]);
        }
    }
    v
}

/// The picker's facts of invented map `n`, read from its text as the
/// install reads a map's.
fn info(n: usize) -> MapInfo {
    let (_, starts, (w, h), theater, preview) = MAPS[n];
    let (pw, ph) = (w as usize * 2, h as usize);
    let mut ini = format!("[Map]\nSize=0,0,{w},{h}\nTheater={theater}\n");
    ini += &format!("[Preview]\nSize=0,0,{pw},{ph}\n");
    // Markers in the header block, spread over the preview (S-0009 claim 25).
    ini += &format!(
        "[Header]\nNumberStartingPoints={starts}\nStartX=0\nStartY=0\nWidth={pw}\nHeight={ph}\n"
    );
    for i in 0..starts as usize {
        let x = pw / 8 + (i % 4) * pw / 4;
        let y = ph / 5 + (i / 4) * ph / 2 + (i % 2) * ph / 4;
        ini += &format!("Waypoint{}={x},{y}\n", i + 1);
    }
    let text = MapText {
        ini,
        preview: preview.then(|| pattern(n, pw, ph)),
        ..Default::default()
    }
    .text();
    map_info_of(&Document::parse(
        text.as_bytes(),
        Default::default(),
        &mut Vec::new(),
    ))
}

fn install() -> Install {
    let file = |id: u32| FileRef {
        archive: ArchiveId(0),
        entry: MixEntry {
            id,
            offset: 0,
            size: 0,
        },
    };
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
            colours: (0..8u8)
                .map(|i| [40 + 25 * i, 200 - 20 * i, 120, 255])
                .collect(),
        },
        maps: MAPS
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
        map_hashes: (0..MAPS.len())
            .map(|i| format!("map-synthetic-{i}"))
            .collect(),
        map_info: (0..MAPS.len()).map(info).collect(),
        rules_hash: glasnost_ai::fixture::rules().hash.clone(),
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

fn client(size: (i32, i32)) -> Client {
    let mut c = Client::new();
    c.resize(size);
    c.done(Done::Prepared(Vfs::default(), Ok(Box::new(install()))));
    let _ = c.frame();
    c
}

/// An update (where the preview is decoded) and a frame.
fn settle(c: &mut Client) {
    c.update(0);
    let _ = c.frame_rgba();
}

fn shoot(c: &mut Client, name: &str) {
    settle(c);
    let rgba = c.frame_rgba();
    assert_eq!(rgba.len(), (c.size.0 * c.size.1 * 4) as usize);
    if let Ok(dir) = std::env::var("GLASNOST_SHOT_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let path = std::path::Path::new(&dir).join(name);
        std::fs::write(path, png::png(c.size.0 as u32, c.size.1 as u32, &rgba)).unwrap();
    }
}

fn click(c: &mut Client, r: Rect) {
    let r = c.ui_scale().rect(r);
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
    let _ = c.frame();
}

fn press(c: &mut Client, hit: SetupHit) {
    let r = c
        .setup_targets
        .rect_of(&hit)
        .unwrap_or_else(|| panic!("{hit:?} is on the screen"));
    click(c, r);
}

#[test]
fn the_setup_screen_draws_its_map_choice() {
    for (size, name) in [((1280, 960), "1280"), ((640, 480), "640")] {
        let mut c = client(size);
        let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
        click(&mut c, r);
        press(&mut c, SetupHit::Map(1));
        shoot(&mut c, &format!("setup-{name}.png"));
        press(&mut c, SetupHit::OpenMaps);
        shoot(&mut c, &format!("picker-{name}.png"));
        c.typed("ridge");
        shoot(&mut c, &format!("picker-search-{name}.png"));
        c.key(Key::Down, true);
        c.key(Key::Down, false);
        shoot(&mut c, &format!("picker-search-down-{name}.png"));
        for _ in 0..5 {
            c.key(Key::Backspace, true);
        }
        press(&mut c, SetupHit::Filter(FilterKind::Players, 1));
        press(&mut c, SetupHit::Filter(FilterKind::Players, 1));
        press(&mut c, SetupHit::Filter(FilterKind::Sort, 1));
        shoot(&mut c, &format!("picker-filter-{name}.png"));
        // A click on a filter ended the typing; a click on the search field
        // starts it again.
        c.typed("x");
        assert!(c.shell.picker.query.search.is_empty());
        press(&mut c, SetupHit::SearchField);
        c.typed("nothing like it");
        shoot(&mut c, &format!("picker-none-{name}.png"));
        c.key(Key::Escape, true);
        shoot(&mut c, &format!("setup-none-{name}.png"));
    }
}

/// The picker drawn through the window's GPU path (`GpuDraw`) offscreen,
/// where the preview goes up as an image by its id like the battlefield's:
/// `picker-gpu-640.png`, with the count of pixels that differ from the CPU
/// frame. Local lane: it needs a GPU adapter.
#[test]
#[ignore = "local lane: needs a GPU adapter"]
fn the_picker_draws_its_preview_on_the_gpu() {
    use glasnost_render::gpu;
    use glasnost_render::offscreen::{OFFSCREEN_FORMAT, render_rgba};
    use glasnost_render::wgpu;
    let Some((device, queue)) = gpu::shared_device().ok().map(|g| (g.device, g.queue)) else {
        println!("no GPU adapter: no GPU shot");
        return;
    };
    let mut c = client((640, 480));
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    press(&mut c, SetupHit::OpenMaps);
    settle(&mut c);
    let cpu = c.frame_rgba();
    let (w, h) = (c.size.0 as u32, c.size.1 as u32);
    let mut draw = glasnost_app::game::GpuDraw::new(&device, OFFSCREEN_FORMAT);
    draw.set_screen(1, (w, h));
    // A new device: the kept preview goes up again.
    c.new_device();
    c.painter.dirty = true;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("picker frame"),
    });
    draw.prepare(&device, &queue, &mut c, (w, h), &mut encoder);
    queue.submit([encoder.finish()]);
    let rgba = render_rgba(&device, &queue, w, h, |pass| draw.draw(pass));
    let differ = (0..(w * h) as usize)
        .filter(|i| rgba[i * 4..i * 4 + 3] != cpu[i * 4..i * 4 + 3])
        .count();
    if let Ok(dir) = std::env::var("GLASNOST_SHOT_DIR") {
        std::fs::create_dir_all(&dir).unwrap();
        let path = std::path::Path::new(&dir).join("picker-gpu-640.png");
        std::fs::write(path, png::png(w, h, &rgba)).unwrap();
    }
    println!(
        "picker-gpu-640.png: {differ} of {} pixels differ from the CPU frame",
        w * h
    );
    assert_eq!(differ, 0, "the GPU picker matches the CPU frame");
}
