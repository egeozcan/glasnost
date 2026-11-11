---
id: M-0004
title: Firing cycle with the random-state recorder (delays, bursts, building rounds, draws)
topic: combat
specs: [S-0301, S-0306, S-0215]
requests: [Q-1703]
verdict: contradicts
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0004: Firing cycle with the random-state recorder

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods, plus a test rules file per run. Each test file is the stock rules file with
  the stated keys changed or added, put in that run's game folder. Four of the files end in a last
  line with no line ending after it (Test files and their last line, below).
- Map: the arena map of M-0001, used as a purpose-made game, not a replay of an archived one.
  No map hash beyond M-0001's was supplied.
- Houses: the human player is British, and two computer players are Americans and Russians.
  The human is a normal player: its objects can be attacked. Both computer players play at the
  hardest difficulty level.
- Game settings: every player starts with 0 credits; tech level 10; superweapons off; crates
  off; short game off; bases on; the map is revealed. None of the harness's own gameplay changes
  is active. The computer players own only the stated
  buildings and no construction yard, so with no credits they cannot build. What else they did
  is not recorded, apart from their draws, which the random-state recorder logs as background
  draws (Procedure step 4).
- Starting objects: every starting vehicle is removed, and so is every object of the map's
  neutral house, including its four bunkers and the civilian buildings. The stated objects are
  created on stated cells before tick 0. If a cell is taken, a free cell nearby is used.
- Objects of every run: the Americans' GAPILE (the first target), GAPOWR and GAAIRC, and the
  Russians' one power plant (NAPOWR). Every run except four also holds a second GAPILE of the
  Americans': the four without it are the base run, the run at game-speed setting 4 and the
  retargets at tick 300 to GAPOWR and to GAAIRC. Every object, firers included, is on a stated
  cell that is kept `local`, the same cell in every run that holds the object, and this record
  does not say that its layout is any other record's. Each run then adds one firer of
  the human's, and no other object: the MTNK in every run but the E1 and pillbox runs. So the
  control holds the same objects as every MTNK run with the second GAPILE. Which attacking runs were
  compared with it is recorded only where a row says so (Procedure step 3).
- Main firer: the human's MTNK, requested on its stated cell with a requested facing, both kept
  `local`; the cell it was placed on is not recorded. Every attack order names the first GAPILE
  unless a row names another target.
- Other firers: in the E1 run, an E1 of the human's requested on its stated cell with a requested
  facing, both kept `local`; in the pillbox runs, a pillbox (GAPILL) of the human's on its stated
  cell, kept `local`, with the first GAPILE in range of its gun. Each is ordered to attack the
  first GAPILE. The cells these firers were placed on are not recorded.
- Test rules changes, by role (each run uses the ones its Result rows name). Every test file holds
  the target strengths, the power-plant-proof primary copy and the secondary below, whether or
  not its run uses them:
  - target buildings: a test strength high enough that every run can fire at them for its whole
    length
  - the power-plant-proof primary: a copy of MTNK's stock cannon, made unable to damage the power
    plant (a test value; how it was set is kept in the local file), so against GAPOWR only the
    secondary is chosen
  - MTNK's secondary: the stock cannon with a test rate of fire *R*ₛ in place of its own
  - the burst-3 primary: a copy of MTNK's stock cannon with burst count 3. In the carry-over run the
    burst-3 copy is also made unable to damage the power plant in the same way, and the secondary
    is a copy of the stock cannon with burst count 2 and rate of fire *R*ₛ
  - burst-delay run: the unit's two burst-delay keys set to 7 and 9
  - spray runs: the unit's spray-attack key on
  - scatter runs: a test projectile, added to the test file, that is arcing and inaccurate (its
    other keys kept local), in place of the cannon copy's own projectile
  - fire-particle run: a stock weapon with the fire-particle key, unchanged, as MTNK's primary
    (which stock weapon is kept local)
  - E1 run: a copy of E1's stock gun with burst count 3
  - pillbox runs: a copy of the pillbox's stock gun with burst count 2, the maximum-ammunition key
    set to 3 or 4 (test values) and a reload key far longer than the run. The test file does not set
    the initial-ammunition key
- MTNK's primary in each run: the power-plant-proof copy in the rate-of-fire runs and the control,
  the burst-3 copy in the burst-3, burst-delay, stop, early-retarget and plain spray runs; the
  power-plant-proof burst-3 copy in the carry-over run; a copy of the stock cannon with the test
  projectile in the scatter run, and the same with burst count 3 in the scatter run with burst 3
  and the spray run with the test projectile; the stock weapon in the fire-particle run. Its
  secondary is the one with *R*ₛ in every MTNK run except the carry-over run's burst-2 copy.
- The full delay of the cannon and its copies is written as base + *u*: the base is a whole number
  of ticks per weapon, kept `local`, and *u* is a draw from 0 to 2 attributed to the shot
  (Procedure step 4). It is the countdown after a shot that ends its burst.
- Seed 1000, game speed fastest unless stated.
- Runs and tick limits. Rate-of-fire runs (tick limit local): the base run, the same at
  game-speed setting 4, the retargets at tick 300 to GAPOWR, to GAAIRC and to the second GAPILE.
  Burst and draw-order runs (tick limit local): burst 3, burst delays, E1, stop, early retarget,
  carry-over, pillbox with 3 and with 4 rounds, test projectile, test projectile with burst 3,
  spray, spray with the test projectile, fire particle, and the control (no
  attack order). This record gives shot counts only where stated in Result.
- Tick convention: "tick *f*" is the game's frame counter, and a per-unit state stamped *f* is
  read after tick *f*'s logic has run.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - every starting vehicle and every object of the neutral house are removed, and the stated
    objects are placed on stated cells
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - the named test rules files above
  - scripted orders for the human's objects (attack and stop), applied just before the stated
    tick's logic
  - the run ends at a fixed tick limit
  - a periodic sampler and a per-tick recorder read state only
  - a random-state recorder logs every draw of the scenario stream: its tick, its range, its
    result and the stream index before and after it, so a ranged draw's rejected values show as
    extra index steps. It reads state only. Replaying every logged draw on the S-0202 generator
    model from the seed gives 0 mismatches in every run, controls included.
- Test files and their last line: the base run, the run at game-speed setting 4 and the
  retargets to GAPOWR and to GAAIRC used test files with no line ending after their last line.
  The Result rows that rest on those runs are the first seven. As set up, the last line of each of
  those files was a key of the last test weapon that changes only how it looks. None of the stated
  overrides is that final line. Whether it was read is not shown here: S-0001 claim 5 says such a
  line is not read, and these runs were not repeated with a line ending added. Every other run used
  a file that ends in a line ending.

## Procedure

1. Apply the attack order at tick 5, and the stated orders at their stated ticks: a retarget,
   or a stop order followed by a new attack order. A later order is stated either at a fixed tick
   (tick 300) or as a number of ticks, chosen for the run, after the tick *t*₁ of the firer's
   first shot, which is kept `local` (Result). An order applied before tick *F*'s logic stands for
   an order in the cycle of tick *F* − 1 (S-0203 claim 7); the record gives the tick *F*.
2. Record every shot's tick (a shot is a fire event the recorder logs for the firer), the rearm
   countdown after each shot, the burst position (not for
   the void shots of the spray runs), the firer's ammunition at the end of every tick, and every
   draw with its tick and range. The first impact on the target, which ends the comparison window
   (step 4), is reported by tick; how it was read (the target's health or the projectile's
   arrival) is not recorded. In the retarget run at tick 300 to GAAIRC only, the start and end
   of the tank's turret turn are recorded as reported (how they were read is not recorded). No
   turret turn is recorded in any other run.
3. For the refusal check, compare an attacking run draw for draw with a control run: the same game
   with no attack order. The control's file ends in a line ending (Test files and their last
   line). The rows that rest on the comparison say which attacking runs were compared where that
   is recorded. Which fields of each draw the comparison matched (tick, range, result,
   index step), and how it was kept aligned once the attack's own draws had shifted the stream, are
   not recorded, and the rows that rest on it say so.
4. Attribute each draw. The record logs each draw's tick, range and result, not its caller. A draw
   was attributed by its range, by its result matching the countdown minus the base (the delay
   draw), by its spacing (the attack processing) and, in the comparison window, by the draw-for-draw
   comparison with the control run. The window runs from tick 6 to a local end tick, the tick before
   the first impact, and it holds the first shot, at *t*₁. The draws attributed to the attack
   processing (by range and spacing) are set aside from that comparison. At a later shot a draw is
   attributed to the shot only by its range and result: as its delay draw (result equal to the
   countdown minus the base) or as a scatter draw (its range). Whether a later shot takes any
   further draw is not separated from the background draws. Since the recorder logs no caller, a
   draw these criteria do not attribute to a shot or to the attack processing counts as a
   background draw.

## Result

A firer's first shot tick is taken to depend on the cell and facing it was placed with, which are
not recorded (Setup). That is an inference: no row varies them. The first shot's tick *t*₁ is kept
`local` in every run, and so is the absolute tick of every shot counted from it: they are not for
tests. The spacings between shots and the countdowns are exact where a row says so.

A delay draw is identified by its range and by its result equal to the countdown minus the base
(Procedure step 4). That equality is the attribution criterion, not a separate finding. So a row
that gives a countdown as base + *u* with its draw, or an attributed delay draw from 0 to 2, tests
only the draw's range and that one such draw is attributed to each countdown it covers.

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| MTNK's power-plant-proof cannon copy: countdown after each shot that ends its burst, by the recorded burst position (which shots those are, and the run's shot pattern, kept in the local file) | ticks | 1 run (the base run), every such shot (count local) | below the table | below the table | exact per draw: the draw's range and one attributed draw per countdown |
| Attributed delay draws per burst-ending shot, base run | draws | 1 run (the base run), every such shot (count local) | below the table |  | exact: the range and the count of one |
| Shot ticks at game-speed setting 4 against the fastest | ticks | 2 runs | identical | | exact |
| Retarget at 300 to GAPOWR: secondary's first shot | tick | 1 | the countdown's due tick (tick local) | | that it falls on the countdown's due tick exact; the tick number not for tests: counted from the first shot |
| Secondary's later countdowns after each shot that ends its burst (which shots those are, and its shot pattern, kept in the local file) | ticks | 1 run | below the table | below the table | exact per draw: the draw's range and one attributed draw per countdown |
| Retarget at 300 to GAAIRC: the turret turn, as reported (how it was read is not recorded) | ticks | 1 | below the table |  | not for tests |
| Shot after that turn | tick | 1 | local: the tick after the reported end of the turn | | not for tests |
| Retarget at 300 to the second GAPILE: the shot | tick | 1 | local: later than the countdown's due tick (turret turn not read) | | not for tests: taken to depend on the turret turn, which was not read |
| Burst 3: shot ticks of the first burst | ticks | 1 run | below the table |  | not for tests |
| Burst 3: countdown after shot 1 or 2 of a burst | ticks | every such countdown of the run (count local) | local, with its draws (whether a draw was attributed, its range and its values) | | not for tests |
| Burst 3: countdown after shot 3 | ticks | 1 run | below the table |  | exact per draw |
| Burst 3, burst-delay keys 7 and 9: in-burst countdowns | ticks | 1 run | below the table |  | exact |
| The same: end-of-burst countdown | ticks | 1 run | base + *u* | | exact per draw |
| E1 with a burst-3 copy of its stock gun: in-burst shot spacing | ticks | 1 run | local | | not for tests: the value, and how it relates to the drawn countdowns, are kept in the local file |
| E1: end-of-burst interval | ticks | every end of a burst in the run (count local) | local | | not for tests, as the row above |
| E1: attributed delay draws per shot | draws | 1 run | below the table |  | exact |
| Stop at *t*₁ + 2, attack the same GAPILE at *t*₁ + 3 | ticks | 1 | after the attack order, a fresh burst of three shots (their ticks local), then the full countdown | | the shot count and the full countdown exact; the ticks not for tests |
| Retarget at *t*₁ + 1 to the second GAPILE | ticks | 1 | two shots (their ticks local), then the full countdown: three shots in the burst | | the shot count exact; the ticks not for tests: the first taken to depend on the turret turn, which was not read |
| Primary burst 3, secondary burst 2 at rate *R*ₛ: the first shot with the primary, retarget at *t*₁ + 1 to GAPOWR | ticks | 1 | below the table |  | the countdown exact; the shot tick not for tests: taken to depend on the turret turn, which was not read |
| The same, later |  | 1 run | below the table |  | exact |
| Pillbox, burst-2 copy of its stock gun, 4 rounds: each group except the first (the first group local) | ticks | 1 run | below the table |  | exact |
| Pillbox, 3 rounds: two consecutive groups (which ones local) | ticks | 1 run | below the table |  | exact |
| Pillbox, 3 rounds: ammunition at the end of each tick of a group, each group except the first (the first group local) | rounds | 1 run | below the table | below the table | exact |
| Pillbox, 4 rounds: ammunition at the end of each tick of a group, each group except the first (the first group local) | rounds | 1 run | below the table | below the table | exact |
| Pillbox: stops for lack of ammunition, up to the tick limit | | both runs | never (the test reload key far longer than the run) | | exact |
| Test projectile (arcing, inaccurate): attributed draws per shot, in order | draws | every shot of both scatter runs | below the table |  | exact |
| The same: delay draw at each shot that ends its burst | draws | both scatter runs, every such shot (count local) | below the table |  | exact |
| Spray-attack key on, burst-3 cannon: shots | shots | 1 run | below the table |  | a shot every tick from the first and the countdown exact; the count (every tick from the first shot to the tick limit) and the first tick not for tests: set by the start placement and facing |
| Spray: attributed draws per void shot | draws | every void shot of the run | below the table |  | exact |
| Spray and test projectile: attributed draws per void shot, in order | draws | 1 run | below the table |  | exact |
| The fire-particle run's stock weapon, unchanged, on MTNK: delay draw | draws | 1 run | none attributed: no draw fits the recorded countdown, which is its rate-of-fire field with no added part; further draws inside its shots are not separated from the background | | exact |
| The same: countdown after a shot | ticks | 1 run | exactly its rate-of-fire field, local | | exact |
| The same: interval between shots, under the one attack order and no later order | ticks | 1 run | local, much longer than the countdown | | exact per run |
| The first shot: draws in its tick beyond the control's | draws | 2 runs, each against the control (one of them a run with the test projectile; the other is not recorded) | below the table |  | exact, as far as the comparison of Procedure step 3 goes (its matched fields not recorded) |
| The ticks from 6 to *t*₁ − 1 (before the first shot) and from *t*₁ + 1 to the window's end (local): draws against the control, the draws attributed to the attack processing set aside (Procedure step 4) | draws | 1 run against the control (which attacking run is not recorded) | none extra; whether the tank attempted to fire in each tick was not recorded | | exact, as far as the comparison of Procedure step 3 goes |
| Attack processing: draws outside the shot | draws | 1 run | below the table |  | attribution only; not an independent test of the processing count |
| Interval between the draws attributed to the attack processing | ticks | every such interval of the run (count local) | base + that draw, base local | | a fit: the spacing is one of the attribution criteria (Procedure step 4) |

- **Relations (values in the local file).** How each base (the MTNK cannon's, the secondary's and
  the pillbox's) relates to its weapon's rate-of-fire field is kept in the local file. The
  fire-particle run's weapon's countdown is its rate-of-fire field after every shot. *S* is the
  rules' ballistic scatter distance in leptons, and the first draw's range is from trunc(*S*/2) to
  *S*.

Relations for the table above:

- *MTNK's power-plant-proof cannon copy: countdown after each shot that ends its burst, by the
  recorded burst position (which shots those are, and the run's shot pattern, kept in the local
  file)*: base + *u*, base local, equal to the interval to the next shot; *u* is the attributed
  delay draw (the equality is the attribution criterion, Procedure step 4), spread *u* from 0 to 2.
- *Attributed delay draws per burst-ending shot, base run*: one attributed draw, from 0 to 2
  (attributed by its range and by its result equal to the countdown minus the base, Procedure step
  4).
- *Secondary's later countdowns after each shot that ends its burst (which shots those are, and its
  shot pattern, kept in the local file)*: the secondary's base + *u* (its base local), equal to the
  interval to the next shot; *u* is the attributed delay draw (the equality is the attribution
  criterion, Procedure step 4), spread *u* from 0 to 2.
- *Retarget at 300 to GAAIRC: the turret turn, as reported (how it was read is not recorded)*:
  starts at 300, ends after the countdown's due tick, length local.
- *Burst 3: shot ticks of the first burst*: local, the first at *t*₁.
- *Burst 3: countdown after shot 3*: base + *u*, one attributed draw from 0 to 2.
- *Burst 3, burst-delay keys 7 and 9: in-burst countdowns*: 7, then 9, no attributed delay draw.
- *E1: attributed delay draws per shot*: from 3 to 5 after a shot inside a burst, from 0 to 2 after
  the shot that ends it.
- Primary burst 3, secondary burst 2 at rate *R*ₛ: the first shot with the primary, retarget at *t*₁
  + 1 to GAPOWR: secondary's first shot (tick local), countdown: the secondary's base + *u*, with
  *u* = 1.
- *The same, later*: ordinary 2-shot bursts of the secondary.
- *Pillbox, burst-2 copy of its stock gun, 4 rounds: each group except the first (the first group
  local)*: countdowns 1, 1, 1 with no attributed delay draw, then base + *u* (base local).
- *Pillbox, 3 rounds: two consecutive groups (which ones local)*: countdowns 1, 1, then one draw
  from 3 to 5; next group 1, 1, then base + *u*.
- *Pillbox, 3 rounds: ammunition at the end of each tick of a group, each group except the first
  (the first group local)*: 3 (the tick before the group's first shot), 2, 1, 3: full again at the
  end of the tick of the group's last shot, spread never 0.
- *Pillbox, 4 rounds: ammunition at the end of each tick of a group, each group except the first
  (the first group local)*: 4 (the tick before the group's first shot), 3, 2, 1, 4: full again at
  the end of the tick of the group's last shot, spread never 0.
- *Test projectile (arcing, inaccurate): attributed draws per shot, in order*: from *S*/2 to *S*
  (*S* local), then from 0 to 2147483646.
- *The same: delay draw at each shot that ends its burst*: one from 0 to 2, after the two draws
  above; the draws after those two at any other shot are kept local.
- *Spray-attack key on, burst-3 cannon: shots*: every tick from *t*₁ to the tick limit (count
  local), counted as the fire events the recorder logged for the tank (Procedure step 2), each with
  its own draw from 0 to 7: countdown 0 after each and no attributed delay draw; reported with no
  projectile and no damage (how those two were read is not recorded); called void shots here, an
  inference from those observations.
- *Spray: attributed draws per void shot*: one from 0 to 7, no attributed delay draw; whether a void
  shot takes any other draw is not separated from the background.
- *Spray and test projectile: attributed draws per void shot, in order*: 0 to 7, then *S*/2 to *S*,
  then 0 to 2147483646, no attributed delay draw; any other draw is not separated from the
  background.
- *The first shot: draws in its tick beyond the control's*: no extra draw beyond the draws
  attributed to the shot (in the run with the test projectile, the two scatter draws among them),
  the draws attributed to the attack processing set aside (Procedure step 4).
- *Attack processing: draws outside the shot*: one from 0 to 2 on the order's tick, followed by
  draws attributed to the attack processing by range and spacing.

## Evidence

Each row comes from one run unless its n says otherwise, every draw and every tick recorded, with
controls for the refusal check. The recordings stay local, and no log hash was supplied. Each run's
draws replay with 0 mismatches against the S-0202 generator model. The refusal comparison covers
only Procedure step 4's window.

## Notes

- **Verdict.** *contradicts* refers to S-0301 revision 36: claims 10 and 11 (a building's
  ammunition, below). The fire weapon's refire (below) does not
  decide revision 36's claim 5.
- **A building is refilled at once (S-0301 revision 36, claims 10 and 11).** Revision 36's
  claim 10 says that a building with 3 rounds fires in three consecutive ticks, "then answers
  *rearming* until its countdown ends, and only then *out of ammunition*". Its claim 11 says
  "Infantry and buildings never regain ammunition over time." The pillbox was full again at the
  end of the tick in which it spent its last round, never held 0 rounds at the end of a tick
  and never stopped, with a reload key far longer than the run set. Both sentences are contradicted.
  S-0301 as revised states a refill instead (its claim 10).
- **A fire weapon refires under one attack order (S-0301 claim 5).** The expectation tested was that
  such a weapon fires only once while its firer keeps its target, stays put and lives. Under the one
  attack order, with no later order, it fired again at intervals much longer than its countdown. The
  firer's target and position were not recorded between the shots, so whether it kept its target and
  stood still is not shown, and the run does not contradict that expectation. The refire would fit
  an emitter alive until its system leaves play, as S-0301 claim 5 as revised states (when the
  system leaves play is S-0613 claim 17's and S-0616 claim 22's); this is an inference from the
  specs. The emitter's system, its kind and its lifetime were not recorded.
- **E1's in-burst shot spacing and end-of-burst intervals** are kept local, and so is how they
  relate to the countdowns drawn in the run. Which data sets them is not recorded.
- **Every spray shot looked void** (an inference from the rows above). Why no projectile was
  put on the map is not known. The order of the spray pick against the delay draw in a shot that
  is made was therefore not observed. This covers part of S-0306 scenario P5 (what a void shot
  leaves) as an observation; P5 itself was not run.
- **Other specs.** The attack-processing rows fit S-0215 claim 6 (attributed by range and
  spacing): the processing delay is a
  base plus one draw from 0 to 2. The test projectile's two draws before the delay draw fit
  S-0306 claim 4's scatter draws by range and order; that is an attribution, not an independent
  check, since range is one of the attribution criteria (Procedure step 4).
- **The attack-processing base** may be an engine constant. It is kept local until that is known.
- **Q-1703.** This record answers rank 1 (S-0301 scenarios F1, F3 and F14) in part. Not settled here
  among its parts: F14's no-draw part and its void-shot part (the spray runs observe void shots but
  do not run that part), F1's run with a turret turn shorter than the countdown, and F3's vehicle
  case with its type's burst-delay keys set to −1 in the test file, which no run here does.
  The building refill and the fire weapon's refire (above) bear on S-0301 claims outside the list
  of claims that rank 1 names.
