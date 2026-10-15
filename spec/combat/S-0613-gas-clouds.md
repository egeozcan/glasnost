---
id: S-0613
title: Gas clouds
topic: combat
revision: 1
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0613: Gas clouds

## Summary

How lingering gas clouds are made and how they behave in skirmish; campaigns and network games are out of scope. It covers the shared gas system that exists in
every skirmish, the poison clouds left by a virus death (clouds whose warhead is a poison warhead in
the stock data), the first particle of a particle system that a warhead creates, and the gas
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
- **Berserk** from a psychic gas hit: S-0610. With the stock data the chaos drone's weapon names no
  particle system, so the stock chaos drone leaves no gas particles (Open questions).

## Terms

- **Particle system.** A world object that owns an ordered list of particles and takes one turn per
  tick (S-0203 claim 2). It has a *held type* (the particle type its warhead-made first particle
  has), a *behaviour kind*, an owner house (possibly none) and a *system lifetime* in ticks, where
  −1 means it never times out.
- **Gas particle.** A particle whose type has the gas behaviour kind. Its state is a position
  (integer leptons), an integer velocity (*v*ₓ, *v*ᵧ, *v*_z) in leptons per tick, a stored
  horizontal *deflection* (*d*ₓ, *d*ᵧ) in single-precision leptons, a remaining lifetime *E* in
  ticks, a damage countdown *C* in ticks, a particle stage *s* and a dead mark.
- **Shared gas system.** The one gas particle system that every game has from its start, with no
  owner house (claim 1). Virus deaths put their clouds into it.
- **Particle stage.** A small signed counter of a particle that advances at a fixed cadence
  (claim 15). It selects the drawn frame and can end the particle. Not the resource stage of
  S-0401.
- **Successor.** The particle that replaces a dead particle whose type names a next particle type
  (claim 7).
- **Object number.** S-0306 (Terms): the number our engine gives each object at creation from its
  own counter. A particle gets one like any other object.
- **Terrain height** *H*(*x*, *y*). The terrain surface height of S-0106 claims 2 and 3 at a
  horizontal position, including its in-cell slope, never a bridge deck.

## Behaviour

### Where gas comes from

1. **Shared gas system.** When a game starts, the engine creates the shared gas system from the
   particle-system type whose rules name is fixed by the engine (Import mapping). It has no owner
   house, no target and the system lifetime of its type. Its own position has no observable effect.
   With an unlimited system lifetime it lasts until the game is cleared and is never removed merely
   for being empty. A positive system lifetime instead marks it to end after that many turns (claim 6).
   A rules set without that type cannot be played: the importer rejects it (Edge cases).
   - Backing: `measurable` (scenario G1: virus clouds appear in a game whose map has no gas source)
   - Confidence: high

2. **Virus death with the animation.** When an infantry unit dies a virus death that plays the
   rules' virus-death animation (S-0107 claim 17, the human row), the animation is created at the
   unit's position. If the damage had an attacker object, the animation's owner house is the
   attacker's house. This ownership changes nothing in this spec: the clouds go into the shared gas
   system, which has no house. The particles requested by the animation's middle event (S-0109
   claim 14: its particle type, its particle count times) are all created in the shared gas system
   at the animation's position, *whatever their type's behaviour kind*.
   - Backing: `measurable` (scenario G2, part a: the number of clouds equals the animation's particle
     count)
   - Confidence: high

3. **Virus death without the animation's middle event.** In the two extra-animation cases of
   S-0107 claim 17 (an infantry type that is not human, and a parachutist killed by a virus death),
   the virus-death animation is created with no loop delay, so its start event happens at once
   (S-0109 claim 13: start sound, and the middle event too when its μ is 0). Then exactly one
   particle of the animation's particle type is created in the shared gas system at the unit's
   position, and the animation is removed in the same step, so it never reaches a later middle
   event. With μ ≠ 0 (as in the stock data) the animation's particle count therefore plays no
   part, and the death leaves one cloud. With μ = 0 the creation middle event's particles (and its
   scorch mark or crater, S-0109 claim 14) come first, then the one extra particle.
   - Backing: `measurable` (scenario G2, part b: one cloud where part a gives the particle count).
     The μ = 0 case uses G2 part c's named test art file.
   - Confidence: high

4. **First particle of a warhead's particle system.** When a detonation creates a particle system
   (S-0303 claim 20), one particle of the system's held type, when named, is created in it at once,
   at the detonation point *P* (subject to claim 5's terrain floor). Without a held type no first
   particle appears. This initial request holds for every system behaviour kind, not just gas.
   A system takes the lifetime of its type, which is −1 unless the rules give one. A gas-behaviour
   system with unlimited lifetime stays inert after its last particle is gone. Other system
   behaviour kinds may emit further particles and are outside this spec.
   - Backing: `measurable` (scenario G3)
   - Confidence: high

5. **Creating a particle.** A new particle is added at the end of its system's list. Its remaining
   lifetime *E* is drawn from its type's lifetime field as S-0202 claim 15 sets. Its damage
   countdown *C* starts at its type's damage interval field reduced modulo 65536 and its stage at
   its type's start-stage field. The lifetime is also reduced modulo 65536; an initial zero expires
   after 65536 updates, not immediately. If its height is at or below *H* at its position, it is raised to *H*. Its velocity and
   deflection start at zero.
   - Backing: `measurable` (G4 and G5 for initial cadence, stage, motion and lifetime,
     S-0202 claim 15 for creation draws).
   - Confidence: high

### The system's turn

6. **Order within a gas system's turn.** In a gas-behaviour system's turn (S-0203 claim 2),
   each existing particle's damage and other update effects use its position before that tick's movement. For any two particles in the system, the newer particle's update effects, including any random draws, precede the older particle's effects. Each particle's own behaviour-kind update occurs in that tick, and its lifetime is reduced by one modulo 65536; a resulting value of 0 marks it dead. All existing particles' update effects finish before any surviving particle moves. Only particles not marked dead move during that tick. A dead particle disappears and may leave a successor (claim 7). When more than one particle dies in the tick, successor creation draws for the newer dead particle precede those for the older dead particle, and all successor creation follows the particles' update effects. Consequences:
   - A particle hits in the tick when its lifetime runs out, then disappears in that tick.
   - A particle created by a death that a cloud causes during the update pass (claim 3) is not
     updated that tick, but it is moved in the second pass. One created later in the tick, for
     example by an animation's middle event, waits for the next turn.
   - A successor is neither updated nor moved in the tick it appears.
   - The system lifetime goes down by 1 each turn. A system whose lifetime reaches exactly 0 is
     marked to end, and a marked system with no particles left is removed. A lifetime of −1 never
     reaches 0.
   - Backing: `measurable` (G5 for lifetime, successor and turn timing; G6 for draw order).
   - Confidence: high

7. **Successor.** When a dead particle's type names a next particle type, a particle of that type
   is created (claim 5, including its creation draws) at the dead particle's position plus the type's
   next-particle displacement (leptons, per axis). It then takes the dead particle's velocity. Successors
   are created in the second pass, so their draws come after all the update draws of that turn.
   - Backing: `measurable` (scenario G5)
   - Confidence: high

### Gas particle motion

8. **Random drift.** On even tick numbers only, each gas particle takes one scenario raw value
   (S-0202). With probability 1/8 (F2) it takes two more and adds −1, 0 or +1 to either *v*ₓ or
   *v*ᵧ, the second draw choosing which. After such a drift event, *v*ₓ and *v*ᵧ are clamped to [−2, 2]. A gas particle
   therefore takes 1 or 3 raw values on each even tick and none on odd ticks.
   - Backing: `measurable` (scenario G6: cloud spread on flat ground, distribution over seeds)
   - Confidence: high

9. **Vertical speed.** On even tick numbers: when the particle's height above the terrain
   (glossary, taken at its own position; a particle is never on a bridge deck) is more than 5 leptons, *v*_z goes down by 1, but not below −5. Otherwise a
   negative *v*_z is set to 0.
   - Backing: `measurable` (scenario G7)
   - Confidence: high

10. **Slope deflection.** On even tick numbers the particle also sets its deflection (F3). Over flat ground, or when nothing below stops it, the deflection is zero. When
    the point a short step below it (the *probe*, F3) is below the terrain, crosses a high bridge's
    deck height, or is within 150 leptons above the terrain in a cell holding an obstacle, the
    deflection points downhill along the slope of the particle's cell. Then on each odd tick the
    particle moves by that deflection, so a cloud on a ramp creeps downhill and one on flat ground
    does not.
    - **Obstacles:** a building, except an inactive section of a laser fence and a vehicle-like
      building (S-0214 Terms); or, in a cell without a building, one of the three wall overlays of
      S-0306's constants (by position in the imported overlay list).
    - Backing: `measurable` (scenario G8)
    - Confidence: medium (downhill direction and the zero on flat ground: high. The exact
      magnitudes and the obstacle and bridge triggers: medium)

11. **Movement.** Each live gas particle's position after the update pass is determined by F5. A due wind push contributes on either parity of tick (claim 12). On even ticks the particle's height is unchanged and its horizontal velocity contributes no movement. On odd ticks, wind and stored deflection determine the horizontal position at which sinking is evaluated; sinking moves the particle toward 5 leptons above the terrain by no more than 2 leptons, and velocity determines its subsequent displacement. Its final height is never below 5 leptons above the terrain at its final horizontal position.
    - Backing: `measurable` (scenarios G6 and G7)
    - Confidence: high

12. **Wind.** A gas particle type with a positive wind-effect field *W* is pushed one step of the
    rules' wind direction on every tick whose number is a multiple of ⌊10 / *W*⌋ (F6). The step is
    given by the table in Constants. With *W* above 10 the original fails on integer division by zero; our engine's
    S-0205 claim 3 departure permits a push only at tick 0. Stock gas
    types have no wind effect, so this matters only with other data.
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
    equal to its damage interval reduced modulo 65536, with zero meaning 65536 updates. Its death kind, poison, berserk and every other effect of the hit
    are S-0302's and S-0107's, and a death it causes can create further clouds (claims 2 and 3).
    - Backing: `measurable` (scenario G4)
    - Confidence: high (cadence, ground layer, no detonation). Medium for *D*: it uses the lift of
      F7, which makes the distance smaller than the lepton distance.

14. **Credit.** Kills and damage by particles of the shared gas system credit no house: chain
    deaths from a virus cloud give no veterancy and no score to anyone, even though the virus-death
    animation itself may be owned by the attacker's house (claim 2). Particles of a warhead-created
    system credit the house that owned the detonation.
    - Backing: `measurable` (scenario G10)
    - Confidence: high

15. **Particle stage.** In each update, after the damage step, the particle stage advances by 1
    on the updates that F8 selects. The period is the type's stage-advance field, or one more than it when the particle's
    object number is odd. Then, whenever the particle stage equals the type's end-stage field, the particle
    is marked dead if its type has the delete-at-stage-limit flag; otherwise its particle stage restarts at
    0 (not at the start-stage field). The drawn image-frame number is the particle stage itself:
    the type's start-frame field is not added for gas.
    - Backing: `measurable` (scenario G5: the successor's life ends at the stage limit)
    - Confidence: high

16. **Drawing detail.** A particle whose type deals no damage is not drawn while the measured
    drawn-frame rate is below the active performance cutoff. This is not a test of the player's
    lowest detail setting. Damaging gas particles bypass this particular suppression. At the
    highest detail setting, a particle with translucency 25 or 50 uses that translucency,
    a stored signed translucency from 75 through 127 uses 75. At other detail settings it is drawn opaque.
    Translucency is reduced modulo 256 at creation.
    - Backing: `measurable` (scenario G11, visual). The adaptive cutoff's settings import and
      defaults remain `unmeasured` here: those inputs have not been traced or independently
      varied, so no fixed cutoff is claimed (Open questions).
    - Confidence: medium

## Formulas

**F1. Signed magnitude.** For a raw value *r* read as a signed 32-bit integer (S-0202 Terms):
*m*(*r*) = |*r*|, except that *m*(−2³¹) = −2³¹ (the magnitude wraps). Remainders are truncated
toward zero, so their sign follows the dividend.

**F2. Drift** (even ticks; integer arithmetic with signed remainders truncated toward zero).
One raw draw tests whether the low three bits of *m*(*r*₁) are zero. A nonzero result leaves
horizontal velocity unchanged and consumes no further drift draws. A zero result consumes two
more raw draws, in order *r*₂ then *r*₃. The step is *k* = (*m*(*r*₃) rem 3) − 1.
Even *m*(*r*₂) selects the horizontal x axis, odd selects y. The selected velocity increases by
*k*, and both horizontal velocities are then clamped to [−2, 2] leptons per tick.

With *r*₃ = −2³¹, *k* = −3 before the clamp.

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

**F8. Particle stage advance.** Let *n* be the particle's object number, *A* the type's stage-advance field,
*E*_max its integer lifetime field and *E* its unsigned 16-bit remaining lifetime before this
update's decrement. Integer sums use signed 32-bit wraparound (S-0205 claim 1). The
particle stage advances when

> (*n* + *E*_max − *E*) is divisible by (*n* mod 2) + *A*.

The particle stage is an 8-bit signed counter, wrapping from 127 to −128. The start-stage, end-stage and
stage-advance fields are stored as 8-bit signed values. With *A* = 0 and an even *n*, the divisor
is 0, and the advance follows S-0205 claim 3.

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
| Particle type defaults | lifetime 1, damage interval 0, damage 0, stage advance 4, start and end stage 0 | | Import mapping |

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
  instead of 2, unlike its mirror direction 5. Off: Δₓ = 2 for direction 3. It affects only gas
  types with a wind effect, and none have one in stock data.
- **Particle stage parity.** The original takes the parity in F8 from its own object identifier.
  Our object number (Terms) has a different sequence, so which clouds advance with period *A* and which with
  *A* + 1 can differ from the original. This is a known departure of the same kind as S-0306's
  missile phase. It decides how long a particle with the delete-at-stage-limit flag lives.
- **Missing shared gas type.** The original then reads outside its type list. Our importer rejects
  the rules set (claim 1).
- **Virus-death animation without a particle type.** The original reads outside its type list in
  claim 3 (and claim 2 creates nothing). Our importer warns, and claim 3 creates no particle.
- **Wind direction outside 0–7,** including the absent default −1. The original reads outside its
  table. When a type with a wind effect exists, our importer warns, and no wind push is applied.
- **Non-gas particle types in the shared gas system** (claim 2). Such a particle updates and moves
  by the rules of its own behaviour kind, which other specs own. Only the turn order of claim 6
  comes from its system. Stock data never does this.
- **Signed magnitude** (F1): an extreme raw value gives a drift step of −3 before the clamp, so it
  is clamped to −2 like any other.
- **Damage interval 0** (F7): one hit per 65536 updates rather than one per tick.

## Import mapping

| Concept | Source | Parse form | Default when absent |
|---|---|---|---|
| Shared gas system type | Rules section named `GasCloudSys`, a particle-system type; the name is fixed by the engine | section | none: rejected (claim 1) |
| System held type | particle-system type `HoldsWhat` | particle type name | none |
| System behaviour kind | particle-system type `BehavesLike` | name: Smoke, Gas, Fire, Spark, Railgun | none (no update) |
| System lifetime | particle-system type `Lifetime` | integer, ticks | −1 |
| Particle behaviour kind | particle type `BehavesLike` | name: Gas, Smoke, Fire, Spark, Railgun | none (no update) |
| Damage interval | particle type `MaxDC` | integer, ticks | 0 |
| Lifetime | particle type `MaxEC` | integer, ticks | 1 |
| Damage amount | particle type `Damage` | integer | 0 |
| Warhead | particle type `Warhead` | warhead name | none |
| Start frame (not added for gas drawing) | particle type `StartFrame` | integer | 0 |
| Start stage | particle type `StartStateAI` | integer, stored as 8-bit signed | 0 |
| End stage | particle type `EndStateAI` | integer, stored as 8-bit signed | 0 |
| Stage advance | particle type `StateAIAdvance` | integer, stored as 8-bit signed | 4 |
| Delete at stage limit | particle type `DeleteOnStateLimit` | boolean | no |
| Next particle type | particle type `NextParticle` | particle type name; `<none>` for none | none |
| Next-particle displacement | particle type `NextParticleOffset` | three integers, leptons | 0, 0, 0 |
| Wind effect | particle type `WindEffect` | integer | 0 |
| Translucency | particle type `Translucency` | integer, reduced to signed 8-bit; claim 16 selects blending | 0 |
| Particle art | particle type `Image` | S-0007 claim 28 | |
| Warhead's particle system | warhead `Particle` | particle-system type name | none |
| Virus-death animation | Rules `[General]` `InfantryVirus` | animation name | |
| Animation's particle type and count | art `SpawnsParticle`, `NumParticles` | name, integer | S-0109 |
| Wind direction | Rules `[General]` `WindDirection` | integer 0–7 | −1 (Edge cases) |
| Gravity | Rules `[AudioVisual]` `Gravity`, as in S-0306's Import mapping | integer | 3 |

## Measurement scenarios

All under the measurement setup (glossary): stock data except explicitly named test files,
engine add-ons, director and unrelated mods off, skirmish, game speed index 2, 800×600, a small
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
  *Falsified if* either kill produces no clouds.
- **G2. Cloud count.** (a) Kill a human infantry with the virus sniper: count the clouds at the
  animation's middle frame. Expected: the animation's particle count. (b) Kill an attack dog (a
  non-human infantry type), and separately a parachutist still in the air, with the virus sniper.
  Expected: exactly one cloud, at the moment of death, and the animation disappears at once.
  *Falsified if* (b) shows the particle count or the animation persists. (c) Named test art file
  G2-zero-middle: give the virus-death animation a one-frame sprite, no loop delay and a positive
  particle count other than one, so μ = 0. Repeat (b): expect the middle-event count plus one,
  in that creation order. Repeat with a smoke-behaviour particle: it still enters the shared
  system and subsequently follows smoke motion, not gas motion.
- **G3. Warhead first particle.** Test rules file: give a test weapon's warhead the shared gas
  system's type as its particle system. Fire once at empty ground. Expected: one cloud at the
  impact point in the detonation tick (raised to terrain if needed), and none appearing later
  from that gas system. Named test rules file G3-system varies the held type between present
  and absent, and the system kind across the imported kinds: absence gives no initial particle,
  presence gives exactly one initial particle before any kind-specific emissions. For gas,
  compare unlimited lifetime with a short positive lifetime: the positive case is marked at
  that turn count, survives while particles remain, and disappears when empty; unlimited stays
  empty until game clear. Record system existence as well as particles.
- **G4. Damage cadence.** The virus sniper kills one human infantry that shares its cell with a
  second enemy infantry ordered to stay. Record the tick numbers of the second unit's health
  changes and of the clouds' creation. Expected: hits every *I* ticks, where *I* is the cloud
  type's damage interval field, the first *I* ticks after a cloud appears, one hit per cloud in
  the cell. Variant (a): the kill on the ground under a
  high bridge, with infantry on the deck above. Expected: those on the deck are never hit.
  Variant (b): two enemy infantry in the cloud's cell, the first killed by a hit. Expected: the
  second is still hit in the same tick. Named test rules file G4-distance: use one damaging
  cloud, a long lifetime and no stage deletion, an interval of 1, and stationary targets at
  several horizontal separations within its cell. Compare health changes to S-0302 using F7,
  not Euclidean or unscaled lepton distance. Give its warhead conspicuous detonation animation
  and particle-system effects: cloud hits must cause neither. Also test interval zero for
  a recorded lifetime shorter than 65536 updates, and a damaging hit on the particle's
  final lifetime update. Expected: the zero interval wraps on its first update and gives no
  hit before removal; final-update damage with a positive interval occurs before removal.
  A harmless variant must not reset its countdown on zero.
- **G5. Lifetime and successor.** Watch one stock virus cloud until its successor ends in 20
  runs. Record each particle's creation tick, position, stage and removal tick and the random
  state used for its lifetime draw. Expected: disappearance and successor creation share a
  tick; the successor position is the old position plus the imported displacement, floored at
  terrain, and it inherits velocity but not horizontal deflection. Its first update and motion
  wait until the next system turn. Compute its stage advances from F8, including the creation
  phase, rather than assuming a full period before its first advance. End occurs on the first
  lifetime-zero or deleting-stage-limit update. Named test rules file G5-stages varies start
  stage, end stage, delete flag and advance period; without deletion, equality resets to zero,
  not the start stage. Capture frames while varying the start-frame field: the selected frame
  must remain the stage itself. Include an odd-tick
  virus chain death: its directly spawned particle moves immediately but has no update until
  the next turn; a middle-event particle created after the system turn waits for the next turn.
  Include a nonzero successor displacement and a parent dying where deflection is nonzero.
  Wrong phase, inherited deflection, immediate successor motion or a wrong frame falsifies the claim.
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
  Direction 3 moves 1 lepton in *x* per push. Include W = 0 (no pushes) and W above 10:
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
  At lower detail all these gas clouds use opaque drawing when otherwise visible.

## Open questions

- **Chaos drone.** With the stock data the chaos drone's weapon creates an animation and an area
  berserk effect, but names no particle system, so the stock chaos drone makes no gas particles.
  An unused stock warhead names a psychic gas system. S-0610 should not rely on this spec for the
  stock chaos drone.
- **Non-gas particles in the shared system** (claim 2). Only other data can do this. Should the
  importer warn?
- **Adaptive drawing cutoff.** Claim 16 establishes the suppression test, but the settings and
  complete tuning of the shared performance-cutoff mechanism are not specified here; their
  import mapping remains untraced. This omission is `unmeasured` in this draft because no
  scenario yet varies those settings independently; it must not be treated as a fixed lowest-detail rule.
- **Other creators of gas.** The original also has a leftover monster type, absent from the stock
  data, that puts gas into the shared system. It is out of scope here.
