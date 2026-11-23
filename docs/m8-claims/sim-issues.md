# Sim issues: the game-logic audit (M8)

The lead's audit of the game logic filed its findings as GitHub issues, tracked in #420. Each batch fixes some of them. Each finding is first confirmed by a test on synthetic data, committed before its fix. Behaviour comes from `spec/` only. A limit that is our own input validation is marked `// nocite: our own input validation (#N)`, and a request is filed where such a limit could refuse something a person really does. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S1: orders and saves from outside)*.

## Batch S1: orders and saves from outside

Base main 4f55f4c, rebased onto e869e80 (which changes no deterministic crate). Save format 128, AI game format 95, wire version unchanged. Failing tests first (6d30848, 2cde2af, 974201e). Requests Q-2971 to Q-2973 (26a1ea0).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #155 | `Order::Routed` held a whole order: a peer could send it to keep a route, and nested bytes overflowed the decoder's stack | fixed (bc7204e) | The marker holds a `RoutedOrder` (S-0213 claim 16c: a behaviour order's fields); `issue` refuses it (nocite #155); routes issue through `issue_routed` | `hostile_orders.rs::a_routed_order_from_outside_is_refused`; `hostile_decode.rs` (60 000-deep bytes, and `input_orders_keep_their_bytes`) |
| #220 | `issue` took an order for any house's object | fixed (719ae72) | `issue` refuses an actor that is missing or not the issuing house's (S-0213 claims 3 and 15, S-0214 Terms *Own object*); an order taken still applies after an owner change (claim 15). Orders in flight over the network: Q-2972, answered by S-0213 r104 and built in batch S2 | `hostile_orders.rs` (refused, control, owner change after issue); `claims_orders.rs::s0213_no_owner_check`, `s0213_pending_limit`; `pairs_m7a.rs` take-over pair |
| #219 | Cells were never checked: far cells overflowed placement and a superweapon's target conversion (panic in debug, wrap in release) | fixed (6f75368) | `orders::in_window`: every cell in -512 … 1023 on each axis (nocite #219); legal cells 0 … 511 (S-0009 claim 2), clicks on map cells only (S-0513 claim 1), off-map margin for S-0513 claim 3 and S-0512 claim 10 | `hostile_orders.rs` (outside refused, corners taken and run 300 ticks); `hostile_orders_sw.rs` (every superweapon kind at the corners, far second stage and curtain refused) |
| #158 | Joins between two holders doubled a route each round, and point adds resumed past 16 | fixed (453d690) | `routes::MAX_STEPS` = 1024: a point add at the bound is refused silently, a join past it does nothing. Q-2971, answered by S-0213 r104 (claim 16b gives this bound as our engine's own); the nocites became citations in batch S2 | `hostile_orders.rs::joins_do_not_double_a_route_without_bound`, `point_adds_past_16_points_stop_at_the_bound` |
| #157 | A save whose tables disagree panicked in the region rebuild or the first ticks | fixed (bcfe309) | `save::load` refuses a save whose cell tables, random streams (S-0202 claim 2), objects (id, house, type) or pending orders (the window) do not hold together, as `SaveError::Format` (nocite #157) | `hostile_save.rs` (7) |
| #156 | A cancelled master's hidden slaves, and a cancelled controller's record, stayed in the world | fixed (13ddfbb) | `production::drop_creation_set`: slot objects (S-0506 claim 13), slaves (S-0402 claim 19, S-0404 claim 6) and the controller record (S-0601 claims 1 and 10), at a cancel and at an upgrade's placement. Q-2973, answered by S-0312 r71: since batch S2 the slaves go by claim 31's group removal, a loss each | `cancel_creation_set.rs` |

Lanes, base 4f55f4c against the batch's code before the rebase: the oracle arena's records are byte for byte the same (failing scored pairings M-0001 14 of 96, M-0002 5 of 24, M-0003 2 of 35); the M6c, M7a and M7b sweeps over seeds 1–16 are identical line for line; the AI suite with its ignored tests passes 264 both times; the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) has the same winner on the same tick in every game. The goldens did not move. So the new checks change nothing that these games do.

Seams left (also in the design section): other removals straight from `w.objects`; a save's overlay indices and cross-record ids; the cost of a 1024-step route scan; release and debug builds differ in `overflow-checks`.

## Batch S2: answered requests, rules data without bound, order marks, unread switches

Base main 1d06a13, rebased onto 576c974 before hand-back. Save format 130, AI game format 99 (the own-AI batch took 98 first), wire version unchanged (12). Failing tests first (dcc0659, 464b8ba, b7c6972, f057bde, 0ff08c5, 2570d6f, c795883). Requests Q-2996 to Q-3001 (4b5febf). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S2: …)*.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-2971 | S1's route bound was our own | answered (a7a8469) | `routes::MAX_STEPS` cites S-0213 r104 claim 16b | `hostile_orders.rs` |
| Q-2972 | S1 checked the owner at `issue`, not when a network game applies the order | answered (60b6a9d, app 86778bd) | `Settings::network`; in a network game `apply_pending` drops an order whose actor the issuing house does not own then (S-0213 r104 claim 15), detonate excepted (claim 11); the app's `net_begin` sets the flag | `network_owner_check.rs` |
| Q-2973 | A cancelled master's slaves were removed without a loss | answered (25352d5) | claim 31's group removal (`slaves::removing`, S-0312 r71), a loss each | `cancel_creation_set.rs` |
| #197 | A scatter or deploy order kept depot contact and a queued depot entry | fixed (2116ba2) | `depot::stop_order` on those orders too (S-0314 claims 20 and 23) | `claims_s0314_depot.rs` |
| #195 | Some target settings left the opportunity mark set | fixed (2dcab1a) | Every setting of a target clears it (S-0309 claim 24); the searches set it after their own setting | `order_marks.rs` |
| #196 | An order's target left the charging mark set | fixed (2dcab1a) | Cleared whenever an order or another ordinary change gives a target (S-0518 claim 15) | `order_marks.rs` |
| #194 | A huge veteran-crate step count looped for a long time | fixed (f56b5cb) | Stops at 3 steps (S-0520 F7: three reach elite from any experience); no outcome changes | `veteran_steps_bound.rs` |
| #170 | A huge sensor range walked its whole disc each tick | fixed (52cc6f1) | `cloak::sensor::disc_on_map`: only the disc's part in the square of legal cells (S-0009 claim 2); no outcome changes | `sensor_disc_bound.rs` |
| #201 | A huge adjacency walked a huge rectangle, and the AI's support table overflowed | fixed (b75e5dc) | Both cut to the map's coordinate square, the table in `i64`; `TABLE_MARGIN` gone; no outcome changes | `adjacency_bound.rs` |
| #198 | A huge spark count or railgun length made that many particles in one tick; a loop of next particle types split without end | fixed (556d431, 33f159c) | `particles::BURST_CEILING` = 4096 (S-0616 r29 claim 20a since batch S4, Q-2999 answered); the loop mark below | `particle_bounds.rs` |
| #224 | A loop of trailer animation types never ended its pass | fixed (33f159c) | `loops::LinkLoops` (Tarjan on a work list) finds looped links at rules build; an object made through one is marked and takes no looped link (S-0319 r30 claim 15 since batch S4, Q-2996 answered) | `trailer_cycle.rs` |
| #174 | A loop of bounce, expiry or spawn animation types, or of meteor piece spawns, grew without end | fixed (33f159c) | The same mark on `Anim` and `Piece` (S-0319 r30 claim 15, S-0317 r59 claim 29 since batch S4; Q-2996, Q-2997 answered) | `spawn_cycle.rs` |
| #203 | A loop of airburst weapons launched children without end | fixed (33f159c) | The same mark on `Projectile`; since batch S4 the links lead between projectile types (S-0306 r87 claim 21a, Q-2998 answered) | `airburst_cycle.rs` |
| #218 | A long row of exploding objects nested one detonation each on the stack and aborted the program | fixed (71b7804; S4) | S2 nested to a depth of 16 and listed deeper ones (Q-3000 open). Q-3000 answered (S-0312 r74 claim 33): batch S4 sets every death weapon off depth first with no limit, on a continuation stack in `World::death_chain` instead of the machine stack | `death_chain_stack.rs` |
| #327 | Quirk switches that a spec names were declared but never read | fixed (ab00c0b) | `table_trig_negative_bias`, `unchecked_substitute`, `repeat_direction_snap`, `air_death_fall_generic_follow_up` and `permadisguise_attack_cursor` read; the rest documented on their fields as not consulted, with the reason; S-0206 claim 16 not built (Q-3001, answered by S-0206 r19 and built in batch S4); `defeated_enemy_house_kept` and `anger_to_first_house` for the next batch | `quirk_switches.rs`, `quirk_switch_disguise.rs` |

Lanes, run before the rebase (base 1d06a13 against the batch on top of it; commit names in this paragraph are the rebased ones): the oracle arena's records are byte for byte the same; the M7b sweep over seeds 1–16 is identical line for line; the AI suite with its ignored tests gives the same results; the M6c and M7a sweeps keep every game length and per-seed line, and only some counters moved (M6c: garrison orders median 21 → 20, deploy orders max 494 → 486, radiation in the field max 7900 → 8135, support beams median 111 → 109, bomb planter orders median 42 → 43). Builds bisected at 2116ba2 and 2dcab1a put the counter change at 2dcab1a (#195, #196). In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games), 20 games are the same and 16 end on another tick; two change winner: seed 8 map 2 (65122 ticks, house 1 → 45736, house 0) and seed 9 map 0 (106239, house 1 → 73848, house 0). Rerun at 2116ba2 both games end as on base, and at 2dcab1a as after the batch, so both come from #195 and #196. They come from the order marks S-0309 claim 24 and S-0518 claim 15 require, so they stand. The goldens moved for the new serialized fields (save 130); hashes only. After the rebase the six AI duel goldens were made again on top of the own-AI batch. On the rebased batch the workspace tests, the AI suite with its ignored tests (275), clippy, fmt, the wasm32 build and `xtask check` and `gate` pass, after three late fixes: the committed rules schema made again for #327's field docs (7a0cd48), lint allowances for the bound tests' harness threads (3ceed21), and the loop search's and support table's literals put in the forms `xtask check` reads (2fe523d).

Seams left (also in the design section): a huge spawn count with no loop still makes that many animations or pieces in one pass; death-weapon chains past the depth went off in listed order, not nested order, until Q-3000 was answered (closed in batch S4); the AI's other tables over rules data were not audited; `table_trig_negative_bias` changes no walk between cell centres (their headings fall on even table indices), so the world test checks the switch through `table_trig` and the step test by facing.

## Batch S3: intake 124's answers, the anger rules, combat

Base main 7072c76; the lanes ran on the batch rebased onto b477743 (spec commits only), and the batch was then rebased onto 841732b (render R5 and AI batch 2; the goldens did not move, the AI suite with its ignored tests passes 277). Commit names in this section are the final ones. Save format 131, AI game format 100, wire version unchanged (12). Failing tests first (73800a8, b3a71f4, 5920241, 0536e54, cdc2775, bdbe246). Requests Q-3011 and Q-3012 (f9dc780). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S3: …)*.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-2986 | *L* was set only by some hits on a building | answered (d1af67c) | `computer::building_hit` at the start of every hit with an attacker that reaches a building (S-0217 r22 claim 3): a 0 amount, a healing hit, a hit S-0302 claim 8 stops | `claims_s0217.rs` |
| Q-2987 | Whether a drop keeps the holder's destination, and a target difference after an eviction | answered, no code | S-0309 r133 claim 27 confirms our drop; its Open questions give no post-eviction rule beyond part j | — |
| Q-2988 | M-0024's un-hit neighbouring-destination row failed | partly answered (247ecfd) | M-0024 makes the row not for tests; the oracle prints it in the full report and no longer judges it. Open for question 3 | `oracle_m0024.rs` |
| S-0309 part j | The two anger rules of a test-only computer house were not built | fixed (54e1f60) | `computer::anger_towards_first_house` (`quirks.anger_to_first_house` off: the nearest house by base records) and `clear_defeated_enemy`, at each check before claims 3 and 4 | `claims_s0309_part_j.rs` |
| #173 | Anger from a killing hit read the victim after the kill | fixed (680788a) | `AngerShare` takes owner, strength and cost before the kill handling; `acquire::anger_of` uses them (S-0309 F8) | `claims_s0309_killing_hit_anger.rs` |
| #176 | A vehicle tethered to a building could fire | fixed (dacade9, 1b6b6fa) | Rank 12 *impossible* (S-0301 claim 9), after the healer's *illegal* (Q-3012; batch S4 puts it first, S-0301 r76); not for the illegal-only ask nor the own AI's lasting test (our design: a reason of the moment; the AI suite's `defends_the_base` caught it) | `claims_s0301_tether.rs` |
| #177 | A deployed deploy-fire vehicle used its primary weapon | fixed (ac16d82) | Its type's deploy-fire index; out of the slots, 0 (S-0308 claim 9, Edge cases) | `claims_s0308_deployed_vehicle.rs` |
| #171 | An occupied building's threat ignored its occupants | fixed (cf93f94) | Occupants × `[General] ThreatPerOccupant` (S-0309 Terms) | `claims_s0309_occupied_threat.rs` |
| #172 | Area damage left out a sinking vessel with health left | fixed (3aef3da) | `damage::dead` (killed and no health) in area, gas and particle hits (S-0504 claims 10 and 13). A bridge repair hut killed with health left is now hit too | `claims_s0504_sinking_area.rs` |
| #175 | A building's explosion delay was drawn and thrown away | fixed (0e2ad41) | `effects::anim_delayed`: the draw is the loop delay (S-0312 claim 19 (b), S-0109 claim 11); stream and order unchanged | `claims_s0312_explosion_delay.rs` |
| #217 | Expected damage included the garrison and bunker multipliers | fixed (6fb9f09) | `damage::shot_e1` (E₁ of S-0302 F1) for `expected_damage` (claim 18, F8) | `claims_s0302_expected_e1.rs` |
| #221 | Deploy and pack-up dropped the holders instead of handing them over | fixed (690bae8; ring test 5324ba5) | `objects::remove_converted` without claim 27's drop, then `deploy::retarget` (S-0415 claims 6 and 10); a refused pack-up drops them (Q-3011). Batch S4 builds the answer, S-0415 r22 claim 13: a pack-up's holders drop the building at its removal, then take the vehicle on success | `claims_s0415.rs` |
| #178 | Weapon choice tested airborne only for aircraft and jumpjets | fixed (4e95c40) | `combat::weapon::airborne`: an aircraft by `in_air`, anything else by S-0307's height test (S-0308 Terms); table P rows 17 to 19, the gattling slot and S-0615 claim 14's disc test | `claims_s0308_airborne.rs`, `claims_s0615.rs` |

Lanes, base 7072c76 against the batch on b477743. The oracle arena moves in two pairings, both bisected to 0e2ad41 (#175): M-0001 DISK against E2 passes now (ours 11/0/1 → 12/0/0; failing scored pairings 11 → 10 of 96, reproduced 157 → 158), and M-0002 YURI against E2 still fails its side outcome (ours 4/8/0 → 3/9/0, nearer the record's 0/12/0; 4 of 24 either way). M-0003 (2 of 35), M-0019 (0 of 6), M-0020 (0 of 25) and M-0021 (10 of 50) keep their counts; some games end on other frames. Traced, DISK against E2 side B seed 2 (none@6080 → row@2432) agrees in every traced field to tick 761; at 762 the disc B#425 takes the GI A#477 as its target in the batch only. The game's GIs garrison neutral buildings, whose destruction starts the explosion animations #175 now delays; the trace does not print animations. M-0024 judges 6 rows, none failing (base 1 of 7, the row now not for tests); M-0025 0 of 6. The M7b sweep is the same as base but for seed 14 (first link 6287 → 6266, drain counters [4, 3, 1560, 4312, 9, 21, 8, 19] → [4, 3, 1760, 4471, 9, 23, 8, 17]); the smallest drain minimum stays 3. The M7a sweep keeps every game length and its smallest minimum 1 (seed 9's takeover orders); its below-3 lists change in seeds 7, 8 and 15 (seed 15's smallest counter 2 → 3). The M6c sweep changes: seeds 6, 7, 11 and 15, which ended early on base, play all 12000 ticks; seeds 1 and 2 end early (9286, 9857). The gate's seed 2 broke on #221 (traced: from tick 39, seven of house 0's infantry keep house 1's new construction yard as their target, S-0415 claims 6 and 10), so `M6C_SEED` is 3 (65dec81: all ticks, smallest counter 4). The AI suite with its ignored tests passes 275, as on base. In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) every game ends on another tick, and 8 change winner (seed/map/game, base → batch): 7/0/2 house 1@59457 → house 0@68082, 7/8/2 1@52566 → 0@42527, 7/8/3 0@40635 → 1@62916, 8/0/3 1@45860 → 0@54204, 8/2/2 0@27317 → 1@38327, 8/8/1 1@41511 → 0@43692, 8/8/2 1@42870 → 0@85813, 10/8/3 1@42461 → 0@29741. Traced, 8/2/2 has the same world fingerprint to tick 4467; at 4468 the only object that differs is house 1's building #650, expected health 944 on base and 953 in the batch: a pick at it no longer counts the garrison or bunker multiplier (#217, S-0302 claim 18 and F8). 7/0/2 is the same through tick 8000; the other six are not traced. The goldens moved (c5993b8, 7e0914f). #221 also ends the path report tool's synthetic game at tick 209 (the person's vehicles that held the machine's construction vehicle destroy its new yard), so the ring test starts without them (5324ba5); from 690bae8 to that commit the workspace builds but that one test fails.

Seams left (also in the design section): S-0301 rank 12's tether against the healer's *illegal* (Q-3012); a refused pack-up's drop order (Q-3011); M-0024 question 3 (Q-2988); M_t in expected damage; the AI crate's comments on the lasting test still name only ranks 5 to 8 (the AI owners'), and under it an armed harvester docked at a refinery now counts as able to attack; `in_air`'s other callers keep their own specs; S-0312 r74 claim 33 (Q-3000 answered: death weapons nest with no depth limit) arrived on main during the batch and is not built here (S2's depth of 16 stayed until batch S4).

## Batch S4: eight answered requests, order targeting

Base main b522007, not rebased (main is at 424de8f, with AI batch B3's game format 101). Save format (131) and AI game format not bumped, wire version unchanged (12): no saved layout changed (`World::death_chain` is `serde(skip)`; the new rules fields reach a save only through its rules hash), and no golden moved; main's AI game format 101 stands after the merge. Failing tests first (78005ca, e8512ee, 72b6c41, a68069b, 447e87e, 581fe17, 3cb31a9, 69a0463, 0d39c1c, 214775a, c603ac4, 9cdf773). No new requests. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S4: …)*.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-2996 | S2's loop mark on animations was our own, with one warning per graph | answered (a3f7bd0) | Cites S-0319 r30 claim 15; `Loops::groups` gives one warning per loop group (a type linked to itself is a group); a spawn link exists only with a spawn count above 0 (S-0319 Terms) | `claims_link_loops.rs` (`one_warning_for_each_loop_of_animation_types`, `a_spawn_type_with_a_count_of_0_is_no_link`); `trailer_cycle.rs`, `spawn_cycle.rs` |
| Q-2997 | The same for debris and meteor pieces | answered (a3f7bd0) | Cites S-0317 r59 claim 29; one warning per piece loop group | `claims_link_loops.rs::one_warning_for_each_loop_of_piece_types`; `spawn_cycle.rs` |
| Q-2998 | Airburst links ran between weapons | answered (a3f7bd0) | The graph's nodes are projectile types (S-0306 r87 claim 21a): a weapon off the loop firing a looped type makes one generation, not two; a shrapnel fragment carries no mark. Q-2998's open questions stay open | `airburst_cycle.rs::a_weapon_off_the_loop_firing_a_looped_type_makes_one_generation` |
| Q-2999 | The particle ceiling and conversion were our own | answered (a3f7bd0) | Citations only: S-0616 r29 claims 8a (a smoke split's successor marked, a gas successor never), 20a (`BURST_CEILING` = 4096), F12 | `particle_bounds.rs`, `claims_link_loops.rs` |
| Q-3000 | S2 nested death weapons 16 deep and listed deeper ones, out of claim 33's order | answered (67c9ad5) | S-0312 r74 claim 33: depth first at every depth with no limit, on the continuation stack `World::death_chain` (not saved, not hashed); `DEATH_WEAPON_DEPTH` and the listing removed | `claims_s0312_d25.rs` (rows of 50 and 600 on a 1 MiB stack), `death_chain_stack.rs` |
| Q-3001 | S-0206 claim 16 (the track body stop) was not built | answered (3d514f3) | `behave::track_body_stop` under `quirks.track_body_turn_stops_at_target`: a turretless track vehicle with a target and no destination stops its body turn at the direction to the target; the switch leaves the not-consulted list | `claims_s0206_track_stop.rs` |
| #218 | A long row of exploding objects nested one detonation each on the stack | fixed (67c9ad5) | Closed by Q-3000's answer: the chain runs on `World::death_chain`, not the machine stack, so no row length aborts the program or changes the order | `claims_s0312_d25.rs`, `death_chain_stack.rs` |
| #327 | The last quirk switch S2 left declared but unread, `track_body_turn_stops_at_target` | fixed (3d514f3) | Read by Q-3001's track body stop; it leaves the not-consulted list | `claims_s0206_track_stop.rs` |
| Q-3011 (#221) | A pack-up's holders' drop order | answered (be0df17) | S-0415 r22 claim 13: the holders drop at the building's removal, before the cell recalculation, and take the new vehicle on success (`deploy::hand_over`); a deploy hands over before the removal; `objects::remove_converted` removed | `claims_s0415.rs::claim_13_pack_up_drops_then_hands_the_holder_to_the_vehicle` |
| Q-3012 (#176) | The tether ranked after the healer's *illegal* | answered (6a2da17) | S-0301 r76 claim 9 rank 12: the tether's *impossible* first; still a reason of the moment | `claims_s0301_tether.rs::a_tethered_healer_gets_impossible_before_its_illegal` |
| #207 | The Guard key's harvester branch ordered another house's harvester | fixed (3d6a3ae) | `key_command` tests `own` there too (S-0213, S-0214 *Own object*) | `claims_s0213_guard_owner.rs` |
| #208 | A building's and a terrain object's `Immune`, `LegalTarget` and `Bombable` were not read | fixed (0974a34) | `targeting::immune`, `legal_target`, `bombable` read them (S-0214 Import mapping); new rules fields `BuildingType::bombable`, `TerrainObjectType::{legal_target, bombable}` | `claims_s0214_object_flags.rs` |
| #209 | A slave and an object being erased counted as movable | fixed (6581794) | `movable` leaves both out, `box_eligible` a slave (S-0214 Terms) | `claims_s0214_slaves.rs` |
| #210 | Rows IC4 and VC1: a healer's attack on a cell | fixed (e00405d) | Infantry: area guard (IC4); vehicle: idle (VC1) | `claims_s0214_healers_on_cells.rs` |
| #211 | The friendly attack cursor ignored claim 6's bomb-planting exception | fixed (a691775) | Not counted when the weapon `would_use` has a bomb-planting warhead and the object has passengers (S-0214 claim 6) | `claims_s0214_bomb_friend.rs` |
| #212 | Row GC4: an enemy wall the warhead can damage was not an attackable cell | fixed (b87e1d6) | `attackable_cell` takes the actor: legal-target overlays (new `OverlayRules::legal_target`) or a destroyable cliff without the fire flag (`WarheadRules::fire`), or a wall not an ally's with the wall flag, or the wood flag on a wood wall | `claims_s0214_walls.rs` |
| #411 | A factory-marked absorber got its primary order only | fixed (bf2e667) | `click` also issues `Order::Deploy` for it when its action is deploy (S-0214, S-0605) | `claims_s0605.rs::a_factory_marked_absorber_gets_both_deploy_orders` |

Lanes, base b522007 against the batch at bf2e667 (f848544 changes docs only). The oracle arena's failures are base's row for row, so none is a regression: failing scored pairings M-0001 10 of 96, M-0002 4 of 24, M-0003 2 of 35 (the same pairings as base); M-0021 10 of 50 (the same failing rows as base, and as in batch S3); M-0019 0 of 6, M-0020 0 of 25, M-0024 0 of 6, M-0025 0 of 6. The determinism goldens (debug prefix and full run, the six AI duels among them) match the committed hashes unchanged. In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) 16 games are the same and 20 end on another tick; three change winner (seed/map/game, base → batch): 7/0/3 house 0@72104 → house 1@39540, 8/2/2 1@38327 → 0@31008, 8/2/3 1@77290 → 0@69072. 8/2/2 bisected: built at 3d514f3 it ends as on base, at be0df17 (Q-3011, S-0415 r22 claim 13's holder drop before the cell recalculation) as after the batch, and no later commit changes it, so it comes from the holder order claim 13 requires and stands. 7/0/3 and 8/2/3 are not traced. The M6c, M7a and M7b sweeps over seeds 1–16 are identical to base line for line. On the final tree `xtask check` passes, the AI suite with its ignored tests passes 277 (as in batch S3), and the sim gate with its ignored tests passes (goldens, the three sweeps and their replays, margins, play fixes).

Seams left (also in the design section): a death weapon reached by a route without the chain flag (a bomb's detonation, a building's death effects, a passenger's death, a crash) nests one level on the machine stack, then drives its own chain; Q-2998's open questions; the veinhole case of GC4; the gap generator's and a garrison's unload orders at a click (#411); the remainder of #81.

## Batch S5: aircraft, jumpjets and erasing

Base main 0a7db0f, rebased onto 581f8fe (AI batch 4's game format 103). Save format 131 → 132 and AI game format 103 → 104 for `JumpState::filtered` (#190); wire version unchanged (12). Goldens regenerated (e3243c3). Failing tests first (7d4d6fd for every issue, a25a0bb for #191's settled switched-off case). Requests Q-3041 to Q-3043 (4b160a8) and Q-3044 (997054d). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S5: aircraft, jumpjets and erasing)*.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #159 | *Move*'s next processing gave a landing aircraft its held destination again and took it off | fixed (3323098) | `air::ignores` applies claim 8's cell test to the destination the flight already holds too (S-0501 claims 8, 40) | `claims_s0501_landing_regive.rs::a_moves_next_processing_does_not_take_off_an_aircraft_landing_at_its_destination` |
| #160 | A destination object out of play resolved to the map origin, and the aircraft turned toward (0, 0) | fixed (9a40c5f), interim | `air::flight_target` gives no flight point for an object destination not in play (S-0501 claims 24, 32, 42); keeping the last point instead is Q-3041 | `claims_s0501.rs::a_strafer_whose_destination_object_is_removed_does_not_steer_to_the_origin` |
| #161 | A shot-down strike aircraft stayed linked until its wreck's removal | fixed (4726ab5) | `death::mark_killed` calls `strike::killed`; `strike::leaving` finds nothing more for it at the removal (S-0519 claim 20) | `claims_s0519_shot_down.rs::a_shot_down_strike_aircraft_is_unlinked_at_its_killing_hit` |
| #162 | The report after a touchdown at rest requested the landing direction as facing | fixed (78cfba7) | `air::dock` sets no facing (S-0501 claim 24 r134, Q-2649; claim 25's direction serves claim 42 only) | `claims_s0501.rs::claim_24_the_report_after_a_touchdown_at_rest_changes_no_facing` |
| #188 | Removing a jumpjet vehicle hovering over a vehicle cleared that vehicle's flag | fixed (efc4496) | `drive::held_claims` has a jumpjet arm (its own vehicle-present mark, by the type's locomotor, so a lifted unit keeps its own arm); `release_claims` drops the released mark from `JumpState` (S-0502 claim 4, S-0209 claim 1) | `claims_s0502_removal_claims.rs::removing_a_hovering_jumpjet_vehicle_keeps_the_flag_of_the_vehicle_below` |
| #189 | A crash held up by a listed object raising the reference | request only | Not changed: F3, F4 and claim 18 as written; Q-3042 asks | none |
| #190 | A destination the own-cell filter ignored was taken by the movement update's retry once the unit had left the cell | fixed (217dd8c), interim | New saved `JumpState::filtered`; the retry in `jump::tick` skips it; cleared when an assignment passes the filter (S-0502 claim 23, S-0213 claim 21); the order-level destination keeping it is Q-3043 | `claims_s0502_own_cell_in_flight.rs::a_filtered_own_cell_assignment_stays_ignored_after_the_unit_leaves_the_cell`; `claims_s0502_r6.rs::claim_23_infantry_own_cell_order_is_ignored` checks the record |
| #191 | A rocket aimed at a moving hovering vehicle's position, not its heading point | fixed (9f2eb6d, 0a3cca3), interim | `rocket::heading_point_with` gives the hover's *P* while it is powered (S-0506 claim 14, S-0503); a settled switched-off hover keeps *P* and is aimed at its position (Q-3044's interim) | `claims_s0506_hover_heading_point.rs` (`rocket_aim_at_a_moving_hover_is_its_heading_point`, `rocket_aim_at_a_switched_off_hover_is_its_position`) |
| #213 | A jumpjet below the listing height counted as out of the ground layer for erasing | fixed (a50eac4) | `jump::in_ground_layer` (*r* < 208 or *r*′ = 0, S-0102, S-0502 claim 4); `temporal::ground_layer` uses it for any unit with a jumpjet state (S-0514 claim 1) | `claims_s0514_low_jumpjet.rs::a_low_jumpjet_is_in_the_ground_layer_for_the_erasure_test` |
| #214 | Contact with any building refused the erasing lock, not only a vehicle factory | fixed (7785cbd) | `temporal::refusal` requires `econ::release::vehicle_factory` (S-0514 claim 3) | `claims_s0514_depot.rs::a_vehicle_on_a_repair_depot_is_locked`; `claims_s0514.rs` (`a_vehicle_in_its_factory_is_refused` on a vehicle factory, `a_vehicle_in_contact_with_a_building_that_is_no_factory_is_locked`) |

Lanes, base 0a7db0f against the batch on that base before the rebase (eafae69, the same commits as eb7f45f; the rebase brings in only main's AI batch 4, a settings change and a rustfmt layout). Oracle arena: M-0001 failing 10 of 96 → 9 of 96. TELE against APOC now holds; TELE against BFRT goes 2/4/6 → 2/3/7 (M-0001's record 0/2/10) and TELE against V3 0/1/11 → 0/5/7 (the record 0/12/0), both toward the record. Only TELE games change, and bisection puts all of it on #213 (a50eac4): a seized unit carries a jumpjet state (S-0607 claim 5), so S-0102's jumpjet layer row now applies to it. M-0002 4 of 24, M-0003 2 of 35 (the first #191 change broke ROBO against V3, which Q-3044's interim restores to base's 4 of 4 for V3), M-0021 10 of 50, M-0019 0 of 6, M-0020 0 of 25, M-0024 0 of 6 and M-0025 0 of 6 are base's. The M6c, M7a and M7b sweeps over seeds 1–16 pass with base's smallest minima (0, 2, 3). In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) 7 games are the same and 29 end on another tick; five change winner (seed/map/game, base → batch): 7/8/2 house 1@49553 → 0@31616, 8/8/1 0@37224 → 1@55103, 8/8/2 0@49887 → 1@49013, 10/0/3 1@47840 → 0@64382, 10/8/2 1@39327 → 0@23830. None is traced. The longest, 9/0/3 (62513 → 110833 ticks, house 0 both times), was rerun and is a long war (5796 shots, 316 killed, 21 lifts, both houses building to the end), not a stall. On the final tree `xtask check` and `xtask gate` pass, the AI suite with its ignored tests passes 306, and the sim gate with its ignored tests passes 16.

Seams left (also in the design section): #189 waits for Q-3042; Q-3041, Q-3043 and Q-3044 may change the interims of #160, #190 and #191; a killed designator's and a destroyed marked building's cases of S-0519 claims 19 and 20 have no audit test.

## Batch S7: refinery animations, rocket range, contacts and rechecks

Base main 946161b; the lanes ran on the batch rebased onto 0b3e137, and the batch was then rebased onto a8fbe56 (spec commits only; `git diff --stat 946161b a8fbe56 -- crates/deterministic data` is empty). Commit names in this section are the final ones. Save format 132 → 134 and AI game format 104 → 107 (the numbers between were named for other batches) for the refinery's *pre-production* slot and its saved animations (#430); wire version unchanged (12). Goldens regenerated (61f7468): six of the seven moved, duel-m7b did not. Failing tests first (6274d04, ebb8788, 5bb355e, 1b8c0b6, 6506bfb, 3925a55, 94311e5, f482a04; eb00f5c for #431 passes unchanged). Requests Q-3059 to Q-3061 (638e5dc, 05160d7, 600a7d6) and Q-3062, Q-3063 (c2bef0a, c203d22); Q-3069 (08492ef) for #431. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S7: refinery animations, rocket range, contacts and rechecks)*.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #430 | The refinery played no docking, payment or unloading animation, and the harvester never waited for the unloading one | fixed (0272301, e1fdc0c), interim | `DepotSlot::PreProduction`; `harvest::process_unload`'s start, payment and empty-load rows play the neighbouring building's slots through `depot_anim`, and *finishing* waits for a refinery-flag building's unloading animation (S-0402 claim 11, S-0108 claims 12 and 25); the order against the smoke event is Q-3062's interim, a missing damaged name Q-3063's | `claims_s0402_dock_anims.rs` (7) |
| #253 | A set-A or set-B rocket below 208 leptons in its ascent was measured on the ground | fixed (136cea3) | `weapon::range_airborne` and `range_floor_bound` give those types S-0506 claim 22's phase answer, read by `range` and the airborne rows of `check` (S-0307 Terms and claim 26, S-0306 claim 23) | `claims_s0506.rs::a_rocket_in_ascent_below_208_is_not_measured_on_the_ground` |
| #431 | A turret building attacked ground units but not aircraft or missiles | finding, no fix | The synthetic test passes unchanged. A local probe on an installation's rules (not a skirmish): the probe's never-captured buildings were not in operation (S-0515 claim 2, S-0405 claim 8), and their check answered *impossible*; a captured one passes the check without range against a helicopter, but S-0307 claim 9's spatial range test, which counts the flight height, never passed against the cruising aircraft. The rockets did not fly in the probe, so the missile half is unmeasured; #253 can explain a short window at most (reasoning). Whether that geometry is right for a building against air is a spec question; Q-3069 asks | `claims_s0309_outpost.rs` |
| #422 | When a harvesting slave plays its shovel sequence | request only | Not changed; Q-3059 asks | none |
| #185 | A harvester ordered off its refinery kept a tether that a later depot contact revived | fixed (5c2c46f) | `harvest::tethered` needs the harvester to be a live holder of its contact (S-0402 claims 10 and 11, S-0213 table T3 (b)) | `claims_s0402_tether.rs` (2) |
| #179 | A docked vessel killed by a sinking or crashing hit held its yard place until removal | fixed (c56b2b3) | `death::release_depot_contact` at the killing hit, before the sink or crash starts (S-0504 claim 14, S-0314 claim 22) | `claims_s0504.rs::claim_14_killed_docked_vessel_frees_its_place_at_the_killing_hit` (sinking; the crash branch has no test of its own) |
| #222 | An erased depot left its contacts linked | fixed (4f6af23) | `temporal::complete` calls `depot::depot_destroyed` for each linked contact before removing the building (S-0314 claim 22, S-0514 claim 12) | `claims_s0314_erased_depot.rs` |
| #223 | A building kept its primary mark through an owner change | fixed (dc492d5) | `econ::events::owner_changed` clears it (S-0509 claim 12) | `claims_s0509.rs::claim_12_primary_mark_cleared` |
| #202 | Gaining an upgrade, selling one and the start of a sale set no recheck | fixed (9b73fa6) | `placement::add_upgrade` and `sell::sell_upgrade` set the owner's recheck; the sale start does when `quirks.sale_start_no_recheck` is off (S-0407 claim 12 (d) and (i), Edge cases) | `claims_s0407_upgrade_recheck.rs` |
| #168 | A sensor array placed under construction and its first-placement test | request only | Not changed; Q-3060 asks | none |
| #163 | A revert to a put-aside target that left the game earlier | request only | Not changed; Q-3061 asks | none |
| #164 | An area guard with no guard range had a leash of 0 | fixed (3d64ec9) | *g* is the larger of the primary and secondary weapon ranges when the guard range is 0 (S-0215 F5) | `claims_s0215_leash.rs` |

Lanes, base 946161b against the batch on 0b3e137. Oracle arena: M-0001 failing 9 of 96 → 10 of 96: LTNK against E1 now fails the side outcome (ours 0/12 → 1/12, the record 0/12); CAOS against E1 goes 10/12 → 9/12 (still failing), YTNK against E2 11/12 → 12/12 (passes); not bisected; by elimination (no refinery, rocket or depot in these pairings) the candidate is #164's leash. M-0002 4 of 24 (INIT against E1 and E2 move by one game each, both still failing as before), M-0003 2 of 35, M-0019 0 of 6, M-0020 0 of 25, M-0021 10 of 50 are base's; M-0024 and M-0025 were not printed in either run (cause not checked). The M6c, M7a and M7b sweeps over seeds 1–16 pass; their smallest minima go 1 → 0, 1 → 0 and 3 → 3 (M6c seeds 5 and 9 now end early, before a jumpjet order or support beam; M7a seed 13 has no bunker entry). In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) every game ends on another tick, and ten change winner (seed/map/game, base → batch): 7/0/1 house 0@42668 → 1@64912, 7/0/3 1@33876 → 0@43890, 7/2/3 1@28267 → 0@53501, 7/8/2 0@34222 → 1@35722, 8/0/2 1@37772 → 0@26578, 8/8/1 1@55103 → 0@34044, 9/0/3 0@40809 → 1@84986, 9/8/2 1@45126 → 0@18104, 10/8/2 0@23830 → 1@45639, 10/8/3 1@52559 → 0@85074. None is traced; the harvest timing of #430 touches every game. The longest, 10/2/2 (40848 → 103051 ticks, house 1 both times), ends inside the cap. On the final tree `xtask check` and `xtask gate` pass, the AI suite with its ignored tests passes 306, and the sim gate with its ignored tests passes 16. The workspace tests pass except `glasnost-rules`' `every_spec_quirk_has_a_switch`, which fails on main too: S-0502's revision on main names `quirks.jumpjet_crash_lookahead` (Q-3042, #189), which no code defines yet.

Seams left (also in the design section): Q-3059 to Q-3061 wait for answers; Q-3062 and Q-3063 may change #430's interims; S-0108 claim 13's power freeze of the refinery animations is not modelled; #431's range geometry against a cruising aircraft waits for Q-3069.

## Batch S6: bridges, crushing and walkers

Base main 0a7db0f, rebased onto 946161b, 0b3e137 and a8fbe56 (main's changes in between are batch S5, the client map picker and spec revisions; no simulation code). Save format 132 → 136 and AI game format 104 → 109 for the layout of `DriveState::claims` (#181; 5a0a7e0, the numbers the main session gave this batch after S7's 134 and 107 and S8's 135 and 108); wire version unchanged (12). Goldens regenerated on the rebased tree (a3b8b8e; unchanged by the renumbering). Failing tests first: 44a7741 (#165, #166, #167), 869aaa6 (#181), f0b1dc8 (#183), 12774a8 (#199), a942d01 (#180), 6d91e04 (#182), 562fe2d (#184), 5b38319 (#186), 6bb8759 (#187), 15e824a (#204). Request Q-3046 (bc226df). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S6: bridges, crushing and walkers)*.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #165 | A moving walker's or drive vehicle's deck reservation in a lost section's cell stayed in the deck record and came back with the repaired deck | fixed (f294773) | `bridges::blow::dropped_layer` picks the deck record for a heading point at least 416 above the surface in a cell that carries or carried a high bridge (S-0511 claims 5, 9; S-0207 claim 24) | `claims_s0511.rs` (`claim_9_moving_walker_drop_clears_its_deck_reservation`, `claim_9_moving_vehicle_drop_clears_its_deck_reservation`) |
| #166 | A pier's reference cell was found by arithmetic and never checked, so a tile painted over the pier made the replacement flood another tile | fixed (89f75ae), interim | `pieces::pier_reference_of` takes the computed cell only when it holds the hit cell's tile at the reference sub-tile; otherwise the attempt fails and a notice does nothing (S-0511 Terms, claims 2, 4, 6); what such a pier does is Q-3046 | `claims_s0511.rs` (`claim_6_a_painted_over_reference_cell_floods_nothing`, `claim_6_a_painted_over_reference_cell_in_state_3_collapses_nothing`, `claim_6_a_cut_off_part_does_not_restore_or_collapse_again`) |
| #167 | A pier collapse with no link within 3 cells left the regions out of date | fixed (65d7f98) | `high::name_cell` marks connectivity out of date on every path; links rebuilt in play are put in creation order through `bridges::sort_spans` (S-0511 claim 12, S-0212 claims 3, 5, 13b) | `claims_s0511.rs::claim_12_pier_collapse_with_no_link_nearby_updates_the_regions` |
| #181 | A drive vehicle set a claim in the layer of the track's start and cleared it in the layer of the release, leaving a flag behind on and off a deck | fixed (bdda544) | Claims are set in the layer the vehicle will be in (`drive::entered_layer`, S-0207 claim 5) and stored with their layer, which every release clears (S-0209 claim 1); save layout changes | `claims_s0209_deck_claims.rs` (`crossing_a_bridge_leaves_no_claim_behind`, `a_vehicle_removed_on_the_deck_clears_its_claims`) |
| #183 | Braking and the give-up test measured to a high-bridge destination at terrain height | fixed (f07e25a) | `drive::move_destination` adds the 416 deck lift to a high-bridge cell destination; `update_rho` and `give_up_test` use it (S-0207 Terms, claim 5, F5, F7) | `claims_s0207_deck_destination.rs` (`braking_on_a_bridge_deck`, `give_up_on_a_bridge_deck`, and their flat-ground controls) |
| #199 | The nearby search judged a bridge cell on its ground outside deck mode | fixed (ef431ab) | `qualifies` passes the bridge flag alone (S-0212 claim 25) | `claims_s0212_bridge_class.rs` (`a_bridge_cell_over_rock_qualifies_outside_deck_mode`, `a_bridge_cell_over_rock_qualifies_in_deck_mode`) |
| #180 | A thief's crush take-over wrote the owner field alone | fixed (408ea1e) | The take-over goes through `mind::set_owner` (S-0210 claim 5, S-0414 claim 4, S-0309 claim 30) | `claims_s0210.rs::a_take_over_is_a_full_change_of_owner` |
| #182 | A crushed object was removed with no kill credit | fixed (b4a3b4f) | `stats::destruction` and `veterancy::destruction_credited` credit the crusher before the removal (S-0210 claim 4, S-0414 claim 21) | `claims_s0210.rs::a_crush_credits_the_kill_to_the_crusher` |
| #184 | A deployed simple-deployer's turret turned back to rest | fixed (07102b4) | The rest rows require that the vehicle is not deployed (S-0206 claim 9) | `claims_s0206_deployed_turret.rs::a_deployed_simple_deployer_turret_keeps_its_facing` |
| #186 | A frightened fraidycat at rest did not flee | fixed (4501630) | `infantry::fear_turn` has the flee row, tested on the decayed level (S-0107 claim 15, S-0208 Terms) | `claims_s0107_flee.rs` (`a_resting_frightened_fraidycat_flees`, `the_flee_rule_tests_the_decayed_level`); `claims_s0208.rs::panic_does_not_change_speed` starts the walk before the fear |
| #187 | A blocked walker's give-up test measured to a high-bridge destination at terrain height | fixed (e2e128d) | A cell destination takes `walk::cell_destination_height` (S-0209 claim 8, F1; S-0208 claim 18, F9) | `claims_s0209.rs::stationary_friendly_close_to_a_bridge_destination_gives_up` |
| #204 | The other occupants of a mission-target building's cell kept their own rows | fixed (c2dfd58) | Row WP4: when the cell lists a building that is the walker's mission target, or whose reference point (S-0204 claim 4) lies in the mission-target cell, every other occupant not being erased gives none (S-0209 claim 4, C17) | `claims_s0209.rs::walker_mission_target_override` (C17 (a) and (b)) |

Lanes, base 946161b against the batch on 0b3e137 (main's only code change in between is the client map picker; the final rebase onto a8fbe56 brings in spec files only). Oracle arena: the failing counts are base's (M-0001 9 of 96, M-0002 4 of 24, M-0003 2 of 35, M-0019 0 of 6, M-0020 0 of 25, M-0021 10 of 50; M-0024 and M-0025 are not in `--record all`); one M-0001 game ends on another frame (LTNK against GGI, side A, seed 4: 912 → 1216). The M7a and M7b sweeps over seeds 1–16 are identical to base. The M6c sweep passes; seeds 1, 4, 10, 12, 13 and 14 change (seed 4 runs 8669 → 9257 ticks and seed 10 12000 → 9079), and its smallest minimum over the 16 seeds goes 1 → 0 (seed 4's support-beam counter, 2 → 0); not traced. In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) 13 games are the same and 22 end on another tick; one changes winner (seed/map/game, base → batch): 10/8/1 house 0@82303 → house 1@118683. Traced: the games are identical through tick 54778; at 54779 house 1's APOC at (79, 91), heading north to a destination cell that carries a high bridge, brakes in the batch where base accelerates. Bisected, the build at f0b1dc8 ends as base and the one at f07e25a (#183) as the batch, so it comes from the deck-height move destination S-0207's Terms require and stands. On the final tree `xtask check` and `xtask gate` pass, the AI suite with its ignored tests passes 306 (as on base), and the sim gate with its ignored tests passes 16. The workspace tests pass but for main's `quirks::tests::every_spec_quirk_has_a_switch` (`quirks.jumpjet_crash_lookahead`, from the S-0502 revision; batch S8's).

Seams left (also in the design section): #166's interim waits for Q-3046; `crush::warn_ahead` still sends S-0210 claim 7's request in the vehicle's current layer, not the layer of the track's first cell; a walker's and a ship's flags are not part of #181; the M6c seed 4 support-beam minimum of 0 is not traced.

## Batch S8: the answers to S5's requests

Base main a8fbe56, rebased onto 22b658d (batches S6 and S7 and render batch R8). Save format 137 and AI game format 111, the numbers the main session gave this batch after S6's 136 and 109 and AI batch 5's 110 (b80ae8d, renumbered in 854e0f8; the full suites were not run again for the renumbering), for `AirState::handed` and `AirState::lost` (#160) and the removal of `JumpState::filtered` (#190); wire version unchanged (12). Goldens regenerated on the rebased tree (3deb6df; all seven moved). New quirk switch `quirks.jumpjet_crash_lookahead` (e69b8b5), which S-0502's revision names; `quirks::tests::every_spec_quirk_has_a_switch` passes again. Failing tests first (6b856e0, fcef404, 9283459, fb750b9); each fails on the commit before its fix. Request Q-3066 (c70735a) for the arena row #191 moves. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S8: the answers to S5's requests)*.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #160 | S5's interim gave no flight point for a destination object out of play; Q-3041 asked whether the aircraft keeps the last point | fixed (7f68d30) | S-0501 r183 claims 8, 40 and 53: `AirState::handed` holds the object's heading point taken at each hand-over, and `flight_point` reads it; `air::destination_lost`, at the end of `held::holders_drop`, hands over a destination not yet handed, then clears it with no destination assignment and keeps the point as the flight's own (`own_point`), marking `AirState::lost`; *Move* falls back at its next processing for a loss before its second processing and flies on after it; claim 4's descent goes to the cell of the flight point the update flies to | `claims_s0501.rs` (`claim_53_a_strafer_whose_destination_object_is_removed_flies_on_to_its_point`, `claim_53_ac37_a_a_strafer_flies_on_to_its_killed_targets_point`, `claim_53_ac37_b_a_move_that_loses_its_destination_after_the_second_processing_flies_on`, `claim_53_ac37_c_the_point_is_the_heading_point_at_the_hand_over`, `claim_53_a_move_that_loses_its_destination_before_the_second_processing_falls_back`); `claims_s0501_r120.rs::ac4_run_1_landing_where_it_is_starts_in_the_cell_and_does_not_steer`; `claims_s0501_r132.rs::ac4_run_4_given_no_destination_over_its_flight_points_cell_the_landing_goes_on` |
| #189 | A crash held up by a listed object raising the reference; Q-3042 asked | fixed (22d40d2) | S-0502 r24 claim 18 and S-0312 r77 claim 16: `fly::start_crash` makes one ordinary empty destination assignment (infantry with a commitment reroutes with its ranged draw); F4's ground hold uses *u*<sub>h</sub>; the held-up crash is kept, and `quirks.jumpjet_crash_lookahead` off makes an ordinary crashing jumpjet use the direct reference | `claims_s0502_r24_crash.rs` (6: J31 parts a, b, c and e); `claims_s0502_crash.rs::claim_18_crash_start` |
| #190 | S5 kept a destination the own-cell filter ignored and guarded it with `JumpState::filtered`; Q-3043 asked | fixed (984bcdc) | S-0502 r24 claim 23 and S-0213 r106 claim 21: the ignored assignment has no effect at all; `infantry::destination_assigned` and `jump::take_destination` put back the order-level destination held before; `JumpState::filtered` and the retry's test of it go | `claims_s0502_r24_own_cell.rs` (4: J32 parts a to c); `claims_s0502_r6.rs::claim_23_infantry_own_cell_order_is_ignored`; `claims_s0502_land.rs` sets the order-level destination its setups assume; `claims_s0502_own_cell_in_flight.rs` removed |
| #191 | S5's interim aimed at a settled switched-off hover's position; Q-3044 asked | fixed (d726cea) | S-0506 r41 claim 14: a hovering vehicle's heading point *P* whenever it holds one, power, throttle and *moving* aside; its position when it holds none. The oracle arena's ROBO against V3 row stops reproducing; Q-3066 asks why | `claims_s0506_hover_heading_point.rs::rocket_aim_at_a_switched_off_hover_is_its_heading_point` |

Lanes, base a8fbe56 against the batch on that base before the rebase (530755c with the formats commit; the rebase brings in batches S6 and S7 and render batch R8, so the final tree's games differ from both). Oracle arena: M-0003 failing 2 of 35 → 3 of 35. ROBO against V3 goes from 0/12/0 (the stock record, restated by M-0016) to 0/0/12: every game runs to the cap and no robot tank is hit. Traced (seed 0, ROBO on side A, a temporary probe of `oracle_trace`, not committed): the front column of tanks chooses *P*, the centre of the next cell toward the launchers, in the first tick and is switched off in that tick; each rocket aims at *P* and detonates at its nose point below the surface (S-0506 claim 19), beyond the spread radius from the settled tank (S-0303 claims 3 and 7). Base aimed at the position, whose detonation reached the tank by a small margin. It comes from #191 (the only commit of the batch that changes the aim), and S5 saw the same row break under the same rule. The code follows S-0506 r41, and Q-3066 asks which part differs from the game. M-0001 9 of 96, M-0002 4 of 24, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 are base's, row for row. The M7a and M7b sweeps over seeds 1–16 are identical to base. The M6c sweep passes; seeds 1, 4, 8, 9, 11, 13, 14, 15 and 16 change (seeds 11 and 16 now end at 8648 and 6977 ticks), and its smallest minimum goes 1 → 0 (seeds 4, 11 and 16: support beams, charger orders); not traced. In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) 15 games are the same and 21 end on another tick; three change winner (seed/map/game, base → batch): 8/0/2 house 1@37772 → 0@29489, 8/8/1 1@55103 → 0@65556, 9/2/3 0@59711 → 1@63631. Bisected: built at 984bcdc (#191 and #190) seed 8 ends as base; built at 22d40d2 (#189) 8/0/2 ends as the batch and 8/8/1 changes winner (0@59007), while 9/2/3 still ends as base (0@68509), so 8/0/2 and 8/8/1 come from #189's crash start and 9/2/3 from #160's hand-over (7f68d30). Both are rules the revised specs require, so they stand. The AI suite with its ignored tests passes 306 before and after. On the rebased tree the workspace tests pass (run before the renumbering; no test names a format number). At the end, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` and `cargo xtask check` pass (a citation that rustfmt had moved off its literal is put back in 5e3adf9); `cargo xtask gate` passes native and wasm32 on all goldens.

Seams left (also in the design section): Q-3066 (#191's arena row); claim 53's other ways of leaving play, its revert paragraph and the earlier hand-over kept through claim 8's exception are `unmeasured` and have engine tests only; S-0502 claim 18's open question; `rocket::heading_point_with` has no aircraft or jumpjet arm, so AC37 part b's *Q* is tested with the point in the air; the M6c minimum of 0 is not traced.

## Batch P1 (sim perf)

Base main 22b658d, not rebased. Performance only, with no behaviour change: the sim goldens, the oracle arena's output files, the M6c, M7a and M7b sweeps (seeds 1–16) and the tick-bench state hashes (synthetic, and the map run's per-tick hash file) are byte-identical to base. Save format 136, AI game format 109 and wire version 12 are unchanged; nothing new is saved or hashed. No requests. The figures are interleaved base/after runs on a machine shared with other builds (load average 25 to 34 during the map runs), so a single row moves by some 10 %; where a fix has no probe row of its own the table says so rather than give it a share of a phase.

| Issue | Finding | Status | Fix | Before → after | Tests |
|---|---|---|---|---|---|
| #405 | Object turns cloned the whole object (2.8 KB) each call | fixed (7bc6abe) | Emitters, heal, acquire, behave, fire, turret, infantry fear and walk borrow the object and copy the fields they read, where the clone was taken | Map probe, mean µs a tick: turn:Building 403/404 → 234/226, turn:Infantry 220/217 → 123/119, turn:Vehicle 111/111 → 59/58 (with #403) | existing (no new logic) |
| #403 | Type records (unit, weapon, combat) cloned on every turn | fixed (7bc6abe) | Borrowed from the shared rules (`rules_arc`); `veterancy::type_combat_in` | With #405 above | existing |
| #390 | The hit path cloned the target, its type and the warhead | fixed (7bc6abe) | `damage::apply_hit`, `killing_hit`, `reactions`, `terrain_hit` and `area::detonate_in` / `hits` borrow and copy | Not isolated: no probe row of its own; inside the turn and phase rows | existing |
| #391 | `drive::tick` cloned the vehicle and its unit type, and its helpers the vehicle again | fixed (7bc6abe) | `tick`, `update_rho`, `set_pos`, `begin_track`, `step_check` and `spend` borrow and copy | Movement: map 448/445 → 257/251; synthetic 42/45 → 14/14 | existing |
| #396 | Projectile launch, flight and detonation cloned weapon, projectile and warhead records; `retaliate` cloned two objects; fresh ids entered with `enter` | fixed (4795122; `retaliate` in 7bc6abe) | Borrowed records; `enter_new` for ids from `World::new_id` | Not isolated | existing |
| #394 | Particle and system types cloned per particle | fixed (bbf9ad6) | `sys_type` and `ptype` return borrows; the system turns copy the fields they read. The particle lists of `smoke_turn` and `fire_turn` stay cloned: their moves read the world | Not isolated | existing |
| #388 | `CombatGeneral` cloned in `bridges::area_attempts` and `blow::explosions` | fixed (dc4545d) | Two fields copied; the rules borrowed | Not isolated (rare path) | existing |
| #399 | `SwType` cloned in `charge::complete` and the grants; the storm cloned each cloud | fixed (cc83b58) | The fields read are copied; the cloud is read in place | Not isolated (rare path) | existing |
| #402 | A removal walked the object table nine or more times, and cloned the object | partly fixed (81fb968, ab34faf) | `tech::object_ends` works the answer out only for a reachable limit above 0; `spawn::target_gone` finds both lists in one walk; `remove_with` copies the fields it reads. Left: the other hooks' walks (`held`, `bomb`, `linked`, `lift`, `settle_waiting`, the attack-move memories). No existing registry proves them empty (for example `w.spawned.launcher` empty does not rule out a spawner whose slots have all emptied); skipping them needs an index or counter, new state | One removal (synthetic, release): 2,025 objects 68.3 → 45.2 µs, 4,096 objects 168.3 → 117.3 µs | existing |
| #404 | The defeat test counted every counted building in full | fixed (cbeb0d8) | Short game: stops at the first. Normal: with the on-map sum m above 0 the house passes, at 0 it passes exactly when a counted building exists, below 0 *S* is counted in full (S-0414 claim 3; wrapping sums) | Houses with #395: 185/192 → 86/82 | `claims_s0414.rs::claim_3_test_reads_the_whole_sum` |
| #392 | `owns_dock` walked every object | fixed (6c1d3d3) | The house's acquired list is tried first with the same test; the walk still decides when none passes | Not isolated | `claims_s0402.rs::claim_13_dock_test_does_not_depend_on_the_acquired_list` |
| #395 | A recheck walked the objects one or more times per registered type | fixed (81fb968) | One walk per recheck builds the house's on-map counts by type and its labs' secret types (derived, dropped after the recheck); `limit_answer` skips the owned count for the no-limit value | One recheck (the issue's test, release): 100 types and 1,000 objects 516 → 40 µs, 400 and 4,000 15,085 → 206 µs | `claims_s0407.rs::claim_1_recheck_answers_match_the_answer` |

Lanes, base 22b658d against the batch. Synthetic tick bench (2000 ticks): mean 144/154 → 61/62 µs, median 126/140 → 52/52, worst 4192/1874 → 1340/1371; state hash 3a025dbb… on all four runs. Map bench (map 9, seed 7, 8 players, 11399 ticks): mean 7076/7006 → 5745/5545 µs, median 6559/6946 → 5348/5191, p95 12966/12301 → 10928/10718, ticks over 16.7 ms 124/51 → 51/31; final hash 0651000c… on all runs, and the per-tick hash file identical. Oracle arena (`--record all`, two threads): every output file identical; wall time M-0001 1326 → 1153 s, M-0003 388 → 268 s, M-0020 110 → 65 s. The three sweeps print the same lines as base. On the final tree the sim gate with its ignored tests passes and its goldens are unchanged, `xtask check` and `xtask gate` pass (the wasm32 gate matches the goldens), the AI suite with its ignored tests passes 306, clippy with `-D warnings` and `cargo fmt --check` are clean, the wasm32 build of `glasnost-app` succeeds, and the workspace tests pass 8470 but for main's `quirks::tests::every_spec_quirk_has_a_switch` (`quirks.jumpjet_crash_lookahead`).

Seams left: the removal walks listed under #402; the particle-list clones of `smoke_turn` and `fire_turn`; `Track::points()` still allocates on each drive tick; the cloned `Projectile` in `projectile::turn`.

## Batch S10: the nose-point test, owner changes, crates and sight

Base main a8896e9, not rebased. Save format 137, AI game format 111 and wire version 12 are unchanged: nothing new is saved or hashed. The lead's answer to Q-3066 (S-0506 r43) comes first, then eight issues. Failing tests are committed first (7d58762, 6e0f3cc, 76c6497, e18c32d, 7361a54, f0c684f, 974c673 corrected in 2f01a3d). Each issue's new test fails on the commit before its fix, apart from the guard tests the table names, and no fix commit changes a test. #205's test was corrected in its own commit before the fix: the first version's arrival check was wrong, because a drive update can reach a cell centre partway through a tick and go on. Requests Q-3085 (7d4925a), Q-3086 (15e4794) and Q-3087 (a1e26d7) are filed. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S10: the nose-point test, owner changes, crates and sight)*.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-3066 | S-0506 r41's claim 19 tested the height the next move would reach; r43 tests the nose point | fixed (d519636) | S-0506 r43 claim 19 and F9: in lazy cruise and descent, the rocket detonates when its nose point's world height, trunc(*z* + *L* × sin *p*) from the position and pitch before the move, is at or below the aim height (or the 416-lepton deck over a high bridge); `rocket::nose_height` serves both the test and `nose_b` | `claims_s0506.rs::claim_19_lazy_flight_ends_when_the_nose_reaches_the_aim_height` |
| #215 | After the current object left play in its own turn, a removal of the object that then held its index counted, so one more object missed its turn | fixed (efb0259) | S-0203 claim 10: `UpdateOrder::leave` counts a removal only before `cur`, or the current object's own while it is still there | `update.rs` tests `current_leaves_then_its_follower_leaves`, `follower_leaves_then_the_current_object_leaves` (fail before the fix); `current_leaves_then_an_earlier_object_leaves` guards the earlier-object rule and passed before |
| #169 | S-0302 claim 12 and D23 (e) keep a permanently disguising unit's disguise on a surviving hit; S-0516 claims 32 and 34 (a) and CS13 (c) reset it to the side default | open: Q-3085 (7d4925a) | None; the code follows S-0302 | none |
| #206 | A spawner that changed house through a contact (a captured building's contacts) or trigger action 14 kept its slots | fixed (d8ea1da) | S-0506 claim 13 with S-0509 claim 14 and S-0011 claim 24: `structures::ownership::contacts` and action 14's non-building branch call `spawn::owner_changed` | `claims_s0506_owner_change.rs` (`a_spawner_that_follows_a_captured_building_clears_its_slots`, `trigger_change_owner_clears_a_mobile_spawners_slots`) |
| #216 | Trigger action 14 changed the owner of a carrier whose destruction had begun | fixed (bdde1c6) | S-0011 claim 24 and Terms (*present object*): `triggers::present_object` leaves out a killed object with no health left and a sinking vessel (S-0312 claims 10 and 29); a delayed-kill building stays present (S-0302 claim 16) | `claims_s0011_present_object.rs` (`a_poll_leaves_the_owner_of_an_infantry_carrier_showing_its_death`, `its_own_destruction_raise_leaves_the_owner_of_the_dying_carrier`) |
| #192 | A unit crate that placed its unit still started the unit power-up's animation | fixed (c2b2f86) | S-0520 claims 11 and 13: `crates::pickup` starts no animation for a placed unit; the money fallback keeps the money animation | `claims_s0520_effects.rs` (`a_placed_unit_shows_no_animation`; `a_unit_crate_that_places_no_unit_shows_the_money_animation` guards the fallback and passed before) |
| #193 | A building's crate beneath used the random death-crate placement | fixed, interim (356acf8); Q-3086 (15e4794) | S-0312 claims 20 and 21: `crates::place_beneath` puts it on the top-left foundation cell after the building has left its cells (in `leave_play`, and in the removal turn after the second round), with only S-0520 claim 1's limit and named crate overlays. Q-3086's interim: no usable-area or overlay test, and a crate already there is replaced. Rubble is not modelled | `claims_s0520.rs` (`a_crate_beneath_lands_on_the_top_left_foundation_cell`, `an_exploding_buildings_crate_beneath_lands_on_its_top_left_cell`) |
| #200 | S-0406's `WaterBound` default reads a locomotion-class key that no spec maps for a building section | open: Q-3087 (a1e26d7) | None; a building type with no `WaterBound` key is not water-bound | none |
| #205 | Sight moved when a walker's or driver's position crossed into a new cell | fixed (539a7c2) | S-0409 r9 claim 8 and Terms, S-0207 claim 12: `walk` and `drive` call `shroud::refresh_sight` at the movement arrival update (after `cell_movement_completed`); `shroud::object_turn` compares cells no more and re-applies sight on the 15-tick refresh and on an owner change only | `claims_s0409_arrival.rs::a_driving_units_sight_moves_at_its_arrival_not_at_the_boundary_crossing` |

Lanes, base a8896e9 against the batch on a7ef0c3. Oracle arena (`--record all`, two threads): M-0001 failing 9 of 96 → 8 of 96. CAOS against V3 now passes (ours 1/0/11 → 2/0/10, p 0.0094 → 0.0350). TELE against V3 goes 0/5/7 → 0/11/1, nearer the record's 0/12/0, but still fails its side outcome. M-0003 stays 3 of 35. ROBO against V3 goes 0/0/12 → 0/10/2 against the record's 0/12/0, so it does not pass yet, but it is near. M-0002 4 of 24, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 are base's. The M6c sweep passes. Seeds 1, 2, 3, 5, 8, 9, 10, 11, 12, 13, 14 and 16 change: seeds 8, 9, 10 and 11 now play all 12000 ticks (base 8596, 9830, 9063, 9766), seed 2 ends at 8039 (base 9490) and seed 13 at 9548 (base 12000). The smallest minimum stays 0 (infiltrations). The M7a sweep passes with every game length and the smallest minimum (0, seed 13's bunker counters) as on base; counters move in seeds 4, 6, 12, 14, 15 and 16. The M7b sweep passes; every seed but 2 and 15 changes, and the smallest drain minimum stays 3.

AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games): every game ends on another tick, and five change winner (seed/map/game, base → batch): 7/2/3 house 0@87512 → 1@58014, 8/0/3 0@74260 → 1@45232, 8/2/3 no winner@120000 → 1@31808, 8/8/2 0@24827 → 1@21876, 9/0/3 1@58013 → 0@73967. Bisected with a build at 356acf8, the batch without #205. 8/8/2 ends as on base there (0@24827) and 9/0/3 keeps base's winner (1@116281), so both changes come from #205. 7/2/3 (1@61676), 8/0/3 (1@119393) and 8/2/3 (1@55357) already have the batch's winner there, so earlier commits in the batch flip them.

Traced with the lane's state hashes and object dumps (`GLASNOST_AI_HASH`, `GLASNOST_AI_OBJS`):
- 8/2/3, base against the batch: the first difference is at tick 1764 and touches only house 0's harvester #643's applied sight. Base moved it to cell (67, 29) as the vehicle crossed into that cell; the batch keeps it on (68, 28) until the arrival (#205). The two states come back together and finally part at tick 4009, where again only an applied sight differs (house 1's walking slave miner #637).
- 8/2/3, base against 356acf8: the first difference is at tick 22588. House 0's rocket #5321 (from launcher #5320), in lazy cruise toward house 1's infantry #3380, has already detonated at 356acf8 while base still flies it. That is Q-3066's nose-point test.
- 8/8/3 (same winner; 356acf8 ends it exactly as the batch, 0@53198): the first difference from base is at tick 15898. House 0's lazy-cruise rocket #3583 detonates a tick earlier at its nose point and hits house 1's infantry #4503 (health 200 → 48). In the same tick house 1's slave miner #178 has one turn fewer (S-0203 claim 10).
- 7/2/3, base against the batch: the first difference is at tick 1760 and touches only house 1's walking slave miner #636's applied sight, cell (70, 34) on base and (71, 34) in the batch (#205).
- 8/0/3, base against the batch: the first difference is at tick 1760, on house 1's walking slave miner #1156's applied sight, cell (42, 97) on base and (43, 97) in the batch (#205).
- 8/8/2, base against the batch: the first difference is at tick 1833, on house 1's vehicle #183's applied sight. The cell is (26, 89) on base and (25, 88) in the batch, while the vehicle's cell is (25, 88) in both (#205).
- 9/0/3, base against the batch: the first difference is at tick 1760, on house 1's walking slave miner #1153's applied sight, cell (110, 62) on base and (111, 62) in the batch (#205).

So in all five games the first difference from base is #205's sight timing. The bisect shows 7/2/3, 8/0/3 and 8/2/3 flip without #205 too. For 8/2/3, the trace against 356acf8 puts that on Q-3066's nose-point test. For 7/2/3 and 8/0/3, the earlier commit that flips them is not traced. The changes traced come from rules S-0409 r9 and S-0506 r43 require. None of the traces points to a fault.

The goldens moved, hashes only, and were regenerated in 3d0e5b9; all nine moved. The AI suite with its ignored tests passes 328, as on base. On the final tree, the sim gate with its ignored tests passes 16. `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check`, the wasm32 build of `glasnost-app`, `xtask check` and `xtask gate` pass, and the wasm32 gate matches every golden. The workspace tests pass but for main's `quirks::tests::every_spec_quirk_has_a_switch` (`named_palette_true_owner`, `gas_through_bridge_deck`).

Seams left (also in the design section): Q-3085 (#169) and Q-3087 (#200) are open with no code change. Q-3086's interim for #193 stands until it is answered, and rubble (S-0312 claim 20) is still not modelled. Hover and ship units have no arrival hook, so their sight moves only on the 15-tick refresh. A tunnel exit as a rocket's aim point stays a seam. ROBO against V3 (M-0003) still fails its side outcome, 0/10/2 against 0/12/0. The M6c and M7a minima of 0 are not traced.

Merge with main 0c8d9f9 (sim batch S9, foundations F1, AI perf AP1 and AI batch 6) in a2ff008. The goldens took main's versions and were regenerated (62bbb78), and the AI game format goes to 114 (024709f).

On the merged tree the gate's M6c duel, seed 3, ends at tick 9139 with one jumpjet order. Traced against main with per-tick state hashes, the first difference is at tick 18. There the only difference is the applied sight of nine of house 1's walking infantry (objects 56–58, 61–64, 67 and 68): one row behind main's, until their arrival (#205, S-0409 r9 claim 8). On batch S10 alone seed 3 played all its ticks, so the early end comes from #205 together with main's newer changes. Nothing in the trace points to a fault.

The M6c sweep over seeds 1–16 on the merged tree:
- seeds 1, 2, 5–9, 11–13, 15 and 16 play all 12000 ticks with smallest counter 4;
- seed 10 plays all ticks with smallest counter 3;
- seeds 3 (9139), 4 (9175) and 14 (9457) end early.

`M6C_SEED` goes from 3 to 2 (59c9361), and every ability is exercised.

On the merged tree:
- the sim gate with its ignored tests passes 17;
- the AI suite with its ignored tests passes 356;
- `cargo fmt --check` and `cargo xtask check` pass;
- the workspace tests pass but for main's `every_spec_quirk_has_a_switch` (`gas_through_bridge_deck`, `named_palette_true_owner`).

## Batch S9: the answers to S6's and S8's requests

Base main a3f48b8, not rebased. Save format 138 and AI game format 113 (15212d6): no saved field changes, but what a saved game replays to does; wire version unchanged (12). The sim-gate goldens were regenerated with the ignored tests, and none of the nine golden files moved. Tests come first (c6a143e, 8f54226, a3de060, c094541, 131e100, 5a741c4). Each failing test fails on the commit before its fix; #431's pass, because the code already followed the answer. No new requests. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S9: the answers to S6's and S8's requests)* (f5389e5).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #430 | S7 built the unload rows on interim readings (Q-3062, Q-3063). In the payment row the smoke came after the payment animation; the empty-load row had no payment animation | fixed (be06b71); the S-0108 claim 13 power freeze is not built (handed back) | S-0402 r84 claim 11: the payment row is smoke, then the payment animation unless it is playing, then the payment. The empty-load row is smoke, then the payment animation unless it is playing, then a refinery-flag building's unloading animation, then *stop payment* last, then *finishing*. `depot_anim::load_emptied` starts the Production slot, then stops Special1. The fill already matched | `claims_s0402_dock_anims.rs` (`claim_11_r84_smoke_before_payment_animation`, `claim_11_r84_empty_load_row_starts_and_stops_the_payment_animation`, `claim_11_r84_damaged_with_only_normal_names_plays_normal`, `claim_11_r84_only_damaged_names_at_full_health_play_nothing`) |
| #431 | S7's finding on a floor-bound firer against an airborne target (Q-3069) | already matches (8f54226) | S-0307 r24 claim 27 (a) to (d): Δz is the target's whole height above the building's reference point, the planar reach shrinks with height, the air bonus is the only addition, and the firing check answers out of range | `claims_s0307.rs` (`claim27a_full_height_above_the_reference_point`, `claim27b_planar_reach_shrinks_with_height`, `claim27c_air_bonus_is_the_only_addition`, `claim27d_firing_check_answers_out_of_range`) |
| #422 | A harvesting slave did not play the shovel sequence (Q-3059) | fixed (a84c5ec) | S-0107 r52 claim 21a: a passing harvest processing asks for *shovel* (not forced) unless shovel is current; a failing one asks for *stand* (not forced) and queues guard; the follow-up of a completed shovel is *shovel* (forced) while harvesting (Harvest current, or none current and Harvest queued), otherwise *stand* | `claims_s0402_slaves.rs` (`s0107_claim_21a_harvest_processing_starts_shovel`, `…_filling_step_still_asks_for_shovel`, `…_no_restart_while_shovel_current`, `…_failing_processing_asks_for_stand`, `…_capacity_tests`, `…_completion_loops_while_harvesting`) |
| #168 | A sensor array placed by order activated at placement, before its grand opening (Q-3060) | fixed (691c65f); the S-0405 parts already matched | S-0516 r20 claims 23 and 40: activation happens only at the grand opening, and only if the array is in operation then. `sensor::grand_opening` is called from `deploy::process_construction` and from the opening at an owner change (`ownership::opening`). S-0405 r57 claim 8: the queued behaviour is read while none is current, and a sell order ends operation at once | `claims_s0516_coverage.rs` (`claims_23_40_r20_placed_by_order_never_covers`, `claims_23_40_r20_owner_change_in_build_up_opens_for_the_new_owner`); `claims_s0405.rs` (`claim_8_r57_queued_behaviour_read_while_none_is_current`, `claim_8_r57_sell_order_ends_operation_at_once`) |
| #163 | A holder whose override went on kept a put-aside target or destination that had left play (Q-3061) | fixed (a9db2ff) | S-0309 r140 claim 27a: after claim 27's drops and reverts, a reverted holder whose restored destination is the object gets an empty destination assignment. Every holder whose override goes on then loses the object from its put-aside target and destination, with no draw and no revert (`held::aside_lost`). At claim 28's cloak moments a put-aside target is kept while the holder's house senses the cell. Claim 23 (d): the attack-move memory converts at each loss (killing hit, going off the map, removal), moved from `objects::remove_with` into `held::holders_drop` | `claims_s0309_r140.rs` (`claim_27a_put_aside_target_lost_while_the_override_goes_on`, `claim_27a_put_aside_destination_that_is_the_object`, `claim_27a_order_within_the_action`, `claim_23d_memory_converted_at_each_loss`, `claim_27a_cloak_moments`) |
| #166 | S6's interim used the receiving pier's reference cell (Q-3046) | fixed (7f74dda) | S-0511 r13 claim 6: a received notice uses no reference cell. A collapse from a notice centres on the arriving cell, or one step toward the start side at a non-body sub-tile. Claim 18: a start-piece pier is restored only from collapsed, raising at(start, f, −1). The interim's `nocite` comments are replaced by citations | `claims_s0511.rs` (`claim_18_r13_a_damaged_start_piece_pier_keeps_its_state`, `claim_6_r13_a_notice_at_a_cut_off_part_changes_that_part`, `claim_6_r13_a_notice_collapse_centres_on_the_arriving_cell`) |

Lanes compare base a3f48b8 with the batch at 15212d6.

**Oracle arena.** M-0001 goes from 9 to 10 failing of 96. LTNK against E1 goes from 0/12 to 2/12 and fails the side outcome. Five games against E1 change: LTNK A2 and B1 change winner, and LTNK B5, YTNK A0 and CAOS B2 end on another frame. Two games were traced:
- **LTNK against E1, side A, seed 2.** The first difference is at frame 492, A#432, an LTNK in a Move override that crushes B#453, the E1 it holds as its target. Base leaves it destination B#462; the batch leaves it none, and at 493 it falls to AreaGuard.
- **YTNK against E1, side A, seed 0.** The first difference is at frame 163, A#430. Its killing hit on B#460 reverts its override, and the revert gives back the put-aside destination B#460. Base keeps that destination; the batch makes the empty assignment.

The arena has no slaves, refineries, piers or sensor arrays, and gives no attack-move orders. So both games come from a9db2ff's S-0309 claim 27a, the rule the answer requires, and they stand. M-0002 (4 of 24), M-0003 (3 of 35), M-0019 (0 of 6), M-0020 (0 of 25) and M-0021 (10 of 50) match base row for row.

**Sweeps.** The M6c, M7a and M7b sweeps over seeds 1–16 are identical to base.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). All 36 games end on another tick, and 11 change winner (seed/map/game, base → batch):
- 7/2/3 0@87512 → 1@31183
- 7/8/3 1@34384 → 0@57217
- 8/0/3 0@74260 → 1@51124
- 8/2/3 none@120000 → 0@46642
- 9/0/3 1@58013 → 0@38931
- 9/2/3 0@75024 → 1@48048
- 9/8/2 1@50590 → 0@39418
- 9/8/3 1@40993 → none@120000
- 10/0/3 0@32454 → 1@55382
- 10/2/3 0@52279 → 1@55899
- 10/8/3 1@63392 → 0@43380

Every changed winner was traced with the state hash every tick, then an object dump at the first differing tick. Each first parts at side 2's house (house 1), whose harvesting slaves (type 61, behaviour Harvest) enter *shovel* in the batch where base is in *stand* or *fire standing*: at ticks 2007, 2132, 2058 (state hash 2057), 2208, 1997, 2007, 2148, 2147, 1997, 2007 and 2147 respectively. That is #422's S-0107 claim 21a, and they stand. The games of sides 0 and 1 have no slaves and also end on other ticks, but none changes winner; not traced.

**Suites.** The AI suite with its ignored tests passes 328 before and after. The sim gate with its ignored tests passes 16 against the unchanged goldens. The workspace tests pass 8529 and fail 1, `quirks::tests::every_spec_quirk_has_a_switch` (below). `cargo clippy --workspace --all-targets -- -D warnings` passes after 583706e, which names a type clippy found too complex. `cargo fmt --check`, the wasm32 build of glasnost-app and `cargo xtask check` pass. `cargo xtask gate` passes native and wasm32 on all nine goldens.

Seams left (also in the design section): the S-0108 claim 13 power freeze (part of #430, handed back); S-0516's trigger-created array and going underground are not modelled. `quirks::tests::every_spec_quirk_has_a_switch` fails here for `quirks.named_palette_true_owner`, which S-0516 r20 claim 37 names for drawing a disguise (S-0103 claim 16). The app registers it in `mapview/quirks.rs`, but the rules crate neither has it as a switch nor lists it in `EXTERNAL_QUIRKS`. On main the test also fails for `quirks.gas_through_bridge_deck` (S-0613 r22, render batch #424). Neither is this batch's: claim 37 is presentation, outside #168's claims 23 and 40.

## Batch F1 (rules, num, map)

Base main 73fce39, not rebased. No format changes: save format, AI game format and wire version stay as on base, and the goldens do not move. Failing tests first: 979cb8e (#293), 16f3121 (#295), 280e08a (#296), acfa97f (#294, with a `direction_real_b` stub so it builds), 2364509 (#297), a90ffe7 (#288), 37fbab4 (#289), 2283a9d (#286), 3775368 (#287), f858763 (#329), a0ffb93 (#330), 12836fa (#328), e5cb567 (#331), 623c952 (#325), 60e7e9d (#323); #326 and #332 are documentation only and have no test of their own (the schema test checks the regenerated schema). Request Q-3082 (4a9c11d). #324 is not done (see the end of this section).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #293 | A versus item written with `%` loaded, and its percent was never applied | fixed (1d9abd8) | `num::decimal::exact_double` refuses a `%`; `validate` refuses such an item under `sim.warheads.{id}.verses` (our format: the importer writes each item as the exact decimal of its double, S-0003 claim 13); `nearest_double` reads text as before | `verses_items.rs`; `decimal::tests::exact_double_refuses_a_percent` |
| #295 | The table root's 64-bit integer conversion lost the carry to 2^64 | fixed (205458a) | `single_of_integer` works in 128 bits (S-0204 F1) | `table_root_carry.rs` |
| #296 | The table index's *I* was converted by a rounding of our own | fixed (d9b1012) | `to_i32_common`, S-0205 claim 7's common conversion (S-0105 F1) | `table_index_conversion.rs` |
| #294 | A real displacement's direction always went through the table, whatever `quirks.table_direction` said | fixed (f5849e8), interim for two cases | `geom::direction_real_b`: with the switch off a finite non-zero displacement takes the exact direction (S-0204 F3 and Edge cases); a zero or non-finite one keeps the table path until Q-3082 answers; the projectile callers pass the switch | `direction_real_exact.rs` |
| #297 | `trunc_i128` gave up above 2^70 | fixed (1c19184) | Cut-off at 2^127 as documented (S-0306 Formulas) | `trunc_i128_range.rs` |
| #288 | A slope code above 20 indexed past the shape table | fixed (cc4f8e2) | `TileCell::shape` reads it as flat; the importer warns (S-0106 Edge cases) | `slope_code_range.rs`; `theater::tests::a_slope_code_above_20_is_warned_about` |
| #289 | The floor below level 0 was raised whatever `quirks.negative_level_floor_raised` said | fixed (41460d1) | `Cell::floor_height(raised)` takes the switch (S-0009 claim 9, S-0106 Edge cases); `claims_s0410.rs` passes it | `floor_height.rs` |
| #286 | Ramp smoothing ran with an unset ramp role under `quirks.unset_role_range` off | fixed (88ec6b4) | `smooth_ramp` returns the tile when either role is unset and the switch is off (S-0010 Edge cases) | `unset_ramp_roles.rs` |
| #287 | Ground smoothing used a pair with an unset ground role under the switch off | fixed (800cf62) | `smooth_ground` skips the pair (S-0010 Edge cases) | `unset_ground_role.rs` |
| #329 | `"inherits": null` was an error, not a removal | fixed (10f2e88) | A null `inherits` means no parents (PLAN.md 4.6: `null` removes) | `inherits_null.rs` |
| #330 | Inheriting from an instance a layer removed, or from a non-object, went unreported | fixed (e52f02a) | `resolve_instance` refuses a removed parent and a parent that is not an object | `removed_parent.rs` |
| #328 | A parent's set field written as a set operation reached heirs as the raw operation | fixed (ebb4464) | `overlay` overlays a member the base lacks onto an empty value, so the operation becomes its list before heirs read it | `inherited_set_operation.rs` |
| #331 | `quirks_with` ignored a non-object `globals.quirks` and failed on a null switch | fixed (aacdf46) | It reads the layer as the pipeline does: a non-object `globals` or `quirks` is an error, a null switch takes its default, a null `quirks` or `globals` resets every switch | `quirks_layer_reading.rs` |
| #325 | An own `weapons` or `warheads` collection replaced the imported registry of that name in `Ids` | fixed (0a745d1) | `Ids::kinds` keeps the imported registries and `Ids::own` the own collections (`get_own`); the doc says own-collection IDs follow first appearance, which `sim_rules_hash` does not cover, and must not enter simulation state, saves or the wire as they are | `interned_ids.rs`; `synthetic_ruleset.rs` reads through `get_own` |
| #323 | Five imported keys stored twice could be set apart by a layer | fixed (835f3e9) | `validate` refuses a ruleset whose two copies of `Cloning`, `Naval`, `LightningWarhead`, `WeatherConBoltExplosion` or `C4Warhead` differ (S-0214, S-0404, S-0308, S-0303, S-0513, S-0302, S-0414 Import mapping); the stock import passes (arena unchanged); the duplicate fields themselves stay | `duplicated_keys.rs` |
| #326 | `number_of_docks` was documented as absent 0 | fixed (f7a98ea) | Doc and schema say 1 (S-0407, S-0404 Import mapping); the consumers already read 1 | none (documentation) |
| #332 | Three read switches were documented as not consulted | fixed (0ace922) | Sentences removed from `quirks.rs` (edited in place; the generator its header names is not in the repository), schema regenerated, `docs/dev/simulation.md` corrected | none (documentation) |
| #324 | `quirks.decimal_conversion_rounding` (S-0003) has no field and no off path | not done | Needs the switch in `rules::Quirks` and `ini::Quirks`, a correctly rounded path in `num::decimal` and the importer's `decimal_to_binary`, and the setting passed at every `Decimal::value` and `single_value` caller in the simulation; out of this batch's scope (small forced sim edits only). A field the simulation does not read would be a switch that is accepted and ignored | none |

Lanes, base 73fce39 against the batch (arena and quick lane from a release `glasnost-headless` built at 835f3e9's tree; the later commits change documentation only). Oracle arena: every output file is byte-identical to base (M-0001 9 of 96, M-0002 4 of 24, M-0003 2 of 35, M-0019 0 of 6, M-0020 0 of 25, M-0021 10 of 50 failing, as base). The M6c, M7a and M7b sweeps over seeds 1–16 are identical to base. The AI suite with its ignored tests passes 328, as base. In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) every game ends on the same tick with the same winner; only run times differ. No stock game changes, so there is nothing to trace: #296's and #295's conversions and #288's range give the stock data the values it had, and the other fixes act only with a switch off or on rulesets the stock import does not produce. On the final tree `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` (after the rustfmt commit babe2db), `cargo xtask check`, `cargo xtask gate` (native and wasm32, every golden matching) and the wasm32 build of `glasnost-app` pass, and the sim gate with its ignored tests passes 16. The workspace tests pass but for base's `quirks::tests::every_spec_quirk_has_a_switch` (`gas_through_bridge_deck`, S-0613 r22, and `named_palette_true_owner`, S-0516 r20; other batches' switches), whose list this batch does not lengthen.

Seams left: Q-3082 (#294's zero and non-finite displacements with `quirks.table_direction` off); #324 as above; the own-collection IDs of #325 are not derived from an order the hash covers; #323 checks agreement but keeps both copies of each key.

## Batch P2 (sim perf)

Base main a1a1a90, not rebased. Performance only, with no behaviour change: the sim goldens, the oracle arena's output files, the M6c, M7a and M7b sweeps (seeds 1–16) and the tick-bench state hashes (synthetic, and the map run's per-tick hash file) are byte-identical to base. Save format 138, AI game format 113 and wire version 12 are unchanged. Two derived indexes are added to `SimRules` (#407, #400); they are rebuilt with the rules and never saved or hashed, and fall back to the old scan when the table they index has grown since the build. Nothing else new is kept. No requests. Each fix that adds logic (an index, a clip, an early exit, a deferral, a gathered set, a streaming hash) has a test that fails when that logic is broken: a clip one ring too tight, no backward look, a first look at one designator only, a cover test that always answers no, a hash that drops single-byte writes. The figures are interleaved base/after runs, one at a time, on a machine shared with other builds (load average about 10 for the map pairs quoted), so a row moves by some 10 %; where a fix has no probe row of its own the table says so rather than give it a share of a phase.

| Issue | Finding | Status | Fix | Before → after | Tests |
|---|---|---|---|---|---|
| #406 | Tick code parsed decimal constants with `nearest_double` on every call | fixed (abfe3a0, 96f23c6) | `consts` entries hold each double's bit pattern, with the citation of its call site | Not isolated: spread over the phases that read them | `consts::tests::per_tick_decimals` (every pattern against the parse of its text) |
| #407 | Every animation spawn searched the animation types by name, case-folded, in a linear scan | fixed (fb063ae, ca87381) | `SimRules` maps each folded name to its first index (derived, not saved); `anim_index` falls back to the scan when the list has grown since the build | Not isolated | `rules::tests::anim_index_matches_linear_search` (duplicates, a type added after the build) |
| #400 | A trigger type event scanned every type name four times, once per kind | fixed (2c824e8, 3e909bf, 806a8d7) | Exact names: one key look-up per kind. Folded names: a per-kind map `SimRules` builds from the names table, keeping the last name in names order, so the scan's answer; scan fallback when the table changed | Not isolated (trigger phase 0.2 µs a tick on the map run) | `triggers::tests::named_type_matches_the_scan_of_every_name` (both modes) |
| #408 | A range or area search worked out the searcher's healer flag, friendly-row exceptions and controlled carrier per cell, and copied each cell's object list | fixed (02f83e4) | Worked out once before the rings; cell lists read in place | With #401 and #254: map probe, Behaviour mean µs a tick 640/639 → 524/526, share 12.9/13.0 % → 10.8/10.8 % | existing S-0309 claim tests (healer, attack-friendlies, allied first object, one object per cell) |
| #401 | Range searches and the fire-suppression count walked rings past the map edge up to an uncapped range | fixed (a131d98) | `Cells::ring_limit` (S-0009 claim 2's coordinate square); the loops stop after it, keeping the uncapped *n*; `search_ring` yields its cells in F2 order instead of building a vector | With #408 above | `cells::tests::every_legal_cell_lies_within_the_ring_limit`; `claims_s0309.rs::a_range_far_beyond_the_map_reaches_its_farthest_legal_cell`, `…::suppression_far_beyond_the_map_reaches_its_farthest_legal_cell` |
| #386 | The harvest ring search visited the whole (2k + 1)² square of each ring, and rings past the map edge | fixed (e3dbf8b) | Walks only the ring's own cells in the same order, and stops after `Cells::ring_limit`'s ring (each caller's test rejects a cell that is not legal first, with no effect) | Not isolated | `harvest::tests::ring_search_asks_about_the_legal_cells_in_square_order_up_to_the_map_edge` |
| #254 | The approach search range-tested every primary cell, those outside the usable area included, though such a cell only matters as claim 17's failure anchor | fixed (47471d1, da3414a) | Outside the close-range infantry search, a primary outside the usable area is put off; only when the search fails are they tested, from the last back to the first put off since the last anchor. Each candidate is tested at most once | With #408 above | `claims_s0307.rs::claim_17_failure_anchor_outside_the_usable_area` (committed first) |
| #409 | The strike sweep collected every designator into a vector and looked at each after every turn | fixed (84580f4) | A first look, without allocating, whether any designator's target or change count differs from what its entry watches; none: return | StrikeSweep mean µs a tick 17.8/18.1 → 17.2/17.1 (within noise) | `claims_s0519_r3.rs::the_sweep_finds_a_change_behind_unchanged_designators` |
| #387 | `air::ty` rebuilt the aircraft type record (dock names cloned and looked up, the primary weapon looked up by name) five to seven times per aircraft update; the update cloned the whole object three times, guidance once more | fixed (57357ce) | `tick_flight` builds the record once and hands it to `flight_point`, `point_from`, `heading_point_for` and the facing requests through guidance and the landing trigger; `tick_flight` and guidance copy the fields they read instead of cloning the object. No cache in `SimRules`: tests and lanes change cloned rules after the build, which a per-type cache would miss | Not isolated (turn:Aircraft 1.1 µs a tick on the map run, under one aircraft a tick) | existing (no new logic) |
| #389 | With `quirks.cloak_field_overlap_loss` off, clearing each cell of a collapsing field walked every object and built a circle per other generator | fixed (03d966c) | The shrink and vanish passes gather the house's other generators and one circle per distinct radius once per pass | Not isolated (rare path) | `claims_s0516_quirks.rs::quirk_cloak_field_overlap_loss_collapse` |
| #398 | `Replay::run` and `run_on` never cleared `World::effects`, so the list grew for the whole replay | fixed (331d995, 1970269) | `run_on` clears the effects after calling `each`, as a drawing driver does; the skirmish lane (`glasnost-tools`) counts each tick's effects instead of the whole list | Not isolated | `scenario.rs::a_replay_takes_each_ticks_effects` (committed first; fails before the fix) |
| #410 | A save cloned the whole world to serialize it, and `state_hash` built a state-sized buffer to hash it | fixed (b15d037, 52c4eed) | `save` serializes a borrowed `SaveRef` (the same fields in the same order); `state_hash` feeds postcard's output into SHA-256 through a hashing flavor | Not isolated | `save::tests::save_and_hash_are_those_of_the_owned_bytes` (committed first; save bytes and hash against the owned form, mid-fight and after a round trip) |

Lanes, base a1a1a90 against the batch at 52c4eed. Synthetic tick bench (2000 ticks, interleaved pairs): mean 63/55 → 60/56 µs, median 50/47 → 54/46, noise at 21 objects; state hash 3a025dbb… on all eight runs. Map bench (map 9, seed 7, 8 players, 11399 ticks, interleaved pairs at load about 10): mean 4972/4927 → 4842/4867 µs, median 4629/4625 → 4480/4549. Earlier runs not interleaved (base at load 7 to 12, after at 13 to 20) are left out of the figures. Final hash c4487272… on all nine map runs, and the per-tick hash file identical. Oracle arena (`--record all`, two threads): every output file identical; wall time M-0001 1038 → 1117 s, M-0003 269 → 357 s, M-0020 62 → 84 s, the after run under the heavier load. The three sweeps print the same lines as base. On the final tree the sim gate with its ignored tests passes 17 against the unchanged goldens. `xtask check` passes after 96f23c6, ca87381 and 806a8d7, which add the citations its cite check asked for on three of the batch's lines (comments only). `xtask gate` passes native and wasm32 on all nine goldens. The AI suite with its ignored tests passes 343. Clippy with `-D warnings` and `cargo fmt --check` are clean, after 3e909bf (two `last()` calls clippy flagged). The wasm32 build of `glasnost-app` succeeds. The workspace tests pass 8568 and fail 1: main's `quirks::tests::every_spec_quirk_has_a_switch` (`quirks.gas_through_bridge_deck`). The sim gate, the workspace tests, clippy, fmt and the wasm32 build ran at 52c4eed, before the three comment-only cite commits; `xtask check`, `xtask gate` and the AI suite ran after them.

Seams left: the carrier and attacks ring branch of #408's search has no test of its own; #254's close-range infantry search is unchanged (its position-choice draws come before the range test), and F4 leaves its off-map outcomes open; `air::st` still clones the `AirState` several times per aircraft update, and `landing_trigger` and the other air paths still clone the object (#387).

## Batch S11: aircraft flight state and the behaviour tables

Base main 59674de, not rebased. Save format 142 and AI game format 118 (25c3de2): no saved field changes, but what a saved game replays to does; wire version unchanged (12). The sim-gate goldens were regenerated with the ignored tests, and only `duel-m6c.txt` moved (the M6c sweep below); the other eight are unchanged. Tests come first (7ecd750, 3b1c527, cdf083a, 58aecbf, dd5e66d, 6fa0386); each fails on the commit before its fix. Requests Q-3100 (d87a754), Q-3101 (f21c0c5) and Q-3102 (3a1cff3). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S11: aircraft flight state and the behaviour tables)* (027f92e, 0f585b1). Rustfmt commit b748b9b.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #225 | A move with no destination handed the aircraft's own landing point back as a destination at the next processing, so a stop between the two chose the point twice | fixed (bf74862) | S-0501 r183 claims 6 and 40: `air::regive` hands nothing over while the aircraft has no destination and its flight target is its own point | `claims_s0501_r183.rs` (`issue_225_a_stop_in_a_moves_first_window_chooses_the_own_point_once`) |
| #226 | A slot aircraft placed again kept the flight fields of the flight it was taken in during | fixed (edb0577) | S-0501 r183 claims 1, 8 and 40: a placement that is not a warp resets the flight fields to a freshly placed aircraft's; visit, carrier, passes, paradrop and pad heading are kept | `claims_s0501_r183.rs` (`issue_226_an_aircraft_placed_again_has_no_flight_of_its_own`) |
| #227 | `FlightLevel=-1` was read as a cruise height of −1 | fixed (4f02526) | S-0501 r183 Import mapping: −1 is the unset value, the game-wide flight level | `claims_s0501_r183.rs` (`issue_227_a_flight_level_of_minus_one_is_the_game_wide_value`) |
| #228 | A cloaked aircraft in *refiring* never asked to uncloak | fixed (8ccc7aa) | S-0501 r183 T1, S-0516 claim 11 (a): *refiring* takes the *firing* step's answer (`cloak::machine::firing_answer`) before the standard return | `claims_s0501_refiring_cloak.rs` (`issue_228_refiring_uncloaks_a_cloaked_aircraft`) |
| #229 | The attack-move starts used the start at once, where "with an immediate start" is the readiness start | fixed for the resumption (94368bf); the search is an interim (9046a28, Q-3102) | S-0215 r137 claims 7, 11 and 13, S-0309 claim 23 (b): `process::refresh_attack_move` uses `queue_and_start` with the actor's readiness. Claim 23 (a)'s search keeps the start at once: with readiness a vehicle moving in *move* keeps *attack* queued until it arrives, against S-0309 scenario A6 part g | `claims_s0309_resume.rs` (`issue_229_a_vehicle_that_is_not_ready_keeps_the_resumed_move_queued`, `issue_229_an_infantry_unit_that_is_not_ready_keeps_the_resumed_attack_queued`, `issue_229_guard_spot_row_after_a_resumed_move_that_did_not_start`); `claims_s0309.rs` (`issue_229_attack_move_destination_search_not_ready_queues_attack`, `issue_229_attack_move_target_search_not_ready_queues_attack`, both ignored until Q-3102) |
| #230 | The deployed row of *attack* required a target | fixed (1cee2c2) | S-0215 r137 claim 23: the row applies to a deployed infantry unit of a human-controlled owner (every player house, Terms) with or without a target; the neutral, special and test-only computer-controlled houses take the general rows | `claims_s0215_r137_deployed.rs` (`issue_230_targetless_deployed_jumpjet_keeps_attack`, `issue_230_targetless_deployed_unit_in_attack_searches`, `issue_230_deployed_row_is_not_for_a_computer_controlled_owner`) |
| #231 | The area-guard charging scan ran only after a failed search in the "may search" row | fixed (798b65d) | S-0215 r137 claim 26, S-0518 claim 14: `linked::start_charging` runs after the table for any actor without a target, also with no guard spot | `claims_s0518_charge.rs` (`issue_231_area_guard_scan_after_a_search_that_finds_nothing`, `issue_231_area_guard_scan_when_the_timer_has_not_run_out`, `issue_231_area_guard_scan_without_passive_acquisition`) |
| #232 | An area-fire deployer in *unload* did not take its own cell as target | fixed (dc94ab3); the named type is open (Q-3100) | S-0215 r137 claim 30: an undeployed infantry deployer whose deploy weapon (secondary, or primary for a spray-attack type) has the area-fire flag takes its own cell before its deploy request; no type is singled out by name in either position of `quirks.named_deploy_area_fire` | `claims_s0215_r137_deployed.rs` (`issue_232_area_fire_deployer_takes_its_own_cell`, `issue_232_spray_attack_deploy_weapon_is_the_primary`, `issue_232_area_fire_primary_without_spray_attack_aims_at_nothing`) |
| #233 | The own-cell shot of an immune deployed unit aimed from an empty target, the map origin | fixed (7c40d3e) | S-0215 r137 claim 31, S-0306 claim 3: the own cell is assigned as the target before the firing check; the clears after the shot stay | `claims_s0215_r137_deployed.rs` (`issue_233_the_own_cell_shot_is_launched_towards_the_own_cell`) |
| #234 | The hover-attack cell of *attack* and *guard* is not built | not done: Q-3101 (f21c0c5); the gap is noted in `process.rs` (fcf845c) | No spec gives the cell | none |
| #235 | Deployed infantry of a non-player house in *move* took the human rule | fixed (9927957) | S-0215 r137 claim 22: it undeploys when its undeploy delay is negative, otherwise takes the general rows of *move* | `claims_s0215_r137_deployed.rs` (`issue_235_deployed_infantry_of_a_computer_owner_in_move_undeploys`, `issue_235_deployed_infantry_of_a_computer_owner_with_a_delay_follows_the_general_rule`) |
| #236 | Row 6 of the vehicle fall-back missed a vehicle deploying in place and one unloading into a building | fixed (46fdc2e) | S-0215 r137 claim 15: row 6 also covers both | `claims_s0215_r137_rest.rs` (`issue_236_row_6_unload_current_and_a_deploys_into_type_changes_nothing`, `issue_236_row_6_unarmed_and_deploying_changes_nothing`) |
| #237 | `turret_was_turning` and `last_shot_target` are never read | documented (325d584); the removal is handed back | Doc comments say they are unused; removing them changes every saved state's bytes and every golden | none (documentation) |

Lanes compare base 59674de with the batch at 9046a28 (25c3de2 and b748b9b change no behaviour).

**Oracle arena.** Every pairing's result is as base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing. One game changes: LTNK against E1, side A, seed 2 ends at frame 1824 instead of 1216 with the same winner (M-0019's sample count for it goes from 4 to 6). Not traced, since the winner stands.

**Sweeps.** M7a and M7b over seeds 1–16 are identical to base. M6c passes, but seeds 1, 6, 7, 8, 13 and 14 change length and counters; seed 7's smallest counter is 0 (charger orders and support beams; on base seed 13's was 0).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). 25 games end on another tick and 3 change winner (seed/map/game, base → batch):
- 7/0/2 0@21635 → 1@44736
- 8/8/2 0@23078 → 1@38549
- 10/2/2 1@52306 → 0@54175

Each was traced with the state hash every tick, then an object dump at the first differing tick:
- **7/0/2**, tick 10937, object 2242 (house 0 infantry with an attack-move memory and a destination). Base resumes *move* at once at 10936; the batch keeps *attack* current and *move* queued, as the unit is not ready. #229's resumption (S-0309 claim 23 (b), S-0215 claim 7).
- **8/8/2**, tick 13273, object 1749 (house 0 infantry, engaged, with an attack-move memory and a destination). It falls back at 13272 from a targetless *attack*; the resumed *move* goes through the readiness start, so its processing delay and timers take other values (delay 16 → 14, timer 13273 → 13301). #229's resumption.
- **10/2/2**, tick 21215, objects 4552 and 4555 (house 0 deployed infantry, *guard* current with *attack* queued, no target and no attack-move memory, so not #229). In both runs the queued *attack* is gone at 21215, so it started and was processed in that tick, in `process_attack`, whose deployed branch 1cee2c2 changed. It now runs the deployed row (#230: deployed firing and the standard delay) where base took the general rows, so their next search moves from 21217 and 21218 to 21241.

They stand. The other changed games keep their winners; not traced.

**Suites.** The AI suite with its ignored tests passes 343 before and after. The sim gate with its ignored tests passes 17 against the regenerated goldens. The workspace tests pass but for base's `quirks::tests::every_spec_quirk_has_a_switch` (below). On the final tree `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` (after b748b9b), the wasm32 build of glasnost-app, `cargo xtask check` and `cargo xtask gate` (native and wasm32, all nine goldens) pass.

Seams left: Q-3100 (the named area-fire type), Q-3101 (the hover-attack cell), Q-3102 (the start of a moving vehicle's attack-move search; two tests ignored); the removal of #237's two fields waits for a batch that wants it. `quirks::tests::every_spec_quirk_has_a_switch` fails on base and here for `quirks.gas_through_bridge_deck` (S-0613 r22); not this batch's.

## Batch F2 (num, rules)

Base main 8e4d8ad, not rebased. Design section first (cff1e46, with a correction in 81152bb and the S-0004 switch in b426efd). Failing tests first: a8f0b4b (#294), a39093f (#324, the simulation's reading), 7783811 (#324, S-0004's attached-system switch); e940558 (#393) and 13221db (#397) pass before their fixes by construction, since the first checks that the fast path gives the old routine's results and the second that the line index gives `line_col`'s lines. #324's unit tests of the new conversions in num, ini and the importer come with c7b005f and bf21895, which they call. Request Q-3103 (4c33f3f).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #294 | With `quirks.table_direction` off, a map-plane facing's θ used the table's angle scale, and a zero displacement gave 16383 | fixed (edb99ba), interim for two cases | S-0306 r90 (Q-3082's answer): θ is (f − 16384) times the double nearest −2π/65536 in F1, F5, F6 and F11, and a zero displacement is 16384 with the switch off; the pitch's θ and c₂ unchanged. Non-finite arguments and a fragment's zero planar displacement keep their form until Q-3103 answers. Forced sim edit: `combat/projectile.rs` passes the switch | `claims_s0306.rs` (F1, F5, F11, F14; F6's lead untested, the table's quantisation hides the scale there); `direction_real_exact.rs` |
| #393 | The exact direction (`quirks.table_direction` off) computed arctan with big integers on every call | fixed (95f15bf; a1d880f lets the timing test print under clippy) | 128-bit fixed point: arctan(i/64) table plus a series, times 32768/π in Q50, error below 2^-44; within 2^-40 of a half it falls back to the big-integer routine, so every result is the same. Per call, release: `direction` 36.7 µs → 79 ns, `direction_real_b` 27.1 µs → 82 ns | `direction_exact_identity.rs` (about 14000 cases; an ignored run of about a million passes); unit tests of the table and constant against `bigmath` |
| #324 | `quirks.decimal_conversion_rounding` (S-0003) had no field and no off path | fixed (c7b005f importer, bf21895 simulation, 54b0e87 S-0004 switch, d2ea36a check) | Simulation: a field of `rules::Quirks`, on by default; with it off `resolve_ruleset` loads through `de::from_node_with(tree, true)`, which replaces each decimal's text by `num::decimal::nearest_single_text` where the correctly rounded single differs, so `Decimal::value` and `single_value` need no setting (versus items keep their text). Importer: a field of `ini::Quirks`; `binfloat::decimal_to_binary_rounded` (ties to even, subnormals, overflow to infinity) feeds Verses, the not-given tests, light thousandths, the prism modifier, the zero tests and map lighting. S-0003 names `quirks.attached_system_none_keeps` (S-0004 claim 2), which was missing: a `LayerQuirks` switch, on, and a reader of its own for the weapon's attached particle system (empty keeps; `<none>` keeps on, clears off; `none` a name while `particle_none_is_a_name` is on). The switch check reads S-0003, and its failing list is base's | `decimal_conversion_rounding.rs`; `binfloat::decimal_conversion_rounding_off_rounds_correctly`; `value::decimal_conversion_rounding_switch`; `verses_items_follow_decimal_conversion_rounding`; `not_given_values_follow_decimal_conversion_rounding`; `attached_particle_system_keeps_on_none_and_empty`; `attached_particle_system_switches_off` |
| #397 | Each provenance line counted line feeds from the start of the source, quadratic in a layer's size | fixed (281449b) | `Ctx` indexes the line starts once per source and finds a line by binary search. Loading a synthetic layer, release: 439.8 ms → 15.3 ms, and 44.05 s → 1.18 s at ten times the size | `provenance_lines.rs` (every line is `line_col`'s); `provenance_timing.rs` (ignored, prints the figures) |

Lanes, base 8e4d8ad against the batch at d2ea36a (arena and quick lane from a release `glasnost-headless` built at d2ea36a). Oracle arena: every output file is byte-identical to base. The M6c, M7a and M7b sweeps over seeds 1–16 are identical to base apart from build lines and run times. The sim gate with its ignored tests passes 17 and the AI suite with its ignored tests passes 356, as base. In the AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games) every game ends on the same tick with the same winner; only run times differ. No stock game changes, so there is nothing to trace: #294's changes and #324's decimal rounding act only with a switch off, and #393 and #397 give the same results by test. #324's S-0004 reader changes the default position: an empty value or `<none>` in a weapon's attached particle system now keeps the value, as S-0004 claim 2 says. It moves no lane row. Whether the stock data has such a value was not checked. No format moves: save format, AI game format and wire version stay as on base. The import gains two switch names in its record (`decimal_conversion_rounding` and `attached_system_none_keeps`). That record lies under `import`, which `sim_rules_hash` leaves out (`hash::is_render_path`), so it moves only the informational render hash. On the final tree `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `cargo xtask check`, `cargo xtask gate` (native and wasm32, every golden matching) and the wasm32 build of `glasnost-app` pass. The workspace tests pass 8638 but for base's `quirks::tests::every_spec_quirk_has_a_switch` (six `gas_through_bridge_deck`, S-0613), whose list this batch does not lengthen although the check now reads S-0003.

Seams left: Q-3103 (#294's non-finite arguments and a fragment's zero planar displacement with `quirks.table_direction` off); rulesets built in code (testkit, `Layers::apply_quirks`) and callers of `de::from_node` (convert's tests, the sim gate's margins) get claim 6's rule whatever the switch says; the typed attached particle system reads the common text limit, not S-0004's 19-byte cut; S-0003's importer warning where the original's result differs from the correctly rounded one is not given; #397's remaining load time still grows faster than the layer's size (about 77 times for ten times the size), so a cost other than line lookup is left (not traced).
## Batch S12: S10's answered requests, bridges, cloak and target acquisition

Base main f99453e, not rebased. The lead's answers to S10's three requests come first: Q-3085 (S-0516 r25), Q-3086 (S-0312 r78) and Q-3087 (S-0406 r13). Ten issues, #238 to #247, follow. Save format 144 and AI game format 120 (0f80024): no saved field changes, but what a saved game replays to does. The wire version is unchanged (12). Failing tests are committed first: d8c9b10, 95286cf, 37fea04, 9170982, eaa35ef, dbe6619, e27167f, 92262e6, 1e9c0b6 and 6a1ef47. Each fails on the commit before its fix, and no fix commit changes a test. #169's test is a guard test that passes on base (6f90891). #242 and #243 are documentation only. The quirk switch `crate_beneath_overlay_refusal`, which S-0312 r78 names, was added at the coordinator's request in a1d0b1b: field, schema default on and `named()` entry, with the simulation's off path in #193's fix. Request Q-3109 (12275fc) is filed. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S12: S10's answered requests, bridges, cloak and target acquisition)* (d535d41). The S-0309 claim ledger names the new tests (34865e0).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #169 | Q-3085: S-0516 r25 claim 34 (a) keeps a permanently disguising type's copied disguise, lockout and default disguise through a surviving hit | already matches (6f90891) | None: `disguise::hit_taken` returns before any loss for such a type | `claims_s0516_disguise.rs::claim_34_a_permanent_disguise_survives_a_hit` (guard; passes on base) |
| #193 | Q-3086: S10's interim put the crate beneath on the top-left cell with no usable-area or overlay test, and replaced a crate already there | fixed (f5ce45d); switch a1d0b1b | S-0312 r78 claim 20 and Edge cases: `crates::place_beneath` runs S-0520 claim 3's single-cell nearby search centred on the top-left foundation cell, then `crates::finish`'s usable-area and overlay tests; a refusal places no crate and draws nothing. `quirks.crate_beneath_overlay_refusal` off: the search asks for a cell without an overlay (`crates::search`'s `no_overlay`; the random, trigger and death placements pass false) | `claims_s0520.rs` (`claim_20_a_crate_beneath_never_replaces_an_overlay`, `claim_20_a_crate_beneath_moves_off_an_occupied_top_left_cell`) |
| #200 | Q-3087: a building with no `WaterBound` key was never water-bound | fixed (5614b5d) | S-0406 r13 claim 21: for each layer that sets either key, in order, a recognised `WaterBound` decides the flag; otherwise a `SpeedType` naming the float class sets it on and any other class off; a layer setting neither keeps it (`convert::sim_econ::water_bound`). The rules doc and schema follow | `crates/convert/src/sim_econ_tests.rs::water_bound_is_decided_per_layer` |
| #238 | A death weapon's re-attempt of the pier during the blow-ups lowered its component a second time | fixed (691b611) | S-0511 claim 6: `high::attempt_pier` and `high::pier_notice` read the reference level before the blow-ups and pass it to `lower_pier` | `claims_s0511.rs::claim_6_a_notice_collapse_lowers_the_pier_once` |
| #239 | Forgetting the hut target kept the burst counter and the attack state | fixed (c02ca58) | S-0511 claim 17, S-0301 claim 4, S-0107 claim 11: `hut::forget_and_scatter` clears through `spawn::clear_own_target`, sets the burst to 0 and runs `infantry::watch_test` for each unit that lost it | `claims_s0511.rs::claim_17_forgetting_the_hut_target_resets_the_burst_and_the_attack_state` |
| #240 | A floored sensor decrement raised a negative count to 0 | fixed (a358200) | S-0516 claim 22 and CS18 part B: `cloak::sensor::change` skips a decrement on a count at or below 0; an increment still floors at 0 | `claims_s0516_coverage.rs::claim_22_a_mobile_removal_does_not_raise_a_negative_count` |
| #241 | `CloakingStages = 2147483647` overflowed the cloaked stage | fixed (e1d1a2e) | S-0516 claim 7, F1 and F2: `cloaked_stage` computes (N + 1) / 2 in 64 bits; `cloak_time` and `uncloak_time` return i64 | `claims_s0516_machine.rs::claim_7_the_largest_stage_count` |
| #242 | `cloak/field.rs`'s header did not state claim 20's phase model | fixed (95833a4), documentation | Header text only | none (documentation) |
| #243 | `cloak/mod.rs`'s header named one turn caller and mixed hooks with seams | fixed (79af8c1), documentation; the design section in d535d41 | Header text only: both callers (`tick::object_turn`, `econ::events::turn`); hooks with callers apart from open seams | none (documentation) |
| #244 | A harvesting slave searched passively | fixed (5c94625) | S-0309 claim 2 (S-0215): `may_search_passively` refuses an object `seams::is_harvesting_slave` names | `claims_s0309_slave_gate.rs::claim_2_a_harvesting_slave_does_not_search_passively` |
| #245 | An area search did not skip a visited bridge cell by its deck read | partly (607e329); Q-3109 (12275fc) | S-0309 claim 11: an area search by a mobile non-aircraft searcher reads its own cell's deck when it is a bridge cell, and skips a visited cell whose deck read lies outside that region (claim 7's comparison, now `reach_match`); a no-link deck read drops the restriction; a range search has no skip. Which read table C's zone row uses in an area search is Q-3109; the row keeps claim 7's read | `claims_s0212.rs::an_area_search_skips_a_bridge_cell_by_its_deck_read` |
| #246 | The deployed-firing search marked its target as an opportunity target | fixed (4cae340) | S-0309 claims 1 and 24, S-0517 claim 16: `SearchContext::DeployedFiring` (claim 1's fifth row) marks nothing; `process::deployed_firing` uses it | `claims_s0517_deployed_mark.rs::claim_16_deployed_firing_search_target_is_not_an_opportunity_target` |
| #247 | A building's guard range read as 0 in its searches | fixed (c18a406) | S-0309 claim 8, F1, F3, F4 and Import mapping: `BuildingRules.guard_range` (leptons, through `combat::data::leptons`); `SimRules::guard_range` answers for every kind; `radius`, `candidate`'s unarmed test and `threat_sum` read it | `claims_s0309_building_guard.rs` (`claim_8_building_guard_range_smaller_than_weapon_range`, `claim_8_building_guard_range_larger_than_weapon_range`; the vehicle control `claim_8_control_vehicle_guard_range_larger_than_weapon_range` passed before) |

Lanes, base f99453e against the batch at 5614b5d. Oracle arena (`--record all`): every output file identical to base (M-0001 9 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25, M-0021 10 of 50 failing). The M6c, M7a and M7b sweeps over seeds 1–16 print exactly what base prints.

AI quick lane (seeds 7–10, maps 0, 2 and 8, 36 games): 25 games change. Those are every game with side 2 (sides 0,2 and 1,2) and 9/8 of sides 0,1; every other sides-0,1 game is identical to base. Ten change winner (seed/map/sides, base → batch):
- 7/0 (0,2) 0@18837 → 1@44312
- 7/2 (0,2) 1@41926 → 0@30957
- 7/2 (1,2) 1@32017 → 0@77312
- 7/8 (0,2) 1@36239 → 0@59156
- 8/8 (0,2) 1@24986 → 0@23391
- 8/8 (1,2) 0@61358 → 1@56992
- 9/0 (1,2) 0@52245 → 1@45785
- 9/2 (1,2) 1@60402 → 0@70306
- 9/8 (0,2) 0@21576 → 1@52813
- 10/8 (0,2) 1@30738 → 0@29211

The other fifteen keep their winner and end on another tick, 9/8 (0,1) among them (0@32939 → 0@27098).

Every changed winner, and 9/8 (0,1), was traced with the state hash every tick (`GLASNOST_AI_HASH`), then an object dump at the first differing tick (`GLASNOST_AI_OBJS`). In each, the only differences are three slave miner slaves (type 61, slave phase Seeking) and the RNG. On base each slave's per-tick passive search restarted its targeting timer with a draw. In the batch the timer stays 0 and no draw is taken (#244, S-0309 claim 2). First difference (game: tick, house, slaves, master):
- 7/0 (0,2): 1738, house 1, slaves 1149–1151, master 1146
- 7/2 (0,2) and 7/2 (1,2): 1738, house 1, slaves 634–636, master 631
- 7/8 (0,2): 2064, house 1, slaves 176–178, master 190
- 8/8 (0,2): 2249, house 1, slaves 178–180, master 193
- 8/8 (1,2): 2248, house 1, slaves 178–180, master 193
- 9/0 (1,2): 1738, house 1, slaves 1152–1154, master 1149
- 9/2 (1,2): 1738, house 1, slaves 635–637, master 632
- 9/8 (0,2): 2079, house 1, slaves 181–183, master 195
- 9/8 (0,1): 8280, house 0, slaves 638–640, master 702
- 10/8 (0,2): 2079, house 1, slaves 180–182, master 194

So every change traced comes from the rule S-0309 claim 2 requires, and none points to a fault. At the first differing tick those slaves hold their own master as target, with the attack flag set, on base and in the batch alike. This was seen but not traced.

The checks all ran on the final tree.
- The sim gate with its ignored tests passes 17. After the format bump the goldens were regenerated with the ignored tests, and none of the nine moved.
- The AI suite with its ignored tests passes 356, as on main.
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` are clean.
- The wasm32 build of `glasnost-app` succeeds.
- `xtask check` passes. `xtask gate` passes, and on wasm32 it matches all nine goldens.
- The workspace tests pass except main's `quirks::tests::every_spec_quirk_has_a_switch`, which fails on `gas_through_bridge_deck` only. The new switch `crate_beneath_overlay_refusal` is read.

Seams left (also in the design section): Q-3109 (#245's zone row in an area search). A building's guard range is read as S-0309's Import mapping gives it (single-precision decimal × 256, truncated), while a mobile type's still reads S-0215's (the double × 256, truncated). The two can differ for a value that a single cannot hold exactly; whether they are one read is not settled here. Rubble beneath a building (S-0312 claim 20) is still not modelled. A seeking slave miner slave holds its own master as target with the attack flag set, on base and in the batch. Whether a spec gives that has not been checked.
## Batch S14: drive, harvest, naval, hover, walker and infantry fixes

Base main 307dea1, not rebased. Save format 146 and AI game format 122 (9639a29): #274 adds a saved field (`WalkState::dest_deck`); wire version unchanged (12). The sim-gate goldens were regenerated once, after the last fix, with the ignored tests. All nine move from their first tick, because the new walker field is part of the hashed state. Each test is committed before its fix and fails on the commit before the fix. Requests Q-3115 and Q-3116 (8cd63bb). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S14: drive, harvest, naval, hover, walker and infantry fixes)* (0fe4d83). Claim ledger rows: S-0315 claim 2 and S-0504 claim 10 (6fe6007). Two commit subjects are worded wrongly and were not rewritten (no rebase): 8fc55b8 is #264's fix, though its subject begins "test for". The #271 pair (3fc7234, 376c21a) describes the defect backwards: settle cleared the path *after* the assignment, which dropped the path the hovering transport had just found. It did not leave a stale path behind. The row below gives the defect correctly.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #263 | With `quirks.path_refresh_keeps_old_step` off, the refresh at the last stored step began the old track and then kept the whole new path, so the vehicle drove both the old step and the new first step | fixed for the straight case (1e10779); the curve case is an interim (Q-3115) | S-0212 claim 20 and Edge cases: with the switch off the step check recomputes before the last step's track begins and checks the new first step instead; a failed search puts the old steps back. With the switch on one new step is still dropped, also after a two-cell curve (`design: Q-3115 interim`) | `claims_s0212_refresh.rs` (`issue_263_off_the_new_first_step_replaces_the_old_one`, `issue_263_on_a_straight_refresh_has_no_visible_effect`) |
| #264 | `crush_mode`, `attack_obstacle` and `lof::wall_overlay` indexed the overlay table with 255, so with 256 or more overlay types every empty cell read as type 255 | fixed (8fc55b8) | S-0009 claim 12: test `NO_OVERLAY` first (`walls::is_wall` for the two wall tests). `combat::area`, `econ::placement` and `cloak::disguise` keep the same shape (seams) | `claims_s0207_no_overlay.rs` (`issue_264_control_255_types_no_slowdown_on_empty_cells`, `issue_264_with_256_types_empty_cells_are_not_overlay_255`) |
| #265 | The move to a track's target ran after the track was cleared, so its cell entry read no crush-mode track | fixed (3f3f13a) | S-0210 claim 9: `drive::set_pos` passes the driven track's crush flag to `crush::entered_cell_on` | `claims_s0210_track_end.rs` (`issue_265_every_speed_enters_the_slowdown_in_the_cell_before_the_overlay`) |
| #266 | After T5's obstacle search a vehicle turned at once towards a new first step that led into an allied closed gate | fixed (2df0ae1) | S-0207 claim 15: it waits without turning, as `begin_track` does | `claims_s0207_gate_t5.rs` (`issue_266_control_no_turn_towards_a_closed_gate`, `issue_266_no_turn_towards_a_closed_gate_after_t5_obstacle_search`) |
| #267 | The direct enter of *enter* took contact at any accepting building, not only at one of the harvester's dock types | fixed (e6d3051) | S-0402 claim 2 and Terms: `of_dock_type` guards the direct enter and `track_arrival`; no owner test (no spec gives one) | `claims_s0402_enter_dock_list.rs` (`issue_267_enter_order_on_a_plain_building_takes_no_refinery_contact`, `issue_267_enter_order_on_a_dock_unload_building_outside_the_dock_list_does_not_unload`, `issue_267_control_enter_order_on_a_dock_type_takes_contact`) |
| #268 | *enter* with no accepting refinery returned a delay of 1 with no draw | fixed (10b9dbe) | S-0402 claim 10, S-0215 claims 6 and 29: the standard delay of *enter* | `claims_s0402_enter_delay.rs` (`issue_268_enter_without_a_refinery_takes_the_standard_delay`) |
| #269 | A killing hit that starts a vessel's sinking left its destination, its own target and its locomotor running | fixed (8380ebe) | S-0504 claim 10: `death::killing_hit_sinking_begins` cancels the destination, clears the own target and burst, and stops the locomotor (`drive::stop` or the hovering stop request); a vessel already sinking and a cull are left alone | `claims_s0504.rs` (`issue_269_killing_hit_sinking_cancels_destination_and_own_target`) |
| #270 | `hover::path_update` searched for the order-level destination after a switch-off's stop request had cleared *D* | fixed (24244af) | S-0503 claims 10 and 15, S-0212 claim 20: no update while the stop request has cleared *D* (`d_cleared`, shared with `moving`) | `claims_s0503_stop_search.rs` (`issue_270_no_search_on_arrival_after_a_switch_off_stop_request`) |
| #271 | `transport::settle` assigned the shore cell and then cleared the stored path, dropping the path a hovering transport had just found at the assignment, so it waited for the retry delay | fixed (376c21a) | S-0507 claim 8, S-0503 claim 5: the path is cleared before the assignment. `depot`, `structures::bunker` and `structures::absorb` keep the assign-then-clear order (seams) | `claims_s0507_water.rs` (`issue_271_hover_shore_approach_keeps_its_path`, `issue_271_hover_shore_approach_keeps_its_path_long_delay`) |
| #272 | A hovering vehicle at a tank bunker | not done: Q-3116 (8cd63bb) | S-0606 gives the arrival, stop and entry track for the driving locomotor only | none |
| #273 | A standing walker whose cell destination was its own cell dropped it without a search, also on the ground under its deck destination | fixed (5c7669b) | S-0208 claims 6, 16 and 18, S-0212 claims 7 and 12: the own-cell shortcut needs claim 16's height test; the search's own-cell refusal compares cell and layer | `claims_s0208_own_cell_deck.rs` (`issue_273_own_cell_under_a_deck_is_searched_for`, `issue_273_control_own_cell_without_a_deck_is_complete`) |
| #274 | The cell-destination height read the high bridge on arrival, so a deck repaired or lost during the walk changed claim 16's test | fixed (7b4a4fe) | S-0208 claim 18, F9: `walk::take_destination` records the cell and its deck at the assignment (`WalkState::dest_deck`); `cell_destination_height` reads the record for that cell | `claims_s0208_dest_height.rs` (`issue_274_deck_added_after_assignment_does_not_change_the_destination_height`) |
| #275 | The completion follow-up counted any movement in progress as moving | fixed (5c92ad7) | S-0107 State and ordering, Constants: moving is movement in progress with a speed fraction above 0.1 (a jumpjet's fraction, else the walker's); a unit with neither keeps the old test (seam) | `claims_s0107_followup_moving.rs` (`issue_275_completed_panic_with_movement_in_progress_and_speed_fraction_0_is_followed_by_stand`) |
| #276 | `paradrop::landing` passed `quirks.top_position_counts_as_centre` to the class it chooses by | fixed (49068b2) | S-0522 claim 9, S-0208 claims 2 and 3 and Edge cases: the choice always takes claim 2's class; the switch only changes the occupancy mark | `claims_s0522_top_point.rs` (`issue_276_switch_on_takes_the_centre_class_draw_and_a_side_position`, `issue_276_switch_off_chooses_exactly_as_with_the_switch_on`) |
| #277 | A fraidycat firing its last round from inside a transport panicked | fixed (61f618f) | S-0315 claim 2: `fear::last_round` does nothing for a firer riding in a transport; garrisons are a seam | `claims_s0315_transport.rs` (`issue_277_fraidycat_last_round_from_inside_an_open_topped_transport`) |

Lanes compare base 307dea1 with the batch at 9639a29.

**Oracle arena.** The output is byte-identical to base: M-0001 9 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps.** M6c, M7a and M7b over seeds 1–16 are identical to base (every seed's length and counters, every minimum and median).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). 14 games end on another tick and 1 changes winner (seed/map/game, base → batch):
- 9/0/1 1@34131 → 0@27983

The game without #274 (61f618f) already has the batch's result, so the state hash (comparable there, as the new field is absent) was taken every tick against base. The runs part at tick 13275, at object 2291 (house 0 deployed infantry in *attack* with an attack-move memory, no destination and walk fraction 0). Its Deployed sequence completed at 13274 with a movement in progress. Base counted that as moving, which leaves the deployed rows, and the unit ended the tick in Stand. The batch counts it as not moving, since the speed fraction is not above 0.1, and keeps it Deployed. This is #275's follow-up (S-0107 State and ordering). After that house 0's deploy orders go from 9 to 10. The change stands. The other changed games keep their winners and were not traced.

**Suites.** The AI suite with its ignored tests passes 356 before and after. The sim gate with its ignored tests passes 17 against the regenerated goldens. The workspace tests pass except base's `quirks::tests::every_spec_quirk_has_a_switch` (below). On the final tree these all pass: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` (after the rustfmt commit d639ac0), the wasm32 build of glasnost-app, `cargo xtask check` and `cargo xtask gate`.

Seams left:
- Q-3115 (the curve case of #263) and Q-3116 (#272).
- The train refresh of S-0212 claim 20.
- The unguarded overlay lookups in `combat::area`, `econ::placement` and `cloak::disguise`.
- The assign-then-clear order in `depot`, `structures::bunker` and `structures::absorb`.
- #275's units with neither fraction.
- #277's garrisons.

`quirks::tests::every_spec_quirk_has_a_switch` fails on base and here for `quirks.crate_beneath_overlay_refusal` (S-0312). It is not this batch's.

## Batch S19: build-up, bridges at load, superweapon records, storm clouds, order targeting and warp members

Base main 0d3ed80, not rebased. Save format 152 and AI game format 128 (141aa6e): #352 adds a saved field (`Cloud::late`); wire version unchanged (12). Renumbered at the merge to save format 146 and AI game format 123. The sim-gate goldens were regenerated once, after the last fix, with the ignored tests, and none moved: no golden scenario makes a storm cloud or meets a changed rule. Each test is committed before its fix (c8366e0 to ce10cbb, and d891200 for #358's value *e*) and fails on the commit before the fix; no fix commit changes a test. Requests Q-3140 and Q-3141 (b49b3e4) and Q-3142 (7e4056d). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S19: build-up, bridges at load, superweapon records, storm clouds, order targeting and warp members)* (6f78c19).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #347 | An owner change replaces the queued *construction* with guard, so a docking building handed over in its build-up accepted at once: the infantry absorber, the recycler, the repair depot's permission and the reload pad tested only the queued mission | fixed (d7c9107) | S-0605 claim 1, S-0603 claim 2, S-0314 claim 16, S-0501 claim 22: each also refuses while `econ.construction.building_up` is on, as the tank bunker's row P3 does | `claims_s0605_build_up.rs` |
| #348 | A blocked low-bridge end marker still recalculated its cell at load, taking an animated tile's draws | fixed (de943d3) | S-0202 claim 10: `end_marker` says whether it wrote its row; `place_overlay` recalculates only then | `claims_s0202.rs` |
| #349 | The slope condition of S-0009 claim 12 is not tested for a crate overlay in play | not done: Q-3140 (b49b3e4) | S-0520 claim 3 does not say what a refused crate overlay does to the crate place and its lifetime draw | none |
| #350 | A fire order on a drain-mode missile record in ready started the effect with no silo, so a type that is not one-time never launched | fixed (5b40df2) | S-0512 claim 19, S-0513 claim 7, S-0407 claim 12 j: the ready branch picks the silo (`nuke::silo_choice`) and makes the tech recheck due, as an ordinary launch does | `claims_s0512_drain_missile.rs` |
| #351 | A one-time record that also got a permanent grant kept the permanent flag after firing, so a later building grant was never lost or suspended | fixed (d0a67ee) | S-0512 claims 7, 12 and 13: `grant_by_building` clears the flag | `claims_s0512_regrant.rs` |
| #352 | A storm cloud made after the turn pass (a trigger action in the house raise) advanced a tick early with *d* = 1 | fixed (5fbbe64) | S-0513 claim 46, S-0109 claim 5: *a* = max(*c* + *d*, *u* + 1) with *u* = *c* + 1 after the turn pass; the transient `World::after_turn_pass` (not saved, false between ticks) and the saved `Cloud::late` | `claims_s0513_storm_late.rs` |
| #353 | A reset or set-charge after a first-stage source pick kept the pick's silent re-completion, so the full recharge was not announced and kept the old ready tick | fixed (c6bc6c4) | S-0512 claims 5, 10, 14 and 15: `reset` and `set_charge` clear the silent flag | `claims_s0512_silent_mark.rs` |
| #354 | Row IC7 ignored its second alternative, a cell out of range of the deployed deployer's first weapon | fixed (8736972) | S-0214 table IC row IC7: `range::in_range` of the first weapon against the cell, as row I21 does for an object | `claims_s0214_ic7_range.rs` |
| #355 | Row I19 offered *bomb* on a target already carrying one | fixed (209e95a) | S-0214 table I row I19: *bomb* only when the target accepts bombs and `bomb::carries` is false | `claims_s0214_bomb_carrier.rs` |
| #356 | The guard key gave nothing to an unarmed vehicle that controls slave units | fixed (8250d81) | S-0213 table T2: an own vehicle that is a harvester or a slave master (`seams::is_slave_master`), movable or not | `claims_s0213_guard_slaves.rs` |
| #357 | The attack-move gesture read the slot in use, so a switching type whose current slot was empty lost it | fixed on the input side (0ced81a); the application side is a seam | S-0214 claim 5 and Terms (*armed*): `targeting::eligible` reads the object's first slot (`weapon::slot_weapon`, slot 0). 0ced81a also moved `orders::attack_move_eligible` to slot 0, which broke `claims_orders::s0213_attack_move` (the plain testkit CART names an undefined weapon, so its slot 0 is empty). b8a4daf puts that half back to the type's first-slot field; 0ced81a's subject ("on both sides") overstates what stayed | `claims_s0214_attack_move_slot.rs` |
| #358 | Values *x*, *q* and *e* ignored their target-state conditions: a transport phasing in still gave *enter*, a hospital being erased still gave *no-enter* | fixed for *x*, *q* and *e* (2e57803); *n* and the engineer branch are seams | S-0214 tables I and V and Terms: *x* and *q* need a target not being erased and not phasing in, *e* a building not being erased | `claims_s0214_phasing_target.rs` (two tests) |
| #359 | A jumpjet vehicle in the air was not a free mover, so its move click searched for a cell | fixed (2845a5c) | S-0214 Terms, S-0502 claim 30: `free_mover` counts an infantry or vehicle `weapon::in_air` | `claims_s0214_free_mover_air.rs` |
| #360 | A jumpjet warp member's departure left its flying registration, landing reservation and hover slot | fixed (e782bbd) | S-0502 claims 4, 15 and 29, S-0513 claim 39: `warp::lift_off` calls `jump::leaving` | `claims_s0513_warp_jump_leaving.rs` (two tests) |
| #361 | A teleporting-locomotor unit phasing in after a warp took a destination given to it | fixed (7756e8f) | S-0207 claim 20 a and claim 27, S-0214 and S-0513 Terms, S-0513 claim 40: the drop asks `teleport::phasing_in` | `claims_s0207_warp_phasing.rs` (three tests) |
| #362 | A jumpjet that joined a warp while moving kept its point, order and moving state and took off again after its placement | fixed (658fe33) | S-0513 claim 51, S-0502 claims 5, 15 and 17: `jump::warp_join` at the join clears them and sets the target height to 0 | `claims_s0513_warp_jump_commitment.rs` |
| #363 | A teleporting infantry member's standing-position reservation at a landing point it never jumped to stayed after the warp | fixed (c1c5e34); the landing choice while pending is Q-3142 | S-0208 claim 19 (Q-0235), S-0513 claim 51: `teleport::warp_release` at the join and at departure | `claims_s0513_warp_teleport_mark.rs` (two tests) |
| #364 | `quirks.warp_naval_road_cell` off has no effect | not done: Q-3141 (b49b3e4) | S-0513 claim 38 does not say which cell is "a road cell that is water" | none |
| #365 | With `quirks.update_order_removal_skip` off a unit boarding an open-topped transport after its own turn took a second turn | fixed (0e06256) | S-0203 claims 10 and 11: with the switch off the pass skips an object whose `last_turn` is this tick | `claims_s0203_second_turn.rs` (two tests) |

Lanes compare base 0d3ed80 with the batch at 141aa6e, and every lane was run again on the final sim at b8a4daf (after #357's orders half was put back) with the same results.

**Oracle arena** (`--record all`, two threads). Every output file is byte-identical to base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps.** M6c, M7a and M7b over seeds 1–16 print the same lines as base (every seed's length and counters, every minimum and median; smallest minimum 0, 0 and drain 3).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). 1 game ends on another tick and none changes winner: seed 9 map 2 sides 0 and 2, house 1 at 48641 → 48582. It was not traced.

**Suites.** The AI suite with its ignored tests passes 362 before and after. The sim gate with its ignored tests passes 17 against the goldens, which did not move.

Seams left:
- Q-3140 (#349), Q-3141 (#364) and Q-3142 (#363's landing choice while warp-pending).
- #358: the "not being erased" of value *n*'s accepting allied building and of the engineer-friendly building's passive-neutral branch.
- A teleporting vehicle's pending reservation at a warp launch with the vehicle ground-reservation switch on.
- #357's application side: `orders::attack_move_eligible` reads the type's first-slot field, so it differs from the input side only where slot 0 is substituted (elite, upgrade, garrison).

No fix makes or moves a rarer back-reference of `World::census` (#402): #360's `jump::leaving` only ends a flight registration.

`quirks::tests::every_spec_quirk_has_a_switch` fails on base and here for `quirks.bunker_hover_no_arrival` (S-0606). It is not this batch's.

## Batch S21: crate places by cell, the flak distance, airburst arming, the exact direction's edge cases and order targeting

Base main 5d8c648, not rebased. Save format 154 and AI game format 130 (da82fba; renumbered to 148 and 125 at the merge): #284 adds two saved fields (`CrateWorld::owners` and `CrateWorld::map_crates`); wire version unchanged (12). The sim-gate goldens were regenerated once, after the last fix (da82fba); their state hashes change with the saved fields. Each test is committed before its fix and fails on base (logs in `local-lane/sim-issues/s21/`), except one of #284's eight (a map crate of another type was already removed by its pickup) and #317's spec example (it holds for d₃ too); no fix commit changes a test. #413's test fails on base and passes from #415's fix (f38ed45) on, which comes before it: under the Terms the issue's scenario keeps *attack*, and rows V5 and VC6 are not reachable yet. #416's second test (a deactivated deployer keeps *deploy*) is also met by #415's fix. #414 has no test. Three test corrections are commits of their own: 0985fad (#294, the exact direction's reference routine kept 0 for a zero displacement), a3bfbc8 (#415, a deactivated vehicle passes the deploy check in `claims_s0213_deploy_rule.rs`) and cb51f54 (#415, the rank test compared with the testkit CART, which is armed; it now uses the unarmed BUGGY). Request Q-3152 (787fbe4). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S21: crate places by cell, the flak distance, airburst arming, the exact direction's edge cases and order targeting)* (855e815, eb36801, 2c2869a).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #284 | An expiry removed whatever overlay stood on its place's cell, and a pickup freed the place the crate belonged to (Q-3118) | fixed (217ff69, a8959b4); a named crate overlay that holds no place is Q-3152 (787fbe4), removed by its pickup in the interim | S-0520 r12 claims 4, 11 and 29, Terms, Import mapping: `[CrateRules] CrateImg` is the campaign crate overlay; an expiry removes only a named crate overlay; a pickup frees the lowest occupied place on the crate's cell and removes a named crate overlay or a map's own crate. `quirks.crate_place_by_cell` (default on) off: the place's own crate only, kept in `CrateWorld::owners`, forgotten by `crates::overlay_changed` | `claims_s0520_r12.rs` (eight tests, both switch positions); `sim_crate_tests.rs` (the import) |
| #317 | Claims 4 and 9 took the distance d₃, not F17's flak distance (Q-3129) | fixed (59b37df stub, 7e4c6f2) | S-0306 r93 F17: `combat::projectile::flak_distance`, 32-bit wrapping differences, Δx and Δy narrowed to single precision, double squares and sums in the stated order, then d₃'s root. Equal to d₃ for every difference on a map | `claims_s0306_flak_distance.rs` |
| #315 | An airburst child's arm time was 0 (Q-3130); the Ω half was fixed earlier | fixed (50553c4) | S-0306 r93 claims 9 and 21: each child takes its own projectile type's arm field | `claims_s0306_airburst_arm.rs` |
| #294 | With `quirks.table_direction` off a zero displacement gave 0, and a real argument that is not a number or is infinite fell back to the table (Q-3103) | fixed (cf3354a) | S-0204 r14 Edge cases: a not-a-number argument counts as 0, then infinite arguments count as ±1 and finite ones as 0, then two zeros give 16384, for integer and real displacements | `direction_off_edges.rs`, `direction_exact_identity.rs` (reference, 0985fad), `claims_s0318.rs` (a fragment straight over its aim point) |
| #415 | `targeting::disabled` read the control-centre state, so a deactivated vehicle lost its rank, deploy check and deploy readiness | fixed (f38ed45) | S-0214 r83 Terms: *disabled* is paralysis (`econ::power::under_pulse`, a seam) or a vehicle's death sequence (a seam); *deactivated* is `econ::power::unit_disabled`. *Movable*, *able to fire* and the band box ask both; table R, the deploy check and deploy readiness ask *disabled* only | `claims_s0214_s21_deactivated.rs` |
| #413 | Rows V5 and VC6 (attack with a failed deploy check gives *nothing*) were not built | fixed as the tables give them (5b5d3aa); the issue's scenario is not a defect under the Terms | S-0214 r83 tables V and VC: V5 before V6 and VC6 before VC7. A deactivated vehicle passes the deploy check, so it still gets *attack*; no vehicle fails the check today (paralysis, the death sequence and tubes are seams) | `claims_s0214_s21_deploy_check.rs` |
| #416 | Rows VD3 and VD5 asked only for the tunnel cell | fixed (4190f15) | S-0214 r83 table VD: both ask `deploy_readiness`, as VD2 does; an in-place deployer with passengers where S-0507 claim 7 refuses an unload gets *no-deploy* | `claims_s0214_s21_readiness.rs` |
| #412 | Table I row v3 was not built | fixed (ccca7b6) | S-0214 r83 table I, S-0215 Terms: an infantry actor whose deploy weapon has the area-fire flag gets *area attack* where *g*₁ is deploy; `in_route_mode` turns it into *no-deploy*, as row v2 does *deploy* | `claims_s0214_s21_area_attack.rs` |
| #417 | A building whose undeploy name names no vehicle type counted as vehicle-like | fixed (b0fc3c6) | S-0214 r83 Terms, S-0415 Import mapping: `targeting::vehicle_like`, the building arm of *movable* and `structures::engineer::vehicle_like` use `econ::sell::vehicle_like` and `econ::deploy::undeploys_into` | `claims_s0214_s21_vehicle_like.rs` |
| #418 | The tap added the deploy building of any type with a deploy name | fixed (087b5fb) | S-0214 r83 claim 32: only a vehicle type's `DeploysInto` (`econ::deploy::deploys_into`) | `claims_s0214_s21_tap.rs` |
| #419 | The cheer rate limit *L* + *R* could overflow 32 bits | fixed (f649f82) | S-0213 Formulas: *t* > *L* + *R* in 64-bit integers | `claims_s0213_s21_cheer.rs` |
| #414 | The targeting docs listed seams that are now live states | fixed (4170f96, docs only) | The module doc names each live state's module and the seams that remain; the docs of `selectable`, `armed` and `disabled` and the comment over the state predicates say what each reads | none |

Lanes compare base 5d8c648 with the batch at 2993a49 (the sim is unchanged after it).

**Oracle arena** (`--record all`, two threads). Every output file is byte-identical to base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps.** M6c, M7a and M7b over seeds 1–16 print the same lines as base (every seed's length and counters, every minimum and median; smallest minimum 0, 2 and drain 3).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). Every game ends on the same tick with the same winner as base.

**Suites.** The AI suite with its ignored tests passes 362 before and after. The sim gate with its ignored tests passes 17 on base and, against the regenerated goldens, here; `cargo xtask gate` matches them on wasm32 too.

Seams left:
- Q-3152 (#284): a named crate overlay that holds no place and is no map crate (a resource bounce over modded data, S-0319 claim 10); its pickup removes it in the interim (`crates.rs`).
- #284: an overlay written straight into a cell without `overlays::place_overlay` or `crush::remove_overlay` (resource growth, wall placement, bridge pieces) does not forget the cell's crate owner.
- #413: rows V5 and VC6 cannot be reached until paralysis, a vehicle's death sequence and tubes are modelled.
- #417: the raw undeploy-name tests in `structures::spy` (S-0007 claim 25) and `objects` (S-0311 claim 11) cite other specs and are left.
- #412: S-0215 r139's named area-fire deployer changes its aim (claim 30), not the cursor; row v3 applies to it too.

No fix makes or moves a rarer back-reference of `World::census` (#402): the crate state holds cells and place numbers only.

`quirks::tests::every_spec_quirk_has_a_switch` fails on base for `quirks.bunker_hover_no_arrival` (S-0606), `quirks.path_congestion_ground_record` (S-0212), `quirks.infantry_animation_stale_marks` (S-0611), `quirks.crate_place_by_cell` (S-0520) and `quirks.offview_release_origin_cell` (S-0214). Here it no longer names `crate_place_by_cell` (217ff69, both positions tested in `claims_s0520_r12.rs`); the other four are not this batch's.

## Batch S23: the launch pass cadence, crates on slopes, warp-pending infantry landing, attack-move application and erased buildings

Base main 7f27529, not rebased. No format change: save 148, AI game 125, wire 12. One new switch, `quirks.warp_pending_infantry_landing` (fddea9d, default on); the committed ruleset schema was refreshed for it and for `crate_phantom_place`'s new doc in a commit of its own (e4fb3a4), because `the_committed_json_schema_is_current` failed from fddea9d on. The sim-gate goldens were regenerated once, after the last sim fix (4d3b6dc). Each test is committed before its fix and fails on base (logs in `local-lane/sim-issues/s23/`), except these, which hold on base and guard the fix: #349's slope-4 and phantom-expiry cases, #363's removal-before-appearance case, #357's first-slot-not-current-slot case and #358's value n case (already implied by row G10). Two test corrections are commits of their own: c9cb57f (#357, `claims_orders::s0213_attack_move` used the plain testkit's cart, whose weapon those rules do not define, so the cart had no first-slot weapon; it now runs on the combat rules with the crusher unarmed) and f05a07c (#131, the cost probe reads the host clock and prints, which the workspace lints refuse; a file-level allow with its reason). Request Q-3164 (21057e3).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #343 | The next launch pass kept the old phase, so a late spawner made passes back to back; a building being erased still made passes (Q-3134) | fixed (0461fbd) | S-0506 r45 claim 2: the next pass falls due 10 ticks after the tick of the pass just made; S-0514 claim 17: a building spawner being erased makes no pass | `claims_s0506_pass_cadence.rs` |
| #349 | A crate's overlay ignored S-0009 claim 12's slope condition (Q-3140) | fixed (de1a9c8) | S-0520 r16 claims 3 and 5: `overlays::slope_allows`, shared with the overlay stream. `quirks.crate_phantom_place` on: a refused overlay still takes the place with its lifetime draw and shows no crate; off: the attempt fails with no place and no draw. The quirk's doc names the slope case | `claims_s0520_r16.rs` (five tests, both switch positions) |
| #363 | A teleporting infantry unit with warp-pending immobilization chose no landing point, and its reservation was released at the join (Q-3142) | fixed (fddea9d switch, 8e66033) | S-0208 r66 claim 19, *Landing choice while a warp is pending*, *Appearance of a teleporting infantry warp member*: a pending unit chooses and reserves a landing point it does not jump to; in the leaving stage a destination is dropped before any choice; switch off, the destination is S-0207 claim 20 b's. A remembered point stays taken through the departure; a unit with none frees its entry at the departure and takes it again at the passing check; the appearance frees what a later choice would free and takes its landing point. A member removed off the map before it appears frees the same (provisional, Open questions). Warp functions touched: `join`, `lift_off`, `check_or_redirect`, `place`, and `objects::remove_with`; `teleport::warp_release` is gone | `claims_s0208_r66_warp_landing.rs` (eight tests, both switch positions) |
| #357 | The attack-move order's application side read the type's first-slot field, not the object's first slot | fixed (bbe37d7) | S-0214 claim 5: `orders::attack_move_eligible` is `targeting::eligible`, the input side's rule (a weapon in the object's first slot, no prevent-attack-move flag) | `claims_s0214_attack_move_slot.rs`; `claims_orders.rs` (c9cb57f) |
| #358 | Table I's passive-neutral branch of *engineer-friendly* took a building being erased; value n did not say *not being erased* | fixed (32bdad5); whether table I's other conditions reach both branches is Q-3164 (21057e3) | S-0214 table I: a passive neutral house's building being erased is not engineer-friendly. Table V value n: `accepting` asks not being erased, already implied by value b (row G10) and written out | `claims_s0214_erased_target.rs` |
| #131 | `Game::save` wrote the world's bytes one serializer call per byte into a vector grown from empty | fixed (fb2b894 probe, 1e112b1) | The world's bytes go through the serializer's byte form, which postcard encodes with the same length and bytes, into a vector sized for them. Same bytes and AI game format 125. Median game save on the synthetic skirmish at tick 3000 (298107 bytes): 796 and 989 µs before, 475 and 460 µs after; the world's own save is 450 to 474 µs throughout | `game::tests::save_bytes_unchanged_by_the_byte_form` (the old encoding, byte for byte, and a load); `tests/save_cost.rs` (ignored probe) |

Lanes compare base 7f27529 with the batch at 1e112b1 (the sim is unchanged after it). Changes are traced by rerunning at the fix commits.

**Oracle arena** (`--record all`, two threads). The failing counts are base's: M-0001 11 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50. In M-0001 only the end ticks of TELE against V3 side B at seeds 0, 2 and 4 change, with the same winners; M-0019 changes only its sample counts. `--record M-0001` at 0461fbd gives the batch's file exactly, so #343 makes the whole change.

**Sweeps.** M7b over seeds 1–16 prints base's lines. M6c changes distribution lines and M7a changes counters for seeds 7, 8, 10, 12 and 15; every minimum is base's and every sweep passes. All of them come from #343 (the sweeps at 0461fbd), except two M7a counters of seed 12 (1210 to 1196, and gas hits 312 to 310), which come from #357 (seed 12 at 8e66033 and at bbe37d7).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). Five winners change: seed 7, map 0, sides 0 and 1 (house 0 to house 1) and map 2, sides 1 and 2 (house 0 to house 1); seed 9, map 8, sides 0 and 1 (house 0 to house 1); seed 10, map 0, sides 1 and 2 (house 1 to house 0) and map 2, sides 0 and 2 (no winner at 120000 ticks to house 0 at 70869). Rerun at 0461fbd, each of these maps ends every game on the batch's tick with the batch's winner, so #343 makes all five. Seed 8 keeps every winner; five end ticks change.

**Suites.** The AI suite with its ignored tests passes 362 on base and 364 here (the identity test and the cost probe). The sim gate with its ignored tests passes 17 on base and, against the regenerated goldens, here; `cargo xtask gate` matches them on wasm32 too.

Seams left:
- Q-3164 (#358): whether table I's other engineer-friendly conditions apply to both branches.
- #363: a vehicle in the leaving stage is still treated as S-0207 claim 20 b; the release of a member removed before it appears is provisional (S-0208 Open questions).
- #358: value n's *not being erased* is redundant beside row G10; it is written out so the two do not drift.

No fix makes or moves a rarer back-reference of `World::census` (#402): the changes hold cells, layers, ticks and flags only.

`quirks::tests::every_spec_quirk_has_a_switch` still names `quirks.path_congestion_ground_record` (S-0212), `quirks.offview_release_origin_cell` (S-0214) and `quirks.infantry_animation_stale_marks` (S-0611), none of them this batch's; `warp_pending_infantry_landing` has its switch (fddea9d), and both positions are tested in `claims_s0208_r66_warp_landing.rs`.

## Batch S13: deaths, depots, power wrap, turning and the bridge deck

Base main 1a1fc50, not rebased. Three doc-comment issues come first (#248 in 1036912, #255 in cd86ad6, #259 in 84bd6da; comments only). Then nine issues and two seams that render batch R9 found in the projectile rules. Failing tests are committed first (fe2f367 to e1d8a6b, one commit per issue or seam), and each fails on the commit before its fix; no fix commit changes a test. Requests Q-3112 and Q-3113 (e664ef5) and Q-3114 (7717829) are filed. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S13: deaths, depots, power wrap, turning and the bridge deck)* (4184af5, d27bd05).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #249 | With `quirks.delay_kill_credited_again` off, a building in a delayed-kill countdown raised its destroyed-object events again at later lethal hits and at the expiry hit, though its credit was given once | fixed (15b628c) | S-0302 claim 16 and Edge cases (S-0312 claim 3: S-0302 owns the switch): `damage::killing_hit` makes a later delayed-kill hit's raises under the credit's test, and `death::kill_in` takes a `raises` flag, off for the final or expiry hit of a building with a countdown when the switch is off | `claims_s0312.rs` (`quirk_delay_kill_credited_again_governs_the_destruction_raises`, `quirk_delay_kill_credited_again_governs_the_expiry_raises`) |
| #250 | The crew of a vehicle killed above the terrain in a high-bridge cell went on the deck layer, even under the deck | fixed (5b4de1f) | S-0312 claim 13 with S-0209 Terms (cell layer): `death::crew_position` gives a crew put at the vehicle's exact position the vehicle's own layer | `claims_s0312_crew_layer.rs::crew_of_a_hover_vehicle_under_a_high_bridge_is_on_the_ground_layer` |
| #251 | A crew or survivor that got no position, or was refused, was removed alone: the slot objects and slaves of its creation set stayed in the object table | fixed (bb11b74) | S-0312 claims 13 and 18, S-0408 claim 7, S-0611 claim 14: `death::crew`, `survivors::place_survivor` and the sale survivors call `objects::discard_unplaced` | `claims_s0312_unplaced_creation.rs::a_refused_crew_takes_its_creation_set_with_it` |
| #252 | `quirks.crash_death_weapon_twice` was never read | fixed (8d9fc38); scope Q-3112 | S-0312 claim 16 and Edge cases: with the switch off, `death::kill_in` skips the death weapon at death for an exploding aircraft (S-0501 Terms, so not a rocket) that is crashing or above the terrain; the impact fires it as before. Q-3112's interim: an aircraft killed on the ground and a crashable vehicle keep the switch-on behaviour | `claims_s0312_crash_death_weapon.rs` (`switch_on_fires_at_death_and_at_the_impact` passed before and guards the default; `switch_off_fires_only_at_the_impact`) |
| #256 | Automatic repair and sale were considered for a building being erased | fixed (c661e82); paid steps Q-3113 | S-0314 claim 12: `sell::building_turn` skips `automatic` for a building being erased or phasing in. Q-3113's interim: a repair already on keeps claim 8's paid steps | `claims_s0314_erasure.rs` (`an_erased_building_does_not_start_automatic_repair`, `an_erased_building_takes_no_sale_draw`) |
| #257 | A docked vehicle ordered to enter another depot that holds a contact got that depot as its destination | fixed (a82dbfa) | S-0314 claims 20 and 23: in `depot::order_applied`'s tethered branch an enter order on another depot that holds a contact takes claim 23 (queued entry and remembered depot, no destination; at full health the order ends); the contact and tether stay. One helper serves both callers | `claims_s0314_docked_enter.rs` (`a_damaged_docked_vehicle_queues_for_another_held_depot`, `a_full_health_docked_vehicle_gets_nothing_from_another_held_depot`) |
| #258 | The blackout end tick, a building's output and drain sums and the adjusted cost used plain `i32` operators, which panic on overflow in a checked build | fixed (ef3127b) | S-0205 claim 1: wrapping operators in `power::start_blackout`, `building_output`, `building_drain`, `sell::adjusted_cost`; `power::house_update`'s saturating sums (a clamp no spec states) wrap too | `claims_s0205_power_wrap.rs` (six tests) |
| #260 | A seized vehicle with no movement destination never rested its turret on its body | fixed (31dfa9c) | S-0206 claim 9: in `turret::tick` the seized exception applies to the destination row only | `claims_s0206_seized_rest.rs::a_seized_vehicle_without_destination_rests_on_its_body` |
| #261 | A turn in place given again while the body was already turning to that facing held the vehicle a whole duration after the body arrived | fixed (da50aa6) | S-0206 claims 3 and 12, S-0207 State and ordering: `drive::turn_in_place` records the body's own turn start (`body.t0`), which the same target keeps; a turn that completes at once keeps the order's tick (claim 12's next-tick start) | `claims_s0206_regiven_turn.rs::a_regiven_turn_in_place_begins_the_track_when_the_body_arrives` |
| #262 | With `quirks.drive_track_end_cap` off, a tick begun without a track that went on into its path's last track set ρ to 0 mid-track | fixed (ce0e611) | S-0207 claims 11 and 12: the cap-off loop in `drive::tick` updates `ended` with each track it spends on | `claims_s0207_cap_off_rho.rs::cap_off_keeps_the_fraction_on_the_last_track` |
| R9 seam: deck crossing | Real. T1 row 1 gave the deck height going up and 20 below it going down, and measured each high-bridge cell's own deck height | fixed (c85086b); steering row Q-3114 | S-0306 r89 T1 row 1 (unchanged in r90): at the deck height going down, 20 below it going up; the deck height is the terrain of the cell of *P*′ + 416 for a crossing in the old or the new cell. The steering row keeps each cell's own height until Q-3114 is answered | `claims_s0306.rs`: `t1_row1_deck_crossing_up` and `t1_row1_deck_crossing_down` corrected in their own commit (e6aa556, the pre-r89 heights), `t1_row1_deck_height_is_that_of_the_new_cell`, `t1_row1_deck_crossing_into_a_lower_cell` |
| R9 seam: a high-bridge cell's target point | Real, but not in `range::target_point`: that is the reference point (S-0204 claim 4; S-0306 Terms keep a cell's reference point at the terrain). The projectile rules used it where the spec asks for the target point | fixed (b496519) | S-0306 r89 Terms (points of a cell target): `projectile::cell_target_point` raises a high-bridge cell by 416 for the shot speed, the aim point, *K* (with the stored-point-off rule, and an airburst child's, claim 21), the led aim point, F9's *P*_R, the flak row's height and claim 20 (c)'s first row; `aim_target_point` starts from it, so the steering rows that take the target point get it too. Claim 20 (b), *D*_R, *D*_S and the voxel pitch keep the reference point; the lob test is open in the spec and keeps it | `claims_s0306_bridge_target.rs` (three tests) |

Formats: save 138 → 145 and AI game 114 → 121 (51b8750); wire stays 12. Goldens (8d4bc30): only `duel-m7a.txt` moves, first at the hash after tick 7910; a build of the final tree with only #260 undone matches all nine old goldens.

Lanes, base 1a1fc50 against the batch at 8d4bc30 (its sim is that of b496519). Attribution builds: detached worktrees at ef3127b, 31dfa9c, da50aa6, c85086b and b496519. Oracle arena (`--record all`, two threads): only M-0001 moves, failing 9 → 10 of 96 (side outcomes reproduced 157 → 156 of 171). TELE against APOC now fails its side outcome: side A seed 3 goes from col@4560 to none@6080, and the other five seeds of side A end at other frames. It is moved by #260: ef3127b matches base, 31dfa9c matches the batch. In the trace of side A seed 3 the first difference shown is at tick 545, where B#439 (an APOC, side B) fires at A#431 (a TELE). B#439 was seized by A#430 at tick 1 and dropped at tick 302, staying seized with no destination: #260's case. The trace shows no turret facings. LTNK against E1 still fails its side outcome, with our row wins 2/12 → 4/12: side A seed 0 and side B seed 0 change winner (column to row), and four other seeds end at other frames. It is moved by #261: 31dfa9c matches base, da50aa6 matches the batch. In the trace of side A seed 0 the first difference is at tick 207, where a hit by B#453 on A#424 lands 18 leptons west (x 35950 against 35968). A#424, an LTNK, was given a new destination at tick 206 and moved on sooner. LTNK against GGI (side A seed 3), TELE against SREF and TELE against TTNK end at other frames with the same winners. M-0019 changes only sample counts; M-0002, M-0003, M-0020 and M-0021 are identical. Sweeps (seeds 1–16): M6c passes with the same seed lines (seeds 3, 4 and 14 still end early) and some medians and maxima moved. M7a passes and every seed's counters move (its magnetic lifters, #260); the smallest minimum stays 0 (seed 13), and seeds 1, 7, 11 and 16 have a counter down to 1 or 2. M7b prints the same lines as base. AI quick lane (seeds 7–10, maps 0, 2 and 8): 21 of 36 games end at another tick, and 2 change winner. In seed 7 two games move with the same winners: map 0 sides 1 and 2 at 35759 → 34074 (house 1), and map 2 sides 0 and 1 at 34148 → 32270 (house 0). (The batch's report first had these two games' base and batch values swapped and took them for a base run that did not reproduce. Determinism check D1 found the slip: the base file and every rerun of the base binary agree, nine of nine and eight of eight runs, and the two lower values appear only in the batch's file.) Reruns of seeds 8 to 10 reproduce their base files exactly. Seed 8 map 0, sides 0 and 2: house 0 at 31574 → house 1 at 62607. Its first difference is at tick 9499, in projectile 2648 (weapon 104, fired by object 1329 of house 0 at tick 9479 at the high-bridge cell (66, 71)): its velocity is aimed at the deck height. That is the seam fix b496519; c85086b matches base all game. Seed 10 map 8, sides 0 and 2: house 1 at 30738 → house 0 at 33596. Its first difference is at tick 11100, in object 557 (a house 0 vehicle, seized and set down, no destination): its turret starts back to the body facing 16384. That is #260: ef3127b matches base, da50aa6 matches the batch, and the only difference at that tick is the seized vehicle's turret. The AI suite with its ignored tests passes 356. On the final tree: the sim gate with its ignored tests passes 17 against the regenerated goldens. `xtask check` and `xtask gate` pass (native and wasm32 on all nine goldens). Clippy with `-D warnings` and `cargo fmt --check` are clean, and the wasm32 build of `glasnost-app` succeeds. The workspace tests pass 8661 and fail 1: main's `quirks::tests::every_spec_quirk_has_a_switch` (`quirks.crate_beneath_overlay_refusal`).

Seams left: S-0206 claim 9's rest rows also send a vehicle whose turret weapon is turret-locked to its body facing; turret-locked weapons are not imported, so `turret::tick` has no such branch. The lob test of S-0306 claim 6 (a) for a high-bridge cell is open in the spec and keeps the reference point.


## Batch P3 (sim perf)

Base main fd702c6, not rebased. Performance only, with no behaviour change: the sim goldens, the oracle arena's output files, the M6c, M7a and M7b sweeps (seeds 1–16) and the tick-bench state hashes (synthetic, and the map run's per-tick hash file) are byte-identical to base. Save format 140, AI game format 117 and wire version 12 are unchanged. No requests.

Issue #402: an ordinary removal walked the whole object table nine times, and a killing hit walked it more. Two walks of the held-target drop are merged into one pass. Six walks, in five hooks, are skipped when no object holds the reference they look for. To know that, the drop's first walk (the target holders, which keeps its timer draws in key order) also takes a census: whether any object is of the aircraft kind, or holds a spawn controller, a bomb, a charger list or a dropper. The census is not an index:
- It lives only inside one removal, or inside one drop at a killing hit or a departure from the map.
- The end of the removal forgets it, and so does every place that can make such a reference: an object's creation (an object's kind never changes), a spawn controller's, a bomb's planting, a charger's registration, a drop, and the put-back of a removed object for a credit or a leftover's bookkeeping.
- Aircraft state is not noted, because it is made on demand, on objects of other kinds too. So the heading-pad walk, which reads any object's aircraft state, is not skipped. It was skipped in d119762, and b62a751 undid that.
- It is `#[serde(skip)]`, empty between actions, so it is neither saved nor hashed.
- A debug assertion checks every skip against the whole table. The debug workspace suite (8707 tests) ran with it and never tripped it.

With none of those references, a removal now walks the table twice (the holders, and the merged pass), not nine times. Where a reference exists its walk runs as before.

The bench `tests/removal_cost.rs` (ignored, release) removes or kills 64 infantry, one at a time, out of a square of synthetic infantry. It gives the median µs per object over 5 runs. Each pair below is base 4d6e2e3 → a743b6e, from interleaved runs (base, after, base, after). The bench removes only infantry with no aircraft state, so b62a751 does not reach what it measures, and 45ca198 adds only a test. The figures cover the batch's walks together; no walk is timed alone.

| Objects | One removal | One killing hit with its removal |
|---|---|---|
| 484 | 13.6/11.7 → 7.6/8.5 | 22.6/20.0 → 15.9/16.7 |
| 2025 | 61.2/52.6 → 29.8/33.4 | 100.5/90.3 → 69.1/72.1 |
| 4096 | 167.3/148.3 → 63.7/69.1 | 280.3/257.0 → 156.8/162.3 |

| Walk | Finding | Status | Fix | Before → after | Tests |
|---|---|---|---|---|---|
| Put-aside slots and attack-move memories (`held::holders_drop`) | Two walks, one after the other, at every removal and killing hit | fixed (20edf08) | One pass clears the put-aside target and destination and converts a remembered target to its cell, the same per object; `aside_lost` stays for cloaking | Not isolated (in the table above) | `removal_walks.rs::one_pass_clears_put_aside_slots_and_converts_memories` (158fed4, committed first; fails with a merge that keeps the destination) |
| Aircraft destination (`air::destination_lost`) | Walked at every removal and killing hit with no aircraft in play | fixed (d119762, with the census; narrowed to the aircraft kind in b62a751) | Skipped when the census has no object of the aircraft kind | Not isolated | `removal_walks.rs::every_rarer_back_reference_is_dropped_after_a_removal_without_them` (d7536df, committed first; fails with the aircraft census wrong) |
| Heading pad (`air::heading_pad_gone`) | Walked at a building's removal | left (skipped in d119762, unskipped in b62a751) | Aircraft state can appear inside a removal without a forget | — | `claims_s0501_r176.rs::claim_21_a_destroyed_or_removed_pad_leaves_no_record` |
| Bomb planter (`bomb::removed`) | Walked at every removal with no bomb in play | fixed (1fff4c1) | Skipped when the census has no bomb; a planting forgets the census | Not isolated | the same test (fails with the bomb census wrong) |
| Charger lists (`linked::leaving`) | Walked at every removal with no charger list | fixed (f9ef214) | Skipped when the census has no charger list; a registration forgets it | Not isolated | the same test, and `…::a_save_and_load_between_removals_changes_nothing` (both fail with the charger census wrong) |
| Dropped units (`lift::removing`) | Walked at every removal with no dropped unit | fixed (fe227c7) | Skipped when the census has no dropper; a drop forgets it, so the drop a lifter makes as it leaves play is undone in the same removal | Not isolated | the same test (fails with the dropper census wrong); `…::a_drop_made_inside_the_removal_is_still_undone` (fails without the drop's forget) |
| Spawn clear requests and controller targets (`spawn::removing`) | Two walks at every removal with no spawner | fixed (ec0199b) | Both skipped when the census has no spawn controller; a new controller forgets it | Not isolated | the same test (fails with the spawner census wrong) |

The census lifetime has a test of its own: `removal_walks.rs::no_census_outlives_its_action` (a743b6e) fails when the end of a removal does not forget it. `…::every_place_that_makes_a_noted_reference_forgets_the_census` (45ca198) fails when the forget at a creation, a planting or a charger's registration is dropped. The forget in a drop is tested through a lifter leaving play, as above. Three forgets have no test that fails without them:
- `spawn::created_in`'s: the creation it follows has already forgotten the census.
- `leftover::with_restored`'s and the lift credit snapshot's: private paths.

They are covered only by the debug assertion. The save and load test compares the state hashes of a game saved and loaded between two removals with those of the game never saved. The guard tests of d7536df pass on base. Each break above was made temporarily with debug assertions off, so that the test's own assertion is the one that fails, and then undone. The bench's lints are allowed in b07b45c, and 4cb3171 is rustfmt.

Lanes, base fd702c6 (4d6e2e3, which adds only the bench) against the batch at a743b6e. Synthetic tick bench (2000 ticks, interleaved pairs): mean 61/48 → 46/45 µs, median 46/41 → 37/37 µs, at 21 objects; state hash 3a025dbb… on all four runs. Map bench (map 9, seed 7, 8 players, 11399 ticks, interleaved pairs at load about 14): mean 4843/4111 → 4229/4139 µs, median 3320/2840 → 2878/2969. That is within noise, since few objects leave play in a tick. Final hash 382bff1a… on all four runs, and the per-tick hash files are identical. Oracle arena (`--record all`, two threads): every output file is identical; wall time M-0001 963 → 1085 s, M-0003 241 → 263 s, under a shared machine's load. The three sweeps print the same lines as base.

After b62a751 and 45ca198 every lane ran again on the final tree (45ca198), one run each against the base runs above. The oracle arena's output files are identical to base (wall time M-0001 920 s, M-0002 140 s, M-0003 237 s, M-0020 63 s). The three sweeps print the same lines as base. The synthetic tick bench keeps hash 3a025dbb… (mean 44 µs). The map bench keeps final hash 382bff1a… and its per-tick hash file is identical to base (mean 4072 µs).

The checks all ran on the final tree (45ca198).
- The sim gate with its ignored tests passes 17 against the unchanged goldens.
- The AI suite with its ignored tests passes 362.
- The workspace tests (debug) pass 8707 and fail none.
- `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --check` are clean.
- The wasm32 build of `glasnost-app` succeeds.
- `xtask check` passes. `xtask gate` passes, and on wasm32 it matches all nine goldens.

Seams left:
- A killing hit still walks the table for the target holders, the merged pass, and `spawn::target_lost`'s settle and controller walks. The census is forgotten at the end of the drop, before `spawn::killed` runs; keeping it across the whole killing hit was left out.
- A building's removal or kill still walks it for heading pads, since aircraft state can appear without a forget.
- Each skipped walk still runs in full when one object anywhere holds its reference.
- `grinder::removing` (buildings only) and the target-holder walk itself are unchanged.
- d119762 holds the census with the aircraft walk, and ec0199b holds two spawn walks, so neither is one walk per commit.

## Batch S16: domination, infantry-making animations, owner changes, particle systems and re-paths

The base is main fd702c6, not rebased. The batch covers thirteen issues: #290, #291, #292 and #298 to #307. The formats are save 148 and AI game 124 (8e4188f), renumbered to save 143 and AI game 120 at the merge. The animation's new `making` record (#291) is the only new saved field, and it has a serde default. The wire version is unchanged (12).

All the failing tests are committed first, c246473 to 8a87929, one commit per issue. On the last of those commits, 8a87929, each new test fails and each control passes. The fixes follow, one commit per issue, and no fix commit changes a test.

Requests Q-3121 (bffdeb6) and Q-3122 (a5af347) are filed; Q-3123 is unused. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S16: domination, infantry-making animations, particle systems and re-paths)* (e83a0a2). The C7 ledger's *Resume* sentence follows #300 (5a0dc0d).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #290 | A drain-mode domination order in ready was accepted while a domination was in force | fixed (f70ad38) | S-0612 claims 3 and 7, S-0512 claims 8 and 19: `superweapons::launch::drain`'s ready branch refuses a dominator-kind order while `domination::in_force` | `claims_s0612_drain.rs` (`claim_3_drain_type_second_launch_by_another_house_is_refused`, `claim_3_drain_type_relaunch_by_the_same_house_is_refused`; control `claim_3_ordinary_type_second_launch_is_refused`) |
| #291 | An infantry-making animation made infantry only when made by the control effects | partly (268c55e); Q-3121 (bffdeb6) | S-0611 claims 8, 9 and 11: an unattached, non-bouncing animation with no owner whose type makes infantry carries a `mutation::Making` record. While its frame makes infantry it marks its position, and it frees the position when it goes. At its natural end it makes its infantry through `mutation::try_make`; a refused attempt steps back and retries. Attached or bouncing animations are Q-3121, and they keep the old path | `claims_s0611_creators.rs` (trailer, plain, reverse, marking, refusal and save/load tests; control `claim_9_control_effects_anim_makes_its_unit`) |
| #292 | With the switch off, a controller's victims were released after its owner was written, so each release's fate decision read the new house | fixed (2f23adf); schema text 91817e3 | S-0601 claim 11 and Edge cases. `mind::owner_changing` releases all victims before the owner is written. It is called from `mind::set_owner_by`, `ownership::change_by` (after the same-owner test), the contact hand-over and the slave hand-over. `mind::owner_changed` goes | `claims_s0601_release_first.rs::switch_off_release_reads_the_house_the_controller_had_before_its_change` (control `switch_on_keeps_the_victims`) |
| #298 | The per-tick approach and the rally-cell reads read the reach regions out of date | fixed (12e03e1) | `regions::ensure` before `approach::reaches` in `approach::per_tick`, and before each of the three `orders::rally_cell` reads in `econ::release` | `claims_s0212_regions_fresh.rs` (2), `claims_s0417_rally_fresh.rs::rally_spread_after_a_category_change` |
| #299 | A deploy order to a building in its construction state replaced the queued construction | fixed (6e49e86) | S-0213 claim 10, S-0406 claim 17: the deploy arm of `orders::apply_other` ignores an actor still being built up, or whose seen behaviour is construction or selling | `claims_s0213_deploy_construction.rs::deploy_order_to_a_building_under_construction` |
| #300 | Any order naming a unit ended its giving way, even one that failed its checks | fixed (949e046); docs 5a0dc0d | C7-P5 design: `path::order_given` moves from `apply_pending`'s up-front loop into the stop, scatter and deploy arms, after their checks, and into `apply_behaviour`, after claim 4 and table T3 (a) | `pathing_c7p5_yield_orders.rs::a_failed_order_leaves_a_unit_giving_way_its_move` |
| #301 | A trigger-made system aimed at the world origin | fixed (f5d6920) | S-0616 claim 1: `emitters::trigger_create` sets the target to the creation point | `claims_s0616_trigger_target.rs::trigger_made_railgun_system_has_its_own_position_as_target` |
| #302 | An effect weapon's smoke emitter jumped onto its firer | fixed (2d5e64e) | S-0616 claims 1 and 5: `emitters::effect_weapon_emitter` sets the owner offset to the creation point less the firer's position, with wrapping | `claims_s0616_weapon_smoke.rs::smoke_kind_weapon_emitter_keeps_its_creation_displacement` |
| #303 | A fire system with spawn interval 0 never emitted | fixed (9cbc2ef) | S-0616 claim 20, S-0205 claim 3: `particles::fire_turn` takes the interval test for 0 too, so it passes at tick 0 only | `claims_s0616_fire_interval_zero.rs::fire_interval_zero_emits_at_tick_zero` |
| #304 | A heading at the integer limits, and a split with spawn radius −1, overflowed | fixed (be68623) | S-0205 claim 1: `particles::heading` uses `wrapping_sub`; `split_and_remove` uses `wrapping_mul` | `claims_s0616_wrapping.rs` (`heading_from_a_position_at_the_range_edge_wraps`, `smoke_emission_with_spawn_radius_minus_one`) |
| #305 | Congestion marks for a unit on a deck read the ground under it | partly (18ae9a5); Q-3122 (a5af347) | S-0212 claims 7 and 19: `path::congestion` reads the cell ahead and each 5 × 5 cell on the layer that a step from the unit's start pair reaches (`candidate_layer`). Whether the other layer counts as well is Q-3122; it does not, as an interim | `claims_s0212_congestion_deck.rs::the_same_on_a_bridge_deck` (control `control_on_the_ground`) |
| #306 | A mode-1 search whose counts were all even ran as mode 0 | fixed (e804de4) | S-0212 claim 19: `compute_once` downgrades to mode 0 only when `congestion` answers none, meaning part (i) counted no cell. The doc of `trace::What::Search::mode` follows | `claims_s0212_mode_one_even.rs::mode_one_with_a_counted_path_cell_stays_mode_one` |
| #307 | A straightening shape's start level came from a per-cell table of the found path | fixed (78264ea) | S-0212 claims 7, 17 and 18: `flatten`'s test and the new internal `shorten_at` (the public `shorten` keeps its form) walk the current path from the start pair with `candidate_layer`. After a tunnel step the walk is on the ground | `claims_s0212_straighten_deck.rs::a_unit_under_the_deck_changes_the_straightened_deck_path` |

Lanes compare base fd702c6 (the binary built there) with the batch at 8e4188f.
- The oracle arena (`--record all`) prints exactly what base prints, apart from timings and paths.
- The M6c, M7a and M7b sweeps over seeds 1–16 print exactly what base prints.

The AI quick lane plays seeds 7–10 on maps 0, 2 and 8, 36 games in all. 22 games change. Per seed, the changed games are:
- seed 7: games 1–6;
- seed 8: games 1–6 and 8;
- seed 9: games 1–3, 5 and 6;
- seed 10: games 1–4.

Map 8 changes least. Ten of the changed games make no infantry-making animation on base or in the batch, so #291 alone does not explain the changes. The pathing and order fixes (#298, #300, #305 to #307) and #302 are the likely other sources; this was not traced.

One game changes winner: seed 8, map 2, sides 1 and 2. On base house 0 wins after 68650 ticks; in the batch house 1 wins after 79998. Its mutation animations go from 9 to 16. The trace stopped at the first step: a hash-every-500 run of each build was made, and the guard refused the command that would read the hash lines. So the cause of the flip is not traced. The two runs are kept in the lane directory.

The checks all ran on the final tree.
- The sim gate with its ignored tests passes 17. The goldens were regenerated after the format bump. duel-m7b moves in 13 isolated sample hashes, each returning to the old value; this is not traced, and a transient particle-system state (#302) is a guess. No other golden moved.
- The AI suite with its ignored tests passes 362, 0 failed.
- The workspace tests pass. The first run failed the committed-schema test: #292's doc comment flows into the schema description. The schema was regenerated (91817e3).
- `cargo clippy --workspace --all-targets -- -D warnings` is clean. `cargo fmt --check` is clean after the rustfmt commit 281060c.
- The wasm32 build of `glasnost-app` succeeds.
- `xtask check` passes. `xtask gate` passes, and on wasm32 it matches all nine goldens.

Seams left (also in the design section):
- Q-3121: an attached or bouncing infantry-making animation.
- Q-3122: the other cell layer in the congestion marks.
- An ordinary infantry-making animation has no owner colour and makes no maker presentation events.
- The stop arm's construction and tunnel checks are still only its construction-or-selling return.

## Batch S15: S11's answered requests, the placement tick, crates and lifts

Base main 3d9f38e, not rebased. The lead's answers to S11's three requests come first: Q-3102 (S-0309 r142), Q-3100 and Q-3101 (S-0215 r139). Then #237's field removal, the simulation half of #81, and #278 to #285. Save format 147 and AI game format 123 (0cb8c8d; renumbered to 144 and 121 at the merge): #237 removes two saved fields and #81 adds one. The wire version is unchanged (12). Every golden moved (a396d37). Failing tests are committed first, one issue per commit: 15060e1, 6688a52, a75ecb6, 5604319, d779337, 006b9d7, bb82c64, 78f8c59, de16c20 and 729abd6. Each fails on the commit before its fix, and no fix commit changes a test. Two scaffold commits let a test commit build: 2a3f535 adds #232's rules field and schema entry, and de839f3 adds #81's field. Neither is read before its fix. #237 has no failing test: the two test set-ups that wrote a removed field go in their own commit (962f925). #278 has no test, since no reachable path differs. #285 is a comment only. Requests Q-3118 (6bd0c1a) and Q-3119 (9317b32) are filed. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S15: S11's answered requests, the placement tick, crates and lifts)* (431e8e0).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #229 | Q-3102: S11's interim started *attack* at once in the attack-move search | fixed (19aa137) | S-0309 r142 claim 23 (a), S-0215 claims 7 and 11: both rows of `combat::acquire::attack_move_search` queue *attack* with the readiness-dependent start (`queue_and_start` with `process::ready`). A moving vehicle goes on in *move* with *attack* queued; the search leaves the destination and route alone | `claims_s0309.rs` (S11's two ignored tests now run: `issue_229_attack_move_destination_search_not_ready_queues_attack`, `issue_229_attack_move_target_search_not_ready_queues_attack`; new `issue_229_attack_move_search_keeps_destination_and_route`) |
| #232 | Q-3100: the type singled out by name in *unload* was not built | fixed (3163109); scaffold 2a3f535 | S-0215 r139 claim 30, Terms and Import mapping: our format's `named_area_fire_deployer` flag, which the importer sets on the infantry type named `DESO` (`convert::sim`). With `quirks.named_deploy_area_fire` on, such a unit whose deploy weapon has the area-fire flag takes no new target, and with a negative undeploy delay waits its deploy sequence's frame count + 1. With the switch off the flag is ignored | `claims_s0215_r139_named.rs` (five tests); `convert/src/sim_tests.rs::named_area_fire_deployer_flag` |
| #234 | Q-3101: the hover-attack cell of *attack* and *guard* was not built | fixed (f94093b) | S-0215 r139 claims 23 and 24, S-0212 claim 25: `process::hover_attack_cell`. A hover-attack type at height 0 above the surface takes the nearby cell centred on its own cell (own class, region and deck mode), or the placeholder cell (0, 0) when none qualifies, through the order assignment (S-0207 claim 27's unchanged-destination rejection). *Attack* takes it with or without a target and *guard* with a target. A changed destination drops the stored route (design) | `claims_s0215_r139_hover.rs` (seven tests) |
| #237 | `turret_was_turning` and `last_shot_target` were never read | fixed (bbe0344; test set-ups 962f925) | Both fields and the one write in `combat/behave.rs` are removed from `combat::state` | none (the removal; every existing test still passes) |
| #81 | The simulation kept no placement tick for the drawing | simulation half done (48ab197; field de839f3) | S-0105 claim 6: `Object::placed_tick`, saved and hashed, set by `objects::place_with` (not buildings), a rocket's placement, a drop-pod stand-in's end, and kept by the warp appearance. The renderer reads it as a public field; nothing in `crates/render` or `crates/app` was touched | `claims_s0105_placement_tick.rs` (five tests), `claims_s0105_placement_tick_warp.rs` (passes before the fix, since every tick was 0) |
| #278 | `walk::arrival_tests_apply` left out "not falling"; `paradrop.rs` still called the parachute's playback a seam | fixed (719883b) | S-0208 claim 16: the test also requires the unit is not falling. The module comment now says `crate::anims` runs the playback (S-0522 claim 13). No result changes | none (no reachable path differs) |
| #279 | A rocket's aim at a driving vehicle took the terrain height at the track target | fixed (11e47c4) | S-0207 Terms (heading point) and claim 24, S-0506 claim 14: `rocket::heading_point_with` uses the track's stored height | `claims_s0506_heading_height.rs` |
| #280 | A seized jumpjet's neighbour step replaced its order-level destination | fixed (8476fd3) | S-0607 claims 5 and 9, S-0502 claim 21: `jump::give` returns at once for a seized unit; the step's draw and its cruising phase stay | `claims_s0607_neighbour_step.rs` |
| #281 | A collector killed by the crate tag raise, still in its death sequence, completed the pickup | fixed (c60f5de) | S-0520 claim 7: `crates::pickup` ends with the step blocked when the collector is gone or `damage::dead` | `claims_s0520_dying_collector.rs` |
| #282 | The unit crate gave base-unit types to the neutral and special houses | fixed (d1a1b4c) | S-0520 claim 13 (*In our engine*) and the glossary (*AI house*): `crates::random_vehicle` refuses a base-unit-list type to a house that is not human-controlled, whatever the bases setting | `claims_s0520_neutral_unit_crate.rs` (two tests) |
| #283 | Napalm with no flame warhead also skipped its position-0 animation | fixed (9f0e6d2) | S-0520 claim 17 and Edge cases: the midpoint and the animation come first; a missing warhead skips only the hit and the area damage | `claims_s0520_napalm_anim.rs` (two tests) |
| #284 | An expiring crate place cleared whatever overlay was on its cell | partly (43772bb); Q-3118 (6bd0c1a) | S-0520 Terms and claim 4: the expiry removes the overlay only when the cell holds a crate. Interim: a place whose crate another rule removed stays occupied until its lifetime ends and still makes its replacement placement | `claims_s0520_stale_place.rs` |
| #285 | Whether an open-topped transport's passenger counts as having left play for a seizing hit | not done: Q-3119 (9317b32); comment d1763b6 | `lift::seize_hit` keeps the firer on-map test; the comment names the interim reading | none |

The lanes compare base 3d9f38e with the batch (a396d37).
- **Arena:** the same table as base apart from timings. Failing rows: M-0001 9 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25, M-0021 10 of 50.
- **Seed sweeps:** M6c, M7a and M7b all pass. Their counters changed and were not traced.
  - M6c: seeds 1, 3 and 14 change length (9233 → 9276, 9889 → 9753, 10131 → 9729). The smallest minimum stays 0.
  - M7a: the smallest minimum goes 0 → 2 (seed 13's bunker counters, 0 → 3).
  - M7b: every seed's drain, takeover and psychic counters move, and all still run 8000 ticks.
- **AI quick lane** (seeds 7 to 10, maps 0, 2 and 8): all 36 games end differently, and 15 change winner (base → batch):
  - 7/0 (0,1): 0@43233 → 1@57367
  - 7/2 (0,2): 0@41695 → 1@47690
  - 7/8 (0,1): 0@34592 → 1@41848
  - 8/0 (0,2): 0@26886 → 1@54389
  - 8/0 (1,2): 1@53020 → 0@72090
  - 8/2 (1,2): 0@68650 → 1@76242
  - 8/8 (0,2): 0@32925 → 1@44612
  - 8/8 (1,2): 1@56992 → 0@47853
  - 9/0 (0,2): 1@60979 → 0@23886
  - 9/0 (1,2): 1@45766 → 0@61924
  - 9/8 (0,2): 0@34210 → 1@36517
  - 10/0 (0,2): 1@59796 → 0@49404
  - 10/2 (0,2): 1@52496 → 0@42375
  - 10/8 (0,2): 0@40494 → 1@34281
  - 10/8 (1,2): 0@69890 → 1@48783

  #237 and #81 change the saved world, so base and batch hash differently from tick 0. The lane was therefore also played on f94093b, which holds #229, #232 and #234 and has base's shape (its tick-0 hashes match base's). f94093b equals the batch in 33 of the 36 games and already holds all 15 winner changes. The other three games keep their winner:
  - 7/0 (1,2): base 1@78181, f94093b 0@58442, batch 1@43065;
  - 8/8 (1,2): f94093b 0@48112, batch 0@47853;
  - 9/0 (1,2): f94093b 0@61913, batch 0@61924.

  Every winner change was traced, base against f94093b, with the state hash every 10 ticks (`GLASNOST_AI_HASH`) and an object dump at the first differing tick (`GLASNOST_AI_OBJS`). Each time the first difference is one unit with an attack-move memory and a target. On base it starts *attack* at once. In the batch it is not ready, so it goes on in *move* with *attack* queued. That is #229 (S-0309 r142 claim 23 (a), S-0215 claims 7 and 11). First difference (game: tick, object, house, kind):
  - 7/0 (0,1): 9456, 1212, house 0, infantry
  - 7/2 (0,2): 8608, 923, house 0, infantry
  - 7/8 (0,1): 8007, 235, house 0, vehicle
  - 8/0 (0,2): 10175, 1489, house 0, infantry
  - 8/0 (1,2): 10750, 1288, house 0, vehicle
  - 8/2 (1,2): 10564, 1986, house 0, infantry
  - 8/8 (0,2): 8313, 540, house 0, vehicle
  - 8/8 (1,2): 8906, 521, house 0, vehicle
  - 9/0 (0,2): 9319, 1397, house 0, vehicle
  - 9/0 (1,2): 11409, 1330, house 0, vehicle
  - 9/8 (0,2): 8023, 559, house 0, vehicle
  - 10/0 (0,2): 9777, 1414, house 1, vehicle
  - 10/2 (0,2): 8146, 871, house 0, vehicle
  - 10/8 (0,2): 7969, 558, house 0, vehicle
  - 10/8 (1,2): 8576, 524, house 0, vehicle

  7/0 (1,2) was bisected over the later commits. 48ab197 and 719883b give f94093b's result (0@58442); 11e47c4 and c60f5de give the batch's (1@43065). So #279 moves that game. 8/8 (1,2) and 9/0 (1,2) were not bisected.

The checks all ran on the final tree.
- The sim gate with its ignored tests passes 17. After the format bump all nine goldens moved and were regenerated (a396d37). Four change length: duel 1931 → 2001 ticks, duel-diagonal 2112 → 1839, duel-naval 1437 → 2126, skirmish 2390 → 3259.
- The AI suite with its ignored tests passes 362, after b1577e4 reworked the launcher climb's set-up. Before that it gave 361 passed and 1 failed: `skirmish::climbs_to_the_launcher_and_fires` passes on base and failed from 19aa137 (#229) on, at every later commit tried.
  - What changed: base and 19aa137 part at tick 16243, where object 77 (house 0's vehicle, attack-moving to (32, 29)) shows #229's signature. On base it starts *attack*; at 19aa137 it goes on in *move* with *attack* queued.
  - On base: house 0's launcher (LNCH), started by tick 17 000, is built at tick 19 273, and rockets fly.
  - With #229: house 0's vehicle production line, with the launcher item on it, is gone at ticks 18 000, 19 000 and 20 000, and back at 21 000. Its building line holds an item of FACT's price (1500) at 18 000, so the war factory was lost between ticks 17 000 and 18 000. Later launcher items wait for money (both houses hold about 0 credits from tick 27 000, as on base). No launcher is built before house 1 wins at tick 55 711.
  - No unit stays in *move* with *attack* queued for more than 2000 ticks, so this is not a stall.
  - The test's premise (the climb comes before the war takes the factory) no longer held at the plain seed. At the coordinator's request the scenario was reworked, following the file's own precedent (the move to the island map), so that the premise holds by construction (b1577e4).
    - The climb's rules open every type that crosses the water or strikes across it (NAVY, HPAD, SILO, BOAT, HOVR, LIFT, LNCH, JET) to house 0's country only, through the required-houses list (S-0407 claim 5 (b)).
    - What house 1 keeps cannot reach house 0's island. Its ground units and wheeled transport keep to their own island, and the gun (5.5 cells) is short of the 9-column strait.
    - House 1 still climbs to LAB. Its waves go to its own shore (`Script::reachable`), where house 0's launcher can reach them from its shore (`Script::reaches`).
    - The map, the seed (5) and the plan are unchanged, and so are the assertions. The set-up was fixed before its one run, which passed.
- The workspace tests pass, `quirks::tests::every_spec_quirk_has_a_switch` among them.
- `cargo clippy --workspace --all-targets -- -D warnings` is clean. `cargo fmt --check` needed a rustfmt commit (c05d067).
- The wasm32 build of `glasnost-app` succeeds.
- `xtask check` passes. `xtask gate` passes, and on wasm32 it matches all nine goldens.

Seams left: the launcher climb now checks an asymmetric game (house 1 cannot leave its island), not the open island war. Q-3118 (#284's stale place), Q-3119 (#285's open-topped firer). The renderer's use of `placed_tick` (#81's drawing half) waits for a render batch. `claims_s0215_r137_deployed.rs` still cites the Q-3100 interim in a comment; its tests pass unchanged, since none uses a type with the named flag.
## Batch S17: the zone row, release, deploy sound, production, disc lasers, projectiles and entry outcomes

Base main c0d6b58, not rebased. Save format 150 and AI game format 126 (f6aa221; renumbered to 145 and 122 at the merge): #309 adds a saved field (`DeployState::sound_requested`, defaulted on load). The wire version is unchanged (12). The sim-gate goldens were regenerated once, after the last fix, with the ignored tests. All nine move from their first tick, because the new deploy field is part of the hashed state. Each test is committed before its fix and fails on the commit before the fix. #313 and #314 share one test commit (ee00ce7). #314's test still fails on #313's fix (30b5e8d). Requests Q-3128, Q-3129 and Q-3130 (924cc2c). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S17: area search, factory release, credits, disc lasers, projectiles and entry outcomes)* (54da55d). 810c55f makes `claims_s0615.rs`'s `loss_and_crash_cancel` take the firer inside with `take_inside`, as play does. It passes before and after the batch. Claim ledger rows: S-0309 claim 11 and step 1c M7b's row for claim 11 (the commit that adds this section).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #245 | Table C's row *zone* applied in an area search, and it read the object's own cell rather than the cell of its reference point (Q-3109) | fixed (3124e96) | S-0309 r144 table C row *zone*: an area search applies no zone row (the visited cell's skip is its only reach-region test); in the ground-layer pass of a map-wide search the row reads `World::reference_point`'s cell | `claims_s0212.rs` (`issue_245_an_area_search_applies_no_zone_row_to_an_object_under_a_deck`), `claims_s0309.rs` (`issue_245_map_wide_zone_row_reads_a_buildings_reference_point_cell`) |
| #308 | At an illegal apron the release gave the clear report and then nothing | fixed (7801c4a) | S-0417 Edge cases and claim 8: after the clear report a vehicle gets claim 8's reply, the run-away from a vehicle factory, then the arrival fallback (`release::arrival_fallback`, split out of `arrival`) | `claims_s0417.rs` (`issue_308_illegal_apron_gives_the_run_away_reply`) |
| #309 | With `quirks.deploy_sound_repeats` off, a landing-first deploy requested the deploy sound at the landing and again at the ground rows | fixed (d1439f4) | S-0517 claim 12: `DeployState::sound_requested` marks the request; the ground rows request the sound only when it is clear; `set_form(Mobile)` and a new deploy order clear it | `claims_s0517.rs` (`issue_309_landing_first_deploy_requests_the_sound_once_with_the_switch_off`) |
| #310 | Which construction entry a filling building of a type that is not a wall type takes | interim, Q-3128 (bf099b6) | S-0406 claim 11 leaves it open. Interim: the entry of any insertion other than a place order (ready mark set, one build-up). The comment at `econ::placement::put_on_map_facing` and S-0406 claim 11's ledger row state it | none |
| #311 | A step payment or refund at the ends of the credit range overflowed | fixed (961d826) | S-0404 and S-0205: credits wrap as signed 32-bit values (`wrapping_sub` in `pay`, `wrapping_sub` and `wrapping_add` in `cancel_item`) | `claims_s0404.rs` (`issue_311_step_payment_wraps_at_the_range_ends`, `issue_311_refund_at_the_top_of_the_range_wraps`) |
| #312 | Two building orders in one tick both started on the building line | fixed (cc14a03) | S-0411 claim 6: a building or defence line that holds an item or a queue takes no start or queue order at application | `claims_s0411_busy_line.rs` (`issue_312_two_building_orders_in_one_tick`) |
| #313 | A killing hit did not lose its object to the disc lasers, so a sinking target kept its disc laser | fixed (30b5e8d) | S-0615 claim 14: loss is an event at the killing hit; `death::mark_killed` calls `disc::object_lost` | `claims_s0615_disc_loss.rs` (`issue_313_a_sinking_target_cancels_the_disc_laser`) |
| #314 | The disc laser's update re-tested `on_map`, so a firer riding open-topped lost its laser | fixed (c70b607) | S-0615 claim 14: loss is recorded only at the killing hit and at `take_inside`; `disc::in_play` tests only that the object exists and is not dead | `claims_s0615_disc_loss.rs` (`issue_314_an_open_topped_passengers_disc_laser_lands`, control `issue_314_boarding_after_the_shot_still_cancels`) |
| #315 | An airburst child's Ω was not its placement distance | fixed (9506ce6) | S-0306 T1: Ω = *d*₃(placement, *K*) for airburst children. The child's arm time stays 0 (Q-3130) | `claims_s0306_airburst_omega.rs` (`issue_315_airburst_child_omega_is_its_placement_distance`) |
| #316 | A hut destroyed by its own bomb's blast did not demolish its bridge | fixed (8d6c4dd) | S-0511 claim 16: the demolition needs no survival condition; `bomb::detonate` takes the hut's cell before the blast and calls `hut::demolish_from` | `claims_s0511.rs` (`issue_316_a_hut_destroyed_by_its_bomb_demolishes_its_bridge`) |
| #317 | The single-precision steps of the flak scatter distance *D* | not done: Q-3129 (924cc2c) | S-0306 claim 4 names single precision but not its steps. *D* stays *d*₃ (equal to any single-precision form below 4096 leptons) | none |
| #318 | A cluster's later detonations kept the firer that its first detonation killed | fixed (d0131fa) | S-0306 claim 21: the attacker rule applies at each detonation; `drop_killed_firer` clears the attacker and house before each cluster detonation and before the airburst children | `claims_s0306_cluster_attacker.rs` (`issue_318_later_cluster_detonation_after_the_firer_is_killed`) |
| #319 | A deployed, deploying or undeploying vehicle accepted a scatter request | fixed (0a092c2) | S-0207 claim 16 (f): `inplace::deployed` or `inplace::in_transition` refuses; (i) reads `transport::letting_out` | `claims_s0207_scatter_deployed.rs` (`issue_319_a_deployed_vehicle_refuses_a_scatter_request`) |
| #320 | Any point delete cancelled every short loop of the route | fixed (2230458) | S-0213 claim 16e: `apply_delete` cancels a short loop only when the deleted point was inside it | `claims_s0213_route_delete.rs` (`issue_320_an_unrelated_delete_leaves_a_short_loop`) |
| #321 | The entry outcomes' armed tests read the ordinary weapons at elite rank | fixed (15b2ff1) | S-0209 Terms *Armed*: the weapons at the current veterancy rank. `occupancy::slot_name` gives the elite name when one exists; used by `walker_armed`, `has_primary`, `primary_warhead_flag` and `primary_hits_ground`. Multi-turret slots and VP29's would-use are seams | `claims_s0209_r55.rs` (`issue_321_elite_only_primary_gives_a_primary_weapon_and_arms_the_elite_walker`, `issue_321_elite_primary_of_no_damage_disarms_the_elite_walker`) |
| #322 | VP5's absorbing and grinding overrides stood above row VP3 | fixed (d474688) | S-0209 claim 5d: the first row from the top wins; VP3 is tested before VP5 in `vehicle_outcome_of` | `claims_s0209_vp3_absorb.rs` (`issue_322_vp3_before_vp5_for_a_vehicle_in_contact_with_its_absorber`) |

**The own AI after #312.** #312 made one AI suite test fail: `antiair::a_defence_out_of_reach_does_not_answer_and_the_new_one_goes_beside_the_drainer` in glasnost-ai fails from cc14a03 and passes at 4da5e08. The anti-air defence's building order came after the script's base building order in the same think, and S-0411 claim 6 now refuses the second order on the same line, at every think. The design is in the S17 section of [docs/dev/simulation.md](../dev/simulation.md), *#312 in the own AI* (6b71693). The fix is `Brain::build_anti_air` (cb2e084): the defence takes its line from this think's other starting orders for that line, and an order it drops does not count as a use. No other part of the think changes.

Lanes compare base c0d6b58 with the batch at cb2e084.

**Oracle arena.** The output is byte-identical to base: M-0001 9 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps.** M7b over seeds 1–16 is identical to base. M6c and M7a pass, but they move. At 4da5e08 (the batch up to #312's test) both are identical to base, and at cc14a03 (#312's fix) both are identical to the batch, so #312 moves them.
- In M6c, seed 1 ends at 8255 (base 9233), seed 3 at 8124 (base 9889) and seed 14 at 7707 (base 10131). Seeds 2, 4, 10, 12 and 15 change their minimums. Medians and maximums move: the most deploy orders go 270 → 510 and the most crates 88 → 170.
- In M7a, only seed 8's counters move. The most berserk weapon orders go 86 → 76 and the most psychic sensors 4 → 6. The gattling top-stage median goes 2614 → 2498, the gas cloud median 154 → 146 and the gas hit median 448 → 433.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; games count from 1 within a map). 4 games end on another tick and 1 changes winner (seed/map/game, base → batch):
- 10/8/2 (sides 0 and 2) 0@37002 → 1@40530.

The others keep their winners: 8/8/3 1@56992 → 1@57422, 9/0/1 0@32010 → 0@33353 and 10/2/2 1@41255 → 1@41269. Seed 7 is identical.

10/8/2 was bisected. 4da5e08 ends it as base does (0@37002), and cc14a03 ends it as the batch does (1@40530). The state hash (`GLASNOST_AI_HASH`) was taken every tick against 4da5e08. The two runs agree up to tick 4800 and part at tick 4801. The object, credit and RNG dumps (`GLASNOST_AI_OBJS`) are the same at ticks 4800 and 4801, so the difference is in production state. House 0's line trace (`GLASNOST_AI_LINES`) shows it. At tick 4801 the building line finishes its AmerParadrop superweapon building and starts a refinery. At 4da5e08 a power plant joins the queue in the same tick, so the queue is 1. In the batch the queue stays 0, as S-0411 claim 6 and S-0404 claim 7 require. The power plant then starts at 6286 rather than 6274, and the games part from there. This is #312, and the change stands.

**Suites.** The AI suite with its ignored tests passes 362 before and after. The sim gate with its ignored tests passes 17 against the regenerated goldens, and cb2e084 moves no golden. On the final tree these all pass:
- `cargo test --workspace --no-fail-fast`, including `quirks::tests::every_spec_quirk_has_a_switch`, which also passes on base;
- `cargo clippy --workspace --all-targets -- -D warnings`, after e2130ed (`contains_key` in two of the batch's tests);
- `cargo fmt --all -- --check`, after the rustfmt commit 4523da2;
- the wasm32 build of glasnost-app;
- `cargo xtask check` and `cargo xtask gate`.

Seams left:
- Q-3128 (#310's interim entry), Q-3129 (#317) and Q-3130 (the arm time of an airburst child, #315).
- #321's multi-turret slots and VP29's would-use test.
- Any other part of the own AI's think that gives a second starting order for a building or defence line in one think now has it refused. The AI suite shows no other case.
## Batch S18: five answers, replays, houses, shroud, linked weapons and spawners

Base main 0d3ed80, not rebased. The answers to Q-3112 to Q-3116 come first (#263, #252, #256, Q-3114 and #272), then issues #333 to #346. Each test is committed before its fix (b30ee1c to acd7006) and fails on the commit before the fix, except the guard tests named below. Two tests were corrected in their own commits before their fixes: 9b896b3 (#272's off test gives the click, not a direct enter order) and c50a372 (#333's boundary test borrows the world once). fcd2fc2 puts back a space that the #342 test commit dropped from a claim 5 citation. The new switch `quirks.bunker_hover_no_arrival` (S-0606 r14) is added in 6a47c22, which also makes main's failing `quirks::tests::every_spec_quirk_has_a_switch` pass. Request Q-3134 (cbf4be2) is filed for #343. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S18: five answers, replays, houses, shroud, linked weapons and spawners)* (81cd855, ef74e20). Claim ledger row: S-0608 claim 5 (7407e2e).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #263 | The refresh as the last stored step begins dropped a new step with the switch on, also after a two-cell curve (the Q-3115 interim) | fixed (c763dd1) | S-0212 claim 20 r100 and Edge cases: the recomputation is made in the step check when exactly one step is stored, and not at a curve over the last two stored steps. With `quirks.path_refresh_keeps_old_step` on, the path becomes the old step and the new path from its second step, so the two choose the track (S-0207 claim 7) and may make a curve; at a moving blocker in its second cell the new first step is taken. Off, the new first step is taken. A failed search keeps the old step | `claims_s0212_refresh_curve.rs` (`issue_263_on_a_curve_over_the_last_two_steps_makes_no_recomputation`, `issue_263_off_a_curve_over_the_last_two_steps_makes_no_recomputation`, `issue_263_on_the_old_step_and_the_new_second_step_choose_a_curve`) |
| #252 | A crashable balloon-hover vehicle kept the switch-on behaviour (the Q-3112 interim), and a vehicle's jumpjet crash end was not modelled | fixed (815002b) | S-0312 r81 claim 16 and *Vehicle crash end*: `quirks.crash_death_weapon_twice` off skips the death weapon at death for a crashable balloon-hover vehicle whose death starts its crash (`jump::crash_end_fires`). At the surface a balloon-hover type sets off its death weapon, exploding or not, and any other type shows claim 9's animations; it leaves play without demolition damage | `claims_s0312_crash_vehicle.rs` (`issue_252_on_a_balloon_hover_vehicle_fires_at_death_and_at_its_crash_end`, `issue_252_off_a_balloon_hover_vehicle_fires_only_at_its_crash_end`, `issue_252_a_balloon_hover_vehicle_that_does_not_explode_fires_at_its_crash_end`, `issue_252_a_crashable_vehicle_of_another_kind_fires_only_at_death`) |
| #256 | A repair already on kept claim 8's paid steps while its building was erased (the Q-3113 interim) | fixed (70b5592) | S-0314 r110 claims 8 and 12: a building being erased or phasing in takes no paid step; the repair state stays on and the steps resume after | `claims_s0314_erasure.rs::issue_256_an_erased_building_takes_no_paid_repair_step` |
| Q-3114 | The steering projectile's deck row kept each cell's own deck height (the interim) | fixed (21d31f7) | S-0306 r91: the deck height is that of the cell of *P*′ + 416 when either cell is a high-bridge cell, as in row 1, with the strict crossing | `claims_s0306.rs` (`q3114_steering_deck_height_is_that_of_the_new_cell`, `q3114_steering_deck_crossing_into_a_lower_cell`) |
| #272 | A hovering vehicle at a tank bunker (Q-3116) | fixed (switch 6a47c22, path 9f886fa) | S-0606 r14 claim 22: `quirks.bunker_hover_no_arrival` on (the default) keeps the current behaviour; off, claim 1's eligibility also asks for a locomotor other than the hovering one | `claims_s0606.rs` (`issue_272_on_a_hovering_locomotor_vehicle_is_eligible_and_takes_contact`, `issue_272_off_a_hovering_locomotor_vehicle_is_refused_at_p8`) |
| #333 | `Replay::run_on` on a save that held its tick's orders issued them again | fixed (a980948) | Every step empties the pending order lists, so a world holding pending orders was saved after its tick's orders were issued; `run_on` does not issue them again for that first tick. A world at a tick boundary is unchanged | `replay_run_on_pending.rs` (`issue_333_run_on_a_save_holding_its_ticks_orders_does_not_issue_them_again`, `issue_333_run_on_at_a_tick_boundary_issues_the_ticks_orders`) |
| #334 | A setup of more than 256 houses wrapped the one-byte house id | fixed (52503cd) | `load::new_game` refuses it with `LoadError::TooManyHouses` before any draw (`load::MAX_HOUSES`, 256; our own input validation). Sandbox setups and a replay's stored setup are covered | `house_limit.rs` (`issue_334_a_setup_of_257_houses_is_refused`, `issue_334_a_setup_of_256_houses_is_accepted`) |
| #335 | Five doc comments in `load.rs` did not describe their code | fixed (443f8fe) | Comments only | none |
| #336 | After an alliance break the cells V15 part A gives as unexplored depended on whether the gap generator or the building was created first | fixed (b4ae723) | S-0409 S1 rows 1, 2, 3 and 5: `shroud::alliance_changed` takes back every changed building sight and every gap field concerned before applying any again, as `whole_map` does | `claims_s0409_alliance.rs` (`issue_336_building_first_cells_unexplored_on_the_break_tick`, `issue_336_generator_first_cells_unexplored_on_the_break_tick`) |
| #337 | An alliance order took back and applied every gap field for every house, so an uninvolved house's pending cell and a satellite owner's count changed | fixed (318b2ac) | S-0409 S1 rows 3 and 5: per field, row 5 runs for a house that left its hostile list and row 3 for one that joined it; each such house's whole-map-revealed mark is cleared. A house in both lists or neither, and a field whose list did not change, are left alone | `claims_s0409_alliance.rs` (`issue_337_pending_cell_of_an_uninvolved_house_survives_an_alliance_order`, `issue_337_satellite_owner_count_is_not_changed_by_an_unrelated_alliance_order`, `issue_337_one_way_alliance_leaves_the_cells_unexplored`) |
| #338 | The 16-bit cover and hostile counts overflowed from a radius of about 102 (S-0409 claim 21's shared state), and a huge radius with the wrap off overflowed its squares | fixed (ff571b7) | The counts are `i32` (postcard encodes an `i16` and an `i32` in range alike, so save bytes do not change). With `quirks.gap_radius_wrap` off, `gap_cells` clamps the radius to `MAX_GAP_RADIUS` (255; our own input validation) and squares in 64 bits | `gap_radius_large.rs` (`issue_338_shared_state_is_covered_again_whatever_the_radius`, `issue_338_a_huge_radius_with_the_wrap_off_starts_and_stops`) |
| #339 | A supporter before its master in the update order beamed one tick early | fixed (e5cb654) | S-0518 claim 6: `linked::recruit` dates the beam max(*d* − 1, 0) ticks ahead for a supporter after the master and one tick more for one before it | `claims_s0518_beam_order.rs::issue_339_supporter_before_the_master_beams_one_tick_later` |
| #340 | A charger taken inside something stayed in its charger lists | fixed (059f5c0) | S-0518 claim 12, S-0203: `objects::take_inside` calls `linked::leaving`, an open-topped transport's passenger included | `claims_s0518_charge_inside.rs` (`issue_340_a_charger_that_boards_a_closed_transport_leaves_the_list`, `issue_340_a_charger_taken_inside_leaves_the_list`) |
| #341 | A building with its own delayed shot pending could be recruited | fixed (175125c) | S-0518 claim 2: `linked::eligible` refuses it, as for a pending beam | `claims_s0518_shot_pending.rs::issue_341_a_building_with_its_own_shot_pending_is_not_recruited` |
| #342 | A relink left the first drain animation on the drainer, and with the switch on the stranded link's end removed the second link's animation | fixed (7407e2e) | S-0608 claim 5 and Edge cases: a second link removes the drainer's animation before the new one starts in both positions of `quirks.drain_relink_strands_previous`; the stranded end removes it only while its building is still the partner | `claims_s0608.rs::issue_342_relink_keeps_one_drain_animation_both_positions` |
| #343 | The launch pass after turns not taken | not done: Q-3134 (cbf4be2) | The code is unchanged | none |
| #344 | A huge spawn or slave count made that many objects at creation | fixed (a6a5ff8) | `spawn::MAX_CREATED_COUNT` (100; our own input validation) caps a spawner's slot list and a slave master's slaves at creation; the importer warns for every type above it | `creation_count_limit.rs` (`issue_344_a_huge_spawn_count_is_bounded`, `issue_344_a_huge_slave_count_is_bounded`) |
| #345 | The doc comment of `linked::break_off` named a return value it no longer has | fixed (941b3d3) | Comment only | none |
| #346 | `absorb::go_in` put the entrant on the ground layer before `objects::take_inside`. By the issue, a vehicle absorbed from a bridge deck then released its claims in the wrong layer; the test did not reproduce that, as it passes on base | changed to the spec's order (233cb77); the defect itself not shown | S-0605 claim 8, S-0209 claim 1: the claims are released in the layer of its position, and the entrant leaves the deck after | `absorb_from_deck.rs::issue_346_vehicle_absorbed_from_a_deck_releases_its_claim` (passes on base: a guard) |

Guard tests that pass on base: `issue_336_building_first_cells_unexplored_on_the_break_tick`, the #272 on test, #346's test, and the switch-default or control tests of #263, #252, #256 and #334.

Formats: save 142 → 151 and AI game 119 → 127 (f71c151, the numbers this batch was assigned); wire stays 12. Renumbered at the merge to save format 147 and AI game format 124. The sim-gate goldens were regenerated once, after the last fix, with the ignored tests. Six of nine move: `duel-diagonal`, `duel-m6c`, `duel-m7a`, `duel-naval`, `duel` and `skirmish`. `combat`, `duel-m7b` and `scripted` are unchanged.

Lanes compare base 0d3ed80 with the batch at f71c151.

**Oracle arena** (`--record all`, two threads). Only M-0001 moves: failing 10 → 11 of 96, failing the win test 3 → 4, side outcomes reproduced 156 → 155 of 171, and stock-replay pairings failing 1 → 2. CAOS against V3 newly fails the win test, our row wins 2/12 → 1/12: side A seed 4 goes from none@6080 to row@2128, and side B seeds 0 and 1 go from row@2128 and row@1520 to none@6080. LTNK against E1 (our row wins 4 → 5/12), MIND against E1 (11 → 10/12), TELE against BFRT (2 → 1/12) and CAOS against E1 (9 → 10/12) move with the same pass or fail. M-0002 (4 of 24), M-0003 (3 of 35), M-0019 (0 of 6), M-0020 (0 of 25) and M-0021 (10 of 50) keep their counts; M-0002's oracle file changes only in the INIT MTNK side A outcomes, and one M-0021 row, failing before and after, changes its text from "some" to "none". In the trace of CAOS against V3, side A seed 4 (`--trace CAOS,V3,A,4`, both binaries), the first difference is A#436's next cell after its destination changed at tick 382 from cell (141, 84) to cell (140, 85), while at (142, 83). Base enters (141, 83) at tick 392 and then (140, 84); the batch enters (141, 84) at tick 394, the diagonal towards the new destination. The trace shows no stored steps. So the move is consistent with #263's refresh, but no state dump confirms it.

**Sweeps** (seeds 1–16). M6c passes; base ended seeds 3 (9889) and 14 (10131) early, the batch ends only seed 14 early (9060), and some medians and maxima moved. M7a passes; every seed's counters move except seeds 3, 6 and 7, seed 10's grinder entry orders and units ground go to 0, and the smallest minimum stays 0. M7b prints the same lines as base.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). All 36 games end on another tick, and 11 change winner (seed/map/sides, base → batch):
- 7/2/0,2 h0@41695 → h1@44328; 7/8/1,2 h0@49516 → h1@41446
- 8/8/0,2 h0@39870 → h1@43326; 8/8/1,2 h1@44845 → h0@39106
- 9/0/0,2 h1@50374 → h0@36117; 9/0/1,2 h1@43968 → h0@67779; 9/8/0,2 h0@28689 → h1@43496
- 10/0/0,2 h1@50625 → h0@23789; 10/2/0,2 h1@41255 → h0@61247; 10/8/0,2 h0@37002 → h1@82455; 10/8/1,2 h0@84803 → h1@37196

Each flip was played alone (`GLASNOST_AI_SIDES`), hashed every 10 ticks to 20000 on both binaries, and dumped tick by tick over the first differing window (`GLASNOST_AI_HASH`, `GLASNOST_AI_OBJS`). All eleven part at one vehicle that holds exactly one stored step on the tick before, as that step's track begins: #263's refresh (S-0212 claim 20). Base keeps a straight track (`e: None`) and drops the new path's first step; the batch keeps the old step and the new path from its second step, so the track takes a next step (`e: Some(..)`), a curve in most cases.

| Flip | First differing tick | Object | Steps the tick before |
|---|---|---|---|
| 7/2/0,2 | 6645 | 845 (house 1 vehicle) | [4] |
| 7/8/1,2 | 4585 | 479 (house 0 vehicle) | [4] |
| 8/8/0,2 | 4941 | 535 (house 1 vehicle) | [0] |
| 8/8/1,2 | 4935 | 518 (house 1 vehicle) | [0] |
| 9/0/0,2 | 5597 | 1397 (house 0 vehicle) | [2] |
| 9/0/1,2 | 5397 | 1330 (house 0 vehicle) | [2] |
| 9/8/0,2 | 4944 | 559 (house 0 vehicle) | [0] |
| 10/0/0,2 | 5597 | 1410 (house 0 vehicle) | [2] |
| 10/2/0,2 | 6607 | 871 (house 0 vehicle) | [6] |
| 10/8/0,2 | 4944 | 558 (house 0 vehicle) | [0] |
| 10/8/1,2 | 4585 | 524 (house 0 vehicle) | [4] |

The change stands: it is what S-0212 r100 gives with the switch on.

**Suites.** The AI suite with its ignored tests passes 362 before and after. The sim gate with its ignored tests passes 17 against the regenerated goldens. The workspace tests pass 8796 and fail none; `quirks::tests::every_spec_quirk_has_a_switch`, failing on base for `quirks.bunker_hover_no_arrival`, passes after 6a47c22. On the final tree these all pass: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` (after the rustfmt commit 9ecbce1), the wasm32 build of glasnost-app, `cargo xtask check` and `cargo xtask gate` (native and wasm32 on all nine goldens).

Seams left:
- #263: the moving-blocker variant at T5 and the switch-off turn have no test of their own; trains have no refresh rule.
- #252: the infantry crash end (the forced *air-death-end* request) is not implemented.
- #334: `sandbox_setup` builds a setup with no cap of its own (refused at `new_game`).
- #337: the building sights keep their rule (a changed sight is taken back and applied for all its houses).
- #339: `linked::recruit` scans the update order once per recruit.
- #343: Q-3134.
- #344: nested spawners still multiply.
- #346: a ground-layer count left at the test map's cell (1, 0) when the vehicle steps onto the deck was seen and not looked into.

## Batch S20: answered requests, creation sets, removed houses and wrapping

Base main eb5d785, not rebased. Save format 153 and AI game format 129 (1c78a4a; renumbered to 149 and 126 at the merge): #291 changes the saved infantry-making state of an animation (`mutation::Making`). The wire version is unchanged (12). Switches `quirks.infantry_animation_stale_marks` and `quirks.path_congestion_ground_record`, both on by default, with the regenerated schema (8f6fd0a). Each test is committed before its fix. #370's test went into #368's test commit (4a3d17b) by mistake. Request Q-3149 (0af009c). The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S20: answered requests, creation sets, removed houses and wrapping)* (e4502cf). Claim ledger rows: S-0611 claim 15, S-0406 claim 11, S-0409 claims 15 and 17 and S-0301 claim 4 (ba9baf3), and S-0412 claims 10 and 14 (2480d60). S-0212 has no claims ledger file, so claim 19's change is recorded here only.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #291 | An infantry-making animation marked a fixed spot and layer taken at its start, also when it moved (Q-3121) | fixed (998fba7) | S-0611 r11 claim 15: each update marks the standing position of the animation's point (`mutation::Stand`); with `quirks.infantry_animation_stale_marks` on an earlier mark is not freed and a removal other than a successful end attempt frees nothing; off, the earlier mark is freed in its own record. A bouncing conversion animation flies as an ordinary one | `claims_s0611_moving.rs` (`claim_15_bouncing_marks_stay_and_a_landing_makes_none`, `claim_15_bouncing_marks_freed_with_the_switch_off`, `claim_15_end_attempt_in_flight`, `claim_15_end_attempt_in_flight_switch_off`, `claim_15_attached_follows_its_owner`, `claim_15_owner_removed`, `claim_15_removed_by_another_rule`, `claim_15_high_free_misses_the_ground_mark`, `claim_15_save_load_mid_flight`) |
| #305 | The congestion marks read the cell ahead and the 5 x 5 block in one layer (Q-3122 interim) | fixed (0f380f6) | S-0212 r107 claim 19: the cell ahead is read on its deck when it is a bridge cell and the blocked unit is on a deck or the levels differ by more than 3; an empty cell ahead considers the first unit of the block heading to it; part (ii) reads the ground record, or with `quirks.path_congestion_ground_record` off the layer read in the cell ahead. The 5 x 5 fallback and (ii)'s non-empty record (`Occ::empty`) also change ordinary ground play, and the switch does not cover them. They make duel-m6c's full run on seed 2 lose the charger orders and support beams (S-0518), and the own AI's land-cell test fail its coverage guard (11016 of 11016 found): 429f6ac passes and 0f380f6 fails. At integration, `M6C_SEED` went to 3 (0e616ad) and the land-cell test took four units per house (aaa8328) | `claims_s0212_congestion_deck.rs` (`deck_blocker_cell_counts_by_the_ground_record`) |
| #310 | A filling building of a type that is not a wall type took the entry of an insertion other than a place order (Q-3128 interim) | fixed (70503c4) | S-0406 r14 claim 11: it enters in its construction state exactly as the placed building, the ready mark clear at entry (`placement::put_with`) | `claims_s0406.rs` (`claim_11_non_wall_fill_enters_as_placed`, `claim_11_non_wall_fill_buildup_twice_off`, `claim_11_non_wall_fill_timings`) |
| #366 | A map entry dropped for its cell, and a map wall building, left their slot objects, slaves and records | fixed (04d9173) | S-0611 claim 14 with S-0009 claim 38 and S-0112 claim 7: `objects::place_map_objects` calls `objects::discard_unplaced` | `claims_s0009.rs` (`claim_38_dropped_entry_takes_its_creation_set`), `claims_s0112.rs` (`claim_7_map_wall_building_leaves_no_creation_set`) |
| #367 | The waypoint reveal reached a house with the whole map revealed | fixed (dba38b4); Q-3149 open | S-0011 claim 26 and S-0409 claim 17: `triggers::reveal` skips a player house whose whole-map-revealed mark is set. Whether it reaches one through an ally (S-0409 claim 2) is Q-3149 | `claims_s0409_reveal_action.rs` (`claim_17_reveal_action_skips_a_marked_house`, control `claim_17_reveal_action_reaches_an_unmarked_house`) |
| #368 | With `quirks.entered_by_country_only` off the entered-by event still read a country | fixed (f692fa0) | S-0011 Edge cases and claim 23: off, the value is resolved by `mp_triggers::resolve_house` and matches the causing object's owner | `triggers.rs` (`entered_by_player_position_with_the_switch_on`, `entered_by_player_position_with_the_switch_off`) |
| #369 | A starting unit or the base unit that found no cell left its creation set | fixed (04d9173) | S-0611 claim 14 with S-0009 claim 45: both branches of `starting::give` call `objects::discard_unplaced` | `claims_s0009.rs` (`claim_45_failed_start_leaves_no_creation_set`) |
| #370 | The trigger coverage tables listed actions 6, 14 and 111 and event 49 as not built | fixed (4f0aebf) | S-0011 claim 24, S-0508 claims 10 to 13, S-0217 claim 1 and S-0520 claim 7: `action_coverage` and `event_coverage` say implemented; the `triggers.rs` module doc and the comment in `judge` are corrected. No behaviour change | `triggers.rs` (`coverage_counts_change_owner_eviction_hunt_and_the_crate_event`, in 4a3d17b) |
| #371 | Making peace cleared a target of the new ally but kept the firer's burst counter | fixed (a8013e0) | S-0301 claim 4 with S-0414 claim 12: `victory::drop_allied_targets` sets the burst counter to 0 | `claims_s0414_peace_burst.rs` (`making_peace_drops_the_target_and_resets_the_burst_counter`) |
| #372 | A failed trigger restoration left its creation set | fixed (04d9173) | S-0611 claim 14 with S-0524 claim 9: `mp_triggers::replace` calls `objects::discard_unplaced` | `claims_s0524.rs` (`claim9_failed_replacement_leaves_none_of_its_creation_set`) |
| #373 | A satellite the defeat's own sweep took stayed active: `radar::on_defeat` cleared the recheck the sweep asked for | fixed (6750d96) | S-0409 claim 15: the recheck stays due | `claims_s0409.rs` (`claim_15_satellite_lost_by_the_defeat_itself`) |
| #374 | The game-start crates ran through the whole count with no crate place free | fixed (89ca901) | S-0520 claim 1: `crates::place_at_start` ends its loop when no place is free or no crate overlay is named | `claims_s0520.rs` (`claim_1_start_placements_past_the_last_place_are_skipped`) |
| #375 | A defeated house's buildings could go to a civilian house the cleanup removed | fixed (6750d96) | S-0414 Terms: the civilian house of `victory::destroy_all` skips removed houses | `claims_s0601.rs` (`claim_14_defeat_skips_a_removed_civilian_house`) |
| #376 | A removed house's country widened the starting-unit budget pool | fixed (6750d96) | S-0009 claim 45: `starting::give`'s live countries skip removed and passive houses | `claims_s0009_removed_pool.rs` (`claim_45_removed_house_country_stays_out_of_the_budget_pool`) |
| #377 | The radar's object tests read the flat grid whatever `quirks.shroud_on_flat_grid` said | fixed (221c7ed) | S-0410 claims 6 and 7 with S-0110 Edge cases: `compose` passes `View::on_flat_grid` (`winner_among_on`, `shown_on`); `listed_on` and `Arrivals::observe_on` give the listing under the switch; the plain forms keep the flat grid | `claims_s0410.rs` (`claim_7_draw_time_test_follows_the_flat_grid_switch`), `claims_s0410_flat_listing.rs` (`claim_6_listing_follows_the_flat_grid_switch`, committed after the fix as a check of the new listing forms) |
| #378 | The setup comment and the S-0412 ledger called the game-start crates' draws a seam | fixed (2480d60) | S-0412 claims 10 and 14 with S-0520 claims 2 and 3: each crate is placed with its own draws, before the secret-lab draws. No behaviour change | none |
| #379 | The loading reply compared reach regions left stale by a category change in the tick | fixed (f4ddab5) | S-0507 claim 5 with S-0212 claim 5: `transport::load_request` calls `regions::ensure` first | `claims_s0507_load_regions.rs` (`claim_5_loading_reply_reads_regions_after_a_category_change`) |
| #380 | The protection's remaining time overflowed for a duration near the 32-bit limit | fixed (054d763) | S-0513 claims 25 and 29: `protect::start` and `tint_turn` compute duration - (t - start) | `claims_s0513_protect.rs` (`claim25_quirk_off_with_a_duration_at_the_integer_limit`, `claim29_envelope_with_a_duration_at_the_integer_limit`) |
| #381 | Rules-fed sums and products overflowed in the dev profile | fixed (9a3d561) | S-0205 claim 1: wrapping arithmetic at the delayed-kill radius, the walker's frame and expiry, the kill value, the price sums and the flying search's far cells | `claims_s0205_rules_wrap.rs` (`delay_kill_radius_wraps_for_a_huge_spread`, `pad_term_sum_wraps`, `free_unit_sum_wraps`, `walker_frame_wraps_for_huge_running_frames`, `kill_value_wraps_for_a_huge_cost`, `flying_search_far_cell_wraps_for_a_huge_radius`) |

#305's ground rules (0f380f6) change duel-m6c's play. On the gate's seed 2, house 1 collapses and the game plays 11758 ticks. The full run then no longer exercises the charger orders or the support beams (S-0518). A bisection gives the cause: 429f6ac passes and 0f380f6 fails.

At first, update mode wrote only the six goldens before duel-m6c (170b645), because it stops at duel-m6c's exercise check. The same change makes the own AI's land-cell test fail its coverage guard (`found < compared`, 11016 of 11016), while all of its equality checks pass.

Two test-only commits fix clippy in this batch's tests: cc123cc (#374, no thread) and 51a1e68 (#379).

At S20's integration, the main session decided the gate and the AI test as in S10:
- `M6C_SEED` goes from 2 to 3 (0e616ad), the scenario seed changed because #305 changed the play. `try_seeds` was run over seeds 1 to 16, counting ticks played out of 12 000:

  | Seed | Ticks played | Missing abilities |
  |---|---|---|
  | 1 | 9297 | radiation exits, charger orders, support beams |
  | 2 | 11758 | charger orders, support beams |
  | 3 | 12 000 | none (the first that passes) |
  | 4, 6 and 8 to 14 | 12 000 | none |
  | 5 | 7375 | support beams |
  | 7 | 8193 | none, but the game ends early |
  | 15 | 12 000 | charger orders |
  | 16 | 8088 | support beams |

- All nine goldens were written in update mode; duel-m6c, duel-m7a and duel-m7b were committed (7a390e6), and the other six were unchanged.
- The land-cell test takes the first four units of each house instead of three (aaa8328). The old queries are all still asked, so the new set is a strict superset of them. The equality checks and the guard are unchanged. An earlier try, m6c setup seed 3 with three units, still found a cell in 11016 of 11016 queries, and was not kept.

Lanes. Base is eb5d785. The after-lanes ran at 170b645 and 235c471; the commits after those change only two test files. The final checks ran at 51a1e68. Outputs are in `local-lane/sim-issues/s20/`; contents were compared by exit code only.

| Lane | Base | After |
|---|---|---|
| Arena | exit 0 | exit 0 |
| M6c, M7a and M7b sweeps (seeds 1 to 16) | exit 0 | exit 0 |
| AI suite with ignored tests | exit 0 | exit 101 (`--no-fail-fast`): 361 of 362 pass; only `brain::land_cell_tests::the_land_cell_of_the_square_is_the_land_cell_of_the_map` fails; skirmish 7 of 7 pass |
| Sim-gate with ignored tests | exit 0 | exit 101 (`--no-fail-fast`): 14 of 17 pass; `golden.rs` (`debug_prefix_matches_golden`, `full_run_matches_golden`) and `m6c.rs` (`every_m6c_ability_is_exercised_and_the_replay_reproduces_the_game`) fail, all at duel-m6c; m7a, m7b, margins and play_fixes pass |
| AI quick lane, seeds 7 to 10 | exit 0 | exit 0; all 4 seed outputs differ from base under `cmp` (not traced) |

Final checks:

| Check | Result |
|---|---|
| Workspace tests | 8909 pass and three fail: the land-cell test, `golden.rs` `debug_prefix_matches_golden`, and `quirks::tests::every_spec_quirk_has_a_switch`. The last fails on main since S-0606 r14 (`bunker_hover_no_arrival`). |
| clippy, fmt, the wasm32 build of glasnost-app, `cargo xtask check` | pass |
| `cargo xtask gate` | exit 1 at duel-m6c |

After the integration changes (at aaa8328; outputs in `local-lane/sim-issues/s20/`):

| Check | Result |
|---|---|
| Sim gate with ignored tests (`sim-gate-seed3.txt`) | exit 0, 17 of 17 |
| AI suite with ignored tests (`ai-seed3.txt`) | exit 0, 362 of 362 |
| `cargo xtask gate` (`xtask-gate-seed3.txt`) | exit 0 |

The workspace run was not repeated, because nothing outside the gate, the AI test module and the docs changed. The quirk test stays the one known workspace failure on main.

Seams left:
- Q-3149 (#367).
- #291: a bouncing conversion animation is given no infantry-making state; the mech locomotor maps to no layer rule; no path detaches an attached animation while it plays.
- #305: the 5 x 5 search of `considered` has no test of its own.
- #305: seed 2 of duel-m6c no longer exercises the charger orders or the support beams. The gate now plays seed 3. Of seeds 1 to 16, seeds 1, 2, 5, 7, 15 and 16 fail the gate's check, by ending early or missing an ability.
- #374's reworked test (cc123cc) has no timeout. If the fix regresses, it runs `i32::MAX` placements and hangs instead of failing.
- `slaves::civilian_recipient` and `garrison::civilian_house` still range over removed houses.
- S-0009 claim 29's References rule is not built (#368).
- The app's radar still calls the plain `observe` (the flat-grid reading, #377).
- #381's walker expiry site has no test of its own.
- Target clears other than `drop_allied_targets` were not audited for the burst counter (#371).
- An in-play wall building turned into overlay is still removed raw (`placement.rs`), by design: it is not the newest creation and has no creation set.

## Batch S22: riding lifters, the wet tile, deaths inside and the kill turn

Base main a7a4651, not rebased. The batch's worktree began at 59b2357 and was fast-forwarded to a7a4651 before any work; the one extra commit is render-only (RP3). The answers to Q-3119 (S-0607 r50, #285) and Q-3141 (S-0513 r124, #364) come first, then issues #382 to #385. Each test is committed before its fix (3be4284, e3f7ff1, d9f6eff, 8b1da78, 4246042) and fails on the commit before the fix, except the guard and control tests named below; no fix commit changes a test. 781dcaa imports the four waterfall role starts (`Roles::waterfalls`) that #364's wet tile needs. 98b5040 is rustfmt on three of the new test files, and a598163 adds a citation `cargo xtask check` asked for. Requests Q-3158 and Q-3159 (a18274c) are filed for #383 and #384, and Q-3160 (8341bef) for #364's landing cell with no tile; all three have interims. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S22: riding lifters, the wet tile, deaths inside and the kill turn)* (74a41ab).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #285 | A lifter riding an open-topped transport could not seize, and a firer that left play or started to cloak after its shot still seized (Q-3119) | fixed (9c7e124) | S-0607 r50 claims 2, 7, 12, 14 and 21: `lift::seize_hit` accepts a firer riding an open-topped transport; the wave's firer side counts such a rider as in play. A projectile's saved `seize_void` flag is set by `objects::take_inside` and by `cloak::machine::start` for the firer's projectiles, and a marked detonation seizes nothing. The new `lift::leaving_play` (called from `take_inside`) clears the dropper of units this object dropped earlier, ends its grip (the dropped unit keeps it as dropper), ends the grip on a gripped unit that goes inside, and ends every wave of the object | `claims_s0607_r50.rs` (`an_on_map_walking_lifter_seizes`, `an_open_topped_passenger_seizes`, `a_riding_firers_wave_lasts_while_it_rides`, `the_grip_lasts_while_the_transport_moves`, `a_firer_that_boards_after_the_shot_seizes_nothing`, `a_firer_that_went_inside_and_came_out_seizes_nothing`, `a_firer_that_started_to_cloak_seizes_nothing`, `a_lifter_that_boards_while_gripping_drops_and_stays_the_dropper`, `a_dropper_that_boards_before_the_landing_is_no_dropper`) |
| #364 | `quirks.warp_naval_road_cell` off had no effect (Q-3141) | fixed (import 781dcaa, fate dc05f8e); a landing cell with no tile is Q-3160 | S-0513 r124 claim 38 and Terms (*wet tile*): `warp::wet_tile` tests the cell's tile against the shore (42), water (14) and four waterfall (4 each) roles; an unset role counts from −1 with `quirks.unset_role_range` on and gives no tile with it off; no catalogue, no wet tile. The road cell is read by land type, and with the switch off a road cell on a wet tile is no road cell. The no-credit sink-or-kill test reads the wet tile instead of the water land type, which also changes the default position: a non-infantry member on a wet tile that is not water land now sinks there with no credit instead of being killed | `claims_s0513_r124.rs` (`quirk_on_a_ship_on_a_wet_road_cell_sinks_with_no_credit`, `quirk_off_a_ship_on_a_wet_road_cell_survives`, `both_positions_a_ship_on_a_dry_road_cell_is_killed_with_no_credit`, `a_waterfall_tile_is_wet`, `an_unset_shore_role_counts_from_the_unset_start`) |
| #382 | A defeat hit on a garrison occupant counted the loss twice | fixed (310bab8) | S-0414 claims 5 and 21, S-0508 claim 19: `death::kill_in` gives a garrison occupant, a transport's passenger and an absorbed passenger the new uncredited `death::passenger_removed` (health 0, the killed mark, the hit's destruction raises, removal at once); `killing_hit` has already credited the hit | `claims_s0414_occupant_loss.rs::defeat_hit_on_an_occupant_inside_counts_one_loss` |
| #383 | A passenger killed inside stayed in its carrier as a dead object, and died a second time with its carrier | fixed (310bab8); what a transport's passenger shows is Q-3158 | S-0414 claim 15, S-0605 claim 15: the passenger leaves at once (an absorber's count drops); `death::passenger_death` only removes a passenger that is already dead (no second mark, credit or raises). The removal of a transport's passenger is the Q-3158 interim | `claims_s0414_passenger_defeat.rs` (`control_human_passenger_is_removed_by_its_house_defeat`, `not_human_passenger_is_removed_by_its_house_defeat`, `passenger_created_before_its_transport_dies_once`) |
| #384 | A carrier its own bomb killed went on with its turn: it took a walking step and could board | fixed (2613f2d); a carrier that starts to crash or sink is Q-3159 | S-0519 claim 4, S-0203 claim 6 (c): after the bomb check and the strike look, `object_turn` returns for a carrier that is gone or dead unless it is crashing (jumpjet or aircraft) or sinking. Those keep the rest of their turn as the Q-3159 interim. A building its bomb kills also ends its turn there | `claims_s0519_bomb_kill_turn.rs` (`a_carrier_killed_by_its_own_bomb_takes_no_walking_step_after_it`, `a_carrier_killed_by_its_own_bomb_does_not_board_in_that_turn`) |
| #385 | A designator whose target was lost in the houses' part of the tick kept its strike until a later turn | fixed (182c4c0) | S-0519 claim 19, S-0203 claim 7: `held::holders_drop` runs `strike::turn` for each holder it changed, after its loops | `claims_s0519_strike_house_loss.rs::a_target_lost_in_the_house_update_cancels_in_that_tick` |

Guard and control tests that pass on base: #285's `an_on_map_walking_lifter_seizes` (control) and `a_firer_that_boards_after_the_shot_seizes_nothing` (guard), #364's `both_positions_a_ship_on_a_dry_road_cell_is_killed_with_no_credit` (guard), and #383's `control_human_passenger_is_removed_by_its_house_defeat` (control). Every other new test fails on the commit before its fix (7 of 9, 4 of 5, 3 of 4, 2 of 2 and 1 of 1).

Formats: save 147 → 155 and AI game 124 → 131 (8e1b2e3, the numbers this batch was assigned; renumbered to 150 and 127 at the merge); wire stays 12. #285 adds a saved field (`Projectile::seize_void`, serde default false). The tile catalogue with its waterfall starts is not saved; it is given again on load. The sim-gate goldens were regenerated once, after the last fix, with the ignored tests. Eight of nine move (`combat`, `duel`, `duel-diagonal`, `duel-m6c`, `duel-m7a`, `duel-m7b`, `duel-naval`, `skirmish`), and `scripted` is unchanged. Each changed run of samples returns to the old hashes after it (the longest, in `combat`, is 231 samples), and every file ends on its old hashes. The state hash serializes the whole world, so the new per-projectile byte changes the hash while a projectile is in flight; no golden diverges in behaviour.

Lanes compare base a7a4651 with the batch at a598163.

**Oracle arena** (`--record all`, two threads). Every output file is byte-identical to base: M-0001 11 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps.** M6c, M7a and M7b over seeds 1–16 pass and print the same lines as base.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). Every game prints the same lines as base apart from timings: none ends on another tick or changes winner. Seed 10 map 2 sides 0 and 2 is not over at 120000 ticks in both.

**Suites.** The AI suite with its ignored tests passes 362 before and after. The sim gate with its ignored tests passes 17 against the regenerated goldens. In the workspace tests the only failure is `quirks::tests::every_spec_quirk_has_a_switch`, which fails on base and here for the same four switches (`path_congestion_ground_record`, `offview_release_origin_cell`, `infantry_animation_stale_marks`, `crate_place_by_cell`); it is not this batch's. On the final tree these all pass: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` (after 98b5040), the wasm32 build of glasnost-app, `cargo xtask check` (after a598163) and `cargo xtask gate` (native and wasm32 on all nine goldens).

Seams left:
- Q-3158 (#383: what a transport's passenger killed inside shows), Q-3159 (#384: whether a carrier that starts to crash or sink from its own bomb moves in that turn) and Q-3160 (#364: whether a landing cell with no tile is wet; the interim compares the no-tile marker with the role ranges, so in practice not wet).
- #285: an open-topped transport that itself goes inside does not end its riding lifters' grips; entering a bunker has no `leaving_play` hook (the wave test reads the bunker, and a bunkered vehicle cannot fire a seizing hit); claim 2's digging-in row cannot be reached (no tunnelling locomotor).
- #364: asset packs converted before 781dcaa read the waterfall starts as unset until they are converted again. A world with no tile catalogue (synthetic maps) has no wet tiles, so a member in claim 38's no-credit branch on water land there is now killed instead of sunk.
- #382/#383: `spawn::start_crash`'s call of `kill` is not a death inside and is unchanged.

The P3 census rule: no fix makes or moves a spawn controller, bomb, charger list, dropper or aircraft-kind back-reference of `World::census`. `lift::leaving_play` only clears droppers and ends grips, and its drop goes through `end_grip`, which already calls `census::forget`. #385's `strike::turn` runs inside `holders_drop` while its census is live; a cancel only empties the linked aircraft's ammunition and queues their retreat, which makes none of those references (aircraft state is never noted).

## Batch S24: the answers to Q-3158, Q-3159, Q-3160 and Q-3164

Base main 5a9c6dc, not rebased. The answers to Q-3160 (S-0513 r126, #364), Q-3159 (S-0519 r21, #384), Q-3158 (S-0312 r88 claim 34, #383) and Q-3164 (S-0214 r84, #358) replace batch S22's interims, followed by seam (d) of S22's #364 work: a teleporting vehicle in the warp's leaving stage (S-0207 r85 claim 20 a). Each test is committed before its fix (6116b2e, 686dc73, 45be90a, ff24bd0, fbd423a). Each fails on the commit before its fix, except the guard and control tests named below. No fix commit changes a test. bcd32a1 adds the quirk switch `closed_passenger_death_effects` (S-0312 claim 34 (f)) with the schema refresh. 99e979c is rustfmt, and 3fb5c37 regenerates the goldens. Requests Q-3182 (bunker entry has no leaving-play hook) and Q-3183 (an open-topped transport that goes inside keeps its riders' lift grips) are filed in 562531d for two of S22's seams; neither has an interim code change. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S24: the answers to Q-3158, Q-3159, Q-3160 and Q-3164)* (bb127c5, ae1cdaa).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #364 | Whether a warp landing cell with no tile is wet (Q-3160) | already right; citation only (1374a18) | S-0513 r126 Terms and claim 38: `warp::wet_tile` compares the stored tile number with each role range as it stands. A cell with no tile stores the marker 65535, so it is wet only when a range reaches the marker, and an unset range counted from −1 never does. That was S22's interim | `claims_s0513_r126.rs` (`no_range_reaches_the_marker_a_no_tile_road_cell_is_a_road_cell_under_both_settings`, `a_waterfall_range_reaching_the_marker_makes_the_no_tile_cell_wet`, `the_water_range_reaches_the_marker_from_its_last_start`, `an_unset_role_range_does_not_reach_the_marker`) |
| #384 | A vessel its own bomb set sinking took a ship-locomotor step and no descent in the detonation turn (Q-3159) | fixed (1058902) | S-0519 r21 claim 4: `tick::object_turn` reads the sinking state again after `bomb::turn`. A vessel that started to sink in this turn takes its first descent (`naval::sinking_turn`) at the movement step instead of its locomotor. The aircraft's first crash fall step and the jumpjet's first crash update already came at the movement step | `claims_s0519_r21_kill_turn.rs` (`a_vessel_its_bomb_sinks_descends_in_the_detonation_turn`, `an_aircraft_its_bomb_kills_takes_its_first_crash_step_in_that_turn`, `a_jumpjet_its_bomb_kills_takes_its_first_crash_update_in_that_turn`) |
| #383 | A transport's passenger killed inside was removed with no effects (S22's interim for Q-3158) | fixed (7345d51) | S-0312 r88 claim 34: `death::kill_in` sends an infantry or vehicle passenger to the new `passenger_killed_inside`. After the raises it leaves the passenger list (`transport::leave_list`), and its death effects come at the transport's position when the transport is open-topped, else where it went inside (new saved `TransportState::went_in`, set by `transport::board`). Inside an open-topped transport an infantry passenger's death sequence plays off the map. Inside a closed one it is removed at once, with no sequence, corpse or crash, while the rows' animations and row 4's virus effect still come. A vehicle passenger is removed at once. With `quirks.closed_passenger_death_effects` off, a closed transport's passenger shows nothing. `transport::kill_passengers` and `escape` pass over a listed passenger that is already dead. Aircraft passengers, garrison occupants and absorbed passengers keep `passenger_removed` | `claims_s0312_r88_inside.rs` (`closed_transport_the_death_animation_comes_where_the_passenger_went_in`, `open_topped_transport_the_death_animation_comes_at_the_transport`, `the_switch_off_closed_transport_passengers_show_nothing`, `open_topped_transport_infantry_plays_its_death_sequence_off_the_map`, `closed_transport_infantry_takes_no_sequence_and_no_corpse`, `a_vehicle_passenger_is_removed_with_no_explosion`, `a_transport_dying_during_its_passengers_hit_passes_over_it`) |
| #358 | A passive-neutral garrisonable building was engineer-friendly when its type is not repairable or it is vehicle-like (Q-3164) | fixed (f129a8b) | S-0214 r84 table I: `targeting::engineer_friendly` applies the repairable and not-vehicle-like tests to both cases. The cases are allied with the actor (garrisonable or not, erased or not) and of a passive neutral house, garrisonable and not being erased | `claims_s0214_r84_engineer_friendly.rs` (`a_passive_neutral_building_that_is_not_repairable_is_not_engineer_friendly`, `a_passive_neutral_building_that_is_vehicle_like_is_not_engineer_friendly`, `an_allied_building_is_engineer_friendly_garrisonable_or_not_erased_or_not`) |
| seam (d) | A teleporting vehicle in the warp's leaving stage kept an offered destination as part (b)'s warp-pending immobilization | fixed (f5148ee) | S-0207 r85 claim 20 (a) and the glossary's *leaving stage* (it counts as being erased throughout, the expiry update included): `teleport::take_destination` drops the destination for `superweapons::being_erased` for every kind. The infantry-only leaving branch (S-0208 claim 19, r66) folds into it | `claims_s0207_r85_leaving_stage.rs::a_vehicle_in_the_leaving_stage_drops_an_offered_destination`; `claims_s0207_r34.rs::claim_20_b_warp_pending_immobilization_keeps_the_landing_point` now checks part (b) with a pending warp |

Guard and control tests that pass on base:
- #364: all four tests, since the code was already right.
- #384: the aircraft and jumpjet tests.
- #383: `closed_transport_infantry_takes_no_sequence_and_no_corpse` and `a_vehicle_passenger_is_removed_with_no_explosion`.
- #358: `an_allied_building_is_engineer_friendly_garrisonable_or_not_erased_or_not`.
- Seam (d): the edited `claims_s0207_r34` test.

Every other new test fails on the commit before its fix: 1 of 3 for #384, 5 of 7 for #383, 2 of 3 for #358 and 1 of 1 for seam (d).

Formats: save 150 → 158 and AI game 127 → 134 (7345d51, for `TransportState::went_in`, serde default none); wire stays 12. The AI constant is a forced caller edit. The sim-gate goldens were regenerated once, after the last fix, with the ignored tests (3fb5c37). All nine move from their first sample, because the state hash serializes the whole world and so includes the new field; the behaviour lanes below are unchanged.

Lanes compare base 5a9c6dc with the batch at 99e979c (3fb5c37 changes only the goldens).

**Oracle arena** (`--record all`, two threads). Every output file is byte-identical to base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps.** M6c, M7a and M7b over seeds 1–16 pass and print the same lines as base (smallest minimum 0, 2 and drain 3).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). Every game prints the same lines as base apart from timings. Each ends on the same tick with the same winner, and all 36 are over before 120000 ticks.

**Suites.**
- The AI suite with its ignored tests passes 364 before and after.
- The sim gate with its ignored tests passes 17 on base. Here it passes 17 against the regenerated goldens.
- In the workspace tests the only failure is `quirks::tests::every_spec_quirk_has_a_switch`. On base it names `offview_release_origin_cell` and `closed_passenger_death_effects`; here it names only `offview_release_origin_cell`, which belongs to render RP8.
- On the final tree these all pass: `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --check` (after 99e979c), the wasm32 build of glasnost-app, `cargo xtask check` and `cargo xtask gate` (native and wasm32 on all nine goldens).

Seams left:
- Q-3182 (bunker entry has no `lift::leaving_play` hook) and Q-3183 (an open-topped transport that itself goes inside keeps its riders' lift grips) are filed, with no interim change.
- #364: S22's old-pack seam, looked at again. No in-tree code reads a stored tile catalogue back; the catalogue is built again at load. A catalogue serialized before 781dcaa has no waterfall key, so it would read the four waterfall role starts as unset (−1, the serde default) until it is converted again. With `quirks.unset_role_range` on, each unset waterfall range then counts from −1, so tiles 0 to 2 would count as wet. No spec decides how an old pack is upgraded; S-0513's Import mapping says only that an absent key is unset.
- #383:
  - Row 7's conversion of an infantry passenger killed inside still runs `mutation::convert` on the off-map victim, and may place the new unit at `went_in`. Claim 34 (d) does not list conversion among the outcomes it replaces, and no test covers it.
  - `transport::sync` still moves a closed transport's passengers with it (S-0507 claim 15 gives that only for open-topped ones). It is left as it is, because `went_in` makes it irrelevant to the death effects.
  - A killed passenger's gunner weapon number is not reset when it leaves the list (before this batch too).
- Seam (e): `objects::remove_with`'s `warp_appear_free` stays provisional under S-0208's Open questions.
- Seam (d): `superweapons::warp_leaving` has no caller left in the simulation; it is kept as a public query.

The P3 census rule: no fix makes or moves a spawn controller, bomb, charger list, dropper or aircraft-kind back-reference of `World::census`. #383's bomb goes through `bomb::carrier_killed`, and the passenger's removal through `objects::remove`, which already calls `census::forget`. `went_in` is a position, and the passenger list is not a census reference. #384, #358 and seam (d) touch none of these references.

## Batch A6c: #284 and #367

Step M8 A6c, base main e2c2efe, not rebased. The answers to Q-3152 (S-0520 r17, #284) and Q-3149 (S-0409 r17, #367) come first; the arena triage that follows them is in [a6c-arena-triage.md](a6c-arena-triage.md). Each test is committed before its fix (604e663, a49bc4c) and fails on the commit before the fix (both of #284's on df1facd, #367's new test on a49bc4c; logs in `local-lane/m8/a6c/`); no fix commit changes a test. df1facd is a test correction of its own: under r17 a crate that holds no place stays when it is picked up, so the two effect tests that picked up such a crate (`pick` and `a_pickup_with_crates_on_places_a_replacement` in `claims_s0520_effects.rs`) now give their crate a place with the new fixture `crate_common::placed_crate_at`; they pass on base and after. b1c588e is rustfmt on the r17 test file. No request is filed for these two. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (M8 A6c: the answers to Q-3152 and Q-3149)* (58e253b).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #284 | A named crate overlay that holds no place and is no map crate was removed by its pickup (S21's interim for Q-3152) | fixed (ad8a830) | S-0520 r17 claim 11: `crates::free_and_remove` removes the crate only when the pickup frees a place (and its overlay is a named crate overlay) or when it is a map's own crate. Otherwise the crate stays: the pickup gives its effect and the random replacement, and the crate can be collected again. With `quirks.crate_place_by_cell` off, a named crate overlay that no crate placement placed frees no place, so it stays even with an occupied place on its cell | `claims_s0520_r17.rs` (`a_named_crate_overlay_without_a_place_stays_when_picked_up`, `a_named_crate_overlay_no_placement_placed_and_a_place_on_its_cell`, both switch positions) |
| #367 | The waypoint reveal made for an unmarked house reached an allied house whose whole-map-revealed mark is set (S20's interim for Q-3149) | fixed (5e79c09) | S-0409 r17 claims 2 and 17: the new `shroud::waypoint_reveal`, called by `triggers::reveal` for each house it reveals for, gives the cells to that house and, under ally reveal, only to allied player houses whose mark is clear | `claims_s0409_reveal_action.rs::claim_17_reveal_action_skips_a_marked_ally` (the marked ally keeps its seen state and counts; the unmarked house is explored) |

Formats: neither fix adds saved or hashed state, so save 150, AI game 127 and wire 12 are unchanged (the step's numbers 157 and 133 are not used). The sim-gate goldens pass unchanged.

Lanes compare base e2c2efe with the batch at b1c588e.

**Oracle arena** (`--record all`, four threads). Every output file is byte-identical to base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps.** M6c, M7a and M7b over seeds 1–16 pass and print the same lines as base (smallest minimum 0, 2 and drain 3).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). Every game ends on the same tick with the same winner as base, and all 36 are over before 120000 ticks; only the timings differ.

**Suites.** The AI suite with its ignored tests passes 362 before and after. The sim gate with its ignored tests passes 17 on base and here, against the unchanged goldens.

Seams left:
- #284: S21's seam stands: an overlay written straight into a cell without `overlays::place_overlay` or `crush::remove_overlay` does not forget the cell's crate owner.
- #367: only the waypoint reveal (trigger action 17) goes through `waypoint_reveal`. `shroud::area_reveal` is unchanged (claim 2's ally rule without the mark test); after this fix no simulation path calls it, only the AI crate, the app and tests.

The P3 census rule: neither fix makes or moves a back-reference of `World::census`. The crate state holds cells and place numbers only, and the waypoint reveal writes shroud cells only.

## Batch F3 (foundations)

Step foundations F3 (#153, the deterministic side), base main 5a9c6dc, not rebased. Design section first: [docs/dev/simulation.md](../dev/simulation.md), *Simulation (foundations F3: extreme modded values on the deterministic side, #153)* (6b14456, with the fire point's sites added in 8d8d6ee). Each test is committed before its fix and fails on the commit before it in a debug build: the clock's five tests on 873a1e3 (overflowing subtractions in `update_held`, `advance` and `initial_stage`), the normalisation's on b94f46d (an overflowing multiply), the importer's two on 8e09208 and 2a711e0 (no such warning), the fire point's three on 9803e58 (overflowing adds at each of the three sites). No fix commit changes a test; 3cb5dfa is rustfmt. No request is filed. Where S-0205 claim 1 says the result wraps, the code wraps and carries no `design:` marker; the importer's thresholds are design choices (`design: sim (#153)`) and only warn.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #153 (clock) | The animation clock's tick difference, loop limit λₑ − *a*, lower turn λₛ − *a*, new-pass stage and reversed start were unchecked 32-bit expressions: a debug build panicked on extreme modded loop ends, starts or ticks | fixed (08b4dc9) | S-0205 claim 1: the five results wrap (`wrapping_sub`), so debug builds agree with release builds. The view plays through the same `anim_clock::Player`, so `mapview::slots::read_anim` takes no bound and is not changed | `claims_s0205_anim_clock_wrap.rs` (five tests: `anim_reverse_raw_loop_end` on and off, `anim_pingpong_lower_turn` off) |
| #153 (rates) | `infantry::normalised`'s 8*b* overflowed for a base delay above 2²⁸. The animation rate path is exact (*d* ≤ 900); a building's body delay and the low-funds speech delay can pass larger values | fixed (dee72ea) | S-0205 claims 1 and 2: 8*b* wraps, the quotient truncates toward zero | `claims_s0205_normalised_wrap.rs` (wrapped products; the largest exact product and two in-range values unchanged) |
| #153 (art Height) | A building image's art `Height` beyond ±128 levels passed the view's `LEVEL_LIMIT` unwarned | warned (78ce9d2) | `VIEW_LEVEL_LIMIT` 128 (`design: sim (#153)`): a warning for each image value beyond it, the value kept | `values_past_the_views_bounds_warn` (129 warns; −128 and an infantry −129 quiet) |
| #153 (pip row) | A pip row longer than the view's `PIP_ROW_LIMIT` passed unwarned: S-0111 T2's ammunition capacity, passenger capacity or mind-control primary's damage | warned (78ce9d2) | `VIEW_PIP_ROW_LIMIT` 1024 (`design: sim (#153)`), located at Ammo, Passengers or the weapon's Damage; the value kept | `values_past_the_views_bounds_warn` (1025 warns for each kind; 1024 quiet) |
| #153 (PipWrap) | A `PipWrap` above the view's pip-row bound passed unwarned | warned (78ce9d2) | `VIEW_PIP_ROW_LIMIT` 1024 (`design: sim (#153)`); the value kept | `values_past_the_views_bounds_warn` (1025 warns; 1024 quiet) |
| #153 (scroll) | A `ScrollMultiplier` outside S-0101 F9's range (negative, or 448 × μ past 2³¹ − 1) passed unwarned | warned (78ce9d2) | Two warnings (`design: sim (#153)`), 448 × μ formed in double precision rounded toward zero and truncated, as the view's `scroll::magnitude`; the value kept | `a_scroll_multiplier_outside_f9s_range_warns` (−1, −0.5, 4793491 and 1e30 warn; 1, 0, −0, 50% and 4793490 quiet) |
| #153 (bracket) | A unit's `PixelSelectionBracketDelta` beyond R12's ±2²⁰-pixel unit-bar bound passed unwarned | warned (78ce9d2) | `VIEW_PIXEL_LIMIT` 2²⁰ (`design: sim (#153)`), infantry, vehicles and aircraft, each layer's value; kept | `values_past_the_views_bounds_warn` (1048577 warns; −1048576 quiet) |
| #153 (turret anchor) | A building's `TurretAnimX`, `TurretAnimY` or `TurretAnimZAdjust` beyond ±2²⁰ pixels passed unwarned; the view adds them to its turret anchor and depth bias reading the rules text, so the importer cannot bound them. A building's pixel-shifted fire point (S-0313 claims 7 and 8) added the shift's lepton form with a plain add, which panicked in a debug build for a `TurretAnimX/Y` or `PrimaryFirePixelOffset` past the range | warned (78ce9d2); fixed (bade2c2) | `VIEW_PIXEL_LIMIT` 2²⁰ (`design: sim (#153)`), each layer's value, kept; S-0205 claim 1: the three adds in `combat::position::fire_point` wrap | `values_past_the_views_bounds_warn` (±1048577 warn; ±1048576 quiet); `claims_s0205_fire_point_wrap.rs` (three tests) |

Formats: no fix adds saved or hashed state, so save 150, AI game 127 and wire 12 are unchanged (the step's numbers 160 and 136 are not used). The sim-gate goldens pass unchanged.

Lanes compare base 5a9c6dc with the batch at 8d8d6ee (arena and quick lane from release `glasnost-headless` binaries of each).

**Oracle arena** (`--record all`, two threads). Every output file is byte-identical to base; the printed rows differ only in paths and run times.

**Sweeps.** M6c, M7a and M7b over seeds 1–16 print the same lines as base, apart from build lines and run times.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). Every game ends on the same tick with the same winner as base; the reports differ only in run times.

**Suites.** The AI suite with its ignored tests passes 364 before and after. The sim gate with its ignored tests passes 17 on base and here.

Seams left:
- The clock's stage advance, step back and image frame *a* + *k* saturate where S-0205 claim 1 wraps; the view's `slots::first_frame` holds a reversed first frame to 32 bits, so at λₑ = −2³¹ the view's and the clock's first frames differ.
- In `combat::position`, the garrisoned muzzle shift (S-0313 claim 6), claim 1's translation-column adds and σ*L* are plain arithmetic.
- The view reads a building's art `Height` from the last image, while the importer warns for each image value.
- The bracket bound ±2²⁰ is R12's, which is not on this base; the warning names the bound, not R12's code.

## Batch F4 (foundations)

Step foundations F4 (a debug-build overflow sweep and F3's seams), base main a5df3f6, not rebased. Design section first: [docs/dev/simulation.md](../dev/simulation.md), *Simulation (foundations F4: a debug-build overflow sweep and F3's seams)* (34d6a8a). Each test is committed before its fix (80c54cf, 96e398c) and fails on the code before it in a debug build: the clock's three by assertion (saturation never panicked), the fire point's three by an overflow panic at each site. No fix commit changes a test. No request is filed. Every fix follows S-0205 claim 1 (an integer result wraps), so none carries a `design:` marker.

**The sweep.** The headless tools and the sim gate were built as release builds with overflow checks on (`CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS=true`, target `found4-ovf`; the build log shows `overflow-checks=on` for `glasnost_sim`) from the base code, and run over the AI quick lane (seeds 7 to 10, maps 0, 2 and 8, 36 games), the sim gate with its ignored tests (17 tests, among them the M6c, M7a and M7b sweeps over seeds 1 to 16) and the oracle arena (`--record all`). **No run hit an overflow panic.** The lane and the arena catch a game's panic and go on, so each run lists every game's. The overflow lane's 36 games end on the same tick with the same winner as base, and its arena output files are byte-identical to base's.

| Item | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Sweep | No overflow panic in the quick lane, the sim gate with its ignored tests (with the three sweeps) or the arena | nothing to fix | — | — |
| Seam: animation clock | `anim_clock::Player::advance`, `Player::step_back` and `Player::frame` saturated *k* + σ, *k* − σ and *a* + *k*. The stage is an integer (S-0109 Terms) and S-0109 states no clamp | fixed (9040eef) | S-0205 claim 1: the three results wrap. At the ends of the range only: an advance from 2³¹ − 1 goes on at −2³¹, and a wrapped frame can land inside the sprite | `claims_s0205_anim_stage_wrap.rs` (`an_advance_past_the_range_wraps`, `a_step_back_past_the_range_wraps`, `the_image_frame_past_the_range_wraps`) |
| Seam: fire point | `combat::position::fire_point`'s garrisoned muzzle shift (S-0313 claim 6) and `claim1_point`'s three adds of F1's shift to the base point were plain adds, σ*L* a plain product: debug builds panicked past the range | fixed (c620391) | S-0205 claim 1: the adds wrap, and σ*L*, an integer converted to single precision by F1, wraps (−1 × −2³¹ is −2³¹) | `claims_s0205_fire_point_claim1_wrap.rs` (`claim_1s_shift_past_the_range_wraps`, `the_lateral_part_of_an_odd_burst_counter_wraps`, `a_garrison_muzzle_shift_past_the_range_wraps`) |
| Seam: `slots::first_frame` (crates/app, not this batch's) | The view's fallback first frame of a reversed animation forms *a* + λₑ − 1 in 64 bits and clamps it (design: render (#153)); the clock now wraps both λₑ − 1 and *a* + *k*. They differ wherever *a* + λₑ − 1 passes the range, not only at λₑ = −2³¹ | left | — | — |

Formats: no fix adds saved or hashed state, so save 151, AI game 128 and wire 12 are unchanged (the step's numbers 164 and 140 are not used). The sim-gate goldens pass unchanged.

Lanes compare base a5df3f6 (a base worktree and target of its own) with the batch at c620391, whose later commits are docs only. Arena and quick lane come from release `glasnost-headless` binaries of each.

**Oracle arena** (`--record all`, two threads). Every output file is byte-identical to base: M-0001 10 of 96 pairings failing, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6 rows, M-0020 0 of 25, M-0021 10 of 50, both sides.

**Sweeps.** M6c, M7a and M7b over seeds 1–16 print the same lines as base, apart from build lines, paths and run times (smallest minimum M6c 0, M7a 2, M7b drain 3).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games). Every game ends on the same tick with the same winner as base.

**Suites.** The sim gate with its ignored tests passes 17 on base and here.

Seams left:
- The view's `slots::first_frame` (crates/app) clamps a reversed animation's fallback first frame *a* + λₑ − 1, where the clock now wraps it; the two differ wherever that sum passes the 32-bit range.
- `combat::position::claim1_point` forms a non-turreted building's aim direction from `tp.x - rp.x` and `tp.y - rp.y` as 32-bit subtractions before widening; positions on a map are far from the range, and no run reached it.

## Batch S25: the seams left open

Base main a0cf7c7, not rebased. The batch takes the seams that S24 left for #383 and S20 left for #375, #368, #374, #381, #305 and #371. It hands back early, before S20's #291, S18's rests (#344, #339, #346, #263, #252) and A6c's (#284, #367), which stay untouched. Each test is committed before its fix (e8a1326, fc21d02, 847baaf, 716d7eb, d8d7200). The new tests were run on the base code before any fix was written, and each fails there except the guard and control tests named below. Logs are in `local-lane/sim-issues/s25/tests-on-base.txt` and `tests-on-base-burst.txt`. No fix commit changes a test. acb8792 is rustfmt on three of the test files, and 6eb638a regenerates the goldens. No request is filed. The design is in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S25: the seams left open)* (95c7f2b, corrected in dfd32a3).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #383 (a) | Row 7's conversion of a passenger killed inside ran `mutation::convert` as for an object on the map. A crashable new unit started a crash, and a refused conversion freed (or recorded) a standing position the passenger does not hold | fixed (4c04b79) | S-0312 claim 34 (c) and (d): `convert` takes `inside`, which `death::kill_in` passes for a passenger. Inside, a refused conversion frees no position and records none, and the new unit starts no crash. The conversion animation still comes where claim 34 (b) puts the death effects: at an open-topped transport, or where the passenger went inside a closed one | `claims_s0312_r88_inside_conversion.rs` (`closed_transport_the_conversion_animation_comes_where_the_passenger_went_in`, `open_topped_transport_the_conversion_animation_comes_at_the_transport`, `closed_transport_a_crashable_converted_passenger_starts_no_crash`, `a_refused_conversion_inside_frees_no_position_it_does_not_hold`) |
| #383 (b) | `transport::sync` moved a closed transport's passengers with it | fixed (f8f611f) | S-0507 claim 15 and S-0312 claim 34 (b): `sync` moves the passengers of an open-topped transport only. A closed transport's passengers keep the position they had when they went inside, until a way out sets a new one. This change alone moves the goldens and one quick-lane game (below) | `claims_s0507_closed_positions.rs` (`a_closed_transport_does_not_move_its_passengers`, `an_open_topped_transport_moves_its_passengers`) |
| #383 (c) | A gunner vehicle whose last passenger was killed inside kept that passenger's weapon number | fixed (cef2531) | S-0308 claim 6 and S-0312 claim 34 (a): `transport::leave_list` sets an emptied gunner transport back to weapon number 0 and gives a held temporal target back, as a passenger leaving it by getting out does | `claims_s0308_gunner_killed_inside.rs` (`a_gunner_vehicle_whose_last_passenger_is_killed_inside_goes_back_to_weapon_number_0`, `a_gunner_vehicle_keeps_its_weapon_number_while_a_passenger_remains`) |
| #383 (d) | `superweapons::warp_leaving` had no caller in the simulation, only the r66 landing test | removed (11b00ca); test-only edit (18555cf) | S-0513 and S-0208 claim 19: the r66 test reads the warp member's leaving stage itself, and the query goes | `claims_s0208_r66_warp_landing.rs` (unchanged assertions) |
| #375 | `slaves::civilian_recipient` and `garrison::civilian_house` still ranged over removed houses | fixed (d74d13a) | S-0312 claim 30, S-0009 claim 28 and S-0414 Terms: both skip a removed house, as `victory::destroy_all` already did | `claims_s0402_slaves.rs` (`s0312_claim_31_removed_civilian_house_is_no_recipient`, `s0508_removed_civilian_house_is_not_the_civilian_house`) |
| #368 | S-0009 claim 29's References rule is not built | left | Not worth doing in this batch: the rule needs the header-pair marks, which the scenario loader does not build. It is a feature of its own, not a seam | — |
| #374 | The game-start crate placements test had no timeout | test-only (383f4be) | S-0520 claim 1: the test runs on a watchdog thread and fails after 60 seconds instead of hanging. The thread and its channel belong to the harness, as in `spawn_cycle.rs` | `claims_s0520.rs::claim_1_start_placements_past_the_last_place_are_skipped` |
| #381 | The walker expiry site had no test of its own | test-only (7d40840) | S-0205 claim 1 at S-0615 claim 11's expiry stage: running frames whose 8 R + 1 overflows wrap | `claims_s0205_walker_expiry_wrap.rs::walker_expiry_stage_wraps_for_huge_running_frames` |
| #305 | The 5 x 5 search of an empty cell ahead had no test of its own | test-only (8871844) | S-0212 claim 19: the first unit of the block whose locomotion heads to the cell ahead is considered, and its remaining steps are marked. Control: a unit not heading there is not found | `claims_s0212_congestion_deck.rs::an_empty_cell_ahead_considers_a_unit_of_the_block_heading_to_it` |
| #371 | Target clears other than `drop_allied_targets` kept the firer's burst counter | fixed (e70fe74) | S-0301 claim 4: `spawn::set_own_target` (when a set target is cleared), `explicit_clear_own_target`, `redirected_to_none` and `aircraft_return` set the counter to 0. A change from one target to another keeps it | `claims_s0301_burst_clear.rs` (`clearing_the_own_target_resets_the_burst_counter`, `an_explicit_clear_resets_the_burst_counter`, `a_redirect_to_none_resets_the_burst_counter`, `a_change_to_another_target_keeps_the_burst_counter`, `going_inside_resets_the_burst_counter`, `a_slot_object_returning_to_its_spawner_resets_the_burst_counter`) |

Guard and control tests that pass on base:
- #383 (a): the two conversion animation tests.
- #383 (b): the open-topped test.
- #383 (c): the test with a passenger remaining.
- #371: `a_change_to_another_target_keeps_the_burst_counter`.
- #383 (d), #374, #381 and #305: test-only, so these pass at once.

Every other new test fails on the base code: 2 of 4 for #383 (a), 1 of 2 for (b), 1 of 2 for (c), 2 of 2 for #375 and 5 of 6 for #371.

Formats: no fix adds saved or hashed state, so save 151, AI game 128 and wire 12 are unchanged. The sim-gate goldens were regenerated once, after the last fix, with the ignored tests (6eb638a). Four of the nine move (duel, duel-m7a, duel-naval and duel-diagonal), all because of #383 (b). Each moves only in windows while passengers ride a closed transport, and matches its old hashes again after each window, the last samples included.

Lanes compare base a0cf7c7 with the batch at acb8792 (dfd32a3 changes only the design text, 6eb638a only the goldens). Logs are in `local-lane/sim-issues/s25/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). Every output file is byte-identical to base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps** (`m6c-base.txt`/`m6c-after.txt`, and likewise for m7a and m7b). M6c, M7a and M7b over seeds 1–16 pass and print the same lines as base (smallest minimum 0, 2 and drain 3).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). 35 games end on the same tick with the same winner as base. Seed 7, map 0, game 3 ends at 47257 instead of 46922, with house 1 winning both times. With #383 (b) disabled, that game ends at 46922 again (`quick-without-1b-7-map0.txt`). Player-house dumps every 5000 ticks (`dump-without-1b.txt`, `dump-after.txt`) differ only in passenger positions up to tick 40000, and in on-map objects by tick 45000. All 36 games are over before 120000 ticks.

**Suites.** The AI suite with its ignored tests passes 364 before and after (`ai-suite-base.txt`, `ai-suite-after.txt`), and no AI golden moves. The sim gate with its ignored tests passes 17 on base (`sim-gate-base.txt`). After #383 (b) it fails at duel tick 798 until the goldens are regenerated (`sim-gate-after.txt`); with (b) disabled it passes 17 (`sim-gate-without-1b.txt`). Against the regenerated goldens it passes 17 (`sim-gate-final.txt`). These all pass on the final tree:
- `cargo test --workspace --no-fail-fast`, with no failures;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo fmt --check`;
- the wasm32 build of glasnost-app;
- `cargo xtask check`;
- `cargo xtask gate`, native and wasm32 on all nine goldens.

Seams left:
- #383 (b): something reads a closed transport passenger's kept position, since the quick-lane game above changes. Every way out sets the position, and the AI's ferry and gunner code read none. The reader is not traced.
- #383 (c): `transport::removing` has the same gap for a passenger removed from the list in other ways. It was not changed, since no test reaches it.
- #368: S-0009 claim 29's References rule (above).
- Not taken: S20's #291 (a bouncing conversion animation is given no infantry-making state; the mech locomotor maps to no layer rule; no path detaches an attached animation while it plays), S18's #344, #339, #346, #263 and #252, and A6c's #284 and #367.
- `docs/dev/play-notes.md` still names `warp_leaving` in its history.

The P3 census rule: no fix makes or moves a back-reference of `World::census`. The conversion inside goes through `mutation::convert`'s existing replacement, `sync` and `leave_list` touch positions and weapon numbers, the civilian house scans read house flags, and the burst counter is a combat field.

## Batch S27: header pairs as start position marks

Base main a5df3f6, not rebased. The batch builds S-0009 claim 29 (r45), *Header pairs beyond the eighth act as start position marks*: the rest of #368, which S25 left. The requests Q-3213, Q-3214 and Q-3215 come first (6c1896c), then the design in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S27: header pairs as start position marks)* (556b181, corrected with this section). Each part's tests are committed before its fix (8e803cc, ad4273d, 2be69bb, 0d8cd69). To show them failing, the base worktree was checked out at each test commit and the tests run there: `local-lane/sim-issues/s27/red-importer.txt`, `red-early.txt`, `red-late.txt` and `red-references.txt`. No fix commit changes a test. 0eb2ef6 is rustfmt on the importer and three test files. The tests use synthetic maps: four start waypoints and house 2 as a neutral house where claim 28's passivity is needed. No quirk switch is added: `quirks.header_pairs_as_start_marks` already gates the import, and M13b's variants E and F are measurements, not switches.

| Part | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Import | The importer read pairs 9 … 16 already. An empty value already counts as missing, since the document keeps no empty value. Nothing warned about the cases claim 29 leaves unspecified | fixed (b7823b3; 87c3e30 drops its redundant empty-value filter) | S-0009 claim 29 and S-0003 claim 9: new `Decoder::header_mark_warnings`, called by `decode` after `header()`. It warns about a mark at position 8 or above. It warns about a mark at or above the map's K start waypoint markers. It warns about pairs that mark every one of positions 0 … K − 1 with fewer than K house numbers | `convert::map::tests` (`claim_29_empty_header_pair_is_missing`, `claim_29_unspecified_case_warnings`) |
| Early phase | Pairs 9 … min(N_h, 16) marked nothing; `load` only noted "not implemented" | fixed (18bceba) | S-0009 claim 29, first bullet: new `starts::apply_pairs`, `highest_held`, `free_marks` and `marks_apply`. In `load::new_game` the pairs are applied, then each non-passive chooser writes its number over its position. Each non-passive holder takes its highest position. The rest go through `assign`, which counts every mark other than −1 below N as taken and writes the house's number on the position it gives. With every position taken a house gets none, with a note. A mark at N or above is ignored with a note (Q-3215). The marks are carried to the late phase, and the cleanup frees a removed house's marks (Q-3213) | `claims_s0009_r45_header_marks.rs` (`variant_a_pair_9_places_both_players`, `variant_c_pairs_above_the_count_are_not_applied`, `variant_d_pair_10_marks_positions_2_and_3`, `variant_e_the_highest_position_held`, `variant_f_a_mark_that_names_no_house`, `a_chooser_wins_over_the_pair_early`, `an_unread_component_keeps_the_free_mark_early`, `marks_at_positions_n_and_above_are_ignored`, `every_position_taken_gives_no_start`, `eight_pairs_read_change_nothing`) |
| Late phase | The late phase did not apply the pairs again | fixed (6d5b703) | S-0009 claim 29, second bullet, and claim 28: after the cleanup the pairs are applied again over the early marks, so an unread component keeps the early mark. Each house that is neither late-passive nor removed takes its highest held position; the rest are assigned against list 3. With `quirks.double_start_resolution` off the one resolution applies the pairs, then the choosers (Q-3214) | `claims_s0009_r45_header_marks_late.rs` (`variant_b_pair_9_frees_chosen_positions_in_the_late_phase`, `an_unread_component_keeps_the_early_mark_late`, `a_house_passive_early_holds_its_mark_late`, `a_removed_house_frees_its_position`, `one_resolution_applies_the_pairs_once`) |
| References | "Player at position X" read only claim 21's holder | fixed (2d3f7aa) | S-0009 claim 29, third bullet, and claim 21: new `World::start_marks` (saved and hashed; `Some` only with N_h ≥ 9) and `starts::player_at`. With marks, the answer is the house whose number is mark k (a passive house may be named; a removed house is none, Q-3213). Otherwise it is claim 21's holder. `mp_triggers::resolve_house` and the owner lookup in `objects::place_map_objects` call it. The objects are placed with the early marks | `claims_s0009_r45_header_marks_references.rs` (`variant_e_references_follow_the_marks`, `variant_f_a_mark_that_names_no_house_is_empty`, `references_read_the_late_marks`, `references_at_positions_n_and_above_are_empty`, `a_reference_may_name_a_passive_house`, `a_removed_house_is_no_reference`, `eight_pairs_read_keep_the_holders`, `marks_survive_a_save_round_trip`) |

Guard and control tests that pass on base:
- Import: `claim_29_empty_header_pair_is_missing` (the document already drops an empty value).
- Early phase: `variant_c_pairs_above_the_count_are_not_applied` and `eight_pairs_read_change_nothing` (guards). `an_unread_component_keeps_the_free_mark_early` also passes on base. That is a coincidence: at seed 1 the draw gives the same positions as a free mark.
- Late phase: `a_house_passive_early_holds_its_mark_late` passes at 2be69bb, which already holds the early fix, since the carried marks keep the passive house's mark.
- References: `references_at_positions_n_and_above_are_empty`, `a_removed_house_is_no_reference`, `variant_f_a_mark_that_names_no_house_is_empty` and `eight_pairs_read_keep_the_holders`. Under claim 21 these give the same answer.

Every other new test fails on the code before its fix: 1 of 2 for the import, 7 of 10 for the early phase, 4 of 5 for the late phase and 4 of 8 for the References rule.

Formats: `World::start_marks` is new saved and hashed state, so the save format goes from 151 to 167 and the AI game format from 128 to 143. Wire stays 12. The sim-gate goldens were regenerated once, in 2d3f7aa, with the ignored tests. All nine move in their hash lines only, from the new field's layout. No sim-gate scene sets a header, so N_h is the default −1 and the field is `None` in each.

Lanes compare base a5df3f6 with the batch at 0eb2ef6. Logs are in `local-lane/sim-issues/s27/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). Every output file is byte-identical to base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps** (`m6c-base.txt`/`m6c-after.txt`, and likewise for m7a and m7b). M6c, M7a and M7b over seeds 1–16 pass and print the same lines as base, apart from build and timing lines.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). All 36 games end on the same tick with the same winner as base, so no winner needs tracing. All are over before 120000 ticks.

**Suites.** The AI suite with its ignored tests passes 364 before and after (`ai-suite-base.txt`, `ai-suite-after.txt`), and no AI golden moves. The sim gate with its ignored tests passes 17 on base (`sim-gate-base.txt`) and 17 after, against the regenerated goldens (`sim-gate-after.txt`). These all pass on the final tree:

## Batch S26: bunkers and riders for the lift, and S25's seams

Base main a5df3f6, not rebased. The batch builds S-0607 r52's answers to Q-3182 and Q-3183 (#285) and takes S25's three seams of #383. Of the seams S25 did not reach, it adds the test for A6c's #367. It hands back #284, #291, #344, #339, #346, #263 and #252 untouched. The design text came first (740b7ab), and each test was committed before its fix (e8a4ef0, ea096f5, 033ce1e). The new tests were run with the base worktree checked out at 033ce1e, the last test commit, which holds every new test and no fix (`tests-at-033ce1e.txt`).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #285 (a) | A lifter that took a vehicle bunker as its destination and then gripped lost its grip on the approach. The enter processing gives the bunker as destination again (S-0606 claim 4, S-0215 claim 29), and `econ::assign_destination` took every non-empty destination as an acceptance | fixed (f94e4df) | S-0607 claim 12 r52 (*Lifters and vehicle bunkers*): a destination it already holds is not an acceptance. `assign_destination` ends a grip by row 2 only when the destination it gives differs from the one held | `claims_s0607_r52.rs::a_lifter_that_grips_while_it_goes_into_a_bunker_keeps_its_grip` |
| #285 (b) | The wave's end test counted a bunkered firer or target as out of play. A wave ended when its target went in, and a bunkered firer's wave ended in the update that made it | fixed (b16d9a7) | S-0607 claim 21 r52 (*Vehicle bunkers and riders*): going into a vehicle bunker is not leaving play. `lift::in_play` and `firer_in_play` drop the bunker test, and the comments of `take_inside` and `lift::leaving_play` no longer name a bunker | `claims_s0607_r52.rs` (`a_wave_lasts_when_its_target_goes_into_a_bunker`, `a_bunkered_firers_wave_lasts`) |
| #285 (c) | The rest of r52 (Q-3182 and Q-3183): a dropper that goes into a bunker, a seizing shot in flight as its firer is bunkered, the update order at the let-out, and the riders of an open-topped transport that goes into a bunker, a closed transport or a building | already right | The bunker entry calls no leaving-play hook, and `objects::take_inside` acts only on the object going in. Guards only | `claims_s0607_r52.rs`, ten tests |
| #383 (S25 seam 1) | Something reads a closed transport passenger's kept position: quick lane seed 7, map 0, game 3 ends at 47257, not 46922 | request (Q-3207) | Traced with player-house dumps, then per-tick watches, at fc21d02 against f8f611f. The first on-map difference is at tick 41271. A hover transport at rest in *guard* holds as its destination the infantry unit it took by S-0507 claim 5, which has just boarded it. The destination's point is that passenger's position. With the kept position it is one cell off the transport's own point, so S-0207 claim 26's guard row no longer removes it. The transport holds it until the AI re-orders it, between ticks 41300 and 41350. The read is right under the approved claims (S-0312 claim 34 (b)), and no claim says whether that destination should stay. Code unchanged | — |
| #383 (S25 seam 2) | `transport::removing` took a removed passenger off the list without the gunner reset | fixed (596e560) | S-0308 claim 6: `removing` goes through `leave_list` | `claims_s0308_gunner_killed_inside.rs` (`a_gunner_vehicle_whose_last_passenger_is_removed_goes_back_to_weapon_number_0`, control `…keeps_its_weapon_number_while_a_passenger_remains_after_a_removal`) |
| #383 (S25 seam 3) | `docs/dev/play-notes.md` still named `warp_leaving` | docs (6f6fa05) | Now names `superweapons::warp_pending_or_leaving` | — |
| #367 | `shroud::area_reveal` keeps claim 2's ally rule with no mark test | test-only (033ce1e) | S-0409 claim 17 r17 leaves a marked ally out of the waypoint reveal only. No simulation path calls `area_reveal` yet (the drop-zone reveal, S-0011 claim 36, is not built) | `claims_s0409_reveal_action.rs::claim_17_another_specs_area_reveal_still_counts_for_a_marked_ally` |

Guard and control tests that pass on base: ten of the thirteen r52 tests, the gunner test with a passenger remaining, and #367's test (test-only). Every other new test fails on the base code: 3 of 13 for #285 and 1 of 2 for the gunner removal.

Formats: no fix adds saved or hashed state, so save 151, AI game 128 and wire 12 are unchanged. No switch is added (r52 names none). No golden moves.

Lanes compare base a5df3f6 with the batch at 681a547 (27ad1d5 and 681a547 change only the request and the design text). Logs are in `local-lane/sim-issues/s26/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). Every output file is byte-identical to base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps** (`m6c-base.txt`/`m6c-after.txt`, and likewise for m7a and m7b). M6c, M7a and M7b over seeds 1–16 pass and print the same lines as base (smallest minimum 0, 2 and drain 3).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). All 36 games end on the same tick with the same winner as base.

**Suites.** The AI suite with its ignored tests passes 364 before and after (`ai-suite-base.txt`, `ai-suite-after.txt`). The sim gate with its ignored tests passes 17 before and after (`sim-gate-base.txt`, `sim-gate-after.txt`). On the final commit these also pass:
- `cargo test --workspace --no-fail-fast`, with no failures;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo fmt --check`;
- the wasm32 build of glasnost-app;
- `cargo xtask check`;
- `cargo xtask gate`, native and wasm32 on all nine goldens.

Seams left:
- `an_unread_component_keeps_the_free_mark_early` passes on base by the draw at seed 1. Its assertion holds, but it does not tell the free mark apart from the draw.
- Two choosers of one position: the later chooser in house order writes last and holds it. The lobby never allows this, and claim 29 does not say.
- The importer's every-position warning counts distinct house numbers. It cannot know which numbers a game's players will have.
- Q-3213 (a house the cleanup removes), Q-3214 (`double_start_resolution` off) and Q-3215 (marks at positions N and above) await answers. Their interim rules are in the design.

The P3 census rule: marks are positions' house numbers, not object back-references, so no fix makes or moves a back-reference of `World::census`.

- Q-3207 (above): what a transport's destination naming the unit that boarded it does.
- #284: A6c's other overlay writers still skip `crates::overlay_changed`. These are `crush.rs`, `load.rs`, five sites in `overlays.rs`, the high, low and hut bridge code, `econ/placement.rs`, two sites in `econ/resource.rs` and `econ/sell.rs`. Not taken.
- Not taken: S20's #291 and S18's #344, #339 (perf, not measured), #346, #263 and #252.

The P3 census rule: no fix makes or moves a back-reference of `World::census`. The wave fix reads object state, the held-destination test compares two destinations, and `removing` reuses `leave_list`, which touches the passenger list and weapon numbers.

## Batch S28: gas and the chaos drone on a high bridge (#424)

Base main 1242a50, not rebased. The batch checks S-0613 revision 24's claim 20 (the chaos drone on a high bridge) and claim 19's new bullet (gas made under the deck) against measurement M-0026, on a synthetic map with a synthetic three-cell high bridge and invented rules (`tests/r24_deck_common`). The engine already does what the claims and M-0026's rows say, so every new test is a guard and no code changes. The tests are committed in 5bfff4e and ecfc938. Each was run in the base worktree checked out at its own commit, and all of them pass there (`local-lane/sim-issues/s28/tests-on-base-5bfff4e.txt`, `tests-on-base-ecfc938.txt`). No request is filed, and `quirks.gas_through_bridge_deck` keeps its default.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #424 claim 20 | A deploy-fire area berserk drone on a high bridge's deck, or under it, against M-0026's drone rows and S-0306 claim 3 (a) | guard (5bfff4e) | None needed. Area fire takes the firer's cell, an invisible straight projectile placed at the centre of that cell at the deck height detonates there in its first turn, and the area damage reads the deck layer (S-0306 claim 3 (a), S-0303 claims 4 and 21). The drone makes no particle. Each pulse's animation is made in its cell at the deck height and keeps that position until it ends. The infantry on the deck in the next cell is berserk from the first pulse on, and each pulse sets its time again. The infantry on the ground under it is never reached | `claims_s0613_r24_deck.rs` (`claim_20_a_drone_on_the_deck_pulses_at_the_deck_height`, `claim_20_a_drone_under_the_deck_pulses_at_the_deck_height`) |
| #424 claim 19 | Gas made on and under a high bridge's deck, against M-0026's deck and under rows | guard (ecfc938) | None needed. Gas made at the deck height, with its first update on an even tick, is 3 below the deck after its first odd-tick move. It sinks 7 leptons per two ticks once v_z is −5, and reaches 5 above the ground 122 ticks after it is made, with the deck 416 above it. Gas made under the deck settles 5 above the ground from its first odd-tick move on. Both hit the ground layer only, never the object on the deck. Every row equals F4 and F5. A virus death on the deck makes its clouds at the dead unit's last position with v_z 0 | `claims_s0613_r24_gas.rs` (`claim_19_gas_made_on_the_deck_follows_m0026`, `claim_19_gas_made_under_the_deck_stays_under_it`, `claim_19_a_virus_death_on_the_deck_makes_its_cloud_at_the_deck_height`) |
| #424 local lane | A local-lane comparison against M-0026's own setup | skipped | M-0026's Setup says "The cells, the levels and the bridge's extent are not given here", and it gives no tick for the deploy order. Rebuilding the setup would mean guessing the test cell, so the comparison is not built | — |

Formats: no saved, hashed or wire state changes, so save 151, AI game 128 and wire 12 are unchanged and no golden moves.

Lanes compare base 1242a50 with the batch at ecfc938, whose engine code is the base's. Logs are in `local-lane/sim-issues/s28/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). `diff -r` of the two output directories is empty (`arena-diff.txt`): M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing, as at S25.

**Sweeps** (`m6c-base.txt`/`m6c-after.txt`, and likewise for m7a and m7b). M6c, M7a and M7b over seeds 1–16 pass. Their seed and summary lines are identical to base (smallest minimum 0, 2 and drain 3; `*-lines.txt`).

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). All 36 games end on the same tick with the same winner as base (`quick-*-over.txt`).

**Suites.** The AI suite with its ignored tests passes 364 on base and on the batch (`ai-suite-base.txt`, `ai-suite-after.txt`). The sim gate with its ignored tests passes 17 on both (`sim-gate-base.txt`, `sim-gate-after.txt`), with no golden regenerated.

Seams left:
- The fixture's weapon has the omni-fire flag (S-0310 claim 2). Without it, the deploy-fire drone firing at its own cell (S-0517 claim 2) answers *not facing* for every processing and never fires. The direction from its reference point to its own cell's centre is the degenerate one. This is S-0310's and S-0204's question, not claim 20's, and it is not traced here.
- Claim 19's `measurable` bullet, objects under the bridge being hit, is checked by both gas tests: the crusher on the ground in the cloud's cell is hit. Three `unmeasured` bullets have no test here: a cloud over water under the bridge, a successor made at the deck height, and objects on the deck while the cloud is still level with it. The deck test asserts no deck hit in any update, but does not make sure a hit falls due before the first odd-tick move.

The P3 census rule: the batch changes no simulation code, so it makes or moves no back-reference of `World::census`.

## Batch S29: the other overlay writers, nested spawners and the seams left open

Base main c481a2b, not rebased. The batch takes the seams earlier batches handed back, in the brief's order: A6c's rest of #284, S18's #344 and #346, S27's weak test and F4's subtraction in the fire point. It hands back #263, #252, #291 and #339 untouched. The design text came first (d40b8a0), in *Simulation (sim issues S29 …)*. Each test was committed before its fix. Every red was shown in a separate base worktree checked out at the test commit (`red-<commit>.txt` in `local-lane/sim-issues/s29/`).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #284 (a) | `overlays::end_marker`'s pieces, `vein_placement` (the placed cell and its edge neighbours) and `rebuild_veins` wrote a cell's overlay without ending its crate entries. A map's own crate or a placed crate kept its place or its map-crate entry after the overlay changed | fixed (026f266; test 1ed7be1, 4 red) | S-0520 r17 claims 11 and 29: each writer calls `crates::overlay_changed` before the write | `overlays.rs` tests on the shared `crates::fixture` (new in `crates.rs`) |
| #284 (b) | `bridges::low::write_strip`, `bridges::high::lose` and `bridges::hut::place_section` did the same | fixed (5889730; test d8bae0f, 3 red) | As (a) | tests in `bridges/low.rs`, `bridges/high.rs` and `bridges/hut.rs` |
| #284 (c) | `econ::resource::empty_cell` (through `take_all`) and `econ::sell::wall_sale` removed a crate-flagged resource or wall type, which S-0520's Edge cases allow as a map's own crate, and left its entry | fixed (dfe9656; test c206098, 2 red) | As (a) | tests in `econ/resource.rs` and `econ/sell.rs` |
| #284 (rest) | The other writers: `end_marker`'s end row, `econ::placement::put_wall`, `econ::resource::new_resource_cell`, the data write of `crates::put` and `load`'s grid copy. `crush::remove_overlay` and `overlays::place_overlay` already forget | already right | Each writes only to a cell with no overlay, keeps the overlay type, or runs before any crate state exists (design list) | — |
| #344 | S18 capped each object's slot and slave lists, but a slot object that is itself a spawner fills its own slots, so the counts multiply: 100 slots of a type with 100 slots made 10101 objects | fixed (1cbc320; test 72dff49, 2 red; lint 96d3dfa, clippy 8f4b1e2) | `design: #344`: `objects::Chain` counts the creation set's members made so far, the root included, and a creation set holds at most `objects::MAX_CREATION_SET` (1000). A full set gives a spawner only the slots it filled and a slave master only the slaves it made. The importer warns for every type whose full creation set is above the limit (`objects::creation_set_limits`). Regeneration starts a set of its own | `creation_count_limit.rs` (`issue_344_nested_spawners_do_not_multiply`, `issue_344_slave_masters_in_slots_do_not_multiply`) |
| #346 | A ground count at (1, 0) for a vehicle on the bridge deck | not reproduced | On base c481a2b neither a deck move nor an enter from the deck sets a ground flag. The layer-aware `claim_track` (bdda544, #181) landed after the audit commit and is the likely fix (not bisected). Guards only (60d0b31) | `absorb_from_deck.rs` (`issue_346_a_vehicle_on_the_deck_sets_no_ground_flag`, `issue_346_a_vehicle_entering_from_the_deck_sets_no_ground_flag`) |
| S27's weak test | `an_unread_component_keeps_the_free_mark_early` passed on base by the draw at seed 1 | test-only (5e5d7ae) | The test runs seeds 1 to 5. A control with house 1 starting at 0 must take the same draws. With the rule removed (check commit 443c79b, never merged) it fails at seed 1 (7 raw values against 6) | `claims_s0009_r45_header_marks.rs` |
| F4's seam | `combat::position::claim1_point` formed S-0204 F3's X and Y with a plain subtraction, which panics in a debug build when the difference leaves i32 | fixed (714de0a; test 740c7db, 1 red) | S-0205 claim 1: an integer result outside the range wraps, and neither S-0313 nor S-0204 states a clamp. Both differences use `wrapping_sub`, as a release build already did. The test was narrowed after the fix (c860486) so that it asserts only a wrapped difference inside F3's decided range. It still fails with the fix undone (check commit a197a72, never merged) | `position.rs::aim_difference_tests` |
| #263 | The moving-blocker variant at T5 and the switch-off turn | not taken | Handed back untouched. The train refresh rule (S-0212 claim 20's last sentence), listed as buildable, is **still not built** | — |
| #252, #291, #339 | The infantry crash end; S20's #291; `linked::recruit` perf | not taken | Handed back untouched. #339 was not measured | — |

Of the new tests, the two #346 guards and S27's control pass on base. All the others fail on the base code, 12 in all: 9 for #284, 2 for #344 and 1 for F4. S27's test fails only with its rule removed.

Formats: no fix adds saved or hashed state. `Chain` lives only while a creation set is being made. So save 152, AI game 129 and wire 12 are unchanged. No switch is added. No golden moves.

Lanes compare base c481a2b with the batch at 8f4b1e2. Logs are in `local-lane/sim-issues/s29/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). Every output file is byte-identical to base, with these rows failing:
- M-0001: 10 of 96;
- M-0002: 4 of 24;
- M-0003: 3 of 35;
- M-0019: 0 of 6;
- M-0020: 0 of 25;
- M-0021: 10 of 50.

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`). M6c, M7a and M7b over seeds 1 to 16 pass. They print the same lines as base apart from timings.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games). All 36 games end on the same tick with the same winner as base.

**Suites.** The AI suite with its ignored tests passes 364 before and after. The sim gate with its ignored tests passes 17 before and after. On the final commit these also pass:
- `cargo test --workspace --no-fail-fast`;
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo fmt --check`;
- the wasm32 build of glasnost-app;
- `cargo xtask check`;
- `cargo xtask gate`.

Seams left:
- #263: the train refresh rule of S-0212 claim 20 is buildable but not built, and its T5 and switch-off tests are not written.
- #252, #291 and #339 are not taken.
- #346 was not bisected to bdda544.
- What S-0204 F3 gives for |X| or |Y| at or above 2²⁴ stays S-0204's open question. The wrapped difference is only its input.
- A refilled slot or regenerated slave starts a creation set of its own, so each refill has its own limit of 1000, not the root's.

The P3 census rule: crate places and map crates are not object back-references, so the overlay writers forget nothing in the census. The creation-set limit makes fewer slot objects and slaves. Each one made still goes through `spawn::created_in` and `census::forget` as before. No fix makes or moves a back-reference of `World::census`.

## Batch S30: S-0009 r47, current places and the answers to Q-3213 to Q-3215

Base main bcc1987, not rebased. The batch builds S-0009 r47's changes to claim 29 and the claims it leans on, and the lead's answers to S27's requests Q-3213, Q-3214 and Q-3215. The design comes first, in [docs/dev/simulation.md](../dev/simulation.md), *Simulation (sim issues S30: S-0009 r47, current places and the answers to Q-3213 to Q-3215)* (917037b). 3c2ba99 marks S27's interim rules there as answered. Each fix's tests are committed before it (8be71a3 and e26a11d, 67409fb, ee0b2a3). To show them failing, the base worktree was checked out at each test commit and the tests run there: `local-lane/sim-issues/s30/red-cleanup-e26a11d.txt`, `red-q3215-67409fb.txt` and `red-q3214-ee0b2a3.txt`. No fix commit changes a test. e26a11d sets the new switch off in S27's `a_removed_house_frees_its_position`, whose rule is now the switch's off position; its assertions are unchanged. 1b9a8ea is rustfmt on `Quirks::named`. The tests use synthetic maps, with M13b's four start waypoints. No request is filed: r47 left nothing ambiguous that this batch needed.

| Item | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| 1. `quirks.cleanup_shifts_start_marks` | The switch was missing, so `every_spec_quirk_has_a_switch` failed | fixed (4613bf3; behaviour d103492) | S-0009 claim 29 (*After the cleanup*) and Edge cases: new field, default on, in `Quirks::named` and the committed schema. On, the cleanup changes no mark and marks are read by current places; off, current places never change and the cleanup frees every mark equal to a removed house's number | `rules` tests (`every_spec_quirk_has_a_switch`, the schema test); `claims_s0009_r47_cleanup_marks.rs`, both positions |
| Q-3213 (a house the cleanup removes) | S27's interim rule (free the removed house's marks, house numbers never change) is r47's off position, not its default. Under the default the engine differed | fixed (d103492) | New `starts::current_place`, `house_at_place`, `after_cleanup`, `take_held`; `highest_held` takes a place, `assign` writes the current place, `player_at` reads by current place. `load::new_game` calls `after_cleanup` after the cleanup and `take_held` in both phases. The current place is derived from `House::removed`, so no state is added | `claims_s0009_r47_cleanup_marks.rs` (`a_kept_special_house_holds_the_removed_neutral_houses_position`, `switch_off_a_kept_special_house_keeps_its_own_position`, `a_kept_special_house_with_no_mark_of_its_place_is_assigned_anew`, `a_late_pair_giving_the_removed_houses_number_names_the_next_house`, `a_shifted_mark_may_name_a_passive_house`, `a_removed_last_houses_position_stays_taken`, `marks_of_removed_houses_name_no_house`); 5 of 7 fail at e26a11d |
| Q-3214 (`quirks.double_start_resolution` off) | S27 resolved once after the cleanup and applied the pairs there. r47 puts the one resolution before the cleanup, then *After the cleanup*, then an assignment against the same list | fixed (c685318) | `load::new_game`: with the switch off, the one list is built after the second recalculation pass and the resolution (pairs from −1, choosers over them, holders, `assign`) runs before `cleanup`; then `after_cleanup`, `take_held` for the kept non-passive houses and `assign` against the same list. The pairs are not applied again | `claims_s0009_r47_one_resolution.rs` (`the_one_resolution_comes_before_the_cleanup`, `after_the_one_resolution_the_kept_special_house_holds_the_neutral_houses_position`, `a_pair_applied_at_the_one_resolution_is_read_after_the_cleanup`, `a_house_left_with_no_position_after_the_cleanup_is_assigned_then`, `an_unread_component_keeps_the_free_mark_in_the_one_resolution`); 3 of 5 fail at ee0b2a3 |
| Q-3215 (marks at positions N and above) | S27 ignored such a mark and the References rule read −1. r47: the References rule reads it for A … H whatever N is; no house holds a position by it | fixed (5126a07) | `starts::apply_pairs` keeps every mark below the marks' length; `highest_held` and `assign` still look only below N, with `load`'s note | `claims_s0009_r45_header_marks_references.rs` (`references_at_positions_n_and_above_read_the_marks`, which replaces `references_at_positions_n_and_above_are_empty`, and `variant_h_a_reference_at_n_or_above_names_a_passive_house`); 2 of 9 fail at 67409fb |
| 3. Terms, *current place* | New term | built (d103492) | `starts::current_place` | as Q-3213 |
| 3. Claim 21, after a cleanup removal | The References rule replaces claim 21 on any map once the cleanup has removed a house | built (d103492) | `starts::references_read_marks`; `World::start_marks` is set from the late phase when N_h ≥ 9 or a house was removed | `a_kept_special_house_holds_the_removed_neutral_houses_position` (no header pairs), `a_shifted_mark_may_name_a_passive_house` |
| 3. Claim 28, a kept special house | Its late-phase position follows claim 29's *After the cleanup* | built (d103492) | as Q-3213 | the cleanup-marks tests |
| 3. Claim 29, late phase | A house with no mark of its current place is assigned anew, every mark other than −1 taken, and its position carries its current place | built (d103492) | `assign` | `a_kept_special_house_with_no_mark_of_its_place_is_assigned_anew` |
| 3. Claim 29, first unspecified case | The house holds only marked positions below N; the References rule still reads the mark | built (5126a07) | as Q-3215 | as Q-3215, and S27's `marks_at_positions_n_and_above_are_ignored` |
| 3. M13b variant F2 | New measurement variant: pair 9 = (5, 5) over chosen 0 and 1 | guard (e247929): the engine already does it | none | `claims_s0009_r47_variant_f2.rs` (`variant_f2_players_whose_positions_a_pair_overwrote_are_assigned_beside_them`, seeds 1–5) |
| 3. M13b variant H | New measurement variant: a reference at N or above naming a passive house | fixed with Q-3215 | as Q-3215 | `variant_h_a_reference_at_n_or_above_names_a_passive_house` |
| 3. Loading pipeline table | The cleanup row keeps the marks and lowers the current places; the late row adds the no-mark case | built (d103492) | as Q-3213 | as Q-3213 |
| 3. Open question removed | r47 drops the open question that Q-3213 answers | nothing to build | none | none |
| 3. Changes for other specs (S-0011 claim 23, `quirks.entered_by_country_only`) | Text that awaits S-0011's revision | nothing to build | `mp_triggers::resolve_house` (actions, 4475 … 4482) and the entered-by event with the switch off both go through `starts::player_at`, so they follow current places already | the cleanup-marks tests read through `resolve_house` |

Guard and control tests that pass on base: `marks_of_removed_houses_name_no_house` (both positions agree there) and `switch_off_a_kept_special_house_keeps_its_own_position` (the off position is S27's interim rule); `a_house_left_with_no_position_after_the_cleanup_is_assigned_then` and `an_unread_component_keeps_the_free_mark_in_the_one_resolution` (the old order gives the same positions); the F2 guard.

Formats: the save format goes from 152 to 173 and the AI game format from 129 to 149 on the branch (5ad3694), renumbered to 153 and 130 at the merge, since a replay of a new game can resolve differently: marks by current places, `start_marks` on any map after a removal, marks at N and above, and the one resolution's order. No state is added. The AI crate's only edit is that constant. Wire stays 12. The sim-gate goldens do not move: the sim gate with its ignored tests passes 17 before and after without regeneration (`sim-gate-base.txt`, `sim-gate-after.txt`).

Lanes compare base bcc1987 with the batch at 1b9a8ea. Logs are in `local-lane/sim-issues/s30/`. No lane or preset turns `quirks.double_start_resolution` off (it is set only in tests), and no lane row moves, so no lane game had a cleanup removal or a header pair that reaches the changed code.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). `diff -r arena-base arena-after` is empty (`arena-diff.txt`): M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing.

**Sweeps** (`m6c-base.txt`/`m6c-after.txt`, and likewise for m7a and m7b; the `*-rows.txt` files drop the build and timing lines). M6c, M7a and M7b over seeds 1–16 pass and print the same rows as base: smallest minimum 0 for M6c and 2 for M7a, smallest drain minimum 3 for M7b.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). All 36 games end on the same tick with the same winner as base, so no winner needs tracing.

**Suites.** The AI suite with its ignored tests passes 364 on base and on the batch (`ai-suite-base.txt`, `ai-suite-after.txt`). The workspace tests, clippy with `-D warnings`, `cargo fmt --check`, the wasm32 build of glasnost-app, `cargo xtask check` and `cargo xtask gate` pass on the final tree (`final-*.txt`, `xtask-check.txt`).

Seams left:
- The one resolution can note "every start position below N is taken" for a house that the step after the cleanup then gives a position. The note is true when it is written.
- *After the cleanup* is `unmeasured` (claim 29), so no lane checks it.

The P3 census rule: start position marks are not back-references, so no fix calls a census forget.

## Batch S31: a destination that goes inside (S-0507 r11, Q-3207)

Base main f24a642, not rebased. The batch builds S-0507 revision 11's claim 6a, which answers S26's Q-3207: an object that goes inside, by boarding a transport of either kind or by going into a building that takes it off the map, is lost at once as the destination of every vehicle and infantry holder, the transport it boards included, and the holder's locomotion keeps the point it last received. The design is the section *Simulation (sim issues S31: a destination that goes inside, S-0507 r11)* of `docs/dev/simulation.md` (9225cf6). Requests Q-3235 to Q-3237 are filed (4141a7b), and their interims are built. The failing tests are 4df3c10: all six fail on their own tree, checked out in the base worktree (`local-lane/sim-issues/s31/t-r11-at-4df3c10.txt`). The fix is f86cb98, rustfmt c0577e2, the goldens b5d62cc and the two guards d4d0052. The absorber test is 5ecfaf0; it was committed after the fix that it checks. r11 names no quirk switch, so none is added.

| Claim part | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| claim 6a, T16 (a) | A hovering transport that holds an infantry unit by claim 5's reply on water kept it as its destination after the unit boarded another transport, so its *D* followed the unit inside (S26's trace) | fixed (f86cb98) | `objects::go_inside` (new, with `Leaving::Inside`) for `transport::board`. `held::holders_drop` makes the loss in its existing pass over all objects (`transport::destination_went_inside`): the destination is cleared without the destination assignment, so the locomotion is not told | `claims_s0507_r11.rs` (`t16a_the_unit_boards_another_transport`) |
| claim 6a, T16 (b) | The same when the unit goes into a building as an occupant (S-0508) | fixed (f86cb98) | `structures::garrison::go_inside` calls `objects::go_inside` | `t16b_the_unit_goes_into_a_building` |
| claim 6a, the transport it boards | The transport the unit boards, holding it by claim 5, kept it as its destination with the unit inside | fixed (f86cb98) | The same loss, at `transport::board` | `the_transport_it_boards_loses_it` |
| claim 6a, other holders | Any vehicle or infantry holder, transport or not: driving, walking and hovering holders lose the object in the same tick. The driving locomotor keeps its goal (S-0207 claims 25 and 26 as r11's *Changes for other specs* amend claim 26) | fixed (f86cb98) | `destination_went_inside` clears `DriveState::for_dest` with the destination, so the next locomotion update does not take it again, and leaves `goal` | `other_holders_lose_it_and_a_driver_keeps_its_point` |
| claim 6a, queued and approach destinations | The object leaves `queued_dests`, which are also the approach destinations (S-0215 Terms), wherever it stands. The other entries keep their order, and a destination that is another target stays | fixed (f86cb98) | `destination_went_inside` | `it_leaves_queued_destinations` |
| claim 6a, a revert in the same moment | The revert of S-0309 claims 27 and 27a comes first. A given-back destination that is the object is then lost by claim 6a's loss, not by claim 27a's `assign_destination(None)`, so the locomotion keeps the object's movement point at the loss | fixed (f86cb98) | For `Leaving::Inside` the claim 27a loop is not run; the pass's loss clears it | `a_revert_in_the_same_moment` |
| claim 6a, the hovering point | The hover read *D* from the destination object's live position, so with no destination it had no point (claim 16 keeps the retained point). At the loss *D* is now the point recorded when the destination was taken | interim Q-3235 (f86cb98) | `HoverState::taken_point` (new, saved and hashed), recorded by `hover::take`. `hover::destination_went_inside` sets `retained` to the held point or the recorded one (none when the hover had stopped for it) | T16 (a) and (b) check that the point is unchanged and that A moves on toward it |
| claim 6a, walking, jumpjet and teleporting holders | The walker has no stored locomotor point, and jumpjet and teleport fix theirs when taken. They are not told | interim Q-3236 | none | `other_holders_lose_it_and_a_driver_keeps_its_point` (walker) |
| claim 6a, which ways out are going inside | Boarding (`transport::board`), the garrison (`garrison::go_inside`) and the absorber (`absorb::go_in`) are going inside. Slave miners, the parasite, the spawner and the lift's pick-up keep `Leaving::OffMap` | interim Q-3237 | `absorb::go_in` calls `objects::go_inside` | T16 (b); `going_into_an_absorber_is_going_inside` (5ecfaf0, committed after its fix) |
| claim 6a, aircraft holders | Aircraft follow S-0501 claim 53, already built in `air::destination_lost`. It runs for `Leaving::Inside` as it did for `OffMap`, and `destination_went_inside` returns for every kind but vehicles and infantry | guard (d4d0052) | none | `an_aircraft_holder_follows_s0501_claim_53` |
| claim 6a, the tank bunker | A vehicle that enters a tank bunker stays on the map (S-0606 claim 7). The bunker's entry calls no leaving-play hook, so its holders keep it | guard (d4d0052) | none | `a_tank_bunker_is_not_going_inside` |
| r11 *Changes for other specs* (S-0207 claim 26) | A spec text change: the driving locomotor's point stays at claim 6a's loss | built with claim 6a | as above | as above |
| T16 local lane | A recorder run against the original | not built | T16 is a recorded scenario with no measurement record yet, so there is nothing to compare | — |

Formats: `HoverState::taken_point` is saved and hashed, so save goes from 152 to 175 and AI game from 129 to 151, the numbers the brief gives; renumbered to 154 and 131 at the merge. The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change. Wire stays 12.

Goldens (b5d62cc): `duel-m6c.txt` moves from line 1137 (tick 11359) to its end, and `duel-naval.txt` from line 1450 (tick 14489), both from f86cb98. In `duel-naval.txt` the hash returns to its old value between hunks, through line 1632 (lines 1569, 1571, 1572 and others). So the first moves are the recorded `taken_point` in the hashed state, not a change of play.

Whether play changes later in either duel is not shown:
- duel-naval's last hunk, from line 1633 to its end, never returns to the old value;
- duel-m6c never returns from line 1137 to its end. The M6c sweep's seed 3 lines are identical, which suggests but does not prove that its play is unchanged.

b5d62cc's message says that the naval moves are not a change of play; that holds only through line 1632. Both files' line counts are unchanged, and the other goldens do not move.

Lanes compare base f24a642 with the batch at b5d62cc. Logs are in `local-lane/sim-issues/s31/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). `diff -rq` of `arena-base/` and `arena-after/` is empty. M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing, the same as at base.

**Sweeps** (`m6c-base.txt`/`m6c-after.txt`, and likewise for m7a and m7b). M6c, M7a and M7b over seeds 1–16 pass. Apart from build lines their seed and summary lines are identical to base: smallest minimum 0, 2, and drain 3.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). 34 games end on the same tick with the same winner as base. Two games move, both on map 0 with sides 1 and 2, and keep their winner:
- seed 7, game 3, S25's game: from 47257 to 46376 ticks, house 1 still wins. It does not go back to 46922, its length before S25. It makes 11 loads onto a hovering transport, against 10 at base.
- seed 9, game 3: from 43203 to 43485 ticks, house 0 still wins. It makes 7 loads onto a hovering transport, against 5 at base.

Both use hovering transports, which fits claim 6a's path, but neither is traced. No winner changes, so no trace is needed.

**Suites.** The AI suite with its ignored tests passes 364 on base and on the batch (`ai-base.txt`, `ai-after.txt`). The sim gate with its ignored tests passes 17 on both (`gate-base.txt`, `gate-after.txt`), after the goldens were regenerated.

Seams left:
- Retry delay: the loss skips `path::destination_changed`, as claim 6a's "the locomotion is not told" reads, so a running retry delay (S-0212 claim 21) runs on. With no destination, driving and walking make no search, and the hovering locomotor is not in claim 21's list for a changed destination, so this makes no difference.
- Q-3235: before the loss our hover's *D* for an object destination is still the object's live position. T16's "the same just before and just after" therefore holds only while the object stands still from the assignment to the loss, as it does in T16.
- Q-3238 is not used.

The P3 census rule: the loss rides on `holders_drop`'s existing pass over all objects and reads no census back-reference, so the batch makes or moves no back-reference of `World::census`.

## Batch S32: trains and the path refresh, the infantry crash end, S20's rest of #291 and the recruit scan

Base main 23ff203, not rebased. The batch takes the four seams S29 handed back, in the brief's order: #263, #252, #291 and #339. The design text came first (00dbc8d, d1b116b, 7c6d5e3, 272e070 and ec2a78d), in *Simulation (sim issues S32 …)*. Each test was committed before its fix, and no fix commit changes a test. Every red was shown in a separate base worktree checked out at the test commit (`red-<commit>.txt` in `local-lane/sim-issues/s32/`). Requests Q-3239 (757c0f4) and Q-3240 (0417705).

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #263 (tests) | The switch-off turn and T5's moving-blocker difference at a path refresh had no test of their own | test-only (224a9f7) | No code change: both pass on base | `claims_s0212_refresh.rs` (`issue_263_off_turns_to_the_new_first_step`, `issue_263_moving_blocker_at_the_curve_takes_the_new_first_step`), each in both positions of `quirks.path_refresh_keeps_old_step` |
| #263 (train) | S-0212 claim 20's last sentence: a train's recomputation keeps the step in progress, searches from where it leads and stores the result after it, up to 24 steps. The code treated a train like any other unit | fixed (2937b22, rustfmt 79f9d41; test 2e4b319, 1 red) | `path::compute_from` starts a search at a given cell. `drive::step_check_with` uses it for a train type and stores the step followed by the new path, cut to `path::STORED`. The switch does not apply to a train | `claims_s0212_train_refresh.rs` (`issue_263_a_train_keeps_the_step_in_progress`; the car control at P9's walls and the 24-step bound pass on base) |
| #252 | S-0312 claim 16, *Infantry crash end*: a crashed jumpjet infantry unit left play at the impact, setting off its death weapon a second time | fixed (fc1b9cd; test 0cc4528, 3 red) | `jump::fly::impact` makes the forced request for *air-death-end* for a dead infantry unit and sets off no death weapon. `tick::object_turn` stops the jumpjet update once it is *crashed*, and `combat::death::infantry_death_turn` removes it with no corpse when *air-death-end* completes (S-0107 State and ordering). A turned-down request leaves play at the crash end (`design: Q-3239 interim`) | `claims_s0312_crash_infantry.rs` (3 tests, both positions of `quirks.crash_death_weapon_twice`) |
| #291 (a) | A bouncing conversion animation was made a fixed infantry maker: it neither flew nor landed (S-0611 claims 8, 9 and 15) | fixed (1b7f9df; test 4417ebc, 2 red) | `mutation::start` makes a bouncing type an ordinary animation (new `combat::effects::anim_made`) that holds its infantry-making state with claim 8's owner. `mutation::mark_now` marks its point at once for a crashed victim | `claims_s0611_bouncing_conversion.rs` (2 tests) |
| #291 (b) | The mech locomotor maps to no layer rule | not reproduced | S-0102's layer table puts mech movement in the ground layer. The importer gives it `Locomotor::None`, and `app::mapview::layers` ends in that ground row | — |
| #291 (c) | No path detaches an attached animation while it plays | request (Q-3240) | S-0109 claim 22 rates detachment `unmeasured`, and S-0614 knows no detaching event. Claim 15's rules after a detachment are not built | — |
| #339 | Whether `linked::recruit`'s two update-order scans cost enough to fix | measured, not changed (probe 3b972b7, clippy d357108) | Release probe, synthetic data, fillers ahead of both prism buildings: at 1000 fillers a recruitment 49 µs, the two scans 1.3 µs, a tick 5.6 ms; at 3000, 185 µs, 2.3 µs and 17.6 ms (`perf-339.txt`). The scans are 0.01 to 0.02 percent of a tick. The rest of a recruitment is the older eligibility pass over every object | `perf_linked_recruit.rs` (ignored; prints figures only) |

Of the new tests, the two #263 guards and two of the three train-refresh tests pass on base. The other six fail on the base code: 1 for #263, 3 for #252 and 2 for #291.

Formats (ee90863): no saved field changes, but replays change, so save 177 and AI game 153 on the branch, renumbered to 155 and 132 at the merge; wire 12 is unchanged. No switch is added. The sim-gate goldens were regenerated once, with the ignored tests, and none moved.

Lanes compare base 23ff203 with the batch. Logs are in `local-lane/sim-issues/s32/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). Every output file is byte-identical to base, with these rows failing:

## Batch S33: a tank under a jumpjet's fire, and dummy mode (#434, #439)

Base main e249203, not rebased. The batch traces #434 and builds #439. The design text came first (5e9ee99), in *Simulation (sim issues S33 …)* in `docs/dev/simulation.md`. #434 changes no code: the trace matches the specs, so its tests are guards. #439 is new behaviour behind a setup flag that is off by default, so it has no red tests. Its proof is that every lane, golden and state hash stays the same with the flag off. Logs are in `local-lane/sim-issues/s33/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #434 | A player house's tank in *guard*, hit by a jumpjet infantry unit hovering at height 500, traced with a ground-only tank and an anti-air tank at 4 and 7 cells, with player-scatter on and off (`trace-434c.txt`). The ground-only tank's firing check is *illegal* (S-0306 claim 23), so retaliation answers *may not* at any distance (S-0309 claim 18). With player-scatter off, claim 19 case 2 does not apply to a house that counts as human-controlled, and the tank stands idle, as in the issue. With the setting on, it scatters. The anti-air tank takes slot 1 (S-0308 Table P row 19) and retaliates within F7, and is *held* beyond it. Every case matches S-0215 claims 18 and 19 and F7 | not a sim fault; guard (6f8a6ff) | None in the sim. The issue was retitled to an own-AI micro (*get out of reach or fall back to anti-air cover*) in `crates/deterministic/ai`. That is not built here and is handed back, to be shared with #443. No request is filed | `retaliate_airborne_jumpjet.rs` (`issue_434_a_ground_only_tank_stands_under_a_hovering_jumpjet`, `issue_434_a_ground_only_tank_scatters_with_the_player_scatter_setting`, `issue_434_an_anti_air_tank_retaliates_within_f7`, `issue_434_an_anti_air_tank_beyond_f7_is_held`) |
| #439 | A skirmish dummy mode: the AI leaves the person alone, and the person sees the whole map | built (04c3335 sim, 3d85ae1 ai, 87549c4 ui, app and platforms; rustfmt 84d7fe1) | `design: #439`. `GameSetup::dummy`, off by default (`serde(default)`), marks every person-played house in `HouseSetupState` (bit 1 of the person-played byte, through `HouseSetupBytes`; a byte above 3 is refused). Sim gates go through `World::dummy_spares` and `World::dummy_house`. `acquire::candidate` rejects a marked house's objects, `retaliate` answers *may not*, `anger_of` adds no anger, and `drive::attack_obstacle` leaves them alone. `shroud::explored` and `seen` are true for a marked house, and `pending` is false. The AI planner leaves the marked house out of its scans, starts, captures, drains, lifts, domination and strikes. The person may attack, AI houses still fight each other, and cloak and disguise stay hidden. The skirmish setup has a *Dummy mode* box, and native has `--dummy`. Network rooms never offer it, and `client_net` forces it off | sim `dummy_mode.rs` (7 tests); ai `dummy_mode.rs` (`the_ai_leaves_the_person_alone_early`, ignored `the_ai_leaves_the_person_alone_all_game`); `ui::setup` `the_dummy_box_sets_the_game_setup`; app `the_dummy_box_and_flag_set_the_game_setup` |

Formats: the mark is saved and hashed world state, so save goes from 152 to 179 and AI game from 129 to 155 on the branch, renumbered to 156 and 133 at the merge. Wire stays 12, because no message changes. `save::FORMAT_VERSION` covers replays, which gain `GameSetup`'s one byte. `net::setup_hash` covers `GameSetup`'s bytes, so a network game's setup hash differs from base's, though its value is the same for every flag-off game on HEAD. No switch is added, and no golden moves.

Lanes compare base e249203 with the batch at 84d7fe1.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). `diff -r` of the two output directories is empty. These rows fail, on base and after:
- M-0001: 10 of 96;
- M-0002: 4 of 24;
- M-0003: 3 of 35;
- M-0019: 0 of 6;
- M-0020: 0 of 25;
- M-0021: 10 of 50.

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`). M6c, M7a and M7b over seeds 1 to 16 pass. They print the same lines as base apart from timings.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-<seed>.txt`, `quick-after-<seed>.txt`). 23 games end on another tick, and 5 change winner. The 12 games of sides 1 and 2 are unchanged in every seed. Each flip was played alone (`GLASNOST_AI_SIDES`), hashed every 10 ticks on both binaries, then hashed and dumped every tick over the first differing window (`GLASNOST_AI_HASH`, `GLASNOST_AI_OBJS`; `trace-*`, `objs-*`). In all five, the first difference is a dead house 0 jumpjet infantry unit (type 4) that is *crashed* and has just started *air-death-end*, where base removed it at its crash end: #252.

| Flip (seed/map/sides) | Base → batch | First differing tick | Object |
|---|---|---|---|
| 7/2/0,2 | h1@49540 → h0@47071 | 8699 | 923 (*air-death-end* from 8698) |
| 7/8/0,1 | h0@69495 → h1@50850 | 11456 | 316 (from 11455) |
| 8/8/0,1 | h0@39300 → h1@38735 | 8546 | 268 (from 8545) |
| 9/8/0,2 | h1@74400 → h0@24057 | 12900 | 948 (from 12899) |
| 10/2/0,2 | h0@38678 → h1@71431 | 20947 | 921 (from 20946) |

In the same tick a few other objects differ in their turn bookkeeping or their next motion step, as the removal on base changed. None of the traces points to a fault.

**Suites.** The AI suite with its ignored tests passes 364 before and after. The sim gate with its ignored tests passes 17 before and after. On the final commit these also pass:
- `cargo test --workspace --no-fail-fast`: 9242 pass; the 1 failure is main's known `quirks::tests::every_spec_quirk_has_a_switch` (`cleanup_shifts_start_marks`);

- M-0020: 0 of 25 checked;
- M-0021: 10 of 50.

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`). M6c, M7a and M7b over seeds 1 to 16 pass, with smallest minimums 0, 2 and drain 3. Apart from timings, they print the same lines as base (`*-nt.txt`).

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). Apart from timings, every line of every seed's output equals base (`quick-*-nt.txt`), so all 36 games end on the same tick with the same winner and the same counts.

**Suites.** The AI suite with its ignored tests passes 364 on base and 366 after: the 364, plus the two new dummy-mode tests. The sim gate with its ignored tests passes 17 on both. `debug_prefix_matches_golden` and `full_run_matches_golden` pass with no golden regenerated, so the flag-off state hashes equal base's. On the final commit these also pass:
- `cargo test --workspace --no-fail-fast`, apart from base's known `quirks::tests::every_spec_quirk_has_a_switch` (S30 adds the switch);
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo fmt --check`;
- the wasm32 build of glasnost-app;
- `cargo xtask check`;
- `cargo xtask gate`.

Seams left:
- Q-3239: what removes a crashed infantry unit whose *air-death-end* request is turned down, and the ground height in a high-bridge cell. Until it is answered, such a unit leaves play at its crash end.
- Q-3240: the events that detach an attached animation. Claim 15's rules after a detachment are not built.
- #339's recruitment cost is mostly the eligibility pass over every object. It was measured and left as it is.

The P3 census rule: a crashed infantry unit now stays in play, dead, until *air-death-end* completes, and then leaves through `combat::death::infantry_death_turn` and the ordinary removal, as a die sequence does. A bouncing conversion animation lives in `w.anims` like any ordinary animation. No fix makes or moves a back-reference of `World::census`.

- #434's own-AI micro (retreat or fall back to anti-air cover) is handed back. It is AI design, not a spec question.
- Dummy mode does not filter walls on an obstacle path, crushing, area damage that reaches the person's objects without targeting them, or S-0519's strike redirect to an object at the strike cell. AI orders are not hard-filtered at `orders::issue`.
- The optional battlefield marker for dummy mode was not built.

The P3 census rule: dummy mode adds a mark to house setup state and reads it in the gates. It makes or moves no back-reference of `World::census`.

## Batch S34: the nuclear warning marker, refinery slots, a balloon-hover crash and the designator line (#441, #444, #446, #448)

Base main 96ca8d8, not rebased. #441 is fixed. #444, #446 and #448 were traced first: in each the simulation does what the claims say, and the observation needs data or behaviour the specs do not give, so each has a request (Q-3254 to Q-3256) and nothing in the engine changes. Each trace is a committed claim test on synthetic rules. An opt-in local lane (`crates/tools/tests/s34_lane.rs`, ignored, never in CI) checks what an install's data give each path, printing positions, slot kinds and booleans only.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #441 (S-0513 claims 10 and 11, S-0109 claim 22) | The warning marker was a free animation with no owner, so a marker type that loops never ended | fixed (353b582) | `anims::WarningMarker { missile, house }` on `Anim::warning` (saved and hashed). A one-time launch ties it to the falling missile and a silo launch to the upward missile. At its first update after that missile has left `World::projectiles` it is deleted and does nothing else. It stays at the target cell's centre (interim Q-3253, 166bcb6) | `claims_s0513_nuke.rs`: `claim11_a_one_time_launch_marker_ends_when_its_falling_missile_leaves_play`, `claim11_a_silo_launch_marker_ends_at_the_upward_missiles_burst` (4bc30f9; both fail on their own tree, `local-lane/sim-issues/s34/t1-at-4bc30f9.txt`) |
| #441, the viewer test | Claim 11's visibility (not allied with the launcher, an operating psychic sensor within 256 × its radius, re-tested every update) is presentation. The simulation gives the client the marker's missile and launching house | handed back (client) | none in the simulation | — |
| #444 (S-0402 claim 11, S-0108) | The simulation starts the docking animation with the start row and the payment animation at each payment. On the empty-load row it starts the unloading animation and stops the payment animation. The *finishing* harvester waits for the unloading animation's end | not reproduced in the simulation; request Q-3254 (d6b26fa) | none | `claims_s0402_dock_anims.rs`: `claim_11_issue_444_the_slots_of_a_whole_cycle_from_contact` (0153a5d) |
| #446 (S-0502 claim 18, S-0113 claim 16, S-0312 claim 16) | A crashable balloon-hover jumpjet vehicle takes the vehicle crash path from its first update. Its heading spins at its rate and its body facing follows the heading each step used. It steps along the heading only while its current speed is above 0, so killed while moving it drifts on a slow spiral, and killed while hovering it falls straight down. The client's crash rocking follows S-0113 claim 16 as written: no upper bound on the pitch | matches the claims; request Q-3255 (51c96a5) | none | `claims_s0502_crash.rs`: `claim_18_issue_446_a_balloon_hover_vehicle_crash_traced` (d428189, 45d9753) |
| #448 (S-0519 claims 16 and 26, S-0518 claims 18 to 20 and 23, S-0514 claim 23) | An ordered infantry designator fires its S-0308 row 6 weapon. The shot reports its line, ending at the building's target point, and the strike is called. The laser flag and duration reach the simulation. By S-0518 claim 18 a beam is drawn only for a weapon with the laser flag. The claim 26 tint reads `strike::marked`, and the building is marked | matches the claims; request Q-3256 (6866de8) | none (no `beams.rs` fault) | `claims_s0519_strike.rs`: `issue_448_an_ordered_infantry_designation_reports_its_laser_line` (ab0c6bc) |

Formats: `Anim::warning` is saved and hashed, so save goes from 154 to 181 and AI game from 131 to 157 on the branch, renumbered to 157 and 134 at the merge. The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change. Wire stays 12. No built claim names a quirk switch, so none is added.

Goldens (e731157): only `duel-m7b.txt` moves, from 353b582. 15 of its 800 hashes change, in eight runs from line 567 (tick 5660) to line 768 (tick 7670). Between the runs and after the last one, the hash returns to its old value through the file's end. That fits a state that differs only while a marker is in play. Its line count is unchanged, and the other goldens do not move.

Lanes compare base 96ca8d8 with the batch at 6866de8. The goldens commit changes no code. Logs are in `local-lane/sim-issues/s34/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). `diff -rq` of `arena-base/` and `arena-after/` is empty. M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 and M-0021 10 of 50 failing, the same as at base.

**Sweeps** (`m6c-base.txt`/`m6c-after.txt`, and likewise for m7a and m7b). M6c, M7a and M7b over seeds 1–16 pass. Apart from build lines, their seed and summary lines are identical to base: smallest minimum 0, 2, and drain 3.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). 30 games end on the same tick with the same winner as base. Six games move, and every one keeps its winner:

| Seed | Game | Base ticks | Batch ticks | Winner |
|---|---|---|---|---|
| 7 | 1 | 47024 | 51114 | house 1 |
| 8 | 6 | 46665 | 50820 | house 0 |
| 8 | 9 | 31029 | 34122 | house 0 |
| 9 | 3 | 43485 | 85012 | house 0 |
| 9 | 6 | 54791 | 49090 | house 0 |
| 10 | 3 | 41664 | 42623 | house 0 |

All six launch a nuclear missile at base, which fits #441's path: a marker that no longer stays in play. Seed 7's games 6 and 7 also launch one and do not move. None is traced, because no winner changes.

**Suites.** The AI suite with its ignored tests passes 364 on base and on the batch (`ai-base.txt`, `ai-after.txt`). The sim gate with its ignored tests passes 17 on both (`gate-base.txt`, `gate-after.txt`), after the goldens were regenerated.

Seams left:
- S-0513's *Presentation fixed outright* removes a warning marker when its owner house leaves play. In our engine, a house leaves only at load (the unused-house cleanup, S-0009 claim 28), before any launch, so nothing is built for it.
- Q-3253: the marker stays at the target cell's centre while it is attached.
- The client's viewer test for the marker (#441) is not built here.

The P3 census rule: the marker reads `World::projectiles` and no census back-reference, so the batch makes or moves no back-reference of `World::census`.

## Batch S36: a hovering jumpjet vehicle and its target (#447, S-0502)

Base main ad2b14f, not rebased. The batch builds S-0502 r27, which answers Q-3252 (#447). The design text came first (19aa738), in *Simulation (sim issues S36 …)* in `docs/dev/simulation.md`. The switch came next (5543c43), then the failing tests (11f06bb), then the fix (2564382). No fix commit changes a test. The red was shown in a separate base worktree checked out at the test commit (`red-11f06bb.txt`). No request is filed. Logs are in `local-lane/sim-issues/s36/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #447 | S-0502 claim 10 (r27): a jumpjet vehicle that keeps a movement commitment takes a physics step every tick, and each step sets its body facing to the movement heading. For the reply to *not facing* without a turret (S-0206 claim 9, S-0310 claim 9), its locomotor is moving it while it is *moving*. On base, `occupancy::in_progress` reads `jump::moving_now`, which is false in *hovering*. So a committed hovering vehicle with no turret and a non-omni weapon started a body turn toward its target every tick, and the next physics step put the body back: its body never settled on the heading | fixed (2564382; switch 5543c43; test 11f06bb, 4 red; rustfmt 5eafbbd) | `jump::moves_for_facing_reply` counts the commitment (`JumpState::moving`) as moving, for `combat::behave::vehicle_reply`'s no-turret branch. The new switch `quirks.hovering_jumpjet_body_on_heading` (default on) gives S-0502's Edge cases. Off, in *hovering* a vehicle's physics steps leave its body alone (`jump::fly::physics`), and its commitment does not count as moving. Its body then turns to the target at the body turn rate and it fires within the tolerance. The heading is unchanged, and the first physics step outside *hovering* sets the body again | `claims_s0502_r27_hover_target.rs` (7 tests). Four fail on base: the committed vehicle holding its fire with the body on the heading, the switch-off turn and shot, the switch-off return to the heading, and the omni-fire vehicle's body (failing in its switch-off half). The guards for no commitment, a turret and a jumpjet-turn infantry unit pass on base |

Formats (84cbdd4): no saved field changes, but replays change, so save goes to 185 and AI game to 163 on the branch, renumbered to 158 and 136 at the merge. Wire stays 12. The switch adds one field to the rules schema (`data/schema/ruleset.schema.json`, refreshed in 5543c43). The sim-gate goldens were regenerated once, with the ignored tests (3d5d19a). Only `duel-m7b.txt` moves, from its line 522 (ticks 5220 to 5229) to its end: 278 of 800 lines. The fixture's DISC is the M7b duel's one jumpjet vehicle type, balloon-hovering with the non-omni gun as its primary. A watch of the three discs every 10 ticks on both builds (`watch-m7b-*`, `GLASNOST_WATCH=duel-m7b,11,10`, `GLASNOST_WATCH_TYPE=DISC`) shows them attacking object 17 from ticks 5220 and 5230, two with a destination cell. Every printed field (cell, behaviour, target, destination) stays equal until the tick 5770 snapshot, where DISC#5's attack starts a tick earlier (5760, against 5761). The watch does not print the body facing. The hash move falls in state the watch does not print, which fits the fix but was not observed directly.

Lanes compare base ad2b14f with the batch at 2564382. The commits after it bump the formats, regenerate the golden and run rustfmt. The AI suite and the sim gate ran again on the final commit.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). `diff -r` of the two output directories is empty. These rows fail, on base and after:
- M-0001: 10 of 96;
- M-0002: 4 of 24;
- M-0003: 3 of 35;
- M-0019: 0 of 6;
- M-0020: 0 of 25 checked;
- M-0021: 10 of 50.

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`). M6c, M7a and M7b over seeds 1 to 16 pass, before and after. M6c and M7a print the same lines as base apart from timings. M7b moves: every seed still plays its 8000 ticks with no result and no drain counter below 3, but the per-seed counters change on seeds 2, 3, 5, 6, 7, 12, 14 and 16. In the summary:
- drain orders: max 10 → 8;
- credits drained: min 1080 → 1240, median 1680 → 1720;
- drained-building ticks: min 4304 → 4276, median 4632 → 4640;
- mutator launches: median 11 → 10;
- infantry mutated: min 21 → 22, max 31 → 29;
- objects taken over: median 21 → 20, max 24 → 28;
- berserk weapon orders: min 14 → 21, median 35 → 32;
- takeover orders: median 1 → 2, max 5 → 7;
- infantry absorbed: min 2 → 5, max 10 → 8;
- bunker exit orders: max 1 → 0.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). Apart from timings, every line of every seed's output equals base, so all 36 games end on the same tick with the same winner. No winner changes, so there is no flip to trace.

**Suites.** The AI suite with its ignored tests passes 366 on base, after the fix and on the final commit (`ai-base.txt`, `ai-after.txt`, `final-ai.txt`). The sim gate with its ignored tests passes 17 on base (`gate-base.txt`). After the fix, `full_run_matches_golden` fails at duel-m7b's line 522 (`gate-after.txt`), as above. With the regenerated golden it passes 17 (`gate-after-2.txt`). On the final commit these also pass:
- `cargo test --workspace --no-fail-fast`, apart from two known failures that name only S35's `buildup_twice_without_break_off`: `quirks::tests::every_spec_quirk_has_a_switch` (S-0406, S-0524) and app `every_rendering_spec_quirk_has_a_switch` (S-0108);
- `cargo clippy --workspace --all-targets -- -D warnings`;
- `cargo fmt --check`;
- the wasm32 build of glasnost-app;
- `cargo xtask check`, apart from the 8 known stock-guard problems in M-0027;
- `cargo xtask gate`.

Seams left:
- S-0502's *Changes for other specs* (S-0206 Terms and S-0310 claim 9) are not yet revised in those specs. The fix reads them through S-0502 claim 10.
- S-0310 claim 7's balloon *moving now* and S-0206 claim 16's track-class stop are left as they were.
- The client needs no change: it draws `body.value`, which the sim now keeps on the heading.

The P3 census rule: the fix reads a vehicle's jumpjet state and changes when its body facing is set. It makes or moves no back-reference of `World::census`.

## Batch S35: the factory break-off (#440) and object destinations (S-0406 r19, S-0503 r29, S-0208 r67, S-0507 r12)

Base main 951cdbf, not rebased. The batch has two items.

**Item 1** builds S-0406 r19 with M-0027, the answer to Q-3243 (#440). Its commits:
- the switch rename bb5dc88;
- the tests moved to r19 b45b73e, and the failing tests cb5f128;
- the request Q-3261 291142e;
- the fix 2242bfd, the app drawing tests 24c6b63 and the docs a27b463;
- the formats 140c90b.

**Item 2** builds S-0503 r29, S-0208 r67 and S-0507 r12, the answers to S31's Q-3235 to Q-3237. Its commits:
- the requests Q-3262 and Q-3263 d15e2c8;
- the failing tests 3fae352: ten fail at their target assertion on their own tree, and the S-0524 lift guard passes (`local-lane/sim-issues/s35/test-i2b.txt`);
- the request Q-3264 cec9863;
- the fix d5f8728 and rustfmt f728931;
- the docs 7c6ed0f, the section *Simulation (sim issues S35: the factory break-off and object destinations)* of `docs/dev/simulation.md`.

The goldens of both items are 06c7466. The save format goes to 183 and the AI game format to 161 on the branch, renumbered to 159 and 137 at the merge; wire 12 is unchanged. Neither revision names a new quirk switch. Item 1 renames the build-up switch to `buildup_twice_without_break_off`.

| Item | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| 1: S-0406 r19 claims 15, 17 and 18, F6 and F7, M-0027 (Q-3243, #440) | The building a place order names ran its build-up twice. Under r19 the break-off requests the idle body as a deferred change at the end of its first turn T + 1. Until its first construction processing C, the building is in the factory break-off window: out of its construction state while still building up. The queued construction becomes current at the ready mark and is processed in it, C resets the range to stage 0, and the handover is at H = C + (n - 1) d_b. A stage advances only in an update, so a missed turn holds a due advance (S-0108 r111). A positive-delay range of fewer than two stages never reports (claim 18). A filling building and a restored replacement keep two runs while `buildup_twice_without_break_off` is on; a deployed building gets none. In the window, a refinery's harvester acceptance and the stop and deploy orders refuse the building only while construction is current (S-0402 claim 10, S-0213 claims 8 and 10) | fixed (2242bfd); an owner change in the window is asked in Q-3261 | `econ::placement` keeps the construction range as a stage and the update of its last change. `econ::deploy`, `econ::harvest`, `orders` and `process` follow the window. The client's `Live::buildup_of` does not yet draw the held advance (reported, not changed) | `claims_s0406.rs` (`claim_17_place_order_runs_the_build_up_once`, `claim_17_positive_delay_idle_range_sets_c`, `claim_17_a_missed_turn_holds_a_due_stage_advance`, `claim_18_deployed_single_stage_never_completes`), `claims_s0402.rs` (`claim_10_acceptance_in_the_factory_break_off_window`), `claims_s0213_deploy_construction.rs` (`deploy_order_in_the_factory_break_off_window`, `stop_order_in_the_factory_break_off_window`), `claims_s0524.rs` (`claim15_restored_replacement_gets_the_break_off_with_the_switch_off`), `claims_s0309_r126.rs`, the app's `buildup_tests` |
| 2: S-0503 claim 20 r29, S-0208 Terms and claims 6 and 16 r67, S-0507 claim 6a r12 (Q-3235 to Q-3237) | The hover's *D* for an object followed the object; it is now the movement point at the effective assignment, and an unchanged assignment that is not effective is ignored (Q-3235). A walker's arrival test read the object's position now, and after a claim 6a loss the walk ended; it now keeps the point the assignment gave and walks on to it along its stored path (Q-3236). At the loss, the jumpjet's and the teleport's tick-start take ran a clear (a reroute, a jump cancel); not "as built" (Q-3236). A slave into its master, a parasite into its host at firing and a spawned aircraft docking into its spawner are going inside; the S-0524 lift is not, and stays as built with a guard test (Q-3237) | fixed (d5f8728); interims of Q-3262, Q-3263 and Q-3264 built | `hover::record_taken_point`, `taken_object_point`, `object_point`, `contact_taken` (called from `depot` and `structures::bunker`), `take_destination(.., effective)`. `path::heading_point` reads the hover's taken point. `WalkState::object_point` and `kept`, with `walk::kept_in` and `kept_out`. In `transport::destination_went_inside`, `JumpState::order` and `TeleportState::for_dest` go to none and the walker's point is kept. `objects::go_inside` is called from `econ::slaves`, `parasite` and `spawn` | `claims_s0507_r11.rs` (`t16c_d_stays_where_the_unit_was_at_the_assignment`, `an_unchanged_assignment_leaves_d_and_a_new_one_takes_the_later_position`, `a_walking_holder_goes_on_to_the_point`, `a_walker_ends_its_walk_at_the_point_of_the_assignment`), `claims_s0507_r12.rs` (`a_jumpjet_holder_keeps_its_point_and_commitment`), `claims_s0402_slaves.rs` (`s0507_claim_6a_a_delivered_slave_is_lost_as_a_destination`), `claims_s0614.rs` (`s0507_claim_6a_a_firing_parasite_is_lost_as_a_destination`), `claims_s0506_r40.rs` (`s0507_claim_6a_a_docking_spawned_aircraft_is_lost_as_a_destination`), `claims_s0524.rs` (`s0507_claim_6a_a_displaced_unit_is_not_lost_as_a_destination`), `claims_s0207_teleport.rs` (`claim_20_a_destination_that_goes_inside_keeps_the_pending_jump`) |

Open requests and interims:

- **Q-3261.** It asks what an owner change does in the factory break-off window. Its row in the S-0309 r126 test ends the window by hand.
- **Q-3262.** Interim: a search for a live object destination looks for the object's current cell.
- **Q-3263.** Interims:
  - the hover's *D* height is the surface height at the movement point itself;
  - only an object destination is ignored, and not after claim 10's stop request;
  - contact taken in the same order sets *D* to the docking point.
- **Q-3264.** Interims:
  - an S-0212 claim 24 substitute cell replaces the walker's kept point;
  - the object point's height is not adjusted.

Lanes. Every AI quick-lane game moves, because the state hash covers the new fields and item 1 changes build-up timing. One winner change is traced to item 1's single build-up run: the first refinery's free harvester comes about 48 ticks earlier.

The arena counts are unchanged. Two sweep seeds now give no bunker entry: M7a seed 12 and M7b seed 15. A per-tick check of the bunker entry gates found no taken bunker and no kept walker point; neither item caused this. The causes:
- **M7b seed 15:** no free vehicle with the *enter* cursor stood within reach of a bunker.
- **M7a seed 12:** one house's bunkers stand beyond its WARD's reach. The other house's bunker has the other house's unarmed PREY within its threat reach all game.

The P3 census rule: the batch reads and moves no census back-reference.

## Batch S38: payment-only docking art, a slave miner that creeps, slaves shooting their master, building veterancy (#444, #450, #451, #452)

Base main e533a16, not rebased. No engine change. #444 is compared with S-0402 r86, Q-3254's answer: what is built already matches it, so a claim test guards it. #451 and #450 were traced on synthetic rules. Each follows the claims as written, and each needs behaviour the specs do not give, so each has a request (Q-3274, Q-3275). #452 is a request only (Q-3273). The batch changes only tests, spec requests and this ledger.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #444 (S-0402 r86 claim 11, *What a docking shows*, scenario H29) | For art that names only the payment animation, the engine plays nothing at the start row. It starts the payment animation at the first payment and keeps it, the same object, while it plays. Once it has ended on its own, it starts again at the next payment. The empty-load row removes it. The *finishing* harvester breaks contact in the following tick | matches r86 | none | `claims_s0402_dock_anims.rs`: `claim_11_r86_payment_only_art_starts_keeps_restarts_and_removes`, with an endless and a short synthetic payment animation (c160a48) |
| #451 (S-0402 claims 22, 24, 25 and F7; S-0415 claims 9 and 10) | Every link up to the pack-up works. A 3 × 3 master whose near field is used up packs up toward a site near a far field, and the vehicle appears at the return cell *A* + (1, 1) with destination *A*. At its second group update it is still driving to *A*, so claim 25 keeps that destination and the group travels. At *A* it deploys again with its top-left cell at *A* − (1, 1) and packs up again. It moves one cell along −*x* and −*y* every 180 ticks or so and never reaches the far field. A faster synthetic vehicle that reaches *A* before that update fails to deploy, seeks a field and drives to the far field | matches the claims; request Q-3274 (600e276) | none | `claims_s0402_slaves.rs`: `claim_25_issue_451_automatic_pack_up_redeploys_one_cell_on` (bfd414f) |
| #450 (S-0402 claim 21, S-0301 claim 9, S-0309 claims 2 and 18) | S-0402 claim 21 gives an emerging slave its master as target, and a harvesting slave keeps it. An armed synthetic slave shoots its master: from full strength to 10 in 1500 ticks. The check answers *can fire*, *rearming* or *moving*, never *illegal*. An unarmed slave never hurts it. The answer that would refuse it, the *illegal* of a firer bound to a slave master, is ranked in S-0301 claim 9's precedence paragraphs but defined nowhere. The passive search and retaliation gates hold, against an enemy and against the own house | matches the claims; request Q-3275 (4fbccb2) | none | `claims_s0402_slaves.rs`: `claim_21_issue_450_an_emerged_slave_targets_its_master`; `claims_s0309_slave_gate.rs`: `claim_18_issue_450_a_harvesting_slave_does_not_retaliate` (4496aac) |
| #452 (S-0311, S-0520 claim 22, S-0111 claim 18, S-0415) | The specs give a trainable building kill experience, a rank and an insignia. The lead reports that the original never ranks a building | request Q-3273 (731f1e6) | none | — |

Formats: no simulation state changes. Save stays 157, AI game 134 and wire 12. No quirk switch is added.

Goldens: none move. The batch changes no non-test source.

Lanes compare base e533a16 with the batch at c160a48. The later commits are formatting and this ledger. Logs are in `local-lane/sim-issues/s38/`.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). `diff -rq` of `arena-base/` and `arena-after/` is empty.

**Sweeps** (`m6c-base.txt`/`m6c-after.txt`, and likewise for m7a and m7b). M6c, M7a and M7b over seeds 1–16 pass. Apart from build and timing lines, they are identical to base.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8; `quick-base-7.txt` to `quick-base-10.txt` and `quick-after-7.txt` to `quick-after-10.txt`). Apart from the timing lines, every file is identical to base. Every game ends on the same tick with the same winner.

**Suites.** The AI suite with its ignored tests passes 366 on base and on the batch (`ai-base.txt`, `ai-after.txt`). The sim gate with its ignored tests passes 17 on both (`gate-base.txt`, `gate-after.txt`).

Seams left:
- #451's other suspects are not built, because the main case already stops at claim 25 (Q-3274). They are ore under the foundation, ore the slaves cannot reach, and a site search that fails on every rectangle.
- #450: until Q-3275 is answered, an armed slave holding its master as target can shoot it. For the next report, the pathing report (key P) should show the slave's target id against its master's id, its slave phase and behaviour, its weapon, and whether it emerged or was recalled shortly before.

## Batch S37: a killed infantry unit left in play, and the infantry crash end's answers (Q-3239, #252; Q-3240, #291)

Base main b4e1e09, not rebased. The batch builds the lead's answer to Q-3239 (S-0107 r55 claim 17a, S-0312 r91 claim 16) with its three new switches. Q-3240's answer (S-0611 r12, S-0614 r35, S-0109 r45) is handed back untouched. The design text came first (4d062ab), in *Simulation (sim issues S37 …)*, then the switches (198381a). Each test was committed before its fix, and no fix commit changes a test. Every red was shown in a separate base worktree checked out at the test commit (`t1-red.txt`, `t2-red.txt` and `t3-red.txt` in `local-lane/sim-issues/s37/`). No request is written.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #252 (S-0107 claim 17a, S-0312 claim 16) | A killed infantry unit left in play kept a health of 0. The jumpjet's request for *air-death-start* was forced. A further killing hit while it fell did nothing. The crash end in a high-bridge cell used the surface, and a turned-down *air-death-end* request removed the unit (the S32 interim) | fixed (b03908c; tests 1b9236a, 12 red: the 11 in the new file and the drone test in `claims_s0613.rs`; guards 99f6da9 and 4984bd8, whose rustfmt is c5d4fc8) | `combat::death::regains_health` and `regain_health_turn`, right after the warp step in `tick::object_turn`: an infantry unit with health 0 outside the die, wet-die and air-death kinds gets 1 and stays killed. Such a unit takes the living path, and `behave.rs` no longer turns away a killed unit with health. `jump::fly`: the *air-death-start* request is not forced. A further crash start of a falling or crashed infantry unit kills its passengers again and makes the empty destination assignment, without restarting the fall. `impact` places infantry in a high-bridge cell at the deck's height off the deck, and a vehicle on the terrain. A turned-down *air-death-end* keeps the unit | `claims_s0312_r91_crash_health.rs` (11 tests, both positions of each new switch, and two guards: a unit kept for good is listed in its cell, and the infantry crash end in a high-bridge cell is listed on its layer). Moved: `claims_s0312_crash_infantry.rs` (the interim test now runs with the keep switch off), `claims_s0502_r24_crash.rs` (J31's crash end counts a unit left crashed), `claims_s0613.rs` (claim 3 takes out the revived drone; a new test for its second death in its cloud) |
| #252 (importer) | S-0312 Edge cases: the importer warns about a crashable infantry type whose *air-death-end* entry has no image frames | fixed (06ddf60; test d33af68, 1 red) | `convert::sim::infantry_art` reads the entry's frame count from `sequence_table` and `Crashable` from the type | `sim_tests.rs` (`empty_air_death_end_entries_of_crashable_types_warn`) |
| #252 (S-0107 claim 17a, S-0612 claim 13) | The dominator's `dying` check read the killed flag alone, so a killed unit that had regained a health of 1 counted as dying and was not taken over. It is a living unit for every rule that looks at health | fixed (d6e0a94; test 59be3bc, 1 red) | `domination::dying` uses `damage::dead` (killed with health 0) or sinking. The other readers of the killed flag in the sim are the death machinery itself, the death crate, and rules for vehicles or buildings only, so none treats a revived infantry unit as dying | `claims_s0612.rs` (`claim_13_a_killed_unit_alive_again_is_not_dying`; `claim_13_dying` now also sets a health of 0) |
| #291 (Q-3240) | The events that end an attachment, and S-0611 claim 15's rules after a detachment | handed back untouched | none | — |

Switches: `quirks.crash_end_bridge_height`, `quirks.falling_infantry_regains_health` and `quirks.refused_air_death_end_keeps_unit`, all on by default. `quirks.refused_death_keeps_unit` keeps its meaning and default; only its doc now cites claim 17a. On the branch, the rules quirk test lists only S35's `buildup_twice_without_break_off` and S36's `hovering_jumpjet_body_on_heading`.

Formats (149b1d5): save 157 to 187 and AI game 134 to 165 on the branch (renumbered to 160 and 138 at the merge), since replays change. The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change. Wire stays 12. The sim-gate goldens were regenerated with the ignored tests at 149b1d5 and again on c5d4fc8, and none moved (`golden-update.txt` and `gate-fix2.txt`: `golden.rs` 2 passed).

Lanes compare base b4e1e09 with the batch at 99f6da9. Logs are in `local-lane/sim-issues/s37/`. After the domination fix, every lane below was run again on c5d4fc8 (the `-fix2` files): the arena CSVs, the sweep seed lines and every quick-lane result are identical to the 99f6da9 runs.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-fix.txt`). The recorded M-0001 to M-0003 CSVs are byte-identical to base, and the summary lines are the same: M-0019 0 of 6, M-0021 10 of 50 and M-0020 0 of 25 failing.

**Sweeps** (`m6c-base.txt`/`m6c-fix.txt`, and likewise for m7a and m7b). M6c and M7a seed lines are identical to base: smallest minimum 0 and 2. M7b passes, with seeds 3, 10 and 15 moving in their drain and M7a counters and the summary's mutation, takeover and berserk spreads. A trace of seed 3 shows two killed infantry units with no current sequence, left in play, regaining health at tick 6595. The trace does not show the request itself; a death request turned down for want of image frames (claim 17's `quirks.refused_death_keeps_unit`) is the inferred cause.

**AI quick lane** (seeds 7–10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-fix-7.txt` to `quick-fix-10.txt`). 12 games end on the same tick with the same winner as base. 24 move, and these seven change winner:

| Seed | Game | Base ticks (winner) | Batch ticks (winner) |
|---|---|---|---|
| 7 | 1 | 68251 (house 0) | 117768 (house 1) |
| 7 | 5 | 47071 (house 0) | 42818 (house 1) |
| 8 | 7 | 40300 (house 1) | 39901 (house 0) |
| 8 | 8 | 59415 (house 1) | 43789 (house 0) |
| 9 | 2 | 70011 (house 1) | 29722 (house 0) |
| 10 | 5 | 71431 (house 1) | 46172 (house 0) |
| 10 | 7 | 50584 (house 0) | 72315 (house 1) |

A trace of seed 7, map 0, finds five killed jumpjet infantry units in games 1 and 2. Each was killed while hovering or firing airborne, and the request for *air-death-start* that is no longer forced is turned down. Each then holds a health of 1 for its fall of about 11 ticks (S-0312 claim 16, *Health while it falls*), and the request for *air-death-end* succeeds at its crash end. None is in a high-bridge cell. The other changed games are not traced one by one.

**Suites.** The AI suite with its ignored tests passes on base, on the batch and on c5d4fc8 (`ai-base.txt`, `ai-fix.txt`, `ai-fix2.txt`: 38 binaries ok). The sim gate with its ignored tests passes 17 on the batch and on c5d4fc8 (`gate-fix.txt`, `gate-fix2.txt`), as on base.

Seams left:
- Q-3240 (#291): an `ended` map of detached animations in `anims`, the detaching events (death, sinking, crash start, leaving the map, removal, and cloaking for a non-building), the dispatch in `tick.rs` and `probe.rs`, and claim 15's rules after a detachment. The client needs no change, since it draws only `world.anims.anims`.
- S-0107 claim 17a's early exit for a tunnel is not built, as our engine builds no tunnels.

The P3 census rule: the batch makes or moves no back-reference of `World::census`.

## Batch S39: the nuclear warning marker's tie and the stop at a jumpjet's crash start (#441, #446, S-0513, S-0502)

Base main 2a4d7db, not rebased. The batch builds two answers. S-0513 r129 claim 11 answers Q-3253 (#441), with scenario SW4 parts (b) and (c) and S-0513's change for S-0203 claim 10. S-0502 r30 claim 18 answers Q-3255 (#446), with S-0113 r21 and S-0312 r93. For each item the design text came first, in *Simulation (sim issues S39 …)* in `docs/dev/simulation.md` (5883a70, add20e8), then the failing tests (7d68fae, f5e4ec9), then the fix (ec59e15, 22da0c6). No fix commit changes a test. Each red was shown in a separate base worktree checked out at the test commit (`red-1.txt`, `red-3.txt`). Two requests are filed from item 2 (2b8076d): Q-3281 and Q-3282. A rustfmt commit (87b6491) and a clippy fix (3f5898f) close the code. Logs are in `local-lane/sim-issues/s39/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #441, the order (S-0513 claim 11 r129, S-0203 claim 3) | The marker was made after its missile: `nuke::one_time` and `nuke::silo_turn` made the missile first, and in a silo launch the take-off animation came between them. So the marker stood after the missile in the update order | fixed (ec59e15; tests 7d68fae, red at 7d68fae: marker index 2, expected 0) | `nuke::marker` makes the marker untied, just before `missile`, and `nuke::tie` then ties it through `anims::tie_warning`. In a silo launch the order is the marker, the upward missile, then the take-off animation. The marker is made only after every check that can stop the missile has passed | `claims_s0513_r129.rs`: `claim11_r129_the_marker_enters_play_just_before_its_missile` |
| #441, the tie and the removal (claim 11 r129, SW4 (b) and (c), S-0203 claim 10) | The marker's end was lazy. `anims::turn` deleted a marker whose missile had gone at the marker's next turn, citing S-0109 claim 22, which treats the tie as an attachment. A missile that left in its own turn cost the marker after it its turn (one counting removal), and the marker went in the next tick. r129: the tie is a link of lifetime only, and the marker leaves at once inside the missile's removal, both removals counting | fixed (ec59e15) | `AnimWorld::markers`, an ordered map from a missile to its marker, saved and hashed. `combat::projectile::remove`, the one way a projectile leaves play, calls `anims::missile_left` after its own leave, which removes the marker through `anims::remove`. That removal reads `quirks.update_order_removal_skip`. The lazy branch of `anims::turn` is gone. A marker ended by its own playback leaves a stale entry, which the missile's removal drops | `sw4_b_the_marker_keeps_its_place_and_is_gone_with_its_missile`; `sw4_c_both_removals_count_with_the_switch_on` (three vehicles after the missile, the first two miss their turn and the third takes it; a one-time and a silo strike); `sw4_c_neither_removal_costs_a_turn_with_the_switch_off`; `claim11_r129_the_tie_survives_a_save_and_load`. `claims_s0513_nuke.rs`'s two marker tests keep their assertions and now cite r129 |
| #441, the place (SW4 (b) runs (i) to (iii)) | `cell_point(w, cell, false)` already gave the cell centre at the terrain surface height with no bridge lift. The marker had no owner, so it never followed its missile | as built | none | `sw4_b_the_marker_keeps_its_place_and_is_gone_with_its_missile` (open ground, a silo five cells away), `sw4_b_iii_on_a_high_bridge_the_marker_stands_below_the_deck`. Their position checks hold on base. Both fail there only at their last check, that the marker is gone at the end of its missile's tick (`red-1.txt`: all six tests of the file fail on base) |
| #441, the silo's tick (claim 11 r129, claim 8) | `silo_turn` runs in the silo's update in which the upward missile leaves, so the marker is made in that update | as built | none | `claim11_r129_the_marker_enters_play_just_before_its_missile` (the silo run) |
| #446, the crash-start stop (S-0502 claim 18 r30, S-0312 claim 16 r93) | `jump::fly::start_crash` made one empty destination assignment at the hit (r24). A balloon-hover vehicle that held a destination and a target could reject it under claim 28, and then kept its order-level destination and its movement point through the fall. r30: the crash start is S-0607 claim 13's stop on dropping | fixed (22da0c6; tests f5e4ec9, red at f5e4ec9: 4 tests) | `jump::crash_start_stop`, called by `start_crash` in place of the one assignment. In order: the first empty assignment unless `clear_rejected` rejects it; the target cleared, with the burst counter (S-0301 claim 4); `lift::break_any_contact` (now `pub(crate)`); the direct stop `jump::lift_release_stop(w, id, false)`; the second empty assignment, which claim 28 can no longer reject. Infantry with a commitment reroutes three times, so it takes three draws over 0 to 3. The drop's deselection is left out, as S-0502 does not list it | `claims_s0502_r30.rs`: `j12_c_a_retained_destination_ends_empty_and_the_point_is_rerouted`, `j31_a_infantry_takes_three_reroute_draws_at_the_crash_start`, `claim18_r30_the_crash_start_breaks_off_contact` (red at f5e4ec9), `claim18_r30_without_a_commitment_the_stop_changes_no_point` (a guard, passes at f5e4ec9). `claims_s0502_crash.rs`'s `claim_18_crash_start` now expects three draws, not one (red at f5e4ec9: 4 against 6). `claims_s0502_r24_crash.rs` keeps its assertions and cites r30 |
| #446, the path of the fall (claim 18 r30 *Path of the fall*, J12 (a) and (b)) | Every crashing jumpjet falls this way, balloon-hover type or not. The body facing follows the heading, claim 10. One killed while cruising steps along its spinning heading and leaves an open arc. One killed hovering with no order keeps current and maximum speed 0 and falls straight down | as built; guard test | none | `j12_r30_the_path_of_the_fall_as_built` (passes at f5e4ec9) |
| #446, the maximum speed at the first crash update | The update that enters *crashing* still runs the preceding state's outcome against the movement point. A unit killed while cruising, now rerouted to a point near it, can have F6's bands lower its maximum speed in that update (a temporary measurement: 14 at the hit, 7 from the first crash update on). J12 (a) does not expect this | request Q-3281 (2b8076d) | none | none asserts J12 (a)'s or (d)'s maximum speed |
| #446, the no-cell switch off | With `quirks.jumpjet_no_cell_self_destruct` off, a failed search in the stop still gives a killed unit the switch-off outcome | request Q-3282 (2b8076d) | none | — |
| #446, the client (S-0113 r21 claim 18) | The tilt-crash flag decides how a jumpjet body is drawn, which is presentation. The simulation's fire points already ignore rocking (claim 19's existing seam), which is exact for an unflagged jumpjet | handed back (client) | none in the simulation | — |

Formats (ec59e15): `AnimWorld::markers` is saved and hashed, so save goes from 158 to 191 and AI game from 136 to 169 on the branch (renumbered to 162 and 140 at the merge). The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change. Wire stays 12. No claim built here names a new quirk switch: item 1 reads the existing `quirks.update_order_removal_skip` and item 2 the existing `quirks.jumpjet_no_cell_self_destruct`, so the rules schema does not change.

Goldens (ec59e15): all nine sim-gate goldens move, and every hash line in each file changes, as the new saved field enters every state hash from the first tick. The line counts are unchanged. The goldens were made with both fixes in the tree. The golden test then passed at f5e4ec9 (item 1's fix, item 2's tests only; `golden-at-f5e4ec9.txt`), so item 2 does not move them.

Lanes compare base 2a4d7db with the batch at 87b6491. The later clippy fix changes no behaviour, and the gate and the AI suite ran again on it. The quick lane was also run on an item-1-only build (f5e4ec9) to split its moves by item.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). `diff -r --brief` of the two output directories is empty. The text outputs differ only in load and play times and the output directory names (`arena-diff.txt`). These rows fail, on base and after:

## Batch S40: a balloon-hover bomber over its target, and what ends an attachment (#445, Q-3257; #291, Q-3240)

Base main 5b6d561, not rebased. The batch builds two answers. S-0502 r34 and S-0310 r13 answer Q-3257 (#445). S-0109 r45, S-0611 r12 and S-0614 r35 answer Q-3240 (#291). The design text came first for each item (5fe88ae, 8c6b6ef), in *Simulation (sim issues S40 …)* in `docs/dev/simulation.md`. Each test was committed before its fix, and no fix commit changes a test. Every red was shown in a separate base worktree checked out at the test commit (`red1.txt`, `red2.txt`, `red2b.txt`). Two AI test commits follow item 1's fix by the main session's leave (7cdc65d, 4230677). No request is filed. Logs are in `local-lane/sim-issues/s40/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #445 (S-0502 claim 16, r34) | A jumpjet's movement point for an object destination was the object's position. For a building that is the centre of its top-left cell, so a balloon-hover bomber sent at a building hovered there, not facing the building's reference point, and never bombed it | fixed (8551a8d; switch 6471b31; tests 6f66ea4, 8 red) | `jump::object_point`, read once at the assignment by `jump::order_point`. A rider in an open-topped transport gives its own position. Any other object gives `lift::lifter_point` with the jumpjet as the seized unit: a building's reference point, or its docking point with the pad, repair-depot or vehicle-bunker flag, and a mobile object's heading point. The point does not follow the object | `claims_s0502_r34_balloon_target.rs` (12 tests: S-0502 J34 parts (a) to (g) and Q-3257's half-cell reach; 8 fail on base, and the 4 that pass are the as-built guards). `claims_s0608.rs` claim_6: the linking case's drain weapon gets no fire-while-moving flag (it passes on base) |
| #445 (S-0310 claims 7 and 9, r13) | Claim 7's first *moving* condition reads the destination for every vehicle, balloon-hover types included. With `quirks.hovering_jumpjet_body_on_heading` off, a hovering jumpjet vehicle counts as having no destination for claim 9's reply | fixed (8551a8d; switch 6471b31) | `combat::check::vehicle_moving` reads `occupancy::in_progress` for a balloon-hover type when the new `quirks.balloon_hover_mobile_fire_by_destination` is off (default on, the original). `jump::hover_reply_ignores_destination` is read by `combat::behave::vehicle_reply` | the same file, both positions of each switch |
| #445 (AI tests) | `air_bomber.rs`'s three #445 traces had asserted the old stall | moved (7cdc65d) | the bomber now bombs the plant on its way in. From afar, east fires too (`("east", false)` → `("east", true)`). `a_bomber_that_reaches_half_a_cell_stalls_over_the_plant` is renamed `…_bombs_the_plant_on_its_way_in`: it now asserts `fired`, a movement point equal to the plant's reference point and an aim error within 2048. The AI bomber test's half-cell row goes from `false` to `true`, and its one-order assertion is unchanged | — |
| #445 (AI fixtures) | With claim 16 and S-0307 claim 9's lengthening for a building, the M7b fixture's SUCK (a reach of a tenth of a cell) is met on the way in, about a cell short of the foundation. In `antiair.rs`'s stand-off scene, FLAKT had fired by the time the link formed | moved (4230677) | `fixture_m7b.rs`: SUCK gets `fire_while_moving: Some(false)`, a synthetic fixture choice that keeps the documented intent (the disc fires once it hovers over the building). `an_anti_air_unit_standing_off_the_drainer_does_not_answer`: FLAKT's spawn goes from (-1, 3) to (4, 3). From (-1, 3) and (2, 3) it had fired; from (3, 3) it stands out of range; (4, 3) adds a cell of margin. The name and assertion are unchanged | — |
| #291 (S-0109 claim 22, r45) | No event ended an attachment. r45 names the *End with the owner* list | fixed (c9ee82a; tests 4242679, 15 red; test update 59cf169, 2 red) | `anims::owner_ended` moves each attached animation into the saved `AnimWorld::ended`, with S-0611 claim 15's release and the parachute's end at the event. Its next turn only deletes it (`anims::takes_turn` in `tick::object_turn` and `probe::classify`). It is called at the killing hit, the start of sinking, an aircraft's and a jumpjet's crash start, going off the map, removal, the start of cloaking, a non-building's full cloak (the forced one at placement included), the vehicle thief's and the vehicle-like building's takeovers, and the hijack seam. An event in the animation's own turn ends it when its playback returns. `parasite::tend_grab_anim` loses its reattachment branch (S-0614 claim 25) | `claims_s0109_r45_attachment_end.rs` (17 tests; 15 fail on base, and the guards for mind control and another rule's removal pass). `claims_s0109_playback.rs` and `claims_s0109_r41.rs`: r41's "still drawn until its next update" becomes r45's "not drawn from the event on" (59cf169, both red at that commit). `claims_s0611_moving.rs` moves its owner on the map instead of taking it off and back |
| #291 (S-0601 claim 18) | The control ring stayed while its object was cloaked, off the map, sinking, crashing or destroyed | fixed (c9ee82a; tests 4242679) | `mind::ring` gives a ring only while the object is on the map, not destroyed, not sinking or crashing and neither cloaking nor cloaked. It comes back with no switch. It changes no game state and takes no draw | the ring tests in `claims_s0109_r45_attachment_end.rs` |

Switch: `quirks.balloon_hover_mobile_fire_by_destination`, default on. On the branch, the rules quirk test lists only S-0519's `airstrike_line_marker_over_shroud`, which is not this batch's (`ws-i2.txt`, line 3441).

Formats (c9ee82a): `AnimWorld::ended` is a new saved field, and replays change. On the branch the save format goes from 160 to 195 and the AI game format from 138 to 173, the numbers the main session gave this batch (renumbered to 163 and 141 at the merge). Wire stays 12. Item 1 regenerated the goldens once with the ignored tests (7f02994). Only `duel-m7b.txt` moves, from tick 576, in 224 lines. With the fix and without the fixture line (`golden-nofixture.txt`, in the base worktree at 7cdc65d), duel-m7b also moves, but to other hashes: the move comes from the behaviour alone and changes again with the fixture line. Item 2 regenerated the goldens again (78a5d82). All nine files move from the first tick, as the new field enters every state hash, and each keeps its length.

Lanes compare base 5b6d561 with the batch at 6d6ce5f, the last code commit. The AI suite ran again on the final commit.

**Oracle arena** (`--record all`; `arena-base.txt`, `arena-i2.txt`). `diff -r` of the two output directories is empty. These rows fail, on base and after:
- M-0001: 10 of 96;
- M-0002: 4 of 24;
- M-0003: 3 of 35;
- M-0019: 0 of 6;
- M-0020: 0 of 25 checked;
- M-0021: 10 of 50.

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`). M6c, M7a and M7b over seeds 1 to 16 pass, before and after. Apart from build lines and timings they print the same lines as base (`m6c-diff.txt`, `m7a-diff.txt`, `m7b-diff.txt`).

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt`, `quick-item1-7.txt` to `quick-item1-10.txt`, `quick-after-7.txt` to `quick-after-10.txt`). Each seed's games are numbered 1 to 9: map 0 with sides 0v1, 0v2 and 1v2, then map 2, then map 8. Ten games end on the same tick with the same winner as base. 26 games move, and eight of them change their winner:

| Seed | Game | Base | Item 1 only | Batch | Moved by |
|---|---|---|---|---|---|
| 7 | 1 | 79563, house 1 | 56117, house 1 | 70483, house 0 | items 1 and 2; winner by item 2 |
| 7 | 2 | 52281, house 1 | 52281, house 1 | 112001, house 0 | item 2; winner |
| 7 | 4 | 46497, house 0 | 46497, house 0 | 47333, house 0 | item 2 |
| 7 | 5 | 75466, house 1 | 75466, house 1 | 47207, house 1 | item 2 |
| 7 | 6 | 84712, house 0 | 58684, house 1 | 58684, house 1 | item 1; winner |
| 7 | 7 | 32400, house 0 | 32400, house 0 | 29114, house 0 | item 2 |
| 7 | 8 | 35923, house 1 | 35923, house 1 | 29849, house 1 | item 2 |
| 8 | 1 | 50672, house 1 | 70051, house 1 | 36009, house 0 | items 1 and 2; winner by item 2 |
| 8 | 2 | 52268, house 0 | 52268, house 0 | 27301, house 0 | item 2 |
| 8 | 3 | 88039, house 0 | 87326, house 0 | 70610, house 0 | items 1 and 2 |
| 8 | 6 | 39971, house 0 | 42776, house 0 | 42776, house 0 | item 1 |
| 8 | 7 | 53668, house 0 | 68305, house 0 | 72911, house 1 | items 1 and 2; winner by item 2 |
| 8 | 8 | 66645, house 1 | 66645, house 1 | 31517, house 0 | item 2; winner |
| 8 | 9 | 32877, house 0 | 35217, house 0 | 35217, house 0 | item 1 |
| 9 | 1 | 34369, house 0 | 34369, house 0 | 38153, house 0 | item 2 |
| 9 | 2 | 45639, house 0 | 45639, house 0 | not over at 120000, no winner | item 2; winner |
| 9 | 3 | 39407, house 0 | 36182, house 0 | 36182, house 0 | item 1 |
| 9 | 4 | 39101, house 0 | 39101, house 0 | 42385, house 0 | item 2 |
| 9 | 5 | 46477, house 1 | 46477, house 1 | 45536, house 1 | item 2 |
| 9 | 6 | 97544, house 1 | 105613, house 0 | 105613, house 0 | item 1; winner |
| 9 | 7 | 28583, house 0 | 28583, house 0 | 20486, house 0 | item 2 |
| 9 | 8 | 30405, house 0 | 30405, house 0 | 31518, house 0 | item 2 |
| 10 | 3 | 54257, house 0 | 60747, house 0 | 60747, house 0 | item 1 |
| 10 | 4 | 34558, house 0 | 34558, house 0 | 26976, house 0 | item 2 |
| 10 | 5 | 33708, house 0 | 33708, house 0 | 31935, house 0 | item 2 |
| 10 | 8 | 40656, house 1 | 40656, house 1 | 40405, house 1 | item 2 |

Each winner change was traced as a single pair (`GLASNOST_AI_SIDES`), which reproduced the full set's result in every case. The six item-2 winner changes were traced from the item-1-only build to the batch, by state hashes (`GLASNOST_AI_HASH=1`) and then an object dump of the first differing tick (`GLASNOST_AI_OBJS`). Each first differs at one tick in one object: a house 0 jumpjet infantry unit of type 4, health 0, at its crash start. Its target goes from an object to none and its target epoch goes up by one, which is r30's stop. Where its point was its own position it is rerouted, with draws (`tr-s<seed>g<game>-objs-diff.txt`):
- seed 7 game 1, tick 15126, object 1239 (*hovering*): target object 1462 → none; jump destination (18752, 15552) → (18880, 15424); draws taken 271115 → 271116;
- seed 7 game 2, tick 10609, object 1486 (*hovering*): target object 1337 → none; destination unchanged; draws 225018 → 225019;
- seed 8 game 1, tick 9760, object 1244 (*hovering*): target object 1218 → none; destination (12608, 17088) → (12736, 16960); draws 160240 → 160242;
- seed 8 game 7, tick 8686, object 268 (*hovering*): target object 169 → none; destination and draws unchanged;
- seed 8 game 8, tick 11188, object 790 (*cruising*): target object 538 → none; destination (9024, 20416) → (9152, 20288); draws 212974 → 212975;
- seed 9 game 2, tick 10600, object 1523 (*cruising*): target object 1414 → none; destination unchanged; draws 215330 → 215333.

The two item-1 winner changes were traced from base (rebuilt at 2a4d7db) to the item-1-only build. State hashes are not comparable across the format change, so house timelines (`GLASNOST_AI_TRACE=10`) and single-tick object dumps bisected the window first, then a dump of every tick in it:
- seed 7 game 6 (`tr-s7g6-w-diff.txt`): ticks 34131 to 34146 are equal. At tick 34147 object 8439 (house 0 vehicle, type 17) has damage smoke animation 8739 on base and 8740 with item 1, and the draws taken are 597771 on base and 597770 with item 1. House 0's silo (object 1968) has a missile in flight (busy until tick 34014).
- seed 9 game 6 (`tr-s9g6-w-diff.txt`): ticks 33451 to 33478 are equal. From tick 33479 to the window's end at 33500 only object 11293 differs, a house 0 spy plane (aircraft type 10, placed at tick 33439). At tick 33479 its position is (17479, 1930, 2124) on base and (17514, 1942, 2124) with item 1, with no difference in draws or spawns. A silo had launched earlier (busy until tick 33311).

Both fit a turn that a marker's removal now costs, or no longer costs, a nearby object. The dump has no animations or projectiles, so the removal itself is not observed directly.

**Suites.** The AI suite with its ignored tests passes 381 on base and after, with every test binary's result line equal (`ai-base.txt`, `ai-after.txt`). The sim gate with its ignored tests passes 17 on both (`gate-base.txt`, `gate-after.txt`), after the goldens were regenerated.

Final checks. Clippy found one `question_mark` lint in `nuke::marker`. It is fixed with no behaviour change (3f5898f), and these ran on that commit:
- `cargo test --workspace --no-fail-fast` (`final-ws-2.txt`): 9329 tests pass, against 9318 on base. The only failures are the two known ones, which name the same quirks as on base: `quirks::tests::every_spec_quirk_has_a_switch` and app `every_rendering_spec_quirk_has_a_switch`;
- `cargo clippy --workspace --all-targets -- -D warnings` (`final-clippy-2.txt`);
- `cargo fmt --check`;
- the wasm32 build of glasnost-app (`final-wasm.txt`);
- `cargo xtask check` (`final-xcheck.txt`): fails only on the 8 known stock-guard problems in M-0027;
- `cargo xtask gate` (`final-xgate.txt`): native 17 pass, and all nine goldens match under wasm32;
- the AI suite with its ignored tests (`final-ai.txt`): 381 pass.

Seams left:
- Q-3281 (the maximum speed at the first crash update) and Q-3282 (the no-cell switch off at the crash start) wait for answers.
- The client, item 1: nothing new. The marker is an ordinary animation with no owner, at the target cell's centre, and leaves play with its missile. The viewer test (#441, S34's seam) reads `Anim::warning` as before.
- The client, item 2 (S-0113 r21 claim 18): the tilt-crash flag (`TiltCrashJumpjet` on an object type, off by default, S-0113 *Import mapping*) is not imported yet; no crate reads it. The jumpjet locomotor's body is drawn with its pitch and roll only when its type has the flag and the pitch or the roll is at least 0.005 radians in magnitude. Otherwise it is drawn turned to its body facing only, with no pitch, roll or slope tilt, in every flight state, crashing included. A crash's spin is drawn through the body facing, which the simulation already keeps on the spinning heading.
- Claim 19's flagged fire point (a flagged jumpjet's fire point through its rocking angles) stays the existing seam: the simulation's fire points ignore rocking.

The P3 census rule: `AnimWorld::markers` maps projectile numbers to animation numbers. It is not an object-table reference, so the batch makes or moves no back-reference of `World::census`.

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`). M6c, M7a and M7b over seeds 1 to 16 pass, before and after. M6c and M7a print the same lines as base apart from build and timing lines. M7b moves on every seed. `m7b-i2.txt` equals `m7b-i1.txt` (item 1 alone), so the move is item 1's. Every seed still plays its 8000 ticks with no result and no drain counter below 3, and the first link is 19 to 31 ticks later on every seed (seed 1: 6107 → 6126, seed 5: 6032 → 6063). In the summary:
- drain orders: median 4 → 3;
- drain links formed: max 4 → 3;
- credits drained: min 1560 → 1520, median 2360 → 2280, max 3360 → 3320;
- drained-building ticks: min 4119 → 4026, median 5437 → 5474, max 6897 → 6893;
- mutator launches: min 6 → 5, median 10 → 9;
- infantry mutated: min 18 → 16, max 33 → 28;
- dominator launches: max 9 → 10;
- objects taken over: min 13 → 15, median 18 → 19, max 25 → 23;
- berserk weapon orders: min 16 → 14, median 26 → 31, max 47 → 60;
- units made berserk: min 13 → 14, median 17 → 18, max 30 → 27;
- takeover orders: max 6 → 8;
- absorber entry orders: median 11 → 9, max 17 → 13;
- infantry absorbed: median 7 → 6;
- grinder entry orders and units ground: max 5 → 6;
- bunker exit orders: max 0 → 1.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt` and `quick-i2-7.txt` to `quick-i2-10.txt`). 5 games end on the same tick with the same winner as base. 31 move, and these five change winner:

| Seed | Map | Sides | Base ticks (winner) | Batch ticks (winner) | First difference | Cause |
|---|---|---|---|---|---|---|
| 8 | 2 | 1, 2 | 72985 (house 0) | 67172 (house 1) | tick 18504: object 4730, a house 1 jumpjet vehicle (type 70) attacking house 0's building 997; `jump.dest` (18816, 9088, 624) → (19072, 9216, 624) | item 1 |
| 8 | 8 | 1, 2 | 65040 (house 1) | 58666 (house 0) | tick 25969: object 6147, a house 0 jumpjet vehicle (type 15) attacking house 1's infantry unit 7212; `jump.dest` (10672, 15552, 650) → (10688, 15552, 546) | item 1 |
| 9 | 0 | 0, 1 | 111410 (house 1) | 51912 (house 0) | tick 22949: object 4824, a cloaked house 1 vehicle (type 24, not a jumpjet), in AreaGuard on base and in Move with an attack-move to cell (66, 82) in the batch | item 2 |
| 10 | 2 | 1, 2 | 63858 (house 1) | 84310 (house 0) | tick 17351: object 2456, a house 1 jumpjet vehicle (type 70) attacking house 0's building 1977; `jump.dest` (17536, 32384, 624) → (17792, 32512, 624) | item 1 |
| 10 | 8 | 1, 2 | 39946 (house 0) | 95421 (house 1) | tick 17154: object 3280, a house 1 jumpjet vehicle (type 70) attacking house 0's building 410; `jump.dest` (7040, 18048, 832) → (7424, 18304, 832) | item 1 |

Each flip was traced with the timeline (`GLASNOST_AI_TRACE`, `trace-*`), then with whole-object dumps (`GLASNOST_AI_OBJS`, `objs-*`), down to the first tick at which a dumped line differs. Every line before it is equal. `save::state_hash` parts from tick 1, because of the new saved field, so it could not be used. In the four item 1 flips, the first difference is only `jump.dest`, which is claim 16's object point (a building's reference point, a mobile object's heading point).

In 9/0/0,1 every dumped field is equal before tick 22949. The cause therefore lies in state the dump leaves out: the animations and the AI brains. An item-1-only build at 8c6b6ef (`objs-9-0-w2-i1.txt`) prints the same lines as base over ticks 22931 to 22965. It parts from the batch at the same line, so the first difference is item 2's. Played out alone, item 1 also flips this game (`quick-i1-9-0-01.txt`: house 0 after 68017 ticks). It parts from base after tick 22965, and that was not traced further.

**Suites.** The AI suite with its ignored tests passes on base, after item 1, on the batch and on the final commit (`ai-base.txt`, `ai-i1.txt`, `ai-i2.txt`, `ai-final.txt`: 41 binaries ok, 381 tests). The sim gate with its ignored tests passes 17 on base (`gate-base.txt`). After item 1's fix, `full_run_matches_golden` failed at duel-m7b tick 576 (`gate-i1.txt`). With the regenerated goldens it passes 17 (`gate-i2.txt`). On 6d6ce5f these also pass:
- `cargo test --workspace --no-fail-fast`, apart from the rules quirk test above (`ws-i2.txt`, at 78a5d82);
- `cargo clippy --workspace --all-targets -- -D warnings` (`clippy-i2b.txt`);
- `cargo fmt --check`;
- the wasm32 build of glasnost-app;
- `cargo xtask check`, apart from the 8 known stock-guard problems in M-0027 (`xcheck-final.txt`);
- `cargo xtask gate`.

Seams left:
- S-0502 claim 16: `lift::lifter_point` reads no next movement point for an aircraft, so an aircraft gives its position (claim 16 rates aircraft `unmeasured`). Claim 23's own-cell filter is left as it was.
- S-0109 claim 22: no underground type moves in our engine, so going underground has no call. The hijack is still `mp_triggers::hijack_seam`.
- The client needs no change for either item. It draws `world.anims.anims`, which no longer holds an ended animation. Its control-ring marker follows `mind::ring`. When the ring's art is drawn, each return of the ring should start a new playback, with draws from the cosmetic random stream (S-0202 claim 11).

The P3 census rule: the batch makes or moves no back-reference of `World::census`.

## Batch S42: the break-off window's owner change, a walker's object point, a hovering vehicle's repeated assignment and a bunkered vehicle's damage (Q-3261 to Q-3264, #457)

Base main 43eaa36, not rebased. The batch builds four answers: Q-3261 (S-0406 r20 claim 17, S-0309 r145 claim 31), Q-3262 (S-0208 r68 claim 25), Q-3263 (S-0503 r30 claim 20) and Q-3264 (S-0208 r71 claims 18 and 26). It also checks issue #457 against S-0302. The design text came first, in *Simulation (sim issues S42 …)* in `docs/dev/simulation.md` (f14458f), and was corrected for the claim 26 hold (53be82c). For each item the failing tests came before the fix: 91d4ab4 then 6699244, d1e89fd then 7335015, 31bef03 then 61a24a9, and f66b778 then aff744e. No fix commit changes a test. Each red was shown in a separate base worktree checked out at the test commit (`red-q3261.txt` to `red-q3264.txt`). Three requests are filed:
- Q-3301 (9274a34): a cell destination under claim 25;
- Q-3303 (2a598e4): claim 26 against S-0605 claim 16;
- Q-3302 (38e9b76): #457.

The #457 guard tests are 7ebcd89. A rustfmt commit (1f7abd9) and a clippy fix (88e3b8b) close the code. Logs are in `local-lane/sim-issues/s42/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-3261 (S-0406 claim 17 r20, S-0309 claim 31 r145) | `ownership::opening` set the opening and ready marks but left `building_up`. A building whose owner changed in the factory break-off window stayed under construction until its next turn's end. r145: the change ends the window | fixed (6699244; tests 91d4ab4, red at 91d4ab4: 3 tests) | `opening` notes `placement::in_break_off_window` before it acts. After the grand opening it calls `placement::end_break_off_window`, which leaves a completed record keeping only the opening mark. The idle body (`econ.idle_body`) is kept | `claims_s0406_r20.rs`: `r20_construction_current_in_the_window_the_change_ends_it`, `r20_construction_queued_in_the_window_the_change_ends_it`. `claims_s0605_build_up.rs` (#347): its rows were moved to the construction state (window ended by hand), and `r20_an_owner_change_in_the_break_off_window_finishes_the_build_up` was added (red at 91d4ab4) |
| Q-3262 (S-0208 claim 25 r68) | A walker's searches looked for the cell of `path::heading_point`, the object's position at the search, not the locomotor point given at the last assignment. A failed fresh search near the point did not clear the destination | fixed (7335015; tests d1e89fd, red at d1e89fd: 2 tests) | `walk::object_locomotor_point` and `path::search_goal` give the point's cell to `compute_once`, `search_gives_up` and the standing walker. `walk::cleared_near_point` in `path::failed` clears an untethered walker's destination within the close-enough distance. Where row 3 keeps the destination, the locomotor point is cleared and no further search is made | `claims_s0208_r68.rs`: `r68_the_fresh_search_looks_for_the_points_cell`, `r68_a_walker_in_the_points_cell_keeps_its_destination_beyond_close_enough` (both red at d1e89fd), `r68_a_walker_in_the_points_cell_within_close_enough_loses_its_destination` (a guard, passes at d1e89fd) |
| Q-3262, a cell destination | Whether the clearing, the no-further-search rule and the standing rule hold for a cell destination | request Q-3301 (9274a34) | none; cell destinations unchanged | — |
| Q-3263 (S-0503 claim 20 r30) | `hover::object_point` took a depot or bunker heading point only in contact, else the object's position. Contact taken later moved *D*. An unchanged object destination after a stop request, or an unchanged cell, was not ignored | fixed (61a24a9; tests 31bef03, red at 31bef03: 7 of 8) | `hover::object_point` follows S-0502 claim 16: a building's foundation centre, or its docking point for the pad, repair-depot and vehicle-bunker flags; a mobile object's heading point (`lift::lifter_point`); an open-topped rider's own position. `take` no longer reads `stopped`. `hover::assignment_ignored` keeps a repeated order's stored path. Only `depot::order_entered` calls `hover::contact_taken` | `claims_s0503_r30.rs`: the building centre, the walking infantry's next point, the bunker's quadrant point without contact, a later contact, the depot enter order (a guard, passes at 31bef03), a stop then the same cell, a stop then the same object, and the repeated order's path. The 31bef03 message says six fail on base; seven do, because the bunker test (with its bunker moved off its quadrant point) fails too. `claims_s0507_r11.rs`'s `an_unchanged_assignment_leaves_d_and_a_new_one_takes_the_later_position` now expects a walker's next movement point (red at 31bef03) |
| Q-3264, claim 18 r71 | An object's point in a high-bridge cell had no 414-lepton addition. With the switch off, `end_of_walk` tested an object point without the deck height | fixed (aff744e; tests f66b778, red at f66b778: 3 tests) | `walk::raised_point` in `take_destination` adds 414 under `quirks.walker_bridge_destination_deck` (default on, unchanged). With it off, `walk::object_point_height` tests the terrain height at the point, plus 416 on the deck. Switch description and schema refreshed in the same commit | `claims_s0208_r71.rs`: `r71_an_object_under_the_deck_gives_h_plus_414_and_the_walk_ends_on_the_deck`, `r71_an_object_on_the_deck_gives_h_plus_830_and_claim_25_clears_it`, `r71_switch_off_an_objects_point_is_tested_on_the_arrival_layer` |
| Q-3264, claim 26 r71 | Built as written, the substitute replaces a queued enter's building. In `claims_s0605.rs` `claim_16_back_in_after_release_both_positions`, the released walker never gets back in: guard gives the building with *enter* queued, a substitute replaces it, and the enter never gives the building again (`q3264-trace.txt`) | request Q-3303 (2a598e4); held | none; the cell-only rewrite and `walk::kept_out`'s interim branch stay | none for claim 26 |
| #457 (S-0302 claim 8, *Hits without a warhead*) | Every damage path reaches `damage::apply_hit`, whose stage 2 bunker rows match claim 8. Area damage reaches them through `area::detonate` for each object hit. A bunkered vehicle takes a penetrating warhead's damage and nothing else. The bunker takes splash at its own point. A hit without a warhead does nothing unless it ignores defences | as built; guard tests; request Q-3302 (38e9b76) | none | `claims_issue_457.rs`: `a_direct_non_penetrating_hit_hurts_neither_the_vehicle_nor_the_bunker`, `a_direct_penetrating_hit_hurts_the_vehicle_and_not_the_bunker`, `splash_on_the_vehicle_and_on_the_bunker_follows_the_two_rows`, `a_hit_without_a_warhead_on_the_bunkered_vehicle`, `a_hit_that_ignores_defences_passes_both_rows` (all pass, `i457.txt`) |

Formats (71fe5cd): no saved field is added, but the replays change, so save goes from 162 to 199 and AI game from 140 to 177 (renumbered to 164 and 142 at the merge). The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change. Wire stays 13. Item 4 changes the description of `quirks.walker_bridge_destination_deck`, and the rules schema is refreshed in aff744e. No new switch is added.

Goldens (71fe5cd): `golden/duel-diagonal.txt` and `golden/duel-m6c.txt` move (`golden-regen.txt`); the other goldens are unchanged.

Lanes compare base 43eaa36 with the batch at 71fe5cd. The later clippy fix and design correction change no behaviour.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). The text outputs differ only in timings and output paths. These rows fail, on base and after:
- M-0001: "scored pairings failing: 10 of 96" (win test 3, side outcome 10);
- M-0002: "4 of 24";
- M-0003: "3 of 35";
- M-0019: "rows failing: 0 of 6";
- M-0020: "rows failing: 0 of 25 checked (28 rows)";
- M-0021: "rows failing: 10 of 50".

The M-0019 to M-0021 values are read from `arena-after.txt`. The diff with base shows only timings and paths, so they are the same on base. `ours-M-0001.csv` differs from line 1139 (per-game records). `ours-M-0002.csv` and `ours-M-0003.csv` are identical.

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`). M6c, M7a and M7b over seeds 1 to 16 pass, before and after. M7a and M7b differ only in timing and target-path lines. In M6c (`m6c-after.txt`) three seeds move:
- seed 1: 11735 to 12000 ticks;
- seed 10: 11393 to 10258;
- seed 13: 8069 to 7675, with its smallest counter going from 3 to 0 (below 3: "jumpjet orders (S-0502) = 1", "support beams (S-0518) = 0").

These summary rows move (base `m6c-base.txt`, after `m6c-after.txt`):
- engineer orders (S-0509): min 5 median 6 max 7, then min 5 median 6 max 6;
- tech building income cycles (S-0515): min 1532 median 2271 max 2347, then min 1453 median 2271 max 2347;
- jumpjet orders (S-0502): min 2 median 21 max 67, then min 1 median 22 max 67;
- jumpjet flight (S-0502): min 5146 median 9536 max 9543, then min 4886 median 9536 max 9543;
- jumpjet units built and flown (S-0417 claim 15): min 6 median 12 max 19, then min 4 median 11 max 18;
- radiation exits (S-0525): min 6 median 16 max 97, then min 5 median 17 max 97;
- radiation in the field (S-0525): min 3931 median 7167 max 11430, then min 3931 median 7172 max 11430;
- chrono erasure orders (S-0514): min 13 median 29 max 47, then min 13 median 29 max 40;
- erasures begun (S-0514): min 6 median 13 max 18, then min 6 median 13 max 17;
- disarm orders (S-0519): min 10 median 12 max 18, then min 10 median 12 max 16;
- suicide orders (S-0519): min 10 median 19 max 29, then min 9 median 19 max 26;
- sabotage orders (S-0519): min 10 median 17 max 25, then min 9 median 16 max 23;
- air-strike orders (S-0519): min 4 median 6 max 15, then min 4 median 6 max 17;
- air strikes called (S-0519): min 4 median 6 max 18, then min 4 median 6 max 19;
- crate orders (S-0520): min 35 median 93 max 150, then min 21 median 91 max 117;
- crates picked up (S-0520): min 9 median 25 max 65, then min 6 median 25 max 62.

The other summary rows are unchanged. "smallest minimum over 16 seeds: 0" holds on base and after.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8; `quick-base-7.txt` to `quick-base-10.txt`, `quick-after-7.txt` to `quick-after-10.txt`). Each seed's file differs from base by `cmp`:
- seed 7: first at byte 366, line 4;
- seed 8: byte 361, line 4;
- seed 9: byte 1358, line 13;
- seed 10: byte 23, line 1.

Their contents were not read: a read of `quick-base-7.txt` was refused by the permission check and not repeated another way. Which games and winners move, and the trace of any winner change, are handed back.

**Suites.** The AI suite with its ignored tests passes 387 on base and after, in 42 result lines (`ai-base.txt`, `ai-after.txt`). The sim gate with its ignored tests passes 17 on both (`gate-base.txt`, `gate-after.txt`), after the goldens were regenerated.

Final checks, on 88e3b8b:
- `cargo test --workspace --no-fail-fast` (`ws-after.txt`): 9409 tests pass, against 9387 on base (`ws-base.txt`). The only failure is the known `quirks::tests::every_spec_quirk_has_a_switch`. It names the same three quirks as on base: `balloon_hover_mobile_fire_by_destination`, `airstrike_line_marker_over_shroud` and `slave_auto_pack_old_top_left`;
- `cargo clippy --workspace --all-targets -- -D warnings` (`clippy.txt` was the first run; it found two `clone_on_copy` in `claims_s0406_r20.rs`, fixed in 88e3b8b; the rerun is clean);
- `cargo fmt --all -- --check`;
- the wasm32 build of glasnost-app (`wasm-after.txt`);
- `cargo xtask check` (`xcheck-after.txt`): fails only in stock-guard. Eight problems are the known ones in M-0027. The ninth is the batch's untracked notes file, which is never staged;
- `cargo xtask gate` (`xgate-after.txt`): ok;
- the AI suite with its ignored tests, on 2646cbf (`ai-final.txt`): 387 pass in 42 result lines.

Seams left:
- Q-3301 (a cell destination under claim 25), Q-3302 (#457: passing damage on to the bunker) and Q-3303 (claim 26 against S-0605 claim 16) wait for answers. Claim 26 is held until Q-3303 is answered.
- The quick lane's moves and any winner trace are handed back.

The P3 census rule: the batch adds no saved field and no object-table reference, so it makes or moves no back-reference of `World::census`.

## Batch S41: the slave miner's pack-up cell, bound slaves, a building's experience, a chrono legionnaire's own grinder and a target taken over (#451, #450, #452, #455, #453)

Base main 43eaa36, not rebased. The batch builds three answers:
- Q-3274's, in S-0402 r90 claim 25 (#451);
- Q-3275's, in S-0301 r78 claim 9 and S-0402 r90 claim 21 (#450);
- Q-3273's, in S-0311 r14 claim 16 (#452).

It also checks two issues against the claims, #455 and #453. Both match them, so each gets a request and guard tests, not a fix.

The work went in this order:
1. The new switch and its schema entry (c263b46).
2. The design text, in *Simulation (sim issues S41 …)* in `docs/dev/simulation.md` (ce8207c).
3. For each item, the failing tests, then the fix. No fix commit changes a test. Item 1's test was corrected once, before its fix (eb5c503, then 950d8a2).
4. The requests, each in its own commit: Q-3297 (f4e92d7) and Q-3298 (ebebce6).
5. A rustfmt commit (4c12521), the format bump (1d11931) and the goldens (b8dd5d4).

Each red was shown in a separate base worktree checked out at the test commit (`red-1.txt`, `red-2.txt`, `red-3.txt`, `red-3b.txt`). Logs are in `local-lane/sim-issues/s41/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #451, the pack-up cell (S-0402 r90 claim 25, Q-3274; H30) | An automatic pack-up remembered the old top-left cell *A* as the guard spot, so a 3 × 3 master's vehicle drove one diagonal cell back and redeployed one cell on, again and again. r90 keeps that as the original and names a switch for it | fixed (c263b46, 69b218e; tests eb5c503 and 950d8a2, red at 950d8a2: guard spot (36, 32) against (37, 33)) | New switch `quirks.slave_auto_pack_old_top_left`, on by default (the original), in the rules schema. `econ::slaves::check_field`: with the switch off the guard spot is the return cell of S-0415 claim 10 (`econ::deploy::return_cell`), where the vehicle appears. So a foundation larger than 2 × 2 behaves as case (a), and its miner reaches the far field. Manual packing is untouched | `claims_s0402_slaves.rs`: `claim_25_issue_451_old_top_left_off_reaches_the_far_field` (red). Guards, 4e2db3a, as built: `claim_25_h30_a_two_by_two_reaches_the_far_field_either_way`, `claim_25_h30_b2_quick_drive_reaches_the_far_field`, `claim_25_h30_c1_no_field_cycle_runs_on`, `claim_25_h30_c2_no_field_packs_up_once` |
| #450, the bound firer (S-0301 r78 claim 9, Q-3275; F23) | A firer bound to a slave master had no *illegal*, so an armed slave shot its master | fixed (4d76f62; tests e65545b, red there) | `combat::check::answer` asks the bound firer's *illegal* after the infantry healer's *illegal* and before the materialising firer's *rearming*. It is asked in every mode, for every target, every slot and no target. A firer is bound while `econ::slaves::is_slave` holds | `s0301_bound_firer_is_illegal_for_every_target_and_slot` (red: *rearming* against *illegal*), `s0301_issue_450_an_armed_slave_never_shoots_its_master` (red: master health 150 against 480), `s0301_freed_slave_is_not_bound` (guard) |
| #450, emergence (S-0402 r90 claim 21, H16, H18) | An emerged slave was given its master as target | fixed (4d76f62; tests e65545b, red there) | `econ::slaves::emerge` gives no target. It sends a forced, move-anywhere scatter request (`scatter::request`) with the master's reference point as threat point, right after the slave's placement draw, then forces *harvest* current. The force drops the queued *move*, and the destination stays | `claim_21_emergence_sends_a_scatter_request`, `claim_21_issue_450_an_emerged_slave_has_no_target`, and `claim_21_building_emergence`, which now expects no target (all red). The `stand_still` helper is set-up only |
| #452, a building's experience (S-0311 r14 claim 16 and Import mapping, Q-3273) | A building type without the trainable key took the shared default (yes), so a tower gained experience and could rank | fixed (2c8f91f; tests 9b8efd6, red there) | `rules::building_rules`: `trainable` is `bt.combat.trainable.unwrap_or(false)`. The kill recipient of claim 8, the veteran crate (S-0520 claim 22) and claim 11 follow from it. No client change | `claims_s0311.rs`: `claim_16_issue_452_building_without_the_key_is_untrainable`, `claim_16_issue_452_armed_building_kills_without_promotion` (both red). `claims_s0311_building_crate.rs`: `claim_16_issue_452_veteran_crate_raises_only_a_trainable_building` (red: 1.0 against 0) |
| #455, a chrono legionnaire and its own grinder (S-0603, S-0213, S-0215, S-0207, S-0208, S-0214) | The claims give the *recycle* cursor (I26) for an armed, self-teleporting infantry type at rest, and also while it phases in. A click while phasing in gives no order, because table M needs *movable*. After phasing, the click gives the eaten order with the own grinder as destination. The landing point is chosen when the order is applied, with *guard* current. No occupant is ignored, and WP19 blocks the foundation, so the nearby search lands the unit beside the building. *Eaten* falls back to rest, and the unit is never consumed | as built; request Q-3297 (f4e92d7) | none | `claims_s0603.rs` (c94e1ad): `claim_1_issue_455_a_self_teleporting_infantry_gets_recycle`, `claims_3_5_issue_455_a_self_teleporting_entrant_lands_beside_and_stays` |
| #453, a target that mind control makes the attacker's own (S-0309 claims 25 k, 29, 30, 31, S-0301 claim 9, S-0601 claim 6) | Claim 30 drops a target only when the holder's own owner changes. Claim 29 drops it only on *illegal* or *impossible*. Claim 25 part k's allied drop covers only the neutral and special houses. So a player's tank keeps attacking a vehicle that its house's controller takes over. That holds whether the attack was ordered or set as a search would set it | as built; request Q-3298 (ebebce6) | none | `claims_s0601.rs` (85bbe36): `s0309_issue_453_a_player_attacker_keeps_a_target_its_house_took_over`. Claim 25 k's neutral drop stays guarded by `claims_s0309_r94.rs` |

**Formats** (1d11931):
- Item 2's new draws and item 3's experience change saved state. Save goes from 162 to 197 and AI game from 140 to 175 on the branch (renumbered to 165 and 143 at the merge).
- The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change.
- Wire stays 13.
- The rules schema gains `slave_auto_pack_old_top_left` (c263b46). With it, the quirk-switch test no longer names it (on base it named it with S40's and R16's switches).

**Goldens** (b8dd5d4): six of the nine sim-gate goldens move: duel, duel-diagonal, duel-naval, duel-m6c, duel-m7a and duel-m7b. Combat, scripted and skirmish stay, and line counts are unchanged.
- At 9b8efd6 (items 1 and 2 in, item 3 not), the golden test with its ignored run passed duel, duel-diagonal, duel-naval and duel-m6c, then failed at duel-m7a ("state hash differs after tick 467"; `golden-at-9b8efd6-full.txt`). So item 2 moves duel-m7a and item 3 moves the four before it.
- The test stops at its first failure, so duel-m7b's mover is not split.

Lanes compare base 43eaa36, built in the base worktree, with the batch at b8dd5d4.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). The failing rows of every measurement stay as base:
- M-0001: 10 of 96;
- M-0002: 4 of 24;
- M-0003: 3 of 35;
- M-0019: 0 of 6;
- M-0020: 0 of 25 checked;
- M-0021: 10 of 50.

Within those rows:
- M-0001's win-test failures go from 3 to 2. LTNK against E1 goes from 4/8/0 to 4/7/1. CAOS against E2 goes from 6/6/0 (p 0.0047) to 4/6/2 (p 0.0285), which now fails on side outcome only.
- M-0002's win-test failures go from 0 to 1. YURI against E1 goes from 5/7/0 (p 0.0120) to 6/6/0 (p 0.0047).
- In M-0019 only the sample counts of infantry pairings change.
- M-0003, M-0020 and M-0021 are identical to base.

Only infantry pairings move, and the arena has no slaves. So these moves are put down to item 3 by elimination. The arena's maps hold the neutral house's buildings, some of them garrisonable. Item 3 changes where a kill made from inside a building puts its experience: claim 8's occupant fallback, as in the quick lane's seed 8 game 4 below. These moves are not traced.

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`, seeds 1 to 16):
- All three pass, before and after.
- M6c prints the same lines as base, apart from build and timing lines.
- M7a: every seed runs 12000 ticks with smallest minimum 0, and every seed's counters move (slave miners: item 2).
- M7b: every seed runs 8000 ticks, with results none.
  - First link, base: 6107 6137 5942 5897 6032 5927 5957 6139 5942 6236 6062 5937 6453 5935 5479 5972.
  - First link, after: 5897 5429 6202 6157 5961 5368 6077 6186 6156 5917 6107 5987 6178 5972 5368 6146.
  - Links ended, base: 0 1 0 0 0 1 0 0 0 0 0 0 0 0 1 0. After: 0 2 0 0 0 1 0 0 0 0 0 0 0 0 4 0.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt`, `quick-after-7.txt` to `quick-after-10.txt`). Each seed's games are numbered 1 to 9: map 0 with sides 0v1, 0v2 and 1v2, then map 2, then map 8. Six games end on the same tick with the same winner as base: seed 7 games 1 and 7, seed 8 games 1 and 7, seed 9 game 1, and seed 10 game 7. 30 games move, and 11 of them change their winner:

| Seed | Game | Base | Batch | First difference | Item |
|---|---|---|---|---|---|
| 7 | 3 | 48653, house 0 | 72781, house 1 | tick 1687, house 1 slaves 1149 to 1151 (placed at tick 1686): target their master on base, none after | 2 |
| 7 | 4 | 44158, house 1 | 56325, house 0 | tick 7998, house 1 buildings 491 and 492 (type 71, health 0): experience 2.0 on base, 0 after | 3 |
| 7 | 8 | 32942, house 1 | 28879, house 0 | tick 2013, house 1 slaves 176 to 178 | 2 |
| 7 | 9 | 36129, house 1 | 46536, house 0 | tick 2012, house 1 slaves 176 to 178 | 2 |
| 8 | 2 | 41018, house 0 | 74028, house 1 | tick 1687, house 1 slaves 1154 to 1156 | 2 |
| 8 | 3 | 113616, house 0 | 37935, house 1 | tick 1687, house 1 slaves 1154 to 1156 | 2 |
| 8 | 4 | 36113, house 0 | 110316, house 1 | tick 8616, house 0 building 610 (type 162): experience 2.0 on base, 0 after; house 0 infantry 710, off the map inside it: 0 on base, above 0 after | 3 |
| 8 | 8 | 34690, house 0 | 31706, house 1 | tick 2198, house 1 slaves 178 to 180 | 2 |
| 8 | 9 | 65194, house 1 | 45735, house 0 | tick 2197, house 1 slaves 178 to 180 | 2 |
| 9 | 6 | 75631, house 1 | 38470, house 0 | tick 1687, house 1 slaves 635 to 637; a house 0 building's last record follows | 2 |
| 10 | 6 | 67566, house 1 | 68233, house 0 | tick 1687, house 1 slaves 637 to 639 | 2 |

Each winner change was traced as a single pair (`GLASNOST_AI_SIDES`):
1. State hashes (`GLASNOST_AI_HASH=1`) of the base binary and the batch binary, capped at 20000 ticks (60000 for the map 2 games of sides 0v1), gave the first differing tick.
2. An object dump of that tick and the one before (`GLASNOST_AI_OBJS`) named the objects.

The pair runs were not played to the end. Nine winner changes first differ at a just-emerged slave of type 61, item 2's emergence. Two first differ at a building's experience, item 3. The large dumps were deleted after reading.

**Suites.** The AI suite with its ignored tests passes 387 on base and after (`ai-base.txt`, `ai-after.txt`). The sim gate with its ignored tests passes 17 on both (`gate-base.txt`, `gate-after.txt`), after the goldens were regenerated.

Final checks on b8dd5d4:
- `cargo test --workspace --no-fail-fast` (`ws-after.txt`): 9402 tests pass, against 9387 on base. The only failure is the known `quirks::tests::every_spec_quirk_has_a_switch`. It now names only `balloon_hover_mobile_fire_by_destination` and `airstrike_line_marker_over_shroud`, other batches' switches, as on base;
- `cargo clippy --workspace --all-targets -- -D warnings` (`final-clippy.txt`);
- `cargo fmt --all -- --check`;
- the wasm32 build of glasnost-app (`final-wasm.txt`);
- `cargo xtask gate` (`final-xgate.txt`): native 17 pass, and all nine goldens match under wasm32.

These ran on dd7a935, the commit that adds this section:
- `cargo xtask check` (`final-xcheck-2.txt`): fails only on the 8 known stock-guard problems in M-0027;
- the AI suite with its ignored tests (`final-ai.txt`): 387 pass.

Seams left:
- Q-3297 (#455: whether a self-teleporting infantry unit's own grinder should take it in) and Q-3298 (#453: whether attackers drop a target that an owner change makes their own) wait for answers.
- Item 2's order of the scatter request and the search, *harvest* forced after the request, is our reading of claim 21's "then" (design text).
- The client: no change. The cursor and the insignia already read the simulation's answers.

The P3 census rule: the batch adds no object-table reference, so it makes or moves no back-reference of `World::census`.

## Batch S43: the jumpjet crash start's answers, a mirage and a defence, a target being erased, one object point, and S42's quick lane (Q-3281, Q-3282, #456, #458)

Base main 3e6489f, not rebased. The batch builds S-0502 r36's answers to S39's two requests, Q-3281 and Q-3282. It checks issues #456 and #458 against the specs, joins the jumpjet's and the hovering vehicle's object-point helpers, and traces S42's quick-lane winner changes. The design text came first, in *Simulation (sim issues S43 …)* in `docs/dev/simulation.md` (46a101c for item 1, 04f2c70 for item 5). Item 1's tests (904a98d) came before its fix (04be785). The fix changes no test. The red was shown in a separate base worktree checked out at 904a98d (`red-1.txt`). Two requests are filed:
- Q-3313 (073f7c2): #456;
- Q-3314 (65018de): #458.

Q-3315 and Q-3316 were not used. The guard tests are cef88eb (#456) and da1d500 (#458). Item 5 is f1b3b21. A rustfmt commit (bb4423e), the format commit (b39e629) and a clippy fix (0cedb16) close the code. Logs are in `local-lane/sim-issues/s43/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-3281 (S-0502 claim 18 r36, *Start*, *Path of the fall*) | `jump::fly::tick` ran the update that enters *crashing* as any other: the preceding state's physics, then that state's own outcome, then `crashing_outcome`. So a unit killed while cruising met claim 11's arrival test or F6's bands against the point the crash start's stop had just moved near it, and its maximum speed fell in the first crash update. r36: only the preceding state's physics runs, then crashing's decisions | fixed (04be785; tests 904a98d, red at 904a98d: 3 tests) | after `physics`, `tick` reads whether this is the entering update (a crash started, the phase not yet *crashing* or *crashed*). If so it skips the state's outcome and goes straight to `crashing_outcome`. Claim 29's registration move still follows for an update that started *ascending* or *cruising*. Physics is unchanged | `claims_s0502_r36.rs`: `j12_a_r36_the_maximum_speed_is_kept_through_the_first_crash_update`, `j12_a_r36_killed_near_the_cell_centre`, `j12_a_r36_killed_in_a_speed_band` (all three red at 904a98d); `j12_e_r36_killed_ascending_before_its_maximum_speed` (a guard, passes at 904a98d) |
| Q-3282 (S-0502 claim 18 r36, *Destination clear at crash start*; Edge cases, *No landing slot*) | `jump::stop_with` gave a unit whose nearby-cell search found no cell the no-cell outcome whatever its health. With `quirks.jumpjet_no_cell_self_destruct` on, the self-destruction was skipped for a unit with no health, but its movement point and order were still cleared. r36: both settings' outcomes act only on a unit that still has health | fixed (04be785; tests 904a98d, red at 904a98d: 1 test) | `stop_with` returns after a failed search when the unit's health is 0 or less. The return comes after the magnetic-release branch (S-0607 claim 13's departure, unchanged). S40's `jump::start_crash` still ends the attached animations: `fly::start_crash` returns `true` after `crash_start_stop` whatever the stop did (`claims_s0109_r45_attachment_end.rs` `a_jumpjet_crash_ends_it` passes) | `claims_s0502_r36.rs`: `claim18_r36_a_killed_unit_with_no_cell_keeps_its_flight`, both settings (red at 904a98d); `claim18_r36_a_crash_without_a_kill_still_takes_the_switch_off_outcome` (a guard, passes at 904a98d) |
| #456 (S-0516 claims 33 to 36 and 39, S-0309 claim 12 table C and claim 25 (g)) | A defence of a player house whose type cannot detect disguises does not acquire, or retaliate against, a still, disguised vehicle, also while the vehicle fires. Claim 39's fake blink does not remove the disguise, and claim 25 (g)'s chance is for the computer-owner houses only. The engine follows the specs. The lead's report that defences should fire back is a spec question | as built; guard tests; request Q-3313 (073f7c2) | none | `claims_s0516_issue_456.rs`: `issue_456_a_still_disguised_mirage_is_not_picked`, `issue_456_a_moving_mirage_is_undisguised_and_picked_on_every_moving_tick`, `issue_456_after_a_damaging_hit_it_is_picked_until_it_disguises_again`, `issue_456_the_fake_blink_does_not_let_a_human_defence_pick_it`, `issue_456_a_just_stopped_mirage_follows_the_predicate` (all pass with no code change; the last prints that the first still state between ticks is already disguised, which Q-3313 reports) |
| #458 (S-0301 claim 9, S-0309 claim 29, S-0302 claim 8 stage 1, S-0514) | S-0301 claim 9 leaves open what the firing check answers for a target that another firer's temporal beam is erasing. S-0309 claim 29's check drops a target only on *illegal* or *impossible*, so other firers keep the target and fire, and their shots do no damage. The engine follows the specs | as built; guard test; request Q-3314 (65018de) | none | `claims_s0301_issue_458.rs`: `issue_458_another_firer_keeps_an_erased_target_and_its_shots_do_nothing` (passes) |
| S-0607 claim 7 (one object point; S-0502 claim 16, S-0503 claim 20) | `jump::object_point` (S40) gave its own position for an object riding in any transport. `hover::object_point` (S42) gave it only for a passenger of an open-topped transport, which is what claim 7 names. The two differ only for an object riding in a closed transport. By the design's argument, no such object reaches either helper: boarding makes the passenger go inside before it is marked riding, S-0507 claim 6a then takes it from every holder's destination, and the orders' target test refuses an object off the map | joined (f1b3b21) | `lift::destination_point` gives both, with the open-topped test. `jump::object_point` and `hover::object_point` call it | no new test; the sim suite passes unchanged (`sim-fix5.txt`) |

Switch: none new. The rules quirk test lists only S-0402's `slave_auto_pack_old_top_left`, on base and after (`ws-base.txt`, `ws-after.txt`).

Formats (b39e629): no saved field is added, but item 1 changes the replays. So save goes from 164 to 203 and AI game from 142 to 181, the numbers the main session gave this batch (renumbered to 166 and 145 at the merge). The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change. Wire stays 13: no order's encoding changes.

Goldens: none moves (`golden-regen.txt`; `gate-i1.txt` passes 17 after item 1).

Lanes compare base 3e6489f with the batch at b39e629. Seed 7's quick-lane run is at bb4423e, which differs from b39e629 only in the format numbers. The clippy fix 0cedb16 only adds an attribute to a test, so it cannot change any result.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). The text outputs differ only in timings and output paths. `ours-M-0001.csv`, `ours-M-0002.csv` and `ours-M-0003.csv` are identical to base. These rows fail, on base and after:
- M-0001: 10 of 96 (`arena-after.txt` line 102);
- M-0002: 4 of 24 (line 146);
- M-0003: 3 of 35 (line 194);
- M-0019: 0 of 6 (line 210);
- M-0021: 10 of 50 (line 269);
- M-0020: 0 of 25 checked, 28 rows (line 300).

**Sweeps** (`m6c-*`, `m7a-*`, `m7b-*`). M6c, M7a and M7b over seeds 1 to 16 pass, before and after (`m6c-after.txt` line 53, `m7a-after.txt` line 67, `m7b-after.txt` line 58). Each differs from base only in build lines and timings: no seed line or summary row moves.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-7.txt` to `quick-base-10.txt`, `quick-after-7.txt` to `quick-after-10.txt`). Each seed's games are numbered 1 to 9: map 0 with sides 0v1, 0v2 and 1v2, then map 2, then map 8. 33 games end on the same tick with the same winner as base. Three move, and two of them change winner:

| Seed | Game | Base ticks (winner) | Batch ticks (winner) | First difference | Cause |
|---|---|---|---|---|---|
| 7 | 3 (map 0, sides 1, 2) | 74473 (house 0) | 105981 (house 1) | tick 21835: a house 0 jumpjet vehicle in its first crash update. On base its maximum speed (`jump.vm`) falls and it starts a new heading turn; in the batch both keep their values | item 1 (Q-3281); winner |
| 8 | 8 (map 8, sides 0, 2) | 27417 (house 0) | 82253 (house 0) | not traced | — |
| 10 | 5 (map 2, sides 0, 2) | 48492 (house 1) | 54848 (house 0) | tick 8475: a house 0 jumpjet infantry unit in its first crash update. Same fields as seed 7 game 3: on base `jump.vm` falls and a heading turn starts; in the batch neither | item 1 (Q-3281); winner |

Each winner change was traced as a single pair (`GLASNOST_AI_SIDES`), which reproduced the full set's result. The base binary was built in the base worktree at 904a98d (base plus item 1's tests only). State hashes every 10 ticks (`GLASNOST_AI_HASH=10,0,0`; `tr-s7g3-hash-*`, `tr-s10g5-hash-*`) found the first differing 10-tick window. A dump of the objects in it (`GLASNOST_AI_OBJS`; `tr-s7g3-objs-diff.txt`, `tr-s10g5-objs-diff.txt`) found the first differing tick. At that tick one object differs, and only in those fields. No item-1-only build was made: the attribution to item 1 is read from the fields. Item 5 is designed to change no behaviour. The length change in seed 8 game 8 was not traced, so it is not known which item moves it.

**S42's AI quick lane** (item 4 of this batch). S42's section above hands back its quick lane: its files differed from base, but their contents were not read. This batch built S42's base 43eaa36 and S42's head 2cd701a in two more worktrees and ran seeds 7 to 14, maps 0, 2 and 8 (`s42base-quick-7.txt` to `s42base-quick-14.txt`, `s42head-quick-7.txt` to `s42head-quick-14.txt`). Most games move. Eleven of the 72 change winner. Each was traced as above: a single pair, state hashes every 10 ticks, then an object dump of the first differing window (`s42tr-s<seed>g<game>-hash-*`, `-objs-diff.txt`). The two S42 binaries' hashes are equal for a long prefix in every game. At the first differing tick one object differs:

| Seed | Game | S42 base ticks (winner) | S42 head ticks (winner) | First difference | Cause |
|---|---|---|---|---|---|
| 8 | 3 (map 0, sides 1, 2) | 113616 (house 0) | 90418 (house 1) | tick 55868: a house 1 hovering vehicle moving to a mobile object; only `hover.taken_point` differs | Q-3263 (S-0503 claim 20 r30: a mobile object's heading point) |
| 8 | 9 (map 8, sides 1, 2) | 65194 (house 1) | 62526 (house 0) | tick 43774: a house 0 walker in Capture with an object point. On base its destination is cleared and it has no path; on head the destination is kept, with a path and a walk target | Q-3262 (S-0208 claim 25 r68) |
| 9 | 7 (map 8, sides 0, 1) | 84628 (house 0) | not over at 120000, no winner | tick 66225: a house 1 walker in Capture; the same fields as seed 8 game 9 | Q-3262 |
| 10 | 9 (map 8, sides 1, 2) | 39761 (house 0) | 38592 (house 1) | tick 24477: a house 0 walker. On base its Capture has ended and it guards, with no destination and a retry; on head Capture is kept, with the destination, a path and a walk target | Q-3262 |
| 11 | 7 (map 8, sides 0, 1) | 49144 (house 0) | not over at 120000, no winner | tick 32896: a house 0 hovering vehicle in contact with a depot, with *enter* queued; only its path and the path's goal cell differ | Q-3263 (a depot's docking point) |
| 12 | 6 (map 2, sides 1, 2) | 78849 (house 0) | 63570 (house 1) | tick 33718: a house 0 walker in Capture; the same fields as seed 8 game 9 | Q-3262 |
| 12 | 7 (map 8, sides 0, 1) | 63140 (house 1) | 49960 (house 0) | tick 32494: a house 1 walker in Capture; the same fields as seed 8 game 9 | Q-3262 |
| 13 | 1 (map 0, sides 0, 1) | 108299 (house 0) | 68463 (house 1) | tick 35133: a house 0 walker sabotaging, with no destination on either; only the path's retry tick and retry object differ | Q-3262 (a search for the point's cell) |
| 13 | 3 (map 0, sides 1, 2) | 108343 (house 0) | not over at 120000, no winner | tick 65122: a house 0 walker entering an object, with no destination on either; the same fields as seed 13 game 1 | Q-3262 |
| 14 | 6 (map 2, sides 1, 2) | 91491 (house 0) | 79151 (house 1) | tick 39354: a house 1 walker sabotaging. On base its destination is cleared and its path is for its own cell; on head the destination is kept and the path is for the point's cell | Q-3262 |
| 14 | 7 (map 8, sides 0, 1) | 51508 (house 0) | 64641 (house 1) | tick 37511: a house 0 walker in Capture; the same fields as seed 8 game 9 | Q-3262 |

Two winner changes come from Q-3263's hover object point and nine from Q-3262's walker object point. None comes from Q-3261 or Q-3264. No check build isolated one answer: each attribution is read from the differing fields. Seed 14 game 6 is the clearest, since its path is for the point's cell.

**Suites.** The AI suite with its ignored tests passes 387 on base and after, in 42 result lines (`ai-base.txt`, `ai-after.txt`). The sim gate with its ignored tests passes 17 on both (`gate-base.txt`, `gate-after.txt`), with no golden regenerated.

Final checks:
- `cargo test --workspace --no-fail-fast`, on b39e629 (`ws-after.txt`): 9461 tests pass, against 9449 on base (`ws-base.txt`). The 12 new tests are 6 for r36, 5 for #456 and 1 for #458. The only failure is the known `quirks::tests::every_spec_quirk_has_a_switch`, which names only `slave_auto_pack_old_top_left`, as on base;
- `cargo clippy --workspace --all-targets -- -D warnings`: the first run (`clippy.txt`) found `eprintln!` in the #456 test, a disallowed macro. The fix (0cedb16) is an allow attribute on that test, as in `claims_s0309_r129.rs`. The rerun on 0cedb16 is clean (`clippy-2.txt`);
- `cargo fmt --all -- --check`, on 0cedb16;
- the wasm32 build of glasnost-app, on 0cedb16 (`wasm-after.txt`);
- `cargo xtask check`, on 0cedb16 (`xcheck-after.txt`): fails only on the 8 known stock-guard problems in M-0027;
- `cargo xtask gate`, on 0cedb16 (`xgate-after.txt`): ok, and all nine goldens match under wasm32;
- the AI suite with its ignored tests, on the final commit 7ee88ba (`ai-final.txt`): 387 pass in 42 result lines, 0 failed.

Seams left:
- Q-3313 (#456: retaliation and acquisition against a disguised vehicle that fires) and Q-3314 (#458: the firing check for a target another firer is erasing) wait for answers.
- The length change in seed 8 game 8 of this batch's quick lane is not traced.

The P3 census rule: the batch adds no saved field and no object-table reference, so it makes or moves no back-reference of `World::census`.

## Batch S44: a teleporting entrant of a recycling building, every holder of a target that changes owner, a stolen vehicle and a bunkered vehicle's damage (Q-3297, #455; Q-3298, #453; Q-3302, #457)

Base main 348c3d7, not rebased. The batch builds three answers:
- Q-3297's, in S-0603 r15 claim 16 (#455);
- Q-3298's, in S-0309 r149 claim 32, its *A stolen vehicle* bullet included (#453);
- Q-3302's, in S-0302 r51 claim 8, *A bunkered unit and its bunker in one detonation* (#457). The engine already matched it, so this item is guard tests.

Q-3301 and Q-3303 (S-0208 r75) were handed back untouched: S42's interim rules stand. The AI B10 counters line for `skirmish_lane` (item 5) was not started.

The work went in this order:
1. The requests Q-3317 and Q-3318 (4f5bfbc).
2. For each item, the failing tests, then the fix. No fix commit changes a test. Item 1's tests carry the new switch and its schema entry (fa2dd8e).
3. A rustfmt commit (1e0185a), the format bump (0080f20) and the goldens (e474db6).

Each red was shown in a separate base worktree checked out at the test commit (`item1-red-base-2.txt`, `item2-red-base.txt`, `item2t-red-base.txt`). Logs are in `local-lane/sim-issues/s44/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| #455, a teleporting entrant of a recycling building (S-0603 r15 claim 16, Q-3297) | A self-teleporting infantry unit sent into its own grinder at rest landed beside it in *eaten* and was never consumed. r15: its landing-point choice is a start, so a ready unit's queued *eaten* is current there, row WP3 lands it on the foundation, and the jump tick consumes it. The fate route assigns the destination before it queues *eaten*, so it lands beside. A vehicle's first order lands beside, a second consumes it | fixed (fbd1383; tests fa2dd8e, red there: 3 failed of 31) | `teleport::take_destination_with` makes a ready infantry unit's queued behaviour current before the landing point is chosen (`landing_start`). `jump()` runs `grinder::jump_arrival` after the kind's arrival effects and returns after the second warp animation when it consumes. `grinder_fate` assigns the destination, then queues *eaten*. New switch `quirks.teleport_vehicle_recycle_lands_beside` (on, the original): off, a vehicle's first order asks row VP6 as if *eaten* were current. The landing point stays the building's position (Q-3317); the occupant-ignoring list stays S-0208 r75's (Q-3318) | `claims_s0603.rs`: claim 16 (b), (c), (e) both switch positions (red); guards for the invulnerable building (b, d) and the fate route (f); claim 1's cursor guard recited to r15 |
| #453, every holder of a target that changes owner (S-0309 r149 claim 32, Q-3298) | Claim 30 dropped a target only at the holder's own owner change, so a tank kept attacking a vehicle its house's controller took over. r149: every holder of an object whose owner changes drops it, of whatever house and however it came to hold it | fixed (dbdb77b; tests b2745a8, red there: r149 5 failed of 7, s0601 5 failed of 64) | `combat::held::owner_change_drop`, called from `econ::events::owner_changed` right after claim 30's own drop when the owner really changes. An overridden holder reverts first (S-0215 claim 10) and drops the object only if the revert left it as its target. No timer draw; behaviours, destination, put-aside slots and attack-move memories kept; a living strike target kept | `claims_s0309_r149.rs` (new): every holder kind and house, an engineer capture, an overridden holder's three cases, put-aside, attack-move memory, strike target, no change. `claims_s0601.rs`: the `s0309_issue_453_*` tests (ordered, searched and force-fired targets; an ally's takeover; a computer-owned holder; the controller). `claims_s0309_r94.rs` recites claim 25 k |
| #453, a stolen vehicle (S-0309 r149 claim 32, *A stolen vehicle*) | A vehicle thief's takeover changed the owner with no cloak-style drop first | fixed (c323ed7; test 1069cfd, red there: timer 30 ticks left, not 4 to 8) | `crush.rs`: after the thief is removed and before `mind::set_owner`, `cloak::machine::drop_attackers` (now `pub(crate)`) runs claim 28 with claim 27's draw, the revert and claim 27a's put-aside loss. Holders that sense the cell or have the old owner keep it there, and the owner change drops it for them with no draw | `claims_s0210.rs`: `a_stolen_vehicle_is_first_dropped_as_at_the_start_of_cloaking` (red) |
| #457, a bunkered vehicle and its bunker in one detonation (S-0302 r51 claim 8, Q-3302) | The engine matches r51 | as built; guard tests (2040fce) | none | `claims_issue_457.rs`: a non-penetrating detonation (spread 1 and 0, centred on the vehicle and on the bunker's centre) deals the bunker exactly its empty-bunker hits; a penetrating one hurts the vehicle as outside and not the bunker; a penetrating killing hit leaves the bunker empty for its later hits; a curtain on the vehicle nullifies a small detonation the empty bunker would not; real shots (invisible and visible projectiles) follow the two rows. S42's five tests recite r51, and the "asked in Q-3302" wording is gone |
| Q-3301, Q-3303 (S-0208 r75 claims 25 and 26) | Not started | handed back untouched | none: S42's interim rules stand (pre-r68 cell destinations; claim 26's substitute replaces only a cell destination, D3) | none. `quirks::tests::every_spec_quirk_has_a_switch` still names `walker_search_ignores_queued_enter` (r75's new switch), as on base |

**Formats** (0080f20): save goes from 165 to 205 and AI game from 143 to 183, the brief's numbers (renumbered to 167 and 146 at the merge). The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change. Wire stays 13. The rules schema gains `teleport_vehicle_recycle_lands_beside` (fa2dd8e).

**Goldens** (e474db6): three of the nine sim-gate goldens move: duel-m6c, duel-m7a and duel-m7b. Combat, duel, duel-diagonal, duel-naval, scripted and skirmish stay. The first differing lines were not read (see the hand-back's Guard section).

Lanes compare base 348c3d7, run in this worktree before any change, with the batch at e474db6.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). The failing rows of every measurement stay as base: M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 checked, M-0021 10 of 50. M-0002's win-test failures go from 1 to 0; M-0001's stay 2.

**Sweeps** (seeds 1 to 16): m6c's smallest minimum stays 0 (charger orders and support beams, as base); m7a's goes from 0 to 2 (the bunker entry, bunkered and bunker exit counters' minimum 0 to 2); m7b's drain minimum stays 3. All three pass.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-<seed>.txt`, `quick-after-<seed>.txt`). 35 games end on another tick and 10 change winner: seed 7 games 5 and 8; seed 8 games 1 (to no winner at 120000), 4 and 6; seed 9 games 5, 6, 7 and 9; seed 10 game 9. They are not traced: the trace's first read was refused by the permission check, and it was not retried another way (hand-back, Guard section).

**Suites.** The sim gate with its ignored tests passes 17 on base and after (`gate-base.txt`, `gate-after.txt`), after the goldens were regenerated. The final checks run on the commit that adds this section and are in the hand-back.

Seams left:
- Q-3317 (a teleporting entrant's destination point of a building) and Q-3318 (the occupant-ignoring list) wait for answers.
- Q-3301 and Q-3303's r75 work (item 4) and the B10 counters line (item 5) are for a later batch.
- The quick lane's 10 winner changes are untraced (traced in batch S45, below).

The P3 census rule: the batch adds no object-table reference, so it makes or moves no back-reference of `World::census`.

## Batch S45: a walker's failed search and its substitute cell, a target being erased, a disguised attacker, S44's quick-lane trace and the B10 counters line (Q-3301, Q-3303; Q-3314, #458; Q-3313, #456)

Base main d9f2750, not rebased. The batch builds three answers:
- Q-3301's and Q-3303's, in S-0208 r75 claims 25 and 26;
- Q-3314's, in S-0301 r80 claim 9, *Targets being erased* within rank 4 (#458);
- Q-3313's, in S-0516 r29 claim 44 (#456). The engine already matched it, so this item is guard tests.

It also traces S44's ten quick-lane winner changes (item 4) and adds the AI B10 counters to `skirmish_lane`'s special abilities line (item 5).

The work went in this order:
1. Item 1's design (3b39fa3), the requests Q-3329 and Q-3330 (4aea11e), its tests with the new switch and its schema entry (95e394f), the fix with the format bump (43bdce5), a rustfmt commit (e7535f8) and the seams' notes (870d3e8).
2. Item 2's design (ac7f0a2), tests (450c0c4) and fix (ad78967).
3. Item 3's design (7f17422) and guard tests (12d5c00), then a rustfmt commit (2cac663).
4. The goldens (a22ab6a) and item 5 (fd848c6).

No fix commit changes a test. Each red was shown in a separate base worktree checked out at the test commit (`item1-red-base.txt`, `item2-red-base.txt`). Logs are in `local-lane/sim-issues/s45/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-3301, a walker's failed search near any destination (S-0208 r75 claim 25) | S42's interim rule applied claim 25's clearing and loss of the locomotor destination only to an object destination, and a standing walker given its own cell finished at once. r75: the clearing near the locomotor destination holds for every kind of destination; after a failed fresh search past the gate that does not clear, the walker keeps its destination with no locomotor destination and speed fraction 0, and searches no more until an assignment; no own-cell shortcut; a walker in *move* that keeps its destination and is not moving is given it again at the end of its update (S-0215 claim 22, S-0310) | fixed (43bdce5; tests 95e394f, red there: 17 failed in 5 targets: r75 11 of 13, own-cell deck 1 of 2, S-0209 1 of 70, S-0212 r97 3 of 16, S-0605 1 of 48) | `walk::locomotor_destination` (an object's point, the kept point, or a cell's centre at claim 18's height) feeds `cleared_near_point`; `locomotor_destination_lost` marks the kept destination in the new saved field `WalkState::lost` and sets the speed fraction to 0; every assignment clears the mark (`take_destination`). The own-cell shortcut in `standing` is gone. `process::end_of_update_check` gives a walker in *move* that is not moving its destination again (stored path cleared, speed fraction 1). C7-P6's hold keeps its destination in the new saved field `PathState::held_for`, so the hold is not ended by that assignment; an assignment and a path found clear it | `claims_s0208_r75.rs` (new): the clearing for a cell, an object and a kept destination, the lost locomotor destination, no search after R, speed fraction 0, the end-of-update assignment, no own-cell shortcut (red). Moved to r75: `claims_s0208_own_cell_deck.rs`'s control, `claims_s0209.rs`'s `kept_after_failure` (seam C5 ignored for Q-3329), `claims_s0212_r87.rs` claim 23 row 3, `claims_s0212_r97.rs`'s kept-destination tests and P25 part e |
| Q-3303, claim 26's substitute and a queued *enter* (S-0208 r75 claim 26) | S42's interim rule replaced only a cell destination with the substitute. r75: the substitute cell is assigned as the destination for every kind; an engineer-type walker's mission target is read in *guard*, *area guard* and *patrol*; a queued *enter* behind a current behaviour is not read (new switch) | fixed (43bdce5; tests 95e394f, counted above) | `path::compute_once` assigns the substitute with `econ::assign_destination` for a walker (other locomotors keep D3's replacement of a cell destination). `occupancy::mission_target` gives an engineer-type infantry unit its destination in *guard*, *area guard* and *patrol* and, with `quirks.walker_search_ignores_queued_enter` off, the queued *enter*'s. New switch `quirks.walker_search_ignores_queued_enter` (on, r75's rule) | `claims_s0208_r75.rs`: the substitute assigned for a cell, an object and a kept destination, the engineer's mission target in the three behaviours, both switch positions. `claims_s0605.rs` claim 16 in both switch positions (the area-guard case ignored for Q-3330) |
| #458, firing at a target another firer is erasing (S-0301 r80 claim 9, Q-3314) | The firing check answered as for any target, so a second firer shot at, or kept, an object a temporal weapon was erasing. r80: within rank 4 the check answers illegal for an object target being erased when the weapon's warhead is not temporal | fixed (ad78967; tests 450c0c4, red there: #458 1 failed of 1, r80 6 failed of 7) | `combat::check` answers illegal for a target being erased (a lock that is not suspended, or a warp member's leaving stage) when the warhead is not temporal, right after the warp-pending illegal. The drops come from the existing illegal rules (S-0309 claim 29, S-0215 claim 32, table C's firing-check row) | `claims_s0301_r80.rs` (new): every target kind and the illegal-only test; a cell target unchanged; temporal firers (the holder's rearming, a second firer joining, F24 e); the rank against the rearm countdown, ammunition and a building's delayed shot; F24 a, b, c, d and f. `claims_s0301_issue_458.rs` rewritten to r80 |
| #456, answering a disguised attacker (S-0516 r29 claim 44, Q-3313) | The engine matches r29: retaliation tests the disguise at the hit with no draw, searches reject the attacker, a held target is kept, a hit on it ends the disguise, and claim 33 runs after the movement and before the firing step | as built; guard tests (12d5c00) | none | `claims_s0516_issue_456.rs`: claim 44 (a) no retaliation against an attacker that appears disguised at the hit, blink on and off, for a human, a test-only, the neutral and the special victim house, with an undisguised control; (c) a holder that took the vehicle while it moved keeps it through two check ticks after it disguises again; (d) after a hit on it, its hits are answered; (e) a stop order on a track disguises it at the first still tick end. S43's tests recite claim 44; the request wording and the printed observation are gone. A stop with no heading point was tried and dropped: the move and stop issued between the same ticks begin a track anyway, so the case is not played |
| Item 4, S44's quick-lane winner changes | Traced by state hash every 10 ticks, then object dumps, S44 base 348c3d7 against S44 head c165844 | traced | none | none. Nine of ten come from #453's owner-change drop (dbdb77b): seed 7 game 5 (tick 13121), seed 9 game 5 (14088) and seed 9 game 6 (12302) are direct drops of an object that became the holder's own house's; seed 7 game 8 (5809), seed 8 games 1 (7129) and 6 (4674), seed 9 games 7 (6597) and 9 (6002) and seed 10 game 9 (5807) are attack-moving holders that revert to *move* when their house takes over the building they attack. Seed 8 game 4 (24320) comes from #455's landing-point start (fbd1383): a fresh self-teleporting unit's queued *move* is made current |
| Item 5, the B10 counters line | `skirmish_lane`'s special abilities line had no B10 counters | done (fd848c6) | the line gains `disguises`, `spy detours`, `bomber rounds` and `bomber retreats` per house (`special::Uses`), printed by the `ai` and `ai-sides` commands | none (tools only) |

**Formats** (43bdce5): save goes from 167 to 209 and AI game from 146 to 187, the brief's numbers (renumbered to 168 and 148 at the merge). The AI game bump in `crates/deterministic/ai/src/game.rs` is forced by the sim change. Wire stays 13. The rules schema gains `walker_search_ignores_queued_enter` (95e394f).

**Goldens** (a22ab6a): all nine sim-gate goldens move on every hash line, header lines unchanged: the state hash covers the two new saved fields, so each line differs from the first tick.

Lanes compare base d9f2750, run in this worktree before any change, with the batch at fd848c6.

**Oracle arena** (`--record all`, two threads; `arena-base.txt`, `arena-after.txt`). The failing rows of every measurement stay as base: M-0001 10 of 96 (win test 2), M-0002 4 of 24 (win test 0), M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 checked, M-0021 10 of 50.

**Sweeps** (seeds 1 to 16): m6c's smallest minimum stays 0, m7a's stays 2 (seed 1's counters move) and m7b's drain minimum stays 3. All three pass.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-<seed>.txt`, `quick-after-<seed>.txt`). 24 games end on another tick and 5 change winner: seed 7 games 2 (house 0 to house 1) and 9 (1 to 0); seed 8 game 8 (1 to 0); seed 9 games 5 (1 to 0) and 6 (no winner at 120000 to house 1). Each was run again with a build of 450c0c4 (item 1's fix without item 2's) and compared by state hash with the head: all five changes come from item 1. Item 2 first acts in seed 7 game 2 (ticks 28231 to 28240; the end moves from 82280 to 71865, the winner stays house 1) and seed 9 game 5 (ticks 51581 to 51590, the end unchanged), and not in the other three. Item 1's first effect cannot be found by hash, which covers its new fields from tick 0. For seed 7 game 9, the 10-tick army timeline of d9f2750 and the head first differs in ticks 21561 to 21570, and an object dump with the new fields' text removed shows a behaviour delay chosen at tick 21491 already differing (78 against 80, building 126). It was not bisected further.

**Suites.** The AI suite with its ignored tests passes 401 on base and after (`ai-base.txt`, `ai-after.txt`); the sim gate with its ignored tests passes 17 on base and after (`gate-base.txt`, `gate-after.txt`), after the goldens were regenerated. The final checks run on the commit that adds this section and are in the hand-back.

Seams left:
- Q-3329 (whether claim 25's end-of-update assignment in *move* restarts S-0209 claim 10's blockage delay; S-0209 seam C5 ignored) and Q-3330 (a walker in *area guard* that reaches its queued *enter*'s building with `quirks.walker_search_ignores_queued_enter` off; the S-0605 claim 16 case ignored) wait for answers.
- C7-P6's `held_for` (our design) has no scene test of its own.
- The first tick of item 1's effect in the quick lane is not located.

The P3 census rule: the batch adds two saved fields that can name an object, `PathState::held_for` and `WalkState::lost` (each an `Option<Target>`, as `failed_for` is). Neither is a kind of reference the census notes (aircraft, spawn controllers, bombs, chargers, droppers), so the batch makes or moves no back-reference of `World::census`.

## Batch S46: the point a building gives a teleporting unit, the occupant-ignoring behaviours, the held walker and S45's first quick-lane tick (Q-3317, Q-3318)

Base main 30f2165, not rebased. The batch builds two answers in S-0208 r82:
- Q-3317's, in claim 27 and W22: a building gives a teleporting unit its centre, a docking type its docking point;
- Q-3318's, in claim 19: the behaviours whose landing ignores occupants are *capture*, *enter*, *eaten* and *patrol*; *sabotage* is not one of them.

It also gives S45's `held_for` seam a scene test (item 3), and finds the first tick at which S45's item 1 parts quick-lane game s7g9 (item 4).

The work went in this order:
1. The design (a5340f9).
2. Item 1's tests (24e8d8f) and fix with the format bump (166a8e3).
3. Item 2's tests (4bc532c) and fix (73b13e1).
4. Item 3's test (474d638) and its design note (50738bc), then a rustfmt commit (42d14b6).

No fix commit changes a test. Each red was shown in a separate base worktree checked out at the test commit (`t1-red-base.txt`, `t2-red-base.txt`). Logs are in `local-lane/sim-issues/s46/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-3317, a vehicle's destination cell at a building (S-0208 r82 claims 19 and 27, W22) | A teleporting vehicle sent to a building took the building's position, the centre of its top-left cell, as its destination cell. r82: a building gives its centre, (W−1, H−1) half cells from that position, or the docking point for the helipad, unit-repair and vehicle-bunker flags; the unit-reload flag alone gives the centre | fixed (166a8e3; tests 24e8d8f, red there: 6 of 7 W22 tests and `claims_s0603.rs` claim 16 b/c; the far-place test passes before the fix too, as its rings meet) | `teleport.rs`'s vehicle branch takes `lift::lifter_point(w, building, unit)`'s cell: the docking point for a docking type (the refinery, the vehicle bunker's quadrant, the docking-place rule), the reference point otherwise | `claims_s0208_r82.rs` (new): W22 a (infantry, the centre's cell), b (a vehicle searches round and lands in the centre's cell, 1×1 to 5×3 grinders), c (a vehicle bunker's quadrant cell), d (an invulnerable building, the search centred on the centre), e (an engineer), f (the unit-reload flag alone gives the centre; a far place). `claims_s0603.rs` claim 16 asserts the centre's cell |
| Q-3317, an infantry unit's point and the search centre (claims 19 and 27) | An infantry unit's P was the building's position, and claim 19's search round a building centred on the cell the unit heads for | fixed (166a8e3, counted above) | P for a building target is `lift::lifter_point`; the search centre for a building is the cell of `World::reference_point` | as above (W22 a, d, e and f) |
| Q-3317, walking and driving (claim 27) | A walker's object point (`walk::take_destination`) and a driver's heading point (`path::heading_point`) still give a building's position | seam, left | none | none |
| Q-3318, the occupant-ignoring behaviours (S-0208 r82 claim 19) | Our list was *capture*, *enter*, *eaten* and *sabotage*. r82: *capture*, *enter*, *eaten* and *patrol* | fixed (73b13e1; tests 4bc532c, red there: the 5 new tests) | `teleport.rs`'s list is `Capture \| Enter \| Eaten \| Patrol` | `claims_s0208_r82.rs`: a saboteur respects a building's occupants and a vehicle's cell (a neighbour, one draw); patrol ignores occupants for a building and a vehicle, and in the second choice (no draw). `claims_s0208_teleport.rs`'s enter test recites r82 claim 19 |
| Item 3, a held walker (C7-P6's `held_for`, S45's seam) | No scene test showed that a held walker waits | tested; no fault | none | `pathing_c7p4.rs::hold_off_a_held_walker_waits_for_its_retry_delay`: an unarmed walker meets moving enemy vehicles in a hall one cell wide (S-0209 claim 4 row WP22 blocks their cells), so its search fails and it is held. Until its retry tick it makes no search, and it keeps its retry tick, destination, *move* and the mark at each update's end. An assignment ends the hold, and so does a found path, which comes at the retry tick once the vehicles are removed. For an armed walker, rows WP16 to WP18 and WP26 give no *blocked*. Nor does WP28 while `quirks.walker_attacks_unhittable_obstacle` is on (its default). With it off (the play-fixes layer), an enemy the walker cannot hit is blocked. A probe of armed infantry scenes (allied units only) held no walker in 3000 ticks |
| Item 4, S45's first quick-lane tick (s7g9) | S45 found that its item 1 parts seed 7's game 9 (map 8, sides 1 and 2) by tick 21491 | traced, 5 steps | none | none. Builds of d9f2750 and 450c0c4 were compared with `GLASNOST_AI_OBJS` dumps with `, held_for: None` and `, lost: None` removed. At 10745 they differ, at 5372 they do not, and a dump of 9300 to 9320 gives the first differing tick, 9316. There, only infantry 573's `retry_at` (0 against 9315) and `retry_for` differ: a failed search in tick 9315 started a retry delay that its destination's clearing ended at once. That state is dormant. The retry-delay code (`path::failed`'s `start_delay`) is the same in both builds, so the difference is whether that search ran and failed. The r75 rule behind it is not identified. With those two fields removed too, 10030 differs, and a dump of 9316 to 10030 gives the first difference outside the retry fields, tick 9583, infantry 613: its `WalkState::dest_deck` reads (52,79) against (51,78). In tick 9582 its fresh search adopted S-0212 claim 24's substitute (51,78) for (52,79). d9f2750 writes the destination directly. 450c0c4 assigns it (S-0208 r75 claim 26: the substitute is assigned as the walker's destination), so `walk::take_destination` records the new cell. At 10030 and 10745, 613's `dest_deck` and 573's retry fields are still the only differences: at 10745 every object's position and behaviour is the same in both builds, so what parts the game most likely lies in (10745, 21491] |

**Formats** (166a8e3): save goes from 168 to 215 and AI game from 148 to 195, the brief's numbers (renumbered to 169 and 149 at the merge). The AI game bump is forced by the sim change. Wire stays 13. The rules schema does not change.

**Goldens**: regenerated, and none moves. The fix changes no gate scenario.

Lanes compare base 30f2165, run in this worktree before any change, with the batch at 42d14b6.

**Oracle arena** (`--record all`, two threads): output and record files are as base. M-0001 10 of 96 (win test 2), M-0002 4 of 24 (win test 0), M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 checked, M-0021 10 of 50.

**Sweeps** (seeds 1 to 16): m6c, m7a and m7b print the same as base. The smallest minima stay 0, 2 and drain 3, and all three pass.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games): every game ends on the same tick with the same winner.

**Suites.** The AI suite with its ignored tests passes 403 on base and after. The sim gate with its ignored tests passes 17 on base and after. The final checks run on the batch's last commit and are in the hand-back; clippy's dead-code lint needed a test-only commit (f069720).

Seams left:
- Claim 27 for the walking and driving locomotors (a building's position, not its centre).
- Item 4: the r75 rule at tick 9316 is not identified. At tick 10745 every object's position and behaviour is the same in both builds, so what parts s7g9 most likely lies in (10745, 21491]; it is not traced.

The P3 census rule: the batch adds no saved field, so it makes or moves no back-reference of `World::census`.

## Batch S47: the blockage delay at a failed re-path, a queued enter at the arrival, and a building's point for walking and driving (Q-3329, Q-3330, S-0208 claim 27)

Base main c985680, not rebased. The batch builds two answers in S-0208 r89 and S46's seam:
- Q-3329's, in claim 25 and W20: a re-path that fails within one cell keeps the walker's movement in progress, so the blockage and retry delays run;
- Q-3330's: with `quirks.walker_search_ignores_queued_enter` off, a walker in area guard with enter queued goes in at its arrival (S-0605 claim 16);
- S-0208 claim 27 (r82, Q-3317) for the walking and driving locomotors: a building gives its centre, or its docking point for the helipad, unit-repair and vehicle-bunker flags.

The work went in this order:
1. Item 1: design (dfde8bd), tests (978f672), fix (029490c).
2. Item 2: design (e230a19, 3c51d72), test (f5c9e66), fix (f05c54c).
3. Item 3: design (3e83650), request Q-3353 (a83f1d9), tests (965e532), fix (21c96dc).
4. Formats: design note (1842eb0) and bump (1e88ac5), then a rustfmt commit (ff69093).
5. The M6c gate seed (1be9bfa) and the goldens (5b5d8ad).

No fix commit changes a test of its own item; 21c96dc moves two set-ups of `claims_s0603.rs` that claim 27 changes (below). Each red was shown in a separate base worktree checked out at the test commit (`r1.txt`, `r2.txt`, `r3.txt`). Logs are in `local-lane/sim-issues/s47/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| Q-3329, a failed re-path within one cell (S-0208 r89 claim 25, W20 c1) | A re-path of S-0209 claim 10 that failed within one cell ended the walker's movement in progress, and the end-of-update assignment of S-0215 claim 22 then ended the retry delay | fixed (029490c; tests 978f672, red there: the W20 c1 test at "the retry delay runs at the end of F") | `path.rs`'s failure keeps the movement in progress for a walker's re-path; `process.rs`'s end-of-update check reads "not moving" as `occupancy::in_progress` (S-0208 Terms), so a walker in *move* whose path is exhausted gets no assignment while its movement is in progress | `claims_s0209.rs`: `r89_w20_c1_a_failed_re_path_within_one_cell_keeps_movement_in_progress` (new); `r89_the_end_of_update_assignment_after_a_failed_search_restarts_the_blockage_delay` (W20 c2, the seams7b test no longer ignored); `seams7b_the_control_…` |
| Q-3330, a walker in area guard with enter queued (S-0208 r89, S-0605 claims 6, 9 and 16) | With the switch off the walker walked back to the building but did not go in while enter was only queued | fixed (f05c54c; test f5c9e66, red there: "it walks back in from area guard") | `occupancy::reads_queued_enter` is the off form's condition; the mission target and the absorber's arrival (`absorb.rs`) both read it; the late start of that update makes no start for a unit inside an absorber (`process.rs`) | `claims_s0605.rs`: `claim_16_switch_off_back_in_from_area_guard` (no longer ignored), `claim_16_switch_on_no_back_in_from_area_guard` (control) |
| S-0208 claim 27 for walking and driving (S46's seam) | A walker's object point and a driver's heading point gave a building's position, the centre of its top-left cell | fixed (21c96dc; tests 965e532, red there: 0 of 7 pass) | `path::object_point`: for a walker or driver and a building target, `lift::lifter_point` (the docking point for a docking type by S-0607 claim 7's precedence, the centre otherwise); `drive::supplied_point` takes it. The readers of the heading point (S-0402 claims 8 and 26, the C7-P5 yield, S-0209 claim 8, the driver's searches, refresh and braking) follow | `claims_s0208_c27.rs` (new, 7 tests): 1×1 to 5×3 foundations from four sides for a walker and a driver; the unit-reload flag alone gives the centre; a unit-repair dock offset; a helipad with no contact gives the centre; a vehicle bunker's docking point for a walker and a crusher; a capturing engineer keeps the centre for 10 ticks and through a save. `claims_s0603.rs` claims 4/5 and 15: the set-ups moved onto the centre cell's row (a diagonal step to a 2×2 centre arrives after the timed sale) |
| Q-3353, when a driver's docking point is taken | A driver's readings would recompute a position-dependent docking point at each reading; claim 27 takes it at the assignment | requested (a83f1d9) | none: the point is taken at each reading, as `lift::lifter_point` gives it, until answered | none |

**Formats** (1e88ac5): save goes from 169 to 217 and AI game from 149 to 197, the brief's numbers (renumbered to 170 and 151 at the merge; main had 169 and 150). The AI game bump is forced by the sim change. Wire stays 13. The rules schema does not change.

**Goldens** (5b5d8ad): all eight move. duel-m6c is seed 2 now (below). duel-m7a moves from the hash after tick 350 and duel-m7b from tick 620 on. duel-diagonal (8 hashes), duel-naval (3), duel (1), scripted (2) and skirmish (3) move in isolated hashes only, the next hash equal again. These were not dumped; the pattern matches item 1's one-update difference (a walker that arrives with its path exhausted keeps its movement in progress for that update).

**M6c gate seed** (1be9bfa): seed 3 now ends at tick 8528 instead of playing its 12000 ticks; its hashes part from base between ticks 10 and 20. The gate plays the reference rules, where `quirks.walker_search_ignores_queued_enter` is on and item 2 cannot act, so the parting comes from item 1 or claim 27; it is not dumped. Seed 2 is the first of 1 to 16 that plays all its ticks (smallest counter 4) and exercises every ability (`try_seeds`), as at S20.

Lanes compare base c985680, run in this worktree before any change, with the batch at ff69093 (the gate and AI suite at 5b5d8ad, the final checks at the last commit).

**Oracle arena** (`--record all`, two threads): six M-0001 rows move, each by one or two games towards our side A; failing counts stay M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0020 0 of 25 checked, M-0021 10 of 50. In M-0020 part 1 one differing game's winner end moves from row end 1216 to row end 912, still 3 of 6. One flipped game was traced with `--trace` (the M-0001 row with four flips on side B; seed 1, from the column's win to the row's): the traces part at tick 22, where three walkers hunting a building since tick 1 step differently. The trace prints no object point; the parting is consistent with claim 27 (a hunting walker's heading point for a building becomes its centre), and item 2 cannot act in the reference rules.

**Sweeps** (seeds 1 to 16), all three pass. m6c: the smallest minimum stays 0, but six seeds end early where one did (seeds 1, 3, 6, 7, 9 and 13 at 8116, 8528, 7933, 7128, 7206 and 7964; seed 15, which ended at 7028, now plays 12000), and eight seeds have a counter below 3 where two did. These are not traced: the M6c harness has no per-tick hash or object dump. m7a: the smallest minimum goes from 2 to 1 (seed 14's psychic sensor orders), and the bunker counters fall to 2 on seeds 1 and 7; no game ends early. m7b: its own counters stay at 3 or more on every seed.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games): 11 games change winner (house 0 wins 26 before, 19 after). Each was traced by per-tick hashes and `GLASNOST_AI_OBJS` dumps to its first differing tick, where exactly one object differs:
- the 7 on map 2 part at tick 2198 (seed 8's at 2371) at an infantry unit headed for building 494, whose object point moves by (+128, +128) to the 2×2 centre: claim 27;
- the 4 on maps 0 and 8 part at an infantry unit that arrived the tick before with its path exhausted: its movement stays in progress, so it gets no end-of-update assignment: item 1's reading of S-0215 claim 22.

**Suites.** The AI suite with its ignored tests passes on base and after with the same per-binary counts, and no AI golden moves. The sim gate with its ignored tests passes 17 on base and after. The final checks run on the batch's last commit and are in the hand-back.

Seams left:
- Q-3353 (open): when a driver's position-dependent docking point is taken.
- The m6c sweep's six new early ends are not traced to a first tick and object.
- Item 4 (S45's s7g9 parting in (10745, 21491]) was not started.

The P3 census rule: the batch adds no saved field, so it makes or moves no back-reference of `World::census`.

## Batch S48: the owner's own building at fade level 15 (S-0516 r36 claim 26), and S47's untraced moves

Base main 44a6bab, not rebased. Item 1 builds S-0516 r36 claim 26, which closes R20's seam: none of the click and hover refusals applies to an object the viewing house owns, so the owner hovers, clicks, picks and selects its own building at fade level 15 that it does not sense. Item 2 traces four of S47's moves that S47 left untraced, between S47's base c985680 and its head eaf93ec.

The work went in this order:
1. Item 1: design (ea20d7d), tests (79ecb92), fix (37b8cd1).
2. Item 2: traced only; no code change.

The fix commit changes no test of its own item. With main's leave, it inverts one assertion in `crates/app/src/game/client_pointing_tests.rs`, which the fix turns red: the owner can now click its own building at fade level 15. The red was shown in a separate base worktree checked out at the test commit (`r1.txt`: the three claim 26 tests fail, 5 pass). Logs are in `local-lane/sim-issues/s48/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| S-0516 r36 claim 26, the owner's own building at fade level 15 (R20's seam) | `view::clickable` refused a building at fade level 15 that the viewing house does not sense, for its owner too, so the owner could not hover, click, pick or select it | fixed (37b8cd1; tests 79ecb92, red there: 3 of 8) | `cloak::view::clickable`: the fade-level-15 refusal, like the full-cloak one, applies only to a house that neither owns nor senses the building. `pickable`, `targeting::clickable_by` and `selectable_by` follow it. The claim 24 hidden-target refusal (`cloak::hidden_from`, levels 11 to 15, the owner too) is not changed | `claims_s0516_hidden.rs`: `claim_26_clicks_and_selection` (level 15: the owner picks it, an ally that does not sense it does not); `claim_26_r36_the_owner_picks_and_selects_its_own_building_at_level_15` (new: hover, pick and select for the owner; the select cursor and click; a not-allied and an allied other house refused; no re-evaluation deselect 30 ticks on); `claim_26_r36_an_order_onto_the_own_building_at_level_15_is_as_at_a_lower_level` (new: cursor and click at level 15 equal those at level 14, default and force-select). `client_pointing_tests.rs`: one assertion inverted |
| Callers of `clickable`, `pickable` and `selectable_by` | Every caller passes the clicking or viewing house: the select offer (S-0214 claim 1), the click, G-table rows G2 and G10, tap (claim 32), and the client's per-tick selection filter. The change acts only for an owner's own building at level 15, which claim 26 settles in each case. Watch mode sees each object as its own house does: that is our design, not a spec question | checked | none | none; no request filed |
| `pointing::cloak_clickable` (crates/app) | It is `owner || view::clickable(w, id, v)`, with `owner = v.is_some() && o.owner == v`. `clickable` now passes every owned object under the same guard and refuses a missing object, as the wrapper does, so the two agree for every input | can go (not touched) | none | none |
| S47's duel-m7a move from tick 350 (seed 17) | Per-tick hashes first differ at tick 350. One object differs: infantry 28 (house 1, in *move*, holding a cell destination, arrived at 349, no walking target, no stored path, no failed search). Its speed fraction is 1 on base and 0 on head. S47 item 1 reads S-0215 claim 22's "not moving" as no movement in progress (S-0208 Terms, *Locomotor moving*; S-0107 Terms). A walker with a destination has movement in progress, so head makes no end-of-update assignment, where base assigned and set the fraction to 1 (S-0208 claim 25) | S47's rule as built; not a fault | none | none |
| S47's duel-m7b move from tick 620 (seed 11) | Hashes differ at ticks 464, 465 and 616 to 620, and are equal from 466 to 615. At 464 one object differs: infantry 16 (house 1, arrived at 463), speed fraction 1 to 0, the same item 1 reading as duel-m7a. At 616 one object differs: infantry 9 (house 0, *guard* with *enter* queued, destination its own building 59). Only its object point differs, from the building's position (4992, 9088) to (5120, 9216): S-0208 claim 27 (r82, Q-3317), S47 item 3 | S47's rules as built; not a fault | none | none |
| Two of S47's new M6c sweep early ends (seeds 3 and 1; head ends at 8528 and 8116, as S47 reported) | Both part at tick 16, at the same four objects. These are house 0's infantry 15 to 18, in *guard* with *capture* queued, each with a destination on a neutral 2×2 building. Only their object points differ, from the buildings' positions to their centres (+128, +128): S-0208 claim 27. The opening is seed-independent, so the first parting names claim 27 but does not explain the end ticks, which come thousands of ticks downstream | S47's rule as built; not a fault | none | none |

**Formats:** none move. Save stays 170, AI game stays 151 and wire stays 13. The fix adds no state.

**Goldens:** none move. The sim gate passes 17 on base and after.

Lanes compare base 44a6bab, run in this worktree before any change, with 37b8cd1.

**Oracle arena** (`--record all`, two threads): equal on base and after but for timings and out-dir names. Failing counts are M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0021 10 of 50, and M-0020 0 of 25 checked (28 rows).

**Sweeps** (seeds 1 to 16): every per-seed line is equal on base and after. m6c's smallest minimum is 0 (early ends at seeds 1, 3, 6, 7, 9 and 13), m7a's is 1, and m7b's smallest drain minimum is 3.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games): all 36 "over after" lines are equal.

Seams left: none from this batch. S47's seam "the m6c sweep's six new early ends are not traced" is now traced to a first tick for two of them (claim 27 at tick 16). The other four are not traced; the first parting is likely the same seed-independent tick 16.

The P3 census rule: the batch adds no saved field, so it makes or moves no back-reference of `World::census`.

## Batch S49: the point a driving vehicle measures to (S-0207 r92 claim 29, Q-3353), a factory exit point under an order destination (Q-3374), and a building's control ring height (S-0601 r32 claim 18)

Base main 6b84072, not rebased. Item 1 builds S-0207 r92 claim 29: a driving vehicle's searches, the start of the recomputation, braking and the give-up test use the locomotor destination point installed at the last assignment, and do not ask the destination again. Read literally, claim 29 keeps a vehicle at its factory exit point for good when its order destination was given before the exit: the exit track (S-0417 claim 5) installs the exit cell's centre at height 0 and keeps the destination, and claim 25's completion row asks that destination and never clears the exit point. The quick lane showed it on seed 7: a fresh slave miner never left its factory. That gap is Q-3374, and the batch builds an interim, `design: sim (Q-3374)`. Item 2 builds S-0601 r32 claim 18: a building's control ring rises above its centre point by 104 leptons per level of its art height.

The work went in this order:
1. Item 1: design (fb40c1e), request Q-3373 (5bc7455), a refactor with no behaviour change (14d60bc), tests (1417619), fix (410c080).
2. Item 2: design (56d0385), test (659f857), fix (0a2e328).
3. Q-3374: request (c2222d5), design (bd72811), tests (ff14fc8, moved to a factory release in 67b5430), fix (94c250c).
4. Goldens (e7f3c18) and rustfmt (f3cc8b2).

No fix commit changes a test of its own item. Under the brief, 0a2e328 also updates the one ring assertion in `crates/app/src/mapview/ring_tests.rs` that the fix turns red, from the building's own height to 208 leptons above it. Each red was shown in a separate base worktree checked out at the test commit:
- `r1.txt`: 4 of 5 claim 29 tests fail at their intended asserts; the new-assignment control passes.
- `r2.txt`: the ring test fails at the height, 0 against 520.
- `r3.txt` and `r4.txt`: the two Q-3374 tests fail (the search looks for the exit cell; the vehicle stays on it); the control passes.

Logs are in `local-lane/sim-issues/s49/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| S-0207 r92 claim 29, the point a driving vehicle measures to (Q-3353) | A driver's searches (`path::search_goal`), the recomputation's 512-lepton start, braking and the give-up test asked the destination for its point at each reading, so a bunker's quadrant cell and a building's docking place followed the vehicle round | fixed (410c080; tests 1417619, red there: 4 of 5) | `drive::measured_point`: the installed point (`drive::goal`), which `move_destination` and `path::search_goal` read. The completion row of claim 25 and S-0606 claim 5's arrival test still ask again. The harvester and slave connection tests (S-0402 claims 8 and 26) read the track's target or the unit's own position (`econ::harvest`, `econ::slaves`). With the installed point cleared while the order destination stays, the readings ask the destination: `design: interim (Q-3373)` | `claims_s0207_c29.rs`: the bunker point is kept through the searches; braking and the give-up test measure to it; a new assignment takes the point again; a chase keeps the assignment's point. `claims_s0207_c29_harvest.rs`: the connection test reads the track target |
| S-0207 r92's other hunks | Terms (the order-level destination's point for a driver is claim 29's); claim 20 e (an infantry unit in *move* falls back to rest at the end of its jump tick; a recycling building consumes an entrant in the jump tick); claim 25's completion row (the point supplied at that moment); D35; the notes to S-0212, S-0402 and S-0606; two Open questions | already built on base or carried by item 1 | claim 20 e's infantry fall-back runs through S-0215 claim 22's end-of-update check and the recycling consumption through S-0603 claim 16 (`structures::grinder`), both on base; 410c080 updates teleport.rs's citation to r92. The completion row already asked the destination at the track's end | none new |
| A factory exit point under an order destination (Q-3374, filed) | The exit track (S-0417 claim 5) installs the exit cell's centre at height 0 and keeps the order destination given while tethered. Under claim 29 the searches look for the exit cell, and claim 25's completion row (destination cell, height within 208) never clears it, so the vehicle stays on its exit cell (seed 7, map 0: slave miner 1255 from tick 4138) | interim built (94c250c; tests ff14fc8 and 67b5430, red there: 2 of 3) | `DriveState::exit_point`, set by `begin_exit_track` and cleared by every installed point (`take`, the stop, the lift stand-in): while it is set and a destination is held, `measured_point` asks the destination, as before claim 29. `design: sim (Q-3374)` | `claims_s0207_c29_exit.rs`: the readings ask the destination after the exit track; a new assignment ends the interim. `claims_s0207_c29_factory_exit.rs`: a tank released from a war factory with a destination given while tethered leaves the apron and reaches it |
| S-0601 r32 claim 18, a building's control ring height | `mind::ring` put a building's ring at its reference point, at the building's own height | fixed (0a2e328; test 659f857, red there) | the centre point raised by 104 leptons (Constants) per art height level (Import mapping: 2 when absent) | `claims_s0601.rs`: claim 18's ring height on a HUTH5-like test building; `ring_tests.rs`: the one assertion updated |

**Formats:** save 170 to 223 and AI game 151 to 203, for the new saved `DriveState::exit_point`. Wire stays 13.

**Goldens:** all 9 move (e7f3c18), each by the hash only: every driver's state gains the flag, and each file keeps its length. A probe of the whole state with the flag stripped (the rules handle dropped) compared base 6b84072 with the head:
- scripted and combat: equal at every tick;
- skirmish: equal at every 100th tick;
- duel, duel-diagonal, duel-naval, duel-m6c and duel-m7b: equal final state at the same end tick.
- duel-m7a: differs from tick 11964, at vehicle 8 in *enter* onto object 370. Both hold the installed point in cell (30, 33). Base searches for (31, 33), the point 370 gives at the search, and the head searches for (30, 33): claim 29.

The sim gate passes 17 on base and after.

Lanes compare base 6b84072, run in this worktree before any change, with 94c250c.

**Oracle arena** (`--record all`, two threads): failing counts are unchanged (M-0001 10 of 96, M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0021 10 of 50, M-0020 0 of 25 checked). Two rows move within FAIL (side outcome): M-0001 LTNK against E1, ours 4/8/0 to 3/9/0, and TELE against BFRT, ours 1/3/8 to 2/2/8.

**Sweeps** (seeds 1 to 16):
- m6c: every per-seed line is equal; the smallest minimum is 0.
- m7a: the smallest minimum stays 1. Counters move on seeds 9, 10, 13, 15 and 16, and seed 9 adds "infantry absorbed = 2" below 3.
- m7b: equal but for timings; the smallest drain minimum is 3.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games): 2 winner changes, both traced to claim 29.
- Seed 7, map 2, sides 1 and 2 (house 1 at 47612 to house 0 at 66448): first differs at tick 26628, vehicle 953 attacking with destination cell (82, 88). Base's search looked for its track's cell (83, 88); the head's looks for the installed point's (82, 88).
- Seed 10, map 2, sides 1 and 2 (house 0 at 68895 to house 1 at 46464): first differs at tick 31851, vehicle 11350 attacking infantry 8985, which had moved a cell since the assignment and dies that tick. Base's search for the infantry's present cell fails and the destination goes; the head searches for the installed point's cell, finds a path and keeps the destination until the body is removed at 31869.
- Same winner, other length: 15 games. Equal: 19 games. Before the interim, seed 7 had 7 of 9 winner changes, every game of side 2 going to its opponent; the first, map 0 sides 0 and 2, traces to the stall at tick 4138.

Seams left: Q-3373 and Q-3374 are open, with their interims. Q-3374's interim reads the destination while the flag is set, so a vehicle sent on from its factory exit keeps the pre-claim-29 readings until its next assignment.

The P3 census rule: the batch adds one saved field, a bool, so it makes or moves no back-reference of `World::census`.

## Batch S50: the point installed again and a failed fresh search (S-0207 r103 claims 29 to 31, the answers to Q-3373 and Q-3374)

Base b453c29, not rebased. S-0207 r103 answers Q-3373 and Q-3374, so S49's two interims go. Claim 29 now reads the installed point only for a driving vehicle. Claim 30 installs the destination's point at an update with no track, no point and no stored steps, after claim 26. It makes no install while the vehicle sinks or turns, or in *guard* or *unload*, and it asks an infantry destination again at a track's end. Claim 31 clears the point at any failed fresh search. A failure within one cell (row 3) keeps the destination and searches again every *R*. A point in the vehicle's own cell, a factory exit end among them, makes a search that fails and clears the point. `DriveState::exit_point` is removed. One case is held back for request Q-3389: a vehicle that has made its movement arrival at a depot gets no claim 30 install.

The work went in this order:
1. Design (fd69186), tests (f31ab4c), request Q-3389 (f4b8445).
2. Fix (7d08b93).
3. Goldens (cf15db5), rustfmt (fefefbc), design additions (8241d9a) and a clippy collapse in one test (dcffd33, its checks unchanged).

The fix commit changes no test. The red was shown in a separate base worktree checked out at f31ab4c (`red-base.txt`): 18 tests fail in 11 targets. `claims_s0507_r11` passes there, as its assertion only got looser. The guard tests also pass there: no vehicle ask at a track's end, no install in *guard*, a new assignment installs its point. Logs are in `local-lane/sim-issues/s50/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| S-0207 r103 claim 29, which vehicles read the installed point (Q-3373) | S49's readings used the installed point for every driver, and asked the destination when the point was cleared and the destination stayed (Q-3373's interim) | fixed (7d08b93; tests f31ab4c) | `drive::measured_point` returns `goal()` only while driving (`drive::driving`); the interim's destination ask is gone | `claims_s0207_c29.rs`, `claims_s0207_goal.rs`, `claims_s0207.rs` re-cited and updated to r103 |
| S-0207 r103 claim 30, the point installed again | With no track, no point and no steps, a driver kept no point until its next assignment | fixed (7d08b93; tests f31ab4c) | `drive::install_again`, after claim 26: installs the destination's point except while sinking, turning, in *guard* or *unload*, or after a depot movement arrival (`design: S50 hold-back pending Q-3389`). `drive::ask_infantry_again` asks an infantry destination at a track's end | `claims_s0207_c30_c31.rs`: an update with no point installs it; a *guard* vehicle installs nothing; an infantry destination is asked at a track's end and a vehicle's is not |
| S-0207 r103 claim 31, a failed fresh search | A failed search cleared the point only within 208 leptons; row 3 cleared the destination | fixed (7d08b93; tests f31ab4c) | `path::failed`: a failed fresh search clears the point at any distance (`drive::clear_after_failure`, in the gate-failed, hold and search-failed branches). Row 3 keeps the destination when the search was fresh, so *R* runs. A point in the vehicle's own cell makes a search (`drive::point_in_own_cell`, `drive::own_cell_search`), which fails and clears it. C7-P4 skips its second try for an own-cell search (`path::compute_tried`), and claim 24's write is made only when the searched cell is not the goal (`path::compute_once`) | `claims_s0207_c30_c31.rs`: a failure within one cell keeps the destination and searches every *R*; a track ending in the point's cell clears it and the driver follows. Updated: `claims_s0212.rs`, `claims_s0314_depot.rs`, `claims_s0417_r20.rs`, `claims_s0507.rs`, `claims_s0507_r11.rs`, `claims_s0607_standin.rs` |
| A factory exit point under an order destination (Q-3374) | S49's interim flag `DriveState::exit_point` asked the destination from the exit track | fixed (7d08b93; tests f31ab4c) | The flag is removed (`drive.rs`, `lift.rs`). At the exit end the point lies in the vehicle's own cell, so claim 31's own-cell search fails and clears it, and claim 30 installs the destination's point at the next update | `claims_s0207_c29_exit.rs`, `claims_s0207_c29_factory_exit.rs` re-cited to r103: the tank leaves the apron and reaches its destination |
| A vehicle asleep at a depot and claim 30's install (Q-3389, filed) | Under claims 30 and 31 the second truck of a two-place depot has its point present in all but one tick of every *R*. The depot's waiting processing never finds it still, so it is never repaired (`claims_s0314_depot.rs`) | interim built (7d08b93) | No claim 30 install after a depot movement arrival: today's rule, `design: S50 hold-back pending Q-3389` | `claims_s0314_depot.rs` |
| S-0207 r103's other hunks | S-0215 claim 11a and B38; the S-0212 notes (claims 20 to 23, 27a, P7); S-0402 claim 10a; S-0314 claims 17, 21 and H19; S-0507 claim 6a; S-0417 claims 5, 16 and U15 | seams, not built | text and notes that carry claims 29 to 31 into those specs; the behaviour they name runs through the items above, and no other code change was made for them | none new |

**Formats:** save 223 to 225 and AI game 203 to 206. Wire stays 13. The saved `DriveState::exit_point` is removed.

**Goldens:** all 9 move (cf15db5): combat, duel, duel-diagonal, duel-m6c, duel-m7a, duel-m7b, duel-naval, scripted and skirmish. Base's hash includes the saved flag, so the hashes part at the first driver and say nothing of behaviour. Three files change length, so those games end at other ticks: duel goes from 1818 to 2141 lines, duel-diagonal from 2415 to 1976, and duel-naval from 1856 to 1854. The sim gate passes 17 on base and after, and `xtask gate` matches all 9 under wasm32.

Lanes compare base b453c29 (worktree `sim50-base`) with 7d08b93 plus goldens.

**Oracle arena** (`--record all`, two threads), after only (`arena-after.txt`): M-0001 10 of 96 failing (win test 2, side outcome 10, 156 of 171 reproduced, stock-replay pairings 1 of 12), M-0002 4 of 24, M-0003 3 of 35, M-0019 0 of 6, M-0021 9 of 50, M-0020 0 of 25. The base result was not read, so no row comparison is given; S49 recorded M-0021 at 10 of 50.

**Sweeps** (seeds 1 to 16; `m6c-base.txt` against `m6c-after.txt`, and so on):
- m6c: seed 1 ends at 8678 (was 8450), smallest counter still 0. Seed 6 runs to 12000 (was 7880) with smallest counter 4 (was 0). Medians and maxima move: jumpjet orders min 0 to 1 and max 61 to 92, garrison max 24 to 23, deploy orders min 60 to 54 and max 167 to 155. Passes.
- m7a: every seed's counters move. The smallest minimum stays 1 (base seed 14's psychic sensor orders; after seed 5's bunker exit orders and seed 7's bunker entry, vehicles bunkered and exit orders). Passes.
- m7b: only seed 4 moves: drain counters 25, 9, 15 to 24, 9, 17, and takeover orders 2 to 0. The infantry mutated median goes from 25 to 24. Passes.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-<seed>.txt` against `quick-after-<seed>.txt`): 7 winner changes, 28 same winner with another length, 1 equal (seed 7 map 0 sides 0 and 1). The state hash parts at the first driver, so each change was traced with whole-object dumps compared with `exit_point` stripped (`objdiff.py`). Each traces to r103 as built:
- Seed 8, map 2, sides 0 and 2 (house 1 at 50329 to house 0 at 29488): tick 4139, slave miner 752 at the end of its factory exit track, its point in cell (66, 115) under destination (66, 109). The own-cell search fails and clears the point, R runs to 4146 and the path is found at 4147; base found it at 4140 (claim 31).
- Seed 8, map 8, sides 0 and 1 (house 1 at 86335 to house 0 at 38595): from 4944, vehicle 232's gate failure clears its point a tick sooner, every 15 ticks (claim 31). The first other difference is at 12350: harvester 199, blocked, fails a fresh search one cell from (35, 65), keeps its destination with R and has the point installed again at 12351. Base dropped the destination and repathed to (33, 65) (row 3, claim 30).
- Seed 9, map 0, sides 1 and 2 (house 1 at 65130 to house 0 at 38166): 11776, vehicle 1346 in *attack* at (106, 64) with destination (105, 63): row 3. Expired retry fields of vehicle 1279 differ from 8000 and lie dormant.
- Seed 9, map 2, sides 0 and 1 (house 1 at 58759 to house 0 at 48063): 10909, harvester 880 at (82, 39) with destination (82, 40): row 3, the point installed again at 10910.
- Seed 10, map 2, sides 1 and 2 (house 1 at 46464 to house 0 at 37296): 10449, harvester 734 at (79, 107) with destination (80, 108): row 3, installed again at 10450.
- Seed 10, map 8, sides 0 and 1 (house 1 at 74101 to house 0 at 52758): 19683, vehicle 1997 at its factory exit cell (24, 76) under destination (87, 78): the own-cell search fails, R runs to 19690, and the point is installed again at 19684. Base pathed at 19684.
- Seed 10, map 8, sides 0 and 2 (house 0 at 34889 to house 1 at 42011): 4138, slave miner 293 at its exit cell with destination (106, 51): the own-cell search fails, R runs to 4145, and the point is installed again at 4139. Base pathed at 4139.

Seams left: Q-3389 is open, with its hold-back. Factory exits now pay one failed own-cell search and an R wait (7 ticks in the traces) before the vehicle paths. That is claim 31 as written; play-fixes may want to look at it. The r103 hunks in other specs are text-only seams, listed in the table.

The P3 census rule: the batch removes one saved bool and adds no saved field, so it makes or moves no back-reference of `World::census`.

## Batch S51: a driving vehicle's depot destination at its arrival (S-0314 r116, the answer to Q-3389)

Base 8b43df9, not rebased. S-0314 r116 answers Q-3389, so S50's hold-back in `drive::install_again` goes. A driving vehicle whose order-level destination is the depot loses it at its movement arrival in its docking-point cell, in the arrival's update (claim 17). A failed own-cell search of a contacted driving vehicle in that cell counts as the arrival, with only the exchange and the removal (claim 21). Hovering vehicles and ships keep their destination.

The work went in this order:
1. Design (00934b7), request Q-3401 (8f34bfa), tests (b9605f6).
2. Fix (167d542).
3. Formats (03c9a8f).

The fix commit changes no test. The red was shown in a separate base worktree checked out at b9605f6 (`red.txt`): all 5 tests of `claims_s0314_r116.rs` fail, and the guard tests pass there (`claims_s0314_depot.rs` 78, `claims_s0504.rs` 47, the ship guard among them). Logs are in `local-lane/sim-issues/s51/`.

| Issue | Finding | Status | Fix | Tests |
|---|---|---|---|---|
| S-0314 r116 claim 17, the depot destination at a driving vehicle's arrival (Q-3389) | A driving vehicle kept the depot as destination from its arrival to the settling step; S50 held claim 30's install back after the arrival so that the depot could find it still | fixed (167d542; tests b9605f6) | `depot::movement_arrival` sets the destination to none after the exchange when `drive::driving` (now `pub(crate)`); the S50 hold-back in `drive::install_again` is removed | `claims_s0314_r116.rs`: the destination is held in every row before the arrival, in the dock cell included, and none from the arrival on, with no point installed again; the exchange and the removal share the update; the vehicle in place 1 of a two-place depot loses it too and is repaired. `claims_s0314_depot.rs`'s `claim_23_free_place_taken_by_the_processing` (the hold-back's guard) re-cited to r116 Edge cases |
| S-0314 r116 claim 21, a same-cell enter order's failed own-cell search | The search failed and cleared the point, claim 30 installed it again *R* later, and the depot never accepted the vehicle | fixed (167d542; tests b9605f6) | `depot::own_cell_failure`, called after a failed own-cell `fresh_search` in `drive::tick`'s no-track update and in `drive::own_cell_search`: makes `movement_arrival`'s exchange and removal under its tests (*enter*, the depot as destination, the docking-point cell below 208 leptons); no other cell-arrival effect | `claims_s0314_r116.rs`: the failure makes the exchange, the vehicle is settled, no point comes back and the depot repairs it. `claim_17_same_cell_enter_supplies_the_point` re-cited: in *guard* the failure is not the arrival |
| S-0314 r116 claim 17, hovering vehicles and ships | Not affected | guard kept | the removal checks the driving locomotor | `claims_s0314_r116.rs` (a hovering vehicle keeps it, a driving one loses it); `claims_s0504.rs` `s0314_claim_17_a_ship_keeps_the_yard_at_its_arrival` |
| S-0314 claim 20, a full-health enter order on the own untethered depot (Q-3401, filed) | Since r116 the untethered vehicle holds no destination before the order, so `claim_20_same_depot_enter_while_docked` shows that the order gives the depot as destination; claim 20's second bullet with claim 23's empty-depot path give one, and the third bullet's "whatever the depot's pad flag" reads as none | request filed, today's rule kept | none | `claims_s0314_depot.rs`: the destination is asserted only for the tethered case, citing Q-3401 |
| S-0314 r110 to r116, the other hunks | The movement-report paragraph's failed-search clearing and its ratings (built in S50 as S-0207 claim 31); claim 23's restart in a later place (Edge cases; follows from claim 17); H18 h and H19 f observation text; the Open questions; the changes for S-0215 B43 and claim 29 and for S-0207 claims 31, 26 and D30 (text that carries claims 17 and 21 into those specs) | seams, not built | no rule of our code changes beyond the items above | none new; `s0215_claim_29_entry_keeps_the_building_destination` still passes (its vehicle is not in *enter*, so no exchange is made) |

**Formats:** save 225 to 227 and AI game 206 to 208. Wire stays 13. No saved state is added or removed; `depot::Link::arrived` stays, claim 22 still reads it.

**Goldens:** none move. The sim gate passes 17 on base and after, and `xtask gate` matches all 9 under wasm32.

Lanes compare base 8b43df9 with 03c9a8f.

**Oracle arena** (`--record all`, two threads; `arena-base.txt` against `arena-after.txt`): identical. M-0001 10 of 96 failing (win test 2, side outcome 10, stock-replay pairings 1 of 12), M-0002 4 of 24, M-0003 3 of 35 (stock replay 1 of 11), M-0019 0 of 6, M-0021 9 of 50, M-0020 0 of 25.

**Sweeps** (seeds 1 to 16; `m6c-base.txt` against `m6c-after.txt`, and so on): m6c, m7a and m7b identical, all pass.

**AI quick lane** (seeds 7 to 10, maps 0, 2 and 8, 36 games; `quick-base-<seed>.txt` against `quick-after-<seed>.txt`): identical, no winner or length change, so nothing to trace. **AI suite** with ignored tests: 419 passed on base and on 03c9a8f.

Seams left: Q-3401 is open. A vehicle factory that is also a repair depot reads `o.dest` in `econ::release::arrival` after the arrival; no synthetic or stock case of ours reaches it.

The P3 census rule: the batch adds and removes no saved field, so it makes or moves no back-reference of `World::census`.
