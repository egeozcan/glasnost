//! A jumpjet that joins a warp with a movement commitment loses it at the
//! launch, as the other locomotors' driving steps end, and stays where it
//! landed (#362; S-0513 claim 51, SW12 part b8; S-0502 claim 17). On the
//! synthetic superweapon rules of `sw_common` (JJ a CART with jumpjet
//! movement), no game data.

mod sw_common;

use glasnost_map::CellXY;
use glasnost_rules::schema::Ruleset;
use glasnost_sim::superweapons::warp::Stage;
use glasnost_sim::world::{ObjectId, Target, World};
use sw_common::*;

fn rules_jj() -> Ruleset {
    let mut r = ruleset();
    {
        let g = &mut r.sim.general.superweapons;
        g.chrono_placement = Some("MARK".into());
        g.chrono_blast = Some("BLASTS".into());
        g.chrono_blast_dest = Some("BLASTD".into());
        g.chrono_delay = Some(7);
    }
    r.sim.general.warp_out = Some("WARPA".into());
    for n in ["MARK", "BLASTS", "BLASTD", "WARPA"] {
        r.sim
            .animations
            .insert(n.into(), glasnost_rules::sim::AnimationType::default());
        r.registries
            .entry("animations".into())
            .or_default()
            .push(n.into());
    }
    let mut t = r.sim.vehicles["CART"].clone();
    t.locomotor = Some(glasnost_rules::sim::Locomotor::Jumpjet);
    t.jumpjet_height = Some(1000);
    r.sim.vehicles.insert("JJ".into(), t);
    r.registries
        .entry("vehicles".into())
        .or_default()
        .push("JJ".into());
    r
}

fn stage(w: &World, id: ObjectId) -> Option<Stage> {
    w.obj(id).and_then(|o| o.sw.warp.as_ref()).map(|m| m.stage)
}

fn warp_with(w: &mut World, src: CellXY, dst: CellXY, prep: impl FnOnce(&mut World)) -> i32 {
    if !rec(w, ALPHA, SOURCE).ready {
        ready_tick(w, ALPHA, SOURCE, 300);
    }
    fire_now(w, ALPHA, SOURCE, src);
    prep(w);
    fire(w, ALPHA, WARP, dst);
    let tl = w.tick;
    step(w);
    tl
}

// spec: S-0513 claim 51 (launch immediately clears the old path, queued destinations and current driving step; none resumes when the warp ends) and SW12 part (b8) set-up; S-0502 claim 17 (stopping a jumpjet that is not moving does nothing)
#[test]
fn issue_362_a_jumpjet_ascending_at_the_launch_stays_where_it_landed() {
    let mut w = game(&rules_jj());
    spawn(&mut w, "CHRONOB", ALPHA, -9, -9);
    ready_tick(&mut w, ALPHA, SOURCE, 300);
    let j = spawn(&mut w, "JJ", BETA, 0, 0);
    let ok = glasnost_sim::orders::issue(
        &mut w,
        BETA,
        glasnost_sim::orders::Order::Behaviour {
            actor: j,
            behaviour: glasnost_sim::rules::Behaviour::Move,
            attack_move: false,
            target: None,
            dest: Some(Target::Cell(at(-4, 0))),
        },
    );
    assert!(ok, "the move order is taken");
    let (o0, d0) = (at(0, 0), at(5, 5));
    let mut launch_h = 0;
    let tl = warp_with(&mut w, o0, d0, |w| {
        until(w, 200, |w| {
            let mut p = w.clone();
            step(&mut p);
            let h = p.height_above_surface(j);
            launch_h = h;
            h > 0 && h > w.height_above_surface(j)
        });
    });
    assert!(
        launch_h > 0 && launch_h < 208,
        "set-up: rising below the airborne height"
    );
    assert_eq!(stage(&w, j), Some(Stage::Pending), "set-up: it joins");
    let mut landed = None;
    while w.tick <= tl + 200 {
        step(&mut w);
        let t = w.tick - 1 - tl;
        let o = w.obj(j).expect("the unit is in play");
        let jj = o.jump.as_ref().unwrap();
        if landed.is_none() && o.on_map && t >= 63 {
            landed = Some(o.pos);
            assert_eq!(w.height_above_surface(j), 0, "it lands at no height");
        }
        if let Some(p) = landed {
            assert!(o.dest.is_none(), "set-up: no destination after the warp");
            assert_eq!(
                (o.pos, w.height_above_surface(j)),
                (p, 0),
                "t_L+{t}: with no order and no destination it stays where it landed (phase {:?}, moving {}, point {:?})",
                jj.phase,
                jj.moving,
                jj.dest.map(|p| p.cell())
            );
        }
    }
    assert!(landed.is_some(), "it appeared");
}
