---
id: S-0404
title: Production queue and payment (production lines, price, pay-as-you-go steps, waiting for money, hold, cancel and refunds, queue limit, completion)
topic: production
revision: 26
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-09
measurements: []
requests: [Q-0122, Q-0190, Q-0120, Q-0195, Q-0226, Q-2841]
supersedes: []
---

# S-0404: Production queue and payment (production lines, price, pay-as-you-go steps, waiting for money, hold, cancel and refunds, queue limit, completion)

## Summary

A house's production lines: the price of an item, how it is paid step by step, waiting for money,
hold, cancel and refunds, the queue, completion, and how factory buildings appearing, leaving or
losing power affect the lines. Step length is S-0403, clicks are S-0411 and the credits display is
S-0416; factory unloading and qualified post-contact destinations are S-0417. Skirmish only,
network games and campaigns are out of scope.

**Paired revisions.** S-0407 separates the issue of claim 9's cancellation orders
at the recheck from their application at the tick's end, exempts held-item resume from the
issue-time limit test and keeps claim 10's application-time limit test on queued starts
(S-0407, claims 13 and 16). S-0411 claim 9 specifies cancellation with refund at the
place-order application boundary for a busy or failed unit release, without retry; no eligible
factory leaves the item done (claims 7 and 13 below). Its claim 14 states the continued steps
and payments while a factory is being sold or queued to be sold. The relevant factory-eligibility
tests exclude such a building (Terms); claim 13's hand-off to an idle twin keeps its expressly
stated exception. S-0402 claim 18 reads a produced building's recorded start price, and
S-0414 claim 15 includes off-map production objects in destruction hits.
S-0411 claim 15 owns full-tab suppression and later admission, including the capacity
check before removals, no refill at that same recheck, and S-0407 claim 17's simultaneous
admission order; S-0004 retains sidebar sorting and capacity ownership. S-0417 claim 14 owns
contact-place count, infantry exit-candidate acceptance and edge-aircraft nearby destinations.
Release queues unloading rather than starting it synchronously. Claim 10 already distinguishes
upgrade placement from ordinary placement, as required by S-0406 claim 12.
S-0202 claim 10 and S-0203 claim 8 now agree with claim 11's
causing-action rule and P11 B/C. Payment priority alone does not determine that assignment.
Their proposed assignment measurements are not measured results. S-0202 claim 15's
middle-event creation draws do not change production-start precedence.
S-0701 claim 24 distinguishes application-time queue refusal's game-wide refusal sound
from refused building placement's announcement only; claims 3 and 7 below retain that distinction.
Q-0193's killed-item production outcomes remain open (Open question 3); ordinary
completion and refund rules do not settle that case. Its request metadata must be reopened
at hand-off rather than treating S-0414's count-only answer as closure of all its parts.

## Terms

Glossary terms are used as defined there (*house*, *country*, *type*, *field*, *credits*, *tick*,
*building*, *infantry*, *vehicle*, *aircraft*, *AI house*, *human-controlled*, *computer-controlled*,
*local player*, *difficulty row*, *difficulty skill block*, *quirk switch*, *importer warning*,
*absent-key default*). From S-0203: *production line*, *turn pass*. From S-0213: *order*, *primary
factory* (Terms; this spec gives the rule, claim 7). From S-0403: *item*, *line kind*, *defence*,
*ordinary building*, *progress*, *done*, *step period* *R*, *factory building of a kind*. From S-0205:
RZ₅₃, RZ₂₄, trunc and the integer quotient ÷. From S-0408: *adjusted cost* *C*′ (claim 2). In
addition:

- **Creation sample** (measurement only). The low 16 bits of the raw scenario-stream value
  consumed by an item's creation, an integer from 0 to 65535. A read-only recorder can associate
  this sample with the created item; this term prescribes no storage layout for our engine.
- **Single, double.** A *single* is a single-precision floating-point number, a *double* a
  double-precision one; "narrowed to single precision" means converted from a double to a single.
- **Owner countries** (of a type). The countries that the type's data names as its owners.

- **Dock types** (of an aircraft type). The building types that an aircraft type docks at, in the
  sense of the glossary's *refinery, dock types* (S-0408, Terms). Claim 2 uses the first of them.
- **Price** *P*. What a house pays for one item of a type, in credits (claim 2).
- **Balance owed** *B*. Per line, the part of the item's price not yet paid, in credits.
- **Credit balance** *Y*. The house's own stock of credits (glossary *credit balance*). Harvests add
  to it (S-0402, claim 12), the payments of a step take from it and cancel refunds add to it (this
  spec), and a sale adds to it (S-0408, claim 1).
- **House storage** and its value *V*. The original's house has a store of resources of its own,
  separate from every building's storage (S-0408, claim 10) and from a weeder's store. *V* is the
  value of the amounts in it, in credits, before the income factor. This spec owns the store
  (claim 4); in a skirmish it is always empty, so *V* = 0.
- **Income factor** *M*. The income factor of the house's country, a single, as in S-0402 (Terms and
  claim 12).
- **Available credits** *A*. What a house can spend at a given moment: its credit balance plus the
  value of its house storage (claim 4). In a skirmish *A* = *Y*.
- **Queue.** Per line, the ordered list of types ordered after the item, waiting to be started. A
  queued type has no object and costs nothing until it starts.
- **Player hold.** A line state set by a hold order: no progress until a resume.
- **Automatic hold.** A line state set by the check of claim 8 on a running or waiting item when
  none of the house's factory buildings that could make it is switched on, and lifted by that check
  only. A line is on at most one hold at a time (claim 8): the automatic hold and the player hold
  exclude each other.
- **Could make, could release.** A factory building of the item's kind *could make* a type when it
  is on the map, is neither being sold nor queued to be sold (S-0407, Terms: its current or its
  queued behaviour is selling), its type's owner countries share at least one country with the
  item type's owner countries, it is of the matching kind for vehicles (a naval factory building for
  a naval vehicle, a non-naval one for every other vehicle, infantry or aircraft item) and the
  build-limit answer with the allowance for an item in production is allowed (S-0407, claim 9).
  This is not a full tech-tree test: prerequisites, tech level, country restrictions, stolen tech
  and the superweapon setting are not judged here. *Could make without the tech-tree rule*
  is the same with that build-limit condition left out too. Thus the latter test judges only the
  factory conditions, not any part of the tech tree. Each rule below says whether it also needs the
  building to be switched on and, for aircraft, to have a *free dock*: an aircraft factory
  building has a free dock when at least one of its docking places is empty or when no aircraft is
  docked with it at all. Dock occupancy matters only for aircraft items. A factory building *could
  release* a done unit when it is switched on, could make the unit's type without the tech-tree
  rule and, for an aircraft, has a free dock. For an aircraft only, when no building of the
  house could release it by that definition, every switched-on building that could make it without
  the tech-tree rule counts as one that could release it (the fallback of claim 7, which gives the
  choice among such buildings).
- **Switched on, switched off.** A building's power switch state: the player's power toggle
  and some trigger actions switch a building off or on, and a map can place a building
  switched off; the power spec S-0405 owns the power switch. A power deficit does not switch a building
  off: it only slows production (S-0403, claim 5). Only building types that drain power, or that
  need power to work, can be switched off.
- **Unloading, guard.** The behaviours of that name (S-0215, Terms) as a building's current
  behaviour; S-0417 owns factory unloading.
- **Exit cell, exit displacement.** The cell selected by claim 13's release search, and a building
  type's displacement of the point where a released unit appears, in leptons (Import mapping).
  An ordinary infantry unit with a rally point outside its own cell initially walks to the exit
  cell, then to the rally point; without a rally point these release rules give no destination
  (S-0417 claim 10).
  The ground vehicle's exit apron cell is a distinct term owned by S-0417.
- **Barracks door kind.** One of three building-type flags (Import mapping) that give an infantry
  factory a door cell tried first as its exit cell (claim 13).
- **Cloning building.** A building whose type has the cloning flag (claim 14).
- **Waiting for money.** A running line whose last step was refused because the step's payment was
  more than the house's available credits (claim 4).
- **Build order, hold order, cancel order, cancel-all order, place order.** The production orders a
  house gives. Each names a type (and so a line); a place order of a building also names a cell.
  They are orders in the sense of S-0213: given in one tick and applied at the order boundary that
  follows it (S-0213, claim 1; S-0203, claim 7). S-0411 says which clicks give them.

## Behaviour

1. **Six independent lines per house.** Each house has at most one production line of each line
   kind: infantry, ground vehicles, naval vehicles, aircraft, ordinary buildings and defences. A type
   is built only by the line of its kind (S-0403, Terms). The six lines run in parallel and do not
   wait for one another; each has its own item, progress, balance owed and queue, and each pays for
   its own steps. A line exists from the first build order applied for its kind until it has neither
   an item nor a queued type; the next build order creates it again. Lines take part in the tick as
   S-0203 (claim 8) states. The observable priority among a house's lines is their creation order: when two lines can each make a step in the same tick but the house can pay for only one, the earlier-created line is paid and the other waits for money. A line created again is treated as newer than every line that already existed.
   - Backing: `measurable` (scenario P1)
   - Confidence: high (six lines and their independence); medium (the update order among lines)

2. **Price.** The price of an item of a type with cost field *C* (S-0403, claim 2) for a house is

   > *P* = trunc(ρ₅₃(ρ₅₃(*C* × *b*) × *a*))

   where *a* is the country's cost factor for the type's kind (infantry, vehicles, aircraft,
   ordinary buildings, defences; a single) and *b* is the product of the cost bonuses for that kind of
   every building the house owns whose type grants cost bonuses (a single, 1 when it owns none,
   formed one building at a time with each partial product narrowed to single precision, ρ₂₄).
   Here ρ₅₃ is RZ₅₃ with `quirks.chop_rounding` on and RN₅₃ with it off; ρ₂₄ is
   correspondingly RZ₂₄ or RN₂₄ (round-to-nearest, ties to even). The final conversion to
   integer credits truncates toward zero in either setting. The bonus product is worked out
   again whenever the house gains or loses such a building.

   A building type whose cost field pays for units that come with it is priced in parts. Its *own
   part* uses, instead of *C*, the type's adjusted cost *C*′, which S-0408 owns (claim 2 and F2: the
   cost field less the halved cost fields of the first two pad aircraft types when the type is the
   first dock type of the first pad aircraft type and the separate-aircraft flag is off, less the
   cost field of the free unit type; raised to 0 only for a type with a free unit). Its price is then

   > *P* = trunc(ρ₅₃(ρ₅₃(*C*′ × *b*) × *a*)) + (*P*₁ + *P*₂) ÷ 2 + *P*ₑ

   limited below by 0 only when the type names a free unit type. The pad term (*P*₁ + *P*₂) ÷ 2
   (integer quotient, toward zero) is present exactly when S-0408's pad deduction applies to the
   type, *P*₁ and *P*₂ being the prices of the first two pad aircraft types (each by this claim,
   with the aircraft factors), and *P*ₑ is the price of the free unit type for the same house (0 when
   there is none). So the units included in a building's cost are priced with the factors of their
   own kind, while the refund of a sale (S-0408) works from the cost fields instead of these prices.
   The difficulty row's cost factor and the country's general cost factor have no effect on any price,
   so in our engine the optional per-player handicap does not change prices.
   - Backing: `measurable` (scenario P2)
   - Confidence: high (the formula for non-building types and ordinary buildings without extra
     parts; no effect of the row and general factors; the parts of a building price, as the
     original forms a building type's price); medium (the pad term's rounding)

3. **Starting an item.** A build order is applied only if the house has a factory building that
   could make the type without the tech-tree rule (Terms), is switched on and, for an aircraft,
   has a free dock, and the build-limit answer with the production allowance is allowed
   (S-0407, claim 9). These conditions are judged when the order is applied, before starting,
   queuing or resuming anything. The order need not pass a fresh full tech-tree test: offer
   legality is judged when it is issued (S-0407, claim 13; claim 10 below). A stale prerequisite
   refusal therefore does not itself prevent a start or resume while the type remains offered.
   The same application test holds for a build order requesting resume of a held item: exemption
   from the issue-time limit test does not exempt it from this production allowance. Otherwise
   the order is ignored, also as a request to queue: for example no aircraft can be ordered,
   even into the queue, while every switched-on aircraft factory building of the house has all
   its docking places taken. For an eligible order, the result depends on the line state:

   | Line state before the order | Ordered type and result |
   |---|---|
   | Idle: no item and empty queue | The item starts at once at progress 0, with balance owed *B* = *P* at that moment; the step period of S-0403 (claims 1 and 8) starts. No money is needed to start. A house with 0 credits can start an item and then wait for money at its first step (claim 4). |
   | Player hold, with an item of the ordered type | The order resumes the line (claim 5) and adds nothing. |
   | Done item of the ordered type | The item is left as it is, and nothing is added. |
   | Any other line state | The type is added at the end of the queue unless the queue already holds the game-wide maximum number of queued types (Constants), in which case the order is refused. A type is also refused when its build limit forbids it at queue time; S-0407 (claim 16) owns that test, including how it counts the item in production, its rule for limits of 0 or below, and the separate house-wide pad-capacity test for an aircraft type that must return to a dock (S-0501 claim 30). This capacity test does not replace the free-dock factory-eligibility test above. |

   A queue request refused at application by the queue limit or S-0407 claim 16's
   queue-time build-limit test requests the game-wide refusal sound only for the local
   player's house (S-0701 claim 24). This is not the busy-building-line click's
   unable-to-comply announcement (S-0411 claim 12), nor feedback for an order ignored
   by the earlier factory-eligibility test.

   An aircraft order also fails the factory-eligibility test while every switched-on aircraft factory building of the house has all its docking places taken; it is then ignored even as a request to queue. An automatic hold means there is no switched-on factory building that could make the held item's type. An order for that type is therefore ignored and cannot resume the line until such a building is switched on; claim 8 then lifts the automatic hold by itself. An order for another type of the same line is judged by that other type's factory eligibility and, if eligible, is queued under the table's rule. A line on the forced-start hold of claim 10 likewise cannot be resumed while every factory building that could make its type is switched off. The item's object exists from the start but is not on the map. Its price at the start is the initial balance owed and is retained for a produced building's free-unit condition (S-0402 claim 18; Edge cases, *Recorded price*), also when `quirks.refund_at_current_price` is on. Being off the map does not protect its object from defeat or surrender destruction hits: S-0414 claim 15 owns their eligibility and order. An item created before the remaining factory can receive its hit before that factory's loss cancels production; an item already deleted by factory-loss cancellation receives no later hit.
   - Backing: `measurable` (scenario P3, parts a to k, P7 for the done item, S-0411 S4 b2
     for resume at and above the infantry limit, S-0402 H13 for the start-price condition and
     S-0414 V11 part c for off-map destruction hits)
   - Confidence: high (start, queue, limit, the free-dock condition for aircraft, and the
     build-limit-only application test and the retained start price); medium (the other factory
     conditions, application-time refusal sound and destruction before factory-loss cancellation)

4. **Pay as you go.** Each progress step is paid when it is made. When the line's progress goes from *k* − 1 to *k* (*k* = 1 … 54), let *B*ₖ₋₁ be the balance owed before the step. For *k* = 1 … 53, the payment is

   > *x*ₖ = min(*B*ₖ₋₁, *B*ₖ₋₁ ÷ (54 − *k*)).

   and *B*ₖ = *B*ₖ₋₁ − *x*ₖ. At step 54, *x*₅₄ = *B*₅₃. All amounts are signed integers in credits; ÷ is integer division rounded toward zero. The step's payment is refused if *x*ₖ > *A*, using a signed comparison; otherwise it is paid in full. For *k* = 53 the divisor is 1, so the 53rd step pays everything still owed and the 54th step pays 0. The payments add up to exactly *P*. Example: *P* = 1000 pays 18 credits at steps 1 to 7,
   19 at steps 8 to 53, and 0 at step 54. If *x*ₖ is more than the house's
   available credits *A* at that moment (a signed comparison: *A* and *x*ₖ can be negative, see
   claim 6 and Edge cases), the step is not made: progress stays at *k* − 1, nothing is
   paid, and the line is waiting for money. Its next attempt comes *R* ticks later (S-0403, State and
   ordering), at the same phase, whether or not money arrived in between; there is no partial
   payment. Since the lines are updated after the turn pass (S-0203, claim 8), credits that a
   harvester delivers during the turn pass of the tick of an attempt count for that attempt. A paid
   step reduces the credit balance by its payment. Nothing puts resources into the house storage in
   a skirmish (harvests and a sale's stored resources are paid straight into the balance: S-0402,
   claim 12 and S-0408, claim 10), so *V* is always 0, the payment always fits in the balance and
   *A* = *Y*. Our engine needs no house storage and keeps the credit balance only. In the original

   > *A* = trunc(ρ₅₃(ρ₅₃(*V* × *M*) + *Y*))

   where *Y* is the credit balance, *V* the value of the house storage and *M* the income factor
   (S-0402, claim 12). With *V* = 0 this is *Y* exactly. The original also lets a payment that is
   more than the balance empty the balance and take the rest from the house storage. That is not
   specified here: which resource is spent first, whether whole units are removed and whether a
   surplus from the last unit returns to the balance are unknown, and a skirmish never reaches it.
   A later spec that adds a way to fill the house storage must measure it and state it.
   - Backing: `measurable` (scenario P4) for the payment, the refusal, the retry period and
     *A* = *Y*; `unmeasured` for the formula for *A* with filled house storage and for spending
     from it (no skirmish can fill it, so the setup cannot observe either)
   - Confidence: high (the payment formula, the refusal without partial payment, the retry period,
     *A* = *Y* in a skirmish); medium (the formula for *A* with a filled house storage); low (the
     spending order from a filled house storage)

5. **Hold and resume.** A hold order puts a running line (also one waiting for money) on a player
   hold when it is applied: no further steps, nothing refunded, progress and balance owed kept. It acts
   on the line of the type it names, whatever type that line's item is. A step
   that fell due in the tick of the order has already been made (S-0203, claim 8). A hold order to a
   line already on a hold of either kind, or with a done item, does nothing; in particular a line on
   an automatic hold stays on an automatic hold, which claim 8 lifts by itself. A build order for the held item's type
   resumes it (claim 3): the step period is worked out again and the next step comes a full *R* ticks
   later (S-0403, claim 8). The queue does not advance while the line is held.
   - Backing: `measurable` (scenario P5)
   - Confidence: high

6. **Cancel and refund.** The result depends on the order and the line state:

   | Order | Queued copies of *X*? | Current item is *X*? | Result |
   |---|---|---|---|
   | Cancel | Yes | Yes or no | Remove the first queued copy of *X* only; nothing else happens and no money changes hands. |
   | Cancel | No | Yes, whether running, waiting for money, on either hold, or done | Cancel the item. The house receives *P*′ − *B* credits, where *P*′ is the item's price at the moment of the cancel (claim 2), or, with `quirks.refund_at_current_price` off, its price at the start (Edge cases), and *B* is its balance owed. Delete the item's object. Start the first queued type, if any, at once as a new item (claim 3, with the start conditions of the forced start in claim 10); if the queue is empty, the line ceases to exist. |
   | Cancel | No | No | Nothing happens. |
   | Cancel-all | Any | Yes or no | Remove every queued copy of *X*. If the current item is *X*, cancel it with the refund above and continue the line as described below; otherwise no item is cancelled. |
   | Cancel-all | No copies | No | Nothing happens. |

   For cancel-all, when an item is cancelled, the first queued type other than *X*, if any, starts in its place; no copy of *X* starts. With an empty remaining queue, the line ceases to exist.

   A cancel-all order for *X* leaves no copy of *X* in the line: every queued copy of *X* is removed,
   and an item of type *X* is cancelled with the refund above. When the item is cancelled, the line
   then goes on as after the cancel above, the first queued type other than *X* taking the place of
   the first queued type, so no copy of *X* starts in its place. The refund is added to the credit
   balance with no upper limit. When the price has not changed since the start, the refund equals
   what was paid, so cancelling at any progress costs nothing. The refund is negative when the price
   of the moment is below the balance owed (a cost bonus gained after the start); a negative refund
   is taken from the credit balance with no lower limit, so the balance, and with it *A*, can drop
   below 0. While *A* is below 0 every step is refused, also one that would pay 0 (a payment of 0 is
   more than *A*), until credits bring *A* back to the step's payment or above; the retry period
   of claim 4 applies as for any refusal. No switch: this follows from `quirks.refund_at_current_price`
   and from the signed arithmetic, and it needs a bonus price below the owed balance, so it arises
   only with data that grants such a bonus.
   - Backing: `measurable` (scenario P6, parts a to h)
   - Confidence: high (refund and cancel cases); medium (the negative refund and the refusal at a
     negative *A*, read from the signed addition and comparison)

7. **Completion.** At step 54 the item is done and the line makes no more steps.
   Acquisition order means the order in which buildings join the house's currently owned
   buildings, by placement into play or ownership change to that house. Capture or transfer
   ranks a building after every building the new owner already held, regardless of creation
   order. Losing and later reacquiring it gives it a new, later rank; buildings not lost keep
   their relative order. Within one tick, acquisitions rank in the order their events take
   effect, not by object number. An unchanged-owner assignment does not count as reacquisition.
   This meaning also applies to the acquisition orders in claims 13 and 14 and F1.
   - **Units** (infantry, vehicles, aircraft). A place order without a cell releases the unit from a
     factory building of its kind that is switched on and could make it with the tech-tree condition
     left out (Terms), and, for an aircraft, has a free dock: among those, the one marked primary if
     there is one, and otherwise the last in the order in which the house acquired its buildings.
     An aircraft for which no such building has a free dock uses the same choice with the free-dock
     condition left out: among the switched-on buildings that could make it without the tech-tree
     rule (Terms, *could release*).
     If no such building exists, the order does nothing and the item stays done, blocking the line.
     Otherwise the chosen building releases the unit by claim 13, which gives one of three
     results: *released*, *busy* or *failed*. When it is released, the line's next queued type
     starts at once (claim 3, with the start conditions of the forced start in claim 10); with an
     empty queue the line ceases to exist. When it is busy or failed, the item is cancelled with
     the refund of claim 6 and the line goes on as after that cancel, in the same order boundary:
     a done unit is never kept waiting for an exit. (In the original a busy result keeps the item
     only for a building's own production line, which only the original's computer-controlled
     houses have, claim 12; an AI house of our engine has none.) S-0411 (claim 9) gives the place
     order that the interface issues by itself when a unit is done, in our engine for every player
     house; its left click may cancel a done unit instead (S-0411, claim 4).
   - **Buildings and defences.** A done building waits until a place order with a cell puts it on the
     map. Placement rules are S-0406. When placement is refused by its cell tests, the item stays
     done and, for the local player's house, requests the cannot-deploy-here announcement,
     without an interface sound or the game-wide refusal sound (S-0701 claim 24).
     An order ignored because no eligible factory exists also leaves the item done,
     but requests neither that announcement nor a refusal sound (S-0406 claim 1 and S-0411
     claim 10). When placement succeeds, the line ceases to exist (buildings never queue,
     claim 10).
   - Backing: `measurable` (scenario P7; S-0406 PL1 for ignored building placement and
     S-0411 S8 for refused placement)
   - Confidence: high (done at step 54, next queued item starts after the release, buildings wait for
     placement); medium (the choice of the releasing building, and the cancel with refund on a busy
     or failed release with no retry)

8. **Factory buildings appearing, leaving or losing power.** Whenever one of a house's factory
   buildings of a kind is placed, leaves the map, changes owner, or is switched off or on (S-0405),
   that house's line of the kind is checked at once (for a building that changes owner, the lines of
   both houses; for a vehicle factory building, the naval-vehicle line when the building is naval
   and the ground-vehicle line otherwise). A line's automatic-hold state is looked at only by this check, so it is never
   put on an automatic hold between two such events. The check judges "could make" (Terms) with
   docks ignored throughout and with the power switch ignored except where switched-on buildings
   are named. The check removes from the queue every type that no factory building of the house could make. It then treats the current item according to this state table. Here, as in Terms, “could make” retains only the build-limit answer with the production allowance,
   not the full tech-tree conditions. Docks are ignored throughout the check, and the power switch
   is ignored except where a switched-on building is expressly required. A prerequisite loss alone
   therefore does not make this check cancel an item or remove a queued type; claim 9 supplies the
   automatic cancellation orders when a recheck removes its offer.

   | Item state at the check | Factory condition | Result |
   |---|---|---|
   | Any item state | No factory building of the house could make the item | Cancel the item with the refund of claim 6. Continue as after a cancel, using the queue after the unsupported types have been removed; the first remaining queued type is the one claim 10's forced-start rule may start. |
   | Running or waiting for money | At least one factory building could make the item, but none is switched on | Put the item on an automatic hold. An item started by this same check in place of a cancelled item is not judged again by this check; only a later check can hold it (claim 10). |
   | Automatic hold | A switched-on factory building that could make the item exists | Lift the hold. A full step period passes before the next step, as for a resume (S-0403, claim 8). |
   | Automatic hold | No switched-on factory building that could make the item exists | Leave the automatic hold in place. |
   | Player hold, including the forced-start hold, or done | Either condition | Leave the item as it is. The check neither adds an automatic hold nor lifts a player hold, so the line never carries two holds. In particular, only a build order for the forced-start item's type resumes that player hold; claim 3 refuses that order while no switched-on factory building could make the type. |

   For factory buildings of buildings, the check covers only the ordinary-building line, not the
   defence line (`quirks.defence_line_partial_factory_check`, Edge cases). Separately, when a factory
   building is removed for good (destroyed, or sold), every line of its kind, the defence line
   included, whose item no other factory building could make without the tech-tree rule, whatever its
   power switch and docks, is cancelled with the refund of claim 6 and the first queued type starts
   as after a cancel (claim 10). This rule looks only at the item, not at the queue, and works in
   addition to the check above.

   A factory building that is being sold or queued to be sold counts in no test of this claim
   (Terms). The start of a sale, and a sale order that is only queued, are not events of this
   check, so they neither hold nor cancel anything by themselves: a running item whose only factory
   building is being sold keeps making and paying its steps (S-0411, claim 14) until the building
   leaves the map, when the removal rule above cancels it. A check run in the meantime by another
   listed event (another factory building of the kind placed, switched off or on, or changing
   owner) judges the line without the selling building, so it cancels the item when that building
   was the only one that could make it, and removes the queued types that only it could make.
   **Two factories with one sale queued (Q-0190).** Let X and Y be the house's only two
   factories of a unit line, both initially switched on, with Y queued to be sold. Assume
   unchanged build limits that allow the item. When X is switched off, X still could make the
   item with power ignored, but Y does not count: the running item goes on automatic hold at
   that factory check. When X is destroyed instead, no other eligible factory remains, so the
   permanent-removal rule cancels and refunds the item in the removal tick, without waiting
   for sidebar cancellation orders. The leaving-map check also finds no eligible factory if
   the item still exists then; an already-cancelled item is not refunded twice. In either
   case that check removes every queued type that only Y could make. Sale queueing alone
   removes no queued types before such a check. These exclusions and queue outcomes were
   already specified by revision 4's Terms and this claim; this passage makes the three
   requested consequences explicit.
   - Backing: `measurable` (scenario P8, including run 10 for the build-limit-only test, run 11
     for factories being sold and run 12 for Q-0190's queued-sale cases)
   - Confidence: medium (the conditions and the defence-line exception); high (that a player hold is
     never lifted by the check and that the factory check omits the full tech-tree conditions)

9. **Items whose type leaves the taken-up set.** When a type leaves its house's taken-up set at
   a recheck (S-0407, claims 1 and 12), the original's interface removes its build icon and issues,
   in the local player's name, a cancel order when the type is the line's item and a cancel-all
   order when a line of that kind exists (S-0411, claim 2). The item is refunded (claim 6) and its
   queued copies removed when those orders are applied. Both orders are issued in the house update
   of that recheck and applied at the end of the same tick (S-0203, claim 7). A prerequisite lost
   in the turn pass of tick *n* therefore causes the orders in tick *n*; a listed recheck event
   caused by an order applied at the end of tick *n* causes them in tick *n* + 1. A condition that
   fails without a recheck event causes no automatic orders until a later recheck removes the type:
   for example the start of a sale does not itself trigger a recheck (S-0407, claim 12).
   For a type suppressed by a full sidebar tab, S-0411 claim 15 owns the consequences,
   including offer membership, order legality, later admission and the absence of sidebar
   cancel and cancel-all orders when a still-suppressed type loses availability. Independent
   factory-check cancellation remains claim 8's rule, and cancellation, refund and
   order-application rules remain claims 3 and 6 here.
   This happens only for the local player in the original; the original's computer-controlled
   houses keep building such items. In our engine it is a rule for every player house: the same
   cancel and cancel-all orders, with the conditions above, are issued automatically for every
   player house, AI houses included (AI-house ruling, glossary *AI house*). The non-player houses
   never build.
   - Backing: `measurable` (scenario P9) for the local player's behaviour; full-tab consequences
     inherit S-0411 claim 15's backing (S10). `unmeasured` for the
     original's computer-controlled houses, which the measurement setup keeps out of every observed
     area (glossary, *bystander computer player*) and which are not a rule of our engine
   - Confidence: medium (the conditions and the two orders); high (the house update of the
     recheck and the order boundary; S-0407, claim 12 and S-0203, claim 7)

10. **Order legality for every house.** In our engine a production order is legal for a house only
    when every rule that applies to it holds, judged when the order is issued, for AI houses as for
    persons (S-0213, claim 3). The order's legality, judged when issued, is determined by the applicable row:

    | Order kind | Legality condition |
    |---|---|
    | Starting or queuing build order for a building or defence | The line has no item (in progress, held, or done), and the type is offered to the house and not at its build limit. These are the state rule of S-0411 (claims 4 to 6) and the offer rule of S-0407 (claim 13). An offer can remain stale until the next recheck; `quirks.stale_offer_orderable` and its alternative are owned by S-0407, claim 13. |
    | Starting or queuing build order for another kind | The type is offered to the house and not at its build limit, under S-0407 (claim 13). An offer can remain stale until the next recheck; `quirks.stale_offer_orderable` and its alternative are owned by S-0407, claim 13. |
    | Build order resuming the held item of the ordered type | The order is exempt from the busy-line and build-limit tests, but not from the offer rule (S-0411, claim 4; S-0407, claim 13). |
    | Place order with a cell | For an ordinary building or defence, adjacency and explored ground hold (S-0406, claim 6, with S-0411, claim 10). For an upgrade order, the upgrade-specific legality conditions of S-0406 claim 12 apply: adjacency does not, but explored ground still does. Applying the order does not judge adjacency or explored ground again; for an upgrade, its target ownership, compatibility and room conditions are checked again at application under S-0406 claim 12. |

    A legal build order is then applied or refused by claim 3 (the factory conditions, the
    build-limit answer with the production allowance, the queue limit and S-0407 claim 16's
    queue-time test, including S-0501 claim 30's house-wide pad capacity for dock-bound aircraft).
    A free dock at an eligible factory and remaining house-wide queue capacity are distinct
    requirements, not one free slot per factory. This claim only lists the rules; each stays owned by the spec
    named. The original applies the building-line case differently (Edge cases); its interface never
    gives it.
    A type taken from the queue to start after a cancel or a release is started even when
    the house's only factory buildings that could make it are switched off or (for aircraft) have no
    free dock, as long as one could make it. This forced start also tests the build-limit answer
    with the production allowance (S-0407, claim 9), but not the full tech tree or the queue-time
    limit of S-0407 claim 16 again. In the original such an item then starts on a player
    hold when the house's available credits are at least *P* ÷ 54 (integer quotient) at the start, and
    running (and waiting for money at its first step) otherwise; being running, it is put on an
    automatic hold only by the next check of claim 8. Our engine reproduces this by default
    (`quirks.forced_start_player_hold`, Edge cases); such an item is resumed by a build order for its
    type, as any player hold (claim 5), which an AI house has to give like a person. When the first queued type cannot start even so (no factory building could
    make it), it is dropped from the queue (Edge cases).
    - Backing: `measurable` (scenario P3 for the refusal of a second building through the sidebar,
      scenario P10 for the forced start and the dropped queue head); upgrade target rechecks
      inherit S-0406 claim 12's backing (PL10 part j, conditional on establishing its event timing).
    - Confidence: high (the legality rule, a rule of our engine); medium (the forced-start hold,
      its affordability test and upgrade target rechecks); low (dropping a queued type that cannot
      start, Edge cases)

11. **Random draws: one creation draw per item started.** Starting an item creates its object, and
    that creation takes one raw value from the scenario random stream (S-0202, claim 10, which
    leaves the timing of production's creations to the production specs). So one draw is taken at
    every start of an item, in the part of the tick in which the start is applied: when a build
    order starts an item on an idle line (claim 3), when a cancel or a release starts the first
    queued type (claims 6, 7 and 10) and when claim 8 does. A queued type that is only added to the
    queue, or dropped from it, creates nothing and takes no draw (claim 10). Nothing else of a
    line's life takes a scenario-stream draw: no step, payment, refusal, retry, hold, resume, cancel, refund,
    completion or deletion of the item's object, and no line's own update. The order among lines in claim 1 determines payment priority, not a separate
    batch of creation draws. Presentation sounds and announcements can take
    unsynchronised-stream draws under S-0701; those are not production creation draws.
    Starts receive their creation draws at their causing action:
    for several production orders in one tick, successive starts receive successive values in
    order of application (S-0213 claim 3). In particular, two queued replacements caused by
    automatic cancellation orders receive values in S-0411 claim 13's tab/icon cancellation
    order, even when the later-cancelled line was created earlier. A replacement caused by
    a factory event in the turn pass receives its draw before starts caused by orders at that
    tick's end (S-0203 claim 1). Draws taken by intervening actions remain between the starts.
    No values are reserved for a line ahead of its causing action. Across houses, starts
    likewise inherit the precedence of their causing actions: turn-pass events follow S-0203's
    turn order, and order-boundary starts in our engine follow S-0213 claim 3's house creation
    order and each house's production-order issue order. The between-house order-boundary rule
    is our engine's deterministic design, not a measured rule of the original's computer
    production (claim 12). Reaching step 54 itself starts no replacement and takes no creation
    draw: the later place order causes release and any queued replacement start (claim 7).
    Thus simultaneous completion is not a separate batch of starts in the line-update part.
    The release of a done unit is claim 13's (its draws are listed there), and the
    placement of a done building belongs to the placement specs.
    - Backing: `measurable` (P11 A for isolated counts, P11 B for same-boundary assignment
      and P11 C for turn-pass creation before order-boundary creation). The between-house
      order-boundary extension is `unmeasured`: it is a design rule, not an oracle outcome.
      The no-draw assertion for paths outside P11 A's cases is `unmeasured` here: that
      scenario does not exercise every failed-start and deletion path.
      These are proposed assignment tests, not results confirmed by isolated counts.
    - Confidence: high (the creation draw at a start, S-0202); medium (assignment at the causing
      action and the no-draw exclusions)

12. **The original's computer players.** The original's computer-controlled houses produce without
    orders, through their factory buildings directly, with their own pacing. That is behaviour of the
    original's computer player and not a rule of our engine: an AI house produces only through the
    orders of this spec (glossary, *AI house*). AI designers may reproduce the original's pacing
    with ordinary orders.
    - Backing: `unmeasured` (it describes the original's computer player, which the measurement
      setup deliberately keeps out of every observed area (bystander computer player, glossary) and
      which is not a rule of our engine; the line is a note for AI designers)
    - Confidence: medium

13. **Releasing a unit from a factory building (Q-0120).** Let *K* be the building that claim 7
    chose, (*x*₀, *y*₀) the top-left cell of its foundation, *W* × *H* its foundation (S-0406,
    Terms) and *c* its reference point (the foundation centre, S-0204 claim 4). A *docking place*
    is one of *K*'s contact places, as in Terms (*free dock*). The result is *released*, *busy* or
    *failed* (claim 7). Directions are S-0204's F3; "rounded to *n* steps" is S-0204's F4. A
    facing given as a value *v* of 0 … 255 is *v* × 256 facing units (S-0204, claim 10). Unless a
    row says otherwise, an appearing unit that cannot be put on the map gives *failed*.

    | Kind of *K* | Result and where the unit appears |
    |---|---|
    | Ground vehicle factory (vehicle factory flag, not naval) | When *K*'s current behaviour is *unloading* (it is still letting out an earlier vehicle), the release passes to the first other building of the house, in acquisition order, of the same building type whose current behaviour is guard; that building releases the unit by this row and its result is the result. Its power switch and claim 7's conditions are not judged. With no such building the result is *busy*. Otherwise the unit appears inside *K* at its *exit point*: *K*'s position (the centre of its top-left cell) plus the type's exit displacement (three integers in leptons), or *c* when the displacement is (0, 0, 0). It faces 16384 (map east). The unit is put there without an entry test of that cell: neither *K* itself, listed in the cell as a building of its foundation, nor any other occupant of the cell, of any house and moving or not, nor its terrain or overlay keeps it out, so the cell never makes this row give *failed*. Whether an exit point off the map gives *failed* by the table's general sentence is an Open question. The unit then shares the cell with whatever stands there. *K*'s max(*d*, 1) contact places are numbered from 1, *d* being the imported docking-place count (S-0417 claim 14), and the new unit's contact with *K* takes the free place with the lowest number. When no place is free, the unit that holds place 1 loses its contact and tether link with *K* at this release, and the new unit takes that place (medium for a *K* with one place, low for a *K* with more than one: backing below). *K* then queues unloading under S-0417 claims 2 to 8, without starting it synchronously; the unit saves *K*'s rally point (S-0213, claim 20) as its guard spot, with the ordinary arrival-report destination governed by S-0417 claim 8. For jumpjet ascent, S-0417 claim 15 instead governs contact and tether release and the subsequent destination; the factory does not wait for touchdown. Result *released*. |
    | Naval vehicle factory | When *K* has no unit in contact, *K* queues unloading under S-0417 claim 9, which commands no exit movement on the released ship. With a rally point, take the 8-direction *e* = the direction from *c* to the rally point rounded to 8 steps, and step from *c*'s cell in direction *e* (S-0204, claim 9) until the first cell not covered by *K*. That cell is the exit cell when it is water (S-0211), holds no unit, infantry, aircraft or building, and is usable (S-0009, claim 4). Otherwise, or with no rally point, the exit cell is the nearby-cell search of S-0212 (claim 25) around *c*'s cell for the unit's movement kind. The unit appears at the exit cell's centre facing 16384 and, with a rally point, gets a move order to it. Result *released*. |
    | Infantry factory, and cloning building (claim 14) | *K* has max(*d*, 1) contact places, where *d* is the imported docking-place count (S-0417 claim 14). When none is free, the result is *busy*. Otherwise the exit cell is the first candidate below that passes S-0417 claim 14's legal-cell and free-entry-result test; with none, *failed*. Candidates: first, for a type with a barracks door kind, its door cell: (*x*₀ + 1, *y*₀ + 2) for the first kind, (*x*₀ + 2, *y*₀ + 2) for the second, (*x*₀ + 2, *y*₀ + 1) for the third, tried in that order when a type has several kinds; then the cells of the foundation's exit list (Constants), each (*x*₀ + *a*, *y*₀ + *b*), in list order. No other cell is tried: a foundation with an empty list (0 × 0) and no legal door candidate gives *failed*, and no cell toward the rally point is preferred. The unit appears at the centre of the cell obtained by adjusting each exit-cell coordinate one cell toward the foundation rectangle: an x coordinate at or above *x*₀ + *W* is reduced by 1, one below *x*₀ is raised by 1, and otherwise it is unchanged; y uses *y*₀ and *H* in the same way. These are integer cell operations with no rounding. For a nonempty foundation this is the neighbouring foundation cell; the same adjustment defines a door release for the 0 × 0 word without presuming a foundation cell exists. When the exit cell is the door cell of a door kind the type has, the type's exit displacement is added. Its facing is the direction from *c* to the exit cell's centre, rounded to 256 steps. It saves *K*'s rally point as guard spot; contact, tether and subsequent destinations follow S-0417 claim 10, with jumpjet ascent reports governed by S-0417 claim 15. With a rally point outside the unit's own cell, ordinary infantry walks first to the exit cell and then to the rally point. Infantry of a jumpjet type and types marked as teleporting head straight to the rally point. With no rally point, or one in the unit's own cell, the release gives no destination; any later walk-out remains S-0417 Open question 3. Result *released*. |
    | Aircraft factory | When *K* has a docking place that is empty or already in contact with this aircraft, the aircraft appears docked at the position of claim 15, facing the game-wide pose direction (Import mapping) × 256 units. With a rally point, a type that need not return to a dock gets a move order to it. That move order is the release's own: the release gives the aircraft the rally point as its destination and queues *move*, and it is not a behaviour order of S-0213 claim 5 (S-0501 claim 48 (d)). Result *released*. Otherwise a type that must return to a dock (S-0407, Import mapping) gives *failed*. Any other aircraft enters from the edge of the local area (S-0009, claim 4: (*L*ₓ, *L*ᵧ, *L*w, *L*h) and map width *M*). Let *Q* = (*L*ₓ + *L*ᵧ + 1, *M* − *L*ₓ + *L*ᵧ), and let (*b*ₓ, *b*ᵧ) be *c*'s cell. Define the start cell *S* by these cases:

| Condition | Start cell *S* |
|---|---|
| (*b*ₓ − *Q*ₓ) − (*b*ᵧ − *Q*ᵧ) > *L*w | (*Q*ₓ − 1 + *L*w, *Q*ᵧ − *L*w) |
| (*b*ₓ − *Q*ₓ) − (*b*ᵧ − *Q*ᵧ) ≤ *L*w | (*Q*ₓ − 1, *Q*ᵧ) |

One ranged draw *r* over 0 … *L*ₕ from the scenario stream gives the cell (*S*ₓ + *r*, *S*ᵧ + *r*), just outside the left or the right edge of the local area. The aircraft appears at that cell's centre facing 0 and its *move* is forced: to the rally point, otherwise to the nearby destination of S-0417 claim 14, otherwise with no destination. Result *released*. |

    Our engine reproduces each foundation's exit list exactly (`quirks.foundation_exit_lists`,
    Edge cases). Every supported imported foundation word has a present exit list, including
    the empty list for 0 × 0. Exhausting that list does not trigger a perimeter search.
    Unsupported words follow S-0108 claim 8's foundation selection and fallback; they do not
    define arbitrary dimensions or select an absent-list perimeter alternative.
    Ground and naval unloading, doors, exit movement and qualified post-contact
    destinations belong to S-0417. Its claim 10 supplies the infantry initial destination.
    A qualifying jumpjet ascent ends factory contact and tether under S-0417 claim 15,
    without waiting for destination arrival or touchdown, even without a rally point.
    No initial destination for no-rally infantry does not imply retention after that report.
    Only subsequent no-rally walk-out remains open there (Open question 3). Claim 15 below
    supplies aircraft pad positions. S-0501 claim 30 owns the house-wide pad-bound aircraft
    queue-capacity test; it is not a release test and does not replace this claim's docking-place
    check or claim 3's free-dock factory eligibility. S-0417 claim 14 supplies the infantry exit-search entry
    test, contact-place count and nearby destination of an edge-entering aircraft. Its nearby
    search takes no draw (S-0212 claim 25). Only initial-placement draws remain unresolved
    there (S-0417 Open question 8), apart from the no-rally walk-out question above. The
    requests list records the answered aircraft docking-position part, not closure of all
    Q-0120 parts. Nothing of a release is retried: claim 7 cancels a busy or failed item at once.

    **Random draws of a release.** The edge entry of an aircraft takes one ranged draw over
    0 … *L*h (none when *L*h = 0, S-0202 claim 4). Each copy made by claim 14 is a newly created
    object and takes its creation draw (S-0202, claim 10). The nearby-cell search takes no draw
    (S-0212 claim 25). Whether initial placement takes further draws remains S-0417 Open
    question 8. S-0417 claim 12 owns
    unloading and post-contact draw consequences, not initial placement's draws. The
    original's computer-controlled houses take one more draw in some releases, through their own
    build planning (claim 12); our engine has no such draw.
    - Backing: `measurable` (scenario P12, parts a to h; the aircraft edge draw with the
      random-state recorder for the edge-row value, not its isolated draw count). The exact
      edge-entry draw count is `unmeasured` here: unresolved placement draws prevent P12 h from
      separating that count from the rest of the release. Repeated identical exit-list entries are
      `unmeasured`: rejecting a candidate and immediately trying that same cell again has no
      distinct observable in P12's static blocking setup. The computer-only extra release draw is
      `unmeasured`: P12 uses only the human house, not a predicted computer build-planning action.
      Unloading, initial infantry destinations and post-contact rules inherit S-0417's backing
      (claims 10 to 12, scenarios U10 and U12); jumpjet ascent release and destination effects
      inherit S-0417 claim 15's backing and limitations (U14). The contact count, exit-candidate
      entry test and nearby-aircraft destination inherit claim 14's backing and U13 scenarios. That
      the aircraft factory row's rally move is the release's own assignment and not a behaviour
      order has S-0501 claim 48's backing: `unmeasured`, since no part of S-0501 scenario AC30 sets
      up a type that need not return to a dock released with a rally point. That the ground vehicle
      factory row makes no entry test is `measurable` for *K* itself through P12 part a, whose
      second run puts the unit at the foundation centre, a cell of *K*, and for an earlier unit of
      *K* still standing at the exit point through S-0309 scenario A19 part k, which also shows, for
      a *K* with one contact place, the contact and tether link moving to the new unit. For every
      other occupant it is `unmeasured`: no scenario part here has one in the exit point's cell at a
      release. For the terrain and overlay of that cell it is `unmeasured` too: no scenario part
      here puts the exit point on terrain or overlay that a vehicle may not enter. The places of a
      factory with more than one, the free place with the lowest number taken and place 1 taken from
      its holder when none is free, are `unmeasured`: no scenario part here gives a vehicle factory
      more than one contact place.
    - Confidence: medium (the results, the hand-off between vehicle factories, the appearance
      points and facings, the exit lists, the aircraft edge entry and its draw); low (the door
      cells); contact count, entry test and nearby destination inherit S-0417 claim 14's confidence.
      The jumpjet ascent consequences inherit S-0417 claim 15's confidence. Medium for the aircraft
      factory row's rally move being the release's own assignment and not a behaviour order, as
      S-0501 claim 48 rates that routing. Medium that the ground vehicle factory row makes no entry
      test, for *K* itself, for every occupant and for the cell's terrain and overlay, and that a
      release with *K*'s one place held takes that place; low for which place is taken when *K*
      has more than one, free or all held.

14. **Cloning buildings copy released infantry.** When an infantry factory building that is not
    itself a cloning building releases a unit by claim 13 and has found its exit cell, each
    cloning building the house owns, in the order the house acquired them, gets a new infantry
    unit of the same type for its owner and releases it by claim 13's infantry row. If the original release has found an exit cell, each eligible cloning building receives its copy even when the original unit does not appear. Copies receive their creation draws in cloning-building acquisition order. Each earlier copy's release outcome and any draws taken while releasing that copy precede the next copy's creation draw; claim 13 leaves initial-placement counts open. Copy location or blocked exits do not change which cloning building has precedence, but any placement draws can change the stream value that the next copy receives. Copies are released before the original unit's own initial placement, so that placement's draws do not precede copy creation. These rules answer Q-0226's copy-order questions without assuming a zero-draw initial placement.
    No copy is made when the original's release is busy or finds no exit cell. A copy whose release
    is busy or failed does not appear; nothing is paid or refunded for copies. A cloning building
    never causes copies itself.
    - Backing: `measurable` (P12 g for copies, blocked copies and creation-draw assignment).
      `unmeasured` for copies when the original fails to appear after finding its exit:
      no setup here separates a successful exit search from that later placement failure.
      Exact grouping of unknown initial-placement draws is `unmeasured` here: creation samples
      distinguish copy precedence but do not identify the source of each intervening value.
      The ordering rule does not assign a guessed count to those values.
    - Confidence: medium (the copies and their order); low (what becomes of a copy that cannot be
      released)

15. **Aircraft pad position (Q-0120).** For an ordinary aircraft factory, let *c* be its
    foundation centre in world leptons, including its world height. A type marked as a pad
    building or a repair building uses its imported docking displacements; without either flag
    the aircraft's released position is *c*. This rule covers ordinary aircraft factories,
    not combinations with resource-unloading or tank-bunker flags.

    With either flag, the released aircraft's position is *c* + *d*ᵢ, component by component
    in the map x, y and world-height axes, where *d*ᵢ is the docking displacement of its
    contact place. The places are numbered from 0 in their imported order. An aircraft already
    in contact keeps that place; otherwise it takes the first empty place, which is the place a
    new contact takes, so the position follows the contact place. If the imported
    docking-place count is 1, the sole displacement is used regardless of contact index.
    With a count of 0, or a contact index outside the imported displacement count, use *c*
    without a displacement. A new release with several places uses its newly established
    contact place for its final position in the release tick; it does not stay at the
    undisplaced centre merely because it was not yet in contact before release.

    All coordinates and displacements are signed 32-bit integers in leptons. Addition wraps into
    that range as S-0205 claim 1 specifies; there is no floating-point calculation or rounding. The
    displacement is not rotated by the aircraft's pose facing, and the exit displacement used for
    vehicles or infantry does not contribute. Choosing this position takes no random draw. Claim 13
    still governs failed placement and full docks, and S-0417 claim 11 governs contact and tether
    timing. This claim governs the release only: the height component is applied only at a factory
    release, which puts the aircraft at this position, height included, and S-0501 claim 48 governs
    what follows. An aircraft that docks by landing is moved toward the docking position
    horizontally and rests at the landing height (S-0501 claim 24). An aircraft that lands over the
    pad without a contact (S-0501 claim 24) is moved toward a docking position that does not depend
    on whether the pad has a free contact place: with one docking place, that place's docking
    position, and otherwise the pad's foundation centre with no displacement.
    - Backing: `measurable` (scenario P13) for ordinary release positions and tested import
      branches. The zero-count and out-of-count contact-index fallbacks are `unmeasured`:
      no P13 release establishes an aircraft contact in either state, so a failed release
      or an edge entry cannot confirm which pad position that contact would select.
      The no-draw assertion for position selection alone is `unmeasured`: P13 cannot
      distinguish such a draw from unresolved initial-placement draws during the same release.
      That docking by landing moves the aircraft horizontally and leaves the height component
      unapplied has S-0501 claim 24's backing: `measurable` (S-0501 scenarios AC30 part k (1) and
      AC14). The docking position of an aircraft that lands without a contact has S-0501 claim 24's
      backing: `unmeasured` (no set-up there lands a waiting aircraft over its pad).
    - Confidence: high (position formula, flags, count branches and no position-selection draw),
      medium (contact-place selection and final position within the release tick); medium, until
      S-0501 scenarios AC30 part k (1) and AC14 run, for docking by landing's horizontal move and
      unapplied height, as S-0501 claim 24 rates them; medium for the docking position of an
      aircraft that lands without a contact, as S-0501 claim 24 rates it

## Formulas

Credit amounts are signed 32-bit integers in credits (S-0205 claim 1), with integer
arithmetic wrapping as that claim states. Factors and intermediate products are floating point.
ρ₅₃ denotes RZ₅₃ with `quirks.chop_rounding` on and RN₅₃ with it off.
ρ₂₄ denotes RZ₂₄ with it on and RN₂₄ with it off. RN rounds to nearest, ties to even.
RZ rounds toward zero (S-0205, Formulas). trunc converts to integer credits toward zero
in both settings; its out-of-range result is owned by S-0205 claim 7.
The original computes F1 and F3 in floating point (doubles, with the bonus product held as a
single); the integer quotient ÷ of F2 is exact integer arithmetic.

- **F1 (price).** *P* = trunc(ρ₅₃(ρ₅₃(*C* × *b*) × *a*)); for a building type with included units,
  claim 2's form with *C*′ (S-0408, F2) and the added prices, limited below by 0 only when the type
  names a free unit type. *b* = ρ₂₄(… ρ₂₄(ρ₂₄(1 × *β*₁) × *β*₂) … × *β*ⱼ) over the house's bonus
  buildings in the order the house acquired them, *β* their bonuses for the kind (singles). The
  first product 1 × *β*₁ is exact and a product of two singles does not depend on their order, so
  the order of acquisition is observable only when the house owns three or more bonus buildings
  whose bonuses for the kind are not all equal: the partial products are then rounded differently,
  and the price can differ.
- **F2 (step payment).** For *k* = 1 … 53: *x*ₖ = min(*B*ₖ₋₁, *B*ₖ₋₁ ÷ (54 − *k*)); *x*₅₄ = *B*₅₃ = 0,
  *B*ₖ = *B*ₖ₋₁ − *x*ₖ; *B*₀ = *P*. The step is refused when *x*ₖ > *A*, a signed comparison. (With *B* < 0, Edge cases, the minimum
  makes the first step pay the whole negative balance.)
- **F3 (available credits, original).** *A* = trunc(ρ₅₃(ρ₅₃(*V* × *M*) + *Y*)), *Y* the credit
  balance, *V* the value of the house storage (0 in a skirmish), *M* the income factor; so *A* = *Y*
  in a skirmish.
- **F4 (refund).** refund = *P*′ − *B*, *P*′ the current price by F1; it can be negative, and the
  credit balance has no lower limit (claim 6). With `quirks.refund_at_current_price` off, refund =
  amount paid so far = *P* − *B*, *P* the price at the start.

Worked payments (illustrative): *P* = 100 pays 1 credit at steps 1 to 6, 2 credits at steps 7 to 53
and 0 at step 54; *P* = 10 pays 0 at steps 1 to 43 and 1 credit at each of steps 44 to 53.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Lines per house | 6 | lines | claim 1 |
| Steps per item | 54 | steps | S-0403 |
| Last paying step | 53 | step | claim 4 |
| Absent-key default, maximum queued types | 5 | types per line | claim 3; counts queued types, not the item in production |
| Facing of an appearing vehicle (ground and naval factories) | 16384 | facing units | map east; claim 13 |
| Facing of an aircraft entering from the edge | 0 | facing units | claim 13 |
| Absent-key default, pose direction | 0 | 1/256 turn | claim 13 |

**Exit lists** (claim 13). S-0314 claim 25 owns repair-departure selection using these same
lists; claim 13 here governs newly produced units, not departure after repair.
Per foundation word (S-0406, Import mapping; W × H, *a* along map *x*,
*b* along map *y*, from the foundation's top-left cell), in the order tried:

| Foundation | Exit list (*a*, *b*) |
|---|---|
| 1 × 1 | (0, 1), (−1, 1), (1, 1), (−1, 0), (1, 0), (0, −1), (−1, −1), (1, −1) |
| 2 × 1 | (0, 1), (1, 1), (−1, 1), (2, 1), (−1, 0), (2, 0), (0, −1), (1, −1), (−1, −1), (2, −1) |
| 1 × 2 | (0, 2), (1, 2), (−1, 2), (−1, 1), (1, 1), (−1, 0), (1, 0), (0, −1), (−1, −1), (1, −1) |
| 2 × 2 | (0, 2), (1, 2), (−1, 2), (2, 2), (−1, 1), (2, 1), (−1, 0), (2, 0), (0, −1), (1, −1), (−1, −1), (2, −1) |
| 2 × 3 | (0, 3), (1, 3), (−1, 3), (2, 3), (−1, 2), (2, 2), (−1, 1), (2, 1), (−1, 0), (2, 0), (0, −1), (1, −1), (−1, −1), (2, −1) |
| 3 × 2 | (0, 2), (1, 2), (2, 2), (−1, 2), (3, 2), (−1, 1), (3, 1), (−1, 0), (3, 0), (0, −1), (1, −1), (2, −1), (−1, −1), (3, −1) |
| 3 × 3 | (0, 3), (1, 3), (2, 3), (−1, 3), (3, 3), (−1, 2), (3, 2), (−1, 1), (3, 1), (−1, 0), (3, 0), (0, −1), (1, −1), (2, −1), (−1, −1), (3, −1) |
| 3 × 5 | (−1, −1), (0, −1), (1, −1), (2, −1), (3, −1), (−1, 0), (3, 0), (−1, 1), (3, 1), (−1, 2), (3, 2), (−1, 3), (3, 3), (−1, 4), (3, 4), (−1, 5), (0, 5), (1, 5), (2, 5), (3, 5) |
| 4 × 2 | (0, 2), (1, 2), (2, 2), (3, 2), (−1, 2), (4, 2), (−1, 1), (4, 1), (−1, 0), (4, 0), (0, −1), (1, −1), (2, −1), (3, −1), (−1, −1), (4, −1) |
| 3 × 3 refinery word | (0, 0) |
| 1 × 3 | (−1, −1), (0, −1), (1, −1), (−1, 0), (1, 0), (−1, 1), (1, 1), (−1, 2), (1, 2), (−1, 3), (0, 3), (1, 3) |
| 3 × 1 | (−1, −1), (0, −1), (1, −1), (2, −1), (3, −1), (−1, 0), (3, 0), (−1, 1), (0, 1), (1, 1), (2, 1), (3, 1) |
| 4 × 3 | (0, 3), (1, 3), (2, 3), (−1, 3), (3, 3), (−1, 2), (3, 2), (−1, 1), (3, 1), (−1, 0), (3, 0), (0, −1), (1, −1), (2, −1), (−1, −1), (3, −1) |
| 1 × 4 | (−1, −1), (0, −1), (1, −1), (−1, 0), (1, 0), (−1, 1), (1, 1), (−1, 2), (1, 2), (−1, 3), (1, 3), (−1, 4), (0, 4), (1, 4) |
| 1 × 5 | (−1, −1), (0, −1), (1, −1), (−1, 0), (1, 0), (−1, 1), (1, 1), (−1, 2), (1, 2), (−1, 3), (1, 3), (−1, 4), (1, 4), (−1, 5), (0, 5), (1, 5) |
| 2 × 6 | (−1, −1), (0, −1), (1, −1), (2, −1), (−1, 0), (2, 0), (−1, 1), (2, 1), (−1, 2), (2, 2), (−1, 3), (2, 3), (−1, 5), (2, 5), (−1, 6), (0, 6), (1, 6), (2, 6) |
| 2 × 5 | (−1, −1), (0, −1), (1, −1), (2, −1), (−1, 0), (2, 0), (−1, 1), (2, 1), (−1, 2), (2, 2), (−1, 3), (2, 3), (−1, 5), (0, 5), (1, 5), (2, 5) |
| 5 × 3 | (−1, −1), (0, −1), (1, −1), (2, −1), (3, −1), (4, −1), (5, −1), (−1, 0), (5, 0), (−1, 1), (5, 1), (−1, 2), (5, 2), (−1, 3), (0, 3), (1, 3), (2, 3), (3, 3), (4, 3), (5, 3) |
| 4 × 4 | (−1, −1), (0, −1), (1, −1), (2, −1), (3, −1), (4, −1), (−1, 4), (0, 4), (1, 4), (2, 4), (3, 4), (4, 4), (−1, 0), (−1, 1), (−1, 2), (−1, 3), (−1, 4), (4, 0), (4, 1), (4, 2), (4, 3), (4, 4) |
| 3 × 4 | (0, 4), (1, 4), (2, 4), (−1, 4), (3, 4), (−1, 2), (3, 2), (−1, 1), (3, 1), (−1, 0), (3, 0), (0, −1), (1, −1), (2, −1), (−1, −1), (3, −1) |
| 6 × 4 | (2, −1) |
| 0 × 0 | empty |

Several lists are not the full ring around their foundation (`quirks.foundation_exit_lists`, Edge
cases): the 4 × 3 list is the 3 × 3 list, so (3, 0), (3, 1) and (3, 2) lie inside the
foundation and the cells beside its right edge are never tried. The 3 × 4 list has no cell beside
row 3. The 2 × 6 list has no cell beside row 4 and the 2 × 5 list none beside row 4. The 4 × 4 list
names (−1, 4) and (4, 4) twice. The 3 × 3 refinery list holds only a foundation cell. The 6 × 4 list
holds only (2, −1), and the 0 × 0 list is empty.

## State and ordering

Line states (per line):

| State | Event | Result |
|---|---|---|
| no line | build order applied, start conditions met | running, progress 0, *B* = *P* |
| running | step due, *x*ₖ ≤ *A* | progress + 1, pay *x*ₖ; done at 54 |
| running | step due, *x*ₖ > *A* | waiting for money; retry *R* ticks later |
| waiting for money | retry due, *x*ₖ ≤ *A* | running (step made and paid) |
| waiting for money | retry due, *x*ₖ > *A* | still waiting; retry *R* ticks later |
| running or waiting | hold order | player hold |
| player hold (also the forced-start hold, claim 10) | build order for the held type, claim 3's start conditions met (a switched-on factory building could make it and, for an aircraft, has a free dock) | running; next step a full *R* later |
| any with item | cancel order for the item's type, no queued copy | refund *P*′ − *B*; next queued starts, or no line |
| any | cancel order with a queued copy of the type | first queued copy removed |
| any with item | cancel-all order for the item's type | every queued copy removed; the item cancelled as above (refund *P*′ − *B*), the next queued type other than that one starts, or no line |
| any | cancel-all order for a type with queued copies and no item of that type | every queued copy removed; no money changes hands |
| any | cancel or cancel-all order for a type with no queued copy and no item of it | no change |
| running, waiting | check of claim 8: some factory building could make the item, none of them is switched on | automatic hold |
| automatic hold | check of claim 8: a switched-on factory building that could make the item exists | running; next step a full *R* later |
| player hold, done | check of claim 8: none switched on | no change (no second hold) |
| any with item | check of claim 8: no factory building of the house could make the item (build limit with production allowance included, full tech-tree conditions omitted, power and docks ignored, buildings being sold or queued to be sold excluded); line of a factory building of buildings: ordinary-building line only | as a cancel (refund *P*′ − *B*) |
| any with item | a factory building of its kind removed for good: no other building could make the item without the tech-tree rule (power and docks ignored; defence line included) | as a cancel |
| done (unit) | place order, release succeeds | next queued starts, or no line |
| done (unit) | place order, release busy or failed (claim 13) | as a cancel (refund *P*′ − *B*); next queued starts, or no line |
| done (building) | place order with a valid cell | building placed; no line |
| player hold (also the forced-start hold), automatic hold | build order for the held type, claim 3's start conditions not met | ignored; the hold stays |
| done | build order for the item's type | nothing added |
| any | build order for any other case of a busy line | type queued (queue limit and build limit permitting) |

Ordering consequences:
- A hold or cancel order given in the cycle of tick *n* is applied after the lines' update of tick
  *n*, so a step due in tick *n* is made and paid first (S-0203, claim 8).
- A refund from a cancel applied at the end of tick *n* is available to every line's step attempts
  from tick *n* + 1.
- Between two lines of one house due in the same tick, the earlier-created line pays first (claim 1).
- Claim 9's automatic orders are issued at the recheck's house update and take effect at that tick's
  order boundary, not during the house update. A step due in that tick can still be paid before
  the cancellation, and its payment counts in the refund.

## Edge cases and quirks

- **Refund at the current price** (`quirks.refund_at_current_price`, default on = original). Claim
  6's refund uses the price at the moment of the cancel. The refund less what was paid is then
  *P*′ − *P*, *P* being the price at the start. With a bonus below 1 (a cost reduction), a house
  that owns a bonus building when the item starts, loses it and then cancels, gets more than it
  paid (*P*′ > *P*); a house that gains a bonus building after the start and then cancels gets less
  than it paid, even a negative refund (claim 6). A bonus above 1 reverses both. With the switch
  off, the refund is the amount paid so far, the price at the start minus the balance owed, which is
  what was paid and never negative for a non-negative price. With the switch off our engine needs,
  per item, the amount paid so far or equivalently the price at the start. With the switch on the
  refund needs neither, but the free-unit condition still needs a produced building's start price
  (S-0402 claim 18).
- **Forced start on a player hold** (`quirks.forced_start_player_hold`, default on = original, medium
  confidence). An item started by claim 10's forced start (from the queue after a cancel or a
  release, with no switched-on factory building, or no free dock for an aircraft, that could make its
  type) starts on a player hold when the house's available credits are at least *P* ÷ 54 (integer
  quotient) at that moment, and running otherwise. This depends on the credits at the start, which
  no rule of the line otherwise does. With the switch off, such an item always starts running,
  waits for money at its first step if need be, and is put on an automatic hold by the next check of
  claim 8 (the same as the unaffordable case), so that no build order is needed to resume it. A
  player hold from this switch is lifted by a build order only (claim 5).
- **A building order to a busy building line (original only).** In the original, a build order for a
  building or defence applied while that line holds a held or done item cancels that item with a
  refund and starts the new one; applied while the line is running, it is refused. The original's
  interface never issues it (S-0411, claim 6). Our engine makes such an order illegal (claim 10), so
  neither outcome arises; no switch, and no scenario (the interface cannot give the order).
- **Free last step.** Step 54 never costs anything (claim 4), so an item at progress 53 completes even
  at 0 credits (not at a negative *A*, claim 6). This follows from the payment formula and has no switch.
- **Zero-payment steps.** A cheap item's early steps cost 0 credits and are made even at 0
  available credits, but not at a negative *A* (claim 6).
- **Price changes during a build** change neither the balance owed nor the remaining payments; only
  the refund (first quirk) and the price of later items.
- **Recorded price.** Our engine records a produced building's price at its production start
  and retains it through placement and build-up: S-0402 claim 18 compares that integer credit
  amount with the type's adjusted cost to decide whether its free unit appears. Later price
  changes do not replace this record. This is required even with the default
  `quirks.refund_at_current_price`. The sale refund still uses the adjusted cost and factors of
  the moment (S-0408 claim 1), and the default cancel refund uses the price of the moment
  (claim 6). With the refund switch off, every item also needs its start price or amount paid
  so far for that refund.
- **Pad aircraft list shorter than 2** (departure, no switch). Claim 2's pad term reads the second
  pad aircraft type even when the game-wide list has only one, which gives an undefined result in
  the original. Our engine takes the same departure as S-0408 (Edge cases): a missing entry counts
  as cost and price 0, no pad term applies when the list is empty or the first pad aircraft has no
  dock, and the importer warns. Unmeasured: the original's result is undefined, so no scenario can check it.
- **Defence line partly unchecked** (`quirks.defence_line_partial_factory_check`, default on =
  original). Claim 8's check after a factory building of buildings is placed, changes owner or is
  switched off or on covers only the ordinary-building line. The defence line is cancelled by claim 8
  only when a factory building of buildings is removed for good and no other could make its item
  without the tech-tree rule, whatever its power switch and docks, and it is never put on an
  automatic hold. So after a house's last factory building of buildings is
  captured, its defence item keeps building unless claim 9's orders cancel it. With the switch off,
  the defence line is checked exactly like the ordinary-building line.
- **A queued type that cannot start is dropped** (`quirks.unstartable_queue_head_stalls`, default on
  = original; low confidence). When a cancel or a release starts the first queued type and no factory
  building could make it (claim 10), that type leaves the queue without starting, and the line is
  left with no item and the rest of its queue. Such a line starts nothing more: a build order only
  adds to its queue, and only claim 8's removals or the loss of every queued type end it. With the
  switch off, the line drops every queued type that cannot start and starts the first one that can.
- **Negative own part.** A building type with a pad term but no free unit type whose adjusted cost
  is negative (S-0408, Edge cases) has a negative own part (claim 2), and its price is the sum of the
  parts without the lower limit of 0. With a balance owed *B* < 0 the first step pays *B* (F2 takes
  the lesser of *B* and the quotient), which gives the house credits, and the later steps pay 0.
  This is a data error of the mod, not a rule: our engine reproduces the original's result, which is
  not a departure, and the importer warns. No switch is proposed, because no other behaviour is
  defined for such a type.
- **Foundation exit lists** (`quirks.foundation_exit_lists`, default on = original). Claim 13's
  infantry exit search uses the hard-coded lists of Constants, gaps and repeats included, so the list never selects an outside exit cell on a 4 × 3 factory building's right side, and a factory building
  with the 3 × 3 refinery word, the 6 × 4 word or no foundation cells finds an exit only at its
  door cell or the single listed cell. With the switch off, every foundation's list is its full
  ring of outside cells: the row below the foundation from left to right (corners included),
  then the cells beside it from the bottom row upwards, left before right, then the row above it
  from left to right, corners included. That alternative is defined only for foundations
  with cells. For the 0 × 0 word the exit list is empty with the switch off as well as on
  (Q-0195): a foundation without cells has no outside ring. Such an infantry factory tries
  only its door candidates under claim 13; no literal four-cell ring or eight neighbours
  are inferred. This off-state rule is a design decision (high confidence), not an observable
  alternative in the original; P12 d covers the original empty-list result.
- **A busy factory refunds** (no switch). A done unit whose release is busy (the vehicle factory
  is still unloading with no idle twin, or every docking place of the infantry factory is in
  contact) is cancelled with a refund, not kept (claim 7). This follows from the original's
  handling of a player's place order and is reproduced for every player house.
- **Queue limit of 0 or below** from data refuses every queued type; the importer warns for a negative
  value.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| `Cost` (rules, type) | cost field *C* | integer, credits | 0 |
| `CostInfantryMult`, `CostUnitsMult`, `CostAircraftMult`, `CostBuildingsMult`, `CostDefensesMult` (rules, country) | country cost factor *a* for infantry, vehicles, aircraft, ordinary buildings, defences | decimal (S-0003, claims 6 and 7), held as a single | 1.0 |
| `FactoryPlant` (rules, building type) | the type grants cost bonuses | boolean | no |
| `InfantryCostBonus`, `UnitsCostBonus`, `AircraftCostBonus`, `BuildingsCostBonus`, `DefensesCostBonus` (rules, building type) | cost bonus *β* per kind | decimal, held as a single | 1.0 |
| `BuildCat` (rules, building type) | combat category: defence line and defence factors | category name | not combat |
| `Naval` (rules, vehicle type) | naval vehicle line | boolean | no |
| `Factory` (rules, building type) | factory building of a kind, including the matching-kind tests in claim 8 and Q-0190 | named value, one of the four producing words (S-0003, claim 21 and row C19) | none |
| `MaximumQueuedObjects` (rules, `General`) | maximum queued types per line | integer | 5 |
| `IncomeMult` (rules, country) | income factor *M* in F3 (S-0402, claim 12) | decimal, held as a single | 1.0 |
| `Cost` (rules, difficulty-row sections; rules, country) | no effect (claim 2) | decimal | — |
| `BuildLimit` (rules, type) | application and factory-check build limit with production allowance (claims 3, 8 and 10; S-0407 claim 9), plus the separate queue-time test (S-0407 claim 16) | signed 32-bit integer | 2³¹ − 1 |
| `Owner` (rules, type and factory building type) | owner-country overlap in factory eligibility (Terms); not the house's own country | list of country names | none; no overlap |
| `Prerequisite`, `TechLevel`, `RequiredHouses`, `ForbiddenHouses`, `RequiresStolenAlliedTech`, `RequiresStolenSovietTech`, `RequiresStolenThirdTech` (rules, type) | full tech-tree conditions for the offer, not retested by claims 3, 8 or 10 | forms and defaults owned by S-0407 Import mapping | S-0407 Import mapping |
| `FreeUnit` (rules, building type) | the unit type included in the price (claim 2); its appearance uses the recorded start price under S-0402 claim 18 | type name | none |
| `PadAircraft` (rules, `General`) | the pad aircraft types (claim 2) | list of type names | empty |
| `SeparateAircraft` (rules, `General`) | the separate-aircraft flag (claim 2) | boolean | no |
| `Dock` (rules, aircraft type) | dock types of an aircraft type; the first one is used in claim 2 and in S-0408's adjusted cost | list of type names | empty |
| `WeaponsFactory` (rules, building type) | vehicle factory flag: the vehicle rows of claim 13 | boolean | no |
| `Naval` (rules, building type) | naval factory building (Terms, claims 7 and 13) | boolean | no |
| `GDIBarracks`, `NODBarracks`, `YuriBarracks` (rules, building type) | the first, second and third barracks door kind (claim 13) | boolean | no |
| `ExitCoord` (rules, building type) | exit displacement (claim 13) | integer triple, leptons (S-0003, claim 9) | (0, 0, 0) |
| `Cloning` (rules, building type) | cloning building (claim 14) | boolean | no |
| `PoseDir` (rules, `AudioVisual`) | pose direction of an aircraft released onto a dock (claim 13) | integer, 1/256 turn | 0 |
| `AirportBound` (rules, aircraft type) | the type must return to a dock: no edge entry (claim 13) | form owned by S-0407 Import mapping | S-0407 |
| `Foundation` (art, building type) | foundation word, which selects a present exit list (Constants), empty for the zero-size word, and the centre used for aircraft pad positions (claim 15); unsupported words do not define arbitrary dimensions | named vocabulary and two-art-section selection, S-0108 claim 8 | S-0108 claim 8's selection and fallback |
| `Helipad` (rules, building type) | pad-building flag enabling docking displacements (claim 15) | boolean | no |
| `UnitRepair` (rules, building type) | repair-building flag also enabling docking displacements (claim 15) | boolean | no |
| `NumberOfDocks` (rules, building type) | imported docking-place count *d* used for displacement selection (claim 15) and max(*d*, 1) contact places (S-0417 claim 14) | integer | 1 |
| `DockingOffset0`, `DockingOffset1`, and subsequent numbered keys (art, building type) | displacement of the corresponding docking place from the foundation centre, including height (claim 15) | integer triple, leptons (S-0003 claim 9); numbering begins at 0, and only indices below the imported docking-place count are read | keep the value in force; new places start at (0, 0, 0) |

Sale queueing is an order state, not an imported type flag. S-0213 claim 12 owns the sale
order and S-0407 Terms define being sold and queued to be sold; no additional INI key is needed
for Q-0190. The factory kind, owner-country overlap, naval match and build limit use the rows
above. Q-0226's ordering uses no additional imported key: item kind, factory eligibility
and cloning membership use the existing rows, while creation precedence follows claims 11
and 14. Q-0195 changes an engine quirk's off state, not a data-file field.

## Measurement scenarios

Measurement setup (glossary), no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), game speed 6; a small purpose-made skirmish
map, one human house, one bystander computer player, named test data of the scenario (a test map's override section, a test rules file or a named test art file) as the only data exceptions, scripted input for exact order ticks,
and slow items (*R* of 100 ticks or more, S-0403) so that the credits display (S-0416) settles
between steps. "Credits" means the settled display value, which is the house's available credits.
The display shows a negative value as 0 (S-0416, claim 1), so a scenario that needs a negative *A*
observes it through the lines' behaviour. "Step ticks" are the ticks at which the build icon's
clock advances (S-0411, claim 8). The observed
area of every scenario is the human house's base: its factory buildings, its production lines and
the test objects the scenario names, plus the bystander's buildings the scenario names. The void rule
of the glossary's *bystander computer player* applies: a run is repeated when an object of the
bystander that the scenario does not name enters the observed area or acts on a test object, or when
the bystander is defeated before the observation ends. Scenarios that use the random-state recorder
(P11) add their own control.

Runs: each part below (and each "second run" it names) is run three times with the same script. The
scenarios are deterministic, so a difference between the three runs means that the setup was
disturbed, and the part is repeated; the three agreeing results are one observation.

**P1. Parallel lines and their update order.**
- Setup: the house owns factory buildings of every kind; starts an infantry item, a ground vehicle, a
  naval vehicle, an aircraft, an ordinary building and a defence in one order sequence.
  Part B: create the infantry line before the ground-vehicle line, then use P11 B's two
  automatic cancellations to start their replacement items in the same tick, with equal *R*
  and equal positive step payments. After the refunds, a named test trigger sets credits so
  only one payment fits at the first due step. Reverse line creation order in a fresh run,
  keeping the automatic cancellation order unchanged.
- Observable: step ticks of each line; in part B, which line's clock advances and the credits.
- Expected: all six advance independently, each at its own *R*; in part B the infantry line is paid
  and the vehicle line waits. Repeated with the vehicle line created first: the vehicle line is paid.
- Falsified if: any line waits for another, or in part B the later-created line is paid first.

**P2. Price.** All costs and factors below are test data.
- Part A (the rounding of the bonus product). Setup: the country's vehicle cost factor 1.0; two
  test cost-bonus buildings of one type, each with a vehicle bonus of 0.8, both owned; a vehicle
  type with *C* = 500. Observable: credits before and after the full build (total paid), and the
  per-step payments, which F2 makes determine *P* exactly. Expected (the original runs with
  `quirks.chop_rounding` on, so the product is narrowed with RZ₂₄): *b* is just below 0.64 and the
  total is 319. Falsified if the total is 320, which is the result with RN₂₄ and with an exact
  decimal product. With this *C* and these bonuses the two roundings give 319 and 320.
- Part B (the factors). Setup: the country's vehicle cost factor 0.8; two test cost-bonus buildings
  of one type with a vehicle bonus of 0.9 each, both owned; a vehicle type with *C* = 1001; a test
  rules file setting the normal row's cost factor and the country's general cost factor to 2.0.
  Observable: as in part A. Expected: total = F1 = 648; the row and general factors change nothing. Falsified if: the total
  differs from 648, or the 2.0 factors change it.
- Part C (acquisition order). Use three distinct bonus-building types with vehicle bonuses
  0.6, 0.7 and 0.75, a vehicle cost of 2000000000 test credits, and country factor 1.0.
  Acquire the buildings in each of the six possible orders, with ample credits to finish the
  item. Record total paid. Expect F1 using the acquisition order and the single-precision
  values imported under the original rounding mode, not a sorted or exact-decimal product.
  With bonuses acquired as 0.6, 0.7, 0.75, expect 629999935 credits; with acquisition
  as 0.6, 0.75, 0.7, expect 629999876 credits. A total differing from the corresponding
  prediction falsifies the rule. Equal-price order pairs do not distinguish relative order.
- Building part (the pad term and its rounding). Setup: the first two pad aircraft types with cost
  fields 1000 and 1002; a test dock building of cost field 3000 that is the first dock type of the
  first pad aircraft type, separate-aircraft flag off, no free unit; country aircraft cost factor
  0.5 and building cost factor 1.0; no bonus buildings. Then *C*′ = 3000 − 1001 = 1999, the own part
  is 1999, the pad prices are 500 and 501 and the pad term is 1001 ÷ 2 = 500. Observable: as in
  part A. Expected: total = 2499.
  Falsified if: the total is 3000 (the plain cost field), or 2500 (the pad term rounded up).
- Free unit part. Setup: a test building of cost field 3000 with a free unit type of cost field 800.
  Vehicle cost factor 0.5 and building cost factor 1.0; no pad term. Observable: as in part A.
  Expected: own part 2200, price
  of the free unit 400, total = 2600. Falsified if: the total is 3000 or 2200.
- Negative own part (low confidence, Edge cases). Setup: pad aircraft cost fields 1000 and 1000; a
  test dock building of cost field 100, the first dock type of the first pad aircraft type,
  separate-aircraft flag off, no free unit; aircraft factor 0.5, building factor 1.0. Then *C*′ =
  100 − 1000 = −900, the own part is −900, the pad term is 500 and *P* = −400. Observable: credits at
  the start and after each step. Expected: the credits rise by 400 at step 1 and later steps change
  nothing. Falsified if: the credits do not rise, or the price is clamped to 0.

**P3. Start, queue and limit.**
- Observable for every part unless the part names its own: whether the line exists and its clock
  advances, the queue count (S-0411, claim 7), the refusal sound and the credits.
- (a) Setup: credits 0; a build order for a vehicle type whose price is at least 53 (so that its
  first step pays at least 1 credit, F2); then credits given by test trigger; then 7 further build
  orders of the same vehicle type with the absent-key queue limit. Expected: at 0 credits the item starts and
  waits at step 1; 5 copies queued and 2 refused with a refusal sound. Falsified if: the item does
  not start at 0 credits or the queue holds another number.
- (b) Setup: a build-limit-1 test type ordered twice. Expected: the second order refused.
  Falsified if: the second copy is queued or started.
- (c) Setup: a second building ordered through the sidebar while one is being built. Expected: the
  click is refused with no order effect. Falsified if: the second building starts or replaces the
  first.
- (d) Free dock. Setup: an aircraft factory building whose docking places are all taken by docked
  aircraft of the house (placed by the map or by test trigger, as in P8 run 5), the only switched-on
  aircraft factory building; a build order for an aircraft on the idle aircraft line. Second run:
  an aircraft item already running (started before the docks filled), then a further aircraft
  build order meant for the queue. Observable: whether the aircraft line exists and its queue count
  (S-0411, claim 7). Expected: no aircraft line is created in the first run, and in the second the
  queue count stays 0. Falsified if: an aircraft order is applied or queued with every dock taken.
- (e) Queue limit 0 in the test rules file: a vehicle item started, then a second order. Expected:
  the first item starts and every further order is refused. Falsified if: a type is queued.
- (f) Switched-off only factory. Setup: the house's only infantry factory building switched off by
  the power toggle; an infantry build order. Expected: nothing starts and no line exists.
  Falsified if: an item starts or is queued.
- (g) Owner countries. Setup: a test factory building type whose owner countries share no country
  with an infantry type's owner countries, the only infantry factory building; an order for that
  infantry type. Expected: ignored. Falsified if: it starts or is queued.
- (h) Naval match. Setup: only a non-naval vehicle factory building; a build order for a naval
  vehicle; then the same with only a naval factory building and a ground vehicle. Expected: both
  orders ignored. Falsified if: either starts or is queued.
- (i) Stale prerequisite. Setup: reproduce S-0407 T14 run 8's stale refinery-group refusal, with
  a switched-on factory, a limit that permits the order and no intervening recheck. Use game
  speed 6 and a read-only order and line-state recorder. Procedure: click the idle offered icon
  once; in a fresh run start and hold the item before the non-event, then click its icon to
  resume while the offer is stale. Observable: issue and application ticks, item existence,
  hold state, progress and queue. Expected: the first order starts an item at its boundary.
  The second resumes the held item with no extra copy. Falsified if: prerequisite refusal alone
  prevents either effect. Void a run if an intervening recheck removes the offer.
- (j) Resume allowance. Use S-0411 S4 b2's held build-limit-1 infantry item, first alone, then
  with an extra on-map object of the type. Procedure: click the held item's icon once in each
  state. Observable: build-order issue and application, hold state and queue. Expected: both
  clicks issue resume requests; the item alone resumes, but with the extra object it stays held
  and no copy queues. Falsified if: the second order resumes despite the allowance being refused,
  the first is refused, or either adds a copy. S-0407 claim 9 owns the infantry exactness.

- (k) Application-time refusal cues. Use the stock full-queue run of (a) with audio
  capture and a read-only order recorder: expect the game-wide refusal sound at each
  refused queue request's application, as in S-0701 F5. For a named test map override,
  give a unit type build limit 2 and a queue limit greater than 2. Start one slow item,
  with no on-map copy, then issue two further build requests in the same tick while
  both pass the issue-time test. Record their issue and application ticks, queue count
  and requested sound entry. Expect the first request to queue one copy and the second
  to fail S-0407 claim 16's queue-time test, requesting the game-wide refusal sound at
  that boundary. Void if either request is refused before issue or another action
  changes the counts. A queued second copy, a different refusal entry, or no refusal
  request falsifies the application-time rule. Repeat as each side; cue identity does
  not depend on side. This does not predict a refusal cue for part j's factory-eligibility
  failure or for an at-limit click that issues no order.

**P4. Payment and waiting for money.**
- Setup: *P* = 1000 item with *R* = 200 at ample credits; then the same with credits 100 and a test
  trigger that gives 50 credits at a known tick. Part C: a *P* = 10 item at credits 0, and a
  second run with a *P* = 1000 item whose house has exactly 1000 credits at the start and no other
  income.
- Observable: credits after every step; step ticks.
- Expected: payments follow F2 (18 at steps 1 to 7, 19 at steps 8 to 53, 0 at step 54); in the
  second run the line stops when the next payment exceeds the credits, retries every 200 ticks at
  the original phase, and resumes at the first retry after the gift.
  Part C: with *P* = 10 at 0 credits, steps 1 to 43 are made (they pay 0) and the line waits at
  step 44; with exactly *P* credits the item completes at step 54 with 0 credits left.
- Falsified if: payments differ, a partial payment occurs, or the retry phase shifts to the gift tick.
  In part C, a step of the first 43 waits at 0 credits, or the 54th step waits or charges anything.
- The harvester delivery in the turn pass of a retry tick (claim 4) is S-0203's scenario O7, part A.

**P5. Hold and resume.**
- Setup: a unit item (infantry, vehicle or aircraft, not a building or defence: claim 10 and Edge
  cases, *A building order to a busy building line*) with *R* = 200; hold at a known tick 120 ticks
  into a step; resume 500 ticks later. Part B:
  the hold order is given in the cycle of a tick in which a step falls due. Part C: a second hold
  order to the held line, and a hold order to a line with a done item.
- Observable: step ticks, credits.
- Expected: no step and no payment during the hold; next step 200 ticks after the resume; no refund
  on hold; in part B the step due in that tick is made and paid, and the hold begins after it; in
  part C neither order changes anything (the held line stays held and resumes as above, the done
  item stays done).
- Falsified if: the line keeps the partial step, money is refunded, a step occurs during the hold,
  in part B the due step is not made, or in part C an order changes the line.

**P6. Cancel and refund.** Test cost-bonus buildings are added or removed by test trigger.
- (a) Cancel at progress 20 with an unchanged price. Expected: the refund equals the amount paid.
- (b) Two queued copies of the same type, cancel once. Expected: one queued copy removed, the item
  continues. Falsified if: the running item is cancelled.
- (c) Cancel-all with two queued copies. Expected: all copies and the item gone, refund as (a).
- (d) The house owns a cost-bonus building of bonus 0.8 for the kind; a vehicle with *C* = 1000 and
  vehicle factor 1.0 is started (*P* = 799 under the original's single-precision import and
  chop rounding); at progress 20 the bonus building is removed and the item is cancelled.
  Then *B* = 499 and *P*′ = 1000. Expected: refund 501, which is 201 more than the 300 paid.
  Falsified if: the refund is 300 (the amount paid) or anything but 501.
- (e) A vehicle with *C* = 1000 and no bonus (*P* = 1000) at ample credits; a bonus building of
  bonus 0.5 is gained at progress 20; the item runs on to the end. Expected: the payments after the
  gain follow F2 on the balance owed of the start (19 per step from step 21 on), and the total paid
  is 1000. Falsified if: the payments change with the bonus building.
- (f) A second line's item waiting for money with retry ticks known; a cancel on the first line is
  given in the cycle of one of those retry ticks, and in a second run in the cycle of the tick before
  it. Expected: in the first run the waiting line's step at the retry tick is refused and made one *R*
  later, in the second run it is made at the retry tick (a refund is available from the next tick
  on). Falsified if: the refund is already available in its own tick.
- (g) Negative refund. Credits 473 at the start and no income; a vehicle with *C* = 1000 and no
  bonus (*P* = 1000); a second line's item of another kind with *P* = 10 started just after it. Both have *R* = 200
  ticks, so the second remains in its zero-payment portion through the cancellation.
  At progress 20 (*B* = 627, 373 paid, 100 credits left) a bonus building of bonus 0.5 is gained
  and the first item is cancelled: *P*′ = 500, refund −127. Observable: the second line's clock.
  Expected: the first line's clock disappears; the second line's steps (which pay 0) are refused
  from then on and its clock stops; after a gift of 27 credits (making *A* = 0) it resumes at its
  next retry tick. Falsified if: the second line keeps advancing, or does not resume after the gift.
- (h) A cancel order for a type that has no queued copy and is not the line's item. Expected:
  nothing changes and no money changes hands. Falsified if: the item or a queued type is removed.
- Observable for all: credits, queue count, line state; in (e) the per-step payments.
- Falsified in general by: any refund that differs from F4.

**P7. Completion and release.**
- Setup: two vehicle factories, the second acquired later, none marked primary; then the first marked
  primary; a run with the only factory switched off by the player's power toggle at completion.
- Observable: which factory releases the unit; tick of release; whether the next queued item starts
  at the release; the item state while blocked; the queue count after the build order.
- Expected: the later-acquired factory releases without a primary mark, the primary one with it.
  After a successful clear-exit release the next queued item starts at that boundary. A build
  order for an item that is still done adds nothing. Switched-off only factory: the item stays
  done until it is switched on and a place order is given (S-0411, claim 4).
- Acquisition-order runs: keep two eligible factories A and B switched on, neither primary,
  with clear exits. Create A before B, acquire B first and capture or transfer A to the observed
  house later; expect release from A. Transfer B away and reacquire it while A stays owned.
  Expect release from B. Assign A to its unchanged owner; expect B still. In fresh runs acquire
  both in one tick, reversing their event order without changing object creation order; expect
  release from whichever acquisition takes effect last. Use scripted input and test-map
  ownership-change events, recording their within-tick order and the releasing factory with a
  read-only recorder. Void a run if either factory becomes primary or fails eligibility.
  A release from the earlier-acquired factory in any of these controls falsifies the rule.
- Busy and failed releases are scenario P12.
- Falsified if: another factory releases the unit in the clear-exit control, its next item starts
  later, a build order for an item recorded as still done adds a queued copy, or the
  switched-off-only-factory run releases or refunds the done item before the switch-on and
  later place order.

**P8. Factory buildings leaving or losing power.**
- Setup, runs 1 to 6: an infantry item in production with two queued infantry types; the only
  infantry factory is sold; in a second run a second factory exists; in a third run the only factory
  is switched off by the power toggle, then switched on again; in a fourth run the house falls into
  a power deficit; in a fifth run an aircraft item runs while every dock of the only aircraft
  factory gets taken; in a sixth run the house's only factory building of buildings is captured by
  test trigger while an ordinary building and a defence are in production (a test rules file lets
  both lines run).
- Run 7 (a player hold is never lifted by the check): the item put on a player hold, the only
  factory switched off and on again. Run 8 (the queue rule): an item whose type the sold factory
  and another factory can both make, with a queued type that only the sold factory could make
  (by owner countries); the sold factory is sold. Run 9 (removal for good, defence line included):
  the only factory building of buildings is destroyed by test trigger while an ordinary building and
  a defence are in production.
- Observable: credits, queue, line state; in runs 6 and 9 the tick of each refund (credits recorded
  per tick by the recorder, or the clocks' disappearance).
- Expected: run 1: the item refunded, the queue cleared; run 2: unchanged; run 3: automatic hold
  while switched off, resumes by itself with a full step period after it is switched on; run 4: no
  hold (only slower steps, S-0403 claim 5); run 5: no hold, the item keeps building; run 6: the
  ordinary building refunded during the capture's turn pass, the defence only through claim 9's
  cancel orders at the end of that same tick; run 7: the hold stays through both changes and ends
  only by a build order; run 8: the queued type leaves the queue and the item continues; run 9: both
  items refunded in the tick of the destruction.
- Falsified if: any of these outcomes differs, in particular if run 5 holds the item, run 6
  refunds the defence before that tick's order boundary, or run 7 lifts the hold.
- Run 10 (the full tech tree is not part of the factory check): use P3 (i)'s stale
  refinery-group loss, with an item and a queued type that both need that group, and build limits
  that permit both. In a tick *n* after the non-event, before another recheck, switch the only
  factory off by an order. Use game speed 6 and a read-only line and order recorder. Procedure:
  record the line at the switch-order boundary and through the next house update and automatic
  cancel boundary. Void if any earlier recheck removes either offer. Observable: hold state,
  queue, item, credits and cancellation-order ticks. Expected: at the end of *n*, the factory
  check puts the item on automatic hold without refunding it or removing the queued type merely
  because their prerequisite fails. The switch calls for an offer recheck in *n* + 1, whose
  cancellation orders then remove them at the end of *n* + 1 (claim 9). Falsified if: the factory
  check itself cancels or removes them at the end of *n* on the failed prerequisite alone.
- Run 11 (a factory being sold): an infantry item in production, two infantry factories, the
  primary one sold by the player; in a second run the sale of the primary is only queued (the
  sell order given while it is busy with another behaviour); in both runs another factory event
  (a third infantry factory completed) happens during the sale. Observable: credits per tick,
  line state, and the building in which the done unit appears. Expected: the item keeps paying
  its steps through the sale until the building leaves the map; a unit done during the sale
  appears from the other factory; the third factory's event leaves the item unchanged. Falsified
  if: the item is held or refunded at the start of the sale, or the unit comes out of the
  building being sold.
- Run 12 (Q-0190). Setup: game speed 6, a tiny map, ample credits, two ground-vehicle
  factories X and Y with overlapping owner countries for item A. Start A and queue type B
  that only Y could make (owner-country overlap), with generous build limits. Queue Y's
  sale while it is busy unloading an earlier vehicle. A read-only recorder must confirm
  Y's queued selling behaviour and that its sale has not begun when the test event occurs.
  Procedure: switch X off through an order; in a fresh run destroy X through a test trigger
  instead. Record from before the event through the next house update and order boundary.
  Void a run if Y starts selling or another recheck removes A or B before the event.
  Observable: item identity, progress, hold state, queue contents, credits, factory states
  and cancellation-order issue/application ticks. Expected: switch-off run, A enters automatic
  hold at the switch-order boundary with no immediate refund, and B leaves the queue. The
  ensuing offer recheck can later cancel A at its order boundary; that is not the factory
  check's hold. Destruction run, A is cancelled with claim 6's refund in the removal tick
  and B leaves the queue, independently of later sidebar orders and without a second refund.
  Falsified if Y keeps A running after X's switch-off, keeps A alive after X's permanent
  removal, or keeps B queued after either factory check. Repeat with Y's sale actually
  running as a control: exclusion is the same. Use scripted input and the read-only recorder
  of the measurement setup; no modified executable or harness change is required.

**P9. Items whose type leaves the taken-up set.**
- Setup: an item whose type needs a test prerequisite building and a queued copy; no factory
  building is lost. Use game speed 6 and the measurement setup above. Three parts: (a) destroy
  the prerequisite by test trigger in the turn pass of a known tick *n*; (b) switch off by an
  order at the end of tick *n* the most recently acquired building that holds a prerequisite
  upgrade, leaving no other building that meets that prerequisite (S-0407, claim 8 c); (c) a
  test refinery-alternative vehicle without the don't-score flag is the only object meeting the
  prerequisite's refinery group, and enters a transport in tick *n*; later place an unrelated
  building, recording its placement-order tick *m*. In (c) no other recheck event may intervene.
- Procedure: start the item and queue the copy before each loss. Record through the first recheck
  after the loss, without further production clicks. In (c) wait several ticks before the
  unrelated placement. Repeat each part three times as above; void (c) if another recheck event
  intervenes.
- Observable: icon presence, line progress, queue count and credits; a read-only recorder logs the
  tick of each automatic order's issue and application (S-0411, Terms, Recorder). The settled
  credit display confirms the refund amount; it is not used to locate the refund tick.
- Expected: (a) both orders issued in tick *n* and applied at its end; (b) both issued in tick
  *n* + 1 and applied at its end; (c) none issued at the vehicle's entry into the transport,
  production continues while the offer is stale, then both issued in tick *m* + 1 and applied at
  its end. At that boundary the item is refunded by claim 6 and every queued copy removed; a step
  paid earlier in that tick counts in the refund.
- Falsified if: either order is issued in another tick, (c) cancels before a recheck, or the
  item or a queued copy remains after the application boundary. If part (c)'s stale offer is not
  reproducible, revise it with S-0407's T14 rather than infer the timing from that run.

**P10. Forced start (claim 10 and its quirks).**
- Setup, part A (forced start on a hold): an aircraft factory building with a free dock; an aircraft
  item in production and a second aircraft type queued, both ordered while the dock was free (claim 3
  refuses aircraft orders while every dock is taken); then every dock is occupied by test trigger
  and the item completes and is released through the fallback (claim 7). A second run with the
  credits set below the queued item's price ÷ 54 (integer quotient) at that moment, a third with
  them at exactly that bound.
- Observable: the queued item's state after it starts.
- Expected: it starts on a player hold when the available credits are at least its price ÷ 54 at
  the start (the third run: held; the second run: running, and waiting for money at its first
  step if it cannot pay), and it is resumed by a build order for its type once a dock is free.
- Falsified if: it is refused, runs while affordable, or is held while unaffordable. A result in
  which every run is running would show the behaviour of `quirks.forced_start_player_hold` off.
- Setup, part B (the dropped queue head, low confidence): vehicle item A running with two types
  B and C queued. By owner-country overlap, only a separate factory X can make B; another factory
  can make A and C. Start selling X, then cancel A before X leaves the map and before another
  factory check or offer recheck removes B. Use game speed 6 and a read-only line-state and order
  recorder. Void the run if B is no longer queued when the cancel is applied. In a control run
  cancel A after X's removal and the removal of B from the queue.
- Observable: at the cancel boundary, whether B or C starts, the remaining queue and item.
- Expected: in the first run B leaves the queue without starting and C stays queued with the line
  holding no item and starting nothing (build orders only add to the queue); in the control B is
  already removed and the cancel of A starts C.
- Falsified if: in the first run C starts, or B starts. If no reachable cancel boundary retains B
  while X is selling, the scenario is inconclusive and the rule keeps its low confidence.
- Part C (a stale queued prerequisite). Reproduce P3 (i), but start A and queue B while B's
  prerequisite holds. Make B's prerequisite fail through the non-event, then cancel A before
  another recheck. Observable: item and queue at the boundary. Expected: B starts despite the
  stale prerequisite refusal, provided its factory and application-time limit permit it.
  Falsified if: that refusal alone drops B or stalls the line. Void if another recheck intervenes.

**P11. Random draws of the lines (claim 11).**
- Part A setup: game speed 6; a test map with nothing that moves or animates and with the glossary's bystander computer
  player kept idle by named test data, on which the random-state recorder's control interval (no
  items, 300 ticks) shows no advance of the scenario stream, as in S-0202's scenario R5 (a); if it
  advances the scenario first removes the cause. Then, with scripted input at known ticks, in one
  run, each in a tick of its own (so that no tick holds two starts): start a building item and let it
  run to step 54 (done, not placed); start an infantry item with a second type queued; give a hold
  and a resume; a cancel with a refund (which starts the queued type); a cancel-all; a line waiting
  for money for several retries.
- Observable: the recorded stream position at every tick, which gives the number of raw values taken
  per tick (S-0202, scenario R5's method).
- Expected: exactly one raw value is taken in each tick in which an item starts (the first start
  of each line, and the start of a queued type after a cancel), and none in any other tick, in
  particular none at steps, payments, refusals, retries, holds, resumes, refunds or at the
  building's completion.
- Falsified if: any other tick takes a raw value, or a start tick takes none or more than one.
- Part B (simultaneous replacement starts). Use the same quiet setup at game speed 6,
  plus the read-only line/order recorder and each item's creation sample. Give an infantry
  type A and vehicle type D the same test prerequisite, with one queued replacement B and E
  respectively that does not need it. Factories, limits and owner-country overlap permit
  all four types. Start the vehicle line first, then the infantry line, keeping both items
  slow enough not to finish. Destroy only the prerequisite with a named test trigger.
  At the recheck, S-0411 claim 13 removes A's icon in the infantry tab before D's in the
  vehicle tab. Record order application, the two replacements' creation samples and the
  scenario stream before and after the boundary. Void if unrelated draws or an earlier
  factory check alter either queue. Repeat with the line creation order reversed.
  Expected: B's sample is the low 16 bits of the first raw value at the boundary and E's
  is the low 16 bits of the next value, in both runs, with exactly two creation draws.
  No step occurs between those starts. Falsified if the samples fit reversed assignment,
  the count differs, or acquisition of a line changes assignment. A run whose two predicted
  samples are equal is inconclusive for assignment and is repeated, not counted among the
  three distinguishing observations. The recorder reads existing values only; it never sets
  state or places instrumentation in the original.
- Part C (a turn-pass start and an order-boundary start). Use the quiet setup above, game
  speed 6 and scripted input. Start infantry A with B queued. By test owner-country lists,
  only factory X can make A, while surviving factory Y can make B. Keep build limits
  permissive and a separate vehicle factory available. Include a named enemy test attacker
  whose lethal hit destroys X during the turn pass of tick *n*. Use a pilot run to schedule
  that hit, then scripted input to issue a build order for vehicle E in the same tick, with
  its vehicle line previously absent. Void if the hit falls outside the turn pass or in
  another tick, either start fails, another factory check changes B's queue, or the input
  lands in another tick. Record B's creation sample after the turn
  pass and E's after the order boundary, plus the stream through both phases. Expect B's
  sample at an earlier stream position than E's, independently of line creation order.
  Destruction or intervening effects may take other values between them: no adjacent-value
  or total-count prediction is made. Require unique sample matches in the recorded
  interval, repeating collisions as in part B. Reversed assignment falsifies the rule.
  Repeat three distinguishing runs. This tests causing-phase precedence, not whether the
  computer player's separate production machinery follows player orders.

**P12. Releasing a unit (claims 13 and 14).** Game speed 6, a read-only line, order and object
recorder (position in leptons, facing, current behaviour per tick) and the random-state recorder.
Test data only; stock building types keep their own exit data.
- (a) Ground vehicle factory, exit displacement set to a test value. Expected: the vehicle
  appears at the building's top-left cell centre plus that displacement, facing 16384, and
  unloading is queued, not synchronously started (S-0417 claim 2 and U2). Second run with displacement (0, 0, 0): it appears at
  the foundation centre. Falsified if: another point or facing.
- (b) Busy vehicle factory. Two vehicle items finishing within a few ticks of each other on one
  ground vehicle factory, with no second factory of that type. Expected: the second release, while
  the first vehicle is still being let out, cancels the second item at that boundary with the
  refund of claim 6, and the next queued type starts. Second run with an idle second factory of
  the same type that is switched off: the second vehicle appears in that one. Third run with a
  second factory of another vehicle-factory type: refund as in the first run. Falsified if: the
  item waits and appears later without a new order, or the hand-off picks another type.
- (c) Naval factory with a rally point east of it, the cells in that direction water and empty.
  Expected: the vessel appears at the first cell beyond the foundation in that 8-direction,
  facing 16384, and moves to the rally point. Second run with that cell occupied by a test
  vessel: it appears at the nearby-cell search's cell instead. Falsified if: another cell.
- (d) Infantry factory, 2 × 2 foundation, no door kind, rally point set. Fill the exit list's
  cells in turn with test vehicles. Expected: the infantry appears in the foundation cell next to
  the first free listed cell, facing its rounded direction. With every listed cell blocked,
  the item is cancelled with a refund. Falsified if: another selected exit cell, appear point or
  facing, or a kept item when all candidates are blocked. Record initial destination and
  subsequent movement separately under S-0417 U10: with a rally outside its own cell, ordinary
  infantry walks to the exit cell then to the rally. Repeat without a rally: the release gives
  no destination; any subsequent walk-out is exploratory, not a promised exit-cell walk.
  Repeat the candidate-blocking procedure for every foundation word in Constants using a
  named test art file, door flags off, and record the first accepted candidate or failure.
  Expect the listed first-accepted order and gaps, not a full outside ring. This static
  blocking procedure does not distinguish repeated identical entries. An unlisted exit,
  a different first accepted candidate, or success with the empty list and door flags off
  falsifies that case. Also test an unsupported foundation word in the image-named art
  section, with no overriding foundation in the type-id section. Expect S-0108 claim 8's
  1 × 1 fallback and its listed exit candidates, not dimensions suggested by the unsupported
  text or a new perimeter search. Record the selected foundation and first accepted exit.
  Any other foundation or candidate order falsifies this import case.
- (e) Door kinds: a test infantry factory with each door kind in turn and an exit displacement.
  Repeat with all three flags, blocking successive door cells. Observable: exit cell, appearance
  point and facing. Expected: the available door cell is used first in claim 13's order and the
  displacement is added only there. Falsified by a different candidate priority or displacement
  on a non-door exit.
- (f) Busy infantry factory: two infantry items that finish in consecutive ticks on one infantry
  factory. Expected: the second is refunded while the first is still in contact with the
  building. Low confidence: if the second appears instead, the busy test of claim 13 is wrong.
- (g) Cloning: the house owns two cloning buildings with empty contacts and unblocked exits.
  Release one ordinary infantry unit with no random idle animation and a rally point outside
  its cell. Establish a quiet control as P11 does. Record copies, release ticks, each copy's
  creation sample and scenario stream. Expect two copies of the original type, one per cloning
  building, and one creation draw per copy, assigned in cloning-building acquisition order.
  Reverse that acquisition order in a fresh run. Locate each copy's sample among the raw
  values of the release interval. Expect the earlier-acquired building's sample at an earlier
  stream position, even if placement consumes intervening values. Require a unique match
  for each sample in that interval; collisions are inconclusive and require another run.
  Total counts alone cannot test this order or assign unknown placement draws.
  Repeat with one cloning building's candidates all blocked: its copy does not appear, but
  still takes a creation draw; the other copy appears. With the original factory's exits
  all blocked, expect no copies and no copy-creation draws. Falsified by a different copy
  count or type, reversed sample assignment in a distinguishing run, a missing creation
  draw for the blocked copy, or any copy when the original found no exit.
  Also record the original unit's appearance and creation samples with copies on opposite
  sides of the original factory. Swap their locations while preserving acquisition order.
  Expect the same copy precedence, each copy's release outcome before the next creation,
  and, using the next tick's recorded first-turn precedence (S-0203), both appeared copies
  before the original. A location-dependent reversal or original map-entry precedence ahead
  of the copies falsifies claim 14. The exact placement-draw grouping is inconclusive when
  no placement draws can be identified independently. Unknown initial-placement draws remain exploratory: a
  changed next sample alone does not falsify acquisition precedence.
  Finally give a test cloning building the infantry-factory role and produce the original
  directly through it. Expect no extra copies or copy-creation draws from that release.
  Either would falsify the no-recursive-cloning rule.
- (h) Aircraft: an aircraft factory with every docking place taken, a type that need not return
  to a dock, factory in the left half of a local area with *L*h ≥ 2. Expected: the aircraft
  appears on the cell column just outside the left edge, facing 0; then a factory in
  the right half: right edge. Decode the raw values in the recorded release interval using
  S-0202's ranged rule over 0 … *L*h. The entry row must match one of those ranged values.
  No match falsifies the edge-row rule. Multiple matches leave draw assignment inconclusive.
  Additional unresolved placement draws mean the interval's total is not a prediction of
  the isolated edge-entry count. A type that must return to a dock: the item is refunded.
  Another edge or facing, or a kept dock-required item, also falsifies the corresponding rule.

**P13. Aircraft pad positions.** Use the measurement setup above, game speed 6, scripted
production clicks and a read-only recorder of world positions, contact-place indices and release
ticks. Run each part three times. The observed area is the human base; the bystander void rule
applies. With stock data first produce aircraft into each free place of an aircraft factory,
then vacate an earlier place and produce another aircraft. Record the building's foundation
centre, imported docking displacements and the aircraft's final position at each release boundary.
Expect claim 15's component-wise sum for the first empty contact place, including world height,
not the building's top-left cell centre or a rotated displacement. A differing position or failure
to reuse the earlier empty place falsifies the rule.

For the import branches, use named test map overrides: a test aircraft factory with pad flag on,
first one docking place, then two, with distinct integer triples of your choice in its art; a
control with both enabling flags off; and a repair-flag-only control. Use a named test art file
for these triples, remove it after the run, and keep unrelated stock data unchanged. Repeat with
a different pose direction and exit displacement. Expect the one-place displacement, then each
selected two-place displacement; neither pose direction nor exit displacement changes the result.
The flag-off control uses the centre and the repair-only control uses the displacement. Also
omit the displacement keys in a fresh test type: expect zero displacements. Any difference
falsifies the corresponding branch. Record the release's random stream as exploratory data
only: unresolved initial-placement draws prevent this scenario from isolating position
selection's no-draw rule. No predicted total draw count is asserted for this release.

## Open questions

1. Q-0120's assigned aircraft docking-position part is answered by claim 15. Most other parts
   were already answered after the request was filed: claims 7 and 13 give facings, release
   results and immediate busy/failed cancellation without retry, and S-0417 claims
   2 to 12 give unloading, infantry rally movement, aircraft release tether timing and their
   draw consequences. S-0417 claim 14 now supplies the nearby destination of edge-entering
   aircraft, the exact infantry exit-entry test and contact-place count max(*d*, 1).
   The remaining Q-0120 follow-up belongs to S-0417: no-rally infantry walk-out (Open
   question 3) and initial-placement draws (Open question 8). Nearby-search draws are
   resolved as none by S-0212 claim 25. No unknown
   aircraft release-tether timing or ordinary pad-position rule remains here.
2. Claim 15 does not establish aircraft docking positions for modded aircraft factories also
   marked for resource unloading or tank bunkering. Those combinations select specialised
   docking positions rather than the ordinary pad rule; no stock aircraft factory using them
   is established for P13.

3. Q-0193: if a destruction hit kills an off-map production item while an eligible factory
   survives, its subsequent production lifetime, refund, progress, completion, release or
   placement, and surviving-factory outcomes remain open. S-0414 claim 17 answers only the
   defeat-count consequence; claims 3, 6 and 7 here describe ordinary production and do not
   establish killed-item behaviour. No answer for these parts is inferred from a hit being
   eligible or from its count remaining unchanged.

4. Claim 13, ground vehicle factory row: whether a release whose exit point lies off the map, with
   an exit displacement that puts it there, gives *failed* by the table's general sentence is not
   established (low). The row's no-entry-test rule covers the occupants, terrain and overlay of an
   exit point's cell on the map only.
