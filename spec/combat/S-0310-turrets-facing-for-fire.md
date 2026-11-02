---
id: S-0310
title: Turrets and facing for fire (aiming tolerance, which facing aims, omni-fire, firing while moving)
topic: combat
revision: 8
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0601]
supersedes: []
---

# S-0310: Turrets and facing for fire (aiming tolerance, which facing aims, omni-fire, firing while moving)

## Summary

When a firer counts as pointing its weapon at its target, how far off it may be, which firers need no
aiming, and when moving stops a firer from firing: the *not facing*, *still turning* and *moving*
answers of the firing check (S-0301, claim 9), plus the vehicle-only answers that rank next to them
and what a vehicle does in reply. How turrets and bodies turn is S-0206. This spec covers
skirmish; network games and campaigns are out of scope.

## Terms

Glossary terms are used as defined there (*facing*, *signed facing difference*, *direction*,
*reference point*, *vehicle*, *infantry*, *building*, *aircraft*, *weapon*, *target*, *destination*,
*locomotor*, *jumpjet*, *being erased*). From S-0301: *firer*, *firing check* and its answers,
*shot*, *rearm countdown*. From S-0206: *turning facing*, *body facing*, *turret facing*,
*turning*, *turn rate*, *homing projectile*, *omni-fire weapon*, *locomotor moving the vehicle*.
From S-0214: *healing actor* (called *healer* here), *counts as a vehicle*, *health ratio*
and *full health*. From S-0215:
*turreted building*, *turn-rate field*, the turret snap and its bound (claim 32,
F8). From S-0107: *interruptible* (Terms and table T1), which decides which infantry sequences
may be broken off. From S-0313: a building's *primary fire pixel shift* and *turret anchor's pixel
shift* (claims 7 and 8 there; the screen-pixel to lepton conversion is its F3). In addition:

- **Turret flag.** The imported type flag that gives a vehicle or building a turret.
- **Spinning turret.** The turret of a vehicle type with the imported turret-spins flag, which
  turns on its own when the vehicle is idle (S-0206, claim 9, spin row).
- **Aiming facing.** The facing of a firer that the firing check compares with the direction of
  the target (claims 1, 4 and 5).
- **Aim error** *ε*. |*δ*(*a*, *d*)|, the magnitude of the signed facing difference between the
  aiming facing *a* and the direction *d* of the target, from 0 to 32768 facing units.
- **Aiming tolerance** *τ*. The largest aim error at which the weapon counts as pointing at the
  target (claims 2, 4 and 5).
- **Visceroid types.** The two vehicle kinds that the rules data marks as small and large
  visceroid (each an imported flag).
- **Mobile-fire flag.** An imported vehicle-type flag; a type without it may not fire while it
  has a destination.
- **Fire-while-moving flag.** An imported weapon flag; a weapon without it may not be fired by a
  vehicle that has a destination (for a balloon-hover type, while its locomotor is moving it
  instead; claim 7).
- **Spark-particle weapon, fire-particle weapon.** A weapon with the imported spark-particle flag
  or fire-particle flag.
- **Temporal weapon.** A weapon that erases its target over time (glossary, *being erased*),
  through a beam attached to the target. A *temporal beam* is the persistent erasing capability
  of S-0514, Terms; it can exist without being attached. Claim 7 uses possession of that beam.
- **Balloon-hover type.** A vehicle type with the imported balloon-hover flag (a hovering
  locomotor of the jumpjet kind).
- **Deploy-to-fire flag.** An imported vehicle-type flag: the vehicle must deploy before it fires
  (claim 9).
- **Healing weapon.** A weapon whose damage is negative, so that it heals (S-0214, *healing actor*).
- **Installed upgrade.** A building type that has been added to another building as an upgrade
  (the structures specs); it counts for the turret flag of the building (claim 4).
- **Aiming point of a building.** The point from which a turreted building measures the
  direction of its target (claim 4).
- **Building with a voxel turret, building with a sprite turret.** A turreted building whose own
  type's imported rules voxel-turret flag is set, or is not set, respectively. These categories
  for aiming depend on that flag, not on whether a turret image successfully loads. The same
  flag selects S-0215 claim 32's voxel-turret category, subject to the correction listed in
  Changes for other specs.
- **Fighter.** An aircraft type with the imported fighter flag (S-0501 claim 11).
- **Passenger carrier.** An aircraft marked at placement because it carries passengers; the mark
  remains after they leave (S-0501 claim 1).
- **Infantry speed fraction.** A floating-point number from 0 to 1 per infantry unit, part of the
  state of its infantry locomotor (the quantity S-0107 Terms call *speed fraction*). It is 1 from
  any walking step of S-0208 (claim 6) until the unit next stops walking, and 0 from that stop
  until its next walking step. A unit stops walking when it arrives, has no walking target left,
  or its walk is cancelled or blocked. Land-type speed percentages never change it (S-0208,
  claim 9). So for a walker it is 0 or 1 and the *moving* test of claim 8 is "has taken a walking
  step and not yet stopped". The vehicle speed fraction of S-0207 is a different quantity, which
  the driving locomotor ramps.
- **Animation stage of a visceroid.** A non-negative integer per visceroid-type vehicle on which
  its drawing depends (claim 10).
- **Jump-jet turn flag.** An imported infantry-type flag (claim 8).
- **Moving now, for a jumpjet.** S-0502 claim 20's flight-state test: true in *ascending*, *cruising*,
  *descending*, *crashing* and *crashed*, false in *grounded* and *hovering*. In claims 7 and 8,
  a jumpjet locomotor *moving it* means this test, not actual displacement or a destination.

## Behaviour

### Aiming

1. **Which facing aims.** A vehicle aims with its turret facing if its type has the turret flag,
   and with its body facing otherwise. The direction of the target is the direction from the
   vehicle's reference point to the target's reference point (S-0204, claim 8). Both facings are
   the current (interpolated) values of S-0206 in the tick of the check.
   - Backing: `measurable` (scenario T1 parts a and b)
   - Confidence: high

2. **Vehicle aiming tolerance.** A vehicle's weapon points at the target when *ε* ≤ *τ*, with
   *τ* = 2048 facing units (1/32 of a turn), or 4096 when the weapon's projectile is a homing
   projectile. With *ε* > *τ* the answer is *not facing*. An omni-fire weapon, and every weapon of
   a visceroid type, is never *not facing*.
   - Backing: `measurable` (scenario T2 and S-0206 scenario R11 for the tolerances; scenario T2
     runs c and d for the omni-fire weapon and the visceroid)
   - Confidence: high

3. **Still turning.** A vehicle whose turret was turning at the end of the previous tick (S-0206,
   claim 9, "Turning, as the rows read it") gets *still turning*, whatever its aim error, unless
   the weapon's projectile is a homing projectile. A vehicle without the turret flag, or with a
   spinning turret, never has that state, so it never gets this answer (S-0206, claim 10 (b)).
   - Backing: `measurable` (scenario T3, and S-0206 scenarios R7 and R4)
   - Confidence: high

4. **Buildings.** A building is aimed only if its type, or one of its installed upgrades, has the
   turret flag (a *turreted building*, S-0215). Its aiming facing is its turret facing. The
   direction of the target is measured from the building's *aiming point* to the target's
   reference point. The aiming point is the building's reference point, shifted as follows:

   | The building's imported settings | Shift of the aiming point |
   |---|---|
   | gives a primary fire pixel shift | that shift |
   | gives none, and its turret anchor's pixel shift is not zero | the turret anchor's pixel shift |
   | gives none, and its turret anchor's pixel shift is zero | none |

   The primary fire pixel shift comes from the building's image art. The turret anchor's
   pixel shift and voxel-turret flag come only from the rules section named by the building
   type, not from its image art. Aiming uses the same imported anchor pair and voxel-turret
   flag as firing (S-0313 claim 10), with no separate art value or art fallback. Rules-layer
   omission retains the current setting as in S-0313 claim 10.
   Both screen pixel shifts are converted to leptons by S-0313, F3. A
   building with a sprite turret has *τ* = 2048 facing units, whatever the projectile. A building
   with a voxel turret has *τ* = 0: it must point exactly at the target
   (`quirks.voxel_turret_exact_aim`, Edge cases and quirks). A building that is not turreted
   fires in any direction.

   The turret snap belongs to S-0215 (claim 32 and F8 own the snap, its conditions and its bound,
   and give no switch disabling the snap itself). It acts only on a building to which this
   check has answered *not facing*, within the non-SAM attack behaviour covered by S-0215
   claim 32. A snapped building can fire in the tick of the snap unless another answer applies.
   Within that behaviour, because *τ* = 0, every aim error above 0 that reaches the aiming test
   gives *not facing*, so every such aim error within the snap bound is snapped, and every
   one if the turn-rate field is 0. With the sprite tolerance (switch off), only aim errors above
   2048 can trigger that snap.
   - Backing: `measurable` (scenario T4; S-0313 scenario P8 for the shared rules source, and
     the snap is S-0215's scenario)
   - Confidence: high (the tolerances, the flag and the shared rules source); medium (the aiming point and the rule for
     installed upgrades)

5. **Infantry and aircraft.** Infantry never get *not facing*: an infantry unit fires in whatever
   direction it faces (its sequences turn it, S-0107 and S-0208). A non-fighter aircraft compares
   its current turret facing with the direction of the target with *τ* = 2048 facing units,
   whatever the projectile. A fighter never gets *not facing*; the exemption is its type flag,
   not a flight state. After the general firing check passes, an aircraft marked as a passenger
   carrier that now carries none gets *out of ammunition*, regardless of its ammunition count
   and ahead of the aim test. This is a cargo-availability rule, not a defect or a quirk:
   passenger shots release cargo (S-0501 claim 20), and an emptied carrier remains in its
   placement-time category rather than becoming an ordinary armed aircraft. It has no switch.
   S-0501 claim 19 owns both aircraft additions.
   - Backing: `measurable` (scenario T5, parts a and b; S-0501 scenario AC11)
   - Confidence: high (infantry, aircraft tolerance and fighter exemption); medium (the
     passenger-carrier refusal)

6. **Precedence.** The aiming answers rank lowest. Every answer of the vehicle-specific reasons and
   of the general part of the firing check (S-0301, claim 9, ranks 1 to 10, such as *rearming*,
   *out of ammunition* or *out of range*) outranks every answer of this spec except the infantry
   healer's *illegal*, which outranks the general part (claim 11). The empty passenger carrier's
   *out of ammunition* follows the general part and outranks the aircraft aim test (claim 5).
   Below the general part, a vehicle's original answers rank, from highest to lowest,
   *must deploy* (claim 9), the healer's *illegal*
   (claim 11), *moving* (claim 7), *still turning* (claim 3) and *not facing* (claim 2); when
   several apply, the vehicle gets the highest-ranked one. So while its rearm countdown runs, a
   vehicle never reports *not facing*. The full rank table, with the order of *moving*, *still
   turning* and *not facing* among themselves, is S-0301's (claim 9), which also says that only the
   differences between each of *must deploy*, *moving* and *still turning* and *not facing*
   show in game state, because *not facing* can re-aim the vehicle and the other three do not.
   The order among those three has no different reply, so an implementation may choose any
   applicable one of them (S-0301 claim 9). The healer's *illegal* shows through its reply
   (claim 11).
   - Backing: `measurable` (S-0301 scenarios F7 and F8 for the countdown and the order; scenario
     T8 part b for *must deploy* before *not facing*; scenario T10 part e for the healer's
     *illegal* before *moving*). The exact order among *must deploy*, *moving* and
     *still turning* is `unmeasured`: their replies are identical, so these scenarios cannot
     distinguish it.
   - Confidence: high

### Firing while moving

7. **Vehicles.** A vehicle gets *moving* when any of these holds:
   - its type lacks the mobile-fire flag and it has a destination,
   - the weapon lacks the fire-while-moving flag and the vehicle has a destination; for a
     balloon-hover type this condition (and only this one) reads *its locomotor is moving it*
     instead of *it has a destination*,
   - the weapon is a spark-particle or fire-particle weapon and the vehicle has a destination,
   - the vehicle has a temporal beam and a destination. A primary weapon whose warhead is temporal
     supplies a beam at creation (S-0514, Terms); gunner hand-over can also supply one (S-0308
     claim 6). Attachment is irrelevant to this condition. A temporal secondary weapon alone
     supplies no beam. When the beam is attached to the very target being checked, the answer is
     *rearming*, which outranks *moving* (S-0301 claim 9, rank 4).

   Having a destination is what counts, not motion, in the first, third and fourth conditions: a
   vehicle that has stopped on the way but still has a destination gets *moving*, and one that is
   still rolling after its destination was cleared does not. Only after these conditions and
   the aiming tests pass can its locomotor add a refusal of its own (S-0301 claim 9).
   For a jumpjet locomotor the balloon-hover test uses *moving now* (Terms; S-0502 claim 20):
   grounded and hovering do not give this refusal, even with a destination. The jumpjet locomotor
   adds no separate refusal after the aiming tests (S-0502 claim 20). Other locomotors' additional
   refusals remain outside this spec.
   - Backing: `measurable` (scenario T6, parts a to j, and S-0502 scenario J14) for the four
     conditions, the living jumpjet phases and the cleared destination. The absence of a separate
     jumpjet-locomotor refusal and its crashing/crashed cases inherit S-0502 claim 20's
     `unmeasured` backing: other refusals mask them in those scenarios.
   - Confidence: high

8. **Infantry.** An infantry unit gets *moving* when its infantry speed fraction (Terms) is above
   0.1, or when it has a destination and its current sequence is not interruptible (S-0107, T1).
   A jumpjet infantry type whose type also has the jumpjet-turn flag (Terms)
   gets it while its jumpjet locomotor is *moving now* (Terms; S-0502 claim 20); one without
   that flag does not get this flight-state refusal. Grounded and hovering do not trigger it.
   A fire-particle weapon gets *moving* while the unit has a destination.
   - Backing: `measurable` (scenario T7, parts a to d) for walking, sequence, jumpjet and
     particle restrictions; S-0502 scenario J14 covers living jumpjet flight phases. Crashing
     and crashed inherit S-0502 claim 20's `unmeasured` backing because other refusals mask them.
     The exact 0.1 boundary is `unmeasured`: the walking fraction takes
     only 0 or 1, so ordinary orders cannot supply intermediate values around the boundary.
   - Confidence: medium (the fraction's 0 and 1 settings follow the walking step of S-0208; the
     exact set of stopping cases that return it to 0 is medium)

### What a vehicle does with these answers

9. **Replies in the vehicle's own firing step.** Each tick in which a vehicle has a target, it
   asks the firing check, range test included, for the weapon it would use (S-0308) and replies to the
   answer as follows. A row applies to the vehicles it names; an answer that no row names for a
   vehicle gets no reply here.

   | Answer | Vehicle | Reply in this step |
   |---|---|---|
   | *can fire* | type with the deploy-to-fire flag | queues *unload* instead of firing, without starting it immediately (S-0517 claim 13) |
   | *can fire* | visceroid type without the deploy-to-fire flag | its animation stage is set (claim 10), and it fires |
   | *can fire* | any other | fires |
   | *not facing* | type with the deploy-to-fire flag | queues *unload* without starting it immediately, and none of the three facing replies below applies (S-0517 claim 13) |
   | *not facing* | without the deploy-to-fire flag, with the turret flag | the turret's target facing becomes the direction of the target (S-0206, claim 9, facing-error row) |
   | *not facing* | without the deploy-to-fire flag, without the turret flag, with no destination, and not being moved by its locomotor | its body's target facing becomes the direction of the target, and its turret follows (S-0206, claim 9) |
   | *not facing* | without the deploy-to-fire flag, without the turret flag, with a destination or being moved by its locomotor | none |
   | *cloaked* | any | starts to uncloak when it is able to (S-0301, claim 12) |
   | *illegal* | default weapon is a healing weapon | drops its target or keeps it, as claim 11's table gives |
   | *still turning*, *moving*, *rearming*, *must deploy* and every other answer | any | none here, apart from what other specs own |

   The deploy-to-fire rows use S-0517 claim 13's rules and `quirks.deploy_to_fire_repacks`:
   with the switch on (default original), they apply even after in-place deployment, so such a
   vehicle queues *unload* again and repacks instead of firing. With it off, a deployed vehicle
   fires normally and only a mobile one queues *unload*; the deployed vehicle uses the ordinary
   firing and facing replies above. A type with both the visceroid and deploy-to-fire flags
   therefore does not set its animation stage when it queues *unload* instead of firing.

   A deploy-to-fire vehicle gets *must deploy* when no higher-ranked refusal applies
   (claim 6), and its cell does not allow it to deploy
   there (for example a sloped cell, a building in the cell, or a land
   type that does not allow it; S-0517 claim 14 owns that cell test). It then neither fires nor
   deploys. *Must deploy* ranks above *moving*, *still turning* and *not facing* (State and
   ordering), so on such a cell the vehicle does not re-aim, whatever its aim error. So a
   deploy-to-fire vehicle with the default quirk setting never fires from this step: where it
   would otherwise fire, it queues *unload* if its cell allows it, and gets *must deploy* if not.
   The switch-off deployed exception is S-0517 claim 13's alternative above.
   - Backing: `measurable` (T1 for facing replies, T8 for deployment, and S-0301 claim 12
     and scenario F7 for uncloaking). The combined visceroid/deploy-to-fire case and the
     building-occupied and land-type deployment refusals are `unmeasured`: T8 isolates only
     ordinary deployment on open ground and a slope, not those combinations.
   - Confidence: high (the facing replies); medium (deploy-to-fire)

10. **A visceroid's attack animation.** In each tick in which a vehicle of a visceroid type
    gets *can fire* and fires rather than queuing *unload* under claim 9 (in practice the tick of
    a shot, because a shot starts the rearm countdown), its
    animation stage is set to *S*<sub>*k*</sub> of the table in Constants, and from then on the
    stage rises by 5 every 5 ticks (the period and step of Constants; when the first rise comes
    after the stage is set is open, Open questions). The index is *k* = ⌊(*d* + 4096) / 8192⌋ mod 8 (integer arithmetic), where *d* is the
    direction of the target in facing units (0 to 65535; S-0204, claim 8), that is, the direction
    rounded to the nearest of eight, with a half going up. The visceroid's body facing and target
    facing are not changed by this: it fires at any angle (claim 2) and the animation stage is its
    only visible reaction. What the drawing does with the stage, which stage values it maps to
    which image frames, and what ends the attack animation, are not stated here (Open questions).
    - Backing: `measurable` (scenario T9)
    - Confidence: medium (the start value and the index); low (the period and step, and that the
      stage rises only as stated)

11. **Healing weapons and the *illegal* answer.** A vehicle whose default weapon is a healing
    weapon (Terms) gets *illegal* from the firing check when the general part of the check and
    the deploy test of claim 9 give no answer and its target is absent, or does not count as a
    vehicle (S-0214, Terms), or is at full health (health ratio at least 1). The answer ranks
    above *moving*. S-0314 claim 1 only says that a versus value of 0 makes an object an illegal
    target; this is the check's own further rule for healer vehicles. The vehicle then replies to
    *illegal* when its default weapon is a healing weapon:

    | Target | Reply |
    |---|---|
    | not a vehicle in the glossary sense (infantry, an aircraft, a building, or no object at all) | drops its target (it holds none afterwards) |
    | a vehicle in the glossary sense, health ratio at least 1 | drops its target |
    | a vehicle in the glossary sense, below full health | keeps it and does nothing in this step |

    The last row covers a target below full health that still gives *illegal*, for instance one
    whose type is marked non-vehicle (a target that does not count as a vehicle) or a versus
    value of exactly 0. For a healer that S-0309 claim 29 covers, the keep of that row lasts
    only to that claim's next check: at the first tick whose tick number is a multiple of 16 at
    which the firing check answers *illegal* (no higher-ranked answer holding), the healer drops
    the target. An infantry unit whose default weapon is a healing weapon gets
    *illegal* when its target is absent, is not infantry, or is at full health. This answer
    outranks every answer of the general part (S-0301, claim 9, ranks 4 to 10, such as *rearming* and *out of
    range*) and the *moving* answer; a vehicle healer's *illegal* ranks below the general part
    instead (S-0301, rank 12). So an infantry healer with a full-health target does not wait for its
    rearm countdown to end before the answer shows. An infantry unit whose default weapon is a
    healing weapon replies to *illegal* in its own firing step, in each of its turns, by the table
    above with *infantry* in place of *a vehicle in the glossary sense*: it drops a target that is
    not infantry or is at full health, and keeps an infantry target below full health and does
    nothing in that step. That keep, of an infantry target below full health that still gives
    *illegal* (for instance by a versus value of exactly 0), lasts only to S-0309 claim 29's next
    check, as for a vehicle healer.
    - Backing: `measurable` (scenario T10: parts a to e for vehicles, f to h for infantry).
      Absent targets, aircraft and building targets, and a damaged vehicle made illegal only
      by a zero versus value are `unmeasured`: T10 provides no accepted-order case isolating
      those checks or replies. Parts c, d and g are evidence only if their orders are accepted.
      Otherwise their wrong-kind or non-vehicle-marked cases remain `unmeasured`, because no
      ordinary order reaching the firing check in those states has been established. The keep
      row's limit at S-0309 claim 29's check is `measurable` (part d, with tick numbers). The
      infantry healer's reply in its firing step, and its keep's limit at that check, are
      `unmeasured`: not measured, since no part
      sets up the wrong-kind or full-health cases with the order accepted (parts g and h are
      evidence only if their orders are accepted).
    - Confidence: medium (the vehicle rule and the infantry rule, including its rank above the
      general part); medium (the keep row's limit at S-0309 claim 29's check); medium,
      `unmeasured` (the infantry healer's reply in its firing step and its keep's limit at that
      check)

## Formulas

**F1. Aim error and test** (claims 1, 2, 4, 5). Integers, facing units; exact integer arithmetic,
so no rounding applies. With *a* the aiming facing and *d* the direction of the target's
reference point from the firer's reference point (for a turreted building, from its aiming point,
claim 4):

> *ε* = |*δ*(*a*, *d*)|, where *δ* is S-0204's signed facing difference (−32768 … 32767), and the
> magnitude of −32768 is 32768,
> pointing ⇔ *ε* ≤ *τ*.

**F2. Tolerances.**

| Firer | *τ* (facing units) |
|---|---|
| vehicle, projectile not homing | 2048 |
| vehicle, homing projectile | 4096 |
| vehicle, omni-fire weapon or visceroid type | no test |
| building with a sprite turret | 2048 |
| building with a voxel turret | 0 |
| building that is not turreted | no test |
| infantry | no test |
| non-fighter aircraft | 2048 |
| fighter aircraft | no test |

**F3. Visceroid animation index** (claim 10). Integer arithmetic, no rounding beyond the integer
division, which truncates toward zero on non-negative values:
*k* = ⌊(*d* + 4096) / 8192⌋ mod 8, with *d* in 0 … 65535; the stage is set to *S*<sub>*k*</sub>.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Vehicle aiming tolerance | 2048 | facing units | Claim 2 |
| Vehicle aiming tolerance, homing projectile | 4096 | facing units | Claim 2 |
| Sprite turret building tolerance | 2048 | facing units | Claim 4 |
| Tolerance of a building with a voxel turret | 0 | facing units | Claim 4 |
| Non-fighter aircraft tolerance | 2048 | facing units | Claim 5 |
| Infantry speed-fraction limit | 0.1 | fraction | Claim 8; compared in double precision, a value above it gives *moving* |
| Visceroid attack stages *S*<sub>0</sub> … *S*<sub>7</sub> | 100, 105, 110, 115, 120, 125, 90, 95 | animation stage | Claim 10, F3 |
| Visceroid attack stage period | 5 | ticks | Claim 10 |
| Visceroid attack stage step | 5 | animation stage | Claim 10 |

## State and ordering

| Situation in the check | Answer |
|---|---|
| vehicle: a vehicle-specific reason for *impossible* (S-0301, claim 9, rank 1) | *impossible* |
| vehicle: launching spawned aircraft | *busy* (S-0301, claim 9, rank 2) |
| vehicle: a vehicle-specific reason for *illegal* (S-0301 claim 9, rank 3) | *illegal* |
| infantry: healing weapon and the target absent, not infantry or at full health (claim 11) | *illegal*, above every row below, even the general part |
| general part of the check gives an answer (S-0301, claim 9) | that answer |
| vehicle: deploy-to-fire and its cell does not allow deploying (claim 9) | *must deploy* |
| vehicle: healing weapon and the target absent, not counting as a vehicle or at full health (claim 11) | *illegal* |
| vehicle: a condition of claim 7 holds | *moving* |
| vehicle: turret was turning at the end of the previous tick and projectile not homing (a vehicle without the turret flag or with a spinning turret never has that state, claim 3) | *still turning* |
| vehicle: aim error above tolerance (and not omni-fire, not visceroid) | *not facing* |
| building with turret: aim error above tolerance | *not facing* |
| aircraft: marked as a passenger carrier, now carrying none (claim 5; ahead of aircraft aiming below) | *out of ammunition* |
| non-fighter aircraft: aim error above 2048 | *not facing* |
| infantry: a condition of claim 8 holds | *moving* |
| otherwise | *can fire* (subject to the locomotor's own refusal, claim 7) |

The vehicle-specific *impossible*, *busy* and *illegal* rows rank above the general part of the
check, so a vehicle in one of those situations never reports any answer below them. The infantry
healer row is the only infantry row above the general part that this spec states (the infantry
*impossible* of dying sequences is an open question).

Ordering consequences: the check of a tick sees the facing that earlier ticks' turns give at that
tick (S-0206 claim 10). A facing target set in reply to *not facing* is seen first by the
next tick's check. A non-homing weapon on a non-spinning turret must also wait until the tick
after its turret finishes turning (claim 3), not merely until it enters the aiming tolerance.
A homing weapon or a turret-less vehicle can pass the aiming test in the tick its current
facing enters its tolerance, subject to the other refusals (T1 and T3).

## Edge cases and quirks

- **Buildings with a voxel turret need an exact aim** (`quirks.voxel_turret_exact_aim`, default
  on, the original behaviour). With the switch on, *τ* = 0 for a building with a voxel turret
  (claim 4). With it off, it uses the sprite tolerance 2048. The switch does not touch the turret
  snap itself, which is S-0215 claim 32's rule: it stays as that claim gives it, so
  with the switch off the snap still happens when the aim error exceeds 2048 and is within the
  bound. The bound's wrap and its separate switch are S-0215 claim 32's, not this tolerance switch.
- **A destination, not motion, makes a vehicle "moving"** (claim 7). This is the rule itself
  rather than a defect of the original, so it has no switch and is not a quirk.
- **Deploy-to-fire repacking** (claim 9). S-0517 claim 13 owns
  `quirks.deploy_to_fire_repacks`, default original, and its alternative allowing an already
  deployed vehicle to fire normally rather than queue *unload* again.
- **Empty former passenger carriers** (claim 5). This is the cargo-availability rule itself,
  not a defect, so it has no switch. The retained placement category prevents an empty
  cargo-dropping aircraft from becoming an ordinary armed aircraft merely because ammunition
  remains. S-0501 claims 19 and 20 own the refusal and passenger-shot outcome respectively.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `Turret` (rules, vehicle or building type) | turret flag | boolean | no |
| `TurretSpins` (rules, vehicle type) | spinning turret (S-0206) | boolean | no |
| `OmniFire` (rules, weapon) | omni-fire weapon | boolean | no |
| `Fighter` (rules, aircraft type) | fighter; exempt from the aircraft aim test (S-0501 claim 19) | boolean | no |
| `FireWhileMoving` (rules, weapon) | fire-while-moving flag | boolean | yes |
| `MobileFire` (rules, vehicle type) | mobile-fire flag | boolean | yes |
| `SmallVisceroid`, `LargeVisceroid` (rules, vehicle type) | visceroid types | boolean | no |
| `BalloonHover` (rules, vehicle type) | balloon-hover type | boolean | no |
| `JumpJetTurn` (rules, infantry type) | jumpjet-turn flag (claim 8) | boolean | no |
| `DeployToFire` (rules, vehicle type) | deploy-to-fire flag (claim 9) | boolean | no |
| `ROT` (rules, projectile) | homing projectile when not 0 (S-0206) | integer | 0 |
| `TurretAnimIsVoxel` (rules, section named by building type) | building with a voxel turret (claim 4; S-0313 claim 10) | boolean | no |
| `PrimaryFirePixelOffset` (art, building image) | primary fire pixel shift (claim 4; S-0313 claim 7) | two integers, pixels | not given |
| `TurretAnimX` (rules, section named by building type) | horizontal component of turret anchor's pixel shift (claim 4; S-0313 claim 10) | integer, pixels | 0 |
| `TurretAnimY` (rules, section named by building type) | vertical component of turret anchor's pixel shift (claim 4; S-0313 claim 10) | integer, pixels | 0 |
| `UseSparkParticles`, `UseFireParticles` (rules, weapon) | spark-particle and fire-particle weapons (claims 7 and 8) | boolean | no |

## Measurement scenarios

All scenarios use an unmodified game with no add-ons and no scripted opponents, stock data unless
a named test file is given, a small purpose-made test map, and game speed index 2. The observed
house is human-controlled; one bystander computer player is isolated outside the observed test
area under the glossary's void rule. Test targets belong to the neutral house unless a scenario
names an own unit. Scripted input may issue the stated orders; a read-only recorder and video
record ticks, orders, facings, destinations, sequences, shots and the stated object state.
No recorder changes game state. Facings and the visceroid animation stage are read with the measurement setup's recorder of object state, or from
the drawn 32-step facing where stated. Unless a scenario says otherwise, each case is run twice.
Each run records its input ticks. Random rearm delays can differ between games; each run
must satisfy the expected relation using its own recorded countdown and other state, rather than
requiring identical shot ticks across games. Unless otherwise stated, the procedure is to give an attack
order on the stated target and then the stated movement, stop or unloading orders, recording until
the expected event or refusal has been observed. All refusals other than those under test must
be absent. Every scenario is invalid in a run where a vehicle is moved by something other than
the scenario's orders.

**T1. Which facing aims and the facing replies.**
- Setup: (a) a stock vehicle without a turret, weapon neither omni-fire nor homing; target placed
  at a known direction; a named facing-test rules file sets its turn rate to 1. (b) A turreted
  vehicle with the same weapon properties, initially with different body and turret facings.
  Each is stationary with no destination and all other firing refusals absent.
- Procedure: give each an attack order on the stationary target off its aiming facing. Repeat
  (a) while it moves past the target, using a named test map override enabling mobile fire and
  fire while moving and a non-track movement class, so S-0206 claim 16's independent body-aiming
  rule does not apply; keep the target outside the body's tolerance throughout this interval.
- Observable: body and turret current and target facings each tick, destination, motion and the
  tick of the first shot.
- Expected: (a) the stationary vehicle re-aims its body, its turret follows, and the shot comes
  in the first tick whose check sees a body facing within 2048 of the target direction. (b) The
  turret re-aims, not the body; shots depend on turret aim and the still-turning rule of claim 3.
  The moving turret-less vehicle gets no body target facing from the firing reply; its movement
  facing follows S-0207 instead.
- Falsified if: (a) it fires with its body more than 2048 away; (b) it aims by the differently
  directed body, or re-aims that body by this reply; or the moving turret-less vehicle's body
  target facing changes to the attack target rather than following its movement.

**T2. Vehicle tolerances, omni-fire and visceroids.**
- Setup: runs a and b: S-0206 scenario R11 (turret placed at aim errors 2048, 2304, 4096, 4352),
  without and with a homing projectile. Run c: the vehicle of R11 with a test rules file that sets
  the omni-fire flag of the weapon, at aim errors 2304 and 4352. Run d: a stock vehicle of a
  visceroid type attacking a target at an aim error of 16384 of its body facing.
- Observable: whether it fires.
- Expected: runs a and b: fires at 2048 (and 4096 with homing), not at 2304 (4352). Runs c and d:
  fires at every aim error.
- Falsified if: without a homing projectile it fires at 2304 or does not fire at 2048; with one it
  fires at 4352 or does not fire at 4096; in run c or d it fails to fire.

**T3. Still turning, with and without a homing projectile.**
- Setup: S-0206 scenario R7 runs b and c (turret turn rate 1, targets east and west, the second
  order given 32 ticks into the first turn), with the shot ticks recorded as well.
- Observable: turret facing each tick and every shot tick.
- Expected: run c (projectile not homing): no shot while the turret is turning, and the first shot
  one tick after it reaches its target. Run b (homing): a shot in a tick of the turn as soon as the
  aim error is at most 4096.
- Falsified if: run c fires during the turn, or run b waits until the turn is over.

**T4. Buildings.**
- Setup: (a) a stock sprite turret defence, a stock voxel turret defence and a stock defence without
  a turret, each given a target at a direction off its facing. Use defences without the SAM flag
  for snap observations; for the voxel one, test turn-rate
  fields 1, 5 and 0, and both homing and non-homing projectiles (the snap's own cases are
  S-0215's scenario). Named aiming-point test rules set the voxel-turret flag in the building
  type section. (b) A named aiming-point test art file gives the sprite turret defence a large
  primary fire pixel shift; the test rules give it a different nonzero turret-anchor shift,
  with their aiming directions separated by more than 2048 facing units. A target stands
  close to the building in a direction such that the direction from the shifted aiming point differs from the direction
  from the reference point by more than 2048. (c) A test rules file gives a stock building without a
  turret flag an installed upgrade whose type has the turret flag, and a target off the turret's
  facing. (d) Repeat (b) with no primary fire pixel shift and a large nonzero turret-anchor
  pixel shift. (e) Repeat with neither shift, and a target off the turret facing. In (a),
  (b) and (d), give the building a distinct image name and put conflicting turret-anchor and
  voxel-turret settings in its image art as negative controls. Repeat with only those art
  settings changed, keeping the primary fire pixel shift unchanged. S-0313 scenario P8
  separately tests art-only, omitted and layered rules settings.
- Procedure: order an attack in each case with every non-aiming refusal absent. Use close
  target positions that separate the candidate aiming directions by more than 2048 facing units.
- Observable: turret facing per tick and the shot tick.
- Expected: (a) the sprite turret fires once within 2048; the voxel turret fires only when exactly
  on the direction, which it reaches by S-0215's snap or by turning; the turret-less defence fires
  at once. (b) At its first shot the turret facing is within 2048 of the direction from the aiming
  point and more than 2048 from the direction from the reference point. (c) The building turns its
  turret and fires only within 2048 of the target direction. (d) Aim follows the turret-anchor
  shift. (e) Aim follows the unshifted reference point. Homing changes neither building tolerance.
  Changing only the art's turret-anchor or voxel-turret settings changes neither the aiming
  direction nor the tolerance; both follow the rules type settings shared with firing.
- Falsified if: the voxel turret fires with an aim error above 0; in (b) or (d) the shot comes
  within 2048 of only the unshifted direction; in (c) the building fires without aiming; in (e)
  it uses a shifted direction; homing changes a building's tolerance; or an art-only
  turret-anchor or voxel-turret change affects aiming or its tolerance.

**T5. Infantry and aircraft.**
- Setup: (a) an infantry unit given an attack order at a target behind it. (b) A stock aircraft that
  attacks ground targets, given an attack order at a target at a known aim error of its turret
  facing. A named facing-test map override repeats S-0501 scenario AC11 with the fighter flag
  off and on, and with homing and non-homing projectiles. Test errors below, at and above 2048
  in both senses, with all other refusals absent and no passenger-carrier mark.
- Observable: the shot tick and facing shown; in part b, the current turret facing and firing-check
  answer before the attack behaviour reacts. Later strafing passes that bypass *not facing* are
  not evidence that the check passes (S-0501 claim 13).
- Expected: (a) no wait for an aiming tolerance beyond its sequence turning (S-0107). (b) A
  non-fighter gets *not facing* only above 2048 with either projectile; a fighter never gets it.
  S-0501 scenario AC11 also tests placement loaded, unloading every passenger, and the retained
  mark's *out of ammunition* with ammunition remaining.
- Falsified if: (a) shots wait for a facing tolerance; (b) a non-fighter's check passes above 2048
  or refuses at 2048, a homing projectile gives 4096, a fighter gets *not facing*, or the emptied
  marked carrier passes the check with all general refusals absent.

**T6. Vehicles while moving.**
- Setup: (a) a stock vehicle with a weapon lacking the fire-while-moving flag ordered to move past
  an enemy while attacking. (b) The same vehicle stopped by a blocking unit with its destination
  kept. (c) A test rules file clearing the mobile-fire flag. (d) A test rules file gives the
  vehicle the balloon-hover flag, and the vehicle of (b) runs again. (e) A test rules file gives
  the type the mobile-fire flag and gives its weapon the fire-while-moving flag and the spark-particle
  flag. (f) The same with the fire-particle flag instead of the spark-particle flag. (g) A vehicle
  whose type lacks the mobile-fire flag, ordered to attack a target slightly out of range, so that
  it moves into range and its destination is cleared while it is still on the way to a cell centre.
  (h) A test rules file gives a vehicle with the mobile-fire flag a temporal weapon as its
  primary weapon, with the fire-while-moving flag and neither particle flag, and the vehicle is
  ordered to move past an enemy while attacking, as in (a); no beam is attached at any time before
  the first shot. (i) The same vehicle with the temporal weapon as its secondary weapon and an
  ordinary weapon, with the same flags, as its primary weapon. (j) Repeat S-0308 scenario W4
  part g's idle-beam gunner transfer into a vehicle with an ordinary primary weapon. Enable
  mobile fire and fire while moving and clear particle flags in named test data. Compare
  movement with the beam-bearing passenger aboard and after its exit, recording possession
  of the beam even while unattached. For (d), repeat S-0502 scenario J14's living flight phases.
- Observable: shot ticks against the movement record (whether the vehicle has a destination,
  its jumpjet flight state where applicable, and whether it has an unattached temporal beam).
- Expected: (a) no shots while a destination exists. (b) also none. (c) no shots with a destination
  for any weapon. (d) the blocked balloon-hover vehicle fires while its locomotor is not moving it,
  although it keeps its destination, and does not fire while its locomotor moves it. (e) and (f) no
  shots while a destination exists. (g) the first shot comes in a tick with no destination, even
  while the vehicle is still rolling. (h) no shot while a destination exists, though no beam is
  attached; shots once the destination is cleared. (i) the primary weapon fires while the
  destination exists. (j) No shot with a destination while it holds the transferred beam.
  After the passenger takes the beam back, movement alone no longer prevents a shot. In (d),
  grounded and hovering do not give the flight-state refusal; ascending, cruising and descending
  do, regardless of actual displacement.
- Falsified if: (a) or (c) fires with a destination, or (b) fires while blocked with a destination,
  (d) fires only when the destination is cleared, or fires while its locomotor moves it; (e) or (f) fires while a destination exists; (g)
  waits for the vehicle to stop; (h) fires while a destination exists; (i) does not fire while a
  destination exists; (j) a vehicle holding the transferred beam fires with a destination, or
  after losing it still refuses solely because of that destination. For (d), a different living
  flight-state boundary also falsifies the claim.

**T7. Infantry while moving.**
- Setup: (a) an infantry unit attacking while ordered to walk, with separate runs ending by
  arrival, stop order, loss of walking target and blockage. (b) One ordered to go prone while it
  has a destination. (c) A jumpjet infantry type, once with the jumpjet-turn flag and once without
  (test rules files), attacking in each living flight phase of S-0502 scenario J14. (d) A test rules file gives an
  infantry weapon the fire-particle flag; the unit attacks with a destination and again without.
- Observable: shot ticks against the recorded speed fraction, destination and sequence.
- Expected: (a) the fraction is 1 from a walking step until each stopping case and 0 thereafter,
  no shot while its speed fraction is above 0.1. (b) no shot while it is in a
  sequence that is not interruptible and has a destination. (c) the type with the flag does not
  fire while ascending, cruising or descending; grounded and hovering do not give this refusal.
  The type without it has no flight-state refusal. (d) no shot while it has a
  destination, a shot without one.
- Falsified if: (a) the fraction differs from those 0/1 settings, or (a) or (b) fires during
  the stated restriction; (c) the flagged type fires while moving or the unflagged type waits despite all other refusals being absent; (d) it fires with
  a destination or fails to fire without one despite all other refusals being absent.

**T8. Deploy-to-fire.**
- Setup: a vehicle type with the deploy-to-fire flag (a stock one if there is one, else a test rules
  file on a vehicle with a turret). (a) Given an attack order, once on flat open ground and once on a
  sloped cell. (b) On the sloped cell, a target outside its aiming tolerance.
- Observable: queued and current behaviour per tick, whether it deploys before the first shot,
  and turret and body facing. S-0517 scenario P5 additionally tests in-place repacking and the
  full cell test.
- Expected: (a) on flat ground it queues *unload* without immediately starting it, then deploys
  when ready to change. On the slope it neither queues deployment nor fires. (b) It does not
  re-aim: turret and body facings stay put.
- Falsified if: (a) it fires undeployed, starts *unload* immediately from this firing reply, or
  deploys on the slope; (b) it turns its turret or body.

**T9. Visceroid attack animation.**
- Setup: a stock vehicle of a visceroid type given an attack order on a target in each of the
  eight directions *k* = 0 … 7, plus targets just below and at each rounding boundary
  (*d* = 4095 + 8192*j* and 4096 + 8192*j*, modulo 65536, for integer *j* from 0 to 7).
  A named animation-test map override lengthens the weapon's rate of fire to keep successive
  attacks at least 30 ticks apart.
- Procedure: attack each stationary target, record the first shot and the following 20 ticks,
  then stop attacking before changing target. No movement orders occur during the observation.
- Observable: the animation stage every tick (recorder) and the drawn frames, the current and
  target body facings, and the shot tick.
- Expected: at the shot the stage equals *S*<sub>*k*</sub> (the table in Constants), and then rises
  by 5 at intervals of 5 ticks between rises (the first rise's phase is still open); neither
  the current nor target body facing changes; *d* = 4095 gives *S*<sub>0</sub> and 4096
  gives *S*<sub>1</sub>.
- Falsified if: the stage at the shot differs, rises with another period or step, or either
  body facing changes toward the target.

**T10. Healers and *illegal*.**
- Setup: a stock vehicle that is a healer (S-0214) with (a) a damaged own vehicle, (b) the same
  vehicle repaired to full health while targeted, (c) an own damaged infantry unit given as an
  attack target, (d) a damaged own vehicle whose type is marked non-vehicle in a test rules file, (e) the
  vehicle of (b), with a destination set (it was ordered to move) while its target is at full
  health. A stock infantry healer (S-0214) with (f) a damaged own infantry unit, (g) an own
  vehicle given as an attack target,
  (h) a damaged own infantry unit whose last healing shot brings it to full health while the
  healer's rearm countdown still runs.
- Procedure: give the stated healing or force-fire orders. In c, d and g, first record whether
  the order is accepted and actually leaves the intended target held by the healer. If not,
  record the part as unmeasured rather than treating cursor refusal as a firing-check result.
  For b and e, keep recording until the general check's refusals have ended.
- Observable: whether the healer still holds the target after a few ticks, and any shot; in part
  d, the tick number of each tick in which it holds the target and of the tick in which it drops
  it; in part
  h, the tick at which the healer stops attacking, against the tick its target reaches full health
  and the tick its countdown ends.
- Expected: (a) it heals; (b) it drops the target at its first firing step after full health in
  which the general check no longer refuses firing; (c) it drops the target; (d) it holds the
  target and does nothing, up to the first tick whose tick number is a multiple of 16 at which
  the firing check answers *illegal* (S-0309 claim 29), where it drops it; (e) it drops the
  full-health target despite its destination once the general check passes. (f) it heals; (g) it
  does not heal the vehicle and does not keep attacking it (claim 11 gives the reply to
  *illegal*); (h) it stops attacking within a tick or two of full health, before its countdown ends.
- Falsified if: (a) or (f) fails to heal with all other refusals absent; (b) keeps the target
  once the general check allows firing after full health, or (c) keeps the target, or (d) drops it
  at a tick other than such a multiple of 16 or keeps it past the first such tick at which the
  check answers *illegal*, (e) keeps the target once the general check passes; an
  accepted order in (g) heals the vehicle or keeps attacking it; (h) waits until the countdown ends.

## Changes for other specs

No cross-spec changes are required. S-0215 claim 32, its supporting definition and import mapping,
and its snap scenario use the building type's rules voxel-turret flag, consistent with claim 4 here
and S-0313 claim 10. The art settings do not supply a separate value or fallback.

## Open questions

- Other locomotors' own refusals (claim 7) need their locomotor specs; S-0502 claim 20 resolves
  the jumpjet case with no separate refusal. The aircraft additions formerly
  left open here are answered by S-0501 claim 19.
- Which weapon a vehicle tests each tick is S-0308's.
- What a visceroid's drawing does with the animation stage, which image frame each stage value
  shows, when the first 5-tick rise comes after the stage is set, and what ends the attack animation
  (claim 10) are not known.
- Does an infantry unit in a dying or falling sequence get *impossible* from its check, ahead of
  the healer's *illegal* of claim 11, and which sequences count? That belongs to the death specs
  (S-0312) and S-0107's sequence list; this spec states nothing about it.
