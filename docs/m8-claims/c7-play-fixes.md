# Pathing and other fixes: ledger (M8 step C7)

Brief: `.claude/worktrees/M8-C7-FIXES-BRIEF.md`. Our own engine and client: the layer only sets switches whose off position an approved spec already states, so no simulation code changed, and the save format (76) and the AI game format (38) are unchanged.

## The layer

Our own rules patch layer, `data/layers/play-fixes.jsonc` (`glasnost_rules::layers`), shown to the person as *Pathing and other fixes*. It sets chosen quirk switches to the off position their spec states; the defaults in `quirks.rs` stay the original's (PLAN.md D6, D8). Decision of 2026-10-25 (m8-plan.md): on by default in the client's new-game setup; the measurement runs stay on the reference ruleset.

**Admission.** An entry is admitted only when (1) an approved spec states the off position and the engine implements it, (2) the on position reads as a bug in play: a play note or a request names it, or (widened in part 3, 2026-10-28) the approved spec's own Edge cases or quirk text states its effect as a unit stuck, giving up although an open way exists, never moving again, or a visible glitch, and (3) it is not a balance change (D14). Part 3 (below) changed the list: see its table. The file cites each entry's claim and what it answers; `layers::tests::an_unknown_switch_is_refused` and `play_fixes_sets_only_the_admitted_switches_off` keep the file to known switches and to these two.

| Switch (off) | Spec | Answers | Why |
|---|---|---|---|
| `corridor_retry_no_fallback` | S-0212 claim 13 and Edge cases | Pathing reports 1 and 2 (a4-play-notes.md), Q-1771 (a unit at a factory's exit), Q-1870 (a harvester on a refinery's docking cell) | On, a computation whose repeated coarse search fails gives up at once although an open way round exists (claim 23 row 2); off, one cell search without a corridor follows, with the same limits. |
| `corridor_split_probe` | S-0212 claim 13a and Edge cases | The same | On, the retries beside the buildings keep running corridors into them and the 5th cell search inside a corridor ends the computation (claim 13's cap); off, the sector reads as split, its connections close, the coarse search fails and the entry above's search without a corridor finds the way. |

The two only work together: with retry-off alone both synthetic shapes still fail at the cap, with split-off alone they fail after the second search (`claims_s0212_cap.rs`: `play_fixes_the_vehicle_on_the_docking_cell_finds_its_way`, `play_fixes_the_unit_at_the_factory_exit_finds_its_way`, `play_fixes_needs_both_switches`, and `play_fixes_change_nothing_where_the_first_search_finds_the_way`).

Considered and rejected: `group_spread_no_detour` (S-0213 leaves the off behaviour as a question for the lead, so condition 1 fails; superseded in part 4, [c7-pathing.md](c7-pathing.md), where the lead's decision of 2026-10-29 lets the off position be our design and the layer admits it); `substitution_distance_flat_destination` (not clearly a bug in play, and it changes approaches broadly); `corridor_free_part_unusable_cells` (jumpjet infantry only, no play note); `path_search_10000_failure` and the other path switches (no play note or request names a bug). Q-1870 question 3, the second harvester circling a taken dock: S-0402 claims 10 and 10a are unchanged and no switch covers them; the layer removes the cause seen in the lead's reports (the first harvester stuck on the dock), not the circling.

**Plumbing.**
- *Setup and room.* `glasnost_ui::setup::Setup::fixes` (`FIXES_BY_DEFAULT` true), a sixth box in the flags grid labelled with `PLAY_FIXES_NAME`, drawn when the setup is game-wide (a room guest sees the host's choice, greyed, and has no box to press). `Setup::lobby` turns it into `Lobby::layers`. In a room it travels in the room's `Setup` (the lobby's `Edit::Game` carries `fixes`; wire version 9, so the relay must be redeployed), not in the S-0412 flags.
- *Rules hash.* `glasnost_convert::game::load` and `mapview::load_with` take the `Layers` and add their sources after the base tree in `resolve_ruleset`, so `sim_rules_hash` names the layer. The room's create, join and rejoin keep comparing the install's base hash; the start (`StartInfo`), saves, replays and `Game::load` compare the played hash, so a peer whose played rules differ only in the layer is refused at the start on the rules hash (`client_net_tests::a_peer_with_other_layers_is_refused_on_the_rules_hash`). A save or replay carries the hash and never resolves the rules again: `sim-gate/tests/play_fixes.rs` saves a game with the layer, shows the save and the replay refused under the reference rules and reproduced under its own.
- *Reports.* The pathing report's bundle (`capture::VERSION` 2) has `layers`; a version 1 file reads as the reference ruleset. The tool resolves the install with and without the layer to match the report's rules hash, refuses a report whose recorded layers disagree with the ones that hash names, and prints "Rules: Pathing and other fixes: on/off". The desync bundle (`BUNDLE_VERSION` 3) has the writer's layers in words (`Evidence::with_layers`); the desync tool resolves the install with and without the layer to match the report's rules hash and prints which one matched.
- *Measurement.* The oracle arena, side audit, oracle, smoke and the sim-gate goldens build the reference ruleset; the tick bench and the four-house test set `fixes` off. `GLASNOST_PLAY_FIXES=1` (`Layers::from_flag`; also `on`, `yes`, `true`) puts the layer on for the M6c, M7a and M7b seed sweeps (`glasnost_sim_gate::layered`; the rules name gains `+play-fixes`) and for the skirmish lane (`skirmish`, `ai`, `ai-sides`, so the quick lane); both print a "rules layers" line only when it is on. The app's local lanes (`crates/app/tests/*_lane.rs`) start through the setup screen and so play with the layer, as a person would.


## Results

**The lead's pathing reports 1 and 2, with the layer** (local lane; outputs and a summary in the git-ignored `local-lane/m8/c7/`). A scratch build of 9ed7477 (save format 76, game format 38, the deterministic crates as at d31775d, which wrote the reports) whose desync tool turns both switches off on the resolved ruleset when an environment variable is set, keeping the report's rules-hash string so the snapshot loads; never committed. Without the variable both reports reproduce their capture; with it they do not, as expected, and the replay plays the person's own orders, given in reaction to the stuck harvesters.
- Report 1, the harvester on the docking cell: at the harvest order (tick 19 694) the reference gives up at the cap (5 failed cell searches in the corridor), and the harvester leaves the dock only on the person's move orders 175 ticks later, after 49 refused steps. With the layer the first two cell searches fail and the third, without a corridor, finds 20 steps; the harvester leaves the dock westwards at 19 732 (1 refused step, no give-up) until the person's later move orders turn it back.
- Report 2, the second harvester: in both it circles beside the taken dock (S-0402 claims 10 and 10a). In the reference it docks at 19 985; then its own harvest computation and each of the person's 7 harvest orders give up at the cap (8 give-ups), and it stands on the dock to the capture. With the layer the first harvester leaves, the second docks at 19 814 (171 ticks earlier), unloads, finds its way to the field by the search without a corridor (21 steps) and harvests; all 13 of its computations find a path, and the person's 7 orders take 1-step paths.

So for Q-1771's and Q-1870's shape the layer is the fix in play. Q-1870 question 3 stays a question about the original's behaviour; the circling ends once the dock frees.

**With the layer on** (`GLASNOST_PLAY_FIXES=1`):
- Seed sweeps 1 to 16 pass, with no seed ended early: M6c smallest minimum 4 (reference 2); M7a 1 (reference 1); M7b smallest drain minimum 3 (reference 3), and its informational M7a counters at 0 are the same 15 as on the reference.
- B6's quick lane (seeds 7 to 14, maps 8, 2 and 0, Yuri as house 1 against Allied and against Soviet, 48 games): every game over with a winner and no panic, with the layer and without. Median length 34 955 ticks with the layer, 37 926 without; Yuri wins 29 with, 31 without. Cost on the same binary, runs interleaved under the same load: the games' mean tick averages 2.43 ms with the layer and 2.40 ms without (largest game mean 4.92 and 4.62 ms). The games differ, so this is not a like-for-like measurement.
- A full game (seed 7, map 8, Yuri against Allied, `GLASNOST_AI_QUICK` unset) is over at tick 47 143, the same as its quick game. The replay reproduces the final state, and the mid-game save, loaded and continued, matches.

**Default runs, byte-identical to main.** The arena and the sweeps were compared against main's binary built from the final base 6021e48 (intake 24), and the gate was run on the rebased branch. The quick lane was compared against main at e4ab2a6. The sweeps print the same output on e4ab2a6, e2330bc, b035aa7 and 6021e48, both with the layer and without, so the layer-on results above hold on the final base.
- The oracle arena (`arena --record all --threads 8`) writes byte-identical `ours-M-0001.csv` to `ours-M-0003.csv` and oracle reports (M-0001 31 of 96 pairings failing, 151 of 172 stock outcomes reproduced; M-0002 13 of 24, 43 of 47; M-0003 2 of 35, 61 of 64).
- The M6c, M7a and M7b sweeps 1 to 16 print the same output line for line.
- The quick lane's 48 games print the same output apart from timings (the same ticks and winners).
- `git diff main -- crates/sim-gate/golden` is empty, and `cargo xtask gate` passes.

Format versions: wire 9 (the relay must be redeployed), pathing report 2, desync bundle 3.

**By eye** (for the lead):
- The skirmish setup shows *Pathing and other fixes* ticked in the flags grid, with its whole label.
- In a network room a guest's box shows the host's choice, greyed; the host's untick reaches the guest.
- A pathing report saved with P gives "Rules: Pathing and other fixes: on" in the tool's `report.txt`.
- In a skirmish with the layer on, a harvester whose refinery stands between its docking cell and the field leaves the dock for the field.

## Part 2: a stranded hover (S-0314)

Brief: `.claude/worktrees/M8-C7-PART2-BRIEF.md`. The play seam of intake 30
([intake30-S-0314.md](intake30-S-0314.md), *Seams*) and S-0314's Edge case *Hovering vehicle
stranded*: with `quirks.depot_loss_strands_hover` on (the original), a hover that is down and
settled outside every depot's cells never moves again unless it is disabled and enabled.

**Admitted: `depot_loss_strands_hover` off.**
1. The spec states the off position (Edge cases; Terms, *What ends it*: "losing contact with a
   depot also ends it", and a hover is switched on again at every end). `depot::end_contact`
   implements it.
2. On, the hover never moves again, and nothing on screen says why.
3. It is no balance change: no stat, cost or timing of a working unit changes.

**What the switch's off position frees** (tests in `claims_s0314_stranded.rs`, both positions,
the rules read from the layer file through `Layers::apply_quirks`):

| Case | Reference (on) | Layer (off) | Test |
|---|---|---|---|
| Depot destroyed after the hover is down and repaired (a 5 × 5 synthetic depot with its places 512 leptons from its centre, so S-0312 claim 17 releases the contact instead of destroying it) | settled, stays put under every later order | not settled once the contact ends; it moves | `a_hover_whose_depot_is_destroyed_moves_only_with_the_play_fixes` |
| An untethered hover ordered off a 1 × 1 depot while it still sinks | claim 20 releases the contact; it stays settled, comes down outside the depot's cells and never moves again | the same release ends being settled; it rises and goes where it was sent | `a_hover_ordered_off_its_depot_while_sinking_moves_only_with_the_play_fixes` |
| The same with a tether (set by the test: our synthetic depot's arrival sets none) | the contact is kept (claim 20) and it is repaired where it stands; after the service ends it stays settled and never moves | the same until the service ends; the contact's end ends being settled and it leaves | `a_docked_hover_ordered_off_while_sinking_is_freed_at_its_service_end_by_the_play_fixes` |
| A deploy order that breaks the contact of a sinking hover (S-0213 claim 10) | settled | not settled from the vehicle's next turn | `a_deploy_order_breaking_a_sinking_hovers_contact_frees_it_only_with_the_play_fixes` |

So the switch frees the drifted case too, and the brief's step 2 (another switch, or a request)
is not needed. Every way to become settled at a depot without a disabling has contact (the
arrival and the settling processing, `depot::settle`), and every such contact ends: by an order
for an approaching vehicle, at the service's end (or for lack of money) for a docked one, or by
the depot's loss. A hover still stuck after its contact has ended was disabled. It is settled
until its enabling, as the spec has it. The layer does not change that.

**Code.** The switch's off position was reached only through `depot::end_contact`. A contact
broken off elsewhere, such as a deploy order's clear (S-0213 claim 10) or an owner change's
leave (S-0509 claim 14), dropped the depot link at the vehicle's next turn (`depot::vehicle_turn`)
without ending being settled. With the switch off, that drop now ends it too. `// design:`: it
ends one tick late at most, at the vehicle's next turn, not at the clear itself. The spec gives
no tick. With the switch on (the default) nothing changes. No new state: save format 80 and AI
game format 42 are unchanged.

**Naval yards and ships.** S-0504 (Open questions) leaves open whether a *ship* is stranded
after its yard's loss and does not extend the switch to ships. Our engine strands no ship in
either position: only the hover locomotor reads being settled (`hover::tick`). A hover at a naval
yard is a hovering vehicle under S-0314, so the switch covers it. No request.

**Formats.** The layer travels as a flag (`Lobby::layers`, the pathing report's `layers`, the
desync bundle's words), and its content travels only through `sim_rules_hash`. A new entry
changes no wire, save, report or bundle layout, so wire 9, pathing report 2 and desync bundle 3
stay. A save, replay or report written with part 1's two-entry layer now matches neither the
reference nor the new layered hash. It is refused on the rules hash, as a file from any build
with other rules is. A room of mixed builds is refused at the start
(`a_peer_with_other_layers_is_refused_on_the_rules_hash`).

**Results** (local lane; outputs in the git-ignored `local-lane/m8/c7-2/`, `before/` built from
main b5b62ba and `after/` from this branch):
- *Default runs, byte-identical to main.* The oracle arena (`arena --record all --threads 8`)
  writes the same `ours-M-0001.csv` to `ours-M-0003.csv` and oracle reports (M-0001 31 of 96
  pairings failing, 151 of 172 stock outcomes reproduced; M-0002 13 of 24, 43 of 47; M-0003 2 of
  35, 61 of 64). The M6c, M7a and M7b sweeps 1 to 16 print the same output line for line.
  `git diff main -- crates/sim-gate/golden` is empty, and `cargo xtask gate` passes.
- *With the layer on* (`GLASNOST_PLAY_FIXES=1`), the sweeps also print the same output as main's:
  M6c smallest minimum 4, M7a 1, M7b smallest drain minimum 3, and no seed ends early. Their duels
  strand no hover, so the new entry changes nothing there. B6's quick lane (48 games) gives the
  same length and winner in every game as part 1's layer run: median 34 955 ticks, Yuri wins 29,
  and every game is over with a winner and no panic.
- *The app lane* (`GLASNOST_LANE_SCENE=hover-stranded`: a real naval yard, 4 × 4, with the hover
  damaged to a quarter). The hover survives its yard's destruction in both positions, because its
  place is far enough from the yard's centre (S-0312 claim 17). Unticked, it stays settled and does
  not move on either of two later orders. Ticked, it is no longer settled once the yard is gone,
  and it goes where each order sends it. In the drift part the hover is ordered, on its arrival
  while it still sinks, to open water beside the yard. Unticked, it comes down one cell outside the
  yard's cells, still settled, and does not move on the next order. Ticked, the order ends being
  settled, and the hover reaches the water it was sent to and later obeys the next order. (The
  lane's "moved after the first order" counts only the 300 ticks after the run's wait, so it is
  false in both drift runs.)

**By eye** (for the lead), in a skirmish with *Pathing and other fixes* ticked:
- Send a damaged hover to a naval yard, and once it is down on the yard, have the yard destroyed.
  The hover should rise and obey orders. Unticked, it should stay down where the yard stood and
  ignore orders.
- Send a damaged hover to a naval yard, and as it arrives, still sinking, order it to the water
  beside the yard. It should go there. Unticked, it should come down just outside the yard and
  ignore orders from then on.
Selling the yard was not tried.

## After intake 34 (S-0212 r82)

S-0212 r82 ([intake34-S-0212.md](intake34-S-0212.md)) answers Q-1771 and Q-1870: **the original
walks out of its own base.** Every building gives the corridor exemption to its exemption
rectangle, the foundation grown by one cell on every side, so the first cell search inside the
first corridor follows the band of cells round the buildings and finds the way. No retry is made.
Pathing reports 1 and 2's shapes were our engine's missing rectangles, not the original's
behaviour, and the reference ruleset now passes them.

Neither switch is removed or changed: both keep their off position as stated. What changed is
the case they stand for: S-0212's short-line entry no longer fails, and both switches are now
stated for *a short strip of ground the unit cannot enter* (a land type open to the coarse
searches through the wheel column but blocked for the unit). A ledge gives the give-up at the
limit of 5 whatever `corridor_retry_no_fallback` says (`a_ledge_reaches_the_limit_of_five_searches`,
with the layer's other entry, `corridor_split_probe`, on); its own new switch, `corridor_free_part_ignores_height`, is the one
that lets the retries go round it.

Re-judged by the admission rule, **both corridor entries now fail condition 2**: no play note
or request names a bug that either fixes. Conditions 1 and 3 still hold.
- `corridor_retry_no_fallback`: reports 1 and 2 are passed with it on or off
  (`claims_s0212_cap.rs`: `play_fixes_change_nothing_in_the_reports_shapes`; the lane below).
  Only the spec's own strip case names the bug, which is neither a play note nor a request, and
  whether stock data has such a land type is not checked (S-0212 Edge cases).
- `corridor_split_probe`: with the entry above off, a strip is passed at every alignment whatever
  this switch says (`corridor_retry_no_fallback_off_goes_round_a_strip`): no pass or fail
  difference in the strip tests, though the split reading can still close another connection
  and so give another route.
- `corridor_free_part_ignores_height` (new, not in the layer): the ledge's give-up (a unit refuses
  an order although an open way round exists) is condition 2's kind of bug, but no play note or
  request names it yet.

**Reports 1 and 2's cases, with the layer and without** (local lane, scene `base-exit` of
`m6b_lane.rs`; outputs in the git-ignored `local-lane/m8/a3-34/lane/`). The reports' own files
were written by older builds and no longer load, so the scene rebuilds their shapes on the
install: P31's 3 × 2 block with a row of ten (an infantry unit and a vehicle at the block's exit,
sent 16 cells north and 3 west), and a harvester on a dock-unload refinery's docking cell with two
rows of eight 1 × 1 buildings west of it, sent 16 cells west. On the seventh offered map
(`GLASNOST_UI_MAP=6`), with *Pathing and other fixes* ticked and unticked alike: no destination is
cleared, each first path computation makes one cell search, inside the corridor, which finds
the path, and the units arrive after 602 (infantry), 500 (vehicle) and 618 (harvester) ticks. The
first and sixth offered maps give the same for P31 and the fourth for the harvester; the others
have no open area large enough.

The M6c, M7a and M7b sweeps 1 to 16 now print the same lines with `GLASNOST_PLAY_FIXES=1` as
without (before this intake the layer changed 23, 39 and 27 lines): their duels have no strip
and no stranded hover, so no entry of the layer acts there. The layer is not inert in games on
the install: in the quick lane (seed 7, maps 8, 2 and 0, 9 games) one game on map 0 ends at tick
34 205 with it and 34 217 without, with the same winner; which of the three entries acted (the
hover entry among them) was not isolated. The other 8 games are the same.

The layer is unchanged in this intake (its comments now say the above), and so is
`sim-gate/tests/play_fixes.rs`. For the lead: by the rule, drop both corridor
entries; readmit `corridor_retry_no_fallback` (and `corridor_free_part_ignores_height`) when a
play note names a strip or ledge give-up.

## Part 4: our own pathing improvements (2026-10-29)

The lead's decision of 2026-10-29 adds an exception to condition (1): our own pathing improvements are admitted with an off position of our design, marked `// design: C7-P4` in the code; conditions (2) and (3) still hold. Part 4 adds three such entries (`path_through_stalled_blockers`, `unreachable_move_dropped`, `group_spread_no_detour`), so the layer holds nine. Part 5 adds three more, marked `// design: C7-P5` (`rally_point_shared`, `deadlock_no_yield`, `harvest_cell_shared`), so it holds twelve. Ledger: [c7-pathing.md](c7-pathing.md).

## Part 3: the layer re-judged (2026-10-28)

Brief: `.claude/worktrees/M8-C7-PART3-BRIEF.md`. Decision of the main session, under *Decisions of
2026-10-25*: condition 2 is widened as the *Admission* paragraph above says; conditions 1 and 3
stand. No simulation code changed: the layer, its tests and its comments only. Save format (83),
AI game format (45), wire 9, pathing report 2 and desync bundle 3 are unchanged.

**The layer after part 3** (six entries; each off position is stated by its spec and built):

| Switch (off) | Spec | Why it passes | Test (layer file read through `Layers::apply_quirks`) |
|---|---|---|---|
| `corridor_retry_no_fallback` | S-0212 claim 13, Edge cases | kept; the strip Edge case: a unit refuses an order "although an open way round exists" | `claims_s0212_cap.rs` and `claims_s0212.rs` strip tests (unchanged) |
| `corridor_free_part_ignores_height` | S-0212 claim 13a, Edge cases (ledge) | admitted: the Edge case gives the ledge "the same gameplay impact as the strip (a unit refuses an order although an open way round exists)" and states the off position (the free part and its borders apply the height rule) | `play_fixes_layer_goes_round_a_ledge` (ledges of 3 and 7 cells at 8 shifts: the reference fails after 5 cell searches, the layer finds the way); the switch's own `corridor_free_part_ignores_height_off_goes_round_a_ledge` |
| `depot_loss_strands_hover` | S-0314 | part 2, unchanged | `claims_s0314_stranded.rs` |
| `unload_without_contact_stalls` | S-0402 claim 11, Edge cases | admitted: "stays in *unload* for good ... until an order gives it a behaviour"; off falls back to rest and the harvester queues what S-0215 claim 15 gives it. Acts only in that state. | `play_fixes_layer_frees_a_harvester_unloading_without_contact` (reference stalls in unload, layer does not); `claim_11_quirk_unload_without_contact_stalls` |
| `absorber_stale_contact_kept` | S-0605 claim 17, Edge cases | admitted: the contact "keeps the building's link place for good, and every later entrant waits in line for ever"; off gives the contact up when the actor has neither *enter* current nor queued. Acts only in that state. | `play_fixes_layer_lets_the_waiting_entrant_in`; `claim_17_stale_contact_both_positions` |
| `hover_landing_request_stands` | S-0503 claim 19, Edge cases | admitted: a landing-first in-place hover whose first deploy step found it above the ground "never deploys"; off treats the request as met once the hover no longer moves. Acts only while the request stands. | `play_fixes_layer_deploys_a_hover_with_a_standing_landing_request`; `quirk_hover_landing_request_stands` |

**Dropped: `corridor_split_probe`.** With `corridor_retry_no_fallback` off a strip is passed at
every alignment whatever this switch says (`corridor_retry_no_fallback_off_goes_round_a_strip`),
and the ledge is answered by its own switch above: no pass or fail difference in any test. The
caveat of *After intake 34* stands (the split reading could still close another connection and so
give another route), but a layer entry with no shown effect is not kept. Its default stays on.

**The brief's candidates**

| Candidate | Verdict | Reason |
|---|---|---|
| `bouncing_anim_impact_hits_walk` (S-0319) | rejected | Its Edge case states no stuck, give-up or glitch; it decides who an impact's hits reach (a hit that takes its object out ends the hits; a building entering is hit). Combat damage: condition 3. |
| `bouncing_anim_rebound_faults` (S-0319) | rejected | Names two faults of the rebound but their only effect is claim 7's choice between an impact and a stopping landing, so which objects are hit: no stuck or glitch stated, and combat: conditions 2 and 3. |
| `anim_resource_no_growth_entry` (S-0319) | rejected | Whether a new resource cell is offered to the growth list, with one more RNG draw: an economy and random-stream change, nothing stuck: conditions 2 and 3. |
| Intake 32 (S-0317) | none named | The intake 32 ledger names no quirk for C7. Its two switches (`voxel_debris_deck_rebound_slope`, `voxel_debris_cliff_step_faults`) are of the rebound family above and fail for the same reasons. |

**Other quirks whose text reads as a bug.** The search was over the quirk doc lines and the specs'
Edge cases for stuck, stall, never, for good, strand, freeze and give-up wording; 43 of the 498
switches are not read by the sim at all (presentation, UI, crates) and were not candidates.
Near misses, with the condition each fails:

| Switch | Spec | Condition |
|---|---|---|
| `aircraft_unit_step_stall` | S-0501 claims 4, 5, Edge cases | Stated as stuck (an attacker hovers and "never fires"; a move or pad return stops short), but the off position changes F3's step for every aircraft in flight at those facings, not only the stalled state, and the spec rates it "a design rule, not an oracle measurement": not clearly condition 3. |
| `aircraft_zero_step_attack_stall` | S-0501 claim 42 | "A proposed design alternative", `unmeasured`; changes when an attacker fires: not clearly condition 3. |
| `unstartable_queue_head_stalls` | S-0404 | A production line, not a unit; low confidence; off changes which items the queue drops: not clearly condition 2 (no unit stuck) or 3. |
| `absorber_release_bridge_ends_search` | S-0605 claim 12, Edge cases | The passenger "stays inside this processing", no stuck stated: condition 2. |
| `lift_standin_stays_seized` | S-0607 claim 24 | Stated as never restored, but the off position changes what a lift does to the units it takes: not clearly condition 3. |
| `drain_relink_strands_previous` | S-0608 | A building stays drained until the drainer dies; off ends a drain earlier: combat, condition 3. |
| `parasite_carrier_loss_strands_unit`, `parasite_suppressed_latch_strands_unit` | S-0614 | A parasite left off the map; rare removal causes, `unmeasured` for most; off changes what a weapon leaves behind: not clearly conditions 2 and 3. |
| `rocket_stalls_at_map_edge` | S-0506 | Keeps the last legal position; no stuck stated: condition 2. |
| `permanent_grant_freezes_suspension` | S-0512 claim 13 | A trigger action's side effect on a superweapon record, not a unit; maps with triggers only: condition 2. |

The data-triggered stalls are one group, judged by one policy (the main session's design): not
admitted, because the stuck state needs a value in the rules data that the spec itself does not
show in the stock rules or marks as a data error, so no play shows an effect, and the layer is
kept to bugs a game reaches by playing. They are `zero_load_period_stalls` (S-0402, a load
period of 0), `slave_negative_capacity_stalls` (S-0402, a negative capacity, data the spec treats as non-positive capacity),
`slow_walker_stall` (S-0208, speed 1; no off-position test either), `single_stage_buildup_never_completes`
(S-0406: "No stock data reaches this case"), `airstrike_empty_strike_stuck` (S-0519: "Stock data
has no designator whose elite strike size is 0"), `anim_fast_rate_freezes` (S-0109, a rate above
900), `cloak_zero_speed_stall` and `zero_rate_freezes_ambient`. A person who sets such a value in a
mod can still add the switch to the layer; each has an off-position test already.

**Results** (local lane; outputs in the git-ignored `local-lane/m8/c7-3/`, built from this branch on main 70eef44).
- *Default runs.* No simulation, importer or default changed, so every default run is main's. The reference M6c, M7a and M7b sweeps 1 to 16 were run on this branch to compare with the layer-on ones (below).
- *Layer-on sweeps 1 to 16* (`GLASNOST_PLAY_FIXES=1`): all three pass, no seed ends early. M6c and M7a print the same lines as the reference (smallest minimum 3 and 0; the M7a 0 is an informational counter that is 0 on the reference too). M7b differs from the reference in one seed, 8: mutator launches 2 (reference 3), infantry mutated 6 (8), drain counters 2/6/7/14 against 3/9/7/17, so the smallest drain minimum is 2 (reference 3; part 2's layer: 3), objects taken over median 16 (17). Isolated by switching each entry back on in turn for seed 8: only `absorber_stale_contact_kept` off changes it (the duel's absorbers); the other five entries leave seed 8 as the layer has it. No stall, no panic.
- *Quick lane with the layer* (seeds 7 to 14, maps 8, 2 and 0, Yuri as house 1 against Allied and against Soviet, 48 games): every game over with a winner (house 1 wins 36, house 0 wins 12), no panic, median length 36 560 ticks. The reference quick lane was not re-run in part 3 (about 4 minutes a game batch; parts 1 and 2 ran it); the layer-on run is the no-stall check the brief asks for.

**Version effects.** Only the layered `sim_rules_hash` changes (the layer file's content is in it). A save, replay, report or room made with part 2's layer matches neither the reference nor the new layered hash and is refused on the rules hash, as part 2 said. Reference files are unaffected. Wire 9, pathing report 2, desync bundle 3, save 83 and AI game format 45 are unchanged.

## Intake 53 (S-0108 r102, S-0113): two rendering switches judged

Ledger: [intake53-S-0107-S-0108-S-0113.md](intake53-S-0107-S-0108-S-0113.md). Both switches are
rules quirks read only by the client (no simulation code reads them).

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `ungranted_super_reads_clock` | S-0108 claims 11 and 18, Edge cases | admitted (off) | (1) The Edge case states the off position (a record that is not granted counts as not charged: *super-1* at the opening, claim 18's check waits for the grant), built in `mapview::live::charge_reading`. (2) The quirk text states a visible glitch: a building that opens with a record never granted, as at most set-up placements, "shows the charged *super-3* although nothing has charged". (3) Not a balance change: in our engine a superweapon building's slots are the client's (`mapview::slot_play`), whose creation draws come from the view's own animation clock and not the scenario stream, so the switch changes no simulation state and the sweeps cannot move. The spec's reason for making it a switch (S-0108 Edge cases: in the original the slot starts take creation draws on the scenario stream, and an ending animation can cost another object its turn) applies only once these slots move into the simulation, as the depot's have; the layer entry should be judged again then. Test: `layers::tests::play_fixes_sets_only_the_admitted_switches_off`; the switch's two positions: `capture_tests::an_ungranted_superweapon_record_is_read_by_its_clock`. |
| `rocking_limit_escape` | S-0113 claims 9 and 15, Edge cases | rejected | Condition 3: the off position decides whether repeated explosions carry a body past its limit and over, which claim 9 makes a destruction (a combat outcome), the reason `drain_relink_strands_previous` was rejected. Condition 2: the quirk text states that destruction, not a stuck unit, a give-up or a glitch, and no play note or request names it. (Our client does not yet destroy a body that rolls over: claim 9 is a simulation seam, so in our engine today the switch only changes how far a body tips.) |

**Version effects.** The layered `sim_rules_hash` changes (the layer file's content is in it), so
a save, replay, report or room made with the previous layer is refused on the rules hash, as for
earlier layer changes. Reference files are unaffected.

## Intake 55b (S-0314 r107): the depot-loss switch judged

Ledger: [intake55b-S-0314-S-0202-S-0601-S-0612-S-0614-M-0007-M-0008-M-0014.md](intake55b-S-0314-S-0202-S-0601-S-0612-S-0614-M-0007-M-0008-M-0014.md).
S-0314 r107 adds no quirk; it widens the text of `depot_loss_leaves_sleep` from a docked vehicle
to every contacted vehicle after its movement arrival, docked or not, in *sleep* or in *guard*.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `depot_loss_leaves_sleep` | S-0314 claim 22, Edge cases (r107) | rejected (not in the layer) | Condition 2: the quirk text states a vehicle that "neither fires nor reacts until ordered", a vehicle that waits for an order and moves on one, not a stuck unit, a give-up, a unit that never moves again or a glitch; no play note or request names it. Condition 3: the off position gives each arrived contact the sale's run-away notice, a scatter move that can carry a vehicle into or out of a fight, so it changes combat outcomes. Both positions are tested (`quirk_depot_loss_leaves_sleep_off`, tethered and not; `claim_22_arrived_vehicle_keeps_its_behaviour`). |

The layer file is unchanged, so the layered rules hash does not move.

## Intake 61 (S-0306 r53): two projectile switches judged

Ledger: [intake61-S-0607-S-0306.md](intake61-S-0607-S-0306.md).

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `switched_off_hover_lead` | S-0306 claim 3 (b), Edge cases | admitted (off) | (1) The Edge case states the off position (a switched-off vehicle is not led; the aim point is its reference point), built in `combat::projectile::led`. (2) The quirk text states a stuck attacker: "Attackers that keep it as their target can then fail to destroy it for the rest of the game" (the robot tank stall of M-0005, Q-2157 and Q-2158). (3) It acts only on a switched-off hovering target, a vehicle that neither travels nor fires; the off position aims at where it is, so it changes no balance between working units. Test: `layers::tests::play_fixes_sets_only_the_admitted_switches_off`; the two positions: `claims_s0306::claim3b_switched_off_hover_lead`. |
| `owned_shell_ground_pass` | S-0306 T1, Edge cases | rejected | Condition 3: the off position detonates every straight or arcing shot that has a firer below the terrain, in a building band or at a wall overlay, which changes the hits of every artillery and cannon shot (a broad combat change). Condition 2: the quirk text states shells that fly on below the ground, and the stuck attacker they can give is the hover lead's, which the admitted switch answers. |

**Version effects.** The layer file changes, so the layered `sim_rules_hash` moves: a save,
replay, report or room made with the previous layer is refused on the rules hash. Reference files
are unaffected.

## Intake 64 (S-0306 r57): the warp deck departure switch judged

Ledger: [intake64-S-0306.md](intake64-S-0306.md).

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `warp_deck_departure_airborne` | S-0306 claim 17, Edge cases | rejected | Condition 2: the quirk text states a classification, a warp member that departed from a high-bridge deck counted as airborne until it appears, and its effects on projectiles aimed at it: at a landing-check kill a steering shot is left without target and climbs, and an anti-air shot can detonate near the point the member left. None of these is a stuck unit, a give-up although an open way exists, a unit that never moves again or a visible glitch; no play note or request names it. Condition 3: the off position turns those shots' loss into a cell target on the deck, where they detonate and deal area damage, and lets anti-air shots fly on, so it changes combat outcomes, however rarely (a warp superweapon launched on a bridge deck while shots are in flight). Both positions are tested (`claims_s0306_r57::quirk_warp_deck_departure_airborne_on_*` and `_off_*`). |

The layer file is unchanged, so the layered rules hash does not move.

## Intake 66 (S-0213 r89): the click-area kind-test switch judged again

Ledger: [intake66-S-0213.md](intake66-S-0213.md).
S-0213 r89 adds no quirk. It adds one consequence to the off position of
`click_area_skips_kind_test`: no turreted building is left under *move* with an *attack* that
never starts (claim 16d, Q-2228). Intake 56 rejected the switch on condition 2, so it is judged
again.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `click_area_skips_kind_test` | S-0213 claim 16d, Edge cases (r89) | rejected (not in the layer) | Condition 2 is now met in part. With the switch on, a turreted building that joins a *move* point keeps *move* current for good, and it never fires again, which is a stuck unit. The person brings this about by pressing a marker with the defence selected while holding the route key, and no play note names it. Condition 3 fails. The off position does more than free that building: it refuses every route point for every aircraft and building. Aircraft routes and a non-turreted building's later *attack* both work in the original, so turning the switch off takes away orders a person can give and changes how aircraft can be used in play. A switch that covers only the stuck building would be a new rule, and no spec states one. Both positions are tested (`claim_16d_a_click_area_skips_the_kind_test_both_ways`). The building is tested on its own in `claims_s0213_r89::claim_16d_a_turreted_building_under_move_never_starts_a_later_attack`. |

The layer file is unchanged, so the layered rules hash does not move.
