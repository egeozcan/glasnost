---
id: M-0020
title: Unit arena, YURI against GGI and SHK and CAOS against E1 (placement at tick 0, standing positions and the winner, first targets, captures, the drones' end)
topic: simulation
specs: [S-0216, S-0208, S-0202, S-0506, S-0402]
requests: [Q-2277, Q-2286]
verdict: new-finding
value-visibility: local
measured-on: 2026-11-02
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0020: Unit arena, YURI against GGI and SHK and CAOS against E1

This record has two parts.

- **Part 1** replays six stock games of the unit arena at seed 0, with every scenario-stream draw
  logged: M-0002's YURI against GGI and YURI against SHK, and M-0001's CAOS against E1, each with
  the row type on side A and on side B. It reads where the infantry stood in their cells at
  tick 0 and which draws came before tick 0's update. It also gives, as local values, each
  side's first targets, the first mind-control capture, and, for CAOS against E1, when the last
  drone was gone and how many E1 were berserk then.
- **Part 2** plays YURI against GGI and YURI against SHK, YURI on side A, over 12 seeds, once as
  set up and once with every side-A YURI moved, before tick 0, to one named standing position of
  its cell. It compares the winners.

In part 1's six games every placed infantry unit stood at a right, left or bottom position at tick
0, and the draws before tick 0's update end in a run that fits the draws S-0216 claim 3 gives the
set-up's objects, in identifier order. In part 2, side A won all 144 games, in every arm of both
pairings: moving the side-A YURI between the centre and the other standing positions of their cells
did not change the winner at any of the 12 seeds.

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files.
- Map, houses and settings, as in M-0001 to M-0003: the stock map `Arena.mmx` around its start
  waypoint 2; two computer houses, side A Americans at start 0 and side B Russians at start 1,
  difficulty hard, no starting construction vehicle, no buildings and 0 credits; an idle observer
  human at start 3. Game speed fastest, tech level 10, superweapons off, crates off, short game
  off. Budget, stated cells, facings and the *hunt* mission are those of M-0001.
- The house director is off for every house, and the five gameplay changes of the harness named
  in M-0017 (a siege rule for long-range artillery, a change to Magnetron carrying, changes to the
  computer houses' production, extra defences, an expansion and factory queue) are all off.
- What is not stock, in every run of this record:
  - no menus: the skirmish setup comes from a settings file.
  - the computer players' starting construction vehicles are removed.
  - the two computer players are not allied with each other.
  - the idle observer house's objects cannot be targeted or damaged.
  - units are created on their stated cells before tick 0, and a unit that its stated cell does
    not take is placed on a neighbouring cell, which may already hold a unit (M-0006). M-0001's
    "Infantry also take one cell each" therefore does not hold for every cell of the archived
    games (M-0006).
  - the map's neutral bunkers are in play from tick 0, as in M-0001.
  - the run ends by M-0001's rule: at the first sample (every 304 ticks) at which only one side
    still owns anything, or at the first sample at or after tick 6000.
  - a recorder reads state only (below).
  - in part 2's moved arms only: every side-A YURI is put at one named standing position of its
    cell before tick 0 (below).
- The recorder gives, until the run ends:
  - for every object, after each tick's update: its identifier, type, owning house, whether it
    is on the map, its position in leptons, its health, its target, whether it is berserk, and
    whether it is under mind control.
  - every shot (firer, target, tick).
  - every damage event (attacker, victim, tick, the victim's health before and after).
  - every scenario-stream draw from the stream's seeding: its tick, whether it falls inside a
    tick's update, whether it is unranged or ranged (and the range), its result, and the number
    of raw values (S-0202) taken since the seeding, before and after it.
- Part 1 games: seed 0 of M-0001's naming, each replayed once from its archived setup and seed.

  | Game | Record | Row type on side |
  |---|---|---|
  | YURI against GGI, A | M-0002 | A |
  | YURI against GGI, B | M-0002 | B |
  | YURI against SHK, A | M-0002 | A |
  | YURI against SHK, B | M-0002 | B |
  | CAOS against E1, A | M-0001 | A |
  | CAOS against E1, B | M-0001 | B |

- Part 2 games: the archived side-A set-ups of YURI against GGI and YURI against SHK (YURI on
  side A), at seeds 0 to 11 of M-0001's naming. Seeds 0 to 5 are the archived games' seeds,
  replayed with the five changes off as in M-0017; seeds 6 to 11 are new, with the same set-up.
  Six arms per pairing and seed:
  - **S:** the set-up as is.
  - **C, T, R, L, B:** the same, except that before tick 0's update every side-A YURI stands at
    the centre, top, right, left or bottom position (S-0208 table P) of the cell it stands on.

## Procedure

**Part 1.**

1. Replay the six games. Compare each result (end tick and winner) with its archived game, and
   each shot and damage event with those recorded during M-0017's replay of the same game (the
   harness's recorder ran during those replays; M-0017 does not publish its recordings, Evidence).
2. At tick 0, after its update, take every infantry unit of the two players' houses that is on
   the map. Its cell is position ÷ 256, truncated, and its position in the cell is position
   mod 256, named by S-0208 table P.
3. Take the scenario-stream draws from the seeding with the game's seed to the start of tick 0's
   update, and the raw values (S-0202) they took.
4. Take the set-up's objects (M-0019's term: the objects of the two houses, on the map or off it, of
   a type one of the set-up's unit entries names) present at tick 0, in identifier order. Form the
   pattern of draws S-0216 claim 3 gives them, with the type data of the stock rules the games
   loaded (local): for each object its creation draw (S-0202 claim 10), the creation draws of the
   objects its creation makes (S-0506 claim 1, S-0402 claim 19), then its placement draw if claim 3
   gives it one; an infantry object's is R(0, 3). An object off the map at tick 0 is counted as
   S-0216 claim 8 counts a refused try, as in M-0019. Compare the pattern, draw by draw in kind and
   range, with the draws before tick 0's update, ending at the last of them.
5. Where step 4 fits, number the R(0, 3) draws of the run 1, 2, … in order, and the infantry objects
   among the set-up's objects 1, 2, … in identifier order. Take the result of the *n*-th R(0, 3)
   draw as *k* for the *n*-th infantry object. Go through the placed infantry in identifier order.
   For each, the expected position is the first position of row R*k* of S-0208 table O that no
   lower-identifier infantry unit holds on the same cell at tick 0. Compare it with the unit's
   position at tick 0.
6. For each house, take the first tick after whose update any of its units holds a target, and
   the kind of each target then. Take each house's first shot, the first YURI shot, and the first
   tick after whose update any unit is under mind control, with each unit under mind control then:
   its group at tick 0 and its owning house at that tick. Targets and mind control are read from
   the object state after each tick's update, so a target held only within one update is not
   seen.
7. In the CAOS games, take the first tick after tick 0 at which no CAOS is on the map with health
   above 0. At that tick, count the E1 on the map, the berserk ones among them, and the E1 that
   were berserk at any tick up to then. A kill is a damage event that leaves its victim's health
   at 0. Count the CAOS kills by the attacker's kind, and the E1 kills up to that tick.

**Part 2.**

8. Run the six arms at the 12 seeds of both pairings.
9. For every game, take the winner and the end tick. For the moved arms, check that every side-A
   YURI stands at the arm's position at tick 0, and that the draws before tick 0's update equal
   arm S's at the same seed.
10. Controls: arm S's results at seeds 0 to 5 against the archived games. A same-spot run per
    pairing at seed 0, which moves every side-A YURI to the position it already stands at,
    against part 1's replay of the same game, on object state, shots, damage and the draws after
    the seeding. And, for every moved game, whether its damage
    events differ from those of arm S's game at the same seed.

## Result

**Part 1: replays.**

| Quantity | Games | Value |
|---|---|---|
| Result equal to the archived game's | 6 | below the table |
| Shot and damage events identical to those recorded during M-0017's replay of the game | 6 | below the table |

Relations for the table above:

- *Result equal to the archived game's*: 6.
- *Shot and damage events identical to those recorded during M-0017's replay of the game*: 6.

**Part 1: standing positions at tick 0 (Q-2277 q1).**

| Quantity | Games | Value |
|---|---|---|
| Placed infantry exactly at one of the five standing-position points of its cell | 6 | all |
| Placed infantry at the centre or top position | 6 | none |
| Placed infantry at the right, left or bottom position | 6 | all |
| Placed infantry at the position Procedure 5 expects | 6 | all |

- Whether any of the six games has a placed cell holding more than one infantry unit, and the
  positions the units on such a cell took, are local (`measurements/local/M-0020.json`,
  `shared-cells`). Stated here, they would bound the named groups' sizes (M-0006).
- The positions are read after tick 0's update. That every unit stood exactly at a
  standing-position point is consistent with none having moved in it. A move during tick 0's
  update is not excluded by the recording itself.

**Part 1: draws before tick 0 (Q-2277 q2).**

| Quantity | Games | Value |
|---|---|---|
| Procedure 4's pattern equal to the last draws before tick 0's update | 6 | below the table |
| Draws of that run each taking exactly one raw value | 6 | all |
| Draws after that run and before tick 0's update | 6 | none |
| Raw values taken from the seeding to the start of that run | 6 | below the table |

- The recording does not tie a draw to an object. What fits is the sequence: in identifier
  order, the set-up's objects present at tick 0, those off the map included, match the run draw
  by draw. Procedure 5's positions follow from the run's R(0, 3) results.
- The 706 raw values before the run are the same in all six games, whichever pairing and side,
  and equal M-0019's *N*₀ for seed 0. They count raw values, not draws: the scenario-stream log gives the raw
  values each draw takes, and some of the draws before the run take more than one, so fewer than
  706 draws come before it.

Relations for the table above:

- *Procedure 4's pattern equal to the last draws before tick 0's update*: 6.
- *Raw values taken from the seeding to the start of that run*: 706 in each.

**Part 1: first targets, shots, captures, the drones' end.** Every value is local
(`measurements/local/M-0020.json`): per game, the first tick at which each house's units hold
targets and the kind of target then, each house's first shot tick and its target's kind, the
first YURI shot tick, the first mind-control capture tick, and in the CAOS games the first tick
after tick 0 at which no CAOS is on the map with health above 0, the E1 counts at that tick (on the map, berserk, berserk at any tick up
to then), the CAOS kills by attacker kind, and the E1 kills.

- In all four YURI games every unit under mind control at the first such tick is of the group
  opposing the YURI, and is then owned by the YURI's house.

**Part 2: standing positions and the winner (Q-2277 q3).**

| Pairing | Arm | Games | Won by side A (YURI) | Won by side B | No winner |
|---|---|---|---|---|---|
| YURI against GGI | S | 12 | 12 | 0 | 0 |
| YURI against GGI | C | 12 | 12 | 0 | 0 |
| YURI against GGI | T | 12 | 12 | 0 | 0 |
| YURI against GGI | R | 12 | 12 | 0 | 0 |
| YURI against GGI | L | 12 | 12 | 0 | 0 |
| YURI against GGI | B | 12 | 12 | 0 | 0 |
| YURI against SHK | S | 12 | 12 | 0 | 0 |
| YURI against SHK | C | 12 | 12 | 0 | 0 |
| YURI against SHK | T | 12 | 12 | 0 | 0 |
| YURI against SHK | R | 12 | 12 | 0 | 0 |
| YURI against SHK | L | 12 | 12 | 0 | 0 |
| YURI against SHK | B | 12 | 12 | 0 | 0 |

| Check | Games | Value |
|---|---|---|
| Moved games with every side-A YURI at the arm's position at tick 0 | 120 | below the table |
| Moved games whose draws before tick 0's update equal arm S's at the same seed | 120 | below the table |
| Arm S games at seeds 0 to 5 with the archived game's result | 12 | below the table |
| Same-spot runs identical to part 1's replay of the same game (object state, shots, damage, draws after the seeding) | 2 | below the table |
| Moved games whose damage events differ from arm S's game at the same seed | 120 | below the table |

Relations for the table above:

- *Moved games with every side-A YURI at the arm's position at tick 0*: 120.
- *Moved games whose draws before tick 0's update equal arm S's at the same seed*: 120.
- *Arm S games at seeds 0 to 5 with the archived game's result*: 12.
- *Same-spot runs identical to part 1's replay of the same game (object state, shots, damage, draws
  after the seeding)*: 2.
- *Moved games whose damage events differ from arm S's game at the same seed*: 120.

## Evidence

- Recordings (local): part 1's six replays, part 2's 144 games and its two same-spot runs. Per
  run: per-tick object state, shots, damage and the scenario-stream log.
- The shot and damage events of M-0017's replays of the same six games: the recordings made
  during those replays (2026-10-28), kept local. M-0017 compares only its sampled columns and the
  result, and does not use them.
- Counts and ticks: `measurements/local/M-0020.json`.

## Notes

- Every moved game played out differently from the unmoved game at the same seed, so the
  standing position does change the course of these games. Only the winner is compared here.
  End ticks and first-capture ticks per game are local (`measurements/local/M-0020.json`).
- Part 2 covers YURI on side A only, these two pairings and seeds 0 to 11. It does not show
  whether the standing position changes the winner with YURI on side B, in other pairings, or
  with mixed positions within one group.
- In the moved arms every side-A YURI of a group stands at the same position of its cell. That
  is a set-up of the harness, not a stock placement. In part 1's games no placed infantry stood
  at the centre or top position.
