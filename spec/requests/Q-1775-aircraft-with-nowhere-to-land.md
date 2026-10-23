---
id: Q-1775
title: The lead's play report: an aircraft with nowhere to land crashes
status: answered
related: [S-0501, Q-1756, Q-1755]
answered-by: [S-0501]
---

## Need

This follows Q-1756. The project lead reports from memory of play of the original game (M8 step
A4, 2026-10-22), with low certainty in their own words ("not 100% sure"): an armed aircraft that
returns from an attack and has nowhere to land crashes.

The approved specs give no such outcome that we have found. S-0501 table T2 and claim 33 have an
aircraft with no dock types hold above its last firing spot (Q-1756 asks whether it lands there),
and claims 15 to 18 send an aircraft with dock types home when a pad grants contact. In our engine
the computer player's aircraft, after attacking the lead's base, circled over their target; the
lead had not built that aircraft type. A crash would explain why the lead does not remember
circling aircraft.

## Questions

1. An armed aircraft that must return to a pad (it has dock types) finishes its run, and its owner
   has no building of any of its dock types on the map. What does it do: wait in the air (where,
   and for how long), land on open ground, or crash (take damage or be destroyed; after how many
   ticks; by which damage route, so that kills, losses and score count as they should)?
2. The same when the owner has such pads but every docking place is taken (S-0501's pad contact
   refused for every pad). Does it wait, circle, land elsewhere or crash, and does that change
   when a pad frees up?
3. The same for an armed aircraft with no dock types at all (S-0501 T2's guard row): does the
   crash rule, if any, apply to it, or does Q-1756's outcome apply?
4. Does a pad destroyed or sold while the aircraft flies to it, or while it is docked, lead to the
   same outcome?
5. Does the owner being human or computer-controlled change any of the above?

## Possible observation

A test map: a house with one armed, landable aircraft type that has dock types and one pad of that
type. Order an attack on a durable enemy vehicle, then sell or destroy the pad before the aircraft
returns. Record the aircraft's behaviour, destination, position, height, health and whether it
exists, every tick until 2000 ticks after its last round, 5 runs. Repeat with the pad's docking
places all taken by other aircraft, and with no pad from the start (the aircraft placed by the map).

## Current workaround

The approved claims as they stand: no crash. An aircraft with no pad to go to holds or circles as
S-0501 and Q-1756's workaround say. The lead's report is recorded in
docs/m8-claims/a4-play-notes.md. No change until this is answered.
