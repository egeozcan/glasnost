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
