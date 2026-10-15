---
id: S-0506
title: Missiles (spawned missiles, launch cycle and rocket flight)
topic: combat
revision: 2
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0506: Missiles (spawned missiles, launch cycle and rocket flight)

## Summary

How an object that keeps spawned objects (a missile launcher, or a carrier of aircraft) holds,
launches, regenerates and reloads them, and how a *rocket missile* (a spawned missile flying under
the rocket-flight movement mode defined here) flies to its aim point and detonates. It covers the launcher's lockouts while
it launches, what happens to the spawned objects when the launcher dies or changes hands, and the
hand-off of the detonation to area damage (S-0303). This revision covers skirmish; network games
and campaigns are out of scope.

Boundaries:
- Ordinary projectiles, including the steering projectiles that S-0306 calls missiles, are S-0306.
  A rocket missile is not a projectile: it is an aircraft-kind object with health, created by its
  launcher's spawn controller, and its launcher's weapon launches no projectile (S-0301, Terms
  *spawner weapon*).
- The flight of spawned aircraft that are not missiles (a carrier's aircraft in the preparing,
  attacking and returning slot states of claim 4) belongs to S-0501. The spawn controller, its slots
  and their cycle (claims 1 to 9 and 11 to 13) are defined here and hold for carriers too.
- The nuclear missile superweapon is S-0513. Its missile is not a spawned object and nothing in
  this spec applies to it.

## Terms

Glossary terms are used as defined there, in particular *aircraft*, *facing*, *step direction*,
*target*, *destination*, *on the map*, *off the map*, *in play*, *removed*, *tracked flying object*
(S-0303), *firing check* and *answer* (S-0301), *movable* (S-0214), *veterancy rank* and *height
above the terrain*.

- **Spawn controller.** The part of an object (the *spawner*) whose type names a spawned type and
  a positive spawn count. It holds a fixed number of *spawn slots*, and is created with the spawner.
- **Spawned type, spawn count.** The aircraft type that the spawner's type names as the type it
  spawns, and the number of slots. Both are imported (Import mapping).
- **Slot index.** The stable launch-position number, from 0 to *N* − 1, also identifying the
  alternating muzzle side in claim 7. Indices remain the same across replacement objects.
- **Missile type.** A type with the missile-spawn flag (S-0311, Terms *missile spawn*). In this spec
  a *missile slot* is a slot whose spawned type is one of the three *rocket types*: the types named
  by the game-wide rocket settings of parameter sets A, B and C (Import mapping).
- **Spawn slot, slot object, slot state.** Each slot holds at most one *slot object* (an object of
  the spawned type) and is in one of the states of claim 4: *docked*, *on the rail*, *preparing*,
  *attacking*, *returning*, *reloading*, *empty*. A slot object is *alive* while its slot is not
  empty.
- **Controller state.** One of *idle*, *launching*, *cooling down* (claim 5).
- **Controller target, pending target.** The controller's own record of what its slots are sent
  against, and the target handed to it by the last shot that has not been adopted yet (claim 3).
  Neither is the spawner's own target.
- **Launch pass.** One run of the spawn controller (claim 2): every 10 ticks.
- **Launch spacing.** The minimum time between two launches of the same controller (claim 6).
- **Launching count.** The number of slots on the rail, plus the number of preparing slots whose
  object is of a missile type and is on the map. The spawner is *launching* while this count is
  above 0 (S-0301, Terms *launching*).
- **Kamikaze.** An aircraft enrolled for the repeated attack orders of claim 12. This is the
  release marker used by the slot-loss test (claim 9), including for a slot aircraft in S-0501
  claim 36; it does not by itself mean that the aircraft is dead.
- **Released missile.** A missile that is a kamikaze (claims 8 and 12).
  This does not start its flight or change an already fixed aim point. While its slot is on the
  rail it can still vanish with the spawner (claim 13).
- **Rocket-flight movement mode.** Movement governed by the phases, speed, pitch and fixed aim
  point of claims 14 to 21, rather than ordinary aircraft flight.
- **Rocket missile.** An aircraft-kind object of a rocket type in the rocket-flight movement
  mode (claims 14 to 21). Each rocket missile uses one *parameter set*: set A when its type is
  the set-A rocket type, set C when it is the set-C rocket type, set B otherwise.
- **Aim point.** The fixed world point a rocket missile flies to (claim 14).
- **Flight phase.** One of *pause*, *tilt*, *raise*, *ascent*, *cruise*, *descent* (claim 15).
- **Pitch** *p*. The nose angle of a rocket missile above the horizontal, in radians, single
  precision; positive is nose up.
- **Rocket speed** *v*. The rocket missile's current speed in leptons per tick, double precision.
- **Elite launch.** A rocket missile whose spawner exists and has the elite veterancy rank (claim 19).

## Behaviour

### The spawn controller and its slots

1. **Creation and initial fill.** An object whose type names a spawned type and a spawn count *N*
   ≥ 1 gets a spawn controller with *N* slots when it is created. Each slot at once receives a new
   slot object of the spawned type, owned by the spawner's house, off the map and docked. A spawner
   therefore starts with its full count; no regeneration time is spent on the first fill.
   - Backing: `measurable` (scenario R1)
   - Confidence: high

2. **Launch passes.** The controller acts only in launch passes. The first pass comes 20 ticks
   after the controller is created; each later pass comes 10 ticks after the previous one. Between
   passes no slot or controller state changes, except for pending-target changes of claim 3 and the events of claims 9 and 11 to 13, which act at
   once. Every slot timer of this spec (spacing, rail, regeneration, reload) is a countdown in ticks
   but is only looked at in a pass, so a change waits for the first pass at or after its timer's end.
   - Backing: `measurable` (scenario R2)
   - Confidence: high

3. **Shots hand over a pending target.** A shot of a spawner weapon (S-0301, Terms) launches no
   projectile. It sets the controller's pending target to the shot's target, unless that target is
   already the controller target, in which case nothing changes. Clearing the spawner's own target
   clears the pending target too. A nonempty pending target replaces the controller target in a
   pass when the controller is idle, when a rocket slot launches, or when a non-missile slot is
   preparing, attacking or returning. A later shot can therefore redirect launches not yet made,
   but cannot redirect a rocket whose aim point is already fixed (claim 14).
   - Backing: `measurable` (scenario R2)
   - Confidence: high

4. **Slot states.** The Slot table under State and ordering defines the transitions visible at
   launch passes. When several docked slots are eligible, only the lowest-index eligible slot
   launches: its launch spacing holds later slots back. In short: a docked slot launches its object when the launch conditions of
   claim 6 hold; a missile slot's object then waits as *preparing* until claim 8 releases it, after
   which a flagged rocket's slot is *on the rail* until the rail time ends and then *empty*; an empty slot gets a
   new object when its regeneration time ends. A non-missile slot goes from preparing to attacking,
   returning and reloading, and back to docked (the flight itself belongs to S-0501).
   - Backing: `measurable` (scenarios R2, R3, R5 and R14)
   - Confidence: high

5. **Controller states.** The Controller table under State and ordering defines the controller's
   transitions and their conditions. A pass that supplies the final preparing object can release
   the salvo in that same pass, with no extra pass of waiting:
   - *idle*: a pending target becomes the controller target (and the pending target is cleared). If
     there is a controller target the controller then checks whether the spawner could attack it
     from where it stands (the weapon range test of S-0307 for the spawner weapon). If not, the
     controller *resets* (claim 11); otherwise it becomes *launching*.
   - *launching*: without a controller target it resets. It does not repeat the range test.
     Otherwise it waits until every slot is preparing or empty. Then it releases (claim 8)
     and becomes *cooling down*.
   - *cooling down*: once no slot is attacking or returning, it becomes *idle*.

   For an initially idle controller with no target, the first launch comes one pass after the
   shot's target is adopted: with claim 6's conditions satisfied, a shot between two passes is
   adopted in the next pass and its first object launches in the pass after. This is not an extra
   delay for pending-target changes during an existing launch cycle (claim 3).
   - Backing: `measurable` (scenario R2)
   - Confidence: high

6. **Launch conditions and launch spacing.** A docked slot launches its object in a pass when all
   of these hold:
   - the controller has a controller target,
   - the launch spacing has run out: at least *s* ticks have passed since this controller's last
     launch, where *s* = 9 when the **spawner's own type** has the missile-spawn flag and *s* = 20
     otherwise (`quirks.launch_spacing_from_spawner_type`),
   - the controller is not cooling down,
   - the spawner, when it is a mobile unit, is not restrained by a magnetic beam (distinct from
     the parasite hold of S-0614 claim 8),
   - for a missile slot only: the spawner's locomotor reports that it is not moving.

   With passes every 10 ticks a spacing of 9 allows one launch per pass and a spacing of 20 one
   launch every second pass. The order of launches is slot order: the first docked slot that meets
   the conditions launches, and the spacing then holds the rest back.
   - Backing: `measurable` (scenario R3)
   - Confidence: high

7. **Where and how an object is launched.** A launched slot object is placed on the map at the
   spawner's fire point (S-0313, claim 1) for the spawner's weapon slot that holds the spawner
   weapon (the primary slot if it does, the secondary otherwise), raised by 10 leptons. When the
   primary weapon's burst is above 1, a missile slot uses the burst alternation of S-0313 (claim 2)
   as if this were shot number *k* mod 2 of a burst, *k* being the slot's index (0 for the first
   slot): even slots take the first-shot side and odd slots the other side. An object of the set-C
   rocket type is placed 40 leptons further towards map north on the map *y* axis and 40 leptons
   further towards map west on the map *x* axis, and a take-off animation (cosmetic) plays at the
   placement point. This set-specific placement is ordinary launch geometry, not a defect or
   quirk: the displaced position is the origin of the actual flight, not merely its drawing.
   The object's facing is the spawner's body facing rounded to 256 steps
   (Formulas F2). A missile slot's object is then given the controller target as its destination
   and the move behaviour; a non-missile slot's object is sent to the neighbouring cell of the
   spawner's cell in step direction 0 (S-0501 owns the rest). The slot becomes preparing.
   - Backing: `measurable` (scenario R4)
   - Confidence: high for the fire-point choice, the 10-lepton rise and the facing rounding; medium
     for the burst side assignment and the set-C shift (signs as stated)

8. **Release.** When a launching controller finds every slot preparing or empty, all preparing
   slots are released in that same pass:
   - an object without the missile-spawn flag is set on the controller target and becomes attacking,
   - an object with that flag is released with the controller target (claim 12). For a rocket slot,
     the slot goes *on the rail* for the pause time plus the tilt time of the rocket parameter set
     (Import mapping): set A for the set-A rocket type, set B otherwise
     (`quirks.raised_missile_rail_time_from_set_b`). A flagged object outside the three rocket
     types is detached at once and its slot starts regenerating.

   The release decision uses the object's flag, not merely whether its type is one of the rocket
   types. Stock rocket types have the flag; a named rocket type with the flag absent would instead
   enter attacking.

   If at least one missile was released, the controller then resets (claim 11; there are no
   preparing missiles left, so the reset only clears the controller target and state). It then
   becomes cooling down. All the missiles of a salvo are therefore released in the same pass even
   though they were launched (placed on the map) one spacing apart. Release is not the start of
   flight: each rocket begins its own flight cycle on its first move order after placement,
   independently of when the other slots launch.
   - Backing: `measurable` (scenarios R2, R3)
   - Confidence: high

9. **Rail time, regeneration and reload.** When a slot's rail time ends, the slot becomes empty
   with a regeneration countdown of the spawner type's regeneration time. The released missile
   flies on; it no longer belongs to any slot. When an empty slot's regeneration countdown has run
   out, the slot gets a new slot object of the spawned type, owned by the spawner's **current**
   house, off the map and docked. A reloading slot becomes docked when its reload countdown (the
   spawner type's reload time) has run out; its object's ammunition is set to its type's ammunition
   and its health to its type's strength. When any slot object, including an ordinary carrier
   aircraft, is announced as lost to the controller, the slot becomes empty with the spawner's
   regeneration countdown if its health is below 1, it is a kamikaze, or it is of a rocket type.
   This can happen at crash start, before final removal. An ordinary slot aircraft removed from
   play while still alive and not a kamikaze leaves its slot unchanged (S-0501 claim 36).
   - Backing: `measurable` (scenarios R5 and R14) except ignoring a loss notification for a
     living, non-kamikaze non-rocket object: `unmeasured`, because no normal-input scenario has
     been established that produces this notification while the object remains living and linked.
   - Confidence: high

10. **Lockouts while launching.**
    - While the spawner's launching count is above 0, its firing check answers *busy* when it is a
      vehicle (S-0301, claim 9, rank 2).
    - A spawner weapon none of whose slot objects is alive gets *rearming* (S-0301, claim 9, rank 4).
    - A mobile spawner under the parasite hold of S-0614 claim 8 gets *impossible* for its
      spawner weapon, regardless of whether its type is organic (S-0301 claim 9, rank 4).
      This is distinct from magnetic restraint in claim 6 and can cause the reset of claim 11
      when a firing attempt is made.
    - While the launching count is above 0, a spawner that moves with the driving or ship
      locomotor cannot start another track. An existing track finishes; between tracks the
      spawner waits with its destination kept until launching ends (S-0504 claim 3).
    - While the launching count is above 0 and below the spawn count *N*, the spawner is not movable
      (S-0214, Terms *movable*).
      With a single slot the count never lies strictly between 0 and *N*, so a spawner with a
      spawn count of 1 stays movable while it launches; the driving and ship lockout above still
      prevents it from starting another track.
    - Backing: `measurable` (scenario R6)
    - Confidence: high

11. **Reset.** The controller *resets* when it finds itself in the idle or launching state with a
    target it cannot attack from where it stands (idle only), or without a target (launching only,
    an idle controller without a target simply waits), when the spawner's
    firing check answers *impossible* to an attempt with the spawner weapon, when the spawner is hit
    by an electromagnetic pulse that disables it, and when the controller target leaves play and no
    pending target is waiting. A reset releases every preparing object of a missile type with the current
    controller target (claim 12), makes each such slot empty with a regeneration countdown, clears
    the controller target and the pending target, and makes the controller idle. A non-missile
    preparing object is not affected by a reset.
    - Backing: `measurable` (scenario R7) for its named events and reset effects.
      `unmeasured` for an impossible firing answer as an independent reset cause: no scenario
      here separates that answer's reset from the disabling event's own reset.
    - Confidence: high for the reset's effect; medium for the complete list of events that cause it

12. **What a release fixes.** A released missile, from claim 8 or 11 (or the spawner events of
    claim 13), gets a *release cell*: the cell under the target's position when there is a target,
    and otherwise the neighbouring cell of the missile's own cell in the step direction nearest its
    body facing (Formulas F8). From then on the game re-orders it every 30 ticks to attack the
    release cell; these orders do not change its flight, because a rocket missile's aim point is
    fixed by its first move order (claim 14) and never changes afterwards. The 30-tick interval
    belongs to a shared re-order cycle, not a fresh per-missile delay; normal salvo release or reset
    brings its next update forward to 2 ticks later. A released object of a type without the
    missile-spawn flag instead starts to crash at once (S-0312, claim 16).
    - Backing: `measurable` for fixed aim and non-missile crash (R7, R14); `unmeasured` for
      release-cell selection and repeated orders: a rocket ignores them after its aim is fixed,
      and no setup separating their effects has been established.
    - Confidence: high

13. **When the spawner dies, changes hands or loses its locomotor.** The controller *clears its
    slots* when the spawner is destroyed or starts to crash (unless it is held by a magnetic beam
    at that moment), when the spawner changes house (mind control, capture), when a locomotor
    warhead replaces the spawner's locomotor, when the spawner is hit by an erasing weapon, and when
    the spawner is removed. Any crash-start draws for released aircraft occur in descending slot index, so earlier
    draw values belong to higher-index occupied slots. Its effects per slot are:

    | Slot state | Effect on the slot object |
    |---|---|
    | docked, reloading | removed silently |
    | on the rail | removed silently: a missile whose slot rail time has not ended vanishes without a detonation, regardless of its own flight phase (`quirks.rail_missile_vanishes_with_spawner`) |
    | preparing, attacking, returning | released (claim 12) with the controller target: a missile flies on to its aim point; a non-missile aircraft starts to crash |
    | empty | (nothing) |

    Each occupied slot then becomes empty. Its regeneration countdown is the spawner type's
    regeneration time when the spawner has health below 1 or is no longer in play, and 0 otherwise.
    Slots already empty keep their existing countdowns. After a house change, replacements for
    previously occupied slots therefore appear for the new house at the next pass; a slot already
    empty can still be waiting for its old regeneration countdown.
    - Backing: `measurable` (scenario R8)
    - Confidence: high for the table; medium for the list of events

### Rocket flight

14. **The aim point.** A rocket missile's aim point is fixed when it first carries out its move
    behaviour, using the destination given in claim 7. The target's heading point is sampled then,
    not necessarily in the placement tick. Later move orders are ignored. The aim point is the destination's *heading point*:
    - for a unit: the point its locomotor is moving it to at that moment (the end of the step it is
      on), or its position when it is not moving; for a unit inside a tunnel, the tunnel's exit,
    - for a building: its centre, the point 128 × (*W* − 1) leptons further on the map *x* axis and
      128 × (*H* − 1) further on the map *y* axis than its position, *W* × *H* being its foundation,
      except that a building whose type is a helipad, a unit repair or reload building, or a bunker
      gives its docking point,
    - for a cell: the cell's centre point (S-0204).

    A missile fired at a moving unit therefore flies to where the unit was heading when its first
    move behaviour supplied the aim point, not to where it is when the missile arrives
    (`quirks.rocket_aim_at_heading_point`; off: sample the unit's position at that same moment).
    - Backing: `measurable` (scenario R9)
    - Confidence: high for the fixing; medium for the building docking cases and the tunnel case

15. **Flight phases.** A rocket missile goes through these phases, by the Rocket table under State
    and ordering:
    - *pause* (skipped when the set's pause time is 0): it stands still for the pause time,
    - *tilt* (sets A and B, and zero-pause set C) or *raise* (set C after a pause), for the tilt time: in tilt the pitch turns from the
      set's initial pitch to its final pitch (Formulas F3), in raise the missile climbs by the set's
      raise rate in leptons per tick while its pitch stays at the initial pitch,
    - at the end of tilt or raise: its pitch is set to the final pitch, it becomes a tracked
      flying object (S-0303, Terms) and its take-off sound plays (and, after tilt, a launch
      animation, cosmetic). It then enters *ascent*,
    - *ascent*: it accelerates (claim 16) along its pitch until its height above the terrain is at
      least the set's cruise altitude, then enters *cruise*,
    - *cruise*: it levels out or follows the lazy curve (claim 17) towards the aim point; a missile
      not on the lazy curve enters *descent* by claim 17,
    - *descent*: it steers its nose to the aim point (claim 17) until it detonates.
    A rocket missile is not a tracked flying object, and cannot be found by the flying-object search
    of S-0303 claim 6, before the end of its tilt or raise.
    - Backing: `measurable` (scenario R10)
    - Confidence: high

16. **Speed.** The rocket speed is 0 in pause, tilt and raise. In ascent and cruise it grows by the
    set's acceleration every tick, up to the type speed *V* of the missile's type (S-0207, claim 1,
    Formulas F1 there); it is not reduced in descent (Formulas F4). The speed field's conversion is
    the only one: no speed multiplier of the house or the terrain applies.
    - Backing: `measurable` (scenario R10)
    - Confidence: high

17. **Steering.**
    - **Heading.** In cruise the missile's body turns towards the direction (S-0204, claim 8) from
      its position to the aim point, at its type's turn rate (S-0206). In the other phases no new
      direction is set, but a turn already under way when cruise ends goes on to completion, and the
      step of Formulas F1 always uses the body facing of that tick.
    - **Plain cruise** (the set's lazy-curve flag off, or a recorded start distance of 0): the pitch
      falls by the set's turn rate each tick until it is 0 (Formulas F5). The missile enters
      descent in the first cruise tick in which its horizontal distance to the aim point (the table
      square root of S-0204, truncated) is no greater than its world height minus the aim point's
      world height: once the aim point lies 45 degrees or more below the horizontal as seen from the
      missile. A plain-cruise missile so flies level until it is about as far from its aim point as
      it is high above it, and then turns its nose down onto it.
    - **Lazy curve** (flag on): the pitch blends the line-of-sight angle to the aim point with the
      final pitch, weighted by how much of the horizontal distance recorded at the start of cruise
      is left (Formulas F6). The missile so curves from steep ascent down onto the aim point, and
      never enters descent.
    - **Descent:** the pitch turns towards the line-of-sight angle to the aim point by at most the
      set's turn rate per tick (Formulas F7).
    - Backing: `measurable` (scenario R10) for the flight paths in its tested distance range.
      `unmeasured` for integer squared-distance overflow in F6 and F7: R10 does not isolate
      distances large enough to wrap the sum, and no discriminating path is established here.
    - Confidence: high for the lazy curve and the descent steering; medium for the plain-cruise
      descent test (the original computes a spatial distance whose height difference is always 0,
      which is why the test is horizontal) and for the exact numbers of Formulas F5 to F7

18. **Movement and the edge of the map.** In every tick with a rocket speed above 0 the missile
    moves by Formulas F1. It moves only when the cell of its new position is a legal cell; if it is
    not, the missile keeps its position for that tick (and keeps its speed and pitch).
    - Backing: `measurable` (scenario R10, including its named map-edge variant)
    - Confidence: high for the step; medium for the edge case

19. **Detonation.** A rocket missile detonates in the first tick in which, before its move:
    - (cruise, any mode) its height above the terrain is below 1 lepton; or
    - (lazy cruise and descent) the world height its move would reach is at or below the aim point's
      world height; or the missile is over a cell with a high bridge, the aim point's world
      height equals that cell's terrain height, and the height its move would reach is at or below
      the aim point's world height plus the 416-lepton height of a bridge deck (glossary *bridge
      deck*); or its height above the terrain is
      below 1 lepton.

    It also detonates after a positive-speed movement attempt with health below 1 (claim 20),
    even if that attempt was rejected at the map edge. When it detonates it
    stops being a tracked flying object, then the detonation of S-0303 takes place at its *nose
    point* (Formulas F9) with:
    - damage amount: the set's damage, or the set's elite damage for an elite launch; the
      firepower multipliers, veterancy and the spawner weapon's own damage play no part (S-0302,
      claim 2 does not apply: there is no shot),
    - warhead: the set's warhead, or its elite warhead for an elite launch,
    - source object: the missile; source house: none; resource reduction allowed (S-0303).
    An explosion animation chosen as S-0303 gives for a detonation of that amount and warhead
    appears at the nose point. Its animation is selected and created before the area-damage hits,
    so any animation-creation random draws precede draws caused by those hits. The missile is
    removed after the area damage. A launch is elite when, in the last
    tick of its pause, tilt or raise (the rank is read in each of those ticks), the spawner exists
    and has the elite rank; the missile's own veterancy plays no part. Kill experience follows
    S-0311 claim 8: a trainable missile can keep its own experience instead of giving it to the spawner.
    - Backing: `measurable` (scenarios R11 and R12) except the independence from the missile's
      own veterancy: `unmeasured`, because no normal-input setup here independently promotes a
      still-linked missile before the rank-sampling cutoff. The absent source house is also
      `unmeasured`: R11 does not isolate a house-sensitive effect from the missile's owner.
    - Confidence: high for the amount, warhead and elite choice; medium for the bridge condition

20. **Interception.** A rocket missile is an aircraft-kind object with health; whether something
    can target it is S-0309's and S-0214's. When a hit destroys it:
    - above the terrain (height above the terrain 1 lepton or more): it shows its type's explosion
      list animation (S-0312, claim 16) and starts to crash, but it keeps the rocket-flight movement
      mode, and it detonates by claim 19, with its full damage and warhead, at the end of its
      next tick of movement. A missile destroyed while its speed is 0 (in pause, tilt or raise)
      has no health-triggered detonation until a positive-speed movement attempt
      (`quirks.destroyed_rocket_still_detonates`; with it off, a
      destroyed rocket missile is removed without a detonation),
    - on the ground (height below 1): it is removed at once, without a detonation.
    - Backing: `measurable` (scenario R13)
    - Confidence: medium for continued rocket flight; high for the health test in a positive-speed
      movement attempt and the ground removal. The exact post-kill trajectory remains untraced:
      this claim does not assert that the ordinary crash fall is absent.
    - Backing limitation: `unmeasured` for any additional crash displacement before that attempt,
      no complete trajectory trace is established (R13 is the proposed discriminating test).

21. **Cosmetic effects.** During positive-speed flight a trailer animation appears at the
    missile's current position every 3 ticks. A set-C missile also emits take-off animations in
    pause and raise, using a 24-tick interval. These intervals share a timer: transitions can
    therefore change the delay before the first animation of the next phase. Animation behaviour
    and its random draws follow S-0109.
    - Backing: `measurable` (scenario R15)
    - Confidence: high

## Formulas

Number domains: positions and recorded distances are integer leptons; table-root results before
truncation are floating point. Pitch is stored in single precision; rocket speed is double
precision. Pitch results in F5 to F7 are stored in single precision each tick. Every floating-point addition, subtraction, multiplication and division below forms a
53-bit result rounded toward zero, including operations on widened single operands; only an
explicit single store narrows to 24 bits, also toward zero. RZ₅₃ and RZ₂₄ denote those roundings.
Products associate left to right and parentheses fix grouping; operations are not fused.
S-0205 claims 4, 5 and 7 own the shared rounding-mode switch and conversion rules. Integer conversions below truncate
toward zero. Every sine and cosine below is the table trigonometry of S-0105 Formulas F1,
including its negative-angle quirk, not a mathematical-library sine or cosine. *T*, *σ* and the table square root *r* are those of S-0204 (Formulas F1
and F3). *trunc* truncates toward zero.

**F1. Rocket step** (claim 18). With *s* = trunc(*v*), *F* the body facing read as a signed 16-bit
value, θ = (*F* − 16383) × (−2π / 65536) (double precision):

> *h* = trunc(cos(*p*) × *s*)
>
> *x*′ = trunc(*x* + *h* × cos θ), *y*′ = trunc(*y* − *h* × sin θ), *z*′ = trunc(*z* + sin(*p*) × *s*)

So facing 0 moves the missile towards map north (decreasing *y*) and 16384 towards map east. The
constant is 16383, not 16384, which turns the direction by one facing unit. This is part of the
formula, with no switch.

**F2. Placement facing** (claim 7). From the spawner's body facing *F* (0 … 65535):

> *f* = ((⌊*F* / 128⌋ + 1) div 2) mod 256, placement facing = 256 × *f*

All operations are integer: div truncates toward zero and mod is the nonnegative remainder.
*f* counts steps of 256 facing units; the placement facing is in facing units.

**F3. Tilt pitch** (claim 15). With *e* the integer ticks elapsed in tilt, *T*ₜ the integer tilt
time, *P*ᵢ and *P*_f the imported single-precision pitches in quarter turns, let *α* and *β* be
the double-precision products *P*ᵢ × π/2 and *P*_f × π/2, respectively. The interpolation
fraction *u* is the double-precision quotient of *e* and *T*ₜ.

> *p* = *α* + (*β* − *α*) × *u*, stored in single precision; at the end *p* = *β*.

The initial pitch, and each endpoint pitch, is the corresponding double product with π/2
narrowed once to single precision. A zero tilt time takes the endpoint without division. Floating-point rounding follows S-0205
claims 4 and 5, with a single-precision store of the resulting pitch, not of *α*, *β* or *u*.

**F4. Speed** (claim 16). *v*ₙ₊₁ = min(RZ₅₃(*v*ₙ + *a*), *V*), with *a* the set's acceleration
(single precision, widened exactly), *v* double precision and *V* the integer type speed widened
exactly. The sum rounds toward zero to 53 bits; the comparison is exact and the chosen value is
stored as a double. Units: leptons per tick.

**F5. Plain cruise pitch.** With *p* and τ single-precision radians and radians per tick,
respectively, each cruise tick with *p* > 0 gives

> *p*′ = max(0, RZ₂₄(RZ₅₃(*p* − τ))).

Operands widen exactly; subtraction rounds to 53 bits, then the result narrows to 24 bits before
the exact comparison with zero. A nonpositive pitch is unchanged.

**F6. Lazy curve pitch.** Write *D* = aim position − missile position, in integer leptons.
Let *n*_d be the double sum of the two double squares of *D*ₓ and *D*ᵧ: each product and the sum
rounds toward zero to 53 bits. The recorded integer distance at cruise start is
*d*₀ = trunc(*r*(*n*_d)). For each later tick:

> *q* = RZ₅₃(trunc(*r*(*n*_d)) / *d*₀)
>
> *n*_i = wrap(wrap(*D*ₓ × *D*ₓ) + wrap(*D*ᵧ × *D*ᵧ)), *ρ* = *r*(*n*_i)
>
> *ℓ* = *A*(RZ₅₃(*D*𝓏 / *ρ*)) when *ρ* > 0, otherwise the double value −π/2
>
> *p*′ = RZ₂₄(RZ₅₃(RZ₅₃(*ℓ* × RZ₅₃(1 − *q*)) +
> RZ₅₃(RZ₅₃(*P*_f × *q*) × π/2))).

The two distances use different squared-distance domains: *n*_d is double precision, whereas
*n*_i is signed 32-bit integer arithmetic with wrapping as S-0205 claim 1 defines. *n*_i is widened
exactly before the table root. *q*, *ρ* and *ℓ* are doubles; integer operands and imported singles
widen exactly. There is no single-precision intermediate in the blend; only the final pitch
narrows. A recorded distance of zero uses F5, not this division. π/2 is the double constant in
these expressions. Units: distances in leptons, squared distances in square leptons, *q*
dimensionless, angles in radians.

*A* is the raw table arctangent. For a double argument *u*,
*i* = |trunc(RZ₅₃(*u* / *σ*))|. The sign comes from RZ₂₄(*u*).
For *i* ≤ 4096, the magnitude is the original single-precision *T*(*i*); for larger *i* it is the
single-precision π/2. The signed value widens exactly to a double. The narrowing for the sign
does not narrow the quotient used to select the index. S-0204 claim 8 describes the raw table.
Its corrections to integer facings are not corrections to these angles. Its surrogate angles
are not an exact substitute here.

**F7. Descent pitch.** With *ρ* and *ℓ* formed as in F6 using the integer squared-distance domain,
δ = RZ₅₃(*ℓ* − *p*). The old single pitch widens exactly and δ stays double precision through
the exact comparison |δ| ≤ τ. The new pitch is

> *p*′ = RZ₂₄(RZ₅₃(*p* + δ)) when |δ| ≤ τ
>
> *p*′ = RZ₂₄(RZ₅₃(*p* + τ × sign(δ))) otherwise.

The sign multiplication is exact; the addition rounds to 53 bits and only its result narrows
to 24 bits. Angles are radians, τ radians per tick. These formulas use the original rounding
mode; with S-0205 claim 5's shared rounding quirk off, each RZ is replaced by RN of the same
precision. Integer conversions still truncate.

**F8. Release cell without a target** (claim 12). With *F* the body facing, step direction
*k* = ((⌊*F* / 4096⌋ + 1) div 2) mod 8.
Integer arithmetic, div truncating toward zero and mod the nonnegative remainder. *F* is in
0 … 65535 facing units; *k* is a dimensionless step-direction index.

**F9. Nose point** (claim 19). With *L* the set's body length in integer leptons, θ as in F1:

> *b* = trunc(*L* × cos(*p*))
>
> nose = (trunc(*x* + *b* × cos θ), trunc(*y* − *b* × sin θ), trunc(*z* + *L* × sin(*p*)))

The horizontal length is truncated before projection; the final world coordinates are truncated
only after adding the missile's position. The trigonometry is floating point.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| First launch pass | 20 | ticks after controller creation | Claim 2 |
| Launch pass interval | 10 | ticks | Claim 2 |
| Launch spacing, missile-spawn spawner | 9 | ticks | Claim 6 |
| Launch spacing, other spawner | 20 | ticks | Claim 6 |
| Launch rise | 10 | leptons of world height | Claim 7 |
| Set-C placement shift | 40, 40 | leptons towards map west, map north | Claim 7 |
| Release re-order interval | 30 | ticks | Claim 12 |
| Return height margin (non-missile) | 20 | leptons | State table; S-0501 |
| Rocket trailer interval | 3 | ticks | Claim 21 |
| Set-C pause and raise animation interval | 24 | ticks | Claim 21 |
| Ground contact height | 1 | lepton | Claims 19, 20 |
| Rocket heading constant | 16383 | facing units | F1 |
| Rocket parameter defaults | see Import mapping | varies by parameter; see each field's unit in Import mapping | Absent-key defaults |

## State and ordering

**Slot table** (claim 4; ordinary transitions at launch passes, loss notifications immediately
as in claim 9). A transition to docked does not produce
an appearance until a later pass; becoming empty starts regeneration but cannot refill the slot
in that same pass, even with a zero regeneration time. These are timing constraints, not a
required order of internal processing.

| Slot state | Event in the pass | Result |
|---|---|---|
| any occupied slot | loss notification, object health below 1, kamikaze or rocket type | empty, regeneration countdown starts immediately (claim 9) |
| any occupied non-rocket slot | loss notification or removal from play, object alive and not a kamikaze | slot unchanged (claim 9) |
| docked | claim 6 conditions hold | launch (claim 7); spacing restarts; preparing |
| docked | otherwise | none |
| preparing, missile slot | | none (claim 8 releases it) |
| preparing, non-missile | no controller target | returning |
| preparing, non-missile | controller target | re-sent to the step direction 4 neighbour of the spawner's cell (S-0501) |
| on the rail | rail time over | empty, regeneration countdown starts |
| attacking | object out of ammunition, or no controller target | returning |
| attacking | otherwise | object keeps attacking the controller target |
| returning | ammunition left and a controller target | attacking |
| returning | object in the spawner's cell, object height minus spawner height < 20 leptons (signed, not absolute) | object taken off the map; reloading, reload countdown starts |
| returning | otherwise | object moves back to the spawner |
| reloading | reload time over | docked; ammunition and health refilled |
| empty | regeneration time over | new object for the spawner's current house, off the map; docked |

**Controller table** (claim 5):

| State | Event | Result |
|---|---|---|
| idle | pending target present | it becomes the controller target |
| idle | controller target not attackable from here | reset (claim 11) |
| idle | controller target attackable | launching |
| launching | no controller target | reset |
| launching | some slot neither preparing nor empty (including a slot on the rail) | wait |
| launching | all slots preparing or empty | release (claim 8); reset if a missile was released; cooling down |
| cooling down | no slot attacking or returning | idle |

**Rocket table** (claims 15 to 19; phase changes and per-tick effects):

| Phase | Event | Result |
|---|---|---|
| (none) | first move order | aim point fixed; pause (or tilt when the pause time is 0, including set C); pitch = *P*ᵢ × π/2 |
| pause | pause time over | tilt (sets A, B) or raise (set C), timer = tilt time |
| tilt | each tick | pitch by F3 |
| raise | each tick | world height + raise rate, when that position is legal |
| tilt, raise | timer over | pitch = *P*_f × π/2; tracked flying object; sound; ascent |
| ascent | each tick | speed by F4 |
| ascent | height above the terrain ≥ cruise altitude | cruise; record *d*₀ |
| cruise | height above the terrain < 1 | detonate |
| cruise, plain | each tick | speed by F4; pitch by F5; descent when claim 17's test holds |
| cruise, lazy | claim 19 test | detonate |
| cruise, lazy | otherwise | speed by F4; pitch by F6 |
| descent | claim 19 test | detonate |
| descent | otherwise | pitch by F7 (speed unchanged) |
| any phase with speed > 0 | no pre-move detonation | movement by F1 with that tick's speed and pitch; health < 1 gives a detonation at the resulting position, including a rejected step's unchanged position |

Ordering consequences:
- With an initially idle, targetless controller, no launch blockers and an intact salvo, a shot
  between passes gives its first appearance at the second pass after the shot. With *N* missiles
  and spacing *s*, missile *k* (from 0) appears ⌈*s* / 10⌉ × *k* passes after the first; all are
  released together in the pass in which the last appears.
- The rail countdown starts at release, not at the start of the missile's own flight phases.
  With zero pause and a set-A rocket, detachment is at the first pass at or after release plus
  the imported tilt time; regeneration starts there, even if the actual tilt ended earlier.

## Edge cases and quirks

- **`quirks.launch_spacing_from_spawner_type`** (default on). The 9-tick spacing is chosen by the
  missile-spawn flag of the spawner's own type, not of the spawned type. A spawner of missiles whose
  own type lacks the flag launches them one every 20 ticks. Off: the spawned type's flag decides.
- **`quirks.raised_missile_rail_time_from_set_b`** (default on). A set-C missile's rail time uses set
  B's pause and tilt times, not set C's, so its slot can be on the rail (and the spawner launching,
  claim 10) for a time that does not match its own take-off. Off: set C's own times.
- **`quirks.rail_missile_vanishes_with_spawner`** (default on). Claim 13: missiles on the rail vanish
  with no detonation when the spawner dies or changes hands. Off: they are released like preparing
  missiles.
- **`quirks.rocket_aim_at_heading_point`** (default on). Claim 14.
- **`quirks.destroyed_rocket_still_detonates`** (default on). Claim 20.
- **`quirks.rocket_stalls_at_map_edge`** (default on). Claim 18: a missile whose path leaves the
  legal cells keeps the last legal position and continues its phase, speed and pitch updates.
  It is not automatically detonated just for crossing the edge; the ordinary impact and health
  tests can still detonate it. Off: it detonates at the last legal position.
- **Reset with a target out of range at adoption.** If the target moves out of range between the
  shot and the pass that adopts it, the controller resets and no object is launched; the spawner's
  next shot starts over.
- **A slot object that is destroyed while it still belongs to its slot** frees the slot (empty, with
  a regeneration countdown) on loss notification, which can precede removal (claim 9).
- **Partial launch and control:** S-0214 owns the rule that the player cannot move a spawner while
  0 < launching count < *N*.

## Import mapping

| Key (file, section) | Concept | Parse form | Absent-key default |
|---|---|---|---|
| `Spawns` (rules, object type) | spawned type | aircraft type name | none (no controller) |
| `SpawnsNumber` (rules, object type) | spawn count *N* | integer | 0 |
| `SpawnRegenRate` (rules, object type) | regeneration time | integer, ticks | 0 |
| `SpawnReloadRate` (rules, object type) | reload time | integer, ticks | 0 |
| `Spawned` (rules, object type) | the type is a spawned type (S-0214 *movable*) | boolean | no |
| `MissileSpawn` (rules, object type) | missile-spawn flag | boolean | no |
| `Spawner` (rules, weapon) | spawner weapon (S-0301) | boolean | no |
| `Speed` (rules, object type) | type speed *V* (S-0207 claim 1) | integer | (S-0207) |
| `V3RocketType`, `DMislType`, `CMislType` (rules, General) | rocket type of sets A, B, C | aircraft type name | none |
| `V3RocketPauseFrames`, `DMislPauseFrames`, `CMislPauseFrames` (rules, General) | pause time of sets A, B, C | integer, ticks | A 0, B 10, C 10 |
| `V3RocketTiltFrames`, `DMislTiltFrames`, `CMislTiltFrames` (rules, General) | tilt time (raise time for set C) of sets A, B, C | integer, ticks | A 60, B 60, C 30 |
| `V3RocketPitchInitial`, `DMislPitchInitial`, `CMislPitchInitial` (rules, General) | *P*ᵢ of sets A, B, C | real, read in double and stored in single precision, quarter turns | A 0.0, B 0.0, C 1.0 |
| `V3RocketPitchFinal`, `DMislPitchFinal`, `CMislPitchFinal` (rules, General) | *P*_f of sets A, B, C | real, as the initial pitch | A 1.0, B 1.0, C 1.0 |
| `V3RocketTurnRate`, `DMislTurnRate`, `CMislTurnRate` (rules, General) | τ of sets A, B, C | real, single precision, radians per tick | A 0.05, B 0.05, C 1.0 |
| `V3RocketRaiseRate`, `DMislRaiseRate`, `CMislRaiseRate` (rules, General) | raise rate of sets A, B, C | real, truncated to an integer, leptons per tick | A 1, B 1, C 1 |
| `V3RocketAcceleration`, `DMislAcceleration`, `CMislAcceleration` (rules, General) | *a* of sets A, B, C | real, single precision, leptons per tick per tick | A 0.4, B 0.4, C 0.6 |
| `V3RocketAltitude`, `DMislAltitude`, `CMislAltitude` (rules, General) | cruise altitude of sets A, B, C | integer, leptons above the terrain | A 768, B 768, C 768 |
| `V3RocketDamage`, `DMislDamage`, `CMislDamage` (rules, General) | damage of sets A, B, C | integer | A 1000, B 1000, C 500 |
| `V3RocketEliteDamage`, `DMislEliteDamage`, `CMislEliteDamage` (rules, General) | elite damage of sets A, B, C | integer | A 1000, B 1000, C 500 |
| `V3RocketBodyLength`, `DMislBodyLength`, `CMislBodyLength` (rules, General) | body length *L* of sets A, B, C | integer, leptons | A 256, B 128, C 128 |
| `V3RocketLazyCurve`, `DMislLazyCurve`, `CMislLazyCurve` (rules, General) | lazy-curve flag of sets A, B, C | boolean | A yes, B no, C no |
| `V3Warhead`, `V3EliteWarhead` (rules, CombatDamage) | set A warhead, elite warhead | warhead name | none |
| `DMislWarhead`, `DMislEliteWarhead` (rules, CombatDamage) | set B warheads | warhead name | none |
| `CMislWarhead`, `CMislEliteWarhead` (rules, CombatDamage) | set C warheads | warhead name | none |

The rocket parameter keys are in General; the warhead keys are in CombatDamage. The defaults
above are the original's own initial absent-key values, not stock rule values. Later rules layers
retain the current value for an omitted key (S-0006 claim 2).

## Measurement scenarios

All scenarios: skirmish, director and mods off, a small purpose-made map, game speed index 2,
800 × 600, one human test house and one isolated bystander computer player (glossary void rule).
Use stock data except the explicitly named test-map overrides below; remove those after each run.
Record tick numbers and positions with a read-only recorder and capture video for visible effects.
Scripted normal input is allowed. No game-state writes or patched executable are allowed. Run each
variant five times unless stated otherwise. A test harness may drive normal input and collect
observables, but it must not assume that the original's seed can be set: record random state when needed.
Every stated expected result is falsified by a differing tick, position, count, target, health delta
or animation event after accounting for the cited specs. No animation implies no conclusion about
its timing when its type is absent.

"Spawner" means a stock missile-launcher vehicle or ship whose missiles use set A or set B,
unless stated. Named test maps may override spawn count, the missile-spawn flag, pause and tilt
durations, body length, lazy curve, initial rank, missile type and test-target health or armour.
Only each scenario's named changes apply. Other named overrides stated within an individual
scenario are allowed for that scenario alone. The observed area is the spawner, its entire
flight path, its targets and the interception or damage-test patch; the bystander void rule
applies throughout that area. These are data-only tests, not stock-data measurements.

- **R1. Initial fill.** Build a spawner; read its spawn pips (S-0111) at once. Expected: full count
  from the first tick. Falsified by pips filling over time.
- **R2. Pass timing and targets (claims 2–5, 8).** In range of a static target, record the spawner's
  creation tick, the actual shot tick *t*₀ (not the attack-order tick), missile appearances, aim
  points and first motion. Expected for a fresh idle controller: appearance at the second pass
  after the shot, with passes at creation + 20 and every 10 thereafter. Repeat ten times with
  input timed on either side of a pass. On a named two-slot map, send a second shot at another
  in-range target between appearances. Expected: the later launch adopts the pending target,
  while the earlier missile keeps its fixed aim. Both slots enter the rail interval together at
  the final launch pass, but their individual flight timers start from their own first move orders.
  Clear the spawner's own target before adoption: the pending shot does not cause a launch.
  During cooling down, deliver another shot and record the next appearances: no launch occurs
  until a later pass after the return to idle. Fire again at the already adopted target while a
  different pending target is waiting: the pending replacement remains, not overwritten.
- **R3. Launch spacing and blockers (claims 6, 8).** A named test map gives a stock launcher two
  slots and varies only its own missile-spawn flag. Record appearance ticks: expected 20 ticks
  apart without the flag and 10 with it. In separate runs order movement before the first launch,
  or restrain the launcher with a magnetic beam. Expected: no rocket placement while its locomotor
  reports motion or while magnetically restrained, then placement on the first eligible pass. A named
  ordinary carrier variant confirms that aircraft launch is not blocked merely by carrier motion.
  A named variant removes the missile-spawn flag from a named rocket type; its released slot
  enters attacking rather than the rail interval. Another gives an ordinary spawned aircraft
  that flag without naming it as any rocket type; its slot starts regenerating at release, with
  no rail delay. Record slot pips and subsequent appearances; a rail delay in either variant
  falsifies the flag/type distinction in claim 8. On the moving-carrier run, also change the
  pending target during preparation, attack and return; subsequent attacks use the new target.
- **R4. Launch point and facing.** Spawner facing set to several facings (including just below and
  above a 256-step boundary); record the missile's first position and facing. Expected: fire point
  + 10 leptons height, facing rounded per F2; set-C type shifted 40 leptons on both axes. A named
  two-slot map with burst above 1 tests alternating sides, and named primary/secondary spawner
  weapon variants test the fire-point selection. Also vary the primary burst while the spawner
  weapon is secondary: the primary burst controls alternation. Any different side or point
  falsifies claim 7.
- **R5. Regeneration.** After a salvo, record the ticks from release to each pip refilling.
  Expected: detachment at the first later pass at or after release plus the rail time
  (pause + tilt, set B's for a set-C missile); refill at the first later pass at or after
  detachment plus regeneration time. A zero duration still waits for a later pass. Include a named set-C launcher with unequal
  set-B and set-C pause-plus-tilt sums: the rail interval follows B, not C. Kill a still-linked
  aircraft and record pip loss at crash start rather than waiting for final removal (claim 9).
  Named zero-regeneration and zero-reload variants test pass separation: empty becomes docked
  at its next eligible pass but does not launch in the refill pass; reload expiry likewise docks
  without a same-pass launch. A different appearance tick falsifies these constraints.
- **R6. Lockouts.** A vehicle spawner whose spawn count is 1, ordered to move during the launch; a
  ship spawner whose spawn count is 2 or more, ordered to move between its first two launches (set
  the counts in a test copy of the data if needed). Expected: a stationary vehicle starts no
  track while launching and starts only after the launching count is zero (including the rail
  interval). Also initiate launch during carrier travel and record track boundaries: the current
  track finishes, but the next waits until launching ends. A mid-track freeze falsifies claim 10.
  The ship cannot be ordered (not movable) between the two launches. A vehicle's firing check
  answers *busy* throughout that interval. A named long-regeneration variant with every slot
  empty tests *rearming*: no new spawner shot occurs until at least one slot is alive.
  Separately infect a stationary carrier or launcher with a parasite, using a named long-lived
  host variant and long hold duration. Attempt a spawner shot while held but not launching, with
  an in-range legal target and an occupied slot. Record hold state, firing answer and shot events:
  expected *impossible* and no shot, even for a non-organic host. Repeat with only the host's
  organic flag changed; the spawner answer remains *impossible*, not the later organic-only
  *illegal* answer. A shot or different answer under these conditions falsifies this lockout.
- **R7. Reset.** (a) Target driven out of range in the ticks between the shot and adoption:
  expected no launch. (b) Target killed by another unit while a missile is preparing: expected the
  missile still flies to the original aim point and detonates there. Also move the target out of
  range after adoption: this alone does not reset a launching controller. Use a named pulse test
  weapon to disable the spawner during preparation: preparing missiles are released, the target
  clears and replacement countdowns start. Repeat target loss with a pending replacement target
  to distinguish reset from pending-target adoption. Repeat a carrier reset during preparation:
  those unflagged aircraft remain linked rather than crashing or regenerating.
- **R8. Spawner death and capture.** Kill the spawner (a) while a missile's slot is on the rail,
  independently of the missile's own flight phase: expected the missile vanishes, no detonation; (b) after the rail interval has ended:
  expected the missile flies on. Mind-control while docked: replacements appear at the next pass.
  Repeat with one slot already empty and regenerating: that slot keeps its old remaining delay.
  Separately apply a locomotor-changing weapon or an erasing weapon during preparation: the
  preparing missile is released, whereas a rail-linked missile vanishes. For carriers include
  docked, reloading, preparing, attacking and returning aircraft: silent removal in the first two
  cases, crash in the latter three (claim 13). Repeat death during magnetic-beam hold:
  destruction or crash start alone does not clear slots while held; a later removal does.
  Record missile existence between those events. Vanishing at the held death/crash event,
  before removal, falsifies that exception. A carrier aircraft starting to crash also clears its
  own spawned slots if given them by a named map variant; compare with ground destruction.
  These variants name only the aircraft's spawned type and positive spawn count as extra data.
- **R9. Moving target.** Target a unit driving across the spawner's line of fire; record the impact
  point, the missile's first move-behaviour tick and the target's step end then. Expected: fixed
  aim at that step end, not the placement-tick or arrival-tick position; compare the nose using F9,
  not an arbitrary tolerance. Add tunnel, ordinary building and docking-building targets to test
  each heading-point case of claim 14.
- **R10. Flight profile.** Record the missile's position and height each tick for sets A (lazy curve)
  and B (plain cruise) at several distances. Expected: no motion during pause/tilt; ascent at
  speed growing by the acceleration per tick to the type speed; cruise start at the cruise
  altitude above the terrain; set A curving down; set B flying level until the aim point is 45 degrees
  or more below it, then turning down onto it. Compare per-tick positions with F1 to F7. Named
  set-C variants with positive and zero pause distinguish raise from tilt; zero pause enters tilt.
  Include heading changes, house speed bonuses and different terrain: only type speed caps rocket
  acceleration. Test flying-object area hits before and after tilt/raise ends: only the latter
  find it through that search. A named map with a launcher near the legal boundary and a strongly
  curved trajectory tests claim 18: rejected steps keep position, not speed or pitch.
- **R11. Detonation (claim 19, F9).** Use a named high-health test structure and a named test
  warhead with an invented neutral armour multiplier; retain the imported rocket damage. Record
  explosion point and target health, accounting for S-0302 and S-0303 rather than assuming a raw
  damage-to-health identity. Expected: the rocket set's amount, elite amount for an elite launch,
  unaffected by a spawner firepower crate or its weapon damage. For resource-bearing impact
  cells record resource reduction, and for kill credit record recipient experience according
  to S-0311 claim 8 and its scenario X3 missile variants. A different resource/experience outcome
  after accounting for S-0303 and S-0311 falsifies that part of claim 19. Vary body length on the named map
  and compare F9, including non-cardinal headings and negative vertical nose components. Repeat
  over a high bridge, with aim height at terrain and deck height, to distinguish claim 19's bridge
  contact test. Record random state around animation creation and damage: animation-creation
  draws precede hit-effect draws. An amount, nose position or draw order mismatch falsifies it.
- **R12. Elite timing.** Promote the spawner to elite (a) before the shot, (b) during the tilt,
  (c) after tilt/raise has ended. Use named unequal normal/elite test amounts or warheads to make
  the choice visible. Expected elite choice in (a) and (b), not (c). Release can occur during tilt,
  so release alone is not the rank-sampling cutoff.
- **R13. Interception.** Shoot down a missile in flight with anti-air fire; record whether and where a
  detonation of the set's damage occurs. Expected: a detonation with full damage on the next
  positive-speed movement attempt, at the nose point after that attempt, not the body position. Kill one in pause or tilt: expected a
  detonation with the set's full damage on its first moving tick after its ascent starts.
  Expected timing is the next rocket update with positive speed, which can be in the killing tick
  if the hit precedes that update. Removal without the predicted detonation falsifies continued
  rocket flight. Record any extra fall before the update separately: it decides the unresolved
  crash displacement, not the confirmed health test. A named low launch-height variant tests
  immediate ground removal without detonation.
- **R14. Carrier cycle (claims 4, 9, 12, 13).** Use an ordinary carrier and an in-range target.
  Record aircraft destinations, ammunition, health, return cell, reload interval and pips.
  Expected: preparation toward the opposite neighbouring cell, attacking only after all slots
  are preparing or empty, return on ammunition exhaustion or target loss, and re-attack if a
  target returns while ammunition remains. In the spawner's cell the signed height difference
  must be below 20 leptons for docking; a named sloped test map tests an aircraft below the
  spawner too. Reload restores type ammunition and health at the first pass after expiry.
  Capture or destroy the carrier during each state to test the clearing table, and compare
  crashed aircraft with silently removed ones. With multiple aircraft in flight, record random
  state and crash-start events: their draws follow reverse slot order. Different states, refill
  values, timing or draw order falsify.
- **R15. Animation intervals (claim 21).** Capture tick-labelled video of set A/B flight and set-C
  pause/raise. Named longer pause/raise durations make multiple emissions visible. Expected:
  3-tick intervals in sustained flight, 24-tick intervals in sustained set-C pause/raise,
  with the shared timer carrying across phase changes. Count any animation draws with the
  random-state recorder. Different sustained intervals or independent phase timers falsify.

## Open questions

- Whether a destroyed rocket missile keeps running its phases (pause and tilt timers) while it is
  crashing, and whether the crash fall of S-0312 claim 16 moves it too (claim 20; scenario R13).
- The complete list of events that reset the controller (claim 11) and clear its slots (claim 13),
  especially other disabling effects than the electromagnetic pulse.
- Whether all building docking classifications in claim 14 are reachable as stock rocket targets.
- S-0204 claim 8 describes but does not supply all original raw arctangent entries. F6 and F7
  retain these angles rather than converting them to integer facings; the facing corrections
  cannot supply exact rocket pitches. A complete shared raw-angle table contract is needed
  from S-0204 before these formulas can be implemented bit-exactly.

## Questions for the project lead

- Confirm the proposed quirk names and alternatives, including the map-edge option, before sign-off.
