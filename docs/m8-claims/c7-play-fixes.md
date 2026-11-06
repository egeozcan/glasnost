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

## Intake 67 (S-0501 r65, S-0113 r17): the smoke switch judged

Ledger: [intake67-S-0501-S-0113.md](intake67-S-0501-S-0113.md).
S-0501 r65 and S-0113 r17 add no quirk. S-0501 r65 widens the off position of
`aircraft_smoke_scenario_draw`: with the switch off, no smoke chance is drawn and no smoke
animation is made (claim 37, Edge cases, Q-2237). The switch now governs a visible animation,
so it is judged.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `aircraft_smoke_scenario_draw` | S-0501 claim 37, Edge cases (r65) | rejected (not in the layer) | Condition 1 is met: the spec states the off position, and it is built. Condition 2 fails. The quirk text says only that the smoke chance draws from the scenario stream, which can change later game-state random outcomes. It names no stuck unit, no give-up and no visible glitch. The smoke itself is the original's look, not a glitch, and no play note or request names the switch. Condition 3 fails too. The off position stops the scenario draws, which moves every later game-state random outcome (combat included). It also takes away a visible effect that the original shows. Both positions are tested: `claims_s0501::claim_37_smoke_draw_both_ways` (the draw) and `claims_s0501_r65::claim_37_the_smoke_animation_follows_the_scenario_draw_switch_both_ways` (the animation). |

The layer file is unchanged, so the layered rules hash does not move.

## Intake 76 (S-0501 r93): two aircraft switches judged

Ledger: [intake76.md](intake76.md). S-0501 r93 adds two quirks, both new switches of
`quirks.rs` with the original's position as default.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `aircraft_edge_hold` | S-0501 claims 4, 10 and 51, Edge cases (r93, Q-2291) | admitted (off) | (1) The Edge case states the off position: an aircraft that may leave the map now is not pushed back, whatever its flags, so no candidate is tried and no draw is taken, and one that claim 10 would not remove before it leaves the legal map is removed in the update tick in which its position leaves it. It is built in `air::tick_flight` (the push-back is skipped; claim 10's legal-map removal in `leave_map_test` then applies). (2) The spec states a unit stuck for good: with the fly-back flag alone, a spy plane in *spy-plane overfly* "stays at the edge it met for the rest of the game, unless it is destroyed" (claim 51), and claim 10 gives the same hold to an aircraft with neither flag at an edge where the usable area's boundary lies on or beyond the legal map's. Q-2291 was filed from a seam trace of exactly that hold. (3) Not a balance change: it acts only on an aircraft that may leave the map now (claim 10: after its run, in *retreat*, in a spy-plane or paradrop exit), which is leaving play in both positions; the off position takes it out at the edge instead of holding it there, and drops the scenario draws that the hold takes in each tick its step would leave the map. Test: `layers::tests::play_fixes_sets_only_the_admitted_switches_off`; the two positions: `claims_s0501_r93_edge::quirk_aircraft_edge_hold_both_positions`. |
| `unarmed_pad_bound_hover` | S-0501 claim 50, Edge cases (r93, Q-2252) | rejected (not in the layer) | Condition 1 is met: the Edge case states the off position (only T2's last in-flight row changes: a pad search with a contact request, and *enter* when granted), built in `air::fall_back_rows`. Condition 2 fails. The aircraft is not stuck: it hovers near its F8 spot with *move* current and takes the next order, as the spec says ("until something else changes its state, such as an order"), the case `depot_loss_leaves_sleep` was rejected on (a unit that waits for an order and moves on one). The quirk text names no stuck unit, give-up or glitch, and no play note or request names the hover. Condition 3 fails too: the off position lands and docks the aircraft at a pad, where it is serviced (claim 26) and can be hit as a landed aircraft, where in the original it stays in the air at cruise height, so it changes which weapons reach it and its repair. Both positions are tested: `claims_s0501_r93::claim_50_an_unarmed_pad_bound_aircraft_hovers_near_its_pad` (on) and `quirk_unarmed_pad_bound_hover_off_docks_at_a_free_pad_and_hovers_at_a_taken_one` (off); `quirk_unarmed_pad_bound_hover_changes_nothing_for_a_type_that_is_not_landable` (both). |

**Version effects.** The layer file changes, so the layered `sim_rules_hash` moves: a save,
replay, report or room made with the previous layer is refused on the rules hash. Reference files
are unaffected.

## Intake 75 (S-0513 r86, S-0514 r12, S-0610 r12): two chrono switches judged

Ledger: [intake75.md](intake75.md). S-0513 r86 claim 39a adds two switches.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `warp_departure_keeps_beam` | S-0513 claim 39a, Edge cases; S-0514 claim 3 | rejected (not in the layer) | Condition 1 is met: the spec states the off position (a pending warp is one more lock refusal), and it is built. Condition 2 fails. The quirk text describes a member that warps away while erased, stops being erased at its departure and is still removed later; it names no stuck unit, no give-up and no visible glitch, and no play note or request names the switch. Condition 3 fails too: the off position stops a temporal shot in flight at a warp launch from erasing the member, which changes how chrono weapons and the chronosphere trade against each other. Both positions are tested: `claims_s0513_r86::claim39a_switch_off_a_pending_warp_is_one_more_lock_refusal` and `claim39a_switch_on_a_pending_warp_is_no_lock_refusal`, with the l1 to l5 tests for the on position. |
| `phase_in_beam_spares_infantry` | S-0513 claim 39a, Edge cases; S-0514 claim 17; S-0610 claim 5 | rejected (not in the layer) | Condition 1 is met: the off position (an infantry unit with an attached beam has its turns cut short while it phases in) is stated and built. Condition 2 fails: the spec gives the difference as which turns an infantry warp member takes during its arrival hold, a few ticks long; it names no stuck unit or glitch, and no play note or request names it. Condition 3: the off position changes a unit's turns, a rule change rather than a fix. Both positions are tested: `claims_s0513_r86::claim39a_an_infantry_member_with_a_beam_phases_in_as_one_with_no_beam_in_both_positions` and `claim39a_the_berserk_countdown_skips_the_cut_short_phasing_in_updates`. |

Intake 75 leaves the layer file unchanged, so it does not move the layered rules hash again.

## Intake 81 (S-0513 r89, S-0514 r14): the two chrono switches judged again

Ledger: [intake81.md](intake81.md). S-0513 r89 claim 39a adds the sparkles and drops in a warp member's turns and the firer's keeping a departed member. It adds no switch. Both of intake 75's switches now also cover the new rules.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `warp_departure_keeps_beam` | S-0513 claim 39a, Edge cases (*Sparkles and drops in the warp's turns* hold in both settings) | rejected (not in the layer), unchanged | The off position refuses the lock, so its member sparkles and drops as one with no beam. That is a presentation and target-clearing difference that follows from the refusal judged in intake 75. It names no stuck unit or glitch, and condition 3 still fails for the same reason. Both positions are tested: `claims_s0513_r89::claim39a_l6_a_member_with_a_beam_sparkles_once_per_turn_and_not_after_its_phasing_end`. |
| `phase_in_beam_spares_infantry` | S-0513 claim 39a (Drops: an infantry member with a beam drops while it phases in only with the switch off) | rejected (not in the layer), unchanged | The new difference is whether an infantry member with a beam keeps a given target and destination in its turns while it phases in. That is still a rule change over a few ticks with no glitch named. Both positions are tested: `claims_s0513_r89::claim39a_an_infantry_member_with_a_beam_drops_off_the_map_only_with_the_switch_off`. |

Intake 81 leaves the layer file unchanged.

## Intake 83 (S-0102 r41): the layer-change switch judged

Ledger: [intake83.md](intake83.md). S-0102 r41 adds no switch. Its Edge cases give the existing
`layer_change_on_events_only` a warp-member part. No earlier C7 entry judged this switch, which
is read only by the client (`mapview::quirks::ViewQuirks`).

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `layer_change_on_events_only` | S-0102 claim 4, Edge cases (r41: neither position moves a warp member between lists while the warp row applies; afterwards a member is treated as any object) | rejected (not in the layer) | Condition 1 is met: the Edge case states the off position (a sweep every tick moves an object whose evaluated layer differs from its list), and it is built in `Layers::evaluate_mobiles`. Condition 2 fails. The quirk text gives the difference as which list an object is in between claim 4's moments, and so its draw order against overlapping images of other layers. It names no stuck unit, no give-up and no glitch, and no play note or request names it. For a warp member the two positions agree until its own movement raises it after the hold. Condition 3 is met: only drawing order changes, and no simulation state. Both positions are tested: `layers::tests::claim4_switch_off_sweeps_every_tick`, `claim4_no_step_of_a_warp_is_a_moment`, `claim4_a_non_landable_aircraft_member_keeps_the_ground_list_while_it_flies`, `claim4_no_rule_moves_a_member_between_lists_while_the_warp_row_applies` and `live_tests::d49c_a_warp_member_keeps_its_place_before_equal_key_objects`. |

Intake 83 leaves the layer file unchanged.

## Intake 84 (S-0513 r91, S-0514 r16, S-0306 r65): the warp switches judged again

Ledger: [intake84.md](intake84.md). S-0513 r91 adds a building firer and a firer beyond range to claim 39a's *The firer*. S-0514 r16 orders the sparkle against the erasure update with no switch. S-0306 r65 extends the ground render layer of a warp member past its hold. None of them adds a switch or changes a switch's text.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `warp_departure_keeps_beam` | S-0513 claim 39a (*The firer*: a building firer and a firer beyond range, Q-2376) | rejected (not in the layer), unchanged | The off position refuses the lock at the launch, so no firer holds a departed member and the new firer rows do not arise. That follows from the refusal judged in intake 75. It names no stuck unit or glitch, and both positions are built and tested |
| `phase_in_beam_spares_infantry` | S-0513 claim 39a; S-0514 claim 18 | rejected (not in the layer), unchanged | It decides only which turns an infantry member with a beam has cut short while it phases in. The sparkle's new order against the erasure update is the same in both positions |
| `warp_deck_departure_airborne` | S-0306 claim 17 | rejected, unchanged | r65 restates the paragraph that cites it without changing it. The deck count and its effects on projectiles are as judged before |
| `layer_change_on_events_only` | S-0102 claim 4, Edge cases; S-0306 r65 claim 17 | rejected (not in the layer), unchanged | Intake 83's verdict stands. The kept ground layer of a non-landable aircraft member after its hold is the switch-on position's own claim 4 rule. The drawing reads `warp::in_warp_row`, so with the switch off the sweep still moves such a member once the warp row has ended. Both positions are tested with the kept layer set: `layers::tests::claim4_a_non_landable_aircraft_member_keeps_the_ground_list_while_it_flies` |

Intake 84 leaves the layer file unchanged.

## Intake 85 (S-0214 r78): the reason-of-the-moment cursor switch judged

Ledger: [intake85.md](intake85.md). S-0214 r78 adds one switch, read by the attack rule's part
(e) (`targeting::attack_rule`). `phase_in_beam_spares_infantry`, which the Edge case names, is
S-0513's and is judged with intake 84.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `moment_reason_attack_cursor` | S-0214 claim 6 (e), Edge cases (*Attack offered over a reason of the moment*) | rejected (not in the layer) | Condition 1 is met: the Edge case states the off position (part (e) asks only whether a reason that gives *illegal* holds, at any rank), and it is built in `check::illegal_at_any_rank`. Which answers it passes over beyond those the text names is under Q-2426, with an interim. Condition 2 fails. The quirk text gives the on position's effect as an attack offered, and an order accepted, over a target the actor will never fire at and drops once its reason of the moment ends. It names no stuck unit, no give-up and no glitch, and no play note or request names it. The off position takes away an attack order the original accepts. Condition 3 is doubtful as well: it changes which orders a person's click gives, and the own AI's, since the AI clicks through the same rule. Both positions are tested: `claims_s0214_r78::t4f_the_countdown_offers_the_attack_over_a_target_its_warhead_cannot_affect`, `out_of_ammunition_and_cloaked_hide_the_zero_versus_test`, `t4e_a_phasing_in_actor_is_offered_the_attack_over_a_target_in_the_air_in_range`, `a_falling_vehicle_that_is_phasing_in_holds_part_e`, `the_land_row_outranks_the_countdown_but_not_the_early_rearming_answers`, `a_pending_delayed_shot_hides_the_zero_versus_test`, and `claims_s0214_combat::claim_6e_switch_off_reloading_neither_gives_nor_takes_the_attack`. |

Intake 85 leaves the layer file unchanged.

## Intake 88 (S-0209 r55): the unhittable-obstacle switch judged

Ledger: [intake88.md](intake88.md). S-0209 r55 adds one switch, read by the entry outcome of rows
WP13, WP24 and WP28 (`occupancy::enemy_obstacle`), so by the step and the path search alike.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `walker_attacks_unhittable_obstacle` | S-0209 claims 4 and 9, Edge cases (r55, Q-2396) | admitted (off) | (1) The Edge case states the off position: each of the three rows gives *blocked* when the firing check without the range test, for the weapon the walker would use against the occupant (S-0308), answers *illegal*, so the path search does not consider that step (S-0212 claim 8) and the walker routes round the occupant or reacts to *blocked* as claim 7 gives. It is built in `occupancy::enemy_obstacle`. (2) The on position is a stuck unit that a request names. Q-2396 was filed from seam trace 3 of exactly this case: in an AI quick-lane game an infantry unit on an attack move stood in front of an enemy building that its weapon cannot fire at for about 1,260 ticks. The spec's own Edge case states the effect. Under a plain move the walker "rest[s] in front of the occupant without its move", so it gives up its order although a way round may exist (the path runs through the occupant whenever every detour costs more than the *enemy obstacle* step, `a_long_detour_is_taken_only_with_the_switch_off`). Under an attack move it stays in the take-and-drop cycle for as long as it is engaged at each fall-back. This differs from `depot_loss_leaves_sleep` and `unarmed_pad_bound_hover`, where the unit is idle and the order it waits for is the next one. Here the unit holds an order it never carries out. (3) Not a balance change, on one weighed exception. In both positions the walker never hurts the occupant, since the firing check answers *illegal*. The off position changes only whether it stops in front of the occupant or goes round it. The exception is S-0209 claim 9's charges case. On, a walker with explosive charges whose path leads through a building that accepts charges and that its weapon cannot fire at can reach *sabotage* through the take, when *sabotage* is current at the next check tick. Off, it routes round and never plants a charge this way. That is an unordered demolition by a walker under a move order, at a timing the spec leaves `unmeasured`. An ordered sabotage (S-0215 claim 23 on an *attack* or *enter* the player gives) is the same in both positions. We judge it incidental and admit the switch. Tests: `layers::tests::play_fixes_sets_only_the_admitted_switches_off`. The two positions: `claims_s0209_r55::the_three_rows_give_enemy_obstacle_to_a_walker_that_cannot_fire_at_the_occupant` (on), `switch_off_the_three_rows_give_blocked_for_an_occupant_the_walker_cannot_fire_at` (off), `an_occupant_the_walker_can_fire_at_gives_enemy_obstacle_in_both_positions`, `a_long_detour_is_taken_only_with_the_switch_off`, `switch_off_a_walker_with_no_way_round_never_takes_the_occupant` and `switch_off_a_charges_walker_never_takes_the_building`. |

**Version effects.** The layer file changes, so the layered `sim_rules_hash` moves: a save,
replay, report or room made with the previous layer is refused on the rules hash. Reference files
are unaffected.

## Intake 90 (S-0214 r80): the reason-of-the-moment cursor switch judged again

Ledger: [intake90.md](intake90.md). S-0214 r80 answers Q-2426: with the switch off, part (e)
passes over every answer that is not *illegal* (the interim intake 85 built), and the off position
is a departure by decision. No switch is added and the switch's default is unchanged.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `moment_reason_attack_cursor` | S-0214 claim 6 (e), Edge cases (r80, Q-2426 answered) | rejected (not in the layer), unchanged | Condition 1 is met in full now: the Edge case names every answer the off position passes over (*busy*, every *rearming* of ranks 4 and 5 with the grip and magnetic-beam answers, every *impossible*, out of ammunition, *cloaked*, *must deploy* and the lower ranks) and the two stops (the empty slot, the range test), and `check::illegal_at_any_rank` builds exactly that. Condition 2 still fails: r80 adds no stuck unit, give-up or glitch to the on position, and no play note or request names one. The four cases it adds (a lifter over its gripped unit in the air, a hidden target a later test rules illegal, a launching vehicle, a deploy-to-fire healer on a failing cell) are, in the on position, further attack orders accepted over a target the actor never fires at. The off position takes those orders away. Condition 3 stays doubtful for the reason intake 85 gives: it changes which orders a click gives, the own AI's included. Both positions of the four new cases: `claims_s0214_r80::off_a_lifter_over_its_gripped_unit_in_the_air_fails_part_e`, `off_a_hidden_target_that_a_later_test_rules_illegal_fails_part_e`, `off_a_launching_vehicle_over_an_illegal_target_fails_part_e`, `off_a_deploy_to_fire_healer_on_a_failing_cell_fails_part_e`; the two stops, off: `off_the_two_stops_remain`. Intake 85's tests stand. |

Intake 90 leaves the layer file unchanged.

## Intake 89 (S-0513 r108, S-0309 r113, S-0306 r72): the warp switches judged again

Ledger: [intake89.md](intake89.md). S-0513 r108 answers Q-2416, Q-2418 and Q-2419 and adds SW12
parts l7a, l7b, l9 and l10, with their off-position text for `quirks.warp_departure_keeps_beam`.
S-0309 r113 restates claim 27 for those answers. S-0306 r72 answers Q-2417 and names
`quirks.layer_change_on_events_only` for a target search. No revision adds a switch.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `warp_departure_keeps_beam` | S-0513 claim 39a (*The firer*, Q-2416, Q-2418), SW12 parts l7a to l10 | rejected (not in the layer), unchanged | The new text for the off position follows from the refusal at the launch that intake 75 judged: (l7a) is as in the original, (l7b) and (l9) have no beam and judge nothing, (l10) has no warp. It names no stuck unit or glitch. Both positions are tested where the scenario judges both: `claims_s0513_r89::r108_claim39a_sw12_l7a_a_building_drops_the_member_at_its_processing_before_the_lock` (on and off) |
| `layer_change_on_events_only` | S-0102 claim 4, Edge cases; S-0306 r72 claim 17 (*An aircraft that cannot land, in flight*) | rejected (not in the layer), unchanged; condition 3 judged again | Condition 2 still fails: no play note or request names it, and the claim 17 case (a flying aircraft that cannot land still found by a ground-layer pass) names no stuck unit or glitch. Condition 3 no longer holds as intake 83 gave it. With the switch off the sweep moves such an aircraft out of the ground list, so S-0309 claim 7's ground-layer pass stops considering it, which changes which units find it as a target. In our engine the switch is a client drawing setting the simulation cannot read, so the simulation always takes the on position (Q-2467, seam). The drawing's positions are tested as before (`layers::tests::claim4_switch_off_sweeps_every_tick`, `claim4_a_non_landable_aircraft_member_keeps_the_ground_list_while_it_flies`); the simulation's on position by `claims_s0306_r62::r72_claim17_p46_part_a_the_ground_layer_pass_considers_a_flying_aircraft_that_cannot_land` |

Intake 89 leaves the layer file unchanged.

## Intake 92 (S-0501 r110): two aircraft switches judged

Ledger: [intake92.md](intake92.md). S-0501 r110 answers Q-2436, Q-2437 and Q-2438 and adds two
quirks, both new switches of `quirks.rs` with the original's position as default.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `aircraft_grounded_enter_stall` | S-0501 claim 24, Edge cases (r110, Q-2436) | admitted (off) | (1) The Edge case states the off position: every aircraft in *enter*'s approaching stage that holds a contact and is on the ground and not landing is moved on to the docking stage as a pad-bound one is, so it reports entry at its next processing and is serviced as claim 24 gives. It is built in `air::enter` (the approaching stage's move on). (2) The spec states a unit stuck: one that is not pad-bound "stays in *enter* with its contact, returning 3 ticks at each processing, and is never serviced (claim 26)", and only an order, a queued behaviour or the end of its contact ends that stay. As with `walker_attacks_unhittable_obstacle`, the unit holds an order it never carries out (its *enter*, from a dock order or claim 18's pad search), not an idle state that waits for the next order. (3) Not a balance change: the off position gives that aircraft the service a pad-bound aircraft in the same state already gets, at the pad it holds a contact with, by the service rules of claim 26 unchanged; it changes nothing for an aircraft that lands at its pad, which is every aircraft that docks from flight. Tests: `layers::tests::play_fixes_sets_only_the_admitted_switches_off`; the two positions: `claims_s0501_r110::claim_24_grounded_in_contact_stall_switch_both_positions`. |
| `aircraft_ready_mark_outlives_enter` | S-0501 claim 31, Edge cases (r110, Q-2438) | admitted (off) | (1) The Edge case states the off position: the ready mark that *enter*'s move on to the docking stage clears holds the aircraft not ready only while *enter* is current; any change out of *enter*, forced or from the queue, leaves it set, so a behaviour queued after that change becomes current at claim 31's next point unless another of claim 31's exclusions holds it. It is built in `air::ready` and `air::ready_mark_after_enter` (called at the start of the aircraft's behaviour turn and before claim 31's third point). (2) The spec states a unit that does not take orders: an aircraft forced out of *enter* before a processing sets the mark again "stays not ready in that behaviour", so a queued behaviour or an order waits, and under a forced *guard* below cruise height "for as long as the aircraft keeps that height" (claim 31), with nothing in that state that sets the mark again. An order that is taken and never carried out is the glitch condition 2 names. (3) Not a balance change: the off position changes only when a behaviour the aircraft already has queued becomes current, by the same rules of claim 31 as for every other aircraft; no weapon, cost, speed or service changes. Tests: `layers::tests::play_fixes_sets_only_the_admitted_switches_off`; the two positions: `claims_s0501_r110::claim_31_the_ready_mark_after_a_forced_change_switch_both_positions`. |

**Version effects.** The layer file changes, so the layered `sim_rules_hash` moves: a save,
replay, report or room made with the previous layer is refused on the rules hash. Reference files
are unaffected.

## Intake 94 (S-0306 r75, S-0513 r115, S-0309 r116, S-0501 r111, S-0102 r43): the layer-change switch judged again

Ledger: [intake94.md](intake94.md). S-0306 r75 answers Q-2467: `quirks.layer_change_on_events_only`
is a drawing setting with no consequence in the simulation. S-0102 r43 and S-0501 r111 claim 48
(a) say the same. S-0513 r115 and S-0309 r116 answer Q-2466 and Q-2468 and name no switch. No
revision adds a switch or changes a switch's default.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `layer_change_on_events_only` | S-0102 claim 4, Edge cases (r43: a drawing setting each client may set for itself); S-0306 r75 claim 17; S-0501 r111 claim 48 (a) | rejected (not in the layer), unchanged; condition 3 holds again | Condition 2 still fails: no play note or request names a stuck unit or glitch in either position. Condition 3 holds again, as intake 83 judged it before intake 89 doubted it. The off position moves objects only between the lists the drawing reads. S-0309 claim 7's ground-layer pass reads the on position's membership in both positions, so the switch changes draw order and nothing else. In our engine this was already so: the switch lives in `mapview::quirks::ViewQuirks`, `Layers` reads the world by shared reference, and the pass reads `air::kept_filed` and `air::ground_filed`. Intake 89's seam (Q-2467) is closed with no code change. Tested: `layers::tests::claim4_a_non_landable_aircraft_member_keeps_the_ground_list_while_it_flies` now checks, in both positions, that the drawing's sweep leaves `save::state_hash` unchanged and the aircraft still filed for the pass. The simulation side is tested by `claims_s0306_r62::r72_claim17_p46_part_a_the_ground_layer_pass_considers_a_flying_aircraft_that_cannot_land`, in runs 1 and 2. |
| `warp_departure_keeps_beam` | S-0513 claim 39a (*The firer*, r115, Q-2468), SW12 parts l9 and l9a | rejected (not in the layer), unchanged | r115 changes no text of the off position. With the switch off the lock is refused at the launch, so no firer holds a beam on a departed member, and the new rows have nothing to release. That is intake 75's refusal. It names no stuck unit or glitch. |

Intake 94 leaves the layer file unchanged.

## Intake 98 (S-0306 r80, S-0301 r73): the hold-end forget switch judged

Ledger: [intake98.md](intake98.md). S-0306 r80 answers Q-2526 and adds one quirk, a new switch of
`quirks.rs` with the original's position as default. S-0301 r73 answers Q-2567 and names no
switch.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `warp_hold_end_forgets_destination` | S-0306 claim 17, Edge cases (r80, Q-2526) | rejected (not in the layer) | Condition 1 holds: the Edge case states the off position (no step forgets a destination after the member's behaviour processing in the tick in which its hold ends, so that destination holds and the aircraft takes off towards it), and it is built in `air::hold_end_forget`, called from the object turn in `tick.rs` after the behaviour processing. Condition 2 fails: on, the aircraft stays at height 0 only until its next *retreat* processing, at most 3 ticks later (S-0501 claim 34's delay), where its first destination that holds comes and it takes off. That is a short delay, not a unit stuck, a give-up or a visible glitch, and no play note or request names it. Tests: both positions in `claims_s0306_r62::r80_quirk_warp_hold_end_forgets_destination_both_positions` (on: no destination at the end of tick *h*, height 0, a destination on the arrival edge at *h* + 3 and the take-off by *h* + 4; off: the destination at the end of *h* and the take-off by *h* + 1). |

Intake 98 leaves the layer file unchanged.

## Intake 102 (S-0501 r132): three aircraft switches judged

Ledger: [intake102.md](intake102.md). S-0501 r132 answers Q-2576 to Q-2579 and adds three quirks,
all new switches of `quirks.rs` with the original's position as default.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `aircraft_descent_touchdown_untested` | S-0501 claim 7, Edge cases (r132) | rejected (not in the layer) | Condition 1 holds: the Edge case states the off position (every touchdown is tested and, when accepted, has the accepted touchdown's effects, so claim 7's refusal, claim 24's refused touchdown in contact and claim 45's refused touchdown at the edge cell arise wherever F7 refuses the cell). It is built in `air::tick_flight` (the touchdown block: the r120 tested touchdown). Condition 2 fails. On, an aircraft can rest in a cell that F7 would refuse, beside an occupant, and one that touches down where its forget test keeps its destination flies back to it (claim 7); the spec names no unit stuck and no play note or request names a bug. The off position brings what on avoids: a refused touchdown that forgot its destination leaves the aircraft "with nothing to move it", 10 leptons above the refused cell "until it gets a destination" (claim 7), and claim 45's hold above the edge cell. Admitting it would trade a resting place for the hold the spec states as a stuck unit. Tests: on, `claims_s0501_r132::claim_7_an_untested_touchdown_in_an_occupied_cell_rests_there` and `claim_7_an_untested_touchdown_that_keeps_the_destination_slides_then_climbs`; off, `claim_7_off_a_tested_touchdown_on_open_ground_forgets_the_destination_in_any_case`, `claim_7_a_refusal_that_kept_the_destination_drops_back_to_the_ground` and the r120 tests, which set it off. |
| `aircraft_refused_kept_drop` | S-0501 claim 7, Edge cases (r132, Q-2578) | rejected (not in the layer) | Condition 1 holds: the Edge case states the off position (the refusal, or the untested touchdown, that kept the destination gives the aircraft the held height of ordinary flight, so it climbs from its next movement update on and guidance steers it back to its flight point), built in `air::tick_flight` (the take-off's end and the held height) and `air::untested_kept`. Condition 2 fails. The quirk text states that the aircraft "slides on along its heading with no steering until its target speed fraction reaches 1, and only then climbs and flies back": an effect that ends by itself, which the text does not call a stuck unit, a give-up or a glitch, and no play note or request names it. That reading that the slide looks wrong is ours, not the spec's; the precedents `warp_hold_end_forgets_destination` and `unarmed_pad_bound_hover` were rejected for an effect that recovers by itself. Tests: the two positions, `claims_s0501_r132::claim_7_an_untested_touchdown_that_keeps_the_destination_slides_then_climbs` and `claim_7_a_refusal_that_kept_the_destination_drops_back_to_the_ground`; AC31 part h (1)'s pad end to end in both positions, `ac31h1_after_an_untested_touchdown_beside_the_docking_position_it_comes_to_rest`. |
| `aircraft_offmap_stop_start` | S-0501 claim 6, Edge cases (r132, Q-2577) | admitted (off) | (1) The Edge case states the off position: the selection starts from the cell the aircraft is over, as anywhere else. It is built in `air::stop_start`, called from `air::given_no_destination`. (2) The spec states an order carried out wrong: on, an aircraft stopped over a cell outside the usable area starts claim 40's selection from a local point with one coordinate moved to 1 or one less than the local area's size, rows counted by its height although they run to twice that, a start that "can lie far from the aircraft and can still be outside the usable area" (Edge cases), so a stop sends it flying there; an order carried out as a flight away is the glitch condition 2 names, as for `aircraft_ready_mark_outlives_enter` (an order taken and never carried out). (3) Not a balance change: the off position gives that aircraft the selection every other aircraft given no destination gets, by claim 40's search unchanged. Tests: `layers::tests::play_fixes_sets_only_the_admitted_switches_off`; the two positions: `claims_s0501_r132::claim_6_given_no_destination_outside_the_usable_area_starts_from_the_local_edge`. |

**Version effects.** The layer file changes, so the layered `sim_rules_hash` moves: a save,
replay, report or room made with the previous layer is refused on the rules hash. Reference files
are unaffected.

## A4 round 8 (note 50): the half-reload switch judged

Ledger: [a4-play-notes.md](a4-play-notes.md), round 8, note 50. The lead's note says that a
pad aircraft ordered to attack before it has reloaded takes off and lands again, and that
this delays its reload. The specs give that behaviour, as the note's row says. No earlier C7
entry judged this switch.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `aircraft_half_reload_with_target` | S-0501 claim 28, Edge cases (*Half reload with a target*) | rejected (not in the layer) | Condition 1 holds: the Edge case states the off position (the aircraft is reloaded to full whatever its target, and takes off at full ammunition), built in the pad's service step in `air.rs`. Condition 2 is met in part. For a type with one round, ⌊*A* / 2⌋ is 0. On, an order given on the pad then sends the aircraft out at once with no round, it turns back at once (claim 17), and it lands again. That is note 50, and in the lane it put the full reload off by 270 ticks. Condition 3 fails. The off position changes every aircraft with two rounds or more as well: with a target it now waits on its pad for full ammunition, where the original sends it out at half. That changes how soon air attacks go and how much they carry, which is a balance change (D14), the reason `aircraft_unit_step_stall` was rejected. A switch that held only an aircraft with no round would be a new rule, and no spec states one; the lead may rule on it as our own design. Tests: the two positions, `claims_s0501::claim_28_half_reload_both_ways`. |

The layer file is unchanged.

## Intake 103 (S-0212 r97, S-0417 r79, M-0023): no new switch; the own pathing improvements re-read

Ledger: [intake103.md](intake103.md). Neither revision adds, renames or rewrites a quirk, so no
switch is added and no default changes. S-0212 r96 only rewraps the lines that name
`quirks.detour_mirrored_reference` and `quirks.path_refresh_keeps_old_step` (P9's falsifiers; the
follower run 3 falsifier is dropped, which is claim 26's and unbuilt); both keep their defaults.

r97's claim 21 ends a driving or walking unit's running retry delay when its destination is changed
to a different one or cleared. Whether that changes what each of our own pathing improvements
fixes:

| Switch (off) | Changed by r97? | Reason |
|---|---|---|
| `path_through_stalled_blockers` | no | It acts on stalled units, which hold their destinations through the deadlock; a delay that runs with a kept destination runs as before. |
| `unreachable_move_dropped` | no | Its try towards the nearest reachable cell runs inside the failed computation itself (`path::compute_untraced`), before the failure starts or clears any delay. What it fixes, a move to a cell the unit cannot reach doing nothing, is the same: on, r97 only means the cleared destination leaves no delay, and the unit still stands where it was. |
| `group_spread_no_detour` | no | It chooses each member's cell at the order; the members crowd the fallback cell in the same way. A member whose delay ran now searches for its new cell at once in both positions. |
| `rally_point_shared` | no | Released units get their rally move at the release, with no delay running. |
| `deadlock_no_yield` | no | The yield already ended the yielding unit's delay (`retry_at = t`) as it gave it the refuge cell; r97's rule now ends it too, so the explicit reset has no further effect. |
| `harvest_cell_shared` | no | It chooses which field cell a harvester heads for. A different field cell now ends a running delay in both positions; the docking offer keeps its delay (S-0402 claim 10a). |

Intake 103 leaves the layer file unchanged.

## Intake 104 (S-0501 r134, S-0302 r43): three aircraft switches judged again

Ledger: [intake104.md](intake104.md). S-0501 r134 answers Q-2646 to Q-2650 and adds no quirk,
but it changes the scope of three switches. S-0302 r43 answers Q-2626 and names no switch. No
default changes.

| Switch | Spec | Verdict | Reason |
|---|---|---|---|
| `aircraft_descent_touchdown_untested` | S-0501 claim 7, Edge cases (r134, Q-2646) | rejected (not in the layer), unchanged | r134 takes one touchdown out of the switch: the touchdown in the update of the landing check's refusal, which that refusal's take-off has already ended, is untested in both positions, and its forget test is made against the free cell the refusal gave as flight point. That removes a case from the off position and adds nothing to the on position, so intake 102's reasons stand: condition 2 still fails (no stuck unit, give-up or glitch stated, no play note or request), and the off position still brings the hold 10 leptons above a refused cell that the spec states as a unit with nothing to move it. Built in `air::check_refused` (the refusing update's touchdown, the same in both positions). Tests, both positions on the new case: `claims_s0501_r134::claim_7_the_touchdown_in_a_refusing_update_is_untested_and_climbs_with_no_slide` and `claim_7_a_refusing_update_over_a_free_cell_rests_there`; the earlier tests unchanged. |
| `aircraft_refused_kept_drop` | S-0501 claim 7, Edge cases (r134, Q-2646) | rejected (not in the layer), unchanged | r134 takes the same touchdown out of this switch: the refusing update's take-off has given the aircraft the held height of ordinary flight, so it climbs from its next update in both positions and never slides. The on position's slide, the effect intake 102 judged, is unchanged and still ends by itself: condition 2 fails as before. Built in `air::refusal_touchdown_kept` (the held height kept at cruise height, no slide). Tests, both positions on the new case: `claims_s0501_r134::claim_7_the_touchdown_in_a_refusing_update_is_untested_and_climbs_with_no_slide`; the earlier tests unchanged. |
| `aircraft_corner_sector_untracked` | S-0501 claim 47 (c), Edge cases (r134, Q-2647) | rejected (not in the layer); no earlier C7 entry judged it | r134 widens the on position to one more state: an aircraft that an untested touchdown left holding its flight point, on the ground as in flight, is not tracked while its cell lies in sector (0, 0). Condition 1 holds: the Edge case states the off position (at its first movement update with a positive carried fraction it joins the end of sector (0, 0)'s list like any other), built in `air::sector_update`. Condition 2 fails: the text states that neither area damage nor the airborne scan finds such an aircraft in that sector, which is not a unit stuck, a give-up or a visible glitch, and no play note or request names it. Condition 3 fails too: the off position decides which aircraft area damage and the airborne scan can reach, a combat change, as for the rebound switches in part 3. Tests, both positions: `claims_s0501_r134::claim_47_the_corner_switch_applies_to_a_sliding_aircraft` (new state) and intake 16's `claims_s0501_tracking::claim_47_corner_sector_quirk_both_ways`. |

Intake 104 leaves the layer file unchanged.
