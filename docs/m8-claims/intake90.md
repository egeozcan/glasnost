# S-0214 r80 and S-0216 r69: claim ledger, M8 step A3 intake 90

Two small deltas, both read in full:
- S-0214 r78 to r80 (`git diff fea4276~1 fea4276 -- spec`) answers **Q-2426**, which intake 85
  filed ([intake85.md](intake85.md));
- S-0216 r67 to r69 (`git diff 2116cf3~1 2116cf3 -- spec`) answers **Q-2446** and **Q-2447**,
  which intake 87 filed ([intake87.md](intake87.md)).

Requests filed: none. Q-2476 to Q-2485 are unused.

Status words: **tested** (built and checked by the named test), **partly** (the part named is
untested or not built), **seam** (another intake's spec builds it; the reason is given), **no
change** (the revision restates what is built).

## S-0214 r80

Q-2426's interim was the literal reading: with `quirks.moment_reason_attack_cursor` off, part (e)
passes over every answer that is not *illegal*. r80 gives that same reading, so no code path
changes. The citations in `check::illegal_at_any_rank`, `Ask::IllegalOnly`, `answer`'s `pass` and
`targeting::attack_rule` move from the interim to r80. The quirk's doc comment in `quirks.rs` (and so
`data/schema/ruleset.schema.json`) now names what is passed over. Tests are in
`crates/deterministic/sim/tests/claims_s0214_r80.rs` (5 tests), each in both positions except the
last, which is off only.

| claim part | status |
|---|---|
| Claim 6 (e): the switch can make part (e) ask every reason that gives *illegal*, "whatever the other answers ahead of it" (was "whatever the reasons of the moment") | No change: the same code as the Edge case below |
| Edge cases, off: part (e) asks only whether a reason that gives *illegal* holds, at any rank of S-0301 claim 9, and passes over every answer that is not *illegal*, so an *illegal* reason ranked below such an answer still fails part (e) | Tested: `answer` with `Ask::IllegalOnly` skips every non-*illegal* return (`!pass` gates, or keeps only an *illegal* answer of the building's and the lifter's own answer sets). The tests below and intake 85's off branches (`claims_s0214_r78.rs`, `claims_s0214_combat::claim_6e_switch_off_reloading_neither_gives_nor_takes_the_attack`) |
| Off, passed over: *busy* (rank 2) | Tested: `off_a_launching_vehicle_over_an_illegal_target_fails_part_e` (a launching count of 1, over a zero-versus target; over a target nothing rules illegal the attack stays in both positions) |
| Off, passed over: every *rearming* of ranks 4 and 5, the own-gripped-unit and magnetic-beam answers (S-0607 claim 11) among them | Tested for the grip: `off_a_lifter_over_its_gripped_unit_in_the_air_fails_part_e` (the projectile refusal against the gripped unit in the air). The magnetic-beam *rearming* is the same gate in `lift::firing_answer` (only an *illegal* answer is kept) and is not tested apart. The other early *rearming* answers and rank 5: intake 85's tests |
| Off, passed over: every *impossible*: the hidden target's (S-0516 claim 24), the held spawner's (S-0506 claim 10), the other weapon's live effect (S-0301 claim 5), a building's own (S-0405 claim 9, table T1) | Tested for the hidden target: `off_a_hidden_target_that_a_later_test_rules_illegal_fails_part_e` (a cloaked enemy, the zero-versus test). The other three are `!pass` gates of the same kind and are not tested one by one |
| Off, passed over: out of ammunition, *cloaked* | No change: intake 85's `out_of_ammunition_and_cloaked_hide_the_zero_versus_test` |
| Off, passed over: *must deploy* (rank 11) and the other lower ranks | Tested for *must deploy*: `off_a_deploy_to_fire_healer_on_a_failing_cell_fails_part_e` (a cell that is not flat, an own vehicle at full health, rank 12's healer *illegal*; the healer has the attack-friendlies flag so that the click reaches the attack rule, since over an enemy a healing vehicle gets *nothing* from table V row b1). The lower ranks after rank 12 give no *illegal* answer, so passing over them changes nothing |
| Off, two stops remain: with no weapon in the slot nothing further is asked and part (e) holds; the range test is not asked | Tested: `off_the_two_stops_remain` (an out-of-range target with nothing illegal, and an unarmed vehicle). `attack_rule` short-cuts the empty slot before asking the check |
| Off: the four consequences the Edge case lists (the lifter, the hidden target, the launching vehicle, the deploy-to-fire healer) | Tested: the four tests above, each with the click and the action over the target in both positions |
| Off: a departure by decision, `unmeasured`, confidence high | No change: backing text. The C7 verdict is judged again below |

**Quirk.** No new switch, and the default of `moment_reason_attack_cursor` stays on. Its C7 verdict
is judged again in [c7-play-fixes.md](c7-play-fixes.md) under *Intake 90*: rejected, unchanged.
`every_spec_quirk_has_a_switch` passes.

## S-0216 r69

| claim part | status |
|---|---|
| Claim 8, *Seeds and the human player house's country*: the arena's game for seed number *s* seeds S-0202's generator with 1000 + *s* (Q-2447) | Tested: `oracle::game_seed` (1000 + *s*, `None` on overflow) is used only where `oracle::setup` seeds the stream; the seed number stays in the CSV `seed` column, `SEEDS`, the reports and every command-line seed argument. `arena_setup::pt6_seed_number_s_seeds_the_stream_with_1000_plus_s` (seeds 0 to 5: the first creation draw's report value is the stream seeded with 1000 + *s* at place `first_raw`, not the stream seeded with *s*). The two synthetic tests that rebuild the stream (`pt6_the_row_types_group_is_created_first_on_whichever_side_it_plays`, `pt6_game_start_two_house_rounds_three_base_units_then_the_final_draw`) and the local-lane diagnostics now seed it the same way |
| The M-0019 acceptance item, PT6 and every other per-seed comparison with the records use that seed | Tested: they all go through `oracle::setup`. PT6's runs "at seed number 0" are the arena-setup tests at seed 0 |
| The human player house takes side A's country (design; Q-2446) | Tested: `oracle::HUMAN_COUNTRY_SIDE` was Q-2446's interim and is now cited to r69; `pt6_game_start_houses_in_claim_19s_order` asserts that the human house's country is side A's |
| The country picks the house's base unit; the house-creation draws do not depend on it | No change: S-0009 claim 45 and S-0202 claim 14, built before |
| *Acceptance (M-0019)*: with seed numbers 0 to 5, each game seeded so, the first object's creation draw takes the raw value at *N*₀ | Tested (local lane): `arena::m0019_the_first_creation_draw_takes_the_raw_value_at_n0` passes. See below |
| PT6: "at seed number 0 (claim 8)" | No change: wording |
| Open question Q-2446 question 1 (how the records' setup turns *s* into the seed, and their human house's country) | No change: open in the spec, not ours to build |

### M-0019's *N*₀ (local lane)

`arena::m0019_the_first_creation_draw_takes_the_raw_value_at_n0`, release, with the install:

| Seed number | 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| M-0019 *N*₀ | 706 | 709 | 711 | 718 | 706 | 701 |
| Ours before (stream seeded with *s*, intake 87) | 711 | 698 | 702 | 709 | 711 | 708 |
| Ours after (stream seeded with 1000 + *s*) | 706 | 709 | 711 | 718 | 706 | 701 |
| Gap after | 0 | 0 | 0 | 0 | 0 | 0 |
| Rounds 1 and 2 (raw values) after | 9, 7 | 5, 6 | 9, 8 | 7, 9 | 7, 7 | 6, 8 |

The place is the same in every all-infantry pairing of a seed. No gap remains, so no request.

### Intake 87's three choices

r69 settles none of them. It adds only the seed rule, the human house's country, the wording of
the acceptance item and PT6, and an open question:
- sandbox starts take no house draws (S-0202 claim 14, unchanged);
- the AI fixture's fixed starts, Alpha at 0 in the west and Beta at 1 in the east (our test
  fixture, not in S-0216);
- the arena idle-house hooks (`acquire::candidate`, `acquire::retaliate`, `acquire::anger`,
  `damage::apply`). r69 keeps r67's *Removal before the groups* text ("no object targets or
  damages its objects, and the computer houses ignore it", design) word for word.

## Formats

The simulation code is unchanged apart from comments (`git diff 7e0ea74 -- crates/deterministic`
changes only comments, the quirk's doc comment and a new test file). No hashed state is added and no
saved game replays to anything else, so the save format (95), the AI game format (58) and wire (12)
stay as they are. No golden moved: the sim gate never goes through `oracle::setup`. The schema
changes only the switch's description.

## Lanes

Main 7e0ea74 against the branch, `glasnost-headless arena --record all --threads 4`, both binaries
built from the worktree (outputs under `local-lane/m8/i90/`, git-ignored).

Every arena game now starts from another stream, so every pairing can move. 86 of 1 860 games
change their winner (72 of 1 152 in M-0001, 6 of 288 in M-0002, 8 of 420 in M-0003) and 266 their
end frame. Failing pairings: 10 of 155 before, 13 after (M-0001 8 → 10 of 96, M-0002 1 → 2 of 24,
M-0003 1 → 1 of 35). Stock outcomes reproduced: 269 of 282 before, 265 after. DISK–FV is still the
only pairing that fails the win test.

| Pairing | Before (row wins/col wins/none, side A, side B) | After | Changed |
|---|---|---|---|
| LTNK–E1 | fails: 0/12/0 and 1/11/0 against stock col on both | passes | yes, now passes |
| TELE–APOC | fails: 1/11/0 | fails: 0/11/1 | counts changed |
| TELE–V3 | fails: 0/4/8 | fails: 0/3/9 | counts changed |
| TELE–ROBO | fails: 0/4/8 | fails: 0/1/11 | counts changed |
| TELE–BFRT | fails: 2/3/7 | fails: 3/6/3 | counts changed |
| CAOS–E1 | fails: 1/6/5 | fails: 0/3/9 | counts changed |
| DISK–FV | fails the win test: 12/0/0 | fails the win test: 11/1/0 | counts changed |
| INIT–E2 | fails: 10/2/0 | fails: 7/5/0 | counts changed |
| ROBO–HTK | fails: 0/10/2 | fails: 0/10/2 | no |
| MIND–E1 | fails: 11/0/1 (side A's unanimous row wins missed) | fails: 10/2/0 (now side B's) | counts changed |
| CAOS–APOC | passes | fails: 3/0/9 against stock none on both sides | newly fails |
| CAOS–TTNK | passes | fails: 1/0/11 against stock none | newly fails |
| CAOS–E2 | passes | fails: 0/11/1 against stock col | newly fails |
| YURI–SHK | passes | fails: 11/1/0 against stock row | newly fails |

The four new failures and LTNK–E1's pass are seed noise of the same kind as before (one or a few
games of 12 flip a unanimous side). CAOS–APOC, CAOS–TTNK and CAOS–E2 failed before intake 87 too.
We changed nothing to fit them: r69 fixes the seed, and with it M-0019's *N*₀ now holds exactly.

The simulation code is unchanged apart from comments, and none of the sweeps, the AI quick lane or
the goldens goes through `oracle::setup`, so main's numbers are the same as the branch's by
construction; `cargo xtask gate` passes with no golden moved. The M6c, M7a and M7b sweeps (seeds 1
to 16, release) were run once on the branch and pass:

- M6c: smallest minimum over 16 seeds 0 (seed 16 ends at tick 5801; its jumpjet orders, charger
  orders and support beams counters are 0).
- M7a: smallest minimum 0 (seed 10's hostile intent seen by a sensor is 0); psychic sensor orders
  min 1, median 2.
- M7b (8000 ticks a seed): no drain counter below the margin on any seed (drain orders min 3,
  median 3; links formed min 3; mutator launches min 3, median 5; dominator launches min 6, median
  7). Of the M7a counters it prints, gattling, gas, bunkered vehicles, lifts and seizures are 0 on
  every seed; the margin applies to the drain counters only.

The AI quick lane was not run: the intake brief ties it to a change of draw order, and there is
none. `cargo test --release -p glasnost-ai -- --include-ignored` on the rebased tree: 246 passed,
0 failed.
