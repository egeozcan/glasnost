# M6b step 2 (superweapons and drops): spec revisions since dcf30dc

Classification of every changed claim in `git diff dcf30dc HEAD -- spec/superweapons/S-0512-superweapon-charge.md spec/superweapons/S-0513-superweapon-effects.md spec/movement/S-0522-paradrops.md`.
Buckets: **(a)** changes behaviour the engine implements (implemented, with a test);
**(a, already done)** the committed code already did what the new text says (a test was added);
**(b)** a seam or a pointer into a spec that is not built (named);
**(c)** editorial, no behavioural change.

Every file's front matter (revision, `requests`) changed. That is (c) and is not repeated.
The per-spec ledgers ([S-0512](S-0512.md), [S-0513](S-0513.md), [S-0522](S-0522.md)) carry the
test names; the code sections are in `docs/development.md` ("Simulation (M6b: superweapons)" and
"Simulation (M6b: parachutes and drops)").

Requests: Q-0730, Q-0731, Q-0732, Q-0733, Q-0734, Q-0735, Q-0736, Q-0737, Q-0739, Q-0740, Q-0750,
Q-0751, Q-0752, Q-0753 and Q-0754 are answered by these revisions, and their interim rows and
markers are retired. Q-0738 and Q-0741 stay open (S-0513 claims 48 and 49 say what is
established and what is not); their interim rows are reworded. Filed: **Q-0800** (the position of a
multi-cell building in the strike witness test).

Changes of saved state or of the rules schema made in this step (the lead regenerates the goldens
and the schema file and bumps the save format):

- `AnimationType` gains `frame_count`, `canvas_height`, `rate` and `normalized`; the doc comments of
  `SuperweaponType::kind`, `pre_dependent` and `action` change.
- `ObjSw::silo_busy_until` and `FallState::chute_once` are new hashed state. `Cloud::delay` now holds
  the effective stage delay (0 freezes the stage).

---

## S-0512 Superweapon charge (rev 6 to 7)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 5 | (a, already done) | A low-power change that suspends the record in the update where it would complete prevents the completion; an announcement is requested even if a claim 2 (a) check in that update then removes the record. | `charge::house_update` (low-power check, `complete`, then `after_recheck`). New tests `claim5_low_power_change_in_the_completing_update_prevents_completion`, `claim5_without_the_power_change_the_same_record_completes`, `claim5_a_check_in_the_same_update_does_not_withdraw_the_announcement`. |
| 10 (Q-0737) | (a) | Before any source pick the source is cell (0, 0) and the warp is attempted; a granted first stage restarts from T whether ready or charging; a never-granted one is untouched; quirk off refuses. | `launch::second_stage` and the `ChronoWarp` branch of `launch::effect`. Tests `claim10_second_stage_before_any_pick_warps_from_cell_zero_and_restarts_the_first_stage`, `claim10_second_stage_before_any_pick_is_refused_with_the_quirk_off`. Interim marker retired. |
| 10 (S-0513 claim 34 pointer, X6/X17) | (c) | Cross-reference and scenario X17. | none |
| 23 (Q-0731) | (a) | Empty `Action=` keeps the previous action; the complete action vocabulary; a word outside it sets no action; an icon with no aim action fires at cell (0, 0) without aiming; any recognised nonzero action enters aiming. | `glasnost_rules::sim_econ::SUPERWEAPON_ACTION_WORDS`, `convert::sim_econ` (`superweapon`), `SwRules::new`, `Play::cameo_click` (app). Tests `claim23_action_vocabulary_words_are_distinct_actions_and_other_words_are_none`, `superweapon_kind_link_and_action_words_layer` (convert), `a_superweapon_icon_without_an_aim_action_fires_at_cell_zero` (app). Interim table row retired. |
| 24 (Q-0730) | (a) | The twelve literal kind and link words, whole-word and case-insensitive, no punctuation variants; an unknown word keeps the previous layer's value; a new type has neither. | `SUPERWEAPON_KIND_WORDS`, `superweapons::kind_of_word` (exact), importer fold. Tests `claim24_kind_words_are_the_twelve_literal_words`, `claim24_unknown_kind_and_link_words_leave_no_kind_and_no_link`, `superweapon_kind_link_and_action_words_layer`. The interim kind-word table is gone. Fixture words that were interim spellings (`ParachuteDrop`, `SpyAircraft`, `ChronosphereWarp`, invented action words) are changed. |
| Import mapping rows `PreDependent`, `Type`, `Action`, kind table, action table | (a) | As claims 23 and 24. | as above |
| X15, X16, X17 | (c) | Oracle scenarios. | none |

## S-0513 Superweapon effects (rev 9 to 10)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms: cloud stage, strike witness | (a) | Defined terms for claims 18 and 46. | `storm::stage`, `storm::cell_state` |
| 8 (silo picture) | (c) | Wording: the picture is presentation, the busy interval is state (claim 47). | none |
| 16, 19 (presence) | (a) | A cloud is present until a weather update finds its stage at least F - 1 (F the current image's frame count); removed in that update. | `storm::update`. Test `claim46_forward_cloud_strike_and_presence_ticks`. |
| 18 (Q-0740) | (a) | Strike when the cloud stage exceeds ⌊F/2⌋; the strike witness is chosen from the ground-layer listing (buildings and landed aircraft included, terrain objects and deck objects excluded) by the truncated table root of the in-cell position from the cell's top corner, ties to the first listed; debris when the pre-damage witness is not infantry and the first building, the witness or the height level changed, or nothing was held on bare land; every debris animation at the strike point. | `storm::cell_state`, `storm::strike`. Tests `claim18_debris_follows_the_top_corner_witness_not_the_centre_nearest_unit`, `claim46_*`. A multi-cell building's position: **Q-0800** filed, interim row. |
| 21 | (a) | A selected cloud type with no image, or a nonempty bolt list whose first type has no image, makes no cloud. | `storm::make_cloud`. Test `claim21_missing_images_make_no_cloud`. |
| 22 (Q-0734) | (a) | Action 109 applies invulnerability at its waypoint-slot cell: ground layer, no house, the imported duration, invoke animation at the surface plus 5 leptons without bridge lift, no radar event or announcement. | `triggers::action` (109), `protect::curtain`, `mp_triggers::action_coverage`. Test `s0513_claim22_waypoint_curtain_action` (in `claims_s0524.rs`). Interim row retired. |
| 29 (Q-0733 completed) | (a, already done) | Same rule for the curtain and the force shield, for buildings and mobile objects; a missed due turn postpones the phase; a returning object draws on its first protected turn after Bright hold, or restarts a fresh hold after an expired Initial rise. | `protect::tint_turn`. New tests `claim29_buildings_and_both_kinds_draw_in_turn_q_plus_ten`, `claim29_missed_turns_postpone_the_phase_and_schedule_no_draw_of_their_own`. |
| 33 (Q-0741 part 1) | (a) | The source marker is created whichever house is local and takes its creation draws on every pick. | `warp::pick_source` (`load::anim_created_at`). Test `claim33_the_marker_takes_its_creation_draws_whichever_house_picks`. The app keeps drawing the picture for the launching viewer only. |
| 46 (Q-0732) | (a) for ordinary playback, (b) for other playback | The cloud uses ordinary animation playback: effective delay from the creation draw, a constant random-rate delay or the `Rate` field (1 when absent, 0 freezes), normalised when the type is; first advance at c + d; strike at c + (⌊F/2⌋ + 1)d + 1; presence ends at c + (F - 1)d + 1; F is the current image's frame count. | `storm::stage_delay`, `storm::stage`, `storm::frames`; importer `frame_count`, `rate`, `normalized` (`convert::sim`, `lib.rs`). Tests `claim46_*`. Seam (b): `Start`, `End`, loops, `Reverse`, `PingPong`, `Shadow`, `Next` are not imported for clouds (S-0109 claim 28: no animation clock in the simulation). The sprite counts are read from the install, so they are checked in the local lane only. |
| 47 (Q-0735) | (a) for the plain busy interval, (b) for a queued different behaviour | Busy from acceptance until guard is queued 8 ticks later; a busy launch replaces the recorded target and plays the sounds and retargets a missile not yet created. | `nuke::accepts`, `nuke::silo_launch`, `ObjSw::silo_busy_until`. Tests `claim47_*`. Seam (b): a different behaviour already queued on the silo (S-0215 slots) is not modelled; the claim marks it unmeasured. Interim row retired. |
| 48 | (a) for the confirmed part, rest open | A member in its leaving stage drops its target and destination, and former driving does not continue; the whole-turn assumption must not be inferred. | `warp::join`. Test `claim48_a_warp_member_drops_its_target_and_destination`. The continuation contract stays open (Q-0738); interim row reworded. |
| 49 | (b) | No safe refusal is established for an off-map destination cell or a failed nearby search. | `warp::examine`, `warp::redirect` keep their departures; Q-0741 stays open; interim row reworded. |
| F4 (Q-0739) | (a) | H is the image header's canvas height, imported; a missing first-bolt image makes no cloud. | `storm::make_cloud`, `AnimationType::canvas_height`. Test `claim21_f4_height_follows_the_header_canvas_height`. `storm::BOLT_HEIGHT` removed. |
| Import mapping rows (`WeatherConBolts`, image header frame count, cloud art section, action 109), T2 heading, SW19 to SW25, Open questions | (a) for the rows named above, (c) for the rest | As claims 22, 46, F4 and the open questions. | as above |

## S-0522 Parachutes and drops (rev 3 to 4)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Cross-spec coordination paragraph | (c) | Lists the corrections other specs need. | none |
| 2 (Q-0736) | (a, already done) in the importer | Payload names are creating references of the infantry kind: an unknown name or a vehicle's name is registered as an infantry type and keeps its place; empty and no-type items are omitted; the spy plane counts the compact list. | The importer's creating references (`convert::refs`). New test `paradrop_payload_names_create_infantry_types`. `superweapons::payload_list` keeps a defensive skip for rulesets not made by the importer. Interim row retired. |
| 9 (Q-0750) | (a) | The entry outcome first and without a draw; then the standing position (its draw), then usable area, deck mark, qualification; the deck or ground mark follows the actual height (deck at or above surface + 416 on a high-bridge cell). | `paradrop::landing`, `paradrop::release`. Tests `claim9_entry_refusal_keeps_passenger_and_ammunition_and_takes_no_draw`, `claim9_refusals_after_a_clear_entry_take_the_standing_position_draw`, `claim9_the_deck_mark_follows_the_actual_height`. Interim row retired. |
| 10 | (a, already done) | Wording about ordinary releases and the low-release exception. | covered by the claim 9 tests |
| 11 (Q-0751) | (a) | A drop shot replaces the whole ordinary shot; a refused drop changes nothing; a success expires the rearm countdown now; both count as a shot so the run's pending round is made. | `combat::fire::shot`. Tests `claim11_a_shot_drop_replaces_the_ordinary_shot_effects`, `claim11_a_refused_shot_drop_changes_nothing_but_still_counts_as_a_shot`. Interim row retired. |
| 13 (Q-0752) | (a) | A held but unlisted name is an animation type (the importer creates it) and attaches normally; an absent type attaches nothing, takes no animation draw, and uses the parachute recurrence once in the release-tick update (quirk on) or the no-parachute one from the first update (off). | `paradrop::release`, `FallState::chute_once`, `fall::turn`. Tests `claim13_absent_type_uses_the_parachute_recurrence_once_in_the_release_tick`, `claim13_an_unlisted_name_is_a_type_not_an_absent_one`. Interim row retired. |
| 16 | (a) | An absent type takes no animation draws. | covered by the claim 13 test |
| 17 (Q-0753) | (a, already done) | Ordinary placement fall-back to rest at release: current none, guard or area guard queued, no target or destination; a falling unit is not movable; landing changes no order. | `transport::drop_out` and `targeting::movable`. New test `claim17_released_infantry_rests_with_guard_queued_and_takes_no_move_while_falling`. Interim row retired. |
| 18 (Q-0754) | (a) for palette and brightness, (a, already done) for order | The parachute is in the ground layer ordered by the unit's key plus its sort adjustment; its palette is the carrier's house-colour unit palette (row 3) and its brightness the cell's unscaled ground light sampled at creation. | `Live::attach_parachute`, `AnimDraw::carrier`, `Builder::live_anim`, `Painter::ground_light`. App test `parachute_follows_its_paratrooper` checks the carrier. Pixel-level checks (scenario P11) are local-lane. The creation-tick versus first-visible-opening brightness sampling is unmeasured (the spec says so). Interim row retired. |
| State table, Import mapping rows, P2/P4/P6/P7/P8/P10/P11, Open question 2 | (c) | Scenario and wording updates. | none |
