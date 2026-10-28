# S-0501 r62: claim ledger, M8 step A3 intake 31

This intake takes in one spec revision, which answers three requests, so three interims are
retired: Q-1940 (does a person's order end a pad-bound contact outside the foundation?), Q-1941
(do *At height 0*'s exemptions hold for any aircraft at height 0, or for the low release only?)
and Q-1942 (does claim 40's replacement at height 0 make the low release take off?).

- **S-0501 revision 62** (spec commit 7a37668; the requests answered in 12f3b0c, b433f64 and
  d827c27). It covers r60 to r62 against r59, which intake 28 built.
  - Claim 40: whether kept or replaced, the selected destination is given to the aircraft at the
    move's first processing, and the move's next processing gives its flight point once more.
    Each is a destination in claim 8's sense, so an aircraft then on the ground by its height, or
    landing, takes off.
  - Claim 48 (c) *Low release on a move* (new) and *At height 0* (rewritten): a low release on a
    move (an order or the release's rally point) takes off at the move's first processing, before
    its second movement update, so it makes no movement update at height 0 with a destination but
    without a take-off. The rules at height 0 are those of claims 5, 6 and 42 for every aircraft.
  - Claim 48 (c) *Pad contact* (rewritten): a behaviour order ends a pad-bound type's contact
    wherever it docks; claim 8's cell rule ends it otherwise. The rally move at the release is not
    a behaviour order: the release assigns the destination and queues *move* (claim 48 (d)).
  - Claims 5 and 47 (a) to (c), claim 48 (b) and (e), the Overview's answers in full, the Edge
    cases row of `quirks.aircraft_released_above_pad`, AC30 part k (4) and (7), Open question 26
    and Changes for other specs (S-0404 claim 13) follow.

The cross-specs it names were read as they stand: S-0213 table T3 (b) and claim 5, S-0417 claim 11,
S-0215 claim 3 (b), S-0404 claims 13 and 15, S-0102 claim 4 and S-0206 claim 1. None of them
changed. Seam: S-0501's Changes for other specs says S-0404 claim 13's aircraft factory row
"could say" that the rally move is the release's assignment with a queued *move*; S-0404 has not
been revised, and the code follows S-0501 claim 48 (d).

**Code.**
- `air::give_point` (new): the locomotor's acceptance of a flight point, moved out of
  `tick_flight` unchanged (flying-to, claim 6's raise fixed now, the unfinished flight, and claim
  8's take-off with the S-0102 re-file for a landable type). `tick_flight` calls it for a new
  destination as before.
- `air::move_` and `air::regive` (new): the first processing of a *move* with a destination runs
  claim 40's selection as before, then gives the selected destination's flight point (kept or
  replaced) and records the move's start tick in `AirState::regive` (new, hashed and saved). The
  move's next processing, when `regive` still names its start tick, clears it and gives the flight
  point once more. The take-off therefore happens at the processing, which precedes the movement
  update in the same tick. The re-file effect pushed at the processing is evaluated after the
  tick, after that update's climb, as at a movement update.
- `air::tick_flight`: claim 5's approach target replacement is gated on a positive height at the
  end of the update, for every aircraft (was: not the low release at height 0).
- `air::landing_trigger`: claim 6's close-approach trigger is gated the same way. The
  no-destination landing keeps `low_release_at_height_0`, now cited to Terms (on the ground by its
  height, not in flight) rather than an interim.
- `air::take_off`: claim 8's pad rule ends a building contact only when the cell of the aircraft's
  horizontal position belongs to that pad (`pad_cell`). A non-building contact ends as before.
- `air::tick_flight`, design for Open question 26 (below): a low release at height 0 with a
  destination, neither landing nor taking off, whose destination did not come with a move, is
  given its flight point at the start of the movement update, so it takes off there.
- `econ::release::aircraft`, docked row: with a rally point, a type that need not return to a dock
  now gets *move* queued and the destination assigned (claim 48 (d)), not started at once.
- Comments: the Q-1940 and Q-1941 interim comments in `destination_taken`, the replacement block
  and the close trigger are replaced by r62's citations; guidance's and (b)'s third case's comments
  lose r59's *Past the destination*.
- `quirks.rs` and the schema text: the switch's doc says the low release reaches height 0 and, on a
  move, takes off at the move's first processing; off, it takes off at the order.

**Design (Decisions of 2026-10-25), Open question 26, second part.** Whether a low release given
its destination by a behaviour other than *move*, such as the *attack* of claim 48 (d), takes off
as one on a move does is open. Our design: it takes off at the start of its next movement update
at height 0 with that destination, the same timing as the move's first processing, so no low
release makes a movement update at height 0 with a destination but without a take-off. No request:
the spec names the case open and the design follows claim 8's rule for an aircraft on the ground.
The first part (a pad-bound type that holds its contact in flight after a destination without a
behaviour order) keeps intake 28's design. No production path of ours reaches it: our AI gives
behaviour orders only, and the rally move is given only to a type that need not return to a dock.
A computer house's *area guard* attack in claim 48 (d) gives a destination without a behaviour
order, but only through the original computer player's behaviours.

**Claim 40, the next processing.** Read literally: the move's second processing gives the flight
point again whatever the aircraft is doing, so an aircraft that has started to land in its
destination cell before that processing, and has not yet settled (which clears the destination),
takes off again and lands once more. Built as stated.

| Claim or item | Status | Notes |
|---|---|---|
| Front matter, revision 62, Q-1940 to Q-1942 in `requests` | followed | |
| Overview, Q-1905 and Q-1940 to Q-1942 in full and their ratings | followed | Ratings; the behaviour is in the rows below. |
| Claim 40: the selected destination, kept or replaced, is given at the move's first processing; a destination in claim 8's sense | built, tested (new) | `move_` and `regive`. `ac30_k7_a` to `ac30_k7_c` (kept) and `ac30_k7_d` (replaced): take-off before the second update. |
| Claim 40: the move's next processing gives its flight point once more | built, tested (new) | `AirState::regive`. `claim_40_the_next_processing_gives_the_flight_point_once_more` (TENPAD, NEAR, a move order): `regive` holds the move's start tick from its first processing, is cleared at a later processing of the same move, and the flight then flies to the destination, unfinished. The landing case above is not tested. |
| Claim 5: the approach target replacement only for an aircraft above the terrain at the end of the update, whatever the aircraft | built, tested (new) | Height gate. `q1941_ordinary_flight_at_height_0_has_no_approach_replacement` (`claims_s0501.rs`, was `..._keeps_the_approach_replacement`): the attacker whose holding approach holds height 0 keeps claim 5's ordinary fraction, checked against `air::slowing`, with updates where the band would differ. `ac30_k7_b` checks the ordinary fraction at height 0 at the end of the first update. |
| Claim 6: the close-approach trigger only above the terrain; the exact-cell trigger at any height | built | Height gate; the exact-cell trigger unchanged. |
| Claim 8: claim 48 (c) for the factory release's contact | built, tested (new) | `take_off` reads `pad_cell`. `claim_48_c_tethered_type_keeps_its_contact_outside_the_foundation` (SCOUT at OUTPAD with a rally point): contact kept through the take-off, tether ended. |
| Claim 47 (a) to (c): the low release on a move tracked at its take-off | tested | `claim_48_low_release`: tracked at the end of the second update; `ac30_k7_a` checks tracking. |
| Claim 48 (b): a take-off, which this aircraft makes only from height 0 | built | `take_off` clears the released mark. |
| Claim 48 (c) *Low release on a move*: the move current in the first update's tick, first processed in the next, before the second update | tested | The order and the rally move both queue *move*; `claim_48_rally_point_at_the_release` checks *move* queued at the release and current after the first update. |
| *Low release on a move*: held height cruise and tracked at the take-off, still listed; body keeps the released facing; second update climbs under take-off steering, no new heading up to trunc(*H* / 2), then toward the destination with full speed; leaves the ground listing; a landable type re-filed into the top render layer | built, tested (new) | `k7_expect` (every k (7) run): height 0 and listed at the end of the first update; at the end of the second a positive height, held = cruise, taking off, unlisted in every recorded cell, not ground-filed; facing 16384 through the first update that ends above trunc(*H* / 2). `ac30_k7_c` and `ac30_k7_d`: then a turn toward the destination. `claim_48_low_release` (LOWPAD, JET) flipped: taking off and re-filed at the end of the second update. |
| *Low release on a move*: another behaviour's destination, open | design, tested | `open_question_26_destination_without_a_move_takes_off_too` (OUTPAD; destination by `assign`, no move): the same take-off, contact held throughout. |
| *At height 0* (rewritten) | built | As claims 5, 6 and 42. Intake 28's height-0 run tests are removed: `claim_48_e_low_release_at_height_0_carries_its_flag`, `..._off_its_heading_goes_straight_on`, `claim_48_c_low_release_straight_ahead_falls_back_in_the_halving_zone`, `..._at_a_right_angle_goes_past_then_climbs`, `claim_48_c_no_distance_bands_at_height_0`, `claim_48_c_positive_fraction_through_the_destination_cell_starts_no_landing`, `open_question_26_outside_the_foundation_falls_back_then_enters`; no low release reaches those cases now. |
| *At height 0*: the take-off needs no switch; off, it rests at the landing height and takes off at the order | built, tested | Doc comment and schema text. `claim_48_quirk_off_takes_off_toward_a_right_angle_destination` unchanged in its checks, comment updated. `every_spec_quirk_has_a_switch` green. |
| *Pad contact*: a behaviour order ends a pad-bound type's contact wherever it docks (T3 (b), S-0417 claim 11) | tested | `ac30_k7_e_person_order_ends_a_pad_bound_contact_outside_the_foundation` (OUTPAD, NEAR): contact and no tether at the release; no contact on either object from the first sample after the order. `destination_taken`'s interim comment retired; logic unchanged. |
| *Pad contact*: a destination without a behaviour order ends it only by claim 8's cell rule | built, tested | `destination_taken` (unchanged) and `take_off` (new cell test). |
| *Pad contact*: a tethered type keeps its contact outside the foundation with or without an order, unless the pad is of the unloading kind | tested | Order side: `orders.rs` T3 (b) (unchanged). Rally side: the tethered-type test above. |
| Claim 48 (d): the rally move is not a behaviour order; the release assigns the destination and queues *move* | built, tested | `release::aircraft`. `claim_48_rally_point_at_the_release` updated. |
| Claim 48 (e): low release on a move, no flag at the end of the second update | tested | `claim_48_e_low_release_climbs_and_clears`, `ac30_k7_a`. |
| AC30 part k (4): a target fraction of 0.75 at the end of the first update, at 80 leptons | tested | `claim_48_leaving_near` gains the check (`Sample::tgt`). |
| AC30 part k (7) (a) to (e) | tested on synthetic data | `ac30_k7_a_far_move_takes_off_at_the_first_processing` (TENPAD), `ac30_k7_b_straight_ahead_takes_off_with_the_ordinary_fraction` (MIDPAD, as intake 28), `ac30_k7_c_right_angle_takes_off_then_turns` (MIDPAD), `ac30_k7_d_replaced_destination_takes_off_too` (MIDPAD: from TENPAD's corner docking position claim 4's 5-lepton descent nudge carried the aircraft into the adjacent foundation cell, which F7 then accepts as the cell holding the lander itself, so claim 40 kept it and the run was void), `ac30_k7_e_...` (OUTPAD). The k (7) runs use NEAR. |
| Open question 26, first part (contact held in flight) | design | Intake 28's, rewritten for flight; `open_question_26_destination_on_its_pad_lands_and_docks` (WESTPAD, `assign`): it takes off, then lands and docks in the foundation cell with its contact kept. |
| Changes for other specs, S-0404 claim 13 | seam | S-0404 unchanged; the code follows S-0501 claim 48 (d). |
| Intake 28 ledger rows Q-1940, Q-1941, Q-1942 | retired | Updated in [intake28-S-0501.md](intake28-S-0501.md). |

## Goldens, arena and sweeps

Before is main 5fe9f1b, after is this branch. Outputs are in the local-only
`local-lane/m8/a3-31/before/` and `after/`.
- Goldens regenerated: duel, duel-diagonal, duel-naval, duel-m6c and skirmish. Causes, found by
  rerunning the gate with parts of the change undone:
  - duel, duel-diagonal, duel-naval and skirmish: the new hashed field `AirState::regive` only.
    With the field left out of the serialized state, they are byte-identical to main's.
  - duel-m6c: the field, and Q-1941. With the field left out it still moves, and with claim 5's
    replacement and claim 6's close trigger gated on the low release again (intake 28's interim)
    it is byte-identical to main's. As intake 28 saw, an attacker that claim 6's holding approach
    brings to height 0 loses its band value.
  - Neither claim 40's processing nor the rally queue moves any golden. No AI aircraft in the
    gate's games runs *move* with a destination: they are given attacks.
- The gate with its ignored tests passes (`--include-ignored`), including the M6c, M7a and M7b
  replays.
- Arena CSVs ours-M-0001, M-0002 and M-0003 are byte-identical before and after: M-0001 31 of 96
  failing and 151 of 172 reproduced, M-0002 13 of 24 and 43 of 47, M-0003 2 of 35 and 61 of 64.
- Sweeps over seeds 1 to 16, all exit 0. M7a and M7b, reference and `GLASNOST_PLAY_FIXES=1`:
  identical seed lines and counters. M6c moves with duel-m6c's cause:
  - Reference: smallest minimum 2 → 4 (seed 3's charger orders and seed 12's jumpjet orders, both 2,
    are now 4 or more). Medians move by a few (jumpjet orders 51 → 47, crate orders 124 → 141,
    garrison occupants 6 → 8). Minima of 4 after: buildings captured, infiltrations, tech building
    orders, changes of deployed form, disarm orders and crates picked up (crates picked up 11 → 4).
  - Play fixes: smallest minimum 4 → 4. Medians move by a few (garrison orders 21 → 26, charger
    orders 14 → 18, deploy orders 132 → 119).
- Save format 80 → 81, AI game format 42 → 43 (`AirState::regive`).

## Local lane

`low-release-ahead` and `low-release-abeam` (`crates/app/tests/m6c_lane.rs`, intake 28's scenes;
the summary line gains three counters). Outputs are in `local-lane/m8/a3-31/lane/` (`run.txt`,
per-update traces and screens).
- Both: the release at height 10, the order inside *S*. One update at height 0 with a destination
  (the first, in which *move* becomes current), then the take-off: at the end of the next update
  height 20, held height its cruise height, taking off, released mark gone. No turn at or below
  half the cruise height, a climb, and no rest at height 0.
- By eye, the end screen: the aircraft in flight above the pad's surroundings with its shadow, the
  pad empty.
- Intake 28's run along the released facing at height 0 (about 100 updates abeam) and its fall-back
  in the destination cell (ahead) are gone, as r62 states.

## Requests

None filed (block Q-1960 to Q-1964 unused). Q-1940, Q-1941 and Q-1942 were answered by r62, and
their interims are retired.
