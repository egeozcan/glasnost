---
id: S-0207
title: Ground vehicle movement (drive locomotor)
topic: movement
revision: 4
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0207: Ground vehicle movement (drive locomotor)

## Summary

How a vehicle with the driving locomotor turns its speed field, speed fraction and terrain into
progress along fixed cell-to-cell tracks (straight, diagonal and two-cell curves), and where it stands
between track points. Where it may go (S-0211), which steps it takes (S-0212), how fast it turns
(S-0206) and every other locomotor, hovering vehicles and ships included, are outside this spec.

## Terms

*Tick*, *lepton*, *cell* (with its centre), *facing* (with map north, map east and the compound
directions), *height level*, *bridge deck*, *high bridge*, *locomotor*, *movement zone*, *field*,
*absent-key default*, *rules layer*, *country*, *engine constant* and *importer warning* are used
as in the glossary. *Land type*, *locomotion class* and *land multiplier* *M*(*ℓ*, *c*) are as in
S-0211. The *8-direction*, its facing 8192 × *d* and its *neighbour step* are as in S-0204 (Terms
and claim 9): 8-direction 0 is map north and each next one is 1/8 turn further clockwise, so N, NE,
E, SE, S, SW, W and NW are 0 to 7. The *table square root* *r*(*n*) is S-0204's (Formulas F1). The
*surface height* *H* is S-0106's (Terms and claim 2). The *crush-slowdown state* is S-0210's
(claim 9). *Crushable overlay* and the movement zone *crusher-all* are used as in the crushing
rules of S-0210.

- **Drive vehicle.** A vehicle whose locomotor is the driving locomotor (wheeled and tracked ground
  vehicles in stock data). In this spec "vehicle" means a drive vehicle.
- **Step.** A move of one cell in an 8-direction, by S-0204's neighbour step: N decreases cell y,
  E increases cell x, and a diagonal step changes both. S-0212 calls the 8-direction of a step its
  *step direction*. In this spec "the direction of a step" always means its 8-direction.
- **Path steps.** The list of 8-directions the pathfinder gives the vehicle, first step first.
  S-0212 owns how it is made and when it is renewed. This spec only reads the first two entries,
  the *current step* and the *next step*.
- **Type speed** *V*. An integer from 0 to 255, derived from the type's speed field (claim 1). Unit:
  progress units per tick.
- **Progress unit.** The unit in which a vehicle's progress along a track is counted. Passing one track
  point costs 7 progress units (claim 3). Track points are about 11 leptons apart, so a progress unit
  is roughly 11/7 of a lepton of travel, but the exact relation is given by the tracks (claim 3), not
  by a lepton conversion.
- **Progress budget** *A*. An integer that the vehicle keeps between ticks: progress it has earned
  but not yet spent on track points.
- **Speed fraction** *ρ*. A double-precision number from 0 to 1 that scales the vehicle's per-tick
  progress. New vehicles start with *ρ* = 0.
- **Target fraction** *ρ*ₜ. The double-precision speed fraction that the vehicle is accelerating
  or braking towards (claims 9 and 10).
- **Speed bonus factor** *b*. A double-precision factor kept by each vehicle. It starts at 1.0 and is
  changed only by the speed power-up (claim 13).
- **Country speed factor** *κ*. The owner's country's speed factor for vehicles, a single-precision
  field (Import mapping).
- **Veteran speed factor** *ν*. A game-wide double-precision field. It applies to a vehicle that has
  the *faster* veterancy ability at its current veterancy level. Which ability a veterancy level grants
  belongs to the veterancy spec.
- **Track.** A fixed list of points that a vehicle drives through to reach a **track target**, which
  is the centre of the cell one step ahead (straight and diagonal tracks) or two steps ahead
  (two-cell curves). Each point is a displacement (*x*, *y*) in leptons from the track target along
  the map axes, with a facing *f* in 1/256 of a turn. The vehicle's body faces 256 *f* facing units
  while at that point. A track ends at the track target itself. The tracks are engine constants
  (Constants, tables T1 and T2). This *track* is unrelated to S-0211's locomotion class *track*,
  which this spec always names with the words "locomotion class".
- **Move destination.** The point the vehicle's current move order leads to, in leptons. It is used
  only for braking (claim 10). When the destination cell is a high-bridge cell, its height is the
  deck's height.
- **Slowdown distance** *D*ₛ, **acceleration factor** *a*, **deceleration factor** *η* and the
  **acceleration flag.** Fields of the vehicle's type (Import mapping), in leptons (*D*ₛ) and as plain
  numbers (*a*, *η*).
- **Passive flag.** A vehicle-type field. In stock data no vehicle type sets it. It changes two rules
  (claims 8 and 10).
- **Moving tick.** A tick in which the vehicle is on a track, or begins one, and is not prevented
  from moving (claims 6 and 12).
- **Sinking.** A state of a vehicle: it is going down in water. This spec only uses the state
  (claim 10, case 3). Which events set it belongs to no written spec yet (Open questions).
- **Following car.** A vehicle linked to another vehicle, its *leader*, as a car of the leader's
  *train*. This spec uses only the speed-fraction copy of claim 10. How the link is made and how a
  car is steered belong to no written spec yet (Open questions).

## Behaviour

### Speed

1. **The speed field.** The type speed is set from the type's speed field *v*, an integer
   (S-0003, claim 3), as follows:

   > *V* = min(⌊256 × clamp(*v*, 0, 100) / 100⌋, 255)

   So *v* = 100 gives 255, not 256, *v* = 1 gives 2, and any value above 100 acts as 100. A value
   below −1 gives 0. A value of −1 is treated like a missing key: the type speed keeps the value it
   already has (from an earlier rules layer, or the absent-key default 0). A type whose speed is 0 never moves. Our importer reproduces this clamping and gives
   an importer warning for a value above 100 or below −1, which the original accepts silently.
   - Backing: `measurable` (scenarios D1 and D2)
   - Confidence: high

2. **Progress per tick.** In each moving tick the vehicle earns *s* progress units:

   > *s*₁ = trunc(*V* × *κ* × *b*)
   > *s*₂ = trunc(*s*₁ × *ν*) if the vehicle has the faster ability, otherwise *s*₁
   > *s* = trunc(*s*₂ × *ρ*)

   Here *ρ* is the speed fraction after this tick's update (claim 10). A vehicle that carries a flag in a
   capture-the-flag game earns trunc(*s* / 2) instead. Every product is formed in floating point
   (Formulas), and trunc truncates toward zero.
   - Backing: `measurable` (scenarios D1, D6 and D12; the capture-the-flag halving by scenario D15,
     whose precondition, that a flag can be carried at all, is itself open)
   - Confidence: high; medium for the halving

3. **Spending progress on track points.** In each moving tick the vehicle adds *s* to its budget
   *A* and spends it on the points of its track at 7 progress units per point. A point is bought
   only with a budget above 7, so the vehicle buys *k* points and its budget falls by 7*k*, where
   *k* = 0 if *A* ≤ 7, and otherwise *k* is the largest whole number with *A* − 7*k* ≥ 1, but no
   more than the points still to pass on the track, its end (the track target) included. *A* = 7
   exactly does not buy a point.

   When the points bought reach the end of the track (the track target), the vehicle moves to the
   track target, and its budget *A* becomes the corrected budget

   > *A*′ = *A* + trunc(7 × (1 − *m* / 11))

   where *m* = |Δ*x*| + |Δ*y*| is the distance, in leptons along the map axes, from the last point
   to the track target. The correction is negative when *m* > 11. The leftover budget is kept.
   Whether the next track begins in the same tick, and how many tracks one tick can finish, is
   claim 12. The budget is set to 0 in every tick in which the vehicle turns in place (claim 6),
   and in every other tick in which it has a move to make but neither drives a track nor begins one. A vehicle that comes to rest keeps
   whatever leftover it had, and that leftover is added to its first tick of progress if its next
   move begins without a turn in place.

   So a track of *n* points whose last point is *m* leptons from its target costs exactly
   7 (*n* + 1) − trunc(7 (1 − *m*/11)) progress units. That is **163** for a straight step
   (curve A of T1) and **227** for a diagonal step (curve B). Over a long straight run at a
   steady *s* well below 163, a vehicle covers one cell per 163/*s* ticks on average. A diagonal run
   takes 227/*s* ticks per cell. Faster vehicles are limited by claim 12 to one track per tick.
   - Backing: `measurable` (scenario D1)
   - Confidence: high for the costs; low for the leftover kept at rest (the ways of arriving were
     read but not exercised)

4. **Position between points.** Whenever a vehicle's budget after this tick's spending (claim 3) is
   at least 1 and the next point *P* of its track is not the end, the vehicle is moved towards *P*.
   Its position at the end of the tick follows from its position *Q* after this tick's point
   purchases, which may itself be a position this rule gave it in an earlier tick, so the step
   compounds from tick to tick:

   > position = *Q*′ if *Q*′ lies in the cell of *Q* or in the cell of *P*,
   > position = *P* if *Q*′ lies in neither cell and *A* > 3,
   > position = *Q* if *Q*′ lies in neither cell and *A* ≤ 3.
   >
   > where *Q*′ = *Q* + (trunc((*P*ₓ − *Q*ₓ) × *t*), trunc((*P*ᵧ − *Q*ᵧ) × *t*)) and *t* = *A* × (1/7),
   > rounded toward zero to single precision (F4).

   Because *t* is always a little less than *A*/7 (F4), a displacement whose exact product would be
   a whole number comes out one lepton shorter; in particular *t* < 1 even when *A* = 7, so the
   vehicle does not reach *P* by this rule. For *s* = 2 on a straight track, the vehicle's
   displacements from its start over the first eight ticks are 3, 7, 10, 12, 16, 20, 21 and 25
   leptons, not 3, 6, 9, ….
   - Backing: `measurable` (scenario D8)
   - Confidence: high

5. **Height and facing on a track.** At each point the vehicle's ground position becomes the track
   target plus the point's displacement. Its height follows the terrain surface (surface height *H*
   of S-0106, claim 2). Its body facing becomes 256 *f*. Between points
   (claim 4) the facing does not change.
   - Backing: `measurable` (scenario D7 for the facing at the end of a curve)
   - Confidence: high for the ground position and the facing; medium that interpolation leaves the
     facing unchanged

### Starting a step

6. **Turn in place first.** A vehicle begins a track only when its body facing equals the current
   step's 8-direction exactly (8192 × *d* for 8-direction *d*). Otherwise it turns in place towards
   that facing, and makes no progress until the facing matches. The turn, its duration and the tick
   in which the track begins are S-0206's (claim 12). Its budget
   is held at 0 while it turns (claim 3). Its speed fraction is kept, so a vehicle that stops to turn resumes at the fraction it had.
   - Backing: `measurable` (scenarios D7 and D9)
   - Confidence: high

7. **Choice of track.** At the start of a step, the current step *d* and the next step *e* choose the
   track (table T2):
   - *e* = *d*, or no next step: the straight track (curve A, for N, E, S, W) or the diagonal track
     (curve B) to the centre of the next cell. One step is used.
   - *e* is 1/8 turn from *d*: a two-cell curve to the centre of the cell after the next. It is
     curve C when *d* is N, E, S or W, and curve F when *d* is diagonal. Both steps are used.
   - *e* is 1/4 turn from *d*: a two-cell curve. It is curve D when *d* is N, E, S or W, and curve E
     when *d* is diagonal. Both steps are used.
   - *e* is 3/8 or 1/2 turn from *d*: the straight or diagonal track for *d* alone. At the next
     cell's centre the vehicle must turn in place (claim 6) before it starts *e*.

   Each two-cell curve ends with the body facing *e*. If the step after the curve is not *e*, the
   vehicle therefore turns in place at the curve's target before going on (claim 8 gives the passive
   exception).

   **Crush mode.** Call the cell that the current step enters the *first cell*, and the cell one
   step beyond it, in 8-direction *e* (or in 8-direction *d* when there is no next step), the *second
   cell*. The vehicle drives straight instead of curving, and enters *crush mode* for this track,
   in either of these cases:
   - the second cell holds a crushable overlay, or, for a type of movement zone *crusher-all*, a
     wall overlay or a vehicle, or
   - the first cell, which the vehicle is free to enter, holds a crushable overlay, or, for a type
     of movement zone *crusher-all*, a wall overlay. A vehicle in the first cell does not count.

   In crush mode the next step is treated as equal to the current one: the straight or diagonal
   track for *d* is used and one step is consumed. Crush mode is used by the crushing rules of
   S-0210, among them the start of the crush-slowdown state (S-0210 claim 9), which claim 10 case 1
   reads.

   If the second cell of a two-cell curve cannot be entered at that moment, because it is occupied or
   being entered by another object, the vehicle takes the straight or diagonal track into the first
   cell instead. Other refusals lead to a stop or a new path, which the occupancy and pathfinding specs
   own.
   - Backing: `measurable` (scenarios D7, D9 and D10)
   - Confidence: high for the table; medium for crush mode (which cells are checked was read but not
     exercised); low for the fallback when the second cell is blocked

8. **No chaining of curves for ordinary vehicles.** Each two-cell curve has a *chain point* and
   names an *entry point* (table T1). A vehicle of a type with the passive flag set may switch,
   when it reaches the chain point, to the next pair's two-cell curve at that curve's entry point. It
   does so when the step after its curve turns again, that pair has a curve with an entry point,
   and the vehicle can enter the cell that step leads to. It then skips the centre of its current
   target, and the track target moves one cell on in the 8-direction of that step. A vehicle without the
   passive flag never does this: every curve runs to its target's centre. Stock data has no passive
   vehicle types, so in stock play no vehicle chains curves. On the tracks of this spec, the speed
   fraction of a passive vehicle whose acceleration flag is on is changed by none of the rules of
   claim 10 (none of its six cases applies). Such a vehicle keeps the fraction it has: a vehicle
   that starts at rest with *ρ* = 0 (claim 11) never moves, unless it is a following car of a train
   whose leader copies a fraction to it (claim 10), or a separate track of a building has given it a
   fraction (Open questions).

   **Quirk `quirks.drive_curve_chaining_passive_only`.** Default (original): only passive vehicles
   chain. Off: every vehicle chains as described.
   - Backing: `measurable` (scenario D9 for the default)
   - Confidence: high that ordinary vehicles never chain; medium for the details of chaining

### Speed fraction

9. **Target fraction at the start of each track.** When a vehicle begins a track (claim 7), its
   target fraction is set from the cell its current step enters. It depends on these parts:
   - (a) Land type *ℓ*. Let *λ* be the height level of the vehicle's current cell, plus 4 if the
     vehicle stands on a bridge deck. If the entered cell's height level differs from *λ* by less
     than 2, *ℓ* is that cell's land type. Otherwise *ℓ* is *road*, which covers a vehicle driving
     along a bridge deck.
   - (b) Land factor *g* = min(*M*(*ℓ*, *c*), 1) for the vehicle's locomotion class *c* (S-0211),
     widened to double.
   - (c) Slope factor *u*: the uphill factor if the surface height *H* (S-0106, claim 2) at the
     entered cell's centre is higher than *H* at the vehicle's position, the downhill factor if it is
     lower, and 1 if they are equal. A vehicle of locomotion class *track* (S-0211) uses the tracked
     factors, and every other locomotion class uses the wheeled factors.
   - (d) Zero fallback: where *g* × *u* = 0 exactly, 0.5 takes its place.
   - (e) Damage factor *h*: 0.75 if the vehicle's health ratio is at or below the yellow ratio
     (S-0108, Formulas F3), otherwise 1.

   > *ρ*ₜ = *h* × *z*, where *z* = *g* × *u* if *g* × *u* ≠ 0, and *z* = 0.5 if *g* × *u* = 0

   A two-cell curve takes its target from its first cell only, and the value stays in force until
   the next track begins. The damage factor also applies to the fallback, so a damaged vehicle whose
   multiplier is 0 gets 0.375.
   - Backing: `measurable` (scenarios D3, D5 and D6; S-0211 scenario L1)
   - Confidence: high for parts (b), (d) and (e); medium for parts (a) and (c) (the reference
     heights were read but not exercised)

10. **Updating the speed fraction.** The speed fraction is updated once in each moving tick, and
    that tick's progress *s* (claim 2) is earned at the updated fraction. At some track boundaries
    it is updated a second time (claim 12). Each update follows these rules:
    - **Acceleration flag off:** *ρ* = clamp(*ρ*ₜ, 0, 1).
    - **Acceleration flag on, vehicle not passive.** Let *d*ₘ = ⌊*r*(*n*)⌋ with
      *n* = Δ*x*² + Δ*y*² + Δ*z*², the distance in leptons from the vehicle to its move destination
      taken with S-0204's table square root (F5), not the exact root. From *n* = 16641 (129 leptons)
      on, *d*ₘ can be one lepton less than ⌊√*n*⌋ (S-0204 claim 5), so near the threshold the
      braking test *d*ₘ < *D*ₛ can hold one lepton earlier than with the exact root. The new
      fraction *ρ*′ is given by the first row of this table whose condition
      holds, and is then clamped to 0 to 1:

      | Case | Condition | New fraction *ρ*′ |
      |---|---|---|
      | 1. Crush slowdown | the vehicle is in the crush-slowdown state (S-0210 claim 9) | min(*ρ*ₜ, 0.2); the target fraction also takes this value |
      | 2. Braking | *d*ₘ < *D*ₛ | max(*ρ* − *V* × *η*, *F*₀.₃) |
      | 3. Sinking | the vehicle is sinking | max(*ρ* − *V* × 0.0015ₛ, *F*₀.₁) |
      | 4. Accelerating | *ρ* < *ρ*ₜ | min(*ρ* + *a*, *ρ*ₜ) |
      | 5. Decelerating | *ρ* > *ρ*ₜ | max(*ρ* − *V* × *η*, *ρ*ₜ) |
      | 6. Steady | none of the above | *ρ* |

      When the crush-slowdown state begins and when it ends belong to S-0210 (claim 9), and this
      spec defers to it. In outline: a vehicle of movement zone *crusher-all* enters the state when
      it drives into a new cell on a track in crush mode (claim 7) while a vehicle is in that cell,
      the driving vehicle itself counting, so on the ground a single vehicle in the path sets it
      from the cell before; and the state is held across tracks, stops and new orders until the
      vehicle next crushes an overlay. Case 1 is tested before braking, so the state also overrides
      the braking floor. The lowered target replaces the target fraction until the next track
      begins (claim 9), and every update while the state lasts lowers the new track's target
      again, so the vehicle runs at min(*ρ*ₜ, 0.2) for as long as the state lasts. It drops to that
      value at once when the state begins, and accelerates from it by case 4 when the state ends.

      Here *F*₀.₃ = 0.300000011920928955078125 and *F*₀.₁ = 0.100000001490116119384765625
      (single-precision 0.3 and 0.1 widened to double), and 0.0015ₛ = 0.001500000013038516044616699
      (single-precision 0.0015). The crushing cap 0.2 is the double nearest to 0.2.

      **Acceleration and deceleration are not symmetric.** Acceleration adds the flat factor *a* per
      tick. Deceleration subtracts *V* × *η*, the type speed times the factor, so faster types brake
      harder per tick for the same field value. The braking floor *F*₀.₃ also *raises* a slower vehicle to
      *F*₀.₃ once it is within *D*ₛ of its destination (quirk below).
    - **Acceleration flag on, vehicle passive:** none of the six cases applies, and *ρ* is not
      changed by this claim. It keeps whatever it had (claim 8).
    - **Trains.** A vehicle that has following cars (Terms), whose acceleration flag is on and
      which is not passive, copies its own *ρ* to every car of its train each time this claim updates
      its own fraction (once or twice per moving tick, claim 12), also when the update leaves *ρ*
      unchanged (row 6). A leader whose acceleration flag is off, or which is passive, does not
      copy.
    - Backing: `measurable` (scenarios D3, D4, D10, D13 and D14), except case 3, which is
      `unmeasured`: a sinking vehicle is being destroyed and is not normally given steps to drive,
      so no practical setup puts one on a track while it sinks
    - Confidence: high for the acceleration-off rule, for cases 2, 4 and 5, including the raise
      by the braking floor, for the table square root in *d*ₘ and for the passive rule (no case
      applies, and a passive leader does not copy); medium for case 1 (the speed rule; when the state
      begins and ends is S-0210's confidence), for case 3 and for trains

    These rules hold on the tracks of this spec (T1 and T2). The separate tracks that buildings use
    are outside it (Open questions).

11. **Stopping.** When a vehicle's move is stopped or cancelled while it is on a track, its move
    destination is cleared, and its target fraction becomes min(*ρ*ₜ, *F*₀.₃). It finishes the track
    it is on and stops at that track's target. When a vehicle reaches a track target and has no further step, it
    comes to rest with *ρ* = 0. A vehicle at rest starts every move from *ρ* = 0, accelerating if its
    acceleration flag is on (unless an earlier case of claim 10 applies: a vehicle still in the
    crush-slowdown state, for example, starts at min(*ρ*ₜ, 0.2) at once).
    - Backing: `measurable` (scenario D11)
    - Confidence: medium (the stopping rule was read; which orders lead to it belongs to the order
      specs)

12. **Ticks and track boundaries.** What a moving tick can do depends on whether the vehicle is on
    a track when the tick begins. A tick begins without a track when the vehicle starts moving from
    rest, has just finished turning in place, or finished a track at the very end of the previous
    tick (see below). In both cases the budget is spent as in claim 3.

    | Tick begins | Updates of *ρ* (claim 10) | Progress earned | When a track ends in the tick | Tracks finished, at most |
    |---|---|---|---|---|
    | without a track | one, towards the target fraction (claim 9) of the track the vehicle begins in this tick (claim 7) | *s* once, at the updated fraction | the tick ends there, with no track, and the leftover is kept | one |
    | on a track | one; and if the track ends and the move goes on, a second, towards the new track's target | *s* once, at the fraction after the first update; nothing more after the second | if the move goes on, the next track begins in the same tick and the leftover is spent on it; if that track ends too, the tick ends there, with no track | two |

    A vehicle at rest has *ρ* = 0, so with the acceleration flag on its first tick from rest gives it
    *ρ* = clamp(min(*a*, *ρ*ₜ), 0, 1) and it earns trunc(*s*₂ × *ρ*), unless an earlier row of claim
    10's table applies. For example, within *D*ₛ of its destination the braking floor gives
    *ρ* = *F*₀.₃ and trunc(*s*₂ × *F*₀.₃) (scenario D13), and a sinking vehicle farther away than
    *D*ₛ gets *F*₀.₁.

    Consequences: a vehicle that is accelerating (row 4 of claim 10's table in both updates) gains
    2*a* in a tick in which it reaches a track target and goes on. A vehicle whose *s* is at or above
    about one track's cost settles into finishing exactly one track per tick: each tick then finishes its track and the next one begins without a
    track. Its extra budget piles up unused, until it is lost at the next tick in which the vehicle
    turns in place, or in which it has a move but neither drives a track nor begins one, or kept when
    it comes to rest (claim 3). On straight and diagonal steps
    this caps a vehicle at one cell per tick.
    - Backing: `measurable` (scenarios D1, D3 and D13)
    - Confidence: medium (the order of the two updates and the cap were inferred from reading, not
      observed; the cap was simulated)

    **Quirk `quirks.drive_track_end_cap`.** Default (original): as above. Off: after every track end
    within a tick, the next track begins and the leftover is spent on it, with no limit on the number
    of tracks finished per tick. The fraction updates stay as in the default.

13. **Speed power-up.** When a speed power-up takes effect on a vehicle whose speed bonus factor is
    exactly 1.0, the factor is multiplied by the power-up's value. Otherwise it is unchanged, so a
    second power-up has no effect. The power-up is not limited to the vehicle that collects it: it
    acts on the mobile ground objects (aircraft excluded) within an area around the power-up's cell.
    The crate specs own which objects that area covers, the power-up's value and when it is
    collected.
    - Backing: `measurable` (scenario D12)
    - Confidence: high for the exactly-1.0 rule; medium that it acts on an area (read, not traced in
      detail)

## Formulas

Every floating-point operation in this spec, in the claims (the speed-fraction updates of claim 10
included) as well as in the formulas below, runs in the original's program-wide *round-toward-zero
mode* (glossary): results are rounded to double precision, toward zero. S-0205 (number domains) owns that
mode and its switch `quirks.chop_rounding`. Every conversion to an integer truncates toward zero
whatever the mode.

**F1 Type speed** (claim 1). *V* = min(⌊256 × *v*′ / 100⌋, 255), with *v*′ = min(max(*v*, 0), 100),
for every *v* except −1 (*v* = −1 leaves *V* unchanged). Integer arithmetic, and the division
truncates (*v*′ ≥ 0). Unit: progress units per tick.

**F2 Progress per tick** (claim 2).
> *s*₁ = trunc((*V* × *κ*) × *b*), where *κ* is single precision widened to double and *b* is double.
> *s*₂ = trunc(*s*₁ × *ν*) when the faster ability applies.
> *s* = trunc(*s*₂ × *ρ*).
> Flag carrier: trunc(*s* / 2) in place of *s*, an integer division truncating toward zero.
Unit: progress units.

**F3 Track cost** (claim 3). cost(track) = 7 (*n* + 1) − trunc(7 × (1 − *m*/11)), where *n* is the
number of points and *m* is the axis distance from the last point to the target. The correction is
computed in double precision under the round-toward-zero mode and truncated toward zero. The end
distances in T1 are *m* = 3 (curve A), 16 (curves B, C and E) and 11 (curves D and F), and their
corrections are 5, −3 and 0, the exact values of trunc(7 × (1 − *m*/11)). Results: curve A 163,
B 227, C 388, D 273, E 437, F 399. Unit: progress units.

**F4 Interpolation factor** (claim 4). *t* = the single-precision value of *A* × (1/7), with 1/7 the
double nearest to 1/7, which is slightly less than 1/7. The product is formed in double precision and
then converted to single precision, both under the program-wide mode (rounding toward zero), so *t* <
*A*/7 for every *A* from 1 to 7. Each displacement component, an integer, is multiplied by *t* in
double precision and truncated. (With rounding to nearest instead, *A* = 7 would give *t* = 1 exactly, and
scenario D8 can tell the two apart.) Unit: none (*t* is a dimensionless factor).

**F5 Distance to the destination** (claim 10). *n* = Δ*x*² + Δ*y*² + Δ*z*², the squared
difference between the vehicle's position and its move destination, an exact integer (the original
sums the squares in double precision, which is exact here). *d*ₘ = ⌊*r*(*n*)⌋, with *r* the table
square root of S-0204, Formulas F1: *n* is converted to single precision, its 14 leading significant
binary digits select the table entry, and the result is truncated to an integer. F1 also states how
the conversion depends on the rounding mode of S-0205. Unit: leptons. The comparison *d*ₘ < *D*ₛ is
between integers. S-0204's switch `quirks.table_root_distance` covers this distance too: off, *d*ₘ =
⌊√*n*⌋, the exact integer square root.

**F6 Target fraction** (claim 9). *ρ*ₜ = *h* × *z*, where *z* = *g* × *u* if *g* × *u* ≠ 0 and
*z* = 0.5 if *g* × *u* = 0, with *g* = min(*M*, 1), *u* the slope factor (1 when the heights are
equal) and *h* = 0.75 for a damaged vehicle and 1 otherwise. Double precision throughout. Unit:
fraction (dimensionless).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Speed field range | 0 to 100 | field value | Clamped before conversion; claim 1 |
| Speed scale | 256/100 | progress units per tick per field unit | Truncating integer division; claim 1 |
| Type speed cap | 255 | progress units per tick | Claim 1 |
| Speed field "keep" value | −1 | field value | Claim 1 |
| Cost of a track point | 7 | progress units | Claim 3 |
| End-correction length | 11 | leptons | Claim 3 |
| Interpolation divisor | 7 | progress units | Claim 4 |
| Interpolation snap threshold | 3 | progress units | Claim 4: snap to the next point if *A* > 3 |
| Braking floor *F*₀.₃ | 0.300000011920928955078125 | fraction | Claims 10 and 11 |
| Sinking floor *F*₀.₁ | 0.100000001490116119384765625 | fraction | Claim 10 |
| Sinking deceleration | 0.001500000013038516044616699 | fraction per progress unit per tick | Multiplied by *V*; claim 10 |
| Crushing cap | 0.2 | fraction | Claim 10 |
| Zero-multiplier fallback | 0.5 | fraction | Claim 9 (d) |
| Damaged factor | 0.75 | factor | Claim 9 (e) |
| Deck height | 4 | height levels | Claim 9 (a); the glossary's bridge deck |
| Level tolerance | 2 | height levels | Claim 9 (a): a difference below 2 uses the cell's land type |
| Flag-carrier divisor | 2 | divisor (dimensionless) | Claim 2 |
| Straight step cost | 163 | progress units | F3 |
| Diagonal step cost | 227 | progress units | F3 |
| Absent-key default: speed field | type speed 0 | progress units per tick | Claim 1 |
| Absent-key default: acceleration flag | on | boolean | Claim 10 |
| Absent-key default: slowdown distance *D*ₛ | 500 | leptons | Claim 10 |
| Absent-key default: deceleration factor *η* | 0.002 | fraction per progress unit per tick | Claim 10 |
| Absent-key default: acceleration factor *a* | 0.03 | fraction per tick | Claim 10 |
| Absent-key default: uphill and downhill factors | 1.0 each | factor | Claim 9 (c) |
| Absent-key default: veteran speed factor *ν* | 1.0 | factor | Claim 2 |
| Absent-key default: country speed factor *κ* | 1.0 | factor | Claim 2 |
| Starting speed bonus factor *b* | 1.0 | factor | Claim 13 |
| Starting speed fraction *ρ* | 0 | fraction | Claim 11 |

**T1 Track point lists.** Each point is (*x*, *y*, *f*): displacement from the track target in
leptons along the map axes (*y* grows towards map south), and facing in 1/256 turn. The lists are in
driving order, and each track ends at (0, 0), its target, which is not listed. The point *numbers*
used for the chain, entry and release points count from 0. The release point is where the vehicle
gives up the cell it came from, which the occupancy specs use. Some points break the regular pattern,
for example (−129, 129) in curve B and the facings 66 then 64 in curve D. They are part of the
original data and are kept.

- **Curve A** (straight, towards map north; 23 points; no chain point): (0, 245, 0), (0, 234, 0),
  (0, 223, 0), (0, 212, 0), (0, 201, 0), (0, 190, 0), (0, 179, 0), (0, 168, 0), (0, 157, 0),
  (0, 146, 0), (0, 135, 0), (0, 124, 0), (0, 113, 0), (0, 102, 0), (0, 91, 0), (0, 80, 0),
  (0, 69, 0), (0, 58, 0), (0, 47, 0), (0, 36, 0), (0, 25, 0), (0, 14, 0), (0, 3, 0).
- **Curve B** (diagonal, towards map north-east; 31 points; no chain point): (−248, 248, 32),
  (−240, 240, 32), (−232, 232, 32), (−224, 224, 32), (−216, 216, 32), (−208, 208, 32),
  (−200, 200, 32), (−192, 192, 32), (−184, 184, 32), (−176, 176, 32), (−168, 168, 32),
  (−160, 160, 32), (−152, 152, 32), (−144, 144, 32), (−136, 136, 32), (−129, 129, 32),
  (−120, 120, 32), (−112, 112, 32), (−104, 104, 32), (−96, 96, 32), (−88, 88, 32), (−80, 80, 32),
  (−72, 72, 32), (−64, 64, 32), (−56, 56, 32), (−48, 48, 32), (−40, 40, 32), (−32, 32, 32),
  (−24, 24, 32), (−16, 16, 32), (−8, 8, 32).
- **Curve C** (N then NE; 54 points; chain point 37, entry point 12, release point 22):
  (−256, 501, 0), (−256, 490, 0), (−256, 479, 0), (−256, 468, 0), (−256, 457, 0), (−256, 446, 0),
  (−256, 435, 0), (−256, 424, 0), (−256, 413, 0), (−256, 402, 0), (−256, 391, 0), (−256, 383, 0),
  (−256, 373, 0), (−256, 363, 0), (−254, 352, 1), (−252, 341, 3), (−250, 332, 4), (−248, 321, 5),
  (−245, 311, 7), (−241, 302, 8), (−237, 292, 9), (−233, 282, 11), (−229, 272, 12),
  (−225, 263, 13), (−220, 252, 15), (−216, 243, 16), (−212, 236, 17), (−206, 224, 19),
  (−202, 215, 20), (−195, 207, 21), (−190, 198, 23), (−183, 186, 24), (−179, 176, 25),
  (−168, 168, 27), (−160, 160, 28), (−152, 152, 29), (−144, 144, 31), (−136, 136, 32),
  (−129, 129, 32), (−120, 120, 32), (−112, 112, 32), (−104, 104, 32), (−96, 96, 32), (−88, 88, 32),
  (−80, 80, 32), (−72, 72, 32), (−64, 64, 32), (−56, 56, 32), (−48, 48, 32), (−40, 40, 32),
  (−32, 32, 32), (−24, 24, 32), (−16, 16, 32), (−8, 8, 32).
- **Curve D** (N then E; 38 points; chain point 26, entry point 11, release point 19):
  (−256, 245, 0), (−256, 235, 0), (−256, 224, 0), (−256, 213, 0), (−255, 203, 0), (−253, 192, 0),
  (−251, 181, 1), (−249, 171, 1), (−246, 160, 2), (−243, 149, 3), (−240, 139, 4), (−236, 127, 5),
  (−232, 117, 8), (−228, 109, 12), (−222, 99, 16), (−219, 90, 20), (−213, 82, 23), (−206, 72, 27),
  (−201, 64, 32), (−195, 56, 36), (−186, 48, 39), (−177, 43, 43), (−168, 36, 47), (−160, 32, 51),
  (−147, 27, 54), (−135, 23, 57), (−126, 20, 60), (−113, 17, 62), (−104, 13, 63), (−94, 9, 64),
  (−84, 6, 64), (−75, 4, 66), (−64, 3, 64), (−53, 2, 64), (−43, 1, 64), (−32, 0, 64),
  (−21, 0, 64), (−11, 0, 64).
- **Curve E** (NE then SE; 61 points; chain point 45, entry point 15, release point 31):
  (−504, −8, 32), (−496, −16, 32), (−488, −24, 32), (−480, −32, 32), (−472, −40, 32),
  (−464, −48, 32), (−456, −56, 32), (−448, −64, 32), (−440, −72, 32), (−432, −80, 32),
  (−424, −88, 32), (−416, −96, 32), (−408, −104, 32), (−400, −112, 32), (−392, −120, 32),
  (−385, −127, 32), (−376, −136, 32), (−368, −143, 32), (−361, −150, 32), (−353, −158, 32),
  (−344, −166, 32), (−336, −173, 35), (−329, −181, 38), (−322, −188, 41), (−316, −194, 44),
  (−306, −199, 47), (−296, −204, 50), (−288, −208, 53), (−277, −211, 56), (−267, −212, 59),
  (−256, −213, 62), (−245, −212, 66), (−235, −211, 69), (−225, −208, 72), (−216, −204, 75),
  (−208, −199, 78), (−198, −194, 81), (−188, −188, 84), (−181, −181, 87), (−176, −173, 90),
  (−168, −166, 93), (−160, −158, 96), (−152, −150, 96), (−144, −143, 96), (−136, −136, 96),
  (−129, −129, 96), (−120, −120, 96), (−112, −112, 96), (−104, −104, 96), (−96, −96, 96),
  (−88, −88, 96), (−80, −80, 96), (−72, −72, 96), (−64, −64, 96), (−56, −56, 96), (−48, −48, 96),
  (−40, −40, 96), (−32, −32, 96), (−24, −24, 96), (−16, −16, 96), (−8, −8, 96).
- **Curve F** (NE then E; 56 points; chain point 44, entry point 16, release point 27):
  (−512, 256, 32), (−504, 248, 32), (−496, 240, 32), (−488, 232, 32), (−480, 224, 32),
  (−472, 216, 32), (−464, 208, 32), (−456, 200, 32), (−448, 192, 32), (−440, 184, 32),
  (−432, 176, 32), (−424, 168, 32), (−416, 160, 32), (−408, 152, 32), (−400, 144, 32),
  (−392, 136, 32), (−385, 129, 32), (−376, 120, 32), (−368, 112, 32), (−360, 104, 32),
  (−352, 96, 32), (−344, 88, 32), (−338, 85, 32), (−328, 78, 35), (−320, 72, 37), (−311, 66, 40),
  (−302, 59, 43), (−294, 55, 45), (−285, 50, 48), (−277, 43, 51), (−267, 38, 53), (−258, 34, 56),
  (−248, 28, 59), (−238, 25, 61), (−229, 21, 64), (−218, 17, 64), (−208, 14, 64), (−199, 11, 64),
  (−189, 9, 64), (−178, 7, 64), (−169, 5, 64), (−158, 3, 64), (−147, 1, 64), (−137, 0, 64),
  (−129, 0, 64), (−117, 0, 64), (−107, 0, 64), (−96, 0, 64), (−85, 0, 64), (−75, 0, 64),
  (−64, 0, 64), (−53, 0, 64), (−43, 0, 64), (−32, 0, 64), (−21, 0, 64), (−11, 0, 64).

**T2 Track for each pair of steps.** A curve of T1 is used as listed or transformed. The transforms
are applied to every point in the order given:
- *mirror*: (*x*, *y*, *f*) → (*y*, *x*, (192 − *f*) mod 256), a reflection in the line *x* = *y*,
- *flip x*: (*x*, *y*, *f*) → (−*x*, *y*, (256 − *f*) mod 256),
- *flip y*: (*x*, *y*, *f*) → (*x*, −*y*, (128 − *f*) mod 256).

"Two" marks a two-cell curve. "—" means no curve: the straight or diagonal track for the current step
alone is used, and the vehicle turns in place afterwards (claim 7).

| current step | next N | next NE | next E | next SE | next S | next SW | next W | next NW |
|---|---|---|---|---|---|---|---|---|
| N | A | C, two | D, two | — | — | — | D flip x, two | C flip x, two |
| NE | F mirror, flip x, flip y, two | B | F, two | E, two | — | — | — | E mirror, flip x, flip y, two |
| E | D mirror, flip x, flip y, two | C mirror, flip x, flip y, two | A mirror, flip x | C mirror, flip x, two | D mirror, flip x, two | — | — | — |
| SE | — | E flip y, two | F flip y, two | B flip y | F mirror, flip x, two | E mirror, flip x, two | — | — |
| S | — | — | D flip y, two | C flip y, two | A flip y | C flip x, flip y, two | D flip x, flip y, two | — |
| SW | — | — | — | E mirror, two | F mirror, two | B mirror | F flip x, flip y, two | E flip x, flip y, two |
| W | D mirror, flip y, two | — | — | — | D mirror, two | C mirror, two | A mirror | C mirror, flip y, two |
| NW | F mirror, flip y, two | E mirror, flip y, two | — | — | — | E flip x, two | F flip x, two | B flip x |

The entries where both steps are the same (the diagonal of the table) are the only straight and
diagonal tracks. In every "—" case the straight or diagonal track for the current step is the
diagonal entry of that row. Each listed transform makes the track start one step (or two) behind
its target in the direction of travel, and end with facing equal to the last step's 8-direction.

## State and ordering

| State | Event | Result |
|---|---|---|
| at rest (*ρ* = 0, no track) | move order gives path steps | turn in place until facing equals the current step (claim 6), then begin a track (claim 7) |
| turning in place | facing reaches the step's 8-direction | begin a track in that tick: *ρ* updated once and *s* earned, as in any moving tick (claim 12) |
| on a track | moving tick | it earns *s* at this tick's updated fraction (claim 10), passes the track points that its budget buys (claim 3), and stands between points as claim 4 gives |
| on a track (tick began on it) | track target reached, move continues | the next track (claim 7) begins in the same tick: *ρ* is updated a second time, and the leftover, with the end correction of claim 3 applied, is spent on the new track (claim 12) |
| on a track (tick began without one) | track target reached, move continues | correction of claim 3; the tick ends with no track and the leftover kept; the next track begins next tick (claim 12) |
| on a track that began in this tick, after the track the tick began on ended | that track's target also reached in the same tick | correction of claim 3; the tick ends with no track (claim 12) |
| no track, move to make | facing differs from the current step | turn in place, *A* = 0 (claim 6) |
| on a track | track target reached, no step left | at rest, *ρ* = 0 (claim 11) |
| on a track | move stopped | target fraction becomes min(*ρ*ₜ, *F*₀.₃), destination cleared, finish the current track (claim 11) |

A change of speed fraction takes effect in the progress of the same tick (claim 10).

## Edge cases and quirks

- `quirks.drive_curve_chaining_passive_only` (claim 8). Default: only passive vehicles chain curves.
  Off: all vehicles do.
- **`quirks.drive_brake_floor_raises`** (claim 10, case 2). Default (original): within the slowdown
  distance, *ρ*′ = max(*ρ* − *V* × *η*, *F*₀.₃), which raises a vehicle slower than *F*₀.₃ (for
  example one starting from rest near its destination) to *F*₀.₃ at once. Off: case 2 applies only
  while *ρ* > *F*₀.₃. For *ρ* ≤ *F*₀.₃ within the slowdown distance, rows 3 to 6 apply as usual and
  their result is capped at *F*₀.₃, so a slow vehicle near its destination accelerates at its usual
  rate up to the floor, and braking never speeds a vehicle up. Scenario D13 tells the two apart.
- **`quirks.drive_subpoint_compounding`** (claim 4). Default (original): the step towards the next point is
  taken from the current, possibly interpolated, position. Off: it is taken from the last point
  passed, so *Q*′ = last point + trunc((*P* − last point) × *t*), which gives an even spacing.
- **`quirks.drive_track_end_cap`** (claim 12). Default (original): a tick finishes at most one
  track if it began without one, and at most two if it began on one, so very fast vehicles move at
  most one cell per tick on straight and diagonal steps and their extra budget piles up unused. Off:
  no limit.
- **Leftover kept at rest** (claim 3). A vehicle that stops at its destination keeps its leftover
  budget, which speeds up the first tick of its next move if that move needs no turn in place. This
  is original behaviour and has no switch, because its effect is small. Below the track-end cap the
  leftover is at most one tick's progress *s* plus 5 progress units (the largest end correction of
  claim 3), so it shortens the next move by about one tick, and by a few ticks only for the slowest
  types. Above the cap, claim 12 limits the first tick to one track whatever the leftover.
- **Asymmetric acceleration** (claim 10) is original behaviour and has no switch, because it is how
  the acceleration and deceleration fields are defined: mods tune the fields with it in mind, and a
  symmetric rule would only change what those fields mean.
- **Progress units are not leptons.** A vehicle covers about 11/7 of a lepton per progress unit on a
  straight step (256 leptons for 163 units) and about 1.6 on a diagonal one (362 for 227). Specs and
  tools that need speed in cells per tick must use F3, not a lepton conversion.
- **Budget lost when stopping to turn** (claim 6). Leftover progress is discarded whenever the vehicle
  turns in place, and whenever it has a move to make but neither drives a track nor begins one, so a path with many sharp turns costs slightly more than the track
  sums. This is original behaviour and has no switch: it is also the only rule that empties the budget
  that piles up above the track-end cap (claim 12), so turning it off would need a new limit of its
  own, which is a design change rather than a quirk.
- **Odd points in the track data** (T1) are original data, and our engine keeps them.
- **Table root in the braking distance** (claim 10, F5). The distance to the move destination uses
  the table square root, so it can be a lepton or more short of the exact distance from 129 leptons
  on, and a vehicle can start braking slightly before it is within *D*ₛ by the exact distance. This
  spec defines no switch of its own: S-0204's `quirks.table_root_distance` governs it.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| vehicle section, key `Speed` | speed field *v* (claim 1) | integer (S-0003, claim 3) | keeps value; engine default type speed 0; −1 also keeps |
| vehicle section, key `Accelerates` | acceleration flag | boolean | yes |
| vehicle section, key `AccelerationFactor` | *a* | decimal (S-0003, claim 6), widened to double | 0.03 |
| vehicle section, key `DeaccelerationFactor` (spelled this way) | *η* | decimal, widened to double | 0.002 |
| vehicle section, key `SlowdownDistance` | *D*ₛ | integer (S-0003, claim 3), leptons | 500 |
| vehicle section, key `Passive` | passive flag | boolean | no |
| vehicle section, key `SpeedType` | locomotion class (S-0211) | name | S-0211 claim 3 |
| `[General]`, keys `TrackedUphill`, `TrackedDownhill`, `WheeledUphill`, `WheeledDownhill` | uphill and downhill factors for class *track* and for every other class (claim 9) | decimal, widened to double | keeps value; engine default 1.0 |
| `[General]`, key `VeteranSpeed` | *ν* | decimal, widened to double | keeps value; engine default 1.0 |
| vehicle section, keys `VeteranAbilities`, `EliteAbilities` | whether the *faster* ability applies | list of ability names | owned by the veterancy spec |
| `[AudioVisual]`, key `ConditionYellow` | yellow ratio (claim 9 e) | as in S-0108 | as in S-0108 |
| country section, key `SpeedUnitsMult`; the country's matching keys for infantry and aircraft do not feed this spec | *κ* | decimal (S-0003, claim 6), stored in single precision | 1.0 |
| land-type sections | *M* | S-0211 | S-0211 |

No other speed field of a country or of a difficulty level enters the formulas of this spec
(Open questions).

## Measurement scenarios

All scenarios run under the glossary's *measurement setup* on a purpose-made flat map of clear
ground, unless the scenario says otherwise, at any game speed (claim G3 of the glossary). A
*recorder* reads, each tick, the vehicle's position in leptons and its facing. Test values are
set by override sections in the test map (a named test file, removed after the run). All values
in them are invented test values:
- the test vehicle type's speed field (given per scenario), with the acceleration flag off unless
  the scenario says otherwise,
- its locomotion class set to wheel, with the clear land type's wheel multiplier at 100 %,
- the owner's country speed factor for vehicles set to 1.0,
- for the uphill and downhill factors, 1.0 unless the scenario says otherwise.

The vehicle starts at full health and without veterancy. No scenario's expected result depends on a
random draw, so each variant runs once unless the scenario gives a run count. "Arrival at
cell *k*" means the first tick at whose end the vehicle's position is at or past the centre of the *k*-th cell of the route, along
its direction of travel. (When a move continues, the next track starts in the same tick, so the
position at the end of that tick is already a little past the centre.) Paths are logged as the
sequence of cells the vehicle's track targets pass through, because which steps the pathfinder
chooses is not part of this spec. Where a scenario needs a particular path, it uses corridors one
cell wide, bounded by cells whose land multiplier for the vehicle's class is 0 (S-0211).

**D1: steady progress, straight and diagonal (claims 1 to 3).**
- *Setup:* speed field 20, then 40, then 100. A route of 30 cells straight towards map north. A second
  route of 30 cells towards map north-east.
- *Observable:* arrival tick at cell 20 minus arrival tick at cell 10.
- *Runs:* 1 per value and route (no random draw is involved).
- *Expected:* straight 31 or 32, 15 or 16, and 10 ticks. Diagonal 44 or 45, 22 or 23, and 10 ticks.
  For fields 20 and 40 these are within one tick of 1630/*s* and 2270/*s* (*s* = 51 and 102). For
  field 100 (*s* = 255, more than one track's cost) claim 12 caps the vehicle at one cell per tick,
  so both routes take exactly 10 ticks.
- *Falsified if:* a value for field 20 or 40 differs from the expected range by more than one tick,
  or field 100 takes other than 10 ticks (which also falsifies the default of
  `quirks.drive_track_end_cap`; without the cap it would take 6 or 7 straight and 9 diagonal). A
  lepton reading of the speed (10 × 256/*s* for straight, about 50 ticks at field 20) falsifies
  claim 3.

**D2: speed field limits (claim 1).**
- *Setup:* as D1 (straight), with speed fields 150, 100, 1, 0 and −1. For −1, also run a control
  without the override.
- *Observable:* as D1, arrival tick at cell 20 minus arrival tick at cell 10; for field 0, whether
  the vehicle moves at all.
- *Expected:* 150 and 100 give the same result. Field 1 (*s* = 2) takes 815 ± 1 ticks for cells 10 to
  20. Field 0 never moves. Field −1 behaves like the control.
- *Falsified if:* 150 is faster than 100, field 1 is off by more than a tick, or −1 differs from the
  control.

**D3: acceleration from rest (claims 9, 10 and 12).**
- *Setup:* speed field 20, acceleration flag on, acceleration factor 0.02, deceleration factor 0,
  slowdown distance 0. The vehicle starts at rest at a cell centre, already facing map north, and is
  a map-placed vehicle that has not moved before (so it has no leftover budget, claim 3). Route:
  30 cells north.
- *Observable:* ticks from the first tick on which the position changes to the arrival at cells 1 and
  10.
- *Expected:* 17 ticks to cell 1 and 55 ticks to cell 10, counted from the first tick of movement.
  These are computed from claims 2, 3, 4, 10 and 12. The fraction is 0.02 in the first tick (which
  already earns 1 unit and moves the vehicle 1 lepton through claim 4), gains 0.02 per tick and a
  further 0.02 at each cell centre reached, and is capped at 1.0. The run is deterministic.
- *Falsified if:* either value differs. Without the second update at cell centres (claim 12), cell 10
  takes 57 ticks. A vehicle that earns nothing in its first tick takes 54.

**D4: braking near the destination (claim 10, case 2).**
- *Setup:* speed field 20, acceleration flag on, acceleration factor 1.0, deceleration factor 0.01,
  slowdown distance 1280 (5 cells). Route of 20 cells north, ending at the destination.
- *Observable:* ticks per cell for the last four cells.
- *Expected:* about 163/15 ≈ 11 ticks per cell (*s* = trunc(51 × *F*₀.₃) = 15), against about
  3.2 ticks per cell earlier on the route.
- *Falsified if:* the last cells take the same time as the earlier ones, or a time that
  corresponds to a floor other than 0.3.

**D5: slopes (claim 9 c).**
- *Setup:* a route north that climbs one height level over a slope cell and later descends one.
  Overrides set the wheeled uphill factor to 0.5 and the wheeled downhill factor to 1.0. Control: a
  tracked vehicle with the tracked factors left at 1.0.
- *Observable:* ticks spent on the track into the uphill cell, against a flat cell.
- *Expected:* about twice as long for the wheeled vehicle (*s* = 25 against 51). The tracked vehicle is
  not slowed.
- *Falsified if:* the wheeled vehicle is not slowed uphill, or the tracked control is slowed.

**D6: damage (claim 9 e).**
- *Setup:* as D1 (straight, field 20), with the vehicle's health set by the map at or just below the
  yellow ratio, and a second run clearly above it.
- *Observable:* as D1, arrival tick at cell 20 minus arrival tick at cell 10.
- *Expected:* 43 ± 1 ticks for cells 10 to 20 in the first run (*s* = 38), and 32 ± 1 in the second.
- *Falsified if:* both runs are equal.

**D7: curve against turn in place (claims 5 to 7).**
- *Setup:* field 20, acceleration off. Map A: a corridor one cell wide that runs 5 cells north and
  then 5 cells east (an L). Map B: a corridor 5 cells north, then an open area to the south-east of
  its end, with the destination 5 cells south-east of the corridor's end.
- *Observable:* the logged path, the positions through each turn, the ticks from arrival at the cell
  before the turn to arrival at the cell after it, and the facing at the end of each turn.
- *Expected:* at every logged pair (N, E), the vehicle drives one curve (curve D) from the centre of
  the cell before the corner to the centre of the cell after it, without passing the corner cell's
  centre. It takes 273/51, so 5 or 6 ticks, and ends facing map east. At every logged pair whose
  directions differ by 3/8 turn (for example N then SE), it reaches the shared cell's centre, stops,
  and turns in place (its duration belongs to S-0206, claim 12) before driving on.
- *Falsified if:* at a logged (N, E) pair the vehicle passes through the corner cell's centre, or at a
  3/8-turn pair it curves.

**D8: positions between points (claim 4).**
- *Setup:* speed field 1 (*s* = 2), acceleration off. A map-placed vehicle that has not moved
  before (so it has no leftover budget, claim 3), already facing map north. Straight route north.
- *Observable:* the displacement from the start for the first eight moving ticks.
- *Expected:* 3, 7, 10, 12, 16, 20, 21, 25 leptons.
- *Falsified if:* the sequence differs. An evenly spaced sequence (3, 6, 9, …) indicates the "off"
  behaviour of `quirks.drive_subpoint_compounding`. A seventh value of 22 instead of 21 indicates that
  the interpolation factor is rounded to nearest rather than toward zero (F4; S-0205, number domains).

**D9: no chaining (claims 7 and 8).**
- *Setup:* field 20, acceleration off (needed for the passive control, see claim 8). A corridor one
  cell wide shaped so that the only path is N, N, NE, E, E, … (two turns in consecutive cells).
  Control: the same with the passive flag set by override.
- *Observable:* the logged path, and the ticks at which the vehicle stops and turns at the middle
  cell of each logged triple, read from the recorder's positions and facings.
- *Expected:* at every logged triple (*d*, *e*, *g*) where *e* is 1/8 turn from *d* and *g* ≠ *e*, the
  ordinary vehicle drives the curve of the pair (*d*, *e*) to its target's centre, stops there, turns
  in place to *g*, and then drives on. The passive control does not stop or turn in place at that cell.
- *Falsified if:* the ordinary vehicle passes the curve's target without stopping to turn.

**D10: the crush slowdown (claims 7 and 10, case 1; S-0210 claim 9).**
- *Setup:* a test vehicle type of movement zone *crusher-all*, speed field 20, acceleration flag on,
  acceleration factor 1.0 and slowdown distance 0 (test values). A straight route of 30 clear cells
  north with no overlay on it, destination cell 30. Run A: one enemy vehicle that the test vehicle
  can crush stands in cell 10. Run C: two infantry units stand in cells 9 and 10. Control: no
  obstacles. The obstacles are given no weapon and no orders by test override, so that they stay in
  place; a run in which an obstacle moves before it is reached is repeated.
- *Runs:* 3 per run, since the obstacles' own behaviour is not part of this spec.
- *Observable:* the per-tick displacement, and the cell in which it changes.
- *Expected:* in the control the vehicle earns 51 progress units per tick all the way (about 3.2
  ticks per cell). In run A, the track from cell 8 into cell 9 is in crush mode (its second cell
  holds the enemy vehicle, claim 7), and from the first tick that begins after the vehicle has
  driven into cell 9 it earns trunc(51 × 0.2) = 10 progress units per tick at once, not after a
  gradual slowdown. It keeps that rate through the crush in cell 10 and up to cell 30 (about 16 ticks
  per cell), because it crushes no overlay (S-0210 claim 9). In run C there is no slowdown:
  infantry neither puts a track into crush mode nor sets the state.
- *Falsified if:* run A shows no slowdown, a slowdown that begins only in cell 10, a speed that
  falls over several ticks, or a recovery before cell 30; or run C shows a slowdown. The state's
  persistence across stops and new orders and its end at an overlay are measured by S-0210
  scenario K5.

**D11: stopping (claim 11).**
- *Setup:* field 20, acceleration off. The vehicle is ordered north, and a stop order is given when
  it is about 1/3 of the way into a straight step.
- *Observable:* the vehicle's position each tick from the stop order until it stops.
- *Expected:* it finishes that step at *s* = 15 and stops at the cell centre.
- *Falsified if:* it stops at once, or finishes at full speed.

**D12: country factor, veteran factor and speed power-up (claims 2 and 13).**
- *Setup:* as D1 (straight, field 20). Run 1: country speed factor for vehicles 0.5. Run 2: a map-placed
  veteran vehicle whose type grants the faster ability at that level, with the veteran speed factor
  set to 1.5. Run 3: a speed power-up of value 1.5 is placed on the route and picked up before
  cell 10. Run 4: two such power-ups are picked up before cell 10. The power-ups come from crates on
  the route, with a test rules file that leaves the speed power-up as the only one a crate can give
  and sets its value to 1.5 (which power-up a crate gives belongs to the crate specs). No other
  mobile object is near the crates (claim 13).
- *Observable:* as D1, arrival tick at cell 20 minus arrival tick at cell 10.
- *Expected:* run 1 66 ± 1 ticks (*s* = 25). Run 2 22 ± 1 ticks (*s* = 76). Run 3 22 ± 1. Run 4 the
  same as run 3.
- *Falsified if:* any run is off by more than one tick, or run 4 is faster than run 3.

**D13: the braking floor raises a slow vehicle (claims 10 and 12, `quirks.drive_brake_floor_raises`).**
- *Setup:* speed field 20, acceleration flag on, acceleration factor 0.02, deceleration factor 0,
  slowdown distance 1280 (5 cells). The vehicle starts at rest at a cell centre, already facing map
  north, and is ordered to a destination 3 cells north, so the whole move is within the slowdown
  distance. It is a map-placed vehicle that has not moved before (so it has no leftover budget,
  claim 3).
- *Observable:* the displacement at the end of the first tick of movement, and the ticks from that
  tick to the arrival at cells 1 and 3.
- *Expected:* the fraction is *F*₀.₃ from the first tick (*s* = trunc(51 × *F*₀.₃) = 15), so the first
  tick moves the vehicle 23 leptons, and cells 1 and 3 are reached 11 and 32 ticks after it. These
  are computed from claims 2 to 4, 10 and 12.
- *Falsified if:* any value differs. The "off" behaviour of the quirk (acceleration from 0 by 0.02 per
  tick up to *F*₀.₃) gives 1 lepton in the first tick and 18 and 39 ticks. A vehicle that never
  moves falsifies the case order of claim 10.

**D14: following cars copy the leader's fraction (claim 10, trains).**
- *Setup:* a test map with a leading vehicle and one following car linked to it by the map's vehicle
  entries. Both types have speed field 20 and the acceleration flag on. The leader's acceleration
  factor is 1.0. The car's type has acceleration factor 0.001 and is not passive. Straight route
  north for the leader.
- *Observable:* the car's displacement per tick over the first 20 ticks of its movement.
- *Expected:* the car moves at about the leader's pace, about 51 progress units per tick, so it
  covers a cell in about 3 or 4 ticks. Precondition: the car must follow the leader at all; if it
  does not move, the scenario is inconclusive, because how a car is steered belongs to no written
  spec (Terms).
- *Falsified if:* the car accelerates by its own factor alone: at 0.001 per tick it earns nothing for
  its first 19 ticks and needs about 90 ticks for its first cell.

**D15: the flag carrier's halving (claim 2).**
- *Setup:* as D1 (straight, field 20), in a skirmish game with a test rules file that turns on the
  game-setup default for capture the flag. The test vehicle picks up an enemy flag before cell 10.
- *Observable:* as D1, arrival tick at cell 20 minus arrival tick at cell 10, while it carries the
  flag.
- *Expected:* 66 ± 1 ticks (*s* = trunc(51 / 2) = 25), against 32 ± 1 without the flag.
  Precondition: a flag must exist and be carried; if none can be picked up, the scenario is
  inconclusive (Open questions).
- *Falsified if:* a vehicle carrying a flag crosses cells 10 to 20 in 32 ± 1 ticks.

## Open questions

- Whether any speed field of a country or of a difficulty level other than the country's vehicle
  speed factor reaches a vehicle's speed. None does in the formula of claim 2. It is believed that no
  other rule of the original uses them, but this has not been searched exhaustively. This also bears
  on S-0006 open question 7.
- The exact rule for the *faster* ability belongs to the veterancy spec.
- Which events set a vehicle sinking (Terms) and what happens to it afterwards. It appears to be part
  of a vehicle's destruction, for example when a vehicle that cannot float is teleported onto a water
  cell that is not a high-bridge cell (inferred from reading, low confidence). No written spec owns
  this yet; the destruction rules should.
- How a following car (Terms) is steered along its leader's route, and how the link is made. It
  appears to come from the map's vehicle entries (inferred from reading, low confidence), which the
  map-format spec does not cover yet.
- Whether a game started with the capture-the-flag option on places flags that a vehicle can carry
  (scenario D15 checks this as its precondition).
- The fallback when the second cell of a two-cell curve is blocked (claim 7) was read at low
  confidence.
- Which cells decide crush mode (claim 7) was read but not exercised. Scenario D10 tests it
  together with the start of the crush-slowdown state, which S-0210 (claim 9, scenario K5) owns.
- The ship locomotor's speed fraction is outside this spec, which covers only the driving
  locomotor. S-0210 claim 9 assumes that the ship locomotor updates its speed fraction like
  claim 10 here, as far as the crush slowdown is concerned; a spec of the ship locomotor should
  state its own rule.
- Whether a vehicle at rest keeps its leftover budget on every way of arriving (claim 3).
- Whether interpolation (claim 4) can move a vehicle into a cell that it has not yet claimed under
  the occupancy rules. The occupancy specs should settle this.
- The tracks used by vehicles leaving factories and other buildings are separate engine data and
  belong to the structures specs. A reading, not exercised (low confidence), points to two rules
  for them. On such a track, a vehicle whose acceleration flag is on gets no update of claim 10 at
  all: none of its six cases applies (the braking and sinking floors and the crush slowdown
  included), and a leader copies nothing to its cars. A vehicle whose acceleration flag is off still
  takes *ρ* = clamp(*ρ*ₜ, 0, 1). And a move that begins while the vehicle is on such a track sets
  its speed fraction straight to the new target fraction of claim 9, without acceleration. The
  second rule would also give a passive vehicle a speed fraction (claim 8). The structures specs
  should settle both.
