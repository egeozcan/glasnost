---
id: S-0408
title: Selling and building repair (refund, what a sale releases, selling vehicles and walls, which buildings can be sold or repaired)
topic: economy
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-13
measurements: []
requests: []
supersedes: []
---

# S-0408: Selling and building repair (refund, what a sale releases, selling vehicles and walls, which buildings can be sold or repaired)

## Summary

What selling yields: the refund formula and the cost it starts from, the course of a building's
sale (the upgrade that goes first, the passengers and survivors that come out, the refund at the
end), the sale of a wall, a rule for a sell order on a unit (no input creates one), and which
buildings input lets the player sell or repair. The timing, step and switch-off of a building's
repair are S-0314's (claims 8 and 5); the orders are S-0213's (claims 12 and 13), their input
S-0214's (claim 13); the spec covers skirmishes only.

## Terms

Shared terms used here are defined in the glossary: *house*, *country*, *building*, *vehicle*,
*infantry*, *type*, *field*, *credits*, *tick*, *health*, *strength*, *on the map*, *off the map*,
*in play*, *removed*, *construction state*, *behaviour*, *foundation*, *scenario random stream*,
*scatter request*, *AI house*, *human-controlled*, *computer-controlled*, *departure*, *importer
warning*, *quirk switch*, *absent-key default*. S-0205's notation applies: trunc, RZ₅₃, RZ₂₄,
*single* and *double*. S-0202's ranged draw R(*a*, *b*) and percent draw apply. S-0405's terms
*switched on*, *in operation*, *being drained*, *absorbing building* and *upgrade* apply. From
S-0214 (Terms): *vehicle-like building* (a building whose type undeploys into a vehicle and has a
1 × 1 foundation). From S-0211 (Terms): the *firestorm-wall flag* and a house's *firestorm* (a
house state, on or off). In addition:

- **Sale.** The course of selling one building, from the first processing of its *selling*
  behaviour (S-0215) to its removal or its return to another behaviour (claims 4 to 10).
- **Docked unit.** A unit linked to a building for a service it gives, such as unloading at a
  refinery, repair on a depot or landing on a pad, from the moment the link is made until it is
  broken. It is a kind of contact with the building (glossary, *contact*); whether it is also
  tethered, and which services make such a link, belong to the specs of those services (open
  question on claim 6).
- **Build-down.** The part of a building's construction state in which it plays its build-up
  backwards while it is sold; S-0108 (claim 5) owns its timing.
- **Refund.** The credits a house receives for an object it sells (claim 1).
- **Adjusted cost.** The cost a building type is refunded from (claim 2). For every other type it is
  the type's cost field.
- **Free unit.** The vehicle type that a building type's free-unit field names (Import mapping); a
  type *has a free unit* when that field names one.
- **Country cost factor *a*, cost-bonus product *b*.** The two factors of S-0404 (claim 2) for the
  type's kind (infantry, vehicles, aircraft, ordinary buildings, defences): *a* the country's cost
  factor, *b* the product of the cost bonuses of the house's cost-bonus buildings, both singles,
  each as it stands when the refund is computed.
- **Refund percentage *r*.** The game-wide refund percentage (Import mapping), narrowed to single
  precision when used.
- **Refund override.** An optional integer field of a type that, when not 0, replaces the cost,
  the cost-bonus product and the refund percentage in the refund (claim 1).
- **Pad aircraft.** The game-wide list of aircraft types that come with the buildings they dock at
  (Import mapping). The *separate-aircraft setting* is a game-wide boolean that, when off, makes the
  first such dock building come with its aircraft. An aircraft type's *dock types* are the building
  types it docks at, in the order of its list (S-0404, claim 2); the *first dock* of the first pad
  aircraft is the first entry of that list.
- **Sale survivors.** The infantry units that a sold building releases (claim 7), as opposed to the
  survivors of a destroyed building (S-0312, claim 18).
- **Firestorm active.** A house's firestorm is *active* when it is on. In the original no event
  turns it on (S-0211, claim 11), so it is never active there.
- **Refinery (sense).** In claim 6, *refinery* is a building at which harvesters unload (S-0402's
  sense).
- **Engineer type.** An infantry type with the engineer flag (S-0214, Import mapping). The
  *technician type* and the *side crew type* are those of S-0312 (F4); either may, in odd data,
  be an engineer type.
- **Armed** (scenario S6 only). In S-0312's sense (F4): the object has the primary
  weapon of S-0312's claim 6 second choice, or, for a building infantry can occupy, its occupants
  can fire from it.
- **Builder of a wall overlay.** The first building type in the registry whose wall overlay is
  that overlay, as defined in S-0406 (Terms, *builder of a wall overlay*). "The building type that
  builds that wall" means this builder, even when other types name the same overlay or the builder
  itself has no wall flag. The unsellable flag in claim 12 is that first type's.
- **Silent wall sale.** A wall sale (claim 12) that plays no sound whoever the house is. The only
  one is the removal of walls under a building being placed (S-0406, claim 9). A sale ordered by
  a sell-wall order is not silent.
- **Sellable.** The test of claim 3, which the sell mode of S-0214 (claim 13) calls "can be sold".
- **Repairable.** The test of claim 14, which the repair mode of S-0214 (claim 13) calls "can be
  repaired".

## Behaviour

### The refund

1. **Refund of an object.** A house that sells an object of type *T* receives

   > refund = trunc(RZ₅₃(*v* × *a*)) when *T*'s refund override *v* is not 0,
   > refund = trunc(RZ₅₃(*P* × *r*)) with *P* = trunc(RZ₅₃(RZ₅₃(*C* × *b*) × *a*)) otherwise,

   where *C* is *T*'s adjusted cost (claim 2) and *a*, *b* are the factors for *T*'s kind as they
   stand at that moment (Formulas F1). The refund does not depend on the object's health, on its
   veterancy, or on the price the house paid for it: a house whose cost-bonus buildings changed since
   the purchase is refunded at the new factors. With a refund override the refund percentage and
   the cost-bonus product play no part.
   - *Computer-controlled houses in the original.* The original applies the refund percentage only
     for a human-controlled house: a computer-controlled house receives *P* in full. In our engine
     every player house, AI houses included, receives the refund given above (glossary *AI house*); the
     original's full refund is recorded for the designers of our AI only. The neutral and
     special houses keep the original's rule (full *P*), though no rule of the original makes them
     sell.
   - Backing: `measurable` (scenario S1, parts a to e)
   - Confidence: high (the formula, no health dependence, the percentage only for human-controlled
     houses), medium (the refund override)

2. **Adjusted cost of a building type.** A building type's adjusted cost is its cost field less two
   deductions (Formulas F2). The first is the *pad-aircraft deduction* trunc((*C*₀ + *C*₁) / 2),
   where *C*₀ and *C*₁ are the cost fields of the first two pad aircraft; it applies only when the
   separate-aircraft setting is off and the type is the first dock of the first pad aircraft
   (Terms). The second is the cost field of the type's free unit, when it has one. Only a type with
   a free unit has its adjusted cost raised to 0 when it would be negative; without a free unit a
   pad-aircraft deduction larger than the cost field gives a negative adjusted cost, and so a
   negative refund (Edge cases). The same adjusted cost is the
   type's cost in S-0314's step cost (claim 5, which names this adjustment as the production specs'
   to give, and whose open question about the further hard-coded case it answers: the pad-aircraft
   deduction) and in the survivor count of S-0312 (F5, through the refund).
   - Backing: `measurable` (scenario S2)
   - Confidence: high (the free unit and the floor that comes only with it), medium (the
     pad-aircraft deduction and its rounding)

### Selling a building

3. **Which buildings can be sold.** A building is sellable when it is not being drained, its type
   does not have the unsellable flag, and either (a) its type has a build-up sprite, its
   construction has completed, and its current behaviour is neither *construction* nor *selling*,
   or (b) its type has the firestorm-wall flag and its house's firestorm is not active (Terms), with
   no condition on a build-up sprite or on the construction state. In the original (b) therefore
   holds for every firestorm-wall type, since no firestorm is ever active. The sell mode of S-0214
   (claim 13) offers *sell* on an own sellable building that is not a vehicle-like building.
   - Backing: `measurable` (scenario S3, parts a, d and e); the case of an active firestorm is
     `unmeasured`: no event of the original turns a firestorm on (S-0211, claim 11), so no setup
     reaches it
   - Confidence: medium; low (the firestorm-wall branch)

4. **What a sell order does to a building.** When a sell order (S-0213, claim 12) reaches a
   building whose type has a build-up sprite and whose current behaviour is not *selling*, the
   building's behaviour becomes *selling*; its sale (Terms) begins with that behaviour's first
   processing. A sell order to a building that is already selling changes nothing, apart from the
   order sound that the local player hears again for an own building. A building whose type has no build-up sprite is not sold: if its
   type has the firestorm-wall flag it is removed at once with no refund and no survivors,
   otherwise nothing happens (and no sound plays). The order does not test the unsellable flag or the other conditions
   of claim 3; input does (S-0214, claim 13). A building that packs up instead of being sold,
   its course, what it releases and the failed pack-up are S-0415's (claims 9 to 11), which this
   spec does not repeat.
   - Backing: `measurable` (scenario S3, parts b, c and d; part c reaches the "nothing happens"
     branch only through a vehicle-like building, the one input path that issues a sell order to a
     type that sell mode never offers *sell* on, S-0213 claim 12)
   - Confidence: medium (the build-up condition, the second order), low (the firestorm-wall case,
     which part d measures; that part c's click path issues the order)

5. **An upgraded building sells its top upgrade.** When a sale begins on a building that holds at
   least one upgrade (S-0405, Terms), only the most recently added upgrade is sold: the house
   receives the refund of claim 1 for the upgrade's type, the upgrade is removed, the house works out
   its power again with the upgrade no longer counted (S-0405, claim 7), and the building goes to *guard*, otherwise unchanged. Each
   further sale removes one more upgrade; a sale on a building without upgrades sells the building.
   Selling an upgrade takes one creation draw of S-0202 (claim 10, every creation of a building
   object takes one), as the creation of a building object of the upgrade's type would, although
   no such object appears.
   - Backing: `measurable` (scenario S4, parts a and b; the creation draw is counted in part b
     by comparing the scenario random stream's state before and after the sale)
   - Confidence: medium (one upgrade per sale, its refund, the creation draw)

6. **The course of a sale.** The selling behaviour's processings are 1 tick apart (claim 16).
   The sale of a building without upgrades has the following observable states. A building that packs up goes through the same course with the differences of S-0415 (claim 9), among them which buildings release survivors, passengers and garrison (`quirks.packup_release_by_guard_spot`); the releases below are those of an ordinary sale.

   | Sale state or condition | Observable result |
   |---|---|
   | First processing | Every docked unit (for example, a harvester unloading at a refinery, a vehicle on a repair depot, or an aircraft on its pad) is told to leave; a vehicle held in the building's bunker is let out; burning-damage animations end. No other release occurs and build-down has not begun. |
   | A later processing while any unit remains docked | The sale waits. |
   | First processing at which no unit is docked | Absorbed passengers, garrisoned infantry and then sale survivors (claims 7 to 9) are released, in that order. The local player hears the sell sound for an own building, and build-down begins. The order is observable in the positions taken (an earlier unit can occupy a standing position a later unit would otherwise take) and in the order of release draws (claim 9 for the occupants). |
   | Build-down | It lasts for the time S-0108, claim 5, gives. |
   | First processing after build-down has finished | The house receives the building's refund (claim 1), then the value of its stored resources (claim 10), and the building is removed. |

   With no unit docked, the release and build-down-start state is reached on the second processing, one tick after the first.

   The building is not in operation from the start of the sale (S-0405, claim 8), so a defence stops
   firing at once, but a power plant keeps its output until it is removed (S-0405, claim 3). A
   building destroyed during its sale pays no refund. Its sale stops at the killing hit: no
   further sale survivors appear and no later selling processing completes the sale. Its death
   effects happen at once, but it keeps occupying its foundation until its own next eligible
   turn, later in the same tick if that turn has not yet come, otherwise in a later tick. Being
   erased or under an electromagnetic-pulse effect postpones removal. On the removal turn it
   ceases to occupy its foundation and takes a second destruction-survivor and smudge round,
   with the count worked out afresh, as S-0312 (claims 18, 19(g) and 21) gives under
   `quirks.building_removal_second_round`. Sale survivors released before the killing hit remain
   unaffected. Killing damage that ignores defences leaves no crew and therefore no survivors
   in either destruction round. A sale that removes the still-living building before a pending
   delayed-kill expiry instead leaves no death or destruction rounds (S-0312 claim 3).
   - Backing: `measurable` (scenario S5, parts a to f; S-0312 scenario D14, variant g)
   - Confidence: high (refund at the end, no refund when destroyed, the stopped sale, the next
     eligible own-turn removal and second destruction round, the releases before the
     build-down), medium (the step timing of the releases), low (which links count as docked and
     hold the sale; the bunker release, measured by S5 part d; the place of the garrison in the
     order, measured by S5 part a)

7. **Sale survivors.** A sold building makes exactly *C* survivor attempts, *C* being the count of
   S-0312 (Formulas F5), whose *K* is the type's refund for the owner (claim 1) as it stands at that
   step; there is no chance draw per cell. Each attempt takes the draws below from the scenario
   random stream, in this order, whether or not its survivor ends up on the map:

   | Draws | Taken | Decides |
   |---|---|---|
   | The type choice's draws (table *Type choice* below) | first, every attempt | The survivor's type, as for a destroyed building (S-0312, claim 18, table *Type*). |
   | The whole type choice again, with all its draws | each time the choice gives an engineer type and this sale has already chosen one | Repeated until it gives another type. |
   | The creation draw of S-0202 (claim 10) | once the type is chosen | None. |
   | R(0, *n* − 1) | once the type is chosen | One of the building's *n* foundation cells, in the cell order of S-0302 (claim 13). |
   | R(0, 3) | unless the cell's record blocks every position (the first row of S-0208's table, claim 3) | The row of standing positions to try. |

   *Type choice.* The crew choice's own draw is taken only when S-0312's F4 takes one.

   | Building | Percent draw *p* | Crew choice's own draw | Survivor's type |
   |---|---|---|---|
   | Has never changed owner, its type produces buildings, *p* < 25 | taken | not taken | an engineer |
   | Has never changed owner, any other case (*p* ≥ 25, or a type that does not produce buildings) | taken, whatever the type | taken, after *p* | the crew type of S-0312 (F4) |
   | Has changed owner | not taken | taken | the crew type of S-0312 (F4); no engineer through *p* |

   The survivor takes the first free position of the row that the R(0, 3) draw chose, in the
   chosen cell. The positions are those
   of S-0208's table (claim 3) for a desired point 128 leptons along the map x axis and 164 leptons
   along the map y axis from the cell's top corner, 36 leptons from the cell centre, so of class
   centre. There is no search of neighbouring cells. When no position of the row is free, or the
   survivor cannot be put there, no survivor appears and the attempt is lost; the draws it took stay
   taken. The first engineer chosen counts even if its attempt is lost for want of a place, so a sale
   releases at most one engineer. A survivor that appears has full strength (no health draw), facing
   0 (S-0204) and a forced scatter request (S-0208), and is on the *move* behaviour, whoever owns it.

   Since the centre position is never given, one cell takes at most 3 survivors, fewer when absorbed
   passengers (claim 8) or other infantry already stand there; on a 1 × 1 building at most 3 of 5
   attempts can succeed. A building with no crew (S-0312, F5) releases none.
   - Backing: `measurable` (scenario S6, parts a to d; the draws by comparing the scenario random stream's state before and after the sale)
   - Confidence: high (the count of attempts and its *K*, the placement within the chosen cell, that
     the percent draw is taken for every building that has never changed owner), medium (the type
     choice, the single engineer, full health, the other draws and their order)

8. **Absorbed passengers on a sale.** A sold absorbing building releases its absorbed passengers at
   the step of claim 6, by the placement rule of S-0312 (claim 24): one per foundation cell in the
   cell order, a vehicle at the cell centre and an infantry unit on a free standing position of its
   cell (chosen as for a sale survivor, claim 7, with the same desired point and so one R(0, 3)
   draw when the cell is not blocked), each facing the building's facing, with a forced scatter
   request, and on *hunt* when its owner is computer-controlled in the original (our engine: no
   order for a player house, as S-0312 claim 25 gives; the neutral and special houses keep the
   original's rule, glossary *AI house*). A building with no crew deletes them instead. A passenger that
   cannot be placed is deleted. With more passengers than foundation cells, S-0312's edge case
   *Absorbed passengers beyond the last cell* applies here too. The passengers come out before the
   survivors, so they can take positions the survivors would otherwise get.
   - Backing: `measurable` (scenario S6, part e)
   - Confidence: medium

9. **Garrison on a sale.** A sold garrisoned building puts out its occupants at the step of claim 6,
   after its absorbed passengers (claim 8) and before its sale survivors (claim 7). Where each
   occupant stands and how it leaves belong to the structures specs; this spec fixes only the
   place in the order, which shows in two ways: the draws that the occupants' leaving takes come
   before the survivors' draws, and an occupant that stands on a position inside the foundation
   can take it from a survivor.
   - Backing: `measurable` (scenario S5, part a, which records occupants' and survivors' positions
     and draws)
   - Confidence: medium (that the occupants leave at that step); low (their place before the
     survivors)

10. **Stored resources on a sale.** In the processing in which a sold building is removed, its house
    is paid, after the refund (claim 6), for the resources the building stores: for each resource
    type, the whole units of the stored amount, *m* = trunc(stored amount), at *w* × *i* credits per
    unit (Formulas F3), where *w* is the resource type's value per bale (S-0401) and *i* the country's
    income factor (S-0402, claim 12). A fraction of a unit is lost with the building, unpaid. The
    types are paid in the building's storage order, which shows only when the balance is negative
    (F3). The house's own storage (glossary, *available credits*) does not change.
    In ordinary play a building's storage stays empty, because harvested resources are paid
    straight into the house's balance (S-0402, claim 12), so this rule matters only when something
    else fills a building's storage; no such path is known in a skirmish.
    - Backing: `unmeasured` (no known skirmish setup fills a building's storage; scenario S7 applies
      if one is found)
    - Confidence: medium (the formula, the truncation of the amount), low (whether any skirmish path
      reaches it)

### Selling other things

11. **A sell order on a vehicle or aircraft.** A vehicle or aircraft that a sell order reaches
    while a building occupies its cell (S-0213, claim 12; otherwise the order does nothing) is
    removed at once, with no animation and no survivors; its house receives its refund (claim 1) and
    its passengers are dealt with (open question). The local player hears the unit-sold announcement
    for an own unit. No input creates such an order (S-0213, claim 12; S-0214, claim 13), so in a
    skirmish no unit is ever sold; the rule is stated so that every sell order has a defined result.
    - Backing: `unmeasured` (no input creates a sell order for a vehicle or aircraft, so neither
      scripted input nor an AI house under the same input rules can reach the rule; no scenario)
    - Confidence: medium (the refund, the immediate removal), low (the passengers)

12. **Selling a wall.** A sell-wall order (S-0213, table T1) for a cell removes the wall in it when
    all of these hold: the cell holds a wall overlay, the wall has an owner, the owner is
    human-controlled (in our engine: any player house, glossary *AI house*), and the first building
    type in the registry whose wall overlay is that overlay exists and does not have the unsellable
    flag (S-0406 Terms, *builder of a wall overlay*). Only that first type's flag matters; a later
    type naming the same overlay cannot make the wall sellable. The silent wall sale of
    S-0406 (claim 9) follows the same rule, without the sound. The order does not check
    that the wall's owner is the issuing house; input offers it only on the acting house's own walls
    (S-0214, claim 13). The wall leaves its cell with no owner, the cell's passability and the
    pathfinding regions are worked out again (S-0211, S-0212), and the sell sound plays when the
    issuing house is the local player's and the sale is not a silent wall sale (Terms). **The house receives
    nothing** (`quirks.wall_sale_no_refund`, Edge cases).
    - Backing: `measurable` (scenario S8; S-0406 scenario PL8 part k for the same builder choice
      during a silent wall sale)
    - Confidence: medium

### Building repair

13. **Repair belongs to S-0314.** The repair order (S-0213, claim 13) switches a building's repair
    on and off; while it is on, the timing, the step and when repair switches off are S-0314's
    (claim 8) and the step cost is S-0314's (claim 5, with the adjusted cost of claim 2 of this spec
    for a building type). The automatic repair and sale gated by the house skill level are S-0314's too
    (claim 8).
    - Backing: `measurable` (S-0314 scenario H5)
    - Confidence: high

14. **Which buildings can be repaired.** A building is repairable when its health is above 0 and
    below its strength, its type has both the repairable flag and the click-repairable flag (both
    on unless the data turns them off), and it is not a vehicle-like building (Terms). The repair mode of S-0214
    (claim 13) offers *repair switch* on an own repairable building. The repair order itself does
    not test this (S-0213, claim 13).
    - Backing: `measurable` (scenario S9)
    - Confidence: medium

### Owners

15. **No owner exceptions in our engine.** Every claim of this spec applies to every player house
    as it applies to a human-controlled house, AI houses included (glossary *AI house*). The
    original's computer-only rules are recorded in Edge cases for the designers of our AI.
    - Backing: `measurable` (scenario S1, part f, shows the original's exception that our engine
      drops)
    - Confidence: high

### Timing

16. **The selling behaviour runs every tick.** At every stage of a sale (the first processing, a wait
    for a docked unit, the start of the build-down, the wait for the build-down, the final
    processing), the *selling* behaviour is processed again on the next tick (a processing delay of 1
    in the terms of S-0215, claim 2), and its timing takes no draw. The processings of a sale
    therefore fall on consecutive ticks. (S-0215, claim 34, names the structures topic as the owner
    of this behaviour's processing but gives no delay.)
    - Backing: `measurable` (scenario S5, the tick of each stage)
    - Confidence: high

## Formulas

All amounts are integers in credits (S-0205, claim 1). Arithmetic results are RZ₅₃ with
`quirks.chop_rounding` on (the original) and RN₅₃ with it off, and the narrowing of *r* to single is
RZ₂₄ on and RN₂₄ off (S-0205); trunc is S-0205's conversion. The original computes F1 and F3 in floating point.

**F1, refund** (claim 1). *v* the refund override, *C* the adjusted cost (integers), *a* and *b*
singles, *r* the refund percentage narrowed to a single:

> *v* ≠ 0: refund = trunc(RZ₅₃(*v* × *a*))
>
> *v* = 0: *P* = trunc(RZ₅₃(RZ₅₃(*C* × *b*) × *a*)), refund = trunc(RZ₅₃(*P* × *r*)) for a player
> house of our engine (and a human-controlled house of the original); refund = *P* for the neutral
> and special houses (and a computer-controlled house of the original).

Example (invented values): *C* = 1000, *b* = 1, *a* = 1, *r* = 0.5: *P* = 1000, refund 500. With
*b* = 0.75: *P* = 750, refund 375. With *r* written as 0.7 (held as the single just below 0.7, S-0003 claim 6): *P* = 1000, 1000 ×
that single is just below 700, refund 699.

**F2, adjusted cost of a building type** (claim 2). *K* the cost field, *F* the free unit's cost
field, *C*₀, *C*₁ the cost fields of the first two pad aircraft, *d* = trunc((*C*₀ + *C*₁) / 2)
when the type is the first dock of the first pad aircraft and the separate-aircraft setting is off,
*d* = 0 otherwise:

> *C* = max(0, *K* − *d* − *F*) for a type with a free unit,
>
> *C* = *K* − *d* for a type without one (it can be negative).

Integer arithmetic; the halving truncates toward zero.

**F3, stored resources** (claim 10), per resource type with stored amount *s* (a single), value *w*
(integer) and income factor *i* (single):

> *m* = trunc(*s*), held as a single; credits′ = trunc(RZ₅₃(RZ₅₃(RZ₅₃(*w* × *i*) × *m*) + credits)).

The credits are added inside the floating-point sum and the total truncated once. With a negative
balance (possible through a negative refund, Edge cases) this differs from truncating the payment
first, which is why the order of payments matters there: the refund first, then the resource types in
storage order (claim 10).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Refund percentage, absent-key default | 0.5 | fraction | Claim 1 |
| Refund override, absent-key default | 0 | credits | Claim 1; 0 means no override |
| Selling processing delay | 1 | ticks | Claim 16 |
| Engineer chance for a building that produces buildings (and has never changed owner) | percent draw below 25 | percent | Claim 7 (as S-0312 claim 18) |
| Engineers per sale, at most | 1 | infantry units | Claim 7 |
| Desired point of sale survivors and absorbed passengers within the chosen cell | 128, 164 | leptons along map x, map y from the top corner | Claims 7 and 8; class centre (S-0208 claim 2) |
| Sale survivors per foundation cell, at most | 3 | infantry units | Claim 7 (the centre position is never given) |
| Repairable and click-repairable flags, absent-key default | yes | boolean | Claim 14 |
| The unsellable flag, absent-key default | no | boolean | Claim 3 |
| Survivor divisors | S-0312's (F5 and its Import mapping) | credits per survivor | Not restated here |
| Type choices for one sale survivor, at most (our engine) | 100 | repetitions | Edge cases, *A crew choice that always gives an engineer* (departure) |

## State and ordering

| State | Event | Result |
|---|---|---|
| building, any behaviour but *selling* | sell order, type with a build-up sprite | *selling* begins |
| *selling*, first processing | building holds upgrades | top upgrade refunded and removed; building to *guard* (claim 5) |
| *selling*, first processing | no upgrades | docked units told to leave; bunkered vehicle let out; burning animations end |
| *selling*, a later processing | a unit still docked | waits |
| *selling*, a later processing | nothing docked | absorbed passengers, garrison, sale survivors released (claims 7 to 9); sell sound; build-down begins |
| building, any behaviour, type with no build-up sprite | sell order, type with the firestorm-wall flag | removed at once; no refund, no survivors (claim 4) |
| building, any behaviour, type with no build-up sprite and no firestorm-wall flag | sell order | nothing (claim 4) |
| build-down | build-down finished, next processing | refund and stored resources credited; building removed |
| *selling*, any stage (the build-down included) | building destroyed | sale stops; no refund or further sale survivors; awaits its own next eligible turn, with a second destruction-survivor and smudge round on removal (claim 6; S-0312 claim 21) |
| *selling* | sell order | nothing changes, except that the local player hears the order sound again for an own building (claim 4) |
| vehicle or aircraft on a building | sell order (no input creates one) | refund; removed at once (claim 11) |
| wall cell | sell-wall order, conditions of claim 12 | wall removed; no credits |

Observable ordering: the survivors stand on the map before the refund arrives, and the refund
arrives in the same processing in which the building is removed, so a house's credits rise on the
tick the building disappears.

## Edge cases and quirks

- **A wall sale pays nothing** (`quirks.wall_sale_no_refund`, default: the original). With the
  switch off, the house receives the refund of claim 1 for the builder of the wall overlay (Terms).
  This is a quirk, not a rule: the missing payment looks like a lost refund rather than a design.
- **At most one engineer per sale (rule, not a quirk).** The repeated choice of claim 7 is the
  original's rule: it keeps a sale from releasing several engineers. It is reproduced without a
  switch; each repetition takes its draws. The 100-repetition limit for a choice that always gives
  an engineer is a separate departure for bad data, described below.
- **Pad-aircraft list shorter than 2** (departure). When the list has fewer than two entries, or
  the first pad aircraft has no dock, the original's adjusted cost (F2) is undefined for every
  building type, not only for dock types. Our engine counts a missing entry as cost 0, applies no deduction when the list is empty
  or the first pad aircraft has no dock, and the importer warns in each of these cases.
- **Negative adjusted cost without a free unit.** When the pad-aircraft deduction exceeds the cost
  field of a dock type with no free unit, the adjusted cost (F2) is negative, and so are its refund
  (a sale lowers the house's credits), its repair step cost basis and its survivor count basis.
  Reproduced; the importer warns.
- **Refund percentage outside 0 to 1.** Used as given, after narrowing to single precision; a
  negative percentage gives a negative refund, which lowers the house's credits. Reproduced; the
  importer warns.
- **Building without a build-up sprite.** It cannot be sold (claims 3 and 4), except a
  firestorm-wall type, which vanishes with no refund. Reproduced; the importer warns for a building
  type with no build-up sprite that has neither the unsellable flag nor the firestorm-wall flag,
  since input would still not offer *sell* on it (a firestorm-wall type is sellable through claim
  3(b), so it gets no such warning).
- **Destroyed during its sale.** No refund or further sale survivors, at whatever stage (claim 6).
  Its second destruction-survivor and smudge round is covered by
  `quirks.building_removal_second_round` (S-0312): default original, with that second round omitted
  when off. If the sale removes the building while it is still alive, no later delayed-kill expiry
  destroys it.
- **A crew choice that always gives an engineer** (departure for bad data). When the data makes the
  crew choice of claim 7 give an engineer every time (for example a side crew type that is an
  engineer type), the original repeats the choice for ever after the first engineer of a sale and
  the game hangs. Our engine stops after 100 repetitions of one survivor's choice and releases no
  survivor for that attempt (its draws up to then stay taken), and the importer warns for a side
  crew type or a technician type that is an engineer type (Terms). This is a safety departure, not
  a quirk switch.
- **The original's computer player** (a note for the designers of our AI, not a rule of our
  engine). Besides the full refund of claim 1, the original, when a building holding upgrades changes
  owner to a computer-controlled house in a game that is not a campaign mission, sells its top upgrade
  at once at its full refund. It also has its computer houses sell walls when they are close to
  losing, which the wall rule of claim 12 refuses for walls they own. Low confidence; our AI may
  reproduce such behaviour through ordinary orders.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules, `[General]`, `RefundPercent` | refund percentage *r* (claim 1) | decimal, a percent sign allowed (S-0003, claims 6 and 7), held as a double, narrowed to single (RZ₂₄) when used | 0.5; a layer without the key keeps the value in force |
| Rules, type section, `Soylent` | refund override *v* (claim 1) | integer, credits | 0 |
| Rules, type section, `Cost` | cost field (claims 1, 2) | integer, credits | owned by the production specs |
| Rules, building section, `FreeUnit` | the free unit whose cost is deducted (claim 2) | vehicle type name | none |
| Rules, `[General]`, `PadAircraft` | pad aircraft list (claim 2) | list of aircraft type names | empty |
| Rules, `[General]`, `SeparateAircraft` | the separate-aircraft setting (claim 2) | boolean | owned by the aircraft specs |
| Rules, building section, `Unsellable` | unsellable flag (claims 3, 12) | boolean | no |
| Rules, building section, `FirestormWall` | firestorm-wall flag (claims 3, 4) | boolean | no |
| Rules, building section, `Repairable` | repairable flag (claim 14) | boolean | yes |
| Rules, building section, `ClickRepairable` | click-repairable flag (claim 14) | boolean | yes |
| Rules, `[General]`, `AlliedSurvivorDivisor`, `SovietSurvivorDivisor`, `ThirdSurvivorDivisor` | survivor divisors of S-0312 F5 (claim 7); S-0312's Import mapping owns them | integer | S-0312's absent-key defaults; keeps |
| Rules, type section, `Crewed` | building has a crew (claim 7, through S-0312 F5) | boolean | owned by S-0312 |
| Rules, `[General]`, `SurvivorRate` | nothing: the original reads it but no rule uses it | decimal | — |
| Rules, `[AudioVisual]`, `SellSound` | the sell sound (claims 6, 12) | sound name | sound specs |

## Measurement scenarios

All scenarios run under the measurement setup in a skirmish on a flat purpose-made test map, with
named test files where a scenario sets fields or defines test types. Credits are read from a
recorder of the house's credits (S-0404's *M*, not the displayed counter, which is presentation
only, S-0416), objects and health from a recorder or the interface, ticks with a recorder of the
tick number. Any game speed.

**S1: refund (claim 1).**
- *Setup:* an own, undamaged, crew-less test building type of cost 1000 with no free unit; the
  refund percentage absent unless a part sets it; country cost factors 1.
- *Procedure:* (a) sell it. (b) Sell one at a quarter of its strength. (c) Sell one while the house
  owns a cost-bonus building giving 0.75 for that kind. (d) Sell one with the refund percentage
  written as 0.7. (e) Sell one of a test type with a refund override of 800. (f) A
  computer-controlled house of the original sells an identical building (its own logic, or a test
  map's trigger), recorded for the AI designers.
- *Observable:* the change of the seller's credits at the building's removal.
- *Expected:* (a) +500; (b) +500; (c) +375; (d) +699; (e) +800; (f) +1000.
- *Falsified if:* (b) less than (a), (d) +700 (it can come only from a conversion to credits that rounds to nearest, or from a percentage left in double precision with the product rounded to nearest: the single just below 0.7 times 1000 is already below 700 and truncates to 699 whatever the rounding of the product), (e) +400, (c) or (f) anything other than its expected value.

**S2: adjusted cost (claim 2).**
- *Setup:* building test type A of cost 2000 with a free unit of cost 800; dock test type B of cost
  1000 that is the first dock of the first pad aircraft, pad aircraft costs 300 and 401; refund
  percentage 0.5.
  Further test types (named test file): C of cost 500 with a free unit of cost 800, and D of cost 300,
  the first dock of the first pad aircraft in a further game (pad aircraft costs 300 and 401).
- *Procedure:* (a) sell A. (b) Sell B with the separate-aircraft setting off, then in a second game
  on. (c) Sell C. (d) In the further game, with the setting off, sell D.
- *Observable:* the credit change at each removal. Country cost factors 1, no cost-bonus buildings.
- *Expected:* (a) A gives 600 (*C* = 1200, half); (b) B gives 325 (*C* = 650) with the setting off
  and 500 with it on; (c) C gives 0 (*C* raised from −300 to 0); (d) D gives −25 (*C* = 300 − 350 =
  −50, no free unit so no floor).
- *Falsified if:* A gives 1000, B gives 500 with the setting off, C gives a negative amount, or D
  gives 0 or more.

**S3: sellable test and the sell order (claims 3 and 4).**
- *Setup:* own buildings: one in its build-up, one being drained by an enemy draining unit, one of
  an unsellable test type, one completed ordinary building, one of a test type with no build-up
  sprite, and, from a named test file, one of a test type with the firestorm-wall flag and no
  build-up sprite and one of a second test type with the firestorm-wall flag and a build-up sprite.
- *Procedure:* (a) hover in sell mode over each of the first five. (b) Sell the ordinary building
  and give a second sell click while it sells. (c) Issue a sell order to a building of a test type with no
  build-up sprite and neither flag that undeploys into a vehicle and has a 1 × 1 foundation (a
  vehicle-like building): select it and give a move click on an open cell (the click that issues a
  guard-spot order and then the sell order, S-0213 claim 12, S-0214 table M part 3). Sell mode never
  offers *sell* on such a building, so this is the one path. (d) Hover in sell mode over the
  two firestorm-wall buildings, then sell each. (e) Repeat (d) with the unsellable flag also set on
  the firestorm-wall test types.
- *Observable:* the action offered; whether a sale starts or changes (and the order sound); in parts d and e the
  building's presence, the house's credits and whether survivors appear.
- *Expected:* (a) *no-sell* over the first three and the fifth, *sell* over the ordinary building,
  (b) the second click changes nothing to the sale, the order sound plays again; (c) the building is neither sold nor packed up and stays as it was; (d) *sell* over both; the
  one with no build-up sprite vanishes at once with no credit change and no survivors, the other
  goes through an ordinary sale with a refund; (e) *no-sell* over both (claim 3 requires the flag
  absent in both branches).
- *Falsified if:* any part differs; in particular (c) the building packs up or is removed, or (d) *no-sell* over the firestorm-wall building
  with no build-up sprite, or a refund for it.

**S4: upgrades (claim 5).**
- *Setup:* an own building holding two upgrades of different test types, the one added first of
  cost 300 and the one added second of cost 500; refund percentage absent (so 0.5), country cost
  factors 1, no cost-bonus buildings.
- *Procedure:* (a) sell it three times, waiting for each sale to end. (b) In a second run, on a
  map where nothing else takes a random draw, record the scenario random stream state before and
  after the first sale (an upgrade sale) and the second.
- *Observable:* (a) the credit changes; the upgrades and building that remain. (b) The number of
  draws taken between the sell order's tick and the end of each upgrade sale.
- *Expected:* (a) the first sale removes only the second upgrade (+250), the second the first
  (+150), the third the building. (b) Exactly one draw per upgrade sale (the creation draw of
  claim 5), taken at the sale's first processing, and no other.
- *Falsified if:* the first sale removes the building or both upgrades, or in (b) an upgrade sale
  takes no draw or more than one.

**S5: course of a sale (claim 6).**
- *Setup:* an own crewed test building of cost 1000, holding an infantry garrison, with a known
  build-down time (S-0108 claim 5); for part c an own refinery with a harvester docked and
  unloading; for part d an own tank bunker holding a vehicle; for part e the refinery of part c.
  For part f use a building with a pending delayed death and a sale that can finish before expiry.
- *Procedure:* (a) sell it, recording the scenario random stream state before and after the sale,
  and record where the occupants and the survivors stand. (b) In a second run, destroy it during
  its build-down. (c) Sell the refinery while the harvester is docked. (d) Sell the bunker. (e)
  Sell the refinery of part c and destroy it while the harvester is still docked. In parts b and e
  use killing damage that does not ignore defences, record the building's turns and the scenario
  random stream state, and repeat across 50 seeds. (f) Complete a sale before the delayed death
  expires and keep recording through the expiry tick.
- *Observable:* the tick of the sell order, the ticks at which the occupants and survivors appear,
  the first tick of the build-down, the tick of the credit change and of the building's removal,
  in (c) the tick at which the harvester leaves the dock, in (d) the tick at which the vehicle
  leaves the bunker, in (a) the draws of the occupants' leaving against the survivors' draws.
  In parts b and e record both destruction rounds' survivor appearances and draws, the killing
  tick, foundation occupancy until removal and the removal turn. In part f record the refund
  and any effects at the former building's position after removal.
- *Expected:* (a) occupants, then survivors, appear 1 tick after the sale's first processing (within
  3 ticks of the order), on the tick the build-down starts; the credits rise on the tick the
  building disappears, after the build-down; (b) no credit rise; (c) the harvester is told to leave
  at once, and the releases and the build-down start only once it has left the dock; (d) the
  vehicle leaves the bunker at the sale's first processing, before any release; (e) no credit rise
  and no sale survivors (only the destruction survivors of S-0312, if any). In both b and e the
  sale stops at death, the foundation remains occupied until the next eligible own turn, and
  removal takes a second destruction-survivor and smudge round, with all the applicable draws
  of S-0312 claim 21. The second round need not produce a survivor on every seed. (f) The sale
  pays its normal refund and removes the living building; no death effects or survivor rounds
  occur at the later expiry.
- *Falsified if:* the refund comes at the start of the sale, the destroyed building pays (in (b) or
  (e)), the build-down starts before the releases, in (c) the build-down starts while the
  harvester is still docked, in (d) the vehicle stays until the releases, or in (a) the survivors'
  draws come before the occupants' leaving or a survivor stands on a position an occupant has
  taken (the order of claim 9 is then wrong; it stays low confidence until measured), or in (a) the
  occupants and survivors appear more than 3 ticks after the order. Also falsified if b or e
  advances the sale after death, frees the foundation before its next eligible own turn or omits
  the second round's applicable draws, or if f produces a death after the completed sale.

**S6: sale survivors (claims 7 and 8).**
- *Setup:* survivor divisor 100 for the owner's side (a named test file); crewed test buildings
  with refunds 500 and 150; a crewed building type that produces buildings, in two variants with
  refunds 100 (*C* = 1) and 500 (*C* = 5), both with foundations of at least 3 × 3; an absorbing crewed
  test building holding three infantry, and a second test type identical to it except that its
  crewed flag is off; a crewed test building that has a primary weapon and an otherwise identical
  one without a weapon (both of refund 500, neither producing buildings), and a crewed armed
  building that produces buildings, of refund 500.
- *Procedure:* (a) sell the 500 and 150 buildings. (b) Sell each variant of the producing type 200
  times (fresh games or rebuilt), recording the scenario random stream state before and after each
  sale. (c) For about 20 sales of the producing building, to see both outcomes of its first draw,
  record the scenario random stream state before and after selling the armed and unarmed buildings
  of refund 500 and the armed producing building of refund 500, and record the draws. (d) Destroy the
  500 building in 50 runs. (e) Sell the two absorbing test types.
- *Observable:* the number, types and health of the survivors; the draws; the passengers; in part b
  the number of sales in which an engineer survivor appears on the map, and the number of sales in
  which some attempt's first percent draw is below 25 (read from the recorder).
- *Expected:* (a) 5 and 1 survivors, all at full strength, each standing in one of the building's
  own foundation cells before it scatters, unless one cell received more than 3 of the 5 cell
  picks (then exactly the excess is missing; use a building of at least 3 × 3 cells to make this
  rare); (b) never more than one engineer on the map in a sale; the number of sales with an attempt whose
  first percent draw is below 25 follows 1 − 0.75^*C* (a binomial count over 200 sales, mean 50 for
  *C* = 1 and about 152.5 for *C* = 5, standard deviation about 6): confirmed when it lies within
  32 to 68 for *C* = 1 and within 135 to 170 for *C* = 5 (three standard deviations); the number of
  sales with an engineer on the map equals it for *C* = 1 and is at most it for *C* = 5 (an
  engineer attempt can be lost for want of a place); (c) per survivor of a building that has never changed owner, in this order: the first percent
  draw *p*, taken whatever the type (so also for the two buildings that do not produce buildings),
  then the crew type's percent draw, only when the building is armed (Terms; so one draw fewer per
  survivor for the unarmed building) and, for the producing building, only when *p* ≥ 25 (a
  survivor with *p* < 25 is an engineer and takes no crew draw); then the creation draw, a ranged
  draw over 0 to *n* − 1, then a ranged draw over 0 to 3 for the standing position (counting out
  survivors whose cell record blocks every position, which take no last draw); (d) a varying count (S-0312 claim 18) where the sale's count
  never varies; (e) the passengers come out on the first three foundation cells, and none from the
  type with the crewed flag off.
- *Falsified if:* fewer survivors than the count above explains, a survivor that first appears
  outside the foundation, two engineers in a sale, a damaged survivor, the draws in another order,
  a missing first percent draw for a building that does not produce buildings (or an extra one),
  or in (b) a count outside the bands above (a count outside them for a producing type of either
  variant rejects the rate 1 − 0.75^*C*, not only the one-engineer rule), or in (e) a passenger
  that comes out elsewhere than on the first three foundation cells, or any passenger from the type
  with the crewed flag off.

**S7: stored resources (claim 10), only if a setup is found that fills a building's storage.**
- *Setup:* an own storing building whose own storage holds a known amount of one resource, in a
  second run a fractional amount (for example 10.5 units).
- *Procedure:* sell the building.
- *Observable:* the house's credits *M* before the sale and after the removal.
- *Expected:* *M* rises by the refund plus trunc(value × income factor × trunc(amount)); in the
  second run the half unit is not paid.
- *Falsified if:* *M* rises by the refund only, or the fraction is paid.

**S8: wall sale (claim 12).**
- *Setup:* an own wall section of a test wall type whose building has a cost of 100 (a named test
  file).
- *Procedure:* (a) sell it in sell mode. (b) Sell a section of a second wall test type whose building
  has the unsellable flag, by a sell-wall order (a test map's scripted click on the cell). (c) After
  (a), order a ground vehicle through the cell. (d) In a named test map, define two building types
  naming the same wall overlay, the first without a wall flag and with the unsellable flag, the
  later a wall type without the unsellable flag. Sell an own section of that overlay, then repeat
  in a fresh game with only the first type's unsellable flag off. Repeat both runs with the first
  type's wall flag on; each run uses one section.
- *Observable:* the cell's overlay and the credits; in (c) the vehicle's route.
- *Expected:* (a) the wall vanishes and the credits do not change; (b) the wall stays; (c) the vehicle
  passes through the cell; (d) the wall stays when the first type is unsellable and vanishes with
  no credit change when it is not, whatever that type's wall flag says.
- *Falsified if:* the credits rise in (a), the wall in (b) is removed, the vehicle is still blocked
  in (c), or any outcome in (d) differs.

**S9: repairable test (claim 14).**
- *Setup:* own buildings: one at full health, one damaged, one damaged of a test type with the
  click-repairable flag off, one damaged of a test type with the repairable flag off, and a damaged
  vehicle-like building.
- *Procedure:* hover in repair mode over each.
- *Observable:* the action offered.
- *Expected:* *repair switch* only over the damaged ordinary building.
- *Falsified if:* it is offered at full health, on either test type or on the vehicle-like building.

## Open questions

- Whether the occupants of a garrison leave before the sale survivors (claim 9; S5 part a), which
  the structures specs' account of the occupants' leaving can settle.
- The exact processing at which the build-down starts and the releases happen (claim 6, ±1 tick),
  and which links make a docked unit that keeps the sale waiting (claim 6).
- Whether any skirmish path fills a building's own storage, so that claim 10 can occur.
- In which order a building's storage lists its resource types, the order in which claim 10 pays
  them (F3).
- Whether the passengers of a unit that a sell order sells are killed or released (claim 11; only reachable if some input is found that creates such an order).
- How a docked unit relates to the glossary's contact and tether link (Terms, claim 6).
