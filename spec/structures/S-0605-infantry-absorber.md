---
id: S-0605
title: Infantry absorber (who may enter an absorbing building, entry, holding, release)
topic: structures
revision: 6
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-18
measurements: []
requests: [Q-1260, Q-1261, Q-1262, Q-1263, Q-1264, Q-1265, Q-1266, Q-1267, Q-1152]
supersedes: []
---

# S-0605: Infantry absorber (who may enter an absorbing building, entry, holding, release)

## Summary

How units get into an absorbing building (glossary) and how they stay there and come out: which
units the building accepts and how its capacity is counted, how an entrant gets contact or waits
in line, what happens in the tick an entrant goes inside, how passengers are held and who owns
them, the owner's order that lets them out again while the building stands, what interrupts an
entry, what ends an entrant's contact, the bio-reactor fate of a taken or released unit (claim
16), and every event that changes the number of passengers. Infantry absorbers are the case
these rules are written for; the same rules with a few differences serve a building type that
absorbs vehicles (claim 8). Skirmish only; network games and campaigns are out of scope.

Neighbouring specs own the rest, and this spec does not repeat them:
- the power each passenger gives, its health scaling, the house totals and when power is worked
  out again: S-0405 (claims 1, 3, 4 and 7),
- passengers that come out when the building is destroyed, the passengers that die with it,
  their credit and their orders afterwards: S-0312 (claims 24 and 25),
- passengers released by a sale: S-0408 (claims 6 and 8),
- the building's animation that depends on the passengers: S-0108 (claim 17),
- the passenger pips: S-0111 (claim 12),
- the repeat of an enter-nearest search in *guard* and *area guard*: S-0215 (claim 24); the search
  for absorbing buildings is claim 16 below,
- the *enter* behaviour, its repeated entry request and the standard delay: S-0215 (claims 6
  and 29),
- the enter order and its cursor (S-0214, transport verdict *x*, rows x1 to x4) and the deploy
  order that starts a release (S-0214 row B5 and table M); the order's effects on contact and
  destination (S-0213 claim 5, table T3),
- the retry of a queued entry in the actor's own turn: S-0314 (claim 23),
- the release of a controlled unit: S-0601 (claims 8 and 9),
- the spin reset of a gattling-type entrant: S-0609 (claim 11),
- the tag raise on entry: S-0524 (claim 12).

Cross-spec changes required at hand-off (the earlier ones on S-0601, S-0405 claim 7, S-0108,
S-0312 and S-0408 are made in the current drafts of those specs):
- S-0215 (Terms, *enter-nearest instruction*, and claim 24) says no actor of our engine holds an
  enter-nearest instruction. The neutral and special houses keep the original's fate decision
  (S-0601 claim 8), so their units do hold it (claim 16 below), and claim 24's repeated search
  applies to them.
- S-0601 claim 8 names the fate "go to the nearest bio reactor"; claim 16 below gives what it
  does.
- S-0312 claim 24 puts an infantry passenger of a destroyed building "on a free standing
  position": claim 14 below gives the choice (centre class, one draw).
- S-0214's passenger test of rows x, q and I13 applies to absorbing buildings as to transports
  (claim 1 below).
- S-0314 claim 23 gives a damaged vehicle on a repair depot holding contact a queued entry and
  makes it remember the depot, so that vehicle has no walk. Claim 4 below gives the walk to a free
  cell next to the building for an actor that waits in line with no remembered entry object (the
  waiting line still names the building; infantry, and a vehicle on a vehicle absorber).
- S-0405 claim 7 and its Open questions: removing a passenger inside does not make power due
  (claim 15 below).
- S-0215 claim 24's "already bound" test (for *enter*: in contact with that building) is widened by
  claim 16's *Found* step to a unit already waiting in line for it. S-0215 claim 24 states the same
  widening; its confidence there stays low (Q-1154 question 2, scenario B21 (d)).
- S-0212 (claim 25 and table T5): the waiting-line approach of claim 4 is a single-cell user of the
  nearby-cell search, with the settings of the T5 row for the waiting absorber approach, and the tick *N*
  is the tick of the *enter* processing (S-0212 lists that tick). The approach to the building uses the
  same path computation as other moves, so it can fail where S-0212's short-line case (Q-1280) makes
  that computation fail: a short line of buildings across the route with open ground round it.
- S-0214's rows x and q, and row I13, are read here as "above 0" and "equal to 0" respectively. S-0214
  uses one phrase, "carries passengers" (rows x and q) and its negation "carries no passengers" (I13),
  and does not give a number. Confirm the reading.

## Terms

- **Absorbing building, infantry absorber, vehicle absorber.** An *absorbing building* (glossary)
  is a building whose type has the infantry-absorb flag, the vehicle-absorb flag or both (Import
  mapping). An *infantry absorber* has the infantry-absorb flag, a *vehicle absorber* the
  vehicle-absorb flag.
- **Passenger (of an absorbing building).** A unit inside an absorbing building: S-0312's
  *absorbed passengers* (Terms). It is off the map and out of play (glossary, *in play*), keeps its
  own owner and is neither destroyed nor removed.
- **Passenger count** *n*. The number of passengers the building holds: S-0405's *n*ₐ.
- **Passenger order.** The order in which a building holds its passengers. Each entrant goes to
  the *front*, so the front passenger is the one that went in most recently. Every release takes
  passengers from the front.
- **Capacity** *c*. An absorbing type's imported passenger count, an integer (Import mapping).
- **Size, size limit.** S-0507's terms: every infantry and vehicle type has a floating-point
  *size*; an absorbing type has a floating-point *size limit*.
- **Link places.** The number of contacts the building can hold at once: the type's imported
  dock count, but at least 1 (Formulas F2).
- **Waiting in line.** An actor's standing intention to enter one building when a link place is
  free. S-0314 (Terms, *queued entry*) gives the same thing for repair depots; this spec uses it
  for absorbing buildings (claim 4).
- **Remembered entry object.** The object an actor remembers as the one it is entering: for a
  damaged vehicle waiting for a repair depot, the depot (S-0314 claim 23). An enter order on an
  absorbing building gives none, and setting up a wait removes any (claim 4). Not the building in
  the waiting line, which the waiting actor still names (S-0213 table T3 part (b)).
- **Acceptance test.** The building's docking answer (S-0214) to a would-be passenger (claim 1).
- **Voluntary release.** The owner's order that lets passengers out of a standing absorbing
  building (claims 11 to 13).
- **Absorber list.** A house's list of its absorbing buildings in play, in the order in which
  they were placed on the map or came to the house (a building that changes owner goes to the end
  of the new owner's list). Removing a building is assumed to keep the others' order; scenario A13,
  variant (f), tests it. Claim 16 reads the list.
- **Contact, enter-nearest instruction.** As in S-0215 (Terms). An actor in contact with a
  building holds one of its link places (claim 3, F2).
- **Search terms.** *Reach region*, *deck mode*, *preferred cell*, *height condition*, *further test*
  and *no-overlay condition* are used here as S-0212 (Terms and claim 25) defines them. None is a glossary
  term, and this spec does not restate them.

## Behaviour

### Who may enter

1. **Acceptance test.** An absorbing building *refuses* a would-be entrant *U* when any of these
   holds:
   - the building's owner does not count *U*'s house as allied (one-way, as in S-0507 Terms),
   - the building is under construction, being sold or has not finished its build-up,
   - exactly one of the building and *U* is naval (S-0504), unless *U*'s movement zone is
     amphibious,
   - *U*'s type hovers by balloon,
   - the building is switched off (glossary). Low power does not refuse,
   - *U* is a vehicle and the building is not a vehicle absorber, or *U* is infantry and the
     building is not an infantry absorber. An aircraft is not refused by its kind,
   - *U* is a controller (S-0601 claim 1) that holds at least one victim.

   Otherwise it *accepts* *U* exactly when the capacity test F1 holds. When F1 fails, the answer
   is the one the building's other features give (for example a building that also grinds),
   which is *refuse* for a building with no other entry feature.

   *U* being controlled itself does not refuse it. The building does not ask whether a link place
   is free here (claim 3 does that). This is the docking answer that S-0214's rows x3 and x4 read
   for an enter order on an absorbing building, and the permission S-0215 claim 24's search
   asks.

   S-0214's rows x3 and x4 (and the vehicle rows that read the same answer) are reached for an
   absorbing building as for a transport, by the test that rows x and q make of whether the clicked
   unit or building carries passengers (S-0214, rows x and q). For an absorbing building that test
   reads the imported passenger count as above 0. An absorbing building whose capacity *c* is 1 or
   more is therefore answered by claim 1's result (enter, or no-enter when refused). One with
   capacity 0 or below does not reach rows x3 and x4 at all, even though claim 1 is the answer it
   would give. Row I13 tests "carries no passengers" as equality with 0, so a negative capacity
   does not meet I13 (cross-spec item 4).
   - Backing: `measurable` (scenarios A1 and A12; scenario A1b for the click rows). Aircraft
     kind is only a docking-answer claim, not a promise that an ordinary aircraft order can
     complete absorption.
   - Confidence: high

2. **Capacity counts passengers, not sizes.** F1 compares the number of passengers, not the sum
   of their sizes, with the capacity, and compares the entrant's own size with the size limit.
   So a building with capacity *c* takes at most *c* passengers through this test whatever their
   sizes, and refuses any entrant whose size is above the limit even when it is empty. The size
   comparison is made in double-precision floating point.
   - Backing: `measurable` (scenario A1)
   - Confidence: high

### Getting contact

3. **Contact or waiting in line.** The rule below runs at every destination assignment that names
   an absorbing building to a mobile actor that holds no contact and has *enter* as its current
   or its queued behaviour. A queued *enter* counts. It runs in three places: in the destination
   assignment of an enter order (S-0213 table T3: part (d) has already queued *enter* when part
   (f) assigns the destination, so claim 3 comes after the order's other effects), in the fate of
   claim 16, and in the retry of claim 4.
   - If the building already holds any contact, the actor starts *waiting in line* for that
     building, whatever the number of link places. Its destination is cleared. It is not told to
     walk yet (claim 4).
   - A vehicle ordered in to a vehicle absorber always starts *waiting in line* for it, whether or
     not the building holds contact: it is given no destination and no contact at the order. It
     has no remembered entry object (claim 4); the waiting line still names the building, as
     S-0213 table T3 part (b) gives it. Its first enter processing then finds a cell next to the
     building (claim 4's approach), and its retry runs in the same turn, after its movement (claim
     4). If the place is free, the retry grants contact and the vehicle goes in, so at an empty
     building the walk it makes is at most its movement in that turn (or none, if the retry comes
     before *enter* is current; see Flag F1). It rests next to the building only while the place
     is held by another contact (claim 4). Vehicle orders at other buildings are owned by S-0314.
     This vehicle rule is `measurable` in scenario A2c's vehicle variant; confidence medium.
   - Otherwise it asks the building for contact. The building grants it when it counts the actor's
     house as allied, the actor's house also counts the building's house as allied, the
     building's health is above 0 and a link place is free.
     - *Granted:* the actor asks to load. For an otherwise ordinary absorbing building, switching
       it off refuses loading. The actor then gives up the contact (the link place is free again)
       and the destination assignment goes on as usual, so the actor holds the building as its
       destination without contact (claim 5). Combined building features may add their own
       loading restrictions; those combinations are not specified here.
     - *Not granted* (an alliance missing in either direction, health 0, no free place): the
       actor does not wait in line. The destination assignment goes on as usual, so the actor
       holds the building as its destination without contact (claim 5).

   With contact, the actor walks to the building under S-0215 claim 29.
   - Backing: `measurable` (scenario A2)
   - Confidence: high (waiting in line, switch); medium (the grant conditions other than a free
     place; the three entry points)

4. **Waiting in line.** An actor waiting in line for a building is in one of two phases.
   - *Approach.* The actor's first processing of *enter* (S-0215 claim 29) after the wait began,
     while it holds no contact and remembers no object it is entering, finds the building still
     in the game. The search is S-0212 claim 25's nearby-cell search, with the settings that its table
     T5 row for the waiting absorber approach gives. Its centre is the cell of the building's reference
     point (the centre of its foundation, S-0204 claim 4), not its top-left cell and not S-0507
     claim 10's exit ring. The actor is judged by its own locomotion class, with *winged* counted as
     *track*, and the movement zone is its type's own, not *normal* and not simplified. The cell test
     uses the reach region of the centre cell for that zone. When the actor is on a bridge deck, that
     reach region is read for the deck and the search runs in deck mode. No height condition, no further
     test and no no-overlay condition applies, and bridge cells may serve. There is no reference cell, so
     the result is the preferred cell at zero-based position *N* mod *n* (S-0212 claim 25), in exact
     integer arithmetic, where *N* is the tick number of the *enter* processing that makes the search and
     *n* is the preferred-sequence length, a centre collected twice taking two positions. The cells of the
     building itself, and every cell whose occupancy record is not empty (a unit, a building, or a standing
     position taken by another actor, S-0212 claim 25), never qualify. The search takes no random draw.

     If a cell is found, the actor assigns it as its destination, queues *move* and starts it at
     once. If none is found (no cell qualifies within the search radius), the actor falls back to
     rest in that same processing, as in claim 5 (its destination was cleared when the wait began,
     so nothing is kept): it stays where it is. If the path to the found cell cannot be made (S-0212
     claim 23: the unit does not move and its destination is cleared), the actor is in the same
     position as when none is found and does the same. The wait itself goes on: the building is still
     remembered, the retry below still runs in each of the actor's turns, and the approach is not
     tried again, because no further *enter* processing follows while the wait lasts.
     So a unit waiting in line walks to a cell next to the building, then rests there by S-0215
     claims 13 and 14. The waiting itself does not end.

     Setting up the wait clears any object the actor remembered entering, so an actor that waits
     in line for an absorbing building never has a remembered entry object and always has the
     approach. A damaged vehicle waiting for a repair depot is the contrast: S-0314 claim 23 makes
     it remember the depot, so it has no approach and stays where it is.
   - *Retry.* In each of its own turns, after its behaviour processing and its movement, an actor
     waiting in line for a building that is still in the game asks for contact (the turn position
     is S-0314 claim 23's; nothing is retried while the actor is still phasing in or otherwise
     immobilised in the way that claim describes):
     - granted: it asks the acceptance test (claim 1). On *accept* it queues *enter* with the
       building as its destination, asking for an immediate start (S-0215 claim 7, so *enter*
       becomes current at once when the actor is ready and otherwise at its next ready moment),
       and stops waiting. On *refuse* it gives up the contact, queues the none behaviour (no
       immediate start, so its current behaviour goes on until it ends), clears its destination
       and stops waiting,
     - not granted (no free link place, or any other reason): it keeps waiting.

   A building that has left the game ends the wait. Any new order ends it too (S-0213, table T3,
   part (b)). So entrants go in one at a time through a building with one link place, and the
   capacity is checked again for every entrant that had to wait. The first entrant's capacity was
   checked only by the order's docking answer (claim 1, S-0214).
   - Backing: `measurable` (scenario A2, including an alliance ending during the wait). The
     approach phase and the retry's start moment are `measurable` in scenario A2c. The path-failure
     outcome of the approach is `unmeasured`: no scenario here blocks the route to a found cell.
   - Confidence: high (re-check, order, retry position); medium (the approach phase, which rests
     on the actor having no remembered entry object; waiting after an owner change; the settings
     and tie choice of the search, which the original shares with the harvester step-off
     (S-0212 table T5), and are not yet separately measured; the no-result outcome)

5. **Refused loading and missing contact at an *enter* processing.** At each processing of
   *enter* (S-0215 claim 29) an actor that holds contact, or remembers an object it is entering,
   asks to load again. A building switched off since then refuses. On refusal:
   - the actor gives up its contact (the link place is free at once),
   - the actor falls back to rest in that same processing (S-0215 claims 13 and 14),
   - its destination and its walk are **not** cleared. With the building still its destination,
     the rest table queues *move* without a start (S-0215 claim 14, row 4). The actor goes on
     walking towards the building. It cannot go in: the completion of entry is asked of the
     actor's contact (claim 6) and it holds none. So it ends its walk next to or at the building
     without going in, and its further behaviour is the ordinary rest chain. Where exactly its
     walk ends, and whether *move* or the per-tick start takes over before the walk ends, are
     not established (S-0215 claims 8 and 11).

   An actor that holds no contact and remembers no entry object (an enter order on an absorbing
   building does not make an actor remember one; a vehicle ordered onto a repair depot that holds
   contact does, S-0314 claim 23) at a processing of *enter*, and is not waiting in line
   (claim 4), falls back to rest at once, in the same way: this is what
   follows a destination assignment whose contact was not granted or whose loading was refused
   at the order (claim 3). It too keeps the building as its destination and never goes in.

   Other interruptions are owned elsewhere: a new order (S-0213, table T3), the building's
   destruction (S-0312 claim 17), its sale (S-0408 claim 6) and its change of owner (S-0509
   claims 12 and 14).
   - Backing: `measurable` (scenario A6)
   - Confidence: high (contact given up, no entry without contact); medium (the rest chain
     after the walk)

### Going inside

6. **When an infantry unit goes in.** An infantry unit tries to go in when it arrives at the end of
   a walking step (S-0208 claim 22) in a cell that holds the building, while its current behaviour
   is *enter* and the building is its destination, or its target, or the building in its
   destination cell. Unlike S-0508 claim 2, the target counts here. At that moment the building's
   tag receives one entry raise with the entrant as causing object (S-0524 claim 12), while the
   entrant is still outside. The raise is not conditional on contact. Then the entrant asks its
   **contact** to complete its entry. An entrant that holds no contact gets no answer and does
   not go in, however it came to hold none (claims 3 and 5): it can raise the tag at an arrival
   and stay outside. The infantry absorber agrees unless it is being sold. It does not test
   capacity, size or the switch again (Edge cases, `quirks.absorber_contact_no_capacity_recheck`).

   When the building does not agree, or the entrant has no contact, nothing more happens at this
   arrival.
   - Backing: `measurable` (scenarios A3, A6 and A7) for ordinary destination entry, capacity and
     the missing-contact case. `unmeasured` for target-only and destination-cell entry, and completion during a sale:
     no fixture here establishes those arrivals with contact retained. Size and switch changes
     between the last loading request and completion are also `unmeasured`: A6 tests the earlier
     loading refusal, not that interval.
   - Confidence: high (agreement without re-check; no entry without contact); medium (the target
     and destination-cell cases; what follows a refusal)

7. **The effects of going in.** An accepted infantry entry has these same-tick results:
   - the entrant's spin value, stage and elapsed-tick count become 0 (S-0609 claim 11),
   - if the entrant is controlled, its controller releases it by S-0601 claim 9: control ring
     removed, release sound, owner back to its original owner, and the original owner's fate
     decision (S-0601 claim 8). It goes in as an object of its original owner,
   - the entry sound plays at the entrant's position (Import mapping; none when not set),
   - the entrant leaves the map and its destination is cleared. Its contact ends, so the
     building's link place is free for the next actor waiting in line (claim 4),
   - it stops counting towards its owner's crate infantry count (S-0520 Terms),
   - when the building type's extra output is above 0, the building's owner's power becomes due
     (S-0405 claims 1 and 7). The building's owner, not the passenger's, gains the power,
   - it becomes the front passenger, and *n* goes up by 1.
   The control-release sound precedes the entry sound, and the entrant is already back under
   its original owner when it disappears. The tag raise of claim 6 sees it outside and still
   controlled. No other ordering among the listed results is required.
   - Backing: `measurable` (scenarios A3 and A4)
   - Confidence: high

8. **Vehicle absorbers.** A vehicle tries to go in when it reaches a cell of a vehicle absorber
   (S-0209 row VP5) with the *enter* behaviour and that building as its destination. Unlike
   infantry, it asks the full acceptance test again (claim 1), so capacity, size and the switch
   are re-checked.
   - On *accept*: it forgets the object it remembers entering (S-0215 claim 29), leaves any
     bridge deck, its spin is reset (S-0609 claim
     11), a controller releases it as in claim 7, it leaves the map and becomes the front
     passenger, and power becomes due as in claim 7. No entry sound plays and the crate infantry
     count does not change.
   - On *refuse*: its destination is cleared, it queues *guard* and it receives a forced scatter
     request (S-0208 claims 11 to 13).

   Whether any imported building type is a vehicle absorber is a question for the imported data. Claim 8 is tested on test data (scenario A8).
   - Backing: `measurable` (scenario A8, test data) for capacity re-check, boarding, passenger
     order, silent entry and refusal. `unmeasured` for bridge status, spin reset and controlled
     vehicle release on this path: no vehicle fixture here isolates those effects.
   - Confidence: medium

### Inside

9. **Passengers inside.** A passenger stays inside until a release (claims 11 to 13, S-0312 claim
   24, S-0408 claim 8), or removal under another rule (claim 15). It is off the map and out of
   play: ordinary battlefield attacks cannot target it and it takes no ordinary unit turns or
   battlefield orders. For an uncontrolled entrant, absorption itself does not reset health or
   experience. A controlled entrant first undergoes claim 7's release, including the ordinary
   ownership-change effects; absorption adds no further reset. Its owner is then kept inside. The building's passenger
   count gives its power (S-0405 claim 1), its pips (S-0111 claim 12) and its animation (S-0108
   claim 17).
   - Backing: `measurable` (scenarios A4 and A5) for ownership, out-of-play behaviour and
     unchanged entry health and experience. `unmeasured` for whether an exceptional rule can
     heal a passenger inside (Open questions).
   - Confidence: high (off the map, owner); medium (health and experience kept)

10. **Owners.** A passenger can belong to any house the building's owner counts as allied, or, after
    claim 7's release, to the house that owned it before it was controlled, which may be an enemy.
    When the building changes owner (S-0509 claim 12), its passengers do not: they stay inside as
    they were, and the power they give follows the building to its new owner (S-0405 claim 7).
    - Backing: `measurable` (scenarios A4 and A9)
    - Confidence: high

### Coming out while the building stands

11. **Voluntary release.** The building owner's deploy order on an absorbing building that holds
    passengers gives it the *unload* behaviour (S-0214 row B5, table M). At each processing of
    *unload*:
    - garrison occupants, if any, are put out first (S-0508), in the same processing,
    - if the building holds a passenger, the front passenger tries to come out (claim 12),
    - a processing that began with passengers ends with the standard delay (S-0215 claim 6),
      whether placement succeeded or failed.

    For an ordinary absorber with no other unload feature, an empty processing queues *guard*
    and chooses 1 tick, without a standard-delay draw. So passengers come out one per processing,
    most recent first.
    - Backing: `measurable` (scenario A5)
    - Confidence: high

12. **Where a passenger comes out.** The front passenger *P* looks for an exit by S-0507 claim 10's
    exit search, with the rear direction taken from the building's facing (S-0406 claim 16,
    S-0507 F3) and with these differences:
    - the neighbouring cells are those of the building's top-left foundation cell (Formulas F3),
      not of the whole foundation. For ordinary destination entry, directions into a blocking
      foundation do not serve, so passengers come out round that one corner,
    - a neighbouring cell with a bridge deck is passed over, as in S-0507. The deck is looked at
      only for a direction that would otherwise serve, that is, whose neighbour and far cells both
      pass the cell tests. When that direction is the last of the first round, the search ends
      without a second round (Edge cases, `quirks.absorber_release_bridge_ends_search`). A deck
      neighbour that fails a cell test at the last direction is not passed over by this rule: it
      takes the ordinary advance, so the second round runs,
    - the second round skips the rear direction (Edge cases,
      `quirks.absorber_release_second_round_skips_rear`),
    - placement at the first serving direction ends the search whether or not it succeeds.

    *P* is put on the map in the serving neighbour cell, facing the exit direction (32 facing
    units of a 256-step circle per step direction): an infantry unit on a free standing position
    of that cell (S-0208 claim 3, desired point the cell centre), a vehicle at a nearby location
    where it can stand, searched from that cell.
    Ordinary infantry destination entry clears the destination while inside. This alone does
    not establish whether a target-only entrant retains a mission target that opens the building's
    own cells under S-0209 claim 4; that exceptional case remains open.
    - Backing: `measurable` (scenario A5) for the corner, the order, the rounds and the facing.
      Scenario A5b covers the bridge-deck end, which needs a bridge beside the building's top-left
      cell. `unmeasured` for a failed placement after a serving direction.
    - Confidence: high (rounds, deck check); medium (the corner, which rests on the building's
      own cells answering *blocked*; failed placement)

13. **After the attempt.**
    - **On success:** *P* is no longer a passenger and *n* goes down by 1. At an infantry absorber
      the exit sound plays at the building (Import mapping), and an uncounted *P* counts again
      towards its owner's crate infantry count. A vehicle-only absorber does not perform that
      infantry-count adjustment. *P* queues *move* to the far cell of the exit direction (first round)
      or to the neighbour cell (second round). Power becomes due as in claim 7. In the original a
      passenger of a human-controlled house also drops an enter-nearest instruction (S-0215 Terms).
      In our engine no player house's unit holds one. A passenger of the neutral or special house
      keeps any instruction it holds (claim 16, Edge cases,
      `quirks.enter_nearest_survives_absorption`).
    - **On failure:** *P* goes back to the front and stays inside. The next processing tries *P*
      again, so a passenger that finds no exit blocks the others.
    - Backing: `measurable` (scenario A5)
    - Confidence: high

### Other ways out

14. **Destruction, sale and erasure.** A destroyed building releases or kills its passengers by
    S-0312 claim 24, and a sold one releases them by S-0408 claim 8. In both cases they come out
    from the front, so the passenger that went in most recently takes the first foundation cell.
    A passenger that comes out this way counts again towards its owner's crate infantry count and
    keeps its owner. An infantry passenger of a destroyed building is placed in its foundation cell
    by the centre placement of S-0208 claim 3: its aim point is 36 leptons from the cell's centre
    along the cell's second axis, which S-0208 claim 2 counts as the centre. Each infantry passenger
    whose cell is open makes one draw over 0 to 3, then takes the first free position of the chosen
    row of S-0208 table O. This is the sale's choice (S-0408 claim 8). A cell that refuses the
    infantry unit (a vehicle in it, or the object flags that S-0208 claim 3's first row names)
    makes no draw. A vehicle passenger comes out
    at the cell centre. An erased building deletes its passengers (S-0514 claim 12).
    - Backing: `measurable` (scenario A10) for release order, owner, crate count and the draw on
      destruction and sale; erasure inherits S-0514 claim 12's backing.
    - Confidence: high (order); medium (crate count, the draw)

15. **What changes the passenger count.** *n* goes up only when an entrant goes in (claims 7 and
    8). It goes down when a passenger comes out by voluntary release (claim 13), by the building's
    destruction or sale (claim 14), or when a passenger leaves the game by another rule while it is
    inside: for example the defeat of the passenger's own house destroys it inside (S-0414 claim
    5), which can happen to a formerly controlled enemy (claim 10). The building's switch, power,
    nonlethal damage and ownership change do not by themselves change *n*.

    **Removal inside does not make the building owner's power due, except possibly in the
    same-house case below.** When a passenger leaves the game inside, the building's passenger count
    drops at once, and nothing marks the building owner's power due. The totals are worked out from
    the live count only when something else makes them due (the events of S-0405 claim 7). Until then
    they still include the removed passenger's contribution (S-0405 claim 1), and the owner's output
    is too high. Where the removed passenger's own loss is registered at all, the registration makes
    its own house's power due. That is the building owner's power only when the two are the same
    house, and whether the registration happens in that case is not established (Edge cases,
    `quirks.absorber_removal_keeps_stale_power`; Open questions).
    - Backing: `measurable` (scenarios A5, A9, A10 and A11) for the count changes and for the
      stale totals.
    - Confidence: high (entry and releases); medium (defeat removes the passenger; no due mark at
      removal, which rests on the path by which a passenger leaves the game, not on a trace of a
      whole defeat)

16. **The bio-reactor fate of a taken or released unit.** The fate decision of S-0601 claim 8 can
    choose *enter the nearest absorbing building* for a unit that has just been taken or
    released, when its new owner is not human-controlled. In our engine only the neutral and
    special houses are such owners. Choosing it does the following, once, at the moment of the
    fate:
    - *Candidates.* The owner's absorbing buildings in the order of its absorber list (Terms).
      Each is asked claim 1's acceptance test for this unit, so alliance, build state, kind,
      switch, capacity and size all count. Whether a link place is free does not: a building that
      holds another contact can win.
    - *Distance and tie-break.* The distance is ρ₃ of S-0215 F4, a whole number, measured from the
      unit's own position to the building's reference point, the centre of its foundation (S-0204
      claim 4). Its exact form is governed by S-0215's `quirks.table_root_distance`. The smallest
      value wins. Where values are equal, the building later in the absorber list wins, that is,
      the one placed on the map or gained by the owner most recently.
    - *Found.* The unit takes the enter-nearest instruction for absorbing buildings (S-0215
      Terms). Unless it already holds contact with that building or already waits in line for it,
      it queues *enter* without a start and assigns the building as its destination, which
      applies claim 3 at once (contact, or waiting in line). The fate succeeds.
    - *None found.* The instruction is cleared. The fate fails and becomes *hunt* (S-0601 claim
      8), queued without a start.

    After that the unit goes in under claims 3 to 7 and by no other path. Nothing searches again
    while the unit is in *enter*: a building that fills up or is destroyed on the way is dealt
    with by claims 4 to 7 and the destruction, sale and owner-change specs. The instruction,
    however, stays with the unit: whenever it is in *guard* or *area guard*, S-0215 claim 24 runs
    the same search again at each processing, and ends the instruction when no building is found.
    A neutral or special house unit that falls back to rest in *guard* therefore looks for a
    building again after a refusal, and finds the same one if it still accepts the unit. Entry
    does not clear the instruction, and a voluntary release clears it only for a human-controlled
    owner (claim 13). So a passenger of a neutral or special house that comes out of a building
    walks back in at its next *guard* processing when some building of its owner accepts it
    (Edge cases, `quirks.enter_nearest_survives_absorption`).

    A victim that is a vehicle has only vehicle absorbers among its candidates, so the search
    finds a building only where an imported building type is a vehicle absorber. The decision's draw, its fate tables and its order of fates are
    S-0601 claim 8's.
    - Backing: `measurable` (scenario A13, including variant (f)). The order of equal-distance
      candidates after a building has been removed from the list is measured by variant (f).
    - Confidence: high (one search, candidates, distance, none found); medium (tie-break by list
      order, the repeat through the guard processing); low (any other clearing of the
      instruction, such as a scripted order for a neutral unit)

17. **What ends an entrant's contact.** An entrant's contact with an absorbing building ends in
    these cases and no others that are known:
    - a refused loading request at an *enter* processing or at the order (claims 3 and 5), or a
      refused acceptance test in a retry (claim 4),
    - the entrant goes in (claim 7),
    - a new behaviour order that breaks contact (S-0213, table T3, part (b)),
    - the entrant leaves the map or is removed from the game,
    - the building's destruction, sale, removal or change of owner (S-0312 claim 17, S-0408 claim
      6, S-0509 claims 12 and 14).

    Falling back to rest, the start of any other behaviour, and the loss of *enter* as the current
    or queued behaviour do not end it. An entrant that leaves *enter* while it holds contact
    keeps its link place until one of the cases above happens. With one link place, every later
    entrant then waits in line (claim 3) and its retries are refused for want of a place (claim
    4), for good. In the original this can be reached by any rule that takes an actor out of
    *enter* with its contact intact; claim 5's refusal is the one that gives the contact up first.
    The *enter* processing itself takes no approach step (S-0215 claim 37 (a) lists the
    processings that do), and an entrant that holds the building as its destination keeps it
    while it has contact (S-0215 claim 29 and S-0207 claim 26 for the rest).
    The window in which a waiting actor has been granted contact and has *enter* queued with an
    immediate start but not yet started is part of the rule: the contact is kept.
    - Backing: `measurable` (scenario A14). `unmeasured` as a statement that no other ending
      exists: the search covered the other order-handling and the answers to contact requests, not
      every update.
    - Confidence: medium

## Formulas

**F1. Capacity test.** With *n* the current passenger count, *c* the capacity, *s* the entrant's
size and *L* the building type's size limit:

> accept ⇔ *n* + 1 ≤ *c* and *s* ≤ *L*.

*n* and *c* are integers. *s* and *L* are double-precision floating-point values compared
directly, without rounding.

**F2. Link places.**

> places = max(*d*, 1)

where *d* is the type's imported dock count, an integer. Exact integer comparison; no rounding.

**F3. Exit cells.** Let (*x*₀, *y*₀) be the building's top-left foundation cell and (Δ*x*, Δ*y*) the
cell step of step direction *e* (glossary, *step direction*), with *e* = (*q* + *k*) mod 8 for the
rear direction *q* of S-0507 F3 computed from the building's facing. The neighbour cell is
(*x*₀ + Δ*x*, *y*₀ + Δ*y*) and the far cell (*x*₀ + 2Δ*x*, *y*₀ + 2Δ*y*). Exit facing = 32 × *e* in
256-step facing units. Exact integer arithmetic, with non-negative remainder for modulo and no
rounding in additions or multiplication. The rear-direction rounding is S-0507 F3's.

For the usual facing 0 (S-0406 claim 16), *q* = 4. On open ground around an ordinary absorber
with a blocking foundation at least 2 × 2, direction 4 points into the foundation, so the first round's first
serving direction is 5: the passenger appears in (*x*₀ − 1, *y*₀ + 1) and walks to
(*x*₀ − 2, *y*₀ + 2).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Minimum link places | 1 | contacts | F2 |
| Facing per step direction | 32 | 256-step facing units | F3, claim 12 |
| Exit rounds | 8 then 7 | directions | Claim 12, as S-0507 |
| Capacity added per entrant | 1 | passengers | F1; size does not count |
| Centre displacement of an infantry placement (destroyed or sold building) | 36 | leptons | claim 14, along the cell's second axis |
| Placement draw range | 0 to 3 | draw values, one draw per open cell | claim 14, as S-0408 claim 8 |

## State and ordering

**Table E: an entrant** (claims 3 to 8, 17).

| State | Event | Result |
|---|---|---|
| *enter* (current or queued), no contact | destination assigned: an absorbing building that holds a contact | waiting in line, destination cleared |
| *enter* (current or queued), no contact | destination assigned: a building with no contact, grant, loading accepted, and not a vehicle ordered to a vehicle absorber (claim 3) | in contact, walks under S-0215 claim 29 |
| *enter* (current or queued), no contact | vehicle ordered in to a vehicle absorber (claim 3), whatever the building's contact state | waiting in line, no destination, no remembered entry object; approach and retry as in claim 4 |
| same | grant, loading refused (switched off) | contact given up; destination kept; no contact |
| same | contact not granted | no waiting; destination kept; no contact |
| *enter* current, no contact, no remembered entry object, not waiting | processing | falls back to rest at once, destination kept, *move* queued; never goes in |
| waiting in line | first *enter* processing, cell found | free cell near the building as destination, *move* queued and started |
| waiting in line | first *enter* processing, no cell found | falls back to rest in that processing; stays where it is; still waiting, and the retry goes on |
| waiting in line | own turn, contact granted, acceptance test accepts | *enter* queued with immediate start, destination = the building, waiting ends |
| waiting in line | own turn, contact granted, acceptance test refuses | contact given up, none queued, destination cleared, waiting ends |
| waiting in line | own turn, contact not granted | keeps waiting, asks again next turn |
| waiting in line | building leaves the game, or a new order | waiting ends |
| *enter*, in contact | processing, building switched off | contact given up, rest in the same processing, destination kept, keeps walking, never goes in |
| *enter*, infantry | arrives in a cell of the building (claim 6), in contact | tag raise, then completion asked of the contact; not being sold: goes in (claim 7) |
| *enter*, infantry | arrives in a cell of the building, no contact | tag raise only; nothing else |
| *enter*, vehicle | reaches a cell of a vehicle absorber (claim 8) | acceptance test; accept: goes in; refuse: destination cleared, *guard*, scatter |
| in contact, not in *enter* | rest, other behaviour starts | contact kept (claim 17, quirk) |

**Table R: an absorbing building in *unload*** (claims 11 to 13).

| State | Event | Result |
|---|---|---|
| holds passengers | processing | occupants out first; front passenger tries to exit; standard delay |
| front passenger | exit found and placed | passenger out, *move* queued, exit sound, power due |
| front passenger | no exit, or placement failed | passenger back at the front |
| no passengers, ordinary absorber | processing | *guard* queued; 1 tick, no delay draw |

Ordering consequences:
- An entry's tag raise comes before the entrant leaves the map and before its controller releases
  it, so its effects see the entrant outside and still controlled.
- Power changes from an entry or a release take effect at the owner's next house update (S-0405
  claim 7). An entry during unit turns can therefore change the totals later in that same tick.
- A passenger is out of the passenger count in the processing in which it comes out. Pips and
  animation use that changed count when their respective rules next evaluate it (S-0111 claim 12,
  S-0108 claim 17); this does not force an animation update earlier in the tick.

The fate of claim 16 is part of the release or takeover that causes it: the search, the queueing
and claim 3's contact request all take place in the tick of that release.

Random draws: an *unload* processing that begins with passengers draws once for its standard
delay (S-0215 claim 6); the ordinary empty processing chooses 1 tick without that draw. Each
infantry placement draws as S-0208 claim 3 gives, in a destruction or sale too (claim 14: one
draw per infantry passenger whose cell is open). The search of claim 16 draws nothing. A controlled entrant whose original owner is
computer-controlled (only the neutral and special houses in our engine) consumes S-0601 claim 8's
draw. An entrant that gives up its contact and rests draws as rest does (S-0215 claim 6).

## Edge cases and quirks

- **No capacity re-check at contact (`quirks.absorber_contact_no_capacity_recheck`, default on).**
  An infantry entrant that reaches the building while it holds contact goes in whatever the passenger
  count, the switch or its size (claim 6). An entrant without contact does not go in (claim 6). With one link place this cannot overfill the building, because the next
  entrant waits in line and passes claim 1 only after the previous one is inside. With two or
  more link places, several entrants can each pass claim 1 against the same count while the others
  are still walking, and all go in: the building then holds more than its capacity. The pips then
  follow S-0111 claim 12's overflow rule, and each extra passenger adds power. *Fixed behaviour:*
  at completion the building applies the acceptance test of claim 1 again, and a refused entrant
  is treated as a refused vehicle (claim 8).
- **Rear direction skipped in the second round
  (`quirks.absorber_release_second_round_skips_rear`, default on).** As S-0507's
  `quirks.unload_second_round_skips_rear`, but for absorbing buildings: when no direction has two
  open cells, the rear neighbour is not tried. *Fixed behaviour:* the second round tries the rear
  direction first.
- **A bridge deck at the last direction ends the search
  (`quirks.absorber_release_bridge_ends_search`, default on).** When the eighth direction of the
  first round passes both cell tests (neighbour and far cell) but its neighbour cell carries a
  bridge deck, the second round never starts and the passenger stays inside this processing. A
  deck neighbour at that direction that fails a cell test does not end the search: the second
  round runs. *Fixed behaviour:* the second round
  starts as it does after any other failure at the eighth direction.
- **Contact outlives *enter* (`quirks.absorber_stale_contact_kept`, default on).** An actor
  that stops being in *enter* with its contact intact keeps the building's link place for good
  (claim 17), and every later entrant waits in line for ever. *Fixed behaviour:* in its own turn,
  after its movement (the place of claim 4's retry), an actor in contact with an absorbing
  building that has neither *enter* current nor *enter* queued gives the contact up. The window
  in which a waiting actor has been granted contact and has *enter* queued but not yet started
  keeps the contact.
- **Removal inside leaves the owner's power stale
  (`quirks.absorber_removal_keeps_stale_power`, default on).** A passenger removed while inside
  does not make the building owner's power due (claim 15). *Fixed behaviour:* the removal makes
  the building owner's power due, as a release does, only when the type's extra output is above
  0.
- **The instruction survives absorption (`quirks.enter_nearest_survives_absorption`, default
  on).** A unit of the neutral or special house that took the enter-nearest instruction through
  the fate of claim 16 keeps it when it goes in and when it is let out, so a released passenger
  walks straight back in (claims 13 and 16). *Fixed behaviour:* going in clears the instruction.
- **Exits round one corner.** Because the exit search starts from the top-left foundation cell,
  passengers of an ordinary large absorber with a blocking foundation come out on its top-left
  side, and a building whose top-left side is blocked keeps them even when its other sides are open. This is the original's
  rule; no switch is proposed (Open questions).
- **An enemy inside.** A controlled enemy unit sent into its controller's absorbing building goes
  in as its original owner's unit (claim 7). It gives power to the building's owner while inside,
  and it comes out as an enemy at a release. This follows from S-0601 claim 9 and is not a quirk.
- **Capacity zero or below.** With a non-negative passenger count, F1 never holds, so the
  absorption capacity test accepts no entrant. Another entry feature may still accept (claim 1).

## Import mapping

| Key (where) | Feeds | Form | Absent |
|---|---|---|---|
| `InfantryAbsorb` (rules, building type) | infantry absorber (Terms, claims 1, 7, 13) | boolean | no |
| `UnitAbsorb` (rules, building type) | vehicle absorber (Terms, claims 1, 8) | boolean | no |
| `Passengers` (rules, building type) | capacity *c* (F1); also the passenger count that S-0214's rows x, q and I13 test on a clicked building (claim 1) | integer | 0 |
| `SizeLimit` (rules, building type) | size limit *L* (F1) | floating point | 0.0 |
| `Size` (rules, infantry and vehicle types) | entrant size *s* (F1), as S-0507 | floating point | 1.0 |
| `NumberOfDocks` (rules, building type) | dock count *d* (F2) | integer | 1 |
| `ExtraPower` (rules, building type) | extra output; power becomes due only when above 0 (claims 7, 13) | integer | 0 |
| `EnterBioReactorSound` (rules, `[AudioVisual]`) | entry sound at an infantry absorber (claim 7) | sound name | none |
| `LeaveBioReactorSound` (rules, `[AudioVisual]`) | exit sound at an infantry absorber (claim 13) | sound name | none |
| `Naval`, `MovementZone`, `BalloonHover` (rules, unit types) | naval, amphibious and balloon rows of claim 1, as S-0504, S-0211 and S-0214 read them | boolean, movement zone, boolean | as their owners give |

The fate weights and thresholds that choose the bio-reactor fate (claim 16) are S-0601 claim 8's
Import mapping; claim 16 adds no key.

The entry and exit sounds are the rules-wide ones. The same sound keys in a type section do not
choose the sounds for these absorption events. The release sound of claim 7 is S-0601 claim 9's.
Absent values above are initial defaults. Later absent keys retain held values under S-0006
claim 3, including session-held rules-wide sounds.

## Measurement scenarios

Use a small purpose-made skirmish map, normal game speed, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Stock data
apply except named test files (data-only test variants). Where a scenario uses a stock infantry
absorber, it is a stock building type whose imported infantry-absorb flag is set; "fill it"
means bringing it to its imported capacity. Use scripted input
and a read-only recorder for ticks, unit positions, owners, health, experience, current and queued
behaviours, destinations, contacts, passenger counts, crate infantry counts, house power and
scenario random-stream state. Capture audio and images for sounds, pips and animation. Run each
variant once with seed 1 and repeat with seed 2. A contrary recorded result falsifies the stated
expectation unless the fixture failed to establish its prerequisite. Instrumentation is proposed,
not supplied here; no writes to live game state are required.

**A1. Acceptance** (claims 1 and 2). House A with an infantry absorber; house B allied, house C
enemy. Order, one at a time: an infantry of A; of B; of C; an infantry of A while the building is
switched off; while A has low power but the building is on; an infantry of A into a full
building; a vehicle of A; an infantry of a type whose size is above the building's size limit (test
type); a controller of A (S-0601) holding a victim; the same controller with no victim; an
infantry of C controlled by A. *Record:* the cursor answer and whether the unit goes in.
*Expected:* in for A, B, low power, the idle controller and the controlled unit; refused for C,
switched off, full, vehicle, oversize and the busy controller. *Falsified* by any other result.
Repeat the full case with entrants whose sizes are below 1 (test type): a full building still
refuses (count, not size). Also fill an empty variant whose passengers each have size above 1 but
at or below its enlarged size limit: exactly its capacity in passengers may enter. Test equality
with the size limit and the smallest imported value above it: equality accepts, above refuses.
For a one-way alliance, record the docking answer separately from contact: it follows the
building's view, whereas contact requires both views (A2).

**A1b. Click rows** (claim 1). Select an own infantry unit and hover, then click, an own
absorbing building whose capacity is 3, then a data-only variant with capacity 0, then one with
capacity -1 (test types). Also a healer over each. *Record:* the cursor and the order given.
*Expected:* capacity 3: the enter cursor and an enter order; capacity 0 and -1: no enter cursor
and no enter order (an ordinary click answer for a building). The healer over capacity 0 takes
S-0214 row I13's no-passenger answer, over capacity -1 it does not. *Falsified* by an enter
cursor over capacity 0 or below, or by none over capacity 3.

**A2. One at a time** (claims 3 and 4). A building with capacity 1 and exactly one free place. Select two
infantry of A and order both in the same tick. *Record:* per tick, each unit's position,
destination, current and queued behaviour and contact, and the passenger count. *Expected:* one
walks to the building with contact. The other has no destination and no contact from the order
tick, and (A2c) walks to a cell next to the building and rests; after the first goes in, the
second gets contact in its own turn, is accepted or refused by the capacity test (the building is
full: refused, so it queues none and has no destination) and does not go in. Repeat with two free
places: the second starts walking in the tick in which the first goes in or the next, by the
update order (S-0203). A second entrant arriving despite full capacity, or both walking with
contact concurrently with only one link place, falsifies these predictions.
For the separate contact test, use A and B with only the building owner counting the entrant's
house as allied: the docking answer may accept but no contact is granted. *Expected:* the unit does
not wait in line. It keeps the building as destination, walks to it, falls back to rest at its first
*enter* processing (a *move* queued) and does not go in; with a map trigger on the building the
arrival raises its tag. Restore both alliance views: contact becomes possible. While a second
entrant waits, end the alliance using ordinary orders without deleting the building: it
continues waiting when contact is refused; give it a new move order to end the wait. Also destroy
its destination while it waits: the wait ends. Record contacts, waiting intention and
destinations in all variants.

**A2c. The waiting unit's approach and the retry** (claim 4). A one-place building held by a
first entrant that is kept walking (a long path). Order a second infantry unit of A in, from a
point on the far side. *Record:* per tick, its destination, current and queued behaviour,
position, and the tick in which the first goes in and in which the second's *enter* becomes
current. *Expected:* the second walks to a free cell next to the building and rests (*guard*) there.
In the first turn after the place is free it queues *enter* with an immediate start, and *enter*
is current in that turn or the next ready moment. Staying where it was ordered until the place is
free falsifies the approach phase. *enter* starting only one or more turns after readiness
falsifies the start moment. *Search variants* (Q-1152): (i) surround the building so that exactly
one cell next to it is free, and start the waiting unit on each side in turn: it walks to that cell
whichever side it starts on (a different or farther choice falsifies the centre or settings); (ii)
leave two equally near free cells, repeat over several ordering ticks: the cell chosen varies
with the tick as *N* mod *n* of S-0212 claim 25 (always the same cell falsifies the tie rule); (iii)
block every cell within the search radius: the unit stays where it is, rests, and still goes in when
the place is free (walking, or an end of the wait, falsifies the no-result outcome). Repeat with a vehicle and a vehicle absorber: the vehicle, even for the
first entrant, gets no destination and no contact at the order (claim 3). With a free place, its
retry grants contact in its first turn, and it goes in without resting at a cell next to the
building: a vehicle that rests next to an empty building with a free place falsifies this part.
With the place held by another contact, it walks to a cell next to the building and rests there
(claim 4). A vehicle onto a repair depot is S-0314's scenario, not this one: it remembers the depot
and does not walk (S-0314 claim 23).

**A3. Entry tick** (claims 6 and 7). A map trigger "entered by" on the building. Order one infantry
in. *Record:* the tick of the trigger, of the entry sound and of the unit leaving the map; the
owner's power output in that tick and the next house update; the pips. *Expected:* trigger and
sound in the same tick as the unit leaves; the trigger sees the unit outside. Output rises by the
health-scaled passenger contribution of S-0405 claim 1 at the next house update, possibly in this
same tick; one additional passenger pip is displayed. Record the crate infantry count before and
after entry: it falls by 1 for a previously counted entrant. With a gattling test infantry that has
nonzero spin before entry, record spin, stage and elapsed count: all become 0. Failure of any
same-tick result or a total differing from S-0405 falsifies that part. In data-only variants with
extra output 0 and below 0, observe the power-due mark in an otherwise quiet house: entry adds
no mark. This isolates the condition even though settled output alone cannot.

**A4. Controlled entrant** (claims 7 and 10). B's infantry is controlled by A's controller and
ordered into A's building. *Record:* release sound, A's controller pip count, then sell the
building. *Expected:* the release sound plays and the controller's count drops in the entry tick.
After the sale the infantry comes out as B's unit. The release sound precedes the entry sound,
a tagged entrant is still controlled when its entry event is raised. *Falsified* if the unit
remains controlled after entry, if these observations are reversed, or if it comes out as A's.
Repeat with an uncontrolled allied infantry of B: its owner remains B. This replaces S-0601
MC6's bio-reactor expectation.

**A5. Voluntary release** (claims 11 to 13, 15, F3). Fill the building with named infantry entered
in a known order, then deploy it. *Record:* per tick, which passenger comes out, its cell, facing
and destination, the sounds, the pips. *Expected:* the most recently entered comes out first, in
(*x*₀ − 1, *y*₀ + 1) facing step direction 5, walking to (*x*₀ − 2, *y*₀ + 2); one passenger per
standard delay; *guard* after the last. A first passenger in (*x*₀, *y*₀ + 1), inside the
foundation, falsifies the corner rule. Then block every cell round the top-left corner (walls or
units) but leave the other sides open: no passenger comes out. Then open only direction 5's
neighbour, with its far cell blocked: the passenger comes out there in the second round and walks
only to that cell. For the rear skip, use a test absorber with a 1 × 1 foundation and facing 0:
block every cell round it except the rear neighbour (*x*₀, *y*₀ + 1), with that cell's far cell
blocked. *Expected* (switch on): nobody comes out. *Falsified* if a passenger comes out there.
For each successful release, record crate infantry count and house power: count rises by 1 for
an uncounted infantry passenger, and output changes at the next house update under S-0405 claim
1. Repeat with extra output 0 and below 0 to isolate the absence of a power-due mark. On blocked
attempts, passenger count, order and off-map status remain unchanged, and no exit sound plays.
clearing the blockers releases that same front passenger first. Record the delay draws: one
standard-delay draw per nonempty processing, none at the ordinary empty processing, which queues
*guard* and chooses 1 tick. Different cadence, order, facing, destinations, sounds or draws
falsify the corresponding part.
Use damaged, experienced uncontrolled infantry and release it immediately after entry: health
and experience are unchanged. During a longer hold, try ordinary selection and orders and fire
near its last battlefield position: it takes no ordinary turns, cannot be selected or ordered as
a battlefield unit, and is not a target at that position. Any contrary result falsifies claim 9's
ordinary out-of-play behaviour, without deciding exceptional healing. A hybrid test absorber that
also supports garrison occupants tests claim 11's precedence: place occupants and absorbed
passengers inside, deploy, and record their appearances. All occupants leave before the first
absorbed passenger in that processing; reversed appearances falsify the precedence.

**A5b. Bridge deck at the eighth direction** (claim 12, quirk). A test absorber of 1 × 1 foundation
and facing 0. One first-round direction *d* before the eighth has an open neighbour cell and a
blocked far cell, so only the second round can use it. Every other neighbour is blocked except
the eighth first-round direction, whose neighbour cell carries a bridge deck. (i) Far cell of the
eighth direction open: *expected* (switch on) nobody comes out, however many processings: the
search ends at the deck. (ii) The same with that far cell blocked: *expected* the passenger comes
out in the second round at direction *d*'s neighbour and walks only to that cell. A passenger
coming out in (i) falsifies the quirk; none in (ii) falsifies the tighter wording of claim 12.

**A6. Switched off while walking** (claims 3, 5 and 6). Order an infantry in from far away and switch
the building off while it walks. *Record:* per tick its contact, destination, current and queued
behaviour, position, the building's link use and, with a map trigger on the building, the tag raise.
*Expected:* at its next *enter* processing its contact ends (the link place is free) and it falls
back to rest; its destination stays and *move* is queued; it keeps walking and does not go in.
Falsified if it goes in, if it stops dead at that processing, or if its destination is cleared.
Repeat with the building switched off before the order is given (forced order): the contact is
refused or lost at the order, the destination is assigned, the unit walks and does not go in.
Switch the building on again while the unit walks: it still does not go in (no contact).

**A7. Over capacity** (claim 6, quirk). Test type: the infantry absorber with dock count 2. One
free place. Order two infantry in the same tick from equal distances. *Expected* (switch on): both
go in, the count exceeds the capacity. Record that both acquired contact and passed acceptance
before either arrived; otherwise the run does not test completion without re-check. *Falsified*
if a contacted second infantry is refused for capacity at arrival. Compare the ordinary one-link
case A2, which must not overfill.

**A8. Vehicle absorber** (claim 8). Test type: a building with the vehicle-absorb flag, two link
places and room for one vehicle. Order two vehicles from different distances while it is empty,
record that both have contact and accepted entry before the nearer one arrives. *Expected:* the
first goes in silently; the farther one reaches the now-full building, is refused on arrival,
loses its destination, queues *guard* and gets a forced scatter request. A second enter order
given only after the building is full is not a test of the arrival re-check: its initial docking
answer already refuses. Record sound, count, order, destination and scatter request; a second
vehicle entering or any entry sound falsifies the respective part. In a capacity-two variant,
let both enter and deploy: last in comes out first. Release of a vehicle from a vehicle-only
absorber does not change the crate infantry count.

**A9. Capture** (claims 10 and 15). A's building holds A's infantry; B's engineer captures it.
*Record:* passenger count, power of A and B, then B deploys the building. *Expected:* the count is
unchanged, the power moves to B, and the released infantry is A's. A changed passenger owner or
count falsifies this. Repeat without capture, switching the building off, restoring it, inducing
low power, and dealing nonlethal damage: none changes passenger count. Deploy after each case
to confirm the same passengers remain, rather than merely unchanged pips.

**A10. Destruction and sale order** (claim 14). Fill a crewed building with named infantry in a
known order and destroy it with a hit that permits passenger escape. Repeat with an ordinary
sale. *Record:* identity, owner and release cell of each passenger, each owner's crate
infantry count and the scenario random-stream state before and after (expected: one draw over 0 to 3
for each infantry passenger whose cell is open, none for a vehicle or a refused cell). *Expected:* the most recently entered infantry uses the first foundation cell of
S-0312 claim 24's order, later ones follow in reverse entry order, each keeps its owner, and
successfully released uncounted passengers are counted again. A different order, changed owner
or missing count restoration falsifies that part. Erasure uses S-0514 claim 12's scenario instead.

**A11. A passenger whose house is defeated** (claim 15). Put a controlled infantry of house C into
A's building (scenario A4), then defeat C. *Record:* A's passenger count and pips, and A's power
output each tick until a later event (for example a building placed) makes power due. *Expected
for claim 15:* the passenger is deleted and the count drops when defeat removes it, and A's output
keeps the removed passenger's contribution until the later event. A drop at the next house update
falsifies the stale-power part, and persistence of the passenger falsifies the count part. Repeat
with an uncontrolled infantry of A's own house taken out inside by any other means available, and
record the same things. This repeat is a discriminating observation for the same-house question
(claim 15, Open questions) and sets no expected value, because that question is open.

**A12. Remaining permission and import gates** (claim 1, Import mapping). Use named data-only
variants of the ordinary absorber and entrants, not another executable. Record docking answers
during build-up, while selling, and after completion: the first two refuse, completion accepts
an otherwise legal entrant. For naval mismatch, give only one of the pair the naval flag:
refusal; with the entrant's amphibious movement zone: acceptance. A balloon-hover entrant is
refused. A test aircraft within size and capacity limits is not refused by kind at the docking
answer; no ordinary absorption flight path is asserted. Any contrary docking answer falsifies
the respective gate. For defaults, omit one absorber key at a time and record the imported
concept or its observable consequence: flags false, capacity and size limit zero, dock count one,
extra output zero. An entrant lacking its size key uses size 1.0. For the two rules-wide sounds,
omit each sound from every rules layer in a fresh game process: the corresponding event is
silent.
Type-section sound variants alone do not change these event sounds. Differing parsed defaults
or sound selection falsify the Import mapping.

**A13. The bio-reactor fate** (claim 16). The neutral house owns two infantry absorbers *P* and
*Q* at different distances from a unit *U* of the neutral house that a controller of house B holds, with the
fate weights of S-0601 claim 8 set so that the bio-reactor fate is always chosen (test data).
Release *U* (B's controller is removed). *Record:* per tick *U*'s owner, destination, current and
queued behaviour, contact, which building it walks to, the scenario random-stream state, and the
building list order of placement. *Expected:* one draw for the decision (S-0601), none for the
search; *U* walks to the nearer building by the table root of S-0215 F4 measured to the
foundation centre, whether or not a link place is free (a building held by another contact makes
it wait in line and walk to a cell next to it). Variants: (a) two buildings at equal table-root
distance, placed in a known order, then the same with the order swapped by capturing and
returning one: *expected* the later one in the list wins in each. (b) The nearer building full or
switched off: it is not a candidate. (c) No building accepts *U*: the fate becomes *hunt*, queued.
(d) The chosen building destroyed or filled while *U* walks: *U* is not redirected while it is in
*enter*; record what it does next, and that at its next *guard* processing it searches again.
(e) After going in, deploy the building: the released passenger of the neutral house walks back
in at its next *guard* processing (switch on). A redirect while in *enter*, a draw by the search,
a nearer building ignored or a farther one chosen falsifies the respective part. (f) Three buildings
at equal distance, in a known list order: destroy the middle one of the three and send *U*. *Expected:*
the later of the two that remain wins. A different winner falsifies the tie-break after removal.

**A14. The stale contact** (claim 17). One-place building. First entrant *E1* in contact and
walking; a second, *E2*, waiting in line. Make *E1* leave *enter* without an order and without
refusal (for example an armed entrant with an earlier target that leaves *enter* by another
behaviour starting; any method that does not end the contact by claim 17's list). *Record:* each
tick *E1*'s contact, current and queued behaviour and destination, the building's link use, and
whether *E2* gets contact and goes in. *Expected* (switch on): *E1* keeps the contact, *E2* waits
and is never granted one. With the switch off (an engine setting, so this part is tested on our
engine, not on the oracle), *E1* gives up the contact in its turn after movement and *E2* goes in.
If no method leaves *enter* without ending contact, record that: it makes claim 17 a statement
about situations that nothing in the game reaches.

## Open questions

- Does anything heal a passenger inside (claim 9)? Not found; unmeasured.
- Does a removal inside make the building owner's power due when the removed passenger belonged to
  the building owner's own house (claim 15)? The registration path is not traced; scenario A11's
  repeat is the discriminator.
- Should exits round the top-left corner (Edge cases) get a switch whose fixed behaviour searches
  round the whole foundation (for example S-0508's exit ring)? Lead decision.
- What an infantry entrant does after the building refuses its completion during a sale (claim 6):
  the arrival does nothing more; whether it then keeps trying is not established.
- Does target-only entry leave a mission target that opens foundation cells on release (claim 12)?
- Exact vehicle placement-search ties and combinations with other loading or unload features
  remain unspecified.
- Where an entrant's walk ends after it lost its contact (claim 5), and whether *move* or the
  per-tick start takes over before the walk ends: S-0215 claims 8 and 11 decide, and scenario A6
  records it.
- Whether a player-house AI that issues an enter order to a unit that already holds an old target
  can reach the stale-contact situation of claim 17 by another route than the ones named there.
