//! S-0205 claim 1 at S-0615 claim 11's expiry stage (S20's seam of #381):
//! a burning walker whose running frames make 8 R + 1 overflow. Synthetic
//! rules: the test kit's combat rules plus BURN, a burning walker of 80
//! frames whose running frames are invented to overflow. No game data.

use glasnost_map::CellXY;
use glasnost_map::terrain::land;
use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::AnimationType;
use glasnost_sim::world::{Pos, World};
use glasnost_sim::{testkit, walker};

const SIZE: i32 = 20;
/// BURN's running frames *R*: 8 R overflows the 32-bit range.
const R: i32 = i32::MAX / 8 + 1;

fn register(r: &mut Ruleset, reg: &str, name: &str) {
    let l = r.registries.entry(reg.into()).or_default();
    if !l.iter().any(|n| n == name) {
        l.push(name.into());
    }
}

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    register(&mut r, "animations", "BURN");
    r.sim.animations.insert(
        "BURN".into(),
        AnimationType {
            frame_count: Some(80),
            is_flaming_guy: Some(true),
            running_frames: Some(R),
            ..Default::default()
        },
    );
    r.sim.general.flaming_infantry = Some("BURN".into());
    r
}

fn c(dx: i32, dy: i32) -> CellXY {
    testkit::mid(SIZE, dx, dy)
}

fn anim_index(w: &World, name: &str) -> u32 {
    w.rules()
        .animations
        .iter()
        .position(|a| a.name == name)
        .unwrap() as u32
}

// spec: S-0615 claim 11 (expiry sets the stage to 8R + 1) with S-0205 claim 1 (an integer result outside the range wraps); S-0615 claim 9 (arrival at a water cell at most 104 above the terrain expires)
#[test]
fn walker_expiry_stage_wraps_for_huge_running_frames() {
    assert!(8i32.checked_mul(R).is_none(), "set-up: 8 R overflows");
    let mut w = testkit::game_with(&rules(), SIZE, 7);
    w.cells.grid.get_mut(c(1, 0)).unwrap().land = land::WATER;
    let a = anim_index(&w, "BURN");
    walker::create(&mut w, a, Pos::centre(c(0, 0), 0));
    let wid = *w.walkers.keys().last().expect("made");
    for _ in 0..40 {
        testkit::run(&mut w, 1);
        if w.walkers[&wid].expiring {
            break;
        }
    }
    let wk = &w.walkers[&wid];
    assert!(wk.expiring, "set-up: the walker expires");
    assert_eq!(wk.stage, 8i32.wrapping_mul(R).wrapping_add(1));
}
