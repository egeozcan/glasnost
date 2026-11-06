# Warp members: claim ledger, M8 step A3 intake 94

- **S-0306** r72 to r75 (`git diff 445d0f1 98808ff`, from the r72 that intake 89 built). Claim 17:
  a warp member's appearance at its landing point sets its height at no height above its surface
  whatever its kind, its type's landing flag and its visiting mark, and an aircraft member then
  rests there after its hold; `quirks.layer_change_on_events_only` is a drawing setting. Scenario
  P46 run 2 judges the height from the appearance on. Changes for S-0501 claims 1 and 48 (a),
  S-0102's switch entry and S-0513 claims 36 and 38.
- **S-0513** r108 to r115 (`git diff 80a301e~1 66a1552`). Claims 36 and 38 (the appearance's
  height, not S-0501 claim 1's rule) and claim 39a's *The firer* (Q-2468: the fall-back to rest
  in *attack* lets the beam go, a jumpjet type keeps it, the once-per-tick limit, the deploy
  order's span), with SW12 parts l9, l9a and l9b.
- **S-0309** r114 to r116 (c5d5898): claim 27 restates r115's firer rules.
- **S-0501** r110 to r111 (06278f3): claim 1 (a warp member's appearance does not follow the
  placement rule) and claim 48 (a) (the ground-layer pass reads the on position's list in both
  positions of the drawing switch).
- **S-0102** r42 to r43 (25edec0): `quirks.layer_change_on_events_only` is a drawing setting;
  simulation rules that read an object's list read the on position's membership.

They answer intake 89's Q-2466 (the arrival height of a warped aircraft that cannot land), Q-2467
(a drawing switch with a targeting consequence) and Q-2468 (a deployed firer in *attack* that
drops a held member and falls back). No delta adds or changes a quirk: S-0306's names only
`quirks.layer_change_on_events_only`, now scoped to the drawing (C7 verdict judged again in
[c7-play-fixes.md](c7-play-fixes.md), unchanged).

Requests filed: Q-2526 (a warped aircraft whose type cannot land, which the first placement
marks as visiting, takes S-0501 T2's retreat row at the expiry search's fall-back and so flies off
at its hold's end, where P46 run 2 expects it to rest until the trigger's move order). Q-2527 to
Q-2535 unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Built

- **The appearance's height** (`air::placed`, Q-2466). `warp::place` calls `objects::place`
  while the object is still a member, and `air::placed` reads `warp::is_member`: a member is placed
  on the ground at its landing point's height, whatever its visiting mark and the usable area, its
  speed at 0. On a high-bridge cell reached from the ground that height is the deck, and
  the aircraft's own deck mark is set (S-0501 claim 43); `warp::arrive` reads that mark as well as
  the deck layer. An aircraft whose type cannot land is then on the ground and keeps the ground list
  from there (`AirState::kept_filed`, set by the ground placement as for one the map places), so
  intake 89's `warp::keep_ground_filing`, which set the mark at the hold's end whatever the
  placement gave, is gone. The appearance no longer makes such an aircraft fly at cruise height.
- **The drawing switch** (Q-2467). The simulation never read `quirks.layer_change_on_events_only`
  (it is `app::mapview::quirks::ViewQuirks`, and `mapview::layers` reads `&World`). The ground-layer
  pass (`combat::acquire`) reads `kept_filed`, the simulation's own mark, which is the list as the
  on position keeps it. Intake 89's interim reading is now the answer; citations, the registry note
  in `rules::quirks` and the client's doc say so. A layers test checks that a sweep with the switch
  off changes no hashed state and keeps the mark, in both positions.
- **The deployed firer** (Q-2468). Intake 89 already let the fall-back to rest in *attack* release
  the beam through S-0514 claim 14's common effects, so a jumpjet type (no fall-back there) and a
  second fall-back of the tick (the common effects taken once per actor per tick) keep it. r115
  states that as the answer; the code did not change. New tests run SW12 parts l9a and l9 on the real
  deploy path: the deploy order, *unload*, the completed deploy in *guard* with the beam attached,
  then an attack order and the release at the first *attack* processing that drops the member.

No new hashed field, but the appearance changes what a saved game replays to (the height, speed
and deck mark of an aircraft member from its appearance on, and `kept_filed` set at the appearance
rather than at the hold's end): save format 97 → 98, AI game format 60 → 61. Wire unchanged (12).

## Goldens

Regenerated (`GLASNOST_UPDATE_GOLDEN=1`, sim-gate `--include-ignored`): none moved, so the change
reaches no golden game (it touches only a warped aircraft's appearance). `cargo xtask gate` passes on the
committed goldens.

## Claims

### S-0306 r75

| claim | status |
|---|---|
| 17, a member's appearance at no height above its surface whatever its kind, landing flag and visiting mark; not at cruise height | Tested: `claims_s0306_r62::r75_claim17_an_aircraft_member_appears_at_no_height_whatever_its_landing_flag_and_visiting_mark` (both landing flags × both visiting marks, from the appearance to the hold's end) |
| 17, on the deck when it lands on a high-bridge cell from the ground | Tested: `claims_s0306_r62::r115_claim36_an_aircraft_member_landing_on_a_high_bridge_cell_from_the_ground_appears_on_the_deck` |
| 17, the ground list kept while it flies, until a crash, for a type that cannot land | Tested: `r72_claim17_an_aircraft_member_whose_type_cannot_land_stays_in_the_ground_list_*`, `r72_claim17_p46_part_a_*` (unchanged, the mark now set by the appearance) |
| 17, the rest at that height after its hold until an order, its own behaviour or a seize | Partly: at rest through the hold (`r75_claim17_p46_run_2_*`). Seam (Q-2526): a type that cannot land, visiting by its first placement, takes T2's retreat row at the expiry search's fall-back and takes off at the hold's end |
| 17, the switch is a drawing setting; the ground-layer pass reads the on position's list in both positions | Tested: `mapview::layers::tests::claim4_a_non_landable_aircraft_member_keeps_the_ground_list_while_it_flies` (both positions, state hash unchanged by the sweep) |
| P46 run 2, every part: appearance at height 0 and rest until the trigger's move order | Partly: `p46()` asserts height 0 from the appearance through the hold; the post-hold rest is the Q-2526 seam, so parts c2 and d2 are judged in run 1 only (`r72_claim17_p46_parts_c_and_d_*`) |
| P46 parts a, c, d | Tested (unchanged): `r72_claim17_p46_part_a_*`, `r72_claim17_p46_parts_c_and_d_*` |

### S-0513 r115

| claim | status |
|---|---|
| 36 and 38, the appearance at claim 36's height, not S-0501 claim 1's rule | Tested: as S-0306 claim 17 above |
| 39a, in *attack* a drop with no target found falls a non-jumpjet unit back to rest, which lets the beam go | Tested: `claims_s0513_r89::r108_claim39a_a_deployed_infantry_firer_beyond_range_*` (now asserts the release), `r91_claim39a_a_vehicle_firer_beyond_range_*` (re-cited) |
| 39a, a jumpjet type stays in *attack* with its beam; a second fall-back of the tick keeps it | Tested: `r115_claim39a_a_deployed_jumpjet_firer_and_a_second_fall_back_of_the_tick_keep_the_beam_in_attack` |
| 39a, in *guard* the beam stays attached | Tested: `r115_claim39a_sw12_l9a_and_l9_*` (after the completed deploy) |
| SW12 l9a, the deploy-order span: *unload* current by the second update, no *attack* processing, no target, beam attached, the deploy goes on | Tested: `r115_claim39a_sw12_l9a_and_l9_*` (one synthetic run, not five seeds). Sharpened in intake 99: r122 makes *unload* current and processed in the first update, and the test asserts that ([intake99.md](intake99.md)) |
| SW12 l9, after the deploy: *guard*, no target, beam attached; the attack order, then no target, beam let go, *guard* current or queued, no move or turn | Tested: `r115_claim39a_sw12_l9a_and_l9_*` |
| SW12 l9b, the release at an *attack* processing before *unload* starts | Seam: S-0215 claim 11's "in the middle of firing" readiness is not built, so *unload* is current at the first update. Closed in intake 99: S-0513 r122 drops l9b and sharpens l9a (*unload* current and processed in the first update), tested ([intake99.md](intake99.md)) |
| 39a, row 6 (zombie, paralysed) keeps *unload*; the attack-move refresh loses it | Seam: neither property nor attack-move memory reaches a deploy order in our engine; `unmeasured` in the spec |
| 39a, the firing state ends after both of the first update's per-tick starts | Seam: as l9b. Closed in intake 99: the firing state is S-0107 claim 11's attack state (Q-2566), and a deploy order's change of target ends it as the order takes effect ([intake99.md](intake99.md)) |

### S-0309 r116

| claim | status |
|---|---|
| 27, the beam stays attached in *guard*; in *attack* the fall-back lets it go unless a jumpjet type or a second fall-back of the tick | Tested: as S-0513 claim 39a above (the tests cite r116) |

### S-0501 r111

| claim | status |
|---|---|
| 1, a superweapon warp member's appearance does not follow the placement rule | Tested: as S-0306 claim 17 above |
| 48 (a), the ground-layer pass reads the on position's list in both positions | Tested: as S-0306 claim 17's switch row above |

### S-0102 r43

| claim | status |
|---|---|
| `quirks.layer_change_on_events_only` is a drawing setting; simulation list readers read the on position's membership | Tested: as above; registry note in `rules::quirks`, doc in `app::mapview::quirks` |

## Lanes

Before: main 4fe2e5b; after: this intake on it; same machine, release. Main then moved to 1055021
(intake 95, the arena tool's M-0021 harness and docs only); the branch was rebased onto it and the
checks were run again there.

| lane | before | after |
|---|---|---|
| M6c sweep, seeds 1 to 16 | smallest counter 0 | identical output (minima and medians) |
| M7a sweep, seeds 1 to 16 | smallest counter 0 | identical output (minima and medians) |
| M7b sweep, seeds 1 to 16 | smallest drain minimum 3 | identical output (minima and medians) |
| arena (`--record all --threads 4`) | 13 of 155 pairings fail; M-0021 2 of 24 rows fail (the pre-intake-95 harness) | identical tables and byte-identical recorded files; the same 13 (ROBO–HTK, TELE–APOC, TELE–V3, TELE–ROBO, TELE–BFRT, CAOS–E1, DISK–FV, INIT–E2, MIND–E1, CAOS–APOC, CAOS–TTNK, CAOS–E2, YURI–SHK), none changed |
| `glasnost-ai --include-ignored` (32 binaries) | 248 passed | 248 passed |
| AI quick lane, seeds 7 to 10 × sides 0,1 / 0,2 / 1,2 × maps 0, 2, 8 | 36 of 36 games over with a winner | identical apart from timings |

Warp members: the quick lane's games launch the Chronosphere 10 times (one launch in each of 10
games) and are identical before and after, so no warped aircraft whose type cannot land changed
course there. The arena and the sweeps warp nothing that this intake changes. Aircraft members and
jumpjet members are judged by the synthetic claim tests above, not by any lane.
