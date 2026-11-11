//! Our own pathing improvements (M8 step C7 part 4, the lead's decision of
//! 2026-10-29; ledger `docs/m8-claims/c7-pathing.md`), on the synthetic
//! scenario set of `pathing_common`. Each improvement is a quirk switch
//! whose default is the original's behaviour and whose off position our
//! patch layer *Pathing and other fixes* sets; its tests run both
//! positions.
//!
//! `measure` (ignored) plays the whole set with the layer off and on and
//! prints the ledger's table: `cargo test --release -p glasnost-sim --test
//! pathing_c7p4 measure -- --ignored --nocapture`.

#![allow(
    clippy::disallowed_methods,
    clippy::disallowed_types,
    clippy::disallowed_macros
)]

mod pathing_common;

use pathing_common::*;

/// Our switches (C7 parts 4 to 6), each defaulting to the original.
const DESIGN: [&str; 7] = [
    "path_through_stalled_blockers",
    "deadlock_no_yield",
    "jammed_move_dropped",
    "unreachable_move_dropped",
    "group_spread_no_detour",
    "rally_point_shared",
    "harvest_cell_shared",
];

/// The seeds the table is measured over.
const SEEDS: [u32; 3] = [7, 8, 9];

// design: C7-P5 tick cost (the wall time of a tick with the layer against the reference's, the fastest of ROUNDS plays each, taken in turn)
#[test]
#[ignore = "measurement: prints the tick cost of the layer against the reference"]
fn cost() {
    let rounds: usize = std::env::var("ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5);
    let sets = [
        ("reference", Rules::Reference),
        ("layer", Rules::Layer),
        ("layer, no yield", layer_without(&["deadlock_no_yield"])),
        (
            "layer, stalled blockers on",
            layer_without(&["path_through_stalled_blockers", "deadlock_no_yield"]),
        ),
        // design: C7-P6 hold (the layer without the hold, for its tick cost)
        ("layer, no hold", layer_without(&["jammed_move_dropped"])),
    ];
    println!(
        "| Scenario | {} ns/tick |",
        sets.map(|(l, _)| l).join(" ns/tick | ")
    );
    let names: Vec<&str> = all(Rules::Reference, 7).iter().map(|s| s.name).collect();
    for (i, name) in names.iter().enumerate() {
        let mut best = [u128::MAX; 5];
        for _ in 0..rounds {
            for (k, (_, r)) in sets.iter().enumerate() {
                let s = all(*r, 7).swap_remove(i);
                let bound = s.bound.max(1) as u128;
                let t0 = std::time::Instant::now();
                play(s);
                best[k] = best[k].min(t0.elapsed().as_nanos() / bound);
            }
        }
        println!(
            "| {name} | {} | {} | {} | {} | {} |",
            best[0], best[1], best[2], best[3], best[4]
        );
    }
}

#[test]
#[ignore = "measurement: prints the scenario table"]
fn measure() {
    // The reference, the layer, each switch of ours alone off, and the
    // layer less each of ours (SETS=a,b picks sets by label).
    let mut sets: Vec<(String, Rules)> = vec![
        ("reference".into(), Rules::Reference),
        ("layer".into(), Rules::Layer),
    ];
    for n in DESIGN {
        sets.push((n.into(), off(&[n])));
    }
    for n in DESIGN {
        sets.push((format!("layer less {n}"), layer_without(&[n])));
    }
    if let Ok(pick) = std::env::var("SETS") {
        let pick: Vec<&str> = pick.split(',').collect();
        sets.retain(|(l, _)| pick.contains(&l.as_str()));
    }
    for (label, rules) in sets {
        println!("\n{label}\n\n{HEADER}");
        let mut by_name: Vec<(&'static str, Vec<Metrics>)> = Vec::new();
        for seed in SEEDS {
            for s in all(rules, seed) {
                let name = s.name;
                let m = play(s);
                match by_name.iter_mut().find(|(n, _)| *n == name) {
                    Some((_, v)) => v.push(m),
                    None => by_name.push((name, vec![m])),
                }
            }
        }
        for (name, ms) in by_name {
            // The seeds pooled: arrivals joined, counts summed.
            let mut m = Metrics::default();
            for x in &ms {
                m.units += x.units;
                m.arrived.extend(&x.arrived);
                m.missing += x.missing;
                m.give_ups += x.give_ups;
                m.open_give_ups += x.open_give_ups;
                m.late_give_ups += x.late_give_ups;
                m.reversals += x.reversals;
                m.jostles += x.jostles;
                m.unsettled += x.unsettled;
                m.shared += x.shared;
                m.searches += x.searches;
                m.failed += x.failed;
                m.credits += x.credits;
                m.micros += x.micros;
            }
            m.micros /= ms.len() as u64;
            println!("{}", line(name, rules, &m));
        }
    }
}

#[test]
#[ignore = "diagnostic"]
fn diag() {
    use glasnost_sim::trace::{self, What};
    let name = std::env::var("DIAG").unwrap_or_default();
    // OFF: the switches to set off (comma-separated) on the reference.
    let names = std::env::var("OFF").unwrap_or_default();
    let names: Vec<&str> = names.split(',').filter(|n| !n.is_empty()).collect();
    let seed = std::env::var("SEED")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);
    let mut s = all(off(&names), seed)
        .into_iter()
        .find(|s| s.name == name)
        .expect("scene");
    trace::switch_on(&mut s.w);
    let ids: Vec<_> = s.goals.iter().map(|g| g.id).collect();
    let watch = ids[std::env::var("WATCH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)];
    let quiet = std::env::var("QUIET").is_ok();
    for t in 0..s.bound {
        if let Some(f) = s.each.as_mut() {
            f(&mut s.w);
        }
        glasnost_sim::tick::step(&mut s.w);
        for e in trace::take(&s.w) {
            if matches!(
                e.what,
                What::GaveWay { .. } | What::Resumed | What::GaveUp | What::Failed { .. }
            ) {
                let i = ids.iter().position(|&x| x == e.unit);
                let at =
                    s.w.obj(e.unit)
                        .map(|o| (o.pos.cell(), o.dest, o.beh.current));
                println!("{} unit {i:?} {:?} at {at:?}", e.tick, e.what);
            }
            if e.unit == watch && t < 1200 {
                match &e.what {
                    What::Step { .. } | What::Blocked { .. } if quiet => {}
                    What::Search {
                        from,
                        to,
                        result,
                        steps,
                        congested,
                        mode,
                        ..
                    } => println!(
                        "{} search {:?} {from:?}->{to:?} {result:?} steps {} cong {} ",
                        e.tick,
                        mode,
                        steps.len(),
                        congested.len()
                    ),
                    other => println!("{} {other:?}", e.tick),
                }
            }
        }
        if t % 100 == 0 {
            let mid = pathing_common::at(0, 0);
            let cells: Vec<_> = ids
                .iter()
                .map(|&i| {
                    let o = s.w.obj(i).unwrap();
                    let c = o.pos.cell();
                    (c.x - mid.x, c.y - mid.y, o.beh.current)
                })
                .collect();
            println!("t={t} {cells:?}");
        }
    }
}

/// Plays `scene` with the reference switches and with `switch` alone off.
fn both(switch: &str, scene: impl Fn(Rules, u32) -> Scene) -> (Metrics, Metrics) {
    (
        play(scene(Rules::Reference, 7)),
        play(scene(off(&[switch]), 7)),
    )
}

/// What the units did in `m`: arrivals and the counts of the movement
/// (not the state hash, which also holds timers, nor the wall time).
fn moves(m: &Metrics) -> (Vec<i32>, usize, u32, u32, u32, u32) {
    (
        m.arrived.clone(),
        m.missing,
        m.give_ups,
        m.reversals,
        m.jostles,
        m.searches,
    )
}

// spec: S-0212 claims 8 and 22 (the original: a mode-2 re-path leads through moving blockers, so two groups that meet head-on in a 2-wide passage stay stuck); design: C7-P4 stalled blockers (off: the stalled units of a deadlock are kept off, and every unit arrives)
#[test]
fn stalled_blockers_off_clears_a_head_on_deadlock() {
    let (on, off) = both("path_through_stalled_blockers", |r, s| corridor(r, 2, 6, s));
    assert_eq!(on.missing, 12, "the original's deadlock: none arrives");
    assert_eq!(off.missing, 0, "off: every unit arrives");
    assert_eq!(off.give_ups, 0);
}

// design: C7-P4 stalled blockers (a queue whose head still moves is no deadlock: the off position moves no unit otherwise in a passage that only queues, nor where groups pass each other)
#[test]
fn stalled_blockers_off_leaves_queues_alone() {
    let (on, off) = both("path_through_stalled_blockers", |r, s| {
        infantry_gap(r, 16, s)
    });
    assert_eq!(moves(&on), moves(&off));
    let (on, off) = both("path_through_stalled_blockers", |r, s| gap(r, 1, 20, s));
    assert_eq!(moves(&on), moves(&off));
    let (on, off) = both("path_through_stalled_blockers", |r, s| corridor(r, 2, 3, s));
    assert_eq!(moves(&on), moves(&off));
}

// spec: S-0212 claims 6 and 23 (the original: a destination outside the unit's reach region fails the gate, the destination is cleared and the units do not move; one walled in by buildings fails the search and the units give up); design: C7-P4 nearest reachable cell (off: they go to the nearest cells they can stand on)
#[test]
fn unreachable_move_off_goes_to_the_nearest_reachable_cell() {
    let (on, off) = both("unreachable_move_dropped", |r, s| unreachable(r, 8, s));
    assert_eq!(on.missing, 8);
    assert_eq!(on.failed, 8, "each unit's computation fails once");
    assert_eq!(off.missing, 0);
    assert_eq!(off.failed, 0);
    let (on, off) = both("unreachable_move_dropped", |r, s| walled_in(r, 8, s));
    assert_eq!(on.missing, 8);
    assert_eq!(on.give_ups, 8);
    assert_eq!(off.missing, 0);
    assert_eq!(off.give_ups, 0);
}

// spec: S-0213 claim 7 and Edge cases (the original: a spoiled look gives the fallback cell, so members share cells beside the obstacle and give up there); design: C7-P4 group spread (off: each member gets a cell of its own)
#[test]
fn group_spread_off_gives_each_member_its_own_cell() {
    let (on, off) = both("group_spread_no_detour", |r, s| by_buildings(r, 16, s));
    assert!(on.shared > 0 && on.give_ups > 0, "{on:?}");
    assert_eq!(off.shared, 0);
    assert_eq!(off.give_ups, 0);
    assert_eq!(off.missing, 0);
    let (on, off) = both("group_spread_no_detour", |r, s| shore(r, 16, s));
    assert!(on.shared > 0, "{on:?}");
    assert_eq!(off.shared, 0);
    assert_eq!(off.missing, 0);
}

// spec: S-0417 claims 8 and 10 and S-0404 claim 13 (the original: every released unit is sent to the rally cell, so a stream crowds it and gives up after arriving); design: C7-P5 rally spread (off: each released unit gets a cell of its own round the rally point)
#[test]
fn rally_spread_off_gives_each_released_unit_its_own_cell() {
    let (on, off) = both("rally_point_shared", |r, s| factory(r, 8, 5, s));
    assert_eq!(on.shared, 8, "the original: one rally cell for all");
    assert!(on.late_give_ups > 0, "{on:?}");
    assert_eq!(off.shared, 0, "off: a cell each");
    assert_eq!(off.give_ups, 0);
    assert_eq!(off.missing, 0);
}

// spec: S-0402 claim 4 (the original: the ring search accepts an ore cell another harvester is heading for, so harvesters sent from one place pick the same cells and give up one step short); design: C7-P5 field cells (off: each takes a cell of its own, with fewer give-ups and no fewer credits)
#[test]
fn field_cells_off_gives_each_harvester_its_own_cell() {
    let (on, off) = both("harvest_cell_shared", |r, s| harvesters(r, 4, s));
    assert!(on.shared > 0, "the original: shared first cells {on:?}");
    assert!(on.give_ups > 0, "{on:?}");
    assert_eq!(off.shared, 0, "off: a cell each {off:?}");
    assert!(off.give_ups < on.give_ups, "{on:?} {off:?}");
    assert!(off.credits >= on.credits, "{on:?} {off:?}");
}

// spec: S-0212 claims 8 and 22 (the original, and C7-P4's detour alone: two groups meeting in a passage with no way round stay stuck); design: C7-P5 yield (off: the side heading against the oldest unit gives way, and every unit arrives)
#[test]
fn yield_off_clears_a_deadlock_with_no_way_round() {
    let detour = off(&["path_through_stalled_blockers"]);
    let both_off = off(&["path_through_stalled_blockers", "deadlock_no_yield"]);
    let on = play(rooms(detour, 6, 7));
    let off = play(rooms(both_off, 6, 7));
    assert!(on.missing > 0, "{on:?}");
    assert_eq!(off.missing, 0, "{off:?}");
    assert_eq!(off.give_ups, 0);
    assert_eq!(off.unsettled, 0);
}

// design: C7-P5 yield (the yield acts only on a deadlock that C7-P4's stalled-blocker graph finds and that has no way round: alone it changes nothing, and with the graph it leaves queues, and a deadlock with a way round, alone)
#[test]
fn yield_off_leaves_queues_and_the_reference_alone() {
    let (a, b) = both("deadlock_no_yield", |r, s| rooms(r, 6, s));
    assert_eq!(
        moves(&a),
        moves(&b),
        "without the graph no deadlock is found"
    );
    let detour = off(&["path_through_stalled_blockers"]);
    let both_off = off(&["path_through_stalled_blockers", "deadlock_no_yield"]);
    for scene in [
        |r, s| infantry_gap(r, 16, s),
        |r, s| gap(r, 1, 20, s),
        |r, s| corridor(r, 2, 3, s),
        |r, s| corridor(r, 2, 6, s),
    ] as [fn(Rules, u32) -> Scene; 4]
    {
        assert_eq!(
            moves(&play(scene(detour, 7))),
            moves(&play(scene(both_off, 7)))
        );
    }
}

// spec: S-0212 claim 23 (the original: units that meet nose to nose in a passage one cell wide fail their searches and give their moves up, and the layer of parts 4 and 5 finds no deadlock among units with no destination); design: C7-P6 hold and three ways (off: the moves are held, the deadlock forms and gives way, and every unit arrives, three groups at one crossing among them)
#[test]
fn hold_off_clears_passages_one_cell_wide() {
    let held = layer_without(&["jammed_move_dropped"]);
    for scene in [
        |r, s| three_ways(r, 3, s),
        |r, s| narrow_rooms(r, 3, false, s),
        |r, s| narrow_rooms(r, 3, true, s),
    ] as [fn(Rules, u32) -> Scene; 3]
    {
        let on = play(scene(held, 7));
        let off = play(scene(Rules::Layer, 7));
        assert_eq!(on.missing, on.units, "on: none arrives {on:?}");
        assert!(on.open_give_ups > 0, "on: they give up {on:?}");
        assert_eq!(off.missing, 0, "off: every unit arrives {off:?}");
        assert_eq!(off.open_give_ups, 0, "{off:?}");
        assert_eq!(off.unsettled, 0, "{off:?}");
    }
}

// design: C7-P6 hold (the hold is bounded: without the yield no unit near gives way, and a held unit gives its move up as the original does once blocked for the first hold)
#[test]
fn hold_off_alone_gives_up_after_the_first_hold() {
    use glasnost_sim::path::HOLD_FIRST;
    use glasnost_sim::trace::{self, What};
    let mut s = narrow_rooms(off(&["jammed_move_dropped"]), 3, false, 7);
    trace::switch_on(&mut s.w);
    // Each unit's first and last held failure.
    let mut held = std::collections::BTreeMap::new();
    let mut gave_up = 0;
    for _ in 0..s.bound {
        glasnost_sim::tick::step(&mut s.w);
        for e in trace::take(&s.w) {
            match e.what {
                What::Failed {
                    gave_up: false,
                    dest_cleared: false,
                    ..
                } => held.entry(e.unit).or_insert((e.tick, e.tick)).1 = e.tick,
                What::Failed { gave_up: true, .. } => gave_up += 1,
                _ => {}
            }
        }
    }
    assert!(!held.is_empty(), "a move is held");
    for (u, (first, last)) in held {
        assert!(
            last < first + HOLD_FIRST,
            "{u:?} held from {first} to {last}"
        );
    }
    assert!(gave_up > 0, "the held moves are given up in the end");
}

// design: C7-P6 hold (only a move with units alone in its way is held: a move into a ring of buildings or onto rock fails and is given up or redirected as before)
#[test]
fn hold_off_leaves_unreachable_moves_alone() {
    let held = layer_without(&["jammed_move_dropped"]);
    for scene in [
        |r, s| walled_in(r, 8, s),
        |r, s| unreachable(r, 8, s),
        |r, s| rooms(r, 6, s),
        |r, s| corridor(r, 2, 6, s),
    ] as [fn(Rules, u32) -> Scene; 4]
    {
        assert_eq!(
            moves(&play(scene(held, 7))),
            moves(&play(scene(Rules::Layer, 7)))
        );
    }
    let (on, off) = both("jammed_move_dropped", |r, s| walled_in(r, 8, s));
    assert_eq!(moves(&on), moves(&off));
}

// design: C7-P5 yield (the giving-way state is game state: saved, loaded and hashed)
#[test]
fn a_unit_giving_way_survives_save_and_load() {
    use glasnost_sim::save;
    let mut s = rooms(Rules::Layer, 6, 7);
    let mut t = 0;
    while !s.w.objects.values().any(|o| o.path.yielding.is_some()) {
        glasnost_sim::tick::step(&mut s.w);
        t += 1;
        assert!(t < s.bound, "a unit gives way");
    }
    let bytes = save::save(&s.w);
    let mut b = save::load(&bytes, s.w.rules_arc(), s.w.catalogue.clone().unwrap()).unwrap();
    assert_eq!(save::state_hash(&s.w), save::state_hash(&b));
    for _ in 0..1500 {
        glasnost_sim::tick::step(&mut s.w);
        glasnost_sim::tick::step(&mut b);
    }
    assert_eq!(save::state_hash(&s.w), save::state_hash(&b));
    assert!(
        s.w.objects.values().all(|o| o.path.yielding.is_none()),
        "every unit resumed"
    );
}

// design: C7-P5 yield (the giving-way state is hashed)
#[test]
fn the_giving_way_state_is_hashed() {
    use glasnost_sim::path::Yield;
    use glasnost_sim::save::state_hash;
    use glasnost_sim::world::Target;
    let s = rooms(Rules::Layer, 3, 7);
    let mut b = s.w.clone();
    b.obj_mut(s.goals[0].id).unwrap().path.yielding = Some(Yield {
        resume: Target::Cell(at(0, 0)),
        to: vec![s.goals[1].id],
        since: 0,
    });
    assert_ne!(state_hash(&s.w), state_hash(&b));
}

// design: C7-P4, C7-P5 and C7-P6 (the play-fixes layer turns each of our switches off)
#[test]
fn the_layer_turns_our_switches_off() {
    let q = pathing_common::ruleset(Rules::Layer).globals.quirks;
    for (n, v) in q.named() {
        if DESIGN.contains(&n) {
            assert!(!v, "{n} is off in the layer");
        }
    }
    assert_eq!(
        q.named().iter().filter(|(n, _)| DESIGN.contains(n)).count(),
        DESIGN.len()
    );
}
