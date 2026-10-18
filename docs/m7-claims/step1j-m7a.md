# M7a step 1j claim ledger (S-0314 revisions 25 and 26, S-0616 revisions 6 and 7, S-0613 revisions 9 and 10, S-0605 revisions 6 and 7, S-0603 revision 6)

This is part 10 of M7a step 1. It covers the approved revisions that reached main after the step 1i intake: first 650a993 to de9dcf4 (`git diff 650a993 de9dcf4 -- spec/`), then the two that landed during this step, de9dcf4 to 05f671b (S-0613 revision 9 and S-0605 revision 6, merged in at the coordinator's instruction), and last 05f671b to ad7fffd (S-0314 revision 26, S-0613 revision 10, S-0605 revision 7 and S-0616 revision 7, merged in the same way), and finally ad7fffd to a588b69 (S-0603 revision 6, with main's S-0606 vehicle bunker, merged in the same way). Every changed claim gets one class, as in [step1i-m7a.md](step1i-m7a.md):
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
| S-0314 healing and repair | 26 (from 25) | none (editorial) |
| S-0613 gas clouds | 10 (from 9) | none (a gas successor also takes the speed) |
| S-0616 particle systems | 7 (from 6) | none (follows S-0613 claim 7) |
| S-0605 infantry absorber | 7 (from 6) | none (claim 16's fate for a vehicle victim; the driving case) |
| S-0603 grinder | 6 (from 4) | Q-1157 |
| `spec/README.md` | | index rows moved and revised: (c) |
| `spec/requests/` Q-1153, Q-1155, Q-1156 (answered), Q-1157 (new, open) | | request status: (c) |

**Retired interims.** Q-1156 (what triggers the depot walk) is answered by S-0314 revision 25: there is no walk. The walk (`depot::queued_walk`, the `DepotState::walked` mark), its interim and its test `claim_23_queued_walk` are gone. Q-1155 (successors across kinds) is answered by S-0616 revision 6, and S-0613 revision 9 says the same. Its interim is gone from `particles.rs`, `gas.rs`, [S-0613.md](S-0613.md), [S-0616.md](S-0616.md) and [development.md](../development.md). Q-1153 was already retired at step 1h. The step 1h and step 1i ledgers are historical and stay as they were. The lead's "M7a step ledger" table in `development.md` is unchanged.

**The S-0605 cross-spec change.** S-0314 revision 25 says that it "needs S-0605 claim 4 revised so that an enter order on a repair depot makes the actor remember it". S-0605 revision 5 said, in general terms, that an enter order on a building makes the actor remember none, so the two disagreed for depots. S-0605 revision 6 landed during this step and settles it. Only an enter order on an *absorbing* building gives no remembered entry object. Setting up a wait clears any, and a damaged vehicle waiting for a depot remembers the depot and makes no approach. No request was needed.

**Q-1157 is answered** by S-0603 revision 6, which arrived last: *eaten* does not hold back the *guard* queued by a failed recycle approach, so it starts by S-0215 claim 11's ordinary rules, as the interim had it. The interim is gone from [S-0603.md](S-0603.md) and [development.md](../development.md).

**New requests** (block Q-1158 to Q-1165):
- Q-1158: S-0314 claim 23 for a docked vehicle's enter order on its own depot (claim 20), and whether the processing that takes a free place gives the destination. Interim: claim 20's queued entry only; the retry gives the destination.
- Q-1159: S-0605 revision 7 claim 3 gives "a vehicle still driving from an earlier order" as not ready to change at its first per-tick start, so it makes no approach step (scenario A2c's driving case). S-0215 claim 11 makes a moving actor ready when its queued behaviour is *enter*, which the order has just queued. Interim: S-0215 claim 11 decides (`process::ready`); a driving vehicle with *enter* queued makes *enter* current and takes the approach step. (The request first asked about revision 6's dangling "see Flag F1"; revision 7 removed it.)

Q-1160 to Q-1165 are unused.

**Formats.**
- Save format 52 → 53 on this branch, then 54: main took 53 for the S-0606 vehicle bunker, so the merge with main made this step's format 54. `DepotState::walked: bool` became `DepotState::entering: Option<ObjectId>`, the remembered depot. `claim_23_queued_state_round_trips` and `depot_state_round_trips` check it. `StructState` gained `fate_tick` (S-0605 revision 7, below) in the same format.
- AI game format unchanged by this step: the AI makes no new decision. Main's S-0606 work made it 22.
- Goldens: regenerated. Before S-0605 revision 7 no golden file changed. After it every golden's hashes change, because `StructState::fate_tick` is new state. A regeneration with that field left out of the state changed no golden, so no gate game plays differently, and the gate figures are the same as at step 1i. The M7a duel still plays all 12 000 ticks at seed 3. Over seeds 1 to 16 every minimum is at least 3, except the sensor orders (2).

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
| 3: a vehicle ordered in to a vehicle absorber always waits in line, even at an empty building: no destination, no contact, no remembered entry object at the order; then claim 4's approach and retry | changed | `absorb::order_entered` with `start_waiting`. `claim_3_vehicle_always_waits_in_line` checks the state right after the order (`orders::apply_pending`), contrasts it with infantry at an empty infantry absorber, and the vehicle then goes in. (Revision 7 replaced the "see Flag F1" reference; see below.) |
| 4: setting up the wait clears any remembered entry object; a depot waiter (S-0314 claim 23) has no approach | changed | `absorb::set_waiting`. In our engine every path into a wait has no remembered object by then, so it is coded but cannot be tested. The depot side is S-0314 claim 23 above. |
| 5: an enter order on an absorbing building gives no remembered object; a depot order does | satisfied | `absorb::partner` (wording updated), `depot::order_applied`. |
| Terms (*remembered entry object*), cross-spec list, table row, scenario A2c's vehicle variant | (c), changed | the table row and A2c are claim 3's new rule. |
| Header | (c) | |

## Editorial

`spec/README.md` moves the S-0314, S-0616, S-0613 and S-0605 rows to the end with their new revisions: (c). Q-1153, Q-1155 and Q-1156 now read `answered`, and Q-1157 (open) is the request filed at step 1i: (c). The last merge moves the S-0603 row to the end at revision 6, and Q-1157 reads `answered`: (c).

## S-0314 revision 26

| Claim | Class | Notes |
|---|---|---|
| Terms, *Remembered entry object* (new entry: for a damaged vehicle with a queued entry, the depot) | (c) | as built at revision 25 (`DepotState::entering`). |
| 23: the parenthesis asking for S-0605 claim 4's revision removed | (c) | S-0605 revision 6 made that change. |
| Header | (c) | |

## S-0613 revision 10

| Claim | Class | Notes |
|---|---|---|
| 7: the successor takes exactly two quantities from the dead particle, overwriting its own creation's: the particle speed and the integer velocity; nothing else (deflection, damage countdown, lifetime, stage, translucency, direction) | changed | `gas::second_pass` copies `speed` as well as `vel`. `gas_successor_takes_the_dead_particles_speed` (in `claims_s0616.rs`): a gas parent (speed field 2.0) dying in a gas system with a smoke next type; the smoke successor's speed and velocity are the parent's, and its stage starts at its own. The successor's rise reads that speed by S-0616 claim 7, unchanged. `claim_3_successors_follow_the_system_kind` part (i) also checks the speed. The second G5-speed variant (a gas parent that is itself a smoke particle's successor) follows from the same copy and is not tested separately. |
| Terms, *Gas particle* (a particle speed in the state) | (c) | `Particle::speed` is shared by every kind. |
| Scenario G5-speed, backing and confidence | (c) | |
| Header | (c) | |

## S-0616 revision 7

| Claim | Class | Notes |
|---|---|---|
| 3, gas-system line: the successor takes the dead particle's speed and velocity and nothing else | changed | as S-0613 claim 7 above. |
| 3, backing and confidence | (c) | |
| Header | (c) | |

## S-0605 revision 7

| Claim | Class | Notes |
|---|---|---|
| 3: claim 16's fate assigning a vehicle victim a vehicle absorber applies the waiting-in-line rule at once, whatever the building's contact state: no contact from the fate, no destination, and no contact request in that tick; its first request is claim 4's retry | changed | The vehicle rule moved from `absorb::order_entered` into `absorb::seek_contact`, which the order, the fate and the instruction's repeat share. `StructState::fate_tick` marks the release tick, and `absorb::waiting_turn` sends no request in it, even from a turn later in that tick. Test: `claim_3_vehicle_fate_waits_in_line` (scenario A13 (g)). |
| 3: the vehicle's first update after an order: ready at its first per-tick start, *enter* current and one approach step; not ready, no step, contact held by the end of the update when the place is free | satisfied, Q-1159 | S-0215 claim 11 and S-0605 claim 4 as built; `claim_3_vehicle_always_waits_in_line` (standing case). The example "a vehicle still driving from an earlier order" for the not-ready case conflicts with S-0215 claim 11's queued-*enter* exception: Q-1159, interim: S-0215 claim 11 decides, so a driving vehicle takes the step. |
| Ordering consequences (claim 3's step at the release tick: a contact request for infantry, the waiting-in-line rule for a vehicle) | satisfied | as claim 3 above. |
| Scenario A2c's vehicle variant (the still-driving case), scenario A13 (g) | changed, Q-1159 | A13 (g): `claim_3_vehicle_fate_waits_in_line`. A2c's driving case: Q-1159. |
| Header | (c) | |

## S-0603 revision 6

| Claim | Class | Notes |
|---|---|---|
| 3: the *guard* queued by a failed approach is not held back by *eaten*; it becomes current at the first per-tick start or late start at which the unit is ready (S-0215 claim 11); a unit idle at the order has *guard* current at the end of the failing tick | satisfied | `process::ready` has no *eaten* case. `claim_3_failed_approach_gives_up` (a vehicle and an infantry unit idle at the order: *guard* current with nothing queued at the end of the failing tick and 10 ticks later). Answers Q-1157. |
| 3: a unit moving in the failing tick (a turn in place before a step counts) is not ready there, and *guard* starts after its locomotor stops | satisfied | `process::ready` reads `occupancy::in_progress`, which counts a turn in place. No test: the spec gives no scenario for it (`unmeasured`), and a moving vehicle with *eaten* current does not arise in our synthetic set-up, since *eaten* becomes current only when the vehicle is ready. |
| 3: a vehicle in contact with a vehicle factory, or standing on a factory's exit cell, stays in *eaten* with *guard* queued | satisfied | `econ::release::holds_back` in `process::ready` (S-0215 claim 11, as built). The spec says only that this case can arise. |
| Terms (*per-tick start, late start, moving*), scenario G9's expectation, Open questions, header (requests), backing and confidence | (c) | |
