---
id: S-0206
title: Turn rates and turning
topic: movement
revision: 10
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0206: Turn rates and turning

## Summary

How a facing of an object (a body, a turret, a barrel, a locomotor's own heading) turns towards a
new target facing over ticks; how the imported turn-rate field becomes a turn rate; which facing of
which object kind uses which rate; how a vehicle's turret chooses where to point (target, body,
spin, rest) and when it may re-aim; and how turning interacts with moving. Facings, directions and
the signed facing difference are defined in S-0204. How a vehicle's facing follows a curved path
while it drives belongs to the ground-vehicle movement spec (S-0207); this spec says only where
that takes over.

## Terms

Glossary terms are used as defined there (*facing*, *tick*, *locomotor*, *jumpjet*, *building*,
*infantry*, *vehicle*, *aircraft*, *weapon*, *measurement setup*). From S-0204: *direction*,
*signed facing difference* δ, *n-step facing* (used in F2), *8-direction*. From S-0201: *tick
number*. From S-0207: *move destination*. From S-0211: *locomotion class* (one of whose values is
*track*).

- **Turning facing.** A facing that can turn over time. Besides its current value it has a
  *target facing* ψ₁ and a *turn rate* ω, and its current turn is described by its *start facing*
  ψ₀, the tick number *t*₀ at which the turn began and its *duration* Θ in ticks.
- **Turn rate** ω. An integer that sets how fast a turning facing turns: nominally 256 × ω facing
  units per tick (Formulas F1 gives the exact rule). ω ≤ 0 means the facing never turns over time:
  it takes every new target facing at once.
- **Turning.** A turning facing is *turning* while the current tick number is below *t*₀ + Θ and
  ω ≥ 1.
- **Body facing, turret facing, barrel elevation.** The turning facings of an object. Every
  vehicle and aircraft has all three. The body facing is the way the object points. The turret
  facing is the way its turret points, as an absolute facing on the map, not relative to the body.
  A vehicle without a turret still has a turret facing; nothing shows it, and its firing check does
  not read it: the check compares the body facing instead (claim 10 (c)). Claim 9's turret-less rule
  keeps that unseen turret facing equal to the body's target facing. The barrel elevation is
  S-0105's (claim 6). A building has no body that turns: it has a barrel elevation and one facing that turns, the turret facing of S-0105 claim 5,
  called its *turret facing* here.
- **Placement facing.** The facing at which an object is put on the map: 256 × *v* for a vehicle
  pre-placed with map facing value *v* (glossary claim G1), and whatever facing the placing event
  gives otherwise (for example a vehicle leaving a transport). The events themselves belong to
  other specs.
- **Movement heading.** A turning facing that a hovering or jumpjet locomotor keeps for the way it
  is travelling, separate from the body facing (claims 13 and 14).
- **Target.** The object a vehicle is currently attacking, if any: narrower than the glossary's
  *target*, which also covers objects being captured or entered. How it gets one (orders, its own
  behaviour) belongs to the order and behaviour specs.
- **Movement destination.** The place a vehicle is currently ordered or sent to move to, if any.
  How it gets one belongs to the order, behaviour and movement specs.
- **Locomotor moving the vehicle.** Whether a vehicle's locomotor is currently moving it. For the
  driving locomotor this holds while the locomotor has a move destination (S-0207), or a next point
  it is driving to whose map coordinates (x, y) differ from the vehicle's current map coordinates.
  Height is ignored in this comparison. This holds from a move order until the vehicle
  has arrived or its move is stopped (S-0207, claim 11), including a turn in place before a step. A
  vehicle standing still with neither, even while its body turns, is not being moved. Exactly when
  those points are set and cleared along a path is S-0207's. S-0207's *move destination* is the
  driving locomotor's own point; it is not the same as the vehicle's *movement destination* above,
  and the two need not be present at the same time. Claims 9 and 16 depend on this state.
- **Numbered weapons.** A type whose imported turret-count field is above 0 and whose imported
  weapon-count field *N* is above 0 has its weapons numbered 1 to *N*, each with an ordinary and an
  elite version (Import mapping), instead of a primary and a secondary weapon. Such a type has
  *numbered weapons*. Which number a vehicle uses at a given moment (its *current weapon number*)
  belongs to the combat and vehicle specs.
- **Turret weapon.** The weapon that the turret rules of claim 9 look at. For a type with a
  turret-count field above 0 it is the numbered weapon at the vehicle's current weapon number; for
  any other type it is always its primary weapon, even while it attacks with its secondary weapon.
  An elite vehicle uses the elite version of that weapon when its type has one, the ordinary version
  otherwise. A vehicle whose type has no weapon in that place has no turret weapon.
- **Elite.** A state a vehicle can be in during play, in which it uses the elite version of a weapon
  where its type has one (Import mapping). When a vehicle is elite belongs to other specs.
- **Turret-locked weapon.** A numbered weapon whose turret-locked flag is set in the type's art
  data (Import mapping). Only numbered weapons carry the flag; a primary or secondary weapon is
  never turret-locked, and neither is a missing turret weapon.
- **Omni-fire weapon.** A weapon whose imported omni-fire flag is set: it fires in any direction
  without aiming.
- **Homing projectile.** A weapon's projectile whose imported projectile turn-rate field is not 0
  (Import mapping). How it flies belongs to the combat specs; this spec uses only whether it is 0.
- **Spinning turret.** The turret of a type whose imported turret-spins flag is set.
- **Firing check, facing error.** The *firing check* is the test, made each tick for a vehicle with
  a target, of whether its weapon can fire now; it belongs to S-0301 (the firing cycle).
  A *facing error* is the result of that check which says the weapon is otherwise able to fire
  but is not pointing at the target. Claim 10 states what this spec relies on about when it is given.
- **Tank bunker.** A building that a vehicle can drive into, which then fires from inside it with
  a range bonus. Its rules belong to the structures specs.
- **Simple-deployer vehicle.** A vehicle type with the imported simple-deployer flag: it deploys in
  place into a second, deployed state without becoming a building. Its rules belong to the
  vehicle specs.
- **Vehicle-seizing weapon.** An enemy weapon that takes hold of a vehicle's locomotor and moves the
  vehicle (such as a magnetic beam). Its rules belong to the combat specs.

## Behaviour

### The turning rule

1. **Turning over ticks.** When a turning facing with turn rate ω ≥ 1 gets a new target facing ψ₁
   while its current value is ψ₀, a turn begins at the current tick number *t*₀. Its duration is
   Θ = ⌊|Δ| / (256 ω)⌋ ticks with Δ = δ(ψ₀, ψ₁), the signed facing difference of S-0204, and the
   facing moves by β = trunc(Δ / Θ) facing units per tick: at tick number *t*₀ + ε it is
   ψ₁ − β × (Θ − ε) for 0 ≤ ε < Θ, and ψ₁ from ε = Θ on (Formulas F1). Because β × Θ can fall short
   of Δ, the facing jumps by the remainder Δ − β × Θ when the turn begins. A turn shorter than
   256 × ω facing units (Θ = 0) completes at once. The intermediate facings use the truncated step
   β: in general β × (Θ − ε) ≠ trunc(Δ × (Θ − ε) / Θ), and F1 uses the former. S-0105
   states the same rule as F9 for the barrel elevation; this spec states it for every turning
   facing.
   - Backing: `measurable` (scenarios R1, R2 and R7)
   - Confidence: high

2. **Which way round.** A turn goes the shorter way round. A turn of exactly half a turn (Δ =
   −32768) goes counter-clockwise, through the facings below ψ₀.
   - Backing: `measurable` (scenario R1)
   - Confidence: high

3. **A new target during a turn.** A new target facing given while a turn is in progress starts a
   new turn from the facing's value at that tick (the interpolated value of claim 1), with a new
   duration from the new difference. Giving the same target facing again changes nothing: the turn
   in progress keeps its start, duration and timing. With ω ≤ 0 the facing takes every new target
   facing at once.
   - Backing: `measurable` (scenario R7, runs a and b)
   - Confidence: high

4. **Turning counts ticks.** The progress of a turn counts ticks (the tick number, S-0201), not real
   time. A turn takes the same number of ticks at every game speed (glossary claim G3), and makes
   no progress while no ticks run.
   - Backing: `measurable` (scenario R2, run at two game speeds)
   - Confidence: high

5. **Setting a facing at once.** Some events set a facing to a value at once instead of turning
   it. Such a setting ends any turn in progress: the facing, its target and its start all take the
   new value, so the facing rests there until it gets a new target. The specs of those events say
   when they happen; this spec names two of them:
   - Placing a vehicle on the map, whether pre-placed by the map or leaving a transport, sets its
     body facing and its turret facing at once to the placement facing (Terms). The turret does
     not keep the way it pointed before. What placement does to the barrel elevation is S-0105's
     (claim 6).
   - A vehicle following a path sets its body facing at once (claim 12).

   One exception (`quirks.set_facing_keeps_target`): if a turn is in progress and the facing's
   current (interpolated) value is already exactly the value being set, the turn ends but the
   target is kept, so the facing jumps to the turn's target facing ψ₁ instead of staying at the
   value set. When no turn is in progress the current value is the target, so nothing changes.
   - Backing: `measurable` (scenario R8: run a for pre-placement, run b for a turret whose
     earlier turn is overridden when it leaves a transport; scenario R2 for a pre-placed body
     turning from its placement facing).
     The exception is `unmeasured`: it needs a set-at-once value that equals an interpolated facing
     exactly in one tick, which no scenario can arrange reliably.
   - Confidence: high for the rule and for pre-placement; medium for leaving a transport being a
     placement; high for the exception's behaviour, low for how often play reaches it

### Turn rates

6. **From the field to the turn rate.** A type's turn rate comes from its imported turn-rate field
   *v* (an integer, Import mapping), whose absent-key default is 0:
   - *v* from 1 to 126 gives ω = *v*, and any *v* of 127 or more gives ω = 127.
   - *v* = 0, the absent-key default, gives ω = 0: the facing takes every new target facing at once.
   - a negative *v* acts through its remainder modulo 256, *b* = *v* mod 256: ω = *b* when *b* is
     1 … 127, and the facing takes new target facings at once when *b* is 0 or 128 … 255. For
     example *v* = −200 turns like 56, and *v* = −5 turns at once
     (`quirks.turn_rate_negative_wrap`).

   The same rule applies to the jumpjet turn-rate field and to the doubled rate of claim 13.
   S-0105's statement that a turn rate of 0 or less turns at once refers to ω after this
   conversion; a negative field reaches ω only through this claim.
   - Backing: `measurable` (scenario R2 for 1 … 127; scenario R3 for 0 and a negative value)
   - Confidence: high

7. **Which facing uses which rate.**

   | Object kind | Facing | Turn rate ω |
   |---|---|---|
   | Vehicle | Body facing | the type's turn rate (claim 6) |
   | Vehicle | Turret facing | the type's turn rate: the same as the body |
   | Aircraft | Body facing and turret facing | the type's turn rate |
   | Building | Turret facing | the type's turn rate |
   | Infantry | Body facing | 127, whatever its turn-rate field says (claim 15) |
   | Vehicle, aircraft, building | Barrel elevation | 3 (S-0105, claim 6) |
   | Hovering locomotor | Movement heading | twice the type's turn-rate field, then claim 6 (claim 13) |
   | jumpjet locomotor | Movement heading | the type's jumpjet turn-rate field, absent-key default 4 (claim 14) |

   - Backing: `measurable` for vehicles (scenario R2 for the body, R4 for the turret) and buildings
     (scenario R10); infantry as claim 15; the two locomotors as claims 13 and 14. The aircraft row
     is `unmeasured`: the flying locomotor sets an aircraft's body facing at once in some flight
     phases, which the aircraft specs own, and until those are specified no scenario can separate
     the rate from those settings (Open questions).
   - Confidence: high for vehicles, buildings and infantry; medium for aircraft and the two
     locomotors

### Turrets

8. **The turret points on the map, not with the body.** A turret facing is an absolute facing. When
   the body turns, a turret facing that has no reason to change (claim 9) keeps pointing the same
   way on the map.
   - Backing: `measurable` (scenario R4 run c and scenario R5: the turret's course does not depend
     on the body's turns)
   - Confidence: medium

9. **Where a vehicle's turret points.** Each tick, the target facing that a vehicle's turret has
   at the end of the tick is the one given by the first row of this table that applies (Formulas F2
   for the spin step). Other rows that apply in the same tick give the turret a target too, which
   leaves a trace in its facing ("Several rows in one tick" below). A row whose effect sets a target
   facing equal to the current target changes nothing (claim 3).

   | Situation in this tick | Turret target facing |
   |---|---|
   | The type has a spinning turret | the turret's current facing rounded to the nearest 256th of a turn, plus 2048 (F2) |
   | It has a target, the turret was not turning at the end of the previous tick, and it has a turret weapon that is turret-locked and not omni-fire | the body's current facing |
   | It has a target, the turret was not turning at the end of the previous tick, and it has a turret weapon that is not omni-fire | the direction of the target (S-0204, claim 8) |
   | It has a target and its firing check gives a facing error (claim 10) | the direction of the target |
   | It has a target otherwise (the turret was still turning, it has no turret weapon, or that weapon is omni-fire) | no change (`quirks.turret_reaim_after_turn`) |
   | No target, the turret is not turning, at least *G* + 5 ticks have passed since the vehicle last fired, the vehicle is not inside a tank bunker, and it is not a deployed simple-deployer vehicle; it has a movement destination and no turret-locked turret weapon | the direction of the movement destination, except no change while the vehicle's locomotor is held by an enemy's vehicle-seizing weapon |
   | The same, but it has no movement destination or its turret weapon is turret-locked | the body's current facing |
   | Otherwise | no change |

   *G* is the game-wide guard-area targeting delay in ticks (Import mapping). A vehicle that has
   never fired counts as having last fired 100 ticks before tick number 0. The spin row applies to
   every type with the turret-spins flag, whether or not it has the turret flag; a spinning turret
   never uses the two rest rows.

   **Turning, as the rows read it.** The target rows look at whether the turret was turning at the
   end of the previous tick, after every target that tick gave it. The rest rows look at whether it
   is turning in the current tick. So a turret that reaches its target in the tick with tick number
   *t*₀ + Θ counts as still turning for the target rows in that tick and gets a new target from them
   at the earliest in tick *t*₀ + Θ + 1, while an idle turret can take a rest target in tick
   *t*₀ + Θ itself. A spinning turret never counts as turning for the target rows (nor for claim
   10 (b)), although it turns almost every tick: while it has a target, the target rows apply to it
   in every tick, and their targets leave the trace that the several-rows rule below describes.

   **Without a turret.** A vehicle without a turret that has a target and a facing error, has no
   movement destination and whose locomotor is not moving it turns its body instead: its body
   target facing becomes the direction of the target, and its turret target facing becomes that
   same body target facing. This happens every tick the facing error lasts. Which locomotor states
   count as moving the vehicle is in Terms: a vehicle turning in place before a step of its path
   counts as moving, and one that only turns its body towards a target does not. A vehicle with a
   turret never turns its body by this rule; claim 16 gives one more rule for turret-less vehicles
   of the track class.

   **Several rows in one tick.** The table gives the turret's target at the end of the tick. Three
   of its rows can apply in the same tick: the spin row, the target row (the second row if it
   applies, otherwise the third) and the facing-error row. The rest rows need "no target", so they
   never apply together with any of these three. A tick in which more than one of the three applies
   leaves an observable trace in the turret facing: the turn towards the final target starts from
   the start value that Formulas F5 gives, not from the turret's current value, and the facing that
   the spin row rounds (F2) is the value F5 gives just before the spin row, *s*ₖ₋₁ when the spin
   row applies (it is always the last row in F5's numbering), not the value before the tick's other
   rows. Each replaced target counts as a new target facing by claim 3, so one equal to
   the target in force at that point changes nothing. Any other replaced target leaves its
   remainder jump of claim 1 behind (at most Θ − 1 facing units, with Θ of the replaced turn).
   - Backing: `measurable` (scenario R4: runs a and b for the target row, c and d for the rest
     rows, e for the omni-fire condition, f for which row wins when the turret-locked row and the
     facing-error row both apply. Scenario R5 for the re-aim rule, R6 runs a to e for the spin row,
     R6 runs f to h for a spinning turret never counting as turning and for the several-rows rule
     including the replaced-target trace, R7 run a for the rule without a turret, R9 for the tank
     bunker, deployed and seized conditions and for a vehicle without a turret weapon, and R11 for
     the turret-locked row holding the turret on the body)
   - Confidence: high for the spin row, for a spinning turret never counting as turning, and for
     what the facing-error row does when it applies (when it applies is claim 10, medium); medium
     for the turret-locked, omni-fire, re-aim, rest and turret-less rows, for which locomotor states
     count as moving and for the replaced-target trace; low for the seized condition and for the
     exact meaning of "last fired" (the tick of the last shot of any weapon)

10. **When the firing check gives a facing error.** The firing check belongs to S-0301 (the firing
    cycle). This spec relies on four properties of it, which decide when the
    facing-error row of claim 9 can act:
    - (a) A target out of the weapon's range gives a range result, never a facing error.
    - (b) While the turret was turning at the end of the previous tick (as the target rows read
      it), the check gives a "turret still turning" result instead of a facing error, unless the
      weapon's projectile is a homing projectile. That result also stops the shot. So for a weapon
      with a projectile that does not home, a turning turret is never re-aimed: it finishes its
      turn, waits one tick (claim 9) and only then turns towards where the target is now. Only
      vehicles with a turret that does not spin keep this state, so a vehicle without a turret, or
      with a spinning turret, can get a facing error in any tick; the body of a vehicle without a
      turret is re-aimed every tick (claim 9).
    - (c) The weapon counts as pointing at the target, and gives no facing error, while the
      vehicle's *aiming facing* differs from the direction of the target by at most 2048 facing
      units, or by at most 4096 when its projectile is a homing projectile. The aiming facing is
      the turret facing for a vehicle whose type has the turret flag, and the body facing for a
      vehicle whose type has not; the unseen turret facing of a vehicle without
      a turret is never compared. An omni-fire weapon never gives a facing error.
    - (d) The firing check of a tick sees the turret facing that targets given in earlier ticks
      produce at that tick number. A target that claim 9 gives in the same tick, even one reached
      at once (Θ = 0), is seen first by the next tick's check.
    - Backing: `measurable` (scenario R5 tests (a); scenario R7 runs b and c with scenario R4
      runs a and g test (b) and (d); scenario R11 tests both tolerances of (c) from both sides, and
      R6 run g the one at exactly 2048)
    - Confidence: medium (these are stated here only because the turret rule depends on them, and
      S-0301 (the firing cycle) and S-0310 (turrets and facing for fire) specify the relevant
      firing-check and aiming behaviour)

11. **Spin speeds.** Because a spinning turret's target facing moves 2048 facing units ahead of it
    every tick, and claim 1 applies, a spinning turret of a vehicle without a target that starts on
    a multiple of 256 turns by a fixed step per tick that depends on ω only in steps: 256 for
    ω = 1, 512 for ω = 2, 1024 for ω = 3 or 4, and 2048 for every ω of 5 or more. A full turn thus
    takes 256, 128, 64 or 32 ticks (`quirks.turret_spin_quantised`).

    While the vehicle has a target, the target rows of claim 9 act in the same ticks, and the
    several-rows rule changes the course. Two consequences, both following from claims 1 and 9
    with F2:
    - With ω = 1 the turret still spins, but each tick's turn starts from the remainder jump of a
      turn towards the target, so the steps are 256 facing units or more, many of them larger and
      not multiples of 256, and a full turn takes fewer than 256 ticks (233 for a target due map
      east, 230 for one due map west, scenario R6).
    - With a ω large enough that both the turn towards the target and the spin step complete at
      once (Θ = 0, for example ω = 127), the turret stops spinning: in every tick it shows the
      target's direction rounded to the nearest 256th of a turn plus 2048, and holds there while
      the target lasts. With a smaller ω of 9 or more it spins until it comes near the target's
      direction and then holds there in the same way.
    - Backing: `measurable` (scenario R6)
    - Confidence: high for the rule without a target; medium for the with-target consequences

### Turning and moving

12. **Driving vehicles turn in place before they start a step.** When a vehicle with the driving
    locomotor is about to start moving towards the next cell of its path in 8-direction *d*, and
    its body facing is not exactly 8192 × *d*, it first turns its body to 8192 × *d* by claim 1 and
    does not advance while the body is turning. Let *t*₀ be the tick in which it finds that its
    facing differs and gives the turn, and Θ the turn's duration (claim 1). It begins the step's
    track in the tick with tick number *t*₀ + Θ, the tick in which its body reaches 8192 × *d*,
    and makes progress in that same tick (S-0207, claim 12). The one exception is a turn that
    completes at once (Θ = 0, or ω = 0): the body has the new facing in tick *t*₀ already, but the
    vehicle makes no progress in tick *t*₀ and begins the track in tick *t*₀ + 1. A vehicle whose
    body already has the step's facing begins the track, and makes progress, in the first tick in
    which it is about to start the step (S-0207, claim 6). So, compared with such a vehicle, a turn
    in place delays the start of the step by max(Θ, 1) ticks, *t*₀ included. While it drives along
    its path, its body facing is set at once (claim 5) to the facing that the path's track gives for
    each point of the path (S-0207, claims 5 to 7); that change is not limited by the turn rate.
    These turning rules apply unchanged to the ship locomotor on ordinary travel tracks
    (S-0504 claim 1).
    - Backing: `measurable` (scenarios R1 and R2, run c of R2 for the tick in which the track
      begins, scenario R3 for the delay of max(Θ, 1) ticks against a vehicle that needs no turn,
      and S-0207 scenario D7 for the curve case; S-0504 scenario N1 for ships)
    - Confidence: high for the turn in place, the stop while turning and the tick in which the
      track begins; when a step counts as "about to start" is S-0207's

13. **Hovering locomotor.** A vehicle with the hovering locomotor keeps a movement heading that
    turns at twice the type's turn-rate field (passed through claim 6, so a field of 64 or more
    gives 127). Its body facing turns towards its direction of travel at the type's own turn rate.
    - Backing: `unmeasured` (no scenario here isolates the heading's rate from the hover movement
      itself, which the hover movement spec owns; Open questions)
    - Confidence: medium for the rates; low for how the heading drives the movement

14. **The jumpjet locomotor.** A jumpjet keeps a movement heading that turns at the type's jumpjet
    turn-rate field (absent-key default 4, passed through claim 6). When a unit gets the jumpjet
    locomotor, the heading is set at once to map east (16384). The rules data also has a game-wide
    jumpjet turn rate (Import mapping). It has no effect on play: from the moment a unit gets the
    locomotor, the heading turns at the type's field.
    - Backing: `unmeasured` (as claim 13: the heading's rate cannot be separated from jumpjet
      flight, which the jumpjet movement spec owns; Open questions)
    - Confidence: medium

15. **Infantry.** An infantry body facing has the fixed turn rate 127, whatever the type's turn-rate
    field says. A turn of fewer than 32512 facing units (127 × 256) therefore completes at once, and
    a larger one, which is at least 32512 and at most a half turn, takes exactly 1 tick
    (`quirks.infantry_fixed_turn_rate`). Many infantry actions set the facing at once anyway
    (S-0107).
    - Backing: `measurable` (scenario R3, run c)
    - Confidence: high for the rate; medium for which infantry facing changes go through it

### Turret-less vehicles of the track class

16. **A body turn can stop at the target's direction** (`quirks.track_body_turn_stops_at_target`).
    A body turn of a vehicle without a turret whose locomotion class is *track* (S-0211), that has a
    target, has no movement destination and whose locomotor is not moving it, stops at the direction
    of its target (S-0204, claim 8) if its interpolated value equals that direction exactly in some
    tick: in that tick the body's current facing becomes its new body target facing (claim 3 with
    Θ = 0). No facing error is needed. A body resting at that direction is not affected, and in every
    other tick this rule has no effect. It looks like a defect of the original rather than a
    deliberate turn towards the target.
    - Backing: `unmeasured`. It needs a tick in which a body turn towards another facing passes
      exactly through the target's direction while the firing check gives no facing error (with a
      facing error, claim 9 already sets the body target to that direction). No scenario can arrange
      that coincidence reliably, as with claim 5's exception.
    - Confidence: high for the behaviour; low for how often play reaches it

### Rotation sampling for sound

17. **A rotation sample is not a sound's lifetime.** For a vehicle whose type has a turret,
    each tick that processes the turret rules records whether the turret is turning after all
    applicable rows of claim 9 have taken effect. This *rotation sample* uses the current-tick
    turning definition in Terms, not the previous-tick re-aim gate. It includes spinning turrets,
    even though those never count as turning for that gate. A turn that completes at once gives
    a nonturning sample; a turn of positive duration begun by the rows gives a turning sample.
    A turn that begins and ends entirely between samples is not reported as turning.

    The vehicle's sound update uses the latest rotation sample, rather than testing its facing
    again. Thus a sampled turn can request a rotation sound even if it has since ended. Starting,
    ending looping, continuation and overlap of audible instances belong to S-0701 claim 23.
    Finishing a turn does not imply that its current sound sample is cut at once. This claim
    establishes sampling for vehicles only, not aircraft or buildings.
    - Backing: `measurable` (scenario R12; S-0701 scenario F12 for instance lifetime)
    - Confidence: high for vehicle sampling and the distinction from the re-aim gate

## Formulas

**F1. Turning** (claims 1 to 3). Integers. A turning facing with rate ω (1 … 127) gets target ψ₁
at tick number *t*₀ while its current value is ψ₀:

> Δ = δ(ψ₀, ψ₁) (S-0204, F5), Θ = ⌊|Δ| / (256 ω)⌋ ticks, β = trunc(Δ / Θ) facing units per tick
> (truncated toward zero, defined when Θ ≥ 1).

At tick number *t* = *t*₀ + ε:

> facing = (ψ₁ − β × (Θ − ε)) mod 65536 for 0 ≤ ε < Θ, and ψ₁ for ε ≥ Θ or Θ = 0.

mod is the non-negative remainder. Example (S-0105, F9): ψ₀ = 16384, ψ₁ = 11264, ω = 3 gives
Δ = −5120, Θ = 6, β = −853 and the facings 16382, 15529, 14676, 13823, 12970, 12117, then 11264.
Example of a half turn: ψ₀ = 0, ψ₁ = 32768, ω = 3 gives Δ = −32768, Θ = 42, β = −780, so the facing
is 65528 at ε = 0 (a jump of 8 counter-clockwise), then 64748, 63968 and so on, and 32768 from ε =
42 on.

**F2. Spin step** (claims 9 and 11). For a spinning turret with current facing *f* (the start value
*s*ᵢ₋₁ of F5 when other rows act in the same tick), with *f*₂₅₆ its 256-step facing (S-0204, F4),
that is *f* rounded to the nearest 256th of a turn, a half step rounding up:

> target = 256 × ((*f*₂₅₆ + 8) mod 256), with *f*₂₅₆ = ⌊(*f* + 128) / 256⌋ mod 256.

Integers, facing units. With F1 applied every tick and a turret that starts on a multiple of 256,
the step per tick is 2048 / max(1, ⌊2048 / (256 ω)⌋) (an exact division for every ω),
which gives 256, 512, 1024, 1024 and 2048 for ω = 1, 2, 3, 4 and 5 or more.

**F3. Turn rate from the field** (claim 6). For an integer field value *v*:

> ω = 127 when *v* ≥ 127; ω = *v* when 0 ≤ *v* ≤ 126; for *v* < 0, with *b* = *v* mod 256: ω = *b*
> when 1 ≤ *b* ≤ 127, else ω = 0.

ω = 0 means "set at once". mod is the non-negative remainder.

**F4. Pointing tolerance** (claim 10 (c)). Integers, facing units. With *f* the vehicle's aiming
facing (turret facing with the turret flag, body facing without it) and *d* the direction of its target, the weapon counts as pointing at the target when
|δ(*f*, *d*)| ≤ 2048, or |δ(*f*, *d*)| ≤ 4096 for a homing projectile.

**F5. Observable facing when several turret rows apply** (claim 9). All values are integers in facing units. In a tick where *k* ≥ 1 of the spin, target and facing-error rows apply, let τ₀ be the target in force before the tick and *s*₀ the facing supplied by F1 for that tick before these rows have an effect. Let τ₁ … τₖ be the target facings supplied by the applicable rows, listed in their table order, from lowest to highest. The final target is τₖ.

The following table gives the observable result associated with each row’s target, relative to the target and facing state immediately before that row:

| Relation of row target to preceding target | Result for target and facing |
|---|---|
| τᵢ = τᵢ₋₁ | No change to the turn or facing. |
| τᵢ ≠ τᵢ₋₁ and the replacement duration Θᵢ is positive | The target becomes τᵢ. The replacement turn starts from *s*ᵢ = (τᵢ − βᵢ × Θᵢ) mod 65536, which is F1’s value at ε = 0, including its remainder jump Δᵢ − βᵢΘᵢ. Θᵢ and βᵢ are given by F1 for the difference from the preceding facing to τᵢ. |
| τᵢ ≠ τᵢ₋₁ and the replacement duration is zero, or ω ≤ 0 | The target and facing become τᵢ at once. |

For the spin row, its target is the F2 target derived from the facing supplied by the preceding applicable row. If the final row’s target equals the target in force immediately before that row, the existing turn continues unchanged. Earlier effective target changes can therefore affect the facing visible in the tick even though the highest applicable row supplies the final target. With only one applicable row, this is claims 1 and 3. Example (scenario R4 run f): *s*₀ = 0 and τ₀ = 0; the facing-error row gives τ₁ = 16383 and *s*₁ = 3 (Θ₁ = 63, β₁ = 260); the turret-locked row gives τ₂ = 0, whose replacement turn completes at once, so the turret rests at 0.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Facing units per tick per unit of turn rate | 256 | facing units per tick | Nominal; F1 gives the exact rule |
| Largest turn rate | 127 | turn rate | Field values above clamp to it |
| Absent-key default of the turn-rate field | 0 | turn rate | Set at once |
| Absent-key default of the jumpjet turn-rate field | 4 | turn rate | Claim 14 |
| Absent-key default of the game-wide jumpjet turn rate | 3 | turn rate | Claim 14; no effect on play |
| Absent-key default of the guard-area targeting delay *G* | 36 | ticks | Claim 9 |
| Infantry body turn rate | 127 | turn rate | Fixed, claim 15 |
| Barrel elevation turn rate | 3 | turn rate | S-0105 |
| Spin lead | 2048 | facing units | F2: 8 steps of 1/256 turn |
| Last shot of a vehicle that has never fired | 100 | ticks before tick number 0 | Claim 9 |
| Turret rest delay extra | 5 | ticks | Added to the guard-area targeting delay, claim 9 |
| Pointing tolerance | 2048 | facing units | F4; claim 10 |
| Pointing tolerance, homing projectile | 4096 | facing units | F4; claim 10 |
| Hover movement heading factor | 2 | times the turn-rate field | Claim 13 |
| Heading of a jumpjet locomotor when attached | 16384 | facing units (map east) | Claim 14 |

## State and ordering

**Turning facing.**

| State | Event | Result |
|---|---|---|
| Resting (not turning) | New target facing ψ₁ ≠ current target, ω ≥ 1, Θ ≥ 1 | Turning: ψ₀ = current value, *t*₀ = current tick number |
| Resting | New target facing, ω ≥ 1, Θ = 0 | Resting at ψ₁ at once |
| Turning | Tick number reaches *t*₀ + Θ | Resting at ψ₁ |
| Turning | New target facing ψ₁′ ≠ ψ₁ | Turning again from the interpolated value, new Θ (resting at ψ₁′ at once if the new Θ = 0) |
| Turning or resting | New target facing equal to the current target | No change |
| Any, current value ≠ φ | Facing set at once to φ | Resting at φ (target and start are φ) |
| Turning, current value = φ | Facing set at once to φ | Resting at the target ψ₁ (`quirks.set_facing_keeps_target`) |
| Resting, current value = φ | Facing set at once to φ | No change |
| Any, ω ≤ 0 | New target facing ψ₁ | Resting at ψ₁ at once |

**Ordering consequences.** These restate claims above as observable timings.
- A turn given in the tick with tick number *t*₀ shows its first value (ψ₁ − βΘ) in that same tick
  and reaches ψ₁ in the tick with tick number *t*₀ + Θ (claim 1).
- A turret that does not spin, with a target, that finishes a turn in tick *t*₀ + Θ turns towards a
  new direction of its target from tick *t*₀ + Θ + 1 at the earliest, unless a facing error
  re-aims it sooner, which only a homing projectile allows (claims 9 and 10 (b)). With a projectile
  that does not home, such a vehicle fires at the earliest in that same tick *t*₀ + Θ + 1.
- A facing error found in a tick re-aims the turret in that same tick. When the target row, the
  spin row or both also apply in that tick, the first of them in claim 9's table gives the target in
  force at the end of the tick, and the turn towards it starts from the value that claim 9's
  several-rows rule gives.
- A driving vehicle that turns in place (claim 12) for Θ ≥ 1 ticks begins the step's track, and
  makes progress, in the same tick *t*₀ + Θ in which its body reaches the step's facing, not in
  the tick after it. After a turn that completes at once (Θ = 0 or ω = 0) it begins in tick
  *t*₀ + 1. A vehicle that needs no turn begins in the first tick of the step, so the turn in place
  delays the step by max(Θ, 1) ticks (claim 12). The facing that the vehicle compares with the
  step in a tick is the value claim 1 gives for that tick number, so the point within the tick at which the vehicle acts (S-0203) does not
  move the start. S-0207 (claim 12 and its state table) states what the vehicle does in that tick.

## Edge cases and quirks

- **`quirks.turn_rate_negative_wrap`** (default: on, the original). A negative turn-rate field acts
  through its remainder modulo 256 (claim 6, F3), and the importer warns about any negative
  turn-rate field.
  Off: every negative value gives ω = 0 (set at once), still with the warning.
- **`quirks.turret_reaim_after_turn`** (default: on, the original). A vehicle whose turret does not
  spin and was turning at the end of the previous tick does not re-aim it at its target
  by the target rows (claim 9). With claim 10 (b), a turret with a projectile that does not home
  therefore first finishes the turn it began, waits one tick, and only then turns towards where
  the target is now. Off: the target rows apply whether or not the turret was turning, so the
  turret re-aims at its target every tick. This switch does not change the firing check (S-0301,
  the firing cycle).
- **`quirks.turret_spin_quantised`** (default: on, the original). A spinning turret turns by the
  quantised steps of claim 11, so turn rates 3 and 4 spin alike and every rate of 5 or more spins at
  2048 facing units per tick. Off: the spin row's target is the facing that F2 reads plus
  256 × min(ω, 8) facing units, without rounding, so a spinning turret without a target turns by
  exactly 256 × ω facing units per tick, capped at 2048.
- **`quirks.infantry_fixed_turn_rate`** (default: on, the original). Infantry bodies turn at the
  fixed rate 127 (claim 15). Off: infantry use their type's turn rate by claim 6.
- **`quirks.set_facing_keeps_target`** (default: on, the original). Setting a facing at once to the
  value it has at that moment during a turn ends the turn at its target instead (claim 5). Off: the
  facing rests at the value set, with target and start equal to it, as in every other case.
- **Half turns go counter-clockwise** (claim 2). Not a switch: it is the signed facing difference of
  S-0204.
- **Remainder jump.** At the start of a turn the facing jumps by Δ − βΘ (at most Θ − 1 facing
  units, claim 1). This is part of F1, not a switch; S-0105 relies on it for the barrel elevation.
- **`quirks.track_body_turn_stops_at_target`** (default: on, the original). A body turn of a
  turret-less vehicle of the track class that passes exactly through the direction of its target
  stops there (claim 16). Off: the rule has no effect, and the body turn goes on to its own target
  facing as for every other vehicle.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, vehicle, aircraft, building and infantry type sections, key `ROT` | turn-rate field *v* (claims 6 and 7; ignored for infantry bodies, claim 15) | integer | 0 |
| Rules, type sections, the key spelled "jumpjet", "turn" and "rate" run together, each part with an initial capital (keys are case-sensitive, S-0001) | jumpjet turn-rate field (claim 14) | integer | 4 |
| Rules, section `[JumpjetControls]`, key `TurnRate` | game-wide jumpjet turn rate; no effect on play (claim 14) | integer | 3 |
| Rules, type sections, key `Turret` | whether the type has a turret (claim 9) | boolean | no |
| Rules, type sections, key `TurretSpins` | spinning turret (claims 9 and 11) | boolean | no |
| Rules, weapon sections, key `OmniFire` | omni-fire weapon (claims 9 and 10) | boolean | no |
| Rules, projectile sections (the projectile a weapon names), key `ROT` | projectile turn-rate field; not 0 makes a homing projectile (claim 10) | integer | 0 |
| Rules, vehicle, aircraft, building and infantry type sections, key `TurretCount` | turret-count field: above 0 makes the turret weapon the numbered weapon at the current weapon number, and, with a weapon count above 0, gives the type numbered weapons (Terms) | integer | 0 |
| Rules, the same type sections, key `WeaponCount` | weapon-count field *N* (Terms, numbered weapons) | integer | 0 in our engine; the original's value for an absent key is not established (low confidence) |
| Art, the type's art section, key `WeaponNTurretLocked`, where N is the weapon number 1 … *N*; read only for a type with numbered weapons | turret-locked flag of the ordinary version of numbered weapon N (claim 9) | boolean | no |
| Art, the type's art section, key `EliteWeaponNTurretLocked`, N as above | turret-locked flag of the elite version of numbered weapon N (claim 9) | boolean | the value of the ordinary version's flag (the row above) |
| Rules, section `[General]`, key `GuardAreaTargetingDelay` | *G*, in ticks (claim 9) | integer | 36 |
| Rules, vehicle type sections, key `IsSimpleDeployer` | simple-deployer vehicle (claim 9) | boolean | no |
| Rules, building type sections, key `Bunker` | tank bunker (claim 9), as in S-0214 | boolean | no |
| Rules, vehicle type sections, key `SpeedType` | locomotion class; claim 16 uses the value `Track` (S-0211 owns the mapping and its defaults) | name | as S-0211 |

The importer warns about every negative value of either turn-rate key (claim 6).

## Measurement scenarios

All scenarios run under the glossary's *measurement setup*, with a recorder that reads the tick
number. Where a scenario needs a field value that stock data does not have, it names a test rules
file (or test art file): a loose copy of that file in which only the fields named are changed,
removed after the run. Times in ticks are taken as differences between runs that share the same
order latency, or are counted from the first tick in which the recorded facing changes, so the
delay between an order and the first tick in which it acts cancels out. Unless a scenario says
otherwise, the only other player is a computer player whose objects are all at least 40 cells from
the test objects, so that nothing picks a target by itself. Targets that a scenario calls *own
targets* are unarmed vehicles of the human player, attacked with the force-fire modifier
(S-0214): no object
picks them as a target by itself, so the test vehicle has a target only when it is ordered to.
Every vehicle a scenario pre-places starts with its body and turret at its placement facing
(claim 5); the scenarios with a turret recorder check this in the first tick.

**R1. Half-turn sense (claims 1, 2 and 12).**
- *Setup:* a purpose-made flat map; one human player; a named test rules file that sets one stock
  vehicle type of the track class to turn rate 1. That vehicle is pre-placed with map facing value
  0 on cell (*x*, *y*).
- *Procedure:* run a: order a move to (*x*, *y* + 1), the neighbour to map south. Run b: order a
  move to (*x* + 1, *y*), the neighbour to map east.
- *Observable:* *m* = ticks from the order to the first tick in which the vehicle's position
  changes; screenshots every 16 ticks during the turn.
- *Runs:* 2 per order.
- *Expected:* in run a the body turns counter-clockwise (through map west, pointing upper left on
  the screen half-way), and *m*_a − *m*_b = 128 − 64 = 64.
- *Falsified if:* the body in run a turns clockwise (through map east), or *m*_a − *m*_b is not 64.

**R2. Turn time and the full facing resolution (claims 1, 4, 5, 6, 12; S-0204 claim 10).**
- *Setup:* as R1, with three copies of the vehicle in separate runs, pre-placed with map facing
  values 0, 1 and 4. Run the set at game speeds 1 and 5.
- *Procedure:* order a move one cell to map east.
- *Observable:* *m* as in R1.
- *Runs:* 2 per facing value and speed.
- *Expected:* the turns take 64, 63 and 60 ticks (Δ = 16384 − 256*v*, ω = 1, starting from the
  placement facing), so *m*₁ − *m*₀ = −1 and *m*₄ − *m*₀ = −4, at both game speeds. This also shows
  that a map facing keeps its full 1/256-turn step (glossary claim G1).
- *Falsified if:* *m*₁ = *m*₀ (the facing was kept more coarsely than 1/256 turn), the differences
  are not −1 and −4, or the values change with the game speed.
- *Run c (the tick in which the track begins, claim 12):* the recorder also reads the body facing
  and the position every tick. The test rules file also turns the type's acceleration flag off
  (S-0207), so that the vehicle moves in its first tick of progress (a type whose progress per tick
  on the test ground is at least 2 units does, S-0207 claim 4). Repeat the runs at speed 1 with
  facing values 0 and 4, and once more with facing value 4 and the turn-rate field set to 127 (ω =
  127, Δ = 15360, so Θ = 0). Observable: *k* = the tick number of the first tick in which the
  position changes minus that of the first tick in which the recorded body facing is 16384.
  Expected: *k* = 0 for facing values 0 and 4 (turns of 64 and 60 ticks), and *k* = 1 for the turn
  that completes at once. Falsified if *k* = 1 after the 64- or 60-tick turn (the track begins in
  the tick after the facing is reached), or *k* = 0 after the turn that completes at once.

**R3. Rate 0, a negative rate and infantry (claims 6, 12 and 15).**
- *Setup:* as R2 with facing value 0; a named test rules file sets the vehicle's turn-rate field to
  0 (run a), −200 (run b) and 56 (runs c′ and d), and in run c an infantry type's turn-rate field to
  1. Run d is the control: the vehicle is pre-placed with map facing value 128 (body facing 32768,
  map south), so it needs no turn.
- *Procedure:* runs a, b, c′ and d: order a move one cell to map south. Run c: pre-place the soldier
  facing map north (map facing value 0, the infantry facing written as the map's infantry entry
  allows) and order it to move one cell to map south.
- *Observable:* *m* as in R1; in runs a, b, c′ and d also the body facing every tick.
- *Runs:* 2 each.
- *Expected:* run a: the body takes the facing 32768 in the first tick in which it changes, with no
  value in between, and *m*_a − *m*_d = 1 (a turn that completes at once still delays the step by
  one tick, claim 12), not 128. Runs b and c′ give the same *m*, with *m*_b − *m*_d = 2 (a half turn
  of ⌊32768 / (256 × 56)⌋ = 2 ticks). In run c the soldier turns within 1 tick, not in 128 ticks.
- *Falsified if:* run a shows a facing between 0 and 32768; *m*_a − *m*_d ≠ 1; *m*_b ≠ *m*_c′ or
  *m*_b − *m*_d ≠ 2; the soldier takes visibly longer than 1 tick. A value *m*_a − *m*_d = 0 would
  show that a turn completing at once costs no tick, against claim 12 and R2 run c.

**R4. Turret aiming (claims 7, 8, 9 and 10).**
- *Setup:* a purpose-made flat map; a recorder that reads the vehicle's body and turret facings
  every tick, whether it has a target, and the ticks in which it fires. A named test rules file
  sets a stock vehicle type with a turret (and a weapon that is neither omni-fire nor turret-locked,
  with a projectile that does not home) to turn rate 1. The vehicle stands on open ground with map
  facing value 0, so body and turret start at 0. Own targets stand 5 cells due map east and 5 cells
  due map west of it, within weapon range. Below, *T* is the first tick in which the turret facing
  changes after an order.
- *Procedure:*
  - Run a: order an attack on the east target.
  - Run b: order an attack on the west target, and as soon as it is destroyed (and before *G* + 5
    ticks have passed), order an attack on the east target.
  - Run c: at least *G* + 5 ticks after game start, with the turret resting at 0 and no target,
    order a move to the cell to map east, and record both facings while the body turns.
  - Run d: after run a, let the target be destroyed and record the facings for the next *G* + 50
    ticks.
  - Run e: as run a, with a second test rules file that also sets the weapon's omni-fire flag.
  - Run f: as run a, with a stock vehicle type with a turret whose weapons are numbered weapons and
    which can attack the target as it is placed (without passengers or other changes), and
    a named test art file that sets the turret-locked flag of every one of its numbered weapons, so
    that its turret weapon is turret-locked whatever its current weapon number is.
  - Run g: as run a, with a second test rules file that also gives the weapon's projectile a
    projectile turn-rate field of 1 (a homing projectile).
- *Observable:* the turret and body facings every tick; the tick of the first shot.
- *Runs:* 1 each.
- *Expected:*
  - Run a: the turret reaches 16383 clockwise in tick *T* + 63 (S-0204, claim 8) and the body stays
    at 0. The first shot comes in tick *T* + 64 at the earliest (claim 10 (b) and the one-tick wait
    of claim 9).
  - Run b: the turret first reaches 49153. Then it turns from 49153 to 16383, and since
    δ(49153, 16383) = 32766 is positive it turns clockwise, through map north, in
    ⌊32766 / 256⌋ = 127 ticks.
  - Run c: the turret turns towards the movement destination (claim 9, rest row), clockwise to
    16383 in 63 ticks, while the body turns to 16384 in 64 ticks.
  - Run d: once *G* + 5 ticks have passed since the last shot, the turret turns back to the body's
    facing (0), counter-clockwise in 63 ticks.
  - Run e: the turret stays at 0 while the vehicle attacks.
  - Run f: the turret facing equals the body facing (0) in every tick, and the vehicle never fires
    (the turret stays 16383 facing units from the target's direction). The turret-locked row wins
    over the facing-error row in every tick. (This run shows which row wins, not the replaced
    target's trace: the replacing turn, from 3 back to 0, has Θ = 0 either way. R6 runs f to h test
    the trace.)
  - Run g: the first shot comes in tick *T* + 48, the first tick in which the turret is within 4096
    of 16383 (16383 − 260 × 15 = 12483), unless something outside this spec delays it, and in any
    case before the turret stops in tick *T* + 63. With the 2048 tolerance it would come in tick
    *T* + 56 at the earliest (16383 − 260 × 7 = 14563).
- *Falsified if:* run a ends at another facing, takes another number of ticks, or fires before
  tick *T* + 64; run b turns counter-clockwise or does not take 127 ticks; in run c the turret
  stays at 0 or turns another way; in run d the turret never returns, or returns before *G* + 5
  ticks; in run e the turret turns towards the target; in run f the turret moves away from 0 or
  the vehicle fires; in run g the first shot comes before tick *T* + 48,
  or only once the turret has stopped. A first
  shot in run g in tick *T* + 56 points to the 2048 tolerance for a homing projectile; scenario R11
  decides between the two. A run f in which the vehicle moves or its body turns is invalid.

**R5. No re-aim during a turn when the target is out of range (claims 8, 9 and 10 (a),
`quirks.turret_reaim_after_turn`).**
- *Setup:* as R4 (recorder, turn rate 1, body and turret at 0, a projectile that does not home).
  One own target stands 25 cells due map east and a second 25 cells due map west, on the vehicle's
  row, both out of weapon range.
- *Procedure:* order an attack on the east target, and 10 ticks later an attack on the west
  target. Run at least *G* + 5 ticks after game start, so that no rest row acts before the first
  order.
- *Observable:* the turret facing every tick; *T* is the first tick in which it changes.
- *Runs:* 2.
- *Expected:* the turret facing keeps growing (clockwise) until it reaches 16383 in tick *T* + 63.
  It is still 16383 in tick *T* + 64, when the new turn towards 49153 begins (claim 9: one tick
  of waiting; δ(16383, 49153) = −32766 gives Θ = 127 and β = −258 with no remainder). It falls
  (counter-clockwise, through map north) from tick *T* + 65 and reaches 49153 in tick *T* + 191.
  The body's own turns and movement do not matter: the turret facing is absolute (claim 8), and
  the vehicle, if it moves at all before then, moves along its row, so the direction of the west
  target stays 49153.
- *Falsified if:* the turret facing starts to fall within a few ticks after the second order (with
  the quirk off it would turn counter-clockwise from 2603 at once and reach 49153 in tick *T* + 84),
  or the reversal and arrival ticks differ from *T* + 65 and *T* + 191.

**R6. Spin speeds, without and with a target (claims 9, 10 (c) and 11).**
- *Setup:* a purpose-made flat map; a recorder that reads the turret facing every tick, whether the
  vehicle has a target and the ticks in which it fires. A named test rules file gives one stock
  vehicle type with a turret (and a weapon that is neither omni-fire nor turret-locked, with a
  projectile that does not home) the turret-spins flag and, in separate runs, the turn rates
  named below. For runs f to h the same file gives the own target's type a strength field high
  enough that the target survives the run; the own targets stand 5 cells from the test vehicle,
  within range.
- *Procedure:*
  - Runs a to e, turn rates 1, 3, 4, 5 and 9: place the vehicle and let it stand without orders.
    No enemy object stands within 30 cells of it, and the recorder must show that it never has a
    target.
  - Run f, turn rate 1: as run a, then order an attack on an own target due map east.
  - Run g, turn rate 127, three sub-runs: order an attack on an own target due map north (g1, its
    direction is 0), due map east (g2, 16383) or due map west (g3, 49153).
  - Run h, turn rate 1: as run f with the own target due map west.
- *Observable:* the change of the turret facing per tick; the number of ticks per full turn,
  counted as the ticks between two successive ticks in which the recorded facing is smaller than in
  the tick before (the turret has passed facing 0 clockwise); and in runs f to h whether and when
  the vehicle fires.
- *Runs:* 1 per rate or sub-run; at least 3 full turns each in runs a to f and h.
- *Expected:*
  - Runs a to e: steps of 256, 1024, 1024, 2048 and 2048 facing units per tick (once the facing is
    on a multiple of 256), so 256, 64, 64, 32 and 32 ticks per turn.
  - Run f: once the attack order acts, the turret goes on spinning clockwise, but its steps are
    256 facing units or more: many are larger (up to 466) and not multiples of 256, and every full
    turn takes exactly 233 ticks, whatever facing the turret had when the order acted.
  - Run g: from the first tick after the order acts at the latest, the turret holds still at 2048
    (g1), 18432 (g2) or 51200 (g3) in every tick while the target lasts. The vehicle fires in g1
    (the turret is 2048 from the target's direction) and g3 (2047), and never in g2 (2049).
  - Run h: as run f, with 230 ticks per full turn.
- *Falsified if:* rates 3 and 4 differ, rates 5 and 9 differ, or rate 1 is not 256 ticks per turn
  in run a. In run f or h, every step is 256 and a turn takes 256 ticks (the target rows did not act
  on the spinning turret, or the spin step ignored their trace), or a turn takes another number of
  ticks than 233 (f) or 230 (h). In run g, the turret keeps spinning (the spin step was taken from
  the facing before the tick's other rows), or in g3 it alternates between 51200 and 53248 (only
  the facing-error row acted, not the target rows), or the vehicle fires in g2 or never fires in g1
  or g3. A run in which the vehicle moves or its body turns is invalid and says nothing about these
  claims.

**R7. A new target during a turn (claims 1, 3, 9 and 10).**
- *Setup:* a purpose-made flat map; a recorder that reads body and turret facings every tick.
  Own targets stand 5 cells due map east and 5 cells due map west of the test vehicle, both within
  range. The test vehicle is pre-placed with map facing value 0.
  - Run a: a stock vehicle type without a turret whose weapon is not omni-fire, set to turn rate 1
    by a named test rules file.
  - Run b: the vehicle of R4 run g (a turret and a homing projectile, turn rate 1).
  - Run c: the vehicle of R4 run a (a turret and a projectile that does not home, turn rate 1).
- *Procedure:* order an attack on the east target, and exactly 32 ticks later an attack on the
  west target. Because both orders take the same number of ticks to act, the second acts 32 ticks
  into the first turn.
- *Observable:* the body facing (run a) or turret facing (runs b and c) every tick, from the first
  tick *T* in which it changes. In run a, also whether the vehicle moves.
- *Runs:* 2 each.
- *Expected:*
  - Runs a and b: the facing turns clockwise for 32 ticks (8063 in tick *T* + 31). In tick
    *T* + 32 the new turn starts from the interpolated value 8323: δ(8323, 49153) = −24706, so it
    lasts 96 ticks and begins with a jump to 8289 (F1), the value recorded in tick *T* + 32. The
    facing reaches 49153 in tick *T* + 128. In run a the vehicle does not move.
  - Run c: the turret first reaches 16383 in tick *T* + 63, waits one tick, and reaches 49153 in
    tick *T* + 191 (63 + 1 + 127).
- *Falsified if:* in runs a or b the facing reaches 49153 in tick *T* + 95 (a new turn measured from
  the start facing 0), or the facings after the reversal differ from F1; in run a the body does not
  turn at all, or the vehicle moves (which invalidates the run); in run b the turret finishes its
  first turn before reversing (claim 10 (b)); in run c the turret reverses before reaching 16383.

**R8. Placement sets body and turret at once (claim 5).**
- *Setup:* a purpose-made flat map; a recorder that reads body and turret facings every tick and
  whether the vehicle is on the map. The vehicle of R4 run a (turret, turn rate 1).
  - Run a: the vehicle is pre-placed with map facing value 64.
  - Run b: the vehicle is pre-placed with map facing value 0 on cell (*x*, *y*). A stock transport
    that carries vehicles, owned by the same player, stands on land on cell (*x*, *y* − 1), the
    neighbour to map north. An own target stands 5 cells due map south, within range.
- *Procedure:* run a: no orders, and record the first 100 ticks. Run b: order the attack on the
  own target, 10 ticks later order the vehicle into the transport (its body already faces it, so
  it drives without turning); as soon as it is inside, order the transport to unload.
- *Observable:* both facings in the first tick and every tick after; in run b, the facings in the
  first tick the vehicle is back on the map.
- *Runs:* 2 each.
- *Expected:* run a: body and turret are both 16384 from the first tick, and the turret does not
  turn. Run b: in the first tick back on the map, the turret facing equals the body facing (the
  placement facing that the unloading gives), whatever the turret pointed at before it went in. If
  a rest row of claim 9 gives the turret a new target in that very tick, the turret may already
  have begun a new turn and differ from the body by that turn's remainder jump (claim 1), which is
  at most 127 facing units at turn rate 1.
- *Falsified if:* in run a the turret starts at 0 and turns to 16384; in run b the turret differs
  from the body by more than 127 facing units in the first tick back, in particular if it shows the
  value it had when it went in or a value on its old turn towards 32767. If the body itself comes
  out within a sixteenth of a turn of 32767, repeat run b with the own target due map east instead.

**R9. What keeps an idle turret where it is (claim 9, rest-row conditions).**
- *Setup:* purpose-made flat maps; a recorder that reads the turret and body facings, whether the
  vehicle has a target and a movement destination, and the direction of the destination from the
  vehicle, every tick. Each run has an unarmed enemy vehicle within range at a direction away from
  the test vehicle's body facing.
  - Run a: a stock vehicle type with a turret sits inside a stock tank bunker.
  - Run b (control for a): the same vehicle on open ground next to the bunker.
  - Run c: a stock simple-deployer vehicle type with a turret, deployed in place.
  - Run d (control for c): the same type, not deployed, standing still.
  - Run e: a stock vehicle type with a turret and without numbered weapons, with its weapons
    removed by a named test rules file (so it has no turret weapon, which counts as not
    turret-locked, and never has a target), is ordered to move to a cell 10 cells to map east and
    3 to map south, so that the direction of its destination is not the 8-direction of any step
    of its path; while it drives, an enemy vehicle with a vehicle-seizing weapon takes hold of it
    and drags it off its line.
- *Procedure:* runs a to d: order the attack, let the target be destroyed, then record for
  *G* + 50 ticks. Run e: record from the move order until the vehicle is released.
- *Observable:* the turret facing over time, against the body facing and, in run e, against the
  direction of the movement destination.
- *Runs:* 2 each.
- *Expected:* runs a and c: the turret keeps the facing at which it last fired. Runs b and d: once
  *G* + 5 ticks have passed since the last shot, the turret turns to the body's facing. Run e: before
  the vehicle is held, its turret turns towards the direction of its destination, not to its body
  facing; while it is held, its turret facing does not change, although the direction of its
  destination does; after release it turns towards the destination again.
- *Falsified if:* in run a or c the turret turns to the body's facing; in run b or d it does not
  (which invalidates the pair); in run e the turret follows the body facing before the vehicle is
  held, or follows the direction of the destination while it is held. If the recorder shows that
  a held vehicle has no movement destination, run e tests the no-destination rest row instead
  and says nothing about the seized condition.

**R10. A building's turret turn rate (claim 7).**
- *Setup:* a purpose-made flat map; a recorder that reads a building's turret facing every tick. A
  named test rules file sets a stock defensive building type with a turret to turn rate 1. The
  building is pre-placed, and one unarmed enemy vehicle stands within its weapon range in a
  direction at least a quarter turn away from the building's starting turret facing.
- *Procedure:* let the building acquire the target on its own.
- *Observable:* the turret facing every tick: its start value *f*₀, the value *d* at which it
  settles, and the facings in between.
- *Runs:* 2.
- *Expected:* the turn from *f*₀ to *d* takes ⌊|δ(*f*₀, *d*)| / 256⌋ ticks, and every facing in
  between is the F1 value for ω = 1.
- *Falsified if:* the turn takes another number of ticks, the facing jumps to *d* at once, or the
  intermediate facings differ from F1.

**R11. Pointing tolerances (claims 9 and 10 (c)).**
- *Setup:* a purpose-made flat map; a recorder that reads the body and turret facings every tick and
  the ticks in which the vehicle fires. The vehicle of R4 run f (numbered weapons, every one of
  them turret-locked by the named test art file), set to turn rate 1 by a named test rules file.
  It is pre-placed with map facing value *v*, so that body and turret start at 256 × *v*. One own
  target stands due map north of it, within the range of its turret weapon; its direction is 0
  (S-0204, claim 8). The test rules file gives the projectile of every numbered weapon of the type
  a projectile turn-rate field of 0 in runs a and b (projectiles that do not home) and of 1 in runs
  c and d (homing projectiles), and gives the own target's type a strength field high enough that
  it survives the run.
- *Procedure:* order an attack on the own target and record for 300 ticks. Run a: *v* = 8. Run b:
  *v* = 9. Run c: *v* = 16, homing. Run d: *v* = 17, homing.
- *Observable:* the turret and body facings every tick; whether the vehicle fires.
- *Runs:* 2 each.
- *Expected:* in every run body and turret stay at 256 × *v* (the facing-error row's turn towards 0
  is replaced by the turret-locked row's target, and the replacing turn has Θ = 0). The vehicle
  fires in run a (|δ| = 2048) and run c (4096), and never in run b (2304) or run d (4352).
- *Falsified if:* the turret leaves 256 × *v*; the vehicle fires in run b or run d, or never fires
  in run a or run c. A shot in run b but not in run d, or none in run c, would show other
  tolerances than 2048 and 4096; a turret that turns towards 0 shows that the turret-locked row did
  not win (claim 9). A run in which the vehicle moves or its body turns is invalid and says nothing
  about the tolerances.

**R12. Vehicle rotation samples (claim 17).**
- *Setup:* the flat map, vehicle and own targets of R4, at game speed 1, with the same named
  test rules file and a bystander computer player kept outside the observed area. Add a named
  test sound file giving the vehicle a long looping rotation entry as in S-0701 F12. A read-only
  recorder logs tick number, turret facing, target facing, rotation samples and sound requests.
- *Procedure:* run a: order an attack with turn rate 1 and follow the turn through completion.
  Run b: repeat with turn rate 127, so this turn completes at once. Run c: use R6 run a's
  spinning-turret setup at turn rate 1. Run each twice; capture audio as in S-0701 F12.
- *Observable:* the rotation sample alongside the turn's duration and the sound request that
  uses it; distinguish requests from audible sample endings.
- *Expected:* run a's sample is turning from the tick that starts the positive-duration turn
  through the tick before its completion, and nonturning at completion. Run b never samples
  this instantaneous turn as turning. Run c samples turning despite the spinning turret's
  re-aim gate never counting it as turning. Sound requests follow the latest sample, and
  instance endings follow S-0701 claim 23 rather than cutting the current audio sample.
- *Falsified if:* a sample follows the previous-tick re-aim gate instead of current turning,
  an instantaneous turn supplies a turning sample, or spinning suppresses the sample. Void a
  run if another target or an unnamed bystander object changes the observed turn.

## Open questions

- The rest position of an idle turret (claim 9, rest rows): whether "last fired" counts every
  weapon's shot is not confirmed; scenario R4 run d uses only one weapon.
- Aircraft turning: the flying locomotor may set the body facing at once in some flight phases,
  the aircraft specs should say when the rate of claim 7 applies.
- Hovering and jumpjet headings (claims 13 and 14): how the heading turns into movement belongs to
  the movement specs of those locomotors, and no scenario here isolates the heading's rate.
- Which infantry facing changes go through the fixed rate 127 and which are set at once (claim 15,
  S-0107).
- Whether the unseen turret facing of a vehicle without a turret can drift away from the body
  facing (for example after the body is set at once while driving, claim 12). It no longer affects
  firing, which compares the body facing there (claim 10 (c)), but a type with the turret-spins
  flag and no turret flag (next item) still has the question.
- A type with the turret-spins flag but without the turret flag: claim 9 applies the spin row to
  it, but it is not stated whether the other rows of claim 9 act on its turret, whether the rule
  for a vehicle without a turret (claim 9) and claim 16 apply to its body, or, if both the spin row
  and that rule set its turret target in one tick, which target is in force at the end of it.
- The original's weapon-count field when the key is absent (Import mapping) has no established
  value; our engine uses 0 there, which gives the type no numbered weapons.
- The rest of the firing check (the order of its other results, and what else stops a shot) is the
  combat specs'; the scenarios here read facings directly where they can, and claim 10 lists the
  only properties they rely on.
