//! Local lane for the M8 A4 round 5 play notes (never in CI): with
//! `GLASNOST_GAME_DIR` set, starts a skirmish through the windowless client
//! and plays the round's fixes through the client's own input: the radar's
//! view outline (note 22), the superweapon countdown (25), the aim image
//! with the selection cleared on arming (27), the type-select key and the
//! control groups (31), the icon tooltips (33) and the defence range while
//! placing (34). Each screen drawn on the CPU goes to `local-lane/m8/a4-r5/ui/`
//! (git-ignored; `GLASNOST_LANE_OUT` moves it). Types are chosen by their
//! fields, never by name; it prints counts and booleans only.
//!
//! Run with `cargo test -p glasnost-app --test r5_lane -- --ignored
//! --nocapture`. `GLASNOST_UI_MAP=n` picks the n-th offered map (default 0).

#![cfg(not(target_arch = "wasm32"))]

use std::rc::Rc;

use glasnost_app::game::client::{Client, Key, run_job};
use glasnost_app::game::play::{Button, Tool};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_sim::econ::TypeRef;
use glasnost_sim::rules::Kind;
use glasnost_sim::superweapons::charge;
use glasnost_sim::targeting::Modifiers;
use glasnost_sim::world::ObjectId;
use glasnost_ui::geom::{Point, Rect};
use glasnost_ui::hud::{self, HudHit};
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
        let done = pollster::block_on(run_job(job));
        c.done(done);
    }
}

fn click_with(c: &mut Client, b: Button, p: Point, mods: Modifiers) {
    // The hit targets come from the last drawn frame.
    let _ = c.frame();
    c.modifiers(mods);
    c.pointer_moved(p);
    c.pointer_down(b, p);
    c.pointer_up(b, p, mods);
    c.modifiers(Modifiers::default());
}

fn click(c: &mut Client, r: Rect) {
    click_with(
        c,
        Button::Left,
        Point::new(r.x + r.w / 2, r.y + r.h / 2),
        Modifiers::default(),
    );
}

fn key_with(c: &mut Client, k: Key, mods: Modifiers) {
    c.modifiers(mods);
    c.key(k, true);
    c.key(k, false);
    c.modifiers(Modifiers::default());
}

struct Shots {
    dir: std::path::PathBuf,
}

impl Shots {
    fn save(&self, c: &mut Client, name: &str) {
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
        let path = self.dir.join(format!("{name}.png"));
        std::fs::write(&path, png::png(w as u32, h as u32, &rgba)).unwrap();
        println!("wrote {name}.png");
    }

    /// Writes `name-zoom.png`: the 300 × 200 pixels around `at`, enlarged
    /// twice.
    fn zoom(&self, c: &mut Client, name: &str, at: Point) {
        let rgba = c.frame_rgba();
        let (w, h) = c.size;
        let (cw, ch, k) = (300, 200, 2);
        let (x0, y0) = (
            (at.x - cw / 2).clamp(0, w - cw),
            (at.y - ch / 2).clamp(0, h - ch),
        );
        let mut out = Vec::with_capacity((cw * ch * k * k * 4) as usize);
        for y in 0..ch * k {
            for x in 0..cw * k {
                let i = (((y0 + y / k) * w + x0 + x / k) * 4) as usize;
                out.extend_from_slice(&rgba[i..i + 4]);
            }
        }
        let path = self.dir.join(format!("{name}-zoom.png"));
        std::fs::write(&path, png::png((cw * k) as u32, (ch * k) as u32, &out)).unwrap();
        println!("wrote {name}-zoom.png");
    }
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

/// Starts a skirmish from the menu on a window of `size`.
fn start(size: (i32, i32)) -> Option<(Client, Shots)> {
    let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
    let out = png::lane_out("m8/a4-r5/ui");
    std::fs::create_dir_all(&out).unwrap();
    let mut c = Client::new();
    c.resize(size);
    c.mount(files(std::path::Path::new(&dir)));
    run_jobs(&mut c);
    let maps = c.install.as_ref().expect("the install reads").maps.len();
    let map = env_num("GLASNOST_UI_MAP", 0).min(maps - 1);
    let _ = c.frame();
    let r = c.menu_targets.rect_of(&MenuHit::Skirmish).unwrap();
    click(&mut c, r);
    for _ in 0..map {
        let _ = c.frame();
        let r = c.setup_targets.rect_of(&SetupHit::Map(1)).unwrap();
        click(&mut c, r);
    }
    let _ = c.frame();
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
    Some((c, Shots { dir: out }))
}

/// Centres the view on `cell`.
fn look_at(c: &mut Client, cell: CellXY) {
    let s = c.session.as_mut().unwrap();
    let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
    s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
}

/// The mean screen point of the pixels that hit object `obj`.
fn centre_of(c: &mut Client, obj: ObjectId) -> Option<Point> {
    let s = c.session.as_mut().unwrap();
    let m = s.map_rect();
    let (mut sx, mut sy, mut n) = (0i64, 0i64, 0i64);
    for y in (m.y..m.y + m.h).step_by(2) {
        for x in (m.x..m.x + m.w).step_by(2) {
            if s.under(Point::new(x, y)).1 == Some(obj) {
                (sx, sy, n) = (sx + i64::from(x), sy + i64::from(y), n + 1);
            }
        }
    }
    (n > 0).then(|| Point::new((sx / n) as i32, (sy / n) as i32))
}

/// The local base unit, deployed where it stands; returns its cell.
fn deploy(c: &mut Client, clock: &mut i64) -> CellXY {
    ticks(c, clock, 5);
    let s = c.session.as_mut().unwrap();
    let local = s.play.me();
    let mcv = s
        .play
        .world()
        .objects
        .values()
        .find(|o| o.owner == Some(local) && o.kind == Kind::Vehicle && o.on_map)
        .map(|o| (o.id, o.pos.cell()))
        .expect("a base unit");
    s.play.select_box(&[mcv.0], false);
    c.key(Key::Char('d'), true);
    c.key(Key::Char('d'), false);
    ticks(c, clock, 200);
    mcv.1
}

/// Writes the zoom around the radar, found by its hit target.
fn radar_zoom(c: &mut Client, shots: &Shots, name: &str) {
    let _ = c.frame();
    let r = c
        .session
        .as_ref()
        .unwrap()
        .hud_targets()
        .rect_of(&HudHit::Radar);
    if let Some(r) = r {
        shots.zoom(c, name, Point::new(r.x + r.w / 2, r.y + r.h / 2));
    }
}

fn selection_len(c: &Client) -> usize {
    c.session.as_ref().unwrap().play.selection.len()
}

/// Every round 5 note that the real data can show, in one game.
#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn round_5_notes_on_the_real_data() {
    let Some((mut c, shots)) = start((1280, 800)) else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let mut clock = 0i64;
    let base = deploy(&mut c, &mut clock);
    let s = c.session.as_mut().unwrap();
    let local = s.play.me();
    let w = s.play.world();
    let r = w.rules();
    let sw_rules = &r.superweapons;
    // The building types to place: the strongest power plant, a radar
    // building, and a superweapon building whose type shows a countdown
    // and aims.
    let plain = |i: usize| sw_rules.building_sw.get(i).is_none_or(|s| s[0].is_none());
    let power = (0..r.buildings.len())
        .filter(|&i| plain(i) && r.buildings[i].raw.econ.power.unwrap_or(0) > 0)
        .max_by_key(|&i| r.buildings[i].raw.econ.power.unwrap_or(0))
        .map(|i| r.buildings[i].name.clone());
    let radar = (0..r.buildings.len())
        .find(|&i| plain(i) && r.buildings[i].raw.econ.radar == Some(true))
        .map(|i| r.buildings[i].name.clone());
    let sw_building = (0..r.buildings.len()).find_map(|i| {
        let sw = sw_rules.building_sw.get(i)?[0]?;
        let t = sw_rules.types.get(sw as usize)?;
        (t.show_timer && t.action.is_some()).then(|| (r.buildings[i].name.clone(), sw))
    });
    println!(
        "power plant {}, radar {}, countdown superweapon {}",
        power.is_some(),
        radar.is_some(),
        sw_building.is_some()
    );
    let wm = &mut s.play.game.world;
    wm.houses[usize::from(local.0)].credits = 100_000;
    if let Some(p) = &power {
        for k in 0..4 {
            let at = CellXY::new(base.x - 10 + 4 * k, base.y + 7);
            glasnost_sim::testkit::spawn(wm, Kind::Building, p, local.0, at, 0);
        }
    }
    if let Some(n) = &radar {
        let at = CellXY::new(base.x + 6, base.y - 7);
        glasnost_sim::testkit::spawn(wm, Kind::Building, n, local.0, at, 0);
    }
    let sw = sw_building.as_ref().map(|(n, sw)| {
        let at = CellXY::new(base.x - 9, base.y - 8);
        glasnost_sim::testkit::spawn(wm, Kind::Building, n, local.0, at, 0);
        *sw
    });
    ticks(&mut c, &mut clock, 300);

    // Notes 22 and 25: the radar's view outline and the countdown.
    let s = c.session.as_mut().unwrap();
    let outline = s.radar_view_box();
    println!(
        "note 22: radar shown {}, outline {:?}",
        s.radar.is_some(),
        outline.map(|b| b.edges)
    );
    println!("note 25: countdown rows {}", s.countdown_rows().len());
    shots.save(&mut c, "r5-22-25-radar-and-countdown");
    if let Some(b) = outline {
        println!("note 22: radar image {:?}", b.image);
        radar_zoom(&mut c, &shots, "r5-22-radar");
    }
    let s = c.session.as_mut().unwrap();
    let centre = s.centre();
    s.set_centre((centre.0 + 900, centre.1 + 450));
    let _ = c.frame();
    let s = c.session.as_mut().unwrap();
    let moved = s.radar_view_box();
    println!("note 22: moved outline {:?}", moved.map(|b| b.edges));
    if let Some(b) = moved {
        println!("note 22: radar image {:?}", b.image);
        radar_zoom(&mut c, &shots, "r5-22-radar-moved");
    }
    look_at(&mut c, base);

    // Note 31: four armed vehicles of one type by the base, two far off.
    let s = c.session.as_mut().unwrap();
    let r = s.play.world().rules();
    let tank = (0..r.vehicles.len()).find(|&i| {
        let v = &r.vehicles[i];
        v.combat.slots[0].is_some() && !v.deployer && v.raw.combat.naval != Some(true)
    });
    let tank = tank
        .map(|i| r.vehicles[i].name.clone())
        .expect("an armed vehicle type");
    let wm = &mut s.play.game.world;
    let near: Vec<ObjectId> = (0..4)
        .map(|k| {
            let at = CellXY::new(base.x + 2 + k, base.y + 4);
            glasnost_sim::testkit::spawn(wm, Kind::Vehicle, &tank, local.0, at, 0)
        })
        .collect();
    let far: Vec<ObjectId> = (0..2)
        .map(|k| {
            let at = CellXY::new(base.x + 2 + k, base.y + 40);
            glasnost_sim::testkit::spawn(wm, Kind::Vehicle, &tank, local.0, at, 0)
        })
        .collect();
    ticks(&mut c, &mut clock, 3);
    look_at(&mut c, CellXY::new(base.x + 3, base.y + 3));
    let _ = c.frame();
    let on_screen: Vec<bool> = near
        .iter()
        .chain(&far)
        .map(|&id| centre_of(&mut c, id).is_some())
        .collect();
    println!("note 31: drawn near and far {on_screen:?}");
    let s = c.session.as_mut().unwrap();
    s.play.selection = vec![near[0]];
    s.clock_ms = 10_000;
    key_with(&mut c, Key::Char('t'), Modifiers::default());
    println!(
        "note 31: one press: {} selected (expected 4)",
        selection_len(&c)
    );
    shots.save(&mut c, "r5-31-type-screen");
    let s = c.session.as_mut().unwrap();
    s.clock_ms = 10_200;
    key_with(&mut c, Key::Char('t'), Modifiers::default());
    println!(
        "note 31: second press 200 ms later: {} selected (expected 6)",
        selection_len(&c)
    );
    let s = c.session.as_mut().unwrap();
    s.play.selection = vec![near[0]];
    s.clock_ms = 12_000;
    key_with(&mut c, Key::Char('t'), Modifiers::default());
    let s = c.session.as_mut().unwrap();
    s.clock_ms = 13_000;
    key_with(&mut c, Key::Char('t'), Modifiers::default());
    println!(
        "note 31: presses 1 s apart: {} selected (expected 4)",
        selection_len(&c)
    );
    // Held: a click on one tank takes its type on the screen.
    c.session.as_mut().unwrap().play.selection.clear();
    let on_tank = centre_of(&mut c, near[1]).expect("a near tank is drawn");
    c.session.as_mut().unwrap().clock_ms = 20_000;
    c.key(Key::Char('t'), true);
    click_with(&mut c, Button::Left, on_tank, Modifiers::default());
    println!(
        "note 31: held click: {} selected (expected 4)",
        selection_len(&c)
    );
    let shift = Modifiers {
        force_select: true,
        ..Modifiers::default()
    };
    click_with(&mut c, Button::Left, on_tank, shift);
    println!(
        "note 31: held toggle on a selected one: {} selected (expected 0)",
        selection_len(&c)
    );
    c.key(Key::Char('t'), false);
    click_with(&mut c, Button::Left, on_tank, Modifiers::default());
    println!(
        "note 31: plain click after release: {} selected (expected 1)",
        selection_len(&c)
    );
    // Type-select boxes around one near tank only (claims 19 and 29).
    let one = centre_of(&mut c, near[0]).expect("the first near tank is drawn");
    let drag = |c: &mut Client, mods: Modifiers| {
        let (a, b) = (
            Point::new(one.x - 14, one.y - 14),
            Point::new(one.x + 14, one.y + 18),
        );
        let _ = c.frame();
        c.modifiers(mods);
        c.pointer_moved(a);
        c.pointer_down(Button::Left, a);
        for k in 1..=4 {
            c.pointer_moved(Point::new(
                a.x + (b.x - a.x) * k / 4,
                a.y + (b.y - a.y) * k / 4,
            ));
        }
        c.pointer_up(Button::Left, b, mods);
        c.modifiers(Modifiers::default());
        let s = c.session.as_ref().unwrap();
        (s.play.selection.clone(), s.play.selected.len())
    };
    c.session.as_mut().unwrap().play.selection.clear();
    let (plain, _) = drag(&mut c, Modifiers::default());
    println!(
        "note 31: plain box: {} selected (expected 1), it is the box's tank {}",
        plain.len(),
        plain == vec![near[0]]
    );
    c.session.as_mut().unwrap().play.selection.clear();
    let spoken = c.session.as_ref().unwrap().play.selected.len();
    // Each press well after the one before, so none widens to the map.
    c.session.as_mut().unwrap().clock_ms = 30_000;
    c.key(Key::Char('t'), true);
    let (typed, after) = drag(&mut c, Modifiers::default());
    println!(
        "note 31: type-select box: {} selected (expected 4), far ones {}, silent {}",
        typed.len(),
        typed.iter().any(|id| far.contains(id)),
        after == spoken
    );
    c.key(Key::Char('t'), false);
    c.session.as_mut().unwrap().play.selection = vec![far[0]];
    c.session.as_mut().unwrap().clock_ms = 40_000;
    c.key(Key::Char('t'), true);
    let (added, _) = drag(&mut c, shift);
    c.key(Key::Char('t'), false);
    println!(
        "note 31: additive type-select box with a far one selected: {} selected (expected 5), far one kept {}",
        added.len(),
        added.contains(&far[0])
    );
    shots.save(&mut c, "r5-31-type-box");
    // Control groups: Ctrl+1 of two near tanks; Shift+1 with a far one
    // selected adds the group to the selection (S-0214 claim 33).
    let ctrl = Modifiers {
        force_fire: true,
        ..Modifiers::default()
    };
    c.session.as_mut().unwrap().play.selection = vec![near[0], near[1]];
    key_with(&mut c, Key::Char('1'), ctrl);
    c.session.as_mut().unwrap().play.selection = vec![far[0]];
    key_with(&mut c, Key::Char('1'), shift);
    let s = c.session.as_mut().unwrap();
    println!(
        "note 31: group 1 has {} (expected 2), Shift+1 selection {} (expected 3)",
        s.play.group(1).len(),
        s.play.selection.len()
    );
    s.play.selection.clear();
    let before = s.centre();
    s.clock_ms = 30_000;
    key_with(&mut c, Key::Char('1'), Modifiers::default());
    let s = c.session.as_mut().unwrap();
    println!(
        "note 31: group 1 selected: {} selected, view moved {}",
        s.play.selection.len(),
        s.centre() != before
    );
    s.clock_ms = 30_300;
    key_with(&mut c, Key::Char('1'), Modifiers::default());
    let _ = c.frame();
    println!(
        "note 31: second press: view moved {}",
        c.session.as_ref().unwrap().centre() != before
    );
    shots.save(&mut c, "r5-31-group-went-to");
    look_at(&mut c, base);

    // Note 27: arm the superweapon from its icon with a tank selected.
    if let Some(sw) = sw {
        let s = c.session.as_mut().unwrap();
        charge::set_charge(&mut s.play.game.world, local, sw, 100);
        ticks(&mut c, &mut clock, 3);
        let s = c.session.as_mut().unwrap();
        let w = s.play.world();
        let at = (0..4).find_map(|tab| {
            hud::entries(w, local, tab)
                .iter()
                .position(|e| *e == hud::Entry::Superweapon(sw))
                .map(|n| (tab, n))
        });
        println!(
            "note 27: ready {}, icon found {}",
            charge::offers_targeting(w, local, sw),
            at.is_some()
        );
        if let Some((tab, n)) = at {
            s.play.tab_click(tab);
            s.play.selection = vec![near[0]];
            let _ = c.frame();
            let cameo = c
                .session
                .as_ref()
                .unwrap()
                .hud_targets()
                .rect_of(&HudHit::Cameo(n));
            // Note 33: the superweapon icon's tooltip.
            if let Some(r) = cameo {
                c.pointer_moved(Point::new(r.x + r.w / 2, r.y + r.h / 2));
                let _ = c.frame();
                shots.zoom(
                    &mut c,
                    "r5-33-tooltip-superweapon",
                    Point::new(r.x - 60, r.y),
                );
                click(&mut c, r);
            }
            let s = c.session.as_ref().unwrap();
            println!(
                "note 27: armed {}, selection after arming {}",
                matches!(s.play.tool, Tool::Superweapon { .. }),
                s.play.selection.len()
            );
            let m = s.map_rect();
            let p = Point::new(m.x + m.w / 2, m.y + m.h / 2);
            c.pointer_moved(p);
            let _ = c.frame();
            shots.save(&mut c, "r5-27-aim");
            shots.zoom(&mut c, "r5-27-aim", p);
            c.session.as_mut().unwrap().play.tool = Tool::Select;
        }
    }

    // Note 33: a building icon's tooltip with its price.
    let s = c.session.as_mut().unwrap();
    s.play.tab_click(0);
    let _ = c.frame();
    let cameo = c
        .session
        .as_ref()
        .unwrap()
        .hud_targets()
        .rect_of(&HudHit::Cameo(0));
    if let Some(r) = cameo {
        c.pointer_moved(Point::new(r.x + r.w / 2, r.y + r.h / 2));
        let _ = c.frame();
        shots.save(&mut c, "r5-33-tooltip-building");
        shots.zoom(&mut c, "r5-33-tooltip-building", Point::new(r.x - 60, r.y));
    }

    // Note 34: placing an armed building shows its weapon's range.
    let s = c.session.as_mut().unwrap();
    let r = s.play.world().rules();
    let armed = (0..r.buildings.len()).find(|&i| {
        let b = &r.buildings[i];
        plain_building(&r.superweapons.building_sw, i)
            && b.combat
                .slots
                .iter()
                .flatten()
                .any(|&id| r.combat.weapon(id).range > 0)
    });
    println!("note 34: an armed building type {}", armed.is_some());
    if let Some(i) = armed {
        s.play.tool = Tool::Place(TypeRef {
            kind: Kind::Building,
            index: i as u32,
        });
        println!(
            "note 34: range {:?}",
            s.play.placement_range().map(|x| x > 0)
        );
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
        let _ = c.frame();
        shots.save(&mut c, "r5-34-range");
        c.session.as_mut().unwrap().play.tool = Tool::Select;
    }
}

fn plain_building(building_sw: &[[Option<u32>; 2]], i: usize) -> bool {
    building_sw.get(i).is_none_or(|s| s[0].is_none())
}
