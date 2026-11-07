---
id: S-0207
title: Ground vehicle movement (drive locomotor)
topic: movement
revision: 85
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-07
measurements: []
requests: [Q-0019, Q-0020, Q-0035, Q-0040, Q-0071, Q-0072, Q-0200, Q-0181, Q-0232, Q-0233, Q-0240, Q-0310, Q-0268, Q-0356, Q-0600, Q-0676, Q-0793, Q-0794, Q-0797, Q-0830, Q-1090, Q-1110, Q-1130, Q-1131, Q-1160, Q-1759, Q-2781]
supersedes: []
---

# S-0207: Ground vehicle movement (drive locomotor)

## Summary

How a vehicle with the driving locomotor turns its speed field, speed fraction and terrain into
progress along fixed cell-to-cell tracks (straight, diagonal and two-cell curves), where it stands
between track points, and what it does when the cell of its next step may not be entered. It also
states which locomotor a mobile type has (claim 14) and how every vehicle, whatever its locomotor,
answers a scatter request (claim 16), while where a vehicle may go (S-0211), which steps it takes
(S-0212), how fast it turns (S-0206) and every other locomotor (apart from claims 16, 19, 20, 21 and 28) are
left to other specs. Claim 20 states how an object with the teleporting locomotor moves on its
own: where it lands, what the jump does and how long it then phases in. Claim 21 covers
its retained state after temporary driving, and the cancellation of a jump pending when that
driving is installed. Claim 22 fixes cell-tag arrival timing for driving vehicles and
own-locomotor teleports. Claim 18 also defines the speed estimate supplied by a driving target for
a shot's lead, including
turning and waiting. Claim 19 states when a vehicle with the driving, ship or hovering locomotor
that approaches a target stops once the target is in range, and what happens to its destination. Claims 24 and 25 distinguish
stored heading height and the separate locomotor destination point, including factory exits.
Claim 26 distinguishes cell-goal removal at rest from retained goals naming buildings or units and fixes
its tick relative to final track completion. Claim 27 covers the qualified teleport/dock
transition and its separate destination-refresh and movement-point suppression states for vehicles.
Claim 28 says which locomotor a vehicle with the teleport-capable flag uses: it drives except on its
refinery docking approach and on its jump when that refinery is sold. This spec covers skirmish. Network games and campaigns are out of scope.

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
rules of S-0210. *Scatter request* (with its *threat point* and its *forced* and *move-anywhere*
options), *infantry*, *aircraft*, *building*, *house*, *departure*, *recorder*, *follower car*
(with its *leader* and *car chain*), *train type*, *veterancy rank* and *veterancy ability* are used
as in the glossary. From S-0209: the *entry outcome* and its values (*clear*, *cloaked
blocker*, *moving blocker*, *closed friendly gate*, *friendly obstacle*, *enemy obstacle*,
*stationary friendly*, *blocked*), *cell layer*, *listed*, *allied* (the mover's view unless
stated), *contact*, the *close-enough distance* (its Formulas F2) and the *path retry timer*. From
S-0208 (Terms): the *screen-cover check* and the rounding of a facing to
the eight step directions (its Formulas F6). From S-0212: the *nearby-cell search* of its claim 25
and a *fresh search* (its claim 20). From S-0215: *behaviour* and the behaviour names (*sleep*,
*sticky*, *unload*, *guard*, *move*, *attack*), the *current* and *queued* behaviour (its claim 1),
the *scatter* and *paralysed* properties of a behaviour, *falling back to rest* (its claim 13), an
*override* (its claim 10), *target*, *destination* and *approach destinations* (its Terms). From
S-0211: the land type *tunnel* and the *high-bridge cell* (its claim 15). Whether a drive vehicle
is *on a bridge deck* (glossary *bridge deck*) is set by claim 5 here. The *exit apron cell*
is the target cell of a ground vehicle factory's exit track (S-0417 Terms and claim 3).

The *order-level destination* is the destination that behaviours and factory report tests see, as S-0212 takes it from S-0503 claims 15 and 16. *Point suppression* is claim 27's one-application skip of the next otherwise due nonempty point application; a *pending* suppression is one not yet consumed.

- **Drive vehicle.** An object of the vehicle type kind (glossary *vehicle*) that
  moves with the driving locomotor. In this spec "vehicle" means a drive vehicle, with these
  exceptions, which the claims themselves state:
  - claim 14 and its scenario D16 cover the types of every kind (infantry, vehicle and aircraft),
  - claim 16 and its scenario D18 cover every object of the vehicle type kind, whatever its
    locomotor,
  - claim 13 and scenario D12 run 5 also say which objects a speed power-up reaches (mobile
    ground objects of every kind, aircraft excluded), although only a drive vehicle's progress is
    this spec's subject,
  - and claim 1 is a rule of the type's speed field, which holds for the speed field of an
    infantry type too (S-0208 uses it).

  - claim 19 and scenario D21 cover vehicles with driving, ship and hovering locomotors,
  - claim 20 and scenario D22 cover every object, of any kind, that moves with the teleporting
    locomotor (S-0208 claims 19–20 own infantry landing-point choice and placement clearing),
  - claim 21 and scenario D23 cover teleporting vehicles whose own movement is temporarily
    replaced by driving,
  - claim 22 and D24 cover driving vehicles and own-locomotor teleports of any kind,
  - claim 24 and D27 cover heading-point heights for driving and ship vehicles,
  - claims 25 and 26, with D29 and D30, cover the separate locomotor destination point
    and order-level goal removal for driving and ship vehicles.

  - claim 27 covers destination refresh and the qualified teleport/dock transition for
    vehicles of any locomotor.
  - claim 28 and scenario D34 cover vehicles whose type has the teleport-capable flag,
    whatever their current locomotor.

  All other claims cover drive vehicles only. An infantry or aircraft type can also name the
  driving locomotor (claim 14), but how an object of such a type moves is outside this spec (Open
  questions).
- **Warp-pending immobilization.** The external-teleport commitment that begins when a
  superweapon warp is accepted, before its leaving stage, and ends when the object
  leaves its old position. It includes the leaving wait, not the arrival hold.
  S-0513 claims 34 and 39 own that warp's eligibility and timing. This is distinct
  from *phasing in*, the parasite-related *held* countdown and the restoration lock.
  An ordinary own-locomotor jump does not create this commitment (claim 27).
- **Restoration lock.** A hold that prevents a temporary driving locomotor from being
  released and its suspended predecessor restored. It is distinct from a stopped
  vehicle, a movement destination, power and the tether. It starts absent on a new
  driving locomotor. Claim 27 gives the other requirements for restoration.
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
  the *faster* veterancy ability at its current veterancy rank. Which rank grants which ability is
  S-0311's (claims 3 and 4); what the ability does to a vehicle's speed is claim 2 here.
- **Track.** A fixed list of points that a vehicle drives through to reach a **track target**, which
  is the centre of the cell one step ahead (straight and diagonal tracks) or two steps ahead
  (two-cell curves). Each point is a displacement (*x*, *y*) in leptons from the track target along
  the map axes, with a facing *f* in 1/256 of a turn. The vehicle's body faces 256 *f* facing units
  while at that point. A track ends at the track target itself. The tracks are engine constants
  (Constants, tables T1 and T2). This *track* is unrelated to S-0211's locomotion class *track*,
  which this spec always names with the words "locomotion class".
- **Move destination.** The point the vehicle's current move order leads to, in leptons. It is used
  for braking (claim 10) and for giving up a move (claim 15). When the destination cell is a
  high-bridge cell, its height is the deck's height. This is not a test for the
  presence of the separate locomotor destination point.
- **Locomotor destination point.** A world point in integer leptons accepted by locomotion
  as its movement goal, distinct from both the order-level destination and the active
  track's heading point. It can be present without a path or track, including while
  standing in its own cell. Claim 25 defines its lifetime for driving and ship vehicles.
  S-0314 claim 17 uses its presence in the depot movement report.
- **Moving destination.** In claim 26, S-0314 claim 17's combined condition: an
  order-level destination exists and the locomotor reports movement. For driving and
  ship locomotion a present locomotor destination point is sufficient for that report,
  even while the vehicle stands still.
- **Heading point.** The point a vehicle's locomotor is moving the vehicle towards at this moment,
  if any. For the driving locomotor it is the track target of the track the vehicle is on, from
  the tick its track begins until the tick that track ends; it moves one cell on with the track
  target when a curve is chained (claim 8). Its stored world height is the vehicle's world
  height when an ordinary track begins, retained through chaining as claim 24 specifies,
  not the terrain or deck height at the target. The factory exit track instead stores
  height 0 leptons (claim 24). A drive vehicle that stands still, turns in place
  before a step, waits (claim 15) or is between two tracks has no heading point. A vehicle with the
  teleporting locomotor has none either (claim 20). The ship locomotor has the same heading
  point as the driving locomotor (S-0504 claim 1). For the hovering locomotor it is the next
  cell centre chosen under S-0503 claim 5, with the height defined in S-0503 Terms. The
  heading points of other locomotors belong to their specs (Open questions).
- **Vehicle scatter centre cell.** The cell around which a vehicle that accepts a scatter request
  looks for its new destination (claim 16): the cell of its heading point when it has one, and
  otherwise its own cell. For a vehicle travelling through a tunnel (S-0212 Terms), the tunnel's
  exit cell takes the place of its own cell, as for infantry (S-0208 Terms *scatter reference
  point*). For a drive vehicle it is never the cell of the move destination unless the vehicle is
  on the track that ends there.
- **Slowdown distance** *D*ₛ, **acceleration factor** *a*, **deceleration factor** *η* and the
  **acceleration flag.** Fields of the vehicle's type (Import mapping), in leptons (*D*ₛ) and as plain
  numbers (*a*, *η*).
- **Passive flag.** A vehicle-type field. It changes two rules (claims 8 and 10).
- **Moving tick.** A tick in which the vehicle is on a track, or begins one, and is not prevented
  from moving (claims 6 and 12).
- **Sinking.** A vehicle is going down in water and stays in play (S-0312 Terms).
  S-0312 claim 10 gives the condition at a killing hit; S-0614 claim 11 gives culling,
  which starts sinking without a hit or health loss. Other possible sources remain in Open
  questions. S-0504 claims 4 and 5 own descent and behaviour while sinking: the locomotor
  is not processed, so claim 10's sinking case is never reached.
- **Locomotor field, class identifier.** The type field that names a type's locomotor (Import
  mapping). Its value is a *class identifier*: 32 hexadecimal digits in five groups of 8, 4, 4, 4
  and 12 digits, joined by hyphens and enclosed in braces, with no spaces. Import mapping table T3
  defines the eleven recognised input values by their numerical groups, not by program names.
- **Step check, first check, repeated check.** On each tick on which a vehicle is not on a track,
  has a current step and its body faces that step's 8-direction (claim 6), the entry outcome
  (S-0209 claim 5) of the cell its current step enters decides whether it begins a track on that
  tick (claim 15). That is its *step check*. The step check of such a tick is a *first check*. When a first check makes the
  vehicle search a new path at once (claim 15) and that search finds one, the vehicle checks the
  first step of the new path on the same tick. That check is the *repeated check*. The outcome a
  step check uses is the entry outcome with the substitutions of S-0209 claim 5 (part 5f).
- **Crate, speed power-up.** A *crate* is an overlay (glossary) that gives a power-up when it is
  collected. The *speed power-up* is the power-up that changes speed bonus factors (claim 13). The
  pickup, area, effects and feedback are owned by S-0520 claims 6, 20 and 26, with the
  own-locomotor teleport pickup specified in claim 20 e here.
- **Capture-the-flag game.** A game started with the capture-the-flag game option on. Whether such
  a game places flags that a vehicle can carry is an Open question.
- **Giving up the move.** A vehicle that gives up its move gets no track on that tick. When it has
  approach destinations, it stops and falls back to rest (S-0215 claim 13, which makes the first
  approach destination its destination). Otherwise its destination is removed, and what its
  behaviour then does belongs to S-0215.

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
   - Backing: `measurable` (scenarios D1, D6 and D12). The capture-the-flag halving is
     `unmeasured`: no stock skirmish setup is established that lets a vehicle carry a flag.
     D15 proposes a test conditional on resolving that precondition
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
   and in every other tick in which it has a move to make and is due to begin a track but does
   not, because it waits for a gate or its step check refuses (claim 15), also when it finished a track earlier in that
   tick. A tick that ends without a track only because claim 12 ends it there keeps the budget. A
   vehicle that comes to rest keeps whatever leftover it had, and that leftover is added to its first tick of progress if its next
   move begins without a turn in place.

   So a track of *n* points whose last point is *m* leptons from its target costs exactly
   7 (*n* + 1) − trunc(7 (1 − *m*/11)) progress units. That is **163** for a straight step
   (curve A of T1) and **227** for a diagonal step (curve B). Over a long straight run at a
   steady *s* well below 163, a vehicle covers one cell per 163/*s* ticks on average. A diagonal run
   takes 227/*s* ticks per cell. A vehicle whose *s* is at or above about one track's cost settles
   into finishing exactly one track per tick (claim 12).
   - Backing: `measurable` (scenario D1 for straight and diagonal costs; D7's track-table sweep
     for the other tracks; scenario D8, parts b to e, for the leftover
     kept at rest after an ordinary move and for the budget emptied by a turn in place and by a wait,
     part e for a wait on a tick in which the vehicle has just finished a track).
     Retention at rest reached by other arrival effects is `unmeasured`: D8 isolates only an
     ordinary move ending, and no scenario here distinguishes every other way of coming to rest
   - Confidence: high for the costs; low for the leftover kept at rest (the ways of arriving were
     read but not exercised)

4. **Position between points.** Whenever a vehicle's budget after this tick's spending (claim 3) is
   at least 1 and the next point *P* of its track is not the end, the vehicle is moved towards *P*.
   Its position at the end of the tick follows from its position *Q* after this tick's point
   purchases, which may itself be a position this rule gave it in an earlier tick, so the step
   compounds from tick to tick:

   | Where *Q*′ lies | Position at the end of the tick |
   |---|---|
   | In the cell of *Q* or the cell of *P* | *Q*′ |
   | In neither cell, with *A* > 3 | *P* |
   | In neither cell, with *A* ≤ 3 | *Q* |

   Here *Q*′ = *Q* + (trunc((*P*ₓ − *Q*ₓ) × *t*), trunc((*P*ᵧ − *Q*ᵧ) × *t*)) and
   *t* = *A* × (1/7), rounded toward zero to single precision (F4).

   Because *t* is always a little less than *A*/7 (F4), a displacement whose exact product would be
   a whole number comes out one lepton shorter; in particular *t* < 1 even when *A* = 7, so the
   vehicle does not reach *P* by this rule. For *s* = 2 on a straight track, the vehicle's
   displacements from its start over the first eight ticks are 3, 7, 10, 12, 16, 20, 21 and 25
   leptons, not 3, 6, 9, ….
   - Backing: `measurable` (scenario D8)
   - Confidence: high

5. **Height and facing on a track.** At each point the vehicle's ground position becomes the track
   target plus the point's displacement, and its body facing becomes 256 *f*. While it drives, its
   world height is set whenever it passes a point or reaches the track target (claim 3), and at no
   other time: it becomes the surface height *H* (S-0106, claim 2) at its new ground position, plus
   416 leptons (4 height levels, the deck height of the glossary's *bridge deck*) when the vehicle
   is on a bridge deck at that moment. *H* is the terrain's surface, never the deck's. Between points (claim 4), the
   facing and the height do not change, also when claim 4 places the vehicle on the next point's
   ground position without passing it.

   *Getting onto and off a bridge deck.* Whether a drive vehicle is on a bridge deck changes only
   when its ground position moves from one cell into another, by a point of its track or by claim
   4, and the change is in force from that move on, so a point that moves it into a new cell
   already takes the new state into its height:
   - it gets onto the deck when the cell it moves into is a high-bridge cell (S-0211 claim 15) whose
     height level is exactly 4 below that of the cell it leaves, which is the step from land at deck
     level onto the deck,
   - it gets off the deck when it moves from a high-bridge cell into a cell that is not one,
   - in every other move, among them a move between two high-bridge cells and a move into a
     high-bridge cell from a cell at any other height level (from the ground under the deck), the
     state is kept.

   S-0212 (claim 10) refuses every step between cells whose height levels differ by 2, 3 or more
   than 4, so the only drivable step into a high-bridge cell from a higher cell is a drop of exactly
   4 levels. So a vehicle that drives along a deck has world height *H* + 416 at every point on the
   deck, and a vehicle that drives under a deck stays off it. This state is what claim 9 (a) means by
   standing on a bridge deck. How a vehicle that is placed by the map, or that leaves a building,
   starts on a deck belongs to other specs (Open questions).
   - Backing: `measurable` (scenario D7 for the facing at the end of a curve and for the facing
     between points; scenario D5 runs 1 to 3 for the height on slopes, run 4 for the height on a
     deck and getting onto and off it, run 6 for a vehicle that drives under a deck from ground of
     the same height level). `unmeasured` for these parts, each for the reason given:
     - that the drop must be exactly 4 levels, not 4 or more: S-0212 claim 10 blocks every step with
       a larger difference, so no drivable step tells the two apart.
     - a move into a high-bridge cell from a cell 1 level higher or lower (state kept): it needs a
       deck over ground that a vehicle reaches by a slope step, and no test map is known that
       S-0211 claim 15 accepts as a deck over such ground.
   - Confidence: high for the ground position, the facing and the height set at each point; medium
     for getting onto and off a deck (read, not exercised) and for the facing and height left
     unchanged between points

### Starting a step

6. **Turn in place first.** A vehicle begins a track only when its body facing equals the current
   step's 8-direction exactly (8192 × *d* for 8-direction *d*). Otherwise it turns in place towards
   that facing, and makes no progress until the facing matches. The turn, its duration and the tick
   in which the track begins are S-0206's (claim 12). Its budget
   is held at 0 while it turns (claim 3). Its speed fraction is kept, so a vehicle that stops to
   turn resumes at the fraction it had.

   **Launching hold.** A driving vehicle that is launching (S-0301 Terms) does not begin
   its next track. It waits between tracks with its destination kept until launching ends,
   then resumes the ordinary step rules. A track already begun finishes before this hold
   applies, as for a ship (S-0504 claim 3). Precedence against concurrent waits is not
   established here.
   - Backing: `measurable` (scenarios D7 and D9; D7 run 3 for the speed fraction kept through the
     turn; D26 for the launching hold)
   - Confidence: high for turning and the ordinary launching hold

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

   In crush mode the vehicle takes the straight or diagonal track for *d*, whatever the next step
   is, and uses only the current step, just as when the next step equals the current one. Crush mode is used by the crushing rules of
   S-0210, among them the start of the crush-slowdown state (S-0210 claim 9), which claim 10 case 1
   reads.

   A two-cell curve is taken only when the step check (claim 15) of its first cell gives *clear*
   and the second cell gives *clear* too. When the second cell gives *moving blocker*, because it is
   occupied by a moving unit or being entered by another vehicle, the vehicle takes the straight or
   diagonal track into the first cell instead. The other outcomes of the second cell are claim 15's
   (table T5).
   - Backing: `measurable` (scenarios D7, including its track-table sweep, D9 and D10,
     D10 runs C to F for the cells that decide
     crush mode, run F for a vehicle in the first cell; D17 part g for the second cell)
   - Confidence: high for the table; medium for crush mode (which cells are checked was read but not
     exercised); low for the fallback when the second cell is a moving blocker

8. **No chaining of curves for ordinary vehicles.** Each two-cell curve has a *chain point* and
   names an *entry point* (table T1). A vehicle of a type with the passive flag set may switch,
   when it reaches the chain point, to the next pair's two-cell curve at that curve's entry point. It
   does so when the step after its curve turns again, that pair has a curve with an entry point,
   and the cell that step leads to gives *clear* or *moving blocker* (S-0209 claim 5). It then skips the centre of its current
   target, and the track target moves one cell on in the 8-direction of that step. A vehicle without the passive flag never does this: every curve runs to its target's centre. Whether any supplied rules set the passive flag is data-specific and is not asserted here. On the tracks of this spec, the speed
   fraction of a passive vehicle whose acceleration flag is on is changed by none of the rules of
   claim 10 (none of its six cases applies). Such a vehicle keeps the fraction it has: a vehicle
   that starts at rest with *ρ* = 0 (claim 11) never moves, unless it is a follower car to which a
   vehicle ahead of it in its car chain copies a fraction (claim 10), or a separate track of a
   building has given it a fraction (Open questions).

   **Quirk `quirks.drive_curve_chaining_passive_only`.** Default (original): only passive vehicles
   chain. Off: every vehicle chains as described.
   - Backing: `measurable` (scenario D9: the ordinary vehicle for the default, control 1 for the
     chaining itself and its chain and entry points, control 2 for a passive vehicle whose
     acceleration flag is on, control 3 for a chain refused because the cell cannot be entered,
     control 4 for a chain taken into a cell that gives *moving blocker*)
   - Confidence: high that ordinary vehicles never chain; medium for the details of chaining

### Speed fraction

9. **Target fraction at the start of each track.** When a vehicle begins a track (claim 7), its
   target fraction is set from the cell its current step enters. It depends on these parts:
   - (a) Land type *ℓ*. Let *λ* be the height level of the vehicle's current cell, plus 4 if the
     vehicle stands on a bridge deck (claim 5). If the entered cell's height level differs from *λ* by less
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

   Because part (c) compares terrain heights, a deck counts as the ground under it: a vehicle that
   drives from land at deck level onto a high bridge over lower ground gets the downhill factor for
   that track, and one that drives off the deck onto such land gets the uphill factor, although the
   deck itself is level. Between deck cells over ground of one height level the factor is 1.
   - Backing: `measurable` (scenarios D3, D5 and D6; S-0211 scenario L1; D5 run 3 for the zero
     fallback of part (d); D5 run 4 for the road on a bridge deck of part (a) and for the slope
     factors at the ends of a deck; D5 run 6 for the land type under a deck)
   - Confidence: high for parts (b), (d) and (e); medium for parts (a) and (c) (the reference
     heights were read but not exercised), and for the slope factors at the ends of a deck

10. **Updating the speed fraction.** The speed fraction is updated once in each moving tick, and
    that tick's progress *s* (claim 2) is earned at the updated fraction. At some track boundaries
    it is updated a second time (claim 12). Sinking vehicles receive no locomotor processing
    (S-0504 claim 5), so they neither progress nor update their speed fraction: case 3 below
    is unreachable, not a rule for continued travel while sinking. The ship locomotor uses
    the same speed-fraction rules (S-0504 claim 1), subject to its crush handling (S-0210 claim 9).
    Each update follows these rules:
    - **Acceleration flag off:** *ρ* = clamp(*ρ*ₜ, 0, 1).
    - **Acceleration flag on, vehicle not passive.** Let *d*ₘ = ⌊*r*(*n*)⌋ with
      *n* = Δ*x*² + Δ*y*² + Δ*z*², the distance in leptons from the vehicle to its move destination
      taken with S-0204's table square root (F5), not the exact root. From *n* = 16641 (129 leptons)
      on, *d*ₘ can be one lepton less than ⌊√*n*⌋ (S-0204 claim 5), so near the threshold the
      braking test *d*ₘ < *D*ₛ can hold one lepton earlier than with the exact root. When more than one condition in the table holds, the resulting fraction is the result in the earliest matching row, clamped to [0, 1]. When none of the conditions before “Steady” holds, the fraction remains unchanged.

      | Case | Condition | New fraction *ρ*′ |
      |---|---|---|
      | 1. Crush slowdown | the vehicle is in the crush-slowdown state (S-0210 claim 9) | min(*ρ*ₜ, 0.2); the target fraction also takes this value |
      | 2. Braking | *d*ₘ < *D*ₛ | max(*ρ* − *V* × *η*, *F*₀.₃) |
      | 3. Sinking (unreachable) | the vehicle is sinking | max(*ρ* − *V* × 0.0015ₛ, *F*₀.₁) |
      | 4. Accelerating | *ρ* < *ρ*ₜ | min(*ρ* + *a*, *ρ*ₜ) |
      | 5. Decelerating | *ρ* > *ρ*ₜ | max(*ρ* − *V* × *η*, *ρ*ₜ) |
      | 6. Steady | none of the above | *ρ* |

      When the crush-slowdown state begins and when it ends belong to S-0210 (claim 9), and this
      spec defers to it. In outline: a vehicle of movement zone *crusher-all* enters the state when
      it drives into a new cell on a track in crush mode (claim 7) and the ground layer of that
      cell holds a vehicle. On the ground, the driving vehicle itself counts, so a single vehicle
      in the path sets the state from the cell before; on a bridge deck, the driving vehicle does
      not count itself, and the state begins only if a vehicle stands in the ground layer under
      that deck cell. The state is held across tracks, stops and new orders until the vehicle
      next crushes an overlay. S-0210 owns the switches that change the layer and self-counting
      rules, and scenario K10 part d tests the bridge-deck case. Case 1 takes precedence over
      case 2 (braking), so the state also overrides the braking floor. The lowered target replaces the target fraction until the next track
      begins (claim 9), and every update while the state lasts lowers the new track's target
      again, so the vehicle runs at min(*ρ*ₜ, 0.2) for as long as the state lasts. It drops to that
      value at once when the state begins, and accelerates from it by case 4 when the state ends.

      Here *F*₀.₃ = 0.300000011920928955078125 and *F*₀.₁ = 0.100000001490116119384765625
      (single-precision 0.3 and 0.1 widened to double), and 0.0015ₛ = 0.00150000001303851604461669921875
      (single-precision 0.0015, exactly 6442451 / 2³²). The crushing cap 0.2 is the double nearest to 0.2.

      **Acceleration and deceleration are not symmetric.** Acceleration adds the flat factor *a* per
      tick. Deceleration subtracts *V* × *η*, the type speed times the factor, so faster types brake
      harder per tick for the same field value. The braking floor *F*₀.₃ also *raises* a slower vehicle to
      *F*₀.₃ once it is within *D*ₛ of its destination (`quirks.drive_brake_floor_raises`, Edge
      cases and quirks).
    - **Acceleration flag on, vehicle passive:** none of the six cases applies, and *ρ* is not
      changed by this claim. It keeps whatever it had (claim 8).
    - **Car chains.** A vehicle that has a follower car (glossary), whose acceleration flag is on
      and which is not passive, copies its own *ρ* to every follower car behind it in its car chain
      (its follower car, that car's follower car, and so on) each time this claim updates its own
      fraction (once or twice per moving tick, claim 12), also when the update leaves *ρ* unchanged
      (row 6). A vehicle whose acceleration flag is off, or which is passive, does not copy. The map
      makes the links (S-0009 claim 41), and S-0212 (claim 26) sends the cars after their leader.
    - Backing: `measurable` (scenarios D3, D4, D10, D13 and D14; D5 run 5 for case 5; D9 control 2
      for the passive rule; D14 control for a leader whose acceleration flag is off), except case
      3, which is `unmeasured`: locomotor suppression makes it unreachable. S-0504 N6
      tests the absence of horizontal travel while sinking
    - Confidence: high for the acceleration-off rule, for cases 2, 4 and 5, including the raise
      by the braking floor, for the table square root in *d*ₘ and for the passive rule (no case
      applies, and a passive vehicle does not copy); medium for case 1 (the speed rule; when the
      state begins and ends is S-0210's confidence) and for car chains; high that case 3
      is unreachable

    These rules hold on the tracks of this spec (T1 and T2). The factory exit track is
    S-0417 table T1; its speed-fraction rules are S-0417 claims 5 and 6. Other separate tracks
    that buildings use remain outside this spec (Open questions).

11. **Stopping.** When a vehicle's move is stopped or cancelled while it is on a track, its move
    destination is cleared, and its target fraction becomes min(*ρ*ₜ, *F*₀.₃). It finishes the track
    it is on and stops at that track's target. When a vehicle reaches a track target and has no further step, it
    comes to rest with *ρ* = 0. A vehicle at rest starts every move from *ρ* = 0, accelerating if its
    acceleration flag is on (unless an earlier case of claim 10 applies: a vehicle still in the
    crush-slowdown state, for example, starts at min(*ρ*ₜ, 0.2) at once).

    **Cancellation during a pre-step turn (Q-1160 question 2).** An ordinary attack
    order on an object removes the order-level destination at application (S-0213 claim 21).
    For a driving vehicle turning in place before its first track, this also removes
    its locomotor destination immediately (claim 25). There was no heading point during
    that turn, and the cancellation creates none. The old path steps do not authorize
    a track without a locomotor destination: the vehicle does not take the cancelled
    step when its body finishes turning. Cancelling the destination need not erase
    the old path steps themselves; their presence is not continued movement in
    S-0206's sense (claim 9).

    The cancellation itself neither stops the body's turn nor gives it a different
    target facing. With no later aiming or other facing instruction, the body completes
    its existing turn according to S-0206 claim 1, without displacement. Ordinary
    firing may instead re-aim the body towards the attack target under S-0310 claim 9,
    even before the queued *attack* becomes current. A later approach destination is
    new movement, not completion of the cancelled step (S-0215 claim 37).
    Behaviour readiness remains S-0215 claim 11's rule: the body turn can keep
    *attack* queued even though neither movement point remains. Destination clearing
    must not leave an independent pre-step movement hold waiting for a track to start.
    This case does not change the completion of an already begun track above.
    - Backing: `measurable` (scenario D11: part a for a move stopped on a track, part b for *ρ* = 0
      at rest and the next move from 0, part c for cancellation during the pre-step turn)
    - Confidence: medium for stopping on a track; high for the pre-step cancellation,
      absent heading point, retained body target and absence of a further step without
      a new destination. Order assignment and subsequent behaviour use S-0213 claim 21
      and S-0215 claims 11 and 37

12. **Ticks and track boundaries.** What a moving tick can do depends on whether the vehicle is on
    a track when the tick begins. A tick begins without a track when the vehicle starts moving from
    rest, has just finished turning in place, or finished a track at the very end of the previous
    tick (see below). In both cases the budget is spent as in claim 3.

    | Tick begins | Updates of *ρ* (claim 10) | Progress earned | When a track ends in the tick | Tracks finished, at most |
    |---|---|---|---|---|
    | without a track | one, towards the target fraction (claim 9) of the track the vehicle begins in this tick (claim 7) | *s* once, at the updated fraction | the tick ends there, with no track, and the leftover is kept | one |
    | on a track | one; and if the track ends and the next track begins in the same tick, a second, towards the new track's target | *s* once, at the fraction after the first update; nothing more after the second | if the move goes on and the vehicle faces its next step and its step check lets it begin, the next track begins in the same tick and the leftover is spent on it; if that track ends too, the tick ends there, with no track. A vehicle that must first turn in place, waits for a gate or is refused by its step check (claim 15) gets no second update and no new track in that tick, and its budget is emptied (claim 3) | two |

    **Arrival reports at completion.** Reaching the track target, including the target of a
    factory exit track, is a cell-arrival update (S-0417 claim 7). The report uses the completed
    arrival position with no current track or heading point, and its effects are visible before
    any next track begins in that tick. A tethered harvester's refinery arrival report and its
    possible refusal scatter are S-0402 claim 10's. That report precedes the cell-centre
    crushing of S-0210 claims 3 and 8; a mid-track cell-boundary slowdown can already have
    happened. S-0417 claims 7 and 8 own factory clear reports and their fallback scatter.
    A vehicle that chains a curve also has a cell-arrival update at the chain-point transition
    (claim 8); crossing a cell boundary mid-track or turning in place is not such an update.
    This does not give a refinery the chain-point report: S-0402 claim 10 requires track
    completion. Repair-depot approaches, including the docking-cell stopping rule at track
    completion and the distinction between the supplied docking point and the stopping
    position, are governed by S-0314 claim 18. That rule does not change these tracks.

    A vehicle at rest has *ρ* = 0, so with the acceleration flag on its first tick from rest gives it
    *ρ* = clamp(min(*a*, *ρ*ₜ), 0, 1) and it earns trunc(*s*₂ × *ρ*), unless an earlier row of claim
    10's table applies. For example, within *D*ₛ of its destination the braking floor gives
    *ρ* = *F*₀.₃ and trunc(*s*₂ × *F*₀.₃) (scenario D13). This does not apply while
    sinking, when no movement update occurs (S-0504 claim 5).

    Consequences: a vehicle that is accelerating (row 4 of claim 10's table in both updates) gains
    2*a* in a tick in which it reaches a track target and begins its next track. A vehicle whose *s* is at or above
    about one track's cost settles into finishing exactly one track per tick: each tick then finishes its track and the next one begins without a
    track. Its extra budget piles up unused, until it is lost at the next tick in which the vehicle
    turns in place, waits for a gate or is refused a track by its step check, or kept when it comes to rest (claim
    3). On straight and diagonal steps
    this caps a vehicle at one cell per tick.
    - Backing: `measurable` (scenarios D1, D3 and D13; S-0402 H14 for the completion
      report's timing, refusal scatter and ordering, S-0417 U8 for factory arrival reporting,
      D8 part e for the budget emptied by a wait
      after a track ended in the tick). The chain-point arrival update is `unmeasured`:
      D9 records chained geometry but no arrival-report effect, and S-0417 claim 7 has no
      established setup that produces a chained curve while tethered
    - Confidence: medium (the order of the two updates and the cap were inferred from reading, not
      observed; the cap was simulated); medium for completion-report ordering and the
      chain-point arrival update

    **Quirk `quirks.drive_track_end_cap`.** Default (original): as above. Off (a design behaviour,
    since the original has no such mode): the two rows of the table become one rule, for a tick that
    begins without a track and for one that begins on a track alike.
    - The speed fraction is updated once at the start of the tick's movement, towards the target
      fraction of the track the vehicle is on, or of the track it begins in this tick when it begins
      the tick without one.
    - It earns *s* once, at the fraction after that first update.
    - Each time a track ends within the tick, the move goes on, the vehicle faces its next step and
      its step check lets it begin, the next track begins in the same tick, the fraction is updated
      once more, towards the target fraction of that new track, and the leftover is spent on it.
      There is no limit on the number of tracks finished in a tick, so a tick in which *k* tracks
      begin after a track end gets 1 + *k* updates.
    - When, at a track end, the vehicle must first turn in place, waits for a gate or is refused by
      its step check, it gets no further update and no new track in that tick, and its budget is
      emptied (claim 3), as in the default. Otherwise the tick's movement ends only when the move
      ends at a track target, or when the budget left on a track buys no further point (claim 3).
    - A leader whose fraction is updated copies it to its follower cars at every one of these
      updates (claim 10, car chains).

    With at most one track end in a tick that began on a track, and none in a tick that began
    without one, the off rule and the default agree.

13. **Speed power-up.** When a speed power-up takes effect on a vehicle whose speed bonus factor is
    exactly 1.0, the factor is multiplied by the power-up's value. Otherwise it is unchanged, so a
    second power-up has no effect. The power-up is not limited to the vehicle that collects it: it
    acts on the mobile ground objects (aircraft excluded) within an area around the power-up's cell.
    S-0520 claim 20 owns the affected objects, power-up value and crate area (its Terms and
    F4), including the off-map behaviour and quirk of its claim 19. S-0520 claims 6 and 26 own
    pickup timing and feedback, subject to the own-locomotor teleport pickup in claim 20 e here.
    - Backing: `measurable` (scenario D12; run 5 for the area and for the exclusion of aircraft)
    - Confidence: high for the exactly-1.0 rule; medium that it acts on an area (read, not traced in
      detail)

### Which locomotor a type has

14. **The locomotor field.** Every infantry, vehicle and aircraft type has exactly one locomotor,
    chosen by its locomotor field (Terms, Import mapping). A value that is one of the class
    identifiers of table T3 selects the locomotor that T3 names. Letters among the hexadecimal digits
    may be in either case. The value is read whole, after leading and trailing blanks are removed,
    and a comma in it is not a list separator.
    - **Absent.** The type keeps the locomotor in force from an earlier rules layer. Before any
      layer gives one, a type has the *teleporting* locomotor, whatever its kind: an infantry,
      vehicle or aircraft type without the field moves by teleporting, not by walking, driving or
      flying.
    - **Not a class identifier.** A value that does not have the form of a class identifier (an
      empty value, a name, digits without braces, a wrong number of digits) is ignored, as if the
      key were absent: the locomotor in force is kept. The original gives no warning. Our importer
      reproduces this and gives an importer warning.
    - **An unknown class identifier.** A value of the right form that is not in T3 replaces the
      locomotor in force. The original then stops with a program error as soon as an object of that
      type is created (placed by the map, produced or otherwise made). **Our engine** treats such a
      value as not given (the locomotor in force is kept) and gives an importer warning. This is a
      departure (glossary *departure*, bad data), with no switch.
    - Building types read the field too, but a building never moves with a locomotor, so the field
      has no effect on buildings. Rules that change the locomotor an object moves with for a while
      (for example a weapon that seizes a vehicle) belong to the combat specs.
      A seizing warhead's locomotor field uses the same class-identifier parsing rule.
      S-0607 claim 22 owns its defaults, warnings and seize outcomes.
    - The locomotor is independent of the type's kind and of its locomotion class (S-0211) and
      movement zone (S-0212): an infantry type may drive, and a vehicle type may walk. How an
      infantry or aircraft object with the driving locomotor moves is outside this spec (Open
      questions); this claim only says which locomotor it has.
    - Backing: `measurable` (scenario D16, including the identifier and malformed-value controls).
      Text after the closing brace, including a comma, is `unmeasured`: parsing may depend on
      the host's text-to-identifier rules, and no reference-host check has yet been specified
      (Open questions). The building field's lack of effect is `unmeasured`: a stationary
      building cannot expose which movement method its unused field selects
    - Confidence: high for T3, for the teleporting default of every kind and for an ignored value,
      medium for the program error on an unknown identifier (read, not observed); low for a value
      with further characters after the closing brace (Open questions)

### A refused step (driving locomotor)

15. **What a driving vehicle does at each entry outcome.** The step check (Terms) decides whether
    the vehicle begins its track, and its outcome has the result of table T4. A vehicle whose next
    cell holds a closed gate whose owner is allied with the vehicle's house (the gate owner's view,
    as in S-0209 row VP9) tells the gate to open, and waits without a track, and without turning, on
    every tick until the gate is open, whether or not its body already faces the step. So the gate
    starts to open on the vehicle's first tick in front of it, and a vehicle that arrives facing
    another way starts turning towards the step (claim 6) only once the gate is open. A vehicle that
    waits keeps its stored path unless the table says otherwise, gets no track on that tick and
    makes a first check again on the next tick on which it faces its step. So while an obstacle stays, the reactions of a first
    check repeat on every such tick.

    **T4. The step check of the first cell.**

    | Outcome | First check | Repeated check |
    |---|---|---|
    | clear | it begins the track (claim 7; for a two-cell curve the second cell is checked as in T5) | as the first check |
    | closed friendly gate | it tells the gate to open and waits | as the first check |
    | moving blocker | S-0212 claim 22 | S-0212 claim 22 |
    | friendly obstacle, enemy obstacle | it discards its stored path and makes a fresh search at once, whatever its path retry timer shows (S-0212 owns the search and the timer). When the search finds a path, the repeated check follows on the same tick | it waits in every case. It takes as its target, and attacks by an override of its behaviour (S-0215 claim 10): the first object listed in the cell (S-0209 claim 13) when its house is not allied with that object (a terrain object is never allied), or, when no object is listed and the cell holds a wall, the wall, whatever its owner. When its house is allied with the first listed object, or when no object is listed and the cell holds no wall, it attacks nothing. The override gives no destination (below) |
    | stationary friendly | as for an obstacle: it discards its stored path, makes a fresh search at once, and makes the repeated check on the same tick | when the give-up test (F7) holds, it gives up the move (Terms). Otherwise it sends a scatter request with no threat point, forced and move-anywhere, to every object listed in the cell layer it drives on (below), and waits. Which of them receive it is S-0209 claim 11; what a vehicle among them does is claim 16, what an infantry unit does is S-0208 claim 11 |
    | cloaked blocker | every cloaked object in the cell's ground layer, of any house, starts uncloaking for every house, not just the mover's house (S-0516 claim 11 e). It discards its stored path. The fresh search that follows (S-0212 claims 20 and 21) is made on the same tick only when its path retry timer has run out, and then the repeated check follows | the same global uncloaking applies, and it gives up the move |
    | blocked | it discards its stored path and makes a fresh search at once, whatever its path retry timer shows (S-0212 claim 22). When the search finds a path, the repeated check follows on the same tick | it gives up the move |

    *The override of an obstacle's repeated check.* The override that makes the vehicle attack
    the first listed object or the wall gives it the behaviour *attack*, that object or the wall's
    cell as its target, and **no destination**. So by S-0215 claim 10 its move destination is put
    aside together with its old target, and from that tick on it has no destination: its move ends,
    it makes no further step check towards that destination, and the override is not made again
    on the following ticks. What the vehicle does in *attack* (approaching, firing) belongs to
    S-0215 and the combat specs. When the override is reverted (S-0215 claim 10, for example when
    the target is destroyed), the move destination that was put aside is the vehicle's destination
    again, and it drives on towards it (S-0212 claims 20 and 21 own the path it then searches).
    The same holds for the wall: the wall's cell is the target, and there is no destination.

    *The cell layer it drives on.* A request goes to the deck layer of the cell when the cell is a
    high-bridge cell and the vehicle's height level, trunc(*z* / 104) for its world height *z* in
    leptons as claim 5 sets it (*H* + 416 on a deck), with the division rounded toward zero (so for
    a negative *z* not a floor), differs
    from the cell's height level by 3 or more; otherwise it goes to the ground layer.

    *The first cell of a new path.* Whenever a fresh search (S-0212 claim 20) finds a path, the cell
    that the first step of the new path enters has these effects on the same tick, whatever the
    vehicle's facing, so before any turn towards the new step. Each row that holds applies.

    | The cell the first step of the new path enters | Effect |
    |---|---|
    | holds a closed gate whose owner is allied with the vehicle's house (gate owner's view) | the gate is told to open |
    | gives *stationary friendly*, the vehicle is not of a train type, the nearest object (below) belongs to a house allied with the vehicle's house, the give-up test (F7) holds and the vehicle is in contact with nothing (S-0209 claim 8 uses the same contact condition) | the vehicle gives up the move |
    | gives *stationary friendly*, the vehicle is not of a train type, the nearest object belongs to a house allied with the vehicle's house, and the give-up test (F7) fails or the vehicle is in contact with something | the vehicle sends the same scatter request as in T4 |
    | gives *stationary friendly*, and the vehicle is of a train type, or no object is found, or the nearest object's house is not allied with the vehicle's house | nothing |

    The nearest object is looked for in the cell's deck layer when the vehicle's world height *z*
    is more than 208 leptons (two height levels) above the surface height *H* (S-0106, claim 2) at
    the centre of that cell, that is when *z* > *H* + 208 in exact integer leptons, and in its ground
    layer otherwise. *H* is the terrain's surface, the same quantity as in claim 9 (c), not a
    bridge deck's height.

    *The nearest object* is chosen among the objects listed in that cell layer (S-0209 claim 13)
    as follows:
    - Only infantry units, vehicles, aircraft and buildings take part. Any other listed object (a
      terrain object such as a tree, for one) never does. No object is left out for being the
      vehicle itself; the vehicle is listed only in the cell it stands in, which its first step
      leaves, so in practice it never takes part.
    - Each one's distance is the distance *d*ₙ of Formulas F8: measured along the map only (heights
      play no part), from the cell's **top corner**, the corner with the least *x* and least *y*
      (S-0208 table P names it), to the object's position within the cell that holds that
      position. It is not measured from the cell's centre (`quirks.drive_nearest_from_top_corner`).
      For a building listed in this cell whose position lies in another cell, its position within
      that other cell is used.
    - The object with the smallest *d*ₙ is the nearest. When several share the smallest *d*ₙ, the
      one listed first (S-0209 claim 13) is taken.
    - When no object of those kinds is listed in that layer, no object is found, and the last row
      (nothing) applies, even though the cell gives *stationary friendly*.

    So among the five standing positions of infantry (S-0208 table P), the top position is nearest
    (*d*ₙ = 90 leptons), then the centre, where a vehicle resting in the cell also stands (181), then
    the right and left positions (202 each, a tie between the two), then the bottom position (271).
    The four values are the same with the table square root and with the exact one (F8).

    The step check then follows on that tick when the vehicle faces the step: a first check after a
    fresh search made because it had no stored path, the repeated check after a search made at once
    by T4 or after the search of T4's *cloaked blocker* row, and after the search of T5's obstacle
    row a check of the same kind as the T5 check that discarded the path (T5 below). So on a tick of a repeated check
    against stationary friends the vehicle can send the request twice, once here and once by T4,
    when the cell still gives *stationary friendly* at the repeated check. This is the default
    of `quirks.drive_duplicate_scatter_request` below. With it off, claim 15
    sends at most one request from this vehicle to the same cell layer in a tick; the
    repeated check still decides giving up, waiting and any other non-request effect.

    *The second cell of a two-cell curve* (claim 7) is checked only after its first cell gave
    *clear*, and only for a curve. Its outcome has the result of table T5.

    **T5. The step check of a curve's second cell.**

    | Outcome of the second cell | First check | Repeated check |
    |---|---|---|
    | clear | the curve begins | as the first check |
    | moving blocker | the straight or diagonal track into the first cell begins instead (claim 7); the stored path is kept | as the first check |
    | closed friendly gate | the gate is told to open; the vehicle discards its stored path and stops at once with *ρ* = 0 | as the first check |
    | friendly obstacle, enemy obstacle | it discards its stored path; a fresh search follows once its path retry timer has run out (on the same tick when it already has). When that search is made on this tick and finds a path, the look at the new path's first cell and a step check of the new path's first step follow on the same tick (below) | as the first check |
    | stationary friendly | as in T4: it discards its stored path, searches at once and makes the repeated check | when the give-up test of F7 holds (without the contact condition), it gives up the move (Terms); when it fails, it sends the scatter request of T4 to the second cell. In both cases it also discards its stored path and stops at once with *ρ* = 0, except that a vehicle that gives up and falls back to rest (it has approach destinations) may already have stopped by falling back |
    | cloaked blocker | as in T4 | as in T4 |
    | blocked | it discards its stored path and makes a fresh search at once | it gives up the move |

    *After T5's obstacle row.* When the fresh search of T5's *friendly obstacle, enemy obstacle*
    row is made on the same tick and finds a path, the look at the first cell of the new path
    (above) is made, and then, when the vehicle faces the new path's first step, a step check of
    that step follows on that tick. It is a first check when the T5 check that discarded the path
    was a first check, and the repeated check when that was a repeated check. On that step check
    the vehicle cannot begin a two-cell curve: when the outcome is *clear* it begins the straight
    or diagonal track into the new first cell, even when the new path's first two steps would form
    a curve, as after T5's *moving blocker* row. The limit holds for that one step check only: a
    search made at once by it (T4, first check) lifts it, and on later ticks curves are chosen as
    claim 7 says. When the vehicle does not face the new first step, it turns (claim 6) and makes no
    step check on that tick. Consequences: a vehicle can search twice in one tick (the search of
    T5, then, when the new first cell gives an obstacle or *stationary friendly* at a first check,
    the search made at once by T4); and when the search of T5 waits for the path retry timer, the
    vehicle stays at its cell with no stored path until a later tick.

    A train type sees every outcome other than *blocked* as *clear* at a step check (S-0209 part
    5f), so for a train only the *clear* and *blocked* rows apply. Unlike a walker (S-0209 claim 8),
    a vehicle's give-up test at a step check does not ask about contact; only the look at the first
    cell of a new path does.
    - Backing: `measurable` (scenario D17: part a for the request, part b for giving up, part c for
      a friend that refuses, part d for an enemy obstacle, part d2 for the wall target, part d3
      for the immediate fresh search after *blocked*, even with a running retry timer, part e for the gate before turning,
      part f for a cloaked blocker, part g for the second cell (its runs 1 to 4 for the T5 rows
      *stationary friendly*, *moving blocker*, *enemy obstacle* and *cloaked blocker*, run 5 for
      *closed friendly gate*, run 6 for *blocked*, run 7 for the search of T5's obstacle row and
      the straight track after it), part h for the cell layer of the request, part d also for the
      override's missing destination and the move resumed after the revert, part i for the
      nearest object: its distance from the top corner, infantry and vehicles taking part, and the
      tie).
      `unmeasured` for these parts, each for the reason given:
      - *blocked* at a repeated check of the first cell: the fresh search excludes blocked
        steps (S-0212 claim 8), so even an immediate search after a blocked first check need
        not repeat that outcome. No known setup makes the new first step become blocked
        between the search and its repeated check on the same tick.
      - an obstacle whose first listed object is allied with the vehicle's house: that needs an
        allied unit that counts as moving (S-0212 claim 27) to be listed first in a cell that also
        gives an obstacle, on the tick of the repeated check, and no setup is known that times this.
      - an obstacle in a cell that lists no object and holds no wall (no attack): S-0209 gives such
        a cell an obstacle only through its row VR10, while an infantry unit that the
        vehicle is not allied with has chosen its walking target in the cell but is not yet listed
        there. That state lasts only while the infantry unit walks in, and no setup is known that
        times it to the tick of a repeated check.
      - the layer in which the look at the first cell of a new path seeks the nearest object (deck
        or ground, by *z* > *H* + 208): the look's effects, giving up and the request, are also
        made by the repeated check that follows on the same tick, with the same give-up test and
        with the request sent to the layer of T4's own rule. Without a contact, the layer of the
        look shows only when the look gives up or sends the request before a turn that the vehicle
        would otherwise make first, or through the second request of the double request listed
        below. The first case needs a fresh search on a bridge deck whose new first step turns
        the vehicle away from its facing, towards a cell holding a stationary friend, and no setup is known that is sure to give
        that path.
      - the contact condition of the look at the first cell of a new path: no situation is known
        that gives a driving vehicle a contact while it stands in front of a stationary friend
        within the close-enough distance (scenario D17, switch note).
      - the double request: a second request shows only through an infantry receiver that accepts
        the first one, takes its draw (S-0208 claim 13) and finds no qualifying neighbour, so that
        it stays where it is and the cell still gives *stationary friendly*. Every other receiver
        that accepts gets a destination and so counts as moving at the repeated check (S-0212
        claim 27), a vehicle whose search finds no cell takes no draw (claim 16), and a receiver
        that refuses takes none either. That needs every neighbour of the cell to be closed to the
        infantry unit while the vehicle still has a path through the cell, and no setup is known
        that is sure to give both.
      - the step check after the search of T5's obstacle row when the T5 check was a repeated
        check: that needs a search on the same tick (made at once by T4) whose new path begins
        with a curve whose second cell gives an obstacle, and which cells the pathfinder routes
        through next to occupied cells belongs to S-0212, so no setup is known that is sure to give
        such a path.
      - that objects other than infantry, vehicles, aircraft and buildings never take part in the
        nearest object: the only listed objects of another kind that S-0209 names are terrain
        objects, and a terrain object gives the cell an obstacle or *blocked* (S-0209 row VP29 or
        VP26), never *stationary friendly*, so the look never runs over a layer that lists one.
      - aircraft and buildings taking part in the nearest-object choice, and a building's
        position within a different cell: D17 part i isolates infantry and vehicles only.
        No setup is known that lists an aircraft or building together with stationary friends
        without that object's own entry outcome preventing this look from running.
      - no object found while the cell gives *stationary friendly*: that needs the layer the look
        searches (by *z* > *H* + 208) to list none of those kinds while the layer that decides the
        entry outcome (S-0209) gives *stationary friendly*, and no setup is known in which the two
        layers differ in that way
    - Confidence: medium for T4 and for the gate opened before turning; high for the override
      giving no destination, medium for the move resumed after the revert; low for the look at the
      first cell of a new path, for the double request and for T5, except medium for how the
      nearest object is chosen (who takes part, the distance from the top corner and the tie) and
      for the search, step check and straight track after T5's obstacle row

### Scatter requests (every vehicle)

16. **When a vehicle accepts a scatter request, and where it goes.** This claim holds for every
    vehicle, whatever its locomotor. A vehicle refuses a scatter request, and does not move, when
    any of these holds:
    - (a) its current behaviour, or its queued behaviour when it has no current one, is *sleep*,
      *sticky* or *unload*, or its type is a train type,
    - (b) its locomotor is the teleporting locomotor,
    - (c) the request is not forced and its current behaviour lacks the scatter property (S-0215
      claim 19 owns this gate),
    - (d) its body is turning (S-0206),
    - (e) it has a destination and the request is not move-anywhere,
    - (f) it is deployed, deploying or undeploying (a vehicle type that deploys in place),
    - (g) its locomotor is switched off (the specs of the effects that switch a locomotor off own
      when that happens); for a hovering vehicle, its locomotor is not powered in S-0503's sense
      (S-0503 Terms: switched off and down at a hover height of 0), so a switched-off hovering
      vehicle still above the ground is not refused by this case (S-0405 claim 20, *Movement and
      turning*, which states the same reading). The hovering reading is `unmeasured` and rated
      low, with the rest of case g (backing, below).

    A request **without a threat point** that is not refused takes no random draw. The vehicle's
    destination becomes the cell that the nearby-cell search of S-0212 (claim 25) gives around its
    vehicle scatter centre cell (Terms), in the same use as an infantry scatter (S-0212 table T5, row *Scatter*), with the
    vehicle's own locomotion class. Its behaviours are not changed: no *move* is queued, so it drives
    there in the behaviour it has. When the search gives no cell, it does not move. A vehicle that
    had a destination (it accepted a move-anywhere request) loses that destination to the scatter
    cell.

    A request **with a threat point** is refused in these further cases: (h) its current behaviour
    has the paralysed property (S-0215 claim 17 owns this refusal), (i) it is unloading passengers,
    (j) it has a destination and its turret is turning (a move-anywhere request). When (a) to (j)
    pass and the vehicle has a target and the request is not forced, it takes one ranged draw from
    the scenario random stream over 1 to 4 and refuses unless the draw gives 1. When it accepts, its
    first-choice direction is *d* = (*d*₀ + *j* − 1) mod 8, the residue from 0 to 7, in exact integer
    arithmetic, where *d*₀ is the facing from the threat point to the vehicle's own position rounded
    to the nearest of the eight step directions (S-0208 Formulas F6) and *j* is one ranged draw from
    the scenario random stream over 0 to 2, taken after the draw over 1 to 4 when both are taken.
    The candidates are the eight neighbours of its vehicle scatter centre cell in the step directions *d*,
    *d* + 1, …, *d* + 7 (mod 8), in that order of preference. A candidate qualifies when it lies
    inside the usable area and the vehicle's entry outcome for it is *clear* (S-0209 claim 5). Its
    destination is the first qualifying candidate that passes the screen-cover check (S-0208 Terms)
    and is not a high-bridge cell, or else the first qualifying candidate. It queues *move* without
    an immediate start (S-0215 claim 7) and then takes that destination. When no candidate
    qualifies, it does not move.

    Consequences:
    - The scatter order (S-0213 claim 9: forced, no threat point, not move-anywhere) moves an idle
      vehicle that stands still, is not turning and is not in *sleep*, *sticky* or *unload*, to the
      cell the nearby-cell search gives. On open flat ground with no other unit within 2 cells that
      is the neighbour at position (*N* mod 8) of the search's collection order, *N* being the tick
      number of the request (S-0212 claim 25). A vehicle that has a destination refuses the order.
    - The request of a blocked vehicle (claim 15) and the crowding requests of S-0209 claim 12 are
      move-anywhere, so a vehicle accepts them even when it has a destination.
    - A refused refinery arrival report sends a forced, no-threat-point request that is not
      move-anywhere (S-0402 claim 10). A receiver with a destination refuses by case e.
      otherwise an accepting receiver searches around its own arrival cell because its
      completed track has no heading point. No behaviour is queued and no scatter draw is taken.
    - The scatter off a building of S-0215 claim 28 and the scatter from damage of S-0215 claim 19
      are forced and have no threat point, so they follow the first case above.
    - Factory apron clearing during unloading or while idle (S-0417 claims 4 and 13) sends forced, move-anywhere requests to every
      object listed in the selected ground layers, including moving receivers. In the exit apron
      cell the request has no threat point: an accepting vehicle takes no draw, uses the
      nearby-cell search above and does not queue a behaviour. In an occupied apron neighbour
      the threat point is the exit apron cell's centre: an accepting vehicle takes one draw over
      0 to 2, no draw over 1 to 4, and queues *move* to the candidate chosen above. Forced and
      move-anywhere bypass only cases c and e, not the other refusals; case j still applies to
      a neighbour's threat-point request.
    - The requests with a threat point that reach a vehicle are sent by the bunker's clearing
      (S-0606 claim 6, S-0215 claim 17) and an aircraft's firing pass (S-0501 claim 16), both forced
      and not move-anywhere, and by factory apron-neighbour clearing (S-0417 claims 4 and 13),
      forced and move-anywhere. A crusher's warning goes to infantry, with no threat point (S-0210
      claim 7).
    - **A seized vehicle** (S-0607 Terms) has no refusal of its own here. Cases a to j and the draw
      over 1 to 4 apply to it as to any vehicle, read on its state at the request: its behaviour
      is the one the seize gave it (S-0607 claims 5 and 6) or a later one, and cases b and g read
      the locomotor it has at that moment. That is the stand-in that the seize installed (S-0607
      Terms) while it has one, and its own locomotor once a drop-pod stand-in's descent has ended,
      although it then stays seized (S-0607 claim 24).
    - While a seized vehicle has a destination, case e refuses every request that is not
      move-anywhere, so of the senders above only factory apron clearing can reach its acceptance.
      A gripped unit normally has one: the seize gives it the lifter as its destination (S-0607
      claim 5), which becomes the cell of the movement point that the seize fixes (S-0607 claim
      7). A seized vehicle with no destination is not refused by case e, so every sender above can
      reach its acceptance. That covers a gripped unit left with none (S-0607 claim 9) and a seized
      vehicle that is no longer gripped (a seized unit is not always gripped, S-0607 Terms), whose
      destination normally becomes none when its grip ends (S-0607 claims 13 and 17). When a
      seized vehicle accepts:
      - a request without a threat point takes no draw and queues no behaviour, and the cell that
        the search gives is not taken as its destination (S-0607 claim 9). Its destination, or its
        lack of one, is unchanged.
      - a request with a threat point takes its draws as above, and its candidates are judged
        around its vehicle scatter centre cell. For a gripped unit with the jumpjet stand-in, that
        is the cell of its heading point, the movement point that the seize fixed (S-0607 claim
        7), not the cell it is flying over. When a candidate qualifies, *move* is queued without
        an immediate start, as above, and the candidate is not taken as its destination (S-0607
        claim 9): its destination, or its lack of one, is unchanged. When none qualifies, nothing
        is queued.
    - **The queued *move* of a seized vehicle.** Unless a later request replaces it in the queue,
      it becomes current at the vehicle's next per-tick start at which it is ready to change
      (S-0215 claim 11). Its processing is a vehicle's ordinary *move* processing (S-0215 claim
      22), which makes no exception for a seized unit and neither gives nor clears a destination.
      - *Current while the vehicle is still seized,* in its grip or in the fall and landing after
        it: the processing does not move it. With a destination, it only chooses the standard
        delay, and the vehicle goes on as S-0607 moves it (the pull of claim 7, the fall and
        landing of claims 13 to 17). With no destination, while its movement is not moving and
        nothing is queued, it falls back to rest (S-0215 claim 22). In the grip that fall-back has
        the effects that S-0607 claim 9 gives for a fall-back in the grip, a destination it would
        give is ignored, and it does not restore the unit. A vehicle with the jumpjet stand-in that
        is set down and whose movement has stopped is restored at that fall-back by S-0607 claim
        20's test, subject to that claim's exclusion of a hovering set-down unit. The processing
        never ends a grip: a grip ends at a rule that gives the unit no destination
        while it has one (S-0607 claim 9), and the processing gives none. Like the queued *hunt*
        of S-0607 claim 6, this *move* cannot make a seized vehicle move under its own power.
      - *Current after restoration* (S-0607 claim 20), when the vehicle is no longer seized and has
        its own locomotor back: it normally has no destination then, since the end of its grip
        gave it none (S-0607 claims 13 and 17) and as a seized unit it took no new one (S-0607
        claim 9). The candidate never became its destination, so the vehicle does not drive
        towards it. Its first *move* processing finds it with no destination and not moving, and,
        unless something is queued, it falls back to rest and chooses a delay of 1 tick (S-0215
        claim 22).
      - So a seized vehicle that accepts a request with a threat point takes the draws and has
        its behaviours changed, but it does not move towards the chosen candidate.
    - The search and the eight neighbours are centred on the vehicle scatter centre cell (Terms),
      never on the cell of the move destination as such. A drive vehicle that stands still, turns in
      place or waits between tracks searches around its own cell; one that is on a track searches
      around the cell of that track's target, the cell one step ahead (two for a two-cell curve).
      The blocked-step and crowding senders (S-0209 claims 5, 8 and 12, claim 15 here,
      S-0212 claim 27) normally reach receivers that do not count as moving. Factory apron
      clearing has no such restriction. A moving drive vehicle that accepts that request searches
      around its track target, not necessarily the apron cell that lists it. The track-target case
      also arises without a destination, for example while it finishes its track after a stop order.
    - Backing: `measurable` (scenario D18: part a for the search without a draw and the behaviours
      left unchanged, part b for case e, part c for case a, part d for case b, part e for the
      threat-point case and its direction draw, part f for case d, part g for case f; scenario D17
      part a for a move-anywhere request, part c for the refusal in *sticky*; D18 part h for case
      h; D18 part i for the vehicle scatter centre cell on a track and at rest; D18 part j for
      factory clearing, including case j and moving receivers; S-0402 H14 for the refused
      refinery-arrival sender; S-0417 U5 and U11 for the unloading and idle senders of the two clearing
      request kinds). `unmeasured` for these parts, each for the reason given:
      - case c: every sender of a request to a vehicle whose options are known sends a forced one
        (S-0209 claim 12, S-0215 claims 19 and 28, S-0417 claims 4 and 13,
        S-0402 claim 10, claim 15 here), so no setup is known that sends
        a vehicle a request that is not forced.
      - case g, including its reading for a hovering vehicle: the effects that switch a
        locomotor off are stated elsewhere (S-0405 claim 20, S-0503 claims 8 and 10 to 11, S-0314
        claim 21), but no known input makes a scatter request reach a vehicle while its locomotor
        is switched off, or reach a switched-off hovering vehicle in the ticks before it touches
        down, so no setup can yet be defined (as S-0405 claim 20's backing says for that state).
      - case i: no setup is known that times a threat-point request to a vehicle unloading
        passengers while it passes the earlier refusals.
      - deploying and undeploying in case f: D18 part g isolates the fully deployed state,
        not the transition ticks; no fixture here establishes a request during either
        transition while every earlier refusal gate passes.
      - the draw over 1 to 4: every identified threat-point sender (the bunker rule of S-0215
        claim 17, an aircraft's firing pass of S-0501 claim 16 and factory apron-neighbour clearing
        of S-0417 claims 4 and 13) sends forced requests. None reaches this non-forced acceptance
        draw
      - *move* queued without an immediate start, for every vehicle: the end-of-tick record shows
        the difference only for a receiver whose turn in that tick came before the sender's
        (S-0203), and no setup here fixes that order
      - a seized vehicle, gripped or not, and what its queued *move* does once current: a gripped
        unit normally holds a destination, so only a move-anywhere request reaches the
        acceptance, and the only such sender with a threat point lists the objects in a factory's
        apron cells, where no setup here holds a gripped unit. No setup here leaves a seized
        vehicle with no destination where a bunker's clearing or an aircraft's firing pass reaches
        it. So no setup here sends a scatter request to a seized vehicle at all, and its refusals
        by case e are not observed either
    - Confidence: medium for cases a to f, h and i, for the search without a draw and for the
      behaviours left unchanged, and for the threat-point case and its draws (read; known senders
      are S-0215 claim 17, S-0501 claim 16 and S-0417 claims 4 and 13). Medium for *move* being
      queued without an immediate start. Medium for a seized vehicle: no refusal of its own, the
      locomotor read by cases b and g, the scatter centre of a gripped unit, *move* queued before
      the destination is refused, the draws taken, and what the queued *move* does once current,
      both while the vehicle is still seized and after its restoration. Low for cases g and j.
      High for the vehicle scatter centre cell of a drive vehicle (the heading point was read
      wherever it is set and cleared), including the ship equivalence of S-0504 claim 1. Medium
      for the tunnel exit.
      The hovering heading point is defined by S-0503 Terms and claim 5, with medium confidence

### Engine defaults of the speed-fraction fields

17. **Precision of the engine defaults of the acceleration and deceleration factors.** A vehicle
    type that no rules layer gives an acceleration factor has *a* equal to the double-precision
    number nearest to 0.03, and one that no layer gives a deceleration factor has *η* equal to the
    double-precision number nearest to 0.002 (Constants give both exactly). Neither is a
    single-precision value widened to double: the engine holds both defaults in double precision
    from the start. A factor that a layer does give is parsed as the Import mapping says, so a type
    whose rules write 0.03 can end up with a different *a* from a type that leaves the key out
    (S-0003, claim 6, gives the parsed value). The engine default of the slowdown distance *D*ₛ is
    the integer 500 leptons.
    - Backing: `measurable` (scenario D19: the tick counts for the values 0.03, 0.002 and 500, and
      the speed fraction read by the recorder for the precision). The precision shows only in the
      speed fraction itself: the nearest double and the widened single value of each factor differ
      by less than 10⁻⁹, which changes no tick count in D19
    - Confidence: high

### Speed used to lead a shot

18. **A driving target's speed for lead.** For the lead of S-0306 claim 3 and F6, a drive
    vehicle supplies the integer given by F2, including the country factor, speed bonus,
    faster ability and flag-carrier halving. The speed fraction is the fraction the target
    holds when the shot is aimed, not a fraction predicted for its next movement update.
    This is a speed estimate: its numerical value is treated as leptons per tick by the
    lead formula, although F2 earns that many progress units per moving tick. There is no
    conversion by 256/163, 11/7 or the diagonal track cost, and no measurement of the
    displacement in the previous tick. Track points, interpolation, leftover budget and
    the track-end cap do not directly change this estimate.

    Turning in place or waiting does not by itself replace the estimate by zero. With a
    retained fraction it can be nonzero even though the position did not change in that
    tick. A turn before the first track from rest has fraction zero, and therefore estimate
    zero; a turn between tracks retains its fraction (claim 6). A wait uses whatever fraction
    is still held: a stop that sets it to zero (claims 11 and 15, T5) gives estimate zero.
    Whether lead is applied at all remains the separate moving-locomotor test of S-0306.
    This claim does not make a stationary target qualify for that test. It also does not
    update the target's fraction or earn progress when a shot is aimed. The estimate need not
    equal the progress earned earlier or later in the same tick: a movement update can change
    the fraction, and a turn or wait can earn no progress while the estimate remains nonzero.
    - Backing: `measurable` (scenario D20, including its factor controls). Flag-carrier halving
      in the lead estimate is `unmeasured`: no setup is yet known to make a skirmish vehicle
      carry a flag (D15's precondition remains open).
    - Confidence: high for the estimate and rounding; medium for its use during a wait

### Approaching a target (driving, ship and hovering locomotors)

19. **What an approaching vehicle does once its target is in range (Q-0072, question 3).** This
    claim holds for a vehicle whose locomotor (claim 14) is the driving, the ship or the hovering
    locomotor and that has both a target and a destination, for example the destination given by
    the approach step of S-0215 claim 37 or by S-0307's search for an approach position. "In range"
    is the range test of S-0215 claim 37 b, from the vehicle's current position, with the weapon
    chosen for the target. The *stored path* and its *steps* are S-0212's (claim 20), and the
    *retry delay* *R* is S-0212's (claim 21 and F3).
    - (a) **Arrival-time stop for the driving and ship locomotors.** At a track target, a driving
      or sailing vehicle clears its destination and discards all remaining path steps when:
      - its target is an infantry unit, vehicle or aircraft, not a building, cell, overlay or
        terrain object,
      - its current behaviour, or its queued behaviour when no current one exists, is *attack*,
        *area guard*, *hunt* or *rescue* (the last is an out-of-scope team behaviour),
      - it has no approach destinations (S-0215 Terms), and
      - the arrival range test passes. For a type that is not open-topped, this is S-0307's
        weapon range test with the weapon selected by S-0308, from S-0307 claim 8's source
        point to the target's cell centre at terrain surface height, as for a cell target
        (S-0307 claim 7), not to the target's own reference point. For an open-topped type,
        it is instead trunc(*r*(Δ*x*² + Δ*y*² + Δ*z*²)) < ρ, measured between the target's and
        vehicle's reference points, with *r* and its floating-point domain and rounding as
        in F5 and ρ the actor's weapon range of S-0307 claim 13, both integers in leptons.

      The vehicle keeps its target and stops at that track target, without beginning a further
      track. A two-cell curve finishes at its target, not at its intermediate cell. Coming into
      range between track targets does not stop it between them. This stop does not wait for a
      behaviour processing. If any condition fails, range alone does not clear its destination:
      it continues under the ordinary movement rules. Whether and when it fires on the way
      belongs to the combat specs (S-0301, S-0310). Separately, a vehicle whose type has the
      deploy-to-fire flag
      deploys, instead of firing, once the firing check answers *can fire* or *not facing*, and
      gets *must deploy* where its cell does not allow it (S-0310 claim 9 and its table of
      replies); deploying ends its move as the deploy specs say. **Original only:** a
      vehicle of a computer-controlled owner whose type deploys into a building with the
      tick-tank property also deploys on those answers when its target is a vehicle and its cell
      allows it; in our engine no player house is computer-controlled (S-0215 Terms).
    - (b) **No drop of the target at the end of a move.** A vehicle of this claim whose move ends
      with its target out of range keeps its target (no rule of the movement side drops it). What it does next is its behaviour's (S-0215
      claim 37: the approach step at the next processing, or the per-tick rule of part a there
      for a vehicle without a destination). The original also contains a rule that drops the target
      of such a vehicle, but only for a vehicle that stands still while keeping a destination right
      after a failed search for a path. That state does not arise for these locomotors: a failed
      search for a path removes the destination of a vehicle with the driving, ship or hovering
      locomotor, also within 1 cell of the destination (in the original, the give-up rule for
      computer-controlled owners gives a new destination instead, which sets the vehicle moving
      again). For all three locomotors this is S-0212 claim 23. No rule is specified for the drop.
    - (c) **Hovering locomotor: the stored path is discarded at each step point while in range.**
      Whenever a hovering vehicle that is moving and has at least one stored step chooses its next
      step (when it starts a move, and each time it reaches the point it was heading for, the
      centre of its next cell), and its target is in range, all its stored steps are discarded.
      This discard alone keeps its destination; part d gives the subsequent arrival-time stop.
      Exception: when its destination is a vehicle or an infantry unit,
      the hovering locomotor first makes the same chase cut as the driving locomotor (S-0212
      claim 20: the stored path is cut to its first ⌊*D* / 256⌋ steps, with *D* as there); when
      that cut is made (⌊*D* / 256⌋ < 24), the range is not tested at that step.
    - (d) **Hovering locomotor: what follows the discarded path.** With no stored steps left and
      not yet arrived, the vehicle makes a fresh search for a path to the same destination
      (S-0212 claim 20), but only once the retry delay *R*, counted from the last search the vehicle
      made, has run out. For the hovering locomotor every such fresh search starts *R*, whether it
      found a path or not.
      - If *R* has already run out, the search is made in the same tick and a found path lets the
        vehicle take its next step at once: the discarded path has no visible effect.
      - If no next step is chosen at the end of this transition, the hovering vehicle applies
        the arrival-time stop of part a, with the same target, behaviour, empty approach-list
        and arrival-range conditions. When those hold, it clears its destination and keeps
        its target. A positive travel distance already computed for this tick may still
        produce a final step (S-0503 claim 7); no later travel tick occurs until a new move. In particular, a still-running *R* leaves no next step and
        therefore triggers this stop when those conditions hold. It does not wait for *R* to
        expire in that case.
      - When the arrival-time stop does not apply, for example against a building, a running
        *R* instead leaves the vehicle waiting with its destination kept. It searches on the
        first tick on which *R* has run out, and resumes when a next step is chosen. At the
        next point the same path-discard and delay rules apply while the target stays in range.

      A hovering vehicle that chooses a next step on the same transition does not apply the
      arrival-time stop there. Thus with an expired *R* and a successful search it can continue
      towards an in-range mobile target; when a subsequent transition leaves no next step,
      part a's conditions decide the stop. A search that fails has S-0212 claim 23's outcomes.
    - (e) **Movement versus approach processing.** S-0215 claim 37 e keeps a held destination
      at its processing while the target is in range. That rule governs only the approach
      step, not the arrival-time clearing in parts a and d. The two range tests also differ:
      the ordinary arrival-time test measures to the target cell centre, while the approach
      step measures to the target's own reference point. S-0208 claim 17 gives the same
      arrival-time conditions for an ordinary walker, whose walk ends on that arrival tick.
    - Backing: `measurable` (scenario D21: runs 1 and 2 and their controls for part a, runs 3,
      3c and 4 for parts c and d, and every run for part e). Part b is `unmeasured`: under our
      rules the state in which the original's drop would act does not arise, so there is nothing
      to observe. Part c's chase exception is `unmeasured` (D21 says why). The *area guard*
      and *hunt* eligibility cases of part a are `unmeasured`: D21 establishes the retained
      target/destination fixture only under *attack*, not those behaviours. The out-of-scope
      *rescue* case and original computer-only deployment aside are also `unmeasured`:
      no qualifying skirmish fixture for those cases is supplied here. The deploy-to-fire
      rule is a pointer to S-0310 claim 9 and uses that claim's backing and confidence.
    - Confidence: medium for the arrival-time stop, its conditions and the hover distinction
      between choosing a next step and having none; medium for parts b, c and e (read, not
      observed); low for the exact tick of the search after a retained-destination wait in part d

### Teleporting locomotor

20. **How an object with the teleporting locomotor moves.** This claim holds for every object that
    moves with the teleporting locomotor (claim 14), whatever its kind, except where it names
    infantry. It covers only the object's own move to a destination. A superweapon or other effect
    that teleports an object belongs to the superweapon and combat specs, even when it uses the same
    locomotor.

    - (a) **Taking a destination.** Vehicle destination assignments, whatever the locomotor,
      obey S-0517 claim 9: a deployed vehicle or one deploying or undeploying receives no
      destination, except while still deployed and carried by another object's locomotion.
      Landing for deploy is not such a transition; destination-driven cancellation of jumpjet
      landing belongs to S-0502 claim 16 (see also S-0517 claim 8).
      For a vehicle whose type has the teleport-capable flag, claim 28 first decides whether
      teleporting movement receives the assignment at all: outside claim 28's three
      exclusions, only a qualified docking assignment does, and claim 28 lists the ones that
      arise in ordinary play.
      When the object is given a destination (its behaviour or an order sets one, S-0215),
      subject to those restrictions, the locomotor takes it at once, unless a pending suppression (claim 27) stops it. It drops the destination and does
      nothing, when the object is disabled by an electromagnetic pulse, is *held*
      (S-0614 Terms), is *being erased* or is *phasing in* (S-0214 Terms). This drop empties the
      order-level destination and happens only when the locomotor is offered the destination:
      a pending one-application point suppression (claim 27) means it is not offered, so a suppressed
      assignment is never dropped by this rule. A pending suppression is the only exception to the drop and to the fixing of a landing point: under one, whatever the object's state, the destination is kept and no landing point is fixed (claim 27, Q-1130 table, second row). Parts (e), (f) and (h) below, the State and ordering table and the
      open questions state that exception where it applies. Otherwise it fixes its
      **landing point** *L* at once (parts b and c). For a vehicle, the vehicle-present
      mark of the occupancy record (S-0209) moves at once from the cell of the object's old point
      to the cell of *L*. For a vehicle this reservation is in the **ground record**, even
      on a high-bridge cell and even when part b's nearby search uses deck mode. The
      reservation uses *L*'s chosen height *H*, not the object's eventual deck height.
      Before the jump, a driving vehicle approaching under a level high bridge reads that
      ground record, while one approaching on the deck reads the deck record (S-0209
      Terms). With no other occupant or reservation, the former therefore meets a
      *moving blocker* from S-0209 claim 5 row VR5, while the latter meets no blocker
      from this reservation. Other terrain, occupants and exceptions still belong to
      S-0209 and can give a different final outcome.
      The old mark is cleared before part (b)'s entry test is made, so a vehicle that fixes a
      landing point again in a cell where it already holds that reservation does not meet its
      own reservation there (claim 28, *One jump*).
      Clearing an earlier reservation likewise uses its remembered point and height:
      a remembered non-infantry landing point at *H* clears the ground record. Before any
      landing point has been remembered, the vehicle clears at its own position; a point
      at least 416 leptons above the terrain surface clears the deck record, otherwise
      ground. These comparisons are exact integer comparisons in leptons.
      An aircraft instead moves the object-present mark, not a vehicle-present
      mark. Its reservation at *L*, whose chosen height is *H*, is in the ground layer, even
      when that cell carries a high bridge. Its old-point mark is read in the deck layer only
      on a high-bridge cell when the old point's height is at least *H* + 416 leptons there
      (an exact integer comparison), otherwise in ground. Infantry reserve the chosen standing position (S-0208 claim 19). The old
      point is the previous remembered *L*, including after a completed jump, and otherwise
      the object's own position. Finishing a jump does not erase that remembered point. The object stays listed in its old cell until the
      jump. From then until the jump it counts as moving (S-0212 claim 27), and the point it
      reports as its locomotor's goal is *L*.
    - (b) **The landing point of an object that is not infantry.** When the object's entry outcome
      for the destination cell (S-0209 claim 5 for vehicles, claim 14 for aircraft) is *clear*,
      *L* is the centre of that cell, at height *H*. A vehicle asks at the destination cell's
      height level plus 4 on a high-bridge cell, and at its ground level elsewhere. This
      initial test is independent of the mover's own height and of the search's deck mode:
      a vehicle on ground can therefore find an occupied deck refused while the ground
      underneath is free. An aircraft's entry outcome is *clear*, so it does not take the
      fallback search merely because the cell is occupied. Otherwise *L* is the
      cell that a nearby-cell search (S-0212 claim 25) gives with these choices:
      - centre: the destination cell,
      - locomotion class and movement zone: the object's own (the zone is not simplified),
      - reach region: that of the destination cell for the object's own zone, read for its
        deck in deck mode; when that cell lies in no reach region, there is no reach-region
        restriction on candidates,
      - deck mode: when the destination cell is a high-bridge cell and the object's world height is
        more than 312 leptons (3 height levels) above the surface height *H* at the destination,
      - no height condition, no further test, no no-overlay condition, bridge cells allowed,
      - no reference cell, so the pick uses the landing-point fixation tick number
        (S-0212 claim 25), not a later jump tick.

      A destination in no reach region does not by itself fail the search. The ordinary
      class, occupancy and usable-area tests of S-0212 claim 25 still apply, and its collection
      order and tick-number pick choose among the qualifying cells, even in different reach
      regions. For example, an unenterable water destination of a ground vehicle can give a
      free bank cell; if none qualifies within the search radius, the search gives no cell.

      *L* is the centre of the chosen cell, and its height is *H* there. The point within the
      destination cell does not matter. No path is searched, so the object can land beyond cliffs,
      across water, on another island or at any height. Two cases leave *L* unchanged: the
      object has warp-pending immobilization (Terms), or the search gives no
      cell. Then, when *L* is still the object's own position or no *L* was ever fixed, no jump
      happens and the destination is removed. When a seize installs teleporting movement
      and this search finds no cell during the seize's destination assignment, that removal
      precedes the seized state: it does not end the grip. S-0607 claim 23 owns this
      case and its grip consequences.
    - (c) **The landing point of an infantry unit.** S-0208 claim 19 owns the standing-position
      choice, entry test, bridge distinctions and nearby search after refusal, which is available
      only for an object destination. Its position choices can take scenario-stream draws.
      S-0208 claim 20 owns placement-time clearing: when placement's fall-back to rest supplies
      a destination, a conditional forced, move-anywhere request to the destination cell's ground
      occupants precedes position choice. This is not owner-dependent and is not sent by an
      ordinary later move merely because the mover is infantry.
    - (d) **When it jumps.** It jumps on its first movement update after *L* is fixed. A vehicle's
      behaviour is processed before its locomotor in its update (S-0203), so a destination set by
      its behaviour in its update is jumped to in that same tick. When an order makes the
      destination appear belongs to S-0214 and S-0215. It does not turn first, does not wait, and has no leaving stage: it is never *being
      erased* because of its own move. When *L* equals its own position, it does not jump, and the
      destination is removed.
    - (e) **What the jump does.** All of this happens in the tick of the jump. Every object that
      has it as its target drops it, except an attacker whose strike target is the living
      jumping object (S-0519 Terms). The exception also applies after a strike ends normally:
      its strike target is kept (S-0519 claims 18 and 20), so no strike need be in progress.
      This is not an erasing-weapon exception: an object being erased drops destinations
      under part a, except that a pending suppression (claim 27) keeps the destination, and it
      cannot make this jump (S-0514 claim 16).
      When an override is in effect and that exception does
      not hold, the attacker reverts on the jump tick: it drops the jumping object if that object
      remains its target, but keeps a different target restored by the revert (S-0215 claim 10).
      These drops are not S-0309 claim 27's or claim 28's, so they take no targeting-timer draw,
      whatever a holder's timer shows (S-0309 claim 27: that reset belongs to the drops of those two
      claims).
      Every projectile aimed at the jumping object loses its target, so a shot already in flight
      towards it misses (the same effect as becoming untargetable). A warp animation plays at the
      old point, then the departure sound at the old point. A parasite inside it exits before
      the host leaves its old position, by S-0614 claims 12 and 13. The object is removed from its old
      cell's listing, takes position *L*, and is listed in *L*'s cell. On a high-bridge cell its
      cell layer is the bridge deck and its height is *H* + 416 leptons; elsewhere its cell layer
      is ground and its height is *H*. This includes an aircraft: it lands at terrain height,
      or on the deck, not at a flight or cruise height, and is listed in that landing layer.
      For a vehicle landing on a high bridge, the deck record is marked by its placement,
      but the earlier ground reservation is not transferred or cleared by the jump.
      Immediately after an otherwise uninterrupted landing, **both records** therefore
      have the vehicle-present mark. A later accepted destination clears the remembered
      ground reservation and reserves the new landing cell's ground record. The old deck
      mark remains while the vehicle is listed there, and is cleared when the next jump
      removes it from that cell. Off a high bridge, reservation and placement use the same
      ground record. S-0209 owns the entry outcomes obtained from either record; the
      ground reservation itself is not a listed ground occupant. This is
      `quirks.teleport_vehicle_ground_reservation` (below).
      **Aircraft bridge marks (Q-0310).** An aircraft likewise keeps its ground
      object-present reservation when deck placement sets the deck object-present flag.
      Immediately after an uninterrupted bridge landing both records have that flag,
      although only the deck lists the aircraft. Its next accepted destination clears
      the remembered ground reservation; the old deck flag clears when the next jump
      removes it. If instead it is destroyed and removed while still standing there,
      removal clears the deck flag but not the remembered ground reservation: the
      ground flag remains after the aircraft has left play. These flags are not
      reference counts; other objects marking or clearing the same flag can overwrite
      that result (S-0209 claim 1). This is
      `quirks.teleport_aircraft_ground_reservation` (below). The retained ground flag
      is not a ground-listed aircraft and does not create a vehicle-present flag.
      S-0209 owns the resulting entry outcomes and infantry standing-position refusal,
      which are distinct tests; a retained object-present flag alone must not be
      treated as a listed aircraft below the bridge.
      This actual landing height can differ from the infantry
      position-choice height (S-0208 claim 19). Its facing does not change. The arrival sound
      plays at *L*. Entering *L*'s cell has the arrival effects of the object's own kind: for a
      vehicle these are the effects of reaching a track target in a new cell, not a rule imposed
      on infantry or aircraft. The respective movement and combat specs own those effects.
      After the cell-arrival effects, the object attempts to collect any crate in the landing
      cell under S-0520 claims 7–11. The pickup happens after placement and does not undo the
      landing when its result would refuse an ordinary ground step. Speed bonuses and feedback
      belong to S-0520 claims 20 and 26 and claim 13 here. S-0520 claim 6 includes this
      own-locomotor pickup; its superweapon-transport exclusion is a separate rule.
      It stops moving and its
      destination is removed. A behaviour that ends once its destination is gone, such as
      *move*, then falls back to rest (S-0215 claim 13), but not within the jump tick: the
      behaviour is processed before the locomotion (part d), so the fall-back comes at a later
      tick, at the behaviour's next processing or at the end of phasing in with no target
      (part f), whichever is first. After a
      docking jump its *enter* behaviour goes on instead while its refinery accepts it, and
      falls back to rest if the refinery refuses it or the contact ends (claim 28). A second
      warp animation plays at *L*.
      Its type's departure and arrival sounds override the game-wide ones.
    - (f) **Phasing in.** At the jump the object starts *phasing in* (S-0214 Terms; *materialising*
      in S-0301) for *D* ticks, *D* from F9. An object whose type has the harvester flag of a
      vehicle type does not phase in at all. While phasing in:
      - it stays where it landed,
      - every new destination is dropped (part a), except one that a pending suppression (claim 27)
        keeps, so orders that would move it are lost, not
        delayed; S-0214 owns the refusal of player orders to a unit that is phasing in,
      - a weapon's firing check gives the early *rearming* answer (S-0301 claim 9),
      - its behaviour is still processed (S-0215 claim 2).

      Phasing in ends on the first tick *t* after the jump tick *t*₀ for which *t* − *t*₀ ≥ *D*:
      for *D* ≤ 0 that is *t*₀ + 1. Only when the object has no target, it attempts the teleport
      search of S-0309 claim 1. Before that attempt, if its targeting timer has more than 10
      integer ticks left, one ranged draw from the scenario random stream over 4 to 8 ticks,
      both included, replaces the timer. That conditional draw does not require the search gates
      to pass. A search allowed by S-0309 claims 2 and 3 then takes its own normal timer draw
      (S-0309 claim 4), replacing that temporary timer value. When the attempt sets no target,
      including when the gates refuse a search, the object falls back to its idle behaviour
      (S-0215 owns which). With a target, neither attempt nor conditional draw happens.
      A harvester-flag object, which does not phase in, has no such end.
    - (g) **Successive jumps.** There is no delay of its own between jumps apart from phasing in.
      After a jump the destination is gone, so a further jump needs a new destination from its
      behaviour or an order. One given after phasing in ends is taken at once (part a). A
      harvester-flag object can therefore jump on consecutive movement updates.
    - (h) **A moving target or a new destination.** *L* is fixed when the destination is given and
      is not followed afterwards. A new destination given before the jump replaces *L*, moving the
      occupancy mark again (part a). Because the jump follows on the next movement update, a target
      that moves rarely gets a chance to move in between. A destination given while the object is
      phasing in is dropped, unless a pending suppression (claim 27) keeps it. Whether a behaviour
      that chases a moving target gives a new
      destination after the jump is S-0215's.
      **An empty assignment while a jump is pending.** Assigning an empty order-level destination
      (what S-0402's seeking drop does) to an object whose landing point is fixed and whose jump
      has not happened cancels the pending jump and the moving state, so the object does not jump.
      The object then has a fixed landing point with no pending jump. That is the case claim 21
      describes as a retained landing point without a pending move. Part (d)'s jump on the first
      movement update therefore does not apply to it, and it does not jump until a later nonempty
      destination is accepted and fixes a new landing point (part a). The remembered landing point is
      not forgotten, and its occupancy reservation does not move: the vehicle-present (or
      object-present) mark stays at that point's cell, and the next accepted destination clears it
      from the remembered point (part a). Its backing is `unmeasured`: no scripted order is known to
      reach the window between fixing and the jump, and the one route into it that the spec describes
      is S-0402's seeking drop, whose timing S-0402 claim 17 itself leaves `unmeasured`. Confidence:
      medium, as for part (h)'s consequences. Run 17 (Measurement scenarios) is a conditional
      diagnostic that checks whether the window can be reached.
    - (i) **Random draws.** Landing choice and position change take no draw of their own.
      The two warp animations are ordinary animation creations and take every applicable
      creation draw of S-0202 claim 10. The old-point animation's creation draws precede
      the landing-point animation's creation draws in the jump tick. There is no special
      draw-free animation creation for a jump. Other arrival effects may take their own
      draws between the two creations. Nearby landing searches depend on the
      tick number, not on a draw. Infantry position choices and placement-clearing recipients can
      take draws as S-0208 claims 19–20 specify, with recipients' draws before position choice.
      At phasing-in end, part f's conditional 4-to-8-tick draw precedes the normal search-timer
      draw when both occur. Depending on target, remaining timer and search gates, both draws,
      only either draw, or neither can occur. Any candidate-dependent search draws belong to
      S-0309.
    - (j) **Other queries.** A request to turn the body turns it at the turn rate of S-0206, as for
      other locomotors. Crushing, damage and visibility rules are not changed by the jump or by
      phasing in, except for the target loss in part e.
    - (k) **Removal with a remembered or pending landing point (Q-0356).** For a
      teleporting vehicle or aircraft, removal from the map clears its actual cell's
      occupancy mark at its actual height, not a separate mark at its remembered
      landing point. Removal does not sweep all of its reservations. This holds both
      after a jump and between destination fixation and the jump. With the respective
      ground-reservation quirks on, and with distinct departure and replacement cells,
      the results below follow. No other object may mark or clear the tested flags in
      the meantime. A vehicle uses the vehicle-present flag, an aircraft the
      object-present flag.

      | Situation at removal | Actual cell after removal | Remembered or pending landing cell after removal |
      |---|---|---|
      | standing on a deck after landing, no replacement destination | deck flag clear, ground flag retained | same cell: retained ground reservation |
      | standing on that deck, replacement destination accepted but not jumped to | old deck flag clear, old ground flag already clear from replacement | new ground flag retained |
      | standing off a high bridge after landing, no replacement destination | ground flag clear | same cell: no flag remains, since actual occupancy and reservation coincide |
      | landing point fixed elsewhere, not yet jumped to | actual layer's flag clear; any separate stale ground flag under a deck is unaffected | pending ground flag retained |

      Thus the deck-removal case with a replacement destination leaves the new ground
      reservation behind, for an aircraft as well as for a vehicle. The retained flag
      is not a listed object and is not reference-counted (S-0209 claim 1). The
      consequences for entry and standing-position choice belong to S-0209 claims 5
      and 6. This is part of the respective ground-reservation quirks, not an
      additional removal switch.
    - Backing: `measurable` (scenario D22: runs 1 and 2 for parts a, b, d, e and f with the
      harvester exception, run 3 for the occupied destination, run 4 for
      cliffs and islands, run 5 for D against distance and each rules field, run 6 for an order
      dropped while phasing in and for part h, run 7 for the draw at the end of phasing in and for
      part i, run 8 for a shot in flight, run 9 for part j; S-0208 W11 for part c and infantry
      reservations, landing draws and placement clearing; S-0215 B18 d for ordinary target-loss
      and revert; runs 10 to 12 for the entry-test level, absent reach restriction, aircraft
      mark and landing, and animation creation draws; run 13 for Q-0240's vehicle
      reservation layers, replacement clearing and post-jump marks; run 14 for Q-0310's
      aircraft two-record landing, next destination and destruction cleanup; run 15
      for part k's actual-cell-only removal and retained pending reservations; run 16 for
      landing-cell crate pickup after placement). `unmeasured`:
      the airstrike target-retention exception, because no stock setup establishes an airstrike
      aimed at a living object making its own locomotor jump. Part a's vehicle destination
      restriction uses S-0517 claim 9's backing (P3 a and P4 b); jumpjet landing cancellation
      uses S-0502 claim 16's backing. Also `unmeasured` are part a's disabling states other
      than phasing in, part b's superweapon hold, part e's parasite expulsion, and part j's
      body-turn and exhaustive unchanged-visibility assertions: D22 isolates none of these,
      and no qualifying setup for their interaction with an own-locomotor jump is specified.
      The animation/sound ordering and type-specific sound overrides of part e are
      `measurable` through D22 run 1's presentation controls. Part f's continued behaviour
      processing is `unmeasured`: no scenario here isolates a behaviour-processing effect
      during phasing in from the locomotor's own completion effects. Its firing-check
      reply is owned and backed by S-0301 claim 9, not independently tested here. The retained-target
      phasing-in-end branch is `unmeasured`: run 7 supplies only a conditional protocol,
      and no reachable setup preserving a target through this interval is established.
      F9's distinction between different nonpositive delays is `unmeasured`: run 5 tests
      their shared next-tick completion, not the stored delay, and completion timing cannot
      distinguish those values. Part g's jumps on consecutive movement updates by a
      harvester-flag object are `unmeasured`: no run gives such an object a new destination on
      the movement update after a jump, and D22 run 2 makes a single docking jump. The
      seize-time no-cell case in part b inherits
      S-0607 claim 23's `unmeasured` backing: no setup establishing it is known.
      That part e's drops take no targeting-timer draw is `unmeasured`: no part here jumps a held
      object with its own locomotor while a holder's timer is read.
    - Confidence: the seize-time no-cell case in part b inherits S-0607 claim 23's
      low confidence. Part a's vehicle destination restriction inherits S-0517 claim 9's
      confidence (high for refusal, low for the carried exception); jumpjet landing cancellation
      inherits S-0502 claim 16's confidence. High for parts d, e and f and F9 (the whole move was read),
      except part e's exception after a docking jump, which takes claim 28's ratings: medium
      for *enter* going on (D34 part c), medium for the fall-back to rest on a refusal (D34
      part g) and medium for it on a lost contact until D34 part h runs (claim 28's backing and
      rating); medium for part b
      (including the entry-test level and the search without a reach restriction), for the
      aircraft reservation and landing rules, including Q-0310's clearing and retained
      ground flag after removal, for part k's vehicle and aircraft removal rules,
      and for Q-0240's vehicle ground reservation,
      retained-point clearing and two-record post-jump result; for part a's list of disabling
      states, for part d's same-tick consequence, for part e's fall-back to rest in a later tick
      than the jump (D22 run 1) and for part h's consequences; low for part g's consecutive jumps,
      which are `unmeasured` and follow from part a's immediate take-up and the absence of phasing
      in for a harvester-flag object; part c
      inherits S-0208 claims 19–20's confidence, high for
      position choice and draws and medium for search choices, bridge consequences and placement
      conditions; low for part j's statement that nothing else changes. Medium, not part e's
      high, for its drops taking no targeting-timer draw (`unmeasured`, as above)

21. **Teleporting locomotion after temporary driving (Q-0233).** The temporary drive of
    S-0417 claim 5 suspends the object's own teleporting movement; it does not make a new,
    empty teleporting state when S-0215 claim 13 restores that movement. At that restoration
    boundary a previously fixed landing point is retained, not rebased to the vehicle's current
    cell. Our engine cancels a jump that is pending when temporary driving is installed over
    teleporting (S-0417 claim 5, claim 28), as an empty assignment cancels one
    (claim 20 h): the landing point and its occupancy reservation stay, and no jump is pending at
    restoration. This has no quirk switch: a jump at restoration to a cell given before the drive
    would look like a fault in play (decided under the delegation, 2026-10-27). A retained
    landing point without a pending move does not by itself cause a jump. A destination changed
    or removed by the rest of the fall-back can replace the landing point as usual.

    A phasing-in state and its original expiry tick are also retained. Time spent temporarily
    driving does not restart or extend its delay: on the first teleporting movement update
    at or after expiry it ends as claim 20 f says. No occupancy mark is moved merely by
    restoration, either to the current cell or to an old landing cell. While driving is active,
    new destinations affect driving movement and its ordinary occupancy rules (S-0209), not
    the suspended teleporting landing point or its reservation. Thus an ordinary newly
    created vehicle with no pending teleport before its factory exit acquires no teleport
    reservation from its exit or rally destination while driving. After restoration, a
    destination fixes a landing point by claim 20 a only when teleporting movement receives
    it. For a vehicle with the teleport-capable flag, outside claim 28's exclusions, that is
    only a qualified docking assignment (claim 27), and any other assignment installs driving
    again (claim 28).

    **A previous deck landing (Q-0356).** Merely installing temporary driving clears
    neither the remembered ground reservation nor the actual deck mark. If a vehicle
    starts temporary driving while both records of its previous landing cell remain
    marked, ordinary driving departure clears its actual deck mark but not the separate
    remembered ground reservation. After it has left that cell, and at restoration,
    the former landing cell therefore keeps ground marked and has deck clear, provided
    no accepted teleport destination or other flag writer intervened. Driving marks
    its current cell by the ordinary actual-height rule: ground off a deck, deck on
    a deck. Restoration does not move that current-cell mark to the remembered landing
    cell, or recreate the old deck mark. A destination accepted before driving was
    installed can already have replaced the old ground reservation (claim 20 a).
    This conditional case does not undo that replacement.
    - Backing: `measurable` (D23 for the ordinary factory exit, destination routing and first
      post-restoration jump). `unmeasured` for the cancellation of a jump pending at
      temporary-drive installation: claim 28's sale branches reach it, but no scenario records
      the pending jump there, and the original's handling was not settled. `unmeasured` for a
      phasing-in state or a two-record deck landing at temporary-drive installation: no
      established stock setup starts a factory exit in those states. D23 gives a conditional
      observation protocol, not permission to manufacture one through state writes.
      These are conditional retention rules, not an assertion that a factory-produced
      vehicle starts in any of those states.
    - Confidence: high for a landing point's retention at restoration and no teleport
      destination handling during driving; medium for the cancellation of a pending jump at
      installation, a design rule; medium for the ordinary factory consequence and the conditional
      previous-deck-landing occupancy result

22. **Cell-tag arrival timing (Q-0268 part 1).** A driving vehicle's cell-arrival
    update (claim 12) raises the current cell's entered-by tag as S-0011 claim 18
    specifies. Reaching a track target is sufficient: the vehicle need not stop, and
    may begin its next track later in the same tick. Reserving a step's cells,
    crossing a cell boundary mid-track and turning in place do not themselves raise
    the arrival tag. An ordinary two-cell curve therefore does not raise the tag of
    its first cell merely by passing through it. A passive vehicle's chain-point
    arrival update uses the cell it actually occupies at that point, not the still
    unreached track target.

    For a vehicle, arrival reporting to a contacted building and cell-centre crushing
    precede the entered-by raise. Consequently trigger actions observe their completed
    effects, and no arrival raise is promised if those effects already removed the
    vehicle. High-bridge ground passage is excluded by S-0011 claim 18's deck-only
    condition; occupancy reservation below a deck is not arrival on the deck.

    An own-locomotor teleport of any object kind has its arrival update in the jump
    tick, after position, height and landing-layer listing have changed (claim 20 e),
    not when its landing point is reserved and not when phasing in ends. It raises
    the landing cell's entered-by tag under the same S-0011 eligibility rules.
    Cells between departure and landing receive no arrival update. For a teleporting
    vehicle, the tag's actions occur after its arrival reporting and crushing, but
    before jump-time crate collection, destination removal and the landing warp
    animation. A tethered infantry unit using its own teleporting locomotor also
    gives its factory clear report in this jump-tick arrival update (S-0417 claim 10).
    Factory contact and tether have therefore ended, and the contacted place is free,
    when the landing cell's entered-by actions execute. This covers an ordinary
    factory release to a clear rally cell without an entry interaction removing the
    unit. Reservation alone, phasing-in completion and falling back to rest do not
    supply that report. Without a jump, another arrival or an independent contact
    break, the unit retains contact and tether under S-0417 claim 10.
    Ordinary infantry walking timing belongs to S-0208, not this claim.
    - Backing: `measurable` (D24 for straight and curved driving, teleport, bridge
      and passive chain-point cases; S-0417 U10 d/e for teleporting infantry's
      jump-time clear report and ordering before entered-by actions). No-jump retention is
      `unmeasured`: no stock setup is known that keeps a released teleporting unit from
      jumping.
      Exact ordering against building reports and
      crushing uses D24 with S-0402 H14 and S-0210's crushing fixtures. Record their
      effects and the tag's variable action in the same arrival tick, using D24's ordering
      controls rather than end-of-tick coincidence alone. Ordering before jump-time crate
      collection, destination removal and the landing animation is `unmeasured`: D24
      establishes the arrival tick but isolates none of those comparisons
    - Confidence: medium; low for no-jump retention (`unmeasured`)

### Wakes while driving

23. **Driving wakes.** In each tick in which a drive vehicle is moving now and the tick number
    is divisible by 10, it creates one instance of the game-wide wake animation at its own
    position, provided it is off a bridge deck, its cell's land type is water, and that animation
    is defined. Moving now means its body is turning, or it has a heading point and its current
    integer progress rate *s* is above 0. Here *s* is the rate of claim 2, evaluated with the
    vehicle's current speed fraction and factors at the wake check: *s* = trunc(*s*₂ × *ρ*),
    in progress units per tick, with that claim's floating-point arithmetic, intermediate
    truncations and flag-carrying adjustment when applicable. It is not the fraction *ρ*
    itself, accumulated progress, distance actually travelled, or a remembered rate from the
    last moving tick. With no body turn, a positive *ρ* that still gives *s* = 0 produces no
    wake, even on an otherwise eligible tick. Positive *s* can produce a wake without a
    position change. The heading point is the active track's target (Terms), not a path
    destination retained between tracks. Thus waiting or being between tracks does not
    qualify through the heading-point test; turning in place can still qualify independently.

    The wake check uses the position, heading point, turning state and speed fraction after
    that tick's locomotor movement and track transitions, including any newly begun track.
    Each wake is at the post-movement position. Completing a track can remove heading-point
    eligibility even if the vehicle travelled earlier in that tick, unless another track has
    supplied a new heading point or its body is still turning. Creation occurs within the
    vehicle's own turn, as part of its locomotor movement, after movement's effects and their
    simulation-stream draws, before that locomotor update ends; it is not a separate
    end-of-tick wake pass. Its animation enters the update order under S-0203 claim 4.
    Wake creation takes no random draw beyond the animation's ordinary creation draws
    (S-0109 claim 3), which therefore follow any draws taken by that movement.
    There is no underwater-type exclusion for driving wakes, unlike the ship rule of
    S-0504 claim 2. That claim owns the ship rate, cadence and within-turn wake timing.
    - Backing: `measurable` (D25 for eligibility gates; D25a for the rate, heading-point
      boundaries, position and draw ordering; Q-0600)
    - Confidence: high

24. **Stored height of a driving or ship heading point (Q-0676).** On ordinary travel
    tracks, the heading point receives the vehicle's current world height when the track
    begins, before any progress along that track. This is an exact copy of an integer
    height in leptons, with no rounding, terrain sampling or additional deck lift. It is
    the same for one-cell tracks and two-cell curves. A high-bridge target cell does not
    by itself give the heading point deck height, whether the vehicle approaches on the
    deck or on the ground below it.

    The stored height stays unchanged while that track runs, even when claim 5 changes
    the vehicle's own height. Chaining a curve moves the heading point horizontally
    one cell on (claim 8) but keeps its previous stored height: it neither copies the
    vehicle's then-current height nor samples the new target's surface. Only beginning
    a new ordinary track after completion assigns the vehicle's height anew.

    **Factory exit exception (Q-0793).** The factory exit track of S-0417 claim 5
    has a heading point at the exit apron cell's horizontal centre with stored world
    height **0 leptons**, irrespective of the vehicle's starting height, target terrain
    height or a bridge deck. It receives no extra deck lift. That height stays 0 while
    the exit track runs. This is the stored heading height, not the vehicle's world
    height while travelling or its height at the end; claim 5's surface-height rule
    still governs those. Other building-supplied special tracks are not generalized
    from this factory case.

    Thus the stored heading height and the vehicle's current height can differ. The
    bridge-loss heading test and reservation clearing use the stored point as specified
    by S-0511 claims 9 and 11; that spec owns the separate current-height track test and
    the resulting hits, including Q-0676 question 3.
    - Backing: `measurable` (D27 for ordinary tracks; D28 for factory exits)
    - Confidence: high

25. **Separate driving and ship movement goal (Q-0794 questions 1 and 2).**
    A newly created driving or ship locomotor has no locomotor destination point.
    An accepted movement instruction installs its supplied point immediately, without
    requiring a successful path search, an enterable next cell, a nonzero speed or a
    different destination cell. Assigning a nonempty order-level destination normally
    supplies that destination's movement point to locomotion at that assignment, not
    only when travel starts. A direct approach instruction likewise supplies its point.
    This is not an unconditional assignment on every received move order.
    Claim 27's one-application suppression is an exception: a nonempty order-level
    assignment can be effective without installing this point, for driving as well
    as for jumpjet movement.
    Order permission and special movement holds still apply: this does not make an
    ignored or refused order an accepted instruction. S-0314 claim 18 owns a depot's
    supplied docking point and that spec owns the same-cell enter-order consequence.

    The locomotor destination is independent of the heading point. Beginning, finishing
    or chaining an intermediate track does not by itself remove the locomotor destination.
    In particular a wait, a turn, a retry delay or an unsuccessful search which retains
    the movement goal keeps it present. Merely crossing into its cell does not clear it.
    These lifetime rules apply equally to ship and driving locomotion:

    | Event | Locomotor destination point afterwards |
    |---|---|
    | Accepted instruction with a point | supplied point, replacing the old one; a high-bridge target adds 416 leptons to the supplied height |
    | Accepted instruction with no point | absent |
    | Stop or cancellation that stops locomotion, including removal of the order destination through the normal destination-setting rule | absent immediately, even while an already begun track finishes under claim 11 |
    | Completion of a track with an order destination still present, vehicle in that destination's supplied movement-point cell, and absolute difference between vehicle world height and locomotor destination height strictly below 208 leptons | absent before the cell-arrival report of claim 12 |
    | Other track completion, or standing in the goal cell without completing a track | unchanged by arrival alone; claim 26's separate order-removal rules can still apply |
    | Give-up with no approach destinations (claim 15) | absent through the normal removal of the order destination |
    | Give-up with approach destinations | fall back to rest under S-0215 claim 13; any resulting stop clears the point and any newly supplied movement goal replaces it, rather than treating give-up as successful track arrival |
    | Successful driving factory exit-track instruction of S-0417 claim 5 | exit apron centre with height 0 leptons, as for that track's heading point; ships retaining ship locomotion instead use that claim's ordinary destination instruction |

    Point coordinates and the height comparisons are integer leptons, with no rounding.
    The high-bridge lift is applied when the point is accepted, not added again each tick.
    A point in the vehicle's own cell therefore remains present unless an actual stop,
    replacement or qualifying track completion removes it. Claim 26 defines when standing
    there removes the order-level destination and thereby stops locomotion; it distinguishes
    a cell destination from one naming a building or a unit. Cell equality must not be used as a
    substitute for the point-presence test. An order-level destination can remain present
    after the track-completion row has cleared the locomotor destination, or can be removed
    by a later behaviour effect. S-0212 claims 20 to 22 own whether a search failure waits,
    retries or gives up; this claim adds no new give-up condition.
    **Factory-exit lifetime.** S-0417 claim 16 owns the subsequent factory-exit
    lifecycle. Completing the exit track with no order-level destination retains
    its locomotor destination point. The clear report, loss of contact and tether,
    and queueing *harvest* do not themselves stop locomotion or restore a suspended
    predecessor; there is no factory-exit timeout clearing that point. A later
    accepted rally, scatter or ore destination replaces it in the active driving
    locomotion. Qualifying completion at that later goal or an effective stop
    clears the resulting point under this claim and claim 26. The retained point
    prevents temporary-driving restoration under claim 27 until such clearing.
    Track completion at the apron alone is insufficient (S-0417 claim 5).
    - Backing: `measurable` (D29; D28 for the factory instruction; S-0417 claim 16
      for the factory lifecycle and its U15 scenario). The strict 208-lepton completion
      boundary is `unmeasured`: D29 proposes 207/208-lepton docking controls, but no legal
      routes establishing both completion heights are known here. Retention with all later
      replacements and effective stops excluded inherits that claim's `unmeasured`
      limitation: no finite trace establishes perpetual retention.
    - Confidence: high for assignment, independent presence, stopping and track-completion clearing,
      medium for the resulting lifetime across approach-destination fallback and the
      composed post-exit restoration consequences

26. **Removing the order-level goal at rest (Q-0797, Q-0830).** For driving and ship vehicles,
    a destination that names a cell differs from one that names an object, including a
    building, vehicle, infantry unit or aircraft. The cell containing an object's supplied
    movement point does not turn that object destination into a cell destination.

    | State at a locomotion update | Event | Result |
    |---|---|---|
    | No active track; order-level destination is a cell | Vehicle occupies that cell | With no approach destinations, the order-level destination is removed and locomotion is stopped, clearing its destination point immediately (claim 25). With approach destinations, the current order-level goal is abandoned and the vehicle falls back to rest under S-0215 claim 13, including any replacement goal that rule supplies |
    | No active track; order-level destination is a building | Vehicle occupies the cell of its supplied movement point | Cell equality alone removes neither destination. Entry exchanges and other behaviour effects remain S-0215 claim 29's and S-0314 claims 17 and 18's |
    | No active track; order-level destination is a vehicle, infantry unit or aircraft | Vehicle occupies the cell of its supplied movement point | Cell equality alone removes neither destination, just as for a building. Whether the named unit is moving or stationary makes no difference to this rule |
    | At rest in guard; locomotor destination point present | Vehicle's world point equals that destination point in all three coordinates | The same removal or approach-destination fallback as in the first row applies, including for a destination naming a building, vehicle, infantry unit or aircraft, whether the named object is moving or stationary |

    The first row is a locomotion rule, not a path-search result or a due *move* or *enter*
    processing. It requires neither exact horizontal alignment, a height tolerance, a
    successful search, nor an absent locomotor destination point. It does not depend on
    the current behaviour being *move*. The guard row instead requires exact equality of
    integer-lepton coordinates, including height, without rounding; being in the same
    cell is insufficient for that row. The resting and guard requirements describe the
    vehicle whose locomotion is being updated, not the unit named by its destination.
    The guard comparison uses the retained locomotor destination point, not the named
    unit's current world point. Movement of that unit alone is not a new movement
    instruction; an accepted replacement instruction can replace the point (claim 25).
    This is not a promise of indefinite retention: other stop, replacement, search-give-up,
    approach or boarding effects can still change either destination. S-0215 claims 22
    and 29 own move and enter processing; S-0507 claims 5 and 6 own transport exchanges
    and arrival-time boarding, which are separate from cell-equality cleanup.

    **Final track.** The qualifying completion of claim 25 clears the locomotor destination
    in the completion tick, not the order-level destination. Completion alone therefore
    leaves an ordinary cell destination present for the rest of that locomotion update.
    With the goal unchanged and no intervening effect, the first row removes it at the
    next tick's locomotion update, which begins without an active track. If completion
    is in tick *T*, due harvest seeking processing in tick *T* + 1 still sees the
    retained order-level destination before locomotion removes it later that tick.
    It therefore remains seeking rather than starting loading; S-0402 claim 3b owns
    the resulting loading-start and first-bale timing. Arrival-report effects can
    separately change the goal in the completion tick (claim 12). For an
    *enter* goal naming a depot, neither completion nor the next update's mere cell
    equality removes the building destination. Its entry and servicing effects own any
    subsequent removal; it is not forcibly discarded at the end of the last track.

    **Same-cell order from rest.** An accepted ordinary move that assigns the vehicle's
    own cell does not leave a moving destination indefinitely. Assignment supplies the
    locomotor point (claim 25); the first subsequent locomotion update without an active
    track removes the cell destination, in that update's tick. This can be the assignment
    tick itself when assignment precedes the vehicle's update. There is no extra one-tick
    timer and no wait for due behaviour processing. Conversely, an accepted same-cell
    *enter* instruction naming a depot can retain both destinations after successive
    locomotion updates: the building row applies, not the cell row. This is persistence
    absent another stop, replacement or applicable behaviour effect, not a guarantee that
    the depot's entry exchange will never change state. The *move* behaviour's subsequent
    fallback when it has no destination is S-0215 claim 22's separate event.
    - Backing: `measurable` (D30; D31 for destinations naming units, moving/stationary
      equivalence and the guard exception; S-0314 claim 17's H19 f for accepted same-cell entry,
      and S-0402 claim 3b's H27 for harvest processing before next-tick cleanup)
    - Confidence: high for the cell/object distinction, moving/stationary equivalence and cleanup conditions; medium for
      the composed tick timing and approach-destination fallback

27. **Destination refresh at placement and a teleporting dock approach (Q-1090, Q-1110).**
    This claim covers vehicles whatever their current locomotor. A destination-refresh
    state makes an otherwise eligible assignment effective even when it names the
    unchanged cell or object, including empty-to-empty. It starts absent.

    **Placement.** Successful placement enables refresh during initial idle-behaviour
    setup and clears it when that setup finishes. This interval is not inert: the normal
    placement fallback for a fresh ordinary vehicle with no destination clears its
    destination, including an empty-to-empty assignment. That assignment stops
    locomotion under claim 25 or the corresponding locomotor's clearing rule. It
    consumes refresh, and does not enable the separate point suppression. For a vehicle whose type
    has the teleport-capable flag it also installs driving (claim 28, the placement bullet). A
    fresh jumpjet without a movement commitment gains no flight from this clear. The type-specific
    fallback exceptions remain S-0215 claims 13 and 14's rules, not an unconditional extra clear
    imposed on every retained order at re-placement. The initial fallback and its destination clear
    precede the vehicle's later placement-only random choice for a visceroid's initial action
    timing, when that choice applies. There is no extra random request for refresh or for the
    clear. This relative order does not assert a complete draw count for all placement effects.

    **Dock evaluation event.** The dock test belongs to an otherwise eligible vehicle
    destination assignment, after unchanged-destination rejection, not to first contact,
    track arrival or a once-per-entry latch. It uses the destination offered by that
    assignment and the contact already held at that moment. An *enter* order naming
    the building does not itself satisfy the cell-destination requirement. In the
    ordinary refinery approach, S-0402 claim 10 row 2 offers the docking cell at the due
    *enter* processings that its row 2 names. Such an offer reaches this test when changed, or when
    refresh permits an unchanged assignment. An unchanged offer without refresh is
    ignored before this test, even if contact or cell occupancy has changed since the
    earlier offer. For a vehicle whose type has the teleport-capable flag, claim 28 adds a
    dock test at every accepted *enter* processing at which it has a destination, changed or
    not (claim 28, the repeated dock test). No other periodic dock test is made.

    All these conditions must hold: the vehicle's type has the teleport-capable flag,
    it has no warp-pending immobilization, is not gripped (glossary) and is
    not seized, its current contact is a building permitting dock unloading, the offered
    destination is a cell containing no ground-layer vehicle, and its current locomotor
    is not teleporting. A vehicle of any house on the ground layer disqualifies the cell,
    including the approaching vehicle itself if it is already there. The type flag is
    independent of the currently active locomotor. Failure of any predicate adds no
    stop, restoration or pending state through this dock rule. For a vehicle that passes
    the first four predicates, claim 28 says what the same assignment then does to its
    locomotor.

    **Which immobilization.** The predicate reads the vehicle's warp-pending
    immobilization, even when its current locomotor is not teleporting. It does not
    read phasing in after an own-locomotor jump or a superweapon arrival, the held
    countdown, or a cooldown before another jump. Own-locomotor jumps neither set
    this state nor introduce a separate cooldown (claim 20 f–g). An ordinary factory
    exit does not set it either. An ordinary accepted superweapon warp installs
    teleporting movement, so that warp alone does not furnish a non-teleporting
    dock-test fixture while immobilized. A simultaneous immobilization and
    non-teleporting locomotor, if reachable by another interaction, must still fail
    this dock test; ordinary-play reachability of that combination is not established.

    | Qualified assignment's locomotion state | Dock consequence |
    |---|---|
    | Temporary driving with a suspended predecessor, no movement report, no restoration lock, and vehicle not seized | Restore the predecessor without enabling either pending state. For driving, the movement report is false only with no locomotor destination point and no heading point horizontally different from the vehicle's position. Eligibility uses the driving points already present, not the destination offered by this assignment. The restored predecessor, not driving movement, receives this assignment's otherwise due movement point. The restored teleporting state follows claim 21 |
    | No suspended predecessor, including an ordinary driving locomotor | Stop the active locomotion, clear current behaviour and queue *enter* without an immediate start. Enable refresh and one-application point suppression |
    | Suspended predecessor exists but restoration is not permitted | Same fallback stop, behaviour and pending states as the preceding row, not a wait for restoration |

    This is not a factory-exit-only test and does not require a separate fall-back-to-rest
    event. The temporary-driving row gives the same restoration eligibility used by
    S-0215 claim 13, tested here at the destination assignment itself. Other temporary
    locomotors use their own restoration permission; magnetic lifting and seizure
    cannot qualify this dock rule because its earlier predicates exclude them. The
    factory exit of S-0417 claim 5 supplies an established temporary-driving case,
    not a restriction on the dock fallback to vehicles currently leaving a factory.
    The fallback does not turn ordinary driving into teleporting locomotion.

    **The restoring assignment's destination.** With a teleporting predecessor,
    successful restoration makes that same assignment fix its landing point under
    claim 20, rather than install a driving destination point first. Provided no
    already-pending point suppression (see the following paragraphs) or claim 20
    disabling state prevents it, the
    vehicle jumps on its next teleporting movement update, possibly in the same tick
    (claim 20 d). A pending earlier teleport can therefore be replaced by this offered
    destination. Restoration sets neither pending state, but does not exempt this
    application from suppression already pending before the assignment. A retained
    driving destination point prevents restoration even when the offered destination
    differs or would be immediately reachable. Assignment-time restoration here is
    separate from a later fall-back-to-rest restoration under S-0215 claim 13; factory
    exit point lifetime belongs to S-0417 claim 16, not an extra clear supplied here.
    That lifecycle retains the point after no-order exit completion, replaces it
    on a later accepted destination, and clears it on qualifying goal completion
    or effective stopping under claims 25 and 26. A factory-built chrono miner
    still using temporary driving therefore drives its first leg to distant ore.
    Queueing harvest or starting loading alone does not restore teleporting
    movement. S-0417 claims 5 and 16 own those temporary-driving consequences.
    A subsequent eligible fall-back to rest can restore the predecessor, while
    the qualified docking assignment described here remains a separate restoration
    route; neither requires an invented unconditional clear at the apron.

    **Suppression meets a teleporting destination (Q-1130).** The suppression test belongs to
    every nonempty vehicle destination assignment, whatever the active locomotor. An unchanged
    offer is rejected without consuming any suppression. A refusal that stops the assignment
    before the order-level destination is kept leaves suppression pending. Otherwise the nonempty
    order-level destination is kept, and while suppression is pending the active locomotor is
    not offered the point. It is not specific to restoration, to a teleporting predecessor or
    to driving. The table gives the results for a vehicle whose active locomotor is teleporting
    and still is once claim 28 has acted. For a vehicle with the teleport-capable flag, that
    means a qualified docking assignment, or one of claim 28's three exclusions.

    | Vehicle at the assignment | Suppression pending before it | Result |
    |---|---|---|
    | Not disabled, held, being erased or phasing in | no | Claim 20 (a) fixes the landing point |
    | Any state, including disabled, held, being erased or phasing in | yes | The locomotor is not offered the point: no landing point is fixed, no claim 20 (a) refusal test runs, and no earlier landing point, pending jump or reservation changes. Suppression is consumed. The new nonempty order-level destination stays |
    | Disabled, held, being erased or phasing in | no | The destination is dropped: the order-level destination becomes empty (claim 20 a). Nothing is consumed |

    S-0402's seeking drop adopts a harvester's remembered cell by an ordinary destination
    assignment. Claim 28 decides whether a chrono miner's teleporting locomotor receives that
    assignment: it does only when the adoption is a qualified docking assignment (at a sale,
    *A sale of the refinery*), and otherwise driving takes it. The rows reach only a harvester
    whose teleporting locomotor receives that assignment. For such a harvester these rows
    apply: pending suppression keeps the cell without a landing
    point, and the disabling states drop it (claims 20 a and 27); the drop that follows then fixes
    no landing point for the adopted cell in either case. Any earlier remembered landing point and
    its reservation are kept, as the table above and part (h) say.

    So a pending suppression is consumed by a teleporting vehicle even while it is phasing in or
    held, and the order-level destination then stays without a landing point; the vehicle
    does not jump toward it. A later equal offer is ignored by the unchanged-destination
    rejection unless refresh is pending, so nothing re-fixes the point until a further
    effective assignment arrives. A refusal that stops the assignment before the order-level
    destination is kept (a bunkered vehicle or one riding in an open-topped transport) leaves
    suppression pending; empty assignments never test it.
    Those refusals belong to the specs that own them.

    Reachability: the dock fallback sets suppression only for a vehicle whose active
    locomotor is not teleporting. It can set suppression in an assignment that an early refusal
    then stops before the order-level destination is kept. When an assignment goes on to keep
    its destination, the same assignment consumes the suppression. Suppression therefore stays
    pending across assignments only if an early refusal stops the assignment that set it and the
    locomotor becomes teleporting before the next assignment. A vehicle holding dock contact is not ordinarily
    in a bunker or an open-topped transport, and no sequence is known that makes it so and
    then switches it to teleporting. Even a purpose-made map with scripted input has no known
    way to build that state, so the table's second row and the restored-predecessor case are
    `unmeasured` for that reason, and D32 gives a conditional diagnostic.

    **Separate consumption events.** A changed or refreshed unchanged assignment
    consumes any refresh already pending even if subsequent point adjustment fails.
    A rejected assignment does not consume it; S-0502 claim 28 owns the balloon-hover
    clear rejection. Refresh newly enabled by the dock fallback remains available
    after the triggering assignment, unless another effective assignment consumes it.
    In contrast, suppression skips the next otherwise due nonempty point application,
    including the point application of the very assignment that triggered the fallback.
    Thus the usual qualified fallback installs the offered order-level cell destination,
    skips its locomotor point and leaves refresh, but not suppression, pending. The
    next otherwise eligible equal assignment can consequently reach the dock test again.
    There is no limit of one test or one fallback per entry attempt.

    Suppression applies equally to claim 25's driving point, to a jumpjet movement
    point and to a teleporting landing point (the table above). It consumes one application, not a whole order or every future assignment.
    An empty clear stops locomotion but does not consume nonempty-point suppression.
    Claim 20's landing-point rules apply only when a point actually reaches teleporting
    movement. Ordinary stop, move, deploy and behaviour changes do not independently
    enable refresh. Neither successful restoration alone nor destination retention
    implies that either pending state was set.
    - Backing: `measurable` (D32 and S-0502 J24 for placement and ordinary-order
      nonactivation; D33 for warp-pending immobilization versus phasing in and an
      ordinary factory exit; S-0417 claim 16 for the factory lifecycle and its U15
      scenario, including that claim's unmeasured no-follow-up limitation). The
      qualified dock evaluation, fallback, restored
      predecessor's same-assignment destination and separate consumption are
      `unmeasured`: no ordinary stock sequence isolating every predicate and both
      outcomes is established. In particular the immobilized, non-teleporting
      combination has no established ordinary-play fixture. The second row of the Q-1130 table
      (suppression pending at an assignment to a teleporting vehicle) is likewise `unmeasured`:
      the reachability paragraph above says why no fixture, ordinary or scripted, is known to
      produce it. D32 gives conditional
      diagnostics, not evidence that those paths are unreachable or that their states
      may be omitted.
    - Confidence: high for immobilization identity, the assignment event, predicates,
      temporary-driving eligibility, routing to the restored predecessor,
      placement clear and distinct consumption events; medium for ordinary stock
      reachability beyond placement and the composed post-exit restoration
      consequences inherited from S-0417 claim 16; medium for rows 1 and 3 of the Q-1130 table,
      which restate claim 20 (a) and its backing (D22); low for row 2, which is `unmeasured` (see above)

### Which locomotor a teleport-capable vehicle uses

28. **A teleport-capable vehicle drives, except on its docking approach (Q-1759).** This claim
    covers vehicles whose type has the teleport-capable flag (Import mapping), whatever their
    current locomotor. It does not cover infantry or aircraft: no rule installs driving for them
    at an assignment, so an infantry unit that moves with the teleporting locomotor takes every
    destination under claim 20, whatever its flag. Nor does it cover a vehicle
    type that has the teleporting locomotor without the flag.

    **The rule.** The rule acts at each vehicle destination assignment that gets past the
    unchanged-destination rejection (claim 27), and an empty assignment counts. It applies when
    the vehicle has no warp-pending immobilization, is not gripped (glossary) and is not
    seized: the same three exclusions as claim 27's dock test. Under any of them the rule
    installs nothing and replaces nothing: the destination goes to the current locomotor, as
    for a vehicle without the flag. The seize's own assignment of the lifter as the unit's
    destination (S-0607 claim 5) installs nothing under this rule, and the stand-in locomotor
    of S-0607 claim 23 takes it. Once seized, a vehicle ignores every new
    destination (S-0607 claim 9), and the one exception there is a rule that gives a gripped
    unit no destination, which ends its grip. Otherwise the assignment has one of two outcomes:

    | Assignment | Locomotor that receives it |
    |---|---|
    | A qualified docking assignment: every remaining condition of claim 27's dock test holds (contact with a building permitting dock unloading, a cell destination, no ground-layer vehicle in that cell) | Claim 27 decides. When the current locomotor is already teleporting, nothing changes and teleporting movement takes the destination under claim 20 |
    | Any other assignment, empty assignments included | Temporary driving, which then receives the destination, whenever the current locomotor is not driving. The suspended teleporting state is kept under claim 21. When the current locomotor is already driving, nothing changes |

    The qualifying conditions leave out claim 27's last condition, that the current locomotor is
    not teleporting, on purpose. A vehicle whose current locomotor is already teleporting (the
    at-rest window after a restoration, below) still makes a qualified docking assignment. Claim
    27's dock test fails for it on that condition and adds nothing, so the assignment goes to the
    teleporting locomotor, which jumps under claim 20; no driving is installed.

    Installing temporary driving here leaves the vehicle's current and queued behaviours as they
    are. In particular S-0215 claim 9's guard reset for an actor whose locomotor is replaced is not
    applied to it, nor to claim 27's and S-0215 claim 13's replacements (Cross-spec changes). D34
    part c observes this at the docking jump, where *enter* stays current. The temporary driving
    installed here is the same kind of temporary drive as S-0417 claim 5's factory exit: it can
    later be ended by the restorations of S-0215 claim 13 and claim 27. While it is active, the
    destination of this and of every later assignment belongs to driving movement and its ordinary
    path and occupancy rules (S-0209, S-0212), until one of those restorations ends it. An
    assignment that itself restores the teleporting locomotor (claim 27) gives its point to the
    restored locomotor instead. An assignment that driving receives fixes no landing point and
    moves no teleport reservation.

    If the current locomotor is some other temporary replacement whose restoration is
    permitted, that replacement is ended first, and driving is installed over the restored
    locomotor. If its restoration is not permitted, it is not ended: driving is installed over
    it, and it becomes driving's suspended predecessor in turn. Magnetic lifting and seizure
    never reach either case, because the exclusions above leave the rule out for them.

    **The repeated dock test in *enter*.** While its contact accepts it, or while it is tethered, a
    vehicle whose type has the teleport-capable flag and holds a destination in *enter*
    (S-0402 claim 10) has that destination treated as a new assignment at each processing of
    *enter*, even when it is unchanged. So a miner standing in contact jumps to the docking
    cell once that cell is clear (*An occupied docking cell*, below). A vehicle with no
    destination makes no such jump. A vehicle with no destination and further points of a
    waypoint path queued is outside this claim.

    **Consequences in ordinary play.** Several assignments in play are never qualified
    docking assignments:
    - **A player's order.** Every behaviour order that S-0213 table T3 part a does not ignore
      breaks off any contact with a building of the unloading kind before it assigns its destination (S-0213 table T3
      part b, before parts e and f). This covers a move, attack-move, guard-area or waypoint
      order, and a harvest order on a resource cell. None of them can therefore be a qualified
      docking assignment, in any harvest phase or behaviour: the vehicle drives to the order's
      destination.
    - **The vehicle's own jump.** When a teleporting jump removes the destination (claim 20 d,
      e), that removal is an empty assignment, so driving is the active movement kind from the
      jump tick on. The same holds for claim 20 b's removal without a jump, and for claim 20
      a's drop of a destination offered while disabled.
    - **After a docking jump.** The docking jump is made in *enter* (S-0402 claim 10), and
      removing its destination does not make that behaviour fall back to rest. The vehicle
      now stands on the docking cell, so the refinery offers it no destination (S-0402 claim
      10, rows 3 to 5), and the repeated dock test (above) has nothing to test, as the vehicle
      has no destination. Tethering, turning to the
      unloading facing and unloading make no destination assignment, and while the refinery
      accepts it the vehicle does not fall back to rest. So driving stays active through the
      docking and the unloading, and unloading ends by breaking the contact before the next
      destination is set (S-0402 claim 11). If instead the refinery refuses it, or the contact
      has ended, at such a processing before it docks, the vehicle falls back to rest. S-0215 claim 13's
      restoration then finds the driving just installed eligible, because the vehicle stands
      still with no destination, and restores teleporting. The vehicle then rests with
      teleporting active (the second window below) until its next destination assignment,
      which has no contact and so installs driving again.
    - **After unloading.** Outside a sale of the refinery (below), a harvester's refinery
      contact ends before the harvest cycle sets its next destination (S-0402 claim 11). Its
      trips to resource cells and to a queueing cell are therefore driven.
    - **A sale of the refinery.** When its refinery is sold, a chrono miner that is unloading
      there, or is still in *enter* with it, receives a scatter request while the contact still
      holds (S-0402 claim 11). The request's destination (claim 16) is a cell, so it is a
      qualified docking assignment unless a ground-layer vehicle is in that cell, and claim 27
      decides. A chrono miner on the docking cell, after its docking jump, has driving
      installed over its teleporting locomotor and no movement report, so teleporting is
      restored, with a landing point fixed at the scatter cell and the jump pending (claim 20
      a). A chrono miner in *enter* makes that jump at its next movement update (claim 20 d).
      That jump removes the destination, which installs driving again (the jump bullet above).
      An unloading chrono miner has *harvest* current from the sale (S-0402 claim 11), and the
      first processing of that *harvest* comes in its next turn, before its locomotion and while
      the contact still holds (S-0402 claim 17, case (f)). That processing decides:
      - if the miner is full, it assigns nothing, and the miner jumps to the scatter cell,
      - if it is not full, a remembered cell that it adopts is a qualified docking assignment, which
        teleporting keeps, unless a ground-layer vehicle is in it: then driving takes it, which
        cancels the scatter jump (claim 21), nothing is dropped, no ring search follows, and the
        miner drives to its remembered cell. Otherwise the seeking drop's empty assignment installs
        driving, which cancels the scatter jump (claim 21), so no jump to the scatter cell or the
        remembered cell is made; and the cell that the ring search sets is a qualified docking
        assignment while the contact holds, unless a ground-layer vehicle is in it, so teleporting
        is restored and the miner jumps to that resource cell in the same turn, without reaching the
        scatter cell. With a vehicle in that cell, driving takes it and the miner drives there. When
        the search finds no cell, it stays where it is with driving installed. Our engine keeps this
        jump to a resource cell, with no quirk switch: it follows from the qualification rule, which
        tests the contact and a free cell destination, not which cell it is, and only this rare case
        reaches it (decided at the delegated sign-off, 2026-10-25).

      The sale breaks off the contact at its next processing, 1 tick after the first (S-0402
      claim 11), so no later assignment is qualified through the sold refinery: a queueing cell on
      the way to another refinery, for example, is driven.

      A chrono miner still approaching gets the request only if it then has no destination: a
      destination naming the refinery building is removed first, and a cell destination is kept with
      no request (S-0402 claim 11). One that gets the request while still moving gets claim 27's
      fallback instead of the jump. A refused request (S-0402 claim 11: a turning body, an existing
      destination, or *unload* still current) assigns nothing. A chrono miner still in *enter* has
      *guard* queued by the sale (S-0402 claim 11), whatever came of the sale's scatter request. It
      falls back to rest at a processing of *enter* if one comes before that *guard* starts: at the
      first one if it is not tethered, since a refinery being sold refuses it (S-0402 claim 10 row
      1), or otherwise at the first one after the sale has broken off its tether and contact (S-0408
      claim 6). That fall-back restores teleporting when it stands still with no destination (the
      at-rest window below). The queued *guard* starts at the first per-tick start at which the
      miner is ready to change (S-0215 claim 11), for example after its jump, or once a refused
      turning body's turn has ended. A processing of *enter* comes only when its processing timer is
      due (S-0215 claim 2), so which of the two comes first depends on that timer. If the *guard*
      starts first, the miner makes no fall-back from *enter*: its behaviour becomes *guard*,
      driving stays installed, and teleporting is not restored this way. One that was unloading has
      *harvest* current and does not fall back.
    - **Placement.** The destination clear of claim 27's placement is an assignment that gets
      past the unchanged-destination rejection, so the rule acts on it. A new vehicle placed
      with its own teleporting locomotor active, such as a refinery's free miner (S-0402 claim
      18), therefore has driving installed at its placement, before its first trip. A
      refinery's free unit has none of the states in which placement skips that clear: when it
      is placed it has no contact, no destination, and no current or queued behaviour, because
      the free-unit rule gives it *harvest* only after the placement (S-0402 claim 18). Its
      placement fall-back is therefore S-0215 claim 15's row 4 for a harvester with nothing
      current or queued: target and destination are cleared, which installs driving, and
      *harvest* is queued. The free-unit rule then gives it *harvest* with an immediate start,
      so the miner starts its harvest cycle at once with driving already active. A
      vehicle-factory-built vehicle leaves its factory with one temporary driving over its
      suspended teleporting locomotor. If its placement has already installed driving, the
      factory exit (S-0417 claim 5) installs nothing more and moves it with that driving.
      Otherwise the factory exit installs it. Either way the later restorations (S-0215 claim
      13, claim 27) bring back teleporting, not driving.

    Apart from a superweapon warp (claim 27) and the three exclusions above, teleporting
    locomotion is therefore active only in these windows:
    - from a qualified docking assignment to the end of that jump, or to an earlier assignment
      that installs driving (an unloading miner at a sale, *A sale of the refinery*, above); a
      further qualified docking assignment before then carries the window on to its own jump,
    - at rest, after a restoration by S-0215 claim 13 or by the end of a seizure (S-0607 claim
      20), until the next destination assignment,
    - from the placement of a vehicle whose own locomotor is teleporting and whose placement
      makes no destination assignment (one of claim 27's type-specific fallback exceptions
      applies), until its first destination assignment or a factory exit's temporary driving.
      A refinery's free miner is not such a vehicle (the placement bullet above). Which ways of
      creating a vehicle reach this window is `unmeasured` and rated low: none is established,
      and
    - after S-0607 claim 4 ends a resource-gathering vehicle's temporary driving (a
      magnetic-beam hit that seizes nothing), until its next destination assignment: that
      ending makes no destination assignment. The restored teleporting locomotion has no
      pending jump, since the destinations given while it drove went to driving and a jump
      pending when its driving was installed was cancelled then (claim 21), so it makes no jump
      in this window: its next jump still needs a qualified docking
      assignment. A second hit inside this window
      finds no temporary locomotor and seizes the vehicle (S-0607 claim 5). This window is
      `unmeasured` and rated low: no scenario records the active movement kind after such a hit.

    A chrono miner's only jumps in ordinary play (outside contrived set-ups such as S-0402 H14's
    named test map) are to a refinery's docking cell, at a
    processing of its *enter* approach while it is in contact with that refinery: through the
    refinery's docking-cell offer (S-0402 claim 10 row 2), or through the repeated dock test of
    that cell (above); and, when that refinery is sold while the contact still holds, to the
    scatter cell next to it or, for a miner that was unloading and is not full, to the resource
    cell that its first seeking processing sets (*A sale of the refinery*, above). Whether it
    has docked before since
    leaving its factory makes no difference to any other trip.

    **An occupied docking cell.** Suppose a vehicle stands on the docking cell when the offer
    is made to a vehicle whose suspended predecessor is its teleporting locomotor. (A vehicle
    with no suspended predecessor whose current locomotor is not teleporting gets claim 27's fallback at every qualified docking
    assignment instead, and does not jump.) The offer is then not a qualified docking assignment: the teleport-capable
    vehicle drives towards that cell, except that a path computation that adopts a substitute for
    it stops the vehicle at the end of its track (S-0402 claim 10a, *A chrono miner*). The
    repeated dock test checks the cell again at each
    accepted processing and fails while the cell stays occupied, so it goes on driving. At the
    first such processing after the cell has cleared, the test is a qualified docking
    assignment, and claim 27 decides:
    - when claim 27 permits the restoration (the vehicle has no movement report, claim 25),
      teleporting is restored and the vehicle jumps to the docking cell,
    - when the vehicle is still moving, it gets claim 27's fallback: it stops, *enter* is
      queued again and refresh is pending. At the first later processing of *enter* at which it
      stands still (it first finishes the track it is on, claim 11),
      the refinery offers the docking cell again (S-0402 claim 10 row 2), refresh lets that
      unchanged offer through, teleporting is restored and it jumps.

    It reaches the docking cell by driving only if it gets there before those processings.

    **One jump.** When the refinery's docking-cell offer (S-0402 claim 10 row 2) and the
    repeated dock test both act in one processing, they are two qualified docking assignments
    of the same cell, the offer first. The first meets claim 27's dock test:
    - when claim 27 permits the restoration, the first restores teleporting and fixes the
      landing point at the docking cell's centre. The second then finds teleporting current, so
      claim 27's dock test fails on its last condition and adds nothing, and teleporting
      movement takes the same cell under claim 20 (the first row of this claim's table). That
      fixes the landing point again: the mark the first left at the docking cell is cleared
      before the entry test (claim 20 a), so the test finds the cell as the first did, and the
      point is the same centre. The vehicle makes one jump, to that centre.
    - when the vehicle has a movement report, the first gets claim 27's fallback: locomotion
      stops, which removes the locomotor destination point (claim 25), *enter* is queued again,
      refresh is pending, and the first assignment's own point is suppressed, which consumes the
      suppression (claim 27, *Separate consumption events*). The second then meets claim 27's
      predicate in the state that fallback leaves:
      - if a heading point horizontally different from the vehicle's position is still ahead
        (a track begun before the stop is still being finished, claim 11), the vehicle still
        has a movement report, and the second gets the same fallback again. The result is that
        of a single fallback: no landing point is fixed, and the vehicle jumps at the first later
        processing of *enter* at which it stands still (*An occupied docking cell*, above),
      - if none is, the vehicle has no movement report after the first fallback, so the second
        restores teleporting and, with no suppression pending, fixes the landing point by claim
        20 (b). The first fixed none, so the docking cell holds no reservation of the vehicle's
        own from that processing, and the point is the docking cell's centre when the entry test
        finds the cell clear. The vehicle jumps at its next teleporting movement update,
        possibly in the same tick (claim 20 d), with *enter* queued by the first fallback.

    The one jump is `measurable` (D34 part c's docking jumps, each to the docking cell, which a
    landing point from the nearby-cell search would not give) and rated as claim 28 rates the
    docking jump. Both branches of the case with a movement report are `unmeasured` and rated
    low: no set-up is known that makes the offer to a vehicle that has a movement report, and
    a recorder sees only the stop it shares with a single fallback, or a jump it shares with a
    restoration at the first assignment.

    | Q-1759 question | Answer |
    |---|---|
    | 1: a player's move order to an open cell after docking | It drives there along a path |
    | 2: the same order while seeking or loading, returning full, or in guard | It drives, in every case |
    | 3: attack-move, guard-area, waypoint path, or a harvest order on a resource cell | It drives, for every kind |
    | 4: what makes it teleport again | Only a qualified docking assignment: the refinery's docking-cell offer, or the repeated dock test of that cell, at a processing of *enter* while it is in contact with that refinery and no vehicle stands on that cell. It jumps when claim 27 permits the restoration. A miner that is still moving gets claim 27's fallback first: it stops, and jumps at the first later processing of *enter* at which it stands still, or in that same processing when a second assignment of the cell there finds it with no movement report (*One jump*). The other qualified docking assignments in ordinary play come at a sale of that refinery while the contact still holds: its scatter request, which makes a miner on the docking cell jump to the scatter cell next to it, and, for a miner that was unloading and is not full, the cells its first seeking processing sets; *A sale of the refinery* gives where it then goes, a jump to a resource cell among them |

    - Backing: `measurable` (scenario D34) for the two outcomes and their consequences in ordinary
      play, including the vehicle staying with driving through docking and unloading (part c) and
      the occupied docking cell (part e), the behaviour kept at an install (part c, at the docking
      jump) and driving from placement (part f). `measurable` through D34 part h for the sale's
      jumps (to the scatter cell in runs 2 and 4, to a resource cell in run 1; run 1's absence of a
      move to the scatter cell rests on claim 21's `unmeasured` design rule, the cancellation of the
      pending scatter jump, and is rated medium on that basis), for the restoration
      after the sale's end of contact, and for the queued *guard* starting first, and through D34
      part i for the at-rest window after a move order. `unmeasured` for an unloading miner's sale
      with a ground-layer vehicle in its remembered cell or in the ring search's cell, or with no
      cell found: D34's map has a
      free field in range, and no other set-up is known. `unmeasured` for a sale's scatter
      request to a miner still approaching and still moving (claim 27's fallback): no set-up is
      known that times the sale to that state. A refusal by the refinery while the contact is still
      present is `measurable` through D34 part g: a refinery switched off (S-0405 claim 10) after
      the docking jump and before the next processing of *enter* refuses the untethered vehicle on
      its docking cell (S-0402 claim 10 row 1), which breaks off the contact itself, with the same
      fall-back and restoration as a lost contact. `unmeasured` for the third window (a placement
      that makes no destination assignment) and for the window after S-0607 claim 4's restoration,
      for the reasons given in the window list. `unmeasured` for the three exclusions and the two
      nested-replacement cases: each needs a teleport-capable vehicle that is warp-pending, gripped,
      seized or under another temporary replacement at the moment of an assignment, and no
      ordinary-play fixture is known that reaches that and can then tell the outcomes apart. D34
      records at the end of each tick, so it shows the movement kind from the tick of an assignment
      on, not that the installation is made at the assignment rather than later in that tick: that
      order within the tick is `unmeasured`, because no recorder field shows the movement kind
      between the assignment and the end of the tick.
      The repeated dock test itself is `unmeasured`: in part e a substitution leaves refresh
      pending, which lets the refinery's unchanged offer through when the cell clears, and a route
      found without a substitute drives *H* on to the cell, so no falsifier of part e fires without
      it.
    - Confidence: medium for the rule, including empty assignments, and driving from the jump
      tick on, shown at the end of the jump tick: no run has yet shown the active movement kind at
      an assignment, because D34 has not run. Medium for driving staying active through docking and
      unloading, and for the repeated dock test in *enter* with the occupied-docking-cell sequence
      that follows from it, for the behaviour kept at an install, and for driving from placement.
      Medium for the sale's jumps, for the queued *guard* that can start before a fall-back, for the
      restoration after a lost docking contact and for the at-rest window after a move order, until
      D34 parts h and i run; low for the sale's request to a miner still moving, and for an
      unloading miner's sale with a vehicle in its remembered cell or in the ring search's cell, or
      with no cell found. Medium
      for the composed consequences: contact broken by every order kind and after unloading. Medium
      for the refusal case until D34 part g runs. Low for the exclusions, the nested replacement
      cases, the third window and the window after S-0607 claim 4's restoration (Open questions).

## Formulas

Every floating-point operation in this spec, in the claims (the speed-fraction updates of claim 10
included) as well as in the formulas below, runs in the original's program-wide *round-toward-zero
mode* (glossary): results are rounded to double precision, toward zero. S-0205 (number domains) owns that
mode and its switch `quirks.chop_rounding`. Every conversion to an integer truncates toward zero
whatever the mode. Throughout, clamp(*x*, *p*, *q*) means min(max(*x*, *p*), *q*).

**F1 Type speed** (claim 1). *V* = min(⌊256 × *v*′ / 100⌋, 255), with *v*′ = min(max(*v*, 0), 100),
for every *v* except −1 (*v* = −1 leaves *V* unchanged). Integer arithmetic, and the division
truncates (*v*′ ≥ 0). Unit: progress units per tick.

**F2 Progress per tick** (claim 2).
> *s*₁ = trunc((*V* × *κ*) × *b*), where *κ* is single precision widened to double and *b* is double.
> *s*₂ = trunc(*s*₁ × *ν*) when the faster ability applies, otherwise *s*₁.
> *s* = trunc(*s*₂ × *ρ*).
> Flag carrier: trunc(*s* / 2) in place of *s*, an integer division truncating toward zero.
Unit: progress units per tick.

**F2a Speed estimate for lead** (claim 18). Use F2 with the target's currently held *ρ*,
including its successive truncations and optional integer halving. The integer result is
interpreted as leptons per tick in S-0306 F6, without scaling. The products have F2's
floating-point domain and round-toward-zero mode. This query makes no fraction update.

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
square root of S-0204, Formulas F1, which owns how *r*(*n*) follows from *n* (with *n* taken in
single precision) and how that depends on the rounding mode of S-0205. The result is truncated to
an integer. Unit: leptons. The comparison *d*ₘ < *D*ₛ is
between integers. S-0204's switch `quirks.table_root_distance` covers this distance too: off, *d*ₘ =
⌊√*n*⌋, the exact integer square root.

**F6 Target fraction** (claim 9). *ρ*ₜ = *h* × *z*, where *z* = *g* × *u* if *g* × *u* ≠ 0 and
*z* = 0.5 if *g* × *u* = 0, with *g* = min(*M*, 1), *u* the slope factor (1 when the heights are
equal) and *h* = 0.75 for a damaged vehicle and 1 otherwise. Double precision throughout. Unit:
fraction (dimensionless).

**F7 Give-up test** (claim 15). The test holds when all three of these hold:
> *d*ₘ < *C*, with *d*ₘ the distance of F5 from the vehicle's position to its move destination and
> *C* the close-enough distance of S-0209 (Formulas F2), both integers in leptons,
> |*z*ᵈ − *z*ᵛ| < 208, with *z*ᵈ the world height of the move destination and *z*ᵛ the vehicle's
> world height, an exact integer comparison in leptons (208 is two height levels of 104 leptons),
> and the land type of the vehicle's own cell is not *tunnel* (S-0211).
The look at the first cell of a new path (claim 15) adds a fourth condition: the vehicle is in
contact with nothing. Unit: leptons.

**F8 Distance of the nearest-object choice** (claim 15, the first cell of a new path). For an
object whose position is (*x*, *y*, *z*) in leptons, *u* = *x* mod 256 and *w* = *y* mod 256, each
the residue from 0 to 255 (the position within the cell that holds it, measured from that cell's
top corner). *n* = *u*² + *w*², an exact integer (the original forms the squares and the sum in
double precision, which is exact here, as *n* ≤ 130050), and *d*ₙ = ⌊*r*(*n*)⌋ with *r* the table
square root of S-0204 (Formulas F1), truncated to an integer as in F5. *z* plays no part. The
*d*ₙ of two objects are compared as integers. Among the objects with the smallest *d*ₙ, the one
listed first (S-0209 claim 13) is the nearest, as claim 15 states. S-0204's `quirks.table_root_distance` covers
this distance too: off, *d*ₙ = ⌊√*n*⌋. Under `quirks.drive_nearest_from_top_corner` off, *u* and
*w* are replaced by (*x* mod 256) − 128 and (*y* mod 256) − 128, so the distance is measured from
the cell's centre. Unit: leptons.

**F9 Phasing-in delay** (claim 20 f). Let *n* = Δ*x*² + Δ*y*² + Δ*z*² be the squared distance
in square leptons from the object's position before the jump to *L*, and *d* = ⌊*r*(*n*)⌋ in
integer leptons. The distance calculation has F5's floating-point domain and rounding, with
the table square root of S-0204 claim 5; `quirks.table_root_distance` covers it too. The delay
calculation below uses integer arithmetic. Let *k* be the distance factor in leptons per tick,
*m* the minimum delay in ticks and *g* the range minimum in leptons. The auxiliary delay *q*
in ticks is defined by these mutually exclusive cases:

| Condition | *q* (ticks) |
|---|---|
| Trigger flag on, *k* ≠ 0 | trunc(*d* / *k*), division toward zero |
| Trigger flag off | 0 |

The zero-factor case with the trigger on is the bad-data departure in Edge cases and quirks:
the original stops with a program error; our engine uses *q* = 0 and warns.

| Mutually exclusive condition | *D* (ticks) |
|---|---|
| Object has the harvester flag of a vehicle type | 0; no phasing in |
| No such flag, *d* < *g* | *m* |
| No such flag, *d* ≥ *g*, max(*q*, 0) > *m* | *q* |
| No such flag, *d* ≥ *g*, max(*q*, 0) ≤ *m* | *m* |

For a non-harvester with *d* ≥ *g*, *D* is not always max(*q*, *m*): when *m* is negative,
*D* = *q* even for a smaller negative *q*. Every non-harvester delay *D* ≤ 0 ends phasing in
on the tick after the jump (claim 20 f), so those different negative delays have the same
observable completion tick. The harvester exception instead has no phasing-in state.

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
| Sinking deceleration | 0.00150000001303851604461669921875 (single-precision 0.0015) | fraction per update (claim 10), per progress unit per tick of *V* | Multiplied by *V*; claim 10 |
| Crushing cap | 0.2 | fraction | Claim 10 |
| Zero-multiplier fallback | 0.5 | fraction | Claim 9 (d) |
| Damaged factor | 0.75 | factor | Claim 9 (e) |
| Deck height | 4 | height levels | Claim 9 (a); the glossary's bridge deck |
| Height added on a bridge deck | 416 (4 height levels of 104) | leptons | Claim 5: world height *H* + 416 at each point while on a deck |
| Level drop that puts a vehicle onto a deck | exactly 4 | height levels | Claim 5: the high-bridge cell entered lies exactly 4 levels below the cell left |
| Level tolerance | 2 | height levels | Claim 9 (a): a difference below 2 uses the cell's land type |
| Flag-carrier divisor | 2 | divisor (dimensionless) | Claim 2 |
| Straight step cost | 163 | progress units | F3 |
| Diagonal step cost | 227 | progress units | F3 |
| Absent-key default: speed field | type speed 0 | progress units per tick | Claim 1 |
| Absent-key default: acceleration flag | on | boolean | Claim 10 |
| Absent-key default: slowdown distance *D*ₛ | 500 | leptons | Claim 10 |
| Absent-key default: deceleration factor *η* | 0.00200000000000000004163336342344337026588618755340576171875 (the double nearest 0.002) | fraction per update (claim 10), per progress unit per tick of *V* | Claims 10 and 17 |
| Absent-key default: acceleration factor *a* | 0.0299999999999999988897769753748434595763683319091796875 (the double nearest 0.03) | fraction per update (claim 10; two updates in some ticks, claim 12) | Claims 10 and 17 |
| Absent-key default: uphill and downhill factors | 1.0 each | factor | Claim 9 (c) |
| Absent-key default: veteran speed factor *ν* | 1.0 | factor | Claim 2 |
| Absent-key default: country speed factor *κ* | 1.0 | factor | Claim 2 |
| Starting speed bonus factor *b* | 1.0 | factor | Claim 13 |
| Starting speed fraction *ρ* | 0 | fraction | Claim 11 |
| Absent-key default: locomotor | the teleporting locomotor | locomotor | Claim 14; every infantry, vehicle and aircraft type |
| Height tolerance of the give-up test | less than 208 (2 height levels of 104) | leptons | F7 |
| Height level of a vehicle for the cell layer of a request | trunc(*z* / 104), rounded toward zero | height levels | Claim 15 |
| Level difference that sends a request to the deck layer | 3 or more | height levels | Claim 15 |
| Height above the surface height *H* at a cell's centre from which the nearest object is looked for in the deck layer | more than 208 (2 height levels of 104) | leptons | Claim 15, first cell of a new path |
| Point the nearest object is measured from | the cell's top corner: position (0, 0) within the cell | leptons | Claim 15 and F8; the residues *x* mod 256 and *y* mod 256 are taken from 0 to 255 |
| Range of the target draw of a threat-point scatter | 1 to 4 (accepts on 1) | draw value | Claim 16 |
| Range of the direction draw of a threat-point scatter | 0 to 2, minus 1 | step directions | Claim 16 |
| Height above *H* at the destination from which a teleport landing search is in deck mode | more than 312 (3 height levels of 104) | leptons | Claim 20 b |
| Height added on landing on a high-bridge cell | 416 (4 height levels of 104) | leptons | Claim 20 e |
| Absent-key default: teleport distance factor *k* | 32 | leptons per tick | F9 |
| Absent-key default: teleport trigger flag | on | boolean | F9 |
| Absent-key default: teleport minimum delay *m* | 0 | ticks | F9 |
| Absent-key default: teleport range minimum *g* | 0 | leptons | F9 |
| Targeting-timer threshold at the end of phasing in | more than 10 | ticks | Claim 20 f |
| Range of the targeting-timer draw at the end of phasing in | 4 to 8, both included | ticks | Claim 20 f |

**T1 Track point tables.** Each table row gives a point in driving order. The index starts at 0. The columns *x* and *y* are the point's displacement from the track target in leptons along the map axes (*y* grows towards map south); *f* is its facing in 1/256 turn. The target itself, (0, 0), is not listed. Point numbers used for chain, entry and release points count from 0. A release point lies inside the curve's first cell (the cell its current step enters) and is the point at which that cell's occupancy record (glossary) stops showing a vehicle present, although the vehicle is still in the cell (S-0209 claim 1). The values include irregular points, including (−129, 129) in curve B and facings 66 then 64 in curve D. Curve F's point 0, (−512, 256, 32), is the track's own start (the centre of the cell the vehicle starts from); every other curve's first point lies one increment along the track. Curve F's point 0 still costs 7 progress units to pass and moves the vehicle 0 leptons; F3's cost of 399 includes it. All listed values are engine constants: fixed inside the original's program, not read from any data file.

**Curve A** (straight, towards map north; 23 points; no chain point).

| Point | *x* (leptons) | *y* (leptons) | *f* (1/256 turn) |
|---:|---:|---:|---:|
| 0 | 0 | 245 | 0 |
| 1 | 0 | 234 | 0 |
| 2 | 0 | 223 | 0 |
| 3 | 0 | 212 | 0 |
| 4 | 0 | 201 | 0 |
| 5 | 0 | 190 | 0 |
| 6 | 0 | 179 | 0 |
| 7 | 0 | 168 | 0 |
| 8 | 0 | 157 | 0 |
| 9 | 0 | 146 | 0 |
| 10 | 0 | 135 | 0 |
| 11 | 0 | 124 | 0 |
| 12 | 0 | 113 | 0 |
| 13 | 0 | 102 | 0 |
| 14 | 0 | 91 | 0 |
| 15 | 0 | 80 | 0 |
| 16 | 0 | 69 | 0 |
| 17 | 0 | 58 | 0 |
| 18 | 0 | 47 | 0 |
| 19 | 0 | 36 | 0 |
| 20 | 0 | 25 | 0 |
| 21 | 0 | 14 | 0 |
| 22 | 0 | 3 | 0 |

**Curve B** (diagonal, towards map north-east; 31 points; no chain point).

| Point | *x* (leptons) | *y* (leptons) | *f* (1/256 turn) |
|---:|---:|---:|---:|
| 0 | −248 | 248 | 32 |
| 1 | −240 | 240 | 32 |
| 2 | −232 | 232 | 32 |
| 3 | −224 | 224 | 32 |
| 4 | −216 | 216 | 32 |
| 5 | −208 | 208 | 32 |
| 6 | −200 | 200 | 32 |
| 7 | −192 | 192 | 32 |
| 8 | −184 | 184 | 32 |
| 9 | −176 | 176 | 32 |
| 10 | −168 | 168 | 32 |
| 11 | −160 | 160 | 32 |
| 12 | −152 | 152 | 32 |
| 13 | −144 | 144 | 32 |
| 14 | −136 | 136 | 32 |
| 15 | −129 | 129 | 32 |
| 16 | −120 | 120 | 32 |
| 17 | −112 | 112 | 32 |
| 18 | −104 | 104 | 32 |
| 19 | −96 | 96 | 32 |
| 20 | −88 | 88 | 32 |
| 21 | −80 | 80 | 32 |
| 22 | −72 | 72 | 32 |
| 23 | −64 | 64 | 32 |
| 24 | −56 | 56 | 32 |
| 25 | −48 | 48 | 32 |
| 26 | −40 | 40 | 32 |
| 27 | −32 | 32 | 32 |
| 28 | −24 | 24 | 32 |
| 29 | −16 | 16 | 32 |
| 30 | −8 | 8 | 32 |

**Curve C** (N then NE; 54 points; chain point 37, entry point 12, release point 22).

| Point | *x* (leptons) | *y* (leptons) | *f* (1/256 turn) |
|---:|---:|---:|---:|
| 0 | −256 | 501 | 0 |
| 1 | −256 | 490 | 0 |
| 2 | −256 | 479 | 0 |
| 3 | −256 | 468 | 0 |
| 4 | −256 | 457 | 0 |
| 5 | −256 | 446 | 0 |
| 6 | −256 | 435 | 0 |
| 7 | −256 | 424 | 0 |
| 8 | −256 | 413 | 0 |
| 9 | −256 | 402 | 0 |
| 10 | −256 | 391 | 0 |
| 11 | −256 | 383 | 0 |
| 12 | −256 | 373 | 0 |
| 13 | −256 | 363 | 0 |
| 14 | −254 | 352 | 1 |
| 15 | −252 | 341 | 3 |
| 16 | −250 | 332 | 4 |
| 17 | −248 | 321 | 5 |
| 18 | −245 | 311 | 7 |
| 19 | −241 | 302 | 8 |
| 20 | −237 | 292 | 9 |
| 21 | −233 | 282 | 11 |
| 22 | −229 | 272 | 12 |
| 23 | −225 | 263 | 13 |
| 24 | −220 | 252 | 15 |
| 25 | −216 | 243 | 16 |
| 26 | −212 | 236 | 17 |
| 27 | −206 | 224 | 19 |
| 28 | −202 | 215 | 20 |
| 29 | −195 | 207 | 21 |
| 30 | −190 | 198 | 23 |
| 31 | −183 | 186 | 24 |
| 32 | −179 | 176 | 25 |
| 33 | −168 | 168 | 27 |
| 34 | −160 | 160 | 28 |
| 35 | −152 | 152 | 29 |
| 36 | −144 | 144 | 31 |
| 37 | −136 | 136 | 32 |
| 38 | −129 | 129 | 32 |
| 39 | −120 | 120 | 32 |
| 40 | −112 | 112 | 32 |
| 41 | −104 | 104 | 32 |
| 42 | −96 | 96 | 32 |
| 43 | −88 | 88 | 32 |
| 44 | −80 | 80 | 32 |
| 45 | −72 | 72 | 32 |
| 46 | −64 | 64 | 32 |
| 47 | −56 | 56 | 32 |
| 48 | −48 | 48 | 32 |
| 49 | −40 | 40 | 32 |
| 50 | −32 | 32 | 32 |
| 51 | −24 | 24 | 32 |
| 52 | −16 | 16 | 32 |
| 53 | −8 | 8 | 32 |

**Curve D** (N then E; 38 points; chain point 26, entry point 11, release point 19).

| Point | *x* (leptons) | *y* (leptons) | *f* (1/256 turn) |
|---:|---:|---:|---:|
| 0 | −256 | 245 | 0 |
| 1 | −256 | 235 | 0 |
| 2 | −256 | 224 | 0 |
| 3 | −256 | 213 | 0 |
| 4 | −255 | 203 | 0 |
| 5 | −253 | 192 | 0 |
| 6 | −251 | 181 | 1 |
| 7 | −249 | 171 | 1 |
| 8 | −246 | 160 | 2 |
| 9 | −243 | 149 | 3 |
| 10 | −240 | 139 | 4 |
| 11 | −236 | 127 | 5 |
| 12 | −232 | 117 | 8 |
| 13 | −228 | 109 | 12 |
| 14 | −222 | 99 | 16 |
| 15 | −219 | 90 | 20 |
| 16 | −213 | 82 | 23 |
| 17 | −206 | 72 | 27 |
| 18 | −201 | 64 | 32 |
| 19 | −195 | 56 | 36 |
| 20 | −186 | 48 | 39 |
| 21 | −177 | 43 | 43 |
| 22 | −168 | 36 | 47 |
| 23 | −160 | 32 | 51 |
| 24 | −147 | 27 | 54 |
| 25 | −135 | 23 | 57 |
| 26 | −126 | 20 | 60 |
| 27 | −113 | 17 | 62 |
| 28 | −104 | 13 | 63 |
| 29 | −94 | 9 | 64 |
| 30 | −84 | 6 | 64 |
| 31 | −75 | 4 | 66 |
| 32 | −64 | 3 | 64 |
| 33 | −53 | 2 | 64 |
| 34 | −43 | 1 | 64 |
| 35 | −32 | 0 | 64 |
| 36 | −21 | 0 | 64 |
| 37 | −11 | 0 | 64 |

**Curve E** (NE then SE; 61 points; chain point 45, entry point 15, release point 31).

| Point | *x* (leptons) | *y* (leptons) | *f* (1/256 turn) |
|---:|---:|---:|---:|
| 0 | −504 | −8 | 32 |
| 1 | −496 | −16 | 32 |
| 2 | −488 | −24 | 32 |
| 3 | −480 | −32 | 32 |
| 4 | −472 | −40 | 32 |
| 5 | −464 | −48 | 32 |
| 6 | −456 | −56 | 32 |
| 7 | −448 | −64 | 32 |
| 8 | −440 | −72 | 32 |
| 9 | −432 | −80 | 32 |
| 10 | −424 | −88 | 32 |
| 11 | −416 | −96 | 32 |
| 12 | −408 | −104 | 32 |
| 13 | −400 | −112 | 32 |
| 14 | −392 | −120 | 32 |
| 15 | −385 | −127 | 32 |
| 16 | −376 | −136 | 32 |
| 17 | −368 | −143 | 32 |
| 18 | −361 | −150 | 32 |
| 19 | −353 | −158 | 32 |
| 20 | −344 | −166 | 32 |
| 21 | −336 | −173 | 35 |
| 22 | −329 | −181 | 38 |
| 23 | −322 | −188 | 41 |
| 24 | −316 | −194 | 44 |
| 25 | −306 | −199 | 47 |
| 26 | −296 | −204 | 50 |
| 27 | −288 | −208 | 53 |
| 28 | −277 | −211 | 56 |
| 29 | −267 | −212 | 59 |
| 30 | −256 | −213 | 62 |
| 31 | −245 | −212 | 66 |
| 32 | −235 | −211 | 69 |
| 33 | −225 | −208 | 72 |
| 34 | −216 | −204 | 75 |
| 35 | −208 | −199 | 78 |
| 36 | −198 | −194 | 81 |
| 37 | −188 | −188 | 84 |
| 38 | −181 | −181 | 87 |
| 39 | −176 | −173 | 90 |
| 40 | −168 | −166 | 93 |
| 41 | −160 | −158 | 96 |
| 42 | −152 | −150 | 96 |
| 43 | −144 | −143 | 96 |
| 44 | −136 | −136 | 96 |
| 45 | −129 | −129 | 96 |
| 46 | −120 | −120 | 96 |
| 47 | −112 | −112 | 96 |
| 48 | −104 | −104 | 96 |
| 49 | −96 | −96 | 96 |
| 50 | −88 | −88 | 96 |
| 51 | −80 | −80 | 96 |
| 52 | −72 | −72 | 96 |
| 53 | −64 | −64 | 96 |
| 54 | −56 | −56 | 96 |
| 55 | −48 | −48 | 96 |
| 56 | −40 | −40 | 96 |
| 57 | −32 | −32 | 96 |
| 58 | −24 | −24 | 96 |
| 59 | −16 | −16 | 96 |
| 60 | −8 | −8 | 96 |

**Curve F** (NE then E; 56 points; chain point 44, entry point 16, release point 27).

| Point | *x* (leptons) | *y* (leptons) | *f* (1/256 turn) |
|---:|---:|---:|---:|
| 0 | −512 | 256 | 32 |
| 1 | −504 | 248 | 32 |
| 2 | −496 | 240 | 32 |
| 3 | −488 | 232 | 32 |
| 4 | −480 | 224 | 32 |
| 5 | −472 | 216 | 32 |
| 6 | −464 | 208 | 32 |
| 7 | −456 | 200 | 32 |
| 8 | −448 | 192 | 32 |
| 9 | −440 | 184 | 32 |
| 10 | −432 | 176 | 32 |
| 11 | −424 | 168 | 32 |
| 12 | −416 | 160 | 32 |
| 13 | −408 | 152 | 32 |
| 14 | −400 | 144 | 32 |
| 15 | −392 | 136 | 32 |
| 16 | −385 | 129 | 32 |
| 17 | −376 | 120 | 32 |
| 18 | −368 | 112 | 32 |
| 19 | −360 | 104 | 32 |
| 20 | −352 | 96 | 32 |
| 21 | −344 | 88 | 32 |
| 22 | −338 | 85 | 32 |
| 23 | −328 | 78 | 35 |
| 24 | −320 | 72 | 37 |
| 25 | −311 | 66 | 40 |
| 26 | −302 | 59 | 43 |
| 27 | −294 | 55 | 45 |
| 28 | −285 | 50 | 48 |
| 29 | −277 | 43 | 51 |
| 30 | −267 | 38 | 53 |
| 31 | −258 | 34 | 56 |
| 32 | −248 | 28 | 59 |
| 33 | −238 | 25 | 61 |
| 34 | −229 | 21 | 64 |
| 35 | −218 | 17 | 64 |
| 36 | −208 | 14 | 64 |
| 37 | −199 | 11 | 64 |
| 38 | −189 | 9 | 64 |
| 39 | −178 | 7 | 64 |
| 40 | −169 | 5 | 64 |
| 41 | −158 | 3 | 64 |
| 42 | −147 | 1 | 64 |
| 43 | −137 | 0 | 64 |
| 44 | −129 | 0 | 64 |
| 45 | −117 | 0 | 64 |
| 46 | −107 | 0 | 64 |
| 47 | −96 | 0 | 64 |
| 48 | −85 | 0 | 64 |
| 49 | −75 | 0 | 64 |
| 50 | −64 | 0 | 64 |
| 51 | −53 | 0 | 64 |
| 52 | −43 | 0 | 64 |
| 53 | −32 | 0 | 64 |
| 54 | −21 | 0 | 64 |
| 55 | −11 | 0 | 64 |

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
| turning in place | facing reaches the step's 8-direction | begin a track in that tick: *ρ* updated once and *s* earned, as in any moving tick (claim 12); a turn that completes at once begins the track in the next tick (S-0206 claim 12) |
| on a track | moving tick | it earns *s* at this tick's updated fraction (claim 10), passes the track points that its budget buys (claim 3), and stands between points as claim 4 gives |
| on a track (tick began on it) | track target reached, move continues, facing the next step, step check passes | the next track (claim 7) begins in the same tick: *ρ* is updated a second time, and the leftover, with the end correction of claim 3 applied, is spent on the new track (claim 12) |
| on a track (tick began without one) | track target reached, move continues | correction of claim 3; the tick ends with no track and the leftover kept; the next track begins next tick (claim 12) |
| on a track that began in this tick, after the track the tick began on ended | that track's target also reached in the same tick | correction of claim 3; the tick ends with no track (claim 12) |
| no track, move to make | facing differs from the current step | turn in place, *A* = 0 (claim 6) |
| on a track | track target reached, no step left | at rest, *ρ* = 0 (claim 11) |
| on a track | move stopped | target fraction becomes min(*ρ*ₜ, *F*₀.₃), destination cleared, finish the current track (claim 11) |
| driving or sailing, track target reached | mobile target passes the arrival range test, qualifying behaviour, no approach destinations | destination cleared, remaining steps discarded, target kept, stop there (claim 19 a) |
| hovering, step transition leaves no next step | the same arrival-time stop conditions hold | destination cleared, target kept; an already-computed final step is permitted, with no later travel until a new move (claim 19 d; S-0503 claim 7) |
| hovering, step transition leaves no next step | arrival-time stop conditions fail, retry timer still runs | wait with destination kept until a search can choose the next step (claim 19 d) |
| no track, step ahead, allied closed gate in the next cell | any tick | tells the gate to open and waits, without turning (claim 15) |
| no track, facing the step | first check: friendly obstacle, enemy obstacle, stationary friendly | discards the stored path and searches at once; repeated check on the same tick when the search finds a path (claim 15, T4) |
| no track, facing the step | first check: blocked | discards the stored path and searches on this tick regardless of the retry timer; a found path gets a repeated check on this tick (claim 15, T4; S-0212 claim 22) |
| no track, facing the step | first check: cloaked blocker | starts uncloaking every cloaked object in the cell's ground layer for every house (S-0516 claim 11 e), discards the stored path; searches on the same tick only if the path retry timer has run out |
| holding a path found on this tick | repeated check: friendly or enemy obstacle | attacks the first listed object if not allied (a wall if nothing is listed) by an override that gives no destination, so its move destination is put aside until the override is reverted; waits |
| no track, facing the step, first cell clear, curve chosen | second cell: friendly or enemy obstacle | discards the stored path; when the path retry timer has run out, searches on that tick, looks at the new first cell and makes a step check of the same kind, which can begin only a straight or diagonal track (claim 15, T5) |
| holding a path found on this tick | repeated check: stationary friendly | gives up the move if the give-up test holds (F7), otherwise sends a forced, move-anywhere scatter request to the cell, waits |
| holding a path found on this tick | repeated check: cloaked blocker or blocked | gives up the move |
| any object of the vehicle type kind, whatever its locomotor | scatter request accepted, no threat point | destination from the nearby-cell search, behaviours unchanged, no draw (claim 16) |
| any object of the vehicle type kind, whatever its locomotor | scatter request accepted, threat point | one draw over 0 to 2 (after a draw over 1 to 4 when it has a target and the request is not forced), destination among the eight neighbours, *move* queued (claim 16) |
| teleport-capable vehicle (claim 28), no warp-pending immobilization, not gripped, not seized | effective destination assignment that is not a qualified docking assignment, empty included | temporary driving installed unless already driving; the destination goes to driving movement, no landing point fixed; a pending jump is cancelled, *L* and its reservation retained (claim 21, claim 28) |
| teleport-capable vehicle outside claim 28's three exclusions, teleporting, jump made | the jump removes its destination | driving active from the jump tick on (claim 28) |
| teleporting, idle (for a teleport-capable vehicle outside claim 28's three exclusions, only a qualified docking assignment reaches this row, claim 28) | destination given, not disabled, not held, not being erased, not phasing in, no suppression pending | landing point *L* fixed at once, kind-specific occupancy reservation moved to *L* (claim 20 a–c) |
| teleporting, any state | effective nonempty destination assignment, suppression pending (claim 27) | no new landing point fixed or jump scheduled by this assignment; any earlier landing point, pending jump and reservation unchanged; destination kept; suppression consumed (claim 27, Q-1130 table, second row) |
| teleporting, *L* fixed | its next movement update, *L* differs from its position | jump: target-loss and revert subject to claim 20 e's airstrike and restored-target distinctions, projectiles lose it, kind-specific arrival and crate rules apply, destination removed; phasing in for *D* ticks (F9), none for a harvester-flag type (claim 20 e, f) |
| teleporting, *L* fixed, jump pending | empty destination assigned | pending jump and moving state cancelled; remembered *L* and its occupancy reservation retained (claim 20 h) |
| teleporting, *L* fixed, no pending jump (cancelled by an empty assignment or by temporary driving's installation) | its next movement update | no jump; the jump row above does not apply to it; remembered *L* and its reservation kept until the next accepted destination (claim 20 h; claim 21) |
| teleporting, *L* fixed | its next movement update, *L* equals its position or none was fixed | destination removed, no jump (claim 20 b, d) |
| phasing in | destination given | dropped (claim 20 a), unless suppression is pending (claim 27): then kept, no landing point |
| phasing in | first tick with *t* − *t*₀ ≥ *D* (at least *t*₀ + 1) | phasing in ends; only with no target, conditional timer shortening then gated teleport search, whose normal timer draw replaces the shortening when both occur; idle fallback when no target is set (claim 20 f) |

A change of speed fraction takes effect in the progress of the same tick (claim 10).

Ordering stated as consequences (claim 15): a vehicle reacts to its step check on the tick on
which its facing first matches the step, so a vehicle that arrives facing another way turns first
and reacts only then, while an allied closed gate starts opening on its first tick in front of
the gate, before the vehicle turns. The scatter request of a blocked vehicle reaches every listed
object on the tick it is sent (S-0209, State and ordering), so an idle friend that accepts it has
its new destination from that tick.

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
  most one cell per tick on straight and diagonal steps and their extra budget piles up unused. Off
  (a design behaviour): no limit on the tracks finished in a tick, with one fraction update at the
  start of the tick and one more at each track that begins after a track end in it, each towards
  the target fraction of the track that begins (claim 12 gives the full rule).
- **`quirks.drive_leftover_kept_at_rest`** (claim 3). Default (original): a vehicle that stops at
  its destination keeps its leftover budget, which speeds up the first tick of its next move if
  that move needs no turn in place. Off: the leftover budget is cleared when the vehicle comes to
  rest. Below the track-end cap the retained leftover is at most one tick's progress *s* plus 5
  progress units (the largest end correction of claim 3), so it shortens the next move by about
  one tick, and by a few ticks only for the slowest types. Above the cap, claim 12 limits the first
  tick to one track whatever the leftover.
- **Asymmetric acceleration** (claim 10) is original behaviour and has no switch, because it is how
  the acceleration and deceleration fields are defined: mods tune the fields with it in mind, and a
  symmetric rule would only change what those fields mean.
- **Progress units are not leptons.** A vehicle covers about 11/7 of a lepton per progress unit on a
  straight step (256 leptons for 163 units) and about 1.6 on a diagonal one (362 for 227). Specs and
  tools that need speed in cells per tick must use F3, not a lepton conversion.
- **Budget lost when stopping to turn** (claim 6). Leftover progress is discarded whenever the vehicle
  turns in place, and whenever it waits for a gate or its step check refuses it a track, so a path with many sharp turns
  costs slightly more than the track sums. This is original behaviour and has no switch: it is also the only rule that empties the budget
  that piles up above the track-end cap (claim 12), so turning it off would need a new limit of its
  own, which is a design change rather than a quirk.
- **Odd points in the track data** (T1) are engine constants of the original, and our engine keeps them.
- **Slope factors at the ends of a bridge deck** (claim 9 (c)). The slope factor compares terrain
  heights, so the track onto a high bridge from land at deck level gets the downhill factor and the
  track off it gets the uphill factor, although the deck is level. This is original behaviour and
  has no switch: it matters only where the uphill or downhill factors differ from 1, it is how
  those fields act on every other change of terrain height, and a rule that compared deck heights
  instead would change what the fields mean for mods tuned with them.
- **Table root in the braking distance** (claim 10, F5). The distance to the move destination uses
  the table square root, so it can be a lepton or more short of the exact distance from 129 leptons
  on, and a vehicle can start braking slightly before it is within *D*ₛ by the exact distance. This
  spec defines no switch of its own: S-0204's `quirks.table_root_distance` governs it.
- **An unknown class identifier** (claim 14) stops the original with a program error when an object
  of the type is created. Our engine keeps the locomotor in force and warns: a departure, with no
  switch, since no engine reproduces a crash.
- **A type without a locomotor field teleports** (claim 14), whatever its kind. This is original
  behaviour and has no switch: it is the meaning of the absent key, and a mod that wants another
  locomotor names it.
- **A teleport distance factor of 0 with the trigger flag on** (F9) makes the original divide by
  zero and stop with a program error at the first jump. **Our engine** treats it as the trigger
  flag off (*q* = 0) and gives an importer warning. This is a departure (bad data), with no switch.
- **A teleporting object ignores terrain between its position and its landing point** (claim 20 b).
  It can reach cells that no path reaches. This is original behaviour, not a bug, and has no switch.
- **`quirks.teleport_aircraft_ground_reservation`** (claim 20 a, e; decided at the delegated sign-off,
  2026-10-25).
  Default original: reserve ground at terrain height, retain that mark through deck
  placement, clear the remembered ground mark on the next accepted destination, and
  clear only actual occupancy on removal, not a separate remembered or pending
  reservation (claim 20 k). Destruction on deck can therefore leave the old ground
  object-present flag, or a replacement landing cell's ground flag. Off: reserve only
  the eventual landing layer, clear that reservation when replacing it, and clear
  both the pending reservation and actual occupancy on removal, leaving no orphan
  reservation. Shared with S-0209; D22 runs 14 and 15 test the default.

- **`quirks.teleport_vehicle_ground_reservation`** (claim 20 a, e; decided at the delegated sign-off,
  2026-10-25).
  Default (original): a teleporting vehicle reserves the ground record even for a high-bridge
  landing, remembers that terrain-height point for later reservation clearing, and keeps the
  ground reservation through deck placement; placement also marks the deck record.
  Removal clears actual occupancy only, leaving any separate remembered or pending
  ground reservation (claim 20 k). Temporary driving leaves a previous deck landing's
  ground reservation behind when driving departure clears the actual deck mark
  (claim 21). Off:
  reserve the eventual landing layer from fixation onwards (deck on a high bridge, ground
  elsewhere), clear the earlier reservation from that same layer when replacing it, and keep
  no separate ground reservation beneath a deck landing. Placement still has claim 20 e's
  deck height and listing. Removal clears both a pending reservation and actual
  occupancy, leaving no orphan reservation. S-0209's ordinary entry rules apply to the selected record in
  either setting. The switch changes no landing search, delay or random draw by itself.
- **Orders given while phasing in are lost** (claim 20 f), not deferred to its end, except a destination that a pending suppression keeps (claim 27). Original
  behaviour, no switch.
- **`quirks.drive_giveup_ignores_contact`** (claim 15, F7). Default (original): at a step check the
  give-up test against stationary friends does not ask about contact, so a vehicle within the
  close-enough distance of its destination gives up even while it is in contact with another object,
  and only the look at the first cell of a new path asks. Off: every give-up test of claim 15 also
  requires the vehicle to be in contact with nothing, as a walker's does (S-0209 claim 8). No
  measurement setup is known that tells the two apart (scenario D17, switch note).
- **`quirks.drive_duplicate_scatter_request`** (claim 15; decided at the delegated sign-off,
  2026-10-25).
  Default (original): a blocked vehicle can send its request twice in one tick, once when it
  looks at the first cell of the path it has just found and once at the repeated check, when
  the cell still gives *stationary friendly* then. Each request reaches the objects eligible
  under S-0209 claim 11, and each infantry unit that accepts takes its own draw (S-0208
  claim 13), so the second request can change the draw count.
  Off (design alternative): among claim 15's requests from one vehicle to the same cell and
  cell layer in one tick, only the earliest is sent. A later otherwise-required request to
  that same layer is omitted entirely, including recipients' draws; requests to a different
  cell or layer, requests from another sender, and requests on later ticks are unaffected.
  Entry checks, path searches, give-up tests and their non-request effects are unchanged:
  suppressing a duplicate request does not suppress the repeated check or its wait.
  The default's double-request result remains `unmeasured`, with low confidence, for claim
  15's stated reason: no established setup keeps an accepting infantry receiver stationary
  after its first draw while also preserving the vehicle's path through that cell.
- **`quirks.drive_nearest_from_top_corner`** (claim 15, F8). Default (original): the nearest object
  in the look at the first cell of a new path is the one whose position lies nearest to the top
  corner of the cell holding it, along the map, so an infantry unit at the top position wins over
  a vehicle or infantry unit at the centre, and the centre over the right, left and bottom
  positions. Off: the distance is measured from the centre of that cell, so an object at the
  centre is nearest and the four outer standing positions are equally far. The switch changes
  which house decides between giving up, the request and nothing when the layer lists objects of
  more than one house. Scenario D17 part i tells the two apart.
- **A scatter without a threat point queues no *move*** (claim 16). The vehicle drives to the scatter
  cell in whatever behaviour it has, so after arriving it does what that behaviour does with no
  destination (S-0215). Original behaviour, no switch.
- **A teleporting vehicle keeps a destination without a landing point under pending suppression**
  (claim 27, Q-1130, answering Q-1131). Original behaviour, with no quirk switch, by lead decision
  on Q-1131. The rule is stated as the claim: the state arises from a pending suppression at an
  assignment to a teleporting vehicle in any state (claim 27, Q-1130 table, second row), so it is not
  limited to disabled, held, erased or phasing-in vehicles. The destination stays, no new landing
  point is fixed and suppression is consumed. No jump towards that new destination follows until
  a later effective assignment; an earlier pending jump is unchanged. The
  reason there is no switch: no ordinary or scripted play is known to reach the state (claim 27,
  reachability), so a switch would select between two behaviours nobody can observe. Consequently
  the exceptions that S-0215 claim 2 and S-0514 claim 17 make for a vehicle under pending
  suppression (the destination is kept while erased or in the leaving stage) apply
  unconditionally; there is no alternative position in which they are removed.

## Import mapping

| Input | Concept | Form and absent-key rule |
|---|---|---|
| No additional key | Warp-pending immobilization, claim 27 | Runtime external-teleport commitment, initially absent; not a cooldown or an independently imported delay. Warp timing belongs to S-0513 claim 39 |
| No additional key | Restored predecessor's same-assignment destination, claim 27 | Runtime destination routing; no imported ordering option or absent-key value |
| vehicle type section, key `Teleporter` | Teleport-capable flag for claim 27's dock test and claim 28's choice of locomotor, independent of current locomotor | Boolean; keeps the earlier layer's value; engine default no |
| building type section, key `DockUnload` | Dock-unloading permission in claim 27 | Boolean; keeps the earlier layer's value; engine default no. Refinery interaction belongs to S-0402 claim 10 |
| No additional key | Factory exit heading height, claim 24 | Engine constant 0 leptons; not imported from terrain or rules |
| No additional key | Driving/ship locomotor destination lifetime, claim 25 | Runtime movement state, initially absent; assignment and clearing are not imported settings |
| No additional key | Destination acceptance lift and arrival height tolerance, claim 25 | Engine constants: 416-lepton high-bridge lift and strict 208-lepton arrival tolerance |
| No additional key | Order-level goal removal at rest, claim 26 | Runtime destination kind (cell, building, vehicle, infantry unit or aircraft), locomotor point and mover behaviour state; the named unit's moving/stationary state does not change cleanup eligibility. No imported delay, alignment threshold or absent-key default |

**T3 Locomotor input-value mapping** (claim 14). Each row maps a value of `Locomotor` in the
user's rules file to a movement method. The five numbers are dimensionless nonnegative integers,
written here in decimal. To interpret a data-file value, read its five hexadecimal groups as
integers and compare the resulting five numbers with a row. Equivalently, to write a test value,
express the row's numbers in hexadecimal, with leading zeroes to widths 8, 4, 4, 4 and 12 digits,
join them with hyphens, and enclose the result in braces. There are no spaces inside the value.
This is solely the text-input mapping: it specifies no program class name, internal identity,
representation or storage order. The recognised values are engine constants, not stock rule
settings. Claim 14 supplies confidence and backing; D16's identifier controls test every row.

| Movement method (ours) | First group | Second group | Third group | Fourth group | Fifth group |
|---|---:|---:|---:|---:|---:|
| driving (this spec) | 1247291201 | 38969 | 4561 | 46857 | 687813275601 |
| hovering | 1247291202 | 38969 | 4561 | 46857 | 687813275601 |
| tunnelling (underground travel, S-0209 claim 5b) | 1247291203 | 38969 | 4561 | 46857 | 687813275601 |
| walking (infantry walking, S-0208) | 1247291204 | 38969 | 4561 | 46857 | 687813275601 |
| drop pod | 1247291205 | 38969 | 4561 | 46857 | 687813275601 |
| flying | 1247291206 | 38969 | 4561 | 46857 | 687813275601 |
| teleporting (the absent-key default) | 1247291207 | 38969 | 4561 | 46857 | 687813275601 |
| walking vehicle (mechanical walking; S-0210) | 1439777208 | 56212 | 4561 | 44184 | 412451429301 |
| ship | 736785633 | 31946 | 4563 | 48660 | 69984231788 |
| jumpjet | 2455841862 | 63263 | 4561 | 44191 | 412451429301 |
| rocket | 3082065766 | 58742 | 4563 | 39897 | 69987676136 |

| User data | Feeds | Form | Absent |
|---|---|---|---|
| vehicle section, key `Speed` | speed field *v* (claim 1), also the lead estimate (claim 18) | integer (S-0003, claim 3) | keeps value; engine default type speed 0; −1 also keeps |
| vehicle section, key `Accelerates` | acceleration flag | boolean | keeps value; engine default yes |
| vehicle section, key `AccelerationFactor` | *a* | decimal (S-0003, claim 6), widened to double | keeps value; engine default the double nearest 0.03 (claim 17) |
| vehicle section, key `DeaccelerationFactor` (spelled this way) | *η* | decimal, widened to double | keeps value; engine default the double nearest 0.002 (claim 17) |
| vehicle section, key `SlowdownDistance` | *D*ₛ | integer (S-0003, claim 3), leptons | keeps value; engine default 500 (claim 17) |
| vehicle section, key `Passive` | passive flag | boolean | keeps value; engine default no |
| vehicle section, key `SpeedType` | locomotion class (S-0211) | name | S-0211 claim 3 |
| `[General]`, keys `TrackedUphill`, `TrackedDownhill`, `WheeledUphill`, `WheeledDownhill` | uphill and downhill factors for class *track* and for every other class (claim 9) | decimal, widened to double | keeps value; engine default 1.0 |
| `[General]`, key `VeteranSpeed` | *ν*, for progress and the lead estimate (claims 2 and 18) | decimal, widened to double | keeps value; engine default 1.0 |
| vehicle section, keys `VeteranAbilities`, `EliteAbilities` | whether the *faster* ability applies | list of ability names (S-0311, Import mapping) | as in S-0311 |
| `[AudioVisual]`, key `ConditionYellow` | yellow ratio (claim 9 e) | as in S-0108 | as in S-0108 |
| country section, key `SpeedUnitsMult`; the country's matching keys for infantry and aircraft do not feed this spec | *κ*, for progress and the lead estimate (claims 2 and 18) | decimal (S-0003, claim 6), stored in single precision | keeps value; engine default 1.0 |
| land-type sections | *M* | S-0211 | S-0211 |
| `[General]`, key `Wake` | game-wide wake animation (claim 23) | animation type name | keeps value; engine default none |
| No data key | driving and ship heading-point height at track start and retention through chained curves (claim 24; Q-0676) | integer world-height leptons copied from runtime vehicle height; no independent imported height | fixed behaviour; target terrain and bridge data do not replace the copied height |
| No data key | wake's integer-rate test, active-track heading eligibility and post-movement creation timing (claim 23) | engine behaviour; rate reuses the speed, country and veterancy fields above and runtime speed fraction and bonus | fixed rules of claim 23; no separate wake-speed field |
| vehicle section, key `Underwater` | underwater-type flag; does not suppress driving wakes (claim 23) | boolean | keeps value; engine default no |
| infantry, vehicle and aircraft type sections, key `Locomotor` (building type sections read it too, without effect) | locomotor (claim 14, table T3) | class identifier: hexadecimal digits in groups of 8, 4, 4, 4 and 12, joined by hyphens, in braces, letters in either case; the value is read whole after trimming. A value of another form is ignored; an identifier not in T3 is treated as not given (departure, claim 14); both warn | keeps value in force; engine default the teleporting locomotor |

The rows for the type speed, country factor and faster ability also feed the lead estimate
(claim 18). Its speed bonus and speed fraction are runtime state (claims 9 to 13), not separate
imported lead fields. No key supplies a progress-to-lepton conversion or a waiting-speed override.

| `[General]`, key `ChronoDistanceFactor` | distance factor *k* (F9) | integer (S-0003, claim 3), leptons per tick | keeps value; engine default 32 |
| `[General]`, key `ChronoTrigger` | trigger flag (F9) | boolean | keeps value; engine default yes |
| `[General]`, key `ChronoMinimumDelay` | minimum delay *m* (F9) | integer, ticks | keeps value; engine default 0 |
| `[General]`, key `ChronoRangeMinimum` | range minimum *g* (F9) | integer, leptons | keeps value; engine default 0 |
| `[General]`, key `WarpOut` | the warp animation played at both ends of a jump (claim 20 e) | animation type name | keeps value; with none in force, no animation is played |
| `[AudioVisual]`, keys `ChronoOutSound`, `ChronoInSound` | game-wide departure and arrival sounds of a jump (claim 20 e) | sound name | keeps value; with none in force, no sound is played |
| vehicle, infantry and aircraft type sections, keys `ChronoOutSound`, `ChronoInSound` | the type's own departure and arrival sounds, used instead of the game-wide ones when given | sound name | the game-wide sound |
| vehicle type section, key `Harvester` | harvester flag: no phasing in after a jump (claim 20 f, F9); its economic meaning is S-0402's | boolean | keeps value; engine default no |

The other chrono keys of `[General]` (the chronosphere's delays and its landing warhead) feed the
superweapon specs, not claim 20.

| No data key | non-infantry entry-test level, unrestricted reach comparison for a destination in no region, kind-specific reservation and landing (claim 20) | engine behaviour, not imported fields | fixed rules of claim 20 |
| No data key | vehicle reservation and clearing layer, retained landing point and two-record bridge landing (claim 20 a, e; Q-0240) | runtime occupancy behaviour; deck-mode search does not select the reservation record | fixed engine rules; original behaviour by default under `quirks.teleport_vehicle_ground_reservation` |
| No data key | aircraft ground reservation, deck placement and removal clearing (claim 20 e; Q-0310) | runtime occupancy behaviour, no imported value | original by default under `quirks.teleport_aircraft_ground_reservation` |
| Map cell tags and entered-by events | arrival-trigger eligibility and actions (claim 22) | import forms and absent defaults belong to S-0011; movement adds no field | no arrival raise without a cell tag |
| No data key | retained teleporting state and no teleport destination handling during temporary driving (claim 21) | runtime state, not imported fields | a newly created teleporting object has no pending jump and no phasing in |
| No data key | vehicle and aircraft removal with a remembered or pending reservation (claim 20 k; Q-0356) | runtime occupancy clearing, no imported value | actual-cell-only clearing under the respective ground-reservation quirk, default original |
| No data key | previous deck landing during temporary driving and restoration (claim 21; Q-0356) | runtime occupancy behaviour, no imported value | ordinary driving clears actual deck occupancy on departure; restoration relocates no mark |

Animation type fields governing creation draws are imported by S-0109 and S-0202,
there is no jump-specific key that suppresses those draws. Temporary factory driving and
fall-back restoration have no additional key here (S-0417 and S-0215).

No other speed field of a country or of a difficulty level enters the formulas of this spec
(Open questions).

Claim 19 has no key of its own: the range test's fields are S-0307's, and the path-retry delay field that sets *R* is imported as S-0212 says.

No key feeds the destination of the override of claim 15 or the choice of the nearest object in
the look at the first cell of a new path (F8): both are engine rules. The close-enough distance
of the give-up test is imported as S-0209 says.

| Input | Concept | Parse form | Absent-key default |
|---|---|---|---|
| Vehicle type `Teleporter` | teleport-capable flag in claims 27 and 28, independent of current locomotor | boolean (S-0003) | false |
| No data key | claim 27's assignment-time dock evaluation, driving restoration permission, placement clear and point suppression | runtime behaviour, no numeric parse or tuning field | refresh, point suppression and restoration lock absent on a new driving vehicle |
| Building type `DockUnload` | permits dock unloading in claim 27 | boolean (S-0003) | false |

Claim 27's destination-refresh and one-application suppression states are runtime
behaviour, with no imported tuning.

## Measurement scenarios

All scenarios run under the glossary's *measurement setup* on a purpose-made flat map of clear
ground, unless the scenario says otherwise, at game speed index 2 unless a scenario chooses another index (claim G3 of the glossary
makes these tick-based movement results speed-independent), with no engine add-ons or mods
apart from each scenario's named test files.
Use one human house and one bystander computer player kept outside the test lanes and cells.
Those lanes and cells are the observed area. Void a run if an unnamed bystander object enters
that area, acts on a test object, or the bystander is defeated before recording ends. Named
computer-player test objects in a scenario are exceptions to the exclusion, not to its other
qualification conditions. A *recorder* reads, each tick, the vehicle's position in leptons and its facing, and, where a
scenario says so, other state of the units: a vehicle's height, speed fraction *ρ* or speed bonus
factor *b*, or its stored path. Test values are
set by override sections in the test map (a named test file, removed after the run). All values
in them are invented test values:
- the test vehicle type's speed field (given per scenario), with the acceleration flag off unless
  the scenario says otherwise,
- its locomotion class set to wheel, with the clear land type's wheel multiplier at 100 %,
- the owner's country speed factor for vehicles set to 1.0,
- for the uphill and downhill factors, 1.0 unless the scenario says otherwise.

The vehicle starts at full health and without veterancy unless a scenario says otherwise. Each
variant runs once unless the scenario gives a run count; random-dependent sub-tests use their
stated draw-recording and qualifying-run conditions. "Arrival at
cell *k*" means the first tick at whose end the vehicle's position is at or past the centre of the *k*-th cell of the route, along
its direction of travel. (When claim 12 permits the next track to begin in the same tick,
the position at the end of that tick may already be a little past the centre.) Paths are logged as the
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
- *Falsified if:* a value for field 20 or 40 lies outside the expected range,
  or field 100 takes other than 10 ticks (which also falsifies the default of
  `quirks.drive_track_end_cap`; without the cap it would take 6 or 7 straight and 9 diagonal). A
  lepton reading of the speed (10 × 256/*s* for straight, about 50 ticks at field 20) falsifies
  claim 3.

**D2: speed field limits (claim 1).**
- *Setup:* as D1 (straight), with speed fields 150, 100, 1, 0, −1 and −5. For −1, also run a
  control without the override.
- *Observable:* as D1, arrival tick at cell 20 minus arrival tick at cell 10; for field 0, whether
  the vehicle moves at all.
- *Expected:* 150 and 100 give the same result. Field 1 (*s* = 2) takes 815 ± 1 ticks for cells 10 to
  20. Fields 0 and −5 never move. Field −1 behaves like the control.
- *Falsified if:* 150 is faster than 100, field 1 is off by more than a tick, −1 differs from the
  control, or field 0 or −5 moves.

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

**D5: slopes, the zero fallback, the bridge deck and deceleration (claims 5, 9 and 10).**
- *Setup:* run 1: a route north that climbs one height level over a slope cell and later descends
  one. Overrides set the wheeled uphill factor to 0.5 and the wheeled downhill factor to 1.0. Run 2
  (control): a tracked vehicle with the tracked factors left at 1.0. Run 3: as run 1 with the
  wheeled uphill factor 0. Run 4: a purpose-made map with a straight high bridge over flat water,
  with land at deck level at both ends, as in S-0208 scenario W2 (e). The vehicle starts on the land
  two cells before the deck and is ordered along the bridge to the land two cells past it.
  Overrides set the wheel multiplier of the road land type to 100 % and that of water to 0, and
  the wheeled uphill and downhill factors to 0.5 each. Which cells carry the deck follows from the
  test map by S-0211 claim 15, and the recorder takes each cell's height level from the test map.
  Run 5: as run 1, with the acceleration flag on, acceleration factor 1.0, deceleration factor
  0.001 and slowdown distance 0. Run 6: a purpose-made map with a straight high bridge over flat
  clear ground at height level *L*, its ends on land at level *L* + 4, and a corridor at level *L*
  that passes under the deck across the bridge. The vehicle drives along the corridor under the
  deck. Overrides set the wheel multiplier of the road land type to 50 % (clear stays at 100 %).
- *Observable:* runs 1 to 4 and 6, the vehicle's position and height read by the recorder on every
  tick; runs 1 to 3, the ticks spent on the track into the uphill cell, against a flat cell; run 4,
  the ticks spent on the track from the land onto the first deck cell, on the track from the last
  deck cell onto the land, and per cell along the middle of the deck; run 5, the speed fraction
  read by the recorder on each tick of the track into the uphill cell; run 6, the ticks spent on
  the track into the cell under the deck, against a flat cell.
- *Expected:* run 1 takes about twice as long uphill (*s* = 25 against 51). The tracked vehicle of
  run 2 is not slowed. Run 3 climbs at the same rate as run 1 (*z* = 0.5 by part (d), so *s* = 25),
  and does not stop. In runs 1 to 3, from the first track point on, the height at the end of every
  tick is the surface height *H* (S-0106, claim 2) at the ground position of the last track point
  the vehicle passed, track targets included (claim 5). Run 4: along the middle of the deck,
  163/51, so 3 or 4 ticks per cell (*ℓ* is *road* by part (a), and the slope factor is 1). Where
  the ground under the first and the last deck cell is flat water 4 height levels below the land,
  the track onto the first deck cell takes 163/25, so 6 or 7 ticks (downhill factor, claim 9), and
  so does the track from the last deck cell onto the land (uphill factor). From the first point in
  the first deck cell to the last point in the last deck cell, the height is *H* of the water
  under the deck plus 416 leptons, the same world height as on the land at both ends, and on the
  land it is *H* of the land. In run 5 the fraction falls from 1 by 51 × 0.001 = 0.051 at each
  update (case 5), not to 0.5 at once; the track ends before it reaches 0.5. Run 6: under the deck
  the height is *H* of the ground there (no 416 added), and the track into the cell under the deck
  takes as long as one into a flat clear cell, 3 or 4 ticks (*ℓ* is the ground's clear land type,
  not *road*). Precondition for run 6: the test map must give a high bridge whose ground cells a
  wheeled vehicle can drive through; if the vehicle cannot pass under the deck, run 6 is
  inconclusive.
- *Falsified if:* the wheeled vehicle of run 1 is not slowed uphill; the tracked control is
  slowed; run 3 stops in front of the slope or climbs at full speed; in runs 1 to 3 a height
  differs from *H* at the last point passed; in run 4 a track along the middle of the deck takes
  about 6 or 7 ticks (the water's multiplier 0 replaced by 0.5), the track onto the first deck cell
  or off the last one takes 3 or 4 ticks (no slope factor at the ends of a deck), or the height
  at a point on the deck is *H* of the water without the 416 leptons, or the height on the land
  past the deck is *H* of the land plus 416 leptons (the vehicle stayed on the deck); in run 5 the fraction drops
  to 0.5 in one update or falls by 0.001 per update; in run 6 the height under the deck is *H* plus
  416 leptons, or the track into the cell under the deck takes 6 or 7 ticks (the vehicle treated as
  on the deck, with *ℓ* = *road*).

**D6: damage (claim 9 e).**
- *Setup:* as D1 (straight, field 20), with the vehicle's health set by the map at or just below the
  yellow ratio, and a second run clearly above it.
- *Observable:* as D1, arrival tick at cell 20 minus arrival tick at cell 10.
- *Expected:* 43 ± 1 ticks for cells 10 to 20 in the first run (*s* = 38), and 32 ± 1 in the second.
- *Falsified if:* either run lies outside its expected tick range, including if both runs are equal.

**D7: curve against turn in place (claims 5 to 7).**
- *Setup:* field 20, acceleration off. Map A: a corridor one cell wide that runs 5 cells north and
  then 5 cells east (an L). Map B: a corridor 5 cells north, then an open area to the south-east of
  its end, with the destination 5 cells south-east of the corridor's end. Run 3: map B again, with
  the acceleration flag on, acceleration factor 0.02, deceleration factor 0 and slowdown distance
  0; the recorder also reads the speed fraction.
- *Observable:* the logged path, the positions and facings on every tick through each turn, the
  ticks from arrival at the cell before the turn to arrival at the cell after it, and the facing at
  the end of each turn; in run 3, the speed fraction on the tick on which the vehicle reaches the
  shared cell's centre and on the first tick of the track after its turn in place.
- *Expected:* at every logged pair (N, E), the vehicle drives one curve (curve D) from the centre of
  the cell before the corner to the centre of the cell after it, without passing the corner cell's
  centre. It takes 273/51, so 5 or 6 ticks, and ends facing map east. At every logged pair whose
  directions differ by 3/8 turn (for example N then SE), it reaches the shared cell's centre, stops,
  and turns in place (its duration belongs to S-0206, claim 12) before driving on. On every tick on
  a curve, the facing is 256 *f* for the last point of the curve (T1, transformed as T2 gives) at
  or before the tick's position along the curve, so it changes only on ticks on which a point is
  passed. In run 3, the fraction on the first tick after the turn is the fraction at the shared
  cell's centre plus the acceleration factor (one update, claim 12; the turn in place makes no
  update), not the factor alone.
- *Falsified if:* at a logged (N, E) pair the vehicle passes through the corner cell's centre; at a
  3/8-turn pair it curves; a facing on a curve lies between the facings of two consecutive
  points; or in run 3 the fraction after the turn equals the acceleration factor (a restart from 0).
- *Track-table sweep:* make one corridor variant for every current/next direction pair of T2,
  with no crushable object or overlay, passive flag off, field 1 and acceleration off. Give
  the move order once per variant. Only a run whose recorded path contains the intended pair
  qualifies. Record positions, facings and height each tick. Claims 2 to 5 and T1/T2 predict
  the complete trace, including the terminal point and the end correction. A different
  trace, an incorrect transformed facing, or a curve at a pair marked with no curve falsifies
  the corresponding table entry. This sweep covers the curves and transforms not isolated
  by the two main maps; a variant with no qualifying path is inconclusive for that entry.

**D8: positions between points, and the leftover budget (claims 3 and 4).**
- *Setup:* speed field 1 (*s* = 2), acceleration off. A map-placed vehicle that has not moved
  before (so it has no leftover budget, claim 3), already facing map north. Straight route north.
  Part a: it is ordered 10 cells north. Part b: it is ordered 2 cells north, comes to rest there,
  and is then ordered 10 cells further north. Part c: as part b, but the second order is to a cell
  2 cells east, so it turns in place first. Part d: as part b, with a closed gate of its own house
  placed by the map in the cell north of where it comes to rest, so that it waits for the gate to
  open before its second move. Part e: a single order 10 cells north, with a closed gate of its own
  house placed by the map in the third cell north, so that the vehicle reaches the centre of the
  second cell on a tick that began on a track, finishes that track and is then held by the gate in
  the same tick (claim 15).
- *Observable:* the displacement from the start of each move for its first eight moving ticks (in
  part c, from the first tick of the track after the turn; in part d, after the wait); in part e,
  the displacement from the second cell's centre for the first eight moving ticks after the wait.
- *Expected:* part a: 3, 7, 10, 12, 16, 20, 21, 25 leptons. Part b: the first move ends with a
  leftover of 6 progress units (claims 3 and 12, computed), so the second move's first four values
  are 12, 16, 20 and 21 leptons. Parts c and d: the turn and the wait empty the budget, so the
  second move starts like part a (3, 7, 10, …). Part e: the vehicle reaches the second cell's
  centre with the same leftover of 6 as in part b, and the wait on that tick empties it (claim 3),
  so after the wait it goes on like part a (3, 7, 10, …).
- *Falsified if:* the sequence of part a differs. An evenly spaced sequence (3, 6, 9, …) indicates
  the "off" behaviour of `quirks.drive_subpoint_compounding`. A seventh value of 22 instead of 21
  indicates that the interpolation factor is rounded to nearest rather than toward zero (F4; S-0205,
  number domains). In part b, a second move that starts 3, 7, … (the leftover dropped at rest); in
  part c, d or e, one that starts 12, 16, … (the leftover kept through the turn or the wait).

**D9: no chaining (claims 7 and 8).**
- *Setup:* field 20, acceleration off (needed for the passive control, see claim 8). A corridor one
  cell wide shaped so that the only path is N, N, NE, E, E, … (two turns in consecutive cells).
  Control 1: the same with the passive flag set by override. Control 2: as control 1 with the
  acceleration flag on. Control 3: as control 1, with a vehicle of the same house placed by the map
  in *sticky* in the cell that the step after the first curve leads to. Control 4: as control 1,
  with a vehicle of the same house that map triggers drive back and forth through that cell instead,
  so that the cell gives *moving blocker* while that vehicle is in it with a destination.
- *Observable:* the logged path, the positions and facings read by the recorder, and from them the
  ticks at which the vehicle stops and turns at the middle cell of each logged triple; for control
  2, also the speed fraction; for control 4, also the other vehicle's position and destination and
  the cell's occupancy record on the tick on which the test vehicle reaches the chain point.
- *Runs:* 1 each for the ordinary vehicle and controls 1 to 3. Control 4 is repeated, with the
  order given at varied ticks, until a *qualifying run*, up to 10 runs: one in which, on the tick on
  which the test vehicle reaches the chain point, the other vehicle is in that cell with a
  destination and the cell's occupancy record shows a vehicle present, so that S-0209 (rows VP22
  and VR5) gives *moving blocker*. If no run qualifies, control 4 is inconclusive.
- *Expected:* at every logged triple (*d*, *e*, *g*) where *e* is 1/8 turn from *d* and *g* ≠ *e*, the
  ordinary vehicle drives the curve of the pair (*d*, *e*) to its target's centre, stops there, turns
  in place to *g*, and then drives on. Control 1 does not stop or turn in place at that cell: once
  past the chain point of the first curve (T1), its positions lie on or between consecutive points
  of the curve of the pair (*e*, *g*) from that curve's entry point on (T1, transformed as T2
  gives), with the track target one cell further on. Control 2 never moves, and its fraction stays 0 (claims 8 and 10). Control 3
  does not chain: it drives the first curve to its target's centre. Control 4, in a qualifying run,
  chains as control 1 does.
- *Falsified if:* the ordinary vehicle passes the curve's target without stopping to turn; control
  1 stops at the curve's target, or its positions after the chain point do not lie on the next
  curve from its entry point; control 2 moves; control 3 follows the next curve past the chain
  point; or control 4, in a qualifying run, drives the first curve to its target's centre.

**D10: the crush slowdown (claims 7 and 10, case 1; S-0210 claim 9).**
- *Setup:* a test vehicle type of movement zone *crusher-all*, speed field 20, acceleration flag on,
  acceleration factor 1.0 and slowdown distance 0 (test values). A straight route of 30 clear cells
  north with no overlay on it, destination cell 30. Run A: one enemy vehicle that the test vehicle
  can crush stands in cell 10. Run B: two infantry units stand in cells 9 and 10. Control: no
  obstacles. The obstacles are given no weapon and no orders by test override, so that they stay in
  place; a run in which an obstacle moves before it is reached is repeated.
- *Observable:* the per-tick displacement, and the cell in which it changes.
- *Expected:* in the control the vehicle earns 51 progress units per tick all the way (about 3.2
  ticks per cell). In run A, the track from cell 8 into cell 9 is in crush mode (its second cell
  holds the enemy vehicle, claim 7), and from the first tick that begins after the vehicle has
  driven into cell 9 it earns trunc(51 × 0.2) = 10 progress units per tick at once, not after a
  gradual slowdown. It keeps that rate through the crush in cell 10 and up to cell 30 (about 16 ticks
  per cell), because it crushes no overlay (S-0210 claim 9). In run B there is no slowdown:
  infantry neither puts a track into crush mode nor sets the state.
- *Falsified if:* run A shows no slowdown, a slowdown that begins only in cell 10, a speed that
  falls over several ticks, or a recovery before cell 30; or run B shows a slowdown. The state's
  persistence across stops and new orders and its end at an overlay are measured by S-0210
  scenario K5.
- *Runs C to F (the cells that decide crush mode, claim 7):* the same test vehicle type, with the
  acceleration flag off, in a corridor one cell wide that turns by 1/8 turn at a cell *X* (as in
  D17 part g), so that the step into *X* and the next one would form a two-cell curve. The test
  rules file makes one overlay type crushable. Run C: that overlay lies in *X*. Run D: it lies in
  the cell after *X*. Run E: a wall overlay of the computer player lies in *X*. Run F: no overlay,
  and an enemy vehicle that the test vehicle can crush, given no weapon and no orders by test
  override, stands in *X* (a vehicle in the first cell does not count). Control: the corridor with
  no overlay and no vehicle. *Observable:* the positions and facings through the turn.
  *Expected:* the control and run F drive the two-cell curve through *X*; in runs C to E the
  vehicle drives the straight track into *X*, stops at its centre and turns in place before it
  goes on. *Falsified if:* a run of C to E curves through *X*, or the control or run F does not.
- *Runs:* 3 per run for runs A, B and F, since the obstacles' own behaviour is not part of this
  spec; 1 each for runs C to E and the controls.

**D11: stopping (claim 11).**
- *Setup:* part a: field 20, acceleration off. The vehicle is ordered north, and a stop order is
  given when it is about 1/3 of the way into a straight step. Part b: field 20, acceleration flag
  on, acceleration factor 0.02, deceleration factor 0 and slowdown distance 0, already facing map
  north; it is ordered 3 cells north, comes to rest there, and is then ordered 3 cells further
  north. The recorder also reads the speed fraction *ρ*.
- *Observable:* part a, the vehicle's position each tick from the stop order until it stops; part
  b, *ρ* on the fifth tick after the first move reaches its destination (the second order is
  given later), and *ρ* on the first tick of movement of the second move.
- *Runs:* 1 each (no random draw is involved).
- *Expected:* part a: it finishes that step at *s* = 15 and stops at the cell centre. Part b: *ρ* is
  0 at rest, and 0.02 (the single-precision 0.02 widened, S-0003 claim 6) on the first tick of the
  second move, which starts from 0 like the first.
- *Falsified if:* in part a it stops at once, or finishes at full speed; in part b *ρ* is not 0 at
  rest, or the second move's first fraction is the fraction the vehicle had on arrival plus the
  factor, or any other value than the factor.

**D11 c: attack during the pre-step turn (claim 11, Q-1160 question 2).**
- *Setup:* a tiny flat map, stock data, speed index 2, no director or mods. Use a
  human-owned turret-less armed driving vehicle, an enemy unarmed durable structure
  within weapon range, and one bystander computer player kept away from the observed
  lane. Place a free adjacent destination opposite the vehicle's initial body facing,
  with the target structure farther along that same direction. No approach destinations,
  tether, external movement hold or other orders are present.
- *Procedure:* scripted input orders the adjacent move and then ordinary attack on
  the structure on the first tick of the pre-step turn, before any track starts.
  Repeat five valid runs. Repeat with the structure in a different in-range direction
  to expose any later body re-aim. A control run receives no attack order.
- *Observable:* a read-only recorder captures the order boundary and each tick's end:
  world position, body facing and target facing, turn start and duration, both
  destinations, heading point, stored path steps, current and queued behaviour,
  and behaviour processing. Separate the order's effects from subsequent aiming
  and any newly assigned approach destination. Void a run only for outside
  interference or failure to issue attack before track start, not for a contrary result.
- *Expected:* at the attack boundary both destinations are absent, the heading point
  remains absent, and neither the body target nor the stored path is erased by the
  destination cancellation. Despite retained path steps there is no old step after
  the turn ends. Without a later facing instruction the existing turn completes
  without displacement. A later aiming instruction can change the body target and
  therefore the remaining turn; it is not a facing change made by cancellation.
  The no-attack control completes the turn and starts its step. S-0215 claim 11
  supplies readiness and same-tick processing expectations, independently of whether
  an aiming instruction has changed the turn.
- *Falsified if:* attack leaves either destination, installs a heading point, itself
  stops or retargets the body turn, or the vehicle takes the cancelled step without
  a new locomotor destination. Record any path erasure at the order boundary too:
  it contradicts the proposed retention observation, although absence of movement
  alone cannot distinguish retained from discarded inert steps.

**D12: country factor, veteran factor and speed power-up (claims 2 and 13).**
- *Setup:* as D1 (straight, field 20). Run 1: country speed factor for vehicles 0.5. Run 2: a map-placed
  veteran vehicle whose type grants the faster ability at that rank, with the veteran speed factor
  set to 1.5. Run 3: a speed power-up of value 1.5 is placed on the route and picked up before
  cell 10. Run 4: two such power-ups are picked up before cell 10. The power-ups come from crates on
  the route, with a test rules file that leaves the speed power-up as the only one a crate can give
  and sets its value to 1.5 (which power-up a crate gives belongs to S-0520 claims 8–10). In runs 1
  to 4 no other mobile object is near the crates (claim 13). Run 5: as run 3, with a second vehicle
  *W* of the test type standing idle in a cell next to the crate's cell and a stock aircraft that
  can land on open ground landed in another cell next to it; the test rules file also sets the
  size of the crate's area (S-0520 Terms and F4) large enough to cover the cells next to the crate.
  Place an idle infantry unit in a further covered neighbouring cell. The recorder also reads
  the speed bonus factors of *W*, the infantry unit and the aircraft.
- *Observable:* as D1, arrival tick at cell 20 minus arrival tick at cell 10; in run 5, the speed
  bonus factors after the pick-up, and the same arrival difference for *W* when it is then
  ordered along the route.
- *Expected:* run 1 66 ± 1 ticks (*s* = 25). Run 2 22 ± 1 ticks (*s* = 76). Run 3 22 ± 1. Run 4 the
  same as run 3. Run 5: *W*'s factor becomes 1.5 and it takes 22 ± 1 ticks; the aircraft's factor
  stays 1.0; the infantry unit's factor becomes 1.5.
- *Falsified if:* any run is off by more than one tick, run 4 is faster than run 3, *W*'s or the
  infantry unit's factor is not 1.5 in run 5, or the aircraft's factor changes. If no stock aircraft can land on open
  ground, run 5 is run without it and the exclusion of aircraft stays unconfirmed.

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

**D14: follower cars copy the fraction (claim 10, car chains).**
- *Setup:* a test map whose vehicle entries make one vehicle the follower car of a leader through
  the follow-index field (S-0009 claim 41). Both types have speed field 20 and the acceleration
  flag on. The leader's acceleration factor is 1.0. The car's type has acceleration factor 0.001
  and is not passive. The leader is ordered along a straight route north, and S-0212 (claim 26)
  then sends the car after it. Control: the same with the leader's acceleration flag off.
- *Observable:* the car's displacement per tick over the first 20 ticks of its movement.
- *Expected:* the car moves at about the leader's pace, about 51 progress units per tick, so it
  covers a cell in about 3 or 4 ticks. In the control the leader copies nothing, so the car
  accelerates by its own factor alone: at 0.001 per tick it earns nothing for its first 19 ticks
  and needs about 90 ticks for its first cell. Precondition: the car must follow the leader as
  S-0212 claim 26 gives (its scenario P9 tests this); if it does not move, the scenario is
  inconclusive for this claim.
- *Falsified if:* in the main run the car accelerates by its own factor alone, as in the expected
  control; or the control's car moves at the leader's pace.

**D15: the flag carrier's halving (claim 2).**
- *Setup:* as D1 (straight, field 20), in a skirmish game with a test rules file that turns on the
  game-setup default for capture the flag. The test vehicle picks up an enemy flag before cell 10.
- *Observable:* as D1, arrival tick at cell 20 minus arrival tick at cell 10, while it carries the
  flag.
- *Expected:* 66 ± 1 ticks (*s* = trunc(51 / 2) = 25), against 32 ± 1 without the flag.
  Precondition: a flag must exist and be carried; if none can be picked up, the scenario is
  inconclusive (Open questions).
- *Falsified if:* in a qualifying run a vehicle carrying a flag crosses cells 10 to 20
  outside 66 ± 1 ticks, including the unhalved result of 32 ± 1 ticks.

**D16: the locomotor field (claim 14).**
- *Setup:* a test rules file adds new types to the type lists, each a copy of the fields of an
  ordinary stock type of its kind except for the locomotor field: vehicle type *A* without the
  field; vehicle type *B* with the driving identifier of T3 written in lower-case letters; vehicle
  type *C* with the value "drive"; vehicle type *B*′ with the driving identifier of T3 with blanks
  before and after it; infantry type *I* and aircraft type *P* without the field; infantry type
  *J* with the driving identifier. A test map override section gives an ordinary driving stock vehicle type the value "drive" (type *S*).
  A second test file adds vehicle type *U* whose value is a well-formed class identifier not in T3
  (for example every digit 0 except the last, 1).
- *Procedure:* place one unit of each of *A*, *B*, *B*′, *C*, *I*, *J*, *P* and *S* by the test map
  on open flat ground and order each to a cell 10 cells away. In a separate run, place a unit of type *U* by the map.
- *Observable:* whether each unit's position moves along the ground from tick to tick or jumps to
  the destination in one movement update and then phases in (teleporting, claim 20; D22 checks the
  jump itself); for *U*, whether the game runs.
- *Runs:* 1 each (no random draw decides the outcome).
- *Expected:* *A*, *C*, *I* and *P* teleport; *B* and *B*′ drive; *S* drives (the ignored value keeps
  the stock layer's locomotor); *J* moves along the ground and does not teleport (how fast and along
  which points is outside this spec, Open questions); the run with *U*
  ends in a program error when the unit is placed.
- *Falsified if:* *A*, *I* or *P* moves along the ground, *B*, *B*′ or *S* teleports, *C* drives, *J*
  teleports, or the run with *U* plays normally.
- *Malformed-value controls:* repeat the override of *S* with an empty value, with the driving
  identifier's braces omitted, and with one hexadecimal digit removed. Each must keep the
  earlier driving choice; a changed choice falsifies claim 14. Run each once.
- *Identifier controls:* for every row of T3, add a test type of a suitable mobile kind with
  that identifier. Record the selected locomotor, expressed using T3's behavioural names,
  at object creation with the read-only recorder, without prescribing how that locomotor
  moves. Run once per row. A selected locomotor other than the row's name, or a creation
  error for a listed identifier, falsifies the table. Also repeat *B* with its locomotion
  class changed to track and its movement zone to amphibious: it must still select driving.
  A different selection falsifies the independence of these fields.

**D17: a refused step (claims 15 and 16).** Common setup: a corridor one cell wide of clear ground,
with no bypass, along map north, and a cell *X* in it. The mover *M* is a vehicle of the human
player of an ordinary driving stock type that is not a crusher, ordered by scripted input to a
corridor cell 10 cells past *X*. A recorder reads every tick: the positions, facings, behaviours,
targets and destinations of the units, *M*'s stored path, and the scenario random stream's state
(random-state recorder). *M*'s *arrival* is the first tick on which *M* stands at the centre of the
cell before *X*.
- (a) An idle unarmed vehicle *V* of the human player in *guard*, of a driving stock type, stands in
  *X*. *Expected:* on *M*'s arrival tick (its facing already matches the corridor) *M*'s stored path
  is replaced and *V* gets a destination in a corridor cell, chosen with no draw from the scenario
  random stream; *V*'s behaviour stays *guard*; *M* gets no track on that tick. *Falsified if:* *V*
  gets no destination while *M* waits in front of *X* (as a vehicle that sends no request would),
  *V*'s behaviour becomes *move*, or a draw is recorded for *V*'s acceptance.
- (b) As (a), with *M* ordered to *X* itself, so that on its arrival *M* is within the close-enough
  distance (read from the stock rules; the scenario needs it to be more than one cell). *Expected:*
  on *M*'s arrival tick *M*'s destination is removed and it stops; *V* gets no destination.
  *Falsified if:* *V* gets a destination, or *M* keeps a destination.
- (c) As (a), with *V* placed by the map in *sticky*. *Expected:* *V* never gets a destination
  (claim 16, case a); *M* stays in front of *X*, its stored path replaced on every tick on which it
  faces *X*. *Falsified if:* *V* moves.
- (d) As (a), with *M* armed (a weapon that reaches the adjacent cell and has no minimum range, so
  that *attack* gives it no approach destination) and *V* replaced by an unarmed vehicle *E* of the computer player that
  *M* cannot crush. *Expected:* on *M*'s arrival tick *M* takes *E* as its target and its behaviour
  is overridden to *attack*, and from that tick *M* has no destination (the recorder reads none on
  the arrival tick and on every tick while the override lasts), and the override is not repeated
  on the following ticks. A second step: map triggers destroy *E* a fixed number of ticks after the
  arrival (or *M* destroys it). On the tick of the killing hit *M*'s destination is its corridor
  cell again, and *M* then drives through *X* to it. *Falsified if:* *M* waits without a target,
  *M* keeps its corridor cell as its destination while it attacks *E*, or after *E* is destroyed
  *M* has no destination and stays in front of *X*.
- (d2) As (d), with *M* of a stock type whose primary weapon can damage walls, and *X* holding a
  wall segment of the human player, placed by the map, and nothing else (a *friendly obstacle*,
  S-0209 row VO5). *Expected:* on *M*'s arrival tick *M* takes the wall's cell as its target and
  its behaviour is overridden to *attack*, although the wall belongs to its own house. *Falsified
  if:* *M* waits without a target. Precondition: *M* must arrive in front of *X*, which needs the pathfinder
  (S-0212) to give it a path through the wall's cell; if it never arrives, the part is
  inconclusive.
- (d3) As (d), with *M* unarmed, so that *E* gives *blocked* (S-0209 row VP26). Move *E* into
  *X* by a map trigger after *M* has obtained its path but before arrival: the pathfinder
  otherwise refuses a path through *X*. Record path-search ticks and the retry timer. The
  discriminating run qualifies only if *M* still has the stored step into *X* at arrival and
  the retry timer is running; otherwise the timing comparison is inconclusive.
  *Expected:* *M* discards its stored path and searches at once on its arrival tick,
  regardless of the running retry timer (S-0212 claim 22).
  A failed search follows S-0212 claim 23; a found path is checked again on that tick, with
  its actual new first cell deciding the result. It need not give *blocked* again.
  *Falsified if:* a qualifying run waits for the retry delay before searching, or a found
  path gets no repeated check on that tick although *M* faces its new first step.
- (e) *X* holds a closed gate of the human player. The corridor turns by 1/4 turn at the cell before
  *X*, so that *M* arrives there facing another way. *Expected:* the gate starts to open on *M*'s
  arrival tick, before *M*'s facing starts to change. *Falsified if:* *M* turns before the gate
  starts to open.
- (f) As (d), with a test file that makes *E*'s type cloak when idle (the visibility specs own
  cloaking) and *M* unarmed. Include a third house with ordinary sight of *X* but no sensor
  coverage. Record *E*'s cloak state and its presentation for both non-owning houses.
  *Expected:* on *M*'s arrival tick *E* starts uncloaking for every house (S-0516 claim 11 e),
  and *M*'s stored path is cleared; in runs in which *M*'s path retry timer has run out on
  that tick, *M* gives up its move on the same tick (destination removed). *Falsified if:* *E* stays
  fully cloaked, only the mover's house sees the uncloak, or *M* keeps its path.
- (g) A corridor that turns by 1/8 turn at *X*, so that the step into *X* and the next one form a
  two-cell curve; *X* itself is clear, and the cell after *X* is the curve's second cell (table
  T5). The recorder also reads *M*'s speed fraction.
  - Run 1: *V* of (a) stands in the cell after *X*. *Expected:* on *M*'s arrival tick *V* gets a
    destination, and *M* stays at the centre of the cell before *X* with no stored path and *ρ* = 0,
    then drives on after a later search.
  - Run 2: *V* is replaced by a vehicle that map triggers move back and forth through the cell
    after *X*. *Expected:* when *M* arrives while that vehicle is in it, *M* drives straight into
    *X* and turns there.
  - Run 3: *M* armed, and an unarmed vehicle *E* of the computer player that *M* cannot crush
    stands in the cell after *X* (an *enemy obstacle*). Also record the path retry timer.
    *Expected:* T5 discards the old path. If the timer is still running on the arrival tick,
    the tick ends with no stored path and no track. If it has expired, a fresh search occurs
    on that tick; a found path whose first cell gives *clear* and matches the facing begins
    a straight or diagonal track, not a curve. A failed search follows S-0212 claim 23.
    The discarded path need not remain empty at the end of a tick that searched again.
  - Run 4: as run 3, with *E*'s type cloaking when idle (as in (f)). *Expected:* on *M*'s arrival
    tick *E* starts uncloaking for every house, as in (f), and *M*'s stored path is cleared; in runs in which
    *M*'s path retry timer has run out on that tick, *M* gives up its move on the same tick.
  - Run 5: the corridor turns by 1/4 turn at *X* instead, and the cell after *X* holds a closed gate
    of the human player through which the corridor passes. *Expected:* on *M*'s arrival tick the
    gate starts to open, and *M* stays at the centre of the cell before *X* with no stored path and
    *ρ* = 0.
  - Run 6: as run 3 with *M* unarmed, so that *E* gives *blocked*. Move *E* into the second
    cell by a map trigger after *M* has obtained its path but before its arrival. The run
    qualifies only if the stored path still names the two-cell curve at arrival. Record the
    tick of every path search. *Expected:* *M* discards its path and searches on its arrival
    tick, even with a running retry timer. If the search fails, S-0212 claim 23 decides the
    result. If it finds a path, the repeated check applies to that new path, not necessarily
    to the old blocked cell; T4 and T5 predict the result from its logged first two steps.
  - Run 7: as run 3, with a fork at *X*: besides the cell after *X* (the 1/8-turn branch, which
    holds *E*), the corridor also leaves *X* by a 1/8 turn to the other side, through a branch
    one cell longer than the first, which leads to the same destination. *E* is moved into the cell after
    *X* by a map trigger a few ticks after *M*'s order, so that *M*'s first path runs through the
    shorter branch, and stands still there from at least 10 ticks before *M*'s arrival. *M* made
    its last search when it was ordered and found a path then, so its path retry timer has run
    out on its arrival tick (S-0212 claim 21). *Expected:* on *M*'s arrival tick *M* has a new
    stored path through the longer branch and begins a track on that tick: the straight track
    into *X*, so that its facing stays the corridor's until it reaches *X*'s centre, where it then
    turns towards the longer branch. *Falsified if:* *M* begins no track on its arrival tick (no
    step check after the search), or it begins the curve into the longer branch on that tick.
    Precondition: the new path runs through the longer branch (S-0212 decides whether the cell
    holding *E* costs more than the extra cell); if it does not, the run is inconclusive.
  - *Falsified if:* *M* curves into an occupied cell (runs 1 and 2); *M* keeps its old stored
    path or begins a track on its arrival tick (runs 1, 4 and 5); in run 3 the old path is
    kept, a search occurs while the retry timer is running, or the logged new clear first
    step begins a curve instead of a straight or diagonal track; in run 4 *E* stays fully cloaked or the uncloak is limited to the mover's house,
    in run 5 the gate does not start to open on that tick; in runs 1 and 5 *ρ* is not 0 at the
    end of that tick; or in a qualifying run 6 the fresh search waits for a later tick, or
    the repeated check disagrees with T4/T5 for the logged new path.
- (h) The cell layer of the request. A purpose-made map with a high bridge over water, with land
  at deck level at both ends (as in D5 run 4), which *M*, starting on the land at one end, must
  cross along its deck. Every deck cell of one cross-section of the bridge holds an idle
  unarmed vehicle of the human player in *guard*, of a driving stock type, and under each of those
  cells an idle ship of the human player in *guard* stands on the water. *X* is the deck cell of
  that cross-section that *M*'s path enters. *Expected:* on *M*'s arrival tick, while *M* is on the
  deck (4 height levels above the water's level, so 3 or more), the vehicle on *X*'s deck gets a
  destination and the ship under *X* does not. *Falsified if:* the ship under *X* gets a destination
  on that tick, or the vehicle on *X*'s deck does not.
- (i) The nearest object (claim 15, F8, `quirks.drive_nearest_from_top_corner`). *M* is a vehicle
  of the human player of a driving stock type that is a crusher, standing at rest in the cell
  before *X* and facing 1/4 turn away from *X*; it is then ordered to the corridor cell 10 cells
  past *X*, so its first search is a fresh search with no stored path, and the look at *X* comes
  before it turns. *M*'s type needs at least 2 ticks for a 1/4 turn (S-0206 claim 12), so that
  the ticks before *M*'s facing first changes can be told from the tick on which it faces *X*.
  The *order tick* below is the tick on which *M*'s fresh search is made (S-0213 owns when an
  order takes effect); the observables are read against *M*'s facing, not against that tick. *X*
  holds an idle unarmed vehicle *V* of the human player in *guard* at the
  centre, and an idle infantry unit *I* of the computer player, of an unarmed stock type that *M* can crush,
  placed by the map at a chosen standing position. *V* gives *stationary friendly* (S-0209 row
  VP20) and *I* no value of its own (row VP25), so *X* gives *stationary friendly*; that is the
  precondition, and the recorder confirms it by *M* sending a request at all.
  - Run 1: *I* at the top position (*d*ₙ 90 against *V*'s 181). *Expected:* *I* is the nearest
    object and is not allied, so the look does nothing: *V* gets no destination until the tick on
    which *M* first faces *X* after turning (then T4 sends the request). *Falsified if:* *V* gets a
    destination before *M*'s facing first changes; that would show the distance
    measured from the centre (the switch off) or *I* left out.
  - Run 2: *I* at the bottom position (271 against 181). *Expected:* *V* is the nearest object,
    allied, and the give-up test fails (the destination is far), so the look sends the request on
    the order tick: *V* gets a destination before *M*'s facing first changes. *Falsified if:* *V*
    gets its destination only once *M* faces *X*.
  - Run 3: no *V*; *X* holds an idle infantry unit *A* of the human player in *guard* and *I*, at
    the right and left positions (202 each, a tie). In run 3a *I* is placed by the map and *A* is
    then walked into *X* by scripted input, so *A* entered last and is listed first (S-0209 claim
    13); in run 3b *A* is placed by the map and *I* is walked in by a map trigger. The run counts
    only when the recorder shows the two at the right and left positions. *Expected:* in 3a the
    request is sent on the order tick (*A* gets a destination or takes its draw, S-0208 claim 13,
    before *M*'s facing first changes); in 3b nothing happens until *M* faces *X*. *Falsified if:*
    both give the same timing (the tie not decided by listing order), or in 3b the request is sent
    before *M*'s facing first changes.
- *Switch note:* no situation of the original is known that gives a driving vehicle a contact while
  it stands in front of a stationary friend within the close-enough distance, so
  `quirks.drive_giveup_ignores_contact` has no discriminating run; part (b) tests the default
  without contact.
- *Runs:* 3 per part and run, with the order given at varied ticks. Only part i uses infantry, so
  no other part expects a scatter draw; in part i the infantry units that accept a request take
  their draws as S-0208 claim 13 gives.

**D18: vehicle scatter (claim 16).**
- *Setup:* open flat ground; an idle unarmed vehicle *V* of the human player of a driving stock type,
  in *guard*, with no other unit within 2 cells. Recorder as in D17.
- *Procedure and expected:*
  - (a) The scatter order (S-0213 claim 9) by scripted input on tick *N*. *Expected:* *V*'s
    destination is the neighbour at position (*N* mod 8) of the nearby-cell search's collection
    order (S-0212 claim 25); no draw from the scenario random stream; *V*'s behaviour stays *guard*.
  - (b) *V* is first ordered to a cell 20 cells away and given the scatter order while it drives.
    *Expected:* its destination does not change.
  - (c) Vehicles placed by the map in *sleep*, *sticky* and *unload*, and an idle vehicle
    whose test-map override sets its train flag, each given the scatter order.
    *Expected:* none gets a destination or moves. A run qualifies only while the recorded
    current or queued behaviour still names the tested behaviour when the request arrives.
    A behaviour that has already changed makes the run inconclusive.
  - (d) An idle vehicle of a stock type whose locomotor is the teleporting one, given the scatter
    order while it is at rest after S-0215 claim 13's restoration, with teleporting as its active
    movement kind (the recorder shows it). *Expected:* it does not move.
  - (e) A vehicle *W* of the human player stands on a cell of a bunker of its house when another
    vehicle of the house enters the bunker, so that the bunker rule of S-0215 claim 17 sends *W* a
    forced request with the bunker's position as threat point. *Expected:* exactly one draw from the
    scenario random stream at that tick for *W*; *W* queues *move* and takes one of the eight
    neighbours of its vehicle scatter centre cell, in the preference order of claim 16.
  - (f) An armed vehicle *T* of a stock type without a turret, idle in *guard*, and an enemy unit
    that map triggers move into *T*'s weapon range behind it, so that *T* turns its body by about
    1/2 turn to aim at it (S-0310), a turn that lasts several ticks. The scatter order is given by
    scripted input while the recorder shows *T*'s body turning. Control: the same order given once
    *T*'s facing has been steady for 10 ticks. *Expected:* *T* gets no destination from the order
    while it turns (case d); in the control it gets one.
  - (g) A vehicle of a stock type that deploys where it stands, deployed by scripted input, then
    given the scatter order. *Expected:* it gets no destination (case f). If no stock vehicle type
    deploys where it stands, this part is skipped and case f stays unconfirmed.
  - (h) As (e), with *W* in *guard* and a named test file that gives *guard* the paralysed
    property (S-0215 claim 17). *Expected:* *W* stays where it is, and no draw is recorded for it (case h).
  - (i) Vehicle scatter centre cell. *V* is ordered by scripted input to a cell *X* 12 cells away in
    a straight line along open flat ground. While the recorder shows *V* on a straight track into
    cell *K* (the cell one step ahead of its own cell), scripted input gives the *stop* order and,
    on the first tick on which *V* has no destination, the scatter order (tick *N*). Repeat with
    the scatter order given on the first tick after *V* has reached its track target and stands
    still. *Expected:* while *V* is still on its track, its new destination is the qualifying cell
    at position (*N* mod *n*) of the search's collection order around *K*, with *n* counted from
    the occupancy marks the recorder shows around *K* (*V*'s own cell may not qualify); once it
    stands still, the same around its own cell; in neither run a cell chosen around *X*. If *V*
    has no destination only once it stands still, the first run is inconclusive and only the
    second applies.
  - (j) Factory clearing. Use S-0417 U5's stock ground vehicle factory and blocked exit apron
    cell E, game speed index 2, no engine add-ons or unrelated mods, with a bystander computer player kept
    outside the observed factory and neighbouring cells. Place an idle drive vehicle in E,
    then in a separate run in an occupied neighbour of E while a persistent scatter-refusing
    unit blocks E. Record requests, destinations, current and queued behaviours and scenario
    stream state. Expect no scatter draw or queued behaviour in E; in the neighbour expect
    one draw over 0 to 2, no draw over 1 to 4, and *move* queued, with destinations given by
    claim 16. For case j, use a turreted vehicle in that neighbour, with a destination and a
    target making its turret turn but not its body. Time production completion by scripted
    input so a clearing request arrives in that state. A simpler alternative uses the same
    factory idle, without production, as S-0417 claim 13 and U11 specify: wait past its first
    guard processing and time the recipient state to a later apron-clearing processing.
    Expect refusal, no destination change
    and no scatter draw; a control with the turret steady accepts when no other refusal holds.
    Separately time a request while a drive vehicle with a destination crosses E or its
    neighbour on a track. When accepted, expect the new destination to be selected around
    its heading point's cell, not around the cell listing it. Runs without the recorded
    required recipient state are inconclusive. Void runs with unrelated activity in the
    observed area. Falsifiers: wrong draw count, behaviour change in E, acceptance in the
    qualifying turret-turning case, refusal in its eligible control, or a destination differing
    from claim 16's search or candidate selection around the recorded scatter centre.
- *Runs:* 3 each; (a) at three different *N*; (j) three qualifying requests per recipient state.
- *Falsified if:* in (a) a draw is recorded, the behaviour becomes *move*, or another cell is chosen,
  in (b) the destination changes, in (c) or (d) a vehicle moves or gets a destination, in (e) no draw, two draws, or no
  *move*, in (f) *T* gets a destination while it turns or none in the control, in (g) the deployed
  vehicle gets a destination, in (h) *W* moves or a draw is recorded for it, in (i) the new destination
  is a neighbour of *X*'s cell, or does not match the pick around the expected centre.

**D19: engine defaults of the acceleration fields (claim 17).**
- *Setup:* as D3 (speed field 20, acceleration flag on, a route of 30 cells north from rest), with
  the test vehicle type's acceleration factor, deceleration factor and slowdown distance keys left
  out of every rules layer (run A). Control runs: B gives the three keys explicitly as 0.03, 0.002
  and 500; C as A but with the slowdown distance key set to 0; D as A but with the acceleration
  factor key set to 0.02 (the D3 value).
- *Observable:* ticks from the first tick of movement to the arrival at cells 1, 10 and 30 (the
  destination). In runs A and B the recorder also reads the speed fraction *ρ* each tick, printed
  with 17 significant digits or more.
- *Expected:* A and B give the same three tick counts. C reaches cell 30 sooner than A (no braking
  in the last two cells, claim 10 case 2), with the same counts at cells 1 and 10. D reaches cells
  1 and 10 later than A and matches D3 at those cells. The fractions show the precision of claim
  17:
  - *ρ* after the first tick of movement is *a* itself (case 4 from *ρ* = 0, far beyond *D*ₛ). In A
    it is 0.0299999999999999988897769753748434595763683319091796875, the double nearest 0.03. In
    B it is 0.02999999932944774627685546875, the single nearest 0.03 widened (S-0003 claim 6).
  - *ρ* is exactly 1 before braking begins (case 4 stops at *ρ*ₜ = 1). The first value below 1 is
    1 − *V* × *η* with *V* = 51, both steps rounded toward zero (Formulas), or that step taken twice
    when braking begins on a tick with two updates (claim 12). In A that is
    0.8979999999999999094058011905872263014316558837890625 (one update) or
    0.795999999999999818811602381174452602863311767578125 (two). In B, with the single nearest
    0.002, it is 0.89799999515525996685028076171875 or 0.7959999903105199337005615234375.
- *Runs:* 1 each (deterministic).
- *Falsified if:* A and B differ at any cell (another default value), C equals A at cell 30
  (another slowdown default, or none), or a fraction of run A or of run B differs from the values
  given for that run (in particular, A showing B's values).

**D20: speed estimate in a shot's lead (claim 18; Q-0071).**
- *Setup:* a tiny flat skirmish map with a human-controlled driving target and an enemy
  firer using a stock weapon whose launch exposes lead under S-0306 P2. Keep a bystander
  computer player outside the observed lane. Use stock rules with no engine add-ons or mods,
  game speed index 2. A read-only recorder logs launch position, initial projectile velocity,
  target reference point, body facing, speed fraction and positions each tick, including the
  target state used at aim time rather than a later movement update in that tick. The observed
  area is the target lane and firer's range. Void a run if unrelated units act there.
- *Procedure:* scripted input sends the target across the line of fire on a long straight
  run, then along a route needing a turn in place, and in a separate run through a friendly
  closed gate. Repeat with health above and below the yellow ratio, and with a veteran
  stock type that grants the faster ability. Also fire during a turn before a first move
  from rest. Include two distinct recorded nonzero speed fractions in the straight-run
  launches, as in S-0306 P2 runs h and i, so their predicted lead displacements differ.
  Capture three qualifying launches per case. For a gate wait or between-track
  turn, a qualifying launch has unchanged position and a recorded nonzero fraction while
  the locomotor still qualifies as moving. If none occurs in ten attempts, that case is
  inconclusive, not evidence for zero speed.
- *Factor controls:* repeat the straight-run case with named test-map overrides, removed after
  the run: country vehicle-speed factor 0.5 in one run, and in another the speed power-up as
  the only crate result, value 1.5, collected before the observed launches (as in D12). These
  are invented test values, not stock values. Log the bonus factor as well as the fraction.
  Capture three qualifying launches in each control. F2a must still predict their aim points.
  An estimate that omits either changed factor falsifies claim 18. These data-only controls
  otherwise use the common setup, with no engine add-ons or unrelated mods.
- *Observable:* reconstruct the aim point from the launch under S-0306 P2, then compare
  its lead displacement with S-0306 F6 using F2a at the recorded fraction. Use the imported
  stock factors without copying them into this scenario. Record enough launches away from
  rounding boundaries that the estimate, actual previous-tick displacement and a
  progress-to-lepton-scaled estimate predict distinct aim points.
- *Expected:* F2a predicts the aim point in each qualifying launch, including a nonzero
  lead during a turn or wait with retained nonzero fraction. A turn from rest with fraction
  zero gives no lead displacement. Veterancy and damage affect the estimate through F2
  and the held fraction respectively, not through a separate lead factor.
- *Falsified if:* a qualifying launch instead follows previous-tick displacement, a
  converted lepton speed, an estimate omitting country or veterancy factors, or zero solely
  because the target is turning or waiting despite a nonzero held fraction.
- The scenario records launches and fractions without changing the oracle.

**D21: an approaching vehicle once its target is in range (claim 19; Q-0072).**
- *Setup:* a tiny flat skirmish map, stock rules apart from the named test file below, no engine
  add-ons or unrelated mods, any game speed. Two
  straight lanes of clear ground 25 cells long and 2 cells apart (for run 2, two lanes of open
  water). A named test file, removed after the run, gives the attacker's type a weapon with an
  invented range field of 5 cells and damage 1, a turret, and gives the target's type an invented
  strength large enough to survive the run. All units belong to the human player, so scripted
  input controls every one. The attacker *A* stands at one end of the first lane; the target *U*,
  of an unarmed stock type with the same locomotor, at the far end of the second lane. A
  read-only recorder logs each tick *A*'s position, destination, number of stored steps and
  target, current and queued behaviour, approach destinations, the tick of each path search
  *A* makes, and *U*'s position. Log track completion for driving and sailing and step
  transitions with and without a chosen next step for hovering; also record the travel
  distance computed before each transition and the movement heading. Compute both the approach
  range test and claim 19 a's arrival range test from the recorded positions.
- *Procedure:* scripted input orders *A* to attack *U* (forced attack on an own unit). Once *A*
  has a destination, scripted input orders *U* to the cell of the second lane next to *A*'s
  starting cell, so that *U* comes in range while *A* still has several cells to go.
  - Run 1: *A* and *U* of stock types with the driving locomotor, *A*'s type without the
    deploy-to-fire flag.
  - Run 2: *A* and *U* of stock types with the ship locomotor.
  - Run 3: *A* and *U* of stock types with the hovering locomotor, with the test file also
    setting the path-retry delay field to 0.1 game minutes (invented), so that *R* = 90 ticks.
  - Run 3c (control): as run 3, with *U* never ordered to move and placed so that it is out of
    range until *A* reaches its destination.
  - Run 4 (hover wait control): as run 3 but targeting a stationary building, with a named
    test-map trigger giving *A* a distant cell destination while retaining that target and
    *attack*. A qualifying run has the target in range, stored steps, a running retry timer
    and a destination several cells away. If no such run occurs in ten attempts, this control
    is inconclusive. The building does not satisfy the mobile-target stop condition.
  - Controls for runs 1 and 2: use a stationary building target and a retained distant cell
    destination as in run 4; separately, repeat with a mobile target under *attack move*.
    Qualify only while the recorded target, behaviour and destination remain as specified.
  - Boundary controls for runs 1 and 2: put an infantry target near its cell's edge. Arrange
    an approach where the target reference point is in range at a track target but its cell
    centre is not, then reverse those two results. Also repeat using a named test override
    making the attacker open-topped and choose distances on either side of its strict
    three-dimensional range threshold. Log the relevant range and reference points.
- *Expected:*
  - Runs 1 and 2: with no approach destinations, *A* loses its destination and remaining
    steps at the first track target where the arrival range test passes. It keeps *U* as
    target and begins no further track, even if its old destination is several cells away.
    No stop occurs between track targets. Building and *attack move* controls do not get
    this early stop. Boundary controls follow the cell-centre result, not the target-point
    result; the open-topped controls instead follow the strict three-dimensional test.
  - Run 3: at a step transition while *U* is in range under the approach test, the stored
    path is discarded. If *R* still runs, no next step is chosen; with claim 19 a's conditions
    satisfied, *A* clears its destination and keeps its target. A positive travel distance
    already computed may produce a final step on that tick under S-0503 claim 7; it is not
    required to stop at its pre-transition position. No later travel occurs until a new move.
    If *R* has expired and a successful search chooses a next step, *A* continues on that
    transition instead. Run 3c has no range-driven path discard before reaching its destination.
  - Run 4: because the target is not mobile, the arrival-time stop does not apply. After
    discarding its path, *A* waits with its destination until 90 ticks after the previous
    search, searches and resumes. These waits repeat while the target stays in range.
- *Falsified if:* a qualifying driving or sailing run continues beyond a track target that
  satisfies the stop conditions, stops between track targets because of range, loses its
  target at this stop, or applies the stop to a building or *attack move* control. The
  boundary controls falsify the claim if they follow the wrong measured point or comparison.
  A qualifying hover run falsifies part d if its transition with no next step retains the
  destination despite satisfying the stop conditions, or clears it in run 4 because of range
  alone, or suppresses an otherwise permitted positive final step solely because the
  destination was cleared. Later travel without a new move also falsifies part d.
  A run 4 search before the retry delay expires falsifies the waiting rule.
- *Runs:* 3 each, including controls. The chase exception of claim 19 c is `unmeasured`: in our rules no order gives
  a hovering vehicle a vehicle or infantry unit as destination together with a target in range
  except the target-object destinations of S-0215 claim 37 h and S-0307, whose setups are those
  specs'.

**D22: the teleporting locomotor's own move (claim 20; Q-0181).**
- *Setup:* open flat ground with a cliff ridge and a second island separated by water, as each run
  needs. Units of the human player: the stock chrono miner *H* (a vehicle type with the harvester
  flag and the teleporting locomotor), and a test vehicle type *T* that copies an armed stock
  vehicle type without its locomotor field (as D16's *A*) and has the harvester flag off. Like
  that type, *T* has the teleport-capable flag off, so claim 28 does not apply to it; *H* has
  the flag, so a player's order makes *H* drive (claim 28). An ordinary driving harvester *G*
  as a control. Recorder as in D17, recording each tick the
  position (leptons and height), the facing, the cell listings and occupancy marks, the
  destination, the behaviour, whether the unit is phasing in, its targeting timer and the draws
  from the scenario random stream.
- *Procedure and expected:*
  - Run 1 (timing and effects). *T* stands still; on tick *t* scripted input orders it to a cell 15
    cells away. *Expected:* no turn and no movement before the jump; the jump on the tick on which the
    recorder first shows the destination; on that tick its position becomes the destination cell's
    centre at height *H*, its old cell does not list it and the new one does, its facing is unchanged,
    its destination is gone and its behaviour is still *move* at the end of that tick; it falls
    back to rest at a later tick (claim 20 e); it is phasing in from that tick
    until the tick F9 gives with the rules in force. Record animation appearances and
    sound requests at both positions, with their tick and within-tick order. In a named
    test-map control give *T* distinct departure and arrival sound entries; compare a
    control with those type entries absent and distinct game-wide entries instead.
    Expect old-point animation creation before its departure sound, arrival sound at
    *L* before the landing-point animation, and type entries in place of the game-wide
    ones when supplied. Wrong position, order or sound selection falsifies these parts
    of claim 20 e; a silent or unidentifiable entry makes that sound control inconclusive.
  - Run 2 (harvester exception). *H* uses its docking jump, not a player's order. It is full,
    15 cells from its owner's free refinery and within the chrono-harvester too-far distance
    (S-0402 claim 9), which a test rules file sets to 50 cells, as S-0402 scenario H8 sets
    it, so its *enter* approach offers the docking cell. *Expected:* the jump to
    the docking cell in one movement update. The recorder shows no phasing-in state at any tick
    from the jump on, and its active locomotion is driving from the jump tick and stays driving
    through its docking and unloading (claim 28). Its behaviour stays *enter* until it docks.
  - Run 3 (occupied destination). The destination cell holds a stationary friendly vehicle.
    *Expected:* *T* lands on the cell that the nearby-cell search of claim 20 b gives (at position
    (*N* mod *n*) of the collection order, *N* the tick number), not on the occupied cell. Variant:
    every cell within the search's largest radius is occupied or unenterable. *Expected:* *T* does
    not move and its destination is removed.
  - Run 4 (no path needed). The destination is on top of a cliff with no ramp, and in a second run
    on another island. *Expected:* *T* jumps there in one movement update. *H* and *G*, given the
    same order, drive and do not arrive (claim 28 for *H*).
  - Run 5 (delay against distance). Orders to cells at several distances from 1 to 30 cells, with
    the stock rules and with test rules files that change, one at a time, the distance factor, the
    trigger flag (off), the minimum delay and the range minimum. *Expected:* the phasing-in
    end tick minus the jump tick is max(*D*, 1), with *D* from F9, for the non-harvester *T*.
    With the trigger off or distance below the range minimum, it is max(*m*, 1), where *m*
    is the imported minimum delay. Include named test variants with minimum delay 0 and
    −1 and the trigger off: both end on the tick after the jump, not on the jump tick.
    A different end tick falsifies this part.
  - Run 6 (orders while phasing in). Scripted input gives *T* a move order on the second tick of
    its phasing in. *Expected:* the order is lost: no jump during or after phasing in until a new
    order is given after it ends.
  - Run 7 (draws at the end). *T* without a target, ordered to jump at varied ticks so that its
    targeting timer immediately before phasing-in completion has more than 10 ticks left in
    some runs and 10 or fewer in others; classify using the preceding tick's recorder reading
    and the elapsed tick. Run both classes with the opportunity-search gates passing, then with
    a named test type whose passive-acquire flag is off to refuse that search. Keep enemies
    outside the search area to exclude candidate-dependent draws. *Expected:* above 10 ticks,
    one conditional scenario-stream draw over 4 to 8, then the normal S-0309 claim 4 timer draw
    if the gates pass; that normal draw replaces the temporary timer. At most 10 ticks, only
    the normal draw if the gates pass. With the gates refusing, only the conditional draw above
    10, and neither at most 10. Also record a qualifying retained-target run if reachable:
    neither completion draw nor search is expected; a run without a retained target is
    inconclusive for that control. Compare S-0309 A1 f.
  - Run 8 (shot in flight). An enemy unit fires a slow projectile at *T*; *T* is ordered to jump
    while the projectile is in flight. *Expected:* the attacker loses *T* as its target on the jump
    tick and the projectile does not hit *T*.
  - Run 9 (hit and crushed while phasing in). While *T* phases in, an enemy unit fires at it from
    in range, and in a second run an enemy crushing vehicle is ordered over *T*'s cell. *Expected:*
    *T*'s recorded health falls as for a standing vehicle of its type, and the crushing vehicle
    treats it as it treats such a vehicle (S-0210).
  - Run 10 (entry level and missing region, Q-0232). At game speed index 2, use a high
    bridge over flat water with banks at deck level, and *T* on ground. First order a deck
    cell occupied by a friendly drive vehicle while its ground layer is empty. Repeat with
    the deck empty and a ship below. Record the selected landing cell, layers, positions and
    marks. Expect refusal of the occupied deck, independent of *T*'s height, and acceptance
    of the empty deck despite the ship below. Separately give *T* a destination on water
    for which its own zone has no reach region, with at least one free enterable bank cell
    inside the nearby search radius. If the click rules substitute a bank destination first,
    use a named test-map move trigger to supply the water cell and qualify only when the
    recorded destination really is that cell. Expect the nearby-cell collection and
    tick-number pick with no region restriction, not automatic failure. Block all
    qualifying cells in a control: expect no jump and destination removal. Different
    entry-layer results, rejection solely for no region, or a pick outside the predicted
    collection falsifies this part.
  - Run 11 (aircraft, Q-0232). A named test type copies an ordinary aircraft but omits its
    locomotor field from every rules layer. Place it on flat ground, then order a clear
    distant cell and, separately, a high-bridge cell. Log listings, both occupancy layers,
    position and height. Expect an object-present reservation, not vehicle-present, at the
    landing point at terrain height; after the jump expect ground listing and height *H*
    on flat ground, deck listing and height *H* + 416 on the bridge. An aircraft may land
    in an occupied destination because its entry outcome is clear. A flight-height landing,
    vehicle-present reservation, or a different listing layer falsifies this part. Also
    record both object-present flags after deck placement: both remain set in an
    otherwise empty cell. Run 14 isolates their later clearing.
  - Run 12 (jump-tick animation draws, Q-0232). Repeat run 1, isolating other actors and
    recording animation creation ticks and the scenario stream. Compute the applicable
    S-0202 claim 10 draws from the imported warp-animation type. As a discriminating
    named test-map variant, select a non-bouncing test animation with no start-event
    effects and a random stage-delay range 2 to 3 ticks. Expect one ranged draw per
    creation, old point before landing point, both in the jump tick. The normal variant
    may take none if no creation condition needs a draw; that is not a special exemption.
    Missing either test draw, reversed assignment or delayed creation falsifies the claim.
  - Run 13 (vehicle bridge reservations, Q-0240). At game speed index 2, use a high bridge
    over traversable flat ground, a human house and one bystander computer player kept away,
    no engine add-ons or unrelated mods. Use *T*: a player's destination makes the stock chrono
    miner drive (claim 28), so it is not a subject here. Observe the bridge,
    its approaches and the departure cell; void runs with unrelated bystander interference or
    defeat. Start once below deck height and once on a deck, so the search's deck mode differs.
    Supply a clear bridge-cell destination by scripted input; record both occupancy records,
    listings, remembered landing point, destination and position with the read-only recorder.
    Qualify pre-jump observations only when the recorder catches a fixed landing point before
    placement; a same-tick jump does not justify inventing a one-tick waiting interval.
    *Expected before the jump:* the ground vehicle-present mark is set, not the deck mark,
    and the vehicle is still listed at departure. In a qualified interval, replace the
    destination with a different free cell and record the earlier ground reservation clearing.
    *Expected after the jump:* deck position and listing at height *H* + 416, with both
    ground and deck vehicle-present marks. Wait for phasing in to end where applicable, then
    give a free-cell destination outside the bridge. Expect the old ground mark to clear
    at fixation, and the old deck mark to clear when the vehicle leaves by its next jump.
    Isolate other occupants and reservations so they cannot account for either mark.
    A second pair of runs sends ordinary non-crushing vehicles under the bridge and along
    the deck towards the landing cell. Record entry outcomes and waits or re-routing,
    separating the pre-jump empty-list case from the post-jump deck occupant case.
    S-0209 claim 5 owns those outcomes; its ground record sees the pending reservation,
    while the empty deck record does not. Repeat each height and replacement case three
    times. A deck-only pre-jump reservation, clearing the remembered reservation from deck,
    or a ground mark disappearing solely on deck placement falsifies claim 20's layer rule.
    Without a captured pre-jump interval, that part is inconclusive, not a pass.
  - Run 14 (aircraft bridge cleanup, Q-0310). Repeat run 11 at game speed index 2
    on a high bridge over traversable flat ground, with a human house and one bystander
    computer player kept away, no engine add-ons or unrelated mods. The named test map supplies an
    aircraft type with teleporting locomotion; all other data are stock. Observe only
    the bridge, departure and landing cells; void unrelated bystander interference or
    its defeat. Use the read-only recorder for both occupancy records, listings, height,
    remembered landing point and tick. In an otherwise empty landing cell expect ground
    object-present at destination fixation and both ground and deck object-present after
    landing, with deck listing only. After phasing in, order a distant clear ground cell:
    expect the old ground flag cleared when the destination is accepted and the old deck
    flag cleared on departure. In a separate run destroy the aircraft while it remains on
    the deck, using a weapon without occupant-producing effects: at removal expect deck
    clear but ground still marked. Keep all other objects out of that cell throughout.
    Repeat each branch three times. A transferred ground flag, a deck flag retained after
    removal, or ground clearing solely because of destruction falsifies the claim.
    Probe the ground below with an ordinary non-crushing drive vehicle and, separately,
    an infantry unit. Record entry outcomes separately from standing-position choice.
    compare against S-0209's tables, not against a fictitious ground-listed aircraft.
  - Run 15 (removal with remembered and pending reservations, Q-0356). Use run 13's
    vehicle and run 14's aircraft on the same isolated bridge over traversable ground,
    game speed index 2, no engine add-ons or unrelated mods. In separate trials destroy each object
    after an uninterrupted deck landing, after a landing on clear ground off the
    bridge, and after a replacement destination has been accepted but before its jump.
    Add a first-jump trial removed after fixation but before any landing. Use weapons
    without occupant-producing effects, and discard trials where damage or another
    object changes the tested flags before removal. Record both layers' flags and
    listings, actual height, remembered landing point, destination and tick with the
    read-only recorder. Qualify a pending trial only when fixation and removal before
    the jump are both captured; ordinary input must not be assumed to supply that
    interval. Repeat each qualified branch three times. Expect claim 20 k's table:
    actual occupancy clears, a separate ground reservation remains. In the deck
    replacement branch old ground is already clear and new ground remains marked.
    In the off-bridge completed-landing branch ground clears because both marks
    coincide. Probe retained flags separately with a drive vehicle and a walker,
    recording entry outcome separately from standing-position choice as in run 14.
    Any separate pending reservation clearing solely on removal, actual deck flag
    surviving removal, or ground surviving off-bridge removal in the coincident-mark
    control falsifies the corresponding row. A missing pre-jump interval is
    inconclusive, not evidence that removal cleans reservations.
- *Runs:* 3 each, run 3's main case at three different tick numbers; run 7 until each class has at
  least 3 runs.
- *Falsified if:* a turn or a wait precedes the jump, the jump comes a tick later than stated, the
  facing changes, *T*'s behaviour is rest at the end of its jump tick in run 1, *T* lands on the
  occupied cell in run 3, *T* fails to reach the cliff top
  or island in run 4, *H* jumps in run 4, *T*'s phasing-in end tick differs from jump tick + max(*D*, 1) with
  *D* from F9, *H* phases in, the order of run 6 is
  carried out, the draw count of run 7 differs, the projectile of run 8 hits *T*, or in run 9 *T*
  cannot be damaged or is treated differently by the crusher.

  - Run 16 (landing-cell crate pickup). On a tiny flat skirmish map at speed index 2,
    with no engine add-ons or unrelated mods, crates disabled for random replacement, give the human house
    an existing base and place a fixed money crate by a map trigger on an empty cell.
    Order *T* to jump onto it (a player's order makes the stock chrono miner drive, claim 28). Record position, cell-arrival event,
    crate presence and credits each tick, with within-tick event order where available.
    Expect placement and the arrival event before pickup, crate removal and a money award
    on the jump tick under S-0520 claim 12. A crate remaining despite passing the pickup
    gates, pickup before landing, or pickup delayed to a later tick falsifies the rule.
    Use scripted input and a read-only recorder at 800×600; record the seed and run one trial
    at a time.
    A named test-map control with fixed unit contents and a successful unit placement tests
    that the pickup's ground-step refusal does not reverse the completed teleport landing.
    Record the created unit and collector positions; a return to the departure cell falsifies
    that distinction. Do not generalise this test to superweapon transport.
  - Run 17 (conditional diagnostic, claim 20 h). First establish whether any scripted order, or any
    order that sets an empty order-level destination, can take effect after a landing point is fixed
    and before its jump. If it can, use *T* as in run 1, with a stop order given on each tick of that
    window. Record position, facing, listings, occupancy marks, moving state and the remembered point
    each tick. *Expected:* no jump and no change of mark until a later nonempty destination is
    accepted. If the window cannot be reached, record that; the empty-assignment rule then stays
    `unmeasured` (claim 20 h). A jump in the window, or a mark that moves at the empty assignment, falsifies
    the rule.

**D23: temporary drive and return to teleporting (claim 21; Q-0233).**
- *Setup:* S-0417 U6's stock vehicle factory and produced chrono miner, a human house,
  one bystander computer player kept away, game speed index 2, no engine add-ons or unrelated mods.
  Record position, active movement kind, destination, pending landing point, phasing-in
  state and occupancy marks each tick. Observe the factory, exit track, apron and ordered
  destinations; void a run if an unrelated bystander object acts there or it is defeated.
- *Procedure:* produce the vehicle with a distant rally cell. In a second run give a move
  order by scripted input during its temporary drive, qualifying only if the order is
  accepted while driving. After the first recorded restoration and fall-back have finished,
  give a new distant clear-cell move order. In a third run, let the miner finish a harvest
  load and return to a free refinery after its restoration. Run each case three times.
- *Expected:* while driving, an accepted destination belongs to driving movement and does
  not fix a teleport landing point or move a teleport reservation. Restoration alone takes
  no occupancy action or jump. The recorder shows teleporting as the active movement kind
  after restoration. The new move order then installs driving again: no landing point, a
  driven path (claim 28). In the third run, the docking-cell offer fixes a landing point and
  the miner jumps to it, as claims 20 and 27 say. Occupancy during the exit follows S-0209's driving
  rules, not a mark attached to the rally cell. Record any destination that the factory
  supplied before temporary driving began; such a run is not evidence for an initially
  empty suspended state.
- *Falsified if:* a destination accepted while driving changes the suspended landing point,
  causes a jump before restoration, or moves a teleport mark; or restoration alone resets
  or moves the recorded teleport state. A recorded active movement kind other than
  teleporting after restoration falsifies the restoration hypothesis shared with S-0215
  B15 e. A jump on the post-restoration move order falsifies claim 28.
- A pending-jump, phasing-in or previous two-record deck-landing control needs
  a stock event that starts temporary driving in that state. No such event is established.
  If one is found, record both records of the old landing cell before installation,
  at installation, after driving has left it, and at restoration, together with both
  records and actual height of the current cell. Qualify the two-record control only
  if no earlier destination replacement has already cleared old ground. Expect a pending
  jump to be cancelled at installation, with no jump at restoration, no clearing just from
  installation, old deck clear and old ground retained after
  departure, and the current actual-layer mark unchanged solely by restoration.
  Old ground clearing on departure or restoration, or old deck being recreated on
  restoration, falsifies claim 21. Do not fabricate these controls through memory
  writes; these conditional rules remain unmeasured.

**D24: cell-tag arrival timing (claim 22; Q-0268 part 1).**
- *Setup:* a tiny named test map, human house and one bystander computer player kept
  away, game speed index 2, stock data apart from named test overrides, no engine add-ons or
  unrelated mods. Give selected cells
  independent one-time entered-by tags with an any-house event and an observable
  variable-setting action (S-0011). Use an ordinary drive vehicle and D22's teleporting
  test type *T*. A player's order makes a chrono miner drive (claim 28), so it is not the
  teleport subject here.
  Named type data may supply a non-harvester teleporter for the phasing-in control,
  and D9 control 1's passive vehicle with acceleration off for a chained curve.
  Observe the lane, bridge and landing cells; void unrelated interference or defeat.
- *Procedure:* drive through a tagged straight-track target without stopping. Repeat
  with tags on both cells of a two-cell curve. Teleport onto a tagged cell, with an
  intermediate tagged cell between departure and landing. Repeat below and on a high
  bridge. For a chained curve, tag the cell occupied at the chain-point transition
  separately from the unreached target. Record position, track completion, arrival
  reports, variable changes and
  phasing-in state by tick. Run each case three times.
- *Ordering controls:* combine the tagged arrival cell with S-0402 H14's tethered
  harvester/refinery arrival fixture, then separately with S-0210's cell-centre
  crushing fixture. Record the arrival report's resulting contact/destination state,
  the crushed object's removal, and the tag's variable change. Qualify an ordering
  observation only when read-only recordings distinguish these events within the arrival
  tick; end-of-tick coexistence alone establishes no within-tick order. Expect the report
  and crushing effects already completed when the variable changes. A variable change
  before either corresponding effect falsifies claim 22's ordering; a run without that
  temporal resolution is inconclusive for ordering. For tethered teleporting infantry,
  use S-0417 U10 d/e: record factory contact and tether at the landing tag's variable
  change. Expect both links ended and the contact place free before that change in the
  jump tick, not on reservation, phasing-in completion or falling back to rest.
  A surviving link when the tag action executes, or a release caused solely by one
  of those non-arrival events, falsifies this rule.
- *Expected:* a straight target raises on its completion tick despite continued travel,
  not on earlier boundary crossing or reservation. An ordinary two-cell curve raises
  at its final target only, not merely for crossing its first cell. A teleport raises
  at landing in the jump tick, before phasing in ends, never for intermediate cells.
  Under a high bridge no entered-by raise occurs. Deck arrival does raise it.
  A chained curve raises for the actual chain-point cell at its transition, not
  for the unreached target merely because it is the track target.
  A raise on reservation, mid-track boundary crossing, stopping only, an intermediate
  teleport cell, or phasing-in completion instead of landing falsifies the claim.
  A ground-under-bridge raise, a missing eligible deck raise, or a chained-curve raise
  assigned to the unreached target rather than the occupied chain-point cell also falsifies
  the corresponding rule. The ordering against jump-time crate collection, destination
  removal and landing animation is `unmeasured`: this scenario does not supply a
  discriminating fixture or sufficiently resolved recording for those three comparisons.

**D25. Driving wakes (claim 23).** Use a tiny flat map with water, a bank and a high bridge,
stock data apart from named test overrides, no engine add-ons or unrelated mods, at game
speed index 2. Record tick numbers, positions,
heading points, body turning and wake creation during three water crossings, turning at rest,
and travel below the bridge. Repeat on land and, where accessible, on the deck. A named test
map gives the same water-capable vehicle the driving locomotor, then repeats with only the
underwater-type flag changed; a further named variant removes the game-wide wake animation.
These variants are diagnostic map overrides, not stock measurements. Use the read-only recorder
and frame capture; compare scenario-stream state with the animation draws of S-0109 claim 3.
Expected: one wake on eligible tick numbers divisible by 10, including turning in place on water,
none on land, on the deck or without an animation; changing the underwater flag alone does not
suppress it. Any other cadence, exclusion or extra wake-specific draw falsifies claim 23.

**D25a. Driving wake rate and within-turn timing (claim 23; Q-0600).** *Setup:* D25's
water map and settings, with one human-controlled test vehicle and one bystander computer
player kept outside the observed crossing. Use named map test data, "driving-wake-speed",
giving the water-capable drive vehicle acceleration and a small positive acceleration factor
so that a straight start has at least 10 ticks with *ρ* > 0 but *s* = 0. Keep a defined wake
animation and initially align the body with the travel direction. These are diagnostic map
overrides, not stock measurements; remove them after testing. *Procedure:* order a long
straight crossing and record through final arrival. Repeat from each of the ten start-tick
residues modulo 10, then repeat with the original type fields as a stock control.
*Observable:* per-tick fraction, computed current *s*, progress budget, active track target,
body turning, pre/post-movement position, and wake creation position, using the read-only
recorder and frame capture. Include track starts, a final arrival with no replacement track,
and a between-track wait while the move destination remains. *Expected:* no wake solely
from positive *ρ* when *s* = 0 and the body is not turning. Positive *s* with a heading point
can qualify even if position does not change. A wake uses post-movement position and state.
A completed track with no new heading point and no body turn creates none, despite earlier
travel that tick. A retained move destination does not qualify during the between-track wait.
*Falsified if:* the fraction alone qualifies, lack of displacement alone suppresses an eligible
positive-rate wake, pre-movement position or a cleared track target is used, or a retained
destination substitutes for an absent heading point. A run without the intended zero-rate
interval, eligible no-displacement tick or boundary tick is inconclusive for that sub-test.

For draw ordering, add a named map variant with a crate collected during a continuing water
crossing and a wake animation with a distinguishable random creation delay (S-0109 claim 3).
Record a collection on an eligible wake tick that retains a heading point or body turn, together
with the scenario stream before and after the movement effects and wake creation. Compare
the crate outcome and animation delay with the raw stream values: movement's collection draws
must precede the animation's creation draw. Reverse assignment falsifies the order. Repeat
until three such qualifying collection ticks are recorded; no qualifying collection makes this
sub-test inconclusive. This proposes recorder observations, not an already completed measurement.
Void any run where an unnamed bystander object reaches the observed area or acts on a test
object, or the bystander is defeated before the recording ends.

**D26. Driving launch hold (claim 6).** Repeat S-0504 N4 at game speed index 2, with no engine
add-ons or unrelated mods, using its stock carrier and missile-vessel controls. In named diagnostic map variants,
change only each vessel's locomotor to driving. Give a move order during launching, and separately
initiate launching during travel. Record launching state, position, track endpoints and destination
per tick for three launches of each kind. Expected: an existing track finishes, no new track begins
while launching, the destination remains, and travel resumes when launching ends and the step
is otherwise legal. Mid-track arrest, starting a track during launch, lost destination or failure to
resume falsifies the hold. Use the read-only recorder and scripted orders; concurrent special waits
are not covered by this scenario.

**D27. Stored heading height (claim 24; Q-0676 questions 1 and 2).**
*Setup:* a tiny skirmish map with level ground, a drivable slope and a high bridge
with a traversable ground lane below it, game speed index 2, stock units and rules,
no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Use the read-only recorder with scripted move orders; record
world position, heading point including its stored height, current track and path.
Keep one human house and a bystander house outside the lanes. Run each case once
for a driving vehicle and once for a ship on a legal water route; if a ship cannot
use the slope or deck route with its stock movement zone, use the water control
for it rather than assuming it traverses land.

*Procedure and observable:* record the height immediately before each ordinary
track begins and compare it to the heading height throughout that track. Include
a one-cell straight track, a two-cell curve and a track across changing terrain
height. Separately move under the bridge and along its deck towards a high-bridge
target cell. For the chaining control use D9 control 1's explicitly named test-map
override, with acceleration off and the passive flag on, and a winding corridor
crossing a slope. Repeat it with ship locomotion on a legal test route; this control
is not a claim that stock passive units are available. Qualify the control only
when a chain actually occurs and the vehicle's height or the new target's surface
height differs from the height at the original track start.

*Expected:* every initial heading height equals the pre-progress vehicle height,
not the target's terrain or deck height. It stays equal to that original value
through all qualified chains, even while the vehicle height changes. A completed
track followed by a new track copies the vehicle height anew. A ground approach
under a bridge starts with ground height, not deck height. *Falsified by:* a
heading height set from target terrain or deck instead of the starting vehicle,
a mid-track change, or a height refresh at a qualified chain. S-0511 B6 tests the
bridge-loss consequences separately; record stored and current heights there too.
No oracle run is claimed here.

**D28. Factory exit heading height (claim 24; Q-0793).**
Use a tiny skirmish map at speed index 2 with a stock ground-vehicle factory on
nonzero-height terrain, a produced stock driving vehicle and a remote bystander
house. No engine add-on and no mods are used (the named test files of a scenario and the purpose-made test maps excepted). Script three productions and record each exit
instruction, vehicle world height, apron terrain height, heading point and locomotor
destination point each tick using the read-only recorder. Expected: the exit heading
height is 0 leptons from installation to completion, and a successful exit instruction
also sets the locomotor destination height to 0, despite nonzero vehicle and terrain
heights. A copied starting height, sampled target height or deck lift falsifies the
claim. Vehicle height still follows the surface along the track. If a legal exit can
be arranged beside a high bridge, repeat with destruction during exit and use S-0511
claim 11's two separate tests; do not infer stored height from a hit alone, since the
current-height track test can independently hit. The direct height trace is the
primary discriminator and does not require that optional bridge setup.

**D29. Separate locomotor goal lifetime (claim 25; Q-0794 questions 1 and 2).**
Use tiny land and water skirmish lanes, a stock driving vehicle and stock ship,
speed index 2, a remote bystander house, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Record order-level
destination, locomotor destination, heading point, world position, track completion,
path-search outcome and movement report with the read-only recorder and scripted
orders. Run each control three times for each locomotor. Assign an ordinary reachable
move, then a move into a temporarily blocked lane. Record assignment before search,
a waiting/retry interval, crossing into the goal cell, and final track completion.
Stop a separate run mid-track and issue a replacement goal in another run. Expected:
assignment supplies the locomotor point before any successful search, waits keep it,
crossing the cell boundary does not clear it; qualifying completion clears it before
arrival reporting while an intermediate completion does not. Stop clears it at once
without preventing completion of the existing track. Replacement supplies the new point.
A point appearing only after search success, disappearing during an ordinary retained-goal
wait, or disappearing on the goal cell's boundary falsifies the corresponding rule.

Also assign a point in the vehicle's own cell and record the assignment before subsequent
behaviour processing. Expected: assignment itself leaves a present locomotor point,
not an automatically absent one. Extend S-0314 H19 d and e to record this point
separately for same-cell depot entry and blocked approaches; S-0314 owns the resulting
entry exchanges. Exercise claim 15 give-up once without approach destinations and once
with them, recording any stop and replacement separately: no-approach give-up clears
through destination removal, while fallback follows the actual stop/new-goal effects.
A retained point after normal no-approach give-up falsifies that row. For the exact
height boundary use named test-map docking displacements giving differences of 207
and 208 leptons at completion, provided both legal approaches can be established.
expected clearing only below 208. If those routes cannot be made, the numerical
boundary remains unmeasured in practice; no oracle run or boundary observation is claimed.

**D30. Cell goal versus building goal at rest (claim 26; Q-0797).** Use D29's tiny
land and water lanes, stock driving vehicle and ship, speed index 2, one distant
bystander computer player, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted). Use scripted orders and a read-only
recorder at 800 × 600. Record each vehicle's order-level destination kind and presence,
locomotor destination point, heading, world point, active track, current and queued
behaviour, behaviour-processing ticks, path-search events and cell-arrival reports.
Record immediately before and after locomotion, not only at end of tick. Repeat each
case three times for each locomotor, excluding unrelated orders, enemies and scatter.

- From rest without approach destinations, issue an accepted ordinary move into the
  vehicle's own cell. Expected: assignment supplies both destinations, then the first
  no-track locomotion update removes the cell destination and clears the point, without
  a path search or due behaviour processing being necessary. Record whether this update
  is in the assignment tick. Persistence through that update falsifies the cell rule.
- Issue an ordinary move from several cells away and record its final track. Expected:
  qualifying completion clears only the locomotor destination before arrival reporting
  in tick *t*. The cell destination remains after that update and disappears during
  locomotion in tick *t* + 1, unless a separately recorded intervening effect changes it.
  No due *move* processing is required for removal. Clearing both as the final track
  ends, or retaining the cell goal after the next no-track update, falsifies this timing.
- Repeat S-0314 claim 17's H19 f accepted same-cell entry, with a displaced docking point,
  and record several updates before any stop, replacement or service-state change.
  Expected: the destination names the depot, not a cell; both destinations can survive
  no-track updates despite cell equality. Repeat an ordinary track approach to that depot:
  qualifying completion clears the locomotor point, not the building destination.
  An unconditional locomotion clear of the building destination on cell equality falsifies
  the distinction. Record entry processing and arrival effects separately, so their
  changes are not misclassified as locomotion cleanup.
- Use a named test-map initial *guard* vehicle with a building destination and no active
  track, first with its supplied point exactly equal to the vehicle's world point, then
  with a nonzero horizontal displacement inside the same cell, then a height displacement.
  Arrange and verify the accepted destination before the observed locomotion update,
  and void a run whose guard processing changes it first. Expected: the guard row removes
  the goal only on exact three-coordinate equality. Cell-only clearing falsifies it.
- Repeat the cell-goal case with a verified nonempty approach-destination list, using
  S-0417 claim 5's factory-exit setup to supply one. Record current-goal abandonment,
  any fallback, and any replacement point separately. Expected: the old goal is abandoned
  and S-0215 claim 13 decides the replacement; treating the old goal as permanently
  retained, or clearing the fallback's new goal solely because the old one was reached,
  falsifies this composition. The recorder must establish the no-track equal-cell
  precondition; a run without it tests neither row.

**D31. Goals naming units (claim 26; Q-0830).** Use D30's stock-data land and
water lanes, driving and ship movers, game speed index 2, distant bystander computer
player, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), scripted input and a read-only recorder at 800 × 600.
Repeat each qualifying case three times. Record immediately before and after locomotion:
order-level destination identity and kind, supplied movement point, retained locomotor
point, mover world point, active track, resting state, behaviour, approach destinations,
searches, boarding and destination assignments. Also record the named unit's world point
and movement state. Separate changes caused by behaviour, arrival reporting or boarding
from cleanup during the no-track update.

- Obtain accepted goals naming, in separate controls, a vehicle, infantry unit and
  aircraft. Use a vehicle passenger entering a compatible stationary transport, then
  a moving transport, as ordinary-input witnesses where available (S-0507 claims 5 and 6).
  Observe a no-track update in the supplied point's cell before boarding or replacement.
  Expected: cell equality alone retains the object goal and locomotor point, for both
  stationary and moving named units. A goal naming a cell at the same point is removed
  under D30. Cell-style removal of a unit goal solely on cell equality falsifies the claim.
- For each destination kind, establish and verify a named test-map initial resting
  *guard* mover with the accepted unit destination and a present locomotor point. Test
  exact three-coordinate equality, a horizontal displacement inside the same cell and
  a height displacement. Expected: only exact equality triggers removal, or S-0215
  claim 13's fallback with a verified nonempty approach list. Repeat while the named
  unit moves without a replacement instruction: compare against the retained point,
  not that unit's new position. Different cleanup eligibility solely because the named
  unit moves, retention at exact equality, or clearing on mere cell equality falsifies
  the respective rule. A separately accepted replacement point changes the comparison
  point, as claim 25 requires.

A run is void for a control unless the recorder verifies its destination kind and all
of its no-track, behaviour and point-presence preconditions. A boarding or order exchange
that changes the goal before locomotion is not evidence of cell-style cleanup. If ordinary
input cannot establish a particular unit-kind or guard combination, retain that control as
a measurement proposal needing a verified test-map setup; do not substitute a cell goal
or report it as measured. No memory-writing oracle tool is proposed.

**D32. Destination refresh and dock-transition diagnostic (claim 27).**

Use a tiny flat skirmish map, stock data, speed index 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted),
one human house and one distant bystander computer player. Record placement,
initial idle setup, effective destination assignments, the order-level destination,
current locomotor, locomotor destination point and the two pending states with a
read-only recorder; propose missing recorder support to the user. Include a fresh
ordinary driving vehicle and a fresh jumpjet, without targets or approach destinations.
Record the empty-to-empty destination clear during initial idle setup, its locomotor
stop effect and refresh consumption. It must precede completion of initial idle setup,
not be discarded as an unchanged assignment. For a stock visceroid placed by a legal
map or ordinary creation event, also record the scenario stream around placement:
the initial fallback precedes its placement-only action-timing choice. Record
creation and any earlier placement effects separately, rather than treating the whole
placement as a single draw-free event. An omitted clear, a clear after that later
choice, or a random request attributed to refresh or the clear falsifies the rule.
Observe the same ordinary vehicle's move, stop and applicable deploy orders after
placement. Expected: refresh is absent after setup and those ordinary orders do not
newly set it. Persistence beyond setup or ordinary-order activation falsifies claim 27.
S-0502 J24 supplies the corresponding jumpjet placement comparison.

Conditional diagnostic: if an ordinary stock dock sequence establishes every
predicate of claim 27, record those predicates individually and distinguish
successful temporary-locomotor restoration from the fallback. Record the offered
destination at each due *enter* processing, distinguishing a building offer from a
cell offer, the refinery's offer from a teleport-capable vehicle's repeated dock test
(claim 28), and ignored unchanged offers from effective assignments. There must be
no dock activation solely at first contact, and an unchanged offer without refresh must not
itself activate docking (claim 28's repeated dock test is a separate source).
For temporary driving record point presence, heading versus actual horizontal
position and the restoration lock. A suspended predecessor is restored only with
no movement report, no lock and no seizure. Compare no predecessor, an unfinished
exit and a completed, unlocked exit where stock play permits these states.

Expect successful restoration to set neither pending state. For a teleporting
predecessor, record its landing point and pending jump within this very assignment:
without pre-existing suppression or a disabling state, the offered destination must
fix the teleport landing point, not a driving destination point, and the next
teleporting movement update must jump there under claim 20. Record the driving points
before the offer: restoration eligibility must use those stored points, not the
new offered point. If suppression was already pending, expect it to skip this
application even after restoration. Driving the new leg first, evaluating eligibility
against the newly offered point, or waiting for another assignment to fix the landing
point without another refusal cause falsifies the respective rule.
Otherwise the fallback
stops locomotion, clears current behaviour, queues *enter* without immediate start,
and enables both states during the assignment. Record point applications within that
same assignment, not merely on later ticks: its nonempty point is skipped, including
for driving, and suppression is consumed while refresh remains. Offer the same cell
again through the ordinary docking exchange. If otherwise eligible, it must reach
the test again through refresh; there is no once-per-entry latch. Occupying the cell
with any ground vehicle, including the mover, must prevent this activation. Empty
clears do not consume any still-pending nonempty-point suppression. Record jumpjet
point applications separately if a stock sequence qualifies a jumpjet.
If a pending suppression ever meets a teleporting vehicle, record its landing point,
order-level destination, phasing-in and held states and the suppression state across that
one assignment. Expect no new landing point fixed by this assignment, an unchanged earlier jump, the new order-level
destination retained and suppression consumed, including while phasing in or held. A
dropped destination with suppression still pending, or a landing point fixed despite
suppression, falsifies the suppression table of claim 27.
Any contrary qualified result falsifies the respective rule. No stock fixture
establishing all these diagnostic states is known here; do not manufacture them
through writes or count an unqualified docking run as confirmation. Void a run if
an unlisted bystander object affects the test area.

**D33. Immobilization identity (claim 27; Q-1110).**

Use a tiny flat revealed skirmish map, stock data, speed index 2, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted),
one human house and one distant bystander computer player. Use an ordinary ground
vehicle eligible for a superweapon warp, an own-locomotor teleporting non-harvester,
and a factory-produced chrono miner. With a read-only recorder, log the warp-pending
immobilization, phasing-in and held states, current locomotor, position and tick.
Perform one clean superweapon warp, one own-locomotor jump, and one ordinary factory
exit, then repeat each once. Expect immobilization from accepted superweapon commitment
through the leaving wait, cleared at departure when phasing in begins. Expect the
own-locomotor jump and ordinary factory exit not to set it. An immobilization starting
only after landing, persisting for the arrival hold, or newly set by either control
falsifies this identity. S-0513 claim 39 owns absolute warp timing; this comparison
adds no delay. Void runs affected by an unlisted bystander object. Propose missing
recorder support to the user; do not write state. This scenario distinguishes states,
not the unestablished immobilized/non-teleporting dock combination; D32 remains the
conditional diagnostic for that combination and assignment-time restoration.

**D34. A teleport-capable vehicle's choice of locomotor (claim 28; Q-1759).**
- *Setup:* a tiny flat revealed skirmish map at game speed index 2. Stock data, no engine
  add-ons and no mods, apart from these named test files: D22's test type *T* (below), the
  test rules file of D22 run 2 (part e), the named test copy of the refinery type (part g,
  where stock data needs one) and part f's variant of the test map with no resource cell at
  all. One human house with a refinery and its resource field, and one bystander computer
  player kept away, so that nothing damages *H* (S-0215 claim 16). A chrono miner *H* of
  the human house that has unloaded at least once. D22's test type *T* is the control: it has
  the teleporting locomotor and lacks the teleport-capable flag. A free open cell *X* 20 cells
  from *H* holds no resource and is far from every refinery. Record with the read-only
  recorder every tick: position, active movement kind, contact, order-level destination, landing
  point, behaviour and harvest phase, in part h its body facing, and in part e whether *H* has
  a movement report (claim 25) and the tick of each processing of *enter*. Pending destination
  refresh is not read. A jump is a change of more than 2 cells in one tick (S-0402 scenario H8); in
  part h, where the scatter cell may be next to the docking cell, a jump is a move to the recorded
  landing point in one tick.
- *Procedure:*
  - Part a: give *H* a move order to *X* in each of these states: seeking, loading, returning
    full before it reserves, returning in its *enter* approach after it has reserved the
    refinery, and in *guard* with nothing to do.
  - Part b: from the seeking state, repeat with an attack-move to *X*, a guard-area order at
    *X*, a two-point waypoint path, and a harvest order on a resource cell 20 cells away.
  - Part c: let *H* run its cycle untouched for three loads.
  - Part d: give *T* part a's move order.
  - Part e: park a vehicle of the human house on the refinery's docking cell. Let a full *H*
    reserve the refinery and approach, starting its return from a resource field whose nearest
    cell is between 15 and 40 cells from the docking cell. The test rules file of D22 run 2 sets
    the chrono-harvester too-far distance (S-0402 claim 9) to 50 cells, above the field's 40-cell
    bound, so that *H* reserves and takes the *enter* approach. Move the parked vehicle away while
    *H* is at least 6 cells from the docking cell.
  - Part f: on a variant of the map with no resource cell at all, build a refinery and record its free miner for 300 ticks from the tick it
    appears. With nothing to harvest it gets no destination from its harvest cycle. Repeat on
    the main map, with a resource field at least 10 cells from the new refinery, up to its
    first load.
  - Part g: as part c, with a refinery whose type has a base drain above 0 or the needs-power
    flag (a named test copy of the refinery type sets one if stock data does not). On the tick
    after one of *H*'s docking jumps, the scripted input gives the refinery a power-off order
    (S-0405 claim 10). A run in which the recorder shows *H* tethered before the refinery is
    off is void.
  - Part h: as part c, with a second refinery of the house far from the first. The resource
    field lies within the range of *H*'s long-range ring search from the docking cell (S-0402
    claim 4), with no vehicle on its cells, and no resource cell lies within 5 cells of the
    docking cell, so that a scatter cell holds no resource. Run 1: the
    scripted input sells the first refinery while *H* unloads there, after its first payment. Run 4:
    as run 1, before the first payment, while *H* is still full. Run 2: it sells the refinery on a
    tick at which *H* stands on the docking cell after its docking jump, in *enter*, once its body
    has stopped turning, before it starts unloading. Run 3: as run 2, on the tick after the docking
    jump, while its body is still turning to the unloading facing. A run whose recorder does not
    show the stated state at the sale is void.
  - Part i: give *H*, empty and in *guard*, a move order to a free cell beside its own
    refinery that holds no resource, and record until its first trip after it has arrived.
- *Expected:*
  - Parts a and b: *H* never jumps. It drives a path to every ordered cell, and its active
    movement kind is driving from the order's tick.
  - Part c: the only jumps are to the docking cell, one per load, each during *enter* while in
    contact. Its active movement kind is driving again from each jump tick and stays driving
    through the docking and the unloading, and its trips to resource cells and to any queueing
    cell are driven. Its behaviour stays *enter* across the docking restoration, the jump and
    the installation of driving.
  - Part d: *T* jumps to *X*.
  - Part e: while the cell is occupied, *H* drives towards it only on a route found without a
    substitute; at a path computation that adopts one it stops at the end of its track and stands
    (S-0402 claim 10a, *A chrono miner*, and its scenario H20 part g). After the cell clears, *H*
    either jumps to it at a later *enter* processing or, if it reaches the cell by driving before
    such a processing, arrives there by driving (claim 28, *An occupied docking cell*), and the run
    records which. If *H* still has a movement report at the first processing after the cell
    clears, the recorder first shows it stopping, and the jump follows either in that same
    processing, when a second assignment of the cell there finds it with no movement report,
    or at the first later processing at which it has no movement report (*One jump*).
  - Part f: the free miner's behaviour is *harvest* from the tick it appears, and its active
    movement kind is driving on every recorded tick from its first, including the ticks before
    any harvest-cycle destination assignment on the variant map. On the main map it drives to
    the resource field without a jump.
  - Part g: at the first processing of *enter* after the switch-off, *H* loses its contact
    without being tethered, its behaviour falls back to rest, and its active movement kind
    changes from driving to teleporting at that fall-back (S-0215 claim 13). *Falsified if* *H*
    is tethered or unloads at the switched-off refinery, or keeps driving as its active
    movement kind after the fall-back.
  - Part h: in runs 2 and 4, *H* jumps, in the sale's tick or the next, to a landing point at a
    scatter cell near the docking cell; its active movement kind is driving at the end of the
    jump tick. In run 4 its behaviour is *harvest*, and after the jump it goes *returning* with
    its full load; its next trip is driven, or is the docking jump at the second refinery. In
    run 1 its behaviour is *harvest* from the sale's tick, and it makes no move to a scatter cell:
    in the sale's tick or the next it jumps to the resource cell that its long-range ring search
    finds from the docking cell (S-0402 claims 4 and 17, case (f)), with the unpaid rest of its
    load, its active movement kind is driving at the end of that jump tick, and it then loads
    there. Since the assignment and the jump can fall in one turn, these runs do not expect
    teleporting as the active movement kind at the end of any tick. In run 3 it gets no scatter
    destination and does not move at the sale. In runs 2
    and 3 *H* has *guard* queued from the sale, and the run records which comes first:
    - a processing of *enter* at which *H* falls back to rest (the first one if it is not
      tethered, else the first after the contact has ended): its active movement kind
      changes from driving to teleporting at that fall-back, and *guard* or *harvest* becomes
      current (S-0215 claim 15 row 4),
    - or the queued *guard* becoming current at the first tick at which *H* is ready to
      change (S-0215 claim 11): in run 2 after its jump, in run 3 once its body turn has
      ended. Its behaviour goes from *enter* to *guard* with no fall-back, and its active
      movement kind stays driving.

    *Falsified if* *H* drives to the scatter cell in run 2 or 4, in run 1 moves to a cell that
    holds no resource or reaches the field by driving before it loads, jumps in run 3, keeps driving
    as its active movement kind at rest after a fall-back of run 2 or 3, shows teleporting as
    its active movement kind on the tick at which *guard* becomes current with no fall-back,
    or has *guard* become current in run 3 while its body turn is still in progress.
  - Part i: when *H* arrives and its behaviour falls back to rest, its active movement kind
    changes from driving to teleporting (S-0215 claim 13). *Harvest* is then queued (by the
    fall-back, S-0215 claim 15 row 4, or by S-0402 claim 17's *guard* rule with the refinery
    next to it), and at its first seeking processing its active movement kind becomes driving
    and it drives to the cell the processing sets. *Falsified if* it jumps there, or shows
    driving as its active movement kind at rest after the fall-back.
- *Falsified if:* *H* jumps to *X*, to a resource cell (outside part h run 1) or to a queueing
  cell. Also falsified if *T* drives in part d. In part e it is falsified if *H* jumps while the
  cell is still occupied,
  or if, after the cell has cleared, *H* does not jump in the tick of a recorded processing of
  *enter* or the next, with its contact accepting and the cell clear at that processing, while the
  recorder shows it with no movement report (claim 25) at the end of the tick before that processing
  and of the processing's own tick. A stop with no jump at a processing where it still has a
  movement report is claim 27's fallback and does not falsify. In part c it is
  falsified if the behaviour becomes *guard* at a jump tick. In part f it is falsified if the
  free miner's active movement kind is teleporting on any recorded tick, in particular before
  its first harvest-cycle destination assignment on the variant map, or if it jumps before its first
  docking.
- *Runs:* 3 for each part.

## Cross-spec changes

- Q-1759: S-0402 Terms (*chrono miner*), claim 17 and scenario H8 assume that a chrono miner
  with active teleporting locomotion jumps to resource cells, queueing cells and a player's
  destinations. Under claim 28 it drives to all of them, and jumps only to a refinery docking cell
  and at a sale of that refinery. S-0402 carries this, together with claim 11's sale scatter (*A
  sale of the refinery*, claim 28 here) and claim 10's account of the vehicle's repeated dock test
  in *enter* (claim 28 here) beside the refinery's docking-cell offer. It rests on claim 28.
- Q-1759: S-0214 (Terms, *self-teleporting type*) describes the flag as a type that teleports
  itself to where it is sent. That holds for infantry. A vehicle with the flag drives to a
  player's destination (claim 28 here), even though claim 11's targeting still takes the
  clicked cell without a search, so the vehicle can be sent to a cell it then cannot reach.
  S-0214 should restate the term.
- Q-1759: S-0417 claim 16 should say that the factory-built chrono miner's temporary
  driving does not end its driving trips. A later restoration makes it teleport only on a
  refinery docking assignment (claim 28 here). S-0417 claim 16 carries this.
- Q-1759: S-0215 scenario B15 part (e) produces a stock teleporting vehicle (a chrono miner,
  whose type has the teleport-capable flag) and expects that, after the restoration at its
  first eligible fall-back, an order to a distant clear cell makes it jump, with "later
  continued driving" as a falsifier. Under claim 28 that order installs driving again and the
  vehicle drives there. The restoration itself is unchanged and stays visible: the recorder
  shows the active movement kind change from driving to teleporting at that fall-back, and back
  to driving on the order's tick. B15 part (e) should expect that sequence, and be falsified by
  no restoration at the fall-back or by a jump to the ordered cell. A docking jump at a later
  refinery approach (S-0402) is a further sign that teleporting locomotion was restored.
- Q-1759: S-0417 scenarios U6 and U15 expect the stock teleporting vehicle to jump after
  restoration: U6's "a newly taken destination after restoration fixes a landing point and jumps under S-0207 claim 20", with driving for that
  order as a falsifier, and U15's "the next accepted distant move then uses teleporting
  locomotion". Under claim 28 both orders are driven, because neither is a qualified docking
  assignment. Those expectations hold only for a vehicle type with the teleporting locomotor and
  without the flag, such as D22's test type *T* here. S-0417 should expect driving for the stock
  vehicle in both scenarios (restoration shown as for B15 above), or run the jump expectation
  with such a test type. S-0417 carries this: it expects driving for the stock vehicle in both,
  with the restoration shown as for B15.
- Q-1759: S-0215 claim 9 lists "an actor whose locomotor is replaced by another" among the
  guard resets; its backing is magnetic seizure's replacement. It should say that the temporary
  driving of claim 28 here, and the restorations of claim 27 here and of S-0215 claim 13, are
  not such replacements and leave the behaviour as it is. Scenario D34 part c here observes
  both kinds at each ordinary docking jump: claim 27's restoration just before it and claim
  28's install at it, with *enter* current throughout.
- Q-1759: claim 20 e here cites S-0215 claim 13 for the fall-back to rest that follows a jump's
  removal of the destination. After a docking jump that removal does not end *enter*: the
  behaviour goes on while the refinery accepts the vehicle, and falls back to rest only when
  the refinery refuses it or the contact ends (claim 20 e, claim 28). S-0215 claim 13, or
  wherever S-0215 describes a fall-back after a jump, should note that exception.

- Q-1130: S-0514 claims 16 and 17 say an erased object's destination is cleared, and claim 16 says
  that stops its teleporting jump. Under a pending suppression (claim 27 here) the destination is
  kept and no jump follows. S-0514 must name that exception.
- Q-1130: S-0215 claim 2 says an infantry unit or vehicle that is being erased drops its target and
  destination on each such tick. Under a pending suppression (claim 27 here) a vehicle's destination
  is kept. S-0215 claim 2 must name that exception. Its leaving-stage clause is not changed by this.
- Q-1130: S-0214 (Terms, "Being erased, phasing in") says a destination given to the teleporting
  locomotor while phasing in is dropped. Under a pending suppression (claim 27 here) it is kept.
  S-0214 must name that exception.
- Q-1110: S-0502 claim 27 must replace the undefined aftereffects predicate with
  warp-pending immobilization from claim 27 here, not phasing in, held or a cooldown.
  the predicate is checked independently of current locomotor. Its successful-restoration
  case must route this assignment's point to the restored predecessor, subject to
  already-pending suppression and ordinary destination refusal.
- Q-1110: S-0417 claim 5 and U6 must distinguish assignments completed while driving
  from an assignment that itself restores the predecessor under claim 27 here: the
  latter applies its point to the restored movement, not to driving first. S-0417 owns
  the factory exit point-clearing event and the first ore leg asked about in question 3.
- Q-1110: S-0402 claim 10 and H10 should state that a qualified successful restoration
  routes the same docking-cell offer to teleporting movement under claims 20 and 27
  here, subject to existing suppression and disabling states; no preliminary driving
  leg or second offer is required by restoration itself.

- S-0502 claim 27 and J24 must use claim 27 here for the effective placement clear,
  assignment-time dock test and same-assignment suppression. Newly enabled refresh
  can remain after that assignment; a qualified repeat is not limited by an entry latch.
- S-0502 claim 27 must name one exception, claim 28's repeated dock test in *enter*, in
  two places. In "evaluated at each otherwise eligible destination assignment after
  unchanged-destination rejection", add after "rejection": ", and at S-0207 claim 28's
  repeated dock test in *enter*, which that rejection does not precede". Replace the two
  sentences "First contact, arrival and entry attempts do not independently trigger it. An
  unchanged offer without refresh does not reach it even if contact or occupancy has
  changed." with: "First contact, arrival and entry attempts do not independently trigger
  it, except S-0207 claim 28's repeated dock test in *enter*: at each processing of *enter*
  at which its contact accepts it, or at which it is tethered, a vehicle whose type has the
  teleport-capable flag and has a destination has that destination tested as an assignment,
  whether or not it has changed and whether or not refresh is pending. Apart from that test,
  an unchanged offer without refresh does not reach it even if contact or occupancy has
  changed."
- S-0417 claim 5 and U6 must distinguish completed exit geometry from restoration
  eligibility in claim 27 here: a retained locomotor destination point or restoration
  lock still prevents release. Ordinary restoration enables neither dock pending state.
- S-0402 claim 10 row 2 and H10 must distinguish repeated docking-cell offers from
  effective assignments. Only an eligible assignment, or claim 28's repeated dock test in
  *enter* for a teleport-capable vehicle, reaches claim 27's dock test, and fallback normally suppresses that assignment's point while retaining refresh.

- Q-0797: S-0215 claims 22 and 29 should cite claim 26 for locomotion's cell-goal
  cleanup and distinguish it from due behaviour processing. S-0314 claim 17 and H19 f
  should make the retained same-cell entry case explicitly a building destination, not
  an ordinary cell move. S-0504 claim 1 should include claim 26's cell-goal cleanup,
  and its blanket statement about standing in the goal cell needs that qualification.
  S-0212 claim 20 should distinguish this no-track cleanup from search failure or give-up.
- Q-0793: S-0417 claim 5 and S-0511 claims 9 and 11 must distinguish claim 24's
  factory exit heading height of 0 leptons from ordinary tracks' copied starting height.
- Q-0794: S-0314 claim 17 must use claim 25's independent point lifetime, not cell
  inequality, and answer question 3 from its depot entry rules. S-0504 claim 1 should
  explicitly include claim 25. S-0212's search/retry rules must not make successful
  search a prerequisite for this point's presence.

- Q-0676: S-0511 claims 9 and 11 should cite claim 24 here for the driving/ship
  heading point's copied starting height and retention through chaining. S-0511
  owns question 3's distinction between that stored height and current height.
- S-0504 claim 1 should explicitly include claim 24 here in its shared rules,
  with D27 as the proposed height-specific measurement; N1 and N2 alone do not
  distinguish the possible height assignments.

- Q-0356: S-0209 claim 1 must extend its vehicle and aircraft lifecycle pointers to
  claim 20 k here: removal clears actual occupancy, not a separate remembered or
  pending ground reservation. After a replacement destination, old ground is already
  clear and the new ground reservation survives removal. Its temporary-driving pointer
  should cite claim 21: installation clears neither mark, driving departure clears
  actual deck only, and restoration relocates no mark. Complete both shared
  ground-reservation quirk alternatives to clear pending reservation and actual
  occupancy on removal. Entry and standing-position effects remain S-0209's.
- Q-0310: S-0209 must add claim 20 e's aircraft reservation lifecycle, share
  `quirks.teleport_aircraft_ground_reservation`, and distinguish entry outcomes from
  infantry standing-position refusal for the isolated object-present flag below the
  bridge. S-0209 claim 1's unmeasured justification does not cover this fixture.
- Q-0268 part 1: S-0011 claim 18 should cite claim 22 here for driving and teleporting
  arrival timing; ordinary infantry walking remains S-0208's responsibility.

- Q-0240: S-0209 claim 1's pointer and claim 5's reservation consequences match this
  spec. Claim 20 a/e here owns the teleporting vehicle's ground
  reservation, remembered-point clearing and two-record bridge placement; S-0209 owns
  the entry outcomes and shares `quirks.teleport_vehicle_ground_reservation`.
  A reservation is not necessarily in the cell layer where the vehicle is listed.
  Retain S-0212 claim 27's summary-only role: S-0209 claim 5 owns the full vehicle
  entry-outcome table.
- The coordinated blocked-first-check rule is already stated in claim 15, D17(d3)
  and S-0212 claim 22: immediate fresh search despite a running retry timer, then
  same-tick repeated check when a path is found.
- S-0208 claim 19's infantry refused-replacement rule remains different from claim 20 b's
  retained non-infantry landing point; its actual deck landing follows claim 20 e.
  S-0215 claim 10's target-loss exception and restored-target distinction likewise remain
  coordinated with claim 20 e. These checks require no new rule in those companions.
- Claim 20 b and S-0212 claim 25's non-infantry teleport row state the same rules:
  destination-centred search, own unsimplified zone, destination reach region with no
  restriction when absent, strict deck-mode height threshold, no height or further
  condition, bridges allowed, and selection on the landing-point fixation tick.
  S-0208 claim 19 owns the different infantry fallback parameters and distinct deck
  modes; neither search waits until the jump to select its result.
- Claim 21, S-0417's temporary-driving contract and S-0215 claim 13's restoration contract agree:
  destinations accepted while driving do not change the suspended teleport landing point or
  reservation, and restoration alone resets neither retained state nor occupancy. Later fall-back
  destination changes are separate from restoration itself.
- Q-1759: S-0215 claim 13 says that S-0207 claim 21 distinguishes "a retained pending jump,
  landing point and phase-in expiry". Under claim 21 a jump pending when temporary driving is
  installed is cancelled then, so none is pending at restoration. S-0215 claim 13 should list
  only the landing point and the phase-in expiry as retained.
- Q-2781: S-0607 claim 9 can point to claim 16 here for a seized vehicle that accepts a scatter
  request: no seized refusal of the request, case e against the grip's destination, the draws
  taken, *move* queued for a request with a threat point, the new destination refused as claim 9
  says, and what that *move* does once current, while the vehicle is seized and after its
  restoration.

## Open questions

- Whether any speed field of a country or of a difficulty level other than the country's vehicle
  speed factor reaches a vehicle's speed. None does in the formula of claim 2. It is believed that no
  other rule of the original uses them, but this has not been searched exhaustively. This also bears
  on S-0006 open question 7.
- Which further events set a vehicle sinking beyond the killing-hit condition of S-0312
  claim 10 and the cull of S-0614 claim 11 (which delivers no hit and does not change health).
  A reading (low confidence) points to a non-naval vehicle arriving by teleport on a water cell
  that is neither a high-bridge cell nor a road, and a unit landing on a cell where it cannot
  exist. Their exact conditions remain for the relevant movement specs to settle. Behaviour
  once sinking is settled: S-0504 claims 4 and 5 own it.
- Whether a game started with the capture-the-flag option on places flags that a vehicle can carry
  (scenario D15 checks this as its precondition).
- The reactions to a curve's second cell (claim 15, T5) were read at low confidence.
- The heading points (Terms) of locomotors other than driving, ship, hovering and teleporting,
  which decide a vehicle scatter centre cell for such vehicles (claim 16), remain for their
  specs. The ship case is settled by S-0504 claim 1 and the hover case by S-0503 claim 5.
- Whether a stock setup can establish an airstrike aimed at a living object that makes its
  own teleporting-locomotor jump (claim 20 e). An erasing-weapon lock supplies no such
  exception: an erased object drops its destination and cannot jump (claim 20 a,
  S-0514 claim 16), except that a pending suppression (claim 27) keeps the destination without a jump.
- Whether any stock setup gives a vehicle a pending teleport or a phasing-in state before
  temporary driving begins (claim 21). Factory exit establishes the ordinary case, not those
  conditional states.
- Whether an object of a type without a locomotor field that is not infantry, landing by claim 20
  on a cell where it cannot exist, starts sinking (see the sinking question above) was not traced, and
  claim 20 b's search normally avoids such cells.
- Whether a value of the locomotor field with further characters after the closing brace counts as
  a class identifier (claim 14) was not checked. The answer may depend on how the host system
  turns the text into an identifier, rather than on the game itself. Our importer treats such a
  value as not a class identifier and warns. Where the original's answer rests on the host system,
  this fixed choice is a departure of the host kind (glossary *departure*, host or real time); a
  check of the original on the reference host would show whether it is one.
- Whether a vehicle keeps its speed fraction while it waits at a refused step of table T4 (claim
  15) was not traced; claim 11 covers only a vehicle that comes to rest, and the rows of T5 that
  stop the vehicle give *ρ* = 0.
- Which cells decide crush mode (claim 7) was read but not exercised. Scenario D10 tests it (runs
  C to F for the track, runs A and B together with the start of the crush-slowdown state, which
  S-0210, claim 9 and scenario K5, owns).
- The original makes a second look for a nearest object, by the same rule as F8: when a fresh
  search finds no path while the vehicle is near its destination, it looks at the cell in its body
  facing's step direction. That case was not traced far enough to state it, and this spec does
  not describe it; the rule of F8 (top corner, not centre) is the one to reuse if a later
  revision does.
- How an infantry or aircraft object whose type names the driving locomotor (claim 14) moves.
  This spec covers drive vehicles only (Terms). The following possible differences from
  claims 2 to 13 and 15 are `unmeasured`, with low confidence: no scenario here tests movement
  for these combinations of object kind and locomotor:
  the country speed factor is the owner's country's factor for the object's own kind (for an
  infantry unit the factor of S-0208 claim 7); the slope factor of claim 9 (c) is 1 for every kind
  but the vehicle kind; the passive flag, the flag carrier's halving and the copying of the speed
  fraction to follower cars belong to the vehicle kind only; the progress of an infantry unit in the
  prone state is changed as S-0208 claim 8 gives; and the step check uses the entry outcome of the
  object's own kind (S-0209 claim 4 for an infantry mover, and claim 14 for an aircraft, which gives
  *clear* everywhere). How the occupancy record (S-0209 claim 1) and the release points of T1 work
  for such an object was not traced. A later revision, or the infantry and aircraft movement specs,
  should settle this.
- How a vehicle that the map places on a high-bridge cell, or that leaves a building onto one,
  starts on or off the bridge deck (claim 5 covers only the changes made while it drives). The
  specs of map placement (S-0009) and of the structures own this.
- The hovering locomotor's fresh search after a wait (claim 19 d) is read as being retried on every
  tick until *R* has run out; scenario D21 run 4 records the search ticks.
- Whether a vehicle at rest keeps its leftover budget on every way of arriving (claim 3). Scenario
  D8 part b tests the end of an ordinary move only.
- Whether interpolation (claim 4) can move a vehicle into a cell that it has not yet claimed under
  the occupancy rules (S-0209). This is the shared Open question of S-0209 on
  between-point movement and reservation timing. Any resolution must reconcile claim 4
  with S-0209 claim 1's reservation and release rules and be handed off in both specs together.
- S-0417 table T1 and claims 5 and 6 now specify the vehicle-factory exit track and its initial
  and updated speed fractions. Other building tracks remain separate engine data belonging to
  the structures specs. A reading, not exercised (low confidence), points to two rules
  for them. On such a track, a vehicle whose acceleration flag is on gets no update of claim 10 at
  all: none of its six cases applies (the braking and sinking floors and the crush slowdown
  included), and a leader copies nothing to its cars. A vehicle whose acceleration flag is off still
  takes *ρ* = clamp(*ρ*ₜ, 0, 1). And a move that begins while the vehicle is on such a track sets
  its speed fraction straight to the new target fraction of claim 9, without acceleration. The
  second rule would also give a passive vehicle a speed fraction (claim 8). The structures specs
  should settle both.
- (Claim 28.) Which ways of creating a teleport-capable vehicle make no destination assignment at
  its placement (the third window), and what the movement kind is after S-0607 claim 4 ends a
  resource-gathering vehicle's temporary driving. Also the three exclusions and the nested
  replacements: no ordinary-play fixture is known for any of them.
- (Claim 21.) How the original handles a jump pending when temporary driving is installed:
  claim 28's sale branches reach it, but no scenario records the pending jump there. Our
  engine's cancellation is a design rule.
