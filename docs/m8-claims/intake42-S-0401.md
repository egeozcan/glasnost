# S-0401 r17 and M-0013: claim ledger, M8 step A3 intake 42

This intake takes in one spec revision and one measurement record:

- **S-0401 revision 17** (spec commit a7aa0d9). Our engine built revision 10 (1a1ee2b); the
  earlier intakes of revisions 5 to 9 are in [../m5-claims/S-0401.md](../m5-claims/S-0401.md).
- **M-0013** (measurement commit a0c3411), S-0401 scenario E8: which cells receive spreading ore.
  Its verdict against revision 10 is *contradicts* (tree cells); revision 17 answers it.

No open request is answered by r17 (its `requests` stay Q-0046). The specs it names were read as
they stand: S-0319 r26 claim 10 (a tree's cell takes no overlay from a bouncing animation's offer,
built at intake 33), S-0317 claim 12, S-0520 claim 24, S-0010 claim 16, S-0009 claims 4 and 5,
S-0211 claim 2. S-0212 r87 and S-0417 r68, also new on main, are not taken in here.

## What r17 changes

The rule change is one case: **a germinable cell that holds a terrain object** (a tree).

- Claim 11: such a cell gets no overlay. The overlay draw and the growth-list offer happen as for
  any new cell, the cell holds no resource afterwards and stays germinable. A spread step that
  chose it has used its attempt and made nothing (claim 9); an ore mine's step that chose it makes
  nothing (claim 15). New switch `quirks.spread_step_lost_on_terrain_object` (default on, the
  original). Off: a cell holding a terrain object is not germinable, so a spread step's scan
  passes over it and it does not count in *c*.
- Claim 10: with the switch on a tree's cell counts in *c* and can be the first germinable
  neighbour; S-0319's offer treats such a cell the same way.
- Claim 7: the growth list is offered such a cell; its entry is later dropped when taken.
- Claims 6, 9, 11, 15, 19: backing from M-0013 only (no rule change). Claim 19's open question
  (whether the first game's first updates fall in tick 0) is settled as built.
- Terms (*germinable cell*), State and ordering (a new row), Edge cases (the switch), E5 and E8
  (parts a, b, c), E11 and *Changes for other specs* (S-0319 claim 10's wording; S-0317 and
  S-0520 need none): text.

## Code

- `rules::quirks`: `spread_step_lost_on_terrain_object` (default on) and the schema. This turns
  `every_spec_quirk_has_a_switch` green again (main was red on it).
- `econ::resource::new_resource_cell` (every producer: spread steps, ore mines, S-0317's pieces,
  S-0520's crates): a cell with a terrain object in its ground layer takes the overlay draw and
  the growth-list offer and is left unchanged. This holds in both positions of the switch; with
  it off no such cell is germinable, so none reaches this point.
- `econ::resource::germinable`: with the switch off, a cell holding a terrain object is not
  germinable. This reaches S-0319's bouncing offer too (it then takes no draws at a tree's cell).
- No new saved or hashed state: save format 83 and AI game format 45 are unchanged.

**Design** (r17 leaves it open):
- The stage of a tree's cell is not set: claim 11 says the cell holds no resource afterwards, and
  the Open questions say a germinable cell keeps its overlay data value. So the cell is
  unchanged; only the next offer of the same cell can see the value (it decides whether the
  growth-list offer takes a draw, a value of 11 or more from map data being the only case where
  it would differ). S-0319's offer does the same.
- A "terrain object" is any terrain object listed in the cell's ground layer, as in S-0319's
  offer (intake 33). Claim 10 looks only at the first terrain object for the ore-mine test; a cell
  whose first terrain object is not an ore mine and that holds one further back is germinable and
  gets nothing.

**Play-fixes layer (C7).** Not named. The on position loses spread attempts next to trees: an
ore cell whose only free neighbours are two or more trees keeps returning to the front of its
spread list with key 0 and spends attempts there (as `quirks.spread_attempts_count_failures`
already does for a cell with a unit on it). It is not a unit stuck, a give-up or a glitch, and
the off position raises ore income near trees, which is closer to balance (D14). The layer's
admission rule (`data/layers/play-fixes.jsonc`) is not met.

## M-0013, fact by fact

Ours is the engine after this intake; the test is `m0013_which_cells_receive_spreading_ore`
(synthetic pens: a stage-11 centre, walls on the other seven neighbours, our own spread field
and seeds, both switch positions). The record's numbers stay local; only the verdict is here.

| M-0013 fact | ours before (r10) | ours now | verdict |
|---|---|---|---|
| free control cells receive ore in every run | yes | yes | matches |
| cells with an infantry unit receive ore | yes (infantry does not prevent it) | yes | matches |
| tree cells (two tree types) never receive ore | **no**: a tree's cell was germinable and got ore | never, in both switch positions | departed; **matches** now |
| a cell with a living building never receives ore | never | never | matches |
| an ore mine's cell never receives ore | never | never | matches |
| high-bridge cells never receive ore (also on rock) | never (the deck mark) | never | matches |
| rock cells (land not buildable) never receive ore | never | never | matches |
| wall cells never receive ore | never (an overlay) | never | matches |
| spread gains fall on tick 0 and on multiples of the spread field | yes: the first game's first update is in tick 0, then every period | yes | matches (claims 6, 19) |
| every new cell, from spread steps and from the ore mine, appears at stage 3 | yes | yes | matches (claim 11) |
| the ore mine's new cells are off the spread schedule | yes: mines spread in their own turns | yes | matches (claim 15; `claim_15_f8_ore_mine`) |
| the ore mine never puts ore on its own cell or on a wall | never | never | matches (`claim_15_ore_mine_next_to_a_tree`) |
| the ore mine fills its free neighbours | yes | yes | matches |
| secondary spread out of new cells, on multiples of the spread field | yes: every spread step is in a spread update on the timer | yes | matches (by construction; not separately tested) |
| gains next to a civilian standing in the cell | yes | yes | matches (infantry in a cell does not prevent it) |
| each run is the first game of its session | the first game's timers start unset, so the first updates are in tick 0 | the same | matches (claim 19) |

Claim 10 lets ore grow under a building that is not drawn, and the local lane saw one such gain
on map 2; that is allowed, not a departure.

The record's Result tables trip our stock guard (counts in a local record), as M-0015 does; left
for the lead.

## Claims

| Claim or item | Status | Notes |
|---|---|---|
| 7 (the offer at a tree's cell; its entry dropped when taken) | tested | `claim_11_tree_cell_growth_entry_dropped` |
| 9 (a tree's cell chosen: the attempt used, nothing made) | tested | `claim_11_tree_cell_chosen_receives_nothing` (E8 part b: the draws of a whole update by a probe, 24 seeds, both positions) |
| 10 (a tree's cell germinable with the switch on and counted in *c*; not with it off) | tested | the same test; `claim_10_terrain_object_neighbour_switch_off` (S-0319's offer takes no draws there, switch off) |
| 10 (the kinds of cell M-0013 tested) | tested | `m0013_which_cells_receive_spreading_ore` |
| 10 (E8 part c: other bridge parts, the usable area, an invisible building, the tileset, slope) | partly | as before: slope, buildable land and a living building in `claim_10_germinable_cells`; the others are in `germinable` without a test |
| 11 (a cell holding a terrain object: overlay draw, growth-list offer, no overlay, no stage) | tested | `claim_11_tree_cell_chosen_receives_nothing`, `claim_11_crate_stage_at_a_tree_cell` (S-0520 claim 24's path) |
| 15 (an ore mine's step choosing a tree's cell) | tested | `claim_15_ore_mine_next_to_a_tree` (on: two draws more than off, the growth entry; nothing placed; the mine's own cell and the walls untouched) |
| `quirks.spread_step_lost_on_terrain_object` | tested | both positions in the four tests above |
| 19 (the first game's tick-0 spread update) | tested | `m0013_which_cells_receive_spreading_ore` (tick 0 is the first tick, gains on the schedule), `claim_6_f3_f4_timers` |
| State and ordering (the row for a cell with a terrain object) | tested | as claim 11 |
| Changes for other specs (S-0319 claim 10's wording) | named, left | Text for S-0319; our offer already gives a tree's cell nothing (intake 33). |
| Scenarios E5, E8, E11 | not ours | Oracle scenarios. |
| Open questions | design | the stage of a tree's cell (above). |

## Before and after

- Goldens (`glasnost-sim-gate`): unchanged. The gate maps hold no terrain object next to ore.
- Sweeps (M6c, M7a, M7b, seeds 1 to 16, ref and fix): every counter identical before and after.
- Arena (`--record all`): our CSVs for M-0001, M-0002 and M-0003 are byte-identical before and
  after; failing pairings stay 30 of 96, 12 of 24 and 2 of 35.
- Local lane (`crates/app/tests/spread_lane.rs`, ignored; the spread field patched to 30 in the
  lane's rules copy, 3000 ticks): no map 0 to 16 gains ore in a cell holding a terrain object, a
  living visible building, a bridge or another overlay. Cells gained, switch on against off:
  map 0 203 / 208, map 1 1 / 1, map 5 155 / 163, map 7 72 / 75. The loss is small and only next
  to trees. No map 0 to 16 has ore next to a bridge, so the bridge cases rest on the unit tests.
