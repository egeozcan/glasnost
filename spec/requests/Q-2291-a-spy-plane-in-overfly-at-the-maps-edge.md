---
id: Q-2291
title: A spy plane in overfly at the map's edge, its heading, and the fly-back flag
status: answered
related: [S-0501, S-0513, S-0522, Q-1776]
answered-by: [S-0501]
---

## Need

M8's seam trace (ledger row "Seam (trace: M6c seed 3, CAOS–E1, quick lane seed 9 map 8)") fixed
our loss of the "entered the usable area" mark (S-0501 claim 10), and spy planes in overfly now
leave the map. It left two things open, which a second trace looked at against the approved
claims.

1. **Heading in overfly.** On a real map a spy plane in *spy-plane overfly* keeps the heading of
   its approach. It does not turn toward the exit cell that S-0501 claim 34 chooses on the edge
   opposite its house's computed edge. Read together, the claims say exactly this:
   - claim 34: overfly holds the run lock;
   - claim 31: the run lock is not cleared while the behaviour is *spy-plane overfly*;
   - claim 42: with the run lock, guidance makes no body-facing request;
   - claim 4: the body facing alone gives the direction of travel.

   So the plane flies straight on until it leaves the usable area, and the exit destination is
   never steered toward. Our engine does this. Nothing states it directly, though. AC20 records
   the exit destination but not the facing. S-0513 claim 45 says only that the route and exit are
   S-0501's. Claim 34 also replaces an absent destination with a new choice on the exit edge,
   which suggests that the destination matters for the flight.
2. **The fly-back flag at the edge.** Claim 10 says an aircraft that may leave the map now and has
   either flight flag is not removed when it leaves the usable area, only when its position leaves
   the map. Claim 4 says such an aircraft without the fly-by flag never moves into a cell that is
   not a legal map cell: it is shifted 128 leptons in x toward the middle, otherwise to a random
   point 64 leptons from there, otherwise it stays where it is. Taken together, a spy plane in
   overfly with the fly-back flag and without the fly-by flag can never be removed. With claim
   42's lack of turning, at an east or west edge it is pushed back at every step, and at a north
   or south edge it takes one raw draw per tick for the fallback point. This goes on for the rest
   of the game. Nothing says whether that is what happens. Claim 45 and S-0501's open question 18
   leave the same moment open for an aircraft in *hunt* with a flight flag ("what it does once it
   reaches the edge cell, and whether it then leaves the map"; Q-1776 questions 2 and 3).

## Questions

1. A spy plane in overfly, neither flight flag. After it passes over its target, does its
   direction of travel stay the approach heading until it is removed? Or does it turn toward a
   cell on the edge opposite its house's edge?
2. The same with the fly-back flag on and the fly-by flag off. What does it do when it reaches
   the map's edge, or the exit cell: hold or circle there, turn back across the map, slide along
   the edge, or leave? Is it ever removed, and after how long?
3. Questions 1 and 2 with the fly-by flag on, alone and together with fly-back.
4. Do the answers depend on which edge the plane meets: north, south, east or west, so the
   house's edge and the exit edge?

## Possible observation

Use a test map whose spy-plane type is first left as it is, then has fly-back on, then fly-by on,
then both. Launch the spy plane from a house whose computed edge is north, and again from one
whose edge is west. Aim at a cell in the middle of the map, so the exit edge is south or east.
Each tick from the start of overfly until 3000 ticks later, record the plane's position, body
facing, behaviour, destination and whether it still exists. Use 3 runs per case.

## Current workaround

The approved claims as written, so no change. The overfly keeps the run lock and makes no body
request, so it keeps its approach heading. A plane with neither flag is removed when it leaves the
usable area. A plane with fly-back only is pushed back at the map's edge for good. A plane with
fly-by is removed when it leaves the map.
