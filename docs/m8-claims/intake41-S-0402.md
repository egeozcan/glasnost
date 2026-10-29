# S-0402 r80: claim ledger, M8 step A3 intake 41

This intake takes in one spec revision, which answers Q-1990 (round 5 note 23, a slave miner with
no order far from a field; [a4-play-notes.md](a4-play-notes.md)).

- **S-0402 revision 80** (spec commit 23661b9; the diff against r76, 34dd574, which intake 36
  built). The rule changes, all in claim 14:
  - *Who takes the owner alternative.* In the original every computer player of a skirmish
    takes it, and so do the neutral and special houses. Such an owner wakes on the kick test on
    *s*<sub>m</sub> and a group at rest alone. While the miner stays in the same *guard*, every
    later processing that finds the group at rest wakes it again. In our engine the alternative
    is the neutral and special houses' and a test-only computer-controlled player house's. An
    AI house takes the human-controlled alternatives, so a person's or an AI house's miner far
    from a field never wakes by itself.
  - *Claim 13's guard rows and the slave miner.* The wake-up rule is tested first, and a wake
    ends the processing. Claim 13's rows are tested only after a processing that did not wake,
    and only for a type with the harvester or the weeder flag. A slave miner type with neither
    gets none of them. A type with both an enslaves field and the harvester flag gets row 2 for
    a computer-controlled owner with a dock-type refinery and an unset no-field record.
  - DECIDED (autonomy, 2026-10-28): our AI gets the original computer player's behaviour with
    an ordinary order, a harvest order on a field cell (claim 22).
  - Import mapping: the `Harvester` row now also feeds whether claim 13's rows apply to a slave
    miner (claim 14).
  - Backing and confidence of claim 14, scenario H7 part (g) (new) and the void-rule note of the
    scenarios' preamble.
  - Changes for other specs: S-0215 claim 24's backing (H7 variants (d) and (g)) and its
    sentence on player houses (the test-only computer-controlled player house takes row 2's
    case and the owner alternative).
  - Overview: the Q-1990 answer.

The cross-specs it names were read as they stand: S-0215 claims 24 and 26 (unchanged, r76's
vehicle table: row 1 the wake-up rule, row 2 a guard rule that queues *harvest*), S-0213 claim 3
(a human stop order to a computer player's unit; no effect in our engine, where every player
house is human-controlled). Neither was revised for this intake.

**Code.** The engine already had every rule of r80; this intake changes no behaviour.
- `econ::harvest::slave_wake_rule_holds` takes the owner alternative from `House.human`, which
  is clear only for the neutral and special houses and a test-only computer-controlled player
  house; AI houses are human-controlled (glossary *AI house*). Cite text updated for Q-1990.
- `process::process_guard` tests the wake-up rule (vehicle row 1) before
  `harvest::guard_rule` (row 2), and `guard_rule` returns at once for a type without the
  harvester flag; `UnitHarvest::harvester` is imported from `Harvester` separately from
  `slave_miner` (resource gatherer, resource destination and an enslaves field). New cite on
  that test.
- Our AI (`ai::scripted`, step 4 *Harvesters*, since commit 284977a): a slave miner resting
  (*guard*, *area guard*, *stop* or nothing current) with its slaves home is sent to harvest on
  the nearest resource cell the house has explored; it finds its site and deploys under claim
  22. **No AI file changed** (step B8 runs in `crates/deterministic/ai`).
- No new saved or hashed state; save format 83 and AI game format 45 unchanged.

**Design** (*Decisions of 2026-10-25*). The original's computer player seeks with the long scan
over the whole map; our AI picks only cells its house has explored (PLAN §4.7: it sees only its
house's shroud). A computer house that has explored no field leaves its miner resting until it
does. This is a design point of the own AI, not a seam.

| Claim or item | Status | Notes |
|---|---|---|
| 14 (owner alternative: neutral and special houses, test-only computer-controlled player house; not a person's house, not an AI house) | tested | `claim_14_far_from_a_field_wakes_only_for_a_computer_controlled_owner` (four houses, each miner more than the short scan from the field, off tiberium land, kick delay 300, 3000 ticks): the person's and the AI house's miners never wake, never move and keep *s*<sub>m</sub>; the neutral and the test-only house's wake at their first *guard* processing with *t* > 300 + *s*<sub>m</sub>, still in that *guard*, then seek with the long scan, travel and deploy by the field. `claim_14_wake_rule` (the rule's inputs) unchanged. |
| 14 (every later processing of the same *guard* that finds the group at rest wakes it again) | tested | `claim_14_computer_controlled_owner_wakes_again_in_the_same_guard` (no field at all: at least 3 wakes in 2000 ticks, *guard* and *s*<sub>m</sub> unchanged throughout, no *harvest* queued; the person's group never leaves rest, last-rest 0). |
| 14 (claim 13's rows do not apply to a type with neither the harvester nor the weeder flag) | tested | The first test's SMVD pass: a type with a dock type and no harvester flag, each owner with a dock-type refinery and an unset no-field record; nothing queues *harvest* before the wake and *s*<sub>m</sub> stays. |
| 14 (wake-up rule first; claim 13's rows after a processing that did not wake, for a type with the harvester flag) | tested | `claim_14_harvester_flagged_slave_miner_takes_claim_13_rows_after_no_wake` (SMVH: before the kick delay row 2 queues *harvest* for the computer-controlled owners only; with a kick delay of 0 their first processing wakes and nothing was queued). What follows that *harvest* is claim 22's, tested by `claim_22_harvest_processing`. The spec marks the case `unmeasured`. |
| 14 (weeder flag) | waits | The weeder's row belongs to a later revision (claim 13, Edge cases), as before. |
| 14 DECIDED (our AI orders the miner to harvest a field cell) | tested | `a_resting_slave_miner_far_from_a_field_is_sent_to_harvest` (`crates/deterministic/ai/tests/ai.rs`): one think gives a harvest order on an explored resource cell to a miner resting 20 cells from its house's field; the order takes it there and it deploys by the field. Behaviour since commit 284977a; no AI change. |
| Import mapping, `Harvester` (feeds claim 13's rows for a slave miner) | tested | Imported as before (`UnitHarvest::harvester`); exercised by the SMVD and SMVH cases above. |
| Backing, scenario H7 part (g), the preamble's void rule | local lane only | The lane scene below plays our AI house, not the original's computer player; H7 itself is the spec workspace's. |
| Changes for other specs (S-0215 claim 24: backing, the test-only house) | by construction | Other spec's text. Our code already gives the test-only house row 2's case (`guard_rule`'s `!h.human` branch) and the owner alternative. |
| Overview (the Q-1990 answer) | wording | |

**Local lane** (`crates/app/tests/harvest_lane.rs`, new test
`far_slave_miners_of_a_person_and_a_computer`; outputs in the git-ignored
`local-lane/m8/a3-41/`). After the game start (the local house deploys its base unit, every player
house allied), one slave miner of the local house (a person's) and one of the computer house (an
AI house) are placed by their bases on a cell off tiberium land with no tiberium land within the
short scan, queued to harvest as a factory release does (so they come to rest in *guard*), and
watched for 4500 ticks.
- Map 0: the computer's miner had its harvest order 13 ticks after placement, moved at 50,
  deployed at 311 and had its first slave out at 371; its slaves then worked. The person's miner
  stayed in *guard*, at rest, with no destination and no slave out, for all 4500 ticks.
- Map 8: the computer's 13, 74, 451 and 511. The person's never had a destination or a slave
  out; a vehicle of the computer house hit it at 1557 and it was gone at 2882 (the lane's peace
  only writes the sim's alliances; our AI still sent a unit at it). Unmoved until hit.
- The scene also fixed the lane's map choice for a map index above 0 (the setup screen is drawn
  before each map step).

**Gate and margins.** Rebased onto main 6ba9e41 (step B8 merged as 0107a3a). The spec commits
on main since 5ff2349 that this intake does not take in: S-0401 r17 (a7aa0d9), M-0013 (a0c3411),
S-0212 r87 (3c4ef1e), S-0417 r68 (a7dc3d7) and the Q-1980 answer (1b56e06). The change to source
is cite text only.
- `cargo xtask gate`: ok. Every golden matches on native and wasm32 and none was regenerated.
- Seed sweeps 1 to 16 (M6c, M7a, M7b; with and without the play fixes): each counter's minimum,
  median and maximum are the same before (main) and after (this branch). The slave-miner rows are
  M7a with 12, 13 and 15 buildings placed or deployed and 7423, 7573 and 7713 slaves at work, and
  M7b with 4, 12 and 12 buildings and 3163, 3643 and 5323 slaves.
- Arena (`--record all`, local lane): our CSVs are byte-identical before and after. M-0001 has
  30 of 96 pairings failing and 150 of 171 unanimous stock outcomes reproduced, M-0002 12 of 24
  and 42 of 47, M-0003 2 of 35 and 60 of 64.
- Main's `glasnost-rules every_spec_quirk_has_a_switch` (S-0401 r17's new quirk, for intake 42)
  and the stock-guard lines of M-0013 and M-0015 fail on main as well; neither is this intake's.
