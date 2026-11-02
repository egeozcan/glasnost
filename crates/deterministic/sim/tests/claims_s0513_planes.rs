//! Claim tests of S-0513 claims 42 to 45 (the paradrop dispatch and the
//! spy plane) and of the launch side of S-0522 (claims 1 to 3 and 16), on
//! synthetic rules (`tests/sw_common`).

mod sw_common;

use glasnost_rules::schema::{Ruleset, Side};
use glasnost_sim::rules::{Behaviour, Kind};
use glasnost_sim::superweapons::{SwEvent, planes};
use glasnost_sim::world::{ObjectId, Target, World};
use glasnost_sim::{air, testkit, transport};
use sw_common::*;

fn of_type(w: &World, k: Kind, name: &str) -> Vec<ObjectId> {
    let ti = w.rules().index_of(k, name).unwrap();
    w.objects
        .iter()
        .filter(|(_, o)| o.kind == k && o.type_index == ti)
        .map(|(&id, _)| id)
        .collect()
}

fn type_name(w: &World, id: ObjectId) -> String {
    let o = w.obj(id).unwrap();
    w.rules().unit(o.kind, o.type_index).unwrap().name.clone()
}

/// The superweapon ruleset with three sides: Alphaland names the first,
/// Betaland the second, and the third is named only by `third`.
fn sided(third: bool) -> Ruleset {
    let mut r = ruleset();
    r.sides = vec![
        Side {
            name: "First".into(),
            members: vec!["Alphaland".into()],
        },
        Side {
            name: "Second".into(),
            members: vec!["Betaland".into()],
        },
        Side {
            name: "Third".into(),
            members: vec![],
        },
    ];
    r.sim.countries.get_mut("Alphaland").unwrap().econ.side = Some("First".into());
    let b = if third { "Third" } else { "Second" };
    r.sim.countries.get_mut("Betaland").unwrap().econ.side = Some(b.into());
    r
}

fn passenger_types(w: &World, plane: ObjectId) -> Vec<String> {
    transport::passengers(w, plane)
        .into_iter()
        .map(|p| type_name(w, p))
        .collect()
}

// --- Claim 42; S-0522 claims 2 and 3 ------------------------------------------------

#[test]
fn claim42_paradrop_launch_sends_planes_and_consumes_the_charge() {
    // spec: S-0513 claim 42 (a paradrop launch hands the cell to S-0522; no announcement); S-0522 claims 2 and 3 (a side with no position in the side list uses the second side group's list: one plane, three units)
    let r = ruleset();
    let mut w = game(&r);
    spawn(&mut w, "AIRB", ALPHA, -6, -6);
    ready_tick(&mut w, ALPHA, DROP, 200);
    w.effects.clear();
    let target = at(2, 1);
    fire(&mut w, ALPHA, DROP, target);
    step(&mut w);
    assert!(
        !rec(&w, ALPHA, DROP).ready,
        "the launch counts (S-0512 claim 8)"
    );
    assert!(!events(&w).iter().any(|e| e.0 == SwEvent::Launch));
    let planes = of_type(&w, Kind::Aircraft, "PDPLANE");
    assert_eq!(planes.len(), 1);
    let p = planes[0];
    let o = w.obj(p).unwrap();
    // spec: S-0522 claim 3 (a visiting aircraft, facing 0, in paradrop approach with the target and no destination, at cruise height)
    assert!(o.air.as_ref().unwrap().visiting);
    assert_eq!(o.beh.current, Some(Behaviour::ParadropApproach));
    assert_eq!(o.target, Some(Target::Cell(target)));
    assert_eq!(o.dest, None);
    assert_eq!(o.owner, Some(ALPHA));
    assert!(air::height(&w, p) > 0);
    assert_eq!(passenger_types(&w, p), vec!["WALKER"; 3]);
}

#[test]
fn s0522_claim2_payload_by_side_position() {
    // spec: S-0522 claim 2 (position 0: the first group's list; position 2: the third's; any other: the second's)
    let mut w = game(&sided(false));
    let a = planes::paradrop(&mut w, ALPHA, at(0, 0), false);
    assert_eq!(a.len(), 2, "first group: two entries");
    for &p in &a {
        assert_eq!(passenger_types(&w, p), vec!["WALKER"]);
    }
    let b = planes::paradrop(&mut w, BETA, at(0, 0), false);
    assert_eq!(b.len(), 1, "second group");
    assert_eq!(passenger_types(&w, b[0]), vec!["WALKER"; 3]);
    let mut w = game(&sided(true));
    let c = planes::paradrop(&mut w, BETA, at(0, 0), false);
    assert_eq!(c.len(), 1, "third group");
    assert_eq!(passenger_types(&w, c[0]), vec!["JUMPER"; 2]);
}

#[test]
fn s0522_claim3_american_list_and_reverse_release_order() {
    // spec: S-0522 claim 2 (the American paradrop always uses the American list); claim 3 (the last paratrooper created is the first released)
    let mut w = game(&sided(false));
    let planes = planes::paradrop(&mut w, ALPHA, at(0, 0), true);
    assert_eq!(planes.len(), 2);
    assert_eq!(passenger_types(&w, planes[0]), vec!["WALKER"]);
    let ps = transport::passengers(&w, planes[1]);
    assert_eq!(ps.len(), 2);
    assert!(ps[0] > ps[1], "the later-created paratrooper is first");
    for p in ps {
        let o = w.obj(p).unwrap();
        assert_eq!(o.transport.riding, Some(planes[1]));
        assert!(!o.on_map);
    }
}

#[test]
fn s0522_claim2_list_length_mismatches() {
    // spec: S-0522 claim 2 (different lengths: the American, first and third lists send nothing; the second sends the entries with a count, the importer warns; extra counts are ignored)
    let mut r = ruleset();
    let g = &mut r.sim.general.superweapons;
    g.amer_para_drop_num = Some(vec![1]);
    g.sov_para_drop_inf = Some(vec!["WALKER".into(), "JUMPER".into()]);
    g.sov_para_drop_num = Some(vec![2]);
    g.yuri_para_drop_num = Some(vec![2, 5]);
    let mut w = game(&r);
    assert!(planes::paradrop(&mut w, ALPHA, at(0, 0), true).is_empty());
    let s = planes::paradrop(&mut w, ALPHA, at(0, 0), false);
    assert_eq!(s.len(), 1);
    assert_eq!(passenger_types(&w, s[0]), vec!["WALKER"; 2]);
    assert!(
        w.rules()
            .superweapons
            .warnings
            .iter()
            .any(|x| x.contains("Sov"))
    );
    assert!(w.rules().superweapons.payloads[2].is_none());
}

#[test]
fn s0522_claim3_negative_and_zero_counts() {
    // spec: S-0522 claim 3 (a count of 0 sends a plane with no passengers; a negative count is treated as 0 with an importer warning)
    let mut r = ruleset();
    let g = &mut r.sim.general.superweapons;
    g.sov_para_drop_inf = Some(vec!["WALKER".into(), "WALKER".into()]);
    g.sov_para_drop_num = Some(vec![0, -4]);
    let mut w = game(&r);
    let s = planes::paradrop(&mut w, ALPHA, at(0, 0), false);
    assert_eq!(s.len(), 2);
    assert!(s.iter().all(|&p| transport::count(&w, p) == 0));
    assert!(
        w.rules()
            .superweapons
            .warnings
            .iter()
            .any(|x| x.contains("negative"))
    );
}

#[test]
fn claim42_no_plane_type_or_illegal_cell_sends_nothing() {
    // spec: S-0513 claim 42 (no plane type or an illegal cell: nothing sent, the launch still counts); S-0522 claims 1 and 2
    let mut r = ruleset();
    r.sim.aircraft.remove("PDPLANE");
    r.registries
        .get_mut("aircraft")
        .unwrap()
        .retain(|n| n != "PDPLANE");
    let mut w = game(&r);
    spawn(&mut w, "AIRB", ALPHA, -6, -6);
    ready_tick(&mut w, ALPHA, DROP, 200);
    fire(&mut w, ALPHA, DROP, at(0, 0));
    step(&mut w);
    assert!(!rec(&w, ALPHA, DROP).ready, "the launch counts");
    assert!(of_type(&w, Kind::Infantry, "WALKER").is_empty());
    let mut w = game(&ruleset());
    let off = glasnost_map::CellXY::new(-5, -5);
    assert!(planes::paradrop(&mut w, ALPHA, off, false).is_empty());
    assert!(planes::spy_plane(&mut w, ALPHA, off).is_empty());
}

#[test]
fn s0522_claim16_draw_order_plane_arrival_passengers() {
    // spec: S-0522 claim 16 (per entry: the plane's creation draw, then its arrival draw, then its passengers' creation draws)
    let mut w = game(&sided(false));
    let before = w.rng.taken();
    let mut probe = w.clone();
    planes::paradrop(&mut w, ALPHA, at(0, 0), false);
    // The same stream consumed in the stated order gives the same state.
    for _ in 0..2 {
        let _ = probe.rng.raw(); // plane creation
        let _ = air::house_edge_cell(&mut probe, ALPHA); // arrival cell
        let _ = probe.rng.raw(); // one paratrooper
    }
    assert_eq!(w.rng.taken(), probe.rng.taken());
    assert_eq!(w.rng, probe.rng);
    assert!(w.rng.taken() > before);
}

// --- S-0522 claim 1 --------------------------------------------------------------------

#[test]
fn s0522_claim1_water_target_is_moved_to_land() {
    // spec: S-0522 claim 1 (a water-role target is moved once by the nearby-cell search to a cell that is not a water-role tile)
    let mut w = game(&ruleset());
    let mut cat = (*testkit::catalogue()).clone();
    let first = cat.tiles.len() as i32;
    let mut water = cat.tiles[0].clone();
    if let Some(Some(c)) = water.cells.first_mut() {
        c.terrain_code = 9; // the catalogue's water terrain code (as in claims_s0614)
    }
    for _ in 0..14 {
        cat.tiles.push(water.clone());
        cat.allow_tiberium.push(false);
    }
    cat.roles.water = first;
    w.catalogue = Some(std::sync::Arc::new(cat));
    let target = at(0, 0);
    let mut cells = Vec::new();
    for dx in -1..=1 {
        for dy in -1..=1 {
            let c = at(dx, dy);
            w.cells.grid.get_mut(c).unwrap().tile = first + 2;
            cells.push(c);
        }
    }
    glasnost_sim::load::recalc_cells(&mut w, &cells);
    assert_eq!(
        w.cells.cell(target).unwrap().land,
        glasnost_map::terrain::land::WATER
    );
    let moved = planes::paradrop_target(&mut w, target);
    assert_ne!(moved, target);
    assert!(!cells.contains(&moved));
    let p = planes::paradrop(&mut w, BETA, target, false);
    assert_eq!(w.obj(p[0]).unwrap().target, Some(Target::Cell(moved)));
    // A land target is kept.
    assert_eq!(planes::paradrop_target(&mut w, at(4, 4)), at(4, 4));
}

// --- Claim 43 ------------------------------------------------------------------------

#[test]
fn claim43_spy_plane_count_from_the_first_paradrop_list() {
    // spec: S-0513 claim 43 (the count is the first side group's paradrop infantry list length, for every side; visiting, spy-plane approach, no passengers)
    let mut w = game(&ruleset());
    let s = planes::spy_plane(&mut w, BETA, at(1, 1));
    assert_eq!(s.len(), 2);
    for &p in &s {
        let o = w.obj(p).unwrap();
        assert_eq!(type_name(&w, p), "SPYP");
        assert_eq!(o.beh.current, Some(Behaviour::SpyPlaneApproach));
        assert_eq!(o.target, Some(Target::Cell(at(1, 1))));
        assert_eq!(o.dest, None);
        assert!(o.air.as_ref().unwrap().visiting);
        assert_eq!(transport::count(&w, p), 0);
    }
}

#[test]
fn claim43_mismatched_list_sends_none_and_quirk_off_sends_one() {
    // spec: S-0513 claim 43 (none when the list and its count list differ in length; quirks.spy_plane_count_from_paradrop_list off: exactly one)
    let mut r = ruleset();
    r.sim.general.superweapons.ally_para_drop_num = Some(vec![1]);
    let mut w = game(&r);
    assert!(planes::spy_plane(&mut w, ALPHA, at(0, 0)).is_empty());
    r.globals.quirks.spy_plane_count_from_paradrop_list = false;
    let mut w = game(&r);
    assert_eq!(planes::spy_plane(&mut w, ALPHA, at(0, 0)).len(), 1);
}

// --- Claims 44 and 45 ------------------------------------------------------------------

fn sight_len(w: &World, id: ObjectId) -> Option<usize> {
    w.obj(id)?.econ.sight.sight.as_ref().map(|a| a.cells.len())
}

#[test]
fn claim44_camera_sight_radius_and_sound_in_approach() {
    // spec: S-0513 claim 44 (a camera with the primary weapon's damage as radius; the camera sound in approach only)
    let mut w = game(&ruleset());
    let target = at(0, 0);
    let s = planes::spy_plane(&mut w, BETA, target)[0];
    w.effects.clear();
    let cams = |w: &World| events(w).iter().filter(|e| e.0 == SwEvent::Camera).count();
    until(&mut w, 600, |w| cams(w) > 0);
    // The camera's radius is the weapon damage 4, wider than the plane's sight 2.
    let camera = glasnost_sim::econ::shroud::shape(4).len();
    let ordinary = glasnost_sim::econ::shroud::shape(2).len();
    assert!(sight_len(&w, s).is_some_and(|n| n > ordinary && n <= camera));
    let n = cams(&w);
    until(&mut w, 600, |w| {
        w.obj(s)
            .is_none_or(|o| o.beh.current == Some(Behaviour::SpyPlaneOverfly))
    });
    run(&mut w, 30);
    // spec: S-0513 claim 44 (in overfly there is no camera sound)
    assert_eq!(cams(&w), n);
    // spec: S-0513 claim 45 (no detection is added; explored cells stay explored)
    assert!(glasnost_sim::econ::shroud::explored(&w, BETA, target));
}

#[test]
fn claim44_no_primary_weapon_gives_no_camera_and_warns() {
    // spec: S-0513 claim 44 (a spy type with no primary weapon gives no camera; our engine warns)
    let mut r = ruleset();
    r.sim.aircraft.get_mut("SPYP").unwrap().primary = None;
    // The weaponless plane answers *impossible* (S-0301 claim 9) and drops
    // its target at its first check tick (S-0309 claim 29 r94), so the
    // camera processing that warns must come first: a short camera frame
    // count, and a launch just after a check tick.
    r.sim.general.economy.spy_plane_camera_frames = Some(4);
    let mut w = game(&r);
    while w.tick % 16 != 1 {
        run(&mut w, 1);
    }
    planes::spy_plane(&mut w, BETA, at(0, 0));
    w.effects.clear();
    run(&mut w, 300);
    assert!(!events(&w).iter().any(|e| e.0 == SwEvent::Camera));
    assert!(w.notes.iter().any(|n| n.contains("claim 44")));
}

// spec: S-0501 claim 10 (a visiting aircraft in spy-plane overfly that has entered the usable area since it appeared may leave the map now: removed, without dying, when its position leaves the usable area) and claim 34 (the overfly flies to its exit edge); the entry its own forward step marks is kept by that movement update
#[test]
fn s0501_claim10_spy_plane_entering_from_outside_is_removed_after_its_overfly() {
    // The exit edge's cells lie outside the usable area (S-0522 claim 4).
    let mut w = banded_game(&ruleset());
    let s = planes::spy_plane(&mut w, BETA, at(0, 0))[0];
    let first = w.obj(s).unwrap().pos.cell();
    assert!(!w.cells.usable(first), "appears outside the usable area");
    let mut inside = false;
    let mut overfly = false;
    for _ in 0..3000 {
        let Some(o) = w.obj(s) else { break };
        assert!(o.health > 0, "alive");
        inside |= w.cells.usable(o.pos.cell());
        overfly |= o.beh.current == Some(Behaviour::SpyPlaneOverfly);
        if inside {
            assert!(o.air.as_ref().unwrap().entered, "the entry is kept");
        }
        step(&mut w);
    }
    assert!(inside && overfly, "flew in and over its target");
    assert!(w.obj(s).is_none(), "removed once it has left");
}
