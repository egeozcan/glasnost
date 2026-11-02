---
id: S-0519
title: Terror and attack animals
topic: combat
revision: 18
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0986, Q-1524, Q-1621]
supersedes: []
---

# S-0519: Terror and attack animals

## Summary

The special attacks of infantry and vehicles that do not work like an ordinary shot. These are
attack animals, planted bombs (planting, legality, countdown, detonation, disarming, the detonate
order and credit to the planter), suicide weapons, demolition charges planted on buildings, and
the air-strike designation: a shot that calls a group of aircraft onto a building (claims 16 to 21),
and how a planted bomb's icon, a charged building's flash and a marked building are drawn
(claims 24 to 26).
S-0214 row I18 owns the air-strike cursor and its attack order, S-0308 row 6 the choice of the
designating weapon, and S-0501 the aircraft's flight and attack runs. Drone and grappling parasites belong to S-0614. Garrisons are S-0508, building capture is S-0509
and deployable units are S-0517.

Against revision 15, revision 16 takes in the S-0519 entry of S-0501's Changes for other specs.
It corrects the committed text for a carrier destroyed without a hit, as by S-0501 claim 46's
fall-back crash: such a carrier has no shared death effects, so its bomb does not detonate at the
destruction; it stays attached during the fall, where its timer and the detonate order still set
it off, and if it has not gone off by the impact it is removed then with no effect. Claim 7 adds
that removal, and the state table's row "carrier dies on the map" becomes "carrier dies on the map
from a killing hit", with the destruction without a hit and its impact as rows of their own.
Revision 17 changes no behaviour: claim 5 now points to the falling carrier of claim 7 and says
that a bomb set off during that fall makes one detonation.

## Terms

Shared terms come from the glossary: *tick*, *turn*, *weapon*, *warhead*, *projectile*, *hit*,
*detonation*, *house*, *in play, on the map, off the map, removed*, *curtain protection,
temporarily invulnerable*, *latching weapon*, *demolition warhead*, *hut demolition*, *damage that
ignores defences*, *quirk switch*. Also used:

- **Planted bomb, planter's house.** As in S-0213 (Terms). A planted bomb is attached to its
  *carrier*, the object it was planted on. Its *planter* is the infantry unit that planted it.
  The planter's house is the planter's owner at the moment of planting, and the bomb keeps it.
- **Bomb-planting warhead, disarm warhead.** Warheads marked by the two imported warhead flags of
  those names (Import mapping).
- **Timed delay *D*.** The game-wide delay in ticks between planting a bomb and its earliest
  detonation (Import mapping).
- **Bomb detector.** An object on the map whose type has a positive *bomb-sight range* in whole
  cells (Import mapping). Detectors let their owner see planted bombs (claim 10).
- **Bomb visible.** A planted bomb that the local player can see by claim 10. It is a presentation
  and input state only.
- **Suicide weapon.** A weapon with the imported suicide flag. S-0312 (Terms) already counts its
  firer as an exploding object.
- **Charge planter.** As S-0312 (Terms): the infantry unit that planted an explosive charge on a
  building. An infantry type *has explosive charges* when its type carries the explosive-charges
  flag (Import mapping).
- **Designator, strike.** A *designator* is an object whose type has a positive air-strike team
  size (Import mapping). Every designator has exactly one *strike* for as long as it exists, created
  with it; objects of other types have none. A strike holds a set of linked aircraft, a *strike
  target* (an object, or none) and an *end tick* *e*.
- **Strike aircraft.** An aircraft linked to a strike. These are the "airstrike aircraft" of
  S-0501 claim 17. A strike aircraft is a visiting aircraft (glossary).
- **Strike in progress.** A strike is *in progress* from its start (claim 17) until its last
  strike aircraft stops being linked (claim 20), or forever in the case of `quirks.airstrike_empty_strike_stuck`
  (Edge cases). Otherwise it is *idle*.
- **Designation shot.** A projectile whose warhead has the air-strike flag (Import mapping).
- **Strike size, strike aircraft type, recharge.** The designator type's team size *n*, aircraft
  type and recharge delay *R* (integer ticks). Each has a normal and an elite value. The elite
  values apply while the designator's veterancy rank (glossary) is elite.
- **Marked building.** A building that is the strike target of some strike, which has *marked* it.
  The mark's colour and brightness are presentation. Its phase sequence (claim 26) is game state,
  because one of its phases takes a scenario-stream draw.
- **Flash count.** A per-building count of ticks of flashing still to come (claim 25).
- **Mark clock.** A per-building countdown of 100 000 ticks, started whenever a start or a redirect
  marks the building (claim 26). It only bounds the mark's phase sequence.
- **Charge countdown.** The building countdown that S-0302 claim 16 calls the delayed-kill
  countdown when a charge starts it. A building has at most one, whichever started it.

## Behaviour

### Attack animals

1. **Stock attack animals use latching weapons.** The stock attack animal's weapon is a latching weapon. S-0614
   owns the whole attack: the launch (claim 1), which targets can be latched (claim 2), when the
   weapon may fire (claim 5), the kill at the first delivery to an infantry host (claim 6) and the
   release (claims 12 to 15). The animal flag alone does not give an infantry type this attack:
   the imported weapon and warhead determine it. No separate effect of the animal flag has been
   traced.
   - Backing: `measurable` (scenario T1)
   - Confidence: high

### Planted bombs

2. **Planting a bomb.** Bomb planting must be the effect selected by S-0514 claim 20.
   Takeover suppresses planting even when takeover cannot succeed; planting in turn suppresses
   all lower-priority effects even when no bomb can be planted. With planting selected, the
   projectile detonation does no area damage. It plants a bomb on its target when all of these hold:
   - the projectile's firer still exists and is an infantry unit,
   - the target is a unit, an infantry unit, an aircraft or a building,
   - the target carries no planted bomb.

   Otherwise nothing happens. The distance between the projectile and the target is not tested.
   The bomb records its planter, the planter's house, its carrier and the plant tick *t*. When the
   planter's house is the local player, the game-wide bomb-attach sound plays. The accepts-bombs
   flag of the target's type and the bomb-planter flag of the firer's type affect only the cursor
   (S-0214 row I19); planting tests neither.
   A departed superweapon warp member that is the target, off the map or phasing in after it
   appears, is an ordinary target here: it gets its bomb at the detonation, by these tests (S-0306
   claim 17).
   - Backing: `measurable` (scenarios T2 and T24; S-0306 scenario P11 run g, off the map, and run h,
     phasing in, for a departed warp member target)
   - Confidence: high; medium for a departed warp member target, as S-0306 claim 17 rates it

3. **When a bomb weapon may fire.** Two more *illegal* answers join the late target tests of the
   firing check (S-0301 claim 9, rank 9). A weapon with a bomb-planting warhead is illegal against
   a target that already carries a planted bomb. A weapon with a disarm warhead is illegal against
   a target that carries none. For an infantry firer the bomb-planting test is applied a second
   time when the general check answers *can fire*, with the same result. A firer whose weapon
   fires once drops its target after the shot (S-0309 claim 21). So a planter does not plant
   twice on one carrier, and a second planter cannot replace a bomb that is already attached.
   - Backing: `measurable` (scenario T3)
   - Confidence: high

4. **When a timed bomb goes off.** The timer is checked during the carrier's own turn,
   not in a shared detonation pass. In that check the bomb goes off when the carrier is on the
   map and the current tick is later than *t* + *D* (Formulas F1). A carrier that takes a turn
   every tick therefore sees its bomb go off in tick *t* + *D* + 1, during its own turn. No
   separate pass outside the turns sets off planted bombs. Observable consequences:
   - an object that comes before the carrier in the turn order has already taken its turn in
     that tick when the bomb's detonation hits it,
   - earlier actions of the carrier in that turn are not undone; in particular, an
     opportunity shot can precede its bomb's detonation,
   - an object that comes after the carrier and is killed takes no turn in that tick, unless
     S-0312 keeps it in play (S-0312 claim 6, S-0203 claim 1).
   - Backing: `measurable` (scenarios T4 and T5)
   - Confidence: high

5. **Detonation and credit.** When a bomb goes off on a carrier that is on the map, including
   one falling after a destruction without a hit (claim 7):
   - if the detonation kills the carrier, the bomb does not detonate a second time (S-0312
     claim 7),
   - one detonation (S-0303) happens at the carrier's position, with the game-wide bomb damage as
     its amount and the game-wide bomb warhead. Its attacker is the planter when the planter
     still exists, and none otherwise. It has no attacking house and does not ignore defences.
     The detonation may reduce resources like any other,
   - an explosion animation is selected by S-0303 Formulas F5 from the bomb damage, the bomb
     warhead, the land type of the carrier's cell and the carrier's position. When the selection
     supplies an animation, it appears at the carrier's position regardless of what the
     detonation hit. No warhead debris is thrown, because
     the bomb is not a projectile,
   - when the carrier is a bridge repair hut, the hut also demolishes its bridge (S-0511 claim 16,
     *hut demolition*),
   - the ticking sound stops.

   Kill credit and experience for objects that the detonation destroys therefore go to the
   planter, if it still exists, under S-0302 claim 1 with no attacking house.
   - Backing: `measurable` (scenarios T2, T6 and T7) for damage, effects and credit.
     The absence of a separate attacking house is `unmeasured`: ordinary credit from a
     surviving planter cannot distinguish that input from its current owner
   - Confidence: high for the detonation, its amount, warhead and position; medium for the
     credit when the planter has been removed; medium for a bomb that goes off on a carrier
     falling after a destruction without a hit, as claim 7 rates that case until S-0501 scenario
     AC29 part i runs

6. **Carriers off the map.** A bomb's timer cannot detonate it while its carrier is off the map
   (for example, inside a transport or a garrison). This remains true for open-topped passengers,
   which do take turns off the map (S-0203 claim 3). If the
   bomb is overdue when the carrier comes back, it goes off at the carrier's first turn on the
   map. A bomb that is set off while its carrier is off the map, by a detonate order or by the
   carrier's death there, is removed with no detonation, no animation and no damage. A passenger
   that dies with its transport therefore does not set off its bomb.
   - Backing: `measurable` (scenario T8)
   - Confidence: high

7. **Disarming.** A disarm removes the bomb with no damage, no animation and no credit. It happens
   when:
   - a projectile with a disarm warhead detonates on a target that carries a bomb (no distance
     test), and disarming is selected by S-0514 claim 20. Every earlier enabled row suppresses
     disarming even when that effect cannot succeed. Direct rocking is an earlier enabled row
     only for a vehicle target; disguise-making and nuke-making are later rows and cannot
     suppress disarming,
   - a building changes owner and its type is not garrisonable (S-0312 claim 7, S-0509 claim 13),
   - the carrier is removed from play other than by death: sold, used up by entering a building,
     or erased,
   - an aircraft carrier destroyed without a hit, as by S-0501 claim 46's fall-back crash,
     reaches the ground with its bomb still attached: the bomb is removed at the impact
     (`quirks.crash_drops_planted_bomb`, S-0501). Such a destruction has no shared death
     effects (S-0312 claim 7), so it does not detonate the bomb. The bomb stays attached while
     the carrier falls on the map, and its timer (claim 4) and the detonate order (claim 9) can
     still set it off by claim 5 at the carrier's position; the timer test comes in the
     carrier's turn before its movement, so a bomb due in the tick of impact goes off before the
     impact. The falling carrier's health is already 0 (S-0501 claim 46), so the detonation
     cannot kill it again: a bomb set off during the fall makes claim 5's one detonation and is
     then gone.

   A departed superweapon warp member that carries a bomb, off the map or phasing in after it
   appears, is an ordinary target for a disarm hit: it loses its bomb at the detonation, by these
   tests (S-0306 claim 17).
   - Backing: `measurable` (scenarios T6, T9 and T24; S-0501 scenario AC29 part i for the bomb
     kept through the fall-back crash's start, removed without effect at impact, and set off by a
     detonate order during the fall). A bomb whose timer comes due during that fall, and its order
     against the impact in the impact tick, are `unmeasured`, as S-0501 claim 46 gives: no normal
     input places the crash start against the bomb's due tick. The disarm hit on a departed warp
     member is `measurable` by S-0306 scenario P11 run g (off the map) and run h (phasing in).
   - Confidence: high; medium for the carrier destroyed without a hit, as S-0501 claim 46 rates the
     planted bomb's handling until AC29 part i runs, and for its timer during the fall
     (`unmeasured`); medium for the disarm hit on a departed warp member, as S-0306 claim 17 rates
     it

8. **Ownership and planter changes.** A change of owner of a unit, infantry unit or aircraft, for
   example by mind control, keeps the bomb, its timer and its planter's house. A garrisonable
   building keeps its bomb when its owner changes (claim 7). When the planter is removed from
   play the bomb forgets its planter but keeps the planter's house, so a later detonation has
   no attacker (claim 5) while the planter's house still decides visibility (claim 10) and the
   detonate order (claim 9).
   - Backing: `measurable` (scenario T9)
   - Confidence: high

9. **Detonate order.** S-0213 claim 11 owns the order, including its departure and quirk. When it
   applies, the bomb goes off at once by claim 5, or is removed silently by claim 6 when the
   carrier is off the map. Every planted bomb is a timed bomb, so only the timed-bomb permission
   of S-0214 row G3 can ever allow the order. The second permission flag has no reachable effect.
   - Backing: `measurable` (scenario T10) for on-map orders and permission flags,
     `unmeasured` for an order applied while off the map: no normal-input fixture is established
   - Confidence: high

10. **Bomb visibility, icon and sounds (presentation).** A planted bomb is visible to the local
    player when either:
    - the planter's house is the local player; or
    - a bomb detector owned by the local player is at a distance less than its bomb-sight range
      from the carrier (Formulas F2).

    The visible state is recomputed every 46 ticks, unless another planting brings the next
    recomputation forward. After a plant it is recomputed within the
    next 2 ticks. Objects become detectors when they arrive on the map and stop being detectors
    when they leave it.
    - The bomb icon is drawn over the carrier only when the bomb is visible and the carrier's
      cell is not under shroud (S-0111 claim 19 gives the shroud test). Its image frame follows
      Formulas F3. Claim 24 gives its image, palette, brightness, placement and drawing order.
    - The engineer's disarm cursor (S-0214 row I2, "active planted bomb") requires the bomb to be
      visible. The firing rules of claim 3 do not test visibility, so a disarm shot already
      ordered still works on a bomb that is not visible.
    - The game-wide ticking sound plays in a loop at the carrier's position while the carrier is
      on the map. It is silent while the carrier is off the map, and it stops when the bomb ends.

    Visibility affects presentation and the disarm cursor, not bomb timing or damage.
    - Backing: `measurable` (scenario T11) for visibility, sounds and frames,
      `unmeasured` for the lepton-scale distinction between the table root and an exact root in
      F2: no validated detector placement isolates that boundary
    - Confidence: high

### Suicide weapons

11. **A suicide shot.** When a firer makes a shot with a suicide weapon, it creates no projectile,
    plays no report sound, starts no rearm countdown and uses no ammunition. Instead the firer
    takes one hit with all of these properties:
    - its amount is the strength of the firer's type, not its current health,
    - it is at distance 0,
    - its warhead is the demolition warhead,
    - it has no attacker and no attacking house,
    - it ignores defences and carries no no-escape marker.

    This happens only for a shot that the firing check allows (S-0301 claim 9), and for a vehicle
    in the firing step of S-0310 claim 9. The shot itself delivers nothing to the target. The
    firer is an exploding object, so its death detonates a weapon at its own position, chosen by
    S-0312 claim 6: its type's death weapon, or else its current primary weapon, which may be the
    suicide weapon itself, or else the game-wide death weapon. That detonation is the only thing
    that harms the target, and its credit follows S-0302 claim 1 for death weapons. The firer's
    own death is credited to no one.
    - Backing: `measurable` (scenario T12)
    - Confidence: high

### Demolition charges

12. **Starting sabotage.** An infantry unit with explosive charges reaches a building through
    *sabotage*. S-0214 row I16 and S-0215 own when that behaviour starts and with which
    destination: the attack processing and the human order test whether the building's type
    accepts charges, and some computer-only starts do not. S-0215 claim 28 owns how *sabotage*
    is processed on the way.
    - Backing: `measurable` (scenario T13)
    - Confidence: medium

13. **Planting a charge.** A charge is planted at the end of a walking step (S-0208 claim 16)
    when the walker is in *sabotage*, its type has explosive charges, and its cell holds the
    building that is its destination. Only the type flag counts here: the veterancy
    explosive-charges ability gives no planting (quirk `quirks.charge_needs_type_flag`, Edge
    cases). On such an arrival the building's trigger tag first gets the entered-by event, with
    the walker as the cause (S-0011 claim 18). The outcome of the arrival is the one shown in the table below; if more than one building
    state applies, the earlier matching row governs:

    | Building state | Charge | Effects on the walker |
    |---|---|---|
    | being sold, or temporarily invulnerable | none | its movement is aborted; it stops being cloaked at once, without a sound; rearm and scatter as below |
    | a charge or delayed-kill countdown already running | none; the running countdown and its recorded planter stay as they are | its destination is cleared; rearm and scatter as below; no abort and no change to its cloak |
    | any other | planted (claim 14) | as in the first row |

    *Rearm.* Arrival is not a shot. In every row the rearm countdown is replaced by the
    second weapon's arrival delay below, using the walker's current burst counter *k*, not
    *k* + 1. Arrival neither advances nor wraps that counter, uses no ammunition, and creates
    no weapon effect. In particular, a fresh walker with *k* = 0 and a positive second-weapon
    burst count takes the short delay even when that count is 1.

    | First applicable condition | Arrival delay, integer ticks | Scenario-stream draws |
    |---|---|---|
    | no second weapon | 1 | none |
    | second weapon is sonic, or is spark-particle, fire-particle or railgun with an effect of its kind already alive on the walker | its rate-of-fire field, unchanged | none |
    | *k* is less than the second weapon's burst count | an inclusive ranged draw from 3 through 5 | one ranged draw |
    | otherwise | full delay of S-0301 claim 2, including the factors of claims 3, 6 and 7 | one inclusive ranged draw from 0 through 2 |

    These draws use S-0202's ranged-draw rule. The full delay has S-0301 Formula F1's
    floating-point domains and truncations, but no berserk halving: arrival does not apply
    S-0301 claim 8 to any row. No weapon-specific burst-delay field applies to this infantry
    walker. The delay's draw, when present, precedes draws caused by the subsequent scatter.
    A separate clearing of the walker's target still resets its burst counter under S-0301
    claim 4; that is not an advance caused by arrival.

    *Scatter.* The walker gets a scatter request whose threat point is the building's centre,
    forced and move-anywhere (S-0208 claims 11 and 12).

    The arrival tests neither whether the building's type accepts charges nor who owns the
    building.
    - Backing: `measurable` (scenario T13) for the arrival event, planting, refusals, cloak,
      rearm and scatter; `unmeasured` for the
      missing owner and accepts-charges tests at arrival, because no validated fixture reaches
      arrival with an otherwise refused destination. The computer-only starts of S-0215 describe
      the original only; our AI uses ordinary legal orders
    - Confidence: high for the three rows and the arrival delay

14. **The charge countdown.** Planting a charge does four things:
    - it marks the building as having a pending countdown,
    - the building flashes for *L* ticks (Formulas F4; claim 25 gives what the flash changes),
    - a countdown of *K* ticks starts in the planting tick (Formulas F4),
    - the walker is recorded as the building's charge planter.

    Expiry, its hit and its credit to the charge planter belong to S-0302 claim 16. A bridge repair
    hut instead demolishes its bridge (S-0511 claim 16). A change of the building's owner does not
    cancel the countdown. Curtain protection that starts during the countdown cancels it (S-0513
    claim 25). Curtain protection that is already running refuses the planting (claim 13, S-0513
    claim 27). On an undemolishable building (S-0302 claim 7) a charge still runs its countdown,
    and its expiry hit leaves the building at 1 health.
    - Backing: `measurable` (scenarios T13 and T14) for countdown, credit, ownership and
      protection; `unmeasured` for the undemolishable case: no validated normal-order arrival
      fixture is established
    - Confidence: high for the countdown length and the planter credit; medium for the
      undemolishable case

### Garrisons

15. **Special attacks and garrisoned buildings.** A bomb planted on a garrisoned building has
    the building as its carrier, and its detonation does not directly hit the occupants
    (S-0508 claim 16). Building-level evacuation can still follow damage or destruction
    (S-0508 claims 9 and 13, S-0312 claim 17). A charge can likewise destroy the building and
    cause building-level evacuation. A bomb on an infantry unit that then enters a garrison
    waits there (claim 6). No assaulter-type case is covered by the measurement scenario (S-0508 claim 14).
    - Backing: `measurable` (scenario T15)
    - Confidence: medium

### Air-strike designation

16. **The designation shot.** A designation shot is fired like any other shot (S-0301; S-0308
    row 6 chooses the weapon). Designation must be selected by S-0514 claim 20: takeover,
    planting, charging, parasite attachment, temporal erasure and locomotor replacement each
    suppress it when enabled, even when the higher-priority effect cannot succeed. Selected
    designation suppresses direct rocking, disarming, disguise-making and nuke-making even
    when no strike can be called. With designation selected, the detonation makes no area hits,
    as S-0303 claim 2 says of parasite projectiles. The detonation's other effects are not suppressed: radiation creation,
    the explosion animation, warhead debris and the light flash still occur where configured
    (S-0303 claims 2, 21 and 22). Instead of area damage, the strike is called (claim 17) or
    redirected (claim 19) when all of these hold:
    - the shot's firer still exists and is a designator,
    - the shot's target is an object: a unit, an infantry unit, an aircraft or a building,
    - the designator's strike is ready for that target (claim 18, Formulas F5).

    When the strike is idle it starts. When it is in progress it is redirected. Otherwise nothing
    happens. A shot at a cell, a shot whose firer has been destroyed, and a shot by a firer that
    is not a designator therefore call no strike. Neither the distance between projectile and
    target nor the target's owner is tested. The weapon's air-strike-cursor flag only affects the
    cursor (S-0214 row I18); it does not make a shot a designation shot.

    Designating therefore takes no time of its own: the strike starts or is redirected in the tick
    the designation shot detonates, after the firing cadence of S-0301 and the projectile's flight
    (S-0306). The designator need not keep firing once a strike has started, but any change of its
    target cancels the strike (claim 19).

    A departed superweapon warp member that is the shot's target, off the map or phasing in after it
    appears, is an ordinary object target: it is designated, by these tests (S-0306 claim 17).
    - Backing: `measurable` (scenarios T16 and T24; S-0306 scenario P11 run g, off the map, and run
      h, phasing in, for a departed warp member target)
    - Confidence: high; medium for a departed warp member target, as S-0306 claim 17 rates it

17. **Starting a strike.** At the start the designator's rank decides the strike size *n* and the
    strike aircraft type (normal or elite values, Terms). A successful start has these results:
    - **Aircraft.** *n* aircraft of the strike aircraft type are created one after another for the
      designator's house at that tick. Each is a visiting aircraft linked to the strike. No credits
      are charged, and no factory, prerequisite, build limit or production queue is involved.
    - **Arrival.** One arrival cell serves the whole strike. It is chosen once, before the first
      aircraft is created, exactly as S-0522 claims 4 and 5 choose a paradrop plane's arrival cell,
      with the same edge, the same candidates and the same draws. Each aircraft is then placed at
      the result of the nearby-cell search (S-0212 claim 25) centred on that arrival cell, with
      locomotion class *winged*, movement zone *normal*, no reach region, never in deck mode, no
      height condition, no further test, no no-overlay condition, bridge cells allowed and no
      reference cell. The search's tick number is the start tick. Placement is at that cell's
      centre, facing 0, under S-0501 claim 1. For class *winged* the search tests neither
      occupancy nor terrain, so every aircraft of one strike is placed at the same cell, which lies
      inside the usable area. A paradrop plane, by contrast, appears at its arrival cell outside it.
    - **Orders.** Each aircraft starts *attack*, with no destination. Its target is the cell that
      contains the designated object's reference point (S-0204 claim 4), each coordinate divided
      by 256 and rounded toward zero. So the aircraft attack that cell, not the object itself
      (S-0501 claims 11 to 17). Only a redirect (claim 19) aims them at an object.
    - **When at least one aircraft was placed:** the designated object becomes the strike
      target. The game-wide air-strike attack sound plays for every player at the position of the
      last aircraft placed. A designated building is marked. While marked it takes the
      appearance of claim 26, whose phase sequence makes one later scenario-stream draw.

    When no aircraft is placed, `quirks.airstrike_empty_strike_stuck` applies (Edge cases).
    - Backing: `measurable` (scenario T17), including a zero-size elite strike,
      `unmeasured` for creation/placement failures and the original's missing-type failure:
      no validated normal-input fixture safely isolates these paths. Our safe results in
      Edge cases are design departures, not oracle measurements
    - Confidence: high; medium for the shared placement cell, which follows from S-0212 claim 25

18. **When a strike is ready.** A designation shot at target *T* in tick *t* finds the strike
    ready (Formulas F5) when either:
    - *T* is not the strike target, which includes every case in which the strike holds no target,
      or
    - the strike is idle and *t* ≥ *e* + *R*, where *e* is the end tick of its last strike
      (claim 20) and *R* is the recharge delay for the designator's rank in tick *t*.

    Consequences:
    - a designator's first designation is always ready, because its strike holds no target yet.
    - While a strike is in progress, further shots at its strike target do nothing. The designator
      may go on firing at it.
    - A strike that ends normally keeps its strike target (claim 20), so designating the same
      object again waits until *e* + *R*.
    - A cancel (claim 19) forgets the strike target. After a cancel, an idle strike is therefore
      ready at once for any object, including the one it held before. The recharge delay only
      ever holds back a repeat designation of the same object with no change of target in between.
      Quirk `quirks.airstrike_recharge_same_target_only` (default on, the original). Off: an idle
      strike that has ended before is ready only when *t* ≥ *e* + *R*, whatever its target; while
      it is in progress, readiness is unchanged.
    - Backing: `measurable` (scenario T18)
    - Confidence: high

19. **Cancel and redirect.** A strike is *cancelled* when any of these happens:
    - the designator's target changes to anything else, including to none. So a move order, an
      attack order on another object or the loss of its target (for example because the target is
      destroyed or removed from play) cancels. Setting the same target again does not. This
      applies whether or not the strike is in progress.
    - its marked building is destroyed,
    - the designator is destroyed or removed from play (claim 20).

    Cancellation takes effect when the change of target applies, not at the strike's next
    look, the designator's next turn or the next designation impact. A change from object A
    to B and back to A within one tick still cancels; restoring A does not undo the retreat
    orders, ammunition loss or forgotten strike target. A change after the designator's turn
    likewise cancels in that same tick. Merely selecting an order before it applies is not a
    target change (S-0213 claim 3). No target-history polling rule substitutes for these effects.

    A cancel has these results in the tick of the cancelling event:
    - if any linked aircraft had no ammunition at cancellation, every linked aircraft now has
      0 rounds; otherwise each keeps its ammunition,
    - the most recently placed linked aircraft, up to 101 of them, have no target or destination
      and have queued *retreat*. What follows is S-0501's (claims 18 and 34). A strike aircraft is not exempt from
      falling back to rest while in *retreat* (S-0501 table T2). The aircraft stay linked, so the
      strike stays in progress until none is linked any more (claim 20).
    - the strike forgets its strike target. Its building mark passes to another strike whose
      strike target is the same building, if there is one. Otherwise the mark is removed.

    A ready designation shot at *T* while the strike is in progress *redirects* it. Because a
    change of target cancels first, this happens after a cancel. When the strike aircraft do not
    all hold the same amount of ammunition, the strike is cancelled again instead, and nothing is
    redirected. Otherwise:
    - *T* becomes the strike target, and a building *T* is marked as at a start. Its phase
      sequence restarts (claim 26),
    - every strike aircraft that still has ammunition takes *T* itself, the object rather than a
      cell, as its target and queues *attack*,
    - aircraft without ammunition keep retreating.

    So once any strike aircraft has spent its last round, changing target leaves all linked
    aircraft without ammunition: a later redirect cannot give any of them an attack order.
    A redirected building shows the same laser-target colour as one marked by a start (claim 26).
    - Backing: `measurable` (scenario T19), except removal of a live designator without
      death, which is `unmeasured` for the fixture limitation in claim 20
    - Confidence: high for the cancel triggers and effects; medium for the redirect

20. **End of a strike, loss of the designator, ownership and credit.**
    - A strike aircraft stops being linked when it is destroyed or removed from play, for example
      when it leaves the map (S-0501 claims 10 and 34). When the last one stops being linked, the
      strike becomes idle and its end tick *e* is that tick. It keeps its strike target (claim 18).
    - When the designator is destroyed (from its killing hit) or removed from play, the strike
      ends with it. If at that moment the strike is *active* and has aircraft, and the designator's
      house is the local player, the game-wide air-strike abort sound plays at the position of the
      most recently placed aircraft that is still linked. A strike is active from a start that
      placed aircraft, or a redirect that re-aimed at least one aircraft, until its next cancel or end. Then the strike is cancelled
      (claim 19), and all its aircraft stop being linked. From then on they are ordinary visiting
      aircraft.
    - The strike aircraft belong to the designator's house at the start tick. A later change of
      the designator's owner moves neither the strike nor its aircraft to another house, and the
      strike stays the designator's.
    - Hits by strike aircraft are credited to the aircraft and their house (S-0302 claim 1).
      The designator gets no kill credit and no experience from those aircraft hits.
      The designation shot itself makes no area hits; separately configured effects remain
      subject to claim 16 and their owning specs.
    - Backing: `measurable` (scenario T20) for death, normal end, ownership and credit,
      `unmeasured` for a designator removed without a killing hit: no validated normal-input
      fixture removes a live stock designator permanently without first killing it
    - Confidence: high; medium for credit after the designator's owner changes

21. **Random draws of a strike start.** On the scenario stream (S-0202 claim 10), in order: the
    arrival cell's draws (S-0522 claim 4), once per strike; then, for each aircraft in creation
    order, its creation draws and any draws of its placement (S-0501). The readiness test, the
    nearby-cell search, a cancel and a redirect take no draw of their own. A start or redirect
    that marks a building leads to one later draw in that building's turn, about 10 ticks
    later, from the mark's phase sequence (claim 26). The draws of the designation shot's
    own detonation are S-0303's.
    - Backing: `measurable` (scenarios T17, T21 and T27 with the random-state recorder)
    - Confidence: high

22. **Bomb with no bomb warhead.** It ends normally, stops ticking and detaches from its
    carrier. It makes no area hits and no explosion animation, not even a generic fallback
    animation. A bridge repair hut still demolishes its bridge by claim 5: that effect does
    not depend on the warhead. Off-map carriers retain claim 6's silent removal.
    - Backing: `measurable` (scenario T22)
    - Confidence: high

23. **Suicide with no demolition warhead.** The self-hit of claim 11 is still delivered.
    Its positive damage bypasses defences and reduces an ordinary firer whose current health
    is no greater than its type's strength to zero even without a warhead. No missing-warhead
    rule makes that self-hit harmless. Completion of death is a different matter: the original
    can fail while choosing an infantry death effect, so a completed suicide explosion cannot
    be promised for this bad data.

    Our engine completes the death deterministically, with ordinary death presentation and
    no warhead-selected special death effect. Its death weapon still detonates under S-0312
    claim 6, provided that weapon has its own valid warhead. The importer warns when a suicide
    weapon is used without a game-wide demolition warhead. This is a bad-data design departure,
    with no quirk switch, not a replacement for the suicide self-hit.
    - Backing: `unmeasured` for the original's complete bad-data death outcome: no validated
      fixture safely completes all death paths; scenario T23 proposes the discriminating run.
      The safe completion rule is `design` and tested on our engine by T23
    - Confidence: high for the self-hit and the unsafe infantry death case

### Presentation of bombs, charges and marks

Claims 24 to 26 describe drawing. They are presentation, not simulation, except the mark's phase
sequence and its draw in claim 26, which are game state.

24. **Bomb icon drawing (presentation; Q-1524 question 1).** When claim 10 lets a bomb icon show,
    it is drawn as follows.
    - **Image and palette.** The bomb icon image file (Import mapping), at the image frame of
      Formulas F3, through the fixed cursor palette (Import mapping). It has no house-colour
      remapping and no translucency.
    - **Brightness.** Neutral, 1000 ‰. Cell light, flashes and effect tints do not change it. Only
      the light-alpha shading of S-0111 claim 2 darkens it.
    - **Placement.** Centred placement (S-0102 claim 19) on the carrier's own draw point, with no
      further displacement. For a vehicle, infantry unit or aircraft this is the screen pixel of its
      position by S-0101's projection, height included, which is also its status anchor (S-0111
      Terms). For a building it is the projected top corner of its foundation, the image anchor of
      S-0108 claim 8. So on a building the icon sits at the top corner, not over the building's
      centre.
    - **Layer and order.** The icon is one of the carrier's status extras (S-0102 claim 1). It is
      drawn after all five object layers, above every object and below the effects of S-0102 claim 2,
      with no depth test. It is the first of its carrier's status extras, so that carrier's own repair
      wrench, health bar, selection marker, pips and insignia paint over it where they overlap.
      Status extras are drawn carrier by carrier, layer by layer in the order of S-0102 claim 1, and
      within a layer in that layer's drawing order (S-0102 claims 6 and 7). A carrier whose extras
      come later therefore paints its icon over an earlier carrier's icon and extras. The icon is
      drawn only when the carrier's status extras pass the culling of S-0111 claim 1.
    - **Houses.** Every viewer who sees the bomb (claim 10) sees the same picture.
    - Backing: `measurable` (scenario T25(a–g)), except light-alpha darkening when T25(g)
      cannot expose an eligible icon pixel under a shroud piece: that part is `unmeasured`
      until such a placement is validated. The missing-asset rule (Import mapping) is
      importer design, tested on our engine only in T25(h), not an oracle claim
    - Confidence: high for image, palette, brightness, placement and first place among the carrier's
      extras; medium for the order between carriers

25. **Charge flash (presentation; Q-1524 question 2).** The flash of claim 14 changes only the
    charged building's brightness, two ticks on and two ticks off.
    - **Count.** Planting sets the building's flash count *c* to *L* (Formulas F4). Each turn of the
      building lowers a positive *c* by 1. A later flash of the same building, from any cause,
      replaces the count; counts do not add.
    - **Pattern.** In a drawn frame the flash is *on* when *c* mod 4 is 2 or 3, and off otherwise
      (Formulas F6). With one building turn per tick it alternates two ticks on and two ticks
      off, starting from the phase that *L* gives, until *c* reaches 0. For *L* = 13 it is on
      after the building's 2nd, 3rd, 6th, 7th, 10th and 11th turns following the planting.
    - **What changes.** While on, the brightness *b* of S-0104 claim 13 is raised by 500 ‰, or
      lowered by 500 ‰ when *b* exceeds 1500 ‰ (F6). This applies to the body however it is drawn,
      to the building's parts drawn as voxel models (turret, barrel) and to its own animations that
      take the building's stored brightness (S-0104 claim 13, row 3). The stored brightness is
      refreshed at each on/off change. The building's shadow keeps its ordinary darkening. There is
      no colour change and no other palette. The damage state still picks the image frames
      (S-0108); the flash brightens whichever frame shows. The normal-light override for animations
      still wins (S-0104 claim 13).
    - **Repaint.** Each on/off change repaints the building's area of the persistent picture
      (S-0102 claim 8), so the body shows every change at once.
    - **Combination.** The flash adjustment comes first. Protection (S-0513 claim 29) and a mark
      (claim 26) then act on the flashed value (F7). Curtain protection that cancels the countdown
      (claim 14) does not stop the flash.
    - **Houses.** The flash looks the same to every viewer.
    - Backing: `measurable` (scenario T26; T27(c) for combined brightness and the animation's
      normal-light override). Negative-input rounding in F7 is `unmeasured`: these scenarios
      use non-negative brightness and do not isolate the signed division
    - Confidence: high for the count, the pattern and the 500 ‰ step; medium for the list of parts
      and for the repaint

26. **Air-strike mark appearance and its phase draw (Q-1524 questions 3 and 4).** A marked
    building is drawn with a colour mask and a pulsing brightness. The pulse follows a phase
    sequence that advances during the building's own turns. The sequence is game state, because one
    phase length takes a scenario-stream draw. The colour and the brightness are presentation.
    - **Colour.** The entry of the colour-adjustment table (S-0006) that the laser-target colour
      selects gives a mask by S-0102 F9. While the building is marked, in every drawn frame, the mask
      is combined by the bitwise union of S-0102 claim 34 with the building's body drawn as a sprite
      (in either pass), its voxel turret and barrel, and building animations at its cells (S-0102
      claim 34's animation rule: the building occupying the animation's cell decides). For the
      body, voxel turret and barrel this covers a viewer to whom the building's centre cell is not
      shrouded; for any other viewer it is open (Houses, below). While the
      cell holding an animation's own position is under shroud for the local player, that
      animation takes no marking colour and no force-shield mask, whatever the state of the
      building's centre cell (S-0513 claim 29b; pending S-0513 SW10 part i5, and `unmeasured`
      for a building that is only marked). A building
      that is also force-shielded takes the union of both masks. Curtain protection adds no mask. The
      mask itself never pulses. It is read from the rules at drawing time, so a building marked by a
      redirect looks the same as one marked by a start. This covers ordinary opaque drawings.
      Translucent and special accessory drawings remain outside this claim, as in S-0102 claim 34.
    - **Selector outside the table (Q-1621).** The laser-target selector is a zero-based signed
      integer naming one of the 16 positions of the colour-adjustment table: position *k*, from 0
      to 15, is entry *k* + 1 in S-0006 claim 2's numbering. The
      original does not reduce it modulo 16 and does not clamp it. This half is carried over by
      analogy from what S-0513 claim 29a says of the protection selectors, a different claim
      with its own rating and scenario; here it is medium, pending T27 part e. This spec adds that it does not fall back to the absent-key
      default and does not keep the earlier layer's selector either (medium, pending T27 part e).
      So 16 does not select position 0 and −1 does not select position 15. What any other
      selector outside the table shows is open (Open questions). The values T27 part e uses, −1 and 16, name no
      table position; what the original's marked building then shows is open
      (Open questions). A selector far outside the table may make the original fail
      (`unmeasured`: no run uses such a value). It is bad data.
      **Our engine (bad data; a departure, no switch).** After every rules layer has been
      applied, a held laser-target selector below 0 or above 15 counts as no selected
      adjustment: every drawing that would take the laser-target mask takes *K* = 0 (S-0102 F9)
      on 5-6-5 and 5-5-5 surfaces alike, so a marked building's body, voxel turret and barrel
      and building animations at its cells take no laser-target colour. A force-shielded marked
      building still takes the force-shield mask, unchanged. The importer warns once, naming the
      key and its value (glossary, *importer warning*). The pulse of the phases below, the phase
      draw and every other part of the mark are the same as for a valid selector. A selector
      from 0 to 15 behaves as above, including a position whose triple is the empty (0, 0, 0)
      entry, which also gives *K* = 0.
    - **Restart.** A start or redirect that marks the building (claims 17 and 19) puts its phase
      sequence back to *before the first marked turn* and starts its mark clock (Terms). A mark that
      passes to another strike holding the same building (claim 19) restarts neither. At any turn of
      the building while it is not marked, the sequence returns to *before the first marked turn*,
      without a draw.
    - **Phases.** Let *q* be the building's first turn while marked after a restart, and *M* the mark
      clock's remaining ticks. A phase of length ℓ entered in tick *s* has *r* = max(0, ℓ − (*n* − *s*))
      in tick *n*. A phase ends at the first turn of the building at which *r* = 0, and the next phase
      starts in that turn. Phases advance only at the building's turns, one phase per turn. The
      tests on *M* are made only at the transitions shown.

      | Phase | Length | Factor *m* | Next phase |
      |---|---|---|---|
      | Before the first marked turn | until *q* | none (brightness unchanged) | Initial rise |
      | Initial rise | 6 ticks, from *q* | trunc((12 − *r*) × 256 / 6) | Bright hold |
      | Bright hold | 4 ticks | 512 | Initial dimming |
      | Initial dimming | 20 + R(−5, 5) ticks | trunc((*r* + 20) × 256 / 20) | Pulse rise |
      | Pulse rise | 64 ticks | 4 × (128 − *r*) | Pulse fall |
      | Pulse fall | 64 ticks | 4 × (*r* + 64) | Low hold if *M* < 158 ticks, otherwise Pulse rise |
      | Low hold | until a later turn finds *M* < 31 ticks | 256 | Final rise |
      | Final rise | 6 ticks | trunc((12 − *r*) × 256 / 6) | Final bright hold |
      | Final bright hold | 4 ticks | 512 | Return |
      | Return | 20 ticks | trunc((*r* + 20) × 256 / 20) | Ordinary |
      | Ordinary | until unmarked or restarted | none (brightness unchanged) | none |

      All arithmetic is integer, divisions truncated toward zero. So the brightness doubles over
      6 ticks, holds 4 ticks, falls back over 15 to 25 ticks (from between 448/256 and 576/256
      of ordinary), and then pulses as a 128-tick triangle between ordinary and double brightness. Low
      hold is reachable only about 99 842 ticks after the mark, so in practice the pulse lasts as long
      as the mark.
    - **Draw.** Each restart that reaches Initial dimming takes exactly one inclusive ranged draw
      from −5 through 5 on the scenario stream (S-0202 claim 10, ranged-draw rule), in the building's
      turn that ends Bright hold: turn *q* + 10 with a turn every tick. A mark that ends or restarts
      before that turn makes no draw for that restart. Within one turn of the building the protection
      sequence of S-0513 claim 29 steps first, so its draw precedes the mark's when both fall in that
      turn.
    - **Brightness.** F7 combines the flash, protection and mark. The mark factor reaches the same
      parts as the flash (claim 25): the body, its voxel parts and its animations with stored
      brightness. The stored brightness is refreshed at each building turn. The animation's
      normal-light override still wins (S-0104 claim 13). Shadows keep their ordinary darkening.
    - **Every frame (design).** Our engine applies the current factor in each drawn frame,
      including to the persistent body picture. Whether the original's persistent body picture
      shows each pulse value or retains the value from its last repaint is unmeasured; T27(b)
      investigates this distinction. The per-frame rule is a presentation choice, not a confirmed
      original result, and introduces no quirk switch.
    - **Houses.** Colour and pulse are the same for every owner, and for every viewer except
      through the shroud exception for building animations above and the open body case below: shroud is kept per viewer, so
      an animation whose own cell is shrouded for one viewer and not for another takes the
      marking colour for the second viewer only. This follows from per-viewer shroud and is
      `unmeasured`: every cited scenario uses one viewer. Whether a marked building's body also loses
      the marking colour for a viewer to whom its centre cell is shrouded, as a protected
      object's body loses its mask (S-0513 claim 29b), is open (Open questions).
    - Backing: `measurable` (scenario T27; the draw with the random-state recorder) for the
      colour, initial phases, repeating pulse, restart and shared-mark behaviour. The original's
      persistent-body refresh is `unmeasured`: no validated capture isolates pulse changes from
      unrelated repaints. For a selector outside the table: `measurable` that the original
      neither wraps nor clamps it, nor rejects it in favour of the absent-key default or of the
      earlier layer's value (T27 part e, which would show that the colour does not follow table
      positions 0, 15 and the earlier layer's position); what the original then shows is open
      (Open questions); the possible failure with a far-out selector is `unmeasured`, since no
      run uses such a value; our engine's rule is a departure by decision, checked on our
      engine by T27 part e's engine fixture. The shroud exception for building animations is `measurable` (S-0513 scenario
      SW10 part i5) for a building that is both force-shielded and marked, the only case that
      part runs; for a building that is only marked it rests on that combined run and is
      `unmeasured`. T27(b) is an investigation, and separately tests our per-frame design.
      The mark-clock thresholds and phases from Low hold onwards are `unmeasured`: no validated
      fixture keeps a mark for nearly 100 000 ticks without ending or restarting it. Missed-turn
      phase advancement is likewise `unmeasured`: T27 does not suspend building turns.
      Negative-input rounding in F7 is `unmeasured` for the fixture limitation in claim 25.
      Per-frame rendering is a design rule, tested on our engine in T27(b), not an oracle claim
    - Confidence: high for the colour source (except whether a body whose centre cell is
      shrouded for the viewer keeps the marking colour, which is open), the union with the force-shield mask, the phase lengths,
      factors and the draw; medium for the list of parts, for the shroud exception for building
      animations on a building that is both force-shielded and marked (pending S-0513 SW10 part
      i5's combined run); medium for that exception on a building that is only
      marked, which no run covers, and for its consequence for two viewers with different
      shroud; medium for the draw's place among other draws
      in the same building turn. For a selector outside the table: medium that the original
      neither wraps nor clamps it (by analogy with S-0513 claim 29a), until T27 part e has run; medium that it
      neither falls back to the absent-key default nor keeps the earlier layer's value (this
      spec's own, pending T27 part e); low for the possible failure with a far-out selector
      (`unmeasured`). Our engine's rule for such a selector is
      a departure by decision: backing `unmeasured` on the original, checked on our engine by
      T27 part e's engine fixture; confidence high

## Formulas

**F1. Timed-bomb detonation tick** (claim 4). *t*: the plant tick; *D*: the timed delay
(integer, ticks). The bomb goes off at the first turn of its carrier, taken on the map, in a tick
*n* with

> *n* > *t* + *D*, exact integer comparison. With a carrier turn every tick, *n* = *t* + *D* + 1.

**F2. Detection distance** (claim 10). *Δ* = (*Δx*, *Δy*, *Δz*) is the difference between the
detector's and the carrier's positions in leptons (integers); *s* is the detector type's bomb-sight
range (integer cells).

> *N* = *Δx*² + *Δy*² + *Δz*², in square leptons.
> *d* = trunc(*r*(*N*)) leptons. The detector sees the bomb when *d* < 256 *s*.

The positions are the reference points of S-0204 claim 4. The sum uses binary64 floating-point
products and additions under S-0205; it is exact for integer *N* below 2⁵³. The root *r* is the
table square root of S-0204 claim 5 and Formulas F1, including `quirks.table_root_distance` and
its exact-root alternative. Conversion to integer leptons truncates toward zero. There is no
building-size subtraction in this detector test.

**F3. Bomb icon frame** (claim 10). *n*: current tick; *t*: plant tick; *D*: timed delay; *F*: the
flicker rate (integer ticks). Let *q* = ⌊*D* / 6⌋. Define *b* by the following cases:

> *b* = 1 when (*n* mod 2*F*) ≥ *F*; otherwise *b* = 0.

> frame = min(11, 2 ⌊(*n* − *t*) / *q*⌋ + *b*). Integer arithmetic, divisions truncated toward
> zero. Frames are counted from 0.

For nonnegative *D* and positive *F*, these divisions have the stated frame meaning. When
*q* = 0, our engine substitutes *q* = 1; independently, when *F* = 0 it uses *b* = 0. The
importer warns for either substitution (bad-data departure, no switch). The original divides
by zero on these inputs while drawing. Negative delays and flicker rates are not specified.

**F4. Charge countdown and flash** (claim 14). *c*: the imported charge delay in minutes. It is
read as a single-precision value under S-0003 and widened exactly to binary64. A value written
with a percent sign is then multiplied by 0.01 in binary64 (S-0003). When the key is absent, *c* is
the binary64 value nearest 0.03, which never passes through single precision.

> *K* = trunc(RZ₅₃(*c* × 900)) ticks.
> *L* = trunc(RZ₅₃(*c* × 900) × 0.5) ticks. Halving is exact.

RZ₅₃ is the binary64 product rounded toward zero, which depends on `quirks.chop_rounding`
(S-0205). With the switch off, RZ₅₃ is RN₅₃. The truncations are toward zero. With the key
absent, *K* = 26 under RZ₅₃ and 27 under RN₅₃, and *L* = 13 in both modes. A key written as 0.03
gives the single 0.02999999932944774627685546875, so *K* = 26 and *L* = 13 in both modes.

**F5. Strike readiness** (claim 18). *t*: the tick of the designation shot's detonation; *T*: its
target; *S*: the strike target, or none; *P*: whether the strike is in progress; *e*: its end
tick; *R*: the recharge delay for the designator's rank at *t* (integer ticks).

> ready = (*T* ≠ *S*) or (not *P* and *t* ≥ *e* + *R*). Exact integer comparison.

With `quirks.airstrike_recharge_same_target_only` off, and when the strike has ended at least
once: ready = (not *P* and *t* ≥ *e* + *R*) or (*P* and *T* ≠ *S*).

**F6. Charge flash** (claim 25). *c*: the building's flash count (integer, ticks, *c* ≥ 0); *b*: its
brightness of S-0104 claim 13 (integer ‰).

> on = (*c* mod 4 ≥ 2).
> *b*<sub>F</sub> = *b* + 500 when on and *b* ≤ 1500; *b* − 500 when on and *b* > 1500; *b* when off.

Integer arithmetic, no clamp. Each building turn with *c* > 0 sets *c* to *c* − 1.

**F7. Building brightness with flash, protection and mark** (claims 25 and 26). Let
*b*<sub>F</sub> be F6's result in integer ‰, *P* the protection-brightness function of S-0513
claim 29 and F9, and *m* the dimensionless integer mark factor of claim 26. *P* is the identity
when unprotected or outside its active phases. Define *b*₁ = *b*<sub>F</sub> and *b*₂ = *P*(*b*₁).
The following mutually exclusive cases define *b*₃ mathematically:

| Building state | *b*₃, integer ‰ |
|---|---|
| Marked in a phase with a factor | min(2000, ⌊*m* × *b*₂ / 256⌋) |
| Marked in a phase without a factor, or unmarked but protected | *b*₂ |
| Unmarked and unprotected | *b*₁ |

*b*₃ goes to S-0104 claim 15. Integer arithmetic. For *b*₂ ≥ 0 the division truncates; for a
negative *b*₂ the original rounds toward minus infinity, which S-0104 claim 15 maps to row 0 either
way. The two phases without a factor are not clamped. The same *b*₃ is the stored brightness of the
building's animations of S-0104 claim 13 row 3, refreshed as claims 25 and 26 say.
The normal-light override still forces their drawn brightness to 1000 ‰.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Ticks per minute of charge delay | 900 | ticks per minute | F4 |
| Bomb visibility period | 46 | ticks | Claim 10 |
| Visibility bound after planting | 2 | ticks | Claim 10 |
| Detection scale | 256 | leptons per cell | F2 |
| Highest bomb icon frame | 11 | frame | F3 |
| Icon frame divisor of the delay | 6 | — | F3 |
| Timed delay, absent-key default | 450 | ticks | Engine default |
| Bomb damage, absent-key default | 100 | health points | Engine default |
| Flicker rate, absent-key default | 8 | ticks | Engine default |
| Charge delay, absent-key default | 0.03 | minutes | Engine default, kept in binary64 (F4) |
| Strike aircraft ordered to retreat at a cancel, at most | 101 | aircraft | Claim 19 |
| Strike target cell divisor | 256 | leptons per cell | Claim 17 |
| Air-strike team size, absent-key default | 0 | aircraft | Engine default: not a designator |
| Bomb icon brightness | 1000 | ‰ | Claim 24 |
| Flash step | 500 | ‰ | F6 |
| Flash step-down threshold | 1500 | ‰ | F6: above it the flash darkens |
| Flash pattern period | 4 | ticks (2 on, 2 off) | F6 |
| Mark clock | 100 000 | ticks | Claim 26 |
| Mark phase lengths | 6, 4, 20, 64, 64, 6, 4, 20 | ticks | Claim 26, in table order |
| Mark dimming draw range | −5 to 5 | ticks, inclusive | Claim 26 |
| Mark low-hold and final thresholds | 158, 31 | ticks of the mark clock | Claim 26 |
| Mark factor scale | 256 | factor units per 1× brightness | Claim 26, F7 |
| Brightness cap | 2000 | ‰ | F7 |
| Recharge delay, absent-key default | 0 | ticks | Engine default |

## State and ordering

**Planted bomb.** Projectile planting and disarming events below require selection under
S-0514 claim 20, as specified in claims 2 and 7. Ownership-change and removal disarms do not.

| State | Event | Result |
|---|---|---|
| none | bomb-planting projectile detonates on a valid target (claim 2) | attached, timer from *t* |
| attached | carrier's turn on the map, tick > *t* + *D* | detonation (claim 5) |
| attached | carrier off the map | timer cannot detonate it; no ticking sound |
| attached | detonate order, carrier on the map | detonation (claim 5) |
| attached | detonate order or death, carrier off the map | removed, no effect (claim 6) |
| attached | carrier dies on the map from a killing hit | the bomb detonates once at the carrier's position; it does not detonate a second time as part of the carrier's death (S-0312 claim 7) |
| attached | aircraft carrier destroyed on the map without a hit (S-0501 claim 46's fall-back crash) | unchanged: no detonation; during the fall the timer and detonate-order rows above still apply |
| attached | that carrier's impact | removed, no effect (claim 7) |
| attached | disarm projectile detonates on the carrier | removed, no effect (claim 7) |
| attached | non-garrisonable building changes owner | removed, no effect |
| attached | carrier removed other than by death | removed, no effect |
| attached | unit or aircraft changes owner | unchanged |
| attached | planter removed | unchanged; no attacker at detonation |

**Charge on a building.**

| State | Event | Result |
|---|---|---|
| none | charge arrival, building neither being sold nor protected | countdown *K*, planter recorded |
| none | charge arrival, building being sold or protected | refused, walker scatters |
| running | charge arrival | refused, countdown and planter unchanged |
| running | owner change | unchanged |
| running | curtain protection starts | cancelled (S-0513 claim 25) |
| running | expiry | S-0302 claim 16 |

**Strike.** Designation-shot events below require designation to be selected under S-0514
claim 20 and delivery to qualify under claim 16.

| State | Event | Result |
|---|---|---|
| idle, no strike target | designation shot at an object | start (claim 17); in progress, target held |
| idle, strike target *S* | designation shot at *S*, before *e* + *R* | nothing |
| idle, strike target *S* | designation shot at *S* from *e* + *R*, or at another object | start |
| in progress, strike target *S* | designation shot at *S* | nothing |
| in progress, any | designator's target changes; marked building destroyed | cancel: aircraft retreat, target forgotten, still in progress |
| in progress, no strike target | designation shot at an object, all aircraft with equal ammunition | redirect: aircraft with ammunition attack the object |
| in progress, no strike target | designation shot, aircraft ammunition unequal | cancelled again; nothing redirected |
| in progress | last strike aircraft destroyed or removed | idle, *e* = this tick, strike target kept |
| idle, any | designator's target changes | target forgotten |
| any | designator destroyed or removed from play | abort sound if active (claim 20); cancel; aircraft unlinked; strike gone |

**Ordering.** A suicide firer's hit and its death-weapon detonation both occur in the tick of the shot.
When a planted bomb's detonation kills its carrier, the bomb does not detonate a second time.
The ticking sound stops in the same tick as the bomb ends.
A marked building's phase sequence (claim 26) steps during the building's own turn, right after
the protection sequence of S-0513 claim 29. Its one draw falls in the turn that ends Bright hold,
after any protection draw of the same turn. A charged building's flash count drops by 1 at each
of its turns (claim 25).

**Cross-spec status.** S-0203 claim 1 and its ordering-table rows 2 and 5 distinguish
planted-bomb maintenance from timed detonation during the carrier's turn, with the removal
consequences of S-0203 claim 10. S-0506 claim 26 likewise places its shared cycle before object
turns, rather than after timed bomb detonations. No corresponding correction is needed.

**Presentation cross-spec changes for joint hand-off.** S-0111 claims 1 and 19 must exempt the
bomb icon from the building status anchor and refer to claim 24 for its image and order.
S-0102 claim 34 must include the marked-building mask and its union with the force-shield mask
(claim 26). S-0104 claim 13 must delegate building flash and mark brightness to claims 25 and 26
and F7, while retaining unresolved non-building flash cases. S-0513 claim 29 must refer to F7
for combinations and state that a same-turn protection draw precedes the mark draw. These are
additions to the earlier isolated-protection contracts, not changes to protection alone.

## Edge cases and quirks

- **Veterancy charges plant nothing.** A unit that has explosive charges only through its veterancy
  ability is offered the sabotage order (S-0214 row I16) and can start *sabotage* (S-0215). At the
  arrival of claim 13, though, only the type flag counts, so it plants no charge. What it does
  next at the building is an open question. Quirk `quirks.charge_needs_type_flag` (default on, the
  original). Off: the veterancy ability also plants, as the type flag does. No stock type has the
  ability.
- **A suicide firer that survives its own hit.** The hit of claim 11 ignores defences, so curtain
  protection and type immunity do not stop it. It is still stopped by the rows of S-0302 claim 8
  that apply to hits that ignore defences, for example a demolition warhead that makes targets go
  berserk or does psychic damage to an immune type. A firer that survives keeps no rearm
  countdown, so it may shoot again whenever the firing check allows. This is
  the original's behaviour with such data; no switch is proposed.
- **No bomb warhead or no demolition warhead.** Both game-wide warheads default to none.
  Claims 22 and 23 distinguish harmless bomb expiry from the unsafe suicide-death case.
- **Overdue bombs after transport.** A bomb that waited off the map goes off at the carrier's
  first turn after it comes back, however long ago its time ran out (claim 6).
- **A strike that places no aircraft.** When a start places no aircraft, because the elite strike
  size is 0 or negative for an elite designator, or because the first aircraft cannot be created, the
  original still leaves the strike in progress, and no aircraft can ever end it. (A normal strike
  size of 0 or less makes no designator at all, Terms, so only the elite size matters here.)
  Later designation
  shots at another object only set and mark a strike target (a redirect with no aircraft), and
  the designator never calls aircraft again. Quirk `quirks.airstrike_empty_strike_stuck` (default
  on, the original). Off: a start that places no aircraft leaves the strike idle, with its strike
  target and end tick unchanged. Stock data has no designator whose elite strike size is 0.
- **Failed aircraft placement.** When an aircraft is created but cannot be placed, it is
  discarded and no further aircraft of that strike are created; the same holds when an aircraft
  cannot be created at all. After a failed placement the original counts one aircraft fewer than
  it placed, so the strike can become idle while one of its aircraft still flies. Our engine
  counts the aircraft actually placed (documented departure, design). No normal-input fixture for
  a failed placement is established.
- **No strike aircraft type.** A type with a positive strike size and no normal strike aircraft
  type stops the original at the first start. Our engine treats such a type as no designator, and
  the importer warns (bad-data departure, no switch). A missing elite type takes the normal type
  (Import mapping).
- **Out-of-range laser-target selector** (claim 26): no laser-target colour and an importer
  warning, a departure for bad data whose original colour is not established (Open
  questions), matching S-0513's out-of-range protection selector.
- **Large strikes.** A cancel orders at most 101 strike aircraft to retreat, taking the most
  recently placed first. Any further ones keep their orders. No stock strike comes near this.
- **Several strikes on one building.** Each strike marks its own strike target. When one is
  cancelled, the mark passes to another strike holding the same building, as claim 19 says.
- **Icon frames with a short delay.** A timed delay below 6 ticks or a flicker rate of 0 makes the
  original divide by zero while drawing. F3 gives our fixed form.

## Import mapping

| Source | Concept | Form | Absent |
|---|---|---|---|
| Warhead `IvanBomb` | bomb-planting warhead (claims 2, 3) | boolean (S-0003) | no |
| Warhead `BombDisarm` | disarm warhead (claims 3, 7) | boolean (S-0003) | no |
| Infantry type `Ivan` | bomb planter, cursor only (S-0214 row I19) | boolean (S-0003) | no |
| Infantry type `Doggie` | animal marker, not the source of the latching attack (claim 1) | boolean (S-0003) | no |
| Object types `Bombable` | accepts bombs, cursor only (S-0214 row I19) | boolean (S-0003) | yes |
| Unit, infantry, aircraft and building types `BombSight` | bomb-sight range (claim 10) | integer, cells | 0 |
| Weapon `Suicide` | suicide weapon (claim 11; S-0312 owns the row) | boolean (S-0003) | no |
| Infantry type `C4` | has explosive charges (claim 13; S-0214 owns the row) | boolean (S-0003) | no |
| Infantry type `Secondary`, `EliteSecondary` | second weapon for arrival delay (claim 13) | weapon name; rank selection by S-0308 claim 2 | S-0308 claim 2 owns fallback; unresolved weapon gives 1 tick |
| Weapon `ROF`, `Burst` | arrival delay's rate-of-fire field and burst count (claim 13) | integers, ticks and shots respectively | 0 ticks and 1 shot, owned by S-0301 |
| Weapon `IsSonic`, `UseSparkParticles`, `UseFireParticles`, `IsRailgun` | arrival bare-rate eligibility (claim 13) | booleans | no, owned by S-0301 |
| Building type `CanC4` | accepts charges: not tested at arrival (claim 13; S-0214 and S-0302 own the row) | boolean (S-0003) | yes |
| Rules, combat-damage section, `IvanWarhead` | bomb warhead (claims 5, 22) | warhead name | none: no area hits or explosion animation; hut demolition remains |
| Rules, combat-damage section, `IvanDamage` | bomb damage (claim 5) | integer, health points | 100 |
| Rules, combat-damage section, `IvanTimedDelay` | timed delay *D* (claims 4, 10) | integer, ticks | 450 |
| Rules, combat-damage section, `IvanIconFlickerRate` | flicker rate *F* (F3) | integer, ticks | 8 |
| Rules, combat-damage section, `CanDetonateTimeBomb` | timed-bomb detonate permission (S-0214 row G3, which owns the row) | boolean (S-0003) | no |
| Rules, combat-damage section, `CanDetonateDeathBomb` | no reachable effect (claim 9) | boolean (S-0003) | no |
| Rules, combat-damage section, `C4Warhead` | demolition warhead (claims 11, 14, 23; S-0011 owns the row) | warhead name | none; suicide retains its self-hit, with safe death completion and warning by claim 23 |
| Rules, combat-damage section, `C4Delay` | charge delay *c*, minutes (F4) | decimal or percent, single (S-0003) | 0.03, kept in binary64 (F4) |
| Rules, audio-visual section, `BombTickingSound` | ticking sound (claim 10) | sound name | none |
| Rules, audio-visual section, `BombAttachSound` | bomb-attach sound (claim 2) | sound name | none |
| Warhead `Airstrike` | air-strike flag: a projectile with this warhead is a designation shot (claim 16) | boolean (S-0003) | no |
| Weapon `MigAttackCursor` | air-strike cursor only (S-0214 row I18, which owns the row); no effect on the shot (claim 16) | boolean (S-0003) | no |
| Object types `AirstrikeTeam` | strike size *n*, normal; a positive value makes the type a designator (Terms) | integer, aircraft | 0 |
| Object types `EliteAirstrikeTeam` | strike size *n*, elite | integer, aircraft | the normal value |
| Object types `AirstrikeTeamType` | strike aircraft type, normal (claim 17) | aircraft type name | none (Edge cases) |
| Object types `EliteAirstrikeTeamType` | strike aircraft type, elite | aircraft type name | the normal type |
| Object types `AirstrikeRechargeTime` | recharge delay *R*, normal (claim 18) | integer, ticks | 0 |
| Object types `EliteAirstrikeRechargeTime` | recharge delay *R*, elite | integer, ticks | the normal value |
| Rules, audio-visual section, `AirstrikeAttackVoice` | air-strike attack sound (claim 17) | sound name | none |
| Rules, audio-visual section, `AirstrikeAbortSound` | air-strike abort sound (claim 20) | sound name | none |
| Rules, audio-visual section, `LaserTargetColor` | laser-target colour of a marked building (claims 17 and 26, presentation): a position in the colour-adjustment table (S-0006), read at drawing time; below 0 or above 15 our engine gives no laser-target mask and the importer warns (claim 26, Q-1621) | integer, dimensionless selector | 0 |
| Imported bomb-icon image asset | bomb icon image (claim 24), image frames by F3 | sprite asset, resolved by S-0005 | missing: no icon is drawn; the importer warns (design) |
| Imported cursor-palette asset | fixed palette of the bomb icon (claim 24), the same asset role as S-0111 claim 19's repair wrench | palette asset, resolved by S-0005 | owned by S-0111 |

These asset roles identify imported resources, not additional rules keys.

## Measurement scenarios

All scenarios use stock data, no engine add-on, director off and no mods (the named test files
of a scenario excepted), on a small flat skirmish map at game speed
index 2 and 800×600. Use one local player and an enemy bystander computer player kept away
from the observed area under the glossary's void rule. Run each variant three times. Record
video, ticks, health, kills and experience with a read-only recorder. Scripted input may time
orders. Follow the glossary's measurement setup, including its read-only recorder and scripted-input allowances. Named test data below
means override sections in the temporary map `S-0519-tests.map`, removed after the run,
only the named fields differ from stock. Any result inconsistent with a variant's expected
result falsifies that part of its claim.

- **T1, attack animal (claim 1).** An attack animal attacks an enemy infantry unit and then an
  enemy vehicle. Expected: the animal leaves the map at the shot, and the infantry dies at the
  first delivery (S-0614 scenarios). Against the vehicle the order is refused. Falsified by any
  difference from S-0614's results. Named test data toggles only the animal marker on the animal
  type: the attack must not change. Give a normal rifle type only that marker: it must not
  gain a latching attack.
- **T2, planting and detonation (claims 2, 5).** A bomb planter plants on an enemy vehicle with
  health above the bomb damage. Record the plant tick, the attach sound (planter local and not
  local), the detonation tick and the health lost by the vehicle and by an adjacent second
  vehicle. Expected: no damage at planting; at detonation one area hit of the bomb damage under
  the bomb warhead; an explosion animation even when nothing else is hit. Falsified by damage at
  planting, or by no animation when a valid explosion-animation entry is available. Repeat
  with infantry, aircraft and building carriers. Named test data removes the firer's bomb-planter
  marker or the target's accepts-bombs marker while retaining a legal attack: planting must
  still work. Give a vehicle the same bomb weapon: it must plant nothing. Kill an infantry
  planter while its projectile is in flight: its later impact must plant nothing. Use a slow projectile
  and move the target before impact: attachment must follow the target despite the separation.
  Plant on a bridge repair hut: detonation must also demolish its bridge (S-0511 claim 16).
  Launch two slow planting projectiles at a clean carrier before either arrives, with distinct
  planter houses and staggered impacts. Only the first impact may attach a bomb: its expiry tick
  and planter credit must survive the second impact unchanged.
- **T3, legality (claim 3).** Order a second bomb planter to attack an already-bombed carrier,
  and an engineer to disarm a carrier without a bomb (forced attack where input allows). Expected:
  neither fires; the first planter does not fire again after its single shot. Falsified by a
  second bomb shot or a disarm shot at a clean target.
- **T4, timer tick (claim 4).** Plant on an idle vehicle. Expected: detonation in tick *t* + *D* + 1.
  Falsified by *t* + *D* or by any other difference.
- **T5, turn position (claim 4).** Reuse S-0203 scenario O10 part A's setup and procedure. Expected: the
  result that O10 lists for detonation during V's own turn. Falsified by O10's pre-turn-pass
  result. An opportunity-firing carrier may fire before its bomb detonates in that turn,
  record shot and explosion ticks rather than assuming the bomb precedes all carrier actions.
- **T6, disarm (claims 5, 7).** An engineer disarms a bomb before it is due, by input and with a
  detector in range. Expected: the bomb icon vanishes, no detonation follows and no damage is
  dealt. Record the ticking sound stopping too. Named test data uses a slow disarm projectile,
  move the carrier after launch: impact must still disarm despite the separation.
- **T7, credit (claim 5).** Two runs in which the bomb kills its carrier. In one the planter
  lives; in the other the planter is killed first. Record house kill totals and surviving
  planter experience. Expected: planter credit in the first run, none from an absent attacker
  in the second, subject to S-0302 claim 1's scoring exemptions. Falsified by opposite credit.
  Repeat after mind-controlling the planter: kill credit follows the surviving planter's
  current house, but bomb visibility follows the house recorded at planting.
- **T8, off the map (claim 6).** (a) Plant on own infantry, load it into a transport before *D*,
  wait past *D*, then unload. Expected: no ticking sound while loaded; detonation at the first
  carrier turn on the map after unloading, not necessarily the following tick. (b) Kill the
  loaded transport in a case where S-0312 kills its passenger. Expected: no bomb detonation
  among those deaths. Repeat (a) with an open-topped transport: taking off-map turns must not
  make its bomb expire. Falsified by a detonation inside either transport.
- **T9, removal and ownership (claims 7, 8).** (a) Plant on own engineer, then capture a building
  with it. Expected: no explosion. (b) Plant on an enemy vehicle and mind-control it. Expected:
  the bomb still goes off on time. (c) Plant on a non-garrisonable enemy building and capture it
  with an engineer. Expected: the bomb is gone. (d) Kill the planter before detonation. Expected:
  the bomb still goes off on time. (e) Transfer a bombed garrisonable building's owner through
  a named test-map ownership trigger: the bomb must remain and keep its expiry tick.
  (f) Sell a bombed own building or erase a bombed enemy vehicle: removal must not explode
  the bomb. (g) Enable the timed-bomb detonate permission in named test data; kill the local
  planter after planting on an own carrier, with no detector present. The icon must remain
  visible and the detonate order must remain available to the planter's original house.
  Record icon, sound and damage as well as timing in each variant.
- **T10, detonate order (claim 9).** Named test data enables only the timed-bomb permission.
  Order a bombed own unit to detonate. Expected: detonation in the tick the order applies.
  Repeat enabling only the death-bomb permission: no detonate cursor or order is available.
  No normal-input fixture for applying the order to an off-map carrier is established.
- **T11, visibility (claim 10).** A bomb planted by the opponent on the local player's unit, with
  and without a local bomb detector nearby. Expected: the icon appears only with the detector
  within its range. Repeatedly move the detector across the boundary: without new planting,
  visibility recomputations are 46 ticks apart. A new planting brings one within 2 ticks.
  Compare inside-range and exact-range cell-centre positions: detection is strict. Compare
  recorded frames with F3, including absolute-tick flicker and the cap. Hide and reveal the
  carrier's cell: the icon requires no shroud. Move the detector out of range after ordering a
  slow disarm shot: the cursor changes but that shot still disarms. Record the ticking loop
  before loading, during transport, after unloading and after disarm. Named test data separately
  sets a short positive delay and zero flicker rate to test our engine's safe F3 departure.
  The original is not required to complete those bad-data runs.
- **T12, suicide (claim 11).** (a) A suicide firer at full health and at reduced health attacks a
  target. Expected: it dies at the shot in both cases, and the target takes only the death weapon's
  damage. (b) Test data: a firer type with a suicide weapon whose damage differs clearly from
  that of the type's own death weapon. Expected: the target loses the death weapon's damage, not
  the suicide weapon's. (c) A suicide vehicle with curtain protection.
  Expected: it still dies. Named test data gives a finite-ammunition firer a suicide weapon
  and a demolition warhead causing nonlethal berserk. A surviving shot must create no
  projectile or report, consume no ammunition and impose no rearm wait. Later shots remain
  subject to the normal firing check.
- **T13, charge (claims 12 to 14).** (a) A charge planter sabotages an enemy building. Record the
  arrival tick and the building's death tick. Expected: death at the first building turn at or
  after arrival + *K*, credited to the planter. (b) A second planter arrives while the countdown
  runs. Expected: the expiry tick is unchanged and the second planter scatters. (c) The building
  is being sold, or is under curtain protection, at arrival. Expected: no charge; the planter
  scatters away from the building. Record flash duration *L* and an entered-by trigger's
  visible counter increment even on refusal. Named test data gives the planter cloak, removes
  enemy detectors and gives its second weapon a distinguishable rearm delay: successful and
  protected/selling arrivals force uncloak, duplicate-countdown arrival does not. Record the
  earliest subsequent second-weapon shot and compare with its arrival-delay table. With the
  random-state recorder isolate the delay draw from scatter and unrelated activity: an ordinary
  second weapon with a positive burst count and initial *k* = 0 must use an inclusive ranged draw
  from 3 through 5, not an inclusive ranged draw from 0 through 2, and leave *k* unchanged. Repeat
  the successful, protected and duplicate-countdown arrivals. Named test data gives different
  primary and secondary burst counts; fire the primary before sabotage so the retained *k* is at
  least the second weapon's burst count: arrival must now use the full-delay calculation with an
  inclusive ranged draw from 0 through 2, without advancing or wrapping *k*. Void any run where
  target clearing reset *k* before arrival. Repeat without a second weapon (1 tick, no delay draw),
  with a sonic second weapon (bare rate, no delay draw), and separately with spark-particle,
  fire-particle and railgun second weapons whose corresponding effect is absent (ordinary
  short/full choice) or already alive (bare rate).
  Record burst counter and rearm remaining directly as well as subsequent firing ticks.
  Extra delay draws, an arrival counter advance, or use of the fixed rate in the ordinary
  *k* = 0 case falsify claim 13. A variant grants
  only the veterancy charge ability: sabotage may be offered but arrival plants nothing.
- **T14, charge and owner change (claim 14).** Capture the charged building with an engineer
  during the countdown. Expected: it still dies at expiry. In a separate run start curtain
  protection during the countdown: it must cancel the charge. The undemolishable case has no
  validated normal-order arrival fixture and is not claimed as measurement coverage.
- **T15, garrisons (claim 15).** Plant a bomb on an enemy-garrisoned building whose health exceeds
  the bomb damage and remains above its evacuation threshold afterwards. Expected: the
  building is damaged and occupant count is unchanged. Separately destroy a charged garrisoned
  building: occupants follow building destruction, not a direct charge hit. Finally bomb an own
  infantry unit, garrison it, wait past expiry, and evacuate it: no explosion while inside,
  detonation at its first on-map turn.

- **T16, designation shot (claim 16).** Use the stock designator against an enemy building that
  accepts charges. Record the building's health at the shot's detonation, the strike's start
  tick and the aircraft that appear. Expected: no health loss from the shot itself; the explosion
  animation of the shot still appears where its warhead supplies one; aircraft are created in the
  detonation tick. Named test data: (a) give a non-designator type the designator's weapon:
  its shots must call no strike and cause no area damage (configured presentation effects may
  remain); (b) give a designator type the same weapon minus the cursor flag
  and order a forced attack: the strike must still be called; (c) kill the designator while its
  shot is in flight: no strike. (d) Give the designation warhead separately configured radiation,
  explosion animation, debris and flash: record each configured effect at impact, but no immediate
  area hit. (e) Use a slow designation projectile and move its object target away before impact:
  the strike still starts for that object. (f) Force-fire at a cell: no strike. (g) With named
  test data giving the primary weapon the designation warhead and permitting air/ground
  targets, attack allied and enemy units, infantry and aircraft as well as buildings: each legal
  object-target impact may call a strike.
  These variants test detonation eligibility, not cursor legality. Falsified by area damage,
  a missing configured effect, a strike in (a), (c) or (f), or a distance/owner rejection.
- **T17, start (claims 17 and 21).** Record, with the random-state recorder, the strike's
  aircraft count, type, house, placement cells and the tick, on two maps whose computed arrival
  edges differ. Repeat with an elite designator, under named test data in which the elite size
  and type differ from the normal ones. Expected: one arrival-cell choice per strike as for a
  paradrop plane; every aircraft placed at the same cell inside the usable area; aircraft attack
  the cell of the building's reference point, so a 2 × 2 building is attacked at its foundation
  cell one cell further along both map axes than its top-corner cell; the attack sound plays at the last aircraft. Draw order as in claim 21.
  Record credits and production queues, with no aircraft factory and no prerequisites satisfied:
  neither credits nor queues change and the aircraft still appear. Repeat with the aircraft type's
  build limit already reached: the call-in still succeeds. Observe the building's mark and tint.
  With named test data giving a positive normal size but zero elite size, an elite designator
  creates no aircraft, and subsequent designations at another building mark it but never create
  aircraft (default empty-strike quirk). On our engine with that quirk off, subsequent starts
  remain possible; this alternative is a design test, not an oracle expectation.
  Falsified by aircraft placed at different cells, outside the usable area, aimed at the object,
  any credit charge or production gate, or a later aircraft call from the empty original strike.
- **T18, recharge (claim 18).** Named test data gives the designator a recharge delay long enough
  to time. (a) Let a strike end without destroying its building, keep the designator attacking:
  the next strike starts at the first designation detonation at or after *e* + *R*. (b) After
  the end, order the designator to move one cell and attack the same building again: a strike
  starts at the first detonation, before *e* + *R*. (c) Designate a second building after the
  end: immediate start. During an in-progress strike, repeated designations of the unchanged
  target must create no extra aircraft. Repeat (a) with different named normal/elite recharge
  delays and promote the designator before the next designation: use its rank at that designation.
  Falsified by a recharge wait in (b) or (c), none in (a), extra in-progress aircraft, or use of
  the earlier rank's delay.
- **T19, cancel and redirect (claim 19).** (a) While aircraft approach, order the designator to
  move: every strike aircraft queues *retreat* in that tick (later behaviour per S-0501); a new designation while
  they are still linked redirects only if all hold equal ammunition. (b) While aircraft approach,
  order an attack on a second building: aircraft with ammunition turn to attack that building.
  (c) After one aircraft has spent its last round, change target: all linked aircraft have
  0 rounds and the newest 101 at most have no target/destination and queued retreat; later
  designation impacts give none an attack order. (d) Destroy the marked building with other
  units: aircraft receive the same cancellation results. (e) Give a strike aircraft enough
  ammunition for unequal positive remaining counts; time the new designation while those counts
  differ: no redirect, counts preserved, retreat queued again. (f) Reissue the same object target
  while approaching: no cancellation. (g) Use two designators on one building; cancelling one
  preserves the mark until the other also cancels. Redirect to a second building: it shows the
  laser-target colour and a restarted pulse (claim 26), as at a start.
  (h) Named test data makes a 103-aircraft strike; cancel before firing: only the 101 newest
  linked aircraft lose their targets/destinations and queue retreat, the two oldest keep their
  orders. Repeat with an empty aircraft before cancellation: all 103 lose their ammunition.
  (i) Use scripted successive attack orders A to B to A and record their application boundaries,
  not just the final target. Each applied change cancels, even when both apply within one tick.
  there is no automatic restoration of the prior aircraft attack orders. If input coalesces the
  orders into one, the run does not test a transient change. (j) Time a later-turn target death
  after the designator has acted, using another attacking object: its target loss cancels in
  the killing tick, before any next designator turn or designation impact. Record cancellation
  effects at the event boundary with a read-only recorder. A fixture unable to expose both
  boundaries does not distinguish immediate cancellation from polling.
  Record behaviour, target, destination, ammunition and marks per tick. Falsified by any contrary
  cancellation result, an attack order in (c)/(e), cancellation in (f), or lost shared mark in (g).
- **T20, end, loss and credit (claim 20).** (a) Record the tick when the last strike aircraft
  leaves the map as *e*. (b) Kill the designator while aircraft approach: the abort sound plays for
  the local owner only, the aircraft retreat and later designations by other designators do not
  affect them. (c) Let the strike destroy an enemy unit: the kill counts for the aircraft's house
  and no experience goes to the designator. (d) Mind-control the designator after the start: the
  aircraft keep their house; their later kills still count for that house, not the designator's
  new house. (e) Repeat death after a move order has cancelled the strike: no abort sound.
  Repeat with a nonlocal designator: no local abort sound. Remove
  the most recently placed aircraft before killing an active designator: the sound originates
  at the most recently placed survivor. Falsified by credit to the designator, aircraft changing
  house, continued linkage after death, or an abort sound in an inactive/nonlocal case.
- **T21, draw isolation (claim 21).** Use T17's random-state recorder with timed input from T18
  and T19. Attribute other draws to their owning specs; record stream states immediately around
  a repeated designation that is not ready, an ordinary cancel, and an equal-ammunition redirect.
  Expected: no draws attributable to readiness, cancel or redirect. At a start, arrival draws
  precede each aircraft's creation/placement draws in creation order, as claim 21 states.
  Falsified by any extra draw or different order attributable to these operations. Runs with
  unrelated unassignable draws across an observation boundary are void, not evidence.

- **T22, absent bomb warhead (claim 22).** Named test file `S-0519-missing-warheads.ini`
  is a temporary base-rules override with only the bomb-warhead entry removed entirely; an empty
  map-layer value would keep the inherited warhead. Start a fresh session. Plant on an ordinary
  vehicle beside another vehicle, then separately on a bridge repair hut. Record health,
  animation, ticking sound, attachment and bridge state at expiry. Expected: bomb ends with
  no hits and no explosion animation, but the hut still demolishes its bridge. Damage, an
  explosion animation or a lingering attached bomb falsifies claim 22. Repeat off-map expiry
  and return to check claim 6. Remove the named file after each three-run variant.
- **T23, absent demolition warhead (claim 23).** Use the same named temporary base-rules file,
  now omitting only the demolition-warhead entry, in a fresh session. A suicide infantry unit
  and, separately, a suicide vehicle attack an isolated enemy target on dry land. Keep each
  death weapon's own warhead valid. Record health immediately around the self-hit, whether
  play continues, death presentation and target damage. Expected hypothesis: self-hit
  reaches zero health; ordinary infantry death selection may stop play rather than complete
  its death effect. This is an investigation of the unmeasured bad-data outcome, not a promise
  of a surviving original run. A harmless self-hit falsifies the health hypothesis. On our
  engine require a warning, completed death with ordinary presentation, no warhead-selected
  special death effect, and exactly the normal death-weapon detonation. Run each variant three
  times under the common setup, then remove the named file.

- **T24, competing detonation effects (claims 2, 7 and 16).** Under the common setup and
  three-run rule, use named map weapon/warhead variants from S-0514 scenario E13, enabling
  only each stated pair of effects. Keep ordinary firing legal and give slow projectiles so
  delivery can be observed separately. Record bomb attachment, carrier health, strike aircraft
  count and configured explosion presentation at impact. (a) Combine takeover and planting on
  an infantry planter whose type supplies no takeover capability: no bomb attaches. (b) Combine
  planting and designation on that planter: only a bomb attaches. Repeat with a vehicle firer:
  neither bomb nor strike occurs, even when the vehicle is a designator. (c) Combine designation
  separately with charging and locomotor replacement; use a designator and a building target
  for which those earlier effects cannot succeed: no strike starts. Erasure over designation
  uses S-0514 E13's beam-equipped firer rather than an unsafe missing-beam arrangement.
  (d) Deliver a combined designation/disarm shot from a non-designator to a bombed building:
  the bomb remains, with no strike. (e) Combine direct rocking and disarming, using a firer
  destroyed during flight so rocking cannot succeed: a bombed vehicle keeps its bomb, while a
  bombed building is disarmed. (f) Combine disarming separately with disguise-making and
  nuke-making: the bomb is disarmed, and neither later effect occurs. Use long bomb delays
  so expiry cannot confound these observations. Selected effects make no immediate area hits.
  Configured shared presentation remains eligible. A suppressed bomb attachment, disarm or
  strike occurring, or a later effect appearing after an unsuccessful earlier selection,
  falsifies the relevant claim. Remove all named test overrides after the runs.

- **T25, bomb icon drawing (claim 24).** Common setup, lossless captures at 800 × 600 at the highest
  detail level, no shroud over the observed cells. The local player's bomb planter plants on an
  own-side test vehicle, an infantry unit, a landed aircraft and a 2 × 2 building, each far from the
  others. Named test data gives a long timed delay. Record every frame until detonation. (a) Compare
  the icon pixels with each frame of the bomb icon image under the fixed cursor palette at neutral
  brightness, and with house-coloured, lit and translucent alternatives. (b) Find the icon canvas
  centre: on the vehicle, infantry unit and aircraft it must coincide with the projected world
  position, including height, not necessarily the visible art's centre; on the building, with the
  projected top corner of the foundation, not with the building's centre. (c) Repeat (a) at night
  lighting of the test map: the icon stays at neutral brightness.
  (d) Select and damage the vehicle carrier: its health bar and selection marker paint over the
  icon. (e) Put two bombed vehicles so that their icons overlap, one in front of the other in the
  ground layer's order: the later one's icon and health bar cover the earlier one's icon. (f) Repeat
  (a) with the bomb planted by the opponent and a local bomb detector in range: identical pixels.
  (g) Overlap the icon with a different object's body, and separately with a beam: the icon covers
  the body but the beam covers the icon. Show the carrier's repair wrench, pips and insignia where
  applicable: each covers its own bomb icon on overlapping pixels. Move the carrier across the
  status-extra culling boundary of S-0111 claim 1: the icon follows that culling. Compare the
  light-alpha-shaded pixels against S-0104 claim 15; only that shading may darken the neutral icon.
  If no eligible nontransparent icon pixel can be exposed beneath a shroud piece while the
  carrier's tested cell is unshrouded, that subtest is inconclusive, not confirmation.
  Falsified by a remapped, lit or misplaced icon, by the icon covering its carrier's other extras,
  by a different overlap or culling result, or by incorrect light-alpha shading.
  (h) On our engine only, omit the imported bomb-icon image asset: the importer must warn and
  no icon may draw, without changing attachment, timing or damage. A missing warning, a fallback
  icon or a simulation change fails this design test, not an established original claim.
- **T26, charge flash (claim 25).** Common setup and captures. Named test data gives a charge delay
  long enough that *L* is at least 40 ticks, and test-map lighting with two cases: ordinary light
  (building brightness well below 1500 ‰) and a bright ambient (above 1500 ‰). A charge planter
  sabotages an enemy test building that has a voxel turret and a slot animation, once at full health
  and once in its damaged state. Record per tick the flash count implied by F6 and the shade rows of
  body, turret and animation pixels. Expected: on in exactly the frames where *c* mod 4 is 2 or 3,
  brightness 500 ‰ up under ordinary light and 500 ‰ down under the bright ambient, the shadow
  unchanged, no colour change, the damaged frames brightened in the same way. In a separate run
  start curtain protection during the flash: the countdown is cancelled and the flash continues.
  While it flashes, cause a separate damage flash and record its starting count: the remaining
  count must be replaced by that flash's duration, not increased by it. Repeat with the building
  owned by the local player: after accounting for house-colour remapping the flash pattern and
  brightness changes must match. Falsified by a different pattern, a different step, a colour change,
  a changed damage-state frame, an added rather than replaced count, or owner-dependent flashing.
- **T27, air-strike mark (claims 26 and 21).** Common setup, captures and the random-state recorder.
  A stock designator designates an enemy test building with a slot animation and a voxel turret.
  Named test data sets the laser-target colour to two different table positions in two runs, and
  gives the colour-adjustment table distinguishable entries. For the pulse runs, give the strike
  aircraft negligible damage and enough approach time to keep the building marked through at
  least three complete 128-tick pulses; discard a run that ends or restarts the mark earlier.
  Record per tick the shade rows and
  colours of body, turret and animation pixels and the stream state. Expected: the mask of the
  selected entry, united with the shaded pixels, constant over the mark; brightness factors as in
  claim 26's table, with the dimming length 20 + the drawn value; exactly one ranged draw from −5
  through 5 in the building's turn *q* + 10 and no other draw caused by the mark; a pulse of
  128 ticks afterwards. Redirect the strike to a second building: the same colour, a restarted
  sequence and a fresh draw ten turns after its first marked turn. Mark a force-shielded building:
  the union of both masks. Cancel one of two strikes holding one building: no restart and no draw.
  Unmark the building before turn *q* + 10: no draw. These are part (a), falsified by a pulsing
  colour, a different factor on a freshly drawn part, a missing or extra draw, a restart when
  sharing the mark, or a redirected building that keeps an older colour.
  (b) For persistent-body refresh, use a separate building with no running animations, no damage
  or flashes, fixed lighting and a stationary view. Capture consecutive frames through a pulse,
  then scroll away and back at a recorded phase. Compare pixels whose predicted shade rows differ,
  rather than expecting every integer-factor change to cross a shade-row boundary. The unresolved
  original hypotheses are: body rows follow F7 every frame, or they retain their last-repaint value
  until the scroll repaints them. Following the changing rows without another repaint falsifies
  the retained-value hypothesis; a stale row that changes to the current F7 row on repaint
  falsifies the every-frame hypothesis. Confounded captures are inconclusive. On our engine only
  the every-frame result is accepted; a stale row falsifies the design rule, not an established
  original claim.
  (c) Mark a charged building during its flash; separately start protection so its first protected
  turn and the first marked turn coincide. Record shade rows and random state. Expected: F7, not
  a mark followed by protection or a flash added last; when both phase draws occur in one building
  turn, the protection draw precedes the mark draw. Choose lighting and sampled factors for which
  those alternative orders give distinct shade rows. Record the shadow too: it remains unchanged.
  Repeat with a named temporary loose art override (the expansion art file used by
  S-0007 claim 2), copied from stock with only the test slot animation's normal-light flag
  enabled, then removed after the run: its brightness remains
  1000 ‰. A reversed draw order, distinguishably different composition, changed shadow or
  pulsing normal-light animation falsifies that part.
  (d) Repeat (a) with the marked building owned by the local player, and with an allied owner.
  After accounting for the ordinary house-colour ramp, mask and phase factors must be identical.
  A viewer- or owner-dependent effect falsifies the houses rule, apart from the shroud exception
  for building animations (S-0513 SW10 part i5) and the open case of a body whose centre cell is
  shrouded, which these runs avoid by keeping every cell of the building's foundation, and every
  cell holding one of its slot animations, visible. The near-expiry phases and missed
  building turns are not covered by these runs (claim 26).
  (e) Selector outside the table (Q-1621). Repeat (a)'s capture of the marked building with the
  laser-target selector set to 16 and then to −1 in a named test selector layer (the named
  test map's own rule sections), each in two
  runs A and B. The two runs differ only in table positions 0 and 15 and in position *e*, the
  position named by the laser-target selector in force before the selector layer (the stock
  data's selector, written here only by its role). Before the runs, *e* is read from the rules
  in force without the selector layer and recorded with the measurement; it is not written
  into the test files. Run A gives each distinct position among 0, 15 and *e* a distinct,
  clearly visible invented triple, run B others, each of B's triples differing from each of A's.
  The fourth control's position (below) is the one exception: it holds (0, 0, 0) in both runs.
  These
  triples are written in a separate named test table layer, loaded before the selector layer
  and present in every run and every control; it changes no selector. The table layer is a named
  test game-mode file, or a named loose copy of the base rules that changes only the table.
  S-0006 claim 2 lets either change the table by position, while a language rules file cannot
  change positions the base rules give, so it is never used here. The selector layer
  holds only the selector. Compare each capture with three control runs in A and in B, each with the table layer in force: at selectors 0
  and 15 set in the selector layer, and at the earlier layer's selector *e* with the selector
  layer left out. A wrapped selector (16 to 0, −1 to 15), a
  clamped one (16 to 15, −1 to 0), a rejected one that falls back to the absent-key default
  (position 0), or a rejected one that keeps the earlier layer's value (position *e*) would make
  the marked colour follow position 0's, 15's or *e*'s triple from A to B. The
  original's colour itself is recorded but not asserted. Expected: the pulse and the phase draw as
  in (a), and for each selector the marked colour is the same in A and B. Falsified (for that
  selector) by a marked colour that changes from A to B as the position-0, position-15 or
  position-*e* triple does. A selector's run is inconclusive when any of its three controls
  (positions 0, 15 and *e*) shows no change in the marked colour from A to B, since that
  position's triples then cannot be told apart in the marked pixels. A colour that changes from
  A to B in any other way is inconclusive. If the stock data's position *e* is 0 or 15, keeping
  the earlier layer's value cannot be told apart from a wrap or a clamp onto that position: a
  marked colour following that position's triple falsifies the claim but does not say which of
  those alternatives occurred, while a marked colour that follows none of the three positions
  still backs the whole claim, keeping the earlier value included. The original's marked
  colour at selectors 16 and −1 is recorded in every run but not asserted. For Q-1621
  question 1, run A adds a fourth control: the selector layer names a table position other
  than 0, 15 and *e*, whose triple both runs' table layers set to (0, 0, 0), a triple that adds
  nothing. A marked colour at 16 or −1 that differs from this control shows that the original
  changes the colour; one that matches it leaves question 1 open, since a colour that adds
  nothing and no mask look the same (Open questions). Engine fixture:
  the same settings give the marked building's pixels without any laser-target colour (*K* = 0),
  the pulse unchanged, the force-shield mask kept in a repeat with the test building also
  force-shielded, and one importer
  warning per run naming the key and its value; any laser-target colour or a missing warning
  falsifies the engine rule. The oracle cannot falsify the engine rule.

## Open questions

- (Q-1621 question 1, the part left open.) Whether the original's marked building shows any
  colour change at all for a laser-target selector outside the table. Its colour would come
  from no imported triple and might add nothing visible. T27 part e records the original's
  colour and compares it with a control whose selector names a position with the triple
  (0, 0, 0), which adds nothing. A marked colour at 16 or −1 that differs from that control
  settles the question: the colour changes. One that matches it leaves the question open,
  since a colour that adds nothing and no mask look the same.
- (Q-1621 question 1, claim 26.) What a laser-target selector outside the table other than −1
  and 16 shows. T27 part e avoids such values, which may make the original fail.
- Whether a marked building's body (and its voxel turret and barrel) loses the marking colour
  for a viewer to whom the building's centre cell is shrouded, as S-0513 claim 29b's
  centre-cell rule removes a protected object's mask. Not traced for the marking; a T27 (a)
  repeat with the centre cell shrouded for the local player and a foundation corner visible
  would show it.

- No separate effect of the imported animal marker has been traced beyond parsing it. Claim 1
  specifies only that the marker does not itself supply the latching weapon attack.
- What an infantry unit with explosive charges does at arrival when it plants nothing because it
  has only the veterancy ability (Edge cases). Not traced.
- Can a unit in *sabotage* reach a destination cell that contains no building, for example a
  bridge cell? No reachable fixture is established, so its arrival effects are not specified.
- A strike whose target is not a building (reachable only with data in which the primary weapon's
  warhead has the air-strike flag, S-0308 row 6). Whether its strike target is forgotten when
  that object dies, other than through the designator's own loss of target (claim 19), is not
  traced.
- Whether a designator that goes off the map without being removed from play (into a transport
  or a garrison) ends its strike, as claim 20 says of removal from play. Not traced.
- Whether the original's persistent body picture of a marked building shows every value of the
  pulse (claim 26), or only the value at its area's last repaint. No repaint tied to the pulse was
  found, only to the flash's on/off changes. Claim 26 chooses the current value in every frame for
  our engine only; T27(b) distinguishes that design test from the unresolved original hypotheses.

## Changes for other specs

- **S-0102 F9.** Its sentence on "no selected adjustment" names the protection selectors (S-0513
  claim 29a) and the berserk selector (S-0610); it should name the laser-target selector too
  (claim 26, Q-1621).
- **S-0102 claim 34's marked-building rows and S-0006 claim 2.** The rows cite claim 26 for the
  laser-target selector, so the out-of-table rule reaches them without a change; S-0006 claim
  2 describes the table by position only and needs none.
- **S-0102 claim 34, shroud exception for building animations.** The change is listed under
  S-0513's Changes for other specs, which owns it (S-0513 claim 29b); this spec's claim 26
  relies on it.
