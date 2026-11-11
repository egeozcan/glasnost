---
id: M-0008
title: Random streams with the random-state recorder (seeding, ranges, creation, idle draws)
topic: simulation
specs: [S-0202, S-0107, S-0216]
requests: [Q-1703]
verdict: confirms
value-visibility: local
measured-on: 2026-10-28
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-11
---

# M-0008: Random streams with the random-state recorder

## Setup

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, no mods. No test rules or art file is used in the runs reported here, apart from
  the MTNK control run and the unsellable run (What is not stock).
- Map: the arena map of M-0001, used as a purpose-made game, not a replay of an archived one.
- Houses: the human player is British, and two computer players are Americans and Russians.
  The human is a normal player: its objects can be attacked. Both computer players play at the
  hardest difficulty level, in every run.
- Game settings, the same in every run except the seed (the starting-unit setting was not recorded,
  below): every player starts with 0 credits (part c then sets the human's credits, below), tech
  level 10, superweapons off, crates off, short game off, bases on, and the map is revealed. None of
  the harness's own gameplay changes is active.
- Starting objects: every starting vehicle is removed, and so is every object of the map's neutral
  house, including its four bunkers and the civilian buildings. The starting-unit setting was not
  recorded, so the figures that include game-start draws hold for the runs' setting only (Result).
  The stated objects are created on stated cells before tick 0. If a cell is taken, a free cell
  nearby is used. The human's start position plays no part in what is reported: the human's
  objects are the stated ones.
- Idle and creation runs (S-0202 scenario R5):
  - the human owns a power plant, stated at 124,82; the cell it was placed on, the same in every R5
    run, is kept `local`. It stands outside the observed area. Each computer player owns
    one far power plant and has no credits, and owns no construction yard (S-0202 scenario R5
    gives its bystander one; this record's computer players have none). The stated cells are
    60,82 (Americans) and 100,70 (Russians); the cells they were placed on are not recorded.
    These far power plants and every object they bring into play are the *bystanders*.
  - part a: one E1 of the human's is placed at 130,84 at tick 300, in *guard*, and is then left
    standing. Its recorded desired body facing before attempt 1 is 2 eighths of a turn. The
    placement is applied just before tick 300's logic, as the order of part c is. A control run is
    the same game without the placement.
  - part b: the draws at part a's placement tick that its control lacks (the creation).
  - part c: the human also owns a power plant at 135,86 and a barracks at 135,81, both
    created on stated cells before tick 0. Scripted input sets the human's credits to 10000 before
    tick 0, and at tick 300 orders the production of one E1. A control run is the same game
    without the order.
  - the unsellable run: the game of part a, placement included, with one change: a test rules
    file under which both computer players' power plants cannot be sold.
- MTNK control run: a standalone control with an armed tank left in *guard*. The human's MTNK
  is created before tick 0 on the stated cell 131,84; the cell it was placed on is not recorded.
  The Americans own barracks at 135,81 and 136,78, a power plant at 135,86 and an air pad at
  127,86. The Russians own one far power plant at 100,70. The human owns no other stated object
  (any starting units of the unrecorded starting-unit setting aside, Starting objects). The run
  uses a test rules file (What is not stock). The MTNK is given no order, so it stays in *guard*.
  It fires no shot.
- Seeding runs (S-0202 scenario R2 a): the settings file gives no seed, so the game's own clock
  seeding runs at game start in place of a given seed. Three fresh launches, about 30 s and 18 s
  apart. Each launch has three objects: the human's power plant, stated at 124,82, and the two
  computer players' far power plants, stated at 60,82 and 100,70. The cells they were placed on
  in these launches are not recorded. Just before the seeding the host's clocks are read: the system
  tick count, the wall-clock time and the uptime. The scenario and unsynchronised streams are read
  at the first tick.
- Seed 1000 in every other run. Game speed fastest.
- Tick limits: the R5 a run, its control and the unsellable run a limit kept `local`, each stopped
  earlier by the operator (Notes); the R5 c run, its control and the MTNK control run a limit kept
  `local`; each R2 a launch 304. In the eight runs of Procedure step 1, every draw logged up to a
  run's last recorded tick is replayed.
- Tick convention: "tick *f*" is the game's frame counter, and a per-unit state stamped *f* is
  read after tick *f*'s logic has run.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - every starting vehicle and every object of the neutral house are removed, and the stated
    objects are placed on stated cells
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - scripted input: the credits change of part c before tick 0, and its production order and
    the placement of part a, each applied just before tick 300's logic. By S-0203 claim 7 an
    order applied before tick 300's logic stands for an order in the cycle of tick 299 (S-0201,
    *cycle*). The record reports their draws at tick 300, the tick whose logic follows them
  - a run ends at its tick limit or when the operator stops it. The R5 a run, its control and the
    unsellable run were stopped (Notes)
  - the MTNK control run's test rules file: it makes the Americans' buildings durable and gives the
    tank test copies of its weapons. The tank fires no shot in this run
  - the unsellable run's test rules file (above)
  - a periodic sampler and a per-tick recorder read state only. The per-tick recorder records each
    unit's desired body facing, among other state
  - a random-state recorder logs every draw of the scenario stream: its tick, its range, its
    result and the stream index before and after it, so a ranged draw's rejected values show
    as extra index steps. It also logs each seeding, with its seed. It reads state only

## Procedure

1. **Generator model.** For each of the eight runs this record describes (the R5 a run and its
   control, the R5 c run and its control, the MTNK control run and the three R2 a launches),
   replay every logged draw on the generator of S-0202 claims 1 to 4, from the logged seed, and
   compare each result and each index step.
2. **Control interval (R5 a).** Ticks 0 to 299 of the R5 a run and its control, compared draw for
   draw, and counted by range.
3. **Creation (R5 b).** The draws at tick 300 of the R5 a run that its control lacks.
4. **Idling (R5 a).** The run's extra draws against the control in the clean window (Notes), grouped
   by range, and the 0..2 draws into chains by their ticks and values only (Notes). For each idle
   attempt, the expiry predicted by S-0107 F4 from the attempt's timer draw and the stock
   idle-action frequency. The E1's desired body facing, read from the per-tick recorder's state
   records, after attempts 1 and 3 and over the rest of the clean window.
5. **Production (R5 c).** The draws at tick 300 that the control lacks, the tick at which the
   unit leaves the barracks, and the unit's first idle attempt after it leaves, identified by its
   timer draw and *r* draw as in step 4.
6. **Seeding (R2 a).** For each launch: the seed against the system tick count and the host's
   uptime, both read just before the seeding, the seed differences against the wall-clock
   differences, and the smallest count *N* for which the seed's computed state equals the recorded
   state, for each stream. Both streams are read at the same point of tick 0; whether that point is
   before or after tick 0's logic is not recorded. The random-state recorder's seeding rows, each
   with its seed, are read for every seeding before the game-start one.
7. **Resting tank (MTNK control run).** The run's 0..2 draws, picked out as chains by their ticks
   and values only (Notes).

## Result

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Replay of logged draws on the S-0202 generator, results and index steps | mismatches | 8 runs; logged calls per run: local | below the table | | exact |
| Control interval, ticks 0 to 299: draws of the R5 a run against its control | draws | 1 pair | identical | | exact |
| Draws in the control interval, ticks 0 to 299, game-start draws at tick 0 included, with the runs' starting-unit setting, by range: raw value | calls / index steps | 1 run | below the table | | exact within the runs' setting; none outside it (not recorded) |
| Range 0..0, equal bounds (the case of S-0202 claim 4) | calls / index steps | 1 run | logged, and none takes a raw value (count local) | | exact within the runs' setting; none outside it (not recorded) |
| Every other range | calls / index steps | 1 run | ranges, bounds, calls and index steps local | | exact within the runs' setting; none outside it (not recorded) |
| Draws at the placement tick (300) that the control lacks, in order | draws | 1 | below the table | | exact |
| The 0..2 draws that the control lacks, taken as the E1's, in the clean window, split into chains by their ticks and values (Notes) | chains | 1 run | below the table | | a fit (Notes), not checked: no test tolerance |
| Family G: spacing to the next draw | ticks | every interval of the clean window (count local) | a fit, not a check (Notes): one base for the whole family + the previous draw, base local | | a fit (Notes), not checked: no test tolerance |
| Idle attempts (identified by their own timer and *r* draws) on a tick of a family G draw | attempts | 4 | below the table | | a fit (Notes), not checked: no test tolerance |
| Family T: first | ticks after the creation | 1 | below the table | | a fit (Notes), not checked: no test tolerance |
| Family T: spacing to the next draw | ticks | every interval of the clean window (count local) | a fit, not a check (Notes): the normal targeting delay (local) + the previous draw | | a fit (Notes), not checked: no test tolerance |
| The MTNK control run: its 0..2 draws, as chains | tick | 1 run | below the table | | a fit (Notes), not checked: no test tolerance |
| Idle attempts 1 to 4, in the clean window | ticks | 4 | local | | exact |
| Draws of one idle attempt, in order | draws | 4 | below the table | | exact |
| Attempt 1 | draws | 1 | below the table | | exact |
| Attempt 2 | draws | 1 | below the table | | exact |
| Attempt 3 | draws | 1 | below the table | | exact |
| Attempt 4 | draws | 1 | below the table | | exact |
| Each later attempt against S-0107 F4's predicted expiry (local) | | 3 | the first family G tick (taken as a guard processing) at or after the expiry | | a fit (Notes), not checked: no test tolerance |
| Family G ticks (taken as guard processings) just before attempts 2, 3 and 4 | tick | 3 | local; each before the predicted expiry | | a fit (Notes), not checked: no test tolerance |
| Attempt 1, timing | | 1 | below the table | | a fit (Notes), not checked: no test tolerance |
| Desired body facing after attempt 3's δ = 7 | eighths of a turn | 1 | below the table | | exact |
| Desired body facing after attempt 1's δ = 2 | eighths of a turn | 1 | below the table | | exact |
| Production, R5 c: draws at 300 that the control lacks | draws | 1 | one raw value | | exact |
| Production: the unit leaves the barracks | ticks after 300 | 1 | local, as is the cell it leaves to | | exact |
| Production: first idle attempt | ticks after leaving | 1 | below the table | | exact |
| The unsellable run against the R5 a run | state records | 1 pair | identical over the ticks both recorded; inconclusive: whether its test rules file took effect was not shown (Notes) | | none |
| Seed, launches 1, 2, 3 | | 3 | below the table | | exact |
| Seed against the system tick count read just before the seeding | | 3 | equal | | exact |
| Seed minus host uptime | ms | 3 | below the table | | ±10 ms |
| Seed differences against wall-clock differences, launches 1–2 and 2–3 | ms | 2 | below the table | | within 2 ms |
| *N*, scenario stream at tick 0, with the runs' starting-unit setting | raw values | 3 | below the table | | exact within the runs' setting; none outside it (not recorded) |
| *N*, unsynchronised stream at tick 0, same seed and setting | raw values | 3 | below the table | | exact within the runs' setting; none outside it (not recorded) |
| An earlier seeding before the game-start seeding, read from the random-state recorder's seeding rows | seed units | 3 | below the table | | launch 1 exact; launches 2 and 3 about 600 |

Relations for the table above:

- *Replay of logged draws on the S-0202 generator, results and index steps*: 0.
- *Draws in the control interval, game-start draws included, by range: raw value*: 3494 / 3494.
- *Draws at the placement tick (300) that the control lacks, in order*: raw value; 0..3; 0..2.
- *The 0..2 draws that the control lacks, taken as the E1's, as chains*: 2: family G, whose first
  draw is the third of the placement tick's draws, and family T, whose first draw comes a fixed
  first run-out after the creation (local).
- *Idle attempts on a tick of a family G draw*: 4.
- *Family T: first*: local: a fixed first run-out, the same as the MTNK control run's after tick 0.
- *The MTNK control run: its 0..2 draws, as chains*: a fit (Notes): a chain from tick 0 at family
  G's spacing and a chain from the first run-out after tick 0 at family T's, in every interval to
  the tick limit.
- *Draws of one idle attempt, in order*: timer draw 0..2147483646; *r* 0..10; δ 0..7 after *r* = 6
  and *r* = 10 (attempts 1 and 3), none after *r* = 0 and *r* = 5.
- *Attempt 1*: timer local; *r* = 6 (one rejected value, 2 index steps); δ = 2.
- *Attempt 2*: timer local; *r* = 0; no δ.
- *Attempt 3*: timer local; *r* = 10; δ = 7.
- *Attempt 4*: timer local; *r* = 5; no δ.
- *Attempt 1, timing*: at the second family G tick, taken as the E1's second guard processing (the
  first is at 300, the creation tick); its tick local.
- *Desired body facing after attempt 3's δ = 7*: 7.
- *Desired body facing after attempt 1's δ = 2*: 2, the facing it already had: no change shows.
- *Production: first idle attempt*: local; a timer draw, then *r* = 0.
- *Seed, launches 1, 2, 3*: 39 557 213, 39 586 941, 39 604 667.
- *Seed minus host uptime*: −497, −499, −503.
- *Seed differences against wall-clock differences, launches 1–2 and 2–3*: 29 728 / 29 727; 17 726 /
  17 728.
- *N, scenario stream at tick 0, with the runs' starting-unit setting*: 725, 708, 714.
- *N, unsynchronised stream at tick 0, same seed and setting*: 147, 133, 115.
- *An earlier seeding before the game-start seeding*: in every launch a seed about 600 lower, on a
  stream the record does not identify (about 0.6 s earlier if that stream is seeded from the same
  millisecond count, which is not shown); launch 1: seed 39 556 602, then 39 557 213, 611 apart.

The control interval's rows (the raw-value and equal-bounds counts and every other range), the
tick-0 *N* readings and the bystander drift in Notes were taken with the runs' starting-unit
setting, which was not recorded. The players' starting units take creation draws at game start
(S-0202 claim 10), and the setting decides which units those are, so a reproduction under another
setting must expect those figures to differ.

Relations, stated without stock values:
- The three draws at the placement tick fit, in order, the creation draw of S-0202 claim 10, the
  standing-position draw of S-0216 claim 3 and the 0..2 draw of the first guard processing.
  They were matched by range and order (a raw value, then 0..3, then 0..2). The one draw of the
  production order fits S-0202 claim 10's production creation draw, matched the same way.
- The recorder does not record guard processings or the targeting timer themselves. The E1's
  extra 0..2 draws are split by observables only, their ticks and values. They split into two
  chains, families G and T, in which each later draw comes one base for the family plus the
  previous draw's value after the draw before it. Family G's first draw is the creation tick's
  third extra draw, and family T's first draw comes a fixed first run-out after the creation, its
  length local. It is the delay to that family's first draw as the split gives it.
- What the split shows: the draws can be split this way, with every interval of the clean window
  at its family's spacing, and the four idle attempts all fall on family G ticks. What it does
  not show: that the split is the only one, or which processing takes each family's draws. The
  chains and their bases were found together, so a family's constant spacing is the fit that
  makes the split, not a separate check of it.
- Family G is taken as the guard-processing delay draws: its first draw is the creation tick's
  third extra draw, and every idle attempt falls on one of its ticks (S-0107 claim 14 makes
  attempts at guard processings). Its spacing fits the guard rate's processing delay of S-0215
  F1 plus the previous draw's value.
- Family T's spacing fits the rules' normal targeting delay (local) plus the previous draw. That
  fits the targeting timer of S-0215 F6 restarted by the per-tick search of S-0309 claim 1, first
  row, which finds nothing here. That family T is that timer's restart is a fit: the recorder
  does not record the timer itself.
- In the MTNK control run the tank's 0..2 draws are not split from the other systems' by a
  comparison. They are picked out of the run's 0..2 draws as two chains of the same form, one from
  tick 0 and one from the first run-out after tick 0. So that row is a fit as well.
- The predicted idle expiries use the stock idle-action frequency (local), by S-0107 F4.
- What the verdict covers. For S-0202, claims 1 to 4 for the logged draws, and claim 10's creation
  draw by a fit of range and order. For S-0107, that claim 14's idle attempts fall at family G
  ticks, taken as guard processings; which condition of that claim fails at the creation tick was
  not recorded. For S-0216, only the order of claim 3's *R*(0, 3) standing-position draw after the
  creation draw; that the third draw is a behaviour-processing draw rests on a fit.

## Evidence

- Eight runs replayed draw for draw (step 1): one R5 a run and its control, one R5 c run and its
  control, the MTNK control run, three R2 a launches. The unsellable run is not counted among
  the eight.
- Recorder logs and state records are kept by the oracle operator; the files stay local. No log hash
  is stated. The ticks, ranges and values of the 0..2 draws and the logged-call counts per run were
  read from those logs.

## Notes

- **The clean window (R5 a).** The recorded events that end it are: the computer players' crews come
  into play, both their power plants' sales are recorded after that, and later one crew walks to the
  human's plant and attacks it. The record does not show which event the sale records mark (the
  order, the start of the build-down or the removal), and their ticks are `local`. The clean window
  runs from tick 300 to the tick before the crews come into play, and ticks from then on are void
  for the comparison. That tick, the later events' ticks, the attempts' ticks, the family G ticks
  before them and the counts of intervals are kept `local`: with the window's length they would give
  or bound the local delays and the idle-action frequency. The attempt count, 4, is public: with the
  window's end, the attempts' ticks and the interval counts local, and no run's length or
  logged-call total committed, no committed figure gives the window's length, so the count bounds
  neither. Whether the clean window covers R5 a's full idling time is not claimed.
- **How the R5 a runs ended.** The R5 a run and its control stopped advancing some time after the
  human's plant was destroyed, and the operator stopped them.
- **The unsellable run.** Its state record was byte-identical to the R5 a run's over the ticks
  both recorded. It was stopped by the operator; where is not recorded. So whether the file took
  effect was not shown by this run.
- **Bystander drift.** The bystanders' own draws also differ a little between the R5 a run and its
  control: one more draw each of two of the other ranges (local) and 28 more raw values, with the
  runs' starting-unit setting (Result). They were matched by count only, not predicted. The
  infantry's draws are inferred: they are the extra draws in the infantry's ranges once the
  bystanders' extra draws are matched by count. Among themselves they are told apart by range and
  order (the idle attempt's timer draw then its 0..10 draw) and by tick and value (the 0..2 draws of
  families G and T). R5's strict per-tick check, with every bystander draw predicted, was not done.
- **Control-interval draws.** The E1 is not yet in play in this window, so none of them is the E1's.
  Which systems take them, the game-start draws at tick 0 included, is not recorded. Extra index
  steps appear only where a value is rejected, as S-0202 claim 4's rejection rule gives: they can
  appear for a range whose size is not a power of two. The replay of step 1 compares each logged
  result and each call's index advance, including the extra steps attributed to rejections.
- **The first idle attempt** comes at the second family G tick, taken as the E1's second guard
  processing, not at the creation tick's.
  S-0107 claim 14 gives a new unit's timer as already run out; which other condition of that claim
  fails at the creation tick was not recorded.
- **The facing change after idle-a (inferred).** After attempt 4's *r* = 5, the recorded desired
  body facing later changes to idle-a's declared direction (stock art data, unmodified; the facing
  and its tick are local), with no draw of the E1's at the change. This fits S-0107 claim 9, which
  sets the facing to a completed sequence's declared direction, and fits *r* = 5 requesting idle-a.
  The recorder did not record the sequence, so that *r* = 5 requested idle-a and that the change
  marks its completion are inferred, not read.
- **Seeding.** In the three launches, the seed equals the host's millisecond tick count read just
  before the game-start seeding, exactly. Here that count equals the host's uptime minus about
  0.5 s. The seed differences compare separate launches, not games within one session. The
  wall-clock times were read just before the seeding, not at the first tick. In the seeded runs
  of this session the recorder's seeding rows show an earlier seeding, then the given seed, 1000.
- **Scope for Q-1703 (partial).** This record settles S-0202 claims 1 to 4 for the logged draws, and
  backs the seeding rows of claim 8 that S-0202 rates `measured` on it, and is consistent with the
  parts of claims 10 and 11 whose backing cites it. It supports the timing of S-0107 claim 14's
  attempts and the S-0215 hand-offs (families G and T) as fits and inferences only. It does not
  settle claim 6: its draws over 0..2147483646 are counted and replayed, but not compared with a
  derived unit-interval value. It is partial evidence for scenario parts R2 a (separate launches
  only) and R5 a, b and c: R5 a's part covers the infantry's draws in the clean window, with the
  bystanders' draws matched by count only, and R5's strict per-tick check was not done.
- **Not measured:** R2 a's two further games in one session (each recorded game was a separate
  launch); R2 b (network); R2 c (campaign); R5's strict per-tick check.
