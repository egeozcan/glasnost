---
id: S-0503
title: Hover and amphibious movement
topic: movement
revision: 28
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-02
measurements: []
requests: [Q-0679, Q-0675, Q-0690, Q-0691, Q-0692, Q-0693, Q-0694, Q-0695, Q-0696, Q-1773]
supersedes: []
---

# S-0503: Hover and amphibious movement

## Summary

How a vehicle with the hovering locomotor moves: its throttle, speed, acceleration and braking,
its straight-line steps between cell centres, its arrival at each point, its hover height, lift and
bob, the wake it leaves over water, what it does at each entry outcome of its next cell, and what
happens when it is switched off (it spins, settles and, over water, sinks), including the
switch-offs and switch-ons by a repair depot or naval yard (claims 10 and 11). It also states which
movement model an amphibious type uses, and how land and water affect the speed of hovering and
amphibious vehicles. Skirmish only; network games and campaigns are out of scope.

Against revision 16, this revision adds claim 19 (Q-1773), the deploy order of a moving hover,
with scenario H16 and two proposed quirk switches for it. No claim of revision 16 changes.

Pointers, not repeated here:
- land types, locomotion classes, land multipliers, entering a cell and zones: S-0211 claims 1–3
  and 10–12.
- the rates of the hover's movement heading and body facing: S-0206 claim 13.
- the locomotor field and the per-tick speed *s*: S-0207 claims 14 and 2.
- the driving model, for amphibious types that drive: S-0207 claims 1–12.
- what an approaching hover does once its target is in range (the discarded path and the
  arrival-time stop): S-0207 claim 19 c and d.
- the hover's exit from a factory: S-0417 claim 5.
- water cells, ships and wakes of ships: S-0504.
- shoreline unloading: S-0507 claim 7 for refusal of unload orders on water (for every player
  house in our engine), and claim 8 for an already-active unload's approach to a nearby location
  where a wheeled unit can stand before releasing passengers. That search does not promise the
  geometrically nearest location.
- approaching a would-be passenger from water or a different reach region for the passenger's
  movement zone: S-0507 claim 5. Taking the passenger as a destination is not acceptance of
  boarding; an untethered passenger breaks contact and falls back to rest, rather than waiting
  indefinitely for the transport. These transport rules do not change the locomotor at the shore.
- bridge decks as surfaces: S-0511.
- disabling by loss of a control centre: S-0405 claim 13.
- switch-off and switch-on by a repair depot or naval yard: S-0314 Terms and claim 21.
- death over water with the bomb mark: S-0312 claim 11.

**Cross-spec corrections required at hand-off.** S-0206 claim 13's body aim must follow claim 5
below, not necessarily the travel direction. S-0207 claim 19 d's stop permits claim 7's final
already-computed step. S-0405 claim 13's description of inactivity must allow the settling, spin,
residual travel and fall of claims 8 and 10–11. These corrections must be checked together at
sign-off. Further coordinated additions: S-0209 claim 1 and its between-steps definition must use
claim 13 below for hovering; S-0212 claims 8, 21 and 23 must distinguish claim 14's ρ test, claim
12's immediate-search timer restart and claim 7's cached final step. For Q-0675, S-0209 claim 1
and S-0417 claim 7 must use claim 15's arrival outcome table, not equate failed searches with
refused choices. S-0212 claims 20–23 and S-0213 claim 8 must distinguish claim 16's retained
movement destination from the cleared order-level destination. S-0209 claim 1 must also preserve
claim 13's distinct flag-setting and flag-clearing layer tests and the no-choice reservation
exception. S-0417 claims 5 and 7 must cite claims 5 and 15 for immediate first-point choice and
hover cell-arrival updates. S-0206 claim 13 must cite claim 15 for initial north and retained
placement heading. S-0213 claim 8 must cite claim 16 for the hover stop outcome.

For Q-1773 (claim 19):
- S-0213 claim 10 cites claim 19 for a hover's outcome of the deploy order's destination clear.
- S-0415 claim 2's moving rule notes that a hovering deployer reaches it only when its *unload*
  becomes current while it still holds a point; D2 part m can then use a hovering test type
  through H16.
- S-0517 claim 9's last sentence becomes: "A deployed vehicle cannot be given movement under its
  own power. A hover that became deployed in place while still moving (S-0503 claim 19) finishes,
  deployed, the movement it holds as points, unless S-0503's
  `quirks.hover_deployed_keeps_moving` is off." Rated medium, backed by H16 part j.
- S-0517 claim 4's two height-0 rows apply only while no landing request of claim 5 stands, with
  a new row "height 0, a landing request stands | nothing changes". No outcome changes for a
  jumpjet. Rated medium, `unmeasured` (claim 19 gives the reason). A hover's exception with
  S-0503's `quirks.hover_landing_request_stands` off is stated in S-0503 (Edge cases and quirks).

## Terms

From the glossary and other specs: *locomotor*, *locomotion class*, *land type*, *land multiplier*,
*entry outcome* (and its eight values), *step*, *path steps*, *stored path*, *retry delay*,
*speed fraction* ρ, *per-tick speed* (S-0207 claim 2's *s*), *heading point* (S-0207 claim 16),
*bomb mark* (S-0312 claim 11), *scenario random stream*, *ranged draw*, *facing*, *table sine*,
*table square root*, *surface height* (S-0106 claim 2), *high-bridge cell*, *deck*.

New terms:
- **Hovering vehicle.** A vehicle whose locomotor (S-0207 claim 14) is the hovering locomotor.
- **Destination** *D*. The locomotor's movement point, distinct from the order-level destination
  read by behaviours and factory reporting. The two can disagree after a stop order (claim 16).
  Unless held, assignment sets its height to the surface height of its cell, plus 416 leptons on
  a high-bridge cell. Held assignment retains the supplied point without this height adjustment.
  For a cell destination that supplied point is its centre at terrain surface height, without
  a bridge-deck lift; claim 18 gives its lifetime and order-level consequences.
- **Held.** The parasite hold timer of S-0614 claims 7–8 is running. This is the meaning of
  *paralysed* for this spec's hover assignment and choice rules, not electromagnetic-pulse disable.
  Claim 17 gives its consequences.
- **Heading point** *P*. For a hovering vehicle, the centre of the next cell of its stored path,
  at the surface height of that cell, plus 416 leptons when the
  vehicle's world height is at least 312 leptons above that surface, even without a high bridge
  (`quirks.hover_elevated_heading_point`, claim 5). A hovering vehicle
  *is moving* when it has a destination or a heading point.
- **Movement heading** η. A facing kept by a hovering vehicle besides its body facing. It turns
  towards the vehicle's heading point at the rate of S-0206 claim 13, and the vehicle travels along
  it (claim 6).
- **Throttle** *T*. A double-precision value of 0, ½ or 1, set in each moving tick (claim 2). A
  hovering vehicle *is moving now* when it is moving and *T* ≠ 0.
- **Hover speed** φ. A double-precision value from 0 to 1 kept by a hovering vehicle: the fraction
  of its per-tick speed it travels in a tick (claims 3 and 4). It starts at 0.
- **Boost** *B*. A double-precision factor kept by a hovering vehicle (claim 3). It starts at 1.
- **Vertical velocity** *v*. A double-precision value in leptons per tick kept by a hovering
  vehicle (claim 8). It starts at 0.
- **Hover height** *h*. The hovering vehicle's world height above the surface height at its
  position, less 416 leptons when it stands on a deck.
- **Powered.** A hovering vehicle is powered when it is not switched off (claim 10), or when its
  hover height is above 0. So a switched-off vehicle still counts as powered until it has settled.
- **Spin.** The short uncontrolled rotation initiated by switching off a hovering vehicle
  (claim 10); re-enabling does not itself end it.
- **Settled** (S-0314's sense). S-0314's *settled vehicle*, and *being settled* at a repair depot,
  are S-0314's own terms (S-0314 Terms). They do not mean this spec's settling down to a hover
  height of 0 (claim 10).

## Behaviour

### Which movement model

1. **Amphibious types move with their own locomotor.** The movement model of a type is chosen only
   by its locomotor field (S-0207 claim 14). The amphibious locomotion class, like every class
   (S-0211 claim 3), supplies its land multipliers and contributes to which cells the type may
   enter (S-0211 claims 10 to 12). Its movement zone is a separate imported property. For a type
   that drives, the class supplies the land multiplier that sets its target speed fraction
   on each track (S-0207 claim 9). Nothing changes the locomotor of a vehicle that crosses a
   shoreline: a driving amphibious vehicle drives on water by the driving rules, and a hovering one
   hovers on land by the rules of this spec.
   - Backing: `measurable` (scenario H1 a for hovering types). The driving side is `unmeasured`:
     no stock vehicle with both the amphibious class and driving locomotor is established.
     H1 b is a conditional proposal, not backing
   - Confidence: medium (no shoreline change of locomotor was found; absence is read, not observed)

2. **Throttle.** In each tick in which a hovering vehicle is moving and has a heading point, it
   sets its throttle *T* (F1):
   - *T* = 0 when it is not powered, or when its movement heading η differs from the direction β
     from its position to the heading point by more than 8192 facing units (45°).
   - otherwise *T* = ½ when it is strictly less than 256 leptons (one cell) from its destination, or, when it
     has no destination, from its heading point, and *T* = 1 otherwise.

   Thus ordinary acceleration starts within 45° of the heading point. The stalled-start rule
   (Edge cases) is an exception. Near its destination it selects half throttle, not necessarily
   half actual speed: boost and the acceleration/braking ramp still apply. The distance test is
   strictly less than 256 leptons and includes height (F1).
   - Backing: `measurable` (scenarios H2 and H3)
   - Confidence: high

3. **Acceleration, braking and boost.** In the same tick, the resulting boost and hover speed are
   determined from the throttle according to F2:
   - while *T* > 0, *B* = the hover boost factor when its stored path has at least two steps and the
     first two are equal (a straight run), and *B* = 1 otherwise. While *T* = 0, *B* keeps its value.
   - the goal is *G* = min(1, *B* × *T*).
   - when φ < *G*, φ rises by 1 / (*a* × 900), but not above *G*, where *a* is the hover
     acceleration field.
   - then, when φ > *G*, φ falls by 1 / (*b* × 900), but not below 0, where *b* is the hover brake
     field. The floor is 0, not *G*, so φ can fall up to one braking step below *G*, and it rises
     back towards *G* on a later qualifying tick.

   This ramp applies only while moving with a heading point; it is not a passive braking rule
   after the move ends. The undershoot is controlled by `quirks.hover_brake_undershoot`.

   As a result, with a boost factor of 1 or more the boost has no effect at full throttle (the goal
   is capped at 1), and it raises the half-throttle goal only while two equal stored steps remain.
   A boost factor below 1 slows straight runs. The fields are in game minutes, at
   900 ticks per game minute (real-time tick rates are S-0201 claims 2 and 5): *a* is the time from rest to full speed, *b* from full speed to rest,
   ignoring the step rounding.
   - Backing: `measurable` (scenario H2 for the ramp). Boost is `unmeasured` pending a stock
     detour satisfying H2's discriminating half-throttle condition; ordinary full-throttle
     straight runs cannot distinguish a boost of at least 1 from a boost of 1
   - Confidence: high for the ramp and the braking floor; medium for the boost condition

4. **Step length; land and water.** In each tick in which it is moving, a hovering vehicle travels
   a provisional distance *d* = trunc(*s* × φ) leptons (F3), subject to the arrival and refusal
   outcomes of claim 7, where *s* is S-0207 claim 2's per-tick speed, read here in
   leptons per tick. *s* contains the speed fraction ρ. A hovering vehicle sets ρ only to 1 or 0:
   to 1 when it is given a move while powered and not moving now, and on a tick on which it is
   moving with no heading point; to 0 for a refused step or a forced immediate halt
   (claims 7 and 12). A person's stop order is not that forced halt (claim 16). The switch-off stop request of claim 10 does not itself zero ρ. It also sets ρ to
   1 on a tick on which *d* = 0 while its movement heading points exactly at its heading point (a
   stalled start, Edge cases). Any other value set by another rule stays until one of those events.
   So the ρ = ½ that S-0417 claim 5 sets after a factory exit can persist during the exit move unless
   its movement heading lands exactly on the direction of its heading point while its hover speed
   is still too small to give a step of 1 lepton.

   No land multiplier enters *d*: a hovering vehicle travels at the same speed on every land type it
   may enter, water, beach and land alike. The land multipliers of its class (S-0211 claim 2)
   matter to it only through which cells it may enter (a multiplier of 0 forbids entry).
   - Backing: `measurable` (H1 for land independence, H3 and H7 for ordinary endings,
     H8 for the factory exit and stalled-start reset, H12 for stop orders)
   - Confidence: high for the land independence; low for how long the factory-exit half speed
     lasts

5. **Choosing the next heading point.** A powered hovering vehicle that is not moving now
   attempts its first path search and heading-point choice immediately when given a nonempty
   destination, not on its next movement tick. This includes a resting factory-produced hover:
   on a successful clear choice it already has its heading point before the factory sets
   ρ = ½ (S-0417 claim 5). A refused choice or unsuccessful search need not supply a point.
   A held vehicle merely remembers the supplied destination without this immediate choice or
   height adjustment (claim 17).
   A powered vehicle already moving now keeps an existing heading point when given a new
   destination. If powered and moving now but without a heading point, it sets throttle and
   ρ to 1, applies the path-update rules immediately, and attempts heading-point choice there
   too. This path update is not an unconditional fresh search: it includes destination-cell
   completion and the ordinary retry-delay gate of S-0212 claims 20–23. An actual fresh search
   starts the retry delay by claim 12; a timer-deferred update does not restart it.
   Giving a destination while unpowered does not itself start the choice.

   A vehicle also attempts a new heading-point choice on moving ticks without a heading
   point and on arrivals at its heading point (claim 7):
   - with no stored step left, it has no heading point, and the path-search rules of S-0212
     (claims 20 to 23, with the hover's in-range rules of S-0207 claim 19 c and d) apply.
   - otherwise the heading point is the centre of the cell its first stored step enters (Terms),
     and the step check of that cell (claim 12) decides whether the vehicle takes it. On *clear*
     the step is used up and the vehicle heads for that point. A crate in that cell is collected
     when the cell is chosen, before the check.

   While it has a throttle above 0, its body facing turns (at its own rate, S-0206 claim 13)
   towards the point one neighbour step (256 leptons on each affected map axis) beyond the heading point in the direction of its
   next stored step, or towards the heading point when there is no further step. So the body looks
   one cell ahead along the path; the body facing does not steer the vehicle.
   - Backing: `measurable` (H2 for body aim, H3 for heading-point selection, H10 for crates,
     H8 for the immediate move-start choice; H14 for held assignment and the already-moving-now
     no-point path update). Assignment while not powered is `measurable` in H14.
     The elevated heading point on a non-bridge cell is `unmeasured`: no stock route isolating
     that height condition is established
   - Confidence: medium

6. **Straight-line travel.** A hovering vehicle travels in a straight line along its movement
   heading, not along tracks: in each tick in which *d* > 0 its ground position moves by *d* in the
   direction of η's current value, by the same planar step formula as S-0208 claim 6 (F4). Its world height is not changed
   by this step (claim 8 sets it). Its listing in cell contents moves as its position crosses
   a cell edge; this is not the timing of its vehicle-present flags (claim 13). When it enters a high-bridge cell with a hover height of 416 or more above
   the cell's surface it is on the deck, and when it enters a cell that is not a bridge cell it is
   off any deck (S-0511 owns bridge surfaces).
   - Backing: `measurable` (scenario H3)
   - Confidence: high

7. **Arrival at the heading point.** At the start of a tick's travel, when the distance along the
   map (heights play no part) from the vehicle to its heading point is at most *d* (F5), the vehicle
   has arrived there. It is not placed on the point: it initially keeps its position, up to *d*
   leptons from the point. The test alone neither snaps it to the point nor cancels the travel
   distance already computed for this tick. Then:
   - it attempts the next heading-point choice when claim 15's destination/contact condition
     permits it (claim 5). When one is chosen, it travels *d* on the same
     tick along its current movement heading (claim 6), still aimed towards the old heading point,
     so at a turn it can swing wide of the cell centres.
   - when no new heading point is chosen, any positive *d* already computed can still produce a
     final step on this tick; absence of a heading point alone does not suppress it. When it has
     reached the cell of its destination, at a height difference strictly less than 208 leptons,
     its destination is cleared and the move ends; otherwise S-0212 claims 20 to 23 decide whether
     it searches again. Once not moving, it takes no further travel ticks and the ramp of claim 3
     does not run. Clearing a destination alone does not promise that throttle and hover speed
     become 0.
   - a refused next cell zeroes throttle, hover speed and ρ (claim 12). Waiting suppresses further
     travel on that tick, except that a give-up can still take the already computed positive *d*
     as a final step. After the destination is cleared there are no later travel ticks until
     another move starts.

   A failed fresh search on an arrival tick, including either failed-search distance case of
   S-0212 claim 23, does not cancel the already computed positive *d*: the surviving on-map
   vehicle can take that final step along its old movement heading even though the search
   cleared its destination and heading point. The flag and cell-arrival update are not implied
   by this final travel: claim 15 distinguishes failure before choice from failure during choice.
   This also applies when the first refusal caused
   an immediate search and that search failed: zeroing φ and ρ during the refusal does not
   change the already computed *d*. It is the resulting nonzero entry outcome that can
   suppress travel, not search failure alone.
   - Backing: `measurable` (scenarios H3 and H9)
   - Confidence: medium (the stop position and the same-tick continuation are read, not observed)

### Hover height

8. **Lift, gravity and bob.** In every tick, moving or not, a hovering vehicle sets its hover height
   (F6).

   | Observable stage of the tick's height update | Result |
   |---|---|
   | Height and bob | The height before lift and gravity is the old hover height plus the old vertical velocity, truncated toward zero, plus the periodic bob term (at most ±2 leptons per tick), with the resulting sum truncated toward zero. If this result is below 0, the resulting hover height is 0 and the vertical velocity used by the remaining height update is 0. Otherwise, the resulting height is that sum and the velocity used by the remaining update is the old vertical velocity. |
   | Lift | If the effective height *h*ₜ is below the hover height field *H*ₕ and the vehicle is not switched off, the velocity used by the remaining update gains ((2*H*ₕ − *h*ₜ) / *H*ₕ) × *g*. If *h*ₜ is also below trunc(*H*ₕ / 4), it additionally gains trunc(*g* / 3), whether or not the vehicle is switched off. |
   | Gravity and damping | In every tick, subtract *g* from the velocity after the applicable lift gains and multiply the result by the hover dampen field; this is the new vertical velocity. |

   The effective height *h*ₜ is the hover height, except when the vehicle has a stored step and the
   surface one neighbour step ahead in that step's direction is higher than the surface under it:
   then *h*ₜ is the hover height less *H*ₕ. So when climbing (from water onto a higher bank, for
   example) the lift is stronger. Because claim 6 keeps the world height while the surface rises,
   the hover height drops on a climb and may be clamped at 0 (with *v* = 0) for a tick; on a descent
   the vehicle is briefly higher and gravity brings it down. Water and land surfaces are treated
   alike: a shoreline matters only through the surface height (S-0106, S-0504).

   The bob's period is trunc(*m* × the hover bob field × 900) ticks, with *m* = 1 for a vehicle
   whose object number is odd and *m* = 1.1 for one whose object number is even, and its phase is set
   by the tick number plus twice the object number (F6). So hovering vehicles bob out of step with
   each other. In our engine the object number is our own creation counter, not the original's
   (Edge cases).
   - Backing: `measurable` (scenario H4 for the settled height and the bob, H5 for the climb)
   - Confidence: high for the formula; medium for the settled height values it implies

9. **Wake over water.** In each tick in which a hovering vehicle is moving now and the tick number is
   divisible by 10, it creates one instance of the game-wide wake animation at its position,
   provided it is off a deck, its cell's land type is water, and the animation is defined. The
   wake is created after the tick's travel. This is the same rule as S-0207 claim 23 with the
   hover's *moving now*. A hovering vehicle at rest (throttle 0) leaves no wake.
   - Backing: `measurable` (scenario H6). The missing-animation guard is `unmeasured`:
     stock data defines the wake, so H6 does not test its absence
   - Confidence: high

### Switching off

10. **Switched off: spin and settling.** A hovering vehicle is switched off by the disabling of
    S-0405 claim 13 and by each switch-off of a repair depot, the first and every repeated one.
    It is switched on again by its re-enabling and by each switch-on of a repair depot, even when
    it is still disabled. S-0314 (Terms and claim 21) states when a depot switches a vehicle off
    or on; this claim does not. This claim states what a switch-off and a switch-on do to a
    hovering vehicle. Each switch-off meets the conditions below on its own, so a repeated
    switch-off of a vehicle that is still above the ground takes the two draws again whenever
    those conditions hold at it (none while its current behaviour is *sleep* or *enter*).
    When it is switched off while it is powered, and its current behaviour is neither *sleep* nor
    *enter*:
    - it takes two ranged draws from the scenario random stream, first R(20, 30), giving the spin
      count *n*, then R(0, 99); when the second is below 50, *n* is negated.
    - When a spin is already active, these two new draws replace its spin count *n*: the spin
      goes on from the new count, and the old count is lost. A repeated switch-off during a spin
      therefore restarts the spin's countdown rather than adding to it or leaving it alone.

    These conditions govern only the start of a spin. Whenever it is switched off, a moving
    vehicle receives a stop request, and its powered lift (claim 8) stops.

    While the spin is active and the vehicle is powered, its body facing turns by *n* × 256
    facing units each tick, and *n* moves one step towards 0 each tick. A vehicle that settles to
    height 0 on this tick while switched off has no further spin rotation on that tick. The spin
    ends on the first tick on which *n* reaches 0 or the vehicle is no longer powered (it has
    settled to a hover height of 0). Without lift it sinks under gravity (claim 8) and rests on the
    surface. When it is switched on again, the lift returns and it rises to its hover height.

    The stop request clears the destination and stored path only when destination and heading
    point differ as full three-dimensional points; otherwise it leaves them. It does not itself
    zero throttle or hover speed. Thus stopping is not necessarily instantaneous horizontal rest.
    The stop request itself never touches the order-level destination (Terms, *Destination*): the
    stop request of a repair depot's switch-off, the first or a repeated one, leaves the vehicle's
    destination that is the depot in place. The depot's removal of that destination is S-0314's
    (Terms and claims 21 and 22).

    A vehicle not powered, or in *sleep* or *enter*, takes no draws and starts no new spin, but
    still receives that stop request and loses lift. Re-enabling restores lift without cancelling
    an already active spin, which can continue until its count reaches 0.

    Whether shots at a switched-off hovering vehicle are led follows from whether it is moving
    (Terms), as S-0306 claim 3 b gives it (quirks.switched_off_hover_lead). One that has a
    destination or a heading point when it is switched off is moving, and shots at it are led while
    it keeps either. One that has neither is not moving, so shots at it are not led: for example one
    that arrives at its heading point by claim 7's test with no destination and no stored step left
    after that arrival (claim 15). Losing the heading point alone, as at a refused step (claim 12),
    leaves it moving while it keeps its destination.

    Backing and confidence, per sub-claim:
    - *Ordinary disabling: the draws, the spin, the settling-tick rotation cutoff, the stop
      request and the loss and return of lift.* Backing: `measurable` (scenario H7). Confidence:
      high.
    - *The exceptions: sleep or enter, the equal-point stop request, and re-enabling during a
      spin.* Backing: `unmeasured`: no stock input sequence isolating these states is yet
      established, and H7's early-restoration subtest is conditional. Confidence: medium.
    - *New draws replacing an active spin count.* Backing: `unmeasured`: it needs a second
      switch-off that takes the draws while a spin is still active, and no scenario ensures one
      (H7's early-restoration subtest is conditional, and the depot's repeated switch-offs are
      `unmeasured` below). Confidence: medium.
    - *A depot's first switch-off and the draws it takes.* Backing: `measurable` (S-0314 H4
      part i run 1). Its tick, at the movement arrival and at the processing that makes the
      vehicle settled in S-0314's sense, is `unmeasured`, as in S-0314 claim 21. Confidence:
      medium.
    - *The loss of lift at the depot.* Backing: `measurable` (S-0314 H4's hovering repeat, which
      S-0314 calls an attempt). Confidence: medium until that repeat runs.
    - *A depot's repeated switch-offs and which of them take the draws.* Backing: `unmeasured`:
      run 1 sets up no later processing that finds the vehicle above the ground in a behaviour
      other than *sleep* and *enter*. Confidence: medium.
    - *A depot's switch-on of a vehicle that stops being settled by receiving a destination.*
      Backing: `measurable` (S-0314 H18 part b's hovering repeat, in a run that has the window
      that repeat needs). Confidence: medium.
    - *A depot's switch-on at the other two ends of being settled* (a waiting processing that
      does not accept the vehicle, a settling processing that finds it moving). Backing:
      `unmeasured`, as in S-0314 claim 21. Confidence: medium.
    - *A depot's switch-on of a still-disabled vehicle.* Backing: `unmeasured`: it needs a
      disabled vehicle in contact to meet an end of being settled, and no scenario ensures that.
      Confidence: medium.
    - *The stop request of a depot's switch-off leaving the order-level destination in place.*
      Backing: `measurable` (S-0314 H4 part a's hovering repeat, an attempt, whose recorder logs
      the order-level destination on every tick and expects it to be the depot from the arrival
      until the processing that takes the settling step). A repeated switch-off is covered only
      when one falls in that window. Confidence: medium.
    - *Whether a switched-off vehicle is moving, and so led.* Backing: `measurable` (S-0306 scenario
      P45 part a, which records the heading point, hover speed and moving state of each vehicle of a
      switched-off group). Confidence: medium, as for the exceptions above.

11. **Sinking over water.** On the tick on which the spin of claim 10 ends, when the vehicle's cell
    is a water cell and its world height is less than 104 leptons (one level) above the surface,
    it is given the bomb mark and starts to fall. Upon dying within 10 leptons above the water,
    S-0312 claim 11 supplies wake and splash instead of the ordinary vehicle death effects, and
    its passengers do not get out. A vehicle whose spin ends while still at least 104 leptons
    above the water does not start this fall. Thus a switch-off over flat water that starts a spin
    (claim 10) leads to loss after settling, unless the spin ends at least 104 leptons above
    the water. This applies to every switch-off of claim 10 that starts a spin, including one by
    a repair depot or naval yard over water (S-0314 claim 21) whenever that switch-off takes the
    draws. A switch-off that starts no spin, while no spin is active, makes no such test, and the
    vehicle rests on the water. A vehicle settling on land or a bridge deck survives this test.
    The test belongs to the end of a spin, even if the vehicle has meanwhile been re-enabled.

    Backing and confidence, per sub-claim:
    - *The ordinary low-water fall, destruction, animations and land or deck survival.* Backing:
      `measurable` (scenario H7). Confidence: medium (the fall's end is S-0312's; in stock play
      the vehicle settles before its spin count runs out, so the test is made close to the
      surface).
    - *The exact 104-lepton boundary, and a spin ending after re-enabling.* Backing:
      `unmeasured`: H7 establishes neither a spin ending near that boundary nor unconditional
      early restoration. Confidence: medium.
    - *Passenger loss.* Backing: `unmeasured` here: no stock passenger-carrying hover type
      dependent on a control centre is established; its rule inherits S-0312 claim 12.
      Confidence: medium.
    - *A disabling at a naval yard while the vehicle is above the water.* Backing: `measurable`
      (S-0314 scenario H4 part i run 6 (i), which uses a named test type and may be reported
      unreached). Confidence: medium until that run runs.
    - *A switch-off by the depot or yard itself during a visit.* Backing: `unmeasured` here, as in
      S-0314 claim 21: which of those switch-offs take the draws is open there. Run 6 (ii) records
      whether the visit's own switch-offs took draws before its cut, and any loss that follows,
      but nothing in it ensures that one of them takes the draws, so it may show none.
      Confidence: medium.
    - *A switch-off that starts no spin, while no spin is active, makes no test.* Backing:
      `measurable` (S-0314 scenario H4 part i run 6 (ii), its no-draw control: the power is cut
      with the vehicle down on the water and already switched off, so the cut takes no draws and
      starts no spin; a loss at the cut would show a test made without a spin). The run uses a
      named test type and may be reported unreached, and it is inconclusive for the cut when a
      loss before the cut follows logged draws. Confidence: medium until that run runs.

### The step check

12. **What a hovering vehicle does at each entry outcome.** The step check of the next cell
    (claim 5) has the results of table T1. A *first check* is the one made when the cell is chosen
    after an arrival or a new move; a *repeated check* is the one made on the same tick after a
    fresh search that a first check started. Every row other than *clear* leaves the vehicle without
    a heading point and with throttle, hover speed and ρ at 0, unless the row says otherwise. A
    vehicle that keeps its destination and has no heading point tries again on later ticks through
    the path-search rules (S-0212 claims 20 to 23).

    **T1. The step check of a hovering vehicle.**

    | Outcome | First check | Repeated check |
    |---|---|---|
    | clear | it heads for the cell centre (claim 5) | as the first check |
    | cloaked blocker | the cloaked objects in the cell become visible to its house (S-0209 claim 9). It discards its stored path and makes a fresh search at once, whatever its path retry timer shows (unlike S-0207 claim 15, where this search waits for the timer). When the search finds a path, the repeated check follows on the same tick | the cloaked objects become visible again. It keeps its destination and waits (it does not give up the move, unlike S-0207 claim 15) |
    | blocked | it reveals cloaked objects in the cell as in the cloaked-blocker row, then discards its stored path and makes a fresh search at once, whatever its path retry timer shows. When the search finds a path, the repeated check follows on the same tick | it gives up the move: its destination is cleared |
    | moving blocker | S-0212 claim 22's blockage timer and re-path rule, as for the driving locomotor | as the first check |
    | closed friendly gate | it tells the gate to open and waits | as the first check |
    | friendly obstacle, enemy obstacle | it discards its stored path and makes a fresh search at once, whatever its path retry timer shows. When the search finds a path, the repeated check follows on the same tick | as S-0207 claim 15's repeated check: it attacks the first listed object when its house is not allied with it, or, when no object is listed, a wall in the cell, by the override of S-0215 claim 10. Otherwise it attacks nothing and waits |
    | stationary friendly | as for an obstacle | when the give-up test of S-0207 claim 15 holds, it gives up the move (its destination is cleared). Otherwise it sends the same scatter request as S-0207 claim 15 (no threat point, forced, move-anywhere) to the cell layer it moves in (the deck layer when the cell is a high-bridge cell and the vehicle's height level differs from the cell's by 3 or more), and waits |

    After any fresh search that finds a path, the cell the first step of the new path enters is
    looked at as in S-0207 claim 15 (*the first cell of a new path*), with two differences: there is
    no gate row and no train exception, and the nearest object is looked for in the cell's deck layer
    when the vehicle's world height is more than 312 leptons (three height levels) above the
    surface at that cell's centre, not 208.
    Every actual fresh search in T1's immediate-search rows starts the full retry delay
    *R* = trunc(*p* × 900) ticks from that search's tick, whether it finds a path, fails in
    the search, or fails at its reachability gate. Here *p* is the imported path-retry delay
    in game minutes (S-0212 claim 21). These rows bypass a running timer to make this search.
    They do not remove the new timer on success. The ordinary no-heading-point retry waits
    for that timer. A first-check refusal later can again bypass it.
    - Backing: `measurable` (H9 for clear, cloaked blocker, blocked, moving blocker, closed
      friendly gate, stationary friendly and enemy obstacle). A repeated friendly-obstacle
      outcome, the missing gate response at the new-path inspection, and the exact deck-layer
      and nearest-object distinctions after a fresh search are `unmeasured`: H9 does not establish
      stock placements isolating these from passability or the ordinary step check. These are not confirmed by an ordinary unobstructed crossing
    - Confidence: medium for the table; low for the first cell of a new path

### Reservations, placement and orders

13. **Vehicle-present flags and between steps (Q-0691).** A newly created hover starts
    between steps. Each F5 arrival puts it between steps again. Choosing a heading point does not
    itself end that state: it ends on the next positive-distance travel with a heading point,
    including travel later on that same arrival tick. Thus a hover with a heading point can
    still be between steps while waiting to make its first positive advance. A heading point
    absent after a refusal does not by itself set the state either.

    A clear heading-point choice reserves that point's cell at once by setting its
    vehicle-present flag, before physical entry. Replacing or abandoning a heading point
    clears its old reservation, except for claim 15's no-choice idle-arrival case.
    On the positive advance that ends the between-steps state,
    the flag at the vehicle's actual position is cleared before its position moves, even if
    it remains in that cell. While not between steps, cell-edge crossings move the listing
    but neither set nor clear these flags: cells merely swung through at a turn are not
    reserved. If a crossing occurs while still between steps, removal clears the flag at
    the old actual position and insertion sets it at the new one. Claim 15 gives the exact
    arrival cases that restore a flag at the actual position, including a deferred search.
    Setting a flag, at either a chosen point or the actual position, uses deck occupancy only
    when that cell has a high bridge and that point is at least 416 leptons above terrain
    surface there, otherwise ground occupancy. Clearing a flag uses the point's height alone:
    at least 416 leptons above terrain clears deck occupancy even without a high bridge,
    below that clears ground occupancy. This asymmetry is controlled by
    `quirks.hover_flag_clear_height_only` (default original); with it off, clearing requires
    the same high-bridge condition as setting. Neither operation uses the vehicle's listing layer.
    These are flags, not counts
    attributed to individual vehicles (S-0209 claim 1).
    - Backing: `measurable` (H11)
    - Confidence: high for reservation and first-advance release; medium for exceptional
      between-steps crossings and no-heading-point restoration

14. **Speed used by the blocker chain (Q-0691).** For S-0212 claim 8's choice of the cell a
    hover is heading into, nonzero current speed means ρ ≠ 0, tested as a double-precision
    fraction. It does not mean φ > 0, *d* > 0, or *s* × φ > 0. Thus even a hover still turning
    with φ = 0 can send the chain along its body facing when ρ is nonzero. When ρ = 0 the
    first remaining stored path direction supplies the heading cell instead, or none if
    there is no stored step. The movement heading is not the body facing used by that test.
    - Backing: `measurable` (H11)
    - Confidence: high

15. **Cell-arrival updates and initial movement heading (Q-0693, Q-0694).** A clear choice
    of a heading point produces a cell-arrival update for S-0417 claims 7–8 at the vehicle's
    actual position, not the chosen point. This includes the first choice when a move is
    given and a further clear choice on an F5 arrival tick. The point is chosen but its
    reservation and consumption of the stored step have not yet taken effect at that update.
    Consequently factory reporting observes the pre-travel position. For an F5 arrival,
    the initial path update of claim 7 can already finish or fail a move before heading-point
    choice. Choice is attempted when *D* still exists after that update, or when the contacted
    object is in the vehicle's present cell. An absent order-level destination does not forbid it.
    The following table concerns a surviving on-map vehicle, before any cached final travel.

    | Arrival situation | Flag and cell-arrival update |
    |---|---|
    | No *D* after the initial update, no contacted object in the present cell | No choice, no actual-position flag restoration and no cell-arrival update. With no stored step, *P* is cleared. This includes an ordinary stop with unequal points and no contact. Merely clearing *P* here does not clear its old reservation: absent another clearing event, that flag remains at the former heading cell (`quirks.hover_idle_arrival_reservation`). |
    | No *D*, but contacted object in the present cell, and vehicle not held | Choice is attempted and ends without a point. The old heading reservation is cleared if still present, the actual-position flag is set, and a cell-arrival update follows. No fresh search is made without *D*. |
    | *D* remains, no stored step, retry delay still running, vehicle not held | Choice clears the old heading reservation and leaves no point. The actual-position flag is set and a cell-arrival update follows. The vehicle waits marked at its actual cell, not the old heading cell. |
    | A search fails during choice, vehicle not held | No point results, but this is still a non-refusing choice: the actual-position flag is set and a cell-arrival update follows, even if that search clears *D*. |
    | A search fails in the initial arrival path update and clears *D* and *P* | The no-*D* rows above decide whether a choice and restoration follow. Without same-cell contact there is no restoration or update, and the failed search has cleared the old heading reservation: no replacement flag is supplied. |
    | Choice refuses entry, including a held vehicle | No actual-position restoration or cell-arrival update from that choice. Claim 12 governs refusal. |

    Search success is not required for restoration and an update, but reaching the
    non-refusing choice outcome is.
    In each restoration row the cached final step may still cross a cell boundary, transferring
    the actual-position flag by claim 13. Thus its eventual waiting cell is the post-travel cell.
    Cell-edge crossing is not itself a cell-arrival update.

    A newly attached hovering locomotor starts with movement heading north, 0 facing units,
    independently of the body facing used to place the vehicle. Ordinary placement on the
    map, factory release, and removal and replacement for teleportation do not reset an
    existing hover movement heading. A newly produced hover therefore starts north, whereas
    a teleported hover retaining its locomotor retains its previous heading and any turn
    already in progress. A replacement with a newly attached hovering locomotor starts north
    again. Body-facing placement does not copy into this heading.
    - Backing: `measurable` (H8 for factory updates, H13 for initial and retained headings,
      H15 for absent destinations and failed or deferred arrival searches)
    - Confidence: high for clear-choice updates and initial north; medium for exceptional
      completion and retained-heading placement cases

16. **A person's stop order (Q-0696).** When the stop order is accepted under S-0213 claim 8,
    its order-level destination is cleared, but the locomotor's destination *D* and stored
    path obey the same point-equality rule as claim 10's stop request: when *D* and *P*
    differ as full three-dimensional points, *D* and the stored path are cleared. When
    equal, that request leaves them. In either case it keeps its heading point, throttle,
    hover speed and ρ at that instant. It is not a forced immediate halt and does not set
    ρ to 0. A hover with a heading point remains moving, updates throttle and hover speed
    by claims 2–3 using distance to that point (also the retained destination when equal),
    and travels towards it. On an ordinary F5 arrival with no remaining path, the
    heading point is cleared and any equal retained destination finishes by claim 7. The
    already computed step may still be taken (claim 7): neither a snap to the centre nor
    an exact-centre stop is promised. Once both points are absent, no subsequent travel
    or passive speed ramp occurs. A hover already without a heading point ceases moving
    when the destination is removed. Target, order queues, behaviour and stop-order
    refusal conditions remain S-0213 claim 8's, not new locomotor exceptions.

    Behaviours and S-0417 claim 7's report tests see the cleared order-level destination,
    not a retained *D*. Hover throttle, moving-state tests and the completion/search rules of
    S-0212 as applied here use *D*. A search targets *D*'s cell even with no order-level
    destination. Clearing the order-level destination also makes the ordinary retry timer
    immediately eligible; the next actual search starts its full delay as usual.
    When *D* equals *P* and the ordinary destination-cell/height completion condition holds,
    arrival still completes: the retained *D* makes choice possible, choice removes *P*,
    and completion then removes *D*. Its non-refusing no-point outcome restores the
    actual-position flag and gives a cell-arrival update even without same-cell contact.
    With unequal points and *D* removed by stop, claim 15's no-*D* rows apply instead.
    An accepted person's stop also breaks contact (S-0213 claim 8), so an unchanged
    factory contact must not be assumed for a later report; a new contact, if any, uses
    the order-level destination tests above.
    - Backing: `measurable` (H12 and H15)
    - Confidence: high for the retained movement quantities; medium for the final position

17. **Held versus switched off (Q-0675).** The hold test used by hover destination assignment
    and heading-point choice is precisely whether the parasite hold timer has ticks left
    (S-0614 claims 7–8 and 13). A held vehicle can still be powered: being held does not
    itself switch off lift. A held destination assignment remembers the supplied point only,
    a held heading-point choice clears its old reservation and point but refuses as *blocked*.
    EMP disable, control-centre switch-off and magnetic lifting do not themselves satisfy
    this timer test. Switch-off separately affects the powered test (Terms and claim 10),
    including its positive-height exception. Magnetic lift uses the replacement movement
    model of S-0607 claims 1–3, not a new hover hold condition. A parasite's presence alone
    is insufficient once its hold timer has expired.
    - Backing: `measurable` (H14); the timer's identity inherits S-0614 claims 7–8.
      The EMP-only negative case is `unmeasured`: no stock skirmish activation is established
      for H14, although the hold test itself is settled
    - Confidence: high

18. **Destination supplied during hold (Q-0679).** For a destination assignment that reaches
    hovering movement while held, a cell supplies the integer point
    (256*x* + 128, 256*y* + 128, *z*ₛ) leptons, where (*x*, *y*) are the cell coordinates and
    *z*ₛ is its terrain surface height at the centre (S-0106 claim 2). The arithmetic is exact
    integer arithmetic. This is not height 0 unless the surface there is 0, and it does not
    use the issuing unit's height. On a high-bridge cell it still omits the 416-lepton deck
    lift. The held hover remembers this point as *D*, whereas an unheld assignment applies
    the destination-height adjustment in Terms.

    Expiry of the hold, the next path search and the next heading-point choice do not adjust
    this remembered *D*. A heading point still gets its own height by claim 5, independently
    of *D*. Until another assignment replaces it or stopping/completion/failure clears it,
    *D* retains the height originally supplied. An assignment made after hold expiry applies
    the ordinary height adjustment anew, even when it names the same cell. Consequently
    raised terrain alone does not leave a held cell destination at height 0. On a high bridge,
    however, a retained terrain-height destination can fail claim 7's arrival-height test
    for a vehicle on the deck; later searching does not silently repair that height.

    The order-level destination changes to the assigned cell or object in the ordinary way
    even while held. Behaviours and S-0417 claim 7's report tests see that new reference,
    not the previous destination and not *D*. Omitting the immediate hover choice does not
    itself generate a cell-arrival update or a factory clear report. This specifies the
    result of an assignment, not permission to issue an otherwise refused move order:
    S-0614 claim 8 and S-0214's order rules still govern held-unit order legality.
    - Backing: `measurable` (H14, raised-ground and high-bridge assignment subtests)
    - Confidence: high

19. **A deploy order (Q-1773).** The deploy order's single destination-clear request (S-0213
    claim 10) reaches hovering movement as the same stop request as a person's stop order
    (claim 16) and claim 10's switch-off stop. When the order-level destination is cleared,
    *D* and the stored path obey the point-equality rule: when *D* and *P* differ as full
    three-dimensional points, both are cleared. When they are equal, both are left. In either
    case the heading point, throttle, hover speed and ρ are kept, and the order forces no
    immediate halt. A vehicle with no heading point stops in its cell only because no point is
    left to move to (the table's last row). *D* = *P* can hold only on the final leg of a move, when the heading point
    is in the destination cell. The final leg is necessary, not sufficient: the points are
    compared in all three coordinates, and they differ on that leg when only one of them carries
    the 416-lepton lift of Terms. For example, *P* is lifted when the vehicle is at least 312
    leptons above the surface, while *D* in a cell without a high bridge is not. That final leg
    then takes the unequal row.
    Being close to the next cell does not make the points equal.
    When the order-level destination is already empty and *D* and *P* have not changed since
    the clear that emptied it, the order leaves *D*, the path and *P* untouched, whether the
    vehicle assignment is then suppressed as unchanged (as S-0213 claim 21 states for a jumpjet
    vehicle) or made: a repeated point-equality clear on unchanged points changes nothing. When
    *D* or *P* has changed since that clear, whether the order acts on them is open (Open
    questions). This spec extends to the hovering locomotor the
    balloon-hover retention that S-0213 claims 10 and 21 state for balloon-hover types of the
    jumpjet kind: a hovering type with the balloon-hover field that meets that retention
    predicate keeps its order-level destination, *D* and stored path. No other spec states the
    retention for a hovering vehicle; it is this claim's own rule.

    | Situation when the order takes effect | Movement that follows |
    |---|---|
    | Heading point present, *D* ≠ *P* (midway along a path of several cells, or a final leg whose heights differ) | *D* and the rest of the path are dropped. The vehicle goes on to *P*, the centre of the next path cell, by claims 2–3 and 7. Its arrival there has no *D*, so claim 15's no-*D* rows apply: *P* is cleared and the already-computed final step may still be taken. It is then no longer moving. |
    | Heading point present, *D* = *P* (final leg, equal heights) | *D* is kept. The vehicle arrives at the destination as an ordinary arrival (claim 16's equal case). When claim 7's completion test holds there, the move ends and it is no longer moving. When it does not (for example claim 18's height case), *D* remains and the searches of S-0212 target *D*'s cell (below). |
    | No heading point | *D* is unequal to the absent point and is cleared. The vehicle is no longer moving from the moment the order takes effect, in its present cell. |

    The table covers a vehicle whose order-level destination is cleared by the order. Two cases
    leave the movement as it was, and the table does not apply to them:
    - **Suppressed assignment.** When the order-level destination is already empty and *D* and
      *P* are unchanged since the clear that emptied it, *D*, the stored path and *P* are
      untouched. The vehicle goes on with the movement it already has,
      and stops where that movement would have stopped without the order (claims 7, 15 and 16),
      which can be more than one cell centre further.
    - **Balloon-hover retention** (this spec's extension, above). A retaining hovering type
      keeps its order-level destination, *D* and stored path. It goes on to *D* as an ordinary
      move, along the whole remaining path, and stops where that move ends.

    In every other case the vehicle travels at most to the heading point current when the order
    takes effect, plus the one already-computed final step of claim 7 on its arrival there. The
    one exception within the table is an equal *D* that is not completed on arrival; its later
    movement is the search's, below. Orders take effect after the turn pass (S-0203
    claim 7). When the order's tick is a tick in which the vehicle arrives at a cell centre and
    chooses its next heading point, the order sees that new point, and the vehicle goes on one
    more cell. When the order's tick is earlier, the vehicle stops on its arrival at the heading
    point it is reaching, plus at most that one final step (whose size and overshoot are open:
    Open questions, *Arrival and stopping*).
    The same holds for a type that deploys into a building and for an in-place deployer.

    Afterwards the vehicle does not move along the old path again without a new order. In the
    unequal case the remaining path is gone. In the equal case only the destination cell
    remained, and the vehicle has reached it. The behaviour that a failed deploy attempt queues
    (S-0415 claim 2) does not resume the old path. Any later movement comes from that
    behaviour's own target rules, not from the old path. If an equal retained *D* is not completed on
    arrival (for example claim 18's height case), the ordinary search of S-0212 then targets
    *D*'s cell even without an order-level destination (claim 16). That destination is the old
    path's last cell, the one the vehicle is reaching, not further cells of the old path.

    **Deploying into a building.** While the hover is still moving in the sense of Terms, it
    makes no deploy attempt (S-0415 claim 2's moving rule). Readiness (S-0215 claim 11) keeps
    *unload* queued while the hover is moving now under *move*, so the moving rule itself is
    reached only when *unload* becomes current while a point remains, by the cases of the
    in-place paragraph below (a targetless attack, a tick with throttle 0) or under balloon-hover
    retention. Once *D* and *P* are both absent, it is no longer moving, whatever its
    remaining throttle. The deploy attempt is then made at the cell where it stopped, with the
    deploy anchor and cell test of S-0415 claim 3 taken there.

    **Deploying in place (S-0517).** The in-place cycle itself has no moving test, but its first
    deploy step comes only once *unload* is current. The deploy order only queues *unload*
    (S-0213 claim 10), and it becomes current at a per-tick start of S-0215 claim 11 (the first
    start, or the late start) at which the vehicle is ready. A hover is held back while it is
    moving now (Terms): it has a destination or heading point and its throttle is not 0. The
    first start reads the throttle left by the previous tick's movement, the late start that of
    the same tick. Outside the exceptions of S-0215 claim 11 (next paragraph), *unload*
    therefore becomes current, with *move* current, only:
    - when the movement of the table has ended, with neither point left. Nothing then remains to
      move. This is the usual case.
    - on a tick in which the hover still has its heading point but its throttle is 0 (claim 2).
      For a hover that is not switched off, that is a tick whose movement heading is more than
      45° from the direction to its heading point, for example just after a sharp corner. The
      hover still glides on its decaying speed (claim 3) and goes on with its movement.

    S-0215 claim 11's exceptions make the hover ready even while it moves now. The one that a
    deploy order reaches in ordinary play is *attack* without a target: the order clears the
    target of a hover sent to attack (S-0213 claim 10), so *unload* becomes current at the next
    first start, while the vehicle still moves. A current *guard* does the same.

    For a hover, the height above the ground that S-0517 claims 4 and 5 read is expected to be
    its hover height (claim 8), which is part of its position; H16 part j tests this. Within a vehicle's turn its
    due behaviour processing comes before its ordinary movement (S-0203, through S-0516 claim 42
    and S-0614 claim 18), so a deploy step reads the hover height left by claim 8's update in the vehicle's previous
    turn. That height is usually above 0 while the vehicle is powered, but it is 0 after
    an update that clamps it on a climb (claim 8). A powered hover at rest hovers above 0, so a
    first step made after its movement has ended usually reads a height above 0. The outcome is set
    by the height that the first deploy step reads:
    - **Height 0.** S-0517 claim 4 applies as on the ground: the vehicle becomes deployed in that
      step, or starts its deploying presentation and becomes deployed when that presentation
      completes, whatever its hover height in the meantime. This holds for both landing-first
      types and other types.
    - **Above 0, type not landing-first.** The step does nothing, and *guard* replaces the order
      (S-0517 claims 3 and 5). With S-0517's `quirks.airborne_deploy_order_dropped` at its
      default, the order is dropped, and a later height of 0 does not deploy the vehicle. With
      that switch off, the order is kept until the vehicle is on the ground, and the first deploy
      step that reads a height of 0 deploys it as S-0517 claim 4 gives.
    - **Above 0, landing-first type.** The step makes the landing request. While it stands, a
      deploy step does not deploy the vehicle even at height 0, a refinement of S-0517 claim 4's
      height-0 rows (listed in the Summary), `unmeasured` and rated medium (reason below). No rule is known by which hovering movement acts on
      that request or ends it, so the request is expected to stand and the vehicle to stay
      undeployed: *unload* stays current, and its deploy step is repeated each tick without
      deploying. That expectation is rated low (below). It is the default of
      `quirks.hover_landing_request_stands` (Edge cases and quirks).

    A hover that becomes deployed while it still moves is therefore the combination of *unload*
    becoming current while it moves (an exception, or a tick with throttle 0) with a first
    deploy step that reads a height of 0, as on a climb. After an ordinary move order it is rare.

    The deploy step does not stop the vehicle in any case. Its movement after the order is the
    table above, or the retention cases, whether the step comes before or after it stops.
    Becoming deployed stops nothing either: no rule clears a hover's heading point, *D* or
    stored path when it becomes deployed, and hovering movement does not test the deployed
    state. So a hover that becomes deployed in place at height 0 while still moving (the
    combination above) finishes, deployed, the movement it holds, and then stands, the default of
    `quirks.hover_deployed_keeps_moving` (Edge cases and quirks). This is an exception to S-0517 claim 9's
    sentence that a deployed vehicle cannot move under its own power (the correction is listed
    in the Summary). S-0517 claim 9's other rule still holds: the deployed vehicle cannot be
    given a new destination, so the movement it holds is its last until it undeploys. The
    exception covers the points the hover holds (*D*, stored path, *P*). It does not cover a
    later search of S-0212 for a retained equal *D* not completed on arrival (claim 18's height
    case): whether S-0517 claim 9's replacement of a deployed vehicle's destinations reaches
    that search is not stated here (Open questions).
    - Backing: `measurable` (H16: parts a to f for the table and the deploy attempt, i for the
      balloon-hover retention, j for when *unload* becomes
      current (run 1 the wait under *move*, run 2 the targetless-attack exception, run 3 a tick
      with throttle 0), for the in-place outcomes at height 0 and above 0 and for the turn order
      that makes the step read the previous turn's height). The suppressed assignment is
      `unmeasured`: an order-level destination is emptied by the same point-equality clear, which
      changes nothing when it is repeated on unchanged points, so a suppressed assignment and an
      effective one leave the same points. No part of H16 tells them apart: parts f and h meet an
      empty order-level destination on unchanged points and check only that the sequence ends
      as stated. The one known change of *D* in between is the search of S-0212 for a retained
      *D* not completed on arrival (claim 18's height case), for which no stock setup is
      settled; that case is open.
      The no-heading-point row is `unmeasured`: no set-up is known that holds the state when
      the order takes effect (H16 part g is an attempt). The refusal to
      deploy while a landing request stands is `unmeasured`: no run is known that gives a later
      deploy step a height of 0 while the request stands (H16 part j records such a step if one
      occurs).
    - Confidence: high for the point-equality rule as claims 10 and 16 give it (which points are
      kept or dropped); medium for its carry-over to the deploy order's clear, until H16 runs; medium
      for the deploy step's height test, which rests on claim 8's hover height; medium for the
      step's refusal to deploy while a landing
      request stands, since no run is known to give a later step a height of 0 while the
      request stands; medium for where the vehicle stops and the absence of resumed movement
      (both rest on claim 7's final step, which is read, not observed), the arrival-tick case,
      the no-heading-point row, the suppressed assignment, the balloon-hover retention, when
      *unload* becomes current for a moving hover (the throttle-0 tick and the targetless-attack
      exception), the turn order that makes the step read the previous turn's height, and a deployed hover's
      continued movement (no rule is known that stops it; H16 part j would show a stop); low for
      whether a landing-first type's landing request ever ends while it hovers, and so for that
      type never deploying once its first step has found it above the ground

## Formulas

Every floating-point operation in this spec runs in the original's program-wide round-toward-zero
mode (glossary), as in S-0205 claim 5: results are rounded to double precision, toward zero.
S-0205 owns that mode and its switch `quirks.chop_rounding`. Every conversion to an integer
truncates toward zero. Integer positions and distances are in leptons, tick numbers and object
numbers are integers, and R(*a*, *b*) denotes one ranged draw on the inclusive integer interval
*a* … *b* (S-0202 claim 4). Sine and cosine below use table trigonometry (S-0205 claim 6),
not exact analytic trigonometry; *r* is the table
square root of S-0204 claim 5. Facings use the units of S-0204 claim 10.

**F1 Throttle** (claim 2). β = S-0204 claim 8's direction from the vehicle's position to *P*.
δ = S-0204 claim 11's signed difference between η's current value and β.
> *T* = 0 if the vehicle is not powered or |δ| > 8192.
> Otherwise, with a destination: *T* = ½ if trunc(*r*(Δ*x*² + Δ*y*² + Δ*z*²)) < 256, measured to *D*,
> else *T* = 1. Without a destination: the same test measured to *P*.
Integers in leptons; the squares are summed exactly (double precision, exact at map scale).

**F2 Hover speed** (claim 3). Let *G* = min(1, *B* × *T*). The acceleration-adjusted value is
φ₁ = min(*G*, φ + 1 / (*a* × 900)) when φ < *G*, and φ₁ = φ otherwise.
The resulting hover speed is φ′ = max(0, φ₁ − 1 / (*b* × 900)) when φ₁ > *G*, and
φ′ = φ₁ otherwise. All quantities are double precision. Each denominator product is formed
before division, with rounding as stated above. The braking floor is 0, not *G*.
Unit: fraction of *s*.

**F3 Step length** (claim 4). *d* = trunc(*s* × φ), *s* the integer of S-0207 claim 2, φ double. Unit:
leptons.

**F4 Position step** (claim 6). Let η̂ = η for η ≤ 32767 and η̂ = η − 65536 otherwise,
exact integer arithmetic. Then θ = (η̂ − 16383) × *c*₂ radians, with *c*₂ = −2π / 65534,
the same double constant as S-0208 claim 6's planar step; η is the movement heading's current value.
> *x*′ = trunc(*x* + *d* × cos(θ)), *y*′ = trunc(*y* − *d* × sin(θ)), *z*′ = *z*.
The sum is formed in floating point before the truncation (not *x* + trunc(*d* × cos(θ))).
This follows the shared `quirks.table_direction` of S-0204 claim 8. With it off, use
θ = (η̂ − 16384) × (−2π / 65536) instead; all other operations stay the same.
Table-trigonometry behaviour, including negative angles, is inherited through S-0205 claim 6.

**F5 Arrival test** (claim 7). Arrived when trunc(*r*(Δ*x*² + Δ*y*²)) ≤ *d*, Δ from the vehicle to
*P*, map plane only. Integers in leptons.

**F6 Height and bob** (claims 8 and 10). *h* is the integer hover height in leptons (Terms),
*N* the integer object number, and *t* the integer tick number. The imported hover height *H*ₕ
and gravity *g* are integers, in leptons and leptons per tick per tick respectively. Vertical
velocity *v*, the bob field *q* (game minutes), damping factor *k*, period *P*ᵈ and factor *m*
are double precision. Heights *A*, *h*ₜ and the resulting height are integer leptons.

Define the effective height *h*ₜ to be *h* − *H*ₕ when the vehicle has a stored step and the
surface height one neighbour step ahead in that step's direction exceeds the surface height at
its position; otherwise *h*ₜ = *h*. Let *A* = trunc(*h* + *v*), *m* = 1 for odd *N* and 1.1
for even *N*. The constants 1.1 and 2π mean, respectively, the binary64 numbers nearest
exact decimal 1.1 and exact mathematical 2π, chosen by round-to-nearest, ties-to-even,
not single-precision values widened to double and not constants rounded towards zero.
Their decimal round-trip representations are 1.1000000000000001 and 6.283185307179586.
The period is *P*ᵈ = (*m* × *q*) × 900 ticks, rounding each product as specified above,
and *P*ᵢ = trunc(*P*ᵈ), a signed 32-bit integer.
Let ϱ = wrap₃₂(*t* + 2*N*) mod *P*ᵢ, using signed 32-bit integer arithmetic with
wraparound and remainder with the sign of the dividend. Here wrap₃₂ reduces modulo 2³²
and interprets the result in −2³¹ … 2³¹ − 1.
The sine argument is α = (ϱ × 2π) / *P*ᵈ: multiply before dividing, rounding after each
operation, not ϱ × (2π / *P*ᵈ). The unclamped height is *u* = trunc(2 × sin(α) + *A*).
The resulting hover height is max(0, *u*). When *u* < 0, the velocity used for the following
adjustments is 0; otherwise it is the old *v*.

The velocity gains ((2*H*ₕ − *h*ₜ) / *H*ₕ) × *g* when *h*ₜ < *H*ₕ and the vehicle is not
switched off. It also gains trunc(*g* / 3) when both *h*ₜ < *H*ₕ and
*h*ₜ < trunc(*H*ₕ / 4), whether switched off or not. These gains apply in the order stated to
the velocity selected by the clamp rule. Subtracting *g* from that adjusted velocity and then
multiplying by *k* gives the new vertical velocity.

The lift term is formed in double precision in the order shown; *g* / 3 and *H*ₕ / 4 are integer
divisions truncated toward zero. The world height becomes the surface height plus the resulting
hover height (plus 416 leptons on a deck). The sine argument uses the double *P*ᵈ; the remainder
uses the integer *P*ᵢ. Sine is the table sine defined above.

**F7 Spin** (claim 10). On each powered tick of an active spin, the body facing increases by
256*n* facing units, modulo 65536. The integer spin count for the next tick is *n* − sign(*n*).
The spin ends when the count reaches 0 or the vehicle is no longer powered.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Ticks per minute of the hover fields | 900 | ticks | F2, F6 |
| Throttle heading tolerance | 8192 | facing units (45°) | F1 |
| Half-throttle distance | 256 | leptons | F1 |
| Bob amplitude factor | 2 | leptons per tick | F6 |
| Bob period factors | 1 (odd object number), 1.1 (even) | factor | F6 |
| Spin facing step | 256 | facing units per count | F7 |
| Spin count draw | R(20, 30), sign by R(0, 99) < 50 | count | claim 10 |
| Wake period | 10 | ticks | claim 9 |
| Sinking height limit | 104 | leptons | claim 11 |
| Depot's repeated switch-off distance | see S-0314 claim 21 | leptons | S-0314's constant, not this spec's: a depot's settling processing switches a hovering vehicle off again only within that distance of its docking point; claim 10 |
| Deck-layer threshold of the new-path look | 312 | leptons | claim 12 |
| Arrival height tolerance at the destination cell | 208 | leptons | claim 7 |
| Absent-key defaults | see Import mapping | | engine defaults only |

## State and ordering

**A hovering vehicle's tick**, as observable results:

| State | Event | Result |
|---|---|---|
| not moving | tick | no travel; height (claim 8) and spin (claim 10) still update |
| moving, has heading point | tick | arrival uses the distance from this tick's updated throttle and hover speed; no snap is implied (claims 2–3, 6–7) |
| moving, no heading point after assignment or a previous arrival/refusal | tick | ρ = 1; search may give a heading point; any travel uses the retained hover speed, without that tick's ramp (claims 3–5). This is not a mandatory delay before the immediate first choice of claim 5 |
| moving now, over water, tick divisible by 10 | after travel | one wake (claim 9) |
| switched off, spinning | settles to hover height 0 on this tick | spin ends without another rotation; claim 11 decides the water fall |
| powered, not *sleep* or *enter* | switched off | two draws and a new spin, whose count replaces that of any active spin; stop request and loss of lift occur regardless of these spin conditions (claim 10) |
| spinning | *n* = 0 or no longer powered | spin ends; over water below 104 leptons, sinks (claim 11) |
| switched off | switched on (re-enabling, or the end of being settled at a repair depot) | lift resumes; an existing spin is not cancelled (claims 8 and 10) |

The wake's position is the position after travel, before the height change. If the height change
makes a settling vehicle no longer powered, its spin ends on that tick without another rotation.
Otherwise an active spin's facing change is visible in that tick's final facing.
These consequences inherit the backing and confidence of claims 8–10 (H6 and H7).

## Edge cases and quirks

- **Object number in the bob.** The bob's period choice and phase depend on the original's
  scenario-wide object creation number. As for the projectile wobble (lead decision of
  2026-10-10), our engine uses its own deterministic creation counter, so which vehicles bob with
  the longer period and their phases are a documented departure, with no switch. The bob formula,
  period and amplitude stay exact. Its effect on play is at most a few leptons of height.
- **Zero bob period.** When trunc(*m* × *q* × 900) = 0 (a hover bob field below about 1/900), the
  original divides by zero and the program fails. Our engine treats that period as 1 tick (the
  remainder is 0 and the bob term is 0) and the importer warns. Documented departure.
  Backing: `unmeasured`, confidence high for the failure condition: no stock-only scenario below
  supplies a zero period; the safe replacement is a design rule.
- **`quirks.hover_brake_undershoot`.** Proposed default original: F2 floors a braking step at 0,
  allowing it to cross below a positive goal. Off: floor that braking step at the goal instead.
- **`quirks.hover_elevated_heading_point`.** Proposed default original: a next heading point gains
  416 leptons whenever the vehicle is at least 312 leptons above that heading point's terrain surface, whether
  or not that cell has a high bridge. Off: require a high-bridge cell too. This changes the
  three-dimensional throttle-distance test, not the cell's actual terrain height.
- **`quirks.hover_stalled_start_reset` (default: on, the original).** When the step length is 0
  while the movement heading points exactly at the heading point, the vehicle sets *T* = 1 and
  ρ = 1 at once and recomputes *d* on that tick, so a fraction left at 0 by another rule does
  not hold it still. This changes throttle and fraction, not hover speed directly; a positive
  step still requires positive hover speed. Off: the vehicle does not apply this stalled-start
  reset or recompute the step length on that tick.
- **Weather switch-off.** The original also contains a switch-off of hovering vehicles that follows
  a house-wide weather condition. It never triggers with the stock program, so our engine has none.
  Backing: `unmeasured`, confidence medium: no established stock input activates that condition,
  so ordinary disabling cannot test this absence.
- **Pushing.** The original's hovering locomotor can be pushed one cell aside, with or without a
  spin. No stock caller of that was found (Open questions), so it is not specified.
- **`quirks.hover_idle_arrival_reservation`.** Default original: clearing the last heading
  point at a no-choice/no-destination arrival does not release its old reservation (claim 15).
  Off: release that reservation and mark the actual position instead. This changes flags only,
  not the absence of a cell-arrival update. Backing: `measurable` (H15), confidence high.
- **`quirks.hover_flag_clear_height_only`.** Default original: flag clearing chooses deck by
  height alone, while setting also requires a high bridge (claim 13). Off: both require the
  high bridge, avoiding a mismatched ground mark and deck clear at elevated non-bridge points.
  Backing: `measurable` (H15's named elevated-point test), confidence high.
- **`quirks.hover_landing_request_stands`.** Default original: a landing-first in-place hover
  whose first deploy step found it above the ground keeps its landing request while it hovers,
  so it never deploys; *unload* stays current with a deploy step each tick (claim 19). Off: a
  hovering vehicle treats a standing landing request as met once it is no longer moving (Terms),
  and its next deploy step deploys it as S-0517 claim 4's height-0 rows give, whatever its hover
  height. Backing: `measurable` (H16 part j, variant C) for never deploying, confidence low, as
  claim 19 rates it; `unmeasured` for the refusal at a later height of 0 while the request
  stands, confidence medium (claim 19 gives the reason).
- **`quirks.hover_deployed_keeps_moving`.** Default original: a hover that becomes deployed in
  place while it still holds movement finishes that movement deployed and then stands (claim 19),
  an exception to S-0517 claim 9. Off: becoming deployed drops *D* and the stored path as the
  table's unequal row of claim 19 does; the hover finishes only the leg to its current heading
  point (with claim 7's final step on arrival), then stands. Backing: `measurable` (H16 part j, run 2), confidence
  medium.
- **Tunnels.** A stored step into a tunnel entrance is handled by the tunnel rules (S-0212 claim 9), not
  by this spec.

## Import mapping

| Key | Feeds | Parse form | Absent-key default |
|---|---|---|---|
| `[General]` `HoverHeight` | *H*ₕ (claim 8, F6) | integer, leptons | engine default 120 |
| `[General]` `HoverBob` | *q* (F6) | decimal (S-0003 claim 6), single precision widened to double; a `%` sign scales by 0.01 | engine default 30.0, exact binary64 |
| `[General]` `HoverBoost` | boost factor (claim 3) | decimal, as above | binary64 nearest exact decimal 1.3 |
| `[General]` `HoverAcceleration` | *a* (F2) | decimal, as above | binary64 nearest exact decimal 0.03 |
| `[General]` `HoverBrake` | *b* (F2) | decimal, as above | binary64 nearest exact decimal 0.03 |
| `[General]` `HoverDampen` | *k* (F6) | decimal, as above | binary64 nearest exact decimal 0.8 |
| `[AudioVisual]` `Gravity` | *g* (F6) | integer, leptons per tick per tick | engine default 3 |
| `[General]` `Wake` | the wake animation (claim 9) | animation name | none: no wake |
| type key `Locomotor` | hovering locomotor (S-0207 claim 14) | identifier | S-0207 |
| type key `SpeedType` | locomotion class, for example amphibious (S-0211 claim 3) | name | S-0211 |

The floating absent-key defaults above are chosen by round-to-nearest, ties-to-even to
binary64, independently of the program's arithmetic rounding mode. They do not pass through
the single-precision text conversion. Giving a key the displayed default decimal text can
therefore differ from omitting it. This precision is specific to these defaults, not a new
parsing rule for S-0003 claim 6. The path-retry field used by claims 5, 12 and 15 is imported by
S-0212 claim 21 and its Import mapping; it introduces no additional hover key.

| Input owned elsewhere | Feeds here | Parse form and absent-key default |
|---|---|---|
| warhead `Paralyzes` | held timer tested in claims 5 and 17 | integer ticks, default 0; S-0614 claims 7–8 and Import mapping own its renewal |
| runtime stop order, destination and contact | claims 15–16 and 18 | runtime state, not an INI setting; no imported default |
| destination cell coordinates and terrain | supplied point in claim 18 | map geometry and terrain surface imported by S-0009 claim 7 and S-0106 claim 2; no separate held-destination key or default |

Each key keeps its value when a later layer omits it (S-0006). The engine defaults are those of the
program when no layer gives the key. The absent-key defaults and numeric import precision are
`unmeasured` here, confidence high: the stock-only scenarios below neither remove these keys
nor isolate the precision of their parsing. Parse-form and layering rules otherwise inherit
the cited import specs' backing and confidence.

## Measurement scenarios

Use stock data, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), game-speed index 2, 800 × 600 pixels, one human-controlled house and
one bystander computer player outside the observed test area under the glossary's void rule.
Use a purpose-made map with flat land, beach and water lanes, a bank with a traversable ramp,
a high bridge, and a stock control centre with dependent hovering vehicles. Use scripted input
and a read-only per-tick recorder where supported. Propose missing
recorder support to the user rather than changing the measurement setup. Record positions, heights,
surface heights, body and movement facings, destination, heading point, throttle, hover speed,
vertical velocity, speed fraction, remaining path directions, entry outcomes and animation events.
One valid run per subcase unless stated. No completed measurement is claimed.

- **H1 Land independence and amphibious drive.** (a) Order a stock hovering vehicle along an 8-cell
  straight run on clear land, then beach, then water, then across a shoreline. Compare plateau
  ticks with equal throttle, fraction, hover speed and direction. Expected: identical F3–F4
  displacements and no movement-model change at the shore. Endpoint timings must also account
  for F1's height-sensitive distance test. A differing plateau step or a change to tracks falsifies it.
  (b) Conditional: only if the stock data has a type with the amphibious class and the driving
  locomotor. Order it across the shoreline. Expected: it drives on tracks
  on both sides (cell-by-cell track pattern of S-0207), with its speed changing by its class's land
  multipliers (S-0207 claim 9). Falsified by straight-line hovering or a change of model.
- **H2 Ramp, heading tolerance and body aim.** Order a resting hover whose movement heading is
  south after a previous southward move to move north 8 cells, then run a 90° turn and
  an approach from full speed. Map placement with a southward body facing alone does not
  establish this starting movement heading (claim 15). Expected:
  F1–F4 each tick, with ordinary throttle 0 beyond 45°, half throttle strictly below the 3-D
  distance threshold, ramp and braking undershoot as F2, and body aim along the next stored
  direction rather than necessarily along movement heading. A differing throttle, increment,
  heading rate, body aim or position falsifies it. Also tests S-0206 claim 13.
  For boost, add a narrow detour beside the destination and retain a run where half throttle
  coincides with two equal remaining steps. Compare F2 with the imported boost and with boost 1.
  Only ticks where those predictions differ test boost. No qualifying tick is inconclusive.
- **H3 Straight steps, arrival and stop.** Order 1-, 2- and 5-cell moves, straight and with a 90°
  turn; also cross the high bridge. Record heading-point selection, position, occupancy and deck
  state on arrival ticks. Expected: F4 without snapping to the old point; a positive precomputed
  step can still move the vehicle when the point is cleared. A new point does not instantly
  redirect movement heading. Cell-layer transitions follow claim 6. Once not moving, no later
  travel or passive ramp occurs. Falsified by a snap, suppression of an otherwise qualifying
  final step, a different layer transition or a ramp while not moving.
- **H4 Height and bob.** Observe two resting hovers of opposite recorded object-number parity
  over land, then water, for 2000 ticks. From known starting heights and vertical velocities,
  predict every tick with F6, including phase and truncation. Expected: that exact trace and
  the two periods. A repeatable height or velocity mismatch falsifies it. Consecutive vehicles
  need not have consecutive object numbers if other objects were created between them.
- **H5 Climb.** Move a hover from water up the bank's ramp and back down, with a flat control
  run from matching vertical state. Expected: F6, with reduced effective height only on a rise,
  and no world-height change from horizontal travel before the vertical update. A trace requiring
  no anticipatory lift, lift on descent, or an extra height snap falsifies claim 8.
- **H6 Wake.** A hovering vehicle crossing water; record wake creation ticks. Expected: one wake on
  each tick divisible by 10 while moving now, none at rest or on the bridge deck, none on land.
  Its position must be the post-travel, pre-height-update position. A wake with different timing,
  conditions or position, or a missing qualifying wake, falsifies claim 9.
- **H7 Switch-off.** Put dependent hovers on land, flat water and bridge deck, including one
  moving. Destroy the last control centre and record spin, settling, residual travel and death.
  Rebuild it and record survivors rising. Record scenario-stream state across disabling.
  Expected: two ranged draws per qualifying vehicle, with count and sign matching the stream,
  F7 rotations until zero count or loss of powered height, and F6 without powered lift. On the
  first switched-off tick whose resulting hover height is 0, expect no further spin rotation,
  even if the previous tick's height was positive and the count was nonzero; another spin
  rotation on that settling tick falsifies the cutoff. The water
  hover gets the bomb mark and falls when its spin ends below 104 leptons, then has the low-water
  death effects of S-0312 claim 11. Land and deck hovers survive. A mismatched draw, rotation,
  height or survival/death under those conditions falsifies the corresponding claim.
  Conditional second run: interrupt and restore the centre's operation through ordinary power
  controls before the spin ends, where those controls permit it. Lift should resume without
  cancelling spin; premature cancellation falsifies that exception. No qualifying sequence is
  inconclusive, not confirmation. This setup tests no passengers.
- **H8 Factory exit.** Record a produced hover's exit, then an ordinary move. Record the
  first heading-point choice in the same tick as the exit instruction, before the factory's
  half-fraction setting, and record cell-arrival updates, contact, the clear report and door
  closing. Repeat a fresh ordinary move from rest. Expected: search and clear-point choice
  at assignment, not deferred to the next tick; each clear choice produces claim 15's update
  at the pre-travel position, including continued arrivals, with S-0417 claim 7 deciding whether
  it reports clear. Crossing a cell edge alone gives no update. A delayed first choice,
  absent qualifying update or boundary-only update falsifies the corresponding claim. Predict F3 from
  the actual recorded fraction rather than an approximate ratio of travel times. Expected:
  ρ = ½ persists only until a listed reset, including a stalled-start reset when step length is 0
  and movement heading exactly matches the heading-point direction. A fraction change without
  a listed reset, or failure to reset in that state, falsifies claim 4.
- **H9 Step checks.** (a) Block the next cell with an allied vehicle that has no orders: expected a
  scatter request after the fresh search, or a give-up within the close-enough distance (claim 12).
  (b) Put a wall across the path of a hovering vehicle on attack-capable orders: expected an attack
  on the wall on the repeated check. (c) Put a cloaked enemy unit (a stock type that can cloak) on the
  hovering vehicle's next cell when the path has no way around it. Expected: the cloaked unit is
  revealed, and the hovering vehicle keeps its destination and waits, retrying, where a driving
  vehicle in the same setup gives up its move (S-0207 claim 15).
  Add (d) a newly placed impassable obstacle giving *blocked*: expect immediate search and
  destination loss on repeated *blocked*. (e) A moving allied blocker: expect the blockage timer
  and mode-1/mode-2 re-paths of S-0212 claim 22; clear entry once it leaves. (f) A closed allied
  gate: expect opening and waiting, then clear entry. Introduce blockers after a path is chosen.
  Retain only traces that actually produce each intended outcome and, for repeated-check tests,
  a successful fresh search followed by that outcome. Failed searches are not repeated checks.
  Compare against T1, including zero throttle, hover speed and fraction on refusal. A differing
  search tick, destination retention, scatter, attack, gate response or refusal state falsifies
  the corresponding row. A different recorded outcome makes the intended subtest inconclusive.
  Also record the retry timer immediately after each immediate search, with success, failed
  search and failed reachability gate distinguished. Expected: *R* from the current tick
  in all three cases; zero delay after success falsifies claim 12. For failed searches on
  an arrival tick, include both within-one-cell and farther-destination cases (S-0212
  claim 23). Record positive *d* before arrival and the resulting position: expect F4's
  final cached step despite lost destination and heading point, unless a later refusing
  entry outcome or death/removal prevents it. No final step in an otherwise qualifying
  run falsifies claim 7. A failed search is not evidence of a repeated refusing check.
- **H10 Crate.** Put a stock crate in a traversable corridor and order a hover towards it. Record
  heading-point selection, crate disappearance/effect and physical arrival. Expected: collection
  on selection of that cell before physical arrival and its ordinary step check. Collection only
  at arrival, or no collection in a qualifying clear-entry run, falsifies claim 5. The crate's
  particular effect is outside this spec.

- **H11 Reservations and blocker chain.** Extend H3's straight and 90°-turn runs with
  a second stock walker and a vehicle ordered towards the hover's old cell, chosen cell
  and cells merely swung through on selected ticks. Record vehicle-present flags, listing,
  between-steps state and entry outcomes each tick, including heading-point selection,
  the first positive advance and each arrival. Expected: claim 13's reservation at choice,
  release before first positive travel rather than at boundary crossing, no reservation
  of swing-only cells, and between-steps state until positive advance with a point. A
  contrary flag/state trace falsifies that part. For the chain, use a narrow lane feeding
  a mode-0 path search into the hover while it is turning from rest, with ρ nonzero and
  φ and *d* zero. Separate its body-facing neighbour from its first remaining stored
  step's cell, leaving one empty and the other blocked. Expected: the chain chooses the
  body-facing cell despite zero hover speed; choosing by φ or last displacement falsifies
  claim 14. Reverse which cell is blocked for a second run. Propose search-cost and
  flag recording to the user if the existing recorder cannot expose them.
- **H12 Stop order.** Send a stock hover at full speed over flat land, then flat water,
  give an accepted stop order midway between points and repeat near a heading point.
  Record assignment-time and per-tick destination, path, heading point, throttle, φ, ρ,
  positions and arrival events. Expected: the order-level destination is cleared, and
  *D* and the stored path obey claim 16's equality rule, with the other movement quantities
  retained at assignment, followed by F1–F5 towards the retained point and
  the final cached step. No travel or ramp after both points disappear. Immediate
  heading-point removal, zeroed φ/ρ or exact-centre snapping falsifies claim 16.
- **H13 Placement heading.** Put two stock hovers on the map facing north and south,
  order both east eight cells on the same tick, and record η independently of body facing.
  Expected: initial η = 0 for both and matching heading/throttle histories for equal
  movement state. Repeat a factory-produced hover (H8): initial η remains north, not the
  factory's body-facing placement. Turn a hover away from north, stop it, teleport it
  through ordinary stock superweapon input, then order a new move. Expected: retained
  η and turn state across replacement with the same locomotor. A reset to body facing
  or north in this last case falsifies claim 15's retention. Record placement and motion
  separately so movement on the placement tick is not mistaken for an initial setting.

- **H14 Hold and assignment-time path update (Q-0675, Q-0679).** Use H1's map/settings and a named
  test-map variant with a hovering host that stock parasite attacks can hold; change only the
  host's locomotor, keeping the attack and its hold field unchanged. Compare held and expired-hold
  assignment with control-centre switch-off, and magnetic lift in separate runs. Record the
  hold countdown, powered state, both destinations, heading point, supplied height, path searches
  and retry timer at assignment. Expected: claim 17's timer predicate, including powered-but-held
  states; only held assignment omits immediate choice and destination-height adjustment.
  Switch-off alone does not set that timer; positive height can still make the hover powered.
  Magnetic lift has its replacement locomotor, not a held hover. EMP has no established stock
  skirmish activation here: if an ordinary stock input is identified, record it as an additional
  subcase, otherwise it is not an observed confirmation of the negative predicate claim.
  To isolate claim 5's moving-now/no-point branch, retain a trace where both *D* and throttle
  are nonzero but no point exists, then retarget while a positive retry delay runs, and again
  after it expires. Expected: immediate choice attempt in both cases, no search while gated,
  and a search with full delay restart only when eligible. A delayed choice, unconditional
  search through the running timer, or missing restart falsifies the corresponding rule.
  If this state is never reached, that subtest is inconclusive, not evidence of a mandatory
  tick delay.
  For claim 18, add raised flat ground reached by a ramp, a sloped destination cell, and a
  high-bridge destination. Retain an assignment that actually reaches the hover while held,
  recording the order-level reference and *D* immediately before and after assignment,
  at hold expiry, and at each subsequent search and point choice. A refused human move
  order does not establish such an assignment; if normal input cannot produce it in the
  named test-map variant, this subtest remains inconclusive. Expected: the new order-level
  reference is present immediately, and *D* is the cell centre at terrain surface height
  there, including slope height but excluding bridge lift. No hover choice, speed-fraction
  change or arrival update is caused by that held assignment alone. Release the parasite
  before host destruction and record movement, final position and both destination kinds.
  Expected: unchanged *D* through expiry/search/choice while it exists, independent
  heading-point height under claim 5, F1's distance to that retained *D*, and claim 7's
  actual height-sensitive completion rather than an assumed successful bridge arrival.
  Give the same cell again after expiry: expect the ordinary adjustment, including the
  bridge lift. A zero remembered height on raised terrain, a bridge lift during held
  assignment, a height repair without reassignment, a retained old order-level reference,
  or an assignment-only arrival update falsifies the corresponding part of claim 18.
  Where a contact/update trace exists, apply S-0417 claim 7's report test to the new
  order-level reference, not *D*. Do not infer reporting merely from assignment.
  One qualifying run per subcase; propose any missing read-only recorder support.
- **H15 Arrival outcome matrix (Q-0675).** Extend H3/H9/H12 with recording of both destination
  kinds, contact cell, hold timer, retry timer, ground/deck flags, between-steps state, searches
  and cell-arrival events. In separate runs retain: no-*D* arrival without contact; no-*D*
  arrival with same-cell contact; deferred search with *D* retained; failure in the initial
  arrival path update; failure during heading choice; and refused choice. Use a corridor
  whose only route can be blocked after a point is reserved. Distinguish event order in the
  read-only trace, not merely the final absence of a point. Expected: exactly claim 15's rows,
  including no replacement flag after early failure without same-cell contact, and actual-cell
  marking after deferred search or failure during choice. Account for a cached step transferring
  that actual mark at a boundary. A contrary restoration, old-cell flag, update or search timing
  falsifies the row. For no-*D*/no-contact stop, expect the former heading reservation to remain
  unless another event clears it. Stop also ends contact, so use a separate newly established
  contact trace rather than assuming a factory remains contacted after accepting stop.
  Repeat stop with full three-dimensional *D* = *P*: expected both local points finish and an
  update occurs despite the absent order-level destination; unequal points take the no-*D*
  case. A retained *D* must not suppress a report as a different order-level destination.
  On a high bridge and beneath it, compare marking and release layers with claim 13's height
  rule rather than listing layer. Add a named test-map variant giving sufficient hover height
  to isolate an elevated non-bridge point: setting marks ground, clearing at that height clears
  deck with the default quirk, ground with it off. A common listing-based rule or symmetric
  original clearing rule falsifies claim 13. Record one qualifying run per matrix row and layer
  case; no qualifying trace leaves the subcase unmeasured rather than confirming it.

- **H16 Deploy order while moving (Q-1773).** Use H1's flat open-ground map and settings, with
  four named test-map variants of one stock hovering vehicle type, each with a passenger
  capacity of 0 and no other unload rule, so that the deploy order reaches the deploy rules
  of claim 19. Variant A gives it the
  deploy-into-building field, naming a stock building whose footprint fits the open ground.
  Variant B gives it the in-place deploy field instead, without the deploy-to-land flag.
  Variant C is B with that flag, and variant D is A with the balloon-hover field (part i).
  Map-placed units only. With scripted input (mouse and keyboard only), order the hover to a
  cell eight cells away in a straight line and press the deploy key after two cells, (a) with
  the vehicle midway between two cell centres; (b) on the tick of its arrival at a cell
  centre (claim 15; claim 7 does not move it onto the exact point); (c) on the tick before that
  arrival; (d) on the final leg, midway towards the
  destination cell. (e) Order it to a single adjacent cell, so that the first leg is the final
  one, and press the deploy key midway. Parts (b) and (c) need the order's tick exact: repeat
  each with the key moved one tick earlier or later until the record shows the order taking
  effect in the stated tick; a run whose order falls elsewhere counts for the part it falls in.
  For variant A, repeat (a) with a stock unit
  standing in one of the building's footprint cells at the expected stopping cell, so that
  the attempt fails and *guard* is queued, and keep recording. Keep every other house's units
  out of the vehicle's guard range in all runs. For variant A, also repeat (a) with an attack
  order on part i's bystander building, placed as there, in place of the move order. (f) Give a
  second deploy order on the tick after the first in (a). It meets an empty order-level
  destination on unchanged points, the suppressed-assignment row of claim 19, and like part h it
  does not tell a suppressed assignment from an effective one.
  Record, every tick for up to 300 ticks: its cell and position, the order-level destination, *D*,
  the stored path length, the heading point, throttle and ρ (read-only recorder), the heights of
  *D* and *P*, its current and queued behaviour, and the deploy attempt or deploy step and its
  outcome. Expected: in (a) and (c)
  the vehicle stops on its arrival at the heading point it was reaching, plus at most claim 7's
  one final step, having dropped *D* and the path at the order; in
  (b) it goes one cell further; in (d) and (e) *D* is kept and the move completes at the
  destination. A run of (d) or (e) whose recorded *D* and *P* heights differ at the order tests
  the table's unequal row instead, and is expected to behave as (a). No tick shows throttle or ρ reset at the order. For variant A, the attempt
  is made only after both points are absent, at the stopping cell; after a failed attempt the
  vehicle stays in that cell for the rest of the 300 ticks, with no target in range. (f) shows no
  difference from (a). In variant A's attack repeat, *unload* becomes current at the first
  start after the order while the vehicle still moves, and no attempt is made until both points
  are absent. Under the plain move orders of parts a to f, the record shows *unload* queued
  under *move* while it shows the hover moving now, in every variant.
  For variant B, when the first deploy step reads a height above 0, *guard* then replaces the
  order without deploying, and the movement matches variant A's. For variant C, when that step
  reads a height above 0, *unload* then stays current with a deploy step each tick and no
  deploy while it hovers (the low-rated case of claim 19: whether the request ever ends while
  it hovers).
  (g) No heading point: a test map whose only route passes through a one-cell gap, with a
  stock vehicle of the same house standing in the gap. Order the hover through the gap, wait
  until the record shows *D* present and no heading point (it waits in the cell before the
  gap), and press the deploy key. The blocker may leave on the scatter request of table T1's
  stationary-friendly row; repeat with the key moved until the order takes effect while the
  record still shows *D* present and no heading point. A run in which it does not is void.
  Part g is an attempt: the row stays `unmeasured` (claim 19) unless a run reaches the state.
  Expected: *D* is cleared at the order and the vehicle does
  not leave its cell.
  (h) Suppressed assignment: on the final leg, midway, press the stop key, and press the deploy
  key on the next tick. Expected: the record shows an empty order-level destination before the
  deploy order and no change in *D*, path or heading point at it; the move completes at the
  destination, as in (d). An effective assignment would leave the same points here (*D* = *P*),
  so this part does not back the suppressed-assignment row, which stays `unmeasured` (claim 19).
  It checks only that the sequence ends as stated.
  (i) Balloon-hover retention, variant D: the map places a stock building of the bystander
  computer player in the lane, beyond the hover's range along the straight line; the bystander's
  units stay outside the observed area. Order the hover to attack that building, and press the deploy key after two cells, at a
  tick when the record shows a nonempty order-level destination, a target and one qualifying
  row of S-0502 claim 28's retention; a run whose record does not show all three before the
  deploy key is void. Expected: the order-level
  destination, *D* and path are unchanged at the order, the vehicle travels on until that move
  ends, and its deploy attempt is made only there. The order leaves *attack* without a target,
  so *unload* may become current before the move ends; the record then shows it current with
  no attempt while the vehicle moves (S-0415 claim 2). A run of the same type with a plain move
  order and no target behaves as (a).
  (j) In-place outcomes, variants B and C: a test map with a water strip and a bank that rises
  at least two height levels, and a stock building of the bystander placed well beyond the
  top of the bank, beyond the hover's range and guard range from every cell it reaches,
  including its stopping cell (as in part i).
  Record the hover height and throttle after each tick's update, the current and queued
  behaviour, and the tick of each deploy step.
  Run 1, plain move: order the hover across the strip and up the bank in a straight line, and
  press the deploy key in the climb. Expected: *unload* stays queued while the record shows
  the hover moving now, and becomes current only after the movement has ended. The outcome
  then follows the height that the first deploy step reads, with the same rows as run 2.
  Run 2, targetless attack: order the hover to attack that building, so that its route crosses
  the strip and climbs the bank, and press the deploy key so that the order's tick falls in a
  climb. Expected: *unload* becomes current at the next first start while the vehicle still
  moves. Repeat with the key pressed one tick earlier and later until a run's first deploy step
  follows a tick whose recorded hover height is 0. In that run both variants deploy in place in
  that step (or start the deploying presentation and complete it), and the deployed vehicle
  finishes the movement of the table and then stands. In runs whose first step follows a height
  above 0, variant B returns to *guard* and variant C stays in *unload* without deploying, even
  in a later tick at height 0.
  Run 3, throttle 0: on a flat test map, order the hover along an L-shaped route with one 90°
  turn, and press the deploy key so that the order takes effect on the tick of its arrival at
  the corner cell, or in a later tick while the record still shows throttle 0 with the heading
  point present; repeat with the key moved as for part b. Expected: *unload* is current no
  later than the first start after a tick whose record shows throttle 0 with the heading point
  present, while the vehicle still moves; the outcome then follows the height read, which on
  flat ground is above 0.
  Only a run whose vehicle deploys while it still moves tests a deployed hover's continued
  movement; a run that deploys only after its movement has ended is inconclusive for that row.
  In every part, the movement after the order follows the table whether or not the vehicle has
  deployed.
  Falsified by, per part:
  - Every part except (g): an immediate halt at the order (throttle, hover speed or ρ reset). In
    (g) no heading point is present, and the expected stop in the vehicle's cell is not such a
    halt; there a reset of throttle or ρ at the order falsifies.
  - (a) to (g): travel past the heading point current at the end of the order's tick beyond the
    one already-computed final step that claim 7 allows on the arrival tick at that point; any
    later travel along the old path. For variant A, an attempt while a point remains, or travel
    after a failed attempt.
  - (f): any difference from (a).
  - (g): *D* kept at the order, or the vehicle leaving its cell.
  - (h): a changed *D*, path or heading point at the order, or a stop before the destination.
  - (i): a changed order-level destination, *D* or path at the order, a stop before that move
    ends, or a deploy attempt before it ends.
  - (j): travel past the heading point as in parts a to g; an in-place deploy on a first step that
    follows a hover height above 0; no deploy on a first step that follows a height of 0 (for
    this height test, starting the deploying presentation in that step counts as a deploy); a
    deploy in variant C at a later step that follows a height of 0 while the landing request
    stands; a deploy step in a tick under a plain move although the record after the previous
    tick shows the hover moving now with *move* current and *unload* queued (runs 1 and 3); no
    deploy step at the first start after the order (run 2); *unload* still queued at the first
    start after a recorded throttle 0 with the heading point present (run 3); a deployed hover
    that stops before its held movement ends (only a run whose record shows the deployed state
    while the vehicle still moves counts for this).
  One qualifying run per case; a part without a qualifying run (in particular (j), whose
  timing is found by repetition) leaves its row unmeasured rather than confirming it.

## Open questions

- **Pushing.** A push of a hovering vehicle one cell aside, with an optional spin, exists in the
  original's locomotor, but no stock caller was found by the search made. If one exists, the pushed
  vehicle turns its movement heading at once and moves at full speed. Medium confidence that it is
  unused.
- **Arrival and stopping.** H3 must isolate the final same-tick step from subsequent behaviour
  processing. No measurement has yet confirmed its size or whether the point is overshot.
- **First cell of a new path.** The deck threshold of 312 leptons and the missing gate row (claim 12)
  are read, not observed; low confidence.
- **Boost and exceptional disabling.** H2 still needs a qualifying half-throttle detour, and H7
  still needs an ordinary-input early-restoration sequence. The *sleep*/*enter* and equal-point
  stop exceptions also lack isolated stock setups. These remain unmeasured, not disproved.
- **Landing-first hover deployer (claim 19, Q-1773).** Hovering movement is not expected to act on the
  landing request, so a landing-first in-place hover whose first deploy step finds it above
  the ground is expected never to deploy. The combination needs a test type; H16's variant C
  runs are the only check. The default of `quirks.hover_landing_request_stands`. Low confidence.
- **Deployed hover still moving (claim 19, Q-1773).** No rule is known that stops a hover's held
  movement when it becomes deployed in place, so a hover that deploys at height 0 mid-move is
  expected to finish that movement deployed, an exception to S-0517 claim 9 (listed in the
  Summary). It needs *unload* to become current while the hover still moves (claim 19), which
  H16 part j's targetless attack provides. An unknown stop would show there. The default of
  `quirks.hover_deployed_keeps_moving`. Medium confidence.
- **Deployed hover and a later search (claim 19, Q-1773).** Whether S-0517 claim 9's replacement of a
  deployed vehicle's destinations applies to the search of S-0212 for a retained equal *D* not
  completed on arrival (claim 18's height case) is not known. No stock setup reaches that case
  for a deployed hover. Open.
- **Deploy order on changed points (claim 19, Q-1773).** When the order-level destination is already
  empty but *D* or *P* has changed since the clear that emptied it (the one known way is the
  search of S-0212 for a retained *D* not completed on arrival, claim 18's height case),
  whether the deploy order's clear acts on the changed points, and so whether a suppressed
  assignment and an effective one differ, is not known. No stock setup reaches the case. Open.
