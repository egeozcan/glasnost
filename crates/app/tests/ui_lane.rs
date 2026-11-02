//! Local lane for the game client (never in CI): with `GLASNOST_GAME_DIR`
//! set, drives the windowless client (`glasnost_app::game::client`) from
//! the main menu through the skirmish setup into a game against the AI,
//! builds a structure to show placement mode, surrenders, and writes each
//! screen drawn on the CPU to `local-lane/ui/` (git-ignored): `menu.png`,
//! `setup.png`, `game.png`, `placement.png` and `end.png`. Prints counts
//! and timings only.
//!
//! Run with `cargo test -p glasnost-app --test ui_lane -- --ignored
//! --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map (default
//! 0), `GLASNOST_UI_TICKS=n` the ticks before `game.png` (default 300).

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;
use std::time::Instant;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::{Button, Tool};
use glasnost_app::mapview::png;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::Modifiers;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

fn files(dir: &std::path::Path) -> Vec<PickedFile> {
    let ext = MountOptions::default().archive_extensions;
    let mut out = Vec::new();
    for e in std::fs::read_dir(dir).unwrap().flatten() {
        let Ok(meta) = e.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let name = e.file_name().to_string_lossy().into_owned();
        let x = name
            .rsplit_once('.')
            .map(|(_, x)| x.to_ascii_lowercase())
            .unwrap_or_default();
        let source: Rc<dyn ByteSource> = if ext.contains(&x) {
            match FileSource::open(&e.path()) {
                Ok(f) => Rc::new(f),
                Err(_) => continue,
            }
        } else {
            Rc::new(Unopened(meta.len()))
        };
        out.push(PickedFile { path: name, source });
    }
    out
}

fn env_num(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn run_jobs(c: &mut Client) {
    while let Some(job) = c.job.take() {
        let t0 = Instant::now();
        let done = pollster::block_on(run_job(job));
        c.done(done);
        println!("job done in {} ms", t0.elapsed().as_millis());
    }
}

fn click(c: &mut Client, r: Rect) {
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

struct Shots {
    dir: std::path::PathBuf,
}

impl Shots {
    fn save(&self, c: &mut Client, name: &str) {
        let t0 = Instant::now();
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
        let path = self.dir.join(format!("{name}.png"));
        std::fs::write(&path, png::png(w as u32, h as u32, &rgba)).unwrap();
        println!("wrote {name}.png in {} ms", t0.elapsed().as_millis());
    }
}

/// Advances the game by `n` ticks (the clock jumps a second per call, so
/// each call is due).
fn ticks(c: &mut Client, clock: &mut i64, n: usize) {
    for _ in 0..n {
        *clock += 1000;
        c.update(*clock);
        if c.shell.screen != Screen::Game {
            break;
        }
    }
}

/// The same frame drawn through the window's GPU path (`GpuDraw`)
/// offscreen, when there is an adapter: written to `game-gpu.png` and
/// compared with the CPU frame.
fn gpu_shot(shots: &Shots, c: &mut Client, cpu: &[u8]) {
    use glasnost_render::gpu;
    use glasnost_render::offscreen::{OFFSCREEN_FORMAT, render_rgba};
    use glasnost_render::wgpu;
    // The process's one shared device (M7b step 2c).
    let Some((device, queue)) = gpu::shared_device().ok().map(|g| (g.device, g.queue)) else {
        println!("no GPU adapter: game-gpu.png not written");
        return;
    };
    let (w, h) = (c.size.0 as u32, c.size.1 as u32);
    let mut draw = glasnost_app::game::GpuDraw::new(&device, OFFSCREEN_FORMAT);
    c.cpu_map = false;
    // The batch's palettes and atlas go to this device too.
    c.painter.dirty = true;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("lane map"),
    });
    draw.prepare(&device, &queue, c, (w, h), &mut encoder);
    queue.submit([encoder.finish()]);
    let rgba = render_rgba(&device, &queue, w, h, |pass| draw.draw(pass));
    assert!(!c.cpu_map, "the GPU battlefield did not fail");
    c.cpu_map = true;
    std::fs::write(shots.dir.join("game-gpu.png"), png::png(w, h, &rgba)).unwrap();
    let m = c.session.as_ref().unwrap().map_rect();
    let mut colours = std::collections::BTreeSet::new();
    let (mut diff, mut n) = (0u64, 0u64);
    for y in m.y..m.y + m.h {
        for x in m.x..m.x + m.w {
            let at = ((y * c.size.0 + x) * 4) as usize;
            colours.insert([rgba[at], rgba[at + 1], rgba[at + 2]]);
            for k in 0..3 {
                diff += u64::from(rgba[at + k].abs_diff(cpu[at + k]));
                n += 1;
            }
        }
    }
    println!(
        "wrote game-gpu.png: {} colours on the battlefield, mean difference from the CPU frame {:.2}",
        colours.len(),
        diff as f64 / n.max(1) as f64
    );
    assert!(colours.len() > 16, "the GPU battlefield is blank");
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn client_screens_from_menu_to_end() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/ui");
    std::fs::create_dir_all(&out).unwrap();
    let shots = Shots { dir: out };
    let mut c = Client::new();
    c.resize((1280, 800));

    c.mount(files(std::path::Path::new(&dir)));
    run_jobs(&mut c);
    let install = c.install.as_ref().expect("the install reads");
    println!("install: {}", install.summary);
    let map = env_num("GLASNOST_UI_MAP", 0).min(install.maps.len() - 1);
    shots.save(&mut c, "menu");

    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    assert_eq!(c.shell.screen, Screen::Setup);
    for _ in 0..map {
        let _ = c.frame();
        let r = c.setup_targets.rect_of(&SetupHit::Map(1)).unwrap();
        click(&mut c, r);
    }
    shots.save(&mut c, "setup");
    let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    c.next_seed = 12345;
    click(&mut c, r);
    run_jobs(&mut c);
    assert_eq!(
        c.shell.screen,
        Screen::Game,
        "the game starts: {}",
        c.status
    );

    // Deploy the base unit where it stands, then play on.
    let mut clock = 0i64;
    ticks(&mut c, &mut clock, 5);
    {
        let s = c.session.as_mut().unwrap();
        let local = s.play.me();
        let mcv = s
            .play
            .world()
            .objects
            .values()
            .find(|o| o.owner == Some(local) && o.kind == Kind::Vehicle && o.on_map)
            .map(|o| o.id);
        if let Some(id) = mcv {
            s.play.select_box(&[id], false);
        }
    }
    c.key(Key::Char('d'), true);
    c.key(Key::Char('d'), false);
    let t0 = Instant::now();
    ticks(&mut c, &mut clock, env_num("GLASNOST_UI_TICKS", 300));
    println!(
        "ticked to {} in {} ms",
        c.session.as_ref().unwrap().play.world().tick,
        t0.elapsed().as_millis()
    );
    shots.save(&mut c, "game");
    let cpu_game = c.frame_rgba();
    gpu_shot(&shots, &mut c, &cpu_game);

    // Build the first building icon and enter placement mode.
    let s = c.session.as_mut().unwrap();
    let local = s.play.me();
    let first = s.play.world().houses[usize::from(local.0)].econ.tech.icons[0]
        .first()
        .map(|i| i.t);
    // The first icon's indices as grey levels, to tell the icon palette
    // (Q-0275) from the decoding.
    if let Some(img) = first.and_then(|t| s.icon(t)) {
        let grey: Vec<u8> = img.pixels().iter().flat_map(|&i| [i, i, i, 255]).collect();
        let path = shots.dir.join("icon-indices.png");
        std::fs::write(&path, png::png(img.width(), img.height(), &grey)).unwrap();
        println!(
            "icon {}x{}: wrote icon-indices.png",
            img.width(),
            img.height()
        );
    }
    s.play.tab_click(0);
    s.play.cameo_click(0, Button::Left, false);
    let mut placing = false;
    for _ in 0..6000 {
        let s = c.session.as_mut().unwrap();
        let ready = first.is_some_and(|t| {
            glasnost_sim::econ::sidebar::icon_state(s.play.world(), local, t)
                .is_some_and(|st| st.ready)
        });
        if ready {
            s.play.cameo_click(0, Button::Left, false);
            placing = matches!(s.play.tool, Tool::Place(_));
            break;
        }
        clock += 1000;
        c.update(clock);
    }
    if placing {
        let s = c.session.as_mut().unwrap();
        let m = s.map_rect();
        let mut at = Point::new(m.x + m.w / 2, m.y + m.h / 2);
        'find: for dy in (-200..=200).step_by(15) {
            for dx in (-300..=300).step_by(30) {
                let p = Point::new(m.x + m.w / 2 + dx, m.y + m.h / 2 + dy);
                let (cell, _) = s.under(p);
                if s.play.preview(cell).is_some_and(|(_, ok)| ok) {
                    at = p;
                    break 'find;
                }
            }
        }
        c.pointer_moved(at);
        shots.save(&mut c, "placement");
    } else {
        println!("no building became ready: placement.png not written");
    }

    // Surrender and wait for the end screen.
    let s = c.session.as_mut().unwrap();
    s.play.tool = Tool::Select;
    s.play.surrender();
    ticks(&mut c, &mut clock, 4000);
    assert!(c.end().is_some(), "the end screen shows");
    shots.save(&mut c, "end");
}
