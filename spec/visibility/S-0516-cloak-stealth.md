---
id: S-0516
title: Cloak and stealth (cloaking, cloak fields, sensors, disguise)
topic: visibility
revision: 4
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0516: Cloak and stealth (cloaking, cloak fields, sensors, disguise)

## Summary

This spec covers how objects hide from other houses and how other houses find them again.
- **Cloaking.** The four cloak states, when an object starts and stops cloaking, how long each
  transition takes, the delay before it may cloak again, and what forces it back into view.
- **Cloak fields.** The area that a cloak-generating building hides.
- **Sensors.** The per-house coverage that lets a house see and attack cloaked objects.
- **Disguise.** How a unit takes on a disguise, whom the disguise fools in game logic, how each
  viewer sees it drawn, and how it is lost or detected.
- **Submarines.** The rules that turn this machinery into submerging and surfacing for
  underwater vessels.

Two kinds of rule are kept apart throughout.
- **Game state.** What a house may target or fire on. It never depends on who is watching.
- **Presentation.** What a viewing house *V* is shown: images, translucency, clicks and the radar.

**What this spec does not restate.**
- **Owned elsewhere, used here:**
  - firing answer *cloaked*: S-0301 claim 12
  - naval weapon choice against a submerged target: S-0308 claim 11 and table N
  - acquisition rows for cloaked and disguised targets: S-0309 claim 12 and table C
  - damage-state result and the disguise-disruption countdown it starts: S-0302 claim 12
  - drawing an infantry unit through its disguise: S-0107 claim 13
  - veterancy insignia: S-0111 claim 18.
- **Defined elsewhere:**
  - shroud: S-0409. Shroud sight is not sensor coverage; neither replaces the other
  - the cloaked-blocker answer to movement: S-0209
  - crates that grant cloaking: S-0520 claim 15.
- **Out of scope:**
  - psychic detection
  - the capture-the-flag mode
  - campaigns
  - network play.

## Terms

- **Cloak state.** One of *uncloaked*, *cloaking*, *cloaked* (also *fully cloaked*) and
  *uncloaking*.
  - An object that is cloaking or uncloaking is *fading*.
  - *Uncloaked* is the same as *fully visible* in S-0308.
- **Stage.** A fading vehicle, infantry unit or aircraft holds an integer stage *k*.
  - *k* moves by one step every *S* ticks (claim 7).
  - *N* is the game-wide number of cloaking stages.
  - The stage's *fade fraction* is *x* = ⌊256 *k* / *N*⌋.
  - A building has a *fade level* (0 to 15) instead (claim 18).
- **Can cloak by itself.** An object that has its own cloak flag (claim 2) and is not barred by
  cloak-stop (claim 3).
- **Cloak delay.** A per-object countdown, in ticks, set when an uncloak completes. The object
  cannot start cloaking again while it runs.
- **Cloak field.** The set of cells that a powered cloak-generating building of house *H* hides
  for *H* (claims 19 and 20). A cell is either in *H*'s cloak field or not; overlaps are not
  counted.
- **Sensor coverage, sensed cell.** For each house *H* and cell *c*, a signed count. The cell is
  *sensed by H* when the count is above 0 (claims 21 to 23).
  - An object is *sensed by H* when the cell under its centre is sensed by *H*.
  - The coverage belongs to the owner only. Allies do not share it.
  - This is the *sensed cell* of S-0309 (Terms).
- **Sensor disc of radius R.** The cells (*X* + *dx*, *Y* + *dy*) around a centre cell (*X*, *Y*)
  whose integer displacements satisfy all of:
  - −*R* ≤ *dx* ≤ *R* − 1
  - −*R* ≤ *dy* ≤ *R* − 1
  - *dx*² + *dy*² < *R*².

  For *R* ≤ 0 the disc is empty. For *R* = 1 it is the centre cell alone. The disc is symmetric:
  the strict squared-distance test excludes both endpoints at distance *R* on each axis.
- **Hidden from H.** An object is *hidden from house H* when it is fully cloaked and not sensed by
  *H*. A building also counts as hidden from *H* at fade levels 11 to 15 when *H* does not sense it.
  - This is a game-state test.
  - It holds for the owner too: the owner does not sense its own cloaked object unless it covers
    the cell with a sensor.
- **Disguise.** A unit's disguise has three parts:
  - a *disguised* flag
  - a *disguise type*: an infantry type, a vehicle type, an overlay type or a terrain-object type
  - a *disguise house*, which may be *none*.
- **Disguise-detector coverage.** A second per-house, per-cell signed count, kept like sensor
  coverage but fed only by detector buildings (claim 40). *H* has detector coverage on a cell when
  the count is above 0.
- **Appears disguised to H.** The game-state test of claim 35. Every logic rule that a disguise
  affects uses it.
- **Drawn as its disguise for V.** The presentation test of claim 37. It says what viewing house *V*
  is shown, and it never feeds game state.
- **Fake blink.** A per-unit countdown, in ticks, started when a disguise-when-still vehicle fires
  (claim 39). While it runs, other houses are shown the real unit.
- **Re-disguise lockout.** A per-unit countdown, in ticks. While it runs, a disguise-when-still
  unit cannot take a new disguise (claim 33).
  - Damage sets it, and S-0302 claim 12 calls it the *disguise-disruption countdown*.
  - Adjacent enemy infantry also set it.
- **EMP-disabled, paralysed.** The cloak rules make two disable tests and treat them alike: being
  under an electromagnetic pulse, and being *paralysed* in S-0214's sense. Which effects set each
  belongs to the special weapons specs. *Warping* means *being erased* (glossary) or being in a
  chrono warp.

## Behaviour

### Cloaking

1. **States and stages.** Vehicles, infantry and aircraft move through the cloak states as
   follows:
   - uncloaked → cloaking → cloaked
   - cloaked → uncloaking → uncloaked
   - cloaking → uncloaking, on an abort or an uncloak cause.

   The stage *k* counts up while cloaking and down while uncloaking, and it is 0 in the uncloaked
   and cloaked states. Buildings use a fade level instead (claim 18). *S* is the type's
   cloaking-speed field, in ticks per stage, and *N* is the game-wide stage count. Both are
   integers (Import mapping).
   - Backing: `measurable` (CS1)
   - Confidence: high

2. **Where the ability to cloak comes from.** An object may cloak when at least one of these holds:
   - **(a) Its own cloak flag.** A vehicle or infantry unit takes it from its type's cloak flag when
     it is created.
   - **(b) A cloaking veterancy ability.** It holds the ability at its current rank: a veteran
     uses the veteran ability list; an elite uses either the veteran or the elite list.
   - **(c) Its owner's cloak field.** The cell under its centre is in its owner's cloak field
     (claim 19).
   - **(d) A cloaking crate.** The crate sets the object's own cloak flag (S-0520 claim 15).

   Aircraft and buildings do not take the flag from their type: the type's cloak flag has no effect
   on them (`quirks.cloak_flag_ground_mobile_only`). A building can nevertheless cloak through a
   cloak field, a crate-set own flag, or a cloaking veterancy ability.
   - Backing: `measurable` (CS2)
   - Confidence: high

3. **Cloak-stop.** For a type with the cloak-stop flag, the own cloak flag of claim 2 (a) only lets
   the object start cloaking while its locomotor is not moving.
   - Once the object is fully cloaked, moving does not uncloak it.
   - The flag does not stop cloaking through (b), (c), or through claim 6. A crate sets the same
     own flag as (a), so it does not bypass cloak-stop outside a field.
   - Backing: `measurable` (CS2, part c)
   - Confidence: high

4. **Ready to cloak.** An object that may cloak (claim 2) is *ready to cloak* unless at least one of
   these holds:
   - it is fully cloaked
   - its rearm countdown is running
   - it has a target within its attack range
   - it is not a building and its stage is not 0, so a vehicle, infantry unit or aircraft that is
     uncloaking cannot turn back before its uncloak completes
   - its cloak delay is running
   - it is held by a magnetic beam
   - its height above the ground is 1 lepton or more.

   The last condition means an aircraft can start cloaking only while landed, and a jumpjet only on
   the ground.
   - Backing: `measurable` (CS3)
   - Confidence: high

5. **Starting to cloak by itself.** In its turn, an uncloaked object starts cloaking when all of
   these hold:
   - **(a)** One of these:
     - it can cloak by itself and is not EMP-disabled, paralysed or warping
     - it holds a cloaking veterancy ability, which is not affected by those states.
   - **(b)** It is not in contact with a building that is a war factory.
   - **(c)** It is ready to cloak.
   - **(d)** Its health passes this test:
     - if its health ratio *h*/*h*ₘₐₓ, computed in double precision, is strictly greater than the
       game-wide red ratio, it starts at once
     - otherwise one ranged draw R(0, 99) is taken from the scenario random stream, and it starts
       when the result is below 4.

   A damaged object below the red line therefore starts with probability 4/100 on each tick on
   which (a) to (c) hold, one draw per such tick. The draw is taken in the object's own turn of the
   update order (S-0203).
   - Backing: `measurable` (CS3, CS4)
   - Confidence: high

6. **Cloaking inside an own cloak field.** An uncloaked object is re-evaluated in these cases:
   - on each tick on which the cell under its centre is in its owner's cloak field
   - when it enters a cell
   - when the cloak field or the sensor coverage over its cell changes.

   If its cell is in its owner's cloak field and it is ready to cloak, it starts cloaking at once.
   - There is no health test and no draw.
   - The EMP, paralysis and warping gate of claim 5 (a) does not apply.
   - The war-factory contact test of claim 5 (b) does not apply.

   The same re-evaluation deselects a fully cloaked object (claim 26).
   - Backing: `measurable` (CS5)
   - Confidence: high

7. **Cloaking and uncloaking durations.** Every *S* ticks of a fading object's turns, the stage
   moves one step, up while cloaking and down while uncloaking. The stage never goes below 0.
   - **Cloaking.** It starts at stage 0. The object becomes fully cloaked when the stage first
     reaches *c* = ⌈*N*/2⌉: the first stage whose fade fraction *x* is at least 128. The stage then
     returns to 0.
   - **Uncloaking.** It always starts at stage *N* − 1, even when the object was still cloaking
     (`quirks.uncloak_from_cloaking_full_duration`), and it completes when the stage reaches 0.
   - **Durations** (formulas F1 and F2):
     - fully cloaked *c* · *S* ticks after cloaking starts
     - uncloaked (*N* − 1) · *S* ticks after uncloaking starts.
   - **Cloak delay.** When an uncloak completes, the cloak delay starts at *D* ticks (F3).
   - Backing: `measurable` (CS1)
   - Confidence: high (the formulas); medium (the exact tick, ±1, at which each state change is
     first observable)

8. **Abort at low health while cloaking.** On each tick of a cloaking object's turn on which:
   - its fade fraction is in the 50 % band (64 ≤ *x* < 128), and
   - its health ratio is at or below the red ratio,

   one ranged draw R(0, 99) is taken. A result below 10 makes it start uncloaking (claim 7, from
   *N* − 1). This abort plays no cloak sound.
   - Backing: `measurable` (CS4, part c)
   - Confidence: high

9. **What keeps a cloaked object cloaked.** On each tick of its turn, a fully cloaked object
   starts uncloaking, with the cloak sound, unless one of these holds:
   - **(a)** it can cloak by itself, or its own cloak flag is set, and it is not EMP-disabled,
     paralysed or warping
   - **(b)** it holds a cloaking veterancy ability
   - **(c)** the cell under its centre is in its owner's cloak field.

   An EMP hit therefore uncloaks a self-cloaking object outside its field. An object that
   cloaked only by a field uncloaks when it leaves the field or the field shrinks away from it.
   - Backing: `measurable` (CS5, part b; CS6)
   - Confidence: high

10. **No re-cloak while uncloaking.** A vehicle, infantry unit or aircraft that is uncloaking always
    completes the uncloak and then waits out the cloak delay before claim 5 or 6 can start a new
    cloak. A building's field-triggered reversal differs (claim 18).
    - Backing: `measurable` (CS1, part c)
    - Confidence: high

11. **What makes an object uncloak.** Besides claims 8 and 9, an object starts uncloaking (claim 7)
    in each of these cases:
    - **(a) Firing.** It tries to fire with a weapon that gets the answer *cloaked* (S-0301
      claim 12). This applies to vehicles that are within range of their target, aircraft,
      infantry and buildings.
    - **(b) C4.** An infantry unit plants a demolition charge (C4).
    - **(c) Crushing.** A vehicle crushes something.
    - **(d) Hits.** It is hit (claim 12).
    - **(e) Blocked movers.** A moving unit finds it as a cloaked blocker. Every cloaked object in
      the ground layer of the blocking cell, of any house, starts uncloaking (claim 12). This is
      a real uncloak, seen by every house, not a sight for the mover's house alone.
    - **(f) Adjacent sensors.** It moves next to an enemy sensor unit (claim 13).

    These causes play the game-wide cloak sound when they actually start an uncloak.
    - Backing: `measurable` (CS7)
    - Confidence: high ((a), (b), (d), (e)); medium ((c))

12. **Hits uncloak.** A hit on a surviving cloaked or cloaking object makes it start uncloaking
    when ordinary damage resolution reports *unaffected*, *unchanged*, *now yellow* or *now red*
    (S-0302 claim 12). A killing hit does not reach this uncloak reaction. Surviving hits include:
    - healing
    - a hit whose final amount is 0
    - a hit that reports *unaffected* because of the target's armour (S-0302 claims 6 and 8).

    A hit that stops before that point does not uncloak. Such hits are:
    - a non-negative hit on a temporarily invulnerable target
    - a hit on a warping target
    - a hit blocked by a tank bunker sheltering the target, not by an infantry garrison
    - a hit by a warhead that the target is immune to by kind (radiation, psionic, poison)
    - a hit that a warhead not affecting allies makes on an allied target
    - a berserk-warhead hit
    - a hit from an attacker of the target's own type and owner, when the target's type is immune
      to its own type.

    The invulnerability and warping exclusions above describe ordinary hits, not hits that
    explicitly bypass those defences (S-0302 claim 10).

    A medic or a repair weapon healing a cloaked ally therefore uncloaks it
    (`quirks.heal_reveals_cloaked`).
    - Backing: `measurable` (CS7, parts d and e)
    - Confidence: medium (the exact list of hits that do not uncloak)

13. **Adjacent sensor reveal.** When a fully cloaked vehicle, infantry unit or aircraft enters a
    cell, the game checks each of the 8 neighbouring cells that pass the usable-area test.
    - For each such cell it takes a single candidate: the object of that cell's ground layer that
      is nearest the cell's top corner.
    - The moving object starts uncloaking at once if that candidate:
      - is not allied with the mover's owner, and
      - has its type's sensors flag or a sensors veterancy ability.
    - Only the candidate is tested. A sensor standing in the cell behind another object does not
      count.
    - Only a cloaked object that moves is revealed this way. A sensor that walks up to a stationary
      cloaked object reveals nothing by adjacency; its coverage (claim 22) only lets its house see
      and attack the object.
    - Backing: `measurable` (CS8)
    - Confidence: medium

14. **Targets when an object cloaks.** Two moments affect attackers of vehicles, infantry and
    aircraft:
    - when the target starts cloaking
    - when it becomes fully cloaked.

    A building also triggers the start-of-cloak target loss, but its completed fade does not
    repeat this target-loss event.

    At each moment, every object that has it as its target keeps that target only if one of these
    holds (S-0215 claim 10):
    - the attacker's house senses the cloaking object's cell
    - the attacker has the same owner.

    Any other attacker drops the target.
    - Backing: `measurable` (CS9)
    - Confidence: medium

15. **Cloak sound.** The game-wide cloak sound plays at the object when it starts cloaking by
    claims 5, 6 or 18, and when it starts uncloaking by claims 9, 11 or 18.
    The low-health abort of claim 8 is silent. Repeating an uncloak cause while already uncloaking
    plays no additional cloak sound. A type-specific cloak sound is not used here.
    - Backing: `measurable` (CS1, part d)
    - Confidence: medium

16. **Vehicles placed already cloaked.** A vehicle whose own cloak flag is set enters play fully
    cloaked, whether it leaves a factory or is placed on the map. This does not apply to infantry
    and aircraft, which start uncloaked and cloak by claim 5.
    - Backing: `measurable` (CS2, part a)
    - Confidence: high

17. **Aircraft.** An aircraft cloaks only while landed (claim 4). Its firing answer *cloaked* is
    S-0301 claim 12's. Once airborne and fully cloaked, it stays cloaked under claim 9.
    - Backing: `measurable` (CS2, part d)
    - Confidence: medium

18. **Buildings.** A building's fade level runs from 0 (uncloaked) to 15 (cloaked).
    - **Steps.** While cloaking it rises by one per tick, and while uncloaking it falls by one per
      tick. Each direction takes 15 ticks. The cloaking speed and the stage count play no part.
    - **Reversal.** Losing eligibility does not by itself reverse a building that is still
      cloaking: it finishes the fade and then starts uncloaking. An uncloaking building can
      reverse when its cell is re-evaluated by claim 6 and it is ready to cloak inside its own
      field. That reversal preserves its current fade level. Its ordinary tick update alone
      does not reverse uncloaking before level 0. Neither start has a health test or a draw.
    - **Delay.** Reaching level 0 does not start the mobile object's cloak delay.
    - **State changes.** At level 15 the building is fully cloaked; its natural particle effects
      stop. At level 0 it is uncloaked; they restart.
    - **Sensor ring.** A building is not ready to cloak, and a cloaked building starts uncloaking,
      while one of the following stands in its foundation or in the 1-cell ring around it:
      a unit whose type has the sensors flag and whose owner is not the building's owner.

      Allies' sensor units count, and a sensors veterancy ability does not
      (`quirks.building_sensor_ring_owner_only`). Each cell uses the nearest candidate of claim 13,
      except that when the candidate is the building itself, the first infantry occupant is
      tested instead.
    - Backing: `measurable` (CS10)
    - Confidence: high (fade timing); medium (the sensor ring)

### Cloak fields

19. **Shape of a cloak field.** A cloak generator of radius *r* covers a filled circle of cells
    around the cell of its centre. The circle is defined by the half-width table that the midpoint
    recurrence F5 produces:
    for each step (*x*ᵢ, *y*ᵢ), the rows at *dy* = ±*x*ᵢ cover |*dx*| ≤ *y*ᵢ, and the rows at
    *dy* = ±*y*ᵢ cover |*dx*| ≤ *x*ᵢ.

    For *r* = 20 this is exactly the cells with *dx*² + *dy*² ≤ *r*² + *r*. For some other radii,
    such as 1, 4, 6, 8 and 9, the two shapes differ in a few cells, and the recurrence is the
    authority.
    - Backing: `measurable` (CS11)
    - Confidence: high

20. **Growth, shrinking and ownership.**
    - **Growth.** When a cloak generator comes into operation, its field grows from radius 0 by one
      ring per tick up to the type's cloak radius. Every object on a newly covered cell is
      re-evaluated (claim 6).
    - **Shrinking.** When the generator goes out of operation, the field shrinks by one ring per
      tick to 0. Cells outside the shrinking circle leave the field at once, and their objects are
      re-evaluated (claim 9 (c)).
    - **Removal.** When the generator is removed, sold or destroyed, the field vanishes at once.
    - **Capture.** When it is captured, the field also vanishes at once, and it grows again for the
      new owner if the building is in operation.
    - **Ownership.** A field covers its owner's objects only, not its allies'.
    - **Overlaps.** Coverage is a yes-or-no flag per house and cell, not a count. When one of two
      overlapping generators of the same house shrinks, the shared cells leave the field even
      though the other generator still covers them. When the shrinking generator reaches radius 0,
      every other powered, idle generator whose centre is closer than 2 (*r* + 2) cells to the
      shrinking one's centre, *r* being the shrinking generator's type radius, restarts its growth
      from 0 if previously at its full radius. This restart is not restricted to the same owner. Until then, objects in the gap may uncloak
      (`quirks.cloak_field_overlap_loss`).
    - **Buildings within a ring.** Covering an own building's centre can interrupt application
      of a growing ring. Whether all its cells eventually receive coverage is not established
      (Open questions). The uninterrupted one-ring-per-tick rule applies to open ground.
    - Backing: `measurable` (CS11, CS17), except coverage after an interrupted ring is `unmeasured`:
      the interruption was identified but its eventual coverage result was not traced.
    - Confidence: high (uninterrupted growth and shrink rate, owner only), medium (restart distance),
      low (coverage after a ring is interrupted)

### Sensors

21. **Sensor coverage.**
    - **Counts.** Each house *H* keeps, per cell, a signed count of sensor coverage. A cell is
      sensed by *H* only while its count is above 0.
    - **Ownership.** Coverage is added and removed only for the sensor's owner. An ally's sensors do
      not let *H* see or attack anything.
    - **Shroud.** Coverage does not lift shroud (S-0409). The cloak-specific tests of claims 24
      to 28 do not themselves test shroud; this does not override separate shroud restrictions
      on drawing, selection or targeting.
    - **Re-evaluation.** Every time coverage is added or removed on a cell, each vehicle, infantry
      unit and aircraft in that cell's ground layer is re-evaluated (claim 6). A building of
      another house on that cell is redrawn.
    - Backing: `measurable` (CS12)
    - Confidence: high

22. **Mobile sensors.** A vehicle, infantry unit or aircraft whose type has a non-zero sensors-sight
    *R* covers the sensor disc of radius *R* around its cell for its owner. The sensors flag alone
    gives no coverage; it serves claims 13 and 18.
    - **Entering play.** The disc is added when the object enters play.
    - **Movement.** When the object changes cell, the disc is removed around the old cell and added
      around the new one.
    - **Owner change.** The disc moves from the old owner to the new one.
    - **Leaving play.** The disc is removed around the object's current cell.
    - **Floor.** Removal never takes a mobile sensor's count below 0.

    A mobile type's disguise-detection range has no effect (claim 40).
    - Backing: `measurable` (CS12, parts a and b)
    - Confidence: high (disc, owner only); medium (the tick at which the disc moves)

23. **Sensor-array buildings.** A building whose type is a sensor array adds the sensor disc of
    radius *R* (its sensors-sight) for its owner.
    - **Activation.** It does so only at its first placement, and only if it is in operation at that
      moment.
    - **Power.** Power changes never add or remove the coverage afterwards. An array that is placed
      unpowered never senses, even after power returns
      (`quirks.sensor_building_activation_once`).
    - **Removal.** When the array leaves play (sale, destruction, removal), the game decrements its
      *current* owner's counts:
      - over the disc of radius equal to the type's **cloak-radius** field, not its sensors-sight
        (`quirks.sensor_array_removal_radius`)
      - with no floor, so counts can fall below 0 (`quirks.sensor_count_negative`).

      A cell left at −1 needs two later sensors before it is sensed again.
    - **Capture.** The old owner keeps the coverage. The new owner gets none, and the array's later
      removal decrements the new owner's counts.
    - Backing: `measurable` (CS12, parts c to e)
    - Confidence: high

### Hidden objects: game state

24. **Firing at a hidden target.** The firing check of S-0301 answers *impossible* when the target is
    hidden from the firer's house and at least one of these holds:
    - the firer's mean weapon damage (S-0308 claim 15) is above 0
    - the target is not allied with the firer.

    Mean weapon damage is a target-independent integer, including the ambient component and
    truncation toward zero defined by S-0308 claim 15, not the selected weapon's damage or shot
    damage after firepower or armour scaling. A forced attack on such a target is refused, even
    against one's own unit. An allied hidden target passes this test when the firer's mean weapon
    damage is at or below 0; all other firing restrictions still apply. In particular, selecting a
    healing weapon does not bypass a positive mean weapon damage.
    - Backing: `measurable` (CS9, part c)
    - Confidence: high

25. **Aircraft drop hidden targets.** An aircraft whose target is fully cloaked, not allied with it
    and not sensed by its house drops the target.
    - Backing: `measurable` (CS9, part d)
    - Confidence: medium

### Hidden objects: presentation for viewing house V

26. **Clicks, hover and selection.** For viewing house *V*, the following cannot be hovered, clicked
    or picked:
    - a fully cloaked object that *V* does not own and does not sense, allies included
    - a building at fade level 15 that *V* does not sense.

    Fading objects that are drawn can be clicked, subject to the ordinary selection rules.
    Selection changes as follows:
    - an object that starts cloaking leaves a non-owner viewer's selection even if that viewer
      senses it
    - a fully cloaked object leaves the selection of a non-owner player who does not sense it
      whenever it is re-evaluated (claim 6).
    - Backing: `measurable` (CS9, part e)
    - Confidence: medium

27. **Radar.** An object of another house that is cloaked appears on *V*'s radar only where *V*
    senses its cell. This includes an ally's object. The radar rules themselves are S-0410's.
    - Backing: `measurable` (CS9, part f)
    - Confidence: medium

28. **How V sees a cloaking or cloaked vehicle, infantry unit or aircraft.**
    - **Uncloaked:** drawn normally.
    - **Fading at stage *k* > 0,** by fade fraction *x*:

      | *x* | Drawn |
      |---|---|
      | < 64 | 25 % translucent |
      | 64 to 127 | 50 % translucent |
      | 128 to 191 | 50 % translucent ("shadowy") |
      | 192 to 254 | Rippling, warped image for every viewer except the owner, who sees it 50 % translucent |
      | ≥ 255 | Not drawn for non-owners; the owner sees it 50 % translucent |

      Sensors play no part while fading.
    - **Fully cloaked:** drawn 50 % translucent for *V* when at least one of these holds:
      - *V* owns it
      - *V* senses its cell
      - *V* and its owner are mutually allied.

      Otherwise it is not drawn at all for *V*.
    - **Buildings:** fade levels 1–5 draw 25 % translucent and 6–10 draw 50 % translucent. Levels
      11 to 15 follow the fully cloaked rule.
    - Backing: `measurable` (CS1, part b; CS12)
    - Confidence: high

29. **Shadows.** A vehicle's shadow is not drawn, for any viewer, while its cloak state is anything
    but uncloaked.
    - Backing: `measurable` (CS1, part b)
    - Confidence: medium

### Disguise

30. **Disguise state.** A unit is *disguised* or not. While disguised it has a disguise type and a
    disguise house (possibly none). Claims 31 to 33 say how it becomes disguised.
    - Backing: `measurable` (CS13)
    - Confidence: high

31. **Disguising by a hit.** When a projectile whose warhead has the disguise flag detonates on its
    target, it disguises its firer and does no damage to the target. How the firer copies the target
    depends on the firer:
    - **An infantry firer** copies only infantry targets. If the target is disguised, the firer
      copies its disguise type and disguise house. Otherwise it copies the target's type and
      owner.
    - **A vehicle firer** copies in these cases:
      - a disguised vehicle target: its disguise
      - a rock overlay target: the overlay, with house none
      - a terrain-object target: the terrain object, with house none.

      Other targets leave the firer unchanged.

    The weapon's range and its targeting are the ordinary ones (S-0307, S-0309). There is no
    disguise-specific range; a weapon with S-0307's unlimited range disguises from any distance
    without moving.
    - Backing: `measurable` (CS13, part a)
    - Confidence: high (infantry); medium (vehicle)

32. **Permanent disguise.** A unit whose type can disguise and disguises permanently is created
    disguised, as follows:
    - its disguise type is the game-wide default disguise of its owner's side
    - its disguise house is its owner
    - the side defaults are: the first side, the allied default; the second side, the soviet
      default; any other side, the third default.

    Because the disguise house is the owner, this default fools no other house in game logic
    (claim 35); it only changes how the unit is drawn (claim 37). When a permanently disguising
    infantry unit is made to clear its disguise (claim 34), it returns to this default instead
    of becoming undisguised. Vehicles do not have this reset: clearing their disguise makes them
    undisguised even if their type has the permanent flag.
    - Backing: `measurable` (CS13, part b)
    - Confidence: high

33. **Disguise when still (vehicles).** These rules apply, on every tick, to each vehicle whose
    type can disguise but does not disguise permanently. They never apply to infantry.
    - **(a) Moving.** A vehicle whose locomotor is moving is not disguised: it loses any disguise on
      each tick on which it moves. This holds for every such vehicle, not only for
      disguise-when-still types.
    - **(b) Neighbouring infantry.** On every tick whose number is not a multiple of 8, the
      vehicle's 8 neighbouring cells are examined, on the bridge layer if it stands on a bridge.
      - In each cell only the first infantry unit the cell lists counts.
      - If that infantry is not allied with the vehicle's owner (neutral counts as not allied), the
        vehicle loses its disguise and its re-disguise lockout becomes the game-wide
        infantry-blink time.
      - The examination happens on 7 ticks of every 8 (`quirks.mirage_adjacent_check_cadence`).
    - **(c) Taking a disguise.** It takes a disguise on a tick on which neither (a) nor (b) applied
      and all of these hold:
      - it is not disguised
      - it is still
      - its type disguises when still
      - it is in contact with nothing
      - its re-disguise lockout has run out.

      One ranged draw R(0, *n* − 1) is then taken, where *n* is the length of the game-wide
      default-disguise list. The disguise type becomes that list entry, and the disguise house
      becomes none.
    - Backing: `measurable` (CS14)
    - Confidence: high (vehicles only, the draw); medium (first infantry per cell)

34. **Losing a disguise.** A unit loses its disguise when one of these happens:
    - **(a) Damage.** The hits of S-0302 claim 12 clear it and set the re-disguise lockout, whose
      length is given there. This applies to infantry and vehicles alike. The lockout only matters
      to a vehicle, through claim 33 (c).
    - **(b) Moving,** for vehicles only: claim 33 (a).
    - **(c) Neighbouring infantry,** for vehicles only: claim 33 (b).
    - **(d) Beginning to undeploy.** An in-place deploying vehicle loses its disguise when its
      undeploy step starts, before an animated undeploy finishes (S-0517 claim 6).
    - **(e) Disguise house leaves the game.** The house its disguise names leaves the game.

    Infiltration uses up the spy rather than leaving a surviving unit whose disguise could be
    observed (S-0510 claim 2). Disguise is preserved through:
    - entering a transport or a building without being used up
    - a change of owner, including mind control
    - being detected.
    - Backing: `measurable` (CS13, part c; CS14, part c)
    - Confidence: high

35. **Appears disguised to H (game state).** A unit *appears disguised to house H* when all of
    these hold:
    - it is disguised
    - its owner does not ally *H*
    - *H* has no disguise-detector coverage on the cell under its centre
    - its disguise house is *H*, or none, or a house that *H* allies.

    A disguised building or aircraft appears disguised to every house, whatever the alliances and
    detector coverage (Open questions). Detection by mobile
    detector units does not enter this test: it acts only in acquisition (S-0309 claim 12, table C).
    - Backing: `measurable` (CS15)
    - Confidence: high

36. **What appearing disguised changes.** Each of the following uses claim 35 and is owned by the spec
    named:
    - target acquisition: S-0309 claim 12, table C, disguise row
    - an object does not retaliate against an attacker that appears disguised to its house
    - automatic crushing target choice excludes a unit that appears disguised to the crusher's
      house (S-0210 claim 11). This exclusion does not prevent ordered drive-through crushing
      under S-0210 claims 2–3
    - the infantry cell-entry answer: S-0209
    - the attack cursor: S-0214 claim 6
    - the veterancy insignia: S-0111 claim 18
    - the hover health bar: S-0111 claim 3, shown only when the disguise has a house
    - the hover name.

    A unit disguised as a terrain object or overlay that appears disguised to the viewing house
    cannot be selected by it, and does not flash when targeted.
    - Backing: `measurable` (CS15)
    - Confidence: medium

37. **Drawn as its disguise for V (presentation).** For viewing house *V*:
    - **(a) Fake blink.** While the unit's fake blink runs, every *V* other than the owner is shown
      the real unit.
    - **(b) Owner.** If *V* owns the disguised unit, let *t* = (current tick − tick the disguise was
      taken) mod 256.
      - For *t* in 8 to 55, *V* is shown the disguise.
      - Otherwise *V* is shown the real unit.
      - Near the switches, the image is drawn translucent as in table B.
    - **(c) Others.** Otherwise, *V* is shown the disguise when the unit is disguised and *V* has no
      detector coverage on its cell. Allies of the owner see the disguise too.

    The disguise is drawn in its disguise house's colours. A unit disguised as an overlay or terrain
    object is lit like that object.
    - Backing: `measurable` (CS16)
    - Confidence: high

38. **Radar colour.** For viewing house *V*, a disguised unit appears on the radar in its owner's
    colour when its owner allies *V*, including when *V* is the owner. Otherwise it appears in the
    disguise house's colour. This is the one-way owner → viewer relation, not viewer → owner.
    Mutual alliance is not required. The colour of a unit whose disguise house is none is an
    open question.
    - Backing: `measurable` (CS16, part c)
    - Confidence: medium

39. **Fake blink.** When a vehicle whose type disguises when still fires and creates a projectile,
    its fake blink is set to the fired weapon's fake-blink time, in ticks, whether or not the vehicle
    is disguised.
    - It does not remove the disguise.
    - While it runs, claim 37 (a) applies, and so do S-0214 claim 6 (attack cursor) and S-0309 claim
      25g.
    - Infantry firing never sets it.
    - Backing: `measurable` (CS16, part d)
    - Confidence: high

40. **Disguise detection.** A building whose type detects disguises adds the sensor disc of radius
    equal to its disguise-detection range to its owner's detector coverage.
    - The activation, power and capture rules of claim 23 apply.
    - Removal uses the detection range, with no floor (`quirks.sensor_count_negative`).
    - Detector coverage serves only claims 35 and 37. It does not reveal cloaked objects.
    - A mobile unit whose type detects disguises gives no coverage; it only changes its own target
      acquisition (S-0309 claim 12, table C).
    - Backing: `measurable` (CS15, part b)
    - Confidence: high

### Underwater vessels

41. **Submerging and surfacing.** A vessel is *submerged* while its type is underwater and its cloak
    state is cloaking, cloaked or uncloaking. Submerging and surfacing are therefore claims 5 to 11
    applied to a vessel, with no naval rule of their own.
    - A self-cloaking underwater vessel enters play already submerged (claim 16).
    - It needs (*N* − 1) · *S* ticks of surfacing before a weapon with the fire-only-visible flag can
      fire.
    - Its detection is that of any cloaked object: sensor coverage (claims 21 to 23) and the
      adjacency reveal (claim 13).
    - An underwater type that cannot cloak is never submerged (S-0504 claim 6).
    - Backing: `measurable` (CS1, part e)
    - Confidence: high

## Formulas

F1, cloaking time (integers). From the tick cloaking starts:

> *T*_cloak = *c* × *S*, with *c* = ⌈*N* / 2⌉. Units: ticks.

These duration formulas apply for positive integer *S* and *N* ≥ 2. Degenerate values are
listed under Open questions, except the zero-speed case below.

The original computes each stage's fade fraction in floating point, with a 53-bit significand
and rounding toward zero after each operation (S-0205 claim 5), as
the stage divided by *N*, times 256, truncated toward zero. For integers 0 ≤ *k* < *N*, this
always equals ⌊256 *k* / *N*⌋, so *c* is the first *k* with ⌊256 *k* / *N*⌋ ≥ 128, which is
⌈*N* / 2⌉.

F2, uncloaking time (integers):

> *T*_uncloak = (*N* − 1) × *S*. Units: ticks.

F3, cloak delay. *d* is the game-wide cloak delay in minutes. It is read as a decimal per S-0003
claims 6 and 7: single precision, widened, and scaled when written with a percent sign.

> *D* = trunc(*d* × 900.0). The multiplication rounds toward zero to a 53-bit significand,
> then the result is truncated toward zero to integer ticks (S-0205 claim 5).
> Units: ticks.

F4, health test (double precision, 53-bit significand). Let *h* be the integer health,
*h*ₘₐₓ the positive integer strength and ρ the red ratio, imported as S-0302 claim 12 says.
The division rounds toward zero to 53 significant bits (S-0205 claim 5); there is no integer
rounding before comparison.
- Claim 5 starts at once when *h*/*h*ₘₐₓ > ρ (strict).
- Claim 8 applies when *h*/*h*ₘₐₓ ≤ ρ.

F5, cloak field circle (integers). The sequence (*x*ᵢ, *y*ᵢ, *e*ᵢ) for *i* = 0, 1, 2, … is:

> *x*₀ = 0, *y*₀ = *r*, *e*₀ = 3 − 2*r*, and *x*ᵢ₊₁ = *x*ᵢ + 1.
>
> When *e*ᵢ < 0: *y*ᵢ₊₁ = *y*ᵢ and *e*ᵢ₊₁ = *e*ᵢ + 4*x*ᵢ + 6.
>
> When *e*ᵢ ≥ 0: *y*ᵢ₊₁ = *y*ᵢ − 1 and *e*ᵢ₊₁ = *e*ᵢ + 4(*x*ᵢ − *y*ᵢ) + 10.

The field of radius *r* is the union, over every *i* with *x*ᵢ ≤ *y*ᵢ, of four spans: the rows
*dy* = ±*x*ᵢ for |*dx*| ≤ *y*ᵢ and the rows *dy* = ±*y*ᵢ for |*dx*| ≤ *x*ᵢ.

F6, owner's disguise cycle (integers):

> *t* = (*now* − *t*_disguised) mod 256, with *now* and *t*_disguised in ticks.

**Table B, owner's view of its disguised unit.**

| *t* | Image | Translucency |
|---|---|---|
| 0 to 3 | real | 25 % |
| 4 to 7 | real | 50 % |
| 8 to 11 | disguise | 50 % |
| 12 to 15 | disguise | 25 % |
| 16 to 47 | disguise | none |
| 48 to 51 | disguise | 25 % |
| 52 to 55 | disguise | 50 % |
| 56 to 59 | real | 50 % |
| 60 to 63 | real | 25 % |
| 64 to 255 | real | none |

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Ticks per minute (cloak delay) | 900 | ticks | F3 |
| Self-cloak chance below red | 4 | per hundred | Claim 5, draw R(0, 99) < 4 |
| Low-health abort chance | 10 | per hundred | Claim 8, draw R(0, 99) < 10 |
| Fade-fraction scale | 256 | fraction units | Bands at 64, 128, 192 |
| Building fade levels | 0 to 15 | levels | One level per tick, claim 18 |
| Neighbouring-infantry cadence | 8 | ticks | Checked when tick mod 8 ≠ 0 |
| Owner's disguise cycle | 256 | ticks | Table B |
| Cloak radius absent-key default | 20 | cells | Signed 8-bit |
| Cloaking speed absent-key default | 7 | ticks per stage | |
| Cloaking stages absent-key default | 9 | stages | |
| Generator restart reach | 2 (*r* + 2) | cells | Claim 20 |

## State and ordering

**Vehicles, infantry and aircraft.**

| State | Event | Next state | Effect |
|---|---|---|---|
| uncloaked | claim 5 or 6 holds | cloaking | Stage 0, sound, target drop (claim 14), deselect for non-owners |
| cloaking | stage reaches ⌈*N*/2⌉ | cloaked | Stage 0, target drop (claim 14) |
| cloaking | claim 8 draw < 10 | uncloaking | Stage *N* − 1, silent |
| cloaking | uncloak cause (claim 11) | uncloaking | Stage *N* − 1 |
| cloaked | claim 9 fails, or cause (claim 11) | uncloaking | Stage *N* − 1 |
| uncloaking | stage reaches 0 | uncloaked | Cloak delay *D* starts |
| uncloaking | anything that would cloak | uncloaking | None (claim 10) |

**Buildings.**

| State | Event | Next state | Effect |
|---|---|---|---|
| uncloaked | ready (claim 18) | cloaking | Level rises 1 per tick, sound |
| cloaking | level 15 | cloaked | Particle effects stop |
| cloaked | should not be cloaked | uncloaking | Level falls 1 per tick |
| cloaking | loses eligibility | cloaking | Completes fade, then uncloaks if still ineligible |
| uncloaking | ready inside own field, cell re-evaluated | cloaking | Current fade level preserved |
| uncloaking | level 0 | uncloaked | Particle effects restart |

**Ordering.**
- The draws of claims 5, 8 and 33 are taken in the object's own turn (S-0203), at most one per
  object per tick for each claim.
- A unit on a newly covered cell can start cloaking in the generator's tick before its own
  update. An attacker processed later that tick can therefore lose this target even when the
  target has not yet had its own update.

## Edge cases and quirks

- **`quirks.cloak_flag_ground_mobile_only`** (default on). The type's cloak flag is ignored for
  aircraft and buildings. Fixed behaviour: aircraft and buildings with the flag can cloak by
  themselves.
- **`quirks.uncloak_from_cloaking_full_duration`** (default on). An uncloak that starts while the
  object is still cloaking takes the full (*N* − 1) · *S* ticks. Fixed behaviour: it starts from the
  current stage.
- **`quirks.cloak_zero_speed_stall`** (default on). With a cloaking speed of 0:
  - cloaking advances one stage per tick
  - uncloaking never advances, so the object stays at the first uncloaking stage forever, drawn as
    warped, and a weapon that needs it visible never fires.

  Fixed behaviour: uncloaking also advances one stage per tick.
- **`quirks.sensor_building_activation_once`** (default on). Sensor arrays and disguise detectors:
  - their coverage is set at first placement only, if powered
  - power changes do nothing afterwards
  - capture leaves coverage with the old owner.

  Fixed behaviour: coverage follows *in operation* and moves to the new owner on capture.
- **`quirks.sensor_array_removal_radius`** (default on). A sensor array's removal decrements over
  its cloak-radius disc instead of its sensors-sight disc. Fixed behaviour: it decrements over the
  same disc it added.
- **`quirks.sensor_count_negative`** (default on). Building removals have no floor, so a cell's
  count can go negative and need extra coverage later. Fixed behaviour: removal undoes exactly what
  was added.
- **`quirks.cloak_field_overlap_loss`** (default on). Claim 20, overlaps. Fixed behaviour: a cell
  stays in the field while any powered generator of the house covers it.
- **`quirks.building_sensor_ring_owner_only`** (default on). In claim 18, sensor units of allied
  houses stop a building cloaking, and a sensors veterancy ability does not. Fixed behaviour: only
  non-allied sensor units count, through the type flag or the veterancy ability.
- **`quirks.heal_reveals_cloaked`** (default on). Healing or a zero-damage hit uncloaks (claim 12).
  Fixed behaviour: only hits that lower health uncloak.
- **`quirks.mirage_adjacent_check_cadence`** (default on). The neighbouring-infantry check runs on
  7 ticks of 8. Fixed behaviour: it runs only on ticks that are multiples of 8.
- **Empty default-disguise list.** If the game-wide default-disguise list is empty, the draw of
  claim 33 (c) reads past the list in the original. Our engine treats an empty list as "no
  disguise is taken" and draws nothing, with an importer warning; this is a data error, not a switch.
- **Disguised infantry with no disguise type.** In the original, when a side has no default
  disguise, the infantry's image falls back on the *local player's* side default. Our engine uses
  the owner's side default, so what is drawn never depends on the viewer's side. This is a
  display-only bug, so it gets no switch.
- **Very large stage counts.** With *N* ≥ 256, the first uncloaking stage has *x* ≥ 255. Such an
  object is not drawn for non-owners at that stage, but it is not hidden in game logic.
- **Leaving play after a move.** A mobile sensor that leaves play removes its disc around the cell
  it is in at that moment. If that differs from the cell where its disc was last added, the counts
  drift (Open questions).

## Import mapping

| Key (section) | Concept | Parse form | Absent-key default |
|---|---|---|---|
| `Cloakable` (object type) | Own cloak flag, claim 2 (a) | boolean | no |
| `CloakStop` (object type) | Cloak-stop, claim 3 | boolean | no |
| `CloakingSpeed` (object type) | *S* | integer, ticks per stage | 7 |
| `CloakingStages` (General) | *N* | integer | 9 |
| `CloakDelay` (General) | *d*, F3 | decimal, minutes | 0.0 |
| `CloakSound` (AudioVisual) | Claim 15 sound | sound name | none |
| `CloakGenerator` (building type) | Cloak generator, claims 19–20 | boolean | no |
| `CloakRadiusInCells` (building type) | *r*, also claim 23 removal | integer, kept as signed 8-bit | 20 |
| `Sensors` (object type) | Sensors flag, claims 13 and 18 | boolean | no |
| `SensorsSight` (object type) | *R*, claims 22–23 | integer, cells | 0 |
| `SensorArray` (building type) | Claim 23 | boolean | no |
| `DetectDisguise` (object type) | Claim 40, S-0309 | boolean | no |
| `DetectDisguiseRange` (building type) | Claim 40 radius | integer, cells | 0 |
| `CanDisguise` (object type) | Claims 30–34 | boolean | no |
| `PermaDisguise` (object type) | Claim 32 | boolean | no |
| `DisguiseWhenStill` (object type) | Claim 33 | boolean | no |
| `MakesDisguise` (warhead) | Claim 31 | boolean | no |
| `DisguiseFakeBlinkTime` (weapon) | Claim 39 | integer, ticks | 0 |
| `DisguiseFireOnly` (weapon) | Read and ignored | boolean | no |
| `AlliedDisguise`, `SovietDisguise`, `ThirdDisguise` (General) | Claim 32 defaults | infantry type name | none |
| `DefaultMirageDisguises` (General) | Claim 33 (c) list | list of terrain-object type names | empty |
| `InfantryBlinkDisguiseTime` (General) | Claim 33 (b) lockout | integer, ticks | 0 |
| `ConditionRed` (AudioVisual) | ρ, F4 | decimal, S-0302 claim 12 | 0.5 (S-0302) |
| `AttackCursorOnDisguise` (General) | S-0214 claim 6 | boolean | no |
| `DisabledDisguiseDetectionPercent` (General) | Acquisition-only chance list, S-0309 claim 25 (g) | list of decimal integers, S-0309 Import mapping | empty at construction; missing key keeps held list |

## Measurement scenarios

All scenarios use the measurement setup, game-speed index 2, 800 × 600 pixels, director off,
mods and engine add-ons off, and a small purpose-made skirmish map. One bystander computer player
is kept outside the observed area under the glossary's void rule. Named fixture **CS-fixture**
means that map's temporary override sections: only the abilities, weapons, radii, timing fields
or health conditions explicitly required by a sub-test are changed, with other data stock.
Cloak generators, self-cloaking infantry and EMP attacks require this fixture because stock
skirmish data does not provide those test combinations. Remove all overrides afterwards.

Use tick-stamped screen and sound capture, scripted input and a read-only recorder. Repeat each
deterministic sub-test three times. Record target lines, selection, radar pixels, images, sound
and health as named below. Every expected result is falsified by a differing result with the
same eligibility conditions. CS17 supplements CS1–CS16 for clauses not isolated below.

- **CS1, timings.** Setup: an infantry type that cloaks by itself, at full health, on open ground.
  - **(a)** Record the tick at which it starts cloaking and the tick at which it becomes fully
    cloaked. Expected: *c* · *S* ticks with the imported *S* and *N*. Falsified by another value.
  - **(b)** Watch, as the owner and as an enemy without sensors, how it is drawn through the bands
    of claim 28, and whether a vehicle's shadow shows while it fades.
  - **(c)** Order it to fire at a target in range. Expected: firing comes (*N* − 1) · *S* ticks
    after the uncloak starts. It then cannot start cloaking again until *D* ticks after the uncloak
    completes, which needs a test file with *d* > 0.
  - **(d)** Listen for the cloak sound at the start and at the uncloak.
  - **(e)** Repeat (c) with a stock submarine.
- **CS2, sources.**
  - **(a)** A stock submarine leaves the yard. Expected: fully cloaked on its first tick.
  - **(b)** A test file gives an aircraft type and a building type the cloak flag. Expected:
    neither cloaks.
  - **(c)** A cloak-stop vehicle: ordered to move while uncloaked, it never starts cloaking. Once
    fully cloaked and then moved, it stays cloaked.
  - **(d)** An aircraft inside an own cloak field cloaks only on the pad.
- **CS3, ready conditions.** A self-cloaker with an enemy inside its weapon range does not start
  cloaking. When the enemy leaves, it starts at once.
- **CS4, the red-health draw.** Use a self-cloaker at health below the red ratio (a test file
  lowers health), repeated over many seeds.
  - **(a)** Record the number of ticks until cloaking starts. Expected: geometric, with
    *p* = 0.04 per eligible tick.
  - **(b)** Repeat at full health. Expected: immediate.
  - **(c)** Count aborts in the 50 % band. Expected: about 10 % per eligible tick.

  The random-state recorder can confirm the number of draws per tick.
- **CS5, the cloak field.**
  - **(a)** A friendly tank parks inside a powered generator's field. Expected: it cloaks with no
    delay, even at low health.
  - **(b)** It drives out. Expected: it uncloaks on leaving.
- **CS6, EMP.** Hit a fully cloaked self-cloaker with an EMP weapon, outside any field and then
  inside one. Expected: it uncloaks outside and stays cloaked inside.
- **CS7, uncloak causes.**
  - **(a)** C4 plant.
  - **(b)** A vehicle crush.
  - **(c)** A blocked mover: an enemy tank's path is blocked by a cloaked vehicle. Record whether a
    third house sees it.
  - **(d)** A medic heals a cloaked ally.
  - **(e)** A weapon whose versus value is 0 hits the target.

  Expected: each uncloaks. For (c), every house sees the uncloak.
- **CS8, adjacency.**
  - A cloaked enemy infantry unit walks next to a stationary enemy sensor vehicle. Expected: it
    uncloaks.
  - The sensor drives next to a stationary cloaked unit. Expected: no uncloak; the sensor's house
    sees it 50 % translucent.
- **CS9, targeting.**
  - **(a)** Units attacking a target that starts cloaking drop it unless covered by a sensor.
  - **(b)** A unit whose house gains sensor coverage of the target between the start and the
    completion keeps it at completion.
  - **(c)** Retain an attack target while it becomes hidden, first an own object, then an allied
    object and an enemy. Use CS-fixture weapon pairs of opposite signs, including their ambient
    components, with positive, zero and negative mean weapon damage (S-0308 claim 15) in separate
    runs. Keep the attempted weapon and all other firing conditions unchanged while changing the
    other slot enough to change the mean's sign. Test both a selected healing weapon with positive
    mean and a selected damaging weapon with nonpositive mean. Record target retention and shot
    creation, repeating with own sensor coverage of the target. Expected: without sensing,
    positive mean refuses an own or allied hidden target even for a selected healing weapon.
    Nonpositive mean passes this cloak test even for a selected damaging weapon. Hidden enemies
    are refused for every mean. Sensing removes this refusal. Use a damaged ally and otherwise
    legal healing conditions for the healing control. Any contrary shot/no-shot result with the
    target retained and all other firing conditions met falsifies the mean test; a lost target
    makes that trial inconclusive.
  - **(d)** Aircraft drop a hidden target.
  - **(e)** Click tests, as owner, ally and enemy.
  - **(f)** Radar, as ally and enemy, with and without own sensors.
- **CS10, buildings.**
  - Count ticks from power-on until a building in a generator's field is fully cloaked. Expected:
    15 ticks after the field reaches it.
  - Place an allied sensor unit next to the building. Expected: the building uncloaks.
- **CS11, field shape and rate.**
  - Record which cells' units cloak for a test file radius of 4, and for the stock radius.
    Expected: F5.
  - Cut power. Expected: rings leave one per tick.
  - Overlap two generators and sell one. Expected: units in the overlap uncloak briefly.
- **CS12, sensors.**
  - **(a)** Map the disc of a mobile sensor with sensors-sight *R* by walking a cloaked enemy
    unit along each axis. Expected: |*dx*| ≤ *R* − 1 and |*dy*| ≤ *R* − 1, with the strict squared-distance test, symmetric on both axes.
  - **(b)** An allied sensor gives no coverage.
  - **(c)** A sensor array placed unpowered never senses.
  - **(d)** Selling an array leaves cells between its sensors-sight and the cloak radius unsensed
    for a second array placed there afterwards, until a third sensor covers them.
  - **(e)** Capture: coverage stays with the old owner.
- **CS13, disguise acquisition.**
  - **(a)** A spy disguises from across the map.
  - **(b)** A new spy shows its side default to an enemy, who still auto-targets it.
  - **(c)** Damage does not undo a permanent disguise. A test file clears the permanent flag, and
    then a hit clears it. An infantry disguiser that is not permanent keeps its disguise while
    moving.
- **CS14, disguise when still.**
  - **(a)** A still disguise-when-still vehicle takes a terrain disguise. The random-state recorder confirms one draw of
    range *n*.
  - **(b)** Enemy infantry stepping next to it removes the disguise on the next checked update,
    with checks on ticks not divisible by 8. It reappears after the infantry-blink time once
    the infantry has left.
  - **(c)** A hit delays re-disguise by the lockout length of S-0302 claim 12.
- **CS15, appears disguised.**
  - **(a)** A spy disguised as an infantry unit of house *E* is not auto-targeted by *E*'s
    vehicles. *E*'s allies are not fooled when the spy's owner is their ally.
  - **(b)** A powered detector building of the enemy makes its units acquire it within the range.
  - **(c)** With the auto-crush rule enabled in CS-fixture, put a crushable enemy infantry
    disguiser within crush range of an idle crusher, disguised as the crusher's house and
    without detector coverage. Record automatic movement and the infantry's survival. Expected:
    no automatic crush choice. Repeat with building detector coverage: it is chosen and crushed.
    Separately order the crusher along a route through the still-disguised infantry's cell to a
    cell beyond it. Confirm passage through the crush radius with the disguise still effective.
    Expected: the ordered drive-through crushes it. Automatic choice while fooled, or survival
    solely due to disguise during a valid drive-through, falsifies the distinction (S-0210
    claims 2–3 and 11). A missed route or lost disguise makes that trial inconclusive.
- **CS16, drawn disguise.**
  - **(a)** The owner sees the disguise image on ticks 8 to 55 of every 256.
  - **(b)** An ally sees the disguise.
  - **(c)** Use three houses with distinct radar colours: viewer *V*, unit owner *O* and disguise
    house *D*. Have an infantry disguiser of *O* copy infantry of *D* before observing its stable
    radar pixel from *V*, on explored ground with radar available, no overlapping radar object,
    no cloak and no damage flash. Use CS-fixture map-house alliances to set *O* allying *V* but
    not the reverse; then reverse only those two relations in a separate run. Record both
    alliance directions, the retained disguise house and radar pixels. Expected: owner's colour
    in the first run, disguise-house colour in the second. Repeat with mutual alliance and
    mutual hostility as controls: owner's and disguise-house colours respectively. The opposite
    colour in any otherwise valid run falsifies claim 38. A missing radar object, lost disguise,
    changed disguise house or interference makes the run inconclusive.
  - **(d)** After the vehicle fires, enemies see the real unit for the weapon's fake-blink time.

### CS17: clause-isolation checks

Use CS-fixture and the same recording setup and three-run count above. Each row supplements
the scenarios cited by the named claims. Vary one condition at a time. A contrary transition,
image, sound, targeting result or draw count falsifies the stated expectation.

| Claims | Procedure and observable | Expected contrast |
|---|---|---|
| 1, 7, 10 | Repeat CS1 with odd/even stage counts and several positive speeds. Hit during the first fading band; restore a field during uncloaking. Record bands and transition ticks. | F1/F2 hold. Interrupted cloaking still gives the full uncloak duration. Mobile objects finish uncloaking and wait out the delay rather than reversing. |
| 2, 3, 16 | Compare placed/factory-created vehicles and infantry. Populate only the veteran, then only the elite cloak-ability list. Give cloak-stop to a crate-cloaked vehicle and move it while visible, then while cloaked. | Vehicles with the own flag begin cloaked, infantry do not. Elite accepts either list. Crates do not bypass cloak-stop on starting; movement does not break a full cloak. |
| 4, 5 | Separately retain a rearm timer, an in-range target, a magnetic grip, positive height and war-factory contact on a self-cloaker. Remove each blocker. Compare out-of-range targets. | Each listed blocker prevents a start; full health and no blockers permit it. |
| 5, 8 | Collect 1000 eligible updates at, below and above the red ratio, and 1000 low-health updates in the abort band across restarts. Read the stream state and match ranged draws. | At/below red starts use one draw below 4, above red none. Aborts require one draw below 10 in the specified band only. |
| 5, 6, 9, 17 | Compare own-flag and ability cloakers inside/outside a field under EMP, paralysis and warp. Give aircraft a crate or ability; compare landed and airborne starts, then take off after cloaking. | The disable gate bars own-flag starts outside a field, not ability/field starts when otherwise ready. Losing the last source breaks a full cloak. Airborne starts fail; takeoff alone does not break a full cloak. |
| 11, 12, 15 | Repeat CS7 for each accepted and rejected hit category in claim 12. Record sounds and fade. Repeat a hit while already uncloaking. | Accepted hits and listed uncloak causes sound; rejected hits do not reveal. Repeated causes during uncloaking do not replay sound. |
| 13 | Share a neighbouring cell between sensor and non-sensor infantry. Swap which is nearest the top corner. Repeat with an ability sensor and on a bridge. | Only the nearest ground-layer candidate counts. A sensor approaching a stationary target gives coverage, not adjacency uncloaking. |
| 14, 24–26 | Retain own-house, sensed-enemy and unsensed-enemy targeters at the mobile start/completion. Repeat on a building. Select a sensed non-owned object before it cloaks. Try positive-damage force fire on a hidden own target and healing on a hidden ally. | Unsensed enemies drop at both mobile moments, buildings only at the start. Non-owner selection is lost at cloak start even with sensors. Positive mean weapon damage is refused; nonpositive mean passes the cloak test for an own or allied target, independently of the selected weapon's sign. |
| 18 | Put sensor infantry in the foundation/border, with candidate-control occupants. Compare own/allied/enemy and veterancy-only sensors. Lose the field during cloaking, then restore it during uncloaking and change sensor coverage. Watch levels and particles. | Foreign type-flag sensors, allies included, block. Interrupted cloaking completes before uncloaking. Field re-evaluation can reverse uncloaking at its current level. Particles stop at 15 and restart at 0. |
| 19, 20 | Witness the circle boundary. Power, disable, sell, destroy and capture a generator separately. Test allied witnesses. With two generators cut one's power, then separately sell it. Repeat with a nearby other-owner generator. | F5, open-ground ring rates and immediate removal hold. Allies are not covered. Overlaps can lose coverage; a full idle neighbouring generator restarts irrespective of owner. |
| 21–23, 40 | Observe detection across a mobile sensor's movement, capture and removal. Compare a type flag alone with a range alone. Toggle array/detector power after placement, and remove an initially unpowered building. Compare mobile and building disguise detectors. | Coverage is owner-only and does not lift shroud. Mobile range supplies coverage. Building power changes do not toggle coverage, and capture leaves it with the old owner. Mobile disguise detectors supply no shared coverage. |
| 23, 40 | Make the array's cloak radius larger than its sensor sight. After removal, cover an affected outer cell with one sensor, then two. For detectors test the detection-radius disc after an initially unpowered building is removed. | A formerly −1 cell needs two additions. Array removal uses cloak radius; detector removal uses detection radius. |
| 27–29 | Compare owner, mutual-ally and enemy views on explored ground with/without own sensors. Record clicks, shadow, radar and images every tick. Include a stage count reaching fraction 255. | Claim 28 bands hold. Unsensed allies see the cloak image but cannot click or list it on radar. Fading vehicles have no shadow. Fraction 255 is not drawn for non-owners. |
| 30–32 | Copy undisguised/disguised infantry, disguised/undisguised vehicles, rock overlays and terrain objects. Create permanent disguisers with each side default. Remove the disguise house. Observe image, colour, health and acquisition. | Claim 31 copies and rejected kinds hold, without target damage. Defaults have the owner's house and do not fool enemy logic. Cleared permanent infantry resets to default; a vehicle clears outright. |
| 33, 34 | Move temporary infantry/vehicle disguisers; introduce neighbours on all 8 tick phases, ground/bridge layers, contact and lockout. Compare damage on permanent/temporary types. Try transport entry and capture. For undeployment, use S-0517's P8 (b) fixture and record disguise at the first undeploy processing and at completion, with and without a deploying presentation. | Only temporary vehicles lose disguise while moving. Adjacent checks skip ticks divisible by 8. An eligible still vehicle draws once. Damage follows S-0302 claim 12. The listed non-removal events preserve disguise. Undeployment clears disguise at the first step, before animated completion, or at the immediate step without a presentation; a disguise lost before the step makes that trial inconclusive. |
| 35, 36 | Vary disguise house among observer, ally and enemy. Vary ownership and detector coverage. Observe acquisition, retaliation, automatic crush choice versus ordered drive-through, cell entry, cursor, hover bar/name, insignia and scenery selection/flash. | Outcomes using claim 35 change with its predicate as claim 36 and the owning specs state. Disguise blocks automatic crush choice, not an otherwise valid drive-through. Detection changes the observer's treatment, not the copied disguise. |
| 37–39 | Capture the owner's full 256-tick cycle and each Table B band. Compare ally/enemy images and radar colours. Fire vehicles using two fake-blink times, also while undisguised; compare infantry. | Owner bands/cycle hold. Non-owners see real during vehicle fake blink, then the same disguise again. Infantry firing starts no fake blink. Radar uses owner colour when the owner allies the viewer, otherwise a non-empty disguise house's colour; repeat opposite one-way alliances as CS16 (c). |
| 41 | Compare stock submarines with an underwater fixture vehicle stripped of all cloak sources. Fire after surfacing; add sensors and then cause adjacency reveal. | The control stays surfaced. Ordinary transitions give submerged states and F2 surfacing delay. Sensing alone does not surface it; adjacency can. |

Zero-speed check (claim 7's quirk): record cloaking and subsequent firing over 256 ticks with
speed 0 in CS-fixture. Expected: cloaking advances one stage per tick but uncloaking does not
advance. A later shot from a weapon requiring visibility falsifies the stall.

## Open questions

- Degenerate stage counts (*N* ≤ 1), negative stage speeds, and arithmetic-overflowing radii or
  counts are not covered by the duration and disc formulas.
- A generator ring interrupted by an own building's centre: which cells miss coverage and when,
  if ever, are they filled? A complete-ring stall rule is not established.
- Does a passenger keep its cloak state inside a transport? This was not traced.
- At which tick does a mobile sensor's disc move: when the sensor enters a cell or when it finishes
  moving through it? Can coverage drift when a sensor leaves play between those moments?
- Which object counts as "nearest the top corner" in the one-candidate tests of claims 13 and 18
  when several share a cell?
- Is the crushing uncloak (claim 11 (c)) reached on every crush?
- Buildings and aircraft treat the disguise test (claim 35) as "disguised for every house", but no
  stock data disguises them. Should our engine apply claim 35 to them unchanged?
- What does a negative cloak radius do?
- What radar colour does a disguised unit whose disguise house is none show to other houses?
