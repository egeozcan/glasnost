# M8 A3 intake 56: S-0213 r74 (route-mode presses)

Brief: `.claude/worktrees/M8-A3-INTAKE56-BRIEF.md`. Spec: S-0213 r60 to r74 (commits f807d22,
1b4b34d, 6f8a8ee, 6a934b1), answering Q-2138, Q-2166 and Q-2139 in part (all but the command
bar's route-mode button).

## What was built

- **`glasnost_sim::routes`**:
  - *Click areas* (claim 16b): `in_click_area` is the 16 by 16 pixel square from 8 pixels left of
    and above the place's screen position to 7 right of and below it. The client finds the
    points whose click areas hold the cursor (`Session::points_under`); the simulation picks among
    them.
  - *The press* (`press`, returning `Press::Take`, `Press::Refused` or `Press::Ordinary`): the
    candidate is the earliest-made point drawn for the player (`drawn_for`). It is taken when it
    is the player's own house's point and the point-click test (`accepts`) passes. Otherwise the
    press is tested against the earliest-made point of any house, drawn or not, and is refused
    with the invalid-point message when the test refuses it (`tested`). `highlighted` gives the
    point the client lights up.
  - *The kind test at the press* (claim 16d, `press_kind_refusal`): made for every left press on
    the battlefield with objects selected, outside every click area, whatever the click would do.
  - *Who gets a press's orders* (`press_list`, `press_orders`): the selection, then every holder
    of each selected route's last point, ordered by when it was given the point and then by
    object, without repeats. Each gets `Order::RouteJoin { actor, point, place }`.
  - *Joins* (`apply_join`): the order acts on the house's earliest-made point at its place (does
    nothing when none is left there). When applied, it closes a loop if the actor's route holds
    the point. Otherwise it adds the copy after the actor's own points, past 16 points, with the
    copy's loop replacing the actor's. The loop begins at the first place in the actor's route
    of the copied loop's first point, so it can begin in its own points and end before the last
    point (`Route.loop_to`).
  - *The release* (`click`): an ordinary click with objects selected. Behaviour orders become
    point adds, a detonate passes as `Order::Detonate`, deploys are refused (`Ignored`). The
    same-route-state and holder-limit tests are made at the release.
  - *A point delete* (claim 16e): `apply_delete` reports `RouteNote::Selected(next)`, the point
    after the deleted point's first place in its longest holder's route.
  - `point_for` (intake 54's interim Q-2166 pick) is gone.
- **Client** (`crates/app/src/game/session.rs`, `play.rs`):
  - `pointer_down` makes the press test, `pointer_up` the release (`route_release`).
  - A press that takes a point with nothing selected selects it, and the delete-help notice
    shows.
  - Any selection of an object clears the point selection (`route_selection_changed`, after
    pointer releases, group keys and type select).
  - Selecting a holder while the key is held also selects the holders of its last point
    (`Play::add_holders`, from clicks, band boxes and group keys).
  - A right click with nothing selected and a release under 4 pixels on each axis
    (`ROUTE_RIGHT_CLICK_BOUND`) clears the point selection.
  - Route mode ends with the commit at the key's release, a focus loss, the options menu and the
    local house's defeat. After the defeat it cannot be entered again (`RouteMode.ended`).
  - Route notes and the delete's `Selected` effect are read after each tick
    (`route_after_tick`).
  - The candidate's marker is drawn in a hover ink (ours: the look is not studied).
- **Quirks** (all default to the original): `hidden_point_refuses_click`,
  `click_area_skips_kind_test`, `delete_selects_after_release`, `stacked_point_takes_earliest`,
  `press_copy_ignores_route_state`. Schema regenerated.
- **Formats**: save 89 to 90 (`Route.loop_to`, the join's place), AI game 51 to 52, wire 10 to 11
  (`Order::RouteJoin` names its point by place).

## Claims

| claim | status | tests or reason |
|---|---|---|
| S-0213 claim 16b, Click areas: a 16 by 16 pixel square around the place's screen position; the cell plays no part | tested | `claim_16b_the_click_area_is_a_16_pixel_square` (`claims_s0213_routes.rs`); through the client, `route_mode_presses_select_join_and_delete_points` (8 pixels right is outside, 7 right and 7 down inside) |
| S-0213 claim 16b, Click areas: the earliest-made drawn point is the candidate; it is taken only when it is the player's own point and accepted; otherwise the press is tested against the earliest-made point of any house and refused with the invalid-point message | tested | `claim_16b_a_hidden_point_refuses_a_press_both_ways`, `claim_16b_and_16d_click_tests` |
| S-0213 claim 16b, the orders of a press name the point by place and act on the house's earliest-made point there; none left there, the order does nothing | tested | `claim_16b_a_press_order_acts_on_the_earliest_point_at_its_place` |
| S-0213 claim 16b, Two points at one place (Q-2166): the earliest-made is the candidate whatever the routes hold; loop close, join, refusal and selection for it | tested | `claim_16b_two_points_at_one_place_both_ways` |
| S-0213 claim 16b, Who gets a press's orders: the selection and the holders of each selected route's last point, by when given the point; a holder that is not selected closes a loop or gets the copy with no point-click test | tested | `claim_16b_a_press_orders_the_holders_of_the_last_point` (O8 w (8)) |
| S-0213 claim 16b, a loop close at the route's last point or the one before it, and over an existing loop, for a holder that is not selected | tested | `claim_16b_a_press_orders_the_holders_of_the_last_point`, `claim_16b_a_press_copy_ignores_route_state_both_ways` |
| S-0213 claim 16b, the copy for a holder that is not selected: after its own points, past 16, the copy's loop replacing its own, beginning at its own point when that holds the copied loop's first point; a route over 16 points takes a further point add | tested | `claim_16b_a_press_copy_ignores_route_state_both_ways` (a 19th point add accepted) |
| S-0213 claim 16b, how a holder goes round a loop that ends before its route's last point | partly | design, Q-2191: the issued point goes back to the loop's end, and the points after it are never issued; checked in `claim_16b_a_press_copy_ignores_route_state_both_ways`. *Replaced by S-0213 r84 claim 16c, A loop that ends early ([intake 63](intake63-S-0213-S-0301.md)): every point from the loop's first to the route's end is issued in turn; `Route.loop_len` counts only for deletes and joins* |
| S-0213 claim 16b, A point that a route holds twice: first places for the last point, the loop close, the copy, the recorded action and the delete's next point | partly | `Route::first_place` and its users follow the rule; part w (8) holds each joined point once, and the spec rates every case unmeasured. The O8 w (8) route (P2 and P3 twice) is built by `claim_16b_a_press_orders_the_holders_of_the_last_point`, with no press on a doubled point |
| S-0213 claim 16b, a detonate click adds no point, meets the release-time tests and sends the detonate at the release | tested | `claim_16b_a_detonate_click_sends_the_detonate_order` (`claims_s0213_route_detonate.rs`: the order, and the holder-limit refusal) |
| S-0213 claim 16b, the patrol action is an ordinary point add, given only outside every click area or inside one the press neither takes nor refuses (Q-2138 question 2) | seam | patrol processing is not built (S-0215), so S-0214 row GC3 still gives area guard; `targeting.rs` notes it |
| S-0213 claim 16b and 16d, what an aircraft or building does after a join or an ordinary click inside a click area | partly | design, Q-2192: the join is applied as for a vehicle; a point add for it is dropped; a non-behaviour order other than a detonate is not sent. Not tested beyond the join (`route_mode_tests_kind_at_the_press`). *Replaced by S-0213 r84 claims 16b and 16d ([intake 63](intake63-S-0213-S-0301.md)): aircraft and buildings are ordinary holders; an aircraft issues at a flight's end, a non-SAM building at an attack with no target, a SAM building never; a building's attack click adds a point and its rally click is sent directly. A building's other object actions stay open (Q-2227)* |
| S-0213 claim 16d, When the tests are made: the kind test at the press, for every battlefield press outside every click area, whatever the click would do; a refused press ends the click (selection kept, no order, point or band box); none with nothing selected | tested | `route_mode_tests_kind_at_the_press` (client, a POWR building), `claim_16d_a_click_area_skips_the_kind_test_both_ways` |
| S-0213 claim 16d, a press inside a click area is not tested for kind; a press that takes a point meets only the point-click test | tested | `claim_16d_a_click_area_skips_the_kind_test_both_ways`, `route_mode_tests_kind_at_the_press` (the building joins) |
| S-0213 claim 16d, the same-route-state and holder-limit tests at the release, except for selecting clicks and tools; the incompatible-selection message alone when both fail | tested | `claim_16b_and_16d_click_tests`, `route_mode_refusals_share_one_cue_per_tick` |
| S-0213 claim 16d, which points count as inside a click area for the kind test; right presses | partly | design, Q-2193: the press's own tested set; left presses only. *Confirmed by S-0213 r84 claim 16d, When the tests are made ([intake 63](intake63-S-0213-S-0301.md)): every house's click areas, only drawn ones with `quirks.hidden_point_refuses_click` off; right presses never tested. Now tested both ways (`claim_16d_a_hidden_points_click_area_and_the_kind_test_both_ways`)* |
| S-0213 claim 16e, with nothing selected a press in a click area selects the point; the delete key sends the delete whenever a point is selected; the applied delete selects the next point of the longest holder's route; any selection of an object and the key's release clear the point selection | tested | `claim_16e_a_point_delete_selects_the_next_point`, `route_mode_presses_select_join_and_delete_points` |
| S-0213 claim 16e and Edge cases, `quirks.delete_selects_after_release` | tested | `a_point_delete_after_the_release_selects_the_next_point_both_ways` (client) |
| S-0213 claim 16h, Selection: selecting a holder while the key is held selects the holders of its last point, right after it, in the order given that point, adding none of their own | tested | `selecting_a_holder_in_route_mode_selects_its_last_points_holders` (client: a click and a group key, with a third holder that tells one round from several). The order of addition is `unmeasured` in the spec |
| S-0213 claim 16h, Tools: a press with a tool active first meets route mode's press rules, then the tool acts, sent at once with no release-time tests | partly | `route_release` returns to the tool path for an ordinary press, and a taken point does that alone; no test drives a tool in route mode |
| S-0213 claim 16h, Right clicks: a right click acts as outside route mode and never ends it; under 4 pixels with nothing selected clears the point selection, 4 pixels keeps it | tested | `route_mode_right_clicks_and_its_end` |
| S-0213 claim 16h, End of route mode: the key's release, the focus loss, the options menu (design) and the defeat, after which it cannot be entered again | tested | `route_mode_right_clicks_and_its_end` (the options menu), `route_mode_ends_at_the_defeat`, `route_mode_adds_points_and_commits_on_release` (the release and the focus loss) |
| S-0213 claim 16h, the command bar's route-mode button | seam | no command bar is built; the spec leaves the button open (Open questions) |
| S-0213 claim 16g, the candidate's highlight | partly | drawn in our hover ink (`ROUTE_HOVER_INK`); the look is not studied |
| S-0213 claim 16b and S-0701, the accepted point add's sound | seam | the audio side ignores `Effect::Route`; the refused add's cue is as in intake 50 |
| S-0213 Edge cases, `quirks.hidden_point_refuses_click` | tested | `claim_16b_a_hidden_point_refuses_a_press_both_ways` |
| S-0213 Edge cases, `quirks.click_area_skips_kind_test` | tested | `claim_16d_a_click_area_skips_the_kind_test_both_ways` |
| S-0213 Edge cases, `quirks.stacked_point_takes_earliest` | tested | `claim_16b_two_points_at_one_place_both_ways`, `claim_16b_a_press_order_acts_on_the_earliest_point_at_its_place` |
| S-0213 Edge cases, `quirks.press_copy_ignores_route_state` | tested | `claim_16b_a_press_copy_ignores_route_state_both_ways` |
| Q-2138, Q-2166 | answered | by the claims above; intake 50's and intake 54's rows updated |
| Q-2139 | answered in part | by claim 16h above; the command bar's button is still open. S-0213 r84 ([intake 63](intake63-S-0213-S-0301.md)) adds that a right press is never tested for kind; the button stays open |

## C7 judgement

None of the five quirks goes into `data/layers/play-fixes.jsonc`. Each changes what a route-mode
press does in a case no play note or request names as a bug, and no unit is stuck in either
position, so condition 2 fails for all five:

- `hidden_point_refuses_click`: a press can be refused by a point the player cannot see. That is
  odd, but the player can press beside it, and nothing is stuck.
- `click_area_skips_kind_test`: on, an aircraft or building can join a point. That is what the
  original allows, and no note names it as a bug.
- `delete_selects_after_release`: whether a point stays selected after a late delete. An
  interface choice.
- `stacked_point_takes_earliest`: which of two points at one place a press takes. An interface
  choice; the player can still add a point at the place.
- `press_copy_ignores_route_state`: lets a holder that is not selected pass 16 points or loop
  inside its own points. No note names it, and the holder still runs its route.

## Seams

- Patrol on a route-point cell (S-0214 GC3): patrol processing is not built (S-0215).
- The command bar's route-mode button: no command bar, and the spec leaves the button open.
- The accepted point add's sound: the audio side ignores `Effect::Route`.
- A tool click in route mode: built, not tested through the session.
- The order in which the holders of a last point are added to the selection: built as claim 16h
  gives it, `unmeasured` in the spec.
- The highlight's look (claim 16g).

## Requests

- Q-2191 (how a holder goes round a loop that ends before its route's last point).
- Q-2192 (what an aircraft or building gets from a press inside a click area).
- Q-2193 (which points count as inside a click area for the kind test, and whether right
  presses meet it).

Q-2191 and Q-2193 are answered, and Q-2192 in part (question 3's other building actions are
open), by S-0213 r84 ([intake 63](intake63-S-0213-S-0301.md)).

Q-2194 and Q-2195 are unused.

## Lanes

Before: origin/main 6339036 (the same simulation code as 767a5ce). After: this branch.
Outputs: `local-lane/m8/a3-56/before` and `local-lane/m8/a3-56/after`.

- **Goldens**: regenerated with `GLASNOST_UPDATE_GOLDEN=1`. No golden file moved: the route
  changes act only on route-mode input, which no golden scenario gives. `cargo xtask gate` matches
  every golden on native and wasm32.
- **Seed sweeps** (M6c, M7a, M7b, seeds 1 to 16): identical to before, line for line. Min and
  median per counter (min/median):
  - **m6c**: garrison orders (S-0508) 15/30; garrison occupants inside (S-0508) 5/9; engineer orders (S-0509) 6/6; buildings captured (S-0509) 4/4; spy orders (S-0510) 6/6; infiltrations (S-0510) 6/6; tech building orders (S-0515) 4/4; tech building income cycles (S-0515) 2346/2346; jumpjet orders (S-0502) 34/60; jumpjet flight (S-0502) 9108/9116; jumpjet units built and flown (S-0417 claim 15) 8/10; deploy orders (S-0517) 57/125; changes of deployed form (S-0517) 4/6; radiation exits (S-0525) 6/25; radiation in the field (S-0525) 6387/7279; chrono erasure orders (S-0514) 17/31; erasures begun (S-0514) 5/9; prism buildings ordered (S-0518) 10/11; charger orders (S-0518) 13/18; support beams (S-0518) 88/118; bomb planter orders (S-0519) 30/47; bombs planted (S-0519) 8/15; disarm orders (S-0519) 6/9; suicide orders (S-0519) 11/19; sabotage orders (S-0519) 7/19; air-strike orders (S-0519) 2/13; air strikes called (S-0519) 8/17; crate orders (S-0520) 77/127; crates picked up (S-0520) 10/15
  - **m7a**: berserk weapon orders (S-0610) 16/38; units made berserk (S-0610 claim 4) 18/40; psychic sensor orders (S-0610 claim 9) 1/2; hostile intent seen by a sensor (S-0610 claims 9 to 11) 0/4781; psychic reveal orders (S-0610 claim 13) 6/9; psychic reveals fired (S-0610 claims 13 and 14) 6/9; slave-miner buildings placed or deployed (S-0402 claim 14) 13/13; slaves at work (S-0402 claim 14) 7178/7503; gattling stage rises (S-0609 claim 2) 15/28; gattling at its top stage (S-0609 claim 2) 1941/5132; takeover orders (S-0601) 4/15; takeovers (S-0601 claim 6) 6/24; gas clouds (S-0613 claims 2 to 4) 98/322; gas hits (S-0613 claim 13) 271/1193; absorber entry orders (S-0605) 2/7; infantry absorbed (S-0605 claims 1 and 7) 0/3; grinder entry orders (S-0603) 7/10; units ground (S-0603 claims 5 and 8) 6/9; tank bunker build orders (S-0606) 2/4; bunker entry orders (S-0606 claim 3) 5/12; vehicles bunkered (S-0606 claim 7) 5/11; bunker exit orders (S-0606 claim 12) 4/10; lifter build orders (S-0607) 2/5; lift orders (S-0607) 3/12; vehicles seized (S-0607 claim 5) 6/38; seized units dropped (S-0607 claim 13) 5/28
  - **m7b**: drain orders (S-0608) 3/3; drain links formed (S-0608 claims 2 and 5) 3/3; credits drained (S-0608 claims 7 to 10) 7240/7440; drained-building ticks (S-0608 claim 1) 19182/19386; mutator launches (S-0611 claim 2) 3/6; infantry mutated (S-0611 claims 9 to 12) 7/20; dominator launches (S-0612 claim 1) 6/7; objects taken over (S-0612 claim 14) 10/17; berserk weapon orders (S-0610) 4/15; units made berserk (S-0610 claim 4) 18/22; psychic sensor orders (S-0610 claim 9) 1/1; hostile intent seen by a sensor (S-0610 claims 9 to 11) 0/0; psychic reveal orders (S-0610 claim 13) 0/1; psychic reveals fired (S-0610 claims 13 and 14) 0/1; slave-miner buildings placed or deployed (S-0402 claim 14) 4/12; slaves at work (S-0402 claim 14) 3163/3643; gattling stage rises (S-0609 claim 2) 0/0; gattling at its top stage (S-0609 claim 2) 0/0; takeover orders (S-0601) 0/1; takeovers (S-0601 claim 6) 0/2; gas clouds (S-0613 claims 2 to 4) 0/0; gas hits (S-0613 claim 13) 0/0; absorber entry orders (S-0605) 10/18; infantry absorbed (S-0605 claims 1 and 7) 7/10; grinder entry orders (S-0603) 3/6; units ground (S-0603 claims 5 and 8) 2/5; tank bunker build orders (S-0606) 4/4; bunker entry orders (S-0606 claim 3) 0/0; vehicles bunkered (S-0606 claim 7) 0/0; bunker exit orders (S-0606 claim 12) 0/0; lifter build orders (S-0607) 1/1; lift orders (S-0607) 0/0; vehicles seized (S-0607 claim 5) 0/0; seized units dropped (S-0607 claim 13) 0/0
- **Arena** (`glasnost-headless arena --record all`): `ours-M-0001.csv`, `ours-M-0002.csv` and
  `ours-M-0003.csv` are byte-identical to before. Scored as intake 47 / A6 score it:
  - M-0001: 11 of 96 scored pairings fail (3 the win test, 9 a side outcome); 161 of 171
    side-wise unanimous outcomes reproduced.
  - M-0002: 1 of 24 fails (a side outcome); 46 of 47 reproduced.
  - M-0003: 2 of 35 fail (side outcomes: MTNK against APOC, ROBO against APOC); 62 of 64
    reproduced.
- **`glasnost-ai --include-ignored`**: 239 passed, 0 failed, the same 31 binaries and counts as
  before.
- **AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, side pairs 0,2 and 1,2): the same as before
  but for timings. All 24 games end, with the same lengths and winners (house 1 wins 16, house 0
  wins 8).
