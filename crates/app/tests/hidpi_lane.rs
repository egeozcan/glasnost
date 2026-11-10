//! Local lane for readable text and the display scale (never in CI;
//! design: readable text and display scale (lead's request 2026-11-10)).
//! With `GLASNOST_GAME_DIR` set, drives the windowless client through the
//! main menu, the settings, the skirmish setup, a game (a status line and a
//! sidebar tip) and the end screen, on a window of 1280 × 800 logical
//! pixels at display factors 1 and 2, and writes each screen as the window
//! would show it to `local-lane/side/hidpi/shots/<phase>/` (git-ignored;
//! the pictures hold game art and are never committed): `<screen>-1x.png`
//! and `<screen>-2x.png`, plus `game-gpu-<f>x.png` through the window's GPU
//! path. `GLASNOST_HIDPI_PHASE` names the phase folder (default `after`).
//! Prints counts and timings only.
//!
//! Run with `cargo test -p glasnost-app --test hidpi_lane -- --ignored
//! --nocapture`.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;
use std::time::Instant;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::game::display::upscale;
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::Modifiers;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

/// The window's size in logical pixels (a common laptop size).
const LOGICAL: (i32, i32) = (1280, 800);

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

fn run_jobs(c: &mut Client) {
    while let Some(job) = c.job.take() {
        let done = pollster::block_on(run_job(job));
        c.done(done);
    }
}

/// A click at the middle of a control laid out in the interface's logical
/// pixels.
fn click(c: &mut Client, r: Rect) {
    let r = c.ui_scale().rect(r);
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

/// Advances the game by `n` ticks (the clock jumps a second per call).
fn ticks(c: &mut Client, clock: &mut i64, n: usize) {
    for _ in 0..n {
        *clock += 1000;
        c.update(*clock);
        if c.shell.screen != Screen::Game {
            break;
        }
    }
}

struct Shots {
    dir: std::path::PathBuf,
    factor: i32,
}

impl Shots {
    /// The screen as the window shows it, in physical pixels.
    fn save(&self, c: &mut Client, name: &str) {
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
        let f = c.display().factor as u32;
        let (w, h, rgba) = (
            w as u32 * f,
            h as u32 * f,
            upscale(&rgba, (w as u32, h as u32), f),
        );
        let file = format!("{name}-{}x.png", self.factor);
        std::fs::write(self.dir.join(&file), png::png(w, h, &rgba)).unwrap();
        println!("wrote {file} ({w} x {h})");
    }
}

/// Opens a client on a window of [`LOGICAL`] at the host scale factor
/// `factor`, as the window layer does (`Client::set_window`); the
/// display-scale setting is Auto.
fn window(factor: i32) -> Client {
    let mut c = Client::new();
    c.set_window((LOGICAL.0 * factor, LOGICAL.1 * factor), f64::from(factor));
    c
}

/// The game frame drawn through the window's GPU path (`GpuDraw`) at the
/// client's display factor offscreen, when there is an adapter, compared
/// with the CPU frame upscaled: `game-gpu-<f>x.png`.
fn gpu_shot(shots: &Shots, c: &mut Client) {
    use glasnost_render::gpu;
    use glasnost_render::offscreen::{OFFSCREEN_FORMAT, render_rgba};
    use glasnost_render::wgpu;
    let Some((device, queue)) = gpu::shared_device().ok().map(|g| (g.device, g.queue)) else {
        println!("no GPU adapter: no GPU shot");
        return;
    };
    // The CPU frame first (the battlefield composed on the CPU).
    let cpu = c.frame_rgba();
    let f = c.display().factor as u32;
    let view = (c.size.0 as u32, c.size.1 as u32);
    let cpu = upscale(&cpu, view, f);
    let (w, h) = (view.0 * f, view.1 * f);
    let mut draw = glasnost_app::game::GpuDraw::new(&device, OFFSCREEN_FORMAT);
    draw.set_screen(f, (w, h));
    c.cpu_map = false;
    c.painter.dirty = true;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("lane frame"),
    });
    draw.prepare(&device, &queue, c, view, &mut encoder);
    queue.submit([encoder.finish()]);
    let rgba = render_rgba(&device, &queue, w, h, |pass| draw.draw(pass));
    assert!(!c.cpu_map, "the GPU battlefield did not fail");
    c.cpu_map = true;
    let file = format!("game-gpu-{f}x.png");
    std::fs::write(shots.dir.join(&file), png::png(w, h, &rgba)).unwrap();
    // Outside the battlefield (the interface) the two paths agree exactly;
    // count the pixels that differ.
    let m = c.session.as_ref().unwrap().map_rect();
    let (mut differ, mut n) = (0u64, 0u64);
    for y in 0..h as i32 {
        for x in 0..w as i32 {
            let v = glasnost_ui::geom::Point::new(x / f as i32, y / f as i32);
            if m.contains(v) {
                continue;
            }
            let at = ((y * w as i32 + x) * 4) as usize;
            n += 1;
            if rgba[at..at + 3] != cpu[at..at + 3] {
                differ += 1;
            }
        }
    }
    println!("wrote {file}: {differ} of {n} interface pixels differ from the CPU frame upscaled");
}

fn screens(dir: &std::path::Path, out: &std::path::Path, factor: i32) {
    let shots = Shots {
        dir: out.to_owned(),
        factor,
    };
    let mut c = window(factor);
    let t0 = Instant::now();
    c.mount(files(dir));
    run_jobs(&mut c);
    assert!(c.install.is_some(), "the install reads: {}", c.status);
    println!("install read in {} ms", t0.elapsed().as_millis());
    let _ = c.frame();
    shots.save(&mut c, "menu");

    let r = c.menu_targets.rect_of(&MenuHit::Settings).unwrap();
    click(&mut c, r);
    assert_eq!(c.shell.screen, Screen::Settings);
    shots.save(&mut c, "settings");
    c.key(glasnost_app::game::client::Key::Escape, true);
    c.key(glasnost_app::game::client::Key::Escape, false);
    assert_eq!(c.shell.screen, Screen::Menu);
    let _ = c.frame();

    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    assert_eq!(c.shell.screen, Screen::Setup);
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

    // Deploy the base unit where it stands; the first status line shows.
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
    c.key(glasnost_app::game::client::Key::Char('d'), true);
    c.key(glasnost_app::game::client::Key::Char('d'), false);
    ticks(&mut c, &mut clock, 120);
    // Two status lines (the start's has gone by now): the game's own
    // message text, in the game font.
    let s = c.session.as_mut().unwrap();
    s.play.say("Game started.");
    s.play.say("Nothing to do there.");
    // A sidebar tip: the pointer rests on the first icon past the wait.
    let s = c.session.as_ref().unwrap();
    let slot = s.ui_scale().rect(s.layout().cameo(0));
    let p = Point::new(slot.x + slot.w / 2, slot.y + slot.h / 2);
    let _ = c.frame();
    c.pointer_moved(p);
    for ms in [16, 1000, 16, 16] {
        clock += ms;
        c.update(clock);
        let _ = c.frame();
    }
    shots.save(&mut c, "game");
    gpu_shot(&shots, &mut c);

    // Surrender and wait for the end screen.
    let s = c.session.as_mut().unwrap();
    s.play.surrender();
    ticks(&mut c, &mut clock, 4000);
    assert!(c.end().is_some(), "the end screen shows");
    shots.save(&mut c, "end");
}

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn screens_at_one_and_two_times() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let phase = std::env::var("GLASNOST_HIDPI_PHASE").unwrap_or_else(|_| "after".into());
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../local-lane/side/hidpi/shots")
        .join(phase);
    std::fs::create_dir_all(&out).unwrap();
    for factor in [1, 2] {
        screens(std::path::Path::new(&dir), &out, factor);
    }
}
