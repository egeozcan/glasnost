# S-0213 r89: claim ledger, M8 step A3 intake 66

S-0213 r87 to r89 (commit 5dfbf5b) answers **Q-2226** to **Q-2230**, all filed by
[intake 63](intake63-S-0213-S-0301.md), which built r87. The diff adds the rules below and the
Scenario O8 parts that back them: q (6), x (6)'s per-tick record and pad-bound game, x (7)'s third
game, and ac. It also adds one Changes-for-other-specs entry (S-0501 claim 6). Requests filed:
**Q-2246** and **Q-2247**.

## Built

- **The place of an attack-move click** (claim 16b, Q-2230). `routes::order_place` no longer
  takes the order's attack-move mark. An attack-move click has the place the same click gives
  without entry:
  - on a cell, the clicked cell, even where the nearby-cell search moved the destination;
  - on an object, the target's place when the order names a target, and otherwise the cell the
    object stands on at the click.
- **The point a delete selects** (claim 16e, Q-2226). `routes::next_after` finds the point in the
  longest holder's route as it stands before the delete.
  - When the deleted point is that route's last, the selected point is the one (*n* mod *C*)
    places after the loop's first point, with *n* the points from the loop's first to the
    route's last and *C* `Route.loop_len`.
  - A route without a loop gives no point.
  - A point held at two places takes the point after its first place.
- **A copied capture issued by a vehicle** (claim 16b, *Joins*, Q-2227 question 2). No code
  change: the recorded order is issued as it is (intake 63), and the vehicle gets *capture*
  with the building as its destination and no target. S-0215 claim 28 leaves such a vehicle as
  it is (the hunt rule never applies in our engine), and only infantry make a building entry
  (`structures::entry::entered_building`). Tested up to the vehicle's arrival next to X.
  - After that, the specs conflict. Part ac allows no attack on X while N is on its way to
    F. But S-0209 row VP28 gives a driving vehicle *enemy obstacle* for a cell of an enemy
    building, and S-0207 claim 15 table T4's repeated check then makes it attack X by an
    override (`drive::attack_obstacle`). No vehicle row exempts a *capture* destination.
  - In the synthetic test the override comes seven ticks after the vehicle stands next to X.
    The engine is unchanged, and the full-window test is ignored on request **Q-2247**.
- **A turreted building under *move*** (claim 16d, Q-2228). No code change: S-0215 claim 11
  gives a turreted building no ready mark under *move*, so a later *attack* stays queued and it
  never fires. One without a turret starts it. Both are now tested.
- **An aircraft's issue at a flight's end** (claim 16d, Q-2229, and Changes for other specs,
  S-0501 claim 6). Intake 63's interim stays as code:
  - the issue is made at the landing trigger, before the pad-bound test;
  - it is preceded by the queue of *wait* (`routes::issue_later`);
  - no landing is begun in that tick.

  The citation now names r89. The *further waypoint* step above it in `air::landing_trigger`
  reads `Object.queued_dests`, which no production path fills, so that step and the route issue
  never meet in play.
- **Intake 63's S-0301 `design:` note** (`emitters::fire_place_let_go`, the retarget test for a
  system that is not of the fire kind). No answer covers it, since S-0301 is still r63 and
  S-0616 still r20. It is now request **Q-2246**, marked `design (Q-2246)`.

## Claims

| claim | status |
|---|---|
| 16b r89 an attack-move click on a cell has the clicked cell's place, not the cell the search finds; entry changes only the kind (part q (6)) | Tested: `claim_16b_an_attack_move_click_on_a_cell_has_the_clicked_cells_place` (an armed vehicle and an enclosed cell, end to end: the point add, its place and destination with and without entry, and the applied marker on the clicked cell) |
| 16b r89 an attack-move click on an object: the target's place when its order names a target; the object's cell at the click when its order would be a *move* without entry | Tested: `claim_16b_an_attack_move_click_on_an_object` (an attack on an enemy end to end; the *move* case through `order_place`, since no click on an object gives our armed test vehicle a *move*) |
| 16b r89, Joins: a copied action a click would not give the joining holder is applied as for any actor; a vehicle issuing an engineer's copied *capture* gets *capture*, X as its destination and no target, and heads for X (part ac) | Tested: `claim_16b_a_vehicle_issues_a_copied_capture_and_heads_for_x` (every tick from the release until T stands next to X, with N still on its way to F: *capture* current, no target, X its destination, no shot, X's health and owner kept) |
| 16b r89, Joins: the vehicle does not attack X while N is on its way to F; X never captured (part ac) | Blocked: Q-2247. S-0207 claim 15 table T4 with S-0209 row VP28 makes T attack X by an override once its next step is in X's foundation. `claim_16b_a_vehicle_with_a_copied_capture_never_attacks_x` (the full window) is ignored on Q-2247 and fails when run. **Superseded by S-0213 r94** (Q-2247 answered: T attacks X by the override at its step into X): the ignored test is replaced by `claim_16b_a_vehicle_with_a_copied_capture_attacks_x_by_the_override` ([intake 70](intake70-S-0213-S-0301.md)) |
| 16b r89, Joins: the vehicle's route goes on at its next fall-back to rest | Partly: rated `unmeasured`. A *capture* point is terminating, so the copy has no later point, and the test checks that the route is used up. The fall-back issue is the ordinary rule (`routes::fall_back`, intake 56) |
| 16b r89 Q-2227 question 1 (primary and a rally point on an object sent directly; a building's *unload* a point at its own place) | Built and tested by intake 63 (r87), no change |
| 16d r89 an aircraft's issue at a flight's end is S-0501 claim 6's planned-route step and comes before the pad-bound test; a pad-bound aircraft away from its pad issues and does not fall back (part x (6), pad-bound game) | Tested: `claim_16d_a_pad_bound_aircraft_issues_before_the_pad_bound_test` (built at its pad and at rest there; it flies to P1's issued destination and on to P2 with no crash and no turn back towards the pad). P1's order is the car's, and its recorded destination is a cell next to P1's own cell, so the test follows the issued destination. Recorded after P2, as the part asks: the aircraft flies back near its pad and hovers there under *move* with a cell destination for the rest of the test (over 1,300 ticks); left to S-0501 |
| 16d r89 an aircraft that issues a point begins no landing in that tick and stays in flight; the issued destination is given while neither landing nor taking off, so it flies on with no landing or take-off (part x (6) record) | Tested: `claim_16d_an_aircraft_flies_on_at_a_flights_end_tick_by_tick` (every tick from P1 until it heads for P2: no landing, no take-off, height not below cruise, a destination in every tick, no *guard* current or queued). Our engine plays no take-off sound for aircraft, so the take-off state is the observable |
| 16d r89 the issue is preceded by claim 16c's queue of *wait* | Partly: rated `unmeasured`. Built (`routes::issue_later`), and the per-tick test shows no *guard*. The test does not require *wait* to be current, since that depends on readiness |
| 16d r89 the issued order with no destination does that order's own effect | By construction: the recorded order goes through the ordinary order path (claim 5) |
| 16d r89, Q-2228 a building under *move* given an *attack* outside route mode: its route ends and *attack* is queued; with no turret it starts and fires (part x (7) first game) | Tested: `claim_16d_a_building_under_move_does_nothing` (`claims_s0213_r84.rs`, POST, now cited to r89) |
| 16d r89, Q-2228 a turreted building keeps *move* current and *attack* queued, and makes no shot (part x (7) third game) | Tested: `claim_16d_a_turreted_building_under_move_never_starts_a_later_attack` (TOWER, 1000 ticks) |
| 16d r89 quirk `click_area_skips_kind_test` off: no turreted building is left under *move* with an *attack* that never starts | Wording only: the off position gives no building a route point (existing test `claim_16d_a_click_area_skips_the_kind_test_both_ways`). Judged again under C7 (below) |
| 16e r89, Q-2226 question 1 the delete of the longest holder's last point on a loop that ends early selects the point (*n* mod *C*) places after the loop's first | Tested: `claim_16e_a_delete_of_an_early_loops_last_point_selects_round_the_loop` (*n* = 7, *C* = 3: one place after the loop's first; the old interim gave the loop's first) |
| 16e r89 on a loop that runs to the route's end, the loop's first point; a route without a loop gives no point | Tested: `claim_16e_a_delete_of_a_last_point_on_a_full_loop_and_without_a_loop` |
| 16e r89, Q-2226 question 2 a route that holds the deleted point at two places takes the point after its first place, not the count round the loop | Tested: `claim_16e_a_delete_of_a_point_held_twice_takes_the_one_after_its_first_place` (the deleted point is the route's last and stands earlier too) |
| Changes for other specs, S-0501 claim 6 (its planned-route waypoint is S-0213 claim 16d's issue) | Cited in `air::landing_trigger`; S-0501 itself is the lead's to change |

## Requests

- **Q-2246**: when the fire place's retarget test is made for a system that is not of the fire
  kind (S-0301 claim 5, S-0616 claim 1). This is intake 63's unnumbered `design:` note in
  `emitters::fire_place_let_go`. The interim is unchanged: the test is made in the held system's
  own turn, after its kind's work. Answered by S-0301 r67 (the test is made at the change of the
  target), built by [intake 70](intake70-S-0213-S-0301.md).
- **Q-2247**: whether a vehicle in *capture* whose destination is an enemy building attacks
  that building as an enemy obstacle (S-0207 claim 15 table T4, S-0209 row VP28) when its next
  step enters the building's foundation, against S-0213 r89 part ac. There is no interim code:
  the engine follows S-0207 and S-0209, and the full-window test of part ac is ignored.
  Answered by S-0213 r94 (it attacks by the override), built by [intake 70](intake70-S-0213-S-0301.md).

## C7

No quirk added; r89 adds none. It adds one consequence to the off text of
`click_area_skips_kind_test`, so the switch is judged again in
[c7-play-fixes.md](c7-play-fixes.md) (intake 66): still rejected. The layer file is unchanged.

## Formats

No new state and no order change, so the save format (92), the AI game format (54) and the wire
version (12) are unchanged. `GLASNOST_UPDATE_GOLDEN=1` on the gate crate moved no golden.

## Lanes

Before is main at 5c72f70. After is this branch. Outputs are in `local-lane/m8/intake66/`, which
is git-ignored.

| Lane | Before | After |
|---|---|---|
| M6c sweep, seeds 1–16 (29 counts) | passes | the same minimum, median and maximum for every count |
| M7a sweep, seeds 1–16 (26 counts) | passes | the same for every count |
| M7b sweep, seeds 1–16 (34 counts) | passes | the same for every count |
| Arena `--record all`, scored as intake 47 / A6: M-0001 | 9 of 96 failing | 9 of 96, `ours-M-0001.csv` byte-identical |
| M-0002 | 1 of 24 | 1 of 24, identical |
| M-0003 | 1 of 35 | 1 of 35, identical |
| Arena total | 11 of 155 | 11 of 155 |
| `glasnost-ai --include-ignored` (release) | passes | passes |
| AI quick lane, seeds 7–10, maps 0, 2 and 8 | 36 games | every game ends at the same tick with the same winner |

None of the changed paths runs in an AI game. Routes are a human input, and no headless game
uses one.
