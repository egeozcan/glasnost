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

/// Our switches (C7 part 4), each defaulting to the original.
const DESIGN: [&str; 3] = [
    "path_through_stalled_blockers",
    "unreachable_move_dropped",
    "group_spread_no_detour",
];

/// The seeds the table is measured over.
const SEEDS: [u32; 3] = [7, 8, 9];

#[test]
#[ignore = "measurement: prints the scenario table"]
fn measure() {
    // The reference, the layer, and each switch of ours alone off.
    let mut sets = vec![("reference", Rules::Reference), ("layer", Rules::Layer)];
    for n in DESIGN {
        sets.push((n, off(&[n])));
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

// design: C7-P4 (the play-fixes layer turns each of our switches off)
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
