---
id: Q-1244
title: When a particle system's lifetime counts down and when an ended system is removed
status: open
related: [S-0613, S-0203]
answered-by: []
---

## Need

S-0613 claim 6: "The system lifetime goes down by 1 each turn. A system whose lifetime reaches
exactly 0 is marked to end, and a marked system with no particles left is removed. A lifetime of
−1 never reaches 0." Claim 1 adds that a shared gas system with a positive lifetime is marked to
end after that many turns. Three things are not stated:

- whether the count-down and the removal test come before or after the particle updates and
  moves of the same turn (this decides by one tick when a system disappears, and whether a
  system whose last particle dies in a turn is removed in that turn or the next);
- what a lifetime of 0, or a negative lifetime other than −1, does;
- what a virus death does once the shared gas system has been removed (claims 2 and 3 put
  their clouds into it).

The code is `crates/deterministic/sim/src/gas.rs` (`system_turn`, `shared`).

## Questions

1. In a gas system's turn, is the lifetime counted down and the removal tested before or after
   the particle update and movement passes?
2. Does a system created with lifetime 0, or with a negative lifetime other than −1, ever end?
3. After a shared gas system with a positive lifetime has been removed, does a virus death create
   no clouds, create a new shared system, or something else?

## Possible observation

G3's finite-lifetime variant: record the tick of the system's disappearance relative to its last
particle's removal tick, with lifetimes 1, 0 and −2. G1-missing's section variant with a short
positive lifetime gives the third case if a listed control type is used.

## Current workaround

The count-down and the removal test come at the end of the system's turn, after both passes.
Only a positive lifetime counts down, so 0 and negative lifetimes never end. With no shared
system in play a virus death still takes its animation's draws but creates no particles, and a
note says so. Docs: docs/development.md, *Simulation (M7a: gas clouds)*.
