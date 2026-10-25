# M8 step A3, intake 19: S-0105 r35 (voxel rendering; Q-1760, the multi-turret table)

S-0105 revision 35 answers Q-1760: what sets the selected turret index of a multi-turret vehicle.
We had built revision 22; this record covers `git diff 193e521 c2ee154 --
spec/rendering/S-0105-voxel-rendering.md`. The revision answers play notes 3 (IFVs have no
turret) and 14 (Prism tanks have no turret) of [a4-play-notes.md](a4-play-notes.md). It also
states the per-stem defaults of the turret table pairs, as the lead's ruling of 2026-10-24 in
[../m8-plan.md](../m8-plan.md) asks.

**Code.**
- `crates/convert/src/turrets.rs` (new) reads the turret table.
  - Its constants: the 17 stems in the Import mapping order with their absent-key index defaults,
    the table size of 18, and the fixed type name.
  - `read_table` applies the pairs in each layer that has the section, in order. A later pair
    wins, and the table is never cleared.
  - `charge_flag` gives the flag the last pair with no weapon number sets in each layer, after
    that layer's own `IsChargeTurret`.
- `crates/convert/src/lib.rs`: `Import::turret_tables` holds each vehicle type's table by
  registry index, leaving out all-*none* tables.
- `crates/convert/src/sim_combat.rs`: `turret_pairs`, called for every vehicle type in
  `unit_combat`, does two things.
  - For the fixed type name it writes the charge-turret flag over the held `is_charge_turret`.
  - It gives the two importer warnings.
- `crates/convert/src/sim.rs`: `SimQuirks` gets `turret_table_fixed_type_only` and
  `turret_pair_no_weapon_sets_charge_flag`, both on by default, and both listed in `named()`.
- `crates/convert/src/coverage.rs`: the 34 pair keys are classed under S-0105.
- `crates/app/src/mapview/build.rs`:
  - `selected_turret_index` (live) and `creation_turret_index` (the static map view) derive the
    index from simulation state through `turret_index_from`. `charge_turret_number` is S-0301
    claim 16's *c*.
  - The turret block draws turret and barrel *k* only for *k* in 0 … *N* − 1, in frame *f_b*,
    and keys the image by (*k*, *f_b*).
  - `ConsideredAircraft`'s absent default now depends on the kind.
- `crates/app/src/mapview/live.rs` passes each vehicle's table to `selected_turret_index`.
- `crates/app/src/mapview/quirks.rs`: both switches are off the `NOT_IMPLEMENTED` list, and their
  rows are gone from the table in [../development.md](../development.md).

**No new simulation state.** The index is derived, never stored:
- The simulation already holds the weapon number (`combat.weapon_number`, S-0308 claims 5 and 6).
  It is 0 from creation and is set at every gunner hand-over.
- The simulation already holds the rearm countdown (`combat.rearm_until`, `combat.rearm_delay`)
  and the charge-turret flag (`type_combat.charge_turret`).
- For a gunner type that is not a charge-turret type, claim 43's "index = *T*[*w*] each time *w*
  is set" is therefore *T*[current weapon number] at every moment, because the table is fixed per
  game. For a charge-turret type, the index is *c* from the countdown at the shown tick.

The simulation code is unchanged: save format 78 and AI game format 40 stay. The one effect on
play comes through the importer. The fixed type name's `is_charge_turret` can change under
`quirks.turret_pair_no_weapon_sets_charge_flag`, which S-0308 claim 5 and S-0301 claim 16 then
follow.

**Where the switches live, and why.** Both are in `glasnost_convert::sim::SimQuirks`, the
importer's switches, not in `glasnost-rules` `Quirks`.
- `turret_pair_no_weapon_sets_charge_flag` is a rule for reading the rules files. The spec says
  "the importer must apply this rule". What it decides is the imported `is_charge_turret` field.
  The simulation then reads only that field, so a simulation-side switch would have nothing left
  to consult. The precedent is `clear_weapons_any_turret_count` (S-0308), also an import rule
  with gameplay effect. The switch reaches the import record through `named()`.
- `turret_table_fixed_type_only` decides which types' pairs the importer reads into
  `Import::turret_tables`. The table is presentation only (the index selects no weapon, claim
  43), so it stays with the import that the renderer reads, and no rules schema change is needed.

**Tests.**
- `crates/convert/src/turrets_tests.rs` (new), on synthetic sections; `FV` appears only as the
  spec-cited constant:
  - `the_fixed_type_name_reads_its_pairs`
  - `every_type_reads_its_pairs_with_the_switch_off`
  - `later_pairs_win_and_entries_stay_across_layers` (MS-33 part h, both runs)
  - `out_of_range_weapon_numbers_are_ignored_with_a_warning`
  - `a_pair_with_no_weapon_number_sets_the_charge_flag` (part g and its mirror, multiples of 256,
    a bare section, the last-in-order rule, layers, and the switch off)
  - `turrets::tests::pair_keys_follow_the_stems`
- `crates/app/src/mapview/turret_index_tests.rs` (new):
  - `a_gunner_type_shows_the_entry_for_its_weapon_number`
  - `the_charge_turret_number_steps_down_with_the_countdown` (MS-33 part f)
  - `a_charge_turret_type_shows_its_countdown` (real shots in a world: indices 2, 1, 0)
  - `a_map_vehicle_before_any_tick_shows_its_creation_index`
- `crates/app/src/mapview/live_tests.rs`:
  - `multi_turret_draws_the_models_of_the_selected_index` now takes the index and the animation
    counter. It checks the keys by (index, frame), frame *f_b* with the counter at 3, and that the
    indices 2 (= *N*), 17, −1 and −5 draw the body alone.
  - `considered_aircraft_defaults_by_kind`, new.
- `crates/deterministic/sim/tests/claims_s0606.rs`: `claim_1_multi_turret_vehicle_by_its_weapon_number`
  (MS-33 part i), new.

**Goldens, arena and sweeps.** See the *Checks* section at the end.

| Claim or item | Status | Notes |
|---|---|---|
| Terms, *Aircraft-like type*, and Import mapping `ConsideredAircraft`: absent, yes for an aircraft type | built, tested (new) | Closes the intake 16 seam. The renderer's shadow rule reads `rule_bool(.., "ConsideredAircraft", kind == Aircraft)`. `considered_aircraft_defaults_by_kind`: an aircraft and a vehicle type, each without the key, give the half-size and full-size shadows, and an aircraft type that clears the key gives full size. The simulation already read S-0501's default (intake 16). |
| Terms, *Multi-turret type* "whatever its body"; a sprite body has no numbered models | followed | `multi_turret` decides from the turret flag, the count and the gattling flag. The sprite path (`sprite_vehicle`) never reads numbered models. |
| Terms, *Selected turret index*: a turret index or *none*, never a weapon | built | `Mobile::turret_index: Option<i32>`, derived. The simulation's weapon tests read `weapon::current_weapon` (the weapon number's slot), never the index. |
| Terms, *Turret table*: 18 entries, all *none* unless imported | built, tested (new) | `turrets::TurretTable`, `TABLE_LEN`, `EMPTY`. |
| Terms, charge-turret type, gunner type, weapon number, mode index (S-0308) | followed | The fields the simulation already holds (above). |
| Claim 12, the model of index *k* for a voxel multi-turret vehicle | built, tested | `live_tests::multi_turret_draws_the_models_of_the_selected_index`. |
| Claim 12, *No model for the index*: *none*, or *k* outside 0 … *N* − 1, draws neither turret nor barrel; a missing barrel *k* alone leaves turret *k* | built, tested (new) | The turret block filters the index to 0 … `TurretCount` − 1 (clamped to 18). Same test: 2 = *N*, 17, −1 and −5 draw the body alone. With no numbered barrel supplied, turrets 0 and 1 are drawn without one. |
| Claim 12, *Original glitches, fixed without a switch* (*N* = 18 with *none*; *k* ≥ 18 or below −1) | followed | Nothing is drawn in either case. The importer warns about the entries (Import mapping row below). |
| Claim 12, *Sprite body*: the index selects nothing; with the turret flag, body and turret frames as S-0102 claim 22 says | followed | `sprite_vehicle` is unchanged and never reads the index. |
| Claim 12, *Sprite body*: the single voxel barrel drawn at the turret position, turned and pitched, with claim 11's overlap rule | seam, Q-1885 | Not built. S-0102 claim 13 has no depth-bias or profile row for a voxel part of a sprite-drawn vehicle, so where to put the barrel image in the scene is unspecified. Interim: no voxel part is drawn on a sprite body, as before. The voxel turret part, which the claim says is never drawn, is not drawn. |
| Claim 16, a numbered turret is drawn in frame *f_b*, never from the animation counter; its key records the same frame | built, tested (new) | For a multi-turret vehicle `turret_frame = body_frame`; the other paths keep `vehicle_turret_frame`. The test gives the turrets 4-frame synthetic HVAs, sets the counter to 3, and expects frame 0 with key (k, 0). Claim 13 reduces the frame by the model's own count, as before. |
| Claim 16, the open question on the multi-turret frame | retired | The interim (claim 16's vehicle turret frame) is gone. |
| Claim 31 backing, the absent default from S-0501 | followed | As the `ConsideredAircraft` row. |
| Claim 43, *Taken from the turret table*: *T*[*w*] each time the weapon number is set, in the same tick, a request outside 0 … 17 giving *T*[0] | built, tested (new) | `turret_index_from`: a gunner type that is not a charge-turret type gives `table[weapon_number]`. `weapon::weapon_number` clamps to 0 as S-0308 claim 5 does. The index changes in the tick of the hand-over, because the simulation sets the weapon number then and the view reads it at the shown tick. `a_gunner_type_shows_the_entry_for_its_weapon_number`: MS-33 part b's modes 1, 2, 3, 17 and 20 give 0, 1, 5 (kept, drawn as nothing), *none* and *T*[0] = 2. |
| Claim 43, only the fixed type name has a filled table | built, tested (new) | `quirks.turret_table_fixed_type_only` (Edge cases row below). |
| Claim 43, *Not a weapon choice*: firing, target and threat choice, the S-0606 bunker test and S-0209's armed tests ask the weapon S-0308 chooses | followed, tested (new) | The simulation's tests already read `current_weapon` or the slot weapons, and none reads the index (it is not simulation state). `claims_s0606::claim_1_multi_turret_vehicle_by_its_weapon_number` (MS-33 part i shape): a multi-turret type with a weapon in slot 0 enters, and one whose slot 0 is empty does not until its weapon number is 1. |
| Claim 43, *When the weapon number is set*: 0 at creation by every route, and at every gunner hand-over | followed | The simulation's weapon number starts at 0 for every new object (map, factory, any other route) and `set_weapon_number` runs at the hand-overs (S-0308 claim 6). `a_map_vehicle_before_any_tick_shows_its_creation_index`. |
| Claim 43, neither a gunner nor a charge-turret type: *none* for life | built, tested (new) | `turret_index_from` gives `None`. `a_gunner_type_shows_the_entry_for_its_weapon_number` covers a non-gunner type with a filled table. |
| Claim 43, *Charge-turret types*: *c* of S-0301 claim 16 in each turn, *N* − 1 after a shot, 0 once the countdown is over and while *D* ≤ 0; ammunition has no direct effect | built, tested (new) | `charge_turret_number(n, until, delay, tick)`: 0 when *D* ≤ 0 or tick ≥ until, else trunc(*N*·*r*/*D*) clamped to 0 … *N* − 1, in i64. The derived *c* reads only the countdown, so ammunition does not enter it. `the_charge_turret_number_steps_down_with_the_countdown` (MS-33 part f: thirds of *D*), `a_charge_turret_type_shows_its_countdown`. |
| Claim 43, charge-turret: "in each of the vehicle's turns" | seam | The derived index reads the countdown at the shown tick, so it also follows the countdown in a tick in which the vehicle has no turn, such as while it is carried (where it is not drawn) or the frame it is unloaded. The view cannot see the turn. The difference is at most one drawn frame per missed turn, and S-0105 rates the original's own first-frame difference a drawing artefact. |
| Claim 43, *From creation (fixed, no switch)*: our engine shows turret 0 from creation | built, tested (new) | *c* is 0 while *D* ≤ 0. `creation_turret_index` gives the static map view the same index. |
| Claim 43, *Nothing else changes it*: veterancy, transport, attacking | followed | The index depends only on the weapon number and the countdown, and neither veterancy nor carriage sets them. |
| Claim 43, *Any body* | followed | The index is derived the same way for a sprite body. It reaches nothing drawn, because the barrel is not built (Q-1885). |
| State table S3 | built | Row by row: creation gives *none* for all but our engine's charge-turret vehicles. The other rows are covered above. The static map view's creation row for a gunner type is *T*[0] (weapon number 0). |
| Constants, *Turret table size* 18 | built | `turrets::TABLE_LEN`, cited. |
| Import mapping, `Gunner`, `IsChargeTurret`, `IFVMode` rows (S-0308 owns them) | followed | Already imported, unchanged. `IsChargeTurret` can be overwritten for the fixed type name (the charge-flag row). |
| Import mapping, the 17 pairs `<stem>TurretIndex` / `<stem>TurretWeapon`, absent index the stem's default (Repair 1, MachineGun 2, Flak 3, all others 0), absent weapon −1 | built, tested (new) | `turrets::STEMS`, `PAIR_KEYS`. `the_fixed_type_name_reads_its_pairs` (MS-33's subject table). The `Normal` stem's 0 is tested by part h's second run, the other defaults by the bare-section case. |
| *Turret table pairs*: order, a later pair wins, never cleared across layers, the per-layer stem default | built, tested (new) | `read_table`. `later_pairs_win_and_entries_stay_across_layers` (MS-33 part h, both runs). |
| *Turret table pairs*: a weapon outside 0 … 17 other than −1 is ignored, and the importer warns | built, tested (new) | `turret_pairs`. `out_of_range_weapon_numbers_are_ignored_with_a_warning`: weapon numbers 18 and −2 are ignored, warned and change nothing. |
| *Turret table pairs*: an index outside −1 … *N* − 1 is kept, draws nothing, and the importer warns | built, tested (new) | `turret_pairs` warns with *N* = `TurretCount` (at most 18). The kept entry reaches the turret block, which draws nothing (claim 12 row). `out_of_range_weapon_numbers_are_ignored_with_a_warning` (index −3 of 3 warned) and `the_fixed_type_name_reads_its_pairs` (MS-33's `Flak` pair, turret 5 of 3, kept with a warning). |
| Edge cases, `quirks.turret_table_fixed_type_only` (on) | built, tested (new) | On: only the fixed type name reads its pairs (`turrets::table_for`), and every other type is left out of `Import::turret_tables`. Off: every vehicle type reads its own pairs by the same rules, while the charge-flag rule stays with the fixed type name. `the_fixed_type_name_reads_its_pairs` and `every_type_reads_its_pairs_with_the_switch_off`. It lives in `SimQuirks` (above). |
| Edge cases, `quirks.turret_pair_no_weapon_sets_charge_flag` (on) | built, tested (new) | On: in each layer that has the section, the layer's own `IsChargeTurret` applies first, then the last no-weapon pair (weapon key absent or −1, both keys absent included) sets the flag on exactly when `index.rem_euclid(256) != 0`. A section with no pair keys therefore ends on the `Guardian` stem's 0 and turns the flag off. A layer whose pairs all carry weapon numbers keeps the held value. Off: `is_charge_turret` is the held key. `a_pair_with_no_weapon_number_sets_the_charge_flag` (part g, the mirror, 512 and −256 off, 257 and −1 on, a bare section, the last pair deciding, a later layer, the switch off). It lives in `SimQuirks` (above). |
| Scenario MS-11, the multi-turret subject and the hand-over | tested on synthetic data | Parts covered by the renderer tests above. The recording is the lead's. |
| Scenario MS-33 parts a to l | tested on synthetic data, in part | a: `a_map_vehicle_before_any_tick_shows_its_creation_index` and the lane. b, c: `a_gunner_type_shows_the_entry_for_its_weapon_number`. d: `every_type_reads_its_pairs_with_the_switch_off`. e: the frame test. f: the countdown tests. g, h: `turrets_tests`. i: `claims_s0606`. j: our engine's fixed departure (turret 0 throughout). k: not built (Q-1885). l: by construction (*c* ignores ammunition). The recordings are the lead's. |
| Open questions, *Turret frame of multi-turret types* | retired | Claim 16 now settles it. |
| Open questions, *Image invalidation* (decided) | followed | Claim 26 is unchanged. |
| Changes for other specs, S-0308 claims 5 and 6, S-0301 claim 16 | noted | S-0301 claim 16 says a charge-turret vehicle fires its slot-0 weapon. S-0105 says S-0308 claim 9's deploy-fire override applies to it too. The simulation's charge-turret type never changes its weapon number, so `current_weapon` gives its slot-0 weapon, as S-0301 claim 16 says. No change here: the specs differ only for a user's type with the deploy-fire flag, and amending S-0301 is the lead's work. Whether S-0308 claim 9's override is built is outside this intake. |
| Changes for other specs, S-0606 claim 1 and S-0209 *Armed* wording | noted, tested (new) | S-0606 claim 1 still reads "the weapon of its current turret index". Our bunker test reads the weapon number's slot, as S-0105 says the original does (the `claims_s0606` test). |
| Changes for other specs, S-0007 claims 16, 17 and 19 | followed | Loading is unchanged. The voxel turret part of a sprite body is loaded and never drawn. |
| Changes for other specs, S-0102 colour-selector and S-0610 claim 7 tables ("voxel turret" → "voxel barrel" on a sprite body) | noted | Wording only. `protection.rs`'s comment now says the barrel. |
| Changes for other specs, hover tip name | noted | Pointer only. No hover-tip code reads the index. |
| Local lane label (`crates/tools/src/local_lane.rs`) | updated | The multi-turret class now reads "numbered parts by the selected turret index (S-0105 claim 43)". |
| Play notes 3 and 14 | answered | Note 3: the IFV is a gunner type with a filled table and shows *T*[weapon number]. Note 14: the other multi-turret type is a charge-turret type and shows its countdown turret (lane, below). |

## Requests

Q-1885 was filed (block Q-1885 to Q-1889). It asks four things about the voxel barrel of a
sprite-body vehicle: its depth row in S-0102 claim 13, its depth at a height above 0, its place in
the drawing order beside the sprite turret, and its HVA frame. Interim: no voxel part is drawn on
a sprite body. Q-1886 to Q-1889 are unused. Q-1760 is answered by this revision, and its interim
(no index for any object) is retired.

## Checks

*Before* is main a32dbdc, built before any edit of this branch; *after* is this branch on a32dbdc,
before its rebase onto intake 18. Of `crates/deterministic`, the branch touches only
`sim/tests/claims_s0606.rs`. Outputs
are in `local-lane/m8/a3-19/` (git-ignored).
- Goldens: none regenerated. The simulation code is unchanged, and the gate's synthetic rules have
  no type with the fixed type name, so no imported field changes.
- Oracle arena (local lane): the CSVs and oracle reports are byte-identical before and after.
  - M-0001: 31 of 96 pairings failing; 151 of 172 unanimous stock outcomes reproduced.
  - M-0002: 13 of 24 failing; 43 of 47 reproduced.
  - M-0003: 2 of 35 failing; 61 of 64 reproduced.
- M6c, M7a and M7b sweeps 1 to 16: every counter has the same minimum, median and maximum before
  and after. The smallest minima are M6c 2 (seed 3 charger orders, seed 12 jumpjet orders), M7a 1
  (seed 7 psychic sensor orders, seed 9 takeover orders), and an M7b drain minimum of 3.
- Lane scene `ifv-turrets` (`crates/app/tests/m7b_lane.rs`; counts only, and no mode-to-turret
  mapping is recorded):
  - multi-turret vehicle types: 2, of which 1 is a gunner type that is not charge-turret, 1 is a
    charge-turret type, and 0 are neither;
  - types with a filled turret table: 1, which is the gunner type;
  - the gunner type with no passenger shows an index (*T*[0]), not *none*;
  - 17 passenger modes were tried, giving 4 distinct indices and 0 passengers showing *none*;
  - the charge-turret type shows index 0 at creation. Over 300 ticks it fires 3 shots, and its
    index runs 3, 2, 1, 0 after each shot (about 25 ticks per step), as S-0301 claim 16 gives
    with *N* = 4.
