# M-0021 as revised for Q-2536 and Q-2546: claim ledger, M8 step A3 intake 97

M-0021 (revised in c286959) answers **Q-2536** and **Q-2546**; both requests are at status
answered. The change is in the arena tool only (`crates/tools/src/oracle_m0021.rs`,
`docs/development.md`): no simulation rule, hashed state, format or golden moved (save 98, AI game
format 61, wire 12 as on main). The simulation crates are untouched, so the seed sweeps, the AI
lanes and the quick lane are skipped; the arena `--record all` is run before and after.

## What the record changes

| Request question | the record's answer | change |
|---|---|---|
| Q-2536 1: distances before F's first shot, and whether the target is in range one tick before | the target is within F's range (6 cells) by the rows stamped *t* − 1 and *t*, and F does not fire in the tick before; G's target is outside G's range by the row stamped *t* − 1 and within it by *t*. The distances at the other ticks stay local | the range rows (`Shooter::rows`; verdict rows *F's target within F's range …*, *G's …*); the local report holds the three readings per shooter |
| Q-2536 2: closing speed | not given: it would bound the countries' speed factors (S-0413 claim 1) | none |
| Q-2536 3: what depends on the seed | the first difference between the seeds is the tick of some G vehicles' second shots, after each side's first shots and before any death, in the same tick in parts A and B on both sides; part B's deaths differ later. The Notes name the full delay's one ranged draw (S-0301 claim 2) as the likely source, and say the recordings do not show what else acts later | `compare_seeds`, `SeedGroup`, `seed_verdicts` (seven rows) |
| Q-2546 1: where the positions are read | per-tick rows stamped *t* are read after tick *t* (the convention of M-0004, M-0011 and M-0015); the first-shot distance is computed from the positions in the shot's own row | `ShotRec::firer_pos` and `target_pos` (derived, below); the first-shot distance rows read them |
| Q-2546 2: whether G took its turns before F | the Notes infer (not identified by the recordings) that the shot rows' positions fit F's vehicles taking their turns before G's in every tick (S-0203 claim 3 with the creation order of the Setup), each vehicle moved before it fired in its own turn; ours has that order | none to the simulation; the four shot reading-point rows |
| Q-2546 3: which tick | the shot's own tick; F's 1527 is read at the shot | the first-shot rows |

## The shot rows' positions: how they are derived

Our shot event carries no positions and the tool cannot stop inside a tick, so `shot_positions`
derives them from state, once per shot, never from the shot's group:

- the firer: its position after the tick. A vehicle's firing step follows its movement in its own
  turn and nothing moves it afterwards (S-0310 claim 9; S-0215 claim 11, the late start after the
  update's movement and before the separate firing; `tick::object_turn`);
- the target: an object moves only in its own turn (our engine's tick structure,
  `tick::object_turn`, not a spec claim of its own). When the target's turn comes before the
  firer's in the update order read at the start of the tick (`World::update.order`, S-0203 claim
  3), it has moved when the firer fires, so its position after the tick; otherwise its position
  before the tick.

Consequences, stated plainly: the four shot reading-point rows and G's *per-tick distance equals
the shot row's* row follow from the update order and the movement-before-firing rule, so they pass
by construction when F's group is created first. They test our order, not an independent capture.
The check that does not depend on it: every shot the engine fired passed its own range test at the
positions of the shot, so the derived distance must be within the weapon's range. The report
shows whether every derived distance is, and, of the four combinations of rows (firer at *t* − 1
or *t*, target at *t* − 1 or *t*), which ones also have every shot within range. A direct capture
(positions on the shot event) would need a field on `Effect::Shot`, a change in the simulation
crate; it is left as an option.

## Built (all in `crates/tools/src/oracle_m0021.rs`)

- **Recording.** `Recording::rows` holds each tick's position and health of every vehicle that is
  still an object of the world, dead or alive, beside `positions` (vehicles in play), which the
  earlier rows keep reading. `ShotRec` carries `firer_pos` and `target_pos`. The recorder also
  reads the update order and the positions before each tick.
- **Judging.** `Findings::first` now holds each shooter of a side's first-shot tick with its
  distance from the shot row's positions and from the rows stamped *t* − 1 and *t*;
  `Findings::own_first` the distance of every vehicle's own first shot at the reading points;
  `Findings::reading` the reading points (hits, destroyed vehicles, the four shot rows);
  `Findings::reach` our check of the derivation.
- **Seeds.** `compare_seeds` compares the runs of one part and side tick by tick (sorted shots,
  hits, and rows with position, health, spin and stage), the vehicles matched by their place in the
  creation order; `seed_verdicts` gives the record's three seed rows and the four prose claims
  around them.
- **Rows.** The two `our check` rows are shown (not counted in *rows failing N of M*); they are not
  rows of the record. The first difference's tick, the gaps between the seeds' shots and every
  count of shots are printed only in the git-ignored report.

## M-0021 Result rows on the install

The harness judges 42 rows and 3 fail (before this change: 24 judged, 2 failing). Counts of shots, the first
difference's tick and the other local values are in the git-ignored report only.

| Row | status | note |
|---|---|---|
| void check; part A, A0, B and control per-shot rows (13 rows) | pass | as before |
| the 3 G vehicles hit stand in the first 3 candidate cells (not for tests) | shown | all but 12 at *t*, all but 3 at *t* − 1 to *t* + 1; the record: all but 13 and all but 6 |
| first shots: F fires first | pass | |
| **first shots: F's centre distance from the shot row's positions, 1527** | **pass** | was 1504 (read after the tick); read at the shot, as the record defines it (derived, above), ours gives the record's value |
| first shots: G's centre distance from the shot row's positions, 1280 | pass | |
| first shots: F's target in range by the rows stamped *t* − 1 and *t*: yes, yes | pass | |
| first shots: G's target in range by the rows stamped *t* − 1 and *t*: no, yes | pass | |
| first shots: G's per-tick distance stamped *t* is 1280, equal to the shot row's | pass | by construction under the derivation above |
| first shots: every vehicle's own first shot is within range at the reading points | pass | |
| reading points: the row that first shows a hit's health after is *t*, never *t* − 1 | pass | a vehicle hit twice in a tick shows only the last hit's health after |
| **reading points: a destroyed vehicle's last row is stamped *t*, with no health left** | **fail** | ours removes a destroyed vehicle at the hit (S-0203 claim 6 (a), S-0312 claim 15), so it has no row stamped *t*: its last row is *t* − 1, with health left; seam, Q-2596 |
| reading points: F shot's firer *t*; F shot's target *t* − 1; G shot's firer *t*; G shot's target *t* (4 rows) | pass | by construction under the derivation above |
| shot gaps (3 rows), stages (3 rows) | pass | as before |
| part B: the side left, F and G dead per run | **fail** | as before: ours 4, 4, 4 and 7, 7, 7 G dead against the record's 4, 4, 7 and 7, 6, 7; seam, Q-2598 |
| part B: every G vehicle reads stage 1 before the first death; the lead's range | pass; shown | ours 55 to 180, the record 54 to 182 |
| seeds: the three seeds of a part and side differ | pass | |
| seeds: the first difference is the tick of some G vehicles' second shots | pass | the shots that differ are G vehicles' second shots, each fired a tick or two later in the other seeds by the same vehicle at the same target |
| seeds: a shot fired in one seed only is fired later in the others by the same vehicle at the same target | pass | |
| seeds: the first difference falls in the same tick in parts A and B on both sides | pass | |
| seeds: after each side's first shots and before any death | pass | |
| seeds: part B's deaths differ between the seeds, after the first difference | pass | |
| **seeds: every vehicle's first shot falls before the first difference** | **fail** | the strict reading of "including every vehicle's first shot": every F vehicle has fired by the first difference, but only half of the G vehicles; Q-2597 |
| seeds: every vehicle's first shot is the same in the three seeds | pass | the other reading of that sentence |

The harness counts 42 rows; one of them, the other reading of the record's sentence on first
shots, is a supplementary row of ours. So 41 rows are the record's, 3 of them failing.

Our checks, shown and not counted (they are not rows of the record): the derived centre distance is
within the weapon's range for every shot but one, which is beyond *R* by 0.8 lepton (not traced;
S-0307 measures with its table square root and adds *E*₁, which may account for it), and no shot
lacks a derived position; no pure reading of the rows (firer and target each at *t* − 1 or *t*)
has every shot within range.

## Seeds and part B

Our seeds split in the way the record's rows describe: at some G vehicles' second shots, which
take one ranged draw from the scenario stream (S-0301 claim 2), after each side's first shots and
before any death, in the same tick in both parts and on both sides (only the kind and the order
can be compared: the record keeps its tick local). The first hit that differs is in that tick. The
deaths differ between the seeds from the first death on, later than the first difference, as the
record says. Part B's G-dead count does not change with the seed within a side
in ours, where the record's does. Nothing in the record says which later draw or choice sets who
dies, so no claim is named: the seam stays with S-0301 claim 2's draw as the one known source, and
Q-2598 asks for the order of deaths and where the seeds' orders split.

## Seams left

1. The destroyed vehicle's row stamped *t* (S-0203 claim 6 (a), S-0312 claim 15; Q-2596). The
   harness does not build a row from the hit's health after, since that would copy the record's
   answer.
2. Part B's G-dead count per seed (S-0301 claim 2's draw and later, unnamed, draws or choices;
   Q-2598).
3. The strict reading of "every vehicle's first shot" (Q-2597).
4. The shot rows' positions are derived, not captured. A capture needs fields on `Effect::Shot`
   (a change in the simulation crate, not hashed).

## Requests

**Q-2596** (a destroyed vehicle's row at *t*), **Q-2597** (every vehicle's first shot and the first
difference), **Q-2598** (part B's order of deaths), filed in `spec/requests/` in a commit of their own.

## Checks and lanes

CHECKS_PLACEHOLDER
