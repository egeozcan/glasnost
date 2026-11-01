---
id: M-0012
title: A rules file's final line feed and a tank's burst firing
topic: rules-import
specs: [S-0001]
requests: []
verdict: confirms
value-visibility: local
measured-on: 2026-11-01
method: harness
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-01
---

# M-0012: A rules file's final line feed and a tank's burst firing

The shot ticks recorded here confirm S-0001 claim 5 (scenario L4) for one key in the rules file
read at game start: in a fresh load, a last line that no line feed follows is not read. A skirmish
reads its rules file by a fresh load (S-0002 claim 6: the one layered load is at a campaign
mission's start). What the runs observe is the tank's firing, not the reading of the line: that
the key went unread is inferred from the firing.

## Setup

There are four recordings: runs 1 and 2, the control, and a rerun of run 1 (Evidence). "Every
run" below means all four.

- Game build: stock Yuri's Revenge 1.001, executable hash verified; run under the oracle harness
  (PROVENANCE §7).
- Data: stock, apart from one test rules file per run: a loose `rulesmd.ini` put in the run's
  game folder. Each is the stock rules file with the changes below. Run 2's file is the file of
  M-0004's burst-3 run; that its last line is the burst key is stated here, as this record's own
  setup. Run 1's file is the same without the line feed after its last line, and the
  control's file is run 2's with the burst key's line removed. Neither is one of M-0004's runs. The
  bullets below are every change the files hold: those every M-0004 test file holds (M-0004
  Setup) and the burst-3 copy.
  - the target building types (GAPILE, GAPOWR, GAAIRC): a test strength high enough that the tank
    can fire at them for the whole run
  - the tank's primary weapon: a copy of its stock cannon with a burst count of 3 (a test
    value). The burst key is the file's last line in runs 1 and 2. In the control the copy has no
    burst key at all, so it fires with the absent-key default, not with any burst data of the
    stock cannon; every other line is the same as in run 2's file, which ends in a line feed
  - the tank's secondary weapon: the stock cannon with M-0004's test rate of fire *R*ₛ
  - M-0004's power-plant-proof cannon copy, made unable to damage the power plant (a test
    value, set in a way kept in the local file). It is in the files only because every M-0004 test
    file holds it: it is not the tank's primary or secondary here, and no shot in any run is of it
- Map: the arena map of M-0001 (`Arena.mmx`, with M-0001's sha256), used as a purpose-made game,
  not a replay of an archived one. No map hash beyond M-0001's was supplied.
- Game: a skirmish, seed 1000, fastest game speed.
- Houses: the human player is British, and two computer players, Americans and Russians, play
  at the hardest difficulty level. The human is a normal player: its objects can be attacked.
- Game settings, the same in every run (and the same as M-0004's): every player starts with 0
  credits; tech level 10; superweapons off; crates off; short game off; bases on; the map is
  revealed. None of the harness's own gameplay changes is active.
- Objects, the same in every run, each created before tick 0 on a requested cell kept
  `local`, as in M-0004; if that cell is taken, a free cell nearby is used, and the cells they were
  placed on are not recorded: the human's medium tank (MTNK), with a requested facing kept
  `local`; the Americans' two barracks (GAPILE), power plant (GAPOWR) and air pad (GAAIRC); the
  Russians' power plant (NAPOWR).
- Order: the tank is ordered to attack the first barracks, applied just before tick 5's logic. By
  S-0203 claim 7 that stands for an order in the cycle of tick 4.
- The tick limit is the same in every run and is kept `local`, as in M-0004, whose burst-3
  run is run 2.
- What is not stock:
  - no menus: the skirmish setup comes from a settings file
  - every starting vehicle and every object of the map's neutral house are removed, and the
    stated objects are created on requested cells
  - the two computer players are not allied with each other (a stock skirmish would ally every
    computer player when nobody is on a team)
  - the test rules files above
  - the scripted attack order
  - the run ends at a fixed tick limit
  - a per-tick recorder and a random-state recorder read state only (no draw is used here)

## Procedure

In every run, a shot is a fire event the recorder logs for the tank, as in M-0004 Procedure step 2.
Record every shot with its tick, its weapon and the rearm countdown after it, up to the tick
limit.

1. Run 1: run the game with the test rules file whose last line is the added burst key, with no
   line feed after it.
2. Run 2: run it again with the same file and one line feed added after that line.
3. Control: run it again with run 2's file with the burst key's line removed.
4. Compare run 1's shot ticks with the control's, tick for tick.
5. Compare run 2's shot ticks with the control's.

## Result

A first shot's tick is taken to depend on the cell and facing the tank was placed with, which are
not recorded (Setup), as in M-0004. That is an inference: no run varies them. So no run's shot
ticks are for tests on their own: only the two comparisons are.

| Quantity | Unit | n | Value | Spread | Tolerance for tests |
|---|---|---|---|---|---|
| Run 1 (no line feed after the last line) against the control (burst key absent): all shot ticks | ticks | run 1, run 1's rerun and the control, 1 recording each, all shot ticks to the tick limit (count local) | equal, tick for tick: every shot tick of run 1 is a shot tick of the control and the reverse (both lists local) | | exact |
| Run 2 (one line feed after it) against the control: all shot ticks | ticks | run 2 and the control, 1 recording each, all shot ticks to the tick limit | differ: some shot tick of run 2 is not a shot tick of the control (both lists local) | | exact |
| Run 2: shot ticks | ticks | 1 run | local | | not for tests |

The number of shots is not given for any run. That run 1's burst key went unread is inferred from
the first row: run 1 fires as the control does, where the key is absent. That the same key is
read once a line feed follows it is inferred the same way, from the second row. Every run has
the same seed, map, objects, requested cells and facing, and only the rules file differs; the
placed cells are not recorded, so no recording rules out a placement difference behind the second
row.

## Evidence

- One key, in the rules file only. The recordings stay local, and no log hash was supplied.
- Every shot in every run is of the tank's primary weapon (the recorded weapon). Besides the
  ticks, every other recorded field of each fire event (weapon, countdowns) is equal between
  run 1 and the control.
- Runs 1 and 2 were played on 2026-10-28, the control on 2026-11-01. With the control, run 1 was
  played again on 2026-11-01 from its own file: every recording of that rerun is identical to run
  1's. The control was played once, on 2026-11-01: both rows compare with that one recording.
  Run 2 was not played again.
- Run 2 is the burst-3 run of M-0004.
- Found while preparing other test runs. M-0004 Setup, item "Test files and their last line",
  states which of its runs used such a file and which of its Result rows rest on them; this
  record says nothing further about those runs.

## Notes

- Not tested: other files, a layered load (S-0002 claim 9 says a layered load reads such a line),
  and a last line that is a section header.
