# S-0306 r62: claim ledger, M8 step A3 intake 65

This intake takes in one spec revision:

- **S-0306 revision 62** (projectiles): answers **Q-2216** (intake 61: the robot tanks against the
  HTK from side B), **Q-2231** (intake 64: the target-object effects of a detonation on a departed
  superweapon warp member, and claim 20 (c)'s render layer for one) and **Q-2232** (intake 64: the
  deck count after a refused placement).

Our engine built S-0306 r57 (intake 64). The diff from r57 to r62 was read in full (`git diff
bebec5f b764d95 -- spec/combat/S-0306-projectiles.md`): the scope note on claim 17's two design
rules, claim 3 (b)'s switched-off hover case and its consequences, claim 17's deck count, special
hits and render layer, the evidence and confidence of both claims, scenario P11 runs g to j, the new
scenario P45, the Open questions and the Questions for the project lead. No other claim changed.

**No new quirk switch** (no C7 judgement is due; `c7-play-fixes.md` is unchanged). **No new saved
or hashed state**: "a warp member" is the member state the object already carries from its warp's
launch until its arrival hold ends (`sw.warp`), and "phasing" its stage. So save format 92 and AI
game format 54 stay. No import row changes.

## What changed

- `combat::projectile::detonation_inner` no longer shadows the target with `effect_target` (removed):
  the special rows of S-0514 claim 20 act on the projectile's target object whatever its warp
  stage, each by its own claim's tests. The disguise call in `explode` (S-0516 claim 31) gets its
  target unchanged as before; only its comment is new. Electric charging (`linked::charge_hit`)
  acts on buildings only, so a vehicle member gets nothing, as any vehicle does. The selected row
  still replaces ordinary damage (unchanged).
- `lift::seize_hit`: after the firer's old grip ends (S-0607 claim 2, unchanged order), a target
  that is a warp member from its departure until its arrival hold ends (`warp::phasing`: stages
  `Blocked`, `Away`, `Placed`, `Hold`) is left alone. S-0607's own eligibility already refused a
  member off the map; the new rule matters for a member phasing in, which our engine seized before.
- `superweapons::warp::in_ground_layer` (new): a warp member is in the ground render layer while
  its member state exists, from launch until the hold ends. It is read in two places only:
  - claim 20 (c)'s threshold (`detonation_point`): 42 for a warp member, also one that departed
    from a deck and counts as airborne;
  - S-0514 claim 1's ground-layer test in `temporal::deliver`.

  `temporal::ground_layer` itself is unchanged, since S-0607 claim 3's seize test and the other
  callers do not read claim 17's rule.
- `superweapons::warp::passed_for_deck` and `combat::projectile::target_airborne`: the Q-2232
  interim is the answer (claim 17's engine-design rule, no switch). Only their docs and comments
  change.
- Claim 3 (b) and T1's flak row: already built (intake 61: `led` uses S-0503's moving test, `lead`
  scales by the speed estimate of the held fraction, and the flak row has no firer exception). New
  tests only.

## Claims

| Item | Status | Tests / notes |
|---|---|---|
| Scope note: claim 17's deck count after a refusal and no seizing of a member are design | built here | the two rules below carry "engine design, no switch" in their comments |
| Claim 3 (b): a switched-off hover keeps its heading point (S-0503 claims 16 and 19) and is led while it has a destination or a heading point | tested | `claims_s0306::claim3b_switched_off_hover_lead` (intake 61), `claim3b_a_switched_off_hover_that_arrives_is_no_longer_led` (led until it arrives) |
| Claim 3 (b): the lead is the amount its held speed fraction gives, none when it is 0 | tested | `claims_s0306::claim3b_a_switched_off_hover_holding_rho_zero_is_not_moved` |
| Claim 3 (b): one that arrives at its heading point with no destination and no stored step loses the point and is no longer led (S-0503 claim 15) | tested | `claims_s0306::claim3b_a_switched_off_hover_that_arrives_is_no_longer_led` (stored path cleared, run through whole ticks until it stops moving: no heading point, the aim is the reference point) |
| Claim 3 (b) consequences: the reached row for a led shot; T1's first row for a shot with a firer | already built | T1 and T2 unchanged; `claims_s0306::p44b_owned_projectile_falls_to_the_deep_fall_row` (intake 61) |
| Claim 3 (b) and T1: a flak-scattering shot with a target and a firer detonates in its first turn below the terrain, at the surface (P45 part b) | tested | `claims_s0306::p45b_owned_flak_projectile_detonates_in_its_first_turn_below_the_surface`; `t1_flak_row` (intake 61) |
| P45 part a (ROBO–HTK from side B, M-0003, seed 2) | local lane | the arena lane below, both sides over seeds 0–63 |
| Claim 17: the deck count while a passed deck check awaits placement; after a refused placement it counts by height until a later deck pass (engine design, Q-2232) | tested | `claims_s0306_r62::claim17_p11_run_i_a_refused_placement_ends_the_deck_count` (refusal forced by a vehicle standing on the deck of the landing cell), `…_parts_a_b_a_failed_check_and_its_redirect_count_by_height` (failure forced by a building on that deck), `…_part_c_a_pass_for_a_cell_not_on_a_deck_counts_by_height` (landing point moved to a ground cell before the check), `…_part_d_a_later_deck_pass_stands_it_on_a_deck_again` (blocker gone: the check passes for the deck cell, then the placement on the deck); each reads `target_airborne` and an anti-air shot's first turn. In our engine a failed check always redirects (S-0513 claim 37 and T2), so run i's parts a and b are one update |
| Claim 17 with `quirks.warp_deck_departure_airborne` off: it stands on its departure deck through any refusal | tested | `claims_s0306_r62::quirk_warp_deck_departure_airborne_off_stands_on_its_departure_deck_through_a_refusal` |
| Claim 17: takeover, bomb planting, bomb disarming, air-strike designation, disguise-making and the erasure lock act on a departed member off the map, by their own tests; its health is unchanged (P11 run g) | tested | `claims_s0306_r62::claim17_p11_run_g_special_hits_act_on_a_member_off_the_map`; controls with no warp in `claim17_p11_controls_each_part_acts_on_a_member_with_no_warp` |
| Claim 17: the same on a member phasing in, appeared and in its arrival hold (P11 run h) | tested | `claims_s0306_r62::claim17_p11_run_h_special_hits_act_on_a_member_phasing_in` (stages `Placed` and `Hold`) |
| Claim 17: electric charging does nothing to a member (buildings only) | tested | part c of the three tests above: no effect, health unchanged |
| Claim 17: no locomotor seizing from departure until the arrival hold ends (engine design, no switch); the row still replaces damage; V appears and has its arrival effects on the same ticks (P11 run h part h, both timings) | tested | `claims_s0306_r62::claim17_p11_run_h_part_h_no_seizing_from_departure_until_the_hold_ends` (`Away`, `Placed`, `Hold`; the next 12 warp updates compared with an unhit member); `claim17_a_seizing_hit_after_the_hold_ends_seizes` (control) |
| Claim 17: in every case the selected row still replaces ordinary damage | tested | the health checks of runs g and h; `special_of` and the area gate unchanged |
| Claim 17: a warp member is in the ground render layer from launch until at least the end of its hold, so claim 20 (c) uses 42 | tested | `claims_s0306_r62::claim17_a_warp_member_is_in_the_ground_layer_for_claim_20c` (an aircraft 600 above the ground, taken out of its cell list so that T1's proximity row does not find it first: a passed-by detonation 80 past it is moved onto it with no warp, threshold 128, and stays at its own position as a member, threshold 42), `claim17_p11_run_j_a_deck_departed_member_uses_threshold_42` (P11 run j: passed-by row 80 leptons past the old reference point, not moved onto it) |
| Claim 17: S-0514 claim 1's ground-layer test passes for a warp member | tested | `claims_s0306_r62::claim17_a_warp_member_is_in_the_ground_layer_for_the_erasure_test` (an aircraft above the ground: no lock with no warp, a lock as a member) |
| Open question: what the erasure lock does to the warp of a member off the map | interim (no request) | Our engine: the lock attaches; S-0513 claim 51 makes the warp update the member's whole turn until the hold, so the erasure's updates (S-0514 claim 7) wait and the member appears and has its arrival effects on its usual ticks; the erasure goes on from the hold. `claims_s0306_r62::claim17_an_erasure_lock_on_a_member_off_the_map_waits_for_the_hold`. The spec records the original by run g part d; nothing here is our choice beyond S-0513 claim 51 |
| Open question: what the direct-rocking and nuke-making rows do to a departed member | not built (no request) | No choice of ours: direct rocking is a selection only in our engine (its effect is a seam of S-0514 claim 20), selected for a vehicle target whatever its stage; nuke-making bursts at the detonation point (S-0513 claims 8 and 9), the old position while the member is off the map. The question is already the spec's (Q-2231's Open questions entry) |
| Open question: when a warp member leaves the ground render layer after its hold | not needed | no rule reads it; `in_ground_layer` ends with the member state |
| Q-2216: ROBO–HTK from side B | answered, as built | r62 states claim 3 (b)'s switched-off hover case and T1's flak row, which our engine already did; the cause in the stock games is left to P45 part a, the lead's record. The pairing still fails with the same counts (see Lanes); no new request |

## Requests

None filed. Q-2241 to Q-2245 are unused.

## Intake 66 (run in parallel)

Intake 66 builds S-0213 r89. No claim here reads human orders, and S-0213 is not touched. No seam.

## Seams

None new. The deck tests set a cell's deck flags directly, as intake 64's do, and put a blocker in
a deck cell's list directly. Run i's refusal is forced with a vehicle standing on the deck of the
landing cell after the check passed (S-0513 claim 50: no vehicle shares a layer with another), its
failed check with a building there (claim 37). The r62 tests run with gravity 0, so that a
straight shot flies level (run j).

## Lanes

Before: main 5c72f70 (its source exported to the scratchpad); after: this branch. Outputs under
`local-lane/m8/intake65/before` and `after` (not tracked).

| Lane | Before | After |
|---|---|---|
| M6c sweep, seeds 1–16 | smallest minimum 2 | 2, output identical |
| M7a sweep, seeds 1–16 | smallest minimum 0 | 0, output identical |
| M7b sweep, seeds 1–16 | smallest drain minimum 3 | 3, output identical |
| Arena (`--record all`) | 11 of 155 fail (M-0001 9, M-0002 1, M-0003 1) | 11 of 155, the three CSVs byte-identical |
| ROBO–HTK (Q-2216), seeds 0–63 | side A col 59 (none 5), side B col 23 (none 41) | identical on both sides; still FAIL (side outcome) |
| `glasnost-ai --include-ignored` | 239 passed, 0 failed | 239 passed, 0 failed |
| AI quick lane, seeds 7–10, maps 0, 2, 8 | 36 games, all over | 36 games, all over; identical lengths and winners |

Nothing moved: no lane game launches a warp superweapon while a special shot or a seizing shot is
aimed at one of its members, and ROBO–HTK's rules (claim 3 (b), T1's flak row) were already built.
The worktree's `target` directory was removed from outside during the after lanes, so the last
three quick games (seed 10, map 8) could not start; the binary was rebuilt from the same tree and
those three games were run again.

## Checks

Rebased onto 26c191b (main with intake 66) without conflicts. On the rebased tree:

- `cargo test --release --workspace --no-fail-fast`: 7190 passed, 0 failed, 118 ignored.
- `cargo test --release -p glasnost-sim-gate -- --include-ignored`: the goldens hold.
- `cargo test --release -p glasnost-ai -- --include-ignored`: 239 passed, 0 failed.
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`: clean.
- `cargo xtask check`: deps, determinism, cite and canary ok; stock-guard 105 problems, all in
  `measurements/`, as on main.
- `cargo xtask gate`: ok, native release and wasm32 under node.
