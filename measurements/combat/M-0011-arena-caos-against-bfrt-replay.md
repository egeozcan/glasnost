---
id: M-0011
title: Unit arena, CAOS against BFRT, stock replay watched per unit
topic: combat
specs: [S-0302, S-0210]
requests: [Q-1712]
verdict: confirms
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-28
---

# M-0011: Unit arena, CAOS against BFRT, stock replay watched per unit

This record watches the CAOS against BFRT games of M-0001, side A, seeds 0 and 1, unit by unit:
what the chaos drones' gas does to the fortresses, what the fortresses do, and how each drone
leaves play. It gives evidence on Q-1712 question 3 for eight of the nine drones in each seed.
The TELE against BFRT games of Q-1712 question 2 are not part of this record.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files. Every gameplay and AI change of the harness
  is off.
- Map, houses, budget, stated cells, facings and the *hunt* mission: as in M-0001 (the stock map
  `Arena.mmx`, start waypoint 2). CAOS is the row type on side A, BFRT the column type on side B.
  Each side is a computer player at hard difficulty. The BFRT vehicles carry no passengers.
- **Replay.** Each game is the M-0001 game of the same pairing, side and seed, replayed from its
  archived setup and seed under the settings above.
- **Placement.** The units are not map entries. They are created on their stated cells once the
  scenario has loaded, before tick 0, so they are in play in the tick-0 state. When the game does
  not accept a unit on its stated cell, the harness places it on a free cell nearby.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - the computer players' starting construction vehicles are removed
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - a third, idle observer house is present, and its objects cannot be targeted or damaged
  - the run ends at the first of M-0001's periodic samples at which only one side still owns
    anything, or at the tick limit (M-0001). Both games here ended that way, at the end ticks below
  - a periodic sampler and a per-tick recorder read state only. With the recorder on, the
    archived games' results, kill statistics and duel records are reproduced exactly.
- Tick convention: as in M-0001. A per-unit state stamped tick *f* is read after tick *f*'s logic
  has run, and every damage event stamped *f* is already reflected in that tick's state.

## Procedure

1. Replay the two games, recording per tick for every unit its position, cell, height, health,
   behaviour, target, berserk state and remaining berserk time, every fire event with its firer
   and target, and every damage event with its attacker, target, warhead, distance from the
   detonation, amount after the damage calculation and the health it takes.
   This record gives the fortresses' fire events as counts and the gas damage events as counts,
   ranges and selected ticks. Per-unit ticks and the drones' own fire events are recorded but not
   given here.
2. From the game's own kill statistics, read which side is credited with each kill.
3. For each drone that leaves play, look up the damage events it took and the units standing on
   its position in the tick before.

## Result

**End of game.**

| Seed | Winner | End tick | Drones left | Fortresses left |
|---|---|---|---|---|
| 0 | BFRT | 608 | 0 of 9 | 3 of 4 |
| 1 | BFRT | 912 | 0 of 9 | 3 of 4 |

The end tick is the tick at which the run stopped. The last drone is in play at the end of tick
492 (seed 0) and of tick 740 (seed 1), so each run stopped at the next of M-0001's samples, every
304 ticks, as M-0001's rule gives. The end ticks, and the column side's share left (0.75), are
those of the archived games in M-0001.

**Gas on the fortresses.**

| Quantity | Seed 0 | Seed 1 | Tolerance for tests |
|---|---|---|---|
| Tick at which the first gas reaches each fortress and makes it berserk | 50 to 58 (both seeds together) | | range only: per-unit ticks are not given here |
| Gas damage events on fortresses | 140 (1 with no attacker) | 120 (3 with no attacker) | exact |
| Remaining berserk time at the end of the tick of the first gas on a fortress, in ticks (the same for all 8 fortresses) | local | local | exact |
| Later gas damage events on fortresses that leave the remaining time at that same full value | 133 of 136 | 116 of 116 | exact |
| Fortresses that leave the berserk state before the end | 0 | 2 (at ticks 433 and 662) | exact |
| Tick at which each of those is made berserk again by gas | | 520 and 704 | exact |
| Health taken by any gas damage event | 0 | 0 | exact |

- "Gas" here means the damage events with the warhead of the drones' attack, including those
  with no attacker. Whether each came from a gas particle or from the weapon's own detonation is
  not recorded. No tick holds two gas events on the same fortress.
- The remaining berserk time is read from the per-tick record at the end of the tick of each gas
  damage event on a fortress. In all 260 events it equals that event's amount after the damage
  calculation, so no countdown update was seen in a gas event's tick here.
- That amount is the full value in 257 events, at distances from 0 to 3 cells from the
  detonation. In the other 3, all later events on one fortress in seed 0, at 1 to 1.4 cells, it is
  lower. What lowers those 3 is not identified here.
- Each of those 3 sets the remaining time to its own amount: one shortens it, two lengthen it.
  So a further berserk hit sets the remaining time anew, longer or shorter, as S-0302 claim 10
  has.
- With no gas event in a tick, the remaining time falls by 1 at the end of the tick, except at 5
  ticks of seed 1 (one fortress at 4, another at 1), at which it does not change. What holds it
  at those ticks is not identified here.
- Each of the two fortresses that left the berserk state did so at the end of the tick at which
  its remaining time reached 0.
- S-0302 claim 10 takes the berserk time at distance 0, which fits the full value at every
  recorded distance. As the cause of the 3 lower amounts is not shown, claim 10's formula is not
  tested further here.
- Gas damage events also land on the drones' own side. They take no health there either.

**What the berserk fortresses do.**

| Quantity | Seed 0 | Seed 1 |
|---|---|---|
| Fire events of one fortress at another fortress | 198 | 275 |
| Fortresses destroyed by that fire | 1 | 1 |

**How the drones leave play.**

| Quantity | Seed 0 | Seed 1 |
|---|---|---|
| Drones that leave play | 9 | 9 |
| Of those, with no damage event, while a grounded fortress that had the drone as its target stood on the drone's position in the tick before | 8 | 8 |
| Of those, after one damage event with the warhead named "Super" and no attacker, which takes the drone's full health | 1 | 1 |
| Drones the kill statistics credit to the BFRT side | 9 | 9 |

The eight removals per seed with no damage event are consistent with crushing under S-0210
claim 4. The kill statistics credit a side, not a unit, so they do not name the crusher, and the
record does not show when in a cell each removal came. The remaining removal in each seed is
unexplained. The fortresses' own fire goes at each other while they are berserk.

## Evidence

- 2 games (seeds 0 and 1, side A), per-tick records kept by the operator. No recording or log hash
  was supplied.
- The end ticks and the surviving shares match the archived M-0001 games. With the recorder on,
  their results, kill statistics and duel records are reproduced exactly (Setup).

## Notes

- **The last drone.** In each seed, one drone leaves play after a damage event with the warhead
  named "Super" and no attacker, which takes its full health, yet the kill statistics still
  credit it to the BFRT side. What deals that event is not identified here.
- **Not reported:** which drone each gas damage event comes from, the drones' own fire events,
  and the cells of the removals.
- **Q-1712 question 1** (an empty transport with no weapon of its own) is not tested: these
  fortresses fire, at each other, while berserk.
- **Q-1712 question 2** (TELE against BFRT) is not part of this record.
