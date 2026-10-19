---
id: S-0507
title: Transports (eligibility, capacity, boarding, passenger order, unloading, open-topped passengers)
topic: combat
revision: 9
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-19
measurements: []
requests: [Q-0700, Q-0701, Q-0702, Q-0703, Q-0704, Q-0705, Q-0706, Q-0672, Q-0673]
supersedes: []
---

# S-0507: Transports (eligibility, capacity, boarding, passenger order, unloading, open-topped passengers)

## Summary

How a unit becomes a passenger of a vehicle or aircraft transport and how it gets out again:
who may board and how capacity is counted, the exchange between the transport and a unit that
wants to board, the order in which passengers are held and let out, how a vehicle transport
chooses where to unload, its cadence and the places where it may not unload, how a transport
aircraft lets its passengers out, and what changes for passengers of an open-topped transport
while they ride, board and leave.

Skirmish only; network games and campaigns are out of scope. The original's team joins are
out of scope too; unloading gives no team membership in our engine.

This spec does not repeat what other specs own:
- Cursor and order verdicts for entering and unloading: S-0214 (transport verdicts *x* and *q*,
  table VD, row AC7).
- Passengers' place in the update order, and their turns while inside: S-0203 (claim 3).
- Death of passengers with their transport, passengers killed by a high fall, and passengers who
  escape a destroyed transport: S-0312 (claims 5, 12 and 16). Removal of passengers still
  aboard after those effects, or without a transport death, is claim 16 here.
- The gunner weapon switch and the weapon of an open-topped passenger: S-0308 (claims 5, 6 and 10).
- Combat effects of firing from inside an open-topped transport:
  - the damage multiplier: S-0302 (claim 2),
  - the range bonus and the transport's shortest range: S-0307 (claims 5 and 13),
  - experience: S-0311 (claim 8),
  - fire points: S-0313 (claim 5),
  - target choice when the transport is mind-controlled: S-0309 (claim 10).
- The target hand-down from an order to an open-topped transport: S-0213 (table T3, part (f),
  and claim 8).
- The *enter* behaviour in general and the standard delay: S-0215 (claims 6 and 29).
- Aircraft flight, landing and pads: S-0501.
- Buildings that take units inside (tank bunkers, garrisons and absorbing buildings): the
  structures and garrison specs.
- Deploy-fire vehicles that fire at their own cell instead of carrying passengers: S-0517 claim 2.
- The absence of an *unload or deploy cycle* for vehicles, including vehicle transports:
  S-0517 claim 15. That cycle is distinct from the passenger-unloading phases below.

## Terms

- **Transport.** A vehicle or aircraft whose type has a capacity above 0 (Import mapping). A
  *vehicle transport* is a vehicle that is a transport, an *air transport* an aircraft that is one.
- **Allied for boarding.** The transport owner's house counts the candidate's house as allied
  (including itself); this is a one-way test.
- **Capacity.** A transport type's imported passenger count: an integer.
- **Size, size limit.** Every infantry and vehicle type has a *size*, a floating-point number. A
  transport type has a *size limit*, also floating point: the largest size of one passenger it
  takes.
- **Load.** The sum of its passengers' sizes that a vehicle transport counts (claim 1, Formulas
  F1).
- **Passenger order.** The order in which a transport holds its passengers. *Position 1* is the
  front of the order (claim 3).
- **Would-be passenger.** A unit in *enter* whose contact is a transport.
- **Unloading phases.** The four phases of a vehicle transport's *unload* processing: *settling*,
  *turning*, *letting out* and *finished* (claims 8 to 12, table U).
- **Kept passenger.** For a vehicle transport whose type's turret count is above 0, the passenger
  that stays inside during an unload (claim 11).
- **Tunnel-marked cell.** A cell whose land type is tunnel and which is recorded as the entrance of an existing map tube (S-0212 claim 7). Tunnel land alone does not qualify.
- **Tunnel edge for unloading.** A cell C for which, in map north or map west, either its first neighbour is tunnel-marked and its second is not, or its second is tunnel-marked and its third is not. All cells in the chosen sequence through the non-marked cell must exist. No south, east or diagonal test is added. This includes non-tunnel-land cells one or two cells south or east of the last marked cell of a tunnel band.
- **Rear direction.** The step direction derived from body facing by F3; the half-turn
  adjustment is one facing unit short of an exact half turn.

## Behaviour

### Who may board

1. **Capacity of a vehicle transport.** A vehicle transport accepts a candidate only when both
   hold:
   - its load plus the candidate's size is at most its capacity, and
   - the candidate's size is at most the transport's size limit.

   Both comparisons are made in floating point. The load is a whole number: each passenger's size
   is added to it in turn and the sum is truncated toward zero after each addition (Formulas F1).
   For non-negative sizes and representable loads, a fractional size counts for less than its value: passengers of size 0.6 add nothing
   to the load at all (Edge cases, `quirks.transport_load_truncated`). The candidate's own size
   enters the comparison untruncated.

   The size limit's absent-key default is 0, and nothing replaces it. A transport type whose data
   gives no size limit therefore accepts no candidate of positive size.
   - Backing: `measurable` (scenario T1)
   - Confidence: high

2. **Capacity of an air transport.** In ordinary loading, an air transport counts heads, not sizes. It accepts a
   candidate allied with its owner while the number of its passengers is below its capacity, and
   refuses one when that number has reached its capacity. It gives neither answer (claim 4) to a
   candidate that is not allied with its owner. Neither the candidates' sizes nor its size limit
   play a part, and claim 4's vehicle-specific exclusions do not apply. Flight-specific states
   suppressing the query are not specified here (Open questions). So a mind-controlled
   unit may board an air transport, and it is released from the control when it boards (claim 6).
   - Backing: `measurable` (T2) for ordinary loading; `unmeasured` for flight-state query
     suppression, whose complete observable conditions are not traced here.
   - Confidence: medium

3. **Passenger order.** A unit that boards goes to position 1, and every passenger already
   inside moves one position back. Passengers leave from position 1. So a transport lets its
   passengers out last-in first-out, and position 1 is always the passenger that boarded most
   recently (after claim 10's put-back, the one put back). S-0313 claim 5 and S-0312 claim 5 read
   this order.
   - Backing: `measurable` (scenario T3)
   - Confidence: high

4. **Who a vehicle transport will not take.** Asked whether a candidate may board, a vehicle
   transport whose type has a capacity gives the docking answer *neither* (S-0214 Terms, *docking
   answer*), rather than accepting or refusing, when any of these holds:
   - the candidate is not allied with the transport's owner (a house is allied with itself),
   - the transport is being erased (glossary *being erased*),
   - a building occupies the transport's cell,
   - the candidate is mind-controlled,
   - the candidate is an infantry unit or vehicle with a parasite inside it (S-0614 claim 1),
   - the candidate is a mind controller that controls at least one object now.

   Otherwise it accepts or refuses by claim 1. The loading exchange of claim 5 does not ask this
   question, so a candidate excluded here can still walk up to the transport. It is turned away
   only when it arrives (claim 6), where *neither* counts as a refusal. S-0214's transport verdicts give
   no *enter* for a *neither* answer, so a player's cursor does not offer such an entry. Only
   an already-established attempt whose candidate becomes excluded can expose this result.
   No trigger forcing an otherwise illegal entry is established here.
   - Backing: `measurable` (T4) for reachable cursor and arrival tests. `unmeasured` for
     building-cell and unreachable exclusion combinations: no normal-order witness is established.
   - Confidence: high

### Boarding

5. **The loading exchange.** At each processing of *enter*, the passenger asks its contact to
   load it. This is separate from the eligibility question of claims 1, 2 and 4.

   A vehicle transport in *move* with a destination or a moving locomotor refuses. Otherwise
   a requester other than its current contact must pass claim 1's tests to proceed. With no
   current contact, an eligible requester becomes its contact.

   With no destination, the transport takes the requester as its destination when it stands
   on water or their cells have different reach regions for the requester's movement zone.
   This reply is not acceptance: an untethered requester breaks contact and falls back to
   rest, although the transport has acquired that destination. Boarding on the same attempt
   is not guaranteed.

   In other cases it accepts when claim 1's tests pass. With its locomotor stopped, body not
   turning, no tether, and a requester not moving toward a destination, it calls the requester
   to its own cell. Failure to take that destination ends contact. An already-contacted
   requester failing the capacity tests receives neither answer.

   Only acceptance, or an existing tether, keeps the passenger in *enter*. Every other answer
   breaks contact and falls back to rest (S-0215 claim 29). With neither contact nor a pending
   entry partner it also falls back to rest. Processing ends with the standard delay
   (S-0215 claim 6).

   For a driving or ship passenger, a destination naming the transport is an object goal,
   not a cell goal. S-0207 claim 26 governs its retention: with no active track, occupying
   the cell of its supplied movement point alone clears neither the object goal nor the
   locomotor destination point, whether the transport is moving or stationary. The resting
   passenger's exact-point *guard* exception still applies. This does not override this
   claim's loading replies, contact loss or fall-back to rest. A later assignment naming
   a cell instead uses that claim's cell-goal cleanup, not its object-goal retention.

   An air transport accepts a loading request while its count is below its positive capacity,
   even while moving. It calls a passenger not currently moving toward a destination only
   while its own locomotor is stopped. Full aircraft give neither answer. The loading exchange
   does not itself test alliance or claim 4's exclusions.
   - Backing: `measurable` (T5 and T6) for the approach, ordinary stopped/moving and
     capacity-change exchanges; S-0207 claim 26's D31 for transport-goal retention.
     `unmeasured` for destination-assignment failure, loss of both
     entry partners, existing-tether exceptions and the precise turning/contact gates:
     no isolating ordinary-input scenario is established for these combinations.
   - Confidence: high for replies; medium for reach-region interpretation

6. **The moment of boarding.** A would-be passenger boards when it arrives at the transport's
   cell while the transport is its destination. A vehicle boards only while its current behaviour
   is *enter* and its contact is a transport. At that moment the transport answers again by claims
   1, 2 and 4. This arrival-time boarding or refusal is separate from S-0207 claim 26's
   locomotion cleanup: retaining an object goal on mere cell equality does not suppress
   either outcome or guarantee that the passenger remains in *enter*.

   An infantry unit's qualifying arrival with the transport as destination gives one
   entered-by raise on the transport's current tag, with the infantry unit as causing object,
   before that final acceptance test (S-0524 claim 14). A refused boarding can therefore
   already have caused entry-trigger effects. Receiving the order alone gives no such raise.
   Vehicle boarding gives no entered-by raise on the transport's tag. Neither kind receives
   an own-tag destroyed-by-anything raise merely for becoming a passenger: it remains an
   object. Ordinary unloading from either vehicle or air transports adds no entered-by raise
   merely for leaving. These are object-tag rules, not the cell-tag rules encountered during
   approach; trigger matching and latches remain S-0011 claims 8 to 12.

   **If it accepts**, in the same tick the passenger:
   - forgets the target it had stored for later use,
   - leaves the bridge-deck layer if it was on it,
   - resets its gattling spin value and stage to 0 (S-0609 claim 11),
   - is released from any mind control that held it,
   - leaves the map (S-0203 claim 3 for whether it stays in play),
   - is recorded as riding in this transport, and goes to position 1 (claim 3).

   If the transport is a gunner type, this is the boarding event of S-0308 claim 6. If the
   boarding unit is itself an open-topped transport, its own passengers lose their targets.

   Successful boarding ends the passenger's current and any stranded earlier drain links
   immediately, including when the receiving transport is open-topped (S-0608 claim 17).
   Turns while riding do not preserve those links, and unloading does not restore them.
   The ended links' effects and the later house power update follow S-0608 claim 14.
   Failed boarding does not itself end a drain link.

   Boarding and ordinary unloading within the usable area preserve the passenger's cloak
   state and stage (S-0516 claim 16). Elapsed game ticks consume its cloak delay during the
   ride; boarding neither resets nor pauses it. A stored passenger's fading does not catch
   up on unloading: an overdue stage permits at most one step on its next ordinary update.
   Ordinary unloading does not force full cloak. S-0516 claim 16 owns these effects and
   their measurement scenarios.

   **If it does not accept** (a refusal or *neither*), an infantry unit gives up its destination,
   queues *guard*, and scatters (glossary *scatter*) even if it would otherwise not.
   A refused vehicle likewise clears its destination, queues *guard*, and receives a forced
   scatter request.
   - Backing: `measurable` (T2, T3, T4, T6 and T12). T3 tests boarding resets and
     notifications; notification rules also use S-0524 claim 14's scenario C16.
     S-0516 claim 16's scenarios test cloak persistence and timing.
     S-0608 claim 17's D13 tests immediate drain cleanup, open-topped boarding and no
     restoration on unloading; an unreachable linked-boarding fixture is inconclusive.
     Drain-link preservation on failed boarding is `unmeasured`: no scenario here isolates
     a refused arrival while the link is still live from its independent position check.
   - Confidence: high for acceptance; medium for the refused infantry's reaction

### Unloading a vehicle transport

7. **When a vehicle transport may unload.** An unload order to a vehicle transport is refused (it
   lacks deploy readiness, S-0214 Terms) when any of these holds:
   - it stands on a tunnel-marked cell or a tunnel edge for unloading (Terms),
   - it is disabled,
   - it is inside a tube,
   - it is on a bridge,
   - its own cell, or any of its four straight neighbours (step directions 0, 2, 4 and 6), carries
     a bridge deck,
   - its cell's land type is water,
   - it stands on a building's cell, unless that building is a tank bunker.

   In the original the water refusal applies only to human-controlled houses. In our engine it
   applies to every player house. Neutral and special houses retain the original's non-human
   exemption. A computer player's habit of unloading
   on water is a behaviour our AI may reproduce by moving to shore first (claim 8).
   - Backing: `measurable` (T7) for bridge, water, disabled and tunnel-marked/edge tests. `unmeasured`
     for tube-interior and non-bunker building-cell readiness: no normal-input witness is established.
   - Confidence: high for bridge, water, building and tunnel-edge geometry; medium for tube

8. **Settling.** The first phase of a vehicle transport's *unload* processing, outside a tank
   bunker (a bunker unload follows the structures rules instead):
   - While its locomotor is moving, it waits 10 ticks and checks again.
   - Standing on water with no destination, it uses S-0212 claim 25's nearby-cell search centred
     on its own cell: locomotion class *wheel*, movement zone *normal*, no reach-region restriction,
     ground mode, 1 × 1 footprint, bridges allowed, no height, no-overlay or burrowing condition,
     and no reference cell. Collection order, visibility preference, 24-candidate limit and
     selection by search tick modulo preferred candidate count are S-0212 claim 25's; there is
     no nearest-distance tie rule and no random draw. The chosen cell becomes its destination
     and it waits 10 ticks. This does not prove that the transport can reach that cell.
     With no result, the original still assigns the placeholder at map cell coordinates (0, 0)
     and waits 10 ticks: it does not continue to claim 9 in that processing or end the unload.
     Subsequent behaviour depends on destination retention and movement; another search occurs
     when it is again stopped on water without a destination. This holds whoever queued *unload*
     (claim 7 refuses only the order).
   - Otherwise it chooses an unloading cell (claim 9). If it has no passengers, or no cell is
     chosen, it queues *guard*, waits one standard delay, and the unload ends with nobody out.
   - Otherwise it turns its body so that its rear points at the chosen cell. A train instead
     turns to its present facing rounded to 8 steps (S-0204 claim 11), so an exact 8-step
     facing stays unchanged. It enters *turning* and is processed again after 1 tick.
   - Backing: `measurable` (T8 and T9), except shore approach is `unmeasured` until T9's
     ordinary-order/map witness is established; human unloading orders on water are refused.
   - Confidence: high

9. **Choosing the unloading cell.** The transport scores each of its 8 neighbouring cells. The
   score of the neighbour in step direction *d* is Formulas F2:
   - **+128** when all of these hold:
     - the cell's land type lets infantry walk on it (its foot land multiplier is not 0),
     - no vehicle or building occupies it,
     - not all of its infantry positions are taken,
     - the ground-layer infantry, vehicle, aircraft or building selected below, if any, is allied
       from the transport owner's side (not every occupant is tested independently),
   - **−128** otherwise,
   - **minus** the gap between that direction and the transport's rear facing,
   - **minus a further 100** for step direction 4 (map south).

   The alliance test selects the ground-layer combat object nearest to the cell's top corner,
   using only its within-cell map coordinates: distance is the Euclidean length in leptons,
   calculated in double-precision floating point and truncated toward zero to integer leptons before comparison.
   Equal integer distances favour the object most recently added to that cell layer; a building
   follows mobile objects in this preference. Terrain objects do not supply an owner for this test.
   It is not the house recorded as holding the infantry positions. When no object is selected,
   the alliance condition passes automatically. Thus right and left standing positions tie, whereas the top
   position is nearer than either (S-0208 table P).

   The highest score wins, and ties keep the lower direction number. A cell is chosen only when the
   best score is above 0. So the transport unloads behind itself where it can, avoids south, and
   gives up when no neighbour is open. The gap is not reduced modulo a turn (Edge cases,
   `quirks.unload_facing_gap_no_wrap`).
   - Backing: `measurable` (scenarios T9 and T13)
   - Confidence: high for the formula and selected-object rule; medium for mixed-occupancy witnesses

10. **Letting out.** After *turning* ends (claim 12), the transport lets out one passenger per
    processing, from position 1. Each processing ends with the standard delay (S-0215 claim 6).

    For passenger *P* it tries exit directions, counting clockwise from the rear direction
    (Formulas F3):
    - **First round:** all 8 directions, the rear direction first. A direction serves when *P*
      could move into both the neighbouring cell and the cell beyond it in the same direction:
      the movement specs' entry test (glossary *entry outcome*, S-0209) for *P* finds each of
      them open, with nothing in the way.
    - **Second round** (only if the first finds none): the 7 directions after the rear direction.
      The rear direction itself is not tried again (Edge cases,
      `quirks.unload_second_round_skips_rear`). A direction serves when *P* could enter the
      neighbouring cell.
    - In both rounds a neighbouring cell that carries a bridge deck is passed over.

    *P* is put on the map facing the exit direction:
    - An infantry unit uses the serving neighbour's centre as desired point and S-0208 claim 3's
      occupant-aware position choice. It takes one scenario ranged draw over 0 to 3. Draws 0 and 1
      prefer right, left, bottom; draw 2 prefers left, bottom, right; draw 3 prefers bottom, right,
      left. The first free position in that preference wins. Neither centre nor top is chosen,
      even when free. No available position means placement failure. This choice is made once,
      not again during placement.
    - A vehicle uses S-0212 claim 25's nearby-cell search centred on the serving neighbour:
      its own locomotion class, fixed movement zone *normal*, no reach region, ground mode,
      1 × 1 footprint, bridges allowed, no height, no-overlay or burrowing condition and no reference
      cell. S-0212 claim 25 supplies collection, visibility preference and search-tick-modulo
      selection, with no random draw. It appears at the selected cell's centre, at world height
      0 leptons before normal placement adjusts it. The subsequent movement destination remains
      the exit-round cell below, even if the placement search selected another cell.
      A failed search supplies the placeholder at cell (0, 0), not permission to
      put it at the serving neighbour's centre.

    **On success:**
    - *P* is no longer a passenger,
    - if the transport is open-topped and *P*'s owner differs from the transport's, *P* loses its
      target,
    - *P* queues *move*, to the far cell (first round) or to the neighbour cell (second round),
    The original also adds *P* to its team when it has one; that effect is out of scope.

    **On failure:** *P* stays inside at position 1 and the processing ends. A gunner transport's
    weapon number becomes *P*'s mode index, subject to S-0308 claim 5's clamping and
    charge-turret exception, even when others remain (an exception needed in S-0308 claim 6,
    `quirks.unload_failed_exit_gunner_handover`). For its only passenger,
    removal and replacement also restart S-0308 claim 6's gunner hand-over.
    The next processing tries *P* again.
    - Backing: `measurable` (scenarios T3 and T10) for exit order, rounds, deck exclusion,
      placement, cadence and the multi-passenger gunner change. `unmeasured` for the
      single-passenger failed-exit hand-over restart: no scenario here isolates its effect
      from the unchanged weapon number. The vehicle-placement search's no-result case is also
      `unmeasured`: no ordinary-input witness separates it from the prior entry refusals.
      T13 tests the position preferences and nearby-search settings.
    - Confidence: high for the order, the rounds and the cadence; medium for which entry
      outcomes count as open; high for the bridge-deck check and gunner weapon-number change

11. **The kept passenger and the end of unloading.** A transport whose type's turret count (Import
    mapping) is above 0 keeps one passenger inside when it unloads, unless exactly one passenger is aboard when
    the unload starts. The passenger kept is the last one in the order (the one that boarded
    first), because passengers leave from position 1.

    When the number of passengers inside is no more than the number to keep, the unload enters
    *finished* and waits one standard delay. At its next processing the transport queues *guard*
    with the allowance to interrupt movement of S-0215 claim 11. That *finished* processing
    itself also ends with the standard delay of S-0215 claim 6, including its scenario random
    draw. It is not a 1-tick completion and does not omit the delay draw when it queues *guard*.
    - Backing: `measurable` (scenario T10)
    - Confidence: high

12. **Turning.** At each processing in *turning*, a transport whose body is still turning waits
    one standard delay (S-0215 claim 6) and checks again. Once its body has stopped turning, it
    enters *letting out* and is processed again 1 tick later. So the first passenger gets out 1
    tick after the first processing that finds the turn finished.
    - Backing: `measurable` (scenario T8)
    - Confidence: high

### Unloading an air transport

13. **Air transports.** Ordinary passenger unloading has *selection*, *approach*, *stopped check*,
    *release* and *reselect* phases, initially *selection*. A carrier aircraft lifting a vehicle
    underneath uses different unloading, outside this claim. All processings below end with
    S-0215 claim 6's standard delay, except *stopped check* and *reselect*, which return 1 tick.

    | Phase | Condition | Result |
    |---|---|---|
    | selection | height above ground 0 leptons, forward pitch 0, and no destination or exact world-position equality with its destination | release at a later processing |
    | selection | above ground, no destination, dropship type | seek an own usable dock of its imported dock types by S-0501 claim 21's pad selection; if none, use S-0501 claim 41's landing spot; set the result as destination and remain in selection |
    | selection | no destination, not the preceding case | stopped check |
    | selection | destination fails the landing test of S-0501 claim 40 | approach when current height equals the type's cruise height, otherwise reselect |
    | selection | destination passes that landing test | replace destination as below; remain in selection |
    | approach | locomotor reports stopped | stopped check, unless the next condition applies |
    | approach | destination fails the landing test of S-0501 claim 40 | selection, overriding the preceding transition |
    | stopped check | locomotor reports stopped | release; next processing in 1 tick |
    | stopped check | locomotor still moving | stay here; next processing in 1 tick |
    | reselect | any | selection; next processing in 1 tick |
    | release | tethered | nobody released |
    | release | no passengers | fall back to rest |
    | release | passenger present, not tethered | attempt position 1 as below; fall back to rest if now empty |

    The stopped answer is S-0501 claim 40's unfinished-flight/pitch test, not merely zero speed.
    In particular, the no-destination non-dropship case does not seek a landing spot: a stopped
    airborne aircraft can proceed to release. Unload does not itself add a command to start
    descending. A destination starts ordinary flight guidance and landing under S-0501 claims
    5 and 6; merely entering *approach* does not mean touchdown has occurred.

    The landing test uses the destination object for the competing-aircraft comparison. Its height
    comparison uses integer leptons above terrain.

    **Replacement destination.** With no original-team membership, use map waypoint 700 as the
    centre of S-0501 claim 40's replacement-cell search, including that claim's circle samples,
    scenario draws and first-accepted tie rule, with integer radii 0 through 31 cells.
    An undefined marker supplies cell coordinates (0, 0), not the aircraft's own cell or its
    previous destination. The search still runs around that centre: one scenario ranged draw
    over 0 through 7 per attempted radius, including radius 0 even when all its samples are
    unusable. Exhausting radii 0 through 31 therefore takes 32 such draws. A successful search
    uses its first accepted cell. Failure retains the search centre as destination, not the
    previous destination: with an undefined marker this is the placeholder cell at (0, 0).
    S-0501 claim 41's end-of-update placeholder cleanup still applies, with the resulting
    fall-back to rest governed by S-0501 claim 18; this is the destination immediately after
    selection, not a promise that it survives that update. Initial map
    loading skips the search and its draws, retaining the centre (S-0501 claim 40).
    The original can instead use a passenger's team waypoint; that team choice is outside
    scope. Passing S-0501 claim 40's landing test, rather than failing it, causes this replacement
    (`quirks.air_unload_reselects_clear_destination`).

    **Release placement.** Position 1 is removed from the passenger order for this attempt.
    Its desired world position is the aircraft's exact world position, not its cell centre.
    For the exit facing and subsequent *move* destination, take the first neighbour whose entry
    outcome for this passenger is *clear*, in step-direction preference 4, 5, 3, 7, 1, 0, 6, 2.
    These tests have no entry direction or requested level. If none is clear, direction 4 and
    the last tested neighbour (direction 2) respectively supply the facing and movement destination.
    Facing is the chosen direction times 8192 facing units, not the aircraft's body facing.
    The aircraft itself does not obstruct this placement; other occupants still count normally.
    Placement distinguishes passenger kinds and integer world height:

    | Passenger and aircraft height | Position on successful placement | Standing-position choice |
    |---|---|---|
    | infantry, exactly at terrain surface height at the aircraft's horizontal point | S-0208 claim 3's chosen horizontal position in that same cell, retaining terrain height at the aircraft's original horizontal point | occupant-aware, excluding only the aircraft; a centre-class desired point takes one scenario ranged draw over 0 through 3 with claim 10's preferences, a side-class point uses its non-random row; refusal means placement failure |
    | infantry, above terrain surface height | aircraft's exact horizontal position and world height | none, and no standing-position draw; no projection to the surface below |
    | vehicle, at or above terrain surface height | aircraft's exact horizontal position and world height | none, no standing-position draw and no nearby-cell search |

    For both kinds the ordinary placement entry test must also find the aircraft's own cell
    *clear*, with no requested entry direction or level. A clear exit neighbour alone does
    not establish this. In an ordinary active-game release of a live off-map vehicle passenger,
    this own-cell entry refusal is the placement obstacle: there is no additional infantry
    position-capacity test or vehicle-specific exit search. General inability to place an
    object still counts as failure, not permission to bypass entry. Placement never lowers
    either kind to terrain: if the supplied world height is below terrain height at its
    horizontal point, it is raised to that terrain height, preserving horizontal coordinates.
    These height comparisons and choices are integer leptons, without rounding. The surface
    infantry choice is not the serving-neighbour placement search of claim 10.

    Vehicles use exactly the same exit-facing and movement-neighbour preference and all-blocked
    fallback as infantry above. Neither kind inherits the aircraft's body facing. This describes
    the appearance instant, before subsequent locomotion, falling or landing.

    Successful placement queues *move* toward that neighbour; there is no separate scatter request.
    Failed placement leaves the unit off-map, removed from the passenger order: it is not put
    back for a later retry and this release does not award a kill. The original's team join is
    out of scope. The lifetime of a failed, off-map unit remains unmeasured (Open questions).
    - Backing: `measurable` (T11, T13 and T15) for ordinary release, height/kind placement,
      undefined-marker replacement and flight phases.
      `unmeasured` for tether inhibition until an ordinary-contact witness is found, the
      all-neighbours-blocked fallback and failed-unit lifetime; T13 proposes discriminating runs.
      Initial-loading search suppression and below-terrain release clamping are also
      `unmeasured`: no ordinary map-loading unload or below-terrain release witness is
      established by T15.
    - Confidence: high for phase delays, non-dropship branch, direction preference,
      height/kind placement and undefined-marker centre, failure retention and draw count,
      and medium for the all-blocked fallback

### Open-topped transports

14. **Which transports are open-topped.** A transport is open-topped when its type has the
    imported open-topped flag. Vehicles and aircraft can both have it. While inside, its
    passengers are in play and take turns (S-0203 claim 3). They choose and fire weapons by the
    combat specs listed in the Summary.
    - Backing: `measurable` (scenario T12)
    - Confidence: high

15. **What an open-topped transport passes to its passengers.**
    - **Position.** Whenever the transport moves to a new position, its passengers' positions
      become the transport's position at once. Range and fire-point rules (S-0307 and S-0313) see
      them where the transport is.
    - **Targets given.** The transport gives its target to its passengers when an order sets it
      (S-0213 table T3, part (f), and claim 8).
    - **Targets taken away.** Every passenger loses its target when the transport leaves the map
      for any reason, for example when it boards another transport (claim 6) or enters a
      building.
    - **Getting out** (claim 10). A passenger whose owner differs from the transport's loses its
      target. Any other passenger keeps it.
    - Backing: `measurable` (scenario T12)
    - Confidence: medium

### Passengers when their transport is removed

16. **Removal cleanup.** Any passengers still carried when a transport is removed from play
    are killed, not released or silently preserved. Their deaths have no attacker credit and
    show no independent death effects, as in S-0312 claim 5's passenger destruction, but with
    no attacker. Nested passengers are lost too. This covers removal without a killing hit,
    an aircraft killed on the ground, a crashable vehicle on the ground that cannot start a crash,
    and passengers still aboard when a sinking vessel finally leaves play. It does not undo
    earlier passenger escapes or attacker-credited deaths under S-0312 claims 5, 12 and 16.
    A completed removal is distinct from a transport merely leaving the map to board another
    transport or enter a building: those events preserve its carried passengers (claims 6 and 15).
    Destruction escapes and whether the dying vehicle itself obstructs them belong to S-0312
    claim 12, not to ordinary unloading's exit tests in claim 10.
    - Backing: `measurable` (T14) for ground aircraft and sinking cleanup; `unmeasured` for
      sale/removal paths lacking an ordinary skirmish input witness (T14).
    - Confidence: high

## Formulas

**F1. Load of a vehicle transport.** For passengers with sizes *s*₁, …, *s*ₙ:

> *L*₀ = 0, *L*ₖ = trunc(RZ₅₃(*L*ₖ₋₁ + *s*ₖ)), *L* = *L*ₙ.

RZ₅₃ rounds arithmetic toward zero to a double, as S-0205 claims 4 and 5 specify.
trunc converts to an integer toward zero (S-0205 claim 7). With the rounding quirk off,
RN₅₃ replaces RZ₅₃. Each addition is rounded before conversion. Since *L*ₖ₋₁ is
a whole number, for non-negative sizes and sums within the integer domain this is equivalent
to *L* = Σ ⌊*s*ₖ⌋: each passenger counts
its size rounded down, and the order of boarding does not matter. A candidate of size *s* is
accepted when RZ₅₃(*L* + *s*) ≤ *C* and *s* ≤ *S*<sub>max</sub>, with exact comparisons
of double-precision values, where
*C* is the capacity (an integer) and *S*<sub>max</sub> the size limit.

Worked examples:
- Capacity 3, size limit 2, candidates of size 1.5. The load after 0, 1 and 2 passengers is 0,
  trunc(1.5) = 1 and trunc(2.5) = 2. A candidate is accepted at *L* = 0 and *L* = 1, and refused at
  *L* = 2 (3.5 > 3). The transport holds 2.
- Capacity 2, size limit 1, candidates of size 0.6. The load stays 0, since trunc(0 + 0.6) = 0, so every
  candidate passes and the transport takes any number (`quirks.transport_load_truncated`). With
  exact sums it would hold 3.

**F2. Unloading-cell score.** Let *f* be the transport's body facing (glossary *facing*, 0 to
65535). Its rear facing in 256 steps is

> *r* = ⌊(⌊((*f* + 32767) mod 65536) / 128⌋ + 1) / 2⌋ mod 256.

For a value *v* from 0 to 255, let σ(*v*) = *v* when *v* < 128 and *v* − 256 otherwise. For step
direction *d* (0 to 7), let *P*(*d*) = 100 when *d* = 4, and *P*(*d*) = 0 for every other step
direction. Then

> score(*d*) = *B*(*d*) − |σ(32*d*) − σ(*r*)| − *P*(*d*),

where *B*(*d*) = 128 or −128 by claim 9. Integer arithmetic, exact. The gap |σ(32*d*) − σ(*r*)| runs
from 0 to 255 and is not reduced modulo 256 (Edge cases). The chosen direction is the smallest *d*
with the largest score, and is used only if that score is above 0. The transport then turns to the
8-step facing ((*d* + 4) mod 8) × 8192, except the train rule of claim 8.

Worked example: transport facing map north (*f* = 0, *r* = 128, σ(*r*) = −128), every neighbour
open. Scores for *d* = 0 to 7: 0, −32, −64, −96, 28, 96, 64, 32. Direction 5 (map south-west) is
chosen.

**F3. Rear direction and exit order.** The rear direction is

> *q* = ⌊(⌊((*f* + 32767) mod 65536) / 4096⌋ + 1) / 2⌋ mod 8.

The first round tries directions (*q* + *k*) mod 8 for *k* = 0, 1, …, 7. The second round tries
*k* = 1, …, 7. Integer arithmetic, divisions round down, modulo gives a non-negative
remainder; the result has units of step directions.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Settling wait | 10 | ticks | Claim 8, while moving or after choosing a shore location |
| Turning step | 1 | tick | Claims 8 and 12 |
| Open-cell base score | 128 | score points | Claim 9, F2; −128 for a closed cell |
| South penalty | 100 | score points | Claim 9, F2, step direction 4 |
| Facing per step direction | 32 | 256-step facing units | F2 |
| Exit rounds | 8 then 7 | directions | Claim 10 |
| Size absent-key default | 1.0 | size | Import mapping |
| Size limit absent-key default | 0.0 | size | Import mapping; no replacement value |
| Capacity absent-key default | 0 | passengers | Not a transport |

## State and ordering

**Table U: unload phases of a vehicle transport** (claims 8 to 12).

| Phase | Event | Next phase | Effect |
|---|---|---|---|
| settling | locomotor moving | settling | wait 10 ticks |
| settling | on water, no destination | settling | the transport chooses a nearby location where a wheeled unit can stand as its destination; it waits 10 ticks |
| settling | no passengers, or no cell scores above 0 | (unload ends) | queue *guard*; standard delay |
| settling | a cell is chosen | turning | decide the kept passenger; turn rear to the cell; 1 tick |
| turning | body still turning | turning | standard delay |
| turning | body stopped | letting out | 1 tick |
| letting out | more passengers than kept, exit found | letting out | one passenger out (claim 10); standard delay |
| letting out | more passengers than kept, no exit | letting out | passenger put back at position 1; standard delay |
| letting out | no more than kept | finished | standard delay |
| finished | processing | (unload ends) | queue *guard* with the allowance to interrupt movement; standard delay, including its draw |

**Ordering consequences.**
- A loaded transport ordered to unload while driving lets nobody out until it has stopped and then
  turned. The first passenger appears 1 tick after the first processing that finds the turn
  finished. Later passengers appear at successive processings with their respective standard delays.
- Because passengers leave from position 1, the unit that boarded last leaves first. A transport whose
  turret count is above 0 keeps the unit that boarded first when more than one was aboard at
  the start.
- A passenger that fails to find an exit blocks those behind it. The transport keeps retrying the
  same passenger each processing and lets out nobody else until it gets out or the unload is
  interrupted.

## Edge cases and quirks

- **Clear air-unload destination reselected (`quirks.air_unload_reselects_clear_destination`,
  default on).** Claim 13 reselects a destination that passes the landing test, not one that
  fails it. *Fixed behaviour:* only a failed destination takes the replacement search; a
  passing destination takes the height-based approach/reselect transition instead.

- **Facing gap without wrap (`quirks.unload_facing_gap_no_wrap`, default on).** The gap between a
  neighbour's direction and the rear facing in F2 is not taken the short way round the circle.
  When the rear points between map south-east and map south-west, neighbours on the far side of
  south get gaps above 128 and drop out, even if they are open and next to the rear. For example a
  transport facing map north unloads to the south-west, never the south-east. *Fixed behaviour:*
  use the signed facing difference (glossary), so the gap is at most 128.
- **Rear direction skipped in the second round (`quirks.unload_second_round_skips_rear`, default
  on).** When no direction has two clear cells, the second round starts one step clockwise of the
  rear, so a passenger cannot get out straight behind the transport even if that neighbour is free.
  If it is the only free neighbour, nobody gets out. *Fixed behaviour:* the second round tries the
  rear direction first, as the first round does.
- **Gunner hand-over after a failed exit (`quirks.unload_failed_exit_gunner_handover`, default
  on).** Subject to S-0308 claim 5, a failed exit changes its weapon number to position 1's mode index,
  even when others remain. *Fixed behaviour:* a failed exit leaves its gunner role and weapon
  number unchanged.
- **Load counted in whole sizes (`quirks.transport_load_truncated`, default on).** A passenger
  adds its size rounded down to the load (claim 1, F1), so passengers smaller than 1 add nothing
  and a vehicle transport takes any number of them; a passenger of size 1.9 counts as 1. The
  candidate's own size is not rounded. This difference requires fractional passenger sizes.
  *Fixed behaviour:* retain the double-precision sum, rounded at each addition by S-0205
  claims 4 and 5, without converting each partial sum to an integer.

## Import mapping

| Rules section | Key | Concept | Parse form | Absent-key default |
|---|---|---|---|---|
| Vehicle and aircraft type sections | `Passengers` | capacity (Terms); above 0 makes a transport | integer (S-0003) | 0 |
| Infantry and vehicle type sections | `Size` | size (Terms; claim 1, F1) | floating point (S-0003) | 1.0 |
| Vehicle and aircraft type sections | `SizeLimit` | size limit (claim 1) | floating point (S-0003) | 0.0 |
| Vehicle and aircraft type sections | `OpenTopped` | open-topped transport (claim 14) | boolean (S-0003) | no |
| Vehicle type sections | `Gunner` | gunner type (S-0308; claims 6 and 10) | boolean (S-0003) | no |
| Vehicle type sections | `TurretCount` | turret count: above 0 gives a kept passenger (claim 11) | integer (S-0003) | 0 |
| Vehicle type sections | `IsTrain` | train: rounds its existing facing rather than facing the exit (claim 8) | boolean (S-0003) | no |
| Aircraft type sections | `IsDropship` | dropship flag (claim 13) | boolean (S-0003) | no |
| Aircraft type sections | `Carryall` | carrier aircraft, excluded from ordinary passenger release | boolean (S-0003) | no |
| Unit type sections | `EnterTransportSound`, `LeaveTransportSound` | sounds of boarding and getting out (presentation); whose type supplies each sound remains open | sound name | none |

The movement zone (claim 5), passenger locomotion class (claim 10) and foot land multiplier
(claim 9) come from S-0211's import mapping. Search settings themselves are engine constants,
not additional imported fields.

| Rules or map section | Key | Concept | Parse form | Absent-key default |
|---|---|---|---|---|
| Aircraft type sections | `Dock` | eligible dock types for claim 13's dropship search | building-type list | empty list; S-0501 claim 21 and its import mapping |
| Map tube definitions | `[Tubes]` | recorded tube entrances used by claim 7 | tube entrance, exit and direction list, owned by S-0212 claim 7 | no tubes |
| Map marker section (S-0009 claim 15) | `700` | ordinary air-unload replacement centre | waypoint cell encoding; value 0 is undefined | undefined marker, consumed by claim 13 as cell (0, 0), with search and draws still enabled |

The cruise height and flight guidance in claim 13 use S-0501 claims 5 and 6's imported
flight fields and defaults, not a new transport-only altitude. Claim 13's release height,
standing-position bypass and shared neighbour preference are engine rules, not additional
imported fields. Passenger kinds come from their infantry or vehicle type registry (S-0004
claim 1); neither kind needs an extra transport-release flag.

## Measurement scenarios

Use the glossary's measurement setup: tiny flat skirmish maps, a human observer and a
bystander computer player kept outside the test area. The units and cells named below form
that area; void any run with interference. No engine add-ons or mods are active except the
named test files allowed by the glossary's measurement setup. Use game-speed index 2 and
800 × 600, one run per part unless noted. A *test file* is a named map rules layer
removed after the run; all numbers below are invented test data, not stock values. Read ticks,
positions, passengers, targets, behaviours and weapon numbers with a read-only recorder.
Video and scripted ordinary input provide an independent view of orders and visible exits.
Every difference from a part's expected result falsifies that part; a run that cannot reach
its stated setup is inconclusive, not confirmation.

**T1. Capacity by size (claim 1).** A test file defines a vehicle transport with capacity 3 and
size limit 2, a second one with capacity 2 and size limit 1, a third with capacity 3 and no size
limit, and infantry types of size 1.5, 2.5, 1.0 and 0.6. Units are ordered to enter one by one.
The recorder counts passengers.
- *Size 1.5, first transport:* two board and the third is turned away.
- *Size 2.5, first transport:* turned away even when it is empty.
- *Size 1.0, first transport:* three board.
- *Size 0.6, second transport:* six of six board (`quirks.transport_load_truncated`).
- *Size 1.0, third transport:* turned away.
- *Falsified if:* any count differs, including only three size-0.6 units boarding or the third
  transport taking anyone. Also vary size limit across a candidate's size while the load stays
  below capacity, to isolate the separate size-limit comparison.

**T2. Air transport counts heads (claim 2).** A test file defines an air transport with capacity 2
and no size limit, and an infantry type of size 3. The transport is landed.
- *Run 1:* two size-3 units board, a third is turned away.
- *Run 2:* with one seat free, an infantry unit of a third house that a mind controller of house B
  holds is selected by B and ordered through ordinary input to enter. House B is allied with the transport's owner; a
  controlled unit belongs to its controller's house, so without that alliance the answer would be
  *neither* (claim 2). *Expected:* it boards, and after unloading it belongs to its original house.
- *Alliance control:* repeat with an uncontrolled non-allied candidate and a free seat.
  Expected: no *enter* action is offered. Restore alliance and expect *enter* again.
- *Falsified if:* sizes limit the air transport, the controlled unit is turned away,
  or either alliance-control cursor verdict differs.

**T3. Last-in first-out (claims 3, 6 and 10).** A test vehicle transport with capacity 5 and a
large size limit, on open flat ground. Board five distinguishable infantry types A, B, C, D, E in
that order, then order an unload.
- *Expected:* they get out in the order E, D, C, B, A.
- *Recorded:* boarding and appearance ticks, passenger count, and exit order.
- *Reset variants (claim 6):* board a unit with an archived target, a unit that was on a bridge
  deck, and a test gattling vehicle after it has spun up. Record archived target, bridge-layer
  state, gattling spin value and stage on the boarding tick. Expected: no archived target,
  ground layer, spin value 0, stage 0, off-map and riding the transport. Any retained reset state or different order
  falsifies the corresponding part.
- *Notification variants (claim 6):* give the transport a repeating entered-by tag with a
  calibrated visible counter action, and each entrant a separate destroyed-by-anything tag
  with a distinguishable counter. Record raises read-only as well as counter firings, ticks,
  passenger state and final acceptance: matching and latches can suppress firings without
  suppressing a raise. Compare infantry and a vehicle passenger with legal capacity and size,
  once in a vehicle transport and once in a landed air transport. Expected: no raise on order
  receipt, one transport-tag entry raise on qualifying infantry arrival before final acceptance,
  none for vehicle boarding, and no own destruction raise for either retained passenger.
  Unload and board again: unloading adds no entered-by raise, while the fresh infantry arrival
  can notify again. For refusal, use T4's established legal infantry entry followed by exclusion
  before arrival, without cancelling *enter*: the arrival raise still precedes refusal. An
  unavailable arrival witness is inconclusive. Any different raise count or precedence
  falsifies the corresponding notification rule. S-0524 claim 14's C16 supplies companion tests.

**T4. Exclusions and arrival refusals (claims 4 and 6).** With T3's transport empty, compare
cursor verdicts for an enemy infantry unit, an allied mind-controlled infantry unit, an own
vehicle carrying a parasite, and an own mind controller holding an object. Expected: no
*enter* action; the same controller after release gets *enter*. Repeat while the transport
is being erased. Observe each verdict before another condition changes.
For arrival, establish legal *enter* first, then make the passenger excluded before arrival
(for example give the entering controller a target to control). Record whether it remains in
*enter*, reaches the cell, fails to board, clears its destination, queues *guard* and receives
forced scatter. Repeat for infantry and vehicle passengers. If changing the exclusion also
cancels *enter*, that run does not test the arrival reaction. A witness for a transport on a
building cell, and exclusion combinations unreachable by normal orders, remains open.

**T5. Transport approaches, but entry breaks off (claim 5).** A test amphibious transport on
water next to shore, infantry on shore, ordered to enter while neither is tethered. Record
the transport's destination and the infantry's contact and behaviour at each processing.
Expected: the transport takes the infantry as destination, but the infantry breaks contact
and falls back to rest on the non-accepting reply. A continuing untethered *enter* falsifies
it. Repeat on land in separate reach regions for the infantry's movement zone. Control on
land in the same region: transport stays still and infantry walks to it and boards.

**T6. Refused while moving (claims 5 and 6).** Order the transport to move, then order an infantry
unit to enter it while it drives.
- *Expected:* the unit's enter attempt is refused and it rests.
- *Repeat* with the transport standing: the unit boards on reaching its cell.
- *Aircraft variant:* establish entry contact, then start the aircraft moving; its loading
  reply remains acceptance while seats are free, unlike the vehicle's refusal. Repeat full:
  without a tether its requester breaks contact on a non-accepting reply. Record the reply
  effects and the requester's behaviour, not merely whether boarding finishes.
- *Capacity-change variant:* establish contact with a vehicle, fill it with another passenger
  before the first arrives, and record the first requester's next processing. Expected:
  neither answer and, without a tether, broken contact rather than indefinite waiting.
- *Vehicle-goal distinction (claims 5 and 6):* use a compatible driving or ship passenger
  and record destination kind, supplied locomotor point, active track, behaviour, contact,
  loading replies and boarding before and after each locomotion update. Follow S-0207
  claim 26's D31 controls for moving and stationary transports and the resting passenger's
  exact-point *guard* exception. With a transport object goal and no active track, mere
  supplied-point cell equality must not itself clear either destination. A later cell-goal
  assignment instead follows that claim's cell rule. Record arrival-time acceptance or
  refusal separately: boarding may remove the passenger from the map, and refusal clears
  its goal and queues *guard* with forced scatter. These effects do not falsify object-goal
  retention. Cleanup caused solely by cell equality for a retained object goal, outside
  the guard exception, does falsify it. A run without a verified ordinary-input or named-map
  witness is inconclusive; do not write state to establish one.

**T7. Where unloading is refused (claim 7).** A loaded vehicle transport:
- on a bridge,
- on a cell next to a bridge deck (straight neighbour), and on a cell diagonally next to one,
- on water, for a human-controlled and for a computer-controlled house,
- on a tank bunker's cell, and on another building's cell (a test map lets it stand there).
- *Recorded:* whether the unload order is given (cursor) and whether anyone gets out.
- *Expected:* refused on the bridge, next to the deck, on water for the human-controlled house,
  and on the non-bunker building cell. A diagonal deck and a tank bunker alone do not refuse
  readiness, although independent cursor checks may still refuse an order.
- *Additional runs:* disable a loaded transport and place one on a tunnel-land cell; readiness
  is refused when that cell is also a recorded tube entrance. Acceptance then falsifies the
  corresponding refusal. For edge geometry, record readiness on each legal cell up to three
  cells south and east of that entrance, and on north/west and diagonal controls. Expect
  precisely the marked/edge predicate in Terms, not an eight-neighbour halo. Repeat with a
  named test map preserving tunnel land but omitting the tube: land alone must not trigger
  this refusal. A tube-interior witness remains open: ordinary input cannot select it there.
- *Our engine:* also refused on water for an AI player house; neutral and special houses keep
  their original exemption.

**T8. Settling and turning timing (claims 8 and 12).** A loaded test vehicle transport driving
north is ordered to unload; a second run starts it standing.
- *Recorded:* processing ticks and phases, the tick it stops, the tick its turn ends, and the
  tick the first passenger appears.
- *Expected:* moving settling checks are 10 ticks apart. Choosing an exit schedules the first
  turning check 1 tick later; checks that still find turning wait one standard delay.
  The first passenger appears 1 tick after a processing that finds the turn finished,
  so between 1 tick and one standard delay plus 1 tick after the turn ends. Later passengers
  appear at successive processings separated by their respective standard delays, including
  each delay's random component.
- *Train:* a named train type at an exact 8-step facing does not turn to the chosen exit.
  Repeat at a non-exact facing: expect the nearest 8-step facing, not the exit-facing result.
- *Empty/blocked variants:* no passengers, then no positive-score neighbour: no exit, queued
  *guard*. A passenger appearing before settling/turning completes falsifies this part.

**T9. Unloading cell and south penalty (claims 8 and 9).** A loaded transport standing at each of
the 8 body facings (8-step), all neighbours open.
- *Recorded:* the body facing requested during settling, and the eventual exit cell separately.
- *Expected:* the requested body facing matches F2's chosen cell, with the rear pointing at it.
  For example facing north with all neighbours open: rear points south-west. Release follows
  claim 10's separate exit search; an exit cell alone does not establish the scoring choice.
- *Second run:* block every neighbour except the one straight south (step direction 4). Facing
  north, the transport unloads south (score 128 − 0 − 100 = 28). Facing south, nobody gets out and
  it rests in *guard* (score 128 − 128 − 100 = −100).
- *Occupancy variants:* block a neighbour with a vehicle, a building, all infantry positions,
  or a single enemy infantry unit; restore it with one allied infantry unit and a free position.
  Record body turn and exit cell and compare F2. Any mismatched choice falsifies claim 9.
- *Shore run:* observe a loaded computer-controlled amphibious transport whose ordinary
  computer behaviour queues *unload* on water, or a named map reinforcement that arrives
  already in that behaviour. With one nearby reachable shore, expect it to approach shore
  before release. Do not simulate the setup by writing its behaviour. If no such map witness
  is available, this part remains unmeasured. Where reached, log qualifying shore candidates
  and the search tick: claim 8 and S-0212 claim 25 predict the selected cell. Use symmetric
  shores and vary the unload tick to distinguish tick-modulo selection from nearest-cell
  preference. With no wheel cell in the search bounds, expect the placeholder destination
  and a 10-tick wait, not an immediate exit-cell search.

**T10. Exit rounds, blocking and the kept passenger (claims 10 and 11).**
- *(a)* Rear neighbour free but the cell beyond it blocked, one side neighbour with both cells free:
  the first passenger exits via that side and walks two cells out.
- *(b)* Only the rear neighbour free, its far cell blocked: nobody gets out
  (`quirks.unload_second_round_skips_rear`).
- *(c)* A test transport whose turret count is above 0, with three passengers: two get out and the first-boarded
  stays.
- *(d)* The same with one passenger: it gets out.
- *End timing, parts c and d:* record processing ticks, phases and queued behaviour after
  the passenger count reaches the kept count. At the next letting-out processing, expect
  *finished* with one standard delay; at the following processing, expect queued *guard*
  with S-0215 claim 11's movement-interruption allowance. That final processing also draws
  the standard delay: log scenario random state across it and the next scheduled processing
  tick, even though *guard* is queued. No draw or a fixed 1-tick delay falsifies claim 11.
  Earlier or later completion falsifies the phase timing.
- *(e)* A named gunner transport with two passengers of different mode indices: the first
  boarded has mode 2, the last mode 3. Let settling choose an exit, then move blockers into
  every exit during its turn, before release. Expected after the failed exit: both still
  inside, most recently boarded still position 1, weapon number changes from 2 to 3.
  Clear blockers: the same passenger exits first. A retained weapon number 2 falsifies the
  failed-exit hand-over. Repeat with an entry-restricted vehicle passenger at position 1
  and an infantry passenger behind it: failure of the first blocks the second.
- *(f)* Allow only a bridge-deck neighbour during the letting-out phase: no passenger is placed
  on it. Any such placement falsifies the deck exclusion.
- Record exit cell, placement position, facing, queued movement destination, passenger order,
  weapon number and ticks in every part. Any departure from the listed results falsifies it.

**T11. Landed air transport unloading (claim 13).** A named air transport, already landed
with zero pitch and no destination, loaded with distinguishable passengers, ordered to unload.
Expected: successful placements at its own cell, last-boarded first, one per release processing
with the respective standard delays, then rest when empty. Record placement and processing ticks.
A tethered landed run should release nobody until untethered; use an ordinary pad/contact
witness if available, not state writes. Release while tethered falsifies that part.
For airborne runs, use named map types with and without the dropship flag, with and without
an existing destination. Provide waypoint 700 and a dock, then remove the dock in a second
run. Record phase, height above terrain, cruise height, pitch, destination, unfinished flight,
landing-test verdict and scenario draws. Compare every transition to claim 13's table.
Expected: a dropship without destination seeks a dock then the S-0501 claim 41 fallback.
a non-dropship without destination takes stopped check, polling at 1 tick. A passing
destination takes waypoint-centred replacement; a failing one takes approach only at cruise
height and otherwise reselect. Approach waits the standard delay, while stopped check and
reselect always wait 1 tick. Record ordinary flight descent separately: approach is not a
descent command or a landing guarantee. Any verdict, replacement or delay mismatch falsifies
its corresponding part.

**T12. Open-topped passengers (claims 14 and 15).** A named test map defines an open-topped
vehicle transport with capacity and size limit sufficient for two armed infantry passengers.
Board both before each part.
- *(a)* Drive it past an enemy: the passengers fire while inside. Record passenger and
  transport positions each tick: all agree after each transport position change. Repeat with a
  named open-topped aircraft carrying infantry to test that the flag is not vehicle-only.
  Failure of either passenger to fire while the enemy is available, any passenger position
  differing from the transport after a position change, or the aircraft not exhibiting the
  same passenger behaviour falsifies the corresponding part.
- *(b)* Order it to attack: both passengers take its target. Either passenger not taking the
  transport's target falsifies this part.
- *(c)* A test file defines a larger transport that takes the open-topped one. While its
  passengers have a target, it boards the larger transport. Expected: every passenger's
  target is nothing on the boarding tick. Also enter a suitable absorbing building: targets
  clear on leaving the map. A retained target falsifies either part.
- *(d)* A test transport owned by house A, carrying an allied house B's unit, with a target, then
  unloaded: B's unit has no target after leaving. An own unit keeps its target. A different
  result for either passenger falsifies this part.

**T13. Placement and alliance selection (claims 9, 10 and 13).** Use the shared setup,
100 ordinary unloads into an empty cell, and one run per blocked-position variant on a named
map. Record positions before passengers start moving, entry verdicts, facings, destinations
and scenario draws. Vehicle-transport infantry exits must follow the recorded 0-to-3 draw
and claim 10's preferences, never centre or top. Block right, then left, then bottom positions
in separate runs and compare the first-free rule. A vehicle passenger must follow the nearby
search with its own class but fixed normal zone and tick-modulo candidate choice, not simply
appear at the serving-cell centre. Vary placement tick and visibility with candidates fixed.
Every mismatch falsifies the corresponding part.

For aircraft, compare exact off-centre and centred aircraft positions. Block neighbour entry
in preference order 4, 5, 3, 7, 1, 0, 6, 2, recording the first *clear* outcome. Expected:
exact aircraft desired point, that direction's facing and queued move to that neighbour,
not body-facing inheritance or scatter. Add occupants in the aircraft's cell after landing:
the aircraft is excluded, but those other occupants still affect placement. With all eight neighbours blocked expect direction 4 facing
and direction 2 destination; inconclusive if ordinary input cannot establish the occupancy.
Use a named passenger type unable to be placed on the release surface to test failure.
Expect passenger count to fall without placement or retry. Track that unit read-only to
settle its still-open off-map lifetime. Reinsertion falsifies the immediate failure rule.

For the alliance score, place allied and non-allied infantry at right and left positions in
a scored neighbour, then reverse arrival order. Expect the most recent cell-layer arrival
to supply the owner, not the infantry-position owner record. A nearer top-position unit
wins over either. With no combat object, the alliance condition passes. Record the requested
body turn rather than only the eventual exit. An unreachable mixed-occupancy setup is
inconclusive, not confirmation. Any mismatched selected owner falsifies claim 9.

**T14. Removal cleanup (claim 16).** On the shared tiny map, destroy a loaded aircraft already
on the ground and sink a loaded non-crashable vessel. Record passenger count and in-play
status through final removal, nested passengers with named test data, kill/loss counts,
destruction notifications and independent death effects. Remaining passengers disappear
through uncredited deaths, never escape or silently survive. Earlier credited deaths stay
credited. Any surviving carried unit, extra attacker kill or independent passenger death
animation falsifies the corresponding part. Sale and direct removal without death are
additional variants only if ordinary skirmish input witnesses are found: do not inject orders
or write state to create them. S-0408 claim 11 establishes no such sale input.

**T15. Release height and undefined marker (claim 13; Q-0672 and Q-0673).** Use the shared
setup and a named test map defining a non-dropship air transport with room for an infantry
passenger and a small vehicle. Arrange an ordinary map placement or flight state that reports
stopped with no destination above terrain; merely hovering with unfinished flight is not
sufficient (S-0501 claim 40). If no ordinary-input witness reaches this state, record that
variant as inconclusive rather than write state. Unload, then repeat landed, centred and
off-centre, once per kind/height/position variant. Record exact positions, terrain heights,
facings, entry outcomes, destinations and scenario random state at the appearance instant,
before passenger movement. Expect airborne infantry and vehicles to retain aircraft horizontal
coordinates and height, without a standing-position draw. Landed infantry use the occupant-aware
choice and its centre-class draw; landed vehicles retain exact aircraft coordinates without
that choice or a nearby search. Both kinds face and move by the same first-clear neighbour.
Block the own cell with another object while leaving a preferred neighbour clear: an own-cell
entry refusal must still fail placement. Observe later falling separately; it must not be
mistaken for surface placement. Surface projection, a vehicle cell-centre snap/search, a
standing-position draw for elevated infantry, or success through a refused own cell falsifies
the corresponding rule.

For replacement, use three map variants: marker 700 absent, explicitly undefined, and defined
at a distant usable cell. Fly a loaded aircraft toward a clear destination and order unload.
At each selection processing record the previous destination, selected destination and draws,
separating search draws from the standard-delay draw and later movement processing. The first
two variants search around (0, 0), not either aircraft position or previous destination.
Compare every sampled candidate and first accepted result to S-0501 claim 40. Repeat on a
map whose usable area has no accepted candidate within any of the 32 sampled radii around
(0, 0). Expect 32 search draws and placeholder destination immediately after selection, even
if end-of-update cleanup removes it. The defined-marker control instead searches around its
marker and keeps that marker if unsuccessful. No search, another centre, retention of the
old destination on failure, or skipped draws for unusable radii falsifies the respective
part. One run per map variant suffices with recorded random state; initial-loading draw
suppression requires a separate ordinary map-loading witness if reachable.

## Open questions

- The infantry type flag that makes an air transport forget its target and destination and leave
  when an infantry unit boards it is not identified (low). It is not stated as a claim.
- Exact aircraft boarding geometry and flight-specific refusal states are not traced here.
- Normal-input witnesses for shore approach, tube-interior readiness and building-cell tests.
- Whether the boarding and leaving sounds are the passenger's or the transport's.
- Q-0700 follow-up: eventual lifetime and cleanup of an air passenger whose placement fails
  after removal from passenger order. Claim 13 settles the immediate result, not its duration.
  T13 proposes a recorder run.
- Normal-input witnesses for claim 13's all-neighbours-blocked fallback and tethered inhibition.
- Q-0707's destruction-entry exemption belongs to S-0312 claim 12 and S-0209, not ordinary
  unloading. Those owners must explicitly exclude the dying vehicle itself while retaining
  other occupants' ordinary effects.
