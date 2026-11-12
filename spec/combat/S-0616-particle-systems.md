---
id: S-0616
title: Particle systems (smoke, fire, spark, railgun)
topic: combat
revision: 29
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-12
measurements: []
requests: [Q-1153, Q-1155, Q-1243, Q-1400, Q-1401, Q-1402, Q-1403, Q-1404, Q-1405, Q-1520, Q-1540, Q-1590, Q-2999]
supersedes: []
---

# S-0616: Particle systems (smoke, fire, spark, railgun)

## Summary

What a particle system of the smoke, fire, spark or railgun behaviour kind does in its turn: when
it emits particles, where and how many, every draw it takes, how its particles move, collide with
terrain and objects, deal damage, change stage and end, and when the system itself ends. Only fire
particles deal damage; the other three kinds change play through their draws from the scenario
random stream and through how long they stay in play, and a system or particle with no behaviour
kind is covered by claims 2, 3, 21 and 22 below, with S-0613 claim 17 for its count-down.
These claims cover skirmish only; network games and campaigns are out of scope.
Damage follows the particle's own kind, including gas when kinds are mixed (claim 3).
Claims 23 to 31 say how every particle is drawn (presentation only): which are drawn, palette and
brightness, image frame, translucency, placement, the pixel colours of sparks and railgun
particles, draw order, and the lights and laser of a system.
Claim 8a says what a smoke particle type whose next type leads back to it does, and claim 20a how
large a spark burst or a railgun line can be; both answer Q-2999.

**What this spec does not restate.**
- **The gas kind,** the shared gas system, and every system's lifetime count-down and leaving
  play: S-0613 (claim 17 for the count-down and leaving play).
- **Creating a system at a detonation** and its first particle: S-0303 claim 20, S-0613 claim 4.
- **Creating a particle:** S-0613 claim 5 for its list place, terrain floor, lifetime, start stage and damage countdown, and
  S-0202 claim 15 for its creation draws (its per-particle draw table applies to every creator of a particle, not only an animation's middle event).
- **Particle requests from an animation's middle event:** S-0109 claim 14.
- **Drawing:** the layer and its order (S-0102 claims 3 and 7), centred placement (S-0102 claim
  19), depth (S-0102 claim 13), the light-alpha value (S-0104 claim 15), the gas frame and the
  translucency rule with its performance cutoff (S-0613 claims 15 and 16), and the look of a laser
  beam (the owner of Q-1521).
- **Art lookup** of particles and systems: S-0007 claims 28 and 29.
- **Special-hit effects:** S-0615.
- **The railgun shot's hits and terrain effects:** S-0306 claim 27. This spec retains the
  beam sampling and end-point rule (F14), and the later particle line (claim 17).
- **When each creator of claim 1 makes its system.** Each is owned by the spec named there.

## Terms

S-0613 (Terms) defines the terms below; each is restated here in one line so that this spec
stands alone. This spec adds the terms after them.

- **Particle system.** A world object that owns an ordered list of particles and takes one turn per
  tick. Its *held type* is the particle type its first particle has (S-0613 claim 4), its
  *behaviour kind* (gas, smoke, fire, spark, railgun or none) chooses its turn, its *owner house*
  may be none, and its *system lifetime* is a count of ticks (−1: never times out).
- **Particle stage.** A small signed counter of a particle that advances at a fixed cadence
  (S-0613 F8) and selects the drawn frame.
- **Successor.** A particle made in place of a dead particle whose type names a next particle type.
- **Object number.** The number our engine gives each object at creation (S-0306 Terms).
- **Terrain height** *H*(*x*, *y*). The terrain surface height of S-0106 claims 2 and 3 at a
  horizontal position, never a bridge deck.
- **Translucency.** A particle's integer drawn-translucency value, in the percentage points of
  S-0613 claim 16. It only affects drawing.


- **Owner object.** The object a particle system belongs to, if any: for example the building that
  smokes or the vehicle whose weapon throws flame. It is not the owner house. A system has at most
  one. Claim 1 says which creators give one.
- **Target point.** A world position (integer leptons) that a system keeps. At creation it is the
  position of the target object given by the creator, when there is one, or else the point the
  creator passes (for a creator that passes none, the world origin (0, 0, 0)). A system with an
  owner object also takes the owner's current target as its target object. Only fire and railgun
  particles use the target point: flame particles head for it, a flame sweep (claim 11) moves it,
  and a railgun line (claim 17) runs to it. A smoke or spark system keeps one but nothing in play
  depends on it.
- **Emitter, emitter place.** An *emitter* is a particle system that an object holds as its own
  effect: a weapon's fire, spark or railgun system, or a damage-smoke or damage-sparks system. An
  object has four *emitter places*, one each for fire, railgun, damage smoke and spark; a weapon's
  spark emitter and a damage-sparks system share the spark place. An object holds at most one
  system per place and makes a new one only when the place is empty. A weapon's emitter goes into
  the place its weapon flag names, whatever the behaviour kind of its system's type, so a place
  can hold a system of another kind. S-0301 claim 5's place-and-kind rule (S-0301 Terms,
  *Effect weapon*) is the complete rule for what marks such a system and when its place is
  freed; claims 1, 20 and 22 state it here. Below, "fire place", "spark place" and "railgun
  place" name the place, and "fire-kind", "spark-kind" and "railgun-kind" name the kind.
- **Smoke point.** One of the four displacements (from a refinery's position) at which the refinery
  makes a smoke system (claim 1).
- **Bridge cell.** A map cell that carries a bridge span, so that it has a deck above its ground
  level.
- **Damage-sparks flag.** The infantry type flag that is the cyborg flag of S-0312 (claim 2): set
  for exactly the infantry types that have the cyborg flag, and for no other type. Its key is in
  the Import mapping.
- **Wall.** A cell counts as a wall for a spark when it holds a wall overlay, the overlay whose
  connection bits S-0112 claim 7 gives. Confidence medium (Open questions: the exact test).
- **Inside a tunnel.** As S-0213 (Terms): the state of a mobile object travelling through a
  tunnel of S-0212 (Terms).
- **Yellow ratio, red ratio.** The game-wide health-ratio thresholds of S-0302 claim 13 and its
  Import mapping. S-0302 claim 12's fixed half does not use them.
- **Deactivated vehicle.** A vehicle that has lost what it needs to operate: the deactivated state
  that S-0214 claim 12 and S-0701 claim 17 name for objects. In play, a vehicle enters it only by
  the disabling of S-0405 claim 13 (the house's last control building lost or stopping, or a
  vehicle of a type that needs one entering a cell with no building while its house has none).
  An electromagnetic pulse, paralysis and being without a driver are other states: they do not
  make a vehicle deactivated here.
- **Resource-carrying slave.** A slave of a slave miner that delivers its load to its master, the
  slave miner (S-0402 claim 14). Only the delivery matters here (claim 1, refinery smoke).
- **Electric bolt.** The instantaneous lightning-style shot of a weapon that is flagged as an
  electric-bolt weapon instead of launching a projectile; it is drawn by S-0102 (beam effects).
  This spec needs only that each such shot has an end point (claim 1). No approved spec owns the
  shot itself (Open questions).
- **Dropping projectile.** A projectile that does not steer or fly out from the fire point but
  drops towards the impact point (S-0306 claim 5; the glossary's *fire point*).
- **Marked.** S-0613 claim 17's state of a system that is to end. A marked system makes no
  emission (claims 9, 11 and 17), though the mark does not stop a spark system's bursts while its
  *K* is above 0 (claim 15, rated medium there), and a smoke system's dead particles still split
  into successors (claim 8, rated medium there for a marked system). It leaves play in the
  first turn in which it holds no particle. Besides the lifetime, claims 1 (the healing marks of
  damage smoke and the trigger action that marks the systems of a cell), 4, 9, 10, 15, 17 and 20 (a spark counter of 0 or less) mark systems.
- **Dead mark.** A particle's flag that it has ended. A particle with the dead mark is removed in
  the removal pass of its system's turn. S-0613 uses the same mark for gas.
- **Particle speed** *S*. A single-precision value of each particle, in leptons per tick. It
  starts at its type's speed field, plus the smoke variation of S-0202 claim 15.
- **Flight direction** *u*. A single-precision three-component vector of each particle, set at creation:
  the unit vector from its creation position to the target point it is given (F1). When the two
  positions are equal, *u* = (0, 0, 0). A spark does not use it: a spark moves only by the velocity that F10 gives it at creation.
  A new particle's velocity starts at zero (S-0613 claim 5), whatever its kind; for a spark it stays
  zero only until F10 sets it, in the step that makes the spark.
- **Emission timer** *T*. A single-precision value of a smoke system. It starts at the type's spawn
  interval and grows by the type's slow-down field each turn (claim 9).
- **Sprite kinds, pixel kinds.** Gas, smoke, fire and no kind are drawn as sprites; spark and
  railgun as single pixels (claims 23 and 28).
- **Direction group** *g*. The fire frame group 0 … 3 of F15 (claim 25).
- **Own colour, colour index, colour progress.** The creation colour *c*₀ of a particle, the list
  position *i* and the mix fraction *t* of claim 29 and F16. Only drawing reads them.
- **Light-alpha value.** S-0104's per-screen-pixel value (Terms, claim 15), 127 by default.
- **Spark counter** *K*. An integer of a spark system. It starts at the type's spark-turns field
  and goes down by 1 in each turn that starts with it positive (claim 15).
- **Next-type link, next-type loop.** A particle type that names a next particle type has a
  *next-type link* to that type; no other type has one. A *next-type loop* is a set of particle
  types, each with a next-type link to the next and the last with one to the first; a type whose
  next type is itself is a next-type loop of one type. A next-type link *lies on* a next-type loop
  when both its types belong to that loop. A type has at most one next-type link, so it belongs to
  at most one next-type loop. Whether a link lies on a loop is a property of the types alone.

## Behaviour

### Inventory

1. **Who creates these systems in play.** The table lists every creator of a particle system in
   play, whatever its kind. Each row names the spec that owns when the creation happens and gives
   this spec's share: whether the system gets an owner object, what its target point is, and
   whether it starts with a particle. A system made without a particle starts empty: its first
   particles come from its own turn (claims 5 to 18). Making a system takes no draw. Claim 22 gives
   how each kind of system ends, by these three properties.

   | Creator | When (owner) | Owner object | Target point | First particle |
   |---|---|---|---|---|
   | A warhead that names a system | at the detonation (S-0303 claim 20; owner house: the source house) | none | the detonation point | yes (S-0613 claim 4) |
   | Exploding overlay | after the nested detonation, 25 % chance (S-0303 claim 19; owner house: none) | none | the detonation point | yes when it has a held type (S-0613 claim 4; S-0303 claim 19 says only that the system appears) |
   | Damage smoke of a damaged object | on turning yellow or red (S-0302 claim 13) | the damaged object | the world origin, unused | no |
   | Destruction smoke of a building | at destruction (S-0312 claim 19 f; which entries are eligible and what counts as attached already: this claim, below) | the building | the world origin, unused | no |
   | Refinery smoke | at each unloading step of a resource delivery, one system per smoke point (this claim, below) | the building that makes it (this claim, below) | the world origin, unused | no |
   | Natural smoke of a building | when the building is placed in play, and again when it uncloaks (this claim, below; S-0516) | none | a point of the building (its position), unused | no |
   | Effect weapon (fire, spark or railgun emitter) | at the shot, after the projectile (S-0301 claims 5 and 18) | the firer | for the fire or spark place: the position of the firer's target object when the shot has one, else the world origin; for the railgun place: the beam's end point (F14), fixed at the shot; whatever the system's kind | no |
   | Mind-control overload | 5 spark systems per damaging pulse (S-0601 claim 16) | none | unused | no |
   | Electric bolt | at the bolt's end, after the bolt's own draw (this claim, below) | none | unused | no |
   | Drone parasite delivery | per delivery to a host that is not infantry (S-0614 claim 9) | none | unused | no |
   | Game start | the shared gas system (S-0613 claim 1) | none | unused | no |
   | Trigger action that creates a particle system | when the action runs (this claim, below) | none | the waypoint cell's centre, at ground surface height | no |
   | Damage sparks of an object | in its turn, below the yellow ratio (this claim, below) | the object | the world origin, unused | no |
   | Sparks of a deactivated vehicle | in its turn (this claim, below) | none | unused | no |
   | A voxel-model animation with an attached system | at the animation's creation (this claim, below) | the voxel animation | the world origin, unused | no |

   An animation's particle request (S-0109 claim 14) makes a particle in the shared gas system,
   not a new system. The rows that no other spec owns:
   - **Refinery smoke.** A *smoke event* on a building makes one system of the building type's
     refinery smoke type at each of its up to four smoke points that is set, with the building as
     owner object. A point of (0, 0, 0) or (128, 128, 0) counts as unset.
     This is an engine constant, not a rules value.
     Each system's lifetime is set to the game-wide refinery-smoke time, not its type's. No draw.
     A building whose type names no refinery smoke type makes nothing; no other property of the
     building is tested (it need not be a refinery). A smoke event on a vehicle, infantry or
     aircraft object makes nothing. The smoke events (Q-1401):
     - **Harvester unloading.** One smoke event on the building in the neighbouring cell at each
       processing of S-0402 claim 11 that falls in one of its two rows with the counter at or
       above 900 × *D*: each payment row, and also the empty-load row that makes the harvester
       *finishing*. So a harvester that carries *k*
       different resources makes *k* + 1 smoke events in one unloading, and one that docks empty
       makes one. Each smoke event comes before the payment of its processing.
     - **Slave delivery.** One smoke event on the slave's master per delivery (S-0402 claim 14),
       after all of that delivery's payments, and only when at least one of them paid a
       positive amount; none per resource. A deployed slave miner is a building and makes the
       smoke at its own smoke points when its type names a refinery smoke type. An undeployed
       slave miner is a vehicle, so its slaves' deliveries make no refinery smoke.
   - **Destruction smoke** (Q-1400). Only smoke-kind entries of the building type's destruction
     list are eligible; entries of every other kind (gas, fire, spark, railgun, none) are
     skipped. *Attached already* means only that a destruction-smoke system made earlier by
     this same building is still in play; the building's damage smoke, natural smoke, effect
     weapon emitters and damage sparks do not count. Since the building makes its destruction
     smoke only at its destruction, in practice the condition never stops it. With *m* ≥ 1
     eligible entries, and the building's height above the surface greater than −10 leptons, one
     ranged draw R(0, *m* − 1) picks one, counted from the end of the list; with none eligible
     nothing is made and nothing is drawn. The system is made at the building's position plus
     the type's destruction-smoke displacement, with no owner house. Its owner object is the
     building, so it is marked when the building is removed (claim 4):
     - A building removed immediately under S-0312 claim 21 (its type does not explode and
       it was not being sold when destroyed) leaves play at the end of its death effects,
       before the system's first turn. Its destruction smoke is marked at once, never emits,
       and leaves play in its first turn.
     - A building awaiting removal under S-0312 claim 21 (its type explodes, or it was being
       sold when destroyed) stays until its next eligible own turn; erasure or an
       electromagnetic pulse can postpone that removal. Its destruction smoke emits in any
       turn before removal that claim 9 allows, and is marked at removal. Particles already
       made carry on until they end.
     In neither case does it emit after the building has gone.
   - **Natural smoke.** A building whose type names a natural particle system gets one when it is
     placed in play, if it has none, at its position plus the type's natural-smoke displacement.
     After each hit on the building that reports *now yellow* as S-0302 claim 12 gives it (health
     crossing half the strength under its default, not the yellow ratio), its natural smoke's emission timer *T* is multiplied
     by the dimensionless factor 1.5, in single precision. A hit that reports *now red* does not
     multiply it, so a single hit from above half straight to red (which reports *now red*, S-0302
     claim 12's precedence) leaves *T* unchanged. Repair does not undo the factor, and each later
     *now yellow* hit (after a repair above half) multiplies *T* again, so the factor compounds.
     A multiplied *T* can pass the spawn cut-off at its next growth and so mark the system (claim 9). Nothing ends it when the building is sold or destroyed; it ends by its lifetime, by its
     emission cut-off (claim 9, which marks a smoke system whose timer passes the cut-off), by full
     cloaking (below) or by the trigger action of this claim. S-0516 claim 18 states the same rule from the cloaking side: reaching full cloaking removes the system at
     once, with its particles (no lingering particles), and reaching full uncloaking makes a new
     system by the placement rule, but only when the type's natural-smoke displacement is not (0, 0, 0) (scenario P1, parts b and c).
   - **Effect weapons.** The emitter is made at the weapon's fire point, or at the firer's
     position for a dropping projectile. The fire, spark and railgun flags of a weapon are tested
     independently at the shot (Q-1403): for each flag the weapon has, in the order fire, spark,
     railgun, an emitter of the weapon's system type is made when the firer's place for that kind
     is empty. So a weapon with two or three flags makes up to two or three emitters at one shot,
     made in that order (which sets their turn order, S-0203). S-0301 claim 5's gate refuses the
     weapon while any place of its flags is held. For a system made for the fire or spark place,
     whatever its kind, the creator passes the shot's target object (target point: its position at
     the shot, the world origin when the shot has none). For a system made for the railgun place,
     whatever its kind, it passes no target object, and the target point is the beam's end point
     of F14, computed at the shot from the emitter's position to the target's position. Like every
     system with an owner object, the system in the railgun place still takes the firer's current
     target as its target object (Terms), but a railgun-kind turn uses only the target point,
     which is never recomputed from the target. So later motion of the target changes nothing: the
     line (claim 17) is made in the system's first turn, to that fixed point. A railgun-kind
     system held in the fire or spark place makes its line to that place's target point, not to
     F14's end point.
     The system held in the fire place, whatever its kind, is marked and let go when its firer
     takes a new target that is not within reach (outside the weapon's range, the test of S-0307
     claim 12), is given a move order, or dies. A system held in the spark or railgun place is not
     let go by any of these, whatever its kind. It ends by the marks of its own kind, owner removal
     (claim 4), a positive lifetime or the trigger action (claim 22). Each kind keeps its own marks
     (claim 22's first column) in whatever place holds it, and a let-go of the fire place marks
     whatever system that place holds. The fire place is freed by its let-go events or by its
     system leaving play; the spark and railgun places are freed only by their system leaving
     play.
     Creating the emitter takes no draw. The railgun shot's hits, terrain effects and their
     draws are owned by S-0306 claim 27 and occur with F14's end-point computation, before
     the emitter is created. F14 remains this spec's sampling and end-point rule.
   - **Electric bolt.** Each electric-bolt shot makes one system of the game-wide spark type at
     the bolt's end. When the game-wide spark type is absent, nothing is created. This is a departure for an absent key, with no switch and an importer warning (Import mapping; Open questions).
   - **Trigger actions.** Two actions touch particle systems. The first names a particle system
     type by its index and makes one system of that type, with no owner object, no owner house and
     no first particle, at the centre of the cell of the action's waypoint, at the ground surface height of that cell (not raised on a bridge, unlike the glossary's waypoint point; confidence medium). The
     second takes a waypoint and marks every particle system in play whose position lies in that
     waypoint's cell (the cell holding the position, 256 leptons per cell, whatever the system's
     kind, type, owner or age: gas systems included, so the shared gas system, whose position is
     the hypothesis of Constants (in cell (10, 10)), is marked by an action on a waypoint in that cell, and
     a gas cloud system made at a detonation in the cell is marked likewise). It removes nothing at once: each such system ends as claim 22 says
     for a marked system, and a system with particles left keeps updating them until they end.
     Systems made later in that cell are not affected. Both actions always succeed. The shared gas system's position is an engine constant set at the game start; it is not given by S-0613, and P1 part b's second run decides it (confidence medium).
   - **Damage smoke position.** A damage-smoke system (the S-0302 claim 13 row) is made at the
     object's position (for a building, the position S-0312 claim 19 f uses) plus the type's
     damage-smoke displacement (three integers, leptons), used as given. If the type's
     screen-relative flag is on, the original reads the displacement in another frame; that case
     is not specified (Open questions), and the importer warns and uses the displacement as given.
   - **Damage smoke ends by a mark.** When a hit leaves the object's health ratio above the yellow
     ratio (S-0302 claim 13, which gives the same state: the effect is marked, not removed), its
     damage-smoke system is marked. There is also a mark that is not a hit. In a turn in which the
     object gains health by its own self-healing (S-0314 claim 9), and in a turn in which a vehicle
     that is not organic gains health by its house's self-healing (S-0314 claim 10), the
     damage-smoke system is marked right after that gain when the health ratio is then above the
     yellow ratio (strictly), or the object's height above the surface is below −10 leptons. An
     infantry or organic object healed by its house's self-healing gets no such test in that turn,
     so its damage smoke is marked only by a later hit or by its own self-healing. The object keeps it in the
     damage-smoke place, and so makes no new damage smoke, until the system leaves play (claim 22).
     The marked system makes no emission (its dead particles still split as claim 8 gives, rated
     medium there for a marked system); the
     particles it holds keep updating, moving and ending, and the system leaves in the first turn
     in which it holds none. Nothing is removed at once, unlike the cloak row of natural smoke
     above. An object healed above yellow and damaged again before its system has left play
     therefore gets no new damage smoke.
   - **Damage sparks.** An infantry object whose type is a cyborg type (the damage-sparks flag, Terms),
     with its health ratio strictly below the yellow ratio, whose height above the surface is greater
     than −10 leptons (the test S-0302 claim 13 uses), and holding nothing in its spark place,
     makes a spark system by the draws of the table below. The list is S-0302's (Import mapping).
     S-0302 claim 13
     uses only its smoke-kind entries, for the damage smoke that a hit starts, and gives no role to the
     spark-kind entries; this claim is the only rule that reads them. The original has a separate
     per-turn rule for them, with its own comparison with the yellow ratio: strictly below here,
     whereas S-0302 claim 13's smoke effect starts at or below it, at a hit. Backing for this row: scenario P16.

     **Place in the object's turn** (Q-1404). The damage-sparks draws come in the general part of
     the object's turn: after the draws of its due behaviour processing (S-0215 claim 2), with any
     shots that processing makes, after any shot at a target of opportunity, after its mind-control
     overload (S-0601 claim 16) and after its
     own and its house's self-healing (S-0314 claims 9 and 10); and before its radiation hits
     (S-0525), its ordinary movement, its parasite step (S-0614 claim 18) and the separate infantry
     firing. A turn spent inside a tunnel, or with the object not on the map, makes no damage
     sparks and takes none of their draws.

     | The object's damage particle-system list (conditions above hold) | Draws, in this order | Outcome |
     |---|---|---|
     | has no spark-kind entry | none | Nothing is made. |
     | has *n* ≥ 1 spark-kind entries | one ranged draw R(0, 2147483646) each turn: the system is made when that draw × 4.656612877414201 × 10⁻¹⁰, in double precision, is below the game-wide sparking probability, which is the red one when the health ratio is below the red ratio and the yellow one otherwise (Import mapping). If it is made, one ranged draw R(0, *n* − 1) picks one of the *n* spark-kind entries, counted from the start of the list (S-0302's smoke pick counts from the end). | The system is made at the object's position plus the type's damage-smoke displacement (as above) and held in the spark emitter place (Terms), shared with an effect weapon's spark emitter. |

   - **Deactivated vehicle sparks.** A deactivated vehicle makes one system of the game-wide spark
     type on the ticks for which (trunc(*x* / 10) + trunc(*y* / 10) × 65536 + *t* + its object number) is a
     multiple of 30, where (*x*, *y*) is its position and *t* the tick number (integer, truncation
     toward zero). The system is made at its position plus those displacements, *z* unchanged. Two ranged draws R(−100, 100) give the *y* and then the *x* displacement, in leptons. With no game-wide spark type, nothing is created and no draw is taken. This absent-key case is a departure with an importer warning (Import mapping; Open questions).
     The cadence in this row is a **departure** (no switch, no importer warning), of the kind S-0306 claim 13 records for
     its object-number phase: the original's object numbers come from its own scenario-wide
     creation counter, which our engine does not reproduce, so the phase of the 30-tick cadence can
     differ from the original's. The period of 30 ticks, the position terms and the displacement
     draws are the original's.

     **Place in the vehicle's turn** (Q-1404). A deactivated vehicle still takes its turn, and the
     rest of the turn is as for any vehicle. Its spark step comes first in the turn: its draws come
     before every other draw of that vehicle's turn, including its behaviour processing,
     self-healing, damage sparks, movement and firing. It also happens in turns that end early for
     another reason (an electromagnetic pulse, paralysis, travel inside a tunnel), since it comes
     before them.
   - **Animation with a voxel model.** A voxel-model animation type with an attached system makes it at the
     piece's creation point (S-0317 claim 13). That point is not the piece's start position: a
     meteor piece starts moved back along its velocity (S-0317 claim 4), and a falling piece starts
     10 leptons above it (S-0317 claim 3). This position rests on S-0317 claim 13 and is rated
     medium, as there (S-0317 scenario V11 is the measurement that would settle it); scenario P1 does not record it, and the
     high confidence below does not cover it. A smoke system then keeps claim 5's displacement from
     its owner, as S-0317 claim 13 has it: for a falling piece the displacement is zero, since the
     piece is at its creation point when the system is made, and for a meteor piece it is the whole
     flight from the start position to the creation point; the two specs agree on this, and it is
     rated medium, as there (S-0317 scenario V11). When the animation is removed the
     system is marked, as for the removal of an owner object (claim 4), and leaves play once it
     holds no particle (claim 22).

   - Place and kind (S-0301 claim 5), confidence: each kind's own marks and a place's freeing
     when its system leaves play are rated as claims 4, 10, 15, 17, 20 and 22 rate them (high,
     except claim 20's let-go end in the fire place, medium). A
     fire-kind system in the spark or railgun place not marked by its own turn while its owner is
     alive and on the map: high. A move order frees neither the spark nor the railgun place: high.
     A retarget to a target not within reach frees neither of them: medium, resting on the let-go
     events above (rated medium). A let-go's mark of the system in the fire place, whatever its
     kind: medium, on the same events. That a retarget's let-go test is made at the change of
     target and not at the start of the system's next turn: medium. That the system's first turn
     after a let-go is the first one the mark affects, whatever the held system's kind: medium.
     Each kind's marked turn reached by a let-go of the fire place (claims 9, 10, 15 and 17, and
     S-0613's marked gas turn): medium, resting on the let-go events, and for the smoke, gas and
     no-kind cases also on those kinds' end rules. The fire place's freeing at the let-go itself
     rather than when its marked system leaves play: medium. The original's let-go end of a spark system with
     a spark-turns field of 0 or less held in the fire place (claim 20): medium. The creation
     reading by place: high for a matched emitter (its kind matches its place), medium for a
     system whose kind does not match its place.
   - Place and kind, backing: `measurable` (S-0301 scenario F4, unmatched part: u3 and u4 for a
     fire-kind system in the spark or railgun place, u5 and u6 for the spark and railgun places'
     freeing, the second run's u1 against u8 for a let-go's mark of a spark-kind system and its u3
     and u4 for a move order in the spark and railgun places, and the third run's u10 for a
     let-go's mark of a smoke-kind system in the fire place, its u10 and fire firer for the first
     turn that a retarget's mark affects, and its u9 for a marked spark system's bursts).
     `unmeasured`: that the retarget's test is made at the change itself rather than at the start
     of the system's next turn (nothing between the two reads the mark, so no run tells them
     apart), for the moment of the mark in a gas-kind or no-kind system (their turns do not depend
     on the mark, so the moment changes nothing observable), for a retarget in the
     spark and railgun places (no scenario gives a firer in those places a retarget to a target not
     within reach), for a let-go's mark of a railgun-kind system (its own line marks it in its
     first turn, so a let-go's mark would show only before that turn, which no scenario sets up),
     for the fire place's freeing at the let-go itself (no scenario records whether the marked
     system still holds a particle at the firer's first attempt after the let-go, so both readings
     give the same shot ticks), for the spark-turns case in the fire place (no scenario sets such a
     type in the fire place), for a let-go's mark of a gas-kind or no-kind system held in the fire
     place (no scenario puts a system of those kinds in the fire place) and for the
     creation reading when the kind does not match the place (no scenario records the line or
     target point of a railgun-kind system held in the fire or spark place). Each kind's own
     marks, the freeing when the system leaves play and the creation reading for a matched
     emitter: as backed below (scenarios P1, P4, P8, P11, P12, P14 and P22).
     Claim 20's fix: a design decision.
   - Backing: `measurable` (scenario P1, one run per row; P15 for the yellow factor of the
     natural-smoke row; P16 for the damage-sparks row and the damage-smoke position; P19 for
     the healing marks; P20 for refinery smoke; P21 for destruction smoke; P22 for effect
     weapons with several flags and the railgun end point; P23 for the place of both spark
     steps in the turn; S-0317 scenario V11, not yet run, for the position and displacement of a
     voxel-model animation's system)
   - Confidence: high for the creators, owners and first particles, for the refinery smoke
     events, the destruction-smoke rules, the natural-smoke factor, the effect-weapon flags and
     the place of the damage sparks in the turn. Medium for the three events
     that let the system held in the fire place go, for the self-healing mark of damage smoke,
     for the deactivated-vehicle row, for the shared gas system's position and for the
     natural-smoke uncloak rule (the displacement test). Medium for the damage-smoke position when
     the screen-relative
     flag is off. Medium for the position and displacement of a voxel-model animation's system
     (S-0317 scenario V11 is the measurement that would settle it). The flag-on conversion is `unmeasured`: no validating scenario for that unresolved conversion is supplied here (Open questions). The below-surface self-healing mark is also `unmeasured`: no setup here isolates a healing gain below −10 leptons.

2. **Which kinds change play.** Every kind changes play through the draws its turns take from the
   scenario random stream (claims 6, 8, 9, 11, 12, 15, 16, 17 and 18). An extra or missing draw
   shifts every later draw. Beyond that:
   - **Fire** particles deal ordinary damage (claim 14).
   - **Smoke, spark and railgun** particles deal no damage and are never targets. Their positions
     reach play only through when the particle ends, because a particle that has ended takes no
     more draws: a smoke particle can end at a bridge deck (claim 7), a spark ends at its first
     contact with ground, deck, wall or building (claim 16). Railgun positions never reach play.
   - **Gas** is S-0613's.
   - **A system with no behaviour kind** does no kind work (S-0613 claim 17). Its particles never
     update, move, take draws or end. This answers Q-1243 question 3: such a system leaves play
     only when it is marked and holds no particle. One that holds a particle stays in play for
     ever; one that never held a particle leaves play when its lifetime, claim 4, the trigger
     action of claim 1 or, while it is held in the fire place, a let-go of that place (claim 1)
     marks it.

   Drawing (frames, colour, translucency, lights, the railgun's laser and helix: claims 23 to 31) changes nothing
   else in play. The colour draws of claims 16 and 18 are taken whether or not anything is drawn.
   - Backing: `measurable` (scenario P2). The no-kind bullet's fire-place let-go: `unmeasured`, as
     claim 1's place-and-kind backing gives (no scenario puts a no-kind system in the fire place).
   - Confidence: high. The no-kind bullet's fire-place let-go: medium (claim 1).

### Common to all four kinds

3. **One particle's update.** Where a claim below says that a particle *updates*, the kind's own
   update of the particle happens (claims 6, 12, 16 and 18), and then its remaining lifetime goes
   down by 1, kept modulo 65536. When the result is 0 the particle gets the dead mark. The
   kind's update runs before that decrement, so a particle whose lifetime reaches 0 in a tick still takes that tick's update. A particle that already has
   the dead mark still updates, with its draws, if its system's turn updates it before removing it
   (smoke, claim 5).

   The particle's own behaviour kind, not its system's, chooses the update: gas (S-0613), smoke
   (claim 6), fire (claim 12), spark (claim 16) or railgun (claim 18). A particle with no behaviour
   kind has no update of its own, but its lifetime still counts down and ends it. Likewise, where
   a system's turn moves its particles (the gas, smoke and fire turns), the particle's own kind
   chooses the move: gas (S-0613), smoke (claim 7) or fire (claim 13). Spark, railgun and no-kind
   particles do not move there. Spark and railgun turns have no move pass. So a particle type put
   into a system of another kind keeps its own update and move, with the system's turn order.

   **Successors follow the system's kind, not the particle's** (Q-1155). When a dead particle's type
   names a next particle type, whether successors are made, how many and where is decided by the
   kind of the system that holds the dead particle, whatever the particle's own kind:
   - **Gas system:** the dead particle, of any kind, gets one successor (at its position plus its type's
     next-particle displacement, taking the dead particle's speed and its velocity and nothing else from it). S-0613 claim 7 gives the details of what is taken; its *Successor* term agrees that the system's kind chooses the successor rule.
   - **Smoke system:** the dead particle, of any kind, gets claim 8's two successors, with the
     draws of claim 8, *d* being taken from the dead particle's type.
   - **Fire, spark and railgun systems:** no successors for any particle, and no draw for them.
   So a smoke particle that dies in a gas system gets the gas rule's one successor and no draw
   from claim 8; a gas particle that dies in a smoke system gets claim 8's two successors; a smoke
   or gas particle that dies in a fire, spark or railgun system is removed with none. The
   successor's kind is that of the next type named, and it then updates by its own kind.
   - Backing: `measurable` (scenario P3, parts b and c, test rules; the speed part of a gas successor: S-0613 claim 7's scenario G5-speed)
   - Confidence: high for the successors' count, kind, position and draws; medium for a gas successor's speed, which takes the medium confidence of S-0613 claim 7 until G5-speed has run

4. **Removal of the owner object marks the system.** When an object is removed (S-0203 claims 6 and 10),
   every particle system whose owner object it is loses its owner object and is marked. This holds
   for every kind, gas and no kind included. When a system's target object is removed, the system
   loses that target object; its target point is unchanged.
   - Backing: `measurable` (scenario P4)
   - Confidence: high for removal. For an owner that leaves the map without being removed (for
     example into a transport) the claim is that it does not mark the system; confidence medium, unsettled (Open questions).

### Smoke

5. **Smoke system turn.** Each tick, a smoke system's turn has these observable effects. Its
   position, when the owner object is not a building, is the owner's position plus the displacement
   the system had from the owner when it was created, so an emission in the same turn starts from
   that position. Every particle takes its update draws (claims 3 and 6), oldest first, and all of
   them come before any successor draw of the turn (a move takes no draw). Each particle with the dead mark at the start of the turn's removal pass splits into successors when its type names a next
   particle type (claim 8) and is removed, in a pass that runs newest first, before any move; the successors become the newest particles of the system, so that pass does not reach them, and their draws come after the draws of the removals that made them. Then each particle without the dead mark moves
   (claim 7). A particle that a move marks dead is removed in the next turn's removal pass, after one more update with its draws. The emission (claim 9) takes its draws after the successors' draws, and
   the emission timer then grows (claim 9), so the next emission uses the grown value.

   S-0613 claim 17's count-down follows. This answers Q-1243 question 1 for smoke: a smoke system
   takes a turn every tick, and these are its effects on its particles.
   - Backing: `measurable` (scenario P5)
   - Confidence: high

6. **Smoke particle update.** |·| is the wrapping absolute value of the Formulas header. On a
   tick whose tick number is odd, the particle takes raw values by this table, and on an even tick
   it takes none and its velocity is unchanged.

   | Draw | Taken | Use |
   |---|---|---|
   | *r*₁ | every odd tick | When |*r*₁| is not a multiple of 4 the horizontal velocity is unchanged (the vertical velocity is still set to 0, as the text below says). |
   | *r*₂ | when |*r*₁| is a multiple of 4 | The parity of |*r*₂| chooses the axis (even: x, odd: y). |
   | *r*₃ | with *r*₂ | The chosen horizontal component changes by (|*r*₃| rem 3) − 1. |

   The change is −1, 0 or +1 lepton per tick, except for *r*₃ = −2³¹, where |*r*₃| stays −2³¹ and
   the change is −3 (as for gas, S-0613 claim 8). Both horizontal components are then limited to
   −5 … 5. On every odd tick the vertical velocity is set to 0; on an even tick it is unchanged. So
   each update on an odd tick takes one or three raw values.

   While its stage is below the type's end stage, the stage advances when S-0613 F8 says so,
   with *A* the type's stage-advance field. When the stage equals the end stage and the type has
   the delete-at-end flag, the particle gets the dead mark. The stage never passes the end stage
   here. Separately, when *S* > 3.0, *S* goes down by the type's slowing field, in single precision. So
   *S* stops changing once it is at or below 3.0 (or never changes when it starts there).
   - Backing: `measurable` (scenario P5, with the random-state recorder), except the raw value −2³¹ case, which is `unmeasured`: the recorder cannot force that rare draw and P5 does not practically isolate it.
   - Confidence: high

7. **Smoke particle move.** A live smoke particle moves once per turn, after the turn's removal pass (claim 5), so that a particle that its move marks dead is removed in the next turn's removal pass: by the wind push, its horizontal velocity, its speed upward and its vertical velocity
   (F2). It takes no draw and never collides with terrain, buildings or units. The one exception is
   a bridge deck: when the cell at the particle's position before the move is a bridge cell and the
   move would take it from below the deck to 260 leptons or less below the deck (F2), the particle
   does not move and gets the dead mark instead. It is removed in the next turn's removal pass,
   after one more update with its draws.

   **Wind.** Smoke uses its own wind table (Constants), not the gas table of S-0613: at direction 3
   the x step is 2, so `quirks.wind_southeast_short` does not apply to smoke. A smoke particle type
   with a wind effect of 0 is unaffected by the direction. For a nonzero wind effect and a direction
   outside 0 … 7 (including the absent value −1) the original's result is undefined. This is a **departure** as in S-0613 (importer warning, no wind push, no
   quirk switch); it is `unmeasured` because the original's result is undefined.
   - Backing: `measurable` (scenario P6 for the bridge stop and its wind part for directions 0 … 7); `unmeasured` for the departure
   - Confidence: high

8. **Smoke successors.** A particle of any kind with the dead mark, in a smoke system, whose type names a next particle type
   splits into two successors of that type when the removal pass reaches it. Two raw values give
   the displacements *o*ₓ, then *o*ᵧ (F3). The first successor is made at (*x* + *o*ₓ, *y* + *o*ᵧ, *z*)
   and the second at (*x* − *o*ₓ, *y* − *o*ᵧ, *z*), where (*x*, *y*, *z*) is the dead particle's
   position. Both have target point (0, 0, 0). Each takes its creation draws (S-0202 claim 15),
   then one raw value *r*; its translucency is the dead particle's plus 25 percentage points when *r* rem 6 ≠ 0, otherwise the dead particle's unchanged value (drawn only). Each successor's speed
   *S* is set to the dead particle's, but its flight direction and velocity are its own. Observably, the draws come in this order: *o*ₓ, then *o*ᵧ, then the first successor's creation draws and its translucency draw, then the second's; the dead particle is removed after both. This claim gives the successors made in a smoke system (claim 3); the successor made in a gas system is S-0613 claim 7's.
   A next type that leads back to the dead particle's type is claim 8a's case; in our engine a
   particle with claim 8a's loop mark does not split.
   - Backing: `measurable` (scenario P7, whose named test rules explicitly supply a next type).
     In a marked system: `measurable` (scenario P1 part b).
   - Confidence: high. In a marked system: medium, until P1 part b has run.

8a. **Next-type loops.** The original takes a next particle type as named, also when it is the
   particle's own type or leads back to it through other next-type links (Terms, next-type loop):
   it neither refuses nor changes such data when it loads the rules, and it sets no limit on the
   particles of a smoke system. The particle-cap field does not limit smoke emission (claim 9) or
   claim 8's successors. The two successors of a split are added after every particle the system
   holds, and the removal pass that makes them goes from the newest particle to the oldest, so a
   successor is not reached again in that pass: its first update comes in the system's next turn.
   So a split never repeats within one turn, and a next-type loop cannot keep a turn of a smoke
   system going. Across turns, each particle of a loop that dies in a smoke system, in whatever
   way, is replaced by two particles of the loop, so the number of particles, and the draws that
   their updates and splits take, can double once per particle lifetime without end. A marked
   system leaves play only in a turn in which it holds no particle (claim 22), so a system whose
   particles all belong to such a loop never leaves play, marked or not, unless claim 1's cloak
   removal removes it. When the program can get no room for a further successor, that successor
   is not made and takes no translucency draw; what else the game does then is not established.
   This differs from a spark burst or a railgun line, where a particle that gets no room makes
   the original fail (claim 20a). The difference is meant: each case states the original's own
   result. A next-type loop in a gas system replaces each dead particle by one successor (S-0613
   claim 7), so its count does not grow; this claim does not change it.

   Our engine departs from this for a smoke system, with no switch. A successor made by claim 8's
   split through a next-type link that lies on a next-type loop carries a *loop mark*, which is
   part of its saved state. Its own type belongs to that loop, so its own next-type link lies on
   the loop too: a marked particle with the dead mark in a smoke system is removed without
   splitting, in claim 8's place in the removal pass, and takes none of claim 8's draws (neither
   displacement raw value, nor either successor's creation draws or translucency draw). A
   particle made in any other way (an emission, claim 9; a detonation's first particle, S-0613
   claim 4; a gas system's successor; a split through a link that lies on no loop) carries no
   mark and splits by claim 8 unchanged. So one unmarked particle of a loop type leads to at most
   two particles of the loop, and neither makes more. Next-type links that lie on no loop, and the
   particles made through them, are unchanged. The importer warns once for each next-type loop.
   - Backing for the acceptance at load, the split's place in the turn and the growth across
     turns: `measurable` (scenario P32 parts a and b). `unmeasured` for what happens when no room is
     left, since a run is stopped long before, and for the absence of a limit on the system's
     particles, since no finite run shows every count. Confidence: high that the successors take
     their first update in the next turn; medium that no limit is checked, for the absence of any
     check at load and for the system never leaving play, until P32 runs; low for the result when
     no room is left.
   - Backing for the departure: a design rule, not an oracle value; `measurable` for our engine
     (P32's engine runs). Confidence: high for the departure, a design rule.

9. **Smoke emission and end.** A turn emits when all of these hold:
   - the system is not marked,
   - the tick number is a multiple of ⌊*T*⌋,
   - the system has no owner object, or its owner is not an infantry, vehicle or aircraft object,
     or it is one that is not inside a tunnel (S-0213 Terms; the tunnel is S-0212's).

   An emission takes two raw values and, when the system has a held type, makes one particle of it
   near the system's position (F4). Its target point is the system's target point. Its
   translucency rises by 25 when *T* is above the system type's translucency cut-off, and its speed
   is lowered as *T* grows (F4). The two raw values are taken even when there is no held type. The
   system type's particle-cap field does not limit smoke (quirk below).

   After the emission step, *T* grows by the slow-down field (single precision). When *T* is then
   above the type's spawn cut-off, the system is marked. A system whose slow-down field is 0 and
   whose cut-off is at or above its spawn interval is never marked by this claim. It ends only by
   claim 4, by the trigger action of claim 1, by its lifetime (S-0613 claim 17) or, while it is
   held in the fire place, by a let-go of that place (claim 1); a damage-smoke system also ends by
   the healing mark of claim 1.
   - Backing: `measurable` (scenario P5). The fire-place let-go: `measurable` (S-0301 scenario F4,
     unmatched part, third run: u10), as claim 1's place-and-kind backing gives.
   - Confidence: high. The fire-place let-go: medium, as claim 1 rates a let-go's mark of the
     system held in the fire place.

### Fire

10. **Fire system turn.** Each tick, a fire system's turn has these observable effects:
    - Every particle takes its update draws (claims 3 and 12), newest first, and then moves (claim
      13). A particle that gets the dead mark in its update still moves in that turn. A move takes no
      draw and no particle affects another's move, so the order of the moves among the particles is
      not observable.
    - Particles with the dead mark are removed in a second pass, before the emission, so the
      emission's particle count (F7) excludes them. In a fire system no particle has a successor, whatever its kind or its type names (claim 3).
    - The owner decides the system's mark. A system with no owner object, or with one that is not an
      infantry, vehicle, aircraft or building object, is marked, and so is one whose owner is no longer
      alive. An alive owner that has a target object and whose body facing is turning makes a flame
      sweep (claim 11), which happens before the emission.
    - The emission (claim 11) takes place unless the system is marked.

    S-0613 claim 17's count-down follows. This answers Q-1243 question 1 for fire.
    - Backing: `measurable` (scenario P8). A marked turn reached by a let-go of the fire place:
      `measurable` (S-0301 scenario F4, unmatched part, third run: the fire firer), as claim 1's
      place-and-kind backing gives.
    - Confidence: high. A marked turn reached by a let-go of the fire place: medium, as claim 1
      rates it.

11. **Flame sweep and emission.**
    - **Sweep.** The target point is placed in the owner's body facing (the facing of the owner,
      not of its target), at the owner's current distance from its target object, measured from the
      owner's position (F5). The system's
      position becomes the owner's fire point for its first weapon, with no extra displacement
      (S-0313 claims 1 and 2). Where that fire point alternates between bursts (S-0313 claim 2), the
      burst counter used is the owner's current one at the sweep, not the one of the shot that made
      the emitter and not 0 (Q-1403). Nothing is remembered: the
      sweep happens afresh in each turn in which the owner's body facing is turning.
    - **When.** A turn emits when the tick number is a multiple of the type's spawn interval, or
      when it is a multiple of 3 and this same turn made a sweep.
    - **What.** When the system has a held type, a particle of it is made 1 lepton above the
      system's position, aimed at a point beside the target point. That point swings across the
      line of fire with a 500-tick period (F6). The new particle takes its creation draws (S-0202
      claim 15) and then one raw value that moves it to a random place among the newest four
      particles of the list (F7). With no held type nothing is made and nothing is drawn.
    - Backing: `measurable` (scenarios P8 and P9)
    - Confidence: high for the cadence, the insertion draw and the sweep (the sweep lasts one
      turn only). Medium for the swing
      (F6), which reaches play through where the particles land.

12. **Fire particle update.** One update of a fire particle has one of these outcomes:

    | Condition, in the update | Draws | Outcome |
    |---|---|---|
    | *S* ≤ 0 | none | *S* becomes 0 and the particle gets the dead mark. Nothing else happens. |
    | *S* > 0 | one raw value (F8), which sets the particle's step for this tick | While the stage is below the end stage it advances when S-0613 F8 says so, with *A* the particle's own stage-advance value (claim 19). Then, if the stage equals the end stage and the type has the delete-at-end flag, the particle gets the dead mark; the step is still applied (claim 13), but there is no slowing and no damage. |
    | *S* > 0, and the row above did not end the update | as above | *S* goes down by the type's slowing field (single precision). The damage countdown goes down by 1, kept modulo 65536. |
    | As the row above, and the countdown reaches 0 with the type's damage amount not 0 and the stage at or below the type's last damaging stage (both compared as signed values) | as above | The countdown restarts at the damage interval, and the particle hits (claim 14). |
    | As the row above, and the countdown reaches 0 but the amount is 0, or the stage is past the last damaging stage | as above | The countdown does not restart. No later countdown makes a hit: the damage amount is fixed and the stage only rises (S-0613 F8 advances it by one step), so a condition that has failed once fails for ever, whatever the lifetime. |

    - Backing: `measurable` (scenario P9)
    - Confidence: high

13. **Fire particle move.** When *S* > 0 at the move, the particle's position changes by its step
    (F8). When the terrain height *H* at the new position is greater than at the old one, the
    particle gets the dead mark, but it is still placed at the new position. Fire passes through
    objects and over lower or level ground; only rising ground stops it. When *S* ≤ 0 it does not
    move.
    - Backing: `measurable` (scenario P9)
    - Confidence: high

14. **Fire damage.** A due hit (claim 12) uses the objects listed in the particle's cell: the
    bridge-deck layer when the cell is a bridge cell and the particle is at least 416 leptons above
    the ground level of the cell, otherwise the ground layer, in listing order (the order is S-0209 claim 13; the layers are the cell layers of the glossary). The ground level of the cell is the cell's height level (glossary) times 104 leptons, one value for the whole cell: it is not the terrain height *H* at the particle's position (which differs on a slope, Terms) and not the deck's own level, so the deck test is *z* ≥ (height level + 4) × 104. This is also the deck height of S-0212's height condition, where a deck pair's level is its cell's height level plus 4. The 416-lepton deck test is this spec's own rule, not S-0209's, and a hit takes the objects of one layer only. Every
    listed object that is alive, has health above 0 and is not the system's owner object takes a
    hit. Each hit is ordinary damage (S-0302) of the type's damage amount with the type's warhead:
    - its distance is ⌊*R* / 10⌋, with *R* the three-dimensional distance of F9.
    - It has no attacker, it does not ignore defences, and passengers may escape as usual.
    - A type with a non-zero damage amount and no warhead has no defined hit; claim 20 gives the departure.
    - It has **no attacking house** (S-0302 claim 1), even when the system has an owner house. So
      a kill by flame credits no house, unlike gas (S-0613 claim 14).

    The hit is not a detonation: the warhead's particle system, animations and spread do not
    happen. As for gas (S-0613 claim 13), an object removed by a hit does not stop the hits on the
    objects listed after it.
    - Backing: `measurable` (scenario P10)
    - Confidence: high, except that the objects listed after a removed one are still hit, which is
      medium (scenario P10, part b)

### Spark

15. **Spark system turn.** The draws of a turn depend on *K* at its start, as this table gives. A spark
    system whose type's spark-turns field is 0 or less is bad data: claim 20 marks it in its first
    turn, with no burst and no draw of this claim's table (its particles still update, claim 16).

    | *K* at the start of the turn | Draws, in this order | Outcome |
    |---|---|---|
    | *K* ≤ 0 | none | No burst and no light draw; *K* is unchanged. |
    | *K* = 1 | no decision draw; the burst draws below; the light draw | The system bursts. *K* goes down to 0, so the system is marked. |
    | *K* > 1 | the decision draw R(0, 2147483646), which passes when F10's chance test passes; if it passes, the burst draws below; the light draw | The system bursts if the decision passes. *K* goes down by 1; the system is marked when *K* is below 1. |

    The burst draws, in order: one raw value gives the number of sparks *c* (F10); three further raw
    values are taken and discarded (F10); then, for each spark in turn, the creation draws of one
    particle of the held type (S-0202 claim 15), made at the system's position with the system's
    position as its target point, followed by three raw values for its velocity (F10). The light
    takes one ranged draw R(0, 2147483646) in every turn that starts with *K* > 0, after the burst's
    draws; its value affects only drawing. Whatever *K* is, each particle then updates, oldest
    first, including the sparks just made (claims 3 and 16), and the particles with the dead mark
    are removed. This answers Q-1243 question 1 for spark. The table reads only *K*, not whether the
    system is marked: a marked spark system whose *K* is above 0 still bursts as the table gives.
    - Backing: `measurable` (scenario P11; for a marked system, scenario P1 part b and S-0301
      scenario F4's unmatched part, third run)
    - Confidence: high for the table, medium for a marked system's bursts

16. **Spark particle update.** Each update moves a live spark by its velocity, with gravity counted
    a second time in the vertical step (F11); the stored vertical velocity is lowered by gravity
    once, and the new position is truncated to whole leptons. The particle gets the dead mark at its
    first contact (F11). The contacts, by priority:
    - with a bridge deck, from above or below, when the cell at its old or new position is a bridge
      cell and the move crosses the deck height. A crossing wins over the two contacts below.
    - otherwise with the ground, when its new height is below the terrain height *H* there.
    - otherwise with a building or a wall in its new cell, at less than 150 leptons above *H*. This
      test also applies on a bridge cell when the move does not cross the deck.

    A contact from above places the particle on the deck; a contact from below places it 20 leptons
    below the deck; a ground, building or wall contact places it on the ground, except that a
    ground contact at least 100 leptons below the ground leaves it where it fell (F11). A spark does
    not bounce in play: it ends at its first contact. The update's colour draw is taken
    after any contact, so a spark that makes contact still takes that update's colour draw, one
    ranged draw R(0, 2147483646) for the progress of its drawn colour along its type's colour list,
    whatever the list holds (this per-update draw is separate from the creation-time colour draw of S-0202 claim 15). So a live spark takes exactly one draw per update, and deals no
    damage.
    - Backing: `measurable` (scenario P11 for motion, draws and deck contact; P17 for wall and ordinary-building contact)
    - Confidence: high for the draw, the motion and the ground contact. Medium for the deck
      placement. Low for the wall test (F11). The rule that every building stops a spark is medium; two possible exceptions (an open gate, one other state) are open (Open questions).

### Railgun

17. **Railgun system turn.** In the first turn in which the system is not marked and holds no
    particle, it emits its whole line at once and is then marked. The line has
    *n* = trunc(*L* × *p*) particles (F12), where *L* is the integer distance from the system to its
    target point and *p* is the type's particles-per-lepton field. Particles are made one after another. For each, the system takes three ranged draws R(0, 2147483646), then the particle's creation draws (S-0202 claim 15), then four more ranged draws R(0, 2147483646), all of this before the next particle's first draw. That is seven ranged draws per particle besides its creation draws. The values that place the particles along a helix and the
    laser line are used only for drawing; the draws themselves still advance the stream (claim 2). When *n* ≤ 0 no particle is made and no ranged draw is taken, but the system is still
    marked; whether it makes a laser beam is claim 31's. Whether the
    system has a held type plays no part in the count *n*. A system without one is claim 20's
    case: the original's behaviour and our engine's departure are stated there only.

    In that turn and every later one, after the emission step, each particle updates, oldest first
    (claims 3 and 18). Then the particles with the dead mark are removed. This
    answers Q-1243 question 1 for railgun.
    - Backing: `measurable` (scenario P12). A system marked by a let-go of the fire place before
      its first turn: `unmeasured`, for the reason claim 1's place-and-kind backing gives.
    - Confidence: high. A marked turn reached by a let-go of the fire place: medium, as claim 1
      rates it.

18. **Railgun particle update.** The particle moves along its flight direction and takes one ranged draw
    R(0, 2147483646) that varies its speed slightly. Then it takes a second ranged draw for its
    colour, as in claim 16. That makes two draws per update, always. It never collides and deals
    no damage. It ends only when its lifetime runs out (claim 3). For this kind the lifetime
    variation is 0 … 9 ticks (S-0202 claim 15).
    - Backing: `measurable` (scenario P12)
    - Confidence: high

### Timing, bad data and ends

19. **Normalised stage timing.** A particle's own stage-advance value is its type's stage-advance
    field. When the type has the normalised flag, it is instead set at creation from the distance
    to its target point and the particle's speed, so that its stage reaches the last damaging
    stage at about the time it arrives (F13). Only fire uses the particle's own value (claim 12).
    Smoke uses the type's field (claim 6). The normalised value takes no draw (S-0202 claim 15).
    - Backing: `measurable` (scenario P9, part b: a second test fire type with the normalised flag,
      fired at a target at a known distance. The stage reaches the last damaging stage at about the
      tick the particle reaches its target point, as F13 says. Falsified by a stage that differs by
      more than one step at that tick. P18 covers a last damaging stage of −1.)
    - Confidence: high for the rule; medium for the single-precision domain of F13, which P9 part b tests only to one step.

20. **Zero divisors, missing held types and a spark-turns field of 0 or less.** These inputs can make the original stop, give an undefined result, fault or (for the spark-turns field) keep an emitter place held until another cause ends the system. The fire insertion case (F7) is a fault on legal play, and the negative-radius case is one whose original behaviour is not known (Open questions). Each is a **departure** (no switch). The importer warns for the data cases, not for the fire insertion case. Integer divisions by zero follow S-0205 claim 3 (quotient 0, remainder equal to the
    dividend) unless this claim says otherwise:
    - **Emission timer below 1** (⌊*T*⌋ = 0, for example a spawn interval of 0): by S-0205 claim 3
      the tick-number test passes only at tick 0, so the system emits at tick 0 only (or never,
      when created later), for as long as ⌊*T*⌋ stays 0. Once the slow-down field (claim 9) raises
      *T* to 1 or more, the system emits as claim 9 says.
    - **Fire spawn interval 0:** likewise, the interval test passes only at tick 0. A turn with a
      sweep still emits on every third tick.
    - **Smoke successor with *d* = 0** (F3: a radius field from 0 to 7 on the type of a particle of any kind that dies in a smoke system and whose type names a
      next type): by S-0205 claim 3, *q* is the raw value itself and *o* = *q*. Both raw values are
      still taken.
    - **Spark velocity fields of 0** (F10) and a smoke spawn radius of −1 (F4): the remainder is
      the raw value itself, by S-0205 claim 3.
    - **Spark burst with *h* = 0** (F10: a particle cap of −1, 0 or 1): S-0205 claim 3 would give a
      count of up to 2³¹ sparks, which no engine can make. Our engine overrides it: it takes the
      count raw value only and makes no spark.
    - **Fire type with a non-zero damage amount and no warhead** (claim 14): the original has no warhead to hit with and fails. Our engine makes the hits deal no damage, with an importer warning; the countdown, the hit count and the draws are unchanged.
    - **Negative radius or spawn radius** (F3 with *d* < 0, F4 with *D* < 0): the formulas apply unchanged with the divisor's sign kept (the remainder takes the sign of the raw value, S-0205 claim 3, so it is the same as for the divisor's magnitude). How a negative radius field is divided by 8 (rounding down or toward zero) is not known; our engine rounds down, as F3 says (Open questions). Backing `unmeasured`; confidence low. The importer warns.
    - **Fire insertion with a negative *j*** (F7): *j* = 0 instead, as F7 says.
    - **Spark or railgun system with no held type** that bursts or emits a line with *n* ≥ 1 (not a
      division: the original uses a missing particle): our engine takes the decision draw, if any,
      and nothing else, and makes no particle. The railgun system is still marked, and the spark
      counter still counts down. A railgun system therefore takes no ranged draw at all in its line
      turn: none of claim 17's per-particle draws, for any particle of the line. This case arises
      through a no-type reading of the held-type key, such as `<none>` (Import mapping). In the original,
      when the line count *n* of claim 17 is at least 1, such a railgun system stops the game
      with an error. Our engine
      instead makes claim 31's laser beam when the type has the laser flag, exactly as for a
      system with a held type: in every line turn the original completes, the beam depends only
      on the laser flag (it is made without a held type when *n* ≤ 0, claim 31). The importer
      warns for a railgun system type with no held type and a positive particles-per-lepton
      field. This is a bad-data departure (glossary, *departure*): the safe result, the beam, is
      this spec's choice. (Q-1590 question 1.)
    - **Spark-turns field of 0 or less** (claim 15; also the absent-key value, Import mapping):
      the original does not stop here, but such a system is never marked by its counter. It ends
      only by its owner's removal (claim 4), a positive lifetime, the trigger action of claim 1
      or, while it is held in the fire place, a let-go of that place (claim 1): the let-go marks
      it, and since it makes no burst it holds no particle and leaves play in its next turn
      (claim 22). Neither a move order, nor a new target, nor the firer's death lets a system held
      in the spark place go (claim 1), so in the original a spark place that holds such a system
      stays held, and every weapon of the firer that S-0301 claim 5's gate ties to that place stays
      refused while the system remains in play. Owner removal, a positive lifetime or a trigger
      mark can end that refusal.
      Our engine fixes this outright (lead decision, no switch): such a system takes no burst and
      no draw of claim 15 and is marked in its first turn. It then leaves play in the first turn
      in which it holds no particle (claim 22), so an emitter, which starts empty, leaves in its
      first turn and frees its place. This applies to every spark system of such a type,
      whoever made it and whatever place holds it, so after that turn a let-go of the fire place
      finds the place free. For a damage-sparks system of such a type (claim 1) the fix frees the
      spark place after one turn, so the object's per-turn decision draw resumes from its next
      turn, where the original takes no further damage-sparks draw once the place is held: an
      expected draw-count difference, not a mismatch. The importer warns for a nonpositive value, including the absent-key default.

    - Backing (the original): `unmeasured`: the original stops or faults in these cases, so there is no behaviour to compare; the negative-radius case is not known (Open questions). One case is an exception: for the spark-turns case the original's behaviour is `measurable` (scenario P22, part c). The railgun line with *n* ≥ 1 and no held type is a second exception: that the original stops is `measurable` (scenario P31 part c, with part d establishing, through the missing first particle at a detonation, that the `<none>` reading gives no held type).
    - Confidence (the original): high that the original stops or faults in the cases other than the railgun line; high for the original's spark-turns behaviour outside the fire place; medium for its let-go end while held in the fire place, which is `unmeasured` (claim 1's place-and-kind items); medium for the railgun stop, until P31 parts c and d have run.
    - Our engine's behaviour in every case of this claim, the railgun beam with *n* ≥ 1 and no held type included, is a departure by decision. Backing: `unmeasured` (a design decision; no measurement of the original decides it). Confidence: high (fixed by this claim).

20a. **Very large spark and railgun counts.** The original sets no limit on the sparks of one
    burst (*c*, F10) or the particles of one railgun line (*n*, F12), and none on the particles
    of a system or of the whole game: it makes them one after another in the one turn, each with
    its draws (claims 15 and 17). This holds for a system with a held type; claim 20 gives the
    original's result for one without. A particle cap near 2³¹ asks for up to about 2³¹ sparks,
    and a line asks for more particles the larger its particles-per-lepton field and its distance
    (F12 gives the count's wrap past 2³¹). The original makes particles until the program can get
    no room for a further one; it then fails (unlike a smoke successor, which is skipped, claim
    8a), so a large enough count stops the game, while a smaller one only makes a long turn,
    after which play goes on. The data is not refused or changed at load.

    Our engine departs from this, with no switch: a burst or a line makes at most 4096 particles.
    A burst whose *c* is above 4096 takes the count raw value and the three discarded raw values
    as F10 gives, then makes 4096 sparks, each with its creation draws and its three velocity raw
    values, in claim 15's order; its light draw follows as usual. A line whose *n* is above 4096
    makes 4096 particles, each with claim 17's three ranged draws, its creation draws and four
    more ranged draws, in that order, and is placed along the line as a line of 4096 particles
    would be (drawing only). Counts up to 4096, and the bursts and lines that have them, are
    unchanged. The limit applies only to a burst or line that makes particles: a burst of a spark
    system with a held type and *h* ≥ 1, and a line of a railgun system with a held type. A spark
    system with no held type or with *h* = 0, and a railgun system with no held type, follow
    claim 20 whatever their count: they take only the draws that claim 20 gives and make no
    particle. A spark system whose spark-turns field is 0 or less makes no burst (claims 15 and
    20), and a turn whose decision fails, a burst whose *c* is 0 or less and a line with *n* ≤ 0
    make no particle (claims 15 and 17, F10, F12). None of these cases is changed by the limit.
    - Backing for the original: `measurable` (scenario P33: a count above 4096 that the original
      completes, counted by the draws its turn takes). `unmeasured` for the point at which the
      original fails, since that depends on the room the program has, and for the absence of a
      limit on either count as such, since no finite run shows every count. Confidence: medium
      that no limit is checked on either count; medium that a count far beyond the program's room
      stops the game; low for the count at which it does.
    - Backing for the departure: a design rule, not an oracle value; `measurable` for our engine
      (P33's engine runs). Confidence: high for the departure, a design rule.

21. **Absent held-type key.** A particle system type section that is read without a held-type key
    names, as its held type, a particle type with the empty name. That type is registered by
    reference when the section is read, at the end of the particle type list (S-0004), unless an
    earlier section already made it. No section of the rules has the empty name, so it keeps every
    absent-key default: no behaviour kind, lifetime field 1. Such a system does get a first particle
    at a detonation (S-0613 claim 4), with its creation draws. A particle with no kind ends after
    one update in a system of any kind (claim 3), and never in a system with no kind. A system type
    that is never read has no held type.
    - Backing: `measurable` (scenario P13)
    - Confidence: high

22. **When systems leave play (Q-1243 question 2).** S-0613 claim 17 gives the common rule: a
    marked system leaves play in the first turn, after its count-down, in which it holds no
    particle, and a positive lifetime marks it. Claim 4 marks every system whose owner object is
    removed, and claim 1's healing marks (after a hit or a self-healing gain) mark a damage-smoke system. Claim 1's trigger action marks
    every system in the waypoint's cell, whatever its kind. Each kind adds its own marks, and
    claim 1's columns decide which can happen. Claim 1's cloak removal of natural smoke does not
    mark a system: it removes the system at once, with its particles. The table
    assumes a lifetime of −1, the absent-key value.

    | Kind | Starts empty, with an owner object | Starts empty, no owner object | Starts with a particle, no owner object (a detonation) |
    |---|---|---|---|
    | Smoke | Marked when *T* passes the cut-off (claim 9), the owner is removed, a damage-smoke healing mark of claim 1 occurs (including its below-surface test) or, while it is held in the fire place, that place is let go (claim 1). | Marked when *T* passes the cut-off. | Marked when *T* passes the cut-off. The first particle (made at the detonation) is an extra particle beyond the emissions; the system leaves in the first turn after the mark in which it holds none. |
    | Fire | Marked when the owner is no longer alive (claim 10), is removed, or, while it is held in the fire place, when that place is let go (claim 1). Held in the spark or railgun place, it is not marked by its own turn while its owner is alive and on the map. | Marked in its first turn. Never emits. | Marked in its first turn. Never emits. Leaves when its first particle is gone. |
    | Spark | Marked when *K* runs out (claim 15), when the owner is removed or, while it is held in the fire place, when that place is let go (claim 1); in its first turn when the spark-turns field is 0 or less (claim 20). | Marked when *K* runs out, or in its first turn as in the column before. | Marked when *K* runs out, or in its first turn as in the first column. The first particle falls from rest (claim 16). |
    | Railgun | Marked after its line (claim 17), when the owner is removed or, while it is held in the fire place, when that place is let go (claim 1). | Marked after its line. | Marked in the first turn after its first particle is gone: its target point is its own position, so its line has no particle (claim 17). It leaves in that turn. |
    | No kind | Marked when the owner is removed, by the trigger action or, while it is held in the fire place, when that place is let go (claim 1); leaves only if empty (claim 2). | Marked only by the trigger action (claim 1). | Leaves only after a trigger mark, once empty (claim 2). |

    A spark system whose type's spark-turns field is 0 or less is marked in its first turn by
    claim 20 (in the original it would never reach its counter's mark, but owner removal, a
    positive lifetime, a trigger mark or, while it is held in the fire place, a let-go of that
    place could still end it). A gas-kind system held in the fire place is likewise marked when
    that place is let go (claim 1); its other marks are S-0613's. A smoke system that claim 9
    never marks keeps emitting, and each emission takes its two raw values whether or not
    the system has a held type (claim 9). With a positive lifetime the system is also marked in
    that turn (S-0613 claim 17).
    - Backing: `measurable` (scenarios P1 and P14). The table's fire-place let-go entries, as
      claim 1 backs them: for a fire-kind system by scenario P1; for a spark-kind system by S-0301
      scenario F4's second run (u1 against u8); for a smoke-kind system by its third run (u10).
      They are `unmeasured` for a railgun-kind, gas-kind or no-kind system, for the reasons claim
      1's place-and-kind backing gives.
    - Confidence: high. The table's fire-place let-go entries: medium, as claim 1 rates a let-go's
      mark of the system held in the fire place.

### Drawing (presentation only)

Claims 23 to 31 answer Q-1520; claims 27 and 28 also answer Q-1540 (depth profile, depth bias,
and that no particle is drawn without a depth test). Drawing is done by the client and is presentation only: drawing
itself changes no game state, takes no draw from the scenario random stream and feeds nothing
back into play (claim 2). A draw taken during a particle-system turn may affect only presentation,
as claim 15 specifies for its light draw. The colour progress of claim 29 is particle state that
only drawing reads; its update draws are those of claims 16 and 18. Every claim here follows the
particle's own kind, not its system's, as damage does in claim 3. *Sprite kinds* are gas, smoke,
fire and no kind; *pixel kinds* are spark and railgun.

23. **Which particles are drawn.** Each drawn frame, every particle in play is drawn once, in the
    air layer (claim 30), unless one of these holds:
    - S-0613 claim 16's performance cutoff hides it (a type that deals no damage, while the
      drawn-frame rate is below the cutoff). This applies to every kind, not only gas.
    - The player's detail setting is the lowest and the particle is of the smoke or spark kind.
      Gas, fire, railgun and no-kind particles are still drawn at the lowest setting.
    - Fog of war is in force and the particle's position is fogged. Fog is not reachable in
      skirmish (S-0104 claim 15), so this never hides a particle here.
    - It is of a sprite kind and its type has no sprite (S-0007 claim 28): it draws nothing.
    A particle of a pixel kind never uses its sprite, its translucency or its stage for drawing,
    and a particle of a sprite kind never uses its colour fields. A particle with the dead mark is
    still drawn in any drawn frame that comes before its removal pass.
    - Backing: `measurable` (scenario P24)
    - Confidence: high for the detail and kind rules; medium for the fog rule, which skirmish
      cannot show.

24. **Palette and brightness of a sprite particle.** A particle of a sprite kind is drawn with the
    anim palette at brightness 1000 ‰ (neutral). Its cell's light level, its height and the map's
    lighting do not change that brightness, and the anim palette takes no tint (S-0104 claims 3
    and 6). It is never drawn in a house-colour palette, whatever the system's owner house. The
    per-pixel light-alpha value of S-0104 claim 15 still applies, as for any sprite drawn at a
    fixed brightness: shroud and fog pieces darken it and light-glow images change it (S-0110
    claims 5 and 6). So under the default light-alpha of 127 a particle shows the same shade row
    on every cell. The unscaled ground level of the cell (S-0103 claim 8 row 5) does not hold
    for particles.
    - Backing: `measurable` (scenario P25)
    - Confidence: high for the palette and the fixed brightness; medium for the light-alpha
      step, which is the same drawing path as the other fixed-brightness sprites.

25. **Image frame of a sprite particle.** The drawn image frame number is:
    - **Gas and smoke:** the particle stage itself (S-0613 claim 15 for gas).
    - **No kind:** frame 0, always.
    - **Fire:** *s* + *g* × *E*, with *s* the particle stage, *E* the type's end-stage field (both
      signed) and *g* (0, 1, 2 or 3) the *direction group* of F15. The group comes from the
      on-screen direction from the particle's creation position to the target point it was given
      at creation, so it is fixed for the particle's whole life. Opposite screen directions share
      one group, and nothing is mirrored. So a fire sprite holds four runs of *E* frames: for
      flight up or down the screen, up-right or down-left, left or right, and down-right or up-left.
    The type's start-frame and loop-frame fields are read but never used: no kind adds the start
    frame. A fire particle that stays at its end stage (no delete-at-end flag) shows frame
    (*g* + 1) × *E*, the first frame of the next group, or past the last group when *g* = 3.
    Our engine draws nothing for a frame number that the sprite file does not hold (a negative
    one included); what the original draws then is open (Open questions).
    - Backing: `measurable` (scenario P26)
    - Confidence: high for the formulas and the unused fields; medium for the group boundaries
      at exact screen slopes (F15).

26. **Translucency of a particle.** Each particle keeps its own translucency value, an 8-bit
    value, for its whole life. It is drawn by S-0613 claim 16's rule, for every sprite kind: at the
    highest detail setting a value of exactly 25 or 50 draws at that translucency, a value whose
    signed reading is 75 … 127 draws at 75, and every other value draws opaque. At the other
    settings every particle draws opaque. So a value below 75 other than 25 or 50 (30, say) draws
    opaque, and a value of 128 or more, read as negative, draws opaque. The value changes only as
    follows, each change kept modulo 256:
    - **At creation:** the type's translucency field (S-0613 claim 16), then any smoke emission
      step of claim 9.
    - **Successor made in a smoke system (claim 8), whatever its kind:** the successor's value replaces its own type's value by the
      dead particle's, plus 25 when its draw says so. So a chain of successors steps through 0, 25,
      50, 75, 100, 125, 150 … and draws opaque, 25, 50, 75, 75, 75 and then opaque again, from
      150, which reads as −106. So a successor of a successor can be more translucent again, up
      to 75, and is drawn opaque once its value passes 127.
    - **Fire stage fields:** at an update in which a fire particle's stage advances (claim 12),
      when the new stage equals the type's translucent-25 stage field the value becomes 25, and
      when it equals the translucent-50 stage field it becomes 50. The value is set, not added.
      When both fields name that stage, 50 wins; otherwise the field reached later wins. The test
      is made only at an advance, so a field equal to the start stage, a field above the end stage
      and a field of 128 or more (the absent value 255 included) never take effect.
    No other kind uses the stage fields, and gas, spark and railgun values never change after
    creation. The pixel kinds never draw translucently (claim 28).
    - Backing: `measurable` (scenario P27)
    - Confidence: high

27. **Placement, depth and shadow of a sprite particle.** The draw point is the screen pixel of the
    particle's position, with its height lift (S-0101 F1). The image frame is placed centred on it
    (S-0102 claim 19). The drawing reads depth and never writes it, with an upright depth profile
    and the depth bias −15 − *L*(*z*), *z* the height of the particle's position in leptons
    (S-0102 claim 13 and F2). No shadow is drawn for any particle.
    - **Same for every sprite kind.** The profile and the bias are the same for gas, smoke, fire
      and no-kind particles, at every detail setting and translucency, and no field of the
      particle type or the system type changes them. The value −15 − *L*(*z*) is final: nothing
      further is added (no −2 as for animations and sprite units, and no depth-adjustment field,
      which particle types do not have). So, like any upright reader, a particle is hidden only by
      depth writers (terrain tiles, overlays, trees, building images; S-0102 claim 10) whose stored
      depth is nearer than its own. Its overlap with another particle, a unit, a projectile or an
      animation is decided by draw order alone (claim 30), never through depth.
    - Backing: `measurable` (scenario P28)
    - Confidence: high for the centring, the absence of a shadow, the upright profile and the
      constant −15; medium for the depth order that results on screen, until P28 is run.

28. **A spark or railgun particle is one pixel.** A particle of a pixel kind is drawn as a single
    pixel, never as a sprite or a line, at the screen pixel of its position with its height lift
    (S-0101 F1), and only when that pixel lies inside the battlefield view. With *a* the
    pixel's light-alpha value (S-0104 claim 15) and *d* the screen-row depth of its row (S-0102
    F1):
    - it is not drawn when *a* = 0 (fully shrouded).
    - it is drawn only when *d* − *L*(*z*) − 50 is strictly less than the stored depth there,
      with no rounding of that value, and it never writes depth.
    - its colour is the particle's current colour (F16). Where *a* < 127, each 8-bit channel is
      multiplied by *a* and divided by 128, truncated. Where *a* ≥ 127 the colour is unchanged,
      so a light-glow image can darken a spark but never brighten it.
    - the colour is used as it is, with no palette, no tint and no cell light, and is packed to
      the display's 16-bit pixel format (S-0103 claim 9).
    - it is always opaque, at every detail setting.
    The railgun's helix is just the arrangement of these pixels (claim 17); its laser is a separate
    drawing (claim 31).
    - **No particle is drawn without a depth test.** Every drawn particle is depth-tested and none
      writes depth: a sprite particle by claim 27, a pixel particle by the test above, at every
      detail setting. The test above is S-0102 F2's flat profile on a one-row drawing with bias
      −50 − *L*(*z*), so no rounding Q applies. No particle is drawn as a line; the line seen with
      a railgun is the system's laser (claim 31), whose depth handling belongs to the laser's own
      spec (Q-1521).
    - Backing: `measurable` (scenario P29)
    - Confidence: high for the single pixel, the colour and the light-alpha rule; medium for the
      depth value.

29. **How a spark or railgun colour moves through the list.** Each particle holds an *own colour*,
    a *colour index* *i* and a *colour progress* *t*, set at creation and changed only by the
    particle's updates (claims 16 and 18). Its drawn colour runs from its own colour to list
    entry 1 while *i* = 0, and then from entry *i* to entry *i* + 1, F16 giving the mix. Each
    update adds to *t* the type's colour-speed field plus a small random amount, from that
    update's colour draw (F16). When *t* passes 1.0 the particle moves on to the next pair, with
    *t* back at 0; the amount above 1.0 is dropped. At the last pair it stays, with *t* held at
    1.0, so the colour stops at the last list entry. List entry 0 is shown only through the own
    colour, which is entry 0 itself when both start colours are black, and otherwise a mix of the
    two start colours drawn at creation (S-0202 claim 15, F16).
    - **Short lists (departure, no switch).** With one entry the original reads past the end of
      the list, and with an empty list it fails when such a particle is drawn. Our engine uses the
      own colour as the colour to move towards when the list has no entry *i* + 1, so a one-entry
      list draws its own colour, and an empty list draws black. The importer warns for a spark or
      railgun type with fewer than two entries. Drawing only.
    - Backing: `measurable` (scenario P29)
    - Confidence: high

30. **Draw order of particles.** S-0102 claims 3 and 7 give the layer and its order: particles are
    in the air layer, which is never sorted, and an object entering it goes to the end of the
    list. A particle enters the air layer when it is made and keeps its place until it is
    removed; removing others does not move it. So, within the air layer, particles are drawn in
    the order in which they were made, oldest first, across all systems together, interleaved
    with the other air-layer objects (projectiles, animations) by the same rule. They are not
    grouped by system or sorted by position, and a successor or a fire particle inserted
    inside its system's list (F7) still goes to the end of the layer. The order inside a system's
    own list (claims 5, 10, 15, 17) does not affect drawing.
    - Backing: `measurable` (scenario P30)
    - Confidence: high

31. **System lights and the railgun laser.** A particle system draws nothing of its own except
    these, all presentation only:
    - **One-frame light.** A system whose type has the one-frame-light flag and a positive light
      size, holding at least one particle, shows a spotlight at its own position in every drawn
      frame, whatever the detail setting. Its size is trunc(λ × *f*), λ the light size and *f* the
      particle count divided by the system type's particle-cap field, clamped to 0.4 … 1.0, in
      floating point.
    - **Spark light.** A spark system whose type has a positive light size and no one-frame-light
      flag makes one lasting spotlight of that size at its position when its first turn bursts
      (claim 15), at the highest detail setting only. A later burst makes none.
    - **Railgun laser.** A railgun system whose type has the laser flag makes one laser beam in
      its line turn (claim 17), from the system's position to its target point: inner colour the
      type's laser colour, outer colour and spread black, lasting 10 ticks, not blinking, fading
      from intensity 0.5 to 0. The beam is made whatever the line
      count *n* in every line turn the original completes: when *n* ≤ 0 (no particle), with or
      without a held type, and when *n* ≥ 1 with a held type. The case of *n* ≥ 1 without a held
      type is claim 20's, for the original and for our engine. A system whose target point is its own position
      (claim 22, a detonation railgun) makes a beam of zero length; whether it shows is the
      beam drawing's (S-0518). (Q-1590.) How a laser beam
      looks is Q-1521's subject.
    How a spotlight looks, and how the light draw of claim 15 changes it, is open (Open questions).
    - Backing: `measurable` (scenario P31: the main part for the beam with a held type and
      *n* ≥ 1; part b for the beam with *n* = 0, with and without a held type, part d
      establishing that the run written `<none>` has none). The beam with *n* < 0, from a
      negative particles-per-lepton field, is `unmeasured`: P31 samples only *n* = 0, and a
      negative field is bad data no stock type uses. The zero-length beam of a detonation
      railgun is `unmeasured`: whether it shows is the beam drawing's (S-0518). The case of
      *n* ≥ 1 without a held type has its backing in claim 20.
    - Confidence: medium for the lights; medium that the beam is made in every line turn the
      original completes, with or without a held type, at *n* = 0 and *n* ≥ 1 (P31's main part and
      parts b and d sample it); low for *n* < 0; low for the zero-length beam.

## Formulas

All positions are integer leptons. "Single precision" and "double precision" are the IEEE formats
of S-0205. Every floating-point result is formed with the 53-bit significand and the rounding of S-0205 (claims 4 and 5), and this spec defines no switch of its own. Where an expression is formed in double precision and only then stored in single precision, with no single-precision rounding of its intermediate results, this is said. ⌊·⌋ on a negative real rounds down. trunc rounds toward zero. *a* rem *b* is
the signed remainder (S-0205 claim 3: the result has the sign of *a*). A *raw value* is one raw
draw of the scenario random stream (S-0202), read as a signed 32-bit integer. |*r*| of a raw value
is the absolute value in 32-bit two's complement arithmetic: |*r*| = *r* for *r* ≥ 0 and −*r* for
*r* < 0, except that |−2³¹| = −2³¹ (the result stays negative). The raw value −2³¹ occurs with chance 2⁻³² per draw, and the claims
that use |·| say what it gives there. *r*(*n*) is the table
square root of S-0204 F1, and sinₜ and cosₜ are the table sine and cosine of S-0105 F1.

**F1. Heading.** For a particle made at *P* with target point *Q*, let *v* = *Q* − *P*, converted
to single precision per component. With *m* = *r*(*v*ₓ² + *v*ᵧ² + *v*_z²), the sum of squares
formed in double precision, *u* = *v* / *m* in single precision when *m* ≠ 0, otherwise
*u* = (0, 0, 0). *P* is the position before the terrain floor of S-0613 claim 5 is applied.

**F2. Smoke move.** With the rules' wind direction ω (0 … 7), the type's wind-effect field *W*,
and the smoke wind steps (Δₓ, Δᵧ) of Constants (not S-0613's gas table; claim 7):

> *x*′ = *x* + Δₓ(ω) × *W* + *v*ₓ, *y*′ = *y* + Δᵧ(ω) × *W* + *v*ᵧ, *z*′ = *z* + trunc(*S*) + *v*_z.

Integer arithmetic. The bridge test uses *H*₀, the terrain height *H* at (*x*, *y*), and the deck
height 416 (Constants): the particle is stopped and marked when the cell at (*x*, *y*) is a bridge
cell, *z* < *H*₀ + 416 and *z*′ ≥ *H*₀ + 416 − 260 = *H*₀ + 156.

**F3. Successor displacements.** *d* = ⌊ρ / 8⌋ with ρ the dead particle's radius field (integer). For
each axis one raw value *r* gives *q* = *r* rem *d*, and

> *o* = *q* + *d* when *q* ≥ 1, otherwise *o* = *q* − *d*.

The formula needs *d* ≠ 0 (*d* = 0 is claim 20) and a negative *d* is claim 20's. So for *d* > 0, *o* lies in *d* + 1 … 2*d* − 1 or −2*d* + 1 … −*d*. The value *o* = *d* is never
produced, and *o* = −*d* is produced when *q* = 0.

**F4. Smoke emission.** With *D* = σ + 1, σ the type's spawn-radius field, and the two raw values
*r*₁, *r*₂ in that order, the particle is made at

> (*x*ₛ + (*r*₁ rem *D*), *y*ₛ + (*r*₂ rem *D*), *z*ₛ + 10),

where (*x*ₛ, *y*ₛ, *z*ₛ) is the system's position. The terrain floor of S-0613 claim 5 applies.
After it is made, with *T* the emission timer and *T*₀ the type's spawn interval:

> *S* = *S* − (*T* − *T*₀) × 0.35, raised to 2.0 when that stored result is below 2.0.

Formed in double precision, as S-0205 claim 4 has it, and then stored in single precision. The translucency test is
*T* > τ, with τ the system type's translucency cut-off, compared in single precision. With *D* = 0
(σ = −1) the remainder is the raw value itself (claim 20).

**F5. Flame sweep.** With the owner at *O*, its target object at *G* and its body facing *f* (an integer 0 … 65535, where values from 32768 up are taken as *f* − 65536, so the range is −32768 … 32767):

> *d* = trunc(*r*((*G*ₓ − *O*ₓ)² + (*G*ᵧ − *O*ᵧ)² + (*G*_z − *O*_z)²)),
> θ = (*f* − 16383) × (−2π / 65534), in double precision,
> target point = (trunc(*O*ₓ + cosₜ(θ) × *d*), trunc(*O*ᵧ − sinₜ(θ) × *d*), *O*_z).

The products and sums are formed in double precision, with no single-precision rounding between them. The constants 16383 and 65534 are exact, not 16384 and 65536 (`quirks.flame_sweep_facing`).

**F6. Flame swing.** A table *V* of 500 signed integers is fixed, the same in every game:
*V*[*i*] = trunc(sinₜ(0.125 × *i*) × 12.0 + 0.49), for *i* = 0 … 499, with the argument in radians (sinₜ takes radians, S-0105 F1) and the argument and the sum formed in double precision. At an emission in tick *t*,
with the system at *P* and target point *Q*:

> *k* = *t* rem 500, ℓ = *r*((*P*ₓ − *Q*ₓ)² + (*P*ᵧ − *Q*ᵧ)² + (*P*_z − *Q*_z)²) (sum in 32-bit integers, compared with 200.0 as a floating-point number),
> *m* = 3 when ℓ < 200.0, otherwise 1,
> *a* = trunc(12 × *V*[*k*] / (3 × *m*)), the quotient formed in double precision. Since 12 × *V*[*k*] is exact and a quotient that is a whole number is exact, *a* equals trunc(4 × *V*[*k*] / *m*) in exact arithmetic.

Let *δ* be the 8-way step direction (0 = toward decreasing *y*, counting clockwise in steps of
45°) from *P* to *Q*. It is chosen with the ratio *q* = (*P*ᵧ − *Q*ᵧ) / (*Q*ₓ − *P*ₓ) and
*t*₈ = sinₜ(π/8) / cosₜ(π/8), in double precision:
- *Q*ₓ = *P*ₓ: *δ* = 0 when *P*ᵧ − *Q*ᵧ > 0, else 4.
- *q* ≥ 1/*t*₈ or *q* < −1/*t*₈: the same as for *Q*ₓ = *P*ₓ.
- *t*₈ ≤ *q* < 1/*t*₈: *δ* = 1 when *Q*ₓ > *P*ₓ, else 5.
- −*t*₈ ≤ *q* < *t*₈: *δ* = 2 when *Q*ₓ > *P*ₓ, else 6.
- −1/*t*₈ ≤ *q* < −*t*₈: *δ* = 3 when *Q*ₓ > *P*ₓ, else 7.

The particle aims at *Q* + (*a* × *e*ₓ(*δ*), *a* × *e*ᵧ(*δ*), 0), with the side table *e* of
Constants. That is the direction a quarter turn clockwise of *δ*, scaled by *a*.

**F7. Fire insertion.** After the particle is made and added at the end of the list of *N*
particles (*N* counts it), *c* = min(4, *N*) and one raw value *r* is taken (also when *c* = 1),
giving *j* = |*r*| rem *c*. The new particle then has exactly *j* particles
of the list newer than it, so it is one of the newest *c* particles. It is not a ranged draw.
For *r* = −2³¹, *j* = 0 for *c* = 1, 2 and 4, and *j* = −2 for *c* = 3. A negative *j* is a
**departure**: the original then faults (crashes), so our engine takes *j* = 0 instead, which leaves the particle the newest of the list (no switch: the crash is the original's fault on legal play). `unmeasured` (chance 2⁻³² per
insertion, and only when the list holds exactly three particles); confidence low.

**F8. Fire step.** With one raw value *r*:

> *k* = (*r* rem 10) − 5, an integer from −14 to 4, φ = *k* × 0.02 + 1.0 (double precision),
> step = (trunc(fl(φ × *u*ₓ) × *S*), trunc(fl(φ × *u*ᵧ) × *S*), trunc(φ × *u*_z × *S*)),

where fl(·) rounds to single precision and the other products are formed in double precision. The z
component skips the single-precision rounding. The move adds the step to the position.

**F9. Fire distance.** *R* = trunc(*r*(Δₓ² + Δᵧ² + Δ_z²)), where Δ is the integer difference
between the object's position and the particle's position. The sum of squares is formed in double
precision. The hit's distance is trunc(*R* / 10), integer division.

**F10. Spark burst.** With the type's chance field χ (double precision), the decision passes when

> *r* × 4.656612877414201 × 10⁻¹⁰ ≤ χ,

for the ranged draw *r*, in double precision. The constant is the double nearest 1 / (2³¹ − 2), that is 1 / 2147483646, so that the top value of R(0, 2147483646) maps to 1.0.
With the type's particle cap κ, *h* = trunc(κ / 2), and the count raw value *r*₀:

> *c* = (|*r*₀| rem *h*) + *h*.

With *r*₀ = −2³¹ the remainder is zero or negative and *c* = *h* − (2³¹ rem *h*), which is at least
1 for *h* ≥ 1. *c* ≤ 0 makes no spark. The three unused raw values
follow and are discarded. Their values matter only through a zero divisor, which is claim 20's stopping case. For each
spark, after its creation draws, three raw values *r*ₓ, *r*ᵧ, *r*_z in that order give

> *w* = (*r*ₓ rem *X*, *r*ᵧ rem *Y*, (|*r*_z| rem *Z*ᵣ) + *Z*ₘ),

converted to single precision, where *X*, *Y* are the held type's x and y velocity fields, *Z*ᵣ
its vertical velocity range and *Z*ₘ its minimum vertical velocity. Then with the system type's
spawn direction *s* (three single-precision values):

> μ = *r*(|*w*|²), *w*′ = *w* + *s*, μ′ = *r*(|*w*′|²), velocity = μ × *w*′ / μ′ (*w*′ unchanged when μ′ = 0).

The sums of squares are formed in double precision and the components stored in single precision.
*w*′ is formed from the single-precision operands and stored in single precision; the product and
quotient of the velocity are formed in double precision and stored in single precision
(confidence medium; a last-place difference could move a landing only at a knife edge, and is
`unmeasured`). With the default spawn direction (0, 0, 0) the velocity is *w* up to rounding.

**F11. Spark motion and contact.** With *G* the rules' gravity (an integer, converted exactly):

> *v′*_z = fl(*v*_z − *G*), the stored vertical velocity after this update; *p* = fl(*A* + (*v*ₓ, *v*ᵧ, *v′*_z − *G*)) per component; *B* = trunc(*p*).

*A* is the integer position at the start of the update. Let *F* = *H*(*B*ₓ, *B*ᵧ) and the deck
height *E* = *F* + 416. When the cell at *A* or at *B* is a bridge cell:
- *B*_z < *E* ≤ *A*_z: contact from above; the height becomes *E*.
- *A*_z < *E* ≤ *B*_z: contact from below; the height becomes *E* − 20.

When neither crossing happens (including every cell that is not a bridge cell), with *p*_z the
single-precision new height:
- *p*_z < *F*: ground contact. The height becomes *F* when *p*_z > *F* − 100, otherwise it stays
  *p*_z.
- *F* ≤ *p*_z < *F* + 150 (tested as *p*_z − 150.0 < *F*), and the cell at *B* holds a building,
  or holds no building but counts as a wall for wall connection (Terms): contact; the height becomes *F*.
  Every building counts. Whether a gate whose door stands open, or a building in one further state, is excepted is not claimed (Open questions); the engine treats them as any building until a measurement decides.

On contact the particle gets the dead mark. Its final position is (trunc(*p*ₓ), trunc(*p*ᵧ),
trunc(height)). Without contact it is *B*. The velocity after a contact is not kept: the particle
is removed.

**F12. Railgun line size.** With Δ = target point − system position (integers): *L* =
trunc(*r*(Δₓ² + Δᵧ² + Δ_z²)), the sum in double precision, and *n* = trunc(*L* × *p*) in double
precision, truncated toward zero to a 64-bit integer and then wrapped to its low 32 bits as a
signed integer. A product with no 64-bit integer form (of magnitude 2⁶³ or more, infinite or not
a number) gives *n* = 0, and so no particle. So a product from 2³¹ to 2³² − 1 gives a negative
*n* and no particle, a larger one below 2⁶³ gives the wrapped value, and one of 2⁶³ or more gives
none (confidence medium; such a product needs a particles-per-lepton field far above any line's
distance in leptons, `unmeasured`). Claim 20a limits the particles made.

**F13. Normalised stage-advance value.** At creation, with *P* the creation position, *Q* the
target point, *u* the flight direction and *S* the speed:

> All of *u*, *S*, *a*ₓ, *a*ᵧ, *t*, the differences and the quotients below are single-precision (32-bit) floating point, each result rounded to single precision; *P* and *Q* are integers converted to single precision first. The division by (δ_F + 1) and the addition of 1.0 are likewise single precision, and the truncation is toward zero.
> *a*ₓ = |trunc(*u*ₓ × *S*)|, *a*ᵧ = |trunc(*u*ᵧ × *S*)|,
> Let *t*ₓ = fl(|*P*ₓ − *Q*ₓ| / *a*ₓ) when *a*ₓ > 10⁻⁶ (not capped at 9999.0), and *t*ᵧ = fl(|*P*ᵧ − *Q*ᵧ| / *a*ᵧ) when *a*ᵧ > 10⁻⁶. Then *t* is 9999.0 when neither test passes, *t*ₓ when only the first passes, min(9999.0, *t*ᵧ) when only the second passes, and min(*t*ₓ, *t*ᵧ) when both pass.
> value = trunc(*t* / (δ_F + 1) + 1.0), reduced modulo 256 into the signed range −128 … 127,

with δ_F the type's last damaging stage. The last stage's absent default is the end stage
(Import mapping). With δ_F = −1 the divisor is 0. For *t* > 0 the quotient is infinite, and for *t* = 0 it is
undefined (0 / 0). In both cases the original gives the value 0, since a quotient with no integer
form converts to 0; this is no departure (confidence
medium; a zero divisor in floating point does not stop the original, unlike claim 20's integer
divisions). With δ_F < −1 the divisor is negative and the formula applies unchanged.

**F14. Railgun beam end point.** Computed once, at the shot that makes the system of the railgun
place, whatever its kind (claim 1). *P* is the emitter's position (the fire point, integers)
and *Q* the target's position at the shot (integers; the target's own position, with no further
raise). Let δ be the fixed
single-precision value nearest 1/49, exactly 10 956 549 / 536 870 912 (about 0.020408163).
This constant is unchanged by the rounding mode; it is not a fresh division in that mode.
For *i* = 0 … 49:

> *t*ᵢ = fl(*i* × δ), *s*ᵢ = trunc((*Q* × *t*ᵢ) + (*P* × (1 − *t*ᵢ))) per component.

The product for *t*ᵢ is formed in double precision and then narrowed to single precision.
Every subtraction, product and sum in the sample position is formed in double precision,
with no single-precision narrowing, followed by truncation toward zero to integer leptons.
The integers enter those products exactly. Each arithmetic result and the narrowing follow
S-0205 claims 4 and 5, including `quirks.chop_rounding`; no multiply-add is fused and the
weighted sum is not replaced by a difference-based interpolation.

**Last sample (Q-1405 question 1).** Every *t*ᵢ, including *t*₄₉, uses that same rounding rule:
there is no special assignment of 1.0 at the last sample. With the original's round-toward-zero
mode, *t*₄₉ = 1 − 2⁻²⁴, since the exact product 49 × δ = 1 − 11 × 2⁻²⁹ narrows downward.
Thus *s*₀ = *P*, but *s*₄₉ need not equal *Q*. For example, *P* = (2688, 2688, 60) and
*Q* = (3840, 2688, 0), in leptons, give *s*₄₉ = (3839, 2688, 0), in the cell before *Q*'s.
With `quirks.chop_rounding` off, every product and narrowing rounds to nearest instead.
*t*₄₉ is then 1.0 and *s*₄₉ = *Q*. Earlier parameters can differ too: *t*₃ is
0.06122448667883873 in the original mode and 0.06122449040412903 with the switch off
(the decimals denote the respective single-precision values).

The samples are visited in the order of *i*. A sample is tested only when its cell (trunc(*x* / 256), trunc(*y* / 256))
differs from the cell of the sample before it (for *i* = 0, the cell of *P*); a sample in the same
cell as the one before it is not tested. A tested sample *s*ᵢ whose height is below the terrain
height there, *s*ᵢ,z < *H*(*s*ᵢ,x, *s*ᵢ,y) (strictly), is the end point, and no later sample is
visited. When no tested sample is below the terrain, the end point is *Q*. *H* is the terrain
height of Terms, never a bridge deck, so a beam passing under or over a deck is not stopped by it.
In particular the no-stop end point is the exact *Q*, not *s*₄₉, even when the samples never
enter *Q*'s cell. Candidate hits and the target fallback remain S-0306 claim 27's rules,
including its line-distance and degenerate-beam cases (Q-1405 questions 2 and 3).
- Backing: `measurable` (P22 parts b and d for claim 1's sampling and end point).
  No oracle measurement is claimed. For a system of another kind in the railgun place:
  `unmeasured`, as claim 1's place-and-kind backing gives.
- Confidence: high for a railgun-kind system. For a system of another kind in the railgun
  place: medium (claim 1's creation reading).

**F15. Fire direction group.** With *P*₁ = (*x*₁, *y*₁) and *P*₂ = (*x*₂, *y*₂) the screen pixels
(S-0101 F1, height lift included) of the particle's creation position and of the target point it
was given, let Δ*x* = *x*₂ − *x*₁ and Δ*y* = *y*₁ − *y*₂ (positive up the screen), integers. Let
τ = sinₜ(π/8) / cosₜ(π/8), about 0.41421, and, when Δ*x* ≠ 0, σ = Δ*y* / Δ*x*, both in floating
point.

| Condition | Screen direction | Group *g* |
|---|---|---|
| Δ*x* = 0, σ ≥ 1/τ, or σ < −1/τ | up or down (a zero vector counts here) | 0 |
| τ ≤ σ < 1/τ | up-right or down-left | 1 |
| −τ ≤ σ < τ | right or left | 2 |
| −1/τ ≤ σ < −τ | down-right or up-left | 3 |

The screen difference does not depend on the view position. Confidence medium for the exact
boundaries, which depend on the table values; the bands are high.

**F16. Particle colour** (pixel kinds). Own colour *c*₀ at creation, per 8-bit channel, with the
creation colour draw *r* of S-0202 claim 15 and *f* = *r* × 4.656 613 × 10⁻¹⁰ in single precision:

> *c*₀ = trunc(*s*₁ × (1 − *f*) + *s*₂ × *f*), clamped to 0 … 255,

*s*₁ and *s*₂ being the two start colours. With both start colours black, *c*₀ is list entry 0
and there is no draw. With an empty list, *c*₀ is black. At creation *i* = 0 and *t* = 0.

At each update, with the update's colour draw *r* (claims 16 and 18) and *v* the type's
colour-speed field, in double precision:

> *t* ← (*r* × 4.656612877414201 × 10⁻¹⁰ × 0.05 + *v*) + *t*, summed in that order.
> If then *t* > 1.0: when *i* < *n* − 2, *t* ← 0 and *i* ← *i* + 1; otherwise *t* ← 1.0.

*n* is the number of list entries. The drawn colour, per channel, with *A* = *c*₀ when *i* = 0
and list entry *i* otherwise, and *B* list entry *i* + 1 (*A* when the list has no such entry,
claim 29):

> *c* = trunc(*A* × (1 − *t*) + *B* × *t*), in floating point.

Then the light-alpha step of claim 28: *c* ← ⌊*c* × *a* / 128⌋ when *a* < 127.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Smoke drift step | −1, 0, +1 | leptons per tick | claim 6, chance 1/4 per odd tick |
| Smoke velocity limit | ±5 | leptons per tick | claim 6 |
| Smoke slowing floor | 3.0 | leptons per tick | claim 6 |
| Smoke emission height | 10 | leptons | F4 |
| Smoke emission speed loss | 0.35 | leptons per tick per unit of *T* | F4 |
| Smoke emission minimum speed | 2.0 | leptons per tick | F4 |
| Sprite particle brightness | 1000 | ‰ | claim 24 |
| Sprite particle depth bias | −15 − *L*(*z*) | depth units | claim 27 |
| Pixel particle depth bias | −50 − *L*(*z*) | depth units, on the pixel's own row, no rounding | claim 28 |
| Pixel full light-alpha | 127 | light-alpha value; below it channels are scaled by *a* / 128 | claim 28 |
| Colour progress jitter | 0.05 | dimensionless, times the scaled colour draw | F16 |
| Fire direction ratio τ | sinₜ(π/8) / cosₜ(π/8), about 0.41421 | dimensionless | F15 |
| One-frame light size range | 0.4 … 1.0 | dimensionless factor on the light size | claim 31 |
| Railgun laser | 10 ticks, intensity 0.5 to 0 | ticks, dimensionless | claim 31 |
| Smoke translucency step | 25 | percentage points of translucency (Terms) | claims 8, 9 (drawn only) |
| Successor radius divisor | 8 | dimensionless (applied to the radius field, in leptons) | F3 |
| Fire swing amplitude divisor | 3 × *m*, with *m* = 3 when ℓ < 200.0 (near) and 1 otherwise (far) | dimensionless | F6 |
| Bridge deck height | 416 | leptons above *H* (F2, F11) or above the cell's height level × 104 (claim 14) | F2, F11, claim 14 |
| Smoke deck margin | 260 | leptons below the deck | F2 (2.5 levels of 104) |
| Smoke wind steps Δₓ by ω = 0 … 7 | 0, 2, 2, 2, 0, −2, −2, −2 | leptons per unit of *W* | F2 |
| Smoke wind steps Δᵧ by ω = 0 … 7 | −2, −2, 0, 2, 2, 2, 0, −2 | leptons per unit of *W* | F2 |
| Fire sweep cadence | 3 | ticks | claim 11 |
| Fire emission height | 1 | lepton | claim 11 |
| Fire swing table | 500 entries, amplitude 12 | dimensionless; F6 scales it by 4 / *m* to leptons | F6 |
| Fire swing near distance | 200.0 | leptons | F6 |
| Fire side table *e*ₓ by δ = 0 … 7 | 1, 1, 0, −1, −1, −1, 0, 1 | unit steps (dimensionless) | F6 |
| Fire side table *e*ᵧ by δ = 0 … 7 | 0, 1, 1, 1, 0, −1, −1, −1 | unit steps (dimensionless) | F6 |
| Fire insertion depth | 4 | particles | F7 |
| Fire step variation | 0.02 per unit, *k* from −14 to 4 | dimensionless factor per unit of *k* | F8 |
| Fire distance divisor | 10 | dimensionless: leptons of *R* per lepton of hit distance (S-0302 gives hit distance in leptons) | F9 |
| Spark wall and building height | 150 | leptons above *H* | F11 |
| Spark under-deck placement | 20 | leptons below the deck | F11 |
| Spark ground snap depth | 100 | leptons below *H* | F11 |
| Draws per live spark update | 1 | ranged draws | claim 16 |
| Draws per railgun line particle | 7 | ranged draws, besides creation | claim 17 |
| Draws per live railgun update | 2 | ranged draws | claim 18 |
| Normalised no-motion time | 9999.0 | ticks | F13 |
| Natural-smoke yellow factor | 1.5 | dimensionless, on the emission timer *T* | claim 1 |
| Damage-sparks depth limit | 10 | leptons below the surface | claim 1 |
| Deactivated-vehicle spark period | 30 | ticks | claim 1 |
| Refinery smoke unset points | (0, 0, 0) and (128, 128, 0) | leptons | claim 1 |
| Damage-spark displacement of a deactivated vehicle | −100 … 100 | leptons per axis | claim 1 |
| Normalised speed threshold | 10⁻⁶ | leptons per tick | F13: the time test for an axis applies only above this |
| Spark chance scale | 4.656612877414201 × 10⁻¹⁰ (the double nearest 1 / (2³¹ − 2) = 1 / 2147483646) | dimensionless factor on a ranged draw | claim 1, F10 |
| Fire swing rounding addend | 0.49 | dimensionless, added before truncation | F6 |
| Fire swing angle step | 0.125 | radians per table index | F6 |
| Fire sweep facing constants | 16383 and 65534 (not 16384 and 65536) | facing units | F5 |
| Counter modulus | 65536 | dimensionless | the modulus of the particle lifetime and damage counts (claims 3 and 12) |
| Deactivated-vehicle cadence divisor | 10 | leptons per unit of the position term (division truncated toward zero) | claim 1 |
| Deactivated-vehicle cadence weight | 65536 | dimensionless (weight of the *y* term) | claim 1 |
| Railgun end-point samples | 50 (*i* = 0 … 49) | sample points along the beam | F14 |
| Railgun end-point step | 10 956 549 / 536 870 912, the fixed single-precision value nearest 1/49 | dimensionless fraction of the beam per sample | F14 |
| Destruction-smoke depth limit | 10 | leptons below the surface | claim 1 |
| Damage-smoke healing-mark depth | 10 | leptons below the surface (the mark applies below −10) | claim 1 |
| Shared gas system position (hypothesis) | (2688, 2688, 0) | leptons | Not given by S-0613; decided by P1 part b. Its position is observable through the second trigger action of claim 1 (Open questions). |

## State and ordering

**One system turn.** Kind work (claims 5, 10, 15, 17; none for no kind), then S-0613 claim 17's
count-down and leaving play. Particles made in a turn's emission are updated in that turn only
where the kind's update pass comes after its emission: spark and railgun (claims 15, 17). For smoke
and fire, emission comes last, so a new particle's first update is in the next turn.

| Kind | Update order | Move | Removal | Emission place in the turn |
|---|---|---|---|---|
| Smoke | oldest first | live particles move after the removal pass; a particle a move marks is removed in the next turn | removal newest first, before the moves; successors follow their dead particle's removal in that order | after moves and removals |
| Fire | newest first | after the updates (order not observable) | second pass (before the emission) | last |
| Spark | oldest first | inside the update | after updates (order not observable) | first |
| Railgun | oldest first | inside the update | after updates (order not observable) | first (one turn only) |

**System states.**

| State | Event | Result |
|---|---|---|
| Active | kind mark (claims 9, 10, 15, 17), lifetime reaching 0, owner removed (claim 4), a trigger action on its cell (claim 1), a damage-smoke healing mark of claim 1 (including its below-surface test), or a spark-turns field of 0 or less at its first turn (claim 20) | Marked |
| Active, natural smoke | building reaches full cloaking (claim 1) | Removed at once, with its particles |
| Marked | particles remain | keeps taking turns and makes no emission, except that the mark does not stop a spark system's bursts while *K* is above 0 (claim 15, medium); its particles keep updating, moving and ending, and a smoke system's dead particles still split (claim 8, medium for a marked system) |
| Marked | holds no particle after the count-down | leaves play (S-0613 claim 17) |
| Active or marked, no kind, holds a particle | system turn | particle unchanged; system cannot leave while it holds the particle (claim 2). Other events can still mark the system. |

**Draw order in one tick.** Systems take turns in the update order (S-0203 claims 2 and 3), and
each system's draws are those of its claims in the order above. Draw order is observable here: within a smoke turn the
update draws (claim 6) come first, then the successor draws (claim 8, newest dead particle first),
then the emission's two raw values and the particle's creation draws. Within a spark turn the burst
decision and count come first, then the three unused raw values and each spark's creation and
velocity draws, then the light draw, then one draw per live spark, oldest first.

## Edge cases and quirks

- **Fire kills credit nobody.** A kill by a fire particle has no attacking house, though the
  system may have one. This is the original's behaviour and gives no house kill credit or
  experience. `quirks.fire_particle_no_house` (default: on, original). Fixed: the hit's attacking
  house is the system's owner house, as for gas.
- **Smoke ignores its particle cap.** A smoke system emits however many particles it holds,
  whatever the system type's particle-cap field says (the field is the spark cap, claim 15).
  `quirks.smoke_ignores_particle_cap` (default: on, original). Fixed (design choice): while a smoke
  system holds as many particles as the cap field, an emission still takes its two raw values and
  advances the emission timer, but makes no particle.
- **Spark gravity counted twice.** A spark's vertical step uses gravity twice in a tick, its
  stored vertical velocity once (F11). This sets when sparks land and so how many draws they take.
  `quirks.spark_double_gravity` (default: on, original). Fixed: the step uses the stored vertical
  velocity.
- **Smoke deck stop takes one more update.** A smoke particle stopped at a deck keeps its update
  draws for one more turn (claim 7), and splits into successors one turn later than it would
  otherwise. `quirks.smoke_deck_extra_update` (default: on, original). Fixed (design choice): the
  stopped particle is removed, with its successors if any, in the same pass that stops it, and takes
  no further update draws.
- **Fire countdown not restarted past its last stage.** Claim 12: the countdown is not reset but wraps modulo 65536 (its last row), so a fire
  particle past its last damaging stage never hits again, whatever its lifetime. No switch: there is no hit for a switch to change (claim 12).
- **Spark bounce.** A spark ends at its first contact (claim 16). The original works out a bounce
  that it never shows, which takes no draw and changes nothing in play, so there is no switch.
- **Flame sweep facing constants.** The sweep's facing constants are 16383 and 65534 (F5), where the
  nominal values are 16384 and 65536; each flame's aim differs from the nominal form by less than
  0.02 degrees. `quirks.flame_sweep_facing` (default: on, the original). Fixed: 16384 and 65536.
- **Fire end-stage frame.** A fire particle that stays at its end stage shows a frame of the next
  direction group (claim 25). This is the original's arithmetic, kept, since stock art may rely on
  it. A frame the file does not hold draws nothing in our engine (Open questions).
- **Short colour lists** (claim 29): a departure with an importer warning and no quirk switch, since it affects drawing only.
- **Bad data** follows claim 20.
- **A next-type loop** (claim 8a) is a departure, with no switch: in a smoke system a particle made
  through a next-type link that lies on a loop is removed without splitting and takes no split
  draws, and the importer warns for each loop. The original's system never leaves play and can
  double its particles once per particle lifetime.
- **Very large burst and line counts** (claim 20a) are a departure, with no switch: a burst or a line
  makes at most 4096 particles, with the draws claim 20a lists. The original makes them all, and a
  count beyond the program's room stops it.

## Import mapping

Every value in an *Absent* column below is an engine constant, not a value taken from the stock rules
files. Except for the `DefaultSparkSystem` row, which says otherwise in its own cell, it is the original's own fallback for a missing key. The refinery smoke lifetime
and the two sparking probabilities hold their absent-key defaults from program start, before any
file is read; the spark particle cap, the railgun particles-per-lepton field and the other per-type
fallbacks are the absent-key defaults of the fields named below. Confidence high, except the `DamageSmokeOffset` row, which is medium (claim 1).

Beam sampling (claim 1, F14):

| Key | Feeds | Form | Absent |
|---|---|---|---|
| none (engine constants, not imported) | beam sample count and fraction δ | 50 samples, indices 0 … 49; fixed single-precision fraction 10 956 549 / 536 870 912 | always these constants; neither a weapon nor a particle-system field changes them |

The game-wide railgun hit distance is imported by S-0306 claim 27's Import mapping; it does not
set the sample spacing. Rounding follows S-0205 claim 5, not an INI field.

Rules file, particle system sections:

| Key | Feeds | Form | Absent |
|---|---|---|---|
| `BehavesLike` | behaviour kind | word: `Smoke`, `Gas`, `Fire`, `Spark`, `Railgun`, letter case ignored | no kind; also when the word is unknown. Absent in a read section resets the kind to none. |
| `HoldsWhat` | held type | particle type name; `<none>` gives no held type. A railgun-kind type with no held type and a positive particles-per-lepton field is bad data: importer warning, claim 20 | a type with an empty name (claim 21) |
| `Lifetime` | system lifetime (S-0613) | integer, ticks | −1 |
| `SpawnFrames` | spawn interval *T*₀ (smoke), emission interval (fire) | integer, ticks | 1 |
| `Slowdown` | smoke emission timer growth | single-precision number | 0.0 |
| `SpawnCutoff` | smoke spawn cut-off | single-precision number | 0.0 |
| `SpawnTranslucencyCutoff` | smoke translucency cut-off τ | single-precision number | 0.0 |
| `SpawnRadius` | smoke spawn radius σ | integer, leptons | 0 |
| `ParticleCap` | spark particle cap κ; also the divisor of the one-frame light's size (claim 31) | integer | 50 |
| `SparkSpawnFrames` | spark counter start *K* | integer, turns | 0. A value of 0 or less, including this absent value on a spark-kind type, is bad data: importer warning, claim 20 |
| `SpawnSparkPercentage` | spark chance χ | double, as a fraction (1 = always) | 0.0 |
| `SpawnDirection` | spark spawn direction *s* | three single-precision numbers | 0, 0, 0 |
| `ParticlesPerCoord` | railgun particles per lepton *p* | double | 0.1 |
| `SpiralDeltaPerCoord`, `SpiralRadius`, `PositionPerturbationCoefficient`, `MovementPerturbationCoefficient`, `VelocityPerturbationCoefficient` | railgun line shape (drawn only) | double | 0.025, 25.0, 0, 0, 0 |
| `Laser`, `LaserColor`, `LightSize`, `OneFrameLight` | railgun laser, its inner colour, the light size λ and the one-frame-light flag (claim 31; drawn only) | boolean, colour, integer, boolean | no, 0,0,0, 0, no |

The `<none>` reading of the held-type key (no held type, unlike an absent key) is S-0004
claim 2's for this field. It is observable apart from the stop of claim 20: a system with no
held type gets no first particle at a detonation (S-0613 claim 4). Backing: `measurable` (P31
part d); confidence high.

Rules file, particle sections:

| Key | Feeds | Form | Absent |
|---|---|---|---|
| `BehavesLike` | particle behaviour kind | word as above | no kind; absent in a read section resets it |
| `Velocity` | speed *S* at creation | single-precision number, leptons per tick | 0.0 |
| `Deacc` | slowing field | single-precision number | 0.0 |
| `MaxEC` | lifetime field (S-0613 F9) | integer, ticks | 1 |
| `MaxDC` | damage interval | integer, ticks | 0 |
| `Damage` | damage amount (fire) | integer | 0 |
| `Warhead` | warhead of fire hits | warhead name | none |
| `StartStateAI`, `EndStateAI` | start stage, end stage; the end stage is also the fire frame-group size *E* (claim 25) | 8-bit signed integers | 0, 0 |
| `StateAIAdvance` | stage-advance field *A* | 8-bit signed integer, ticks | 4 |
| `FinalDamageState` | last damaging stage δ_F | 8-bit signed integer | the end stage's value as read |
| `DeleteOnStateLimit` | delete-at-end flag | boolean | no |
| `Normalized` | normalised flag (claim 19) | boolean | no |
| `NextParticle` | next particle type (smoke successors); a type that leads back to itself makes a next-type loop (claim 8a), for which the importer warns | particle type name; `<none>` gives none | unchanged (none for a new type) |
| `Radius` | radius field ρ (F3) | integer, leptons | 0 |
| `WindEffect` | wind effect *W* (F2) | integer | 0 |
| `XVelocity`, `YVelocity` | spark *X*, *Y* (F10) | integers, leptons per tick | 1, 1 |
| `MinZVelocity`, `ZVelocityRange` | spark *Z*ₘ, *Z*ᵣ (F10) | integers, leptons per tick | 0, 1 |
| `ColorList`, `StartColor1`, `StartColor2`, `ColorSpeed` | colour list, start colours *s*₁ and *s*₂, colour speed *v* of a pixel-kind particle (claim 29, F16); the creation colour draw of S-0202 claim 15 depends on them; no effect on sprite kinds | colour list of 8-bit RGB triples (S-0202 Import mapping), colours, double | empty, black, black, 0.0. A spark or railgun type with fewer than two entries: importer warning (claim 29) |
| `Translucency` | starting translucency (claim 26) | integer, kept modulo 256 | 0 |
| none (engine constants, not imported) | depth profile and bias of a particle (claims 27, 28): upright with −15 − *L*(*z*) for a sprite kind, a one-row flat test with −50 − *L*(*z*) for a pixel kind | — | always these constants; the behaviour kind chooses sprite or pixel kind, and no other particle or system key changes them |
| `Translucent25State`, `Translucent50State` | fire stage fields (claim 26); no effect on other kinds | 8-bit unsigned stages | 255, 255 (never reached) |
| `StartFrame` | read and kept, but no effect on drawing or play (claim 25); the type's loop-frame field is likewise read and unused | integer | 0 |

Rules file, other sections (claim 1's creators):

| Key | Feeds | Form | Absent |
|---|---|---|---|
| weapon `AttachedParticleSystem` | the effect weapon's emitter type | particle system name | none (an effect weapon without one: the original fails; our engine makes no emitter, with an importer warning) |
| object type `NaturalParticleSystem`, `NaturalParticleLocation` | natural smoke and its displacement | name; three integers, leptons | none; 0, 0, 0 |
| building type `RefinerySmokeParticleSystem`, `RefinerySmokeOffsetOne` … `RefinerySmokeOffsetFour` | refinery smoke and its points | name; three integers each, leptons | none; unset |
| `[General]` `RefinerySmokeFrames` | refinery smoke system lifetime | integer, ticks | 25 |
| infantry type `Cyborg` | damage-sparks flag (the same key as the cyborg flag of S-0312 claim 2; no separate key) | boolean | no |
| object type `DamageSmokeOffset` | damage-smoke displacement (claim 1: damage smoke position, damage sparks) | three integers, leptons | 0, 0, 0 (medium) |
| object type `DamSmkOffScrnRel` | screen-relative flag of that displacement (claim 1; Open questions) | boolean | no |
| `[General]` `ConditionRedSparkingProbability`, `ConditionYellowSparkingProbability` | game-wide sparking probabilities of claim 1's damage sparks | decimal, held as a double (no `%` form is claimed) | red 0.02, yellow 0.01 |
| `[CombatDamage]` `DefaultSparkSystem` | game-wide spark type (electric bolt, deactivated vehicle; also the overload sparks of S-0601 claim 16 and the drone parasite's spark effect of S-0614 claim 9) | name | none. The original's result for an absent key is not established (Open questions). Our engine departs here, with no switch: no system is created, nothing is drawn, and the importer warns, as in S-0601 claim 16. |

Rules file, `[AudioVisual]`: `Gravity` (integer, leptons per tick per tick) for F11, absent value
as S-0613 gives it. `[General]`: `WindDirection` (integer 0 … 7) for F2, absent value −1 (claim 7).
Object type sections, `DamageParticleSystems` (the list of damage-smoke and damage-sparks candidates
that claim 1 reads): owned by S-0302 (Import mapping, claim 13), empty when absent. S-0312 owns
`DestroyParticleSystems` and `DestroySmokeOffset` (the destruction list and the destruction-smoke
displacement of claim 1). S-0301 owns the weapon flags `UseFireParticles`, `UseSparkParticles` and
`IsRailgun` (claim 1's effect weapons).

## Measurement scenarios

All scenarios: stock data unless a test rules file is named, no engine add-on and no mods (the named test files of a scenario excepted),
a tiny flat test map unless the part names terrain, game speed index 2, one human-controlled test house and one enemy bystander computer player kept outside the observed area (the test objects and their cells), with the glossary's bystander void rule. The recorder logs tick-indexed positions, particle counts, health and effect events where a part needs them, as well as random state. Scripted input and frame capture may be used. The random-state recorder logs raw values taken per tick, with the taking object, where draws are counted. No scenario fixes the seed: each game start takes it from the host clock (S-0202 claim 8), and the recorder logs it with the draws. Each part of a scenario is one run unless the part states otherwise; P5, P9 and P11, whose results depend on the draws, are three runs per part, and each expected result must hold in every run.

- **P1. Creators.** One run per row of claim 1. Setup: the creator's situation (a warhead, a barrel, a damaged
  vehicle, a refinery unloading, an overload, an electric-bolt shot, a drone, a
  trigger, a building's destruction, a building placed in play, a building's uncloaking, an object
  with the damage-sparks flag, an effect weapon's shot, a deactivated vehicle, a voxel-model
  animation with an attached system, and the game start).
  Observable: the number of particle systems in play per tick, their kind, owner, and the tick in
  which each leaves play, the ticks at which deactivated-vehicle systems are made, the points at which refinery smoke is made, the position of each system that a trigger action makes, and the two displacement draws of each deactivated-vehicle system. Expected: claim 1's columns and claim 22's end. Falsified by a system
  that leaves play at another tick, by an owner object or first particle other than listed, by a
  deactivated-vehicle system whose spark ticks are not 30 apart or whose displacement lies outside −100 … 100 (the phase is not compared with the original's, since claim 1's departure note allows it), by refinery smoke
  made at an unset point or missed at a set one, or by the system held in the fire place let go
  at a tick other than claim 1 gives (a new out-of-reach target, a move order and a death, one run
  each).
  Part b: a trigger action that marks the systems of one cell (a smoke, a fire, a spark and a gas system
  in it, each holding particles; named test rules give the smoke system's held type a next
  particle type, as in P7, and give the spark system's type a spark-turns field large enough that
  its *K* is still above 0 at the mark; a second run with the waypoint in the cell of the shared gas system), and a natural-smoke building that reaches full cloaking. Expected:
  each marked system makes no emission, except that the mark does not stop the spark system's
  bursts while its *K* is above 0 (claim 15), the marked smoke system's dead particles still
  split into successors after the mark (claim 8), and each leaves in the first turn in which it holds
  none (the observable adds, for this part, each successor made and each spark burst, per tick); the cloaked building's system is removed at once, with its particles. Falsified by a marked smoke, fire or railgun system that emits, by a marked smoke system that makes no successor after the mark while dead particles of a type with a next type occur in it, by a marked spark system that takes no burst draws after the mark in a turn in which claim 15's table gives them while its *K* is above 0, by one that leaves at another tick, by a removal that keeps its particles, or (second run) by a shared gas system that is not marked by an action on a waypoint in cell (10, 10), or that is marked by one in another cell. Part c: two natural-smoke building types that differ only in the natural-smoke displacement, one (0, 0, 0) and one not, each cloaked fully and then uncloaked. Expected: after the uncloak the second building has a new natural-smoke system and the first has none. Falsified by a new system for the first type or none for the second (if the first also gets one, claim 1's condition and S-0516 claim 18's displacement condition are revised).
- **P2. Kinds that change play.** Two runs that differ only in a smoke system's slow-down field.
  Observable: the recorder. Expected: draw counts differ from the first tick in which the
  emission count differs, and later draws of other objects shift. No unit's health differs except
  through those draws. Falsified by a smoke particle changing an object's health. Part b: the same
  two-run comparison for a fire, a spark and a railgun system, each varied in its own timing field
  (the fire emission interval, the spark counter start, the railgun particles-per-lepton field).
  Expected: each kind's draw count differs between the two runs. Falsified by equal draw counts. Part c: named test rules create no-kind systems, one empty through a trigger and one holding a short-lived smoke particle through a detonation, both with unlimited system lifetime. After 100 ticks, mark each through the cell trigger action and observe another 100 ticks. Expected: neither system updates its particles or takes turn draws; the empty system leaves on its first marked turn, while the populated one keeps its particle without motion, expiry or update draws even after marking. Falsified by a turn draw, a particle update or expiry, or either system leaving at another time.
- **P3. Lifetime decrement.** A test spark type with a large vertical velocity minimum (so it does
  not land) and lifetime field 3. Observable: the tick at which the spark's draws stop. Expected:
  it takes 3 + *u* draws (*u* the lifetime variation), then none. Falsified by one more or fewer.
  Part b: a test rules file whose smoke system holds a fire particle type that has damage. Expected:
  the fire particles hit (claim 14) and move by claim 13, in the smoke turn's order. Falsified by a missing due hit, missing move, or an effect in another order.
  Part c (successors across kinds, claim 3): test rules with (i) a gas-kind system holding a smoke
  type with lifetime 3 and a next type, (ii) a smoke-kind system holding a gas type with lifetime 3
  and a next type, (iii) a fire-kind system holding a smoke type, and (iv) a spark-kind system holding
  a gas type, both with a next type. Observable: the particle count and positions after the first
  particle dies, and the raw values taken in that turn. Expected: (i) one successor at the displacement of S-0613 claim 7, with its creation draws (S-0202 claim 15) and no draw of this spec's claim 8; (ii) two successors mirrored about the dead particle's position, with this spec's claim 8 draws; (iii) and (iv) no successor and no successor draw, the turn's other draws as claims 3, 10 and 15 give. Falsified by any other count or successor draw.
- **P4. Owner removal.** A building with a damage smoke system (S-0312 claim 19 f or S-0302
  claim 13), sold or destroyed while the system holds particles. Observable: the tick in which the
  system leaves play. Expected: it stops emitting at once and leaves in the turn its last
  particle is removed. Falsified by an emission after the sale or destruction, or by leaving play in a turn other than the one that removes the last particle. Part b: the owner enters a transport; the system's emission is recorded over the next 100 ticks
  (observable: its emission ticks and whether it leaves play). This part decides the open question: the hypothesis is that the system keeps emitting and is
  not marked (claim 4, confidence medium). Falsified by emission stopping or by the system leaving play
  after the owner enters; if so, claim 4 and the open question are revised to say that leaving the map
  marks the system.
- **P5. Smoke turn.** A test rules file giving a smoke system type spawn interval 10, slow-down
  0.05 and cut-off 12.0, made by a warhead on a flat map. Observable: emission ticks, draw counts
  per tick, the tick the system is marked (emission stops) and leaves play. Expected: *T* grows
  every turn, so after *k* turns it is 10 + 0.05*k* accumulated in single precision. A turn emits
  when its tick number is a multiple of ⌊*T*⌋ for that turn: 10 for about the first 20 turns, then
  11. The system is marked in the first turn with *T* > 12.0, about 41 turns after creation. Each
  live particle takes one or three raw values on odd ticks only, and each emission two. Falsified
  by draws on even ticks, by a cap on particles, by emissions after the mark, by a horizontal velocity outside −5 … 5, or by a particle speed that changes at or below 3.0. Part b: the same
  type with a particle-cap field smaller than the number of live particles. Expected: emissions
  continue and the particle count exceeds the cap (`quirks.smoke_ignores_particle_cap`). Falsified
  by a count that stops at the cap. Part c: record each smoke particle's creation, stage, lifetime, speed, velocity and draws until removal, using named test rules with a finite lifetime, positive slowing, a next type, radius 40 and a delete-at-end flag. Predict every update from claims 3 and 6 and the logged raw values, including the stage cadence and the slowing threshold. Expected: updates oldest first, successor draws newest dead particle first after all update draws, then emission draws; emissions follow claim 9. Falsified by a stage, speed, velocity, removal tick, draw count or draw order that differs from that prediction.
- **P6. Smoke at a bridge.** A smoke system under a bridge deck. Observable: the height at which
  particles stop and the extra update draws. Expected: the first move whose proposed height reaches at least 156 leptons above *H* while starting below the deck is refused; they stay at their pre-move position, take
  one more update's draws and, if their type names a next type, split one turn after stopping
  (`quirks.smoke_deck_extra_update`). Falsified by particles passing through the deck, or by no
  further update draws after the stop. Part b (wind): a test smoke type with wind effect 3 on a
  flat map, one run per rules wind direction 0 … 7, with the particle's drift at rest. Observable:
  the particle's horizontal displacement per tick, summed over 10 ticks of a particle whose drift
  draws are known from the recorder. Expected: the wind part is Δ(ω) × 3 per tick from the Constants
  table, so at direction 3 it is 6 in x, not 3. Falsified by a wind part of 3 in x at direction 3.
- **P7. Smoke successors.** A test rules file: a smoke type with a next type and radius 40 (so
  *d* = 5). Observable: successor positions relative to the dead particle and the draw order.
  Expected: displacements in 6 … 9 or −9 … −5 per axis, mirrored for the second successor, and never +5. Record each successor's speed, velocity, flight direction, translucency and creation draws too. Its speed equals the parent's, its velocity and direction are its own creation values, and its translucency equals the parent's plus the increment predicted from its post-creation raw value (claim 8). Falsified by a displacement outside those ranges, a count other than two, inherited velocity or direction, a different speed or translucency, or a draw order other than claim 8's.
- **P8. Fire turn.** A test rules file giving a vehicle a weapon whose particle system is the
  fire kind, firing at a building while the vehicle turns on the spot. Observable: emission ticks before and
  after the turn starts, the system's position, the tick it is marked after the vehicle is
  killed. Expected: emissions every spawn interval, plus every third tick only on the ticks
  while the vehicle's body is turning; once it stops turning the extra emissions stop. Falsified by
  extra emissions that continue after the turn ends. Part b: a warhead that names a fire system: one particle, no emission, the system
  leaves when that particle is gone. Falsified by an emission, or by the system staying in play after that particle is gone. Part c: the direction of each particle's first-tick displacement (it points along the particle's flight direction, toward the aim point of F6) and the list place of each particle that a sweep makes, over 500 ticks, against a fixed target. Expected: the direction is that of the F6 aim point from the creation position, to within the rounding of the whole-lepton displacement, and the list places follow F7. During sweeps, record the system position and compare it with the first weapon's fire point for the owner's current burst counter, including both parities with named test burst data. Falsified by a direction or list place that differs, or by a sweep position using the shot's saved burst parity instead of the current one.
- **P9. Fire particle.** The fire system of P8 against a row of targets on level and rising
  ground. Observable: the tick and cell where each particle ends, and its hits. Expected: particles
  end at the first higher terrain height; hits only while the stage is at or below the last
  damaging stage; one raw value per update while *S* > 0. Falsified by a particle that passes
  rising ground, by a hit with the stage past the last damaging stage, or by an update with *S* > 0
  and no draw. Part b: a second test fire type with the normalised flag, fired at a target at a
  known distance. Expected: the stage reaches the last damaging stage at about the tick the particle
  reaches its target point (claim 19, F13). Falsified by a stage that differs by more than one
  step at that tick.
- **P10. Fire damage.** As P9 with infantry of two houses in the flame. Observable: health loss per
  hit, the house credited with kills, and whether the firing vehicle is hurt. Expected: damage of
  the type's amount at distance ⌊*R* / 10⌋; no house credited; the owner object never hit. Falsified by a hit at another distance, by a kill credited to a house, or by a hit on the owner object.
  Part b: two objects in one cell, the first killed by a hit. Expected: the second is hit in that
  same update. Falsified by a second object whose health is unchanged in that update.
  Part c: a bridge cell with an infantry object on the deck and another on the ground under it, and a flame whose particles pass through the cell once at least 416 leptons above the cell's ground level (height level × 104) and once below that; run it on a flat cell and on a sloped bridge-end cell, to tell the cell's height level from *H* at the particle's position. Expected: while the particle is at least 416 above, only the deck object is hit; below that, only the ground object. Falsified by a hit on the other layer, or on both. Falsified also, on the sloped cell, if the switch follows *H* at the particle's position instead of the cell's height level. If the threshold lies elsewhere, the run records the particle height at which the hit switches layers, and claim 14's rule is revised.
- **P11. Sparks.** The repair vehicle's welding sparks, and a mind-control overload (S-0601 claim 16)
  on a flat map, then on a bridge deck and next to a wall. Observable: per tick, the number of
  sparks and the raw values taken by the system. Expected: the burst decision draw on turns with
  *K* > 1, *c* sparks with three plus creation plus three draws, one light draw per turn while
  *K* > 0, and one draw per live spark; each spark ends at its first contact. Falsified by a spark surviving a ground contact, or by a burst-decision, count or light draw on a turn with *K* = 0 (a live spark's colour draw is still taken on such a turn). Part b: a spark type with a chance
  of 0 and a spark counter of 1. Expected: its first turn bursts with no decision draw, whatever the
  chance (claim 15). Falsified by a decision draw on that turn, or by no burst.
- **P12. Railgun.** A test rules file giving a weapon a railgun system, fired across a known
  distance *L*. Observable: the particle count and the draws in the firing tick and later ticks.
  Expected: trunc(*L* × *p*) particles; seven ranged draws per particle besides creation draws; then two
  per live particle per update; the system leaves play when its last particle ends. Falsified by
  another particle count or draw count per particle.
- **P13. Absent held type.** A test rules file: a smoke system type section with no held-type key,
  made by a warhead. Give it spawn interval 10 ticks and a cut-off below that interval, so that it is marked in its first
  turn. That first turn may still emit once, when the tick number is a multiple of ⌊*T*₀⌋ (claim 9),
  so choose a detonation tick that is not such a multiple; otherwise the one first particle
  becomes two. Observable: the draws in the detonation tick and the next, and
  the number of particles. Expected: one first particle with the creation draws of S-0202 claim 15
  for a lifetime field of 1 (one lifetime raw value), which ends after its first update; then the
  system leaves play. Falsified by no particle, or by more than one.
- **P14. Detonation railgun.** A test warhead naming a railgun system. Observable: the number of
  particle systems in play and the draws over 1000 ticks. Expected: the system leaves play in the
  first turn after its first particle ends, with no line of particles and no draw of its own.
  Falsified by the system staying in play or by a line of particles. (Its laser beam, if the type
  has the laser flag, has zero length and is not judged here: claim 31.)
- **P15. Natural smoke turning yellow.** A building whose type names a natural particle system, with
  a test rules file giving that system an emission interval of 8, a slow-down of 0 and a spawn
  cut-off of 100 (so that claim 9 does not mark it). Emission follows multiples of ⌊*T*⌋ (claim 9). Part a: a hit that
  takes the building from full health to just below half. Part b: repair above half, then a second
  hit below half. Part c: a new building, one hit from full health straight below the red ratio.
  Observable: the emission ticks before and after each hit. Expected: part a, emissions on
  multiples of 8 before the hit and of 12 after it; part b, still 12 after the repair, and 18 after
  the second *now yellow* hit; part c, still 8 (the hit reports *now red*).
  Falsified by another factor, by repair restoring the first interval, by no change at the second
  hit, or by a change in part c. A fourth run with a cut-off of 15 expects the system to be marked
  (emission stops) in the turn after the second hit, when *T* = 18 passes the cut-off. Three runs
  each.
- **P16. Damage sparks and damage-smoke position.** Part a: a cyborg-flag infantry type with a test
  rules file giving a damage list with two spark-kind entries and one smoke-kind entry, a
  displacement of (32, 48, 0) and sparking probabilities of 1.0 (yellow) and 0.0 (red), standing on
  flat ground at a health ratio below the yellow ratio. Part a also runs once with the health ratio exactly at the yellow ratio: the recorder must show no draw for the system and no system made, because claim 1's rule is strictly below. Observable: the recorder's draws per tick,
  and the position and type of the system made. Expected: one draw R(0, 2147483646) per tick while
  no emitter is held, a second draw R(0, 1) when the system is made, which is of the spark kind
  picked from the start of the list, at the infantry's position plus (32, 48, 0). Swap the
  probabilities (red 1.0, yellow 0.0) and lower the ratio below the red ratio: the system is made.
  Falsified by a missing draw, a smoke-kind system, another position, the wrong probability, or a system made (or a draw taken) at exactly the yellow ratio. A
  type with a list holding no spark-kind entry takes no draw. Part b: a vehicle with a displacement of (16, 0, 40)
  damaged to yellow with a list of one smoke entry. Expected: the damage-smoke system at the
  vehicle's position plus (16, 0, 40). Falsified by another position. Part c: as part a with both probability keys absent from the rules file, over 5000 turns in 5 runs
  (three runs for part a), with the spark counter start of the test spark type set to 1. A made
  spark system stays in the spark emitter place until it leaves play, and in the original a counter
  of 0 never lets it leave (claim 20), so with the absent counter the place would stay held after
  the first success and the rate could not be measured. With the counter at 1 the place is free
  again once the system's sparks have ended. One set of runs at a ratio below the red ratio and one between the red
  and yellow ratios. Observable: the turns on which the spark place holds nothing, and how many
  of them make a system. Expected: about 0.02 of those turns make a system below the red ratio (red
  default), and about 0.01 between the red and yellow ratios. Falsified by a rate outside twice the
  standard error of those figures.
- **P17. Sparks and a wall.** Sparks over a wall cell beside a building. Observable: whether each spark ends at its contact. Expected: sparks end at the wall and at the building. Falsified by a spark that passes either.
- **P18. Normalised stage with no damaging stage.** As P9 part b, with a last damaging stage of
  −1. Run once with the target at distance 0 (so *t* = 0) and once at a known distance. Observable:
  the particle's stage in each tick, against a second run whose type has no normalised flag and a stage-advance
  field of 0. Expected: the stage sequence equals that of the second run, in both cases (claim 19:
  the particle's own advance value is 0). Falsified by a stage sequence that differs from it.
- **P19. Healing mark on damage smoke.** A building with a damage-smoke type whose particles last
  longer than a few ticks, damaged to red so that it makes damage smoke, then healed above the
  yellow ratio by hits from a test healing weapon, then damaged below it again before the system's particles have gone. Observable:
  the number of damage-smoke systems in play per tick, their emission, and the draws in the recorder.
  Expected: at the healing hit the system is marked: it makes no further particle, its particles
  carry on, and it leaves in the first turn in which it holds none. The second damage makes no
  system while the first is in play; once it has left, the next damaging hit (to yellow or red)
  makes a new one. Falsified by the system and its particles vanishing at the healing hit, by
  emission after it, by a second system made while the first is still in play, or by no new system
  after it has left play. Three runs.

- **P20. Refinery smoke events.** A refinery type with one set smoke point and a test refinery smoke
  type with a long emission interval, so each system is easy to count. Runs: a harvester carrying
  one resource, one carrying two different resources, one docking empty, a slave delivery to a
  deployed slave miner whose type names a refinery smoke type (one smoke point set), and a slave
  delivery to an undeployed slave miner. Observable: the number of refinery smoke systems made per
  unloading, the tick each is made and the tick of each payment (credits delta). Expected: 2, 3 and
  1 systems for the three harvesters, each made in the same tick as and before its payment row, the
  last one in the tick in which the harvester enters its finishing phase (not the later undocking tick); 1 system per slave delivery to the deployed
  miner, at its smoke point, after the delivery's payments; none for the undeployed miner.
  Falsified by one system per whole delivery, by no system for the empty-load row, or by any system
  at the undeployed miner.
- **P21. Destruction smoke.** A test rules file giving a building type a destruction list of one
  smoke, one spark and one fire system, in that order and in other orders. Part a: a building of a
  type that does not explode and is not being sold, destroyed 30 times. Part b: the same with a type that explodes.
  Part c: a building that holds a damage-smoke system and a natural-smoke system, destroyed.
  Part d: the non-exploding type destroyed while being sold, before the sale removes it.
  Repeat with an electromagnetic pulse postponing its removal under S-0312 claim 21.
  Observable: the kind of each destruction system made, the draws in the recorder, and the ticks in
  which each emits and leaves play. Expected: only the smoke entry is ever made, with no pick draw
  (one eligible entry gives the equal-bounds draw R(0, 0), which takes no raw value, S-0202 claim 4,
  a second test type with two smoke entries shows one ranged draw R(0, 1)); in part a it never emits and leaves play in its first turn; in part b it may emit
  until the building's next eligible removal turn and is then marked, its particles carrying on.
  In part c it is made all the same. Part d permits the same pre-removal emission as part b,
  including during the pulse's postponement, and marks the system only at removal. Record the
  building's removal tick as well as each system's emission and marking ticks. Falsified by a
  spark or fire system made, by a missing smoke system in part c, by immediate owner-removal
  marking in part d while the building still awaits removal, or by emission after removal.
- **P22. Effect weapons with several flags, and the railgun end point.** A test rules file. Part a:
  a weapon with the fire, spark and railgun flags and one named system type, firing at a vehicle. Observable:
  the systems made at the shot, their emitter places and order, and the ticks at which the weapon fires again.
  Expected: three emitters of that same system type at the shot, assigned in the order fire place, spark place, railgun place; the flags do not change the type's behaviour kind. The weapon fires again only
  when all three places are empty. Falsified by one emitter only, or by a shot while any place is
  held. Part b: a railgun weapon fired at a moving vehicle, and at a standing one across flat ground
  and across a ridge higher than the beam. Enable the system's drawn laser in the named test rules. Observable: the laser end point from frame capture, checked against the recorded system target point, and the target's position at the shot and in the line's tick. The last helix particle is not assumed to lie at the beam end point. Expected: on flat ground the line runs to the
  target's position at the shot, not its later one; across the ridge it ends at F14's first sample
  below the terrain, in the first cell of the ridge that a sample enters. Falsified by a line to the
  target's later position, or by an end point that is not one of F14's samples. Part c (claim 20): a
  spark-kind emitter type with a spark-turns field of 0 and unlimited system lifetime on a weapon, in the original. Keep its cell free of system-marking trigger actions. Record shots, system persistence and emitter-place occupancy. Expected: after
  the first shot the weapon never fires again, through a move order, a new target and 1000 ticks, and
  fires again only after the firer is removed and a new one built. This records the original's
  behaviour that claim 20 fixes; our engine's expected result is the reverse (the place is free in the
  turn after the shot). Falsified by the original freeing the place or firing again while the same system survives, or by a burst or system-turn draw. Repeat with a negative spark-turns value. Separate controls apply a finite lifetime and a cell-marking trigger: each must free the empty system on its first marked turn, falsified by continued refusal after that removal.
  Part d (Q-1405 question 1): use a long flat lane and named test rules giving the railgun enough
  range, a positive hit distance that includes all lane occupants, and nonlethal damage. Arrange
  a shot with recorded fire point *P* = (2560, 2688, 60) and target position
  *Q* = (15104, 2688, 0), in leptons. Use two non-target infantry, one listed only in cell
  (12, 10), at its centre, and one listed only in the target cell (59, 10), at its centre.
  Verify those positions and cell listings with the read-only recorder before the shot; no
  bystander may occupy another sampled cell. Record health changes, hit order and the emitter's
  target point. Expected in the original mode: sample 3 has x = 3327 leptons and visits cell
  (12, 10), so the first bystander is hit; sample 49 has x = 15103 leptons and remains in cell
  (58, 10), so the target-cell bystander is not hit. The shot target is nevertheless hit by
  S-0306 claim 27's fallback, and the emitter's target point is the exact *Q*, not sample 49.
  Rounding the sample parameters to nearest would instead skip cell (12, 10) at sample 3.
  Pinning the last parameter to 1.0 would wrongly visit cell (59, 10). Either corresponding
  bystander result, or an emitter target point short of *Q* on this flat lane, falsifies F14.
  One shot per fresh run, three runs, with the director off; use the shared speed, houses and
  recorder setup above. This is a proposed oracle test, not a recorded measurement.
- **P23. Place of the spark steps in the turn.** The random-state recorder, with the taking object,
  on (a) a cyborg-flag infantry below the yellow ratio with self-healing, moving and firing, standing
  in a radiation field, and (b) a vehicle disabled by selling its house's last control building
  (S-0405 claim 13), moving and self-healing, then hit by an electromagnetic pulse. Observable: the
  order of each object's draws within its turn. Expected: (a) the damage-sparks decision draw comes
  after the draws of its behaviour processing and self-healing, and before its radiation hit and its
  firing draws; (b) the deactivated-vehicle displacement draws, on a spark tick, come before every
  other draw of the vehicle's turn, and continue under the pulse. Falsified by any other order, or by
  no deactivated sparks during the pulse. Part c (healing mark): a vehicle with self-healing, damaged
  below the yellow ratio so that it makes damage smoke, then left to heal above it with no hit.
  Expected: the damage-smoke system is marked in the turn its health ratio passes the yellow ratio,
  stops emitting and leaves play once empty. Repeat with a building and infantry using their own self-healing, then with a non-organic vehicle receiving only house-wide healing. Controls use infantry and an organic vehicle receiving only house-wide healing, with no later hit or own self-healing: their damage smoke keeps emitting across the threshold. Use named test rules with no emission cut-off or finite system lifetime during the observation. Falsified by emission after the marking turn in a positive case, or by threshold marking in either control. Three runs per variant.

- **Frame-rate check (P24, P25, P27).** The recorder confirms that the drawn-frame rate stays above S-0613 claim 16's cutoff throughout; a run in which it does not is inconclusive.
- **P24. Which particles are drawn (claim 23).** Named test rules: a smoke system, a fire system
  with a zero damage amount, a spark emitter and a railgun emitter, each fired at flat open ground.
  Run once at each of the three detail settings, at 800×600 so that the drawn-frame rate stays
  above the cutoff, with tick-labelled lossless capture. Observable: which particles appear in the
  capture. Expected: at the lowest setting no smoke and no spark particle is drawn, while the fire
  sprites and railgun pixels are; at the other two settings all four are. Falsified by a smoke or
  spark particle drawn at the lowest setting, or a fire or railgun particle missing there.
- **P25. Palette and brightness (claim 24).** A test map with a dark ambient light and one cell
  lit by a coloured building light, plus a raised plateau. A smoke particle type with a known sprite
  (named test rules, a slow rise, no wind) is released on flat dark ground, on the coloured cell and
  on the plateau, well away from shroud edges and glow images. Observable: the pixels of one image
  frame in each place. Expected: identical pixels in all three places, equal to the frame drawn with
  the anim palette at brightness 1000 and no tint.
  Falsified by pixels that follow the cell's light level, the coloured light or the height, or match
  the unit, house-colour or tile palette. The unscaled ground level of the cell
  predicts a darker row on the dark ground; this run separates the two.
- **P26. Image frames (claim 25).** Part a: a smoke type with a nonzero start-frame field and a
  loop-frame field other than 1 (named test rules), and a control with both at their absent values.
  Expected: the same frame sequence in both, frame = stage. Part b: a fire weapon fired by a unit in
  eight screen directions in turn, its particle type's end-stage field *E* > 1 and a sprite whose
  frames are numbered visibly. Expected: up and down use frames 0 … *E* − 1, up-right and
  down-left *E* … 2*E* − 1, left and right 2*E* … 3*E* − 1, down-right and up-left 3*E* … 4*E* − 1.
  Falsified by mirrored frames, a start-frame shift, or another grouping.
- **P27. Translucency (claim 26).** Highest detail setting. Part a: a smoke type with translucency
  0 and a finite chain of seven distinct named types (generations 0 to 6), each naming the next and the last naming none, radius 40 and a short lifetime, so that successors chain,
  record each generation's translucency value (drawn shade) and the raw values of claim 8. Expected:
  each successor draws at the parent's value plus 25 when *r* rem 6 ≠ 0, drawn 25, 50, 75, 75, 75,
  and opaque from 150. Part b: a fire type with translucency 0, end stage 6, translucent-25 stage 2
  and translucent-50 stage 4. Expected: opaque at stages 0 and 1, 25 % from stage 2, 50 % from
  stage 4. Part c: the same with both fields 3, and with the 25-field equal to the start stage.
  Expected: 50 % from stage 3, and no change at the start stage. Part d: a smoke type with
  translucency 30. Expected: opaque. Repeat part b at the middle setting: opaque throughout.
- **P28. Placement and shadow (claim 27).** A no-kind particle (named test rules, frame 0 only, an
  odd-sized sprite canvas) made by a warhead at a known point on flat ground at level 0 and again
  on a raised cell. Observable: the image frame's top-left pixel and the absence of any shadow
  pixels. Expected: placement by S-0102 claim 19 on the projected point with its height lift, and
  no shadow. Part b: the particle behind a building's upright body. Expected: the body covers
  the particle where the depth bias −15 − *L*(*z*) loses. Falsified by a displaced frame, a shadow, or a
  depth order that needs another bias. Part c (Q-1540): on flat, evenly lit ground, a gas weapon
  releases a cloud at the foot of a building on its far side (screen above the body), then a cloud
  just behind a one-level cliff, and a smoke type (named test rules, no-kind and smoke variants)
  whose column rises above the building's roof. Capture lossless frames at the highest and the
  lowest-but-one detail setting. Observable: which pixels of each particle sprite the building
  body and the cliff tiles cover. Expected: the cut follows the upright profile of each sprite's
  own rectangle with bias −15 − *L*(*z*) (S-0102 F2), the same for gas, smoke, fire and no-kind
  particles and unchanged by translucency or detail setting; the part of a smoke sprite that rises
  above the roof's top row stays visible, while a sprite whose bottom row lies behind the body is
  covered wherever the body's stored depth is nearer. Falsified by a cut along a flat row-by-row
  line, by a kind-dependent cut, or by a cut that needs a further constant.
- **P29. Spark and railgun pixels (claims 28, 29).** Named test rules: a spark type with a three-entry
  colour list of distinct pure colours, both start colours black, a colour speed of 0.1, and a
  railgun type with the same list. Fire each on flat explored ground, then with the sparks crossing
  a shroud edge. Observable: per tick, each particle's pixel colour and position (lossless capture)
  with the recorder's colour draws. Expected: one pixel per particle; colour equal to F16 from the
  logged draws, moving from entry 0 to entry 1 and then to entry 2, where it stays; channels scaled
  by the light-alpha at the shroud edge, and no pixel inside full shroud. Part b: start colours set
  (one black, one white). Expected: the first colour is the mix of the creation draw, and entry 0
  never shows. Part c (depth, claim 28): the spark type bursts at the foot of a building on its far
  side and beside a tree, so that its pixels fall behind the body and the tree. Observable: which
  spark pixels show over the body and the tree. Expected: a pixel is hidden exactly where
  *d* − *L*(*z*) − 50 at its own row is not less than the stored depth, with no 3-row stepping.
  Falsified by sparks drawn over the whole body, or hidden by a unit standing in front of them.
  Falsified (parts a and b) by a sprite, a line, translucency, a palette-shifted colour or another colour
  path.
- **P30. Draw order (claim 30).** Two smoke systems made at the same point a few ticks apart, then a
  projectile crossing them. Observable: which drawing covers which where they overlap. Expected:
  each later-made particle covers every earlier one, whatever its system, and the projectile covers
  the particles made before it entered the air layer and is covered by those made after. Falsified
  by grouping by system or by sorting.
- **P31. System lights and the railgun laser (claim 31).** Named test rules: a spark system with a
  light size and no one-frame flag, the same with the one-frame flag, and a railgun type with the
  laser flag, a pure laser colour, a held type and a particles-per-lepton field large enough that
  the line count *n* of F12 is at least 1 at the firing distance. Run at the highest and the middle detail setting. Observable:
  frames with a spotlight, its size against the particle count, and the laser's frames and colour.
  Expected: the lasting spotlight only at the highest setting and only after a first-turn burst; the
  one-frame light at both settings while the system holds particles, sized by claim 31; the laser
  for 10 ticks in the laser colour, fading. Falsified by another count, size or duration.
  Part b (Q-1590 question 2): a railgun system type with the laser flag, a pure laser colour, no
  held type (its held-type key written as `<none>`, Import mapping; an absent key would give the
  empty-named type of claim 21, which is a held type) and a particles-per-lepton field of 0, fired at a point a few cells away. Frame capture
  from the tick of the shot through the 10 ticks after it. Expected: the laser line from the firer's system position to the
  target point, in the laser colour, for 10 ticks, fading, and no particle. Falsified by no line,
  a line lasting other than 10 ticks, or a particle.
  Repeat with a held type and the same field of 0: the same line, no particle. Since *n* = 0 makes
  no particle either way, part d's result says which case the run written `<none>` backs: the
  case without a held type when part d finds no first particle, the beam alone otherwise.
  Part c (Q-1590 question 1): the same type, its held type still written as `<none>`, with a
  particles-per-lepton field large enough that the line count *n* of F12 is at least 1 at that
  distance. The firer gets its fire order as scripted input at a fixed tick (the glossary's
  scripted-input allowance), and each run is set to end 300 ticks after that order, long enough
  for the firer to turn and fire. Observable: whether the game stops with an error before the
  run's end. The tick of the stop and the frames just before it are not judged, since nothing
  cited shows that they survive the stop. Expected: the game stops with an error. Control: the
  same run with the held-type key absent reaches the run's end, and its recorder log, which
  survives, shows that it fired, because the empty-named type of claim 21 is a held type. Part c
  is judged together with part d. With part d finding no first particle for `<none>`, a `<none>`
  run that reaches the run's end falsifies claim 20's stop. With part d finding a first particle, the
  `<none>` run has a held type and part c backs nothing. A stop in the control too makes the
  part inconclusive (a line of the empty-named type might fail for another reason).
  Part d (the `<none>` reading, Import mapping): named test rules give a test warhead, as its
  particle system, the railgun type of part b (laser flag, held-type key written as `<none>`,
  particles-per-lepton field 0, so that its line has no particle and cannot stop the game).
  Fire once at empty ground and record, from the recorder's particle log, the particles
  created in the detonation tick and the system's existence. Controls: the same with the
  held-type key absent, and with a named test particle type. Expected: no first particle in
  the `<none>` run (S-0613 claim 4 for a system with no held type); exactly one, of the
  empty-named type, in the absent-key control, and one of the named type in the other.
  Falsified by a first particle in the `<none>` run while both controls show theirs. A control
  without its particle makes the part inconclusive.
- **P32. Next-type loops (claim 8a; Q-2999).** Named test rules: a loose rules copy that adds an
  invented smoke system type whose held type is an invented smoke particle type *X* with a short
  lifetime, a radius field of at least 8 and *X* itself as its next type, and changes nothing else.
  Part a: a test warhead whose particle system is that type, fired once at empty flat ground, with
  the random-state recorder. Record per tick, for 30 ticks or until the run is stopped from
  outside after 60 seconds, the number of particles of the system and the draws of its turn, and
  whether the system is still in play. Expected on the original: the layer loads unchanged; every
  dead particle is replaced by two in the same turn's removal pass, which update first in the next
  turn; the count doubles once per particle lifetime; the system stays in play after its mark.
  Falsified by a refusal at load, a successor that updates in its own split's turn, or a system
  that leaves play while its particles remain. Part b: two types *X* and *Y* that name each other,
  the same observables. Engine runs: one importer warning per loop; the emitted particle splits
  once, and its two successors are removed at their end without splitting and without split draws.
- **P33. Very large burst and line counts (claim 20a; Q-2999).** Named test rules: a loose rules
  copy with an invented spark system type with a held type whose x and y velocity fields and
  vertical velocity range are not 0 (claim 20), a spark-turns field of 1 (so that its first turn
  bursts with no decision draw, claim 15) and a particle cap that gives counts *c* from 5000 to
  9999, and an invented railgun system type with a held type and a particles-per-lepton field
  that gives a line of about 5000 particles at the firing distance, each on a weapon. Fire each
  once, giving the firer no order and keeping its target in place until the system's first turn,
  with the random-state recorder. Observable: the draws of the burst's or the line's turn.
  Expected on the original: the turn takes the count raw value, the three discarded raw values
  and the draws of all *c* sparks, then the light draw (claim 15), or the draws of all *n*
  particles (claim 17), in those claims' order, and play goes on. Falsified by fewer draws than
  the count requires. Engine runs: the draws of 4096 sparks or particles after the burst's count
  and discarded raw values (for the burst, then its light draw), or for the line, in the same
  order. A run with a much larger count is not proposed: the original's result there depends on
  the program's room and can stop the game.

## Open questions

- **Owner leaving the map** (claim 4): whether an owner that goes into a transport or a building,
  without being removed, marks its systems. The original's mark comes from the general notice that
  an object is gone; whether that notice is also sent for leaving the map is not settled.
- **Wall test** (Terms, F11): the exact test by which a cell counts as a wall for a spark is read
  as "holds a wall overlay"; P11 and P17 check it against a wall, not against other overlays.
- **Spark building exceptions** (claim 16, F11): the original excepts at least two building states from stopping a spark: a gate whose door stands open far enough (the door stage is not known on S-0215 claim 35's scale), and one further state not identified. No claim is made for them. A measurement to take: a gate on flat ground with sparks falling on its cell while it is at rest closed, opening, at rest open and closing, recording per spark whether it ends at the gate and the gate's door stage in that tick (and, for the other state, the same over the other building types and states). A result would give a threshold or a state and add a claim.
- **Screen-relative damage-smoke displacement** (claim 1): when the type's flag is on, the original
  converts the displacement from a screen frame into the world; the conversion is not specified
  here. The importer warns and uses the plain displacement.
- **Electric-bolt shot** (Terms, claim 1): no approved spec owns the shot itself (its range, damage
  and draw). This spec relies only on the shot having an end point.
- **Absent game-wide spark type** (claim 1; Import mapping, game-wide spark type row): the original's result when that key is absent is not established. Our engine creates no spark system and takes no draw there, as a departure. S-0601 claim 16 has the same key and case. No measurement is proposed yet; a test rules file with the key absent, and an electric-bolt shot, would show the original's result.
- **Shared gas system's position** (claim 1, Constants): the second trigger action of claim 1 makes the system's own position observable (S-0613 claim 1 says so). P1 part b decides the position.
- **Rounding of a negative radius** (claim 20): the original's rounding is not known. Our engine rounds down, as F3 says. No measurement is proposed yet.
- **Frames a sprite does not hold** (claim 25): what the original draws for a frame number beyond
  the file's frames, or a negative one, is not established. Our engine draws nothing. Scenario
  P26 with a fire type held at its end stage and *g* = 3 would show it.
- **Spotlight look** (claim 31): the shape, colour and brightness of a particle system's
  spotlight, and how claim 15's light draw changes it (the value it moves lies between 17 and 41),
  are not analysed. No spec owns spotlights yet.
- **Laser beam look** (claim 31): owned by the spec that answers Q-1521.
- **Loops through other families** (claim 8a): loops that pass through links of another family
  are not covered by claim 8a's rule, for example a particle system whose particles' damage
  destroys an object whose death makes the same system again (S-0312), or an animation whose
  middle event requests particles (S-0109 claim 14), made in the shared gas system of S-0613
  claim 1, that lead back to that animation. Whether such a loop can close within one pass is not
  established.
- **Natural smoke and cloaking** (claim 1): S-0516 claim 18 states the removal at full cloaking and the displacement condition at full uncloaking; both specs agree. Scenario P1 parts b and c decide it.

## Changes for other specs

- **S-0613 claim 4.** "Only a system type whose section is never read has no held type and gets
  no first particle" should also name a system type whose held-type key reads `<none>`, which
  S-0004 claim 2 gives as no type for this field. Such a system gets no first particle at a
  detonation either. Scenario: S-0616 P31 part d (it can join G3's held-type variants).
- **S-0004 claim 2.** The `<none>` row for a particle system's emitted particle type gains a
  scenario: S-0616 P31 part d.
- **S-0613 claim 17.** "owner removal (above) and a trigger action (S-0616 claim 1) are the only
  other ways a system is marked here" should also name a let-go of the fire place, while the
  system is held there (S-0616 claim 1, S-0301 claim 5): it marks the system that place holds,
  whatever its kind, a gas-kind system included. Level: medium, `unmeasured` for a gas-kind
  system (no scenario puts one in the fire place), as S-0616 claim 1 gives.
