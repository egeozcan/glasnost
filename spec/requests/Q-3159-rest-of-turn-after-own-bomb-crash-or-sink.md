---
id: Q-3159
title: What the rest of its turn does for a carrier that its own bomb kills and that starts to crash or sink
status: open
related: [S-0519, S-0203, S-0312, S-0501, S-0502, S-0504]
answered-by: []
---

## Need
S-0519 claim 4 checks a planted bomb in its carrier's own turn, after the behaviour processing and
before the carrier's movement. A carrier that the detonation kills and that leaves play at once has no
more turn. One that stays in play only plays out its death (S-0203 claim 6 (c)): an infantry unit in
its death sequence takes no more of its living turn (issue #384).

Two dead carriers keep moving in later turns: a crashing aircraft or jumpjet keeps its flying or
jumpjet movement until it hits the ground (S-0501 claim 9, S-0502 claim 18, S-0312 claim 16), and a
vessel that sinks descends in each of its own updates (S-0312 claims 15 and 32, S-0504 claim 4). No
claim says what the rest of the turn in which the bomb kills them does: whether the carrier's movement
of that turn still runs (the first crash or sinking step coming in that turn), or the turn ends at the
detonation and the crash or sinking first moves in the next turn; and whether the later parts of a
living turn (the spawner's launch pass, the cloak update, the late start, firing) run in it.

## Questions
1. A flying aircraft or a jumpjet in the air that its own bomb kills and that starts to crash: does
   its crash movement take a step in the turn of the detonation, after the bomb check?
2. A vessel that its own bomb kills and that starts to sink: does it descend in that turn?
3. In either case, do any other parts of the living turn after the bomb check run in that turn?

## Possible observation
S-0519 scenario with a bomb planted on a flying aircraft, a jumpjet in the air and a ship, each with
strength below the bomb damage: record the carrier's height or position in the tick of the detonation
and the tick after it.

## Current workaround
A carrier that its own bomb kills ends its turn at the bomb check unless it is then crashing (aircraft
or jumpjet) or sinking; those go on with the rest of the turn as before, their movement included
(`object_turn`, `crates/deterministic/sim/src/tick.rs`).
