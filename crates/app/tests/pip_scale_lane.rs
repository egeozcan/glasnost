//! Local lane for the pip scale and the control-group number (never in CI;
//! S-0111 claims 10 to 17): with `GLASNOST_GAME_DIR` set, loads the first
//! real maps whose sandbox game gives one house a building, a vehicle and
//! an infantry unit, puts those three (and the house's first harvester, if
//! any) in control groups 1 to 4, selects them as that house and writes a
//! close-up view to `local-lane/m8/a3-46/` (`GLASNOST_LANE_OUT` names
//! another folder). For the picture only, the vehicle is set to half its
//! strength, its owner given the unit-repair ability, and the harvester
//! given half a load of the first resource and a quarter of the second; the
//! game is not run on. Prints counts only. Without the variable it does
//! nothing.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::rc::Rc;

use glasnost_app::game::status_font;
use glasnost_app::mapview::combat::GroupFont;
use glasnost_app::mapview::{find, load, png};
use glasnost_render::compose::{ComposeQuirks, DEPTH_REFERENCE};
use glasnost_render::projection::{limit_centre, project_lifted, view_origin};
use glasnost_render::scene::{Filter, View};
use glasnost_sim::rules::Kind;
use glasnost_sim::world::{Object, ObjectId};
use glasnost_vfs::source::{FileSource, Unopened};
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

/// A 640 × 400 view centred on world pixel `centre`, limited to the map.
fn view_at(map: &glasnost_map::Map, centre: (i32, i32)) -> View {
    let size = (640, 400);
    let c = limit_centre(centre, size, map.footprint.width, map.footprint.local);
    View {
        origin: view_origin(c, size),
        size: (size.0 as u32, size.1 as u32),
        reference: DEPTH_REFERENCE,
    }
}

#[test]
fn grouped_units_show_their_pip_scales() {
    let Some(dir) = std::env::var_os("GLASNOST_GAME_DIR") else {
        eprintln!("GLASNOST_GAME_DIR not set: skipped");
        return;
    };
    let out_dir = std::env::var_os("GLASNOST_LANE_OUT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../local-lane/m8/a3-46")
        });
    std::fs::create_dir_all(&out_dir).unwrap();
    let want: usize = std::env::var("GLASNOST_LANE_MAPS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2);
    pollster::block_on(async {
        let mut vfs = Vfs::default();
        vfs.mount_install(files(std::path::Path::new(&dir))).await;
        vfs.mount_startup(GameOptions::default()).await;
        let font = status_font::read(&vfs).await;
        println!(
            "game font: {}",
            if font.is_ok() { "read" } else { "missing" }
        );
        let group_font = font.as_ref().ok().and_then(|f| GroupFont::new(f.font()));
        let maps = find::find_maps(&vfs).await;
        let mut done = 0;
        for (n, m) in maps.iter().enumerate() {
            if done >= want {
                break;
            }
            let bytes = vfs.read(&m.file).await.unwrap();
            let mut loaded = load::load(
                &mut vfs,
                bytes,
                m.archive.clone(),
                &Default::default(),
                true,
                Default::default(),
            )
            .await;
            let Some(live) = loaded.live.as_ref() else {
                continue;
            };
            let w = &live.world;
            let owned = |o: &Object, k: Kind| o.on_map && o.kind == k && o.owner.is_some();
            let Some(b) = w.objects.values().find(|o| owned(o, Kind::Building)) else {
                continue;
            };
            let near = |k: Kind| {
                w.objects
                    .values()
                    .filter(|o| owned(o, k) && o.owner == b.owner)
                    .min_by_key(|o| {
                        i64::from(o.pos.x - b.pos.x).pow(2) + i64::from(o.pos.y - b.pos.y).pow(2)
                    })
                    .map(|o| o.id)
            };
            let (Some(v), Some(i)) = (near(Kind::Vehicle), near(Kind::Infantry)) else {
                continue;
            };
            let house = b.owner.unwrap();
            let harvester = w
                .objects
                .values()
                .find(|o| owned(o, Kind::Vehicle) && o.owner == b.owner && o.econ.harvest.is_some())
                .map(|o| o.id);
            let building = b.id;
            done += 1;
            let colours = loaded.house_radar_colours(&loaded.live.as_ref().unwrap().world);
            let live = loaded.live.as_mut().unwrap();
            let strength = live
                .rules
                .unit(Kind::Vehicle, live.world.obj(v).unwrap().type_index)
                .map_or(2, |u| u.strength);
            live.world.obj_mut(v).unwrap().health = strength / 2;
            live.world.houses[usize::from(house.0)].combat.self_heal[1] = 1;
            let mut members = vec![building, v, i];
            if let Some(h) = harvester {
                let cap = live
                    .rules
                    .econ
                    .harvest
                    .vehicles
                    .get(live.world.obj(h).unwrap().type_index as usize)
                    .map_or(0, |u| u.capacity);
                if let Some(state) = live.world.obj_mut(h).unwrap().econ.harvest.as_mut() {
                    state.load = vec![cap / 2, cap / 4, 0, 0];
                }
                members.push(h);
            }
            let digits: BTreeMap<ObjectId, u8> =
                members.iter().zip(1u8..).map(|(&id, d)| (id, d)).collect();
            live.set_viewer(Some(house));
            live.set_house_colours(colours);
            live.set_group_font(group_font.clone());
            live.set_group_digits(digits);
            live.set_marks(&members, None);
            let mut scene = std::mem::take(&mut loaded.built.scene);
            {
                let (inputs, live) = loaded.split();
                live.unwrap().redraw(&inputs, &mut scene);
            }
            loaded.built.scene = scene;
            for (name, id) in [("group", v), ("building", building)] {
                let live = loaded.live.as_ref().unwrap();
                let p = live.world.reference_point(id);
                let view = view_at(&loaded.map, project_lifted(p.x, p.y, p.z));
                let f = loaded.built.scene.compose(
                    &view,
                    &Filter::default(),
                    &ComposeQuirks::default(),
                );
                let file = format!("pips-{n}-{name}.png");
                std::fs::write(
                    out_dir.join(&file),
                    png::png(view.size.0, view.size.1, &f.to_rgba()),
                )
                .unwrap();
                println!("map {n}: wrote {file}");
            }
            println!(
                "map {n}: grouped {} objects, harvester {}",
                members.len(),
                if harvester.is_some() { "yes" } else { "no" }
            );
        }
        println!("maps with a building, a vehicle and infantry of one house: {done}");
    });
}
