---
id: Q-1942
title: When claim 40 replaces the low release's move destination at the move's first processing, after it has reached height 0, does claim 8 make it take off?
status: open
related: [S-0501]
answered-by: []
---

## Need

S-0501 revision 59, claim 48 (c), *At height 0*, says of the low-released aircraft that
"Claim 8's take-off steering applies only during a take-off, which it does not make." It then
follows the run along the released facing at height 0.

That holds when the move order's cell is kept. Claim 40 runs at the first processing of every
ordinary *move* and keeps the destination only when F7 accepts its cell. Otherwise it seeks a
replacement cell around it. A person can give the low release a move order to a cell that F7
refuses, such as a cell of its own pad's foundation one cell from the docking cell.

The order is given while the aircraft is still at its release height, above the terrain. Its first
movement update can bring it to height 0 (claim 4's descent) before the move's first processing.
Claim 40's replacement then gives the aircraft a new destination while it is at height 0, which is
on the ground (Terms). Claim 8 says: "An aircraft on the ground (or landing) that gets a
destination takes off: its held height becomes its cruise height and it climbs by claim 2."

Read together as written, the low release takes off at that point, with claim 8's take-off
steering. That contradicts "a take-off, which it does not make" in claim 48 (c). The engine has to
choose one. The choice changes the held height, the climb and the body facing from that update on,
which are part of the state.

A local-lane run with a pad whose docking position is 10 leptons up in a foundation cell shows the
case. A move order to a cell one cell ahead of the docking cell (inside the foundation) gets a
replacement cell at the move's first processing, one update after the low release's first
movement update. The aircraft then takes off and turns toward the replacement. A move order to a
cell three cells ahead (outside the foundation, inside the slowdown distance) is kept: the aircraft
stays at height 0 unturned and falls back to rest in that cell, as claim 48 (c) describes.

## Questions

1. Is a destination that claim 40 replaces at the move's first processing a destination that the
   aircraft "gets" in claim 8's sense?
2. If so, does claim 8 apply to the low release once it is at height 0, so that it takes off at the
   replacement (and claim 48 (c)'s "which it does not make" covers only a kept destination)?
3. If not, does the low release go on at height 0 toward the replacement cell's centre, as
   *At height 0* describes for a kept destination?
4. Does the answer change when the replacement is the same cell as the order's (claim 40 keeps
   it), or when claim 40's first processing comes before the first movement update brings the
   aircraft to height 0?

## Possible observation

A pad whose docking position is a small height (10 leptons) above level ground, in a cell of its
foundation, with a pad-bound aircraft type that has a long slowdown distance. Release the aircraft
low, then give a move order to a cell of the pad's own foundation next to the docking cell. Log
height, held height and body facing for each update. A climb with a turn toward a nearby cell
outside the foundation answers questions 1 and 2 with yes. A run at height 0 along the released
facing answers question 3 with yes.

## Current workaround

Claims 40 and 8 as written: the replacement is a destination the aircraft gets, and the low release
on the ground at height 0 takes off at it. Claim 48 (c)'s *At height 0* applies to a kept
destination. This is our design until this is answered. No code change was needed for it: the sim
already applies claim 8 when a destination arrives on the ground.
The local lane's `low-release-ahead` and `low-release-abeam` scenes in
`crates/app/tests/m6c_lane.rs` use destinations outside the foundation, so they show the kept case.
