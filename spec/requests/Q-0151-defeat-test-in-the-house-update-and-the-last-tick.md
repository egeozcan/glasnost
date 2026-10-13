---
id: Q-0151
title: Where the defeat test sits in a house update, which tick is the last, and whether removed houses are contenders
status: answered
related: [S-0414, S-0203, S-0405, S-0410, S-0407, S-0009]
answered-by: [S-0414]
---

## Need

S-0414 claim 1 runs the defeat test "during its own house update", and claim 11 runs a surrender's
destruction "in its next house update". S-0203 row 7 holds several topics in one house update:
power (S-0405), the radar recheck (S-0410), the tech-tree recheck (S-0407) and the sidebar's
orders (S-0411). No spec says where the surrender's destruction and the defeat test sit among
them. The destruction makes the house's power and radar due. Whether those topics see it in the
same update or the next one decides when they change.

Claim 8 says the game "is over in tick *t* + max(*D*, 1)", so at least one more tick runs. It
does not say whether tick *t* + max(*D*, 1) itself runs in full, or whether a defeat in that tick
still counts as a defeat during the end delay (claim 9).

S-0414 defines a contender as an undefeated house that is not passive. A house that S-0009's
unused-house cleanup removed (claim 28) owns nothing. If it counted as a contender, it would fail
in tick 1, and while undefeated it would count in the end test.

The engine needs this in `econ::houses::house_update`, `econ::victory::house_update`,
`econ::victory::outcome` and `econ::victory::contender`.

## Questions

1. Within a house's update, do the surrender's destruction and the defeat test come before or
   after the power, radar and tech-tree work of the same update?
2. Does tick *t* + max(*D*, 1) run in full, with its defeats counting under claim 9, before the
   game is over? Or is the game over at its start?
3. Is a house that the unused-house cleanup removed a contender (tested in tick 1, counted in the
   end test and in claim 12's enemy count), or no house of the game at all?

## Possible observation

1. Defeat a house that owns a radar building and a power plant, and record its radar availability
   and power balance from the debugger tick by tick.
2. Use a test rules file with an end delay of a few ticks, and have the last enemy house's
   remaining unit die by a timed effect in tick *t* + *D*. Check whether the result changes.
3. Start a skirmish in which the cleanup removes a non-passive computer house, and check whether a
   defeat message for it appears in tick 1.

## Current workaround

1. The destruction and the test come first in the house update, before power, radar and the tech
   tree.
2. Tick *t* + max(*D*, 1) runs in full and its defeats count under claim 9. `victory::outcome`
   reports the game as over once that tick has run, and no defeat test runs after it.
3. A removed house is no contender: it is never tested, never counted and gets no result.

Listed in `docs/development.md`'s provisional table.
