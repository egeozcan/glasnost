# Own AI: issues from the lead's audit

The lead's static audit of our own AI (`crates/deterministic/ai`) filed its findings as GitHub
issues. This ledger holds one row per issue as it is taken: whether a test confirmed it, what was
changed and where the test lives. Every rule added is our own design (`// design: own AI (#N)`,
docs/dev/own-ai.md): the AI gives only orders a player could give and sees only its own shroud.

## Batch 1

Issues #2 to #10 (correctness and livelocks). AI game format 98. Performance items (#52, #59) are
batch 2.

| issue | confirmed | change | test |
|---|---|---|---|
| #3 gunner load slice | yes: a healer with two earlier orders panicked (slice start past the end); with one, the load was not noted | the special orders' start moves back by the healer's earlier orders removed before it (`Special::gunners`) | `tests/gunner.rs`: `a_healer_with_two_earlier_orders_is_loaded`, `a_load_after_an_earlier_order_is_noted` |
| #2 stale teleport source | yes: the second stage fired again on a consumed source, and the first stage never picked again | a fresh (unconsumed, S-0512 claim 10) source that still holds the group is sent; otherwise the first stage picks again (`Brain::teleport_cells`) | `tests/ai_sw.rs`: `fires_the_second_stage_once_per_pick`, `picks_again_when_the_group_has_left_the_source` |
| #4 intruder out of reach | yes: 90 attack orders in 90 ticks at an enemy on an island | intruders out of the unit's reach region and weapon range are skipped (`Script::reaches`); intruder orders go through the order memory as raider orders do | `tests/livelocks.rs`: `an_intruder_out_of_reach_is_not_attacked_every_tick` |
| #5 deploy spot across water | yes: the same move across water, repeated | both spirals keep to the unit's reach region; a unit driving to a spot that still takes it is left to arrive; an expansion with no reachable cell by the far field deploys as a spare yard | `tests/livelocks.rs`: `a_yard_vehicle_is_not_sent_across_water_to_deploy` |
| #6 type at its build limit | yes: a build order the sim refused for a type at its limit (1 in the test's game) | `Script::candidates` leaves out a type at its limit (S-0407 claims 9 and 13) | `tests/ai.rs`: `a_type_at_its_build_limit_is_not_ordered` |
| #7 walk plant between thinks | yes: placed in the spiral, south of the base's northern row | one placement choice for think ticks and the ticks between (`Script::done_cell`) | `tests/water.rs`: `a_walk_plant_done_between_thinks_lands_toward_the_water` |
| #8 grinder on a busy line | partly: the lead measured refused grinder orders in `fixture_m7a` and `fixture_m7b`; our test scene does not reproduce a refusal | `grinder::build_type` needs a free line; the brain's special buildings replace only the script's own tech-climb order (`Script::climbed`), not a resume or another helper's order | `tests/grinder.rs`: `no_grinder_order_while_the_building_line_is_busy` (a guard; it passes on the old code too) |
| #9 naval yard as the ferry goal | yes: a walker that could walk to the enemy base was sent home for a ferry | the ferry's goal and the hold-home goal are chosen per movement zone (`Script::base_goal_for`); a full ferry at home with no drop cell lets its passengers out, and boards none without one | `tests/livelocks.rs`: `a_naval_yard_does_not_keep_the_infantry_home` |
| #10 unreachable search cell | yes: the search cell was in the middle of a lake | the wave's search cell lies in its lead unit's reach region (`Script::unseen_cell`) | `scripted.rs`: `unseen_cell_tests::the_search_cell_is_one_the_wave_can_reach` |

## Batch 2

Issues #52, #53, #54, #57, #58, #59 and #60 (speed). Every change gives the same answers: no golden moved, AI game format
stays 99, and the M6c, M7a and M7b sweeps (seeds 1 to 16) and the quick lane (seeds 7 to 14,
sides 0,2 and 1,2) match base game for game. Nothing is kept from one tick to the next, and think
ticks are not staggered further. Timing comes from `glasnost-headless ai-bench` (new; release,
install map 9, 8 players, seed 7, 9000 ticks, state hashes equal to base every 500 ticks).

| issue | confirmed | change | test |
|---|---|---|---|
| #52 place_cell dominates think | partly: the steps that search for cells (tech climb, base walk, wanted role) are about a third of think time after the change; the split inside them could not be sampled. The free-gap map saved about 2 % of think time | the free-gap test reads a map of every gap built once per brain call (`Blocked`), not every known object per tried cell; the objects listed once per call (`Script::known_objects`) | `scripted.rs`: `place_cell_tests::the_lists_kept_for_a_think_answer_as_asked_afresh`, `the_kept_and_once_asked_placement_answers_match_the_three_passes` |
| #53 full object scans per brain per tick | yes: own objects, roles, seen enemies and the drain claims were each listed afresh, several times a call | the drain claims are read only when an order moves a drainer (`drain::filter_orders`); own, role, candidate and seen-enemy lists are kept for the brain call. AI time between thinks 107 to 22 µs a house | `drain.rs`: `filter_tests::the_drainer_filter_keeps_the_same_orders_with_or_without_drainers`; the kept-list test |
| #54 superweapon search every tick | yes, but small: 0.8 % of the AI time between thinks after the change | the search reads the call's kept seen-enemy and own lists; keeping "no target" across ticks would not give the same answers, so it is not kept | the kept-list test; sweeps and quick lane unchanged |
| #57 failing placement spirals, free_gap per cell | yes | free_gap answers from `Blocked` during a think (falls back to the object list with an object left out or outside the map's square); the spirals are not kept across ticks (not exact) | the kept-list test (every gap, with and without an object left out) |
| #58 whole-grid scans within a think | partly: the far-field search was 15 % of think time | the far field is looked for only when an idle yard vehicle or the expansion tests reach it. A dense map for the field joining was tried and gained about 1 %; left out | sweeps and quick lane unchanged |
| #59 same world queries per think; all houses think on one tick | yes for the queries; the stagger is on main since 8734871 | kept lists as #53. With the stagger the AI takes 2.0 ms a tick on mean (all houses); were every house to think on one tick, that tick would take 26.8 ms on mean (base: 3.4 and 38.9 ms) | the kept-list test |
| #60 tech::buildable polled per type | yes | the superweapon choice, the economy resume and the ferry type read the think's kept tech answers; the broke test (every type's tech) runs only after the patience and refund tests | the kept-list test; `the_kept_and_once_asked_placement_answers_match_the_three_passes` |

## Batch 3

Issues #40, #11, #12, #13, #14, #17, #19, #27, #28 and #29 (play logic). AI game format 101. Each
issue was confirmed on main with a test that failed there, committed before its fix. The changes
alter play, so the quick lane's winners move (seeds 7 to 14, sides 0,2 and 1,2, 48 games): Yuri
wins 23 (base 21), Allied 12 of 24 (base 13), Soviet 12 of 24 (base 14); one game (Soviet against
Yuri, map 8, seed 12) reaches 120 000 ticks with no winner. It is a stalemate: the Soviet house has
lost its yard and barracks, runs on low power and sends small groups of vehicles from its one war
factory at Yuri's base, where the dominator takes them; Yuri's house is broke (under 10 credits
from tick 110 000) and cannot build an army. Where a game's winner changed, the games part at an
order of a fix: in Allied against Yuri, map 8, seed 7, the first differing tick is 14 633, where an
idle Yuri vehicle is no longer sent at an Allied aircraft it can neither fire at nor reach (#28).

| issue | confirmed | change | test |
|---|---|---|---|
| #40 unplaceable building or role blocks building | yes: a done building with no cell held the line and no other building was ordered | every done building is tried for a cell, one placed a think; one that no cell takes for 900 ticks is cancelled (S-0404 claim 6); the base role and the tech climb skip types with no cell (`Script::place_done`, `base_role`, `climb`) | `tests/unplaceable.rs`: `a_done_building_with_no_cell_gives_way`, `a_done_defence_is_placed_beside_a_stuck_building`, `an_infantry_factory_with_no_cell_does_not_hold_up_the_war_factory` |
| #11 parked units keep the wave threshold met | yes: two ships parked at the wave's goal made up the wave with one tank, and were sent to where they stood | units already at their goal do not count toward a wave nor get its order; the goal rotates only when a wave goes | `tests/ai.rs`: `parked_ships_do_not_make_up_a_wave` |
| #12 home falls back to the oldest object or a captured yard | yes: with the own yard lost, home moved to the first object (a far tank) | home is kept as a cell: it stays while no yard exists or a yard stands on it, else moves to the yard nearest it; with none kept, the first yard, then the first building, then the first object (`Script::keep_home`) | `tests/home.rs`: `a_captured_yard_does_not_move_home`, `a_lost_yard_does_not_move_home_to_a_far_unit` |
| #13 lifters march with waves | yes: a lifter counted toward a wave and took its attack-move | a lifter is not counted toward a wave, and a wave's attack-move to a lifter is dropped (`lifter::keeps_order`) | `tests/lifter.rs`: `a_lifter_does_not_go_with_a_wave` |
| #14 charger cap never limits, vehicle chargers sent | yes: more than three chargers were sent at one building, and a vehicle charger was sent | only infantry chargers are sent (S-0518 claim 11); at most three at a building, counting the registered chargers and the own ones already bound for it (`linked::aimed_at`) | `tests/linked.rs`: `sends_no_more_than_three_chargers_at_a_coil`, `chargers_on_their_way_count_toward_the_three`, `a_vehicle_charger_is_not_sent_at_a_coil` |
| #17 deploy-fire vehicle stays parked | yes: a deploy-fire vehicle in its deployed state stayed there with no enemy near | with no enemy within reach it is ordered to guard its own cell, which lets it leave the deployed state (S-0213 table T2, S-0517 claim 2) | `tests/deployers.rs`: `releases_a_deploy_fire_vehicle_when_the_enemy_is_gone` |
| #19 memory records orders never issued | yes: an order the pending limit dropped was noted as given | the memory notes only the offered orders that are their actor's last order in the brain's output and inside the pending limit (`Memory::note_given`). The use counters are still counted when an order is offered | `tests/special.rs`: `an_order_dropped_by_the_pending_limit_is_not_given_up` |
| #27 wave targets include allied, unoccupied and own starts | partly: allies were already left out (`ally_base_at`); the own start was a target when the base unit stood more than 5 cells off it | the one start nearest home is left out, however far. Unoccupied starts stay targets: a player does not know a start is empty until scouted | `scripted.rs`: `start_tests::the_own_start_is_no_target_when_the_base_unit_stands_off_it` |
| #28 ENGAGE has no reach test | yes: an idle unit was sent at an enemy it could neither fire at nor reach, every think | an idle unit engages an enemy within ENGAGE only if it can fire at it from where it stands or reaches it (S-0215 claim 37 (b), `Script::reaches`) | `tests/livelocks.rs`: `an_idle_unit_is_not_sent_at_an_enemy_it_cannot_reach` |
| #29 stranded infantry re-ordered every think | yes: a rifleman on an islet was sent to its own cell every think | a unit that stands at the cell nearest home it can reach is not sent there again; the nearest cells are kept per region for the think | `tests/livelocks.rs`: `a_stranded_rifleman_is_not_sent_to_where_it_stands` |

## Batch 4

Issues #15, #16, #18, #20, #21, #22, #23, #24, #25 and #26, and the anti-air repeat seen in batch 3's
lane (play logic). AI game format 103. Each issue was confirmed on main with a test that failed there,
committed before its fix, except #20, which main had already fixed: its test passes on main and is
kept. The rest of #19 (the use counters) is not in this batch. The changes alter play, so the quick
lane's winners move against batch 3's after lane (seeds 7 to 14, sides 0,2 and 1,2, 48 games; 15
winners changed): Yuri wins 24 (before 23), Allied 12 of 24 (12), Soviet 12 of 24 (12). Every game
has a winner: Soviet against Yuri, map 8, seed 12, which reached 120 000 ticks with none, now ends at
114 701 with Yuri's win. Where a game's winner changed, the games part at an order of a fix: in
Allied against Yuri, map 8, seed 7, the first differing tick is 12 008, where Yuri's absorber takes
in infantry it held back before (#15). In the M7a sweep, bunker entries fall from a median of 14 to 4
and exits from 12 to 4 (#18: no vehicle goes in to be let out at the next think).

| issue | confirmed | change | test |
|---|---|---|---|
| #15 absorber reads power through `power::totals` | yes: in a blackout the totals read zero, so the absorber found no shortage and sent in the infantry it keeps back (3 sent, 2 expected) | the absorber reads the shortage as the power bar shows it (`power::short`), which a blackout does not zero | `tests/absorber.rs`: `a_blackout_does_not_send_the_kept_infantry_in` |
| #16 deployers deploy bunkered vehicles | yes: a bunkered deploy-fire vehicle firing from the bunker was given Deploy, which lets it out (S-0606 claim 12, S-0214 row VD5) | the deployers skip a vehicle in a bunker or on its way into one | `tests/bunker.rs`: `a_bunkered_deploy_fire_vehicle_is_not_deployed_out` |
| #18 bunker entry does not mirror the exits | yes: a vehicle was sent into a bunker at the retreat percentage, and into a bunker while a seen enemy stood near the base; both let it out at the next think | no vehicle is sent in while the base is attacked or into a bunker at or below `RETREAT_PERCENT` | `tests/bunker.rs`: `no_vehicle_is_sent_in_to_be_let_out_again` |
| #20 defend sends spent aircraft back | no: fixed on main by 61fc12e (defend leaves an aircraft with no ammunition alone); the test passes on main and is kept | none | `tests/ai.rs`: `a_spent_aircraft_on_its_way_back_is_not_sent_again` |
| #21 ferry boarding mixes heads with sizes, riders taken before the cursor test | yes: three riders of size 2 were clicked aboard a ferry for 4, and a refused rider kept a loaded ferry at home | the room is reckoned in sizes as the transport reckons it (S-0507 claim 1 and F1, the truncation switch read); only infantry the Enter cursor shows on the ferry count as free (claim 4); `FERRY_LOAD` stays a head count | `tests/ai.rs`: `a_refused_rider_does_not_hold_the_ferry_at_home`, `riders_are_clicked_aboard_by_size` |
| #22 engineers repair buildings the house does not own | yes: an engineer was sent into an ally's badly damaged pillbox (the cursor offers the repair there, S-0214 rows I4 to I7) | `worth_repair` holds only for the house's own buildings | `tests/special.rs`: `an_engineer_does_not_repair_an_allys_building` |
| #23 grinder side-step memory under the substituted cell | yes: the failure was kept under the cell the click's search found (S-0214 claim 11), so the blocked side was clicked again | `Memory::ask` keeps the goal a part asked for beside the order it gave; a failure counts against both | `tests/grinder.rs`: `a_far_unit_whose_side_click_is_moved_does_not_repeat_it` |
| #24 lifter lets go early without looking below | yes: a held vehicle near its lifter was let go over the house's own infantry, which the drop removes (S-0607 claims 13 and 16) | the early let-go needs open dry land with nothing else on the cell (`lifter::clear_below`) | `tests/lifter.rs`: `a_held_vehicle_is_not_let_go_over_own_infantry` |
| #25 gunner pairs not checked against the Enter cursor | yes: a healer too large for the vehicle was picked and the one that fits was not sent | the nearest healer whose Enter cursor holds on the vehicle is picked | `tests/gunner.rs`: `a_healer_the_vehicle_refuses_is_not_picked` |
| #26 terror units have no claim tracking | yes: two idle planters (or disarmers) in one think went for the same target, and a planter went for the enemy another was already sent at | targets another own unit of the role is sent at, in this think or before, are passed over | `tests/terror.rs`: `two_planters_take_two_targets`, `a_planter_leaves_a_claimed_enemy_to_its_planter`, `two_disarmers_take_two_bombs` |
| anti-air repeat (no issue number) | yes: Allied against Yuri, map 2, seed 7, house 0 ordered Infantry type 3 (an engineer type with an anti-air weapon) at every think from tick 45 000 while its superweapon building was short of money; the think's non-fighter filter dropped the order and nothing was built | the anti-air part picks only among the types that filter keeps (`antiair::build_type_where`) | `tests/antiair.rs`: `no_anti_air_non_fighter_is_picked_while_saving_for_a_superweapon` |
| #19 (rest) counters count offered orders | not started in this batch | — | — |

## Batch 5

Issues #426, #36, #37, #38, #39, #47, #45, #30, #31, #32, #33 and #427 (sight, targets and one job a
unit). AI game format 110. Each issue was confirmed on main with a test that failed there, committed
before its fix, except the deploy case of #47: its test passes on main and is kept. #427 follows the
lead's decision in the batch brief. The lanes were run on main 946161b and on the batch rebased onto
it. The quick lane's winners move (seeds 7 to 14, sides 0,2 and 1,2, 48 games; 12 winners changed):
Yuri wins 26 (before 22), Allied 12 of 24 (12), Soviet 10 of 24 (14). Every game has a winner. Where a
winner changed, the games part at an order of a fix: in Soviet against Yuri, map 2, seed 9, they part
at tick 4 096, where a Soviet infantry unit just out of its factory keeps its garrison order; before,
the crate module sent it to a crate in the same think (#47). That game was Soviet's at 59 711 and is
now Yuri's at 62 695. In the M7a sweep (seeds 1 to 16), bunker entries go from min/median/max 1/4/7
to 3/5/7 and exits from 1/4/6 to 3/4/6 (#427); the smallest counter over the sweep rises from 1 to 2,
and the m7a gate passes. In the M6c sweep, infiltrations (S-0510) fall from 6/6/6 to 0/1/2, with none
at seeds 5, 9, 11, 15 and 16. The fall comes with #39 (the sweep at #38's fix still gives 6/6/6):
the fixture's spyable buildings are neutral power buildings, which are no longer a spy's targets. The
fixture needs a spyable building of an enemy player within the spies' reach; that is left to a later
change.

| issue | confirmed | change | test |
|---|---|---|---|
| #426 build orders the sidebar refuses are given again at every think | yes: with the house's one docking place taken by its jet, a jet build order the sidebar refuses was given at every think (60 refused orders in 900 ticks); the repeat came from the script's candidate filter, which asked only the build limit | the special builders' startable test also asks `sidebar::legal` (offered, the limit, the pad total of a dock-bound aircraft, a busy building line; S-0411 claim 4, S-0407 claims 13 and 16); the script's candidate filter asks `tech::refused_at_issue` | `tests/ai.rs`: `no_build_order_is_refused_at_issue` |
| #36 special abilities see cloaked buildings and read the victim's credits | yes: an engineer was sent at a cloaked enemy building and ordered for a cloaked tech building, a spy was sent into a cloaked building, and the spy's cash rank read what the victim holds | a new `in_view` asks what a person in the house's seat sees: on the map, on an explored cell and, for an object neither the house nor an ally owns, not hidden by a cloak the house does not sense (S-0516 Terms and claim 28). The special modules' sight tests and the tech-building tests use it; the spy's cash rank no longer reads the victim's credits | `tests/special.rs`: `an_engineer_does_not_go_for_a_cloaked_building`, `no_engineer_is_ordered_for_a_cloaked_tech_building`, `a_spy_does_not_go_for_a_cloaked_building`, `a_spy_goes_for_cash_whatever_the_victim_holds` |
| #37 seen enemies ignore disguise | yes: an enemy tank disguised as one of the house's own was attacked, and an enemy building whose top-left cell alone is shrouded was not seen | `seen_enemies` asks `in_view`, which sees a building by any explored cell it stands on (S-0406 claim 2), and leaves out an enemy unit the house is shown as its disguise (S-0516 claims 35 and 37: no fake blink or detector shows the real unit) | `tests/ai.rs`: `a_disguised_enemy_is_not_attacked`; `tests/special.rs`: `a_building_is_seen_by_any_of_its_cells` |
| #38 superweapon and ferry cell choosers read objects the house cannot see | yes: a curtained group was picked as the teleport's source; a cloaked enemy tank or an enemy building beside a group stopped the pick; a cloaked tank on the landing block moved it | the teleport's source test skips a protected vehicle (S-0513 claim 34) and is held back only by another house's infantry or vehicle the house sees; the curtain, landing, drop and ferry land cells count only objects the house sees (`Brain::seen_taken`); the house's own and allied objects are seen wherever they stand | `tests/ai_sw.rs`: `does_not_teleport_a_curtained_group`, `a_hidden_tank_or_a_building_beside_a_group_does_not_stop_the_teleport`, `a_hidden_tank_does_not_move_the_landing` |
| #39 spy targets not limited to enemy players | yes: an idle spy was sent into a neutral house's power plant | the spy's targets take the purchase test's owner rule (`Special::enemy_player`: a player house that is neither the house nor an ally) | `tests/special.rs`: `a_spy_does_not_go_into_a_passive_houses_building` |
| #47 one unit gets two jobs in a think | partly: a spy sent into an enemy building was also sent to a crate beside it (two orders; the later won). The deploy case does not reproduce: the brain's deploying filter (M8 B7) already drops a behaviour order for a unit given the deploy order; its test passes on main and is kept | after each special module, the actors of the orders given so far leave the own-unit list the later modules see, so a unit gets one job a think | `tests/special.rs`: `a_spy_sent_into_a_building_is_not_sent_to_a_crate`; `tests/deployers.rs`: `a_deployed_vehicle_gets_no_second_order` |
| #45 two lifters go for one vehicle | yes: a lifter earlier in the think picked the vehicle a later lifter already aimed at | every aiming lifter's target is taken before any lifter picks | `tests/lifter.rs`: `two_lifters_do_not_go_for_one_vehicle` |
| #30 strike, landing and drop cells treat a building as its top-left cell | yes: a building reaching into the landing did not move it, and a storm was aimed beside a cell of an own building other than its top-left one | a shared `cells_of` gives a building's whole foundation; the landing and drop cells' taken cells and the strike cell's own-object test use it | `tests/ai_sw.rs`: `a_building_reaching_into_the_landing_moves_it`, `no_storm_beside_any_cell_of_an_own_building` |
| #31 landing and spy plane aim at the nearest seen enemy | yes: an enemy scout at home pulled the teleport's landing, and drew the spy plane to a cell the house already sees | the landing goes by an enemy base (`base_goal`); the spy plane takes the psychic reveal's cell, which the house does not see | `tests/ai_sw.rs`: `a_scout_at_home_does_not_pull_the_landing`, `a_scout_at_home_does_not_draw_the_spy_plane` |
| #32 strike cell ignores allied objects | yes: a storm was aimed beside an ally's unit | the strike cell keeps every cell of the house's and its allies' objects out of the radius | `tests/ai_sw.rs`: `no_storm_beside_an_allys_unit` |
| #33 dominator aim ignores damage to allies | yes: the dominator fired where its damage destroys an ally's units | the area damage to the house's and its allies' objects counts as loss; allied objects the sweep takes are counted whole, once | `tests/dominator.rs`: `holds_fire_where_the_damage_destroys_an_allys_units` |
| #427 bunker exit and entry read any enemy near the base | yes: an enemy by another building far from the bunker let the vehicle out and held entries back; an enemy aiming at the bunker from beyond the reach did not let it out | the lead's decision: a bunker is under threat when a seen enemy stands within `ATTACK_REACH` of one of its cells or aims at it; exit and entry read that per bunker, and the retreat-health condition stays | `tests/bunker.rs`: `an_enemy_far_from_the_bunker_neither_lets_out_nor_holds_back`, `an_enemy_aiming_at_the_bunker_lets_the_vehicle_out` |

## Batch 6

The M6c fixture (batch 5's follow-up) and issues #42, #43, #44, #35, #46, #34, #41, #49, #50, #48 and
#51 (claims, sight, the income watch, the end of a game). AI game format 112 (the income watch's two
new fields; postcard is positional, so their serde defaults keep no older save readable). Each issue
was confirmed on main 5dcbc5c with a test that failed there, committed before its fix; the radiation
case of #48 has a test, and its chrono, drain and lifter cases are doc changes. #19's rest is left
for a later batch. The lanes were run on main 5dcbc5c and on the batch. The quick lane (seeds 7 to
14, sides 0,2 and 1,2, 48 games) keeps every Allied v Yuri winner; 7 of the 24 Soviet v Yuri games
change: Soviet 11 (before 12), Yuri 12 (12), and one game (map 2, seed 8) has no winner at 120 000
ticks, both houses with large armies and spent credits. In Soviet v Yuri, map 0, seed 7, the games
part at tick 29 063: Yuri's lifter holds a Soviet vehicle over a water cell, nothing else on it; before,
it dropped the vehicle there; now the cell is no killing cell for that vehicle (a bridge deck, or water
its class may enter: #46), and the lifter keeps it. That game was Yuri's at 60 314 and is Soviet's at
59 727. In the M6c sweep (seeds 1 to 16), infiltrations (S-0510) rise from 0/1/2 to 1/4/5, with none
at zero; the same sweep with the batch's fixes but not the fixture gives the base's lines seed for
seed. With spies that reach the enemy base, seeds 6, 8 and 14 end before 12 000 ticks (8 683, 5 942
and 9 669), and seed 8's short game has no charger order and no support beam; seed 15, which ended at
9 594 with zeros before, runs to 12 000 with every counter at 4 or more. The smallest counter over the
sweep stays 0, now at seed 8; the m6c gate's seed 3 has every counter at 4 or more. In the M7b sweep,
recycled units fall from 6/9/11 to 4/4/5 and absorbed infantry from 7/8/10 to 5/7/10, as dominated
units now count as the house's own (#35); drain counters stay at 3 or more. The M7a sweep keeps its
smallest counter (0, the bunker counters of seed 13, before and after).

| issue | confirmed | change | test |
|---|---|---|---|
| M6c spies find no enemy building (from batch 5, #39) | yes: the sweep on main 5dcbc5c counted no infiltration at seeds 5, 7, 9, 10, 11, 13, 14 and 15 (min/median/max 0/1/2) | the fixture's spy type (SPYU) wears the HULK's armour class, which only the temporal, bomb-planting and disarming warheads affect (S-0304 claim 1), so the armies let the spies cross to the enemy player's base. A pre-placed enemy building near the spies was not an option: in a skirmish a pre-placed building goes to the first computer house only, and to no human house (S-0009 claim 27), so the two sides could not each get one | `sim-gate/tests/m6c.rs`: `a_spy_infiltrates_in_the_seeds_main_found_none`; the M6c sweep |
| #42 controllers aimed at one enemy across thinks | yes: a second idle controller was sent at the enemy a first controller was already on its way to (sent the think before) | the aimed list starts with the targets of the house's controllers in *attack* | `tests/mind.rs`: `a_controller_on_its_way_holds_its_enemy_across_thinks` |
| #43 drain claims keyed by building only | yes: another house's drainer on the way to a building hid the house's own claim, and a second own drainer was sent there | the claims are keyed by house and building (`drain::claims`) | `drain.rs`: `filter_tests::another_houses_claim_does_not_hide_an_own_claim` |
| #44 absorber pick and stalled entrants | yes: a switched-off nearest absorber kept the unit from the next one; a stalled entrant sent again was not counted, and one more entrant than the room was sent | the unit goes to the nearest absorber that takes the *enter* click; an entrant sent again takes its building's room | `tests/absorber.rs`: `a_refusing_nearest_absorber_does_not_block_the_others`, `a_resent_stalled_entrant_takes_its_room` |
| #35 dominated units treated as controlled enemies | yes: a permanently controlled infantry unit was sent into the absorber with power to spare and no surplus, and a dominated vehicle was recycled | only a unit a controller holds (`mind::controlled_by`) counts as a controlled enemy in the absorber and grinder steps (`absorber::held_by_controller`); a dominated one is the house's own (S-0601 claim 13) | `tests/absorber.rs`: `a_dominated_infantry_unit_counts_as_own`; `tests/grinder.rs`: `a_dominated_vehicle_is_not_recycled` |
| #46 lifter drops over water any unit, and over hidden enemies | yes: a hovercraft over water and a tank over a bridge deck were dropped; a cloaked enemy below made a killing cell | water kills only a unit whose class cannot enter it, away from a bridge deck (S-0607 claim 17); the enemy below must be in view (`in_view`) | `lifter.rs`: `drop_tests::water_kills_only_what_cannot_stand_on_it`, `drop_tests::a_hidden_enemy_does_not_make_a_killing_cell` |
| #34 mutator blast estimate on a bridge deck | yes: on a high-bridge cell the estimate counted the vehicles beneath the deck and not the ones on it | the blast estimate takes a point and a layer (`dominator::Blast`); the mutator passes its mutation point, lifted onto the deck, and the deck layer on a high-bridge cell (S-0611 F1, S-0303 claim 4) | `tests/mutator.rs`: `the_blast_estimate_on_a_bridge_deck_counts_the_deck_only` |
| #41 expansion spot beyond the field's reach | yes: the yard vehicle was sent to (43,9), 242 squared cells from the field, beyond the 196 it deploys within | the spiral centred on the far field keeps to cells within `FIELD_REACH` of it; with none there, the spot by the unit is taken as before | `tests/expansion.rs`: `the_cell_by_the_far_field_is_within_its_reach` |
| #49 unblocker for a type nothing can unblock | yes: a type with a blocker no building clears got a building to climb for its other blocker | the search returns none when any blocker has no building that clears it | `scripted.rs`: `unblocker_tests::a_type_with_a_blocker_nothing_clears_has_no_unblocker` |
| #50 income watch misses quick items and counts refunds | yes: an item made between two thinks counted as a loss of its price (earned 0, 300 expected), and a sale refund as income (750) | an own object made since the last think, with a start price and off the lines, paid its price; the refund of a building seen in its sale and gone since is taken off. Upgrades sold off and stored resources paid at a sale still count | `scripted.rs`: `income_tests::an_item_made_between_two_thinks_is_no_lost_income`, `income_tests::a_sale_refund_is_no_income` |
| #48 doc and code disagree | yes, by reading; the radiation case by a test: a walker hovering 400 leptons above a field got a move order | radiation: the code follows the doc and S-0525 claim 8 (aircraft hurt too; nothing airborne moved). Chrono, drain and lifter: the doc follows the code | `tests/radiation.rs`: `a_unit_in_the_air_above_a_field_stays` |
| #51 step after game over; clones in save and diagnose | yes: a step after the outcome applied a pending order and changed the hash | `Game::step` does nothing once the game has an outcome; `save` writes from borrows (same bytes); `diagnose` reads the world unless its regions are out of date | `tests/game.rs`: `a_step_after_the_game_is_over_changes_nothing`; `save_load_and_replay_keep_the_state` |
| #19 (rest) counters count offered orders | left untouched in this batch | — | — |

## Batch AP1 (AI perf)

Issues #52, #58, #60, #56, #55, #61, #62, #54 and #64 (time the AI spends thinking), with #57 and
#63 looked at and left without new code. The batch changes no behaviour, so no format changes (AI game
112, save 137, wire 12) and the goldens stay as they are. Each change gives the same answers as the
code it replaces. A test checks this against a copy of the old code, or the old map scan, on the M6c,
M7a and M7b duels.

**Results against base c0d6f52.** The install bench (map 9, 8 houses, seed 7, 9 000 ticks) writes
the same state hash every 500 ticks; the end hash is `02388e7c…0fb71824`. Each other check prints the
same lines as base, timing lines apart:

- the quick lane: seeds 7 to 14, maps 0, 2 and 8, sides 0,2 and 1,2, 48 games;
- the M6c, M7a and M7b sweeps: seeds 1 to 16.

**No state carried across ticks.** Nothing is cached from one tick to the next. Every kept answer
lasts one think, or one call, and none is saved.

**Final bench, run base, after, base, after back to back:**

- AI time a tick, every house: 2256.5 / 1277.8 / 2090.6 / 1243.2 µs.
- Think-tick AI per house: 3615.7 / 2082.4 / 3392.2 / 2026.5 µs.
- A whole period of thinks summed: 27 137.5 → 16 211.8 µs (the second pair).
- `tick::step` (the simulation, not changed): 2284.9 / 2102.7 / 2128.2 / 2053.7 µs.

The rows below give that second pair, in ms over the run.

**Where the time moved.** army_types (1401 → 2587) and build_superweapon (944 → 1300) grew. The
think keeps its answers on whether each type can be built. The climb, which runs earlier in the
think, used to work those answers out first. Now that the climb is skipped while its line is busy,
these later steps work them out instead. The bench at the #52 commit shows the shift: climb
3253 → 375 ms, army_types 1414 → 2924 and build_superweapon 950 → 1769.

| issue | confirmed | change | test |
|---|---|---|---|
| #52 base walk and tech climb on a busy line | yes: both searches ran while every line their building could go on was busy, and their answers were thrown away | neither search runs while those lines are busy (`order_build` orders only on a free line). Bench: climb 3204.7 → 338.7 ms, walk 942.0 → 28.8. The net saving is smaller, because the climb's kept answers are now worked out by later steps: think-tick AI per house 3443.8 → 3221.5 µs at the #52 commit | `scripted.rs`: `every_climb_answer_is_a_climbable_building` |
| #58 field questions scan the map | yes: field_cell, field_refinery, ore_goal, harvesters_wanted and fields() each scanned the map, and the ferry's land cell scanned the whole map | the think keeps one list of explored resource cells, and the field questions read it. fields() floods a dense box. The expansion tests ask the cheap questions first. The ferry scans only the square around its centre. Bench: expand:tests 1375.6 → 28.9 ms (with #64), plant_first 859.9 → 320.4 | `scripted.rs`: `the_field_answers_match_the_map_scans`; `brain.rs`: `the_land_cell_of_the_square_is_the_land_cell_of_the_map` |
| #60 special builders ask buildable and startable afresh | yes: each startable answer scanned the objects, for every builder that asked | the special builders, the yard vehicle and the special infantry ask `Script::buildable` and a new kept `Script::startable`, which are worked out once a think. sell_when_broke tests the price first. Bench: special:jumpjets 258.8 → 59.5 ms | `scripted.rs`: `the_kept_startable_answer_is_the_plain_one` |
| #56 defence views every enemy between thinks | yes: on a tick between thinks, building the list of seen enemies viewed every enemy object on the map | `Script::seen_enemies_where` reads the kept list during a think. Outside one, it views only the enemies near the base (a bounding box, then the distance to each building) or raiding it, in the same order. Enemies are tried nearest first, and the threats are a set. Bench, between thinks: defend 1944.5 → 927.0 ms | `brain.rs`: `the_nearest_first_defence_is_the_defence_of_every_enemy`, `the_threats_of_the_one_pass_are_the_threats_of_every_seen_enemy` |
| #55 dominator aim scans the objects for each cell | yes: three scans of the objects for each cell weighed, and the spread radius found again for each object | the radius is found once a search, and the objects the blast can hurt are listed once. M7b bench: fire_superweapons between thinks 30.4 → 25.1 ms, then 19.5 with #54 | `dominator.rs`: `the_aim_read_once_is_the_aim_of_every_object` |
| #61 gunner plan counts healers it may not want | yes: for every own vehicle, a pass over all objects ran before it knew whether a healer was wanted | the plan stops early when no healer is wanted or the house has no switching gunner vehicle. One pass finds the awaited vehicles as a set. Bench: special:gunners 140.0 → 21.4 ms | `gunner.rs`: `the_awaited_set_and_the_early_plan_are_as_before` |
| #62 garrison, engineers and spies count en route per building | yes: a pass over every object for each building, and garrison summed it again for every own object | one pass counts the infantry bound for each building. The engineers' rank drops a component that is always false. Each step reads its seen and ranked buildings once, for the first idle unit. Bench: special:engineers 637.4 → 378.8 ms, special:spies 63.1 → 21.7 | `special.rs`: `the_counts_of_one_pass_are_the_counts_of_a_pass_for_each_building` |
| #54 curtain reads every vehicle block | partly: a block with no seen enemy near was listed and filtered before the test that rules it out | the enemy test now runs first. Caching "no target" across ticks is not done, because the answer would not be exact | `brain.rs`: `the_curtain_cell_asking_for_enemies_first_is_the_cell_as_before` |
| #64 smaller scans (field known, absorbers, sensor, berserk) | partly: the scan of the whole map for a known field ran once each in many steps of a think; the absorber room ran a pass over the objects for each absorber; the sensor searched a cell before asking its line; berserk counted each target's group again for every unit | the think keeps whether the house knows a field. One pass counts the units bound for every absorber. The sensor asks its line first. Berserk counts each group once a call. Bench: build_specials 1127.9 → 219.7 ms, build_lifter 342.4 → 30.3, build_drainer 381.3 → 52.0, ferry 498.8 → 213.2. Left: anti-air slots, bunker exit reasons, jumpjets, grinder contains, crates and memory lookups, all small rows | `scripted.rs`: `the_kept_field_answer_is_the_scan_of_the_map`; `absorber.rs`: `the_counts_of_one_pass_are_the_counts_of_a_pass_for_each_object`; `psychic.rs`: `the_sensor_and_the_berserk_targets_are_as_before` |
| #57 deploy spot spiral | partly, in batch 2; find_deploy_spot is about 39 to 41 ms between thinks, before and after this batch (2.7 % of that AI time) | no new code. Keeping a failed spiral across ticks would not be exact | — |
| #63 refused or redundant orders | yes, as counted | won't fix. Skipping the think while orders are blocked moves the M7b game by tick 500, because the first think also sets the brain's state. Thinking but returning no orders keeps the hashes on the M7b bench, but the think still runs, so it saves nothing. The harvest re-issue goes only to resting, idle or stalled harvesters, which the sim accepts and acts on. Batching waves changes when orders are sent; the wave bookkeeping belongs to #19. The grinder Build refusals are #8 | — |

## Batch 7

Issues #19 (rest) and #27 (rest), the AP1 seam (`rebuild_yard` between thinks), and #54 and #64
(rest) looked at and left. AI game format 115 (#19's counters count differently; #27 drops a field
from the brain), save 138, wire 12. Base 0c8d9f9. #19 and #27 were each confirmed with tests
committed before the fix that failed there: #19's on a refactor commit that keeps every answer
(f006696, goldens byte-identical), #27's on their own commit (1c24f24). The AP1 seam was confirmed by
the bench, and its equivalence test came with the change.

**#19.** The fix changes no order: the counters only. In the sweeps (seeds 1 to 16) each seed's
lines stay the same; the counter lines fall where an order was dropped or replaced. M6c: garrison
median 15 → 14; jumpjets 1/40/70 → 1/38/64; radiation exits 5/22/52 → 5/21/50; chrono erasures
23/30/40 → 22/29/37; prism buildings ordered 7/11/13 → 4/7/9; charger orders max 22 → 21; sabotage
max 24 → 23; crate orders 32/67/106 → 32/65/105. M7a: berserk orders 11/31/88 → 9/31/86; lift orders
18/33/73 → 13/31/68. M7b: berserk orders max 48 → 47. Every gate passes, no margin changed. Goldens
moved: duel-m6c, duel-m7a, duel-m7b (the counters are hashed with the brain).

**#27.** The six goldens moved (duel, duel-diagonal, duel-m6c, duel-m7a, duel-m7b, duel-naval):
removing a field from the hashed brain moves every hash from tick 1, so the goldens alone do not
show whether play changed. The sweeps print the same lines as with #19 alone. The quick lane (seeds 7
to 14, maps 0, 2 and 8, sides 0,2 and 1,2, 48 games) keeps every winner: Allied 12, Yuri 12; Soviet
8, Yuri 13, no winner 3, as before. 9 games end at another tick: Allied v Yuri map 0, seed 8 (49 977
→ 50 033) and seed 10 (36 456 → 36 094); Soviet v Yuri map 8, seed 7 (118 787 → 118 784), map 2,
seed 9 (49 074 → 49 088), seed 10 maps 0 and 2 (78 863 → 78 933, 65 002 → 64 993), map 2, seed 11
(37 650 → 37 647), seed 12 maps 0 and 2 (59 085 → 59 086, 40 848 → 41 027).

**AP1 seam.** No game moves: the goldens, the sweeps' lines, the quick lane's lines (timings apart)
and the install bench's state hash every 500 ticks are those of the #27 tree; the end hash is
`b1a15b9f…e31b1a4d4` (AP1's `02388e7c…` no longer applies: #27 changed the brain's layout). Bench
(map 9, 8 houses, seed 7, 9 000 ticks), base, after, base, after, back to back on a shared machine:

- AI time a tick, every house: 1411.7 / 1521.1 / 1417.7 / 1583.4 µs;
- `tick::step` (not changed): 2613.0 / 2839.2 / 2624.5 / 2945.4 µs, so the after runs ran on a
  machine 8 to 12 % slower; the AI's share of all step time 35.1 / 34.9 / 35.1 / 35.0 %;
- between thinks, `rebuild_yard` with the pass: 452.8 → 257.7 ms (119.1 + 138.6) and 451.6 → 320.9
  ms (166.6 + 154.3).

| issue | confirmed | change | test |
|---|---|---|---|
| #19 (rest) counters count offered orders | yes: a use counted for an order the pending limit (S-0213 claim 2) dropped, and for an order a later part of the think replaced for the same unit | each use is offered with its order and counted at the end of the think only when that order is given: a behaviour order that is its actor's last one within the room left, any other matched once among the orders within the room (`memory::behaviour_given`, `Special::note_given`). The plant-first use is offered once, and only when the think had not ordered the plant already. The spin filter's count stays as it was (it counts drops). The sensor and gattling steps return whether they ordered, instead of reading a counter. `tests/bunker.rs`: an Enter the defence replaces is no entry | `tests/special.rs`: `an_order_dropped_by_the_pending_limit_is_not_counted`, `an_order_a_later_part_replaces_is_not_counted`; `tests/linked.rs`: `a_prism_order_dropped_by_the_pending_limit_is_not_counted` |
| #27 (rest) waves to a start seen empty | yes: with every start explored and no enemy building seen there, the rotation sent waves (and the ferry, rally and landing goals) to those starts in turn | a start explored (S-0409 claims 9 and 19) with no enemy building seen by it is no goal. With none left a wave searches unseen cells, and waits when there are none. The ferry, rally and landing goal takes a seen enemy building, then a start not yet cleared, then any seen enemy (an enemy base vehicle not yet deployed). The rotation counter is gone | `scripted.rs`: `start_tests::a_start_seen_empty_is_no_goal`; `tests/ai.rs`: `no_wave_goes_to_a_start_seen_empty` |
| AP1 seam: `rebuild_yard` between thinks | yes: between thinks it was the first part to ask `Script::own`, so it paid each house's scan of every object, 452.8 ms of the bench | `Game::step` lists every house's living objects on the map in one pass when the first brain thinks, and hands each brain its list. Issuing an order only adds it to the pending list, so each list is the house's own scan. `Brain::think` and the other drivers still scan | `scripted.rs`: `the_lists_of_one_pass_are_each_houses_own_scan`; the goldens and the bench's hashes |
| #54, #64 (rest) | looked at: AP1's leftovers are small rows of the bench (7bde6d0, think ticks, of about 11.2 s of think-tick AI: special:bunkers 83.9 ms, special:jumpjets 65.9, special:grinders 36.0, build_anti_air 6.4, special:crates 5.0; together 1.8 %), and #54's "no target" cache would hold an answer across ticks | left untouched by choice: the AP1 seam (rebuild_yard, 452 ms between thinks) was the larger saving | — |

## Batch 8

Issues #434 and #443 (units under air attack, own-AI design) and #445 (an AI airship bomber idle over
a base, traced). AI game format 159 on the branch, renumbered to 135 at the merge (the air-attack part's state and three use counters join the
brain), save 154, wire 12. Base 96ca8d8. The part is `air_attack.rs`, owned by the default script's
special part (`Special::think_with`); notes in [own-ai.md](../dev/own-ai.md). Each fix came after
tests committed before it. For the escape: c41edc9 and ff434a7, then 91b37d5, which counts only the
orders the shot adds, since on usable cells the base's own rally move met two of the tests (ff434a7
said they failed without the module; for those two that did not hold). For the step: e190e5b and
34228a2. For the job test: d889fce, which failed before 83818fc.

**Goldens.** All six AI duel goldens moved (the new state is hashed from the first tick): duel,
duel-diagonal, duel-m6c, duel-m7a, duel-m7b, duel-naval (dd7054b; duel-m6c again in e45724f after
the job test). duel runs longer (2588 hash lines for 2135), duel-diagonal (2081 for 2131) and
duel-naval (1855 for 1929) end sooner, so play changes there; the goldens hash the brain, so they do
not show where, and those three were not traced.

**Sweeps** (seeds 1 to 16, after the job test). M6c passes, its smallest minimum 0 as before; per seed
lines move (it has jumpjets and aircraft): seed 2 11758 → 8100, seed 7 8092 → 9032, seed 10 → 11393,
seed 13 → 8069, seed 16 8088 → 12000; seeds 1, 4, 10 and 16 are no longer below 3; garrison median
15 → 12; crate orders 72 → 97. M7a: every line as before. M7b passes: drain orders median 5 → 4 (min
3), credits drained median 1680 → 1880, takeovers median 1 → 2; seed 6's hostile-intent counter (no
gate) 0, base 202. No gate margin changed.

**Quick lane** (seeds 7 to 14, maps 0, 2 and 8, sides 0,2 and 1,2, 48 games; every game ends in
both): 10 winners change. Allied 10 (base 11), Soviet 9 (base 8), Yuri 29 (29). Each was traced to
its first differing tick (world hash without the AI's state, then the objects); in each the first
difference is one escape order of this part to a unit of the house a seen airborne enemy shoots:

| game | tick | unit (house, kind, behaviour before) | attacker | goal |
|---|---|---|---|---|
| 0,2 map 0 seed 8 | 7613 | 1407 (1, infantry, area guard) | aircraft 1469 | away |
| 0,2 map 0 seed 9 | 18218 | 5053 (1, infantry, move) | aircraft 1478 | away, replacing the move |
| 0,2 map 0 seed 12 | 11813 | 1573 (1, infantry, guard) | jumpjet infantry 1477 | away |
| 0,2 map 0 seed 13 | 8873 | 1391 (1, infantry, move) | air vehicle 1500 | cover (closer to the attacker) |
| 0,2 map 8 seed 11 | 8258 | 802 (1, vehicle, attack) | jumpjet infantry 939 | away |
| 1,2 map 0 seed 8 | 11483 | 1374 (1, infantry, attack) | air vehicle 1975 | cover or home |
| 1,2 map 2 seed 9 | 18631 | 881 (0, vehicle, guard) | air vehicle 2445 | away |
| 1,2 map 8 seed 11 | 23603 | 6199 (1, infantry, attack) | air vehicle 5854 | home |
| 1,2 map 2 seed 12 | 27203 | 4250 (1, vehicle taken by the dominator, attack) | air vehicle 4516 | cover (closer to the attacker) |
| 1,2 map 2 seed 14 | 13223 | 839 (1, vehicle, guard) | air vehicle 1754 | home |

In 0,2 map 0 seed 13 a second unit (1293, guard, not shot) got in the same think the move the
escaping unit had in the base: no other part reads this part's watch, so the other part's pick
shifted once the first unit was taken, likely through the pending-order memory the parts share (not
traced further). Twice the cover goal is nearer the attacker than the unit was
(the cover unit stands under it). Air steps: 3 orders in 96 house-games; slow attackers over a unit
that can hit them are rare in stock games. A first run of the lane (before the job test) changed 9
winners; the trace of one showed an engineer on a capture sent home, which the job test fixed.

**Bench** (map 9, 8 houses, seed 7, 9 000 ticks), base then after, back to back on a shared machine:
`tick::step` 2651.3 / 2787.3 µs, AI time a tick 1465.7 / 1570.8 µs, AI share 35.6 / 36.0 %; the
games differ (1634 / 1610 objects at the end), and the growth lies in parts not touched
(army_types 3049 → 3386 ms, build_superweapon 1682 → 1965 ms); `special:air_attack` costs 18.3 ms
(0.15 % of think-tick AI). The base alone ranged 2239 to 2807 µs this session.

| issue | confirmed | change | test |
|---|---|---|---|
| #434 units under air attack they cannot answer | yes: a unit shot by an air unit it cannot hit stood under it (no retaliation, S-0306 claim 23, S-0215 claim 18), and the base's rally move sent a moved unit straight back | `Special::air_attack`: each army unit a seen airborne enemy shot within `RECENT` ticks, that cannot hit it and that it reaches, moves to cover (a house unit or defence that can hit the attacker within `COVER` cells), else home from the field, else out of reach plus `MARGIN` (S-0307 claim 13); up to `HELPERS` idle units within `HELP_REACH` that can hit it are sent at it; a unit sent away keeps its order for up to `HOLD` ticks (`Watch::keeps_order` in `Brain::think_from`); a unit on another part's job (current or queued behaviour not a fighting one) is left alone (`free_of_job`) | `tests/air_attack.rs`: `a_ground_unit_hit_from_the_air_goes_to_anti_air_cover`, `an_anti_air_unit_nearby_is_sent_at_the_attacker`, `a_ground_unit_hit_from_the_air_in_the_field_goes_home`, `a_ground_unit_hit_from_the_air_at_home_moves_out_of_reach`, `a_unit_getting_away_is_not_ordered_at_every_think`, `a_unit_on_its_way_into_a_bunker_is_not_ordered_away`, `a_unit_of_another_house_is_never_ordered` |
| #443 a unit under a slow air attacker it can hit | yes: it fired from under a hovering attacker that kept over it | `Special::air_step`: when the attacker is still within `OVERHEAD` cells one think later and the unit fired at it within `RECENT` ticks, it steps `STEP` cells away, at most once in `STEP_GAP` ticks; no attack order follows, since retaliation brings it back to firing | `tests/air_step.rs`: `an_anti_air_unit_under_a_slow_bomber_steps_aside_and_keeps_firing`, `after_a_step_retaliation_alone_brings_the_unit_back_to_firing`, `a_unit_under_a_slow_bomber_is_not_ordered_at_every_think`, `a_unit_of_another_house_under_a_slow_bomber_is_never_ordered` |
| #445 an AI bomber idle over a base | traced: the brain gives one valid attack order (the lasting check neither illegal nor impossible) and nothing more; a bomber reaching half a cell hovers over the building's top-left cell with no re-aim (S-0307 claim 18, S-0310 claim 9); one without the mobile-fire flag never fires (S-0310 claim 7) | none: the AI's order is valid and the simulation follows its claims; the open questions are spec request Q-3257 | `tests/air_bomber.rs`: `an_ai_bomber_at_an_enemy_base_is_sent_at_it_once`, `a_bomber_that_reaches_from_afar_bombs_the_plant_from_some_sides`, `a_bomber_that_reaches_half_a_cell_stalls_over_the_plant`, `a_bomber_without_mobile_fire_never_bombs_the_plant` |

## Batch 9

Issue #449: pluggable AI scripts, a per-player difficulty and a headless Elo tournament (own design;
notes in [ai-scripts.md](../dev/ai-scripts.md)). Save format 193, AI game format 171, wire 13 on the branch (renumbered to 161, 139 and 13 at the merge), each
in its own commit. Base 9cc9b4a. The default script at normal changes nothing: a game of default
seats hashes its brains as before, so no golden moved and none was regenerated.

**Lanes**, base then after: the quick lane (seeds 7 to 14, maps 0, 2 and 8, sides 0,2 and 1,2, 48
games) matches line for line apart from the timing lines; the M6c, M7a and M7b sweeps (seeds 1 to
16) match apart from the build lines; the arena's outputs match file for file; the sim gate with
its ignored tests and the AI suite with its ignored tests pass as on the base (plus the new tests);
the bench (map 9, 8 houses, seed 7, 9 000 ticks) writes the same state hashes.

| part | confirmed | change | test |
|---|---|---|---|
| 1 interface and registry | the brain was the only AI; `Game` held a list of brains | `scripts::AiScript` (in: the house's shroud-filtered `View` and the think `Schedule`; out: orders), the fixed `REGISTRY` (id 1, `default`, the brain unchanged), one `Seat` per AI house with its choice and its script's state, saved and hashed with the AI state | `tests/scripts.rs`: `every_script_decides_only_on_what_it_sees` (every script and difficulty), `the_difficulty_row_is_a_persons_at_every_ai_difficulty`; `scripts.rs`: `the_registry_is_ordered_and_stable` |
| 2 setup field and difficulty | no house carried an AI choice; every player house uses the normal row (S-0302 Terms and claim 3, S-0301 claim 3, S-0412 claim 5, the glossary's *difficulty skill block*), so no request | `GameSetup.ai` (serde default, carried and never read by the simulation); easy, normal and hard: easy keeps one harvester, attacks in waves of three and defends its base on think ticks only, hard keeps three harvesters, normal is today's; a Script and a Difficulty cell on each computer row of the setup screen, the lobby's choices edit carries them, the skirmish lane takes `GLASNOST_AI_SEATS` | `tests/scripts.rs`: `the_choice_survives_save_load_and_the_hash`, `a_setup_without_choices_plays_the_default_at_normal`, `the_difficulties_change_the_plan_only_away_from_normal`; `ui/src/setup.rs`: `a_computer_row_chooses_its_script_and_difficulty`; `net/tests/lobby.rs`: `the_host_sets_a_computer_rows_script_and_difficulty` |
| 3 tournament | — | `glasnost-headless ai-tournament`: a round robin over grounds, seeds, starts and sides, a tick cap won at 3/2 of the other's worth (the cost fields of S-0403 claim 2) or drawn, Elo from 1500 with K 32 in the fixed game order, a report and one replay per game, threads in the tool only. A small install round (maps 0 and 8, seed 7, cap 27 000, 24 games): hard 1540.0, easy 1489.6, normal 1470.3; the side played decided more games than the level, and easy and normal are not told apart by 16 games each. On the fixture (maps 0 and 1, seeds 1 and 2, 48 games) normal 1532.7, hard 1518.2, easy 1449.1 | `tools/tests/ai_tournament.rs`: `the_same_options_give_the_same_report` (threads 1 and 2, twice, each replay plays its game again), `the_ratings_follow_the_games_in_order`, `the_options_parse` |

## Batch 10

Issues #454 (spies disguise before they infiltrate), #445's AI side (a bomber that holds over its
target without firing) and #449's follow-up (the difficulty levels told apart); own design, notes in
[own-ai.md](../dev/own-ai.md) and [ai-scripts.md](../dev/ai-scripts.md). AI game format 179 (renumbered to 144 at the merge) on the
branch, in its own commit (the spies' jobs, the bombers' watch and four use counters join the
brain); save 163 and wire 13 unchanged. Base f62d868. Each fix came after tests committed before it
(#454: 9aa7246, then 4c40ecf, a test change, before 4a52ce4; #445: 7706919, then dac7f6b, a test
change, before d19cd01).

**Goldens.** All six AI duel goldens moved, as in batch 8 (the new state is hashed from the first
tick): duel, duel-diagonal, duel-m6c, duel-m7a, duel-m7b, duel-naval (593454d). Each keeps its
length. scripted, combat and skirmish did not move. The difficulty levers (902cffb, 82cb228) add no
state and leave normal as it was, so they moved no golden.

**Sweeps** (seeds 1 to 16). M6c and M7a: every seed line as on the base. M7b passes, no gate below
3; seeds 1 and 2 move and every other seed line is as before. A probe of the sweep's games showed
that only the bombers' part acts there (seed 1 two orders, seed 2 six; no spy order): at tick 5626
in both, Alpha's disc (a jumpjet hovering in attack on building 17, last shot at 5142 and 5018) is
sent west of its target, then the other discs, and at 17 again at 5761 to 5776. Seed 1's drain
counters 1800 → 1680 credits, seed 2's first link 6164 → 6284; the medians that hang on those two
seeds move: infantry mutated 23 → 22, berserk weapon orders 31 → 25, hostile intent seen 1584 →
1599, and the maxima of takeovers 4 → 5 and objects taken over 23 → 25.

**Arena**: every output file as on the base.

**Quick lane** (seeds 7 to 14, maps 0, 2 and 8, sides 0,2 and 1,2, 48 games): every 0,2 game
changes length, 4 of the 1,2 games change at all, and 7 winners change. Allied 11 (base 10), Soviet
11 (11), Yuri 25 (25), not over at the cap 1 (2). Each was traced to its first differing tick
(`GLASNOST_AI_HASH`, then `GLASNOST_AI_OBJS`):

| game | base → after | tick | first difference |
|---|---|---|---|
| 0,2 map 0 seed 7 | Yuri → Allied | 19306 | house 0's infantry factory starts infantry type 15 for type 16 |
| 0,2 map 0 seed 8 | not over → Allied | 20686 | the same |
| 0,2 map 8 seed 8 | Allied → Yuri | 13261 | the same |
| 0,2 map 8 seed 10 | Allied → Yuri | 14626 | the same |
| 0,2 map 0 seed 13 | Yuri → Allied | 20401 | the same |
| 1,2 map 0 seed 12 | not over → Soviet | 41836 | house 0's jumpjet 8287, hovering in attack on 7472 with no shot since 40157, is sent west to cell (65, 88) (#445) |
| 1,2 map 8 seed 13 | Soviet → not over | 50116 | house 0's jumpjet 13069, in attack on 14678 with no shot since 47225, is sent to cell (40, 61) (#445) |

In the five 0,2 games the after's type list counts one army type fewer (90 for 91): the spy, by
elimination, since the only role the batch changes is the spy's (#454: a spy is no longer an army
unit). The script's army pick then lands on another infantry type, which suggests the base's pick
there was the spy (type 16 is not otherwise identified). The after's houses still field spies: the
Allied house's infiltration orders over the 24 0,2 games are 88 (base 65).

**Bench** (map 9, 8 houses, seed 7, 9 000 ticks), base then after on a shared machine:
`tick::step` 2328.5 / 1981.7 µs, AI time a tick 1490.6 / 1163.7 µs, AI share 39.0 / 37.0 %; every
part fell alike, so the fall is the machine's load, not the change. The game differs (1634 / 1633
objects at the end, the end hash moves): `special:spies` 27.9 → 60.2 ms (0.64 % of think-tick AI),
the new `special:bombers` 6.7 ms (0.07 %). The bench plays at normal, so the difficulty levers do
not reach it.

| issue | confirmed | change | test |
|---|---|---|---|
| #454 spies disguise before they infiltrate | yes: a spy with a disguise weapon went in undisguised; a permanent disguiser is created disguised for a person's house and an AI house alike (S-0516 claim 32), so no simulation fault | `disguise.rs`, run by `Special::spies` once it has chosen the building: a disguiser (a weapon with the disguise-flag warhead, S-0516 claim 31) that does not fool the owner (claim 35) attacks the nearest seen infantry of the owner or an ally within `COPY_REACH` for up to `COPY_TIME`; with none in sight it goes in undisguised and turns to disguise once one comes into sight, which also re-disguises a spy a hit uncovered (claim 34 (a)); a fooling spy walks round the seen disc of a detector building of the owner or an ally (claim 40) when that crosses fewer covered cells, once per disguise; a spy is no longer an army unit; counters `disguises`, `spy_detours` | `tests/spy_disguise.rs`: `a_spy_disguises_as_the_victims_infantry_first`, `a_spy_goes_in_once_disguised`, `a_spy_with_no_infantry_to_copy_goes_in_undisguised`, `a_spy_without_a_disguise_weapon_goes_in_as_before`, `a_spy_whose_disguise_a_hit_cleared_disguises_again`, `a_disguised_spy_goes_round_a_seen_detector`, `a_spy_copies_only_infantry_it_sees`, `a_spy_avoids_only_detectors_it_sees` (every script and difficulty), `a_permanent_disguiser_is_created_disguised_for_every_owner` |
| #445 a bomber that holds over its target without firing (AI side) | yes, on synthetic rules with person orders: a type without the mobile-fire flag never fires at its attack target (S-0310 claim 7) and hovered over the plant until the next wave, over 7000 ticks; a weapon without the fire-while-moving flag fired from over the point only after an approach flown towards map east (S-0502 claim 10, Q-3257), else held. What the AI relies on from the simulation since S40: a jumpjet sent at a building flies to its reference point (S-0502 r34 claim 16), bombs on its way in when its weapon fires while moving, keeps a vertical projectile's target as its destination (S-0307 claim 18) and does not re-aim (S-0310 claim 9); a held hover slot gives a neighbour step (S-0502 claim 12) | `bomber.rs` (`Special::bombers`, after `air_attack`): a jumpjet in attack on an object with no shot for `STALL` ticks since it came within `NEAR` cells of the target's point, and there still, is sent `ROUND` cells west of the point and then at the target again (an approach towards map east); the next time over the same target it is sent home and kept out of attacks and attack-moves for `BENCH` ticks; `Watch::filter_orders` in `Brain::think_from` keeps its orders; counters `bomber_rounds`, `bomber_retreats` | `tests/air_bomber_stall.rs`: `an_ai_bomber_that_never_fires_over_its_target_is_moved_on`, `an_ai_bomber_whose_bomb_needs_its_heading_comes_again_from_the_west`, `an_ai_bomber_that_fires_is_left_alone`; `tests/air_bomber.rs`'s four #445 tests stay green |
| #449 follow-up, the difficulty levels told apart | not separated: in 144-game rounds on the install (maps 0, 2 and 8, seeds 7 to 10, cap 27 000) hard leads normal (0.58 to 0.66) and easy (0.55 to 0.61), easy and normal stay level (0.53 at the base, 0.56 after both levers); the side played decides more than the level (country index 0 scores 0.56 to 0.75), and normal v hard moved 0.08 between rounds with neither level changed, so about 0.08 is noise; tables in [ai-scripts.md](../dev/ai-scripts.md) | `Brain::think_from`: at easy the house acts on one of every `EASY_EVERY` (3) of its think ticks, read from the tick (902cffb); at easy it builds no superweapon, sensor, grinder, gattling defence, bunker, lifter or drainer (82cb228); normal and hard unchanged, no state added; the difficulty table and the rounds in ai-scripts.md | `tests/scripts.rs`: `easy_acts_on_one_think_tick_in_three` (a688b11, then 35970b1, a test change, before 902cffb), `easy_builds_no_superweapon_nor_special_buildings` (70d728b before 82cb228) |

## Batch 11

Issue #449's next follow-up: why the default script on country index 1 loses to itself on
country index 0, and the levels told apart. Own design; notes in [ai-scripts.md](../dev/ai-scripts.md).
AI game format 185 in its own commit (1e24d93; renumbered to 147 at the merge): a replay with an easy seat resolves differently.
Save 165 and wire 13 unchanged; no state added. Base f393424. Each lever's test failed on the
base code before the lever was written, but test and levers went in one commit (0666777), since
both levers sit in `Brain::think_from`.

**Goldens.** None moved, and none was regenerated: no golden, lane or gate game seats easy.

**Lanes**, base then after:
- **Quick lane** (seeds 7 to 14, maps 0, 2 and 8, sides 0,2 and 1,2, 48 games): every game ends
  on the same tick with the same winner, and the s7 0,2 output differs in timing lines only. No
  winner changed, so there was nothing to trace.
- **Sweeps.** The M6c, M7a and M7b sweeps (seeds 1 to 16) match apart from the build lines.
- **Arena.** Its output files match byte for byte.
- **Sim gate.** It passes 17 of 17 with its ignored tests, as on the base.
- **Bench** (map 9, 8 houses, seed 7, 9 000 ticks). It ends on the same state hash; `tick::step`
  ran 2035.5 / 1863.0 µs on a shared machine.

| item | confirmed | change | test |
|---|---|---|---|
| 1 why country index 1 loses | normal mirror (maps 0, 2 and 8, seeds 7 to 14, cap 27 000, 96 games): country index 0 scores 0.844 (63-32-1). Its country paradrop is about half of that: a probe that never fires it scores 0.667. The paradrop is a rules asymmetry, which #449 allows, and stays. The rest is play: in the probe, the capped games scored on army and aircraft worth alone give 0.714, and on map 2 country index 1 holds half the army worth at the cap. It sends no first wave in the base lane, and in one dump lost 2.2 times the army units (82 against 37). The map starts are swapped within each pair, and no simulation claim was found | none (findings) | lane files only: `tour-nn-base`, `tour-nn-p1`, `tour-nn-p1c` (worth by part), dumps d1 to d3 |
| 2 the weaker side's play | not fixed. One fix was tried: a house that sees more enemy army aircraft than it has army units that hit them picks anti-air types per factory. It moved the probe mirror from 0.667 to 0.656, within one standard error (about 0.05), against an aim of at most 0.60 fixed before the round. Only 9 of the 82 losses were to aircraft. Not shipped; handed back with the numbers | none | its three synthetic tests went with it (described in the report) |
| 3 the levels told apart | separated. Criterion fixed before the first round: each pair's stronger level scores at least 0.60 in the 144-game round (maps 0, 2 and 8, seeds 7 to 10), and the Elo order is hard, normal, easy. Base: normal over easy 0.552, hard over easy 0.615, hard over normal 0.667, Elo easy above hard above normal. E1 alone passed the pairs but not the Elo order. E1 and E2: 0.812, 0.823, 0.667, Elo 1631.3 / 1521.6 / 1347.1. On fresh seeds 11 to 14: 0.750, 0.750, 0.646, Elo 1606.8 / 1499.9 / 1393.3. B10's two levers stay | `Brain::think_from`: at easy the house fires no superweapon, its country's own included (E1). At easy it builds no refinery by a new field (E2: `Script::stay_on_home_field`, a flag of the think's memo, read by `Script::field_refinery`; a lost refinery is still rebuilt). Normal and hard unchanged | `tests/ai.rs`: `easy_fires_no_superweapon`; `tests/expansion.rs`: `easy_builds_no_refinery_by_a_new_field` (both failed on the base code first) |

## Script AS1

Issue #449: a second AI script, `siege` (registry id 2), our own design, notes in
[ai-scripts.md](../dev/ai-scripts.md#siege-script-as1). The goal was its hard level at 2000 Elo in
the tournament, playing fair. The default script and every code path it runs are unchanged; the
siege calls the brain, never edits it. AI game format 146, save 167 and wire 13 unchanged: the
siege's seat is a new variant, appended, and no existing encoding changed. Base d9f2750.

**Measurement**, fixed before the first round: the certifying round is
`ai-tournament game=install maps=0,2,8 seeds=11-14 ticks=27000 threads=2` with the six entrants
`default:easy,default:normal,default:hard,siege:easy,siege:normal,siege:hard` (720 games), seeds
11 to 14 never used in development. Development rounds: `siege:hard` against `default:hard` on
seeds 7 to 10 (48 games). Stop rule: the goal met, or three development rounds in a row with no
clear gain (a rise of at least 0.05 in the score or of 4 victories, neither country index falling
by more). The rule stopped development after dev-7, dev-8 and dev-9.

| part | confirmed | change | test |
|---|---|---|---|
| the siege script | the registry had one entry; "To add a script" in ai-scripts.md | `siege.rs`: `Siege` (the brain on a plan that holds its waves, the house's view, the army), `Tuning` by level, `ScriptState::Siege`, its start arm and `REGISTRY` entry; at normal and hard no superweapon building, the climb building in its place; one gathered strike with attack-moves and a guard, joined in groups; at hard the remembered enemy army, the edge, the recall, economy first and the hunt | `tests/siege.rs`: `the_registry_lists_the_siege_script`, `a_setup_choosing_siege_plays_it_at_every_level`, `the_brains_waves_are_held_at_every_level`, `the_levels_play_worse_to_better`, `the_strike_goes_at_its_count_and_leaves_the_guard`, `an_outmatched_strike_is_called_home`; `siege.rs`: `the_memory_forgets_a_unit_once_its_cell_is_seen_empty_or_after_a_while`, `a_level_without_superweapons_orders_no_superweapon_building`; `tests/scripts.rs`: `every_script_decides_only_on_what_it_sees` (now with the siege) |
| fair play | the default's shroud filter (`Script::seen_enemies`), the house's explored and seen cells, the map's start cells; a person's rules and money at every level (S-0302 claim 3, S-0412 claim 5) | the siege reads nothing else and gives move, attack-move and build orders on its think ticks | as above |
| the watch tool | — | `glasnost-headless siege-watch map= seed= a= b= [start= side= ticks= every=]`: one tournament game with a line per house every `every` ticks (credits, buildings, harvesters, army by type, cost got by kind, army lost by tenth of the way to the enemy and by killer's kind, lines, the siege's strikes, recalls and remembered worth) | — |
| the setup screen | the screen lists `REGISTRY` | none here (crates/ui is outside this batch); its test `a_computer_row_chooses_its_script_and_difficulty` expects a single script and fails with two: the click on the Script cell now gives id 2, a second click 1 | to be updated by a batch that may touch crates/ui |

**Certifying round cert-1** (HEAD 9bb84ad, 720 games, none failed): the goal is not met. Final Elo:
`siege:hard` 1642.2, `siege:normal` 1605.7, `default:easy` 1602.9, `siege:easy` 1445.3,
`default:hard` 1368.4, `default:normal` 1335.5. `siege:hard` won 152, lost 37 and drew 51 of 240
games: against `default:easy` 25-5-18, `default:normal` 41-2-5, `default:hard` 34-5-9,
`siege:easy` 30-8-10, `siege:normal` 22-17-9. By country index 0 76-26-18, 1 76-11-33. Of its 152
wins 106 were victories and 46 worth at the cap. 2000 needs about 98 percent against the defaults
and 95 against the siege's own lower levels; the draws (most on map 2, where equal armies stall)
and the small lead over `siege:normal` keep it far short. The Elo is order-dependent: in this
round `default:easy` rates above `default:hard`.

**Lanes**, base d9f2750 against this branch, none moved: the quick lane (seeds 7 to 14, sides 0,2
and 1,2, maps 0, 2 and 8; 48 games, every result and tick count the same), the M6c, M7a and M7b
sweeps (seeds 1 to 16), `arena --record all` (every record file the same), the sim gate with
ignored tests (17 passed), and `ai-bench game=install map=9 players=8 seed=7 ticks=9000` (the same
hashes). The siege on both seats, which `ai-bench` cannot set (its install setup is built in
crates/app), was played in the skirmish lane's AI mode with `GLASNOST_AI_SEATS=siege:hard,siege:hard`
(map 0) and `siege:normal,siege:easy` (map 2): the replay reproduces the end and the mid-game save,
loaded and continued, matches.

## Batch 12

Issue #449's next follow-up: why country index 1 fields half the army, the fix, and the levels
told apart again around the new normal. Own design; notes in [ai-scripts.md](../dev/ai-scripts.md).
AI game format 193 in its own commit (41201e4; renumbered to 150 at the merge): normal's army round picks differently, so replays
resolve differently. Save 167 and wire 13 unchanged; no state added (the hard flag is a field of
the think's memo, set each think). Base 2ce4555. Each failing test went in before its fix: 52b8710
before caae09d, 812f5b0 before cb153f1.

**Goldens.** `crates/sim-gate/golden/duel-m6c.txt` moved (the M6c duel seats normal; its hash first
differs after tick 594) and was regenerated (157e2f2). The other eight are byte-equal.

**Lanes**, base then after:
- **Quick lane** (seeds 7 to 14, maps 0, 2 and 8, sides 0,2 and 1,2, 48 games). 19 games change
  winner: 6 with sides 0,2 and 13 with sides 1,2. Each was traced with `GLASNOST_AI_HASH`, then
  `GLASNOST_AI_OBJS`, to its first differing tick. In every one it is the infantry type a house's
  line starts, with credits equal: the army round's pick at normal, the only place normal changed.
  The first tick is 4703 to 5888 in 15 games. The other four are all with sides 0,2: seed 8 map 0
  (33016), seed 9 map 2 (35063), seed 10 map 8 (8566) and seed 12 map 2 (23941).
- **Sweeps.** M7a and M7b match apart from build lines. M6c passes on both, but its games move:
  seeds 2, 10 and 13 end at other ticks, and counters move (suicide orders minimum 10 to 8). The
  smallest minimum is 0 on both, from support beams on seeds 6 and 13.
- **Arena.** Its nine output files match byte for byte.
- **Sim gate.** It passes 17 of 17 with its ignored tests, as on the base, after the golden. The
  sim-only scenarios and tests do not move.
- **Bench** (map 9, 8 houses, seed 7, 9 000 ticks). The end state hash differs, since normal plays
  otherwise. `tick::step` ran 1942.3 / 1846.6 µs on a shared machine.

| item | confirmed | change | test |
|---|---|---|---|
| 1 why country index 1 fields half the army | **Setup.** The paradrop-off probe mirror (maps 0, 2 and 8, seeds 7 to 14, cap 27 000, 96 games) scores 0.620 (37-45-14) for country index 0; the normal mirror 0.781 (55-40-1). **What is alike.** Both countries buy about the same army (20995 against 20646 credits a game) and fire a first wave alike, and their fighting ground types survive alike. **What differs.** Country index 1 spends 45.7 % of its army's credits on units sent alone and lost on use (bomb, suicide, charge, designator, engineer), country index 0 21.9 %. So country index 1 stands with 5529 credits of army a game against 8867. **Cause.** The round ranks by fewest living, and a spent-alone type has none standing once used; country index 1's list holds four such types | none (findings) | lane files only: `tour-nn-base`, `tour-nn-p1`, `tour-nn-p1d` and `tour-nn-p2off` (probe reads: spend by type, tagged; hashes equal to `tour-nn-p1`'s) |
| 2 the fix | fixed. Probe mirror 0.620 to 0.516 (26-47-23), under the 0.60 bar. Country index 1 sends 2.99 waves a game (1.77 before) and stands with 6799 credits of army. Standard mirror 0.781 to 0.771 (56-36-4): the country paradrop, a rules asymmetry, stays | `scripted.rs` army round: the types `spent_alone` names are offered only while their spend times `SPENT_SHARE` (4) is under the rest of the army's spend, where a kind has another type (about a fifth of the army's credits). A person's orders and economy only | `tests/spent_alone.rs`: `the_round_builds_a_fighter_once_the_spent_share_is_full` (failed on the base code first), `the_round_still_builds_a_spent_type_within_the_share` |
| 3 the levels told apart | not met for one pair; handed back. Same criterion as B11 (each pair at least 0.60, Elo hard, normal, easy). Base: 0.719, 0.781, 0.594 (hard over normal already under), Elo 1390.9 / 1545.6 / 1563.5 (easy / normal / hard). With the fix: 0.812, 0.812, 0.531. H1 (kept): 0.812, 0.812, 0.552, Elo 1305.0 / 1562.0 / 1633.0; on seeds 11 to 14 0.781, 0.812, 0.531, Elo 1391.8 / 1498.5 / 1609.7. Hard over normal by hard's country: 16-7-1 on country index 0, 2-10-12 on country index 1 (seeds 11 to 14: 14-9-1, 1-12-11), so the country gap is what is left. H2 (a plan of 4 harvesters at hard) gave 0.594 but cost hard 9 of 24 games against easy on country index 0, since the army is held back until the plan's harvesters stand; dropped (2972f76 removed its test a690c74). Try 3 (four wanted at hard beside the army) did not bind on a large field; dropped before any commit. B11's E1 and E2 stay | H1: at hard `Brain::think_from` sets the memo flag `Script::spend_less_alone`, and the share is `HARD_SPENT_SHARE` (8, about a ninth). Easy and normal use 4 | `tests/spent_alone.rs`: `hard_keeps_a_smaller_spent_share_than_normal` (failed on the base code first) |

## Script AS2

Issue #449, the siege's second batch: its hard level toward 2000 Elo in the tournament, playing
fair. Notes in [ai-scripts.md](../dev/ai-scripts.md#hard-script-as2). The default script and the
shared code it runs are unchanged; the siege's own code and tests change, and the tournament
report gains one line. Base 6cffcc6.

**Measurement**, fixed before the first round:

- **Baseline cert-0:** AS1's certifying round on the base (seeds 11 to 14, the six entrants,
  720 games).
- **Certifying round:** `ai-tournament game=install maps=0,2,8 seeds=15-18 ticks=27000 threads=2`
  with `default:easy,default:normal,default:hard,siege:easy,siege:normal,siege:hard`. Seeds 15 to
  18 are not used in development. The goal is `siege:hard`'s final Elo in the report at 2000 or
  more.
- **Development rounds:** `siege:hard` against `siege:normal`, `siege:easy`, `default:easy` and
  `default:hard`, one two-entrant tournament each, on maps 0, 2 and 8, seeds 7 to 10 (192 games).
- **A clear gain:** the score over the 192 games (win 1, draw ½) up at least 0.03 on the best kept
  round, or 8 more victories, with no pair down more than 0.06.
- **Stop rule:** the goal met, or three development rounds in a row with no clear gain.

| part | confirmed | change | test |
|---|---|---|---|
| order-free Elo | the table's Elo takes the games in their fixed order, so the last games weigh most (AS1: `default:easy` rated above `default:hard`, which led their pair) | `ai_tournament::order_free_ratings`: the mean final Elo over 100 seeded shuffles of the games, put first in the order of their job and result; one more report line. The table and its Elo are unchanged | `tests/ai_tournament.rs`: `the_order_free_ratings_do_not_depend_on_the_games_order`, `the_report_gives_the_order_free_ratings` |
| the early guard (dropped) | cert-0: 12 of `siege:hard`'s 23 losses on map 0 end between ticks 6 000 and 9 300; a watch of game 173 shows no army unit before tick 4 500, the brain's army round waiting for the plan's three harvesters, and `siege:easy`'s five infantry taking the base | tried: below six army units on the ground, the siege ordered the cheapest infantry army type not sent alone. Development round: 0.760 (+0.023), 99 victories (+8), but `default:hard` fell 0.105; on map 2 the house stayed at two refineries. Reverted | none (its test was reverted with it) |
| more factories when rich | a watch of the base's game on map 2, seed 10, against `default:hard`: 3 000 to 9 566 credits in hand from tick 19 500, an army of 6 to 12, one infantry and one vehicle line running | `Tuning::rich` (hard only): with that much money in hand, power enough for the new factory, no building ordered and the building line free, the siege orders the cheapest factory of the kind it owns fewer of, once it owns one, up to three of each (S-0403 claim 6: more factories, shorter build times) | `tests/siege.rs`: `hard_adds_factories_when_rich` |
