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
