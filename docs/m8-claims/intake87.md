# S-0202 r62 and S-0216 r67: claim ledger, M8 step A3 intake 87

S-0202 r54 to r62 (commit 6638955) and S-0216 r59 to r67 (commit cc2a7f5) answer **Q-2336**
(our arena's first creation draw at another place than M-0019's *N*₀). S-0202 claim 14 gains
*House creation*: each house a game start creates takes one ranged draw R(450, 1800), its result
unused, and a skirmish or network start creates S-0009 claim 19's house list twice, so there are
two rounds of one draw per house, the first before the early phase and the second after it and
before the initial fill. S-0216 claim 8 gains *Game start* (the records' houses: a human player
house at start 3, the two computer houses, the neutral and the special house; the bases setting
on, the unit-count setting 0, crates off) and *Removal before the groups* (every vehicle the
human player house does not own taken off the map with no draw, no animation and the cells
freed; the human's base unit stays on its base cell all game, and no object targets or damages
it). Request filed: **Q-2446** (the human player house's country).

## Built

- **House-creation rounds** (S-0202 claim 14). `load::house_creation_round` takes one
  `rng.ranged(450, 1800, quirks.ranged_draw_half_span)` per house of `World::houses`, in house
  order, its result discarded. `load::new_game` calls it right after the `World` is built,
  before `shroud::start` and `triggers::begin` (round 1, so before the early phase and anything
  else that could draw), and again after the early-phase block and before the initial fill
  (round 2), outside the `quirks.double_start_resolution` gate. With that switch off there is no
  early phase, so the two rounds come together before the fill. A sandbox start (our viewer's)
  takes neither round: claim 14 names skirmish and network starts and a campaign mission, and the
  sandbox is none of them (design, documented in `docs/development.md`).
- **Our arena's game start** (S-0216 claim 8, *Game start*). `oracle::setup` now builds the
  players in claim 19's order: the human player house `oracle::HUMAN` (house 0, start
  `oracle::HUMAN_START` = 3, `human` set, no AI flag, a player and not an observer), then side A
  and side B (`oracle::SIDE_HOUSES`, houses 1 and 2, as before computer houses with no brain);
  `setup::skirmish` appends the neutral and special houses. Crates off, the unit-count setting 0
  and the bases setting on, so `starting::give` gives each of the three player houses its base
  unit (its creation draw and number, put on its base cell with no draw), no budget unit, and the
  final R(0, 65535). The human house takes side A's country, Q-2446's interim
  (`oracle::HUMAN_COUNTRY_SIDE`).
- **Removal before the groups** (S-0216 claim 8). `oracle::take_off_before_the_groups` runs after
  `new_game` (so after resource-glint setup) and before the first group's creation: every
  vehicle the human house does not own leaves the game through the new
  `objects::take_out_quietly`, which is `objects::remove` without the ground-cell recalculation,
  so no draw and no tile animation; the cells are freed for the take test, and `next_id` is
  untouched, so the creation numbers stay taken. `Arena::removed` lists them.
- **The idle house** (S-0216 claim 8, design). New hashed field `House::idle_test_house`
  (`#[serde(default)]`, clear in every game `new_game` starts; only `oracle::setup` sets it for
  `HUMAN`) and `World::of_idle_house`. Four hooks:
  - `acquire::candidate` rejects a target of an idle house (table C's first test), so no target
    search takes its objects.
  - `acquire::retaliate` ignores an attacker of an idle house. The first run of the test below
    found this path: an armed idle unit that fires draws a retaliation that bypasses the search.
  - `acquire::anger` adds no anger towards an idle house, so it never becomes a house's enemy
    house.
  - `damage::apply` returns `Unaffected` for an idle house's object, before the hit and the crush
    response.

  The arena's computer houses run no AI brain, so no explicit order picks targets.
- **Synthetic ground.** `oracle::synthetic_ruleset` adds to the combat ruleset the neutral and
  special countries and an unarmed base-unit vehicle `BASE` of both player countries (all
  invented); `oracle::synthetic_ground` uses it and adds start waypoint 3.
- **The AI fixture's starts.** `glasnost_ai::fixture::setup` now gives Alpha start 0 (west) and
  Beta start 1 (east) instead of random starts. With the house draws in front of the position
  draw, seed 5 put Alpha in the east, and 20 debug AI tests (in `ai`, `ai_sw`, `economy`,
  `expansion`, `grinder`, `gunner`, `psychic` and `water`) failed. Their cells are laid out for
  Alpha in the west, which seed 5 used to give by chance. With fixed starts all of them pass
  again. Random starts are still tested in the simulation (S-0009 claim 19 and
  `claim_14_variant_c_position_draw_between_the_rounds`).
- **Formats.** Save format 94 to 95 and AI game format 57 to 58 (the new hashed field, and
  every skirmish replay now starts from another stream).

## Claims

| Claim part | Status | Where, and why |
|---|---|---|
| S-0202 r62 claim 14: one R(450, 1800) per house, 2 × (*p* + 2) draws in two rounds in claim 19's order, then the final R(0, 65535) and no other draw before tick 0 (R17) | tested | `claims_s0202_houses::claim_14_two_house_rounds_then_the_final_draw` (*p* = 2, 3, 4; seeds 1 to 5; exact stream) |
| 14: variant C, the human's random-position draw between the rounds | tested | `claim_14_variant_c_position_draw_between_the_rounds` |
| 14: variant D, the water fill's draws directly after round 2 | tested | `claim_14_variant_d_fill_after_the_second_round` |
| 14: variant B, a neutral pre-placed vehicle's creation draw after every house draw | tested | `claim_14_variant_b_a_neutral_vehicle_after_both_rounds` (the vehicle's report value is the raw value right after round 2; seeds 1 to 5) |
| 14: `quirks.double_start_resolution` off: no early phase, the rounds together before the fill | tested | `claim_14_without_the_early_phase_the_rounds_are_adjacent`; `claims_s0009::claim_20_early_draws_before_the_fill` (both positions) |
| 14: the first round's houses take no further part | trivially | our engine creates the houses once; only the draws are taken twice |
| 14: a network game's start | built | the same `new_game` path as a skirmish |
| 14: a campaign mission creates its houses once, in round 2's place | seam | we have no campaign start |
| 14: our sandbox start takes no house draw | tested (design) | `claim_14_sandbox_takes_no_house_draw` |
| S-0009 claims 19 and 20 (S-0202 r62 Changes) | tested | `claim_20_early_draws_before_the_fill` and `claim_46_switch_off_standing_position` now count both rounds |
| S-0216 r67 claim 8 *Game start*: five houses in claim 19's order, the human at start 3, a player and not an observer, crates off | tested | `arena_setup::pt6_game_start_houses_in_claim_19s_order`; on the records' map `arena::pt6_the_records_game_start_in_our_setup` (local lane) |
| *Game start*: two rounds of five, three base units with their creation draws, the final R(0, 65535), no budget unit, no other draw before the first group | tested | `pt6_game_start_two_house_rounds_three_base_units_then_the_final_draw` (four seeds; `first_raw` against the predicted count) |
| *Game start*: the base units' creation numbers before every set-up object's, in house order | tested | `pt6_computer_base_units_are_taken_off_and_the_humans_stays` |
| *Game start*: the human house's country | request | **Q-2446**; interim side A's country (design) |
| *Removal before the groups*: every vehicle the human house does not own, the neutral house's included; no draw, no animation, cells freed, creation numbers kept; the neutral house's infantry stay | tested | `pt6_the_removal_takes_no_draw_starts_no_animation_and_frees_the_cells` |
| *Removal*: none of them at the next tick; the human's base unit on its base cell, unmoved and unhurt, all game | tested | `pt6_computer_base_units_are_taken_off_and_the_humans_stays` (600 ticks); `pt6_the_records_game_start_in_our_setup` (local lane, to the game's end) |
| *Removal*: no object targets or damages the human house's objects | tested (design) | `pt6_no_object_targets_or_damages_the_idle_house` (flag on and, as a control, off; a direct hit and 300 ticks beside an armed enemy) |
| *Removal*: no retaliation against the human house's objects, and no house's anger towards it ("no house treats it as an enemy") | tested (design) | the same test: the idle unit is armed and fires, so without the `retaliate` hook the enemy targeted it (the test's first failure); the `anger` hook keeps the enemy house unset |
| *Removal*: kills that bypass target search and `damage::apply` | seam | none reaches a unit at start 3 in the arena; the flag covers the two paths the spec names |
| *Acceptance*: the first creation draw at M-0019's *N*₀ | see Lanes | `arena::m0019_the_first_creation_draw_takes_the_raw_value_at_n0` (local lane) |

## Quirks

No new switch. Neither delta names a quirk, and none of the existing switches changes meaning.
`quirks.ranged_draw_half_span` applies to the house draws as to every ranged draw (claim 4).
No C7 verdict is needed.

## Acceptance: M-0019's *N*₀

`arena::m0019_the_first_creation_draw_takes_the_raw_value_at_n0` (local lane) still fails. The
place no longer depends on the pairing, but it misses M-0019 in every seed (raw values):

| Seed | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| Before (r54, r59) | 693 | 685 | 685 | 695 | 693 | 689 |
| After (r62, r67) | 711 | 698 | 702 | 709 | 711 | 708 |
| M-0019 *N*₀ | 706 | 709 | 711 | 718 | 706 | 701 |
| After − *N*₀ | +5 | −11 | −9 | −9 | +5 | +7 |
| Round 1, round 2 (raw values) | 6, 6 | 6, 5 | 6, 8 | 9, 5 | 10, 7 | 8, 8 |

The test now prints the rounds' places by seed. Filed as **Q-2447** (what else the records'
game start takes before *N*₀). We changed nothing to fit the numbers. The other local-lane
checks pass: `pt6_the_records_game_start_in_our_setup` (five houses, one human object on
waypoint 3, 19 vehicles taken off, which are the two computer base units and the 17 the map
gives the neutral house, and the base unit unmoved and unhurt to the end),
`m0006_bunkers_and_placed_cells_in_our_setup`, `m0006_placements_parse` and
`m0019_creation_order_and_positions_in_our_setup`.

## Lanes

Main (53e846c) against the branch. The base binaries and tests were built from the
worktree at 53e846c before any change. All lane outputs are under `local-lane/m8/i87/`
(git-ignored). Every skirmish start now takes the house draws, so every lane moves.

| Lane | Before | After |
|---|---|---|
| M6c sweep, seeds 1 to 16 | smallest minimum 0; 10 seeds with no counter below 3; seed 5 ends at 9 890 | smallest minimum 0; 9 seeds with no counter below 3; seeds 5 and 16 end at 8 447 and 5 801 with their jumpjet and charger counters at 0 or 1 |
| M7a sweep, seeds 1 to 16 | smallest minimum 0; 1 seed with no counter below 3 | smallest minimum 0; none (the psychic sensor orders stay at 1 to 2, as before) |
| M7b sweep, seeds 1 to 16 | smallest drain minimum 3 | smallest drain minimum 3 |
| Arena, `arena --record all --threads 4` | 11 of 155 pairings fail (LTNK–E1, TELE–APOC, TELE–V3, CAOS–APOC, CAOS–TTNK, CAOS–E1, CAOS–E2, DISK–FV, INIT–E2, ROBO–HTK, MIND–E1) | 10 fail. CAOS–APOC, CAOS–TTNK and CAOS–E2 now pass. TELE–ROBO (4 side-B col wins where the stock rows are all draws) and TELE–BFRT (2 side-A row wins) now fail. The other eight still fail, with changed counts: LTNK–E1, TELE–APOC, TELE–V3, CAOS–E1, DISK–FV (still the win test), INIT–E2, ROBO–HTK and MIND–E1 |
| `glasnost-ai --include-ignored` (release) | 246 passed, 0 failed | 246 passed, 0 failed |
| AI quick lane, seeds 7 to 10, maps 0, 2 and 8, sides 0–1, 0–2, 1–2 (36 games) | every game ends, at 25 734 to 91 418 ticks | every game ends, at 19 974 to 93 847 ticks; the winners and tick counts change in most games |
| `glasnost-sim --include-ignored` (release) | not run; the brief's known failure is `pathing_c7p4::diag` | 5201 passed, 1 failed: `pathing_c7p4::diag` only |
| Goldens | | all nine move: `combat`, `duel`, `duel-diagonal`, `duel-m6c`, `duel-m7a`, `duel-m7b`, `duel-naval`, `scripted`, `skirmish` |
