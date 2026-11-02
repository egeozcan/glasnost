---
id: S-0502
title: Movement of jumpjets
topic: movement
revision: 21
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0975, Q-0976, Q-0977, Q-0978, Q-0979, Q-0980, Q-0981, Q-1010, Q-1011, Q-1012, Q-1030, Q-1031, Q-1500, Q-1501, Q-1504]
supersedes: []
---

# S-0502: Movement of jumpjets

## Summary

How a unit that moves with the jumpjet locomotor flies:
- its seven flight states (take-off, hover, cruise, landing and crash among them)
- its target height, climb and descent, and the look-ahead over obstacles
- its speed, acceleration and braking, and the speed bands near its destination
- the wobble of its height and its straight-line step along the movement heading
- the body facing that follows the heading
- where it may hover and where it may land, and the random step to a neighbouring cell when a
  hover slot is taken
- what happens when a landing fails
- the crash fall
- which flight states count as "moving now" for the firing checks
- when a jumpjet is registered for the flying-object search, and when it counts as in the air.

Skirmish only. Network games and campaigns are out of scope.

Pointers, not repeated here:
- **Turn rates.** The rate of the movement heading (the type's jumpjet turn-rate field, its
  conversion and its absent-key default) and the heading's initial value: S-0206 claims 6, 7
  and 14.
- **Draw order.** The layer a jumpjet is drawn in, the game-wide cruise-height threshold, the
  minimum of the type's cruise height and its absent-key default: S-0102.
- **Range and firing.** Range against or from a jumpjet: S-0307 claims 18 and 24. The
  firing-check refusals that use claim 20's *moving now*: S-0310 claims 7 and 8. The rank of a
  locomotor's own refusal: S-0301 claim 9 (rank 16).
- **Factory exit.** Release destinations: S-0417 claims 5 and 10. The ascent clear report,
  factory contact and tether release, and subsequent destination effects: S-0417 claim 15.
- **Death and impact.** What a killed jumpjet does when it starts to crash, and what happens on
  impact: S-0312 claims 15 and 16. This spec supplies only the crash motion (claim 18).
- **Sinking.** Once claim 19 sets a vehicle sinking, its sinking descent and removal are
  S-0504 claims 4 and 5.
- **Magnetic lift.** A unit seized by magnetic lift: S-0607 claims 5, 15 to 20. Claims 18
  and 19 name only the parts of the jumpjet's own movement that change while it is seized.
  Those exceptions are not a specification of magnetic lift itself.
- **Other movement specs.**
  - Aircraft flight: S-0501.
  - Hovering vehicles (a different locomotor): S-0503.
  - Coordinates, facings, directions and the n-step rounding: S-0204.
  - The scenario random stream and ranged draws: S-0202.
  - Occupancy, entry outcomes and standing positions: S-0209.
  - The nearby-cell search: S-0212 claim 25.
  - Deploying vehicles: S-0517.
- **Chrono weapons and teleports.** Temporal delivery to a jumpjet requires the ground render
  layer (S-0514 claim 1; S-0102 claim 3), not a particular jumpjet flight state or combat
  airborne test. Ground-layer delivery still faces S-0514 claim 3's lock refusals.
  Superweapon teleport eligibility, landing and arrival belong to S-0513 claims 34 to 40.
- **Cloaking.** A jumpjet can start to cloak only on the ground: S-0516 claim 4.
- **Airborne.** A jumpjet state is not either of the glossary's *airborne* tests. Whether a
  jumpjet counts as in the air for combat is a height test, not its being *hovering* or
  *cruising*: claim 30. Whether detonations above the terrain find it through the
  flying-object search is its registration: claim 29.

## Terms

From the glossary and other specs:
- *jumpjet type*, *jumpjet*, *jumpjet cruise height*, *movement heading*, *body facing*
- *balloon-hover type* (S-0310 Terms) and *jumpjet-turn flag* (S-0310 claim 8)
- *locomotor*, *speed fraction*, *destination*, *target*, *landing point* (S-0207 claim 20)
- *height above the surface*, *height above the terrain*, *world height*, *high bridge*,
  *bridge deck*, *land type*
- *occupancy record*, *entry outcome*, standing position (S-0209)
- *step direction*, *n-step facing*
- *scenario random stream*, *ranged draw*
- *demolition warhead*; *damage that ignores defences* and the *no-escape* marker (S-0312 Terms)
- *behaviour*, current and queued (S-0215); *sight*; *sinking*; *sinking descent*
- *legal cell*.
- *tracked flying object*, *sector* and the flying-object search (S-0303 Terms, claim 6 and
  F7). Here *registered* means being a tracked flying object.
- *airborne* for an ordinary object (S-0307 Terms); *on the map*, *off the map* (glossary).
- *seized*, *grip*, *dropped*, *set down* (S-0607 Terms). Here “held by magnetic lift”
  means *seized*, including after the grip ends and during a drop, until restoration of the
  unit's own locomotor (S-0607 claim 20). It does not mean only actively gripped.

New terms:
- **The jumpjet fields.** These are copied from a jumpjet type to each unit when it gets the jumpjet
  locomotor (claim 1):
  - the speed *S* (integer, leptons per tick)
  - the acceleration *a* (single precision, leptons per tick per tick)
  - the climb rate *c* (single precision, leptons per tick)
  - the crash rate *k* (single precision, leptons per tick)
  - the cruise height *H* (integer, leptons; S-0102 gives its minimum)
  - the wobble rate *W* (single precision, wobbles per 15 ticks)
  - the wobble amplitude *d* (integer, leptons)
  - the no-wobble flag
  - the jumpjet turn rate *T*.

  The single-precision values widen to double precision in every formula below.
- **The jumpjet state.** One of seven flight states of a jumpjet. Each tick runs the rules of the
  current state (claims 5 and 11 to 18):

  | State | Meaning |
  |---|---|
  | *grounded* | on the ground or a deck, at rest |
  | *ascending* | rising after a move order |
  | *hovering* | holding position in the air |
  | *cruising* | flying towards its destination |
  | *descending* | coming down to land |
  | *crashing* | falling after a crash start |
  | *crashed* | the fall has ended |
- **Moving, moving now.**
  - A jumpjet is *moving* while its locomotor retains a movement commitment: a successful
    point assignment sets it (claim 16), and touchdown or an accepted direct no-point
    instruction clears it. This is independent of the order-level destination: stop can
    clear that destination while retaining movement through rerouting (claim 24).
  - It is *moving now* in every jumpjet state except *grounded* and *hovering*.
  - Glossary candidate: this is the jumpjet sense of "the locomotor is moving it" in S-0310
    claims 7 and 8.
- **Destination-refresh state.** A pending requirement to reapply a vehicle's order-level
  destination even when unchanged (claims 24 and 27). It is runtime state, initially absent,
  not an imported field. It is distinct from a movement commitment.
- **Movement point.** The locomotor's stored destination point, distinct from the unit's
  order-level destination (S-0213 claim 8). Here flight headings, distance and arrival use
  this point. Claim 24 distinguishes clearing an order-level destination from giving the
  locomotor no point.
- **Current speed** *v*<sub>c</sub>, **maximum speed** *v*<sub>m</sub>. Two double-precision
  values in leptons per tick. Both start at 0. The current speed moves towards the maximum
  speed each tick (F1).
- **Target height** *h*<sub>t</sub>. An integer in leptons that the jumpjet climbs or descends
  towards: the height it wants above its ground reference. The states set it (claims 5 and 11 to
  18).
- **Wobble phase** φ. A double-precision angle in radians that a hovering or cruising jumpjet
  advances each tick (F2).
- **Desired height** *h*<sub>d</sub>. The target height plus the wobble, truncated to an integer
  (F2).
- **Ground reference** *g*. The world height below a jumpjet that it measures its height against
  (F3). It includes an allowance for buildings and other objects under and ahead of it.
- **Hover slot.** A mark that each cell carries for at most one hovering jumpjet. A jumpjet
  claims the slot of its cell when it starts to hover there and releases it when it leaves the
  hover, lands or crashes (claims 7, 13, 14 and 18). Glossary candidate.
- **Landing reservation.** A descending jumpjet's mark in the occupancy record of its landing
  cell, made before it touches down (claim 15). Glossary candidate.
- **Neighbour step.** The jumpjet's move to a randomly chosen neighbouring cell when the hover
  slot it wants is taken (claim 21). Glossary candidate.
- **Warp-pending immobilization.** S-0207's external-teleport commitment, from
  acceptance through the leaving wait until departure from the old position, not the
  arrival hold. It is distinct from phasing in and the parasite-related held countdown
  (S-0207 claim 27).
- **Teleport-capable flag.** The imported type flag used by claim 27's dock handling.
  It does not itself mean that the current locomotor is teleporting.
- **Deploy-to-land vehicle.** A vehicle whose type has both the simple-deployer flag and the
  deploy-to-land flag (Import mapping; S-0517 owns deploying). It hovers when idle and lands only
  to deploy.

## Behaviour

### Fields and game-wide controls

1. **Each unit copies its type's jumpjet fields.**
   - When a unit gets the jumpjet locomotor, it copies its type's jumpjet fields (Terms) and
     keeps them while it has the locomotor.
   - The cruise height *H* is the type's jumpjet cruise height after S-0102's minimum.
   - None of these fields is ever read from the game-wide jumpjet controls (claim 2).
   - Backing: `measurable` (scenario J1) for field effects. Retaining the copied values is
     `unmeasured`: the scenario cannot change a live type's fields after the unit was created.
   - Confidence: high

2. **The game-wide jumpjet controls have almost no effect.** The rules data has a game-wide set
   of jumpjet controls: turn rate, speed, climb, cruise height, acceleration, wobble rate and
   wobble amplitude (Import mapping). The original reads all seven.
   - The cruise height is used only as S-0102's layer threshold.
   - The turn rate has no effect (S-0206 claim 14).
   - The other five have no effect on play.
   - A type key written with different letter case does not set the corresponding field
     (Import mapping; S-0001 claim 13).
   - Backing: `measurable` (scenario J1 b for controls; J1 c for case sensitivity). Reading values that
     never affect play is `unmeasured`: no observable distinguishes reading and ignoring from
     never reading them.
   - Confidence: high

### Per-tick update

3. **What runs in a tick.**
   - In each tick in which a jumpjet is *moving* or *moving now* (Terms), the state outcomes use
     that tick's physics results. A state change, or a new maximum speed or target height, first
     affects the physics results in the following tick (claims 6 to 10, F1 to F5).
   - In a tick in which the jumpjet is neither moving nor moving now (grounded or hovering
     without a move order), neither runs, except during claim 17's no-cell magnetic-release
     safety fall:
     - its height does not change
     - its wobble phase does not advance
     - its speed fraction keeps its last value
   - **Safety departure (design).** A pending crash start above the surface also makes the
     unit eligible for its next update, even without a move order. Claim 17 separately
     guarantees every update of its no-cell magnetic-release safety fall. Claim 18 then applies:
     a dead hovering unit cannot remain suspended merely because its order ended.
     A grounded unit cannot start an aerial crash; S-0312 claims 15 and 16 own removal.
   - The original's movement eligibility does not itself include a pending crash start.
     A successful later destination assignment would resume updates, but no general automatic
     wake or independent removal of an idle suspended unit is established here.
   - Backing: `measurable` (scenario J2) for ordinary tick timing and grounded idle values.
     Uncommitted hovering idle values are `unmeasured`: J2 does not establish an ordinary
     input sequence producing that starting state independently of a retained commitment.
     The original idle-crash aftermath is `unmeasured` (diagnostic J15): an ordinary killing
     hit also clears orders and can invoke claim 17, so establishing the otherwise idle case
     requires recording those effects. The safety departure is a design rule, not an
     observation of the original.
   - Confidence: high

4. **Cell listing and occupation are not a zero-height test.**
   - A cell-layer listing, an actual-position occupancy mark, a landing reservation and a
     hover slot are distinct. S-0209 claims 1 and 13 own the first two concepts; claims 15
     and 21 here own the latter two.
   - In *grounded*, *ascending*, *descending*, *crashing* and *crashed*, each physics step
     removes the previous cell listing and makes the new position eligible for listing only
     when its evaluated draw layer is ground (S-0102 claim 3). Listing is on the deck when
     the unit's on-deck state is set, and on the ground otherwise. A jumpjet does not use
     the ordinary aircraft's positive-listing-height exclusion of S-0209 claim 13.
   - **Exact height predicate (Q-1010).** Let *r* be the signed integer height above the
     surface: above terrain unless the on-deck state is set, then above the deck. Let
     *r*′ = *r* − 416 leptons only over a high-bridge cell, with on-deck state unset,
     *r* ≥ 416 leptons and no bridge/parachute falling state; otherwise *r*′ = *r*.
     New listing qualifies exactly when *r* < 208 leptons **or** *r*′ = 0 leptons.
     The 208-lepton test uses *r*, not *r*′. The type's cruise height does not change
     this predicate; it separates the two non-ground render layers only.
   - Thus flat-land heights 1, 50 and 100 leptons qualify, while 208 and 250 do not.
     Above a high bridge without on-deck state, *r* = 416 qualifies if not falling,
     but *r* = 417 does not: positive clearance below 208 leptons above the deck is
     not itself sufficient. With on-deck state set, *r* already excludes the deck height.
     These are exact integer comparisons, without rounding.
   - **Falling is not crashing.** The bridge/parachute falling state that suppresses the
     subtraction is distinct from the jumpjet states *crashing* and *crashed*.
     Starting a jumpjet crash does not itself set that falling state. An ordinary
     crashing jumpjet therefore still gets the subtraction, unless independently falling.
     S-0102 claim 3's table must use this same two-height distinction.
   - Actual-position marking follows that placement's ordinary kind-specific rules of
     S-0209 claim 1. In particular a vehicle can mark the cell during low ascent or descent,
     or near the end of a crash. Its first positive height does not permanently forbid that
     mark. Descent can therefore supply an actual-position mark as well as its separate
     landing reservation; the reservation is not the only way a mark returns.
   - *Hovering* and *cruising* physics make no new cell listing or actual-position mark.
     Normally they have none after the higher part of ascent. This is not a general erasure
     of a listing already retained from a low flight-state transition (claim 22).
   - At the first rise from the surface, the previous occupancy is cleared and the on-deck
     state ends. Subsequent low-flight placement can mark again as above.
   - Backing: `measurable` (J3, including separate listing and reservation observations).
     The independently falling exclusion is `unmeasured`: J3 does not establish a sequence
     combining that state with a jumpjet placement at the discriminating bridge height.
   - Confidence: high for the placement eligibility; medium for the full kind-specific
     occupancy consequences, which depend on S-0209 claim 1.

### Take-off and climb

5. **Take-off.** In its first eligible update with a movement commitment, a *grounded*
   jumpjet takes off, even when its order-level destination is empty (claims 16 and 24):
   - its speed fraction becomes 1, except 0 for *S* = 0 under claim 25's design rule
   - its current and maximum speeds become 0
   - its target height becomes *H*
   - it enters *ascending*.
   - Backing: `measurable` (scenario J2)
   - Confidence: high

6. **Speed.** Each physics step adjusts the current speed according to F1:
   - acceleration adds *a* per tick, capped at *S*
   - braking subtracts 1.5 × *a* per tick, floored at 0.

   These limits are not clamps to the maximum speed: an acceleration overshoot can also
   incur braking in the same tick.

   The jumpjet's speed fraction is then set to *v*<sub>c</sub> / *S*. S-0310 claim 8 uses that
   fraction for an infantry jumpjet. For *S* = 0 use fraction 0 instead (design, claim 25).
   - Backing: `measurable` (scenario J4)
   - Confidence: high

7. **Ascending.** In each tick of *ascending*, let *r* be the jumpjet's height above the
   surface. Let *h* = *r* − 416 leptons when its cell has a high bridge, it is not on the deck
   and *r* ≥ 416 leptons; otherwise *h* = *r*. These are integer heights.
   - **While *h* is below the target height:**
     - Once *h* exceeds one quarter of the target height (one half for a balloon-hover vehicle),
       each truncated toward zero, two things happen in the same tick:
       - the maximum speed becomes *S*
       - the movement heading starts turning towards the destination.

       **Factory release.** S-0417 claim 15 owns the ascent clear report. An infantry
       unit or vehicle using jumpjet locomotion, with either the jumpjet-type or balloon-hover
       flag and still in contact with a vehicle factory or a building with a barracks door
       kind, reports on its first qualifying ascending update. Both contact and tether end
       in that unit update. The strict integer window is trunc(*h*<sub>t</sub> / *q*) < *h*
       < *h*<sub>t</sub>, evaluated after physics, where *q* = 2 only for a balloon-hover
       vehicle and 4 otherwise. Both equalities are excluded; balloon-hover infantry uses 4.
       No foundation departure, cell crossing, cruise, destination arrival or touchdown is
       required. Saved-guard-spot reassignment or destination discard and scatter follow
       S-0417 claim 15, not its claim 8's vehicle-arrival handling. Destination discard
       alone leaves the movement point and commitment intact; it is not claim 24's effective
       clear. An accepted scatter can replace the destination.
       The report does not require a rally point, but unsolicited take-off of resting
       no-rally infantry remains unestablished. A physics step directly to or beyond the
       target skips the original's window; no original ascent-completion report is
       established for that case.
     - If the jumpjet is already over its destination cell and is not of a balloon-hover type,
       its target height becomes its current *r* and it enters *cruising*. Away from a bridge,
       a jumpjet whose accepted movement point remains in its own cell therefore stops its
       take-off climb as soon as it passes that fraction. This is conditional on the resulting
       point, not a promise that an own-cell player order is accepted there (claim 23). Above a bridge while not on its deck, the retained target height includes
       the 416 leptons that the threshold height *h* excludes
       (`quirks.jumpjet_bridge_early_cruise_height`; Edge cases).
   - **Once *h* reaches the target height:**
     - if another jumpjet holds the hover slot of its cell, it takes a neighbour step
       (claim 21) and enters *cruising*
     - otherwise it claims the slot and enters *hovering*.
   - **Safety departure (design).** A released unit satisfying the same type and factory
     contact conditions that reaches or passes the target height without a qualifying
     report gives that report in the reaching update, with the same link and destination
     effects. This prevents a skipped window from holding the factory indefinitely.
     There is no quirk switch.
   - Backing: `measurable` (scenario J2) for thresholds and transitions. The factory
     report inherits S-0417 claim 15's `measurable` backing (U14), including its
     `unmeasured` limitations for balloon-hover infantry, non-balloon jumpjet vehicles,
     missing type flags, skipped-window aftermath and unsolicited no-rally take-off:
     no stock release fixture establishing those cases is settled there. The fallback
     is design, not an observation of the original (J26).
   - Confidence: high (thresholds, transitions and report predicates); medium for combined
     factory-release consequences, inherited from S-0417 claim 15.

### Height, wobble and ground reference

8. **Height control.** Each physics step moves the jumpjet's world height *z* towards its
   desired height above the ground reference (F2 to F4):
   - It climbs or descends by the climb rate *c* per tick. The crash rate *k* is used for
     crash motion (claim 18) and the bridge-reference threshold (F3), not ordinary descent.
   - When the remaining difference is within one step, it snaps to the desired height exactly.
   - It never descends below the terrain, or below the deck when it is on or over a deck
     (F3).
   - **Ground hold.** In the same step, if the jumpjet is not over its destination cell and its
     height above the ground reference is less than half its desired height (truncated toward
     zero), its current speed is set to 0.
   - So a jumpjet away from its destination only moves sideways once it is at least half its
     desired height. It stops moving sideways when an obstacle ahead raises the ground
     reference by enough (claim 9).
   - Backing: `measurable` (scenario J5)
   - Confidence: high

9. **Wobble and look-ahead.**
   - **Wobble.** In *hovering* and *cruising*, unless its type has the no-wobble flag, the
     jumpjet's height oscillates around its target height with amplitude *d*, at the rate of
     *W* wobbles per 15 ticks (F2). In every other state, the wobble phase is 0.
   - **Ground reference.** In every state except *grounded* and *descending*, the ground
     reference uses a look-ahead (F3) when the jumpjet is not over its destination cell, and
     always for a balloon-hover type. “Destination cell” here means the cell of the stored
     movement point, not the order-level destination. This includes *crashing*: a dropped
     seized unit over its cell centre that retains a stop-reroute point in another cell
     (S-0607 claim 13) uses look-ahead even with zero horizontal speed. With zero speed,
     only its own cell's clearance contributes, not the neighbouring cell's. Thus its
     ordinary height step can rise towards an object's clearance before the extra crash
     drop, rather than using bare terrain as its reference. The look-ahead combines two clearances:
     - the clearance of the jumpjet's own cell
     - while its current speed is above 0, the clearance of the neighbouring cell in the
       8-direction of its movement heading.

     A cell's clearance is its terrain height plus an allowance:
     - a building's art height (in height levels, times 104 leptons) when a building is listed
       in the cell's ground layer
     - otherwise 85 leptons when infantry, a vehicle or an aircraft is listed in the cell's
       ground layer (S-0209). Terrain objects and overlays do not supply this allowance,
       and an object listed only on the bridge-deck layer does not supply it
     - plus the bridge height when the cell has a high bridge.

     When the cell ahead is not higher, the two clearances are averaged (F3).
   - **Direct ground reference.** In the other cases, the ground reference is the terrain under
     the jumpjet, or the deck when it is over a high-bridge cell and its world height is at least
     the deck's height less *k* (F3). Claim 17's no-cell magnetic-release safety fall also
     uses this direct reference while it has no movement point (design).
   - Backing: `measurable` (scenario J5) for wobble, the object-kind and cell-layer
     clearances, and the bridge reference; J27 b proposes a conditional measurement of the
     retained-point seized-crash combination. Its stock reachability is not established:
     a run must record both the different-cell retained point and a nonzero own-cell
     clearance allowance before it can confirm or falsify that combination.
   - Confidence: high

### Horizontal motion and facing

10. **Straight-line step and body facing.**
    - **Step.** Each physics step moves the jumpjet trunc(*v*<sub>c</sub>) leptons along its movement
      heading (F5). It moves in a straight line with no path: the heading turns towards the
      destination at S-0206 claim 14's rate, and terrain and objects do not block it.
    - **Truncation bias.** Because each coordinate is truncated separately (F5), the step loses
      up to 1 lepton in some directions and drifts by 1 lepton across others
      (`quirks.jumpjet_step_truncation`; Edge cases).
    - **Body facing.** The body facing is set to the movement heading in each physics step. It
      follows the heading exactly and has no turn rate of its own.
    - **Exception.** A hovering infantry jumpjet whose type has the jumpjet-turn flag and that
      has a target sets its movement heading at once to the direction of the target (S-0204
      claim 8). Its body facing is not set from the heading in that step.
    - **Magnetic lift.** A seized unit keeps its body facing (S-0607 claim 7).
    - Backing: `measurable` (scenario J6 for ordinary facing and steps; S-0607 scenario L2
      for retained facing while seized).
    - Confidence: high

### Cruise and arrival

11. **Cruising.** In each tick of *cruising*:
    - the movement heading turns towards the destination
    - the jumpjet measures the horizontal distance ρ from its position to the destination point,
      using S-0204 claim 5's table square root, truncated toward zero to integer leptons.
    - **Arrival.** When ρ < 20 leptons, the jumpjet arrives (claim 12).
    - **Speed bands.** Otherwise it sets its maximum speed by distance band (F6). The three nearer
      bands change its target height only when it has no target; the far band sets full target
      height even when it has a target:
      - closer than *S*: a slow final approach at reduced height
      - closer than 2*S*: a slower band, slower still when the remaining turn is large
      - closer than 50*S* / *T*: a middle band at three-quarters height, slower when the remaining
        turn is very large
      - farther: full speed *S* at full height *H*.
    - **Full height kept.** After the band, the target height is set back to *H* in three cases:
      - the jumpjet is of a balloon-hover type
      - the destination cell's land type is water or beach
      - it is a vehicle whose type has the deploy-to-land flag.

      Such a jumpjet keeps target height *H* through cruise and arrival; its actual height
      still follows the rate-limited height control of claim 8.
    - The remaining turn is measured one-sidedly (`quirks.jumpjet_turn_slowdown_one_sided`,
      Edge cases).
    - Backing: `measurable` (scenario J7)
    - Confidence: high

12. **Arrival.** On arrival:
    - the current and maximum speeds become 0
    - the jumpjet's position moves exactly onto the destination point's horizontal position.

    The arrival outcome is determined by the following precedence table:

    | Arrival condition | Result |
    |---|---|
    | Held by magnetic lift | Target height becomes 0 and the unit enters *descending* (S-0607 claim 8 owns the arrival drop). |
    | Not held by magnetic lift; no target; not a balloon-hover type; deploy-to-land vehicle | It claims its cell's hover slot and enters *hovering*, or takes a neighbour step if the slot is held. |
    | Not held by magnetic lift; no target; not a balloon-hover type; not a deploy-to-land vehicle | Target height becomes 0 and the unit enters *descending*. |
    | Not held by magnetic lift; has a target or is a balloon-hover type | It claims its cell's hover slot and enters *hovering*, or takes a neighbour step if the slot is held by another jumpjet. |
    - Backing: `measurable` (scenario J7 for ordinary arrival; S-0607 scenario L2 for
      seized arrival).
    - Confidence: high

13. **Hovering.** In each tick of *hovering* in which the jumpjet is moving, it checks these rows
    in order:

    | Condition | Result |
    |---|---|
    | its destination's horizontal position differs from its own | its heading starts turning towards the destination, it releases its hover slot, and it enters *cruising* |
    | it has a target | it keeps hovering |
    | it is a simple-deployer vehicle, not held by magnetic lift, with the deploy-to-land flag and no landing request (S-0517 claim 5) | it keeps hovering; its target height is set to *H* |
    | it is any other simple-deployer vehicle not held by magnetic lift | it releases its hover slot and enters *descending* |
    | it is of a balloon-hover type | it keeps hovering |
    | otherwise | it releases its hover slot and enters *descending* |

    So an ordinary jumpjet that loses its target over its destination comes down and lands.
    The balloon-hover rule prevents spontaneous landing except when an earlier
    simple-deployer row requires descent.
    - Backing: `measurable` (scenario J8 for ordinary and deploying cases; S-0607 scenarios
      L2 and L3 for the seized pull and release). The seized simple-deployer row exclusions
      remain `unmeasured`: those scenarios do not establish a seized simple deployer hovering
      with the relevant landing request and flags.
    - Confidence: high (rows); medium (the deploy-to-land row, whose landing request S-0517
      owns)

### Landing

14. **No landing on water or beach.** A *descending* jumpjet whose destination cell's land type
    is water or beach does not land there, unless its current or queued behaviour is *enter*
    (S-0215 claim 1), or it is seized (S-0607 claim 7).
    - It claims the hover slot of that cell, sets its target height to *H* and enters
      *hovering*.
    - If the slot is held by another jumpjet, it takes a neighbour step and enters *cruising*.

    It keeps doing this until it reaches a cell where it may land, or receives an order.
    - Backing: `measurable` (scenario J9 for ordinary water/beach and entering cases,
      S-0607 scenarios L3 and L6 for seized descent and water/beach outcomes).
    - Confidence: high

15. **Landing check and touchdown.**
    - **Landing check.** In every tick that remains in *descending* after claim 14's water/beach
      rule, the jumpjet checks its landing cell. There is no alternating-tick cadence. If its
      current or queued behaviour is *enter*, or it is held by magnetic lift, it skips the
      landing check. Otherwise the cell passes when both of these hold:
      - **Standing position.** For infantry, the standing position at the destination point is free
        (S-0209), or the jumpjet already holds its landing reservation. A vehicle always passes
        this part.
      - **Entry outcome.** Unless the cell has a high bridge, its entry outcome for the jumpjet
        (S-0209 claim 4 for infantry, claim 5 for vehicles) is *clear* or *cloaked blocker*, or *moving blocker* when the jumpjet
        already holds its landing reservation. Any more severe outcome fails.
    - **On a pass,** it makes its landing reservation.
    - **On a fail,** its movement is stopped (claim 17).
    - **Deploy direction.** A simple-deployer vehicle that is not held by magnetic lift starts to
      turn its heading towards the game-wide deploy direction (Import mapping). An imported
      8-step direction *b* gives the target heading (8192 × *b*) mod 65536 facing units, using
      integer arithmetic. Negative and larger inputs wrap through the same rule.
    - **Descent.** The target height is 0, so the jumpjet descends by *c* per tick (claim 8),
      without a look-ahead.
    - **Touchdown.** When the bridge-adjusted height *h* of claim 7 reaches 0 (including a
      jumpjet at deck height that is not yet marked as on the deck):
      - its speed fraction becomes 0
      - its position becomes the destination point, on the deck when it is above the terrain on
        a high-bridge cell
      - it marks its place in the occupancy record
      - its destination and move order are cleared.
    - **Sensors and adjacency reveal.** Completed touchdown supplies the cell-arrival update
      of S-0516 claims 13 and 22: any sensor disc transfers to the landing cell and the
      adjacency-reveal test applies after landing placement, before the later sight refresh
      and crate pickup. Take-off, in-flight cell crossings, cruising arrival and hovering
      supply neither event; S-0516 claim 22 owns the stationary-flight coverage quirk.
    - **If it is still in play and alive,** in the same tick:
      - its sight is refreshed at its new position
      - it releases its hover slot
      - it enters *grounded*
      - it attempts crate pickup in its landing cell under S-0520 claim 6; pickup rejection
        does not reverse the completed landing
      - a deploy-to-land vehicle's landing request ends (S-0517).
    - Backing: `measurable` (scenario J9 for ordinary landing checks, entering and living
      touchdown, S-0516 scenario CS20 for touchdown sensor transfer, adjacency reveal and
      their absence during flight, and S-0607 scenarios L3 and L6 for seized landing without
      the ordinary obstruction or water/beach refusal). Seized deploy-turn suppression remains `unmeasured`:
      these scenarios do not establish a seized simple deployer during descent.
      The dead/out-of-play touchdown exception is `unmeasured`:
      J9 has only living landings, and a normal kill changes the unit to crashing instead.
    - Confidence: high (water and beach, the outcomes, touchdown); medium (the crate pickup and
      the deploy direction)

### Orders

16. **A move order.** When a jumpjet is given a destination point:
    - **Magnetic seize.** This point-assignment rule also runs once at a successful magnetic
      seize, using the lifter's point defined by S-0607 claim 7. Claim 23's search applies
      then too. The resulting movement point stays fixed during the grip; arrival follows
      S-0607 claim 8. This is not a new player order accepted while seized.
    - **Release.** It releases any landing reservation it holds, unless it is grounded or
      claim 24's safety refusal retains the preceding movement.
    - **No point.** Except for claim 24's safety refusal while moving now, an instruction
      giving no point clears its movement point and moving flag, changing nothing else.
    - **Destination.** Otherwise a nearby-cell search (S-0212 claim 25) runs from the point's
      cell, for the hover locomotion class and the flying movement zone.
      - The destination becomes the centre of the cell found, at its terrain height (the deck
        height on a high-bridge cell). An infantry jumpjet instead takes a free standing position in
        that cell (S-0209).
      - Two kinds keep the exact point that was ordered: a balloon-hover vehicle, and a vehicle
        whose current or queued behaviour is *enter*.
      - If adjustment finds no point, the ordered point itself remains the movement
        destination, with the moving flag, flight state, speeds and target height unchanged.
        A unit already moving uses that point in the ordinary heading, distance, arrival and
        landing rules, even when it is not a cell centre. A descending unit does not get the
        successful-order re-ascent below. A grounded or hovering unit without a move order
        stays idle; merely retaining this point does not start it.
      - Off-map failure has a safety exception in claim 24.
    - Otherwise it becomes moving.
    - **Descending.** A *descending* jumpjet goes back to *ascending* with target height *H*.
      Any landing request it had for deploying ends, and an *unload* behaviour in progress is
      replaced by *guard* (S-0215; S-0517 owns deploying).
    - A grounded jumpjet takes off (claim 5). A hovering one leaves its hover in the next tick
      (claim 13).
    - Search settings and the infantry standing-position choice are claim 23.
    - A direct no-point instruction is distinct from an effective order-level destination
      clear: the latter invokes claim 17, not this claim's no-point row. Claim 24 specifies
      effective clears and redundant assignments that never reach movement.
    - Backing: `measurable` (J10 and J16) for successful orders and re-ascent, and
      S-0607 scenario L2 for the initial seize's movement-point adjustment.
      Direct no-point and failed-adjustment consequences are `unmeasured` (diagnostic J17):
      ordinary clicks do not establish either condition independently of order handling.
    - Confidence: high for the snapping, exceptions, retained-point rule and re-ascent.

17. **A stopped or failed landing.** When the movement of a moving jumpjet is stopped (a failed
    landing check, claim 15, is one case):
    - it releases its landing reservation
    - it runs a nearby-cell search (S-0212 claim 25) around its current cell, for its type's own
      locomotion class and movement zone
    - if a cell is found, it re-orders itself to that cell's centre (claim 16), at its terrain
      height plus the bridge height on a high-bridge cell
    - if no cell is found and it still has health, it is destroyed by damage equal to its
      remaining health with the demolition warhead, ignoring defences, with the no-escape marker
      and no attacker (so its kill is credited to no one). Its destination is cleared.

    **Magnetic-release safety departure (design).** When the lifter ends a grip by any
    event of S-0607 claim 12 other than its no-destination exception, no-cell failure
    instead follows S-0607 claim 13: do not self-destruct, irrespective of
    `quirks.jumpjet_no_cell_self_destruct`. Both a dropped unit and a set-down unit clear
    their movement point and commitment. A dropped unit follows S-0607 claim 15's fall:
    over its cell centre, with no point, it uses claim 9's direct ground reference rather
    than look-ahead; off-centre, the fall's detour supplies its own cell centre as its point.
    It takes its jumpjet update every tick until its fall ends, even while *hovering*,
    despite its cleared commitment (S-0607 claim 13). A set-down unit stays put and becomes
    eligible for restoration under S-0607 claim 20. The original can terminate with a
    fatal error when self-destruction occurs while the lifter is releasing its unit.
    S-0607 claim 9's no-destination exception ends the grip before this stop and keeps
    this claim's ordinary no-cell outcome, quirk switch and lack of kill credit unchanged.

    Stopping a jumpjet that is not moving does nothing. The search has claim 23's settings.
    A player stop that clears the order-level destination uses this rule (claim 24); it is
    not an instruction to erase the locomotor's point.
    - Backing: `measurable` (scenario J11) for rerouting, self-destruction and no kill credit.
      Exact damage amount, warhead, defence bypass and no-escape marker are `unmeasured`:
      J11's unprotected empty infantry unit dies under several damage mechanisms, so the
      result cannot distinguish those properties. Stopping a non-moving unit has
      `measurable` backing through J19's recorded idle cases. The magnetic-release
      replacement is design, not an observation of a completed original release (J27).
    - Confidence: high (re-routing); medium (self-destruction, which needs a situation with no
      reachable cell)

### Crash

18. **Crash motion.**
    - **Start.** When a jumpjet has started to crash (S-0312 claim 16), is above the surface and
      is not already *crashing* or *crashed*, and is not held by magnetic lift, its next update
      includes the extra crash drop *k* and enters *crashing* with target height −5 leptons.
      That update's ordinary physics still uses the preceding flight state and target height:
      its height may therefore rise before the extra drop. The −5-lepton target first affects
      ordinary height control in the following update (claim 3). A jumpjet that is neither
      moving nor moving now takes no update, so it does not start this fall until it next
      updates in the original. Claim 3's safety departure instead guarantees that next
      update for a pending aerial crash. An infantry jumpjet switches to its *air-death-start* sequence
      (S-0107). A seized dropped unit instead follows S-0607 claim 15's cell-centre detour
      before crash entry.
    - **Fall.** In each tick of *crashing*:
      - after that tick's ordinary physics height step (F4), its world height drops by an
        additional *k*, with the resulting world height truncated toward zero. Away from the
        surface, with target height −5, both the ordinary descent by *c* and this drop apply
      - its movement heading's target is set 11520 facing units (about 63°) clockwise ahead of
        the heading's current value, so it spins at the rate of S-0206 claim 14
      - its current speed continues to evolve under F1 and it keeps stepping
        along the heading (F5).
    - **Crossing a deck.** Over a high-bridge cell, the extra crash drop ends the ordinary
      jumpjet's fall when its height before that drop is at or above the deck and its height
      after it is below the deck. It does not snap its world height back to the deck.
    - **Ordinary crash end.** When its height above the surface falls below 1 lepton, or
      that drop crosses a deck:
      - it releases its hover slot
      - its maximum speed and target height become 0
      - it enters *crashed*.

      An ordinary jumpjet's impact then follows S-0312 claim 16. A dropped seized unit
      instead follows S-0607 claims 16 and 17: a clear cell leads to *descending*, not
      *crashed*, and blocked water can lead to sinking (claim 19).
    - **Magnetic lift.** A seized dropped unit does not spin. Its speeds are 0 and its
      target height is 0 leptons in each crashing update, rather than the ordinary
      −5-lepton target. Its crash rate starts afresh at its own type's crash value at
      every seize (claim 1). At each crashing update, the extra drop is determined by the current crash rate, and the resulting world height is truncated toward zero. The crash rate for the next update is 1.0 lepton per tick greater than the current rate; the addition uses double precision and its result narrows to single precision under S-0205 claim 5. Consequently, the first extra drop uses the starting rate, and each later extra drop uses the rate increased by 1.0 lepton per tick per tick (S-0607 claim 15).
    - Backing: `measurable` (scenario J12 for ordinary crash motion; S-0607 scenarios L3
      and L4 for the cell-centre detour, zero-speed fall, no spin, initial rate,
      drop-before-growth ordering and zero target height).
      Crash requests on idle units share claim 3's `unmeasured` limitation.
    - Confidence: high (fall and spin); medium (the hand-over to S-0312's impact)

19. **What sets a jumpjet sinking.** Only a dropped unit (S-0607 claim 13) is
    checked on impact. S-0607 claims 16 and 17 decide the outcome: a blocked water cell,
    not on a deck and not on beach, sets it sinking unless a vehicle or building underneath
    forces the blocked-land crash outcome. S-0504 claims 4 and 5 then govern sinking.
    - An ordinary jumpjet that was shot down is never set sinking by its movement, whatever cell
      it falls on.
    - The rest of the dropped unit's impact, including effects on what is underneath and
      *descending* when the cell is clear, belongs to S-0607 claims 16 and 17.
    - Backing: `measurable` (S-0607 scenarios L5 and L6 for the dropped-unit outcomes,
      including a vessel underneath and legal water/deck landings). Ordinary shot-down
      attribution remains `unmeasured`: J13 can record sinking but cannot attribute an
      impact-tick change to movement rather than impact/death rules.
    - Confidence: medium (the release condition was traced to the single place that sets it)

### Firing

20. **Flight states and firing.**
    - The jumpjet locomotor adds no refusal of its own to the firing check (S-0301 claim 9,
      rank 16, never comes from it). S-0301 claim 9's generic locomotor-refusal row does not
      assign any jumpjet flight-phase refusal; those belong to S-0310 claims 7 and 8.
    - The flight-phase refusals come from the firer's own checks, which use *moving now*
      (Terms): S-0310 claim 7 for a balloon-hover vehicle, and claim 8 for an infantry jumpjet
      with the jumpjet-turn flag.
    - This flight-state test permits firing while *grounded* or *hovering*, subject to all
      other firing restrictions. It gives the moving refusal while *ascending*, *cruising*,
      *descending*, *crashing* or *crashed*, unless a higher-ranked refusal applies.
    - An infantry jumpjet is also refused while its speed fraction (claim 6) is above S-0310
      claim 8's threshold.
    - Backing: `measurable` (scenario J14) for living flight phases. The speed-fraction
      refusal inherits S-0310 claim 8's backing: its precise threshold is `unmeasured` here,
      since J14 does not establish hovering with a controllable nonzero fraction independently
      of the flight-phase refusal. J4 separately tests the fraction's numeric value.
      The absence of a locomotor-specific refusal, and the *crashing*/*crashed* cases, are
      `unmeasured`: other firing refusals mask them for the dead units in these scenarios.
    - Confidence: high

### Random draws

21. **Neighbour step.**
    - **Cases.** A jumpjet takes a neighbour step in three cases:
      - it finishes ascending in a cell whose hover slot another jumpjet holds (claim 7)
      - it arrives to hover at such a cell (claim 12)
      - it descends to a water or beach destination whose slot is held (claim 14).
    - **Draw.** Each step takes one ranged draw over 0 to 7 from the scenario random stream
      (S-0202). The value is a step direction (glossary): the neighbouring cell in that direction
      becomes the jumpjet's new destination, through the move-order rule of claim 16, and it
      enters *cruising*. Infantry point adjustment can then take the additional draw of
      claim 23; the neighbour draw is not the total draw count of that whole order.
    - **No check before the draw.** Nothing tests the chosen neighbour first. An off-map or
      occupied neighbour is handled only by claim 16's nearby-cell search.
    - Backing: `measurable` (scenario J8 b)
    - Confidence: high (the draw and its range); medium (the handling of an off-map neighbour)

### Clearance, searches and exceptional orders

22. **Listings and self-clearance (Q-0978).** F3 reads actual ground-layer listings, not
    hover slots or reservations. A listed jumpjet supplies the 85-lepton allowance, including
    to itself: there is no self-exclusion. A deck-only listing supplies none.
    - Non-hover/cruise physics removes its own listing before evaluating clearance. Ordinary
      high hover has no listing left from ascent and therefore no automatic self allowance.
    - Hover/cruise physics leaves existing listings untouched rather than creating new ones.
      A low own-cell ascent-to-cruise transition can therefore retain its former ground
      listing. It counts whenever look-ahead is used, including for itself, until a listing
      removal. Hover/cruise cell crossings do not transfer it to the new cell. Direct-reference
      cases of claim 9 do not use this allowance at all.
    - Backing: `measurable` (J3, J18) for the predicate and normal high-flight absence.
      Low-flight retained-listing persistence is `unmeasured` pending J18's diagnostic:
      its history has not been isolated from reservations in an observation.
    - Confidence: high for the predicate; medium for that exceptional history.

23. **Nearby searches and standing positions (Q-0975).** The following settings complete
    S-0212 claim 25's single-cell search. Every row uses a 1 × 1-cell footprint, no further
    test, no no-overlay condition and no reference cell. The search itself takes no draw.

    | Use | Centre | Locomotion class | Movement zone | Reach region | Deck mode | Height condition | Bridge cells |
    |---|---|---|---|---|---|---|---|
    | Claim 16, vehicle not balloon-hover | ordered point's cell | hover | flying | none | never | no | excluded |
    | Claim 16, infantry or balloon-hover vehicle | ordered point's cell | hover | flying | none | centre carries a high bridge | no | allowed |
    | Claim 17 | unit's current cell | type's own | type's own, not simplified | none | never | no | allowed |
    | Animated landing-first undeploy, S-0517 claim 6 | vehicle's current cell | type's own, winged counted as track | type's own, not simplified | centre's region for that zone, read for its deck when on deck; none if that read has no region | vehicle is on deck | no | allowed |

    The last row selects the undeploy destination before the subsequent claim-16 search.
    S-0517 claim 6 owns when it is assigned. A magnetic seize also runs the applicable
    claim-16 search once, centred on the lifter's point (S-0607 claim 7), not necessarily
    its current position. The ordinary seized vehicle uses the non-balloon vehicle row:
    its lifter's occupied cell normally fails, leaving a free nearby cell as the movement
    point. Arrival drops it over that point (S-0607 claim 8), not necessarily onto the lifter.

    **Own-cell orders (Q-1011).** Claim 16's search does not excuse or lift a grounded
    unit's own occupancy. A nonempty searched occupancy record rejects the cell even
    when its only mark belongs to the ordered unit (S-0212 claim 25). On flat land a
    grounded vehicle's own mark therefore excludes its own cell. An ordinary vehicle
    gets a nearby qualifying cell, not guaranteed own-cell take-off. Balloon-hover and
    entering vehicles can keep the exact ordered point under claim 16 despite the search
    finding another cell; the balloon-hover vehicle still has no early-cruise transition.
    Infantry standing-position adjustment is subsequent to the search and supplies no
    exemption from this whole-record test.

    There is also an earlier order filter: outside an ion storm, a jumpjet-type infantry
    unit ignores a nonempty destination assignment whose cell equals its current cell.
    This applies while grounded too, before claim 16's search, so an ordinary own-cell
    player order does not start that infantry unit's ascent. This filter concerns the
    imported jumpjet-type flag, not merely use of the jumpjet locomotor.
    J2's former unconditional own-cell expectation was incorrect; claim 7 describes the
    ascent transition only when the accepted movement point actually is in that cell.

    Infantry adjustment uses the found cell's centre, not the ordered point's position
    class. The desired point for this choice has world height 0 leptons. It reads ground
    occupancy, except when the cell has a high bridge whose deck at its centre is at or
    below world height 0; then it reads deck occupancy. The final point nevertheless has
    terrain height plus 416 leptons on any high bridge.
    - The vehicle-present refusal and ground object-present/open-gate refusal of S-0208
      claim 3 apply, giving no adjusted point and no draw.
    - Otherwise one ranged draw over 0 to 3 from the scenario random stream selects the
      corresponding row R0 to R3 of S-0208 claim 3's table O. The first free side position wins:
      0 or 1 tries right, left, bottom; 2 tries left, bottom, right; 3 tries bottom, right,
      left. Centre and top are never selected. The draw still occurs if all side positions
      are taken, in which case adjustment fails.
    - An own-cell infantry order also takes this draw when adjustment is eligible. When a
      neighbour step reaches infantry adjustment, this draw follows claim 21's neighbour draw.
    - Backing: `measurable` (J2 and J16, including the own-cell filter and occupancy test, and
      S-0607 scenario L2 for the initial seize's nearby-cell search).
      The world-height-0 deck boundary is `unmeasured`
      (J16 diagnostic): a legal stock-map fixture at that boundary is not established.
    - Confidence: high for settings and choice rules.

24. **Destination clearing, stop, empty points and off-map failure (Q-0976, Q-0981,
    Q-1012).** An effective order-level destination clear invokes claim 17 whenever the
    jumpjet retains a movement commitment. This is not exclusive to player stop or to
    failed landing. It applies equally when deploy or behaviour processing supplies the
    effective clear. It does not send claim 16's direct no-point instruction. After the
    reroute the order-level destination is empty, while the replacement movement point
    and commitment survive. Without a commitment claim 17 does nothing. Behaviour
    changes alone are not destination clears: S-0213 claims 8 and 10 and S-0215
    claims 13 to 15 and 22 own which orders or processing outcomes assign an empty destination.
    The factory ascent report's direct destination discard is not such an assignment
    (claim 7; S-0417 claim 15), and neither is the approach step's direct removal of a
    gripped unit's destination (S-0215 claim 37 part e; S-0607 claim 9).

    **Redundant assignments.** Not every repeated request to assign an empty destination
    is an effective clear. An ordinary jumpjet vehicle ignores an unchanged order-level
    destination, including empty-to-empty, unless its destination-refresh state requires
    reassignment (claim 27). Equality means the same cell or the same object, not two
    objects sharing a cell. A balloon-hover vehicle can reject a requested clear under
    claim 28; this retains its destination, rather than substituting a different one.
    A suppressed assignment does not reroute or erase the movement point.

    **Infantry reassignment (Q-1030).** An infantry unit of a jumpjet type has no corresponding
    unchanged-destination suppression. Subject to claim 23's own-current-cell filter
    and ordinary order legality, assigning the same nonempty destination elsewhere,
    whether a cell or an object, repeats claims 16 and 23's movement-point adjustment.
    Eligible standing-position adjustment therefore takes its ranged draw over 0 to 3
    again, even if it chooses the preceding point. Search failure or the pre-draw
    refusals still have their stated outcomes; a draw is not unconditional.
    An empty-to-empty infantry assignment is an effective clear: with a movement
    commitment it repeats claim 17's reroute, including the subsequent infantry
    adjustment and eligible standing-position draw. Without a commitment it does nothing
    to movement. Stop and behaviour processing obey the same rule when they actually
    assign an empty destination; changing behaviour without assigning one does not reroute.
    A stop order for a balloon-hover type supplies two empty assignments, with target
    clearing between them (claim 28). That type flag does not give infantry the vehicle's
    equality suppression: if infantry retains its commitment through both assignments,
    both are effective clears and can reroute. Ordinary non-balloon infantry stop
    supplies one clear. The rule is per assignment, not one reroute per named order.

    **Deploy while hovering.** For a landing-first vehicle with a commitment, an effective
    deploy-order clear reroutes exactly as the table below. It keeps the replacement
    point for descent and touchdown, not an absent point. If the order-level destination
    was already empty and its repeated assignment is suppressed, the preceding movement
    point and commitment remain instead. A qualifying balloon-hover rejection instead
    retains even the nonempty order-level destination (claim 28). An equal horizontal
    replacement lets the landing request of S-0517 claim 5 produce descent under claim 13.
    A different replacement first causes cruise to that point. Landing and touchdown use the point then retained.
    With no initial commitment, the clear cannot create one or wake idle hover: the
    landing request alone does not bypass claim 3's update eligibility.

    A stop order of S-0213 claim 8 is one source of an effective clear. Its successful
    reroute has these outcomes:

    | Flight state when stopped | Successful claim-17 reroute |
    |---|---|
    | grounded with a movement commitment | keeps its commitment and replacement point; its next eligible jumpjet update takes off under claim 5, then follows ordinary flight and arrival towards that point |
    | ascending | stays ascending towards the replacement point |
    | cruising | stays cruising towards the replacement point; ordinary arrival applies |
    | hovering with a move order | remains moving; stays hovering only if the replacement horizontal point equals its position, otherwise claim 13 starts cruise |
    | descending | reservation released; re-ascends under claim 16 |
    | grounded or hovering without a movement commitment | claim 17 does nothing |

    Both searches of claims 17 and 16 apply, so the point is not necessarily the current
    position. Infantry can pick another side position in the same cell. The committed
    grounded row also covers a seized unit set down before take-off (S-0607 claim 13):
    set-down alone neither clears a successful reroute's commitment nor prevents take-off.
    Restoration requires that commitment to end (S-0607 claim 20); it cannot pre-empt
    flight merely because the unit is grounded. Failure follows claim 17's no-cell outcome,
    including its magnetic-release safety departure. Stop clears the target too (S-0213 claim 8); later arrival
    uses the then-current target. This is neither an immediate halt nor a command to finish
    the old destination.

    A direct no-point instruction in the original instead replaces the locomotor point
    with the absent point and clears its moving flag, without changing state or speeds.
    Ascending, cruising and descending still satisfy *moving now*; grounded and hovering
    become idle. It is not a promise to keep flying to the previous point.

    **Safety departure (design).** A failed adjustment that would retain an off-map point
    is refused: retain the preceding point, moving flag, state, speeds, target height and
    landing reservation. Do not clamp to the edge. An idle unit remains idle; an active
    one continues its preceding movement. Also refuse a direct absent-point instruction
    while *moving now*, retaining that preceding movement instead of navigating towards an
    absent point. Ordinary player stop still uses the reroute table, not this refusal.
    The original stores its supplied point before adjustment, with no such rejection.
    Later off-map queries can encounter S-0009 claim 44's stand-in. A safe original
    completion there is not established.
    - Backing: `measurable` (J19 and J22) for effective clears, suppressed assignments and
      the committed hovering deploy cases. An idle, uncommitted hovering deploy request
      remains `unmeasured` (J22 diagnostic): an ordinary completed flight usually retains
      a commitment, and no stock order sequence establishing that idle starting state is
      settled here. Infantry repeated nonempty and empty assignments have `measurable`
      backing through J23. The two-clear stop of balloon-hover infantry is `unmeasured`:
      no stock infantry fixture combining those properties is established (J23 diagnostic).
      Claims 27 and 28 give the refresh and balloon-hover rules
      and their separate backing; no balloon-hover destination substitution is asserted.
      The gripped unit's direct removal is S-0607 claim 9's: `unmeasured` there, medium.
      Original direct-empty and off-map aftermath are
      `unmeasured` (J17 diagnostic): ordinary input cannot independently establish these
      instructions. The refusal is design, not an original-behaviour claim.
    - Confidence: high for stop dispatch and unchanged flight state; medium for the
      complete original exceptional aftermath. The committed grounded set-down case has
      `measurable` backing through conditional scenario J27 a, with high confidence in
      its movement and restoration predicates. Stock reachability before the first
      take-off update remains unsettled; a run missing that timing is inconclusive.

25. **Zero flight fields (design, Q-0977).** For *S* = 0, the speed fraction and every
    cruise-band speed are 0. Horizontal travel from that speed is zero; vertical take-off
    and landing retain their own rules. For *T* = 0 the middle cruise band is empty, with
    no division by *T*. The importer warns for every type having either zero field.
    No quirk switch controls these safety departures.
    - Backing: `unmeasured` as original behaviour: the original divisions supply no safe
      replacement. The replacements are engine design rules, testable by J20 without
      running invalid data in the original.
    - Confidence: high for the division sites; design for the replacements.

26. **Flight and land recomputation (Q-0981).** This is S-0211 claim 5's land recompute,
    including S-0010 claim 19's possible animation and draw consequences.
    - Hover/cruise travel does not recompute either cell merely by crossing their boundary.
    - In the other flight states, the cell before every physics step is recomputed on
      removal of the old placement, even high above it or without horizontal travel.
      The resulting cell is also recomputed when the new position qualifies for ground-layer
      placement under claim 4. Low ascent/descent can thus recompute both sides of a cell
      crossing; high ascent recomputes the cell left, without new placement in the cell entered.
    - A crash's additional legal-position drop likewise recomputes its pre-drop cell and
      supplies another placement/recompute only when the new position qualifies under
      claim 4. Touchdown supplies its ordinary placement/recompute too. Multiple such
      events within a tick are not collapsed into one.
    - Backing: `measurable` (J21).
    - Confidence: high for placement events; medium for animation consequences, whose
      eligibility belongs to S-0010 claim 19.

27. **When a vehicle needs destination refresh (Q-1031).** The state starts absent.
    Successful placement into play temporarily enables it during initial idle-behaviour
    setup and clears it when that setup finishes. Any otherwise eligible destination
    assignment during that interval is reapplied even when unchanged. This includes
    the normal initial idle fallback's empty-to-empty clear for a fresh ordinary vehicle
    without a destination. This is an effective locomotion stop under claim 24, consuming
    refresh but not enabling point suppression. Without a movement commitment it cannot
    start flight or reroute. Type-specific fallback exceptions remain S-0215 claims 13
    and 14's rules, not an unconditional clear at every re-placement.
    - S-0207 claim 27 owns the later dock test, evaluated at each otherwise eligible
      destination assignment after unchanged-destination rejection, using the offered
      destination and the contact already held then. First contact, arrival and entry
      attempts do not independently trigger it. An unchanged offer without refresh
      does not reach it even if contact or occupancy has changed. A building destination
      is not the required cell destination. The type must have the teleport-capable flag,
      and the vehicle must have no warp-pending immobilization, no magnetic-lift source
      and must not be seized. Its dock contact must be a building permitting dock unloading.
      The requested destination must be a cell with no ground-layer vehicle, and its
      current locomotor must not be teleporting. The immobilization predicate applies
      independently of the current locomotor: it does not mean phasing in, the held
      countdown or a cooldown before another jump. An ordinary own-locomotor jump or
      factory exit does not create it (S-0207 claim 27).
      If a suspended locomotor can finish and restore its predecessor, that restoration
      enables neither refresh nor point suppression. That same assignment's otherwise
      due point goes to the restored predecessor, not to the former temporary locomotor,
      subject to already-pending point suppression and the restored locomotor's ordinary
      destination refusals (S-0207 claims 20 and 27). Eligibility uses the temporary
      locomotor's preceding movement state, not the point offered by this assignment.
      Otherwise movement is stopped, current behaviour is
      cleared, *enter* is queued without an immediate start, and both destination refresh
      and suppression of one nonempty movement-point application become pending.
      S-0207 claim 27 defines restoration eligibility, including temporary driving.
    - An effective changed or refreshed unchanged assignment consumes refresh already
      pending, even if its movement-point adjustment fails. Claim 28's rejected clear
      does not consume it. Refresh newly enabled by the dock fallback can remain after
      that very assignment, unless another effective assignment consumes it.
      Point suppression is separate: it skips the next otherwise due nonempty movement
      point, including the triggering assignment's own point. It also covers a teleporting
      locomotor's landing point (S-0207 claim 27). Thus the usual qualified
      fallback installs the order-level cell destination without its movement point,
      consumes suppression and leaves refresh pending. A qualifying equal assignment
      can reach the dock test again, with no once-per-entry limit. An empty clear stops
      locomotion but does not consume nonempty-point suppression. The resulting
      teleport/dock movement remains S-0207 claim 27's scope.
    - Apart from initial placement and that qualified teleport/dock case, stop, deploy,
      move and behaviour changes do not enable refresh. Initial placement can reach its
      temporary interval in ordinary play; the later dock case requires the independent
      teleport-capable flag and is not an ordinary jumpjet flight transition. It must not
      be inferred from flight state, retained commitment, an empty destination or a
      landing request.
    - Backing: `measurable` (J24) for placement and ordinary-order nonactivation.
      The later teleport/dock activation, same-assignment point routing after restoration,
      and a retained refresh state on a jumpjet are `unmeasured` here: no stock jumpjet
      sequence establishing those type properties and docking conditions is settled.
      Nor is an immobilized, non-teleporting dock fixture established. J24 includes
      conditional diagnostics for these limitations; S-0207 claim 27 owns the shared
      immobilization predicate and its separate backing.
    - Confidence: high for activation and consumption predicates; medium for ordinary
      stock reachability beyond placement.

28. **Balloon-hover destination retention, not substitution (Q-1031).** A vehicle of a
    balloon-hover type rejects an assignment of an empty destination when it currently
    has both a nonempty order-level destination and a target, and at least one row below
    holds. Nonempty assignments do not have this exception. The rule applies to a seized
    vehicle too, before the seized unit's destination refusals (S-0607 claims 9 and 13).

    | Relationship at assignment | Retain destination? |
    |---|---|
    | Destination is the target object itself | yes |
    | Destination is a cell and the target is its first ground-layer listed object | yes |
    | Otherwise, the target direction is at most 16384 facing units from the current turret facing | yes |
    | None of the above | no; ordinary assignment and equality rules apply |

    The angular test uses the signed wrapped difference in −32768 to 32767 facing
    units and its absolute value, with integer comparisons and no rounding. The target
    direction is S-0204 claim 8's horizontal direction from the vehicle to its target.
    The facing is the turret facing, not the jumpjet movement heading; the test still
    uses that facing for a type without a turret.
    - Rejection leaves the old order-level destination, movement point, commitment and
      landing reservation intact. It makes no nearby search or movement-position draw.
      If the current behaviour, or queued behaviour when no current behaviour exists,
      is not *attack*, the result makes *attack* current and clears the queued behaviour.
      Already being in *attack* leaves that behaviour alone.
    - This applies before unchanged-destination suppression, even with refresh pending.
      It does not manufacture a new destination from the target or from remembered
      movement. Having a target alone is insufficient: an already empty destination
      cannot be retained by this rule.
    - This is reachable through ordinary orders, not only a special state: give a move
      destination, then attack a legal target while retaining that destination and while
      the target is within the angular bound. Attack assigns its target before requesting
      the empty destination, so this clear can be rejected. Behaviour processing that
      requests a clear while the same conditions hold can also trigger retention.
      An assignment after target clearing cannot trigger it.
    - **Stop versus deploy.** Stop initially requests the clear while the target is still
      present, then clears the target. For a balloon-hover type it requests the clear
      again after target clearing. A balloon-hover vehicle therefore ends stop with
      an empty destination: if the first clear was rejected, the second is effective
      and reroutes with a commitment; if the first was effective, the second unchanged
      empty assignment is suppressed. A first rejection can still have changed its
      current behaviour to *attack*, despite the later target clear.
      Deploy requests its single clear before clearing the target and queuing *unload*.
      It can therefore retain a nonempty destination despite ending with no target.
      It does not make stop's second attempt. A nonempty move assignment bypasses
      retention altogether. S-0213 claims 8, 10 and 21 own the remaining order effects.
    - Backing: `measurable` (J25) for ordinary move/attack/stop reachability, retention,
      angle boundary, behaviour result and absence of a reroute. A stock deployable
      balloon-hover fixture remains `unmeasured`: J25 does not assume such a type exists.
      The seized vehicle's case is S-0607 claim 9's: `unmeasured` there, medium.
    - Confidence: high for predicates and retained values; medium for the stock
      deployable balloon-hover fixture.

### Flying-object registration and the in-the-air test

29. **Registration as a tracked flying object (Q-1500).** This claim fixes when a unit moving
    with the jumpjet locomotor is a tracked flying object. Two searches read the registration:
    the flying-object search of area damage (S-0303 claim 6) and the airborne candidates of
    target acquisition (S-0309 claim 9).
    - **Start.** It becomes registered in the update in which it takes off (claim 5), when it
      leaves *grounded* for *ascending*. Its height then is normally 0. Registration does not
      wait for a first rise, a height or a flight state, and from that update on detonations
      above the terrain can find it. It is registered in the sector of the cell it is over at
      that moment. A unit that is already registered is not registered a second time.
    - **Seized units.** A unit of any type seized by magnetic lift (S-0607 claim 5) gets a
      fresh jumpjet locomotor in *grounded*. When the seize's point assignment (claim 16)
      finds a cell and gives it a movement commitment, its first eligible jumpjet update
      takes off and registers it by the same rule as a unit of a jumpjet type. If the initial
      search finds no cell, it has no movement commitment and stays *grounded*, without
      take-off registration (claims 3, 5 and 16; S-0607 claim 7).
    - **Sector updates.** In each update of *ascending* or *cruising* in which the unit is over
      a different cell from the one recorded at its last registration or sector update, its
      registration moves to the sector of the new cell, with S-0303 claim 11's arrival order.
      *Hovering*, *descending* and *crashing* updates do not move it between sectors, so a
      unit that moves horizontally in one of them keeps the sector of its last update. For
      an ordinary crash fall (claim 18) this is moot: the unit has no health (S-0312 claim 16),
      so S-0303 claim 6 never takes it as a candidate.
    - **End.** It stops being registered at the first of these events:
      - **touchdown** (claim 15): in the touchdown tick, together with the other effects for a
        unit still in play and alive, before it enters *grounded*
      - **crash end** (claim 18): when it enters *crashed*, before S-0312 claim 16's impact.
        Its own impact effects can therefore never find it through the flying-object search
      - **going off the map** for any reason: for example entering a transport or a
        building, or its removal.
    - **Not an end.** None of these ends registration:
      - the start of a crash; a crashing unit stays registered until the crash ends
      - hovering or cruising at any height
      - the end of a magnetic grip. A dropped unit whose fall ends over a clear cell goes on to
        *descending* (S-0607 claims 16 and 17) and stops being registered at its touchdown.
        A set-down unit stops at its touchdown too. Restoring a seized unit's own locomotor
        (S-0607 claim 20) does not change its registration.
    - **Placed on the map in the air.** A unit placed on the map is registered at its placement
      only when it is then airborne (claim 30) and its type has the counts-as-aircraft flag
      (Import mapping). This rule is the same for every kind of unit. A jumpjet placed in the
      air without that flag is not registered until it next takes off from *grounded*.
    - **Touchdown while falling** (`quirks.jumpjet_falling_touchdown_tracked`, default on, YR
      behaviour). The touchdown event does not end registration when the unit is then in
      the bridge or parachute falling state of claim 4, or is no longer alive or in play. Such
      a unit stays registered while *grounded*, and its next take-off does not register it
      again. It stops being registered at a later touchdown or crash end that does end it,
      or when it goes off the map. Off (fixed behaviour): every touchdown ends registration.
    - Ion storms, which would prevent the take-off and so the registration, do not occur in
      skirmish (Edge cases and quirks).
    - Backing: `measurable` (scenario J28 a to d for the start, touchdown, crash and hovering
      boundaries; J28 e for a seized unit). The initial-seize no-cell exception is
      `unmeasured`: no ordinary setup establishing that search failure is supplied here
      (claim 16; S-0607 claim 7). Sector updates are `unmeasured` here: S-0303
      scenario A20 measures sector crossings of cruising jumpjets only.
      The placement rule and the falling touchdown are `unmeasured`: J28 does not establish
      a jumpjet placed in the air, nor a touchdown in the falling state.
    - Confidence: high for the start, touchdown, crash end and the seized unit's
      registration. Medium for going off the map, the placement rule and sector updates.
      Low for the falling touchdown.

30. **In the air for combat (Q-1501).** A unit moving with the jumpjet locomotor is in the air
    (S-0307 Terms, *airborne*) exactly when it is on the map and its height above the surface
    *r* is at least 208 leptons. *r* is claim 4's signed integer height: above the terrain,
    or above the deck when the on-deck state is set. The comparison is exact, with no rounding.
    This is the only test for these units in weapon choice (S-0308 table P rows 18 and 19),
    range (S-0307) and target acquisition (S-0309). S-0303 claim 10's halving is for aircraft
    only and never applies to these units.
    - It does not depend on the jumpjet state, on moving or moving now, on the type's cruise
      height, or on registration (claim 29). A *hovering* or *cruising* jumpjet whose wobble
      takes it below 208 leptons is not in the air there. An *ascending*, *descending* or
      *crashing* one at 208 leptons or more is in the air.
    - A jumpjet vehicle and a jumpjet infantry unit use the same test.
    - A unit seized by magnetic lift uses the same test, whatever its own kind and whether it
      is gripped, dropped or set down.
    - **Over a high bridge without the on-deck state,** the test uses *r*, not claim 4's
      deck-adjusted *r*′. A jumpjet at deck height there (*r* = 416 leptons) is in the air,
      although claim 4 lists it in the ground layer. At *r* = 207 leptons it is not.
    - A unit off the map is not in the air.
    - Aircraft use S-0501 claim 18's in-the-air answer instead.
    - Backing: `measurable` (scenario J29)
    - Confidence: high

31. **A low jumpjet can be both registered and listed.** A unit is registered (claim 29) and, when claim 4's evaluated-draw-layer condition permits a new listing, is listed in its cell layer during:
    - its take-off update and the rest of its ascent, while *r* < 208 leptons or *r*′ = 0
    - its descent, from the moment *r* < 208 leptons or *r*′ = 0 until touchdown
    - a crash fall below 208 leptons, where it has no health and is no candidate
    - a listing retained from a low flight-state transition while it hovers or cruises
      (claims 4 and 22)
    - the falling touchdown of claim 29, while it stays registered on the ground.

    While both hold, a detonation above the terrain finds it once through the flying-object
    search and once in its cell's layer (S-0303 claim 6). Its two entries are not merged.
    What the second entry does is S-0303's. At 208 leptons or more, in any state, only the
    registration can find it; a grounded unit that is not registered is found only in its cell.
    - Backing: `measurable` (scenario J28 b and c)
    - Confidence: high for take-off, ascent and descent; medium for retained listings

32. **Disc-laser attacks do not force a descent (Q-1504 question 2).** Firing a
    disc-laser weapon, including its charge-up and final damage step (S-0615 claims 12,
    13, 15 and 16), does not itself lower the firer, change its target height or put it
    in the ground render layer.
    - A jumpjet hovering at its movement point with a target keeps hovering under
      claim 13. Its height still follows claims 8 and 9, including wobble; this is not
      a separate attack-height mode. A balloon-hover type keeps full target height
      during cruise under claim 11, whether or not it is attacking.
    - This does not mean every attacking jumpjet is above the ground-layer boundary.
      Low ascent, descent, retained low target height or wobble can satisfy claim 4's
      height predicate independently of the weapon. The evaluated layer remains
      S-0102 claim 3's height result, not a test of whether a disc laser is active.
      S-0102 claim 4 separately owns membership in the drawing lists.
    - A settled hover over flat terrain that remains at least 208 leptons above the
      surface therefore does not become a ground-layer candidate for S-0309 claim 7's
      map-wide search merely by attacking. Retaliation is a separate route to acquiring
      the firer (S-0309 claim 18; S-0615 claim 16).
    - Backing: `measurable` (scenario J30)
    - Confidence: high

## Formulas

All heights and positions are signed integers in leptons. Current and maximum speed and wobble
phase are double precision. The acceleration, climb rate, crash rate and wobble rate are imported
single-precision values widened exactly to double precision for arithmetic. Every floating-point
sum, difference, product and quotient below rounds toward zero to a 53-bit significand, including
the operations inside F1's speed fraction and F2's phase update. S-0205 claims 4, 5 and 8 govern
precision, rounding and exponent range. With `quirks.chop_rounding` off, each such operation
instead rounds to nearest, ties to even. Integer conversions still truncate toward zero
(S-0205 claim 7); "trunc" denotes that conversion. Comparisons and min/max choose exact operand
values without rounding. Sine and cosine mean the table trigonometry of S-0205 claim 6, not exact
mathematical functions. Integer arithmetic follows S-0205 claims 1 to 3.

**F1. Speed (claim 6).** For each physics step, let *q* be the speed after the acceleration stage:

| Condition | *q* |
|---|---|
| *v*<sub>m</sub> > *v*<sub>c</sub> | min(*v*<sub>c</sub> + *a*, *S*) |
| *v*<sub>m</sub> ≤ *v*<sub>c</sub> | *v*<sub>c</sub> |

The resulting current speed is

| Condition | *v*<sub>c</sub>′ |
|---|---|
| *v*<sub>m</sub> < *q* | max(*q* − 1.5 × *a*, 0) |
| *v*<sub>m</sub> ≥ *q* | *q* |

Thus the braking comparison uses the speed after acceleration, and braking also applies in that step if acceleration overshoots the maximum speed. The acceleration result is capped at *S* and the braking result at 0, not at the maximum speed. If the speeds are equal, the current speed is unchanged. The speed fraction is *v*<sub>c</sub>′ / *S*, in double precision, except that *S* = 0 gives fraction 0 under claim 25's design rule.

**F2. Wobble and desired height (claim 9).** Here 2π denotes the stored double
6.283185307179586 radians, not an exact real constant. Evaluate the inner division, outer
division and phase addition separately under the floating-point rule above.
- In *hovering* and *cruising*, without the no-wobble flag: φ′ = φ + 2π / (15 / *W*).
- Otherwise φ′ = 0.

*h*<sub>d</sub> = trunc(sin φ′ × *d* + *h*<sub>t</sub>), with sine and multiplication in double
precision.

**F3. Ground reference (claim 9).**
Use look-ahead only in states other than *grounded* and *descending*, and within those
states when either the movement point is in another cell or the type is balloon-hover.
This includes *crashing*, even at zero horizontal speed; otherwise use the direct reference.
- **Direct.** Let *f* be the terrain height at the jumpjet's position. On a high-bridge cell,
  when *z* ≥ *f* + 416 − *k*, use *f* + 416; otherwise use *f*.
- **Clearance of a cell.** clear(cell) = *f*(cell centre) + *A*(cell) + *B*(cell), where:
  - *A* = 104 × the building's art height when a building is listed in the cell's ground layer,
    else 85 when infantry, a vehicle or an aircraft is listed in that layer, else 0
  - *B* = the bridge height (416) on a high-bridge cell, else 0.
- **Look-ahead.** Let *C* = clear(own cell) and *N* = the clearance of the neighbouring cell in
  direction *e*, the 8-step facing of the movement heading (S-0204 claim 10). *N* counts only while
  *v*<sub>c</sub> > 0; otherwise *N* = *C*.
  - The original adds the neighbouring cell's *B* to *C* instead of to *N*
    (`quirks.jumpjet_lookahead_bridge`).
  - *g* = *N* when *N* > *C*; otherwise *g* = trunc((*N* + *C*) / 2).

**F4. Height step (claim 8).** Let *u* = *z* − *g* (the height above the ground reference).
- If *u* < *h*<sub>d</sub>:
  - *z*′ = *g* + *h*<sub>d</sub> when *u* + *c* ≥ *h*<sub>d</sub>
  - else *z*′ = trunc(*z* + *c*).
- If *u* > *h*<sub>d</sub>:
  - *z*′ = *g* + *h*<sub>d</sub> when *h*<sub>d</sub> ≥ *u* − *c*
  - else *z*′ = trunc(*z* − *c*).
  - *z*′ is never below the terrain, or the deck selected by F3's direct reference.
- If *u* = *h*<sub>d</sub>, the world height is unchanged.
- **Ground hold.** Let *u*⁺ = max(*u*, 0). If the jumpjet is not over its destination cell and
  *u*⁺ < trunc(*h*<sub>d</sub> / 2), then *v*<sub>c</sub> = 0.

**F5. Horizontal step (claim 10).** With heading *F* (facing units, 0 = map north, clockwise),
let *F*ₛ = *F* for *F* < 32768, otherwise *F* − 65536. This signed representative is used only
for this angle calculation. Let *v* = trunc(*v*<sub>c</sub>) and
θ = (*F*ₛ − 16383) × (−0.00009587672516830327 radians per facing unit), in double precision:
- *x*′ = trunc(*x* + *v* × cos θ)
- *y*′ = trunc(*y* − *v* × sin θ).

Map north is −*y*; map east is +*x*. The angular factor is the stored double approximation to
−2π / 65534. The shift and factor make θ slightly off the exact angle of *F*, but the table
quantisation can remove that discrepancy. With *v* = 10 from positive integer coordinates,
using the table values and round-toward-zero arithmetic:

| Heading | Δ*x* | Δ*y* |
|---|---|---|
| north | 0 | −10 |
| east | +10 | 0 |
| south | −1 | +10 |
| west | −10 | −1 |
| north-east | +7 | −8 |
| south-east | +7 | +7 |

With *v* = 1, a jumpjet heading exactly east moves one lepton east. Tiny nonzero table values
at nominal half/full turns still cause the southward and westward cross-axis drift under
round-toward-zero arithmetic.

**F6. Cruise bands (claim 11).**
- **Remaining turn.** δ = (desired heading − current heading) mod 65536, read as 0 … 65535. Its
  256-step facing is *D* = ((⌊δ / 128⌋ + 1) div 2) mod 256.
- **Bands.** With ρ the truncated distance, *S* the speed field and *T* the type's jumpjet turn-rate field as imported, before S-0206 claim 6's conversion (all
  divisions of integers truncate toward zero):

  | Band | *v*<sub>m</sub> | *h*<sub>t</sub> |
  |---|---|---|
  | ρ < *S* | *S* / 8 | *H* / 2 when no target; otherwise unchanged |
  | ρ < 2*S* | *S* / 4; when *D* > *T*: max(*S* / 10, 1.0) | *H* / 2 when no target; otherwise unchanged |
  | ρ < (50 × *S*) div *T* | *S* / 2; when *D* > 5*T*: max(*S* / 5, 1.0) | trunc(*H* × 0.75) when no target; otherwise unchanged |
  | otherwise | *S* | *H*, whether or not it has a target |

- The speed divisions are of integers. The floor of 1.0 is applied to the result.
  Under claim 25's design rule, *T* = 0 makes the middle band empty, and *S* = 0
  gives speed 0 in every band, including any speed-floor alternative.
- In the original, *D* comes from the unsigned difference, so a small anticlockwise remaining
  turn reads as nearly 256 (`quirks.jumpjet_turn_slowdown_one_sided`).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| arrival radius | 20 | leptons | claim 11: ρ < 20 |
| braking factor | 1.5 | × acceleration | F1 |
| wobble base rate | 15 | ticks per wobble at *W* = 1 | F2 |
| take-off fraction | 1/4 (1/2 balloon-hover vehicle) | of target height | claim 7 |
| ground-hold fraction | 1/2 | of desired height | F4 |
| ordinary crash target height | −5 | leptons | claim 18; seized crash target is 0 |
| crash spin lead | 11520 | facing units | claim 18 |
| magnetic-lift crash growth | 1.0 | leptons per tick, per tick | claim 18 |
| occupied-cell clearance | 85 | leptons | F3 |
| building clearance unit | 104 | leptons per art-height level | F3 |
| bridge height | 416 | leptons | F3, claims 7, 15 and 17 |
| band speed divisors | 8, 4, 10, 2, 5 | of *S* | F6 |
| band speed floor | 1.0 | leptons per tick | F6 |
| band distance factor | 50 | × *S* / *T* | F6 |
| middle-band height factor | 0.75 | of *H* | F6 |
| low-band height divisor | 2 | of *H* | F6 |
| heading shift | 16383 | facing units | F5 |
| angle divisor | 65534 | facing units per 2π | F5 |
| neighbour draw range | 0 to 7 | step direction | claim 21 |
| absent-key defaults | see Import mapping | varies by field | engine constants |

## State and ordering

| State | Event (checked each tick after the physics step) | Next state | Effect |
|---|---|---|---|
| grounded | eligible update with a movement commitment, even with empty order-level destination | ascending | speeds 0, fraction 1 (0 for zero speed, design), *h*<sub>t</sub> = *H*; registered (claims 5, 24, 25 and 29) |
| ascending | *h* > ¼ *h*<sub>t</sub> (½ balloon-hover vehicle) | ascending | *v*<sub>m</sub> = *S*, heading towards destination |
| ascending | over the destination cell, past that fraction, not balloon-hover | cruising | *h*<sub>t</sub> = *r* (claim 7) |
| ascending | *h* ≥ *h*<sub>t</sub>, slot free | hovering | slot claimed |
| ascending | *h* ≥ *h*<sub>t</sub>, slot held | cruising | neighbour step |
| hovering | destination elsewhere | cruising | slot released |
| hovering | a descent row of claim 13 applies | descending | slot released |
| cruising | ρ < 20, landing case | descending | speeds 0, *h*<sub>t</sub> = 0 |
| cruising | ρ < 20, hover case, slot free | hovering | speeds 0, slot claimed |
| cruising | ρ < 20, hover case, slot held | cruising | neighbour step |
| descending | water or beach destination, neither entering nor held by magnetic lift | hovering or cruising | as claim 14 |
| descending | landing check fails | (stopped) | claim 17 |
| descending | move order | ascending | *h*<sub>t</sub> = *H* |
| descending | bridge-adjusted height *h* = 0 | grounded | touchdown effects (claim 15); registration ends (claim 29) |
| any but crashing, crashed | crash started, above the surface, not held by magnetic lift | crashing | pending aerial crash guarantees the next update (design); *h*<sub>t</sub> = −5 leptons; extra drop this tick, new ordinary height target next tick |
| crashing, ordinary unit | height < 1 lepton or extra drop crosses a deck | crashed | slot released; registration ends (claim 29); S-0312 claim 16 impact |
| crashing, dropped seized unit | height < 1 lepton or extra drop crosses a deck | descending, sinking or crashed | S-0607 claims 16 and 17 |

Ordinary state outcomes for a tick use that tick's physics results. A transition to a new state,
speed maximum or target height affects physics beginning with the next tick. Arrival is the
exception for its stated immediate effects: its horizontal position is at the destination and
both speeds are zero in the arrival tick (claim 12). A move order and crash entry take effect
at the times specified in claims 16 and 18. A neighbour-step destination is observable in the
tick in which the transition requiring that step occurs. Its one ranged draw is not deferred
until the following tick. Ordinary jumpjet physics itself takes no random draw (claim 21).

## Edge cases and quirks

- **Step truncation** (`quirks.jumpjet_step_truncation`, default on, YR behaviour).
  - On: the step uses F5's shifted angle, table trigonometry and separate coordinate
    truncation. Southward and westward steps drift one lepton across the line of travel.
    Diagonal steps can differ by one lepton between axes (F5).
  - Off (fixed behaviour): x′ = x + round(v × sin α), y′ = y − round(v × cos α), with α = 2π *F* / 65536
    and rounding half away from zero. In this off alternative only, sine and cosine mean the
    mathematical functions evaluated in double precision. The surrounding arithmetic follows
    S-0205 claim 5. The integer displacement is added to the old integer position.
- **One-sided turn slowdown** (`quirks.jumpjet_turn_slowdown_one_sided`, default on, YR behaviour).
  - The cruise bands' remaining-turn test uses the clockwise difference only. A jumpjet that
    still has to turn anticlockwise by even a small amount takes the slower speed of the band.
  - Off (fixed behaviour): use the smaller of the clockwise and anticlockwise differences (|δ| folded
    into 0 … 32768) in F6.
- **Look-ahead bridge height** (`quirks.jumpjet_lookahead_bridge`, default on, YR behaviour).
  - On: the bridge height of the cell ahead is added to the jumpjet's own cell's clearance,
    not the ahead cell's. On level terrain with no objects, approaching a bridge therefore
    gives a 208-lepton reference increase instead of 416 leptons (F3).
  - Off (fixed behaviour): add each cell's bridge height to that cell's clearance (F3).
- **Bridge early-cruise height** (`quirks.jumpjet_bridge_early_cruise_height`, default on,
  YR behaviour).
  - On: an own-cell early cruise retains height above the surface *r* as its target, even
    when the take-off threshold used the smaller bridge-adjusted *h*. Height control can
    therefore make it climb again above the deck rather than retain its current clearance.
  - Off (fixed behaviour): retain *h* instead of *r* at this transition.
  - Backing and confidence are claim 7's; J2 gives the original-behaviour falsifier.
- The first three switches above inherit claims 10, 11 and 9's confidence and backing
  respectively. J6, J7 and J5 give their original-behaviour falsifiers. Off alternatives are
  design rules, not claims about the original.
- **Zero speed or zero turn rate.** A type with a speed field of 0 divides by zero in its speed
  fraction (F1), and one with a turn rate of 0 divides by zero in its middle band (F6).
  In the original, *S* = 0 gives an infinite or undefined speed fraction,
  and *T* = 0 is an integer division by zero that stops the program. Backing: `unmeasured`,
  confidence high for the divisions: no scenario here isolates their consequences before other
  invalid-data effects. Claim 25 gives the approved safety replacement, with no quirk switch.
- **No landing slot.** A jumpjet that cannot land and finds no cell nearby destroys itself
  (claim 17). This is the original's behaviour and stays the default
  (`quirks.jumpjet_no_cell_self_destruct`, default on, YR behaviour). Off: it clears its
  movement point and commitment. At an integer height above the surface of 1 lepton or more,
  it re-enters *hovering* over its current cell, neither descending nor flying on. At height
  0 leptons or less, it stays put and becomes *grounded*, including from *ascending* at height
  0. For every seized set-down unit in this switch-off outcome, whatever route brought it
  there, S-0607 claims 9 and 20 apply: while hovering it is not restored and retries the
  search at the end of each of its turns (S-0203), not at a fall-back to rest. Claim 17's
  magnetic-release safety departure overrides both switch settings for lifter-ended release,
  but not S-0607 claim 9's no-destination exception. Backing and confidence are claim 17's,
  and the off alternative is a design rule. The seized set-down unit's retry cadence is S-0607
  claim 20's: `unmeasured`, a design rule for the switch-off setting, with medium confidence
  for the test at the end of every turn.
- **Falling touchdown keeps registration** (`quirks.jumpjet_falling_touchdown_tracked`,
  default on, YR behaviour). On: a touchdown while the unit is in the bridge or parachute
  falling state, or no longer alive or in play, leaves it registered as a tracked flying object
  (claim 29), so on the ground it can still be found by the flying-object search as well as in
  its cell (claim 31). Off (fixed behaviour): every touchdown ends registration. Backing and
  confidence are claim 29's.
- **Ion storms.** An ion storm makes every flying jumpjet of the affected house crash, and stops a
  grounded one taking off. Ion storms do not occur in skirmish, so this is out of scope.

## Import mapping

| Key | Feeds | Parse form | Absent-key default |
|---|---|---|---|
| type key `JumpJet` | jumpjet type (glossary); infantry own-cell destination filter in claim 23 | boolean | no |
| type key `BalloonHover` | balloon-hover type (S-0310 Terms), including destination retention in claim 28 | boolean | no |
| type key `Teleporter` | teleport-capable flag in claim 27's later dock activation; independent of using jumpjet locomotion | boolean | no |
| building type key `DockUnload` | permits dock unloading in claim 27 | boolean | no |
| type key `JumpjetTurnRate` | *T* (claims 11, F6; rate: S-0206 claim 14); zero warns and gives an empty middle band (design, claim 25) | integer | engine default 4 |
| type key `JumpjetSpeed` | *S*; zero warns and gives zero fraction and all cruise speeds (design, claim 25) | integer, leptons per tick | engine default 14 |
| type key `JumpjetClimb` | *c* | decimal, single precision | engine default 5.0 |
| type key `JumpjetCrash` | *k* | decimal, single precision | engine default 5.0 |
| type key `JumpjetHeight` | *H* (S-0102 owns its minimum) | integer, leptons | S-0102 |
| type key spelled "jumpjet" and "accel" run together, the first letter of each part a capital (keys are case-sensitive, S-0001) | *a* | decimal, single precision | engine default 2.0 |
| type key `JumpjetWobbles` | *W* | decimal, single precision | engine default 0.15 |
| type key `JumpjetDeviation` | *d* | integer, leptons | engine default 40 |
| type key `JumpjetNoWobbles` | no-wobble flag | boolean | no |
| type key `ConsideredAircraft` | counts-as-aircraft flag: registration of a unit placed on the map in the air (claim 29); other uses belong to the specs that read it | boolean | no (S-0105 Import mapping) |
| infantry type key `JumpJetTurn` | jumpjet-turn flag (claim 10; S-0310 claim 8) | boolean | no |
| vehicle type key `IsSimpleDeployer` | deploy-to-land vehicle, with the next row (S-0517) | boolean | no |
| vehicle type key `DeployToLand` | deploy-to-land vehicle; full-height cruise (claims 11 to 13) | boolean | no |
| building art key `Height` | building clearance height (claim 9, F3) | integer, height levels of 104 leptons | engine default 2 |
| `[AudioVisual]` `DeployDir` | the game-wide deploy direction (claim 15) | integer, 8-step direction | engine default 0 at program start; a later missing layer keeps the current value |
| `[JumpjetControls]` `TurnRate`, `Speed`, `Climb`, `Acceleration`, `WobblesPerSecond`, `WobbleDeviation` | nothing (claim 2) | read and ignored | — |
| `[JumpjetControls]` `CruiseHeight` | the layer threshold (S-0102) | integer | S-0102 |
| Type movement class and movement zone | own-class/zone searches of claim 23 | keys, parse forms and defaults: S-0211 Import mapping | S-0211 Import mapping |
| No additional rules, art or map key | claims 3, 4, 22 to 24 and 26, including the two-height listing test, own occupancy, repeated infantry assignments, effective destination clears and runtime refresh/retention states of claims 27 and 28 | engine behaviour and the stated safety departures, not imported tuning | no configurable default |

- **Key case.** Key names are matched case-sensitively (S-0001 claim 13). Changing the
  letter case of a key above does not set that field. If no layer supplies the exact key,
  its absent-key default applies; otherwise the value from an earlier layer remains.
- **Layering.** Each key keeps its value when a later layer omits it (S-0006).
- **Backing.** The engine defaults are those of the program when no layer gives the key.
  Their flight effects are `measurable` through named test files in J1 c, confidence high.
  Defaults for flags that no J1 c flight can distinguish remain `unmeasured`: the files can
  omit the key, but this scenario has no separate target, deploy or landing observation for
  each flag. The building-height default is `measurable` in J5 and the deploy-direction
  default in J9, confidence high.

## Measurement scenarios

**Common setup.**
- Stock data, no engine add-on and no mods (the named test files of a scenario excepted); game-speed index 2; 800 × 600 pixels.
- One human test house and one bystander computer player outside the test area under the
  glossary's void rule.
- A purpose-made flat map with:
  - a land lane, a beach lane and a water lane
  - a high bridge
  - a single-cell pocket enclosed by buildings
  - an open area of 20 × 20 cells.
- Units: one stock jumpjet infantry type, one stock jumpjet vehicle type if stock data has one,
  and one balloon-hover type if stock data has one. Named test file *jumpjet-flight-test*
  supplies explicitly identified variants when stock data cannot establish a subcase. It changes
  only the fields that subcase names and is removed after the run. Every subcase using it is
  recorded as named test data, not a stock-data run.
- Use the measurement setup's read-only recorder where supported. Record position, world height, height above
  the surface, body facing, movement heading if exposed, speed fraction, current behaviour, target and
  destination. Propose any missing recorder support to the user rather than changing the setup.
- Also record the flight state, current and maximum speed, target height, landing reservation,
  hover-slot holder and cell occupancy when needed. These are read-only observations, not writes
  to game state. Extend recorder support only with user approval. Record shots, crate collection,
  explored cells and score changes from video as appropriate.
- The observed area is the entire purpose-made map except the bystander's isolated start area.
  One run per subcase unless stated. No measurement is claimed yet.

**Magnetic-lift coverage.** Use S-0607's common setup and scenarios, not J1's ordinary
jumpjet setup: L2 tests retained facing, the initial movement-point search beside the
lifter and seized arrival (claims 10, 12, 16 and 23). L3 and L4 record the cell-centre
detour and accelerating fall, including L4's starting rate, drop-before-growth ordering
and zero target height (claim 18), and L5 and L6 test blocked,
clear, water, beach and deck impact outcomes (claim 19). L3 and L6 also exercise seized
descent without the ordinary landing refusals (claims 14 and 15). The special seized
simple-deployer exclusions in claims 13 and 15 remain unmeasured as stated there.

**Scenarios.**
- **J1 Field copy and controls.**
  - (a) Order the jumpjet infantry 15 cells east and record the height profile and the per-tick
    step, then shoot it down and record the fall. Expected: the climb, crash rate, height
    and speed follow the imported type fields and Import mapping's absent-key defaults.
    A profile inconsistent with the imported type fields and F1–F6 falsifies claim 1.
  - (b) A test rules file changes every game-wide jumpjet control except the cruise height.
    Expected: profiles identical to (a). Any difference falsifies claim 2. In a separate run,
    change only the game-wide cruise height: the global draw-layer threshold of S-0102
    claim 3 changes, not the jumpjet's flight profile or its type-specific layer threshold.
    A changed flight profile falsifies claim 2.
  - (c) In *jumpjet-flight-test*, omit each numeric type key in turn from every layer, then
    repeat with its stated engine default explicitly supplied. Keep all other fields equal.
    Compare take-off, straight flight, wobble, turning and crash traces. Expected: identical
    traces for omission and explicit default. A difference falsifies that default. For the
    acceleration and turn-rate keys, remove the exact key from every layer, then supply a
    different value with only its first letter changed to lower case. Repeat with the exact
    key and that same different value. Only the exact key changes the profile. A change
    from the altered-case key, or no change from a distinguishable exact-key value,
    falsifies claim 2's case rule.
- **J2 Take-off timing.**
  - Order a grounded jumpjet 10 cells away. Record the tick of the order, the first rising tick,
    the first tick of sideways movement, and the tick at which the height reaches *H* and
    stops rising.
  - Expected:
    - the order makes the unit moving; its first grounded update sets fraction 1, speeds 0,
      target height *H* and state *ascending*. Height first rises in the following update,
      by trunc(*z* + *c*) − *z* leptons
    - sideways movement requires passing claim 7's take-off fraction and reaching at least
      claim 8's half-height ground-hold threshold, each with its stated integer rounding.
      A new maximum speed affects acceleration in
      the tick after the threshold update
    - with nothing to do afterwards and no pending crash start, a grounded or hovering unit
      without a move order keeps its height, wobble phase and speed fraction unchanged. A hovering unit that still has a
      move order is not this idle case.
  - Falsifiers: rising in the grounded transition tick, horizontal movement below the ground-hold
    threshold, or any idle change in these recorded values (claims 3, 5 and 7).
  - Repeat with a player order to the unit's own cell, recording order-level destination,
    movement point, commitment and occupancy before any rising tick. Expected: grounded
    jumpjet-type infantry ignores that assignment; an ordinary grounded vehicle's mark
    excludes its own flat-land cell and its adjusted point is elsewhere (claim 23).
    An accepted own-cell assignment for that infantry, or self-exclusion from the vehicle
    search, falsifies those rules. Do not infer a chosen cell from the click alone.
  - Separately test claim 7 when the accepted point is in the ascent cell: use an entering
    non-balloon vehicle in named test data, with a successful nearby search and exact-point
    retention under claim 16, or record a move that reaches its destination cell during
    ascent. Expected: while still below target height, it enters cruise once over that cell
    and past the quarter-height threshold. Failure then to enter cruise falsifies claim 7.
    A balloon-hover vehicle retaining an own-cell point instead uses the half threshold
    and continues to *H*; early own-cell cruise for it falsifies that exception.
    Repeat the established same-cell ascent below a high bridge and from its deck.
    Record *r*, *h* and target height. Expected: the 416-lepton
    adjustment applies only at or above deck height while not on the deck; an early own-cell
    cruise retains *r*, not *h*. Subtracting the deck height below it, or retaining *h* when
    these differ, falsifies claim 7. A balloon-hover infantry variant still uses the quarter
    threshold for its speed/heading change; a half threshold falsifies that distinction.
- **J3 Occupation.** On flat land drive a ground vehicle towards a jumpjet vehicle's cell
  during ascent, descent and crash. Record height, listing, actual-position flag and landing
  reservation separately. Expected: low non-hover/cruise placements can list and mark at
  heights 1 and 50 leptons; at 300 leptons they do not. The first rise clears the old mark
  but does not permanently forbid low-height replacement. Descent can reserve before
  touchdown independently of its actual-position placement. A permanent zero-height cutoff
  or an ordinary high-flight actual-position placement falsifies claim 4. Repeat with
  infantry to distinguish its standing-position reservation from a vehicle flag. For an
  ordinary high hover/cruise over previously empty cells expect no new listing/mark.
  A newly created listing there falsifies it. Repeat from a bridge deck: on-deck state ends
  at first rise, and later eligibility follows the bridge layer rule, not zero height alone.
  Record ground/deck listings and records separately. In named test data compare type
  cruise heights 208 and 500 leptons, recording actual placing steps at flat-land heights
  100, 208 and 250, and bridge heights *r* = 100, 416, 417 and 516 leptons with on-deck
  state unset. Predict eligibility from claim 4: true, false, false on flat land and true,
  true, false, false over the bridge when not independently falling. A cruise-height-dependent
  cutoff, or listing at *r* = 516 merely because deck clearance is 100, falsifies it.
  Repeat during an ordinary crash: record bridge/parachute falling separately from the
  crash state. Crash start alone must not suppress the bridge subtraction. A subtraction
  suppressed solely by crashing falsifies claim 4. A fixture that independently sets the
  falling state is diagnostic unless a stock bridge-collapse/parachute sequence establishes it.
- **J4 Acceleration and braking.** Record the per-tick step during acceleration and the final
  approach. Expected: *v*<sub>c</sub> rises by *a* per tick to *S* and brakes by 1.5 × *a*
  (F1), including the possible acceleration-then-braking tick in F1. The step is
  trunc(*v*<sub>c</sub>) along the heading, subject to F5. Record speed fraction too: it is the
  rounded quotient in F1 before any ground-hold reset. Any different speed sequence,
  fraction or displacement falsifies claim 6 or 10 respectively.
- **J5 Look-ahead and ground hold.**
  - Fly a jumpjet across a row of buildings of known art height, then across a cell holding a
    parked vehicle. Repeat with infantry, a grounded aircraft, a terrain object and an overlay
    without a building. For an aircraft needing special placement, use *jumpjet-flight-test*.
    Over the bridge, compare a vehicle listed on the deck with one listed on the ground
    beneath it, using the occupancy recorder to establish the layer.
  - Expected:
    - the height rises ahead of the building by the F3 average or maximum
    - infantry, vehicles and aircraft on the ground layer supply the 85-lepton allowance.
      Terrain objects, overlays and deck-only vehicles do not. An extra allowance for any
      excluded object, or no allowance for an included ground-layer object, falsifies claim 9
    - a large rise of the ground reference stops sideways movement until the height recovers
      (claim 8).
  - Repeat over both ends of the high bridge and while ascending and descending at the
    destination. Compare every recorded world-height step with F3/F4, including the final
    snap and the terrain/deck floor. A wrong reference, a step using *k* rather than *c* in
    ordinary descent, a below-surface height, or continued horizontal motion during ground
    hold falsifies claims 8 or 9. Use *jumpjet-flight-test* with unequal climb and crash rates
    to distinguish the two fields. The bridge approach on level empty terrain uses a 208-lepton
    reference rise with the look-ahead bridge quirk on; 416 leptons falsifies that quirk.
  - Hold a target so a jumpjet stays hovering with its move order retained. Record 300 ticks,
    then repeat cruising. Compare the entire trace with F2/F4 using the imported wobble rate
    and amplitude and table sine. Repeat with the no-wobble flag set in *jumpjet-flight-test*.
    Expected: phase advances only in eligible states, otherwise is zero; the flag suppresses
    the oscillation. Any phase or height mismatch falsifies claim 9.
    In *jumpjet-flight-test*, compare a building whose art height is omitted with one explicitly
    given the absent-key default. Expected: identical clearance and height traces. A difference
    falsifies the building-height default.
- **J6 Step bias and facing.**
  - Order straight moves north, east, south and west of 15 cells each, at full speed.
  - Expected: per-tick displacements computed anew by F5 at trunc(*v*<sub>c</sub>) and the recorded
    heading, with body facing equal to the heading each tick.
  - Also attack with an infantry jumpjet that has the jumpjet-turn flag while it hovers.
    Expected: its heading snaps to the target at once, without copying that heading to body
    facing in that step. A delayed heading or a body-facing copy in that exception falsifies
    claim 10. For straight flight, any displacement differing from F5 evaluated with the
    recorded speed, or a body facing different from the heading, also falsifies it. Do not
    scale the illustrative table linearly: truncation applies anew at each speed.
- **J7 Bands.**
  - Order moves of 10, 30 and 60 cells, and moves that need a large clockwise and a large
    anticlockwise turn near the end.
  - Expected:
    - the speed and height plateaus of F6
    - the anticlockwise approach is slower than the clockwise one by the band rule
      (`quirks.jumpjet_turn_slowdown_one_sided`)
    - arrival within 20 leptons is followed by a landing: horizontal position snaps exactly to
      the destination and both speeds become zero in the arrival tick.
  - Repeat to a water destination. Expected: full height and a hover. For each threshold,
    compare the trace tick-by-tick against F6 using the recorded distance, heading and target.
    A different band, height, arrival radius, snapped point or nonzero arrival speed falsifies
    claims 11 or 12. Choose small clockwise and anticlockwise turns of equal magnitude that
    fall on opposite sides of a slowdown test: identical band speeds falsify the one-sided test.
  - Repeat with a retained target, a balloon-hover variant and a deploy-to-land vehicle from
    *jumpjet-flight-test*. Expected: the first keeps its target height through reduced-height
    bands, the latter two keep full height; all hover on arrival, using a neighbour step when
    the slot is held. After the ordinary variant has reached a reduced target height with no
    target, acquire a target and give a distant destination in the far band. Expected: its target
    height returns to *H* despite the target; retaining the reduced height falsifies claim 11.
    Landing or the wrong height/slot outcome falsifies claims 11 or 12.
- **J8 Hover slots.**
  - (a) Order two jumpjets to attack the same target from the same cell.
  - (b) Repeat over 20 seeds and record which neighbour the second one goes to.
  - Expected: the second takes one ranged draw over 0 to 7 at the slot conflict, after that
    tick's physics, and uses its step direction (claim 21). Use the random-state recorder to
    compare the before/after states with S-0202's ranged-draw rule. Uniformity is the range
    rule's property, not a requirement that 20 runs give equal counts. A different draw count,
    range, tick or resulting neighbouring destination falsifies claim 21.
  - Repeat the conflict at ascent completion and at water/beach descent; repeat at a map edge
    and with the drawn neighbour occupied. Expected: the same one-draw rule, followed by the
    nearby-cell search, with no redraw for an unsuitable neighbour. A missing conflict step
    or an extra rejection draw falsifies claims 7, 12, 14 or 21.
  - Remove the target. Expected: ordinary jumpjets descend and land (claim 13). Repeat with
    balloon-hover and deploy-to-land variants from *jumpjet-flight-test*: they keep hovering,
    the latter until requested to deploy (S-0517 claim 5). Give each hovering variant a new
    destination: it releases its slot and cruises. Retaining a slot after departure, ordinary
    hovering after target loss, spontaneous balloon landing, or deploy-to-land descent without
    its landing request falsifies claim 13. In *jumpjet-flight-test*, also combine balloon-hover
    with simple-deployer while omitting the deploy-to-land flag. Expected: the earlier
    simple-deployer row descends despite balloon-hover. Keeping the balloon hover falsifies
    that row's precedence.
- **J9 Landing.**
  - Order a jumpjet to land on a beach cell, and on a cell where a vehicle stands.
  - Expected:
    - beach: it hovers, and never lands (claim 14). Touchdown on the beach falsifies it
    - occupied cell: it is re-routed to a nearby free cell and lands there (claims 15 and 17).
  - Introduce the blocker only after the destination was chosen, so the move-order search
    does not avoid it in advance. Repeat with the blocker arriving on each of two successive
    descending ticks before touchdown. Expected: failure reroutes on that tick in both runs,
    not on alternating ticks. A one-tick delay in either run falsifies claim 15's cadence.
  - Repeat with a free infantry standing position, an occupied position, a moving blocker
    before/after reservation, a cloaked blocker and a high-bridge cell. Record the reservation
    and entry outcome. Expected: exactly the pass/fail combinations of claim 15; any contrary
    reservation or reroute falsifies it. Check each tick that continued descent uses *c*.
  - Place an eligible crate in a landing cell. Expected: pickup occurs at touchdown under
    S-0520 claim 6. Repeat with a successful unit power-up: its pickup rejection does not
    reverse the completed landing. Re-ascent or restoration of the old move order solely
    because pickup was rejected falsifies this result. Record the same
    tick's speed fraction, position, on-deck state, occupancy, sight footprint, destination,
    moving flag and flight state. Repeat from above the high bridge. Expected: touchdown at
    bridge-adjusted height 0, even before the on-deck state is set, rather than descent to the
    terrain beneath the deck. Expected: claim 15's touchdown values together, including
    fraction 0 and state *grounded* for the surviving unit. A retained order, an incorrect
    point/deck/occupancy, delayed sight or crate pickup falsifies that part of claim 15.
  - With *jumpjet-flight-test*, use a simple-deployer jumpjet vehicle and explicitly set the
    game-wide deploy direction to 1, then to 9 in a separate run. Expected: it starts turning
    towards north-east (8192 facing units) in both runs while descending, and clears the
    landing-for-deploy request at touchdown. On a fresh program start, omit the deploy-direction
    key from every layer and compare with explicitly setting 0: both aim north (0 facing units).
    A different heading target, different omission/control result or uncleared request falsifies
    claim 15 or its import mapping. Run water/beach approach with
    current-only and queued-only *enter* in boarding setups: neither invokes claim 14's
    forced hover or claim 15's obstruction check. A forced hover or obstruction reroute in
    either entering case falsifies those exceptions.
- **J10 Orders in flight.** Give a new move order while the jumpjet is descending. Expected: it
  climbs back towards *H* (claim 16). Give an order to a cell inside a building. Expected: the
  destination snaps to the centre of a nearby cell. Repeat with infantry: it uses a free
  standing position. Repeat with a balloon-hover vehicle and an entering vehicle: they keep
  the exact ordered point. Record the old reservation, destination, moving flag, target height
  and deploy request; expected: claim 16's values, including reservation release and re-ascent
  cancelling landing-for-deploy and replacing *unload* with *guard*. Any retained reservation,
  wrong snapping/exception, continued descent or retained landing request falsifies claim 16.
  The no-point order and failed-search cases are not established by ordinary clicks here.
- **J11 No landing cell.** Order a jumpjet infantry to land in the single-cell pocket after
  filling the pocket's only standing spots with infantry. Fill the surroundings so that no nearby
  cell is free. Expected: it destroys itself with no kill credited (claim 17). A hover or a
  landing falsifies it. First run a less crowded control with a nearby legal cell: expected
  reroute and landing, not death; death or failure to reroute falsifies claim 17's success case.
  Record health and kill credit in the fully blocked run; nonzero remaining health after the
  failure or a credited attacker falsifies the corresponding self-destruction result.
- **J12 Crash.**
  - Kill a cruising jumpjet vehicle of a crashing type. Record its height and body facing each
    tick to impact.
  - Expected:
    - the height follows the ordinary F4 step plus claim 18's extra crash drop; well above the
      surface, the two successive resulting heights are each truncated toward zero
    - the facing turns at the S-0206 claim 14 rate in one sense
    - the impact follows S-0312 claim 16.
  - Repeat with unequal, non-integer climb and crash rates in *jumpjet-flight-test* so a
    single fall step or truncation of the rates rather than of world heights is distinguishable.
  - Repeat the kill during ascent. In the first crash update, expect ordinary height control
    for the former ascending state, followed by the extra drop; only the following update
    uses the −5-lepton target. A first-update ordinary descent using that new target falsifies
    claim 18's entry timing.
  - Repeat over a high bridge. Expected: the extra drop that crosses below its deck ends the
    fall at the resulting height, without a snap back to the deck. Any trace inconsistent with
    the two height steps, a spin in the opposite sense or at a different rate, a hover
    slot still held after crash completion, continued fall after the crossing or a snap back
    to exact deck height falsifies claim 18. Repeat with a crashing infantry variant from *jumpjet-flight-test*: expected
    air-death-start presentation at crash entry; a different sequence falsifies that part.
- **J13 Sinking only after a magnetic-lift drop.** Shoot down a jumpjet vehicle over water.
  Expected: it is not set sinking by its movement (claim 19). A magnetic-lift drop over water is
  tested by S-0607 scenarios L5 and L6. Record sinking state through impact as a diagnostic observation.
  A sinking state before impact would falsify the fall's absence of a sinking trigger.
  An impact-tick sinking state alone cannot distinguish movement from impact/death effects,
  so this run does not back claim 19's attribution.
- **J14 Firing states.**
  - Order a balloon-hover type, if stock has one, and an infantry jumpjet with the jumpjet-turn
    flag to attack while ascending, cruising, hovering and descending.
  - Expected: shots only while grounded or hovering (claim 20). Retain a target in range,
    ammunition and a completed reload; use *jumpjet-flight-test* to give the test infantry a
    weapon legal against the target in every tested height state. Add a grounded subcase.
    Give the balloon-hover vehicle the mobile-fire flag but remove the weapon's
    fire-while-moving flag, so S-0310 claim 7's other destination tests do not mask this one.
    A shot in an excluded living phase falsifies claim 20. Failure to fire
    in an otherwise legal grounded/hovering case also falsifies it.

- **J15 Idle crash diagnostic.** Common setup; use a landing-first jumpjet vehicle, or a
  named test-data equivalent if no stock type establishes the case. Record a hover with its
  moving flag false before shooting it down. Record crash start, order clear/reroute, health,
  removal and the next 60 ticks of position/state. The source hypothesis is no movement
  update from crash-start alone; an update without a new movement order or state change
  falsifies that hypothesis. An order-induced wake does not. This diagnoses the original
  uncertainty, not the design rule. Engine test: a pending aerial crash must get its next
  update without any new order; continued idle suspension falsifies the design rule.
- **J16 Search settings and position draws.** Common setup; record the destination at order
  application, cell occupants, selected standing position, tick and scenario stream before
  and after. Compare the chosen cell with S-0212 claim 25 at that tick and claim 23's row.
  Use infantry and ordinary/balloon-hover vehicles separately. Include high-bridge centres,
  an overlay-bearing candidate, candidates over one height level from the centre, and an
  isolated candidate in a different reach region. Ordinary vehicle claim-16 searches exclude
  bridges; the infantry/balloon searches allow them and have deck mode at bridge centres.
  All rows lack a height condition. Repeat during failed landing and animated landing-first
  undeploy; record their first selection separately from the resulting move-order search.
  The latter's initial search uses its centre's own-zone region and its current on-deck state.
  Any result inconsistent with those full search settings falsifies claim 23. On a flat empty
  cell, eligible infantry adjustment takes one ranged draw over 0 to 3 and chooses a side,
  never centre. Block the preferred side and compare each remaining preference with table O.
  Record at least 20 exact seeds; a missing/extra draw, centre choice or wrong preference
  falsifies the choice rule. A vehicle/object refusal takes no position draw. A full set of
  taken side positions still takes its eligible position draw and fails adjustment. Test the
  world-height-0 deck boundary only if a legal negative-elevation map fixture can establish
  it; this boundary remains diagnostic rather than claimed stock measurement coverage.
- **J17 Failed-adjustment diagnostic.** Common setup with a map-edge area and a recorder
  able to confirm actual adjustment failure, rather than inferring it from a rejected click.
  If stock orders cannot establish failure, record that limitation instead of injecting
  state. Compare an already moving unit, an idle grounded one and an idle hovering one.
  Source hypothesis for a legal on-map retained point: unchanged moving flag/state and the
  ordinary motion rules thereafter, not successful-order re-ascent. A new moving flag for
  either idle unit or a different retained point falsifies it. Record exact non-centre point,
  arrival and landing attempts. Off-map and direct-empty outcomes are diagnostics with no
  claimed safe original result. In the engine's safety test, refusal retains the preceding
  point and reservation; movement towards an invalid point or edge clamping falsifies the
  design rule of claim 24. No unsafe original test is required to validate that design.
- **J18 Self-clearance and low-flight listing.** Common setup; establish ordinary high hover
  over open flat terrain, no other occupants. Record actual cell lists and height/reference.
  Expected: no self listing or automatic 85-lepton reference increase. Add a second listed
  ground vehicle: expect its 85-lepton allowance when look-ahead is eligible. Either contrary
  result falsifies claim 22. Diagnostic variant: order own-cell early cruise, retaining a
  target so that low cruise can lead to hover; record lists at every transition and crossing,
  independently of reservations. If a low ground listing survives, look-ahead must include
  it even for its owner. Self-exclusion falsifies that predicate; clean removal at the low
  transition falsifies the retained-listing history. A direct-reference interval is not a
  test of the allowance. Establish the accepted same-cell point as in J2, rather than
  assuming an own-cell click bypasses claim 23. Repeat with a deck-only occupant: no
  ground-layer allowance.
- **J19 Stop in each state.** Common setup, stock jumpjet infantry and any eligible vehicle.
  Issue stop once each during ascent, cruise, descent and hover with a retained order, and
  separately during grounded/hovering idle. Record order-level destination separately from
  the movement point, moving flag, reservation, target and state. Expected: claim 24's table,
  using both searches and any infantry point draw, with cleared order-level target/destination.
  Idle stopping leaves movement alone; moving descent re-ascends after successful reroute.
  An immediate no-point moving-flag clear, retained old cruise point despite a different
  successful reroute, or continued descent after successful replacement falsifies the table.
- **J27 Magnetic-release boundaries (claims 5, 9, 17 and 24).** Conditional original-game
  measurements (a and b), and a separate engine design test (c).
  Use the common flat skirmish map, game-speed index 2 and 800 × 600 pixels, with a
  stock magnetic lifter and an eligible enemy ground vehicle in the test area; keep the
  bystander isolated. No engine add-on or mod is active. Use scripted ordinary
  input and a proposed read-only recorder for grip, seized/set-down state, order-level
  destination, movement point, movement commitment, flight state, speeds, target height,
  world height, crash rate, cell listings and restoration. Propose missing recorder support
  to the user; never inject state. For each part attempt 20 runs, record each seed and exact
  input/release ticks, and retain every qualifying trace. No qualifying trace means
  inconclusive reachability, not confirmation of the rule or a falsifier.
  - (a) Grounded release: use a legal new lifter destination to end the grip, varying its
    input tick around the seize. Keep surrounding cells free so the stop searches can
    succeed. Qualify only a trace in which release precedes the seized unit's first
    jumpjet update: height is 0, state is *grounded* and commitment is present.
    Record both stop searches, their resulting point, the empty order-level destination,
    commitment, next eligible update and eventual restoration. Expected: a successful
    reroute retains commitment and point, and the next eligible update enters *ascending*
    under claim 5; ordinary height rise starts in the following update. Restoration cannot
    occur while commitment remains. Staying grounded through that eligible update, losing
    commitment despite successful reroute, or restoration while committed falsifies
    claims 5 or 24. Stock input timing establishing this boundary is not yet settled.
    The proposed test makes the conditional rule `measurable`, not `measured`.
  - (b) Retained-point crash: first use arrival release (S-0607 claim 8), which centres the
    unit. Place a parked ground vehicle or building in that cell after the seize's initial
    point search but before arrival. Vary the placement timing so release can retain a
    successful stop-reroute point in another cell. Qualify only a trace with that retained
    point, zero crash speed and a nonzero ground-layer clearance allowance in the current
    cell. Record listings independently of occupancy marks and record every height step
    through crash entry and impact. Compare the observed height after each update with
    F3/F4 followed by claim 18's extra drop, using the recorded preceding state, target
    height and crash rate. At zero speed only own-cell clearance contributes: changing
    a neighbouring occupant must not change the reference. A direct-terrain-only step
    when its predicted result differs from the own-cell-clearance result, a reference
    supplied by the neighbour at zero speed, or any other height mismatch falsifies
    claim 9's combination. Stock reachability of all these conditions together is
    unsettled; the conditional measurement does not assert it.
  - (c) No-cell release is an engine design test, not an original-game oracle run: arrange
    failed stop searches for lifter-ended release at positive and non-positive height.
    Expected: both units lose their point and commitment. The dropped unit falls with
    an update every tick until its fall ends, including release while *hovering*. At its
    cell centre with no point it uses the direct ground reference; off-centre its detour
    supplies its own cell centre. The set-down unit stops in place. Neither self-destructs,
    with either setting of the no-cell quirk. Missing fall updates or look-ahead while
    falling with no point falsifies the design rule.
    Test the no-destination exception separately: claim 17's ordinary quirk outcome still
    applies. With the switch off, expect no point or commitment: height above the surface
    of 1 lepton or more gives *hovering* without descent or onward flight; height 0 leptons
    or less gives *grounded* without moving. Include *ascending* at height 0, and heights
    −1 and 1 leptons as boundary controls. A seized hovering set-down unit is not restored
    and retries the search at the end of each of its turns under S-0607 claims 9 and 20,
    regardless of how it reached this outcome. Expected retry ticks: the search runs at the
    end of the unit's turn in every tick from the tick of the no-cell outcome on in which it
    takes a turn. A cell freed in a tick is found at the end of the unit's turn in that tick
    when the freeing comes before that turn in the update order, and otherwise at its next
    retry. Test both failed and successful retries,
    including a set-down unit that retained commitment, flew to its point and reached the
    no-cell outcome through a failed landing or another stop. Descent or onward flight
    before a successful retry, hovering at non-positive height, restoration while still
    hovering, a route-dependent retry, a turn that ends without a retry, a retry at a
    fall-back to rest outside that test, self-destruction on lifter
    release, or applying that safety exemption to the switch-on no-destination exception
    falsifies the design rule.
- **J20 Zero-field engine test.** Named importer test data, not an original-game oracle run:
  one type with zero speed, one with zero turn rate, and one with both. Expect a warning for
  each affected type, zero fraction and all zero cruise speeds for zero speed, no middle
  band for zero turn rate, and otherwise ordinary vertical motion. A division fault,
  nonzero zero-speed band/fraction, or missing warning falsifies claim 25's design rule.
- **J21 Flight recomputation.** Common setup; cross tile-animation-eligible cells during
  cruise and during low/high ascent, descent and crash. Record per-tick position, state,
  listing/placement events and animation creation/stream changes. Hold position in a
  non-hover/cruise state as a no-crossing control. Expected: claim 26's events, including
  repeated old-cell recomputes even without crossing, but no cruise/hover crossing event.
  Use S-0010 claim 19 to predict which recomputes can actually create an animation; lack
  of animation when that eligibility fails does not falsify recomputation. A cruise-crossing
  creation attributable only to land recompute, or missing eligible non-hover placement
  recomputes, falsifies claim 26. Recorder support must be proposed to the user, not added
  to the harness here.

- **J22 Destination-clear sources and deploy.** Common setup; use a landing-first jumpjet
  vehicle, with named test data only if no stock type supplies the flags. Establish hovering
  with a movement commitment and record the order-level destination and separate movement
  point. Issue deploy with a nonempty destination, then in another run after stop has left
  that destination empty but a commitment retained. Record every destination assignment,
  refresh state, movement point, flight state, landing request and touchdown position.
  Expected: the effective first clear runs both searches and keeps their resulting point.
  The ordinary empty-to-empty vehicle assignment is suppressed and keeps its preceding
  point without another reroute. A point erased by either case, or an extra reroute on the
  suppressed assignment, falsifies claim 24. Descent at an equal point and cruise before
  descent at a different point must follow claim 13 and end at the retained point.
  Repeat a behaviour-processing destination clear after a nonempty move, identifying the
  effective assignment in the recorder: the same reroute applies irrespective of its source.
  Continued old-point travel despite a distinct successful replacement falsifies universality.
  Repeat an unchanged nonempty vehicle destination as a suppression control. J24 and J25
  separately test refresh and balloon-hover retention; neither is an unspecified destination
  substitution. Use J19 for
  stop and J11 for no-cell failure; those outcomes are not replaced by deploy-specific rules.
  Diagnostic only: if ordinary inputs establish idle hover without a commitment, request
  deploy and record whether a separate event creates a commitment. Without such an event,
  landing-request-only motion would falsify the source hypothesis; no stock fixture for
  this starting state is asserted. Propose missing recorder support rather than writing it.

- **J23 Infantry duplicate assignments (Q-1030).** Common setup, stock jumpjet infantry,
  20 exact seeds, random-state recorder and scripted input. While it retains a commitment
  and remains outside the destination cell, issue two identical move assignments to a
  free cell. Repeat with the same object as destination using a legal guard or enter
  assignment, recording actual assignments separately from any order-family rejection.
  Expected: both eligible assignments repeat point adjustment and its one ranged draw
  over 0 to 3. A missing second eligible draw falsifies claim 24. As a control, a
  nonempty assignment into the unit's current cell is rejected before searching.
  Then issue stop twice while a commitment survives the first reroute. Record destination,
  point, reservation and stream around each application, without intervening touchdown.
  Expected: both empty assignments reroute; the second is not suppressed. Repeat with
  a behaviour-generated empty assignment after the first stop, recording that the
  assignment actually occurred. Suppression solely because the destination was empty,
  or a caller-dependent difference for the same starting state, falsifies claim 24.
  Compare the complete stream against both searches and claim 23: a search/pre-draw
  refusal is not evidence of unchanged-destination suppression. Propose missing
  assignment/stream recorder support to the user; do not modify the harness.
  Diagnostic only: if a stock infantry type with the balloon-hover flag is established,
  stop while commitment survives both empty assignments. Expect two effective reroutes,
  not vehicle equality suppression of the second. A suppressed second assignment
  falsifies claim 24; absence of such a stock type does not establish this subcase.
- **J24 Refresh lifetime (Q-1031).** Common setup, an ordinary jumpjet vehicle, read-only
  assignment recorder. If no shipped type supplies this combination, use an explicitly
  identified *jumpjet-flight-test* vehicle variant and record it as named test data.
  Record initial placement and idle-behaviour setup, then stop,
  repeated stop, move, repeated move and deploy where the stock type permits it.
  Expected: a fresh ordinary jumpjet without a destination receives an effective
  empty-to-empty clear during initial idle fallback. It consumes refresh, enables no
  point suppression and gains no flight without commitment. Refresh is absent after
  setup and not newly enabled by the ordinary orders. An omitted initial clear,
  persistence after setup or ordinary-order activation falsifies claim 27. Record
  actual assignments separately from orders and type-specific fallback exceptions.
  Diagnostic only, following S-0207 claim 27: record a teleport-capable vehicle's
  effective destination assignments, contact already held, offered destination kind,
  ground-layer vehicle presence, warp-pending immobilization separately from phasing
  in and held, current locomotor, restoration permission, both pending states and each
  nonempty point application within the assignment. Distinguish unchanged rejected
  offers from effective ones. Only assignments satisfying every gate qualify:
  warp-pending immobilization disqualifies the assignment independently of current
  locomotor. Restoration enables neither pending state and routes the same assignment's
  otherwise due point to the restored predecessor, subject to already-pending suppression
  and ordinary destination refusal. A point applied to the temporary locomotor first,
  or eligibility decided using the newly offered point, falsifies this contract.
  Fallback normally skips that assignment's point and leaves refresh, not suppression,
  pending. A later refreshed equal assignment can
  qualify again. Changed contact or occupancy alone must not retest a suppressed equal
  offer. Retesting on contact alone, delayed point suppression, loss of newly enabled
  refresh without another effective assignment, or a once-per-entry limit falsifies
  the contract. Record an empty-clear control separately: it does not consume pending
  nonempty-point suppression. No stock jumpjet fixture for later activation is asserted.
  A run missing those conditions is inconclusive, not evidence that the states can be
  omitted. Named data combining the properties is not a stock-play reachability
  measurement. Extend recorder support only with user approval.
- **J25 Balloon-hover clear retention (Q-1031).** Common setup, stock balloon-hover
  vehicle with a legal target, 20 exact seeds. Move towards a distant cell, then attack
  a legal target while the recorded turret facing is within claim 28's angular bound
  and the order-level destination is still nonempty. Attack installs its target before
  requesting the clear; expect retention at that ordinary order boundary. Through
  subsequent pursuit, record every attempted empty assignment, target, destination, point, commitment,
  ground-layer listing, turret facing, current/queued behaviour and stream. Classify
  each assignment by claim 28, not by the clicked order. Expected: qualifying clears
  retain all movement values and perform no reroute; nonqualifying effective clears
  use claim 24. Where the prior behaviour was not attack, the retention outcome makes
  attack current and empties the queued behaviour. A different retained value, a
  reroute on a qualifying clear, or the wrong behaviour result falsifies claim 28.
  Include empty-destination and absent-target controls and nonempty assignments.
  Probe pursuit from both sides of the angular boundary, with differences of 16383,
  16384 and 16385 facing units if ordinary traces establish them, using destinations
  that satisfy neither identity row. Expected: retain at the first two, not the last.
  A contrary boundary falsifies the angle test. Identify the first ground-listed
  target separately from other occupants; mere sharing a cell is not that identity
  test. If ordinary input supplies no qualifying assignment or exact boundary, report
  that subcase as unmeasured rather than injecting state or inferring substitution.
  Also apply stop with an initially qualifying destination/target pair. Expected:
  retention at the first empty assignment, then target clearing and an effective second
  clear/reroute. The final destination is empty; the interim attack behaviour can survive.
  An extra reroute on the vehicle's second assignment when the first already cleared,
  or a final retained nonempty destination, falsifies claim 28. Apply deploy where the
  stock type accepts it: expect only one clear, before target clearing and queuing unload,
  so a qualifying rejection retains the nonempty destination and point. If no stock
  deployable balloon-hover type establishes it, that deploy subcase remains diagnostic.

- **J26 Skipped factory-report window, engine test.** Named engine test data, not an
  original-game measurement: release a qualifying jumpjet in factory contact and make
  its first rising physics step reach or pass the target height, without an earlier
  qualifying report. Record flight state, post-physics height, target height, both links,
  saved guard spot, order-level destination and movement commitment. Expect claim 7's
  design fallback in that reaching update, with S-0417 claim 15's destination effects.
  Either link surviving the update, delayed reporting until arrival or touchdown, or a
  destination effect inconsistent with that claim falsifies the design rule. Repeat
  with and without a saved rally. Original skipped-window aftermath remains diagnostic
  under U14; no safe original fallback or stock fixture is asserted.

- **J28 Flying-object registration and two entries (claims 29 and 31, Q-1500).** Common
  setup. Target: a stock jumpjet vehicle (named variant of *jumpjet-flight-test* if stock data has
  none), owned by a second test house that is the firer's enemy, at full health. Firer: one stock
  unit whose weapon can hit ground and air targets and whose projectile detonates at the aimed
  object's position. Decoy: an enemy jumpjet hovering in the target's cell or next to it, when a
  variant says so. Use a named test variant of the firer's warhead whose spread radius *R*
  exceeds the decoy's height above the target, so that a detonation at the decoy is above the
  terrain and within *R* of the target. No veterancy and no other damage sources. Use the
  read-only recorder for the target's health and separate damage events, flight state, height
  above the surface *r*, and each detonation's tick and position. *Observable:* per detonation,
  the number of damage events the target takes and their sizes. Two hits need not total twice a
  single hit, since each entry is sized at its own distance (S-0303 claim 7).
  - (a) The target hovers at its cruise height (at least 208 leptons), aimed at directly.
    Expected: one hit per detonation; it is found only through registration. No damage
    falsifies registration while hovering.
  - (b) The target descends to land, aimed at directly, with detonations while *r* < 208
    leptons. Expected: two hits per detonation, unless the first kills it. A single hit
    falsifies claim 31 or S-0303's handling of the two entries.
  - (c) The target is ordered to move, and a detonation at the decoy arrives in its take-off
    update, at *r* = 0. Expected: two hits. A single hit (cell entry only) falsifies the start
    of claim 29, for example registration at the first rise.
  - (d) The target is grounded and idle after a completed touchdown, with a detonation at the
    decoy. Expected: one hit (cell only; touchdown ended registration). Two hits falsify the
    touchdown end.
  - (e) A stock magnetic lifter seizes an enemy ground vehicle and lifts it above 208 leptons.
    A second firer of the vehicle's enemy fires at it while it is gripped. Expected: each
    above-terrain detonation damages it. No damage falsifies the seized unit's registration.
  Attempt each variant 10 times and record each seed and input tick.
- **J29 In the air for weapon choice (claim 30, Q-1501).** Common setup. Firer: one stock unit
  whose primary weapon cannot hit air targets and whose secondary weapon has an anti-air
  projectile (S-0308 table P row 19). Targets, each in a separate run: a stock jumpjet vehicle
  (or named variant), a stock jumpjet infantry unit, and an enemy ground vehicle seized by a
  stock magnetic lifter. Use the read-only recorder for each target's height above the surface
  *r*, on-deck state and flight state, and for the firer's chosen weapon slot at each shot.
  *Procedure:* order the firer to attack the target while the target is grounded, ascends,
  hovers and descends. For the jumpjets also order a landing on the high bridge's deck and
  record the shots of its final descent, where the on-deck state is not yet set and *r* is
  between 416 and 624 leptons (*r*′ below 208). *Observable:* the chosen slot against *r* at each shot.
  *Expected:* the secondary exactly at shots with *r* ≥ 208 leptons, the primary (or no shot,
  when the primary is refused) at shots with *r* < 208, in every flight state; during the deck descent
  the secondary, because *r*, not *r*′, is tested. A switch at another height, or one that follows the flight
  state (for example the primary while ascending above 208 leptons), falsifies claim 30. Repeat each
  target and flight-state/height subcase in 10 runs, recording the seed and input tick.

- **J30 Disc-laser attack height (claim 32, Q-1504 question 2).** Common setup,
  game speed 6, a tiny flat test map without bridges or objects beneath the observed
  hover. Use a stock balloon-hover jumpjet vehicle with a disc-laser weapon, owned by
  the human test house, and one stationary enemy target within weapon reach. Keep
  the bystander computer player's unrelated objects outside the observed area.
  First order the jumpjet to its hover cell and allow its ascent to finish. Record
  its world and terrain heights, flight state, target height, evaluated render layer,
  attack target, shot ticks and damage ticks through a read-only recorder, with video
  of the beams. Then order an attack without a move, and observe three completed
  shots while the target survives. Repeat 10 times, logging seeds and input ticks.
  *Expected:* the jumpjet keeps hovering, its target height does not change at the
  attack order, shot, charge-up or damage events, and its actual height follows the
  ordinary wobble and height-control formulas. In each interval whose recorded height
  stays at least 208 leptons, its evaluated layer remains air or top, never ground.
  A shot-correlated height change beyond those formulas, a changed target height or
  a ground evaluation at such a height falsifies claim 32. Target death, movement,
  crash or interference makes that interval inconclusive. For a joint target-acquisition
  test use S-0309 scenario A11 part k; do not infer a layer change from retaliation.

## Open questions

- **Registration of a sinking dropped unit.** Claim 29 fixes the registration boundaries and
  its two readers. Whether a seized unit dropped onto blocked water, which starts sinking
  (claim 19), stops being registered before it goes off the map is not established
  (confidence low). It is registered at the latest until then.
- **The crash-end event.** The event raised at the end of a crash (claim 18) remains
  untraced here. The separate ascent factory-contact interaction is resolved by claim 7
  and S-0417 claim 15. Its original skipped-window aftermath and unsolicited no-rally
  infantry take-off retain that claim's limitations; claim 7 supplies the decided
  skipped-window safety fallback, not an observed original release event.
- **Original exceptional outcomes.** J15 and J17 retain the unmeasured original idle-crash
  aftermath and off-map failed-adjustment diagnostic. Claims 3 and 24 supply deterministic
  safety departures, not a promise that the original recovers safely.
- **The crashed state.** What a *crashed* jumpjet does until S-0312's impact removes it, and what
  an ordinary dead jumpjet does on water after impact (S-0312's open question), are not settled
  here.


