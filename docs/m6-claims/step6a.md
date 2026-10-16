# M6b step 6a (warp whole-turn exclusion, deck-rest occupancy flag, behaviour rewrites): spec revisions since 090046f

Classification of every changed claim in `git diff 090046f HEAD` of the specs below, in the style of
[step5.md](step5.md). Buckets: **(a)** changes behaviour the engine implements (implemented, with a
test); **(a, already done)** the committed code already did what the new text says (a test was added
where none existed); **(b)** a seam or a pointer into a spec that is not built; **(c)** editorial;
**(e)** not built, with the reason. Front matter changes (revision, approval date, `requests`) are (c)
and not repeated.

| Spec | Revisions |
|---|---|
| S-0215 Behaviours | 47 to 48 |
| S-0301 Firing cycle | 17 to 18 |
| S-0203 Tick order | 31 to 32 |
| S-0209 Cell occupancy and crowding | 36 to 37 |

Requests. **Answered and retired here:** Q-0870 (the parts of the warp member's turn outside claim 51's
five columns; answered by S-0513 claim 51 and 52 with S-0203 claim 4, S-0301 claims 11 and 13 and S-0215
claims 2 and 11), Q-0872 (the cell layer of a deck-marked aircraft's flag; S-0209 claim 1 and S-0501
claim 44). **Left open for the S-0513 session:** Q-0871 (placement acceptance, S-0513 claim 50). **Filed:**
none; nothing in the new text was unclear or contradictory.

Changes of saved state or of the rules schema. **No change to the saved state layout** (the flag's
record is chosen at each event from the aircraft's position, nothing new is stored) **and none to the
rules schema.** Behaviour changes that move state hashes of a game that meets them: a warping vehicle's
expected-health bookkeeping and protection-envelope phases skip the excluded turns (the envelope's one
scenario-stream draw comes later, at the first hold turn, or never for an envelope that ends first); a
warping vehicle's reload countdown keeps elapsing; an aircraft resting on a high-bridge deck holds its
object-present flag in the deck record; a control-centre-dependent vehicle gets a guard reset when its
house loses or regains the centre. The sim-gate goldens of games with a chrono warp of a protected or
reloading vehicle, or with a control centre going on and off, may move; none was regenerated.

---

## S-0203 Tick order (rev 31 to 32)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 4 (the whole-turn list: self-healing, per-tick behaviour start and processing, movement, firing, turret turning) | (a, already done) | The warp update is the whole member turn from the first pending update through arrival effects. | `tick::object_turn` returns after `warp::turn` in every stage but the hold. Tests: `claim51_*` in `claims_s0513_contract.rs`; `a_queued_behaviour_waits_for_the_first_hold_turn`. |
| 4 (ordinary cloak update, attached parasite step, economy and power work, depot queued-entry retry) | (a, already done) | Part of the suppressed list. | Same early return, which skips `cloak::turn`, `parasite::host_turn`, `econ::events::turn` and `depot::vehicle_turn`. No isolated test of the four (their fixtures belong to S-0516, S-0614, S-0405 and S-0314); the structure is covered by the tests above. |
| 4 (ammunition reload update and firing-frame advancement; elapsed clocks keep running) | (a) for reload, (a, already done) for the frames | Suppressed reload work grants nothing and restarts nothing, but the countdown still elapses; the firing-frame counter is not advanced and the rearm deadline (an absolute tick) is not postponed. | `combat::fire::reload_elapsed`, called from `tick::object_turn` on every excluded warp turn. Tests below under S-0301. |
| 4 (expected-health turn bookkeeping, protection-brightness phase processing) | (a) | Suppressed through arrival effects; eligible in the hold after the hold-end test; no catch-up. Both used to run before `warp::turn`. | `tick::object_turn` (both moved after the warp early return; the dead-object paths keep them; `last_turn` stays before, since the exclusion is not a missed turn). Tests `expected_health_bookkeeping_is_skipped_and_resumes_in_the_hold`, `the_protection_envelope_waits_for_the_hold_but_its_duration_elapses` (`claims_s0203_warp_turn.rs`). |
| 4 (elapsed protection duration, intrinsic target search, sensor update, immediate damage, house updates not suppressed) | (a, already done) | Not part of the ordinary turn. | `protect::protected` is tick based; `warp::turn` runs `phasing_search`; the arrival sensor transfer is in `warp::arrive`. The protection test checks that the duration still runs. |
| 4 (launch/departure parasite releases still occur) | (a, already done) for the naval launch release; (b) for a drone parasite at departure | Owned by S-0513 claims 34 and 39 and S-0614; the departure release of a drone parasite is unresolved there and stays unbuilt. | `warp::depart` comment; not touched. |
| Scenario text SW23 part f | (c) | Scenario. | The tests above. |

## S-0301 Firing cycle (rev 17 to 18)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 11 (warp exclusion for reload) | (a) | From the first pending update through arrival effects: no round, no restart; the countdown elapses (not the moving freeze); in the hold one round on the first eligible turn after the hold-end test, then the ordinary restart, no catch-up; a never-started countdown first grants in an eligible hold turn. Before this the countdown froze in those turns. | `fire::reload_elapsed`; `tick::object_turn`. Tests `reload_countdown_keeps_its_clock_but_grants_nothing_until_the_hold`, `a_never_started_reload_countdown_first_grants_in_an_eligible_hold_turn` (`claims_s0203_warp_turn.rs`). |
| 11 (backing, F10 warp part) | (c) | Scenario. | The tests above. |
| 13 (warp exclusion for firing frames) | (a, already done) | The counter keeps its value through arrival effects, resumes one advancement per hold turn, and the rearm countdown is not extended. Materialising still refuses a shot (claim 9). | `tick::object_turn`; `check` (materialising). Test `the_firing_frame_counter_is_kept_and_the_rearm_deadline_is_not_postponed`. |
| 13 (backing, F11 warp part) | (c) | Scenario. | The test above. |

## S-0215 Behaviours (rev 47 to 48)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| Houses rule, Terms (campaign house levels), claims 15, 27, 20; Terms *Campaign game*; the Constants and Import rows for campaign skill and `IQ` of a map's house section | (c) | The campaign rules are no longer specified; the skirmish rules and the engine's rules are unchanged. The code never built the campaign rules (no campaign hunt, no per-house `IQ`). | none |
| 2 and 11 (per-tick start suppressed on the warp turns; resumes in the hold) | (a, already done) | The literal text now excludes the per-tick start from the first pending update through arrival effects, where revision 47 left it open (Q-0870). | `tick::object_turn` (the early return skips `process::turn`). Test `a_queued_behaviour_waits_for_the_first_hold_turn`. |
| 2 (the chrono arrival while being erased edge) | (c) | Removed; no rule asserted. The code never built it. | none |
| 3 (draw assignment after a targetless hunt search: B45 a) | (c) | Backing only. | none |
| 9 (force sites: B45 f for disabling and enabling a control-centre-dependent vehicle; S-0607 L2) | (a) for the control-centre sites; (b) for the rest | The guard reset applies to a deactivated or reactivated actor. The code had no guard reset at the control-centre switch; it now forces *guard* (the suspended slot kept) and clears the target, the destination and the guard spot when the count of active control centres goes to 0 or back from 0. A pulse disable (S-0610), mind control (S-0601) and the magnetic seizure (S-0607) are M7 and stay seams; entering an open-topped transport and the slave sites are unchanged. | `econ::power::set_disabled`, `guard_reset`. Test `claim_13_control_centre_switch_gives_the_dependent_vehicle_a_guard_reset` (`claims_s0405.rs`). A placed vehicle that is born disabled gets no reset (it is not a switch). |
| 10 (cloak start revert with and without sensor coverage: B45 b; the suspended slot kept across a force: B45 f) | (c) | Backing only. | none |
| 11 (readiness unchanged; B45 c backs the firing and noninterruptible sequence) | (c) | Backing only. | none |
| 27 (hunt without a target: the campaign move removed; the delay uses the rate of capture or sabotage only) | (c) | Editorial: the campaign rule is out of scope; the skirmish outcome is unchanged. | none |
| 32 (secondary refusals, out of ammunition: B45 d, e) | (c) | Backing only. | none |
| 37 a2 (the resulting state, not a count of internal evaluations) | (a, already done) | The one-approach-per-tick sentence became a statement of the resulting state: a processing's destination or a target-loss outcome must still hold at the end of the tick. The code takes the per-tick step after a processing's step, which satisfies it; the comment in `process::turn` no longer claims a "no one-step limit". | `process::turn`, `approach::per_tick`. Tests `a_destination_left_by_the_processing_survives_the_later_rule`, `an_unchanged_in_range_state_keeps_the_target_and_takes_no_destination` (`claims_s0215_approach.rs`). |
| Open questions, B41 wording, B45 | (c) | Editorial and proposed measurements. | none |

## S-0209 Cell occupancy and crowding (rev 36 to 37)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 1 (other objects: the deck-rest flag) | (a) | An ordinary landed aircraft marks the deck record only when a high bridge is present and its world height is at least the terrain surface height plus 416 leptons (signed integer leptons), the ground record otherwise; neither the listing nor the deck mark substitutes for the test. Take-off clears the deck flag in the first climbing movement tick, without a transfer to ground; accepting the destination alone releases nothing. | `air::mark_layer`, `set_mark`, `clear_mark`, `removal_marks`. Tests `ac27_rest_on_terrain_marks_the_ground_record_only`, `ac27_deck_rest_marks_deck_only_and_take_off_clears_it_in_the_first_climbing_tick` (`claims_s0501.rs`). |
| 1 ("the flags are not counts") | (b) | The code keeps `others` as a counter, as for the vehicle and building flags and the teleport reservations (an earlier revision already says "flags, not per-vehicle counts"); the difference shows only with two writers on one flag. Not changed here. | `cells::Occupancy::others` |
| 5 (bridge deck paragraph: the flag alone refuses nothing; below clear, on deck stationary friendly from the listing; clear after take-off) | (a, already done) | The vehicle tables never read the object-present flag; the refusal is the allied listed occupant (row VP20). | `occupancy::vehicle_outcome`. Test `ac27_a_resting_deck_aircraft_refuses_only_the_deck_vehicle_through_its_listing`. |
| Backing and confidence lines | (c) | AC27 occupancy and vehicle variants. | none |

The clearing event computes the record from the aircraft's actual position, as the text says. A
bridge collapse under a marked aircraft (S-0511) is still a seam: after a collapse the deck test
fails, so a clear would hit the ground record while the deck flag stays; no case reaches it yet.

## Not touched

No M6c spec (S-0508, S-0509, S-0510, S-0515, S-0502, S-0517, S-0525, S-0514, S-0518, S-0519, S-0520)
and no M7 spec was started. S-0513 revision 14 (claims 50 to 52, Q-0871), S-0501 revision 16
(claim 44 beyond the occupancy flag), S-0516 revision 9 and S-0614 revision 7 belong to another
session.
