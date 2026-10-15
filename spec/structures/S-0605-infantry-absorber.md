---
id: S-0605
title: Infantry absorber (who may enter an absorbing building, entry, holding, release)
topic: structures
revision: 1
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0605: Infantry absorber (who may enter an absorbing building, entry, holding, release)

## Summary

How units get into an absorbing building (glossary) and how they stay there and come out: which
units the building accepts and how its capacity is counted, how an entrant gets contact or waits
in line, what happens in the tick an entrant goes inside, how passengers are held and who owns
them, the owner's order that lets them out again while the building stands, what interrupts an
entry, and every event that changes the number of passengers. The stock data use this for
infantry; the same rules with a few differences serve a building type that absorbs vehicles
(claim 8). Skirmish only; network games and campaigns are out of scope.

Neighbouring specs own the rest, and this spec does not repeat them:
- the power each passenger gives, its health scaling, the house totals and when power is worked
  out again: S-0405 (claims 1, 3, 4 and 7),
- passengers that come out when the building is destroyed, the passengers that die with it,
  their credit and their orders afterwards: S-0312 (claims 24 and 25),
- passengers released by a sale: S-0408 (claims 6 and 8),
- the building's animation that depends on the passengers: S-0108 (claim 17),
- the passenger pips: S-0111 (claim 12),
- the original's enter-nearest instruction and its search: S-0215 (claim 24),
- the *enter* behaviour, its repeated entry request and the standard delay: S-0215 (claims 6
  and 29),
- the enter order and its cursor (S-0214, transport verdict *x*, rows x1 to x4) and the deploy
  order that starts a release (S-0214 row B5 and table M),
- the release of a controlled unit: S-0601 (claims 8 and 9),
- the spin reset of a gattling-type entrant: S-0609 (claim 11),
- the tag raise on entry: S-0524 (claim 12).

Cross-spec changes required at hand-off:
- S-0601 claim 12's bullet on victims absorbed by a bio reactor, its neighbour list on bio
  reactors (Summary), and MC6's bio-reactor repeat, say that absorption drops a controlled victim
  silently as an object of the controlling house. That is wrong for absorbing buildings: an
  accepted entry releases the victim by S-0601 claim 9 first (claim 7 below), with its release
  sound and its return to the original owner. Accepted absorption belongs among S-0601's release
  events. Grinders are not covered here.
- S-0405 claim 7 lists "a passenger enters or leaves an absorbing building" without a condition.
  Entry and release make power due only when the type's extra output is above 0 (claims 7 and
  13). With extra output 0 or below the passengers add no power (S-0405 claim 1), so only the
  due mark itself differs.
- S-0108 claim 17 gives "no release path is established" as the reason one case is unmeasured.
  Claims 11 to 13 below are such a path while the building survives.
- S-0312 claim 24 and S-0408 claim 8 say "the first passenger"; that passenger is the one that
  went in most recently (Terms, *passenger order*).

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
- **Acceptance test.** The building's docking answer (S-0214) to a would-be passenger (claim 1).
- **Voluntary release.** The owner's order that lets passengers out of a standing absorbing
  building (claims 11 to 13).

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
   - Backing: `measurable` (scenarios A1 and A12). Aircraft kind is only a docking-answer
     claim, not a promise that an ordinary aircraft order can complete absorption.
   - Confidence: high

2. **Capacity counts passengers, not sizes.** F1 compares the number of passengers, not the sum
   of their sizes, with the capacity, and compares the entrant's own size with the size limit.
   So a building with capacity *c* takes at most *c* passengers through this test whatever their
   sizes, and refuses any entrant whose size is above the limit even when it is empty. The size
   comparison is made in double-precision floating point.
   - Backing: `measurable` (scenario A1)
   - Confidence: high

### Getting contact

3. **Contact or waiting in line.** When a mobile actor with the *enter* behaviour gets an absorbing
   building as its destination and holds no contact:
   - If the building already holds any contact, the actor starts *waiting in line* for that
     building. Its destination is cleared and it stays where it is.
   - Otherwise it asks the building for contact. The building grants it when it counts the actor's
     house as allied, the actor's house also counts the building's house as allied, the
     building's health is above 0 and a link place is free. The actor then asks to
     load. For an otherwise ordinary absorbing building, switching it off refuses loading,
     and the actor then gives up the contact. Combined building features may add their own
     loading restrictions; those combinations are not specified here.

   With contact, the actor walks to the building under S-0215 claim 29.
   - Backing: `measurable` (scenario A2)
   - Confidence: high (waiting in line, switch); medium (the grant conditions other than a free
     place)

4. **Waiting in line.** In each tick, an actor waiting in line for a building that is still in the
   game asks for contact:
   - granted: it asks the acceptance test (claim 1). On *accept* it queues *enter* with that
     building as its destination and stops waiting. On *refuse* it gives up the contact, its
     behaviour becomes none, its destination is cleared and it stops waiting,
   - not granted (no free link place, or any other reason): it keeps waiting.

   A building that has left the game ends the wait. Any new order ends it too (S-0213, table T3,
   part (b)). So entrants go in one at a time through a building with one link place, and the
   capacity is checked again for every entrant that had to wait. The first entrant's capacity was
   checked only by the order's docking answer (claim 1, S-0214).
   - Backing: `measurable` (scenario A2, including an alliance ending during the wait).
   - Confidence: high (re-check and order); medium (waiting after an owner change)

5. **Interruptions while approaching.** At each of its processings (S-0215 claim 29) the entrant
   asks to load again. A building switched off since then refuses, so the entrant gives up the
   contact and falls back to rest. Other interruptions are owned elsewhere: a new order (S-0213,
   table T3), the building's destruction (S-0312 claim 17), its sale (S-0408 claim 6) and its
   change of owner (S-0509 claims 12 and 14).
   - Backing: `measurable` (scenario A6)
   - Confidence: high

### Going inside

6. **When an infantry unit goes in.** An infantry unit tries to go in when it arrives at the end of
   a walking step (S-0208 claim 22) in a cell that holds the building, while its behaviour is
   *enter* and the building is its destination, or its target, or the building in its destination
   cell. Unlike S-0508 claim 2, the target counts here. At that moment the building's tag
   receives one entry raise with the entrant as causing object (S-0524 claim 12), while the
   entrant is still outside. Then the entrant asks to complete its entry. The infantry absorber
   agrees unless it is being sold. It does not test capacity, size or the switch again (Edge
   cases, `quirks.absorber_contact_no_capacity_recheck`).

   When the building does not agree, nothing more happens at this arrival.
   - Backing: `measurable` (scenarios A3 and A7) for ordinary destination entry and capacity.
     `unmeasured` for target-only and destination-cell entry, and completion during a sale:
     no fixture here establishes those arrivals with contact retained. Size and switch changes
     between the last loading request and completion are also `unmeasured`: A6 tests the earlier
     loading refusal, not that interval.
   - Confidence: high (agreement without re-check); medium (the target and destination-cell
     cases; what follows a refusal)

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

   No stock type is a vehicle absorber.
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
    - a neighbouring cell with a bridge deck is passed over, as in S-0507. When this happens at the
      last direction of the first round, the search ends without a second round (Edge cases,
      `quirks.absorber_release_bridge_ends_search`),
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
      `unmeasured` for the bridge-deck end, which needs a bridge beside the building's top-left
      cell, and for a failed placement after a serving direction.
    - Confidence: high (rounds, deck check); medium (the corner, which rests on the building's
      own cells answering *blocked*; failed placement)

13. **After the attempt.**
    - **On success:** *P* is no longer a passenger and *n* goes down by 1. At an infantry absorber
      the exit sound plays at the building (Import mapping), and an uncounted *P* counts again
      towards its owner's crate infantry count. A vehicle-only absorber does not perform that
      infantry-count adjustment. *P* queues *move* to the far cell of the exit direction (first round)
      or to the neighbour cell (second round). Power becomes due as in claim 7. In the original a
      passenger of a human-controlled house also drops an enter-nearest instruction (S-0215 Terms).
      In our engine no actor holds one.
    - **On failure:** *P* goes back to the front and stays inside. The next processing tries *P*
      again, so a passenger that finds no exit blocks the others.
    - Backing: `measurable` (scenario A5)
    - Confidence: high

### Other ways out

14. **Destruction, sale and erasure.** A destroyed building releases or kills its passengers by
    S-0312 claim 24, and a sold one releases them by S-0408 claim 8. In both cases they come out
    from the front, so the passenger that went in most recently takes the first foundation cell.
    A passenger that comes out this way counts again towards its owner's crate infantry count and
    keeps its owner. An erased building deletes its passengers (S-0514 claim 12).
    - Backing: `measurable` (scenario A10) for release order, owner and crate count on
      destruction and sale; erasure inherits S-0514 claim 12's backing.
    - Confidence: high (order); medium (crate count)

15. **What changes the passenger count.** *n* goes up only when an entrant goes in (claims 7 and
    8). It goes down when a passenger comes out by voluntary release (claim 13), by the building's
    destruction or sale (claim 14), or when a passenger leaves the game by another rule while it is
    inside: for example the defeat of the passenger's own house destroys it inside (S-0414 claim
    5), which can happen to a formerly controlled enemy (claim 10). The building's switch, power,
    nonlethal damage and ownership change do not by themselves change *n*. Whether removal of
    a passenger inside makes the building owner's power due is not established (Open questions).
    - Backing: `measurable` (scenarios A5, A9, A10 and A11) for the count changes.
      `unmeasured` for power-due behaviour on removal inside: the complete removal path is not
      established; A11 proposes a discriminating observation, not a required stale-power result.
    - Confidence: high (entry and releases); medium (defeat removes the passenger)

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

## State and ordering

**Table E: an entrant** (claims 3 to 8).

| State | Event | Result |
|---|---|---|
| *enter*, no contact | destination set to an absorbing building that holds a contact | waiting in line, destination cleared, stays put |
| *enter*, no contact | destination set to an absorbing building with no contact | asks for contact; with contact asks to load; switched off: gives up contact |
| waiting in line | contact granted, acceptance test accepts | *enter* queued, destination = the building, waiting ends |
| waiting in line | contact granted, acceptance test refuses | contact given up, behaviour none, destination cleared, waiting ends |
| waiting in line | contact not granted | keeps waiting, asks again next tick |
| waiting in line | building leaves the game, or a new order | waiting ends |
| *enter*, in contact | processing, building switched off | contact given up, rest |
| *enter*, infantry | arrives in a cell of the building (claim 6) | tag raise, then completion asked; not being sold: goes in (claim 7) |
| *enter*, vehicle | reaches a cell of a vehicle absorber (claim 8) | acceptance test; accept: goes in; refuse: destination cleared, *guard*, scatter |

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

Random draws: an *unload* processing that begins with passengers draws once for its standard
delay (S-0215 claim 6); the ordinary empty processing chooses 1 tick without that draw. Each
infantry placement draws as S-0208 claim 3 gives. A controlled entrant whose original owner is
computer-controlled (only the neutral and special houses in our engine) consumes S-0601 claim 8's
draw. An entrant that gives up its contact and rests draws as rest does (S-0215 claim 6).

## Edge cases and quirks

- **No capacity re-check at contact (`quirks.absorber_contact_no_capacity_recheck`, default on).**
  An infantry entrant that reaches the building goes in whatever the passenger count, the switch
  or its size (claim 6). With one link place this cannot overfill the building, because the next
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
  first round passes both cell tests but its neighbour cell carries a bridge deck, the second round
  never starts and the passenger stays inside this processing. *Fixed behaviour:* the second round
  starts as it does after any other failure at the eighth direction.
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
| `Passengers` (rules, building type) | capacity *c* (F1) | integer | 0 |
| `SizeLimit` (rules, building type) | size limit *L* (F1) | floating point | 0.0 |
| `Size` (rules, infantry and vehicle types) | entrant size *s* (F1), as S-0507 | floating point | 1.0 |
| `NumberOfDocks` (rules, building type) | dock count *d* (F2) | integer | 1 |
| `ExtraPower` (rules, building type) | extra output; power becomes due only when above 0 (claims 7, 13) | integer | 0 |
| `EnterBioReactorSound` (rules, `[AudioVisual]`) | entry sound at an infantry absorber (claim 7) | sound name | none |
| `LeaveBioReactorSound` (rules, `[AudioVisual]`) | exit sound at an infantry absorber (claim 13) | sound name | none |
| `Naval`, `MovementZone`, `BalloonHover` (rules, unit types) | naval, amphibious and balloon rows of claim 1, as S-0504, S-0211 and S-0214 read them | boolean, movement zone, boolean | as their owners give |

The entry and exit sounds are the rules-wide ones. The same sound keys in a type section do not
choose the sounds for these absorption events. The release sound of claim 7 is S-0601 claim 9's.
Absent values above are initial defaults. Later absent keys retain held values under S-0006
claim 3, including session-held rules-wide sounds.

## Measurement scenarios

Use a small purpose-made skirmish map, normal game speed, director and mods off. Stock data
apply except explicitly named data-only test variants. The stock infantry absorber is the Yuri
faction's power building; "fill it" means bringing it to its imported capacity. Use scripted input
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

**A2. One at a time** (claims 3 and 4). A building with exactly one free place. Select two
infantry of A and order both in the same tick. *Record:* per tick, each unit's position and
destination, the passenger count. *Expected:* one walks, the other stays put with no destination, and
after the first goes in, the second is refused when it asks (the building is full) and ends with no
behaviour. Repeat with two free places: the second starts walking in the tick in which the first
goes in or the next, by the update order (S-0203). A second entrant arriving despite full capacity,
or both walking concurrently with only one link place, falsifies these predictions.
For the separate contact test, use A and B with only the building owner counting the entrant's
house as allied: the docking answer may accept but no contact is granted. Restore both alliance
views: contact becomes possible. While a second entrant waits, end the alliance using ordinary
orders without deleting the building: it continues waiting when contact is refused; give it a
new move order to end the wait. Also destroy its destination while it waits: the wait ends.
Record contacts, waiting intention and destinations in all variants.

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

**A6. Switched off while walking** (claim 5). Order an infantry in from far away and switch the
building off while it walks. *Expected:* at its next processing it stops and stands. *Falsified*
if it goes in. Record its contact, destination and behaviour as well as position: the contact ends
and it falls back to rest at that processing.

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
sale. *Record:* identity, owner and release cell of each passenger, and each owner's crate
infantry count. *Expected:* the most recently entered infantry uses the first foundation cell of
S-0312 claim 24's order, later ones follow in reverse entry order, each keeps its owner, and
successfully released uncounted passengers are counted again. A different order, changed owner
or missing count restoration falsifies that part. Erasure uses S-0514 claim 12's scenario instead.

**A11. A passenger whose house is defeated** (claim 15). Put a controlled infantry of house C into
A's building (scenario A4), then defeat C. *Record:* A's passenger count and pips, and A's power
output each tick until a later event (for example a building placed) makes power due. *Expected
for claim 15:* the passenger is deleted and the count drops when defeat removes it; persistence
falsifies that part. *Discriminating observation for the open power question:* output retaining
the old contribution until the later event supports a missing due mark, while a drop at the next
house update rejects that hypothesis. Neither power outcome is required by this draft.

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

## Open questions

- Does anything heal a passenger inside (claim 9)? Not found; unmeasured.
- Does removal of a passenger inside make power due (claim 15)? The complete path is untraced,
  scenario A11 distinguishes stale output from an immediate re-evaluation.
- Should exits round the top-left corner (Edge cases) get a switch whose fixed behaviour searches
  round the whole foundation (for example S-0508's exit ring)? Lead decision.
- What an infantry entrant does after the building refuses its completion during a sale (claim 6):
  the arrival does nothing more; whether it then keeps trying is not established.
- Does target-only entry leave a mission target that opens foundation cells on release (claim 12)?
- Exact vehicle placement-search ties and combinations with other loading or unload features
  remain unspecified.
