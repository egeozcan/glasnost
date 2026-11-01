---
id: S-0613
title: Gas clouds
topic: combat
revision: 13
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-01
measurements: []
requests: [Q-0662, Q-1240, Q-1241, Q-1242, Q-1244]
supersedes: []
---

# S-0613: Gas clouds

## Summary

How lingering gas clouds are made and how they behave in skirmish; campaigns and network games are out of scope. It covers the shared gas system that exists in
every skirmish, the poison clouds left by a virus death (clouds whose warhead is a poison warhead), the first particle of a particle system that a warhead creates, and the gas
particle itself: its drift, slope deflection, sinking, wind, lifetime, particle stage, successor and
the damage it deals at a fixed cadence. Gas damage is ordinary damage. What a hit then
does (immunities, poison, berserk, death kinds) belongs to the specs named below.

**What this spec does not restate.**
- **Which deaths are virus deaths,** and the death sequences: S-0107 claim 17 and S-0312 claim 8.
- **Poison immunity** and the rest of damage resolution: S-0302 (claim 8 for poison).
- **Creating a particle system** from a warhead: S-0303 claim 20. Claim 4 below adds the first
  particle.
- **Animation events** (start, middle, removal): S-0109 claims 13, 14, 16 and 17.
- **Creation draws** of a particle (its lifetime draw and the other conditional draws): S-0202
  claim 15.
- **Particle art lookup:** S-0007 claims 28 and 29.
- **Berserk** from a gas hit with a berserk-causing warhead: S-0302 claim 10 decides whether
  the hit takes effect and its duration; S-0610 claims 4 to 7 govern entry, re-hits, countdown,
  expiry, restrictions and tint. Whether the chaos drone's weapon names a particle system is outside
  this spec (Open questions).

## Terms

- **Particle system.** A world object that owns an ordered list of particles and takes one turn per
  tick (S-0203 claim 2). It has a *held type* (the particle type its warhead-made first particle
  has; see *Empty-named type*), a *behaviour kind*, an owner house (possibly none) and a *system lifetime* in ticks, where
  −1 means it never times out.
- **Empty-named type.** The particle type whose name is the empty string. A particle system type
  section read without a held-type key names it as its held type (S-0616 claim 21; the absent-key
  rule of S-0004 claim 2). It has no behaviour kind and the absent-key defaults of any particle
  type. A system type that is never read has no held type at all.
- **Gas particle.** A particle whose type has the gas behaviour kind. Its state is a position
  (integer leptons), an integer velocity (*v*ₓ, *v*ᵧ, *v*_z) in leptons per tick, a stored
  horizontal *deflection* (*d*ₓ, *d*ᵧ) in single-precision leptons, a remaining lifetime *E* in
  ticks, a damage countdown *C* in ticks, a particle stage *s*, a particle speed *S* (glossary, *particle speed*) and a dead mark.
- **Shared gas system.** The one particle system that every game has from its start, with no
  owner house (claim 1). Normally its type has gas behaviour; a missing type is created without
  any behaviour kind. Virus deaths put their clouds into it.
- **Particle stage.** A small signed counter of a particle that advances at a fixed cadence
  (claim 15). It selects the drawn frame and can end the particle. Not the resource stage of
  S-0401.
- **Successor.** The particle that replaces a dead particle whose type names a next particle type
  (claim 7). Which rule applies follows the kind of the system that holds the dead particle
  (S-0616 claim 3), not the particle's own kind: in a gas system claim 7's one successor, in a smoke system
  S-0616 claim 8's two, and in a fire, spark or railgun system none.
- **Object number.** S-0306 (Terms): the number our engine gives each object at creation from its
  own counter. A particle gets one like any other object.
- **Terrain height** *H*(*x*, *y*). The terrain surface height of S-0106 claims 2 and 3 at a
  horizontal position, including its in-cell slope, never a bridge deck.

## Behaviour

### Where gas comes from

1. **Shared gas system.** When a game starts, the engine creates the shared gas system from the
   particle-system type whose rules name is fixed by the engine (Import mapping). **When it
   enters play:** it is the last object of the game-start step to enter play, apart from the
   ore-twinkle animations below. The map is read
   first, so every map-placed object (S-0203 claim 3), the skirmish starting units and every
   tile animation or cell animation started while loading enter play before it, and the shared
   system is appended to the update order after all of them (S-0203 claim 3, appending). Only the
   ore-twinkle animations of the same game-start step, when the rules name one, come after it.
   So within a tick the shared system's turn, and with it every cloud hit and draw of claims 6
   to 15, comes after the turn of every map-placed object and of every starting unit that is
   still in play, and before any object created later (for example by a production or a
   death in play). It has no owner
   house, no target and the system lifetime of its type. Its own position is observable only through S-0616 claim 1's
   second trigger action, which marks every particle system in the cell of a waypoint, this system
   included when its position lies in that cell (claim 17). This spec does not give the position
   value: S-0616 states it as an engine constant and its scenario P1 decides it. Nothing else
   reads it.
   With an unlimited system lifetime it lasts until the game is cleared, unless a trigger action marks it (claim 17); it is never removed merely for being empty. A positive system lifetime instead marks it to end after that many turns (claim 17).
   When no type of that name exists after all rules layers, the original creates it at game
   start and reads no rules section, as S-0004 claim 3 states. The game is not rejected. This
   newly created type has unlimited system lifetime (−1 ticks), no behaviour kind and no held
   particle type. A section alone does not supply its properties if nothing registered the type
   during the rules layers. The shared system starts empty; with no behaviour kind it neither
   updates nor moves particles later delivered into it, so those particles do not deal gas damage
   or expire through claims 5 to 15. Those claims' gas-system turns require gas behaviour.
   - Backing: `measurable` (G1 for creation, ownership, target and empty persistence,
     G1-missing for the missing-type case and its defaults, G3 for finite system lifetime, G12 for
     its place in the update order). The shared system's position is not given here. Its value is decided by S-0616 P1 (part b, second run), which is `measurable`; no scenario of this spec varies it. The claim that nothing else reads the position is `unmeasured`: the position is an engine constant fixed at game start, so no setup can vary it independently, and the absence of other readers cannot be observed.
   - Confidence: high

2. **Virus death with the animation.** When an infantry unit dies a virus death that plays the
   rules' virus-death animation (S-0107 claim 17, the human row), the animation is created at the
   unit's position. If the damage had an attacker object, the animation's owner house is the
   attacker's house. With no attacker object, it instead takes the hit's supplied attacking house
   (S-0302 claim 1), or no house when neither is supplied. This ownership changes nothing in this
   spec: the clouds go into the shared gas
   system, which has no house. The particles requested by the animation's middle event (S-0109
   claim 14: its particle type, its particle count times) are all created in the shared gas system
   at the animation's position, *whatever their type's behaviour kind*.
   - Backing: `measurable` (G2 part a for animation position, ownership, shared-system
     membership and particle count; G2 part d for ownership without an attacker object)
   - Confidence: high

3. **Virus death without the animation's middle event.** In the two extra-animation cases of
   S-0107 claim 17 (an infantry type that is not human, and a parachutist killed by a virus death),
   the virus-death animation is created with no loop delay, so its start event happens at once
   (S-0109 claim 13: start sound, and the middle event too when its μ is 0). Then exactly one
   particle of the animation's particle type is created in the shared gas system at the unit's
   position, and the animation is removed in the same step, so it never reaches a later middle
   event. With μ ≠ 0 the animation's particle count therefore plays no
   part, and the death leaves one cloud. With μ = 0 the creation middle event's particles (and its
   scorch mark or crater, S-0109 claim 14) come first, then the one extra particle.
   The order of this extra virus effect relative to the other death effects is given by
   S-0615 claim 1.
   - Backing: `measurable` (scenario G2, part b: one cloud where part a gives the particle count).
     The μ = 0 case uses G2 part c's named test art file.
   - Confidence: high

4. **First particle of a warhead's particle system.** When a detonation creates a particle system
   (S-0303 claim 20), one particle of the system's held type, when named, is created in it at once,
   at the detonation point *P* (subject to claim 5's terrain floor). A system type section that is
   read without the held-type key names the empty-named type (Terms), so it does get a first
   particle, of that type (S-0616 claim 21). Only a system type whose section is never read has no
   held type and gets no first particle. This initial request holds for every system behaviour kind,
   not just gas.
   A system takes the lifetime of its type, which is −1 unless the rules give one. A gas-behaviour
   system with unlimited lifetime stays inert after its last particle is gone.
   Every system, whatever its kind, takes a turn each tick, and claim 17 gives what is common
   to all kinds: the system lifetime and when a system leaves play. How the smoke, fire, spark
   and railgun kinds emit and move their particles is given by S-0616 (claims 5 to 18).
   - Backing: `measurable` (scenario G3 for the first particle; scenario G13, part b, for the turns
     of each kind)
   - Confidence: high

5. **Creating a particle.** A new particle is added at the end of its system's list. Its remaining
   lifetime *E* is the type's lifetime field *E*ₘₐₓ **plus** one lifetime variation
   *u* (F9), which is the draw of S-0202 claim 15: *u* = |*r* rem *E*ₘₐₓ| for one raw value *r*
   (|*r* rem 10| for railgun behaviour, which a gas particle type does not have). For a positive
   field, before the reduction below, *E* lies in *E*ₘₐₓ … 2*E*ₘₐₓ − 1, and the default field of 1
   gives *E* = 1. *E* is kept as an unsigned 16-bit count: the sum is reduced modulo 65536. Its damage
   countdown *C* starts at its type's damage interval field reduced modulo 65536 and its stage at
   its type's start-stage field. A sum that reduces to zero expires by lifetime after 65536
   updates, not immediately, unless another cause removes it earlier. That needs a positive field
   of at least 32769 or a negative field of at most −65536, with a draw that makes the sum a
   multiple of 65536; the other way is a field of 0 with a raw value whose low 16 bits are zero. If its height is at or below *H* at its position, it is raised to *H*. Its velocity and
   deflection start at zero.
   - Backing: `measurable` (G4 and G5 for initial cadence, stage, motion and lifetime,
     S-0202 claim 15 for creation draws). The initial-zero lifetime's full 65536-update life is
     `unmeasured`: no practical setup here guarantees that creation draw without writing random state.
     The sum *E*ₘₐₓ + *u* (F9) is `measurable` by G5's lifetime comparison.
   - Confidence: high

### The system's turn

6. **Order within a gas system's turn.** In a gas-behaviour system's turn (S-0203 claim 2),
   each existing particle's damage and other update effects use its position before that tick's movement. For any two particles in the system, the newer particle's update effects, including any random draws, precede the older particle's effects. Each particle's own behaviour-kind update occurs in that tick, and its lifetime is reduced by one modulo 65536; a resulting value of 0 marks it dead. All existing particles' update effects finish before any surviving particle moves. Only particles not marked dead move during that tick. A dead particle disappears and may leave a successor (claim 7). When more than one particle dies in the tick, successor creation draws for the newer dead particle precede those for the older dead particle, and all successor creation follows the particles' update effects. Consequences:
   - A particle whose damage is due still hits in the tick when its lifetime runs out,
     then disappears in that tick.
   - A particle created by a death that a cloud causes during the update pass (claim 3) is not
     updated that tick, but it is moved in the second pass. One created later in the tick, for
     example by an animation's middle event, waits for the next turn.
   - A successor is neither updated nor moved in the tick it appears.
   - The system lifetime and the removal of an ended system come after both passes of the
     turn (claim 17).
   - Backing: `measurable` (G5 for lifetime, successor and turn timing; G6 for draw order).
   - Confidence: high

7. **Successor.** When a dead particle of any kind in a gas system has a type that names a next particle
   type, one particle of that type is created (claim 5, including its creation draws) at the dead
   particle's position plus the type's next-particle displacement (leptons, per axis). It then takes
   exactly two quantities from the dead particle, overwriting what its own creation gave it: the
   particle speed (S-0616 Terms, a single-precision value) and the integer velocity (*v*ₓ, *v*ᵧ, *v*_z).
   The gas-system line of S-0616 claim 3 gives this successor the dead particle's velocity only; that line
   is revised in the same round to give the speed as well, and the speed part of this claim depends on
   that revision.
   Nothing else is copied: not the horizontal deflection, the damage countdown, the lifetime, the particle
   stage, the translucency or the direction. A gas particle never reads its speed (gas motion is velocity and
   deflection only, claims 8 to 11), so for a dead gas particle the speed is whatever value it holds at that
   moment: its type's speed value from its own creation, or, if it was itself made as a successor, the speed
   it took from its parent under this claim; it matters only when the successor's type is of a kind that reads it
   (a smoke successor rises by it; S-0616 claim 7). A dead smoke particle in a gas system passes its own speed
   as it then stands (the value S-0616 gives for it, including the emission lowering of its claim 9) and its
   velocity; both are carried as they are, with no conversion between smoke and gas meanings. The successor's
   kind is that of the next type named. The system's kind decides, not the particle's: a smoke
   particle dying in a gas system gets this one successor, and a gas particle dying in a smoke system
   gets S-0616 claim 8's two (mixed-kind rule: S-0616 claim 3). Successors
   are created in the second pass, so their draws come after all the update draws of that turn.
   - Backing: `measurable` (scenario G5 for the gas-to-gas velocity, deflection and timing; scenario G5-speed for the speed part; S-0616 scenario P3, part c, for the
     smoke-in-gas and gas-in-smoke cases)
   - Confidence: high for the velocity, deflection and timing; medium for the speed part until G5-speed has run.

### Gas particle motion

8. **Random drift.** On even tick numbers only, each gas particle takes one scenario raw value
   (S-0202). With probability 1/8 (F2) it takes two more and adds −1, 0 or +1 to either *v*ₓ or
   *v*ᵧ, the second draw choosing which. The exceptional third raw value −2³¹ instead gives
   a step of −3 (F1/F2). After such a drift event, both *v*ₓ and *v*ᵧ are clamped to [−2, 2]
   leptons per tick. A gas particle
   therefore takes 1 or 3 raw values on each even tick and none on odd ticks.
   - Backing: `measurable` (scenario G6: cloud spread on flat ground and recorded draws).
     The −2³¹ exception is `unmeasured`: the recorder cannot force this exceptionally rare value,
     and G6's runs do not practically isolate it.
   - Confidence: high

9. **Vertical speed.** On even tick numbers: when the particle's height above the terrain
   (glossary, taken at its own position; a particle is never on a bridge deck) is more than 5 leptons, *v*_z goes down by 1, but not below −5. Otherwise a
   negative *v*_z is set to 0.
   - Backing: `measurable` (scenario G7)
   - Confidence: high

10. **Slope deflection.** On even tick numbers the particle also sets its deflection (F3). Over flat ground, or when nothing below stops it, the deflection is zero. When
    the point a short step below it (the *probe*, F3) is below the terrain, crosses a high bridge's
    deck height, or is within 150 leptons above the terrain in a cell holding an obstacle, the
    deflection follows F3. With gravity greater than −2 leptons per tick² it points downhill
    along the slope of the particle's cell; other gravity values use F3 without that guarantee.
    On each odd tick that stored deflection contributes to movement. With the downward probe,
    a cloud on a ramp therefore creeps downhill, unlike one on flat ground.
    - **Obstacles:** a building, except an inactive section of a laser fence and a vehicle-like
      building (S-0214 Terms); or, in a cell without a building, one of the three wall overlays of
      S-0306's constants (by position in the imported overlay list).
    - Backing: `measurable` (scenario G8)
    - Confidence: medium (downhill direction and the zero on flat ground: high. The exact
      magnitudes and the obstacle and bridge triggers: medium)

11. **Movement.** Each live gas particle's position after the update pass is determined by F5. A due wind push contributes on either parity of tick (claim 12). On even ticks the particle's height is unchanged and its horizontal velocity contributes no movement. On odd ticks, wind and stored deflection determine the horizontal position at which sinking is evaluated; sinking moves the particle toward 5 leptons above the terrain by no more than 2 leptons, and velocity determines its subsequent displacement. After an odd-tick move its final height is never below 5 leptons above the terrain at its final horizontal position. Even-tick wind movement applies no terrain-height clamp.
    - Backing: `measurable` (G6 and G7; G9 for even-tick wind without a height clamp)
    - Confidence: high

12. **Wind.** A gas particle type with a positive wind-effect field *W* is pushed one step of the
    rules' wind direction on every tick whose number is a multiple of ⌊10 / *W*⌋ (F6). The step is
    given by the table in Constants. With *W* above 10 the original fails on integer division by zero; our engine's
    S-0205 claim 3 departure permits a push only at tick 0. This matters only for data that gives a gas
    particle type a positive wind effect.
    - Backing: `measurable` (scenario G9, with a test rules file)
    - Confidence: high

### Gas damage

13. **Damage cadence.** Every tick, in its update, each gas particle's damage countdown *C* goes
    down by 1 (a 16-bit count, F7). When it reaches 0 and the type's damage field is non-zero, *C*
    restarts at the damage interval and the particle hits every object listed in the ground layer of
    its cell, in that layer's listing order (S-0209 claim 13), that is alive with health above 0. Each hit
    is ordinary damage (S-0302) of the type's damage amount with the type's warhead:
    - its distance is F7's *D*, from the particle to the object, horizontal only.
    - it has no attacker, it does not ignore defences, and passengers may escape as usual.
    - its attacking house (S-0302 claim 1) is the owner house of the particle's system, which is
      none for the shared gas system.
    The hit is not a detonation. The warhead's particle system, animations, area spread and other
    detonation effects of S-0303 do not happen. So a gas cloud never makes new particle systems,
    however its warhead is set.
    The ground layer is used whatever the particle's height, so an object on a bridge deck is never
    hit, even by a cloud level with the deck. An object removed by a hit does not stop the hits on
    the others listed after it. The first hit of a new particle comes on its update number
    equal to its damage interval reduced modulo 65536, with zero meaning 65536 updates. Its death
    kind, poison and other damage effects follow S-0302 and S-0107 claim 17. For a berserk-causing
    warhead, S-0302 claim 10 decides whether the hit takes effect and its duration; effective hits
    then follow S-0610 claims 4 to 7. A death it causes can create further clouds (claims 2 and 3).
    - Backing: `measurable` (scenario G4; G10 for house credit and no attacker).
      A completed zero-interval cycle is `unmeasured`:
      G4 tests its wrap and absence of early hits, but reaching update 65536 also needs the
      unforced initial-zero lifetime of claim 5, with no earlier stage deletion.
    - Confidence: high (cadence, ground layer, no detonation). Medium for *D*: it uses the lift of
      F7, which makes the distance smaller than the lepton distance.

14. **Credit.** Kills and damage by particles of the shared gas system credit no house: chain
    deaths from a virus cloud give no veterancy and no score to anyone, even though the virus-death
    animation itself may be owned by the attacker's house (claim 2). Particles of a warhead-created
    system credit the house that owned the detonation.
    - Backing: `measurable` (scenario G10)
    - Confidence: high

15. **Particle stage.** In each update, after the damage step, the particle stage advances by 1
    on the updates that F8 selects. For a positive stage-advance field, the period is that
    field in ticks, or one more tick when the particle's object number is odd; F8 also covers
    nonpositive fields. Then, whenever the particle stage equals the type's end-stage field, the particle
    is marked dead if its type has the delete-at-stage-limit flag; otherwise its particle stage restarts at
    0 (not at the start-stage field). The drawn image-frame number is the particle stage itself.
    No kind adds the type's start-frame field (S-0616 claim 25).
    - Backing: `measurable` (scenario G5: the successor's life ends at the stage limit,
      and S-0616 claim 25 for the other kinds' frame tests)
    - Confidence: high

16. **Drawing detail.** A particle whose type deals no damage is not drawn while the measured
    drawn-frame rate is below the active performance cutoff. This is not a test of the player's
    lowest detail setting. Damaging gas particles bypass this particular suppression. At the
    highest detail setting, a particle with translucency 25 or 50 uses that translucency,
    a stored signed translucency from 75 through 127 uses 75. At other detail settings it is drawn opaque.
    Translucency is reduced modulo 256 at creation. The value drawn is the particle's own stored
    translucency, which S-0616 claim 26 changes for smoke successors, smoke emission and the fire
    stage fields. The performance cutoff applies to particles of every kind, and at the lowest
    detail setting smoke and spark particles are not drawn at all (S-0616 claim 23).
    - Backing: `measurable` (scenario G11, visual; S-0616 claims 23 and 26 for the other kinds'
      detail suppression and stored translucency changes). The adaptive cutoff's settings import and
      defaults remain `unmeasured` here: those inputs have not been traced or independently
      varied, so no fixed cutoff is claimed (Open questions).
    - Confidence: medium

17. **System lifetime and leaving play.** This holds for every system, whatever its behaviour
    kind (also none). After the kind's own turn work (for a gas system: both passes of claim 6),
    the system lifetime, a signed integer count of turns, goes down by 1, whatever its value.
    The system is marked to end in the turn in which the result is exactly 0. Removal of the
    system's owner object (S-0616 claim 4) is a second, separate cause of the mark, for every kind
    and at any lifetime; it sets the mark at the moment of removal, not at the next count-down.
    Then, still in the same
    turn and after that count-down, a marked system that holds no particles leaves play. So:
    - A lifetime *N* ≥ 1 marks the system in its *N*-th turn. If it is empty by the end of that
      turn (its last particle was removed in that turn's second pass or earlier) it leaves play in
      that same turn; otherwise it leaves in the turn in which its last particle is removed.
    - A lifetime of 0 goes to −1 on the first turn and so never again equals 0: it never ends. Every
      negative lifetime, −1 included, likewise never ends (its count comes back to 0 only after
      2³² + *L* turns, *L* being the lifetime, once it has wrapped round: far beyond any game).
      Only a positive lifetime ever sets the mark through the count-down. Owner removal (above), a
      trigger action (S-0616 claim 1) and a let-go of the fire place while the system is held
      there (S-0616 claim 1, S-0301 claim 5) are the only other ways a system is marked here. A
      let-go marks the system that place holds, whatever its kind, a gas-kind system included.
      S-0616 claim 22 lists the marks of the other kinds.
    - A system marked by owner removal leaves play like any marked system: in the first turn,
      after that turn's count-down, in which it holds no particle. Removal in the middle of a tick
      therefore leaves an empty system in play until its own turn comes, in that same tick if its
      turn is still to come, else in the next tick.
    - A marked system that still holds particles keeps taking turns, and its particles keep moving
      and expiring under claim 6.
    - A system with no behaviour kind does no kind work, so its particles are never updated or
      moved and never die by lifetime; it counts down and is marked as above but, holding
      particles, never leaves play. One without particles leaves play like any other.
    - When the shared gas system leaves play (positive lifetime, or a trigger action marking it (S-0616 claim 1)), the game has no shared system
      for the rest of the play: it is not created again. The original then fails on the next virus
      death or the next middle event that requests a particle (it uses the missing system without
      a check). **Departure** (no switch: the original stops): our engine takes every draw of those
      particle requests (S-0202 claim 15) and creates no particle; the importer warns when the
      shared system's lifetime is positive.
    - Backing: `measurable` (scenario G13, parts a and b: the lifetime 1, 0, −2 and 3 runs, the
      same-turn removal, the systems of each kind, and the shared system's positive-lifetime
      departure). The owner-removal mark and its departure turn are `measurable` through S-0616's
      scenario P4 (a smoke system whose owner is removed), which falsifies a mark placed at a later
      count-down or a departure in another turn. The trigger-action mark of the shared system and its departure turn are `measurable` through S-0616's scenario P1 (part b, second run). For a gas-kind system the owner-removal rule is
      `unmeasured` here: no stock data path gives a gas system an owner object, so no scenario can
      vary it. The behaviour after removal is `unmeasured`: the original stops there, so there
      is no original behaviour to compare. The 2³² + *L* turns to the first return to 0 is
      `unmeasured`: no run here is that long. The fire-place let-go's mark is backed as S-0616
      claim 1's place-and-kind backing gives; for a gas-kind system it is `unmeasured`: no
      scenario puts a gas-kind system in the fire place.
    - Confidence: high (count-down placement, lifetime 0, negatives, same-turn removal, owner removal); medium
      for the no-behaviour-kind line, which follows from the kind work being absent; medium for the
      fire-place let-go's mark, as S-0616 claim 1 rates a let-go's mark of the system held in the
      fire place.

18. **Order inside one gas particle's update.** The countdown and hits, the stage step and the
    lifetime count-down all run on every tick. On even ticks the particle also runs its drift, its
    slope deflection and its vertical speed (claims 8 to 10); the deflection and the vertical speed
    read the particle's position and the terrain and take no draw. Those even-tick effects change
    only the particle's velocity and deflection, which move nothing until the second pass, so their
    order among themselves has no observable effect. What is observable: a particle's drift draws
    precede the draws of every death its own hits cause; its hits precede its own stage step
    (claim 15); and its lifetime count-down comes last and only sets the dead mark that the turn's
    second pass reads (claim 6).
    - Backing: `measurable` (scenario G13, part c: the draw order of a virus cloud whose hit kills a
      unit on an even tick)
    - Confidence: high

## Formulas

**F1. Signed magnitude.** For a raw value *r* read as a signed 32-bit integer (S-0202 Terms):
*m*(*r*) = |*r*|, except that *m*(−2³¹) = −2³¹ (the magnitude wraps). Remainders are truncated
toward zero, so their sign follows the dividend.

**F2. Drift** (even ticks; integer arithmetic with signed remainders truncated toward zero).
Let *v*ₕ = (*v*ₓ, *v*ᵧ) be the velocity before drift, *r*₁ the first raw value, and
*c*(*u*) = clamp(*u*, −2, 2). The cases below specify the resulting velocity and total drift
raw-value count. In the three-value cases, the raw values are *r*₁, *r*₂, *r*₃ in that order,
and *k* = (*m*(*r*₃) rem 3) − 1. Velocity components and *k* are in leptons per tick.

| Condition | Drift draw count | *v*ₕ′ |
|---|---|---|
| Low three bits of *m*(*r*₁) nonzero | 1 | (*v*ₓ, *v*ᵧ) |
| Low three bits zero; *m*(*r*₂) even | 3 | (*c*(*v*ₓ + *k*), *c*(*v*ᵧ)) |
| Low three bits zero; *m*(*r*₂) odd | 3 | (*c*(*v*ₓ), *c*(*v*ᵧ + *k*)) |

With *r*₃ = −2³¹, *k* = −3 before clamping the resulting velocity.

**F3. Deflection** (even ticks). Let *G* be the rules gravity (integer, 3 when absent) and
*v*⁰ = (0, 0, −2 − *G*) in single precision. Let *p* = (*x*, *y*, *z*) be the particle's position.
The probe is *q* = (*x*, *y*, *z* − 2 − *G*), in the particle's cell. Let *h* = *H*(*x*, *y*) and
the deck height *b* = *h* + 416.
- **Bridge crossing:** when the particle's cell is a high bridge's cell and either
  *z* ≥ *b* > *z* − 2 − *G* (downward crossing) or *z* < *b* ≤ *z* − 2 − *G* (upward crossing).
  The upward case requires gravity below −2 leptons per tick².
- **Obstacle:** without a crossing, when *h* ≤ *z* − 2 − *G* < *h* + 150 and the particle's cell holds
  an obstacle (claim 10).
- **Deflect** when there is a crossing, an obstacle, or *z* − 2 − *G* < *h*. Then

  > *w* = *S* · *M* · *Z* · *M*ᵀ · *S* · *v*⁰,  *S* = diag(1, −1, 1),  *Z* = diag(1, 1, −1)

  where *M* is the ramp matrix of S-0106 table C4 for the slope shape of the particle's cell (the
  identity for flat and saddle shapes). The vector is transformed from right to left, not by
  pre-multiplying the matrices. Matrix entries and each completed transformed vector component
  are single precision; intermediate arithmetic follows S-0205's floating-point mode. For a
  row's terms *a*ₓ, *a*ᵧ, *a*_z, the sums are grouped (*a*ᵧ + *a*_z) + *a*ₓ for x,
  and (*a*ᵧ + *a*ₓ) + *a*_z for y and z, with no fused operations. Single-precision stores
  round in that same mode. The stored deflection is (*w*ₓ, *w*ᵧ).
- **Otherwise** the deflection is (0, 0).

**F4. Vertical speed** (even ticks, integers): with *a* = *z* − *H*(*x*, *y*),
> *v*_z ← max(*v*_z − 1, −5) when *a* > 5, otherwise *v*_z ← max(*v*_z, 0).

**F5. Move** (every tick, integer leptons). From the position (*x*, *y*, *z*), with the wind push
(Δₓ, Δᵧ) when it is due (F6) and (0, 0) otherwise:
- **Even ticks:** (*x*′, *y*′, *z*′) = (*x* + Δₓ, *y* + Δᵧ, *z*).
- **Odd ticks:** let *x*₁ = trunc(*x* + Δₓ + *d*ₓ) and *y*₁ = trunc(*y* + Δᵧ + *d*ᵧ), where the
  integer sum and the single-precision deflection are added in the floating-point precision of
  S-0205 and truncated toward zero (`quirks.chop_rounding`). Let *h*₁ = *H*(*x*₁, *y*₁). Then

  > *z*₁ = *z* − clamp(*z* − *h*₁ − 5, 0, 2)
  >
  > (*x*′, *y*′) = (*x*₁ + *v*ₓ, *y*₁ + *v*ᵧ)
  >
  > *z*′ = max(*z*₁ + *v*_z, *H*(*x*′, *y*′) + 5)

**F6. Wind cadence.** With *W* the type's wind-effect field and *t* the tick number: the wind is due
when *W* > 0 and *t* rem ⌊10 / *W*⌋ = 0. For *W* > 10 the divisor is 0: the original fails, whereas our engine's S-0205 claim 3
departure makes the remainder *t*, so the wind is due only at *t* = 0. (Δₓ, Δᵧ) is the wind table entry for the
rules' wind direction (Constants).

**F7. Damage countdown and distance.** *C* is a 16-bit unsigned count: *C* ← (*C* − 1) mod 65536.
A hit is due when the result is 0 and damage is nonzero; only then is the countdown reset.
The imported interval is reduced modulo 65536. With a damage interval of 0, *C* wraps to 65535 on the first
update, so the particle hits once every 65536 updates. The distance passed to S-0302 is

> *D* = *L*(|*x*ₒ − *x*| + |*y*ₒ − *y*|)

where (*x*ₒ, *y*ₒ) is the object's position and *L* is the simulation lift of S-0409 F2, applied
to this horizontal sum as if it were a height. Its integer pixel result is used numerically as
S-0302's lepton distance, without unit conversion; its floating-point precision and truncation
are those of S-0409 F2.

**F9. Initial lifetime.** With *E*ₘₐₓ the integer lifetime field and *r* the raw value of the lifetime
draw (a signed 32-bit value, S-0202 Terms), the remainder signed and truncated toward zero:
*u* = |*r* rem *E*ₘₐₓ| (the remainder is taken first, so its magnitude is below |*E*ₘₐₓ| and the wrap
of F1 does not arise), and

> *E* = (*E*ₘₐₓ + *u*) mod 65536  (unsigned 16-bit result).

With *E*ₘₐₓ = 0 the original stops; our engine follows S-0205 claim 3 (remainder equal to the
dividend, so *u* = |*r*|, reduced modulo 65536) with its importer warning. Integers throughout.

**F8. Particle stage advance.** Let *n* be the particle's object number, *A* the type's stage-advance field,
*E*_max its integer lifetime field and *E* its unsigned 16-bit remaining lifetime before this
update's decrement. Integer sums use signed 32-bit wraparound (S-0205 claim 1). The
particle stage advances when

> (*n* + *E*_max − *E*) is divisible by (*n* mod 2) + *A*.

The particle stage is an 8-bit signed counter, wrapping from 127 to −128. The start-stage, end-stage and
stage-advance fields are stored as 8-bit signed values. With *A* = 0 and an even *n*, or
*A* = −1 and an odd *n*, the divisor is 0: the original stops, and our engine's advance
follows S-0205 claim 3 with its importer warning.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Drift chance | 1/8 | per even tick | F2 |
| Drift clamp | ±2 | leptons per tick | *v*ₓ, *v*ᵧ |
| Fall limit | −5 | leptons per tick | F4 |
| Hover height | 5 | leptons above the terrain | F4, F5 |
| Sink step | 2 | leptons per odd tick, at most | F5 |
| Probe depth | 2 + gravity | leptons | F3 |
| Obstacle band | 150 | leptons above the terrain | F3 |
| Bridge deck height | 416 | leptons above the terrain | F3, as in S-0102 |
| Wind cadence numerator | 10 | ticks | F6 |
| Wind table Δₓ by direction 0–7 | 0, 2, 2, 1, 0, −2, −2, −2 | leptons | Direction 3 is short (Edge cases) |
| Wind table Δᵧ by direction 0–7 | −2, −2, 0, 2, 2, 2, 0, −2 | leptons | |
| Damage countdown width | 16 | bits | F7 |
| Particle stage width | 8 | bits, signed | F8 |
| Gravity when absent | 3 | leptons per tick² | F3 |
| Particle lifetime default | 1 | ticks | Import mapping |
| Damage interval default | 0 | ticks | F7 wraparound |
| Damage amount default | 0 | health points per hit, before damage resolution | Import mapping |
| Stage advance default | 4 | ticks | F8 adds the object-number parity |
| Start and end stage defaults | 0 each | stage indices, dimensionless | Import mapping |

## State and ordering

| State | Event | Result |
|---|---|---|
| live | update: *C* reaches 0, damage ≠ 0 | hits the ground-layer objects of its cell; *C* restarts |
| live | update: particle stage equals the end stage, delete flag set | dead |
| live | update: particle stage equals the end stage, no delete flag | particle stage ← 0 |
| live | lifetime *E* reaches 0 after an update | dead |
| live | second pass | moves (F5) |
| dead | second pass | successor created if named (claim 7); removed |
| system, lifetime reaches 0 | any turn | marked to end |
| system, owner object removed | at the removal | marked to end |
| system, marked, no particles | any turn | removed |

Ordering consequences:
- In one tick, every particle of a system hits before any of them moves.
- Two clouds of one system in the same cell hit in newest-first order.
- A particle that dies at its stage limit still hits in that update if its countdown is due, since
  the damage step comes before the particle stage step.
- Draws within a system's turn: all update draws (claim 8), newest particle first, then the
  successor creation draws (claim 7), newest dead particle first. Deaths caused by the hits add the
  draws of their own sequences (S-0107, S-0202 claim 15) at the point of the hit.

## Edge cases and quirks

- **Short wind step** (`quirks.wind_southeast_short`, default on). Direction 3 pushes 1 lepton in *x*
  instead of 2, unlike its mirror direction 5. Off: Δₓ = 2 for direction 3. It is gas-only: smoke has its own wind table
  (S-0616 claim 7), so it does not apply to smoke. It affects only gas
  types with a wind effect.
- **Particle stage parity.** The original takes the parity in F8 from its own object identifier.
  Our object number (Terms) has a different sequence, so which clouds advance with period *A* and which with
  *A* + 1 can differ from the original. This is a known departure of the same kind as S-0306's
  missile phase. It decides how long a particle with the delete-at-stage-limit flag lives.
- **Particle requested before the shared system exists** (loading). The original creates the
  shared system in the game-start step, after the map and the starting units and before any
  ore-twinkle animation (claim 1). A tile animation or cell animation with μ = 0 that is created
  during loading and requests a particle (S-0010 claim 19, S-0109 claim 14) therefore meets no
  shared system, and the original stops.
  **Departure** (no switch): our engine takes the particle's creation draws (S-0202 claim 15) in
  the same order and creates no particle; the importer warns when such an animation exists.
  The engine must not create the system early merely to receive these particles: claim 1 fixes
  its place in the update order.
- **Missing shared gas type.** Creation without a rules-section read is original behaviour,
  not a departure: S-0004 claim 3 and claim 1 here apply. Missing the type is not a reason for
  importer rejection or a warning. Without a behaviour kind, the resulting system cannot run
  gas updates even when a virus death delivers gas particles into it.
- **Virus-death animation without a particle type.** The original reads outside its type list in
  claim 3 (and claim 2 creates nothing), so its result is undefined. This is a departure: our
  importer warns, and claim 3 creates no particle. There is no quirk switch.
- **Wind direction outside 0–7,** including the absent default −1. The original reads outside its
  table, so its result is undefined. When a type with a wind effect exists, this is a departure:
  our importer warns, and no wind push is applied. There is no quirk switch.
- **Non-gas particle types in the shared gas system** (claim 2). Such a particle updates and moves
  by the rules of its own behaviour kind, not its system's (S-0616 claim 3 gives each kind's
  update and move rules). Its remaining lifetime goes down by 1 per update and sets its dead mark at 0, as in claim 6
  (S-0616 claim 3 gives the same rule for every kind); only the turn order of claim 6 comes from
  its system. This spec makes no claim about whether the data supplies such a particle.
- **Signed magnitude** (F1): an extreme raw value gives a drift step of −3 before the clamp.
  F2 clamps the resulting velocity, not the step itself.
- **Damage interval 0** (F7): one hit per 65536 updates rather than one per tick.

## Import mapping

| Concept | Source | Parse form | Default when absent |
|---|---|---|---|
| Shared gas system type | Particle-system type named `GasCloudSys`; registered through `[ParticleSystems]` or a creating reference (S-0004 claims 2, 3 and 13); its rules section supplies properties only on a section read | type name, fixed by the engine | created at game start with no section read: lifetime −1 ticks, no behaviour kind, no held particle type (claim 1) |
| System held type | particle-system type `HoldsWhat` | particle type name | the empty-named type (Terms) when the section is read; none when the section is never read |
| System behaviour kind | particle-system type `BehavesLike` | name: Smoke, Gas, Fire, Spark, Railgun | none (no update) |
| System lifetime | particle-system type `Lifetime` | integer, ticks | −1 |
| Particle behaviour kind | particle type `BehavesLike` | name: Gas, Smoke, Fire, Spark, Railgun | none (no update) |
| Damage interval | particle type `MaxDC` | integer, ticks | 0 |
| Lifetime | particle type `MaxEC` | integer, ticks | 1 |
| Damage amount | particle type `Damage` | integer, health points per hit before damage resolution | 0 health points |
| Warhead | particle type `Warhead` | warhead name | none |
| Start frame (not added for any kind's drawing) | particle type `StartFrame` | integer frame index, dimensionless | 0 |
| Start stage | particle type `StartStateAI` | integer stage index, dimensionless; reduced to signed 8-bit | 0 |
| End stage | particle type `EndStateAI` | integer stage index, dimensionless; reduced to signed 8-bit | 0 |
| Stage advance | particle type `StateAIAdvance` | integer, ticks; reduced to signed 8-bit | 4 ticks |
| Delete at stage limit | particle type `DeleteOnStateLimit` | boolean | no |
| Next particle type | particle type `NextParticle` | particle type name; `<none>` for none | none |
| Next-particle displacement | particle type `NextParticleOffset` | three integers, leptons | 0, 0, 0 |
| Wind effect | particle type `WindEffect` | integer cadence divisor, dimensionless | 0 |
| Translucency | particle type `Translucency` | integer percentage points, reduced to signed 8-bit; claim 16 selects blending | 0 percentage points |
| Particle art | particle type `Image` | S-0007 claim 28 | |
| Warhead's particle system | warhead `Particle` | particle-system type name | none |
| Virus-death animation | Rules `[General]` `InfantryVirus` | animation name | |
| Animation's particle type and count | art `SpawnsParticle`, `NumParticles` | name, integer | S-0109 |
| Wind direction | Rules `[General]` `WindDirection` | integer table index 0–7, dimensionless | −1 (Edge cases) |
| Gravity | Rules `[AudioVisual]` `Gravity`, as in S-0306's Import mapping | integer, leptons per tick² | 3 leptons per tick² |

## Measurement scenarios

All under the measurement setup (glossary): stock data except explicitly named test files,
engine add-ons and unrelated mods off, skirmish, game speed index 2, 800×600, a small
flat purpose-made map unless stated. Use one bystander computer player kept away from the
observed cells under the glossary's void rule. Run each deterministic variant three times,
statistical variants name their run counts below. Record the original's random state rather
than setting its seed. A read-only recorder logs tick numbers, particle positions, visible
stages, creation/removal events and health/house counters; use screen captures for art tests.
Every expected result below is falsified by a contrary recorded result, including any tick,
position, count or draw mismatch. Named test rules/art files are removed after each variant.

- **G1. Shared system exists.** Two houses: a virus sniper and one enemy human infantry, on a map
  with no other gas source. Kill the infantry. *Observable:* clouds appear and damage nearby
  infantry. Repeat a kill after all earlier clouds have gone: clouds still appear. G3's finite
  lifetime variant separately tests expiry rather than assuming all systems are immortal.
  At game start and after each cloud disappears, also record particle-system existence,
  owner and target with the read-only recorder. Expected: the shared system already exists
  before the first death, has neither owner nor target, and persists empty between deaths.
  *Falsified if* either kill produces no clouds, the system first appears at a death, acquires
  an owner or target, or disappears merely for being empty.
- **G1-missing. Missing shared type.** Named test rules file G1-missing removes the fixed
  shared-system type from every type list and creating reference in every applied rules layer.
  Keep the stock virus-death animation and its particle types. Run once with its section absent,
  and once with an otherwise unreferenced section naming gas behaviour, a held particle type
  and a short positive system lifetime. Use G1's houses and kill procedure. Record the type's
  existence, behaviour kind, held type and lifetime at game start, shared-system existence,
  particle creation, positions, remaining lifetime and nearby infantry health for 200 ticks.
  Expected in both variants: a created type with no behaviour kind, no held type and lifetime
  −1 ticks; an initially empty ownerless shared system that persists; virus-death particles
  arrive but receive no system updates or movement, no lifetime decrement and no gas damage.
  Registration during the rules layers must be absent in both variants, not merely the type-list
  entry: any surviving creating reference invalidates the test. Startup failure, properties
  from the unreferenced section, particle updates or gas damage falsify claim 1. Repeat each
  variant three times under the common setup. A listed-type control must recover G1/G4 behaviour.
- **G2. Cloud count.** (a) Kill a human infantry with the virus sniper: count the clouds at the
  animation's middle frame. Also record the animation's initial position and owner and each
  particle's creation position and system membership. Expected: the animation begins at the
  victim's position, belongs to the sniper's house, and requests the imported particle count
  at its own position in the existing ownerless shared system. A different position, owner,
  membership or count falsifies the corresponding rule. (b) Kill an attack dog (a
  non-human infantry type), and separately a parachutist still in the air, with the virus sniper.
  Expected: exactly one cloud, at the moment of death, and the animation disappears at once.
  *Falsified if* (b) shows the particle count or the animation persists. (c) Named test art file
  G2-zero-middle: give the virus-death animation a one-frame sprite, no loop delay and a positive
  particle count other than one, so μ = 0. Repeat (b): expect the middle-event count plus one,
  in that creation order. Repeat with a smoke-behaviour particle: it still enters the shared
  system and subsequently follows smoke motion, not gas motion. (d) Named test rules file
  G2-house uses G10-owned's warhead-created gas system, with its particle warhead selecting a
  virus death, to kill ordinary human infantry. Record the killing hit's attacker and attacking
  house, the death animation's owner and the spawned clouds' system owner. Expected: no attacker
  object, the source system's house as attacking house and animation owner, but no owner for the
  shared system receiving the new clouds. Contrast a virus chain death from the ownerless shared
  system: neither the hit nor its death animation has a house. Any different ownership falsifies
  claim 2's fallback; later shared-system cloud kills must still credit no house (claim 14).
- **G3. Warhead first particle.** Test rules file: give a test weapon's warhead the shared gas
  system's type as its particle system. Fire once at empty ground. Expected: one cloud at the
  impact point in the detonation tick (raised to terrain if needed), and none appearing later
  from that gas system, with the system's section giving a held type. Named test rules file
  G3-system varies the held type between named, key absent from a section that is read, and a
  system type with no section at all, and the system kind across the imported kinds: a named type
  and an absent key each give exactly one initial particle (of the named type, respectively of the
  empty-named type) before any kind-specific emissions; a system type with no section gives none.
  *Falsified if* an absent key in a read section gives no initial particle, or a never-read system
  type gives one. For gas,
  compare unlimited lifetime with a short positive lifetime: the positive case is marked at
  that turn count, survives while particles remain, and disappears when empty; unlimited stays
  empty until game clear. Record system existence as well as particles.
- **G4. Damage cadence.** The virus sniper kills one human infantry that shares its cell with a
  second enemy infantry ordered to stay. Record the tick numbers of the second unit's health
  changes and of the clouds' creation and updates. Expected: hits every *I* particle updates,
  where *I* is the cloud type's damage interval reduced modulo 65536 (zero means 65536),
  starting on update *I*, one hit per due cloud in the cell. A creation tick is not counted as
  an update when claim 6 defers that particle's first update. Variant (a): the kill on the ground under a
  high bridge, with infantry on the deck above. Expected: those on the deck are never hit.
  Variant (b): two enemy infantry in the cloud's cell, the first killed by a hit. Expected: the
  second is still hit in the same tick. Named test rules file G4-distance: use one damaging
  cloud, a long lifetime and no stage deletion, an interval of 1, and stationary targets at
  several horizontal separations within its cell. Compare health changes to S-0302 using F7,
  not Euclidean or unscaled lepton distance. Give its warhead conspicuous detonation animation
  and particle-system effects: cloud hits must cause neither. Also test interval zero for
  a recorded lifetime shorter than 65536 updates, and a damaging hit on the particle's
  final lifetime update. Record the damage countdown as well as health: the zero interval
  wraps to 65535 on its first update and gives no hit before removal; final-update damage
  with a positive interval occurs before removal. A harmless variant must not reset its
  countdown on zero. Repeat with an interval above 65535: predict its initial countdown and
  first hit from reduction modulo 65536, counting particle updates rather than elapsed ticks.
  Named test rules file G4-defences supplies otherwise identical gas hits on standing and
  prone infantry, a poison-immune target, allied and enemy targets of the same type, and
  a transport with passengers. Expect ordinary S-0302 damage using the particle amount,
  warhead and F7 distance, including defences and immunity. The no-attacker hit is not
  suppressed merely by an allies-excluding warhead. On a killing transport hit passenger
  escape follows S-0312 claims 5 and 12 rather than being forcibly disabled. Any contrary health,
  hit order, countdown, detonation effect or forced no-escape result falsifies that part.
- **G5. Lifetime and successor.** Watch one stock virus cloud until its successor ends in 20
  runs. Record each particle's creation tick, position, stage and removal tick and the random
  state used for its lifetime draw. Expected: disappearance and successor creation share a
  tick; the successor position is the old position plus the imported displacement, floored at
  terrain, and it inherits velocity but not horizontal deflection. Its first update and motion
  wait until the next system turn. Compute its stage advances from F8, including the creation
  phase, rather than assuming a full period before its first advance. For oracle predictions,
  record the original's object identifier and use it in F8; engine conformance instead uses
  our object number, as the stated numbering departure requires. End occurs on the first
  lifetime-zero or deleting-stage-limit update. Named test rules file G5-stages varies start
  stage, end stage, delete flag and advance period; without deletion, equality resets to zero,
  not the start stage. Include start stage 127, no deletion and an unreachable end stage for
  the observed interval: the next advance wraps to −128. Compare imported stage values 128
  and −128, and advance periods 255 and −1: each pair must give identical signed-byte values
  and F8 timing for the same identifier parity. With advance −1, even identifiers advance
  every update; odd identifiers give division by zero. An advance field of zero with an even
  identifier likewise stops the original on its
  update; our engine must instead apply S-0205 claim 3, not be compared with continued oracle
  play. Compare each particle's removal tick with F9 (the field plus the drawn variation, not the variation
  alone), and vary the lifetime field above 65535 and compare the recorded creation draw and
  remaining lifetime with reduction modulo 65536, without assuming an initial zero will occur.
  Capture frames while varying the start-frame field: the selected frame
  must remain the stage itself. Include an odd-tick
  virus chain death: its directly spawned particle moves immediately but has no update until
  the next turn; a middle-event particle created after the system turn waits for the next turn.
  Include a nonzero successor displacement and a parent dying where deflection is nonzero.
  Wrong phase, inherited deflection, immediate successor motion or a wrong frame falsifies the claim.
  Named test rules file G5-speed gives a gas-kind system a virus-cloud type whose next particle type is of smoke kind, and gives the cloud's type a speed field different from the smoke type's. The cloud is killed with its damage countdown part-run and with a translucency different from the next type's. Record each particle's speed (as the recorder logs it), velocity, damage countdown and translucency, the smoke successor's rise per tick, and the draws taken for its creation. Expected: the successor's speed equals the dead gas particle's speed and its velocity equals the dead particle's velocity; its rise follows S-0616 claim 7 from that inherited speed, not from the smoke type's own speed field; its countdown and translucency start at its own type's values; and its creation draws are taken as claim 5 gives. Repeat with a gas parent that is itself a successor of a smoke particle: its speed must be the inherited smoke speed, not its own type's field. Three runs per variant. Falsified if the rise follows the smoke type's own speed field, the velocity or speed is not the parent's, the countdown or translucency is copied from the parent, or the draw count differs.
- **G6. Flat spread.** One stock virus cloud on flat open ground. Record its position every 100
  ticks, and its position change per tick for the first 200 ticks, over 20 seeds. Expected: no
  position change on even ticks, a change of at most 2 leptons per axis on odd ticks, a random walk
  overall. Use the random-state recorder over the same 20 runs to predict every F2 choice:
  one drift draw on even ticks, plus exactly two on a successful chance test; none on odd
  ticks. A named test art file G6-pair creates two distinguishable gas particles in the shared
  system at known different times: their observed drift must match assigning the newer particle
  its draws first, and successor creation draws must follow all update draws. Include
  a lifetime-ending even tick, which still consumes that particle's update draws.
- **G7. Settling.** A parachutist killed in the air by the virus sniper (G2 b). Expected: the cloud
  sinks to 5 leptons above the terrain and never goes below it. Record every tick: the vertical
  trajectory must equal F4/F5, including no vertical displacement on even ticks and the initial
  zero velocity. Named test rules file G7-floor creates a particle below terrain: creation
  raises it to terrain, then an odd-tick move raises it to at least terrain plus 5.
- **G8. Ramp drift.** The virus sniper kills an infantry on a straight ramp cell, once for each
  slope shape of S-0106 that a test map can hold, and once on flat ground. Expected: on the ramp the
  cloud moves steadily downhill; on flat ground there is no steady drift. Variant: a kill beside a
  building or a wall. Named test map G8-probes and test rules file G8-gravity add clouds at
  selected heights on ramp cells with a building, an inactive laser-fence section, a
  vehicle-like building, each of the three eligible overlays, and a high bridge. Compare
  per-tick motion with F3/F5 and paired obstacle-free controls. Test heights immediately on
  either side of the 150-lepton band and deck-crossing inequalities, plus negative gravity
  for the upward crossing. A wrong direction, magnitude, trigger boundary or deflection on
  the flat control falsifies the corresponding part of claim 10.
- **G9. Wind.** Test rules file: a cloud type with wind effect 1 to 10 and wind direction 0 to 7.
  Record the position every tick. Expected: a push of the table step every ⌊10 / *W*⌋ ticks.
  Direction 3 moves 1 lepton in *x* per push. Repeat over a ramp with an even-tick wind push
  toward higher terrain: even ticks preserve absolute height without a floor clamp, and the
  next surviving odd-tick move restores F5's floor. A height change on the even tick falsifies
  claim 11. Include W = 0 (no pushes) and W above 10:
  the original's integer division fails when movement is reached; our engine instead follows
  S-0205 claim 3's remainder departure, so only tick 0 can push. Do not claim this safe result
  as a measurement of continuing original play.
- **G10. Credit.** The virus sniper kills one infantry in a group of five enemies standing
  together, and the clouds kill the rest. Record the sniper's veterancy and each house's kill
  count. Expected: only the first kill is credited. Named test rules file G10-owned gives a
  warhead a gas system whose particle kills the same targets: the source house gets the kill
  credit, but with no attacker object no firer gains veterancy from those particle kills.
- **G11. Drawing detail.** Named test rules file G11-drawing supplies otherwise identical
  damaging and harmless clouds. Compare light-load and heavily populated maps while recording
  the measured frame rate and the active cutoff. Below the cutoff, harmless clouds disappear
  but damaging clouds bypass this gate; at or above it both pass this gate. Vary the player's
  detail setting separately. At highest detail compare translucencies 0, 25, 50, 75 and 80
  against the same background: 80 matches 75, and the others match their selected fraction.
  Also compare imported translucencies 127, 128, 255 and 281: expect respectively
  75-percent, opaque, opaque and 25-percent drawing at highest detail. At lower detail all
  these gas clouds use opaque drawing when otherwise visible. Wrong suppression, blending
  fraction or signed-byte reduction falsifies the corresponding part of claim 16.

- **G12. Place in the update order.** A skirmish map with a building that fires at an enemy
  infantry unit every tick it can, and a virus sniper that kills a second infantry unit so that
  the cloud hits in the same tick. Use the recorder to log the random state at each object's turn.
  Expected: the shared system's draws (claim 8's drift draws) come after the last map-placed
  object's and the last starting unit's turn and before the turn of a unit built after the start.
  Variant: the test art file G2-zero-middle on a tile animation at loading, with a gas particle
  type. The original stops at that request (Edge cases and quirks), so on the original this variant
  records only the stop. Our engine's observable is no cloud and the request's creation draws taken
  at loading. *Falsified if* the shared system's draws precede any map-placed object's turn, or the
  loading-time request makes a cloud in our engine.
- **G13. System lifetime, leaving play and draw order.** Each part runs three times. Named test
  rules files add test particle-system types, which are not the shared system's type, so that the
  shared system stays as the game gives it. Test types have a particle type whose lifetime field is
  30 ticks and no damage, unless a part says otherwise. A warhead fires once at empty ground, as in
  G3.
  (a) *Lifetime.* Test rules file G13-life gives the warhead a test gas system with lifetime 1, then
  with 0, then with −2, then with 3, one variant each, plus a fifth variant with lifetime 1 whose
  particle field is 1 tick. Record every tick for 60 turns: whether the system exists, its lifetime
  count, whether it is marked to end, and its particles. *Expected:* with lifetime 1 the system is
  marked in its first turn and leaves play in the turn its last particle is removed (about 30 ticks
  later); in the fifth variant it leaves play in its first turn, since it is empty by then. With
  lifetime 3 it is marked in its third turn and stays until its particles are gone. With lifetime 0
  or −2 it is never marked within the 60 turns. *Falsified if* a system is marked or removed in a
  turn other than claim 17 gives, or a lifetime 0 or −2 system is marked in these runs.
  (a′) *Shared system departure.* Test rules file G13-shared registers the shared system's type with
  lifetime 3. The shared system exists from game start and is empty. *Expected:* it is marked in its
  third turn and leaves play in that turn. A virus death after that makes no cloud, and the original
  stops there; our engine makes no cloud and takes the request's creation draws. *Falsified if* the
  shared system leaves play in any other turn, or a request after its removal makes a cloud in our
  engine.
  (b) *Kinds and no behaviour kind.* Test rules file G13-kinds gives a warhead a test system of each
  behaviour kind (smoke, fire, spark, railgun, gas) with lifetime 1 and a particle field of 1 tick,
  and a further test system with no behaviour kind, lifetime 1 and a held particle type. Emissions of
  the smoke, fire, spark and railgun kinds are S-0616's (G13 checks only the common rules), so for those four count only each
  system's existence and marks, not its particles. *Expected:* each system is marked in its first
  turn; for the gas system it leaves play in that turn once it holds no particle (the other four
  kinds' departure is checked by S-0616's scenarios P1 and P14). The no-kind system keeps its held
  particle, which neither moves nor expires by lifetime, so it never leaves play while the particle
  exists. *Falsified if* a no-kind system's particle moves or expires, a no-kind system leaves play
  while it holds a particle, or a system is marked or removed in a turn other than claim 17 gives.
  (c) *Draw order.* A virus cloud, on an even tick, whose hit kills a unit (the G12 recorder).
  *Expected:* its drift draws precede the draws of the death its hit causes, and its stage step
  follows the hit. *Falsified if* the death draws come before the drift draws, or the stage step
  comes before the hit.

## Open questions

- **Chaos drone.** A chaos-drone weapon that creates an animation and an area berserk effect but
  names no particle system leaves no gas particles under claim 4. S-0610 should not rely on this
  spec to assert whether the chaos drone's weapon names a particle system.
- **Adaptive drawing cutoff.** Claim 16 establishes the suppression test, but the settings and
  complete tuning of the shared performance-cutoff mechanism are not specified here; their
  import mapping remains untraced. This omission is `unmeasured` in this draft because no
  scenario yet varies those settings independently; it must not be treated as a fixed lowest-detail rule.
- **Other creators of gas.** The original also has a leftover monster type, absent from the stock
  data, that puts gas into the shared system. It is out of scope here.

## Questions for the project lead

- Should the importer warn when other data gives a non-gas particle to the shared gas system
  (claim 2)?
