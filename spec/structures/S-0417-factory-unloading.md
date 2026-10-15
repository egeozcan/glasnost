---
id: S-0417
title: Factory unloading (doors, exit movement and rally destinations)
topic: structures
revision: 10
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0221, Q-0222, Q-0223]
supersedes: []
---

# S-0417: Factory unloading (doors, exit movement and rally destinations)

## Summary

Factory doors, apron clearing (while unloading and while an idle vehicle factory guards),
vehicle exit movement, the first destinations of released infantry, the tether given to a
released aircraft, the destination chosen on release from contact, and the release details that
S-0404 leaves to this spec (claim 14). S-0404 claim 13 owns release positions, facings, busy and
failed results, and refunds; S-0404 claim 15 owns aircraft pad positions. Skirmish only: campaigns
and network games are out of scope. This answers Q-0221, Q-0222 and Q-0223 and most of Q-0120.
Still open: what moves a released infantry unit when there is no rally point (Open question 3)
and the draws of putting a released unit on the map (Open question 8). Claim 14 owns the
contact-place count, infantry candidate acceptance and edge-entry nearby destination, including
the search's no-draw rule; their untested cases remain qualified there.

**Paired hand-off confirmations.** S-0404 claim 13 retains the fulfilled release
hand-off: release queues *unload*, and contact-place count, infantry candidate acceptance and
edge-entry nearby destination refer to claim 14 here. Nearby-search draws are resolved as none.
Initial-placement draws remain open. S-0212 claim 25 table T5 includes the aircraft factory
edge-entry-without-rally row with claim 14's parameters, and claim 25 uses the release tick to
select its destination. S-0215 claim 11 records that factory release and unloading
neither set nor clear the ready mark, with start consequences owned by claim 2 here. Its claim 13
and S-0207 claim 21 jointly own restoration and retained teleporting state after
claim 5's temporary drive. The decided apron off-state rules remain in Edge cases. Confirm
these companion passages at sign-off. No further change to them is requested by this revision.

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
- **Clear report:** a tethered unit's notice that releases its contact and tether (claim 7).
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
   E's centre, at ground height under the movement rules. E need not lie inside the
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
   from its present position. Tunnelling and teleporting vehicles temporarily receive driving
   locomotion and use the same track; restoration belongs to S-0215 claim 13. S-0207 claim 21
   owns the suspended teleporting state and its occupancy consequences: destinations accepted
   while driving use driving movement, not the suspended teleporting movement, and restoration
   alone neither resets that state nor moves an occupancy mark. Those rules inherit that claim's
   backing and confidence, including its unmeasured pre-existing pending-jump and phasing-in
   cases. If that temporary replacement cannot be established, successful exit is not promised
   (Open question 4).
   Any other locomotor instead gets destination (*x*₀ + 4, *y*₀ + 1), independent of the
   foundation and its exit list. It moves by its own locomotor's rules. K sets u's speed fraction
   to 0.5 after this exit instruction in all these cases.
   - Backing: `measurable` (U6 for driving, teleporting and hovering). Tunnelling is
     `unmeasured` unless U6 establishes a stock factory-produced type; no such setup is
     specified here. Failed temporary replacement is `unmeasured`: no stock setup is
     established that makes it fail.
   - Confidence: high (driving and fixed destination); medium (temporary replacement)

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

7. **Clear report.** On a cell-arrival update (defined below) while tethered, a vehicle reports clear unless it
   is in *unload*, or in *enter* with its contacted building as destination, or both these hold:
   it has a destination other than its present cell, and its cell holds a building. A destination
   object other than a cell is not treated as its own cell for this test. The vehicle-factory
   contact also suppresses a report on that factory's cell with a different destination; the
   building test already covers that case.

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
   - Backing: `measurable` (U2, U8 for normal exits and the report at E). The *enter* and
     *unload* exceptions and the different-destination building-cell suppression are
     `unmeasured` here: no release setup retaining those conditions on an arrival
     update is established. The chain-point update is `unmeasured` here: no scenario part in
     this spec produces a chained curve while tethered, and a factory exit has no chain point.
   - Confidence: high (tests, the two update events, the report only at E on the exit track),
     medium (normal arrival consequence)

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
    interaction that removes the unit before it can report and does not prescribe jumpjet
    or teleporting arrival timing.

    With no rally point (or a rally point in the unit's own cell), neither placement nor the release
    gives a destination or queues *move*. The unit rests in *guard*, and no rule of resting
    infantry found so far moves it off a building cell: the building-cell scatter of a walking
    unit applies only when a *move* ends, and the periodic scatter of a resting unit concerns a unit
    hidden behind higher ground, not a building. What moves the unit off the foundation, if
    anything, is therefore not established (Open question 3). Only a cell arrival produces the
    clear report, so if nothing moved it, it would stay in contact with K and hold one of K's
    contact places (claim 14). With a single contact place the next infantry release would then be
    busy (S-0404 claim 13), which makes the case observable (U10 b).

    The original's computer-controlled release also queues *area guard* and can choose a
    base-cell guard spot. Our player houses use the human rule; neutral and special houses retain
    the original owner rule.
    - Backing: `measurable` (U10 a and S-0208 W15 for ordinary release, first-arrival
      clear-report timing and the two-leg walk; U10 b for the
      absence of an immediate no-rally destination; U10 c for jumpjet release). Teleporting
      infantry, cloning contact hand-over, a rally in the appear cell and zero-draw contact
      exchanges are `unmeasured` here: these parts do not exercise those releases or isolate
      their contact draws. Non-player-house production is `unmeasured`: no stock factory-release
      setup for it is established. Later spontaneous movement in the no-rally case remains an
      open question, not a prediction of a permanent wait.
    - Confidence: high (release destinations, contact hand-over and first-arrival report test),
      medium (the two-leg walk and intermediate/foundation arrival consequences),
      low (later movement in the no-rally case)

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
    question 8).
    - Backing: `measurable` (U3, U11 for delay/scatter assignment and U3's isolated ground
      exit events). Placement guard-spot changes, aircraft contact/tether exchanges and the
      edge-entry destination's no-draw exclusions are `unmeasured` here: the release scenarios
      do not separate their effects from untraced placement draws. Failed replacement's effects
      are `unmeasured`: no stock setup causing that failure is established.
    - Confidence: high (explicit delay and scatter assignments); medium (downstream effects)

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
      factory's release is busy when none of its *n* places is free.
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
    - Backing: `measurable` (U13 a for a single held contact place, U13 b for the entry test,
      U13 c for a successful aircraft nearby destination). Counts other than one, the
      no-result search, deck-mode search and forced-move distinction are `unmeasured` here:
      no scenario part establishes those setups or distinguishes a forced move from an ordinary
      move. Their owning general rules remain as cited.
    - Confidence: high (contact-place count, forced *move* and destinations); medium (the entry
      test's exact results, inherited from the cell-entry rules)

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
| `JumpJet`, `Teleporter` | rules, infantry type | boolean exceptions to initial exit destination | S-0208 and S-0402 |
| `Rate` in the current behaviour's section (`[Guard]` and the others named by S-0215) | rules | decimal game minutes, rate of claim 13's delay | S-0215 |
| `HasStupidGuardMode` | rules, building type | boolean; set means no idle apron clearing (claim 13) | S-0215 |
| `AirportBound` | rules, aircraft type | boolean; set means the type refuses the release tether (claim 11) | form and default owned by S-0407 |
| `NumberOfDocks` | rules, building type | integer docking-place count *d*; contact places max(*d*, 1) (claim 14) | owned by S-0404 |
| `AnimIdle` and the other body ranges | art, building type | body ranges whose stage updates set the ready mark (claim 2) | owned by S-0108 |

## Measurement scenarios

Use stock data, no engine add-ons, director off and mods off, a tiny flat clear map, one human house and one bystander computer player, fixed game speed index 2 (S-0201 supplies real-time pacing). Run each part with three recorded seeds. Use scripted input and a read-only recorder for tick numbers, positions, destinations, contact/tether, behaviours and scenario stream state. These are recorder proposals, not claims that all fields are currently exposed. The observed area comprises the tested factories, their foundations, exit tracks, aprons and neighbours, and every route or destination followed by a named test unit. Apply the bystander void rule: repeat a run if an unnamed bystander object enters that area or acts on a test object, or if the bystander is defeated before observation ends. Do not assume the bystander remains idle. Separate other actors' draws and animation events from unloading draws; recording only the global tick draw count is insufficient. Do not change data to manufacture the unmeasured cases above.

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
  Driving and successful temporary driving use T1, with first-point position
  independent of release displacement. Hovering uses F4, not T1. Falsifier: a different track or
  a hovering vehicle following T1, a destination other than F4 for that hovering vehicle,
  or a speed fraction other than 0.5 immediately after the exit instruction. Initial fractional-progress ticks are S-0207's, not necessarily
  a complete first-point jump. Unavailable stock types remain untested.
  For the teleporting vehicle, also record destination, suspended landing point, pending jump,
  phasing-in state and expiry tick, and ground/deck occupancy marks before temporary driving,
  throughout it and across restoration. Separate restoration itself from the other effects of
  falling back to rest. Repeat with a scripted move order during temporary driving, scoring
  only runs where it is accepted then. After restoration and fall-back finish, give a new
  distant clear-cell move order. Expect destinations accepted while driving to leave the
  suspended teleporting landing point and reservation unchanged; occupancy follows S-0209's
  driving rules. Restoration alone neither resets teleporting state nor moves a mark. A newly
  taken destination after restoration fixes a landing point and jumps under S-0207 claim 20.
  A changed suspended landing point or teleport reservation due to a driving destination, a
  state reset or occupancy action due solely to restoration, or driving rather than teleporting
  for that post-restoration order falsifies the corresponding rule (S-0207 D23). Record any
  factory-supplied destination before temporary driving starts: do not presume an empty
  suspended state. No stock setup for a pre-existing pending jump or phasing-in state at
  installation is established, so U6 does not score those conditional retention rules.
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
  draw count, because placement's own draws remain open. Part b: no rally point.
  Record destination, position, contact and tether over 300 ticks, then order a second infantry
  unit and record its release result. Expect no destination and no *move* from the release
  itself. A destination or queued *move* produced by the release falsifies that immediate
  prediction. Report every later movement and its first tick, without scoring it, and whether the
  second release is busy. Part c: rally point set, stock jumpjet infantry type if one is
  available from an infantry factory. Expect the rally point as first destination and an empty
  guard spot. Falsifier: an exit-cell leg.
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

- **U13, release details.** Part a: an infantry factory with no rally point. Hold the first
  released unit inside the foundation (U10 b), without issuing it any new order, and order a
  second unit. Score only a run where the first unit actually remains in contact. Record contact
  places in use and the second release result. For a stock factory with one contact place,
  expect busy; a released second unit while that place is held falsifies it. This does not test
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

## Open questions

1. Aircraft docking position: resolved, S-0404 claim 15 owns pad positions. Still open: test
   claim 11's take-off tether hypothesis and exact tether-loss tick with U12, distinguishing
   destination-triggered pad-contact release from physical lift-off; neither tether timing is
   inherited from the glossary.
2. The aircraft's cell near K without a rally point: resolved by claim 14.
3. What moves a released infantry unit off the foundation when there is no rally point, or the
   rally point is its own cell (Q-0120). The release and its placement give no destination
   (claim 10). No rule of resting infantry that would move it has been found. U10 b and U13 a
   record what happens.
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

