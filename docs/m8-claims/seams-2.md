# M8 seam sweep 2: claims coded but not tested

Brief: `.claude/worktrees/M8-SEAMS-2-BRIEF.md`. The list below comes from the rows of
`docs/m8-claims/*.md` for intakes 15 onwards, C1, C5, C7, `S-0109-playback.md`, `S-0612.md`,
`mapflags.md` and `intake-M-0017.md`, plus sweep 1's *Left* rows whose reason no longer holds, that
say "partly", "not tested", "untested", "by construction", "by inspection" or "built, no test".
Each row's spec is approved and current, and no open request blocks it. Left out: the rows that
need a real install, and S-0607 (intake 45 revises it). The specs as read: S-0401 r17, S-0109 r41,
S-0501 r62, S-0317 r57, S-0306 r38, S-0207 r82, S-0402 r80, S-0215, S-0417 r68, S-0212 r87, S-0503,
S-0319 r26, S-0504. Q-1762's file still says open, but S-0319's `requests:` field shows it answered
(r9 and r26), so its rows were not treated as blocked.

**Result.** 22 rows: 17 tested by new tests through the real order and tick path, and 5 stale
rows closed by tests that already existed. No test found the code differing from the spec, so no
simulation code changed. There is no save format bump and no golden change, and the oracle arena
and the M6c, M7a and M7b sweeps are unaffected. Under the brief's rule, sweeps are needed only
for a simulation change, so none were run. No spec request was filed.

## Tested

| # | Spec | Claim or scenario part | Ledger state before | Plan and result |
|---|---|---|---|---|
| 1 | S-0401 | claim 10, E8 part c: other bridge parts, the usable area, the tileset | partly | `claim_10_e8_part_c_bridge_usable_area_and_tileset` (`claims_s0401_r17.rs`). None of these get ore: a bridge-body cell, a level-2 cell at the usable edge, or a tileset whose tiberium flag is off. A tile number outside the theater and a raised cell inside the usable area do get ore. `germinable` agrees. Both switch positions, seeds 1 to 5. Code agrees. |
| 2 | S-0401 | claim 10, E8 part c: an invisible building | partly | `claim_10_e8_part_c_invisible_buildings`. Both invisibility flags let ore in. An invisible building listed first hides a visible one, and a visible building blocks. Code agrees. |
| 3 | M-0013 | secondary spread out of new cells, on multiples of the spread field | by construction | `m0013_secondary_spread_from_new_cells_on_the_schedule`: a corridor C, A, B with growth 2 and spread 7. Each gain falls on a multiple of the spread field. Code agrees. |
| 4 | S-0109 | claim 3, a follow-up type's random rate draws in the animation's turn | partly | `claim3_a_follow_up_with_a_descending_random_rate_draws_its_delay_in_its_turn` (`claims_s0109_playback.rs`). A descending pair makes one ranged draw at the switch. A pair that collapses to one value makes no draw. Code agrees. |
| 5 | S-0109 | claim 4, a normalised delay at two speeds | partly | `claim4_a_normalised_delay_follows_the_speed_at_creation`: the delay follows the speed at creation; with the quirk off it does not. A speed change after creation is ignored. Code agrees. |
| 6 | S-0109 | claim 18, damage timing at two speeds | partly | `claim18_a_normalised_animations_damage_timing_follows_the_speed`: deliveries come at advances 1, 3, 6 and 8, times the delay. Code agrees. |
| 7 | S-0109 | claim 9, ping-pong | partly | `claim9_a_ping_pong_animation_turns_and_never_ends`. Code agrees. |
| 8 | S-0109 | claim 10, a shadow type's last pass | partly | `claim10_a_shadow_types_last_pass_ends_at_the_loop_end`. The shadow and plain fixtures run in separate worlds, because an S-0203 claim 10 removal skip showed up when they shared one. Code agrees. |
| 9 | S-0501 | claim 48 (a), range searches pass over a resting release | built, by construction | `claim_48_a_a_range_search_passes_over_the_resting_release` (`claims_s0501_released.rs`): an anti-air vehicle of the other house, three cells away, never targets the release in 300 ticks. A flying control is acquired. Code agrees. |
| 10 | S-0501 | AC30 part k (5), the airborne scan | by construction | `ac30_k5_the_airborne_scan_acquires_the_leaving_release`: a defence acquires the leaving release in flight while it is tracked and still `ground_filed`. Code agrees. |
| 11 | S-0317 | claim 1, a detonation's and the barrel debris's pieces have no house | by inspection | `claim_1_a_detonations_pieces_have_no_house` (`claims_s0317.rs`): a crusher's attack sets off a barrel overlay. All pieces have no house, and the barrel debris is made. Code agrees. |
| 12 | S-0306 | claim 23, the late tests (rank 9) | not tested on their own (sweep 1 Left) | `claim23_rows_outrank_the_late_target_tests` (`claims_s0306_modes.rs`): a hunter-seeker firer gives mode 1 *illegal* and mode 0 *out of range*, with and without the range test. Code agrees. |
| 13 | S-0207 | claim 28, a sale with a miner not full: the ring-search cell | by construction | `claim_28_a_sale_with_a_part_load_jumps_to_the_ring_search_cell` (`claims_s0207_r82.rs`): the remembered cell is dropped and the miner jumps to the ring-search cell, never to the scatter cell. The jump comes in the tick of the sale's first processing (the sale stage is still *begun*, so the contact holds). The jump installs driving again. Code agrees. |
| 14 | S-0207 | claim 28, a sale with a ground-layer vehicle in the remembered cell | by construction | `claim_28_a_sale_with_the_remembered_cell_taken_drives_there`: driving takes the remembered cell, the scatter jump is cancelled, and the first move is a driven step. Code agrees. |
| 15 | S-0207 | claim 28, a sale whose ring search finds nothing (S-0402 claim 17 case (f)) | by construction | `claim_28_a_sale_with_nothing_found_stays_on_the_docking_cell`: the miner goes *idle* with driving installed and no destination, and stays on the docking cell. Code agrees. |
| 16 | S-0501 | claim 32, the revert at the loss (S-0215 claim 10) | built, no test (sweep 1 Left) | `claim_32_the_loss_reverts_an_overridden_aircraft`: the override's target is taken inside, and the aircraft is reverted at once to its behaviour, its target and its destination. The override and the loss are called directly (`acquire::override_attack`, `objects::take_inside`), not through an order and a tick, because the fixture has no transport type. Code agrees. |
| 17 | S-0501 | claim 32, the revert brings back the very object that left | built, no test (sweep 1 Left) | `claim_32_the_revert_brings_back_the_object_that_left`: taken inside, the object comes back as the target. At a removal the revert happens too; the target it leaves is not stated by the claim, so it is not asserted. Called directly, as row 16. The kill path (`mark_killed`) has no test of its own. Code agrees. |

## Stale rows closed (an existing test covers them)

| # | Ledger | Row | Covered by |
|---|---|---|---|
| 18 | `intake17-S-0501.md` | Claim 8, a docking position outside the foundation keeps the contact ("not tested") | `claim_48_c_tethered_type_keeps_its_contact_outside_the_foundation`, `ac30_k7_e_person_order_ends_a_pad_bound_contact_outside_the_foundation` (intake 31's OUTPAD and WESTPAD) |
| 19 | `intake43-S-0417.md` | 17, a new destination does not cut *R* short ("partly", for intake 44) | `claim_21_walker_new_destination_waits_for_the_delay`, `state_table_drive_new_destination_waits_for_the_delay` (intake 44) |
| 20 | `intake36-S-0402.md` | 18, the placement fall-back ("waits on S-0207 r82") | `claim_28_placement_installs_driving` (intake 39) |
| 21 | `intake36-S-0402.md` | Terms, 5a, 9, 10, 17 (a) to (f), 18, H8 ("waits") | built at intake 39; its ledger rows carry the tests |
| 22 | `seams-1.md` Left | S-0503 claim 11, a naval-yard hover's water spin | `claim_21_docked_hover_lost_at_a_naval_yard_by_default` (intake 18) |

## Found, not a test gap

| Spec | Row | Finding |
|---|---|---|
| S-0504 | Terms (pitch and roll) and claim 15 (`intake26-S-0504.md`) | Rocking is not built at all (S-0303 claim 20, S-0614 claim 10, S-0210's rates). The ledger already names this as a seam. It is a missing feature, not a missing test. Closed by M8 A4 rocking ([a4-rocking.md](a4-rocking.md)). |

## Left (no test written)

| Spec | Claim or scenario part | Ledger state | Why left |
|---|---|---|---|
| S-0501 | claim 47 (a), a destination under paralysis, erasure, warp or a switched-off locomotor | partly | Needs one of the holds on a landed aircraft. Paralysis is the parasite hold (S-0614), so the fixture is costly. Each hold has its own tests elsewhere. |
| S-0501 | claim 48 (c), the low release's lower-fraction branch | built | The spec rates the low release low. A fixture needs a target fraction below 1 at the release. |
| S-0501 | claim 18, the in-flight pad search | satisfied | Sweep 1's reason holds: no fixture makes a grounded aircraft fall back with a pad and no contact. |
| S-0501 | T1, *firing* and *refiring* with no target | built, no test | Sweep 1's reason holds (`unmeasured`, no scenario places the loss there). |
| S-0501 | claim 48 (d), an AI house's release | seam | A design seam: our AI takes the human-player rows. |
| S-0207 | claim 28, a sale with the miner still in *enter* | by construction | The same assignment paths as rows 13 to 15. A fixture needs a miner in contact and off the docking cell at the sale. |
| S-0207 | claim 28 exclusions, warp-pending and gripped | by construction | Gripped needs S-0607, which intake 45 revises. |
| S-0306 | claim 23, the magnetic-beam *rearming* | not tested on its own | Needs a live wave and a grip (S-0607, intake 45). |
| S-0212 | claims 17 and 18, the flattening's threat test | partly | Needs a synthetic threat shape. No gate or arena unit has a coefficient, so it is rare in play. |
| S-0319 | claims 7 (no skip at no health), 10 (vein, a 127-entry list) and 13 | by construction | Rare. Claim 10 needs a 127-entry overlay list. |
| S-0317 | claim 7, no skip for an object with no health left | by construction | No real path holds such an object in the cell. |
| S-0316 | claim 7, no sound | by inspection | Presentation. The empty match arm in `glasnost_audio::events` is the whole rule. |
| S-0316, S-0317 | the drawing "partly" rows (claims 5, 13, 24, 25) | partly | Drawing only. Test art has no VPL. |
| S-0612 | claims 1 and 4 | partly | Our own AI tuning. |
| S-0213 | *Deployed* | satisfied, no test | Sweep 1's reason holds. |
| S-0301 | claim 14 | seam | Sweep 1's reason holds. |
| S-0212 | 23 "stays in its team" (`intake44-S-0212.md`) | by construction | Our engine has no teams. |
| S-0315 | Terms (`mapflags.md`) | by construction | A naming row: no S-0315 code reads the amount. |
| S-0319 | scenarios BA1 to BA12, S-0401 E5/E8/E11, other oracle scenarios | by construction / not ours | Oracle scenarios. |
| C1 (S-0703, S-0701) | host-clock and missing-file rows | partly | Host clock and real files, not simulation. |
| any | "Quirks (decided)" rows | by construction | No value to test. The quirk lint passes. |

## Ledger rows changed

`intake42-S-0401.md` (claim 10 E8 part c, M-0013 secondary spread), `S-0109-playback.md` (claims 3,
4, 9, 10, 18), `intake17-S-0501.md` (claim 8, claim 48 (a) searches, AC30 part k (5)),
`intake9-S-0501-S-0318-S-0503.md` (claim 32 revert), `intake5-S-0306-S-0309-S-0107.md` (claim 23
rank 9), `c5-S-0317.md` (claim 1), `intake39-S-0207.md` (claim 28 sale), `intake43-S-0417.md` (claim
17), `intake36-S-0402.md` (claim 18; Terms and the chrono rows), `seams-1.md` (Left: S-0501 claim
32, S-0306 claim 23, S-0503 claim 11).
