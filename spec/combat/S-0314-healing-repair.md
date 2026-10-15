---
id: S-0314
title: Healing and repair in combat (healers, self-healing, house-wide healing, repair depot, hospital, building repair)
topic: combat
revision: 17
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0341, Q-0641, Q-0680, Q-0681, Q-0682, Q-0683]
supersedes: []
---

# S-0314: Healing and repair in combat (healers, self-healing, house-wide healing, repair depot, hospital, building repair)

## Summary

How objects regain health: healers, self-healing (one object, or every infantry unit or vehicle
of a house), repair steps at a repair depot and a hospital, and building repair. This spec owns the
timings, the step sizes, the step cost and the house-wide healing. The arithmetic of a healing hit is
S-0302's, and the orders that send a unit to be healed are S-0213's and S-0214's.
Only skirmish is covered; network games and campaigns are out of scope. Claims 12 to 15 specify
automatic building repair and sale when the skill-level gate passes, including their random choices
and the differences between player and non-player houses.

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
- **Depot contact, docking place.** Contact and tether have S-0215's Terms meanings. A docking
  place is one of the depot's contact places, reserved by an accepted entry request (claim 17).
  Contact can exist before the vehicle reaches the building.
- **Queued entry.** A vehicle's standing intention to enter one depot. While it holds one, the
  vehicle requests a contact place again on every tick (claim 23).
- **Docking point.** The world position the depot supplies for that vehicle's approach and
  waiting-distance check (claim 18 for repair depots, including naval yards).
- **Building repair.** The repair state of a building that its owner turns on and off with the
  repair order (S-0213, claim 13). It is separate from the repair steps of claims 3 to 7 (claim 8).
- **Terms owned elsewhere.** *Radar event* (S-0011, claim 32), *rally point* (S-0213, claim 20),
  *rate delay* and *processing* (S-0215, F1) are used as those specs define them.
- **Automatic-repair hold.** A house-wide block on starting another building's repair, initially
  clear. An automatic start sets it; claim 15 gives its release. It does not stop repair already on.
- **Repair reserve.** A game-wide integer number of credits (Import mapping), separate from the
  cost of a repair step.
- **Computer-repair flag.** The per-building map flag of S-0009 claim 40, not a type flag.
- **Captured mark.** A building's persistent record of a successful ownership change, initially
  clear. Capture or a map-trigger ownership transfer sets it; a later transfer does not clear it.
- **Hostile-hit mark.** An object's persistent record of a surviving hit with a known attacker
  whose house the object's owner does not count as allied, after S-0302 claim 8's hit-stopping
  tests have allowed the ordinary damage response. Initially clear; a qualifying hit sets it,
  and subsequent healing does not clear it. Claim 14 uses it for buildings.
- **Sale skill.** The setup skill value used only for claim 14's additional sale test. It is
  distinct from the current house skill level of claim 8: in skirmish it stays 0, including for
  the original's computer players, whose current level instead receives the maximum.
- **Turn.** An object's share of a tick, as in S-0203 claims 2 to 4, not a rotation.
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
   S-0302 claim 6 states this distinction. A negative result raises health by its magnitude,
   never above strength. The versus value of
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
   is the owner of the repaired object, not necessarily the depot's owner: an allied depot can
   accept it too (claim 16).
   - Backing: `measurable` at non-negative available credits (H3, H4 and H6). The affordability
     test at a negative balance is `unmeasured`: no scenario here arranges a hospital entry at
     such a balance, and vehicles have a positive step cost so cannot isolate the zero-cost case.
   - Confidence: high

4. **Step size.** The step size of an infantry unit is the game-wide infantry repair step; for
   every other object it is the game-wide repair step. A step size below 1 counts as 1. In the
   original, building repair takes the game-wide repair step without this minimum; our engine applies
   the minimum there too (claim 8, Edge cases). Aircraft on reload pads use this same step size.
   S-0501 claims 26 and 27 own servicing cadence, reload priority and release.
   - Backing: `measurable` for depot and hospital steps (H3 parts a and d; H6 part b),
     and for aircraft pad steps (S-0501 AC16).
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
     infantry cost of 0; H11 for signed inputs and F2's truncations; S-0501 AC16 for the same
     cost formula on an aircraft pad step, with servicing governed by S-0501 claims 26 and 27).
   - Confidence: high (the formula, the minimum, 0 for infantry, no house or country factor),
     medium (the free-unit adjustment and the pad-aircraft deduction, S-0408 claim 2)

### Repair depot, hospital and building repair

6. **Repair depot.** Write *K*<sub>u</sub> for the vehicle repair count and *D* for the depot's
   processing delay (Formulas F1). Claims 16 to 18 describe contact and entry, and claims 20 to 24
   how contact ends. A repair depot treats a vehicle docked on it in three phases:
   - *Waiting.* At each processing, normally *D* ticks apart, the depot checks whether the
     vehicle stands still within the starting distance of its docking point: strictly less than
     100 leptons for ordinary locomotors, or 200 leptons for ship and hovering locomotors.
     Distance is the floating-point spatial distance, including height, truncated toward zero
     to integer leptons, not distance to the building centre (S-0504 claim 8).
     A successful check schedules settling 3 ticks later; coming to rest alone takes no step.
   - *Settling.* At that later processing, provided the vehicle still stands still and its
     locomotor has finished settling, the depot asks for one repair step (claim 3). On *not needed* the vehicle is sent away at once, as below, and no repair-complete announcement is
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
     - *cannot pay:* the vehicle stays in contact and the depot returns to settling, with its
       first retry on the next tick and subsequent unsuccessful retries *D* ticks apart.
       Restoring the paying house's funds can therefore restart repair without a new order.
       The depot's owner, when it is the local player and its available credits are exactly 0,
       hears the low-funds announcement on leaving the working phase.
   *The repair-complete announcement.* It belongs to the depot's owner: it is heard only when that
   house is the local player. It tries to create the *unit repaired* radar event, the kind that
   depots and hospitals share (claim 7). If a nearby event of that kind suppresses creation,
   neither a new event nor the announcement is raised. S-0701 claim 28 owns the distance test and
   post-zoom lifetime. S-0410 claims 14–15 own the zoom progression and cadence: the radius
   contracts and rotation slows until its completion condition holds, advancing once per drawn
   frame even without radar or while the panel opens or closes. Completion starts the post-zoom
   tail at that frame's tick number, not at repair completion. A timed freeze advances zoom but
   not the tick-based tail; a stand-still with no drawn frames advances neither. Restoring radar
   does not restart the event. The total lifetime is therefore the position- and
   imported-setting-dependent zoom phase followed by S-0701 claim 28's tail, not a fixed tick count from
   repair completion. S-0011 claim 32 owns triggered events.
   Zoom timing inherits S-0410 claims 14–15's backing and confidence, including their unmeasured
   exact floating-point equality boundaries. The *unit repaired* suppression row and the
   trigger/depot creator pair are `measurable`, with high confidence, through S-0701 claim 28,
   scenario G7: it tests the row through trigger requests and the trigger/depot pair in both
   directions. These are proposed observations, not completed measurements, and do not cover
   every event creator. H4 part f tests only its stated depot-completion cases.
   The low-funds announcement belongs to the depot's owner
   too: it is heard only when that house is the local player, and it is that house's available
   credits that must be exactly 0. It has no such suppression. The paying house of a step is
   the vehicle's owner (claim 3), even at an allied depot (claim 16).
   A vehicle that the settling step brings to full health (*done*) therefore waits for one more
   step, which reports *not needed*, before it is sent away. A vehicle that is sent away moves to
   the depot's rally point when the depot has one (S-0213, claim 20), and otherwise to an exit cell
   selected by claim 25. With no selected cell and no rally point, no departure is ordered and
   contact is retained. A failed departure from working leaves the vehicle in settling with
   a 1-tick retry. A failed departure while already settling retains that phase and retries
   after *D* ticks.
   A selected destination queues *move* without immediate start, replaces the destination,
   releases contact and tether, and clears queued entry. A rally point is not subjected to the
   exit-cell suitability test; exit selection itself still occurs before that override.
   *The original's return to the guard spot.* In the original, a vehicle whose owner is not
   human-controlled and that has a guard spot (S-0215, Terms) when it is sent away moves to that
   guard spot instead of the rally point or the exit cell, and the guard spot is cleared. In our
   engine this holds for the vehicles of the *neutral house* and the *special house*, which keep the
   original's rules for owners that are not human-controlled (glossary, *AI house*, *Player houses
   only*). It holds for no player house: the vehicles of an *AI house*, like a person's, follow the
   destination rule above, and our AI sends them on by ordinary orders.
   - Backing: `measurable` (scenario H4, parts a to f, H16 b for an allied depot's owner and
     announcement distinction, H18 b for a moving vehicle, H21 for departure and blocked-exit
     retries, and S-0504 N8 for the waiting gate, spatial distance and 3-tick delay; S-0701 claim 28, G7 for the suppression row and trigger/depot pair), except the
     following cases.
     `unmeasured` for the return to the guard spot of a non-player house: no input of
     the measurement setup makes a vehicle of the original's neutral or special house, or of its
     computer player, dock at a depot in a way that can be timed and recorded, and a player house
     never takes this path. It is justified as a rule recorded from the original's behaviour that the
     measurement setup cannot reach. H18 b tests movement while tethered (claims 20 and 21),
     including a working step falling due while moving and being taken when movement ends.
   - Confidence: high (one step per count of *K*<sub>u</sub>, the cost, the end on *done*,
     the waiting thresholds, spatial distance, 3-tick delay, suppression row and trigger/depot
     pair, and departure destination/no-cell outcome); medium
     (the settling step, the phase *D* + *K*<sub>u</sub> − 1, the return to settling after *cannot
     pay*, the owner conditions of the two announcements, that the settling *not needed* makes no
     announcement, and that *not needed* is tested before *cannot pay*), and low (the return to
     the guard spot; the moving step's exact phase)

7. **Hospital.** An infantry unit inside a hospital gets repair steps at a cost of 0, on the
   timing of the depot's working phase with the infantry repair count *K*<sub>i</sub> (Formulas
   F1): the hospital takes no step when the unit enters, the first step falls due *D* +
   *K*<sub>i</sub> − 1 ticks after the hospital's first processing after the unit entered, and each
   later step *K*<sub>i</sub> ticks after the step before it. On *more needed* the unit stays
   inside; on *done* it is put out, and the local player, when the hospital is theirs, hears the
   repair-complete sound and, unless it is suppressed, the repair-complete announcement (claim 6:
   the *unit repaired* radar-event kind shared with the depot, with suppression owned by
   S-0701 claim 28, with G7 covering the row and trigger/depot pair, not the hospital creator).
   On *not needed* it is put out
   without them. *Not needed* is rare here: S-0214 (table I, row e4) refuses the order that sends a
   full-health infantry unit into a hospital, so it arises only when a unit reaches full health
   while inside, which is not known to happen (Open questions). *Cannot pay* cannot happen here at non-negative available credits: the infantry step cost
   is 0 (claim 3). A negative balance is not exercised by H6; that case is `unmeasured` because
   no scenario here can arrange hospital entry for an infantry unit at such a balance. The armory of S-0311 (claim 12) uses
   the same infantry repair count, but counts it in processings instead of ticks (S-0311, Formulas
   F6).
   - Backing: `measurable` (H6 parts a to c for own hospitals; S-0701 claim 28, G7 for the
     shared suppression row). Hospital-specific cross-creator suppression is `unmeasured`:
     neither H6 nor G7 isolates a hospital/trigger pair. Absence of feedback for another
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

   *Automatic repair and sale.* Claims 12 to 15 give the automatic-start rule. It is gated
   by the *house skill level* (glossary): it can apply
   only to a house whose level is at least the game-wide repair-and-sell threshold (Import
   mapping; absent-key default 3). In a skirmish the levels are these:
   - every human-controlled house has level 0,
   - an *AI house* has level 0 in our engine (glossary), where the original gives its computer
     player the rules' maximum level (absent-key default 5), which passes the gate at the default
     threshold. This is an original-only note, not an exemption for our AI (Edge cases), and
   - the *neutral house* and the *special house* keep level 0, since the original gives the
     maximum only to the houses of computer players when the houses are set up (S-0209, claim 11); they keep the original's rules in our engine, so their level is 0 here too.

   With a threshold above 0 the gate excludes every level-0 skirmish house. With a threshold of
   0 or below the gate passes for every level-0 house, including human-controlled houses, AI
   houses, the neutral house and the special house. Passing the gate is necessary, not sufficient:
   it does not promise either a repair or a sale. Claims 12 to 15 give the further conditions.
   AI houses follow human-controlled unit rules, and the neutral and special houses retain the
   original's non-human rules. There is no threshold-specific departure or importer warning.
   Approved S-0408 claim 13 already states that claims 12 to 15 specify the further conditions and automatic-start outcomes when the skill-level gate passes.
   - Backing: `measurable` for periodic repair (H5) and the exclusion at a positive threshold
     (H9 runs 1 and 3). House levels are `measurable` through S-0412 claim 3's K10 and S-0209 C7.
     Gate passage at 0 and −1 is `measurable` in H13 and H14. The exact absent threshold is
     `unmeasured` for H12's stated limitation: controlled level 0 distinguishes a positive
     threshold from a non-positive one, but not one positive threshold from another.
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
    Both this claim's healing and claim 9's healing precede an attached parasite's delivery
    in the host's same turn (S-0614 claim 18). Health already restored can prevent a lethal
    delivery, but neither kind heals damage from that delivery later in the same turn.
    - Backing: `measurable` (H8 parts a to e, including the building-kind exclusion and
      same-turn parasite ordering; S-0614 claim 18). The
      health-0 exclusion is `unmeasured` for the same destruction-handling reason as claim 9.
    - Confidence: high (amounts, intervals, the kinds, the stacking, same-turn parasite ordering); medium (the events that change
      the totals, the floor at removal, and that power plays no part)

11. **Healing on resources has no effect.** The type flag and the game-wide value for healing on
    resource cells, and the resource-heal veterancy ability (S-0311, claim 5), change nothing in the
    original, and our engine gives them no effect. The rules key for a healer search radius
    has no observable effect on the search radius either (Import mapping).
    - Backing: `measurable` (scenario H10 here; S-0311 scenario X11 for the ability)
    - Confidence: medium

### Automatic building repair and sale

12. **Common eligibility and credit split (Q-0341).** On each ordinary living building turn that
    reaches repair work, automatic action is considered when claim 8's skill gate passes, the
    current behaviour is neither *construction* nor *selling*, and the building is repairable
    under S-0408 claim 14. In particular a full-health building is ineligible. Buildings being
    erased or phasing in after a chrono jump do not reach this work. No damage event, repair
    period or behaviour-processing countdown is needed to prompt the check.
    Available credits at that turn choose the action: at least the repair reserve permits only
    the repair-start rule (claim 13); below the reserve permits only the sale rule (claim 14).
    Equality belongs to repair. The comparison is signed integer arithmetic, without rounding.
    Passing neither branch's further tests does nothing; failing the sale tests does not try repair.
    This split does not switch off an existing repair: claim 8 still controls its paid steps.
    - Backing: `measurable` (H13 a to c; H14 a to d; H13 f for erasure). The phase-in exclusion
      is `unmeasured`: no ordinary input here puts a building into chrono phase-in.
    - Confidence: high

13. **Starting repair automatically (Q-0341).** In claim 12's repair-credit case, a building
    whose repair is off starts it if the owner's automatic-repair hold is clear and at least one
    of these is true: the owner is human-controlled, the captured mark is set, or the building's
    computer-repair flag is set. In our engine AI houses count as human-controlled here; neutral
    and special houses do not. Thus a human or AI house needs neither building mark, while an
    uncaptured building of a neutral or special house needs the map flag.
    The start sets the house's hold. It takes no payment and changes no health by itself; the
    next applicable claim-8 step pays and heals. At a global repair-period multiple, that step
    can occur in the same turn as the start. No check here asks whether the house could pay a
    step, beyond comparing its credits with the reserve. A reserve smaller than the step cost
    can therefore allow a start that the next step immediately stops for lack of money.
    Repair already on neither sets the hold nor takes a new start draw.
    - Backing: `measurable` (H13 a to e; H15)
    - Confidence: high

14. **Trying a sale automatically (Q-0341).** In claim 12's below-reserve case, a building with
    the hostile-hit mark takes one ranged draw R(0, 50) from the scenario random stream on each
    eligible turn, provided its owner's sale skill is at least the game-wide sale threshold.
    The skill comparison treats both integers as unsigned 32-bit values (a negative value
    denotes that value plus 4294967296). Consequently, in skirmish with sale skill 0 a threshold
    of 0 permits the draw, but a positive or negative threshold does not.
    The draw succeeds when its result is strictly less than the owner's tech level, also read
    as unsigned for this comparison. Even with tech level 0 the draw is taken. This is not a
    percentage draw: for tech level *L* in 0 … 51 the success probability is *L*/51, and at
    *L* ≥ 51 it is 1. These are exact rational probabilities, without rounding.
    A successful draw initiates selling only when the building carries no tag, its type is not
    a factory for buildings, and its floating-point health ratio is strictly below the
    game-wide red-health threshold. Equality does not qualify. The draw is taken even when
    any of these last three tests prevents a sale. Its raw-value consumption, including
    rejections, is S-0202 claim 4's, not necessarily one raw value.
    A sale attempt has the same effect as S-0408 claim 4, except that a pending delayed kill
    prevents it. Refund, build-down and upgrades remain S-0408 claims 1 and 4 to 10's,
    and packing up remains S-0415 claims 9 to 11's.
    The ordinary input sellability test is not an additional condition. In skirmish the
    per-building computer-sell flag plays no part. Neither the automatic-repair hold nor the
    computer-repair or captured marks constrain this sale branch, and there is no human-only
    or non-player-only exception to it.
    - Backing: `measurable` (H14, including draw counts and rejection controls)
    - Confidence: high (credit split, comparisons, range, draw position and last tests),
      medium (hostile-hit mark and its persistence)

15. **House-wide pacing and observable order (Q-0341).** The first eligible automatic repair
    start in a house blocks later buildings of that house in the same tick. In a tick with multiple eligible buildings, their automatic checks and sale draws have the same observable order as the object-turn order of S-0203 claims 3 and 4: for example, a healing step paid by one building can change the credits available to a later building's credit split in that tick. The credit-order consequence, rather than a particular internal pass or data structure, is the required behavior.
    For a human-controlled house, and for an AI house in our engine, the hold is released by
    that tick's later house update (S-0203 claim 8). No repair-start random choice is made for
    such a house: at most one new automatic start per house per tick, but any number of repairs
    already on may continue. Turning repair off by order can be undone on a subsequent eligible
    turn; a paid step that stopped for lack of funds can likewise be started again.
    The original gives a non-human owner a ranged choice of delay at each automatic start,
    F4, and releases the hold in the first house update at or after that delay expires. The
    next building can start no earlier than the following tick. Neutral and special houses
    retain this rule in our engine. They have repair-start delay 0 in skirmish because no
    difficulty row is applied to them (S-0412 claim 5): their range is R(0, 0), which consumes
    no raw value, so their hold is also released in the start tick's house update. The original's
    computer players use their difficulty row's delay; our AI instead follows the human rule.
    - Backing: `measurable` (H15 for starts, credit ordering, hold release and draws; H13 d for
      restarting). The nonzero computer-player delay is `unmeasured` here: the setup cannot
      require that player to leave the chosen buildings and repair orders alone during the
      cooldown. It is recorded for AI designers, not required of AI houses.
    - Confidence: high (one start, hold release, the two draw sites),
      medium (the nonzero computer-player delay)

### Docking at a land repair depot

16. **Permission and player entry (Q-0641).** S-0214 claim 8, table V row n3,
    already supplies the ordinary click's *enter* action over an accepting allied building.
    Its claim 10, table M, supplies the enter order. S-0215 claim 29 owns the repeated entry
    requests. The depot's permission does not require damage or affordable repair: a full-health
    vehicle can enter, and claim 6 sends it away at the settling check without payment.
    The depot's owner must count the vehicle as allied. Construction, selling, remaining in
    the build-up state, being switched off, and no contact place free for this vehicle prevent
    acceptance. A place already assigned to this vehicle is not considered occupied by another.
    S-0504 claim 7's naval match applies. A vehicle already standing on this very depot does
    not receive a new acceptance through this query. The click's fly-zone refusal remains
    S-0214 claim 8, table V row n1. These are permission rules, not a new repair-sidebar command.
    - Backing: `measurable` (H16 a, b and d).
    - Confidence: high (no damage or money gate, alliance and contact-place tests), medium
      (input consequences of the refusal cases)

17. **Contact is earlier than repair (Q-0641).** A depot has max(*d*, 1) contact
    places, where *d* is its imported docking-place count. A successful entry request establishes
    contact in the first free place before the vehicle has reached the docking point. It is
    this assigned vehicle, not the nearest damaged vehicle, that the depot services. At a
    one-place depot an ordinary click by a second vehicle while that contact is held does not
    produce another enter order: the permission fails, rather than adding the vehicle to a
    depot waiting queue. The existing contact is not displaced.

    Contact alone does not set a tether. Each due *enter* processing repeats the entry request.
    Acceptance or an existing tether allows entry to continue. Refusal without a tether releases
    contact and falls back to rest (S-0215 claim 29). This is not a repeat of claim 16's initial
    permission test: entering a foundation cell does not by itself trigger its already-standing
    refusal.

    For a depot with the pad flag and neither resource-unloading nor obsolete-unloading flag,
    the repeated request establishes tether when the vehicle occupies the cell containing the
    depot's foundation centre and either has no moving destination or the first contacted
    vehicle's truncated spatial distance from that centre is greater than 128 leptons. It need
    not be at the supplied docking point, have stopped, or have entered settling. A moving destination nearer the centre postpones this
    exchange. Without the pad flag this ordinary repeated-request exchange sets no tether.
    Completed-entry confirmation also establishes a tether if one was not already present.
    There is no rule that every foundation cell, or initial reservation, tethers the vehicle.

    Reaching the destination cell is distinct from first contact. The completed-entry exchange
    can turn a tethered vehicle toward 16384 facing units (map east) and report arrival once it
    is no longer moving and still has *enter* as its behaviour. Separately, reaching the supplied
    docking-point cell during movement queues *repair* at the depot and *sleep* at the vehicle.
    It does not require exact agreement with the docking point's horizontal coordinates or a
    tether. Claim 6's waiting checks begin when the depot processes repair, not on the click.
    There is also an earlier start opportunity for an unarmed depot in the waiting stage of
    its guard processing, after initial idle-body selection (S-0215 claim 29): a contacted
    entering unit, standing still at a spatial distance strictly below 64 leptons from the
    foundation centre, causes the depot to queue repair with a 1-tick processing delay.
    This opportunity does not apply to an armed depot's guard processing or to the initial
    idle-body selection itself.
    This guard test uses floating-point Euclidean distance truncated toward zero to integer
    leptons. It is not claim 6's distance to the docking point and never takes a paid step.
    Simultaneous requests are claim 23's; servicing with more than one imported contact place
    is in Edge cases.
    - Backing: `measurable` (H16 a and c, H19 a; contact, tether, behaviours and processing
      ticks recorded separately). Flag combinations absent from stock are `unmeasured`: the
      stock-only scenarios cannot establish them.
    - Confidence: high (capacity, repeated request, cell test and absence of initial tether),
      medium (stock approach observations and early guard start)

18. **Depot docking point and stopping (Q-0641, Q-0680).** For an ordinary repair depot,
    including a naval yard, without obsolete-unloading, resource-unloading or tank-bunker flags,
    let *c* be the foundation centre
    including the depot's world height. With an imported docking-place count of 1, the point
    is *c* plus the sole imported docking displacement. With a count greater than 1, use the
    displacement of the vehicle's assigned contact place, counted from 0. With a count of 0,
    or no assigned place within the displacement count, the point is *c*. These are the same
    displacement-selection rules as S-0404 claim 15, applied to a vehicle approaching repair.
    Coordinates and additions are signed 32-bit integer leptons with S-0205 claim 1's wrapping
    arithmetic, without floating-point calculation or rounding. The height displacement is
    added too. Neither approach side nor vehicle facing rotates or changes the point.
    The obsolete-unloading flag instead selects the centre of the cell displaced by (2, 1)
    cells from the top-left foundation cell, at the building's world height. It takes precedence
    over pad displacements. The vehicle-factory flag does not select that fixed point.

    The depot directs the vehicle toward the building as its movement destination; the supplied
    docking point, not a searched neighbouring exit cell or the rally point, is the destination
    position. The vehicle remains on the map. Foundation blocking is still S-0209 claim 5's:
    the repair flag does not make all blocking foundation cells traversable. The general approach
    exchange is S-0215 claim 29's, not an unconditional permission to cross the foundation.
    Driving vehicles and ships complete their cell-to-cell tracks normally. At track completion,
    being in the docking-point cell with a vertical difference strictly below 208 leptons is
    sufficient to stop the approach. Horizontal alignment with an off-centre docking point is
    not required: on ordinary level tracks they finish at that cell's centre, not teleported to
    the supplied point. The distance gates in claims 6 and 21 still use the supplied point,
    including its height, not the stopping position. A stopped vehicle outside the starting
    distance can therefore keep contact without beginning repair. Claims 20 to 23 govern
    interruptions; ordinary track geometry remains S-0207's and S-0504's.
    - Backing: `measurable` (H16 a and e; H19 b). Unused flag combinations and the exact
      208-lepton vertical boundary are `unmeasured`: no stock-only setup is established here
      that distinguishes the boundary without also changing the route.
    - Confidence: high (point selection, precedence, integer arithmetic and cell-based stop),
      medium (stock route observations)

19. **Healing during sinking.** A sinking vehicle still receives the individual and house-wide
    self-healing of claims 9 and 10 when their ordinary eligibility tests pass. Sinking neither
    adds a health test nor changes their tick schedules, integer amounts or caps. Raising health,
    even to full health, does not clear sinking or cancel descent and removal (S-0504 claims 4,
    5 and 10). This does not grant depot contact or another healing-weapon target.
    - Backing: `measurable` (H17; S-0504 N10).
    - Confidence: high

### Ending and losing depot contact

In claims 20 to 24 a vehicle is *approaching* a depot while it holds contact with it but is not
yet tethered to it, and *docked* as soon as it is tethered (claim 17), even during movement.
These terms describe tether status, not whether repair has begun; a contacted vehicle without a
tether can already be in *sleep* or receiving repair.
The *serviced place* is the depot's first contact place (place 0): every check of claim 6 reads
only the vehicle in that place.

20. **A new order.** When an order is executed for a vehicle in contact with a depot:
    - An enter order on the same depot, given while the vehicle already holds contact with it
      and has *enter* as its behaviour, is ignored entirely.
    - An *approaching* vehicle given any other order loses its contact at once, so the place is
      free for another vehicle in the same tick. Its queued entry (claim 23) is cleared too.
    - A *docked* vehicle keeps both contact and tether whatever order it is given, with two
      exceptions. First, a vehicle holds one contact, so a successful entry request to another
      building releases the depot. Second, an enter order on the same depot ends the docking.
      A damaged vehicle takes claim 23's queued entry, and at its next eligible retry the permission
      refusal for a vehicle standing on the depot (claim 16) clears its contact and queued
      entry. A vehicle at full health loses its contact at once. Otherwise the order runs
      normally, and claim 21 decides what the depot then does.
    - Backing: `measurable` (H18 a).
    - Confidence: high (approaching release, ignored repeat), medium (docked retention under
      each order kind), low (same-depot enter order while docked)

21. **Moving away while docked.** The depot's checks of claim 6 continue for a docked vehicle
    wherever it goes:
    - *Waiting:* the vehicle advances only from inside the starting distance. A vehicle that has
      left keeps its contact place, receives no step and blocks that place until it returns,
      leaves by claim 20, or is destroyed.
    - *Settling:* while the vehicle is within 200 leptons of its docking point (the same
      truncated spatial distance as claim 6) and still moving, the depot holds settling and
      checks again on the next tick. Once it stands still there, the depot switches its
      locomotor off (claim 22) before stepping. A vehicle whose locomotor is already off and
      which has a destination has its movement stopped. At a distance of 200 leptons or more,
      neither of those near-point locomotor adjustments applies. The depot still asks whether
      the vehicle is moving: a moving vehicle takes no step, keeps settling and contact, has
      its locomotor switched on unless ion-sensitive shutdown is in force, and is checked
      again after *D* ticks, not 1 tick. A stationary
      vehicle proceeds to the settling repair step despite that distance. There is no extra
      near-point restriction after this test.
    - *Working:* only movement defers a step (claim 6). A docked vehicle standing still at any
      distance from the depot, for example after a move order or while it attacks without moving,
      receives its steps and pays for them (`quirks.depot_remote_repair`).
    - When the depot sends the vehicle away at full health (claim 6), the destination it gives
      replaces any destination the player gave while docked. The repair-complete announcement
      follows claim 6.
    - Backing: `measurable` (H18 b; H20 for distant moving settling).
    - Confidence: high (working-phase distance, release override and distant-moving delay),
      medium (stock settling observations)

22. **Sale, destruction and power.**
    - *Sale.* When the depot starts selling, every vehicle in contact with it, docked or
      approaching, receives a run-away notice:
      - it clears its destination if that destination is the depot,
      - a vehicle in *sleep* or *enter* switches to *guard* (from *sleep* at once, from *enter*
        when its current processing ends),
      - a vehicle with no destination left can be sent a scatter request, which S-0215 claim 19
        accepts or refuses. A further vehicle-state condition on sending it is not established.

      At its next processing the selling depot releases every contact. A selling depot already
      refuses new entry (claim 16).
    - *Destruction or other removal.* Every contact is released without a run-away notice.
      An approaching vehicle loses its target and falls back to *guard* where it stands.
      A docked vehicle stays in *sleep*: it does nothing until it receives an order
      (`quirks.depot_loss_leaves_sleep`).
    - *Locomotor switched off.* The settling step switches the docked vehicle's locomotor off
      (claim 21). Normally it is switched on again when the vehicle receives a destination
      while a depot or tank bunker occupies its cell. A hovering vehicle with its locomotor
      switched off cannot move. If its depot is sold or destroyed after settling, nothing switches
      it on again, and the vehicle stays immobile under every later order
      (`quirks.depot_loss_strands_hover`). No such effect was found for other land locomotors.
    - *A vehicle destroyed while in contact* frees its place. A depot left without any contact
      plays its idle animations and returns to *guard*.
    - *Power.* Neither the owner's low power nor the depot being switched off stops or slows the
      servicing of a vehicle already in contact: steps, costs and timings are claim 6's. Being
      switched off refuses only new acceptance, including a queued vehicle's (claims 16 and 23).
      Low power refuses nothing here.
    - Backing: `measurable` (H18 c, d and e).
    - Confidence: high (sale notice and release, no power test), medium (behaviour after
      destruction, hover stranding), low (the scatter request, no stranding for other land
      locomotors)

23. **Simultaneous and queued entry.** Orders execute one at a time in event order (S-0213), so
    two vehicles sent together are resolved in that order. The first entry request finds a free
    place and takes contact (claim 17). For a later enter order on a depot that holds *any*
    contact, whatever its place count:
    - a vehicle below full health (health ratio below 1) gets no destination and no contact. It
      stays where it is with a *queued entry* for that depot. Once per eligible vehicle turn
      it asks again for a contact place (claim 16's alliance and free-place tests). While none is free it keeps
      waiting. When one is free it takes it and asks permission. If permission is granted, it
      queues *enter* with immediate start (S-0215 claim 7), assigns the depot as destination
      and clears queued entry. It can therefore make *enter* current in that same turn when
      ready; this does not process the new behaviour synchronously. If permission is refused (for example the
      depot is switched off), it gives up: contact, queued entry and
      destination are cleared and it falls back to rest.
    - a vehicle at full health also gets no destination and no queued entry: the order ends,
      and the vehicle stays put.

    For a vehicle receiving an ordinary living turn, the retry is after that turn's locomotor
    movement, self-healing and due behaviour processing, and after completed locomotor
    replacement has been dealt with. It is not an orders-phase batch, not before movement,
    and not an extra behaviour processing. It is skipped on a turn excluded by the vehicle's
    immobilized-state check; queued entry is retained. An inactive remembered depot clears
    the queued entry. A newly current *enter* is first eligible for due processing on a later
    vehicle turn; movement already performed this turn is not repeated.

    A place freed before this retry is available immediately, including release by an earlier
    object turn in the same tick. A place freed after the retry waits for a later tick. If
    several waiters can take it, their object-turn order (S-0203), not their original order
    arrival time or distance to the depot, decides. This retry introduces no global phase.

    Any later order clears the queued entry (claim 20). So at a one-place depot the second
    vehicle enters by itself at the first eligible retry after the first leaves. The sequential case of claim 17 differs:
    a click made while the place is held gives no enter order at all. With an enter order on a
    depot that holds no contact, the vehicle asks for entry at once. If it is refused, it takes
    no contact and does not queue.
    - Backing: `measurable` (H18 f; H22 for local turn ordering, same-tick acquisition and
      next-turn processing). The immobilized-state and inactive-reference cases are
      `unmeasured`: a stock-only setup distinguishing each check has not been established.
    - Confidence: high (event order, retry position, queue/start distinction and automatic
      entry), medium (stock observations of refusal and the full-health case)

24. **No automatic return for repair.** No original rule sends a damaged vehicle of a player
    house to a depot by itself, at any health. The only automatic movements related to a depot
    are claim 23's queued entry, which needs a prior enter order, and claim 6's release
    destinations, including the original's guard-spot return for owners that are not
    human-controlled. The original computer player's own choice to send units for repair belongs
    to its scripts and is out of scope. Our AI sends vehicles to a depot only by ordinary enter
    orders, which follow claims 16 to 23.
    - Backing: `measurable` (H18 g).
    - Confidence: medium

25. **Repair departure cell (Q-0682).** This choice is for a vehicle already in contact,
    not the appearance or walk-out of a newly produced unit. Let (*x*₀, *y*₀) be the depot's
    top-left foundation cell, and *W*, *H* its foundation width and height excluding any bib.
    All candidate coordinates are integer cells, with signed 16-bit wrapping (S-0205 claim 1).
    The selected cell is the first suitable candidate in the following preference order:

    | Preference | Candidate cells, in order | Additional locomotion test? |
    |---|---|---|
    | Barracks door kinds, if present on the depot type | (*x*₀ + 1, *y*₀ + 2) for the first kind; then (*x*₀ + 2, *y*₀ + 2) for the second; then (*x*₀ + 2, *y*₀ + 1) for the third. Only kinds the type has contribute a candidate. | yes |
    | Naval vehicle factory | Let (*q*ₓ, *q*ᵧ) be the cell of the depot's supplied docking point for this vehicle (claim 18). Try (*q*ₓ, *q*ᵧ + 1), (*q*ₓ, *q*ᵧ), then (*q*ₓ − 1, *q*ᵧ + 1). | no |
    | Foundation exit list, when present and the type is not a hospital | Each displacement (*a*, *b*) of S-0404's foundation exit list, in its given order, gives (*x*₀ + *a*, *y*₀ + *b*). Preserve omissions and repeats, including an empty list. | no |
    | Otherwise, foundation perimeter | In increasing order of *i* from −1 through *W*, include (*x*₀ + *i*, *y*₀ + *H*) followed by (*x*₀ + *i*, *y*₀ − 1). Then, in increasing order of *j* from −1 through *H*, include (*x*₀ + *W*, *y*₀ + *j*) followed by (*x*₀ − 1, *y*₀ + *j*). The corner repeats are intentional. | yes |

    The foundation-list and otherwise-perimeter rows are alternatives, not successive fallback
    searches: exhausting a present list produces no cell. S-0404's Constants and
    `quirks.foundation_exit_lists` own the lists and the switch's alternative; this selection
    uses those same lists. Door-kind and naval preferences do not stop a later row being tried
    when their candidates all fail. No supplied preferred cell is used for repair departure.

    Every candidate must be a legal map cell and give the *free* result from the vehicle's
    cell-entry test (S-0209 claim 5 and S-0211), with no specified approach facing, height level
    or source cell. A high-bridge candidate consequently uses deck-level entry as specified by
    S-0209's entry-layer rules. All non-free results reject it, including temporary obstruction
    and a result that would only make an ordinary moving vehicle wait. Where the table says
    yes, the vehicle's locomotion-specific cell-entry test must also give free. This is not
    a route search and imposes no additional reach-region, shortest-path, nearest-water or
    approach-side preference. A candidate in a non-blocking foundation cell can be chosen.
    The search consumes no random draw and does not rotate its starting candidate with the
    tick, facing or house. Ordinary later pathfinding can still fail to reach the chosen cell.

    With no suitable candidate, the result is no cell: there is no widening-radius fallback,
    random destination, automatic scatter or forced contact release. Claim 6 gives the retained
    contact and retry, and the rally-point and guard-spot overrides. A successful ordinary
    release uses the selected cell as its movement destination, not a new appearance position.
    - Backing: `measurable` (H21). Door kinds combined with repair, the hospital/list bypass,
      a missing rather than empty exit list, and signed coordinate wrap are `unmeasured`:
      no stock-only setup exercising those combinations is established. Foundation-list
      contents and their quirks inherit S-0404's backing. No measurement record is claimed.
    - Confidence: high (candidate order, tests, no draws and no-cell result), medium (stock
      obstruction observations).

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

**F4, original non-human repair-start delay** (claim 15). For the house's difficulty-row
repair-start delay *d* (double, in game minutes), the duration is the single ranged choice
R(trunc(RZ₅₃(225 × *d*)), trunc(RZ₅₃(1800 × *d*))) ticks, inclusive, from the scenario random
stream (S-0202 claim 4, which also normalises reversed bounds). Products round in floating
point under S-0205; each conversion truncates toward zero. The range operation is taken once
at a start, not again while waiting. At start tick *t* and non-negative chosen duration *w*,
the hold is released in the house update of tick *t* + *w*. Negative durations are already
expired and release in tick *t*. Conversion overflow and exceptional spans follow S-0205
claim 7 and S-0202 claim 5. For the neutral and special houses *d* = 0, hence *w* = 0.
Backing and confidence are claim 15's.

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
| Depot waiting distance, ordinary locomotors | 100 | leptons, strict upper bound | Claim 6 |
| Depot waiting distance, ship and hovering locomotors | 200 | leptons, strict upper bound | Claim 6 |
| Waiting-to-settling delay | 3 | ticks | Claim 6 |
| Early guard repair-start distance | 64 | leptons, strict upper bound | Claim 17; measured to foundation centre, not docking point |
| Arrival body facing | 16384 | facing units | Claim 17; map east |
| Settling hold distance | 200 | leptons, strict upper bound | Claim 21; measured to the docking point |
| Settling recheck while moving | 1 | ticks | Claim 21 |
| Serviced contact place | 0 | place index | Claims 20 to 23; Edge cases |
| Infantry and vehicle self-heal intervals, absent-key default | 1000 | ticks | Claim 10 |
| Infantry and vehicle self-heal amounts, absent-key default | 1 | health points per source | Claim 10 |
| Infantry and vehicle self-heal counts of a building type, absent-key default | 0 | sources | Claim 10 |
| Repair-and-sell threshold, absent-key default | 3 | house skill level | Claim 8 |
| Repair reserve, absent-key default | 1000 | credits | Claim 12 |
| Sale threshold, absent-key default | 2 | sale skill | Claim 14 |
| Sale draw bounds | 0, 50 | ranged-choice values, inclusive | Claim 14 |
| Red-health threshold, absent-key default | 0.5 | health ratio | Claim 14 |
| Repair-start delay, difficulty-section absent-key default | 0.02 | game minutes | F4; only original computer players use a nonzero row delay here |
| Repair-start delay range coefficients | 225, 1800 | ticks per game minute | F4 |

## State and ordering

| State | Event | Result |
|---|---|---|
| no depot contact | successful entry request with a place free | contact assigned before arrival (claim 17) |
| completed entry exchange | vehicle still, tethered and entering | depot queues repair; vehicle queues sleep (claim 17) |
| waiting for a docked vehicle | processing finds it still within its starting distance | settling scheduled 3 ticks later (claim 6) |
| waiting for a docked vehicle | moving or outside its starting distance | waiting, next processing after *D* ticks |
| settling scheduled | later processing finds the vehicle still and its locomotor settled | settling step (claim 3) |
| settling | *not needed* | sent away |
| settling | *cannot pay* | asks again after *D* ticks |
| settling | *more needed* or *done* | working; the next step falls due *D* + *K*<sub>u</sub> − 1 ticks later |
| working | a step falls due, vehicle standing still | repair step; the next falls due *K*<sub>u</sub> ticks later |
| working | a step falls due, vehicle moving | the step is taken on the first tick the vehicle stands still |
| working | step reports *more needed* | stays |
| working | step reports *done* or *not needed* | sent away if a destination is available; otherwise contact held in settling with a 1-tick retry (claims 6 and 25); announcement as claim 6 says |
| working | step reports *cannot pay* | contact retained; settling retry next tick, then unsuccessful retries every *D* ticks; low-funds announcement as claim 6 says |
| approaching vehicle | any order other than a repeated enter on this depot | contact released at once; queued entry cleared (claim 20) |
| docked vehicle | any order, unless it gains contact elsewhere | contact and tether kept; the phase continues (claims 20 and 21) |
| settling, vehicle within 200 leptons and moving | processing | recheck next tick (claim 21) |
| settling, vehicle at least 200 leptons away and moving | processing | no step, settling and contact retained; recheck after *D* ticks (claim 21) |
| settling, no departure destination available | departure attempted | no departure, settling and contact retained; recheck after *D* ticks (claims 6 and 25) |
| working, vehicle standing still at any distance | a step falls due | repair step (`quirks.depot_remote_repair`) |
| any phase | depot starts selling | run-away notice to every contact; contacts released at the next processing (claim 22) |
| any phase | depot destroyed | contacts released; docked vehicle stays in *sleep* (claim 22) |
| depot holds any contact | enter order for a damaged vehicle | queued entry; no destination (claim 23) |
| queued entry | tick with a free place and permission | contact taken; *enter* (claim 23) |
| queued entry | tick with a free place, permission refused | gives up (claim 23) |
| infantry inside a hospital | a step falls due (claim 7) | repair step at cost 0 |
| same | *more needed* | stays |
| same | *done* or *not needed* | put out; announcements as claim 7 says |
| building repair on | tick number a multiple of *P*<sub>b</sub> | step (claim 8's table) |
| building repair on | health reaches strength, or money is short | repair off |
| automatic-repair hold clear | eligible funded automatic start (claims 12 and 13) | building repair on; house hold set |
| automatic-repair hold set | its delay has expired at the house update (claim 15) | hold clear; existing repairs unchanged |
| eligible building below the reserve | eligible sale choice succeeds and late tests pass (claim 14) | sale attempt under S-0408 claim 4 |

An object that claims 9 and 10 both heal in one tick ends it with the health that F3 gives: the two
gains add, and the sum is capped at the strength. Both kinds precede an attached parasite's
same-turn delivery, with no later self-healing after that delivery (claim 10; S-0614 claim 18).
The place of healing among the other work of a tick is S-0203's.
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
- **Automatic repair and sale with a threshold of 0 or below.** Claims 12 to 15 apply, with no
  threshold-specific alternative, warning or new quirk. Neither the credit split, the once-per-tick
  repair start nor the sale tests are treated as defects requiring switches. In particular, the
  sale draw's use of tech level rather than a percentage is reproduced as a rule.
- **Repair away from the depot** (quirk `quirks.depot_remote_repair`, default: the original's
  behaviour). A docked vehicle that is ordered away and stands still anywhere keeps receiving
  paid working-phase steps (claim 21), and in settling beyond 200 leptons too. With the switch off,
  a docked vehicle given a move or attack order loses its contact as an approaching one does
  (claim 20), so steps happen only at the depot.
- **Docked vehicle left asleep** (quirk `quirks.depot_loss_leaves_sleep`, default: the
  original's behaviour). After its depot is destroyed, a docked vehicle stays in *sleep* and
  neither fires nor reacts until ordered (claim 22). With the switch off, destruction treats
  contacts as a sale does: each receives the run-away notice and switches to *guard*.
- **Hovering vehicle stranded** (quirk `quirks.depot_loss_strands_hover`, default: the
  original's behaviour). A hovering vehicle whose depot is sold or destroyed after the settling
  step keeps its locomotor switched off and never moves again (claim 22). With the switch off,
  losing contact with a depot switches the vehicle's locomotor on. S-0504 owns the same
  question for naval yards.
- **Several contact places** (departure). The original services only place 0 (claims 20 to
  23). With an imported docking-place count above 1, a queued second vehicle can take place 1.
  It is then accepted, approaches its own docking point and docks, but receives no step while
  the first vehicle is serviced. When place 0 is then released while place 1 is still held, the
  original's next depot processing reads a vehicle that does not exist. The outcome is undefined:
  the expected result is a program error. Stock data is not believed to reach
  this case. Our engine services the vehicle in the lowest-numbered occupied place, one at a time,
  with claim 6's phases. When the serviced place changes, the phases restart at *waiting*.
  The importer warns for a count above 1 on a repair depot.
  - Backing: `measurable` for the unserviced second place (H18 h). The original's outcome when
    place 0 empties is `unmeasured`: it is a fault, not a defined behaviour to reproduce.
  - Confidence: high (only place 0 is read), medium (a second place can be taken through a
    queued entry)
- **The original's computer player** passes claim 8's gate at the absent-key default maximum
  level and threshold. Its difficulty-related repair-start pacing is recorded in claim 15 and
  F4; its separate strategy for repair or sale is not specified here. Our AI can use ordinary
  repair and sell orders. This does not remove
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
| Rules, building section, `NumberOfDocks` | docking-place count *d* (claims 17 and 18) | integer | 1; later absent entries keep the value in force |
| Art, building section, `DockingOffset0`, `DockingOffset1`, and subsequent numbered entries | displacement from the foundation centre including height (claim 18) | integer triple, leptons; only indices below the imported docking-place count are read | keep the value in force; a newly added place starts at (0, 0, 0) |
| Rules, building section, `WeaponsFactory` | vehicle-factory flag: naval departure preferences (claim 25), not fixed docking-point selection | boolean | no |
| Rules, building section, `Weeder` | obsolete-unloading flag: fixed docking point (claim 18); excluded from claim 17's ordinary pad exchange | boolean | no |
| Rules, building section, `Helipad` | pad flag in claim 17's tether exchange | boolean | no |
| Rules, building section, `Refinery`, `Bunker` | resource-unloading and tank-bunker exclusions (claims 17 and 18); their separate entry rules are not redefined here | booleans | both no |
| Rules, building section, `GDIBarracks`, `NODBarracks`, `YuriBarracks` | first, second and third barracks door kinds for claim 25's optional preferences | booleans | all no |
| Rules, building section, `Naval`; vehicle section, `MovementZone` | naval permission comparison and amphibious exception (claim 16) | boolean; named movement zone, S-0504 claim 7 | no; S-0211 owns the zone default |
| Rules, building section, `Bib` | bib in the foundation-blocking test (claim 18) | boolean, S-0209 claim 5 | no |
| Art, building section, `Foundation` | foundation shape during approach and departure (claims 18 and 25), including dimensions and exit list | foundation word, S-0406 and S-0404 Constants | S-0406 |
| Rules, `[IQ]`, `RepairSell` | house skill level threshold for automatic building repair and sale (claim 8) | integer | 3; keeps |
| Rules, `[AI]`, `CreditReserve` | repair reserve (claim 12) | signed integer credits | 1000; a missing or empty key keeps the value in force |
| Rules, `[IQ]`, `SellBack` | additional sale threshold (claim 14) | integer; unsigned interpretation at the comparison | 2; keeps |
| Rules, `[AudioVisual]`, `ConditionRed` | red-health threshold (claim 14) | decimal or percent, double, S-0003 claims 6 and 7 | 0.5; keeps |
| Rules, difficulty sections, `RepairDelay` | original computer player's repair-start delay (F4); does not impose a delay on a human or AI house or supply a row to neutral or special houses | decimal game minutes, double | 0.02 when a present difficulty section omits the key; absent section keeps its row |
| Map, `[Structures]`, computer-repair field | per-building repair permission for a non-human owner (claim 13) | positional integer, nonzero means set; S-0009 claims 36 and 40 | clear |
| Rules, building section, `Repairable`, `ClickRepairable` | common automatic eligibility (claim 12) | booleans, S-0408 claim 14 | both yes |
| Rules, building section, `Factory` | building-factory exclusion (claim 14) | named value, S-0003 claim 21 | no factory; missing or empty keeps |
| House tech level | sale draw's success bound (claim 14) | setup value, S-0009 claim 45; not the building type's tech level | player houses receive the game's tech level; neutral and special houses start at 1 |
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
step; the first step follows claim 6's waiting gate and 3-tick delay; the last step brings health to
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
*repair* behaviour rate of 0.015625 (*D* = 14). *Observable:* record the tick the vehicle comes
to rest, the waiting processing that accepts its distance, and each health-changing step using a
read-only recorder. *Expected:* no step merely on coming to rest; with the vehicle remaining
still with its locomotor settled, settling occurs 3 ticks after the accepting processing,
the second step 126 ticks later
(14 + 113 − 1), then one every 113 ticks. With the vehicle repair rate absent (0.016): every 15
ticks after the second. Repeat S-0504 N8's land, ship and hovering comparisons for the strict
starting distances and spatial-distance calculation. *Falsified if:* the gate-to-settling interval
is not 3 ticks in the stable case, a second step comes other than 126 ticks after settling,
or any later interval differs from 113 ticks (15 at the absent rate). Intervals of 112 or
14 would mean the depot uses the truncated building period. The
tick on which the vehicle leaves the depot after the step that brings it to full health is recorded
too.
- *Part b (full health):* a vehicle at full health docks. *Expected:* it is sent away on the tick it
  takes its settling processing, 3 ticks after the accepting waiting processing, with no healing,
  no payment and no repair-complete announcement (the local player owns the depot). Repeat with
  the paying house at 0 credits. *Expected:* the same settling-time departure. *Falsified if:*
  a charge, departure at another processing in the stable case, or an
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
  *Expected:* the vehicle stays in contact and returns to settling, with its first retry on the
  next tick and then every *D* ticks while funds remain insufficient. After observing two failed
  retries give the house credits for two more steps. *Expected:* a paid settling step at the next
  retry without another enter order, followed by claim 6's working cadence and departure.
  In a second run the house has 1 credit left after the first settling step (still below the
  step cost): *expected:* the low-funds announcement is not heard. *Falsified if:* a step is paid
  without money, contact ends merely for lack of funds, repair cannot restart without an order,
  retry timing differs, the announcement is missing at 0 credits, or it is heard at 1 credit.
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
  largest distance and the longest gap at which run 2's second announcement stays silent,
  as observations toward S-0701 claim 28. Use S-0410 claims 14–15's zoom progression and cadence
  and its R13 recording setup to identify the zoom-completion frame and its tick number.
  Expect expiry after S-0701 claim 28's post-zoom tail counted from that tick, not from repair completion.
  A fixed total tick duration or a zoom restart on restoring radar would falsify the timing rule.
  R13's freeze and stand-still controls distinguish drawn-frame zoom from the tick-based tail.
  If zoom completion cannot be identified, the gap is inconclusive for the exact tail.
  These runs do not isolate the full *unit repaired* suppression row or cross-creator
  suppression. S-0701 claim 28, G7 proposes tests for the row and trigger/depot pair in both
  directions; neither scenario asserts exhaustive event-creator coverage.
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

- *Part e (parasite delivery ordering):* use S-0614 claim 18's delivery-order setup: a
  non-organic vehicle host with an attached drone parasite, whose nonlethal delivery resolves
  to 10 health points of damage. Arrange a delivery on a due self-healing tick, with the host
  already at full health before its turn. Test individual self-healing alone, house-wide
  self-healing alone with amount 2 and one source, and both together. Record health before
  and after the host's turn, excluding every other health-changing event. *Expected:* in
  each run the host ends that turn 10 below strength, not 9, 8 or 7 below it: the due gains
  cannot heal damage delivered later in that turn. Any other final health falsifies the
  corresponding ordering rule. Repeat each variant three times under the common setup.

**H9: exclusion by the skill-level gate (claim 8).** The human house has 100 000 credits and two
damaged buildings of a repairable type, with individual and house-wide self-healing disabled,
and gives no repair or sell order. No map action heals, repairs or sells them. Record health,
credits, repair state and whether each building is sold over 3000 ticks.
- *Run 1:* threshold at the absent-key default 3, then repeat with threshold 1.
  *Expected:* no automatic repair or sale. *Falsified if:* either occurs.
- *Run 2:* repeat with threshold 0, then −1, the reserve below the house's credits, and neither
  building in construction or selling. *Expected:* one repair starts in the first eligible tick,
  the other in the next, in object-turn order (H13 and H15). Either failing to start falsifies it.
- *Run 3:* as run 1, also pre-place two damaged repairable buildings for each of the neutral
  and special houses (S-0009 claim 27). Their starting credits follow S-0412 claim 2; do not
  assume that the lobby gives them the human's money. *Expected:* none starts repair or is
  sold. *Falsified if:* either event occurs for any of the four buildings. Their skill level
  itself is independently tested by S-0412 K10 and S-0209 C7, not inferred from silence here.
- *Run 4:* repeat run 3 with threshold 0, then −1, reserve 0 and the computer-repair map flag
  set on the test buildings. *Expected:* each non-player house starts one repair per eligible
  tick in its own object-turn order, even at its initial 0 credits. Choose a tick off the periodic
  repair boundary to observe the start before an unaffordable paid step stops it. Failure to
  start under these conditions falsifies claims 12 and 13; H13 and H15 isolate the conditions.

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

For the newly mapped automatic-action fields, repeat H13 a with the reserve omitted from
all layers: *expected:* the start boundary is 1000 credits (test 999, 1000 and 1001). Repeat
H14 c with the red-health threshold omitted: *expected:* draws at ratios 0.49, 0.5 and 0.6,
but a sale is possible only at 0.49. Any different boundary falsifies the respective absent
value. The exact absent sale threshold 2 is `unmeasured` here: sale skill 0 distinguishes 0
from a nonzero threshold, not 2 from other positive thresholds. The original computer player's
absent row delay 0.02 inherits claim 15's unmeasured control limitation. Neither limitation
leaves an unspecified engine action for human, AI, neutral or special houses.

**H13: automatic repair eligibility (claims 12 and 13; Q-0341).** Use a tiny flat test map
with a completed ordinary building of strength 1000 and cost 1000, repair step 10, percentage
25 %, building repair rate 0.125 (period 112), and automatic repair-and-sell threshold 0.
Disable self-healing. Give no repair or sell orders except the explicit stop test. Damage a
full-health test building by one scripted force-fire shot from an own test unit, well between
multiples of 112. Observe the first ordinary building turn after the hit, repair state, health,
credits, behaviour and the scenario random stream. Use the read-only recorder and scripted
input, not an engine modification; the common setup and three repeats apply.
- *a, reserve:* human owner, repair reserve 100, credits 99, 100 and 101 in separate runs.
  Set the sale threshold to 1 to exclude sale draws. *Expected:* no automatic start at 99,
  at 100 and 101, repair starts on the first eligible turn without payment or a repair-start
  draw. Health first rises by 10 at the next multiple of 112, costing 2. Repeat the equality
  test with automatic threshold −1, with the same result. A different credit boundary, a
  later start or a start draw falsifies the corresponding claim.
- *b, common eligibility:* repeat the funded run with full health, each repairability flag off
  separately, a vehicle-like building, and a building still in construction. *Expected:* none
  starts automatically while its exclusion holds. Complete construction on a damaged control:
  it starts on its first eligible turn, not at a special damage or 112-tick boundary. A start
  in an excluded state, or failure after that state ends, falsifies the eligibility rule.
- *c, house and map flag:* pre-place damaged buildings for neutral and special houses, reserve
  0 so their initial 0 credits pass. Each house has an otherwise identical pair with the
  computer-repair map flag clear and set. *Expected:* only the flagged one starts; the
  unflagged one never does. Repair can turn off at a payment boundary for lack of money,
  so record state between boundaries rather than treating lack of healing as lack of a start.
  Repeat the pair for the human owner: both start, at most one per tick. A different result
  falsifies the owner/flag distinction. The original cannot test our AI-house policy.
- *d, reserve is not step affordability:* human credits 1, reserve 0, same step cost 2.
  *Expected:* automatic start off the periodic boundary, repair off with no payment at the
  next multiple of 112, then automatic restart on the next eligible turn. In a funded run,
  turn repair off by order between boundaries: it restarts on the next eligible turn after
  the order. A failure to start, payment without funds or permanent stop falsifies the claim.
- *e, captured mark:* pre-place a full-health human-owned building whose map repair flag is
  clear. A timed map trigger transfers it to the neutral house (S-0011 claim 24), then an own
  test unit damages it. Use reserve 0. *Expected:* unlike the never-transferred control in
  part c, it starts repair despite the clear map flag. Repeat with transfer to the special
  house, and with successive transfers through both houses. A successful transfer followed
  by persistent failure to start falsifies the mark rule. The tag does not prevent repair.
- *f, skipped work:* in a funded run, damage the building, let a named hostile erasing unit
  lock it, and use a repair-off order if repair is already on. Record that repair is off
  while the erasure lock remains. Remove the attacker before erasure completes. *Expected:*
  no automatic restart during erasure, start on the first eligible turn after release. Starting during the excluded
  state or failing afterward falsifies the exclusion; record the state transitions, not merely
  the drawn fading effect. Building phase-in remains unmeasured for claim 12's stated reason.

**H14: automatic sale and its draw (claims 12 and 14; Q-0341).** Same map and recorder as H13,
reserve 100, human credits 0, both automatic thresholds 0, red-health threshold 0.5. Use a
completed repairable building with build-up art, no tag and no building-factory kind. Give it
a surviving hit from a named hostile stationary turret placed for the bystander, then destroy
that turret with an own unit. Keep the damaged building alive below half health. Those named
attackers are part of the observation, not a violation of the bystander void rule. Record exact
health, the attacking object and alliances, each building turn, selling behaviour, credits and
random states before and after each turn. Decode ranged choices with S-0202 claim 4 rather
than assuming one raw value per choice. Repeat each control three times and the baseline
until 100 eligible sale turns or a sale, over at least 20 games.
- *a, baseline and timing:* after the hostile hit, *expected:* one R(0, 50) on every eligible
  building turn while below reserve; selling starts on the first such choice less than the
  house tech level. A gap tied to the repair period, a different range or stream, extra choices,
  or a different start predicate falsifies the claim. Use the recorder's actual random states,
  not a requested seed. Other effects' draws are distinguished by recording within this turn.
- *b, mark:* a map-damaged control never hit by a hostile object, and a control damaged only
  by own force fire, take no sale choice. After one hostile hit the choice begins; after healing
  it above half health, it keeps drawing but cannot sell. Damage it below half by own fire:
  it can sell again, showing that healing did not clear the mark. Any choice before the mark,
  or a lost mark after healing, falsifies it.
- *c, late rejection:* keep the hostile-hit mark but in separate runs give the building a tag
  whose trigger never fires, make it a building factory, or leave it at health ratios 0.5 and
  0.6. *Expected:* still one ranged choice per eligible turn, but no sale even on a successful
  choice. At ratio 0.49 the otherwise identical control can sell. Disable each repairability
  flag separately: no choice. A stopped draw in a late-rejection case, sale at equality or
  continued draws with a common eligibility failure falsifies the rule. Repeat with the
  computer-sell map flag clear and set: no change in draws or sale eligibility.
- *d, skill and credits:* repeat below reserve with sale threshold 1 and then −1:
  *expected:* no sale draw at either threshold for the level-0 house, unlike threshold 0.
  Repeat with automatic repair-and-sell threshold 1: no automatic draw or start. At credits
  equal to reserve: no sale draw; repair may start instead. Any contrary choice or action
  falsifies the respective comparison.
- *e, tech bound:* named test base-rules files choose the game's tech level before a fresh
  program start (S-0009 claim 45), using 0, 1 and 51 separately. *Expected:* one choice each
  eligible turn at all three levels, but success only for a result below the chosen level,
  level 0 never sells and level 51 sells on the first choice. Repeat with a negative tech
  level: every choice passes its unsigned bound. A missing draw at 0 or another bound
  falsifies the rule.
- *f, non-player houses:* repeat with neutral and special test buildings hit by an own named
  attacker not allied from the victim's side, a positive reserve, and their actual credit balance
  below it. Their tech level starts at 1. *Expected:* one choice per eligible turn and a sale
  attempt exactly when the choice is 0 and the late tests pass, with no computer-sell flag
  requirement. A different draw count or bound falsifies the shared rule. If the chosen
  alliances do not allow a hostile hit, the run is void rather than evidence about the mark.
- *g, sale effect:* repeat a successful-choice run with the unsellable type flag set. *Expected:*
  it still starts selling; the input sellability gate is not reapplied. A no-build-up ordinary
  type still takes its choice but is not sold, as S-0408 claim 4 says. With a pending delayed
  kill, a successful choice does not start selling. Record the attempted choice and outcome.
  contradictory outcomes falsify these qualifications, not the range rule.

**H15: one start per house per tick and credit ordering (claims 13 and 15; Q-0341).** H13's
funded human setup, reserve 100, three otherwise identical damaged buildings. Arrange damage
in the same tick off a multiple of 112, using three own attackers; record which buildings are
eligible before the first one's turn. Keep that eligible set fixed and use S-0203 claims 3 and
4's recorded order. *Expected:* the first eligible building starts, the second starts in the
next tick, and the third in the next, with no repair-start draw. Repeat with their map order
reversed: start order reverses, not a position-based or type-based order. Repair already on
is not paused while another building starts. A second new start for one house in a tick,
a skipped eligible turn, or a start draw falsifies the claim.
Repeat for neutral and special houses with reserve 0 and all map repair flags set; start them
off a periodic payment boundary. *Expected:* one start per house per tick, and no raw value
consumed by the equal-bound delay choice. Different houses can each start a building in the
same tick. Include a transferred, flag-clear building
from H13 e in that house's ordering too. Extra raw consumption or an extra waiting tick falsifies the
zero-delay and hold-release rule.
Finally use two human buildings in turn order: A already repairing, B eligible for a sale
choice, both sufficiently damaged. Credits and reserve are 100, A's step cost is 2 and the
next tick is a multiple of 112. Set both automatic thresholds 0 and B's hostile-hit mark.
*Expected:* A pays 2, then B sees 98 below reserve and takes a sale choice. Reverse their
order: B sees 100 and starts repair without a sale choice, before A pays. Record state and
draws within each turn and the later house update. Any opposite credit split falsifies the
within-turn consequence. Void a run with another credit-changing event. Nonzero repair-start
delays of the original computer player remain unmeasured for claim 15's stated control limit.

Aircraft pad repair timing and release are specified by S-0501 claims 26 and 27, with scenario
AC16 covering step size and cost; they are not open questions here.

**H16: land docking (claims 16 to 18; Q-0641).** Use a tiny flat skirmish map with each
stock land depot type in separate runs, two own stock ground vehicles, ample credits and one
isolated bystander computer player. Director and mods off, game speed index 2, 800 × 600.
Repeat each part three times. A read-only recorder logs input action, issued orders, positions,
body facing, contact places, tether, behaviour, building processing and health every tick.
- *a, ordinary entry:* damage one vehicle and click the depot with it selected, from each
  accessible side. Expected: enter order, contact before arrival, destination directed to the
  depot's supplied point, map-east arrival facing, depot repair and vehicle sleep at completed
  entry, then claim 6's waiting and settling timing. Where the guard-start conditions of claim
  17 occur first, record that earlier repair scheduling instead. Distinguish the unarmed
  depot's initial idle-body selection from its later guard waiting stage: the early transition
  is allowed only in the latter. Repeat with an otherwise identical armed depot supplied by
  named test-map overrides, with no enemy target in range: its guard processing must not take
  this early transition. Either excluded transition falsifies claim 17's qualification.
  Repair on the initial click,
  a neighbouring-water or exit-cell destination substituted for the docking point, a point
  changing with approach side, or a different recorded transition falsifies the relevant rule.
- *b, eligibility:* repeat at full health, then with no credits. Expected: entry is still allowed,
  full health leaves without payment at settling, and an injured unaffordable vehicle stays for
  retries. Repeat with a depot switched off and one in selling: no new enter action. Repeat on
  an allied bystander's depot, keeping that building in view and excluding its other objects
  from the observed area. Expected: the own vehicle can enter and its owner pays; the depot's
  owner need not pay. Record sounds too: completion at that non-local depot produces no local
  repair-complete announcement, unlike the own-depot control. Exhaust the vehicle owner's funds
  during working and leave the allied depot owner funded: no local low-funds announcement.
  An own-only restriction, damage/money entry gate, payment by the depot owner, or either local
  announcement at the non-local depot falsifies the relevant claim. Void a run if the bystander
  sells or loses the allied test depot.
- *c, contention:* send the first vehicle, wait until its contact is recorded, then click with
  the second selected. At a one-place depot expect no second enter order, no displacement and
  no repair of the second vehicle. Repeat the click after release: entry can succeed. A
  second vehicle automatically entering without another order falsifies this sequential case.
  H18 f covers orders issued before either contact.
- *d, refusal controls:* attempt entry during build-up, after switching off, and with a vehicle
  already standing on the same depot. Expected: no new acceptance in each case. Restore the
  operational depot and move the vehicle off it: the ordinary accepting case returns. Any
  acceptance while a listed exclusion holds falsifies that exclusion.
- *e, position:* record the supplied point relative to the foundation centre and world height.
  For isolated displacement tests use a named test map with otherwise identical depot types:
  counts 0, 1 and 2 and invented displacements (32, −48, 16) and (−64, 24, 0) leptons.
  Record the supplied point even if a displaced height prevents the waiting gate. Expected:
  claim 18's componentwise sums, count-0 fallback and no facing rotation. A different point
  falsifies the formula. This tests point selection, not simultaneous servicing of two places.
  Use screen capture and scripted input alongside the recorder; no harness changes are assumed.

**H17: healing while sinking (claim 19).** Repeat S-0504 N10's sinking setup with a stock
vessel and an owned vehicle-healing source. Use a named test map setting the house-wide vehicle
healing interval to 1 tick and its amount to 2 health points, and a separate variant granting
individual self-healing with building repair period 1 tick. An otherwise identical control has
neither source. Keep further damage out of the observed area after the sinking hit. Use the
common speed, resolution, repeat count and recorder; log health, sinking and height per tick.
Expected: eligible sinking turns still gain the applicable health, capped at strength, while
height and removal follow the same sinking rule as the control. No healing, a changed schedule
or amount, or recovery that cancels sinking falsifies claim 19. Void a run that destroys the
vessel outright instead of entering sinking.

**H18: ending and losing depot contact (claims 20 to 24; Q-0641).** H16's map, settings, repeat
count and recorder, with credits, contact places, tether, queued entry, locomotor state and
health logged every tick. Use one stock land depot and stock tracked and hovering vehicles,
damaged to half strength unless a part says otherwise.
- *a, orders:* give a move, stop or attack order (i) while the vehicle approaches, (ii) after it
  docks. Repeat the enter order on the same depot while approaching. Expected: (i) contact
  released in the order's tick, and a second vehicle can then take the place; (ii) contact and
  tether kept; the repeated enter changes nothing. Release while docked, or a reset approach on
  the repeated order, falsifies claim 20. (iii) Give a docked damaged vehicle an enter order on
  the same depot through scripted input. Expected: contact ends one tick later and repair stops.
  Repeat at full health: contact ends in the order's tick.
- *b, moving away:* order a docked vehicle 6 cells away during working and let it stand still.
  Expected: health keeps rising by steps, credits fall, and at full health the vehicle is sent to
  the rally point or exit cell, not left at the ordered cell. Repeat during waiting: no step while
  away, place still held. Repeat during settling within 200 leptons: the depot rechecks each tick
  until the vehicle stands still. Any distance-based stop in working falsifies claim 21.
- *c, sale:* sell the depot (i) with a docked vehicle, (ii) with an approaching one. Expected:
  both switch to *guard*, the depot destination is cleared, contacts end one depot processing
  later; record any scatter movement.
- *d, destruction:* destroy the depot (scripted damage outside the vehicle) in the same two
  cases. Expected: contacts released; the approaching vehicle falls back to *guard*; the docked
  vehicle stays in *sleep* and does not fire at an enemy test unit placed in range. Then
  destroy a docked vehicle instead: the depot returns to *guard* and accepts another vehicle.
- *e, locomotors:* repeat c and d with a hovering vehicle after its settling step, then give
  move orders. Expected: it never moves. Repeat with a tracked vehicle and record whether
  it moves (claim 22's low-confidence part).
- *f, power and simultaneous orders:* (i) during working, drop the owner's power below demand,
  then switch the depot off. Expected: steps, costs and timing unchanged. (ii) Select two
  damaged vehicles and click the depot once, so both orders are issued before either vehicle
  takes contact. The first applied enter order takes contact before the second is applied
  (S-0213 claim 3).
  Expected: one approaches; the other stays put with a queued entry and enters by itself at most
  one tick after the first leaves. Repeat with the second at full health: it stays put and never
  enters. Repeat with the depot switched off before the first leaves: the queued vehicle gives
  up when the place frees.
- *g, no automatic return:* leave a damaged vehicle of the human house idle near the depot for
  9000 ticks with no orders. Expected: it never enters. Any automatic entry falsifies claim 24.
- *h, two places:* a named test depot type with a docking-place count of 2 and two invented
  displacements. Order the second vehicle in the same tick as the first. Expected: it takes place
  1, docks at its own point, and receives no step while the first is serviced. Stop recording
  before the first leaves, because the original's outcome there is undefined.

### H19 — Tether, repeated entry and off-centre stopping (Q-0680)

- **Setup:** tiny flat land-and-water test map, stock data only, director and mods off, speed
  index 2, 800 × 600. Use an own stock land repair depot and driving vehicle, then a stock
  naval yard and compatible ship. Damage the vehicle before entry. Record the imported pad
  flag and supplied docking displacement for each stock type without changing either.
- **Procedure:** order entry from several approach sides. Record initial contact, entry
  processing ticks, destination, tether onset, occupied cells, world position, moving/stopped
  state, first repair processing and first health/credit change. A read-only event trace would
  be needed; propose its addition to the existing bench, not a modified executable.
  (a) Repeat with a second move order executed on each side of the first tether tick. Also
  follow entry processings after the vehicle first occupies a foundation cell.
  (b) Use a stock docking displacement off the centre of its cell where one exists. Compare
  the final driving/ship position with both that cell's centre and the supplied point.
- **Expected:** no tether on initial contact; ordinary pad tether follows claim 17's centre-cell
  condition, not first foundation occupancy or first repair. A new order before tether frees
  contact, and one after tether retains it subject to claim 20. Repeated entry does not run
  the already-standing permission refusal. On level tracks the final horizontal position is
  the docking cell's centre. Paid repair still waits on the supplied-point distance gate.
- **Falsification:** tether at reservation, unconditional foundation-cell refusal during
  continuing entry, or exact off-centre snapping. If no stock type separates a tested condition
  (including pad combinations), report it untested rather than editing its rules.

### H20 — Moving out of settling range (Q-0681)

- **Setup:** H19's stock vehicles and map, plus a second damaged vehicle with queued entry.
  Use speed index 2 and the imported repair processing delay *D* without changing it.
- **Procedure:** after the depot accepts the waiting-distance gate but before its settling
  processing, give a move order away while tethered. Vary the order tick and approach position
  until the due settling check finds a moving vehicle at truncated distance at least 200
  leptons from the supplied docking point. Record distance, movement, processing tick,
  health, credits, tether/contact and the second vehicle's ability to acquire the place.
- **Expected:** no step or credit debit, retained settling and contact, locomotor on in the
  absence of ion-sensitive shutdown, and the next settling processing *D* ticks later. Contrast a still-moving check below 200: next
  processing 1 tick later. Stationary distant settling can take the repair step.
- **Falsification:** distant moving settling takes a step, frees contact, advances to working,
  or uses the near-point 1-tick retry when *D* differs from 1. If stock motion cannot reach the
  distant-moving setup during the 3-tick interval, this part remains unmeasured.

### H21 — Repair departure candidates and a blocked exit (Q-0682)

- **Setup:** H19's stock depot/yard and vehicle types, human-controlled owner, no guard-spot
  return, no rally point. Mark foundation cells, the appropriate exit list and claim 25's naval
  candidates on a tiny map. Stock blockers can be stationary own units or impassable terrain.
  director and mods off, speed index 2, 800 × 600.
- **Procedure:** repair a damaged vehicle to completion. Record the assigned departure cell,
  health/credit ticks, movement and contact-release tick. Re-run with successive preferred
  candidates blocked, then all candidates blocked; remove one blocker after a known tick.
  At the yard, separate the three naval preferences from the first exit-list candidate.
  Repeat selected layouts with different seeds and completion ticks. Finally set an ordinary
  rally point outside the candidate set and repeat with every exit candidate blocked.
- **Expected:** first suitable candidate in the stated order; no seed/tick rotation, no search
  farther out, and no automatic scatter. All blocked with no rally leaves contact held and no
  departure order. Failure from working gives a 1-tick settling retry, and further unsuccessful
  settling retries are *D* ticks apart. Opening a candidate permits release
  at a later due retry. A rally point overrides even a failed exit search, but not the original
  non-human guard-spot exception of claim 6.
- **Falsification:** different preference order, accepting a non-free entry result, changing
  the pick solely with seed/tick, widening the search, releasing contact with no destination,
  or remaining held despite a rally-point override. A bridge part requires a legal stock
  placement that actually presents a bridge candidate; otherwise leave that part unmeasured.
A departure-destination and contact-event trace is a proposed harness addition, not a change made here.

### H22 — Late vehicle-turn queued retry (Q-0683)

- **Setup:** tiny stock map, one-place depot or yard, one contacted vehicle and two damaged
  vehicles with queued entry. Director and mods off, speed index 2, 800 × 600. Arrange separate
  runs with the releasing vehicle/depot before and after each waiter in the actual object-turn
  order; record that order rather than assuming creation order establishes it.
- **Procedure:** release the first contact by completed repair, destruction or an untethered
  move order at a known tick. Record each waiter's movement, due behaviour processing,
  self-healing if applicable, retry/contact tick, queued/current behaviour and destination.
  Reverse the waiters' original order-arrival order without reversing their turn order.
- **Expected:** each retry is after that vehicle's movement, due behaviour and self-healing,
  not a pre-movement or orders-phase batch. The first eligible retry after release wins,
  including in the release tick if its turn is later. A waiter that already retried waits for
  another tick. Successful retry may make *enter* current and assign its destination at once,
  but adds neither a second movement nor same-turn processing of the new *enter* behaviour.
- **Falsification:** FIFO allocation independent of actual turn order, a mandatory one-tick
  reservation delay, a pre-movement retry, or a second same-turn movement/enter processing.
- **Tool:** propose a read-only ordered event trace in the existing bench; do not infer local
  ordering from a video frame containing only the end-of-tick positions.

## Open questions

- Whether drive and other non-hovering land locomotors ignore being switched off entirely
  (claim 22, low confidence). H18 e records it.
- Whether a scatter request after the sale's run-away notice moves a vehicle that is already
  standing on or beside the depot (claim 22; S-0215 claim 19 owns acceptance).
- The former allied-depot blocking question concerns route-point action selection, owned by
  S-0215 claim 13, not a separate permission to cross a blocking foundation. Foundation entry
  remains S-0209 claim 5's.
- The exact phase of the second depot step and the first hospital step (±1 tick); scenarios H4
  and H6 record them. Claim 21 distinguishes the near moving 1-tick settling retry from the
  distant moving *D*-tick retry. Claim 25 and H21 cover a failed departure search.
- Whether a unit inside a hospital can reach full health by another rule, so that *not needed* can
  arise there (claim 7, H6 part d).
- Whether passengers inside a transport (not a hospital) self-heal: the original is believed not to
  heal them while they are inside. This question covers open-topped transports too, whose
  passengers are in play and take turns (glossary, *in play*); no measurement has settled it for them.
- Hospital-specific cross-creator suppression remains unmeasured (claim 7); S-0701 claim 28,
  G7 proposes tests for the *unit repaired* row and trigger/depot pair, not every creator.
  No completed measurement is claimed. Zoom progression and cadence are specified by
  S-0410 claims 14–15, with their exact floating-point equality boundaries still unmeasured.
  The post-zoom tail begins at the completion frame's tick number. There is no separate
  unresolved zoom-duration rule here.

