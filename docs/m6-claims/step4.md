# M6b step 4 (storm witness, unit goals, occupancy listing): spec revisions since e2d1bc0

Classification of every changed claim in `git diff e2d1bc0 HEAD` of the specs below, in the style of
[step3-sw.md](step3-sw.md). Buckets: **(a)** changes behaviour the engine implements (implemented,
with a test); **(a, already done)** the committed code already did what the new text says (a test
was added where none existed); **(b)** a seam or a pointer into a spec that is not built; **(c)**
editorial; **(e)** not built, with the reason. Front matter changes (revision, approval date,
`requests`) are (c) and not repeated.

| Spec | Revisions |
|---|---|
| S-0513 Superweapon effects | 11 to 12 |
| S-0507 Transports | 7 to 8 |
| S-0504 Naval units | 17 to 18 |
| S-0215 Behaviours | 45 to 46 |
| S-0209 Cell occupancy and crowding | 34 to 35 |

Requests. **Answered:** Q-0800 (the position of a building in the storm strike witness test; its
file already reads `answered`). **Still open, unchanged:** Q-0850 and Q-0851 (revision 12 did not
touch S-0513 claims 33 to 49, so their interim rows and `Q-0850`/`Q-0851` markers stay); Q-0840 and
Q-0841 (not touched by these specs). **Filed:** none.

Changes of saved state or of the rules schema: **none to the saved state's layout and none to the
rules schema.** One behaviour change moves the state hash of a game that meets it: a storm strike
on a cell holding a building now measures that building at its foundation centre instead of its
top-left cell centre (`storm::cell_state`). A 1 by 1 building is unchanged (its position and
reference point coincide); larger buildings change the witness, hence the debris decision and the
following draws. The goldens of games with a lightning storm over multi-cell buildings may move.

---

## S-0513 Superweapon effects (rev 11 to 12)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 18 (witness reference point) | (a) | The strike witness is measured from each candidate's reference point (S-0204 claim 4) modulo 256; a non-building is its own position, a building its foundation centre without the bib row. Remainders are compared with (0, 0). Height and the building-size distance reduction do not enter. | `storm::cell_state` (was the stored `Object::pos`, the top-left cell centre), through `World::reference_point`. Tests `claim18_a_building_witness_is_measured_at_its_foundation_centre` (1 by 1 at 181, 2 by 2 at 0, 1 by 2 and 2 by 1 at 128), `claim18_debris_follows_the_top_corner_witness_not_the_centre_nearest_unit` exists. The test margins avoid the table root's coarse steps (a candidate one lepton past 128 still ties with it). |
| 18 (Q-0800, building witnesses listed per foundation cell) | (a, already done) for the listing; (a) for the threshold test | An on-map ground-layer building is listed in every actual foundation cell (S-0406 claim 2), a missing cell of an irregular foundation excluded; the same threshold applies in every cell. | `objects::place_with` (listing, `econ::placement::foundation_cells`). Tests `claim18_the_building_threshold_is_the_same_in_every_foundation_cell` (2 by 2 and 3 by 3, struck on non-anchor cells), `claim18_a_building_is_listed_in_every_actual_foundation_cell` (the 8-cell word lacks (2, 1)). |
| 18 (tie against a listed non-building at 181) | (a, already done) | A non-building at exactly 181 beats a 1 by 1 building because it precedes buildings in the listing. | The `d < best` rule keeps the first listed (`storm::cell_state`); the listing order is S-0209 claim 13. Covered by the infantry-below and above cases; the exact-181 tie is not isolated (the table root makes it a range, not one position). |
| Import mapping row `Foundation` | (a, already done) | The foundation vocabulary and default `1x1` are S-0108 claim 8's and S-0406 claim 2's. | `glasnost_rules` importer, unchanged. |
| SW7 part f | (c) | Scenario. | none |
| Claims 48, 49 and the other Q-0850 / Q-0851 rows | unchanged | No revision-12 text. | Interim rows stay in `docs/development.md` ("Simulation (M6b: superweapons)"). |

## S-0209 Cell occupancy and crowding (rev 34 to 35)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 13 (building in every foundation cell) | (a, already done) | A building is listed in every actual foundation cell, not only its anchor; a missing cell does not list it; the listing order is unchanged. | `objects::place_with`, `objects::remove` (touched cells from `econ::placement::foundation_cells`). New test `claim18_a_building_is_listed_in_every_actual_foundation_cell` (in `claims_s0513_storm.rs`, where the consequence is observed). |
| 13 (backing and confidence) | (c) | Scenario SW7(f) added to the backing. | none |

## S-0507 Transports (rev 7 to 8)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 5 (transport object goal) | (a, already done) | For a driving or ship passenger, a destination naming the transport is an object goal: with no active track, occupying the cell of the supplied point alone clears neither the goal nor the locomotor point, whether the transport moves or not; the resting guard exception remains; a later cell assignment uses the cell-goal cleanup. | `drive::goal_cleanup` (`named`; Q-0830, step 3). New tests `claim_5_a_transport_goal_is_kept_by_cell_equality`, `claim_5_the_guard_exception_compares_the_retained_point`. |
| 6 (arrival-time boarding separate from cleanup) | (a, already done) | Retention on cell equality suppresses neither boarding nor refusal and does not guarantee the passenger stays in *enter*. | `transport::arrival` (called at a track's end by `drive`, `hover`, `walk`), `process::turn`. New test `claim_6_boarding_happens_at_arrival_not_at_cleanup`; `moving_transport_turns_the_passenger_away`, `claim_6_refused_boarding_has_already_notified` exist. |
| T-scenario "Vehicle-goal distinction" | (c) | Scenario. | none |

## S-0504 Naval units (rev 17 to 18)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 1 (unit goals) | (a, already done) | A destination naming a building, vehicle, infantry unit or aircraft is not a cell destination; the named unit's motion neither clears nor replaces the point; the resting guard exception compares the retained point for all these goals. | The ship runs the shared `drive::tick` (`naval::tick`), so `drive::goal_cleanup` serves it. New tests `claim_1_ship_keeps_a_destination_naming_a_unit_under_cell_equality`, `claim_1_ship_guard_row_compares_the_retained_point_for_a_unit_goal` (a vehicle, an infantry unit and an aircraft as the named unit). Building goals: `claims_s0314_depot.rs`. |
| 1 (backing, confidence, scenario N2 text) | (c) | D31 added. | none |

## S-0215 Behaviours (rev 45 to 46)

| Claim | Bucket | Change | Code location, tests |
|---|---|---|---|
| 22 (unit goals in the move behaviour's cleanup note) | (a, already done) | A destination naming a vehicle, infantry unit or aircraft is kept under cell equality; the exact-point guard exception removes it or invokes the approach-destination fallback and compares the retained point, not the named unit; replacement destinations are separate effects. | `drive::goal_cleanup`, `process::fall_back`. New tests `s0215_claim_22_a_unit_goal_is_kept_through_processing_and_cleanup`, `s0215_claim_22_guard_exception_with_a_unit_goal_falls_back_to_the_approach_destination` (`claims_s0207_goal.rs`). |
| 29 (enter: unit goals) | (a, already done) | The same ownership statement for goals naming a vehicle, infantry unit or aircraft, a transport included; refusal, replacement orders, entry guidance, boarding and servicing change destinations by their own rules, no promise of retention. | `transport::process_enter`, `transport::arrival`, `drive::goal_cleanup`; the S-0507 tests above and the existing depot protocol tests (`claims_s0314_depot.rs`). |
| Backing rows | (c) | D31 added. | none |

## Not touched

No M6c spec (S-0508, S-0509, S-0510, S-0515, S-0502, S-0517, S-0525, S-0514, S-0518, S-0519, S-0520)
and no M7 spec was started.
