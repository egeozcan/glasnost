# S-0216 r59 and M-0019: claim ledger, M8 step A3 intake 77

S-0216 r37 to r59 rewrites claim 8 of our arena's set-up (the order of the groups, the draws of a
refused cell, the acceptance against M-0019) and claim 3 (where the creation draws of the objects
a unit's creation makes fall). M-0019 is new: the records' units placed at tick 0 in M-0001 to
M-0003 (creation order, standing positions, the raw values of the scenario stream before tick 0).
It answers **Q-2277** and **Q-2286** in part; their open questions stay open (below).
[Intake 71](intake71-S-0513-S-0216.md) built r37; its S-0216 rows are superseded by the rows
here. Request filed: **Q-2336**.

## Built

- **The order** (claim 8, *Order*). `oracle::setup(ground, types, row_side, seed)` creates the
  group of the pairing's row type first, on whichever side it plays, then the other group, each
  in increasing unit index. `play`, `spread` and `oracle_trace` pass the game's side; a type
  against itself and the `--groups` listing pass `Side::A`.
- **The tries** (claim 8, *Draws of a refused cell*). `oracle::place_unit` tries the stated cell
  and then its eight neighbours in claim 8's order. Each try is a new unit of the type made by
  `objects::create` with its whole creation set (slot objects, S-0506 claim 1; slaves, S-0402
  claim 19; S-0611 claim 14's order). The take test (`refusal`, no draw) is asked; a cell that
  takes it gets `objects::place_in_play` and the queued *hunt*. A refused try takes, for
  infantry, one unused R(0, 3), even where claim 2's first row would refuse the cell, and its
  objects are then discarded by the new `objects::discard_unplaced`, which keeps their creation
  numbers reserved (S-0611 claim 14). If no cell takes the unit, the set-up stops with an error
  that gives the stated cell's refusal.
- **`objects::discard_unplaced`** is `mutation::discard` moved to `objects` so the tools crate can
  call it; `mutation::discard` and `testkit::spawn_in_play`'s failure path call it. No behaviour
  of the simulation changes.
- **`Arena::first_raw`**: the raw values taken from the scenario stream before the first
  object's creation draw, for claim 8's acceptance against M-0019's *N*₀.
- `docs/development.md`, arena *Differences*: the r59 order and tries.

## Design choices

- The tries keep no object, as claim 8 states (the documented departure from the original, which
  leaves an object off the map). The departure has no switch: claim 8 is a rule of our own
  arena, not of the game, so `every_spec_quirk_has_a_switch` has nothing to add (it passes).
- The error for a unit no cell takes reports the stated cell's refusal, as before.

## Claims

### S-0216 r59

| claim | status |
|---|---|
| 3 (r59): the creation draws of the objects a unit's creation makes fall after the unit's own and before its placement draw; slot objects in S-0506 claim 1's order (PT3 third run) | Tested: `claims_s0216_r59::claim_3_pt3_third_run_slot_objects_draw_before_the_standing_position` |
| 3 (r59): a vehicle's initial slaves' creation draws, then the next unit's (PT3 fourth run); the slaves take no R(0, 3) | Tested: `claims_s0216_r59::claim_3_pt3_fourth_run_slaves_draw_before_the_next_unit` |
| 8, *Order*: the row type's group first on whichever side it plays, then the other, each by index | Tested: `arena_setup::pt6_the_row_types_group_is_created_first_on_whichever_side_it_plays`; on the local lane `arena::m0019_creation_order_and_positions_in_our_setup` (192 all-infantry games of M-0002) |
| 8, *Order*: each unit created, tested and placed before the next; per unit its creation draw, its made objects' creation draws, its placement draw; a creation number right after each creation draw | Tested: `arena_setup::pt6_each_unit_takes_its_creation_draw_then_its_one_placement_draw` (updated), `pt6_a_refused_try_makes_its_whole_creation_set_and_keeps_none_of_it` |
| 8, *Draws of a refused cell*: one try per refused cell, a new unit with its whole creation set, then for infantry one unused R(0, 3), even where the first row refuses the cell | Tested: `arena_setup::pt6_a_unit_on_the_p_th_cell_follows_p_refused_tries_and_keeps_no_try` (2p draws for infantry, p for a plain vehicle), `pt6_a_refused_try_makes_its_whole_creation_set_and_keeps_none_of_it` (aircraft slot objects, infantry slaves, a spawner, a slave master) |
| 8, *Draws of a refused cell*: no object kept, creation numbers reserved (the departure, no switch) | Tested: the two tests above (`next_id` advances past every try; no try's object is in the world) |
| 8, *Moving on*, *The take test*, *Placement*, *Group counts*, *Map objects* | Unchanged from r37; tested as in intake 71's rows (`pt7_a` to `pt7_d`, `pt6_a_full_cell_moves_infantry_on_and_the_mission_is_queued`, `oracle::tests`, `arena::m0006_bunkers_and_placed_cells_in_our_setup`) |
| 8, *Acceptance* (M-0019): row type's units first; no placed infantry at the centre or top position | Tested on the local lane: `arena::m0019_creation_order_and_positions_in_our_setup` (passes) |
| 8, *Acceptance* (M-0019): the first object's creation draw takes the raw value at place *N*₀ (706, 709, 711, 718, 706, 701 for seeds 0 to 5); draws before the set-up are our engine's map loading and game start, no padding | Tested on the local lane: `arena::m0019_the_first_creation_draw_takes_the_raw_value_at_n0`, **fails** (a known acceptance failure). Ours is 693, 685, 685, 695, 693, 689: short by 13, 24, 26, 23, 13, 12, the same in every pairing for a seed. Filed **Q-2336**; interim as claim 8 says, no padding |
| 8, *Acceptance*: the off-map counts of M-0019 | Not claimed: M-0019's tolerance says claim 8's arena cannot be required to match them |
| 8, aircraft-kind groups | Seam, as at intake 71 (no record has one) |
| Q-2277 question 1 (several infantry on one cell) and question 3; Q-2286 (CAOS against E1) | Open in the spec; left open |

## Requests

- **Q-2336**: which draws make up the records' game start before the set-up. Ours takes 12 to
  26 fewer raw values, depending on the seed: 693, 685, 685, 695, 693, 689 against M-0019's *N*₀
  of 706, 709, 711, 718, 706, 701. M-0019 finds starting objects (vehicles of both houses, gone
  at the next tick) that our arena does not create: its houses have no starting units and bases
  off. Interim: as claim 8 says, no padding, our game start as it is. The local-lane check of
  *N*₀ fails as a known acceptance failure.
- Q-2277 and Q-2286 stay open.

## Lanes

The sim change is a function move (`mutation::discard` to `objects::discard_unplaced`), so the
lanes outside the arena are unchanged. Before is main 23869cf; after is this branch on 23869cf.
The arena was run again after rebasing on main d645ca3 (intakes 74 and 78), against a build of
d645ca3 itself.

- **M6c, M7a, M7b seed sweeps 1 to 16**: identical output line by line, timings aside. Every
  sweep passes. In M6c the smallest minimum is 0 (seed 7: air-strike orders).
- **`cargo test --release -p glasnost-ai -- --include-ignored`**: 246 passed, 0 failed, before
  and after.
- **AI quick lane** (`GLASNOST_AI_QUICK=1`, seeds 7 to 10 on maps 0, 2 and 8, 36 games):
  identical apart from timings, with the same tick counts and winners.
- **Arena** (`arena --record all --threads 8`): scored pairings failing, of 155 (M-0001 / M-0002
  / M-0003):

  | Build | Failing | Win test failures |
  |---|---|---|
  | main 23869cf | 13 (9 / 3 / 1) | CAOS–V3, DISK–FV, YURI–GGI, YURI–SHK |
  | intake 77 on 23869cf | 15 (11 / 3 / 1) | DISK–FV, YURI–GGI |
  | scratch: intake 77 padded to *N*₀ (not committed) | 11 (7 / 3 / 1) | CAOS–V3, DISK–FV, YURI–SHK |
  | main d645ca3 | 10 (8 / 1 / 1) | DISK–FV |
  | intake 77 on d645ca3 | 11 (9 / 1 / 1) | DISK–FV |

  Only games that the r59 order or tries touch change. In M-0001 and M-0003 every changed game is
  a side-B game, where the row type's group is now created first (M-0001: 73 of 1152 games
  changed, 29 winners; M-0003: 32 of 420, 7 winners). M-0002 also changes side-A games (15),
  where infantry moved off full cells now take the tries' draws (41 of 288 changed, 4 winners).
  Counts are on d645ca3; on 23869cf they are 73, 44 and 32 changed games.

  The 13 pairings of the 23869cf baseline, and the pairings that changed status, ours as W/L/N
  of the row type:

  | Pairing | 23869cf | i77 on 23869cf | d645ca3 | i77 on d645ca3 |
  |---|---|---|---|---|
  | LTNK–E1 | 1/11/0 fail | 2/10/0 fail | 1/11/0 fail | 2/10/0 fail |
  | TELE–APOC | 0/11/1 fail | 0/11/1 fail | 1/10/1 fail | 1/10/1 fail |
  | TELE–TTNK | 0/11/1 fail | 0/11/1 fail | 0/12/0 pass | 0/12/0 pass |
  | TELE–V3 | 0/7/5 fail | 0/6/6 fail | 0/5/7 fail | 0/3/9 fail |
  | CAOS–APOC | 1/0/11 fail | 1/0/11 fail | 1/0/11 fail | 1/0/11 fail |
  | CAOS–TTNK | 1/0/11 fail | 1/0/11 fail | 1/0/11 fail | 1/0/11 fail |
  | CAOS–V3 | 1/0/11 fail (win) | 3/0/9 pass | 3/0/9 pass | 5/0/7 pass |
  | CAOS–E1 | 0/4/8 fail | 0/6/6 fail | 0/4/8 fail | 0/6/6 fail |
  | DISK–FV | 12/0/0 fail (win) | 12/0/0 fail (win) | 12/0/0 fail (win) | 12/0/0 fail (win) |
  | INIT–E2 | 7/5/0 fail | 9/3/0 fail | 7/5/0 fail | 9/3/0 fail |
  | YURI–GGI | 3/9/0 fail (win) | 3/9/0 fail (win) | 12/0/0 pass | 12/0/0 pass |
  | YURI–SHK | 4/8/0 fail (win) | 7/5/0 fail | 12/0/0 pass | 12/0/0 pass |
  | ROBO–HTK | 0/9/3 fail | 0/8/4 fail | 0/9/3 fail | 0/8/4 fail |
  | MIND–E1 | 12/0/0 pass | 10/2/0 fail | 11/1/0 fail | 11/1/0 fail |
  | TELE–ROBO | 0/0/12 pass | 0/1/11 fail | 0/0/12 pass | 0/0/12 pass |
  | CAOS–E2 | 0/12/0 pass | 0/11/1 fail | 0/12/0 pass | 0/11/1 fail |

  On d645ca3 the one difference intake 77 makes to the failing set is CAOS–E2. One side-B game
  (seed 4) ends with no winner where E2 won before, so the side outcome is no longer unanimous;
  the win test passes. The noise test (`arena --record M-0001 --seed-range CAOS,E2,B,0-63`)
  gives E2 62 of 64 side-B games and 2 with no winner (seeds 4 and 45). *p*⁶ = (62/64)⁶ ≈ 0.83 ≥
  0.01, so this failure is within noise. CAOS–E1 gets closer to the stock result (E1 wins 6 of 12 rather than 4, stock 12):
  side B's seeds 0 and 3 now end in E1 wins. YURI–GGI and YURI–SHK pass on d645ca3 with and
  without intake 77: with intake 74's held-target check, the drawn standing positions no longer
decide them. Whether they would with the records' positions is still Q-2277's question 3.
  The padded scratch run (on 23869cf): YURI–GGI side A / side B row wins 2 / 5, YURI–SHK 1 / 5,
  CAOS–E1 column wins 4 / 5.
- **Local lane, arena tests** (on d645ca3): `m0006_bunkers_and_placed_cells_in_our_setup` and
  `m0019_creation_order_and_positions_in_our_setup` pass; `m0019_the_first_creation_draw_takes_the_raw_value_at_n0`
  fails (Q-2336).

## Checks (on d645ca3)

`cargo test --workspace --no-fail-fast`: 7355 passed, 0 failed, 120 ignored
(`quirks::tests::every_spec_quirk_has_a_switch` passes). Clippy `--workspace --all-targets -- -D
warnings` and `cargo fmt --all -- --check` are clean. `cargo xtask gate` passes. In `cargo xtask
check`, deps, determinism, cite and canary pass. Stock-guard reports 99 problems, all in measurement records
M-0004, M-0005, M-0007, M-0008, M-0013, M-0015 and M-0016. Intake 74's run on main reports the
same 99; none is in this intake's files.

## Format and goldens

No hashed state is added. The save format stays 93 and the AI game format 56. No golden moves
(the arena is not in the gate).
