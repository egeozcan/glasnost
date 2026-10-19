# M7a step 1i claim ledger (S-0212 revision 49, S-0603 revision 4, S-0605 revision 5, S-0506 revision 10, S-0214 revision 41, S-0616 revision 5, the setup-wording sweep)

This is part 9 of M7a step 1. It covers the approved revisions that reached main between the step 1h base and its merge, 850767f to 650a993 (`git diff 850767f 650a993 -- spec/`), and were not yet taken in. Every changed claim gets one class, as in [step1h-m7a.md](step1h-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *seam*: a claim for a part not built yet, with the seam's name;
- *n/a yet*: the spec is not built;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered, so no citation needed re-mapping.

| Spec | Revision | Answers |
|---|---|---|
| S-0212 pathfinding | 49 (from 45) | Q-1280 |
| S-0603 grinder | 4 (from 2) | none |
| S-0605 infantry absorber | 5 (from 2) | Q-1152 |
| S-0506 missiles | 10 (from 9) | none |
| S-0214 order targeting | 41 (from 40) | none |
| S-0616 particle systems | 5 (from 4) | none |
| S-0606 vehicle bunker | 3 (from 2) | handled by M7a step 3.7 |
| S-0607, S-0608, S-0612, S-0615 | one revision on each (S-0607 1 to 2, S-0608 2 to 3, S-0612 1 to 2, S-0615 1 to 2) | n/a yet; S-0612 revision 2 built at M7b step 3.5 ([S-0612.md](S-0612.md)) |
| `spec/glossary.md` | 54 (from 51) | (c) |
| every other spec (list below) | one revision on (S-0215, S-0404, S-0106, S-0522 two) | (c) |
| `spec/README.md`, `spec/requests/` Q-1152, Q-1154, Q-1155, Q-1156, Q-1280 | | index rows and request status: (c) |

**Retired interims.** Q-1280 (the path past a short line of buildings) is answered by S-0212 revision 49, and Q-1152 (the absorber approach's nearby location) by S-0605 revision 5. Their interims are gone from the code (`path.rs`, `econ/harvest.rs`, `structures/absorb.rs`, `depot.rs`), the tests (`claims_s0603.rs`, `claims_s0605.rs`, `claims_s0314_depot.rs`), [S-0603.md](S-0603.md), [S-0605.md](S-0605.md) and [development.md](../development.md). Q-1156 (what triggers the S-0314 depot walk) stays open: its walk now finds its cell with S-0605 claim 4's settings, which S-0314 claim 23 points at, but its trigger is still the interim. The lead's "M7a step ledger" table in `development.md` is unchanged.

**The AI's grinder workaround stays.** `glasnost_ai::grinder` sends a unit to a cell beside the grinder before the recycle order when a direct path would fail. With S-0212 revision 49 at the default `corridor_retry_no_fallback` (on, the original), a recycle click past a line of 7 buildings still fails at once (S-0603 claim 3, S-0212's short-line edge case; `claims_3_4_approach_past_a_line_of_buildings`), so the workaround is still needed.

**New requests** (block Q-1157 to Q-1165): Q-1157 (does the guard queued by a failed recycle approach start while *eaten* is current?). Q-1158 to Q-1165 are unused.

**Quirk switches.** One new one, `corridor_retry_no_fallback` (S-0212 claim 13 and Edge cases), default on (the original), tested both ways (`claims_3_4_approach_past_a_line_of_buildings`, `corridor_retry_no_fallback_off_passes_the_line`, `run_s_shifts_with_the_switch_on`). It loads from rules in both positions (`corridor_retry_quirk_loads_in_both_positions`). `glasnost_rules` `every_spec_quirk_has_a_switch` passes. The schema is regenerated.

**Formats.** Unchanged: save format 52, AI game format 21. No new state is kept (a quirk is a rule, and the rules hash already covers it), and the AI makes no new decision. The goldens were regenerated and no golden file changed (no gate game reaches the changed rows). The M7a duel still plays all 12 000 ticks at seed 3, and the minima over seeds 1 to 16 are those of step 1h, every one at least 3 except the sensor orders (2): see the Grinder section of [development.md](../development.md).

## S-0212 revisions 46 to 49

| Claim | Class | Notes |
|---|---|---|
| 13, 13a (the corridor search's retry table: a failed repeated coarse search) | changed | `path::corridor_search`: with `corridor_retry_no_fallback` on, a failed repeated coarse search fails the computation; off, one more cell search without a corridor, which is final. The first coarse search's failure still falls back to a cell search without a corridor. Tests `claims_3_4_approach_past_a_line_of_buildings`, `corridor_retry_no_fallback_off_passes_the_line`, `run_s_shifts_with_the_switch_on` (`claims_s0603.rs`), which also cover scenario P12 run S: with the switch on, exactly the placements (7, 0), (7, 1) and (9, 0) fail. |
| Departure D2 (a chain longer than 500 sectors fails, whatever the switch) | changed | `corridor::coarse_why` tells `CoarseFail::ChainLength` from `NoChain`; the switch-off cell search is skipped for `ChainLength`. |
| Edge case: a short line of buildings | changed | as claim 13. Answers Q-1280. |
| 23 row 2 (the give-up: destination cleared, *guard* queued; an AI house's unit as a human's, D5) | satisfied | `path::failed`; `claim_3_failed_approach_gives_up` (`claims_s0603.rs`). Whether the queued *guard* starts during *eaten* is Q-1157. |
| 25, table T5 rows *step-off* and *waiting absorber approach* (own class with winged as track, own zone, the centre cell's reach region read for the deck, deck mode on a bridge deck, no height condition, no further test, no reference cell, pick position *N* mod *n*) | changed | `NearbySettings::own_class` (`path.rs`), used by `econ::harvest::step_off_refinery` and `structures::absorb::approach`. Tests `claim_4_approach_one_free_cell`, `claim_4_approach_tie_by_tick`, `claim_4_approach_no_cell` (`claims_s0605.rs`) and the S-0402 step-off tests. |
| 25, table T5 row *no-move order, mover not free* | satisfied | already uses the move order's settings (`free_no_move`); the S-0213 no-move tests. |
| 20 (object and guard text now points at S-0207 claim 26) | pointer | |
| Header, Terms, measurement setup, scenarios, Open questions, backing | (c) | |

## S-0603 revisions 3 and 4

| Claim | Class | Notes |
|---|---|---|
| 3 (a recycle approach whose path computation fails more than 1 cell from the building gives up as S-0212 claim 23 row 2) | changed | the give-up was already `path::failed`; S-0212 revision 49 decides when the computation fails. `claim_3_failed_approach_gives_up`, `claims_3_4_approach_past_a_line_of_buildings`. Whether the queued *guard* starts while *eaten* is current is open: Q-1157, interim S-0215's ordinary start rules (it starts at the next ready moment). |
| 4 (the approach past a short line) | changed | as claim 3; the vehicle and infantry cases and a plain move fail alike at the default switch. |
| 15 (interruptions) | (c) | wording only; the ledger row in [S-0603.md](S-0603.md) is updated. |
| Header, measurement setup, scenarios G9 and P12 references, Open questions | (c) | |

## S-0605 revisions 3 to 5

| Claim | Class | Notes |
|---|---|---|
| 4 (the waiting entrant's approach: a free location near the reference point's cell by S-0212 claim 25's absorber row; no cell found, it falls back to rest and keeps waiting) | changed | `structures::absorb::approach` now uses `NearbySettings::own_class` from the reference point's cell and returns whether a cell was found; `process_enter` calls `process::fall_back` when none was. Tests `claim_4_approach_one_free_cell`, `claim_4_approach_tie_by_tick`, `claim_4_approach_no_cell`. Answers Q-1152. |
| Header, measurement setup, scenarios, Open questions | (c) | |

## S-0506 revision 10

| Claim | Class | Notes |
|---|---|---|
| Slot table (a flagged non-rocket object in a non-missile slot: not moved by the preparing rows, keeps claim 3's adoption, detached by claim 8) | changed | `spawn::slot_step`, *preparing* in a non-missile slot: after the adoption, a missile-type object is neither sent back nor re-sent. Test `flagged_non_rocket_slot` (both positions of the flag). |
| 8 (detach) | satisfied | `flagged_non_rocket_slot` ends with the slot empty for the flagged object. |
| 18 (a stalled missile at the map edge keeps its speed) | satisfied | `claim_18_map_edge` now also asserts the speed after the stalled tick. |
| F6 (the arctangent table: 4 097 entries, the entry chosen before the sign is narrowed) | satisfied | wording; `rocket.rs` comments re-cited. |
| Citation fix (S-0303 claim 5 to claim 3) | (c) | the S-0303 claim 5 citation in `claims_s0303.rs` is a different, correct use. |
| Header, measurement setup, scenarios | (c) | |

## S-0214 revision 41

| Claim | Class | Notes |
|---|---|---|
| Group click on a depot: the first selected object takes the contact | satisfied | `s0214_group_click_first_selected_takes_contact` (`claims_s0314_depot.rs`). |
| Unreachable inputs read literally | satisfied | the code follows the tables literally. |
| Cursor frames (engine constants), harvester truce (unreachable) | (c) | |
| Header, measurement setup | (c) | |

## S-0616 revision 5

| Claim | Class | Notes |
|---|---|---|
| Absent spark type (departure) | satisfied | already built in step 1h; the S-0616 spark tests. |
| Creators not yet built | seam | unchanged seams, as listed in [S-0616.md](S-0616.md). |
| Header, measurement setup, rewording | (c) | |

## S-0606 revision 3

Handled by M7a step 3.7 (vehicle bunker). No bunker code is touched here.

## Not built yet

S-0607 (magnetic lift) revision 2, S-0608 (drain) revision 3, S-0612 (psychic dominator) revision 2, S-0615 (special hit effects) revision 2: n/a yet. S-0612 revision 2 is built at M7b step 3.5 ([S-0612.md](S-0612.md)).

## Editorial (the setup-wording sweep)

Every spec below changed only by the measurement-setup wording ("director … off" became "no engine add-on and no mods", with its exceptions), revision and date bumps, and in some cases rewording, backing or Open questions. A diff filter that ignores the sweep's lines found no other behavioural change. Class (c) for each:

S-0001 4 to 5 (also "No switch" text and a lead question), S-0002 7 to 8, S-0003 18 to 19, S-0004 18 to 19, S-0005 8 to 9, S-0006 20 to 21, S-0007 13 to 14, S-0009 26 to 27, S-0010 18 to 19, S-0011 25 to 26, S-0012 4 to 5, S-0013 2 to 3, S-0014 3 to 4, S-0101 12 to 13, S-0102 20 to 21, S-0103 14 to 15, S-0104 24 to 25, S-0105 20 to 21, S-0106 10 to 12, S-0107 15 to 16, S-0108 13 to 14, S-0109 29 to 30, S-0110 11 to 12, S-0202 31 to 32, S-0203 36 to 37, S-0204 8 to 9, S-0205 8 to 9, S-0207 37 to 38, S-0208 32 to 33, S-0209 41 to 42, S-0210 21 to 22, S-0213 36 to 37, S-0215 60 to 62 (records Q-1154 and an open question), S-0301 21 to 22, S-0302 28 to 29, S-0303 16 to 17, S-0306 12 to 13, S-0307 15 to 16, S-0308 13 to 14, S-0309 19 to 20 (renames Disc to drainer), S-0311 11 to 12 (the list text limit now cited from S-0003), S-0312 31 to 32, S-0313 9 to 10, S-0315 8 to 9, S-0401 9 to 10, S-0402 28 to 29, S-0404 19 to 21, S-0405 17 to 18, S-0406 10 to 11, S-0407 19 to 20, S-0408 20 to 21, S-0409 14 to 15, S-0411 19 to 20, S-0412 13 to 14 (the C22 limit), S-0413 4 to 5, S-0414 14 to 15, S-0415 16 to 17, S-0416 2 to 3, S-0417 20 to 21, S-0501 16 to 17, S-0502 11 to 12, S-0503 6 to 7, S-0504 18 to 19, S-0508 8 to 9, S-0509 6 to 7, S-0511 7 to 8, S-0512 8 to 9, S-0513 17 to 18, S-0514 7 to 8, S-0515 4 to 5, S-0516 10 to 11, S-0517 8 to 9, S-0518 4 to 5, S-0519 4 to 5, S-0520 5 to 6, S-0522 5 to 7, S-0525 2 to 3, S-0609 5 to 6, S-0610 9 to 10, S-0614 11 to 12, S-0701 17 to 18.

The glossary (51 to 54) adds *bio-reactor fate*, refines the building passenger count (exactly 0 for I13; a negative count meets neither) and drops the "director" question: (c), the passenger-count refinement satisfied by the existing S-0605 tests.
