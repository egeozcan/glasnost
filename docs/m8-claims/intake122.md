# M8 A3 intake 122: S-0212 r98

Brief: `.claude/worktrees/M8-A3-INTAKE122-BRIEF.md`.

S-0212 r97 → r98 (commit 1924dc4, from the r97 that intake 103 built, [intake103.md](intake103.md)).
It answers **Q-2896** (6c7096a), which intake 121 filed ([intake121.md](intake121.md)): S-0209 r76
claim 10 ended a walker's running retry delay when it was given the destination it already holds,
and S-0212 r97 kept that delay running. The delta is claim 21's new end of the delay for the walk
locomotor, its third-row sentence ("unless an assignment ends the delay first"), its backing and
confidence text, the state table's *destination changes* row, scenario P25 part e, the Open
questions entry, and three *Notes for other specs* items (S-0208 claim 6, S-0417 claim 17 and Open
question 16, S-0209 claim 10). It adds no quirk name and no Edge cases text.

Intake 121's interim (`design: Q-2896`) is replaced: every assignment that S-0209 claim 10 says is
made now also ends a walker's running retry delay when the destination given is the one it already
holds. No request was filed; Q-2906 to Q-2915 are unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| 21 (r98): for the walk locomotor, the destination it already holds, the same cell, given again by an order ends a running retry delay started in an earlier tick | tested | `path::destination_changed(.., given)`, which `infantry::destination_assigned` calls with `given` for every assignment; `claims_s0212_r97::claim_21_r98_the_same_destination_again_ends_the_delay`, `p25_e_run_1_an_order_to_the_cell_it_holds_ends_the_delay` |
| 21 (r98): the same object given again ends it | tested | `claims_s0212_r97::claim_21_r98_the_same_object_again_ends_the_delay`; `claims_s0209::r76_an_enter_order_given_again_to_an_idle_transport_restarts_it` (an enter order on the same transport) |
| 21 (r98): by a scatter | tested | `claims_s0208_r58::s0209_r76_a_scatter_to_the_walkers_own_destination_restarts_the_blockage_delay`: the delay ends, the extra locomotion searches the scatter's cell and the walker moves in the scatter's tick, as S-0212's note on S-0208 claim 6 reads it |
| 21 (r98): by any other rule | tested | `claims_s0209::r98_the_same_destination_again_ends_the_retry_delay` (`econ::assign_destination`) |
| 21 (r98): a delay started earlier in the same tick is included | tested | `claims_s0209::r98_the_same_destination_again_ends_the_retry_delay`, `claims_s0212_r97::claim_21_r98_the_same_object_again_ends_the_delay` (the delay starts and ends in one tick) |
| 21 (r98): only at an assignment that S-0209 claim 10 says is made | tested | `claims_s0209::r76_a_duplicate_enter_order_assigns_nothing`: a duplicate enter order (S-0213 T3 a) assigns nothing, and the retry delay runs on |
| 21, third row (r98): a walker that keeps its destination after a failed search waits for *R*, unless an assignment ends the delay first | tested | `path::failed` passes `given` only for claim 23a's house-selected destination, so the kept destination is given nothing; `claims_s0212_r97::claim_21_walker_kept_destination_waits_for_r` (searched at *t* + *R*). This also covers the "not an assignment" row that intake 121 left without a test of its own (its seam) |
| 21 and Open questions (r98): every locomotor other than the walk locomotor keeps the delay at the destination it already holds | tested | `claims_s0212_r97::claim_21_r98_others_keep_the_delay_at_the_same_destination` (driving and ship locomotors, in the same tick and a later one) |
| 21: a destination given as nothing to a walker that has none | built | Claim 21 names the same cell or the same object, so nothing is ended (`ob.dest.is_some()`). A walker with no destination makes no search, so either reading looks the same; the guard keeps the state as it was |
| State table, row *destination changes* (r98): the walker's same destination given again ends the delay, except for a harvester's offer of its docking cell (S-0402 claim 10a) | tested | `harvest` reads `path::running_delay` before the offer and puts it back with `path::keep_delay`, as before; `claims_s0212_r97::docking_offer_keeps_the_delay` |
| P25 part e, runs 1 and 2 | tested | `claims_s0212_r97::p25_e_run_1_an_order_to_the_cell_it_holds_ends_the_delay`, `p25_e_run_2_an_order_to_the_second_tile_ends_the_delay`: a move order in tick *c* + 10; at the end of that tick the timer has run out, or reads *R* from a computation in that tick, and the first computation after *c* comes at or after *c* + 10 and before *c* + 90. Our walker's failed search is `path::failed` on a cell that is open, so the walker walks once it searches; the "leaves its cell" void is checked only up to the order |
| Notes for other specs: S-0208 claim 6, S-0417 claim 17 and Open question 16, S-0209 claim 10 | built from S-0212 | Those specs are unchanged. `claims_s0417_r79::open_question_16_the_kept_destination_given_again` now asserts the delay ends; its `design: S-0417 Open question 16` note (later requests refused while movement is in progress) stays, as that question is S-0417's |
| `design: Q-2896` notes | removed | `path::destination_changed` and the four tests now cite `S-0212 claim 21 (r98, Q-2896)` |

## Switches

- S-0212's quirk switches: the delta has no quirk or Edge cases text, so their defaults, tests and
  C7 verdicts stand.
- Pathing design switches (C7-P4, C7-P5; decisions of 2026-10-29): none reads the retry delay, so
  each off position is r98 as written. See [c7-play-fixes.md](c7-play-fixes.md), intake 122.

## Formats

A walker given its destination again now searches at once instead of after the delay, so a saved
game or an AI game can replay differently: save format 124 → 125 and AI game format 90 → 91. No
serialized field is added.

## Lanes

Base 6c7096a (main), against this intake's code on that base (4208cab). The sim-gate goldens,
regenerated on the changed tree, did not change.

| Lane | Base | After |
| --- | --- | --- |
| Arena, `--record all` | M-0001 12/96, M-0002 2/24, M-0003 2/35, M-0019 0/6, M-0020 1/25 (YURI–SHK arm S), M-0021 11/50 | the same; the output directories are identical file for file |
| M6c sweep, seeds 1–16 | smallest minimum 0 | the same, every line |
| M7a sweep, seeds 1–16 | smallest minimum 1 (seed 2) | the same, every line |
| M7b sweep, seeds 1–16 | smallest drain minimum 3 | the same, every line |
| `glasnost-ai --include-ignored` | 264 pass | 264 pass |
| AI quick lane, seeds 7–10, maps 0, 2, 8 | 36 games | the same winner in all 36; 35 end at the same tick. Seed 7, map 2, sides 0 and 1: house 0 wins at tick 42035 instead of 42866 |

The one game that changed keeps its winner, so it was not traced to its first differing tick (the
brief asks for a trace only when a winner changes). The only rule this intake changes is the
walker's retry delay at its own destination given again, so that is where it must first differ.
