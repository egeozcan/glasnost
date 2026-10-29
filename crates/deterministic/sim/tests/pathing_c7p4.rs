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

/// The seeds the table is measured over.
const SEEDS: [u32; 3] = [7, 8, 9];

#[test]
#[ignore = "measurement: prints the scenario table"]
fn measure() {
    println!("{HEADER}");
    for rules in [Rules::Reference, Rules::Layer] {
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
    let mut s = all(Rules::Reference, 7)
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
