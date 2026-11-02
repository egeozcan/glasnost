# S-0501 r93 and S-0502 r21: claim ledger, M8 step A3 intake 76

- **S-0501** r65 to r93 (commit 87ef8af, from be0579e). It answers Q-2251 (how an aircraft
  answers the scatter off a building), Q-2252 (an unarmed pad-bound aircraft away from its pad)
  and Q-2291 (a spy plane in overfly at the map's edge, filed by seam trace 2). It adds two
  quirks, `quirks.aircraft_edge_hold` and `quirks.unarmed_pad_bound_hover`.
- **S-0502** r20 to r21 (15f729a). It restates S-0607 r14's retry cadence and S-0215 claim
  37 part e's direct removal.

Requests filed: Q-2326 (what a processing of *enter* does for an aircraft with a pad destination
and no contact). Q-2327 to Q-2335 unused.

Status words: **tested** (built and checked by the named test), **partly** (built, with the
part named left untested or not built), **seam** (not built; the reason is given), **no change**
(the revision restates what is built).

## Built

- **Claim 4's side rule** (`air::edge_column_low`): the push-back's sign comes from the new
  position's local column *a* = ⌊(*d* + *W*) / 2⌋ − *L*x against *T* = ⌊(*W* + *H* − 1) / 2⌋,
  in place of our earlier map-middle rule.
- **`quirks.aircraft_edge_hold`** (`air::tick_flight`): off, an aircraft that may leave the map
  now is not pushed back, takes no candidate and no draw, and claim 10's legal-map removal
  (`leave_map_test`) then takes it out.
- **Claims 5 and 6** (`air::landing_trigger`): the exact-cell trigger reads the target of the
  same tick's ordinary calculation, passed in from the speed-fraction block, so claim 5's
  approach replacement (0 below 128 leptons) never starts a landing. The cell tested is that of
  the flight point the flight holds (`flight_target`), not that of the destination object.
- **Claims 18 and 24** (`air::pad_accepted`, called from `destination_taken`): the contact is
  made when a pad of the aircraft's dock types is accepted as destination while *enter* is
  current or queued and no contact is held. That is in the order's application (the behaviour is
  queued before the destination is set, `orders.rs`) and in every economy assignment
  (`econ::assign_destination`). The search after T2's row now uses `find_pad`, which asks for
  no contact, in place of `seek_pad`.
- **Claim 49** (`scatter::answer`): an aircraft answers every request alike. It falls back to
  rest when its current behaviour has the scatter property, or when it has no current behaviour,
  and otherwise does nothing; it never accepts (no nearby cell, no take-off of its own). Before
  this, a request to an aircraft did nothing.
- **Claim 50 and `quirks.unarmed_pad_bound_hover`** (`air::fall_back_rows`): on (default), an
  unarmed aircraft makes no pad search, as before. Off, for an unarmed aircraft of a landable
  pad-bound type with dock types falling back in flight with no contact, T2's last in-flight row
  first calls `seek_pad` (claim 21, with its contact request): granted, the pad and *enter*;
  otherwise the row's F8 spot and *move*.

No state is added, so the save format (92) and the AI game format (55) are unchanged. The
changed aircraft play moves five sim-gate goldens, regenerated: `skirmish`, `duel`,
`duel-diagonal`, `duel-naval` and `duel-m6c` (each first differs mid-game).

## Claims

### S-0501 r93

| claim | status |
|---|---|
| 4, the side rule of the edge push-back (+128 when *a* < *T*, else −128) | Tested: `claims_s0501_r93_edge::claim_4_the_side_rule_of_the_push_back` (every shift checked against the formula and the step of F3, on a banded map where every shift is +128, and on a wholly usable map 20 wide and 8 high, where *T* = 13 and both signs occur) |
| 4, the shifted point when legal, with no draw; else one raw draw and the candidate 64 leptons away, taken only when legal | Tested in the same test (the rng is compared before and after each push-back tick) |
| 4, the fly-by flag: no push-back and no draw | Tested: `claim_51_the_edge_outcome_by_flight_flags` |
| 5, the speed fraction for a flight point with or without a destination object | Partly: built through `flight_target`, which keeps the flight point without a destination object only for claim 42's *attack* case. Claim 49's cleared destination (the hunt rule) never applies in our engine (S-0215 claim 28), so no other route reaches it |
| 5, Backing (AC31 part d: at height 0 the logged target is the ordinary one) | Tested: `claims_s0501_r93::claim_50_a_type_that_is_not_landable_stops_short_at_height_0` checks the stop; the replacement's height condition was built at r65 (`height(w, id) > 0`) |
| 6, the exact-cell trigger reads the same tick's ordinary target, so a stall within 128 leptons in the flight point's cell starts no landing | Tested: `claims_s0501_r93::claim_6_a_stall_in_the_flight_points_cell_starts_no_landing` (AC31 part c: converted speed 5, slowdown 600, westward on the row) |
| 6, the cell tested is the flight point's | Partly: built (`flight_target`); the case where it differs from the destination object's cell is reached in our engine only by claim 42's *attack* case, untested here (the spec rates it `unmeasured`) |
| 6, an unarmed pad-bound aircraft's refused landing gives an F8 spot (claim 50) | Tested with claim 50 below |
| 10, with the fly-back flag alone, or at an edge where the bounds coincide, never removed there | Tested: `claim_51_the_edge_outcome_by_flight_flags`, `claim_10_neither_flag_is_held_where_the_bounds_coincide` |
| 18, the search after the row asks the pad for no contact; the acceptance makes one only with *enter* current or queued | Tested: `claims_s0501_r93::claims_18_and_24_the_second_fall_back_makes_the_contact` |
| 18, an armed pad-bound aircraft crashes; any other gets F8 | No change (built); the unarmed pad-bound case is claim 50's |
| 24, the contact at the acceptance with *enter* current or queued | Tested: `claim_24_an_ordered_enter_makes_the_contact_at_the_acceptance` (in the tick the order applies) and `claim_24_a_move_onto_the_pad_makes_no_contact` |
| 24, *enter*'s processing asks only a pad in contact and makes no contact itself | Partly: `air::enter` asks only the contact pad. With no contact it clears the destination and falls back, as for a lost reservation; the spec names an "unlinked retry" it does not describe. Seam, Q-2326 |
| 37, 40, 42 and the run of claim 34 (the new cross-references) | No change |
| 49, every request alike; the gate is the current behaviour's scatter property, or no current behaviour | Tested: `claim_49_a_cells_request_is_gated_by_the_current_behaviours_scatter_property` (AC31 part e runs 2S and 2C, with and without a threat point and the forced option), `claim_49_no_current_behaviour_always_falls_back` |
| 49, capture, sabotage and eaten on a building's cell end as on open ground (unarmed, landed) | Tested: `claim_49_capture_sabotage_and_eaten_on_a_building_end_as_on_open_ground` (AC31 part a runs 1, 3 and 4, each with the scatter property set and clear) |
| 49, landed, armed, ammunition spent, on a building: the second fall-back makes the contact only through the scatter property | Tested: `claim_49_the_request_on_a_building_makes_the_contact_only_through_the_scatter_property` |
| 49, the take-off "at once, in that processing" when a pad is accepted on the ground | Partly: our take-off starts at the movement update that takes the destination (claim 8, `give_point`), later in the same tick, so the end-of-tick state is the same; the moment within the tick is not built as stated |
| 49, a computer-controlled owner and the hunt rule | No change: the hunt rule never applies in our engine (S-0215 claim 28), so it ends as a human-owned one, as the claim says |
| 49, the scatter from damage (S-0215 claim 19 cases 1 and 2) | Seam: the sender is S-0215's (intake 74). Whatever request reaches an aircraft is answered by claim 49's gate here |
| 49, a cleared destination keeps the flight's point (claim 40's new flight point) | Seam: arises only through the hunt rule, which our engine does not apply |
| 50, no pad search, no crash; F8 and *move*; the hover near the pad, free or taken | Tested: `claim_50_an_unarmed_pad_bound_aircraft_hovers_near_its_pad` (AC31 part b runs 1 and 3) |
| 50, the sold pad (AC31 part b run 4) and the route's last point (run 2) | Partly: not tested here; F8 with no building of the dock types is claim 41's, tested for other aircraft |
| 50, the slow-speed stalls | Tested with claim 6 above (V < 10) |
| 50, a type that is not landable: no trigger, the descent to 0 within 768 leptons, the stop below 86 leptons in the point's cell along the row | Tested: `claim_50_a_type_that_is_not_landable_stops_short_at_height_0` (AC31 part d) |
| 50, the own-cell no-result case, the slowdown distance of 0, no dock types, no building | Partly: built by the rules they name (claims 5, 41); not tested here (the spec rates them `unmeasured`) |
| 51, the held heading, the run lock, nothing ends overfly | Tested: `claim_51_the_heading_is_held_and_nothing_ends_overfly` |
| 51, the outcome at the map's edge by the flight flags | Tested: `claim_51_the_edge_outcome_by_flight_flags` |
| Edge cases, `quirks.aircraft_edge_hold` (both positions) | Tested: `quirk_aircraft_edge_hold_both_positions`. Admitted to the play-fixes layer (c7-play-fixes.md) |
| Edge cases, `quirks.unarmed_pad_bound_hover` (both positions; a type that is not landable unchanged) | Tested: `quirk_unarmed_pad_bound_hover_off_docks_at_a_free_pad_and_hovers_at_a_taken_one`, `quirk_unarmed_pad_bound_hover_changes_nothing_for_a_type_that_is_not_landable`. Rejected for the layer |
| Import mapping, `FlyBy` and `FlyBack` (wording) | No change |

### S-0502 r21

| claim | status |
|---|---|
| 24, the approach step's direct removal of a gripped unit's destination is not an effective clear | No change: `approach.rs` already says so (S-0215 claim 37 part e), and S-0607 claim 9 is built |
| 28, the balloon-hover rule applies to a seized vehicle too, before its refusals | No change: `jump::clear_rejected` is tested before `lift::refuses_destination` (S-0607 r33); comment in `lift.rs` updated |
| Edge cases and J27, the retry at the end of each of the set-down unit's turns, not at a fall-back to rest | No change (S-0607 r14, `lift::end_of_turn`); tested by `claims_s0607_r14::claim_20_hovering_set_down_retries_at_the_end_of_each_turn`, now cited to S-0502 r21 too |
