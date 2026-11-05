# S-0212 r97, S-0417 r79, M-0023: claim ledger, M8 step A3 intake 103

- **S-0212** r87 to r97 (r96 at commit 295e0f9, r97 at d5c17da). Claim 21 is rewritten: for the
  driving and walk locomotors a running retry delay ends when the destination is changed to a
  different one or cleared, a delay started earlier in the same tick included, so no delay remains
  after a failure that clears the destination (claim 23). The ship and the other locomotors keep it;
  the hovering locomotor's is S-0503's, unchanged; the offers of a refinery's docking cell are
  S-0402 claim 10a's. The delay after a follower car's computation is no longer claim 26's
  (claim 26 stays a seam). The Open questions say that giving a unit the destination it already
  holds keeps the delay running. The rest of r96 is rewrapped text. r97 rewrites only the
  measurement note on U10 part g.
- **S-0417** r68 to r79 (r76 at bc8ebbf, r79 at 20a60e6). Claim 17, *No path*: a different
  destination ends the delay; a walker that keeps its destination after a failed search waits for
  it, but its movement in progress ended at the failure, so the first request after it is accepted
  with its draw; in the cycle, each failing destination that differs from the kept one is searched
  for without a wait, so it takes a draw in each turn with a request. Open question 16: whether
  later requests are refused where a request gives the kept destination. U10 part g's run 2.
- **M-0023** (bd5d144), new, `value-visibility: local`: U10 part g run 1, with its controls and its
  baseline (part b on its map). Its values are read at run time from the git-ignored
  `measurements/local/M-0023.json`; no tick, cell or type of it is in a tracked file.
- No quirk is added or renamed in either spec, so no new switch. C7 in
  [c7-play-fixes.md](c7-play-fixes.md), *Intake 103*: the six own pathing improvements are
  unchanged in what they fix; `quirks.detour_mirrored_reference` and
  `quirks.path_refresh_keeps_old_step` keep their defaults (r96 only rewraps their lines). The
  layer file is unchanged.

Requests filed: Q-2656 (why *guard* is queued at the end of the first turn of a released walker
whose ordered search fails at the gate, and *G* − *F* = 1; M-0023's row for the end of *F* and its
*G* − *F*), Q-2657 (the first step one tick after the scatter request's turn, and the controls'
walk to the old destination while the destination reads the pen's centre). Q-2658 to Q-2665
unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The delay ends at a change** (`path::destination_changed`, `path::delay_runs`, `path::waits`,
  `path::start_delay`; new field `PathState::retry_for`, hashed and saved). A retry delay started
  by a driving or walking unit remembers the destination it was started for. It runs while that
  destination is still the unit's; a destination changed to a different one, or cleared, ends it,
  whether it was started in an earlier tick or earlier in the same one (`path::failed` clears it
  after a failure that clears the destination, so no delay remains). Giving the same destination
  again keeps it running (S-0212 Open questions). Ships and the other locomotors keep the old rule:
  the delay runs until its tick. The hovering locomotor's delay is S-0503's and unchanged.
- **The gates** in `drive.rs` (the fresh search, the T4/T5 checks, `blocked_step`) and `walk.rs`
  (the fresh search, the moving-blocker case, the re-path) read `path::waits` and
  `path::delay_runs` instead of comparing the tick with `retry_at`. A walker's re-path restarts *R*
  through `path::start_delay`, unless the failure clears the destination.
- **The docking offer** (`econ::harvest`): S-0402 claim 10a's offer of the docking cell keeps a
  running delay through the change of destination (`path::running_delay`, `path::keep_delay`), as
  that claim waits for it "like any fresh search".
- **Infantry** (`infantry::destination_assigned`): a destination accepted from a request calls
  `path::destination_changed`, so the request's different destination ends the delay of a walker
  that kept its destination after a failed search; the kept destination given again keeps it
  (S-0417 Open question 16, our reading through S-0212's Open questions).
- The 18 comment sites of intake 44 (`depot`, `transport`, `orders`, `structures::bunker`,
  `structures::grinder`, `econ::slaves`, `structures::absorb`, `econ::harvest`) cite r97's claim 21.
- **The M-0023 check** (`glasnost_tools::oracle_m0023`, test `m0023_rows_hold_in_our_engine` in
  `crates/tools/tests/arena.rs`, local lane). It reads the local values file
  (`GLASNOST_M0023_LOCAL`, by default `measurements/local/M-0023.json`), adds the ring of wall
  sections to M-0001's map, plays the three arms (order before the first turn, order two ticks
  after the release, no order), derives *F*, *G* and *W* as the record's procedure does and judges
  each Result row. The summary (row and verdict) goes to stdout; the report with our readings,
  which holds local values, goes to the git-ignored `local-lane/m7b/arena/oracle-M-0023.txt`. It
  skips without the install or the values file.

## Claims

Tests in `crates/deterministic/sim/tests/claims_s0212_r97.rs` and `claims_s0417_r79.rs` unless
named otherwise.

| Claim | Status | Tests and notes |
|---|---|---|
| S-0212 21: a walker's gate failure leaves no delay | tested | `claim_21_walker_gate_failure_leaves_no_delay`; `claims_s0212_r87::claim_23_walker_repath_gate_failure_clears` and `p7_run_7b_walker_to_an_island` now assert no delay |
| S-0212 21: a walker that keeps its destination waits *R* | tested | `claim_21_walker_kept_destination_waits_for_r` |
| S-0212 21: a different destination ends the delay | tested | `claim_21_walker_different_destination_ends_the_delay`, `claim_21_drive_repath_delay_ended_by_a_new_order` |
| S-0212 21: a stop (destination cleared) ends it | tested | `claim_21_a_stop_ends_the_delay` |
| S-0212 21: driving failures that clear leave no delay | tested | `claim_21_drive_failures_that_clear_leave_no_delay`; the older `claims_s0212::retry_delay_after_fresh_searches`, `search_failure_far_away_gives_up` and `search_failure_within_one_cell_does_not_give_up` now assert no delay after the driving unit's gate failure and rows 2 and 3 |
| S-0212 21 / Open questions: the same destination again keeps the delay | tested | `claim_21_the_same_destination_again_keeps_the_delay` |
| S-0212 21: the ship keeps its delay | tested | `claim_21_ship_keeps_its_delay` |
| S-0212 21 / S-0402 10a: the docking offer keeps the delay | tested | `docking_offer_keeps_the_delay` |
| S-0212 21: the hovering locomotor | no change | S-0503's |
| S-0212 26: follower cars | seam | unbuilt, as before |
| S-0417 17, *No path*: the gate failure, then the walk without a wait (U10 g run 2) | tested | `u10_g_run_2_gate_failure_then_the_walk_without_a_wait` (move order before the first turn; the first step as many ticks after the accepted request as part b's, under 88 ticks after the failure); `claims_s0417_r68::claim_17_gate_failure_clears_then_accepts` rewritten for no delay |
| S-0417 17: the kept destination's first request is accepted and ends the delay | tested | `claim_17_kept_destination_request_ends_the_delay` |
| S-0417 17: a delay running at the first request is ended by its destination | tested | `claims_s0417_r64::claim_17_running_delay_ended_by_the_request` (was r68's `claim_17_destination_without_a_path_is_kept`: the walker now steps at once and walks off in the same tick as one with no delay) |
| S-0417 Open question 16: the kept destination given again | tested (our reading) | `open_question_16_the_kept_destination_given_again` |
| S-0417 17: every differing failing destination takes a draw each turn | tested | `claim_17_every_failing_destination_takes_a_draw_each_turn` (a moat, *R* = 90) |

The two r87 wait tests of claim 21 (`claims_s0212_r87`) and `claims_s0417_r68::claim_17_one_draw_per_failed_search`
tested the delay that r97 and r79 end; they are removed, with a comment in their place.

### M-0023, row by row

Run 1 against our engine (local lane, the stock install; run 2 does not count, by the record).
"Holds" means our reading equals the record's.

| Result row | Ours | Notes |
|---|---|---|
| Void rules of step 2 | holds | ring, pen, release, order and K's cells |
| *U* on a cell that holds K at *t* + 1 | holds | every arm |
| At *t* + 1: *guard* current, a new destination | holds | run 1 no; controls and baseline yes |
| *F* − *t* | holds | 1 |
| At the end of *F*: *move* current, *guard* queued, no destination | **fails** | ours: *move* current, nothing queued, no destination. Q-2656 |
| *G* − *F* | **fails** | ours: the standard delay of S-0215 claim 22, not 1. Q-2656 |
| At the end of *G*: *guard* current, nothing queued, a new destination | holds | |
| New destination on a cell that holds K | holds | no |
| New destination in the pen or next to it | holds | no |
| Steps before *W* | holds | none |
| First step after *G* (the baseline: after its request turn) | **fails** | ours 2 ticks, run 1 and baseline alike, so S-0417 claim 17's relative form ("as soon after as the baseline's") holds. Q-2657 |
| *U* enters the pen within 300 ticks | holds | no |
| Controls: positions as the baseline's; from *t* + 2 the pen's centre and *move* | **fails** | ours: the order at *t* + 2 comes before our unit has searched (the lag of Q-2657), so its search for the pen fails at the gate and it waits; in the record it walks the baseline's path with the pen's centre as its destination. Q-2657 |

The check expects exactly these four rows to fail until Q-2656 and Q-2657 are answered; any other
failing row fails it. The engine is unchanged for them.

## Goldens

(filled in at the rebase)

## Lanes

(filled in after the runs)
