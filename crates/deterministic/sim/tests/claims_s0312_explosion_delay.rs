//! S-0312 claim 19 (b) (#175): each foundation cell's explosion animation
//! of a destroyed building starts after its drawn delay of 0 to 3 ticks,
//! S-0109 claim 11's initial loop delay. Synthetic data: an invented 3 × 3
//! building and an invented animation type with no creation or start-event
//! draws.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::{AnimationType, BuildingType, CombatFields};
use glasnost_sim::combat::damage::{self, Hit};
use glasnost_sim::combat::death;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit;
use glasnost_sim::world::Pos;

const SIZE: i32 = 24;

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    r.registries
        .entry("buildings".into())
        .or_default()
        .push("HUT".into());
    r.registries
        .entry("animations".into())
        .or_default()
        .push("BX".into());
    // Four frames at a fixed rate, no middle-event effects: no creation or start-event draws.
    r.sim.animations.insert(
        "BX".into(),
        AnimationType {
            frame_count: Some(4),
            rate: Some(900),
            middle_frame: Some(2),
            ..Default::default()
        },
    );
    r.sim.buildings.insert(
        "HUT".into(),
        BuildingType {
            strength: Some(500),
            foundation: Some([3, 3]),
            combat: CombatFields {
                explosion: Some(vec!["BX".into()]),
                ..Default::default()
            },
            ..Default::default()
        },
    );
    r
}

// spec: S-0312 claim 19 (b) (each foundation cell's explosion animation at N(cell centre, 64, b), with a starting delay of a ranged draw over 0 to 3 ticks, its type by one list pick) and S-0109 claim 11 (a loop delay at creation, when the creator asks for one)
#[test]
fn claim_19b_foundation_explosions_take_their_drawn_starting_delay() {
    let r = rules();
    let mut w = testkit::game_with(&r, SIZE, 11);
    let id = testkit::spawn(&mut w, Kind::Building, "HUT", 0, testkit::mid(SIZE, 0, 0), 0);
    let o = w.obj(id).unwrap().clone();
    let half = w.rules().quirks.ranged_draw_half_span;
    // Replay claim 19 (a) and (b) on a copy of the stream.
    let mut rr = w.rng.clone();
    rr.ranged(0, 1, half);
    rr.ranged(0, 1, half);
    rr.percent();
    let mut expect = Vec::new();
    let tl = o.pos.cell();
    for dy in 0..3 {
        for dx in 0..3 {
            let c = glasnost_map::CellXY::new(tl.x + dx, tl.y + dy);
            let raw = rr.raw();
            let p = death::nearby_point(&w, Pos::centre(c, o.pos.z), 64, raw, false);
            let d = rr.ranged(0, 3, half);
            rr.list_pick(1);
            expect.push((p, d));
        }
    }
    assert!(w.anims.anims.is_empty());
    damage::apply(
        &mut w,
        id,
        Hit {
            amount: 100_000,
            warhead: None,
            distance: 0,
            attacker: None,
            house: None,
            ignore_defences: true,
            no_escape: false,
        },
    );
    assert_eq!(w.rng, rr, "the draws are claim 19's");
    let got: Vec<(Pos, i32)> = w
        .anims
        .anims
        .values()
        .map(|a| (a.pos.unwrap(), a.player.loop_delay()))
        .collect();
    assert!(expect.iter().any(|(_, d)| *d > 0), "the seed draws a delay");
    assert_eq!(got, expect, "(position, starting delay) of each explosion");
}
