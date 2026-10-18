---
id: S-0607
title: Magnetic lift (seizing a vehicle's locomotor, lifting, dropping and the fall)
topic: combat
revision: 3
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-18
measurements: []
requests: [Q-1320, Q-1322, Q-1323, Q-1324, Q-1325, Q-1326, Q-1327, Q-1328, Q-1329, Q-1330, Q-1331]
supersedes: []
---

# S-0607: Magnetic lift (seizing a vehicle's locomotor, lifting, dropping and the fall)

## Summary

A shot whose warhead seizes locomotors replaces an eligible vehicle's own movement with jumpjet
movement and links it to the firer, which pulls it in. This spec owns:

- which hits seize and which targets are immune,
- the lift and what a seized unit can and cannot do,
- every way the link ends, and the difference between a unit dropped from the air and one set
  down,
- the fall, the impact on what lies underneath, and the falling damage and its credit,
- the outcomes on water, beach, clear and blocked cells,
- when the unit gets its own locomotor back,
- the magnetic wave's lifetime and how it interacts with a second, building-attacking weapon.

Skirmish only. Network games and campaigns are out of scope.

Rules owned elsewhere and not restated here:

- **Firing cycle.** The shared sonic and magnetic wave slot is S-0301 claim 5. The early *rearming*
  answer against the firer's own gripped unit is S-0301 claim 9, rank 4. The weapon chosen
  against a building is S-0308 claim 3, table P row 7.
- **Protection.** Temporary invulnerability refuses a seize: S-0513 claim 27.
- **Movement of a seized unit as a jumpjet.** S-0502 owns it:
  - claim 10: body facing,
  - claim 12: the arrival row,
  - claim 16: turning a destination into a movement point (claim 7 here),
  - claims 13 to 15: hovering, water and beach, the landing check,
  - claim 18: the crash fall, no spin, zero speeds and the growing crash rate,
  - claim 19: sinking.
- **Other rules that test the seized state:**
  - the approach step: S-0215 claim 37 c and d,
  - deployed vehicles: S-0517 claims 9 and 11,
  - cloaking: S-0516 claim 4,
  - spawners: S-0506 claims 6 and 13,
  - parasites: S-0614 claim 12,
  - crash start of a released slot aircraft: S-0501 claim 36,
  - turning: S-0206 claim 14,
  - restoration of a teleporting locomotor: S-0207 claim 21,
  - the falling-back table: S-0215 claim 13,
  - a building lifter that stops being in operation: S-0405 claim 9.

## Terms

From the glossary: *jumpjet state* (grounded, ascending, hovering, cruising, descending, crashing,
crashed), *hover slot*, *height above the surface*, *render layer*, *contact*, *team member*,
*temporarily invulnerable*, *locomotor*.
- **Projectile damage:** S-0302 (Terms).
- **Demolition warhead, crushing warhead:** S-0303 (Import mapping).
- **Game-wide death weapon:** S-0312 claim 6.
- **Magnetic-beam weapon, magnetic wave:** S-0301 (Terms).
- **Passive house:** S-0009.

New terms:

- **Seizing warhead.** A warhead with the imported locomotor flag. It names a locomotor that it
  installs on its target as a temporary replacement for the target's own. This spec covers the
  case where that locomotor is the jumpjet locomotor, which is the case that stock data uses.
  Claim 22 says what a hit does when the warhead names another locomotor; the movement of a unit
  seized with another locomotor is outside this spec.
- **Movement point:** S-0502 (Terms): the point the jumpjet locomotor flies to, distinct from the
  unit's order-level destination.
- **Heading point** of a mobile object: the point its own movement is heading for at that moment
  (its next movement point, or its position when it has none), as in S-0212 claim 25, table T5.
- **Seized.** A unit is *seized* from a successful seize (claim 5) until it gets its own locomotor
  back (claim 20). The state survives the end of the grip and a drop.
  - Other specs say "held by magnetic lift" (S-0502), "held by a magnetic beam" (S-0501, S-0506,
    S-0516), "restrained by a magnetic beam" (S-0506), "magnetic grip" (S-0516) and "carried by
    another object's locomotion" (S-0517). All of these mean *seized*.
  - It is distinct from the glossary's *held* (the parasite countdown).
- **Grip, lifter, gripped unit.** The link between a firer and the unit it has seized. The firer is
  the *lifter* and the seized unit its *gripped unit*. An object grips at most one unit, and a unit
  has at most one lifter.
  - A gripped unit is always seized. A seized unit is not always gripped.
  - S-0301's *held target* and S-0405's *held unit* are the lifter's gripped unit.
- **Drop, dropped unit, dropper.** A grip that ends while the gripped unit's height above the
  surface is 1 lepton or more *drops* it (claim 13).
  - The dropped unit records the lifter as its *dropper*, and the lifter's house as the
    *dropper house*.
  - A unit stays *dropped* until it touches down on a clear cell (claim 17) or its fall ends at
    the impact (claim 18).
- **Set down.** A grip that ends while the gripped unit's height above the surface is 0 or less
  *sets it down*. A seized unit is also set down when it touches down at the end of a *descending* flight.
  Being set down is what lets a seized unit get its own locomotor back (claim 20).

## Behaviour

### Seizing

1. **A seizing hit does no damage.** When a projectile with a seizing warhead detonates, the
   seize of claims 2 to 5 takes the place of ordinary damage. Neither the target nor anything
   near it takes damage from that hit.
   - A takeover warhead suppresses the seize (S-0601 claim 5).
   - Effects that S-0303 claim 1 lists as independent of this selection still happen.
   - Backing: `measurable` (scenario L2: the target's health before and after the hit)
   - Confidence: high

2. **When a hit seizes nothing.**
   - The hit does nothing at all when any of these holds:
     - the firer has left play,
     - the firer already grips this very target,
     - the firer is itself seized.
   - Otherwise, before the target is tested, the firer's current grip ends as claim 13 says. A
     dropped unit falls. The firer keeps its target. This happens even when the target then
     turns out to be immune.
   - Backing: `measurable` (scenario L9 parts c and e)
   - Confidence: high

3. **Which targets can be seized.** The target is seized only when all of these hold:
   - it is a vehicle or an aircraft. Infantry and buildings are never seized,
   - it is in the ground render layer (S-0102). An aircraft in flight and a jumpjet in the air
     layer are immune; a landed aircraft is not,
   - it is not temporarily invulnerable (S-0513 claim 27),
   - it is not already seized,
   - it is not protected by its factory. A vehicle is protected when it is in contact with a
     building of a war-factory type and that building covers the vehicle's cell,
   - its type's size is strictly less than the projectile damage (F1).
   - Backing: `measurable` (scenario L1)
   - Confidence: high (each condition); medium (the factory-contact condition, whose window is
     short in play)

4. **Resource gatherers already on a temporary locomotor.** A target that is a resource-gathering
   vehicle has a special case when its own locomotor is already temporarily replaced and it is not
   gripped. An example is a teleporting harvester while it drives.
   - If that replacement may end at this moment (claim 20's condition), it ends: the vehicle gets
     its own locomotor back and is not seized.
   - Otherwise the vehicle stops and its current behaviour is cleared. It queues *stop* and is
     not seized.
   - Both cases still lose spawned objects and release a naval parasite as claim 5 describes.
   - Backing: `measurable` (scenario L1 part f)
   - Confidence: medium

5. **The seize.** A seized unit:
   - stops: its route is cleared, it gives up the occupancy mark of its cell, and it stands
     still,
   - starts *guard* with no target and no destination,
   - loses its spawned objects as S-0506 claim 13 says, and its naval parasite as S-0614 claim
     12 says.

   Its jumpjet locomotor is then linked (for a warhead naming another locomotor, claim 22), with
   the jumpjet fields of the unit's own type. If the
   link succeeds:
   - the firer's previous grip ends (claim 13) without clearing the firer's target,
   - a grip the unit itself held ends (claim 13), clearing the unit's own target,
   - the grip is made, and the unit is given the lifter as its destination. Its jumpjet movement
     turns this at once into a fixed movement point, and its order-level destination becomes the
     cell of that point, not the lifter (claim 7),
   - the unit is deselected,
   - it breaks off a contact with a building,
   - it queues *guard*.

   Ordering is observable: the firer's old gripped unit is released, and starts its fall, in the
   same tick in which the new unit is seized.
   - In the original it also stops being a team member. The original's team system is outside
     this spec's implementation scope.
   - Backing: `measurable` (scenario L2, including contact and cleanup variants). Team removal is
     `unmeasured` here: it concerns the original's out-of-scope team system.
   - Confidence: high (stop, guard, grip, destination, deselection); medium (contact)

6. **Seized units of computer-controlled houses.** When the seized unit belongs to a
   computer-controlled house that is not a passive house, it queues *hunt* instead. A
   resource-gathering vehicle is the exception. This describes the original only. Our AI houses
   use the human-house rule; our AI may reproduce the later hunt through ordinary orders after
   restoration. The neutral and special houses retain the original's non-human rule. The queued
   hunt cannot make a seized unit move under its own power or fire (claims 9 and 10).
   - Backing: `measurable` (scenario L11 part b)
   - Confidence: medium

### The lift

7. **The pull.** A gripped unit flies with jumpjet movement (S-0502), with the jumpjet fields and
   absent-key defaults of its own type (S-0502 Import mapping).
   - **Its movement point is fixed at the seize.** The unit rises and cruises towards the point
     produced by applying S-0502 claim 16 to the lifter's point at the moment of seizure:
     - the lifter's point is its heading point when the lifter is mobile (a lifter standing
       still gives its own position), and the centre of its foundation when it is a building.
       A building of a type that services docking units gives its docking point instead.
     - S-0502 claim 16 then takes the centre of a cell found by the nearby-cell search around
       that point's cell (S-0212 claim 25, table T5, the jumpjet movement-point row for a
       vehicle that is not balloon-hover). That search rejects a cell whose occupancy record is
       not empty, so the lifter's own cell, which the lifter marks, normally does not qualify:
       the movement point is normally the centre of a free cell next to the lifter's cell,
       chosen by that claim's visibility preference and tick rule. Bridge cells never qualify.
     - when the search finds no cell, the movement point is the lifter's point itself, with S-0502
       claim 16's no-result consequences.
   - The movement point is not refreshed while the grip lasts. When the lifter moves, the gripped
     unit keeps flying to the point fixed at the seize. A mobile lifter that accepts a new
     destination ends the grip anyway (claim 12).
   - It keeps the body facing it had when seized (S-0502 claim 10).
   - The hovering, water/beach and landing exceptions of S-0502 claims 13 to 15 apply to it
     because it is seized.
   - Backing: `measurable` (scenario L2 for the path, facing and the movement point beside the
     lifter; scenario L12 for a lifter that moves during the pull)
   - Confidence: high (the fixed point, the search, no refresh); medium (the building lifter's
     point and the docking-point case)

8. **Arrival drops the unit beside its lifter.** When a gripped unit arrives (S-0502 claim 12: its
   horizontal distance from its movement point is strictly less than 20 leptons), these happen in
   the same update:
   - it is placed exactly on its movement point,
   - its target height becomes 0,
   - it enters *descending*,
   - its lifter's grip ends as claim 13 says.

   Normally it is still in the air at that moment, so it is dropped over the cell of its movement
   point, which is normally a free cell next to the lifter's: claims 15 to 17 then normally find
   that cell clear, and the unit descends and is set down unhurt. The unit falls onto the lifter
   only when the lifter is in that cell at the impact, for example because it has driven there.
   - Backing: `measurable` (scenarios L2 and L12)
   - Confidence: high

### What a seized unit can do

9. **Orders and destinations.** A seized unit ignores every new destination, from orders, approach
   steps or any other rule.
   - A rule that gives a gripped unit *no* destination is the exception. That ends its grip
     without a drop, and the unit is set down where it is, at whatever height. It then descends
     and lands under claim 17's clear-cell rule.
   - Its own locomotor does not return until claim 20 allows it.
   - Backing: `unmeasured`: the destination refusals and the no-destination exception are traced,
     but deselection prevents a fresh player order, and no ordinary order reaching this state
     has been established. Scenario L9 part a is a conditional protocol, not established backing.
   - Confidence: high (ignored destinations); medium (the no-destination exception)

10. **No firing.** A seized unit gets *illegal* for every weapon and target, at S-0301 claim 9's
    rank 4 (a reason about the firer's state). This lasts from the seize until it gets its own
    locomotor back, including while it falls.
    - **Deploying and unloading are not refused.** Being seized adds no refusal of its own to
      deploying, undeploying or unloading passengers beyond what S-0517 claims 9 and 11 already
      say, while gripped or while set down before restoration. Those actions go through their ordinary checks (S-0517 claims 9 and 11, and
      the transport specs), which can still refuse them for other reasons, such as height.
    - Backing: `measurable` (scenario L9 part b; part g for deploying and unloading)
    - Confidence: high (the refusal); medium (the rank); low (deploying and unloading)

11. **The lifter's own weapons while it grips.**
    - **Against its gripped unit.** A firing check by the lifter against its own gripped unit
      answers *rearming*, with any weapon. This answer comes ahead of every test of the target's
      layer, height, kind or armour and of every projectile test, so a gripped unit that is in the
      air never gets *illegal* or *impossible* from its lifter. Of S-0301 claim 9's rank-4 answers
      it is preceded only by the materialising firer's *rearming* and by the refusal of a firer
      that is bound to a slave master. The lifter therefore keeps the gripped unit as its target
      through the whole pull: nothing in this spec or in that check drops it, and the grip lasts
      until an event of claim 12.
    - A lifter with a magnetic-beam weapon gets *rearming* when all of these hold:
      - it grips a unit,
      - its magnetic wave is alive (claim 21),
      - the target is not its gripped unit,
      - the target is not a building.
    - Against a building, a weapon is not refused for this reason. With stock arrangements, S-0308
      table P row 7 chooses the secondary weapon against a building.
    - When the lifter turns to a building its target changes. That ends the wave (claim 21) but
      not the grip: the gripped unit keeps being pulled towards the lifter, and the lifter is
      still refused against other non-building targets until the wave's fade-out ends.
    - A magnetic-beam weapon is not otherwise held back by its own live wave. Apart from S-0301
      claim 5's shared slot, a live wave refuses only the cases above.
    - This *rearming* belongs to S-0301 claim 9's rank 4, before the ordinary countdown and
      own-effect gates of rank 5. It comes after the target's layer and projectile tests, so
      against an airborne target the beam cannot hit, that refusal wins.
    - Backing: `measurable` (scenario L9 parts c, d and f)
    - Confidence: high (the conditions, the gripped-unit answer and its precedence); medium (the
      building-attack sequence)

### Ending the grip

12. **What ends a grip.** Each event in this table ends the grip at once.

    | Event | Lifter's target cleared? |
    |---|---|
    | The gripped unit arrives at its lifter (claim 8) | yes |
    | The lifter is a mobile unit and accepts a non-empty destination (from an order, an approach step or another rule) | yes |
    | The lifter is a vehicle that falls back to rest, unless an override is in effect (S-0215, Terms) or its current behaviour is *wait* | yes |
    | The lifter is a building that stops being in operation (S-0405 claim 9) | yes |
    | The lifter is a building that is destroyed | yes |
    | The lifter's seizing projectile detonates again at any target (claim 2) | no |
    | The lifter seizes another unit (claim 5) | no |
    | The lifter leaves play | not applicable |
    | The gripped unit leaves play | yes |
    | The gripped unit is given no destination (claim 9) | no |

    Ending a grip never clears the *seized* state. Only claim 20 does.
    - Backing: `measurable` (L2 arrival, L3 accepted destinations and rest, L7 leaving play,
      L8 building rows, L9 new hits). The no-destination row retains claim 9's `unmeasured` backing.
    - Confidence: high (arrival, orders, a new seizing hit, leaving play); medium (falling back
      to rest, building rows)

13. **Drop or set down.** When a grip ends:
    - **Height above the surface 1 lepton or more:** the unit is dropped.
      - It is marked as crashing, with the lifter as its dropper and the lifter's house as its
        dropper house.
      - Its orders stop (the **stop on dropping**): its destination becomes none, its route is
        cleared, its target is cleared, it breaks off any contact and it is deselected. Its jumpjet
        movement is stopped by S-0502 claim 17's rule; claim 15's detour to its cell centre then
        decides where it falls.
      - This is not a crash start in the sense of S-0312 claim 16: its passengers stay in, and no
        tumbling draws are taken.
    - **Height 0 or less:** the unit is set down and its destination becomes none.
    - Backing: `measurable` (scenario L3 runs a and b)
    - Confidence: high

14. **A dropper that leaves play.** When a dropper leaves play before its dropped unit lands, the
    unit no longer has a dropper, but keeps its dropper house. Credit under claims 16 to 18 then
    goes to that house.
    - A mobile lifter that leaves play while it grips drops the unit (claim 12) just before.
    - Backing: `measurable` (scenario L7)
    - Confidence: medium

### The fall and the impact

15. **The fall.** A dropped unit falls in two stages.
    - **To its cell centre.** If it is not exactly over the centre of its current cell when its
      crash would begin, its destination becomes that centre first and it keeps flying under
      S-0502's cruising rules. Its crash begins only once it is over the centre. The arrival snap
      of S-0502 claim 12 puts it there.
    - **The fall itself.** From there it falls by S-0502 claim 18 (no spin, speeds 0, a growing
      crash rate).
      - **Starting rate.** The crash rate starts at the unit's own type's jumpjet crash value
        (S-0502 Import mapping), narrowed to single precision. It starts afresh at every seize, so
        a unit seized a second time does not keep the rate it reached before.
      - **Each update.** S-0502 claim 18's extra crash drop uses the current rate (the resulting
        height truncated toward zero), and only after that drop does the rate grow by 1.0, the
        new rate narrowed to single precision. The first update's extra drop is therefore the
        starting rate itself.
      - **Target height.** While a seized unit is crashing, its jumpjet target height is 0 in
        every update.
    - **When the fall ends.** It ends when its height above the surface falls below 1 lepton, or
      when its fall crosses a bridge deck from at or above to below. Claims 16 and 17 then apply,
      in that order, in the same update.
    - Backing: `measurable` (scenario L4)
    - Confidence: medium (the detour, which is the carried S-0502 item); high (the hand-over to
      S-0502 claim 18, the starting rate, the order of drop and growth)

16. **What lies underneath.** When a dropped unit's fall ends, the first object listed in its
    cell's ground layer, in S-0209 claim 13's listing order, other than the unit itself decides
    the result. The ground layer is read even when the fall ended on a bridge deck (quirk below):

    | First other object | Result |
    |---|---|
    | none | The cell is not blocked by an occupant; claim 17 decides. |
    | a vehicle or a building | No damage from this rule. The cell is blocked and claim 17's blocked row on land applies, even on water: a unit dropped onto a vessel crashes rather than sinks. |
    | an infantry unit | That infantry unit and every object after it in the cell's list are removed at once, without death effects or death animations. The crush sound of each one's type plays at the dropped unit's position. Each removal is credited as a kill to the dropper, else to the dropper house, else to the dropped unit. Claim 17 then decides. |
    | anything else (for example a landed aircraft or a tree) | It takes damage equal to its current health with the crushing warhead, ignoring defences. The attacker is the dropper; with no dropper, the attacker is the dropped unit and the house is the dropper house. Claim 17 then decides. |

    - Backing: `measurable` (scenario L5)
    - Confidence: high (infantry, other objects, the credit), medium (vehicles and buildings,
      the listing order in mixed cells, the bridge-deck case). Removal of later non-infantry
      objects has high confidence as a rule but `unmeasured` backing for reachability: no legal
      setup establishing such a mixed list at impact is known. L5 gives a conditional test.

17. **Water, beach, clear and blocked.** After claim 16, unless a vehicle or building made the
    cell blocked:
    - **Water and beach.** The cell counts as water when its land type is water and the unit did
      not land on a bridge deck. It counts as beach likewise.
    - **Blocked.** The cell is blocked when the entry outcome that S-0209 gives the dropped unit
      for that cell (claim 5's table for a vehicle) is anything other than *clear*: *cloaked
      blocker*, *moving blocker*, *closed friendly gate*, *friendly obstacle*, *enemy obstacle*,
      *stationary friendly* and *blocked* all count. A crushable occupant whose row gives *clear*
      (rows VR2 and VR3 of S-0209 claim 5) does not block. Since claim 16 has already dealt with
      the first other object, this test decides in practice only for a cell whose list was empty,
      held a tree or a landed aircraft, or was emptied by claim 16's erasure, and for a deck.
      A dropped aircraft or jumpjet type also faces an extra clearance test:
      use the bridge-deck occupancy record when the cell has a high bridge, otherwise the
      ground record (S-0209 claim 1). A building-present mark or a wall overlay fails this
      test; infantry and vehicle/object marks do not. Away from a high bridge, a zero wheeled
      terrain-speed multiplier also fails it (S-0211 claim 2). A failed test counts as blocked.

    | Cell | Result |
    |---|---|
    | blocked, water and not beach | The unit is set sinking (S-0504 claims 4 and 5) and stunned. The stun repeats claim 13's stop on dropping, so it adds nothing observable beyond S-0504's sinking. Its slaves are freed by the death of their controller, credited to the dropper. Its own coming death is credited to the dropper, else to the dropper house. It does not take claim 18's falling damage. |
    | blocked, on land or beach | The unit ends its fall in *crashed*, releases its hover slot, and claim 18 applies. |
    | not blocked | The unit releases its hover slot and enters *descending* with no destination. At touchdown it is set down and is no longer dropped or crashing. It survives unhurt (claim 20 then restores it). |

    - Backing: `measurable` (scenario L3 for clear land, L6 for water, L5 and L4 for blocked)
    - Confidence: high (clear land and water, the blocked outcomes); medium (beach, decks,
      aircraft)

18. **Falling damage.** A dropped unit that ends its fall in *crashed* receives one impact
    notice.
    - **Vehicle types that can crash.** If its type is crashable, the ordinary crash end of S-0312 claim 16
      applies instead and none of this claim does.
    - **Otherwise,** while it is seized, its impact has these effects:
      - The impact produces two detonations using the projectile of the game-wide death weapon
        and damage *D*ꜰ (F2). The source is the dropper, or the falling unit with no dropper.
        The first is at ground level (the unit's world height less its height above the surface)
        with the crushing warhead. The second is at the unit's own position with the demolition
        warhead. Ordinary area damage applies (S-0303 claims 6 to 12), so the falling unit,
        its lifter if underneath, and neighbours can all be hurt.
      - **No death weapon (design).** The original reads the game-wide death weapon without
        checking that one is configured, and stops with a fatal error when none is. Our engine
        warns at import, and each of the two detonations becomes a plain area detonation of its
        warhead at its point, with the same amount *D*ꜰ, source and house, and no projectile
        effects.
      - Impact destruction is credited to the dropper, else to the retained dropper house.
      - The falling unit leaves play after these effects even if both detonations left it
        alive (quirk below).
    - **Ordering.** A neighbour destroyed by the first detonation is absent for the second. Both
      use the same pre-impact damage amount even if the first changes the falling unit's health.
    - **Credit when a detonation kills the falling unit** (`quirks.lift_impact_double_credit`,
      default on). The impact credit is given whether or not the unit is already dead. A falling
      unit killed by either detonation is therefore credited twice: once to the detonation's
      source under the ordinary rules, and once more by the impact credit. Both credits count in
      the kill and loss tallies, the score and the experience of the credited side. A survivor
      removed under `quirks.lift_impact_removes_survivor` is credited once. Fixed behaviour:
      the impact credit is skipped for a unit that a detonation has already killed.
    - **Passengers.** A dropped transport that a detonation kills loses its passengers under the
      ordinary rules for a dying object (S-0312 claim 5). A transport that survives both
      detonations and is then removed loses its passengers without attacker credit (S-0507
      claim 16).
    - Backing: `measurable` (scenario L4 and its test-data variants); the missing-weapon
      departure is `unmeasured` because it is a project design choice, not an observation of the
      original.
    - Confidence: high (the two detonations, shared amount, removal, the double credit and the
      passenger rule); medium (the credit of a survivor's removal).

19. **Dropped aircraft.** A seized aircraft that ends its fall in *crashed* takes the same falling
    damage, but its first detonation uses the demolition warhead. Claim 18's crashable-type
    vehicle exception does not apply to this aircraft impact.
    - Backing: `measurable` (scenario L4 part d, named test data)
    - Confidence: medium

### Getting its own locomotor back

20. **Restoration.** A seized unit's temporary jumpjet locomotor ends, and its own locomotor
    returns, when the unit falls back to rest (S-0215 claim 13) and all of these hold:
    - its jumpjet movement is not moving (S-0502's *moving*, not *moving now*),
    - it is set down, or it is no longer seized.

    Restoration clears *seized*, *set down* and the dropper. A seized resource gatherer of a
    computer-controlled house then queues *harvest* in the original. Our AI houses use the
    human-house rule and may issue an ordinary harvest order after restoration; the neutral
    and special houses retain the original's non-human rule.

    So a gripped unit hovering over its lifter is never restored. A unit set down by claims 9,
    13 or 17 is restored on its first fall-back to rest after touchdown.
    - **Only at rest.** Restoration is tested only at the fall-back to rest. A set-down unit
      whose current behaviour stays *guard* (claim 5) and which never falls back to rest stays
      seized indefinitely, with its temporary jumpjet movement.
    - **The first order after touchdown.** While the unit is still seized, a move order's
      destination is refused (claim 9). The refused order leaves the unit in *move* with no
      destination, so it falls back to rest, is restored, and a second move order then moves
      it normally.
    - **Readiness while seized.** Wherever S-0215 claim 11 asks whether a vehicle is moving on
      this tick, it asks the unit's current locomotor. While seized that is the temporary
      jumpjet movement, so the question is S-0502's *moving now* (every state except *grounded*
      and *hovering*). This holds also for a unit that was driving on a temporary locomotor of
      its own when it was seized: the driving locomotor's moving test no longer applies.
    - Backing: `measurable` (scenarios L3 and L11) for restoration and orders.
    - Confidence: high (restoration conditions, only at rest, readiness); medium (the
      two-order sequence)

### The magnetic wave

21. **The wave's lifetime.**
    - **Creation.** A magnetic-beam weapon's shot creates a magnetic wave only when its firer has
      no live wave (sonic or magnetic, S-0301 claim 5). The wave is not created when the
      horizontal distance between the shot's firing and impact points is under 240 leptons
      (F3). Either way the shot
      itself is fired, so a seize can happen with no wave.
    - **Ending.** The wave has no fixed duration. It ends when its firer or its target leaves
      play, or when its firer's target is no longer the wave's target. The wave's distance limit
      applies to sonic waves only. Leaving play includes going inside a transport, a bunker or a
      building: a target or firer that does so ends the wave at the wave's next update, even
      though it is not destroyed.
    - **When it updates.** The wave joins the update order when it is created, appended after
      every object already in play (S-0203 claims 2 and 3). It has one update at creation; the
      same tick's pass may reach it again under S-0203's rules. Each later tick it updates in its
      own turn, after its firer's turn. Its end test reads its firer's target as it stands at
      that turn, so a retarget made in the firer's turn of the same tick is seen at once.
    - **Its points.** F3 is evaluated once, at creation, from the shot's firing and impact points
      as computed when the shot is fired. Later movement of firer or target does not re-test it.
    - **Fade-out.** After it ends it fades out (F4) and stays alive until the fade-out finishes,
      about 20 ticks after a full fade-in. While alive it counts for S-0301 claim 5 and claim 11.
    - **No damage.** A magnetic wave deals no damage of its own.
    - **Not tied to the grip.** The grip does not depend on the wave. The wave can end while the
      grip lasts (claim 11), and a grip can begin with no wave.
    - Backing: `measurable` (scenario L10) for creation, lifetime, update turn, damage and fade
      timing. The exact accumulator precision and table-root choice in F3 and F4 are `unmeasured`:
      no setup here distinguishes alternative arithmetic with the same visible thresholds.
    - Confidence: high (no damage, end conditions, numeric rules, update turn); medium
      (observable fade timing)

### Other locomotors

22. **A seizing warhead that names another locomotor.** Claims 1 to 3 decide the seize the same
    way whatever locomotor the warhead names. What follows depends on that locomotor:
    - **One that cannot stand in temporarily** (hovering, flying, rocket or tunnelling
      movement): the hit still has claim 5's other effects on an eligible target (it stops, is
      put in *guard*, loses its spawned objects and its parasite), and still
      releases the firer's previous victim (claim 2). But the target is not seized: no
      locomotor is installed, it keeps moving with its own, and it can be hit again.
    - **One that can stand in temporarily** (driving, walking, ship, teleporting or drop-pod
      movement): the target is seized as in claim 5, with that locomotor installed in place of
      its own. How it then moves, and when it is restored, are outside this spec.
    - **An unknown identifier (design).** The original stops with a fatal error. Our importer
      warns, and a hit is treated as with a locomotor that cannot stand in temporarily.
    - An absent key means the teleporting locomotor (Import mapping).
    - Backing: `measurable` (scenario L13) for the first two bullets; the unknown-identifier
      departure is `unmeasured` because it is a project design choice, not an observation of the
      original.
    - Confidence: medium (which locomotors can stand in, the effects without a seize)

## Formulas

**F1 Size test.** The target can be seized only when

> *s* < *D*

where *s* is the target type's size (a decimal held in double precision) and *D* is the projectile
damage when it detonates (an integer, widened exactly). The comparison is exact and strict, so a
size equal to the damage is immune.

**F2 Falling damage.**

> *D*ꜰ = max(0, trunc(RZ₅₃(*B* × *m*)))

where:
- *m* is the falling-damage multiplier, held in single precision,
- *B* is the unit's current health when the current-strength switch is on, else its type's
  strength (both integers),
- the product is one arithmetic result under S-0205 claims 4 and 5,
- trunc is the common-form conversion (S-0205 claim 7).

A negative result becomes 0. With the engine defaults (*m* = 1.0, switch on), *D*ꜰ is the unit's
current health.

**F3 Wave distance.** With the horizontal integer lepton coordinates of the firing and impact
points of the shot (S-0313 claim 1 and S-0306 claim 1):

> *n* = (*x*ₜ − *x*ꜰ)² + (*y*ₜ − *y*ꜰ)²
> *d* = *r*(*n*)

The differences, squares and sum use signed 32-bit integer arithmetic, wrapping as S-0205
claim 1 states. For non-negative *n*, *r* is S-0204 claim 5's table square root, without integer
truncation of its result. For a sum that wraps negative, *r*(*n*) = *r*(−*n*): the root's input
is made positive first. No wave is created when *d* < 240 leptons. This distance includes no height or building-size term.

**F4 Wave fade.** The two dimensionless values *a* and *b* start at 0 and are held in double
precision. Let *q* = 13421773 / 268435456, the exact single-precision engine constant close to
0.05, widened exactly to double precision. Each addition and multiplication below uses RZ₅₃
(S-0205 claims 4 and 5); no single-precision narrowing occurs in these accumulators.

While the wave travels, each update with *a* < 1 gives *a* the value RZ₅₃(*a* + *q*). A result
above the double-precision engine threshold 8827055269646172 / 9007199254740992 (approximately
0.98) becomes exactly 1. Once the wave has ended, each update with
*b* ≤ RZ₅₃(*a* + RZ₅₃(*q* × 0.5)) gives *b* the value RZ₅₃(*b* + *q*). It is removed in that
update if the new *b* ≥ *a*. This also applies if the wave ends during fade-in: *a* stops growing
and *b* catches up to that retained value. There is one update at creation as well as the
subsequent per-tick updates.

A visual integer countdown starts at 100 ticks, decreases by 1 each tick and is reset to 64
when an update finds it at trunc(1 / *q*) = 19 ticks. It never ends a magnetic wave. Its drawing
belongs to the rendering specs.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| arrival distance | 20 | leptons | S-0502 claim 12 (owned there) |
| drop height threshold | 1 | lepton above the surface | claim 13: non-positive height sets down, 1 or more drops |
| minimum wave distance | 240 | leptons, horizontal | F3 |
| wave fade step | 13421773 / 268435456 | per update, dimensionless | F4; single-precision constant, double-precision accumulator |
| wave full-strength threshold | 8827055269646172 / 9007199254740992 | dimensionless | F4; approximately 0.98 |
| wave fade-out margin | half the wave fade step | dimensionless | F4 |
| wave visual counter start, reset trigger, reset value | 100, 19, 64 | ticks | F4; no game effect |
| falling-damage multiplier absent-key default | 1.0 | — | F2 |
| current-strength switch absent-key default | on | — | F2 |
| magnetic-lift crash growth | 1.0 | leptons per tick, per tick | S-0502 claim 18 (owned there) |

## State and ordering

Life of a seized unit:

| State | Event | Next state | Effect |
|---|---|---|---|
| ordinary | eligible seizing hit (claims 1 to 5) | gripped (seized) | stops, guard, deselected, movement point fixed (claim 7) |
| gripped | arrives at its movement point (claim 8) | dropped | drop over that cell, normally beside the lifter |
| gripped | grip ends at height ≥ 1 (claim 12) | dropped | crashing mark, dropper recorded |
| gripped | grip ends at height ≤ 0, or no destination (claims 9, 12) | set down | destination none |
| dropped | not over its cell centre | dropped | flies to the cell centre (claim 15) |
| dropped | fall ends, cell clear (claim 17) | descending, then set down at touchdown | survives |
| dropped | fall ends, blocked on water (claim 17) | sinking | death credited to the dropper |
| dropped | fall ends, blocked on land or beach | crashed, then removed | claim 18, unless crashable |
| set down | falls back to rest, jumpjet not moving (claim 20) | ordinary | own locomotor returns |

**Ordering consequences.**
- One hit can drop the old gripped unit and seize the new one in the same tick (claim 5).
- Within one seizing hit, the firer's previous gripped unit is released even when the new target
  is immune. A resource gatherer made immune by claim 3 keeps its temporary locomotor. A
  resource gatherer that reaches claim 4's special case loses its spawned objects and releases
  its naval parasite in either outcome of that case.
- Claim 16's effects on what lies underneath happen before claim 17's sinking or landing.
- A neighbour destroyed by claim 18's first detonation is absent for the second; both
  detonations nevertheless use the damage computed from the falling unit before either.

## Edge cases and quirks

- **Erasing the rest of the cell** (`quirks.lift_drop_erases_cell_tail`, default on).
  - When the first other object under a dropped unit is an infantry unit, every object after it
    in the cell's list is also removed (claim 16), whatever its kind. That includes vehicles and,
    in principle, the dropped unit itself when it is listed after the infantry.
  - Fixed: only infantry units in the cell are removed, and the dropped unit never is.
- **Removing a survivor** (`quirks.lift_impact_removes_survivor`, default on).
  - Claim 18 removes the unit even when the falling damage did not kill it. That can happen
    with armour against the crushing and demolition warheads, a multiplier below 1, or curtain
    protection gained after the seize.
  - Fixed (design): a unit that survives both detonations stays in play. It is placed on the
    surface below its impact point, or on the deck if its fall ended crossing a deck, grounded
    and set down, and is then restored by claim 20.
- **Double impact credit** (`quirks.lift_impact_double_credit`, default on). A falling unit
  killed by either impact detonation is credited a second time by the impact credit (claim 18).
  Fixed: it is credited once.
- **A deck landing reads the ground layer** (`quirks.lift_deck_impact_reads_ground`, default on).
  - When a dropped unit's fall ends on a bridge deck, claim 16 judges the objects listed in the
    ground layer under the bridge, not those on the deck. Infantry under the bridge are erased
    and a vehicle under it blocks.
  - Fixed: claim 16 reads the layer the unit landed in.
- **A harvester driving temporarily is immune** (claim 4). This is not a quirk: it is the only way
  a resource gatherer on a temporary locomotor is treated.
- **An infantry kill credited to the dropped unit** (claim 16) when there is neither a dropper nor a
  dropper house. This happens only if the dropper house no longer exists; the original behaviour
  is kept.

## Import mapping

| Source | Key | Feeds | Form | Absent |
|---|---|---|---|---|
| rules, weapon | `IsMagBeam` | magnetic-beam weapon (claims 11, 21; S-0301 owns the term) | boolean | no |
| rules, warhead | `IsLocomotor` | seizing warhead (Terms, claim 1) | boolean | no |
| rules, warhead | `Locomotor` | the locomotor that the seize installs (Terms, claim 22) | locomotor identifier | teleporting locomotor (claim 22); an unknown identifier warns (claim 22) |
| rules, infantry or vehicle or aircraft type | `Size` | *s* (F1) | decimal | 1.0 (S-0507) |
| rules, object type | `Strength` | *B* when the current-strength switch is off (F2) | integer | as S-0302 |
| rules, type | the jumpjet type keys of S-0502 (Import mapping) | the lifted unit's jumpjet movement (claim 7) and its crash value, the starting crash rate (claim 15) | as S-0502 | as S-0502 |
| rules, `[CombatDamage]` | `FallingDamageMultiplier` | *m* (F2) | decimal, single precision | 1.0 |
| rules, `[CombatDamage]` | `CurrentStrengthDamage` | current-strength switch, F2 | boolean | yes |
| rules, `[CombatDamage]` | `CrushWarhead` | crushing warhead (claims 16, 18) | warhead name | as S-0303 |
| rules, `[CombatDamage]` | `C4Warhead` | demolition warhead (claims 18, 19) | warhead name | as S-0303 |
| rules, `[CombatDamage]` | `DeathWeapon` | game-wide death weapon, whose projectile claim 18 uses | weapon name | as S-0312; none configured: claim 18's design rule |
| rules, building type | `WeaponsFactory` | war-factory type (claim 3) | boolean | no |
| rules, vehicle type | `ResourceGatherer` | resource-gathering vehicle (claims 4, 6, 20) | boolean | no |
| rules, country | `MultiplayPassive` | passive house (claim 6) | boolean | as S-0009 |
| rules, type | `Crashable` | crashable vehicle type (claim 18) | boolean | no |
| rules, type | `JumpJet` | jumpjet-type extra landing-clearance test (claim 17) | boolean | no |
| rules, type | `CrushSound` | crush sound (claim 16) | sound name | none: no sound |

## Measurement scenarios

Use a purpose-made small skirmish map with clear land, water, beach, a bridge and a tree cell,
with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), speed index 2 and 800×600 pixels. One human house owns the magnetic
lifter and, unless a test says otherwise, its victims: force-fire supplies friendly hits without
a second human player. One bystander computer player stays outside the observed test area.
Apply the glossary's bystander void rule. Named test map overrides below are removed afterwards.
They change data only, never the executable. Run each deterministic variant three times and
record the initial random state, rather than writing a fixed seed into the game.

Use screen capture, scripted input and a read-only per-tick recorder for position, height,
health, facing, current and queued behaviour, destination, owner, kill counts, selection and
passenger counts. Record random-stream positions for L3's draw comparison. A project test bench
may supply recording and input only if it can run the hash-checked stock executable without a
patched launcher; otherwise use a standalone recorder and scripted input.
Request any missing recorder capability rather than patching the executable or harness.

- **L1 Eligibility (claims 3, 4).**
  - Fire at each of: an infantry unit, a light and a heavy vehicle, a landed aircraft, an
    aircraft in flight, a vehicle under the iron curtain, a vehicle just leaving a war factory,
    and an already-seized vehicle.
  - Named test data: give one vehicle type a size just below and one equal to the weapon's damage.
  - Part f: a teleporting harvester while it drives back to its refinery.
  - Named test map L1 also supplies a seizing weapon able to hit each target class; otherwise a
    refusal to fire cannot test immunity. Keep its damage above the tested size except for the
    equal-size variant. Test a grounded and an airborne jumpjet vehicle separately.
  - Expected: each otherwise eligible ground-layer vehicle or aircraft lifts; infantry,
    buildings, air-layer units, invulnerable units, already-seized units, equal-size units and
    factory-protected vehicles do not. Merely being heavy is not immunity.
  - Record the harvester's movement, queued order and restoration: a replacement able to end
    is restored without a lift; otherwise it stops and queues stop without a lift. For the
    factory case, the recorder must confirm contact and foundation coverage at impact; without
    that state this part is void, not a confirmation.
  - Falsified by: any immune target lifting, an eligible one not lifting, or a harvester outcome
    contrary to its confirmed temporary-movement state. Part f's two states are conditional if
    stock driving never supplies them; do not manufacture either by writing game state.
- **L2 Pull and arrival drop (claims 1, 5, 7, 8, 16, 17, 18).**
  - Seize a vehicle at long range and let the lifter stand still.
  - Record:
    - the target's health at the hit, which should not change,
    - its body facing during the pull, which should stay constant,
    - the movement point fixed at the seize (its cell) and the tick of arrival there,
    - the drop over that cell: the fall, and either a harmless descent and touchdown (clear
      cell) or both detonations and removal (blocked cell),
    - the health of the lifter and of the dropped unit,
    - whose kill count rises.
  - Also record the stopped route, forced guard, cleared target, initial destination and
    deselection. Put a separate undamaged unit nearby to detect area damage at the seizing hit.
    For cleanup, use a stock spawned-aircraft carrier and, separately, a parasitised vessel.
    Expect spawned objects lost and the naval parasite released at seizure. If a contacted
    factory-protected target is immune, it cannot test post-seize contact breaking; record
    contact breaking only when a different reachable building contact is confirmed at seizure.
  - Expected: no damage from the seizing hit, unchanged body facing, a path to a free cell
    next to the lifter's cell, a drop at arrival within the strict 20-lepton threshold of that
    cell's centre, and, with that cell clear, a harmless descent and touchdown beside the lifter.
    A named L2 map may vary the victim's jumpjet speed or climb field one at a time, its trajectory must follow its own fields, not the lifter's.
  - Falsified by: any missing seizure effect above, damage at the hit, a turning body, trajectory
    following the lifter's fields, a grip surviving arrival, an arrival over the lifter's own
    cell while a free neighbour cell existed, damage after a clear-cell drop, or survival after
    a blocked impact.
- **L3 Release by order over clear land (claims 9, 12, 13, 17, 20).** This is the first scenario
  to run, because it decides whether a release is harmless.
  - Run a: mid-pull, at full cruise height over clear land, order the lifter to move one cell.
    - Expected: the unit is dropped; it flies to its cell centre, falls with a growing rate,
      finds the cell clear, descends, touches down undamaged, is no longer crashing, and drives
      normally after its next fall-back to rest.
    - Record the ticks from the order to touchdown and the health at touchdown.
    - Falsified by: damage or removal on clear land.
  - Run b: give the order in the tick of the seize, at height 0.
    - Expected: set down with no fall.
  - Run b is void unless the recorder confirms height 0 at release; a fall, damage, or failure
    to restore then falsifies it. Run a also falsifies claim 15 if a non-centred release falls
    vertically without the cell-centre detour.
  - Run c: use a transport with passengers. After a clear-land release, verify all passengers
    remain. Compare its random-stream trace with an empty transport under the same recorded
    conditions: no crash-start tumbling draws should occur. Passenger loss at drop or those
    extra draws falsifies claim 13.
  - Run d: order stop on the lifter while gripping, with no override or wait behaviour. Record
    its return to rest and the victim's release. A retained grip after that return falsifies
    claim 12's rest row. Repeat with wait current or an override; that rest event alone must
    not release the victim.
- **L4 Blocked landing (claims 15, 16, 18, 19; crash rate, credit, passengers).**
  - Release over a cell holding another vehicle. Record the cell-centre detour, the crash rate per
    tick, both detonations, the health of the vehicle underneath, and the dropped unit's removal.
  - Crash rate: record the height per tick of the fall. Each tick's fall is S-0502's ordinary
    height step toward target height 0 plus the extra crash drop. Expect the extra drop (the fall
    beyond the ordinary climb-rate descent) to equal the lifted type's crash value in the first
    crashing update and to grow by 1 lepton per update after that, so that while the ordinary
    descent is steady, successive per-tick falls differ by exactly 1, with heights truncated.
    Seize and drop the same unit twice (it must survive the first drop over a clear cell, then be
    seized again): the second fall must again start from the type's crash value. A named L4 map
    may give the lifted type a non-integral crash value to expose the truncation. A seized
    crashing unit must never aim below the surface (target height 0, not −5).
  - Double credit: give the dropper a kill counter and the lifted unit low health and no armour
    against the impact warheads, so the first detonation kills it. Expect the dropper's kill
    tally and its house's tallies and score to rise by two for that one unit, and its
    experience to grow by twice the unit's value. A survivor removed by
    `quirks.lift_impact_removes_survivor` (variant (a)) must add one.
  - Passengers: drop a loaded transport onto a blocked cell. Expect its passengers to die
    credited to the attacker when a detonation kills it, and without attacker credit when it
    survives and is removed.
  - Named test-data variants:
    - (a) a multiplier of 0.5,
    - (b) the current-strength switch off,
    - (c) a crashable lifted type, which should give the ordinary crash end,
    - (d) a lifted landed aircraft, using a test weapon whose damage exceeds its size.
  - Name the overrides L4 impact map. Give test bystanders enough health and simple armour
    factors to distinguish damage from the two impact warheads without killing them. Compare
    every health delta with F2 and the ordinary area-damage rules. In (b), injure the victim
    before seizure so maximum and current health predict different amounts. Add a negative
    multiplier and an absent-key variant, and a non-integral product to distinguish truncation
    from rounding to nearest. Both detonations must use the amount computed before either.
  - In (d), give the two impact warheads distinct data-only damage modifiers. The aircraft must
    use the demolition modifier in both detonations, unlike a vehicle. Repeat with its
    crashable-type flag toggled: that flag must not replace the aircraft's magnetic impact.
  - Expected under (a): removal even when both defended detonations leave the victim alive.
    Under (b), the uninjured type strength supplies the base; negative damage becomes 0.
    Variant (c) follows the ordinary vehicle crash end, not the two magnetic detonations.
  - Falsified by: a wrong F2 delta, a crash rate that does not start at the type's value or
    grows before the first drop, a single credit for a detonation kill, recomputing damage after
    the first detonation, survivor retention, a crashable vehicle using magnetic damage, or an aircraft using crushing damage
    in its first detonation. Repeat a release above a bridge: stopping the fall anywhere other
    than the downward deck crossing or surface contact falsifies claim 15.
- **L5 Dropping on infantry and objects (claim 16).**
  - Variants: release over a cell holding infantry; infantry plus a vehicle listed after them; a
    tree; a landed aircraft.
  - Record which objects vanish without death animations, the kill counts, and whether the
    dropped unit survives (clear cell after erasure).
  - Expected: infantry erased without death effects, with each type's crush sound if present,
    and kills credited to the dropper (else its house). A tree or landed aircraft receives its
    own health as defence-ignoring crushing damage; subsequent landing depends on occupancy.
    A vehicle or building underneath takes no direct crushing damage from claim 16, though
    the later impact detonations may hurt it. Also drop onto a vessel: expect a blocked crash,
    not sinking.
  - Use named L5 map data with conspicuous death effects to distinguish erasure from ordinary
    damage, and distinct crush sounds. Repeat over a bridge with infantry and then a vehicle
    below the deck; their ground-layer presence, not deck occupants, decides claim 16.
  - The infantry-plus-vehicle-tail test is conditional: run it only if ordinary placement or
    movement establishes that list order without writing state. Otherwise its reachability
    remains unmeasured. Reverse entry order wherever legal to test the first-occupant rule.
  - Falsified by: infantry death effects, missing credited kills or configured crush sounds,
    direct damage to a first vehicle/building, the deck layer deciding instead of ground, or
    sinking after landing on a vessel. A later non-infantry occupant surviving the confirmed
    infantry-first list falsifies the cell-tail rule.
- **L6 Water, beach, bridge (claim 17).**
  - Release over deep water, over beach, and over a bridge deck above water.
  - Expected for an ordinary land vehicle:
    - blocked water: sinking and stun, no magnetic impact detonations, death credit to dropper
      or its retained house,
    - beach: clear occupancy gives harmless descent; blocked occupancy gives crashed impact,
    - clear deck: descent and touchdown on the deck, not sinking.
  - Repeat with an amphibious vehicle on water that it can occupy: expect harmless descent,
    not sinking merely because the land type is water. Named L6 map data gives a vehicle
    amphibious movement and toggles its jumpjet-type flag: the extra wheeled-clearance test
    may turn an otherwise clear water landing into sinking. Confirm both clearance results
    with the recorder; a clear result despite failed extra clearance falsifies claim 17. Repeat with a slave-bearing vehicle
    and record freed slaves and ownership under S-0312 claim 30's controller-death rule.
  - Falsified by: sinking on a clear legal water landing or on a deck, magnetic falling damage
    on the sinking branch, beach always causing sinking, or an ownership/credit result that
    differs from these rules.
- **L7 Lifter killed mid-lift (claims 12, 14).**
  - Destroy the lifter by friendly force-fire while it grips a unit over clear land and,
    separately, over infantry. Repeat by destroying the gripped unit itself. For the latter,
    observe that the surviving lifter loses its target. Separately destroy the dropper only
    after an ordinary move-order drop but before impact.
  - Expected: lifter removal drops its airborne victim; gripped-unit removal ends the link and
    clears the lifter's target. A surviving dropped unit retains the former dropper house for
    impact credit after the dropper disappears.
  - Falsified by: a grip surviving either participant's removal, no fall after lifter removal,
    a retained lifter target after victim removal, or impact kills assigned to the killer of
    the lifter rather than the former dropper house.
- **L8 Building lifter (claim 12).**
  - Named test data: a building with a magnetic-beam weapon and a seizing warhead. Seize a
    vehicle, then cut power (S-0405) and, separately, destroy the building.
  - Expected: a drop and cleared building target in both runs; record release in the same tick
    as the loss of operation/removal. A retained grip or target, or release only when the wave
    subsequently disappears, falsifies claim 12.
- **L9 Refusals and the building attack (claims 2, 9, 10, 11).**
  - Part a is conditional: try a pre-existing control group or queued move/stop order to the
    friendly victim. Record whether an order actually reaches destination handling after the
    seize. If one does, a non-empty destination must be ignored; clearing it must unlink
    without a crash drop and allow ordinary descent. Horizontal movement from its own move
    order, or a crash drop on destination clearing, falsifies claim 9. Deselection blocking the
    order is not confirmation of destination handling.
  - Part b: use an armed friendly victim with enemies in range and record shots from seizure
    through fall, touchdown and restoration. No shot is allowed while seized; any such shot
    falsifies claim 10. Absence of a target is not a firing-refusal test.
  - Part c: while gripping, force-fire at a second vehicle.
    - Expected: rearming until the wave's fade-out ends.
    - After that: a seize of the second vehicle and a drop of the first in the same tick.
  - Part d: while gripping, attack an enemy building (or force-fire at a friendly building).
    - Expected: the secondary weapon fires, the wave fades, and the gripped unit keeps being
      pulled in. Once other firing gates permit it, a new magnetic shot while an old wave is
      alive creates no second wave. A named L9 map may shorten the rearm delay to expose this.
    - Falsified by: a building refused solely by the grip/wave gate, the grip ending merely on
      retargeting, a second wave, or non-building fire before the live-wave grip gate ends.
  - Part f: while gripping, keep the lifter's target on its gripped unit with a non-magnetic
    weapon too (named L9 map data gives the lifter such a secondary weapon). Expect no shot at
    the gripped unit from any weapon, and the lifter's target to stay on it until arrival or
    another event of claim 12. A shot at the gripped unit, or a dropped target without such an
    event, falsifies claim 11.
  - Part g: seize a deployable vehicle and, separately, a loaded transport. After touchdown and
    before restoration (in *guard*, so no fall-back to rest), order deploy, undeploy and unload.
    Record whether each starts. Expect the ordinary result for a grounded unit; a refusal
    explained only by the seized state falsifies claim 10's last bullet.
  - Part e: named L9 map data supplies a slow seizing projectile. Launch it at an immune
    target while a grip exists, after any wave refusal has ended; expect release of the old
    victim without seizure of the immune one. Repeat with the firer destroyed, seized, or
    still gripping that very target when its delayed projectile arrives: expect no new seize
    and no release from that hit. Only use states reached through ordinary play. A contrary
    release/seize with the recorded guards falsifies claim 2; an unreachable variant remains
    unmeasured rather than confirmed.
- **L10 Wave lifetime (claim 21, F3).**
  - Named L10 map: remove the weapon's minimum range. Record actual firing/impact coordinates,
    not unit-centre distances. Test point separations below, exactly at and above 240 leptons
    according to F3, including a diagonal separation at which the table and exact roots differ.
    Expect seizure even without a wave at a sub-threshold separation.
  - Let a wave reach full strength, retarget and count its remaining updates: expect 20 fade
    updates by F4. Repeat retargeting after several different fade-in lengths, including the
    first available tick; F4 predicts the shorter lifetime from the retained strength.
    Separately remove target and firer to test both ending conditions. With stationary
    participants and unchanged targeting, observe well past 100 ticks: no fixed expiry.
  - Update turn: retarget the firer and record the tick in which the wave begins fading; expect
    the same tick. Separately, let the wave's target enter a transport or a bunker; expect the
    wave to begin fading in the next update, while the target is unharmed. A named L10 map may
    place the firer and target so that the squared horizontal distance wraps negative; expect
    creation exactly as F3's mirrored root predicts, if such a separation is reachable.
  - For no-damage testing, keep a non-seizing magnetic shot's ordinary impact outside nearby
    test units and record their health along the wave path: the wave alone must not hurt them.
  - Falsified by: wave creation below F3's threshold, missing creation at/above it with a free
    slot, failure to seize solely because no wave exists, a lifetime inconsistent with F4,
    fixed expiry with unchanged targeting, or health loss attributable only to the wave.
- **L11 Restoration and computer houses (claims 6, 20; parts c and d for claim 20's later bullets).**
  - Part a: after L3, record the tick the unit first accepts a move order.
  - Part b: repeat L3 against an isolated unit of the computer opponent, excluding resource
    gatherers and passive countries. Record its queued hunt at seizure, rather than inferring
    it from later AI movement. A second run uses a resource gatherer and records its harvest
    queue at restoration; a human-owned control must not receive either computer-only queue.
    A passive-country control must not receive the seizure hunt.
  - Part c: after a clear-cell touchdown, leave the unit alone for well over 100 ticks; expect
    it still seized (its own movement not back). Then give two move orders a few ticks apart.
    Expect the first to move it nowhere and the second to move it with its own locomotor.
  - Part d: seize a vehicle while it is mid-turn or mid-step and give it an order that waits for
    readiness; expect readiness to follow the jumpjet's *moving now* state, not its earlier
    driving state.
  - Expected: no restoration while gripped, no accepted own movement before restoration,
    restoration at the first eligible fall-back after touchdown, and the queues above.
    Falsified by earlier restoration, failure to restore when resting and set down, or a
    queue contrary to the owner/type conditions. Keep incidental AI orders out of the observed
    interval, or mark the run void.
- **L12 Seizing from a moving lifter (claims 7, 8).**
  - Let the lifter drive across clear land and fire at a vehicle as it goes (named L12 map data
    may give a mobile lifter a moving-fire ability or a short rearm delay); separately, use a
    building lifter.
  - Record the lifter's heading point at the seize, the gripped unit's movement point and its
    arrival cell, and the lifter's position at arrival.
  - Expected: the movement point is a free cell beside the lifter's heading point at the seize
    (the foundation centre or docking point for a building) and does not follow the lifter
    afterwards; the unit drops over that cell and lands unharmed when it is clear.
  - Falsified by: a movement point that follows the moving lifter, an arrival over the lifter's
    current position after it moved, or a drop onto an occupied cell when a free neighbour
    cell existed.
- **L13 Other locomotors (claim 22).**
  - Named L13 map data: four seizing warheads naming hovering, flying, driving and walking
    movement, each fired at an eligible vehicle.
  - Expected: with hovering and flying, the target stops, enters *guard*, loses spawned objects,
    is not lifted and keeps its own movement, and can be hit again with the same result; with
    driving and walking, it is seized (it ignores orders and cannot fire). The firer's previous
    victim is dropped in all four.
  - Falsified by: a seize with hovering or flying, a missing stop, or no seize with driving or
    walking.
- **S-0502 carry items.** L2, L3 and L4 together give the held body facing (S-0502 claim 10), the
  hovering, landing and arrival exclusions (claims 12 to 15), the growing crash rate (claim 18),
  the cell-centre detour (claim 15 here), and the sinking-only-after-a-drop case (S-0502 claim 19,
  via L6).

## Open questions

- Whether any player order can reach a gripped unit after it is deselected, which would exercise
  claim 9's no-destination exception. Our engine follows claim 9 if one does.
- How the factory-contact protection of claim 3 can be reached in play.
- Whether a squared sum that wraps negative in F3 is reachable on a playable map. F3 gives its
  result if it is.
- How an empty or malformed locomotor value on a seizing warhead is parsed (claim 22), and how a
  unit seized with a stand-in locomotor other than the jumpjet's moves and is restored. Both are
  outside this revision.
- Deploying, undeploying and unloading while seized (claim 10's last bullet) rest on the absence
  of any seized-state test, not on a traced refusal; low confidence until scenario L9 part g.
