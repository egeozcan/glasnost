---
id: S-0603
title: Grinder (recycling units into credits)
topic: structures
revision: 9
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-19
measurements: []
requests: [Q-1157]
supersedes: []
---

# S-0603: Grinder (recycling units into credits)

## Summary

Recycling buildings consume arriving infantry and vehicles, pay credits, and remove their
first two passenger levels and attached parasites. Deeper passenger nesting has the
separate, unmeasured outcome in Edge cases. This spec covers eligibility, arrival, payment,
removal effects and interruptions in skirmish only; network games and campaigns are out of scope.

Consumption is a **removal from the game, not a death**. None of the death effects of S-0312
(claims 5 to 7 and 12) apply, nobody is credited with a kill and no loss is counted.

Neighbouring specs own the rest, and this spec does not repeat them:
- the recycle order, its cursor and who may give it: S-0214 (tables I, V and M, rows I5, I26, n1
  and n2),
- the *eaten* behaviour's processing, its fall-back to rest and the scatter off a building cell:
  S-0215 (claims 14, 15 and 28),
- moving onto the building's foundation: S-0209 (claim 4 row WP3 for infantry, claim 5 rows VP6
  and VP7 for vehicles),
- the refund formula and its factors: S-0408 (claim 1, Formulas F1),
- the entered-by raise: S-0524 (claim 12), event matching: S-0011 (claims 18 and 19),
- the death response and death sound choice: S-0701 (claim 20, F5),
- the building's slot animations and the yellow ratio: S-0108 (claim 20, F3),
- the release of a controller's victims when it leaves the game: S-0601 (claim 10),
- a parasite's release when its host is consumed: S-0614 (claims 12 and 15),
- the original computer player's fate decision that sends a controlled unit to a grinder: S-0601
  (claim 8),
- house counts: S-0414 (claims 4 and 21),
- the original's enter-nearest instruction, which looks for an *absorbing* building and is a
  different action from claim 14's search: S-0215 (claim 24),
- a slave master that is consumed: its untransferred slave group ends by S-0312 claim 31,
- an entrant converted by mutation before it arrives: S-0611 claim 5,
- being drained adds no direct entry or consumption restriction and leaves the recycle payout
  going to the building's owner: S-0608 claim 18. Any separate transfer to the drainer is
  S-0608 claim 7's payout, not a change to this spec's recycle payout.

## Terms

- **Recycling building.** S-0214's term (Terms): a building whose type has the grinding flag
  (Import mapping). This spec uses no other name for it.
- **Entrant.** An infantry unit or vehicle in the *eaten* behaviour (S-0215 Terms) whose destination,
  or target, is a recycling building.
- **Consumption.** The moment a recycling building takes in an entrant and removes it from the
  game (claims 5 to 11). The entrant is *removed* in the glossary's sense (*in play … destroyed,
  removed*), not destroyed.
- **Recycle payout.** The credits the building's owner receives at a consumption (claim 8,
  Formulas F1). It uses S-0408's sale refund formula but differs in whose factors are used and
  who is paid.
- **Path computation, level-2 sector.** S-0212's terms (claims 13 and 13a); not in the glossary.
- **Link places.** As S-0605 (Terms): the building's imported dock count, at least 1.
- **Yellow ratio.** The rules-wide health ratio at or below which an object counts as damaged
  for its slot animations (S-0108 F3 and Import mapping).
- **Per-tick start, late start, moving.** The first per-tick start and the late start of an infantry unit or vehicle, as S-0215 claim 11 defines them. Moving means the current movement state read by S-0215 claim 11's readiness test, not remembered displacement earlier in the update. For a vehicle with the driving locomotor it means either a body still turning for any reason, or all three of: locomotor movement in S-0206's sense, a present heading point (S-0207 Terms), and a strictly positive integer movement step (S-0207 claim 2). S-0215 claim 11 owns the full readiness predicate, including its exceptions and other blockers. Claim 3 uses these terms in these senses.

## Behaviour

### Getting the order

1. **Who may be sent in.** The recycle order is S-0214's, and it gives the *eaten* behaviour
   with the building as destination and no target (table M):
   - an infantry unit may be sent only into a recycling building of its own house (row I26),
     without asking the building,
   - an engineer may be sent into any engineer-friendly recycling building at full health,
     which includes an allied one (row I5),
   - a vehicle may be sent into an *accepting allied* recycling building (table V rows n1 and
     n2), so the building's docking answer of claim 2 decides. A vehicle whose movement zone is
     *fly* is not offered the order.

   So infantry of an ally cannot be recycled in your building (except engineers), but an ally's
   vehicles can.
   - Backing: `measurable` (scenario G1)
   - Confidence: high (the rows are S-0214's)

2. **Docking answer of a recycling building.** A recycling building with neither absorption
   flag (S-0605 claim 1) *refuses* a would-be entrant *U* when any of these holds:
   - the building's owner does not count *U*'s house as allied (one-way, as in S-0605 claim 1),
   - the building is under construction, being sold or has not finished its build-up,
   - no link place is free for *U* (a place already held by *U* counts as free),
   - exactly one of the building and *U* is naval, unless *U*'s movement zone is amphibious,
   - *U*'s type hovers by balloon,
   - the building is switched off (glossary). Low power does not refuse.

   Otherwise it *accepts*. It tests no kind, size, passenger count or capacity, and, unlike an
   absorbing building, it does not refuse a controller that holds victims. A building that has
   both absorption and recycling features still refuses the wrong infantry or vehicle kind and
   a controller holding victims under S-0605 claim 1. If those tests pass, it accepts when either
   the absorption capacity test passes or its recycling feature applies. The link-place row does
   not apply to a building with either absorption flag. Because entrants never
   take a link place (claim 3), the link test refuses only when something else holds every link
   place.
   - Backing: `measurable` (scenario G2) except for occupied-link refusal and a holder's
     exemption, which are `unmeasured`: no combined recycling/docking fixture is yet
     established that fills all links while leaving a recycle query possible. G2 proposes
     that fixture conditionally, not as confirmed backing.
   - Confidence: high

### Approaching

3. **No contact.** An entrant never makes contact with the building and never takes a link
   place: it walks or drives to it as its *eaten* destination, and the building is never told
   that it is coming. The *eaten* behaviour is processed by S-0215 claim 28's table with
   S-0215's *eaten* differences: occupiers, assaulters and infiltrators are not building takers
   there, and the hunt rule never applies. With no destination, an infantry entrant falls back to
   rest by S-0215 claim 14, and a vehicle entrant by S-0215 claim 15, which queues nothing for a
   vehicle in *eaten*, so the vehicle stays in *eaten*. An entrant on a cell that holds a building
   is then sent S-0215 claim 28's scatter request.

   The approach does not always succeed. A unit ordered to a recycling building that stands behind
   a short line of buildings, with open ground round the line, can fail at once: the path
   computation fails, the unit does not start a move and its destination is cleared (S-0212 claims 13, 13a
   and 23, edge case "short line of buildings", scenario P12 run S). The failure follows from the
   corridor rules of S-0212 as written (claims 13, 13a and 23). Which alignments of the line against
   the level-2 sectors of S-0212 claim 13 fail depends on `quirks.corridor_split_probe` (default on,
   the original; with it off the failure is expected at fewer alignments, S-0212 claim 13a). A unit whose search failed this way, more than 1 cell from
   the building, is not handled by the no-destination fall-back above: it gives up as S-0212
   claim 23 row 2 gives (it leaves its team and queues *guard*; an AI house's unit is treated as
   human-controlled there). The queued *guard* is not held back by *eaten*: neither the infantry readiness test nor the vehicle
   one of S-0215 claim 11 mentions the *eaten* behaviour, so the ordinary start rule applies to it as
   to any queued behaviour. The unit leaves *eaten* and *guard* becomes current at the first
   per-tick start or late start (S-0215 claim 11) at which the unit is ready. A failed search that leaves the unit more than 1 cell from the building clears
   the unit's destination (S-0212 claim 23, row 2). Readiness at the late start uses the movement state that the update has left (S-0215 claim 11), not a record of earlier displacement. A cleared
   destination does not by itself stop a vehicle, which may finish the step it had begun (S-0212 claim 23, row 2: the unit stops when its current motion ends).
   If the unit is ready at that tick's late start under S-0215 claim 11's full predicate,
   *guard* is current when that tick ends, even if the unit displaced earlier in the update.
   Otherwise it remains queued until the first eligible start or late start that finds it ready. For a driving
   vehicle, a continuing body turn blocks this change, as does the conjunction of locomotor
   movement, a present heading point and a positive integer movement step (Terms).
   Earlier displacement alone does not block it, and lack of visible displacement alone
   does not make it ready. When a body turn ends by tick number, the first per-tick start
   of its completion tick already sees it ended (S-0215 claim 11), subject to every remaining blocker. A *guard* that becomes current at a first per-tick start is processed
   on that tick, and one that becomes current at a late start is first processed on the unit's next
   update (S-0215 claims 3a, 3b and 11). The queued *guard* is not dropped merely because
   *eaten* is current. Until a start succeeds, any due processing still uses *eaten*.
   Clearing the destination does not itself start *guard*. Infantry and vehicles have no
   *eaten*-specific readiness exception. A further vehicle readiness blocker is a
   vehicle that is in contact with a vehicle-factory building, or that is in contact with nothing and stands on a vehicle factory's
   exit cell (S-0215 claim 11): it is not ready while *guard* is the queued behaviour, so it stays in
   *eaten* with *guard* queued until that contact or position ends. Whether a recycle order can reach
   this state is not established (a vehicle just built, standing on the exit cell and given a recycle
   order whose approach fails at once would be one route), so the case is only noted.
   - Backing: `measurable` (scenario G9, approach-failure test; S-0212 scenario P12, run S, sets up
     the same order) for contact, fall-back, failure and queueing, and start timing when G9
     establishes readiness. The late-start comparisons involving earlier displacement or a
     retained positive movement step without displacement are `unmeasured`: G9 does not
     establish a repeatable order sequence producing either contrasting state. The turning-body
     and factory-exit/contact combinations after a failed recycle approach are also `unmeasured`:
     no setup establishes those combinations. G9's conditional observations are investigation
     proposals, not backing for these cases. Which shifts fail is tracked in Open questions.
   - Confidence: high (no contact; the destination is cleared on a failed search; that *eaten* does
     not block the queued *guard*, both readiness tests as S-0215 claim 11 states them); medium
     (that a short line makes the approach fail at once; the current-state readiness at the late start after an approach failure; the turning-vehicle case); low (which lengths and
     alignments fail)

4. **Onto the foundation.** The entrant may move onto the building's foundation cells: an infantry
   unit because the building is its mission target (S-0209 claim 4 row WP3), a vehicle by S-0209
   claim 5 rows VP6 and VP7. It is consumed on the first foundation cell it arrives in (claim 5),
   so it does not need to reach the centre. This presumes the approach is not cut short as
   described in claim 3 (a short line of buildings in the way).
   - Backing: `measurable` (scenario G1, positions)
   - Confidence: medium (which foundation cell, for a large building)

### Consumption

5. **The moment of consumption.** An entrant is consumed in the tick in which it arrives in a
   cell that holds the building it is bound for:
   - an infantry unit at the arrival update of a walking step (S-0208 claim 22),
   - a vehicle at a cell-arrival update (S-0417 claim 7),

   provided its current behaviour is *eaten* and the building in that cell is its destination
   or its target. Nothing else is tested at that moment: not the alliance, not the building's
   owner, not power or the switch, not a sale or construction in progress, not the building's
   health. A building that is being sold, is switched off or has changed owner since the order
   still consumes the entrant (`quirks.grinder_no_arrival_recheck`, Edge cases).
   - Backing: `measurable` (scenarios G1, G7 and G8), except arrival during
     construction, which is `unmeasured`: no setup establishes an already-bound entrant
     arriving while the same destination building is under construction.
   - Confidence: high

6. **Order of what an observer sees in that tick.** All of claims 7 to 11 happen in the tick of
   consumption, with these orderings:
   - the building's tag gets an entry raise (glossary) with the entrant as causing object while
     the entrant is still on the map (S-0524 claim 12),
   - every payment of claim 8 is credited before the entrant leaves the game, so a trigger that
     reacts to the entrant's destroyed-by-anything raise sees the building owner's balance
     already raised,
   - the entrant's own tag gets the destroyed-by-anything raise (S-0011 claim 19, whose event
     row includes no-hit destructions) before it leaves the game,
   - a controller entrant's victims are released (claim 9) when the entrant leaves the game,
     after every payment.

   The building's entry raise observes the unsynchronised stream before claim 7's response
   and death-sound choices; the entrant's destroyed-by-anything raise observes those choices
   already taken. The animation change has no specified order relative to payment.
   - Backing: `measurable` (scenario G6)
   - Confidence: high

7. **Sounds.** At consumption:
   - if the entrant's owner is the local player and its type has a death response list, one
     death response is chosen and played at the entrant's position, and then, for any owner, if
     its type has a death sound list one death sound is chosen and played there. The choice, the
     position and the stream are exactly S-0701 claim 20's *now dead* case (both positional,
     unsynchronised stream, F5),
   - the rules-wide enter-grinder sound plays at the entrant's centre position, if it is set.

   Each non-empty eligible list uses one raw unsigned 32-bit draw of the unsynchronised stream,
   choosing entry *x* mod *n* for draw *x* and list length *n* (integer remainder, entries numbered
   from 0). The response draw precedes the death-sound draw; an ineligible or empty list takes no
   draw. Passengers get no consumption sound of their own. A parasite's release effects are
   S-0614 claim 12's, not an extra death-sound choice here.
   - Backing: `measurable` (scenario G10)
   - Confidence: high

8. **The recycle payout.** The building's owner, as it is at the moment of consumption, receives
   the sum of these amounts, each added directly to its credit balance as a sale refund is
   (S-0408 claim 1), not to resource storage:
   - the entrant's refund,
   - for a vehicle entrant, the refund of each of its direct passengers and each of their direct
     passengers,
   - for a vehicle entrant with a parasite attached, the parasite's refund.

   Each refund is S-0408 claim 1's formula (F1 below) for that object's type, **computed as if
   that object's own current owner sold it**: the factors *a* and *b* and the use of the refund
   percentage are those of the object's owner, while the money goes to the building's owner.
   Health, veterancy, the price paid and a harvester's load play no part: a full harvester pays
   the same as an empty one. A building's owner recycling its own units therefore gets exactly
   what selling them would give, if they could be sold. An ally's unit is valued with the ally's
   factors.
   - Backing: `measurable` (scenarios G1, G3 and G4)
   - Confidence: high (formula, recipient, factors), medium (the harvester load, from the absence
     of any storage change)

9. **Controlled entrants and controller entrants.** A controlled unit (S-0601 claim 1) sent into a
   recycling building is **not released first**. It is consumed as an object of its current owner,
   the controlling house: its refund uses that house's factors (claim 8), and as it leaves the
   game it silently leaves its controller's list, freeing the slot (S-0601 claim 12). No release
   sound plays and it does not return to its original owner.

   A controller that is itself consumed leaves the game, which releases all its own victims by
   S-0601 claim 10 (mobile controller removed from the game), with the effects that claim gives.
   - Backing: `measurable` (scenario G5)
   - Confidence: high

10. **Passengers and a parasite of a vehicle entrant.** The passengers of a consumed vehicle are
    removed from the game with it, after their refunds are paid (claim 8), whatever their owner.
    The same holds for the passengers of those passengers. They are removed, not killed: no
    death effects, no raise on their tags and no sounds. Only these two passenger levels receive
    refunds (Edge cases).

    A parasite attached to a consumed vehicle is released by S-0614 claim 12's consuming-building
    row after its refund is paid: it is suppressed for 50 ticks and release is attempted.
    A non-naval parasite is removed silently by suppression. A naval parasite proceeds to
    S-0614 claim 13's exit test, but that test also removes it silently because the host's
    cell holds the building. Neither is refunded a second time.
    - Backing: `measurable` (scenarios G3 and G4). The no-refund removal of nesting deeper
      than two passenger levels (Edge cases) is `unmeasured`: no legal loading sequence
      establishing that nesting is known.
    - Confidence: high (passengers, parasite refund), medium (passengers of passengers and deeper nesting)

11. **Removal, not death.** The entrant leaves the game in the ordinary way for a removed mobile
    object. In particular:
    - S-0312's death effects are not invoked: no death weapon, no explosion of passengers, no
      survivors, crew, debris or death animation, and the attached-bomb rule is the one for
      removal other than death (S-0519 claim 7: the bomb is disarmed with no damage),
    - no house is credited with a kill and the entrant's house records no loss (S-0414 claim 21),
      and no unit-lost announcement plays,
    - the entrant leaves the counts of S-0414 claim 4 as an object that leaves play for good,
    - a magnetic lifter that is consumed ends its grip as a lifter that leaves play (S-0607 claim 12),
    - a consumed slave master ends its untransferred slave group (S-0312 claim 31),
    - every reference to the entrant (targets, destinations, contacts) is cleared, as for any
      removed object.
    - Backing: `measurable` (scenario G6) for the named removal effects and target clearing.
      Clearing every possible destination or contact reference is `unmeasured` here: G6
      establishes a targeting observer, not a legal fixture for each other reference kind.
    - Confidence: high

12. **The building's animation.** When a recycling building consumes an entrant while its
    *active-1* slot animation runs, *active-1* stops and *special-1* starts, in its damaged
    variant when the building's health ratio is at or below the yellow ratio and its normal
    variant otherwise. When *special-1* then ends (on its own, or removed by anything but the
    building stopping or replacing it), *active-1* starts again, damaged or normal by the same
    test, provided the selected *active-1* variant exists. With *active-1* not running at the
    consumption, the animation is left unchanged. If the selected *special-1* variant is absent,
    keep *active-1* running instead. These are display effects only; the original instead stops
    *active-1* without replacing it in that absent-animation case.
    - Backing: `measurable` (scenario G10) for consumption, natural completion and the
      missing-animation display fix. Restart after external removal is `unmeasured`: no
      data-only fixture is established that removes the slot animation without the building
      itself stopping or replacing it. The absent-*active-1* restart case is also
      `unmeasured`: no setup establishes that absence at completion while still allowing
      *active-1* to run at consumption and start *special-1*.
    - Confidence: high (the swap at consumption), medium (the restart)

13. **A destination that is not a recycling building.** A unit whose *eaten* destination is a
    building without the grinding flag is still consumed and paid for at that building by claims
    5 to 11, without the enter-grinder sound or the animation change. The order and fate
    decision described in claims 1 and 14 both require a recycling building, so neither
    establishes this case.
    - Backing: `unmeasured` (unreachable through orders. Test data cannot create it either,
      because every route to *eaten* checks the grinding flag)
    - Confidence: medium

### Searching for a recycling building

14. **Nearest recycling building (original only).** S-0601 claim 8's fate decision can send a
    controlled unit to the nearest recycling building of its current owner. In our engine this
    runs only for the neutral and special houses (S-0601 claim 8's design rule). The unit considers every
    recycling building its owner holds, whatever its state (power, switch, sale, docking answer),
    and picks the one with the smallest 3-D distance ρ₃ of S-0215 F4 between the two centre
    positions. This is the integer distance in leptons, including F4's floating-point root and
    truncation toward zero, so different exact distances can tie after truncation. Of buildings
    tied at that integer distance it picks the one its owner placed or gained most recently. It
    then takes that building as its destination and queues *eaten*, without an immediate start. When the owner has no recycling building, S-0601 claim 8 gives what happens
    instead.
    - Backing: `measurable` in the original with a computer player (scenario G11)
    - Confidence: high (search set, distance), medium (the tie order)

### While an entry is pending

15. **Interruptions.** Because an entrant holds no contact (claim 3), nothing the building does is
    passed on to it:
    - a new order to the entrant replaces *eaten* in the ordinary way (S-0213 claim 5),
    - when the building is destroyed or removed, the entrant's destination is cleared. Its next
      processing falls back to rest (claim 3): infantry by S-0215 claim 14, a vehicle stays in
      *eaten* with nothing queued (S-0215 claim 15) until it gets another order,
    - while the building is being sold, entrants are not sent away (they are not among S-0408
      claim 6's contacts). One that arrives before the building is removed is consumed and paid
      for, by claim 5,
    - switching the building off or a low-power state does not affect entrants already bound
      for it. Switching off refuses new vehicle orders through claim 2; low power alone does not,
    - a change of the building's owner (capture, test data) does not affect entrants: at
      consumption the new owner is paid (claim 8). S-0509 claim 14 concerns contacts only,
    - a blocked route is handled by claim 3's processing, which keeps the destination (S-0215 claim 28); a search that fails and leaves the unit more than 1 cell from the building (S-0212 claim 23, row 2, which includes the immediate failure of S-0212 claim 14) clears it (claim 3).
    - Backing: `measurable` (scenarios G7 and G8; G9 for failed-search clearing and queueing)
    - Confidence: high (sale, switch, destruction), medium (owner change)

## Formulas

All amounts are integers in credits. Rounding, number domains and the floating-point sites are
S-0408's Formulas, which this spec does not restate.

**F1. Recycle payout** (claim 8). For each valued object *X*—the entrant and, for a vehicle entrant, either of its two passenger levels or its attached parasite—determine the refund by applying S-0408 F1 to *X*'s type, using *X*'s own current owner as the seller. Thus *v* is *X*'s type refund override, *C* is its type cost field, and *a*, *b* and the use of *r* are taken from *X*'s owner exactly as S-0408 F1 takes them from the seller. The building's owner *H* at consumption receives the sum of the refunds for all valued objects. Calculate and add each refund separately, truncating each one before summing.

**F2. Link places** (claim 2): as S-0605 F2, max(*d*, 1) for the building's integer dock count
*d*. Exact integer comparison, no rounding.

## Constants

| Name | Value | Unit | Where |
|---|---|---|---|
| Parasite suppression on host consumption | 50 | ticks | claim 10 (S-0614 claim 12) |
| Grinding flag, absent-key default | no | boolean | Terms |
| Enter-grinder sound, absent-key default | none | sound | claim 7 |

## State and ordering

**Entry state per entrant.**

| State | Event | Next state | Effect |
|---|---|---|---|
| bound (in *eaten*, destination the building) | arrives in a cell holding that building (claim 5) | removed | consumption, claims 6 to 11 |
| bound | new order | per the order | *eaten* replaced (claim 15) |
| bound | building destroyed or removed | resting (infantry) or *eaten* with nothing queued (vehicle) | destination cleared, fall-back (claims 3, 15) |
| bound | building begins sale, switched off, low power, owner change | bound | nothing (claim 15) |

**Within the tick of a consumption.** The building's entry raise can still observe the entrant
on the map and the pre-payment balance; the entrant's destroyed-by-anything raise sees all
payments credited, before that entrant leaves play and releases any victims (claim 6).

**Several entrants.** No per-tick intake capacity delays arrivals. Payments and entry raises for
several entrants in the same tick follow their movement-update order (S-0203 claim 3), so a later
entrant's entry raise sees earlier entrants' payments. Removal during an entrant's turn is subject
to S-0203 claim 10's effect on which later objects get turns (claims 5 and 6; G9).

## Edge cases and quirks

- **No re-check at arrival (`quirks.grinder_no_arrival_recheck`, default on, the original).** The
  building tests nothing about itself or its owner when an entrant arrives (claim 5). A building
  that is being sold, is switched off, or now belongs to a house that does not count the
  entrant's house as allied still consumes the entrant and pays its owner. An infantry unit sent
  into its own building that is then captured pays the capturer. *Fixed behaviour:* at the moment
  of consumption the building gives the docking answer of claim 2 to every entrant, infantry
  included, with the link-place row left out. An entrant that is refused is not consumed, its
  destination is cleared, and it falls back to rest as in claim 15's destroyed case.
- **Valued with the entrant's owner's factors.** A house recycling an ally's vehicle, or a
  controlled enemy unit, is paid what that unit's owner would get for selling it (claim 8). This
  is the original's design, not a quirk.
- **Controlled units are not freed first.** Claim 9. The controller's slot is freed silently, and
  this follows from S-0601 claim 12 and is not a quirk.
- **Nesting beyond two levels.** Should a passenger's passenger itself hold passengers, those
  would be killed with no attacker and no refund when their transport is removed. The
  two-level refund limit is the original's payout scope, not an assertion about which
  transports the imported data permits. No optional alternative is specified here. This case
  is `unmeasured`, with medium confidence: no legal loading sequence establishing that
  nesting is known (claim 10).
- **Leave-grinder sound.** The rules-wide leave-grinder sound does not play during consumption
  (claim 7). Whether it is unused in every other situation is not claimed here.
- **No *special-1* animation.** Keep *active-1* running when the selected consumption animation
  is absent (claim 12). The original instead leaves that slot stopped, a display-only bug with
  no switch.

## Import mapping

| Key (where) | Feeds | Form | Absent |
|---|---|---|---|
| `Grinding` (rules, building type) | recycling building (Terms, claims 1, 2, 12, 13, 14) | boolean | no |
| `EnterGrinderSound` (rules, `[AudioVisual]`) | enter-grinder sound (claim 7) | sound name | none |
| `LeaveGrinderSound` (rules, `[AudioVisual]`) | read, not played during consumption (claim 7) | sound name | none |
| `NumberOfDocks` (rules, building type) | dock count *d* (F2) | integer | 1 |
| `TogglePower` (rules, building type) | whether the building can be switched off (claim 2), as S-0214 claim 13 reads it | boolean | yes |
| `Naval`, `MovementZone`, `BalloonHover` (rules, unit types and building type) | naval, amphibious and balloon rows of claim 2, as S-0605 claim 1 reads them | boolean, movement zone, boolean | as their owners give |
| `Soylent`, `Cost`, `RefundPercent` | refund override, cost field and refund percentage of F1, as S-0408 reads them | as S-0408 | as S-0408 |
| `ActiveAnim`, `SpecialAnim` and their damaged variants (art, building) | *active-1* and *special-1* (claim 12), as S-0108 reads them | as S-0108 | as S-0108 |
| `ConditionYellow` (rules, `[AudioVisual]`) | yellow ratio (Terms, claim 12), as S-0108 reads it | decimal or percentage ratio | 0.5 |

The death response and death sound lists of claim 7 are S-0701's. The enter-grinder sound is the rules-wide one. The same sound keys in a type section do not choose
the sound for a consumption. Absent values above are initial defaults; later absent keys keep held
values under S-0006 claim 3.

## Measurement scenarios

Use a small purpose-made skirmish map, game-speed index 2, no engine add-on and no mods (the named test files and named test-map overrides of a scenario excepted). Stock data apply
except named data-only override sections in the test map. Keep the director off.
Keep one bystander computer player outside the test area; void a run if an
unnamed bystander object affects that area or the bystander is defeated. Use scripted input and a
read-only recorder for ticks, unit positions, owners, behaviours, destinations, house balances,
trigger raises and the unsynchronised stream's state. Capture audio and images for sounds and
animation. Run each variant twice, recording rather than setting the random state. A contrary
recorded result falsifies each stated expectation unless the fixture failed to establish its
prerequisite.

**G1. Own infantry and vehicle** (claims 1, 4, 5, 8). House A owns a recycling building. Send in
an infantry unit of a type without a refund override, one with an override, and a vehicle.
Record the tick each enters a foundation cell, the tick it disappears and A's balance each tick.
*Expected:* each disappears in the tick it first arrives in a foundation cell, and A's balance
rises in that tick by exactly F1 for that type and A's factors. *Falsified by:* a payment that
depends on health (repeat with a damaged unit) or veterancy (repeat with an elite unit), or a
delay after arrival. Repeat with a full and empty harvester of the same type: equal payouts are
expected. To test eligibility (claim 1), let the local house own the infantry while an allied
bystander owns the building: an ordinary infantry unit has no recycle order, but an
engineer does when the building is at full health. Repeat with an enemy building, and with a named test-map vehicle in the fly movement zone:
neither offers the recycle order. For claim 4, record the arrival position for approaches along
each side and a corner, including a named test-map building with a larger foundation.

**G2. Docking answer** (claim 2). House B allied with A, house C enemy. Order a vehicle of B into
A's building while it is normal, switched off, under build-up and being sold, and with A at low
power. Order a vehicle of C. Order a test vehicle whose type hovers by balloon (the balloon-hovering override named above).
*Expected:* the cursor and order succeed only for B with the building normal or at low power.
*Falsified by:* acceptance while switched off, or refusal at low power. The asking house is the
local player in each order test. Use separate named test-map overrides for naval mismatch,
amphibious movement and balloon hovering. Expect mismatch to refuse except for amphibious
movement, and balloon hovering always to refuse. Use a large vehicle, a loaded vehicle, and a
controller holding victims: a pure recycling building accepts all three. Add an absorption flag
in a separate variant: a wrong-kind entrant and a controller with victims are now refused even
when recycling is enabled; an otherwise eligible entrant is accepted even when absorption
capacity is full. Observe the recorded docking answer, not just the cursor, for these tests.
The all-links-occupied case needs a combined repair/docking test building with recycling enabled:
hold every link with other docking clients, then query entry with a further vehicle. Expect refusal
until a link is free; a repeated query by a holder does not fail the link test. If that fixture
cannot establish full contact occupancy, it is inconclusive rather than evidence for that row.

**G3. Ally's loaded vehicle** (claims 8, 10). House B (allied, a country with a different cost
factor from A's, from a named test-map override) sends a transport with two infantry passengers of B into A's
building. *Expected:* A's balance rises by the sum of three F1 refunds computed with B's factors
and B's refund percentage use. No sounds for the passengers, no losses for B.
Variant for passengers of passengers (named test-map override): a transport type that accepts vehicles carries
a loaded troop vehicle. *Expected:* A is paid for all of them, each with its own owner's factors.

**G4. Infected vehicle** (claims 8, 10). A vehicle of A with a parasite of house C attached is
sent into A's building. *Expected:* A receives the vehicle's refund plus the parasite's refund
computed with C's factors, and the parasite is removed silently (non-naval) in the same tick.
In a named test-map variant use a naval parasite and a compatible host and recycling building:
expect the parasite's refund and 50 ticks of suppression as in S-0614 claim 12, but no surviving
parasite: S-0614 claim 13's host-cell building test prevents exit placement. A missing parasite
payment, a death-sound choice for it, or a surviving parasite falsifies the corresponding prediction.

**G5. Controlled entrant and controller entrant** (claim 9). (a) A's controller takes an enemy
vehicle of C and A sends it into A's building: the payment uses A's factors, no release sound
plays, C does not get it back, and the controller can take a new victim at once. (b) A's
controller holding a victim of C is itself sent in: the victim is released with S-0601 claim 10's
effects in the same tick, after A is paid.

**G6. Raises and counts** (claims 6, 11). Give the building a tag with an entered-by event and
the entrant a tag with a destroyed-by-anything event whose action records the building owner's
balance. *Expected:* both raise in the consumption tick, the building's raise names the entrant,
the recorded balance already includes the payment. Repeat G5(b) with the recorder logging
balance and victim ownership: release of a controller's victims must follow all payments.
The entrant's house's losses and every house's
kills are unchanged, no unit-lost announcement plays, and no death animation or survivor appears.
Also record the entry raise's balance and entrant position: it sees the pre-payment balance and
the entrant still in play. Record the unsynchronised stream at both raises and consumption sound
choices: the entry raise precedes those choices, the entrant's raise follows them. Give passengers
their own matching tags in G3: those tags do not raise.
Repeat with named test-map entrants having a conspicuous death weapon, debris and crew/survivor
settings, and with a planted bomb: no death damage, debris or survivors appear and the bomb is
disarmed. Check existence and on-map counts fall, and an observer targeting the entrant loses its
reference. A slave master and a magnetic lifter use the removal outcomes cited in claim 11.
Record their slaves' and lifted victim's states against those cited claims. Any contrary effect
falsifies the corresponding part of claim 11.

**G7. Sale while bound** (claims 5, 15). Order an infantry unit into the building from a few cells
away, then sell the building so that the unit arrives during the sale. *Expected:* the unit is
consumed and the owner paid. Repeat with the unit arriving after removal: it falls back to rest
and is not paid for.

**G8. Switch, power, destruction, owner change** (claims 5, 15). Bind a vehicle, then switch the
building off: it is still consumed. Bind a vehicle, then destroy the building: it stays in
*eaten* with nothing queued for at least 10 processings. With a named test-map override making the building
capturable, bind an infantry unit, capture the building with another house's engineer and let the
unit arrive: the capturing house is paid even though it is enemy to the entrant's house.
Repeat G1 at several positive building health ratios on both sides of the yellow ratio:
consumption and payout are unchanged. Separately lower power without switching off: the
pending entrant still enters. Give a new move order before arrival: the entrant follows it and
is not consumed. Block the route temporarily: the destination is retained until the obstacle is
removed, after which entry completes. These observations test all interruption rows.

**G9. No contact** (claim 3). Two vehicles of A are sent in at once with one link place. *Expected:*
neither waits for the other, and both are consumed on arrival. Record contact ownership and
verify neither entrant holds a link at any point. Where both arrive in one tick, the later entry
raise sees the earlier payment. In a separate test destroy the destination and leave the entrant
on a different building's cell: verify the scatter request and the infantry/vehicle fall-back of
claim 3. A held link, intake delay, wrong balance at the later raise, or missing fall-back falsifies
the corresponding expectation. Record the entrant's current and queued behaviour and its
processing ticks as well: a *guard* started at a first per-tick start can process that tick,
whereas one started late first processes on the next update. While readiness is blocked,
any due processing must still use the current behaviour.

In a separate test, the approach failure of claim 3. The setup is that of S-0212 scenario P12 run S:
open clear ground, a straight north-south line of 1 x 1 buildings of house A, of lengths 3, 5, 7 and 9,
with the line's middle at each of eight shifts of 0 to 7 cells along the line. A recycling building with
a 2 x 2 cell footprint stands just behind the line. House A gives two orders from 20 cells west of the
line's middle, with the goal 2 cells east of it: one to a vehicle, and one to a non-jumpjet infantry unit.
Each order is the recycle order into that building (no target). Each unit is also given a plain move to
the same cell on the same map, for comparison. Observable, for each order: whether the unit moves in the
tick of the order or its destination is cleared in that tick, and then, over about 10 processings after
the failure, its current behaviour, its queued behaviours and its destination on each tick. The
observable confirms when the queued *guard* of claim 3 becomes current: by the end of the failing tick for a unit that is ready at that tick's late start, and otherwise from the first eligible start or late start that finds it ready under S-0215 claim 11. Record earlier displacement separately from the movement state at the late start. For driving vehicles record the body turn, locomotor movement, heading-point presence and integer movement step. If a run displaced earlier but is ready at the late start, expect *guard* current by update end; if its current movement predicate still blocks readiness despite no displacement, expect *guard* queued. A run that does not establish either contrasting state is inconclusive for that comparison.
*Expected:* a recycle order and a plain
move pass or fail on the same placements; lengths 3 and 5 are passed; at least one 7-line and one 9-line
placement fails at once; the vehicle and the infantry agree on each map; the failed unit has *guard*
current by the end of the failing tick and nothing queued when it is ready at that tick's late start (verified with the full readiness predicate, not inferred from its being idle before the order); otherwise *guard* is current from the first eligible start or late start that finds it ready under S-0215 claim 11. A vehicle in contact with nothing and standing on the exit cell of a vehicle-factory building, or in contact with one, is excluded from this expectation, since claim 3 predicts it stays in *eaten* with *guard* queued. *Falsified by:* a failed unit that is ready at that tick's late start whose *guard* is not current at the end of that tick, a unit that is not ready there and whose *guard* is not current by the end of the tick on which its first eligible start or late start finds it ready, or a unit that is ready there still in *eaten*
with *guard* queued two ticks after the failure, any 3- or 5-line placement failing, every 7- and 9-line placement passing, a recycle
order and a plain move differing on one placement, the vehicle and the infantry differing on one map, or
nothing queued and nothing current after a failure.

**G10. Sounds and animation** (claims 7, 12). With the building's *active-1* running, send in an
infantry unit and then a vehicle of the local player's house; use G11's fate fixture separately for a non-local
entrant with both sound lists non-empty. *Expected:* the local
player's unit gives a death response and a death sound, the non-local entrant only a death sound, both at the
entrant's position, plus the enter-grinder sound. *active-1* stops and *special-1* starts (the
damaged variant with the building at or below the yellow ratio), and *active-1* starts again when
*special-1* ends. Read the unsynchronised stream around consumption: each eligible non-empty list
consumes exactly one raw draw, response before death sound, with the variants predicted by claim
7; empty or ineligible lists consume none. Repeat with *active-1* absent: consumption changes no
slot. With named test sounds and map overrides give the enter and leave sounds distinct,
recognisable clips; only the enter clip plays. In a separate variant omit the enter sound:
neither clip plays. Override a type's enter sound with a third distinct clip while retaining
the rules-wide clip: consumption still uses the rules-wide clip. With named test art omitting the selected *special-1* variant, the original stops *active-1*
and starts nothing; our engine keeps it running as the display-only fix. Test normal and
damaged selections separately. A wrong sound, draw count, variant, position or
natural-completion outcome falsifies the corresponding expectation. The external-removal
and absent-*active-1* restart cases remain unmeasured as claim 12 explains; removal by the
building's own slot replacement would not test external removal.

**G11. Nearest recycling building** (claim 14). In the original, a computer player
with a controller, fate weights set by a named test-map override so that only "go to the nearest grinder"
can be chosen, owns two recycling buildings at equal distance from the victim's spot, placed in a
known order, and a third nearer one that is switched off. It takes a unit. *Expected:* the unit takes the
nearest one even when switched off. In separate repeats leave it on but lower power,
begin selling it before takeover, or give its type an absorption flag that refuses the
victim's kind: it remains the chosen destination in each case. Record destination selection
before any sale removal; a run where it has already left play is inconclusive. With named
test-map terrain heights and positions, choose candidates whose planar and 3-D rankings
differ, and then candidates whose exact 3-D distances differ but whose truncated integer
distances tie. Expect the 3-D integer ranking and the latest-gained tie winner respectively.
Remove the nearer building and repeat the takeover to test
the tie: expect the one placed last. Reverse placement order and repeat; also test ownership gain
order with a capturable test type. Compare recorded integer distances rather than apparent screen
distances, and record that *eaten* is queued, not started immediately. Choosing the farther building,
choosing the earlier-gained equal-distance building, or immediately starting *eaten* falsifies the
respective prediction.

## Open questions

- Which foundation cell a large building consumes in, and whether a vehicle approaching along
  the edge can be consumed in a cell it only touches (claim 4). Medium confidence.
- An infantry entrant with a parasite attached is predicted to pay no parasite refund.
  Its parasite is released by S-0614 claim 15 rather than claim 10's vehicle-specific
  release. Backing: `unmeasured`; no data-only fixture has established parasite attachment
  to infantry followed by consumption. Confidence: low. Whether such a fixture can be
  established remains open.
- Whether the harvester load really pays nothing (claim 8) should be confirmed by a measurement:
  recycle a full harvester and an empty one (scenario G1 variant).
- Which lengths and alignments of a short line of buildings make the approach fail. Low confidence. Tracked by Q-1280.
- Whether a recycle order can reach a vehicle standing on a vehicle factory's exit cell whose approach fails at once (claim 3). Not established.
- Whether a vehicle whose body is turning, in place or to face a target, when its approach fails keeps *guard* queued until the turn ends, as claim 3 predicts from S-0215 claim 11. No scenario yet fixes its setup, so the case is `unmeasured`, medium confidence.
