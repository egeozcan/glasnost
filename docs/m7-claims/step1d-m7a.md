# M7a step 1d claim ledger (spec revisions that landed on main after 8f5b743)

This is part 4 of M7a step 1. Each spec is compared with the revision the code had taken in at step 1c ([step1c-m7a.md](step1c-m7a.md)). The old state is the spec file at 8f5b743, the new one at 49bcef3 (`git diff 8f5b743 49bcef3 -- spec/<path>`). Every changed claim gets one class, as in [step1c-m7a.md](step1c-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *not built*: with the reason (a claim for a part not built yet);
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

Per-spec ledgers: [S-0601.md](S-0601.md), [S-0609.md](S-0609.md). The glossary and the template have no ledger of their own.

| Spec | Revision | Commits | Answers |
|---|---|---|---|
| S-0601 mind control | 12 (from 10; 11 7067bc6, 12 17b9604) | ef2533c | Q-1150 |
| S-0609 gattling spin-up | 5 (from 4, 517b9c9) | 0ebc932 | Q-1151 (and closes Q-1220: its rules are no longer proposals) |
| Glossary | 45 (from 44, cffad7e) | none needed | (with S-0601, Q-1150) |
| spec/TEMPLATE.md | editorial (3789980) | none needed | (with Q-1150) |
| spec/README.md | index rows of S-0601 and S-0609 | none needed | |

The other spec files in the diff are request files: Q-1150 and Q-1151 now read `status: answered`, and Q-1240 to Q-1244 were filed by the S-0613 build (bbeffc4), not by this intake.

Requests:
- **Q-1150** (answered by S-0601 revision 12 claim 5 and the glossary's *departure*, revision 45). The lead decided that a fault of the original on legal data is a departure with no switch. The interim rule (in both positions event 44 is raised only if the target still carries a tag after the event-6 raise, and then on that tag) becomes the claim, so the behaviour does not change. The switch `quirks.removed_tag_attack_event`, wired at step 1c, is removed from `quirks.rs` and the generated schema (following 94caa1a, which dropped S-0413's withdrawn switch). A ruleset that still names it is now refused by the strict schema.
- **Q-1151** (answered by S-0609 revision 5 Edge cases, *Stage count lowered below 2 by a later layer*). The interim rule (thresholds not used while the count in force is below 2) becomes the rule, keyed to the count as the layers finally leave it. One difference from the interim: the importer used to warn at every layer that lowered a populated type below 2 stages, even when a later layer raised it back. It now warns once, at the layer that reduced the count, and only when the final count is below 2 and the flag is set.
- New requests: none (block Q-1152 to Q-1159 unused).

Save format: unchanged at 49 (no state added or removed; the quirk switch lives in the rules, not in the save). AI game format: unchanged at 18. Hashes: the sim-gate goldens pass unchanged (run with `--include-ignored`), as expected: neither change alters what the simulation does.

## S-0601 revisions 11 and 12 (answer Q-1150)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1150) | (c) | |
| 5, *Judging*: a raise of event 44 after a tag has left is "governed by the next item" (was: by the switch) | (c) / pointer | |
| 5, *Tag removed by the first raise*: event 44 only on a tag the target still carries, whichever tag it is; **departure**, the original faults; no quirk switch (lead decision on Q-1150) | changed | `quirks.removed_tag_attack_event` removed (`quirks.rs`, `data/schema/ruleset.schema.json`). `mind::detonation` already looked the tag up again before the event-44 raise and skipped it without one; its citation now names the departure. `tags::claim_5_removed_tag_skips_event_44` (replaces `removed_tag_attack_event_both_positions`): repeat type 0, repeat type 1 with one and with two carriers, all of which lose the tag in the event-6 raise (S-0011 claim 10's table) and get no event-44 raise, and repeat type 2, which keeps the tag and does hear event 44. The takeover follows in every case, and the withdrawn switch is checked to be absent from `Quirks::named`. `every_spec_quirk_has_a_switch` stays green with no `WITHDRAWN` entry, because the spec no longer names the switch. |
| 5, Backing: the removed-tag departure is `unmeasured`; MC14 part a uses only a repeating tag, which cannot provoke the fault | (c) | |
| Edge cases, *Second attack event after a tag-removing first raise*: the raise is skipped (departure, no switch) | changed | As claim 5. |
| Changes named for other specs: S-0701 claim 39 needs no change; the glossary's *departure* and *quirk switch* gain the third case (done in glossary 45); S-0011 needs no change | (c) | S-0701 claim 39 is as built (`claim_6_building_taken`, `claim_9_released_building_announces`). |
| Questions for the project lead: none open | (c) | |

## S-0609 revision 5 (answers Q-1151; the Q-1220 rules approved)

| Claim | Class | Notes |
|---|---|---|
| Header (`requests` gains Q-1151) | (c) | |
| Formulas preamble: three engine rules (negative settings as 0, saturation, the final stage count), no longer proposed | satisfied | `spin_type_from_fields`, `saturation_at_the_32_bit_bound`, `layered_reduction_below_2_uses_no_thresholds`. Citations in `gattling.rs` and the tests no longer say "revision 4" or "proposed". |
| Constants, the saturation bound: no longer proposed | (c) | |
| Edge cases, *Stage count 0 or 1 from creation*: the layered case moves to the next item | (c) / pointer | |
| Edge cases, *Stage count lowered below 2 by a later layer* (new; engine rule, no switch): the final count governs; with fewer than 2 stages the thresholds in force are all 0, the cap is 0, the value never grows and the stage stays 0, even if an earlier layer populated them; the importer warns about the reducing layer; the thresholds are still held as imported (S-0006 claim 3) | changed (importer) / satisfied (simulation) | `SpinType::from_fields` already used no thresholds below 2 stages: `layered_reduction_below_2_uses_no_thresholds` (new; the held rows with a final count of 1 give all 0 and 20 spin-ups leave value and stage at 0, and with a final count of 3 the same rows are used as imported). The importer (`sim_combat::spin_settings`) still writes the held rows, and now warns only when the final count is below 2 (with the flag set), at the first layer of the final run below 2: `gattling_spin_settings` (new type `Back`, lowered to 1 and raised back to 2, keeps its thresholds and gets no warning; the old code warned at every lowering layer, so the no-warning check would fail there). |
| Edge cases, *Negative settings are used as 0* and *Saturation*: no longer proposed | satisfied | As at step 1c (`spin_type_from_fields`, `gattling_spin_settings`, `saturation_at_the_32_bit_bound`). |
| Import mapping: a type whose final stage count is below 2 uses no thresholds, with a warning about the reducing layer; negative values with a warning each | changed | As the Edge cases item. |
| Questions for the project lead (section removed) | (c) | |

## Glossary revision 45

| Claim | Class | Notes |
|---|---|---|
| Summary, *Document identity* shortened | (c) | |
| *departure*: a third case, a fault of the original (crash or hang) on legal data, fixed outright with no switch; the first case is S-0601 claim 5 | satisfied | S-0601 claim 5 above. |
| *quirk switch*: no switch for a fault of the original on legal data | satisfied | The switch is removed. The existing rule that an obvious bug with no gameplay impact gets no switch (PLAN D6) is unchanged. |
| *measurement setup*: "no engine add-on and no mods" summary; the condition of the measuring machine | (c) | Not applicable to the code: it concerns the oracle runs. |
| Measurement scenarios G1 to G3: the same wording | (c) | |

## spec/TEMPLATE.md

| Item | Class | Notes |
|---|---|---|
| Edge cases and quirks: a fault of the original on legal data gets no switch (departure) | (c) | A writing rule for specs; the code side is the glossary row above. |
