---
id: Q-2072
title: Which spawn slots show a full pip, and in which order occupant pips fill
status: open
related: [S-0111, S-0506, S-0508]
answered-by: []
---

## Need

S-0111 claim 11's spawned-aircraft row shows image frame 1 "for each aircraft currently
docked or ready". S-0506 claim 4 names seven slot states (docked, on the rail, preparing,
attacking, returning, reloading, empty), and "ready" is not one of them. The same claim's
occupant row shows "one per slot: the occupying infantry type's occupant pip image frame",
without saying which occupant takes which pip when occupants of different types show different
frames. The live view's pip rows (`crates/app/src/mapview/live.rs`, `Live::pip_rows`) need
both answers.

## Questions

1. For each slot state of S-0506 claim 4, does the slot's pip show image frame 1 or 0? In
   particular: a slot whose aircraft has come back and is reloading; a missile slot whose
   missile is preparing on its launcher; a slot whose aircraft is on its way back.
2. Does a slot's pip stand at the slot's own index (so a launched slot leaves a gap between
   full pips), or do the full pips come first, packed from pip 0?
3. A building holding occupants of two infantry types with different occupant pip image
   frames: does pip 0 show the occupant that has been inside longest, the most recent one, or
   some other order? Does a pip move when an earlier occupant leaves?

## Possible observation

S-0506's R2 and R5 setups, recording the spawn pips while one aircraft attacks, returns and
reloads, and a missile spawner while its missile prepares. For occupants, a building with two
slots filled by two infantry types of different `OccupyPip` values, in both entry orders, then
the first one leaving.

## Current workaround

A slot in the docked or the reloading state (the two in which its aircraft is inside the
spawner) shows image frame 1, every other state 0, each at its own slot index. Occupant pip
*k* shows occupant *k* in S-0508's occupant order (the one inside longest first), the empty
slots after them. Both are marked `design:` with this request's ID.
