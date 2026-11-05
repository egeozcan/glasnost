# M-0020, M-0021 and M-0019 as revised: claim ledger, M8 step A3 intake 101

This intake takes three records:

- **M-0020** (new, ee3d044). It answers **Q-2277** and **Q-2286**.
- **M-0021**, revised in f70ce6b to 2ee8f56. It answers **Q-2596** and **Q-2598**. **Q-2597** is still
  open, and its row is unchanged.
- **M-0019**, revised in 0c0ba94. It adds *Other houses at tick 0* and Procedure 8.

The change is in the arena tool only:

- `crates/tools/src/oracle.rs`, `oracle_m0020.rs` (new) and `oracle_m0021.rs`;
- `crates/tools/tests/arena.rs`;
- `docs/development.md`, *Arena mode*.

No simulation rule, hashed state, format or golden moved: save 102, AI game 65 and wire 12, as on
main. The simulation crates are untouched, so the seed sweeps, the AI lanes and the quick lane are
skipped. The arena `--record all` was run before and after, with `--threads 2`.

- `ours-M-0001.csv` to `ours-M-0003.csv` are byte-identical before and after.
- `oracle-M-0001.txt` to `oracle-M-0003.txt` are byte-identical too.
- None of the 155 pairings moved. The same 13 scored pairings fail as on main:
  - ROBO–HTK;
  - TELE–APOC, TELE–V3, TELE–ROBO, TELE–BFRT;
  - CAOS–E1, CAOS–APOC, CAOS–TTNK, CAOS–E2;
  - DISK–FV, INIT–E2, MIND–E1, YURI–SHK.
- `oracle-M-0021.txt` differs because of its new and split rows.
- `oracle-M-0019.txt` and `oracle-M-0020.txt` are new.

Values the records keep local are printed only in those git-ignored reports. That covers ticks,
the stock map's counts, positions per house and shared cells. They are not written here.

## Tick numbering

The records' tick *n* is our step run while the tick number is *n*. S-0201 claim 1: the tick
number is 0 at game start and grows by 1 at the end of every tick. So *after tick 0's update* is
our state after the game's first step. *The next tick* is our state after the second step. An effect
seen after a step belongs to the tick just run. The M-0021 harness keeps its own stamping, as before.

## M-0020 (`oracle_m0020`)

**How the games are played.** The games are the arena's own, through `oracle::play_with`, with two
hooks:

- `prepare` sees the set-up before the first step. It reads the placement log, and in part 2's
  moved arms it moves the side-A YURI.
- `observe` reads the world and each step's effects after every step, and changes nothing.

`play_checked`, and so every arena game, is `play_with` with no hooks. The unchanged CSVs show this.

**How the values are derived.**

- **Placement log.** `oracle::place_unit_logged` records each try: the cell, the raw values at its
  start, the raw values its creation took, the creation numbers taken, whether the cell took the
  unit, and the raw values after it. Our stream keeps counts of raw values, not a log of draws.
  So the record's "draws" are read as raw values: one per creation number (S-0202 claim 10), then
  one R(0, 3) per infantry try (S-0216 claims 3 and 8), one R(0, 29) for a visceroid vehicle once
  placed, and none for any other vehicle (S-0216 claim 3).
- **The record's off-map objects.** The record counts its objects off the map at tick 0 as refused
  tries. Ours keeps no object for a refused try (S-0216 claim 8's documented departure), so the log
  counts those tries directly.
- **Procedure 5.**
  1. `expected_positions` replays `Rng::new(1000 + s)` to each infantry try's R(0, 3) and takes its
     result as *k*.
  2. It takes row R*k* of S-0208 table O, restated in `table_o_row` rather than read from our walk
     code.
  3. It picks the first position of that row that no lower-identifier infantry unit holds on the
     same cell at tick 0.

  Our placement uses the same stream and the same table, so this row passes partly by construction.
  Its real content is that no unit moved in tick 0's update, and that the draws line up with the
  units.
- **Moved arms.** `move_to_position` moves the unit to the table P point of its cell, on the ground
  there. It moves the unit's mark in the cell record to the slot of the new position, keeping the
  mark's sequence number. A synthetic test checks that a same-spot move leaves the state hash
  unchanged. No move in the 120 moved games met a slot held by another unit. The move is the tool's
  set-up, not a placement.
- **Same-spot runs.** These are compared with part 1's replay of the same game step by step: the
  state hash and the raw-value count after every step, every shot, every hit, and the stream
  before the first step.

**Rows.**

| Row | ours |
|---|---|
| Part 1: result equal to the archived game's (6) | shown: 4 of 6 equal. The arena's statistics judge results, so this row is not counted. Both CAOS–E1 games differ, a known failing pairing. |
| Part 1: shots and damage equal to M-0017's recordings (6) | not checkable: M-0017 does not publish them. |
| Placed infantry at one of the five table P points (all) | pass |
| Placed infantry at the centre or top (none) | pass |
| Placed infantry at the right, left or bottom (all) | pass |
| Placed infantry at Procedure 5's position (all) | pass, partly by construction (above) |
| Procedure 4's pattern equal to the draws before tick 0's update (6) | pass, read from the log's raw-value counts |
| Draws of the run each one raw value (all) | pass, read as raw values per creation number and per placement draw |
| Draws after the run and before tick 0's update (none) | pass |
| Raw values from the seeding to the run (706 in each) | pass |
| YURI games: every unit under mind control at the first such tick is of the opposing group and owned by the YURI's house | pass |
| Part 2 winners: 11 rows of 12 for side A | pass |
| Part 2 winners: YURI against SHK, arm S, 12 for side A | **fail**: ours 11 for side A, 1 for side B. This is the arena's known YURI–SHK discrepancy. Every moved arm of the same pairing gives side A 12 of 12. |
| Moved games with every side-A YURI at the arm's point at tick 0 (120) | pass |
| Moved games' draws before tick 0's update equal to arm S's (120) | pass: the whole stream state is compared |
| Arm S at seeds 0 to 5 against the archived games (12) | shown: 10 of 12 equal (YURI–SHK seeds 3 and 4 differ) |
| Same-spot runs identical to part 1's replay (2) | pass |
| Moved games' damage different from arm S's (120) | pass |

25 rows are checked and 1 fails. Three rows are shown or not checkable.

**Procedures 6 and 7 (local values).** These are printed beside the record's values only in
`oracle-M-0020.txt`. In outline, without values:

- The first targets agree with the record's in five of the six games.
- The first shot and first capture ticks differ from the record's in three of the four YURI games.
- In both CAOS games the drones' end and the bunker events differ from the record's. This is the
  arena's known CAOS discrepancy, and the record does not explain it.
- The firer and target labels are read after the step. A unit captured in the tick of the shot
  therefore shows its new owner.

No new spec claim is in question for M-0020's rows. The failing row is an open discrepancy of the
arena (docs/development.md, *Open discrepancies*), so no request was filed.

## M-0021 revised (`oracle_m0021`)

**What was built.**

- The recorder keeps the vehicles off the map (`Recording::off_map`) and each test weapon's
  warhead.
- `death_order` gives each vehicle's first hit that leaves it with no health, in recording order.
  Deaths in one tick are grouped, as in the record's table.
- `last_f_after` reads the hits in the death tick of each run's last F vehicle destroyed that come
  after its destroying hit.
- `death_shots` reads the destroyed vehicles' shots at *t* and where each one's hit comes.
- `compare_seeds` adds each run's order of deaths and the first place at which the seeds' orders
  differ. It also adds each run's first shot, in shot order, whose target differs from the same
  vehicle's same-numbered shot in another run. A shot number that another run never reaches is not
  compared.

**Rows.**

| Row | ours |
|---|---|
| Destroyed at *t*: a row stamped *t*, off the map with no health | **fail**: 0 of 123 deaths |
| Destroyed at *t*: no row from *t* + 1 on | pass |
| Destroyed at *t*: last row on the map stamped *t* − 1, with health | pass |
| The last F vehicle destroyed takes one more hit after its destroying hit (no attacker, no test warhead, no health) | **fail**: ours has no hit after the destroying hit in any part B run |
| Some destroyed vehicles have a shot at *t* whose hit comes later, naming no attacker | pass |
| Part B order of deaths: each run's order is the record's table | **fail**: ours agree only in their first entries, in some runs |
| The seeds' orders differ at the first death, an F vehicle's | **fail** with F on B: our three seeds share their first death |
| Each seed's first death is the record's | **fail** on both sides |
| Targets the same up to each seed's first death | **fail**: with F on A, seed 1001, a G vehicle's target differs before that seed's first death |
| The first differing shot comes after the first death and is a G vehicle's | **fail**: same run |
| Those firers are the record's | **fail** on both sides |
| Q-2597's row (unchanged) | **fail**, as on main; Q-2597 is still open |
| Part B: side left, F and G dead (Q-2598's row) | **fail**, as on main: the G-dead count differs in 2 of 6 runs |

10 of 50 rows fail, against 3 of 41 on main:

- The old single destroyed-vehicle row was replaced by three rows, and the old seed row on part B's
  deaths by five rows on the order of deaths.
- Two prose rows are new, and so is the table row.

**What is behind the failures.**

- **The seam.** The first row's failure, and the missing extra hit, come from S-0203 claim 6 (a)
  and S-0312 claim 15. Our engine removes a destroyed vehicle from the world at its destroying hit.
  The record's rows show the vehicle still listed at the end of tick *t*, off the map with no
  health, and still taking a hit. Neither claim says whether a vehicle that has left play stays an
  object to the end of its tick. **Q-2636** asks, including what the extra hit is. Interim rule: the
  claims as built. Nothing in the simulation changed.
- **No request for the order of deaths.** The order-of-deaths rows rest on how the vehicles move and
  which target each picks. The record keeps the movement local (country speed factors, S-0413 claim
  1), and it does not explain where the seeds split beyond the delay draw it names. These rows fail
  as part B's G-dead count already did. No approved claim is shown to be departed from, and no
  narrower question than Q-2598's could be asked, so no request was filed.

## M-0019 revised (`oracle::other_houses_verdicts`)

`play_checked` reads Procedure 8's facts in every arena game:

- the houses other than the two players' and the observer's that own an object at tick 0 and at the
  next tick;
- the neutral house's vehicles on and off the map, and whether their identifiers last to the next
  tick;
- its infantry and buildings on and off the map;
- the owners of the tank bunkers (S-0606 Terms);
- the observer's objects;
- the observer's count by `owned` and its defeat at every sample.

The six rows pass in all 1860 games. The row on the neutral house's vehicles passes vacuously: ours
has no neutral vehicle at tick 0, because S-0216 claim 8's removal before the groups takes vehicles
out of the world, which the row's text says. The full facts, including the map's counts, are only
in `oracle-M-0019.txt`.

## Requests

- **Q-2636** (new): whether a destroyed vehicle stays an object, off the map with no health, to the
  end of the tick it is destroyed in; what it can still receive; what the extra hit is; and whether
  its shot at *t* comes before the destroying hit.
- No other request was filed. The block Q-2637 to Q-2645 is unused.
