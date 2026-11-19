//! A jumpjet vehicle's body and the tilt-crash flag (#446; S-0113 r21
//! claim 18): two invented jumpjet vehicle types, one with the flag, rocked
//! by the same push. Invented types and map, no game data.

use std::sync::Arc;

use glasnost_render::palette::{PaletteQuirks, PaletteState};
use glasnost_render::scene::Scene;
use glasnost_rules::sim::{Locomotor, UnitType};
use glasnost_sim::combat::effects::Effect;
use glasnost_sim::rules::{Kind as SimKind, SimRules};
use glasnost_sim::testkit;
use glasnost_sim::world::{ObjectId, Pos};

use super::build::{Attitude, Inputs, Options, build_parts};
use super::live::Live;
use super::tests::{ART, RULES, context_full};

const SIZE: i32 = 24;

/// The simulation's rules: the synthetic combat ruleset with JJA and JJB,
/// two vehicle types of the jumpjet locomotor.
fn rules() -> Arc<SimRules> {
    let mut r = testkit::combat_ruleset();
    for name in ["JJA", "JJB"] {
        r.registries.get_mut("vehicles").unwrap().push(name.into());
        r.sim.vehicles.insert(
            name.into(),
            UnitType {
                strength: Some(400),
                speed: Some(4),
                locomotor: Some(Locomotor::Jumpjet),
                jumpjet: Some(true),
                jumpjet_height: Some(500),
                ..Default::default()
            },
        );
    }
    Arc::new(SimRules::new(&r, "synthetic-test"))
}

/// Each type's drawn attitude after a push of `damage` from 200 leptons
/// behind it, a tick on: (JJA, which the rules text gives the tilt-crash
/// flag, and JJB, which it does not).
fn attitudes(damage: i32) -> (Attitude, Attitude) {
    let text = RULES.replacen(
        "[VehicleTypes]\n1=CARX\n",
        "[VehicleTypes]\n1=CARX\n2=JJA\n3=JJB\n",
        1,
    ) + "[JJA]\nTiltCrashJumpjet=yes\n[JJB]\n";
    let (import, catalogue, assets) = context_full(&text, ART);
    let rules = rules();
    let setup = testkit::setup(7);
    let mut world = glasnost_sim::load::new_game(
        rules.clone(),
        &testkit::empty_map(SIZE),
        testkit::catalogue(),
        &setup,
    )
    .expect("the synthetic game loads");
    let a = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "JJA",
        0,
        testkit::mid(SIZE, -3, 0),
        0,
    );
    let b = testkit::spawn(
        &mut world,
        SimKind::Vehicle,
        "JJB",
        0,
        testkit::mid(SIZE, 3, 0),
        0,
    );
    let map = testkit::empty_map(SIZE);
    let palettes = PaletteState::at_program_start(None, None, PaletteQuirks::default());
    let options = Options::default();
    let cells = world.cells.grid.clone();
    let inputs = Inputs {
        map: &map,
        import: &import,
        catalogue: &catalogue,
        palettes: &palettes,
        assets: &assets,
        options: &options,
        rules: Some(&rules),
        cells: Some(&cells),
    };
    let (mut built, painter) = build_parts(&inputs, true);
    let mut live = Live::new(
        &inputs,
        &mut built,
        painter,
        (world, rules.clone(), setup, Vec::new()),
    );
    let mut scene: Scene = built.scene;
    live.redraw(&inputs, &mut scene);
    let behind = |live: &Live, id: ObjectId| {
        let p = live.world.obj(id).unwrap().pos;
        Pos::new(p.x, p.y + 200, p.z)
    };
    let pushed = vec![(a, behind(&live, a)), (b, behind(&live, b))];
    live.world.effects.push(Effect::Rock { damage, pushed });
    // The tick's pushes set the rates; the next tick's step moves the angles
    // (S-0113 claim 19).
    live.after_tick();
    live.step();
    live.redraw(&inputs, &mut scene);
    let of = |id| live.mobile(&inputs, id).expect("drawn").attitude;
    (of(a), of(b))
}

/// #446. spec: S-0113 r21 claim 18 (the jumpjet locomotor draws the pitch and roll only for a type
/// with the tilt-crash flag; otherwise the body is drawn turned to its body facing only, with no
/// pitch or roll, while the angles still step) and Import mapping (`TiltCrashJumpjet`, off when
/// absent)
#[test]
fn a_jumpjet_draws_its_rocking_only_with_the_tilt_crash_flag() {
    let (with, without) = attitudes(400);
    let Attitude::Rocking { pitch, drawn, .. } = with else {
        panic!("the flagged jumpjet rocks: {with:?}")
    };
    assert!(f32::from_bits(pitch) != 0.0, "its angles step");
    assert!(drawn, "with the flag its locomotor draws the angles");
    let Attitude::Rocking { pitch, drawn, .. } = without else {
        panic!("the other jumpjet rocks too: {without:?}")
    };
    assert!(f32::from_bits(pitch) != 0.0, "its angles step as well");
    assert!(!drawn, "without the flag its body is drawn level");
}
