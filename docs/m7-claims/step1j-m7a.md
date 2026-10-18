# M7a step 1j claim ledger (S-0314 revision 25, S-0616 revision 6, S-0613 revision 9, S-0605 revision 6)

This is part 10 of M7a step 1. It covers the approved revisions that reached main after the step 1i intake: first 650a993 to de9dcf4 (`git diff 650a993 de9dcf4 -- spec/`), then the two that landed during this step, de9dcf4 to 05f671b (S-0613 revision 9 and S-0605 revision 6, merged in at the coordinator's instruction). Every changed claim gets one class, as in [step1i-m7a.md](step1i-m7a.md):
- *satisfied*: already true, with the test that shows it;
- *changed*: code changed, with the test;
- *pointer*: the claim only points at another spec's claim;
- *seam*: a claim for a part not built yet, with the seam's name;
- *(c)*: editorial (front matter, scenario text, rewording, backing and confidence).

No claim was renumbered, so no citation needed re-mapping.

| Spec | Revision | Answers |
|---|---|---|
| S-0314 healing and repair | 25 (from 24) | Q-1156 |
| S-0616 particle systems | 6 (from 5) | Q-1155 (and records Q-1153) |
| S-0613 gas clouds | 9 (from 8) | none (follows S-0616 claim 3) |
| S-0605 infantry absorber | 6 (from 5) | none (the cross-spec change S-0314 revision 25 asks for) |
| `spec/README.md` | | index rows moved and revised: (c) |
| `spec/requests/` Q-1153, Q-1155, Q-1156 (answered), Q-1157 (new, open) | | request status: (c) |

**Retired interims.** Q-1156 (what triggers the depot walk) is answered by S-0314 revision 25: there is no walk. The walk (`depot::queued_walk`, the `DepotState::walked` mark), its interim and its test `claim_23_queued_walk` are gone. Q-1155 (successors across kinds) is answered by S-0616 revision 6, and S-0613 revision 9 says the same. Its interim is gone from `particles.rs`, `gas.rs`, [S-0613.md](S-0613.md), [S-0616.md](S-0616.md) and [development.md](../development.md). Q-1153 was already retired at step 1h. The step 1h and step 1i ledgers are historical and stay as they were. The lead's "M7a step ledger" table in `development.md` is unchanged.

**The S-0605 cross-spec change.** S-0314 revision 25 says that it "needs S-0605 claim 4 revised so that an enter order on a repair depot makes the actor remember it". S-0605 revision 5 said, in general terms, that an enter order on a building makes the actor remember none, so the two disagreed for depots. S-0605 revision 6 landed during this step and settles it. Only an enter order on an *absorbing* building gives no remembered entry object. Setting up a wait clears any, and a damaged vehicle waiting for a depot remembers the depot and makes no approach. No request was needed.

**Q-1157 stays open** (the *guard* queued by a failed recycle approach while *eaten* is current). Its interim is unchanged: S-0215's ordinary start rules, `claim_3_failed_approach_gives_up`.

**New requests** (block Q-1158 to Q-1165):
- Q-1158: S-0314 claim 23 for a docked vehicle's enter order on its own depot (claim 20), and whether the processing that takes a free place gives the destination. Interim: claim 20's queued entry only; the retry gives the destination.
- Q-1159: S-0605 claim 3's "see Flag F1" names no flag. No interim is needed; the behaviour is as built.

Q-1160 to Q-1165 are unused.

**Formats.**
- Save format 52 → 53. `DepotState::walked: bool` became `DepotState::entering: Option<ObjectId>`, the remembered depot. `claim_23_queued_state_round_trips` and `depot_state_round_trips` check it.
- AI game format unchanged at 21: the AI makes no new decision.
- Goldens: regenerated; no golden file changed, so no gate game reaches the changed rows. The gate figures are the same as at step 1i. The M7a duel still plays all 12 000 ticks at seed 3. Over seeds 1 to 16 every minimum is at least 3, except the sensor orders (2).

## S-0314 revision 25

| Claim | Class | Notes |
|---|---|---|
| 23, first bullet: the order (no depot destination and no contact; *enter* queued without an immediate start; the depot recorded as the remembered entry object; the queued entry) | changed | `depot::order_applied` returns `OrderEffect::NoDestination`. `orders::apply_behaviour` then runs the order's other effects (S-0213 table T3: contact, target, attack-move memory, guard spot, queued destinations, the forgotten override, the absorber wait) with no destination, and queues *enter* (S-0215 claim 12). `DepotState::entering` is the remembered depot. Tests: `claim_23_queued_enter_current_no_walk`, `claim_23_queued_entry`. |
| 23: no walk; *enter* current at the next ready per-tick start and processed in that turn; the repeated request to the remembered depot; a full depot answers accept without contact; the vehicle stays where it is with no destination, standard delay | changed | `depot::entering_depot` and `depot::process_enter` take the target from the contact, then the remembered depot, then the destination (S-0215 claim 29). `depot::repeated_request` drops claim 16's free-place refusal for this request (`Reply::NoPlace`). `claim_23_queued_enter_current_no_walk` checks every tick before the first vehicle leaves: *enter* current, no destination, no contact, the same position, the queued entry and the memory kept. |
| 23: a free place at the processing: that request takes the contact; the retry finds it held, asks permission, and on a grant assigns the depot and clears the queued entry | changed | `repeated_request` takes the place. The processing gives no destination of its own (Q-1158 interim; `from_memory`). `vehicle_turn` then grants. Tests: `claim_23_free_place_taken_by_the_processing` (a two-place depot holding one contact: no movement before the retry) and `claim_23_queued_enter_current_no_walk` (it enters at most one tick after the first leaves). |
| 23: the depot refuses the request (switched off): contact ended, fall back to rest, the queued entry stays and the retry still runs | changed | `depot::end_contact` keeps the queued entry; `release` still clears both. `claim_23_switched_off_queued_vehicle`: it falls back with the queued entry kept, then gives up when the place frees (permission refused) and never enters. |
| 23: an *enter* made current by the order is processed in its first turn; one granted by the retry is first due on a later turn | satisfied | S-0215 claim 3(a) through `process::turn`, unchanged; `claim_23_queued_entry` (the retry's turn does not move the vehicle). |
| 23: an inactive remembered depot clears the queued entry | satisfied | `vehicle_turn`. Our engine forgets the remembered depot with it; a request to that depot would be refused anyway. |
| 20 (unchanged): a docked vehicle's enter order on its own depot | satisfied, Q-1158 | It keeps claim 20's queued entry only, with no *enter* and no memory (`claim_20_same_depot_enter_while_docked`). Claim 23's first bullet could also be read to cover it: Q-1158. |
| Table row "queued entry, *enter* queued" | changed | as claim 23; `claim_23_queued_enter_current_no_walk`. |
| Scenario H18 f(ii) falsifiers (a walk or any destination before the first leaves, a current behaviour other than *enter* while waiting, entry before the first leaves, no give-up when switched off) | changed | `claim_23_queued_enter_current_no_walk` and `claim_23_switched_off_queued_vehicle` check each falsifier on synthetic rules. The full-health repeat is `claim_23_queued_entry` (vehicle `c`). |
| Open questions (the walk's cell, *enter* current, the fall-back, the walk versus the retry: removed; the full depot's reply having any side effect: added) | (c) | We take the reply as accept with no other effect. |
| Header (requests list), backing and confidence | (c) | |

## S-0616 revision 6

| Claim | Class | Notes |
|---|---|---|
| 3: successors follow the system's kind, not the particle's (a gas system gives one successor, a smoke system claim 8's two, a fire, spark or railgun system none, for a dead particle of any kind) | changed | `particles::split_and_remove` (smoke systems) and `gas::second_pass` (gas systems) no longer filter on the particle's kind. Fire, spark and railgun turns already removed dead particles with none. `claim_3_successors_follow_the_system_kind` covers scenario P3 part c (i) to (iv). It checks counts, positions and velocity, and the raw values taken against twin systems whose type names no next type: (i) the creation draws only; (ii) 2 + 2 × (creation + 1); (iii) and (iv) none. Answers Q-1155. |
| 8: a dead particle of any kind in a smoke system; *d* from the dead particle's type | changed | as claim 3, case (ii). |
| 10 (fire turn wording) and Edge cases (smoke successor with *d* = 0) | satisfied | the fire turn makes no successors; the *d* = 0 case reads the dead particle's type, now of any kind. |
| Scenario P3 part c, header (requests list), backing | (c) | |

## S-0613 revision 9

| Claim | Class | Notes |
|---|---|---|
| 7: a dead particle of any kind in a gas system gets one successor | changed | as S-0616 claim 3: `gas::second_pass`; `claim_3_successors_follow_the_system_kind` (i), `claim_7_successor`. |
| Terms, *Successor* | (c) | the rule now follows the system's kind. |
| Header, backing | (c) | |

## S-0605 revision 6

| Claim | Class | Notes |
|---|---|---|
| 3: a vehicle ordered in to a vehicle absorber always waits in line, even at an empty building: no destination, no contact, no remembered entry object at the order; then claim 4's approach and retry | changed | `absorb::order_entered` with `start_waiting`. `claim_3_vehicle_always_waits_in_line` checks the state right after the order (`orders::apply_pending`), contrasts it with infantry at an empty infantry absorber, and the vehicle then goes in. The "see Flag F1" reference names nothing: Q-1159. |
| 4: setting up the wait clears any remembered entry object; a depot waiter (S-0314 claim 23) has no approach | changed | `absorb::set_waiting`. In our engine every path into a wait has no remembered object by then, so it is coded but cannot be tested. The depot side is S-0314 claim 23 above. |
| 5: an enter order on an absorbing building gives no remembered object; a depot order does | satisfied | `absorb::partner` (wording updated), `depot::order_applied`. |
| Terms (*remembered entry object*), cross-spec list, table row, scenario A2c's vehicle variant | (c), changed | the table row and A2c are claim 3's new rule. |
| Header | (c) | |

## Editorial

`spec/README.md` moves the S-0314, S-0616, S-0613 and S-0605 rows to the end with their new revisions: (c). Q-1153, Q-1155 and Q-1156 now read `answered`, and Q-1157 (open) is the request filed at step 1i: (c).
