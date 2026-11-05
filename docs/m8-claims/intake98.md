# S-0306 r80 and S-0301 r73: claim ledger, M8 step A3 intake 98

- **S-0306** r76 to r80 (commit 948100b, from the r75 that intake 94 built). r80 answers Q-2526
  (claim 17's rest after the hold for an aircraft whose type cannot land, P46 run 2's post-hold
  rest), which intake 94 filed and left as a seam with no interim in code. At the expiry search's
  fall-back a visiting aircraft member on the ground takes *retreat* (S-0501 table T2); a
  destination given while an aircraft member phases in, its arrival hold included, is dropped as
  soon as it is given; its first destination that holds comes at most 3 ticks after its hold
  ends, at a *retreat* processing, a cell on its house's arrival edge. New quirk
  `quirks.warp_hold_end_forgets_destination` (default on): in the tick *h* in which the hold ends,
  a destination the behaviour processing gave is forgotten and starts no take-off. P46 run 2 is
  redefined: no trigger, A's own *retreat* flight is the run, recorded per tick from *t_L*; parts
  c2 and d2 run in run 1 only.
- **S-0301** r72 and r73 (commit 5d4e925, from r71). r73 answers Q-2567 (seam sweep 5b): every
  answer of S-0405 table T1 ranks above the held-temporal-target *rearming*. A building whose beam
  holds its target drops that target while a row of T1 holds, at the first of its next *attack*
  processing (S-0215 claim 32's row: *guard*) and its checking turn in a check tick of S-0309
  claim 29, and the beam stays attached; on the map and for a departed warp member. No quirk.
  Scenario F22 parts a (switched off), b (short of power) and c (control).
- One quirk is added: `warp_hold_end_forgets_destination`, a switch of `quirks.rs` with the spec's
  default, both positions tested, judged in [c7-play-fixes.md](c7-play-fixes.md) (rejected for
  the layer). `quirks::tests::every_spec_quirk_has_a_switch` passes again (it failed on main
  since 948100b named the switch).

Requests filed: none. Q-2606 to Q-2615 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The drop while phasing in** (`air::destination_taken`). An aircraft that is a superweapon
  warp member and phases in (`warp::phasing`: departed, placed or in its arrival hold) has a
  destination it is given dropped at once, whatever gave it: the behaviour processing
  (`econ::assign_destination`) or an order (`orders.rs`, which sets the destination and then
  calls the same hook). Nothing else of the hook runs for that destination.
- **The forget at the hold's end** (`air::hold_end_forget`, called in the object turn in
  `tick.rs` right after the behaviour processing when this turn's hold-end test ended the
  membership). With the switch on, a destination the processing gave in that turn is forgotten
  before the movement update, so it starts no take-off; an order's destination, applied after the
  turn pass (S-0203 claim 7), is not touched. Off: nothing is forgotten and the aircraft takes off
  at its movement update in that tick. Aircraft only: for another kind the switch is not defined.
- **The expiry fall-back's *retreat***: already so since intake 94 (`teleport::phasing_search` →
  `process::fall_back` → S-0501 T2). Only tests and citations change.
- **S-0301 r73 claim 9**: `combat::check` already asks T1 (`econ::power::firing_answer`) before
  the held-temporal-target *rearming*, and a building's fall-back to *guard* lets no beam go
  (`process::fall_back` returns for an object that is not mobile, so S-0215 claim 13's common
  effects, where the beam is let go, never run for a building).
- **S-0309 claim 2's temporal condition** (`combat::acquire::may_search_passively`). F22 part a
  failed first: at the dropping *attack* processing the building started *guard*, and the per-tick
  search after that processing (S-0215 claim 33 (b)) took the erased target back in the same
  update, so "just after that update it holds no target" did not hold. Claim 2 forbids passive
  acquisition while "a temporal weapon of its own is erasing a target", which our engine had not
  built (claim 2 marks it `unmeasured`, since a mobile firer that loses its target lets the beam
  go). It is now built: no range or area search while the searcher's beam is attached to an object
  that is being erased (`temporal::erased`). A departed warp member is no longer being erased
  (S-0513 claim 39a), so a building that drops a departed member may search, as S-0309 claim 27
  says ("after the drop it takes targets as any building does").

No hashed state is added. The behaviour change alters what a saved game replays to: save format
99 → 100, AI game format 62 → 63.

## Goldens

Regenerated (`GLASNOST_UPDATE_GOLDEN=1`, sim-gate `--include-ignored`): only `duel-m6c` moves,
from line 81. Its sweep's chrono-erasure counters move too (below), which points at S-0309 claim
2's search condition; the two changes were not run apart. `combat`, `duel`, `duel-diagonal`,
`duel-naval`, `duel-m7a`, `duel-m7b`, `scripted` and `skirmish` do not move. `cargo xtask gate`
(native and wasm32) matches all nine.

## Claims

### S-0306 r80

| claim | status |
|---|---|
| 17, at the wait's expiry a visiting aircraft member on the ground with no passengers falls back to rest and takes *retreat*; the fall-back at the hold's end leaves it | Tested: `claims_s0306_r62::p46` (run 2 asserts *retreat* after the expiry), `r80_claim17_p46_run_2_retreat_from_the_expiry_and_a_first_destination_on_the_north_edge` |
| 17, a destination given while an aircraft member phases in, its arrival hold included, is dropped at once | Tested: `r80_claim17_a_destination_given_while_an_aircraft_member_phases_in_is_dropped` (a held delay of 12, so *retreat* processes in the hold; every phasing row has no destination and height 0), `r80_claim17_a_move_order_while_an_aircraft_member_phases_in_does_not_hold` (an order in the hold) |
| 17, the first destination that holds at most 3 ticks after the hold's end, a cell on the arrival edge; the take-off by the end of the next tick | Tested: `r80_claim17_p46_run_2_…` (the north edge by S-0522 F2; the destination at *h* + 3, height above 0 by *h* + 4) |
| 17 and Edge cases, `quirks.warp_hold_end_forgets_destination`: on, the destination of tick *h* forgotten, no take-off; off, it holds and the aircraft takes off | Tested: `r80_quirk_warp_hold_end_forgets_destination_both_positions` (and the default on) |
| 17, for a member of another kind the switch is not defined | No change: `air::hold_end_forget` returns for an object that is not an aircraft |
| 17, which of the warp's updates process an aircraft member's behaviour (S-0513 claim 51's design rule) | No change: claim 51's rule as built (behaviour processing only in the hold, after the hold-end test) |
| P46 run 2 redefined: no trigger; A's own *retreat* flight; parts c2 and d2 in run 1 only | Tested: `p46()` run 2 now takes off by its own *retreat* (`take_off_in`); `r75_claim17_p46_run_2_…` asserts *retreat*, no destination and height 0 at the end of *h*; the airborne parts a, c and d run on that flight |

### S-0301 r73

| claim | status |
|---|---|
| 9, every T1 answer ranks above the held-temporal-target *rearming* | Tested: `claims_s0301_r73::r73_claim9_t1_answers_rank_above_the_held_temporal_rearming` (in operation: *rearming*; switched off: *impossible*) |
| 9, F22 part a: switched off, the drop at the first of the *attack* processing (*guard*) and the check tick, the beam kept | Tested: `r73_claim9_f22_part_a_…` (10 runs, *w* at both parities and several distances to the check tick) |
| 9, F22 part b: short of power and not charged | Tested: `r73_claim9_f22_part_b_…` (10 runs, a second building switched on after the lock) |
| 9, F22 part c: the control keeps the target, *attack* every other tick, no second shot, the beam kept | Tested: `r73_claim9_f22_part_c_the_control_keeps_the_target_and_the_beam` |
| 9, the same for a departed warp member (seam sweep 5b's row of intake 84) | Tested: `claims_s0513_r89::r73_claim9_a_switched_off_building_firer_drops_a_departed_member_and_starts_guard` (switched off at the departure tick; dropped off the map at a processing or a check tick, *guard* at a processing, the beam kept) |
| 9, the other T1 rows (drained, under a pulse, occupiable without occupants, *construction*, *selling*) against the held rearming | Partly: built by the same rank (one `firing_answer` call); not run, as the spec marks them `unmeasured` |
| S-0309 claim 2, no passive search while its own temporal weapon erases a target | Tested through F22 part a (it failed before: the per-tick search took the target back in the dropping update) |

## Lanes

Before: main e15a93b (binaries built there); after: this intake's code commit on that base
(11f925f), before the rebase onto main 4051d83 (which gained the S-0215 claim 16 seam, no
format change); same machine, release. After the rebase the goldens, the sweeps, the test suites
and the gate were run again (below).

| lane | before | after |
|---|---|---|
| M6c sweep, seeds 1 to 16 | smallest counter 0; seeds 5, 9 and 12 end early | smallest counter 0; minima and medians move (chrono erasure orders min/median/max 17/32/40 → 15/30/53, erasures begun 6/11/18 → 5/13/21); seeds 2, 5, 9 and 12 end early |
| M7a sweep, seeds 1 to 16 | smallest counter 0 | identical minima and medians |
| M7b sweep, seeds 1 to 16 | smallest drain minimum 3 | identical minima and medians |
| arena (`--record all`) | 13 of 155 pairings fail; M-0021 3 of 41 rows | identical output and recorded files: the same 13 (ROBO–HTK, TELE–APOC, TELE–V3, TELE–ROBO, TELE–BFRT, CAOS–E1, DISK–FV, INIT–E2, MIND–E1, CAOS–APOC, CAOS–TTNK, CAOS–E2, YURI–SHK), none changed; M-0021 the same 3 of 41 |
| `glasnost-ai --include-ignored` (33 binaries) | 248 passed | 248 passed |
| AI quick lane, seeds 7 to 10 × sides 0,1 / 0,2 / 1,2 × maps 0, 2, 8 | 36 of 36 games over with a winner | 36 of 36 over with a winner |

The quick lane: 34 of the 36 games are identical. Two change, both on map 2: seed 9, sides 0
and 2 (house 0 won after 68 804 ticks before, house 1 after 49 964 after) and seed 10, sides 0
and 1 (house 0 won after 45 682 ticks, then after 46 184). Erasure orders over the lane went from
74 to 88; Chronosphere launches stay at 14. The M6c and quick-lane moves come with more erasure
orders, which points at S-0309 claim 2's search condition (a chrono firer that keeps its beam
without a target no longer searches) rather than the aircraft member's hold; the two were not run
apart.
