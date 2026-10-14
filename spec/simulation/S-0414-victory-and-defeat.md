---
id: S-0414
title: Victory and defeat (the defeat tests of the normal and short game, what defeat does, when the game ends, the end delay, surrender and alliance orders)
topic: simulation
revision: 9
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0150, Q-0151, Q-0193, Q-0267, Q-0285]
supersedes: []
---

# S-0414: Victory and defeat (the defeat tests of the normal and short game, what defeat does, when the game ends, the end delay, surrender and alliance orders)

## Summary

When a skirmish house is defeated, what happens to it, and when the game ends and with which result
for each house: the defeat test of the normal game and of the short game, the destruction of a
defeated house's objects, the end test with allied victory, the delay between the end and the
result, the order of events when several houses fall in one tick, and what the surrender and
alliance orders of S-0213 do. Campaign missions (whose results come from map triggers) and network
games are outside this spec. Claims 19–23 give the skirmish result screen's statistics,
score and duration; its layout and visual design belong to the implementer.

**Joint hand-off confirmation.** S-0412 claim 11 agrees with claims 2, 6, 9 and 10 and the
bases-off edge case here: qualifying objects still prevent defeat; otherwise the first defeat
leaving no hostile pair ends the game, and later same-tick defeats turn pending wins into
losses only with `quirks.defeat_during_end_delay` on. Claims 8 and 16 retain the final-tick
commitment boundary. S-0009 claims 28 and 45 agree too: cleanup removes the house itself, and a removed house
receives no starting units and takes no defeat test. These are confirmations of existing rules,
not additional behaviour.

**Partial request answer.** Q-0193 is listed only for claim 17's defeat-count answer to
part of question 3. Reopen the request at hand-off and record that limited answer.
Its death effects, deletion, production lifetime, progress, completion, release, placement,
refund and surviving-factory outcomes remain unresolved in S-0312 and S-0404.
S-0404 claim 3 agrees with claim 15's off-map hit eligibility and creation-order precedence:
an item deleted by earlier factory-loss cancellation receives no later hit. Neither rule
settles the killed item's subsequent death or production outcomes.

## Terms

Glossary terms are used as defined there, in particular *house*, *country*, *tick*, *tick number*,
*game minute*, *in play*, *on the map*, *off the map*, *destroyed*, *removed*, *AI house*,
*human-controlled*, *computer-controlled*, *local player*, *damage that ignores defences*,
*warhead*, *quirk switch*, *departure*, *importer warning* and *measurement setup*. S-0009's terms
*house order*, *passive house*, *neutral house*, *special house*, *observer* and *base unit* (with
the *base-unit list*), S-0412's terms *alliance*, *allied*, *mutually allied*, *short-game setting*,
*bases setting* and *human participant*, and S-0203's *house update* (a house's own update in row
7 of S-0203's table) are used as those specs define them. In addition:

- **Defeated house.** A house that the defeat of claim 5 has struck. A house stays defeated for
  the rest of the game. A house that is not defeated is *undefeated*.
- **Contender.** An undefeated house still in the game that is not passive. Only contenders are tested
  (claim 1) and counted (claims 6 and 12). A house removed by unused-house cleanup (S-0009 claim 28)
  is no house of the game, not an empty contender.
- **Observer's house set aside.** In our engine an observer's house (S-0009 claim 22) is never
  tested (claim 1) and is left out wherever this spec counts or pairs contenders (claims 6 and 12,
  Formulas *K*), although it stays an undefeated contender by the definition above. The original
  tests and counts it like any other house (Edge cases, *Observers*).
- **Person-played house.** A player house created for a human participant who is not an observer.
  In a skirmish there is at most one: none when every player house is an AI house, or when the one
  human participant is an observer. An AI house is never person-played, and neither is an
  observer's house. Our engine's end test does not read this term (claim 6); only the end delay
  (claim 9) and presentation do. The original's own end condition (claim 6, departure) counts the
  houses created for human participants, an observer's house included.
- **Counted building.** A building that does not count as a vehicle (next entry) and whose type is
  marked neither insignificant nor not scored (Import mapping). It is on *B* and on the short-game
  count of claim 4.
- **No-escape marker, ignore-defences marker.** The two markers a hit can carry. S-0302 (Terms and
  claim 1) owns both and says which sources set them; S-0312 (claim 13) owns the one effect of the
  no-escape marker, that a destroyed vehicle leaves no crew.
- **Building that counts as a vehicle.** A building whose type has an undeploy type (the vehicle
  type it packs up into) and either has a one-cell foundation or names an undeploy type that
  carries the resource-gatherer flag (Import mapping: the flag is the vehicle type's, not the
  building type's). It is not a counted building, it is on neither *B* nor *V*, and it does not
  count in the short-game test (claims 2 and 4).
- **Object in existence.** An object that has been created for the house and not yet left play
  for good (S-0312): on the map, off it (inside a transport or a building), or still being
  produced. Starting an item creates its object (S-0404 claim 11), so an item in production, or
  finished and waiting to be placed, is in existence.
- **Nominal survivor.** An infantry unit that comes out of a destroyed building as one of its
  survivors (S-0312) when its type is marked nominal (Import mapping) and the building is one whose
  survivors are so marked (a condition on the building that is set when it is created and is
  not fully identified; low confidence). Its role is claim 4's.
- **Refinery-list building.** A building whose type is the type at the third position of the
  rules' refinery list (Import mapping). Its role is claim 3's.
- **End of the game.** The moment the end test of claim 6 first succeeds. The game keeps running for the
  end delay (claim 8) and is then over.
- **End delay.** The number of ticks between the end of the game and the moment it is over (claim
  8).
- **Result.** Win or loss, one per player house, fixed at the end of the game (claim 7) and
  changed only by claim 9. A result is *pending* from the end of the game until its commitment
  in the final tick (claim 16), and then *committed*: later defeats cannot change it.

## Behaviour

### The defeat test

1. **When houses are tested.** In a skirmish every contender is tested once per tick, during its own
   house update, in every tick except the first (tick number 0). A house removed by unused-house
   cleanup is never tested, counted as an enemy or contender, or given a result: S-0009 claim 28
   already removes the house itself, not merely its objects. Claim 16 fixes the test's observable
   place within the house update. A defeated house is never tested
   again, and a passive house is never tested at all: passive houses can be neither defeated nor
   counted. In our engine an observer's house is never tested either, so it is never defeated
   (Terms, *observer's house set aside*; Edge cases, *Observers*). The test is claim 2 when the short-game setting is
   on and claim 3 when it is off.
   - Backing: `measurable` (scenario V2 for tick 0 and the per-tick test, V3 for passive houses,
     V13 for cleanup-removed houses),
     while the observer's house is a rule of our engine and `unmeasured` (the original tests it, so no
     run of the original can show our rule; V10 records what the original does)
   - Confidence: high

2. **The short-game test.** With the short-game setting on, a house passes the test while it has
   at least one counted building (claim 4) or at least one vehicle of one of the types at the first
   three positions of the base-unit list. Both counts cover the house's objects in existence
   (Terms), wherever they are: on the map, inside a transport, or still in production. A vehicle of
   a type marked insignificant or not scored never counts. Infantry, aircraft, other vehicles,
   buildings that count as vehicles and buildings that are not counted do not keep the house
   alive. Only the first three positions are looked at, whatever the list's length (quirk
   `quirks.short_game_first_three_base_units`). A house that fails is defeated (claim 5).
   - Backing: `measurable` (scenario V1, with variants g, h, i and j) for the tested cases.
     A base-unit vehicle still in production is `unmeasured`: its vehicle factory is itself a
     counted building, so it masks that vehicle's contribution; no setup is identified that
     retains that production item after losing the factory while isolating its contribution
   - Confidence: high for buildings, the three list positions and vehicles off the map (claim 4:
     the count runs from an object's creation until it leaves play); medium for items still in
     production

3. **The normal test.** With the short-game setting off, a house passes the test while

   > *S* = *B* + *V* + *I* + *A* + *R* ≠ 0

   with *B* its counted buildings in existence (claim 4), *V*, *I* and *A* its on-map counts of
   vehicles, infantry and aircraft as claim 4 keeps them, and *R* the on-map count of its
   refinery-list buildings (in our engine a zero term when the refinery list has no third position,
   a departure, since the original's behaviour there is not established: Edge cases,
   `quirks.vehicle_buildings_keep_nobody_alive`). All are
   integers; *S* can be negative through claim 4's survivor rule, and a negative *S* passes too.
   The on-map counts mean that a unit inside a transport or a building does not count, while the
   transport or building counts by its own kind. A house that fails is defeated (claim 5). The term
   *R* is what keeps a house alive whose last object is a deployed resource-gathering vehicle that
   has become a refinery-list building, since such a building counts as a vehicle and not as a
   building (claim 4). Other buildings that count as vehicles keep nobody alive under this test
   (quirk `quirks.vehicle_buildings_keep_nobody_alive`).
   - Backing: `measurable` (scenario V2 with its variants)
   - Confidence: high for the five terms and the "not zero" test; medium for which building types
     count as vehicles

4. **How the counts are kept.** Each count changes only at the events in the table, with a change
   of owner treated as the paragraph after it says; a type marked insignificant or not scored
   (Import mapping) is treated as the last two columns say.

   | Count | An object is added | It is taken off | Marked insignificant | Marked not scored |
   |---|---|---|---|---|
   | *B* (counted buildings) and the short-game vehicle count | when the object is created (production start, map placement, starting units, survivors and any other creation) or comes to the house by a change of owner | when it leaves play for good (S-0312) or passes to another owner | never counted | never counted |
   | *V* (vehicles on the map) | when it enters the map | when it leaves the map (into a transport or building, or by death), only for a type not marked as not scored | counted | added but never taken off |
   | *I* (infantry on the map) | when it enters the map, unless it is a nominal survivor (Terms) | when it leaves the map | counted | never counted |
   | *A* (aircraft on the map) | when it enters the map | when it leaves the map | counted | never counted |
   | *R* (refinery-list buildings on the map) | when it enters the map | when it leaves the map | counted | never counted |

   **Change of owner.** For the four on-map counts (*V*, *I*, *A*, *R*), a change of owner of an
   object that is on the map counts as the object leaving the map for the old owner and entering
   it for the new owner, in the same instant. Each side follows its row and columns as for any
   leaving or entry: a not-scored vehicle stays on the old owner's *V* and is added to the new
   owner's *V* as well, and the new owner does not add a nominal survivor to *I* while the old owner
   takes it off. A change of owner of an object off the map (inside a transport or a building, or
   still in production) changes no on-map count on either side. *B* and the
   short-game count move with the object, from the old owner to the new one, wherever the object
   is (first row). Mind control (S-0215) and every other change of owner follow this rule.

   A building that counts as a vehicle is on neither *B* nor *V*: it is a building on the map, so
   only *R* can see it. Two consequences of the table are quirks of the original:
   - **A vehicle of a not-scored type** that has once been on the map stays on *V* for good, so in a
     normal game that contribution keeps its house alive even after the vehicle is gone, unless
     negative count contributions cancel it to make *S* = 0 (quirk
     `quirks.unscored_vehicle_stays_counted`, off: such a vehicle is never counted).
   - **A nominal survivor** is not added when it enters the map but is taken off when it leaves it,
     so each one that leaves the map (by death, or by entering a transport or building) lowers *I*
     one below the true number. A house can then fail the normal test while it still has units on
     the map, or pass it for good with nothing left when *S* has become negative (quirk
     `quirks.nominal_survivor_count`; off: a nominal survivor is counted like any infantry unit).
   - Backing: `measurable` for the isolated cases (scenario V2, variants d, e, g, i, j and k,
     and V1 variants g, h and j).
     The change-of-owner rule is measurable by V1 variant g run 1 (a building, losing side), V2
     variant j (a vehicle and an infantry unit, losing side, by mind control) and V2 variant k
     (an infantry unit, a vehicle and an aircraft, both sides, and a not-scored vehicle, by a test
     map trigger). A nominal survivor in a change of owner is `unmeasured`: which buildings make
     their survivors nominal is not identified (V2 variant h), so no run can be arranged to give
     one to another house until that is found. A change of owner of an object off the map is
     `unmeasured`: no input is known that changes the owner of an object while it is inside a
     transport or building (a map trigger leaves such an object's owner as it is, S-0011 claim 24).
     Other count-table cases not isolated by V1 or V2 are `unmeasured`: in particular aircraft
     and refinery-list building flags, the gaining side of a building's ownership change, and
     production-item ownership changes have no scenario here that separates their contributions.
     Their event rules retain the confidence stated below; they are not claimed as measured.
     The nominal-survivor counting rule is also `unmeasured` until its building condition is
     identified: V2 variant h is a conditional diagnostic, not an arranged confirming run
   - Confidence: high for the events of the table, the change-of-owner rule and the not-scored
     column; medium for the insignificant column and for every kind of change of owner (mind
     control, a map trigger, a capture) following the one rule; low for which survivors are
     nominal survivors

### What defeat does

5. **Defeat.** A house that fails its test is defeated at once, in the same house update:
   - **Its objects are destroyed.** Every infantry unit, vehicle, aircraft and building attributed
     to the house by the ownership conditions below is offered a single hit, except for claim 15's
     cancellation-skip quirk, including objects
     off the map (passengers, occupants, production items and finished items awaiting placement).
     Claim 15 fixes the order and how intervening deaths or cancellations affect this set. Its amount is the object's current health and its warhead is the
     demolition warhead (S-0303, Import mapping); it is damage that ignores defences, it carries the
     no-escape marker, and it has no attacker and no attacking house (S-0302, S-0312). So passengers
     do not escape. An object that is
     being erased (glossary) stops being erased and takes the hit like any other object. Objects
     under mind control (S-0215, Terms) are treated by their original house:

     | Object | Condition | Effect |
     |---|---|---|
     | of another house, held by this house | — | not hit |
     | of this house, held by another house through a controlling unit | some house of the game plays a country of the civilian side (the civilian side of claim 12, Import mapping) | not hit; the first such house in house order takes this house's place as its original house, so when the control ends it passes to that house instead of coming back |
     | of this house, held by another house through a controlling unit | no house of the game plays a country of the civilian side | hit |
     | of this house, taken over for good by another house, with no controlling unit | — | hit |

     How each object dies (death weapons, survivors, remains) is S-0312's.
   - **The house is marked defeated** (claim 1), whatever happens to those objects afterwards.
   - **The reveal and the radar** (owned by S-0409 and S-0410, summarised here). In the tick of the defeat the defeated
     house's whole map becomes explored, as a change of its game state and for every defeated
     player house, human or AI (S-0409 claim 15, *Defeat*; the original does it for the local
     player's house only), and its radar comes on at the same moment and stays on (S-0410 claim 1).
     The exploration is immediate, not guaranteed permanent: pending re-cover marks survive
     the defeat reveal and may make their cells unexplored at a later sweep; loss of an active
     spy satellite can re-cover the map at a later recheck (S-0409 claim 15). Radar stays on
     despite either loss of exploration. In our engine a defeated person keeps watching until
     the game is over (claim 6), subject to those exploration changes. In the original the game
     ends at that house's defeat and then runs only for the end delay (claim 8); a test rules
     file may lengthen that delay to observe these effects (S-0409 scenario V9, parts F, F2
     and F3).
   - **Presentation.** A message names the defeated house and an announcement plays. Neither
     happens for an observer's house, which only the original defeats (claim 1; scenario V10).
     S-0701 claim 36 owns the defeat, player-defeated and victory announcement roles.
     After the local player's house is defeated, the alliance-formed and enemy-alliance-formed
     announcements and the superweapon launch and detection announcements of S-0701 claim 28
     are muted. This does not mute every announcement: alliance requests and alliance breaks
     retain S-0701 claim 36's conditions. The muting also applies while our defeated local player
     watches the remaining sides. These are presentation only and reach no game state.
   - **The end test** of claim 6 belongs to the same defeat. It counts the house as already
     defeated, and every house later in house order as it stands before its own house update in
     that tick, so a house that fails later in the same tick is still a contender at this
     test (claim 10, State and ordering, scenario V7).
   The original's computer players may react to a defeat (an option of its computer player makes
   the remaining computer houses ally each other); our AI houses follow ordinary orders only, so
   this is recorded for AI designers.
   - Backing: `measurable` (scenario V2: every object gone in the tick of the defeat; scenario V4
     part a for the passengers and the mind-control cases of the first two table rows, part b for
     the civilian-side cases, part c for the ignored defences, the missing crew and the missing
     credit). Not measured, with reasons: the amount equal to the current health is
     `unmeasured` (no observable separates an amount equal to the current health from a larger
     one, and S-0312 owns any effect that reads the amount of a killing hit); which warhead is
     named is backed by S-0303 (claim 19) and not by this spec's scenarios; the last table row
     (taken over for good, no controlling unit) is `unmeasured` (no way to arrange such an object
     is identified). Stopping an erasure is `unmeasured`: V4 does not arrange an object under
     an erasing weapon at surrender. The defeat message and announcement are `measurable`
     through V7's message capture and audio recording; post-defeat announcement suppression
     is `measurable` through V20, using a lengthened end delay in the original.
     The computer-player alliance reaction is `unmeasured`: no scenario isolates its optional
     AI response, and it is outside our engine's rules. Reveal and radar are backed by the
     scenarios of S-0409 and S-0410 cited above
   - Confidence: high for the destruction, the ignored defences and the first two table rows, and
     medium for the missing attacker and attacking house (S-0302 claim 1 gives them, V4 part c
     checks them through the result screen only); medium for the civilian-side cases

### The end of the game

6. **The end test.** The end test runs only right after a house is defeated (claim 5), never at
   another time. It counts the contenders, the defeated house no longer among them and, in our
   engine, an observer's house left out (Terms, *observer's house set aside*). The game ends
   when **no hostile pair remains**: no two contenders exist of which one does not ally the other.
   This includes the case of at most one contender. So the game ends when every remaining
   contender is mutually allied with every other: an *allied victory*, or the victory of the last
   contender. Otherwise play goes on. AI houses are contenders like any other house, and the test
   needs no person-played house: a skirmish in which no house is person-played runs, and ends, by
   the same rule. A person whose house is defeated keeps watching until the game is over, while
   that house's allies fight on. Once the game has ended, a later success of the test changes
   nothing: what a later defeat changes is claim 9's. Because the test only runs after a defeat,
   making peace (claim 12) can never end the game by itself, and claim 12 refuses an alliance that
   would leave its issuer with no enemy.

   **Departure (design choice of scope, no switch; lead decision of 2026-10-11, "play to the last
   side").** The original also ends the game when **no contender created for a human participant
   remains**: in a skirmish, as soon as the human participant's house is defeated, whatever its
   allies still hold, so the person never watches allies fight on. (With an observer as the only
   human participant, the observer's house is the one such house, so read literally the original
   ends the game at that house's defeat in tick 1; Edge cases, *Observers*.) Our engine does not
   reproduce this second condition. Every other part of the test (when it runs, who is counted,
   the allied victory) is the original's.
   - Backing: `measurable` for the original's test (scenario V5: part a for an allied victory, part
     b for the original's end at the human's defeat while its ally lives, part c for one-way
     alliances not ending it); that our engine plays on after the person-played house's defeat is
     a rule of our engine and `unmeasured`, since the original has no such game to observe
   - Confidence: high

7. **Results.** At the end of the game every player house that is undefeated wins and every
   defeated player house loses. The local player is shown its own house's result. In the original
   only the local player's result exists: it wins when its house is undefeated at the end and loses
   otherwise, which is the same result. Our engine gives every house its result so that no game
   state depends on the local player (glossary, *local player*). The result is per house: a
   defeated house loses even when its allies win, so a person who watched their allies win after
   their own defeat (claim 6) gets a loss. The neutral and special houses get no result, and in our
   engine neither does an observer's house (Edge cases, *Observers*).
   - Backing: `measurable` (scenario V5, the result screen) for the local player's result; the
     results of the other houses are a rule of our engine and `unmeasured`, since the original keeps
     no result for them
   - Confidence: high for the local player's result

8. **The end delay.** After the end of the game, play goes on until the game is over in tick
   *t* + max(*D*, 1), with *t* the tick of the end (Formulas), so at least one more tick runs.
   This final tick runs all its simulation parts, including every house update and the application
   of orders. No next tick runs. Houses are still tested and can still be defeated, including in
   the final tick. However, the result and expiry are committed before that tick's defeat tests:
   a defeat reported by those tests cannot change the committed result or restart the delay
   (claim 16). Thus 'during the end delay' in claim 9 excludes defeats tested in this final tick,
   even when the killing hit occurred earlier in that tick. In the original the program additionally waits, in real time, for
   the result announcement to finish before showing the result screen; that wait is presentation
   only, takes no tick, and our engine does not reproduce it.
   - Backing: `measurable` (scenarios V12 part c and V6, including part e for the minimum and the zero-result
     extreme). Wrapped end-delay values between the integer and 64-bit bounds are `unmeasured`
     here: V6 does not isolate their delay outcomes; the general conversion rule is backed by
     S-0205 claim 7 and its scenarios
   - Confidence: high for the formula; medium for the exact count (it depends on the rounding mode
     of S-0205)

9. **A defeat during the end delay.** Two effects, both under `quirks.defeat_during_end_delay`:
   - **The result.** Any player house that is defeated during the end delay loses, even if the end
     of the game had given it a win (claim 7). This is a rule of our engine for the AI houses,
     whose results the original does not keep.
   - **The delay.** When the defeated house is a person-played house whose win is pending, the end
     delay restarts: the game is over in tick *t* + max(*D*, 1) (Formulas), with *t* the tick of
     the later defeat. The defeat of any other house during the delay, an
     AI house included, leaves the delay as it is. This is the original's behaviour put without
     the local player: in the original the delay restarts only when the local player's house is
     defeated with a win pending, and the defeat of any other house leaves it as it is; in a
     skirmish without an observer the local player's house is the one person-played house, so the
     two statements agree, and the rule
     depends on no local-player state. A person-played house defeated before the end of the game
     has no win pending, so no later defeat restarts the delay, and in a game with no
     person-played house the delay never restarts. (In the original, a game that ended through
     the human participant's own defeat, claim 6, behaves the same way: a later defeat of a
     computer house changes nothing, and the delay runs from the end of the game.)
   With the switch off, the results and the end delay are fixed at the end of the game.
   - Backing: `measurable` (scenario V6, parts b, c and d); the AI house's result is `unmeasured`
     (the original keeps no result for an AI house, claim 7). The absence of a restart with no
     person-played house is `unmeasured`, since stock skirmish offers no such setup; it follows
     from our engine's person-played-house rule, not from V6
   - Confidence: high for the original's local-player case (a later defeat of the local player's
     house gives a loss and a fresh delay, an earlier loss is not restarted); medium for the exact
     tick count; the rule for AI houses is ours and has no original to check

10. **Several houses failing in one tick.** Each house is tested in its own house update, and the
    house updates of a tick follow house order (S-0203 row 7); each defeat's end test sees the houses
    later in house order as they stand before their own updates (claim 5). So when the last houses of two hostile
    sides all fail in the same tick, they are defeated one after another in house order, and the
    game ends at the first of those defeats after which no hostile pair remains, giving the houses
    still undefeated a win. Those are then tested in their own house updates later in the same
    tick, fail too, and lose by claim 9. Everyone loses (with `quirks.defeat_during_end_delay` on,
    while with it off claim 9 leaves the later houses their win). In particular a human who is
    wiped out in the same tick as the last enemy, with no ally left, loses with the switch on or
    off: the human's house is first in house order (S-0009 claim 19), so its defeat leaves the enemy as
    the last contender and ends the game with a loss for the human; were it later, its win would be
    taken back in the same tick when `quirks.defeat_during_end_delay` is on (with the switch off it
    would keep that win). When the human still has an undefeated ally, the human's defeat does not
    end the game, the enemy's later defeat does, and the ally wins while the human loses. In the
    original the end comes at the human's own defeat (claim 6, departure) even when the human has
    allies left; the human's result, the only one the original keeps (claim 7), is a loss there as
    in our engine.
    - Backing: `measurable` (scenario V7) for the human's result and the defeats in one tick; the
      results of the other houses are a rule of our engine and `unmeasured`, since the original
      keeps no result for them (claim 7)
    - Confidence: medium

### Surrender and alliance orders

11. **Surrender.** A surrender order (S-0213, table T1) marks its house to die when, at the moment
    the order is applied, the house is undefeated and has no result pending (State and ordering,
    and orders are checked when they are applied, S-0213). In its next house update, which is in the tick after the
    order (orders are applied after the houses' updates, S-0203 claim 7), every object of the house
    is destroyed as in claim 5. The defeat test of that same house update (claim 1) already sees
    the destruction, so the house is defeated in that tick when nothing counts any more. An object that stays in play for some ticks after its
    death (an exploding building, a sinking vehicle, a crashing aircraft, S-0312) may keep the house
    alive until it leaves play, and the counts of claim 4 may keep it alive for good, so the defeat
    can come later or not at all. A
    surrender order for a defeated house, or for a house whose result is pending, does nothing.
    In the original only the local player can surrender, through the options menu, and not when
    the local player is an observer. In our engine the order is legal for every player house, human
    or AI (S-0213); an observer's house is not a player house here, and a surrender order of it does
    nothing (Edge cases, *Observers*). A surrender ends the game
    only through the end test of claim 6, after the defeat it leads to: in our engine play goes on
    while a hostile pair remains, and a person who surrendered keeps watching; in the original the
    human participant's surrender ends the game at that defeat (claim 6, departure).
    - Backing: `measurable` (scenario V8 and its part b for a pending result); the order for a
      defeated house is `unmeasured` (such a house has no objects, so the order has no observable
      effect); the observer's order is a rule of our engine and `unmeasured` (the original offers
      an observer no surrender, so no run can give it)
    - Confidence: high for the destruction in the next house update; medium for the timing of the
      defeat when destroyed buildings stay in play; medium for the pending-result case

12. **Making peace: the alliance order towards a house not yet allied.** An alliance order (S-0213,
    table T1) from house A naming house B, when A does not ally B yet, makes A ally B (one way: B's
    view of A does not change) only when all of these hold:
    - B is not A, and B's country does not belong to the civilian side (Import mapping),
    - A is undefeated,
    - A does not leave itself with no enemy: the order is refused when exactly one contender other
      than A is one that A does not ally, whoever B is (so with one enemy left, A can make peace
      with no house at all, not even a passive or defeated one). In our engine an observer's
      house is left out of this count (Terms, *observer's house set aside*), so it never counts as
      that enemy.
    Otherwise the order does nothing. When A allies B, A's objects on the map whose current target
    belongs to a house that A now allies (B, an earlier ally, or A itself) drop that target. When A
    is person-played and B is not passive, a message reports the new alliance, and an announcement
    follows S-0701 claim 36's request-versus-mutual-alliance conditions and post-defeat muting
    (presentation). The original's computer player
    reacts to some alliances made by human houses (an option of its computer player makes its
    houses ally each other); our AI houses act only through orders.
    - Backing: `measurable` (scenario V9, part a: the one-way effect, the refusal at the last enemy
      and the dropped targets); `unmeasured` for the refusals "B is A", "B's country is of the
      civilian side" and "A is defeated" (the original's in-game player list is not known to offer
      such an order to a player, so no run can give it; a part is added if an input is found that
      can). The optional computer-player reaction is `unmeasured`: V9 voids reciprocal
      alliances rather than isolating that AI response. Alliance messages and announcements
      are `measurable` through the screen and audio capture in V9
    - Confidence: high for the last-enemy condition and the one-way effect; medium for the dropped
      targets; low for the three unmeasured refusals

13. **Breaking an alliance: the alliance order towards an allied house.** An alliance order from
    house A naming house B, when A already allies B, ends the alliance in both directions: A no
    longer allies B and B no longer allies A. The order does nothing when A or B is passive. An
    alliance order that names A itself counts as naming an allied house and changes nothing. Unlike
    making peace (claim 12), breaking an alliance does not require A to be undefeated: a defeated
    house's order towards an allied house still ends the alliance in both directions. A message
    reports a break when A is a person-played house, and an announcement plays when B had allied A
    and A or B is the local player's house (presentation).
    - Backing: `measurable` (scenario V9, part b: the break in both directions); `unmeasured` for
      the passive-house refusal and the order that names A itself (the original's interface is not
      known to offer them to a player), and for the break by a defeated house (in the original a
      human's house is defeated only at the end of the game, claim 6, departure, or within the end
      delay, claim 9, so the order would have to come in the end delay, through an interface not
      known to offer it there). The break message and announcement are `measurable` through
      V9's screen and audio capture
    - Confidence: high for the break; medium for the break by a defeated house; low for the
      passive-house refusal and the order that names A itself

14. **When the alliance order can be given.** In the original the alliance order can be given only
    while two flags of the rules' multiplayer settings are both on, the allies-allowed flag and the
    ally-change-allowed flag (Import mapping; their absent-key defaults are off and on), and the
    game does not fix alliances. No skirmish does: in every game other than a campaign mission, a
    fixed-alliance mark that the map sets is cleared once the map has loaded. The two flags are
    read with the rules layers of the game (S-0006). How the player gives it is interface: from the in-game player
    list for any house listed there, or by a key for the owner of a selected object, which the
    original offers only when that owner is a human participant's house. In our engine the two
    flags are a condition of the alliance order, checked when the order is applied, for every
    house alike: with either flag off, every alliance order does nothing. Alliances made at game
    start (S-0412 claim 8) do not depend on the flags.
    - Backing: `measurable` (scenario V9, part c: both flags, the cleared mark, and the start
      alliances that ignore the flags); the interface details are `unmeasured` (presentation of the
      original's menus; they reach no game state)
    - Confidence: high for the two flags, the cleared mark and the start alliances; medium for the
      interface details

### Destruction order and the final tick

15. **Observable hit ordering and scope (Q-0150).** For defeat and surrender destruction, the observable precedence of offered hits and their associated death effects follows object creation order across all four object kinds, subject to the cancellation exception below; it is not grouped by kind or by order of entering the map. Boarding, leaving a transport and changing owner do not change this precedence. For a produced item, creation is the start of production (S-0404 claim 11), not eventual release. Off-map objects are included. Where two offered hits are both reached, the earlier-created object's death effects and death draws precede the later-created object's hit and death draws (S-0312 claim 1). The destruction process itself takes no random draws.

    Eligibility and hit consequences reflect the object's state when its hit is offered, including changes caused by earlier death effects, rather than a fixed eligibility set from before destruction begins. A passenger already killed with its carrier receives no second death and contributes no second set of death draws. A production object immediately deleted by factory loss and cancellation is unavailable for a hit. An object created by a death effect is included at its creation position when claim 5's ownership conditions hold. A death effect may change control or cancel production in time to affect a later-created object's eligibility. An object that remains present after a hit because its removal is delayed receives no repeated hit for that reason. The offered hit uses the object's health at that time; an already-dead object does not undergo a second death from a zero hit. S-0312 and S-0404 retain ownership of passenger, cancellation and death rules.

    **Cancellation exception.** Under `quirks.defeat_cancellation_skip` (default original), if the effects of a destruction hit immediately delete *k* objects whose creation positions are at or before that hit's position in the combined creation order, the next max(*k* − 1, 0) objects in that order receive no destruction hit from this sweep. Count all four object kinds and all owners, including objects that would not themselves qualify for a destruction hit. An immediate deletion means the object ceases to exist at once, as in production cancellation; merely leaving the map or awaiting delayed removal after death is not a deletion for this rule. Thus an otherwise eligible later object can miss its hit. The count and formula use non-negative integers, exact arithmetic, and no rounding. With the switch off, such deletions do not cause any eligible object to miss its hit. One immediate deletion skips no objects; otherwise objects that remain retain their creation order.
    - Backing: `measurable` (V11 for cross-kind order, re-entry, off-map passengers and production
      items, V11 part d for the cancellation exception; S-0312 claim 1 for completion of each
      death's effects). Inclusion of newly created
      eligible objects is `unmeasured` here: no isolated defeat setup that creates such an object
      without a nested trigger destruction or cancellation masking its later hit is identified
    - Confidence: high for creation order, no off-map exclusion and no repeated hit solely due to
      delayed removal; medium for changes to eligibility and cancellation skips during the sweep

16. **Consequences within a house update and at expiry (Q-0151).** Power totals and the ordinary
    radar recheck of a house update do not yet see that update's surrender or defeat destruction.
    A power change made due by that destruction is taken up in the next house update, not this
    one. The forced radar-on effect at defeat remains immediate (claim 5, S-0410 claim 1),
    overriding ordinary availability without waiting for that later recheck. The defeat test
    already sees the surrender destruction of the same update. The tech-tree recheck and its
    sidebar cancellation orders, when due, see both kinds of destruction in that same update
    (S-0407 claim 12, S-0411 claim 2). This says nothing about an object kept on the map after
    death: each topic still uses its own counting conditions.

    At the final tick of claim 8, the pending result becomes committed before surrender
    destruction and defeat testing. The remainder of the tick still runs, so a house can be
    marked defeated in that tick while keeping its committed win, and orders applied after the
    houses can still change objects. Such a defeat neither changes that win nor buys extra ticks,
    even with `quirks.defeat_during_end_delay` on. A defeat tested one tick earlier changes the
    pending win and restarts the delay as claim 9 says. In the original this commitment is in
    the local player's house update, before its own defeat test; the local player's house is
    first in skirmish house order. Our per-house results and games without a person-played house
    use the same cutoff for all results: the start of the houses' part of the final tick. This
    extension is a design rule, not a claim that the original stores other houses' results.
    - Backing: `measurable` (V12); the common commitment for non-local houses and a game without
      a person-played house is `unmeasured`, since the original has no such result state
    - Confidence: high for the relative places of destruction, power, radar and tech-tree work,
      medium for the boundary result, pending V12's discriminating run

17. **Defeat counts of a killed production item (Q-0193, count part of question 3).**
    For an item that has never entered the map, reducing its health to 0 does not by itself
    remove its contribution to the existence counts of claim 4. While its object still exists
    for the same owner, a counted-building item contributes one to *B* in either game, and an
    eligible base-unit vehicle contributes one to the short-game vehicle count. The type
    exclusions and first-three-position rule of claim 2 still apply. An infantry, aircraft or
    other vehicle item contributes nothing to the normal game's on-map counts, before or after
    that hit. All these contributions are integer counts, with no rounding.

    Cancellation that deletes the object removes its existence contribution at that deletion,
    not at the earlier killing hit. If deletion and the defeat test happen in the same tick,
    the test sees whichever contributions remain at its own house update. This does not
    reverse a defeat already recorded: a defeated house is never tested again (claim 1).
    Nor does the zero-health item exempt the house from the destruction sweep (claim 15).
    This is the existing event-count rule made explicit, not a separate quirk.

    Whether the hit itself causes deletion, what death effects it takes, and whether the line
    continues, finishes or releases the item are separate death and production questions,
    owned by S-0312 and S-0404. This claim does not assume that every killed item survives
    until cancellation, or that a killed item can be placed.
    - Backing: `measurable` (V14 for the retained counted-building item and its cancellation).
      The base-unit vehicle's isolated contribution remains `unmeasured` for the factory
      masking reason in claim 2. The absence of on-map contributions for never-placed units
      follows claim 4 and is checked with V14's infantry control
    - Confidence: high for the health-independent count and deletion rule; medium for arranging
      a retained killed item in the proposed setup

18. **Owner-house tag raises within the house update (Q-0267).** In ticks whose integer
    tick number is a multiple of 8, including tick 0, the assigned-tag raise of S-0011 claim 35
    sees the effects of that house's surrender destruction and defeat test, including defeat
    destruction, from the same update. The power and ordinary radar rechecks have already
    happened. The due tech-tree recheck and its sidebar cancellation orders come afterwards
    (S-0407 claim 12, S-0411 claim 2). This claim does not relocate the separate automatic
    unit-release work of S-0411 claim 9.

    Thus a type-count condition in that raise reads the counts remaining after surrender or
    defeat destruction, not the pre-destruction counts. An action of the raise that removes a
    prerequisite can affect a due tech-tree recheck and its cancellation orders in that same
    update, but cannot alter the power totals or ordinary radar result already established
    there: its newly due power or radar work waits for the next house update. Objects retained
    after death still follow each topic's own counting rules. No second defeat test occurs in
    the same update: if the raise removes the last qualifying object of an otherwise surviving
    house, its next defeat test is in the next tick. A variable changed by an earlier house's
    raise is already changed when a later house's tags are evaluated in that tick.

    Defeated houses still take their places in house order and keep these raises, including
    the raise in the update that defeats them and later eligible ticks during the end delay.
    A cleanup-removed house takes no place and raises nothing: S-0009 claim 28 removes the
    house itself, and S-0011 claim 35 already excludes it from tag assignment. The final-tick
    result cutoff of claim 16 is unchanged by a later tag action.
    - Backing: `measurable` (V15, with V13 for cleanup absence and S-0011 T20 variant C
      for variable effects in house order)
    - Confidence: high for the relative places and continued raises after defeat; medium for
      arranging the same-update tech-tree probe

### Result-screen statistics (Q-0285)

19. **Statistics shown and their cutoff.** The skirmish result screen lists each surviving
    non-passive house other than the observer's house, including defeated houses, with its
    player name, kills, losses, built total and score. Cleanup-removed houses have no row.
    These totals are read after play stops, not frozen at the first successful end test:
    events during the end delay and its final tick still contribute. There is no separate
    harvested-credits figure. Harvesting contributes to score as claim 22 says; neither the
    credit balance nor credits spent is itself the score. Screen layout is not specified.
    Results in our engine remain the committed results of claims 7–16, even if the house
    is defeated later in the final tick. Neither the statistics nor the score are a second
    result test. In particular, the winner-score bonus of claim 23 can be absent for a
    house that kept a committed win through the final tick's defeat.
    - Backing: `measurable` (V16 and V12 part c) for the original's statistics and local
      result; our per-house results remain the design extension of claim 7
    - Confidence: high

20. **Built total.** The total is the sum of the four kinds' ever-built counts, not the
    number of objects ever owned or created and not the surviving-object count. Factory
    release raises it by one for infantry, vehicles and aircraft, provided the released type
    is not marked not scored. The insignificant mark alone does not exclude a release.
    Starting units, map objects, free refinery vehicles, clones, crates, survivors, trigger
    creations, captures, mind-control ownership changes, deploys and pack-ups do not raise it.
    An item merely started, completed but awaiting release, or cancelled before release raises
    nothing. Ordinary building placement raises nothing under
    `quirks.placed_buildings_not_ever_built` (S-0407 claim 10); with that shared switch off,
    eligible placed buildings also raise the total. Counts never decrease on death, sale,
    conversion or ownership loss. These are signed 32-bit integer counts with S-0205's
    wrap and no rounding.
    - Backing: `measurable` (V16; S-0407 T10 and S-0404 release scenarios)
    - Confidence: high for the total and not-scored exclusion; medium for the listed creation
      sources, checked by V16's separate controls

21. **Kills and losses.** Kills sum credited destructions of infantry, vehicles, aircraft and
    buildings of every victim house, including allies, the same house and non-player houses.
    A not-scored victim contributes neither a kill nor a loss. An insignificant building
    contributes neither; an insignificant infantry unit, vehicle or aircraft still contributes.
    A destruction without an attacking object gives no house a kill, but still gives the victim's
    house a loss when it qualifies. This includes defeat and surrender hits. Passenger deaths
    are counted individually when their death rules credit them to an attacker (S-0312).
    Selling, cancellation, map exit, boarding and deployment conversion are not destructions
    for these totals merely because an object leaves the map or ceases to exist.

    Every change of owner to a different house, including capture and mind control, additionally
    gives the gaining house one kill for a not-scored-unmarked object, without giving the
    losing house a loss. The insignificant mark does not exclude this ownership-gain kill,
    even on a building. A return to an earlier owner is another ownership gain. The counts
    are integers with S-0205's signed 32-bit wrap and no rounding. A building whose initial
    owner's country registration index is −1 contributes no destruction loss; ordinary
    registered player countries do not have that index. This initial-country exception does
    not exclude its credited kill.

    **Quirk `quirks.ownership_gain_counts_as_kill`**, default original: ownership gains add
    kills. Off: only qualifying destructions add kills. This switch does not change ownership
    gain's score contribution in claim 22.
    - Backing: `measurable` (V17) for destructions, ownership gains, flags and passenger totals.
      The initial-country-index exception is `unmeasured`: no skirmish input creating a
      building for a country outside the registered list is identified
    - Confidence: high for the event counters and flags; medium for isolating the passenger
      and ownership-return cases; low for the exceptional initial-country setup

22. **Running score.** Each house starts with 0 score points. The score is not a function of
    its displayed kill, loss or built totals. Its contributions are:

    | Event | Contribution to the house's running score |
    |---|---|
    | A qualifying victim is destroyed by an attacking object | To the attacker's current house: the victim type's current price for the victim's house, multiplied by 1 for rookie, 2 for veteran or 3 for elite victims; 0 instead when the attacker's house allies the victim's house |
    | An object changes owner to a different house | To the gaining house: the object's type's current price for the old owner, with no allied or rank multiplier and no not-scored or insignificant exclusion |
    | A resource amount is credited by unloading or conversion to credits | To the receiving house: 5 score points per credited resource unit, with the rounding below; purifier bonus amounts count too |

    The destruction row excludes not-scored victims. An insignificant building can still
    contribute destruction score even though claim 21 excludes its kill. A destruction without
    an attacking object gives no destruction score. The rank is the victim's rank, not the
    attacker's. Current price is the owner-adjusted type price of S-0404 claim 2, evaluated
    at this event; it is not the amount previously paid or the type's separate point-value
    field (Import mapping). Losing objects, spending money, starting credits, refunds,
    cash crates and passive cash income do not by themselves change this score. Stored-resource
    conversion on sale also counts when such storage exists (S-0408); ordinary skirmish storage
    is empty. This score never subtracts the price of one's losses.

    Resource score uses the incoming single-precision resource amount *a*, not its credit
    value or the country's income factor. Update the integer running score *p* to
    trunc(*p* + 5 × *a*), using double-precision floating-point multiplication and addition
    under S-0205's original rounding mode, followed by its integer conversion toward zero
    and overflow rules. A purifier bonus is a separate update, so
    fractional contributions are not accumulated between updates. Resource removed through
    the building-loss conversion also adds 5 times the removed amount truncated to an
    integer, per resource type; this storage-only path is not ordinary skirmish harvesting.
    Other additions and integer multipliers wrap as signed 32-bit integers (S-0205).
    - Backing: `measurable` (V17 and V18) for destruction, ownership gains and normal
      resource income; storage-only conversions are `unmeasured`, since no skirmish setup
      filling that storage is known (S-0312 claim 22)
    - Confidence: high for the ordinary event contributions and rank/alliance conditions,
      medium for the storage-only path

23. **Displayed score and duration.** Let *b* = max(*p*, 0), in integer score points, for the
    running score at the screen's cutoff. A defeated house displays *b*. An undefeated house
    displays *b* + ⌊*b* / 2⌋ + *r*, where *r* is an inclusive ranged draw from
    ⌊*b* / 2⌋ through *b* on the scenario random stream (S-0202). There is one ranged
    request per undefeated listed house, in house order, after simulation has stopped.
    Defeated and omitted houses request none. Equal bounds consume no raw value and other
    ranges consume the values S-0202 claim 4 specifies. Integer sums use signed 32-bit wrap.
    The original bases this bonus on the house's defeat state at screen preparation, not
    on the local player's committed result. Our engine retains that score rule independently
    of its committed verdict display (claim 19). The bonus changes no simulation outcome.

    The duration is active wall-clock time, not ticks multiplied by the game's current paced
    period and not game minutes. The original accumulates elapsed 16-ms clock units while
    play is active, pauses that clock when the game is paused for the options menu or lost
    focus, and stops it on leaving play for the result screen. Frozen cycles and the end delay
    are included. It converts 60 such units to one displayed second, truncating toward zero:
    thus 960 ms of accumulated active time yields one displayed second, rather than 1000 ms.
    Display hours, minutes and seconds from that whole-second total, capped at 99 hours,
    59 minutes and 59 seconds. Hours, minutes and seconds are integer quantities with no
    further rounding. The result-announcement wait after play is included if it happens
    before the active clock is stopped; time subsequently spent on the result screen is not.
    This is a presentation clock only and reaches no simulation rule. Our engine may read
    active elapsed wall time for this display without changing S-0201's deterministic pacing
    or its prohibition on wall time reaching game state.
    - Backing: `measurable` (V18 for the score and draws; V19 for duration)
    - Confidence: high for the bonus formula, stream, clock units and time conversion,
      medium for paused intervals and the announcement-wait boundary

## Formulas

**Result duration** (claim 23). For each active interval with endpoints *u* and *v* in
wall-clock milliseconds, its contribution in clock units is ⌊*v* / 16⌋ − ⌊*u* / 16⌋.
Sum these contributions as integers to obtain *E*. The displayed whole-second total is
*T* = min(⌊*E* / 60⌋, 359999). Hours are ⌊*T* / 3600⌋, minutes are
⌊(*T* mod 3600) / 60⌋ and seconds are *T* mod 60. These formulas cover non-negative
elapsed time before the host clock wraps; they use integer arithmetic with the indicated
flooring, no floating-point operation. Host-clock wrap during an exceptionally long game
is not established (Open questions).

**End delay** (claim 8). With *d* the end-delay field in game minutes (a double, Import mapping), let *x* be the product *d* × 900, rounded toward zero to double precision according to S-0205 and `quirks.chop_rounding`. The integer delay *D*, in ticks, is *x* truncated toward zero to a whole number. The factor 900 is the glossary's conversion from game minutes to ticks. The conversion to a whole number follows S-0205, claim 7: a product of 2³¹ or more wraps (from a field of about 2.39 million game minutes up, *D* no longer follows the field, and whenever the wrapped *D* is 0 or negative the delay is the shortest one of the next formula), and a product of 2⁶³ or more gives 0. Example: at the absent-key default *d* = 0.03 the exact product of the stored double and 900 lies just below 27, so *D* = 26 with the switch on (the original) and 27 with it off.

**When the game is over** (claims 8 and 9). With *t* the tick number of the end of the game (or, under
claim 9, of the later defeat), the game is over in tick *t* + max(*D*, 1): the delay is counted from
the tick of the end, and a delay of 0 still lets one more tick run. All quantities are integers,
and the arithmetic is exact.

**End test** (claim 6). With *K* the set of contenders after the defeat (in our engine without an
observer's house; in the original with it) and *a*(*x*, *y*) true
when *x* allies *y*: the game ends when *a*(*x*, *y*) holds for every ordered pair of different
members *x*, *y* of *K* (always true when *K* has at most one member). The original also ends it
when *P*, the set of members of *K* created for human participants (an observer's house
included), is empty (claim 6, departure); our engine does not.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| First tick with a defeat test | 1 | tick number | claim 1 |
| Base-unit list positions read by the short-game test | first three | list positions | claim 2 |
| Refinery list position read by the normal test | third | list position | claim 3 |
| End delay field, absent-key default | 0.03 | game minutes | claim 8 |
| End delay at that default | 26 | ticks | claim 8, Formulas (27 with `quirks.chop_rounding` off) |
| Surrender delay before the destruction | 0 | ticks | claim 11; counted from the start of the house's next house update |
| Shortest end delay that takes effect | 1 | ticks | Formulas, when the game is over |
| Allies-allowed flag, absent-key default | off | boolean | claim 14 |
| Ally-change-allowed flag, absent-key default | on | boolean | claim 14 |

## State and ordering

**House states.**

| State | Event | Next state | Effect |
|---|---|---|---|
| undefeated | its test fails in its house update (tick ≥ 1) | defeated | claim 5: objects destroyed, end test run |
| undefeated | surrender order applied, no result pending | undefeated, marked to die | none until its next house update |
| undefeated, marked to die | its next house update | undefeated | all its objects destroyed (claim 5); the test of the same update already sees this, so the house can be defeated in that tick |
| defeated | — | defeated | never tested again |
| any | end of the game | same | pending result set (claim 7); end delay starts |
| any with a pending result | houses' part of the final tick starts | same, result committed | later defeat tests cannot change the result or expiry (claims 8 and 16); all remaining simulation parts still run |
| undefeated with a win | defeated during the end delay | defeated | result becomes a loss (claim 9); the end delay restarts when the house is person-played (both with `quirks.defeat_during_end_delay` on) |

**Observable ordering.**
- The defeat test of a house update already sees the destruction of a surrender pending for that
  house, so a surrendering house with nothing left that counts is defeated in the tick its objects
  die (claim 11, V8).
- A defeat's end test sees every house later in house order as it stands before that house's own
  test in the same tick, so a house that fails later in the tick still counts as a contender when
  an earlier defeat ends the game (claim 10, V7).
- Power and ordinary radar work do not see the surrender destruction of their own house update,
  but its due tech-tree recheck does see it (claim 16, V12).
- In each house's update, its every-eighth-tick assigned-tag raise sees its surrender and
  defeat effects; the due tech-tree work sees that raise's effects, but the already completed
  power and ordinary radar work do not (claim 18, V15).
- The final tick runs fully, but its defeat tests are too late to change the committed outcome
  (claims 8 and 16, V12 part c).
- A surrender order applied in tick *t* destroys objects in tick *t* + 1, since orders take effect
  after every house update of their tick (S-0203 claim 7). Its mark is set in tick *t*, but no
  surrender destruction occurs then. This delay does not apply to alliance orders.

## Edge cases and quirks

- **`quirks.short_game_first_three_base_units`** (claim 2). Original: only the first three positions
  of the base-unit list count, whatever its length. Off: every position counts. A base-unit list
  shorter than three positions makes the original read past its end, which is unspecified; our
  engine treats a missing position as holding no type, and the importer warns (a departure).
- **`quirks.vehicle_buildings_keep_nobody_alive`** (claim 3). Original: a building that counts as a
  vehicle keeps its house alive in the normal test only when it is a refinery-list building. Off:
  every building that counts as a vehicle keeps its house alive. With this switch off, such a building is
  counted in *R* as if it were a refinery-list building: added when it enters the map, taken off
  when it leaves it, counted when marked insignificant and never counted when marked not scored.
  A refinery list shorter than three positions: the original's behaviour is not established (low
  confidence); our engine makes the term *R* count nothing and the importer warns (a departure).
- **`quirks.unscored_vehicle_stays_counted`** (claim 4). Original: a vehicle of a not-scored type
  is added to the on-map vehicle count when it enters the map and never taken off. That positive
  contribution keeps the house alive unless negative contributions (such as nominal survivors)
  cancel the total to *S* = 0. Off: vehicles of
  not-scored types are never counted, like not-scored infantry and aircraft.
- **`quirks.nominal_survivor_count`** (claim 4). Original: a nominal survivor is not added to the
  on-map infantry count but is taken off when it leaves the map, so the count can fall below the
  true number and below 0; a house can be defeated with units on the map, or never be defeated with
  none. Off: a nominal survivor is added and taken off like any infantry unit.
- **`quirks.defeat_during_end_delay`** (claim 9). Original: the local player's house, defeated
  with a win pending, turns it into a loss and restarts the end delay. In our engine the same
  holds for the person-played house; any other player house defeated in the delay loses and
  leaves the delay alone. Off: results and delay fixed at the end of the game. A defeat tested
  in the final tick cannot change the committed outcome with either setting (claim 16).
- **Short game with the bases setting off** (S-0412 claim 11). Bases off removes the starting
  base unit, not the short-game test's other ways to survive: a contender with a counted building
  or an eligible base-unit vehicle supplied by the map or among its budget units still passes
  claim 2. In our engine these in-game outcomes apply only when the setup is allowed: with
  `quirks.short_game_without_bases` off, a setup with short game on and bases off is refused
  (S-0412 claim 11). When none of the contenders has such an object, every tested contender fails at tick 1
  (an observer's house remains set aside). The houses are defeated one after another in house
  order, and the game ends at the first defeat after which no hostile pair remains (claim 10).
  The houses still undefeated then fail in their own updates of the same tick and lose (claim 9),
  so with `quirks.defeat_during_end_delay` on every tested player house loses, all in tick 1.
  With it off, those undefeated at the end retain their wins. This also applies with no
  person-played house. In the original the human's house, first in house order, ends the game
  with its defeat (claim 6, departure); the human's loss in tick 1 is the same. When some
  contenders have qualifying objects, only those without them fail, and claim 6 decides whether
  those defeats end the game.
  - Backing: `measurable` for the original (S-0412 scenario K1, parts b and c); the end test and
    per-house results of our engine are the design rules of claims 6, 7 and 9
  - Confidence: high for the defeat test; medium for the setup cases, as S-0412 claim 11
- **Observers.** An observer's house (S-0009 claim 22) owns nothing and receives no starting
  units. *The original* creates it like the house of any human participant: it is tested and
  counted like any house, and it counts for the original's end condition of claim 6 (the
  departure). It takes the first country of the rules' country list, so unless that country is
  passive its test fails at tick 1; with the observer as the only human participant, that defeat
  leaves no human participant's house among the contenders, and the original ends the game in
  tick 1, the observer being shown a loss. Confidence low; whether the original's skirmish lobby
  can start a game with an observer at all is not established (Open questions), and scenario V10
  records it. *Our engine* (a departure, design choice, no switch): an observer's house is set
  aside (Terms) and is not a player house for this spec. It is never tested and never defeated
  (claim 1), it is left out of every count of claims 6 and 12, it gets no result (claims 7 and 9),
  and surrender and alliance orders it issues do nothing (claims 11 to 14). A game watched by an
  observer runs and ends like any game with no person-played house, at the first defeat after which
  no hostile pair remains (claim 6).
  - Backing: the original's behaviour is `measurable` (scenario V10); our engine's rule is
    `unmeasured` (a rule of our engine, which the original does not follow)
  - Confidence: low for the original; high for our engine's rule, a design choice that follows
    the lead decision of 2026-10-11 (claim 6)
- **Making peace with the last enemy** is refused (claim 12), so an alliance can never be what ends
  the game.
- **Items in production** (claims 2, 4 and 15). A building in production, or finished and waiting to
  be placed, is a counted building in both tests, and a base-unit vehicle in production counts in
  the short-game test, because starting an item creates its object (S-0404 claim 11). No switch:
  it is how the original keeps its counts. How long such an item lives once the factory
  building of its kind is lost is S-0404's (claim 8): when a house's last factory building of a
  kind is removed for good (destroyed or sold), every item that no remaining factory building
  could make is cancelled in the same tick, so it keeps nobody alive after that tick. The
  exception is S-0404's `quirks.defence_line_partial_factory_check`: after the capture (a change
  of owner, not a removal for good) of the last factory building of buildings, a defence item
  goes on and keeps the house alive (variant g of V1).
- **`quirks.defeat_cancellation_skip`** (claim 15). Original: immediate deletions caused by one
  destruction hit can skip later objects, with the exact count in claim 15. Off: deletion costs
  no remaining eligible object its destruction hit. Backing: `measurable` (V11 part d).
  Confidence: medium.
- **Mind-controlled objects of a defeated house** (claim 5) pass to the first house of the
  civilian side when their control ends. No switch.

## Import mapping

| Key | Concept | Parse form | Default when absent |
|---|---|---|---|
| `[AudioVisual]` `SavourDelay` | end-delay field *d* (claim 8), in game minutes | decimal, double | 0.03 |
| `[CombatDamage]` `C4Warhead` | the demolition warhead of the defeat's destruction (claim 5; S-0303, S-0312) | warhead type name | none, as S-0303 (claim 19) and S-0312 give it |
| `[General]` `BaseUnit` | base-unit list (claim 2) | list of vehicle type names | empty |
| `[AI]` `BuildRefinery` | refinery list; its third position is the refinery-list building type (claim 3) | list of building type names | empty |
| any object type, `Insignificant` | marked insignificant (claim 4) | boolean | no |
| infantry, vehicle, aircraft or building type, `DontScore` | marked as not scored (claim 4) | boolean | no |
| building type, `UndeploysInto` | undeploy type (Terms) | vehicle type name | none |
| vehicle type, `ResourceGatherer` (read on the vehicle type that a building's undeploy type names) | resource-gatherer flag (Terms) | boolean | no |
| infantry type, `Nominal` | marked nominal (Terms, *nominal survivor*) | boolean | no |
| `[MultiplayerDialogSettings]` `AlliesAllowed` | allies-allowed flag (claim 14) | boolean | no |
| `[MultiplayerDialogSettings]` `AllyChangeAllowed` | ally-change-allowed flag (claim 14) | boolean | yes |
| building type, `Foundation` | one-cell foundation (Terms; S-0108) | foundation name | as S-0108 |
| country section, `MultiplayPassive` | passive house (claim 1; S-0009) | boolean | as S-0009 |
| object type, `Cost`; house/country cost factors | current owner-adjusted type price used for score (claim 22) | integer credits and multipliers, per S-0404 Import mapping | exactly S-0404's defaults and price formula; no score-specific price default |
| object type, `Points` | separate point-value field, not used by the skirmish score of claim 22 | integer | 0 |
| resource values, country income factor and refinery bonus inputs | credited resource amounts used by claim 22 | exactly S-0402 Import mapping | S-0402's defaults; resource credit value and country income factor do not multiply score |
| Map `[Tags]`, `[Triggers]`, `[Events]`, `[Actions]` entries | assigned tags, their owner countries, conditions and actions (claim 18) | comma-separated records with integer, ID and type-name items, exactly as S-0011 Import mapping | absent entries create no definitions; omitted record fields and unresolved references follow S-0011 Import mapping |

Claim 17 uses the existing type-exclusion and base-unit-list rows above; zero health and
object deletion introduce no additional imported field. Its cancellation input is owned by
S-0404, and any death-effect input by S-0312.

The creation-order destruction and the house-update cutoff of claims 15 and 16 use no INI
settings; the cancellation exception is controlled only by its quirk switch. Their warhead and end-delay field use the existing rows above. Cleanup and
its maximum-player-count input remain owned by S-0009 claim 28 and its Import mapping.

Claim 18 adds no timing input: the 8-tick interval is an engine constant, not a rules setting.
Tag assignment, type-count tests and variables retain S-0011's import rules.

The civilian side of claim 12 is the side named Civilian; the original finds it by that fixed
name, so the name is an engine constant, not stock data.

Claims 19–23 use the existing not-scored, insignificant and owner-country mappings and
S-0404's cost mapping. Built, kill and loss counts start at zero and have no imported
initial-total field. The score bonus and the presentation clock have no INI setting.
S-0701 owns the announcement-entry bindings and audio imports used by claim 5.

## Measurement scenarios

All scenarios use the measurement setup with a skirmish on a small test map, the human against
computer players, at game-speed index 2 and 800 × 600, with the director and mods off except for the named test
files. Run each deterministic variant once; repeat void runs. Computer players are bystander computer players
(glossary), kept away from the observed area; a run is void when a computer player's own defeat,
alliance or attack changes the events the scenario observes, and is then repeated. Every scenario
in which the human gives an alliance order (V5 part c, V9) uses a test rules file that sets the
allies-allowed and ally-change-allowed flags on (claim 14).

Ticks are read from the log of the recorder (glossary, *measurement setup*), which stamps each
entry with the tick number; the defeat message, or the message of a test map trigger that fires on
the house's destruction, is matched against that log by a screen capture taken at every tick
under scripted input. "Defeat tick" is the tick of that message. "Defeated at the first house
update after the state is reached" means: in the tick in which the state is reached, or in the
next tick when the state is reached after the house's update of that tick. Each scenario is
falsified if any of its variants or parts gives other than its expected result.

**V1: the short-game test.** Setup: short-game setting on; test rules file with a base-unit list
of four test vehicle types B1 to B4. The human keeps only: (a) one building; (b) one B1 vehicle,
(c) one B3 vehicle; (d) one B4 vehicle; (e) one B1 vehicle inside a test naval transport,
(f) only infantry, aircraft and tanks (types outside B1 to B4). Achieve each state by selling or
deleting the rest with test triggers. Observable: whether the human is defeated, and the defeat
tick. Expected: (a), (b), (c) and (e) alive; (d) and (f) defeated at the first house update after
the state is reached.
- *Variant g (an item in production keeps the house alive).* The human owns one factory building
  of buildings and nothing else that counts (no other building, no B1 to B3 vehicle), and has a
  defence-building type in production on the defence line, its tech-tree prerequisites met by
  nothing the test removes. Run 1: a test enemy engineer captures the factory building. Run 2: a
  test trigger destroys it instead. Observable: whether and when the human is defeated, and
  whether the item is still in production. Expected: run 1, the factory building leaves the human
  and the defence item stays in production (S-0404 claim 8,
  `quirks.defence_line_partial_factory_check`), so the human is not defeated while the item is in
  existence, and is defeated at the first house update after the item is cancelled or is placed
  and then lost; run 2, the item is cancelled in the tick of the destruction (S-0404 claim 8: a
  removal for good cancels every line whose item no other factory building could make), so the
  human is defeated at the first house update after that tick. A building in production on the
  ordinary line is no variant of this: with the factory building gone for good it is cancelled in
  either run.
- *Variant h (marked types).* Test rules file marking B1 as insignificant and B2 as not scored.
  Run 1: the human keeps only one B1 vehicle. Run 2: only one B2 vehicle. Expected: defeated at the
  first house update after the state is reached in both runs (such a vehicle never counts in the
  short-game test).
- *Variant i (a building that counts as a vehicle).* The human keeps only one building of a test
  type that counts as a vehicle (Terms). Expected: defeated at the first house update after the
  state is reached.
- *Variant j (excluded buildings).* Run 1: the human keeps only one otherwise counted building
  whose test type is marked insignificant. Run 2: the same, marked not scored instead.
  Observable: defeat and its tick. Expected: defeated at the first house update after the state
  is reached in both runs; surviving that update falsifies the exclusion.

**V2: the normal test.** Setup: short-game setting off. The human is reduced to exactly one of:
(a) one infantry unit on the map; (b) one test transport vehicle on the map with the human's last
infantry unit inside; (c) one infantry unit garrisoned in a neutral occupiable building; (d) one
building of a test type with an undeploy type and a one-cell foundation; (e) one deployed
resource-gathering vehicle that has become a refinery-list building (test rules file listing its
building type third in the refinery list), then the same with that type moved to the first
position; (f) nothing at tick 0 (a test map and setup giving the human no starting units: bases
off, unit count 0, with a test rules file setting the minimum unit count to 0). Observable:
whether and in which tick the human is defeated. Expected: (a) and (b) alive; (c) alive exactly
when the garrisoned building now belongs to the human and its type is neither insignificant nor
not scored, defeated otherwise; (d) defeated; (e) alive in the first run and defeated in the second
run; (f) defeated in tick 1, not in tick 0.
- *Variant g (not-scored vehicle).* Test rules file marking a test vehicle type as not scored. The
  human's only units are one such vehicle and one infantry unit; the vehicle is destroyed, then the
  infantry unit. Expected: the human is never defeated.
- *Variant h (nominal survivor).* Test rules file marking the human's building survivor type as
  nominal. The human owns one tank, kept away from all fighting, and one building that a test
  trigger destroys so that survivors come out. Wait until the destroyed building has left play
  before killing survivors in either run. The recorder log notes the tick in which each
  survivor leaves the map. The count after each event is the table of claim 4 applied to those
  ticks: the tank is one on *V*, a nominal survivor adds nothing to *I* and takes one off when it
  leaves the map, and the destroyed building leaves *B* when it leaves play.
  Run 1 (exactly one survivor comes out; the test kills it). Expected: defeated at the first house
  update after the survivor leaves the map, while the tank is still on the map.
  Run 2 (two survivors come out and are killed by one area weapon that does not reach the tank).
  Expected, from the recorded ticks and house-update boundaries: if a defeat test sees exactly
  one survivor already off the map, the human is defeated there (*S* = 0). If both leave before
  any test can see that intermediate state, the human is never defeated, even after the tank is
  then destroyed (*S* goes from 1 to -1 and then to -2). Deaths in different ticks do not alone
  guarantee a test between them. A result differing from the applicable case falsifies the
  counting rule, once the nominal-survivor condition is known to hold.
  Low confidence: if no survivor is created as a nominal survivor, the runs show nothing (the
  tank alone keeps the human alive, and no defeat comes after the survivors die) and the building
  condition must be found first; that result is recorded as "no nominal survivor", not as a
  falsification.
- *Variant i (marked types).* Test rules file marking one infantry type as insignificant and one
  as not scored, and one building type as insignificant. The human is reduced to exactly one of:
  (1) an infantry unit of the insignificant type on the map; (2) an infantry unit of the
  not-scored type; (3) a building of the insignificant type. Expected: (1) alive, (2) defeated,
  (3) defeated, each at the first house update after the state is reached for the defeats.
- *Variant j (a unit taken by mind control).* Run 1: the human's only unit is one vehicle. Run 2:
  the human's only unit is one infantry unit. In both runs the human owns nothing else that counts,
  and a computer player's mind-controlling unit takes that unit. Expected: defeated at the first
  house update after the control begins, in both runs (the unit leaves the human's on-map count
  by the change of owner).
- *Variant k (a change of owner by a test map trigger, S-0011 claim 24).* The human owns one
  infantry unit X on the map and nothing else that counts.
  Runs 1 to 3 (the gaining side): a test trigger gives the human one object of a computer player
  that is on the map: an infantry unit (run 1), a vehicle (run 2), an aircraft in the air, kept
  away from every building it could land at (run 3). A second test trigger then destroys X.
  Expected: the human is not defeated in any of the three runs while the given object is on the
  map. A defeat at the first house update after X's death falsifies the gaining side of the
  change-of-owner rule.
  Runs 4 to 6 (the losing side): the human owns instead only one infantry unit (run 4), one
  vehicle (run 5) or one aircraft in the air (run 6), and a test trigger gives that object to a
  computer player. Expected: defeated at the first house update after the change of owner.
  Run 7 (a not-scored vehicle, losing side): test rules file marking a test vehicle type as not
  scored. The human owns X and one vehicle of that type that has been on the map; a test trigger
  gives the vehicle to a computer player, and a second test trigger then destroys X. Expected:
  the human is never defeated (the vehicle stays on the human's *V*). A defeat falsifies it.

**V3: passive houses.** Setup: test map whose overrides make a computer player's country passive
for the late phase (S-0009 claim 33); the computer has no units. Observable: whether it is ever
defeated and whether the game ends when the human destroys the only other enemy. Expected: never
defeated; the game ends with a win.

**V4: what defeat destroys.** Setup: the human owns a test transport vehicle with infantry
inside, a garrisoned building, and a mind-controlling unit that holds one of the computer's
vehicles. The human surrenders (claim 11 uses the same destruction as claim 5).
- *Part a.* Observable: which objects die, whether passengers or occupants come out, and what
  happens to the controlled vehicle. Expected: every human object dies in one tick; no passenger
  or occupant comes out alive; the controlled vehicle is not hit (it goes back to the computer
  when its controller dies).
- *Part b.* A computer's mind-controlling unit holds one of the human's vehicles when the human
  surrenders. Run 1 with a test rules file that puts the neutral and special countries on the
  civilian side; run 2 with one that gives them a side other than the civilian side. Expected: run 1, the vehicle is not hit, and when
  the controlling unit is then destroyed it passes to the first house of the civilian side in house
  order; run 2, the vehicle is destroyed with the human's objects.
- *Part c (the hit's markers).* The human owns a test vehicle type, a test infantry type and a
  test building type, one of each at partial health, in a test rules file that sets every armour
  multiplier of every warhead to nothing, so that no ordinary hit could hurt them. The vehicle's
  type is one that leaves a crew when an ordinary hit destroys it (checked first in a run that
  gives the vehicle ordinary damage under stock multipliers). Observable: whether the three objects die, whether a crew comes out of the
  vehicle, and the kill counts of every house on the result screen. Expected: all three die
  although no ordinary hit could hurt them (damage that ignores defences); no crew comes out of
  the vehicle (the no-escape marker); no house's kill count rises through the defeat (no attacker,
  no attacking house). The kill counts are the only observable of the last two; if the result
  screen does not list them, that item is `unmeasured`.

**V5: the end test.**
- *Part a.* Human and computer A on one team, computer B and C enemies. Destroy B, then C.
  Expected: the game ends with a win right after C's defeat, A still alive.
- *Part b.* As part a, but the human's house is destroyed first while A lives. Expected: the game
  ends with a loss at the human's defeat (the original's condition that our engine drops, claim
  6, departure; our engine would play on until A or the enemies fall).
- *Part c.* Human and computers A and B on no team; during play the human makes peace with A (one way),
  then destroys B. Expected: the game does not end (A does not ally the human). The run is void if
  A allies the human back (claim 12, the original's computer-player reaction).

**V6: the end delay.** Setup: test rules file setting the end-delay field to 0.03 (and in a second
run to 0.1). Observable: ticks from the last enemy's defeat to the tick at which the result screen
appears, from the recorder log. Expected: 26 in the first run and 90 in the second (the stored
double of 0.1 lies just above 0.1, so its product with 900 lies just above 90). The wait for the
announcement is real time and takes no tick (claim 8).
- *Part b (the person-played house is defeated in the delay).* Human and computer A allied, a
  computer B at war with both; the end-delay field is raised in a test rules file (0.1, 90 ticks)
  so that there is room. B is destroyed, the game ends with a pending win, and the human's last
  object is killed by a delayed explosion 30 ticks later (the human's defeat is then at tick *t* +
  30, A undefeated). Expected: a loss for the human, game over at tick *t* + 30 + 90 (a game over
  at tick *t* + 90 would mean the delay was not restarted).
- *Part c (an AI house is defeated in the delay).* As part b, but the human survives and a test
  trigger destroys A's last object 30 ticks after B's defeat. Expected: game over at tick *t* + 90
  with the human's win, not at *t* + 120. A's own result (a loss in our engine) cannot be observed
  in the original and is not checked.
- *Part d (the end through the human's own defeat).* The end delay is 0.1 as in part b. Human and
  computer A allied, computer B at war with both; the human's last object is killed while A and B
  live, and a test trigger destroys all of A's objects 30 ticks later. Expected: game over at tick
  *t* + 90 with a loss for the human (the later defeat restarts nothing).
- *Part e (shortest delay and extreme conversion).* Repeat the basic last-enemy defeat with
  named test end-delay values of 0, −1 and 100000000000000000000 game minutes. Observable:
  ticks from the defeat to the end of tick advancement, followed by the result screen.
  Expected: one more tick in each run, since *D* is zero, negative or converted to zero
  respectively. Zero further ticks or two or more falsifies the minimum-delay rule. This tests
  the zero-result extreme, not every intermediate wrapped value.

**V7: same-tick wipe-out.** Setup: the human and the last enemy each own one infantry unit standing
next to a test building that a timed trigger destroys with a large area-damage weapon killing both
in the same tick. Observable: the result and the tick of each defeat message. Expected: a loss for
the human, with both defeat messages in the same tick. Capture audio as well: each defeat
message names the corresponding house and has a defeat announcement. A missing message or
announcement for either non-observer house falsifies that presentation claim.

**V8: surrender.** Setup: the human surrenders from the options menu at tick *t*. The human has no
item in production, and no not-scored vehicle or nominal survivor of the human has been on the map
in the game. Observable: the tick at which the human's objects die and the defeat tick. Expected:
objects die in tick *t* + 1, defeat in the same tick, or later only when a destroyed object stays
in play (claim 11); a defeat later with no such object falsifies it.
- *Part b (a pending result).* The end delay is raised (0.1 in a test rules file). The human and A
  are allied against B; B is destroyed and the human surrenders in the delay. Expected: nothing
  is destroyed, the human's result stays a win, game over at the original tick (claim 11).

**V9: alliance orders.** Capture the player list, message area and audio during parts a and b.
Expected presentation: successful peace orders towards B and C each produce an alliance message
and announcement; the refused order towards D produces neither. Part b produces a break message
and announcement. Missing or extra presentation falsifies these presentation rules.
- *Part a.* Three computer players B, C, D at war with the human. The human makes peace with B
  while a human unit is attacking a unit of B, then with C, then tries D. Expected: B and C allied
  by the human (the attacking unit stops, and the human's units ignore B and C from then on), the
  order towards D refused (D is the last enemy). B's units still attack the human (the run is void
  if B allies the human back).
- *Part b.* Human and B mutually allied through lobby teams; the human breaks the alliance.
  Place one human unit and one armed unit of B within each other's range in the observed area,
  away from other targets. Observable: whether a normal human attack order against B is accepted
  and whether B's unit automatically acquires and fires at the human after the break. First
  verify in an enemy-house control run that B's unit acquires that target in this setup.
  Expected: the human's order is accepted and B's unit fires; a refusal of the human's order or
  B continuing to ignore the target in the validated setup falsifies the two-way break.
- *Part c.* As part a, with a test rules file setting the allies-allowed flag off (run 1) and the
  ally-change-allowed flag off (run 2), the other flag on; in both runs the human and a computer
  player E are also put on one lobby team. Expected: in both runs the order to ally with B cannot
  be given (no alliance forms), and E stays allied with the human. Run 3: both flags on, and a test
  map that sets the fixed-alliance mark; expected: the order to ally with B can be given and forms
  an alliance (the mark is cleared once the map has loaded).

**V10: an observer in the original.** Setup: the skirmish lobby with the one human participant set
to observe, and two computer players on separate sides; short-game setting off. Observable:
whether the game can be started, and if so the tick at which the result screen appears and the
result shown. Expected (low confidence, Edge cases, *Observers*): the game ends in tick 1, so the
result screen appears in tick 1 + 26 at the absent-key end delay (Formulas), showing a loss; no
defeat message is shown for the observer's house (claim 5). A game that runs on past that tick
falsifies the reading. If the lobby offers no way to start a skirmish with an
observer, that is recorded as "no observer in a skirmish" and is not a falsification; the Edge
case's description of the original then concerns no game that can be played.

**V11: creation order and off-map hits (claim 15).** Setup: small test map, normal game, named
map overrides defining two vehicle and two building types with distinct debris draw ranges, with
other random death effects disabled. Keep an enemy bystander away. Create the human's objects
in a recorded interleaving of kinds through production and placement, at separated cells so
deaths cannot damage one another. Procedure: surrender. Tool and observable: read-only health,
death-message and scenario-stream recorder, plus scripted input. Repeat for three recorded seeds
with discriminating draws. Expected: draw allocation matches creation order across kinds and
there is no extra draw from the sweep. Any kind-grouped or map-entry allocation falsifies this.
- *Part b, re-entry and passengers.* Produce infantry P before transport T, then board P into T.
  Make a later infantry Q. Record P's health during surrender. Expected: P receives its hit while
  off the map before T's hit, not only at T's death; boarding does not put P after Q. Reverse
  creation order in a control: T's passenger-death rules may kill P before P's place, without
  a second death or extra set of draws. Repeat with an absorbing building instead of T.
  Garrison occupants have the same inclusion, with their death consequences owned by S-0312.
  Survival until the later carrier death in the first run, or a second death in the control,
  falsifies the rule. The recorder must sample within the destruction interval: an end-of-tick
  disappearance alone cannot distinguish these cases.
- *Part c, production.* Start and hold infantry item P. Build a replacement infantry factory
  after P's start, then sell the original factory without cancelling P. Surrender while P is
  still held. Expected: P's health receives the killing hit before the replacement factory dies.
  Repeat with a finished building awaiting placement and a replacement factory created after
  its production start. Unchanged item health until factory-loss cancellation falsifies off-map
  inclusion. Mere disappearance confirms nothing: cancellation would also delete the object.
  A run where an earlier factory loss cancelled the item is void.
- *Part d, multiple earlier cancellations.* Hold one ordinary-building item and one defence
  item. Deploy a replacement factory of buildings after both items were started, then sell
  the old factory without cancelling either. Create two test infantry units after the new
  factory. Surrender and record immediate item deletions, the infantry health and creation
  order. Expected if the factory's death immediately deletes both earlier items: the first
  following object in combined creation order is skipped and can survive, while the second
  eligible one is hit. Control with only one earlier item: neither is skipped. If another
  object occupies the intervening place, the skipped place belongs to it instead. A run
  without the two immediate deletions is inconclusive, not confirmation. No skip despite
  those confirmed deletions, or a different number of skipped places, falsifies the formula.

**V12: house-update timing and the final tick (claims 8 and 16).** Use the common setup and named
map overrides. Tool: read-only recorder of power totals, radar availability, build icons, object
health, defeat messages and order destinations, with scripted input. Repeat each deterministic
variant once; void unintended computer attacks or defeats.
- *Parts a and b.* Normal game. Human owns a power plant, radar building, production buildings
  and a not-scored vehicle that entered the map; its stale count keeps the house undefeated
  after surrender. Use no nominal survivors. Wait for pending rechecks and surrender in tick
  n − 1. Expected in n: objects receive hits, no defeat, power totals and ordinary radar still
  show pre-destruction values, but the due tech-tree recheck removes offers whose factories or
  prerequisites actually left the map. In n + 1 power and radar reflect the losses. Record
  actual removal so lingering buildings cannot masquerade as stale totals. Same-update power
  or ordinary radar changes, or a due tech-tree change postponed to n + 1, falsify the ordering.
  Control with an ordinary vehicle instead: at defeat radar is forced on in n, not postponed.
- *Part c, boundary.* Human and A allied against B. Use V6's test 90-tick end delay. Defeat B
  at t. A timed map action removes the human's last counted object at e = t + 90 before houses
  update; A survives. Expected: defeat message at e, a win on the result screen, no restart,
  last tick e. Control with removal at e − 1: a loss and fresh delay ending at e − 1 + 90.
  A loss or restart in the e run falsifies the cutoff. For full-tick execution use a separate
  run with the human surviving: scripted input queues a move order in e. Expected: its unit's
  destination changes in e although no e + 1 runs. Failure to apply it falsifies full-tick
  execution. These tests distinguish commitment from stopping simulation at the tick's start.

**V13: cleanup-removed houses (claim 1).** Use S-0009 M16's setup with no computer slots, the
neutral country non-passive and the special country passive. Confirm the neutral house itself
was removed, not merely emptied. Tool and observable: read-only house-state recorder and defeat
messages through tick 1. Expected: no update, defeat, result or contender/enemy membership for
that removed house. A test counting it or a defeat message naming it falsifies this. Repeat with
cleanup prevented as in S-0009 M16 part G: only a surviving non-passive house can be a contender.
A lobby unable to start the no-computer setup makes this run unavailable, not a defeat-test
result. S-0009 claim 28 already answers removal of the house itself; this scenario checks its
victory-rule consequence. Run each available deterministic variant once.

**V14: a killed item and the defeat counts (claim 17).** Use the common skirmish setup,
normal game, with a remote enemy bystander and V11 part c's named test map and production
ordering. Start and hold a counted building item P before creating the replacement factory
of buildings, then sell the original factory without cancelling P. Tool: read-only recorder
of P's health and continued existence, the house's counted-building count and defeat state,
plus scripted input. Procedure: surrender and sample during the destruction interval, between
P's hit and the later replacement factory loss, then through P's cancellation. Expected:
while P still exists at 0 health, its contribution to *B* is unchanged; deletion removes that
contribution. Distinguish the factory's own contribution and deletion from P's. An end-of-tick
count alone does not distinguish death from cancellation. In a control use V11 part c's held
infantry item: it contributes nothing to the on-map infantry count at either point. A count
reduction at P's killing hit while P still exists falsifies claim 17. If P is deleted by its
own hit, the retained-item part is inconclusive and that deletion is recorded for the death
and production specs, not treated as proof of continued production. Repeat each variant once.
Void an earlier cancellation of P or an unintended bystander intervention. This scenario
checks counts only; progress, draws, refunds and release results belong to the separate
Q-0193 questions.

**V15: owner-house raises among house topics (claim 18).** Use the common setup and a
named test map, with normal game and separate countries for the human and two remote
bystanders. Use repeating tags with an any-event and the relevant condition together, and
record every firing with its tick. The ordinary per-tick poll remains enabled: distinguish its
firings from the additional owner-house firing in multiples of 8, as in S-0011 T20. Tool:
scripted input, screen capture and a read-only recorder of type counts, trigger variables,
defeat state, power totals, radar, build offers and issued cancellation/release orders.
Run each deterministic part once; void unplanned bystander actions.
- *Part a, surrender and defeat.* Give the human only an on-map infantry unit, with no crew,
  survivors, production or stale count contributions. Apply surrender in tick 7. A human-owned
  tag tests absence of that infantry type and sets a variable, which a later house's tag tests.
  Expected: no such firing before destruction; in tick 8 the owner-house raise fires after
  destruction and after the recorded defeat, and the later house sees the set variable in
  its same-tick raise. Use a named test end-delay field long enough to include tick 16.
  In a separate counter run, a repeating any-event tag owned by the human has its additional
  owner-house firing in tick 8 and again in tick 16, despite defeat. Missing either additional
  firing, or seeing the infantry's pre-destruction count at the tick-8 house raise, falsifies
  the rule. Record phase samples within the update: an end-of-tick variable alone does not
  locate the defeat test relative to the raise.
- *Part b, tag-caused loss.* Keep the human alive with a remote ordinary infantry unit.
  In tick 16, after the ordinary poll, a unit enters a tagged cell during the turn pass and
  its entered-by trigger sets a gate variable. Script the move and void any run where arrival
  is not in tick 16. A human-owned tag tests that gate and any-event together and destroys a
  power-producing prerequisite building and the radar building. Use named test data making
  those removals immediate and without secondary effects. Before tick 16, have
  an offer depending on the prerequisite with its ordinary factory still present, and a
  held production item requiring it. Record the due marks rather than assuming a recheck.
  Expected: the tick-16 raise removes the buildings; power totals and ordinary radar still
  reflect them for that update, whereas the now-due tech-tree recheck removes the offer and
  issues its cancellation in tick 16. Power and radar reflect the losses in tick 17. A tick-16
  power/radar recheck of those losses or deferral of the due tech-tree work to tick 17 falsifies
  the ordering.
- *Part c, no second defeat test.* Give the human only ordinary infantry, and remove it with
  a human-owned tag in its tick-16 house raise, using part b's turn-pass gate setter on that
  infantry's cell entry. Expected: removal
  in tick 16 but defeat in tick 17, unlike surrender in part a. Defeat in tick 16 falsifies this
  consequence. V13 supplies the cleanup-removed-house control: a removed house has no raise
  at all, not merely a raise after the remaining houses.

**V16: result rows and built totals (claims 19–20).** Use the common setup, a remote
opponent and a counted building to keep the human alive. Record event ticks, ever-built
counts with a read-only recorder, and the final screen. Run separate controls: starting
objects only; one released infantry, vehicle and aircraft; one started-and-cancelled item,
one completed but held item; one placed building; a free refinery vehicle; a clone; a crate
unit; a deploy/pack-up pair; capture; a trigger-created object. Expected built deltas:
1 per eligible factory release, 0 for the other listed sources. Repeat one release with
named test type flags: not scored gives 0, insignificant alone gives 1. A death or sale
never decreases built. Record the final screen in a run where a qualifying event occurs
in a lengthened end delay: it is included. Expected rows omit passive, observer and removed
houses, retain defeated player houses, and show kills, losses, built and score but no
harvested-credits figure. Any different delta, extra total or excluded defeated player
falsifies the corresponding claim. V12 part c also compares the local committed win with
the original's absence of a winner-score bonus after that final-tick defeat. Run each realizable control once; void unplanned
bystander events. A clone or crate control without that creation is inconclusive.

**V17: kills, losses and event score (claims 21–22).** Use the common setup and named test
map placing isolated victims, with every death effect and survivor disabled. Keep a remote
enemy alive. Tool: scripted input, screen capture and read-only score/count recorder.
Record baseline and delta for destruction of one infantry, vehicle, aircraft and building,
then for capture and a mind-control/return pair. Expected: each eligible destruction adds
1 kill to the attacking object's current house and 1 loss to the victim's current house.
No-attacker surrender gives losses only; capture/control/return each gives the gaining
house 1 kill and the old owner no loss. Test a not-scored victim (neither death counter)
and insignificant victims (building excluded, other kinds included), then an insignificant
building capture (gain kill included). Compare normal sale, boarding and deploy/pack-up
controls: no death counter merely from removal. For passenger deaths compare S-0312's
carrier outcomes with the individual passenger death records, not carrier disappearance alone.

For score, use an invented type price whose owner-adjusted value is recorded before each
event, and rookie/veteran/elite victims. Expected destruction deltas: 1, 2 and 3 times that
price respectively, independent of attacker's rank; an allied kill gives 0 score but still
a kill. Not-scored destruction gives 0, insignificant-building destruction gives its price.
Ownership gain gives the old owner's current price regardless of rank, alliance or those
flags. Repeat with a different separate point-value field: no score difference. Sample the
running score before the winner bonus; comparing final scores alone is insufficient. Any
different delta falsifies the applicable row. Run each deterministic variant once. Void
unplanned attacks, ownership gains, income or passenger escape masking the named death.

**V18: resource score and winner bonus (claims 22–23).** Common setup, one refinery and
harvester, a remote enemy, no combat or other income while recording each unload. Named
map overrides supply a known credited resource amount; repeat with a different resource
credit value and country income factor, then with a purifier bonus. Expected score update:
trunc(previous score + 5 × credited amount), independently of credit value and income
factor; a purifier bonus contributes a separate update. Use a fractional bonus to distinguish
separate truncation from accumulated fractions. Cash-crate, refund and initial-credit
controls give no score merely from credits. For final score, lose in one run and win in
another with a recorded running score. Expected loss score: max(running score, 0).
Expected win score: the exact formula in claim 23, not a constant victory multiplier.
Record the scenario stream immediately before and during score preparation. With multiple
undefeated listed houses, ranged requests follow house order; a zero-score winner consumes
no raw draw. Run ten recorded seeds and compare the exact drawn values, not just their range.
A mismatching contribution, out-of-formula displayed score or other stream/order falsifies
the rule. Named negative-score data checks clamping; ordinary score growth need not become
negative. Storage-only conversions are conditional diagnostics until a way to fill storage
in a skirmish is found, not a promised confirming run.

**V19: duration (claim 23).** Common setup, fixed-tick defeat trigger, recorder of tick
numbers, active wall-clock intervals and the final duration string, with screen capture.
Run at speed indices 2 and 6, then run with a 30-second options-menu pause, with a 30-second
loss of focus, and with a named timed-freeze trigger. Include a named lengthened end delay.
Expected whole-second total is accumulated active 16-ms clock units divided by 60 and
truncated, expressed as hours/minutes/seconds. The same tick count at slower speed has a
longer duration; menu and focus pauses add none, freezes and the end delay add active time.
Record the clock's stopping point relative to the last tick and the result announcement:
any announcement wait before leaving play is included, later result-screen waiting is not.
An active run near 60 seconds distinguishes the 960-ms displayed second from a true wall
second. Compare values allowing only clock-boundary sampling uncertainty, not a 4-percent
scale change. A tick-derived total, counted menu pause or different scale falsifies the rule.
The 99-hour cap is checked by a read-only trace of a naturally long run if practicable.
Short runs do not confirm that cap. No clock is altered by the recorder.

**V20: announcements after local defeat (claim 5).** Common setup with two hostile
bystanders and named test map end-delay long enough for a later enemy superweapon launch
and detection request. Capture audio and record each request and tick; a matched pre-defeat
control verifies that its entry is present and audible. Expected: the launch and detection
announcements muted after local defeat. For alliance-formed and enemy-alliance-formed
requests, use a recorded computer alliance during that same delay, voiding a run without
such a request; it too is muted, while an otherwise legal alliance-break request is not
muted merely by defeat. These conditional alliance controls require observing the original
computer player's orders, not issuing human input on its behalf. Any audible muted-role
entry after defeat falsifies the rule. Muting throughout our longer observation period is
our presentation requirement and cannot itself be measured in the original.

## Changes for other specs

- S-0202: add the post-game winner-score ranged requests of claim 23, on the scenario
  stream in listed-house order, with no raw draw for equal bounds; simulation has stopped.
- S-0407: claim 10's ever-built count excludes not-scored released types, but not a type
  marked only insignificant; this same total feeds claim 20's end-screen built statistic.
- S-0701: claim 36 currently gives announcement roles, not the literal entry-name bindings
  described in the supplied report. Restore those bindings in Import mapping under the
  lead's 2026-10-14 convention. Its post-defeat muting agrees with claim 5 here; V20 adds
  an end-delay diagnostic without claiming an already measured result.

## Open questions

- The duration clock's outcome across host-clock wrap in an exceptionally long game is
  not established. Claim 23 and its duration formula cover elapsed intervals before wrap.

- Q-0193 questions 1, 2, the progress/completion/release/placement parts of question 3, and
  question 4 remain for S-0312 and S-0404. Claim 17 answers only this spec's defeat-count part
  of question 3. The existing creation-order hit and factory-cancellation rules do not establish
  the killed item's subsequent death effects or production-line lifetime. At hand-off,
  reopen Q-0193 and retain claim 17 as the partial defeat-count answer only, not closure
  of its death, deletion, production, release, placement or refund questions.

- Whether the original's skirmish lobby can start a game with an observer, and whether the game
  then ends in tick 1 as Edge cases, *Observers* reads it (scenario V10; our engine's rule does not
  depend on the answer).
- Which buildings mark their survivors as nominal survivors (claim 4; a condition set when the
  building is created, not identified).
