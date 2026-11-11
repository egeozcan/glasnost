---
id: M-0005
title: Hunters against disabled robot tanks in the unit arena (the LTNK against ROBO stall)
topic: combat
specs: [S-0405, S-0503, S-0319]
requests: [Q-1710]
verdict: contradicts
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0005: Hunters against disabled robot tanks in the unit arena (the LTNK against ROBO stall)

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, no loose rules or art files.
- Games: the twelve LTNK against ROBO games of M-0001 (seeds 0 to 5, the LTNK group on side A in
  games A0 to A5 and on side B in games B0 to B5), replayed from M-0001's archived setup and seed.
  Map, players and game settings (M-0001's *Map* and *Players* lines), the budget, stated cells
  and the *hunt* mission are M-0001's (start waypoint 2, cell W).
- Harness: the oracle's unit arena. Its house director is off for every house. The houses' own
  skirmish AI runs as in a normal game but has nothing to build with.
- **The five gameplay changes.** M-0001's archived games ran with the five gameplay changes of
  the harness that its Setup lists: a siege rule for long-range artillery, a change to Magnetron
  carrying, changes to the computer houses' production, extra defences, and an expansion and
  factory queue. This record's replays have all five off and are the stock reference. In M-0006's
  replays the twelve LTNK against ROBO games are identical tick for tick with and without the
  five changes.
- The recorder's object list gives every unit and building in play in every recorded state from
  tick 0. No robot control building is in that list at any time.
- The map's four bunker structures owned by a neutral house are in play from tick 0 (cells 36,80,
  36,85, 126,78 and 126,87). In some timeout games, during the stall, hunters' shells damage a
  bunker (Result).
- In the archived games' logs, a unit that its stated cell did not take was placed on a
  neighbouring cell (M-0006). This record does not compare its replays' tick-0 cells with those
  logs.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - units are created before tick 0, on their stated cells by the harness's account as M-0001
    lists it, which this record neither takes nor observes
  - the computer players' starting construction vehicles are removed
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - the idle observer house's objects cannot be targeted or damaged
  - the run ends by M-0001's rule: at the first sample (every 304 ticks) at which only one side
    still owns anything, or at the first sample at or after tick 6000
  - a periodic sampler, a per-tick recorder and, in the per-shot run, a detonation recorder read
    state only. With the recorders on, the archived games' results (M-0001's per-game rows) are
    reproduced exactly, and the per-shot run's unit states, damage events, fire events and
    per-tick random state are identical to the first stock replay's
- Tick convention: "tick *f*" is the game's frame counter, and a per-unit state stamped *f* is read
  after tick *f*'s logic has run. The convention is inferred: 188 damage events were checked and
  each is already reflected in its stamped tick's state, but they are not every damage event of
  the twelve games, and how they were chosen is not recorded. Every statement below about the
  state of the tick before a fire tick, or of the fire tick, depends on this inferred
  convention. So do the statements that the tick-0 set is disabled in the tick-0 state (Result,
  Notes, Q-1710 questions part (3)): they say each such tank is disabled once tick 0's logic has
  run, not that it was disabled before.

## Procedure

1. Replay each game to its end, recording every tick: for each unit its position in leptons,
   cell, height above the ground, health, mission, target and disabled state; every fire event,
   with the firer's position and its target's position at that tick and whether it made a
   projectile; and every damage event (target, attacker, warhead, raw damage, position).
2. Replay the twelve games again (the per-shot run), recording as well, for every projectile of
   an LTNK, its launch tick and position, its aim point and, at its detonation, the tick, its
   position, its target (an object, or a cell) and that target's position. Join each projectile
   to its fire event and to its detonation.
3. Count per robot tank: its first recorded disabled tick, its first change of cell, whether it is
   ever hit, and whether it is in play at the end. Count per hunter after its game's last hunter
   hit: its fire events, the damage events with it as attacker, its target, its distance to it and
   its cell.
4. From the damage events of step 1, list those whose target is a structure, with the structure,
   the attacker and the tick.

## Result

Terms used below: a robot tank *is hit* when it takes any damage event, with or without an
attacker (so B3's attacker-less damage counts). A *hunter hit* is a damage event on a robot tank
with a hunter as its attacker. The *stall* is the part of a timeout game after its last hunter hit.
A unit's *start cell* is its cell in the tick-0 record, and the group's start cells are its units'
start cells.

The robot tanks fall into two sets by their recorded state: the *tick-0 set*, disabled in the
tick-0 state, and the *later set*, enabled at tick 0. How the sets lie in the group's layout is
local. The LTNK group hunts with every unit. Counts of robot tanks, of hunters, of projectiles and
of shell groups are local, except where a row or note gives one.

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Games ending at the tick limit (A0, A2, A5, B0, B2, B3, B5) | games | 12 | below the table | each game's result equals the archived one | exact per seed |
| Games won by the LTNK group | games | 12 | below the table |  | exact per seed |
| Tick-0 set: never changing cell | robot tanks | every tank of the set, 12 games | all | | exact |
| Tick-0 set: hit robot tanks destroyed; robot tanks in play at the end | robot tanks | 12 games | all hit ones but one destroyed; some in play at the end, all but one of them never hit | counts local | exact per seed |
| Later set: rising to hover height while enabled | robot tanks | local | all | height local | local |
| Later set: robot tanks never disabled | robot tanks | every tank of the set, 12 games | below the table |  | exact |
| Later set: first recorded disabled tick | tick | local | local | counts local | ±0 per seed |
| Later set: disabled before its first change of cell | robot tanks | local | all | the delay from disabling to the first change local; some never change cell (count local) | exact |
| Later set: hit robot tanks destroyed; robot tanks in play at the end | robot tanks | 12 games | all hit ones but two destroyed; some in play at the end | counts local | exact per seed |
| Later set, once disabled: height above the ground from a local tick on (the tick each one reached 0 is not reported) | leptons | local | below the table |  | exact |
| The game's last hunter hit, per game | tick | 12 | below the table | ticks local | ±0 per seed |
| Hunters' projectiles in the stall, per timeout game | projectiles | 7 | many in every game | counts local | exact per seed |
| Hunters' projectiles in the stall that damage a neutral bunker, all aimed at a robot tank | projectiles | 7 | some in some games (which games is local); in the others no damage event with a hunter as attacker | counts and ticks local | exact per seed |
| Every hunter at the stall: mission, target | | every hunter, 7 games | *hunt*, a robot tank | | exact |
| Shelling hunters at the stall (every hunter but those of the next rows): distance to its target (horizontal, between the two units' recorded positions) | cells | 7 | local | | local |
| Shelling hunter at the stall: interval between its projectiles | ticks | 7 | local | | local |
| Hunters that make a fire attempt every tick and no projectile, from a tick in 410 to 549 to the end | hunters | 7 | below the table |  | exact per seed |
| Such a hunter: distance to its target | leptons | every such hunter | local | | local |
| Hunters changing cell after tick 700 in a timeout game | hunters | 7 | below the table |  | exact |

- **Relations (values in the local file).** The later set's hover height while enabled is the
  stock hover height reached by a lifting hovering vehicle. Each survivor of A0 at full health is
  at the type's full strength, and its one damaged survivor is below it. A shelling hunter's
  recorded distance to its target at the stall (horizontal, between the two units' recorded
  positions) is within its weapon's range. Its interval between projectiles agrees with S-0301
  Formula F1: its weapon's rate of fire times the firing house's rate-of-fire factor, plus the 0
  to 2 tick delay draw, rounded and truncated as F1 gives. Each relation was checked against the
  local values.
- **Hunters firing without a projectile.** Each such hunter made its last projectile in ticks 274 to
  466. Another hunter of B5, not one of these, does the same for 55 ticks only (425 to 479), and
  then makes projectiles again: at the stall it is one of the shelling hunters. Why no projectile is
  made is not recorded.
- **A0 survivors.** All are disabled and at height 0. One is damaged, and every other survivor is
  at full strength. Their number and cells are local. At least one of them stands on a cell outside
  the group's start cells, so it moved there.
- **Attacker-less damage (B3).** One survivor in B3 takes two damage events with no attacker, at two
  ticks kept `local`, each landing a local distance from it, which take it from full strength in two
  small steps (warhead, raw damage and health local). **Relation (values local):** their warhead and
  raw damage are those of S-0319's expiry area damage for the sprite debris that destroyed vehicles
  throw (not S-0317's pieces with voxel models), so they fit that damage, which has no source
  object. Which destruction threw the debris is not reported.

Relations for the table above:

- *Games ending at the tick limit (A0, A2, A5, B0, B2, B3, B5)*: 7.
- *Games won by the LTNK group*: 5.
- *Later set: robot tanks never disabled*: 0.
- *Later set, once disabled: height above the ground from a local tick on (the tick each one reached
  0 is not reported)*: 0.
- *The game's last hunter hit, per game*: before tick 700 in every game.
- *Hunters that make a fire attempt every tick and no projectile, from a tick in 410 to 549 to the
  end*: in A5, B3 and B5 (counts local); none in the other games.
- *Hunters changing cell after tick 700 in a timeout game*: 0 (checked every tick).

**Per-shot run.** Every LTNK projectile detonated except those still in flight at the tick limit
(counts local). The rows below count the projectiles that detonated. A projectile *hits* when its
hunter makes a hunter hit in its detonation tick. That ties a damage event to one projectile only if
the hunter has no other projectile detonating in that tick. Whether that ever happens is not
reported. So *r*, the projectiles matched to a robot tank and the detonations within *r* that do not
damage it, below, rest on this hit rule: they hold only as far as no hunter had two projectiles
detonating in one tick. For every projectile, the robot tank its shot was fired at is disabled and
at height 0 in the state recorded for the tick before the fire tick, and also in the state recorded
for the fire tick for every projectile but those fired at the tick limit, after the last recorded
state. The fire event falls between those two states.

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Launch point: a fixed distance from the hunter's position at its fire event (its bearing from there local) | leptons | every detonated projectile | local | | local |
| Aim point: the target's position at its fire event (for these robot tanks its cell centre at ground height) | projectiles | every detonated projectile | all | | exact |
| Projectiles whose target at detonation is a cell, not the robot tank fired at | projectiles | every detonated projectile | some (count local), all before the stall | | exact |
| Detonation point: the projectile's own position at that tick | projectiles | every detonated projectile | all | | exact |
| Flight, launch to detonation | ticks | every detonated projectile | local | local | local |
| Groups of shells by hunter, launch point and the robot tank fired at (its fire event's target) | groups | 12 games | local | | exact |
| Groups that detonate exactly at the target's position every time (horizontal position, compared with the target's position at detonation) | groups | every group | most | count local | exact |
| Groups that never do | groups | every group | below the table | counts and shells per point local | exact |
| Mixed groups | groups | every group | below the table | shell counts local | exact |
| Projectiles matched to a robot tank | projectiles | the detonated projectiles that the hit rule matches to a robot tank (every other detonated projectile, the stall shells among them, damages no robot tank by that rule) | most detonate exactly at their target's position, the rest within *r* of the robot tank matched to them by the hit rule (not always their target) | counts local | exact, by the hit rule |
| Detonations within *r* of a robot tank that do not damage it | detonations | every detonated projectile | below the table |  | exact, by the hit rule |
| Stall shells (detonated in the stall) | projectiles | 7 games | all from groups that never detonate at their target's position | count local | exact |
| Stall shell: distance from its target | leptons | every stall shell | local, more than *r*, beyond the target along the line of fire and to either side (how far local) | | local |
| Stall shell: nearest detonation to any robot tank | leptons | every stall shell | local, more than *r* | | local |

- *r* is the largest recorded distance from a detonation to a robot tank it damaged by the hit
  rule (the robot tank's position in the detonation tick). Which distance the oracle computed
  for *r* and for the rows within *r* (horizontal or spatial, and any rounding) is not recorded.
  **Relation (value local):** *r* is at most the spread radius of S-0303 claim 3 for the LTNK
  weapon's warhead; compared with the local values, it equals it.
- "The target's position" in the group rows is the target's position at detonation, as the
  detonation record gives it: the robot tank's, or for those aimed at a cell the cell's. A
  detonation counts as at it when the horizontal positions are equal.
- Neither the range from the launch point to the aim point nor the bearing from the hunter's
  position to the aim point separates groups that detonate at the target from groups that never
  do: the two kinds' ranges overlap, and both kinds have bearings all round. Nor does the
  target's state: direct hits, early misses and stall shells were all fired at robot tanks
  disabled and at height 0 in the states recorded around the fire tick (above).
- In some timeout games the fixed detonation points of hunters' shells (counts local) lie next to a
  neutral bunker (cells 127,78 to 127,79 by the bunker at 126,78, and 127,87 to 127,88 by the one at
  126,87). Their shells are the bunker damage of the table above, matched as hits are: a
  projectile is counted as damaging a bunker when its hunter makes a damage event on that bunker
  in the projectile's detonation tick. That holds only as far as no hunter had two projectiles
  detonating in one tick, as for the hit rule (Per-shot run). Which bunker per game is not
  reported.

Relations for the table above:

- *Groups that never do*: all the rest but one; each puts its shells on one ground point or on 2 or
  3 points.
- *Mixed groups*: 1, with shells of both kinds.
- Detonations within *r* of a robot tank that do not damage it: 1, beside a robot tank destroyed by
  another hunter's shell in the same tick.

## Evidence

Twelve games, every tick recorded, and the same twelve again in the per-shot run. The recordings
stay local, and no recording or log hash was supplied with the results. Each replay's result
equals the archived game's (M-0001's per-game rows).

## Notes

- **Verdict.** *contradicts* refers to committed S-0405 revision 36, claim 13 only (below).
  S-0503 claim 15 and S-0319 are cited as consistent with the record, and the targeting part of
  S-0405 claim 20 is confirmed.
- **Disabling of a placed hovering vehicle (contradicts committed S-0405 revision 36, claim 13).**
  No robot tank's recorded disabling follows a change of cell: the tick-0 set is disabled in the
  tick-0 state and the later set while still in its start cell, before any change of cell. This fits
  the hovering locomotion's cell-arrival update at the vehicle's actual position when it chooses a
  heading point (S-0503 claim 15): an inference, not a recorded event. Committed S-0405 revision
  36, claim 13, says such a vehicle "stays enabled, and can act normally, until its first
  completed arrival in a cell without a building", where "An arrival is the end of a step of its
  own movement into the cell". S-0405
  claim 13 as revised states the inferred update.
- **Who survives.** In the tick-0 set, whether a robot tank is in play at the end depends almost
  entirely on whether it was ever hit: all hit ones but one were destroyed, and every survivor but
  one was never hit. In the later set all hit robot tanks but two were destroyed. How many of the
  later set's survivors were never hit is not reported.
- **The stall is a selection.** Every stall shell lay farther than *r* from every robot tank, and by
  the hit rule, of all detonations within *r* of a robot tank only one did not damage it (beside a
  robot tank destroyed in the same tick), with the attribution qualification above. The per-shot
  groups have the outcomes tabulated above: every shell at the target's position, none (on one point
  or on 2 or 3), or a mix. Every stall shell comes from a group that never detonates at its target's
  position and lands outside every robot tank's damage reach. The misses continue to the tick limit,
  and no hunter changes cell after tick 700. What sets where a group's shells land is not recorded:
  it needs the projectile's position at every tick of its flight. The hunters that fire without a
  projectile add no shells at all.
- **S-0405 claim 20.** The record confirms its targeting part: hunters pick disabled robot tanks of
  the tick-0 set and keep disabled robot tanks as targets. It also shows disabled robot tanks taking
  damage and being destroyed: the tick-0 set, disabled in the tick-0 state and never changing cell,
  has all its hit robot tanks but one destroyed, and the per-shot run's direct hits were all fired
  at disabled robot tanks. It does not compare the damage a disabled and an enabled robot tank take.
  The stall's shells detonate outside every robot tank's damage reach. Whether an enabled target
  would change that is not separated by these games.
- **Q-1710 questions.** (1) Hunters pick and keep disabled robot tanks as targets, as S-0405 claim
  20 states: every projectile is fired at one (above). (2) At the stall, in each of the 7 timeout
  games, every hunter is in *hunt* with a robot tank as its target, so none is idle. The shelling
  hunters' targets are disabled in the states recorded around each fire tick. The state of the
  targets of hunters that make no projectile is not recorded at the stall. Each hunter shells
  fixed points that miss every robot tank, or makes a fire attempt every tick with no projectile
  (the table above). The record does not say which hunters had destroyed an earlier target. (3)
  Every placed robot tank is disabled, the tick-0 set in the tick-0 state and the later set while
  still in its start cell. Whether a still-enabled robot tank fares differently as a target is not
  separated by these games.
