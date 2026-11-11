---
id: M-0025
title: The scatter from damage for a hunting vehicle hit by its own house, an allied house or an enemy it cannot harm (S-0215 B6 part g)
topic: simulation
specs: [S-0215]
requests: [Q-2782]
verdict: confirms
value-visibility: committed
measured-on: 2026-11-07
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0025: The scatter from damage for a hunting vehicle hit by its own house, an allied house or an enemy it cannot harm

This record places a test vehicle *G* of a computer-controlled house in *hunt*, with nothing on the
map that it can attack, and lets one firer's area shots reach it. The rules file's *hunt* section
has the scatter property and the retaliate property both on. The firer belongs to *G*'s own house
(part g1), to a house allied with *G*'s (part g2) or to an enemy house whose firer *G*'s weapon
cannot harm (part g3). In every run, every judged hit (Procedure, step 1) left *G* at the end of the hit's tick with a
destination in one of the 8 cells next to its own, no target, and still in *hunt*. In part g3, *G*
held no target in any row. This is what S-0215 claim 19 case 2 gives for an actor whose current
behaviour has the scatter property and whose owner is computer-controlled, hit by an attacker it may
not retaliate against (claim 18).

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Harness, map, game settings and what is not stock: as M-0021's *Setup*, apart from the houses,
  the units and the test rules file below. No art file is changed.
- Houses: three computer houses, all at the hard computer level:
  - *A*, the Americans, which owns *G*,
  - *B*, the French, on *A*'s team, so *A* and *B* are allied,
  - *C*, the Russians, on another team, so *C* is an enemy of *A* and *B*.

  *A* and *B* stand at the start positions of M-0001's two computer houses and *C* at another of
  the map's start positions. The human player is M-0021's observer, with its one building far from
  the fight. Each house's starting construction vehicle is handled as in M-0021.
- Test types, built on M-0021's test chassis (M-0021, *Setup*, "both" line, apart from strength,
  and apart from sight for *G*), with copied warheads, projectiles and weapons made as M-0021's:
  - *G*: strength 1000, sight 3 cells. Its weapon: damage 5, rate of fire 20 ticks, range 3 cells.
    Its warhead has no cell spread, 100% against every armour class except one of the two special
    armour classes, against which it has 0%. That class is given below to *Q* and to *C*'s firer,
    and to no other object in these runs, so *G* cannot attack either of them.
  - *Q*: strength 60000, no weapon, that special armour class.
  - *firer*: strength 1000. Its weapon: damage 50, rate of fire 150 ticks, range 8 cells. Its
    warhead has a cell spread of 1.5 cells, full damage across the spread, and 100% against every
    armour class. *C*'s firer has *Q*'s special armour class, and the firers of *A* and *B* keep
    the chassis's class.
- Test rules file: the types above, a *hunt* section with the scatter property on and the retaliate
  property on, its other fields left as stock, and the player-scatter setting off.
- Units, in creation order: *G* of *A*; *Q* of *C*, on the cell next to *G*'s; one firer; the
  observer's building. Every unit starts in *hunt*, and no order is given during a run.
  - *part g1*: a firer of *A*, 7 cells from *Q*. Its only enemy object it can attack is *Q*, and
    its shots at *Q* reach *G*.
  - *part g2*: a firer of *B*, 7 cells from *Q*, likewise.
  - *part g3*: a firer of *C*, 7 cells from *Q*. Its shots go to *G*.

  The cells and facings are not reported.
- Runs: seeds 1000, 1001 and 1002 for each part, 9 runs, each to a tick limit of 600.

## Procedure

The recorder logs, for every unit each tick, its position, health, current behaviour, target (or
none) and destination cell (or none), and every hit with its victim and the attacker's house (or no
attacker). A unit's cell is its position in leptons divided by 256, truncated, per axis. The rows
stamped *t* are read after tick *t* has run, so they show the state at the end of tick *t*,
including the hits recorded at *t*.

1. A *judged hit* is a hit on *G* at tick *t* where *G*'s row stamped *t* − 1 shows *hunt*, no
   target and no destination, and *G* is on the map with health left at the end of *t*. Several
   hits in one tick count as one. A hit with no row at *t* − 1 or *t* − 2 is not judged.
2. For each judged hit, record whether *G*'s position at the end of *t* − 2 equals its position at
   the end of *t* − 1, and the house of the attacker.
3. Read *G*'s row stamped *t*: its target, its behaviour, and its destination's Chebyshev distance
   from its cell at the end of *t* − 1.
4. In part g3, check *G*'s target in every row.

## Result

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Judged hits whose attacker is of *A* (g1), *B* (g2), *C* (g3) | hits | each part's 3 runs, every judged hit | all | none | exact |
| Judged hits leaving *G* with a destination in a cell next to its own, no target, in *hunt* | hits | 9 runs, every judged hit (count not reported) | all | none | exact |
| Judged hits whose *G* position changed from the end of *t* − 2 to the end of *t* − 1 | hits | same | 0 | none | exact |
| Runs with at least one judged hit | runs | 9 | 9 | none | exact |
| Part g3: rows in which *G* holds a target | rows | 3 runs, every row | 0 | none | exact |

Each run's first hit on *G* comes in the run's first tick, with no earlier row, and is not judged.

## Evidence

- 9 recordings, played on 2026-11-07. The recordings stay local, and no log hash is given.
- No run is void. No run crashed or stopped before its tick limit.

## Notes

- **What the verdict covers.** S-0215 claim 19, case 2, for a driving vehicle of a computer-
  controlled house in a behaviour with the scatter property, hit by:
  - its own house's or an allied house's attacker (g1, g2), which it may not retaliate against,
  - an enemy attacker its weapon cannot harm (g3).

  In these runs, such a vehicle, with neither a target nor a destination, standing still and surviving the hit,
  holds a destination in a cell next to its own and no target at the end of the hit's tick.
- **Not tested here:**
  - an enemy attacker that *G* could harm, in or out of its range (not set up),
  - which of the vehicle's acceptance conditions (S-0207 claim 16) held, and which cell next to
    its own it takes, and why,
  - case 1, human-controlled owners and the player-scatter setting,
  - the scatter property off: M-0024's control covers that for enemy hits,
  - what *G* does after it reaches the destination cell.
- **"Standing still"** is read from positions over the two ticks before the hit, not from the
  vehicle's locomotor state.
- **Not reported:** the number of judged hits per run, the ticks of the hits, the cells, facings and
  start positions, and how long *G* takes to reach the destination cell. The hit counts and ticks
  follow from the firer's test weapon and the units' movement, which takes the countries' speed
  factors (M-0021, *Notes*). This record states only the rows above.
- **Reproducing it.** A comparison builds the test types on a tank type of its own choice, as
  M-0021's *Reproducing it* note gives. 
