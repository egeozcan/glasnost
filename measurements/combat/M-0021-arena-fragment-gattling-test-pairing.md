---
id: M-0021
title: Unit arena, a test pairing after Q-1714's (fragments per shot, first shots, gaps, stages)
topic: combat
specs: [S-0318, S-0609, S-0301, S-0307]
requests: [Q-1714]
verdict: confirms
value-visibility: local
measured-on: 2026-11-04
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-04
approved-by:
approved-on:
---

# M-0021: Unit arena, a test pairing after Q-1714's

This record plays a two-group fight with the shape of Q-1714's pairing, built from two test vehicle
types whose fields that the per-shot and stage rows depend on are set in the test files: a *fragment
side* F, whose shots burst into fragments (S-0318), and a *gattling side* G, whose firing stages
rise while it fires (S-0609). Neither type is, or is copied from, a type of Q-1714's stock pairing.
It answers Q-1714 for this test pairing, not for the stock one. Every judged shot of F damaged its
target and 3 further G vehicles, never an F vehicle and never its own target, as S-0318 claims 1 to
3 and 5 give for a fragment count of 3 with at least 3 candidates around the target. F fired first,
each G vehicle's stage rose at the values S-0609 claims 2 and 3 give, and in part B, where it also
fell, fell at them, and the minimum gap between one vehicle's shots is the full delay's base of
S-0301 claim 2 for the firing house's rate-of-fire factor, whose value is kept local.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Harness: the oracle's unit arena. Its house director is off for every house, and so are all five
  gameplay changes of the harness that M-0001's Setup lists: a siege rule for long-range artillery,
  a change to Magnetron carrying, changes to the computer houses' production, extra defences, and an
  expansion and factory queue. The houses' own skirmish AI runs as in a normal game but has nothing
  to build with.
- Data: stock, apart from one test rules file for each part and one for the control set: a loose
  rules file put in the run's game folder, which is the stock rules file with the test sections and
  changes below. No art file is changed.
- Map: the arena map of M-0001 (`Arena.mmx`, with M-0001's sha256), used as a purpose-made game,
  not a replay of an archived one.
- Game: a skirmish at the fastest game speed. Every player starts with 0 credits, at tech level
  10, with superweapons, crates and the short game off, bases on and the map revealed. Bases on
  is the stock rules data's bases setting (S-0412 claim 1), which these runs keep, so it is no
  change from a stock skirmish.
- Houses: as M-0001's *Players* line, with the same start positions. In every run F is the
  computer house of the Americans and G that of the Russians, both at the hard computer level.
  The human player is British: an observer whose objects cannot be targeted or damaged, and who
  owns one building far from the fight, only so that it is not defeated. That building's type,
  cell, facing and mission are kept `local`. It takes no part in the fight.
- Groups: 16 vehicles a side, F's type on one side and G's on the other, on the cells and facings
  of M-0001's *Groups* line for 16 units (two columns of 8, the front columns 12 cells apart).
  A unit that its stated cell does not take is placed on a neighbouring cell (M-0006, M-0016).
  Every unit starts in the *hunt* mission. Each part is played with F on side A and with F on
  side B: *F on side A* means that F's group stands on M-0001's side-A cells, facing east, and G's
  on the side-B cells, facing west; *F on side B* is the reverse. The houses do not change with
  the side. No order is given during a run.
- Test types. Both are copies of one stock tank type (its name kept `local`), with these fields
  set in the test files:
  - both: strength (parts A and A0, and the control: 60000; part B: 400), speed 6, turn rate 8,
    no acceleration, sight 8 cells, opportunity fire off, no veteran or elite abilities, no
    debris.
  - every warhead: a copy of one stock warhead (its name kept `local`) with every armour class at
    100%, no cell spread (it damages a single object), no explosion animation, and no wall or
    wood damage. F's shot warhead and its fragment warhead are two separate test warheads.
  - every projectile: a copy of a stock invisible projectile type (S-0306 claim 9; its name kept
    `local`).
  - every weapon: a copy of one stock cannon (its name kept `local`) with its damage, rate of
    fire, range, projectile and warhead set, a weapon speed of 100, and no fire animation, report
    sound or brightness.
  - F: primary and elite primary weapon, damage 20, rate of fire 60 ticks, range 6 cells. Its
    projectile names the fragment weapon with a fragment count of 3 (part A0: its projectile
    names no fragment weapon and no count). The fragment weapon: damage 7, range 3 cells (a
    fragment range *k* of 3, S-0318 Terms), on an invisible projectile that names no fragment
    weapon. F carries it as its secondary and elite secondary weapon only so that its section is
    read (S-0004 claim 6).
  - G: two firing stages with thresholds 30 and 60 (normal and elite alike), a spin-up rate *u*
    of 1 and a spin-down rate of 1 (S-0609), and a turret count of 1. Stage 0 fires a weapon of
    damage 5, rate of fire 20 ticks, range 5 cells. Stage 1 fires a weapon of damage 10, rate of
    fire 10 ticks, range 5 cells. Normal and elite alike.
- Parts and seeds. The game seeds are 1000, 1001 and 1002.
  - part A (fragment count 3, nothing dies): seeds 1000 to 1002, F on side A and on side B, 6
    runs, each to a tick limit of 600.
  - part A0 (the control without fragments): seed 1000, F on side A and on side B, 2 runs, to the
    same limit.
  - part B (strength 400): seeds 1000 to 1002, F on side A and on side B, 6 runs, each until one
    side has no vehicle left (the tick limit of 3000 was not reached).
  - the control set: part A's files with one more change, the easy difficulty row's rate-of-fire
    factor set to 1 (a test value), seed 1000, F on side A and on side B, 2 runs to the same
    limit as part A.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - every starting vehicle and every object of the map's neutral house are removed, and the
    stated objects are created on their cells
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - the observer's objects cannot be targeted or damaged
  - the test rules files above
  - the run ends at a fixed tick limit or when one side has no vehicle left
  - a per-tick recorder reads state only

## Procedure

The recorder logs, for every run: every shot (its tick, firer, target and weapon); every hit (its
tick, the object hit, its warhead, its attacker, and the health before and after); every vehicle's
position in leptons each tick, and its cell (the position divided by 256, truncated); and each G
vehicle's spin value and stage each tick. F's two warheads tell a shot's hit from a fragment's.

1. Void check: a run in which F fires any weapon other than its primary is void.
2. For each F shot at tick *t* on target *X*, the shot's *window* runs from *t* up to, but not
   including, the tick of that firer's next shot (to the run's end for its last shot). The shot
   owns *X*'s hit by F's shot warhead, and every fragment-warhead hit in its window whose attacker
   is that firer. The windows of one firer do not overlap.
3. A shot is judged unless its target is not a G vehicle among the recorded objects at its tick,
   or (in part B) its window holds the death of any vehicle of either side.
4. For each judged shot, record the hits it owns: on which objects, how many, and how much health
   each took. From the cells read at *t*, let *c* be the number of cells of rings 1 to 3 around
   *X*'s cell (S-0303's ring metric, as S-0318 claim 3 visits them) that hold a G vehicle other
   than *X* and no F vehicle: the *candidate cells*. Also check every cell at Chebyshev distance 1
   to 3 from *X*'s cell for one that holds both an F and a G vehicle.
5. For each judged shot, compare the cells of the 3 G vehicles its fragments hit with the first 3
   candidate cells in S-0303 table T1's order, the cells read at *t*, and again with the cells read
   at *t* − 1 and at *t* + 1 (the recording gives no burst tick, and positions once a tick).
6. Who fires first: the first shot of each side, and the centre distance from its firer to its
   target at that tick.
7. Shot gaps: for each vehicle and weapon, the shortest gap between two of its shots in ticks 0 to
   299.
8. Stages: for each G vehicle, the value and stage it reads in the tick before and the tick of each
   stage change, and the largest value it reads.
9. Part B: the side left with vehicles, the deaths on each side, and, for each G vehicle, the
   tick it first reads stage 1 against the run's first death of either side.
10. For each shot, whether its target was the firer's nearest, second, third or a further enemy
    in range by centre distance at *t*, or out of range (Q-1714's question 4). This is not part of
    this record's rows (Notes).

## Result

Counts of shots are kept `local` throughout: with the published rates of fire they would bound
the houses' rate-of-fire factor (Notes).

**Objects damaged per F shot.**

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Part A: *c* in a judged shot | cells | 6 runs, every judged shot (count local) | 3 or more in every judged shot | | exact |
| Part A: hits a judged shot owns | objects | same | 1 shot-warhead hit on *X*, and 3 fragment-warhead hits on 3 distinct G vehicles other than *X* | none | exact |
| Part A: F vehicles hit by a judged shot's fragments | objects | same | 0 | none | exact |
| Part A: health one fragment hit took | health | every fragment hit owned by a judged shot | the same in every hit (value local) | none | exact |
| Part A: fragment-warhead hits whose attacker is not an F vehicle, or that name none | hits | 6 runs | 0 | | exact |
| Part A: fragment-warhead hits in a judged shot's window naming its firer as their attacker | hits | 6 runs, every judged shot | 3 | none | exact |
| Part A0: fragment-warhead hits | hits | 2 runs | 0 | | exact |
| Part A0: objects damaged by one F shot | objects | 2 runs, every shot (count local) | 1, its target | none | exact |
| Part B: hits a judged shot owns | objects | 6 runs, every judged shot (count local) | as part A, with *c* ≥ 3 in every judged shot | none | exact |
| Control: hits a judged shot owns | objects | 2 runs, every judged shot (count local) | as part A, with *c* ≥ 3 in every judged shot | none | exact |

By cells read from positions at *t*, no cell at Chebyshev distance 1 to 3 from *X* held both an F
and a G vehicle in any judged shot, so by that reading the first-listed object (S-0318 Terms) of
every candidate cell was a G vehicle, whatever the listing order. By the same reading *c* ≥ 3 in
every judged shot, so no judged shot reached S-0318 claim 4 (the random cells): this record does not
test that claim. By that reading too, in no judged shot did a cell holding only F vehicles come,
in T1's order, before the third candidate cell, so the row of F vehicles hit (0) does not test
S-0318 claim 3's alliance test.

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Judged shots whose 3 G vehicles hit stand in the first 3 candidate cells of T1's order, cells read at *t* | shots | parts A and B and the control, every judged shot (count local) | all but 13 | | not for tests |
| The same, cells read at *t* − 1, *t* or *t* + 1 | shots | same | all but 6 | | not for tests |

Of those 6 shots, 4 have, at one or more of the three readings, a candidate cell holding two G
vehicles by position, so the recording does not say which of them was listed first. The other 2
are not explained. Cells here are read from positions, which need not be the cells in which the
game lists a moving vehicle.

**First shots (parts A, A0 and B, and the control).**

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Side that fires first | side | 16 runs | F in every run | | exact |
| Centre distance at F's first shot | leptons | 16 runs | 1527 (5.96 cells), at equal heights | none | exact |
| Centre distance at G's first shot | leptons | 16 runs | 1280 (5.00 cells), at equal heights | none | exact |

Both first shots are within the firing weapon's own range (S-0307 claim 1): G's is at exactly its
range.

**Shot gaps (ticks 0 to 299).**

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Shortest gap between two shots of one vehicle, per weapon (F's primary, G's stage 0 and stage 1 weapons) | ticks | 16 runs | local | | exact |

> In every run, for each of the three weapons, the shortest gap equals the full delay's base of
> S-0301 claim 2 with its draw at 0: the weapon's rate of fire *R* times the firing house's
> rate-of-fire factor *h*, converted to whole ticks as that claim gives. *h* is that of S-0301
> claim 3 for a hard computer house of these countries: the easy row's factor (S-0412 claim 4) times
> the country's. In the control, the easy row's factor is the control file's test value. The factors
> stay local, and so does whether they tell the easy row from another row.

**Stages (S-0609, every G vehicle).**

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Value read in the tick before, and the tick of, a stage change from 0 to 1 | spin value | 16 runs, every such change | 30, then 31 | none | exact |
| Largest value read | spin value | 16 runs | 60 | none | exact |
| Value read in the tick before, and the tick of, a stage change from 1 to 0 | spin value | part B, 6 runs, every such change (parts A, A0 and the control have none) | 30, then 29 | none | exact |

**Part B (to the end).**

| Run (F's side, seed) | Side left with vehicles | F vehicles dead | G vehicles dead |
|---|---|---|---|
| A, 1000 | G | 16 | 4 |
| A, 1001 | G | 16 | 4 |
| A, 1002 | G | 16 | 7 |
| B, 1000 | G | 16 | 7 |
| B, 1001 | G | 16 | 6 |
| B, 1002 | G | 16 | 7 |

In every part B run, every G vehicle first read stage 1 before the run's first death of either
side, by 54 to 182 ticks over all 96 G vehicles.

## Evidence

- 16 recordings: 14 for parts A, A0 and B, played on 2026-11-04, and 2 for the control, played the
  same day. The recordings stay local, and no log hash is given.
- No run is void: F fired only its primary weapon in every run. No run crashed or stopped before
  its end.
- In each part A run at most 2 F shots are not judged, their target not being among the recorded
  objects at the shot's tick. Parts A0 and the control have none. In part B, shots whose window
  holds a death are not judged, and about half of each run's F shots are judged.
- The hits a judged shot owns were found as step 2 gives. In part A every fragment-warhead hit
  names an F vehicle as its attacker, and each judged window holds exactly 3 that name its firer,
  so the fragments of a shot name the firing vehicle (S-0318 claim 5). The same holds in the
  control and in part B's judged windows.

## Notes

- **Shot counts and hit values.** The number of shots in a run grows as the houses' rate-of-fire
  factor *h* shrinks, so with the published rates of fire and tick limits a count of shots would
  bound *h*, which comes from rules data. Every count of shots and of judged shots is therefore
  local. With the published damage fields, the health a hit took would bound the houses'
  firepower and armour factors (S-0302), which come from rules data too, so those values are
  local as well.
- **Not judged.** Windows holding a death are left out. Some of them own fewer than 3 fragment
  hits, and in part B some fragment-warhead hits name no attacker. Both lie only in windows with a
  death, and this record does not explain them.
- **Q-1714's question 4** (whether a group spreads its fire) was recorded per shot as the rank of
  the target among the firer's enemies in range (Procedure step 10), and is not part of this
  record's rows. S-0309 owns
  target choice.
- **Computer players.** Both houses were the original's computer players at the hard level, so
  every rule the original gives only to computer-controlled owners applied, and each house's
  difficulty row is the easy row (S-0412 claim 4). A comparison in our engine, whose AI houses
  use the normal row, sets both houses' row to the easy row through the handicap (S-0412 claim 5),
  and gives its two houses the same rules (S-0215, houses in our engine).
- **Reproducing it.** A comparison builds the two test types on a tank type of its own choice.
  The per-shot rows and the stage rows do not depend on the fields kept from the copied stock
  types in any way this record states. The first-shot distances and part B's outcome may: they
  depend on how the vehicles move, which also takes the countries' vehicle speed factors (S-0413
  claim 1), kept local. Part B's outcome also takes the houses' factors of S-0301 and S-0302.
