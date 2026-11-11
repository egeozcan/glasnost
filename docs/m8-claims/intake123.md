# M8 A3 intake 123: S-0508 r19 (garrisons; Q-2926 and Q-2927 answered)

Brief: `.claude/worktrees/M8-A3-INTAKE123-BRIEF.md`.

S-0508 r13 → r19 (commit 2da7bcd). It answers **Q-2926** (f38bd6c) and **Q-2927** (0c9aa37), both
filed by step A6b ([a6b-arena-triage.md](a6b-arena-triage.md)). The delta: claim 1's paragraphs
*Units on their way count for nothing* and *"May occupy" in other specs*, with their backing and
confidence; claim 4's unchanged target and *No hunt after a refusal (original only)*; scenario G21;
and two Open questions entries (the answers, and what a refused unit does after the scatter
request). It adds no quirk name (no `quirks.` token in the delta; `every_spec_quirk_has_a_switch`
passes).

Status words as in intake 76's ledger: **tested**, **partly**, **seam**, **no change**.

## What changed in the code

- **"May occupy" for an assaulter (Q-2927).** A6b built S-0215 claims 23 and 27's "may occupy" as
  claim 1's test for every type, so an assaulter type that is not an occupier type never passed.
  r19 gives it claim 14's assault eligibility. `garrison::may_occupy` now holds the rule (claim 1's
  test for an occupier type, a type with both marks included; claim 14's for an assaulter that is
  not an occupier; never for neither mark), and both `process::computer_taker_target` and the
  arrival test `garrison::attempt` (claim 2) call it.
- **Capture's hunt rule (S-0215 claim 28).** r19 claim 4 states that the rule leaves out occupier
  types only and that an assaulter that is not an occupier is not left out. Our engine had not
  built the rule for any house: the seam ledger ([seam-S-0215-claim28.md](seam-S-0215-claim28.md))
  read claim 28's "In our engine the hunt rule never applies" for every house, but S-0215's Terms
  (*houses in our engine*) say such a sentence speaks of the player houses, and that the neutral and
  special houses keep the rule; the *Test-only computer-controlled player house* term gives it to
  the arena houses. `process::capture_hunt_rule` now applies it in rows 2 (no target after the
  fall-back) and 4 (a destination, no target) for those owners: the destination is cleared and
  *hunt* queued, replacing what the fall-back or scatter queued. Occupier types and *eaten* are
  left out, and an aircraft that S-0501 claim 34 holds (in *retreat* or an approach behaviour, not a
  strike aircraft) ignores it, as S-0501 claim 49 gives.

Formats: save 126 → **127**, AI game 92 → **93** (both changes alter what a saved game replays to,
for the neutral and special houses' units in *capture* and *sabotage* in any game). Goldens:
regenerated on the final tree (see the report).

## Ledger

| Claim part | Status | Where, tests |
|---|---|---|
| 1 (r19): units on their way count for nothing; the count is that of the occupants inside, and any number may hold the building at once | tested (no code change) | `targeting::may_garrison` compares `garrison::count`, the occupant list; nothing reserves a place. `claims_s0508_r19::units_on_their_way_hold_no_place` (capacity 3, five computer occupiers all go, a sixth still finds it, two refused) |
| 1 (r19): this holds in S-0309 claim 25 (e) and S-0215 claims 23 and 27 | tested | the same test (claim 27's hunt row and 25 (e)'s range search) |
| 1 (r19): "may occupy" in S-0215 claims 23 and 27, an occupier type: claim 1's test | tested | `garrison::may_occupy`; `claims_s0309_c25e_occupy::hunt_computer_occupier_goes_for_the_building`, `attack_computer_occupier_goes_for_the_building` |
| 1 (r19): an assaulter type that is not an occupier type: claim 14's assault eligibility | tested (changed) | `garrison::may_occupy`; `claims_s0508_r19::hunt_computer_assaulter_goes_for_an_eligible_building` (a hostile garrisoned building), `attack_computer_assaulter_keeps_attacking_an_empty_building` |
| 1 (r19): never for a type with neither mark | tested | `claims_s0309_c25e_occupy::hunt_computer_rule_needs_an_occupation` |
| 1 (r19): an infiltrator type takes the target and is forced into *capture* whatever the test gives | tested (no code change) | `claims_s0508_r19::attack_computer_infiltrator_goes_whatever_the_test` |
| 1 (r19): S-0309 claim 25 (e) asks the test only of a searcher of an occupier type | tested (no code change) | `acquire::candidate` uses `garrison::may_enter`; `claims_s0508_r19::computer_assaulter_never_finds_a_passive_building` |
| 1 (r19): part (e) has no behaviour condition; in *area guard* the search is area guard's own | tested (no code change) | `claims_s0508_r19::area_guard_search_picks_the_passive_building` |
| 1 (r19): a queued *hunt* runs no search; the attack-move search when the unit carries that memory; the opportunity search does not run in *area guard* | no change | S-0215 claims 26 and 33 as built; r19 restates them and carries no backing of its own for them |
| 4 (r19): the refusal does not change the target | tested (no code change) | `entry::refuse` clears the destination and sends the scatter only; `claims_s0508_r19::refusal_keeps_the_target` (calls `garrison::attempt` at a full building) |
| 4 (r19): the force into *capture* removed a current or queued *hunt* (S-0215 claim 9) | tested (no code change) | `claims_s0508_r19::forced_capture_removes_a_queued_hunt` |
| 4 (r19): the force does not touch a suspended *hunt*; a later revert can make it current | tested (no code change) | `claims_s0508_r19::forced_capture_keeps_a_suspended_hunt` (`quirks.stale_suspended_revert` on) |
| 4 (r19): capture's hunt rule leaves out every occupier-type unit by type; an assaulter that is not an occupier is not left out | tested (built) | `process::capture_hunt_rule`; `claims_s0508_r19::capture_hunt_rule_leaves_out_occupiers_only`, `capture_hunt_rule_not_for_a_human_owner`; `claims_s0215_capture::claim_28_row_4_the_hunt_rule_for_the_neutral_house`, `claim_28_a_computer_engineer_without_a_target_hunts`, `claim_28_a_neutral_landed_aircraft_in_capture_queues_hunt` |
| 4 (r19): a new order, a map action or another spec's rule can also give *hunt* | no change | those rules as built |
| Open question: what a refused unit does after the scatter request | seam (spec open) | Ours: the refused unit keeps the building as target unless another rule drops it. While the scatter destination is held, claim 28 row 3 changes nothing; with no destination, row 1 gives it the building again. In A6b's watched arena games the target was gone by the next *capture* processing (row 4 while the scatter destination was held, then row 2's fall-back to rest once it was reached): a full passive building no longer passes the computer occupier's candidate test, so the target-keeping rules of S-0309 do not hold it. No rule was added here |
| G21 | not run (oracle scenario) | the lead's; see *Arena* below |

Requests: **Q-2951** (below). Q-2952 to Q-2960 are unused.

Seam that the hunt rule now reaches: S-0501 claim 49's "a cleared destination keeps the flight's
point" (intake 76's row: "arises only through the hunt rule") now arises for an aircraft of the
neutral or special house, or of an arena house, in *capture* or *sabotage* in flight. No aircraft of
those houses is known to be in *capture* or *sabotage* in the arena or the lanes; it stays a seam.

## Arena

Base main ef9f0ae, after this branch: M-0001 fails 14 of 96, M-0002 5 of 24, M-0003 2 of 35, M-0019
0 of 6, M-0020 1 of 25 and M-0021 11 of 50, before and after. Every game of every record plays the
same (the stored per-game CSVs are identical). The arena's E1 and E2 are occupier types, so neither
change reaches them, and no arena type is an assaulter that is not an occupier.

The rows A6b attributed to Q-2926 (none to Q-2927), with our counts over seeds 0 to 63 on this
branch (identical to A6b's):

| A6b row | Pairing, side | Record | Ours, 64 seeds | r19 |
|---|---|---|---|---|
| 10 | M-0001 TELE–E2, B | col | none 64 | explains it: the chain is the original's, so record and spec disagree → **Q-2951** |
| 18 | M-0002 INIT–E1, B | row | col 54, none 10 | as row 10 → **Q-2951** |
| 20 | M-0002 YURI–E1, B | col | row 46, col 16, none 2 | as row 10 → **Q-2951** |
| 9 | M-0001 TELE–E1, B | col | col 31, none 33 (*p*⁶ = 0.013) | within noise; named in Q-2951 |
| 21 | M-0002 YURI–E2, B | col | col 36, row 26, none 2 (*p*⁶ = 0.032) | within noise; named in Q-2951 |
| 4 | M-0001 YTNK–E1, B | row | row 41, col 23 (*p*⁶ = 0.069) | within noise; named in Q-2951 |

No row is fixed by this intake. Under PROVENANCE the measurement wins where it disagrees with r19:
**Q-2951** names the rows and the claims (S-0508 claims 1 and 4; S-0215 claims 9, 14, 23, 27, 28;
S-0309 claim 25 (e)), asks which step the original takes differently, and points to G21 on
M-0001's set-up. That part stops there; nothing was tuned toward the records. Wishlist A6b-1 now
names G21 and Q-2951.

**Since then:** S-0217 (new) and S-0508 r39 answered Q-2951; intake 124 built them
([intake124.md](intake124.md)). What the rows above do after that, and the request it filed for
the rows that still disagree (**Q-2987**), are in that ledger.

## Play-fixes layer

r19's original-only rules here (no *hunt* after a refusal for an occupier type; units on their way
not counted) reach only owners that follow the computer-controlled rules: the neutral and special
houses and the arena's test houses. Our AI houses and the player are human-controlled (S-0215
Terms), so in a normal game no player-owned or own-AI unit is affected, and a player would not feel
it as a bug. No design switch is added (brief).
