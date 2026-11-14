//! S-0210 claim 9: a crusher-all vehicle enters the crush-slowdown state
//! each time it drives into a new cell on a crush-mode track, also when the
//! entry coincides with the end of the track (#265). Synthetic types and map.

mod common;

use common::{Scene, TRACK, cell, objects_of, open_orders, order_move, run, unit_type};
use glasnost_rules::sim::{MovementZone, OverlayType};
use glasnost_sim::rules::Kind;

/// Drives a crusher-all vehicle (no crusher flag, not accelerating) with
/// speed field `speed` east along a clear row from dx = -14 to dx = 14,
/// past a crushable overlay at dx = 8. Returns the dx of the cell it stood
/// in at the end of the first tick on which it was in the crush-slowdown
/// state.
fn drive(speed: i32) -> Option<i32> {
    let mut s = Scene::new();
    let mut t = unit_type(TRACK, speed, 8);
    t.movement_zone = Some(MovementZone::CrusherAll);
    t.accelerates = Some(false);
    s.vehicle("ROLLER", t);
    s.ruleset
        .registries
        .insert("overlays".into(), vec!["BRUSH".to_string()]);
    s.ruleset.sim.overlays.insert(
        "BRUSH".into(),
        OverlayType {
            has_art: Some(true),
            crushable: Some(true),
            ..Default::default()
        },
    );
    s.cell_mut(cell(8, 0)).overlay = 0;
    s.place_unit(0, "ROLLER", cell(-14, 0), 64);
    let mut w = s.build();
    let id = objects_of(&w, Kind::Vehicle)[0];
    open_orders(&mut w);
    order_move(&mut w, id, cell(14, 0));
    let mut entered = None;
    for _ in 0..3000 {
        run(&mut w, 1);
        let o = w.obj(id).unwrap();
        let d = o.drive.as_ref().unwrap();
        if o.crush_slowdown && entered.is_none() {
            entered = Some(o.pos.cell().x - cell(0, 0).x);
        }
        if o.pos.cell() == cell(14, 0) && d.track.is_none() && o.path.steps.is_empty() {
            break;
        }
    }
    assert_eq!(w.obj(id).unwrap().pos.cell(), cell(14, 0), "arrived");
    entered
}

// spec: S-0210 claim 9 (a vehicle on the ground enters the state each time it drives into a new cell on a crush-mode track); S-0207 claim 7 (the track into the cell before the overlay is in crush mode)
#[test]
fn issue_265_every_speed_enters_the_slowdown_in_the_cell_before_the_overlay() {
    let mut wrong = Vec::new();
    for v in [6, 30, 40, 45, 50, 60, 64, 65, 70, 100] {
        let entered = drive(v);
        // The track from dx 6 to dx 7 is in crush mode (its second cell holds the overlay).
        if entered != Some(7) {
            wrong.push((v, entered));
        }
    }
    assert!(
        wrong.is_empty(),
        "not in the state at dx 7: (speed field, first dx) {wrong:?}"
    );
}
