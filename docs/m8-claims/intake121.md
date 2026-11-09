# M8 A3 intake 121: S-0209 r76

Brief: `.claude/worktrees/M8-A3-INTAKE121-BRIEF.md`.

S-0209 r60 → r76 (commit 7ef82d3, from the r60 that intake 114 built, 1a30a5f). It answers
**Q-2822**, the same destination given again and the blockage delay, which intake 114 filed with
an interim ([intake114.md](intake114.md)). r61 to r75 were never committed on their own: the one
spec commit in the range that touches S-0209 is r76, and no ledger builds from a revision in
between. The delta is claim 10's restart rule and its new part *A destination given again*, two
new Open questions entries, scenario C5 part c2, and three Cross-spec coordination items (S-0212
claim 21, S-0208 claim 6, S-0507 claim 5 with S-0213 claim 21). It adds no quirk name and no Edge
cases text.

The interim is replaced: every assignment that is made starts a walker's blockage delay afresh,
the destination it already holds included (`path::destination_changed` with `given`, and the
`design: Q-2822` note is gone). One part contradicts S-0212 r97, which owns the retry delay, and
waits on **Q-2896** (filed, interim below). Q-2897 to Q-2905 are unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| 10: the blockage delay starts afresh at every assignment that is made, the destination the walker already holds (the same cell or the same object) included | tested | `path::destination_changed(.., given)`, called with `given` by `infantry::destination_assigned`; `claims_s0209::r76_another_rules_assignment_of_the_same_destination_restarts_it` |
| 10, C5 c2: an order to the cell that is already the walker's destination restarts the delay; the wait counts from the first *moving blocker* answer after the order, and the search that avoids the blocker comes no earlier than *D* after it | tested | `claims_s0209::c5_c2_the_same_destination_again_restarts_the_blockage_delay` (through the trace: a control without the order makes its first avoiding search, mode 2, at the first re-path at least *D* after the first stop; with the order it comes no earlier than *D* after the restart, and later than the control's) |
| 10: an accepted scatter whose cell is already the walker's destination restarts it (S-0208 claim 6) | tested | `scatter::request` → `destination_assigned`; `claims_s0208_r58::s0209_r76_a_scatter_to_the_walkers_own_destination_restarts_the_blockage_delay` |
| 10, object destinations: an attack order on the walker's target given again makes the destination nothing (S-0213 T3 f), so the wait ends | tested | `orders` (unchanged); `claims_s0209::r76_an_attack_order_given_again_ends_the_wait` |
| 10: a duplicate enter order (T3 a) assigns nothing, so the wait counts on | tested | `orders` (unchanged); `r76_a_duplicate_enter_order_assigns_nothing` |
| 10: a tethered walker's enter order given again that is not a duplicate keeps the link and contact (T3 b), the assignment is made and the stored path is dropped (S-0208 claim 6 for a walker in contact) | tested | `orders`, `walk::keeps_stored_path` (unchanged); `r76_a_tethered_walkers_enter_order_given_again_restarts_it` |
| 10: an enter order given again by a walker with no contact to an idle transport with room, no destination and no other contact, on land in its reach region: the assignment is made | tested | `r76_an_enter_order_given_again_to_an_idle_transport_restarts_it` (our engine assigns the transport object) |
| 10 and Open questions: whether that destination is the transport or its cell | seam | S-0209 Open questions (*The destination at an accepted enter order*). Our order gives the transport object, and `transport::load_request`'s call gives the transport object too; either way the restart holds |
| 10 and Open questions: an enter order given again to an object that holds another contact, to one S-0507 claim 5's tests do not let the walker board, or to a vehicle transport with no destination on water or in another reach region | seam | S-0209 Open questions (*An enter order's assignment to a walker with no contact*, low, `unmeasured`). Our order assigns the object in every one of these cases, so the delay restarts. The recorded original differs at the transport on water or in another reach region: there no destination is assigned and the wait goes on counting. For a vehicle or building holding another contact, the recorded original clears the destination, which also ends the wait. The absorbing building is S-0605 claim 3's, as before. Nothing is built from these low results |
| 10: the same assignment also ends a running retry delay of the walker | blocked (Q-2896), interim built and tested | Contradicts S-0212 r97's Open questions, which keep the delay for every locomotor. `// design: Q-2896` in `path::destination_changed` (S-0212's text holds). `claims_s0209::q2896_interim_the_same_destination_again_keeps_the_retry_delay`; `claims_s0212_r97::claim_21_the_same_destination_again_keeps_the_delay` and `claims_s0417_r79::open_question_16_the_kept_destination_given_again` now cite the interim too |
| 10: not an assignment: a failed search after which a walker keeps its destination (S-0212 claim 23 row 3) leaves the blockage delay running; claim 23a's house-selected destination is an assignment | built | `path::failed` passes `given` only for claim 23a's destination; with `given` false the call behaves as before this intake (the delay restarts only when the destination differs). No test of its own: the re-paths of the claim 10 scenes all find a route. A seam for a later test |
| 10: a destination written without the hook | no change | `PathState::blocked_for` still restarts the delay for a different destination (`a_destination_written_by_another_rule_restarts_the_blockage_delay`); a same-destination write outside the hook is not an assignment in our engine (every rule that assigns calls the hook) |
| Cross-spec coordination: S-0212 claim 21, S-0208 claim 6, S-0507 claim 5, S-0213 claim 21 | not ours | Text for the spec side; the S-0212 item is the subject of Q-2896 |
| S-0209 switches (`occupancy_house_side_positions_only`, `occupancy_house_stale`, `stationary_friendly_exactly_three`, `occupancy_override_hides_later_blockers`, `walker_attacks_unhittable_obstacle`, `crusher_overlay_index0_clear`, `crusher_entry_ground_layer`, `cell_layer_split_on_uneven_deck`, `walker_targets_allied_wall`, and the shared S-0207/S-0209 teleport reservation switches) | no change | The r60 → r76 delta has no Edge cases or quirk text; defaults, both-position tests and C7 verdicts stand |
| Pathing design switches (C7-P4, C7-P5; decisions of 2026-10-29) | no change in either position | None decides claim 10's restart. With the play-fixes layer on, C7-P4's `stalled` test reads `blocked_since`, so a walker given its destination again is not stalled until the delay has run from its next wait; C7-P5's yield clears `blocked_since` itself and resumes a destination that differs from its refuge cell, so it is unchanged |

## Formats

The restart changes how a walker given its destination again behaves, so a saved game or an AI
game replays differently: save format 123 → 124 and AI game format 89 → 90 (one past intake
119's, the numbers the brief reserves). No serialized field is added.

## Lanes

Base 16df36f (main before intake 119), against this intake's code on that base (before the
rebase onto f66fd39). Every lane is identical: no game in them gives a walker its destination
again while it waits behind a moving blocker in a way that changes the outcome.

| Lane | Base | After |
| --- | --- | --- |
| Arena, `--record all` | M-0001 12/96, M-0002 2/24, M-0003 2/35, M-0019 0/6, M-0020 1/25 (YURI–SHK arm S), M-0021 11/50 | the same; the output directories are identical file for file |
| M6c sweep, seeds 1–16 | smallest minimum 0 (seed 1) | the same, every line |
| M7a sweep, seeds 1–16 | smallest minimum 1 (seed 2) | the same, every line |
| M7b sweep, seeds 1–16 | smallest drain minimum 3 | the same, every line |
| `glasnost-ai --include-ignored` | 264 pass | see the hand-back report (run on the rebased tree) |
| AI quick lane, seeds 7–10, maps 0, 2, 8 | 36 games | the same winner at the same tick in all 36 |
