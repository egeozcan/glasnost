# S-0616 r27, S-0001 r7 with M-0012, S-0306 r63, S-0613 r13: claim ledger, M8 step A3 intake 72

- **S-0616** r20 to r27 (commit 48c87b3, from c4c1e6e).
- **S-0001** r4 to r7 (a578337, from b1a2e37), with the new record **M-0012** (dbfcd15).
  The ini reader was last built at r4 (fcb52c0). r5 (056de19) changed nothing that is built.
- **S-0306** r62 to r63 (0cd21d1, from b764d95).
- **S-0613** r12 to r13 (6d6952a, from a4c6b28).

No request is answered by these revisions, and none of them names one. Requests filed: none
(Q-2281 to Q-2285 unused).

## Built

The simulation's behaviour does not change: `git diff main -- crates/deterministic/sim/src`
touches only comments. Each revision restates rules that are already built:

- **S-0616 r27** takes in S-0301 claim 5's place-and-kind rule (S-0301 r63, intake 63) and the
  marked spark system's bursts (S-0301 r67's change (d), intake 70). Both were built and cited to
  S-0301.
- **S-0613 r13** names the fire place's let-go among claim 17's marks. `gas::system_turn` already
  calls `emitters::fire_place_let_go` for every kind other than fire, gas and no kind included.
- **S-0001 r7** adds backing (M-0012), test-plan wording, and the reasons why the byte-order mark
  and CR-only endings have no switch. It also adds a question for the project lead; this intake
  does not answer it.
- **S-0306 r63** changes P16 part g's procedure only: the deadline, and a note that retaliation
  keeps the processing timer.

This intake adds the missing tests and moves the citations to S-0616's own claims:

- `emitters.rs`: the creation reading, the fire place's let-go and the damage sparks' spark-place
  test.
- `particles.rs`: the smoke removal pass in a marked system, and the spark table reading only
  *K*.
- `gas.rs`: the let-go for every kind, S-0613 claim 17 included.

## Claims

### S-0616 r27

| claim | status |
|---|---|
| Terms, emitter places: a weapon's emitter goes into the place its flag names, whatever its system's kind | Existing (intake 63), cited in `emitters::effect_weapon_emitter`; tested by `claims_s0301_r63` (u1 to u6) |
| 1, creation by place: the fire or spark place passes the shot's target object, with its position at the shot as the target point; the railgun place passes none, and its target point is F14's end point | Existing; `claims_s0301_r63::creation_follows_the_place` checks the stored fields |
| 1, a railgun-kind system held in the fire or spark place makes its line to that place's target point, which is never recomputed from the target | Tested: `claims_s0616_r27::a_railgun_kind_system_in_the_fire_or_spark_place_lines_to_the_shots_target`. The target moves before the first turn, and the line goes to its position at the shot |
| 1, the target point of the fire or spark place is the world origin when the shot has no target object; the railgun place's point is F14's end point, whatever the kind | Tested: `claims_s0616_r27::a_cell_shot_lines_to_the_origin_from_the_fire_place_and_to_f14_from_the_railgun_place`. Rated medium and `unmeasured` for a kind that does not match its place |
| 1, the fire place's let-go (a retarget out of reach, a move order, death) marks whatever the place holds; the place is freed at the let-go | Existing. Tested for the spark kind (`claims_s0301_r63`), smoke and fire (`claims_s0301_r67`), and now gas and no kind for all three events: `claims_s0616_r27::a_no_kind_or_gas_kind_system_in_the_fire_place_is_marked_by_a_let_go` |
| 1, the retarget test is made at the change of the target | Existing (intake 70, `emitters::retarget_let_go`) |
| 1, the spark and railgun places have no let-go; a fire-kind system there is not marked by its own turn while its owner is alive and on the map | Existing; tested by `claims_s0301_r63::spark_and_railgun_places_have_no_let_go` and `f4_unmatched_u3_u4_…`. An owner that leaves the map without removal remains claim 4's open question (seam, unchanged) |
| 1, damage sparks need nothing in the spark place, whatever the kind of a system held there | Tested: `claims_s0616_r27::damage_sparks_wait_for_an_empty_spark_place_whatever_it_holds` (a cyborg's spark-flag weapon puts a fire-kind system there; no draw and no system, then the control with the place empty) |
| 1, damage smoke: a marked system makes no emission, and its dead particles still split | Covered by the claim 8 row below. The healing marks are unchanged |
| 2, a no-kind system in the fire place leaves play when a let-go marks it and it holds no particle | Tested in the no-kind run of the let-go test above |
| 8, successors in a marked system (medium, P1 part b) | Tested: `claims_s0616_r27::p1_part_b_a_trigger_mark_stops_emission_but_not_successors_or_bursts`. After S-0011 code 89's mark of the cell, only successors (ASH) appear in the smoke system, never an emission (SOOT); it holds a particle while in play and then leaves |
| 9, a smoke system held in the fire place ends by a let-go | Existing; tested by `claims_s0301_r67::f4_third_run_…` (u10) |
| 10, a marked turn reached by a let-go | Existing; tested by `claims_s0301_r67` (the fire firer) |
| 15, the table reads only *K*: a marked spark system with *K* > 0 bursts | Existing; tested by the P1 part b test (a trigger mark) and by `claims_s0301_r67::a_marked_spark_system_bursts_while_its_counter_lasts` (now cited to S-0616 r27) |
| 17, a system marked by a let-go before its first turn | Tested: `claims_s0616_r27::a_railgun_kind_system_let_go_before_its_first_turn_makes_no_line` (no line, no draw; it leaves play in that turn and the place is free). Rated `unmeasured` |
| 20, the spark-turns fix in any place: after one turn a let-go of the fire place finds the place free | Tested: `claims_s0616_r27::claim_20_fix_frees_the_fire_place_after_one_turn` |
| 20, the original's let-go end in the fire place | Not built: the original's behaviour, replaced by our fix (lead decision, no switch) |
| 22, the table's fire-place let-go entries (smoke, fire, spark, railgun, no kind) and the gas-kind sentence | Tested by the tests above, one kind each |
| F14, computed for the system of the railgun place, whatever its kind | Existing (`emitters::effect_weapon_emitter`); tested by the cell-shot test (railgun place) and `claims_s0301_r63::creation_follows_the_place` |
| Terms *Marked* and the state table's marked row | Tested by the P1 part b test |
| P1 part a and part b wording | The new set-up is followed: a next type for the smoke system's held type, and a spark counter still above 0 at the mark |
| P1 part b as a whole (a trigger mark on a smoke, a fire, a spark and a gas system in one cell; a second run on the shared gas system's cell) | Partly. The smoke and spark systems are tested by the P1 part b test above. The trigger mark of every system in a cell, whatever its kind, is tested by `claims_s0616_emitters::claim_1_trigger_actions` (existing). A marked fire system that makes no emission is tested by `claims_s0301_r67` (the fire firer). A marked gas system's turn does not read the mark (S-0613). No new test covers the shared gas system's run |
| Changes for other specs, S-0613 claim 17 | Taken in by S-0613 r13 (below) |

### S-0613 r13

| claim | status |
|---|---|
| 17, a let-go of the fire place marks the system that place holds, a gas-kind system included (medium; `unmeasured` for gas) | Tested: the gas-kind (CLOUD) runs of `claims_s0616_r27::a_no_kind_or_gas_kind_system_in_the_fire_place_is_marked_by_a_let_go`. Existing code (`gas::system_turn` → `emitters::fire_place_let_go`), now cited to S-0613 r13 |

### S-0001 r7 and M-0012

| claim | status |
|---|---|
| 5, measured (M-0012): in a fresh load, a rules file's last line with no line feed after it is not read | Tested end to end through `convert`: `glasnost_convert` `sim::combat_tests::a_rules_files_unterminated_last_line_reads_as_absent`. A synthetic base rules file whose last line is a weapon's burst key, with no line feed, gives the same `sim` data as the control without the line. With one line feed after it, the data differs and the key is read. The importer warns only for the unterminated file. The switch off reads it. Only M-0012's two comparisons are used; the names and the value are ours |
| 5, the base rules file is a fresh load (S-0002 claim 6) | Checked: `convert` reads the base file with `Document::parse` (`FinalLine::PerQuirk`); only `Document::layer` reads an unterminated line |
| Edge cases: no switch for the byte-order mark or CR-only endings, and why | Unchanged (no switch; the importer warns). The question to the project lead is not ours to answer |
| Test-plan wording (no engine add-on or mods, except named test files and test maps) | Wording only |
| `doc.rs::final_line_and_order` | Now also cites M-0012 |

S-0001 r7 does not change how a rules file is read. No synthetic fixture and no committed ruleset
reads differently.

### S-0306 r63

| claim | status |
|---|---|
| P16 part g: retaliation does not restart the copy's processing timer (S-0215 claim 10), so the deadline may be the guard delay's upper bound plus 300 ticks | Tested: `claims_s0306_modes::claim18_retaliation_keeps_the_guard_processing_timer` (a computer-house mode-0 copy in guard, its guard processing run: `acquire::retaliate` overrides to *attack* with *guard* suspended, and the timer, last processing and expired flag, is unchanged). The land-row result itself is unchanged (`claim18_land_row_stops_retaliation_and_mode_zero_fires_back`) |

## C7

No revision adds a quirk. S-0001 r7 explains why the byte-order mark and CR-only endings have no
switch, and asks the lead whether they should; this intake adds no switch. Claim 20's fix stays a
fix with no switch (lead decision). [c7-play-fixes.md](c7-play-fixes.md) is unchanged.

## Formats

No new state. The save format (92), the AI game format (54) and the wire version (12) are
unchanged, and no golden moves.

## Lanes

Skipped under the brief's exemption. The simulation is unchanged: the only edits in
`crates/deterministic/sim/src` are comments, and the rest are tests, an importer test and docs.
The arena, the sweeps, the AI include-ignored run and the AI quick lane run the same code as
main.
