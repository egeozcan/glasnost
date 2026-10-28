# S-0212 r82: claim ledger, M8 step A3 intake 34

This intake takes in one spec revision, which answers three requests: Q-1751 (a house's threat
records), Q-1771 (a unit routed through its own base) and Q-1870 (a harvester leaving a
refinery's docking cell; also answered by S-0402, spec commit 6020b95, not taken in here).

- **S-0212 revision 82** (spec commit ceb99a2; the requests answered in b2aad3f and 75942db). It
  covers the whole diff against the revision our engine built (40568b7):
  - Terms: *exemption rectangle*, *threat block*, *threat record*, *threat amount*, *threat value*
    and *coarse threat* (new); the corridor exemption now names the exemption rectangle.
  - Claim 13: every building on the map gives the corridor exemption to its exemption rectangle,
    whatever its owner, from its placement until it leaves the map (a to-tile building's
    rectangle too: the original's bug, fixed with no switch); an aircraft gives it round its
    placement cell and each landing cell, not in flight; the sources add up. The coarse threat
    term at levels 1 and 2 (F5).
  - Claim 13a: the free part ignores claim 10's height rule
    (`quirks.corridor_free_part_ignores_height`).
  - Claim 16a (new): the threat records of a house, their spread, events, aircraft rule and
    rebuild, in which an *AI house* counts as computer-controlled.
  - Claims 17 and 18: the threat tests of the straightening passes read the records of claim 16a.
  - Claim 24: the repeated substitution on high ground (Q-1870), stated, no rule change.
  - F5 (the coarse threat at levels 1 and 2, seed blocks and quarters), F7 and F9 (the threat term
    in a sector's cost).
  - Constants: the threat block, the spread.
  - Edge cases: the base, docking-cell and short-line entries rewritten (the original walks out);
    a short strip of ground the unit cannot enter now carries `quirks.corridor_retry_no_fallback`
    and `quirks.corridor_split_probe`; a ledge and four new switches
    (`corridor_free_part_ignores_height`, `threat_rebuild_counts_friends`,
    `negative_threat_field`, `aircraft_threat_stays_put`).
  - Import mapping: `ThreatPosed` (building types too) and `[General] ThreatPerOccupant`
    (default 5).
  - Scenarios P12 (run T new, run S changed), P30 and P31 (new); Changes for other specs (S-0309,
    S-0603, S-0605, S-0412, the glossary's *AI house*).

The cross-specs it names were read as they stand: S-0406 claims 2 and 10 and Import mapping,
S-0508, S-0606, S-0412 claims 8 and 14, S-0414, S-0209 claim 5, S-0211 claim 11, S-0009 claims 1
and 38, S-0402 claims 3, 3a, 6, 10, 10a and 13, S-0215 claims 11a and 26, the glossary's *on the
map*, *alliance* and *AI house*. None was revised for this intake. Spec commits on origin/main not
taken in (later intakes): S-0207 r82, S-0402 r76 (34dd574), S-0417 r64.

**Code.**
- `threat` (sim, new module): the corridor exemption counts of claim 13 (buildings' exemption
  rectangles, aircraft at placement and landing, the old per-object exemption), and the records of
  claim 16a: `ThreatWorld::records` (per house, an ordered map of threat block to record), each
  object's `Track` (amount last added, block), the 3 × 3 spread with floor 0, the event hooks
  (enter and leave the map, a vehicle's or infantry unit's recorded cell crossing into another
  block, owner change, occupants entering or leaving, alliance made or broken) and the rebuild.
  Saved and hashed: save format 83, AI game format 45.
- `corridor`: `coarse_threat` (F5: level 0 none, level 1 the entered sector's block, level 2 the
  seed blocks and the quarters towards *d*), `SectorLevel::first` (each sector's first cell in row
  order), `threat_of` (the owner and *k* when *k* exceeds the binary64 value nearest 0.00001), the
  term *T* in F9's cost.
- `path`: the corridor searches pass the unit's threat; claims 17 and 18 test each entered cell's
  threat value × *k* (claim 17 under `quirks.flatten_threat_start_cell`).
- `econ::setup`: `HouseSetupState::ai` (an AI house, read by the rebuild); the game-start
  alliances rebuild every house that makes one, after the objects are placed.
- `econ::victory`, `econ::events`, `structures::garrison`, the locomotors (`drive`, `hover`,
  `teleport`, `walk`, `jump`): the hooks of claim 16a's event table.
- Rules: the four switches (`quirks.rs`), `ThreatPosed` on building types, `ThreatPerOccupant`
  (default 5).

**Design.**
- Breaking an alliance by an order rebuilds both houses' records when each allied the other, and
  the breaker's when the alliance was one-sided. The spec says "that house's records are rebuilt"
  for a player's order; our order breaks both directions (S-0412), so both houses rebuild.
- Map-trigger alliance actions are not implemented in our engine (no trigger action makes or ends
  an alliance yet), so claim 16a's two-house rebuild for them has no caller. Seam.
- A sandbox game's `GameSetup::alliances` (a test and harness device) are applied at load before
  the objects, with no rebuild.
- A change of bunkered vehicle has no hook; the building's amount follows it at its next event,
  as the spec says.
- An owner change of an object off the map has no track and so changes nothing, as the spec's row
  says (only an object on the map gets its current amount added).
- `released_docked` (an aircraft leaving its dock without a landing) is not treated as a landing.

| Claim or item | Status | Notes |
|---|---|---|
| Terms (exemption rectangle, threat block, record, amount, value, coarse threat) | tested | As claims 13, 16a and F5 below. |
| 13 (buildings' exemption rectangles, whatever the owner, from placement to leaving, the to-tile fix) | tested | `claim_13_a_buildings_exemption_rectangle` (a 5 × 4 rectangle from a 3 × 2 foundation, sources adding up, taken away at removal), `claim_13_and_16a_an_owner_change` (an owner change leaves it); `claims_s0212_cap.rs`: `claim_13_out_of_a_factory_exit_at_every_shift`, `claim_13_a_vehicle_on_a_docking_cell_walks_out`, `p31_the_unit_reaches_the_destination`, `play_fixes_change_nothing_in_the_reports_shapes`. |
| 13 (aircraft: placement and landing cells, not in flight) | tested | `aircraft_threat_stays_put` (both positions) reads the exemption with the record. |
| 13 (coarse threat term, *k* above the limit) | tested | `claim_13_a_unit_with_a_threat_coefficient_goes_round_threat` (*k* 0 and 0.000001 straight, *k* 1 bends round the enemy's blocks, no enemy straight). |
| 13 (backing, confidence, the retry table) | by construction | No rule change; the cap and retry tests of `claims_s0212.rs` still pass. |
| 13a (the free part ignores height; the switch) | tested | `a_ledge_reaches_the_limit_of_five_searches` (on), `corridor_free_part_ignores_height_off_goes_round_a_ledge` (off). |
| 16a (spread, blocks, the amount table) | tested | `claim_16a_an_enemy_vehicle_is_spread_over_3_by_3_blocks`, `claim_16a_a_large_building_counts_at_its_top_left_cell`, `claim_16a_an_armed_type_without_a_threat_field_adds_nothing`, `claim_16a_occupants_set_a_buildings_amount` (per-occupant 7 in the test). Bunkered vehicle row: by construction (no synthetic bunker in the test set). |
| 16a (the event table) | tested | `claim_16a_removal_takes_the_amount_away`, `claim_16a_a_moving_vehicle_moves_its_amount_between_blocks`, occupants and owner changes as above, `claim_16a_the_records_follow_the_objects_in_a_fight` (900 ticks of the testkit fight: the incremental records always equal a sum over the objects' tracks). |
| 16a (whose records; one-sided alliance) | tested | `claim_16a_a_one_sided_alliance`. |
| 16a (rebuild: friends, human, computer, AI house; break) | tested | `claim_16a_a_rebuild_counts_friends` (human, computer and AI-house variants), `threat_rebuild_counts_friends_off_leaves_friends_out`, `claim_16a_breaking_an_alliance_rebuilds_both_houses`; game-start rebuild by construction (`econ::setup::start_alliances`). |
| 16a (aircraft rule; the switch) | tested | `aircraft_threat_stays_put` (on: the amount stays at the placement block, removal at the recorded cell, or at (0, 0) when never landed; off: follows). |
| 16a (negative threat field; the switch) | tested | `negative_threat_field_on_leaves_a_record_higher` (both positions). |
| 17 (flattening's threat test, the start cell) | partly | Coded and cited (`path::straighten`); no test isolates it, because the gate games give no unit a coefficient and a synthetic shape whose flattening alone turns on threat was not built. |
| 18 (the count of threatened cells) | partly | As claim 17. |
| 23, 23a, 27a (a give-up on a docking cell) | by construction | Unchanged rules; the give-up row stands. |
| 24 (repeated substitution on high ground) | by construction | No rule change: derived from the existing rules. Intake 37 (S-0212 r83): the estimate's coarse search adds the threat term with the unit's own coefficient and owner (Q-1975 answered); test `claim_24_the_estimate_reads_the_units_threat`. |
| F5, F7, F9 | tested | As claim 13's coarse threat. F7's coefficient limit and 0.01: `consts` tests. |
| Constants (threat block, spread) | tested | As claim 16a. |
| Edge cases (base, docking cell, short line: the original walks out) | tested | As claim 13; `claims_s0603.rs`: `run_s_shifts_with_the_switch_on` (lines of 3 to 9 at all eight shifts), `lines_are_passed_with_corridor_retry_no_fallback_off`. |
| Edge cases (a short strip; `corridor_retry_no_fallback`, `corridor_split_probe`) | tested | `a_strip_fails_at_some_alignments`, `corridor_retry_no_fallback_off_goes_round_a_strip`, `corridor_split_probe_off_fails_at_fewer_alignments`. |
| Edge cases (four new switches) | tested | Each both ways, above; `every_spec_quirk_has_a_switch` lists no S-0212 quirk. |
| Import mapping (`ThreatPosed` on buildings, `ThreatPerOccupant` 5) | tested | Both keys were already imported for S-0309 (`sim_combat_tests.rs`); the building tests of claim 16a read a building type's field. The record's own default, 5 when the key is absent, is `record_threat_per_occupant` (cited); S-0309's acquisition keeps 0 (lead item). |
| Scenarios P12 (runs S, T), P30, P31 | not ours | Oracle scenarios; their shapes are mirrored in the tests above. |
| Changes for other specs (S-0309, S-0603, S-0605, S-0412, glossary) | not ours | Text for other specs. S-0603 claim 3's short line: `claims_s0603.rs` updated as above. |

**Play fixes (C7) re-judged.** See [c7-play-fixes.md](c7-play-fixes.md), *After intake 34*.

**Goldens.** All nine change from their first hashed line: the records are saved and hashed. The
building rectangles and aircraft cells of claim 13 (commit 933df00) change the behaviour of
seven gate games (the combat and scripted games keep theirs); the threat records (5ed2e54) change only the hashes: with the new fields left out of
the hash, every golden of that commit is identical to the one before it, since no gate unit has a
threat coefficient above the limit. `cargo xtask gate` passes.

**Sweeps and arena** (local lane; outputs in the git-ignored `local-lane/m8/a3-34/before` and
`after`, before from base eac7a26, after from 389a36a, this branch before its rebase onto
intake 33, which changed no golden). Every sweep exits 0 and no seed ends early.
- M6c 1 to 16: smallest minimum 3 (before 4), reference and with the layer.
- M7a 1 to 16: smallest minimum 1 (before 1); 16 seeds have a counter below 3 (before 15), as
  before only reported.
- M7b 1 to 16: smallest drain minimum 2 (before 3): seed 2's mutator launches are 2. The gate's
  seed 11 keeps every drain counter at 3 or more.
- With `GLASNOST_PLAY_FIXES=1` every sweep prints the same lines as the reference, line for
  line; before this intake the layer changed 23, 39 and 27 lines.
- Arena (`arena_matches_the_oracle`, which fails before and after on the known pairings): M-0001
  30 of 96 pairings failing (before 31; column-left KS 13, before 14), 151 of 172 stock outcomes
  reproduced; M-0002 13 of 24, 43 of 47; M-0003 2 of 35, 61 of 64, as before. 15, 1 and 11
  rows of `ours-M-0001.csv` to `ours-M-0003.csv` differ.

**Local lane** (outputs in the git-ignored `local-lane/m8/a3-34/lane` and `quick`).
- New scene `base-exit` in `crates/app/tests/m6b_lane.rs`: P31's layout and a harvester on a
  docking cell (pathing reports 1 and 2's shapes), each with *Pathing and other fixes* ticked and
  unticked. Every unit keeps its destination and finds its path on the first cell search, inside
  the corridor, in both positions; details in [c7-play-fixes.md](c7-play-fixes.md), *After
  intake 34*.
- Quick lane (`GLASNOST_AI_QUICK=1 glasnost-headless ai-sides 120000 7 <map>`, maps 8, 2 and 0,
  three pairs of sides each): all 9 games over with a winner and no panic, reference and with
  `GLASNOST_PLAY_FIXES=1`. Same lengths and winners in both but one game on map 0 (34 217 ticks
  without the layer, 34 205 with it, same winner).

**Requests.** Q-1975 (does a unit's threat coefficient steer the estimate's coarse search of
claim 24; answered by S-0212 r83 in intake 37). Q-1976 to Q-1979 unused.
