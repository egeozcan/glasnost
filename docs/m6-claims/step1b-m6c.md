# M6c step 1b claim ledger (spec revisions that landed on main after the code took them in)

Part 2 of step 1, branched from main at a1921ef. The old state is the spec file at commit 401c33b (`git diff 401c33b HEAD -- spec/<path>`). Each changed claim is classified as *satisfied* (already true, with the test that shows it), *changed* (code changed, with the test) or *pointer* (the claim only points at another spec's claim). Per-spec ledgers: [S-0502.md](S-0502.md), [S-0508.md](S-0508.md), [S-0517.md](S-0517.md). S-0525, S-0514, S-0518, S-0519 and S-0520 belong to the parallel track and are not touched here.

| Spec | Revision | Answers | Result |
|---|---|---|---|
| S-0312 | 29 (from 28) | | claim 16: the pending aerial crash of a jumpjet gets its next update (S-0502 claim 3), a design rule. Satisfied: `claim_3_crash_start_wakes_an_idle_hover` (the former interim rule of Q-0979 is now the spec's). |
| S-0313 | 9 (from 8) | | claim 6: assault animation at list position i >= 10 uses point i mod 10 (S-0508 claim 25). Changed: `garrison::assault`; test `claim_25_positions_ten_and_later_use_the_remainder`. |
| S-0517 | 6 (from 4) | Q-0982 | claim 6: the animated-undeploy search settings (S-0502 claim 23's last row): changed, `inplace::landing_first_take_off`. Claim 7, F1, F2, Table V, the import rows for `Rate`, `RandomRate`, `Normalized`: the completion is exactly on t0 + (E - 1) d + 1 (undeploy: E - 2) with the base delay (no normalisation): changed, `inplace::completed` and `stage_delay`; tests `claim_4_deploying_presentation_and_f1`, `claim_6_undeploy_presentation_and_f2`, `claim_7_exact_completion_and_the_base_delay`. The presentation's playback (first displayed advance after creation, random rate, normalisation) belongs to the viewer: the simulation has no animation clock (S-0109 note below). |
| S-0402 | 25 (from 24) | Q-0900 | claim 3b, F1a, H27, Terms (destination means the order-level destination): satisfied. `harvest::seeking` tests `Object::dest`, the object turn runs due behaviour processing before locomotion (`tick::object_turn`), S-0207 claim 26's cleanup removes the destination in the next locomotion update; test `claim_3b_loading_starts_after_the_destination_is_removed` (loading starts strictly after the removal tick; the first bale 19 ticks after loading starts, over eight starting distances). No AI game course moves: the sweeps below read the same before and after. |
| S-0208 | 32 (from 31) | | claim 3: the infantry jumpjet's adjustment uses the occupant-aware choice (S-0502 claim 23). Pointer; changed in `jump::standing_choice` (`walk::choose_position`), test `claim_23_search_settings_and_the_standing_choice`. |
| S-0207 | 30 (from 29) | | claim 26: the due harvest seeking processing in T + 1 still sees the destination. Satisfied (S-0402 claim 3b's test). |
| S-0209 | 39 (from 38) | | claims 1 and 13: jumpjets do not use the aircraft's positive-listing-height exclusion; low-flight listing and marks; hover/cruise no new listing; clearance listings (S-0502 claims 4 and 22). Changed: `fly::remove_placement`, `fly::place_after_step`, `jump::clearance`; tests `claim_4_occupancy_marks`, `claim_4_a_listing_survives_into_cruise_and_is_not_transferred`, `claim_22_a_retained_listing_counts_for_its_owner`. |
| S-0211 | 18 (from 17) | | claim 5, Table, scenario: the recompute events of jumpjet flight (S-0502 claim 26). Changed; test `claim_26_flight_recomputes`. |
| S-0212 | 43 (from 42) | | T5 and claim 25: four jumpjet rows (S-0502 claim 23). Changed: `jump::Use`, `jump::search`, `inplace::landing_first_take_off`; test `claim_23_search_settings_and_the_standing_choice`. |
| S-0502 | 5 (from 4) | Q-0975 to Q-0981 | claims 3, 4, 5, 6, 16, 17, 22, 23, 24, 25, 26, F1, F6, Import mapping: see [S-0502.md](S-0502.md). Changed throughout `jump.rs` and `jump/fly.rs`; save format 38. New open points with interim rules: Q-1010, Q-1011, Q-1012. |
| S-0109 | 28 (from 27) | | claims 3 to 5 and 8, creation table, companion scenario: the in-place deploy presentation is created in the vehicle's turn with no initial loop delay, forward for a deploy and reversed for an undeploy, its playback separate from the vehicle's base-delay clock. Satisfied: `inplace::start_presentation` raises `Effect::Anim` (reverse for an undeploy) in the vehicle's turn; the simulation has no animation playback (S-0109 claim 14 note in `load::anim_created_at`); the completion clock is S-0517 claim 7's. |
| S-0213 | 33 (from 32) | | claim 8: a stop order's jumpjet reroute table. Changed: `orders::apply_other` calls `jump::stop_ordered`; test `claim_24_stop_in_each_flight_state`. |
| S-0215 | 49 (from 48) | | claim 2: due processing sees the destination state before that turn's locomotion; the field arrival of S-0402 claim 3b. Satisfied (the same test). |
| S-0508 | 8 (from 7) | Q-0990 | claim 14 pointer, claim 25, G19: changed, see [S-0508.md](S-0508.md). |

## Results of the runs

- Sim-gate goldens (`GLASNOST_UPDATE_GOLDEN=1 cargo test --release -p glasnost-sim-gate -- --include-ignored`): unchanged. The determinism scenario holds no jumpjet and no in-place deployer, and its harvesters already ran as claim 3b says, so the golden hashes did not move and nothing was regenerated.
- AI whole games, seeds 1 to 32 (`GLASNOST_SWEEP=1-32`, [docs/development.md](../development.md) M6c row 2a): the naval sweep passed 29 of 32 before (seeds 5, 7 and 17 send no ship across) and 29 of 32 after; the superweapon sweep passed 29 of 32 before and 29 of 32 after (the same seeds). The fixed-seed tests `builds_a_navy_on_a_water_map` and `builds_and_fires_a_superweapon` pass before and after.

## Interim rules added

- Q-1010: a flying jumpjet is listed and marked when its effective height is below 208 leptons (S-0502 claim 4 against S-0102 claim 3's table).
- Q-1011: a grounded jumpjet's own mark is lifted for the claim 16 search (claim 23's sentence against scenario J2's own-cell result).
- Q-1012: every clear of the order-level destination of a moving jumpjet reroutes it as a stop order does.
