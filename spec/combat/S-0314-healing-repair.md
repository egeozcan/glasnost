---
id: S-0314
title: Healing and repair in combat (healers, self-healing, house-wide healing, repair depot, hospital, building repair)
topic: combat
revision: 88
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-27
measurements: []
requests: [Q-1156, Q-0341, Q-0641, Q-0680, Q-0681, Q-0682, Q-0683, Q-0674, Q-0790, Q-0794, Q-1158, Q-1162, Q-1610, Q-1876]
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
Claim 6 gives the spacing of a repair depot's processings (Q-1610, in part). Claim 22 answers
that request's question 3 for a depot back in *guard*; the rest of that question is open.
Claims 21 and 22 say when a vehicle at a depot becomes settled, when it stops being settled, and
what that changes. This includes a hovering vehicle being switched off at claim 21's settling
processings until it is down.
A vehicle becomes settled on its movement arrival in the docking-point cell (Terms, claim 17).
Its first repair step then comes 3 ticks after the waiting processing that accepts it, if it is
still settled then and stands still (claim 6). If it is not settled, a vehicle standing still within
200 leptons of its docking point gets the step 4 ticks after that processing; this
case is `unmeasured` and rated medium (claim 21). Claim 6's *production* slot hold, or a hovering
vehicle's wait to come down, can delay the step. The step comes at the first processing that
meets the completed-settling condition of S-0504 claim 8 (Changes for other specs).
A hovering vehicle lost over water at a naval yard has the quirk switch
`quirks.docked_hover_water_loss` (claim 21, Edge cases), whose default is the original's loss.
With the switch off, only a vehicle in contact with a depot or settled at a depot (Terms) is
spared the water test; a hovering vehicle disabled away from any depot is tested as usual
(Q-1876).

## Terms

Shared terms used here are defined in the glossary: *house*, *infantry*, *vehicle*, *aircraft*,
*building*, *type*, *field*, *weapon*, *warhead*, *tick*, *tick number*, *credits*, *departure*,
*importer warning*, *quirk switch*, *round-toward-zero mode*, *local player*, *behaviour*,
*house skill level*, *AI house*, *neutral house*, *special house*, *computer-controlled*,
*human-controlled*, *available credits*, *credit balance*.
S-0205's notation applies: trunc, ⌈ ⌉, RZ₅₃, *single* and *double*. In addition:

- **Settled vehicle.** A state that a vehicle takes on in three ways: on its movement arrival
  at a depot's docking point, at a depot's settling processing, or at the switch-off of a
  disabling of S-0405 claim 20, wherever it is (*Away from a depot*). Taken on at a depot, it can
  last after the vehicle's contact with the depot ends (claim 22).
  - *What makes it settled:* its movement arrival in the docking-point cell (claim 17), whatever
    its state before; and a settling processing that finds it standing still within 200 leptons
    of its docking point, unless the *production* slot hold of claim 6 applies at that
    processing (claim 21); and the switch-off of a disabling of S-0405 claim 20 (claim 21). The
    separate completed-entry confirmation of claim 17 does not make it settled.
  - *What ends it:* each of the events below. This list gives the default; with
    `quirks.depot_loss_strands_hover` off, losing contact with a depot also ends it (Edge cases).
    At every end, a hovering vehicle is switched on again (claim 21).
    - *A destination received* while a depot or tank bunker occupies its cell, an empty one
      included (a cleared destination), by a vehicle whose locomotor then counts as switched
      off (claim 22). For a vehicle that does not hover, that is every moment while it is
      settled. A hovering vehicle counts as switched off only once it is down: while it is still
      above the ground it is powered in S-0503's sense (S-0503 Terms), so a destination it
      receives then does not end its being settled, wherever it is. Only an assignment that takes
      effect counts:
      S-0207 claim 27 rejects an assignment that leaves the destination unchanged, empty to empty
      included, unless a destination-refresh state is present (it is present only during a
      vehicle's placement and its teleporting dock approach, S-0207 claim 27), and a rejected
      assignment is not a receipt.
    - *A settling processing that asks whether it is moving and finds it moving.* This is
      expected only at 200 leptons or more from the docking point (claim 21); a settled vehicle
      within 200 leptons whose locomotor still reports movement is open (Open questions).
    - *A waiting processing that does not accept it* (claim 21).
    - *An enabling* of S-0405 claim 20.
    - *The guard reset of a disabling* of S-0405 claim 20, when its cleared destination takes
      effect (the vehicle still has a destination) while a depot occupies the cell, on the same
      condition of a locomotor that counts as switched off as above. The same
      disabling's switch-off makes it settled again on the same tick (claim 21).
    - *Not an end:* the removal of the vehicle's order-level destination by the depot's own
      settling processing (claim 21) is not a receipt of a destination. It neither ends its being
      settled, nor switches it on, nor stops its locomotor; a settled vehicle standing still
      within 200 leptons takes its settling step at that processing, with its order-level
      destination empty from then on (claim 21).
  - *What it changes:* within 200 leptons of the docking point, the settling step comes only at
    a processing that finds the vehicle already settled and, if it hovers, down on the ground
    (claims 6 and 21). So after an end of being settled, the first settling processing that
    again finds the vehicle standing still within 200 leptons takes no step. A hovering vehicle
    is switched off in S-0503 claim 10's sense when it becomes settled and again at each later
    settling processing that finds it within 200 leptons, settled, standing still and not yet
    down, so it
    comes down on the ground (claim 21). A settled hovering vehicle that is down on the ground
    cannot move while it stays settled (claim 22). Being settled is seen only through these
    effects, through the ends listed above, and, with `quirks.docked_hover_water_loss` off,
    through the water test that spares a vehicle settled at a depot (*Settled at a depot*). For a
    vehicle settled away from a depot that has no contact with a depot and stays in cells that
    neither a depot nor a tank bunker occupies, neither these effects nor that sparing arises,
    and of the ends only the enabling does (*Away from a depot*).
  - *Away from a depot:* a disabling of S-0405 claim 20 makes a vehicle settled wherever it is,
    with or without contact with a depot. The disabling switches every such vehicle's locomotor
    off, which makes it switched off in S-0503 claim 10's sense; a hovering one counts as switched
    off only once it is down (*What ends it*). Take a vehicle that became settled in a cell that
    holds neither a depot nor a tank bunker, and that since then has had no contact with a depot
    and has stood in no cell that a depot or tank bunker occupies. It shows no effect of being
    settled beyond those of the disabling itself (S-0405 claim 20, S-0503 claims 8 and 10).
    While it stays so, no end of being settled other than the enabling can fire for it: it has
    no contact with a depot, so no settling or waiting processing reads it, and no depot or tank
    bunker occupies its cell, so neither a destination it receives nor a later disabling's guard
    reset ends it. The enabling that ends the disabling also ends its being settled. Once it
    stands in a cell that a depot occupies, or gains contact, *Settled away, then at a depot*
    applies. Once it stands in a cell that a tank bunker occupies, a destination that takes
    effect there while its locomotor counts as switched off ends its being settled (*What ends
    it*), and a hovering one is switched on again at that end although it is still disabled
    (claim 21); no settling or waiting processing reads it there.
  - *Settled away, then at a depot:* such a vehicle may still reach a depot while it is disabled.
    A hovering one may move while it is still above the ground if a late order or an accepted
    scatter request gives it a destination (S-0405 claim 20, which rates that movement low; Open
    questions), and a late enter order can gain it contact with a depot (claim 17). Reaching a
    cell that a depot occupies changes nothing by itself. Whether it is still above the ground
    when it gets there is not established: S-0503 gives a vehicle that is not powered no thrust
    but slows it by braking rather than stopping it at once, so it may still be moving when it
    touches down. Both cases follow *What ends it*:
    - If it is still above the ground, a destination it receives there does not end its being
      settled (above).
    - If it is down, a destination that takes effect while the depot occupies its cell ends its
      being settled. That includes a late order, a scatter request's destination and the
      cleared one of a later disabling's guard reset (S-0405 claim 20). A guard reset's
      disabling makes it settled again on the same tick, and settled at a depot (*Disabling
      while docked*, claim 21).
    - A contact gained by an entry request is contact in claim 17's sense, and its movement
      arrival in the docking-point cell makes it settled again (claim 17).
    - With `quirks.docked_hover_water_loss` off, which of these it reaches decides whether its
      spin end is tested (*Settled at a depot*, below), whichever height it arrives at.
  - *Settled at a depot:* a vehicle is settled at a depot while it is settled and the event that
    last made it settled was its movement arrival (claim 17), a settling processing (claim 21), or
    a disabling while it had contact with a depot or while a depot occupied its cell (claim 21,
    *Disabling while docked*). A disabling anywhere else makes it settled but not settled at a
    depot. A vehicle settled away from a depot that then stands in a cell a depot occupies,
    with no contact and made settled by no later event of this list, is still not settled at a
    depot (*Settled away, then at a depot*). Only `quirks.docked_hover_water_loss` reads the
    difference (claim 21, Edge cases); with the switch at its default nothing does. So with the
    switch off, such a vehicle is spared the water test while it has contact (claim 17) or once
    its arrival at the docking point or a later disabling in the depot's cell has made it settled
    at a depot, and is tested otherwise, as in open water. This was decided at the delegated
    sign-off, 2026-10-25: the switch spares a vehicle that has contact with a depot or that an
    arrival or a disabling at the depot made settled, and tests one that reached a depot's cell
    already disabled, with no contact.
  - *Backing:* for a vehicle with no contact with a depot, in cells that neither a depot nor a
    tank bunker occupies, the away-from-a-depot rule is `unmeasured`: every effect it could have
    there is the disabling's own (S-0405 claim 20), so no run tells being settled there from not
    being settled. Confidence medium. Its tank-bunker end is `unmeasured`: no scenario here puts
    a disabled, settled vehicle in a tank bunker's cell and gives it a destination there, and
    the case's effect on the vehicle belongs to S-0405 claim 20 (Changes for other specs).
    Confidence medium. That a destination received while a hovering vehicle is still above the
    ground ends nothing is `unmeasured`, rated medium. In settling, an end would switch the
    vehicle on (claim 21) only until the depot's next settling processing, which comes every
    tick while the vehicle is within 200 leptons and not down, switches a vehicle that stands
    still there off again. Its height over those ticks mixes S-0503 F6's bob term with S-0503
    claim 8's damped lift and retained vertical velocity, so no end-of-tick recording tells a
    vehicle that had lift for a few ticks from one that had none. A longer end needs the order
    to set the vehicle moving before it is down, and whether an order can do that is open (Open
    questions). In working, or after the depot's removal, nothing switches it off again (claims
    21 and 22), so an end would leave it raised; but that needs a destination in the few ticks
    between a disabling and its touching down, and no known input ensures one (S-0405 claim
    20), so no scenario here sets those cases up. How a vehicle settled away from a depot
    then reaches one is `unmeasured`: it needs a late order or a scatter request in the few
    ticks between the disabling and the vehicle touching down near a depot, and no known input
    ensures either (S-0405 claim 20). Confidence low, as S-0405 claim 20 rates that movement.
    Whether it arrives still above the ground or already down is `unmeasured` for the same
    reason, confidence low. *Settled at a depot* is our engine's term for the switch-off
    position: `unmeasured` (design), confidence medium; the rule holds whichever way the
    vehicle arrives.
  - *What is observed:* the ends are stated so that the processings at which the settling step
    comes are fixed. With `quirks.docked_hover_water_loss` at its default, an engine that keeps
    no such state but takes the settling step at the same processings, and switches a hovering
    vehicle off and on at the same moments, conforms. With the switch off, the water test also
    reads whether the vehicle is settled at a depot (*Settled at a depot*).
  The depot's *settling* phase
  (claim 6) and the *settled vehicle* are this spec's senses; neither is S-0503 claim 10's
  settling of a hovering vehicle down to height 0, which this spec calls coming down on the
  ground.
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
- **Vehicle bunker.** A building type with the bunker flag (Import mapping): the *tank bunker* of
  S-0606 (Terms). Claims 6 and 16 to 18 apply their bunker rules to a type that is also a repair depot.
- **No-contact settling hold.** The wait of claim 22 that begins at the first settling processing
  to find no contact, when that processing leaves *special-3* running. From there the depot keeps its repair
  behaviour, processing every tick, while its *special-3* slot animation runs; it returns to
  *guard* at its first processing after *special-3* has ended.
- **Depot contact, docking place.** Contact and tether have S-0215's Terms meanings. A docking
  place is one of the depot's contact places, reserved by an accepted entry request (claim 17).
  Contact can exist before the vehicle reaches the building.
- **Moving destination.** For claim 17 and for the depot's question whether the vehicle is
  moving (claims 6 and 21): the vehicle has an order-level movement
  destination and its locomotor gives the movement report defined in claim 17. Claim 21 gives
  one exception: within 200 leptons of its docking point, a vehicle not settled or, hovering,
  not down stands still or moves by its locomotor's movement report alone. This is not a
  test of position change, speed, a track in progress or stored path steps. The locomotor's
  destination point is distinct from that order-level destination (S-0207 claim 25 for
  driving and ship locomotion; S-0503 claim 16 for hovering).
- **Queued entry.** A vehicle's standing intention to enter one depot. While it holds one, the
  vehicle requests a contact place again on every tick (claim 23).
- **Remembered entry object.** The object an actor remembers as the one it is entering. For a
  damaged vehicle that holds a queued entry for a repair depot (claim 23), it is that depot. The
  term is S-0605's (Terms), and the glossary does not define it.
- **Locomotion class.** The class name of a mobile type, as S-0211 claim 3 gives it (see Import
  mapping).
- **Docking point.** The world position the depot supplies for that vehicle's approach and
  waiting-distance check (claim 18 for repair depots, including naval yards).
- **Building repair.** The repair state of a building that its owner turns on and off with the
  repair order (S-0213, claim 13). It is separate from the repair steps of claims 3 to 7 (claim 8).
- **Terms owned elsewhere.** *Radar event* (S-0011, claim 32), *rally point* (S-0213, claim 20),
  *rate delay* and *processing* (S-0215, F1), and *overload pulse* and *unlimited controller* (S-0601,
  Terms), are used as those specs define them.
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
  processing delay, is the rate delay of the *repair* behaviour of S-0215 (F1). S-0108 writes
  the same delay *D*<sub>p</sub>, keeping *D* for its door-stage count.

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
   because it has reached full health, has stopped being of the right kind, or is gone, the firing
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
   how contact ends. A repair depot treats its serviced vehicle in contact, tethered or not,
   in three phases:
   - *Waiting.* At each processing, normally *D* ticks apart, the depot checks whether the
     vehicle stands still within the starting distance of its docking point: strictly less than
     100 leptons for ordinary locomotors, or 200 leptons for ship and hovering locomotors.
     Distance is the floating-point spatial distance, including height, truncated toward zero
     to integer leptons, not distance to the building centre (S-0504 claim 8).
     A successful check schedules the first settling processing 3 ticks later (the settling
     step comes at that processing only if the vehicle is then settled and, if it hovers, down
     on the ground, below); coming to rest alone takes no step. A check that does not accept
     the vehicle ends its being settled (claim 21).
   - *Settling.* While the depot's *production* slot animation runs, settling takes no
     repair step and rechecks after 1 tick. This matters only for a type combining the repair
     flag with a role that starts that slot (S-0108 claim 20). Otherwise, at that later
     processing, provided the vehicle still stands still, the depot asks for one repair step
     (claim 3). A vehicle within 200 leptons of its docking
     point gets no step unless it is already settled (Terms) and, if it hovers, down on the
     ground: the depot rechecks after 1 tick, and a processing that finds it standing still
     makes it settled (claim 21). A vehicle becomes settled on its movement arrival in the
     docking-point cell (Terms). If nothing has ended that by the acceptance, and nothing ends it before the
     first settling processing (for example an enabling, claim 21), a vehicle that
     does not hover (a driving vehicle or a ship, S-0504 claim 8) standing still gets its settling step at the first settling processing, 3 ticks after the
     accepting waiting processing. If something has, for example a waiting processing that did
     not accept it, or if it had no movement arrival and only claim 17's separate confirmation
     reported its arrival, the first
     settling processing makes it settled and the step comes at the second, 4 ticks after the
     accepting processing. Only a depot type that also has the resource-unloading or the
     obsolete-unloading flag sends the entry exchange that leads to that confirmation (claim
     17), so the confirmation-only case arises at no other depot. Which of the two an ordinary docking gives depends on whether the
     depot's first waiting processing after the arrival accepts the vehicle (Open questions). A
     hovering vehicle's step also waits until it is down. Neither condition applies at 200
     leptons or more (claim 21). On *not needed* the vehicle is sent away at once, as below, and
     no repair-complete announcement is
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

     Every working step that answers *done*, *not needed* or *cannot pay* puts the depot back in
     settling on the tick of that step, whether or not the vehicle is then sent away, and its next
     processing comes 1 tick later.
   *Processing spacing (Q-1610, in part).* The waiting, settling and working checks above happen only at the depot's repair processings.
   For a depot that is not also a vehicle bunker (for that type the processing timing is open:
   Open questions), the outcome of each processing fixes when the next one comes. This table does not specify
   servicing or scheduling for a replacement contact acquired before the depot returns to
   *guard*; those cases remain under Open questions.

   | Phase, and what the processing finds or does | Next processing (if it falls on a turn missed by S-0203 claim 10, it waits until the depot's next update that runs, as in claim 22) |
   |---|---|
   | waiting, no contact (claim 22) | none: the depot returns to *guard* |
   | waiting, the vehicle not accepted (it stops being settled, if it was: claim 21) | *D* ticks later |
   | waiting, the vehicle accepted | 3 ticks later, in settling |
   | settling, no contact, *special-3* slot animation running after this processing's slot changes (claim 22; S-0108 claim 20) | 1 tick later |
   | settling, no contact, no *special-3* running after them (claim 22) | none: the depot returns to *guard* |
   | settling, *production* slot animation running | 1 tick later |
   | settling, the vehicle within 200 leptons of its docking point and not settled or, hovering, not down on the ground, moving or standing still (claim 21; standing still, this processing makes it settled) | 1 tick later |
   | settling, the following outcomes: *cannot pay*; *not needed* with the vehicle sent away; a failed departure; the vehicle moving while 200 leptons or more away (claim 21); a step that starts working | *D* ticks later |
   | working, every outcome | 1 tick later |

   So the working phase is processed on every tick, and the spacing of its steps comes from the
   step count *K*<sub>u</sub>, not from the spacing of its processings. The one longer gap is the
   first working processing, which comes *D* ticks after the settling step that started working.
   This gap is where the *D* in the second step's *D* + *K*<sub>u</sub> − 1 comes from. The end of
   a contact by destruction or departure (claims 20 to 22) neither brings a processing forward
   nor puts it back. (Backing and confidence for this table, row by row: the claim's final
   lists below.)
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
   contact is retained. The working step's repair-complete event and announcement are still
   attempted, with the same owner and suppression tests, even when departure fails; the later
   settling *not needed* retries remain silent. A failed departure from working leaves the vehicle in settling with
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
   *Uncharacterised original branch.* On the transition from a paid settling step to working,
   the original has an additional vehicle-state condition under which a vehicle whose owner
   is not human-controlled sends the depot back to waiting without starting work. The condition
   is not characterised here, so this is an open question, not an unconditional refusal of
   repair. Under our house rules it concerns only the neutral and special houses, not AI
   player houses, just as the guard-spot return above does.
   - **A type that is also a vehicle bunker.** For a type with both the unit-repair and bunker flags,
     each repair processing carries out at most one row of the bunker's table and also runs this
     claim's depot step in the same processing; S-0606 claim 20 owns the combined rules and the
     delay that results.
     - Backing for this bunker case: `measurable` (S-0606 claim 20, scenario K9, a test fixture).
     - Confidence for this bunker case: low, for the combined outcome as S-0606 claim 20 gives it.
   - Backing for the rest of the claim (the bunker case has its own, above): `measurable` (scenario H4, parts a to f, H16 b for an allied depot's owner and
     announcement distinction, H18 b for a moving vehicle, H21 for departure and blocked-exit
     retries, S-0504 N8 for the waiting gate, spatial distance and 3-tick delay; S-0701 claim
     28, G7 for the suppression row and trigger/depot pair), except the following cases. The
     processing spacing table's rows are backed and rated one by one in the table at the end of
     this claim. The *production*-slot settling hold is `measurable` (H23) for its missing step,
     with the combined-role restriction of S-0108 claim 20; that the held processing does not
     make the vehicle settled is `unmeasured`, medium (claim 21). The
     uncharacterised original non-human-owner branch is `unmeasured`: its additional
     vehicle-state condition is not established, so no reproducible setup and expected outcome
     are available. The waiting row with no contact is `measurable` (H4 part h run 7, and H18 d's
     waiting case): H18 b's waiting repeat holds a contacted vehicle in waiting outside its
     starting distance, since a move order given to the docked vehicle while the depot waits
     keeps its contact (claim 21) and every later waiting processing finds it moving or outside
     that distance and does not accept it; run 7 destroys it there. It is rated medium until
     run 7 runs. Its rule is the one the settling row with
     no contact follows (H4 part h runs 1 and 5; S-0108 B18 part g): the processing already
     scheduled finds no contact and the depot returns to *guard*. H4 part h run 2 shows whether
     the depot processes the working phase every tick or only when a step falls due: under this
     claim the reaction comes within a tick of the loss, while processing only at due steps would
     put it at the next step's due tick, a full step interval later (*K*<sub>u</sub> = 113 ticks at
     H4 part a's invented repair rate). The
     settling step's place is `measurable` (H4 part a, which records the waiting processings
     between the arrival and the acceptance) for a vehicle still settled from its arrival: 3
     ticks after the accepting waiting processing. For one that is not, 4 ticks after it, at
     the processing after the one that makes it settled, it is `unmeasured` (claim 21: no input
     is known that ensures an earlier waiting processing that does not accept the vehicle). It is the moment at which an arriving
     vehicle first meets the completed-settling condition that S-0504 claim 8 already puts on
     the first step; the 3-tick delay from waiting to settling is unchanged. `unmeasured` for the
     return to the guard spot of a non-player house: no input of the measurement setup makes a
     vehicle of the original's neutral or special house, or of its computer player, dock at a
     depot in a way that can be timed and recorded, and a player house never takes this path. It
     is justified as a rule recorded from the original's behaviour that the measurement setup
     cannot reach. H18 b tests movement while tethered (claims 20 and 21), including a working
     step falling due while moving and being taken when movement ends.
   - Confidence for the rest of the claim: high (one step per count of *K*<sub>u</sub>, the cost, the end on *done*,
     the waiting thresholds, spatial distance, 3-tick delay, suppression row and trigger/depot
     pair, and departure destination/no-cell outcome); medium
     (that the working phase is processed every tick and that in settling and working the end
     of a contact reschedules no processing, both resting on H4 part h, not yet run; the
     settling step and which earlier event makes the vehicle settled, the movement arrival or
     the processing before the step, the hovering
     vehicle's wait to come down, the waiting row with no contact (until H4 part h run 7 runs), the phase *D* + *K*<sub>u</sub> − 1 and the *D*-tick gap after the
     settling step that it comes from, the return to settling after *cannot
     pay*, the owner conditions of the two announcements, that the settling *not needed* makes no
     announcement, and that *not needed* is tested before *cannot pay*), and low (the return to
     the guard spot; the moving step's exact phase; the uncharacterised non-human-owner branch).
     The *production*-slot settling hold is medium in both its parts: its missing step until
     H23 part a runs, since H23 uses a named test type that no stock type matches and may be
     reported unreached (as S-0108 rates its B18 parts i to k), and its not making the vehicle
     settled.
   - Processing spacing table, row by row. The rows that H4 part h tests are held lower than
     other unrun scenarios, because they are the timings least confirmed by observation.

     | Row | Backing | Confidence |
     |---|---|---|
     | waiting, no contact | `measurable` (H4 part h run 7, with H18 b's waiting set-up) | medium until run 7 runs |
     | waiting, the vehicle not accepted | `measurable` (S-0504 N8) | medium, as S-0504 claim 8 rates this *D*-tick spacing |
     | waiting, the vehicle accepted | `measurable` (S-0504 N8 as committed, for the 3-tick spacing to the settling processing, which S-0504 already gives; H4 part a for the parts that the amendment under Changes for other specs adds to S-0504, and for the place of the first step after it when the accepting processing is the depot's first waiting processing after the arrival, which does not establish which event makes the vehicle settled (claim 21), the 4-tick place being `unmeasured` (claim 21); H4 part h run 5 repeats it, but the row does not rest on it) | high for the 3-tick spacing; medium for the first step's place: 3 ticks after the acceptance until H4 part a runs, and 4 ticks after it, which is `unmeasured` (claim 21) |
     | settling, no contact, *special-3* running | `measurable` (H4 part h runs 1 and 5) | medium until H4 part h runs |
     | settling, no contact, no *special-3* running | `measurable` (H4 part h runs 1 and 5) | medium until H4 part h runs |
     | settling, *production* slot animation running | `measurable` (H23 part a) for the missing step; `unmeasured` (claim 21) for not making the vehicle settled | medium for the missing step until H23 part a runs; medium for not making the vehicle settled |
     | settling, within 200 leptons and not settled or not down, moving | `measurable` (H18 b) | high for the 1-tick recheck of a moving vehicle, which does not rest on how the vehicle stopped being settled, and H18 b observes it with movement and step ticks alone, needing no new recorder; medium for its resting on the order having ended being settled (claim 22), until H18 b and e run, as claim 22 rates that end |
     | settling, within 200 leptons and not settled or not down, standing still | `measurable` (H4 part a's hovering repeat) for a hovering vehicle not yet down; `unmeasured` for a vehicle that is not settled (claim 21) | medium |
     | settling, *cannot pay* | `measurable` (H4 part h run 3) | medium until H4 part h runs |
     | settling, *not needed* with the vehicle sent away | `measurable` (H4 part h run 6) | medium until H4 part h runs |
     | settling, a failed departure | `measurable` (H21) | high for the held contact and the absence of a departure order; medium until H4 part h runs for the 1-tick and *D*-tick spacing of the retries, which H21 reads with H4 part h's recorder, as the rows H4 part h tests |
     | settling, moving at 200 leptons or more | `measurable` (H20) | medium until H20 runs |
     | settling, a step that starts working | `measurable` (H4 part h run 1) | medium until H4 part h runs |
     | working, every outcome | `measurable` (H4 part h runs 1 and 2) | medium until H4 part h runs |

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
   Hospital entry retains the infantry as an object. At arrival at its destination object,
   before entry completion, it raises one entered-by notification on that object's tag,
   with itself as causing object; entry alone raises no destroyed-by-anything notification
   on the infantry's own tag. Release adds no entered-by notification, while a fresh entry
   can raise it again. S-0524 claim 14 owns these raises and their distinction from
   trigger firings, not repair payment or cadence.
   - Backing: `measurable` (H6 parts a to c for own hospitals; S-0524 claim 14, C16 for
     retained-infantry notifications; S-0701 claim 28, G7 for the
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
   It is free. After this gain, an existing damage-smoke system is marked when the resulting
   floating-point health ratio is strictly above the game-wide yellow ratio, or the object's
   height above the surface is below −10 leptons (S-0616 claim 1). Marking stops new particles,
   not existing ones; the system stays attached until it leaves play, so renewed damage before
   then does not create replacement smoke (S-0616 claim 22).
   Berserk plays no part: a berserk object heals as any other (S-0610 claim 6). For a berserk unit with
   no overload pulse, the order of this healing against the berserk countdown within an update has no
   observable effect, because self-healing reads no berserk state and the countdown reads no health
   (S-0610 claim 6), so an engine may run them in either order. For an unlimited controller, the place of
   the countdown step against its overload pulse and its heal is S-0601 claim 16's, and this claim adds no
   order of its own. Backing for the order: `unmeasured` for a unit with no overload pulse, since no
   observable depends on it, for the reason given; for an unlimited controller, as S-0601 claim 16 gives it.
   Confidence: high for a unit with no overload pulse. A vehicle, infantry unit or landed aircraft still in the superweapon warp's
   leaving stage after its warp update skips this healing in that turn (S-0513 claim 48).
   In the original, pending alone does not establish that exclusion; other warp-turn
   continuation boundaries retain S-0513 claim 48's unresolved status.
   **Engine design:** S-0513 claim 51 suppresses this healing for vehicles, teleporter infantry
   and landed aircraft from their first pending update through arrival effects, including
   departure, off-map retries and placement, with no ordinary healing before or after warp
   work in those turns. During arrival hold, including its expiry turn, ordinary healing
   eligibility is considered after the hold-end test; this does not override another healing
   refusal. The extension beyond the confirmed leaving-stage exclusion is a design rule,
   not confirmed original behaviour, and has no quirk switch.
   Objects out of play do not ordinarily take these updates (S-0203); warp updates follow
   S-0513 claim 51's separate scheduling rule. Healing inside transports and hospitals
   remains the separate open question below.
   - Backing: `measurable` for the above-yellow smoke mark (S-0616 claim 1, P23 c).
     The below-surface smoke mark is `unmeasured`: no setup here isolates a healing gain
     below −10 leptons. `measurable` for living objects taking updates (H7) and the leaving-stage
     exclusion (S-0513 claim 48, SW23). The engine-design continuation rule is `measurable`
     in our engine (H8 f; S-0513 claim 51, SW23 part f); original continuation at the other
     boundaries remains `unmeasured` for claim 48's incomplete per-kind evidence.
     The health-0 exclusion is
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

    After this healing of a non-organic vehicle, an existing damage-smoke system receives the
    same mark as in claim 9 when its resulting floating-point health ratio is strictly above
    the yellow ratio, or its height above the surface is below −10 leptons (S-0616 claim 1).
    House-wide healing of infantry and organic vehicles does not perform this test; without
    a later hit or individual self-healing their damage smoke keeps emitting across the
    yellow threshold. Marked smoke ends as claim 9 describes.

    A berserk unit receives this healing like any other. For a berserk unit with no overload pulse, the
    order of this healing against the berserk countdown within an update has no observable effect (S-0610
    claim 6: none of the healing conditions reads the berserk state, and the countdown reads no health).
    For an unlimited controller, the place of the countdown step is S-0601 claim 16's, as claim 9 says.
    Backing: `unmeasured` for a unit with no overload pulse, for that reason; for an unlimited controller,
    as S-0601 claim 16 gives it.

    This healing requires reaching ordinary update work too: the still-leaving-stage
    exclusion of claim 9 and S-0513 claim 48 applies equally to it. In our engine,
    claim 9's design rule from S-0513 claim 51 also suppresses this healing from the
    first pending update through arrival effects. Arrival hold and its expiry turn
    offer ordinary healing eligibility after the hold-end test, without overriding
    another refusal. This extension is engine design, not confirmation of the
    original's unresolved continuation boundaries.
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
    - Backing: `measurable` for the above-yellow smoke mark and its infantry/organic
      exceptions (S-0616 claim 1, P23 c). The below-surface smoke mark is `unmeasured`
      for claim 9's setup limitation. `measurable` (H8 parts a to e, including the building-kind exclusion and
      same-turn parasite ordering; S-0614 claim 18; S-0513 claim 48, SW23 for the
      leaving-stage exclusion). The engine-design continuation rule is `measurable`
      in our engine (H8 f; S-0513 claim 51, SW23 part f); original continuation at the
      other boundaries remains `unmeasured` for claim 48's incomplete per-kind evidence.
      The health-0 exclusion is `unmeasured` for the same destruction-handling reason as claim 9.
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
    S-0504 claim 7's naval match applies. For a type that is also a vehicle bunker, the
    permission follows the bunker's rows (S-0606 claims 2 and 20) and not the depot rows of this
    claim.
    Backing for these two sentences: `measurable` (S-0606 claim 20, scenario K9, a test fixture).
    A vehicle already standing on this very depot does
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

    Ordinary vehicle service docking raises neither an entered-by notification on the
    depot's tag nor a destroyed-by-anything notification on the vehicle's tag merely
    for this entry. Departure adds no entered-by raise either. This differs from retained
    infantry hospital entry (claim 7); S-0524 claim 14 owns the notification distinction.

    For a type that is also a vehicle bunker, the movement-arrival exchange below is this
    claim's (the depot's), and a repeated request is refused when either this claim's check or
    the bunker's eligibility check refuses it (S-0606 claim 20).
    Backing for this paragraph: `measurable` (S-0606 claim 20, scenario K9, a test fixture).

    Contact alone does not set a tether. Each due *enter* processing repeats the entry request.
    Acceptance or an existing tether allows entry to continue. Refusal without a tether releases
    contact and falls back to rest (S-0215 claim 29). This is not a repeat of claim 16's initial
    permission test: entering a foundation cell does not by itself trigger its already-standing
    refusal.

    For a depot with the pad flag and neither resource-unloading nor obsolete-unloading flag,
    the repeated request establishes tether when the vehicle occupies the cell containing the
    depot's foundation centre and either has no moving destination (Terms) or the vehicle in
    contact place 0 is more than 128 leptons from that centre. This is the distance to place 0's
    vehicle's current world position, including height, not its destination or the requesting
    vehicle unless that vehicle occupies place 0. Distance is the floating-point Euclidean
    spatial distance truncated toward zero to integer leptons. Thus an integer distance of
    exactly 128 does not pass. An empty contact place 0 fails this distance alternative:
    it supplies no substitute position, not even the requester's position in another place.
    With place 0 empty, a requester with a moving destination therefore cannot tether through
    this exchange; one without a moving destination still can. No distance to a movement
    destination is tested.
    It need not be at the supplied docking point, have stopped, or have entered settling.

    The movement report used here has these meanings (Q-0790, Q-0794). For driving and
    ship locomotion, S-0207 claim 25 owns the separate destination point's lifetime:
    an accepted movement instruction sets it before path-search success, and retained-goal
    waits, blockages and zero speed do not clear it. Stop or cancellation that stops
    locomotion clears it immediately. Qualifying final-track completion clears it before
    the cell-arrival report, whereas intermediate completion and merely standing in the
    goal cell do not by themselves. S-0207 claim 26 separately removes a cell destination
    at a no-track update in its cell, or applies its approach-destination fallback; this
    cell-goal cleanup does not apply merely because a building's supplied point is in
    that cell. Its exact-point guard exception can nevertheless remove a building goal
    when at rest in guard with all three integer-lepton coordinates equal to the
    locomotor destination, including height. Thus neither successful path search nor
    goal-cell inequality is a substitute for testing whether that point is present.

    | Locomotor | Reports movement when |
    |---|---|
    | driving or ship | its locomotor destination point is present, or its heading point is present and differs from its current horizontal position |
    | hovering | its locomotor destination point or heading point is present: S-0503 claim 5's *is moving*, not its throttle-dependent *is moving now* |

    For driving and ship locomotion the heading-point comparison uses the two horizontal
    integer coordinates only; height does not affect that comparison. A turn in place,
    track in progress or stored path is not independently sufficient. Conversely, a present
    locomotor destination reports movement during a turn or a blockage even without translation.
    Hovering throttle 0 does not clear the report. In every row the separate order-level
    destination must also exist for a *moving destination*. A stationary vehicle with that
    destination can therefore still have a moving destination in this spec's sense.

    **Same-cell enter order (Q-0794 question 3).** Yes: when an accepted enter order assigns
    the depot as the vehicle's destination, it supplies claim 18's docking point to driving
    or ship locomotion even when the point is in the vehicle's current cell. No successful
    path search or new track is required for that assignment. With no movement hold refusing
    the instruction, the locomotor destination point is then present, and the retained
    order-level destination makes this a moving destination despite no translation. Merely
    standing there with the depot as a building destination does not perform the final-track
    completion that would clear the point, nor the cell-destination cleanup of S-0207
    claim 26. This is not indefinite retention for an ordinary same-cell move to a cell:
    that claim's no-track cleanup and exact-point guard exception still apply where
    eligible. Later stop, replacement or qualifying track completion follows S-0207
    claim 25; completion alone clears the locomotor point, not the depot's order-level
    building destination. Entry and servicing effects can separately change that destination. The
    tether test uses the state at the repeated request, not the earlier order-issue state.
    This answer does not bypass claim 16's permission refusal for a vehicle already standing
    on this depot, claim 20's ignored repeated enter, or claim 23's queued entry without a
    depot destination. A refused click or an ignored order supplies no new movement instruction.
    A repeated approach exchange that recognizes the vehicle's cell likewise does not by
    itself erase the point already assigned by the accepted order.
    Without the pad flag this ordinary repeated-request exchange sets no tether.
    A separate completed-entry confirmation can establish tether when received, but an
    ordinary repair depot without resource-unloading or obsolete-unloading flags does
    not supply that confirmation through its repeated entry request, whether or not it has
    the pad flag or the vehicle is already tethered. The movement-arrival
    exchange below does not establish tether either. Such a vehicle can remain untethered
    throughout waiting, settling and working: paid repair does not require tether. Neither
    initial reservation nor occupying a foundation or docking-point cell by itself sets it.

    Reaching the destination cell is distinct from first contact. The completed-entry exchange
    can turn a tethered vehicle toward 16384 facing units (map east) and report arrival once it
    has stopped moving and still has *enter* as its behaviour. Separately, reaching the supplied
    docking-point cell during movement sets the depot's ready mark and queues *repair* at
    the depot and *sleep* at the vehicle, neither with an immediate start. S-0215 claim 29
    already states this ready-mark effect; starts still follow S-0215 claim 11, rather than
    a forced or synchronous repair processing.

    **Arrival does not start a turn (Q-1162).** Docking-point-cell movement arrival itself
    requests no map-east body turn, even for a tethered vehicle facing another way. There is
    no turn scheduled for a later enter processing merely because arrival queued sleep.
    An ordinary depot's repeated request, including the pad case above, does not provide
    the separate turn-producing confirmation. Thus tether alone neither guarantees map-east
    facing nor holds sleep pending such a turn. If the vehicle is ready at the arrival update's
    late start, sleep becomes current there and no further enter processing is needed.
    Neither exact agreement with the docking point's horizontal coordinates nor a tether
    is required for the arrival queues.

    **An existing turn and readiness.** If a driving vehicle is already turning when arrival
    queues sleep, S-0215 claim 11's ordinary readiness test can hold that queue until its
    first start opportunity after the turn ends. This is conditional on an actual existing
    turn, not a command to create one at arrival. Ship and hovering locomotion do not inherit
    that driving-only body-turn hold: body rotation alone does not prevent their sleep from
    becoming current. Their movement and all other readiness conditions remain S-0215
    claim 11's. In every case arrival supplies no extra depot-specific readiness gate.
    Claim 6's waiting checks begin when the depot processes repair, not on the click.
    There is also an earlier start opportunity for an unarmed depot in the waiting stage of
    its guard processing, after initial idle-body selection (S-0215 claim 29): a contacted
    entering unit, standing still at a spatial distance strictly below 64 leptons from the
    foundation centre, causes the depot to queue repair with a 1-tick processing delay.
    This opportunity does not apply to an armed depot's guard processing or to the initial
    idle-body selection itself.
    This guard test uses floating-point Euclidean distance truncated toward zero to integer
    leptons. It is not claim 6's distance to the docking point and never takes a paid step.
    At a one-place ordinary pad depot, place 0 is the requester itself. An uninterrupted
    straight, level approach into the foundation-centre cell, also the supplied docking-point
    cell, can remain untethered throughout repair (Q-0790). Specifically, when all repeated
    requests in that cell occur with a moving destination and a truncated centre distance no
    greater than 128 leptons, none tethers. Movement arrival queues sleep before a later
    enter processing could use the stopped state. Repair still proceeds without tether.
    This is not a general prohibition on tethering at one-place pads: a request in that cell
    without a moving destination, or entry farther than 128 leptons from the centre,
    can satisfy the ordinary test. Physical stopping alone does not establish absence of
    a moving destination; the separate point lifetime above still applies. Cell occupancy alone does not bound spatial distance to
    128 leptons on diagonal or elevated approaches.
    Simultaneous requests are claim 23's; servicing with more than one imported contact place
    is in Edge cases.
    - Backing: `measurable` (S-0524 claim 14, C16 for vehicle docking notifications,
      H16 a and c, H19 a and c; contact, tether, behaviours and processing
      ticks recorded separately; H19 d and e for the empty-place test, movement reports, ready
      mark and one-place approach; H19 f for same-cell assignment and blocked persistence,
      with S-0207 claims 25 and 26's D29 and D30 controls for set, clear and cleanup events). Named test data distinguish pad combinations unavailable
      among stock types; no observation of them is claimed.
    - Confidence: high (capacity, repeated request, cell test, movement reports, empty-place
      result, ready mark, same-cell destination assignment and absence of initial tether), medium (stock approach observations,
      early guard start and the uninterrupted one-place approach); high (arrival itself requests no turn,
      and no extra depot-specific readiness gate); the driving-only existing-turn hold inherits
      S-0215 claim 11's confidence

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

    For a type that is also a vehicle bunker, the docking point is the bunker's quadrant point
    (S-0606 claim 4, applied by its claim 20), not the point of this claim. Backing for this
    sentence: `measurable` (S-0606 claim 20, scenario K9, a test fixture).

    The depot directs the vehicle toward the building as its movement destination, except that a repeated entry request which takes a place gives none (claim 23); the supplied
    docking point, not a searched neighbouring exit cell or the rally point, is the destination
    position. Assignment also supplies that point to driving or ship locomotion in the
    same-cell case of claim 17; S-0207 claim 25 owns its independent lifetime.
    The vehicle remains on the map. Foundation blocking is still S-0209 claim 5's:
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
      This includes an untethered vehicle already in *sleep*, settling or working at a depot
      without the pad flag (claim 17). Repair phase does not confer docked-order retention.
      A move order therefore releases that vehicle, and it cannot retain remote repair by
      that order. Claim 21's continued steps require retained contact; no repair phase
      creates a missing tether.
    - A *docked* vehicle keeps both contact and tether whatever order it is given, with two
      exceptions. First, a vehicle holds one contact, so a successful entry request to another
      building releases the depot. Second, an enter order on the same depot ends the docking.
      A damaged vehicle takes claim 23's queued entry in full, although the depot holds its own
      contact: *enter* is queued without an immediate start, the depot is remembered, and no
      destination is given. *Enter* becomes current at the vehicle's first start after the order
      (the queued *enter* is exempt from the moving test of S-0215 claim 11, so a tethered
      vehicle is ready) and is processed in that turn. That processing is only the repeated entry
      request of claim 17: the depot accepts the vehicle it already holds, a pad depot sets the
      tether again, and nothing is queued, neither *repair* at the depot nor *sleep* at the
      vehicle, and no destination is given. The retry of claim 23 comes later in that same
      turn: the permission refusal for a vehicle standing on the depot (claim 16) clears the
      vehicle's contact, queued entry and destination, queues no behaviour, and the vehicle's
      *enter* falls back to rest at its next processing. So, for a vehicle standing on the depot,
      the contact is lost in the first turn in which the order's *enter* is current, not in a later
      one (Q-1158). A docked vehicle not standing on the depot is not refused by claim 16, so its
      retry is decided by claim 23's permission instead. A vehicle at full health loses its contact at once. Otherwise the order runs
      normally, and claim 21 decides what the depot then does.
    - Backing: `measurable` (H18 a; H19 c for untethered repair-phase interruption).
    - Confidence: high (approaching release, ignored repeat), medium (docked retention under
      each order kind; that the processing of the own-depot *enter* queues nothing and the
      contact is lost in the same turn), low (the full tick-by-tick sequence of a same-depot
      enter order while docked, composed from separately established steps)

21. **Moving away while docked.** The depot's checks of claim 6 continue for a docked vehicle
    wherever it goes:
    - *Waiting:* the vehicle advances only from inside the starting distance. A vehicle that has
      left keeps its contact place, receives no step and blocks that place until it returns,
      leaves by claim 20, or is destroyed. A vehicle is made settled by its movement arrival in
      the docking-point cell (claim 17), and every waiting processing that does not accept it,
      because it is moving or outside the starting distance, ends its being settled. So a
      vehicle is still settled at the acceptance only if no such processing, and no other end of
      Terms, came between its arrival and the acceptance.
    - *Settling:* while the vehicle is within 200 leptons of its docking point (the same
      truncated spatial distance as claim 6) and is not settled (Terms) or, hovering, is not
      down on the ground, the depot holds settling and checks again on the next tick, whether
      the vehicle is moving or not. A processing that finds it standing still makes it settled
      and takes no step; the step can come at the next processing (claim 6). A settled vehicle
      within 200 leptons (down on the ground, if it hovers) that stands still takes its settling
      step at that processing, and its order-level destination, if it held one, is empty from
      that processing on. At 200 leptons or more and in the waiting phase, the depot's question
      whether the vehicle is moving is the moving-destination test of Terms: the vehicle counts
      as moving only while it holds an order-level destination and its locomotor gives claim
      17's movement report (for a hovering vehicle, S-0503's *is moving*). Whether a vehicle within 200 leptons that is not settled, or not down,
      stands still is its locomotor's movement report alone, whatever its destination. The removal neither stops its locomotor nor switches it on, and it does not end the
      vehicle's being settled (Terms). An arriving vehicle is
      settled while it still holds the depot as its order-level destination (claim 17; S-0215
      claim 29), so in an ordinary docking the processing that takes the settling step removes
      that destination. At a
      distance of 200 leptons or more, being settled and being down on the ground play no part
      in whether a step comes, and no processing makes the vehicle settled; there the depot asks whether the vehicle is
      moving: a moving vehicle takes no step, keeps settling and contact, stops being settled,
      and is checked again after *D* ticks, not 1 tick. A stationary vehicle proceeds to the
      settling repair step despite that distance.
      While the depot's *production* slot animation runs, that hold (claim 6) takes precedence: the
      processing takes no step and does not make the vehicle settled, whatever its distance, so
      once the hold ends a vehicle that was not yet settled still takes no step at the first
      processing that finds it standing still within 200 leptons.
    - *A hovering vehicle* is switched off in the sense of S-0503 claim 10 by its movement
      arrival and by the processing that makes it settled, and again by every later settling
      processing that finds it within
      200 leptons, settled, standing still and not yet down on the ground. Those processings
      come every tick until it is down (above). It is switched on again when it stops being
      settled, at every end of being settled (Terms), and also while it is still disabled
      (S-0405 claim 20): the switch-on makes no exception for the disabled state, so a disabled
      hovering vehicle that stops being settled, for example through a waiting processing that
      does not accept it or through an order accepted before the disabling (S-0405 claim 20's
      late order), gets its lift back and rises until a later switch-off of this claim. No
      exception is made for an ion-sensitive shutdown: no vehicle is ever in that state in the
      original, during an ion storm too. From the first switch-off on, its powered lift stops (S-0503 claims 8 and 10),
      so it sinks under gravity and comes down on the ground (on the water surface at a naval
      yard standing on water), which is what lets its settling step come (claim 6). When it is switched on again its lift returns and it rises to its
      hover height (S-0503 claim 10).
    - *Draws of the repeated switch-off.* Each of these switch-offs meets S-0503 claim 10's
      conditions on its own. A hovering vehicle above the ground counts as powered there, so
      each one takes that claim's two draws from the scenario random stream, and restarts the
      spin, whenever the vehicle's current behaviour is then neither *sleep* nor *enter*. A
      vehicle in another behaviour therefore takes two draws at each of these switch-offs while
      it is above the ground; one in *sleep* or *enter* takes none. The
      switch-off at the movement arrival takes none, because the vehicle's behaviour is then
      *enter* (claim 17). The vehicle
      queues *sleep* on reaching the docking point (claim 17), so which behaviour is current
      during these ticks follows its starts (S-0215 claim 11).
    - *Over water, at a naval yard.* No switch-off of this claim is skipped or changed over water:
      a switch-off that takes S-0503 claim 10's two draws starts a spin, and the end of that spin
      brings S-0503 claim 11's water test, at a depot or a naval yard as anywhere. A building
      leaves the land type of the cells under it unchanged, so the docking cell of a naval yard
      standing on water is a water cell. A hovering vehicle docked there (S-0504 claim 7 says which
      can dock; one whose movement zone is plain amphibious, for example) whose spin ends while it
      is less than 104 leptons above the water is given the bomb mark, falls and is lost (S-0503
      claim 11), and the yard then meets the loss of a vehicle in contact (claim 6's rows with no
      contact). The spin ends at the latest on the first tick on which the vehicle has stopped being
      powered (S-0503 claim 10), so one that takes the draws while still above the water is given
      the bomb mark when its spin ends, on the tick it comes down at the latest, and is lost when
      its fall ends (S-0503 claim 11, which leaves the fall's end to S-0312), unless the spin ends
      while it is still at least 104 leptons above the water (S-0503 claim 11: no fall then), and
      one that takes them while down but switched on (as at a disabling before the settling step,
      below) is given the bomb mark within a tick of the switch-off. This applies to every
      switch-off here that takes the draws: the repeated switch-off above, the one after an
      enabling and the disabling below. A switch-off that takes no draws (current behaviour *sleep*
      or *enter*, or a vehicle that is down and already switched off) starts no spin; with no spin
      active, no water test is made, and the vehicle rests on the water and is serviced as at a
      land depot. So whether an ordinary repair visit to such a yard, with no disabling or
      enabling, loses the craft turns on its current behaviour at the switch-offs during settling
      while it is above the water, which is open (above: *Draws of the repeated switch-off*, H4
      part i run 1); run 6 (ii) records it at the yard. At a land depot the spin ends over land and
      the vehicle survives it. This loss is the original's behaviour and the default of quirk
      `quirks.docked_hover_water_loss` (Edge cases). With the switch off, a spin end that comes
      while the vehicle has contact with a depot (Terms) or is settled at a depot (Terms) makes no
      water test: the vehicle is not given the bomb mark and does not fall. Once its contact has
      ended and it is not settled at a depot, a later spin end is tested as usual by S-0503 claim
      11. So a hovering vehicle disabled over open water away from any depot, for example when its
      house loses its last control centre, is subject to S-0503 claim 11's water test, with that
      claim's exception for a spin that ends at least 104 leptons above the water, in the original
      (H4 part i run 7), and in our engine in both positions of the switch. With the switch off,
      the same holds for a vehicle settled away from a depot that then reaches a cell a depot
      occupies while it is disabled, with no contact, as long as no later event has made it
      settled at a depot (Terms, *Settled away, then at a depot*).
      - Backing: `measurable` (H4 part i run 6: a disabling at a naval yard while the vehicle
        is above the water, a no-draw control in working, and a land-depot control). That
        buildings leave their cells' land type unchanged is shown there only for the yard in
        that run. The behaviour with `quirks.docked_hover_water_loss` off is our engine's rule:
        `unmeasured` (design, as the original has no such setting), confidence medium. Its
        limit to a vehicle in contact or settled at a depot was decided at the delegated
        sign-off, 2026-10-25: the switch removes a loss that a vehicle suffers because it is
        being serviced, and the loss of a hovering vehicle switched off over open water is an
        ordinary rule of S-0503 claim 11 that the switch leaves alone. The original's loss away
        from a depot is `measurable` (H4 part i run 7), confidence medium.
      - Confidence: medium until run 6 runs, which uses a named test type that no stock type
        matches and may be reported unreached.
    - *Enabling while settled.* An enabling of S-0405 claim 20 switches a settled vehicle on
      again and ends its being settled (Terms); a hovering vehicle's lift returns and it rises.
      In settling, a later processing that finds it within 200 leptons and standing still (for a
      hovering vehicle, not yet down on the ground) makes it settled again and switches a
      hovering vehicle off again, as above, so the step of a vehicle that does not hover (a
      driving vehicle or a ship) comes 1 tick after that
      processing and a hovering vehicle's waits until it is down again. The later
      switch-off follows the powered and current-behaviour conditions above; the enabling's guard
      reset alone does not guarantee that it takes the two draws. When it takes them at a naval
      yard standing on water, the spin's end by default loses the vehicle (unless
      `quirks.docked_hover_water_loss` is off), unless it ends at least 104 leptons above the water
      (*Over water, at a naval yard*). In the working phase
      nothing at the depot switches it off: it stays raised for the rest of that phase, and its
      height does not defer its steps (only movement does, claim 6).
    - *Disabling while docked.* When S-0405 claim 20 disables a vehicle in contact (one not
      passed over there as tethered to the depot standing in its cell), the net result on the
      tick of the disabling is the same in every case: the vehicle is settled and switched off,
      whether or not it was settled or switched on before, it does not rise, and the contact is
      kept, so the depot goes on servicing it. In settling, the first processing that finds it
      within 200 leptons and standing still (a hovering one down on the ground) takes its step,
      with no extra processing; a hovering vehicle that was still raised loses its lift and comes
      down. The cases differ only in the draws a hovering vehicle takes at the disabling:
      S-0503 claim 10's two draws, from the scenario random stream, when the disabling's cleared
      destination switches it on (it takes effect while it is down and switched off, Terms) or
      when it is above the ground or was still switched on from an earlier end of being settled,
      and none otherwise:
      - *Before the settling step,* at a docking point in a depot cell, the vehicle still has its
        destination, so the cleared destination takes effect: it switches on a hovering vehicle
        that is down, and one still above the ground is powered already, so either way a hovering
        vehicle takes the two draws.
      - *After the settling step,* in settling or working, its order-level destination is
        already empty (claim 21). Unless a destination-refresh state is present (only during
        placement and a teleporting dock approach, S-0207 claim 27), the cleared destination is
        rejected as unchanged and switches nothing on: a hovering
        vehicle already down on the ground takes no draws and stays down; one not yet down takes
        the two draws.
      - *At a docking point outside the depot's foundation* (claim 18's displacement, as in H19
        part f), the serviced depot does not occupy the vehicle's cell, so the cleared
        destination switches nothing on there unless another depot or a tank bunker does
        (Terms); the draws then follow the previous case.

      The one exception to the same net result is a water cell: at a naval yard standing on
      water, a disabling that takes the two draws starts a spin whose end by default loses the
      vehicle (unless `quirks.docked_hover_water_loss` is off) unless it ends at least 104 leptons
      above the water (*Over water, at a naval yard*, above), so its contact ends with its loss and
      the yard
      does not go on servicing it.
      The behaviour condition of S-0503 claim 10 always holds at the disabling, because claim 20's
      guard reset forces *guard* (or keeps *wait*, S-0215 claim 9). A vehicle that does not hover
      takes no draws: S-0503 claim 10's draws are the hovering locomotor's.
      - Backing: `measurable` (H4 part i run 4 before the settling step at a docking point in
        a depot cell; run 5 after the settling step and at a docking point outside the
        foundation). That the hovering vehicle's lift stays off after the disabling is S-0405
        claim 20's and S-0503 claim 10's rule, and these runs record its height without judging
        it (run 4). That no destination-refresh state is present for a vehicle in contact is
        `unmeasured`: S-0207 claim 27 gives refresh only at placement and in its dock approach,
        and no scenario here sets one up.
      - Confidence: medium until runs 4 and 5 run; medium for the absence of refresh.
      - Backing: `measurable` (H4 part i: run 1 for the first switch-off, run 2
        for the enabling in working, run 3 and its land repeat for the enabling in settling, an
        attempt); `unmeasured`
        for the repeated switch-off and its repeated draws: run 1 sets up no later processing that
        finds the vehicle above the ground in a behaviour other than *sleep* and *enter*.
        Confidence: medium for the repeated switch-off, which run 1 tells from a single
        switch-off only if the vehicle is above the ground at a later processing in a behaviour
        other than *sleep* and *enter*; medium for which repeats take the draws,
        since no scenario has yet recorded the vehicle's behaviour during those ticks; medium for
        the enabling.
      - Backing for the switch-off: `measurable` (H4's hovering repeat: the vehicle comes down
        at the depot; a vehicle that kept its lift would never be down and would never be
        stepped). That the loss of lift comes at the movement arrival, and again at the
        processing that makes a vehicle settled after a waiting processing has switched it on, is
        `unmeasured`: the height trace shows the descent only through S-0503 claim 8's height
        update, whose old vertical velocity and bob term can delay its first downward movement.
        Confidence: high for the switch-off and the loss of lift; medium for their tick; medium,
        as in claim 6, for the arrival and that processing being the ones that make the vehicle
        settled; the rate of descent is S-0503's.
      - Backing for the switch-on: `measurable` (H18 part b's hovering repeat, for a vehicle
        that stops being settled by receiving a destination, in a run that has the window that
        repeat needs). The switch-on at the other two ends of being settled (a waiting
        processing that does not accept the vehicle, a settling processing that finds it
        moving) is `unmeasured`, rated medium: H4 part a and H18 e reach the waiting end only in
        runs that no known input ensures (below), and H20 does not record a hovering vehicle's
        height. That a still-disabled vehicle is switched on at an end of being settled is
        `unmeasured`, rated medium: it needs a disabled vehicle in contact to meet such an end,
        through a late order (which S-0405 claim 20 rates `unmeasured`, since it needs an order
        and a disabling in the same tick) or through a waiting processing that does not accept
        it, and no scenario here ensures either. That no vehicle is ever in an ion-sensitive
        shutdown is `unmeasured`, rated medium: no finite set of runs shows that a state never
        arises.
    - *Same-cell enter order.* A vehicle whose accepted enter order found it already in the
      docking-point cell (claim 17) holds the depot as its order-level destination and a present
      locomotor destination point, so it has a moving destination (Terms) for as long as that
      point lasts. If the depot is in waiting, each waiting processing finds it moving and does not accept it, so it
      stays in waiting, checked every *D* ticks, and receives no step. In settling within 200
      leptons, not settled, each processing would find it moving and check again after 1 tick,
      never making it settled. It is accepted only once that point is cleared (S-0207 claim 25).
      Whether and when anything clears it while the vehicle stands there is open (Open
      questions).
      - Backing: `measurable` (H19 part f's same-cell control, with the depot's phase and the
        vehicle's health recorded).
      - Confidence: medium until that control runs.
    - *Working:* only movement defers a step (claim 6). A docked vehicle standing still at any
      distance from the depot, for example after a move order or while it attacks without moving,
      receives its steps and pays for them (`quirks.depot_remote_repair`).
    - When the depot sends the vehicle away at full health (claim 6), the destination it gives
      replaces any destination the player gave while docked. The repair-complete announcement
      follows claim 6.
    - Backing: `measurable` (H18 b, and H20 for distant moving settling). The settling step 1 tick
      after
      the processing that makes the vehicle settled, the only way being settled is seen (Terms),
      is `unmeasured`: H4 part a reaches it only in a run in which an earlier waiting processing
      did not accept the vehicle, which no known input ensures (below). The production hold's
      no-step half is `measurable` (H23 part
      a); its other half, that the held processing does not make the vehicle settled, is
      `unmeasured`: H23 part b tests it only in a run in which a waiting processing after the
      arrival did not accept the vehicle, and no input is known that ensures one (below, for
      why H18 b's waiting repeat does not), so part b
      remains an attempt. After an order received on the depot, a vehicle standing still within 200
      leptons that has stopped being settled (its step then comes at the next processing, as above) is
      `unmeasured`: no order is known that leaves the vehicle standing still within 200 leptons
      of its docking point, since the nearest other cell centre is 256 leptons away, and H18 b's
      settling repeat checks only the moving recheck. The removal
      of a settled vehicle's destination within 200 leptons is `measurable` (H4 part a: the depot
      destination held since the arrival is gone from the processing that takes the settling
      step). That the step comes at that processing is `measurable` by the same part: the step
      comes at the processing from which the destination is gone. Which test the question uses is `unmeasured`: H4 part a does
      not tell the moving-destination test from a test of the locomotor's movement report
      alone.
      That a moving vehicle at 200 leptons
      or more stops being settled is `unmeasured`: H20 does not observe it. It is rated medium:
      the rule is the same at every distance, with no exception, but is seen only through the
      extra processing without a step when the vehicle later stands still within 200 leptons.
      That the movement arrival makes a vehicle settled is `unmeasured`: H4 part a measures the
      first step's place in that case (3 ticks after the acceptance when the accepting
      processing is the depot's first waiting processing after the arrival), but a vehicle made
      settled at the acceptance would give the same timing, so the run does not establish which
      event makes the vehicle settled. It is rated medium, as in claim 6. That a
      waiting processing that does not accept the vehicle ends its being settled, and switches
      a hovering vehicle's lift on again, is `unmeasured`: H4 part a, its hovering repeat and
      H18 e's second-depot repeat test it only in a run in which an earlier waiting processing
      did not accept the vehicle, and no input is known that ensures one there. H18 b's waiting
      repeat does give such processings, but cannot show their own effect: its move order has
      already ended the vehicle's being settled and switched it on (a destination received,
      Terms), and a vehicle that returns to its docking point is made settled again by its
      movement arrival. So those parts remain
      attempts that report which case each run gives. It is rated medium: the switch-on is the
      same at every end of being settled. The disabling's two effects (below) are
      `measurable` (H4 part i, runs 4 and 5).
    - Confidence: high (working-phase distance, release override, and
      the 1-tick recheck within 200 leptons of a vehicle that is moving, H18 b), medium (that
      recheck's resting on the order having ended being settled, claim 22, as in claim 6's
      table; which test the depot's question uses; the
      distant-moving delay, until H20 runs, as in claim 6; the 1-tick recheck of a vehicle standing still that is not yet settled, H4 part a, other stock
      settling observations, the production hold not making the vehicle settled, the removal of a
      settled vehicle's destination, and the end of being settled for a vehicle moving at 200
      leptons or more, for the reasons in the backing; the disabling's end and new start of being
      settled and its draws, until H4 part i runs 4 and 5 run)

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
    - *Settled vehicles.* Becoming settled (Terms) and the settling step that follows are as
      claim 21 states. Normally a vehicle stops being settled when it receives a destination
      while a depot or tank bunker occupies its cell and its locomotor counts as switched off
      (Terms); the depot's own removal of its destination
      at a settling processing is not such a receipt (Terms, claim 21). A settled hovering vehicle that is down on
      the ground cannot move while it stays settled. While it is still above the ground after becoming settled it counts
      as powered in S-0503's sense (S-0503 Terms), so a destination it receives in that window
      does not end its being settled, whether or not a depot still occupies its cell (Terms): its
      lift stays off and it goes on sinking. Once it is down, a destination received while a
      depot or tank bunker occupies its cell ends its being settled, as above. Whether an order
      given in that window can move it is not settled (Open questions). If its depot is destroyed or otherwise removed without a sale after the
      vehicle became settled, a later destination does not end its being settled while no depot
      or tank bunker occupies its cell (Terms). A sale is different: the depot still occupies its cells from the start
      of the sale to its removal, since a building occupies its foundation (glossary,
      *building*) and a sold building stays in play until the processing that finds its
      build-down complete removes it (S-0408 claims 6 and 17), so the run-away notice's clearing of a
      destination that is the depot, the destination of a scatter request that notice brings,
      and a destination that an order gives it before the removal each end the vehicle's being
      settled as above, once a hovering vehicle is down (Terms). A
      vehicle settled in a cell of a sold depot's foundation is still settled after the removal only if no destination
      it received between the start of the sale and the removal ended its being settled (one
      received while a hovering vehicle is still above the ground does not, Terms); one settled at a docking point
      outside the foundation follows the ends of Terms.
      Once it is down, a hovering vehicle stays immobile under every later order
      (`quirks.depot_loss_strands_hover`). No such effect was found for other land locomotors
      (Open questions). Its being settled still ends at any
      other end of Terms, for example an enabling (S-0405 claim 20). Leaving a tank bunker and
      being placed on the map again also switch a vehicle on, but they are not ends that this
      spec states: a vehicle stranded here cannot move to a bunker or a transport, neither can
      happen between a docking and the depot's processings, and the next docking makes the
      vehicle settled whatever its state before, so no observation bears on them. Whether a
      superweapon teleport re-places a stranded vehicle on the map and so lifts it is open (Open
      questions). If it later docks at another depot, its movement arrival there makes it
      settled whatever its state before (claim 21), so its earlier being settled has no effect
      on its timing there: claim 6's timing applies as for any arriving vehicle.
    - *A vehicle destroyed while in contact* frees its place. A depot left without any contact
      reacts at its next repair processing. This is the processing that claim 6's spacing table
      had already scheduled; losing the contact neither brings it forward nor puts it back
      (Q-1610):
      - in working, it is the depot's first processing at or after the tick of the loss. That is
        on the same tick when the depot's own update in that tick comes after the loss (S-0203
        claim 3),
        otherwise on the next tick, provided the depot loses no turn in between. A turn the depot
        misses by S-0203 claim 10 (the vehicle's own removal can be such a cause) moves the
        reaction to its next update that runs. The exception is a loss during the *D* ticks
        after the settling step that started working: the depot then reacts at its first
        working processing, *D* ticks after that step,
      - in settling or waiting, it is the processing already pending, which claim 6's table
        places 1, 3 or *D* ticks after the processing before it: for example 3 ticks after the
        accepting waiting processing, or *D* ticks after a *cannot pay* retry. The accepting
        waiting processing has already put the depot in settling, so a loss in the 3 ticks after
        it is met by the settling row of S-0108 claim 20, not its waiting row, at the
        processing 3 ticks after the acceptance (H4 part h, run 5). So the wait after
        the loss is at most the larger of 3 and *D* ticks, provided the depot loses no turn in
        between. If the pending processing falls on a missed turn, it waits until the depot's next update that runs, as in working.

      These timings apply when no other vehicle has taken the freed contact place by that
      processing; that case is open (Open questions).

      A vehicle that leaves by claims 20 and 21 ends its contact in the same way and gets the
      same timing.
      In waiting the depot returns to *guard* at that processing
      without an animation wait. In working it enters settling and rechecks after 1 tick.
      In settling, from that already scheduled processing on, it stays in its repair behaviour,
      checking every tick, until its *special-3*
      slot animation has stopped running, and only then returns to *guard*. The slot's duration
      from the art data is deterministic simulation state: neither visibility nor drawing-frame
      cadence changes this wait. S-0108 claim 20 owns the slot changes: *special-3* and
      *active-1* start on loss of contact in waiting or working, and in settling when
      *production* or *special-2* is running; *idle* follows when *special-3* finishes.
    - *After a send-away.* A step that sends the vehicle away has already put the depot in
      settling (claim 6), so its next processing finds no contact in settling, not in working.
      That processing follows the settling row above. Whether it changes any slot is S-0108
      claim 20's.
    - *A later vehicle (Q-1610).* Each entry into the working phase starts the step count afresh,
      so no part of a lost vehicle's step spacing carries over to the next vehicle served once
      the depot has returned to *guard*. The no-contact settling hold is excepted (below).
      A vehicle whose arrival at the docking point queues *repair* (claim 17) at a depot that has
      already returned to *guard* restarts the depot from waiting once that queued *repair* becomes
      current, which happens when the depot is ready to change (S-0215 claims 7 and 11). A depot still in its no-contact settling hold has *repair* current. If
      nothing is queued, the arrival queues *repair* without an immediate start (claim 17), so by
      S-0215 claim 7 the queue stays unchanged and nothing is started by the arrival itself; the
      depot's ready mark is still set (claim 17). What that depot then does with the vehicle, and with one that takes
      contact during the hold before it arrives, is an open question.
    - *Power.* Neither the owner's low power nor the depot being switched off stops or slows the
      servicing of a vehicle already in contact: steps, costs and timings are claim 6's. Being
      switched off refuses only new acceptance, including a queued vehicle's (claims 16 and 23).
      Low power refuses nothing here.
    - Backing: stranding of a hovering vehicle whose depot is lost between becoming settled and
      its settling step is `unmeasured`: H18 e loses the depot only after the settling step. The
      immobility of a stranded hovering vehicle once it is down is `measurable` (H18 e: its sale
      repeat's move order after the depot's removal; in its destruction repeat, a run in which
      S-0312 claim 17 destroys the vehicle with the depot is inconclusive). That
      a destination received between the start of a sale and the depot's removal ends being
      settled is `measurable` (H18 e's sale repeat, through its move order), rated medium until it
      runs. For the run-away notice's scatter request it is `unmeasured`, medium: H18 e starts
      after the settling step, where S-0207 claim 16 can refuse the scatter. That a destination
      received while a hovering vehicle is still above the ground does not end its being settled
      is `unmeasured`, rated medium, for the reason that Terms gives.
    - Backing for the other ends and the restart: `measurable` (H18 b for the 1-tick recheck
      within 200 leptons of a vehicle that has received a destination and is moving. It backs the end of being settled on receiving a destination
      only that far: a rule that rechecks after 1 tick whenever the vehicle moves, settled or not,
      would give the same recheck, so that end stays medium; H18 c, d
      and e, with e's second-depot repeat for a vehicle settled at a lost depot that docks
      again; S-0108 claim 20, B18 g for the no-contact
      settling hold and deterministic animation timing; H4 part h for the reaction tick after
      a loss, the fresh step count of a later vehicle and (run 5) a loss just after an acceptance,
      S-0108 B18 part h for the processing
      after a send-away). The restart from waiting for a later vehicle at a depot back in *guard*
      is `unmeasured`: run 4 times that vehicle's steps from its own settling step, not its
      waiting gate, so it is rated medium for that gap. Run 4 tells a fresh step count from one
      carried over from the lost vehicle: its two repeats are set up so that the new vehicle's
      settling steps, counted from the lost vehicle's last step, differ by a number of ticks
      that is not a multiple of 113, so a carried-over schedule would move
      against the new vehicle's settling step between them, while a fresh count gives 126 ticks
      in both. What a depot in its no-contact settling hold does with a
      newly arriving or contacting vehicle is an open question, not a claim.
    - Confidence:
      - high: the sale notice and release, and no power test.
      - medium, resting on H4 part h and S-0108 B18 part h, which are not yet run: a lost contact
        in working or settling reacted to only at an already scheduled processing, the depot
        already in settling after a send-away, and the fresh step count for a later vehicle
        served after the depot's return to *guard*.
      - medium: the no-contact settling hold, and the same for a loss in waiting (until H4 part
        h run 7 runs, claim 6).
      - medium: behaviour after destruction, and hover stranding.
      - medium: the reaction tick within the *D*-tick gap after the settling step, and the
        reaction tick after a missed turn.
      - medium: the restart from waiting for a later vehicle at a depot back in *guard*.
      - medium, resting on H18 b and e, which are not yet run: that a destination received on
        the depot ends being settled, and that a vehicle settled at a lost depot gets claim 6's
        timing at another.
      - medium, `unmeasured` (Terms): no end of being settled at a destination received while a
        hovering vehicle is still above the ground.
      - medium: the settling row for a loss in the 3 ticks after an acceptance.
      - low: the scatter request, and no stranding for other land locomotors.

23. **Simultaneous and queued entry.** Orders execute one at a time in event order (S-0213), so
    two vehicles sent together are resolved in that order. The first entry request finds a free
    place and takes contact (claim 17). For a later enter order on a depot that holds *any*
    contact, whatever its place count:
    - a vehicle below full health (health ratio below 1) gets no depot destination and no contact
      from the order. The order queues *enter* without an immediate start (S-0215 claim 12),
      records the depot as the object the vehicle is entering, and gives it a *queued entry* for
      that depot. The vehicle does not move because of the order: it has no destination, and it
      does not walk to a cell near the depot, because the depot is now its remembered entry object. *Enter* becomes current
      at the vehicle's next per-tick start at which it is ready to change (S-0215 claims 11 and 12),
      which is its first turn after the order takes effect, at the earliest the next tick (S-0215
      claim 12), and is processed in that same turn (S-0215 claim 3(a)). While it is current it
      repeats its entry request to the remembered
      depot at each processing (S-0215 claim 29). With no free place for this vehicle the
      depot answers accept and takes no contact (claim 16's free-place refusal governs the click
      and the retry, not this repeated request; claim 17), so the vehicle stays where it is, in
      *enter*, with no destination, and chooses the standard delay. If a place is free when a processing runs,
      that request takes the contact and gives the vehicle no destination (Q-1158: the depot's
      reply to the repeated request never directs the vehicle); the retry below, which comes later
      in the same turn, then finds the contact already held and goes on to ask permission. Only
      that retry's accepted permission assigns the depot as destination, so the vehicle is first
      moved by the following turn's movement, not in the turn that took the place. If the depot refuses the
      request (for example it is switched off), the vehicle, having no tether, ends contact and
      falls back to rest; its queued entry stays and the retry below still runs.
      Once per eligible vehicle turn
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
    and not an extra behaviour processing. The formerly named immobilized-state check is
    specifically chrono phase-in, not a generic inability to move. If phase-in is still active
    at retry time, the retry is skipped and queued entry retained. Erasure can exclude the
    ordinary living turn before this retry is reached.
    A parasite hold, electromagnetic-pulse disable, deactivation, magnetic lift or an active
    locomotor replacement does not itself skip this retry on an otherwise ordinary living
    turn. Completing a locomotor replacement is dealt with before the retry. Acquiring contact
    does not thereby clear any of those states or guarantee that movement or behaviour can start.
    An attached parasite's same-turn step is later than the retry: contact acquisition can
    therefore precede a hold or lethal damage delivered by that step. Open-topped passenger
    position updates caused by this turn's movement occur with the position change, before
    the retry, not in a new passenger-sync phase after it (S-0507 claim 14).
    An inactive remembered depot clears
    the queued entry. A newly current *enter* granted by the retry above is first eligible for due
    processing on a later vehicle turn; movement already performed this turn is not repeated. An
    *enter* made current by the order of the first bullet of claim 23 is processed in its first turn
    (S-0215 claim 3(a)).

    A place freed before this retry is available immediately, including release by an earlier
    object turn in the same tick. A place freed after the retry waits for a later tick. If
    several waiters can take it, their object-turn order (S-0203 claims 3 and 4), not their original order
    arrival time or distance to the depot, decides. This retry introduces no global phase.

    Any later order clears the queued entry (claim 20). So at a one-place depot the second
    vehicle enters by itself at the first eligible retry after the first leaves. The sequential case of claim 17 differs:
    a click made while the place is held gives no enter order at all. With an enter order on a
    depot that holds no contact, the vehicle asks for entry at once. If it is refused, it takes
    no contact and does not queue.
    - Backing: `measurable` (H18 f; H22 for local turn ordering, same-tick acquisition and
      next-turn processing and the state/parasite/passenger distinctions). The inactive-reference
      case is `unmeasured`: a stock-only setup distinguishing that check has not been established.
    - Confidence: high (event order, retry position, queue/start distinction and automatic
      entry), medium (stock observations of refusal and the full-health case; that *enter* is
      current during the wait and that no walk happens, which H18 f(ii) records tick by tick; the
      depot's accepting reply while it is full, and the request taking a free place before the retry)

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
    uses those same lists. Every supported imported foundation word has a present list,
    including the present empty list for 0 × 0. There is no supported arbitrary-size foundation
    with an absent list merely because its dimensions were not listed: foundation-word parsing
    is S-0108 claim 8's, including its fallback rather than a new arbitrary rectangle. Consequently
    an ordinary imported non-hospital depot does not reach the absent-list perimeter alternative.
    Door-kind and naval preferences do not stop a later row being tried
    when their candidates all fail. No supplied preferred cell is used for repair departure.

    Every candidate must be a legal map cell and give the *free* result from the vehicle's
    cell-entry test (S-0209 claim 5 and S-0211), with no specified approach facing, height level
    or source cell. A high-bridge candidate consequently uses deck-level entry as specified by
    S-0209's entry-layer rules. All non-free results reject it, including temporary obstruction
    and a result that would only make an ordinary moving vehicle wait. Where the table says
    yes, the vehicle's locomotion-specific cell-entry test must also give free. For driving
    locomotion (both wheeled and tracked), hovering locomotion and ship locomotion, that
    additional test always gives free: it adds no terrain, occupancy, motion, power or
    connectivity restriction and cannot reject a cell the ordinary test accepted. S-0209
    claim 5b already states the absence of an additional restriction for these locomotors.
    Its underground exception is outside this spec's scope. This is not
    a route search and imposes no additional reach-region, shortest-path, nearest-water or
    approach-side preference. A candidate in a non-blocking foundation cell can be chosen.
    The search consumes no random draw and does not rotate its starting candidate with the
    tick, facing or house. Ordinary later pathfinding can still fail to reach the chosen cell.

    With no suitable candidate, the result is no cell: there is no widening-radius fallback,
    random destination, automatic scatter or forced contact release. Claim 6 gives the retained
    contact and retry, and the rally-point and guard-spot overrides. A successful ordinary
    release uses the selected cell as its movement destination, not a new appearance position.
    - Backing: `measurable` (H21, including named-test departures using ordinary locomotors
      and foundation words). Door kinds combined with repair, the hospital/list bypass,
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
below 14.4 for the double and just above it (by about 7 × 10⁻⁷) for the single, so in both cases *P*<sub>b</sub> = 14
ticks and *K* = 15 ticks. A rate written as 0.1 is the single 0.100000001…, so 900 times it is
just above 90 (by about 1.3 × 10⁻⁶), giving *P*<sub>b</sub> = 90 but *K* = 91. A rate of 0.125, exact in single precision,
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
| Steps at the settling step | 1 | steps | Claim 6 |
| Depot waiting distance, ordinary locomotors | 100 | leptons, strict upper bound | Claim 6 |
| Depot waiting distance, ship and hovering locomotors | 200 | leptons, strict upper bound | Claim 6 |
| Waiting-to-settling delay | 3 | ticks | Claim 6 |
| Early guard repair-start distance | 64 | leptons, strict upper bound | Claim 17; measured to foundation centre, not docking point |
| Separate confirmation's requested facing | 16384 | facing units | Claim 17; map east, not a movement-arrival requirement |
| Settling hold distance | 200 | leptons, strict upper bound | Claim 21; measured to the docking point |
| Settling recheck within 200 leptons while the vehicle is not settled or, hovering, not down | 1 | ticks | Claims 6 and 21 |
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
| entering with contact, tether present or absent | arrival in the supplied docking-point cell | depot sets its ready mark and queues repair; vehicle queues sleep, neither with immediate start and without creating tether or requesting a body turn; the vehicle becomes settled (claims 17 and 21; S-0215 claim 11 owns subsequent starts) |
| waiting for a contacted vehicle, tethered or not | processing finds it still within its starting distance | settling scheduled 3 ticks later (claim 6) |
| waiting for a contacted vehicle, tethered or not | moving or outside its starting distance | waiting, next processing after *D* ticks; the vehicle stops being settled, if it was (claim 21) |
| settling, with contact | *production* slot animation running | no step; recheck after 1 tick (claim 6) |
| settling scheduled | later processing finds no *production* slot animation, the vehicle still (and, within 200 leptons, settled and, if hovering, down) | settling step (claim 3) |
| waiting, no contact | repair processing | return to *guard* without animation wait (claim 22) |
| working, no contact | repair processing | settling; recheck after 1 tick (claim 22) |
| settling, no contact | repair processing | remain in repair and recheck after 1 tick while *special-3* runs; otherwise return to *guard* (claim 22) |
| settling | *not needed* | sent away |
| settling | *cannot pay* | asks again after *D* ticks |
| settling | *more needed* or *done* | working; first working processing *D* ticks later; the next step falls due *D* + *K*<sub>u</sub> − 1 ticks later |
| working | any processing | next processing 1 tick later (claim 6's spacing table) |
| any phase, with contact | contact ends (vehicle destroyed, or it leaves by claims 20 and 21) | the depot reacts at the processing already scheduled (claims 6 and 22); the vehicle's own effects follow claims 20 to 22 and their quirk settings |
| working | a step falls due, vehicle standing still | repair step; the next falls due *K*<sub>u</sub> ticks later |
| working | a step falls due, vehicle moving | the step is taken on the first tick the vehicle stands still |
| working | step reports *more needed* | stays |
| working | step reports *done* or *not needed* | settling; sent away if a destination is available; otherwise contact held in settling with a 1-tick retry (claims 6 and 25); announcement as claim 6 says |
| working | step reports *cannot pay* | contact retained; settling retry next tick, then unsuccessful retries every *D* ticks; low-funds announcement as claim 6 says |
| approaching vehicle | any order other than a repeated enter on this depot | contact released at once; queued entry cleared (claim 20) |
| docked vehicle | any order other than an enter order on this depot, unless it gains contact elsewhere | contact and tether kept; the phase continues (claims 20 and 21) |
| docked vehicle standing on the depot | enter order on this depot | queued entry, no destination; *enter* current and processed in the vehicle's next turn, and contact lost in that turn (claim 20) |
| settling, vehicle within 200 leptons, not settled or, hovering, not down | processing | no step; standing still, it becomes settled; recheck next tick (claims 6 and 21) |
| settling, vehicle at least 200 leptons away and moving | processing | no step, settling and contact retained; it stops being settled, if it was; recheck after *D* ticks (claim 21) |
| settling, no departure destination available | departure attempted | no departure, settling and contact retained; recheck after *D* ticks (claims 6 and 25) |
| working, vehicle standing still at any distance | a step falls due | repair step (`quirks.depot_remote_repair`) |
| any phase | depot starts selling | run-away notice to every contact; contacts released at the next processing (claim 22) |
| any phase | depot destroyed | contacts released; docked vehicle stays in *sleep* (claim 22) |
| depot holds any contact | enter order for a damaged vehicle | queued entry; no destination (claim 23) |
| queued entry, *enter* queued | vehicle's next ready per-tick start (claim 23) | *enter* current and processed that turn; its request to the remembered depot is accepted without contact while no place is free; no destination, no walk |
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
  original's behaviour). A hovering vehicle whose depot is destroyed after it became settled
  (claim 21), or sold without the vehicle receiving any destination before the depot's removal
  (claim 22), stays settled and, once it is down on the ground,
  never moves again (claim 22). For a depot lost before the settling step this is `unmeasured`
  (claim 22's backing); H18 e loses the depot after it. With `quirks.depot_loss_strands_hover`
  off,
  losing contact with a depot ends its being settled, so the vehicle is switched on again and
  rises to its hover height (claim 21). S-0504 owns the same
  question for naval yards.
- **Hovering vehicle lost at a naval yard** (quirk `quirks.docked_hover_water_loss`, default: the
  original's behaviour). The loss can cost the player a vehicle docked for service; whether an
  ordinary repair visit, with no disabling or enabling, reaches it is open (claim 21). The switch
  is a design choice recorded here with that reason; it was decided at the delegated sign-off,
  2026-10-25. A hovering vehicle docked at a naval yard standing on water, whose switch-off there
  starts a spin (S-0503 claim 10) that ends while it is less than 104 leptons above the water, is
  lost (claim 21, *Over water, at a naval yard*; S-0503 claim 11). With the switch off, a spin end
  that comes while the vehicle has contact with a depot (Terms) or is settled at a depot (Terms)
  makes no water test: the vehicle is not given the bomb mark and does not fall. Once its contact
  has ended and it is not settled at a depot, a later spin end is tested as usual by S-0503 claim
  11. Everywhere else S-0503 claim 11's water test is unchanged: a hovering vehicle disabled away
  from any depot is settled (Terms) but not settled at a depot, and its spin end over water is
  tested in both positions of the switch. Limiting the switch to service at a depot was decided at
  the delegated sign-off, 2026-10-25. The behaviour with the switch off is our engine's rule:
  `unmeasured` (design), confidence medium.
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
| Rules, building section, `Helipad` | pad flag in claim 17's tether exchange, including its empty-place-0 and one-place cases; repair without it does not implicitly tether on arrival | boolean | no |
| Rules, vehicle section, `Locomotor`, `SpeedType` | movement method selects claim 17's movement-report row, driving/ship same-cell assignment, driving-only existing-turn readiness hold (S-0215 claim 11) and claim 25's tests; driving covers wheeled and tracked classes, whose additional test is always free | movement-method identifier (S-0207); class name (S-0211 claim 3) | owned by S-0207 and S-0211; no repair-specific default |
| No additional key | driving/ship destination-point assignment and clearing, including same-cell depot entry (claims 17 and 18; S-0207 claim 25) | runtime integer-lepton point, separate from order destination; supplied docking point uses the displacement rows above | initially absent; no repair-specific override |
| Rules, building section, `Refinery`, `Bunker` | resource-unloading and tank-bunker exclusions (claims 17 and 18); their separate entry rules are not redefined here | booleans | both no |
| Rules, building section, `GDIBarracks`, `NODBarracks`, `YuriBarracks` | first, second and third barracks door kinds for claim 25's optional preferences | booleans | all no |
| Rules, building section, `Naval`; vehicle section, `MovementZone` | naval permission comparison and amphibious exception (claim 16) | boolean; named movement zone, S-0504 claim 7 | no; S-0211 owns the zone default |
| Rules, building section, `Bib` | bib in the foundation-blocking test (claim 18) | boolean, S-0209 claim 5 | no |
| Art, building section, `Foundation` | foundation shape during approach and departure (claims 18 and 25), including dimensions and present exit list; an unsupported word does not define arbitrary dimensions | foundation word, S-0108 claim 8 and S-0404 claim 13's lists | S-0108 claim 8's art-section selection and fallback |
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

All scenarios run in skirmish under the glossary's measurement setup, engine add-ons and mods off,
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
rate of 1/64 game minutes (*D* = 14), the house has 1000 credits. *Observable:* health and credits after each step. *Expected:* +10 health and −3 credits per
step; the first step follows claim 6's waiting gate and comes 3 or 4 ticks after the accepting
waiting processing, as H4 part a gives; the last step brings health to
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

**H4: depot timing (claim 6, F1).** *Part a (timing):* as H3 part a with a vehicle repair rate of 1/8 game minutes and a
*repair* behaviour rate of 1/64 game minutes (*D* = 14). *Observable:* record the tick of the
vehicle's movement arrival in the docking-point cell, the tick it comes to rest, every waiting
processing and whether it accepts the vehicle, the vehicle's height above the ground and its
order-level destination on every tick, and each health-changing step using a read-only
recorder. *Expected:* no step merely on
coming to rest; with the vehicle remaining still, if the accepting processing is the depot's
first waiting processing after the arrival, the settling step 3 ticks after it (the vehicle is
still settled from its arrival: claim 21); if an earlier waiting processing did not accept the
vehicle, no step 3 ticks after the accepting processing (that processing makes the vehicle
settled) and the settling step 1 tick later, 4 ticks after it. Report which case each run
gives (Open questions). In both cases the vehicle's order-level destination is the depot from
its arrival, and is empty from the processing that takes the settling step on (claim 21).
Then the second step 126 ticks after the settling step
(14 + 113 − 1), then one every 113 ticks. With the vehicle repair rate absent (0.016): every 15
ticks after the second. Repeat S-0504 N8's land, ship and hovering comparisons for the strict
starting distances and spatial-distance calculation. The hovering repeat is an attempt for
its switch-off and switch-on sequence, whose tick-level points claim 21 rates `unmeasured`: a
run backs only what its height trace and the recorder's processing ticks show. In it,
*expected:* the vehicle loses its powered lift at its movement arrival and from then sinks as S-0503
claim 8 describes; a waiting processing that does not accept it switches it on again, so it
rises, and the processing that then makes it settled switches it off again (claim 21); no
step while the vehicle is above the ground, and the settling step 3 or more ticks after the
accepting processing; if the vehicle comes down 3 or more ticks after the acceptance, on the
first tick on which it is down or the next one, depending on the order of the depot's and the
vehicle's turns within the tick (S-0203 claim 3, while the depot checks it every tick until
then). *Falsified if:* in the stable land case the settling step comes other than 3 ticks
after an acceptance that was the first waiting processing after the arrival, or other than 4
ticks after one that followed a waiting processing that did not accept the vehicle, the
hovering vehicle never comes down at the depot, the depot is still the vehicle's order-level
destination after the processing that takes the settling step, the hovering vehicle's step
comes while it is
still above the ground, or, when it comes down 3 or more ticks after the acceptance, more
than 1 tick after it is down, a second
step comes other than 126 ticks after the settling step,
or any later interval differs from 113 ticks (15 at the absent rate). Intervals of 112 or
14 would mean the depot uses the truncated building period. The
tick on which the vehicle leaves the depot after the step that brings it to full health is recorded
too.
- *Part b (full health):* a vehicle at full health docks. *Expected:* it is sent away at its settling
  step, 3 or 4 ticks after the accepting waiting processing as in part a (part a's records
  tell which), with no healing,
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
- *Part h (contact loss and processing spacing, Q-1610):* part a's rates (*D* = 14,
  *K*<sub>u</sub> = 113) and the test art of S-0108 B18's setup (not the art B18 part h names
  for itself), with a *special-3* lifetime shorter than
  *D* and a non-looping *special-1* whose lifetime outlasts the whole run from the first paid
  step to the depot's return to *guard*, so that *special-2* never starts in runs 1 to 3 (S-0108
  claim 20). The vehicle is destroyed by scripted damage. A read-only recorder logs per tick the
  depot's repair phase, its processing ticks, its current behaviour, contact, and the start and
  stop ticks of its slot animations.
  *Run 1 (loss in the gap after the settling step):* destroy the vehicle 5 ticks after the settling
  step. *Expected:* no reaction until 14 ticks after the settling step; there the depot enters
  settling, *special-3* and *active-1* start, and it returns to *guard* at its first processing
  after *special-3* ends.
  *Run 2 (loss later in working):* destroy it 1 tick after the second step. *Expected:* the same
  reaction on the tick of the loss or the next tick, not at the next step's due tick 113 ticks
  later. This bound assumes the depot misses no turn in that window; the recorder logs the
  depot's updates, and a run in which it misses one (S-0203 claim 10) is judged against its
  next update that runs.
  *Run 3 (loss between* cannot pay *retries):* part e's setup, with the credits running out
  during working (the short *special-3* starts at the working step that answers *cannot pay*,
  as S-0108's table gives). After the second failed settling retry, destroy the vehicle 3 ticks
  later. *Expected:* the depot's next processing is still 14
  ticks after that retry, and it returns to *guard* there (the short *special-3* has ended), with
  no processing in between.
  *Run 4 (a later vehicle):* after run 2, wait until the recorder shows the depot back in
  *guard*, then order a second damaged vehicle onto the depot. Do this twice, with waits
  before the order chosen so that the new vehicle's settling steps, counted from the lost
  vehicle's last step, differ between the repeats by a number of ticks that is not a multiple
  of 113 (the recorder shows both; a pair of repeats that does not meet this is inconclusive).
  *Expected:* in both repeats its second step falls due 126 ticks after its own settling step,
  then every 113 ticks, whatever the timing of the lost vehicle. A schedule carried over from
  the lost vehicle would move against the new vehicle's settling step between the repeats.
  *Run 5 (loss just after acceptance):* destroy the vehicle 1 tick after the waiting processing
  that accepts it (the recorder shows the phase change to settling). *Expected:* the depot's
  next processing comes 3 ticks after the acceptance; there *special-3* and *active-1* do not
  start (neither *production* nor *special-2* runs, and no *special-1* has started, since no
  step was paid), and the depot returns to *guard* at that processing when no *special-3* runs.
  Run 5 is an attempt: a repeat in which the recorder does not show the loss on the tick after
  the acceptance is inconclusive and backs nothing; repeat it a few times.
  *Run 6 (send-away in settling):* on a newly placed depot of the same type. A named test healer vehicle in *area
  guard* beside the docking point (claims 1 and 2), whose one heal restores a vehicle's full
  strength, heals the damaged vehicle after its enter order has been accepted (claim 23) and
  before the depot's settling step. Repeat until the recorder shows a
  settling step that reports *not needed* and sends the vehicle away, at most ten times. If no
  repeat reaches that step, dock H4 part b's full-health vehicle at a newly placed depot
  instead: its settling step reports *not needed* and sends it away (claim 6). *Expected,* for
  either set-up: the depot's next processing
  is *D* = 14 ticks after that step, in settling with no contact, and it returns to *guard* at
  that processing: on a depot fresh from placement no *special-3* runs, and none starts there
  (S-0108 claim 20).
  *Run 7 (loss in waiting):* use H18 b's waiting repeat to hold the vehicle in waiting: on
  the first tick after its movement arrival at the docking point on which the recorder shows
  it docked, its contact held and the depot waiting, order it 6 cells away (scripted input),
  let it stand still there, and destroy it once the recorder shows a waiting processing after
  the order that did not accept it, with the contact still held. A repeat in which the depot
  leaves waiting before the order, or the order releases the contact (as an order given while
  approaching does, H18 a), is inconclusive and is run again; vary the vehicle's starting cell
  between repeats so that its arrival falls at different points of the depot's *D*-tick
  cadence. *Expected:* at the depot's pending waiting processing, at most 14
  ticks after the loss, it returns to *guard*, with no settling processing and no *special-3* wait.
  *Falsified if:* run 1 reacts before or after the first working processing (14 ticks after the
  settling step), or does not return to *guard* at its first processing after *special-3* ends; run 6's
  next processing comes other than 14 ticks after the step, or the depot does not return to
  *guard* at that processing; run 7 reacts at any processing other than the pending waiting one, or
  enters settling; run 2 reacts later than the
  tick after the loss (or, after a missed turn, later than the depot's next update that runs), or
  only at a step's due tick; run 3 reacts on the tick after the loss or
  at any processing other than the pending retry; in either repeat of run 4 the second step comes other than 126 ticks after its settling step, or any later
  interval differs from 113 ticks; run 5 reacts before the processing 3 ticks after the acceptance, starts *special-3*
  or *active-1* there, or stays in repair after it.

- *Part i (a hovering vehicle switched off again; enabling and disabling while settled; the naval-yard water case, claim 21):* part a's
  hovering repeat, with the random-state recorder logging every scenario-stream draw and a
  read-only recorder logging, per tick, the depot's phase and processing ticks and the vehicle's
  height and current behaviour.
  *Run 1 (repeated switch-off):* as part a's hovering repeat. *Expected:* no draw at the
  movement arrival (its behaviour is then *enter*); at each settling processing from the one
  that makes the vehicle settled until it is down, two draws, R(20, 30) then R(0, 99), when its
  behaviour is then neither *sleep* nor *enter*, and none when it is.
  Run 1 tells a repeated switch-off from a single one only if the vehicle is above the ground at
  a later processing in a behaviour other than *sleep* and *enter*; otherwise it is
  inconclusive for that.
  *Run 2 (enabling in working):* a hovering vehicle of a type that S-0405 claim 13 disables, and
  its house's power cut and then restored (S-0405 claim 20's enabling) while the recorder shows
  the working phase. *Expected:* it rises to its hover height and stays there for the rest of
  the working phase, and its steps keep part a's spacing.
  *Run 3 (enabling in settling):* as run 2, with the power restored while the recorder shows
  settling and the vehicle settled. *Expected:* it rises, takes no step while above the ground,
  and is switched off again at the next settling processing that finds it above the ground,
  with the two draws if its
  behaviour is then neither *sleep* nor *enter*. Run 3 is an attempt: a repeat whose recorder
  does not show settling at the enabling, or in which the depot takes its step before the
  vehicle has left the ground, is inconclusive and backs nothing. Repeat run 3 with a land
  vehicle of a type that S-0405 claim 13 disables, enabled while the recorder shows settling
  before the settling step. *Expected:* the next settling processing that finds it standing
  still takes no step, and the step comes 1 tick later (the enabling ended its being settled:
  claim 21).
  *Run 4 (disabling while settled):* part a's hovering repeat with a hovering vehicle of a
  type that S-0405 claim 13 disables, at an ordinary depot without the pad flag and
  without either unloading flag, so that the vehicle is not tethered (S-0405 claim 20), with
  the owner's power cut while the recorder shows settling, the vehicle settled and down on the
  ground, before the settling step; power stays cut. *Expected:* at the disabling tick, two
  draws, R(20, 30) then R(0, 99), with the vehicle's current behaviour *guard* after it (or
  *wait* if it was in *wait*). Its height is recorded but not judged: with its lift on, S-0503
  claim 8's truncated height update can hold a vehicle short of its hover height field, so no
  fixed height tells lift on from lift off. The next settling processing that
  finds it standing still takes its step, with no processing in between that takes none; the
  contact is kept and later steps follow part a's spacing. Repeat run 4 with a land vehicle of
  a type that S-0405 claim 13 disables: no draw at the disabling, and the step at the next
  settling processing. Repeat both with the power cut while the recorder shows settling but
  before the processing that makes the vehicle settled (the vehicle still within 200 leptons
  and standing still, or a hovering one not yet down): the step comes at the first settling
  processing after the disabling that finds the vehicle standing still (and down), not 1 tick
  after it. Run 4 is an attempt: a repeat whose recorder does not show settling at the
  disabling is inconclusive.
  *Run 5 (disabling with nothing switched on):* run 4's hovering vehicle, with the power cut
  after the settling step, while the vehicle is settled and down on the ground (in settling
  between steps, and in a second repeat in working), at a depot without the pad flag and
  without either unloading flag. To reach settling between steps, the owner starts with
  credits for the settling step and one working step only (claim 6's step cost), so the second
  working step answers *cannot pay* and the depot returns to settling with the vehicle in
  contact (claim 6); the power is cut while the recorder shows that settling phase, and the
  owner is given ample credits 2 *D* ticks after the cut, so that the later steps can be
  judged. *Expected:* no draw at the disabling tick, its height recorded but not judged (as in
  run 4), its current behaviour is *guard* after it (or *wait*), and its later steps keep part
  a's spacing. Repeat on H19 part f's named pad-test map, whose depot's docking point lies in an
  accessible cell outside its foundation, with run 4's hovering vehicle docked there by an
  ordinary entry and the power cut before the settling step while the vehicle is settled and
  down there (S-0405 claim 20 passes over only a vehicle tethered to a depot standing in its
  own cell): no draw. Run 5 is an attempt: a repeat
  whose recorder does not show the vehicle down and settled at the disabling is inconclusive.
  *Run 6 (a naval yard, claim 21's water case):* a tiny test map with water cells, an own
  naval yard of a type with the repair flag and without the pad flag or either unloading flag
  standing on them (so the ordinary entry establishes no tether, S-0405 claim 13), an own land depot and the
  control centre that S-0405 claim 13 needs; a named test copy of a hovering vehicle type
  whose movement zone is plain amphibious (S-0504 claim 7) and that S-0405 claim 13 disables,
  damaged to half strength. The recorder also logs, per tick, the land type of the vehicle's
  cell, its height above the surface, its bomb mark and its removal. (i) Dock it at the naval
  yard by an ordinary enter order and cut the owner's power while the recorder shows it at
  its docking point, in waiting or settling before its settling step, and still above the
  water. *Expected:* a water cell; at the disabling, two draws, R(20, 30) then R(0, 99); on
  the tick its spin ends (at the latest when it comes down), when the recorder shows it less
  than 104 leptons above the water then, it is given the bomb mark, and when its fall
  ends it is lost (with S-0312 claim 11's wake and splash if it dies within 10 leptons above the
  water), and the yard reacts as to a vehicle lost in
  that phase (claim 6's rows with no contact). (ii) As (i) with ample credits, but cut the
  power only in working, with the vehicle down on the water (run 5's working repeat).
  *Expected:* no draw at the cut, no spin, no loss, and its later steps keep part a's spacing.
  Before the cut, the recorder shows whether the ordinary visit's switch-offs took draws: a
  loss before the cut that follows logged draws agrees with claim 21's water case and makes
  (ii) inconclusive for the cut. (iii) As
  (i) at the land depot. *Expected:* two draws, no loss, and the step at the first settling
  processing that finds it standing still and down. Run 6 uses a test type that no stock type
  matches and may be reported unreached; a repeat whose recorder does not show the phase and
  height that its case names at the cut is inconclusive.
  Run 7 (Q-1876): the same test type with the powered-unit flag, ordered over an open water cell
  at least 10 cells from any depot or naval yard, its house given one control centre that is then
  destroyed while the recorder shows the vehicle standing still above the water and never docked.
  *Expected:* two draws at the disabling, and on the tick its spin ends less than 104 leptons above
  the water the bomb mark and the loss, as S-0503 claim 11 gives. A spin that ends at least 104
  leptons up leaves that repeat inconclusive: repeat the run with a fresh vehicle, destroying the
  control centre at a different tick of the vehicle's hover, until one repeat's spin ends less
  than 104 leptons up, and stop after 10 repeats, reporting run 7 unreached if none does. The
  same run repeated in our engine, once with
  `quirks.docked_hover_water_loss` in each position, checks claim 21's switch-off rule there.
  *Falsified if:* in run 1, draws come only at the first switch-off while the vehicle is above
  the ground in a behaviour other than *sleep* and *enter* at later processings, or come at a
  processing where its behaviour is *sleep* or *enter*; in run 2, it comes down during working
  or a step is deferred while it stands still; in run 3, it keeps its lift after the next
  settling processing, or a step comes while it is above the ground; in the land repeat, the
  step comes at the first settling processing after the enabling; in run 4, the hovering
  vehicle takes no draw at the disabling, or any
  repeat's step comes 1 tick after a settling processing that took none while the vehicle stood
  still (and was down), or the contact is released; in run 5, a draw at the disabling; in run 6
  (i), the vehicle
  survives the end of its spin less than 104 leptons above the yard's water cell, or takes no
  draws (a spin that ends at least 104 leptons up leaves (i) inconclusive for the loss); in (ii), a loss
  with no draw logged since the vehicle came down; in (iii), the vehicle is lost; in run 7, the
  vehicle survives the end of its spin less than 104 leptons above the water.

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
hospital; infantry repair step 25, infantry repair rate 1/8 game minutes, *repair* behaviour rate 1/64 game minutes
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

**H7 b: smoke after self-healing (claims 9 and 10).** Use S-0616 claim 1's P23 c
setup, procedure, three runs per variant and smoke-emission observations. Individual healing
must mark existing damage smoke on healing above yellow; house-wide healing must do so for
non-organic vehicles, but not infantry or organic vehicles. Existing particles must survive
the mark and end normally. Continued emission after a required mark, immediate removal of
existing particles, or a threshold mark in either house-wide control falsifies the rule.
The below-surface condition is not exercised by this scenario.

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

**H8 f: warp exclusion (claims 9 and 10).** Use S-0513 claim 48's SW23 setup and
read-only turn recorder with damaged eligible infantry, vehicles and landed aircraft.
Test individual healing, and for infantry and vehicles house-wide healing, with due
healing ticks during turns that remain in the leaving stage after the warp update.
Expected: no gain from either source on those turns; a gain falsifies that exclusion.
Record pending, departure and arrival turns separately rather than extending this
prediction to unresolved continuation boundaries. This is SW23's proposed test, not
a completed measurement.
For our engine only, also apply S-0513 claim 51's SW23 part f contract test to both
healing sources: on due healing ticks, expect no gain from the first pending update
through departure, retries, placement and arrival effects. During arrival hold and
its expiry turn, expect the ordinary due gain only when the object remains otherwise
eligible after the hold-end test. A gain in an excluded turn, or suppression solely
by this contract in an otherwise eligible hold turn, falsifies the design contract.
These engine-only expectations do not settle the original's unresolved boundaries.

**H8 g: service notifications (claims 7 and 17).** Use S-0524 claim 14's C16
hospital and vehicle-repair fixtures, repeating destination entered-by tags and entrant
own-tag destroyed-by-anything tags, with its raise recorder and visible counters.
Expected: hospital infantry raises one destination entry notification before completion,
vehicle docking none, and neither entry destroys the entrant. Release adds no entry
raise; fresh entry repeats the respective count. Any contrary count, causing object
or timing falsifies the corresponding rule. C16 supplies the setup and repetitions.

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
isolated bystander computer player. Engine add-ons and mods off, game speed index 2, 800 × 600.
Repeat each part three times. A read-only recorder logs input action, issued orders, positions,
body facing, contact places, tether, behaviour, building processing and health every tick.
- *a, ordinary entry:* damage one vehicle and click the depot with it selected, from each
  accessible side. Expected: enter order, contact before arrival, destination directed to the
  depot's supplied point, deferred depot repair and vehicle sleep queued at movement
  arrival, then claim 6's waiting and settling timing; no compulsory map-east arrival facing.
  Where the guard-start conditions of claim 17 occur first, record that earlier repair scheduling instead. Distinguish the unarmed
  depot's initial idle-body selection from its later guard waiting stage: the early transition
  is allowed only in the latter. Repeat with an otherwise identical armed depot supplied by
  named test-map overrides, with no enemy target in range: its guard processing must not take
  this early transition. Either excluded transition falsifies claim 17's qualification.
  Repair on the initial click,
  a neighbouring-water or exit-cell destination substituted for the docking point, a point
  changing with approach side, or a different recorded transition falsifies the relevant rule.
  For Q-1162, add a named test-map pad-depot fixture with two contact places and a docking
  displacement permitting a diagonal approach and tether before arrival. Approach facing away
  from 16384 facing units. Repeat with driving, ship and hovering vehicles in suitable land or
  water fixtures, and record both current and desired body facing, locomotor movement report,
  destination, tether, processing ticks and current/queued behaviours before and after arrival
  and the late start. Include arrivals between enter processings, varying the approach length
  if needed. Expected: arrival queues sleep but introduces no new map-east turn, regardless of
  tether. If otherwise ready, sleep is current at that turn's late start without another enter
  processing. A newly requested map-east turn caused solely by movement arrival falsifies the
  claim. A driving vehicle already turning is held only by S-0215 claim 11's readiness rule.
  a ship or hovering vehicle with rotation alone is not held by that driving-only condition.
  Compare the driving control with S-0215 B47. Report a fixture inconclusive if ordinary input
  cannot produce the required tethered approach or existing-turn state; do not create either
  by writing into the running game. No separate completed-entry confirmation is assumed from
  an ordinary depot, with or without the pad flag.
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
count and recorder, with credits, contact places, tether, queued entry and health logged every tick, plus each logged vehicle's current behaviour and destination. Use one stock land depot and stock tracked and hovering vehicles,
damaged to half strength unless a part says otherwise.
- *a, orders:* give a move, stop or attack order (i) while the vehicle approaches, (ii) after it
  docks. Repeat the enter order on the same depot while approaching. Expected: (i) contact
  released in the order's tick, and a second vehicle can then take the place; (ii) contact and
  tether kept; the repeated enter changes nothing. Release while docked, or a reset approach on
  the repeated order, falsifies claim 20. (iii) Give a docked damaged vehicle an enter order on
  the same depot through scripted input. Expected: contact ends one tick later and repair stops.
  Record the vehicle's current and queued behaviour, destination and position, and the
  depot's current and queued behaviour, every tick from the order on (Q-1158). Expected: *enter* queued by the order and
  current in the tick the contact ends; no destination; the depot queues no new *repair* and
  the vehicle no new *sleep* between the order and the end of contact. *Falsified if:* either is
  queued again, a destination appears, or the contact outlives the first tick with *enter*
  current. Repeat at full health: contact ends in the order's tick.
- *b, moving away:* order a docked vehicle 6 cells away during working and let it stand still.
  Expected: health keeps rising by steps, credits fall, and at full health the vehicle is sent to
  the rally point or exit cell, not left at the ordered cell. Repeat during waiting: no step while
  away, place still held. Repeat during settling within 200 leptons (the order, received on the depot,
  ends its being settled: claim 22), recording the depot's processing ticks: the depot
  rechecks each tick, with no step, while the vehicle moves within 200 leptons. *Falsified if:*
  a step comes while it moves within 200 leptons, or the rechecks there come *D* ticks apart (the
  order would not have ended being settled). This repeat tests the 1-tick recheck. It backs the
  order's ending of being settled only that far: a rule that rechecks every tick while the
  vehicle moves, settled or not, would give the same ticks (claim 22 rates that end medium). Once it stops 6 cells away, claim 21's row for a
  vehicle standing still at 200 leptons or more applies. Any distance-based stop in working
  falsifies claim 21.
  Repeat the settling
  case with a hovering vehicle, given its order after it has come down on the ground and
  before its settling step, logging its height above the ground every tick. Expected: from
  the tick after the order's tick (S-0203 claim 7) it rises to its hover height and moves (claim 21). *Falsified if:* it
  stays on the ground or does not move.
  This hovering repeat is an attempt: whether a window exists between the tick on which the vehicle
  is down and its settling step depends on the order of the depot's and the vehicle's turns within
  the tick (S-0203 claim 3), and the step can come on the first tick on which it is down. A run
  whose recorder shows no such window, or shows the order given outside it, is inconclusive and
  backs nothing; repeat it a few times. If no repeat has the window, the switch-on is reported
  unmeasured by this part.
- *c, sale:* sell the depot (i) with a docked vehicle, (ii) with an approaching one. Expected:
  both switch to *guard*, the depot destination is cleared, contacts end one depot processing
  later; record any scatter movement. *Falsified if:* the depot stays in a working state, a
  contact outlives the next depot processing, or a destination on the depot remains.
- *d, destruction:* destroy the depot (scripted damage outside the vehicle) in the same two
  cases. Expected: contacts released; the approaching vehicle falls back to *guard*; the docked
  vehicle stays in *sleep* and does not fire at an enemy test unit placed in range. Then
  destroy the contacted vehicle instead, separately during waiting (held 6 cells away as in
  b's waiting repeat and H4 part h run 7), settling and working.
  Log depot behaviour and slot-animation starts and ends every tick. Expected: waiting returns
  to *guard* at its next processing without waiting for *special-3*; working enters settling
  with a 1-tick retry; after its first no-contact processing, no-contact settling checks every
  tick and returns to *guard* only once
  *special-3* has stopped running. Repeat the settling hold with the depot off-screen and at
  different drawing-frame cadences: the simulation ticks of the animation end and guard return
  do not change (S-0108 claim 20, B18 g). Then order another vehicle to enter.
  *Falsified if:* a contact survives, the approaching vehicle keeps its depot order, the docked
  vehicle fires, no-contact settling returns to *guard* while *special-3* still runs, the wait
  changes with drawing cadence, or the depot refuses the next otherwise eligible vehicle.
- *e, locomotors:* repeat d with a hovering vehicle after its settling step, then give move
  orders. Expected: it never moves. Repeat c with a hovering vehicle after its settling step,
  recording whether the run-away notice sends it a scatter request and whether that request
  gives it a destination. *Expected:* when it gets a destination there, it rises and moves
  (claim 22: the depot still occupies its cell). When it gets none, it stays down; then,
  in one run, a move order given during the build-down raises it and moves it, and in
  another, a move order given only after the depot's removal does not. *Falsified if:* it
  stays down after receiving a destination before the removal, or moves under an order given
  after the removal when it received no destination before it. Repeat with a tracked vehicle and record whether
  it moves (claim 22's low-confidence part). Then order that tracked vehicle, after its depot
  was destroyed while it was settled, to enter a second own depot, recording its arrival in
  the docking-point cell, every waiting processing of that depot and the settling step.
  *Expected:* claim 6's timing as for any arriving vehicle: the step 3 ticks after the
  accepting processing when that is the depot's first waiting processing after the arrival, 4
  ticks after it when an earlier one did not accept the vehicle. *Falsified if:* the step
  comes 3 ticks after an acceptance that followed a waiting processing that did not accept
  the vehicle, or 4 ticks after one that was the first after the arrival.
- *f, power and simultaneous orders:* (i) during working, drop the owner's power below demand,
  then switch the depot off. Expected: steps, costs and timing unchanged. *Falsified if:* any
  step, cost or its tick differs from the same run without the power drop or switch-off. (ii) Select two
  damaged vehicles and click the depot once, so both orders are issued before either vehicle
  takes contact. The first applied enter order takes contact before the second is applied
  (S-0213 claim 3).
  Expected: one approaches; the other stays where it was ordered, with *enter* current from its
  first turn after the order takes effect and no destination (claim 23), with a queued entry, and enters by itself at most
  one tick after the first leaves. Repeat with the second at full health: it stays put and never
  enters. Repeat with the depot switched off before the first leaves: the queued vehicle gives
  up when the place frees. Record the second vehicle's cell, destination and current and queued behaviour every tick from the order on (answers Q-1156). Record the tick in which the first vehicle's place frees. Where the place frees before the second vehicle's retry in that tick, expected: the second vehicle has no destination before that retry, has the depot as destination after it, does not move in that tick, and its position first changes in the next tick (Q-1158). Falsified by a position change in that tick, or a destination present before that retry.
  *Falsified if:* the second walks to another cell or has any destination before the first leaves, has a current behaviour other than *enter* while waiting, enters before the first leaves, or, with
  the depot switched off, does not give up when the place frees.
- *g, no automatic return:* leave a damaged vehicle of the human house idle near the depot for
  9000 ticks with no orders. Expected: it never enters. Any automatic entry falsifies claim 24.
- *h, two places:* a named test depot type with a docking-place count of 2 and two invented
  displacements. Order the second vehicle in the same tick as the first. Expected: it takes place
  1, docks at its own point, and receives no step while the first is serviced. Stop recording
  before the first leaves, because the original's outcome there is undefined.
  *Falsified if:* the second takes place 2 instead of 1, or receives a step while the first is
  serviced.

### H19 — Tether, repeated entry and off-centre stopping (Q-0680)

- **Setup:** tiny flat land-and-water test map, stock data only, engine add-ons and mods off, speed
  index 2, 800 × 600. Use an own stock land repair depot and driving vehicle, then a stock
  naval yard and compatible ship. Damage the vehicle before entry. Record the imported pad
  flag and supplied docking displacement for each stock type without changing either.
- **Procedure:** order entry from several approach sides. Record initial contact, entry
  processing ticks, destination, tether onset, occupied cells, world position, moving/stopped
  state, first repair processing and first health/credit change. A read-only event trace would
  be needed; propose its addition to the existing measurement harness, not a modified
  executable.
  (a) Repeat with a second move order executed on each side of the first tether tick. Also
  follow entry processings after the vehicle first occupies a foundation cell.
  (b) Use a stock docking displacement off the centre of its cell where one exists. Compare
  the final driving/ship position with both that cell's centre and the supplied point.
  (c) At an ordinary depot without the pad flag, repeat move orders during waiting, settling
  and working, separately. Record contact, tether, health and credits before and after each.
  Expect no arrival tether, continued paid repair without an order, but immediate release
  on a new move order in every phase; no later remote repair under that lost contact.
  (d) Where stock flags cannot separate these cases, propose a named test map with two depot
  test types differing only in the pad flag, with neither unloading flag. Give them two contact
  places and arrange requester and place-0 vehicle at distinguishable positions. Record
  repeated-request ticks, destinations, locomotor movement, both world positions and tether.
  Test a requester with an order-level destination but no locomotor movement report, then
  one with that destination and a movement report, while place 0's integer spatial distance
  is 128 and then 129 leptons. Only the latter distance enables the moving-destination
  case; the requester without that report needs neither distance.
  Also leave place 0 empty after the requester has acquired place 1, without giving the
  requester a new order. Expect the moving-destination case not to tether; repeat without a
  moving destination and expect tether. Record the order-level destination separately from
  locomotor destination and heading point. Trace each point assignment and clearing event,
  using part f's same-cell and blocked-approach controls and S-0207 claim 25's lifetime,
  rather than deriving presence from cell inequality. For driving and ship requesters compare a turn
  in place, a blocked destination, stored steps with neither point present, and a heading
  point at the current horizontal position. Expect claim 17's table, not a general
  in-progress-track test. For hovering compare throttle 0 and nonzero with a retained point:
  both report movement. Repeat each isolated case three times, with a bystander computer
  player kept outside the observed depot area; void any run it interferes with.
  (e) Use a one-place variant of that named pad test map, centred docking point, flat ground
  and a straight approach. Record every repeated request, centre distance, both destination
  kinds, their assignment/clearing events, heading point, tether and current/queued behaviours.
  Include the same-cell entry control of part f before comparing the approach run.
  Expect no tether when the
  conditions of claim 17's one-place case hold, but normal paid repair. A pause that allows
  an enter processing with no moving destination in the centre cell is a separate control,
  expected to tether, not evidence that the uninterrupted case tethers. Check that the
  separate locomotor destination has actually cleared: a pause in physical motion alone
  does not create this control. In both pad and
  no-pad runs record the depot ready mark immediately around movement arrival, and its
  first later behaviour start and processing. Expect the arrival to set the mark and queue
  repair without synchronously processing it. Repeat three valid runs of each variant.
  (f, Q-0794) For both driving and ship locomotion, issue entry while the vehicle already
  occupies the supplied docking-point cell. First attempt the stock depot/yard; distinguish
  refused permission from accepted destination assignment. Where stock permission cannot
  admit the setup, use the named pad-test map with a displaced docking point in an accessible
  cell outside the depot foundation. Issue entry from rest in that cell, with no movement
  hold and no existing contact; record order execution, contact, supplied point, order-level
  destination, locomotor destination, heading point and every later stop or track completion
  within the turn, not just at tick end. Expected at accepted assignment: the locomotor
  point is present at the supplied point despite cell equality, before any successful search,
  and its movement report is true while that building destination remains. Across later
  no-track updates, mere cell equality clears neither destination: the depot is a building
  destination, not a cell destination. Record behaviour as well: S-0207 claim 26's
  exact-point guard exception remains applicable if it becomes at rest in guard with
  all three coordinates equal. Use that claim's D30 cell-goal and guard controls to
  distinguish those cleanups from entry exchanges or servicing changes. In a separate
  run order entry from outside that cell and block the route
  short of it with stock obstacles; a wait or failed search retaining the goal retains the
  point and movement report. Record give-up separately rather than treating every failure
  as give-up. Compare ordinary final-track arrival, expecting S-0207 claim 25's clearing
  before the arrival report, and a stop order, expecting immediate point clearing even if
  an existing track continues. S-0207 D29 supplies the complementary general movement
  controls. Repeat each reachable case three times with the same settings as above.
  Refused same-cell input is a refusal control, not evidence that accepted assignment omits
  the point. If the named map cannot reach accepted same-cell entry, report that case
  untested, not an inferred absence. Omission on accepted assignment, clearing solely at
  cell equality or on a retained-goal search failure, or different stop/arrival clearing
  falsifies the respective rule, excluding S-0207 claim 26's applicable guard cleanup
  and separately recorded entry or servicing effects.
  These tests need a read-only state trace proposed for the harness; do not set marks,
  points or tethers by writing into the running game. If a requested state cannot be
  reached by ordinary input and the named map, report that sub-test inconclusive.
- **Expected:** no tether on initial contact; ordinary pad tether follows claim 17's centre-cell
  condition, not first foundation occupancy or first repair. A new order before tether frees
  contact, and one after tether retains it subject to claim 20. Repeated entry does not run
  the already-standing permission refusal. On level tracks the final horizontal position is
  the docking cell's centre. Paid repair still waits on the supplied-point distance gate.
- **Falsification:** tether at reservation, unconditional foundation-cell refusal during
  continuing entry, or exact off-centre snapping. If no stock type separates a tested condition
  (including pad combinations), report it untested rather than editing its rules except for
  the explicitly named test map of part d. Tether created solely by no-pad repair arrival,
  retention after an untethered working-phase move order, using the requester's destination
  distance, substituting another vehicle for empty place 0, failing to set the ready mark
  on movement arrival, or a movement report disagreeing with claim 17's table would falsify
  the added cases. Tether on the uninterrupted one-place run with every recorded request
  meeting claim 17's stated failing conditions would also falsify it.

### H20 — Moving out of settling range (Q-0681)

- **Setup:** H19's stock vehicles and map, plus a second damaged vehicle with queued entry.
  Use speed index 2 and the imported repair processing delay *D* without changing it.
- **Procedure:** after the depot accepts the waiting-distance gate but before its settling
  processing, give a move order away while tethered. Vary the order tick and approach position
  until the due settling check finds a moving vehicle at truncated distance at least 200
  leptons from the supplied docking point. Record distance, movement, processing tick,
  health, credits, tether/contact and the second vehicle's ability to acquire the place.
- **Expected:** no step or credit debit, retained settling and contact, and the next settling processing *D* ticks later (that
  the vehicle has stopped being settled afterwards is claim 21's rule
  and is not observed here). Contrast a still-moving check below 200: next
  processing 1 tick later. Stationary distant settling can take the repair step.
- **Falsification:** distant moving settling takes a step, frees contact, advances to working,
  or uses the near-point 1-tick retry when *D* differs from 1. If stock motion cannot reach the
  distant-moving setup during the 3-tick interval, this part remains unmeasured.

### H21 — Repair departure candidates and a blocked exit (Q-0682)

- **Setup:** H19's stock depot/yard and vehicle types, human-controlled owner, no guard-spot
  return, no rally point. Mark foundation cells, the appropriate exit list and claim 25's naval
  candidates on a tiny map. Stock blockers can be stationary own units or impassable terrain.
  engine add-ons and mods off, speed index 2, 800 × 600.
- **Procedure:** repair a damaged vehicle to completion. Record the assigned departure cell,
  health/credit ticks, movement and contact-release tick. Re-run with successive preferred
  candidates blocked, then all candidates blocked; remove one blocker after a known tick.
  At the yard, separate the three naval preferences from the first exit-list candidate.
  Repeat selected layouts with different seeds and completion ticks. Finally set an ordinary
  rally point outside the candidate set and repeat with every exit candidate blocked.
  Also record the repair-complete event and announcement at the working step that fails to
  depart, and at the following silent settling retries. Keep other repair events outside the
  suppression distance.
  For the additional-test distinction, propose a named test map with otherwise identical depot
  types using a supported foundation and the first door kind, and test vehicles using driving
  (wheeled and tracked), hovering and ship locomotion on compatible land/water. Present legal
  candidates with ordinary entry result free, including occupied non-blocking foundation cells
  and cells separated by impassable terrain from the vehicle's current position. Record both
  the candidate entry outcome and chosen departure; no second restriction rejects a free cell.
  For list presence, use named test art entries with a supported foundation, the zero-size word
  and an unsupported word. Record the imported foundation and departure candidates. Supported
  words use their present lists, zero-size has a present empty list and no perimeter fallback,
  and the unsupported word follows the ordinary foundation parser's fallback, not its written
  dimensions. These are proposed named test files, removed afterwards.
- **Expected:** first suitable candidate in the stated order; no seed/tick rotation, no search
  farther out, and no automatic scatter. All blocked with no rally leaves contact held and no
  departure order. Failure from working gives a 1-tick settling retry, and further unsuccessful
  settling retries are *D* ticks apart. Opening a candidate permits release
  at a later due retry. The failed working departure still attempts the repair-complete event
  and announcement; subsequent settling *not needed* retries do not. A rally point overrides
  even a failed exit search, but not the original
  non-human guard-spot exception of claim 6.
- **Falsification:** different preference order, accepting a non-free entry result, changing
  the pick solely with seed/tick, widening the search, releasing contact with no destination,
  or remaining held despite a rally-point override. Settling retries spaced other than the
  expected 1 tick after the failing working step and then *D* ticks apart (logged by H4 part
  h's read-only recorder) falsify the failed-departure row of claim 6's table. A free candidate refused solely by the
  additional test for any of the four named movement cases, or an unsupported foundation
  word creating its own perimeter-sized building, also falsifies the relevant added claim.
  A bridge part requires a legal stock
  placement that actually presents a bridge candidate; otherwise leave that part unmeasured.
A departure-destination and contact-event trace is a proposed harness addition, not a change made here.

### H22 — Late vehicle-turn queued retry (Q-0683)

- **Setup:** tiny stock map, one-place depot or yard, one contacted vehicle and two damaged
  vehicles with queued entry. Engine add-ons and mods off, speed index 2, 800 × 600. Arrange separate
  runs with the releasing vehicle/depot before and after each waiter in the actual object-turn
  order; record that order rather than assuming creation order establishes it.
- **Procedure:** release the first contact by completed repair, destruction or an untethered
  move order at a known tick. Record each waiter's movement, due behaviour processing,
  self-healing if applicable, retry/contact tick, queued/current behaviour and destination.
  Reverse the waiters' original order-arrival order without reversing their turn order.
  In separate runs apply a parasite hold, deactivation or magnetic lift to a waiting vehicle,
  and record whether it gets an ordinary living turn, whether a retry occurs, and whether contact
  is acquired despite inability to move. A named test map may supply a pulse-disabled test
  vehicle for the electromagnetic-pulse case, with no executable change. Separately teleport
  a waiter and record phase-in and retry ticks, including completion of the replacement
  locomotor. Record hold timers and health immediately around a due attached-parasite step.
  Use an open-topped transport as another waiter and record passenger positions at each
  transport position change and retry. These named-test extensions need an ordered read-only
  event trace; a missing ordinary turn is not evidence of a retry-specific state gate.
- **Expected:** each retry is after that vehicle's movement, due behaviour and self-healing,
  not a pre-movement or orders-phase batch. The first eligible retry after release wins,
  including in the release tick if its turn is later. A waiter that already retried waits for
  another tick. Successful retry may make *enter* current and assign its destination at once,
  but adds neither a second movement nor same-turn processing of the new *enter* behaviour.
  Still-active chrono phase-in skips retry and keeps the queued entry. A hold, pulse disable,
  deactivation, lift or replacement alone does not skip it on a turn that reaches retry.
  Movement-caused passenger position propagation precedes retry; retry precedes the attached
  parasite's step, so a same-turn newly applied hold does not retroactively cancel acquisition.
- **Falsification:** FIFO allocation independent of actual turn order, a mandatory one-tick
  reservation delay, a pre-movement retry, or a second same-turn movement/enter processing.
  A generic movement-disable gate excluding one of the stated non-phase-in cases on an
  otherwise ordinary turn, retry before movement-caused passenger propagation, or retry
  after the same-turn parasite step falsifies the respective added claim.
The measurement scenario requires a read-only record of the ordered events; end-of-tick positions in a video frame alone are insufficient to establish local ordering.

### H23 — Production-slot hold during settling (claim 6)

- **Setup:** a tiny test map, one human house and a remote bystander computer player, fixed
  seed, speed index 2, stock data with no add-ons or mods apart from the named test type below.
  Use an explicitly non-stock combined-role test type: a repair depot that also has a role starting its *production* slot, as S-0108 claim 20
  describes, with a finite slot duration and an otherwise eligible damaged contacted vehicle that does not
  hover, standing within 200 leptons of its docking point. The production role is the
  refinery-docking role: it starts the slot when one of the owner's harvesters, docked at the
  building to unload, has emptied its load, so "make the production role start the slot" below
  means timing such an unload (S-0108 B18 part j).
  Record depot phase, slot lifetime, health and credits each tick with a read-only recorder.
  No recorder writes to simulation state are permitted. Void a run if the bystander interferes.
- **Procedure:** the run does not rely on the harvester and the vehicle being docked at the
  building at the same time. *Input sequence (both parts):* place the damaged vehicle on a cell
  next to the test building and order one of the owner's harvesters, carrying a load, to unload
  there. On the tick the recorder shows that the harvester has emptied its load and the slot has
  started, order the harvester away and give the vehicle its enter order. Choose the slot
  duration from a pilot run of the same set-up, so that the slot still runs at least 5 ticks
  after the depot's first settling processing for that vehicle. Keep the vehicle still once it
  has docked. *Part a:* the owner can pay throughout. *Part b:* as part a, with a slot duration
  that also outlasts the settling processing 3 ticks after the acceptance. The hold acts only in
  settling (claim 6's table), so the acceptance is not delayed. Record every waiting processing
  and the vehicle's movement arrival: a part b run in which no waiting processing between the
  arrival and the acceptance failed to accept the vehicle is inconclusive, because the vehicle
  is then still settled from its arrival (claim 21), and so is a run in which the recorder does
  not show the slot running at the first settling processing. Repeat each part three times.
- **Observable:** ticks of settling checks, slot completion and the first paid repair step.
- **Expected:** no health gain or repair payment while the slot runs; settling rechecks each
  tick. Part a: once the slot is absent, the first check takes the normal settling step, unless
  a waiting processing between the arrival and the acceptance failed to accept the vehicle; a
  run with such a failure is judged as part b. Part b: once the slot is absent, the first check
  takes no step and the settling step comes 1 tick later, since the held processings did not
  make the vehicle settled (claim 21).
- **Falsified if:** a repair step occurs while the slot runs, or the hold uses the ordinary
  processing delay instead of 1 tick; in part b, the step comes at the first check after the
  slot ends (the hold made the vehicle settled). If no slot duration that the pilot run tries
  lets the slot run into the settling phase, report the fixture as unreached, not confirmed.

## Open questions

- (Claim 21, the settling question within 200 leptons.) A settled vehicle (down on the ground, if
  it hovers) within 200 leptons of its docking point whose locomotor still reports movement: no
  input is known that reaches this state, and the state table has no row for it. Claim 17's
  same-cell enter case keeps the locomotor's destination, but such a vehicle is not accepted in
  waiting and is never made settled (claim 21), so it does not reach the state. Expected,
  `unmeasured`: the processing removes its order-level destination (claim 21), so by the
  moving-destination test of Terms it counts as not moving and takes its settling step, and it
  stays settled. A scenario that reaches the state would test this.
- (Claim 21, the same-cell enter order.) Whether anything clears the locomotor destination point
  of a vehicle whose accepted enter order found it already in the docking-point cell while it
  stands there, so that the depot accepts it, and when. Until then it stays in waiting with no
  step. H19 part f's same-cell control, run long enough, would show it.
- (Claim 22, a stranded hovering vehicle.) Whether a superweapon teleport (S-0513) places a
  vehicle stranded by `quirks.depot_loss_strands_hover` on the map anew, which would switch it
  on and let it move again, or moves it without doing so.
- (Q-1610 question 3, the part left open.) A vehicle that arrives at the docking point of a
  depot still in its no-contact settling hold (claim 22, while *special-3* runs), where the
  arrival's queue request leaves the queue unchanged, or that
  takes contact during that hold and is processed by the depot before it arrives: whether the
  settling checks of claims 6 and 21 then apply to it directly, skipping waiting's starting
  distance and 3-tick delay. Likewise, a vehicle that takes the freed contact place before
  the depot's pending processing after a loss in waiting, working or settling (claim 22:
  the loss neither brings that processing forward nor puts it back; claim 23 lets a
  queued vehicle take the place at its first eligible retry after the first leaves). After a loss in
  working or settling: whether that processing's step schedule restarts for the new vehicle or
  carries over from the lost one. After a loss in waiting: whether the pending waiting
  processing checks the new vehicle's stillness and starting distance as for any vehicle, or
  finds no contact and returns the depot to *guard*. No reproducible setup is established.
- (Claims 6 and 22.) Q-1610 questions 1 and 2 for a depot type that is also a vehicle bunker:
  when such a depot's processings come after a docked or approaching vehicle's contact ends.
  S-0606 claim 20 gives only that its processing delay is the depot's; this spec's claim 6
  table does not cover that type, and no trace of it is recorded.
- (Claims 6 and 21.) Which first-step timing an ordinary docking gives. A vehicle is settled
  from its movement arrival, so its step comes 3 ticks after the acceptance if the depot's
  first waiting processing after the arrival accepts it, and 4 ticks after it if an earlier
  waiting processing found it moving or outside the starting distance and so ended its being
  settled. That turns on when the depot's *repair* becomes current after the arrival (S-0215
  claims 7 and 11) and on whether the vehicle, at rest at its docking point with the depot
  still its destination, gives the movement report of claim 17 at that processing; neither is
  established here. H4 part a records which case each run gives.
- Which additional vehicle-state condition makes the original's non-human-owner branch return
  a depot from a paid settling step to waiting without starting work (claim 6), and what
  subsequent vehicle orders result? Only neutral and special houses retain this branch in our
  engine; it is not an AI-player repair exception. No reproducible fixture is established.

- Whether a settled vehicle with a driving or other non-hovering land locomotor can still
  move (claim 22, low confidence). H18 e records it.
- Whether an order can move a settled hovering vehicle while it is still sinking, between its
  becoming settled and its touching down (claim 22), whether or not its depot is still on the map.
  S-0503 counts it as powered then, so a destination could start a path search (S-0503 claim 5).
  The order does not end its being settled, whether or not a depot occupies its cell (Terms,
  claim 22), and its lift stays off. A hovering vehicle settled by its arrival sinks from
  that arrival on, since it is switched off when it becomes settled (Terms), so the window opens at
  the arrival in that case.
- Whether a scatter request after the sale's run-away notice moves a vehicle that is already
  standing on or beside the depot (claim 22; S-0215 claim 19 owns acceptance).
- The former allied-depot blocking question concerns route-point action selection, owned by
  S-0215 claim 13, not a separate permission to cross a blocking foundation. Foundation entry
  remains S-0209 claim 5's.
- The exact phase of the second depot step and the first hospital step (±1 tick); scenarios H4
  and H6 record them. Claim 21 distinguishes the near 1-tick settling retry of a vehicle that is
  not settled (moving or not) from the *D*-tick retry of a moving vehicle
  200 leptons or more away. Claim 25 and H21 cover a failed departure search.
- Whether a unit inside a hospital can reach full health by another rule, so that *not needed* can
  arise there (claim 7, H6 part d).
- Whether passengers inside a transport (not a hospital) self-heal: the original is believed not to
  heal them while they are inside. This question covers open-topped transports too, whose
  passengers are in play and take turns (glossary, *in play*); no measurement has settled it for them.
- Whether the depot's reply to a full-depot entry request has any side effect besides answering accept
  (claim 23). Its effect on the vehicle's destination is settled (none, Q-1158); H18 f(ii) would show any other effect.
- Hospital-specific cross-creator suppression remains unmeasured (claim 7); S-0701 claim 28,
  G7 proposes tests for the *unit repaired* row and trigger/depot pair, not every creator.
  No completed measurement is claimed. Zoom progression and cadence are specified by
  S-0410 claims 14–15, with their exact floating-point equality boundaries still unmeasured.
  The post-zoom tail begins at the completion frame's tick number. There is no separate
  unresolved zoom-duration rule here.


## Changes for other specs

- S-0503 claim 10, first sentence: "A hovering vehicle is switched off by the disabling of
  S-0405 claim 13, and switched on again by its re-enabling" becomes "A hovering vehicle is
  switched off by the disabling of S-0405 claim 13, by its movement arrival in a repair
  depot's docking-point cell (S-0314 claim 17), by the settling processing of a repair
  depot that makes it settled, and by each later settling processing of that depot that finds
  it within 200 leptons of its docking point, settled, standing still and not yet down
  (S-0314 Terms and claim 21), and
  switched on again by its re-enabling and when it stops being settled, also while it is still
  disabled (S-0314 claim 21). Each switch-off meets
  the conditions below on its own, so a repeated switch-off of a vehicle that is still above
  the ground takes the two draws again whenever those conditions hold at it (none while its
  current behaviour is *sleep* or *enter*)". Claims 8, 10 and 11's effects of being switched
  off then apply unchanged, claim 11's water test at the end of a spin included: a depot's
  switch-off that starts a spin over water, at a naval yard standing on water, ends by default
  in the vehicle's loss (unless `quirks.docked_hover_water_loss` is off) unless the spin ends at
  least 104 leptons above the water (S-0314 claim 21).
- S-0503 claim 11, after "A switch-off that starts no spin, while no spin is active, makes no such
  test, and the vehicle rests on the water.", gains: "With S-0314's quirk
  `quirks.docked_hover_water_loss` off, a spin end that comes while the vehicle has contact with a
  repair depot or naval yard or is settled at one (S-0314 Terms) makes no water test: the vehicle
  is not given the bomb mark and does not fall. Once its contact has ended and it is not settled at
  a depot, a later spin end is tested as usual by this claim (S-0314 claim 21 and Edge cases). The
  default is the original's loss." S-0314 scenario H4 part i run 6 tests the default loss at a
  naval yard.

- S-0405 claim 20, *Movement and turning*: "a scatter request is refused because its
  locomotor is off" becomes "a scatter request is refused because its locomotor is off, unless a
  repair depot, or a destination in a tank bunker's cell, has switched it on again at an end of
  being settled (S-0314 Terms and claim 21)". A disabled vehicle in contact with a depot is
  switched on when it stops being settled, like any other vehicle (S-0314 claim 21), and so is a
  disabled hovering vehicle that is down in a cell that a tank bunker occupies when a destination
  takes effect there (S-0314 Terms, *What ends it* and *Away from a depot*). Its second state,
  "after it has been switched on again at an end of its being settled at a repair depot",
  becomes "… at a repair depot or in a tank bunker's cell", with the same switch-on that leaves
  it disabled and does not reset its guard state. The claim's "It starts no new movement of its
  own" takes the same exception for that scatter in both cases; claim 20's other rules for it
  are unchanged. Otherwise S-0405
  claim 20 (enabling and disabling) states its own effects on the vehicle without using this
  spec's *settled* term. What the enabling and the disabling do to a vehicle's being settled is
  stated once, in this spec's Terms and claim 21, which own the term. S-0606 needs no change:
  leaving a tank bunker is not an end that this spec states (claim 22).
- Checked, no change needed: S-0504 claim 8's sentence that settling is scheduled 3 ticks
  after the processing that first finds the vehicle at rest, and its state table's row
  "settling scheduled 3 ticks later", concern the waiting-to-settling delay, not the first
  step; S-0606 claim 20 uses the depot's processing delay and states no first-step tick.
- S-0504 claim 8 and scenario N8: a vehicle becomes settled on its movement arrival in the
  docking-point cell (S-0314 Terms). If it is still settled when its first settling processing
  comes, 3 ticks after the accepting waiting processing, the first repair comes at that
  processing. If something has ended its being settled by then, for example a waiting
  processing that did not accept it, that processing makes it settled and takes no step, and
  the first repair comes at the next processing, 4 ticks after the acceptance. For a hovering
  vehicle it comes not before the vehicle has come down (claims 6 and 21). S-0504 claim 8
  already makes the first step wait for completed
  locomotor settling and says that scheduling settling "does not guarantee a paid repair on that
  tick"; this states when that condition is first met. Sentence by sentence:
  - claim 8, "At that later processing, the first repair step additionally requires …
    completed locomotor settling": add "Settling checks, when the vehicle becomes settled and first-step timing follow
    S-0314 claims 6 and 21."
  - claim 8, "scheduling settling does not guarantee a paid repair on that tick": unchanged.
  - claim 8, "The settling phase's separate 200-lepton distance check controls approach and
    stopping, not eligibility for a repair step": "The settling phase's separate 200-lepton
    distance check controls approach and stopping. Within 200 leptons, a step also needs the
    vehicle to be settled and, if it hovers, down on the ground (S-0314 claims 6 and 21); at 200
    leptons or more, being settled and being down on the ground play no part in whether a step
    comes." That exception also holds for the N8 wording below: N8's set-up docks at the
    docking point, within 200 leptons. The next sentence, "It is not a second
    repair-distance gate", and N8's matching sentence stay: no distance by itself refuses a
    step to a vehicle standing still.
  - N8 Expected, "the first repair occurs then": "the first repair occurs at the first
    settling processing that finds the vehicle settled and, if it hovers, down on the ground:
    3 ticks after the successful waiting gate for a vehicle still settled from its arrival,
    otherwise 4 ticks or more after it".
  - N8 Falsified if, "the first paid repair step does not occur at that settling processing":
    "… does not occur at the first settling processing that finds the vehicle settled and, if
    it hovers, down on the ground".
  - Unchanged: the 3-tick waiting-to-settling delay, and claim 8's high confidence for its
    locomotor test as the choice of starting distance by locomotor kind. If that rating also
    covers the completed-settling condition, it should be narrowed to medium for when that
    condition is first met, as in S-0314 claim 6.
