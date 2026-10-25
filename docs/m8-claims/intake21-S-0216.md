# S-0216 r20: claim ledger, M8 step A3 intake 21

This intake takes in one new spec, **S-0216 revision 20** (units placed after the game has
started; spec commit 8b1e0b8). It answers Q-1711 questions 2 and 3 for play-time placements and
Q-1713 questions 2 and 3 (claim 6) for the measurement records' units (406d635, 3635992). Q-1711
questions 1, 4 and 5 and Q-1713 question 1 stay open in the spec.

**What a play-time placement is.** S-0216 Terms defines it as the placement of the measurement
harness of its scenarios PT1 to PT5, "and so can another rule of the game where its own spec says
so". No other spec names S-0216, and Terms itself excludes by name the passengers coming out of an
absorbing building, the survivors of a sale, a transport letting out its passengers, a
paratrooper's drop, garrison occupants put out, a released parasite on its host's cell, a
returning parasite (S-0614 claim 20, which already makes the same choice) and a map entry. So no
placing rule of the game becomes a play-time placement at this intake: each path keeps the rule of
its own spec (table below). The rule is built once, as `objects::place_in_play`, and our own
placing steps use it: the testkit's harness (`testkit::spawn_in_play`) and the synthetic arena of
`glasnost-tools` (`arena::setup`).

**Code.**
- `objects::place_in_play(w, id, cell, v) -> bool`, for a created vehicle or infantry unit not yet
  on the map, in this order:
  1. An infantry unit on a usable cell: S-0208 claim 3's choice for a desired point of class
     centre, on the ground record (`walk::choose_position`): no position when the vehicle-present
     or object-present flag refuses (no draw); otherwise one R(0, 3) and the first free position of
     row R*k*; none free: refused, the draw kept (claim 2).
  2. The entry test: `occupancy::infantry_outcome` or `vehicle_outcome` for the cell, read with
     the cell's own level as reference (no approach direction or source elevation); only *clear*
     accepts (claim 5).
  3. A vehicle of a type with either visceroid flag: one R(0, 29), only now (claims 3 and 5).
  4. `objects::place` in the ground layer at the centre (a vehicle) or the standing point (an
     infantry unit), on the terrain surface there, with facing 256 × *v* for body and turret
     (`facing_from_value`, claim 1). `place` runs the placement fall-back as for every placement.
  5. When the unit is then ready to change (`process::ready`, S-0215 claim 11), its queued resting
     behaviour becomes current (start-next), so nothing is queued (claim 4). A unit that is not
     ready keeps it queued.
  A refused unit stays off the map with every draw its attempt took; the function says so and the
  caller decides.
- `testkit::spawn_in_play(w, kind, name, house, cell, v, behaviour) -> Option<ObjectId>`: the
  creation (and its creation draw, S-0202 claim 10), the placement, then the given behaviour
  queued without a start (S-0215 claim 7, claim 4). A refused unit is discarded with its creation
  set, as `mutation::discard` does: it never enters play and its draws and creation numbers stay
  used.
- `glasnost-tools` synthetic arena (`arena::setup`): units now go on the map by
  `testkit::spawn_in_play`, facing the nearest facing value of the direction to the enemy front
  (exact, since the fronts lie on the x axis). Infantry stand at the position their draw gives and
  rest in the fall-back's behaviour instead of a forced *guard*. A refused placement is an error of
  the scenario.
- No new state: save format 78 and AI game format 40 are unchanged.

**Which side each piece lives on** (S-0216 *Changes for other specs*).
- S-0206 claim 5 (the facing conversion for a play-time placement): on S-0216's side, in
  `place_in_play`. `objects::place` already sets both facings at once for every placement, and the
  conversion is the one of S-0009 claim 39 (`glasnost_map::objects::facing_from_value`).
- S-0215 claims 4, 11, 14 and 15 (a unit ready to change when placed has the resting behaviour
  current, rated low until PT4): on S-0216's side only, in `place_in_play`. The placement
  fall-back of every other placement (`process::fall_back` from `objects::place`) keeps S-0215's
  text, queue only, until S-0215 carries the change. Applying it there would change set-up,
  starting units and every placing rule of the game.

**Placement paths in the simulation** (`objects::place` callers and the brief's list).

| path | code | governing rule | result |
|---|---|---|---|
| map entries (set-up) | `objects::place_map_objects` | S-0009 claims 38 to 42, S-0208 claim 5 | set-up; claim 5 contrast. Unchanged |
| starting units (set-up) | `starting.rs` | S-0009 claims 45 and 46 | set-up. Unchanged |
| factory exits | `econ::release` (`put`, ground, naval and aircraft rows) | S-0404 claims 13 and 14, S-0417 claim 14 | own exit point, facing, entry test and draws. Claim 4's not-ready case agrees: `release::holds_back` keeps the resting behaviour queued. Unchanged |
| free unit of a building | `econ::harvest` (`free_unit`) | S-0402 claim 18 | own places and facings. Unchanged |
| crates (unit crate) | `crates.rs` | S-0520 claim 13 | a vehicle: direct placement accepting only *clear*, then a nearby search; consistent with claim 5. Unchanged |
| undeploy | `econ::deploy` | S-0415 claim 10 | own cell and facing. Unchanged. Deploying makes a building (S-0406), outside S-0216 |
| passengers out of a transport | `transport::put_out` | S-0507 claim 10 | excluded by Terms. Unchanged |
| paradrop | `transport::drop_out` | S-0522 claim 9 | excluded by Terms. Unchanged |
| survivors of a destroyed transport | `transport.rs` (S-0312 claim 12 survivors) | S-0312 claim 12 | own standing positions and entry test. Unchanged |
| passengers out of an absorbing building | `structures::absorb::put_out` | S-0312 claim 24, S-0408 claim 8, S-0605 claim 13 | excluded by Terms. Unchanged |
| survivors and crew | `combat::death` | S-0312 claims 13 and 18 | own positions. Unchanged |
| garrison occupants | `structures::garrison::put_out` | S-0508 claim 11 | excluded by Terms (occupancy ignored, no draw). Unchanged |
| parasites | `parasite::put`, `place_ordinary` | S-0614 claims 15 and 20 | excluded by Terms; claim 20 already makes claim 2's choice and claim 5's test. Unchanged |
| slaves emerging | `econ::slaves` | S-0402 claim 21 | own choice (S-0208 claim 3) and facing. Unchanged |
| made infantry | `mutation::place` | S-0611 claim 10 | own choice and facing. Unchanged |
| chronosphere members | `superweapons::warp` | S-0513 claim 39 | own landing. Unchanged |
| displaced units put back | `mp_triggers::put_back` | S-0524 claim 13 | own position and facing 0. Unchanged |
| strike and paradrop aircraft, spawned missiles | `strike.rs`, `superweapons::planes`, `spawn.rs` | S-0519, S-0522, S-0506 | aircraft: outside S-0216. Unchanged |
| buildings | `econ::placement`, `mp_triggers::replace` | S-0406, S-0524 | outside S-0216. Unchanged |
| reinforcements, trigger-created units, teams | none | S-0011 claim 30 hands them to teams, which have no spec | not built |
| our harness | `testkit::spawn_in_play` | S-0216 | **new**, follows claims 1 to 5 |
| our synthetic arena | `glasnost_tools::arena::setup` | S-0216 | **switched** from `testkit::spawn` |
| the oracle arena | `glasnost_tools::oracle::setup` | A1 interim (Q-1711 question 5) | **kept** (below) |
| tests and diagnostics | `testkit::spawn` (sim, app, tools tests), `path_report` | none | unchanged: a test fixture that places at the centre with *guard* forced, no draw, no entry test |

**The oracle arena** (the records' units). Q-1711 question 5, how the stock harness placed the
units of M-0001 to M-0003, stays open. S-0216 says that claims 1 to 5 hold for those units only if
they were play-time placements, and an ordinary play-time placement would have refused the
infantry on the *rock* cliff cells that the records say they stood on (claim 5, Q-1502). So
`oracle::setup` keeps the A1 interim, `testkit::spawn`: no standing-position draw, no entry test,
every infantry unit at its cell's centre point, *hunt* forced with its timer expired.
- Claim 1: the records' facings are map east and map west, 16384 and 49152, that is 256 × 64 and
  256 × 192, so the units start as claim 1 and S-0206 for a map entry both give. Nothing changes.
- Claim 4: forcing *hunt* with an expired timer has the effect claim 4 gives a queued *hunt* for a
  unit that is ready at its first update's first per-tick start: *hunt* is current and processed
  in that update. The interim stays as written, citing S-0009 claim 41.
- Claims 2, 3 and 5: the interim (centre point, no draws, no entry test).
- Claim 6: it holds however the units were placed, and it already holds in our games. The arena's
  houses are test-only computer-controlled player houses (`human = false`), so S-0215 claim 18's
  behaviour list does not hold back their hunters, *hunt* has the retaliate property by default,
  retaliation has no reach-region test, and the hunt search rejects across the cliff cells
  (S-0309 claim 7, built at M7b step 1c). The tests below confirm it. The arena CSVs are
  byte-identical before and after, and A1's rows 3, 4 and 8 fail as before (Q-1713 question 1
  and the mind-control item of *Arena mode (M7b)* stay open).

**Interim rules, our design** (Decisions of 2026-10-25; each is a case S-0216 leaves open).
- **Outside the usable area** (claim 2 `unmeasured`, PT2 part c): no position choice and no draw;
  the unit stands at the cell's centre, then the entry test. The entry outcome applies no
  usable-area check to a unit not yet on the map (S-0209 part 4a's "for a unit that is inside
  it"). This is how S-0614 claim 20's placement is built.
- **Infantry facing** (Open question): the same conversion, 256 × *v*.
- **A failed placement** (Open question): `place_in_play` leaves the unit off the map. Our harness
  discards it with its creation set; the synthetic arena reports an error.
- **Above the ground** (Open question): a play-time placement is always in the ground layer, on
  the terrain surface, as the Terms say ("on the ground of that cell").
- **A cell that is not a legal map cell**: refused, with no draw.

No request is filed: each item above is one of S-0216's own Open questions or `unmeasured` parts,
which its scenarios PT2 part c and PT3 to PT5 are meant to settle.

**Tests.** `crates/deterministic/sim/tests/claims_s0216.rs` (14 tests, synthetic types on a
synthetic map, draws checked against a cloned scenario stream).

| claim | status | test names or reason |
|---|---|---|
| 1 (vehicle facings) | tested | `claim_1_vehicle_facings_are_256_v_in_every_seed` (PT1's values, 37, 101 and 201 among them; a turreted and a non-turreted type; body and turret exactly 256 × *v*, no turn, the same in 3 seeds) |
| 2 (infantry standing position) | tested | `claim_2_empty_cells_stand_by_the_draw` (PT2 part a: 128 placements, position by the draw, PT2's bounds, never centre or top), `claim_2_occupied_cell_takes_the_first_free_position_of_its_row` (part b), `claim_2_cell_with_a_vehicle_refuses_without_a_draw` (part d), `claim_2_full_row_refuses_after_one_draw` (part e). The object-present refusal and its open-gate exception: S-0208 claim 3's existing tests of `walk::choose_position`. Outside the usable area: `claim_2_outside_the_usable_area_is_our_design` (our design, above). On a bridge deck: not established (Open question) |
| 3 (draws and their order) | tested | `claim_3_draws_in_placing_order` (PT3: creation, creation, R(0, 3), creation, creation, R(0, 3); with a visceroid vehicle third, R(0, 29) right after its creation draw; nothing else before the first update) |
| 4 (behaviour before the first update) | tested | `claim_4_resting_current_then_hunt_at_the_first_update` (PT4: a ready vehicle and infantry unit have the resting behaviour current and nothing queued; *hunt* queued without a start; current and processed in the first update), `claim_4_harness_queues_the_given_behaviour`. The not-ready case is S-0215's placement fall-back as built (`process::fall_back`, the factory's `holds_back`), unchanged; the late-start case is S-0215 claims 3 (b) and 11 as built |
| 5 (the entry test) | tested | `claim_5_vehicles_refused_on_water_and_wall_without_a_draw` (PT5 parts a and b, controls with their draws), `claim_5_infantry_refused_on_water_and_rock_after_its_draw` (parts c and d, flat *rock*). Other outcomes: S-0209's and S-0211's tests of the entry functions |
| 6 (units on cliff cells of no reach region) | tested | `claim_6_cliff_hunter_ignores_an_enemy_in_range` (question 2: no target from an enemy in weapon range in 120 ticks; no limited-scan mark), `claim_6_computer_hunters_retaliate_across_the_cliff` (questions 2 and 3: retaliation from and against such a cell; the hunt search rejects), `claim_6_player_hunter_does_not_retaliate` (the contrast). Also S-0309's `cells_of_no_reach_region_compare_as_one_region`, `test_house_retaliation_outside_the_listed_behaviours` and `neutral_retaliation_compares_threat_sums` (the threat comparison for a hunter that has a target). The limited-scan route: the mark is set by no rule yet (seam of S-0309 claim 22, movement specs). Question 1: open (Q-1713) |
| Formulas | tested | claims 1 to 3 above |
| Constants | tested | R(0, 3), 256 and R(0, 29) cited at their use and checked by the tests of claims 1 to 3 |
| State and ordering | tested | claims 3 and 4 above |
| Edge cases | none | the spec has none |
| Import mapping (`SmallVisceroid`, `LargeVisceroid`) | tested | already imported (`UnitRules::visceroid`, S-0202 and S-0009 claim 46); `claim_3_draws_in_placing_order`, `claim_5_vehicles_refused_on_water_and_wall_without_a_draw` |
| Scenarios PT1 to PT5 | by construction | Oracle scenarios; the tests above play each one's synthetic shape. No measurement record yet |
| Changes for other specs: S-0206 claim 5 | built (S-0216's side) | `place_in_play` |
| Changes for other specs: S-0215 claims 4, 11, 14, 15 | built (S-0216's side only) | `place_in_play`; the general placement fall-back waits for S-0215's revision |
| Open questions | interim / open | above; Q-1711 questions 1, 4 and 5, Q-1713 question 1 open |
