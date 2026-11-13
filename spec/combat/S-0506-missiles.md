---
id: S-0506
title: Missiles (spawned missiles, launch cycle and rocket flight)
topic: combat
revision: 41
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-13
measurements: []
requests: [Q-0610, Q-0611, Q-0612, Q-0613, Q-0615, Q-0616, Q-0645, Q-1561, Q-2159, Q-2711, Q-2748,
Q-2749, Q-2811, Q-3044]
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

S-0204 claim 13 supplies the exact raw angles used by the rocket pitch formulas. Its claim 8's
integer-facing surrogate is not a substitute. Scenario R20 tests selected raw-angle contributions.

## Terms

Glossary terms are used as defined there, in particular *aircraft*, *facing*, *step direction*,
*target*, *destination*, *on the map*, *off the map*, *in play*, *removed*, *tracked flying object*
(S-0303), *firing check* and *answer* (S-0301), *movable* (S-0214), *veterancy rank* and *height
above the terrain*.

- **Seized.** The magnetic-lift state of S-0607 claims 5 and 20, lasting until the unit gets
  its own locomotor back, including after the grip ends and during a drop. It is distinct from
  the parasite hold of S-0614 claim 8.
- **Spawn controller.** The part of an object (the *spawner*) whose type names a spawned type and
  a positive spawn count. It holds a fixed number of *spawn slots*, and is created with the spawner
  except for the cycle-closing objects of claim 1.
- **Spawned type, spawn count.** The aircraft type that the spawner's type names as the type it
  spawns, and the number of slots. Both are imported (Import mapping).
- **Slot index.** The stable launch-position number, from 0 to *N* − 1, also identifying the
  alternating muzzle side in claim 7. Indices remain the same across replacement objects.
- **Missile type.** A type with the missile-spawn flag (S-0311, Terms *missile spawn*). In this spec
  a *missile slot* (also called a *rocket slot*) is a slot whose spawned type is one of the three
  *rocket types*: the types named by the game-wide rocket settings of parameter sets A, B and C
  (Import mapping). The flag does not make a slot a missile slot. It decides the release in claim 8,
  the reset release in claim 11 and the flagged object's place in the launching count (Terms). Every
  other slot is a *non-missile slot*, including a slot of a flagged type that is not a rocket
  type. Such a slot has the launch conditions and destination of a non-missile slot (claims 3, 6
  and 7).
  The Slot table's preparing rows for a non-missile slot move its preparing object (re-sent, or
  returning when there is no controller target). While that object is still preparing, claim 8's
  release and a reset (claim 11) release it, as they release any flagged object.
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

   Each initial slot object takes one raw creation draw from the scenario random stream and
   one creation number, with its number following its own draw. The spawner's own draw and
   number precede these. Slot objects take their draws and numbers in increasing slot index,
   with each object's own initial slot objects and slaves taking theirs before the next
   slot object takes its draw and number. All initial slot objects and their descendants
   precede any initial slave group of the spawner made in the same creation (S-0402 claim 19).
   S-0611 claim 14 and F5 define this recursive draw and number order and our engine's use
   of its own deterministic creation counter rather than the original's exact global numbers.
   **Bad-data cycle exception.** When a chain of spawned and slave types returns to a type
   already in that same chain, S-0611 claim 14 and F5 override the creation rules above.
   The cycle-closing object retains its one raw creation draw and one creation number, but
   gets neither a spawn controller nor a slave group, regardless of its type's settings.
   It has no slots and no slave places, not empty ones, so no later fill, regeneration or
   replacement of children occurs for it. The exception applies to that object only, not
   every object of its type. The original has no defined outcome for this bad data; our
   engine uses this design rule and the importer warns.
   - Backing: `measurable` (scenario R1; S-0611 claim 14's scenario MU12 for creation-set variants).
     The cycle exception is `unmeasured`, design: the original has no defined outcome to
     compare; S-0611 F5's engine-only scenario MU14 checks it
   - Confidence: high for initial fill, each object's draw and number and the design exception,
     medium for nested order

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
   affects pending only under claim 25; it need not cancel a shot awaiting its first adoption.
   A nonempty pending target replaces the controller target in a
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
   returning and reloading, and back to docked (the flight itself belongs to S-0501), except that
   claim 8's release detaches the object of a flagged one (Terms).
   - Backing: `measurable` (scenarios R2, R3, R5 and R14)
   - Confidence: high

5. **Controller states.** The Controller table under State and ordering defines the controller's
   transitions and their conditions. A pass that supplies the final preparing object can release
   the salvo in that same pass, with no extra pass of waiting:
   - *idle*: a pending target becomes the controller target (and the pending target is cleared). If
     there is a controller target the controller then checks whether the spawner could attack it
     from where it stands (the weapon range test of S-0307 for the spawner weapon). If not, the
     controller *resets* (claim 11); otherwise it becomes *launching*.
   - *launching*: without a controller target it resets, even while a pending target waits, unless
     a non-missile slot has adopted that pending target earlier in the same pass (claims 3 and 11,
     `quirks.launching_spawner_resets_after_target_loss`). It does not repeat the range test.
     Otherwise it waits until every slot is preparing or empty. Then it releases (claim 8)
     and becomes *cooling down*.
   - *cooling down*: once no slot is attacking or returning, it becomes *idle*.

   For an initially idle controller with no target, the first launch comes one pass after the
   shot's target is adopted: with claim 6's conditions satisfied, a shot between two passes is
   adopted in the next pass and its first object launches in the pass after. This is not an extra
   delay for pending-target changes during an existing launch cycle (claim 3).
   - Backing: `measurable` (scenario R2; R7 part f for the launching state's reset while a pending
     target waits, after a loss at the destroying hit or on going inside, and for the adoption by a
     non-missile slot without the flag before it). `unmeasured` for that reset after a loss at
     removal, cloaking or going underground, for the adoption by a flagged slot that is not a rocket
     slot before it, and for the switch's off setting, for claim 11's reasons.
   - Confidence: high; for the launching state's reset while a pending target waits, high after a
     loss at the destroying hit or on going inside, medium for the adoption by a non-missile slot
     without the flag before it, and medium (`unmeasured`) after a loss at removal, cloaking or
     going underground, for that adoption by a flagged slot and for the switch's off setting

6. **Launch conditions and launch spacing.** A docked slot launches its object in a pass when all
   of these hold:
   - the controller has a controller target,
   - the launch spacing has run out: at least *s* ticks have passed since this controller's last
     launch, where *s* = 9 when the **spawner's own type** has the missile-spawn flag and *s* = 20
     otherwise (`quirks.launch_spacing_from_spawner_type`),
   - the controller is not cooling down,
   - the spawner, when it is a mobile unit, is not seized (S-0607 claims 5 and 20),
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
   placement point. This set-specific placement is ordinary launch geometry, not a defect or quirk:
   the displaced position is the origin of the actual flight, not merely its drawing. The object's
   facing is the spawner's body facing rounded to 256 steps (Formulas F2). A missile slot's object
   is then given the move behaviour, and the controller target as its destination when that target's
   in-the-air answer is false; when that answer is true, it is given no destination at all, neither
   that target nor any point of its own (claim 14); a non-missile slot's object is
   sent to the neighbouring cell of the spawner's cell in step direction 0 (S-0501 owns the rest).
   The slot becomes preparing.
   - Backing: `measurable` (scenario R4); (scenario R26) for no destination when the controller
     target is in the air
   - Confidence: high for the fire-point choice, the 10-lepton rise, the facing rounding and no
     destination for a controller target in the air; medium for the burst side assignment and the
     set-C shift (signs as stated)

8. **Release.** When a launching controller finds every slot preparing or empty, all preparing
   slots are released in that same pass:
   - an object without the missile-spawn flag is set on the controller target and becomes attacking,
   - an object with that flag is released with the controller target (claim 12). For a rocket slot,
     the slot goes *on the rail* for the pause time plus the tilt time of the rocket parameter set
     (Import mapping): set A for the set-A rocket type, set B otherwise
     (`quirks.raised_missile_rail_time_from_set_b`). A flagged object outside the three rocket
     types is detached at once and its slot starts regenerating.

   The release decision uses the object's flag, not merely whether its type is one of the rocket
   types. A named rocket type with the flag absent would instead enter attacking.

   If at least one missile was released, the controller then resets (claim 11; there are no
   preparing missiles left, so the reset only clears the controller target and state). It then
   becomes cooling down. All the missiles of a salvo are therefore released in the same pass even
   though they were launched (placed on the map) one spacing apart. Release is not the start of
   flight: each rocket whose destination's in-the-air answer is false at its launch begins its own
   flight cycle at that launch (claim 14), independently of when the other slots launch, and a
   rocket launched at a controller target in the air begins none then (claim 14).
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
      This is distinct from the seized state in claim 6 and can cause the reset of claim 11
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
    by an electromagnetic pulse that disables it, and when the controller target is lost while no
    pending target is waiting. The controller target is lost, among other events (Open questions),
    at the hit that destroys it, even while it then stays in play (a death sequence, a sinking or
    crashing object), when it leaves play (going inside a building or a transport, its removal),
    and when it starts to cloak or goes underground. The loss at cloaking or going underground
    comes whatever the spawner's house and whatever sensors cover the target. It has no exception,
    unlike the drop of an ordinary held target at those events, which spares a holder whose house
    senses the target's cell or which has the target's owner (S-0309 claim 28, S-0516 claim 14). The
    controller target is cleared as it is lost, before the reset, so such a reset has no target
    when it releases (claim 12). A pending target is lost at the same events, and is then cleared
    with no reset (claim 25). A reset releases every preparing object of a missile type with the current
    controller target (claim 12), makes each such slot empty with a regeneration countdown, clears
    the controller target and the pending target, and makes the controller idle. A preparing
    object without the missile-spawn flag is not affected by a reset.

    **A loss while a pending target waits** (Q-2811). The controller target is cleared at the loss
    then too, at each loss event above alike, and the loss itself gives no reset. A pending target
    replaces the controller target only at the moments of claim 3, so what follows depends on the
    controller's next pass. A launching controller adopts it in that pass only when a non-missile
    slot (Terms, including one of a flagged type that is not a rocket type) whose object is
    preparing, attacking or returning adopts it (claim 3): such a slot adopts it before the
    controller's check of the same pass, and the controller goes on launching with it. Otherwise no
    slot adopts it, since a docked slot launches only with a controller target (claim 6) and a
    rocket slot's preparing object takes no target, and the controller resets for having no
    controller target (claim 5). That reset releases each preparing object of a missile
    type with no target, so with claim 12's cell for no target, and clears the pending target. A
    rocket missile so released that has an aim point flies on to it (claim 14). For a spawner of
    rocket missiles the salvo therefore ends at the next pass after the loss, and its next launch
    needs a later shot of its spawner weapon to supply a pending target again (claim 3) and an idle
    pass to adopt it (claim 5). A controller that is cooling down keeps the pending target until a
    slot attacking or returning adopts it, or until its first idle pass (claims 3 and 5). When the
    pending target is lost as well before it is adopted, it is cleared with no reset (claim 25), and
    a launching controller resets at its next pass as above. A preparing object of a flagged slot
    that is not a rocket slot has then turned returning earlier in that pass, for want of a
    controller target (Slot table), so that reset does not release it, and it flies back to the
    spawner. So at each loss event above, neither a
    lost controller target nor a lost pending target leads to a launch at its object: a launch needs
    a controller target (claim 6), and each loss clears the record that held it
    (`quirks.launching_spawner_resets_after_target_loss`, Edge cases). The Backing gives which
    events a scenario here runs.
    - Backing: `measurable` (scenario R7) for its named events and reset effects, including the
      reset at the destroying hit (part c), at cloaking (part d) and on going underground (part e).
      `unmeasured` for an impossible firing answer as an independent reset cause: no scenario here
      separates that answer's reset from the disabling event's own reset. For a loss while a pending
      target waits: `measurable` (scenario R7 part f) for the clear and no reset at the loss, the
      reset at the next pass of a launching controller with the pending target's clear and no launch
      at the pending target in that pass, at the destroying hit and on going inside a transport,
      with the pending target lost as well, and no launch at the lost object then (its first three
      runs), for the release of a preparing missile at that reset (its fourth run), and for the
      adoption by a carrier's aircraft (its fifth run). The adoption by a preparing object of a
      flagged slot that is not a rocket slot is `measurable` as such (scenario R3's flagged
      variant, claim 3). `unmeasured` for its adoption counting for the controller's check after a
      loss, and for its turn to returning before the reset after a double loss: part f's fifth run
      uses an ordinary carrier, and no part here loses a target while such an object prepares.
      `unmeasured` for a reset's release of such a preparing object, for the same reason.
      `unmeasured` for part f's first four runs' items at removal,
      cloaking and going underground: no part here runs those events with a pending target waiting.
      `unmeasured` for a controller cooling down at the loss: no part here loses a controller target
      while its controller cools down. `unmeasured` for the switch's off setting: it is a design
      alternative for our engine. `unmeasured` for the moment of the clear with a pending target
      waiting (at the loss rather than at the next pass): part f does not record the controller
      target, and no event between the loss and the next pass reads it, so both moments give the
      same record. Part f tells only that the next pass finds no controller target.
    - Confidence: high for the reset's effect, for the loss at the destroying hit, at going inside
      and at removal, and for the target being cleared before the reset; medium for the loss at
      cloaking or going underground and its disregard of house and sensors; medium for the complete
      list of events that cause a reset. For a loss while a pending target waits: high for the
      clear, no reset at the loss, the reset at the next pass of a launching controller, the pending
      target's clear and no launch at it in that pass, at the destroying hit and on going inside,
      and with the pending target lost as well, and for no launch at the lost object then; medium
      for the release of a preparing missile at that reset and for the adoption by a non-missile
      slot before the controller's check; medium (`unmeasured`) for removal, cloaking and going
      underground with a pending target waiting, for a flagged slot that is not a rocket slot (its
      adoption counting for that check, its turn to returning after a double loss, a reset's
      release of its preparing object), for a
      controller cooling down at the loss, for the
      moment of the clear with a pending target waiting and for the switch's off setting

12. **What a release fixes.** A released missile, from claim 8 or 11 (or the spawner events of claim
    13), gets a *release cell*: the cell under the target's position when there is a target, and
    otherwise the neighbouring cell of the missile's own cell in the step direction nearest its body
    facing (Formulas F8). A missile that claim 11's reset releases when the controller target is
    lost (going inside a building or a transport, removal, a destroying hit, cloaking, going
    underground) has no target at that release, so it gets the neighbouring cell of F8, not the cell
    the lost object stood in, nor the cell of the transport or building it went into. From then on
    the game re-orders it every 30 ticks to attack the release
    cell; these orders do not change the flight of a missile whose aim point claim 14 fixes at its
    launch, because that point never changes afterwards. The 30-tick interval belongs to a shared
    re-order cycle, not a fresh per-missile delay; claim 26 defines its deadlines and ordering. A
    released object of a type without the missile-spawn flag instead starts to crash at once
    (S-0312, claim 16).
    - Backing: `measurable` for fixed aim and non-missile crash (R7, R14), and for release-cell
      selection and repeated orders with an ordinary-flight flagged aircraft (R21), including the
      cell after a controller target's loss (R21).
    - Confidence: high

13. **When the spawner dies, changes hands or loses its locomotor.** The controller *clears its
    slots* when the spawner is destroyed or starts to crash (unless it is seized
    at that moment; S-0607 claims 5 and 20), when the spawner changes house (mind control, capture), when a locomotor
    warhead replaces the spawner's locomotor, when the spawner is hit by an erasing weapon, and when
    the spawner is removed. Any crash-start draws for released aircraft occur in descending slot index, so earlier
    draw values belong to higher-index occupied slots. Its effects per slot are:

    | Slot state | Effect on the slot object |
    |---|---|
    | docked, reloading | removed silently |
    | on the rail | removed silently: a missile whose slot rail time has not ended vanishes without a detonation, regardless of its own flight phase (`quirks.rail_missile_vanishes_with_spawner`) |
    | preparing, attacking, returning | released (claim 12) with the controller target: a missile whose aim point is fixed (claim 14) flies on to it; a non-missile aircraft starts to crash |
    | empty | (nothing) |

    Each occupied slot then becomes empty. Its regeneration countdown is the spawner type's
    regeneration time when the spawner has health below 1 or is no longer in play, and 0 otherwise.
    Slots already empty keep their existing countdowns. After a house change, replacements for
    previously occupied slots therefore appear for the new house at the next pass; a slot already
    empty can still be waiting for its old regeneration countdown. Clearing the slots does not
    change the controller target, and the controller's passes go on while the spawner is seized,
    so the pending target then follows claim 25 alone. S-0607 claim 20 covers the
    launch that can follow restoration with no new shot.
    - Backing: `measurable` (scenario R8). The kept controller target and the passes while the
      spawner is seized have S-0607's backing for them: `measurable` (S-0607 scenario L19 parts b
      and d for the kept controller target, parts c and d for the passes while seized). The launch
      after restoration with no new shot has S-0607 claim 20's backing: `measurable` (S-0607
      scenario L19 part b).
    - Confidence: high for the table; medium for the list of events; medium for the kept controller
      target and the passes while seized, as S-0607 rates them; medium for the launch after
      restoration with no new shot, as S-0607 claim 20 rates it

### Rocket flight

14. **The aim point.** When the destination's in-the-air answer is false at a rocket missile's
    launch, the missile's aim point is fixed at that launch, when claim 7 gives it the controller
    target as its destination: in the launch pass, in the tick of its placement. It does not change,
    whatever the destination does afterwards. Later move orders to such a missile are ignored. The
    aim point is the destination's *heading point*:
    - for a unit: the point its locomotor is moving it to at that moment (the end of the step it is
      on), or its position when it is not moving; for a hovering vehicle, as the paragraph *A
      hovering destination* below gives; for a unit inside a tunnel, the tunnel's exit,
    - for a building: its centre, the point 128 × (*W* − 1) leptons further on the map *x* axis and
      128 × (*H* − 1) further on the map *y* axis than its position, *W* × *H* being its foundation,
      except that a building whose type is a helipad, a unit repair or reload building, or a bunker
      gives its docking point,
    - for a cell: the cell's centre point (S-0204). A missile whose controller target's in-the-air
      answer is true at its launch is given no destination then (claim 7); its aim point is the
      subject of the paragraph *A controller target in the air at the launch* below.

    A missile fired at a moving unit therefore flies to where the unit was heading at the launch,
    not to where it is when the missile arrives (`quirks.rocket_aim_at_heading_point`; off: sample
    the unit's position at that same moment).

    **A hovering destination** (Q-3044). For a hovering vehicle (S-0503 Terms), the heading point of
    the list above is its heading point *P* (S-0503 Terms, at the height given there) whenever it
    holds one at that moment, and its position when it holds none. Holding *P* is the whole test:
    whether the vehicle is powered, its throttle *T* and S-0503's *moving* play no part. So:
    - a switched-off vehicle that holds *P*, whether still above a hover height of 0 or already down
      at 0, gives *P*, although it may never travel there (S-0503 claim 10: the switch-off keeps *P*),
    - a powered vehicle that holds *P* in a tick in which its throttle is 0 because its movement
      heading is more than 45° from the direction to *P* (S-0503 claim 2) gives *P*,
    - a vehicle that keeps a destination but holds no *P*, as after a step check whose outcome is
      not *clear*, for example while it waits at a closed gate of its own house (S-0503 claim 12),
      gives its position, although S-0503 counts it as moving.

    A rocket missile launched at a switched-off vehicle that holds *P* therefore detonates where
    claim 19 gives for the aim point *P*, wherever the vehicle has come to rest. The vehicle takes
    S-0303's area damage at its own distance from that detonation point, like any other object
    there, and none when that distance exceeds the warhead's spread radius. With
    `quirks.rocket_aim_at_heading_point` off the aim point is the vehicle's position in each of
    these cases.

    **A destination that leaves play after the launch** (Q-2711). After the launch the missile's aim
    point and flight do not depend on the destination. When the destination object leaves play at
    any later time, whether at the end of the ticks it stays in play once destroyed (a death
    sequence, a sinking or crashing object), by its removal, or by going inside a building or a
    transport, the missile goes on through the phases of claim 15, on claim 24's ticks, toward the
    aim point fixed at the launch, and detonates by claim 19. The leaving changes neither that aim
    point nor those phases. A leaving by removal or by going inside is itself a removal, so, like
    any removal, it can cost the missile one turn (S-0203 claims 4 and 10), its first update in the
    launch tick included (claim 24). Claim 24's timer runs on the tick number (S-0203 claim 9), so a
    lost turn within a phase does not shift its ticks: the missile makes no update in that turn, and
    its next update finds the timer run on. A phase's end is acted on at the missile's first update
    at or after its timer runs out, and the next phase's timer starts at that update (Rocket table),
    so where the lost turn is the one in which a phase's timer runs out, the later phases come one
    tick later. In any phase, the lost turn is an update the missile does not make: none of the
    Rocket table's steps for that update (movement, height, speed or pitch) is applied. In tilt,
    whose pitch F3 computes from the elapsed ticks, the next update's pitch is the same as without
    the loss. In every other phase (pause aside, which makes no step) the path has that update's
    step fewer, and ascent ends at the first later update whose test finds cruise altitude. The
    leaving changes claim 24's ticks and the path in no other way. Other events end a missile's
    flight as they do for any rocket missile (claims 13 and 20). So no rocket missile stays on the
    map without a flight because its destination has gone, and none keeps its house alive under the
    survival test of S-0414 claims 3 and 4 for that reason: Q-2711's question 4 does not arise from
    this cause. The missile that Q-2711 reports staying without a flight was seen in our engine, not
    in the original. How a rocket missile counts in that test while it flies is S-0414's, and this
    spec does not give it. What the leaving does to the spawner depends on whether the object is
    the controller target when claim 11 counts it as lost. Claim 7 gave the controller target as the
    missile's destination, but the controller target need not still be that object: a pending target
    adopted at a later launch replaces it (claim 3), and the reset after claim 8's release, like any
    other reset of claim 11, clears it. When the leaving object is the controller target and no
    pending target is waiting, claim 11's reset follows: at the hit that destroys the object when
    it stays in play after that hit (a death sequence, a sinking or crashing object), and otherwise
    when it leaves play. The reset releases every preparing object of a missile type (claim 11)
    and makes its slot empty. A preparing rocket missile of a type without the
    missile-spawn flag is not affected by that reset (claim 11), and claims 8 and 13 apply to it as
    they would with the destination still in play. A missile that this reset releases gets claim
    12's cell for no target, because the controller target is cleared before the reset (claims 11
    and 12). A missile that the reset releases belongs to no slot from then on, so claim 13's row
    for a slot on the rail does not apply to it: when claim 13's events later clear the spawner's
    slots, it flies on, whereas a missile that claim 8 released onto the rail vanishes when they
    come during its slot's rail time. When a pending target is waiting, the controller does not
    reset at the leaving, and claim 11 gives what follows at the controller's next pass: for a
    launching spawner of rocket missiles, a reset at that pass, which releases a still-preparing
    missile of a missile type as above. A controller that is cooling down gets no reset then
    (claim 11).
    When the leaving object is not the controller target at that moment, claim 11 gives no reset:
    claims 8 and 13 apply to the missile and its salvo as they would with the destination still in
    play. This holds for a destination whose in-the-air answer is false at the launch.

    **A controller target in the air at the launch** (Q-2748). When the controller target's
    in-the-air answer is true at a rocket missile's launch, the missile is given no destination
    (claim 7). It then has no aim point and makes no flight: it stays at its placement point, in
    none of claim 15's phases, with speed 0 and pitch 0. For a set-A or set-B type its own
    in-the-air answer is then false (claim 22); for a distinct set-C type it is claim 22's ordinary
    aircraft test, so it depends on its placement point's height above the terrain. A later move
    order whose destination's in-the-air answer is true leaves it in the same way without a
    destination, so a destination in the air never fixes an aim point. The first
    destination given to it later whose in-the-air answer is false fixes its aim point, from that
    destination's heading point at that moment (the list above), and its flight then follows
    claims 15 to 21 on claim 24's ticks, *t*₀ being the tick of that fixing. Later move orders are
    then ignored as for any rocket missile. The missile does not follow the controller target: the
    target landing, leaving play or ceasing to be in the air does not itself give the missile a
    destination or fix its aim point. The loss of that target still resets the spawner as claim 11 gives, and the
    reset releases a missile still preparing with claim 12's cell for no target; which destination
    a missile is given after such a release is among the Open questions below. Which destination such a missile is first given, and in which
    tick, has not been traced (Open questions), and this spec gives no behaviour for it.
    - Backing: `measurable` (scenario R9) for the fixing at the launch, and (scenario R25) for the
      aim point, the path and the detonation after a destination leaves play by going inside, by a
      death sequence, by its removal or by sinking, early or late in the flight, and (scenario R7
      part b) for the flight on, after a later death of the spawner, of a missile of a missile type
      still preparing when the destination is lost (claim 11), and (scenario R7 part c) for the reset at
      the destroying hit of a destination that stays in play after it, and (scenario R7
      part f) for the reset at the next pass with a pending target waiting, at the destroying hit
      and on going inside (`unmeasured` at removal and for a controller cooling down, as claim 11
      gives), and (scenario R21, its
      lost-target variant) for claim 12's cell for no target after the reset, and (scenario R26)
      for a controller target in the air at the launch, the fixing by a later destination in the
      runs that record one. `unmeasured` for a later destination in the
      air not fixing an aim point: R26 gives the waiting missile no destination in the air after
      the launch, and no set-up here arranges one. `unmeasured` for the leaving not delaying the
      flight, that is
      for claim 24's ticks after it, and for what an update lost to the leaving does to the timer
      and the path: a missed turn (S-0203 claim 10) and a delay give the same record, so no set-up
      here tells them apart, and R25 rejects only an earlier first tick. `unmeasured` for no reset
      when the leaving object is not then the controller target: it rests on claim 11's list of
      reset events, which no scenario here shows to be complete. `unmeasured` for a preparing rocket
      missile of a type without the missile-spawn flag being unaffected by the reset: it rests on
      claim 11's rule for such an object. `unmeasured` for a crashing destination: no stock rocket
      launcher reliably targets an object that is not in the air at the launch and then crashes.
      `measurable` (scenario R27) for a hovering destination. The result of the M-0003 row
      restated by M-0016 that Q-3044 names (the launchers win all 12 games against switched-off
      hovering vehicles) is consistent with it; those records give outcomes only and do not by
      themselves tell it from an aim at the position.
    - Confidence: high for the fixing at the launch. High, for a destination that leaves play other
      than by crashing and a missile that loses no update after the leaving, for the aim point, the
      path and the detonation and for no missile being left on the map without a flight, which
      follows from that flight. High for the reset's release of a still-preparing missile of a
      missile type and its flight on after a later death of the spawner. Medium for all of these
      with a crashing destination, which claim 14 treats like any other leaving. Medium for what an
      update lost to the leaving does (the later phases one tick later after a lost turn at a
      phase's end, one step fewer in a phase other than pause and tilt) and for the leaving changing
      the timer and the path in no other way. High for the reset's timing (at the destroying hit
      for a destination that stays in play after it, otherwise at the leaving) and for claim 12's
      cell for no target after it. With a pending target waiting, as claim 11 rates them (scenario
      R7 part f): high for the reset at the next pass at the destroying hit and on going inside,
      medium for its release of a still-preparing missile, and medium (`unmeasured`) for both when
      the destination leaves play by its removal and for no reset of a controller that is cooling
      down. Medium for no reset when the leaving object is
      not then the controller target, as claim 11 rates its list of events. Medium for a preparing
      rocket missile without the flag being unaffected by the reset, and for the building docking
      cases and the tunnel case. For a controller target in the air at the launch: high for no
      destination, no flight and no phase while the missile has none, for the missile not
      following the target, and for the fixing by the first later destination not in the air
      from its heading point at that moment; medium for claim 24's ticks counting from the tick
      of that fixing, for pitch 0 while the missile waits, and (`unmeasured`) for a later
      destination in the air not fixing an aim point. For a hovering destination: high for *P*
      while the vehicle is switched off, above a hover height of 0 or down at 0, for *P* while its
      throttle is 0, for the position when it keeps a destination without *P*, and for the absence
      of any power, throttle or settling test

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
    of S-0303 claim 6, before the end of its tilt or raise. Claim 23 separately excludes it from
    cell scans even then. Registration, sector crossings and the alternative global hit order
    follow S-0303 claim 11; these do not change the tracking start defined here.
    Body appearance throughout every phase follows S-0105 claim 41: the body shows its
    current pitch, and the shadow ignores that pitch. Flight phases do not override drawing.
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
    - **Drawing boundary.** S-0105 claim 41 owns current-pitch body drawing, without rounding
      pitch to 32 facings, and its exact level/initial/final image-reuse eligibility classes.
      Intermediate pitches are drawn fresh; these classes do not quantise flight pitch.
      That claim also owns the pitch-independent shadow. Pitch evolution and movement remain
      governed by this spec's formulas.
    - Backing: `measurable` (scenario R10) for the flight paths in its tested distance range.
      Selected raw-angle contributions are `measurable` only where R20 obtains an update
      distinguishing the exact entries of S-0204 claim 13 from the facing surrogate of
      S-0204 claim 8. Exhaustive raw-angle equality and independence of fixed entries from
      either shared quirk setting are `unmeasured`: no legal flight geometries exposing
      every entry or mode-comparison scenario are established here.
      `unmeasured` for integer squared-distance overflow in F6 and F7: R10 does not isolate
      distances large enough to wrap the sum, and no discriminating path is established here.
    - Confidence: high for the lazy curve and the descent steering; medium for the plain-cruise
      descent test (the original computes a spatial distance whose height difference is always 0,
      which is why the test is horizontal) and for the exact numbers of Formulas F5 to F7

18. **Movement and the edge of the map.** In every tick with a rocket speed above 0 the missile
    moves by Formulas F1. It moves only when the cell of its new position is a legal cell; if it is
    not, the missile keeps its position for that tick; its phase, speed and pitch updates still
    proceed. With `quirks.rocket_stalls_at_map_edge` off it instead detonates there, at its nose point
    (claim 19), with the missile still at its last legal position (Edge cases and quirks).
    - Backing: `measurable` (scenario R10, including its named map-edge variant)
    - Confidence: high for the step; medium for the edge case

19. **Detonation.** A rocket missile detonates in the first tick in which, before its move:
    - (cruise, any mode) its height above the terrain is below 1 lepton; or
    - (lazy cruise and descent) the world height its move would reach is at or below the aim point's
      world height; or the missile is over a cell with a high bridge, the aim point's world
      height equals that cell's terrain height, and the height its move would reach is at or below
      the aim point's world height plus the 416-lepton height of a bridge deck (glossary *bridge
      deck*); or
    - (lazy cruise and descent) its height above the terrain is below 1 lepton (the same test as
      the first bullet above, which covers cruise in both modes).

    It also detonates after a positive-speed movement attempt with health below 1 (claim 20),
    even if that attempt was rejected at the map edge. When it detonates it
    stops being a tracked flying object (ending its tracking-order membership under S-0303
    claim 11), then the detonation of S-0303 takes place at its *nose point* (Formulas F9) with:
    - damage amount: the set's damage, or the set's elite damage for an elite launch; the
      firepower multipliers, veterancy and the spawner weapon's own damage play no part (S-0302,
      claim 2 does not apply: there is no shot),
    - warhead: the set's warhead, or its elite warhead for an elite launch,
    - source object: the missile; source house: none; resource reduction allowed (S-0303).
    An explosion animation chosen by S-0303 Formulas F5 for that amount and warhead appears at the
    nose point. For that choice the land type is that of the cell containing the nose point, whatever
    the missile's height (a rocket detonation has no "air" case; this gives the same choice as the
    projectile rule, because F5's splash case has its own height test), and F5's position *Q* is
    the nose point. Its animation is selected and created before the area-damage hits,
    so any animation-creation random draws precede draws caused by those hits.
    **Placement height.** The area damage, the explosion animation and the drawing-only light flash
    all take the nose point exactly, its height included. Nothing raises it to the terrain under
    it, to the top of the building hit or to the aim point's height. A steep dive can therefore
    detonate with the nose point below the surface, for example under a building on raised ground
    (claim 19's tests compare the next height with the aim point, and F9 adds a negative vertical
    component when the pitch is negative). The animation then stays at that point for its whole
    life. Its creator replaces the animation's depth adjustment by the fixed value −15 depth units,
    instead of its type's (S-0102 claim 13 takes the animation depth adjustment from the creator
    when the creator replaces it). How much of an animation placed below a building shows is
    S-0102's depth rule applied to that point and that adjustment; this spec adds no drawing lift.
    Projectile detonations follow the same placement rule at their own detonation point (S-0303
    claim 21; where that point can lie is S-0306's). The missile is
    removed after the area damage. A launch is elite when, in the last
    tick of its pause, tilt or raise (the rank is read in each of those ticks), the spawner exists
    and has the elite rank; the missile's own veterancy plays no part. Kill experience follows
    S-0311 claim 8: a trainable missile can keep its own experience instead of giving it to the spawner.
    - Backing: `measurable` (scenarios R11 and R12) except the independence from the missile's
      own veterancy: `unmeasured`, because no normal-input setup here independently promotes a
      still-linked missile before the rank-sampling cutoff. The absent source house is also
      `unmeasured`: R11 does not isolate a house-sensitive effect from the missile's owner.
    - Backing for the placement height, the land type and the −15 depth adjustment: `measurable`
      (scenario R22).
    - Confidence: high for the amount, warhead and elite choice, and for the placement at the nose
      point with no height correction and the −15 depth adjustment; medium for the bridge condition
      and for the visible result over a building (it depends on S-0102's depth rules)

20. **Interception.** A rocket missile is an aircraft-kind object with health; whether something
    can target it is S-0309's and S-0214's. When a hit destroys it:
    - above the terrain (height above the terrain 1 lepton or more): it shows its type's explosion
      list animation (S-0312, claim 16) and starts to crash, but it keeps the rocket-flight movement
      mode, and it detonates by claim 19, with its full damage and warhead, at the end of its
      next positive-speed movement attempt. A missile destroyed while its speed is 0 (in pause, tilt or raise)
      has no health-triggered detonation until a positive-speed movement attempt
      (`quirks.destroyed_rocket_still_detonates`; with it off, a
      destroyed rocket missile is removed without a detonation),
    - on the ground (height below 1): it is removed at once, without a detonation.

    The hit does not restart or suspend the rocket's phase timer. Pause, tilt and raise continue
    with the same expiry ticks as for an undamaged missile (claim 24). There is no additional
    crash-fall displacement: pause and tilt keep the position fixed, raise retains its ordinary
    rise, and positive-speed motion remains F1. S-0501 claim 9's growing fall contribution belongs
    to ordinary aircraft flight, not rocket-flight movement. Thus a missile killed in pause or
    tilt reaches its first positive-speed attempt on the same tick as its surviving counterpart,
    unless another event removes it first, such as claim 13's on-the-rail removal. Damage before
    that tick's rocket update can cause detonation in the killing tick. Existing pre-move impact
    tests still take precedence over the health-triggered detonation.
    - Backing: `measurable` (scenario R13)
    - Confidence: high

21. **Cosmetic effects.** During positive-speed flight a trailer animation appears at the
    missile's current position every 3 ticks. A set-C missile also emits take-off animations in
    pause and raise, using a 24-tick interval. These intervals share a timer: transitions can
    therefore change the delay before the first animation of the next phase. Animation behaviour
    and its random draws follow S-0109.
    - Backing: `measurable` (scenario R15)
    - Confidence: high

22. **In-the-air answer.** For an aircraft whose type is the set-A or set-B rocket type, the
    in-the-air answer is false before flight starts and during pause, tilt and raise, and true
    during ascent, cruise and descent. This phase test is independent of height, speed, health
    and rail membership. The transition to ascent at tilt/raise expiry already gives true,
    although speed stays zero until the next update. A destroying hit does not itself change
    this answer while the rocket remains in play. A distinct set-C type instead uses the
    ordinary aircraft test: on the map and height above the terrain at least 208 leptons
    (S-0307 claim 26). If parameter sets name the same type, matching A or B is enough to select
    the phase test. Weapon choice, range and area-damage distance use their owning specs'
    in-the-air rules, not flying-search registration as a substitute.
    - Backing: `measurable` (scenario R16)
    - Confidence: high

23. **Cell scans and occupancy.** A rocket missile in rocket-flight movement is not listed among
    the objects in the cell under it, in any flight phase, including pause, tilt and raise at
    its launch point. Crossing cells does not add it to those cell lists. It sets no
    object-present occupancy flag there. Consequently S-0303 claim 3's cell scan cannot hit it:
    before claim 15's tracking start it is absent from both area-damage candidate searches.
    After that start it can be found through S-0303 claim 6's flying-object search, subject to
    that search's restrictions. S-0209 claims 1 and 13 own the general occupancy and cell-list
    rules. Being below the ordinary airborne threshold does not make a rocket ground-listed.
    - Backing: `measurable` (scenario R17)
    - Confidence: high

24. **Exact flight and pass boundaries.** For a rocket missile, let *t*₀ be the tick in which
    claim 14 fixes its aim point (the tick of its launch, unless its controller target was in the
    air then), *P* the nonnegative integer pause time, and *T* the
    nonnegative integer tilt time, both in ticks. The timer starts in *t*₀ with zero elapsed ticks,
    not one elapsed tick. Its first rocket update is in that same tick, unless a removal costs it
    that turn (S-0203 claims 4 and 10). With positive acceleration and positive type speed,
    uninterrupted ordinary rocket updates and no earlier removal, the boundaries are:

    | Pause time | Phase boundaries and first positive-speed movement attempt |
    |---|---|
    | *P* = 0 | tilt starts at *t*₀, including set C; tilt ends and ascent starts at *t*₀ + *T*; first positive-speed attempt at *t*₀ + *T* + 1 |
    | *P* > 0 | pause ends at *t*₀ + *P*, starting tilt or set-C raise; that phase ends at *t*₀ + *P* + max(*T*, 1); first positive-speed attempt one tick later |

    With both times zero, ascent starts in *t*₀ and movement starts in *t*₀ + 1. With positive
    pause and zero tilt, the two transitions occupy successive ticks. Positive speed need not
    change an integer position yet: F1 truncates speed and coordinates, so the first visible
    displacement can be later. On an expiry tick the new phase's update is not also applied:
    tilt expiry fixes final pitch and tracking but does not accelerate, and pause expiry does
    not interpolate pitch or raise height. Raise adds its rate only on updates before expiry,
    so positive *T* gives *T* − 1 raise increments. Tick arithmetic here is integer.

    The ascent test uses the altitude before that tick's movement. The update whose test first finds
    the altitude at or above cruise altitude still accelerates and moves with ascent pitch and does not apply
    cruise steering; the cruise-start horizontal distance is the one at the position before that
    movement. Cruise acceleration and pitch/heading
    steering begin in the next tick. Crossing cruise altitude during an F1 step cannot apply
    cruise steering in that same tick. Likewise the plain-cruise update selecting descent still
    uses cruise pitch and acceleration; descent pitch starts in the next update.

    For a vehicle or ship spawner, a shot made by its ordinary firing step on a tick with a due
    launch pass cannot be adopted by that pass; the next pass is its first opportunity. A shot
    from an aircraft spawner's attack run can be adopted by a pass due on that same tick, under
    claims 3–5. A pending target supplied before the tick is also available to the pass. These
    observable timing rules do not define a global phase before every kind of firing (S-0203 owns
    global turn order).
    - Backing: `measurable` (scenario R18); (scenario R26) for an aim point fixed after the launch
    - Confidence: high; medium for an aim point fixed after the launch (claim 14)

25. **Pending targets without an own target.** No launch pass drops pending merely because the
    spawner has no own target. Adoption and launch still obey claims 3–6. An own-target clear
    requests a pending clear when the own target was nonempty. Assigning a nonempty target that
    is redirected to empty (S-0309 claim 24) also requests that clear, even when the previous
    own target was empty. The pending clear succeeds only when the controller target is nonempty.
    If the controller target is empty, pending survives. Asking to clear an already empty own
    target leaves pending unchanged (`quirks.spawner_pending_target_survives_idle_clear`).
    With this switch off, every such request empties pending, and any explicit own-target clear
    does so even when the own target or controller target is already empty.

    Direct nonempty own-target replacement does not itself alter pending: another spawner shot
    must supply a replacement. Clearing and restoring the same own target before a pass neither
    undoes an effective pending clear nor cancels pending that survived the clear. Move/stop
    orders, fall-backs and acquisition use these rules when they request an own-target clear
    or assignment. S-0215 claims 12–15 and S-0309 claims 23 and 24 (and claim 4 for opportunity targets) own those events.
    The discard and restoration of the own target in attack-move searches that run as for an
    object with no target (S-0309 claim 23) request no pending clear, with either quirk setting.
    Pending is unchanged whether that search finds a replacement, leaves the spawner targetless
    or restores the remembered target; subsequent shots, loss notifications and resets still
    have their usual effects. The order or behaviour name is not a separate controller
    cancellation rule.

    Loss notification for the pending target itself clears it immediately, regardless of the
    own target. Loss of the controller target clears that target and resets only when no pending
    replacement exists (claim 11). With a pending target waiting, a launching controller resets at
    its next pass unless a non-missile slot adopts that pending target first (claim 11). Explicit
    resets clear both records. Finding no candidate in a search is not an additional pending-clear
    event. An actual target-loss notification is distinct from merely observing a targetless
    spawner at a pass.
    - Backing: `measurable` (scenario R19; R7 part f for the reset at the next pass at the
      destroying hit and on going inside, and for the adoption by a non-missile slot without the
      flag), except a
      shot supplied with no own target ever set: `unmeasured`, because no normal-input setup
      establishing that independent shot is known. Its adoption rule is nevertheless fully
      specified above. The next pass after a loss at removal, cloaking or going underground with a
      pending target waiting, and that adoption by a flagged slot that is not a rocket slot, are
      `unmeasured`, as claim 11 gives.
    - Confidence: high; medium for the adoption by a non-missile slot before the reset at the next
      pass, as claim 11 rates it; medium (`unmeasured`) for the next pass after a loss at removal,
      cloaking or going underground with a pending target waiting and for that adoption by a
      flagged slot that is not a rocket slot, as claim 11 rates them, and for a shot supplied with
      no own target ever set

26. **Shared kamikaze re-orders (Q-0645).** The shared cycle is checked once per tick,
    outside object turns, before diving-aircraft steering and the turn pass (S-0203 claim 1).
    Its ammunition, target and queued
    behaviour changes are already available to each aircraft's turn in that tick. Spawn
    controllers act within their spawners' turns (claim 24), so a missile released by such a
    turn cannot receive a shared-cycle order in that same tick, even if a cycle was due then.
    This cycle is distinct from the carrier-slot orders of S-0501 claim 35.

    In a newly loaded skirmish the first cycle is due at tick number 1, not tick 0. A cycle
    schedules the next for its own tick plus 30, even when no kamikaze exists. Thus an
    uninterrupted empty game has due ticks 1, 31, 61 and so on; emptying the enrolled group
    does not stop or restart that cadence.

    A normal salvo release (claim 8), or a reset that actually releases at least one
    preparing flagged aircraft (claim 11), replaces the shared deadline with the release
    tick plus 2. At release tick *r*, neither *r* nor *r* + 1 is due from that replacement:
    the next cycle is in *r* + 2, followed by *r* + 32 unless another release intervenes.
    These are integer tick-number additions, with no rounding, counted in exactly the same
    way as the 30-tick interval. It is a replacement, not a minimum: a release can postpone
    an already imminent cycle, and repeated releases can keep postponing it. A reset that
    releases none changes no deadline. Clearing slots under claim 13 enrolls surviving
    flagged aircraft but does not itself replace the shared deadline.

    At each due cycle every still-enrolled aircraft gets ammunition set to exactly 1 round,
    its target set to its release cell and *attack* queued without forcing an immediate
    behaviour start. These changes themselves take no scenario random draws; later aircraft
    behaviour processing and firing retain their ordinary draws and gates. Orders are
    assigned in enrollment order, not aircraft creation or placement order. Removing an
    enrolled aircraft preserves the relative order of the others; later enrollments follow
    them. Within a salvo release or reset, enrollment follows increasing slot index.
    Claim 13's slot clearing instead enrolls in descending slot index. The cycle affects
    the whole enrolled group, including older salvos, not only the release that set its
    deadline. Release does not itself queue the first shared-cycle *attack*.
    - Backing: `measurable` (R21) for release deadlines, orders, ammunition, release-cell
      choice, enrollment order, turn-pass precedence and absence of immediate draws.
      `unmeasured` for the initial and empty-group cadence: an empty cycle changes no
      gameplay observable, and ordinary salvo release replaces its deadline before its
      first distinguishing order. R21's slot-clear variant tests only inherited deadlines
      that it can establish independently, not the otherwise hidden startup cadence.
    - Confidence: high

### Close targets

27. **Close targets and the launcher's own blast (Q-2159).**
    - **No shot inside the minimum range, and a reset at adoption.** A spawner's shot needs the
      firing check, whose range test includes the spawner weapon's minimum range (S-0307 claim
      9). An idle controller makes the same range test, minimum range included, when it adopts
      the target (claim 5). So a spawner whose weapon has a minimum range and is not of
      unlimited range (S-0307 claim 2) does not fire at a target inside it, and the spawner's
      approach step (S-0215 claim 37) acts on it as for any other target that is not in range:
      whether the spawner keeps the target and gets a destination follows that claim's parts d
      and f from its current behaviour and its type's can-approach property. For such a spawner, a
      controller whose adopted target is inside the minimum range at the adopting pass resets
      (claim 11) instead of launching. Being berserk changes neither test: the only firing
      refusal that the berserk state adds is S-0610 claim 6's berserk-friendly one.
    - **A target that comes close after adoption.** A launching controller does not repeat the
      range test (claim 5). A target that comes inside the minimum range, or next to the spawner,
      after the controller adopted it therefore still gets the launches of claims 6 to 8. A
      missile slot launches only while the spawner's locomotor reports that it is not moving
      (claim 6), whatever its position shows. Each rocket flies to
      the aim point that claim 14 fixes, which can lie next to the spawner.
    - **The launcher in its own blast.** The missile is the source object of its detonation (claim
      19), so it is the hit's attacker (S-0302 claim 1). Area damage leaves out that source object
      (S-0303 claim 9), so the spawner and the objects near it are candidates like any others. The
      allies row of S-0302 claim 8 reads the owner of the hit's attacker object, the missile, and
      not the detonation's source house, which claim 19 leaves empty. The missile belongs to the
      spawner's house (claim 9), and a house is allied with itself. So, for a
      spawner that has not changed house since the missile was made, when the rocket's warhead does
      not affect allies, the stage-2 allies row of S-0302 claim 8 stops the hit on the spawner and
      on the other objects of the spawner's house. For objects of other houses, S-0302 claim 8
      governs whether the allies row stops the hit, with that claim's backing and confidence.
      When the warhead affects allies, they take the
      hit like any other object within the warhead's reach. Neither rule reads the berserk state, so
      a berserk spawner's rockets damage it, and its housemates, exactly as an ordinary spawner's
      do.
    - **Crushing a berserk spawner.** Being berserk does not change whether a crusher can crush
      an object (S-0210 claim 2): that test reads the object's house, flags and state, and not
      whether it is berserk. So a crusher whose movement reaches the centre of a berserk spawner's
      cell (S-0210 claim 3) crushes it when it would crush the same spawner not berserk, and
      S-0210 claim 11's variants that drive at the crusher's target treat a berserk target the
      same way. None of S-0210 claim 11's
      choices names a cell: each sets the crusher's destination to the object itself, so the cell
      under a spawner is never taken as a crush destination.
    - **The crush response and a spawner's rockets.** A spawner's weapon launches no projectile,
      and each hit of its rockets names the missile as its attacker. So when a spawner's rocket
      hits a crusher, the crush response (S-0210 claim 11) can only name that missile as the
      crusher's destination, and never the spawner, when neither the spawner nor the crusher is
      berserk. The response's own test does not read whether the attacker, or the crusher, is
      berserk: it judges alliance by house only, through S-0210 claim 2's test. So a berserk
      spawner's rocket hit is answered in the same way, and the response never names that spawner
      either.
    - Backing: `measurable` (scenario R23: part a for the refusal inside the minimum range at
      the shot and for the destination that the approach step gives, part b for the reset at
      adoption, part c for the berserk case, part d for the
      target that comes close after adoption, part e for the blast on the spawner and a
      housemate with both settings of the affects-allies field, part f for crushing a berserk
      infantry unit along a route, part g for crushing a berserk spawner in its cell, part h for
      the response to a spawner's rocket).
      The blast on objects of other houses is S-0302 claim 8's, at that claim's backing and
      rating: part e's only object of another house is the target, whose house is allied with
      no other house, so no part here has a one-way alliance.
      That a berserk spawner's own blast is resolved like an ordinary one is `unmeasured`: no
      scenario here puts a berserk spawner within its own rocket's reach. That the target-driven
      variants and the response treat a berserk object like any other is `unmeasured`: no
      scenario here gives a crusher a berserk target, or has a berserk crushable object hit a
      crusher whose owner the response allows. Part h makes nothing berserk, so it backs the
      response naming the missile and never the spawner only when neither is berserk. A berserk
      crusher's response is S-0210 claim 11's, at that claim's rating.
    - Confidence: high for the refusal inside the minimum range at the shot, for the reset at
      adoption, and that being berserk does not change whether a crusher crushes an infantry
      unit or a spawner whose reference point lies less than 128 leptons from the crusher's
      centre on a tick on which its movement reaches a cell's centre (parts f and g compare each
      berserk object with a twin that is not). Medium for the berserk case, for the launch
      at a target that comes close after adoption, for the blast outcome on the spawner and the
      objects of its house with each setting of the affects-allies field, for a berserk
      spawner's own blast, for the allies row reading the missile's owner, for the destination
      that the approach step gives, and for the response naming the missile and never the
      spawner when neither is berserk. Low for the target-driven variants treating a berserk
      target like any other, as S-0210 claim 11 rates the omni-crusher and attack-target
      variants low, and low for the response treating a berserk attacker like any other,
      including a berserk spawner's rocket hit never naming that spawner.

## Formulas

Number domains: positions and recorded distances are integer leptons; table-root results before
truncation are floating point. Pitch is stored in single precision; rocket speed is double
precision. Pitch results in F5 to F7 are stored in single precision each tick. Every floating-point addition, subtraction, multiplication and division below forms a
53-bit result rounded toward zero, including operations on widened single operands; only an
explicit single store narrows to 24 bits, also toward zero. RZ₅₃ and RZ₂₄ denote those roundings.
Products associate left to right and parentheses fix grouping; operations are not fused.
S-0205 claims 4, 5 and 7 own the shared rounding-mode switch and conversion rules. Integer conversions below truncate
toward zero. Every sine and cosine below is the table trigonometry of S-0105 Formulas F1,
including its negative-angle quirk, not a mathematical-library sine or cosine. The table square
root *r* is defined by S-0204 claim 5, the tangent step *σ* by S-0204 claim 8, and the exact
raw angles *T* by S-0204 claim 13. *trunc* truncates toward zero.

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

*A* is the raw table arctangent of S-0204 claim 13 (the exact raw angles, the same angles used
before S-0204 claim 8's integer-facing conversion), applied to the double argument *u* with the
tangent step *σ* of S-0204 claim 8 as its unit. For *u*, the entry is the one numbered by the
magnitude of the double quotient *u* / *σ* truncated toward zero (RZ₅₃ division), and the sign
comes from RZ₂₄(*u*). When that number is past the last entry of the table of S-0204 claim 13,
the magnitude is the single-precision π/2. The entry is selected from the double quotient before
the sign is narrowed, so the narrowing does not change which entry is used. The signed value
widens exactly to a double. Neither facing corrections nor surrogate angles are applied here. Fixed raw entries remain unchanged
with either setting of the shared table-direction quirk. The rounding-mode quirk changes the
arithmetic sites stated here, not those entries.

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
only after adding the missile's position. The trigonometry is floating point. The nose point is
used as it is, with no clamping of its height to the terrain or to a building: it is the point of
the area damage, of the explosion animation and of the light flash. The cell whose land type F5
uses is the one containing the nose point, its cell coordinates being the nose's *x* and *y*
divided by 256, truncated toward zero.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| First launch pass | 20 | ticks after controller creation | Claim 2 |
| Launch pass interval | 10 | ticks | Claim 2 |
| Launch spacing, missile-spawn spawner | 9 | ticks | Claim 6 |
| Launch spacing, other spawner | 20 | ticks | Claim 6 |
| Launch rise | 10 | leptons of world height | Claim 7 |
| Set-C placement shift | 40, 40 | leptons towards map west, map north | Claim 7 |
| Release re-order interval | 30 | ticks | Claims 12 and 26 |
| First shared re-order cycle | 1 | tick number | Claim 26 |
| Salvo/reset re-order delay | 2 | ticks after release | Claim 26 |
| Return height margin (non-missile) | 20 | leptons | State table; S-0501 |
| Rocket trailer interval | 3 | ticks | Claim 21 |
| Set-C pause and raise animation interval | 24 | ticks | Claim 21 |
| Ground contact height | 1 | lepton | Claims 19, 20 |
| Detonation animation depth adjustment | −15 | depth units | Claim 19; replaces the animation type's own adjustment |
| Rocket heading constant | 16383 | facing units | F1 |
| Rocket parameter defaults | see Import mapping | varies by parameter; see each field's unit in Import mapping | Absent-key defaults |

## State and ordering

**Slot table** (claim 4; ordinary transitions at launch passes, loss notifications immediately
as in claim 9). A transition to docked does not produce
an appearance until a later pass; becoming empty starts regeneration but cannot refill the slot
in that same pass, even with a zero regeneration time. A non-missile slot's adoption of a pending
target in a pass counts for the controller's launching check of that same pass (claim 11; observable
in scenario R7 part f's fifth run as no reset at that pass). These are timing constraints, not a
required order of internal processing.

| Slot state | Event in the pass | Result |
|---|---|---|
| any occupied slot | loss notification, object health below 1, kamikaze or rocket type | empty, regeneration countdown starts immediately (claim 9) |
| any occupied non-rocket slot | loss notification or removal from play, object alive and not a kamikaze | slot unchanged (claim 9) |
| docked | claim 6 conditions hold | launch (claim 7); spacing restarts; preparing |
| docked | otherwise | none |
| preparing, rocket slot | | none (claim 8 releases it) |
| preparing, non-missile slot (flagged or not) | no controller target, after claim 3's adoption | returning |
| preparing, non-missile slot (flagged or not) | controller target, after claim 3's adoption | re-sent to the step direction 4 neighbour of the spawner's cell (S-0501) |
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
| launching | no controller target, none adopted by a non-missile slot earlier in this pass (claim 11) | reset |
| launching | some slot neither preparing nor empty (including a slot on the rail) | wait |
| launching | all slots preparing or empty | release (claim 8); reset if a missile was released; cooling down |
| cooling down | no slot attacking or returning | idle |

**Rocket table** (claims 15 to 19; phase changes and per-tick effects):

| Phase | Event | Result |
|---|---|---|
| (none) | launch (claim 7), controller target's in-the-air answer true (claim 14) | no destination, no aim point; stays at its placement point, speed 0, pitch 0 |
| (none) | destination whose in-the-air answer is true (claim 14) | ignored; no aim point (`unmeasured`, claim 14) |
| (none) | first destination whose in-the-air answer is false: at the launch (claim 7) or later (claim 14) | aim point fixed; pause (or tilt when the pause time is 0, including set C); pitch = *P*ᵢ × π/2 |
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
- **`quirks.spawner_pending_target_survives_idle_clear`** (default on). Claim 25. An own-target
  clear can leave pending alive when the controller target is empty, permitting a later launch
  despite cancellation of the own target. Off: every explicit own-target clear empties pending.
  Neither setting adds a per-pass sweep of targetless spawners.
- **`quirks.launching_spawner_resets_after_target_loss`** (default on). Claim 11: when the
  controller target is lost while a pending target waits, a launching controller none of whose
  slots adopts the pending target at its next pass resets then for having no target. It releases
  its preparing missiles with no target and drops the pending target, so the salvo ends there.
  Off: a launching controller with no controller target and a pending target waiting adopts the
  pending target at that check, with no range test, as a launch does (claim 3), and goes on
  launching. Its preparing missiles stay preparing until claim 8 releases them with that target.
- **`quirks.rocket_aim_at_heading_point`** (default on). Claim 14.
- **A destination that leaves play after the launch.** Claim 14, for a destination whose in-the-air
  answer is false at the launch: the missile flies on toward the aim point fixed at its launch and
  detonates by claim 19, unless an event that ends any rocket missile's flight ends it (claims 13
  and 20). No missile is left standing on the map for that reason, so none keeps its house alive
  under S-0414's survival test for that reason.
- **A controller target in the air at the launch.** Claim 14: the missile gets no destination and
  stands without a flight until a destination not in the air is given to it; it never follows the
  target. Which destination comes first is an Open question.
- **The release cell after a lost controller target.** Claims 11 and 12: the target is cleared
  before the reset, so the released missile gets the neighbouring cell of F8, not the lost object's
  cell or the cell of the transport or building it went into.
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
| No data key | shared re-order deadlines, enrollment priority and tick placement (claim 26) | fixed integer-tick behaviour; no per-type or per-house interval | not imported |
| No data key | raw pitch angles (claim 17), airborne answer (claim 22), cell exclusion (claim 23), tick boundaries (claim 24), pending-clear conditions (claim 25) | fixed behaviour; raw angles from S-0204 claim 13 | not imported |
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
| `V3RocketDamage`, `DMislDamage`, `CMislDamage` (rules, General) | damage of sets A, B, C | integer, hit points of damage before warhead scaling (S-0302) | A 1000, B 1000, C 500 |
| `V3RocketEliteDamage`, `DMislEliteDamage`, `CMislEliteDamage` (rules, General) | elite damage of sets A, B, C | integer, hit points of damage before warhead scaling (S-0302) | A 1000, B 1000, C 500 |
| `V3RocketBodyLength`, `DMislBodyLength`, `CMislBodyLength` (rules, General) | body length *L* of sets A, B, C | integer, leptons | A 256, B 128, C 128 |
| `V3RocketLazyCurve`, `DMislLazyCurve`, `CMislLazyCurve` (rules, General) | lazy-curve flag of sets A, B, C | boolean | A yes, B no, C no |
| `V3Warhead`, `V3EliteWarhead` (rules, CombatDamage) | set A warhead, elite warhead | warhead name | none |
| `DMislWarhead`, `DMislEliteWarhead` (rules, CombatDamage) | set B warheads | warhead name | none |
| `CMislWarhead`, `CMislEliteWarhead` (rules, CombatDamage) | set C warheads | warhead name | none |

Cycles formed by `Spawns` and `Enslaves` with positive corresponding counts are bad data.
The importer warns. Claim 1's design exception suppresses both the spawn controller and slave
group only on each cycle-closing object, retaining its creation draw and number and leaving
imported type references and counts unchanged. Such an object has neither slots nor slave
places and cannot fill children later (S-0611 claim 14 and F5).

The rocket parameter keys are in General; the warhead keys are in CombatDamage. The defaults
above are the values the original program holds at a fresh program start when no layer gives the
key, not stock rule values; a later game in the same session keeps what an earlier game left (glossary
*rules database*). Later rules layers retain the current value for an omitted key (S-0006 claim 2).

## Measurement scenarios

All scenarios: skirmish, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), a small purpose-made map, game speed index 2,
800 × 600 pixels, one human test house and one bystander computer player kept away from the test area (glossary void rule).
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

- **R1. Initial fill (claim 1).** Build a spawner; read its spawn pips (S-0111) at once. Expected:
  full count from the first tick. Falsified by pips filling over time. Also record scenario
  random state and creation numbers across creation, separating unrelated creations and
  behaviour draws. A named test-map variant gives the spawner 2 slots; another also gives
  it 3 initial slaves; a third gives each slot object's type 1 initial slave. These invented
  counts and spawn/slave type assignments are the additional overrides for this scenario.
  Expected: one raw draw and one number per created object, the spawner's first, followed
  by increasing slot index, with each slot object's initial slave before the next slot
  object, and the spawner's own slaves after all its slot objects and their descendants.
  Compare S-0611 claim 14 and F5, whose scenario MU12 tests the same creation-set order at
  animation-end creation. Our engine preserves this relative number order using its own
  counter, not the original's exact numbers. A missing creation draw or number, reversed
  slot order, or a spawner slave preceding a slot object's descendants falsifies the
  corresponding original-order prediction. Use the common five-run count and read-only
  recorder; sub-tick ordering unresolved by the samples is inconclusive.
- **R2. Pass timing and targets (claims 2–5, 8).** In range of a static target, record the spawner's
  creation tick, the actual shot tick *t*₀ (not the attack-order tick), missile appearances, aim
  points and first motion. Expected for a fresh idle controller: appearance at the second pass after
  the shot, with passes at creation + 20 and every 10 thereafter. Repeat ten times with input timed
  on either side of a pass. On a named two-slot map, send a second shot at another in-range target
  between appearances. Expected: the later launch adopts the pending target, while the earlier
  missile keeps its fixed aim. Both slots enter the rail interval together at the final launch pass,
  but their individual flight timers start from their own launches. Clear the own target before
  first adoption: with an empty controller target the pending shot survives (claim 25), rather than
  being cancelled merely by that clear. During cooling down, deliver another shot and record the
  next appearances: no launch occurs until a later pass after the return to idle. Fire again at the
  already adopted target while a different pending target is waiting: the pending replacement
  remains, not overwritten.
- **R3. Launch spacing and blockers (claims 6, 8).** A named test map gives a stock launcher two
  slots and varies only its own missile-spawn flag. Record appearance ticks: expected 20 ticks
  apart without the flag and 10 with it. In separate runs order movement before the first launch,
  or seize the launcher by magnetic lift. Expected: no rocket placement while its locomotor
  reports motion or while seized, then placement on the first eligible pass. A named
  ordinary carrier variant confirms that aircraft launch is not blocked merely by carrier motion.
  A named variant removes the missile-spawn flag from a named rocket type; its released slot
  enters attacking rather than the rail interval. Another gives an ordinary spawned aircraft
  that flag without naming it as any rocket type; its slot starts regenerating at release, with
  no rail delay. In the same variant, with the carrier moving, the object still launches at its
  first eligible pass (claim 6's motion lockout applies to rocket-type slots only),
  and before release it is sent to the neighbouring cell of the spawner's cell in step direction 0
  and not given the controller target as destination (claim 7); a pending target adopted while it
  is preparing replaces the controller target (claim 3). Record slot pips and subsequent
  appearances; a rail delay in either variant falsifies the flag/type distinction in claim 8, and
  a motion lockout (claim 6), a controller-target destination or burst alternation for the flagged
  non-rocket variant falsifies the missile-slot definition. The claim 10 lockouts that its launching
  count causes are expected, not a falsifier. On the moving-carrier run, also change the
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
- **R7. Reset.** (a) Target driven out of range in the ticks between the shot and adoption: expected
  no launch. (b) Target killed by another unit while a missile is preparing: expected the missile
  still flies to the original aim point and detonates there. In a second run of (b), with no pending
  target, record whether the missile is still preparing when the target is lost (claim 11: at the
  destroying hit); only runs in which it is count, and a run in which claim 8 released it onto the
  rail before that loss is void for this test (R8 (a) covers it). In a counted run, kill the
  spawner after the target has been lost and within the rail time that claim 8's release would
  have given that missile: expected the missile flies on and detonates by claim 19, and its
  vanishing falsifies claim 14's account of the reset. Part (c) tells the tick of the reset. Also move the target out of range after adoption: this alone does not reset a
  launching controller. Use a named pulse test weapon to disable the spawner during preparation:
  preparing missiles are released, the target clears and replacement countdowns start. Repeat target
  loss with a pending replacement target in part (f). Repeat a
  carrier reset during preparation: those unflagged aircraft remain linked rather than crashing or
  regenerating. (c) Reset at the destroying hit (claims 11 and 14): a named test launcher type
  with two or more rocket slots fires at an infantry unit that has a death sequence, and in a second variant at a
  ship on water that sinks, with no pending target; a second unit's weapon destroys the target
  while at least one missile of the salvo is preparing (runs in which none is preparing at the hit
  are void). Record per tick, with the recorder, the target's health, the tick of the destroying
  hit and the tick it leaves play, the controller state and each slot's state, and each released
  missile's release tick. Expected: the preparing missiles are released and their slots become
  empty with a regeneration countdown in the tick of the destroying hit. Falsified by a release in
  the tick the target leaves play, or later, with no release at the hit. (d) Loss at cloaking: as
  (c), but the target is a unit of a named test type that cloaks, which starts to cloak while a
  missile is preparing; in one run a sensor of the spawner's house covers the target's cell, in
  another the target belongs to the spawner's house (a forced shot). Expected in both: the reset
  and release in the tick the cloaking starts; falsified by no reset in either run. (e) Loss on
  going underground: as (c), but the target is a vehicle of an underground type (glossary) with
  tunnelling movement, given a move order far enough that it digs in and goes underground while a
  missile is preparing; one run without a sensor, and one in which a sensor of the spawner's
  house covers the cell where it goes underground. Record the tick it goes underground (the tick it is no longer drawn above the ground). Expected:
  the reset and release in that tick; falsified by no reset. (f) Loss while a pending target
  waits (claim 11, Q-2811): a named test launcher vehicle type with two or more rocket slots and a
  named test spawner weapon that can fire again within one pass interval fires at a first target,
  an infantry unit with a death sequence. After the pass that adopts that target and before the
  next pass, the launcher is ordered to attack a second target in range and fires at it, and a
  second unit's weapon then destroys the first target, still before that next pass. Runs in which
  these events come in another order, or a missile launches before the destroying hit, are void.
  Record per tick the spawner's shot ticks, the destroying hit, the controller state, each slot's
  state, each launch with its missile's destination, and each release tick and release cell.
  Expected: the controller idle after that next pass with no launch in it, and no launch at the
  second target until a later shot of the launcher at it has been adopted by a pass of the idle
  controller. Falsified by a launch at
  the second target in that next pass. Second run: as the first, but the first target boards a
  transport instead of being destroyed; expected the same. Third run: as the first, and the second
  target is destroyed too before that next pass; expected no launch in that pass and none at the
  second target. Fourth run: a named test infantry type with the same weapon and slots (claim 10's
  *busy* answer is a vehicle's only) fires at the second target while a missile launched at the
  first is preparing, and the first target is destroyed before the next pass. Expected: that
  missile released in that next pass, its slot empty with a regeneration countdown and not on the
  rail, its release cell claim 12's cell for no target, and no launch in that pass. Runs in which
  that type launches no missile are void. Fifth run: an ordinary carrier fires at the second target
  while one of its aircraft is preparing, and the first target is destroyed before the next pass.
  Expected: that aircraft does not return at that pass, and the carrier's aircraft attack the
  second target after the release (claim 8). Falsified by the preparing aircraft returning at that
  pass. Five runs per variant.
- **R8. Spawner death and capture.** Kill the spawner (a) while a missile's slot is on the rail,
  independently of the missile's own flight phase: expected the missile vanishes, no detonation; (b) after the rail interval has ended:
  expected the missile flies on. Mind-control while docked: replacements appear at the next pass.
  Repeat with one slot already empty and regenerating: that slot keeps its old remaining delay.
  Separately apply a locomotor-changing weapon or an erasing weapon during preparation: the
  preparing missile is released, whereas a rail-linked missile vanishes. For carriers include
  docked, reloading, preparing, attacking and returning aircraft: silent removal in the first two
  cases, crash in the latter three (claim 13). Repeat death while seized by magnetic lift:
  destruction or crash start alone does not clear slots while seized; a later removal does.
  Record missile existence between those events. Vanishing at the seized death/crash event,
  before removal, falsifies that exception. A carrier aircraft starting to crash also clears its
  own spawned slots if given them by a named map variant; compare with ground destruction.
  These variants name only the aircraft's spawned type and positive spawn count as extra data.
- **R9. Moving target.** Target a unit driving across the spawner's line of fire; record the impact
  point, the missile's launch tick and the end of the step the target is on at the missile's
  placement (when the target begins a new step in the launch tick, record both of that tick's step
  ends and which step was under way at the placement), and the target's step end at the missile's
  first move-behaviour processing. Use a target fast enough that the two step ends differ. Expected:
  fixed aim at the end of the step under way at the placement, not the step end at that first
  processing nor the arrival-tick position; compare the nose using F9, not an arbitrary tolerance.
  Add tunnel, ordinary building and docking-building targets to test each heading-point case of
  claim 14.
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
  rocket flight. Compare killed and surviving launches in separate runs: phase expiry ticks
  match, with no extra crash fall. Pause and tilt keep the body fixed; set-C raise keeps its
  ordinary rise. A changed expiry tick, extra downward movement or delayed first positive-speed
  attempt falsifies claim 20. A named low launch-height variant tests
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

- **R16. Phase airborne status (claim 22).** Use named A/B and distinct-C launcher variants
  with long pause/tilt and low cruise altitude, and a two-weapon defence whose ground/airborne
  choices have distinguishable shot effects. These are the additional test-map timing, altitude
  and weapon overrides. Keep all other targeting gates legal under S-0308/S-0309. Record chosen
  weapon, phase and height each tick. Expected A/B: ground choice in pause/tilt/raise, airborne
  from expiry onward even below 208 leptons, unchanged by crossing that height. Distinct C
  follows the 208-lepton threshold. Repeat a destroying hit before ascent on a long-pause
  variant: the hit alone does not change classification. A different weapon choice with all
  other gates satisfied falsifies; inability to fire alone proves no classification answer.
- **R17. Cell exclusion (claim 23).** On a named map extend the launcher's fire point over an
  otherwise empty adjacent cell, with long pause/tilt and high missile health. These are the
  additional fire-point and health test overrides. Detonate a named small-spread test weapon
  there during pause, tilt, raise and flight. Record health, phase, terrain and explosion height.
  Expected: no cell-scan hit in any phase. At terrain height choose flight altitude so S-0303
  claim 6's flying search cannot include it; repeat at eligible explosion height after tracking
  and expect a flying-search hit. Before tracking neither search hits it. Order a vehicle
  through the empty launch cell: no occupancy flag is contributed by the missile and it does
  not obstruct entry. Record occupancy as well as movement. A cell-scan hit or missile-contributed
  flag falsifies. Do not infer occupancy from a cell still holding its launcher.
- **R18. Boundary ticks (claim 24).** Extend R2/R10 with named pause/tilt pairs (0, 0), (0, 1),
  (1, 0), (1, 1), (3, 4), invented durations in ticks; include set C with and without pause.
  Give the test missile positive acceleration and positive type speed. Record aim-fixing tick,
  phase, pitch, speed, tracking and position. Expected: claim 24's exact table, raise increments
  only before expiry and first position change by F1, not necessarily on first positive speed.
  Record pre-step ascent altitude: the update finding cruise altitude still uses ascent pitch,
  with cruise steering next tick. Time a vehicle/ship's ordinary shot exactly on a pass tick:
  adoption waits until the next pass, initial launch one further pass later (claim 5). On a
  named aircraft-spawner variant give an ordinary aircraft a spawned type, positive spawn count
  and spawner weapon: its attack-run shot supplies pending before the pass and can be adopted
  in that same tick. Different boundaries, pitch-update or adoption ticks falsify.
- **R19. Pending cancellation (claims 3 and 25).** Extend R2 with move, stop, target-loss,
  fall-back and acquisition variants between shot and pass. Record own, controller and pending
  targets and appearances each tick. Use ordinary carriers for replacement shots where missiles
  would give busy. With empty controller target an own-target clear leaves pending intact.
  Do not infer cancellation from failed launch while moving (claim 6). With nonempty controller
  target, supply a different pending target then clear own target: expected immediate pending
  clear. Restore own target without another shot: pending is not restored. Direct nonempty
  own-target replacement without a clear or another shot leaves pending alone. Destroy pending's
  target: its loss notification clears pending immediately. Lose the controller target with
  and without pending replacement: reset at the loss only without replacement (R7 part (f) gives
  the next pass with a replacement). A failed search leaving an
  already empty own target unchanged does not cancel pending. Add attack-move variants under
  S-0309 claim 23 that search as for an object with no target: expected pending unchanged on
  replacement, no result and restoration of the remembered target, with either quirk setting.
  Keep referenced targets alive and avoid shots and resets during that search to isolate it.
  Where normal input can assign a nonempty dead or sinking target before its removal, test
  S-0309 claim 24's redirection to empty with an already empty own target. Expected: a pending
  clear only with a nonempty controller target under the original setting; with the quirk off,
  that request always clears pending. An unreachable redirected-assignment case is inconclusive.
  Classify by clear requests and assignments, not behaviour name or final target alone.
  Different pending changes or adoption falsify. A shot with no own target ever set remains
  outside this scenario's established normal-input reach.

- **R20. Raw-angle discrimination (claim 17).** Extend R10 with a set-A lazy-cruise missile
  and a set-B descent missile aimed at a stationary target. Vary only firing distance and
  test-map terrain height, retaining R10's parameter settings. Use the common setup, recorder
  and five-run count. Record position, fixed aim point, phase, cruise-start distance and exact
  single-precision pitch immediately before and after each rocket update. Compute the table entry
  and two F6/F7 predictions from those inputs: one using the exact raw entries required by
  S-0204 claim 13, the other using S-0204 claim 8's surrogate without integer-facing corrections.
  Seek nonzero entries, including index 1024 where reachable
  (the descent case below), and both signs of aim-height difference. Count an update only when the predictions differ after final single-precision
  rounding. Lazy cruise must have a cruise-distance ratio other than 1; descent must reach
  its line-of-sight pitch rather than only taking the maximum pitch step. Expected: the
  recorded pitch equals the exact-entry prediction. In particular, descent reaching the
  line-of-sight pitch at index 1024 gives 12841411 / 8388608 radians for a positive angle,
  its negation for a negative angle. A different pitch in a valid distinguishing update
  falsifies the exact-entry use. Coincident predictions, unavailable entries or signs, or
  observation only of drawn orientation or integer positions are inconclusive, not coverage.
  Entries without a valid distinguishing update remain unmeasured; this scenario does not
  test every entry or either shared quirk's alternative setting.

- **R21. Shared release orders (claims 12 and 26; Q-0645).** Use the common skirmish
  setup and five runs per variant. A named ordinary-flight release test map gives a carrier
  a spawned ordinary aircraft type with the missile-spawn flag, not named as any rocket
  type, and enough health to survive the observation. Give it an ordinary weapon; retain
  all other flight rules. Record release tick, release cell, enrollment sequence, ammunition,
  target, current/queued behaviours, due shared deadlines and scenario random state, with
  samples separating the shared changes from object turns. Use a read-only recorder and
  scripted ordinary attack input, never state writes. Expected after an isolated normal
  release in *r*: no shared attack in *r* or *r* + 1; ammunition becomes 1, target becomes
  the release cell and attack is queued in *r* + 2, before any object's turn, then every
  30 ticks. Move the original target after release: the ordered cell stays the release
  cell. Repeat a targetless reset with different body facings: the release cell is F8's
  neighbour. The aircraft's own turn can consume the order or ammunition, so end-of-tick
  values alone are inconclusive about the pre-turn changes.

  Release a second spawner's salvo while the first survives. Expected: both receive orders
  on the second release's replacement deadline. Arrange separate releases one tick apart,
  and another one tick before the existing deadline: each replaces it with its own tick
  plus 2, even where this postpones the older aircraft's order. Compare releases exactly
  on an old due tick: the older group receives that tick's pre-turn orders, the newly
  released object does not. A reset with no preparing flagged object leaves the deadline
  unchanged. For enrollment priority, vary spawner placement/creation order independently
  of release order, use multiple slots, then remove a middle enrolled aircraft normally.
  Record the within-cycle target/queued-behaviour changes: surviving enrollment order is
  retained, increasing slot index within normal release/reset, not creation order.

  In a separate variant clear a spawner's slots during preparation by capture or destruction.
  Expected: flagged survivors join in descending slot index without replacing the known
  shared deadline. Establish that deadline with an earlier normal release; do not infer it
  from an empty initial game. Where reachable, trigger slot clearing by a timed planted bomb
  during its carrier's turn (S-0519 claim 4): the released aircraft cannot receive that tick's
  already-completed shared-cycle order, even when the cycle was due. Record zero scenario draws across the shared assignment
  itself; exclude later behaviour processing, firing and damage from that sample. A
  different tick, cell, ammunition reset, enrollment priority, pre-turn boundary or draw
  count falsifies the corresponding prediction. If sub-tick sampling cannot resolve order
  assignment, that priority/boundary test is inconclusive, not established by equal
  end-of-tick values. Startup and empty-group cadence remain unmeasured as claim 26 states.

  Lost-target variant (claims 11 and 12; Q-2749): the test carrier type is given two or more
  slots and a launch spacing that holds some back, so that a flagged aircraft is preparing when the controller
  target is lost, with no pending target; runs in which none is preparing then are void. Lose the
  target (a) by its going inside a transport of its house standing next to it, (b) by its going
  inside a building (an engineer capturing it), (c) by a destroying hit on an infantry target with
  a death sequence, and (d) by the sale of a building target. Record each released object's
  release tick, its own cell and body facing at the release, the release cell, the lost target's
  last cell and the transport's or building's cell. Expected in every variant: the release cell is
  F8's neighbour of the released object's own cell for its body facing, whether or not an object
  stands there. Falsified by a release cell other than that neighbour. A run in which that
  neighbour is also the lost target's last cell or the transport's or building's cell does not
  tell the expected cell from those and is set aside; five runs per variant in which they differ,
  with the body facing varied between runs.

- **R22. Detonation below the surface (claim 19, F9; Q-1561).** Named map: a flat low area and a
  plateau several height levels higher, with a construction yard (or another named high-health
  test building) standing on the plateau and a second copy of it on the low ground as a control.
  A missile launcher on the low ground, set B (descent) and set A (lazy curve), fires at each
  building in turn. A named test copy of the art file points the rocket warhead's explosion
  animation at a named test image (a tall image of one flat colour), with no vertical draw shift, a type depth adjustment
  of 0 in one run and an invented positive value in a second run, so visible rows can be counted.
  Record, from tick-labelled frame captures, the missile's last positions, the explosion's screen
  position and its visible rows over the building in each frame, and the building's health.
  Expected: the explosion's draw point lies at the screen position of the nose point computed by
  F9 from the last recorded missile position, in both setups, including when that point is below
  the plateau's surface; the building's health loss matches area damage taken at that point
  (S-0303). Its visible rows
  match S-0102's depth rule evaluated at the nose point with a depth adjustment of −15, and do not
  change between the two type-adjustment runs. Falsifiers: an explosion drawn at the plateau's
  surface or at the aim point's height (a height correction); visible rows that match the type's
  adjustment, or that change between the two runs (no replacement). Repeat with a dive into open
  water and a conventional test warhead: expected the splash list (F5's splash case, decided at the
  nose point and its cell).

- **R23. Close targets and the launcher's own blast (claim 27; Q-2159).** A named test map
  overrides the spawner weapon's range and minimum range (the minimum range non-zero and well
  below the range), the pause and tilt durations (long), the rocket warhead's affects-allies
  field, reach and damage, and the spawner type's can-approach property (set to yes) and its
  deploy-to-fire setting (off, so S-0215 claim 37 f2 does not apply). All of these are test
  values. From every detonation point within the weapon's range of the spawner, the reach covers
  the positions and the cells of both the spawner and the housemate below, and the damage is high
  enough that S-0303's area damage at every such distance is above zero. Record on every tick the
  spawner's position, its current behaviour, its destination, the spatial distance from its
  range-test source point to its target, each pass's controller target and every fire event,
  launch, reset, aim point and detonation point. Also record the health of the spawner, of a test
  vehicle of the spawner's house standing in a cell next to the spawner's cell (the housemate),
  and of the target. Unless a part says otherwise, the target belongs to the spawner's house and
  the spawner force-fires at it, by a behaviour order for *attack* with that target (S-0213 claim
  5).
  - (a) A named high-health test structure (as in R11) stands inside the minimum range of the
    spawner, which is ordered to fire at it. The target is then not in range (S-0215 claim 37
    b), and in *attack* the spawner keeps it (that claim's part d). Expected: no fire event and
    no launch while the distance is below the minimum range; the spawner keeps the target and
    gets a destination by S-0215 claim 37 f; every fire event comes at a distance at or above
    the minimum range. Falsified by a fire event or a launch at a distance below the minimum
    range, or by a processing of *attack* with the target inside the minimum range after which
    the spawner has neither the target nor a destination.
  - (b) The target is a test vehicle just outside the minimum range, ordered to drive to a cell
    next to the spawner right after the spawner's fire event. Count only the runs in which the
    distance is below the minimum range at the adopting pass. Expected in a counted run: a reset
    at that pass and no launch. Falsified by a launch in a counted run.
  - (c) Repeat part a with the spawner made berserk as in S-0610 scenario P2 (that scenario's
    chaos drone, withdrawn afterwards); the berserk spawner takes the test structure as target
    on its own. Void a run in which its target is any other object, or in which the berserk
    state ends before its first fire event. Its behaviour is then *hunt* (S-0610), in which a
    type with the can-approach property keeps a target out of range (S-0215 claim 37 d).
    Expected and falsified as in part a, with *hunt* in place of *attack*.
  - (d) The target is a test vehicle outside the minimum range, ordered to drive to a cell next
    to the spawner after the adopting pass. The spawner gets no order after its fire order, and
    the cell the target drives to is free and off the spawner's cell, so nothing asks the
    spawner to move. Also record on every tick the *moving now* report (glossary) of the
    spawner's current locomotor, the report that claim 6 reads; an unchanged position does not
    establish that report. Vary the order tick, and count only the runs in which the distance
    is at least the minimum range at the adopting pass and below it at the next pass, the pass
    at which claim 5 gives the first launch, and in which that report is "not moving" at that
    next pass. Expected in a counted run: a launch at that next pass, and no reset. Falsified
    by a reset, or by no launch at that next pass, in a counted run.
  - (e) This part adds a computer house allied with no other house, owning only a named
    high-health test structure, which is the target; it stands outside the minimum range, and
    the spawner is ordered to attack it. Run once with the affects-allies field on and once with
    it off. Expected: the target loses health in both runs and survives to the end of each run.
    With the field on, the spawner and its housemate lose health by S-0303's area damage at
    their distances from the recorded detonation point; with it off, neither does. Falsified by
    a health loss of either with the field off, or by none with it on.
  - (f) A crusher of a test type of the human house drives over a cell holding a crushable
    infantry test unit of the computer house of part e that is berserk (made so by a chaos drone
    of the human house), and
    over a cell holding a twin that is not berserk. The crusher's warning can make either unit
    leave (S-0210 claim 7), so record on every tick each unit's cell and the ticks on which the
    crusher's movement reaches a cell's centre (S-0210 claim 3), and on each such tick the
    horizontal distance of S-0210 Formulas F1 from each unit's position to the crusher's
    centre. The map is flat ground with no bridge. Count a unit in a run only on such a tick
    on which that distance is less than 128 leptons. Expected: each counted unit is crushed on
    the first tick on which it is counted, the berserk one as its twin. Falsified by a counted
    unit that survives that tick. A run set in which either unit is never counted gives no
    result.
  - (g) The test map also makes the spawner type crushable (a test value). A crusher of a test
    type of the human house is ordered to drive to the cell of a spawner of the computer house of
    part e that is berserk (made so by a chaos drone of the human house), and then to the cell
    of a twin that is not berserk. Record on every tick each spawner's position, the ticks on
    which the crusher's movement reaches a cell's centre (S-0210 claim 3), and on each such tick
    the horizontal distance of S-0210 Formulas F1 from each spawner's position to the crusher's
    centre. The map is flat ground with no bridge. Count a spawner in a run only on such a tick
    on which that distance is less than 128 leptons. Void a run in which either spawner is never
    counted. Expected: each spawner is crushed on the first tick on which it is counted.
    Falsified by either one surviving that tick.
  - (h) The spawner type stays crushable as in part g, and in this part the test map gives the
    spawner weapon no minimum range and sets the affects-allies field off (test values). A
    crusher of an unarmed test type, owned by a computer-controlled house that S-0210 claim 11's
    table allows the response (as in S-0210 scenario K7), stands next to the spawner, within
    the crush range of it, and the spawner is ordered to attack it. Record the crusher's
    destination on every tick and every hit on it, with the hit's attacker by role. Expected:
    every rocket hit names the missile as attacker, and the crusher's destination is never the
    spawner. Falsified by a rocket hit that names the spawner, or by the spawner as the
    crusher's destination.
- **R24. How launchers leave play against chaos drones (Open questions; Q-2159 question 1).**
  Replay the games of M-0016 in which a chaos-drone group meets a missile-launcher group and the
  drone group wins, with the recorder. For every launcher record the tick at which it leaves
  play and what removed it: the object that made the killing hit and the hit's warhead, each by
  role, or the cause of a removal without a hit. Record also whether it was berserk then, its
  target, and for each rocket it launched the launch position, the aim point, the detonation
  point and the distance from the launcher to that point. No expected value: the record answers
  the open question.

- **R25. A destination that leaves play after the launch (claim 14; Q-2711).** A stock launcher with
  one rocket slot, of the human-controlled house, force-fires at a unit of its own house, on a small
  flat map with the recorder. Per tick from the launch until the missile leaves play, record the
  missile's position, height and pitch, its phase as drawn, its speed, and its detonation tick and
  point; record the target's heading point in the launch tick (as claim 14 gives it: its step end, or
  its position when it is not moving, or for a hovering vehicle its *P* or position) and the tick
  and way it leaves play. Variants, each with 5 runs: (a) the target is
  an infantry unit that boards a transport of its house standing next to it, ordered in so that it
  goes inside within 10 ticks after the launch; (b) the target is an engineer that enters and
  captures a building of a computer-controlled house, timed to go inside in the launch tick (the
  capture order is given first and the launcher fires when the engineer is one step from the
  building; runs in which it goes inside in another tick are void for this variant); (c) the target
  is an infantry unit killed by a second unit's weapon within 10 ticks after the launch, with a
  death sequence; (d) the target is a vehicle destroyed the same way; (e) as (d), with the vehicle
  destroyed 30 ticks or more after the launch and before the detonation; (f) where a stock launcher
  can target one, the target is a naval unit on water destroyed the same way so that it sinks. A run
  in which the target leaves play before the launch or only in or after the tick of the detonation,
  as the recorded tick of its leaving shows, or the missile is hit or disabled before its
  detonation, is void: claim 24's ticks hold only for uninterrupted ordinary rocket updates.
  Expected in every variant: the missile's first tick with positive speed is not earlier than claim
  24's first positive-speed attempt for the launch tick as *t*₀; a later one is consistent with
  claim 14 and with turns the missile misses (S-0203 claim 10), so this scenario does not test that
  the leaving leaves claim 24's ticks unchanged. Its path is that of claims 15 to 18 towards the
  target's heading point in the launch tick, and it detonates by claim 19 with its nose point (F9)
  at the detonation, then leaves play. Falsified by a first tick with positive speed earlier than
  claim 24's first positive-speed attempt for that *t*₀, *P* and *T*, by a missile whose speed is
  still 0 or that is still on the map 2000 ticks after the launch, or by a detonation elsewhere than
  claim 19 gives for that aim point.
- **R26. A rocket launched at a controller target in the air (claims 7, 14 and 24; Q-2748).** Named
  test data: a test copy of a rocket launcher type with one rocket slot whose spawner weapon may
  target objects both in the air and on the ground, of the human-controlled house, on a small flat map with the recorder,
  and a test aircraft of a computer-controlled house that flies in range and has a landing pad of
  its house on the map. Per tick from the launch, record the missile's position, height and pitch,
  its phase as drawn, its speed, its destination and the heading point of that destination when it
  is given, its current and queued behaviours, its release tick and release cell (claim 12), and
  its detonation tick and point; record the aircraft's height and in-the-air answer in the launch
  tick and its cell in the release tick. Variants, each with 5 runs: (a) the
  aircraft stays in the air; (b) it lands on its pad within 30 ticks after the launch; (c) it is
  destroyed by a second unit's weapon within 30 ticks after the launch and crashes; (d) as a
  control, the launcher fires while the aircraft stands landed on its pad, in-the-air answer false.
  Expected in (a) to (c): no destination in the launch tick; the missile stays at its placement
  point with speed 0, pitch 0 and no phase until the first recorded destination whose in-the-air
  answer is false; from that tick, as *t*₀, its phases follow claim 24 and its path claims 15 to 18
  toward that destination's heading point at that tick, and it detonates there by claim 19. For
  each run, classify that first destination (a cell near its release cell, a cell at a map edge,
  another point, or none within 2000 ticks) and record its tick: claim 14 gives no expectation for
  them. A run classified "none" decides nothing on the fixing by a later destination; it still
  counts for the waiting before it. Expected in (d): claim 14's fixing at the launch toward the
  aircraft's heading point.
  Falsified in (a) to (c) by positive speed, a pitch change or a phase before any recorded
  destination not in the air, or by a path that changes as the aircraft
  moves or lands; in any variant by an aim point that moves after it is fixed; in (d) by no fixing
  at the launch.
- **R27. A rocket launched at a hovering vehicle (claim 14; Q-3044).** A small flat map with the
  recorder; a stock rocket launcher and vehicles of a stock hovering type that is the powered unit
  of a control centre, all of the human-controlled house, with one control centre of that house in
  operation and a closed gate of that house on the map. The vehicles get their orders by normal
  input, selling the control centre switches them off (S-0405 claim 13, S-0503 claim 10), and the
  launcher fires at them by a forced attack, as in R25. Per tick from 10 ticks before the launch,
  record for each hovering vehicle its position, hover height, heading point *P* or none,
  destination or none, throttle *T* and whether it is switched off; record the missile's launch
  tick, its path, its detonation tick and point, and the health of every object within 3 cells of
  that point before and after the detonation. Variants, each with 5 runs:
  - (a) a vehicle ordered to a cell at least 4 cells away, switched off while it holds *P* and
    still above a hover height of 0; the launcher fires only after the record shows it down at
    hover height 0 and at rest for 30 ticks,
  - (b) as (a), the launcher fires while it is still above a hover height of 0,
  - (c) a control: a vehicle given no order, switched off, and fired at after it is down at hover
    height 0,
  - (d) a powered vehicle ordered along a path with a turn of at least 90°, the launch timed in the
    tick after the turn's corner cell; a run counts only when the record shows *P* held and *T* = 0
    in the launch tick,
  - (e) a powered vehicle ordered to a cell beyond the closed gate, the launch timed while it waits
    at the gate (S-0503 claim 12, the closed-friendly-gate row),
  - (f) the set-up of the M-0003 row that Q-3044 names, replayed in stock with this recorder: for
    each launch at a switched-off vehicle, its *P* or none in the launch tick and the detonation point.

  In (c) a run counts only when the record shows no *P* and no destination in the launch tick, and
  in (e) only when it shows a destination and no *P* then. Expected: in (a), (b), (d) and (f) for a
  vehicle holding *P*, a detonation where claim 19 gives for the aim point *P* of the launch tick,
  and in (c), (e) and (f) for a vehicle holding none, one for its position in the launch tick. Every
  vehicle's health loss is S-0303's area damage at its own distance from the detonation point.
  Falsified by a detonation for the vehicle's position when it held *P* in the launch tick, by one
  for any other point when it held none, or by a health loss that differs from S-0303's for the
  recorded distance.

## Open questions

- The complete list of events that reset the controller (claim 11) and clear its slots (claim 13),
  especially other disabling effects than the electromagnetic pulse, and whether a controller
  target taken over by hijacking is lost as claim 11 gives.
- Whether all building docking classifications in claim 14 are reachable as stock rocket targets.
- (Q-2748 questions 2 and 3.) Which destination whose in-the-air answer is false a rocket missile
  launched at a controller target in the air is first given, and in which tick (claim 14), has not
  been traced. The candidates are the fall-back behaviour its move behaviour takes when it finds no
  destination, which treats missile slot types apart from other aircraft (S-0501 claim 35) and
  is not given here, the shared re-order cycle's attack on its
  release cell after its release (claims 12 and 26), and other orders. For the first, S-0501
  claim 18 (*Missile types*) gives an object of a missile type the rows for an aircraft on the
  ground at any height, so that row itself gives it no destination. Nor is it traced whether
  every such missile is given one, so whether such a missile can stand on the map without a flight
  until claim 13 or 20 ends it is not given. Scenario R26 records what the original does.
- Which behaviours a rocket missile takes after its launch, with or without its destination, and the
  random draws of their processing, have not been traced. Claim 14 states only that they change
  neither its aim point nor its phases and ticks.
- (Q-2159 question 1.) How each of the last launchers leaves play in the games of M-0016 that a
  chaos-drone group wins against a missile-launcher group is not established: M-0016 records
  outcomes only. Scenario R24 records it.

## Questions for the project lead

- Q-1561: in the original, the explosion of a missile that dives steeply onto a building on
  raised ground is placed below the building's surface and is drawn with the depth of that point
  (claim 19), so the building can hide part of it. The placement also feeds game state (area
  damage and the animation's own effects), so it is not a drawing-only defect and this spec keeps
  it. Should a drawing-only remedy (for example a depth lift for detonation animations only) be
  added to S-0102 as a quirk, or should the original's drawing stand?

- Glossary *aim point*: the entry says the point is set when the missile first carries out its move
  behaviour. Claim 14 fixes it from the heading point of the first destination given to the missile
  whose in-the-air answer is false, at that moment: at the launch, when the controller target is not
  in the air then. The entry should say that.

- Confirm the proposed quirk names and alternatives, including the map-edge option, before sign-off.
