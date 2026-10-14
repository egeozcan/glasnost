---
id: S-0408
title: Selling and building repair (refund, what a sale releases, selling vehicles and walls, which buildings can be sold or repaired)
topic: economy
revision: 16
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0130, Q-0131, Q-0225, Q-0282, Q-0284, Q-0440]
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
house state, on or off). S-0402 claims 19 to 21 define a miner's *slave group*,
its *master*, and slaves *inside* rather than on the map. In addition:

- **Sale.** The course of selling one building, from the first processing of its *selling*
  behaviour (S-0215) to its removal or its return to another behaviour (claims 4 to 10).
- **Docked unit.** A unit linked to a building for a service it gives, such as unloading at a
  refinery, repair on a depot or landing on a pad, from the moment the link is made until it is
  broken. It is a kind of contact with the building (glossary, *contact*); whether it is also
  tethered, and which services make such a link, belong to the specs of those services (open
  question on claim 6).
- **Build-down.** The part of a building's construction state in which it plays its build-up
  backwards while it is sold; S-0108 (claim 5) owns its timing. Its completion report is
  S-0215's *ready mark* (Terms), with the sale's completion tests given in claim 17.
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
- **Civilian-side recipient.** The first existing house in house-creation order whose country's
  side is the civilian side (Import mapping). In ordinary stock setup this is the neutral house.
  This is a side test, not an unconditional choice of the house named neutral.
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
   the purchase is refunded at the new factors. The refund is added directly to the seller's
   credit balance (S-0404 claim 4), not to house resource storage. Production's recorded start
   price and cancellation refunds (S-0404 claims 3 and 7) do not replace this sale formula.
   With a refund override the refund percentage and
   the cost-bonus product play no part. A free unit that cannot be placed pays this same refund
   with *r* = 1, as S-0402 claim 18 gives: *P* without an override, or the override form otherwise.
   - *Computer-controlled houses in the original.* The original applies the refund percentage only
     for a human-controlled house: a computer-controlled house receives *P* in full. In our engine
     every player house, AI houses included, receives the refund given above (glossary *AI house*); the
     original's full refund is recorded for the designers of our AI only. The neutral and
     special houses keep the original's rule (full *P*). Whether they sell automatically is
     governed by S-0314 claims 8 and 12 to 15: a passing skill-level gate is necessary,
     and claims 12 to 15 specify the further conditions and automatic-start outcomes.
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

5. **An upgraded building sells its top upgrade.** When a sale begins on a building with a
   positive upgrade count (S-0406 claim 12), the upgrade recorded at that count's position is
   sold: the house receives the refund of claim 1 for its type, that position is cleared, the
   house works out its power again (S-0405 claim 7), and the building goes to *guard*. Its health
   is unchanged. For contiguous upgrades this is the most recently added upgrade.
   - When the sold upgrade's type has no turret flag, the integer count *c* becomes *c* − 1,
     exactly, with no rounding. It drops by 1 even when that upgrade's level field raised it by
     *n* above 1, and even on the wall tower type when *c* exceeds its type's upgrade count.
     The count is not recomputed from occupied positions. A level-less upgrade can therefore
     be added again when the new count is below the type's limit. A level-field upgrade still
     needs count 0 (S-0406 claim 12), so selling a non-turret level-2 or level-3 upgrade does
     not by itself permit another level-field upgrade.
   - When the sold upgrade's type has the turret flag, the count becomes 0 instead, whatever
     its level field or the target type's limit. Only the sold position is cleared; lower
     recorded upgrades are not cleared. The next sale sells the building, since its count is 0.
     Their power no longer counts at count 0 (the count gate of S-0405 claims 1 and 2).
   - A count of 0 sells the building, even if positions still hold upgrades. A positive count
     whose position is empty is the bad-data departure in Edge cases; this can occur after
     selling a level-field upgrade without the turret flag.
   Selling an upgrade takes one creation draw of S-0202 (claim 10, every creation of a building
   object takes one), as the creation of a building object of the upgrade's type would, although
   no such object appears.
   - Backing: `measurable` (scenario S4, parts a to e; the creation draw is counted in part b
     by comparing the scenario random stream's state before and after the sale)
   - Confidence: high (the count changes, including the turret exception and the wall tower); medium (the refund, the creation draw)

6. **The course of a sale.** Subject to S-0215 claim 2's processing gates, the selling behaviour's processings are 1 tick apart (claim 16).
   The sale of a building without upgrades has the following observable states. A building that packs up goes through the same course with the differences of S-0415 (claim 9), among them which buildings release survivors, passengers and garrison (`quirks.packup_release_by_guard_spot`); the releases below are those of an ordinary sale.

   | Sale state or condition | Observable result |
   |---|---|
   | First processing | Every docked unit (for example, a harvester unloading at a refinery, a vehicle on a repair depot, or an aircraft on its pad) is told to leave; a vehicle held in the building's bunker is let out; burning-damage animations end. No other release occurs and build-down has not begun. |
   | A later processing while any unit remains docked | The sale waits. |
   | First processing at which no unit is docked | Absorbed passengers, garrisoned infantry and then sale survivors (claims 7 to 9) are released, in that order. The local player hears the sell sound for an own building, and build-down begins. The order is observable in the positions taken (an earlier unit can occupy a standing position a later unit would otherwise take) and in the order of release draws (claim 9 for the occupants). |
   | Build-down | Its range and stage delay are S-0108's (claim 5); its completion and removal tick are claim 17 below. |
   | Processing that finds build-down complete | In that same tick the house receives the building's refund (claim 1), then the value of its stored resources (claim 10), and the building is removed. There is no extra tick after the completion report. |

   For a slave-master building, the first processing also follows S-0402 claim 28's
   conditional resource-directed site rule. It recalls slaves and puts their group into
   packing only when a qualifying remembered resource destination yields a site; a failed
   search or a destination that does not qualify leaves the group unchanged by that rule.
   Starting selling or packing therefore does not by itself recall every slave. S-0402
   claim 27 owns the group updates that can continue while the building is selling,
   and claim 28's scenario H23 tests this conditional recall.

   For a refinery, being told to leave has the effects of S-0402 claim 11: an unloading
   harvester leaves its unloading state, scatters, starts *harvest* with its unpaid load,
   and loses contact; a harvester still in *enter* queues *guard* and scatters instead.
   S-0402 scenario H12 measures both cases.

   With no unit docked and uninterrupted processing, the release and build-down-start state is reached on the second processing, one tick after the first. The build-down restarts at stage 0 then, with a fresh stage delay; any completion report from before that restart is discarded.

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
     build-down, the conditional slave recall), medium (the step timing of the releases), low (which links count as docked and
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
    (F3). These payments go directly into the house's credit balance, not into its resource
    storage. The house's own storage (S-0404 claim 4; glossary, *available credits*) does not
    change. S-0404 owns available credits and spending; this claim owns only the sale payment,
    using S-0402's country income factor.
    In ordinary play a building's storage stays empty, because harvested resources are paid
    straight into the house's balance (S-0402, claim 12), so this rule matters only when something
    else fills a building's storage; no such path is known in a skirmish.
    - Backing: `unmeasured` (no known skirmish setup fills a building's storage; scenario S7 applies
      if one is found)
    - Confidence: medium (the formula, the truncation of the amount), low (whether any skirmish path
      reaches it)

### Slaves after a true sale

10a. **Removing a slave master by sale.** A true sale that removes a building master does not
    transfer its working slave group to a new vehicle. When the removed master's slaves are
    dealt with, every existing slave loses its master. Slaves still inside and off the map are
    destroyed and removed without being placed: they do not join the sale survivors or get a
    placement attempt. Each contributes a loss for its current house, subject to S-0414
    claim 21's victim exclusions, with no attacker credit. It receives S-0312 claim 29's
    common destruction notifications without an attacker, including the destroyed-by-anything
    raise when tagged, not the extra vehicle group. As in S-0312 claims 30 and 31,
    it plays no death sequence and produces none of its own debris, death-weapon or
    attached-bomb effects. Removal is not a separate killing hit.
    Slaves already on the map remain there and change owner to the civilian-side recipient
    (Terms), not to the seller or a nearby enemy. Their destination and target are cleared and
    they are queued to *guard*, ending their harvesting for that master. After that reset,
    each transfer requests the *cheer* sequence (S-0107 claim 21). This is a sequence request,
    not a guarantee of visible animation: S-0107 claim 6's start conditions still apply.
    The removed group makes
    no further replacements. No additional sale refund is paid for either kind of slave.

    If no civilian-side recipient exists, an on-map slave instead receives a hit whose integer
    damage amount is its current health, using the demolition warhead, with no attacker and
    without the ignore-defences marker. S-0302 claims 6 and 9 determine the resulting health.
    This is not an unconditional deletion. That fallback does not itself clear the slave's
    target or destination, queue *guard* or request *cheer*; the hit and later ordinary
    processing can still change them. Hidden slaves are still destroyed and removed.

    This happens as a consequence of removing the master, not at the earlier passenger,
    garrison and survivor release of claim 6. Selling only an upgrade (claim 5) does not remove
    the master and therefore does not free its slaves. Successful packing-up instead preserves
    the working group and makes the resulting vehicle its master (S-0415 claim 10); the
    retained off-map vehicle after failed placement is S-0415 claim 11's outcome, not a
    successful transfer of the working group. That vehicle does not inherit the old group's
    slaves. Removal of the old building ends that group with the hidden-slave removal and
    on-map recipient or no-recipient hit described above; the retained vehicle is not their
    new master. This shared slave-cleanup result does not make a failed pack-up a true sale:
    its vehicle, refund and quirk remain governed by S-0415 claim 11.
    Destruction during a sale remains a destruction case (S-0312),
    not this attacker-less sale case. The exact tick of slave cleanup relative to refund and
    foundation removal remains open.
    - Backing: `measurable` (scenario S13, including its failed-placement control) except
      tagged-hidden-slave notifications and their firing counts, which are `unmeasured`:
      no established data-only setup attaches a tag to a generated slave. S13's tagging
      proposal is conditional on finding a valid fixture, as in S-0011 claim 19.
    - Confidence: high for hidden removal, recipient selection, no replacement and no
      old-group transfer on failed placement; medium for the sale integration, guard reset
      and no-recipient hit, destruction notifications and losses, and the cheer request

### Selling other things

11. **A sell order on a vehicle or aircraft.** A vehicle or aircraft that a sell order reaches
    while a building occupies its cell (S-0213, claim 12; otherwise the order does nothing) is
    removed at once, with no animation and no survivors; its house receives its refund (claim 1) and
    its passengers are dealt with (open question). The local player hears the unit-sold announcement
    for an own unit. No input creates such an order (S-0213, claim 12; S-0214, claim 13), so in a skirmish no unit is ever sold. The removal, refund and announcement are specified here; how the unit's passengers are handled remains an open question and is not specified by this claim.
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

12a. **The cell targeted by a wall-sale click (Q-0284).** With sell mode on and an empty
    selection, a click that picks no individual object gives a sell-wall order for the cell under
    the cursor when that cell holds a wall overlay owned by the acting house and is neither
    shrouded nor fogged for that house. The cell is the cursor cell of S-0101 claims 9 and 10,
    without a foundation adjustment: S-0406 F2 is a placement formula, not a wall-sale formula.
    The wall's drawn pixels do not choose the cell. A click on wall art extending into a
    neighbouring cell therefore does not sell that wall unless the cursor cell itself holds it.
    A click within the wall's cell can sell it even where the wall art has no opaque pixel.
    A shrouded or fogged cell, a wall of another house, an ownerless wall, a non-wall overlay
    or an empty cell gives no wall-sale order. There is no search for a nearby wall.

    Input does not test the wall's builder or its unsellable flag: it can give an order whose
    application removes nothing under claim 12. Conversely claim 12 does not test shroud or fog:
    these are input conditions, not a second visibility test when the order applies.
    S-0214 claim 13 already states the own-wall and visibility conditions; this claim supplies
    the cell choice and the distinction between input and application.

    When the click picks an individual object, a *sell* action gives the object sell order,
    not a sell-wall order. In particular an own sellable ordinary building is sold as a building
    even if the cursor cell differs from its foundation. S-0214 owns object picking and the
    action on such an object; this claim does not turn a refused object sale into a wall sale.
    - Backing: `measurable` (scenario S11; S8 for application refusal)
    - Confidence: high (cell choice, wall conditions and lack of a visibility recheck), medium
      (object-versus-cell dispatch)

### Building repair

13. **Repair belongs to S-0314.** The repair order (S-0213, claim 13) switches a building's repair
    on and off; while it is on, the timing, the step and when repair switches off are S-0314's
    (claim 8) and the step cost is S-0314's (claim 5, with the adjusted cost of claim 2 of this spec
    for a building type). S-0314 claim 8 specifies the skill-level gate for automatic building
    repair and sale; S-0314 claims 12 to 15 specify the eligibility, credit split, automatic
    repair starts, sale attempts, random choices and house-wide pacing when that gate passes,
    including the distinctions between player and non-player houses.
    - Backing: `measurable` (S-0314 scenarios H5 and H13 to H15, with the unmeasured
      limitations stated in S-0314 claims 12 to 15 retained)
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

16. **The selling behaviour runs every tick.** Subject to S-0215 claim 2's processing gates,
    at every stage of a sale (the first processing, a wait
    for a docked unit, the start of the build-down, the wait for the build-down, the final
    processing), the *selling* behaviour is processed again on the next tick (a processing delay of 1
    in the terms of S-0215, claim 2), and its timing takes no draw. The processings of a sale
    therefore fall on consecutive ticks. (S-0215, claim 34, names the structures topic as the owner
    of this behaviour's processing but gives no delay.)
    - Backing: `measurable` (scenario S5, the tick of each stage)
    - Confidence: high

17. **Exact build-down completion and refund tick.** Let *D* be the tick at which the sale
    starts its build-down (claim 6), *n* its actual stage count and *d_b* its stage delay in ticks
    (S-0108 claim 5). The completion report uses the same ready mark as construction
    (S-0406 claim 17): a building turn whose stage advance lands on the range's last stage
    sets it. Passing the end and wrapping to the range's start does not set it. Selling's
    every-tick processing sees the mark in that same tick. In uninterrupted play, for
    *n* ≥ 2 and *d_b* ≥ 1, removal and payment occur at *D* + (*n* − 1) × *d_b*, on the
    tick the last stage is reached, not a tick later. The building need not remain on the
    map for a drawn frame showing that last stage.
    - A gate uses *n* = *G* + 1, with *G* S-0108's gate stage count. Reversing the displayed
      frames changes neither the advancing stage nor the refund tick. After a successful
      theater-flag refresh, use the whole sprite count and special delay of S-0108 claim 5,
      for gates too, not the ordinary half-count range.
    - A building whose type names a vehicle to pack into and which has no guard spot reports
      completion also when its advancing build-down reaches stage 23. For a positive-delay
      range of at least 2 stages, its completion tick is therefore
      *D* + min(*n* − 1, 23) × *d_b*. This applies even when S-0213's pack-up conditions
      make it sell as an ordinary building. A guard spot suppresses this early report.
      The guard spot includes automatic relocation's remembered old top-left cell, not just
      a manually ordered destination (S-0402 claim 28). It therefore suppresses stage-23
      completion too. Whether the completed sequence gives a refund or a vehicle is still
      S-0415 claim 9's rule.
    - A zero-delay build-down does not advance its stage but reports completion on the next
      eligible building turn. With uninterrupted processing it pays and removes at *D* + 1,
      not at *D*. This applies to a sellable type with a build-up sprite and zero delay.
    - No build-up sprite is a different case: claim 4 applies at the sell order's application,
      without any build-down. An ordinary type remains and pays nothing; a firestorm-wall
      type is removed in that application tick and pays nothing. The default one-stage,
      zero-delay construction range of such a type does not create a sale or a refund.
    These formulas assume no interrupted building turns, processing gates, intervening
    behaviour change or restarted animation. A completion report that cannot be acted on
    immediately is acted on at the next eligible selling processing; a killing hit instead
    stops the sale as claim 6 gives.
    - A positive-delay range with *n* ≤ 1 never reports completion: each advance runs past
      the end and wraps without landing on its last stage, or on stage 23. The building
      remains in *selling*, in build-down, with no sale removal or refund, although claim 6's
      releases have already occurred. The stalled sale itself neither changes nor freezes its
      health: without another effect its health stays unchanged; damage, repair and destruction
      retain their ordinary rules (claim 6; S-0314 claim 8). The stage-23 exception cannot rescue
      these ranges, even for a type naming a vehicle to pack into without a guard spot. A non-positive count uses delay 1 by S-0108 claim 5 and therefore falls
      in this case. A one-stage range with zero delay instead follows the zero-delay rule
      above. This is the sale effect of `quirks.single_stage_buildup_never_completes`
      (S-0406 claim 18; Edge cases below).
    - Backing: `measurable` (scenario S10; S3 parts c and d for no-sprite orders)
    - Confidence: high (same-tick report, restart, ordinary and early completion); medium
      (gate and theater ranges, zero-delay completion and ranges that never report)

### Sell and repair click messages

18. **No transient sell/repair refusal text (Q-0282).** A click offering *no-sell* or
    *no-repair* adds no battlefield text message. The refused action remains cursor feedback,
    not a timed notice. An accepted building-sale, wall-sale or repair-switch click likewise
    adds no text message of its own. A wall-sale order that applies but removes nothing
    (claim 12) does not add refusal text either. Audio, sale effects and the repair indicator
    remain governed by the existing claims and their owning specs; they do not imply a text
    notice. Thus these clicks have no message text, position, list capacity or expiry timer,
    and pausing has no sell/repair-message timer to stop. This does not suppress an independent
    message from a trigger or another game rule.
    - Backing: `measurable` (scenario S12)
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

**F4, sale completion tick** (claim 17). *D* is the build-down-start tick, not the sell-order
or first-processing tick. *n* and *d_b* are S-0108's actual construction range.

> *E* = *D* + (*n* − 1) × *d_b*, for *n* ≥ 2 and *d_b* ≥ 1.
>
> *E* = *D* + min(*n* − 1, 23) × *d_b* instead when the type names a vehicle to pack into
> and the building has no guard spot.
>
> *E* = *D* + 1 for a zero-delay build-down in uninterrupted play.

Integers, in ticks; exact multiplication and addition, no rounding. S-0108 owns the
floating-point calculation and rounding of *d_b*. Payment and removal share tick *E* for
an ordinary sale. S-0415 owns the result for packing up. No formula applies to a no-sprite
sell order (claim 4). With `quirks.single_stage_buildup_never_completes` on, no finite
*E* exists for a positive-delay build-down with *n* ≤ 1. With it off, *E* = *D* + *d_b*
for such a range: completion is reported on its first wrap (Edge cases).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Refund percentage, absent-key default | 0.5 | fraction | Claim 1 |
| Refund override, absent-key default | 0 | credits | Claim 1; 0 means no override |
| Selling processing delay | 1 | ticks | Claim 16 |
| Early completion stage for a packing-capable type without a guard spot | 23 | stage advances from build-down start | Claim 17 |
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
| *selling*, first processing | positive upgrade count | upgrade at that count's position sold; count reduced by 1 or reset to 0 for a turret upgrade; building to *guard* (claim 5); empty position follows the departure |
| *selling*, first processing | no upgrades | docked units told to leave; bunkered vehicle let out; burning animations end |
| *selling*, a later processing | a unit still docked | waits |
| *selling*, a later processing | nothing docked | absorbed passengers, garrison, sale survivors released (claims 7 to 9); sell sound; build-down begins |
| building, any behaviour, type with no build-up sprite | sell order, type with the firestorm-wall flag | removed at once; no refund, no survivors (claim 4) |
| building, any behaviour, type with no build-up sprite and no firestorm-wall flag | sell order | nothing (claim 4) |
| build-down | completion report, eligible selling processing on this turn | refund and stored resources credited; building removed on the report tick (claim 17) |
| build-down, positive-delay range of 1 stage or fewer | stage wraps, original quirk on | No completion report, removal or payment; sale remains in build-down (claim 17) |
| *selling*, any stage (the build-down included) | building destroyed | sale stops; no refund or further sale survivors; awaits its own next eligible turn, with a second destruction-survivor and smudge round on removal (claim 6; S-0312 claim 21) |
| *selling* | sell order | nothing changes, except that the local player hears the order sound again for an own building (claim 4) |
| vehicle or aircraft on a building | sell order (no input creates one) | refund; removed at once (claim 11) |
| wall cell | sell-wall order, conditions of claim 12 | wall removed; no credits |
| slave group | master removed by true sale | hidden slaves removed; on-map slaves freed to civilian-side recipient, or hit if none exists (claim 10a) |

Observable ordering: the survivors stand on the map before the refund arrives, and the refund
arrives in the same processing in which the building is removed, so a house's credits rise on the
tick the building disappears.

## Edge cases and quirks

- **Sale with an empty upgrade position** (bad-data departure, claim 5). The original fails
  when a positive count names an empty position. Our engine pays no refund, takes no creation
  draw, lowers the count by 1, leaves other positions and health unchanged, and returns to
  *guard*. The importer warns about a non-turret level-field upgrade above 1, which can leave
  this state after its sale, and about target limits above 3 (S-0406 claim 12). This also
  defines a sale while the count is above 3: no position exists there in our engine, so the
  same departure applies. This is not a quirk switch.

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
- **Single-stage build-down never completes** (claim 17). The shared
  `quirks.single_stage_buildup_never_completes` of S-0406 (default on, original) also
  governs selling: positive-delay ranges with 1 stage or fewer never report completion,
  so no refund or sale removal occurs. Off: such a range reports completion on the first
  building turn whose advance passes its end and wraps, giving payment and removal at
  *D* + *d_b* in uninterrupted play. Ranges with at least 2 stages and zero-delay ranges
  are unchanged. The importer warning of S-0406 claim 18 applies in both switch states.
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
| Rules, side list `[Sides]`, side named `Civilian`; country section, `Side` | civilian-side recipient of claim 10a: first existing house whose country belongs to that named side; name lookup is case-insensitive | side registry and country-to-side mapping, S-0004 claim 9 | no matching recipient uses claim 10a's hit fallback |
| Rules, type section, `Enslaves`; `[CombatDamage]`, `C4Warhead` | slave group and fallback demolition warhead of claim 10a | infantry type name (S-0402 claims 19 to 21); warhead type name (S-0303 Import mapping) | existing owning specs' defaults |
| No data key | hidden-slave destruction notifications and loss accounting, transferred-slave cheer request, and no cheer/reset on the no-recipient path (claim 10a) | engine behaviour; notification evaluation is S-0011 claim 19, loss exclusions S-0414 claim 21, sequence start S-0107 claim 6 | fixed |
| Map object tag field, `[Tags]`, `[Triggers]`, `[Events]`, `[Actions]` | hidden slave's destruction notifications (claim 10a), not a new sale-specific event | S-0011 claim 19 and Import mapping; S-0312 claim 29's common group | no matching tag: no tag raises |
| Art, infantry sequence section, `Cheer` | transferred slave's requested sequence (claim 10a) | sequence entry; S-0107 claims 1 and 22 own defaults and parsing | zero frame count when absent, so the direct request is refused (S-0107 claims 1 and 6) |
| Rules, `[General]`, `RefundPercent` | refund percentage *r* (claim 1) | decimal, a percent sign allowed (S-0003, claims 6 and 7), held as a double, narrowed to single (RZ₂₄) when used | 0.5; a layer without the key keeps the value in force |
| Rules, type section, `Soylent` | refund override *v* (claim 1) | integer, credits | 0 |
| Rules, upgrade building section, `Turret` | resets upgrade count to 0 on sale when yes; otherwise lowers it by 1 (claim 5) | boolean | no |
| Rules, upgrade building section, `PowersUpToLevel` | upgrade level field; addition and room test owned by S-0406 claim 12; non-turret values 2 or 3 warn about an empty-position sale | integer | −1 |
| Rules, target building section, `Upgrades` | type's upgrade count limit, owned by S-0406 claim 12; sale does not clamp to it | integer | 0 |
| Rules, `[General]`, `WallTower` | wall tower type, owned by S-0406; no special count reduction on sale | building type name | none |
| Rules, type section, `Cost` | cost field (claims 1, 2) | integer, credits | owned by the production specs |
| Rules, building section, `FreeUnit` | the free unit whose cost is deducted (claim 2) | vehicle type name | none |
| Rules, `[General]`, `PadAircraft` | pad aircraft list (claim 2) | list of aircraft type names | empty |
| Rules, `[General]`, `SeparateAircraft` | the separate-aircraft setting (claim 2) | boolean | owned by the aircraft specs |
| Rules, building section, `Unsellable` | unsellable flag (claims 3, 12); not a wall-click input gate (claim 12a) | boolean | no |
| Rules, overlay section, `Wall` | wall-overlay flag required by both input and application (claims 12, 12a) | boolean; S-0210 owns parsing | no |
| Rules, building section, `ToOverlay` | overlay used to identify its builder for application (claim 12); no builder lookup for input (claim 12a) | overlay type name; S-0406 owns resolution | none |
| Rules, building section, `FirestormWall` | firestorm-wall flag (claims 3, 4) | boolean | no |
| Rules, building section, `Repairable` | repairable flag (claim 14) | boolean | yes |
| Rules, building section, `ClickRepairable` | click-repairable flag (claim 14) | boolean | yes |
| Rules, `[General]`, `AlliedSurvivorDivisor`, `SovietSurvivorDivisor`, `ThirdSurvivorDivisor` | survivor divisors of S-0312 F5 (claim 7); S-0312's Import mapping owns them | integer | S-0312's absent-key defaults; keeps |
| Rules, type section, `Crewed` | building has a crew (claim 7, through S-0312 F5) | boolean | owned by S-0312 |
| Rules, `[General]`, `SurvivorRate` | nothing: the original reads it but no rule uses it | decimal | — |
| Rules, `[AudioVisual]`, `SellSound` | the sell sound (claims 6, 12) | sound name | sound specs |
| Rules, `[General]`, `BuildupTime` | build-down stage delay through S-0108 claim 5, including the non-positive-count delay of 1 tick (claim 17) | decimal, game minutes | S-0108 Import mapping owns the default and layering |
| Art, building section, `GateStages` | gate stage count, whose construction range has one more stage (claim 17) | integer | S-0108 Import mapping owns the default |
| Rules, building section, `UndeploysInto` | packing-capable type for the stage-23 completion test (claim 17), not a way to finish a positive-delay range of one stage or fewer | vehicle type name | none; S-0415 Import mapping owns resolution |

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
  cost 300 and the one added second of cost 500, both level-less and without a turret flag, with
  refund percentage absent (so 0.5), country cost
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

**S4 count variants (claim 5, Q-0131).**
- *Setup:* named test data on the same flat skirmish map, one distant bystander computer player,
  director and mods off apart from the named test data. Country factors 1 and no cost bonuses.
  (c) A target with limit 2 and two level-less non-turret upgrades.
  (d) An empty target with limit 3 and one non-turret level-3 upgrade.
  (e) A wall tower target with limit 1 and one non-turret level-3 upgrade.
  Repeat d and e with the upgrade's turret flag on. Also repeat c with only the top
  upgrade's turret flag on. Any speed, one run per variant.
- *Procedure:* sell one upgrade, then try a level-field upgrade and a level-less upgrade in
  separate fresh repetitions. Record count, occupied upgrade positions, refund and placement
  acceptance with a read-only recorder and scripted clicks. Do not issue a second sale on the
  empty position in the original; it risks failure.
- *Expected:* c has count 1 and room for a level-less replacement. Non-turret d and e have
  count 2, with all positions empty: level-field placement refuses in both, level-less placement
  succeeds only in d. Turret d and e have count 0 and accept a level-field replacement.
  The turret variant of c has count 0 but retains the lower recorded upgrade; its power
  contribution is 0, and a further sale sells the building rather than that upgrade.
  Each successful upgrade sale gives the claim 1 refund. On our engine only, a further sale
  on either empty count-2 position pays nothing and lowers the count to 1.
- *Falsified if:* the original's count or acceptance differs, especially a non-turret level-3
  sale restoring count 0, or a wall tower sale clamping the result to its type's limit.

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

**S10: exact removal timing (claim 17).**
- *Setup:* flat purpose-made skirmish map, one distant bystander computer player kept outside
  the observed area; director and mods off except named test data. Use crew-less own buildings
  without contacts, country cost factors 1 and no cost bonuses. Test ordinary ranges with
  invented stage counts 6 and 30 and positive delays 2 and 3 ticks, a gate with a known gate
  range, and theater-flag ordinary and gate types with 30 whole sprite frames after refresh
  (S-0108 B19's test-file setup). Include a build-up sprite with zero stage delay, both
  multi-stage and one-stage. Also use completed, map-placed buildings with named test
  sprites giving *n* = 1 at delay 2 and *n* = 0 at delay 1; map placement avoids the
  construction that never completes in S-0406 claim 18. Repeat these two types naming
  a vehicle to pack into, once without and once with a guard spot, checking that neither
  completion test makes a positive-delay range of 1 stage or fewer finish. For the
  early-report variants use a building that names a vehicle to pack into and a 30-stage range,
  once with no guard spot and once with a guard spot. S-0415 D6/D7 supply reachable pack-up
  inputs. Also use S-0402 claim 28's scenario H25 to obtain an automatic relocation
  destination, then sell during build-down without replacing that destination. Expect
  ordinary last-stage completion, not stage-23 completion, while that guard spot remains.
  Any game speed; repeat each variant once at another speed. One sale per variant
  and speed; void a run if the bystander affects the observed area or is defeated.
- *Procedure:* sell each ordinary building and pack up each packing variant with scripted
  input. With a read-only recorder, log each tick's build-down stage, behaviour, foundation
  occupancy, health and house credits, from order application through removal. Record *D*, the
  restart at stage 0, rather than assuming it equals the order tick. Repeat the no-sprite
  cases of S3 c/d, logging their order-application ticks too. For the positive-delay
  one-stage and zero-stage types, record for 100 ticks after *D*, including the first
  wrap and later wraps. On our engine only, repeat these with the shared single-stage
  quirk off. Disable self-healing and leave repair off in the stalled-sale runs. In separate
  repetitions after the first wrap, apply a nonlethal hit of known resolved damage, and then
  a killing hit, to distinguish a stalled sale from an invulnerable or health-frozen building.
- *Observable:* *D*, last stage seen before removal, removal tick, refund tick, and any
  packed vehicle's appearance. Record the ready mark and the actual range for gate and
  theater variants; for degenerate ranges, record wraps and the continued presence of
  the building and its unchanged credits.
- *Expected:* ordinary positive-delay refunds and removal at *D* + (*n* − 1) × *d_b*.
  6 stages at delay 2 give *D* + 10 ticks. The theater test range is 30 stages at delay 2,
  giving *D* + 58, for the gate too. The packing-capable 30-stage, delay-3 variant with no
  guard spot completes at *D* + 69; with a guard spot at *D* + 87. Its vehicle/refund result
  follows S-0415. Zero delay completes at *D* + 1. No-sprite ordinary orders do nothing.
  no-sprite firestorm-wall orders remove in the application tick with no refund. Tick
  intervals agree at both speeds. Positive-delay *n* = 1 and *n* = 0 never set the
  ready mark, stay in build-down through all 100 observed ticks and give no refund,
  including the packing-capable variants. Health stays unchanged in the undisturbed runs.
  the nonlethal hit lowers it by the resolved damage without completing the sale, and the
  killing hit stops the sale with no refund and removes it as claim 6 specifies.
  A one-stage zero-delay range instead completes
  at *D* + 1. On our engine with the shared quirk off, the positive-delay one-stage
  type completes at *D* + 2 and the zero-stage type at *D* + 1.
- *Falsified if:* payment/removal is one tick after the report, uses *n* rather than *n* − 1
  advances, uses a gate's sprite half-count or a theater sprite's half-count, ignores the
  stage-23 early report or applies it with a guard spot, pays at *D* for zero delay, or
  treats an ordinary no-sprite order as a zero-delay refunded sale, or a positive-delay
  one-stage or zero-stage range reports completion, pays or removes on any observed wrap,
  changes health merely because it wraps, or prevents the ordinary effects of either hit.

**S11: wall-sale input cell and cover (claim 12a, Q-0284).**
- *Setup:* a tiny flat skirmish map with isolated own and enemy stock walls, an ownerless
  map wall, an empty cell and a non-wall overlay. One distant bystander computer player,
  director and mods off, stock data, speed index 2, 800 × 600. The observed area is the wall
  test patch. Include a completed own sellable ordinary building away from the walls.
  For the cover variants, a purpose-made map trigger re-covers an own wall cell; use a
  separate named test-map fog setting for the fog variant. Keep each wall isolated so selling
  one does not change neighbouring wall art.
- *Procedure:* clear selection and enter sell mode. With scripted mouse input, click the
  centre and several transparent-art pixels inside each own wall cell, then pixels of its
  drawing that fall inside neighbouring cells according to S-0101 F7. Rebuild or restart
  after each accepted sale. Repeat on enemy, ownerless, empty and non-wall cells.
  Repeat with an own wall cell shrouded, and separately fogged, moving the cursor onto it
  after cover changes. In a further variant, issue the click while uncovered and have the
  test trigger cover the cell before the order applies; accept this variant only when the
  recorder establishes that timing, otherwise record it as inconclusive. Click the own
  building's image as an object-sale control. Three
  fresh repetitions per click and cover variant. Void a run if the bystander affects the
  patch or is defeated. If no map setup keeps an own wall covered, record that variant as
  inconclusive, not as a pass.
- *Observable:* input pixel, cursor cell, picked object if any, shroud/fog state, action
  offered, issued order kind and target, and which overlay or building disappears. Use
  tick-labelled captures and a read-only recorder; these observe, never set game state.
- *Expected:* with no picked object, every tested pixel selecting an uncovered own wall
  cell gives the wall-sale order for exactly that cell. Transparent wall art does not refuse
  it; wall art over an empty neighbouring cell gives no order. Enemy, ownerless, shrouded,
  fogged, empty and non-wall cells give no wall-sale order. The building control gives an
  object sell order and its ordinary sale. An order issued before cover changes still sells
  the wall when applied after that change. Credits for accepted wall sales remain unchanged
  under the original quirk (claim 12). S8's unsellable-builder variant gives an order but
  leaves the wall in place.
- *Falsified if:* a neighbouring wall is chosen by its drawn pixels, transparent art refuses
  an otherwise eligible wall cell, a covered or non-own wall receives a wall-sale order,
  the post-issue cover change alone prevents an already-issued wall sale, or the building
  control gives a wall-sale order instead of an object sale.

**S12: sell/repair clicks do not add text (claim 18, Q-0282).**
- *Setup:* tiny flat skirmish map, stock data, director and mods off, speed index 2,
  800 × 600, one distant bystander computer player kept outside the observed patch.
  Place own completed sellable buildings, an own damaged repairable building, a full-health
  building, enemy buildings, own walls and empty cells. For an application-refused wall
  sale only, use a named test-map override making its builder unsellable (S8 part b).
  No message-generating triggers or superweapon effects during the test.
- *Procedure:* clear selection. Hover and click each target in sell mode and then repair
  mode, recording the action offered. Include accepted building and wall sales, a repair
  switch, refused clicks on full-health and enemy buildings and empty cells, and the
  application-refused wall sale. Repeat each three times in fresh games or after restoring
  the test objects. Capture from before each click through 180 ticks afterward, and open
  and close the options menu in a separate repetition. Void a run if the bystander acts in
  the patch or is defeated.
- *Observable:* tick-labelled screen captures, cursor action, order issuance/application,
  sale or repair state, and any new battlefield text. Scripted input and a read-only
  recorder may establish the application-refused wall case.
- *Expected:* accepted actions retain their specified effects; every listed click adds zero
  text messages, including the wall order that leaves the wall in place. No refusal text
  appears before, during or after the pause.
- *Falsified if:* a listed click itself adds any battlefield text message. Record its event
  and duration rather than treating an independently generated message as a click notice.

**S13: slave fate after true sale (claim 10a).**
- *Setup:* flat purpose-made skirmish map, stock slave-miner types and a resource patch, one
  distant bystander computer player kept outside the patch, speed index 2, 800 × 600,
  director and mods off. Use named map overrides only for variants: a long slave reload delay
  to keep some slaves inside, and a deployed master with its packing conversion absent so
  sell mode gives a true sale. Keep other own buildings so the sale cannot defeat the player.
  A second variant keeps the packing conversion and uses a move click for successful packing-up.
- *Procedure:* let some slaves emerge and harvest while at least one is inside. Sell the
  non-converting master with normal scripted input. Use a read-only recorder to identify each
  existing slave, its position, owner, master, destination, target and behaviour, and the
  master's refund and removal ticks. Record each transferred slave's sequence request and
  accepted sequence, rather than inferring the request solely from visible cheering. Compare
  the seller's end-of-game losses with a matched no-sale control, counting only the existing
  hidden slaves separately from other deaths. Confirm no hidden-slave death effects.
  Observe through removal and for at least twice the
  imported regeneration delay afterward. Repeat three times. Repeat with all slaves inside,
  and with all on the map. Run the packing-up control with both hidden and emerged slaves.
- *Expected:* true sale destroys every hidden slave without emergence or placement, with
  one loss per non-excluded hidden slave for its current house and no attacker credit.
  No death sequence, debris, death weapon or attached bomb occurs
  for those hidden slaves. On-map slaves keep their positions, lose their master and
  become the neutral house's in this stock setup, with target and destination cleared,
  *guard* queued and *cheer* requested afterward. No replacements or slave
  refunds occur. The control preserves the existing group under the resulting vehicle.
  Record cleanup ticks rather than assuming they equal the foundation-removal tick.
- *Conditional tagging proposal, unmeasured:* no data-only fixture attaching a tag to a
  generated slave is established. If one is found, follow S-0011 claim 19 and scenario T25 B
  with a fresh repeating destroyed-by-anything tag and a calibrated visible counter action.
  Verify with the read-only recorder that exactly one hidden slave carries the fresh tag
  before the true sale; do not write into the running game to attach it. Use 5 valid runs.
  Expect the no-attacker common destruction group of S-0312 claim 29 before removal and
  two counter firings under S-0011 claim 19's latches, not one firing per slave. A different
  valid count falsifies that notification prediction. Until the fixture is established,
  this is a conditional proposal, not measurable backing; losses and disappearance alone
  do not confirm notifications. An absent tag or interference with the counter makes the
  notification test inconclusive, not a failure of the ordinary S13 test.
- *Failed-placement control:* use S-0415 claim 11's scenario D8 setup with a slave-master
  building, some hidden and some emerged slaves, and the ordinary civilian-side recipient.
  Repeat three times, recording the same slave identities and group links through building
  removal. Expect the old hidden slaves to be removed and the old on-map slaves to become
  the recipient's, not the retained off-map vehicle's slaves. That vehicle's existence and
  refund remain S-0415 claim 11's; do not confuse any new group it has with the old group.
  Inheritance of any old slave by that vehicle falsifies this control.
- *Recipient variants:* in named map data, give the human player's country the civilian side
  while retaining the normal neutral house. The earlier-created player is then the recipient.
  In a separate variant ensure no existing country's house belongs to that side; the on-map
  slaves then receive health-sized demolition hits, with resolved damage following S-0302,
  rather than an ownership transfer. That path requests no cheer and does not itself
  reset target or destination or queue guard. Keep ordinary defence factors and record
  health changes and sequence requests.
  Verify the house-side setup with the recorder before accepting either variant.
- *Falsified if:* hidden slaves appear as survivors, the seller keeps on-map slaves in the stock
  run, a slave continues harvesting for the removed master, a replacement appears, an extra
  slave refund is paid, the recipient variants choose a later qualifying house or ignore the
  no-recipient hit, the hidden losses are missing, a transfer
  omits its cheer request, the no-recipient path requests cheer, or the successful conversion
  frees rather than transfers the group.
  Void and repeat a run if the bystander affects the patch or is defeated.

## Open questions

- A valid data-only fixture attaching a tag to a generated hidden slave, needed before
  S13's conditional notification proposal becomes measurable (S-0011 claim 19).
- The exact slave-cleanup tick relative to a true sale's refund and foundation removal
  (claim 10a; S13 records it).
- Whether the occupants of a garrison leave before the sale survivors (claim 9; S5 part a), which
  the structures specs' account of the occupants' leaving can settle.
- Which links make a docked unit that keeps the sale waiting (claim 6).
- Whether any skirmish path fills a building's own storage, so that claim 10 can occur.
- In which order a building's storage lists its resource types, the order in which claim 10 pays
  them (F3).
- Whether the passengers of a unit that a sell order sells are killed or released (claim 11; only reachable if some input is found that creates such an order).
- How a docked unit relates to the glossary's contact and tether link (Terms, claim 6).
