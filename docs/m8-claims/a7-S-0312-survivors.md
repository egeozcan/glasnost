# S-0312 claim 18: building survivors, M8 step A7

This step builds **S-0312 revision 44's building survivors**: claim 18 with F4 and F5, claim 19
(g)'s per-cell order, claim 21's second round, claim 24's start after the absorbed passengers'
cells and claim 25's orders, and the **sale survivors of S-0408 claim 7**, which share F5's
count. Before this step F5 gave *C* = 0 for every building (sides were treated as missing), the
sale released none and a vehicle's crew was always the technician. Sides, the side crews and
the survivor divisors were already imported (S-0412); this step reads them.

## Built

- **`combat::survivors`** (new). `source` reads what a round needs of a building (owner, top-left
  cell, foundation, crewed flag, the no-crew mark, the changed-owner mark `combat.captured`, the
  charge planter, whether its type produces buildings, whether it is armed). `crew_type` is F4,
  `count` is F5, `survivor_type` is claim 18's table *Type* (and S-0408's *Type choice*),
  `place_infantry` creates and places one crew or survivor, `destruction_round` is one round of
  claims 18 and 19 (g), `sold` is S-0408 claim 7.
- **Armed** (F4, S-0312 Terms) is claim 6's second weapon choice (`current_weapon`), or a type
  infantry can occupy whose occupants may fire from it (S-0301 Terms, *garrisoned building*).
- **`combat::death`**: the vehicle crew (claim 13) now takes its type from F4 (it was always the
  technician), with F4's percent draw for an armed vehicle whose country has a side. The
  building's death takes the passengers' cells from `absorb::destroyed` (now returning how many
  it used) and runs the round from the next cell; killing damage that ignores defences sets the
  building's no-crew mark first. `removal_turn` reads the source before the building leaves play
  and, under `quirks.building_removal_second_round`, runs the round again from the first cell.
- **`econ::sell::release_occupants`** calls `survivors::sold` after the absorbed passengers and
  the garrison (S-0408 claim 9's order).
- **`absorb::destroyed`** also kills the passengers when the building carries the no-crew mark.
- **Import**: building types read `Crewed` (`BuildingType::crewed`, the building combat table of
  `convert::sim_combat`); absent is no (Q-2135).
- **State**: `CombatState::no_crew` (serde default false). Save format 86 to 87, AI game format 48
  to 49.

## Design choices

| question | our choice |
|---|---|
| Which free standing position a survivor or crew takes (claims 13 and 18 give none, and no draw) | The first free one in S-0208 table P order, reading the cell's infantry positions only; no draw (Q-2132). The same as the crew used before. **Q-2132 answered by S-0312 r51** (claims 13 and 18: S-0208 claim 3's table on the cell's ground record, a centre-class choice that the first row does not refuse taking one R(0, 3) draw), built in [intake 55a](intake55a-S-0301-S-0312-S-0313-S-0303-S-0318-M-0004.md) |
| What is clear for movement in claim 19 (g)'s smudge step | S-0211 category *open*, from the land type and overlay without the cell's objects (Q-2133). **Q-2133 answered by S-0312 r51** (claim 19 (g)'s three tests: the record's object-present flag, a wall overlay, a tracked multiplier of exactly 0), built in [intake 55a](intake55a-S-0301-S-0312-S-0313-S-0303-S-0318-M-0004.md) |
| `Crewed` absent on a building type | No crew (Q-2135). **Q-2135 answered by S-0312 r51**: absent is no for every kind of type, as built |
| The map's no-crew flag (F5) | Not read: S-0009 gives no such field (Q-2131). The mark exists and killing damage that ignores defences sets it. **Q-2131 answered by S-0312 r51** (claim 18, *No crew*): no field of a map's building entry sets the state, only that killing damage does, as built |
| What *g*'s charge planter term reads | The building's planter record (claim 18 *Chance*), whether or not the planter is still in the game; the attack order needs the planter in the game and not allied with the owner |
| Claim 21's second round | Starts at the first foundation cell (no absorbed passenger comes out then); *C* afresh from F5 with the refund as it stands before the building leaves play, so a hit that ignored defences gives *C* = 0 through the no-crew mark |
| A sale attempt whose type choice gives no type (a missing crew or technician) | The attempt ends with no creation draw, as S-0312's Edge cases give for a destruction |
| A sale attempt that gives a second engineer 100 times | No survivor for that attempt, its draws taken (S-0408 Constants and Edge cases) |
| Who counts as a player house for claim 25 | A house with the human-controlled flag (an AI house of our engine has it: `econ::setup::skirmish` makes every player house human-controlled); the neutral and special houses and a test-only computer-controlled player house take *hunt* (glossary *AI house*) |
| The importer warning for a side crew or technician that is an engineer type (S-0408 Edge cases) | Not built |

## Claims

All tests are in `crates/deterministic/sim/tests/claims_s0312_survivors.rs` and go through
damage resolution (`damage::apply`), `death::removal_turn`, the sale order or the sale's release
step; draws are checked against a replay of the specs' draw lists on a copy of the stream.

| claim | status | tests or reason |
|---|---|---|
| F5, the bounds (quotients below 1, 1 to 5, above 5; *K* = 0 and *K* < 0 give 1) | tested | `f5_count_bounds` |
| F5, the side's divisor, the doubled divisor of a building that changed owner, side 2 | tested | `f5_side_divisor_and_changed_owner` |
| F5, *C* = 0: type without the crewed flag, side outside 0 to 2, country without a side, *q* = 0 | tested | `f5_zero_cases`; through the kill, no survivor and only claim 19 (a)'s draws: `claim_18_no_round_when_c_is_zero` |
| F5, no crew: killing damage that ignores defences | tested | `claim_18_ignoring_defences_no_crew` |
| F5, no crew: the map's no-crew flag | tested | the mark is tested (`f5_no_crew_mark`, destruction and sale); no map sets it (Q-2131, answered by S-0312 r51: no map field exists, [intake 55a](intake55a-S-0301-S-0312-S-0313-S-0303-S-0318-M-0004.md)) |
| claim 18, the chance R(0, *g*) per cell while fewer than *C* are out, the type draws, the creation draw, health R(5, S), the scatter request; claim 19 (g)'s smudge step in every cell, also after *C* are out | tested | `claim_18_round_draws_and_cell_order` (*C* = 1, 2, 5; 12 seeds each) |
| claim 18, *g* with a charge planter and with a changed owner (no percent draw *p* then) | tested | `claim_18_chance_bound` |
| claim 18, table *Type*: the engineer row (*p* < 25, a building factory that never changed owner) and F4's draw after it for an armed building | tested | `claim_18_engineer_row` (engineer, technician and side crew all seen) |
| claim 18, a changed owner: F4's draw only; armed by the occupants' fire | tested | `claim_18_changed_owner_and_garrison_armed` |
| claim 18, no chance once the owner's game is over; the smudge steps go on | tested | `claim_18_owner_defeated` |
| claim 18, a cell with no free position: the creation draw, no health draw, not counted against *C* | tested | `claim_18_full_cell_and_unclear_cell` |
| claim 18, which free position | tested | S-0312 r51 (Q-2132 answered): S-0208 claim 3's table, centre class, one R(0, 3) unless the first row refuses; no entry test. The survivor tests replay the position draw (`replay_round`); [intake 55a](intake55a-S-0301-S-0312-S-0313-S-0303-S-0318-M-0004.md) |
| claim 19 (g), a cell not clear for movement takes no smudge step | tested | S-0312 r51 (Q-2133 answered): `claim_19g_clear_for_movement` (the object-present flag, a wall overlay, a zero and a below-threshold tracked multiplier, the cell's own objects ignored); the high-bridge case is partly, [intake 55a](intake55a-S-0301-S-0312-S-0313-S-0303-S-0318-M-0004.md) |
| claim 19 (g) and claim 26, the smudge itself | seam | smudge types are not modelled: no type fits, no draw (as before) |
| claim 18, the attack order against an unallied planter; an allied planter gives the ordinary order | tested | `claim_18_attack_order_against_the_planter` |
| claim 25, survivor orders by owner (*move* for a player house; *hunt* for the neutral and special houses) | tested | `claim_25_survivor_orders_by_owner` |
| claim 24, the survivors start after the cells the absorbed passengers used, a passenger that could not be placed using its cell too | tested | `claim_24_survivors_start_after_absorbed_passengers` |
| claim 21, the second round from the first cell with *C* afresh; *C* = 0 after a hit that ignored defences | tested | `claim_21_second_round` |
| Edge cases, a missing survivor type | tested | `departure_missing_survivor_type` |
| claim 13 and F4, the crew type for no side, sides 0 to 2 unarmed and armed, a side past 2 unarmed and armed (a discarded draw) | tested | `claim_13_f4_crew_types` |
| Import mapping, `Crewed` on building types | tested | `crewed_absent_on_a_building_is_no`; default absent = no (Q-2135, answered by S-0312 r51 for every kind of type; the vehicle case: `claim_13_crewed_absent_on_a_vehicle_is_no`, [intake 55a](intake55a-S-0301-S-0312-S-0313-S-0303-S-0318-M-0004.md)) |
| S-0408 claim 7, exactly *C* attempts, the draw order, full strength, facing 0, *move* for any owner | tested | `s0408_claim_7_sale_survivors` (*C* = 1, 2, 5; a player and the neutral house) |
| S-0408 claim 7, at most one engineer: a second engineer repeats the whole choice | tested | `s0408_claim_7_one_engineer` |
| S-0408 Edge cases, at most 100 type choices | tested | `s0408_engineer_choice_limit` |
| S-0408 claims 6, 7 and 9, through the sale order | tested | `s0408_claim_7_through_the_sale_order` |
| S-0312 F5 and S-0408 claim 7, the same *C* for a sale and a destruction | tested | `sale_and_destruction_share_the_count` |
| S-0408 Edge cases, the importer warning for an engineer crew type | not built | |

## Requests

- **Q-2131**: the map field that leaves a building without crew. Answered by S-0312 r51.
- **Q-2132**: which free standing position a crew or destruction survivor takes. Answered by S-0312 r51.
- **Q-2133**: which cells are clear for movement in the smudge step. Answered by S-0312 r51.
- **Q-2135**: the default of `Crewed` for building types. Answered by S-0312 r51.

All four are built in [intake 55a](intake55a-S-0301-S-0312-S-0313-S-0303-S-0318-M-0004.md); their files are the lead's to close.

Q-2134 is unused.

## Goldens

All nine moved from tick 0, because every object's combat state gains the hashed no-crew mark:
`combat`, `duel`, `duel-diagonal`, `duel-naval`, `duel-m6c`, `duel-m7a`, `duel-m7b`, `scripted`
and `skirmish`. The layout is the only change: no synthetic ruleset of the gate sets `Crewed` on a
building type (so every *C* is 0) or gives a country a side (so every crew stays the
technician). The M6c, M7a and M7b sweep counters below are unchanged as well.

## Arena

Local lane, `local-lane/m8/a7/after/` (never committed), scored as intake 47, A6 and intake 49
score them. *Before* is intake 49's *after* (`local-lane/m8/a3-49/after/`, measured at 8f96a14):
`git diff 8f96a14 bfa1ec5 -- crates/` is empty, so main bfa1ec5, this step's base, plays the
same games.

| record | before | after |
|---|---|---|
| M-0001 | 10 of 84 (3, 9); 140 of 152 | 10 of 84 (3, 9); 140 of 152 |
| M-0002 | 2 of 24 (0, 2); 45 of 47 | 2 of 24 (0, 2); 45 of 47 |
| M-0003 | 4 of 24 (0, 4); 38 of 42 | 4 of 24 (0, 4); 38 of 42 |
| all | 16 of 132 | 16 of 132 |

(Scored pairings failing, with those failing the win test and the side outcome; side-wise
unanimous stock outcomes reproduced.) Our three CSVs are byte-identical before and after.

## Seed sweeps

`GLASNOST_SWEEP=1-16`, release, M6c, M7a and M7b: all pass, and every counter line is identical
to intake 49's after (`local-lane/m8/a7/after/sweep-*.txt`).

## AI

- `cargo test --release -p glasnost-ai -- --include-ignored`: all pass.
- Quick lane (24 games, `GLASNOST_AI_QUICK=1`, sides 0 and 2 and sides 1 and 2, maps 8, 2 and 0,
  seeds 7 to 10; `local-lane/m8/a7/quick/{before,after}/`, before built at bfa1ec5): every game
  changes, as destroyed and sold buildings now release survivors and crews take their side's type.
  Every game that ended before still ends; only sides 1 and 2, map 8, seed 9 reaches the
  120000-tick limit, before and after. House 0 wins 7 of 12 games of sides 0 and 2 before and after, and 3 of
  12 of sides 1 and 2 before and after (the winners of single games differ). Over the 23 games
  that end, the mean length goes from 49 820 to 50 381 ticks and the median from 48 615 to 48 790.
  No panic.
