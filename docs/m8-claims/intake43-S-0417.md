# S-0417 r68: claim ledger, M8 step A3 intake 43

This intake takes in one spec revision:

- **S-0417 revision 68** (spec commit a7dc3d7; the Q-1980 answer is commit 1b56e06). Our engine
  built revision 64 at intake 35 ([intake35-S-0417.md](intake35-S-0417.md)).

It answers **Q-1980** (claim 17's request to a walking unit that holds a destination with no
path), so intake 35's interim design for it is retired. The specs it names were read as they
stand: S-0208 claims 11 and 24, S-0212 claims 6, 21 and 23 (revision 87; its own changes are
intake 44, [intake44-S-0212.md](intake44-S-0212.md)), S-0215 claim 12.

## What r68 changes

Only claim 17, in a new paragraph, *No path*, and its backing, confidence and scenario U10 g.

- *Waiting for the retry delay*: a walking unit that holds a destination it has not yet searched
  for has movement in progress from the moment the destination reaches it, so S-0208 claim 11's
  third case refuses each request during the wait, with no draw, and the destination is kept
  (the fraidycat exception of *During the walk-off* still applies).
- *Failed at the reachability gate*: a walker's fresh search that fails at the gate clears its
  destination in the same movement update and ends its movement in progress. Its behaviour is
  unchanged (*guard* or *area guard*), and the first request after it is accepted with its draw.
- *Failed search*: a fresh search that passes the gate and finds no path also ends the walker's
  movement in progress, whether or not it keeps the destination (S-0212 claim 23).
- *The cycle*: a destination accepted after a failed search waits for the rest of *R*; one draw
  per failed fresh search, not one per turn; with *R* = 0, one draw per turn.
- U10 g (new): a walker given an island cell after its release and before its first turn; run 2
  sets the path-retry delay to 0.1 game minutes (*R* = 90).

U6 and U15 are not touched by r68 and still read the old rule that every destination teleports,
which S-0207 claim 28 overrides (intake 39 named them); they stay stale.

## Code

- `path::PathState::failed_for` (new, saved and hashed): a walker's destination at the failed
  search that ended its movement in progress while it kept that destination. Set in
  `path::failed`'s search-failure branch (S-0212 claim 23 rows 2 and 3; after row 2's give-up it
  is the cleared destination, `None`), cleared by a found path (`compute_untraced`) and by every
  destination that reaches the locomotor (`infantry::destination_assigned`).
- `occupancy::in_progress`: a walker with a destination other than `failed_for` has movement in
  progress (the *waiting* case), on top of the old tests (a walking target, a stored path, the
  other locomotors). Every caller of `in_progress` sees it: the S-0107 term is one.
- `occupancy::moving_on_tick` (new): the old body of `in_progress`, now read only by
  `process::moving`, S-0215 claim 11's readiness test ("moving on this tick").
- `path::failed`, gate branch: a walker's fresh search that fails at the gate clears its
  destination (claim 17, *Failed at the reachability gate*). The re-path case is S-0212 r87's and
  is built in intake 44.
- `scatter::answer`: intake 35's `holds_dest` interim (design, Q-1980) is gone; the third case
  reads `occupancy::in_progress` alone.
- Save format 83 to 84, AI game format 45 to 46 (the new field).

**Readiness.** S-0215 claim 11 asks whether an infantry unit is "moving on this tick", which is
S-0107's *is moving now* ("true while the unit is physically moving on this tick"; S-0107 Terms
tell it from *has movement in progress* and say the two differ while a unit starts). So
readiness reads `moving_on_tick`, and a walker that waits for its search, with no walking target
and no stored steps, is not moving now. With `in_progress` there a queued behaviour would never
start for such a walker (an *enter* order in S-0605's absorber tests), since its search waits
for the behaviour.

**Design** (r68 leaves it open): a destination reaches the locomotor in
`infantry::destination_assigned`, which orders, behaviours, economy assignments
(`econ::assign_destination`) and claim 17's answers go through. A direct write of another
destination also ends the failed state, since `failed_for` is compared with the destination;
only the same destination written directly, without that call, would leave it.

The other readers of `in_progress` see the waiting walker as moving too, where their spec says
*movement in progress*: S-0209 claim 4 rows WR4 and WR5 (the still allies in a full cell), the
fear rules (S-0107 claim 15, behind their destination test). The vehicle-only readers (reload,
disguise, lead) do not reach walkers.

**Play-fixes layer (C7).** Not named. The new refusals make a walker on a factory's cell wait out
the rest of *R* before it walks off. That is a delay, not a unit stuck, and the original does it;
the layer's admission rule (`data/layers/play-fixes.jsonc`) is not met.

## Claims

| Claim or item | Status | Notes |
|---|---|---|
| 17 *No path*, waiting for the retry delay (movement in progress, refused with no draw, the destination kept) | tested | `claim_17_gate_failure_clears_then_accepts` (89 refusals, no draw, the same destination, then the walk off), `claim_17_destination_without_a_path_is_kept` (r64 test, now asserts movement in progress during the wait) |
| 17 *No path*, failed at the gate (cleared in the same update, behaviour unchanged, the next request accepted with its draw; *R* from the failure) | tested | `claim_17_gate_failure_clears_then_accepts` (U10 g run 2's expectations, *R* = 90) |
| 17 *No path*, failed search (movement in progress ends, the destination kept within 1 cell; accepted with its draw) | tested | `claim_17_failed_search_kept_destination_accepts` |
| 17 *No path*, the cycle (one draw per failed fresh search; failures *R* apart) | tested | `claim_17_one_draw_per_failed_search` (a water ring round the barracks, 400 ticks) |
| 17 *No path*, *R* = 0 (one draw per turn) | tested | `claim_17_no_delay_one_draw_per_turn` |
| 17 *No path*, the fraidycat exception | as built | unchanged from intake 35 (`scatter::answer`'s third case) |
| 17, a new destination does not cut *R* short | tested | for claim 17's own destination: the wait in `claim_17_gate_failure_clears_then_accepts`. Orders and other assignments: S-0212 r87's claim 21, built at intake 44 (`claim_21_walker_new_destination_waits_for_the_delay`, `state_table_drive_new_destination_waits_for_the_delay`, [intake44-S-0212.md](intake44-S-0212.md)); stale "partly" closed at seams sweep 2. |
| Backing and confidence text | text | |
| U10 g | not ours | Oracle scenario; needs input in the release window. |
| U6, U15 | stale, named | still read "every destination teleports" (S-0207 claim 28 overrides it) |
| Q-1980 interim (intake 35) | retired | |

Tests changed by the rule:
- `claims_s0209::stationary_friendly_far_asks_them_to_move` and `stationary_friendly_contact_asks`:
  the friend's scatter destination lies on the corridor's raised side and fails the gate in its
  own turn, so the destination is cleared the same tick; the tests now read the acceptance from
  the trace.
- `claims_s0210::drive_at_infantry`: the same, for the infantry on the brush cell.
- `drive::tests::duplicate_request_in_one_tick` (helper `after_requests`): the walker's
  destination is cleared between the two requests, so the second one is not refused by claim
  11's third case.

## Before and after

- Goldens (`glasnost-sim-gate`): the new field moves every hash from tick 0 (its own commit, with
  the format bumps, so the rule's moves show alone). The rule then moves two scenarios, both
  traced to the waiting term of `in_progress`:
  - duel-m6c from tick 97: an engineer on its way to a capture stands between the end of its
    stored path and its next search, still holding its destination, and refuses a scatter
    request it used to accept (S-0208 claim 11's third case);
  - skirmish from tick 26871: three infantry just ordered on stand in a full cell; a vehicle
    stepping in now sees them as moving (S-0209 claim 4 row WR5, moving blocker) where it saw
    three still allies (WR4, stationary friendly).
  The other seven scenarios are unchanged.
- Sweeps (M6c, M7a, M7b, seeds 1 to 16, ref and fix): M7a unchanged; M7b five counters, M6c 22
  of 29 (the game goes its own way after the engineer at tick 97; the largest maxima drop, for
  example deploy orders 238 to 156 and radiation exits 145 to 70, and the medians move by a few).
  Every seed still exercises every counter and no run ends early. The play-fixes sweeps move
  exactly as the reference ones. Tables in `local-lane/m8/a3-43/` (before, after43).
- Arena (`--record all`): M-0003's CSV is byte-identical; M-0001 changes 7 rows of 1152 and
  M-0002 1 of 288, all with an infantry column (E1, E2, INIT). Failing pairings stay 30 of 96,
  12 of 24 and 2 of 35.
