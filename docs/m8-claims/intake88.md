# S-0209 r55: claim ledger, M8 step A3 intake 88

S-0209 r45 to r55 (commit d5152ab) answers **Q-2396**: an armed walker blocked by an enemy
building, a closed enemy gate or another enemy object that its weapon cannot fire at. Claim 4
gains *Armed walkers and the firing check*: rows WP13, WP24 and WP28 ask no firing check, so
such an occupant gives *enemy obstacle*, the path search gives the step the *enemy obstacle*
cost, and a path can run through it. Claim 9 gains *The take is an override* (to *attack* with
the target and no destination, S-0215 claim 10), *After the take* (the drop of S-0309 claim 29,
then a rest in front of the obstacle without attack-move memory, or the engaged mark of S-0309
claim 23 (b) deciding with it) and the *Charges case*. A new switch,
`quirks.walker_attacks_unhittable_obstacle`, and scenario C15 parts e1 to e3 and e5 to e7.
Nothing else in the diff changes `quirks.chop_rounding` or any other switch (it appears only in
unchanged context lines). No request filed.

## Built

- **The rows and the switch** (claim 4, Edge cases). `occupancy::enemy_obstacle` gives the
  value of rows WP13, WP24 and WP28. With the switch on (default, the original) it is
  *enemy obstacle* with no further test, so the default reads no firing check and is the same
  code path as before for WP24 and WP28. With it off it asks
  `combat::weapon::would_use` for the occupant and `combat::check::check(.., false)` (no range
  test) for that slot, and gives *blocked* on *illegal*. The path search reads the same entry
  outcome, so the off position also takes the step out of the search (S-0212 claim 8).
- **Row WP13 reads the primary weapon** (claim 4, r55 text: "a primary weapon for WP13, *armed*
  in Terms for WP24 and WP28"). The closed-enemy-gate branch tested *armed*. It now tests the
  mover's primary weapon (`Mover::primary`). A walker whose primary weapon has no damage now gets
  *enemy obstacle* at a closed enemy gate (it is not armed, so WP24 and WP28 still give it
  *blocked*); a walker with only a secondary weapon now gets *blocked* there (row WP14).
- **The take is an override** (claim 9). The *friendly obstacle* / *enemy obstacle* arm of the
  repeated ask in `walk::ask` clears the path as before and, when there is a target (the first
  listed object not allied, or the wall's cell), calls the new
  `acquire::override_attack_on(w, id, target)`: S-0215 claim 10's override to *attack* (the queued
  behaviour, else the current one, suspended; the target and destination put aside; the
  destination cleared for a mobile actor; the let-go of S-0301 claim 5), the body of the old
  `override_attack`, which now calls it with an object target. When the first listed object is
  allied, nothing is taken and nothing is overridden: the move stays current with its
  destination. The let-go is no longer called in that case (the target does not change).
- **After the take.** No new code: S-0309 claim 29 (`held::periodic_check`) drops the target on
  *illegal* at a check tick, the drop does not revert, the *attack* processing then finds no
  target and falls back (S-0215 claims 13 and 14, with `refresh_attack_move` for the attack-move
  memory, S-0309 claim 23 (b)), and a killing hit on the target reverts the override
  (`death::mark_killed` → `held::holders_drop`). The tests below show the shapes the claim gives.
- **S-0215 claim 23, first infantry row** (needed by the charges case; it was not built). In
  `process::process_attack`, before every other infantry rule: an infantry unit with explosive
  charges (type flag or the veterancy ability) whose target is a building that accepts charges
  (`can_c4`) and is not invisible takes the target as its destination and queues *sabotage*,
  delay 1, no draw, none of the general rules (`process::charges_building`). The destination
  assignment runs the destination-driven get-up (S-0107 claim 15). This row reads any *attack*
  of such a unit on such a building, not only the take of claim 9.
- **Formats.** No serialized field is added, so save format 95 and AI game format 58 stay.
  Behaviour changes in the default (the override, the WP13 test, the charges row), so the goldens
  move (see Checks).

## Claims

| Claim part | Status | Where, and why |
|---|---|---|
| Claim 4: rows WP24, WP13 and WP28 give *enemy obstacle* to an armed walker whose firing check answers *illegal* against the occupant | tested | `claims_s0209_r55::the_three_rows_give_enemy_obstacle_to_a_walker_that_cannot_fire_at_the_occupant` (a building, a closed gate and a vehicle, warhead versus 0 against every armour) |
| Claim 4: WP13 tests the primary weapon, WP24 and WP28 *armed* | tested | `row_wp13_reads_the_primary_weapon_and_rows_wp24_and_wp28_read_armed` (a zero-damage primary; a secondary only) |
| Claim 4: the path search gives the step the *enemy obstacle* cost, so a path can run through the occupant when every detour costs more | tested | `a_long_detour_is_taken_only_with_the_switch_off` (switch on: the walker takes B although a way round 32 straight steps longer exists) |
| Claim 9: the take is an override to *attack* with no destination; the move suspended, the order's destination put aside, the path discarded | tested | `assert_take` in every C15 test; `claims_s0209::walker_targets_first_listed_enemy` (now also checks the override); `walker_spares_allied_first_listed` (no take, no override) |
| Claim 9 / C15 e1: no shot, the loss at the first multiple of 16 after the take, the rest in front of B, no take again in 600 ticks, never in B's cell; the suspended move stays suspended | tested | `e1_a_walker_takes_a_building_it_cannot_fire_at_loses_it_and_rests` |
| C15 e6 (closed enemy gate, WP13) and e7 (enemy vehicle, WP28): as e1 | tested | `e6_the_same_rest_in_front_of_a_closed_enemy_gate`, `e7_the_same_rest_in_front_of_an_enemy_vehicle` |
| C15 e2: with attack-move memory, each loss at a check tick, never past X, once at rest it stays at rest, the memory removed | tested | `e2_an_attack_moving_walker_takes_and_loses_the_building_until_it_rests` (3000 ticks; in our synthetic run no targeting-timer expiry comes while the walker holds B, so it rests after the first loss) |
| Claim 9: a walker engaged at the fall-back resumes its attack move, ending the mark, and takes the occupant again | tested (harness) | `e2_an_engaged_walker_resumes_its_attack_move_and_takes_the_building_again` (the harness sets the engaged mark after the take, as for a walker engaged before it; the spec leaves this `unmeasured`) |
| Claim 9: the expiry rows of S-0309 claim 23 (a) while the walker holds the occupant, with it out of reach, and the re-engagement by a search finding another target | built, not tested here | the existing S-0309 claim 23 code; the spec gives no set-up (`unmeasured`) |
| C15 e5: the destruction of B reverts the override, *move* with the order's destination again, the walker leaves the cell in front of B | tested | `e5_the_destruction_of_the_building_reverts_the_override` (our harness runs the killing hit's death path directly in the tick after the take, in place of the attacker K) |
| C15 e3: a short way round is taken in both positions | tested (shape differs) | `e3_a_short_detour_is_taken_in_both_positions`. Our second gap is the cell next to X (2 straight steps longer). A farther gap (4 steps) made the walker take B with the switch on: our walker starts inside the usable area, so the corridor of S-0212 claim 14 applies, which C15 e3 removes by starting the walker outside it. Seam of the test, not of the claim. |
| An occupant the walker can fire at: the same take, and it fires, in both positions | tested | `an_occupant_the_walker_can_fire_at_gives_enemy_obstacle_in_both_positions`, `a_walker_that_can_fire_at_the_building_takes_it_and_fires_in_both_positions` |
| Charges case: *sabotage* current at the next check tick keeps B as target and destination; otherwise the check drops B | tested (disjunction) | `charges_case_the_take_leads_to_sabotage_or_to_the_drop` (our run gives *sabotage*); S-0215 claim 23's row: `claim_23_a_charges_walker_in_attack_on_a_building_queues_sabotage`, `claim_23_the_charges_row_needs_charges_and_a_building_that_accepts_them` (no charges; a building that does not accept charges; an invisible building) |
| Edge cases, switch off: the three rows give *blocked* on *illegal*; no take, the walker routes round or reacts to *blocked*; a walker with charges never reaches *sabotage* this way | tested | `switch_off_the_three_rows_give_blocked_for_an_occupant_the_walker_cannot_fire_at`, `switch_off_a_walker_with_no_way_round_never_takes_the_occupant` (B, G and V; move and attack move), `a_long_detour_is_taken_only_with_the_switch_off`, `switch_off_a_charges_walker_never_takes_the_building` |
| S-0215 claim 10's list of users (S-0209 Changes) | built | `acquire::override_attack_on` is the shared override |

## Interpretation notes (no request)

- **The off position's check is the whole firing check without the range test.** The Edge case
  says "the firing check without the range test ... answers *illegal*". `check` answers by rank,
  so a walker that is rearming, out of ammunition or cloaked gets that answer ahead of a versus-0
  *illegal* and so *enemy obstacle* in that tick. We follow the text literally. It matters only
  with the switch off, and only for a walker that fires at something else or is cloaked while it
  walks.
- **The charges case after the check** (S-0215 claims 10, 11 and 28 own it). In our run the
  walker with charges goes to *sabotage* in the tick after the take. On its way into B its next
  repeated ask meets B again, which is a second take: an override that suspends the queued
  *sabotage* in place of the move, which is lost. It enters B, B is destroyed, and the walker
  rests beside B's cell with *sabotage* still suspended. That follows S-0215 claim 10 (an
  override replaces the suspended behaviour) and is not claimed by S-0209.

## Quirks

`walker_attacks_unhittable_obstacle` (new; default on, the original). Both positions tested
(above). C7 verdict: **admitted** to the *Pathing and other fixes* layer (off); see
[c7-play-fixes.md](c7-play-fixes.md), section *Intake 88*. The layer file changes, so the layered
rules hash moves. `quirks.chop_rounding` does not change.

## Checks

- `cargo test --workspace --no-fail-fast`: 7448 passed, 0 failed, 120 ignored.
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check`: clean.
- `cargo xtask check`: deps, determinism, cite and canary ok; stock-guard 99 problems (the
  known ones in measurement files; none in this change).
- `cargo xtask gate`: ok (the wasm32 runs match the regenerated goldens).
- `glasnost-sim --include-ignored` (release): 5228 passed, 1 failed: `pathing_c7p4::diag` only
  (the brief's known failure).
- Goldens regenerated (`GLASNOST_UPDATE_GOLDEN=1`): `duel-diagonal`, `duel-m6c`, `duel-m7a`,
  `duel-m7b` and `duel-naval` move; `combat`, `duel`, `scripted` and `skirmish` do not.

## Lanes

Main (f8553f5) against the branch, base binaries built from f8553f5 before any change. Outputs
under `local-lane/m8/i88/` (git-ignored).

| Lane | Before | After |
|---|---|---|
| M6c sweep, seeds 1 to 16 | smallest minimum 0; seed 5 ends at 8 447 | smallest minimum 0; the per-seed smallest counters the same; seed 5 ends at 8 353; medians move a little |
| M7a sweep, seeds 1 to 16 | smallest minimum 0 | smallest minimum 0; seed 9 now ends early at 7 357 (house 0 wins), so the psychic reveal minimum goes 6 → 2 and slaves at work 7 333 → 2 780 |
| M7b sweep, seeds 1 to 16 | smallest drain minimum 3 | smallest drain minimum 3 |
| Arena, `arena --record all --threads 4` | 10 of 155 pairings fail (DISK–FV on the win test; MIND–E1, CAOS–E1, TELE–V3, TELE–BFRT, LTNK–E1, TELE–APOC, TELE–ROBO, INIT–E2, ROBO–HTK on a side outcome) | the same 10 fail; every other row identical. Only MIND–E1 changes: ours 10/1/1 → 11/0/1 (p 0.14 → 0.31), side B now matches |
| `glasnost-ai --include-ignored` (release) | not rerun (f8553f5's lane) | all pass, 0 failed |
| AI quick lane, seeds 7 to 10, maps 0, 2 and 8, sides 0–1, 0–2, 1–2 (36 games) | every game ends, at 19 974 to 82 013 ticks | every game ends, at 19 974 to 89 077 ticks; most streams change |
| Seed 10 map 2, dumps every 400 ticks, all three pairs | no walker held in front of an enemy building it cannot hurt (the stuck walker of Q-2396 does not reproduce at f8553f5) | none either: every infantry hold of three or more dumps on an object target hurts it (a brute on a construction yard, conscripts on a power plant, a flak trooper on a captured outpost, GIs on a self-repairing Apocalypse tank) or is a slave miner at work |
| Play-fixes layer (`GLASNOST_PLAY_FIXES=1`), seed 10, maps 0, 2 and 8, three pairs | | all 9 games end, at 22 806 to 59 897 ticks, with no slower ticks than the default (686 to 2 190 µs per tick) |

## Seams

- **A diagonal step past a building's corner.** In the e3 shape, the walker's diagonal step from
  the cell in front of X into the second gap below X puts its position, and its listing, in X's
  cell for about 18 ticks without an entry ask (walking geometry of S-0208 between infantry
  positions). No take follows, as no ask is made. Not changed here; noted for the lead.
- **A mind-control infantry unit holding an enemy building it cannot hurt** (baseline quick lane,
  seed 10, map 2, sides 0 and 2, dumps every 400 ticks): object 8366 holds a power plant 3 to 4
  cells away in *attack* from tick 42 800 to 44 800 or later, with the plant's health unchanged.
  It is not next to the building, so it is not claim 9's case. It may be S-0309 claim 29 not
  answering *illegal* for that weapon. Not traced here. The branch changes that game's stream and
  the hold does not appear in it.
