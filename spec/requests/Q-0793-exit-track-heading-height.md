---
id: Q-0793
title: Which height an exit track's heading point stores
status: answered
related: [S-0207, S-0511]
answered-by: [S-0207]
---

## Need

S-0207 claim 24 says that when an ordinary track starts, its heading point keeps the
vehicle's height at that moment. S-0511 claim 11 then uses that stored height to decide
whether a driving vehicle is heading onto a lost bridge deck. The claim covers ordinary
tracks only. It does not say what height the heading point stores for the exit track a
vehicle takes when it leaves a factory or similar building. The code in
`crates/deterministic/sim/src/drive.rs` (`begin_exit_track`) needs the rule.

## Questions

1. When an exit track starts, does its heading point store the vehicle's height at that
   moment, as for an ordinary track?
2. If not, which height does it store: the ground at the target cell, a deck height, or
   none?

## Possible observation

A factory whose exit cell is next to a high bridge deck. Destroy the deck while a newly
built vehicle is on its exit track, at different heights above the deck. Record whether
claim 11 hits the vehicle.

## Current workaround

The exit track's heading point also stores the vehicle's height when the track starts.
This choice is marked in the code with this request's ID.
