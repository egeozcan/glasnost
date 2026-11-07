---
id: S-0601
title: Mind control
topic: combat
revision: 23
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-07
measurements: []
requests: [Q-1150, Q-1200, Q-1201, Q-1202, Q-1203, Q-2717]
supersedes: []
---

# S-0601: Mind control

## Summary

How one object takes another over with a mind-control weapon, and how that ends. The spec covers
the controller's capacity, unlimited controllers and their overload pulses, which targets can be
taken, what a takeover changes, what releases a controlled object, and who the original owner is.
It also gives the permanently controlled state that psychic domination leaves behind. It covers the
Yuri Clone, Yuri Prime, Psychic Tower and Mastermind, together with controlled buildings, the
controller changing hands or being removed, the victim's death, boarding, deploying and packing up,
and grinding. It is the producer of the *controlled* and *permanently controlled* states that other
specs consume. These claims cover skirmish only.

**What this spec does not restate.**
- **Where the firing check places the takeover test** among its answers: S-0301 claim 9 (the
  capture-tests rank). This spec gives only the test itself (claim 3).
- **Searches and retaliation of a controller at capacity:** S-0309 claims 2 and 18; the candidate
  tests of S-0309 claims 7 and 9 to 12. This spec gives only when a controller is *at capacity*
  (claim 2).
- **The guard reset and what it does to the behaviour slots:** S-0215 claims 9 and 10.
- **What a change of owner does to a house's counts, defeat, kills and losses:** S-0414 claims 4,
  5 and 21. What it does to experience: S-0311 claim 13. To spawned missiles and aircraft: S-0506
  claim 13. To a disguise: S-0516 claim 34. These consumers run on every owner change that this
  spec causes, in both directions.
- **What protection refuses:** S-0513 claim 27.
- **The psionics immunity**, its defaults and its other consumers: S-0610 claim 1.
- **Psychic domination itself** (the superweapon's area, its damage and which objects it reaches):
  S-0612 claims 13 and 14. This spec gives only the permanently controlled state it sets (claim 13).
- **Boarding a transport:** S-0507 claims 4 and 6. **Garrisoning:** S-0508 claim 1.
- **Erasing hits and the release of an attached attacker:** S-0514 claims 2 and 14.
- **When a building is not in operation:** S-0405 claim 9.
- **The charge shot:** S-0518 claim 11; planted bombs and demolition charges: S-0519 claims 2
  and 14.
- **The control pips and their overload highlight:** S-0111 claim 13. The damage of an overload
  pulse is claim 16 here.
- **Grinders:** S-0603 claims 6, 8 and 9. Consumption silently frees the controller's slot and
  values the victim as an object of its current owner, the controlling house (claim 12).
- **Bio reactors and other absorbing buildings:** S-0605 claim 7. Accepted absorption releases
  a controlled entrant by claim 9 before it goes inside as its original owner's object.

## Terms

- **Takeover warhead.** A warhead whose mind-control flag is set (Import mapping). Its hit takes the
  target over instead of damaging it (claim 5).
- **Controller.** An object that holds a *control record*. It gets one when it is created, if
  its type's primary weapon has a takeover warhead (claim 1). The objects it holds are its
  *victims*, kept as an ordered list in the order they were taken.
- **Control capacity.** The number of victims a controller may hold, fixed when the record is
  created (claim 1).
- **Unlimited controller.** A controller whose primary weapon has the unlimited-control flag.
  Its capacity test never refuses a target, and it suffers overload pulses (claim 16). A capacity
  of exactly 1 still replaces the previous victim (claim 7).
- **At capacity.** A controller that is not unlimited and holds at least as many victims as its
  capacity (claim 2).
- **Controlled.** The state of an object held by a controller. While it lasts the object belongs to
  the controller's house at takeover and its *original owner* is remembered (claim 6).
  A later controller owner change can leave those houses different (claim 11). It ends with a *release*
  (claim 9) or with a silent drop (claim 12).
- **Permanently controlled.** The state that psychic domination gives an object. It is never
  cleared, has no controller and no original owner (claim 13).
- **Mind-controlled.** Either controlled or permanently controlled. This is the condition that
  S-0214, S-0507 claim 4 and the other consumers test (claim 13). One consumer reads only the
  *controlled* half: the construction-yard deploy readiness (claim 15).
- **Original owner.** For a controlled object, the house that owned it at the moment of takeover.
  The *original-owner query* is defined for every object by claim 14.
- **Release.** The end of control in which the victim goes back to its original owner, with the
  effects of claim 9.
- **Takeover test.** The test of claim 3 that decides whether a controller can take a given
  object now.
- **Overload pulse.** The periodic self-damage of an unlimited controller (claim 16).
- **Link line.** The line drawn from a controller to a victim (claim 17).
- **Control ring.** The animation attached above a controlled object, or above a permanently
  controlled one (claim 18).

Glossary terms used: *object*, *house*, *local player*, *human-controlled*,
*computer-controlled*, *scenario random stream*, *lepton*, *tick*, *being erased*, *guard reset*
(S-0215), *protection* (S-0513), *psionics immunity* (S-0610).

- **Tag.** The one tag that a tag definition gives the game. Everything that names the tag definition
  carries that same tag, and the tag keeps its own state for each member of the chain (S-0011, Terms).
- **House value.** The integer that an action's parameter uses to name a house (S-0011, Terms and
  claim 23). An event's value is read by that event's own rule: S-0011 claim 18 for the
  entered-by event, and S-0011 claim 8 for event 44.
- **Triggering house.** Of a trigger instance: the owner of the causing object of its last entered-by event judged true (S-0011, Terms and claim 18). Neither event 6 nor event 44 sets it (claim 5).
- **Player-position value.** A house value from 4475 to 4482, which names the player at start
  position A to H (S-0011, claim 23).
- **One-time and all-carriers.** The repeat types 0 and 1 of a tag definition (S-0011, Terms and
  claim 10).
- **Deploy readiness.** Whether an object can carry out a deploy now, by its kind (S-0214, Terms).
- **House number.** A house's place in the creation order, counting from 0 (S-0009, Terms).

The glossary also defines *trigger*, used here.

## Behaviour

### Controllers

1. **Who is a controller, and its capacity.** When an infantry unit, vehicle, aircraft or building
   is created, it becomes a controller if the weapon in its type's primary slot has a takeover
   warhead. Only the primary slot is examined, as the type gives it, without any veterancy
   upgrade. A type whose primary weapon has no takeover warhead never gets a record, even when its
   secondary or elite weapon has one.
   - The control capacity is that weapon's damage value. The controller is unlimited when that
     weapon's unlimited-control flag is set.
   - Both are fixed for the object's lifetime. A promotion that switches the object to an elite
     weapon with a different damage value does not change its capacity. The control pips of S-0111
     claim 13 read the same primary weapon's damage.
   - A new record starts with no victims, an overload countdown of 30 overload updates (claim 16), a cleared
     pip highlight and a cleared overload-sound mark.
   - Backing: `measurable` (MC2 for capacity; MC11 for creation and promotion variants,
     MC8 for the initial countdown and cleared presentation marks)
   - Confidence: high

2. **At capacity.** A controller is at capacity when it is not unlimited and holds at least as
   many victims as its capacity. S-0309 claims 2 and 18 read this: a controller at capacity does
   not pick targets by itself and does not retaliate. A capacity-1 controller that holds a victim
   is at capacity for those searches, but an explicit order to take another object is not
   refused for that reason (claims 3 and 7).
   - Backing: `measurable` (MC2)
   - Confidence: high

### Taking over

3. **The takeover test.** Controller *C* can take object *T* now only when all of these hold:
   - *T* is an infantry unit, vehicle, aircraft or building (not a cell, terrain object or
     overlay),
   - *T*'s owner is not *C*'s owner (an ally may be taken; only the same house is excluded),
   - *T*'s type does not have psionics immunity (S-0610 claim 1),
   - *T* is not a vehicle that sits in a tank bunker,
   - *T* is not mind-controlled (Terms) and is not held by a trigger's house takeover,
   - *T* is not under protection (S-0513 claim 27),
   - *C* is unlimited, or holds fewer victims than its capacity, or its capacity is exactly 1,
   - *T*'s current behaviour is neither *construction* nor *selling*.

   The test is made twice: by the firing check (claim 4) and again when the shot lands (claim 5).
   So a target that becomes immune, protected or controlled while the shot is in flight is not
   taken.
   - Backing: `measurable` (MC3 for target restrictions and the landing recheck; MC2 for capacity,
     MC11 for named test-data variants, including allied targets, construction and selling).
     Trigger-house takeover is `unmeasured`: that campaign-only state is outside these scenarios
   - Confidence: high

4. **Firing check.** For a weapon whose warhead is a takeover warhead, the firing check answers
   *illegal* when the takeover test fails, at the capture-tests rank of S-0301 claim 9. When the
   test passes, the check goes on to its later ranks, including the zero-versus test. So a
   takeover warhead whose versus value against a target's armour is 0 cannot be fired at that
   target even when the takeover test would pass. A shot that does land still takes the target regardless of versus (claim 5).
   - A firer that has no control record but carries a takeover weapon in another slot cannot be
     checked by the original, which fails as soon as such a weapon is checked. Our engine answers
     *illegal* for that weapon, and the importer warns when a takeover weapon is present but
     the rookie primary does not create a control record, including an elite primary takeover
     weapon over a rookie primary without one (departure).
   - Backing: `measurable` (MC3 for firing refusal). The missing-record crash is `unmeasured`:
     the original's fault must not be provoked on the oracle. The safe refusal and warning are
     design
   - Confidence: high

5. **Detonation: takeover replaces damage and other special hits.** A warhead with the
   takeover flag never does ordinary area damage and never triggers bomb planting,
   electric charging, parasite attachment, temporal erasure, locomotor replacement, air-strike
   designation, bomb disarming or nuke-making through the same hit, even when its other flags
   request those effects. S-0514 claim 20 gives the remaining special-hit precedence when
   takeover is not enabled. Effects that do not depend on this selection still
   happen: radiation is laid before it, and the detonation's animation, debris and flash after it
   (S-0303 claim 1).

   Ownership can change only for the projectile's target, and only when that target is an
   infantry unit, vehicle, aircraft or building. A shot aimed at a cell takes nothing. When the
   firer no longer exists, or has no control record, there is no takeover or attack-tag notification.
   Otherwise:
   - when the target is a live object and has a trigger tag, that tag hears attack events 6 and 44,
     whether or not the takeover then succeeds (the trigger-event vocabulary and the judging are
     S-0011's; the order and cause are given below),
   - the takeover test (claim 3) is made again, and if it holds the takeover of claim 6 happens,
   - on success the take-over sound (Import mapping) plays at the target when the local player
     owns the controller or owned the target before the takeover.

   **The two attack events of a takeover hit.**
   - *Order.* Both raises happen before the takeover test of the landing. No event-44 trigger is
     judged until every action of the event-6 raise's satisfied triggers has completed, so an action
     may change the target (its owner, its tag, its existence) before event 44 is judged, and the
     takeover test sees the result of both raises. The order of the triggers within one raise is
     that of S-0011 claim 11.
   - *Cause.* Both raises name the shot's firer, the controlling object, as the causing object. For
     event 44, the causing object's owner at the moment of that raise is the attacking house.
     Except for a change made by an action of the event-6 raise, the target's owner in both raises
     is still its owner from before the takeover.
   - *Judging.* S-0011 claim 8 owns how events 6 and 44 are judged, including the base of event 44's
     value, which counts houses from zero. Neither event records a triggering house. S-0011
     claim 8 covers only raises on a tagged target; a raise of event 44 after a one-time or all-carriers tag has left the
     target is governed by the next item. This claim supplies only the
     raises' order and cause above.
   - *Tag removed by the first raise.* When a raise of a one-time or all-carriers trigger on event 6
     takes the tag off the target, as the repeat-type table of S-0011 (State and ordering) gives, the
     tag leaves the target. Event 44 is then raised only if the target still carries a tag at that
     point, and then on that tag, whichever tag it is. This follows S-0011 claim 10 and its repeat-type
     table, under which the raising carrier loses the tag. **Departure:** the original still raises
     event 44 at the target although the tag has gone, and that raise faults (it crashes or hangs); the
     fault needs no bad data, only a legal one-time trigger on the tag, so it is not reproduced and
     has no quirk switch, by lead decision on Q-1150: a fault of the original (it crashes or hangs)
     on legal data is fixed outright as a departure.
   A departed superweapon warp member that is the target, off the map or phasing in after it
   appears, is an ordinary target for this hit: it changes owner at the detonation, by these tests
   (S-0306 claim 17).
   - Backing: `measurable` (MC3 for target-only, no-damage hits; MC1 for takeover sound,
     MC11 for mixed-effect variants, and MC14 for the order, cause and judging of the two events.
     S-0514 scenario E13 for the remaining special-hit exclusions and independent
     radiation/presentation). Missing-firer and missing-record hits are `unmeasured`: no
     scenario here isolates a delayed shot after firer removal. The removed-tag departure
     is `unmeasured`: the original's fault must not be provoked on the oracle. MC14 part a uses
     only a repeating tag (repeat type 2), which no raise removes, so it cannot provoke the fault,
     and no scenario here exercises the skipped raise. The takeover of a departed warp member is
     `measurable` by S-0306 scenario P11 run g (off the map) and run h (phasing in).
   - Confidence: high; medium for the outcome with special house values (−1 and the player-position
     values), which MC14 part a tests; medium for a departed warp member target, as S-0306 claim 17
     rates it

6. **The takeover.** A successful takeover by controller *C* of object *T* has these results:
   - if *C*'s capacity is exactly 1, *C* first releases its current victim (claim 7),
   - *T*'s original owner is recorded as its owner at this moment,
   - *T* changes owner to *C*'s house. Every consumer of a change of owner applies (S-0414 claims 4,
     5 and 21; S-0311 claim 13; S-0506 claim 13; S-0516 claim 34),
   - *T* becomes controlled by *C* and is appended to the end of *C*'s victim list,
   - *T*'s link-line timer starts (claim 17),
   - *T* gets a guard reset (S-0215 claim 9): its target, destination and guard spot are cleared
     and *guard* is forced, subject to that claim's refusal while *wait* is current.
     A vehicle that deploys in place (a simple-deployer vehicle, S-0517
     claim 1) and whose current behaviour is *unload* keeps its behaviour and gets no guard reset,
   - *T* leaves any team it belonged to; a temporal attacker *T* lets go of its target (S-0514
     claim 14); an open-topped *T* clears the targets of its passengers,
   - the computer player's fate decision of claim 8 runs,
   - the control ring of claim 18 is attached to *T*.

   A building can be taken like any other object when the takeover test allows it. After the
   takeover it belongs to the controller's house in every respect, including power, production and
   prerequisites. A controlled construction yard cannot pack up (S-0213, S-0214).

   **Announcement for a taken building.** The owner change of a building asks for the capture or
   loss announcement, exactly as an engineer capture does (S-0509 claim 12 owns the eligibility:
   the local player is the old or the new owner and the new owner's country is not passive.
   S-0701 claim 39 owns which entry is chosen and played). A mind-control takeover requests the
   announcement, and the entry chosen does not depend on the cause, so it is the same as for an
   engineer capture of that building. S-0509 claim 12 lists the mind-control takeover and release of a
   building among the causes that request one. The announcement is requested at the owner change itself, after the trigger
   events of claim 5.
   - Backing: `measurable` (MC1 and MC3, MC9 for a building including the announcement,
     MC11 for the simple-deployer, temporal-attacker, passenger-target and construction-yard
     variants; S-0215 scenario B3 case i for the *wait* exception). Team membership is
     `unmeasured` here: original scripted teams are outside the skirmish tests
   - Confidence: high

7. **Capacity 1 replaces.** A controller of capacity exactly 1 that already holds a victim may
   still fire at a new legal target (claim 3). When the new takeover lands, the old victim is
   released with the full effects of claim 9 before the new ownership transfer is attempted.
   When the new target fails the takeover test at landing, the old victim stays controlled.
   If the test passes but the ownership transfer itself fails, the old victim has already been
   released and the new target is not controlled. A limited controller of capacity 2 or more
   that is full cannot fire a takeover weapon at all; unlimited controllers are exempt.
   - Backing: `measurable` (MC1 for replacement; MC3 for a refused landing; MC11 for unlimited
     capacity 1). Failure of the
     ownership transfer after a passing test is `unmeasured`: no measurement setup
     here reaches it
   - Confidence: high

8. **The original computer player's fate decision (original only).** After every takeover and
   every release, the original makes a decision for the object's new owner when that owner is
   computer-controlled. It picks one of four weighted fate tables, using the controlling house's
   money, its power and the object's health ratio (Import mapping). It then takes one draw from
   the scenario random stream in 1 to 100, and the cumulative weights select a fate: join the
   controller's team, go to the nearest grinder, go to the nearest bio reactor (an absorbing building; S-0605 claim 16), hunt, or nothing.
   The draw range is inclusive; cumulative weights use exact integer addition with no rounding.
   A team type's fate override replaces the selected fate, not the draw: the draw is still consumed.
   The table precedence is low money, then insufficient power, then wounded health, otherwise
   normal. Money and power belong to the controller's house even on release. Wounded means the
   victim's current health divided by its type's maximum health is strictly below the imported
   threshold. Health is first narrowed to binary32 toward zero; division by the exact integer
   maximum health uses floating point rounded toward zero to a 53-bit significand (S-0205
   claim 5). The ratio is compared directly with the imported binary32 threshold, without
   narrowing the quotient to binary32. A team assignment is attempted only when the victim's
   new house is still the controller's house and the controller has a team; otherwise that fate
   becomes *hunt*. A failed team, grinder or bio-reactor assignment also falls back to *hunt*.
   - **Marks absent from every rules layer.** For a rules set that omits a key, the original's
     result is not determined (Open questions). (A rules set
     that gives both marks never meets this case.) Our engine uses a design rule instead: an absent
     low-money mark never selects the low-money table whatever the house's money (which can be
     negative, S-0404 claim 6), and an absent wounded mark never selects the wounded table
     whatever the health ratio; the decision skips only that mark's own table and the other tables keep their
     places in the precedence above (so an absent low-money mark leaves low power, wounded, then
     normal; an absent wounded mark leaves low money, low power, then normal). Each mark is judged on its own, so one present key does not
     change the other's default. The importer warns once for each absent mark when at least one
     of the four tables is non-empty. A key that is absent from the top layer but present in a
     lower layer keeps the lower layer's value (S-0006).
   - In our engine no player house is computer-controlled (glossary, *AI house*), so the decision
     never runs for a player house. It is recorded as something our AI may reproduce through
     ordinary orders. The neutral and special houses keep the original's rule, including its
     draw.
   - The parts that do not depend on the owner (leaving the team, the temporal let-go and the
     passengers' targets of claim 6) happen for every owner.
   - Backing: `measurable` (MC12 for the original computer-player fate decision, including the
     draw when a team-type override replaces the selected fate). The policy for our AI is design.
     The absent-mark rule is a design rule and `unmeasured`: the original's result is not
     determined, and MC12's absent-key variant only records what one oracle build shows,
     informationally, so nothing confirms or falsifies the design rule.
   - Confidence: medium

### Ending control

9. **Release.** When controller *C* releases victim *V*:
   - *V*'s control ring is removed,
   - the release sound plays at *V*: the sound of *V*'s type when set, otherwise the rules-wide
     release sound when set,
   - *V* changes owner back to its recorded original owner (with every change-of-owner consumer of
     claim 6),
   - for that owner the fate decision of claim 8 runs,
   - *V* stops being controlled, and leaves *C*'s victim list.

   Release makes no explicit guard reset of S-0215 claim 9. Nevertheless the ownership change clears
   the target and destination and then normally requests *guard* with a readiness-dependent
   immediate start (S-0215 claim 7). The readiness that this request reads is the victim's after
   both clears. For an infantry victim that holds a target, the clear of its target is a change of
   target (S-0215 claim 11): an attack state that the victim was in has ended at the clear, with its
   request for *stand*, *prone* or *deployed*, and does not hold the start back. An infantry victim
   in the attack state with no target (the attack-move discard of S-0215 claim 11) has no change of
   target at the clear, so that state still holds the start back. A moving infantry victim whose
   current behaviour is *attack* is then *attack* without a target, which its movement does not hold
   back (S-0215 claim 11), so with an interruptible current sequence it starts *guard* at once.
   Every other readiness condition applies as usual: an infantry victim moving under *move*, for
   example, is not ready, and *guard* stays queued for a later per-tick start. It does not promise
   preservation of the last order. A simple-deployer vehicle currently in *unload* skips that guard
   request. The owner-independent effects of claim 6 also apply on release: leaving a team, letting
   go of a temporal target and clearing open-topped passenger targets.

   The release of a building asks for the capture or loss announcement in the same way as its
   takeover does (claim 6), with the controlling house as the old owner and the original owner as
   the new one.
   - Backing: `measurable` (MC4 for order clearing and sound, and its parts (a) and (b) for the
     readiness that the guard request reads; MC9 for a released building's announcement; MC11 for
     the release exceptions). Scripted-team departure remains `unmeasured` for the reason in claim
     6. `unmeasured` for an infantry victim in the attack state with no target: no MC part times a
     release inside an attack-move discard
   - Confidence: high; medium for the at-once start of a standing infantry victim in the attack
     state that holds a target, which rests on S-0107 claim 11's end of that state at once at a
     change of target (medium there), medium for the at-once start of a walking infantry victim
     under *attack*, which also needs its current sequence to be interruptible (S-0215 claim 11),
     and medium for the attack state with no target

10. **What releases every victim.** Each of these releases all of a controller's victims at once,
    the most recently taken first:
    - the controller receives its killing hit (S-0302 claim 14), whatever kind of object it is,
    - the controller is destroyed in any other way, or a mobile controller is removed from the
      game,
    - a controller building is not in operation (S-0405 claim 9 owns when that is: low power,
      the start of a sale, and the rest of that claim's list). The release happens in the
      building's own turn, as that claim says,
    - an erasing hit is delivered on the controller, even one refused by protection (S-0514
      claim 2),
    - a drain link forms on a controlling building (S-0608 claim 5); a failed attempt releases nothing,
    - a trigger action changes the controller's house (campaign maps; out of scope for skirmish
      but stated for completeness).

    Other releases covered here are claim 7's replacement, psychic domination (claim 13),
    boarding (S-0507 claim 6: a controlled unit that boards is released), and accepted absorption
    (S-0605 claims 7 and 8: the entrant is released before going inside).
    Claim 12's silent drop removes the link without a release.
    - Backing: `measurable` (MC4 for death and reverse acquisition order; MC6 for absorption,
      MC9 for low power and selling; MC11 for draining and non-combat removal; S-0514 claim 2
      for the erasing hit).
      Trigger changes are `unmeasured` here because campaigns are outside scope
    - Confidence: high

11. **A controller that changes owner keeps its victims.** When a controller changes owner by
    any means other than the trigger action of claim 10 (capture by an engineer, for example),
    nothing is released. The victims keep their owner (the controller's old house) and their
    original owners, and stay on the controller's list. They still count against its capacity
    and are still released by claim 10 later, back to their original owners.
    `quirks.mind_control_owner_change_keeps_victims` (default on, the original). Off: a controller
    that changes owner releases all its victims first, as claim 10 does.
    - Backing: `measurable` (MC11, using a named test-map controller made engineer-capturable)
    - Confidence: high

12. **The victim leaves the game.** When a victim is removed from the game (it is deleted, not
    merely out of play), it silently leaves its controller's list. It does not change owner, no
    sound plays and no fate decision runs.
    - A dying victim holds its slot only for as long as it still exists as the same object.
      A ship that is still sinking or a building awaiting delayed destruction can therefore
      remain on the list and in the pips. A separate death-animation object does not preserve the
      victim or its slot merely because the animation is still visible (S-0312 claim 1).
      `quirks.dying_victim_holds_slot` (default on, the original). Off: a victim leaves the list
      when it receives its killing hit.
    - A victim that leaves the map without leaving the game (into a transport, a building or a
      tunnel) keeps its place, except where its own rule releases it (S-0507 claim 6 for
      boarding; S-0605 claims 7 and 8 for accepted absorption).
    - A victim consumed by a grinder is removed from the game and silently leaves the list,
      freeing the slot. The payout values it as an object of its current owner, the controlling
      house (S-0603 claims 6, 8 and 9). This is not the bio-reactor path: accepted absorption
      releases the victim by claim 9, including ring removal, release sound, return to the
      original owner and claim 8's fate decision. It then goes inside as its original owner's
      object, not as a removed object (S-0605 claim 7).
    - Backing: `measurable` (MC6 for removal and absorption). Retention during other out-of-play
      states, including tunnel travel, is `unmeasured`: MC6 does not isolate a controlled
      traveller that enters and returns without a separate release event
    - Confidence: high for removal-based silent drop; medium for individual death-path lifetimes

### Permanent control and the original owner

13. **Permanently controlled.** Psychic domination (S-0612 claims 13 and 14) can make an object
    permanently controlled when all of these hold:
    - it is not a building,
    - its type does not have psionics immunity (S-0610 claim 1),
    - it is not under protection (S-0513 claim 27),
    - its type does not hover like a balloon (Import mapping),
    - it is not in the air.

    When it does: a controlled object is first released by its controller (claim 9, with all its
    effects), then it belongs to the dominating house, becomes permanently controlled, and
    gets the permanent control ring (claim 18). If it already belongs to the dominating house,
    there is no owner change and no change-of-owner consumer runs, but permanent control and the
    ring still apply. S-0612 claim 14 owns `quirks.dominator_marks_own_objects` (default on, the
    original). Off: an object already belonging to the dominating house is left unchanged.
    No original owner is recorded. The state is never cleared for the object's lifetime.
    - A permanently controlled object is mind-controlled for every consumer: it fails the takeover
      test (claim 3) and is refused where S-0214 and S-0507 claim 4 refuse a mind-controlled
      object, with one exception: the construction-yard deploy readiness (claim 15) reads only
      control by a controller.
    - Being permanently controlled is not among the conditions above, so a later psychic
      domination by another house takes the object again and changes its owner to that house.
    - Backing: `measurable` (MC10, including exclusion, own-house and repeat-domination variants)
    - Confidence: high

14. **The original-owner query.** For any object:
    - controlled: the recorded original owner (claim 6),
    - held by a trigger's house takeover: the house that trigger records,
    - otherwise, including permanently controlled: its current owner.

    S-0414 claim 5 reads this query when a house is defeated, and may redirect a recorded original
    owner to a civilian-side house, as it states. Permanently controlled objects have no remembered
    pre-domination owner: they survive that former owner's defeat and receive the defeat-cleanup
    hit when their current owner is defeated, as S-0414 claim 5 states.
    - Backing: `measurable` through S-0414 scenarios V2 variant j and V4 parts a and b for
      ordinary controlled objects and original-owner redirection.
      The permanent-control query is `measurable` by MC10's defeat variant. The trigger-takeover
      branch is `unmeasured`: its campaign-only state is outside these skirmish scenarios
    - Confidence: high

### Deploying and packing up

15. **Deploying ends control for good.** When a controlled vehicle deploys into a building (for
    example, a vehicle whose imported deploy-into field names a test building), the new building belongs to the vehicle's
    current owner, the controlling house. It is not controlled.
    The vehicle is removed, so it leaves its controller's list silently (claim 12). The owner change
    is permanent: killing the controller afterwards gives nothing back. The same holds for any
    other conversion of a vehicle into a building that replaces a controlled object with a new one.
    The experience passing of S-0311 claim 13 applies as usual.
    `quirks.deploy_makes_control_permanent` (default on, the original). Off: the new object
    inherits the control link, the original owner and its place on the controller's list, and is
    released as the old object would have been.

    **Which controlled vehicles can deploy.** A vehicle that is controlled by a controller (claim 6)
    is not deploy-ready when the building it would deploy into is a construction yard (S-0214
    Terms, *deploy readiness*, whose approved wording already reads *controlled by a controlling
    object*). So a taken construction vehicle cannot deploy while it is
    controlled:
    - the deploy cursor on it is the refusal one (S-0214 table VD) and the deploy command of S-0213 (the
      deploy rule) gives it no order because it fails deploy readiness, so a player's order never
      reaches it and no yard appears; no refusal sound or message is requested, because the
      refusal is the cursor only (S-0701 scenario D4 expects silence apart from the unit's own
      deploy response). Whether a refused deploy plays that response is `unmeasured`; the absence
      of a refusal sound and message is `measurable` under MC7 part b.
    - a deploy order already queued when it is taken does not survive the guard reset of claim 6
      (`unmeasured`: no scenario times a takeover between a deploy order and its start; the
      rule follows from claim 6).
    - after a release it is deploy-ready again, with no other trace of the control.
    - a vehicle held by trigger takeover and one that is *permanently controlled* (claim 13) are
      not refused: the readiness test looks only at control by a controller. A psychically
      dominated construction vehicle therefore deploys, and the yard belongs to the dominating
      house.
    A taken vehicle that deploys into any other building (a vehicle whose deploy-into field names a
    building that lacks the construction-yard marking) is deploy-ready, and the
    paragraph above applies to it: the order is accepted, the building appears as the controlling
    house's, and the controller's slot is freed.
    - Backing: `measurable` (MC7 parts a to d) for the refusal, release and permanent-control
      bullets; `unmeasured` for the queued-order bullet (claim 6) and for the trigger-held bullet
      (claim 3: campaign-only state).
    - Confidence: high (the readiness test); medium for the absence of a refusal sound

### Overload

16. **Overload pulses.** An unlimited controller's pip highlight decreases by 1 on each of its
    own updates while positive (S-0111 claim 13 reads it). Its pulse timing is:

    | Countdown at the start of an overload update | Result |
    |---|---|
    | positive | Countdown decreases by 1; no pulse check, including the update that changes 1 to 0. |
    | nonpositive | A pulse check occurs at level *i* for the current victim count *N* (F2). The countdown becomes framesᵢ overload updates; pulse damage is *d* = damageᵢ health points. |

    The first check is the 31st overload update after creation. With an unchanged level and
    positive frames value, consecutive checks are separated by frames + 1 overload updates.

    | Pulse result | Observable effects |
    |---|---|
    | *d* < 1 | The overload sound mark is cleared. There is no pulse damage, highlight reset, sound, spark creation or rocking update. The ordinary highlight countdown still applies. |
    | *d* ≥ 1 | The pip highlight is set to 10. The controller receives a hit of *d* with the demolition warhead (S-0303, Import mapping), with no attacker or attacking house and without ignoring defences. S-0302 resolves it like any other hit; a killing hit releases the victims under claim 10. The overload sound plays at the controller when its sound mark was clear, and the mark becomes set. When the spark type key is present (Import mapping), five spark particle systems of the rules' default spark type appear around the controller (F3), even after a lethal hit, and their positions consume 10 placement draws from the scenario random stream, taken at the pulse before the rocking draw. When the key is absent, neither happens (Import mapping). Creating a spark system takes no draw of its own; the draws of the sparks that each system later emits come in that system's turns (S-0616 claim 15). When *i* > 0 and the controller survives, its sideways body-rocking rate takes the F4 value, consuming one further draw. |

    Thus the sound plays once per uninterrupted run of damaging checks. The hit's effects and
    draws precede the spark placement draws, and the rocking draw follows the sparks (F3, F4).
    Controllers that are not unlimited never pulse.

    **Where the overload update falls in the controller's turn.** The overload update is part of
    the object's ordinary per-turn work and runs after the berserk countdown of S-0610 claim 5
    (and a berserk unit's own berserk step). Work the controller does later in that turn is
    observed after the pulse: a heal of that update is applied to the health the pulse leaves, a
    shot the controller fires in that update leaves after the pulse, and the pulse's hit, sparks
    and rocking draw all fall inside the overload update. S-0203 owns the order of the rest of the
    turn, and S-0516 claim 42 owns cloak timing within it.
    Consequences. A controller that self-heals and pulses in the same update loses health to the
    pulse first and heals afterwards, from the reduced health; a heal never softens a pulse. A
    controller that fires in an update fires after the pulse of that update. A pulse that kills a
    vehicle controller ends that update's remaining work: no self-healing, behaviour, movement or
    firing follows, so the shot does not leave. A shot already in flight is a separate object and
    is not affected (S-0203 claim 5). The removal counts under S-0203 claim 10 when it happens in
    the controller's own turn. For controllers that are not vehicles the same rule ends the
    update (`unmeasured`: no measurement scenario covers such a controller), but whether the object counts as no longer in play at once after a lethal hit (an infantry death sequence, a building's delayed destruction) is not
    established here. The
    random draws of the turn's earlier work precede the pulse's draws. The chance draw for a
    damaged object's smoke, which belongs to the same turn's ordinary work, comes after the pulse
    draws.

    **Updates in which the overload update runs.** In exactly those updates in which the berserk
    countdown runs (S-0610 claim 5, *Which updates are countdown updates*): every update of an
    object that is in play, except the excluded updates of S-0215 claim 2 (tunnel travel, being
    erased by a temporal weapon, the leaving stage of a chrono move). A controller inside a
    transport that is not open-topped, or otherwise out of play, takes no update and its overload
    countdown and pulse timing stay as they are. The overload countdown is therefore a count of
    such updates (F2), not of ticks, whenever updates are missed. Superweapon warp: its leaving-stage
    updates are excluded as S-0513 claim 48 gives for the berserk countdown (S-0215 claim 2 names
    only the chrono leaving stage). For the other warp updates of the kinds that S-0513 claim 51 names,
    the original's behaviour is not established (S-0513 claim 48 gives only the leaving stage), and
    our engine's rule is a design rule: the overload update is ordinary work that S-0513 claim 51
    suppresses from the first pending update through arrival effects, exactly as S-0610 claim 5
    does for the berserk countdown. No step is made up later, and the overload countdown and
    timing are kept. In arrival hold the overload update runs in each update after that update's
    hold-end test.
    - Backing: `measurable` (MC8 and MC14 for the order against self-healing, firing, a lethal
      pulse and the missed updates). The warp-time rule is a design rule of our engine, not a claim
      about the original, and is `measurable` in our engine (S-0513 SW23 part f, which S-0203 claim 4
      also cites for it).
    - Confidence: high for the order inside the update and for the set of updates; medium for
      the fate of the rest of the turn after a lethal pulse of a non-vehicle controller, and for
      F4's replacing a nonzero roll rate and its rate's not being scaled by the weight (F4)

### Presentation

17. **Link lines.** A line is drawn from each controller to each of its victims, in the
    controller house's colour. It starts at one of five imported alternate link anchors, chosen
    by the victim's current position *k* on the list (Formulas), not at the primary weapon muzzle. It ends at the victim's position raised by the
    victim type's link height (Import mapping). It is drawn when any of these holds: the
    controller is selected, a transport carrying it is selected, that victim is selected, or
    that victim's link-line timer is running. The original has a drawing-only bug: selecting
    only the carrying transport does not display lines after their timers expire. Our engine
    displays them in that case, with no quirk switch. The timer starts at the takeover and
    runs for the imported attack-line frames, in ticks. Lines are drawn only for the local
    player's view, and only presentation depends on them.
    - Backing: `measurable` (MC1, visual, including timer expiry and list compaction)
    - Confidence: high for anchor selection and original visibility conditions; medium for
     projected geometry. Transport-selection repair is design

18. **Control rings.** A controlled object carries the rules' controlled-animation type, and a
    permanently controlled object carries the permanent-controlled-animation type, attached to it
    and following it. The ring is raised above a mobile object by its type's ring height in leptons.
    On buildings it uses a building-height-based placement and draws in front of the building.
    The exact vertical conversion is not established here.
    The ring is removed at release (claim 9). When the object leaves the game the ring goes with it.
    - Backing: `measurable` (MC1 for mobile rings; MC9 for building foreground placement; MC13
      for the exact building-height conversion)
    - Confidence: high for animation choice; medium for placement and exact building-height
      conversion

## Formulas

**F1. Capacity.**
> capacity = damage of the type's primary weapon, an integer, read once at creation.
> at capacity ⇔ ¬unlimited ∧ N ≥ capacity, where N is the number of victims.
> The takeover test's capacity row passes ⇔ unlimited ∨ N < capacity ∨ capacity = 1.

**F2. Overload level.** Let the rules give three equal-length integer lists *count*, *damage* and
*frames*, indexed from 0 to *L* − 1.
> *i* = the smallest index such that countᵢ ≥ *N*; if no such index exists, *i* = *L* − 1.
> The next countdown is framesᵢ overload updates; pulse damage *d* is damageᵢ health points.
> interval between checks = max(framesᵢ, 0) + 1 overload updates when the level is unchanged.

All values and comparisons in F1 and F2 are signed integers; there is no rounding. For an object
receiving one overload update each tick these intervals are also tick intervals.

So a controller falls into the first level whose count is at least its victim count, and every
victim count beyond the last level's count uses the last level.

**F3. Spark positions.** For each of 5 systems, in order, two draws from the scenario random
stream:
> *y* = draw in [−200, 200], then *x* = draw in [−200, 200] (inclusive integers, leptons)
> position = controller's position + (*x*, *y*, 100) leptons.

**F4. Rocking.** After the sparks, one draw *r* in [0, 100] (inclusive):
> magnitude = 0.015 radians per tick if *i* = 1, 0.03 radians per tick if *i* ≥ 2,
> rate = −magnitude if *r* < 50, otherwise +magnitude. The magnitudes are the nearest binary32 representations of these decimal constants.
> Choosing the sign is exact; no arithmetic rounding is involved. The rate replaces the
> controller's roll rate, not scaled by the type's weight (S-0113 claim 17). The draw changes
> later random results.

That the rate replaces a nonzero roll rate rather than adding to it, and that the weight does not
scale it, are `unmeasured` here (medium): MC8 repeats its pulses, so a later pulse can land on a
rolling controller, but it does not compare such a pulse with one onto a controller at rest, and
it does not vary the weight.

**F5. Link anchor.** For the victim at its current position *k* on the list (0 = the first):
> alternate link anchor index = *k* mod 5.

This is exact nonnegative integer remainder, with no rounding. Removing a victim closes the gap,
so later victims can change anchors. Anchor coordinates use the firing-position transform of
S-0313 claim 1, but the imported alternate anchor replaces the weapon's muzzle triple.
F3 uses exact integer additions and integer draws, with no rounding.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| initial overload countdown | 30 | overload updates | for a new record (claim 1); counts updates, not ticks (claim 16) |
| pip highlight after a damaging pulse | 10 | ticks | read by S-0111 claim 13 |
| sparks per damaging pulse | 5 | particle systems | claim 16 |
| spark displacement range | −200 to 200 | leptons | each horizontal axis, F3 |
| spark height | 100 | leptons | F3 |
| rocking magnitude, level 1 | 0.015 | radians per tick | F4 |
| rocking magnitude, level ≥ 2 | 0.03 | radians per tick | F4 |
| rocking sign threshold | 50 | draw value | F4 |
| fate draw range | 1 to 100 | draw value | claim 8 |
| alternate anchors cycled by link lines | 5 | anchors | F5 |
| default link height | 70 | leptons | absent type key, claim 17 |
| default ring height | 140 | leptons | absent type key, claim 18 |

## State and ordering

| State | Event | Result |
|---|---|---|
| free | takeover lands, test holds | controlled by *C*; owner = *C*'s house; original owner recorded (claim 6) |
| free | takeover lands, test fails | free; tag events still raised (claim 5) |
| controlled by *C* | *C* (capacity 1) takes another object | free, back to original owner (claim 7) |
| controlled by *C* | *C* killed, destroyed, removed, not in operation, erasing hit, drain link forms | free, back to original owner (claims 9, 10) |
| controlled by *C* | *C* changes owner (not by trigger) | still controlled by *C*, owner unchanged (claim 11) |
| controlled by *C* | victim boards a transport | free, back to original owner (S-0507 claim 6) |
| controlled by *C* | accepted absorption | released by claim 9, then inside as its original owner's object (S-0605 claims 7 and 8) |
| controlled by *C* | victim removed from the game | gone; *C*'s list shrinks; no owner change (claim 12) |
| controlled by *C* | victim deploys or converts | new object of the controlling house, not controlled (claim 15) |
| controlled by *C* | psychic domination reaches it | released, then permanently controlled by the dominating house (claim 13) |
| any but permanently controlled | psychic domination reaches it, conditions hold | permanently controlled (claim 13) |
| permanently controlled | any | stays permanently controlled |

**Ordering consequences.**
- The takeover test runs again at landing, so a slow takeover shot that lands after its target
  has been taken by another controller does nothing.
- A capacity-1 controller's old victim returns to its owner in the same tick as the new victim is
  taken, before the new owner change.
- The killing hit on a controller releases its victims in the same tick, as part of that hit.
- A controller building that loses power releases its victims in its own turn of the tick in which
  it is found not in operation (S-0405 claim 9), not at the moment the power changes.
- An overload pulse happens in the controller's own turn, after the berserk countdown and before
  self-healing, behaviour processing, movement and firing, and its sparks and rocking draws come
  after the hit, in the order of F3 and F4 (claim 16).

## Edge cases and quirks

- **`quirks.mind_control_owner_change_keeps_victims`** (claim 11; default on, the original). Off:
  an owner change releases all victims first.
- **`quirks.dying_victim_holds_slot`** (claim 12; default on, the original). Off: a victim leaves
  its controller's list at its killing hit.
- **`quirks.deploy_makes_control_permanent`** (claim 15; default on, the original). Off: the
  converted object keeps the control link.
- **`quirks.dominator_marks_own_objects`** (S-0612 claim 14; claim 13 here). Default on:
  eligible objects already belonging to the dominating house become permanently controlled with
  a ring without an owner change. Off: those objects are left unchanged.
- **Takeover weapon without a control record** (claim 4). The original fails when such a weapon
  of an object without a record is checked. Departure: the weapon is never legal, and the importer
  warns. A secondary takeover weapon is not malformed when the rookie primary already creates
  the required record.
- **Capacity from the rookie weapon** (claim 1). A type whose elite primary weapon has a different
  damage keeps the rookie capacity after promotion. This is consistent with the pips of S-0111
  claim 13 and is kept as is, with no switch.
- **Allies can be taken** (claim 3). Only the controller's own house is excluded.
- **Versus decides the firing check, not the hit** (claims 4 and 5). A zero versus value only
  stops the controller from firing. A landing takeover shot does not look at versus, which
  matters only for a shot that skips the firing check (the unchecked shots of S-0301 claim 9).
- **A takeover hit on a target that fails the test** at landing does nothing at all, not even
  damage; the tag events of claim 5 are still raised.
- **Overload hit with no attacker** (claim 16): S-0302 credits no house with the kill when it
  destroys the controller.
- **Construction vehicle taken by control** (claim 15): it is not deploy-ready until released,
  while a permanently controlled or trigger-held one is. No switch: the readiness rule is
  S-0214's, with the narrowing named in Open questions.
- **Second attack event after a tag-removing first raise** (claim 5): the original faults; the
  raise is skipped when no tag is attached (departure, no switch).
- **Absent fate-table marks** (claim 8): an absent mark never selects its table, with an importer
  warning (departure from an original whose result is not determined).

## Import mapping

| Key | Section | Parse | Absent | Feeds |
|---|---|---|---|---|
| `MindControl` | warhead | boolean | no | takeover warhead (claims 1, 4, 5) |
| `Primary` | infantry, vehicle, aircraft or building type | weapon name | none | the weapon that decides controller, capacity and unlimited (claim 1) |
| `Damage` | weapon | integer | (S-0302) | control capacity of a controller whose primary weapon this is (claim 1) |
| `InfiniteMindControl` | weapon | boolean | no | unlimited controller (claim 1) |
| `ImmuneToPsionics` | type | boolean | S-0610 claim 1 | takeover test and permanent control (claims 3, 13) |
| `BalloonHover` | type | boolean | no | excluded from permanent control (claim 13) |
| `MindClearedSound` | type | sound name | none | release sound for this type (claim 9) |
| `MindClearedSound` | `[AudioVisual]` | sound name | none | rules-wide release sound (claim 9) |
| `YuriMindControlSound` | `[AudioVisual]` | sound name | none | take-over sound (claim 5) |
| `MasterMindOverloadDeathSound` | `[AudioVisual]` | sound name | none | overload sound (claim 16) |
| `LeptonMindControlOffset` | type | integer, leptons | 70 | link-line end height (claim 17) |
| `MindControlRingOffset` | type | integer, leptons | 140 | mobile ring height (claim 18) |
| `AlternateFLH0` through `AlternateFLH4` | type's art section | integer triples, forward/lateral/height in leptons | zero triple | five alternate link anchors (claim 17, F5) |
| `ControlledAnimationType` | `[CombatDamage]` | animation name | none | control ring (claim 18) |
| `PermaControlledAnimationType` | `[CombatDamage]` | animation name | none | permanent control ring (claim 18) |
| `MindControlAttackLineFrames` | `[CombatDamage]` | integer, ticks | 15 | link-line timer (claim 17) |
| `OverloadCount` | `[CombatDamage]` | integer list | empty | *count* of F2 |
| `OverloadDamage` | `[CombatDamage]` | integer list | empty | *damage* of F2 |
| `OverloadFrames` | `[CombatDamage]` | integer list | empty | *frames* of F2 |
| `C4Warhead` | `[CombatDamage]` | warhead name | (S-0303) | warhead of the overload hit (claim 16) |
| `DefaultSparkSystem` | `[CombatDamage]` | particle-system name | none (the original's result for an absent key is not established; as a safe design fallback, the overload pulse creates no spark systems and takes no spark placement draws, and the importer warns) | overload sparks (claim 16); the same key is also read by S-0616 (game-wide spark type) and S-0614 claim 9, and all three read the same value |
| `AICaptureNormal`, `AICaptureWounded`, `AICaptureLowPower`, `AICaptureLowMoney` | `[General]` | integer list of four weights | empty | fate tables of claim 8, in the order team, grinder, bio reactor, hunt |
| `AICaptureLowMoneyMark` | `[General]` | integer, credits | none: never selects the table (engine rule; the original's result is not determined, claim 8) | below this the controlling house's money selects the low-money table (claim 8) |
| `AICaptureWoundedMark` | `[General]` | decimal fraction | none: never selects the table (engine rule; the original's result is not determined, claim 8) | below this health ratio the wounded table is used (claim 8) |

Bad data: a type whose primary weapon has a takeover warhead is a controller even when that
weapon's damage is 0 or below. The capacity row of claim 3 then fails, so it can take nothing
unless it is unlimited; the importer warns about such a weapon. The three overload lists are
expected to have equal lengths; *L* is the length of the count list, and the importer warns when
the other two are shorter or when the count list is empty (Open questions). The original's result
for those malformed lists is unestablished. As a safe design fallback, our engine uses 0 for a
missing damage or frames entry and gives no overload pulses when the count list is empty.

## Measurement scenarios

All scenarios: skirmish on a small purpose-made map, stock data apart from the named test files of each scenario (the named test-map variants of MC11 among them), with no engine add-on and no mods otherwise (*measurement setup*, glossary), game-speed index 3, bench at 800×600. Ticks have the
rate specified by S-0201 claim 4; all intervals below use tick counts, not elapsed seconds.
A is the viewing human. B and C are named bystander computer players: their unrelated objects
cannot reach the observation area, and a run is void if they interfere or are defeated. For
order-retention tests use B's preplaced neutral-house counterpart instead, so the fate decision
is separately identified; prefer observations while A owns the victim. Record colour, selection,
health, orders and tick numbers with video and a read-only recorder. Use scripted input where
exact timing is needed. Make three valid runs of each deterministic variant. Any result differing
from the stated expected owner, count, timing, sound or order effect falsifies that part of its
claim, not just an alternative explicitly mentioned below. No scenario writes a random seed.

- **MC1. Capacity 1, replacement and presentation** (claims 1, 6, 7, 17, 18). A named test copy of the
  Yuri Clone with its primary weapon's damage set to 1 by test data, and two of B's tanks. Order the clone onto tank 1, then onto tank 2. Record per tick the owner of
  both tanks. Expected: tank 1 becomes A's on landing and is guarded (stops, no target); when the
  second shot lands, tank 1 turns back to B in the same tick as tank 2 becomes A's; the release
  sound plays at tank 1. The takeover sound plays for A. A link line and a ring are visible on
  the current victim. Deselect both objects and wait beyond the imported line duration: the line
  disappears but the ring remains. Selecting the victim or controller restores the line. Repeat
  with a Mastermind holding at least six victims: compare anchors against its alternate art
  anchors, then remove a middle victim and check that later lines move to their newly indexed
  anchors. Also board an empty clone into an open-topped transport, acquire a victim through
  passenger fire, wait for line expiry, and select only the transport. The original's line is
  absent in this last case; our engine's fixed view shows it (claim 17).
  Falsified by wrong colours, ring following or anchor cycling as well as by two concurrent
  victims of the clone, excluding that documented drawing fix. MC11 tests promotion and named
  anchor-height changes.
- **MC2. Capacity** (claims 1, 2, 3). A's Psychic Tower and more of B's infantry than its capacity
  walk past it. Record how many become A's. Expected: exactly the capacity; the tower stops
  picking targets and retaliating while full. Attack it with an otherwise legal extra target to
  test retaliation separately. Kill one victim and record its actual removal, not merely the
  disappearance of a drawn death effect: another can be taken only after a slot becomes free.
  Falsified by an extra takeover or a full tower retaliating with its takeover weapon.
- **MC3. Illegal targets** (claims 3, 4, 5). Variant (a): a named test copy of the Clone whose takeover
  weapon's versus against building armour is set to 0 by test data, ordered onto a B power plant
  that test data marks not psionics-immune, is refused; a named test copy of the Prime with a
  nonzero versus against the same building takes it. Variant (b): A's Yuri Clone ordered onto: one of A's own units, a
  psionics-immune unit (for example one that the test data marks immune), a unit already controlled by B's Yuri
  Clone, and a unit under B's force shield or iron curtain. Expected: each attack order is refused
  or never fires; no damage. Include a vehicle in a tank bunker, an ordinary eligible vehicle
  outside it, and a cell-targeted force-fire order. Expected: no takeover inside the bunker or
  at a cell, but the outside vehicle can be taken. Check adjacent units' health as well: takeover
  does not deal ordinary area damage. MC11 supplies the slow-shot case: protect the target or
  have another controller take it between launch and landing. Expected: no takeover, no damage,
  and the firer's old capacity-1 victim remains held. A successful late takeover falsifies the
  landing recheck.
- **MC4. Death releases** (claims 9, 10). A's Mastermind holds three of B's units; B kills the
  Mastermind. Expected: in the tick of the killing hit all three return to B, the release sound
  plays three times in reverse acquisition order, and the returned victims have no attack target or
  movement destination. Give each victim a distinct long movement order while A controls it and
  record the release tick before the new computer owner's fate order can mask the result. Repeat
  with a Yuri Clone holding a human-owned map ally where available. If a new computer order masks
  order clearing, that run tests only ownership and sound, not orders. MC11 isolates order clearing
  by arranging that the released victims return to A. Continued execution of the old order in that
  isolated variant falsifies claim 9. Parts (a) and (b) (claim 9, the readiness of the guard
  request). A's Yuri Clone takes V, a pre-placed neutral-house infantry unit (the set-up's
  counterpart rule), and A orders V to attack a neutral-house object: in part (a) one far enough
  away that V walks towards it, in part (b) one within V's range, so that V stands and fires. A's
  tank then destroys the Clone with a projectile shot (S-0203 claim 5), at a recorded tick. The
  projectile entered play after V, so it takes its turn after V's in the release tick (S-0203 claims
  3 and 4). Record V's current and queued behaviour at the end of every tick, and in part (b) the
  tick in which each firing kind of V begins and the tick of each of its shots. Part (a) counts only
  runs in which V is walking in the release tick, and part (b) only runs in which a firing kind of V
  began at or before the release tick and V fired no shot of it in or before the release tick, so
  that V is in the attack state at the release. A run in which V's current or queued behaviour at
  the end of the release tick is *hunt*, or a trip to a grinder, a bio reactor or a team (claim 8),
  is void. Expected: in both parts *guard* is V's current behaviour, with nothing queued, at the end
  of the release tick. Falsified by *guard* still queued at the end of the release tick.
- **MC6. Removal versus absorption** (claims 9, 10 and 12). On a coastal test map A's
  Mastermind holds B's naval vehicle; sink it. Record victim existence, pip count, ownership and
  sounds each tick. Expected: the slot stays occupied while the original hull still exists,
  then disappears without a release sound or owner change at removal. Repeat with infantry
  deaths that retain the actor and deaths that replace it with a separate animation: only the
  former can retain a slot. Repeat by grinding an eligible controlled vehicle: consumption
  frees the slot without a control-release sound or owner change, and the payout uses the
  controlling house's factors (S-0603 claims 8 and 9). Continued slot occupancy after removal,
  or a control-release sound at silent removal, falsifies that part.
  Separately order B's eligible controlled infantry into A's bio reactor. Expected: in the entry
  tick the ring disappears, the control-release sound precedes the entry sound, the controller's
  slot is freed, and the infantry returns to B before going inside. Sell the building and record
  the infantry emerging as B's object (S-0605 claim 7). No release sound,
  continued control, or emergence as A's object falsifies the absorption expectation. Do not
  infer actor lifetime from a visible animation alone.
- **MC7. Deploy keeps the new building** (claim 15). Part a: A's Yuri Clone takes a named test vehicle
  whose deploy-into field test data sets to a named test building; A deploys it, then B kills the
  clone. Expected: the test building stays A's, the clone's slot
  is free. Falsified if it goes to B. Part b: A's Yuri Clone takes B's construction vehicle on
  open ground. Select it, hover the deploy cursor over it and press the deploy hotkey. Expected: the
  refusal cursor, no yard and no order accepted, no refusal sound or message; the vehicle keeps its
  guard state. Falsified by a yard appearing, by a deploy turn starting, or by any refusal sound.
  Part c: kill the clone, so the vehicle returns to B; B deploys it at once. Expected: deploys.
  Falsified if it is refused or shows any trace of the control.
  Part d: A's psychic dominator takes B's construction vehicle (S-0612); A orders it to deploy.
  Expected: it deploys and the yard is A's. Falsified if it is refused; that would mean permanent
  control also blocks the deploy and would change claim 15 and claim 13's exception. Run each
  part three times.
- **MC8. Overload** (claim 16). A's Mastermind; feed it victims one at a time, holding each count
  for at least 300 ticks. Record the Mastermind's health per tick and the ticks between drops.
  Expected: no damage for a level whose imported damage is nonpositive. For a damaging level,
  feed its damage and demolition warhead through ordinary damage resolution to predict the health
  drop, rather than assuming raw overload damage equals health lost. With one update per tick,
  checks are frames + 1 ticks apart; the first check is the 31st overload update after creation.
  Record the sound once at the start of a damaging run, ten-update pip highlights and rocking
  from the second level on. Include a transition through a non-damaging level and back to test
  the sound-mark reset. Use the random-state recorder, not a fixed seed, to attribute ten
  placement draws per damaging pulse and one rocking draw when the level is above zero and
  the controller survives. Account separately for draws made by the hit and by the spark systems' later turns (S-0616 claim 15).
  Check F3 positions and F4 signs and binary32 rocking magnitudes against the recorded values.
  Include a pulse that kills the controller, run on a named test copy whose test overload table gives a lethal level: expected release of all victims in that tick,
  five spark systems and ten placement draws, but no rocking update or rocking draw and no
  house credited with the overload kill. Compare a named test copy of a controller that is not unlimited
  (test data clears its unlimited-control flag) over the same observation interval: expected no overload check, pulse effects or attributed draws.
  MC14 covers the position of the pulse in the turn.
  A frames-tick interval, a
  premature first check, an extra sound, wrong placement or a different attributed draw order
  falsifies the corresponding part.
- **MC9. Controller building not in operation** (claims 6, 10). A named test copy of the Psychic Tower (test data: capacity 2) holds two of B's
  units; drop A's power below demand. Expected: both return to B within one tick of the power
  change. Repeat by selling the tower: the units return when the sale starts. Then with a named test
  copy of Yuri Prime whose versus against the building's armour is set nonzero by test data, take
  one of B's buildings that test data marks not psionics-immune (a power plant): it becomes A's and A's
  power rises by its output. Record the capture announcement requests (read-only) and sounds in the
  takeover tick: expected the same announcement entry an engineer capture of that building gives
  the local player (once with A local, once with B local, and once with the local player a
  third house: no announcement). Repeat for a building whose type has the needs-engineer flag and an
  imported capture announcement, naming the type in the setup (S-0701 claim 39 gives the custom entry only to such buildings). Then let B kill the Yuri Prime that holds the power plant: the plant returns to B
  and the release announcement goes to whichever of A and B is local, none to a third local
  player. A missing or different announcement, or one for a third house, falsifies claim 6 or 9. Observe the building's ring in front of its sprite. For a captured
  construction yard test packing up: the controlled yard cannot do so. A power change from
  capture, or an attempted pack-up, must not silently remove the control link.
- **MC10. Permanent control** (claims 13, 14). Three houses A, B and C. C's Yuri Clone controls
  one of B's tanks; A's psychic dominator then hits that tank and other tanks of B. Expected: all
  become A's, the controlled tank's release sound plays first, and killing C's clone afterwards
  changes nothing. A Yuri Clone of B ordered onto any of them is refused. Then C's psychic
  dominator reaches the permanently controlled tanks: they become C's and remain permanently
  controlled. Include a protected unit, a psionics-immune unit, a balloon-hovering unit, an
  airborne unit and a building beside an eligible ground tank. Expected: only the eligible
  ground targets are transferred. Also include an eligible ground tank already belonging to A
  at A's sweep: its owner stays A, but it gains permanent control and a ring, with no owner-change
  effects (S-0612 claim 14; scenario PD7 part b). A transfer of an excluded target, successful
  ordinary takeover of a permanent victim, later return to B, refusal of the second domination,
  or failure to mark the own-house tank falsifies claim 13 under the default quirk setting.
  In separate short-game defeat variants, C permanently takes a tank from B. Destroy B's last
  building while A and C remain opposing contenders: expected the tank survives as C's object.
  In a fresh run instead destroy C's last building while A and B remain opposing contenders:
  expected the permanent tank receives C's defeat-cleanup hit and is destroyed. Use an ordinary
  unprotected tank outside combat; record owner and existence through cleanup. Removal on B's
  defeat or survival through C's cleanup falsifies claim 14's use of the current owner rather
  than the pre-domination owner.
- **MC11. Named test-map variants** (claims 1, 3, 5 to 7, 9 to 11, 17 and 18). Each variant changes
  only the properties named below on a copy of a stock type, and is removed after its run.
  These are data-only overrides, never executable changes. Record per-tick ownership, orders,
  sounds, pips and presentation as above.
  - Give the rookie primary and elite primary distinct positive capacities, then promote the
    controller through kills. Expected: capacity stays at the rookie value. Repeat with unlimited
    control enabled and capacity exactly 1: a new victim still replaces the old one. Put a takeover
    weapon only in the secondary slot over an ordinary primary: the importer warns and our
    engine refuses it; do not run that crash-producing arrangement on the oracle.
  - Use a controller whose takeover projectile has a deliberately long flight time. For the
    protected-target landing test put the controller in the neutral house, fire at A's target,
    and have A protect its own target before impact. Also test the already-controlled landing
    variant of MC3 and a mutually allied target of a different house. Expected: late-invalid targets are untouched; the allied different-house
    target can be taken when explicitly fired upon. Record ordinary order legality separately
    from the takeover test, so a cursor refusal does not by itself test an unchecked hit.
  - Set the takeover flag together with a demolition-charge or electric-charge flag on the test
    warhead. Expected: takeover only, no charge planting, electric-charge change or area damage.
    Attach separate repeating tags (repeat type 2) with attack events 6 and 44 and observable local-variable effects:
    both events occur on an alive target even if protection is applied before landing.
    Launch control shots at a building in construction and at one beginning a sale, with
    eligible idle buildings as controls. Expected: construction and selling refuse takeover.
  - For release-order tests, use a neutral-house controller and a victim initially belonging to
    A. Let it take the victim, then remove the controller through a skirmish destroy-object
    trigger. Expected: the victim returns to A with target and destination cleared; this avoids
    any computer-owner fate order after release. Repeat while the victim is a simple-deployer
    vehicle in *unload*: takeover and release preserve that behaviour. Repeat with a temporal
    attacker and an open-topped transport firing through passengers: takeover and release let
    go of the temporal target and clear passenger targets. A retained temporal attack or
    passenger target falsifies the cleanup, independently of the victim's behaviour slots.
  - Make a Psychic Tower engineer-capturable. While it holds A's victims, have A's engineer
    capture it. Expected: victims retain the old controller house as current owner and keep
    their slots and original owner; destroying the tower afterwards returns them to A. On a
    fresh tower with victims, form a flying-disc drain link: all victims return immediately,
    most recently taken first. Repeat with a failed drain attempt from beside the tower, using
    S-0608 scenario D4 part b: no victim is released.
  - Give a freshly created unlimited controller a nonempty overload table whose first level has
    positive damage and a short invented positive frames value. Keep victim count in that level.
    Expected: first health loss at overload update 31, subsequent losses frames + 1 updates apart.
    Repeat with zero and negative frames: one check each update after the initial delay. Use
    this variant to isolate MC8's initial countdown even when the imported level-zero damage is zero.
  - Give the five alternate anchors distinct invented triples and vary the link-end and mobile
    ring heights independently. Expected: each visual moves only with its own imported field,
    and line anchors cycle and compact as F5 states. Compare pixels using S-0313 claim 1.
  Each wrong capacity, unexpected takeover or release, retained target, owner, slot, sound or
  presentation result falsifies the relevant part, except for the explicitly fixed transport
  selection drawing bug in claim 17. No stock value is prescribed by these
  overrides.

- **MC12. Computer-player fate decision** (claim 8). On a purpose-made skirmish map, use a
  named test controller and eligible victims, with the controller owned by a bystander computer
  player; keep the observed area clear under the glossary's void rule. Use named test data to
  make each fate-table selection condition and each fate outcome observable, including a team
  type with a fate override. Record the controller's money and power, the victim's health and
  maximum health, team assignment, destination/behaviour, and scenario-random-stream state before
  and after each takeover and release. Repeat deterministic variants three times; use separate
  runs for each table precedence and outcome. Expected: the selected table follows the stated
  precedence, one inclusive 1–100 draw is consumed even when a team override replaces the fate,
  and assignment failures fall back to hunt. A different table, draw count, selected fate or
  fallback falsifies the corresponding part of claim 8. The neutral/special-house rule can be
  checked by repeating with that house as the new owner. Absent-key variant, informational only:
  on a test copy of the rules remove each mark in turn and, with fate tables that make each
  table's choice visible (only the hunt weight set in one table), record which table the decision
  uses with the controlling house at zero money and with a badly damaged victim. The original's
  result is not determined, so any outcome is recorded without falsifying claim 8, and it never
  changes claim 8's design rule; repeat on a fresh game start to see whether it varies.
- **MC13. Building control-ring height** (claim 18). On a named test map, have a named test copy of Yuri Prime
  (versus nonzero against the building's armour by test data) control a building with a visible controlled animation. Record the building sprite and ring
  pixels in a fixed-resolution capture, and compare the ring's vertical position against the
  building's height and the corresponding mobile ring-height setting. Repeat with named test
  variants that change only the building height and the ring-height field, three valid runs per
  variant. Expected: the ring stays in front of the building and its vertical placement follows
  the original's building-height conversion. A pixel position inconsistent with the same
  measured conversion across repeats, or a change caused by altering an unrelated field,
  falsifies the placement rule. Record the measured conversion in the measurement record before
  stating a numeric formula.

- **MC14. Attack events and the overload turn** (claims 5 and 16). Part a (claim 5): on a named
  test map attach one repeating tag (repeat type 2, which no raise removes) to a B tank. The tag
  carries two triggers: one on event 6 and one on
  event 44 whose value is A's house number, each setting a different local variable and playing a different
  sound; add a third trigger on event 44 with C's house number. Fire a takeover shot from A's
  Yuri Clone. Expected: the event-6 effects, then the event-44 effects, in that order and
  both before the tank changes owner; the C-valued trigger never fires. Repeat with a tank that is
  put under protection after the shot is launched, so that the landing test refuses it: both events
  still fire in the same order and no takeover happens. Repeat with the
  event-44 value −1, then with a player-position value, then with the registration index of the
  country A plays when that differs from A's house number: never fires in any of the three. Repeat
  with the value one above and one below A's house number: never fires. A firing for any of these, or
  no firing for A's house number, falsifies the judging rule. A number counted from one, or from any other base, would show as the one-above or one-below value firing. Repeat twice with an event-6
  trigger action that changes the tank's owner, to A in the first run and to C in the second.
  Expected: with the owner changed to A the takeover is refused, since the owner is the controller's
  house; with it changed to C the takeover lands, the tank becomes A's, and its original owner is C.
  In both runs the event-44 trigger with A's house number still fires and the C-valued one does not,
  since the value is judged against the causing object's owner, not the target's. A trigger action
  that names the triggering house (house value 9013) has no effect in either event's trigger. Any
  other result falsifies the order or the cause rule. Never use a one-time event-6 trigger on the oracle: the
  original faults. Part b
  (claim 16): a named test copy of a vehicle type (test data: an unlimited takeover weapon, a short invented positive frames value, two overload levels, one non-lethal and one lethal at the low health, and a self-healing rule at low health), holding enough victims for each level. Record
  per tick its health, shots and the recorder's draws. Expected: the health dip of the pulse
  tick and the heal of the same tick appear in that order (the net change equals the heal
  applied to the reduced health); the shot of a pulse tick leaves after the pulse; for a lethal
  pulse no shot leaves in that tick and no heal is applied. Then put the test vehicle into the
  states of S-0215 claim 2 (a chrono jump's leaving stage, temporal erasure), into a transport that is not open-topped (expected: no step while it is inside), and into an open-topped transport where possible (expected: steps); record whether the pulse countdown steps in those updates
  (expected: no, in the excluded ones and while inside a transport that is not open-topped; yes in the open-topped transport). Falsified by a heal
  applied before the pulse, a shot that leaves before the pulse, a heal or a shot after a lethal
  pulse, or a countdown step in an excluded update. Part b has no case for the superweapon warp. The warp part of claim 16 is measured by S-0513 scenario SW23 part f, an engine-only contract test, not an oracle expectation: it arranges a due overload update of an unlimited controller in each row of S-0513 claim 51, for each applicable kind, and expects no overload step in those rows through arrival effects, with the hold-end test first. Any overload step there falsifies the warp rule.

## Open questions

- The exact height of a control ring above a building (claim 18) is not settled; it is
  presentation only.
- Whether a controller can change owner in a skirmish without named test data (claim 11).
  MC11 uses an explicitly capturable test type rather than assuming such a case exists.
- What the original does when the damage or frames list of the overload is shorter than the count
  list, or when the count list is empty (F2).
- What the original's fate decision does for rules that omit the two marks: its result is not determined
  (claim 8). Our engine's never-select rule is a design choice, not a reproduction (Q-1202).

**Changes named for other specs.**
- S-0214 Terms, *deploy readiness*: already as stated in the approved S-0214 (its construction-yard
  exclusion reads *controlled by a controlling object*; permanently controlled and trigger-held
  vehicles are not refused; claim 15; Q-1200). No further change. The other uses of
  *mind-controlled* there are unchanged.
- S-0701 claim 39: no change needed. Approved S-0701 claim 39 already states that S-0509 claim 12 lists
  the mind-control takeover and release of a building, without a condition. Its medium confidence for
  mind-control entry selection rests on MC9 being proposed and not run; this revision does not change that.
- Glossary, *departure* and *quirk switch*: add a third case to *departure*, a fault of the original (it crashes
  or hangs) on legal data, which our engine fixes outright with no switch, as the lead decided on
  Q-1150. The *quirk switch* entry's "no switch" rule, which now covers only bad data and host or
  real-time dependence, takes the same case. This claim (claim 5) is the first case.
- S-0011: no change needed; it names no switch for this case.

## Questions for the project lead

None open.
