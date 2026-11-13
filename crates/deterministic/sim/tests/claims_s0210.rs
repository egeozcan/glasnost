//! Claim tests for crushing (S-0210): who crushes and who is crushed, the
//! vehicle thief, the warning ahead, overlays and walls, the crush-slowdown
//! state and the imported crush rules, on synthetic data.

mod common;

use common::{FOOT, Scene, TRACK, cell, open_orders, order_move, run, unit_type};
use glasnost_map::{CellXY, NO_OVERLAY};
use glasnost_num::F64;
use glasnost_rules::sim::{BuildingType, Decimal, MovementZone, OverlayType, UnitType};
use glasnost_sim::consts::CRUSH_CAP;
use glasnost_sim::crush;
use glasnost_sim::econ::victory;
use glasnost_sim::infantry::Seq;
use glasnost_sim::occupancy;
use glasnost_sim::orders::{self, Order};
use glasnost_sim::rules::{Kind, SimRules};
use glasnost_sim::world::{HouseId, Layer, ObjectId, Target, World};

const ALPHA: u8 = 0;
const BETA: u8 = 1;

/// Overlay indices in the "overlays" registry of [`scene`].
const BRUSH: u8 = 0;
const FENCE: u8 = 1;
const PALING: u8 = 2;
const HEDGE: u8 = 3;
const LOWWALL: u8 = 4;

fn vehicle(f: impl FnOnce(&mut UnitType)) -> UnitType {
    let mut t = unit_type(TRACK, 6, 8);
    f(&mut t);
    t
}

fn foot(f: impl FnOnce(&mut UnitType)) -> UnitType {
    let mut t = unit_type(FOOT, 4, 8);
    f(&mut t);
    t
}

fn overlay(f: impl FnOnce(&mut OverlayType)) -> OverlayType {
    let mut o = OverlayType {
        has_art: Some(true),
        ..Default::default()
    };
    f(&mut o);
    o
}

/// A scene with invented vehicle, infantry, building and overlay types.
fn scene() -> Scene {
    let mut s = Scene::new();
    s.vehicle("MASHER", vehicle(|t| t.crusher = Some(true)));
    s.vehicle(
        "MASHALL",
        vehicle(|t| {
            t.crusher = Some(true);
            t.movement_zone = Some(MovementZone::CrusherAll);
        }),
    );
    s.vehicle("OMNI", vehicle(|t| t.omni_crusher = Some(true)));
    s.vehicle("PLAIN", vehicle(|_| {}));
    s.vehicle("PEBBLE", vehicle(|t| t.crushable = Some(true)));
    s.vehicle("TOUGH", vehicle(|t| t.omni_crush_resistant = Some(true)));
    s.vehicle(
        "ROLLER",
        vehicle(|t| t.movement_zone = Some(MovementZone::CrusherAll)),
    );
    s.vehicle(
        "STEADY",
        vehicle(|t| {
            t.movement_zone = Some(MovementZone::CrusherAll);
            t.accelerates = Some(false);
        }),
    );
    s.vehicle(
        "LOCO",
        vehicle(|t| {
            t.crusher = Some(true);
            t.is_train = Some(true);
        }),
    );
    s.infantry("GRUNT", foot(|t| t.crushable = Some(true)));
    s.infantry(
        "DIGGER",
        foot(|t| {
            t.crushable = Some(true);
            t.deployed_crushable = Some(true);
        }),
    );
    s.infantry(
        "SITTER",
        foot(|t| {
            t.crushable = Some(true);
            t.deployed_crushable = Some(false);
        }),
    );
    s.infantry(
        "SNATCHER",
        foot(|t| {
            t.crushable = Some(true);
            t.vehicle_thief = Some(true);
        }),
    );
    s.ruleset
        .registries
        .insert("buildings".into(), vec!["HUT".into(), "SHED".into()]);
    s.ruleset.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            ..Default::default()
        },
    );
    s.ruleset.sim.buildings.insert(
        "SHED".into(),
        BuildingType {
            strength: Some(400),
            foundation: Some([1, 1]),
            crushable: Some(true),
            ..Default::default()
        },
    );
    s.ruleset.registries.insert(
        "overlays".into(),
        ["BRUSH", "FENCE", "PALING", "HEDGE", "LOWWALL"]
            .iter()
            .map(|n| n.to_string())
            .collect(),
    );
    let ov = &mut s.ruleset.sim.overlays;
    ov.insert("BRUSH".into(), overlay(|o| o.crushable = Some(true)));
    // A wall type with four damage stages and a strength above 200.
    ov.insert(
        "FENCE".into(),
        overlay(|o| {
            o.wall = Some(true);
            o.damage_levels = Some(4);
            o.strength = Some(300);
        }),
    );
    // A wall type with four damage stages and a strength of at most 200.
    ov.insert(
        "PALING".into(),
        overlay(|o| {
            o.wall = Some(true);
            o.damage_levels = Some(4);
            o.strength = Some(150);
        }),
    );
    // A wall type with two damage stages.
    ov.insert(
        "HEDGE".into(),
        overlay(|o| {
            o.wall = Some(true);
            o.damage_levels = Some(2);
            o.strength = Some(300);
        }),
    );
    // A second wall type, like FENCE.
    ov.insert(
        "LOWWALL".into(),
        overlay(|o| {
            o.wall = Some(true);
            o.damage_levels = Some(4);
            o.strength = Some(300);
        }),
    );
    s
}

/// The objects of a type (by name), in ID order.
fn of_type(w: &World, kind: Kind, name: &str) -> Vec<ObjectId> {
    let r = w.rules_arc();
    w.objects
        .values()
        .filter(|o| o.kind == kind)
        .filter(|o| {
            let n = match kind {
                Kind::Building => r.buildings.get(o.type_index as usize).map(|b| &b.name),
                _ => r.unit(o.kind, o.type_index).map(|u| &u.name),
            };
            n.is_some_and(|n| n == name)
        })
        .map(|o| o.id)
        .collect()
}

fn one(w: &World, kind: Kind, name: &str) -> ObjectId {
    let v = of_type(w, kind, name);
    assert_eq!(v.len(), 1, "exactly one {name}");
    v[0]
}

fn set_overlay(w: &mut World, c: CellXY, overlay: u8, data: u8) {
    let cell = w.cells.grid.get_mut(c).expect("cell on the map");
    cell.overlay = overlay;
    cell.overlay_data = data;
}

fn overlay_at(w: &World, c: CellXY) -> u8 {
    w.cells.cell(c).map_or(NO_OVERLAY, |c| c.overlay)
}

// spec: S-0210 claim 1 (the crusher flag makes a crusher; the omni-crusher flag alone does not)
#[test]
fn only_the_crusher_flag_makes_a_crusher() {
    let mut s = scene();
    s.place_unit(ALPHA, "MASHER", cell(0, 0), 64);
    s.place_unit(ALPHA, "OMNI", cell(2, 0), 64);
    s.place_unit(ALPHA, "PLAIN", cell(4, 0), 64);
    let w = s.build();
    assert!(occupancy::is_crusher(&w, one(&w, Kind::Vehicle, "MASHER")));
    assert!(!occupancy::is_crusher(&w, one(&w, Kind::Vehicle, "OMNI")));
    assert!(!occupancy::is_crusher(&w, one(&w, Kind::Vehicle, "PLAIN")));
}

// spec: S-0210 claim 1 (an omni-crusher without the crusher flag crushes nothing at a cell centre)
#[test]
fn an_omni_crusher_alone_crushes_nothing_at_a_cell_centre() {
    for (crusher, survives) in [("OMNI", true), ("MASHER", false)] {
        let mut s = scene();
        s.place_unit(ALPHA, crusher, cell(0, 0), 64);
        s.place_infantry(BETA, "GRUNT", cell(0, 0), 2);
        let mut w = s.build();
        let c = one(&w, Kind::Vehicle, crusher);
        let g = one(&w, Kind::Infantry, "GRUNT");
        assert!(w.cells.list(cell(0, 0), Layer::Ground).contains(&g));
        crush::at_cell_centre(&mut w, c);
        assert_eq!(w.obj(g).is_some(), survives, "crusher {crusher}");
    }
}

// spec: S-0210 claim 2 (the crushable flag; own and allied objects are never crushed)
#[test]
fn crushable_flag_and_alliance() {
    for allied in [false, true] {
        let mut s = scene();
        if allied {
            s.setup.alliances = vec![(0, 1)];
        }
        s.place_unit(ALPHA, "MASHER", cell(0, 0), 64);
        s.place_unit(BETA, "PEBBLE", cell(2, 0), 64);
        s.place_unit(BETA, "PLAIN", cell(4, 0), 64);
        s.place_unit(ALPHA, "PEBBLE", cell(6, 0), 64);
        let w = s.build();
        let m = one(&w, Kind::Vehicle, "MASHER");
        let pebbles = of_type(&w, Kind::Vehicle, "PEBBLE");
        let (enemy_pebble, own_pebble) = if w.obj(pebbles[0]).unwrap().owner == Some(HouseId(1)) {
            (pebbles[0], pebbles[1])
        } else {
            (pebbles[1], pebbles[0])
        };
        assert_eq!(occupancy::crushable_by(&w, m, enemy_pebble), !allied);
        assert!(!occupancy::crushable_by(&w, m, own_pebble));
        assert!(!occupancy::crushable_by(
            &w,
            m,
            one(&w, Kind::Vehicle, "PLAIN")
        ));
    }
}

// spec: S-0210 claim 2 (the omni-crusher route: non-buildings that are not resistant)
#[test]
fn the_omni_crusher_route() {
    let mut s = scene();
    s.place_unit(ALPHA, "OMNI", cell(0, 0), 64);
    s.place_unit(ALPHA, "MASHER", cell(0, 4), 64);
    s.place_unit(BETA, "PLAIN", cell(2, 0), 64);
    s.place_unit(BETA, "TOUGH", cell(4, 0), 64);
    s.place_structure(BETA, "HUT", cell(6, 0));
    s.place_structure(BETA, "SHED", cell(8, 0));
    let w = s.build();
    let omni = one(&w, Kind::Vehicle, "OMNI");
    let masher = one(&w, Kind::Vehicle, "MASHER");
    let plain = one(&w, Kind::Vehicle, "PLAIN");
    let tough = one(&w, Kind::Vehicle, "TOUGH");
    let hut = one(&w, Kind::Building, "HUT");
    let shed = one(&w, Kind::Building, "SHED");
    assert!(occupancy::crushable_by(&w, omni, plain));
    assert!(!occupancy::crushable_by(&w, omni, tough));
    // Only the crushable flag makes a building crushable.
    assert!(!occupancy::crushable_by(&w, omni, hut));
    assert!(occupancy::crushable_by(&w, omni, shed));
    assert!(occupancy::crushable_by(&w, masher, shed));
    assert!(!occupancy::crushable_by(&w, masher, hut));
    assert!(!occupancy::crushable_by(&w, masher, plain));
}

// spec: S-0210 claim 2 (a deployed infantry unit is crushable only if its type stays crushable when deployed)
#[test]
fn deployed_infantry_and_the_deployed_crushable_flag() {
    let mut s = scene();
    s.place_unit(ALPHA, "MASHER", cell(0, 0), 64);
    s.place_infantry(BETA, "SITTER", cell(2, 0), 1);
    s.place_infantry(BETA, "DIGGER", cell(4, 0), 1);
    let mut w = s.build();
    let m = one(&w, Kind::Vehicle, "MASHER");
    let g = one(&w, Kind::Infantry, "SITTER");
    let d = one(&w, Kind::Infantry, "DIGGER");
    assert!(occupancy::crushable_by(&w, m, g));
    assert!(occupancy::crushable_by(&w, m, d));
    for id in [g, d] {
        w.obj_mut(id).unwrap().infantry.as_mut().unwrap().seq = Some(Seq::Deployed);
    }
    assert!(!occupancy::crushable_by(&w, m, g));
    assert!(occupancy::crushable_by(&w, m, d));
}

/// A crusher type at cell (0, 0) with a Beta thief coming for it and a
/// crushable Beta infantry unit placed before or after the thief.
fn thief_world(crusher: &str, grunt_first: bool) -> (World, ObjectId, ObjectId, ObjectId) {
    let mut s = scene();
    s.place_unit(ALPHA, crusher, cell(0, 0), 64);
    if grunt_first {
        s.place_infantry(BETA, "GRUNT", cell(0, 0), 2);
    }
    s.place_infantry(BETA, "SNATCHER", cell(0, 0), 3);
    if !grunt_first {
        s.place_infantry(BETA, "GRUNT", cell(0, 0), 2);
    }
    let mut w = s.build();
    let c = one(&w, Kind::Vehicle, crusher);
    let t = one(&w, Kind::Infantry, "SNATCHER");
    let g = one(&w, Kind::Infantry, "GRUNT");
    w.obj_mut(t).unwrap().dest = Some(Target::Object(c));
    (w, c, t, g)
}

fn listed_before(w: &World, a: ObjectId, b: ObjectId) -> bool {
    let l = w.cells.list(cell(0, 0), Layer::Ground);
    let pa = l.iter().position(|&x| x == a).expect("listed");
    let pb = l.iter().position(|&x| x == b).expect("listed");
    pa < pb
}

// spec: S-0210 claim 5 (a thief coming for the crusher takes it over instead of being crushed)
#[test]
fn a_vehicle_thief_takes_the_crusher_over() {
    let (mut w, c, t, _) = thief_world("MASHER", true);
    crush::at_cell_centre(&mut w, c);
    assert!(w.obj(t).is_none(), "the thief disappears");
    assert_eq!(w.obj(c).unwrap().owner, Some(HouseId(1)));
}

// spec: S-0210 claim 5 (the crusher changes owner to the thief's house), S-0414 claim 4 (a change of owner counts as leaving the map for the old owner and entering it for the new one, whatever its cause) and S-0309 claim 30 (a holder that changes owner drops its target)
#[test]
fn a_take_over_is_a_full_change_of_owner() {
    let (a, b) = (HouseId(0), HouseId(1));
    // The placing order that lists the grunt after the thief.
    let grunt_first = {
        let (w, _, t, g) = thief_world("MASHER", true);
        listed_before(&w, t, g)
    };
    let (mut w, c, t, g) = thief_world("MASHER", grunt_first);
    assert!(listed_before(&w, t, g));
    w.obj_mut(c).unwrap().target = Some(Target::Object(g));
    assert_eq!(
        (victory::normal_sum(&w, a), victory::normal_sum(&w, b)),
        (1, 2)
    );
    crush::at_cell_centre(&mut w, c);
    assert!(w.obj(t).is_none(), "the thief disappears");
    assert!(
        w.obj(g).is_some(),
        "listed after the thief: judged against the new house"
    );
    assert_eq!(w.obj(c).unwrap().owner, Some(b));
    assert_eq!(
        (victory::normal_sum(&w, a), victory::normal_sum(&w, b)),
        (0, 2),
        "the vehicle counts for the thief's house, not for its old one"
    );
    assert_eq!(w.obj(c).unwrap().target, None, "the target is dropped");
    glasnost_sim::objects::remove(&mut w, c);
    assert!(!victory::passes(&w, a), "the old owner has nothing left");
    assert!(
        victory::passes(&w, b),
        "the thief's house still has its infantry"
    );
}

// spec: S-0210 claim 4 (the kill is credited to the crusher, as for any kill) and S-0414 claim 21 (the result screen's kill to the credited object's house, a loss to the victim's)
#[test]
fn a_crush_credits_the_kill_to_the_crusher() {
    let mut s = scene();
    s.place_unit(ALPHA, "MASHER", cell(0, 0), 64);
    s.place_infantry(BETA, "GRUNT", cell(0, 0), 2);
    let mut w = s.build();
    let m = one(&w, Kind::Vehicle, "MASHER");
    let g = one(&w, Kind::Infantry, "GRUNT");
    let counts = |w: &World| {
        (
            w.houses[0].combat.kills,
            glasnost_sim::econ::stats::of(w, HouseId(0)).kills,
            w.houses[1].combat.losses,
            glasnost_sim::econ::stats::of(w, HouseId(1)).losses,
        )
    };
    assert_eq!(counts(&w), (0, 0, 0, 0));
    crush::at_cell_centre(&mut w, m);
    assert!(w.obj(g).is_none(), "the grunt is crushed");
    assert_eq!(
        counts(&w),
        (1, 1, 1, 1),
        "a kill for the crusher's house, a loss for the victim's"
    );
}

// spec: S-0210 claim 5 (a thief whose destination is not the crusher is crushed as usual)
#[test]
fn a_thief_not_coming_for_the_crusher_is_crushed() {
    let (mut w, c, t, _) = thief_world("MASHER", true);
    w.obj_mut(t).unwrap().dest = None;
    crush::at_cell_centre(&mut w, c);
    assert!(w.obj(t).is_none());
    assert_eq!(w.obj(c).unwrap().owner, Some(HouseId(0)));
}

// spec: S-0210 claim 5 (a train is not taken over: the thief is crushed)
#[test]
fn a_train_crushes_the_thief() {
    let (mut w, c, t, g) = thief_world("LOCO", true);
    crush::at_cell_centre(&mut w, c);
    assert!(w.obj(t).is_none());
    assert!(w.obj(g).is_none());
    assert_eq!(w.obj(c).unwrap().owner, Some(HouseId(0)));
}

// spec: S-0210 claim 5 (objects listed after the thief are judged against the new house, those before against the old)
#[test]
fn the_new_house_applies_to_objects_listed_after_the_thief() {
    let mut orders_seen = Vec::new();
    for grunt_first in [true, false] {
        let (mut w, c, t, g) = thief_world("MASHER", grunt_first);
        let grunt_after_thief = listed_before(&w, t, g);
        orders_seen.push(grunt_after_thief);
        crush::at_cell_centre(&mut w, c);
        assert_eq!(w.obj(c).unwrap().owner, Some(HouseId(1)));
        // Listed after the thief: judged against Beta (own house), survives.
        assert_eq!(
            w.obj(g).is_some(),
            grunt_after_thief,
            "grunt placed first: {grunt_first}"
        );
    }
    assert_ne!(orders_seen[0], orders_seen[1], "both listing orders tested");
}

fn scatter_scene() -> Scene {
    let mut s = scene();
    s.ruleset.sim.general.player_scatter = Some(true);
    s
}

// spec: S-0210 claim 7 (a cell-wide scatter request to the first cell's layer when it holds infantry)
#[test]
fn the_warning_scatters_infantry_in_the_first_cell() {
    let mut s = scatter_scene();
    s.place_unit(ALPHA, "MASHER", cell(0, 0), 64);
    s.place_infantry(BETA, "GRUNT", cell(1, 0), 1);
    let mut w = s.build();
    let m = one(&w, Kind::Vehicle, "MASHER");
    let g = one(&w, Kind::Infantry, "GRUNT");
    open_orders(&mut w);
    assert!(!occupancy::in_progress(&w, g));
    let rng = w.rng.clone();
    // An empty cell: no request, no draw.
    crush::warn_ahead(&mut w, m, cell(0, 1));
    assert_eq!(w.rng, rng);
    crush::warn_ahead(&mut w, m, cell(1, 0));
    assert_ne!(w.rng, rng, "the scatter request takes its draw");
    run(&mut w, 60);
    assert_ne!(
        w.obj(g).unwrap().pos.cell(),
        cell(1, 0),
        "the infantry scattered"
    );
}

/// A crusher drives east from (0, 0) to (4, 0) past a stationary Beta
/// infantry unit at (2, 0), optionally standing on a crushable overlay
/// (which puts the track into that cell in crush mode). Returns whether the
/// infantry accepted a scatter request before the crusher reached it, and whether it is alive at that point.
fn drive_at_infantry(brush: bool) -> (bool, bool) {
    let mut s = scatter_scene();
    if brush {
        s.cell_mut(cell(2, 0)).overlay = BRUSH;
    }
    s.place_unit(ALPHA, "MASHER", cell(0, 0), 64);
    s.place_infantry(BETA, "GRUNT", cell(2, 0), 1);
    let mut w = s.build();
    let m = one(&w, Kind::Vehicle, "MASHER");
    let g = one(&w, Kind::Infantry, "GRUNT");
    open_orders(&mut w);
    order_move(&mut w, m, cell(4, 0));
    // Acceptance is read from the trace: standing on the brush, the walker's
    // new destination fails the reachability gate in its own turn of the
    // same tick and is cleared at once (S-0417 claim 17, No path; S-0212
    // claim 23), so the destination is not there to read.
    glasnost_sim::trace::switch_on(&mut w);
    for _ in 0..300 {
        run(&mut w, 1);
        let accepted = glasnost_sim::trace::take(&w).iter().any(|e| {
            e.unit == g
                && matches!(
                    e.what,
                    glasnost_sim::trace::What::Asked { accepted: true, .. }
                )
        });
        if w.obj(g).is_none() {
            return (accepted, false);
        }
        if accepted {
            return (true, true);
        }
    }
    (false, true)
}

// spec: S-0210 claim 7 (on a crush-mode track the infantry in the first cell is warned)
#[test]
fn a_crush_mode_track_warns_the_infantry_ahead() {
    assert_eq!(drive_at_infantry(true), (true, true));
}

// spec: S-0210 claim 7 (the warning is sent at the start of every track, crush mode or not)
#[test]
fn the_warning_needs_no_crush_mode() {
    let (warned, alive) = drive_at_infantry(false);
    assert!(
        warned,
        "the infantry in the path was never sent a scatter request (alive: {alive})"
    );
}

/// A world with a vehicle of type `ty` standing at (0, 0) on the given
/// overlay, with optional neighbours (dx, dy, overlay, data).
fn overlay_world(
    ty: &str,
    ov: u8,
    data: u8,
    neighbours: &[(i32, i32, u8, u8)],
) -> (World, ObjectId) {
    let mut s = scene();
    s.place_unit(ALPHA, ty, cell(0, 0), 64);
    let mut w = s.build();
    let id = one(&w, Kind::Vehicle, ty);
    set_overlay(&mut w, cell(0, 0), ov, data);
    for &(dx, dy, o, d) in neighbours {
        set_overlay(&mut w, cell(dx, dy), o, d);
    }
    (w, id)
}

// spec: S-0210 claim 8 (a crushable overlay that is not a wall stays, and the slowdown state ends)
#[test]
fn a_crushable_overlay_stays_and_ends_the_slowdown() {
    let (mut w, id) = overlay_world("MASHER", BRUSH, 0, &[]);
    w.obj_mut(id).unwrap().crush_slowdown = true;
    crush::at_cell_centre(&mut w, id);
    assert_eq!(overlay_at(&w, cell(0, 0)), BRUSH);
    assert!(!w.obj(id).unwrap().crush_slowdown);
}

// spec: S-0210 claim 4 (the crushed type's crush sound plays at the crusher's position)
// spec: S-0210 claim 8 (every crushed overlay plays its crush sound at the crusher)
#[test]
fn crushing_plays_the_crush_sounds() {
    use glasnost_sim::combat::effects::{Effect, Sound};
    let (mut w, id) = overlay_world("MASHER", BRUSH, 0, &[]);
    let mut s = scene();
    s.place_unit(ALPHA, "MASHER", cell(0, 0), 64);
    s.place_infantry(BETA, "GRUNT", cell(0, 0), 2);
    let mut w2 = s.build();
    let m = one(&w2, Kind::Vehicle, "MASHER");
    let pos = w.obj(id).unwrap().pos;
    w.effects.clear();
    crush::at_cell_centre(&mut w, id);
    assert!(
        w.effects
            .iter()
            .any(|e| matches!(e, Effect::OverlayCrushed { overlay, pos: p } if *overlay == BRUSH && *p == pos))
    );
    let grunt = w2.rules().index_of(Kind::Infantry, "GRUNT").unwrap();
    let mpos = w2.obj(m).unwrap().pos;
    w2.effects.clear();
    crush::at_cell_centre(&mut w2, m);
    assert!(w2.effects.iter().any(|e| matches!(
        e,
        Effect::Sound { sound: Sound::Crush, of: Some(t), pos: Some(p), .. }
            if t.kind == Kind::Infantry && t.index == grunt && *p == mpos
    )));
}

// spec: S-0210 claim 8 (a wall is crushed only by a crusher of movement zone crusher-all, and is then removed)
#[test]
fn a_wall_is_crushed_only_by_a_crusher_all_crusher() {
    let (mut w, id) = overlay_world("MASHER", FENCE, 0, &[]);
    w.obj_mut(id).unwrap().crush_slowdown = true;
    crush::at_cell_centre(&mut w, id);
    assert_eq!(overlay_at(&w, cell(0, 0)), FENCE);
    assert!(w.obj(id).unwrap().crush_slowdown, "nothing was crushed");

    let (mut w, id) = overlay_world("MASHALL", FENCE, 0, &[]);
    w.obj_mut(id).unwrap().crush_slowdown = true;
    crush::at_cell_centre(&mut w, id);
    assert_eq!(overlay_at(&w, cell(0, 0)), NO_OVERLAY);
    assert_eq!(w.cells.cell(cell(0, 0)).unwrap().overlay_data, 0);
    assert!(!w.obj(id).unwrap().crush_slowdown);
}

// spec: S-0210 claim 8 and S-0516 claim 11 (c) (crushing only an overlay or wall does not uncloak the crusher)
#[test]
fn crushing_only_a_wall_keeps_the_crusher_cloaked() {
    use glasnost_sim::cloak::{self, CloakState};
    let (mut w, id) = overlay_world("MASHALL", FENCE, 0, &[]);
    {
        let o = w.obj_mut(id).unwrap();
        o.cloak.state = CloakState::Cloaked;
        o.cloak.stage = 0;
    }
    crush::at_cell_centre(&mut w, id);
    assert_eq!(
        overlay_at(&w, cell(0, 0)),
        NO_OVERLAY,
        "the wall was crushed"
    );
    assert_eq!(cloak::state(&w, id), CloakState::Cloaked);
}

fn stage_at(w: &World, c: CellXY) -> u8 {
    w.cells.cell(c).unwrap().overlay_data / 16
}

// spec: S-0210 claim 8 (stage D − 2 hits undamaged neighbours of the same type: one draw each when S > 200)
// spec: S-0210 F3 (north, east, south, west; success when the draw is below 200); S-0303 F4 (a successful hit raises the stage by one)
#[test]
fn a_wall_at_stage_d_minus_two_hits_its_undamaged_neighbours() {
    // Stage 2 = D − 2 (data 32 … 47). North and east: undamaged FENCE;
    // south: FENCE at stage 1; west: another wall type, undamaged.
    let (mut w, id) = overlay_world(
        "MASHALL",
        FENCE,
        37,
        &[
            (0, -1, FENCE, 5),
            (1, 0, FENCE, 0),
            (0, 1, FENCE, 16),
            (-1, 0, LOWWALL, 0),
        ],
    );
    let mut expect = w.rng.clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    let draws = [expect.ranged(0, 300, half), expect.ranged(0, 300, half)];
    crush::at_cell_centre(&mut w, id);
    assert_eq!(overlay_at(&w, cell(0, 0)), NO_OVERLAY);
    assert_eq!(w.rng, expect, "exactly two ranged draws over 0 … 300");
    // North took the first draw, east the second: each rises to stage 1 on success.
    assert_eq!(stage_at(&w, cell(0, -1)), u8::from(draws[0] < 200));
    assert_eq!(stage_at(&w, cell(1, 0)), u8::from(draws[1] < 200));
    // South (damaged) and west (another type) are not hit.
    assert_eq!(stage_at(&w, cell(0, 1)), 1);
    assert_eq!(stage_at(&w, cell(-1, 0)), 0);
    assert_eq!(overlay_at(&w, cell(0, -1)), FENCE);
    assert_eq!(overlay_at(&w, cell(-1, 0)), LOWWALL);
}

// spec: S-0210 claim 8 (with a wall strength of at most 200 the hit always succeeds and takes no draw)
// spec: S-0303 F4 (the successful hit raises the neighbour to stage 1; it stays, as 1 is not L − 1)
#[test]
fn a_weak_wall_hit_always_succeeds_without_a_draw() {
    let (mut w, id) = overlay_world("MASHALL", PALING, 32, &[(0, -1, PALING, 0)]);
    let rng = w.rng.clone();
    crush::at_cell_centre(&mut w, id);
    assert_eq!(overlay_at(&w, cell(0, 0)), NO_OVERLAY);
    assert_eq!(w.rng, rng);
    assert_eq!(overlay_at(&w, cell(0, -1)), PALING);
    assert_eq!(stage_at(&w, cell(0, -1)), 1);
}

// spec: S-0210 claim 8 (another stage, or a type with at most two stages: removed, no hit)
#[test]
fn other_stages_remove_the_wall_without_hits() {
    for (ov, data) in [
        (FENCE, 0),
        (FENCE, 16),
        (FENCE, 48),
        (PALING, 16),
        (HEDGE, 0),
    ] {
        let (mut w, id) = overlay_world(
            "MASHALL",
            ov,
            data,
            &[(0, -1, ov, 0), (1, 0, ov, 0), (0, 1, ov, 0), (-1, 0, ov, 0)],
        );
        let rng = w.rng.clone();
        crush::at_cell_centre(&mut w, id);
        assert_eq!(
            overlay_at(&w, cell(0, 0)),
            NO_OVERLAY,
            "overlay {ov} data {data}"
        );
        assert_eq!(w.rng, rng, "overlay {ov} data {data}");
        for n in [cell(0, -1), cell(1, 0), cell(0, 1), cell(-1, 0)] {
            assert_eq!(stage_at(&w, n), 0, "overlay {ov} data {data}: no hit");
        }
    }
}

/// One tick of [`drive_past_brush`]: slowdown state, ρ, ρₜ, cell, on a track.
type Entry = (bool, F64, F64, CellXY, bool);

/// Drives a vehicle of type `ty` east from (0, 0) to (8, 0) past a
/// crushable overlay at (4, 0); returns the world, the vehicle and, per
/// tick, (slowdown state, ρ, ρₜ, cell, on a track).
fn drive_past_brush(ty: &str) -> (World, ObjectId, Vec<Entry>) {
    let mut s = scene();
    s.cell_mut(cell(4, 0)).overlay = BRUSH;
    s.place_unit(ALPHA, ty, cell(0, 0), 64);
    let mut w = s.build();
    assert_eq!(overlay_at(&w, cell(4, 0)), BRUSH);
    let id = one(&w, Kind::Vehicle, ty);
    open_orders(&mut w);
    order_move(&mut w, id, cell(8, 0));
    let mut log = Vec::new();
    for _ in 0..600 {
        run(&mut w, 1);
        let o = w.obj(id).unwrap();
        let d = o.drive.as_ref().unwrap();
        log.push((
            o.crush_slowdown,
            d.rho,
            d.rho_t,
            o.pos.cell(),
            d.track.is_some(),
        ));
        if o.pos.cell() == cell(8, 0) && d.track.is_none() && o.path.steps.is_empty() {
            break;
        }
    }
    assert_eq!(
        w.obj(id).unwrap().pos.cell(),
        cell(8, 0),
        "notes: {:?}",
        w.notes
    );
    (w, id, log)
}

// spec: S-0210 claim 9 (crusher-all on a crush-mode track enters the state; a non-crusher keeps it past the overlay)
#[test]
fn a_crusher_all_vehicle_enters_the_slowdown_and_keeps_it() {
    let (mut w, id, log) = drive_past_brush("ROLLER");
    let first = log.iter().position(|e| e.0).expect("the state was entered");
    // Entered in the cell before the overlay's, on the ground counting itself.
    // Crush mode starts with the track whose second cell holds the overlay.
    assert_eq!(log[first].3, cell(3, 0));
    assert!(
        log[first..].iter().all(|e| e.0),
        "the state never ends for a non-crusher"
    );
    // Neither a new order nor a stop ends it.
    order_move(&mut w, id, cell(8, 4));
    run(&mut w, 40);
    assert!(w.obj(id).unwrap().crush_slowdown);
    assert!(orders::issue(&mut w, HouseId(0), Order::Stop { actor: id }));
    run(&mut w, 20);
    assert!(w.obj(id).unwrap().crush_slowdown);
}

// spec: S-0210 claim 9 (while in the state the target and speed fractions both become min(target, 0.2))
#[test]
fn the_slowdown_caps_the_speed_fraction() {
    let (_, _, log) = drive_past_brush("ROLLER");
    let mut checked = 0;
    for pair in log.windows(2) {
        if pair[0].0 && pair[1].0 && pair[1].4 {
            let (_, rho, rho_t, _, _) = pair[1];
            assert!(rho.le(CRUSH_CAP), "ρ above the cap");
            assert_eq!(rho, rho_t, "ρ jumps to the lowered target");
            checked += 1;
        }
    }
    assert!(checked > 10);
}

// spec: S-0210 claim 9 (a vehicle of a type that is not accelerating is not slowed)
#[test]
fn a_non_accelerating_vehicle_is_not_slowed() {
    let (_, _, log) = drive_past_brush("STEADY");
    assert!(log.iter().any(|e| e.0), "the state was entered");
    assert!(
        log.iter().any(|e| e.0 && e.1.gt(CRUSH_CAP)),
        "never above the cap"
    );
}

// spec: S-0210 claim 9 (the state needs movement zone crusher-all)
#[test]
fn other_movement_zones_never_enter_the_slowdown() {
    let (_, _, log) = drive_past_brush("PLAIN");
    assert!(log.iter().all(|e| !e.0));
}

// spec: S-0210 claim 9 (a crusher ends the state when it crushes the overlay)
#[test]
fn a_crusher_ends_the_slowdown_at_the_overlay() {
    let (w, id, log) = drive_past_brush("MASHALL");
    assert!(log.iter().any(|e| e.0), "the state was entered");
    let last = log.iter().rposition(|e| e.0).unwrap();
    assert!(
        log[last].3.x <= cell(4, 0).x,
        "still slowed in {:?}",
        log[last].3
    );
    assert!(!w.obj(id).unwrap().crush_slowdown);
    // The crushable overlay that is not a wall stays.
    assert_eq!(overlay_at(&w, cell(4, 0)), BRUSH);
}

// spec: S-0210 claim 11 (crush range: absent 384 leptons, otherwise trunc(256 × c); player-auto-crush absent: off)
#[test]
fn the_imported_crush_rules() {
    let s = scene();
    let r = s.sim_rules();
    assert_eq!(r.general.crush_range, 384);
    assert!(!r.general.player_auto_crush);
    let mut s = scene();
    s.ruleset.sim.general.crush_range = Some(Decimal("2.5".into()));
    s.ruleset.sim.general.player_auto_crush = Some(true);
    let r = SimRules::new(&s.ruleset, "claims");
    assert_eq!(r.general.crush_range, 640);
    assert!(r.general.player_auto_crush);
}

// spec: S-0210 claim 11 (a crush range of exactly −1 counts as no value: the range in force stays)
#[test]
fn a_crush_range_of_minus_one_keeps_the_default() {
    let mut s = scene();
    s.ruleset.sim.general.crush_range = Some(Decimal("-1".into()));
    assert_eq!(s.sim_rules().general.crush_range, 384);
}

// ---------------------------------------------------------------------------
// Wall data and wall owners (claims 13 and 14)

// spec: S-0210 claim 13 (absent from every layer: damage-stage count 1 and wall strength 1)
// spec: S-0210 Constants (damage-stage count and wall strength absent-key defaults)
#[test]
fn wall_defaults_are_one_stage_and_strength_one() {
    let mut s = scene();
    s.register("overlays", "BARE");
    s.ruleset
        .sim
        .overlays
        .insert("BARE".into(), overlay(|o| o.wall = Some(true)));
    let r = s.sim_rules();
    let bare = r.overlays.get(5).expect("BARE");
    assert_eq!(bare.damage_levels, 1);
    assert_eq!(bare.strength, 1);
    // D = 1: only stage 0, so crushing it hits no neighbour and takes no draw.
    s.place_unit(ALPHA, "MASHALL", cell(0, 0), 64);
    let mut w = s.build();
    let id = one(&w, Kind::Vehicle, "MASHALL");
    set_overlay(&mut w, cell(0, 0), 5, 0);
    set_overlay(&mut w, cell(0, -1), 5, 0);
    let rng = w.rng.clone();
    crush::at_cell_centre(&mut w, id);
    assert_eq!(overlay_at(&w, cell(0, 0)), NO_OVERLAY);
    assert_eq!(w.rng, rng);
    // The neighbour (one stage only) was not hit: it still stands (a hit would remove it, s ≥ L).
    assert_eq!(overlay_at(&w, cell(0, -1)), 5);
}

/// A scene with FENCE walls at `walls` and HUTs of the given houses.
fn owner_world(walls: &[CellXY], huts: &[(u8, CellXY)], beta_flag: Option<bool>) -> World {
    let mut s = scene();
    s.ruleset
        .sim
        .countries
        .get_mut("Betaland")
        .expect("country")
        .wall_owner = beta_flag;
    for &c in walls {
        s.cell_mut(c).overlay = FENCE;
    }
    for &(h, c) in huts {
        s.place_structure(h, "HUT", c);
    }
    s.build()
}

fn wall_owner(w: &World, c: CellXY) -> Option<HouseId> {
    w.cells.wall_owner[w.cells.idx(c).expect("cell")]
}

// spec: S-0210 claim 14 (a) (the nearest building whose owner's country has the wall-owner flag; a flag off is skipped)
#[test]
fn map_walls_take_the_owner_of_the_nearest_qualifying_building() {
    let w1 = cell(0, 0);
    let w2 = cell(-14, 0);
    // Beta's hut 3 cells from w1 (flag off), Alpha's 6 cells away.
    let w = owner_world(
        &[w1, w2],
        &[(BETA, cell(3, 0)), (ALPHA, cell(-6, 0))],
        Some(false),
    );
    assert_eq!(wall_owner(&w, w1), Some(HouseId(ALPHA)));
    assert_eq!(wall_owner(&w, w2), Some(HouseId(ALPHA)));
    // With Beta's flag at its default (yes), Beta's nearer hut owns w1.
    let w = owner_world(&[w1], &[(BETA, cell(3, 0)), (ALPHA, cell(-6, 0))], None);
    assert_eq!(wall_owner(&w, w1), Some(HouseId(BETA)));
    // No qualifying building: no owner.
    let w = owner_world(&[w1], &[(BETA, cell(3, 0))], Some(false));
    assert_eq!(wall_owner(&w, w1), None);
}

// spec: S-0210 claim 14 (a) (no distance limit: a wall far from every building still takes the nearest qualifying building's owner)
#[test]
fn a_wall_far_from_every_building_still_has_an_owner() {
    let far = cell(-15, -15);
    let near = cell(13, 15);
    // Alpha's hut more than 40 cells from `far`; Beta's (flag off) much nearer.
    let w = owner_world(
        &[far, near],
        &[(BETA, cell(-12, -15)), (ALPHA, cell(15, 15))],
        Some(false),
    );
    assert_eq!(wall_owner(&w, far), Some(HouseId(ALPHA)));
    assert_eq!(wall_owner(&w, near), Some(HouseId(ALPHA)));
    // No building on the map: no owner, wherever the walls lie.
    let w = owner_world(&[far, near], &[], None);
    assert_eq!(wall_owner(&w, far), None);
    assert_eq!(wall_owner(&w, near), None);
}

// spec: S-0210 claim 14 (a) (equal distances: the building created first, in map order)
#[test]
fn equally_near_buildings_the_first_created_wins() {
    let w1 = cell(0, 0);
    let w = owner_world(&[w1], &[(BETA, cell(4, 0)), (ALPHA, cell(-4, 0))], None);
    assert_eq!(wall_owner(&w, w1), Some(HouseId(BETA)));
    let w = owner_world(&[w1], &[(ALPHA, cell(-4, 0)), (BETA, cell(4, 0))], None);
    assert_eq!(wall_owner(&w, w1), Some(HouseId(ALPHA)));
}

// spec: S-0210 claim 14 (c) (a wall that is removed has no owner from then on)
#[test]
fn a_crushed_wall_loses_its_owner() {
    let mut s = scene();
    s.cell_mut(cell(0, 0)).overlay = FENCE;
    s.place_structure(BETA, "HUT", cell(4, 0));
    s.place_unit(ALPHA, "MASHALL", cell(0, 0), 64);
    let mut w = s.build();
    assert_eq!(wall_owner(&w, cell(0, 0)), Some(HouseId(BETA)));
    let id = one(&w, Kind::Vehicle, "MASHALL");
    crush::at_cell_centre(&mut w, id);
    assert_eq!(overlay_at(&w, cell(0, 0)), NO_OVERLAY);
    assert_eq!(wall_owner(&w, cell(0, 0)), None);
}
