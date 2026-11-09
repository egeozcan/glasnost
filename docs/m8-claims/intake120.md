# M8 A3 intake 120: S-0309 r129 and S-0404 r26

Brief: `.claude/worktrees/M8-A3-INTAKE120-BRIEF.md`.

Both revisions answer **Q-2841**, the entry test at a ground vehicle factory's exit point, which
intake 116 filed with an interim ([intake116.md](intake116.md)).

- S-0309 r126 → r129 (commit a24b022, from the r126 that intake 116 built, 4163d97). Only claim
  31's part on a unit that a vehicle factory has not yet sent out changes, with its Edge case
  text, its consequences, scenario A19 part k, a new Open question and the S-0417 claim 2 entry
  of Changes for other specs.
- S-0404 r23 → r26 (commit 62a2a57, from the r23 that intake 80 built, 23a1e47). r24 and r25
  were never committed on their own, and no ledger builds from them: the one spec commit in the
  range is r26. Claim 13's ground vehicle factory row gains the no-entry-test rule and the
  contact place rule, with their backing and confidence, and Open question 4 is new. The rest
  of the delta re-wraps claim 13's backing text.

The interim is replaced: a ground vehicle factory's release no longer tests the cell of its exit
point at all (`release::stranded_at` and both `design: Q-2841` notes are gone). No request is
filed; Q-2886 to Q-2895 are unused.

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| S-0404 13, ground row: the unit is put at the exit point without an entry test; K itself, any other occupant of any house, moving or not, and the cell's terrain and overlay never keep it out, and the cell never gives *failed* | tested | `release::ground` (the `stranded_at` test removed); `claims_s0404_r26::claim_13_no_entry_test_at_the_exit_point` (K itself at the foundation centre, an enemy vehicle, an own infantry unit, an own moving vehicle, an own vehicle standing still with no destination, water terrain), `claims_s0404::claim_13_ground_exit_point` |
| S-0404 13, ground row: K's max(*d*, 1) places, numbered from 1; the new unit takes the free place with the lowest number | tested | `release::ground` takes `first_free_place` over `contact_places` (our numbering starts at 0); `claim_13_the_new_unit_takes_the_lowest_free_place` (two places) |
| S-0404 13, ground row: no place free: the holder of place 1 loses its contact and tether link at the release, and the new unit takes that place | tested | `release::ground` (`econ::release::drop_place`, new); `claim_13_the_new_unit_takes_the_lowest_free_place` (two places, both held), `claim_13_one_held_place_is_taken` (one place: the holder keeps its position and behaviours) |
| S-0404 13, ground row: an exit point off the map | seam | S-0404 Open question 4; unchanged (the release puts the unit where `objects::place` puts it, as before) |
| S-0404 13, backing and confidence text | no change | text only |
| S-0309 31, *A unit that the factory has not yet sent out*: the factory's releases go on; the new unit appears in the cell of the unit standing there; its contact takes the factory's one place, the unit standing there loses its contact and tether link, keeps its behaviour and stays; the factory's unloading of the new unit starts by S-0417 claim 2 | tested | `release::ground`; `claims_s0309_r129::a_later_release_takes_the_stranded_units_contact` (scenario A19 part k: production order, no refund, contact and tether moved, the left unit unchanged for 300 ticks, then moved away; the door open until a closing stage) |
| S-0309 31, *A unit that the factory has not yet sent out*: an order that moves it while it still holds the contact ends its contact at its clear report | tested | `claims_s0309_r126::an_ended_unloading_leaves_the_door_open` (cite updated) |
| S-0309 31: whether the unit left standing holds up the new unit's exit track | seam | S-0309 Open questions; nothing is built for it. In our engine it does not: in `a_later_release_takes_the_stranded_units_contact` the factory puts V2 on its exit track, V2 leaves the exit point's cell and the factory is back in *guard*, all before V1 is moved (the test prints this and does not assert it) |
| S-0309 31: which unit a factory with more than one place sends out first in that case | seam | S-0309 Open questions; `release::contacted` takes the first in object order, as before |
| S-0309 31, Edge cases: `owner_change_interrupts_building_work`, its default (on) and its off position | no change | the r129 text drops only the failed releases from the on position's effects; default, doc string and both positions' tests (`claims_s0309_r126.rs`) still match. C7: still not admitted, re-read below |
| S-0309 31: the sale, the build-up, the depot repair and the domination reset | no change | the r126 → r129 delta has no hunk on them; intake 116's tests pass unchanged |
| S-0417 claim 2 as S-0309's Changes for other specs reads it: the released vehicle stays in contact with K until a later release takes its contact place | tested | as the S-0309 31 rows above |
| S-0404 switches (`refund_at_current_price`, `forced_start_player_hold`, `defence_line_partial_factory_check`, `unstartable_queue_head_stalls`, `foundation_exit_lists`) | no change | the r23 → r26 delta has no Edge cases hunk |

## Formats

A release that the interim failed (with a refund) now succeeds, so a saved game or an AI game
replays differently: save format 120 → 122 and AI game format 86 → 88 (the numbers the brief
reserves). The sim-gate goldens were regenerated and came out byte-identical: the golden runs
never release a ground vehicle onto a unit still holding its factory's contact.

## Lanes

Base: main 2b0e419 (intake 117's ledger row). After: this branch at 7f4f1db.

- M6c, M7a and M7b sweeps (seeds 1 to 16): identical to the base, line for line (M6c smallest
  minimum 0, M7a 1, M7b smallest drain minimum 3).
- Arena (`--record all --threads 2`): identical to the base, row for row: M-0001 12 of 96
  failing, M-0002 2 of 24, M-0003 2 of 35, M-0019 0 of 6, M-0021 11 of 50, M-0020 1 of 25.
- `glasnost-ai --include-ignored`: 264 pass, as on the base.

- Quick lane (`GLASNOST_AI_QUICK=1 ai-sides 120000 <seed> 0 2 8`, seeds 7 to 10, 36 games): 35
  games are identical. One changes length, seed 8 map 8 sides 0 and 2: 41817 → 42135 ticks,
  house 1 wins both. No winner changes.
- That game's first differing tick is 36839. In the base, war factory 7413 (house 0) fails a
  release through the interim, because its own vehicle 7622 stands at the exit point with K's
  contact, and refunds the unit. In the after build the release succeeds and 7622 loses its contact and
  tether link, as S-0404 claim 13 says. 7622 had stood there since tick 34672, when it was
  released while 7413 was still in its *construction* behaviour; 7413 went to *guard* at tick
  34678 without unloading it. That stand is older than this intake and is not changed by it.
