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
