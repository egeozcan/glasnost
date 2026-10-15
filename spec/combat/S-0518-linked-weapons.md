---
id: S-0518
title: Linked weapons (prism support beams, charging of overpowerable buildings)
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

# S-0518: Linked weapons (prism support beams, charging of overpowerable buildings)

## Summary

There are two ways in which one weapon strengthens another. In the first, buildings of the prism type
send support beams to another building of that type, which then fires one stronger shot. This spec
covers which buildings may support and which one is chosen, how many may support at once, when the
beams and the shot happen, how much the shot gains, and what happens when a participant is lost. In
the second, infantry with an electric-charge weapon charge a building of an overpowerable type. This
spec covers how a charger is registered and counted, when the building is overpowered, how infantry
start charging by themselves, and how they stop.

Other specs own the following, and this spec only points to them:
- the ordinary rearm countdown and firing check (S-0301 claims 1 to 6) and the timing of a building's
  delayed shot (S-0301 claim 14),
- the drawing of charge turrets (S-0301 claim 16), which is not a charging rule,
- shot damage (S-0302 claim 2) and area damage (S-0303 claims 1 and 2), whose detonation input claim 9 here scales,
- how a building processes its attack behaviour (S-0215 claim 32),
- the weapon an overpowered building and a charging unit choose (S-0308 rows 10 and 11),
- the operation rule that chargers bypass (S-0405 claim 8).

Superweapon charge (S-0512) has nothing to do with charging here. No deployed weapon state (S-0517)
enters these rules.

## Terms

- **Prism type.** The one building type that the rules name for prism support (Import mapping).
  When no type is named, nothing in claims 1 to 10 happens.
- **Prism master, supporter.** A building of the prism type that is making an attack is the
  *master*. A building of the prism type that it recruits to send it a beam is a *supporter*.
- **Support beam.** The beam that a supporter draws towards its master. It is drawing only: it has
  no projectile and does no damage (claim 6).
- **Support count *N*.** A per-building whole number, 0 at creation. It counts recruitments
  for the master's next shot (claims 4, 8 and 9), not necessarily distinct supporters if
  a supporter's beam and rearm finish soon enough to let it be recruited again.
- **Support cap *N*ₘₐₓ, support modifier *s*, support rearm *R*ₛ, beam duration *L*ₛ.** Four
  game-wide imported values (Import mapping). *s* is held as a whole-number percentage.
- **Support reach.** The master's weapon range ρ(master, 1) for its slot-1 weapon (S-0307 claim 13).
  It is 0 when the master's type has no slot-1 weapon.
- **Charger list, charger.** Each building has a list of *registered chargers*: infantry units whose
  electric-charge shots have reached it (claim 11). The list holds each unit at most once. *n* is
  its length at a given moment. This is S-0405's *n*<sub>c</sub> (S-0405, Terms, *Charger*).
- **Overpowered.** A per-building state of an overpowerable building, decided in each of its turns
  (claim 13). S-0308 row 10 reads it.
- **Charging mark.** A per-unit flag that a unit carries while it is charging a building by itself
  (claims 14 and 15).
- **Electric-charge warhead, overpowerable type.** As in S-0308 (Terms, rows 11 and 10).

## Behaviour

### Prism support

Claims 1 to 10 cover the ordinary building-attack path: a prism type without the SAM flag and
without installed upgrades. Combinations with those special attack paths remain outside these
claims (Open questions).

1. **Who links.** Only a building of the prism type takes part in support. It does so when its
   attack behaviour gets *can fire* (S-0215 claim 32). Then claims 2 to 5 replace the ordinary
   shot of that processing. Other buildings, including buildings of other types with the same
   weapons, fire as S-0301 and S-0215 give. A unit never takes part.
   - Backing: `measurable` (scenario PS1)
   - Confidence: high

2. **Who may support.** When the master gets *can fire* and its support count is below the cap
   (*N* < *N*ₘₐₓ), it looks for a supporter. A building *B* is eligible when all of these hold:
   - *B* belongs to the master's own house (an ally's building never supports),
   - *B* is of the prism type and is not the master,
   - *B* is alive,
   - *B*'s rearm countdown has run out (S-0301 claim 1),
   - *B*'s delayed-fire countdown is 0 (S-0301 claim 14). This keeps out a building whose beam or
     shot is pending, except one whose countdown was set to 0 (quirk
     `quirks.prism_zero_delay_double_count`),
   - *B* is not being drained (glossary; S-0405, Terms),
   - *B*'s current behaviour is not *attack*,
   - the distance F2 between *B* and the master is at most the support reach.

   Nothing else is checked. *B* need not be in operation, switched on or powered, it may be
   under a pulse (S-0405 claim 8), and it may be under construction or being sold (only the *attack*
   behaviour excludes it; quirk `quirks.prism_support_ignores_operation`). *B* need not be able to see or reach the master's
   target.
   - Backing: `measurable` (scenarios PS2 and PS3); `unmeasured` for pulse-disabled
     support, because no stock pulse fixture is supplied to isolate that condition
   - Confidence: high (each condition); medium (the build-up and sale cases, never observed)

3. **Which one is chosen.** Of the eligible buildings, the one at the smallest distance F2 is
   chosen. On a tie, the one that the house acquired first wins (the order in which the house
   acquired its buildings, S-0404 claim 7).
   - Backing: `measurable` (scenario PS4)
   - Confidence: high (nearest); medium (the tie order)

4. **One supporter per processing.** When a supporter is found, the master's support count rises
   by 1 and the master does not fire in that processing. The supporter's beam becomes pending
   (claim 6), and the supporter starts its firing animation. Since *can fire* processing repeats
   every tick (S-0215 claim 32), the master recruits at most one supporter per tick. The first
   *can fire* processing that finds no eligible building, or finds *N* ≥ *N*ₘₐₓ, starts the
   master's own delayed shot (claim 7). With initial count 0, a non-negative cap and *K*
   initially eligible buildings that remain available until recruited and cannot become eligible
   again during recruitment, the shot therefore starts at its (min(*K*, *N*ₘₐₓ) + 1)-th
   consecutive *can fire* processing. These are integer counts, with no rounding.
   - Backing: `measurable` (scenarios PS1 and PS5)
   - Confidence: high

5. **No chains.** A supporter never recruits supporters of its own for the master. Recruiting
   happens only in a master's *attack* behaviour, and a building in *attack* is never eligible.
   So the master gains only from direct recruitments. Starting at count 0, at most
   max(*N*ₘₐₓ, 0) recruitments are made for one shot; a non-positive cap gives no support.
   - Backing: `measurable` (scenario PS5)
   - Confidence: high

6. **Support beam.** The supporter's beam uses the supporter's delayed-fire countdown, set to its
   type's delayed-fire field *d* (ticks) when it is recruited.
   - **When.** The beam comes in the supporter's turn max(*d* − 1, 0) ticks after the tick of
     recruitment, if the supporter takes its turns after the master in the update order (S-0203
     claim 3). If it takes them before the master, the beam comes one tick later.
   - **Path.** The beam runs from the supporter's fire point for weapon slot 0 to the master's fire
     point for weapon slot 0 (glossary, *fire point*), as that point was at recruitment. The beam
     does not follow the master afterwards.
   - **Drawing.** The beam lasts *L*ₛ ticks, is 3 pixels thick and has the house's laser colour.
   - **Effects.** None in play: no projectile, no damage, no ammunition, no burst count.
   - **Afterwards.** In the same turn, the supporter's rearm countdown starts with exactly *R*ₛ
     ticks: no rate-of-fire factor and no random draw apply. Its own support count becomes 0.
   - **While pending.** The supporter's firing check answers *rearming* (S-0301 claim 14).
   - Backing: `measurable` (scenario PS5)
   - Confidence: high (path, rearm, no damage); medium (the one-tick dependence on update order)

7. **The master's shot.** For a master, claim 4's last processing always starts S-0301 claim 14's
   delayed shot, whether or not its type has the animation-delayed-fire flag. The countdown is the
   type's delayed-fire field. The shot always uses weapon slot 0, even if weapon choice gave slot 1
   for the check that answered *can fire*. When the delayed shot comes, the check and the shot
   follow S-0301 claim 14. The shot's rearm countdown is the ordinary one (S-0301 claims 1 to 6).
   With a delayed-fire field of 0, the shot comes in the same turn as the processing that started
   it.
   - Backing: `measurable` (scenarios PS1 and PS5)
   - Confidence: high

8. **The bonus is fixed when the master fires.** The gain depends only on the master's support
   count *N* at the moment its delayed shot launches a projectile. It does not depend on whether
   the beams have been drawn yet or whether the supporters are still alive. A supporter destroyed,
   sold or captured after it was recruited still counts. Recruitment, not arrival of a beam,
   establishes the contribution. All participants have the same type and thus the same
   delayed-fire field; independently longer supporter delays are not a stock-play case.
   - Backing: `measurable` (scenario PS6)
   - Confidence: high

9. **Supported damage.** When the master's delayed shot launches a projectile and *N* > 0, the
   projectile's damage multiplier becomes *M* (F1) and the master's *N* becomes 0. Every
   projectile starts with *M* = 256, that is, ×1. For an ordinary damaging detonation, the
   amount supplied to area damage is ⌊*D*ₜ × *M* / 256⌋ (F1), where *D*ₜ is the integer
   damage still carried at detonation: S-0302 claim 2's shot damage after any in-flight loss
   (S-0306 claim 18). Falloff, versus values and target defences apply afterwards, not before
   this scaling. This is not necessarily the target's health loss. A shot with *N* = 0 is
   unchanged. When *N* > 0 the master's own beam, if its weapon draws one, is 5 pixels thick
   instead of 3 (drawing only).
   - Backing: `measurable` (scenario PS1)
   - Confidence: high

10. **Count reset and participant loss.** The master's support count becomes 0 in these cases:
    - its shot launches a projectile (claim 9),
    - its attack behaviour is processed with no target,
    - its attack behaviour drops the target (S-0215 claim 32: *illegal*, *impossible*,
      *ammunition* or *out of range*).

    The count is kept, with no beam redrawn, when the delayed shot is dropped by its own firing
    check (S-0301 claim 14) or launches no projectile while the master keeps its target. The next
    *can fire* processing then adds up to *N*ₘₐₓ − *N* further supporters, and the shot after
    that is scaled by the total (quirk `quirks.prism_count_survives_dropped_shot`).

    If the master is destroyed or sold after recruiting, its supporters still draw their beams to
    the stored point and then rearm by claim 6.
    - Backing: `measurable` (scenario PS6); `unmeasured` for a projectile-less launch
      retaining the count, because no isolated fixture preserves the target through that failure
    - Confidence: high (the resets and the master's loss); low (how often the kept count can
      occur in play)

### Charging of overpowerable buildings

11. **The charge shot.** A detonation that selects the electric-charge effect skips ordinary
    area damage at any target. This effect is lower priority than mind control and demolition
    charge planting if the same warhead enables them too. It does not suppress effects shared
    by detonations, including radiation creation before effect selection and the later
    detonation animation, debris and flash where configured. If the projectile's target is a
    building and its firer is an infantry unit, the electric-charge effect puts the firer on
    the building's charger list:
    - this happens to any building, whatever its type or owner,
    - a firer already on the list is not added twice,
    - a firer that is not infantry registers nothing.
    - Backing: `measurable` (scenarios TC1 and TC4)
    - Confidence: high (area-damage bypass and registration); medium (combined effects)

12. **Who stays on the list.** A charger leaves the list when it leaves play. For a building of an
    overpowerable type there is a second way off: in each of the building's turns, every charger
    whose current target is not that building is removed. So *n* counts the registered chargers
    that are still aiming at the building, as of its last turn. For a building of another type,
    a charger stays on the list until it leaves play (quirk `quirks.stale_chargers_kept`).
    S-0405 claim 8 reads *n* whenever it judges the building.
    - Backing: `measurable` (scenarios TC1 and TC4)
    - Confidence: high

13. **Overpowered.** In each of its turns, after claim 12's removal, a building of an
    overpowerable type is *overpowered* when either of these holds:
    - *n* ≥ 3,
    - *n* ≥ 1, the building is switched on, and its owner has full power (*p* = 1 in S-0405 F3).

    Otherwise it is not overpowered. A building of another type is never overpowered. While a
    building is overpowered, weapon choice gives it slot 1 (S-0308 row 10). Independently of this
    state, *n* ≥ 2 bypasses only the switched-on and house-power requirements of operation,
    every other requirement still applies (S-0405 claim 8).
    - Backing: `measurable` (scenario TC1)
    - Confidence: high

14. **Starting to charge on its own.** An infantry unit or vehicle whose slot-1 weapon has an
    electric-charge warhead looks for a building to charge:
    - **when:** in the processing of its *guard* or *area guard* behaviour (S-0215), whenever it
      has no target at that point. In *area guard* this comes after the guard spot and automatic
      search steps of that processing,
    - **where:** the eight cells next to its own cell, taken in step-direction order 0 to 7
      (glossary, *step direction*),
    - **what:** the first of these cells that holds a building of an overpowerable type owned by
      its own house. An ally's building is never chosen.

    The unit then takes that building as its target, sets its charging mark and gets *attack*. In
    *guard*, if no such building is found, the ordinary *guard* processing runs instead; when one
    is found, it does not run in that processing. A human player can also make a unit charge with
    an attack order on an allied overpowerable building (S-0308 row 11). There is no special
    cursor for this. A vehicle charges in the same way, but registers nothing (claim 11).
    - Backing: `measurable` (scenarios TC2 and TC4)
    - Confidence: high (the scan and the own-house rule); medium (the order of the steps within
      *area guard*)

15. **Breaking off.** While an infantry unit's charging mark is set, every processing of its *attack*
    behaviour first runs an automatic search of the range kind centred on its position (S-0309
    claim 1). If the search finds a target, the unit takes it instead and the mark clears; it then
    attacks that target as usual. There is no targeting-timer gate or targeting-timer reset
    draw for this search; ordinary behaviour-delay draws still apply (S-0215 claim 21).
    The mark also clears whenever the unit is given a target by an
    order or by any other ordinary target change. A unit that stops aiming at the building drops
    off the building's list at the building's next turn (claim 12). This search belongs to the
    *attack* behaviour of infantry only. A vehicle with the mark set does not break off and keeps
    its target until another target change (Edge cases).
    - Backing: `measurable` (scenarios TC3 and TC4)
    - Confidence: high (search and switch); medium (the other clears)

16. **Support-modifier layering.** Reading a rules layer that contains the general section
    rescales the held support modifier by F4, even if that section omits the modifier key.
    A layer without the general section does not rescale it. Consequently a map can change
    supported damage without naming the modifier (quirk `quirks.prism_modifier_reread_scale`).
    - Backing: `measurable` (scenario PS7)
    - Confidence: high

## Formulas

**F1, supported damage multiplier** (claim 9):

> *U* = ((*s* × *N* + 100) × 256) modulo 2³², interpreted as a non-negative integer.
> *M* = ⌊*U* / 100⌋, unsigned integer division truncating toward zero.
> *P* = the signed 32-bit interpretation of (*D*ₜ × *M*) modulo 2³².
> *D* = ⌊*P* / 256⌋, integer division rounded toward −∞.

All integer additions and multiplications here retain only the low 32 bits. When no product
wraps, this is simply *M* = ⌊(*s* × *N* + 100) × 256 / 100⌋ and
*D* = ⌊*D*ₜ × *M* / 256⌋. *D* is the area-damage input, not a defended health loss.

*s* is the held whole-number percentage, *N* the master's support count (1 to *N*ₘₐₓ) and *D*ₜ
the projectile's damage at detonation (claim 9). For example, *s* = 50 and *N* = 2 give
*M* = 512, which is ×2.

**F2, support distance** (claims 2 and 3):

> δ = trunc(√(Δx² + Δy² + Δz²)), in leptons.

Δx, Δy and Δz are the differences between the two buildings' positions, each the centre of the
building's top-left foundation cell at its height (glossary, *top-left cell*). These are integers
in leptons. Differences use signed 32-bit integer subtraction. The height difference is
converted through single precision, then widened to double. In the original's floating-point
round-toward-zero mode (S-0205), the sum is formed as (Δx² + Δz²) + Δy²
in double precision, followed by the square root, then truncation toward zero to an integer. Height counts, so a supporter on a cliff is farther away
than one at the master's level at the same plan distance.

**F3, overpowered** (claim 13): overpowered ⇔ *n* ≥ 3 ∨ (*n* ≥ 1 ∧ switched on ∧ *p* = 1).

**F4, held support modifier** (Import mapping): each read of a rules layer that has the general
section turns the held value *h* into *h*′ = trunc(*v* × 100), where *v* is a double.
- With the key present, *v* is the field's decimal value (S-0003 claims 6 and 7): the written
  number in single precision, times the double nearest to 0.01 when the text contains a percent
  sign.
- With the key absent, *v* = *h*.

The product *v* × 100 is one double multiplication in floating point in the original, rounded as
S-0003 claim 7's percent product is, and the result is then truncated toward zero to an integer.
So a written percentage whose ×0.01 product falls just below the exact value loses 1: "10%" gives
9, while "125%" gives 125. The held result is a signed 32-bit integer; values outside the
conversion domain follow S-0205's invalid-conversion rule.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Support beam thickness | 3 | pixels | claim 6; drawing only |
| Master beam thickness when supported | 5 (3 otherwise) | pixels | claim 9; drawing only |
| Unscaled damage multiplier | 256 | 1/256 | claim 9 |
| Chargers that overpower whatever the power | 3 | infantry units | claim 13 |
| Chargers that overpower when switched on at full power | 1 | infantry units | claim 13 |
| Cells scanned for a building to charge | 8 | cells | claim 14 |
| Supporters recruited per processing | 1 | buildings | claim 4 |
| Support modifier scale per read | 100 | – | F4 |

## State and ordering

**Master and supporter stages.** Each building of the prism type is *idle*, *support pending* or
*shot pending*.

| State | Event | Result |
|---|---|---|
| master idle, *N* < *N*ₘₐₓ | *can fire*; an eligible building exists | the nearest becomes *support pending*; *N* += 1; no shot |
| master idle | *can fire*; no eligible building, or *N* ≥ *N*ₘₐₓ | master *shot pending* (claim 7) |
| master *shot pending* | its delayed shot comes and launches a projectile | projectile scaled by F1 when *N* > 0; *N* ← 0; master idle; ordinary rearm |
| master *shot pending* | its delayed shot is dropped or launches nothing | master idle; *N* kept (claim 10) |
| master idle | attack processing with no target, or the target is dropped | *N* ← 0 |
| supporter *support pending* | its beam comes (claim 6) | beam drawn; rearm ← *R*ₛ; its own *N* ← 0; idle |
| supporter *support pending* | the master is destroyed | no change; the beam still comes |

**Charger and overpowered state** (overpowerable building).

| State | Event | Result |
|---|---|---|
| any | an infantry unit's electric-charge projectile detonates on it | that unit is on the list (once) |
| any | its turn | chargers no longer aiming at it are removed; overpowered ← F3 |
| any | a charger leaves play | removed from the list at once |

Observable consequences:
- Recruitment gives a bonus without waiting for an observed beam. Loss of a supporter after
  recruitment therefore cannot remove that contribution (claim 8).
- Under claim 4's assumptions, a master with *K* eligible supporters starts its delayed shot
  min(*K*, *N*ₘₐₓ) ticks later than it would alone. Its own delayed-fire time is unchanged.
- Two masters of one house share the pool. Except for the zero-delay window described below,
  a building recruited by one is not eligible for another recruitment until it has drawn its beam
  and its support rearm has run out. Very short support rearm can permit repeated recruitment
  of one building during a single master's recruitment period.
- The overpowered state changes only in the building's own turn. A change in power, or a charger
  that stops aiming, takes effect at that turn. A charger that leaves play stops counting for
  S-0405 claim 8 at once.

## Edge cases and quirks

- **`quirks.prism_support_ignores_operation`** (default on). Supporters need not be in operation
  (claim 2). A supporter that is switched off, under a pulse, under construction or being sold
  still supports a working master. (At low power the master itself is out of operation and does
  not attack, so low power alone shows nothing.) *Fixed:* a building that is not in operation
  (S-0405 claim 8) is not eligible.
- **`quirks.prism_count_survives_dropped_shot`** (default on). A dropped or projectile-less delayed
  shot keeps the support count, and the next shot gets the kept bonus on top of new supporters,
  with no beams for the kept part (claim 10). *Fixed:* the count becomes 0 whenever the delayed
  shot comes, fired or not.
- **`quirks.prism_zero_delay_double_count`** (default on). When the prism type's delayed-fire field
  is 0, a recruited supporter's delayed-fire countdown stays 0 until its turn. Another master of the
  same house processed before that turn finds it eligible and also counts it. One beam is drawn
  and both masters gain. *Fixed:* a building with a pending beam is not eligible. Stock data gives
  the prism type's image a nonzero field, so this does not arise in stock play.
- **`quirks.prism_modifier_reread_scale`** (default on). Every rules layer that has a general
  section is re-read with the held modifier as its default, and the result is multiplied by 100
  (F4), even when the layer lacks the key. A map with a general section but no support-modifier
  key therefore multiplies the support contribution by 100 before F1's integer quantisation,
  it does not multiply the unassisted contribution by 100. No stock multiplayer map and no
  stock game-mode file has a general section, so this is dormant in stock skirmish. *Fixed:* a layer without the key leaves
  the held value unchanged.
- **`quirks.stale_chargers_kept`** (default on). A building of a non-overpowerable type never
  removes chargers that stopped aiming at it (claim 12). Two such registrations keep it in
  operation's switch and house-power conditions through S-0405 claim 8 until the chargers leave play. Stock data gives no way to
  register on such a building, since the electric-charge weapon is chosen only against
  overpowerable targets (S-0308 row 11). *Fixed:* the same removal as for overpowerable types.
- The support reach is the master's slot-1 range. A master type with no slot-1 weapon has a reach
  of 0, so only a supporter at distance 0 qualifies. Normally placed buildings cannot share
  that position, so such a master fires alone after one processing.
- A master always fires slot 0. If weapon choice gave slot 1 (for example, an overpowered master
  of a type that is also overpowerable), the slot-0 weapon still fires (claim 7).
- A supporter's own weapons play no part. Its rearm comes from *R*ₛ and its beam from *L*ₛ, not
  from its weapons' fields.
- A vehicle with an electric-charge secondary goes through claim 14's scan and fires charge shots
  that do nothing (claim 11). Since it lacks claim 15's break-off search, it keeps firing at the
  building while enemies approach, until an order or another target change. Stock data has no
  such vehicle. Quirk `quirks.vehicle_charge_no_break_off` (default on). *Fixed:* vehicles break
  off as infantry do.

## Import mapping

| Source key | Concept | Parse form | Absent |
|---|---|---|---|
| Rules, `[General]`, `PrismType` | prism type (Terms) | building type name | none: no prism support |
| Rules, `[General]`, `PrismSupportModifier` | support modifier *s* | real with optional percent sign, then F4 | held value (F4); the engine's starting held value is 100 |
| Rules, `[General]`, `PrismSupportMax` | support cap *N*ₘₐₓ | integer | held value; 8 |
| Rules, `[General]`, `PrismSupportDelay` | support rearm *R*ₛ, ticks | integer | held value; 100 |
| Rules, `[General]`, `PrismSupportDuration` | beam duration *L*ₛ, ticks | integer | held value; 15 |
| Rules, `[General]`, `PrismSupportHeight` | not used by these support rules | integer, leptons | held value; 300 |
| Rules, prism type's section, `Secondary` (that weapon's `Range`) | support reach (Terms) | as S-0307 claim 13 | no slot-1 weapon: reach 0 |
| Art, building image, `DelayedFireDelay` | delayed-fire field *d* (claims 6 and 7) | integer, ticks | 0 |
| Art, building image, `IsAnimDelayedFire` | not read for the prism type (claim 7) | boolean | no |
| Rules, warhead section, `ElectricAssault` | electric-charge warhead (claims 11 and 14) | boolean | no |
| Rules, building type section, `Overpowerable` | overpowerable type (claims 12 to 14) | boolean | no |

The modifier is a percentage field (S-0003, Terms and claim 7) scaled by F4. The invented written value
"125%" becomes the held value 125, while a bare "125" becomes 12500.

## Measurement scenarios

Use tiny purpose-made skirmish maps, game speed index 2, 800 × 600, director and mods off,
except the named test data below. Use one enemy bystander computer player outside the observed
area, following the glossary's void rule. The observed area is the test buildings and the cells
within their weapon and support reach. Repeat each case three times. Record ticks, targets,
health, firing and beams through video and a read-only recorder; count random draws with the
random-state recorder where specified. Recorder extensions are proposals, not assumed existing
bench features. Never write running game state. Named test data is removed after each case.
For every case, a different recorded result from the stated expectation falsifies that part.

- **PS1 (claims 1, 4, 7 and 9).** Give house A master *P*, enemy house B a stationary target *T*
  within its primary range, and *K* idle own-house supporters behind *P*, unable to attack *T*.
  Run *K* = 0, 1, 2 and cap + 1. Order *P* to attack; record first-shot tick, recruitment counts,
  projectile damage at detonation and target health. Expect count min(*K*, cap), recruitment
  one per consecutive *can fire* processing, and F1's area-damage amount. Predict health loss
  by applying S-0303 and S-0302 to that amount, not by treating F1 as health loss directly.
  Repeat with named test-map overrides giving invented damage/modifier values that expose
  integer rounding, signed damage and 32-bit product wrap, and an in-flight-loss projectile.
  The supported shot must scale its remaining damage, not its original launch damage.
  Compare a different building type with identical weapons: it fires ordinarily, without links.
- **PS2 (claim 2).** Start from PS1 with two candidates. Separately make one candidate attack
  another enemy, belong to an allied house, have a rearm countdown running from an earlier
  shot, have a beam pending, be drained by a stock draining aircraft, or be destroyed before
  recruitment. Expect only the other to count. In separate runs switch one off, recruit it
  during its build-up, or order its sale and recruit before its removal: expect both to count.
  Record behaviour and recruitment ticks to distinguish current from merely queued sale.
  Pulse-disabled eligibility remains unmeasured: no stock pulse fixture is supplied here.
- **PS3 (claim 2, reach).** A named test-map override sets the secondary's range to the measured
  F2 separation of two normally placed buildings, then to one lepton less. Expect support
  only in the first case. Repeat across different terrain heights and with a missing secondary:
  expect F2 to include height and no normally placed supporter to qualify without the secondary.
- **PS4 (claim 3).** Use a named test-map override with cap 1, two candidates at unequal F2
  distance, then at equal F2 distance. Swap build order and repeat with capture-order reversed.
  Expect nearest in the unequal case, earliest acquired in a tie; record which draws the beam.
- **PS5 (claims 4 to 7).** Use three candidates, and another prism building outside the master's
  reach but within one candidate's reach. Expect three direct beams, no chained beam, and the
  master starting its own delay in the fourth consecutive *can fire* processing. Vary placement
  order to put supporters before and after the master in the update order; record all beam and
  shot ticks and compare claim 6's formula. Read rearm remaining immediately after each beam:
  expect exactly the support rearm field, with no rearm draw. Count down from that event,
  rather than timing the next shot, which also needs attack processing and its own delay.
  Use named test art/rules files to vary delayed-fire time, disable the animation-delayed-fire
  flag, give the secondary a visibly different shot, and make weapon choice select it. Expect
  the master still to delay and fire slot 0. Test delay 0 with two masters processed before a
  candidate's turn: expect one beam but both masters counting it. Also test delay 0 and support
  rearm 0 with one master and one candidate: expect repeated recruitment on later ticks up to
  the cap, not a distinct-supporter limit. Observe beam endpoints,
  house colour, duration and
  thickness; expect claims 6 and 9's values, with no damage, ammunition or burst use by support.
- **PS6 (claims 8 and 10).** Recruit support, then separately destroy, sell or capture a supporter
  before the master's shot: expect the recruited bonus unchanged. Destroy or sell the master
  instead: expect surviving supporters to beam towards its stored point and rearm normally.
  Move the target out of range early enough for attack processing to drop it: expect count 0,
  not a retained bonus on the next attack. Repeat by destroying the target before the next
  attack processing: expect the no-target reset. For the retained-count case use named test art/rules files
  with a long delayed shot and a slowly turning turret. Move a target sideways but keep it in
  range, timing movement so the final delayed-shot check fails for facing. Keep the target
  until another shot succeeds. Expect retained count, no replacement beams for that count,
  and F1 scaling on the successful shot. A run where attack processing drops the target does
  not test retention. The projectile-less variant is unmeasured: no isolated fixture here
  makes a launch fail while preserving that target and support count.
- **PS7 (claim 16 and F4).** Compare named test maps with no general section, a general section
  lacking the modifier, and one explicitly giving an invented modifier. Repeat PS1 with one
  supporter and record area-damage input before target defences, choosing a target that survives.
  Expect respectively the held modifier *s*, 100 × *s*, and the explicitly parsed F4 result,
  then F1 in each case. Do not expect total damage to grow exactly 100-fold: its unassisted
  part is not rescaled. Include the invented percent examples of F4 to distinguish rounding.
- **TC1 (claims 11 to 13).** Own-house infantry charge an overpowerable building *C*. Test 0, 1,
  2 and 3 distinct chargers, both switched on and off, at full and low power. Record registrations,
  *C*'s weapon choice and health. Expect F3 and no ordinary damage from charge shots. Two
  chargers bypass switch and house-power gates, not the other operation gates. Repeated hits
  by one infantry unit must never count as two. Order one charger away, and separately kill it:
  expect target-change pruning in *C*'s next turn, but leave-play removal at once. Compare ticks
  just before and after *C*'s turn to distinguish a cached overpowered state from list length.
- **TC2 (claim 14).** Put an infantry charger without a target in *guard*, then *area guard*,
  next to own-house and allied overpowerable buildings. Expect only an own-house building to
  be chosen automatically. With two own-house candidates in different adjacent cells, expect
  the one first in step-direction order. Reverse directions and repeat. Give an attack order
  to an allied building: expect charging, with no special charge cursor. Put an enemy where
  area guard acquires it first: expect that target rather than charging in that processing.
- **TC3 (claim 15).** While infantry charges automatically, bring an eligible enemy into range.
  Expect replacement at the next attack processing, followed by list pruning at *C*'s next
  turn. Repeat after an explicit target-setting order to the same building: the mark must clear,
  so that later approach must not trigger this special break-off search. Record random state:
  expect ordinary behaviour-delay draws but no additional targeting-timer draw for the search.
- **TC4 (claims 11, 12, 14 and 15, named test-map overrides).** Supply an infantry charge primary
  able to target non-overpowerable and enemy buildings, and a vehicle with a charge secondary.
  Expect infantry registration irrespective of building type or owner, no vehicle registration,
  and no ordinary area damage even when aimed at a non-building. Register two infantry units
  on a non-overpowerable building, then order them away: expect the registrations and operation
  gate bypass to persist until they leave play, but no overpowered state. The vehicle must
  automatically choose an own-house building and continue charging when an enemy approaches.
  Add configured charge-warhead debris/flash and a charge weapon's radiation field: expect
  those common effects despite the ordinary-area-damage bypass. Combine electric charge with
  mind control or charge planting separately: expect the higher-priority effect, not registration.

## Open questions

- How the support multiplier affects detonation side effects other than ordinary area damage.
  No general rule about those effects is claimed here.
- Which search-permission restrictions are intrinsic to claim 15's range search, as opposed
  to the ordinary passive-acquisition gates in S-0309 claims 2 and 3. The no-timer-gate and
  no-timer-draw rules are traced; a complete permission comparison remains untraced.
- The exact place of claim 14's scan within the *guard* processing of infantry with special guard
  states. It runs only when nothing earlier in that processing has taken over.
- Prism-type combinations with the SAM flag or installed upgrades. Those can enter special
  attack paths instead of recruitment. Stock skirmish prism types do not need those combinations,
  and their complete interaction remains untraced here.
