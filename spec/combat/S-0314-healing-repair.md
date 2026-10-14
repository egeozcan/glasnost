---
id: S-0314
title: Healing and repair in combat (healers, self-healing, house-wide healing, repair depot, hospital, building repair)
topic: combat
revision: 5
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
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
Only skirmish is covered; network games and campaigns are out of scope. For automatic building
repair and sale this revision specifies only the skill-level gate, not the remaining eligibility
conditions or their outcomes when the gate passes.

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

1. **A healing hit.** An ordinary projectile shot from a negative-damage healing weapon delivers
   a negative amount. Subject to S-0302's hit-stopping rules (claim 8) and its undemolishable-building
   exception (claim 7), S-0302 (claims 5, 6 and 11) gives what it does: the amount is unchanged,
   with no versus scaling, falloff or defence multiplier applied to it, at a distance strictly
   less than 8 leptons from the point of detonation. At 8 leptons or farther it changes no health.
   A negative delivered amount bypasses the maximum-damage cap even when the maximum is negative
   (H1 part c; S-0302 F3, cases iii and iv). This differs from a positive delivered amount made
   negative by its versus value: that amount still takes the cap (S-0302 F3, case vi).
   S-0302 claim 6's sentence about a negative maximum needs this explicit qualification; it must
   not extend the cap to the negative-delivered-amount cases. A negative result raises health by its magnitude, never above strength. The versus value of
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
   - Backing: `measurable` (S-0302 D4 for the amount, D5 for the building exception and D26
     for hit-stopping rules, H1 for the versus split and negative-maximum distinction,
     and S-0310 T10 for the *illegal* answer).
     The absence of separate healer-role type flags is `unmeasured`: observing H1's versus
     selection cannot establish the absence of every other possible data flag.
   - Confidence: high

2. **Which objects a healer picks by itself.** S-0309 (claim 6) owns this rule, and it is not
   repeated in full here: an infantry unit or vehicle that is a *healer for acquisition*
   (Terms; S-0308, claim 15) looks for allied damaged objects instead of enemies. Infantry searches
   for infantry; vehicles search for the objects S-0309 claim 5 includes in the vehicle search
   class, including landed aircraft on open ground. The ordinary candidate rules and their
   exceptions remain S-0309's, including its unmeasured aircraft exceptions. The search radius
   under *guard* is 512 leptons; the versus
   value test of S-0304 (claim 4) applies as for every automatic search. The automatic searches use the healer for
   acquisition, not S-0214's healer of claim 1: an infantry type with a primary weapon of damage −3 and a
   secondary weapon of damage +3 has a mean weapon damage of 0, so it is not a healer for
   acquisition and searches for enemies, although its primary weapon heals. An enemy object is
   never picked by an infantry or vehicle healer for acquisition. Building healers do not get
   that allied-only search modifier; what they pick, including whether they can pick an enemy,
   remains open under S-0309 claim 6.
   - Backing: `measurable` for the infantry and vehicle cases (H2; S-0309 A12).
     Building-healer acquisition is `unmeasured`: the building's healing-weapon firing check
     is not established, so no expected candidate outcome can be supplied (S-0309 claim 6).
     The aircraft exceptions inherit S-0309 claim 6's stated unmeasured limitations.
   - Confidence: high (infantry and vehicle main rule); low (building healer, as S-0309 claim 6)

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
   paying house cannot pay reports *not needed*, not *cannot pay* (scenario H4, part b). A step cost of 0 is not refused at non-negative available credits, including 0. The
   affordability comparison still applies if available credits are negative. The paying house
   is the owner of the repaired object. Through the orders of S-0214 a vehicle enters only an own
   depot, so the paying house and the depot's owner are then the same house.
   - Backing: `measurable` at non-negative available credits (H3, H4 and H6). The affordability
     test at a negative balance is `unmeasured`: no scenario here arranges a hospital entry at
     such a balance, and vehicles have a positive step cost so cannot isolate the zero-cost case.
   - Confidence: high

4. **Step size.** The step size of an infantry unit is the game-wide infantry repair step; for
   every other object it is the game-wide repair step. A step size below 1 counts as 1. In the
   original, building repair takes the game-wide repair step without this minimum; our engine applies
   the minimum there too (claim 8, Edge cases).
   - Backing: `measurable` for depot and hospital steps (H3 parts a and d; H6 part b).
     Aircraft repair steps are `unmeasured`: pad repair timing and a controllable step setup
     remain open, so no scenario here isolates an aircraft step.
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
   - Backing: `measurable` (scenario H3, parts a and c; scenario H5, parts d and e; scenario H6, part a for the
     infantry cost of 0; H11 for signed inputs and F2's truncations). The same formula for
     an aircraft repair step is `unmeasured`: no scenario here isolates such a step while
     landing-pad repair timing remains open.
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
   (claim 7), and it is not shown, and so not announced, when an event of
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
     statements and the non-local-owner announcement conditions). The latter are `unmeasured`:
     the other-owner attempt in H4 part f needs the original's computer player to dock a vehicle,
     which normal player input cannot require; failure to dock is not evidence about sound.
     `unmeasured` for the return to the guard spot of a non-player house: no input of
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
   while inside, which is not known to happen (Open questions). *Cannot pay* cannot happen here at non-negative available credits: the infantry step cost
   is 0 (claim 3). A negative balance is not exercised by H6; that case is `unmeasured` because
   no scenario here can arrange hospital entry for an infantry unit at such a balance. The armory of S-0311 (claim 12) uses
   the same infantry repair count, but counts it in processings instead of ticks (S-0311, Formulas
   F6).
   - Backing: `measurable` (H6 parts a to c for own hospitals). Absence of feedback for another
     owner's hospital is `unmeasured`: this scenario cannot order that owner's infantry to enter,
     and spontaneous entry is not a reproducible setup. `unmeasured` for *not needed* in a hospital
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
     threshold. The computer player's automatic outcomes are `unmeasured` here: H9 controls
     no computer-owned building's state, so cannot test the further conditions; this is an
     original-only note, not an exemption for our AI (Edge cases), and
   - the *neutral house* and the *special house* keep level 0, since the original gives the
     maximum only to the houses of computer players when the houses are set up (S-0209, claim 11); they keep the original's rules in our engine, so their level is 0 here too.

   With a threshold above 0 the gate excludes every level-0 skirmish house. With a threshold of
   0 or below the gate passes for every level-0 house, including human-controlled houses, AI
   houses, the neutral house and the special house. Passing the gate is necessary, not sufficient:
   it does not promise either a repair or a sale. The further conditions (available credits
   against a reserve, random choices for selling and per-building states) remain an open
   behavioural question, not a licence to disable the rule. This revision specifies no complete
   automatic-start rule for the gate-passing case. The existing house policy remains unchanged:
   AI houses follow human-controlled unit rules, and the neutral and special houses retain the
   original's non-human rules (S-0215, Houses in our engine). There is no threshold-specific
   departure or importer warning. H9 distinguishes the gate's exclusion from exploratory
   observations when it passes. S-0408 claim 1 must limit its assertion that non-player houses
   never sell automatically to the positive-threshold exclusion; claim 13 must point here for
   the gate only, not promise complete automatic outcomes.
   - Backing: `measurable` for periodic repair (H5) and the exclusion at a positive threshold
     (H9 runs 1 and 3). House levels are `measurable` through S-0412 claim 3's K10 and S-0209 C7.
     The exact absent threshold and the gate comparison in the gate-passing case are
     `unmeasured` (H12 explains the default's limitation): without a specified
     setup satisfying the further conditions, absence of repair or sale cannot distinguish a
     failed gate from later ineligibility; H9 runs 2 and 4 are exploratory, not confirmation.
   - Confidence: high (periodic repair and the skirmish house levels, S-0412 claim 3); medium
     (the automatic rule's skill-level comparison)

### Self-healing

9. **Self-healing objects.** An object whose type is self-healing, or which has the self-heal
   veterancy ability (S-0311, claim 3), gains 1 health at each tick whose tick number is a multiple
   of the building repair period *P*<sub>b</sub> (Formulas F1), provided its health is not 0 and
   not equal to its strength. This holds for infantry, vehicles, aircraft and buildings while
   they take their ordinary object updates, and uses the same game-wide rate as building repair.
   It is free. Objects out of play do not take these updates (S-0203); healing inside transports
   and hospitals remains the separate open question below.
   - Backing: `measurable` for living objects taking updates (H7). The health-0 exclusion is
     `unmeasured`: no scenario here keeps a zero-health object in an ordinary living update
     without destruction handling, which would confound the observation.
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
    and strength *S*, with no other health-changing event in that tick, ends it with health
    min(*H* + 1 + *a* × *T*, *S*) (Formulas F3, which
    also gives our engine's treatment of a negative *a*).
    - Backing: `measurable` (H8 parts a to d, including the building-kind exclusion). The
      health-0 exclusion is `unmeasured` for the same destruction-handling reason as claim 9.
    - Confidence: high (amounts, intervals, the kinds, the stacking); medium (the events that change
      the totals, the floor at removal, and that power plays no part)

11. **Healing on resources has no effect.** The type flag and the game-wide value for healing on
    resource cells, and the resource-heal veterancy ability (S-0311, claim 5), change nothing in the
    original, and our engine gives them no effect. The rules key for a healer search radius
    has no observable effect on the search radius either (Import mapping).
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

F1's boundary and decimal examples are `measurable` in H11, confidence high. F2's integer
quotients and final truncation are `measurable` in H3 and H11, confidence high.

**F3, health after a tick of self-healing** (claims 9 and 10): all integers, exact arithmetic, so
no rounding applies. This formula assumes no other health-changing event during that tick.
*H* is the object's health at the start of the tick and *S* its strength.
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
Claims 8 to 10 use the global tick number: objects sharing a period or interval have the same
scheduled ticks, not schedules counted from when each began healing. Different intervals need
not coincide. Depot and hospital steps instead use claim 6's and claim 7's entry-relative timing.

## Edge cases and quirks

Original arithmetic edge claims below are `measurable` in H11 (and the named H3, H4, H5 and H8
parts), confidence high unless their owning claim gives another confidence. Undefined outcomes
outside the health domain are `unmeasured` for H11's stated reason. Our departures and quirk-off
alternatives are design rules, not claims of oracle measurements.

- **Periods below one tick** (departure). A building repair period *P*<sub>b</sub> of 0 (a rate
  whose size is below 1/900 of a game minute) makes the original stop with a program error at the
  first applicable self-healing or building-repair test, and so does a self-heal interval of 0 for the
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
- **Automatic repair and sale with a threshold of 0 or below.** The gate alone does not disable
  the rule for any level-0 house (claim 8). No threshold-specific alternative, warning or new
  quirk is specified. The remaining eligibility conditions are open; this spec must not be used
  to infer that neutral or special houses never start repair or sell automatically.
- **The original's computer player** passes claim 8's gate at the absent-key default maximum
  level and threshold. Its strategy for repair or sale, including difficulty-related pacing,
  is not specified here. Our AI can use ordinary repair and sell orders. This does not remove
  any skill-gated unit rule that also reaches human-controlled houses at a lower threshold,
  nor the retained original rules of neutral and special houses.

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
| Rules, `[General]`, `HealScanRadius` | nothing: no search-radius effect (claim 11) | ignored | — |
| Rules, type section, `SelfHealing` | self-healing type (claim 9) | boolean | no |
| Rules, type section, `Organic` | organic (claim 10) | boolean | yes for infantry types, no for vehicle, aircraft and building types |
| Rules, building section, `InfantryGainSelfHeal`, `UnitsGainSelfHeal` | infantry and vehicle self-heal counts (claim 10) | integer (not a boolean: a value of 2 counts twice; a negative value is kept, or read as 0 with `quirks.negative_self_heal_counts` off, Edge cases) | 0 |
| Rules, building section, `UnitRepair` | repair depot (claim 6) | boolean | no |
| Rules, building section, `Hospital` | hospital (claim 7) | boolean | no |
| Rules, `[IQ]`, `RepairSell` | house skill level threshold for automatic building repair and sale (claim 8) | integer | 3; keeps |
| Rules, `[Repair]`, `Rate` (S-0215) | processing delay *D* (F1) | owned by S-0215 | S-0215 |
| Rules, type section, `Cost`, `Strength` | *C*, *S* (F2) | integer | owned by the production and type specs |

## Measurement scenarios

All scenarios run in skirmish under the glossary's measurement setup, director and mods off,
with named test files: a flat test map whose override sections set the game-wide fields and
define the test types named. Use game speed index 2 and 800 × 600 resolution; repeat each part
three times unless it gives another count. One bystander computer player starts in an isolated
area farthest from the test objects. The observed area contains all named test objects; apply
its glossary void rule. There is no combat in that area except a hit a part explicitly requests.
Disable both kinds of self-healing except where a part tests them, and use a positive automatic
repair-and-sell threshold except in H9. Record exact health, credit balance and tick numbers
with a read-only recorder; use scripted input for exact-tick orders where needed. Record sounds
and announcements separately. No timing here is speed-normalised.

**H1: healer kinds and amount (claim 1).** *Part a (kinds):* a healer infantry type whose warhead has 100 % against an
infantry armour class and 0 % against a vehicle armour class, and a damaged infantry unit and a
damaged vehicle of those classes, both own. Order the healer onto each. *Expected:* the infantry unit
heals. The vehicle gives no heal cursor and is not healed. *Falsified if:* the vehicle heals,
or the infantry cannot be ordered healed or does not heal.
*Part b (amount):* give the warhead 50 % against the infantry class and a weapon damage of −20 and
read the health after one hit on the damaged infantry unit. *Expected:* +20. *Falsified if:* +10 or
any other amount (that would mean the versus value scales a healing hit).
*Part c (negative delivered amount versus negative cap):* repeat part b on an own test infantry
unit of strength 1000 at health 500, with maximum damage −30. Fire one ordinary projectile hit
of damage −20 at distance 0. *Expected:* health 520, not 530. Repeat with maximum damage 0:
health is still 520. In a separate run put a second damaged own infantry unit in the warhead's
2-cell spread, more than 8 leptons from the impact point, and use maximum damage −30 again.
*Expected:* the directly hit unit gains 20, the second gains nothing. *Falsified if:* the directly
hit unit gains any other amount, or the second changes health. For comparison, S-0302 D3 part c
uses positive delivered damage with a negative maximum and expects healing from the cap; it
is a different case, not a prediction for a healing weapon.

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
- *Procedure:* leave the healer under the named behaviour with no further orders for 600
  ticks in each part; record target choices as well as health changes. Use unarmed test enemies
  so they do not interrupt the observations. Repeat the stated behaviour and weapon variants.
- *Falsified if:* the enemy, vehicle or full-health unit is picked for healing; the own or allied
  injured infantry in reach is never healed; the unit at 3 cells is healed under *guard*; or that
  unit is not healed in the *area guard* repeat.
- *Part b (healer vehicle):* a healer vehicle in *area guard* whose warhead has 100 % against the
  armour classes used and a projectile that is anti-air and anti-ground, as in S-0309 A12 c.
  Around it, all damaged and own: a vehicle, an infantry unit, an aircraft
  landed on open ground and an aircraft on a landing pad. *Expected:* it heals the vehicle and the
  aircraft on open ground by itself, never the infantry unit or the aircraft on the pad.
  *Falsified if:* the infantry or pad aircraft is picked for healing, or the vehicle or open-ground
  aircraft is not healed.
- *Part c (healer for acquisition):* an infantry type with a primary weapon of damage −3 and a
  secondary weapon of damage +3 (mean 0), in *guard*, with a damaged own infantry unit and an enemy
  infantry unit within range. *Expected:* it never heals the own unit by itself and selects the
  enemy as its target. Repeat with a primary of −5 and a secondary of +3 (mean −1): it picks
  and heals the own unit and never picks the enemy. *Falsified if:* the first type picks the
  own unit or fails to pick the enemy, or the second fails to pick and heal the own unit or
  picks the enemy. The sign of health change on an enemy hit is not the acquisition observable.

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
adds another amount or costs anything other than 1 credit, including 2 (that would mean the
original guards a negative step as our engine does).

**H4: depot timing (claim 6, F1).** *Part a (timing):* as H3 part a with a vehicle repair rate of 0.125 game minutes and a
*repair* behaviour rate of 0.015625 (*D* = 14). *Observable:* the tick of each step, counted from the
tick the vehicle comes to rest. *Expected:* a step at once, the second 126 ticks later
(14 + 113 − 1), then one every 113 ticks. With the vehicle repair rate absent (0.016): every 15
ticks after the second. *Falsified if:* no step at rest, a second step other than 126 ticks
later, or any later interval other than 113 ticks (15 at the absent rate). Intervals of 112 or
14 would mean the depot uses the truncated building period. The
tick on which the vehicle leaves the depot after the step that brings it to full health is recorded
too.
- *Part b (full health):* a vehicle at full health docks. *Expected:* it is sent away on the tick it
  comes to rest, with no step, no payment and no repair-complete announcement (the local player
  owns the depot). Repeat with the paying house at 0 credits. *Expected:* the same, sent away on the
  tick it comes to rest. *Falsified if:* a charge, departure on any later tick, or an
  announcement; or, in the repeat, the vehicle stays and the depot asks again *D* ticks later
  (that would mean *cannot pay* takes precedence over *not needed*).
- *Part c (done, then not needed):* a vehicle one step short of full health docks. *Expected:* the
  settling step brings it to full health and pays once; it leaves on the second step, 126 ticks
  later, with no further payment. *Falsified if:* it leaves on any other tick, or a second
  payment is taken.
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
  its completion. Run 4 is exploratory: if no vehicle docks, it supplies no observation and the
  non-local-owner condition remains `unmeasured` (claim 6).
  *Falsified if:* run 1 is silent or announces at another step, run 2 is silent for the first
  or sounds for the second, run 3 is silent for either, or, if completion is actually observed,
  run 4 is heard (the depot's owner is not the local player). Record the
  largest distance and the longest gap at which run 2's second announcement stays silent: they are
  the suppression's distance and duration, for the interface specs.
- *Part g (the moving step; the return to the guard spot):* both are `unmeasured` (claim 6). The
  attempt: after the settling step, order the docked vehicle to move on the tick a step falls due,
  and read the tick of its next health rise. *Expected:* a health rise on the first tick it stands
  still, the next *K*<sub>u</sub> ticks later; if the order ends the docking instead, the statement
  stays `unmeasured`. If docking persists through movement, a rise while moving, a delayed
  rise after stopping or a later interval other than *K*<sub>u</sub> falsifies the rule.

**H5: building repair (claim 8).** Building repair rate 0.016 (the absent-key default, so *P*<sub>b</sub> =
14), a building type with cost 1000 and strength 1000, repair step 10, repair percentage 25 % (step
cost 2), the house has 1000 credits.
- *Part a (steps):* the own building at health 500, repair turned on at a tick that is no
  multiple of 14. *Observable:* the tick numbers of the health rises, the credits. *Expected:* the
  first rise at the next multiple of 14, rises only on multiples of 14, each +10 at −2 credits.
  Repair turns off at health 1000. Repeat starting at health 995: the last step adds only 5,
  still charges 2 and turns repair off; any other gain, payment or later step falsifies the cap.
  A second run with 5 credits at the start: two steps are
  paid (2 + 2), the third multiple of 14 finds 1 credit, nothing is paid and repair turns off.
  *Falsified if:* a rise on another tick, an amount other than +10 or −2, or a step paid with less
  money than its cost.
- *Part b (full health):* the building at health 1000, repair turned on. *Expected:* at the next
  multiple of 14 the credits fall by 2, health stays 1000, and repair turns off; no further
  charge. *Falsified if:* nothing is charged (a full-health test exists), or a second charge
  follows.
- *Part c (step below 1):* the building at health 500, repair step −5. *Expected:* each step lowers the health by 5 and charges
  1 credit (the original; our engine departs, Edge cases). *Falsified if:* any gain other than
  −5, any cost other than 1 or a different repair tick.
- *Part d (free unit):* a building type with cost 2000 that comes with a free unit of cost 800,
  strength 1000, repair step 10, repair percentage 25 %. *Expected:* each step costs 3 credits
  (cost 2000 less 800, so *q* = 100, *c* = 12, 12 × 0.25 = 3). *Falsified if:* any cost other
  than 3, including 5 (the free unit is not deducted).
- *Part e (pad aircraft):* a building type with cost 2000, strength 1000, no free unit and two
  pad aircraft types, each of cost 300. Turn the separate-aircraft setting off and make this
  building the first dock of the first aircraft, as S-0408 claim 2 requires.
  Repair step 10 and percentage 25 %. Damage the building and turn repair on.
  *Expected:* 4 credits per step, from adjusted cost 1700, quotient 100, integer cost per step
  17 and truncated percentage product 4. *Falsified if:* any other cost, including 5.
  Repeat with both pad aircraft costs 2400, leaving adjusted cost −400: *expected:* 1 credit
  per step from the minimum; any other cost falsifies that minimum.

**H6: hospital (claim 7).** *Part a (steps):* an own infantry unit at health 10 of strength 100 enters an own
hospital; infantry repair step 25, infantry repair rate 0.125, *repair* behaviour rate 0.015625
(*D* = 14), credits 0. Record entry, the hospital's first repair processing afterward, each
health rise and exit. *Expected:* no step at entry; the first step 126 ticks after that first
processing, then one every 113 ticks without paying (35, 60, 85), and it comes out on the fourth
step at 100. Entry alone does not fix the processing phase.
*Falsified if:* any payment, wrong gain, step at entry, different first-step delay, interval
other than 113, health above 100 or exit on a tick other than the fourth step.
- *Part b (minimum step):* infantry repair step 0. *Expected:* each step restores 1 health.
  *Falsified if:* health does not rise, or rises by another amount.
- *Part c (sound and announcement):* when the unit comes out at full health and the local player
  owns the hospital. *Expected:* the repair-complete sound and announcement are heard. Two own
  hospitals on adjacent cells, a unit leaving each within 10 ticks of the other: both sounds are
  heard, only the first announcement (claim 6, suppression; with the hospitals 30 cells apart, both
  announcements). *Falsified if:* an own completion lacks either feedback in isolation, the
  sound is missing for the second adjacent hospital, the second adjacent announcement is heard,
  or either distant announcement is missing. The other-owner condition is `unmeasured` as
  claim 7 explains; a spontaneously observed completion at another house's hospital would
  falsify it if either feedback is heard, but this part does not promise such a completion.

- *Part d (not needed inside, an attempt):* claim 7's *not needed* is `unmeasured`. A self-healing
  infantry type at 1 health point below full is ordered into an own hospital (legal, since it is
  hurt). *Observable:* its health after entry, and whether it is put out at its first step
  (126 ticks after the first hospital processing) with no sound or announcement. Try to find
  a run in which it reaches full health inside before that step. *Conditional expected result:*
  if full health is reached inside, it is put out at the step without sound or announcement. If full health inside is observed, ejection at another time, a sound or an
  announcement falsifies the conditional rule. If its health does not rise inside the hospital,
  the statement stays `unmeasured`
  and the observation answers the open question on healing inside.

**H7: self-healing (claim 9).** A damaged own infantry unit, vehicle, landed aircraft and
building, each of a self-healing type, building repair rate 0.016 (the absent-key default). *Observable:* tick numbers of the health
rises and the credit balance. *Expected:* +1 for each on tick numbers that are multiples of 14,
until full, with no payment. Stagger placement by several ticks: health rises must still coincide
on the global multiples. Otherwise identical non-self-healing controls of each kind never rise.
Another object,
a vehicle with the self-heal ability on its veteran list placed with map veterancy 100, heals the
same way; at veterancy 0 it does not. *Falsified if:* a rise happens on another tick or by another
amount, a charge occurs, schedules shift with placement, a control heals, the objects keep
rising at full health, the veteran vehicle at veterancy 0 heals, or the veteran one at 100 does not.

**H8: house-wide healing (claim 10).** *Part a (amounts):* own damaged infantry, a damaged non-organic vehicle, a damaged
organic vehicle, a damaged aircraft (landed) and a damaged building, none individually
self-healing. One own building with infantry self-heal count 1
and vehicle self-heal count 1; intervals 40 and 60 ticks, amounts 10 and 3. *Expected:* the infantry
and the organic vehicle gain 10 on tick numbers that are multiples of 40; the other vehicle 3 on
multiples of 60; the aircraft and damaged building nothing. Add a second such building: the gains become 20 and 6. Sell
both: no more gains. Repeat with the buildings unpowered (power plant removed): same gains.
*Falsified if:* any amount or tick differs, frequency rather than amount doubles, the aircraft
or damaged building heals, healing continues after both sources are sold, or power changes it.
Continue until the infantry and both vehicles reach full health: *expected:* no increase above
strength and no charge; either falsifies the rule.
- *Part b (capture):* capture one such building with an engineer. *Expected:* the gains of the
  former owner's objects stop and those of the captor's start, with the building's counts.
  Then destroy one such building of the captor's: its gains stop with it. *Falsified if:* the former
  owner keeps healing, the captor does not heal, or a destroyed building's gains continue.
- *Part c (negative counts):* infantry self-heal counts, with test types of count 3 and −5 (one
  building each, placed in the order of the example in Edge cases, negative self-heal counts), amount
  1, interval 40. *Observable:* whether a damaged infantry unit gains health at multiples of 40 after
  each placement and removal. *Expected:* it gains after the first placement (total 3), not after the
  second (−2), not after the removal of the first (0), and gains 5 per interval after the removal of
  the second (5). *Falsified if:* any stage's gain differs from 3, 0, 0 and 5 respectively,
  or a gain occurs outside a multiple of 40 (a missing final gain would mean a floored sum). With the switch off (the count −5 read as 0), the unit gains after the
  first placement and after the second (total 3), and not after the removal of the first (0) or of
  the second (0).
- *Part d (both gains in one tick):* a damaged own non-organic vehicle of a self-healing type, one
  building with vehicle count 1, amount 2, and the vehicle interval equal to the building repair
  period. At 10 below full health. *Expected:* on the first common tick it gains 3 (1 and 2). At 2
  below full health it becomes full, and never rises above the strength. *Falsified if:* it gains
  another amount, or ends above the strength.

**H9: exclusion by the skill-level gate (claim 8).** The human house has 100 000 credits and two
damaged buildings of a repairable type, with individual and house-wide self-healing disabled,
and gives no repair or sell order. No map action heals, repairs or sells them. Record health,
credits, repair state and whether each building is sold over 3000 ticks.
- *Run 1:* threshold at the absent-key default 3, then repeat with threshold 1.
  *Expected:* no automatic repair or sale. *Falsified if:* either occurs.
- *Run 2 (exploratory):* repeat with threshold 0, then −1. Record whether repair or sale starts,
  and the health and credits at each event. A start shows that a level-0 human house can reach
  the rule; no start does not falsify the gate, since later conditions are still unknown.
  This run does not back a promised automatic outcome.
- *Run 3:* as run 1, also pre-place two damaged repairable buildings for each of the neutral
  and special houses (S-0009 claim 27). Their starting credits follow S-0412 claim 2; do not
  assume that the lobby gives them the human's money. *Expected:* none starts repair or is
  sold. *Falsified if:* either event occurs for any of the four buildings. Their skill level
  itself is independently tested by S-0412 K10 and S-0209 C7, not inferred from silence here.
- *Run 4 (exploratory):* repeat run 3 with threshold 0, then −1. Record each non-player house's
  available credits and any repair or sale. No outcome is promised while further conditions
  remain open. Neither silence nor a start alone confirms those conditions. This run supplies
  observations for that question, not a falsifier for a complete automatic rule.

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

**H11: arithmetic boundaries and bad data (F1, F2, Edge cases).** Use the objects and ordinary
repair orders of H3, H5, H6 and H8 as indicated; each variation is a fresh program run so a
previous test field cannot persist. Stop negative-health tests before health reaches 0.
- *Decimal timing:* set all three repair rates to the written decimal 0.1. *Expected:* building
  steps on global multiples of 90 ticks; depot and hospital working steps 91 ticks apart.
  Record the entry-relative phase separately using their first processing. Repeat with rates
  0.125: period 112, working count 113. Any other period, count or phase than F1 and claims 6
  and 7 falsifies those timings.
- *Short working count:* set the vehicle and infantry rates to 0, then −0.125. *Expected:* a
  working step every tick, the first working step *D* ticks after the settling step or first
  hospital processing. Any larger interval or a program error falsifies the count's minimum.
- *Zero periodic divisor:* in separate runs set the building repair rate to 0 for H5, then for
  H7, and set each house-wide interval to 0 for an eligible H8 object. *Expected:* the original
  stops with a program error at its first applicable periodic test; progression beyond that
  test falsifies the claim. Our engine instead uses 1 tick and warns; that departure is not
  an oracle prediction. Repeat each divisor with a negative value: use building rate −0.125
  and house-wide interval −40. *Expected:* the same ticks as positive 112 and 40 respectively,
  with no error. A different schedule or an error falsifies the negative-divisor rule.
- *Cost divisions and minimum:* H3 vehicle, cost 1000, strength 605, step 10, repair percentage
  written as 0.99. *Expected:* quotient 60, integer cost per step 16, payment 15 credits.
  Any other payment, including 16 from delaying the integer truncations, falsifies F2.
  Repeat with cost −1000 and percentage 25 %, then cost 1000 and percentage −25 %:
  *expected:* 1 credit per step in both runs; any other payment falsifies the minimum.
- *Cost division by zero:* separate H3 runs with repair step 0, and with strength 5 and step
  10. *Expected:* the original stops with a program error when the depot first asks for a
  repair step on the damaged vehicle. A successful step falsifies the asserted failure.
  An infantry unit in H6 with infantry step 0 still heals by 1 for free (H6 part b); an error
  there falsifies the infantry exemption. Our engine's repaired divisors are not measured here.
- *Negative health gain:* H8 infantry at health 100, amount −3 and one positive source, interval
  40. *Expected in the original:* health falls by 3 at the first multiple of 40 without a
  charge; a positive or zero gain, another amount or another tick falsifies it. End the run
  before health 0. H5 part c similarly records a negative building step. Behaviour beyond
  leaving the normal health domain is `unmeasured`: no stable health-domain prediction can
  be tested there. Our engine uses 0 for the negative self-heal amount and warns.

**H12: absent-key defaults (Constants and Import mapping).** Use a named loose test copy of
base rules plus the test map. Omit the tested key from every applied rules layer, start a fresh
program for each test and retain the other H3, H5, H6 or H8 inputs. Record health, payment and
tick numbers. For each omission the expected result is the owning claim's formula using the
absent value from Constants; any different gain, payment, period or count falsifies that default.
- Omit the game-wide repair step, infantry step and percentage one at a time in H3/H6: expected
  step 5 for the vehicle, 1 for infantry, percentage 0.25 in F2, minimum costs unchanged.
- Omit each repair rate: expected global building period 14 or working count 15 as appropriate.
- In H8 omit each interval or amount separately: expected interval 1000 ticks or amount 1 per
  positive source. Omit each building source count: that source contributes 0, so a house with
  only that source heals nothing of its affected kind. Omit the self-healing flag in an H7
  control with no ability: no individual healing. Omit the organic flag on an H8 vehicle:
  expected the vehicle interval and amount, not the infantry ones.
- Omit the depot flag from an otherwise identical test depot: no repair-depot entry cursor
  or depot exchange. Omit the hospital flag from an otherwise identical test hospital: no
  hospital-entry cursor or hospital exchange. A remaining respective cursor or exchange
  falsifies the absent false flag. The organic defaults for infantry, aircraft and buildings
  are `unmeasured` here: their kind already determines whether this spec applies a healing
  total, so changing the organic flag cannot distinguish those defaults through its health
  observations.
- Omit the automatic threshold in H9 run 1: expected exclusion of the level-0 human, neutral
  and special houses. This tests only the exclusion, not the exact numeric threshold; the
  default threshold 3 is `unmeasured` here, because the only controlled skill level is 0 and
  any positive threshold has the same observable consequence.

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
- The remaining eligibility conditions and outcomes of automatic building repair and sale when
  the skill-level gate passes (claim 8, H9 runs 2 and 4). Their reproduction must follow the
  existing house policy; no departure has been chosen.
- The distance and duration of the repeat suppression of the repair-complete announcement (claim 6,
  H4 part f), which the interface specs own.

