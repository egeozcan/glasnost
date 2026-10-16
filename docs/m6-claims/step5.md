# M6b step 5 (warp contracts, aircraft deck mark, restoration readiness): spec revisions since 525ea4f

Classification of every changed claim in `git diff 525ea4f HEAD` of the specs below, in the style of
[step4.md](step4.md). Buckets: **(a)** changes behaviour the engine implements (implemented, with a
test); **(a, already done)** the committed code already did what the new text says (a test was added
where none existed); **(b)** a seam or a pointer into a spec that is not built; **(c)** editorial;
**(e)** not built, with the reason. Front matter changes (revision, approval date, `requests`) are (c)
and not repeated.

| Spec | Revisions |
|---|---|
| S-0513 Superweapon effects | 12 to 13 |
| S-0524 Multiplayer trigger completion | 7 to 8 |
| S-0501 Aircraft flight, attack runs and pads | 14 to 15 |
| S-0105 Rendering of voxel models | 18 to 19 |
| S-0203 Tick order | 30 to 31 |
| S-0215 Behaviours | 46 to 47 |
| S-0406 Building placement | 9 to 10 |
| S-0314 Healing and repair in combat | 21 to 22 |
| S-0209 Cell occupancy and crowding | 35 to 36 |

Requests. **Answered:** Q-0840 (S-0524 claim 15), Q-0841 (S-0501 claim 43), Q-0850 (S-0513 claim 50),
Q-0851 (S-0513 claim 51); their interim rows and `Q-08xx` markers are retired. **Filed:** Q-0870 (what
a warp member does in the parts of the turn outside claim 51's five columns, and the per-tick
behaviour start), Q-0871 (which placements of a warp member fail in claim 50), Q-0872 (which
occupancy record holds the object-present flag of a deck-marked aircraft at rest).

Changes of saved state or of the rules schema. **Saved state changes**, all with `serde(default)` so
format 29 saves still load: `AirState::deck` (the aircraft deck mark) and `warp::Member::fell_back`
and `refused` (the claim 50 report flags). The rules schema is **unchanged**. Behaviour changes that
move state hashes of a game that meets them: an aircraft in flight is now listed in no cell layer
(it was listed in the ground layer throughout), an aircraft over a high-bridge cell at least 416
leptons above the terrain measures its height from the deck, the warp clears target and destination in
the member's first update instead of at launch, and a warp whose landing cell is outside the map no
longer places the member there. Goldens of games with aircraft in flight or with warps to the map's
edge may move; none was regenerated.

---

## S-0513 Superweapon effects (rev 12 to 13)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 48 (original-side evidence, Q-0851) | (a, already done) for the rows; (a) for the clearing | The claim is now the original's evidence, not our rule; its table marks the unmeasured rows. Claim 51 supplies ours. Target and destination are dropped by the leaving-stage rule, with no earlier clearing. | `warp::turn` (the `Pending` arm clears target, burst and destination when the leaving stage starts; `warp::join` no longer does). Test `claim48_a_warp_member_drops_its_target_and_destination` now checks both sides. |
| 49 (original-side exceptional cells, Q-0850) | (a, already done) | The rows are the original's; claim 50 supplies ours. | `claim49_*` in `claims_s0513_warp.rs` (notes now `warp: object ...`). |
| 50 (exceptional landing contract) | (a) | Fallback cell (0, 0) for an off-map destination and a failed search; the first point is the centre (vehicle) or the claim 36 point (infantry, aircraft); after a failed search the position within the previous landing cell is kept; height 0 and no lift while (0, 0) is not a legal cell (S-0009 claim 1, never, since s = 0). | `warp::examine`, `warp::join`, `warp::redirect`, `landing_in`. Tests `claim50_an_off_map_destination_falls_back_to_cell_zero_with_the_claim_36_point`, `claim50_a_failed_search_falls_back_keeping_the_position_within_the_previous_landing_cell`. |
| 50 (blocked, never placed outside the map; retry; no cap) | (a) | A point outside the map is blocked and uses the redirect search; a failed search replaces the point with the fallback again; retries set the held delay; the member stays alive with no cap and no timeout death. | `warp::check` (an illegal cell is `Blocked`; before this the check passed on an empty listing and the member was placed off the map). Tests `claim50_a_member_with_no_legal_landing_stays_alive_in_the_retry_without_a_cap`, `claim50_a_legal_redirect_after_the_fallback_places_and_arrives_on_separate_updates`. |
| 50 (placement failure, report) | (a), with Q-0871 | A refused placement returns the member to the landing retry with point and owner kept; the first fallback and first refusal per member and warp are reported, with no draw and no game-rule effect. | `warp::place` (a landing point outside the map fails; other causes are Q-0871), `Member::fell_back` and `refused`, world notes `warp: object ...`. Test `claim50_a_refused_placement_keeps_the_member_in_the_retry_and_tries_again`. |
| 50 (passed check: placement and arrival on separate updates) | (a, already done) | A passed check schedules placement on the next update, and placement schedules the effects on the next. | `Stage::Away`, `Stage::Placed`; `claim39_40_clean_timeline` and the test above. |
| 50 (a transport's passengers, tracking by the house) | (a, already done) | The member stays an object with its owner and passengers; nothing deletes it. | `claim50_a_member_with_no_legal_landing_stays_alive_in_the_retry_without_a_cap` (owner, health). The on-map counts of S-0414 are not changed (a seam); no loaded transport fixture exists in the synthetic rules. |
| 51 (the five columns) | (a, already done) | No ordinary self-healing, behaviour processing, movement, firing or turret turning from the first pending update through arrival effects; permitted in the hold after the hold-end test. | `tick::object_turn` and `warp::turn` (the warp update is the whole turn before the hold). Tests `claim51_no_self_healing_before_the_hold_and_healing_in_it`, `claim51_no_movement_before_the_hold`, `claim51_no_behaviour_firing_or_turret_before_the_hold`. Parts of the turn outside the five columns: Q-0870 (interim: skipped as before). |
| 51 (launch clears the route) | (a) for the walking and hover steps; (a, already done) for the rest | Launch clears the path, the queued destinations and the driving step at once. | `warp::join` (vehicle drive state, as before; now also a walker's fraction and reserved standing position, `release_walk`, and a hover's heading point). Test `claim51_launch_clears_the_path_the_queue_and_the_driving_step_at_once`. |
| 51 (one warp update per tick, off the map included) | (a, already done) | Pending, leaving, awaiting landing, placed and holding each get one update per tick. | Members stay in the update order. Test `claim51_a_member_receives_one_warp_update_per_tick_even_off_the_map`. |
| 51 (protection duration, searches, sensor update not suppressed) | (a, already done) | Elapsed protection, the intrinsic target search and the sensor update are not suppressed. | `protect::tint_turn` runs before `warp::turn`; `phasing_search`; `cell_movement_completed` at the arrival effects (`claim_22_warp_member_disc_transfers_at_the_arrival_effects`). |
| Import mapping rows (warp turn, exceptional landing) | (c) | Engine design, no data key. | none |
| SW23 (f), SW24 (c) | (a) | Engine-only contract tests. | The tests above. |

## S-0524 Multiplayer trigger completion (rev 7 to 8)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Terms (ready mark) | (c) | Definition. | none |
| 13 (the placed case, missing art) | (a, already done) | A restored replacement is the placed-building case, even without a build-up sprite. | `mp_triggers::replace` calling `placement::begin_construction(w, id, false)`; the Q-0840 interim marker is retired. |
| 15 (clear mark at entry; first mark; the table) | (a, already done) | Mark clear at entry; at least 2 stages and a positive delay: the mark at the turn that lands on the last stage, two build-up runs; zero delay: the first eligible building turn sets it, C = U + 1, H = U + 2; one stage or fewer with a positive delay: never completes. | `placement::begin_construction`, `construction_turn`. Tests `claim15_restored_multi_stage_range_runs_twice_and_hands_over_by_f6`, `claim15_restored_zero_delay_range_sets_the_mark_at_its_first_eligible_turn`, `claim15_restored_single_stage_range_never_completes`. |
| 15 (the shared `placed_buildup_twice` switch) | (a, already done) | Off sets the mark at entry and uses the deployed-style start. | `claim15_restored_replacement_follows_the_placed_buildup_twice_switch`. |
| 15 (healing an existing building) | (a, already done) | Healing starts no construction and changes no mark. | `claim15_healing_an_existing_building_does_not_restart_construction`. |
| Import mapping row, C17 | (c) | Range inputs are S-0108 claim 5's; scenario. | none |

## S-0406 Building placement (rev 9 to 10)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 17 (restored replacement joins the construction-state entries; place order and restoration leave the mark clear) | (a, already done) | The restored case follows the placed timing, with U the actual first eligible turn. | `placement::begin_construction`; the S-0524 claim 15 tests above, which measure U instead of assuming T + 1. |
| F7, switch text, transitions table | (c) | Restoration added to the wording. | none |

## S-0501 Aircraft (rev 14 to 15)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Terms (aircraft deck mark, height) | (a) | The mark is its own state; *h* is the signed world height minus the terrain height, minus 416 when the deck is the reference (marked, or unmarked at least 416 above a high-bridge cell). | `air::deck_marked`, `deck_reference`, `base`, `height`, `AirState::deck`. |
| 43 (set, clear and keep rules) | (a) | Descent reaching height 0 above the deck sets it; landing finishing at deck height sets it; a climbing adjustment and a refused touchdown clear it; horizontal flight leaves it. | `air::tick_flight` (altitude block), `touchdown`. Tests `ac27_flying_across_a_bridge_never_marks`, `ac27_landing_on_a_bridge_sets_the_mark_and_lists_on_deck`, `ac27_taking_off_from_the_deck_clears_the_mark_in_the_first_climbing_tick`, `ac27_a_refused_touchdown_clears_the_mark`, `ac27_a_marked_aircraft_at_the_held_height_keeps_its_mark_across_cells`. |
| 43 (render layer follows claim 7 with the mark in the height) | (a) | A marked aircraft at deck height has height 0 and the ground render layer; no bridge render layer. | `air::on_ground` (`listing_height`), `World::surface_for` (adds the deck for a marked aircraft, so S-0307's airborne test agrees). Asserted in the landing test. |
| 43 (claim 1: fresh placement) | (a) | A fresh ground placement clears the mark. | `air::placed`, `released_docked` (a pad grants none). |
| Import mapping row | (c) | No data key. | none |
| AC27 | (a) | The scenario as an engine test. | The tests above. |
| Seam | (b) | A bridge collapse under a marked aircraft (S-0511) and the occupancy flag's layer for a deck-marked aircraft at rest. | Q-0872 interim (`air::set_mark`). |

## S-0209 Cell occupancy and crowding (rev 35 to 36)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 13 (listing height for aircraft) | (a) | An aircraft in ordinary flight is listed by its listing height (world height minus terrain height minus 416 when marked): positive, in neither layer; 0 or below, on the deck when marked and on the ground otherwise. Before this the code listed an aircraft in the ground layer throughout its flight. | `air::listing_height`, `listing_layer`, `sync_listing` (called after every move and at the end of `air::tick`, in `placed` and `released_docked`), `cells::leave_both` (removal, going inside, warp lift-off). Tests `s0209_claim_13_ordinary_flight_is_listed_only_at_height_zero` and the AC27 tests. |
| 13 (backing, AC27 contrast) | (c) | Scenario. | none |

## S-0105 Voxel rendering (rev 18 to 19)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Terms (aircraft deck mark, shadow height *a*) | (a) | *b* is 416 only while the aircraft deck mark is set. | `mapview::live::shadow_anchor` reads `AirState::deck`; the Q-0841 interim (layer == deck) is retired. |
| 40 (the mark is not a layer or an occupancy inference) | (a) | A deck layer without the mark does not count. | `aircraft_shadow_anchor_bridge_qualification` (`live_tests.rs`, extended). |
| MS-30 additions | (c) | Scenario. | none |

## S-0203 Tick order (rev 30 to 31)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 4 (within-turn superweapon warp: original evidence, then engine contract) | (a, already done) | The five suppressions through arrival effects, ordinary eligibility after the hold-end test, one warp update per tick, immediate route clearing; these are not missed turns (claim 10). | `tick::object_turn`, `warp::turn`; the S-0513 claim 51 tests. |
| Scenario text (SW23 f) | (c) | Scenario. | none |

## S-0215 Behaviours (rev 46 to 47)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 2 (warp design exclusion; locomotor phase-in does not stop processing) | (a, already done) | No ordinary behaviour processing from the first pending update through arrival effects; allowed in the hold. A teleporting locomotor's own phase-in still allows processing. | `warp::turn`; `claim51_no_behaviour_firing_or_turret_before_the_hold`; existing teleport tests in `claims_s0207_teleport.rs`. The literal text keeps the per-tick start of claim 11 on the warp turns (it lists only the tunnel, erasing and leaving-stage exclusions); the code skips it: Q-0870. |
| Scenario text B-series | (c) | Scenario. | none |

## S-0314 Healing and repair (rev 21 to 22)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 9 and 10 (engine-design suppression of individual and house-wide healing) | (a, already done) | No healing from the first pending update through arrival effects; ordinary eligibility in the hold, expiry turn included. | `warp::turn` returns before `heal::turn`, which holds both kinds. Test `claim51_no_self_healing_before_the_hold_and_healing_in_it`; `claims_9_and_10_leaving_stage_skips_healing` (house-wide, leaving stage). |
| H8 f | (a) | Engine-only contract test. | The test above. |

## Not touched

No M6c spec (S-0508, S-0509, S-0510, S-0515, S-0502, S-0517, S-0525, S-0514, S-0518, S-0519, S-0520)
and no M7 spec was started.
