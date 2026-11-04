# S-0513 r108, S-0309 r114, S-0306 r72, S-0102 r42, S-0301 r70, S-0514 r17, S-0607 r39: claim ledger, M8 step A3 intake 89

Three deltas, built together because they answer intake 84's four requests:

- **S-0513 r91 to r108** (commit 83a4d8f) answers **Q-2416** (a held temporal target's *rearming*
  outranks a protected target's *illegal* and a hidden target's *impossible*, on the map and off
  it), **Q-2418** (building, SAM, deployed-infantry and sticky firers of a member that waits to
  land) and **Q-2419** (claim 34: a listed object in the air is not examined by height, an
  aircraft by its in-the-air answer; no jumpjet state and no layer is read). It adds SW12 parts
  l7a, l7b, l9, l10 and b8, and the Changes item for S-0301 claim 9 (*A held temporal target
  within rank 4*).
- **S-0309 r95 to r113** (commit c236355) changes claim 27 only: it repeats S-0513 r108's firer
  rules for a held member (building, SAM, beyond range, deployed infantry), and keeps the rule for
  a member a revert gives back while it is off the map.
- **S-0306 r65 to r72** (commit 4e0d4ea) answers **Q-2417**: claim 17 gains *An aircraft that
  cannot land, in flight*. Such an aircraft stays in the ground list (S-0102 claim 4) after a
  ground placement or a warp's hold, warped or not; only S-0309 claim 7's ground-layer pass reads
  that list; the seizing and erasure tests, the airborne scan and claim 20 (c) read the layer from
  state and height. Scenario P46.

Five more revisions came with the coordinator's scope addition. Each restates, in the spec that
owns the rule, what the three deltas above give, so the build cites them and adds no behaviour:

- **S-0102 r41 to r42** (4d15336, answers **Q-2419**): claim 3, a unit with jumpjet movement that
  is airborne (S-0307) is never a warp member, listed or not; one listed and not airborne is
  examined (S-0513 claim 34). Backing: no block has such a unit listed and airborne at once.
- **S-0301 r69 to r70** (1bc8013, answers **Q-2416**): claim 9 gains *A held temporal target
  within rank 4*, the S-0513 r108 Changes item.
- **S-0309 r113 to r114** (e5a76be): claim 7, the ground-layer pass considers an aircraft that
  cannot land in flight; claim 9, the airborne scan does not leave it out once it is tracked and
  above height 0.
- **S-0514 r16 to r17** (048e51d): claim 1, the erasure test reads the layer from state and
  height, not the list: no erasure of such an aircraft above height 0.
- **S-0607 r38 to r39** (38a4638): claim 3, the same for the seizing test.

None adds a quirk name.

Intake 84's interim rules for the four requests are replaced. Three requests filed: **Q-2466**,
**Q-2467**, **Q-2468** (Seams).

## Built

- **The held-temporal rearming within rank 4** (S-0513 r108 claim 39a, S-0301 claim 9 Changes
  item, Q-2416). `combat::check::check` answers the held-target *rearming* (S-0514 claim 19)
  before the protected target's *illegal* (S-0513 claims 27 and 28) and the hidden target's
  *impossible* (S-0516 claim 24), so a firer whose temporal beam is attached keeps a protected or
  hidden target at S-0309 claim 29's check, on the map and, for a warp member, off it. Intake
  84's interim (the drop off the map) is gone.
- **A revert that gives a member back off the map** (S-0309 r113 claim 27, S-0513 r108 claim
  39a). New hashed field `combat.revert_gave_departed`. Every caller of the behaviour revert
  (`beh.revert()`: `held::holders_drop`, both branches, `teleport`, `bridges`, `cloak::machine`
  and `air`, which is every call in the simulation) calls `held::note_revert` once it has
  restored the target; when that target is a departed warp member the holder is claim 27's case,
  and `held::beam_held` no longer covers it, so the holder drops it at the start of its next turn
  whatever its beam. The mark clears at that turn's start.
- **Claim 34's in-the-air test** (S-0513 r108, Q-2419). `warp::in_air` now reads S-0501 claim
  18's in-the-air answer for an aircraft and `combat::weapon::airborne_by_height` (S-0307: at
  least 208 leptons above the surface, above the deck while the on-deck state is set) for every
  other object. No jumpjet state and no layer is read. Intake 84's interim (a listed jumpjet
  above its surface left alone) is gone.
- **An aircraft that cannot land, in flight** (S-0306 r72 claim 17, Q-2417). `AirState::
  warp_filed` becomes `kept_filed`: set by a ground placement of a type that cannot land
  (`air::placed`) and at the end of a warp member's hold, cleared at a crash
  (`effects::layer_refile`). `warp::in_ground_layer` is gone: the erasure test (`temporal`), the
  airborne scan and claim 20 (c) (`projectile`) read the warp row (`warp::in_warp_row`) and the
  object's own rows. Only the ground-layer pass of S-0309 claim 7 (`acquire`, pass 1) reads
  `air::kept_filed`, and considers such an aircraft in flight there as a landed aircraft. The
  aircraft pass (pass 0) takes every aircraft anyway when the search looks for aircraft, so it
  holds by construction. The drawing (`app::mapview::layers`) files such an aircraft that it first
  sees in flight, as after a load, in the ground layer.
- **No new code for the building, SAM, deployed-infantry and sticky firers** (Q-2418). The rows
  r108 gives follow from S-0215 claims 32, 37 and S-0517 claim 16 as built, with the r108
  *rearming* above; the tests below show them.
- **Formats.** One hashed field (`revert_gave_departed`), so save format 95 to 96 and AI game
  format 58 to 59. Wire stays 12.

## Claims

| Claim part | Status | Where, and why |
|---|---|---|
| S-0513 claim 39a / S-0301 claim 9 Changes: the held-temporal *rearming* outranks a protected target's *illegal* and a hidden target's *impossible*, on the map | tested | `claims_s0513_r89::r108_claim39a_the_held_temporal_rearming_outranks_a_protected_targets_illegal_on_the_map`, `..._hidden_targets_impossible_on_the_map` |
| Same, off the map: the firer keeps a protected or hidden member while it waits to land | tested | `r108_claim39a_a_firer_keeps_a_protected_or_hidden_member_off_the_map` |
| *The firer*: a building without the SAM flag drops the member at its first processing before the lock (SW12 l7a) | tested | `r108_claim39a_sw12_l7a_a_building_drops_the_member_at_its_processing_before_the_lock` (both switch positions; gravity-launched straight shots so the lock falls after the building's next processing) |
| *The firer*: a building that holds the member at the lock keeps it, the *rearming* row, no shot, no turn, no *guard* (SW12 l7b) | tested | `r108_claim39a_sw12_l7b_a_building_that_holds_the_member_at_the_lock_keeps_it` |
| *The firer*: a building that holds the member again after the lock holds it as at the lock | tested | `r108_claim39a_a_building_that_dropped_the_member_holds_it_again_after_the_lock_as_at_the_lock` |
| *The firer*: a delayed shot pending at the launch answers its *rearming* for the member | tested | `r108_claim39a_a_building_with_a_delayed_shot_pending_answers_its_rearming_for_the_member` |
| *The firer* / S-0309 r113 claim 27: a member a revert gives back off the map is dropped, whatever the beam | tested | `r108_claim39a_a_member_a_revert_gives_back_off_the_map_is_dropped_whatever_the_beam` |
| *The firer*, SAM building short of power and not charged: *impossible*, the drop at the first check tick while the member is off the map, no *guard*, the beam kept | tested | `r108_claim39a_a_sam_building_firer_short_of_power_and_not_charged_drops_the_member_at_the_check` (launch at a tick leaving 1 by 16, so the check tick falls strictly between the departure and the appearance) |
| Same, charged (2 chargers, S-0405 claim 8): *rearming*, kept through the check | tested | `r108_claim39a_a_charged_sam_building_firer_short_of_power_keeps_the_member` |
| SW12 l10: a SAM building short of power drops an aircraft target on the map by the first check tick | tested | `r108_claim39a_sw12_l10_a_sam_building_short_of_power_drops_its_target_by_the_first_check_tick` (with a control that keeps the plant) |
| *The firer*, beyond range, *sticky*: drops the member with its destination, the beam kept | tested (harness) | `r108_claim39a_a_sticky_firer_beyond_range_drops_the_member_with_its_destination_and_keeps_its_beam`: our *sticky* is processed as *guard* (S-0215 claim 5), whose processing takes no approach step, so the harness calls `approach::step` directly, as S-0215 claim 37's table gives for *sticky* |
| *The firer*, deployed infantry of a human owner in *guard*: S-0517 claim 16 drops the member, no move or turn, the beam stays (SW12 l9) | tested (harness) | `r108_claim39a_a_deployed_infantry_firer_beyond_range_drops_the_member_and_stays_put`, guard arm. The harness forces `Seq::Deployed`; SW12 l9's deploy order is not run |
| Same in *attack*: the beam stays attached | **blocked (Q-2468)** | same test, attack arm: S-0517 claim 16 falls the unit back to rest and S-0514 claim 14 lets the beam go at the fall-back, so our engine releases it. Interim asserted and marked |
| Earlier r91 firer parts (vehicle beyond range approaches, zero speed, SAM ground member) | tested (unchanged) | the r91 tests in `claims_s0513_r89` |
| S-0513 claim 34: a listed jumpjet ascending, descending or hovering below the airborne height is examined and joins | tested | `claims_s0513_warp::r108_claim34_a_listed_jumpjet_ascending_below_the_airborne_height_joins_the_warp` (100 and 207), `..._in_any_state_below_the_airborne_height_joins_the_warp`; a rehearsal of t_L confirms the height and listing the launch reads |
| Claim 34: an airborne object is not examined, listed or not | tested | `r108_claim34_an_airborne_object_is_not_examined_listed_or_not`: a jumpjet at 208 (its own turn lists it out of the ground layer, S-0502 claim 4); a vehicle without jumpjet movement raised to 208, still listed. (A jumpjet ascending over a high bridge with the on-deck state unset was tried and dropped: its own turn in t_L lists it out before the launch, so it never reaches the launch listed.) |
| Claim 34: the height is taken above the deck while the on-deck state is set | tested | `r108_claim34_the_height_of_a_jumpjet_on_a_deck_is_taken_above_the_deck` (a jumpjet standing on a deck, 416 above the terrain, 0 above the deck, joins). Our take-off unsets the on-deck state, so a jumpjet in flight over a deck reads its height above the terrain |
| SW12 b8: an ordered jumpjet ascending at the launch joins and lands in the destination block | tested | `r108_claim34_sw12_b8_an_ordered_jumpjet_ascending_at_the_launch_joins_and_lands` |
| S-0306 r72 claim 17: a warped aircraft that cannot land stays in the ground list in flight but reads its layer from height | tested | `claims_s0306_r62::r72_claim17_an_aircraft_member_whose_type_cannot_land_stays_in_the_ground_list_but_reads_its_height_layer_in_flight` |
| P46 (a): the ground-layer pass considers it in flight; control: a landable one is not in the ground list | tested | `r72_claim17_p46_part_a_the_ground_layer_pass_considers_a_flying_aircraft_that_cannot_land`, `..._control_a_flying_aircraft_that_can_land_is_not_in_the_ground_list` (P46 run 1, ground placement; run 2, the warp, asserts the list membership only, Q-2466) |
| P46 (c) and (d): no seizing and no erasure above height 0 | tested | `r72_claim17_p46_parts_c_and_d_no_seizing_and_no_erasure_above_height_0` |
| Claim 17: the airborne scan and claim 20 (c) read the layer from state and height | tested | `r72_claim17_the_airborne_scan_and_claim_20c_read_the_layer_from_state_and_height` |
| Claim 17: the aircraft pass (pass 0) of a search that looks for aircraft considers it | built (by construction) | pass 0 takes every aircraft; no separate test |
| Claim 17, with `layer_change_on_events_only` off, the sweep moves it out of the ground list for the ground-layer pass | **seam (Q-2467)** | the switch is a client drawing setting; the simulation reads the on position |
| The drawing files a kept-filed aircraft first seen in flight in the ground layer | tested | `app::mapview::layers::tests::r72_claim17_a_kept_filed_aircraft_first_seen_in_flight_is_filed_on_the_ground` |
| S-0309 r113 claim 27 (the whole new text) | tested | the S-0513 rows above; the claim repeats them |
| S-0102 r42 claim 3: an airborne jumpjet is never a member, listed or not; a listed one not airborne is examined | tested | the claim 34 rows above (`..._ascending_below_the_airborne_height_joins_the_warp`, `..._an_airborne_object_is_not_examined_listed_or_not`, `..._sw12_b8_...`) cite it; the listed-and-airborne jumpjet is unreachable, as its Backing gives and the dropped case above found |
| S-0301 r70 claim 9: *A held temporal target within rank 4* | tested | the claim 39a rearming rows above cite it; `combat::check` cites it |
| S-0309 r114 claim 7: the ground-layer pass considers an aircraft that cannot land, in flight | tested | `r72_claim17_p46_part_a_...` cites it; `acquire` pass 1 cites it |
| S-0309 r114 claim 9: the airborne scan does not leave such an aircraft out once tracked and above height 0 | tested | `r72_claim17_the_airborne_scan_and_claim_20c_...` cites it; the scan in `acquire` cites it |
| S-0514 r17 claim 1 and S-0607 r39 claim 3: no erasure and no seizing of such an aircraft above height 0 | tested | `r72_claim17_p46_parts_c_and_d_...` cites both; `temporal` and `lift::eligible` cite them |
| Intake 88's S-0209 r55 tests under S-0309 r113 | tested | `claims_s0209_r55`: 18 passed |

## Interpretation notes (no request)

- **The SAM check tick.** "Its checking turn in the first check tick of S-0309 claim 29 in that
  state" needs a check tick while the member is off the map. With the launch at a tick leaving 0
  by 16 the first check tick after the departure is the appearance tick, so the tests launch at
  a tick leaving 1, where the tick after the departure is a check tick.
- **A jumpjet on a deck.** Claim 34 reads the height above the deck while the on-deck state is
  set. S-0502 claim 4's scenario J3 gives "Repeat from a bridge deck: on-deck state ends at
  first rise", and our jumpjet unsets it at take-off (`jump::leave_ground`), so only a jumpjet
  standing on a deck reads its height above the deck. One in flight over a deck reads its
  height above the terrain. The deck test therefore stands the jumpjet on the deck: 416 above
  the terrain, 0 above the deck, it is examined and joins.

## Quirks

No new switch. Two existing switches are judged again, both verdicts unchanged; see
[c7-play-fixes.md](c7-play-fixes.md), section *Intake 89*.

- `warp_departure_keeps_beam` (default on): both positions tested in the l7a test and the
  existing r89 tests. Rejected, unchanged.
- `layer_change_on_events_only` (client drawing, default on): S-0306 r72 claim 17 gives its off
  position a targeting consequence that the simulation cannot read (Q-2467). Rejected, unchanged;
  condition 3 judged again.

The layer file does not change.

## Checks

On the tree rebased onto origin/main a097d11 (intake 86, S-0501 r103):

- `cargo test --workspace --no-fail-fast`: 7480 passed, 0 failed, 120 ignored;
  `every_spec_quirk_has_a_switch` passes.
- `cargo clippy --workspace --all-targets -- -D warnings`: clean. `cargo fmt --all -- --check`:
  clean.
- `cargo xtask check`: stock-guard reports the 99 known problems and nothing else; canary ok.
- `cargo xtask gate`: ok, wasm32 matches every golden.
- `cargo test --release -p glasnost-sim -- --include-ignored`: only `pathing_c7p4::diag` fails,
  as on main.
- `cargo test --release -p glasnost-ai -- --include-ignored`: 246 passed, 0 failed.
- Goldens, regenerated on the rebased tree: all nine move (`combat`, `duel`, `duel-diagonal`,
  `duel-naval`, `duel-m6c`, `duel-m7a`, `duel-m7b`, `scripted`, `skirmish`), each from its first
  hashed tick with its line count unchanged: `revert_gave_departed` is new hashed state.

## Lanes

Baseline: origin/main **a097d11** (intake 86), built in the same target directory and run on the
same machine. A first baseline at 85b2d32, taken before intake 86 landed, is superseded and not
compared here: intake 86 changes aircraft behaviour.

- **Sweeps, seeds 1 to 16.** M6c, M7a and M7b: every counter's minimum, median and maximum is
  the same before and after; only the run time differs.
- **Arena** (`--record all --threads 8`, not yet re-seeded: intake 90 is not on origin/main).
  Identical output apart from timings. The same 10 pairings fail before and after, none
  changed: LTNK–E1, TELE–APOC, TELE–V3, TELE–ROBO, TELE–BFRT, CAOS–E1, DISK–FV (win test),
  INIT–E2, ROBO–HTK, MIND–E1.
- **AI tests** (`glasnost-ai --include-ignored`): 246 passed, 0 failed.
- **AI quick lane**, seeds 7 to 10, maps 0, 2 and 8, side pairs 0–1, 0–2 and 1–2 (36 games):
  every game's output is identical before and after apart from timing lines (end tick, winner,
  every counter). 15 games launch a Chrono warp, once each, in both lanes.
- **Warp members.** The lanes give little evidence on this intake's rules. The 15 Chrono
  launches show the warp still runs in play and changes no outcome. No lane counts warped
  aircraft or jumpjets, a held temporal target off the map, or a revert that gives a member
  back off the map. Those rules rest on the claim tests above.

## Seams

- **Q-2466, the arrival height of a warped aircraft that cannot land.** S-0306 r72 claim 17 (P46
  run 2) has it at height 0 after the hold. Our placement at the landing point
  (`objects::place` → `air::placed`) puts a non-landable aircraft that is not visiting at cruise
  height. Interim: cruise height kept; the P46 run-2 test asserts the ground-list membership
  only.
- **Q-2467, a drawing switch with a targeting consequence.** S-0306 r72 claim 17 names
  `layer_change_on_events_only`'s off position for the ground-layer pass. The switch is a
  per-client drawing setting; a target search is lockstep state. Interim: the simulation reads
  the on position (the default); the registry note in `rules::quirks` records it.
- **Q-2468, a deployed firer in attack that drops a held member and falls back.** S-0513 r108
  says its beam stays attached; S-0517 claim 16's fall-back to rest and S-0514 claim 14 release
  it. Interim: the fall-back releases the beam in *attack*; in *guard* it stays.
- **SW12 l9 and the *sticky* row are harness-only.** l9's deploy order and *sticky*'s approach
  step are not run through orders: our *sticky* is processed as *guard*.
