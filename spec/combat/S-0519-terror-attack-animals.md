---
id: S-0519
title: Terror and attack animals
topic: combat
revision: 1
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
supersedes: []
---

# S-0519: Terror and attack animals

## Summary

The special attacks of infantry and vehicles that do not work like an ordinary shot. These are
attack animals, planted bombs (planting, legality, countdown, detonation, disarming, the detonate
order and credit to the planter), suicide weapons, and demolition charges planted on buildings.
Drone and grappling parasites belong to S-0614. Garrisons are S-0508, building capture is S-0509
and deployable units are S-0517.

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

2. **Planting a bomb.** When a projectile whose warhead is a bomb-planting warhead detonates, it
   does no area damage. Instead it plants a bomb on its target
   when all of these hold:
   - the projectile's firer still exists and is an infantry unit,
   - the target is a unit, an infantry unit, an aircraft or a building,
   - the target carries no planted bomb.

   Otherwise nothing happens. The distance between the projectile and the target is not tested.
   The bomb records its planter, the planter's house, its carrier and the plant tick *t*. When the
   planter's house is the local player, the game-wide bomb-attach sound plays. The accepts-bombs
   flag of the target's type and the bomb-planter flag of the firer's type affect only the cursor
   (S-0214 row I19); planting tests neither.
   - Backing: `measurable` (scenario T2)
   - Confidence: high

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

5. **Detonation and credit.** When a bomb goes off on a carrier that is on the map:
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
     credit when the planter has been removed

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
     test),
   - a building changes owner and its type is not garrisonable (S-0312 claim 7, S-0509 claim 13),
   - the carrier is removed from play other than by death: sold, used up by entering a building,
     or erased.
   - Backing: `measurable` (scenarios T6 and T9)
   - Confidence: high

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
      Formulas F3.
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

    *Rearm.* The walker's rearm countdown is set to the delay that S-0301 claims 1 to 8 give a
    shot of its second weapon.

    *Scatter.* The walker gets a scatter request whose threat point is the building's centre,
    forced and move-anywhere (S-0208 claims 11 and 12).

    The arrival tests neither whether the building's type accepts charges nor who owns the
    building.
    - Backing: `measurable` (scenario T13) for the arrival event, planting, refusals, cloak,
      rearm and scatter; `unmeasured` for the
      missing owner and accepts-charges tests at arrival, because no validated fixture reaches
      arrival with an otherwise refused destination. The computer-only starts of S-0215 describe
      the original only; our AI uses ordinary legal orders
    - Confidence: high for the three rows; medium for the rearm delay

14. **The charge countdown.** Planting a charge does four things:
    - it marks the building as having a pending countdown,
    - the building flashes for *L* ticks (Formulas F4, presentation only),
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
    waits there (claim 6). Stock data has no assaulter type (S-0508 claim 14).
    - Backing: `measurable` (scenario T15)
    - Confidence: medium

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
flicker rate (integer ticks). Let *q* = ⌊*D* / 6⌋. Let *b* = 1 when (*n* mod 2*F*) ≥ *F*, else 0.

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

## State and ordering

**Planted bomb.**

| State | Event | Result |
|---|---|---|
| none | bomb-planting projectile detonates on a valid target (claim 2) | attached, timer from *t* |
| attached | carrier's turn on the map, tick > *t* + *D* | detonation (claim 5) |
| attached | carrier off the map | timer cannot detonate it; no ticking sound |
| attached | detonate order, carrier on the map | detonation (claim 5) |
| attached | detonate order or death, carrier off the map | removed, no effect (claim 6) |
| attached | carrier dies on the map | the bomb detonates once at the carrier's position; it does not detonate a second time as part of the carrier's death (S-0312 claim 7) |
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

**Ordering.** A suicide firer's hit and its death-weapon detonation both occur in the tick of the shot.
When a planted bomb's detonation kills its carrier, the bomb does not detonate a second time.
The ticking sound stops in the same tick as the bomb ends.

**Cross-spec correction required.** S-0203 claim 1 and its ordering-table row 2 currently put
planted-bomb detonation before the turn pass. They must instead use claim 4 here: detonation
during the carrier's turn, with the turn-removal consequences of S-0203 claim 10. The separate
bomb update only maintains sounds and visibility. S-0506 claim 26 must drop its corresponding
pre-turn planted-bomb reference. These corrections must accompany this spec at hand-off.

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
- **No bomb warhead or no demolition warhead.** Both game-wide warheads default to none. What a
  detonation or a suicide hit does without one is an open question.
- **Overdue bombs after transport.** A bomb that waited off the map goes off at the carrier's
  first turn after it comes back, however long ago its time ran out (claim 6).
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
| Building type `CanC4` | accepts charges: not tested at arrival (claim 13; S-0214 and S-0302 own the row) | boolean (S-0003) | yes |
| Rules, combat-damage section, `IvanWarhead` | bomb warhead (claim 5) | warhead name | none |
| Rules, combat-damage section, `IvanDamage` | bomb damage (claim 5) | integer, health points | 100 |
| Rules, combat-damage section, `IvanTimedDelay` | timed delay *D* (claims 4, 10) | integer, ticks | 450 |
| Rules, combat-damage section, `IvanIconFlickerRate` | flicker rate *F* (F3) | integer, ticks | 8 |
| Rules, combat-damage section, `CanDetonateTimeBomb` | timed-bomb detonate permission (S-0214 row G3, which owns the row) | boolean (S-0003) | no |
| Rules, combat-damage section, `CanDetonateDeathBomb` | no reachable effect (claim 9) | boolean (S-0003) | no |
| Rules, combat-damage section, `C4Warhead` | demolition warhead (claims 11, 14; S-0011 owns the row) | warhead name | none |
| Rules, combat-damage section, `C4Delay` | charge delay *c*, minutes (F4) | decimal or percent, single (S-0003) | 0.03, kept in binary64 (F4) |
| Rules, audio-visual section, `BombTickingSound` | ticking sound (claim 10) | sound name | none |
| Rules, audio-visual section, `BombAttachSound` | bomb-attach sound (claim 2) | sound name | none |

## Measurement scenarios

All scenarios use stock data, director and mods off, on a small flat skirmish map at game speed
index 2 and 800×600. Use one local player and an enemy bystander computer player kept away
from the observed area under the glossary's void rule. Run each variant three times. Record
video, ticks, health, kills and experience with a read-only recorder. Scripted input may time
orders. Use the unit arena or bench only with the stock executable. Named test data below
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
  the bomb. Record icon, sound and damage as well as timing in each variant.
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
  the original is not required to complete those bad-data runs.
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
  earliest subsequent second-weapon shot and compare with its rearm formula. A variant grants
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

## Open questions

- No separate effect of the imported animal marker has been traced beyond parsing it. Claim 1
  specifies only that the marker does not itself supply the latching weapon attack.
- What an infantry unit with explosive charges does at arrival when it plants nothing because it
  has only the veterancy ability (Edge cases). Not traced.
- Can a unit in *sabotage* reach a destination cell that contains no building, for example a
  bridge cell? No reachable fixture is established, so its arrival effects are not specified.
- The effect of a missing bomb warhead or demolition warhead (Edge cases).
- The air-strike designation by a weapon with the air-strike flag is owned by no spec.
