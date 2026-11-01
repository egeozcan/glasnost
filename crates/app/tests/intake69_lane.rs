//! Local lane for M8 A3 intake 69 (S-0411 claim 19, never in CI): with
//! `GLASNOST_GAME_DIR` set, (a) resolves the sidebar's button images and
//! their palette with each of the first three sides' groups mounted (S15
//! part a) and prints, per side and file, whether it resolved, its frame
//! count and its canvas size; (b) starts a skirmish through the windowless
//! client and writes the sidebar's buttons in their states to
//! `local-lane/m8/intake69/<tag>/side<n>/` (git-ignored; the tag is
//! `GLASNOST_I69_TAG`, default `after`; `GLASNOST_LANE_OUT` moves the
//! `intake69` folder). It
//! prints counts, sizes and booleans only.
//!
//! Run with `cargo test --release -p glasnost-app --test intake69_lane --
//! --ignored --nocapture --test-threads 1`.

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::Button;
use glasnost_app::mapview::png;
use glasnost_fmt::shp::Shp;
use glasnost_map::CellXY;
use glasnost_sim::rules::Kind;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::hud::HudHit;
use glasnost_ui::menu::MenuHit;
use glasnost_ui::modes::Mode;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::theater::Theater;
use glasnost_vfs::{ByteSource, GameOptions, MountOptions, PickedFile, Vfs};

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

/// S15 part a: every file of S-0411 Import mapping's claim 19 rows, per
/// side group.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn s15_a_the_button_files_per_side() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    pollster::block_on(async {
        let mut vfs = Vfs::default();
        vfs.mount_install(files(std::path::Path::new(&dir))).await;
        vfs.mount_startup(GameOptions::default()).await;
        for side in 0..3u32 {
            let report = vfs.mount_game(Theater::Temperate, Some(side)).await;
            vfs.finish_loading();
            println!(
                "side {side}: side archive number {:?}, reaches play {}",
                report.side_number, !report.never_reaches_play
            );
            for name in glasnost_app::game::sidebar_art::FILES {
                let Some(r) = vfs.resolve(name) else {
                    println!("  {name}: not resolved");
                    continue;
                };
                let bytes = vfs.read_resolved(&r).await.unwrap_or_default();
                match Shp::parse(&bytes) {
                    Ok(s) => println!(
                        "  {name}: {} frames, canvas {} x {}",
                        s.frames.len(),
                        s.width,
                        s.height
                    ),
                    Err(e) => println!("  {name}: does not parse ({e})"),
                }
            }
            let pal = vfs.resolve(glasnost_app::game::sidebar_art::PALETTE);
            let len = match &pal {
                Some(r) => vfs.read_resolved(r).await.map_or(0, |b| b.len()),
                None => 0,
            };
            println!("  palette: resolved {}, {len} bytes", pal.is_some());
        }
    });
}

fn run_jobs(c: &mut Client) {
    while let Some(job) = c.job.take() {
        let done = pollster::block_on(run_job(job));
        c.done(done);
    }
}

fn click_at(c: &mut Client, p: Point) {
    let _ = c.frame();
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Default::default());
}

fn click(c: &mut Client, r: Rect) {
    click_at(c, Point::new(r.x + r.w / 2, r.y + r.h / 2));
}

fn ticks(c: &mut Client, clock: &mut i64, n: usize) {
    for _ in 0..n {
        *clock += 1000;
        c.update(*clock);
        if c.shell.screen != Screen::Game {
            break;
        }
    }
}

fn rect(c: &Client, hit: HudHit) -> Option<Rect> {
    c.session.as_ref()?.hud_targets().rect_of(&hit)
}

/// The sidebar's buttons, from the options button to below the scroll
/// buttons, enlarged.
struct Shots {
    dir: std::path::PathBuf,
}

impl Shots {
    fn save(&self, c: &mut Client, name: &str) {
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
        let l = glasnost_ui::hud::Layout::new(c.size);
        // The sidebar's column, whole, enlarged 3 times.
        let (x0, cw) = (l.bar.x, l.bar.w);
        let k = 3;
        let mut out = Vec::with_capacity((cw * h * k * k * 4) as usize);
        for y in 0..h * k {
            for x in 0..cw * k {
                let i = (((y / k) * w + x0 + x / k) * 4) as usize;
                out.extend_from_slice(&rgba[i..i + 4]);
            }
        }
        let path = self.dir.join(format!("{name}.png"));
        std::fs::write(&path, png::png((cw * k) as u32, (h * k) as u32, &out)).unwrap();
        println!("wrote {name}.png");
    }
}

/// The number of icons in the shown tab.
fn icons(c: &Client) -> usize {
    let s = c.session.as_ref().unwrap();
    let p = &s.play;
    glasnost_ui::hud::entries(p.world(), p.local, p.shown_tab()).len()
}

/// The local house's side index (S-0004 claim 9).
fn local_side(c: &Client) -> Option<usize> {
    let s = c.session.as_ref()?;
    glasnost_audio::events::local_side(s.play.world(), Some(s.play.local))
}

/// Starts a skirmish on map 0 with slot 0's country stepped `steps` times,
/// every player house allied (lobby team 1).
fn start(size: (i32, i32), steps: usize) -> Option<Client> {
    let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
    let mut c = Client::new();
    c.resize(size);
    c.mount(files(std::path::Path::new(&dir)));
    run_jobs(&mut c);
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    for _ in 0..steps {
        let _ = c.frame();
        let r = c.setup_targets.rect_of(&SetupHit::Country(0, 1))?;
        click(&mut c, r);
    }
    for slot in 0..2 {
        let _ = c.frame();
        // spec: S-0412 claim 15 (no team, then teams 1 … 4: one step gives team 1)
        let r = c.setup_targets.rect_of(&SetupHit::Team(slot, 1)).unwrap();
        click(&mut c, r);
    }
    let _ = c.frame();
    let r = c.setup_targets.rect_of(&SetupHit::Start_).unwrap();
    c.next_seed = 12345;
    click(&mut c, r);
    run_jobs(&mut c);
    (c.shell.screen == Screen::Game).then_some(c)
}

/// The local base unit, deployed where it stands; returns its cell.
fn deploy(c: &mut Client, clock: &mut i64) -> Option<CellXY> {
    ticks(c, clock, 5);
    let s = c.session.as_mut().unwrap();
    let local = s.play.local;
    let mcv = s
        .play
        .world()
        .objects
        .values()
        .find(|o| o.owner == Some(local) && o.kind == Kind::Vehicle && o.on_map)
        .map(|o| (o.id, o.pos.cell()))?;
    s.play.select_box(&[mcv.0], false);
    c.key(Key::Char('d'), true);
    c.key(Key::Char('d'), false);
    ticks(c, clock, 200);
    Some(mcv.1)
}

/// The tab-feeding building types the lane places so that the tabs fill:
/// one of each factory kind and one 2 × 2 power plant, chosen by their
/// fields.
fn spawn_base(c: &mut Client, clock: &mut i64, home: CellXY) {
    let s = c.session.as_mut().unwrap();
    let local = s.play.local;
    let w = &mut s.play.game.world;
    let r = w.rules_arc();
    let mut cells =
        (0..16).map(|k| CellXY::new(home.x - 8 + 4 * (k % 4), home.y + 4 + 4 * (k / 4)));
    let mut kinds = std::collections::BTreeSet::new();
    for b in &r.buildings {
        let pick = match b.factory {
            Some(f) if !b.naval => kinds.insert(format!("{f:?}")),
            None => {
                b.foundation == (2, 2)
                    && b.raw.econ.power.unwrap_or(0) > 0
                    && kinds.insert("power".into())
            }
            _ => false,
        };
        if !pick {
            continue;
        }
        let Some(cell) = cells.next() else { break };
        glasnost_sim::testkit::spawn(w, Kind::Building, &b.name, local.0, cell, 0);
    }
    ticks(c, clock, 30);
}

/// S15 part b in our engine: the sidebar buttons in each state, for each
/// of the first three sides whose country the lobby offers.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn s15_b_the_sidebar_buttons_in_play() {
    if std::env::var_os("GLASNOST_GAME_DIR").is_none() {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    }
    let tag = std::env::var("GLASNOST_I69_TAG").unwrap_or_else(|_| "after".into());
    let size = (1024, 600);
    let mut done = std::collections::BTreeSet::new();
    for steps in 0..14 {
        let Some(mut c) = start(size, steps) else {
            println!("country step {steps}: no game");
            continue;
        };
        let side = local_side(&c);
        println!("country step {steps}: local side {side:?}");
        let Some(side) = side.filter(|s| *s < 3 && !done.contains(s)) else {
            continue;
        };
        done.insert(side);
        let out = png::lane_out("m8/intake69")
            .join(&tag)
            .join(format!("side{side}"));
        std::fs::create_dir_all(&out).unwrap();
        let shots = Shots { dir: out };
        let mut clock = 0;
        let away = Point::new(10, 10);
        c.pointer_moved(away);
        ticks(&mut c, &mut clock, 2);
        shots.save(&mut c, "a-start");
        let Some(home) = deploy(&mut c, &mut clock) else {
            println!("  no base unit");
            continue;
        };
        c.pointer_moved(away);
        shots.save(&mut c, "b-deployed");
        // Sell and repair, entered and left by their buttons.
        for (mode, name) in [(Mode::Sell, "sell"), (Mode::Repair, "repair")] {
            let Some(r) = rect(&c, HudHit::Mode(mode)) else {
                continue;
            };
            click(&mut c, r);
            c.pointer_moved(away);
            shots.save(&mut c, &format!("c-{name}-on"));
            click(&mut c, r);
            c.pointer_moved(away);
            shots.save(&mut c, &format!("c-{name}-off"));
            // The pointer resting over it, and a held press.
            c.pointer_moved(Point::new(r.x + r.w / 2, r.y + r.h / 2));
            shots.save(&mut c, &format!("c-{name}-hover"));
            let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
            c.pointer_down(Button::Left, p);
            shots.save(&mut c, &format!("c-{name}-held"));
            c.pointer_up(Button::Left, p, Default::default());
            click(&mut c, r);
            c.pointer_moved(away);
        }
        spawn_base(&mut c, &mut clock, home);
        // Each tab shown in turn.
        for t in 0..4 {
            if let Some(r) = rect(&c, HudHit::Tab(t)) {
                click(&mut c, r);
                c.pointer_moved(away);
                println!("  tab {t}: {} icons", icons(&c));
                shots.save(&mut c, &format!("d-tab{t}"));
            } else {
                println!("  tab {t}: unavailable");
            }
        }
        // The scroll buttons on the first tab: held, then over and off.
        if let Some(r) = rect(&c, HudHit::Tab(0)) {
            click(&mut c, r);
        }
        // At the full height (all icons fit: disabled), then in a short
        // window (more icons than fit: enabled).
        for (height, name) in [(size.1, "e-scroll"), (400, "e-scroll-short")] {
            c.resize((size.0, height));
            let l = c.session.as_ref().unwrap().layout();
            println!(
                "  height {height}: rows {}, tab 0 icons {}",
                l.rows(),
                icons(&c)
            );
            // The scroll-down button.
            let down = l.scroll[1];
            let p = Point::new(down.x + down.w / 2, down.y + down.h / 2);
            let off = Point::new(down.x - 30, down.y - 100);
            c.pointer_moved(away);
            shots.save(&mut c, name);
            c.pointer_moved(p);
            shots.save(&mut c, &format!("{name}-hover"));
            c.pointer_down(Button::Left, p);
            shots.save(&mut c, &format!("{name}-held"));
            c.pointer_moved(off);
            shots.save(&mut c, &format!("{name}-held-off"));
            c.pointer_up(Button::Left, off, Default::default());
            c.pointer_moved(away);
        }
        c.resize(size);
        // A building of the first tab built until it waits for placement:
        // its tab flashes, shown and not shown.
        let cameo = rect(&c, HudHit::Cameo(0));
        if let Some(r) = cameo {
            click(&mut c, r);
            c.pointer_moved(away);
            let mut ready = false;
            for _ in 0..4000 {
                ticks(&mut c, &mut clock, 1);
                let s = c.session.as_ref().unwrap();
                let w = s.play.world();
                let local = s.play.local;
                ready = glasnost_ui::hud::entries(w, local, 0).iter().any(|e| {
                    matches!(e, glasnost_ui::hud::Entry::Type(t)
                        if glasnost_sim::econ::sidebar::icon_state(w, local, *t).is_some_and(|s| s.ready))
                });
                if ready {
                    break;
                }
            }
            println!("  a first-tab building waits for placement: {ready}");
            if ready {
                for k in 0..4 {
                    ticks(&mut c, &mut clock, 5);
                    shots.save(&mut c, &format!("f-lit-shown-{k}"));
                }
                if let Some(r) = rect(&c, HudHit::Tab(2)).or_else(|| rect(&c, HudHit::Tab(3))) {
                    click(&mut c, r);
                    c.pointer_moved(away);
                    for k in 0..4 {
                        ticks(&mut c, &mut clock, 5);
                        shots.save(&mut c, &format!("f-lit-other-{k}"));
                    }
                }
            }
        }
        if done.len() == 3 {
            break;
        }
    }
    println!("sides shot: {done:?}");
}
