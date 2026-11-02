//! Claim tests for S-0216 revision 59 claim 3 (the draws of a play-time
//! placement when a unit's creation makes other objects: scenario PT3's
//! third and fourth runs), on synthetic data with invented type names and
//! values. M8 step A3 intake 77; the ledger is `docs/m8-claims/intake77.md`.
//!
//! Added types: aircraft DRN (plain); infantry SLV (plain) and NEST (names
//! the spawned type DRN, count 2); vehicle HERD (names the slave type SLV,
//! count 2). The placing step is the testkit's play-time harness
//! (`testkit::spawn_in_play`), as in `claims_s0216.rs`.

use glasnost_rules::schema::Ruleset;
use glasnost_rules::sim::UnitType;
use glasnost_sim::cells::position;
use glasnost_sim::rng::Rng;
use glasnost_sim::rules::Kind;
use glasnost_sim::testkit::{self, mid};
use glasnost_sim::walk;
use glasnost_sim::world::{ObjectId, World};

const SIZE: i32 = 30;

fn rules() -> Ruleset {
    let mut r = testkit::combat_ruleset();
    let push = |r: &mut Ruleset, reg: &str, n: &str| {
        r.registries.entry(reg.into()).or_default().push(n.into());
    };
    push(&mut r, "aircraft", "DRN");
    push(&mut r, "infantry", "SLV");
    push(&mut r, "infantry", "NEST");
    push(&mut r, "vehicles", "HERD");
    let walker = r.sim.infantry["WALKER"].clone();
    let cart = r.sim.vehicles["CART"].clone();
    let s = &mut r.sim;
    s.aircraft.insert(
        "DRN".into(),
        UnitType {
            strength: Some(50),
            speed: Some(10),
            ..Default::default()
        },
    );
    s.infantry.insert("SLV".into(), walker.clone());
    let mut nest = walker;
    nest.combat.spawns = Some("DRN".into());
    nest.combat.spawns_number = Some(2);
    s.infantry.insert("NEST".into(), nest);
    let mut herd = cart;
    herd.econ.enslaves = Some("SLV".into());
    herd.econ.slaves_number = Some(2);
    s.vehicles.insert("HERD".into(), herd);
    r
}

/// The standing position of infantry unit `id` in its cell.
fn standing(w: &World, id: ObjectId) -> u8 {
    let p = w.obj(id).unwrap().pos;
    let (u, v) = (p.x.rem_euclid(256), p.y.rem_euclid(256));
    [
        position::CENTRE,
        position::TOP,
        position::RIGHT,
        position::LEFT,
        position::BOTTOM,
    ]
    .into_iter()
    .find(|&k| walk::standing_point(k) == (u, v))
    .expect("at a standing position")
}

/// The first position of row R*k* of S-0208 table O (an empty cell).
fn first_of_row(k: i32) -> u8 {
    // spec: S-0208 table O (rows R0 to R3: k = 0 or 1 right first, 2 left first, 3 bottom first)
    match k {
        0 | 1 => position::RIGHT,
        2 => position::LEFT,
        _ => position::BOTTOM,
    }
}

/// Places the four slots of PT3 in order on empty cells and checks, against
/// a cloned stream, every placed or made object's creation draw (its report
/// value, S-0202 claim 10) and every infantry unit's standing position.
fn pt3(seed: u32, order: [(Kind, &str); 4], made: [u32; 4]) {
    let mut w = testkit::game_with(&rules(), SIZE, seed);
    let half = w.rules().quirks.ranged_draw_half_span;
    let mut probe: Rng = w.rng.clone();
    for (i, (kind, ty)) in order.into_iter().enumerate() {
        let c = mid(SIZE, -6 + 4 * i as i32, 0);
        let first = w.next_id;
        let id = testkit::spawn_in_play(&mut w, kind, ty, 0, c, 0, None)
            .expect("placed on an empty cell");
        assert_eq!(
            id.0, first,
            "slot {i}: the unit takes the first creation number"
        );
        // spec: S-0216 claim 3 (r59: the unit's creation draw, then the creation draws of the objects its creation makes, each one raw value with its creation number, S-0611 claim 14's order)
        for n in 0..=made[i] {
            let x = ObjectId(first + n);
            let want = (probe.raw() & 0xFFFF) as u16; // spec: S-0202 claim 10 (the creation draw; its report value r = x mod 65536)
            assert_eq!(
                w.obj(x).unwrap().report_value,
                want,
                "slot {i}: object {n} of its creation set"
            );
        }
        if kind == Kind::Infantry {
            // spec: S-0216 claims 2 and 3 (the standing-position draw R(0, 3) after all the creation draws)
            let k = probe.ranged(0, 3, half);
            assert_eq!(standing(&w, id), first_of_row(k), "slot {i}: draw {k}");
        }
        assert_eq!(w.next_id, first + 1 + made[i], "slot {i}: no other number");
    }
    // spec: S-0216 claim 3 (no other draw in the window)
    assert_eq!(w.rng, probe, "no other draw");
}

// spec: S-0216 claim 3 (r59) and scenario PT3's third run (the second slot's infantry type makes slot objects: its creation draw, then its slot objects' creation draws in S-0506 claim 1's order, then its R(0, 3))
#[test]
fn claim_3_pt3_third_run_slot_objects_draw_before_the_standing_position() {
    for seed in [2, 3, 4] {
        pt3(
            seed,
            [
                (Kind::Vehicle, "CART"),
                (Kind::Infantry, "NEST"),
                (Kind::Vehicle, "CART"),
                (Kind::Infantry, "WALKER"),
            ],
            [0, 2, 0, 0],
        );
    }
}

// spec: S-0216 claim 3 (r59) and scenario PT3's fourth run (the first slot's vehicle makes an initial slave group: its creation draw, one creation draw per slave in S-0611 claim 14's order, then the second slot's creation draw; the slaves take no R(0, 3))
#[test]
fn claim_3_pt3_fourth_run_slaves_draw_before_the_next_unit() {
    for seed in [2, 3, 4] {
        pt3(
            seed,
            [
                (Kind::Vehicle, "HERD"),
                (Kind::Infantry, "WALKER"),
                (Kind::Vehicle, "CART"),
                (Kind::Infantry, "WALKER"),
            ],
            [2, 0, 0, 0],
        );
    }
}
