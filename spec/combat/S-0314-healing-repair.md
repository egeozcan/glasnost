---
id: S-0314
title: Healing and repair in combat (healers, self-healing, house-wide healing, repair depot, hospital, building repair)
topic: combat
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-12
measurements: []
requests: []
supersedes: []
---

# S-0314: Healing and repair in combat (healers, self-healing, house-wide healing, repair depot, hospital, building repair)

## Summary

How objects regain health: healers, self-healing (one object, or every infantry unit or vehicle
of a house), repair steps at a repair depot and a hospital, and building repair. This spec owns the
timings, the step sizes, the step cost and the house-wide healing. The arithmetic of a healing hit is
S-0302's, and the orders that send a unit to be healed are S-0213's and S-0214's.

## Terms

Shared terms used here are defined in the glossary: *house*, *infantry*, *vehicle*, *aircraft*,
*building*, *type*, *field*, *weapon*, *warhead*, *tick*, *tick number*, *credits*, *departure*,
*importer warning*, *quirk switch*, *round-toward-zero mode*, *local player*, *behaviour*,
*house skill level*, *AI house*, *neutral house*, *special house*, *computer-controlled*,
*human-controlled*, *available credits*, *credit balance*.
S-0205's notation applies: trunc, ⌈ ⌉, RZ₅₃, *single* and *double*. In addition:

- **Health, strength.** As S-0302 (Terms): current hit points, an integer from 0 to the strength
  given by the type's strength field.
- **Health ratio, full health.** As S-0214 (Terms, F2): health divided by strength in floating
  point. *Full health* is a health ratio of exactly 1; the original fixes this threshold at 1 and
  does not read it from data.
- **Healer.** As S-0214 (Terms): an armed object whose weapon it would use by default does negative
  damage. This is the sense of claim 1 and of the order rules. The automatic searches use another
  test, the *healer for acquisition* of S-0308 (claim 15): an object whose mean weapon damage is
  below 0 (claim 2).
- **Allied.** A house is allied with itself and with the houses its alliance state names (the
  glossary's *house*).
- **Self-healing type.** A type flag (Import mapping).
- **Organic.** A type flag (Import mapping). An organic vehicle is healed by the house-wide rule
  for infantry instead of the one for vehicles (claim 10).
- **Self-heal sources.** Two integer fields of a building type, the *infantry self-heal count* and
  the *vehicle self-heal count* (Import mapping). A house's *infantry self-heal total* and *vehicle
  self-heal total* are running totals that the buildings it places, captures and loses adjust
  (claim 10). Because a total is floored at 0 at each removal (claim 10), it depends on the order
  of those events, not only on which buildings the house owns, and is not always the sum of their
  counts (Edge cases, negative self-heal counts).
- **Repair step.** One restoration of health by a fixed amount, paid for by the owner of the object
  being repaired, the *paying house* (claim 3, and claim 8 for building repair). This is the event. The amount it restores is the *step size*,
  and what it costs is the *step cost*; both depend on the type (claims 4 and 5). The two
  game-wide amounts, in health points, are the *game-wide repair step* and the *game-wide infantry
  repair step* (Import mapping).
- **Money of a house.** The house's *available credits* (glossary; S-0404, claim 4 owns the sum).
  In a skirmish nothing puts resources into a house's own storage, so available credits equal the
  *credit balance* there (S-0404, claim 4), and the spec writes "available credits" throughout.
- **Depot's owner.** The house that owns the repair depot or hospital building. It is the
  *paying house* of a step when the repaired object is its own. The two are told apart only where
  a claim says so (claim 6).
- **Repair depot, hospital.** Building type flags (Import mapping). A unit enters a repair depot by
  docking on it and a hospital by entering it (S-0214, table I, rows e4 to e6 for the hospital).
- **Building repair.** The repair state of a building that its owner turns on and off with the
  repair order (S-0213, claim 13). It is separate from the repair steps of claims 3 to 7 (claim 8).
- **Terms owned elsewhere.** *Radar event* (S-0011, claim 32), *rally point* (S-0213, claim 20),
  *rate delay* and *processing* (S-0215, F1) are used as those specs define them.
- **Periods and counts.** Three game-wide rates, each a decimal number of game minutes converted at 900
  ticks per game minute (glossary *game minute*): the *building repair rate* *r*<sub>b</sub>, the
  *vehicle repair rate* *r*<sub>u</sub>, the *infantry repair rate* *r*<sub>i</sub> (Import
  mapping), and from them the building repair period *P*<sub>b</sub> and the vehicle and infantry
  repair counts *K*<sub>u</sub> and *K*<sub>i</sub> of Formulas F1. *D*, a depot's or hospital's
  processing delay, is the rate delay of the *repair* behaviour of S-0215 (F1).

## Behaviour

### Healers

1. **A healing hit.** A healer's shot delivers a negative amount. S-0302 (claims 5, 6 and 11) gives
   what it does: the amount is delivered unchanged, with no versus value, no falloff, no cap and no
   defence applied to it, to a target within 8 leptons of the point of detonation (to no other
   target), and it raises health by the amount's size, never above strength. The versus value of
   the warhead against the target's armour class plays no part in the amount. It decides only
   whether the object is a legal target (a value of exactly 0 makes it an illegal target, S-0304,
   claim 1) and whether an automatic search may pick it (S-0304, claim 4; claim 2 here). There is no separate
   "medic" or "mechanic" type flag: a healer that repairs vehicles but not infantry is a healer
   whose warhead's versus values are 0 for infantry armour classes, and the reverse.
   *Dropping the target.* When a healer's target stops being a legal healing target, for example
   because it has reached full health, is no longer of the right kind, or is gone, the firing
   check gives its *illegal* answer. S-0310 (claim 11) owns which targets give *illegal* for a
   healer vehicle and for a healer infantry unit, and what the answer does to a vehicle's target.
   What the unit then does is the attack behaviour of S-0215. This spec adds no rule to that.
   - Backing: `measurable` (S-0302 scenario D4 for the amount; scenario H1 here for the versus
     split; S-0310 scenario T10 for the *illegal* answer)
   - Confidence: high

2. **Which objects a healer picks by itself.** S-0309 (claim 6) owns this rule, and it is not
   repeated here: a searcher that is a *healer for acquisition* (Terms; S-0308, claim 15) looks for
   allied objects below full health of its own kind instead of enemies, with the landed-aircraft
   exception for a healer vehicle and a search radius of 512 leptons under *guard*; the versus
   value test of S-0304 (claim 4) applies as for every automatic search. The automatic searches use the healer for
   acquisition, not S-0214's healer of claim 1: a type with a primary weapon of damage −3 and a
   secondary weapon of damage +3 has a mean weapon damage of 0, so it is not a healer for
   acquisition and searches for enemies, although its primary weapon heals. An enemy object is
   never picked by a healer's automatic search.
   - Backing: `measurable` (scenario H2 here; S-0309 scenario A12)
   - Confidence: high (as S-0309, claim 6)

### Repair steps

3. **One repair step.** A repair step on an object, asked for by a repair depot or a hospital, has
   one of four outcomes. Building repair does not use them (claim 8):

   | Condition when the step is asked | Outcome |
   |---|---|
   | the object is at full health | nothing changes; the step reports *not needed* |
   | the paying house's available credits are less than the step cost | nothing changes, nothing is paid; the step reports *cannot pay* |
   | otherwise, and health plus the step size leaves the health ratio below 1 | the paying house pays the step cost, health rises by the step size; the step reports *more needed* |
   | otherwise | the paying house pays the step cost, health becomes the strength; the step reports *done* |

   A step has the outcome of the first row whose condition holds, so a full-health object whose
   paying house cannot pay reports *not needed*, not *cannot pay* (scenario H4, part b). A step cost of 0 is never refused for lack of money, even when the house has none. The paying house
   is the owner of the repaired object. Through the orders of S-0214 a vehicle enters only an own
   depot, so the paying house and the depot's owner are then the same house.
   - Backing: `measurable` (scenarios H3, H4 and H6)
   - Confidence: high

4. **Step size.** The step size of an infantry unit is the game-wide infantry repair step; for
   every other object it is the game-wide repair step. A step size below 1 counts as 1. In the
   original, building repair takes the game-wide repair step without this minimum; our engine applies
   the minimum there too (claim 8, Edge cases).
   - Backing: `measurable` (scenario H3, part a; scenario H6, part b)
   - Confidence: high

5. **Step cost.** The step cost of an infantry unit is always 0, whatever the repair percentage.
   For every other object it is the cost of Formulas F2: the type's cost field, divided by the
   number of steps that its strength holds, multiplied by the game-wide repair percentage,
   truncated, and at least 1 credit. The number of steps is always counted with the game-wide repair
   step, even for an object whose step size is another (our engine raises a game-wide repair step
   below 1 to 1 here too: F2, Edge cases, step cost with strength below the repair step). The type's cost is used without the house
   and country cost factors. For most types it is the cost field. For a building type it is the
   *adjusted cost* that S-0408 owns (claim 2 and F2: the deductions for a free unit and for the
   pad aircraft, and the order in which its floor at 0 applies). That adjusted cost can be negative,
   and the step cost's minimum of 1 credit then still applies.
   - Backing: `measurable` (scenario H3, parts a and c; scenario H5, part d; scenario H6, part a for the
     infantry cost of 0)
   - Confidence: high (the formula, the minimum, 0 for infantry, no house or country factor),
     medium (the free-unit adjustment and the pad-aircraft deduction, S-0408 claim 2)

### Repair depot, hospital and building repair

6. **Repair depot.** Write *K*<sub>u</sub> for the vehicle repair count and *D* for the depot's
   processing delay (Formulas F1). A repair depot treats a vehicle docked on it in two phases:
   - *Settling.* As soon as the vehicle has come to rest on the depot, the depot asks for one repair
     step on it at once (claim 3). On *not needed* the vehicle is sent away at once, as below, and no repair-complete announcement is
     made (unlike *not needed* in the working phase). On
     *cannot pay* nothing happens and the depot asks again at its next processing, *D* ticks later,
     for as long as the vehicle stays. On *more needed* or *done* the depot starts working.
   - *Working.* The second step falls due *D* + *K*<sub>u</sub> − 1 ticks after the settling step,
     and each later step *K*<sub>u</sub> ticks after the step before it. A step that falls due while
     the vehicle is moving is taken on the first tick on which the vehicle stands still, and the next
     step then falls due *K*<sub>u</sub> ticks after that one. The step's outcome:
     - *more needed:* the vehicle stays,
     - *done* or *not needed:* the local player hears the repair-complete announcement when the
       depot is theirs and the announcement is not suppressed (below), and the vehicle is sent
       away,
     - *cannot pay:* repairing stops; the depot's owner, when it is the local player and its
       available credits are exactly 0, hears the low-funds announcement. Whether the vehicle then
       stays docked or is sent away is not known (Open questions).
   *The repair-complete announcement.* It belongs to the depot's owner: it is heard only when that
   house is the local player. It is also a radar event of one kind that depots and hospitals share
   (claim 7), and like every radar event it is not shown, and so not announced, when an event of
   the same kind is already shown close by (S-0011, claim 32; the distance and how long an event
   counts belong to the interface specs). The low-funds announcement belongs to the depot's owner
   too: it is heard only when that house is the local player, and it is that house's available
   credits that must be exactly 0. It has no such suppression. The paying house of a step is the vehicle's owner
   (claim 3), and the two houses are the same one for every vehicle that can enter the depot
   through the orders of S-0214.
   A vehicle that the settling step brings to full health (*done*) therefore waits for one more
   step, which reports *not needed*, before it is sent away. A vehicle that is sent away moves to
   the depot's rally point when the depot has one (S-0213, claim 20), and otherwise to an exit cell
   that the depot picks next to itself (the structures specs own that choice); when there is no
   such cell its destination is not known (Open questions).
   *The original's return to the guard spot.* In the original, a vehicle whose owner is not
   human-controlled and that has a guard spot (S-0215, Terms) when it is sent away moves to that
   guard spot instead of the rally point or the exit cell, and the guard spot is cleared. In our
   engine this holds for the vehicles of the *neutral house* and the *special house*, which keep the
   original's rules for owners that are not human-controlled (glossary, *AI house*, *Player houses
   only*). It holds for no player house: the vehicles of an *AI house*, like a person's, follow the
   destination rule above, and our AI sends them on by ordinary orders.
   - Backing: `measurable` (scenario H4, parts a to f, for everything above except the next two
     statements). `unmeasured` for the return to the guard spot of a non-player house: no input of
     the measurement setup makes a vehicle of the original's neutral or special house, or of its
     computer player, dock at a depot in a way that can be timed and recorded, and a player house
     never takes this path. It is justified as a rule recorded from the original's behaviour that the
     measurement setup cannot reach. `unmeasured` also for a step that falls due while the docked vehicle is
     moving (taken on the first tick it stands still, the next step *K*<sub>u</sub> ticks after
     that one): no input of the setup is known that moves a docked vehicle without ending the
     docking. If H4 part g finds one, this statement becomes `measurable`.
   - Confidence: high (one step per count of *K*<sub>u</sub>, the cost, the end on *done*); medium
     (the settling step, the phase *D* + *K*<sub>u</sub> − 1, that repairing stops after *cannot
     pay*, the owner conditions of the two announcements, that the settling *not needed* makes no
     announcement, and that *not needed* is tested before *cannot pay*), and low (the destination without a rally
     point; what happens to the vehicle after *cannot pay*; the suppression of the repair-complete
     announcement; the return to the guard spot; the moving step)

7. **Hospital.** An infantry unit inside a hospital gets repair steps at a cost of 0, on the
   timing of the depot's working phase with the infantry repair count *K*<sub>i</sub> (Formulas
   F1): the hospital takes no step when the unit enters, the first step falls due *D* +
   *K*<sub>i</sub> − 1 ticks after the hospital's first processing after the unit entered, and each
   later step *K*<sub>i</sub> ticks after the step before it. On *more needed* the unit stays
   inside; on *done* it is put out, and the local player, when the hospital is theirs, hears the
   repair-complete sound and, unless it is suppressed, the repair-complete announcement (claim 6:
   one radar-event kind shared with the depot, with its suppression); on *not needed* it is put out
   without them. *Not needed* is rare here: S-0214 (table I, row e4) refuses the order that sends a
   full-health infantry unit into a hospital, so it arises only when a unit reaches full health
   while inside, which is not known to happen (Open questions). *Cannot pay* cannot happen here: the infantry step cost is always 0 and a cost of 0
   is never refused (claim 3). The armory of S-0311 (claim 12) uses
   the same infantry repair count, but counts it in processings instead of ticks (S-0311, Formulas
   F6).
   - Backing: `measurable` (scenario H6, parts a to c). `unmeasured` for *not needed* in a hospital
     (put out without the sound and announcement): no input of the measurement setup is known that
     brings a full-health infantry unit into a hospital, and whether a unit inside can heal by
     another rule is open; H6 part d makes the attempt.
   - Confidence: high (the count *K*<sub>i</sub> between steps and the step); medium (the phase
     of the first step, the sound and the owner condition); low (the suppression of the
     announcement; *not needed* in a hospital)

8. **Building repair.** This claim replaces claim 3 for buildings: a building's repair has no
   *not needed* and no *cannot pay* outcome. While a building's repair is on, it takes one step at
   each tick whose tick number is a multiple of the *building repair period* *P*<sub>b</sub>
   (Formulas F1), with this outcome:

   | Condition when the step is taken | Outcome |
   |---|---|
   | the owner's available credits are less than the step cost | nothing is paid, health is unchanged, repair turns off |
   | otherwise, and health plus the game-wide repair step is below the strength | the owner pays the step cost; health rises by the game-wide repair step; repair stays on |
   | otherwise | the owner pays the step cost; health becomes the strength; repair turns off |

   The step is the game-wide repair step as it stands. Unlike claim 4, the original does not raise
   a value below 1 to 1 here; our engine does (Edge cases). The owner of the building pays and
   is the house whose credits are tested. Full health does not prevent a step: a building whose
   repair is on at full health pays one step at the next multiple of *P*<sub>b</sub>, its health
   stays at the strength, and then repair turns off.

   *Automatic repair and sale.* The original also turns a building's repair on, and sells
   buildings, by itself. That rule is gated by the *house skill level* (glossary): it can apply
   only to a house whose level is at least the game-wide repair-and-sell threshold (Import
   mapping; absent-key default 3). In a skirmish the levels are these:
   - every human-controlled house has level 0,
   - an *AI house* has level 0 in our engine (glossary), where the original gives its computer
     player the rules' maximum level (absent-key default 5), which passes the gate at the default
     threshold: the original's computer player repairs and sells by itself, and ours does not
     (Edge cases, the original's computer player), and
   - the *neutral house* and the *special house* keep level 0, since the original gives the
     maximum only to the houses of computer players when the houses are set up (S-0209, claim 11); they keep the original's rules in our engine, so their level is 0 here too.

   So with a threshold above 0 no house of a skirmish in our engine turns repair on or sells by
   itself: only the repair order does. The rule's further conditions (the house's money against a
   reserve, a random draw for selling, per-building states) are not specified here. With a
   threshold of 0 or below the gate passes for every house, the neutral and special houses
   included. What our engine does then is a question for the project lead (Edge cases; Questions
   for the project lead), and scenario H9 records what the original does.
   - Backing: `measurable` (scenarios H5 and H9, runs 1 to 3 for the levels of a human house, of
     the neutral house and of the special house)
   - Confidence: high (the step, its timing, its cost and the charge at full health); medium (that
     the automatic rule is gated by the house skill level as stated; the level 0 of the neutral and
     special houses, which S-0215 lists as an open question)

### Self-healing

9. **Self-healing objects.** An object whose type is self-healing, or which has the self-heal
   veterancy ability (S-0311, claim 3), gains 1 health at each tick whose tick number is a multiple
   of the building repair period *P*<sub>b</sub> (Formulas F1), provided its health is not 0 and
   not equal to its strength. This holds for every kind of object, and it uses the same game-wide rate as
   building repair. It is free.
   - Backing: `measurable` (scenario H7)
   - Confidence: high

10. **House-wide self-healing.** Each building type has an infantry self-heal count and a vehicle
    self-heal count (both 0 unless given). A house keeps one running total for each, adjusted by
    events: when a building is placed on the map for the house, or captured by it, the building's
    counts are added to the totals; when it is removed, sold, destroyed or captured away, they are
    subtracted and a total that is then below 0 is set to 0. The floor applies at each removal and
    not to any sum of the buildings owned, and an addition is never floored; with
    `quirks.negative_self_heal_counts` turned off a negative count is read as 0 instead, so no
    addition lowers a total (Edge cases, negative self-heal counts). The building's power state plays no part. Then:
    - every vehicle that is not organic gains *a*<sub>v</sub> × *T*<sub>v</sub> health, but no more
      than it lacks, at each tick whose tick number is a multiple of the vehicle self-heal interval
      *n*<sub>v</sub>, when its house's vehicle self-heal total *T*<sub>v</sub> is above 0,
    - every infantry unit, and every organic vehicle, gains *a*<sub>i</sub> × *T*<sub>i</sub> in the
      same way at multiples of the infantry self-heal interval *n*<sub>i</sub>, with the infantry
      total *T*<sub>i</sub>,
    - aircraft and buildings are not healed this way.

    *a*<sub>v</sub>, *a*<sub>i</sub>, *n*<sub>v</sub> and *n*<sub>i</sub> are game-wide fields
    (Import mapping, intervals in ticks). An object at full health or at 0 health gains nothing.
    Several buildings of a house add up: two such buildings double the amount, not the frequency.
    When claim 9 and this claim both apply to an object in one tick, the two gains add and the sum
    is capped at the strength: an object with health *H* at the start of the tick (0 < *H* < *S*)
    and strength *S* ends the tick with health min(*H* + 1 + *a* × *T*, *S*) (Formulas F3, which
    also gives our engine's treatment of a negative *a*).
    - Backing: `measurable` (scenario H8, parts a to d)
    - Confidence: high (amounts, intervals, the kinds, the stacking); medium (the events that change
      the totals, the floor at removal, and that power plays no part)

11. **Healing on resources has no effect.** The type flag and the game-wide value for healing on
    resource cells, and the resource-heal veterancy ability (S-0311, claim 5), change nothing in the
    original, and our engine gives them no effect. The rules key for a healer search radius is
    not read by the original either (Import mapping).
    - Backing: `measurable` (scenario H10 here; S-0311 scenario X11 for the ability)
    - Confidence: medium

## Formulas

**F1, periods and counts.** *r*<sub>b</sub>, *r*<sub>u</sub>, *r*<sub>i</sub> are the game-wide
rates in game minutes (decimals; S-0003, claims 6 and 7, held as doubles):

> *P*<sub>b</sub> = trunc(RZ₅₃(*r*<sub>b</sub> × 900)). Unit: ticks. Used as "tick number is a
> multiple of *P*<sub>b</sub>" (claims 8 and 9).
>
> *K* = the smallest positive integer with RZ₅₃(*r* × 900) ≤ *K*, an exact comparison, for
> *r* = *r*<sub>u</sub> (giving *K*<sub>u</sub>) or *r*<sub>i</sub> (giving *K*<sub>i</sub>). For
> a product above 1 this is ⌈RZ₅₃(*r* × 900)⌉; for a product of 1 or less it is 1. Unit: ticks
> (claims 6 and 7).
>
> *D* = the rate delay of the *repair* behaviour (S-0215, Formulas F1, without random draw),
> counted as 1 when it is 1 or less. Unit: ticks. It is the delay between a depot's or hospital's
> processings outside the working count (claims 6 and 7) and between an armory's processings
> (S-0311).

A rate written without a percent sign is read as a single-precision value and held in a double
(the same value, widened), so its product with 900 is exact and does not depend on the rounding
mode (S-0205, claim 9). Example: with the absent-key default rate 0.016 (a
double; the same decimal written in data is the single just above it), 900 times the rate is just
below 14.4 for the double and 14.4000006… for the single, so in both cases *P*<sub>b</sub> = 14
ticks and *K* = 15 ticks. A rate written as 0.1 is the single 0.100000001…, so 900 times it is
90.0000013…, giving *P*<sub>b</sub> = 90 but *K* = 91. A rate of 0.125, exact in single precision,
gives 112.5: *P*<sub>b</sub> = 112, *K* = 113. The two timings differ by design of the original.

**F2, step cost** (claim 5, not infantry). *C* is the type's cost field (for a building type, the adjusted cost of S-0408 F2), *S* its strength, *R*ₛ the
game-wide repair step (all integers), *p* the repair percentage (double):

> *q* = *S* ÷ *R*ₛ, *c* = *C* ÷ *q* (integer quotients truncated toward zero),
> step cost = max(1, trunc(RZ₅₃(*c* × *p*))). Unit: credits.

*Our engine* (departure, Edge cases, step cost with strength below the repair step): *R*ₛ′ = max(1,
*R*ₛ), *q* = max(1, *S* ÷ *R*ₛ′), then the same *c* and step cost. It equals the original's
whenever 1 ≤ *R*ₛ ≤ *S*.

The rounding follows the *round-toward-zero mode* of S-0205 (`quirks.chop_rounding`). Example: *C* =
900, *S* = 600, *R*ₛ = 10, *p* = 0.25 gives *q* = 60, *c* = 15, step cost 3.

**F3, health after a tick of self-healing** (claims 9 and 10): all integers, exact arithmetic, so
no rounding applies. *H* is the object's health at the start of the tick and *S* its strength.
*g*<sub>9</sub> = 1 when claim 9 applies to the object in that tick, and 0 otherwise.
*g*<sub>10</sub> = *a*′ × *T* when claim 10 applies to it in that tick (the tick number is a
multiple of the object's self-heal interval *n* and its house's total *T* is above 0), and 0
otherwise. *a*′ is the amount *a* of claim 10, or 0 when *a* is negative (Edge cases, negative
amounts). For 0 < *H* < *S*:

> health at the end of the tick = min(*H* + *g*<sub>9</sub> + *g*<sub>10</sub>, *S*). Unit: health
> points.

An object with *H* = 0 or *H* = *S* at the start of the tick gains nothing from either claim.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Ticks per game minute | 900 | ticks per game minute | F1 (glossary) |
| Self-heal gain | 1 | health points | Claim 9 |
| Full-health ratio | 1 | health ratio | Claim 3; not read from data |
| Healing reach | 8 | leptons | S-0302, claim 6 |
| Minimum step size | 1 | health points | Claim 4 |
| Minimum step cost (not infantry) | 1 | credits | Claim 5 |
| Infantry step cost | 0 | credits | Claim 5 |
| Repair step, absent-key default | 5 | health points | Claim 4 |
| Infantry repair step, absent-key default | 1 | health points | Claim 4 |
| Repair percentage, absent-key default | 0.25 | fraction | Claim 5 |
| Building, vehicle and infantry repair rates, absent-key default | 0.016 | game minutes | F1 |
| Steps at a depot when a vehicle settles | 1 | steps | Claim 6 |
| Infantry and vehicle self-heal intervals, absent-key default | 1000 | ticks | Claim 10 |
| Infantry and vehicle self-heal amounts, absent-key default | 1 | health points per source | Claim 10 |
| Infantry and vehicle self-heal counts of a building type, absent-key default | 0 | sources | Claim 10 |
| Repair-and-sell threshold, absent-key default | 3 | house skill level | Claim 8 |

## State and ordering

| State | Event | Result |
|---|---|---|
| vehicle comes to rest on a depot | — | settling step at once (claim 3) |
| settling | *not needed* | sent away |
| settling | *cannot pay* | asks again after *D* ticks |
| settling | *more needed* or *done* | working; the next step falls due *D* + *K*<sub>u</sub> − 1 ticks later |
| working | a step falls due, vehicle standing still | repair step; the next falls due *K*<sub>u</sub> ticks later |
| working | a step falls due, vehicle moving | the step is taken on the first tick the vehicle stands still |
| working | step reports *more needed* | stays |
| working | step reports *done* or *not needed* | sent away (rally point or exit cell); repair-complete announcement as claim 6 says |
| working | step reports *cannot pay* | repairing stops; low-funds announcement as claim 6 says |
| infantry inside a hospital | a step falls due (claim 7) | repair step at cost 0 |
| same | *more needed* | stays |
| same | *done* or *not needed* | put out; announcements as claim 7 says |
| building repair on | tick number a multiple of *P*<sub>b</sub> | step (claim 8's table) |
| building repair on | health reaches strength, or money is short | repair off |

An object that claims 9 and 10 both heal in one tick ends it with the health that F3 gives: the two
gains add, and the sum is capped at the strength. The place of healing among the other work of a tick is S-0203's.
Because claims 8, 9 and
10 use tick numbers, all objects of a game heal on the same ticks, not on ticks counted from when
each began healing.

## Edge cases and quirks

- **Periods below one tick** (departure). A building repair period *P*<sub>b</sub> of 0 (a rate
  whose size is below 1/900 of a game minute) makes the original stop with a program error at the
  first self-healing or building-repair test, and so does a self-heal interval of 0 for the
  house-wide rule (S-0205 claim 3, integer division by zero). Our engine treats such a period or
  interval as 1 tick and the importer warns. A negative period or interval is no error: "a multiple
  of" then behaves as for its absolute value, reproduced, with an importer warning. A depot's or
  hospital's rate whose product RZ₅₃(*r* × 900) is 0 or less is no error either: *K* = 1 (F1),
  reproduced.
- **Building repair step below 1** (departure). A game-wide repair step of 0 makes the original stop
  with a program error when it computes the step cost (next edge case); a negative one is used as it
  is by building repair, which then lowers the building's health and still charges at least 1
  credit, without any death check, so health can reach 0 or below, outside the health domain
  (S-0302, Terms), and the result is undefined. Our engine raises a repair step below 1 to 1 for building repair
  as claim 4 does for the other steps, and the importer warns.
- **Step cost with strength below the repair step** (departure). When the type's strength is below
  the repair step, the number of steps *q* of F2 is 0 and the original stops with a program error
  the first time it computes the step cost, as it does for a repair step of 0. Our engine takes
  *q* = 1 whenever *q* would be 0, uses a game-wide repair step below 1 as 1 in F2 too (previous
  edge case), and the importer warns. The second part changes observable costs, not only the cases
  where the original stops. For a negative game-wide repair step whose size is at most the
  strength, the original's depot step cost is well defined: 1 credit whenever the cost and the
  repair percentage are not negative, through the maximum of F2. Our engine's is the quotient
  *C* ÷ *S*, truncated, times the percentage, truncated, and at least 1. The two can differ there.
  For example *C* = 4000, *S* = 500, *R*ₛ = −5 and *p* = 0.25 give 1 credit in the original and 2
  in our engine (H3 part d). It is a departure of every step cost except an infantry unit's, which
  is always 0, and not only of building repair.
- **Self-healing uses the building repair rate.** Claim 9's period is the building repair period.
  The self-heal intervals of claim 10 do not apply to it. This is the original's design and needs no
  switch.
- **Building repair at full health and the depot's extra step** (rules, not quirks). The charge of
  one step for a building repaired at full health (claim 8) and the extra *not needed* step of a
  depot after *done* (claim 6) are deliberate rules of this spec, reproduced as the original's
  design, with no switch: neither damages data or can be set off to any gain, and both are
  observable in H5 part b and H4 part c.
- **Negative step costs.** A negative cost field or repair percentage gives a step cost of at least
  1 all the same (the maximum of F2). Reproduced; the importer warns for a negative cost field.
- **Negative self-heal counts** (quirk `quirks.negative_self_heal_counts`, default: the original's
  behaviour). The counts are integers, so a building type can carry a negative one. The original
  then adds the negative count to the total without a floor, and floors only at removal (claim 10).
  Example, infantry counts: place a building of count 3 (total 3), then one of count −5 (total −2),
  remove the first (−5, floored to 0), remove the second (0 + 5 = 5). The house now heals its
  infantry although it owns no source. A total that is not above 0 heals nothing. With the switch
  turned off, a negative count is read as 0 (as a negative amount is, next item), so no total is ever
  below 0 and a house with no source heals nothing. The importer warns for a negative count with the
  switch either way. This is a quirk, not a rule, because a house healing without any source is a
  side effect of the missing floor that no data would rely on by design.
- **Negative self-heal amounts** (departure). A negative amount *a* gives the original a negative
  gain: health falls, with no check for death, so health can reach 0 or below, outside the health
  domain (S-0302, Terms), and the result is undefined. Our engine treats a negative amount as 0 (F3)
  and the importer warns.
- **Automatic repair and sale with a threshold of 0 or below** (proposed design departure, pending
  the lead's decision in Questions for the project lead). The original's gate of claim 8 then
  passes for every house, human-controlled ones and the neutral and special houses included, and
  the rule's further conditions are not specified. Proposed: our engine turns repair on only by
  the repair order and never sells by itself, for every house including the neutral and special
  ones, and the importer warns when the threshold is 0 or below.
- **The original's computer player** also starts building repair by itself, passes the gate of
  claim 8 at the default threshold, and has a difficulty delay for doing so; those rules belong to
  its script system, which is out of scope. Our AI uses the repair order.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, `[General]`, `RepairRate` | building repair rate *r*<sub>b</sub> (claims 8, 9) | decimal (S-0003, claims 6 and 7), double | 0.016; a layer without the key keeps the value in force |
| Rules, `[General]`, `URepairRate` | vehicle repair rate *r*<sub>u</sub> (claim 6) | decimal, double | 0.016; keeps |
| Rules, `[General]`, `IRepairRate` | infantry repair rate *r*<sub>i</sub> (claim 7; S-0311 claim 12) | decimal, double | 0.016; keeps |
| Rules, `[General]`, `RepairStep` | repair step (claims 4, 5, 8) | integer | 5; keeps |
| Rules, `[General]`, `IRepairStep` | infantry repair step (claim 4) | integer | 1; keeps |
| Rules, `[General]`, `RepairPercent` | repair percentage *p* (F2) | decimal, a percent sign allowed (S-0003 claim 7), double | 0.25; keeps |
| Rules, `[General]`, `SelfHealInfantryFrames`, `SelfHealUnitFrames` | infantry and vehicle self-heal intervals *n*<sub>i</sub>, *n*<sub>v</sub> (claim 10) | integer, ticks | 1000; keeps |
| Rules, `[General]`, `SelfHealInfantryAmount`, `SelfHealUnitAmount` | amounts *a*<sub>i</sub>, *a*<sub>v</sub> per source (claim 10) | integer; a negative value is read as 0 (Edge cases) | 1; keeps |
| Rules, `[General]`, `TiberiumHeal`; type section, `TiberiumHeal` | nothing (claim 11) | decimal; boolean | — |
| Rules, `[General]`, `HealScanRadius` | nothing: the original does not read this key (claim 11) | — | — |
| Rules, type section, `SelfHealing` | self-healing type (claim 9) | boolean | no |
| Rules, type section, `Organic` | organic (claim 10) | boolean | yes for infantry types, no for vehicle, aircraft and building types |
| Rules, building section, `InfantryGainSelfHeal`, `UnitsGainSelfHeal` | infantry and vehicle self-heal counts (claim 10) | integer (not a boolean: a value of 2 counts twice; a negative value is kept, or read as 0 with `quirks.negative_self_heal_counts` off, Edge cases) | 0 |
| Rules, building section, `UnitRepair` | repair depot (claim 6) | boolean | no |
| Rules, building section, `Hospital` | hospital (claim 7) | boolean | no |
| Rules, `[IQ]`, `RepairSell` | house skill level threshold for automatic building repair and sale (claim 8) | integer | 3; keeps |
| Rules, `[Repair]`, `Rate` (S-0215) | processing delay *D* (F1) | owned by S-0215 | S-0215 |
| Rules, type section, `Cost`, `Strength` | *C*, *S* (F2) | integer | owned by the production and type specs |

## Measurement scenarios

All scenarios run under the measurement setup with named test files: a flat test map whose override
sections set the game-wide fields and define the test types named. Health is read from a recorder
of health values or from the health bar of a selected object, credits from the sidebar, timings in
ticks with a recorder of the tick number. Any game speed (no timing here is speed-normalised).

**H1: healer kinds and amount (claim 1).** *Part a (kinds):* a healer infantry type whose warhead has 100 % against an
infantry armour class and 0 % against a vehicle armour class, and a damaged infantry unit and a
damaged vehicle of those classes, both own. Order the healer onto each. *Expected:* the infantry unit
heals. The vehicle gives no heal cursor and is not healed. *Falsified if:* the vehicle heals.
*Part b (amount):* give the warhead 50 % against the infantry class and a weapon damage of −20 and
read the health after one hit on the damaged infantry unit. *Expected:* +20. *Falsified if:* +10 or
any other amount (that would mean the versus value scales a healing hit).

**H2: automatic healing targets (claim 2).** Part a runs with S-0309 scenario A12; the two must give the
same results.
- *Setup (part a):* a healer infantry unit in *guard*, with a weapon range of 5 cells. Around it, all damaged
  unless said: an own infantry unit at 1 cell, an allied infantry unit at 1.5 cells, an enemy infantry
  unit at 1 cell, an own vehicle at 1 cell (with a versus value of 100 % for its class), an own
  infantry unit at full health at 1 cell, and an own damaged infantry unit at 3 cells.
- *Observable:* which objects the healer heals by itself within 600 ticks, in order.
- *Expected:* the own and the allied infantry at 1 and 1.5 cells; never the enemy, the vehicle, the
  unit at full health, or the unit at 3 cells (outside 2 cells). A second run in *area guard*
  heals the unit at 3 cells too.
- *Falsified if:* the enemy, the vehicle or the full-health unit is healed; or the unit at 3 cells
  is healed under *guard*.
- *Part b (healer vehicle):* a healer vehicle in *area guard* whose warhead has 100 % against the
  armour classes used. Around it, all damaged and own: a vehicle, an infantry unit, an aircraft
  landed on open ground and an aircraft on a landing pad. *Expected:* it heals the vehicle and the
  aircraft on open ground by itself, never the infantry unit or the aircraft on the pad.
  *Falsified if:* the infantry unit or the aircraft on the pad is healed, or the aircraft on open
  ground is not.
- *Part c (healer for acquisition):* an infantry type with a primary weapon of damage −3 and a
  secondary weapon of damage +3 (mean 0), in *guard*, with a damaged own infantry unit and an enemy
  infantry unit within range. *Expected:* it never heals the own unit by itself and attacks the
  enemy. Repeat with a primary of −5 and a secondary of +3 (mean −1): it heals the own unit by
  itself and never attacks the enemy. *Falsified if:* the first type heals by itself, or the
  second one does not.

**H3: step size and cost (claims 3 to 5).** *Part a (steps):* a damaged own vehicle of a test type with cost 900 and
strength 600, at health 300, docks on a repair depot; repair step 10, repair percentage 25 %, *repair* behaviour
rate 0.015625 (*D* = 14), the house has 1000 credits. *Observable:* health and credits after each step. *Expected:* +10 health and −3 credits per
step; the first step comes as soon as the vehicle has come to rest; the last step brings health to
600 exactly.
*Part b (credits):* a second run with 2 credits: no step happens while the credits stay at 2; when
the house gets 1 more credit, a step follows within *D* = 14 ticks. *Falsified if:* other amounts,
or no step after the credit.
*Part c (cost factors):* repeat the first run with the house's cost factor and the country's cost
factor both set to 2 in the test files. *Expected:* still −3 credits per step. *Falsified if:* the cost per step changes.
*Part d (step below 1, depot):* repeat the first run with a test type of cost 4000 and strength
500, and the game-wide repair step −5 (a case where the depot's cost differs between the original
and our engine, Edge cases). *Expected in the original:* each step adds 1 health (claim 4) and
costs 1 credit (the maximum of F2 with a negative number of steps). Our engine would charge 2
credits (F2, our engine); that departure is not measured here. *Falsified if:* the original's step
adds another amount or costs 2 credits or more (that would mean the original guards a negative
step as our engine does).

**H4: depot timing (claim 6, F1).** *Part a (timing):* as H3 part a with a vehicle repair rate of 0.125 game minutes and a
*repair* behaviour rate of 0.015625 (*D* = 14). *Observable:* the tick of each step, counted from the
tick the vehicle comes to rest. *Expected:* a step at once, the second 126 ticks later
(14 + 113 − 1), then one every 113 ticks. With the vehicle repair rate absent (0.016): every 15
ticks after the second. *Falsified if:* 112 ticks between later steps, or 14 at the absent rate
(that would mean the depot uses the truncated period of building repair), or no step at rest. The
tick on which the vehicle leaves the depot after the step that brings it to full health is recorded
too.
- *Part b (full health):* a vehicle at full health docks. *Expected:* it is sent away on the tick it
  comes to rest, with no step, no payment and no repair-complete announcement (the local player
  owns the depot). Repeat with the paying house at 0 credits. *Expected:* the same, sent away on the
  tick it comes to rest. *Falsified if:* a charge, a wait of *D* ticks or more, or an
  announcement; or, in the repeat, the vehicle stays and the depot asks again *D* ticks later
  (that would mean *cannot pay* takes precedence over *not needed*).
- *Part c (done, then not needed):* a vehicle one step short of full health docks. *Expected:* the
  settling step brings it to full health and pays once; it leaves on the second step, 126 ticks
  later, with no further payment. *Falsified if:* it leaves at once, or a second payment is taken.
- *Part d (destination):* run with a rally point set on the depot and without. *Expected:* the
  vehicle moves to the rally point when there is one, and otherwise to a cell next to the depot.
  *Falsified if:* it stays on the depot or goes elsewhere.
- *Part e (cannot pay while working):* the house has credits for exactly one step (the step cost
  is 3); the vehicle is two steps short. The local player owns the depot and the vehicle.
  *Expected:* the settling step is paid, leaving 0 credits; no step is paid at the second step
  tick, and the low-funds announcement is heard once, when the failed step is taken at 0 credits.
  Record whether the vehicle stays docked or leaves. After the failed step give the house credits
  for two more steps. If the vehicle stays docked, watch it for *D* + 2 × *K*<sub>u</sub> ticks:
  *expected:* no further step is taken while it stays docked. In a second run the house has 1
  credit left after the settling step (still below the step cost): *expected:* the announcement is
  not heard. *Falsified if:* a step is paid without money, a later step is taken while the
  vehicle stays docked, the announcement is missing at 0 credits, or it is heard at 1 credit.
- *Part f (the repair-complete announcement):* the local player owns the depot and the vehicle.
  *Run 1:* one vehicle is repaired to full health and sent away. *Expected:* the announcement is
  heard once, when the step that reports *done* (or, when it comes from the settling step, *not
  needed* on the next one) is taken. *Run 2:* two own depots on adjacent cells, a vehicle on each
  that is repaired to full health within 10 ticks of the other's. *Expected:* the first announcement
  is heard and the second is not, and both vehicles are sent away. *Run 3:* as run 2 with the
  depots 30 cells apart. *Expected:* both are heard. *Run 4:* a depot of a computer-played house
  allied with the local player, in view, repairing its own vehicle. *Expected:* no announcement for
  its completion. *Falsified if:* run 2 is silent for the first or sounds for the second, run 3 is
  silent for either, or run 4 is heard (the depot's owner is not the local player). Record the
  largest distance and the longest gap at which run 2's second announcement stays silent: they are
  the suppression's distance and duration, for the interface specs.
- *Part g (the moving step; the return to the guard spot):* both are `unmeasured` (claim 6). The
  attempt: after the settling step, order the docked vehicle to move on the tick a step falls due,
  and read the tick of its next health rise. *Expected:* a health rise on the first tick it stands
  still, the next *K*<sub>u</sub> ticks later; if the order ends the docking instead, the statement
  stays `unmeasured`.

**H5: building repair (claim 8).** Building repair rate 0.016 (the absent-key default, so *P*<sub>b</sub> =
14), a building type with cost 1000 and strength 1000, repair step 10, repair percentage 25 % (step
cost 2), the house has 1000 credits.
- *Part a (steps):* the own building at health 500, repair turned on at a tick that is no
  multiple of 14. *Observable:* the tick numbers of the health rises, the credits. *Expected:* the
  first rise at the next multiple of 14, rises only on multiples of 14, each +10 at −2 credits.
  Repair turns off at health 1000. A second run with 5 credits at the start: two steps are
  paid (2 + 2), the third multiple of 14 finds 1 credit, nothing is paid and repair turns off.
  *Falsified if:* a rise on another tick, an amount other than +10 or −2, or a step paid with less
  money than its cost.
- *Part b (full health):* the building at health 1000, repair turned on. *Expected:* at the next
  multiple of 14 the credits fall by 2, health stays 1000, and repair turns off; no further
  charge. *Falsified if:* nothing is charged (a full-health test exists), or a second charge
  follows.
- *Part c (step below 1):* the building at health 500, repair step −5. *Expected:* each step lowers the health by 5 and charges
  1 credit (the original; our engine departs, Edge cases). *Falsified if:* health rises, or the
  step is taken as 1.
- *Part d (free unit):* a building type with cost 2000 that comes with a free unit of cost 800,
  strength 1000, repair step 10, repair percentage 25 %. *Expected:* each step costs 3 credits
  (cost 2000 less 800, so *q* = 100, *c* = 12, 12 × 0.25 = 3). *Falsified if:* each step costs 5
  (the free unit is not deducted).

**H6: hospital (claim 7).** *Part a (steps):* an own infantry unit at health 10 of strength 100 enters an own
hospital; infantry repair step 25, infantry repair rate 0.125, *repair* behaviour rate 0.015625
(*D* = 14), credits 0. *Expected:* no step at entry; the first step about 126 ticks after entry
(within one processing for the phase), then one every 113 ticks without paying (35, 60, 85), and it
comes out after the fourth step, at 100.
*Falsified if:* it pays, comes out after one step, or the interval between steps is not 113.
- *Part b (minimum step):* infantry repair step 0. *Expected:* each step restores 1 health.
  *Falsified if:* health does not rise, or rises by another amount.
- *Part c (sound and announcement):* when the unit comes out at full health and the local player
  owns the hospital. *Expected:* the repair-complete sound and announcement are heard. Two own
  hospitals on adjacent cells, a unit leaving each within 10 ticks of the other: both sounds are
  heard, only the first announcement (claim 6, suppression; with the hospitals 30 cells apart, both
  announcements). A hospital of another house: neither. *Falsified if:* either is heard for another
  house's hospital, the sound is missing for the second adjacent hospital, or the second adjacent
  announcement is heard.

- *Part d (not needed inside, an attempt):* claim 7's *not needed* is `unmeasured`. A self-healing
  infantry type at 1 health point below full is ordered into an own hospital (legal, since it is
  hurt). *Observable:* its health after entry, and whether it is put out at its first step (about
  126 ticks after entry) with no sound and no announcement. *Expected:* it reaches full health
  inside through claim 9 before that step and is put out at the step without the sound or
  announcement. If its health does not rise inside the hospital, the statement stays `unmeasured`
  and the observation answers the open question on healing inside.

**H7: self-healing (claim 9).** A damaged own vehicle of a self-healing type and a damaged own
building of a self-healing type, building repair rate 0.016 (the absent-key default). *Observable:* tick numbers of the health
rises. *Expected:* +1 for each on tick numbers that are multiples of 14, until full. A third object,
a vehicle with the self-heal ability on its veteran list placed with map veterancy 100, heals the
same way; at veterancy 0 it does not. *Falsified if:* a rise happens on another tick or by another
amount, the objects keep rising at full health, the veteran vehicle at veterancy 0 heals, or the
veteran one at 100 does not.

**H8: house-wide healing (claim 10).** *Part a (amounts):* own damaged infantry, a damaged non-organic vehicle, a damaged
organic vehicle and a damaged aircraft (landed). One own building with infantry self-heal count 1
and vehicle self-heal count 1; intervals 40 and 60 ticks, amounts 10 and 3. *Expected:* the infantry
and the organic vehicle gain 10 on tick numbers that are multiples of 40; the other vehicle 3 on
multiples of 60; the aircraft nothing. Add a second such building: the gains become 20 and 6. Sell
both: no more gains. Repeat with the buildings unpowered (power plant removed): same gains.
*Falsified if:* frequency rather than amount doubles, the aircraft heals, or power changes it.
- *Part b (capture):* capture one such building with an engineer. *Expected:* the gains of the
  former owner's objects stop and those of the captor's start, with the building's counts.
  Then destroy one such building of the captor's: its gains stop with it. *Falsified if:* the former
  owner keeps healing, the captor does not heal, or a destroyed building's gains continue.
- *Part c (negative counts):* infantry self-heal counts, with test types of count 3 and −5 (one
  building each, placed in the order of the example in Edge cases, negative self-heal counts), amount
  1, interval 40. *Observable:* whether a damaged infantry unit gains health at multiples of 40 after
  each placement and removal. *Expected:* it gains after the first placement (total 3), not after the
  second (−2), not after the removal of the first (0), and gains 5 per interval after the removal of
  the second (5). *Falsified if:* it gains with the total at −2, or never gains at the end (that
  would mean a floored sum). With the switch off (the count −5 read as 0), the unit gains after the
  first placement and after the second (total 3), and not after the removal of the first (0) or of
  the second (0).
- *Part d (both gains in one tick):* a damaged own non-organic vehicle of a self-healing type, one
  building with vehicle count 1, amount 2, and the vehicle interval equal to the building repair
  period. At 10 below full health. *Expected:* on the first common tick it gains 3 (1 and 2). At 2
  below full health it becomes full, and never rises above the strength. *Falsified if:* it gains
  another amount, or ends above the strength.

**H9: the skill-level gate of automatic building repair (claim 8).** A skirmish on the test map; the
human house has 100 000 credits and two damaged buildings of a repairable type, and gives no repair
order; no other house can attack. *Observable:* the buildings' health over 3000 ticks. *Run 1*
(threshold at the absent-key default 3): *Expected:* it never rises. *Run 2* (the repair-and-sell
threshold set to 0): *Expected:* repair starts by itself on at least one building, and the credits
fall by the step cost at each rise. *Run 3 (neutral and special houses):* as run 1, with two
damaged buildings of the repairable type pre-placed by the map for the neutral house and two for
the special house (each house has 100 000 credits as far as the setup can give them; S-0009, claim
27 for the owners of pre-placed objects). *Expected:* none of the four rises or is sold over 3000
ticks. *Run 4:* as run 3 with the threshold 0. *Expected:* record whether a building of either
house starts repair or is sold. *Falsified if:* run 1 shows a rise (the gate does not exclude a
human house, and claim 8 needs a new revision), run 2 shows none (the gate is not the stated one),
or run 3 shows a rise or a sale (a neutral or the special house has a skill level at least the
threshold, contrary to S-0209, claim 11, and claim 8 needs a new revision).

**H10: healing on resources and the healer search radius key (claim 11).** S-0311 scenario X11
covers the veterancy ability and, in the same run, the type flag; it does not set the game-wide
value or the search radius key, which this scenario does.
- *Part a (type flag and game-wide value):* named test files give a vehicle test type the
  resource-heal type flag and set the game-wide resource-heal value to a positive value. The type is
  not self-healing and has no veterancy abilities. The house owns no building with a self-heal count
  and no depot. *Setup:* a damaged own vehicle of that type stands on a resource cell.
  *Observable:* its health over 3000 ticks. *Expected:* it never rises. *Falsified if:* it rises.
- *Part b (search radius key):* repeat H2 part a twice, with the healer search radius key set to 10
  and then to 2560. Whichever the key's unit, cells or leptons, one of the two values gives a radius
  of 10 cells (10 cells, or 2560 leptons), far beyond the 2 cells of *guard*. *Expected:* the
  same results as H2 part a (under *guard*, the unit at 3 cells is never healed). *Falsified if:*
  either run differs from H2 part a, in particular if the unit at 3 cells is healed under *guard*.

## Open questions

- The exact phase of the second depot step and the first hospital step (±1 tick), and whether a
  depot's settling step can be retried before *D* ticks; scenarios H4 and H6 record them.
- Where a vehicle goes when the depot has no rally point and no exit cell is found (claim 6).
- Whether a unit inside a hospital can reach full health by another rule, so that *not needed* can
  arise there (claim 7, H6 part d).
- Whether a vehicle stays docked or is sent away after *cannot pay* in the working phase (H4,
  part e).
- Whether passengers inside a transport (not a hospital) self-heal: the original is believed not to
  heal them while they are inside. This question covers open-topped transports too, whose
  passengers are in play and take turns (glossary, *in play*); no measurement has settled it for them.
- How aircraft are repaired on their landing pads (the same repair step is believed to apply, at a
  timing not measured); a future aircraft spec.
- The other conditions of the original's automatic building repair and sale, for a threshold of 0 or
  below (claim 8, H9).
- The distance and duration of the repeat suppression of the repair-complete announcement (claim 6,
  H4 part f), which the interface specs own.
- Whether the neutral and special houses really keep skill level 0 in a skirmish (claim 8, H9 run
  3); S-0215 lists the same question.

## Questions for the project lead

- **Automatic repair and sale at a skill threshold of 0 or below (claim 8).** The original's gate
  passes for every house when the threshold is 0 or below, the neutral and special houses
  included, and what the rule then does depends on conditions this spec does not specify (H9 runs 2
  and 4 record it). Proposal: our engine never starts repair or a sale by itself, whatever the
  threshold, for any house including those two (a departure, for them, from the lead's decision of 2026-10-10 that they keep the
  original's rules; glossary, *AI house*, since the rule's conditions are unknown), and the importer warns for such a threshold. The alternative
  is to reproduce the original's rule once H9 and a further measurement specify its conditions.
  Which one?
