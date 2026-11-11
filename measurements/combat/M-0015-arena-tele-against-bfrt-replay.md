---
id: M-0015
title: Unit arena, TELE against BFRT, stock replay watched per unit
topic: combat
specs: [S-0607, S-0210]
requests: [Q-1712]
verdict: confirms
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0015: Unit arena, TELE against BFRT, stock replay watched per unit

This record watches the TELE against BFRT games of M-0001, side A, seeds 0 and 1, unit by unit:
what the Magnetrons' beams do to the fortresses, how a fortress lands after a hold, and how each
side loses units. It gives evidence on Q-1712 question 2 for these two games. The CAOS against
BFRT games of question 3 are M-0011, and question 1 is not tested here.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files. Every gameplay and AI change of the harness
  is off.
- Map, houses, budget, stated cells, facings and the *hunt* mission: as in M-0001 (the stock map
  `Arena.mmx`, start waypoint 2). TELE is the row type on side A, BFRT the column type on side B.
  Each side is a computer player at hard difficulty. The BFRT vehicles carry no passengers.
- **Replay.** Each game is the M-0001 game of the same pairing, side and seed, replayed from its
  archived setup and seed under the settings above.
- **Placement.** The units are not map entries. They are created on their stated cells once the
  scenario has loaded, before tick 0, so they are in play in the tick-0 state. A unit can be
  placed off its stated cell. Placed cells are not given here.
- The map's four bunkers, owned by a neutral house, are in play from tick 0.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - the computer players' starting construction vehicles are removed
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - a third, idle observer house is present, and its objects cannot be targeted or damaged
  - the run ends at the first of M-0001's periodic samples at which only one side still owns
    anything, or at the tick limit (M-0001). Both games here reached the tick limit and ended at
    tick 6080, M-0001's first sample at or after tick 6000. Per-tick states are recorded to tick
    6079
  - a periodic sampler and a per-tick recorder read state only. With the recorder on, the
    archived games' results, kill statistics and duel records are reproduced exactly.
- Tick convention: a per-unit state stamped tick *f* is read after tick *f*'s logic
  has run, and every damage event stamped *f* is already reflected in that tick's state.

## Procedure

1. Replay the two games, recording per tick for every unit its position, cell, height, health,
   behaviour, target and the Magnetron moving it, if any (its holder), every fire event with its
   firer and target, and every damage event with its attacker, target, warhead, distance from
   the detonation, whether it ignores defences and the health it takes.
2. Sort the Magnetrons' fire events by what each did to its target, from the holders recorded:
   - *started a hold*: the target's holder was not the firer at the end of the tick before the
     fire event, and is the firer at the end of the window. The window ends at the earlier of 10
     ticks after the fire event and the tick before that Magnetron's next fire event.
   - *met a fortress that another Magnetron already held*: otherwise, the target's holder at the
     end of the fire event's tick is another Magnetron.
   - *met a free fortress*: every other fire event. A fortress with no recorded holder counts as
     free.
   - For a fire event in tick 0, the holder at the end of the tick before is taken as none: the
     units are created without a holder before tick 0.
   - A window that would run past tick 6079, the last recorded tick, ends at tick 6079. One fire
     event has such a window (seed 1, tick 6073). It is sorted by the second test, which does not
     use the window.

   This record gives these as counts per seed, and the fortresses' fire only as a count of hits
   on Magnetrons.
3. A hold is a run of ticks in which a fortress has a holder. For each hold that ends in a
   landing, read the carry time (from the first tick with a holder to the first tick without
   one, the release), the height at the end of the release tick, the time from release to the
   first later tick at height 0 or below (the landing), and the damage events at landing.
4. From the game's own kill statistics, read which side is credited with each kill.
5. For each Magnetron that leaves play, look up the damage events it took and the units standing
   on its position in the tick before.

## Result

**End of game.**

| Seed | End | Magnetrons left | Fortresses left | Fortress deaths in the kill statistics |
|---|---|---|---|---|
| 0 | tick limit, at tick 6080 | local | local | 3, one more than the fortresses that left play |
| 1 | tick limit, at tick 6080 | local | local | 3 |

The units left in each seed are local. The winners and end ticks are those of the archived games
in M-0001.

**The Magnetrons' fire events.**

| What the fire event did | Seed 0 | Seed 1 |
|---|---|---|
| Started a hold on its target | 56 | 52 |
| Target held by another Magnetron at the end of the fire event's tick; no hold taken | 501 | 439 |
| Any other fire event (target free by that test); no hold taken | 22 | 20 |
| All fire events | 579 | 511 |

- No Magnetron made a fire event while it held a fortress.
- In each seed, the only damage events with a Magnetron as attacker are the two of the landing
  crush below, with the crushing warhead. No damage event is tied to a Magnetron's fire event.
- Every fortress was held at tick 0. Those holds were started by fire events in tick 0 and are
  among the 56 and 52.
- What made the 22 and 20 fire events at a free fortress take no hold is not recorded. Whether
  that fortress was still seized from an earlier hold (S-0607 claims 3 and 12) is not recorded:
  only the holder is.

**Holds and landings.**

| Quantity | Value | Tolerance for tests |
|---|---|---|
| Holds (one per fire event that started a hold; no hold passed from one Magnetron to another without a tick between) | below the table | exact |
| Of those, holds that end in a landing | local | exact |
| Of those, holds still running at tick 6079, the last recorded tick | local | exact |
| Time a held fortress is carried before release, in ticks | local | range only: per-hold values are not given here |
| Height above the surface at the end of the release tick, in leptons | below the table | range only |
| Time from release to landing, in ticks | local | range only |
| Damage events at a landing other than the landing crush | below the table | exact |

- What ended each hold (S-0607 claim 12) is not recorded.

Relations for the table above:

- *Holds (one per fire event that started a hold; no hold passed from one Magnetron to another
  without a tick between)*: 56 in seed 0, 52 in seed 1.
- *Height above the surface at the end of the release tick, in leptons*: local; above 0 in every
  one.
- *Damage events at a landing other than the landing crush*: 0.

**The landing crush.** In both seeds two fortresses, held by two different Magnetrons, were
released one tick apart and came down in the same tick, the landing tick.
Both left play in that tick. No other fortress left play in either seed. Below,
*later* and *earlier* name the fortress released second and first.

| Quantity | Seed 0 | Seed 1 |
|---|---|---|
| Fortresses of the crush that leave play | both | both |
| Damage events on them, all in the landing tick: crushing warhead, defences not ignored, each taking the fortress's full strength | one on each | one on each |
| Attacker of both events | the later fortress's holder | the same |
| Distance from the detonation: later fortress, earlier fortress | 3 and 8 leptons | 3 and 8 leptons |
| Damage events with the other fortress's holder as attacker | 0 | 0 |
| Fortress deaths in the kill statistics, and fortress kills credited to the TELE side | 3 and 3, each one more than the fortresses that left play | 3 and 3 |

- This fits S-0607 claims 16 to 18 if the later fortress ended its fall on the cell
  holding the other fortress, a vehicle, and so crashed. Its first impact detonation (crushing
  warhead, its dropper as source, ordinary area damage) then kills it and the other fortress. Under
  claim 18's double credit the falling fortress killed by a detonation is credited twice and the
  other fortress once: one credit more than the fortresses that left play.
- No damage event with the demolition warhead (claim 18's second detonation) was recorded. Both
  fortresses had no health left after the crushing-warhead events.
- The damage record holds no other event on either fortress. Not recorded: which of the two
  reached the ground first within the landing tick, and the flight state of the earlier fortress.
  The fall of that fortress left no damage event with its holder as attacker. Not checked: the
  amount against S-0607 F2.

**How the Magnetrons leave play.**

| Quantity | Seed 0 | Seed 1 |
|---|---|---|
| Hits on Magnetrons by fortress fire | 15 | 18 |
| Magnetrons destroyed by that fire | 0 | 0 |
| Magnetrons that leave play | local | local |
| Of those, with no damage event, above half strength, while a grounded fortress that had the Magnetron as its target stood within 0.04 cells of its position in the tick before | all of them | all of them |
| Distance of that fortress from the Magnetron's position in that tick | 0.02 to 0.04 cells (both seeds) | |
| Magnetrons the kill statistics credit to the BFRT side | as many as left play | the same |

The removals with no damage event are consistent with crushing under S-0210 claim 4. The kill
statistics credit a side, not a unit, so they do not name the crusher.

## Evidence

- 2 games (seeds 0 and 1, side A), per-tick records kept by the operator. No recording or log hash
  was supplied.
- The winners and end ticks match the archived M-0001 games. With the recorder on, their results,
  kill statistics and duel records are reproduced exactly (Setup). The units left are local.

## Notes

- **One fortress death more than removals.** The kill statistics count credits: in each seed they
  give one fortress death, and one TELE kill, more than the fortresses that left play. This
  matches S-0607 claim 18's double credit, as set out under the landing crush above.
- **Not reported:** the fortresses' fire events other than their hits on Magnetrons, the damage
  events' amounts (only the health each takes), what ended each hold, and the cells of the
  removals.
- **Q-1712 question 2.** In these two games the fortresses are lost only at the landing crush,
  and never from the beams themselves or from an ordinary landing. The Magnetrons are
  lost only by removals with no damage event while a fortress that targeted them stands within
  0.04 cells of them.
- **Q-1712 question 1** (an empty transport with no weapon of its own) is not tested: these
  fortresses fire at the Magnetrons.
