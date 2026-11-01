# S-0306 r57: claim ledger, M8 step A3 intake 64

This intake takes in one spec revision:

- **S-0306 revision 57** (projectiles): answers **Q-2212** (intake 60: where a projectile aimed at
  a departed superweapon warp member flies, whether it can hit, and its target after a
  landing-check kill).

Our engine built S-0306 r53 (intake 61) and r54's claim 17 sentence (intake 60, through S-0513
claim 39). The diff from r54 to r57 was read in full (`git diff 1bf986f bebec5f --
spec/combat/S-0306-projectiles.md`): claim 17's warp paragraph, its evidence and confidence, scenario
P11 run f, the new quirk, a new Open question, and two Changes for other specs (S-0307 Terms,
S-0513 claim 39). No other claim changed.

**One new quirk switch**, `warp_deck_departure_airborne`, default on (judged for C7 in
[c7-play-fixes.md](c7-play-fixes.md)). **No new saved or hashed state**: "departed" is the warp
member's stage (`Stage::Blocked` or `Stage::Away`), "a landing check passed for a deck cell" is
`Stage::Away` with a landing cell whose layer is the deck, and the switch is a rules field (in the
rules hash, not the state hash). So save format 91 and AI game format 53 stay. No import row changes.

## What changed

Intake 60's interim for Q-2212 was that a projectile's own update drops its off-map target object,
as for any off-map object. Revision 57 replaces it: the projectile keeps a departed member until it
appears or leaves the game.

- `combat::projectile::turn`, the turn-start drop of an off-map target object, skips a departed
  member (`superweapons::warp::departed`). Every rule then reads the member's kept position, which
  the departure (`warp::lift_off`) leaves in place: the steering, T1's steering-distance row and
  claim 20's detonation point.
- `combat::projectile::target_airborne` answers claim 17's airborne question for T1 (the anti-air
  row), claim 20 (c) and the loss at a removal or kill. Any object that is not a departed member
  keeps S-0307's airborne (`airborne_by_height`). A departed member is not airborne while
  `warp::passed_for_deck` holds; otherwise it counts by its height above the terrain beneath its
  kept position (switch on) or above the deck it departed from (switch off), against S-0307's
  208.
- `combat::death::mark_killed` and `projectile::target_leaves` use it, and `target_leaves` acts for
  a departed member too, so a landing-check kill (S-0513 claim 37) or a removal gives the cell the
  member stood in at departure, or nothing when it counts as airborne.
- `teleport::targeters_drop`'s doc names the answer.
- The detonation's area damage and T1's proximity row find objects through the cell lists and the
  map flag, so they miss a member off the map with no change (claim 17). After it appears it is in
  its landing cell; S-0302 claim 8's leaving-stage protection (`being_erased`) covers only the
  leaving stage, so the phase-in does not stop a hit, as before.

T2 asks no airborne or floor-bound question of the target (its rows and T1's read only the
anti-air row's "airborne target"; the ground row reads the projectile's own height), and no other
projectile flight path in the simulation crate classifies its target object: the remaining
`airborne_by_height` and `floor_bound` callers are the firing check, range, acquisition and
locomotion, which claim 17 does not name. No change there. After the shadowing
`effect_target` in `detonation_inner`, `target` is read only by the special arms; the area
damage, animation and debris do not read it.

## Claims

| Item | Status | Tests / notes |
|---|---|---|
| Claim 17: a projectile keeps a departed member as its target object | tested | `claims_s0306_r57::claim17_p11_run_f_…`, `claim17_a_steering_projectile_detonates_at_the_old_position_…`; `claims_s0513_r43::claim39_a_projectile_in_flight_keeps_the_member` now also runs a projectile turn after departure |
| Claim 17: every rule reads the departure position until it appears; a steering projectile steers to it, T1's steering-distance row and claim 20 place a detonation there | tested | `claim17_a_steering_projectile_detonates_at_the_old_position_and_area_damage_misses_the_member` (the detonation point is the old position) |
| Claim 17: from its first turn after the member appears, a steering projectile turns towards the landing point and hits the member there (P11 run f) | tested | `claim17_p11_run_f_the_projectile_keeps_the_member_and_hits_it_at_its_landing_point`, through whole ticks with a real warp: departure in t_L + 62, appearance in t_L + 63, the shot closes on the landing point and the member loses health |
| Claim 17: a straight or arcing projectile keeps its stored target point K | tested | `claim17_a_straight_projectile_keeps_its_stored_target_point` (K and the target object unchanged through departure and placement); an arcing shot: `claims_s0513_r43::claim39_a_projectile_in_flight_keeps_the_member` (target kept through its own turn after departure) |
| Claim 17: while off the map, neither T1's proximity row nor the area damage finds the member | tested | the old-position test: the member's health is unchanged, a vehicle now in the cell it left is damaged. No code change (the cell lists) |
| Claim 17: after it appears it is found in its landing cell, and its phasing in does not stop a hit | tested | `claim17_a_hit_while_the_member_phases_in_after_it_appears` (stage Placed, `warp::phasing` true before and after the hit) |
| Claim 17: a landing-check kill gives the departure cell, or nothing for a member counted as airborne | tested | `claim17_a_member_killed_by_its_landing_check_leaves_its_departure_cell` (at departure, and at a later retry after the projectile kept the member through its own turn); the deck cases below |
| Claim 17: the airborne count of a departed member (not S-0307's off-map rule; height above the surface beneath its kept position; on a deck once a check passed for a deck cell) | tested | `claim17_a_member_departed_from_the_ground_is_not_airborne`, `claim17_a_passed_landing_check_for_a_deck_cell_stands_the_member_on_a_deck`, and the switch tests below. A placement refused after a passed check (S-0513 claim 50): **Q-2232**, answered by S-0306 r62 (intake 65: the interim is claim 17's engine-design rule, [intake65-S-0306.md](intake65-S-0306.md)) |
| Claim 17 (claim 15): a steering projectile left without target by a deck member's kill climbs | tested | `quirk_warp_deck_departure_airborne_on_a_deck_member_killed_leaves_no_target` |
| A departed member removed from the game | tested | `claim17_a_departed_member_removed_from_the_game_leaves_its_departure_cell` |
| `quirks.warp_deck_departure_airborne` on (default) | tested | `quirk_warp_deck_departure_airborne_on_a_deck_member_killed_leaves_no_target`, `quirk_warp_deck_departure_airborne_on_the_anti_air_row_finds_the_member` |
| `quirks.warp_deck_departure_airborne` off | tested | `quirk_warp_deck_departure_airborne_off_a_deck_member_killed_leaves_its_deck_cell`, `quirk_warp_deck_departure_airborne_off_the_anti_air_row_does_not`. Rules: `every_spec_quirk_has_a_switch` (green); schema regenerated |
| Open question: the target-object effects of a detonation (S-0514 claim 20's special hits, S-0516 claim 31's disguise) on a departed member, off the map and phasing in | answered (intake 65) | **Q-2231**, answered by S-0306 r62. Intake 64's interim (`combat::projectile::effect_target`: none acted on the member while it was off the map) is replaced: each row acts by its own tests, no locomotor seizing from departure until the hold ends ([intake65-S-0306.md](intake65-S-0306.md)) |
| Open question: claim 20 (c)'s threshold for a deck-departed member counted as airborne (its render layer) | answered (intake 65) | **Q-2231**, answered by S-0306 r62: a warp member is in the ground render layer until at least the end of its hold, so 42 (`warp::in_ground_layer`; intake 64's interim gave an aircraft member above the ground 128) |
| Changes for other specs: S-0307 Terms (the exception for a departed member) | built here | `target_airborne` is the exception, used only by S-0306's rules; S-0307 itself has not been revised |
| Changes for other specs: S-0513 claim 39's open question settled here | built here | Intake 60's Q-2212 rows now point here ([intake60-S-0513.md](intake60-S-0513.md)) |

## Requests

- **Q-2231**: what a projectile's target-object effects do to a departed warp member (off the map,
  and phasing in), and which render layer a deck-departed member is in for claim 20 (c). Answered
  by S-0306 r62 (intake 65).
- **Q-2232**: whether a departed member still stands on a deck after its placement on a deck
  is refused (S-0513 claim 50), and after a later check for another cell. Interim: only while a
  passed check for a deck cell waits for its placement (`Stage::Away`); a refusal takes it back to
  the count by height. Answered by S-0306 r62 (intake 65): the interim is the engine-design rule.

## Intake 63 (run in parallel)

Intake 63 builds S-0213 r87 (human orders) and S-0301 r63 (the firing cycle). No claim here
depends on them: the S-0301 r63 diff names no warp member, projectile flight or S-0306 rule, and
this intake touches only the projectile's turn, its detonation's target-object effects, the kill's
projectile loss and the warp member's stage queries. No seam.

## Seams

None new. The deck tests set a cell's deck flags directly (no bridge object), as
`claims_s0208_teleport`'s bridge world does.

## Lanes

Before: main 99fe542 (its source exported to the scratchpad); after: this branch. Outputs under
`local-lane/m8/intake64/before` and `after` (not tracked).

| Lane | Before | After |
|---|---|---|
| M6c sweep, seeds 1–16 | smallest minimum 2 | 2, output identical |
| M7a sweep, seeds 1–16 | smallest minimum 0 | 0, output identical |
| M7b sweep, seeds 1–16 | smallest drain minimum 3 | 3, output identical |
| Arena (`--record all`) | 11 of 155 fail (M-0001 9, M-0002 1, M-0003 1) | 11 of 155, the three CSVs byte-identical |
| ROBO–HTK (Q-2216) | FAIL (side outcome) | unchanged |
| `glasnost-ai --include-ignored` | all pass | 239 passed, 0 failed |
| AI quick lane, seeds 7–10, maps 0, 2, 8 | 36 games, all over | identical lengths and winners |

Nothing moved: no lane game launches a warp superweapon while a projectile is aimed at one of its
members.
