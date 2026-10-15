# M6b step 1: spec revisions since e882402

Classification of every changed spec in `git diff e882402 HEAD -- spec/`.
Buckets: **(a)** changes behaviour the engine already implements (implemented, with a test);
**(a, already done)** the committed code already does what the new text says;
**(b)** pointer into a spec M6b builds later (S-0503, S-0507, S-0511, S-0516, S-0512,
S-0513, S-0522, S-0524, the depot cycle of S-0314 claims 16 to 24, walls and gates,
teleport); **(c)** editorial, no behavioural change; **(e)** a new spec, or an M6c/M7 spec
that is not built (S-0502, S-0508, S-0514, S-0517, S-0601, S-0603, S-0605, S-0606, S-0607,
S-0608, S-0609, S-0611, S-0612, S-0615): recorded only.

Every file's front matter (revision, approved-on) changed. That is (c) and is not repeated.
No file under `spec/requests/` changed in the range, and no revision sets `answered-by` on an
open request.

---

## S-0301 Firing cycle (rev 11 to 12)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms (magnetic-beam weapon) | (e) S-0607 | The held target is the gripped unit; the grip lasts until S-0607 claim 12, independently of the wave. | No magnetic lift; `combat::data` keeps `mag_beam` unread. |
| 2 | (a, already done) | Each floating-point operation rounds by `quirks.chop_rounding` and S-0205; each conversion to ticks truncates. | `combat::fire::full_delay`. New test `claim_2_full_delay_rounds_by_the_rounding_switch` (both settings). |
| 5 | (a, already done) | A magnetic-beam weapon is not an effect weapon: full delay with its draw, no general own-wave block. | `CombatWeapon::effect` excludes `mag_beam`. New test `claim_5_magnetic_beam_weapon_is_not_an_effect_weapon`. |
| 5 (conditional rearming), 9 rank 4 | (e) S-0607 claim 11 | A gripping firer with a live wave is *rearming* against a target that is neither the gripped unit nor a building. | Seam in `combat::check` (no grip state). |
| F6 | (a, already done) | Restates `quirks.initial_ammo_above_capacity`: on keeps an explicit *J* above *A*. | `combat::fire::starting_ammo`; `claim_10_starting_ammunition` covers both settings. |
| Scenarios, open question removed | (c) | Magnetic control scenario added; the claim 5 open question is gone. | none |

## S-0308 Weapon choice (rev 8 to 9), S-0311 Veterancy (rev 8 to 9), S-0508 Garrisons (rev 3 to 4), S-0313 Firing positions (rev 4 to 5)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| S-0308 claim 13 and index table; S-0311 Terms and claim 15; S-0508 claim 18 | (e) S-0508 | An eviction or clearing action resets the occupant index to 0 before any release or removal; an additional entry keeps it; set-up removal keeps it unreduced. | Garrisons not built; interim row reworded in docs/development.md (Q-0660). |
| S-0313 claim 6 and Cross-spec changes | (e) S-0508 | Only the completed-action result of 0 is required; no ordering against individual releases. Contradicts the three above. | Filed **Q-0670**; interim row added. |
| S-0311 Open questions | (c) | Q-0660 question 2 stays open; the building-weapon fallback is not an experience fallback. | none |
| S-0508 other hunks | (e) | Garrison claims (not built). | none |

## S-0312 Death and destruction (rev 24 to 25)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 16 and its table | (e) S-0607 | A magnetic drop is not this claim's crash. | Seam (no magnetic lift). |
| 17 | (a, already done) / (e) S-0606 | Contacts are those held just before the destroying hit; a bunkered vehicle is let out instead (S-0606 claim 13); an entering vehicle is an ordinary contact. | `combat::death::building_death` collects contacts at the death; seam comment added for the bunker. |
| 24 | (e) S-0605 | Absorbed passengers come out most recently entered first; crate infantry count restored. | Seam (absorbers not built). |
| Ordering table row 4 | (e) S-0606 | Bunkered vehicle let out first. | none |

## S-0506 Missiles (rev 7 to 8)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, 6, 13, scenarios | (c) / (e) S-0607 | "Restrained by a magnetic beam" becomes *seized* (lasts past the grip). | `spawn::magnet_held` doc and the claim 6 and 13 comments reworded; still a seam. |

## S-0206 Turn rates (rev 12 to 13)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| Terms, 9 (rest row), 12 | (c) / (e) S-0607 | "Vehicle-seizing weapon" becomes *seized*; a seized unit does not spin or rest-turn, also after the grip ends. | Seam comment added in `turret.rs` rest row; `parasite::locomotor_replaced` doc reworded. |

## S-0212 Pathfinding (rev 36 to 37), S-0504 Naval units (rev 11 to 12)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| S-0212 nearby-cell table (bunker exit row) | (e) S-0606 | New caller row. | none |
| S-0212 S-0504 agreement note; S-0504 claim 9 | (c) | The S-0504 distance wording now agrees. | none |
| S-0504 (magnetic lift water landing pointer) | (e) S-0607 claim 17 | Pointer. | none |

## S-0215 Behaviours (rev 35 to 36)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 13 (restoration row) | (a, already done) / (e) S-0607 | The row applies when the replacement can end; magnetic-lift conditions added. Temporary drive at factory exit is unchanged. | `drive::begin_exit_track` path unchanged. |
| bunker clearing pointer | (c) / (e) S-0606 | Owned by S-0606 claim 6. | none |

## S-0214 Order targeting (rev 33 to 34)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| band-box selection | (e) S-0606 | A bunkered vehicle and an occupied bunker are excluded. | Seam (no bunkers). |

## S-0405 Power (rev 14 to 15), S-0408 Sell and repair (rev 18 to 19)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| S-0405 Terms (held unit), claim 8 release | (e) S-0607 | Held unit is the gripped unit; release drops or sets down. | Seam. |
| S-0405 due events | (e) S-0605 | Absorber entry and voluntary release make power due. | Seam (`econ::power`). |
| S-0408 first processing, scenarios | (e) S-0606 | Bunkered vehicle takes the release track. | Seam. |
| S-0408 passengers | (e) S-0605 | Most recently entered first. | Seam. |

## S-0108 Building sprites (rev 10 to 11), S-0414 Victory (rev 12 to 13), S-0415 MCV (rev 14 to 15)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| S-0108 claim 5 | (c) | Rewording of deferred-change visibility. | none |
| S-0108 claims 17 and 20, Terms | (e) S-0605, S-0603 | Held-infantry release; recycling-building slot row. | none |
| S-0414 totals | (e) S-0603 | Recycling consumption is not a destruction. | none |
| S-0415 backing | (c) | S-0606 claim 11 qualification. | none |

## S-0516 Cloak (rev 4 to 5), S-0513 Superweapon effects (rev 4 to 5), S-0522 Paradrops (rev 2 to 3)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| S-0516 cloak blockers | (b) | "Seized" blocker wording. | M6b builds S-0516. |
| S-0513 warp group | (b) | Bunker link ends in place with walls-down effects. Switches `superweapon_block_one_layer`, `superweapon_list_cut_by_removal` added. | M6b builds S-0513. |
| S-0522 claims 5 and 16, Import mapping | (b) | South-edge draw sequence and the draw summary reworded; fall damage only for magnetic lift. | M6b builds S-0522. |

## S-0701 Sound selection (rev 12 to 13)

| Claim | Bucket | Change | Code location / evidence |
|---|---|---|---|
| 30 | (a), not done | Presentation scheduling must be deterministic. Revision 14 answers Q-0671 with the logical cycle-end clock, sample durations, waits and draw assignment; not built yet, our passes run on host time. | `crates/audio` `Audio::frame`, `Mixer::service`. Row in docs/development.md (Audio). |
| consumption death sounds | (e) S-0603 | none | none |

## New or unbuilt specs: S-0502 (rev 2 to 3), S-0514 (rev 3 to 4), S-0517 (rev 2 to 3), S-0601 (rev 1 to 2), S-0609 (rev 2 to 3), S-0608, S-0611, S-0612, S-0615 (rev 1)

All (e). Recorded only. Existing code they will change:

- S-0615 claims 1 to 3 (infantry death animations for death kinds 3 to 7 and 10): `combat::death::infantry_death`, which today leaves play with no animation for those kinds.
- S-0611 (mutation, death kind 9): the same function, which plays Die2 for kind 9.
- S-0615 claim 16 (disc laser has no house): the disc-laser shot path in `combat::fire` and `combat::projectile` (`disk_laser`).
- S-0609 claim 5 (no bunker exclusion for spin-up): gattling is not built.

## Glossary (rev 36 to 38), spec/README.md

(c) and (e): new terms (seized, gripped unit, erasure amount, arrival hold, leaving stage,
recycling building, absorbing building) for the specs above; the audio-clock departure rule
pairs with S-0701 claim 30.

## Quirk switches added (commit 6a9ebc0, `rules::Quirks`, all default on, not implemented yet)

`airborne_deploy_order_dropped` (S-0517), `drain_relink_strands_previous`,
`drain_position_check_vehicles_only` (S-0608), `superweapon_block_one_layer`,
`superweapon_list_cut_by_removal` (S-0513, S-0611), `mutation_test_ground_record`,
`refused_mutation_frees_position`, `infantry_animation_occupancy` (S-0611),
`disc_laser_no_house` (S-0615). The `dominator_*` names of S-0612 sit in `spec/superweapons/`,
which the quirk inventory test does not scan; they are not added (S-0612 is M7).

## Requests

- Filed: Q-0670 (garrison index reset ordering), Q-0671 (deterministic audio schedule).
- Q-0654 stays open: S-0501, S-0208, S-0105 and S-0205 were not revised in the range.
- Q-0660 question 2 stays open (S-0311 Open questions, S-0508 claim 18).

## Spec revisions since 2878894

Buckets as above. Request answers retired from `docs/development.md` in this round: Q-0672 to
Q-0675, Q-0680, Q-0697, Q-0710 to Q-0719. The cloak requests Q-0720 to Q-0729 (S-0516, and
their pointers in S-0203 revision 28, S-0209 revision 34 row WP25, S-0210 revision 21,
S-0410 revision 20 and S-0507 revision 6) belong to the cloak work and are not touched here.

- **S-0202 (rev 27 to 28), (a).** Claim 13: the map-generator stream starts at seed 0,
  independently of the game seed (Q-0711). Continuing a session across games is a seam.
- **S-0207 (rev 24 to 25), S-0208 (rev 29 to 31), S-0417 (rev 13 to 15), (a, already done).**
  A tethered teleporting infantry unit reports clear in its jump's arrival update, before the
  entered-by raise (Q-0697). S-0208's air-release height rule is S-0507 claim 13's (Q-0672).
  S-0417's hover destination split and arrival outcome table follow S-0503 claims 15 and 16;
  `econ::release::arrival` reads the order-level destination.
- **S-0209 (rev 30 to 34), (a, already done) and (b).** Hover flag rules point to S-0503
  claims 13 and 15; bridge drops to S-0511 claim 9. The dying-vehicle escape gate accepts only
  *clear* and *moving blocker* (`transport::escape`), so a stationary friendly outcome kills.
- **S-0211 (rev 16 to 17), (a, already done) and seam.** Moves already recompute their cells
  in full (`drive`, `walk`, `hover`, `teleport`). The local-area resize action (S-0524 claim 8)
  is not built; when it is, it recomputes every legal cell, not only cells gained or lost.
  Claim 15 (a)'s historical responsibility is S-0511's (Q-0717).
- **S-0212 (rev 39 to 40), S-0213 (rev 31 to 32), (a).** A hover's searches and completion
  use its movement destination, kept by a stop with equal points (S-0503 claim 16).
- **S-0214 (rev 35 to 36), (a, already done).** The repair cursor is S-0511 claim 21's.
- **S-0215 (rev 39 to 41), (a).** Falling and due processing (S-0511 claim 10); the
  centre-cell revert (S-0511 claim 22, Q-0719); the depot tether and untethered release rows
  (S-0314 claims 17 and 20); passenger sync before the queued-entry retry, the parasite
  delivery after it (S-0314 claim 23).
- **S-0301 (rev 13 to 14), (a).** A falling firer is *illegal* at rank 4 (S-0511 claim 10).
- **S-0314 (rev 17 to 18), (a).** See `depot-cycle.md`; Q-0674 and Q-0680 retired, Q-0790 filed.
- **S-0404 (rev 16 to 17), (a, already done).** Every imported foundation word has a present
  exit list, empty for 0 × 0; exhausting it starts no perimeter search.
- **S-0501 (rev 12 to 13), S-0507 (rev 4 to 6), (a, already done).** The air-unload
  placeholder centre (Q-0673) and air release (Q-0672) are in `transport.rs`.
- **S-0503 (rev 3 to 5), (a).** See `S-0503.md`; Q-0675 retired, Q-0679 filed.
- **S-0511 (rev 3 to 4), (a).** See `S-0511.md`; Q-0676 to Q-0678 filed.

## Spec revisions since 17:50 (cloak round)

Buckets as above. Request answers retired from `docs/development.md` in this round: Q-0676 to
Q-0679, Q-0720 to Q-0729, Q-0790, and Q-0327 (S-0701 claims 21 and 32). Filed: Q-0791, Q-0792
(S-0516), Q-0793 (S-0207 exit tracks), Q-0794 (S-0314 driving destination point).

- **S-0516 (rev 5 to 6), S-0203 (rev 28), S-0209 (rev 34), S-0210 (rev 21), S-0410 (rev 20),
  S-0507 (rev 6), (a).** See `S-0516.md`; the two new quirks
  `cloak_field_building_interrupt` and `mobile_sensor_removal_centre` are built. WP25 needs no
  disguise house; a no-house disguise shows the neutral radar colour.
- **S-0503 (rev 5 to 6), (a).** See `S-0503.md`; claim 18's held destination point is
  `HoverState::held` (Q-0679).
- **S-0207 (rev 25 to 26), S-0511 (rev 4 to 5), (a).** See `S-0511.md`; `Track::height`
  (claim 24, Q-0676), the retained bomb mark (Q-0677), the combined span extent and
  `quirks.collapsed_pier_span_search` (Q-0678).
- **S-0314 (rev 18 to 19), (a).** See `depot-cycle.md`; claim 17's movement-report table, the
  empty place 0 and the arrival's ready mark (Q-0790).
- **S-0504 (rev 14), (a).** Claim 7's two other amphibious zones are tested
  (`claim_7_other_amphibious_zones_match_the_flag`); Q-0643's marker is retired.
