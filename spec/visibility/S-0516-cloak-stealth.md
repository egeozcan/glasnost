---
id: S-0516
title: Cloak and stealth (cloaking, cloak fields, sensors, disguise)
topic: visibility
revision: 20
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-13
measurements: []
requests: [Q-0720, Q-0721, Q-0722, Q-0723, Q-0724, Q-0725, Q-0726, Q-0727, Q-0728, Q-0729, Q-0791, Q-0792, Q-0795, Q-0796, Q-3060]
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
- **Cell-arrival update.** The movement event that relocates mobile sensor coverage and
  tests adjacency reveal. It can also result from a superweapon warp or fall landing.
  Placement alone is distinct. Its timing depends on the movement or relocation (claim 22). The earlier name
  *completed-cell movement event* means this event, not every cell-boundary crossing.
- **Idle generator.** A cloak generator with neither growth nor shrinking pending. This is
  independent of its radius and of being in operation (claim 20).
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
- **Natural smoke system.** The particle system that S-0616 claim 1 gives a building whose type names a
  natural particle system. Its *natural-smoke displacement* is the type's displacement from the building's
  position at which that system is placed (S-0616 claim 1).
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
   - it is seized (S-0607 claims 5, 12 and 20)
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

7. **Cloaking and uncloaking durations.** For positive *S* and *N* ≥ 2, every *S* ticks of
   a fading object's turns, the stage
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
   Claim 43 and the zero-speed quirk define inputs outside this formula's domain.
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
    - **(c) Crushing.** A vehicle actually crushes at least one object, including infantry or a
      vehicle (S-0210 claims 2–4). Crushing only an overlay or wall does not uncloak it. Neither
      crossing an empty cell nor a vehicle-thief takeover alone counts (S-0210 claims 5 and 8).
    - **(d) Hits.** It is hit (claim 12).
    - **(e) Blocked movers.** A moving unit finds it as a cloaked blocker. Every cloaked object in
      the ground layer of the blocking cell, of any house, starts uncloaking (claim 12). This is
      a real uncloak, seen by every house, not a sight for the mover's house alone.
    - **(f) Adjacent sensors.** It moves next to an enemy sensor unit (claim 13).

    These causes play the game-wide cloak sound when they actually start an uncloak.
    - Backing: `measurable` (CS7)
    - Confidence: high

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

13. **Adjacent sensor reveal.** When a fully cloaked vehicle, infantry unit or aircraft
    has a cell-arrival update (claim 22), the game checks each of the 8 neighbouring
    cells that pass the usable-area test. This is the same event that relocates its sensor
    disc, if any; an object need not itself be a sensor to receive this reveal test.
    All movement and relocation rows in claim 22 use this test, including same-cell
    updates. Ordinary aircraft flight, jumpjet flight above touchdown and rocket flight
    supply no such event, so flying past a sensor does not cause this adjacency uncloak.
    A jumpjet's touchdown, a surviving superweapon warp's arrival effects and a fall landing
    do supply it. Entering play alone, including unloading, does not.
    - For each such cell it takes a single candidate from infantry, vehicles, aircraft and
      buildings in the ground layer. For position (*x*, *y*, *z*) in leptons, let *u* = *x* mod 256
      and *w* = *y* mod 256, residues from 0 to 255. Its distance is the integer
      ⌊*r*(*u*² + *w*²)⌋, where *r* is S-0204 claim 5's table square root, subject to
      `quirks.table_root_distance` (default on: the table root; off: the exact integer square
      root, as defined under Edge cases and quirks). The squares and sum are formed in double precision, exactly
      in this range, and the root is truncated toward zero. Height is ignored. The least
      integer distance wins, with ties going to the first object in the cell-layer order
      (S-0209 claim 13). This is the same distance as S-0207 claim 15, but its optional
      change to measuring from the centre does not change this sensor test.
    - A building uses its position within the cell containing that position, even when another
      foundation cell supplied the candidate. Thus equal truncated distances tie even when
      the exact squared distances differ. At the infantry standing positions, preference is
      top, centre, left/right tied, bottom (S-0208 table P).
    - The moving object starts uncloaking at once if that candidate:
      - is not allied with the mover's owner, and
      - has its type's sensors flag or a sensors veterancy ability.
    - Only the candidate is tested. A sensor standing in the cell behind another object does not
      count.
    - Only a cloaked object that moves is revealed this way. A sensor that walks up to a stationary
      cloaked object reveals nothing by adjacency; its coverage (claim 22) only lets its house see
      and attack the object.
    - Backing: `measurable` (CS8, CS17 candidate checks, CS18 parts C and J, CS20) for event
      timing, standing-position preference and first-listed integer-distance ties, except the
      fixture-limited movement cases marked `unmeasured` in claim 22. The distinction
      between table-root and exact-root candidate distances is `unmeasured`: no legal
      sub-cell placement pair distinguishing the winning candidate has been established.
      CS18 part C's non-standing-position extension is conditional on finding such placements,
      not established coverage of that distinction.
    - Confidence: high for event timing; medium for the candidate test

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

16. **Placement and passengers.** Ordinary placement within the usable area does not force
    a vehicle fully cloaked, even when its own cloak flag is set. Newly created vehicles start
    uncloaked there, as infantry and aircraft do, and use claim 5. Only a vehicle placed
    outside the usable area is forced fully cloaked when its own cloak flag is set. The
    placement's usable-area result is tested afresh, not a remembered first-placement flag.
    An ordinary transport exit within the usable area therefore does not force a cloak.
    - Boarding and unloading preserve the passenger's cloak state and stage. An uncloaked
      self-cloaking vehicle therefore unloads uncloaked, not automatically fully cloaked.
      Cloaked infantry also preserve their state. Subsequent eligibility and uncloak causes
      still apply.
    - The cloak delay is not reset or paused by boarding: elapsed game ticks consume it during
      the ride. A stored passenger takes no ordinary cloak updates. Its fading stage does not
      catch up by several steps on unloading: an expired stage timer permits at most one step
      on its next ordinary update (claim 42).
    - Transport eligibility and exit placement are S-0507 claims 1–12, not S-0506.
    - Backing: `measurable` (CS2 part a, CS18 part G)
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
    - **Explicit uncloak cause.** A hit or other applicable cause of claim 11 immediately
      changes cloaking to uncloaking, preserving the current fade level. There is no jump to
      level 15 and no wait for completion. It plays the cloak sound once. For example, a hit
      at level 7 starts falling from 7 on subsequent fade updates. This differs from merely
      losing eligibility. A field re-evaluation can subsequently reverse it as above.
    - **Delay.** Reaching level 0 does not start the mobile object's cloak delay.
    - **State changes.** At level 15 the building is fully cloaked; its natural smoke system, if
      any, is removed at once together with its particles. At level 0 it is uncloaked; a new
      natural smoke system is made (S-0616 claim 1) only when its type names a natural particle
      system, the building has none, and its type's natural-smoke displacement is not (0, 0, 0).
      Otherwise none is made.
    - **Sensor ring.** A building is not ready to cloak, and a cloaked building starts uncloaking,
      while one of the following stands in its foundation or in the 1-cell ring around it:
      a unit whose type has the sensors flag and whose owner is not the building's owner.

      Allies' sensor units count, and a sensors veterancy ability does not
      (`quirks.building_sensor_ring_owner_only`). Each cell uses the nearest candidate of claim 13,
      except that when the candidate is the building itself, the first infantry occupant is
      tested instead.
    - Backing: `measurable` (CS10, CS17 building checks, CS18 part F), except the
      table-root versus exact-root candidate distinction is `unmeasured` for the same
      placement limitation as claim 13.
    - Confidence: high (fade timing and explicit uncloak); medium (the sensor ring)

### Cloak fields

19. **Shape of a cloak field.** A cloak generator of positive radius *r* covers a filled circle of cells
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
    - **Growth.** A newly created generator is idle at radius 0, with no covered cells.
      For positive type radius *r*, becoming in operation makes growth pending as below. It
      adds radius 1 on its first growth update, then radius 2, and so on through *r*, one per
      tick. Radius 0 is an empty field, not the radius-zero circle of F5. A zero or negative
      type radius gives no field and cannot initiate a neighbour restart by shrinking.
      A zero-radius generator can still receive a restart when idle, as specified below.
      Negative radii are rejected
      as invalid with an importer warning and treated as zero: the original instead lets its
      growth radius wrap through invalid sizes. This invalid-data failure is not reproduced.
      Every object on a newly covered cell is re-evaluated (claim 6).
    - **Pending changes.** Operation is S-0405 claim 8's building state, separate from
      field radius and field phase. The ordinary building turn detects changes in operation
      before its field update, so a newly pending direction can take its first step that turn.
      Merely remaining in operation does not repeatedly request growth.

      | Field state | Event | Next field state and radius effect |
      |---|---|---|
      | Idle or shrinking, radius unequal to type radius | A change to being in operation is detected | Growing from the current radius; no wait for the shrink to reach 0 |
      | Shrinking, radius equal to type radius | A change to being in operation is detected | Still shrinking; no growth request |
      | Growing | A change to being in operation is detected | Still growing |
      | Idle or growing, positive current radius | A change to being out of operation is detected | Shrinking from the current radius |
      | Any phase, current radius 0 | A change to being out of operation is detected | Phase unchanged: idle stays idle, growing stays growing, shrinking stays shrinking |
      | Idle, below full radius | Ordinary update with operation unchanged | Still idle, even in operation; no automatic recovery |
      | Growing, current radius equals type radius | Field update | Idle, no radius change |
      | Shrinking, current radius 0 | Field update | Idle, no radius change and no neighbour-restart test |

      The radius-0 growing case can arise by a restart followed by loss of operation before
      its next field update. That update still grows: pending growth has no further power
      gate. Similarly, restoring operation at full radius before the first shrink step does
      not reverse shrinking; a later unchanged-operation turn does not request growth even
      after the collapse. A fresh unpowered placement instead remains idle at radius 0.
      An idle below-full field requires a new operation transition or a neighbour restart
      to grow; changing the type radius alone is not a growth request.
    - **Shrinking.** A pending shrink reduces the radius by one per tick to 0. Cells outside
      the shrinking circle leave the field at once, and their objects are re-evaluated
      (claim 9 (c)). Growth becomes idle only on the update after reaching the type radius, and
      shrinking becomes idle only on the update after reaching 0. The building-centre
      interruptions below do not change these phase-ending tests.
    - **Manual power-off.** An accepted power-off order has an additional immediate effect:
      shrinking becomes pending even at radius 0, and a current radius of 0 becomes the
      type radius before the shrink. It does not add coverage. This differs from ordinary
      low power or another operation change (`quirks.cloak_generator_zero_radius_shutdown`).
      S-0405 claim 10 owns which orders are accepted.
    - **Removal.** When the generator is removed, sold or destroyed, the field vanishes at once.
    - **Capture.** When it is captured, the field also vanishes at once, and it grows again for the
      new owner if the building is in operation.
    - **Ownership.** A field covers its owner's objects only, not its allies'.
    - **Overlaps.** Coverage is a yes-or-no flag per house and cell, not a count. When one of two
      overlapping generators of the same house shrinks, the shared cells leave the field even
      though the other generator still covers them. When an uninterrupted collapse reaches
      radius 0, every other in-play generator that is in operation and idle within the strict
      integer squared-distance test Δ*X*² + Δ*Y*² < 4(*r* + 2)² is restarted. *X*, *Y* are the cell coordinates obtained
      from each building's centre, and *r* is the shrinking generator's type radius. No lepton
      distance, height, foundation-edge distance or square-root approximation is used. At its
      full radius, the neighbour restarts from 0. Otherwise its current radius decreases by
      one, floored at 0, and growth resumes. This restart is not restricted to the same owner.
      Idle means no growth or shrinking is pending, not merely powered or stationary:

      | Neighbour's field state at the collapse | Restart eligibility, assuming the other tests pass |
      |---|---|
      | Idle at full radius | Yes; radius becomes 0 and growth becomes pending |
      | Idle below full radius, including radius 0 | Yes; radius decreases by one with floor 0 and growth becomes pending |
      | Growing, including a newly restarted radius-0 field | No |
      | Growing at full radius, before its next update ends growth | No; reaching the radius alone has not made it idle |
      | Shrinking, even if the building is in operation again | No |
      | Not in operation | No, whatever the radius or field state |

      Thus an ordinary growing neighbour does not take the below-full-radius restart branch.
      A second collapse cannot restart a neighbour already made growing by the first.
      Until recovery, objects in the gap may uncloak (`quirks.cloak_field_overlap_loss`).
    - **Buildings within a ring.** With `quirks.cloak_field_building_interrupt` on, a growing
      field's newly covered cells take effect in increasing *Y*, then increasing *X*. The
      first newly covered centre cell of an own building is covered, then the rest of that
      update's circle is omitted. The generator's own centre is not exempt, including on
      the first radius-1 update. Alone on previously uncovered open ground, that first
      update covers only the relative cells (0, −1), (−1, 0) and (0, 0), where coordinates
      are integer cells relative to its centre. The other two radius-1 cells remain
      uncovered. Its radius has already increased. The next growth update
      examines the whole next larger circle, so omitted cells can be covered then. Already
      covered building centres do not interrupt again. If this was the final radius, there
      is no next larger circle: omitted cells stay uncovered until another growth cycle or
      generator covers them. A shrink update similarly stops after clearing an own building's
      centre, unless it is an immediate collapse. An interruption on the update reaching 0
      also prevents that update's neighbour restart. The generator's own centre can cause
      this stop. At radius 0 the following shrink update stops without retrying the omitted
      cells or the restart.
    - **Immediate collapse.** Sale, destruction, removal and capture clear the field without
      building-centre interruptions and perform the same neighbour-restart test, even when
      both current and type radius are 0. For type radius 0 its strict integer-cell reach is
      Δ*X*² + Δ*Y*² < 16 square cells. There need not have been any covered cell to clear.
      This differs from ordinary shrinking starting at radius 0, which performs no test.
      On capture, this happens for the old field before any new-owner growth.
    - Backing: `measurable` (CS11, CS17, CS18 parts D and I, CS19, CS21), except the restart of
      an idle below-full positive-radius neighbour is `unmeasured`: no ordinary setup for
      that idle state is established. CS19 gives the conditional test. Unchanged-operation
      behaviour after a runtime type-radius change is also `unmeasured`: no sanctioned
      skirmish fixture for that change is established (CS21). These rules are settled.
      Only construction of the distinguishing states remains unavailable.
    - Confidence: high

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
    *R* supplies its owner's sensor disc of radius *R*. Entering play centres it on the object's
    cell; subsequent relocation follows the events below, not continuous position. The sensors
    flag alone gives no coverage; it serves claims 13 and 18.
    - **Entering play.** The disc is added when the object enters play, including successful
      unloading, a carrier putting down its cargo, and initial parachute release. This is
      not itself a cell-arrival update and supplies no adjacency reveal. It does not wait
      for the object's next locomotor update. Boarding has already removed its disc, subject
      to the removal-centre quirk below; any leaked coverage is not repaired by unloading.
    - **Movement.** At a cell-arrival update, the old disc is removed around the last recorded
      coverage cell and a disc is added around the object's cell at that update. That cell
      becomes the recorded coverage cell. This happens even if it equals the old cell.
      Boundary crossing alone is not the event, nor is a later cloak update a substitute.

      | Locomotor | Cell-arrival update for sensor coverage and claim 13 |
      |---|---|
      | Driving or ship tracks | Track arrival at the cell centre (S-0207 claims 7 and 12, S-0504 claim 1), not the earlier boundary crossing; also a driving curve's chain-point arrival update (S-0207 claim 22) |
      | Walking infantry | Arrival and snap to the chosen standing position (S-0208 claims 6 and 23), including intermediate walking targets and same-cell arrivals; not reservation, boundary crossing or a physical step alone |
      | Hovering, including amphibious hovering | The non-refusing heading-point choices and no-point restoration cases of S-0503 claim 15. Coverage uses the actual position at that event, before any cached final travel, not the newly chosen heading cell. A clear choice can produce it when movement starts. Arrival need not snap to a cell centre; crossing a boundary during later travel does not produce it |
      | Ordinary flying aircraft | None from taking off, flight across cells, destination arrival or landing. Coverage remains where entering play or another independent coverage-changing event left it (`quirks.flying_sensor_disc_stationary`) |
      | Teleporting locomotor, any mobile object kind | At the destination in the jump tick, after position, height and landing-layer placement, not reservation or phasing-in completion (S-0207 claims 20 and 22) |
      | A jumpjet | At completed touchdown (S-0502 claim 15), after landing placement and before the later sight refresh and crate pickup; none on takeoff, in-flight cell crossings, cruising arrival or hovering |
      | Rocket flight | None during launch, flight or impact (S-0506 claims 14–21); its entering-play disc stays behind until an independent coverage-changing event |
      | Underground movement | On completion of emergence, not underground cell crossings |
      | Mechanical walking | At completion and snap to its movement point, not intermediate boundary crossings |
      | Drop-pod descent | No cell-arrival update from descent or touchdown alone; landing leaves and re-enters play, so coverage follows those independent removal and placement rules |

      S-0501 claims 1–8 own ordinary flight; S-0502 claim 3 identifies the distinct jumpjet
      movement. The stationary-flight quirk below applies to the flight portions of all three
      flying modes, but does not suppress a jumpjet's real touchdown event. A sensors flag
      without nonzero sensors-sight gives no disc in any of them. Missile-like projectiles,
      including a superweapon's projectile, are not mobile sensor objects and have neither a
      disc nor claim 13's adjacency reveal; their flight belongs to S-0306 claims 1 and 14–15.
    - **Teleport arrival consequences.** The teleporting locomotor's vehicle arrival reporting
      and crushing precede these sensor effects. Entry interactions can remove an infantry
      mover before it receives them. For a surviving arrival, disc relocation precedes the
      adjacency test and the landing cell's entered-by actions. Jump-time crate pickup,
      destination removal and the landing animation follow (S-0207 claim 22). No cells
      between departure and destination become disc centres or receive an adjacency test.
    - **Superweapon teleport.** S-0513 claim 39's removal and appearance are not ordinary
      leaving-play and entering-play sensor changes: coverage stays at its old recorded cell
      through departure, landing retries and appearance. A surviving member gets a cell-arrival
      update at its later arrival-effects update, after arrival sound and the arrival-fate test
      (S-0513 claim 38), before the landing animation and arrival hold. Its disc then transfers
      to the destination, and adjacency reveal applies. It does not wait for its own locomotor
      or hold expiry. A member killed by the arrival-fate test gets no such arrival update.
    - **Drops and unloads.** A successful ground unload or carrier put-down adds the disc at
      placement without an adjacency test. Any later walking or driving arrival is a separate
      event. A parachute release likewise adds coverage immediately in its release cell, and
      descent does not move it continuously. Completed fall landing is a cell-arrival update,
      including a same-cell update, after height becomes 0 and falling ends, before the
      parachute is told to end (S-0522 claims 9, 12 and 13).
    - **Owner change.** The disc moves from the old owner to the new one.
    - **Leaving play.** The disc is removed around the object's current cell, not necessarily
      the last recorded coverage cell (`quirks.mobile_sensor_removal_centre`). If it crossed a
      boundary but did not finish the movement event, the old-only crescent remains covered
      and the new-only crescent is decremented despite not having received this sensor.
      This affects destruction and boarding alike.
    - **Floor.** Removal never takes a mobile sensor's count below 0.

    A mobile type's disguise-detection range has no effect (claim 40).
    - Backing: `measurable` (CS12 parts a and b, CS18 parts B and J, CS20). Underground,
      mechanical-walking and drop-pod rows, and carrier vehicle put-down, are `unmeasured`:
      no supported skirmish fixture for those otherwise unused movement modes or the
      vehicle-lifting carrier is established here. The rocket's no-adjacency-event rule is
      also `unmeasured` until a pre-cloaked rocket fixture is established; its stationary
      sensor disc is measurable without cloaking the rocket. CS20 gives conditional checks
      rather than claiming a completed observation.
    - Confidence: high

23. **Sensor-array buildings.** A building whose type is a sensor array adds the sensor disc of
    radius *R* (its sensors-sight) for its owner.
    - **Activation.** It does so only at its grand opening (S-0108 claim 11), and only if it is in
      operation (S-0405 claim 8) at that moment. Nothing later adds the disc.
      - An array that enters play under construction (a place order, a deploy under S-0415
        claim 6, or a restored replacement under S-0524 claim 15) has its grand opening at its
        handover (S-0406 claim 15), unless its owner changes first (below). The handover queues
        *guard* only after the opening effects, so *construction* is still its current behaviour
        at the opening: it is not in operation, whatever its power, and it never adds the disc.
      - An array that a trigger action creates in play has its grand opening at once (S-0108
        claim 11), which gives no order between that opening and the queueing of *construction*.
        This spec infers that *construction* is already queued, with no current behaviour, at the
        opening (medium, as an inferred order), so it fails in the same way.
      - An array that the map places at game set-up passes the behaviour condition at its grand
        opening (S-0405 claim 8). It adds the disc if every other condition of that test holds
        at that moment.
      - An array whose owner changes during its build-up has its grand opening inside the
        change (S-0309 claim 31), for its new owner, after the change's *guard* request. If
        *construction* was still queued, that request has replaced it, so the behaviour read is
        *guard*. If *construction* was current, the read is *guard* when the request's
        readiness-dependent immediate start (S-0215 claim 7) made *guard* current, which needs the
        array's ready mark at the request (S-0406 claim 17 gives the build-up turns that set it),
        and *construction* otherwise. With *guard* read, the array passes the behaviour condition
        and adds the disc for its new owner if every other condition holds; with *construction*
        read, it fails.
    - **Power.** Power changes never add or remove the coverage afterwards. An array that is not in
      operation at its grand opening never senses, even after power returns
      (`quirks.sensor_building_activation_once`).
    - **Removal.** When the array leaves play (sale, destruction, removal), the game decrements its
      *current* owner's counts:
      - over the disc of radius equal to the type's **cloak-radius** field, not its sensors-sight
        (`quirks.sensor_array_removal_radius`)
      - with no floor, so counts can fall below 0 (`quirks.sensor_count_negative`).

      A cell left at −1 needs two later sensors before it is sensed again. The decrement applies
      whether or not the array ever added its disc, so an array that entered play under
      construction lowers its owner's counts by 1 over the whole cloak-radius disc when it leaves
      play, so a cell of that disc whose count was 0 is left at −1.
    - **Capture.** When an array that has already had its grand opening is captured, the old owner
      keeps the coverage. The new owner gets none, and the array's later removal decrements the
      new owner's counts. An array whose owner changes before its grand opening follows the
      build-up bullet above instead.
    - Backing: `measurable` (CS12 parts c to f) for the activation at the grand opening (no part
      separates it from entry, see Confidence), the place-order, deploy and trigger-created
      arrays, power that returns adding nothing (part c, when a run reaches its case),
      the removal radius and the count below 0 (parts d and e) and the new owner getting no
      coverage at the capture of an array that has had its grand opening (part e, which has no
      control for the array's coverage before the capture). `unmeasured` for:
      - the restored replacement: no part produces one, and when its handover falls rests on
        S-0524 claim 15, at medium
      - the owner change during the build-up: no part times a change-owner action inside a
        placed array's build-up
      - the old owner keeping its coverage at the capture of an opened array or after the sale,
        and a depowering
        removing nothing: *K*'s coverage is seen only through its shooter's fire, a hit makes
        the witness start uncloaking (claim 12), so fire after the first hit does not show kept
        coverage, and no witness first exposed after the capture or the depowering is
        established here.
    - Confidence: high (no disc for an array placed by a place order or a deploy; the removal
      radius and count); medium (the new owner getting none at the capture of an array that has
      had its grand opening: part e has no control showing that *K*'s set-up array added coverage
      before the capture, which rests on the medium item below; if it never added any, the human
      not sensing the witness after the capture would also fit a rule that moves the old owner's
      coverage to the new owner; activation only at the grand opening rather than at entry: for a
      placed or deployed array both moments fail the behaviour condition, for a set-up or
      trigger-created array they coincide, and the owner change during the build-up, which would
      separate them, is `unmeasured`; power that returns adding nothing: part c reaches its case
      only if *K*'s set-up array is out of operation at its grand opening, which rests on the
      medium item below, so a run may never reach it; the trigger-created array, which rests on
      an inferred order of opening and queueing; the restored
      replacement; the owner change during the build-up, both of its outcomes; the old owner
      keeping its coverage and a depowering removing nothing; which other conditions a set-up
      array meets at its grand opening)

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
    - **Uncloaked, or fading at stage 0:** drawn normally.
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

    A protected object drawn translucent or rippling keeps its protection brightness and takes
    no protection colour mask (S-0513 claim 29b).
    - Backing: `measurable` (CS1, part b; CS12); protection appearance inherits S-0513
      claim 29b's backing and measurement scenario
    - Confidence: high for cloak appearance; protection appearance inherits S-0513
      claim 29b's confidence

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

    A departed superweapon warp member that is the target, off the map or phasing in after it
    appears, is an ordinary target: the firer copies it as above (S-0306 claim 17).
    - Backing: `measurable` (CS13, part a; S-0306 scenario P11 run g, off the map, and run h,
      phasing in, for a departed warp member target)
    - Confidence: high (infantry); medium (vehicle); medium for a departed warp member target, as
      S-0306 claim 17 rates it

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

    A disguised building or aircraft instead appears disguised to every house, its owner
    included, whatever the alliances and detector coverage. The common rules above apply to
    infantry and vehicles, not these two kinds. This includes targeting and retaliation
    wherever they use this predicate. A disguise without a house satisfies the last condition
    for infantry and vehicles: it is not a failed or missing-house test. In particular,
    enemy infantry with such a disguise satisfy S-0209 claim 4 row WP25 when its other
    conditions hold. Detection by mobile
    detector units does not enter this test: it acts only in acquisition (S-0309 claim 12, table C).
    - Backing: `measurable` (CS15, CS18 part H), except the no-house infantry case is
      `unmeasured`: no ordinary skirmish disguise action is known to create that state.
      CS18 part H describes the discriminating fixture if such a state becomes available.
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

    The disguise is drawn in its disguise house's colours. The exception is an object of a type
    with a named palette, within the scope of S-0103 claim 16: it is drawn with that named palette
    at its real owner's colour definition instead, unless `quirks.named_palette_true_owner`
    (S-0103 Edge cases) is off. A unit disguised as an overlay or terrain object is lit like that
    object.
    - Backing: `measurable` (CS16), except:
      - the detector-coverage condition of (c) is `unmeasured`: the viewing house must be the
        human player, and no part here gives a human house building detector coverage in a
        skirmish (S-0009 claim 27 drops its set-up buildings; claims 23 and 40 give none to a
        built, deployed or trigger-created one, none at the capture of one that has had its grand
        opening, and none to a mobile detector; the one route left, an owner change inside a
        detector's build-up, is timed by no part),
      - the named-palette exception carries S-0103 claim 16's backing for its disguise rule:
        S-0103 scenario PQ part C for infantry, `unmeasured` for aircraft, buildings and vehicles
        drawn as sprites without a turret, as S-0103 gives.
    - Confidence: high ((a), (b), (c) without detector coverage, the disguise house's colours for
      a type without a named palette, the lighting); medium (the detector-coverage condition of (c); the named-palette exception, at
      S-0103 claim 16's level for its disguise rule)

38. **Radar colour.** For viewing house *V*, a disguised unit appears on the radar in its owner's
    colour when its owner allies *V*, including when *V* is the owner. Otherwise it appears in the
    disguise house's colour. This is the one-way owner → viewer relation, not viewer → owner.
    Mutual alliance is not required. When the disguise-house branch gives none, the radar object is still drawn, using the fixed neutral radar fallback colour of S-0410 claim 8, not the owner's colour. This is the fallback scheme identified in Import mapping and also used for an ownerless radar object. This agrees with S-0410 claim 8's no-disguise-house fallback; no change to that claim is needed.
    - Backing: `measurable` (CS16 part c, CS18 part H)
    - Confidence: high

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
    - The activation, power and capture rules of claim 23 apply. A detector building placed by a
      place order, deployed, restored as a replacement or created by a trigger action therefore
      never adds detector coverage, unless its owner changes during its build-up as claim 23
      gives.
    - Removal uses the detection range, with no floor (`quirks.sensor_count_negative`), whether
      or not the building ever added its coverage.
    - Detector coverage serves only claims 35 and 37. It does not reveal cloaked objects.
    - A mobile unit whose type detects disguises gives no coverage; it only changes its own target
      acquisition (S-0309 claim 12, table C).
    - Backing: `measurable` (CS15 part b) for the coverage of a detector placed at set-up and its
      detection range. `unmeasured` for a detector that entered play under construction or that a
      trigger action created: its coverage would be observed by its owner's units acquiring a
      disguised enemy, which needs the human's detector and infantry of a computer house disguised
      as the human's, and no set-up or scripted means of giving a computer house's infantry such
      a disguise is established here. The rule rests on claim 23's activation rule, which CS12
      part f measures for arrays. Removal: `unmeasured` here. A count left below 0 would show
      where the removed detector's disc overlaps a set-up detector of the same owner, whose
      coverage would end there; no part runs this, and the rule rests on claim 23's removal
      rule, which CS12 parts d and e measure for arrays. Capture after the grand opening:
      `unmeasured`: *K*'s units already
      hold the disguiser when the capture happens, a search never replaces a kept target (S-0309
      claim 4), and whether lost coverage drops a held target is not established here, so
      acquisition that goes on after the capture does not show kept coverage.
    - Confidence: high (the coverage of a detector placed at set-up and the detection range),
      medium (a detector placed by a place order, deployed, restored or created by a trigger
      action never detects; the owner change during the build-up; removal and capture, by
      claim 23's rules)

### Underwater vessels

41. **Submerging and surfacing.** A vessel is *submerged* while its type is underwater and its cloak
    state is cloaking, cloaked or uncloaking. Submerging and surfacing are therefore claims 5 to 11
    applied to a vessel, with no naval rule of their own.
    - A newly placed self-cloaking underwater vessel within the usable area starts surfaced
      and begins submerging on an eligible update. Outside-area placement can start it
      already submerged (claim 16).
    - For positive *S* and *N* ≥ 2, it needs (*N* − 1) · *S* ticks of surfacing before a weapon
      with the fire-only-visible flag can fire. Claim 43 and the zero-speed quirk govern the
      other inputs.
    - Its detection is that of any cloaked object: sensor coverage (claims 21 to 23) and the
      adjacency reveal (claim 13).
    - An underwater type that cannot cloak is never submerged (S-0504 claim 6).
    - Backing: `measurable` (CS1, part e)
    - Confidence: high

### Turn timing and degenerate inputs

42. **Cloak timing within a turn.** The following order applies when the object receives
    an ordinary cloak update; it does not grant that update during every warp stage.
    Due behaviour processing precedes the ordinary cloak update, for both mobile objects
    and buildings. Consequently a shot or uncloak cause
    produced by that behaviour is already visible to the cloak update in the same turn.
    Ordinary mobile locomotion and the separate infantry/vehicle firing attempt follow the
    cloak update. A unit whose uncloak completes there can therefore make its ordinary
    firing attempt later in that turn, whereas an aircraft attack-run shot made during due
    behaviour processing has already had its chance. Building behaviour-fired shots likewise
    precede that building's fade update. This is not an end-of-turn cloak phase.
    - The fading-stage step, when due, precedes the low-health abort test and completion
      test. They use the resulting stage. No abort draw occurs on a step reaching the fully
      cloaked threshold, because that stage is no longer in the abort band. A stage still
      in the abort band can trigger a draw even on an update with no step.
    - Each ordinary cloak update advances at most one stage, never several overdue steps.
      Event-driven field or sensor re-evaluations are additional and immediate (claim 6),
      not delayed until the object's next ordinary cloak update. Explicit uncloak causes
      likewise change the cloak state when they occur (claims 11–13), without requiring
      an ordinary update; starting an uncloak does not itself advance its fading stage.
    - **Warp suppression (engine design).** For the members covered by S-0513 claim 51,
      ordinary cloak updates are suppressed from the first pending warp update through
      arrival effects, inclusive. They regain ordinary eligibility during arrival hold,
      after the hold-end test. No missed fade steps are replayed: the next ordinary update
      still advances at most one stage. Elapsed-tick cloak and stage countdowns continue
      to elapse, but elapsed time alone does not advance a fade or complete an uncloak.
      This suppression does not postpone applicable immediate damage reactions, field or
      sensor re-evaluations, or the arrival sensor transfer and adjacency reveal of
      claims 13 and 22. Each event retains its own conditions. This is the engine contract,
      not a finding that every original object kind suppresses cloak work at all those
      boundaries; S-0513 claim 52 retains that original-side uncertainty.
    - Backing: `measurable` for ordinary ordering (CS18 part A) and for warp suppression
      in our engine only (CS18 part A's engine controls). Original per-stage warp
      continuation is `unmeasured`: no isolated per-kind continuation observation is
      established here; S-0513 claim 52 retains that evidence gap.
    - Confidence: high for ordinary ordering and the design contract; original per-stage
      warp continuation remains unresolved as stated in S-0513 claim 52

43. **Degenerate mobile fade inputs.** A negative stage speed is nonzero but its stage
    countdown is already expired on every update. Both fade directions therefore advance
    one stage per ordinary cloak update, not stall. For *N* ≥ 2, the durations are F1 and F2
    with an effective speed of 1 tick per stage. The zero-speed case remains distinct
    (`quirks.cloak_zero_speed_stall`).
    - For *N* = 1 and positive speed, cloaking takes one stage step, hence *S* ticks after
      an ordinary start. Uncloaking starts at stage 0, and completes at its next ordinary
      cloak update without waiting for the stage timer. This can be later in the same turn
      when an uncloak cause precedes that update (claim 42). It starts the usual cloak delay.
      With negative speed the single cloaking step takes one update instead.
    - A nonpositive stage count is invalid. Import it as 1 with a warning, giving the preceding
      behaviour. The original permits invalid division or a fade that cannot complete for
      such values; that invalid-data failure is not reproduced and has no quirk switch.
      This normalization does not change buildings' level-based fade.
    - Backing: `measurable` (CS18 part E, including the original's invalid-input contrast)
    - Confidence: high

## Formulas

F1, cloaking time (integers). From the tick cloaking starts:

> *T*_cloak = *c* × *S*, with *c* = ⌈*N* / 2⌉. Units: ticks.

These duration formulas apply for positive integer *S* and *N* ≥ 2. Claim 43 defines
negative speeds and stage counts below 2; zero speed is covered by its quirk below.

The original computes each stage's fade fraction in floating point, with a 53-bit significand
and rounding toward zero after each operation (S-0205 claim 5), as
the stage divided by *N*, times 256, truncated toward zero. For integers 0 ≤ *k* < *N*, this
always equals ⌊256 *k* / *N*⌋, so *c* is the first *k* with ⌊256 *k* / *N*⌋ ≥ 128, which is
⌈*N* / 2⌉.

F2, uncloaking time (integers):

> *T*_uncloak = (*N* − 1) × *S*. Units: ticks.

F3, cloak delay. *d* is the game-wide cloak delay in game minutes. It is read as a decimal per S-0003
claims 6 and 7: single precision, widened, and scaled when written with a percent sign.

> *D* = trunc(*d* × 900.0). The multiplication rounds toward zero to a 53-bit significand,
> then the result is truncated toward zero to integer ticks (S-0205 claim 5).
> Units: ticks; 900 ticks per game minute.

F4, health test (double precision, 53-bit significand). Let *h* be the integer health,
*h*ₘₐₓ the positive integer strength and ρ the red ratio, imported as S-0302 claim 12 says.
The division rounds toward zero to 53 significant bits (S-0205 claim 5); there is no integer
rounding before comparison.
- Claim 5 starts at once when *h*/*h*ₘₐₓ > ρ (strict).
- Claim 8 applies when *h*/*h*ₘₐₓ ≤ ρ.

F5, cloak field circle (integer midpoint recurrence). For a positive integer radius *r*,
the initial state is (*x*₀, *y*₀, *e*₀) = (0, *r*, 3 − 2*r*). For each state with
*x*ᵢ ≤ *y*ᵢ, the next state is defined by:

| Condition | *x*ᵢ₊₁ | *y*ᵢ₊₁ | *e*ᵢ₊₁ |
|---|---:|---:|---:|
| *e*ᵢ < 0 | *x*ᵢ + 1 | *y*ᵢ | *e*ᵢ + 4*x*ᵢ + 6 |
| *e*ᵢ ≥ 0 | *x*ᵢ + 1 | *y*ᵢ − 1 | *e*ᵢ + 4(*x*ᵢ − *y*ᵢ) + 10 |

The candidate circle is the union, over all states with *x*ᵢ ≤ *y*ᵢ, of the cells in rows
*dy* = ±*x*ᵢ with |*dx*| ≤ *y*ᵢ and rows *dy* = ±*y*ᵢ with |*dx*| ≤ *x*ᵢ.
Displacements are integer cells relative to the generator centre; all arithmetic is integer,
with no rounding. This defines a set of cells, not a required enumeration order. Claim 20
separately defines interruptions that can leave some candidate cells uncovered.

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
| Cloak-delay conversion | 900 | ticks per game minute | F3 |
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
| cloaking | level 15 | cloaked | Natural smoke system removed at once with its particles |
| cloaked | should not be cloaked | uncloaking | Level falls 1 per tick |
| cloaking | loses eligibility | cloaking | Completes fade, then uncloaks if still ineligible |
| cloaking or cloaked | explicit uncloak cause, claim 11 | uncloaking | Preserve fade level, sound once |
| uncloaking | ready inside own field, cell re-evaluated | cloaking | Current fade level preserved |
| uncloaking | level 0 | uncloaked | New natural smoke system, only if its type names one, the building has none, and the natural-smoke displacement is not (0, 0, 0) |

**Ordering.** Claim 42 places the ordinary cloak update after due behaviour processing,
  before ordinary locomotion and separate infantry/vehicle firing. Its stage step precedes
  its abort/completion tests.
- The draws of claims 5, 8 and 33 are taken in the object's own turn (S-0203), at most one per
  object per tick for each claim.
- A unit on a newly covered cell can start cloaking in the generator's tick before its own
  update. An attacker processed later that tick can therefore lose this target even when the
  target has not yet had its own update.

## Edge cases and quirks

- **`quirks.table_root_distance`** (default on, original behaviour; owned by S-0204 claim 5).
  Claim 13 uses ⌊*r*(*n*)⌋, with *n* = *u*² + *w*² in square leptons and the table root
  defined by S-0204 claim 5. Off: use ⌊√*n*⌋, the exact integer square root, in leptons.
  Both results are non-negative integers, rounded down. This changes only the candidate's
  distance measure, not the top-corner reference, height exclusion or first-listed tie rule.
  The same choice applies to claim 18's candidate test. It is the shared distance switch,
  not a separate sensor switch.
- **`quirks.cloak_flag_ground_mobile_only`** (default on). The type's cloak flag is ignored for
  aircraft and buildings. Fixed behaviour: aircraft and buildings with the flag can cloak by
  themselves.
- **`quirks.uncloak_from_cloaking_full_duration`** (default on). An uncloak that starts while the
  object is still cloaking takes the full (*N* − 1) · *S* ticks. Fixed behaviour: it starts from the
  current stage.
- **`quirks.cloak_zero_speed_stall`** (default on). With a cloaking speed of 0:
  - cloaking advances one stage per tick
  - for *N* ≥ 2, uncloaking never advances, so the object stays at stage *N* − 1 forever,
    drawn according to claim 28, and a weapon that needs it visible never fires.
  - for *N* = 1, uncloaking starts at stage 0 and completes at its next ordinary update
    as in claim 43, despite the zero speed.

  Fixed behaviour: uncloaking also advances one stage per tick when its stage is positive.
- **`quirks.sensor_building_activation_once`** (default on). Sensor arrays and disguise detectors:
  - their coverage is set at their grand opening only, if they are in operation then, so one whose
    grand opening is at its handover or at its creation by a trigger action never gains it
    (claim 23)
  - power changes do nothing afterwards
  - the capture of one that has had its grand opening leaves coverage with the old owner.

  Fixed behaviour: coverage follows *in operation* and moves to the new owner on capture, so a
  built array or detector that is in operation gains its coverage once its construction has
  ended.
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
- **Empty default-disguise list.** Our engine treats an empty game-wide default-disguise list
  as "no disguise is taken" and draws nothing, with an importer warning; this is a data error,
  not a switch. The original instead attempts a selection outside the empty list.
  - Backing: `measurable` for the engine's safe result (CS22 part a); the original's invalid
    selection is `unmeasured` because no safe, reproducible observable outcome is established.
  - Confidence: high
- **Disguised infantry with no disguise type.** When resolving a missing infantry disguise
  type, our engine uses the owner's side default, never the viewer's side default. The original
  instead falls back on the local player's side default, a display-only bug with no switch.
  This does not invent a default when the owner's side default is itself absent.
  - Backing: `measurable` (add a measurement scenario using named fixture data that supplies
    a missing disguise type and distinct usable owner/local-player defaults; record the
    resulting type and repeat with the defaults swapped).
  - Confidence: medium
- **Very large stage counts.** With *N* ≥ 256, the first uncloaking stage has *x* ≥ 255. Such an
  object is not drawn for non-owners at that stage, but it is not hidden in game logic.
  - Backing: `measurable` for the image (CS1 part b) and the game-state distinction (add a
    measurement scenario using named fixture data with *N* ≥ 256; during the first uncloaking
    stage, record the non-owner image and an otherwise-valid ordinary targeting/order result).
  - Confidence: high for the image; medium for the game-state distinction
- **`quirks.mobile_sensor_removal_centre`** (default on). Leaving play removes a mobile
  sensor's disc around its current position cell, producing the coverage drift of claim 22
  when it has not yet completed movement into that cell. Off: remove the disc actually last
  added by that sensor, retaining the floor at zero.
- **`quirks.flying_sensor_disc_stationary`** (default on). Ordinary aircraft flight, rocket
  flight and airborne jumpjet movement never relocate a sensor disc. Ordinary aircraft
  landing also does not; jumpjet touchdown does, as a real cell-arrival update. Off:
  horizontal cell-boundary crossings during these flights relocate the disc to the new cell.
  This alternative changes coverage only; it adds no adjacency uncloak events to flight.
- **`quirks.cloak_generator_zero_radius_shutdown`** (default on). Manual power-off makes
  shrinking pending and substitutes the type radius for a current radius of 0, without
  adding coverage. Off: it requests shrinking only at a positive current radius, matching
  ordinary loss of operation. It never invents a positive radius for an empty field.
- **`quirks.cloak_field_building_interrupt`** (default on). Claim 20's partial coverage and
  interrupted shrinking/restart are retained. Off: apply each circle in full, without an
  early stop at an own building's centre.
- **Invalid fade count and negative field radius.** Claims 43 and 20 give the safe import
  substitutions, without switches. The original's invalid arithmetic or wrapped-radius
  growth has no useful gameplay meaning.

## Import mapping

| Key (section) | Concept | Parse form | Absent-key default |
|---|---|---|---|
| `Cloakable` (object type) | Own cloak flag, claim 2 (a) | boolean | no |
| `CloakStop` (object type) | Cloak-stop, claim 3 | boolean | no |
| `CloakingSpeed` (object type) | *S*, claims 7 and 43 | signed integer, ticks per stage; negative means one step per update in both directions; zero retains its quirk | 7 |
| `CloakingStages` (General) | *N*, claim 43 | signed integer; warn and substitute 1 for values ≤ 0 | 9 |
| `CloakDelay` (General) | *d*, F3 | decimal, game minutes | 0.0 |
| `CloakSound` (AudioVisual) | Claim 15 sound | sound name | none |
| `CloakGenerator` (building type) | Cloak generator, claims 19–20 | boolean | no |
| `CloakRadiusInCells` (building type) | *r*, also claim 23 removal | integer, kept as signed 8-bit; zero gives empty field; negative effective value warns and uses zero for generator growth | 20 |
| `Sensors` (object type) | Sensors flag, claims 13 and 18 | boolean | no |
| `SensorsSight` (object type) | *R*, claims 22–23, cell-arrival disc and removal drift; a nonzero range, not the sensors flag, supplies coverage for aircraft too | integer, cells | 0 |
| No data key | Claims 13 and 22: cell-arrival timing for every movement mode, warp, drop and unload, distinct from entering-play coverage | engine behaviour; locomotor import belongs to S-0207 claim 14; ordinary flight, jumpjets and rockets to S-0501, S-0502 and S-0506 | claim 22 table and relocation clauses |
| No data key | Claim 20: phase transitions, own-centre interruption, idle-neighbour eligibility and zero-radius immediate collapse | engine behaviour, not a separate radius or power setting; operation and accepted power orders belong to S-0405 claims 8 and 10 | initially idle at radius 0; operation changes request direction, unchanged operation does not; immediate collapse tests neighbours even for type radius 0 |
| No data key | Claims 11, 13, 18 and 42: object-crush uncloak (not overlay-only), candidate distance, ties, explicit building uncloak and within-turn timing | engine behaviour | fixed |
| No data key | Claim 16: passenger cloak state, stage and elapsed delay | engine behaviour, independent of transport capacity imports in S-0507 | preserved on ordinary boarding and exit |
| `LightGrey` (Colors) | Claim 38 fallback radar colour | named colour definition, S-0103 claims 6–10; use its one-shade scheme's radar colour as for an ownerless radar object in S-0410 claim 8 | fixed lookup name; colour values come from user data |
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

All scenarios use the measurement setup, game-speed index 2, 800 × 600 pixels, no engine add-on and no mods (the named test files of a scenario excepted), and a small purpose-made skirmish map. One bystander computer player
is kept outside the observed area under the glossary's void rule. Named fixture **CS-fixture**
means that map's temporary override sections: only the abilities, weapons, radii, timing fields
or health conditions explicitly required by a sub-test are changed; unrelated data is left
unchanged. All named test subjects below are CS-fixture roles, not assertions about any shipped
type's settings. Explicitly configure each role's required cloak sources, sensor range,
disguise flags and weapon properties. Where a source must be absent, clear it in the fixture
and provide no field or crate granting it. Remove all overrides afterwards.

**Coverage of sensor-array and detector buildings.** Of the ways the parts here bring such a
building into play (placement by the map at game set-up, a place order, a deploy, a trigger
action, each with no owner change during a build-up), placement by the map at game set-up is the
one that adds coverage (claims 23 and 40), and in a skirmish a structure placed at set-up for a
human player's house is dropped (S-0009 claim 27). A part that needs active building coverage
therefore gives the building to a computer player *K*, placed at set-up, unless the part says
otherwise; a building that a part has the human place, deploy or get from a trigger is the
human's. *K*'s coverage is observed through *K*'s own objects, not through *K*'s view:
- **Sensing.** A fully cloaked human-owned witness on a cell that *K* senses is acquired and fired
  at by *K*'s **shooter**; on a cell that *K* does not sense it is not (S-0309 table C, row
  "cloak"; claim 24). The shooter is an armed defence building of *K*, placed at set-up, of a
  CS-fixture type with no needs-power flag that is not a sensor, with the witness inside its
  weapon range and outside every adjacency reveal (claim 13). The witness has CS-fixture-raised
  health, so that it survives the observed span. Only the shooter's first fire at a witness
  decides a part: a hit makes the witness start uncloaking (claim 12), so later fire does not
  show coverage.
- **Detection.** Human-owned infantry disguised as one of *K*'s infantry types is acquired by *K*'s
  armed units only on cells where *K* has detector coverage (claim 35; S-0309 table C, disguise
  row).
- **Later sensors.** Mobile sensors that a part adds for *K* after set-up are unarmed CS-fixture
  types created by map triggers, so that no order of *K*'s is needed.
- **Route control.** Each such part has a run without *K*'s array or detector, in which the
  shooter does not fire at the witness and *K*'s units do not acquire the disguised infantry. A
  run whose route control fails is void.

*K*'s start units, its base and its other activity are kept outside the observed area under the
glossary's void rule. Inside it, *K* owns only the objects a part names.

Use tick-stamped screen and sound capture, scripted input and a read-only recorder. Repeat each
deterministic sub-test three times. Record target lines, selection, radar pixels, images, sound
and health as named below. Each sub-test inherits this map, speed, recording setup and repeat count unless it says
otherwise. Its observed area is the test objects' cells, their coverage discs and the routes
between them. Views from different houses are recorded in separate skirmish runs with the
viewing house as the human participant; a view change must not change game state.
Every expected result below, including each table entry, is falsified by a differing result
with the same eligibility conditions. A failed setup or an independent refusal makes a run
inconclusive, not confirming evidence. CS17 supplements CS1–CS16 for clauses not isolated below.

- **CS1, timings.** Setup: an infantry type that cloaks by itself, at full health, on open ground.
  - **(a)** Record the tick at which it starts cloaking and the tick at which it becomes fully
    cloaked. Expected: *c* · *S* ticks with the imported *S* and *N*. Falsified by another value.
  - **(b)** Record stage 0 and each positive stage and its image as owner and as an unsensing
    enemy. Repeat with a self-cloaking vehicle to observe its shadow. Expected: stage 0 draws
    normally; at positive stages below fraction 64,
    25 % translucency; 64–191, 50 %; 192–254, an enemy sees rippling and the owner sees
    50 %; at least 255, no enemy image and 50 % for the owner. Use a large stage count
    to reach the last band while uncloaking. Fully cloaked, only the owner sees the
    unsensed unit, at 50 %. No vehicle shadow appears in any non-uncloaked state.
    Any wrong band, visible unsensed enemy full cloak or surviving fading shadow falsifies it.
  - **(c)** Order it to fire at an in-range target using a weapon requiring visibility, with
    ammunition, facing and rearm ready. Expected: uncloak completes after (*N* − 1) · *S*
    ticks; its first otherwise-due separate firing attempt may shoot then, not earlier.
    In a separate delay run use *d* > 0, clear the target and let rearm expire before the
    delay expires. Expected: no new cloak before *D* ticks after completion, then a start
    on the first eligible update. An earlier shot or re-cloak, or a further delay without
    another blocker, falsifies the corresponding rule.
  - **(d)** Record sound events and their positions at cloak start and forced uncloak start.
    Configure an audible game-wide cloak sound and a distinct type-specific sound in
    CS-fixture. Expected: one game-wide sound at each start, located at the unit, never
    the type-specific sound. A missing, duplicate or wrong sound falsifies claim 15.
    CS17 separately checks silent aborts and repeated causes.
  - **(e)** Repeat (c) with a fixture underwater vehicle given its own cloak flag and a
    visibility-requiring weapon. Expected: it is submerged throughout all three
    non-uncloaked states and cannot fire its visibility-requiring weapon until surfacing
    completes after F2 ticks. Earlier firing or a different state boundary falsifies claim 41.
- **CS2, sources.**
  - **(a)** Record the fixture self-cloaking underwater vehicle's placement at a compatible
    yard within the usable area, before its first ordinary update. Expected: initially
    uncloaked, then normal submerging on an eligible update, not an immediate full cloak. Compare outside-usable-area placement
    with the same own cloak flag: that placement is forced fully cloaked.
  - **(b)** Give a landed aircraft and a building the type cloak flag only, with no crate,
    ability or field source. Keep them otherwise ready at full health. Expected: neither
    starts cloaking. Either starting falsifies the type-flag restriction.
  - **(c)** A cloak-stop vehicle: ordered to move while uncloaked, it never starts cloaking. Once
    fully cloaked and then moved, it stays cloaked.
  - **(d)** Place a landed aircraft inside its owner's field, then repeat airborne.
    Expected: a ready landed aircraft starts cloaking; an airborne one does not. After
    cloaking while landed, take off without another uncloak cause: it stays fully cloaked.
    A failed eligible landed start, airborne start or takeoff-only uncloak falsifies it.
- **CS3, ready conditions.** Give a full-health self-cloaker a retained enemy target inside
  its attack range, with all other start blockers absent. Keep rearm expired and prevent
  firing with a separate weapon restriction. Record target, range and cloak state. Expected:
  no start until the target leaves attack range, then a start on the first eligible update.
  A start with that in-range target or an unexplained wait after it leaves falsifies the rule.
  Mere enemy presence without a retained target is not the tested blocker.
- **CS4, the red-health draw.** Use CS-fixture self-cloakers below, exactly at and above
  the red ratio, with no other start blocker. Across fresh skirmishes collect at least
  1000 eligible start updates and 1000 eligible abort-band updates; record the random
  stream and correlate each ranged result with the transition, rather than using a
  statistical tolerance as the verdict.
  - **(a)** At/below red, expected: one R(0, 99) per eligible start update, with a start
    exactly when the result is below 4. Any different draw count or outcome falsifies it.
    The waiting-time distribution should consequently be geometric with *p* = 0.04.
  - **(b)** Above red, including full health, expected: immediate start without a start
    draw. A random wait or such a draw falsifies it.
  - **(c)** During cloaking at/below red, expected: one R(0, 99) on every update in the
    post-step 64–127 fraction band, and an abort exactly for results below 10. There is
    no abort draw outside that band. An inconsistent transition or draw falsifies it.
    No sound accompanies an abort. CS18 A isolates the completing-step boundary.
- **CS5, the cloak field.**
  - **(a)** An own vehicle with no independent cloak source parks inside an operational
    generator's field, with no readiness blocker. Expected: it starts cloaking on the field
    re-evaluation even at low health, without a health draw; the normal fade duration still
    applies. A health-dependent wait or draw falsifies this field-start rule.
  - **(b)** It drives out. Expected: it uncloaks on leaving.
- **CS6, EMP.** Disable an own-flag self-cloaker with a fixture EMP attack, then let it
  finish any hit-triggered uncloak and cloak again inside its owner's field while still
  EMP-disabled. Keep a matched disabled cloaked control inside coverage and shrink the
  field off the test unit. Record coverage, disable expiry and cloak state. Expected:
  the disabled unit inside the field stays cloaked; the one losing coverage starts
  uncloaking on its next ordinary cloak update. A spontaneous inside-field uncloak or
  continued full cloak outside falsifies claim 9, provided no ability supplies another
  cloak source. This isolates ongoing disable eligibility from claim 12's immediate hit
  reaction; a revealing hit is not evidence that EMP alone defeated field coverage.
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
  - **(d)** Keep an aircraft targeting an enemy through both cloak transitions by sensing
    the target at both moments. Once fully cloaked, remove that coverage without moving
    or damaging the target. Repeat with coverage retained and with an allied target.
    Record retained target. Expected: the now-unsensed non-allied target is dropped.
    This specific aircraft rule does not drop the sensed or allied control. A retained
    unsensed hostile target falsifies claim 25. Earlier loss or an independent target-loss
    cause makes the aircraft-specific trial inconclusive.
  - **(e)** On explored ground, hover/click a fully cloaked mobile object as owner, ally and
    enemy, then add the viewer's own coverage. Expected: owner can pick it; unsensing
    non-owners cannot, allies included; sensing removes this cloak-specific refusal.
    Select a sensed non-owned object before cloak start: selection is still lost at start.
    Repeat on buildings: level 15 without sensing cannot be picked even by the owner.
    A differing click/selection result without another refusal falsifies claim 26.
  - **(f)** With radar available and the cell otherwise visible to radar, record the pixel
    of a fully cloaked object as ally and enemy, with and without the viewer's coverage.
    Expected: no pixel without own sensing, a pixel with it. An unsensed pixel or a
    sensed omission solely due to cloak falsifies claim 27.
- **CS10, buildings.**
  - Count ticks from power-on until a building in a generator's field is fully cloaked. Expected:
    15 ticks after the field reaches it.
  - Place an allied sensor unit next to the building. Expected: the building uncloaks.
- **CS11, field shape and rate.**
  - Record which cells' units cloak for invented fixture radii of 4 and 11 cells.
    Expected: F5's candidate circle, with claim 20's own-centre interruptions and omissions.
  - Cut power. Expected: rings leave one per tick.
  - Overlap two generators and sell one. Expected: units in the overlap uncloak briefly.
- **CS12, sensors.**
  - **(a)** Map the disc of a mobile sensor with sensors-sight *R* by walking a cloaked enemy
    unit along each axis. Expected: |*dx*| ≤ *R* − 1 and |*dy*| ≤ *R* − 1, with the strict squared-distance test, symmetric on both axes.
  - **(b)** Put an allied sensor over an explored, fully cloaked hostile witness outside
    adjacency distance, with no viewer-owned sensor there. Expected: the viewer has no
    sensed count and cannot see/attack it; the sensor's owner can. Shared coverage
    falsifies owner-only sensing.
  - **(c)** *K*'s array is placed at set-up while *K*'s output is below its drain, with the
    witness in its disc and inside the shooter's weapon range from the start; a map trigger later
    gives *K* a power plant that ends the low power. The trigger fires no earlier than twice the
    longest targeting-timer interval that S-0215 F6 allows the shooter after the shooter stands in
    *guard* with the witness in range, so that at least one of its automatic searches (S-0309
    claim 1) runs before the restoration with the witness a candidate in every other respect.
    Expected: the shooter fires at the witness neither before nor after the restoration. A run in
    which it fires before the restoration is inconclusive for this half (the array was in
    operation at its grand opening, claim 23's medium on set-up conditions); so is one whose
    trigger fires earlier than that. Firing that starts after the restoration falsifies the
    once-only rule. A depowering is not tested here (claim 23,
    its backing).
  - **(d)** Give *K*'s array a cloak radius larger than its sensor sight in CS-fixture, and put
    the witness on a cell of the annulus between the two discs, which no other sensor of *K*
    covers. A map trigger destroys the array. A later trigger creates one mobile sensor of *K*
    whose disc covers the witness's cell, and a later one a second. Expected: the shooter does
    not fire at the witness after the array's removal, nor with one added sensor, and fires once
    the second covers the cell (counts −1, 0 and 1 there). Route control for the count: in a run
    without the array, one added sensor is enough. Firing with one sensor after the removal, or
    no firing with two, falsifies the removal-radius and negative-count rules.
  - **(e)** A human engineer captures *K*'s array, placed and in operation at set-up. A witness
    owned by *K* and fully cloaked stands on a cell that lies in both the array's sensor disc and
    its cloak-radius disc, and that no other sensor of the human covers. Expected: the human does
    not see *K*'s witness and cannot attack it after the capture (none is added for the new
    owner). The human then sells the array. Expected: in the human's view, *K*'s witness is
    revealed by two mobile sensors of the human moved over its cell, not by one. Sight of the
    witness after the capture and before any mobile sensor arrives, or a reveal with one
    sensor, falsifies claim 23. Whether the old owner keeps its coverage is not tested here
    (claim 23, its backing).
  - **(f)** The human house, with output well above its drain, places an array by a place order,
    a fully cloaked witness of *K* waits inside the array's sensor disc, and an armed vehicle of
    the human stands in *guard* with the witness in its weapon range. The human's base comes from
    a construction vehicle that the map places for it and that it deploys in the first ticks, and
    its power from plants it then places by place orders (S-0009 claim 27 drops structures placed
    at set-up for a human house). Each witness of this part, the control's included, stays on its
    cell throughout, so it has no cell-arrival update and no adjacency reveal (claim 13), by the
    human's objects or any other; a run in which one changes cell, or starts uncloaking before any
    fire at it, is void. Record in the human's view,
    from the handover onward, whether the witness is drawn, whether a forced attack on it is
    accepted, and whether the vehicle fires at it. Repeat with an array of a CS-fixture vehicle
    type that the human deploys into it, and with an array that a map trigger creates in play for
    the human house. Then sell the placed array and move one mobile sensor of the human over a
    cell of its cloak-radius disc that no other sensor of the human covered, then a second,
    while the witness stands in that cell. Control: an array of *K* placed at set-up with *K*'s
    output above its drain, with a human witness in its disc: the shooter fires at it (with its
    route control). Expected: no built, deployed or trigger-created array of the human ever
    senses the witness, power notwithstanding; after the sale, only two mobile sensors reveal it.
    Sensing after the handover, or after the trigger creation, falsifies claim 23's activation rule (the trigger-created run
    separates S-0405 claim 8's queued reading from a current-only one, as S-0405 scenario W6 part j
    does for a placed building before *construction* becomes current); a reveal with one mobile
    sensor after the sale falsifies its removal rule.
- **CS13, disguise acquisition.**
  - **(a)** Give a fixture infantry disguiser an unobstructed legal infantry target across
    the map. Configure its weapon with unlimited range and a disguise-making warhead. Record
    movement, detonation, both objects' health, and copied disguise type/house. Expected: no approach movement, unchanged target
    health, firer copies the target's type and owner; a disguised target instead supplies
    its disguise type/house. Movement required solely for disguise range, damage, or the
    wrong copied identity falsifies claim 31. CS17 tests the vehicle target-kind matrix.
  - **(b)** Create fixture infantry with both disguise capability and permanent disguise,
    with a valid side-default infantry type explicitly supplied. Record its disguise type,
    disguise house and an unsensing enemy's acquisition. Expected: the default type and
    owner's house are held, while the enemy can still acquire it. A different identity or
    refusal solely from this default disguise falsifies claim 32.
  - **(c)** Disguise permanent infantry as another house, then deliver an accepted surviving
    hit. Expected: its disguise resets to its owner's side default, rather than preserving
    the copied identity or becoming undisguised. Clear the permanent flag in a separate
    fixture run: a hit now clears the disguise. Move that temporary infantry disguiser
    without damage: movement alone preserves its disguise. The opposite state/identity
    in any of these controls falsifies claims 32–34.
- **CS14, disguise when still.**
  - **(a)** A still disguise-when-still vehicle takes a terrain disguise. The random-state recorder confirms one draw of
    range *n*.
  - **(b)** Enemy infantry stepping next to it removes the disguise on the next checked update,
    with checks on ticks not divisible by 8. It reappears after the infantry-blink time once
    the infantry has left.
  - **(c)** A hit delays re-disguise by the lockout length of S-0302 claim 12.
- **CS15, appears disguised.**
  - **(a)** Fixture infantry disguised as an infantry unit of house *E* is not auto-targeted by *E*'s
    vehicles. *E*'s allies are not fooled when the disguiser's owner is their ally.
  - **(b)** A detector building of *K*, placed at set-up with *K*'s output above its drain, makes
    *K*'s armed units acquire human-owned infantry disguised as one of *K*'s infantry types within
    the detection range (with its route control). A capture is not tested here (claim 40, its
    backing).
  - **(c)** With the auto-crush rule enabled in CS-fixture, put a crushable enemy infantry
    disguiser within crush range of an idle crusher, disguised as the crusher's house and
    without detector coverage. Record automatic movement and the infantry's survival. Expected:
    no automatic crush choice. Repeat with building detector coverage, the crusher and the
    detector building then belonging to *K* and the disguiser to the human house: it is chosen
    and crushed.
    Separately order the crusher along a route through the still-disguised infantry's cell to a
    cell beyond it. Confirm passage through the crush radius with the disguise still effective.
    Expected: the ordered drive-through crushes it. Automatic choice while fooled, or survival
    solely due to disguise during a valid drive-through, falsifies the distinction (S-0210
    claims 2–3 and 11). A missed route or lost disguise makes that trial inconclusive.
- **CS16, drawn disguise.**
  - **(a)** Record the owner's view every tick for two full 256-tick cycles after disguise
    acquisition, without damage, motion or firing. Expected: disguise image at cycle
    phases 8–55, real image otherwise, with the exact translucencies of Table B.
    Any wrong image, band boundary or cycle period falsifies claims 37 and F6.
  - **(b)** Record the same unit, of a type without a named palette, as mutual ally and enemy,
    with no fake blink or detector coverage. Expected: both see its disguise in the disguise
    house's colours. An ally-only real image or wrong colours falsifies the corresponding rule. A
    disguiser of a type with a named palette is not part of this scenario (claim 37's exception,
    its backing). A viewing house with building
    detector coverage is not part of this scenario: the viewer is the human player, and no part
    here gives a human house such coverage in a skirmish (claim 37's backing; mobile detectors
    give none, claim 40).
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
  - **(d)** Fire a stationary disguised vehicle at a legal target without moving or taking
    damage, using two positive fake-blink times in separate CS-fixture runs. Record the
    projectile-creation tick, timer, retained disguise and enemy image. Expected: the
    timer is set on projectile creation and non-owners see the real unit while it runs,
    then the retained disguise again. The owner keeps its ordinary cycle. Repeat while
    undisguised: the timer is still set. Repeat with infantry: no fake blink starts.
    Wrong duration, disguise loss solely from firing, owner-cycle override or an infantry
    fake blink falsifies claim 39. Failure to create a projectile is not a timer-start test.

### CS17: clause-isolation checks

Use CS-fixture and the same recording setup and three-run count above. Each row supplements
the scenarios cited by the named claims. Vary one condition at a time. A contrary transition,
image, sound, targeting result or draw count falsifies the stated expectation.

| Claims | Procedure and observable | Expected contrast |
|---|---|---|
| 1, 7, 10 | Repeat CS1 with odd/even stage counts and several positive speeds. Hit during the first fading band; restore a field during uncloaking. Record bands and transition ticks. | F1/F2 hold. Interrupted cloaking still gives the full uncloak duration. Mobile objects finish uncloaking and wait out the delay rather than reversing. |
| 2, 3, 16 | Compare placed/factory-created vehicles and infantry within the usable area and vehicle placement outside it. Populate only the veteran, then only the elite cloak-ability list. Give cloak-stop to a crate-cloaked vehicle and move it while visible, then while cloaked. | Fresh ordinary placements are uncloaked. Only outside-area vehicle placement with the own flag forces a full cloak. Elite accepts either list. Crates do not bypass cloak-stop on starting; movement does not break a full cloak. |
| 4, 5 | Separately retain a rearm timer, an in-range target, the seized state, positive height and war-factory contact on a self-cloaker. Remove each blocker. Compare out-of-range targets. | Each listed blocker prevents a start; full health and no blockers permit it. |
| 5, 8 | Collect 1000 eligible updates at, below and above the red ratio, and 1000 low-health updates in the abort band across restarts. Read the stream state and match ranged draws. | At/below red starts use one draw below 4, above red none. Aborts require one draw below 10 in the specified band only. |
| 5, 6, 9, 17 | Compare own-flag and ability cloakers inside/outside a field under EMP and paralysis, after any immediate hit reaction. Give aircraft a crate or ability; compare landed and airborne starts, then take off after cloaking. For warp use CS18 A's engine contract and preserve its original-side limitation. | On eligible ordinary updates the disable gate bars own-flag starts outside a field, not ability/field starts when otherwise ready. Losing the last source breaks a full cloak. Airborne starts fail; takeoff alone does not break a full cloak. Suppressed warp updates are not failed eligibility tests. |
| 11, 12, 15 | Repeat CS7 for each accepted and rejected hit category in claim 12. Record sounds and fade. Repeat a hit while already uncloaking. | Accepted hits and listed uncloak causes sound; rejected hits do not reveal. Repeated causes during uncloaking do not replay sound. |
| 13 | Share a neighbouring cell between sensor and non-sensor infantry. Swap which is nearest the top corner. Repeat with an ability sensor and on a bridge. | Only the nearest ground-layer candidate counts. A sensor approaching a stationary target gives coverage, not adjacency uncloaking. |
| 14, 24–26 | Retain own-house, sensed-enemy and unsensed-enemy targeters at the mobile start/completion. Repeat on a building. Select a sensed non-owned object before it cloaks. Try positive-damage force fire on a hidden own target and healing on a hidden ally. | Unsensed enemies drop at both mobile moments, buildings only at the start. Non-owner selection is lost at cloak start even with sensors. Positive mean weapon damage is refused; nonpositive mean passes the cloak test for an own or allied target, independently of the selected weapon's sign. |
| 18 | Put sensor infantry in the foundation/border, with candidate-control occupants. Compare own/allied/enemy and veterancy-only sensors. Lose the field during cloaking, then restore it during uncloaking and change sensor coverage. Watch levels and particles. Use one named test building type whose natural-smoke displacement is (0, 0, 0), and one whose displacement is not (0, 0, 0); both are named test data, removed after the run. | Foreign type-flag sensors, allies included, block. Interrupted cloaking completes before uncloaking. Field re-evaluation can reverse uncloaking at its current level. Natural smoke is removed at once at 15; a new system appears at 0 only when the building has none and the natural-smoke displacement is not (0, 0, 0). |
| 19, 20 | Witness the circle boundary. Power, disable, sell, destroy and capture a generator separately. Test allied witnesses. With two generators cut one's power, then separately sell it. Repeat with a nearby other-owner generator. | F5's candidate circles, interrupted growth/shrinking and immediate removal hold. Allies are not covered. Overlaps can lose coverage; a full idle neighbouring generator restarts irrespective of owner. |
| 21–23, 40 | Observe detection across a mobile sensor's movement, capture and removal. Compare a type flag alone with a range alone. Restore *K*'s power after placing its array at set-up under low power (CS12 c). Capture *K*'s array (CS12 e). Compare mobile and building disguise detectors. | Coverage is owner-only and does not lift shroud. Mobile range supplies coverage. Restored power adds no array coverage, and capture adds none for the new owner. Mobile disguise detectors supply no shared coverage. |
| 23, 40 | Make the array's cloak radius larger than its sensor sight. After removal, cover an affected outer cell with one sensor, then two. Detector removal is not tested here (claim 40). | A formerly −1 cell needs two additions. Array removal uses cloak radius. |
| 27–29 | Compare owner, mutual-ally and enemy views on explored ground with/without own sensors. Record clicks, shadow, radar and images every tick. Include a stage count reaching fraction 255. | Claim 28 bands hold. Unsensed allies see the cloak image but cannot click or list it on radar. Fading vehicles have no shadow. Fraction 255 is not drawn for non-owners. |
| 30–32 | Copy undisguised/disguised infantry, disguised/undisguised vehicles, rock overlays and terrain objects. Create permanent disguisers with each side default. Remove the disguise house. Observe image, colour, health and acquisition. | Claim 31 copies and rejected kinds hold, without target damage. Defaults have the owner's house and do not fool enemy logic. Cleared permanent infantry resets to default; a vehicle clears outright. |
| 33, 34 | Move temporary infantry/vehicle disguisers; introduce neighbours on all 8 tick phases, ground/bridge layers, contact and lockout. Compare damage on permanent/temporary types. Try transport entry and capture. For undeployment, use S-0517's P8 (b) fixture and record disguise at the first undeploy processing and at completion, with and without a deploying presentation. | Only temporary vehicles lose disguise while moving. Adjacent checks skip ticks divisible by 8. An eligible still vehicle draws once. Damage follows S-0302 claim 12. The listed non-removal events preserve disguise. Undeployment clears disguise at the first step, before animated completion, or at the immediate step without a presentation; a disguise lost before the step makes that trial inconclusive. |
| 35, 36 | Vary disguise house among observer, ally and enemy. Vary ownership and detector coverage. Observe acquisition, retaliation, automatic crush choice versus ordered drive-through, cell entry, cursor, hover bar/name, insignia and scenery selection/flash. | Outcomes using claim 35 change with its predicate as claim 36 and the owning specs state. Disguise blocks automatic crush choice, not an otherwise valid drive-through. Detection changes the observer's treatment, not the copied disguise. |
| 37–39 | Capture the owner's full 256-tick cycle and each Table B band. Compare ally/enemy images and radar colours. Fire vehicles using two fake-blink times, also while undisguised; compare infantry. | Owner bands/cycle hold. Non-owners see real during vehicle fake blink, then the same disguise again. Infantry firing starts no fake blink. Radar uses owner colour when the owner allies the viewer, otherwise a non-empty disguise house's colour; repeat opposite one-way alliances as CS16 (c). |
| 41 | Compare two fixture underwater vehicles, one given its own cloak flag and one stripped of all cloak sources. Give both a visibility-requiring weapon. Fire after surfacing; add sensors and then cause adjacency reveal. | The control stays surfaced. Ordinary transitions give submerged states and F2 surfacing delay. Sensing alone does not surface it; adjacency can. |

Zero-speed check (claim 7's quirk): record cloaking and subsequent firing over 256 ticks with
speed 0 and *N* ≥ 2 in CS-fixture. Expected: cloaking advances one stage per tick but uncloaking does not
advance. A later shot from a weapon requiring visibility falsifies the stall.

### CS18: request-resolution checks

Use the CS-fixture, speed, capture tools and three deterministic repeats above. Overrides
are explicit temporary measurement data, not an engine modification. Record positions,
cloak states, stages, fade levels, countdowns and coverage with the read-only recorder,
and correlate each with visible firing, radar, sound and target retention. Tests needing
unavailable state construction are identified below rather than counted as measurements.

- **A — Turn timing (claims 8 and 42; Q-0720).** Give a self-cloaking infantry unit or vehicle
  a weapon requiring visibility. Arrange uncloak completion on a due ordinary firing tick,
  recording movement and shot ticks. Repeat with a building and an aircraft whose due
  behaviour attempts the shot. Expected: the mobile separate firing attempt can use the
  completed uncloak that turn, but behaviour-fired shots saw its pre-update cloak state.
  A reversed consequence falsifies the corresponding order. With several small positive
  stage counts and health below red, record the random stream over at least 1000 eligible
  turns. Expect abort draws only for the post-step 50 % band, never on the completing step.
  An abort on that step or a pre-step-only band draw falsifies claim 42. An in-range target
  that itself prevents cloaking is not a valid start-cloak timing control.
  For the engine-only warp contract, repeat with a fading member at each boundary of
  S-0513 claim 51. Record stage, cloak state, remaining countdowns and arrival sensor
  coverage. Expected: no ordinary fade step through arrival effects, elapsed countdowns
  continue, and the first eligible hold update advances at most one overdue stage.
  Arrange a surviving cloaked arrival beside an enemy sensor: arrival coverage transfer
  and adjacency uncloak still occur during arrival effects, before ordinary fade work
  resumes. An otherwise applicable field re-evaluation or surviving hit must likewise
  take effect immediately, not wait for hold. Any ordinary fade step in an excluded
  boundary, burst catch-up, or delayed applicable event falsifies this engine contract.
  These checks do not establish the original's unresolved per-kind continuation.
- **B — Mobile disc and removal (claim 22; Q-0721).** Give a fixture ship a positive sensor
  range and drive it past a stationary cloaked witness at the disc edge, outside adjacency
  range. Record boundary crossing, completed-cell arrival and first visibility. Expected: coverage changes on
  arrival, not crossing. Repeat with temporary ground-sensor data. Arrange destruction
  between crossing and arrival by a tick-timed incoming shot. Probe old-only, shared and
  new-only crescent cells afterwards, including an unrelated sensor contributing to the
  new-only crescent. Expected: old-only coverage remains, shared coverage loses this
  contribution and new-only counts lose one where positive. Removal of the old disc
  instead falsifies the drift. Repeat with boarding where a between-event exit from play
  can be arranged; a ride reached only after arrival is an inconclusive drift test.
- **C — Candidate and crushing (claims 11 and 13; Q-0722, Q-0723).** Share a neighbouring cell
  between sensor and non-sensor infantry. Exchange top, centre, left, right and bottom
  standing positions. Record cell-layer listing order and the moving enemy's uncloak.
  Expected: top wins over centre, centre over left/right, left/right tie by first listing,
  bottom last. If legal non-standing placements with unequal squared distances but equal
  truncated root distances can be found, record those placements and repeat: listing order
  must still break that tie. This extension remains conditional until such placements are
  established; the standing-position runs alone do not distinguish table and exact roots.
  A different candidate
  falsifies claim 13. Separately cloak a crusher and drive it through infantry, a crushable
  vehicle, only a crushable overlay/wall, and an empty cell. Remove other uncloak causes.
  Expected: only actual object crushing uncloaks with sound. A takeover-only run also
  produces no crushing uncloak. Record actual removals to exclude missed crushes.
- **D — Radius and interrupted circles (claim 20; Q-0724, Q-0729).** On an open-ground map,
  record the first covered cells with radius zero and positive radii, including 1. Expected:
  zero covers nothing; positive growth begins at radius 1, but the generator's own centre
  interrupts that update. Alone with initially clear coverage it covers exactly (0, −1),
  (−1, 0), (0, 0), not the full radius-1 circle. For a type radius of 1 the other two cells
  remain uncovered after growth becomes idle. On our engine with the interruption quirk
  off, the full circle is the control. Place own buildings with centre cells in several successive circles. Record covered
  cells immediately before and after each growth update. Expected: the first newly covered
  own centre in increasing row/column order ends that update, later cells wait for a larger
  circle, and omissions at the final radius remain. Repeat cutting power, including the
  update reaching zero: interruption can also leave uncleared cells and suppress restart.
  Complete circles despite such a newly changed centre falsify the interruption rule.
  A negative effective radius run is an original-only diagnostic, stopped before invalid
  growth becomes unsafe; our importer instead warns and uses zero, without an oracle
  equality requirement for that documented departure.
- **E — Degenerate fades (claim 43; Q-0725).** Compare positive, zero and negative speeds on
  an infantry self-cloaker outside a field, forcing an uncloak after completion. With a
  stage count of 1, expected: one positive-speed step to cloak, uncloak starts at 0 and
  completes on its next ordinary update regardless of the positive timer. For negative
  speed and a stage count at least 2, expect one stage per update in each direction.
  A negative-speed stall falsifies the claim. Probe zero and negative stage counts only as
  isolated original diagnostics with a bounded 256-tick recording: record unusable or
  non-completing fade arithmetic. Our import substitutes 1 with a warning; verify that
  separately as a documented invalid-data departure, not an original-equality claim.
- **F — Building uncloak (claim 18; Q-0726).** In a field, hit a building at fade level 7 with
  a zero-damage weapon that reaches the accepted hit reaction. Expected: immediate state
  change to uncloaking at level 7, one cloak sound, then decreasing levels. Record any
  subsequent field re-evaluation separately. A jump to 15 or continued cloaking without
  a distinct re-evaluation falsifies the claim. Merely losing power is a separate control:
  loss of eligibility alone does not reverse the fade early.
- **G — Passengers (claim 16; Q-0727).** Make an established self-cloaking vehicle uncloak by
  firing, board it, and unload after a brief ride. Repeat after a ride longer than its
  remaining cloak delay. Configure a compatible transport in the fixture if necessary.
  Record state/stage and remaining delay on both sides of the ride, before the next ordinary
  cloak update. Expected: no exit reset to fully cloaked, preserved stage, elapsed rather
  than paused/reset delay, and at most one overdue stage step on the next update. Repeat
  with cloaked infantry: its state persists until a subsequent ordinary cause changes it.
  A fresh automatic full cloak on ordinary exit, restarted delay or multi-step catch-up
  falsifies the corresponding clause. Fresh map/factory placement within the usable area
  is the initially uncloaked control. Outside-area placement with the own flag is the
  forced-full-cloak control, not a second ordinary transport exit.
- **H — Disguise kinds and no house (claims 35 and 38; Q-0728).** Have fixture buildings and
  aircraft use a disguise-making weapon, while observing them through a read-only recorder
  rather than relying on unsupported disguise art. Compare owner, ally and enemy targeters,
  with and without detector coverage. Expected: the disguised flag makes the game-state
  predicate true for every house, while independent targeting restrictions still apply.
  For radar, give a fixture vehicle disguise capability, the disguise-when-still flag and
  a nonempty terrain-disguise list. Let it acquire a no-house disguise while still, then
  observe it from a hostile house on explored ground. Compare its pixel with the neutral
  fallback colour, and change the imported fallback colour definition in a separate fixture run. Expected: its no-house radar mark
  follows that colour, not the owner, and remains present. Either another colour or absence
  solely because the disguise house is none falsifies claim 38. For no-house infantry,
  no ordinary creation route is established: if a sanctioned fixture can supply that state,
  walk an armed enemy past it. Expected: S-0209 claim 4 row WP25 applies when the rest of
  claim 35 holds; needing a nonempty disguise house falsifies this clause. Until such a
  fixture is available, this last case remains unmeasured, not a completed oracle test.
- **I — Restart reach and collapse (claim 20; Q-0729).** Put idle powered generators at cell
  displacements strictly inside, exactly on and strictly outside Δ*X*² + Δ*Y*² = 4(*r* + 2)².
  Include different foundation sizes, unequal heights and another owner. Shrink a source
  and record whether its own centre interrupts the zero-radius update. Repeat immediate
  sale, destruction and capture, which bypass that interruption. Record each neighbour's radius and growth state. Expected: only strictly
  inside neighbours restart, independently of owner, height and sub-cell centre distance.
  Immediate collapse also restarts them, including old-field collapse on capture. A normal
  shrink interrupted at zero is the no-restart control from part D. A boundary restart,
  a foundation-lepton-distance decision or no restart after an uninterrupted immediate
  collapse falsifies the corresponding rule.
- **J — Other sensor locomotors (claims 13 and 22; Q-0791).** Give CS-fixture walking
  infantry, a hovering vehicle and an ordinary flying aircraft the same positive sensor
  range, then repeat with only the sensors flag and zero range. Keep stationary cloaked
  witnesses at the old-only and new-only disc edges, outside adjacency distance. Record
  position, standing-position arrivals, heading-point choices, boundary crossings and
  sensed counts each tick. Expected: walking transfers coverage only at standing-position
  arrival. Hover transfers it at S-0503 claim 15's updates at its pre-travel position,
  including the initial clear choice and the listed non-refusing no-point cases, not at
  boundary crossing or necessarily exact-centre arrival. The aircraft leaves coverage at
  its placement cell through take-off, flight and landing. Range alone supplies the disc,
  whereas the flag alone never does. A differing relocation tick or moving aircraft disc falsifies
  the corresponding rule. Repeat with fully cloaked movers and stationary hostile sensor
  candidates, no weapons or other reveal causes. Give the aircraft a cloaking veterancy
  ability and cloak it while grounded before take-off. For the hover, cloak it while
  switched off and grounded, with that ability, then restore lift. Record uncloak state
  and sound: walking
  and hovering test adjacency on their listed updates, even in the same cell; ordinary
  flying movement produces no adjacency uncloak. A sensor approaching a stationary
  cloaked witness remains the coverage-only control. Compare both senses of approach so
  sensed visibility is not mistaken for a global uncloak.

### CS19: Generator eligibility and own-centre interruption (claim 20; Q-0792)

Use CS-fixture and the common three-run skirmish recording setup. Place a single powered
positive-radius generator on previously uncovered open ground, with no other buildings in
its first circle. Record every covered cell after the first growth update. Expect exactly
(0, −1), (−1, 0), (0, 0), including its own centre, with relative coordinates in cells.
A full circle with the interruption quirk on falsifies own-centre interruption. Repeat
with type radius 1 and wait until idle: the two omitted cells still lack coverage. With
larger radius, later circles may fill the omissions. Repeat on our engine with the quirk off:
the first circle is full.

For restart tests use immediate sale of a nearby source generator, avoiding an interrupted
ordinary shrink. Keep three neighbours in range: one idle at full radius, one growing below
full radius, and one just restarted at radius 0 by an earlier source collapse. Record their
field states and radii immediately before and after sale, distinguishing their own growth
turns from the restart effect. Expect only the idle neighbour to restart. A growing neighbour,
including one at its target radius whose next update has not yet made it idle, must not
restart. Repeat with a shrinking neighbour and with a neighbour not in operation: neither
qualifies. Any contrary immediate radius/state change falsifies the corresponding row.
For an in-operation idle radius-0 neighbour, use a zero-radius generator as a reachable
control: it can become growing after restart but produces no field, and its next growth
update becomes idle again. If an in-operation idle below-full positive-radius state becomes
available, repeat with it and expect a decrement floored at 0 followed by growth, not
exclusion by radius alone. This last state-construction test is conditional, not a completed
measurement. The eligibility test is field state plus operation, not radius.

### CS20: Other arrivals and placements (claims 13 and 22; Q-0795)

Use CS-fixture and the common three-run skirmish setup. Give each tested mobile type the
same positive sensor range. In separate runs give it a cloaking veterancy ability and cloak
it on the ground before movement. Remove weapons, field changes and damage as other reveal
causes. Keep stationary cloaked witnesses at old-only and new-only disc edges, beyond
adjacency, and an enemy type-flag sensor in a neighbouring destination cell. Record coverage,
position, cloak state and sound every tick, with event-resolution recordings where an order
inside one tick is being tested. Coverage lets one house see a still-cloaked witness, whereas
adjacency reveal changes the mover's cloak state globally. Do not confuse those outcomes.

- **Own teleport.** Use the teleporting locomotor on a vehicle, then on fixture infantry and
  aircraft. Record reservation, jump, phasing-in completion and destination removal. Expect
  disc transfer and adjacency reveal in the jump tick at the landing cell, not reservation,
  intervening cells or phase completion. Add a landing-cell entered-by action whose visible
  effect is recorded, and a crate in a separate run. Expect the sensor effects before the
  tag action and crate pickup. For a vehicle use S-0207 claim 22's arrival-report/crushing
  controls: their effects precede disc transfer. A consumed infantry entrant is not a
  surviving-arrival test. A different tick or reversed resolvable order falsifies the claim, but
  end-of-tick coincidence alone leaves the within-tick order inconclusive.
- **A jumpjet.** Fly across several cells, pause hovering at the destination, then land. Expect
  coverage to stay at its initial cell during flight and hovering and transfer on completed
  touchdown, with the adjacency test then, not at cruising arrival. Record landing placement,
  sensor changes, sight refresh and crate pickup to test their stated order. Repeat landing
  in the same cell: the adjacency test still occurs. A transfer in flight, missed touchdown
  transfer or in-flight adjacency uncloak falsifies the corresponding rule. Takeoff requires
  a compatible infantry or vehicle fixture; an ordinary flying aircraft is not this test.
- **Rocket.** Give a spawned rocket type a sensor range and observe from placement until
  impact, with edge witnesses away from its impact damage. Expect no flight transfer or
  adjacency reveal. The disc remains at placement until removal, whose current-cell
  subtraction can leave leaked coverage under the existing removal-centre quirk. A flight
  transfer falsifies the rule. A no-adjacency-reveal test requires a rocket already fully
  cloaked before flight; no such fixture is established, so that sub-test remains conditional
  and unmeasured, not confirmed merely by an uncloaked rocket staying uncloaked. Ordinary
  steering projectiles are a separate no-disc control, not rocket aircraft with their sensors
  flag cleared.
- **Superweapon warp.** Warp a living sensor vehicle to a clean landing, then repeat with a
  forced redirected landing. Record old coverage at departure, off-map retry updates,
  destination appearance and the later arrival-effects update of S-0513 claim 39. Expect
  no disc removal at departure and no addition at mere appearance; transfer and adjacency
  reveal occur at the surviving arrival-effects update, after the fate check. Compare a
  long arrival hold: transfer must not wait for its expiry. Use an arrival that fails the
  fate check as a no-arrival-update control. A different transfer tick falsifies the rule.
- **Release and unloading.** Board a sensor vehicle or infantry unit, move the transport,
  and record successful ordinary unloading before any passenger locomotor update. Expect
  coverage at exit placement immediately, but no adjacency uncloak solely from placement.
  Record its later ordinary arrival separately. Repeat with a carrier putting down a sensor
  vehicle if a data-only carrier fixture can be made operational; without one this carrier
  sub-test remains conditional. Drop sensor infantry with a parachute: expect coverage at
  release, none of the flight crossings as relocation events, and a same-cell arrival update
  and possible adjacency uncloak when falling ends at height 0. Observe the parachute-end
  request after that event. Contrary timing or a placement-only uncloak falsifies the rule.
- **Unused movement modes, conditional.** If a sanctioned data-only skirmish fixture supplies
  underground, mechanical-walking or drop-pod movement, repeat the disc-edge and adjacency
  controls. Expect underground transfer only after emergence completes, mechanical walking
  at movement-point snap, and no drop-pod arrival test solely from touchdown; the latter's
  landing placement uses removal/re-entry coverage instead. Log failure to construct such
  a fixture as unavailable, not confirmation. These rows remain unmeasured until then.

### CS21: Generator phase requests (claim 20; Q-0796)

Use CS-fixture, the common recording setup and three deterministic repeats. Record operation,
phase, radius and covered cells on every building update, not just the visible cloak fade.
Use a powered positive-radius generator and separately a zero-type-radius one.

- **Initial low power.** Place the generator out of operation through insufficient power,
  without issuing a manual power-off order. Expect idle at radius 0 indefinitely. Restore
  power: the first turn detecting operation makes growth pending and can grow radius 1
  that turn. Growing or shrinking before that transition falsifies the initial-state rule.
- **Partial shrink reversal.** Let growth reach full radius and become idle. Cut supply,
  allow two shrink steps while the current radius remains positive, then restore supply.
  Expect growth from that current radius in the first turn detecting restored operation,
  without waiting for radius 0. Collapse to 0 despite that qualifying transition falsifies
  the reversal. Sell another nearby source during the restored generator's pending growth:
  it must not restart, because growing is not idle.
- **Zero-radius pending cases.** Arrange a neighbour collapse after this generator's turn
  to restart it at radius 0, then cut supply before its next turn. Expect its pending growth
  to survive the detected loss of operation and advance one radius. Contrast the initially
  idle unpowered radius-0 placement, which stays idle. A forced shrink or lost growth at
  this zero-radius operation transition falsifies the table.
- **No automatic recovery.** On a full idle generator issue accepted manual power-off and
  power-on orders without an intervening building turn. Expect shrinking to remain pending
  despite operation being restored before a field step; unchanged operation does not ask
  for growth. After collapse and its stop update the generator remains idle at radius 0
  while in operation, until a further operation transition or neighbour restart. Automatic
  growth on the next unchanged-operation turn falsifies this rule. Verify both orders were
  accepted; an intervening turn or rejected order makes this trial inconclusive. A fixture
  making an idle positive radius less than a changed type radius is not established: if
  sanctioned runtime type changes become available, confirm unchanged-operation updates
  likewise leave it idle. That positive-radius extension remains unmeasured.
- **Manual versus supply loss.** At idle radius 0 with positive type radius, issue an accepted
  manual power-off order. Expect shrinking pending with radius set to the type radius,
  without adding coverage. Compare supply loss alone, which preserves idle at radius 0.
  Already existing overlapping coverage may be cleared by the ensuing shrink. A different
  immediate phase/radius falsifies the manual rule. The quirk-off control is on our engine
  only and never invents a positive radius for the empty field.
- **Immediate collapse at type radius 0.** Place full idle operational neighbours at cell
  displacements of 3 and 4 cells from a zero-radius generator. Sell, destroy, remove and
  capture that source in separate runs. Expect the 3-cell neighbour to restart, but not
  the 4-cell neighbour: the strict squared reach is less than 16 square cells. The source
  has covered nothing. Repeat ordinary shrinking already at radius 0 as a no-restart
  control. No inside restart on immediate collapse, a boundary restart, or a restart from
  the ordinary zero-radius stop update falsifies the corresponding rule.

### CS22: Edge-case controls

Use CS-fixture and the common map, recording setup and three repeats.

- **(a) Empty disguise list, engine only.** Supply an empty terrain-disguise list and an
  otherwise eligible still vehicle that can disguise when still. Observe importer warnings,
  disguise state and random-stream draws for 256 ticks with no adjacent enemies or contact.
  Expected: a warning, no acquired disguise and no disguise-selection draw. A disguise,
  selection draw or missing warning falsifies the safe import rule. This does not test or
  require the original's invalid selection outcome.
- **(b) Missing infantry disguise type.** In CS-fixture, create permanent-disguise infantry
  for a house whose side default is absent. Supply two distinct usable infantry defaults
  for the other two sides, naming them **fallback A** and **fallback B**, with recognisably
  different images. Transfer the subject by mind control to a house of one of those sides,
  without clearing or reacquiring its disguise, and observe from a local player of the other
  side. Record the disguised flag, absent held disguise type, current owner and both side
  defaults, together with the resulting drawn type. Keep detector coverage, fake blink and
  other image effects absent. Repeat with fallback A and fallback B swapped between those
  sides, three runs for each arrangement under the common setup. Expected in the original:
  the missing held type resolves to the local player's default in each arrangement. On our
  engine it resolves to the current owner's default instead. A different resolved image
  with the required missing type and distinct defaults falsifies the corresponding fallback
  rule. A transfer that fills or clears the disguise type makes the run inconclusive, not
  confirmation. This test neither writes game state nor assumes a missing owner default
  invents a usable type.
- **(c) Undrawn initial uncloak stage.** In CS-fixture set the invented stage count to
  *N* = 512 stages and speed to *S* = 16 ticks per stage. Place a full-health self-cloaking
  infantry subject and a hostile ordinary attacker on explored open ground, within legal
  weapon range but outside sensor adjacency. Give the attacker's house removable sensor
  coverage and retain its ordinary attack order through the subject's cloak start and
  completion. Hold the attacker rearming until the observation interval, then force the
  subject to uncloak with a surviving accepted hit. Remove coverage after uncloaking
  starts and let the otherwise-ready attacker make its due firing attempt before the
  first fading-stage step. Record cloak state, stage, coverage, retained target, the
  non-owner image and shot creation with its target and resulting health change. Repeat
  three times, with matched fully uncloaked and fully cloaked unsensed controls. Expected:
  the initial uncloaking stage is 511, with fade fraction 255; the unsensing non-owner sees
  no image, yet its retained ordinary attack can fire. The uncloaked control also permits
  firing; the fully cloaked unsensed control refuses it. An image at that initial stage,
  or a cloak-only firing refusal while the subject is uncloaking and all other conditions
  are met, falsifies the corresponding rule. Lost targeting, an expired observation stage
  or any independent firing restriction makes the run inconclusive.

## Open questions

- Arithmetic overflow of sensor counts or inputs beyond the signed import domains is not
  specified here. The ordinary signed 8-bit cloak-radius import and nonpositive fade inputs
  are covered by claims 20 and 43.
- No oracle measurements have yet been performed for CS18. The claims are source-backed,
  not measured; the scenarios distinguish the identified timing, tie and coverage cases.
