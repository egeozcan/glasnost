---
id: S-0502
title: Movement of jumpjets
topic: movement
revision: 4
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-16
measurements: []
requests: []
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
- which flight states count as "moving now" for the firing checks.

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
- **Factory exit.** The factory roof a new jumpjet lifts off from: S-0417 claim 10.
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
  jumpjet counts as in the air for combat is given by S-0307's height test and S-0308's
  in-the-air state, not by being *hovering* or *cruising*.

## Terms

From the glossary and other specs:
- *jumpjet type*, *jumpjet*, *jumpjet cruise height*, *movement heading*, *body facing*
- *balloon-hover type* (S-0310 Terms) and *jumpjet-turn flag* (S-0310 claim 8)
- *locomotor*, *speed fraction*, *destination*, *target*
- *height above the surface*, *height above the terrain*, *world height*, *high bridge*,
  *bridge deck*, *land type*
- *occupancy record*, *entry outcome*, standing position (S-0209)
- *step direction*, *n-step facing*
- *scenario random stream*, *ranged draw*
- *demolition warhead*; *damage that ignores defences* and the *no-escape* marker (S-0312 Terms)
- *behaviour*, current and queued (S-0215); *sight*; *sinking*; *sinking descent*
- *legal cell*.
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
  - A jumpjet is *moving* while it holds a move order: from the order (claim 16) until it
    touches down (claim 15) or the order is cleared.
  - It is *moving now* in every jumpjet state except *grounded* and *hovering*.
  - Glossary candidate: this is the jumpjet sense of "the locomotor is moving it" in S-0310
    claims 7 and 8.
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
   - Some keys in the stock rules data are spelled with a different letter case from the keys the
     original reads, so they are never read (Import mapping; S-0001).
   - Backing: `measurable` (scenario J1 b) for effects and case sensitivity. Reading values that
     never affect play is `unmeasured`: no observable distinguishes reading and ignoring from
     never reading them.
   - Confidence: high

### Per-tick update

3. **What runs in a tick.**
   - In each tick in which a jumpjet is *moving* or *moving now* (Terms), the state outcomes use
     that tick's physics results. A state change, or a new maximum speed or target height, first
     affects the physics results in the following tick (claims 6 to 10, F1 to F5).
   - In a tick in which the jumpjet is neither moving nor moving now (grounded or hovering
     without a move order), neither runs:
     - its height does not change
     - its wobble phase does not advance
     - its speed fraction keeps its last value
     - a crash start is not taken up (claim 18).
   - Backing: `measurable` (scenario J2) for tick timing and idle values. The deferred idle
     crash-start case is `unmeasured`: no scenario establishes a crash request on an idle
     hovering unit without some other update waking it.
   - Confidence: high

4. **Ground occupation.**
   - A *hovering* or *cruising* jumpjet takes no place in the occupancy record (S-0209).
   - In every other jumpjet state, its occupancy mark is lifted and laid again at its current
     position in each physics step.
   - When a jumpjet leaves the ground (its height above the surface rises from 0), its occupancy
     mark is lifted and it is no longer on a deck.
   - Backing: `measurable` (scenario J3) for occupancy outcomes. Replacing an unchanged
     occupancy mark each tick is `unmeasured`: it has no separate observable effect.
   - Confidence: medium (the per-state marking was read; its interaction with other movers is
     S-0209's)

### Take-off and climb

5. **Take-off.** In its first update with a move order, a *grounded* jumpjet (claim 16):
   - its speed fraction becomes 1
   - its current and maximum speeds become 0
   - its target height becomes *H*
   - it enters *ascending*.
   - Backing: `measurable` (scenario J2)
   - Confidence: high

6. **Speed.** Each physics step moves the current speed towards the maximum speed (F1):
   - upwards by *a* per tick, not beyond *S*
   - downwards by 1.5 × *a* per tick, not below 0.

   The jumpjet's speed fraction is then set to *v*<sub>c</sub> / *S*. S-0310 claim 8 uses that
   fraction for an infantry jumpjet.
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

       Factory contact at this threshold has no additional established observable effect here
       (Open questions; S-0417 claim 10 owns factory contact).
     - If the jumpjet is already over its destination cell and is not of a balloon-hover type,
       its target height becomes its current *r* and it enters *cruising*. Away from a bridge,
       a jumpjet ordered to its own cell therefore stops its take-off climb as soon as it passes
       that fraction. Above a bridge while not on its deck, the retained target height includes
       the 416 leptons that the threshold height *h* excludes
       (`quirks.jumpjet_bridge_early_cruise_height`; Edge cases).
   - **Once *h* reaches the target height:**
     - if another jumpjet holds the hover slot of its cell, it takes a neighbour step
       (claim 21) and enters *cruising*
     - otherwise it claims the slot and enters *hovering*.
   - Backing: `measurable` (scenario J2)
   - Confidence: high (thresholds and transitions)

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
     always for a balloon-hover type. The look-ahead combines two clearances:
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
     the deck's height less *k* (F3).
   - Backing: `measurable` (scenario J5) for wobble, the object-kind and cell-layer
     clearances, and the bridge reference.
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
    - **Release.** It releases any landing reservation it holds, unless it is grounded.
    - **No point.** If the order gives no point, the jumpjet stops being moving and nothing else
      happens.
    - **Destination.** Otherwise a nearby-cell search (S-0212 claim 25) runs from the point's
      cell, for the hover locomotion class and the flying movement zone.
      - The destination becomes the centre of the cell found, at its terrain height (the deck
        height on a high-bridge cell). An infantry jumpjet instead takes a free standing position in
        that cell (S-0209).
      - Two kinds keep the exact point that was ordered: a balloon-hover vehicle, and a vehicle
        whose current or queued behaviour is *enter*.
      - If the search finds no cell, the ordered point itself is kept as the destination and
        the jumpjet's moving state is left as it was. One that was not moving does not start.
        How a later update uses the retained point remains unresolved (Open questions).
    - Otherwise it becomes moving.
    - **Descending.** A *descending* jumpjet goes back to *ascending* with target height *H*.
      Any landing request it had for deploying ends, and an *unload* behaviour in progress is
      replaced by *guard* (S-0215; S-0517 owns deploying).
    - A grounded jumpjet takes off (claim 5). A hovering one leaves its hover in the next tick
      (claim 13).
    - Backing: `measurable` (scenario J10) for successful orders and re-ascent. No-point and
      failed-search cases are `unmeasured`: J10's ordinary clicks do not establish a cleared
      destination or failure of the flying-zone search, independently of other order handling.
    - Confidence: high (the snapping, the exceptions and the re-ascent); medium (the search's
      class and zone)

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

    Stopping a jumpjet that is not moving does nothing.
    - Backing: `measurable` (scenario J11) for rerouting, self-destruction and no kill credit.
      Exact damage amount, warhead, defence bypass and no-escape marker are `unmeasured`:
      J11's unprotected empty infantry unit dies under several damage mechanisms, so the
      result cannot distinguish those properties. Stopping a non-moving unit is likewise
      `unmeasured`: no scenario here isolates that stop from ordinary order handling.
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
      updates (Open questions). An infantry jumpjet switches to its *air-death-start* sequence
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
    - **Magnetic lift.** A seized unit does not spin. Its speeds are 0 and its
      crash rate grows by 1 lepton per tick, each tick, with each new rate narrowed to single
      precision under S-0205 claim 5 (S-0607 claim 15).
    - Backing: `measurable` (scenario J12 for ordinary crash motion; S-0607 scenarios L3
      and L4 for the cell-centre detour, zero-speed fall, no spin and growing crash rate).
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
      enters *cruising*.
    - **No check before the draw.** Nothing tests the chosen neighbour first. An off-map or
      occupied neighbour is handled only by claim 16's nearby-cell search.
    - Backing: `measurable` (scenario J8 b)
    - Confidence: high (the draw and its range); medium (the handling of an off-map neighbour)

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

Thus the braking comparison uses the speed after acceleration, and braking also applies in that step if acceleration overshoots the maximum speed. The acceleration result is capped at *S* and the braking result at 0, not at the maximum speed. If the speeds are equal, the current speed is unchanged. The speed fraction is *v*<sub>c</sub>′ / *S*, in double precision. *S* = 0 divides by zero (Edge cases).

**F2. Wobble and desired height (claim 9).** Here 2π denotes the stored double
6.283185307179586 radians, not an exact real constant. Evaluate the inner division, outer
division and phase addition separately under the floating-point rule above.
- In *hovering* and *cruising*, without the no-wobble flag: φ′ = φ + 2π / (15 / *W*).
- Otherwise φ′ = 0.

*h*<sub>d</sub> = trunc(sin φ′ × *d* + *h*<sub>t</sub>), with sine and multiplication in double
precision.

**F3. Ground reference (claim 9).**
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

- The speed divisions are of integers. The floor of 1.0 is applied to the result. *T* = 0 divides
  by zero (Edge cases).
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
| crash target height | −5 | leptons | claim 18 |
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
| grounded | move order | ascending | speeds 0, fraction 1, *h*<sub>t</sub> = *H* (claim 5) |
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
| descending | bridge-adjusted height *h* = 0 | grounded | touchdown effects (claim 15) |
| any but crashing, crashed | crash started, above the surface, not held by magnetic lift | crashing | *h*<sub>t</sub> = −5 leptons; extra drop this tick, new ordinary height target next tick |
| crashing, ordinary unit | height < 1 lepton or extra drop crosses a deck | crashed | slot released; S-0312 claim 16 impact |
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
  fraction (F1), and one with a turn rate of 0 divides by zero in its middle band (F6). Stock
  data has neither. In the original, *S* = 0 gives an infinite or undefined speed fraction,
  and *T* = 0 is an integer division by zero that stops the program. Backing: `unmeasured`,
  confidence high for the divisions: no scenario here isolates their consequences before other
  invalid-data effects. The safe replacement is a decision for the project lead below, not a
  quirk switch.
- **No landing slot.** A jumpjet that cannot land and finds no cell nearby destroys itself
  (claim 17). This is the original's behaviour and stays the default
  (`quirks.jumpjet_no_cell_self_destruct`, default on, YR behaviour). Off: it re-enters *hovering* over its
  current cell. Backing and confidence are claim 17's; the off alternative is a design rule.
- **Ion storms.** An ion storm makes every flying jumpjet of the affected house crash, and stops a
  grounded one taking off. Ion storms do not occur in skirmish, so this is out of scope.

## Import mapping

| Key | Feeds | Parse form | Absent-key default |
|---|---|---|---|
| type key `JumpJet` | jumpjet type (glossary) | boolean | no |
| type key `BalloonHover` | balloon-hover type (S-0310 Terms) | boolean | no |
| type key `JumpjetTurnRate` | *T* (claims 11, F6; rate: S-0206 claim 14) | integer | engine default 4 |
| type key `JumpjetSpeed` | *S* | integer, leptons per tick | engine default 14 |
| type key `JumpjetClimb` | *c* | decimal, single precision | engine default 5.0 |
| type key `JumpjetCrash` | *k* | decimal, single precision | engine default 5.0 |
| type key `JumpjetHeight` | *H* (S-0102 owns its minimum) | integer, leptons | S-0102 |
| type key spelled "jumpjet" and "accel" run together, the first letter of each part a capital (keys are case-sensitive, S-0001) | *a* | decimal, single precision | engine default 2.0 |
| type key `JumpjetWobbles` | *W* | decimal, single precision | engine default 0.15 |
| type key `JumpjetDeviation` | *d* | integer, leptons | engine default 40 |
| type key `JumpjetNoWobbles` | no-wobble flag | boolean | no |
| infantry type key `JumpJetTurn` | jumpjet-turn flag (claim 10; S-0310 claim 8) | boolean | no |
| vehicle type key `IsSimpleDeployer` | deploy-to-land vehicle, with the next row (S-0517) | boolean | no |
| vehicle type key `DeployToLand` | deploy-to-land vehicle; full-height cruise (claims 11 to 13) | boolean | no |
| building art key `Height` | building clearance height (claim 9, F3) | integer, height levels of 104 leptons | engine default 2 |
| `[AudioVisual]` `DeployDir` | the game-wide deploy direction (claim 15) | integer, 8-step direction | engine default 0 at program start; a later missing layer keeps the current value |
| `[JumpjetControls]` `TurnRate`, `Speed`, `Climb`, `Acceleration`, `WobblesPerSecond`, `WobbleDeviation` | nothing (claim 2) | read and ignored | — |
| `[JumpjetControls]` `CruiseHeight` | the layer threshold (S-0102) | integer | S-0102 |

- **Key case.** Key names are matched case-sensitively. The stock rules data spells two of the
  type keys `JumpJetAccel` and `JumpJetTurnRate`. Those spellings are not the keys above and
  have no effect, so the absent-key defaults apply (claim 2; S-0001).
- **Layering.** Each key keeps its value when a later layer omits it (S-0006).
- **Backing.** The engine defaults are those of the program when no layer gives the key.
  Their flight effects are `measurable` through named test files in J1 c, confidence high.
  Defaults for flags that no J1 c flight can distinguish remain `unmeasured`: the files can
  omit the key, but this scenario has no separate target, deploy or landing observation for
  each flag. The building-height default is `measurable` in J5 and the deploy-direction
  default in J9, confidence high.

## Measurement scenarios

**Common setup.**
- Stock data, director off and mods off; game-speed index 2; 800 × 600.
- One human test house, and one computer bystander outside the test area under the glossary's
  void rule.
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
jumpjet setup: L2 tests retained facing and seized arrival (claims 10 and 12), L3 and L4
record the cell-centre detour and accelerating fall (claim 18), and L5 and L6 test blocked,
clear, water, beach and deck impact outcomes (claim 19). L3 and L6 also exercise seized
descent without the ordinary landing refusals (claims 14 and 15). The special seized
simple-deployer exclusions in claims 13 and 15 remain unmeasured as stated there.

**Scenarios.**
- **J1 Field copy and controls.**
  - (a) Order the jumpjet infantry 15 cells east and record the height profile and the per-tick
    step. Expected: the climb, crash rate, height and speed follow the type's fields, with the
    stock case-mismatched keys behaving as their engine defaults (Import mapping).
    A profile inconsistent with the imported type fields and F1–F6 falsifies claim 1.
  - (b) A test rules file changes every game-wide jumpjet control except the cruise height.
    Expected: profiles identical to (a). Any difference falsifies claim 2. In a separate run,
    change only the game-wide cruise height: the global draw-layer threshold of S-0102
    claim 3 changes, not the jumpjet's flight profile or its type-specific layer threshold.
    A changed flight profile falsifies claim 2.
  - (c) In *jumpjet-flight-test*, omit each numeric type key in turn from every layer, then
    repeat with its stated engine default explicitly supplied. Keep all other fields equal.
    Compare take-off, straight flight, wobble, turning and crash traces. Expected: identical
    traces for omission and explicit default. A difference falsifies that default. Repeat for
    the mismatched-case spellings, then the actual spellings with a different value: only the
    latter change the profile. A change from the mismatched spelling falsifies claim 2.
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
    - with nothing to do afterwards, a grounded or hovering unit without a move order keeps
      its height, wobble phase and speed fraction unchanged. A hovering unit that still has a
      move order is not this idle case.
  - Falsifiers: rising in the grounded transition tick, horizontal movement below the ground-hold
    threshold, or any idle change in these recorded values (claims 3, 5 and 7).
  - Repeat with an order to the unit's own cell. Expected: it climbs past one quarter of *H*,
    then cruises at that height and lands (claims 7 and 12). Climbing to *H* before changing
    state, or continuing to hover without a target, falsifies that own-cell result. Repeat with
    a balloon-hover vehicle variant from *jumpjet-flight-test*: its ascending speed/heading threshold
    is one half, and it continues to *H* instead. The quarter-height transition or an own-cell
    early cruise for that variant falsifies claim 7. Repeat the own-cell take-off below a high
    bridge and from its deck. Record *r*, *h* and target height. Expected: the 416-lepton
    adjustment applies only at or above deck height while not on the deck; an early own-cell
    cruise retains *r*, not *h*. Subtracting the deck height below it, or retaining *h* when
    these differ, falsifies claim 7. A balloon-hover infantry variant still uses the quarter
    threshold for its speed/heading change; a half threshold falsifies that distinction.
- **J3 Occupation.** While a jumpjet hovers over a cell, drive a ground vehicle into that cell.
  Expected: the vehicle enters without blocking (claim 4). Repeat while the jumpjet is ascending
  in the cell. Expected: the vehicle is blocked while the jumpjet still has ground occupancy.
  That mark is released as it first rises off the surface. Repeat while cruising, descending
  with a reservation, and taking off from a deck. Expected: no cruise mark, a descent
  reservation and no retained on-deck state after take-off. A cruise/hover occupancy block,
  a missing descent reservation or retained deck state falsifies claim 4.
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

## Open questions

- **Flying-object registration.** From take-off until it lands or crashes, a jumpjet is entered
  in a registry of flying objects. Which searches read that registry (for example, targeting by
  air defences) is not established here.
- **The factory-contact event.** The internal event that a climbing jumpjet raises while in
  contact with a factory building (claim 7), and the one raised at the end of a crash (claim 18),
  are not traced to an effect.
- **A crash start while idle.** A grounded jumpjet, or a hovering one with no move order, takes
  no update (claim 3). Whether something else wakes it when it starts to crash in the air is not
  traced.
- **Stopping outside a landing.** Which orders stop a moving jumpjet through claim 17 (for
  example the stop command of S-0213) is not traced. Only the failed landing is.
- **A failed move order.** When the nearby-cell search of claim 16 finds nothing, the ordered
  point is kept as the stored destination while the moving state is unchanged. How a later tick
  uses that point is not traced.
- **The crashed state.** What a *crashed* jumpjet does until S-0312's impact removes it, and what
  an ordinary dead jumpjet does on water after impact (S-0312's open question), are not settled
  here.

## Questions for the project lead

- For zero speed or zero turn rate, approve the proposed deterministic departure: a speed
  fraction of 0 for *S* = 0 and an empty middle band for *T* = 0, with an importer warning
  and no quirk switch. This draft does not yet prescribe those replacement values.
