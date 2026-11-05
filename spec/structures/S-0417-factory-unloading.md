---
id: S-0417
title: Factory unloading (doors, exit movement and rally destinations)
topic: structures
revision: 76
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-05
measurements: []
requests: [Q-0221, Q-0222, Q-0223, Q-0697, Q-1070, Q-1110, Q-1757, Q-1850, Q-1980]
supersedes: []
---

# S-0417: Factory unloading (doors, exit movement and rally destinations)

## Summary

Factory doors, apron clearing (while unloading and while an idle vehicle factory guards), vehicle
exit movement, the first destinations of released infantry, the tether given to a released aircraft,
the destination chosen on release from contact, and the release details that S-0404 leaves to this
spec (claim 14). S-0404 claim 13 owns release positions, facings, busy and failed results, and
refunds; S-0404 claim 15 owns aircraft pad positions. Skirmish only: campaigns and network games are
out of scope. This answers Q-0221, Q-0222 and Q-0223 and most of Q-0120. Q-0697's
teleporting-infantry clear report is specified in claim 10. Q-1070's jumpjet ascent clear report is
claim 15: normal factory-released flight does not hold the factory until landing or destination
arrival. Q-1757 is answered in part by claim 17: an infantry unit resting in *guard* or *area guard*
on a cell that holds a building that counts there is sent a scatter request in each of its turns
while those conditions hold, so a walking or teleporting infantry unit released with no rally point
onto a cell that holds the factory is sent off it from its first turn in which it rests there, when
its request gets a destination (S-0208 claim 13, with that claim's fall-back to the neighbours).
Claim 17 answers Q-1757 questions 1 and 2, rated medium until U10 part b is run, and question 3 in
part: whether a later release is busy with stock types is Open question 10. Question 5's cloning and
teleporting parts are answered at medium until U10 parts b2 and e are run, and its jumpjet part is
Open question 3. Q-1757 question 4 is answered in claim 10: a rally point set after the release does
not move the resting unit. That answer is backed by U10 b, which has not been run (Open question
13). An infantry unit released with no rally point does not rest on the foundation holding the
contact place; claim 17 normally sends it the scatter request in its first turn, and it walks off
(its exceptions: a refused request, an accepted one that gets no destination or, for a walking unit,
no path, a missed turn, S-0203 claim 10), so a later release from that factory is busy only during
that walk. Q-1850 questions 1 to 3 are answered in claim 17 (*Unit after unit*): each unit normally
receives, in the tick after its release (claim 17's exceptions as above), a destination on a cell of
its own off the foundation. When claim 17's conditions hold for every earlier unit, and each earlier
walk off the foundation finishes before the next unit's request, every queued unit comes out at its
release unless an earlier unit's first arrival comes 54 ticks or more after its release, or S-0404
claim 13 gives the release another failure. That walk is expected to end within the 54 ticks between
releases at a stock infantry speed, but this spec has no figure for a stock walking speed, and only
U10 f would show it. This is rated medium until U10 parts b and f are run, and only on that
condition. Q-1980 is answered in claim 17 (*No path*). A walking unit whose fresh search fails at
the reachability gate loses that destination in the same movement update, and its movement in
progress ends, so the next request is accepted with its draw (question 1). A destination different
from the one it holds ends a running retry delay (S-0212 claim 21), so no walking unit given a
different destination holds it unsearched because the delay runs. Whether giving it the destination
it already holds ends the delay is left open, as S-0212 leaves it. A walking unit that keeps its
destination after a failed search waits for the delay with it, but its movement in progress ended at
the failure, so the first request after the failure is accepted with its draw (question 2). For a walking unit other than a
fraidycat whose current behaviour has the scatter property (claim 17, *During the walk-off*), each
accepted destination that differs from any destination it kept is searched for without a wait, so a
unit whose every scatter destination fails, each differing from any destination it kept, takes a
request's draw in each turn in which a request comes (question 3). Where a request gives it the
destination it kept, whether later requests are refused is not established (Open question 16).
Question 1, and the search for a new destination without a wait after the gate's failure, are rated
medium (`unmeasured`: U10 g needs an order at the unit's release, which only a harness not
established here can give). That a new, different destination ends a retry delay still running is
rated medium (`unmeasured`), as S-0212 claim 21 rates it. The kept destination of question 2 and the
repeated cycle of question 3 are rated low (both `unmeasured`). Still open: whether a released
jumpjet infantry unit is in *guard* or *area guard* over K's foundation and so receives that request
(Open question 3), whether a busy second finished item differs between factories with one contact
place and those with several (Open question 10), and the draws of putting a released unit on the map
(Open question 8). Claim 14 owns the contact-place count, infantry candidate acceptance and
edge-entry nearby destination, including the search's no-draw rule; their untested cases remain
qualified there.

**Paired hand-offs.** S-0404 claim 13 retains the fulfilled release hand-off: release queues
*unload*, and contact-place count, infantry candidate acceptance and edge-entry nearby destination
refer to claim 14 here. Nearby-search draws are resolved as none. Initial-placement draws remain
open. S-0212 claim 25 table T5 includes the aircraft factory edge-entry-without-rally row with
claim 14's parameters, and claim 25 uses the release tick to select its destination. S-0215 claim
11 records that factory release and unloading neither set nor clear the ready mark, with start
consequences owned by claim 2 here. Its claim 13 and S-0207 claim 21 jointly own restoration and
retained teleporting state after claim 5's temporary drive. The decided apron off-state rules
remain in Edge cases. Q-1110 question 3 is answered by claim 16. For this hand-off, S-0207 claims
25 and 27 must cite its factory-exit lifecycle, and S-0402 claim 17 must qualify teleporting travel
by the active locomotion, including the first factory-built chrono miner's driving leg (S-0402
claim 17 carries this). Q-1110 questions 1 and 2 belong to S-0207 claim 27.

## Terms

Contact, tether link, foundation, exit cell, exit displacement, barracks door kind, guard spot,
scatter request, threat point, forced, move-anywhere and locomotor use the glossary and S-0404's
terms. Behaviours, queueing, processing, falling back to rest and standard delay are S-0215's.
Track, drive vehicle, speed fraction ρ, target fraction ρₜ and progress unit are S-0207's.
The nearby-cell search is S-0212 claim 25. Door drawing is S-0108's, animation playback S-0109's.
Armed and unarmed use S-0215 claim 32's building weapon test.

- **Factory K, unit u:** the building and the unit it has just released. Its foundation's top-left
  cell is (*x*₀, *y*₀).
- **Unloading:** K's current behaviour is *unload*, specifically to release a produced unit.
  Passenger unloading, absorbed units and non-factory buildings are outside this spec.
- **Idle apron clearing:** the apron test and requests of claim 4, made by a vehicle factory in
  *guard* (or *sticky* or *area guard*) rather than while unloading (claim 13). It is the only
  rule for those behaviours that this spec gives.
- **Door:** closed, opening, open or closing. Open and closed mean at rest. Movement takes the
  door time of F2. This spec specifies only the factory-unloading uses of the door.
- **Exit apron cell E:** the target cell of a ground vehicle factory's exit track (claim 3).
  It need not belong to K's foundation, even when the exit list has eleven or more entries.
- **Apron neighbours:** E's eight neighbours, in step-direction order 0 to 7 (S-0204 claim 9).
- **Exit track:** table T1, distinct from the ordinary movement tracks of S-0207.
- **Clear report:** a unit's notice that releases its contact and tether (claims 7, 10 and 15).
  The name does not mean that it has physically left the foundation.
- **Run-away reply:** the reply a vehicle factory gives to that report, governing claim 8.

## Behaviour

1. **Factories that unload.** A ground vehicle factory releases u into contact with K and queues
   unloading; claim 2 governs when it becomes current. A naval vehicle factory queues unloading
   only when it had no unit in contact before
   release; it does not establish contact with the new ship. An infantry factory and a cloning
   building use contact hand-over, without entering the unloading stages. An aircraft factory
   does not enter these stages. A later release while a ground factory unloads passes to another
   factory or gives busy, exactly as S-0404 claim 13 states; busy does not retain the finished item
   for retry (S-0404 claim 7).
   - Backing: `measurable` (U1 for factory kinds; S-0404 P12 for busy/refund).
     The naval start with pre-existing contact and cloning contact hand-over are
     `unmeasured` here: U1 creates neither setup.
   - Confidence: high

2. **Ground vehicle unloading stages.** Table S1 gives the stages. Each processing handles one
   stage and assigns one standard-delay draw over 0 … 2, including an unsuccessful processing.
   The delay is F1; it is not necessarily the interval between processing ticks when it is 0 or 1
   (S-0215).
   At unloading start, the contacted unit starts *guard* without the ordinary readiness test, the door starts opening, and K stops its idle animation and starts its production animation when that slot has art (using the damaged variant as S-0108 specifies). Apron clearing proceeds without waiting for the door: a blocked apron receives claim 4's requests and remains in that stage; a free apron advances to letting out without sending u out. Letting out waits for an open, stationary door. If contact is absent, K starts closing; otherwise K queues *move* on u, applies claim 5 and waits for the tether to end. The waiting condition is tether end, not u's position. K then starts closing; closing waits for a stationary closed door, then falls back to rest, stopping the production animation and queuing *guard*. Unloading ends when *guard* starts.

   **When the queued *guard* starts (Q-0222).** Falling back to rest only queues *guard*; it does
   not start it, and it requests K's *idle* body state as a deferred change (S-0108 claim 6). No
   unloading processing and nothing in the release sets or clears K's ready mark (S-0215 claim 11).
   *Guard* therefore becomes current at the first of K's per-tick starts that finds the mark set,
   and is first processed on the tick after a second-start change (S-0215 claim 3b). For an
   unarmed factory without a turret, the mark comes from its body range (S-0215 claim 11, S-0108 claims 5
   and 6). Starts that find nothing queued do not clear it, so a mark set at any time since *unload*
   became current is still set at closing (a behaviour queued and started in between would itself
   have ended the unloading). *Unload* itself is queued at release without a start, so it too
   becomes current only at a per-tick start that finds the mark set:
   - When K's running body range has delay 0 (the absent-key default range has delay 0), the
     body update of every building turn sets the mark. *Guard* becomes current at the second
     per-tick start of the closing tick.
   - When the range has a positive delay and at least two stages, the mark is set on the turn the
     stage advances onto the range's last stage. If that has happened since *unload* became
     current, *guard* starts at the second start of the closing tick. Otherwise it starts at the
     first per-tick start of the tick whose body update next sets the mark; the body update
     comes before that start.
   - A positive-delay range of one stage or fewer never sets the mark, and a turreted factory's
     body range sets it only during construction or selling. For an unarmed factory with either
     condition and no other source of the mark, once it has used the mark left by its
     construction, a queued *unload* never becomes current: the released vehicle
     stays at its appear point in contact with K, and K stays in its resting behaviour. If *unload*
     did become current, the queued *guard* would never start and K would stay in *unload*
     (Edge cases, `quirks.factory_unload_waits_ready_mark`). An armed factory can instead
     obtain the mark from its resting-behaviour processing (S-0215 claim 32), so the
     never-starts prediction does not apply to it merely because its body range gives no mark.
   - Backing: `measurable` (U2 for stages and the *guard* start, U3 for delay assignments). The
     positive-delay and never-ending cases are `unmeasured`: no stock vehicle factory is
     established with such a range or a turret. The armed-factory qualification is
     `unmeasured` here: U2 does not exercise an armed factory; its ready-mark source is
     owned by S-0215 claim 32.
     The damaged production-animation variant is
     `unmeasured` here: U2 does not damage K or distinguish that variant.
   - Confidence: high (stages, queueing without a start, no ready-mark change while unloading),
     medium (the start tick, inherited from the ready-mark and body-range rules)

3. **Apron position.** For a foundation with at least eleven exit-list entries, take its eleventh
   pair (*a*, *b*) from S-0404 Constants. E is (*x*₀ + *a* − 1, *y*₀ + *b*). The track ends at
   E's horizontal centre; the vehicle's own height follows the movement rules, distinct
   from claim 5's stored movement-point heights. E need not lie inside the
   foundation; its position depends on the selected pair, not on a containment test.
   Shorter lists use the results in Edge cases. Apron selection uses the original lists under
   either state of S-0404's full-ring switch, as specified in Edge cases.
   - Backing: `measurable` (U4 for stock factory types). Other foundation words are `unmeasured`:
     no stock vehicle factory using those words is established for this scenario.
   - Confidence: high

4. **Blocked apron.** E is blocked when its ground layer lists an infantry unit, vehicle,
   aircraft or building other than K. The released u counts. Terrain objects and overlays do not.
   On a blocked processing every object in that layer receives a forced, move-anywhere scatter
   request without a threat point. Each apron neighbour that lists at least one such occupant
   other than K also sends a request to every object in its ground layer: forced, move-anywhere,
   with E's centre as threat point. Neighbours receive nothing when E is free.

   S-0209 claim 11 owns receipt by all listed objects; S-0207 claim 16 and S-0208 claims 11 to 13
   own acceptance and destinations. An accepting vehicle in E takes no scatter draw; an accepting
   vehicle in a neighbour takes one ranged draw over 0 … 2, and no acceptance draw over 1 … 4.
   Objects that cannot scatter simply ignore their request. A persistent blocker can hold the
   factory indefinitely. The observable assignment of random-stream values is as follows: when
   accepting occupants take scatter draws, values assigned to occupants in E precede those
   assigned to occupants in apron neighbours. Neighbour assignments follow step-direction order, and
   assignments within one cell follow S-0209 claim 13's listing order. The value assigned to the
   processing's delay follows those scatter values. Refused or ignored requests take no scatter
   draw under the owning movement rules. This constrains recorded directions, delays and stream
   state, not an internal traversal.
   - Backing: `measurable` (U5, U3 for assignments). Aircraft/building blockers and the
     terrain/overlay exclusions are `unmeasured` here: these scenarios do not place them on E.
     Same-cell recipient assignment is `unmeasured` unless U3 establishes multiple accepting
     occupants in one cell; a single recipient cannot distinguish the assignment order.
   - Confidence: high (tests and options); medium (destinations inherited from movement specs)

5. **Sending u out.** A drive vehicle is put on T1 towards E's centre, without a path search
   from its present position. Its stored heading point has integer world height 0 leptons
   throughout the exit track, not the vehicle's starting height or the terrain or deck height
   (S-0207 claim 24). A successful exit-track instruction also supplies the separate locomotor
   destination point at E's horizontal centre with integer world height 0 leptons
   (S-0207 claim 25). Neither point receives an extra deck lift; these integer assignments
   involve no rounding. The vehicle's own height while travelling remains surface-derived
   under S-0207 claim 5, including its deck rule, rather than being forced to 0.
   Tunnelling and teleporting vehicles temporarily receive driving
   locomotion and use the same track; restoration belongs to S-0215 claim 13, with
   temporary-driving eligibility owned by S-0207 claim 27. Restoration requires a suspended
   predecessor, no driving movement report, no restoration lock and a vehicle that is not
   seized. The driving report is false only when there is no locomotor destination point
   and no heading point horizontally different from u's actual position. Completing T1
   alone is therefore insufficient: a retained locomotor destination point still prevents
   restoration. Claim 16 specifies the subsequent replacement and clearing events.
   This is not a promise of an automatic restoration at the apron. Restoration alone enables neither the destination-refresh state nor the
   one-application point suppression of S-0207 claim 27. S-0207 claim 21
   owns the suspended teleporting state and its occupancy consequences: destination
   assignments completed while driving use driving movement, not the suspended teleporting
   movement. A qualified docking assignment that itself restores the predecessor instead
   supplies its point to the restored locomotion (S-0207 claim 27). Restoration
   alone neither resets that state nor moves an occupancy mark. Those rules inherit that claim's
   backing and confidence, including its unmeasured pre-existing pending-jump and phasing-in
   cases. If that temporary replacement cannot be established, successful exit is not promised
   (Open question 4).
   Any other locomotor instead gets destination (*x*₀ + 4, *y*₀ + 1), independent of the
   foundation and its exit list. It moves by its own locomotor's rules. K sets u's speed fraction
   to 0.5 after this exit instruction in all these cases.
   A resting, powered, non-paralysed hovering vehicle attempts its first path search and
   heading-point choice during the exit instruction itself (S-0503 claim 5), before K sets
   ρ = ½. A successful clear choice already supplies a heading point and produces claim 7's
   cell-arrival update at u's pre-travel position (S-0503 claim 15). A refused choice or
   unsuccessful search need not supply a point; there is no mandatory next-tick delay
   before the first choice.
   For a hovering vehicle, ρ = ½ can persist during the exit move, but is not guaranteed
   for the whole leg. S-0503 claim 4 owns its subsequent fraction changes: in particular,
   a stalled start with provisional step length 0 leptons and movement heading exactly
   towards its heading point resets ρ to 1. This is a fraction reset, not an immediate
   rise to full hover speed.
   - Backing: `measurable` (U6 for driving, teleporting and hovering). Tunnelling is
     `unmeasured` unless U6 establishes a stock factory-produced type; no such setup is
     specified here. Failed temporary replacement is `unmeasured`: no stock setup is
     established that makes it fail. Hover fraction persistence and the stalled-start
     reset are `measurable` (U6 and S-0503 H8), with backing inherited from S-0503 claim 4.
     Stored driving heading and locomotor destination heights inherit S-0207 claims 24
     and 25's `measurable` backing (D28, also U6).
     Temporary-driving restoration eligibility is `measurable` in U6; a restoration lock
     or seizure during that exit remains `unmeasured` here unless an ordinary stock
     sequence establishes it. The no-pending-state consequence inherits S-0207 claim 27's
     `unmeasured` backing for isolated restoration: U6 proposes a conditional trace.
     Immediate heading-point choice and its arrival update inherit the `measurable`
     backing of S-0503 claims 5 and 15 (H8).
   - Confidence: high (driving, stored movement-point heights and fixed destination),
     medium (temporary replacement), high (temporary-driving restoration predicates
     and no pending state enabled by restoration, inherited from S-0207 claim 27),
     medium (immediate hover choice, inherited from S-0503 claim 5),
     high (clear-choice arrival update, inherited from S-0503 claim 15),
     low (duration of the hovering exit fraction, inherited from S-0503 claim 4)

6. **Exit track and speed.** T1 gives every point and facing. Passing its first point places u
   at E's centre plus (−670, −68) leptons regardless of the appear point. Between-point movement,
   progress cost and completion follow S-0207 claims 3 and 4: a point costs 7 progress units.
   A tick need not have enough progress to pass even the first point, so no universal first-tick
   jump is promised.

   The exit-track instruction sets ρₜ to 1.0, after which K sets ρ to 0.5. A type marked to
   accelerate does not receive ordinary acceleration on this special track, so retains 0.5
   absent another effect. A non-accelerating type copies ρₜ into ρ on its movement update and
   therefore uses 1.0. Speed and integer progress earned are S-0207's, not geometric distance
   divided by the unit's imported speed.
   - Backing: `measurable` (U7 and U6)
   - Confidence: high (track and speed tests); medium (first-point observable consequences)

7. **Clear report on vehicle cell arrival.** A jumpjet's ascent uses claim 15 instead.
   Its report does not have this claim's arrival or building-cell tests.
   On a cell-arrival update (defined below) while tethered, a vehicle reports clear unless it
   is in *unload*, or in *enter* with its contacted building as destination, or both these hold:
   it has a destination other than its present cell, and its cell holds a building. A destination
   object other than a cell is not treated as its own cell for this test. The vehicle-factory
   contact also suppresses a report on that factory's cell with a different destination; the
   building test already covers that case. These tests read the order-level destination,
   not a hovering locomotor's possibly retained movement destination (S-0503 claim 16).
   An accepted person's stop breaks contact under S-0213 claim 8, so a later hover
   arrival update must not be treated as a report through an unchanged factory contact.
   That order is refused while tethered; it is not a way to stop a normal tethered exit.

   With no destination, or with its own cell as destination, u can report even on K's foundation.
   Thus a normal exit track can report at E itself, without a separate move off the foundation.
   The report ends contact and the tether, enabling K's closing stage.

   **Cell-arrival updates of a driving vehicle (Q-0223).** For a vehicle moved by driving
   locomotion, including the temporary driving of a teleporting or tunnelling vehicle (claim 5),
   exactly two events are cell-arrival updates for this claim and for claim 8's fallback:
   - reaching a track target, the cell centre at the end of a track (S-0207), also when the move
     goes on with a further track in the same tick, and
   - reaching a chain point and switching to the next curve, for a vehicle that chains curves
     (S-0207 claim 8). This update falls before the cell centre.
   Crossing a cell boundary in the middle of a track is not a cell-arrival update, and neither is
   turning in place. The exit track of T1 has no chain point, and only its last point is its
   target, so on it the report can happen only at E's centre, never in an earlier cell.

   **Cell-arrival updates of a hovering vehicle.** S-0503 claim 15 owns these events:
   a clear heading-point choice produces an update at u's actual, pre-travel position,
   including the immediate first choice of S-0503 claim 5. Its arrival outcome table,
   rather than search success alone, decides updates without a new heading point:
   - A non-refusing choice with no point restores the actual-position occupancy mark
     and produces an update. This includes a path update deferred by the retry delay, a search that
     fails during choice, and no movement destination with contact in the present cell
     when the vehicle is not held.
   - A search that fails in the initial arrival path update and clears the movement
     destination and heading point produces no update without same-cell contact.
     With that contact, claim 15's no-movement-destination contact row applies instead.
   - No movement destination and no same-cell contact gives no choice or update.
     A refused choice, including a held vehicle, gives no update from that choice.

   These are S-0503 claim 15's movement-destination tests, distinct from this claim's
   order-level report tests. Normal completion in the contacted factory's cell can
   therefore update without a new heading point, but arrival alone, destination clearing
   alone and cell-edge crossings do not guarantee an update. The report tests above
   apply at the actual pre-travel position, not at the chosen heading point or a position
   reached by a final already-computed travel step. Thus an immediate clear choice on a
   building cell with a different order-level destination still suppresses the report.
   Choosing a point does not itself end the tether.
   - Backing: `measurable` (U2, U8 for normal exits and the report at E). The *enter* and
     *unload* exceptions and the different-destination building-cell suppression are
     `unmeasured` here: no release setup retaining those conditions on an arrival
     update is established. The chain-point update is `unmeasured` here: no scenario part in
     this spec produces a chained curve while tethered, and a factory exit has no chain point.
     Hover update events inherit S-0503 claim 15's `measurable` backing (H8 and H15).
     The destination distinction and accepted-stop contact break inherit S-0503 claim 16
     (H12 and H15) and S-0213 claim 8's backing, including its unmeasured contact-break
     qualification; they are not an established tethered-exit stop setup.
   - Confidence: high (tests, the two update events, the report only at E on the exit track),
     medium (normal arrival consequence). Hover update confidence is inherited from
     S-0503 claim 15: high for clear choices, medium for exceptional completion cases.
     The destination distinction and stop consequences inherit S-0503 claim 16 and
     S-0213 claim 8's confidence.

8. **Run-away destination.** A vehicle factory gives the run-away reply to the clear report.
   These are precedence rules, not a required internal implementation sequence:
   - A destination other than u's current cell is kept instead of the rows below. An additional
     interaction with a destination unit or building remains open (Open question 7).
   - Otherwise an ore-gathering vehicle queues *harvest* with a readiness-dependent immediate
     start (S-0215's queueing terms).
   - Otherwise a human-controlled unit whose saved guard spot exists and differs from its
     destination takes that spot as destination, without queueing a new *move*. The spot was
     copied from K's rally point at release, not re-read from K at the clear report.
   - Otherwise it discards its stored path and receives a forced scatter request without a
     threat point, **not move-anywhere**. An accepting vehicle without a destination uses the
     nearby-cell search without a random draw. This row also applies when its saved guard spot
     already equals its destination, not just when no rally point was saved.

   In the original, a team member is handled by team logic; otherwise a computer-controlled unit
   leaving a vehicle factory can be sent towards its house's base cell followed by *area guard*.
   Base-cell selection is not a zero-draw rule. In our engine all player houses use the human rows
   and AI chooses its own orders (lead policy). Neutral and special houses retain the original
   non-human rule; this spec does not establish a produced-unit setup for them.

   A tethered vehicle's arrival update also requests a forced, move-anywhere scatter without a
   threat point if, after report handling, it remains on a building cell with no destination,
   no queued destinations and no stored path. This is a fallback, not a mandatory preliminary
   move before the rally point. If the report at E supplies a rally destination, u heads there
   directly. Without a rally, the run-away scatter normally already supplies a destination:
   there is no mandatory second scatter.
   - Backing: `measurable` (U8 for normal report, rally, harvest and no-rally outcomes).
     Keeping a different destination, an already-equal saved spot, and the remaining-on-building
     fallback are `unmeasured` here: no U8 part establishes those conditions at the report.
     The additional destination-object interaction, the neutral-house production case and the
     exact computer-only base-cell choice are `unmeasured`: no precise
     stock setup or complete expected base-cell selection is established here.
   - Confidence: high (human rows, options and saved rally); medium (normal exit consequences)

9. **Naval factory.** A released ship has no new contact with K and keeps the rally move from
   S-0404 claim 13. When K starts unloading, its first processing opens the door and starts the
   production animation, but advances directly to closing. Its second processing falls back to
   rest regardless of door state, stopping the production animation and queueing *guard*.
   There are exactly two unloading processings and two delay draws, no apron test and no exit
   movement commanded by those processings. The door can still be moving after unloading ends.
   The queued *guard* starts by claim 2's rule (Q-0222); with the ready mark already set, that is
   the second per-tick start of the tick of the second processing.
   - Backing: `measurable` (U9)
   - Confidence: high

10. **Infantry factory and cloning building.** Placement, facing and cloning are S-0404 claims 13
    and 14. K has no unloading stages or door movement. Before placement, the released unit saves
    K's rally point as its guard spot, or saves no spot if K has no rally point. Placement immediately
    makes the unit fall back to rest, so S-0215 claim 13's guard-spot row applies at that moment. A
    freshly made unit has no current behaviour; consequently, if its rally point is not its own cell,
    it takes the rally point as its destination, queues *move*, and clears its guard spot. After that
    placement consequence, if the unit has a destination and is neither jumpjet infantry nor marked
    as teleporting, its guard spot becomes that destination, *move* is queued, and its destination
    becomes the exit cell (S-0404 claim 13). K then offers contact. If the unit accepts, K gives it
    clearance to unload and tethers it. Contact and clearance take no draw.

    Thus, with a rally point outside its own cell, an ordinary infantry unit walks first to the
    exit cell and then, when *move* falls back to rest there, to the rally point by the guard-spot
    row of S-0215 claim 13. A jumpjet or teleporting infantry type heads straight for the rally
    point with an empty guard spot. A tethered ordinary walking infantry unit reports clear
    on its first standing-position arrival after tether establishment (S-0208 claim 23),
    not merely on crossing a cell boundary. An intermediate arrival while continuing towards
    the exit cell counts, including an arrival still on K's foundation: neither reaching the
    exit cell nor leaving the foundation is required. Claim 7's vehicle building-cell
    suppression does not apply. An ordinary infantry factory does not give the run-away reply:
    contact and tether simply end, without the report itself finishing or redirecting the
    exit walk. S-0208 claims 16 and 17 own walking completion; claim 23 excludes an entry
    interaction that removes the unit before it can report. The jumpjet timing remains separate.

    **Teleporting infantry clear report (Q-0697).** A released, tethered infantry unit moving
    by its own teleporting locomotor reports clear on its first jump's cell-arrival update
    after tether establishment. S-0207 claim 22 supplies that update in the jump tick, after
    landing position, height and landing-layer listing change. Contact and tether end in that
    same update, before the landing cell's entered-by tag is raised. Thus tag actions observe
    the factory contact place already free. Neither phasing-in completion nor *move* falling
    back to rest is required. The report has no foundation-exit or different-destination test:
    claim 7's vehicle suppression does not apply. An ordinary infantry factory gives no
    run-away reply and adds no scatter or exit-cell leg to this jump. This covers an ordinary
    factory release to a clear rally cell, without an entry interaction removing the unit.

    Landing-point reservation alone does not report clear. If the unit never jumps and has no
    other cell-arrival update or independent contact-breaking event, it retains both contact
    and tether as a resting walker does. Without a rally point, claim 17 normally sends it a request
    in its first turn in which its resting behaviour is current, and it then jumps (claim 17,
    rated medium there).
    So this retention needs a request that is refused, that gets no destination (after S-0208
    claim 13's fall-back to the neighbours), or that gives a destination to which it never
    jumps. A jump needs no path (S-0208 claim 19), so the reachability gate that stops a walking
    unit (S-0208 claim 24) gives a teleporting unit no retention. U10 e tests the ordinary
    no-rally jump; retention without an arrival report is otherwise `unmeasured`.
    There is no factory-release contact timeout. Clearing
    a destination without jumping, resting in *guard*, and falling back to rest do not themselves
    supply the missing arrival report. With every contact place held, a later release is busy
    under claim 14 and S-0404 claim 13, not successfully released with a permanently stale link.
    - Backing: `measurable` (U10 d for jump-time release and ordering, U10 e for the jump
      that claim 17 causes without a rally point). The retention of a teleporting unit that
      never jumps is `unmeasured`: claim 17 moves a resting unit on the foundation, and no
      stock setup is known that keeps it from jumping without an order
    - Confidence: high (arrival/report connection, contact and tether loss, absence of vehicle
      suppression); medium (combined factory-release consequences); low (the no-jump
      retention, `unmeasured`)

    **Release of jumpjet infantry.** Claim 15 owns its ascent clear report, which ends both factory
    contact and tether during the climb, without waiting for a walking arrival or touchdown.

    With no rally point (or a rally point in the unit's own cell), neither placement nor the release
    gives a destination or queues *move*. A walking or teleporting unit rests in *guard* (or
    *area guard*) where the release puts it. When that cell holds K (claim 17's cell test), claim
    17 sends it off that cell when its request gets a destination (for jumpjet infantry, Open
    question 3); a cell of K's foundation rectangle that K's foundation word leaves out (S-0406
    claim 2) does not hold K. In its first turn in which its resting behaviour is current on a
    cell that holds K it is sent a
    forced, move-anywhere scatter request with no threat point (claim 17) and, when it accepts and a
    destination is found, moves to that destination (a teleporting unit jumps, claim 17). Its first standing-position arrival after tether establishment
    is its clear report, as above, which ends contact and tether and frees K's contact place.
    The no-arrival contact rule still holds: while it remains resting without a cell-arrival
    update or an independent contact-breaking event (for example while each request is refused,
    gets no destination after S-0208 claim 13's fall-back to the neighbours, or, for a walking
    unit, gives a destination for which no path is found, S-0208 claim 24 and S-0212 claim 6's
    reachability gate), it keeps contact and tether and holds one of K's contact places
    (claim 14). There is no factory-release contact timeout. For a jumpjet, claim 15's ascent
    report is such an independent event: no-rally release must not be read as retention after
    it has flown away. With a single contact place, a later infantry release is busy (S-0404
    claim 13) only while that place is held, which after claim 17 is normally the unit's short
    walk to its first arrival; so later units of the factory come out (claim 17, *Unit after
    unit*).

    Setting K's rally point after the release does not move a unit already released: its guard
    spot, if it has one, was saved at its release, and no rule gives a resting unit its factory's later rally
    point. This negative is proposed for measurement (U10 b) and has not been run.

    The original's computer-controlled release also queues *area guard* and can choose a
    base-cell guard spot. Our player houses use the human rule; neutral and special houses retain
    the original owner rule.
    - Backing: `measurable` (U10 a and S-0208 W15 for ordinary release, first-arrival
      clear-report timing and the two-leg walk; U10 b for the
      absence of an immediate no-rally destination, the walk off the foundation, its contact end
      and a rally point set afterwards; U10 c for jumpjet release; U10 d/e for teleporting
      release). No-arrival contact retention is `unmeasured`: claim 17 moves a resting unit off
      the foundation, and no stock setup is known that keeps it there. Cloning contact hand-over,
      a rally in the appear cell and zero-draw contact
      exchanges are `unmeasured` here: these parts do not exercise those releases or isolate
      their contact draws. Non-player-house production is `unmeasured`: no stock factory-release
      setup for it is established.
    - Confidence: high (release destinations, contact hand-over and first-arrival report test),
      medium (the two-leg walk and intermediate/foundation arrival consequences, the no-rally
      walk off the foundation as composed with claim 17, and a rally point set afterwards,
      until U10 b is run), low (no-arrival contact retention, `unmeasured` here)

11. **Aircraft factory.** It does not enter these factory unloading stages. Edge-entry orders and
    where the aircraft appears belong to S-0404 claim 13. When the aircraft appears docked at K,
    K offers contact and then sends a tether request on the same tick:
    - a type that need not return to a dock accepts and is tethered to K from the tick of its
      release, and
    - a type that must return to a dock refuses the request, so the release does not tether it.
      Contact is established in both cases.

    **Destination and pad contact.** An aircraft standing on a pad ends its contact with that
    pad when it takes a destination other than the pad (S-0501 claim 8). Contact release can
    therefore precede physical lift-off; it is not conditional on leaving the foundation.
    **Take-off hypothesis:** a release-tethered aircraft has lost its tether by the time it
    leaves the ground. Destination-triggered contact release supports this hypothesis but
    does not establish the exact tether-loss tick. U12 distinguishes the two events; the
    glossary's provisional pointer is not evidence for either timing. Where the docked aircraft
    appears is S-0404 claim 15 (pad positions). The
    destination of an aircraft entering from the map edge without a rally point is claim 14.
    - Backing: `measurable` (U1 for no unloading; U12 for release tethering,
      destination-triggered pad-contact release and tether loss by take-off). A branch with no stock docked-release setup available is `unmeasured` for that
      reason; an edge-entry release does not test docked tethering.
    - Confidence: high (no unloading, destination-triggered pad-contact release, inherited
      from S-0501 claim 8); medium (tether at release, both branches); low (tether lost by
      take-off hypothesis)

12. **Random-stream consequences.** Each vehicle-factory unloading processing assigns exactly
    one ranged delay draw over 0 … 2. In a blocked processing, the random-stream values assigned
    to accepting scatter recipients precede the value assigned to that processing's delay,
    with recipient order as stated in claim 4. The recorder can therefore identify which stream
    value determined each recipient's direction and which determined the delay. Opening and closing the door take no draw of their own. Sending out on an ordinary
    drive track, reporting clear and taking the human rally destination take no additional draw
    of their own. An accepting vehicle's no-threat scatter takes none.

    This is not an assertion that every processing tick consumes only one value. Production
    animation creation, start events and replacement effects have S-0202 claim 10 and S-0109's
    draws, assigned before that processing's delay. Failed temporary locomotion may invoke damage
    and death effects; those are not covered by the zero-draw statements. Other actors' processing
    and movement draws, initial placement, cloning creation and the original's computer-only
    base-cell choice are not unloaded-unit delay draws either.
    Idle apron clearing (claim 13) assigns its scatter draws, in claim 4's order, before that
    *guard* processing's own delay draw. Turning the guard spot into a destination at placement
    (claim 10), offering contact and the tether request to a docked aircraft take no draw. The
    edge-entering aircraft's nearby destination of claim 14 takes no draw (S-0212 claim 25).
    Whether putting a released unit on the map takes draws of its own is not established (Open
    question 8). A unit released with no rally point takes the draw of the scatter request of claim
    17 in the turn in which it accepts it (S-0208 claim 13), normally its first turn, in the next
    tick (claim 17 gives the exceptions). That draw belongs to that turn of the unit, not to its
    release.
    - Backing: `measurable` (U3, U11 for delay/scatter assignment and U3's isolated ground
      exit events; U10 b for the first-turn scatter draw of a unit released with no rally
      point). Placement guard-spot changes, aircraft contact/tether exchanges and the
      edge-entry destination's no-draw exclusions are `unmeasured` here: the release scenarios
      do not separate their effects from untraced placement draws. Failed replacement's effects
      are `unmeasured`: no stock setup causing that failure is established.
    - Confidence: high (explicit delay and scatter assignments), except the scatter draw of a
      unit released with no rally point, medium as claim 17 and S-0208 claim 13 rate it; medium
      (downstream effects)

13. **Idle vehicle factory clears its apron.** This applies to a building whose type has the vehicle
    factory flag, naval or not, that is unarmed and lacks the stupid-guard-mode flag (S-0215
    claim 32), while its current behaviour is *guard*, or *sticky* or *area guard*, which S-0215
    claim 32 processes by the same rule.
    - The first processing after such a behaviour starts returns K's animation to idle and makes
      no apron test. This holds for every start, including the *guard* that ends unloading.
    - Every later processing of that behaviour applies claim 4's blocked test to E. When E is
      blocked, the observable requests are those of claim 4: occupants of E and occupied apron
      neighbours receive the specified requests. Their observable random-stream assignments
      follow claim 4. The result has no other effect, and the behaviour does not change.
    - Each processing chooses S-0215 claim 32's delay 3 × *D*r + *U*(0, 2), with *D*r F1 applied
      to the rate of the current behaviour, unless that claim's repair-bay rule has already ended
      the processing.

    E is claim 3's apron for K's own foundation, including the short-list results (Edge cases).
    A naval factory therefore clears an apron while idle, although it never tests one while
    unloading (claim 9). Thus a unit left on E of an idle vehicle factory is asked to scatter at
    about every 3 × *D*r ticks. A refusing occupant changes nothing, because the factory is idle.
    No apron test happens while K is armed, has the stupid-guard-mode flag, or is in any
    behaviour other than these three or *unload*.

    **Refinery boundary (Q-0371).** This is vehicle-factory apron clearing, not general
    eviction from a refinery's docking cell. A refinery uses this rule only if it also
    satisfies the vehicle-factory conditions above, and the tested cell is E, not its
    docking cell as such. S-0402 claim 10a owns the blocked docking exchange: it does not
    request an unrelated resting harvester on the docking cell to scatter.
    - Backing: `measurable` (U11 for unarmed guard clearing and delay; S-0402 H20 for
      the blocked docking exchange, with confidence inherited from S-0402 claim 10a). The armed and
      stupid-guard-mode exclusions, sticky and area-guard equivalence, and repair-bay early end
      are `unmeasured` here: U11 supplies no stock setup exercising those branches.
    - Confidence: high (tests, assignments, delay); medium (naval case)

14. **Release details left to this spec (Q-0120).** S-0404 claim 13's release rows use three
    rules that it leaves here:
    - **Contact places of a factory.** A factory building has *n* = max(*d*, 1) contact places,
      where *d* is its type's imported docking-place count (S-0404 Import mapping, including its
      absent default). A place is free while no unit is in contact through it. An infantry
      factory's release is busy when none of its *n* places is free. Without a rally point, claim
      17 normally ends the earlier unit's contact at its first arrival off the walk it starts in
      its first turn, so a later release is busy only if it comes before that arrival. One line
      finishes its items at least 54 ticks apart (S-0404 claim 7, S-0403 F7), so that needs a
      first arrival at least 54 ticks after the release. This consequence of claim 17 has
      claim 17's rating (medium), not this claim's.
    - **Entry test of an infantry exit candidate.** A candidate cell that is a legal map cell
      passes when the unit's own cell-entry test for that cell (S-0209 and S-0211, made with
      no particular facing or level) gives the free result. Any other result rejects the
      candidate, including the results that only make a moving unit wait.
    - **Nearby destination of an edge-entering aircraft.** With no rally point, the aircraft's
      destination is the result of the nearby-cell search (S-0212 claim 25) around the cell of
      K's reference point. The search uses the aircraft type's locomotion class, with *winged*
      counted as *track*, and its own movement zone. Its reach region is the centre cell's for
      that zone, read for its deck in deck mode. It runs in deck mode only when the aircraft
      is on a bridge deck. It has no height condition, further test or reference cell,
      as in S-0212's idle harvester step-off
      row. The tick number used to select among preferred cells is the release tick.
      When the search finds a cell, the aircraft's *move* is forced with that cell as
      destination. When it finds none, *move* is forced with no destination. With a rally point,
      *move* is forced with the rally point as destination. The search takes no draw.
    - Backing: `measurable` (U13 a for a single held contact place, with its slow test type,
      U13 b for the entry test,
      U13 c for a successful aircraft nearby destination). Counts other than one, the
      no-result search, deck-mode search and forced-move distinction are `unmeasured` here:
      no scenario part establishes those setups or distinguishes a forced move from an ordinary
      move. Their owning general rules remain as cited.
    - Confidence: high (contact-place count, forced *move* and destinations); medium (the entry
      test's exact results, inherited from the cell-entry rules, and the busy release before the
      first arrival, which rests on claim 17)

15. **Ascent of a jumpjet releases the factory (Q-1070).** A released infantry unit or vehicle
    currently using jumpjet locomotion, whose type is marked as jumpjet or balloon-hover,
    gives a clear report on its first qualifying *ascending* update while still in contact
    with a vehicle factory or a building with one of the barracks door kinds. Both factory
    contact and the tether end during that same unit update. This is an ascent report,
    not a cell-arrival report: claim 7's different-destination and building-cell suppression
    does not apply. No distance from K, foundation departure, cell crossing, first cruise,
    destination arrival or touchdown is required.

    **Exact tick.** Use the integer bridge-adjusted height *h* and integer target height
    *h*ₜ of S-0502 claim 7, evaluated after this tick's jumpjet physics (S-0502 claim 3).
    The qualifying window is trunc(*h*ₜ / *q*) < *h* < *h*ₜ, in leptons, where *q* = 2
    for a balloon-hover vehicle and *q* = 4 otherwise. Division is signed integer division
    truncated toward zero, not floating point. Balloon-hover infantry still uses *q* = 4.
    Equality at either boundary does not qualify. The test applies on every qualifying
    ascending update, not only on a detected threshold crossing; after a successful report
    there is no factory contact left to report again.

    **Skipped-window safety departure (design).** A released unit satisfying the same
    type and factory-contact conditions whose ascent reaches or passes *h*ₜ without a
    qualifying report gives the clear report in that reaching update. Both links end
    and the destination effects below apply in that same update. This fixes a permanent
    factory block, with no quirk switch. The original's strict-window report is skipped
    by such a rise; no alternative original release event is established.

    A ground vehicle factory in the waiting stage of table S1 can start closing on its
    first subsequent unloading processing that observes the ended link, in the same tick
    or later according to actor update order and its processing delay. It need not wait
    for any part of the jumpjet's journey after this report. An infantry factory has no
    unloading stage: the contacted place becomes free in the report tick itself, so a later
    release is not busy on account of that unit's old contact (claim 14).

    **Rally and no rally.** The report condition itself does not test for a rally point.
    A ground vehicle receives claim 5's fixed exit destination whether or not K has a
    rally point, so its ordinary successful exit starts ascent in both cases. Infantry
    with a rally point outside its own cell starts towards that rally under claim 10.
    Infantry without such a point receives no release destination; if it later starts
    ascending, the same report applies. Whether claim 17's request reaches a resting no-rally
    jumpjet infantry unit, and so starts its movement, remains Open question 3. It is not a
    prediction that a unit already flying retains its factory forever.
    A newly commanded legal move can supply
    the ascent; a refused tethered stop is neither required nor a remedy.

    **Destination after the ascent report.** This report does not use claim 8's ordinary
    vehicle-arrival run-away handling. If u has a saved guard spot different from its
    order-level destination, it takes that spot as destination. Otherwise its
    order-level destination is discarded without directly stopping its jumpjet
    locomotion, and it receives a forced scatter request without a threat point,
    not move-anywhere. Scatter acceptance and any resulting destination remain
    S-0207 claim 16's rules for vehicles and S-0208 claims 11 to 13's rules for infantry.
    Discarding the order-level destination here is not S-0502 claim 24's effective destination
    clear and does not itself reroute or erase the locomotor's movement point and
    commitment; an accepted scatter can supply a replacement destination.
    Neither a refused scatter nor continued flight recreates factory contact or tether.
    Thus with a saved rally, a vehicle normally takes it at this report instead of
    finishing the fixed exit leg; infantry whose placement already consumed its guard spot
    has no saved spot to reapply.
    - Backing: `measurable` (U14 for ordinary stock vehicle and infantry release, ascent
      tick, both links, destination effects and subsequent production). Balloon-hover
      infantry, non-balloon jumpjet vehicles, missing type flags, the original aftermath
      of a one-step ascent that skips the window and a resting no-rally jumpjet infantry
      unit's take-off are
      `unmeasured` here: no stock factory-release fixture exercising those cases is
      established. The report rule is established conditional on ascent, not on a measured
      automatic no-rally take-off. U14 records that remaining limitation explicitly.
      The skipped-window fallback is a design rule, not an observation of the original.
      U14's engine-only variant tests it separately.
    - Confidence: high (report, same-update loss of both links, height and type tests,
      no foundation test and destination effects; decided safety fallback); medium
      (combined normal-release consequences and the first eligible factory processing).

16. **Lifetime of temporary driving after a factory exit (Q-1110 question 3).**
    Finishing T1 with no order-level destination does not clear the exit locomotor
    destination point. Neither the clear report, loss of contact and tether, nor
    queueing *harvest* is a stop or a restoration event. There is no separate
    factory-exit timeout that clears this point. The later movement and restoration
    consequences are as follows, subject to the ordinary order-acceptance rules:

    | Event after the exit | Movement and restoration consequence |
    |---|---|
    | Claim 8 supplies a rally or an accepted scatter supplies a destination | Its movement point replaces the exit point in the active driving locomotion. It is not a teleporting leg |
    | A factory-built chrono miner starts seeking and finds ore in a different cell | The ore destination replaces the exit point. The miner drives its first leg to that ore with its temporary driving locomotion, retaining the suspended teleporting locomotion |
    | A later driving track completes at its order-level goal with the height condition of S-0207 claim 25 met | The replacement locomotor destination point clears by that claim, before the cell-arrival report. This is a later goal's completion, not retroactive clearing at the apron |
    | The ordinary cell-goal cleanup, an effective destination removal that stops locomotion, or another accepted stop applies | The driving destination point clears under S-0207 claims 25 and 26 |
    | A subsequent fall-back to rest finds all claim 5 restoration predicates satisfied | The suspended locomotion is restored under S-0215 claim 13. Restoration is not implied merely by arriving at the first ore cell or starting loading. For a chrono miner, its next destination that is not a qualified docking assignment installs temporary driving again (S-0207 claim 28) |
    | A qualified docking assignment occurs (S-0207 claim 28 lists those of ordinary play) | S-0207 claim 27 owns restoration within that assignment or its fallback stop and refreshed reassignment; this is a separate route to restoration, not a required prior fall-back to rest |

    The seeking exception that drops an initial destination for teleporting movement
    (S-0402 claim 17) tests the active locomotion. It does not drop that destination
    for a miner still using temporary driving, even though the type is teleport-capable.
    The run-away reply's *harvest* start therefore does not restore the miner before
    its first field leg. A refinery's free unit is a different release path
    (S-0402 claim 18), not evidence that a vehicle-factory exit skips this driving leg.

    A later restoration does not end a factory-built chrono miner's driving trips. Every
    destination it later takes that is not a qualified docking assignment installs temporary
    driving again, outside S-0207 claim 28's three exclusions (warp-pending immobilization,
    being gripped, seizure), so after a restoration it teleports only on a qualified
    refinery docking assignment: the refinery's docking-cell offer, the repeated dock test
    of that cell in *enter*, or, at a sale of that refinery while the contact still holds,
    the sale's scatter request and, for a miner that was unloading there and is not full, the
    cells its first seeking processing sets (S-0207 claim 28, *A sale of the refinery*). Its
    other trips to ore, to a queueing cell and to a player's destinations are driven.

    **No follow-up goal.** There is no unconditional guarantee that every conceivable
    exit is followed by restoration. If no later instruction replaces or stops the
    retained point, it continues to prevent restoration, indefinitely while that
    condition persists. An unchanged empty-to-empty destination offer without refresh
    is rejected under S-0207 claim 27 and is not such a stop. Merely attempting to fall
    back to rest does not bypass claim 5's movement-report test. This is the original
    conditional retention, not an instruction to keep a normal miner permanently
    driving after its later stopping or qualified docking events. Claim 8 normally
    supplies a follow-up destination or queues harvest; the latter can remain without
    a field destination when no field is found. No ordinary stock factory-produced
    non-harvesting teleporting vehicle is established here, so the non-harvester rows
    are conditional rules, not a claimed stock release fixture.
    - Backing: `measurable` (U15 for the first field leg, replacement, clearing and
      subsequent restoration). The chrono-miner consequences, and the expectations of U6 and
      U15, are composed with S-0207 claim 28. Isolated retention with every subsequent stop and
      destination replacement excluded is `unmeasured` until U15's no-field control
      establishes that interval; no finite observation proves perpetual retention.
      No stock non-harvester teleporting release fixture is established for that branch.
    - Confidence: high (point lifetime, active-locomotion test, harvest queue not a
      restoration event); medium (composed first-field-leg and subsequent restoration
      consequences).

17. **Resting infantry on a building cell is sent off it (Q-1757).** An infantry unit is sent a
    scatter request in each of its turns (S-0203 claim 2) in which all of these hold:
    - it is in play on the map,
    - its current behaviour is *guard* or *area guard*,
    - the cell that holds its position holds a building that counts here.

    The request has no threat point, is forced and is move-anywhere, unlike claim 8's request and
    S-0215 claim 28's, which are not move-anywhere. It is part of the unit's own turn, not of its
    behaviour's processing. It comes in every turn of
    the unit in which these conditions hold, whatever the behaviour's processing delay,
    including the turn in which a queued resting behaviour becomes current (S-0215 claim 11).
    So the first request comes in the unit's first turn in which the resting behaviour is
    current on such a cell, on that turn's own tick and not one tick later, and while the conditions hold a request comes in
    every turn of the unit, each taking its draw when it is accepted. In our engine the request
    comes after the rest of the unit's turn: after its queued resting behaviour has become
    current, after any processing of its behaviour in that turn (with that processing's draw)
    and after its movement in that turn. So it tests the behaviour current at the end of the
    turn, also in the turn of a late start (S-0215 claim 11), and its draw follows the
    processing's draw (design); the original's order within the turn is `unmeasured` (Open
    question 11).
    Our engine does not send the request in a turn of the unit in which the unit is being erased
    (S-0514 claim 17), which includes a superweapon warp's leaving stage, since a unit in that
    stage counts as being erased (glossary *leaving stage*), nor in a tick in which the unit
    travels through a tunnel. This is a design rule, decided at the delegated sign-off,
    2026-10-25: the request comes after the rest of the unit's turn, and in those ticks the unit
    does none of its ordinary work (S-0513 claim 48, S-0514 claim 17 and S-0215 claim 2 leave
    such a unit out of its behaviour processing, movement and firing), so a turn that ends
    early makes no request. Whether the original sends the request in those ticks depends on
    where it falls within the turn, which is not established (Open question 11). At a warp's earlier
    update boundaries (S-0513 claim 51) our engine does not send the request (design); whether the
    original sends it there is open (Open question 12). The cell is the one under the unit's
    position on the ground plane, whatever the unit's height. A building does not count when it
    fits one of the following occupant rows of S-0209 claim 4 (part 4c), whose value is *none*:
    this test borrows those rows to decide whether a standing building counts.
    They are:
    - a building of a type that is invisible in play (row WP7),
    - a laser-fence section in either of row WP8's two states. Which states those are in the
      original is not established (S-0209; Open question 14). Our engine takes them to be the
      states in which the fence is down and shows no beam (design),
    - a firestorm wall whose owner's firestorm is off (row WP10),
    - an open gate (row WP11).

    An enslaved unit is not sent the request when the building is its master and the cell is
    the master's unloading cell (row WP2).

    The unit judges the request by S-0208 claim 11 and, when it accepts, finds its destination
    by S-0208 claim 13 and its single draw. A unit with no movement in progress accepts it unless
    claim 11's first case refuses it (a sequence that cannot be interrupted), since the request
    is forced, and since it is also move-anywhere the second case does not refuse it either. A
    walking unit that accepts and gets a destination walks there as under a move order, within
    S-0208 claim 24's limits on a move order. A teleporting unit jumps there (below), which
    needs no path (S-0208 claim 19). Whether a further request is sent depends on the conditions
    above. A refused request, or an accepted one that gets no destination, leaves the unit
    resting on the building cell. If the conditions
    above still hold in its next turn, the request is sent again; each accepted request takes its
    draw.

    **No path (Q-1980).** A walking unit whose destination gets no path also stays on the
    building cell. What a later request finds depends on why it has no path:
    - *Waiting for the retry delay.* A destination different from the one it holds ends a running
      retry delay (S-0212 claim 21), so a walking unit given a different destination never holds it
      unsearched because the delay runs. Whether giving it the destination it already holds ends the
      delay is left open, as S-0212 leaves it (its Open questions). A walking unit that keeps its
      destination after a failed search waits for the delay with it (S-0212 claim 23, third row).
      Its movement in progress ended at that failure (*Failed search*, below), so the first request during
      the wait is accepted with its draw, and a new destination different from the kept one ends the
      delay.
    - *Failed at the reachability gate.* When a walking unit's fresh search fails at the
      reachability gate (S-0208 claim 24, S-0212 claim 6), its destination is cleared in the same
      movement update and its movement in progress ends, as with the driving and ship locomotors
      (S-0212 claim 23, the row for the gate's failure). Its
      behaviour is unchanged, so it still rests in *guard* or *area guard*. The first request
      after that update is accepted unless claim 11's first case refuses it, with its draw. No
      request finds the unit holding the destination that failed.
    - *Failed search.* A fresh search that passes the gate but finds no path also ends the
      walking unit's movement in progress in that update, whether or not the unit keeps its
      destination (S-0212 claim 23, the sentence after its table on rows 2 and 3), so the first
      request after it is accepted in the same way.

    A destination different from the one the unit holds ends a running retry delay of S-0212 claim
    21, and so does the clearing at the gate's failure. The rest of this paragraph is for a walking
    unit other than a fraidycat whose current behaviour has the scatter property: every unit has
    scatter permission for this request, so S-0208 claim 11's third case refuses each such unit's
    request that is not effectively forced, and the fraidycat exception of *During the walk-off*
    accepts each request instead. The destination of a request accepted after a failed search, where
    it differs from any destination the unit kept, is searched for without a wait for the delay. If
    that search fails as well, the next request is accepted with its draw (the bullets above), and
    the cycle repeats. Such a unit on a building cell whose every scatter destination fails, each
    differing from any destination it kept, therefore takes a request's draw in each turn in which a
    request comes. Where a request gives it the destination it kept, whether later requests are
    refused is not established (Open question 16). A destination whose search succeeds is walked to
    as above.

    **During the walk-off.** An accepted request whose destination comes from S-0212 claim 25's
    search leaves the unit's current behaviour as it was. So a walking unit that has accepted one
    is still in *guard* or *area guard* while it walks off, and the request is sent in each of
    its turns until its position leaves the cell that holds K. Its locomotor then has movement
    in progress (S-0208 Terms), so the request is not effectively forced, and S-0208 claim 11's
    third case refuses it: it takes no draw, and the unit keeps its destination. The exception is
    a unit of a type with the fraidycat flag whose current behaviour has the scatter property:
    the move-anywhere option gives it scatter permission, so it accepts each such request, with
    its draw and a new destination from the search around its walking target's cell (S-0208
    claim 13). When the destination comes instead from S-0208 claim 13's fall-back to the
    neighbours, *move* is queued with it (S-0215 claim 11 decides when it becomes current), and
    until it does the requests are refused in the same way. For a teleporting unit, an accepted
    request whose destination comes from the search gives its locomotor one movement update at
    once, in which it jumps, with no wait, to the landing point that the destination fixed
    (S-0208 claim 19, S-0207 claim 20 d), so the jump falls in the request's own turn. A
    destination from the fall-back to the neighbours gives no such update, and the jump comes at
    the unit's next movement, which in our engine precedes the request in its next turn (above).
    When no landing point is fixed, the destination is removed and the unit does not jump (S-0208
    claim 19), and it rests on the building cell as after a request that gets no destination.

    The rule is not specific to factories, but a factory release is where it is seen:
    - **Released with no rally point.** The release happens when the place order takes effect,
      after the turn pass (S-0404 claim 7; S-0203 claim 7). The unit therefore takes its first
      turn in the next tick unless it misses that turn (S-0203 claim 10). Its queued resting
      behaviour becomes current at the first update at which it is ready to change (S-0215
      claim 11). When the cell claim 10 has it resting on holds K, and K counts here, the
      first request comes in its first turn in which that behaviour is current.
    - **Where it goes.** It goes to the destination of S-0208 claim 13. Which cells qualify is
      S-0212 claim 25's. A qualifying cell has an empty occupancy record, so a destination that
      the search finds is not a cell that holds K, unless it is a bridge cell: there the deck's
      record is read, so a bridge cell over part of K with an empty deck could qualify. No stock
      placement is known that puts a factory's foundation under a bridge, so that case is
      `unmeasured`.
    - **When K is free again.** Its first standing-position arrival after tether
      establishment is claim 10's clear report, which ends contact and tether.
    - **A unit that stays on the foundation.** It keeps both links while each request is
      refused, gets no destination after S-0208 claim 13's fall-back to the neighbours, or, for a
      walking unit, gives a destination for which no path is found (S-0208 claim 24, S-0212 claim
      6's reachability gate) (claim 10).
    - **Unit after unit (Q-1850).** The unit has no walking target when the request comes, so its
      scatter centre cell is its own cell (S-0208 Terms), a cell that holds K. The search of S-0212
      claim 25 around that cell never gives a cell that holds K, other than a bridge cell (*Where it
      goes*), nor one whose occupancy record shows any unit, even one infantry unit in one standing
      position. On
      otherwise open ground it stops at the end of the first ring that has such an empty cell
      passing S-0212 claim 25's visibility test, normally the
      cells next to the foundation around the release cell, and takes the preferred cell at
      position *N* mod *n*, where *N* is the tick number of the request and *n* the number of
      preferred cells (S-0212 claim 25). The request's draw gives only a direction, which plays
      no part in that choice (S-0208 claim 13). So which cell the unit takes depends on the tick
      on which its first turn falls and on which cells already hold units, not on a draw. A later unit of
      the same factory finds the cells of earlier units taken, since each earlier unit rests
      where its walk ended and is not sent claim 17's request there, and so takes another
      cell. A unit of a type without the fraidycat flag accepts one request, with one draw, for
      its walk-off (*During the walk-off*). So no two of them that rest in *guard* end on one cell,
      provided each earlier unit's walk has ended by the next unit's request: a release comes at
      least 54 ticks after the previous one (claim 14), and a walk to a cell next to the
      foundation at a stock infantry speed is expected to end within that time. That expectation
      is untested: this spec has no figure for a stock walking speed, and only U10 f, which has
      not run, would show it. A unit that rests in *area guard* takes its own cell as its guard
      spot (S-0215 claim 26), which can be a cell of K, and may return there, so for *area guard*
      the result is `unmeasured`.
      Where it is released is S-0404 claim 13's exit
      cell rule, unchanged. Each later unit is released when its own place order takes effect
      (S-0404 claim 7) and is not busy, because the earlier unit's contact ended at its first
      arrival (*When K is free again*), so it appears with no wait beyond its build time.
      Only a release that comes before the earlier unit's first arrival is busy, and its item
      is then cancelled with a refund, with no retry (S-0404 claims 7 and 13); with one line
      finishing items at least 54 ticks apart, that needs a first arrival at least 54 ticks
      after the release (claim 14). Thus five units queued at one factory with no rally point
      and nothing else near it, each resting in *guard*, each making its first arrival less
      than 54 ticks after its release and ending its walk before the next request, all come
      out, each in the order boundary in which its own place order takes effect, each
      receives the scatter request in its next turn, and they end on five different cells
      next to or near the foundation, none of which holds K (bridge cells aside, *Where it
      goes*).

    The same conditions apply to a cloning building's copy. A teleporting infantry unit whose accepted request
    produces a destination jumps to it, and its first jump's arrival is its clear report (claim 10). The cell test ignores the unit's
    height, so a jumpjet infantry unit released with no rally point over K's foundation
    receives the request whenever its current behaviour there is *guard* or *area guard*.
    Which behaviour it has there is not established (Open question 3).
    - Backing: `measurable` (U10 b for the first-turn request, its draw, the walk and the contact
      end, U10 f for *Unit after unit*: five releases in a row, their ticks, their cells and no busy
      release, including the stock walk ending within 54 ticks, which no other part tests, U10 b
      part b2 for a cloning copy, U10 e for a teleporting type, and S-0208 W7 for the acceptance
      rules). U10 b measures the request in the turn in which the queued resting behaviour first
      becomes current, by the tick of the first request. A request in the same turn as a later
      behaviour change is `unmeasured`: no part of U10 makes the resting behaviour current at a late
      start. The exempt buildings are `unmeasured` here: no setup in this spec leaves an infantry
      unit resting on such a cell, and the rows themselves carry S-0209's backing. The repeat while
      every request gets no destination is `unmeasured`: no stock setup is known in which neither
      the search around a factory's foundation nor S-0208 claim 13's fall-back to the eight
      neighbours of the scatter centre cell gives one. *Area guard*, buildings other than factories
      and cloning buildings, and the enslaved unit's exception are `unmeasured`: U10 places only
      *guard* infantry on a factory's or cloning building's foundation. No request while being
      erased (a warp's leaving stage included) or while the unit travels through a tunnel is
      `unmeasured`: no part of U10 puts a resting unit on a factory cell in either state. For our
      engine it is a design rule (above); for the original it rests on the request's place within
      the turn, which Open question 11 leaves open. That the request is forced is `measurable` (U10
      b): its unit is of a type without the fraidycat flag and rests with no movement in progress,
      so by S-0208 claim 11's third case it would refuse a request that is not forced and stay on
      the foundation. That it is move-anywhere is `unmeasured`. The option changes the outcome in
      two places of S-0208 claim 11 only: for a unit whose sequence is a deploy kind (its second
      case and the *undeploy* request), and, as scatter permission in its third case, for a unit
      whose locomotor is moving, so that the request is not effectively forced. In the third case
      the option matters only for a fraidycat whose behaviour has the scatter property (S-0208 claim
      11), and no part of U10 uses such a type. For the deploy kinds, no setup here rests a unit
      with a deploy-kind sequence on a factory cell. The bridge-cell destination over K is
      `unmeasured`, as said above. *During the walk-off* is `measurable` (U10 b for a walking unit
      of a type without the fraidycat flag keeping its first destination while it is on a cell that
      holds K, U10 e for a teleporting unit's jump in the tick of its request). The fraidycat
      exception is `unmeasured`: no part of U10 uses a type with the fraidycat flag. The fall-back's
      queued *move*, and a teleporting unit's jump after a fall-back destination, are `unmeasured`
      for the reason given for the repeat above: no stock setup is known in which the search around
      a factory's foundation gives no cell. That the gate's failure clears a walker's destination
      and leaves no retry delay is `measurable` by S-0212 scenario P25(a), as S-0212 claim 21 rates
      it. The rest of *No path* is `unmeasured`: U10 g tests the accepted request after the gate's
      failure with its draw, and the walk to the new destination without a wait for the retry delay,
      which the gate's clearing leaves, but it runs only with a harness that can give an order at
      the unit's release, which is not established. That a new, different destination ends a retry
      delay still running is `unmeasured`: in U10 g no delay runs when the new destination is given,
      and no part of U10 gives a walker a new destination while its delay runs. The cycle repeated
      over several failed searches is `unmeasured`: U10 g gives one destination behind the gate, and
      no stock setup is known in which every scatter destination around a factory fails the gate. A
      failed search that passes the gate, and so the wait with a kept destination, is `unmeasured`
      here: no part of U10 makes one for a resting unit. That the request follows the start of the
      resting behaviour in the same turn is our engine's rule (design), and the original's order
      within the turn is `unmeasured` (Open question 11).
    - Confidence:
      - the conditions and the absence of a threat point, for *guard* infantry on a cell that
        holds a factory or cloning building: medium, until U10 b is run.
      - the first request in the tick after the release, composed with S-0203 claim 7: medium,
        until U10 b is run.
      - the forced option: medium, until U10 b is run.
      - the move-anywhere option: medium (`unmeasured`, as above).
      - the three exclusions (being erased, a warp's leaving stage, a tunnel): medium as our
        engine's design rule; low for the original (`unmeasured`, and the request's place
        within the original's turn is open, Open question 11).
      - *Unit after unit*, composed with S-0208 claim 13, S-0212 claim 25 and S-0404 claims 7
        and 13: medium, until U10 parts b and f are run, and only for the walk length stated
        there.
      - *During the walk-off*, a walking unit keeping its first destination and a teleporting
        unit's jump in its request's turn: medium, until U10 b and e are run. The fraidycat
        exception,
        the fall-back's queued *move* and the jump after a fall-back destination: low
        (`unmeasured`).
      - *No path*: medium for the cleared destination and no retry delay after the gate's failure
        (`measurable`, S-0212 P25(a)), and for the accepted request after it and the walk to its
        destination without a wait for the retry delay (`unmeasured`, U10 g, as above). Medium
        (`unmeasured`) for a new, different destination ending a retry delay still running, as
        S-0212 claim 21 rates it. Low for the end of movement in progress after a failed search that
        passes the gate, as S-0212 claim 23 rates that sentence, for the first request accepted while a
        kept destination waits, and for the repeated cycle (all `unmeasured`).
      - *area guard*, other buildings, the enslaved unit's exception, the request in the same
        turn as a later behaviour change, the repeat in every turn, the exempt buildings and the
        bridge-cell destination (all `unmeasured` here), and the jumpjet case: low.
      - the request's place in the turn, after the behaviour's start, its processing and the
        movement, and our engine's laser-fence states: medium as our engine's design rules, which
        only our engine's fixtures check; the original's order and states are `unmeasured`
        (Open questions 11 and 14), low.

## Formulas

- **F1. Standard delay:** *D* = *D*r + *U*(0, 2) ticks. *D*r is S-0215 F1 in full, including
  double-precision multiplication by 900 ticks per game minute, rounding, signed-integer
  conversion and exceptional values. The rate is imported by S-0003's decimal form, not
  invariably stored as a single. S-0215 owns scheduling for delays of 0 or 1 tick.
  Backing: `measurable` (U3); confidence high. Exceptional conversions inherit S-0215 F1's backing.
- **F2. Door time:** use S-0215 F9's deploy-time conversion for both opening and closing with
  K's imported deploy time *t*. For ordinary nonnegative values,
  *T*d = trunc(RZ₅₃(*t* × 900)) ticks with `quirks.chop_rounding` on, RN₅₃ instead with it off.
  This is double-precision floating-point multiplication followed by truncation toward zero.
  F9 owns the signed 32-bit wrap and exceptional conversion. Negative converted durations
  behave as 0. The door becomes stationary on its timer update once the duration has elapsed.
  Duration 0 completes at the first update, not synchronously with the opening request.
  Backing: `measurable` (U2 for stock door duration); zero and negative durations are
  `unmeasured` here because U2 does not establish a stock factory with them. Confidence high
  for the shared conversion; exceptional conversions inherit F9's backing.
- **F3. Apron:** E = (*x*₀ + *a* − 1, *y*₀ + *b*), integer cell coordinates, with the signed
  16-bit result range, −32768 … 32767, reducing each result modulo 65536 into that range. (*a*, *b*) is claim 3's eleventh pair or the
  special pair of Edge cases. E's centre is (256 Eₓ + 128, 256 Eᵧ + 128) leptons.
  Backing: `measurable` (U4 for stock aprons); signed wrapping is `unmeasured` here because
  those setups do not reach the range boundary. Confidence high.
- **F4. Other-locomotor destination:** (*x*₀ + 4, *y*₀ + 1), integer cells with the same wrapping.
  Backing: `measurable` (U6 hovering case); wrapping is `unmeasured` here because the setup
  does not reach the range boundary. Confidence high.

## Constants

| Name (ours) | Value | Unit | Use |
|---|---|---|---|
| Rate and door time conversion | 900 | ticks per game minute | F1, F2 |
| Delay draw | 0 … 2 inclusive | ticks | every unloading processing |
| Idle clearing rate factor | 3 | dimensionless | claim 13, S-0215 claim 32 |
| Apron list position | 11 | entries counted from 1 | claim 3 |
| Apron displacement from that pair | (−1, 0) | cells | F3 |
| Exit speed fraction | 0.5 | fraction | claim 5 |
| Exit target fraction | 1.0 | fraction | claim 6 |
| Fixed fallback destination displacement | (4, 1) | cells | F4 |
| Exit track count | 69 | points | T1 |
| Exit track facing | 64 | 1/256 turn | 16384 facing units, map east |

**T1. Exit track.** Integer displacements in leptons from E's centre, in driven order. Every
point has facing 64. For points 1 to 68, *y* = *n* − 69 leptons, where *n* counts from 1.

| Points | x | y |
|---|---|---|
| 1 to 3 | −670, −660, −650 | −68, −67, −66 |
| 4 | −639 | −65 |
| 5 to 68 | −630 + 10 (*n* − 5) | *n* − 69 |
| 69 | 0 | 0 |

## State and ordering

**S1. Ground factory unloading.** Every row assigns F1's delay. Door updates and behaviour
processing are distinct; S-0203 owns their tick ordering.

| State | Event at processing | Effect | Next state |
|---|---|---|---|
| starting | always | contacted unit starts guard, door opening, production animation | apron clearing |
| apron clearing | blocked | scatter requests | apron clearing |
| apron clearing | free | no exit movement yet | letting out |
| letting out | door not stationary open | none | letting out |
| letting out | door open, contact absent | start closing | closing |
| letting out | door open, contact present | queue move, exit instruction, set fraction | waiting |
| waiting | tether remains | none | waiting |
| waiting | tether ended | start closing | closing |
| closing | door not stationary closed | none | closing |
| closing | door closed | stop production animation, queue guard | ends when guard starts |

The vehicle cannot be sent out before the third processing, even with a clear apron and zero
door time. The waiting stage need not observe an empty foundation: contact can end at E. A
factory processing after that report can start closing on the same tick or a later one, depending
on actor update order, rather than necessarily one tick after physical departure. Random ordering
requirements concern the resulting direction choices, delay and stream state (claim 12).
Backing: `measurable` (U2 for the stage minimum and closure following tether loss), confidence
high. The same-tick closure possibility is `unmeasured` here: U2 does not establish both
relative actor update orders; confidence medium.

## Edge cases and quirks

- **Short exit lists: `quirks.factory_apron_short_list`.** Let *L* be the number of
  entries in the foundation's original exit list (S-0404 Constants), before any full-ring
  alternative. Default original: for *L* = 10 entries the pair used by F3 is
  (32767, 32767) cells; for *L* < 10 entries it is (0, 0) cells. These are engine
  constants, not imported coordinate values. The first case gives an outlying apron with
  signed wrapping; the second gives (*x*₀ − 1, *y*₀). Lists with at least eleven entries
  use claim 3 unchanged. These length-based rules cover all foundation words S-0404 defines.
  - **The outlying apron in the original.** With F3's signed wrapping, the apron from the
    constant (32767, 32767) pair selects no legal cell (S-0009 Terms and claim 1) for any
    top-left cell a map can have. The
    original then reads E as a cell holding no object: the apron is never blocked and no clearing request is
    sent. At letting out, the exit-track instruction is accepted, and the first point carries the
    vehicle to E's centre plus (−670, −68) leptons, far off the map. What follows has not been
    traced, and our engine cannot hold an object off the map.
  - **An apron that is not a legal map cell.** When E's coordinates select no legal cell
    (S-0009 Terms and claim 1), the apron counts as free, as in the original. At letting out,
    the exit instruction is refused and u stays where it appeared. u gives its clear report
    at once, as on a cell-arrival update with no destination, so K can close under table S1.
    Claim 8's reply then applies. This is a documented departure from the off-map journey,
    applying under both states of `quirks.factory_apron_short_list`.
  - **E a legal cell outside the usable area**, for example a factory at the map edge, is an
    ordinary apron: claims 4 to 8 apply unchanged.
  - **Fixed alternative** (switch off). For a nonempty list shorter than eleven entries,
    the pair used by F3 is its last listed pair. For an empty list it is (0, 0) cells.
    Lists of eleven or more entries keep their eleventh pair.
  - **With S-0404's `quirks.foundation_exit_lists` off.** E still uses the original lists' pairs
    (this edge case and claim 3), whatever that switch's state. With
    `quirks.factory_apron_short_list` off, its last-pair alternative likewise uses the original
    short list, not the full ring. The apron selection is separate from the infantry exit
    search, so the full ring's length never matters here.
  - Backing: `unmeasured` (no stock ground vehicle factory with a short exit list is
    established; the
    fixed alternative, invalid-cell handling and independence from full rings are design).
    Confidence high (pairs, the apron read as free, the accepted original instruction and
    the decided design rules); low (what the original does after the off-map placement).
- **`quirks.factory_unload_waits_ready_mark`.** Proposed default original: a factory's queued
  *unload* and the *guard* queued at its end both wait for the ready mark (claim 2), so an
  unarmed factory whose body range never sets the mark and has no other source of that mark
  either never starts unloading a released vehicle or, once
  unloading, never leaves *unload*. Fixed alternative: queueing *unload* at release, and the
  processing that queues *guard* at the end of unloading (claims 2 and 9), also set the ready mark.
  *Unload* then starts at K's next per-tick start, and *guard* at the second per-tick start of the
  closing tick.
  Backing: `unmeasured` (no stock vehicle factory with a non-reporting body range or a turret is
  established); confidence medium.
- **`quirks.factory_fixed_hover_exit`.** Proposed default original: non-driving locomotors other
  than the temporarily replaced two use F4, regardless of foundation. Fixed alternative: use E.
  Backing: `measurable` (U6 c for default); confidence high.
- **Refusal blocks release movement.** An occupant refusing scatter can keep the factory in
  apron clearing indefinitely. This is the ordinary refusal rule, not an extra factory quirk.
  Backing: `measurable` (U5 d); confidence high.
- **Released unit on apron.** Only K is excluded from the blocker test. If placement puts u in E,
  it receives the clearing request as well. It moves only if S-0207's acceptance gates pass:
  being in guard alone does not guarantee acceptance, for example while its body is turning.
  Backing: `unmeasured` (no stock exit placement on E established); confidence high (request),
  medium (subsequent interaction). The track still ignores its appear point if later instructed.
- **Unsuccessful track instruction.** A target-cell or unit-state refusal does not by itself
  prove u remains motionless forever: the instruction can leave partial movement state. Subsequent
  movement and contact loss have not been traced. Backing: `unmeasured` (no stock setup established).
  Confidence low. No permanent-wait rule is prescribed (Open question 4).
- **Contact lost before letting out.** K still clears the apron and waits for an open door before
  closing without a contacted unit. Backing: `measurable` (U2 b); confidence high.
- **Changing rally after release.** Ground vehicles retain the saved rally destination rather
  than reading a later change at their clear report. Backing: `measurable` (U8 d); confidence high.

## Import mapping

| Key | File and section | Concept and parse form | Absent default |
|---|---|---|---|
| `DeployTime` | rules, building type | decimal game minutes, door time | held value, engine default 0 (S-0215) |
| `Rate` in `[Unload]` | rules | decimal game minutes, behaviour rate | S-0215 |
| `WeaponsFactory` | rules, building type | boolean vehicle factory kind | no (S-0404) |
| `Naval` | rules, building type | boolean naval kind | no (S-0404) |
| `Foundation` | art, building type | foundation word and exit list | S-0406 |
| `ExitCoord` | rules, building type | integer triple in leptons, release point | (0, 0, 0) (S-0404) |
| `Locomotor` | rules, vehicle type | locomotor identifier, exit mode | S-0207 |
| `Accelerates` | rules, vehicle type | boolean speed-update choice | S-0207 |
| `Harvester`, `Weeder` | rules, vehicle type | booleans selecting claim 8's harvest reply and claim 16's field-leg case | form and defaults owned by S-0402 |
| `Teleporter` | rules, vehicle type | boolean teleport-capable type flag, distinct from the active locomotion used by claim 16 | form and default owned by S-0402 |
| No data key | runtime exit movement and order destinations | claim 16's replacement, clearing and restoration events | no factory-exit restoration timeout |
| `JumpJet`, `Teleporter` | rules, infantry type | boolean exceptions to initial exit destination | S-0208 and S-0402 |
| `Locomotor` | rules, infantry type | locomotor identifier, parsed as S-0207 claim 14; actual teleporting locomotion supplies claim 10's jump update, distinct from the boolean destination exception | S-0207 claim 14 |
| No data key | runtime factory contact and tether | claim 10's teleport arrival report, claim 15's jumpjet ascent report, and claim 10's retention for a resting unit whose every request is refused or gets no destination (`unmeasured`), not an imported delay | no release-contact timeout |
| `JumpJet`, `BalloonHover` | rules, infantry or vehicle type | booleans; either enables claim 15's ascent-notification response; the latter selects the half-height threshold only for vehicles | no for each (S-0502) |
| `JumpjetHeight` | rules, infantry or vehicle type | integer leptons; imported cruise height starts the ascent target (S-0502 claims 1 and 5); claim 15 reads the current target, not a separate factory height | S-0102 and S-0502 |
| `GDIBarracks`, `NODBarracks`, `YuriBarracks` | rules, building type | booleans; barracks door kinds that qualify a contacted building for claim 15, as does `WeaponsFactory` | no for each (S-0404) |
| `Rate` in the current behaviour's section (`[Guard]` and the others named by S-0215) | rules | decimal game minutes, rate of claim 13's delay | S-0215 |
| `HasStupidGuardMode` | rules, building type | boolean; set means no idle apron clearing (claim 13) | S-0215 |
| `AirportBound` | rules, aircraft type | boolean; set means the type refuses the release tether (claim 11) | form and default owned by S-0407 |
| `NumberOfDocks` | rules, building type | integer docking-place count *d*; contact places max(*d*, 1) (claim 14) | owned by S-0404 |
| `AnimIdle` and the other body ranges | art, building type | body ranges whose stage updates set the ready mark (claim 2) | owned by S-0108 |

## Measurement scenarios

Use stock data, no engine add-on and no mods (the named test files of a scenario excepted), a tiny flat clear map, one human house and one bystander computer player, fixed game speed index 2 (S-0201 supplies real-time pacing). Run each part with three recorded seeds. Use scripted input and a read-only recorder for tick numbers, positions, destinations, contact/tether, behaviours and scenario stream state. These are recorder proposals, not claims that all fields are currently exposed. The observed area comprises the tested factories, their foundations, exit tracks, aprons and neighbours, and every route or destination followed by a named test unit. Apply the bystander void rule: repeat a run if an unnamed bystander object enters that area or acts on a test object, or if the bystander is defeated before observation ends. Do not assume the bystander remains idle. Separate other actors' draws and animation events from unloading draws; recording only the global tick draw count is insufficient. Do not change data to manufacture the unmeasured cases above.

- **U1, factory kinds.** Produce a vehicle, ship, infantry and aircraft from their respective
  factories. Record factory behaviours and door state after release. Expect only the vehicle and
  naval factories to enter unload. Falsifiers: infantry or aircraft factory entering these stages,
  or a ground or naval factory with its ready mark set and no pre-existing contact failing to
  enter them. Record contact at release: expect new contact with the ground vehicle, but none
  with the ship; either result reversed falsifies that contact rule. Cloning is not exercised
  by this part. Compare busy release/refund with S-0404 P12, not a retained-item retry.
- **U2, door and stages.** With a clear apron, produce a vehicle and record every processing,
  door timer update, movement start, contact loss and queued guard. Also record K's current and
  queued behaviour and ready mark at each of its two per-tick starts. Use a stock factory whose
  running body range has delay 0. Expect *guard* queued, not current, after the closing
  processing, and current at the second per-tick start of that tick; *guard* is first processed
  on the next tick. Falsifier: *guard* current before closing, or later than that second start. Expect F2 duration, at least
  three processings before sending out, and closing enabled by tether loss rather than physical
  departure. Falsifier: exit instruction before the door is stationary open or before processing
  three, or a door becoming stationary before or after F2's predicted timer update.
  At the first unloading processing also record u's current behaviour, the door's state and
  K's idle and production animations. Expect u to start *guard*, the door to start opening,
  the idle animation to stop, and the production animation to start when its slot has art.
  A missing change or a change at a different unloading stage falsifies the corresponding
  start effect. Continue animation recording through closing: expect production to stop
  when *guard* is queued; stopping at another stage falsifies that effect.
  Record the ready mark throughout unloading; a change attributable to unloading or release,
  rather than the body update or a successful behaviour start, falsifies the no-change rule.
  Part b: destroy the contacted vehicle while a persistent blocker occupies E, then remove
  the blocker. Expect clearing to continue until E is free, then closure after the door opens.
  Immediate closure while blocked falsifies the rule.
- **U3, draw assignments.** Repeat U2 and U5 with the random-state recorder. Label factory
  processing and production-animation events. Expect one ranged delay assignment per processing.
  A blocked processing assigns accepting neighbour vehicles' direction draws first, in direction
  and cell-list order. Starting animation may add its own S-0202 draws. Falsifier: a second
  unloading delay assignment, a missing assignment, or a reversed assignment to a scatter
  direction and the delay. Include U9's two delay assignments. For U2 with a saved rally,
  separately record the stream states immediately before and after door start/end, the ordinary
  drive-track instruction, the clear report and the rally assignment. Expect no draw attributable
  to these events; an attributable draw falsifies the corresponding exclusion. Runs whose
  intervening animation or other-actor draws cannot be separated do not score that exclusion.
  In U5, include an accepting infantry occupant of E and of each of two differently numbered
  neighbours, logging its scatter draws under S-0208. Expect accepting E occupants' values
  before neighbour values, neighbours' values in step-direction order, and the delay value last.
  Where several occupants can share a cell, record their S-0209 listing and assigned values.
  Reversal falsifies the same-cell rule. A setup unable to establish multiple accepting occupants
  supplies no backing for that part.
- **U4, apron.** For each stock vehicle factory type place a persistent refusing unit on candidate
  cells in separate map runs, using pre-placement where normal orders cannot reach them.
  Record processing state and movement. Only occupancy of F3's E should hold apron clearing.
  Falsifier: another candidate alone holding it, or E not holding it. A stock scenario that cannot
  put a blocker there supplies no backing for that foundation word.
- **U5, scatter.** Part a: idle vehicle in E, expect no scatter draw and S-0212's nearby-cell
  destination. Part b: also idle vehicle in a neighbour, expect one 0 … 2 draw and S-0207's
  threat-point candidate selection. Part c: neighbour only, E free, expect no clearing request.
  Part d: deployed or sticky blocker in E, expect repeated clearing and no sending out until it
  leaves. Record requests, destinations and factory state. Falsifiers: a vehicle no-threat draw,
  a missing or extra neighbour draw, a destination differing from the owning movement rule,
  neighbour clearing with E free, or exit instruction while E remains blocked.
- **U6, exit modes.** Produce a stock drive, teleporting and hovering vehicle; use a stock
  tunnelling type only if one is available from a vehicle factory. Record every position and
  locomotor change and facing. Compare each passed point and facing with T1.
  For driving exits, also record the heading point, separate locomotor destination point
  and vehicle world height from the successful instruction through track completion.
  Repeat with a legally placed factory on elevated terrain, using S-0207 D28's controls.
  Expect both supplied points at E's horizontal centre with height 0 leptons, the heading
  height unchanged throughout the exit track, and the vehicle's own height following
  S-0207 claim 5. A starting-height, terrain-height or deck-height assignment to either
  supplied point, an extra deck lift, or forcing vehicle height to 0 falsifies the
  corresponding rule. Record the separate destination before any completion or reply
  can clear or replace it under S-0207 claim 25; later replacement is not a falsifier.
  Driving and successful temporary driving use T1, with first-point position
  independent of release displacement. Hovering uses F4, not T1. Falsifier: a different track or
  a hovering vehicle following T1, a destination other than F4 for that hovering vehicle,
  or a speed fraction other than 0.5 immediately after the exit instruction.
  For the hovering exit, also record ρ, hover speed, provisional step length, movement
  heading and heading point each tick. Also apply S-0503 H8: record the first point choice
  and cell-arrival update during the exit instruction. On a successful clear choice for a
  resting powered hover, expect a heading point before K sets ρ = ½ and an update at the
  pre-travel position, with report suppression if it is a building cell with a different
  destination. A compulsory next-tick first choice, an update at the chosen point rather
  than the actual position, or a report despite that suppression falsifies the rule.
  Compare later update events, contact and tether against S-0503 claim 15's arrival
  outcome table and H8/H15, not cell-edge crossings or destination clearing alone.
  Record both the order-level destination and the hovering movement destination,
  plus search timing, retry delay, held state and same-cell contact. Where H15's
  conditional stock fixtures establish a deferred search or a failure during choice,
  expect an update despite no new point; a refusal instead gives none. An initial
  arrival-update failure clearing the movement destination and point gives no update
  without same-cell contact. With no movement destination but same-cell contact and
  no hold, expect an update. The opposite result falsifies the corresponding row.
  Score only established fixtures, not an assumed ability to manufacture contact.
  For stop controls use S-0503 H12/H15 and S-0213 claim 8: tethered stops are refused,
  while an accepted stop clears the order-level destination and breaks contact even
  if movement quantities survive. A later update alone must not count as a report to
  the former factory. An accepted tethered stop or surviving contact after an accepted
  stop falsifies the relevant order rule; a run without an eligible untethered contact
  supplies no measurement of that contact break.
  Expect ρ = ½ to persist only until a reset listed
  in S-0503 claim 4. In particular, a 0-lepton provisional step with heading exactly
  towards the heading point resets ρ to 1, without itself setting hover speed to 1.
  A fraction change without a listed reset, or no reset in that stalled-start state,
  falsifies the corresponding rule. A run with no stalled start does not test that reset.
  Initial fractional-progress ticks are S-0207's, not necessarily
  a complete first-point jump. Unavailable stock types remain untested.
  For the teleporting vehicle, also record destination, suspended landing point, pending jump,
  phasing-in state and expiry tick, and ground/deck occupancy marks before temporary driving,
  throughout it and across restoration. Also record the suspended predecessor's presence,
  driving movement report, locomotor destination and heading points, actual horizontal
  position, restoration lock and seizure at each restoration attempt. Under S-0215 claim 13,
  expect restoration only with a predecessor present, no movement report, no lock and no
  seizure (S-0207 claim 27). Track completion alone must not restore while a locomotor
  destination remains or a heading point differs horizontally from the actual position.
  Restoration despite any failed predicate falsifies eligibility; failure to restore at
  a fall-back attempt with all predicates met falsifies the successful-restoration rule.
  Score locked or seized controls only if an ordinary stock sequence establishes them.
  do not manufacture states through memory writes. Where the recorder can isolate
  restoration, record destination refresh and one-application point suppression before
  and after: neither is enabled by restoration alone. Either newly enabled solely by
  restoration falsifies that rule; a trace unable to separate destination assignments
  supplies no backing for this exclusion.
  Separate restoration itself from the other effects of falling back to rest.
  Repeat with a scripted move order during temporary driving, scoring
  only runs where it is accepted then. After restoration and fall-back finish, give a new
  distant clear-cell move order. Expect assignments completed entirely in driving
  locomotion to leave the suspended teleporting landing point and reservation unchanged.
  Occupancy follows S-0209's
  driving rules. Restoration alone neither resets teleporting state nor moves a mark. The
  stock teleporting vehicle is a chrono miner, whose type has the teleport-capable flag: its
  newly taken destination after restoration installs temporary driving again and is driven
  (S-0207 claim 28). The restoration still shows: the active movement kind changes from
  driving to teleporting at the fall-back and back to driving on the order's tick. Only for a
  vehicle type with the teleporting locomotor and without that flag, if one is used, does a
  newly taken destination after restoration fix a landing point and jump under S-0207 claim 20.
  A qualified docking assignment that restores within the assignment also applies its
  point to the restored locomotion under S-0207 claim 27, not to driving first.
  A changed suspended landing point or teleport reservation due to an assignment
  completed entirely in driving locomotion, a
  state reset or occupancy action due solely to restoration, no restoration at the eligible
  fall-back, or a jump by the stock vehicle to that post-restoration order's cell falsifies the
  corresponding rule (S-0207 D23). Record any
  factory-supplied destination before temporary driving starts: do not presume an empty
  suspended state. No stock setup for a pending jump or phasing-in state at installation is
  established, so U6 does not score S-0207 claim 21's conditional rules for them.
- **U7, speed.** Produce stock accelerating and non-accelerating drive types and record speed
  fractions, per-tick positions and arrival time. Calculate each prediction using its own imported
  speed, bonuses, S-0207 progress formula and T1. Expect 0.5 retained for accelerating types,
  1.0 on movement update for non-accelerating types, absent another effect. Falsifier: ordinary
  acceleration increasing the former on this track or half speed retained by the latter. Do not
  compare raw tick counts of different-speed types as though their speeds were equal.
- **U8, clear and rally.** Part a: saved rally point, expect the normal clear report at E and
  direct rally destination, without a mandatory intervening nearby-cell stop. Part b: no rally,
  expect one run-away scatter destination, not a required two-scatter sequence. Part c: ore
  gatherer, expect harvest queued at the reply. Part d: set rally A, release, change to B before
  clear report, expect A retained. Record destinations and tether tick at track end. In every
  part, record u's cell per tick and the tick of the report: expect the report on the tick u
  reaches E's centre, never on an earlier tick in which u crosses into a new cell. Falsifiers:
  a report before reaching E's centre, no report on its track-target update despite the report
  conditions being met, a mandatory intermediate stop in a, two obligatory scatter destinations
  in b, harvest not queued in c, or B replacing saved A in d.
- **U9, naval.** Produce a ship with rally set. Record two unload processings, two delay
  assignments, then guard queued; the ship keeps its release move. Door animation can continue
  after unloading. Record contact/tether, door state, idle and production animations, and the
  queued/current guard at both per-tick starts as in U2. Expect the first processing to start
  door opening and the production animation when its slot has art, and the second to stop that
  animation and queue guard even if the door is still moving. With the ready mark set, expect
  guard current at the second start of the second processing tick. Falsifiers: a third unload
  processing, unloading creating contact with the ship or redirecting it, a missing delay
  assignment, a missing or differently timed animation change, the second processing waiting
  for the door to stop, or a different guard-start tick under that ready-mark condition.
- **U10, infantry.** Part a: rally point set several cells from an infantry factory. Record
  the saved guard spot, current and queued behaviours, destination, position, cell and walking
  target per tick from release, plus contact and tether. Distinguish boundary crossings,
  standing-position arrival snaps, first departure from the foundation and reaching the exit.
  Expect: on the release tick the destination is the exit cell, *move* is queued and the guard
  spot equals the rally point. Contact and tether end on the first standing-position arrival
  after tether establishment, without a run-away reply. The unit continues to the exit cell
  if it has not reached it, then heads for the rally point with an empty guard spot.
  Include S-0208 W15's multi-cell release arrangement and, where legal placement and exit
  geometry permit, its first-arrival-on-foundation subcase. W15 permits a named test-map
  release-displacement override if stock geometry gives only one arrival before the exit.
  This is an exception to the no-data-change instruction above, limited to that arrangement.
  Keep all other rules unchanged and score only runs with successful placement, contact and
  tether. An unavailable foundation-arrival subcase remains untested.
  Falsifiers: rally point as the first destination, barracks unload stages or door movement,
  a guard spot differing from the saved rally on release or remaining nonempty on the second
  leg, contact/tether loss merely at boundary crossing, retention past the first ordinary
  standing-position arrival (including on the foundation), premature completion of the exit
  walk caused solely by the report, or a vehicle-factory run-away reply given by the infantry
  factory. Contact exchange draws are not scored from the release's total
  draw count, because placement's own draws remain open. Part b (Q-1757): no rally point, a
  stock infantry factory on open flat ground with no other units within 3 cells. With the
  random-state recorder, record per tick from release the current and queued behaviour,
  destination, position, cell, contact, tether and the draws, for 300 ticks. Then order a second
  infantry unit and record its release result. Then set a rally point and record the first
  unit for a further 100 ticks. Also record whether the cell the unit is released onto holds K.
  Use an infantry type without the fraidycat flag, and record its type.
  The expectation is that every stock infantry factory type releases onto a cell of its own
  foundation, since S-0404 claim 13 moves the exit cell one cell toward the foundation
  rectangle. The record guards against a foundation cell that the
  building does not occupy: a run in which the cell does not hold K is reported so and does
  not test claim 17's request. Expect:
  - no destination and no *move* from the release itself,
  - in the tick after the release, the unit's first turn, in which its queued resting
    behaviour becomes current (S-0215 claim 11), a scatter request with no threat point that
    the unit accepts (claim 17; since the type has no fraidycat flag, only a forced request is
    accepted, so this tells forced from not forced, but not move-anywhere), its
    single draw, and a destination by S-0208
    claim 13 that is not a cell that holds K (bridge cells aside, claim 17, *Where it goes*; the
    test ground has no bridge),
  - that destination kept, unchanged, while the unit's position is on a cell that holds K,
    since the requests of its later turns there are refused while it walks (claim 17, *During
    the walk-off*),
  - contact and tether ending on its first standing-position arrival,
  - the second release not busy,
  - the first unit not moved by the later rally point.

  Falsifiers: the unit still on the foundation after 300 ticks with no order (as a request that is
  not forced would leave it), no scatter request in the tick after the release although the unit
  takes its turn then (a first request one tick later falsifies the request in the turn in which the
  behaviour becomes current, claim 17), a destination given by the release itself, contact kept past
  its first arrival, a destination change while the unit is still on a cell that holds K, a busy
  second release, or the resting unit moved by the later rally point. Run it with each stock
  infantry factory type available to the test house. Part b2: the same with a cloning building of
  the same house, recording the copy as the original unit. Expect the copy to rest on the cloning
  building's foundation and to be sent the same request in its first turn. Part c: rally point set,
  stock jumpjet infantry type if one is available from an infantry factory. Expect the rally point
  as first destination and an empty guard spot. Falsifier: an exit-cell leg. Also record contact,
  tether, flight state, target height and S-0502 claim 7's bridge-adjusted height on each ascent
  tick. Expect both links to end at the first qualifying ascent update of claim 15, not at
  touchdown. Apply U14's tick and destination recording to score that report. Part d (Q-0697):
  produce stock teleporting infantry with a clear rally cell several cells away. Record landing
  reservation, actual position, contact, tether, factory contact-place use, current/queued
  behaviour, destination and phasing-in state from release through the first jump and phasing-in
  completion. Expect direct rally movement, contact and tether ended in the jump tick's arrival
  update, not on reservation, phasing-in completion or fall-back. Expect the contacted place free
  then, with no factory run-away scatter. For an ordering control, give the landing cell an
  entered-by tag whose action changes a scenario variable. The proposed read-only recorder samples
  contact and tether when that variable changes: expect them already ended. End-of-tick coincidence
  alone cannot score this ordering. Produce another unit after the jump: with a free exit and no
  other blocker, the old contact must not make release busy. Falsifiers: either link surviving the
  ordinary jump arrival, loss on reservation alone, a run-away detour, or contact still present when
  the tag action executes. A run with an entry interaction removing the unit is not scored. Part e:
  repeat part d with no rally point and no new order, for 300 ticks, recording with the random-state
  recorder as well every position, arrival update, contact change and draw. Expect no
  release-supplied destination. In its first turn, expect claim 17's request, and then a jump to
  that request's destination in the tick of the request's draw (claim 17, *During the walk-off*),
  with contact and tether ended in that jump's arrival update. Produce a second unit after the jump:
  expect it released, not busy. Falsifiers: no request in the first turn although the unit takes
  that turn with its resting behaviour current, no jump while the request has a destination, a jump
  on a later tick than the request's draw, contact kept after the jump, or a busy second release
  after it. Part f (Q-1850): as part b, but queue five units of that infantry type in a row at the
  one factory, with no rally point and nothing else within 3 cells of the factory. With the
  random-state recorder and the read-only recorder, record for each unit its release tick and
  result, the tick its item became done and the tick its place order took effect, the sidebar state
  of each item, and per tick for 300 ticks after its release its current and queued behaviour,
  destination, position, cell, contact and tether, and the occupancy of the cells within 2 cells of
  the foundation at each request tick. Expect: each unit released (not busy) in the order boundary
  in which its place order takes effect, its item neither cancelled nor refunded; each sent claim
  17's request in the tick after its release, and its destination the preferred cell at position *N*
  mod *n* of S-0212 claim 25's search around its own cell, predicted from the recorded request tick
  and the recorded occupancy; each contact ended at that unit's first standing-position arrival,
  before the next release; at the end, the five units on five different cells, none of which holds K
  (bridge cells aside; the test ground has no bridge). Falsifiers: a busy release, a cancelled or
  refunded item, or a release later than its place order; a unit still on the foundation after 300
  ticks; two of the units ending on one cell, in a run in which each earlier unit's walk ended
  before the next unit's request (a run in which one did not is reported so and does not test it); a
  destination other than the predicted cell. A run in which a unit's first arrival comes at least 54
  ticks after its release does not test the no-busy expectation (claim 14) and is reported so; a
  cell that the visibility test of S-0212 claim 25 makes non-preferred is taken into the prediction,
  not scored as a falsifier. Part g (Q-1980): as part b, on a test map with a small area in another
  reach region for the unit's movement zone within a few cells of the factory and no bridge to it
  (for example a pen of clear ground closed by wall sections, or S-0212 scenario P3's island). Run 1
  uses stock data. Run 2 sets the path-retry delay field to 0.1 game minutes, giving *R* = 90 ticks
  (S-0212 P25 a), an exception to the no-data-change instruction above limited to this part. With
  the random-state recorder, record per tick from release, for 300 ticks: current and queued
  behaviour, destination, movement in progress, the remaining retry delay, position, cell, whether
  the cell holds K, and the draws. Give the unit by scripted input a move to a cell of that area, as
  an order that takes effect after its release and before its first turn, so that it is still on the
  release cell when it first searches. Give it through a team script if targeting would replace that
  cell (as P3 does). The part runs only with a harness that can queue that order at the release
  itself, for the unit just created, in the tick of the release (S-0215 claim 12 for when it takes
  effect), which is not established. An order given in a later window, after the unit's first turn,
  does not test the part, and such a run is reported so (below). Run 2 counts only if S-0212
  P25(a)'s passed-gate search, run with the same test rules file in the same game folder, reads the
  90 ticks that P25(a) expects. Otherwise it does not test the part and is reported so. Expect:
  - no step towards the area,
  - the destination cleared and movement in progress ended in the tick of the fresh search that
    fails at the gate, with no retry delay left at the end of that tick (S-0212 claim 21),
  - in the first later turn in which *guard* is current and the unit is on a cell that holds K, an
    accepted request with its draw and a new destination,
  - the walk towards that destination without a wait for the retry delay: its first step comes as
    many ticks after the accepted request as part b's first step comes after its first request, and
    in run 2 fewer than *R* − 2 = 88 ticks after the gate's failure.

  Falsifiers: the area's destination still held in a later tick; no draw at the first request after
  the gate's failure although *guard* is current and the unit is on a cell that holds K; a remaining
  retry delay above 0 at the end of the tick of the gate's failure; a first step towards the new
  destination later after the accepted request than part b's after its first request; in run 2, that
  step 88 or more ticks after the gate's failure; a request's draw or a destination change between
  the new destination and that step. A run in which the unit has left the cells of K when the gate
  fails, or whose first request comes before the order to the area takes effect, does not test the
  part and is reported so. Which tick the first request falls on, relative to the failure, is not
  scored (Open question 11).
- **U11, idle apron clearing.** Part a: an idle stock ground vehicle factory in *guard*, no
  production. Place an idle vehicle of the factory's own house on E by map pre-placement or a move
  order. Record the factory's guard processings, the scatter requests and the vehicle's
  destination. Expect no request at the first processing after *guard* starts and a forced,
  move-anywhere request at each later one while E is occupied. Assigned delays are
  3 × *D*r + 0, 1 or 2 ticks from the guard rate; predict processing ticks with S-0215's
  scheduling rule rather than equating every assigned delay with an interval. Part b: also occupy an apron neighbour; expect E's request,
  and the neighbour's request; expect accepting scatter recipients' assigned stream values
  before the delay value, in claim 4's order (random-state recorder). Part c: a stock naval factory
  with a ship on its E; expect the same requests while idle. Part d: an occupied neighbour with
  E free; expect no request. Falsifiers: a request at the first processing, a request with E
  free, a current behaviour changed by clearing, a delay outside the predicted three values,
  or the delay draw assigned before the scatter draws.
- **U12, aircraft tether.** Produce from an aircraft factory with a free docking place a stock
  type that need not return to a dock, and one that must. Record contact and tether on the
  aircraft and K from the release tick, then order each to a destination other than K.
  Record destination assignment, pad-contact release, tether loss, height and ground render
  layer separately, including their tick numbers. Expect: the first is tethered on its release
  tick and has lost the tether by the time it leaves the ground. Both have contact at release.
  The second has no tether. Contact with K ends on taking the different destination, without
  waiting for physical lift-off. Earlier tether loss does not falsify the take-off hypothesis:
  record whether it coincides with destination-triggered contact release instead.
  Falsifiers: missing release contact in either branch, either release branch reversed,
  contact retained after taking the different destination, or the first retaining its tether
  after leaving the ground. If a stock type cannot be produced docked for a branch, record
  that branch as untested, not confirmed.

- **U13, release details.** Part a: an infantry factory with no rally point. Release a second
  unit while the first one still holds its contact: with claim 17 that is only before the first
  unit's first arrival. One line finishes its items at least 54 ticks apart (S-0404 claim 7,
  S-0403 F7), so the first arrival must come at least 54 ticks after the first release. A
  test rules file named *u13a-rules*, supplied through the file system for this part and removed
  after it, adds a test infantry type named *U13SLOW*, buildable at that factory, with a speed
  field low enough that *a* − *r* ≥ 54 ticks, where *r* is the first unit's release tick and *a*
  its first-arrival tick, and with *a* later than the tick of the second release at the line's
  shortest build time; the run records the first unit's release and first-arrival ticks and the
  second item's finishing tick. Score only a run where the first unit actually remains in contact
  when the second release happens; if no run reaches that, the busy result stays unmeasured
  for stock data. Record contact
  places in use and the second release result. For a stock factory building with one contact place
  releasing the test type while the first unit holds the contact, expect busy (claim 14); a released second unit while that place is held falsifies it. This does not test
  the count formula for multi-place factories. Part b: place persistent refusing units on the door cell
  and on the first exit-list cells in separate runs. Record the chosen exit cell. Expect a cell
  holding a unit that would only make a mover wait to be rejected like any other non-free
  result. Part c: an aircraft factory with no free docking place and no rally point, producing a
  type that need not return to a dock. Record the edge-entry cell, the aircraft's behaviour and
  destination. Predict the nearby-cell result from S-0212 claim 25 using the recorded map
  occupancy, the aircraft type's locomotion class (*winged* counted as *track*), its own
  movement zone and the release tick as its selection tick. This ordinary-ground setup
  uses the centre cell's ground reach region and does not score the deck-mode branch.
  Expect *move* to that result around the
  factory's reference-point cell. Falsifiers: a released second infantry unit with every place
  held, a non-free candidate accepted, or a destination differing from the predicted search
  result. Do not score total release draws as the search's draws: placement draws are open,
  and the zero-draw search rule is owned by S-0212.

- **U14, jumpjet factory release (Q-1070).** Use the shared stock skirmish setup,
  three recorded seeds, scripted input and the proposed read-only recorder. Produce
  stock jumpjet infantry and a stock balloon-hover jumpjet vehicle from their normal
  factories, separately with a distant clear rally cell and without a rally point.
  Record each unit update's flight state before state handling, height after physics,
  target height, bridge and on-deck state, contact, tether, guard spot, order-level
  destination, stored path, movement point and commitment. Record factory processing,
  unloading state and later releases. Compute the first tick strictly inside claim 15's
  integer window. Expect both links to end in that unit update, even if still over the
  foundation and before the first cruise or touchdown. With a saved guard spot distinct
  from the order-level destination, expect it as the new destination; otherwise expect
  the order-level destination discarded before scatter. Score scatter under
  S-0207 claim 16 or S-0208 claims 11 to 13. When scatter is refused, expect the previous
  movement point and commitment retained, rather than an effective-clear reroute.
  Falsifiers: either link surviving that eligible
  update, a foundation/distance gate, a different threshold, or a destination or movement
  consequence inconsistent with the corresponding row. For jumpjet infantry without a rally,
  observe for 300 ticks without orders and record any movement, its first tick and whether
  it follows a scatter request of claim 17, without assuming that it must occur.
  If still grounded, give a legal move to a distant clear
  cell; expect the same ascent report once it climbs. A grounded interval does not score
  ascent timing or settle Open question 3. After each report, produce another unit with
  the exit clear: expect no busy result attributable to the old infantry contact, and
  the vehicle factory to close by table S1 and release further production. Record both
  door closure and the new release, rather than equating either with the report tick.
  If a stock factory-produced non-balloon jumpjet vehicle or balloon-hover infantry
  exists, repeat and expect respectively the quarter and quarter thresholds; otherwise
  record those branches as untested. Do not alter type data or force memory states to
  manufacture them. A stock ascent skipping the strict window supplies a diagnostic
  of the original's aftermath, not evidence for the design fallback.

  **Engine-only skipped-window variant (design, not an original-game measurement).**
  Use named engine test data for an otherwise qualifying factory-released jumpjet whose
  first rising physics step reaches or passes the target height without a prior
  qualifying report. Repeat with and without a saved rally, and at equality and above
  the target. Record the same unit and factory observables as above. Expect both links
  to end in that reaching update, with claim 15's saved-guard-spot or destination-discard
  and scatter effects; the factory then follows its ordinary closing or free-contact-place
  rules. A surviving link, a report delayed until cruise, destination arrival or touchdown,
  or different destination effects falsifies the design rule. S-0502 J26 tests the same
  departure; do not score this variant as evidence about the original.

- **U15, temporary driving lifecycle (Q-1110 question 3).** Use the shared stock
  skirmish setup and three recorded seeds. Produce a chrono miner from a ground
  vehicle factory with no rally, a reachable ore field several cells away, and an
  available refinery. Do not issue orders before its first loading. Record each
  destination assignment, order-level destination, locomotor destination and heading
  points, current and queued behaviour, harvest phase, active and suspended locomotion,
  movement report, restoration permission, position, contact and tether. Distinguish
  exit-track completion, run-away harvest queue/start, first ore assignment, final field
  track, first loading and each subsequent restoration attempt. Expect the exit point
  retained at the no-order track completion, harvest queue without restoration, an ore
  point replacing it in driving locomotion, and driving to the first field. Expect
  clearing on qualifying field-track completion, not a mandatory restoration there.
  Continue through a refinery return and docking: score the qualified assignments
  against S-0207 claim 27, not a factory-exit timer. In a separate run, after the first
  ore arrival give an accepted ordinary move to a distant clear cell. Let it complete
  and fall back to rest with no lock or seizure. Expect the point cleared and the
  suspended locomotion restored at the eligible fallback, shown by the active movement
  kind changing from driving to teleporting. The next accepted distant move installs
  temporary driving again and is driven (S-0207 claim 28): the active movement kind is
  driving from that move's tick. At its next refinery approach the miner jumps to the
  docking cell. Falsifiers: exit completion or harvest queue
  alone unconditionally clearing/restoring, a first-field teleport without an
  intervening restoration event, retention despite a qualifying stop, failure to
  restore at an eligible fallback, or a jump to that later move's cell. A different outcome
  caused by a refused order or
  missing restoration predicate does not score those exclusions.

  **No-field control.** Repeat on a map with no resource field and no rally. Observe
  300 ticks after the clear report, without new orders, logging all assignments and
  stops, including idle factory scatter requests. Score retention only during intervals
  without an effective stop or replacement. Expect no factory timer to clear the exit
  point; a clear attributable solely to elapsed exit time falsifies that exclusion.
  Record whether the miner remains on driving locomotion, and which later event first
  clears or replaces the point. Then give an accepted clear-cell move and let its
  completion and fallback exercise restoration as above. This control does not assume
  that no unrelated event can supply a destination, nor prove infinite retention from
  a finite trace. Non-harvesting teleporting factory releases remain untested without
  an ordinary stock-produced type.

## Open questions

1. Aircraft docking position: resolved, S-0404 claim 15 owns pad positions. Still open: test
   claim 11's take-off tether hypothesis and exact tether-loss tick with U12, distinguishing
   destination-triggered pad-contact release from physical lift-off; neither tether timing is
   inherited from the glossary.
2. The aircraft's cell near K without a rally point: resolved by claim 14.
3. Specified, and proposed for measurement (U10 b and e, not yet run), for walking and
   teleporting infantry (Q-0120, Q-0697, Q-1757). Release and
   placement give no destination without a rally point or with a rally in the unit's own cell.
   Claim 17 then sends the resting unit the scatter request in its first turn in which it rests
   on a cell that holds K, and it leaves when that request gets a destination; claim 10's
   no-arrival retention applies only while it stays there. U10 b and e are proposed to test it.
   Still open: claim 17 tests the cell whatever the unit's height, so the open point is whether
   a jumpjet infantry type released with no rally point is in *guard* or *area guard* over K's
   foundation, and so receives claim 17's request, or takes off by some other event.
4. Failed temporary driving replacement and refused exit-track instructions: subsequent damage,
   movement and contact loss remain untraced. They must not be implemented as guaranteed
   permanent waits on the evidence here.
5. Q-0221 short-list alternative and illegal-apron handling: resolved by the decided rules in
   Edge cases. Switch off uses the last pair, or (0, 0) for an empty list. An illegal apron is
   free, the exit instruction is refused and the clear report is immediate under either state.
6. Q-0221 full-ring interaction: resolved. Apron selection retains the original lists' pairs
   under either state of S-0404's full-ring option, as specified in Edge cases.
7. Existing destination of a unit or building at the run-away reply: the additional interaction
   is untraced; no stock produced-unit setup is established.
8. Whether putting a released unit on the map takes random draws of its own (Q-0120, question 5).
   The search and destination rules of this spec take none, but placement's own effects have not
   been traced.
9. Resolved for the ascent interaction (Q-1070): claim 15 establishes the clear report
   and same-update contact/tether loss, with U14 to confirm it. Still unmeasured: a
   original aftermath of a one-step climb that skips the strict window and missing
   type flags; no stock release fixture is established for either. Claim 15 supplies
   the decided reaching-update safety fallback for the skipped window, not an inferred
   original release event. The separate cause of any unsolicited no-rally jumpjet
   take-off remains Open question 3.
10. Q-1757 question 3, a second finished item. Without a rally point, claim 17 normally ends the
    earlier unit's contact at its first arrival, so a later release is busy only if it comes
    before that arrival (claim 14). What becomes of the finished item when the release is busy
    is S-0404's: nothing of a release is retried, and a busy item is cancelled at once (S-0404
    claims 7 and 13, including refunds). U13 a reaches that with a slow test type whose first
    arrival comes at least 54 ticks after its release, the shortest gap between two items of
    one line. With stock types the busy case is not established. Whether the answer differs between factories with one contact
    place and those with several is not established: claim 14's counts other than one are
    `unmeasured`.

11. Q-1757 question 1, the unmeasured parts of claim 17. Claim 17 states them, with their
    backing: the request in the same turn as a later behaviour change, the repeat in every turn
    while each request gets no destination, *area guard*, buildings other than factories and cloning
    buildings, the enslaved unit's exception, and the turns of erasure or a warp's leaving
    stage. No setup in this spec reaches them, and they stay open until one does. The turn in
    which the request comes is claimed in claim 17, including a turn in which a queued
    behaviour becomes current: U10 b tests it for the first turn after a release (medium there),
    and for a later behaviour change it is `unmeasured` and low. A request only from the next
    turn would show as a request one tick later. U10 b observes only the tick of the first
    request and its draw. Where the request falls within the turn, relative to the behaviour
    processing, its draws and the locomotor's progress, is not established for the original,
    since no U10 part makes a processing draw and the request's draw fall in one turn. Claim 17
    gives our engine's order (design).
12. Claim 17 and a superweapon warp, in the original. At the update boundaries of a warp before
    its arrival hold (S-0513 claim 51), our engine does not send claim 17's request (design).
    Whether the original sends it at those boundaries, other than in the leaving stage (claim
    17), is not established.
13. Q-1757 question 4, a rally point set after the release. Claim 10 answers it: the resting
    unit is not moved. That answer is backed `measurable` by U10 b, which has not been run.
14. Claim 17's laser-fence exception. S-0209 does not name the two states of a laser-fence
    section that give no value (row WP8), and which states they are in the original is not
    established. Our engine's rule (design) takes them to be the states in which the section's
    fence is down and shows no beam, so a resting infantry unit on a laser-fence section with its
    fence up is sent the request, and one on a section whose fence is down is not.
15. Q-1757 questions 2 and 5, the unrun parts. Question 2, when the contact ends, is answered by
    claim 17 (the first standing-position arrival after the request's walk or jump), answered
    at medium until U10 part b is run. Question 5's cloning and teleporting parts are answered
    at medium until U10 parts b2 and e are run; its jumpjet part is Open question 3. Question 3
    is answered in part by claim 17: whether a later release is busy with stock types is Open
    question 10.
    Q-1850 questions 1 to 3 are answered by claim 17 (*Unit after unit*), rated medium until
    U10 parts b and f are run, and only on the condition that each earlier unit's walk off the
    foundation ends before the next unit's request: this spec has no figure for a stock walking
    speed, and only U10 f would show that the walk ends within the 54 ticks between releases.
16. Claim 17's scatter cycle when a request gives a walking unit the destination it kept after a
    failed search. S-0212 leaves open whether giving a unit the destination it already holds ends a
    running retry delay, and takes it as not ending it. If it does not end it, the unit holds
    that destination unsearched while the delay runs, and whether its locomotor then has movement in
    progress, so that S-0208 claim 11's third case refuses later requests that are not effectively
    forced with no draw, is not traced.

## Changes for other specs

- **S-0208 claim 11.** Its closing list of forced requests with no threat point (the idle
  scatters and the scatter off a building of S-0215 claim 28) should add claim 17's request.
  Claim 17's request is move-anywhere as well, unlike S-0215 claim 28's, so the two are not the
  same request. Claim 8's request goes to a vehicle (S-0207 claim 16), not to infantry, so it does
  not belong in this list. Claim 17's
  acceptance ("accepts it unless claim 11's first case refuses it") rests on S-0208 claim 11's
  cases for a forced request to a resting unit, so the amendment should keep those cases as they
  are for it.
- **S-0208 claims 12 and 13, the behaviour after an accepted scatter.** Both say only that the
  unit moves to its destination "as under a move order". Claim 17's *During the walk-off* relies
  on the difference between their paths: a destination from claim 13's nearby-cell search leaves
  the unit's current behaviour as it was, while a destination from claim 12 (a threat point) or
  from claim 13's fall-back to the neighbours queues *move* with it. For a teleporting unit, the
  search destination also gives its locomotor one movement update at once, in which it jumps,
  and the other two give no such update. S-0208 should state this in claims 12 and 13. The
  immediate update is asked for the teleporting unit only: for a walking unit this spec relies
  on no such update, and S-0208 claim 15's rule that the walking target is chosen only at the
  next walking step stays as it is.
- **S-0208 claim 23, backing.** It cites this spec's U10 d/e for teleporting release, the
  entered-by ordering and no-jump retention. U10 e has a teleporting unit released with no rally
  point accept claim 17's request and jump off the foundation, so it shows the jump, not
  retention without a jump. The backing should cite U10 d and e for the teleporting release and
  the ordering only. Retention without a jump needs a teleporting unit whose claim 17 request
  gets no destination (after S-0208 claim 13's fall-back to the neighbours), which no stock
  setup is known to give, so that part is `unmeasured`. S-0207
  claim 22's backing cites U10 d/e for the same retention; S-0207 already makes that part
  `unmeasured`.
- **S-0404 claim 13, infantry row.** For a unit released with no rally point, it should point to
  claim 17 here for how the unit leaves the foundation and frees the contact place. Both of
  that claim's sentences that leave the walk-out open change with it: "any later walk-out
  remains S-0417 Open question 3" and "Only subsequent no-rally walk-out remains open there
  (Open question 3)". Claim 17 settles the walk-out for walking and teleporting infantry, and
  Open question 3 here is about jumpjet infantry only, so each sentence should point to claim
  17 and keep Open question 3 for the jumpjet case.
- **S-0404 scenario P12, part (f).** Its setup, "two infantry items that finish in consecutive
  ticks on one infantry factory", cannot be made with one line, which finishes its items at least
  54 ticks apart (S-0404 claim 7, S-0403 F7). Part (f) should take two infantry items finished on
  that line, and score only a run in which the first released unit is still in contact with the
  factory when the second is released. With no rally point that needs a first arrival 54 ticks or
  more after the first release (claim 14 here, *Contact places of a factory*). A run in which the
  first unit has left by then is inconclusive, not a falsification. Its expectation and its low
  confidence stay as they are.
- **S-0215 claims 24 and 26 (*guard*, *area guard*).** A pointer to claim 17. The rule runs in
  the infantry unit's turn whatever its behaviour's processing delay, so it is not a step of
  those processings. Claim 17's backing goes with the pointer: `measurable` (U10 b) for the
  first turn after a no-rally release, and `unmeasured` for the request in the same turn as a
  later behaviour change. The committed S-0215 claim 24 says "The actor stays where it is".
  That sentence needs claim 17 carved out: an infantry unit resting in *guard* on a cell that
  holds a counting building is sent claim 17's scatter request and can move.
- **S-0215 claim 11 (the late start).** It says of a *guard* made current at the late start that
  "nothing of it (search, hold, processing) happens before the unit's next update". Claim 17's
  request is not part of the *guard*, so it needs the same carve-out: in the turn of a late start
  the request can still come, after that start (claim 17).
