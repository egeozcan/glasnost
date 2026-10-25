# Pathing and other fixes: ledger (M8 step C7)

Brief: `.claude/worktrees/M8-C7-FIXES-BRIEF.md`. Our own engine and client: the layer only sets switches whose off position an approved spec already states, so no simulation code changed, and the save format (76) and the AI game format (38) are unchanged.

## The layer

Our own rules patch layer, `data/layers/play-fixes.jsonc` (`glasnost_rules::layers`), shown to the person as *Pathing and other fixes*. It sets chosen quirk switches to the off position their spec states; the defaults in `quirks.rs` stay the original's (PLAN.md D6, D8). Decision of 2026-10-25 (m8-plan.md): on by default in the client's new-game setup; the measurement runs stay on the reference ruleset.

**Admission.** An entry is admitted only when (1) an approved spec states the off position and the engine implements it, (2) the on position reads as a bug in play (a unit stuck, giving up, never moving, a visible glitch), named by a play note or a request, and (3) it is not a balance change (D14). The file cites each entry's claim and what it answers; `layers::tests::an_unknown_switch_is_refused` and `play_fixes_sets_only_the_admitted_switches_off` keep the file to known switches and to these two.

| Switch (off) | Spec | Answers | Why |
|---|---|---|---|
| `corridor_retry_no_fallback` | S-0212 claim 13 and Edge cases | Pathing reports 1 and 2 (a4-play-notes.md), Q-1771 (a unit at a factory's exit), Q-1870 (a harvester on a refinery's docking cell) | On, a computation whose repeated coarse search fails gives up at once although an open way round exists (claim 23 row 2); off, one cell search without a corridor follows, with the same limits. |
| `corridor_split_probe` | S-0212 claim 13a and Edge cases | The same | On, the retries beside the buildings keep running corridors into them and the 5th cell search inside a corridor ends the computation (claim 13's cap); off, the sector reads as split, its connections close, the coarse search fails and the entry above's search without a corridor finds the way. |

The two only work together: with retry-off alone both synthetic shapes still fail at the cap, with split-off alone they fail after the second search (`claims_s0212_cap.rs`: `play_fixes_the_vehicle_on_the_docking_cell_finds_its_way`, `play_fixes_the_unit_at_the_factory_exit_finds_its_way`, `play_fixes_needs_both_switches`, and `play_fixes_change_nothing_where_the_first_search_finds_the_way`).

Considered and rejected: `group_spread_no_detour` (S-0213 leaves the off behaviour as a question for the lead, so condition 1 fails); `substitution_distance_flat_destination` (not clearly a bug in play, and it changes approaches broadly); `corridor_free_part_unusable_cells` (jumpjet infantry only, no play note); `path_search_10000_failure` and the other path switches (no play note or request names a bug). Q-1870 question 3, the second harvester circling a taken dock: S-0402 claims 10 and 10a are unchanged and no switch covers them; the layer removes the cause seen in the lead's reports (the first harvester stuck on the dock), not the circling.

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
