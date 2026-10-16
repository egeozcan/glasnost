//! Local lane for the M6c features (never in CI): with `GLASNOST_GAME_DIR`
//! set, every scene of [`SCENES`] starts its own skirmish through the
//! windowless client, puts the objects it needs on the map by their rules
//! flags (never by type name), plays the scene and writes its screens to
//! `local-lane/m6c/` (git-ignored). Prints counts, cells and ticks only,
//! never type names or rules values.
//!
//! Scenes now: infantry garrisoning a building and put out again (S-0508),
//! an engineer capturing an enemy building (S-0509) and a spy infiltrating
//! one (S-0510). A new scene is one more function and one more row of
//! [`SCENES`].
//!
//! Run with `cargo test --release -p glasnost-app --test m6c_lane --
//! --ignored --nocapture`. `GLASNOST_LANE_SCENE=a,b` plays only the scenes
//! with those keys; `GLASNOST_UI_MAP=n` picks the n-th offered map.

#![cfg(not(target_arch = "wasm32"))]
#![allow(dead_code)]

use std::rc::Rc;

use glasnost_app::game::client::{Client, run_job};
use glasnost_app::mapview::png;
use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_sim::rules::Kind;
use glasnost_sim::targeting::{self, ClickOn, Modifiers};
use glasnost_sim::world::{HouseId, Layer, ObjectId, World};
use glasnost_ui::geom::Rect;
use glasnost_ui::menu::MenuHit;
use glasnost_ui::setup::SetupHit;
use glasnost_ui::shell::Screen;
use glasnost_vfs::source::{FileSource, Unopened};
use glasnost_vfs::{ByteSource, MountOptions, PickedFile};

/// A scene: its key for `GLASNOST_LANE_SCENE` and the function that plays
/// it on a freshly started skirmish.
type Scene = (&'static str, fn(&mut Lane));

/// The scenes, in the order they are played.
const SCENES: &[Scene] = &[("garrison", garrison_and_evict)];

#[test]
#[ignore = "local lane: needs GLASNOST_GAME_DIR"]
fn m6c_scenes() {
    if std::env::var_os("GLASNOST_GAME_DIR").is_none() {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    }
    let only: Vec<String> = std::env::var("GLASNOST_LANE_SCENE")
        .map(|v| v.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();
    for (key, play) in SCENES {
        if !only.is_empty() && !only.iter().any(|o| o == key) {
            continue;
        }
        println!("== scene {key}");
        let mut lane = Lane::start().expect("the install is set");
        play(&mut lane);
    }
}

// ---------------------------------------------------------------------------
// Scenes

/// S-0508: three infantry of an occupier type are sent into a garrisonable
/// building with the click a person gives (S-0214 row I8); the building is
/// then drawn with its occupied frame (S-0108 claim 2) and finally emptied
/// by a deploy order (S-0508 claim 13).
fn garrison_and_evict(lane: &mut Lane) {
    let w = lane.world();
    let r = w.rules();
    let building = (0..r.buildings.len())
        .filter(|&b| {
            r.buildings[b].building.can_be_occupied && r.buildings[b].building.max_occupants >= 2
        })
        .min_by_key(|&b| (r.buildings[b].foundation.0 * r.buildings[b].foundation.1, b))
        .map(|b| (r.buildings[b].name.clone(), r.buildings[b].foundation));
    let man = (0..r.infantry.len())
        .find(|&i| r.infantry[i].raw.occupier.unwrap_or(false))
        .map(|i| r.infantry[i].name.clone());
    println!(
        "garrison type found: {}, occupier found: {}",
        building.is_some(),
        man.is_some()
    );
    let (Some((bname, size)), Some(man)) = (building, man) else {
        return;
    };
    let local = lane.local();
    let neutral = (0..lane.world().houses.len())
        .map(|i| HouseId(i as u8))
        .find(|&h| {
            lane.world().houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Neutral
        })
        .unwrap_or(local);
    let home = lane.home();
    let Some(site) = lane.area(CellXY::new(home.x + 6, home.y + 6), size, 2, |l| {
        l != land::WATER && l != land::ROCK
    }) else {
        println!("no open ground");
        return;
    };
    // A garrisonable building the map placed, as a person meets one; else one of ours.
    let placed = lane
        .world()
        .objects
        .values()
        .filter(|o| {
            o.kind == Kind::Building
                && o.on_map
                && glasnost_sim::structures::garrison::garrisonable(lane.world(), o.id)
                && glasnost_sim::structures::garrison::capacity(lane.world(), o.id) >= 2
        })
        .map(|o| o.id)
        .min_by_key(|&id| {
            let c = lane.world().obj(id).map_or(home, |o| o.pos.cell());
            ((c.x - home.x).pow(2) + (c.y - home.y).pow(2), id)
        });
    println!("a map-placed garrison building found: {}", placed.is_some());
    let b = placed.unwrap_or_else(|| lane.spawn(Kind::Building, &bname, neutral, site));
    let site = lane.world().obj(b).map_or(site, |o| o.pos.cell());
    println!("building at {site:?}, home {home:?}, size {size:?}");
    lane.look_at(site);
    lane.ticks(3);
    lane.save("garrison-empty");
    let mut men = Vec::new();
    for i in 0..3 {
        let at = lane.open_land(CellXY::new(site.x - 3, site.y + i * 2));
        men.push(lane.spawn(Kind::Infantry, &man, local, CellXY::new(at.x, at.y + i)));
    }
    let sent: usize = men
        .iter()
        .map(|&m| lane.order(local, m, ClickOn::Object(b)))
        .sum();
    println!("{} infantry sent, {sent} orders", men.len());
    let capacity = glasnost_sim::structures::garrison::capacity(lane.world(), b).max(0) as usize;
    for t in 0..900 {
        lane.ticks(1);
        let inside = glasnost_sim::structures::garrison::count(lane.world(), b);
        if t % 60 == 0 {
            println!("tick {}: {inside} inside", lane.world().tick);
        }
        if inside as usize == men.len().min(capacity) {
            break;
        }
    }
    lane.ticks(3);
    let owner = lane.world().obj(b).map(|o| o.owner);
    println!(
        "{} inside, owner is the local house: {}",
        glasnost_sim::structures::garrison::count(lane.world(), b),
        owner == Some(Some(local))
    );
    lane.save("garrison-occupied");
    glasnost_sim::orders::issue(
        lane.world_mut(),
        local,
        glasnost_sim::orders::Order::Deploy { actor: b },
    );
    for _ in 0..200 {
        lane.ticks(1);
        if glasnost_sim::structures::garrison::count(lane.world(), b) == 0 {
            break;
        }
    }
    lane.ticks(8);
    let on_map = men
        .iter()
        .filter(|&&m| lane.world().obj(m).is_some_and(|o| o.on_map))
        .count();
    println!(
        "after the deploy order: {} inside, {on_map} on the map",
        glasnost_sim::structures::garrison::count(lane.world(), b),
    );
    lane.save("garrison-evicted");
}

// ---------------------------------------------------------------------------
// The lane

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

fn click(c: &mut Client, r: Rect) {
    use glasnost_app::game::play::Button;
    use glasnost_ui::geom::Point;
    let p = Point::new(r.x + r.w / 2, r.y + r.h / 2);
    let _ = c.frame();
    c.pointer_moved(p);
    c.pointer_down(Button::Left, p);
    c.pointer_up(Button::Left, p, Modifiers::default());
}

struct Lane {
    c: Client,
    clock: i64,
    dir: std::path::PathBuf,
}

impl Lane {
    /// Starts a skirmish from the menu, the map revealed to the local house.
    fn start() -> Option<Lane> {
        Lane::start_on(env_num("GLASNOST_UI_MAP", 0))
    }

    /// The number of maps the install offers.
    fn map_count() -> usize {
        let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
            return 0;
        };
        let mut c = Client::new();
        c.mount(files(std::path::Path::new(&dir)));
        run_jobs(&mut c);
        c.install.as_ref().map_or(0, |i| i.maps.len())
    }

    /// Starts a skirmish on the first offered map whose world satisfies
    /// `fits`, chosen by content, never by name. Prints the map's index.
    fn start_where(fits: impl Fn(&World) -> bool) -> Option<Lane> {
        for i in 0..Lane::map_count() {
            let lane = Lane::start_on(i)?;
            if fits(lane.world()) {
                println!("map index {i} fits");
                return Some(lane);
            }
        }
        None
    }

    /// Starts a skirmish on the `map`-th offered map.
    fn start_on(map: usize) -> Option<Lane> {
        let dir = std::env::var_os("GLASNOST_GAME_DIR")?;
        let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/m6c");
        std::fs::create_dir_all(&out).unwrap();
        let mut c = Client::new();
        c.resize((1280, 800));
        c.mount(files(std::path::Path::new(&dir)));
        run_jobs(&mut c);
        let maps = c.install.as_ref().expect("the install reads").maps.len();
        let map = map.min(maps - 1);
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
        let mut lane = Lane {
            c,
            clock: 0,
            dir: out,
        };
        let s = lane.c.session.as_mut().unwrap();
        let local = s.play.local;
        glasnost_sim::econ::shroud::defeat_reveal(&mut s.play.game.world, local);
        lane.ticks(5);
        Some(lane)
    }

    fn ticks(&mut self, n: usize) {
        for _ in 0..n {
            self.clock += 1000;
            self.c.update(self.clock);
            if self.c.shell.screen != Screen::Game {
                break;
            }
        }
    }

    fn world(&self) -> &World {
        self.c.session.as_ref().unwrap().play.world()
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.c.session.as_mut().unwrap().play.game.world
    }

    fn local(&self) -> HouseId {
        self.c.session.as_ref().unwrap().play.local
    }

    /// The cell of the local house's first object on the map (its start).
    fn home(&self) -> CellXY {
        let w = self.world();
        let local = self.local();
        w.objects
            .values()
            .find(|o| o.on_map && o.owner == Some(local))
            .map(|o| o.pos.cell())
            .expect("a local object")
    }

    /// Centres the view on `cell`.
    fn look_at(&mut self, cell: CellXY) {
        let s = self.c.session.as_mut().unwrap();
        let (x, y) = glasnost_render::projection::cell_centre(cell.x, cell.y);
        s.set_centre(glasnost_render::projection::project_lifted(x, y, 0));
    }

    /// The live view's animation counts: created, with art, without a
    /// sprite, drawn in the last frame.
    fn counts(&self) -> (u64, u64, u64, usize) {
        let s = self.c.session.as_ref().unwrap();
        let k = &s.loaded.live.as_ref().expect("a live game").counts;
        (
            k.anims,
            k.anims_with_art,
            k.anims_without_sprite,
            k.anims_drawn,
        )
    }

    fn save(&mut self, name: &str) {
        let rgba = self.c.frame_rgba();
        let (w, h) = self.c.size;
        std::fs::write(
            self.dir.join(format!("{name}.png")),
            png::png(w as u32, h as u32, &rgba),
        )
        .unwrap();
        let (made, art, none, drawn) = self.counts();
        println!(
            "wrote local-lane/m6c/{name}.png (tick {}): animations created {made}, \
             with art {art}, without sprite {none}, drawn now {drawn}",
            self.world().tick
        );
    }

    fn spawn(&mut self, kind: Kind, name: &str, house: HouseId, at: CellXY) -> ObjectId {
        glasnost_sim::testkit::spawn(self.world_mut(), kind, name, house.0, at, 0)
    }

    /// Orders `actor` of `house` by a click on `target`, as a person would.
    fn order(&mut self, house: HouseId, actor: ObjectId, target: ClickOn) -> usize {
        let w = self.world();
        let r = targeting::click(w, house, &[actor], target, Modifiers::default());
        let mut n = 0;
        for o in r.orders {
            n += usize::from(glasnost_sim::orders::issue(self.world_mut(), house, o));
        }
        n
    }

    /// A player house that is not allied with the local house.
    fn enemy(&self) -> Option<HouseId> {
        let w = self.world();
        let local = self.local();
        (0..w.houses.len()).map(|i| HouseId(i as u8)).find(|&h| {
            h != local
                && !w.allied(Some(local), Some(h))
                && w.houses[usize::from(h.0)].role == glasnost_sim::world::HouseRole::Player
        })
    }

    /// The strength of `id`'s type.
    fn strength(&self, id: ObjectId) -> i32 {
        let w = self.world();
        w.obj(id)
            .and_then(|o| w.rules().unit(o.kind, o.type_index))
            .map_or(1, |u| u.strength.max(1))
    }

    /// `id`'s health in per-mille of its type's strength.
    fn permille(&self, id: ObjectId) -> i32 {
        let h = self.world().obj(id).map_or(0, |o| o.health);
        (i64::from(h) * 1000 / i64::from(self.strength(id))) as i32
    }

    /// The nearest cell to `near` whose 5 × 5 neighbourhood is legal land
    /// that is not water, so that units can stand there.
    fn open_land(&self, near: CellXY) -> CellXY {
        self.area(near, (1, 1), 2, |l| l != land::WATER && l != land::ROCK)
            .unwrap_or(near)
    }

    /// The nearest top-left cell to `near` of a `size` area that, with a
    /// `margin` of cells around it, is usable, of land types `ok` and free of
    /// ground objects.
    fn area(
        &self,
        near: CellXY,
        size: (i32, i32),
        margin: i32,
        ok: impl Fn(u8) -> bool,
    ) -> Option<CellXY> {
        let w = self.world();
        let good = |c: CellXY| {
            (-margin..size.0 + margin).all(|dx| {
                (-margin..size.1 + margin).all(|dy| {
                    let k = CellXY::new(c.x + dx, c.y + dy);
                    w.cells.usable(k)
                        && w.cells.cell(k).is_some_and(|x| ok(x.land))
                        && w.cells.list(k, Layer::Ground).is_empty()
                })
            })
        };
        w.cells
            .grid
            .iter()
            .map(|(k, _)| k)
            .filter(|&k| good(k))
            .min_by_key(|k| {
                let (dx, dy) = (i64::from(k.x - near.x), i64::from(k.y - near.y));
                (dx * dx + dy * dy, k.y, k.x)
            })
    }
}
