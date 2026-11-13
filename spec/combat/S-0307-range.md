---
id: S-0307
title: Weapon range test
topic: combat
revision: 24
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-13
measurements: []
requests: [Q-0172, Q-0230, Q-0237, Q-2171, Q-3069]
supersedes: []
---

# S-0307: Weapon range test

## Summary

When the original asks whether a target is in range of a weapon, it applies one test, the *range
test*: the weapon's range field, changed by bonuses and replacements and by height, is compared with
the table-root distance between a source point and a measured point, with refusals for bridges and
for a blocked line of fire. The spec also defines the *firer's weapon range* (claim 13), which other
rules read and which differs from the range test's own range for open-topped transports. How
projectiles fly once fired is S-0306. Claim 15 adds the verified open-ground part of choosing an
approach destination for an attacking infantry unit or vehicle. This is not yet a complete answer
to Q-0072: claims 16 to 18 add obstructed level-ground searches, failure outcomes and a special
object destination. S-0215 claim 37 owns the trigger, gating and destination renewal.
S-0207 claim 19 owns the resolved vehicle movement outcomes; S-0208 claims 16 and 17 own ordinary
walking infantry's stopping and destination outcomes. Claims 19 to 21 extend ordinary searches to terrain, bridges and ordinary cell targets,
and resolve their failure centre and usable-area exception for Q-0172. Claim 22 gives the search
of types with the close-range property, claim 23 the replacement of a destroyable-cliff cell
target, and claim 24 the one rule that treats jumpjets differently. Deploying searches and
jumpjet stopping remain open below. Claim 21 resolves the usable-area/reach-region distinction
of Q-0230. Claim 22 b cites S-0208 claim 21 for the ordinary walker's shortcut outcome.
Claim 25 resolves the non-walking outcome of Q-0237. Claim 19 reads a high-bridge candidate's
reach region for its deck, so deck cells over water can qualify (Q-2171).
Claim 27 states what the test gives a floor-bound firer against an airborne target (Q-3069).
Skirmish is covered; campaigns and network games are out of scope.

## Terms

Glossary terms are used as defined there, in particular *lepton*, *cell*, *height level*, *world
height*, *foundation*, *reference point*, *high bridge*, *bridge deck*, *weapon*,
*target*, *round-toward-zero mode*, *game-wide section*, *importer warning*, *recorder*. From S-0204:
*table square root* *r*(*n*), *planar distance*, *spatial distance*, β(*B*) (claim 6; its use in the
range test is claim 7 there). From S-0205: RZ₅₃ and trunc (truncation toward zero to
an integer). From S-0301: *firer*, *firing check*. From S-0306: *arcing projectile*, *arc speed*
*v*(*L*, *g*), the *reachability test*, the *line-of-fire walk*, *gravity* *g*, *subject to cliffs*
and *subject to walls*. In addition:

- **Range test.** The yes/no question "is target *T* within reach of weapon *W* of firer *A*,
  measured from source point *S*". The firing check's *out of range* answer (S-0301 claim 9) is its
  "no". Other rules that use it say so (claim 12).
- **Weapon range** *R*₀. The weapon's range field converted to leptons (claim 1).
- **Effective range** *R*. *R*₀ after the bonuses and the replacement of claims 3 to 5 (F1). The
  elevation bonus of claim 6 is not part of *R*: it is added to the comparison limits of claim 9.
- **Airborne, floor-bound.** An ordinary object on the map is *airborne* when its height above
  the surface is at least 208 leptons (two height levels), and *floor-bound* otherwise.
  Aircraft use the in-the-air answer of S-0501 claim 18, including its two rocket-type
  exceptions, as qualified in claim 26 below. An ordinary object not on the map is neither,
  with one exception: a superweapon warp member between its departure and its appearance (S-0513
  claim 39), which is off the map then. For the rules of a projectile's flight and detonation that
  target it (S-0306 table T1, table T2, claim 20 and claim 17's loss), it is counted by its height
  above the surface beneath its kept position, so one that departed from a high-bridge deck counts
  as airborne, except while a landing check that passed for a landing cell on a deck awaits its
  placement (S-0306 claim 17, `quirks.warp_deck_departure_airborne`, which S-0306 owns). Medium,
  `unmeasured`, as S-0306 claim 17 rates it.
- **Measured point** *M*_T. The point of the target that the range test measures to (claim 7). It
  is not S-0306's target point: it has no aim shift, and its height may be replaced.
- **Source point** *S*. The point of the firer that the range test measures from (claim 8).
- **Subject to elevation.** A projectile type property (Import mapping). It switches on the
  elevation bonus of claim 6.
- **Cell rangefinding.** A weapon property (Import mapping). It moves the source point to the
  firer's cell (claim 8 a).
- **Close-range property.** A boolean type field (Import mapping). Claims 15, 16, 19 and 20
  cover only types without this property; claim 22 gives the differences for types with it.
- **Standing position, position class, position choice.** As in S-0208 (Terms, claims 1 to 3):
  the *position choice* for a cell and a desired point is the standing position that S-0208
  claim 3's table gives for that cell's occupancy record, including its random draw.
- **Scatter reference point.** As in S-0208 (Terms): the actor's walking target while it has
  one, otherwise its own position. Claim 23 uses it for vehicles as well, with the next point its
  locomotor is heading for in place of the walking target.
- **Destroyable-cliff cell.** As in S-0214 (Terms). Its tile's cells and sub-tile are those of
  S-0010 and the glossary (*sub-tile*); a *tile cell* is a non-empty cell of a tile (S-0010 Terms).
- **Hunter-seeker property, balloon-hover property.** Boolean type fields (Import mapping), read
  only for the exceptional destinations of claims 17 and 18. This spec does not define their
  special abilities or locomotion.
- **Vertical projectile.** A projectile with the vertical-flight flag of S-0306 (claim 11),
  imported as in Import mapping.
- **Approach-position search.** Choosing a firing destination after S-0215 claim 37 has decided
  to search. It is distinct from the nearby-cell search of S-0212 claim 25 and from computing a
  path to the destination. *Approach destinations* means S-0215's held list of destinations.
- **Wall-damaging warhead.** A warhead with the wall-damage property (Import mapping). It is the only
  property of a warhead that this spec reads (claim 11).
- **Open-topped transport.** A transport type whose passengers fire from inside it. Its passengers
  and its own weapon are each a firer.
- **Garrisoned building.** As in S-0301 (Terms): an occupiable building that may fire while
  occupied and has at least one occupant.
- **Firer's weapon range** ρ(*A*, *i*). The reach that other rules (flak scatter, S-0306 claim 4, and
  the scan and guard reach of S-0309, and the approach radius of claim 15) read for firer *A*'s
  weapon in slot *i* (claim 13). It is not
  the range test's *R*.
- **Elevation step count** *k*. trunc(Δ*h* / *I*) (integer division) for the elevation-level
  difference Δ*h* ≥ 0 between the firer's cell and the target's cell and the game-wide elevation
  increment *I* ≠ 0 (Formulas F2). It is ⌊Δ*h* / *I*⌋ for *I* > 0 and ≤ 0 for *I* < 0.
- **Elevation-deck mark.** The cell mark used by the elevation bonus and by the deck-cell depth
  lift of S-0102 claim 13. It is distinct from both the high-bridge-cell test and S-0212's
  deck-entry mark: the latter controls movement onto a deck, not this bonus. An intact
  high-bridge overlay sets the elevation-deck mark on its own cell; destroying that overlay
  clears it there. Its bridge marking does not spread this mark to the neighbouring cells
  that receive the high-bridge and deck-entry marks. Overlapping bridge markings and their
  repair lifecycle remain outside the verified marking scope (Open questions).
- **Elevation level of a cell** λ(*c*). The cell's height level, plus 4 height levels when it
  has the elevation-deck mark. It depends on the cell only, not on whether the object stands
  on the deck or under it. Claim 6 carries this definition's backing.

## Behaviour

### The range field

1. **Weapon range in leptons.** The weapon's range field is a decimal number of cells. The
   importer reads it with the decimal form of S-0003 (claim 6, single precision), multiplies it by
   256 in double precision and truncates toward zero: *R*₀ = trunc(RZ₅₃(*x* × 256)) leptons. A
   value read as exactly −1 is treated as absent and keeps the value already in force (the
   absent-key default is 0). The minimum-range field and the type's air range bonus field use the
   same form. The domain is a field magnitude below 8 388 608 cells (|*R*₀| below 2³¹); beyond it
   the value is not covered.
   - Backing: `measurable` (RG1 a, b and d for weapon conversion, e for negative weapon
     fields, f for minimum-range conversion; RG2 g for air-bonus conversion and negative
     air bonuses). `unmeasured` for distinguishing a negative minimum range from zero:
     every non-negative measured distance passes both, so there is no differing range answer
   - Confidence: high

2. **Unlimited range.** A weapon whose *R*₀ is exactly −512 (a range field of −2) passes the range
   test for every target, without any other range-test condition being applied (no minimum range,
   no height rule, no line-of-fire walk).
   - Backing: `measurable` (scenario RG1, run c)
   - Confidence: high

### Effective range and elevation bonus

3. **Air bonus.** When the target is airborne, the firer type's air range bonus (leptons, claim 1)
   is added: *R* = *R*₀ + air bonus.
   - Backing: `measurable` (scenario RG2, run a)
   - Confidence: high

4. **Garrisoned buildings replace the range.** When the firer is a garrisoned building, its
   effective range is *R* = 256 × (⌊min(*W*, *H*) / 2⌋ + *G*) leptons, where *W* × *H* is its
   foundation and *G* the game-wide occupied-weapon range (whole cells). This replaces the value
   of claim 3: the weapon's own range and any air bonus are not used. (Quirk
   `quirks.garrison_range_drops_air_bonus`.)
   - Backing: `measurable` (scenario RG2, runs b and c)
   - Confidence: high

5. **Bunkers and open-topped transports.** A firer that is bunkered and is not a building (a vehicle
   inside a tank bunker; glossary) adds 256 × the game-wide bunker range bonus (whole cells). A
   firer that is a passenger of an open-topped transport adds 256 × the game-wide open-topped range
   bonus (whole cells). Each is added to the range that claims 3 and 4 give (so a garrison
   replacement does not remove them), and the two additions are independent of each other: a firer
   to which both apply gets both. No scenario here arranges the combination.
   - Backing: `measurable` (RG2 d and e, each addition alone); `unmeasured` for the combined
     state and a garrison replacement together with either addition, since no reachable fixture
     for those combinations is established
   - Confidence: high for each addition; medium for the combinations

6. **Elevation bonus.** When the weapon's projectile is subject to elevation and both the firer
   and the target are floor-bound, a bonus *E* determined by the height-level difference is
   added. It differs by projectile kind: *E*₁ (F2) for a projectile that is not an arcing
   projectile, *E*₂ (F2) for an arcing projectile. Without the subject-to-elevation property, or
   when either object is airborne or off the map, *E* = 0. When the firer's cell has an elevation
   level lower than or equal to the target's, Δ*h* = 0 and F2 gives *E* = 0 for every cap *c* ≥ 0
   (a negative cap is an edge case below). The levels compared are elevation levels
   (Terms), so an object under a bridge counts as 4 levels higher only when its cell has the
   elevation-deck mark, not merely because it is a high-bridge cell
   (quirk `quirks.elevation_deck_cell_level`).
   - Backing: `measurable` (RG3, including d for the distinct bridge marks; RG14 for zero
     increment; RG15 for overflow; RG22 for negative increment and cap); `unmeasured` for
     off-map objects, since no valid off-map firing query is established, and for F2's
     out-of-64-bit conversion case, since no practical fixture isolates that conversion
     from the later wrap and range refusals
   - Confidence: high for the formulas and the distinction between the marks; medium for the
     bridge-mark lifecycle and the absent-section case

### What is measured

7. **Measured point.** *M*_T is the target's reference point (S-0204 claim 4: the centre of a
   building's foundation, without any bib row). When the target is floor-bound, the world height
   of *M*_T is replaced by the terrain height at *M*_T (on a slope, the height of the sloped surface
   at that point, S-0106), plus 416 leptons when the cell under *M*_T carries a high bridge. So a
   floor-bound target counts as standing on the ground surface,
   and a floor-bound target in a high-bridge cell counts as standing on the bridge deck even when
   it is under the bridge (quirk `quirks.range_target_on_deck`).
   - Backing: `measurable` (RG4 a and b; RG21 for the sloped-surface case)
   - Confidence: high; medium for the sloped-surface case

8. **Source point.** In the firing check, *S* is the firer's reference point, with two
   substitutions:
   - (a) when the weapon has the cell rangefinding property, the horizontal position of *S* is the
     centre of the firer's cell, and its height is the terrain height at that centre (on a slope,
     the height of the sloped surface there, S-0106), plus 416 when the firer is on a bridge deck,
     and
   - (b) when the firer is airborne, the world height of *S* is replaced by the world height of the
     target's reference point as it is, without the replacement of claim 7. When both apply, *S*
     has the horizontal position of (a) and the height of (b). So against a target that is not
     floor-bound, an airborne firer measures no height difference; against a
     floor-bound target the difference Δ*z* is the target's own height above the point that claim 7
     measures to: its height above the terrain at its position, less 416 when it is in a high-bridge
     cell. A floor-bound target under a high bridge therefore gives Δ*z* = −416 for an airborne firer.

   The fire point of S-0313 is not used for the range test.
   - Backing: `measurable` (RG5 and RG21 cover (a), RG2 f covers (b), RG21 c covers both)
   - Confidence: high

### The comparison

9. **Two branches.** With Δ = *S* − *M*_T (integers, leptons):
   - **Minimum range (both branches).** When the weapon's minimum range *m* (leptons, claim 1) is
     not 0, the target is out of range if ⌊*r*(Δ*x*² + Δ*y*² + Δ*z*²)⌋ < *m*.
   - **Projectile not arcing.** The target is in range when ⌊*r*(Δ*x*² + Δ*y*² + Δ*z*²)⌋ ≤
     *R* + *E*₁ + β, where β = 64 × (*W* + *H*) for a building target with a *W* × *H* foundation
     and 0 otherwise (as S-0204 claim 7). The spatial distance is used for every firer, aircraft
     included; the height difference of an airborne firer is the one of claim 8 (b).
   - **Arcing projectile.** The target is in range when the planar distance *d* = ⌊*r*(Δ*x*² +
     Δ*y*²)⌋ ≤ *L* = *R* + *E*₂, no building lengthening applies, and the reachability test of
     S-0306 succeeds for a projectile launched at the arc speed *v*(*L*, *g*) towards a point *d*
     away horizontally and *M*_T.z − *S*.z higher.
   - Backing: `measurable` (scenario RG1 for both comparisons, RG6 for the minimum range, RG7 for
     reachability)
   - Confidence: high

10. **High-bridge refusals.** A target that meets the comparison of claim 9 is still out of range
    in either of two cases:
    - for a projectile that is not arcing, when the source's cell carries a high bridge, *S* lies
      below the deck (*S*.z below that cell's terrain height + 416) and *M*_T lies at or above that
      deck height: a firer under a bridge cannot reach up onto it, and
    - for an arcing projectile, when the cell of *M*_T carries a high bridge and
      *M*_T.z − *S*.z ≥ 312 leptons (three height levels).
    - Backing: `measurable` (scenario RG4, runs c and d)
    - Confidence: medium

11. **Line of fire.** The target is also out of range when the line-of-fire walk of S-0306 from
    *S* to *M*_T, with the weapon's projectile and the firer's house, finds a blocking cell, unless
    the warhead is wall-damaging (Terms) and the blocking cell holds one of the three
    overlay types with registration indices 2, 26 and 243 in the overlay registry (counted from 0,
    S-0004; an engine constant). A blocking cell whose wall overlay has any other index, or a
    cliff cell without one of those overlays, refuses even a wall-damaging warhead. Which wall
    blocks at all (the owner, allied wall transparency) is S-0306's walk. Projectiles
    that are subject to neither cliffs nor walls always pass.
    - Backing: `measurable` (scenario RG8, runs a to c)
    - Confidence: high for the rule; as S-0306 for the walk itself

12. **Who uses the range test.** The firing check uses it with the source point of claim 8.
    Movement rules that look for a position from which to fire use the same comparison with the
    candidate position as source point, without the substitutions of claim 8.
    Claims 15, 16 and 19 to 21 specify covered cases of choosing such a candidate. S-0215 claim 37 uses
    the firing-check source point for the decision whether to search at all, not these candidate
    source points. S-0208 claim 17's ordinary walker stop instead uses claim 8's source point and
    the weapon chosen against the target by S-0308, but tests the target's cell as the measured
    target, not the target object. For an open-topped type that stop uses the separate strict
    three-dimensional distance comparison against claim 13's ρ, as specified there.
    The movement outcomes are S-0208 claims 16 and 17 and S-0207 claim 19, not this inventory.
    The target-scan and order rules that measure distance in other ways are owned by their specs
    (S-0309, S-0214).
    - Backing: `unmeasured` (an inventory of rules; each user measures its own rule)
    - Confidence: medium

### The firer's weapon range

13. **Firer's weapon range ρ(*A*, *i*).** For firer *A* and its weapon in slot *i*: ρ = 0 when *A*
    has no weapon in that slot. Otherwise ρ is that weapon's range field in leptons (*R*₀ of claim
    1: no air bonus, garrison replacement, bunker, open-topped or elevation bonus applies). When
    *A*'s type is open-topped, ρ is the smallest of that value and the range fields of the
    current weapons of *A*'s passengers (glossary, *current weapon*): the weapon-number slot
    for a multi-weapon passenger, slot 0 otherwise. A passenger without a current weapon is
    skipped, and a transport without passengers uses its own value. This is not weapon choice
    against the transport's target: a passenger's current weapon is read regardless of that target. The result is read by the rules that scale
    with the firer's reach: the flak scatter of S-0306 (claim 4, its *R*_W) and the reach used by
    the target-scan and guard rules of S-0309 (its F1), the approach step and search
    (S-0215 F10 and F4 below), and the prism support reach of S-0518 claim 2, which reads
    ρ(*A*, 1) for the master building. The range test of claims 1 to 11 does
    **not** use it: it uses the weapon's own range field, so an open-topped transport whose
    passengers have short weapons still passes the range test at its own weapon's range, while
    those other rules see the shorter value. The passengers' own range tests use their own weapons
    and the bonus of claim 5.
    S-0306 claim 4 specifies this current-weapon rule, including the weapon-number slot for a multi-weapon passenger and skipping a passenger without a current weapon. Its P3 f does not distinguish the current-weapon rule from a primary-only rule, so those passenger cases remain unmeasured there.
    - Backing: `measurable` (RG9 and RG10 for an ordinary armed passenger and the range test's
      non-use; scatter and scan consequences through S-0306 P3 and S-0309's scenarios,
      and prism support reach through S-0518 PS3).
      Backing: `unmeasured` for multi-weapon current-slot selection and missing passenger weapons,
      since RG16 has no established normal-action fixture for obtaining the required states
      without writing game state, and for ρ = 0 with no firer weapon, since no valid firing
      query in these scenarios exposes that result. RG16 is pending, not measurement backing.
    - Confidence: high for the rule; medium for completeness of its listed users

### Degenerate questions

14. **No target, no weapon.** The range test as the firing check asks it answers *in range* when
    there is no target (the firing check's other answers decide, S-0301), and *out of range* when
    there is a target but the firer has no weapon in the slot. In both cases no other claim of this
    spec changes the answer.
    - Backing: `unmeasured` (no scenario of this spec can ask the question without a target or
      without a weapon)
    - Confidence: medium

### Approach destinations

15. **Open-ground approach-position choice (Q-0072, question 2, partial).** Consider a ground
    infantry unit or vehicle for which an approach-position search is requested, with no existing
    destination or approach destinations. The case covered here has flat terrain at world height
    0, no bridges, no obstacles or other occupants in the candidate cells, a target other than a
    cell, no close-range property, and no special crushing or deploying approach. The search is
    around the target's reference point, not around the attacker. Its initial radius and candidate
    points are F4. If qualifying candidates occur at more than one radius, the chosen destination is from the greatest qualifying radius. Among qualifying candidates at that radius, the preference order specified in F4 decides. The chosen destination is the cell of the first qualifying point, not the target
    object itself, and not the point nearest to the attacker among all qualifying points.

    Each point is at a cell centre at world height 0. It must pass the range test of claims 1 to
    11 with that point as source and with the weapon chosen for this target (S-0308). Thus the
    radius uses the firer's weapon range of claim 13, whereas acceptance uses the full range test:
    these are separate tests. A point outside the usable area never qualifies. The cell must also
    be clear for the unit's locomotion class and movement zone and in its current reach region
    (S-0211 and S-0212). In these ground/no-bridge cases, a **clear cell** is enterable by class
    for that class and zone (S-0211 claim 10), has an occupancy record empty as a whole (S-0209),
    and belongs to that region. For locomotion class *winged*, the clear-cell test instead
    accepts without occupancy, region, land or wall tests (claim 21). This is not the actor's neighbouring-cell entry outcome: the
    approach test does not accept a moving blocker or an enemy obstacle as a clear cell.
    On the covered open ground, the coarse route-length estimate is the
    Chebyshev distance in cells and introduces no further rejection. No random draw selects
    between candidates in this covered case.

    This claim specifies the search result only. It neither promises that the unit reaches the
    chosen cell nor says that reaching it is necessary before firing. Path computation and
    movement towards a destination are S-0212, S-0207 and S-0208. S-0215 claim 37 owns the trigger,
    gating, destination retention and renewal, and the building, deployed-infantry and zero-speed
    actor qualifications (part g).
    In particular, a held destination is not cleared by that step just because the target comes
    in range; the movement outcomes belong to S-0207 claim 19 for driving, sailing and hovering
    vehicles, and to S-0208 claims 16 and 17 for ordinary walking infantry.
    - Backing: `measurable` (RG10)
    - Confidence: high for radius, angular order and the separation of the two range tests,
      medium for the covered-case qualification as a whole

16. **Obstructed level-ground choice (Q-0072 question 2, partial).** Claim 15 also applies with
    obstacles or occupants, with these additions. This claim covers ordinary ground infantry and
    vehicles on flat height-0 terrain without bridges, cell targets, close-range types or special
    crushing, deploying or balloon-hover approaches. All primary and alternative cells in this
    claim are legal and inside the usable area; claim 21 extends the alternative-cell scope.

    A primary point is a point of F4. A primary point that fails the range test is skipped; one
    whose cell is not clear is skipped for infantry. For a vehicle, a primary point that passes
    the range test but whose cell is not clear instead offers alternative cell centres at these
    displacements from that primary cell, in this preference order (integer cells):

    > (0, −1), (1, −2), (2, −2), (3, −1), (3, 0), (2, 1), (1, 1), (0, 0).

    These are not the eight neighbours of the primary cell: the search reaches three cells in
    the positive map-x direction and revisits the primary cell last
    (`quirks.approach_alternative_walk`). An alternative must pass the same range and clear-cell
    tests. It is measured at its cell centre at world height 0. Only the first alternative
    passing both tests is offered to the detour limit below. If none passes, the primary point is
    skipped. A qualifying alternative of an earlier-ranked primary is preferred to every candidate of a later-ranked primary, even when the later primary is closer to the attacker.

    A clear primary, or the first qualifying alternative, is accepted only when at least one of
    the following holds, with *c* its cell, *t* the cell of the target's reference point and *a*
    the attacker's cell:

    > E(*c*, *t*) ≤ C(*c*, *t*) + 8 cells, or E(*a*, *c*) ≤ C(*a*, *c*) + 8 cells.

    E is the coarse route-length estimate of S-0212 claim 24 and F8 for this attacker, using the
    ground layer at both ends. C(*p*, *q*) = max(|*p*.x − *q*.x|, |*p*.y − *q*.y|), an exact
    integer cell distance; the addition is integer, without rounding. A failed estimate never
    satisfies its inequality. If the detour limit refuses the first locally
    qualifying alternative, later alternatives of that primary are not considered: the next
    F4 primary has preference. No random draw chooses a primary or alternative. If every primary
    fails, claim 17 applies.
    - Backing: `measurable` (RG11)
    - Confidence: high for the alternative displacements and preference; medium for the detour
      limit and the qualification as a whole

17. **When no approach position is found (Q-0072 question 2, partial).** A failed
    approach-position search has these outcomes, when it is allowed to change the actor's
    destination:

    | Actor | Destination and target |
    |---|---|
    | its type has the hunter-seeker property, or it is vehicle-thief infantry | destination becomes the target itself, and the target is kept |
    | any other actor | target is dropped; destination becomes the cell returned by a nearby-cell search, or is cleared if that search finds no cell |

    There is no unconditional fallback to the target object or its cell for an ordinary actor.
    The nearby-cell result need not be a position from which the weapon can fire, since the
    target has been dropped. The ability of a hunter-seeker or vehicle thief to enter or attack
    the object is outside this claim.

    **Empty-radius case.** When the radii of F4 are empty, the nearby-cell search is centred on
    the cell of the target's reference point. For an ordinary ground infantry unit or vehicle
    on terrain without bridges, it is S-0212 claim 25 with the actor's locomotion class, its own
    movement zone, its current ground reach region, deck mode off, the height condition on, no
    further test and no reference cell. Bridge cells are excluded. The choice is therefore the
    tick-number choice of that claim, not the cell nearest the attacker, and takes no random
    draw.

    **Failure centre after candidates (Q-0172 question 1).** For every search of claims 15,
    16 and 19 to 24, let the *failure anchor* be the last primary in F4's preference
    order that passes the range test (or, for a close-range type, is treated as passing it by
    claim 22 d), whether or not it is usable or clear. If there is none,
    the centre is the target-reference cell, even when the radii were not empty. Otherwise
    the centre is the failure anchor's cell, except when that primary offered an alternative
    that passed both range and clear-cell tests but failed the detour limit: the centre is
    that alternative's cell. If all eight alternatives failed range or clear-cell tests,
    the centre is the primary's cell, not the last alternative examined. Later primaries
    failing the range test do not move this centre.

    Thus an ordinary actor still drops its target and receives the nearby-search result or
    no destination as in the table; non-empty radii do not suppress the nearby search.
    For ground actors without bridges its parameters are the empty-radius parameters above,
    now using this centre. This includes close-range failed approaches in S-0212 claim 25's
    failed-approach row, with that row's other parameters and exclusions unchanged.
    Close-range infantry always have a non-empty radius (claim 22 a), but their failed
    candidate search uses the same nearby-cell parameters. No rule of the failure
    distinguishes close-range types or jumpjets, and the nearby-search parameters for actors
    outside that ground scope are S-0212's.
    - Backing: `measurable` (RG12, RG17 and RG23 f)
    - Confidence: high for keeping or dropping the target and the object-destination exceptions,
      medium for the nearby-search parameters and failure-centre rule

18. **Balloon-hover vehicle with a vertical projectile (Q-0072 question 2).** When a vehicle with
    the balloon-hover property takes the approach step and the weapon chosen for its target has
    a vertical projectile, its destination becomes the target itself, and the target is
    kept. This includes a cell target: the cell becomes its destination (Q-0172 question 7),
    with no approach-position search. It does not search for a firing cell, even when it already has a destination or the
    target is already in range. The weapon and its projectile must both exist. This replaces
    the common approach step of S-0215 claim 37, subject to that claim's earlier crushing
    exception. It takes no random draw. Without either property this exception does not apply.
    - Backing: `measurable` (RG13, including the cell-target run)
    - Confidence: high

19. **Ordinary candidates on terrain and bridges (Q-0172 questions 4 and 5).** For ground
    infantry and vehicles without the close-range property or a special approach, the F4
    primary positions and preference remain the same on raised or sloped terrain and near
    high bridges. Only their height changes: a primary is measured at the terrain surface
    height at its cell centre (S-0106), plus 416 leptons if that cell carries a high bridge.
    This measures the deck even when the attacker is underneath it. A vehicle's alternatives
    retain world height 0, even on raised terrain or in a high-bridge cell
    (`quirks.approach_alternative_zero_height`). Neither height is the attacker's height.

    High-bridge cells can qualify. Their clear-cell test reads the entire deck occupancy
    record, not the ground record beneath it; cells without a high bridge use ground occupancy.
    The test requires the actor's locomotion class, its own movement zone and its current
    reach region, determined from its actual cell and its current ground/deck layer. The
    candidate's reach region is read for its deck (S-0212 claim 4) when its cell carries a high
    bridge, and is its own region otherwise. The candidate qualifies only when that region is
    the actor's. No candidate-height equality is required.
    - **A deck over water (Q-2171).** The ground under such a deck is in no reach region of a
      ground zone, but the candidate is not read for its ground. With an enabled bridge link
      it gets the reach region of the link's first end. A ground vehicle or infantry unit
      standing on a deck has its own region read for its deck too, each cell through the link
      that S-0212 claim 4 picks for that cell. A deck cell qualifies for it when the two reads
      give the same region, as they do when both cells read through enabled links whose first
      ends lie in one reach region. An attacker on a bank qualifies a deck cell when its own
      region is the one that cell's deck read gives. The attacker's kind changes none of this.
      A close-range type takes the same deck reads through claim 22's pointer to this claim.
      That is `unmeasured` here (RG18 and run e use an attacker without the close-range
      property), medium.
    - **A deck read through a link that is not enabled, or with no link found.** The candidate
      and a deck-standing actor then take S-0212 claim 4's other rows for their deck reads, and
      the same equality decides. What a same-deck actor gets there is left open here: this spec
      states no outcome for those rows beyond that equality.
    - **A deck over other ground.** The same read applies to a high-bridge cell whose ground is
      in a reach region. An attacker on the ground under or beside the bridge qualifies such a
      cell only when its region is the deck's, whatever the region of the cell's ground.
    The *winged* exception of claim 21 bypasses all these clear-cell conditions, not the range
    or detour tests. S-0211 claim 10 supplies the clear-cell land and wall tests, with a high-bridge deck
    bypassing the underlying land multiplier. This can accept a deck candidate whose ground
    is occupied, and reject it solely because its deck is occupied.

    The detour inequalities of claim 16 still apply. For E(candidate, target), the candidate
    endpoint is on the deck when its cell carries a high bridge, and the target endpoint is
    on the deck for a deck-standing mobile target or a high-bridge cell target, ground otherwise.
    For E(attacker, candidate), the attacker endpoint uses its movement-layer rule of S-0212
    and the candidate endpoint uses that cell's high-bridge mark. The ground-only qualification
    in claims 15 and 16 is replaced by these rules. The range test's elevation bonus still
    uses the actual firer's cell, not the proposed candidate cell; its line of fire and
    distances use the candidate source point.
    - Backing: `measurable` (RG18; run e for a deck over water, run c for a deck whose ground
      region differs from the deck's). The deck read through a link that is not enabled, or
      with no link found, is `unmeasured` here, as in S-0212 claim 4: no scenario of this spec
      places such a deck. That two cells of one deck give the same region is `measurable` only
      for run e's bridge; no scenario here varies the links of one bridge.
    - Confidence: medium (heights, occupancy layers, detour endpoints, and the deck read through
      an enabled link, the equality of the actor's and candidate's deck reads included); low
      (the deck read through a link that is not enabled or with no link found)

20. **Ordinary cell targets (Q-0172 question 2).** For an ordinary ground attacker without
    the close-range property, a cell target that is not a destroyable cliff uses the same
    search as an object target: T is the cell's reference point, its centre, and β is 0.
    Candidate range tests measure to that cell by claims 7 and 9. Success assigns the chosen
    candidate cell as destination and keeps the cell target. Failure uses claim 17, including
    its failure centre and target-drop rule. This does not imply an infantry early stop:
    S-0208 claim 17 excludes cell targets.
    - Backing: `measurable` (RG19)
    - Confidence: high for ordinary cells; claim 23 gives the destroyable-cliff replacement

21. **Alternatives outside the usable area (Q-0172 question 6; Q-0230).** The usable-area
    test rejects primary cells, but is not applied to a vehicle's alternatives. This omission
    does not make outside cells members of the attacker's reach region.

    For an actor of locomotion class other than *winged*, the clear-cell test still compares
    the candidate's reach region, read as claim 19 gives, with the actor's region of claims 15
    and 19. A legal
    outside-area cell on ordinary flat ground has category *off-map* (S-0211 claim 11) and
    no reach region (S-0212 claims 2 to 4). A stationary actor inside a ground reach region
    therefore refuses that alternative, even when its terrain is open and continuous with
    the actor's terrain. No replacement category based on land or overlay is used, and no
    boundary-specific region exemption exists.

    For locomotion class *winged*, the clear-cell test accepts without testing the region,
    occupancy, height, land multiplier or wall. This applies to primaries and alternatives,
    inside and outside the usable area. It does not bypass the primary usable-area test or
    either candidate's range and detour conditions. Since a range-passing inside primary
    is always clear for a winged vehicle, it never offers alternatives: failure at the
    detour test proceeds to the next primary, not to alternatives. Winged class therefore
    does not supply a way for this actor to select an outside alternative either.

    The missing alternative boundary test is still reproduced as
    `quirks.approach_alternative_outside_area`; selection remains conditional on every other
    test, not guaranteed to have a reachable fixture. The earlier claim that ordinary flat
    outside ground could be in an inside actor's ground reach region is withdrawn. Whether
    bridge or moving-actor states make this omission observable remains open. A chosen
    destination does not promise a path to it.

    Nearby-cell searches still exclude outside-area cells, for *winged* class too
    (S-0212 claim 25). Path computation uses S-0212 claim 6's independent reachability gate:
    a ground-region actor and an ordinary outside-ground destination fail it unless one of
    that claim's three exceptions applies. The *winged* clear-cell exception alone does not
    waive that gate. The nearby and path rules already answer Q-0230 question 3 in S-0212.
    This claim changes neither rule. No result is specified here for an illegal map cell.
    - Backing: `measurable` (RG20; S-0212 P11 for the path gate and P21 for nearby searches).
      `unmeasured` for the missing alternative boundary test itself: no reachable fixture
      distinguishing it from an explicit boundary refusal is established
    - Confidence: high for clear-cell acceptance and refusal; as S-0212 for path outcomes

22. **Close-range searches (Q-0172 question 3).** For an attacker whose type has the
    close-range property, the search of claims 15 to 21 applies with the changes below. Nothing
    else differs: F4's bearing order, the vehicle alternatives of claim 16, the detour limit,
    the heights and layers of claim 19, cell targets (claims 20 and 23) and the failure
    outcomes and centre of claim 17 are as stated there. *T* is the target's reference point.

    a. **Radius (infantry only).** When F4's *u*′ is at most 332 leptons, an infantry attacker
       uses the single radius 332 leptons; a larger *u*′ keeps F4's radii. Because the next
       radius, 76 leptons, is below F4's limit, the search then has exactly one radius. An
       empty-radius case therefore never occurs for close-range infantry, and claim 17's
       empty-radius branch does not apply to them. The original compares *u*′ with 332.8 in
       double precision, which for the integer *u*′ is the same as *u*′ ≤ 332.
    b. **Near-target shortcut (every actor kind).** Before any candidate is examined, let *q* be
       the centre of the cell holding the target's position when the target is a vehicle,
       infantry unit, aircraft or building (for a building, the cell of its position, not of
       its reference point), and *T* for any other target, such as a cell. Let *d*₀ be the planar
       distance, truncated to an integer, from the centre of the attacker's cell to the centre
       of the cell holding *q*, and *d*₁ the same distance from a point *P* to *q*. Only when
       *d*₀ ≤ 384 leptons (the attacker's cell is the target's cell or one of its eight
       neighbours), *P* is the position choice in the attacker's own cell with desired point *T*,
       read from that cell's ground occupancy record whatever surface the attacker is on. The
       attacker's own entry counts as taken there. When a position is found and *d*₁ < *d*₀, the
       search ends successfully at once: an immediate movement towards *P* is requested,
       its destination becomes the cell of *T*, and the target is kept. No candidate is examined.
       For ordinary walking infantry, S-0208 claim 21 owns the second standing-position
       choice, reservation replacement and arrival outcome. The actual walking target can
       differ from *P* on a bridge deck. It is not an unconditional stop at *P*: after arrival,
       S-0208 claims 16 and 17 decide whether the destination is cleared or walking continues.
       These rules already answer Q-0237 questions 1 to 3 in S-0208 claims 3 and 21, including
       replacing an existing walking target and releasing its old reservation immediately.
       Other locomotors' shortcut outcome is claim 25.
       Otherwise the search continues with (c). With *d*₀ = 0 the shortcut never succeeds. A vehicle on the ground carries
       its own vehicle mark in its cell's ground record, so the position choice gives it no
       position and the shortcut does not succeed for it.
    c. **Primary points (infantry only).** Each F4 primary's point is not its cell centre but
       the position choice in that primary cell with desired point *T*, read from the cell's
       ground record. Its height is then that of claim 19: the terrain surface height at the
       chosen point, plus 416 leptons when the cell carries a high bridge. The cell's other
       tests, its failure-anchor cell and the destination are those of the primary cell. When
       the choice gives no position, the point is the map origin (map coordinates 0, 0) at
       the surface height there. That point lies outside the usable area, so it can only
       ever become a failure anchor, whose cell is then the cell at map coordinates (0, 0),
       never a destination. It passes the range test only with a weapon reaching across the map.
    d. **Range bypass (every actor kind).** A primary or vehicle alternative whose point fails
       the range test is treated as passing it when the planar distance from its point to *T*,
       truncated to an integer, is at most 307 leptons (the original compares it with 307.2).
       From then on it is treated as a range-passing candidate in every respect, including claim
       17's failure anchor.
    e. **Own occupancy disregarded (every actor kind).** During each clear-cell test of claims 15,
       16 and 19, an attacker in the ground layer has its own occupancy entry withdrawn and restored right after the
       test. A cell whose only occupant is the attacker can therefore be clear. Any other
       occupant still makes it not clear for a non-winged actor: the whole occupancy record
       must be empty, even though (c) found a free standing position in it. Winged actors
       retain claim 21's clear-cell bypass.
    f. **Random draws.** Every position choice of (b) and (c) takes a scenario-stream draw of S-0208 claim 3 exactly when the desired class is the centre and the table's first row (the refused cell: vehicle-present flag, or object-present flag without an open gate) does not apply. The draw occurs even when every position of the selected row is taken and the choice returns no position. The observable draw order places the shortcut's choice, when its distance condition holds, before draws attributable to primary candidates; primary-candidate draws follow F4's preference order. A primary contributes its draw even if its range test fails, and each occurrence of a repeated cell contributes another draw. No draw attributable to a later primary occurs after the search ends. These same draw results apply when the search may not change the actor. The class of *T* is the centre, for example, for a vehicle standing on a cell centre and for every building, whose reference point lies on a cell centre, edge or corner (S-0208 claim 2).

    When the search may not change the actor (claim 17), the shortcut moves nothing and sets no
    destination, but it still ends the search.
    - Backing: `measurable` (RG23); `unmeasured` for the map-origin fallback and
      the search when it may not change the actor: RG23 establishes neither a range-passing
      map-origin fixture nor a normal-action fixture for the latter query mode
    - Confidence: high for the radius, the shortcut's conditions and constants, the bypass,
      the disregarded own occupancy and the draws; medium for the shortcut's combined movement
      and destination outcome and for the map-origin case

23. **Destroyable-cliff cell targets (Q-0172 question 2).** When the target is a destroyable-cliff
    cell and the search of claims 15 to 22 is about to run, the target is first replaced by one
    cell of the same tile. Let the target cell be (*x*, *y*), with sub-tile *s* of a tile *W*
    cells wide and *H* cells high (sub-tiles counted row by row, *W* per row). The tile's cells
    are taken at (*x*₀ + *i*, *y*₀ + *j*) for 0 ≤ *i* < *W* and 0 ≤ *j* < *H*, where
    *x*₀ = *x* − (*s* mod *W*) and *y*₀ = *y* − ⌊*s* / *W*⌋. A cell qualifies when its part of
    the tile is a tile cell and the cell is inside the usable area. Whether the map cell there
    still holds that tile is not checked. Let *c* be the cell of the attacker's scatter
    reference point. Among qualifying cells the one with the smallest *r*((Δ*x*)² + (Δ*y*)²),
    the table square root of S-0204 of the squared cell-coordinate differences from *c*, is
    chosen. If qualifying cells tie for the smallest distance, the selected cell is the one earliest in row-major order: increasing *j*, then increasing *i*.
    The new target may be the old one. The search then continues with that cell as an
    ordinary cell target (claim 20, or claim 22 for close-range types), keeping it on success
    and dropping it on failure. No random draw is taken. When the search may not change the
    actor, the target is not replaced and the search continues with the original cell.
    When no cell qualifies, the target becomes the cell at map coordinates (0, 0), which is
    reachable only when every tile cell lies outside the usable area.
    - Backing: `measurable` (RG24 for stationary actors); `unmeasured` for the moving
      vehicle's scatter-reference cell, since RG24 does not arrange replacement during
      movement, for the query that may not change the actor, since no normal-action fixture
      invokes that mode, and for the case with no qualifying cell, which needs an attackable
      cliff whose whole tile is outside the usable area
    - Confidence: high for the replacement rule; medium for the scatter-reference cell of a
      moving vehicle; low for the no-qualifying-cell result

24. **No detour limit for jumpjets.** When the attacker is a jumpjet (glossary: moving with
    its jumpjet locomotor at the time of the search), a clear primary, or the first qualifying
    alternative, is accepted without either inequality of claim 16. Within the search this is
    the only rule that distinguishes a jumpjet; its clear-cell tests use its own locomotion
    class, movement zone and reach region as in claims 15 and 19. Whether and when a jumpjet
    takes the search is S-0215 claim 37; its stopping is S-0208.
    - Backing: `measurable` (RG25)
    - Confidence: high for the exemption; medium that nothing else in the search differs

25. **Shortcut for non-walking actors (Q-0237 question 4).** Claim 22 b's choice is not
    restricted to infantry. A vehicle on a high-bridge deck can succeed when its ground
    record supplies a position *P* with *d*₁ < *d*₀: its deck vehicle mark is not read by
    that initial choice. Other mobile actors can likewise succeed when those same conditions
    hold. The search still ends immediately, keeps the target and, when allowed to change
    the actor, assigns the target-reference cell as destination.

    For driving, sailing, hovering, flying, jumpjet, teleporting, tunnelling, rocket and
    drop-pod locomotion, the immediate request towards *P* changes no movement point and
    reserves no standing position. These actors move, if at all, according to their ordinary
    response to the assigned cell destination, not towards *P* as an infantry walking target.
    This is a distinction of locomotor, not of object kind or locomotion class. Ordinary
    walking infantry instead use S-0208 claim 21. No additional random draw belongs to this
    non-walking response; the initial choice still draws as claim 22 f specifies.
    - Backing: `measurable` (RG26 for the deck vehicle and non-walking response).
      `unmeasured` for exercising the shortcut under every other listed locomotor, since
      no normal-action fixture for each is established
    - Confidence: high for the common non-walking response; medium for reachable fixtures

26. **Aircraft classification exceptions.** For an aircraft, *airborne* in this spec means
    the in-the-air answer of S-0501 claim 18. Ordinary aircraft use the 208-lepton height
    threshold in Terms. The two exceptions are the game-wide V3-rocket and dreadnought-missile
    types: their locomotor's own in-the-air answer replaces that height test. S-0506 claim 22
    defines their phase-based answer (parameter sets A and B). For those two types,
    *floor-bound* is the opposite of that answer, not a separate height comparison.
    This qualification applies to the air bonus (claim 3), elevation eligibility (claim 6),
    target-height replacement (claim 7) and source-height replacement (claim 8 b). It is not
    S-0501's on-the-ground or in-flight test.
    - Backing: `unmeasured` for the two exceptional types' range-test consequences: no
      normal-action fixture is established here that isolates their range answers across
      locomotor states from the other targeting and firing refusals. Ordinary height-test
      backing remains RG2 and RG3 as cited by claims 3, 6 and 8
    - Confidence: high for ordinary aircraft's height test; medium for the two exceptional
      types' range-test consequences, which are `unmeasured`

### Airborne targets of floor-bound firers

27. **A floor-bound firer against an airborne target (Q-3069).** Claims 3 and 7 to 9 apply to a
    floor-bound firer, a building in particular, and an airborne target, with no further term:
    - (a) **Full height.** *M*_T keeps the target's world height, because claim 7 replaces it
      only for a floor-bound target. *S* is claim 8's source point: the firer's reference point
      (for a building the centre of its foundation, S-0204 claim 4) at that point's own world
      height, or, for a weapon with the cell rangefinding property, the position and height that
      claim 8 (a) gives. Neither the fire point of S-0313 nor a building's height raises it. So
      Δ*z* is the target's whole height above *S*. Over level ground at the firer's height, for
      an aircraft at its cruise height that is its cruise height (S-0501), and for a jumpjet
      hovering near its jumpjet cruise height (S-0502) its height at that moment, while it is at
      least 208 leptons. A jumpjet hovering lower is floor-bound (Terms), and claim 7 replaces its
      height.
    - (b) **Reach against a cruising flier.** For a projectile that is not arcing, *E*₁ = 0 and
      β = 0 against an airborne target, so the target passes claim 9's comparison only while
      ⌊*r*(Δ*x*² + Δ*y*² + Δ*z*²)⌋ ≤ *R* (claims 9 to 11's other refusals aside). A firer whose
      weapon is not of unlimited range (claim 2) and whose *R* is less than ⌊*r*(Δ*z*²)⌋ never
      reaches a flier at that height above *S*. A firer whose *R* is a little above it reaches
      the flier only within a small planar distance of *S*, about √(*R*² − Δ*z*²). For an arcing
      projectile, claim 9's planar comparison and its reachability test with the full height
      difference apply.
    - (c) **No other reach.** The air bonus of claim 3 is the only term that adds to *R*
      because the target is airborne: claim 4 replaces *R* for a garrisoned building whatever
      the target, and claim 5's additions are for a bunkered firer that is not a building and for
      a passenger, neither of which a building is. Claim 6's bonus, added to claim 9's limits rather than to *R*, is 0.
      A building's shots on its attack paths (S-0215 claim 32, S-0301 claim 14) are made on a
      firing check that answers *can fire*, so it has passed this test, with one exception:
      the secondary shot of a SAM building and the secondary attempt of an upgrade pair ask no
      firing check of their own (S-0301 claim 9). Each follows, in the same processing, the
      primary weapon's check answering *can fire*, so for them the secondary weapon's own range
      is not tested.
    - (d) **Scans.** The target scans of S-0309 use their own distance tests (S-0309 F3) and
      do not change this one, except that with a search range of 0 an armed searcher needs the
      candidate within this test's reach for the weapon it would use (S-0309 F3). A building that
      holds an airborne target, however it was chosen, makes no shot on a firing check asked with
      a weapon for which this test fails. That check answers *out of range* unless a reason of a
      higher rank holds, such as a running rearm countdown (S-0301 claim 9).
    - (e) **Rockets.** A V3-rocket or dreadnought-missile target (S-0506) is measured the same
      way, at its position at that moment. Only whether it counts as airborne or floor-bound
      follows claim 26. While it counts as floor-bound, claim 7's terrain height replaces its
      height, and while it counts as airborne its whole altitude counts.
    - Backing: `measurable` (RG27 runs a and b for parts a and b and part d's absence of a
      shot; RG27 c for the air bonus in part c; RG2 c for the garrison replacement against an
      airborne target). Part d's scan sentence restates S-0309 F3 and is backed there.
      `unmeasured`: part a's statement that neither a building's fire point nor its height raises *S*, because
      RG27 does not fix them so that a raised point would move the reach boundary across the
      tested values of *p*; part b's arcing case, because RG27's projectile is not arcing; part
      c's statement on claim 5, because neither of its states exists for a building, and its
      unchecked secondaries, because RG27's building has one weapon; part d's answer of the
      firing check, because RG27 records shots and targets, not that answer; part e, as claim 26
      states for the two rocket types
    - Confidence: high for part a's full height of an aircraft at its cruise height and of a
      jumpjet hovering at 208 leptons or more over level ground, part b for a projectile that
      is not arcing, part c's air bonus as the only term that adds to *R* for a building that is
      not garrisoned, part c's garrison replacement and part d's absence of a shot (the runs named
      in Backing). Part d's scan sentence at S-0309 F3's own level. Medium for part d's answer of
      the firing check, which follows from S-0301 claim 9's ranks, part a's statement on the fire
      point and the building's height, part b's arcing case, part c's statement on claim 5 and
      its unchecked secondaries, and part e

## Formulas

All integers are leptons unless stated. trunc is truncation toward zero. The original computes
every floating-point step in the round-toward-zero mode (S-0205, `quirks.chop_rounding`); this spec
defines no switch of its own for it. Squares of integer differences are summed in double precision
(exact here) and passed to the table square root of S-0204 F1 (`quirks.table_root_distance`).
Every other sum and product of integers (*R*, the limits, F2) is reduced to the signed 32-bit range
with the wrap function

> *w*(*n*) = the integer congruent to *n* modulo 2³² that lies in [−2³¹, 2³¹ − 1].

*w* changes a value only outside that range, so it matters only for bad data (Edge cases).
This is S-0205 claim 1's shared integer domain, not a range-specific quirk. Unless a quirk
below explicitly supplies another input, the formulas describe the default, original behaviour.

**F1. Effective range** (claims 2 to 6).

When *R*₀ = −512 the target is in range and no other range-test formula applies (claim 2).
This does not bypass the approach-search formulas, whose radius uses ρ, not this limit.
Otherwise:

> *R* = *R*_b + 256 *B*_k · [the firer is bunkered and is not a building] + 256 *B*_o · [the firer is
> a passenger of an open-topped transport]
> *R*_b = 256 × (⌊min(*W*_A, *H*_A)/2⌋ + *G*) when the firer is a garrisoned building, and
> *R*_b = *R*₀ + *a* · [the target is airborne] otherwise
> comparison limit (not arcing) = *R* + *E*₁ + β; limit (arcing) = *L* = *R* + *E*₂.

[·] is 1 when the condition holds and 0 otherwise. *a*, *R*₀ and *m* in leptons (claim 1); *G*,
*B*_k, *B*_o whole cells (integers); ⌊ ⌋ on non-negative integers. Every sum and product is
reduced with *w*.

**F2. Elevation bonuses** (claim 6). Let Δ*h* = max(0, λ_A − λ_T) with λ_A, λ_T the elevation
levels (Terms) of the firer's and target's cells (cells of their reference points, taken by
truncating each lepton coordinate toward zero after dividing by 256). *I* is the elevation increment (integer), *b* the elevation
increment bonus and *c* the elevation bonus cap (both double precision, read as S-0003 decimals).

> *k* = trunc(Δ*h* / *I*) (integer division, *I* ≠ 0; *E*₁ = *E*₂ = 0 when *I* = 0, see Edge cases)
> *f* = RZ₅₃(*k* × *b*); *f*′ = min(*f*, *c*) (the cap)
> *t* = trunc(*f*′) when −2⁶³ ≤ trunc(*f*′) ≤ 2⁶³ − 1, and *t* = −2⁶³ otherwise
> *E*₂ = *w*(256 *t*)
> *σ* = *w*(*E*₂² + (104 Δ*h*)²)
> *E*₁ = ⌊*r*(|*σ*|)⌋

*E*₁ is the table root of the squared hypotenuse of the bonus and the height drop. Without a
bonus (*E*₂ = 0) it is the height drop 104 Δ*h* less the table root's shortfall (S-0204 claim 5).
*σ* is the plain sum whenever that is at most 2³¹ − 1. The height term alone never reaches that
(104 Δ*h* ≤ 26936 for the largest legal height difference, 259 levels), and for |*t*| ≤ 147 the
sum never does either. From |*t*| = 148 a large height difference can push it over, and from
|*t*| = 182 the bonus term alone does. Past that *σ* wraps, so it can be smaller or negative, and
|*σ*| is what the root takes.

**F3. Arc speed and reachability** (claim 9) are S-0306's F2 and F3; claim 9 uses *v*(*L*, *g*)
with *L* from F1 and *g* the projectile's gravity (S-0306 Terms: the game-wide gravity, halved for a
floater projectile).

**F4. Open-ground approach candidates** (claim 15). Let ρ be claim 13's range for the weapon slot
chosen for the target, and let β be the building lengthening of claim 9, or 0 for a non-building.
All radii and coordinates are integers in leptons. The margins are executable constants, not
weapon fields.

> ρ′ = ρ − 128 when ρ ≥ 512 leptons, and ρ′ = ρ otherwise
> u = max(0, trunc(RZ₅₃(ρ′ + β − a)))
> a = 281.6 leptons for infantry, 179.2 leptons for vehicles
> u′ = 256 leptons when 0 < u ≤ 256 leptons, and u′ = u otherwise
> radii = u′, u′ − 256, u′ − 512, …, retaining only radii strictly greater than 204.8 leptons.

ρ′ is S-0215 F10's adjusted reach *r*′. For a deploy-to-fire vehicle standing on a cell where
its deploy test fails, S-0215 claim 37 f2 replaces it with ρ₃ + 512 leptons when that value is
not larger than ρ′, where ρ₃ is the attacker's spatial distance to the target in leptons. That
vehicle also searches when its target is already in range, subject to claim 37's destination
retention and gating. Claims 15 and 16 exclude that exception; this equality of adjusted reaches
is a cross-reference, not a complete deploying search rule.

The margins use the nearest double-precision representations of the decimals shown. Let *f* be S-0204's direction from the target's reference point *T* to the attacker's reference point. The bearing number *b* is S-0204's 256-step facing of *f*, a whole number from 0 to 255. Candidate positions form an ordered sequence: radii follow the radius order above, and within each radius the bearing differences follow this preference order, in 1/256-turn units:

> 0, +1, −1, +2, −2, +3, −3, +4, −4, +5, −5, +6, −6, +8, −8,
> +16, −16, +24, −24, +32, −32, +48, −48, +64, −64.

A repeated candidate cell remains in the sequence at every position where it occurs. For a bearing difference δ, let *p* be the signed representative of 256(*b* + δ) modulo 65536, in [−32768, 32767]. With *c* = −0.00009587672516830327 radians per facing unit, held in double precision, and the table trigonometry of S-0105 F1, the corresponding candidate is determined as follows:

> θ = RZ₅₃((*p* − 16383) × *c*)
> x = trunc(RZ₅₃(T.x + RZ₅₃(cosₜ(θ) × radius)))
> y = trunc(RZ₅₃(T.y − RZ₅₃(sinₜ(θ) × radius)))
> candidate cell = (trunc(x / 256), trunc(y / 256))
> candidate point = (256 × cell.x + 128, 256 × cell.y + 128, 0).

The products and sums in the position formula use double precision in round-toward-zero mode. Integer conversions truncate toward zero. The integer radius arithmetic uses *w* as above. This claim covers valid map coordinates and radii without wrap; malformed-data outcomes of this search are still open.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Approach large-range cut | 512 | leptons | F4, inclusive |
| Approach large-range reduction | 128 | leptons | F4 |
| Approach infantry margin | 281.6 | leptons | F4, nearest double representation |
| Approach vehicle margin | 179.2 | leptons | F4, nearest double representation |
| Approach inner-radius cut | 204.8 | leptons | F4, strict lower bound |
| Approach radius spacing | 256 | leptons | F4 |
| Approach detour allowance | 8 | cells | Claim 16 |
| Hunter-seeker, balloon-hover, vehicle-thief, vertical-flight defaults | no | boolean | Claims 17 and 18 |
| Range field scale | 256 | leptons per cell | Claim 1 |
| "Keep previous value" sentinel | −1 | cells (field value) | Claim 1 |
| Unlimited-range sentinel | −512 | leptons (field −2) | Claim 2 |
| Airborne threshold | 208 | leptons above the surface | Terms; two height levels |
| Bridge deck height | 416 | leptons above the terrain | Claims 7, 8, 10 (glossary) |
| Arcing deck refusal | 312 | leptons | Claim 10; three height levels |
| Building lengthening | 64 | leptons per foundation cell of width plus height | Claim 9, as S-0204 |
| Height level | 104 | leptons | F2 (glossary) |
| Deck lift of an elevation level | 4 | height levels | Terms (claim 6) |
| Wall overlay indices | 2, 26, 243 | registration indices in the overlay registry (from 0) | Claim 11; engine constants |
| Weapon range, minimum range, air bonus | 0 | leptons | Absent-key defaults |
| Occupied-weapon range *G* | 5 | cells | Absent-key default |
| Bunker range bonus *B*_k | 2 | cells | Absent-key default |
| Open-topped range bonus *B*_o | 2 | cells | Absent-key default |
| Elevation increment *I* | 0 | height levels | Absent-key default (see Edge cases) |
| Elevation increment bonus *b* | 1.0 | cells per step | Absent-key default |
| Elevation bonus cap *c* | 0.0 | cells | Absent-key default |

## State and ordering

The range test keeps no state. It reads positions as they are at the moment it is asked, so its
result depends on where the firer and target stand when the firer takes its turn (S-0203).

The rows exclude one another, so exactly one applies to each question.

| Situation | Result |
|---|---|
| no target | in range (claim 14) |
| a target; no weapon in the slot | out of range (claim 14) |
| a target; a weapon with *R*₀ = −512 | in range, whatever the distance, minimum range, bridges and line of fire (claim 2) |
| a target; a weapon with *R*₀ ≠ −512; minimum range *m* ≠ 0 and spatial distance below *m* | out of range (claim 9) |
| a target; a weapon with *R*₀ ≠ −512; not inside a minimum range; projectile not arcing | in range exactly when the spatial distance is at most *R* + *E*₁ + β, the high-bridge refusal of claim 10 does not apply and the line of fire passes (claim 11); otherwise out of range |
| a target; a weapon with *R*₀ ≠ −512; not inside a minimum range; arcing projectile | in range exactly when the planar distance is at most *L*, the target is reachable, the deck refusal of claim 10 does not apply and the line of fire passes (claim 11); otherwise out of range |

## Edge cases and quirks

- **`quirks.approach_alternative_walk`** (default on, the original). A vehicle's obstructed
  primary approach cell offers the displaced cells of claim 16, rather than its eight neighbours.
  Off: the alternatives are the eight neighbours of the primary cell, in step-direction order
  0 to 7, with the same range, clear-cell and detour tests. This changes destination choice only.
  It does not change F4's primary preference or the infantry search.

- **`quirks.approach_alternative_zero_height`** (default on, original). Vehicle alternatives
  are measured at world height 0. Off: use terrain height at the alternative centre plus
  416 leptons when its cell carries a high bridge, just as for primary candidates.
- **`quirks.approach_alternative_outside_area`** (default on, original). Vehicle alternatives
  may be outside the usable area, but still face claim 21's clear-cell and detour conditions.
  For a stationary non-winged actor inside a ground reach region, ordinary outside-ground
  alternatives fail the region test even with this switch on. Off: apply the primary
  usable-area test to alternatives too, including for the winged class.

- **`quirks.garrison_range_drops_air_bonus`** (default on, the original). A garrisoned building's
  range replaces the weapon range together with its air bonus, so the air bonus is lost (claim 4).
  Off: the air bonus is added to the garrison range.
- **`quirks.range_target_on_deck`** (default on). A floor-bound target in a high-bridge cell is
  measured at deck height even below the bridge (claim 7). Off: a floor-bound target is measured
  at the height of the surface it is on (the deck only when it is on the bridge deck, the terrain
  otherwise).
- **`quirks.elevation_deck_cell_level`** (default on). The elevation bonus compares elevation
  levels, which add 4 height levels for every cell with the elevation-deck mark, whether the
  object stands on the deck or under it (claim 6). Thus an elevation-deck-marked cell can
  give a firer under a bridge a bonus against open-ground targets, or reduce the bonus against
  a target under the bridge. Off: the 4 levels
  are added only for an object that is on the bridge deck.
- **Elevation increment 0 (departure, bad data).** With *I* = 0 (its absent-key default, also when
  the elevation section is missing) the original stops with a program error, an integer division
  by zero (S-0205, claim 3), whenever the range test is asked for two floor-bound objects with a
  projectile subject to elevation, whatever their height difference, Δ*h* = 0 included. Our engine takes *E*₁ = *E*₂ = 0 when *I* = 0, and the importer warns when a
  projectile subject to elevation exists while *I* = 0. A negative *I* is not zeroed: it gives the original's integer division, truncating toward zero (*k* = trunc(Δ*h* / *I*) ≤ 0). *E*₂ and *E*₁ then follow F2, including the configured increment bonus *b*, cap *c* and wrap; their signs are not fixed for arbitrary imported values. This behaviour is reproduced, with the same warning. A negative increment is not a separate quirk: the field's domain is signed integers, and this is the ordinary signed division of S-0205 claim 2, confirmed by RG22 a. Only zero needs the safe departure.
- **`quirks.negative_elevation_cap`** (default on, the original; bad data). The cap applies even when Δ*h* = 0 (*k* = 0 and *f* = 0), so *f*′ = *c* for *c* < 0. The resulting values are still calculated by F2, including its signed-32-bit wrap: *t* is obtained by the stated conversion rule, *E*₂ = *w*(256*t*), *σ* = *w*(*E*₂² + (104 Δ*h*)²), and *E*₁ = ⌊*r*(|*σ*|)⌋. When the intermediate products do not wrap and |trunc(*c*)| ≤ 127, *E*₂ = 256 × trunc(*c*) and, at Δ*h* = 0, *E*₁ = |*E*₂| (S-0204 claim 5). This applies to every pair of floor-bound objects with a subject-to-elevation projectile, including a firer on lower ground. The importer warns. Off: use max(0, *c*) in place of *c* in F2, without changing the imported value or any other elevation term. Claim 6 carries the original behaviour's backing and confidence (RG22 b).
- **`quirks.negative_range_fields`** (default on, the original; bad data). Negative range fields, other than the −1 sentinel, whose *R*₀ is not −512, give a negative
  *R*₀ (a field slightly beyond −2, such as −2.001, still truncates to −512 and is unlimited). F1
  and claim 9 apply to it unchanged, so the target is out of range unless the other terms of the
  limit reach the distance; the importer warns. Off: after claim 1's conversion, use
  max(0, value) for a negative weapon range, minimum range or air bonus, except that a
  weapon range converted to −512 remains unlimited. A field read as exactly −1 cells
  still keeps the value in force before this replacement.
  This replacement also supplies the weapon range read by ρ and the approach radius.
  Claim 1 carries the original conversion's backing and confidence (RG1 e and RG2 g).
  Its negative-minimum equivalence is unmeasured for the reason stated there.
- **Overflow in F2 and the limits (bad data, reproduced; RG15).** This is not a separate
  quirk: it is the shared signed-32-bit domain of S-0205 claim 1, applied at the sites
  specified in F1 and F2, rather than a range-specific defect or extra clamp. The topic
  formulas and RG15 identify where it is observable. Saturating these operations would
  change that shared domain; this spec proposes no such alternative.
  The bonus term of F2 can overflow: |*t*| = 182 already wraps its square even at
  Δ*h* = 0. For larger values the exact result is F2, not a monotonic threshold, since
  *E*₂ itself can wrap before squaring. The limits of claim 9 wrap when
  *R* + *E* leaves the 32-bit range. A large positive result needs a cap of 182 or more together
  with a *k* × *b* that reaches it (for example *I* = 1, *b* = 200, *c* = 200, Δ*h* = 1).
  Negative caps can also cause wrap at Δ*h* = 0: for example a cap of −200 gives
  *E*₂ = −51200 leptons, whose square wraps. For larger magnitudes, *E*₂ itself may wrap
  before squaring; no sign or monotonicity beyond F2 is guaranteed. The importer warns when such a projectile exists and |trunc(*c*)| ≥ 148 (a warning that
  can come without an actual overflow).

## Import mapping

| File, section | Key | Concept | Form | Absent |
|---|---|---|---|---|
| rules, weapon section | `Range` | weapon range *R*₀ (claims 1, 2); also the approach radius through ρ in claims 13 and 15 | lepton form of claim 1 | 0 |
| rules, weapon section | `MinimumRange` | minimum range *m* | lepton form of claim 1 | 0 |
| rules, section of an infantry or vehicle type | `CloseRange` | close-range property (claim 22) | boolean (S-0003) | no |
| rules, infantry or vehicle type section | `SpeedType` | locomotion class for approach-cell qualification (claims 15 to 21), including the winged clear-cell bypass, and the failure nearby search | name, as S-0211 Import mapping | S-0211 claim 3: infantry foot; vehicle track when its crusher flag is yes, otherwise wheel |
| rules, infantry or vehicle type section | `MovementZone` | own movement zone and reach region for claims 15 to 21 | name, as S-0211 claim 12 and Import mapping | normal |
| rules, infantry, vehicle or aircraft type section | `Locomotor` | locomotor response to the shortcut's immediate movement request (claims 22 b and 25) | class identifier parsed and recognised as S-0207 claim 14 and Import mapping | keeps value in force; engine default teleporting locomotor |
| rules, weapon section | `CellRangefinding` | cell rangefinding property (claim 8) | boolean (S-0003) | no |
| rules, weapon section | `Projectile` | the projectile type whose properties claims 6, 9 to 11 and 18 use | type name | — |
| rules, section of an infantry or vehicle type | `HunterSeeker` | hunter-seeker property (claim 17) | boolean (S-0003) | no |
| rules, infantry type section | `VehicleThief` | vehicle-thief flag (claim 17; glossary) | boolean | no |
| rules, vehicle type section | `BalloonHover` | balloon-hover property (claim 18) | boolean | no |
| rules, projectile section | `Vertical` | vertical-flight flag (claim 18; S-0306 claim 11) | boolean | no |
| rules, section of an infantry, vehicle, aircraft or building type | `AirRangeBonus` | air bonus *a* (claim 3) | lepton form of claim 1 | 0 |
| rules, projectile section | `SubjectToElevation` | subject to elevation (claim 6) | boolean | no |
| rules, warhead section | `Wall` | wall-damaging warhead (claim 11) | boolean | no |
| rules, section of a transport type | `OpenTopped` | open-topped transport (claim 13) | boolean | no |
| rules, passenger type section | `TurretCount` | above 0 makes the current weapon use the passenger's weapon-number slot (claim 13; S-0308) | integer | 0 |
| rules, passenger type section | `WeaponCount`, numbered `Weapon` and `EliteWeapon` keys | multi-weapon passenger slots read by claim 13 | integer count and weapon type references, with forms and limits of S-0308 Import mapping | count 0, slots empty |
| rules, passenger type section | `Primary`, `Secondary`, `ElitePrimary`, `EliteSecondary` | ordinary and elite passenger slots (claim 13; S-0308) | weapon type names | empty |
| rules, projectile section | `Arcing` | arcing projectile (claim 9) | boolean | no |
| rules, combat-damage section | `OccupyWeaponRange` | *G* (claim 4) | integer (cells) | 5 |
| rules, combat-damage section | `BunkerWeaponRangeBonus` | *B*_k (claim 5) | integer (cells) | 2 |
| rules, combat-damage section | `OpenToppedRangeBonus` | *B*_o (claim 5) | integer (cells) | 2 |
| rules, `[ElevationModel]` | `ElevationIncrement` | *I* (F2) | integer | 0 |
| rules, `[ElevationModel]` | `ElevationIncrementBonus` | *b* (F2) | decimal (S-0003), held in double precision | 1.0 |
| rules, `[ElevationModel]` | `ElevationBonusCap` | *c* (F2) | decimal, double precision | 0.0 |

The combat-damage section is the game-wide section of combat-damage settings (glossary, *game-wide
section*). The three elevation keys are read only when the section exists in a layer; each layer keeps the
values in force for keys it omits. The other keys keep the value in force when a layer omits them
(S-0006).

## Measurement scenarios

All scenarios run under the glossary's *measurement setup*, at game speed index 2, with no engine add-on
and no mods (the named test files of a scenario excepted), and with a named test rules file (a loose copy of
the rules file changing only the fields named) removed after each run. The observable is
the one of S-0204 scenario C2: an attacker ordered to attack a pre-placed, unarmed enemy target
either fires its first shot without leaving its cell ("fires") or moves first ("moves"), watched
for up to 300 ticks; a building attacker that cannot move either fires or does not. Targets belong
to an enemy bystander computer player. The observed area is the attacker, target and tested
candidate cells; the glossary's bystander void rule applies. No run assumes the computer player
remains inactive. Unless stated otherwise the attacker is a
stock turreted vehicle on cell (*x*, *y*) of flat open ground at height level 0, its weapon has no
minimum range and a projectile that is neither arcing nor subject to elevation, cliffs or walls,
and targets stand on cell centres. One run per row; no random draw decides the outcome, except in
RG23, whose predictions use the recorded draw values.

**RG1. Range field and comparisons (claims 1, 2, 9).** Target vehicle on (*x* + 5, *y*) (1280
leptons).
- Run a: range field 5 (1280). Expected: fires.
- Run b: range field 4.99 (single precision 4.98999…, × 256 truncated: 1277). Expected: moves.
- Run b′: range field 4.9999 (1279.97…: 1279 truncated, 1280 rounded). Expected: moves.
- Run c: range field −2, target on (*x* + 40, *y*). Expected: fires. Repeat with a minimum
  range exceeding the recorded target distance and with an intervening enemy wall that blocks
  a control weapon subject to walls with finite range. Expected: still fires; refusal in either
  repeat falsifies the bypass of those range conditions.
- Run d (the −1 sentinel): the test rules file gives the weapon range field 5; the test map's own
  override section for the same weapon gives −1. Target on (*x* + 5, *y*). Expected: fires (the
  value 1280 of the earlier layer is kept). Control: the override section gives 4 instead: moves.
- Run e: range field −2.001, target 40 cells away: fires (conversion still gives the
  unlimited −512 leptons). Control range field −3, with no bonuses and an ordinary nearby
  target: no shot. Firing in the control or refusal in e falsifies the negative-field result.
- Run f (minimum-range conversion): keep the five-cell target and a generous maximum range.
  Set the minimum-range field to 5.003 cells (1280 leptons after truncation).
  Expected: fires at distance 1280. Control minimum 5.004 cells (1281 leptons): no shot.
  Refusal in f or firing in the control falsifies the minimum-range conversion.
- Repeat a and b′ with an arcing projectile, no elevation or cliff property and gravity chosen
  so reachability passes under F3. Expected: a fires, b′ moves; the reverse falsifies the planar
  comparison. Also repeat b with the original non-arcing projectile: firing falsifies its limit.
- Falsified if: run a moves, run b′ fires (rounding instead of truncation), run c moves, or run d
  moves (the −1 was taken as a range).

**RG2. Bonuses and the airborne firer (claims 3 to 5, 8 b).**
- Run a (air bonus): weapon able to fire at aircraft, range field 5 (1280), the firer type's air
  range bonus 1 cell (256). Target: an enemy jumpjet infantry hovering at its cruise height *h*
  (read by the recorder) at planar distance *p* chosen so that 1280 < ⌊*r*(*p*² + *h*²)⌋ ≤ 1536.
  Expected: fires; control run with the bonus 0: moves.
- Run b (garrison): a named test occupiable building with a 2 × 3 foundation supplied by the test fixture (⌊min/2⌋ = 1) and one named test occupant whose weapon is set to fire at ground units, occupied-weapon range set to 3 cells (limit 1024 leptons).
  Targets: a vehicle 3 cells from the foundation centre along a map axis (fires) and one 5 cells
  away (does not fire, although the occupant's own weapon range is set to 8 cells).
- Run c (garrison, air): as b with an occupant weapon able to fire at aircraft and the building
  type's air range bonus set to 2 cells; target a hovering jumpjet infantry at spatial distance
  between 1024 and 1536. Expected: does not fire (bonus dropped).
- Run d (bunker): a vehicle in a stock tank bunker, bunker bonus 2, range field 3 (limit 1280): targets
  at 5 cells (fires) and 6 cells (does not).
- Run e (open-topped): infantry passenger of a stock open-topped transport standing still,
  open-topped bonus 2, range field 3: same targets and expectations as d.
- Run f (airborne firer, claim 8 b): flat map with a high bridge along the *y* axis through column
  *x* + 3. Attacker: a jumpjet infantry whose weapon can fire at ground units, hovering
  at its jumpjet cruise height on (*x*, *y*) (not under the bridge); the recorder reads its exact position
  and so the planar distance *p* to the target. Target: an unarmed vehicle on (*x* + 3, *y*), under
  the bridge. The test file gives the weapon a range field whose lepton value *Q* satisfies
  *p* + 20 ≤ *Q* ≤ ⌊*r*(*p*² + 416²)⌋ − 20 (about 3.2 cells; the window is at least 40 leptons
  wide because *p* stays below 900), and the run is repeated with that file. Expected: moves, because
  Δ*z* = −416 for a floor-bound target under a high bridge (claim 8 b). Control: the same attacker
  and range, the target on open ground at the same planar distance: fires (Δ*z* = 0).
- Run g (air-bonus conversion): use run a's eligible airborne target and record its
  integer spatial distance D. Set the weapon range to (D − 128)/256 cells, exactly
  representable in the domain of claim 1. Air bonus 0.4999 cells converts to 127 leptons:
  expected moves. Control bonus 0.5 cells converts to 128 leptons: fires. For the negative
  bonus run, set weapon range to (D + 64)/256 cells and air bonus −0.5 cells:
  expected moves (limit D − 64). Control bonus 0: fires. One run per setting.
  Firing in either refused run or movement in either control falsifies this part.
  Use D well below 2²⁴ leptons so the weapon fields remain exactly representable.
- Falsified if: run a's control fires or run a moves, run b fires at 5 cells, run c fires, d/e
  are off by a whole cell, or run f fires (an airborne firer would then measure no height
  difference to a target under a bridge) or its control moves.

**RG3. Elevation (claim 6, F2).** A map with a plateau 4 levels high whose edge cell holds the
attacker, and a plain of level 0 beyond it. Test file: elevation increment 1, bonus 0.5, cap 1.0,
range field 4 (1024). Projectiles not subject to cliffs. *k* = 4, *f*′ = 1.0, *E*₂ = 256,
*E*₁ = ⌊*r*(256² + 416²)⌋ = 488 (the exact root is 488.46; S-0204 F1 gives 488); the limit is 1512.
- Targets: (*x* + 3, *y*), (*x* + 4, *y*), (*x* + 5, *y*), (*x* + 6, *y*), (*x* + 5, *y* + 1).
- Straight projectile subject to elevation (limit 1512 on the spatial distance with the 416 drop):
  fires at 3, 4, 5 and (5, 1) (spatial 1370), not at 6 (spatial 1591).
- Control, the same projectile without the property (limit 1024): fires at 3 (spatial 873) only.
- Arcing projectile subject to elevation (planar limit 1280): fires at 3, 4, 5, not at (5, 1)
  (planar 1305) or 6.
- Run d (distinct bridge marks): flat level-0 map with an intact high bridge. Place the
  attacker on the ground under an overlay's own cell, with the straight weapon subject to
  elevation (same test file), and an open-ground target 5 cells away across the bridge axis.
  Expected: fires (λ_A = 4, spatial distance 1280, limit 1512). Repeat from the neighbouring
  marked bridge cells that have no high-bridge overlay of their own or prior elevation-deck
  marking, keeping the same target displacement. These include a cell with S-0212's deck-entry
  mark and the fourth row without that mark. Expected: moves in both (λ_A = 0, limit 1024).
  Open-ground control at the same displacement: moves. Record the overlay cells, their marking
  directions and the attacker/target positions, not merely whether they lie under a deck.
  Repeat after destroying the overlay, with the ground attacker still at its own cell:
  expected moves. Rebuilding and overlapping markings are not covered by this run.
- Run e (floor-bound gate): repeat a distinguishing range boundary from the raised-attacker
  run with a hovering jumpjet attacker, then with a hovering jumpjet target. Use eligible
  weapons and recorded source/measured points. In each run choose a range strictly between
  the predicted limits with and without E. Expected: the result with E = 0, for both projectile
  kinds. Repeat with both objects floor-bound at equal height, then with the firer lower:
  expected E = 0 with the non-negative cap. One run per case.
- Falsified if: the straight weapon fails at (5, 1) or the arcing one fires there, the control
  fires at 4, any run-d result differs, or run e needs a nonzero E to explain its outcome.
  In particular, a bonus on a neighbouring deck-entry cell falsifies the stated distinction
  in its observable use.

**RG4. Bridges (claims 7, 10).** A high bridge running along the map *y* axis over flat land.
Range field 2 (512).
- Run a: target vehicle under the bridge, attacker on the ground 1 cell away across the bridge's
  axis (spatial distance to deck height ⌊*r*(256² + 416²)⌋ = 488 ≤ 512): fires; with the
  attacker 2 cells away (⌊*r*(512² + 416²)⌋ = 659 > 512 although 512 ≤ 512 at ground level):
  moves.
- Run b: same at 2 cells with the target on open ground: fires.
- Run c: attacker under the bridge, target on the deck 1 cell along the bridge: does not fire.
- Run d: test file gravity 10 (as RG7) and a range field for the arcing weapon whose lepton value
  *Q* is large enough that S-0306 F3, with *v*(*Q*, 10) and *d* = 256, is reachable for a height
  difference of 416 (the range field 2 of the other runs is not: RG7 shows that it reaches only
  250 at that distance, so reachability alone would refuse). Arcing attacker (projectile not
  subject to elevation, cliffs or walls) on the ground beside the bridge, target on the deck 1
  cell away in plan: does not fire (deck 416 above the ground, and 416 ≥ 312). Control (i): the
  same attacker on a cell 2 levels above that ground (a plateau beside the bridge, 208 below the
  deck, 208 < 312): fires. Control (ii): the same attacker and test file, no bridge, target on a
  cliff top 4 levels (416) higher, 1 cell away: fires (the height difference alone is reachable,
  so run d's refusal is the deck rule). A cell lower than the ground would only increase the
  difference and is still refused.
- Falsified if: run a fires at 2 cells (target measured at its own height), run c fires, or run d
  fires at the deck while control (ii) fires. If control (ii) does not fire, *Q* was too small
  and the run says nothing about claim 10.

**RG5. Cell rangefinding (claim 8 a).** Infantry attacker in a corner sub-cell position (its
displacement from the cell centre is read by the recorder; S-0208 gives the sub-cell positions), a
target infantry placed so that its distance from the attacker's cell centre is at most the range
while its distance from the attacker's own position exceeds it by at least 20 leptons.
- Expected: fires with the cell rangefinding property, moves without.
- Falsified if: the two runs give the same outcome.

**RG6. Minimum range (claim 9).** Arcing stock weapon, minimum range field set to 2 (512), range 6.
Targets at (*x* + 1, *y*) and (*x* + 3, *y*).
- Expected: no shot at 1 cell (the attacker moves away or stays without firing), fires at 3.
- Repeat both targets with a non-arcing projectile: same expected results. Then put the
  one-cell target 4 height levels above the firer, with no bridge, elevation or cliff property.
  Its spatial distance is 488 leptons. Set the minimum range to 450 leptons through the test
  data and retain a generous maximum range. Expected: fires, although planar distance 256 is
  below the minimum. Repeat with an arcing projectile and gravity/range that pass F3.
- Falsified if: either kind fires at the flat one-cell target, refuses the three-cell target,
  or refuses the raised target while all other firing conditions pass.

**RG7. Reachability (claim 9, S-0306 F3).** Test file: gravity 10, range field 2 (512); an arcing
projectile not subject to cliffs or elevation. *v*(512, 10) = 78, and F3 with *d* = 256 is
reachable exactly for a height difference of at most 250 leptons.
- Run a: target on a cliff top 2 levels (208) higher, 1 cell away: fires.
- Run b: target 3 levels (312) higher, 1 cell away: does not fire.
- Falsified if: run b fires, or run a does not.

**RG8. Line of fire (claim 11).** Straight weapon whose projectile is subject to walls and not to
cliffs; an enemy wall cell between attacker and target 3 cells away on level ground; range 4. The
recorder reads the registration index of the wall's overlay type.
- Run a: the wall overlay type has index 2, 26 or 243 and the warhead is not wall-damaging:
  moves (or does not fire).
- Run b: same wall, wall-damaging warhead: fires (the exception of claim 11).
- Run c: a wall overlay type with the wall property at any other index (a stock one if the
  overlay registry has one; otherwise the test rules file registers a copy of a stock wall type at the end
  of the overlay list, which gives it an index beyond the stock ones), wall-damaging warhead:
  moves (the wall refuses even a wall-damaging warhead).
- Falsified if: run a fires, run b does not, or run c fires. Walls of an ally with allied wall
  transparency are S-0306's (claim 22, scenario P15).

**RG9. Firer's weapon range (claim 13).** An open-topped transport with its own weapon of range
field 6 (1536) carrying one infantry passenger whose current weapon has range field 3 (768); no
bonuses beyond the defaults. Target vehicle on (*x* + 5, *y*) (1280 leptons), ordered to be
attacked by the transport itself.
- Expected: the transport fires (the range test uses its own 1536). Control run with the passenger
  removed: fires. The passenger-reach consequences are measured through S-0306 P3 (flak scatter
  scaled by 768 against 1536) and the scan scenarios of S-0309.
- Falsified if: the loaded transport moves first (the range test would then use the shortest
  range).

**RG10. Open-ground approach destination (claim 15, Q-0072 question 2).** Use a tiny flat
level-0 map, game speed index 2, one human house's attacker and one unarmed enemy target. Include
one bystander computer player kept outside the observed strip; the glossary's void rule applies.
No engine add-on and no mods (the named test files of a scenario excepted). Use named test data to give one ordinary ground infantry type and one
ordinary turreted vehicle type a weapon with range 5 cells, zero minimum range, no elevation,
cliff or wall properties, and no close-range property. The target is on a cell centre ten cells
along a map axis from the attacker. Keep every F4 candidate cell empty and inside the usable area.

- Procedure: issue an attack order, then record the first non-empty destination, the selected
  weapon slot, reference points, tick number, and scenario random state each tick. Repeat on
  eight compass bearings, one run per bearing and attacker kind. Repeat with a larger-foundation
  building target, and with a named test open-topped vehicle whose passenger weapon has a shorter
  range than the vehicle's own weapon. Distinguish passenger shots from vehicle shots.
- Expected: recorded destination is the first cell satisfying F4 and claim 15, using the recorded
  positions and selected weapon. With no passenger and a non-building target, the first shell
  radius is 870 leptons for infantry and 972 leptons for the vehicle. Subsequent shells, if needed,
  are 256 leptons closer to the target. The building adds β before the margin; the loaded
  transport uses the shorter ρ for the shell radius but its own weapon for the acceptance test.
  The destination is a cell, not the target object. The search itself takes no scenario draw.
  S-0215's behaviour delay may still draw in that tick and must be accounted for separately.
- Falsifier: any other first destination when the setup conditions hold, selection by nearest
  attacker distance instead of the stated preference, or a search-specific random draw. A run in
  which no approach search is requested tests S-0215 claim 37's trigger and gating, not claim 15.
The observation must distinguish a cell destination from an object destination; if it cannot, add that distinction to the recorded observations before running this test.

**RG11. Obstructed approach preference and detour limit (claim 16).** Use RG10's map, houses,
speed and named test data. The observed area includes every F4 primary and alternative and the
paths between attacker, candidate and target. The recorder reads the first destination, reference
points, tick, selected weapon, occupancy and random state. The attacker is neither a crusher nor a
special approach type. Place immobile blockers on selected cells, leaving all candidate cells
legal and inside the usable area.
- Run a: block the first primary and leave its first alternative clear and in range. Order the
  vehicle to attack. Expected: that alternative is its first destination. Repeat with infantry:
  expected, it skips the blocked primary and uses the next qualifying F4 primary, not the vehicle's
  alternative.
- Run b: block the first primary and the first three displaced cells, leaving the fourth clear,
  in range and passing the detour limit. Expected: the vehicle chooses displacement (3, −1), not
  a member of the primary's immediate neighbour ring. Repeat with each of the seven first
  displaced cells blocked in turn so that the first clear alternative has a different preference
  position. Evaluate repeated primary cells from F4 without removing them.
- Run c: a purpose-made wall maze with an accepted-range clear primary whose estimate to the
  target exceeds its Chebyshev distance by more than 8, but whose estimate from the attacker does
  not. Expected: accepted. Repeat with both estimates exceeding their respective distances by
  more than 8: rejected in favour of the next qualifying primary. Repeat with the first clear
  alternative failing both limits and a later alternative passing them: expected, that later
  alternative of the same primary is not chosen. A maze whose recorded terrain does not give
  these inequalities under S-0212 F8 is not a valid run of c.
- Runs: one per setup and attacker kind. Expected: the selected cell agrees with claim 16,
  without a search-specific random draw. Account separately for S-0215's processing-delay draw.
  Falsified by a different preference, an infantry alternative, acceptance when both estimates
  fail, or an additional search draw. Path length travelled afterwards is not this test's
  observable.

**RG12. Failed search and empty radius (claim 17).** Use RG10's flat map, houses and speed, with
all test cells well inside the usable area and no bridges. The named test file gives an ordinary
infantry type a weapon range of 1 cell, and an ordinary vehicle type a weapon range of 0, with
zero minimum range and no close-range, special crushing, deploying or balloon-hover property.
Each attacks an unarmed enemy vehicle ten cells away. These values make F4's radii empty. Keep the
target cell occupied and its eight neighbouring cells empty, enterable and in the attacker's reach
region. The observed area is those cells and the attacker's approach strip.
- Procedure: issue an attack order and record, on the tick the approach step first runs, the target,
  destination kind and cell, tick number and random state. One run per attacker kind. Repeat the
  infantry run with the vehicle-thief flag, and each run with the hunter-seeker property, using
  named test data removed after the run. Record the step before any subsequent movement or
  target acquisition obscures its result.
- Expected for ordinary types: target cleared, destination the nearby-cell result centred on
  the target cell, with the eight adjacent cells in S-0212 claim 25's scan order and the current
  tick selecting position *N* mod 8. Not the cell nearest the attacker unless that is the same
  result. No search-specific random draw. For a vehicle thief or hunter-seeker: target kept,
  destination the target object, not a cell. Repeat the ordinary runs with every cell in the
  nearby search's maximum radius blocked or outside the attacker's reach region: expected,
  target and destination both cleared.
- Falsifier: ordinary target retained, ordinary destination forced to the target object or its
  occupied cell, a different cell from the nearby rule, an object-destination exception losing
  its target, or a failure leaving a destination when no nearby cell qualifies.

**RG13. Vertical balloon-hover destination (claim 18).** Use RG10's map, houses and speed. The
named test file gives one ordinary non-crushing vehicle the balloon-hover property and a weapon
with a vertical projectile, no minimum range and range 5 cells. Its target is an unarmed enemy
vehicle ten cells away. The observed area contains the pair and all possible approach cells.
- Procedure: attack the target and record target, destination kind and identity, reference points,
  selected weapon and projectile flags each tick through two approach processings. Repeat with
  the target initially two cells away (in range). Then repeat both distances with the balloon-hover
  property off, and with only the vertical-flight flag off. One run per setup. A further run
  moves the target into range after the first step while the attacker still has a destination.
- Expected: with both properties the destination is the target object in all three situations,
  not a firing cell, and the target is kept; this exception takes no random draw. With either
  property off, it does not apply: at ten cells the common search of claim 15 applies, and at two
  cells an ordinary in-playfield actor without a destination gets none (S-0215 claim 37 f).
- Cell-target run (Q-0172 question 7): repeat the far-target run with a force-attack order
  on an empty ordinary ground cell. Expected: that exact cell is the destination and remains
  the target, without a search draw; repeat with a destination already held.
- Falsifier: a different destination or no destination with both properties and a cell target,
  a cell destination or no destination with both properties and an object target, an object
  destination under the common search in either control, or a search-specific random draw.

**RG14. Absent elevation section and zero increment (claim 6, Edge cases).** Use named rules test data with the elevation section absent in every layer and a projectile whose subject-to-elevation property is on. Place a floor-bound firer and target on the map and issue an attack that invokes the range test. Record whether the original reports a program error/stops or completes the test. Expected: the original stops with the integer-division-by-zero program error described in Edge cases. Falsified if it completes the range test without that error. Repeat with an explicit zero increment if the section-absence case is not equivalent. This measurement observes the original's failure; it does not validate the safe result of our engine's departure.

**RG15. Signed-32-bit overflow in F2 (claim 6, F2).** Use a named test map with a firer one height level above a floor-bound target, and named rules test data setting *I* = 1 height level, *b* = 200 cells per step, *c* = 200 cells, and weapon range *R*₀ = 100 leptons, with other range adjustments disabled. Put the target at a recorded spatial distance of 45000 leptons; arrange a clear line of fire and an eligible target. The stated F2 gives *k* = 1, *E*₂ = 51200 leptons and a wrapped *σ* of −1673516480 square leptons, so the comparison uses *E*₁ = ⌊*r*(1673516480)⌋ = 40908 leptons and a limit of 41008 leptons. Expected: the target is out of range and the attacker moves before firing. Falsified if it fires within range under these inputs. A control with a cap/increment combination that does not overflow must produce the result predicted by that control's F2 value. Record the input values and first-fire/first-movement outcome.
- Limit-wrap variant: a bunkered vehicle on flat ground, non-arcing projectile without
  elevation, weapon range field 8 388 607 cells (2147483392 leptons), bunker bonus 2 cells,
  and an eligible target 5 cells away. Expected: no shot, because the effective limit wraps
  to −2147483392 leptons. Control with the bunker bonus 0: fires. Firing in the wrapped
  run or refusal in the control falsifies that variant. One run each, recording both fields
  and target distance; this tests limit wrap separately from F2's squared-sum wrap.

**RG16. Current passenger weapon, not primary (claim 13; pending, unmeasured).** Use RG10's map, houses,
speed and recorder, with no engine add-on and no mods (the named test files of a scenario excepted). Named test map data gives an open-topped
vehicle its own range of 8 cells and a multi-weapon infantry passenger two slots, of ranges
3 and 6 cells. The passenger's current weapon-number slot must be recorded, not inferred
from its target. Observe the first approach destination for an enemy object ten cells away.
- Procedure: obtain one loaded run with current slot 0 and one with current slot 1 through
  normal game actions; if the fixture cannot obtain both states without writing game state,
  the comparison remains pending. Repeat with the passenger's current slot empty. One run
  per state. Record the transport and passenger weapon slots, imported ranges and destination.
- Expected: F4 uses ρ = 768 and 1536 leptons respectively, not 768 for both. An empty current
  slot contributes nothing, so the transport uses its own 2048 leptons. Candidate acceptance
  still uses the transport's chosen weapon. A primary-only implementation gives a different
  radius and destination in the slot-1 run. Any different ρ consequence falsifies claim 13.

**RG17. Failure centre after trials (claim 17, Q-0172 question 1).** Use RG11's flat map,
houses, speed and recorder. Named test data changes only weapon range, minimum range and
obstacle placements. Before running, calculate every F4 primary and the claim-16 alternatives.
Record the target, first resulting destination, tick and occupancy on the failure tick.
- Run a: non-empty radii, every primary fails range (a minimum range larger than every
  candidate distance is one fixture). Keep nearby cells clear. Expected: nearby search at the
  target-reference cell, target dropped, destination selected by S-0212's tick-number rule.
- Run b: infantry, range-passing primaries blocked or outside the usable area, with no successful
  primary. Leave reachable clear nearby cells around the last range-passing primary, different
  from those around the target. Expected: the nearby result centred on that primary.
- Run c: vehicle, every alternative of that last primary fails range or clearance. Expected:
  centre at that primary, not the eighth alternative. Run d: its first range-and-clear-passing
  alternative fails both detour inequalities and no later primary passes range. Expected:
  centre at that alternative. A fixture must establish those inequalities under S-0212 F8.
- One run per fixture, repeated at two tick residues giving different nearby choices. Record
  the destination before reacquisition can hide the drop. A missing destination while a nearby
  cell qualifies, or any centre-dependent result different from these predictions, falsifies
  the claim. A search-specific random draw also falsifies it.

**RG18. Candidate heights and bridge layers (claim 19).** Use RG11's houses, speed and
recorder on a named map with a raised plateau, slopes and an intact high bridge. Observe only
one non-close-range attacker and its unarmed enemy object; the bystander void rule applies.
Named test data disables projectile elevation, walls and cliffs, and sets range so that
terrain-height and zero-height source tests differ at a candidate. Record both reference
points, terrain heights, candidate occupancy layers, chosen slot and first destination.
- Run a: empty raised and slope primary cells. Expected: F4's first qualifying primary uses
  its centre's surface height, not 0. Construct a recorded range boundary that rejects one
  height and accepts the other; identical predictions are not a distinguishing run.
- Run b: obstruct a vehicle primary and leave one alternative clear on raised terrain.
  Expected: its acceptance uses height 0. Repeat at a bridge cell: still height 0 for the
  alternative. With the zero-height quirk off in our engine, the alternate prediction uses
  the cell surface or deck height.
- Run c: make a high-bridge primary the first qualifying one. Put an immobile occupant
  underneath, leaving the deck empty. Expected: it can qualify; reverse occupancy, ground
  empty and deck occupied: rejected. Keep the actor's current reach region equal to the
  candidate's deck read (claim 19). Then use a high bridge over land whose ground under the
  deck lies in a reach region other than the banks', and put the attacker on that ground, so
  that every primary that passes the range test is a deck cell. Expected: those deck cells are
  rejected (the deck read gives the banks' region). Falsified by: a deck cell chosen, which a
  ground-region read would predict.
- Run d: enable elevation on the plateau fixture, with a floor-bound attacker and target.
  Arrange a primary at a different elevation level from the attacker's cell and choose the
  range so the primary passes with the bonus based on the actual firer's cell but fails with
  the bonus based on the candidate's cell, or vice versa. Expected: the actual-firer-cell
  prediction. Repeat with elevation disabled as a control and compute the new destination.
  Fixtures giving the same destination under both hypotheses do not test this part.
- Run e (Q-2171): a map with an enabled high bridge over water, of at least 8 deck cells, named
  in the record. Stand an unarmed enemy vehicle in the middle of the deck, and give the attacker
  a test range such that every F4 primary that passes the range test is a deck cell. Order an
  ordinary ground vehicle on the same deck, out of range, to attack it. Record whether it keeps
  its target and its first destination. Expected: it keeps the target, and its first
  destination is the first qualifying deck primary in F4's order. Judge target dropping only
  when the recorded inputs establish at least one primary that satisfies claim 19's
  enabled-link region, clearance and detour conditions (claim 16); otherwise the run is
  inconclusive for this contrast. Under that condition, falsified by: the target dropped at
  once, which a ground-region read of the deck cells predicts. Repeat with the
  attacker on a bank cell next to an end of the bridge, and with an infantry attacker on the
  deck. Expected: a deck primary qualifies in each, provided the bank lies in the reach
  region the deck read gives (S-0212 claim 4); record which bank was used.
- One run per fixture and ground attacker kind where applicable. Check the two detour tests
  with the endpoint layers of claim 19. A different selected cell under the recorded inputs
  falsifies the corresponding height, occupancy or region rule. Travelling to the destination
  afterwards is not this observable.

**RG19. Ordinary cell target (claim 20).** Use RG10's map, houses, speed and recorder, with
an empty ordinary terrain cell ten cells away and no destroyable cliffs. Named test data
makes one infantry type and one vehicle type able to force-attack it and gives each range
5 cells, no minimum range and no close-range property.
- Procedure: force-attack the cell. Record the target kind and cell, selected weapon and
  first destination. Repeat with obstacles forcing alternatives, then with every primary
  failing range while nearby cells remain clear. One run per actor and fixture.
- Expected: success keeps that cell target and chooses F4/claim-16's destination with β = 0.
  Failure drops the target and uses claim 17's nearby result. Infantry does not clear its
  destination merely because the cell comes in range. Any different target/destination pair
  falsifies the search rule; a later infantry stop tests S-0208 instead.

**RG20. Outside-area clearance and the winged exception (claim 21; Q-0230).** Use RG11's
houses, speed and recorder on a named flat map with legal cells beyond its usable area.
No engine add-on and no mods (the named test files of a scenario excepted). Keep the bystander away. Observe
one attacker, its unarmed enemy target, and all F4 cells. Record the range results, cell
categories, reach-region comparisons, occupancy, locomotor, class and first destination.
- Run a: ordinary wheeled ground vehicle inside a ground reach region. Arrange a blocked
  range-passing primary inside the boundary whose first alternative is empty open terrain
  just outside. Other candidates fail. Expected: the outside alternative is refused by
  its off-map reach-region result. Control with the alternative inside and passing the
  detour test: it is selected. Acceptance of the outside alternative falsifies this part.
- Run b: named test data gives a jumpjet vehicle the winged class and jumpjet locomotor,
  without close-range or special crushing, deploying or balloon-hover properties. F4's
  first primary inside the usable area fails range; its next primary lies outside and
  passes range. Both are skipped. A later inside primary also passes range. Expected:
  that later primary is selected even if occupied and in a different ground reach region:
  winged clearance never fails, so the vehicle does not try alternatives. Repeat with
  the actor's class changed to wheel and a blocked inside primary whose first alternative
  is outside. Expected: the outside alternative fails clearance. One run per fixture.
- Because winged clearance always accepts a range-passing inside primary, a winged vehicle
  never needs its alternatives. Thus no covered stationary actor inside a ground reach
  region can select an ordinary outside-ground alternative: non-winged alternatives fail
  clearance, and winged actors select the primary instead. The switch can be inert for
  these fixtures despite the missing alternative boundary test.
- Path/nearby controls: S-0212 P11 tests an ordinary outside-ground destination's independent
  path gate and its exceptions. P21 tests nearby searches, which retain their usable-area
  requirement even for winged class. Do not infer path reachability from an approach
  destination or ordinary move-click acceptance. A different recorded destination under
  the computed candidates falsifies the corresponding rule. Illegal cells are not tested.

**RG21. Sloped measured points and source substitutions (claims 7 and 8).** Use RG10's
houses, speed and recorder on a named map with slopes and no bridges, one unarmed enemy
vehicle target and one infantry attacker. Named test data disables minimum range, air bonus,
elevation, cliff and wall effects and uses a non-arcing projectile. Record exact reference
points, cell centres and surface heights from S-0106. Choose each range boundary before the
attack, and observe the first shot versus first movement over 300 ticks, one run per fixture.
- Run a: place the floor-bound target on a slope at a point whose sloped-surface height differs
  from its cell's base height. Set the range Q between the two predicted spatial distances,
  so only one passes. Expected: claim 7's sloped-surface prediction, not the base-height one.
- Run b: put the infantry firer at an off-centre standing position on a slope, with cell
  rangefinding enabled. Compute the distance using the centre's surface height and a control
  distance using the height at the firer's standing position, but the same centre x and y.
  Choose Q between them. Expected: the centre-height prediction. Repeat with cell rangefinding
  disabled and compute the expected result from the unmodified reference point.
- Run c: enable cell rangefinding on an airborne jumpjet infantry firer attacking an airborne
  jumpjet target. The target's recorded world height must differ enough from the source-cell
  surface height to distinguish the results. Choose Q between the planar distance from the
  source-cell centre and the spatial distance from that centre at surface height to the target.
  Expected: fires, because substitution (b) supplies target height after substitution (a).
- Every Q is in leptons and is imported by an exactly representable cells value Q/256. Reject
  fixtures whose predicted distances are equal or whose other firing conditions refuse.
  Any first-shot/first-movement result opposite to the stated prediction falsifies that part.

**RG22. Negative elevation inputs (claim 6 and F2 edge cases).** Use RG3's plateau, houses,
speed and recorder, with an arcing weapon of range 8 cells, no minimum, cliff or wall property,
and a target on the plain 5 cells away. Set gravity so both candidate limits pass F3 whenever
planar distance passes. Record positions and first-shot/first-movement outcome, one run each.
- Run a: I = −1 height level, b = 1 cell per step, c = 10 cells, height difference 4 levels.
  Expected: k = −4, E₂ = −1024 leptons, limit 1024 leptons, so moves. Control I = 1:
  E₂ = 1024 and limit 3072 leptons, so fires. A shot in a or movement in the control falsifies
  the signed-increment rule. A zeroing of negative I would instead permit the shot.
- Run b: equal-height floor-bound objects, I = 1, c = −1 cell, b = 1 cell per step, target
  still 5 cells away, and weapon range now 5 cells. Expected for the arcing projectile:
  E₂ = −256, limit 1024, so moves. Repeat with a non-arcing projectile: E₁ = 256, limit
  1536, so fires. Control c = 0: both fire. A different result falsifies the negative-cap
  rule or its different use by the two comparisons. Repeat b with the firer one level lower
  and compute the recorded spatial distance: Δh remains 0 and the same bonuses apply.

**RG23. Close-range searches (claim 22; claim 17 for f).** Use RG10's flat map, houses, speed
and recorder, and add to the recorded values each tick: the attacker's position and standing
position, its destination kind and cell, the point its movement is heading for, the target, and
the scenario random stream's state and draw count. Named test data gives one infantry type and
one turreted vehicle type the close-range property and a weapon of range 1 cell, no minimum
range and no elevation, cliff or wall properties. The target is an unarmed enemy vehicle on a
cell centre. Before each run, compute F4's bearing, the claim 22 points and the predicted
draws from the recorded positions. One run per part; the controls repeat a part with the
close-range property off.
- (a) Radius: target ten cells away. Expected: a destination from the single radius of 332
  leptons, target kept. Control: the empty-radius outcome of claim 17 (target dropped).
- (b) Shortcut: the infantry attacker pre-placed at the top standing position of the cell
  diagonally adjacent to the target's cell on the side of the target's top corner, with two
  idle friendly infantry at the right and left positions of the same cell. Expected on the
  first search: movement heading for the bottom position of its own cell (271 leptons from the
  target's cell centre, against 362 for the attacker's cell centre), destination the target's
  cell, target kept, exactly one draw. With the right position free instead, the draw decides:
  a choice of the right position (373 leptons) gives no shortcut. Control: attacker two cells away (*d*₀ = 512),
  no shortcut. Repeat (b) with the ground vehicle attacker: no shortcut. S-0208 W13
  tests the ordinary walker's second choice, immediate reservation replacement, interruption
  of an earlier walk and arrival outcome; RG26 tests the deck-vehicle distinction.
- (c) Bypass: the vehicle with weapon range 0.75 cell (192 leptons), so its single radius is 256
  leptons with the property on or off, ten cells from the target along a map axis. Expected:
  the axis-neighbour cell of the target on the attacker's side (256 leptons, failing the range
  test, within 307) is the destination. Control, property off: every candidate fails, claim 17
  applies. Diagonal candidates (362 leptons) fail in both runs.
- (d) Own occupancy: the same vehicle standing on the centre of the cell axis-adjacent to the
  target, so that it is out of range, its shortcut finds no position (its own vehicle mark)
  and the first F4 primary is its own cell. Expected: its own cell is the destination. Without
  the withdrawal the prediction would instead be a claim 16 alternative or a later primary.
- (e) Draw count (predictions from the recorded random state): compare a vehicle target on a cell centre (class centre) with an infantry
  target standing at the right position (a side class, free in each candidate cell). Expected:
  one draw per primary examined (plus one for a shortcut whose distance holds) for the first,
  none for the second.
- (f) Failure: block every primary cell with immobile units while candidates pass the range
  test or the bypass, leaving nearby cells free. Expected: target dropped, nearby result centred
  on the last range-passing or bypassed primary.
- Falsifier: an empty-radius outcome for close-range infantry, a shortcut where *d*₀ > 384 or
  *d*₁ ≥ *d*₀, rejection of a candidate within 307 leptons for range alone, a draw count other
  than predicted, or a different destination or centre under the recorded inputs.

**RG24. Destroyable-cliff target (claim 23).** A named map on a theater whose destroyable-cliff
role is set, with one destroyable cliff tile at least 2 cells wide inside the usable area and
open ground around it. Named test data gives an infantry type and a vehicle type a weapon that
can attack such a cell (S-0214), range 3 cells, no close-range property. Use RG10's houses,
speed and recorder, recording the target cell before and after the first approach step and
the first destination.
- Runs: force-attack one cliff cell from a stationary attacker placed so that another cell of
  the same tile is nearest to it; then from a position equidistant from two tile cells; then
  with part of the tile outside the usable area and the nearest cell among those outside.
- Expected: the target becomes the nearest qualifying tile cell, the first in row order on a
  tie, never a cell outside the usable area, and the search then runs as claim 20 on it, with
  no draw. A different replacement or no replacement falsifies claim 23.

**RG25. The jumpjet detour exemption (claim 24).** Use RG11 c's wall maze, houses, speed and
recorder. Named test data gives an infantry type the jumpjet flag, the jumpjet locomotor and the
same weapon as RG11's infantry, and sets the locomotion class and movement zone of it and of
the control type explicitly to the same values. The control has ordinary walking locomotion. Order it to attack while airborne so that the first clear,
range-passing primary fails both detour inequalities, computed by S-0212 F8 for that class
and zone, and a later one passes them. A fixture in which the jumpjet's own estimates pass is
not a valid run.
- Expected: the jumpjet takes the first primary. Control: the matched ordinary walking type without the jumpjet
  flag takes the later primary (claim 16). One run each; a jumpjet choosing the later primary
  falsifies claim 24.

**RG26. Successful deck-vehicle shortcut without walking to P (claim 25; Q-0237).** Use
RG23's houses, speed, recorder and named close-range test data, on a map with an intact high
bridge. Place an ordinary driving vehicle on a deck cell diagonally adjacent to its unarmed
enemy target's cell. Keep the ground beneath the attacker empty; its own vehicle mark is
on the deck. Its weapon range must make S-0215 request an approach search. Record the actor's
layer, both occupancy records, exact position, movement heading point, destination, target
and scenario stream state each tick through the first subsequent movement.
- Procedure: issue an attack order. Make up to 20 runs, evaluating claim 22 b from the
  recorded initial draw. Count only runs whose selected P is nonempty and closer than
  the two cell centres' distance. Obtain at least one qualifying run. A draw choosing a
  point that fails the shortcut is a control, not a failure of claim 25. If no qualifying
  run occurs in 20 attempts, this part remains pending rather than falsified.
- Expected in a qualifying run: target kept, destination the target-reference cell, no
  reservation or immediate heading towards P. Any later driving uses that destination's
  ordinary movement rules. Control with the same vehicle on ground: its ground vehicle
  mark prevents the initial choice, hence no shortcut. Repeat a qualifying geometry with
  hovering and sailing vehicles using named test data and legal supporting surfaces,
  if those fixtures can be arranged; their response to P is likewise absent.
- Falsifier: failure of the shortcut despite its recorded inequalities and a free ground
  choice, losing the target, assigning the own cell instead, or installing P as an immediate
  movement point for a driving vehicle. Ordinary walking infantry are the contrast of
  S-0208 W13, not a non-walking control. No result here establishes full flight or sailing
  movement rules.

**RG27. A building against cruising fliers (claim 27; Q-3069).** Flat open map, game speed
index 2, the recorder. A computer player owns a named test building type with a turret and one
weapon, able to fire at aircraft and at ground units, with no minimum range, no cell
rangefinding property, a projectile that is not arcing, and no air range bonus. The type has no
secondary weapon and no SAM flag, and no upgrade is installed. A computer player of another team
owns the fliers: an unarmed named test aircraft type that is not airport-bound, and an unarmed
named test jumpjet type, each with a strength that no shot in the run can use up. The fliers are
moved by a scripted move (a map trigger, a team script or the arena's scheduled order), not by
the human player. Record each tick the building's reference point and target, every shot with
its target's exact position, and each flier's exact position and height.
- Run a (plane): a first pass with a long range field records the plane's cruise height *h*
  above the building's reference point. The test file then gives the weapon a range field
  about one cell above *h*, and the run records its converted lepton value *R* (claim 1). The
  plane is moved east to west along a row at planar distance *p* from the building's
  reference point, starting and ending more than *R* away by planar distance. A pass counts
  only if the plane's recorded height has reached *h* before its planar distance from the
  reference point first falls to *R*. One run each for *p* = 0, 1, 2, 4 and 6 cells.
- Run b (jumpjet): as run a with the jumpjet and the same *R*. The test jumpjet type's jumpjet
  cruise height is set to run a's *h*. A pass counts only if the
  jumpjet's recorded height is at least 208 leptons at every tick at which its planar distance
  from the reference point is at most *R*.
- Run c (air bonus): run a at *p* = 2 and 4 cells with the building type's air range bonus
  set to 1 cell.
- Expected: every shot at a flier has a recorded spatial distance from the reference point
  of at most *R* (run c: *R* + 256). In runs a and b no shot is fired in a pass whose smallest
  recorded spatial distance exceeds *R*, which holds for the larger values of *p*. The
  building's target may be the flier while it is still out of range, without a shot.
- Positive control: the counted pass at *p* = 0 of each of runs a and b has at least one shot
  at the flier, and run c has at least one shot at a flier whose recorded spatial distance
  exceeds *R*. A run without its shot is inconclusive, not a pass.
- Falsified by any shot at a flier whose recorded spatial distance exceeds the limit, in
  particular a shot within *R* by planar distance but beyond it spatially.

## Open questions

- **Q-0230.** Questions 1 and 2 are resolved by claim 21: an ordinary outside-ground
  alternative is not in an inside ground actor's reach region; winged clearance skips that
  comparison rather than relabelling the cell. Question 3 was already answered by S-0212
  claims 6 and 25. The missing boundary test alone does not establish a reachable case
  that chooses an outside alternative; RG20 replaces the earlier impossible same-region
  fixture. Whether a reachable bridge or moving-actor combination makes the outside-area
  switch affect destination choice remains open, not an ordinary flat-ground exemption.
- **Q-2171.** Questions 1 to 3 are answered by claim 19: a high-bridge candidate's region is read
  for its deck. Through enabled links whose first ends lie in one reach region, a ground
  vehicle on the same deck keeps its target and is given a deck cell that also passes the
  clearance, range and detour tests (claims 16 and 19), and infantry and
  attackers on a bank follow the same read (medium). Through a link that is not enabled, or
  with no link found, the same equality of deck reads decides, and the outcome is left open
  (low, `unmeasured`). Question 4 concerns the
  reach comparison of S-0215 claim 37 (a), which belongs to that spec; the change it needs is
  under *Cross-spec changes required in this round*.
- **Q-0237.** Questions 1 to 3 were already answered by S-0208 claims 3 and 21 and W13.
  Claim 22 b now cites them explicitly; claim 25 and RG26 answer question 4. Complete
  subsequent motion of other locomotors remains owned by their movement specs, not by
  this shortcut result.
- **Q-0172.** Answered: question 1 by claim 17 (for every search, close-range included),
  question 2 by claims 20 and 23, question 3 by claim 22, questions 4 and 5 by claims 19 and
  22 c, question 6 by claim 21 and question 7 by claim 18. Two edges stay open: a vehicle
  alternative outside the map's legal cells (claim 21), and whether withdrawing and restoring
  the attacker's occupancy (claim 22 e) changes any observable order among a cell's occupants.
- **Reach region of a moving attacker.** The reach region that the clear-cell test of claims 15
  and 19 requires is looked up from the cell of one point of the attacker, the same point as
  S-0215 claim 37 (a1) names. While the attacker travels through a tunnel (S-0212 Terms), it is
  the tunnel exit's cell centre. Otherwise it is the point its locomotor is moving it towards at
  that moment while there is one, and otherwise its own position. For a walking infantry unit that is its walking target
  (its scatter reference point, S-0208 Terms). For a vehicle it is its heading point as its
  movement spec gives it (S-0207 Terms for driving, S-0503 Terms for hovering). Claims 15 and 19
  state the stationary case. A moving attacker whose next point lies in another reach region has not been
  measured, and this point for a moving attacker has low confidence, as S-0215 claim 37 rates a
  moving actor's cell.

- **Q-0072 question 2, remaining coverage.** S-0517 claim 14 defines the deploy-to-fire cell
  test and its primary-cell filter: when the vehicle's own cell fails that test, a primary
  cell is examined only when it passes the same test. A skipped primary never becomes
  claim 17's failure anchor. S-0215 claim 37 f2 gives the adjusted reach. The rest of the
  deploying search still needs verification. Claims 19 to 24 now cover ordinary searches on terrain and
  bridges, cell targets, close-range types, the jumpjet detour exemption, the failure centre
  and legal alternatives outside the usable area. Q-0072 is not yet in this spec's requests list. Crushing destinations are
  owned by S-0210 claim 11 and S-0215 claim 37 h, not by this search.
- **Q-0072 question 3, remaining stopping coverage.** S-0215 claim 37 e's approach-step retention
  does not itself decide when movement ends. S-0207 claim 19 resolves driving, sailing and
  hovering vehicle outcomes; S-0208 claims 16 and 17 resolve ordinary walking infantry's stop
  and destination outcome (scenario W10 there). The arrival and stop rules for jumpjet infantry remain
  open in S-0208. No general arrival-before-firing rule is implied here.
- **Q-0072 questions 1, 4 and 5.** S-0215 claim 37 owns the trigger, gating, renewal cadence,
  buildings, deployed-infantry exceptions and zero-speed actor qualifications (part g).
  Zero type speed alone does not exclude an actor from the approach step or prevent it receiving
  a search destination; it does not promise actual movement. Those qualifications have that
  claim's medium confidence and measurement scenario, not a complete immobile-type locomotion
  contract. Its in-the-playfield mark remains open there. The range test alone is not the whole
  firing check: rearming or turning need not prevent it from passing.

- The elevation-deck mark is distinct from S-0212's deck-entry mark and from the high-bridge
  test of claims 7 and 10. Its ordinary intact-overlay and destruction scope is specified in
  Terms and tested by RG3 d. How overlapping markings and rebuilding affect previously marked
  cells remains unverified. No equality with either other mark is asserted.
- RG16 still needs a reachable normal-action fixture before its multi-weapon and empty-slot
  distinctions can be called measurable; these parts of claim 13 remain unmeasured.
- Whether any rule besides the flak scatter, scan and guard reach, the approach step and the
  open-topped infantry stop of S-0208 claim 17 reads the firer's weapon range of claim 13.
- Whether any rule besides the firing check, the approach-position search and the movement
  users named in claim 12 calls the range test.

## Cross-spec changes required in this round

The earlier coordinated changes are already present: S-0306 revision 6 introduced the
current-weapon rule and bridge-mark question and approved revision 7 retains both.
S-0212 claim 25 includes close-range approach failures while retaining its other parameters
and exclusions; S-0215 claim 37 cites claims 15 to 24; S-0208 claims 3 and 21 own the initial
choice's use and the ordinary walking outcome; the glossary defines the close-range property.
These behavioural changes are already in the approved copies as well as the current drafts.
Sign-off should retain the failed-approach row's agreement with claim 17, including close-range
failures, and the shortcut's target-reference desired point and ground occupancy inputs in
claim 22 b and c. S-0215's current cross-spec hand-off note now recognises approved S-0307
revision 10 and its current correction draft, retaining claims 15 to 25. Its claim 37's
approach processing and destination retention agree with this spec and require no behavioural
change. S-0208's current hand-off note likewise retains claim 17's agreement with S-0212
claim 25. Confirm these matching contracts at joint sign-off; no further edit to either spec
is requested here. The shared teleport-ground-reservation and same-tick search and landing
checks named by S-0215 concern S-0207, S-0209, S-0212 and S-0208; they do not change this
spec's approach-position selection or failed-approach search parameters.

S-0215 claim 37 (a), for Q-2171 question 4: the per-tick rule's "cannot reach its target's cell"
is the comparison of S-0212 claim 6, with the actor in place of the unit and its target's cell
in place of the destination cell. This changes S-0215's wording ("the target's cell is not in
the actor's reach region"). Outside claim 6's exceptions (below), the two agree whenever the
actor's cell is in a reach region, and differ only where the comparison counts two cells as one
region although neither lies in a reach region (both of no reach region, or both off the map).
The actor's cell is read for its deck (S-0212 claim 4) when the actor is on a bridge deck, and
the target's cell whenever that cell carries a high bridge, whichever layer the target is on.
Claim 6's exceptions skip the comparison, and the target then counts as reachable. Which cell
stands for a moving actor in this comparison is left to S-0215's revision: claim 19's scatter
reference point is not measured for a moving attacker (Open questions), and this hand-off states
no rule for it.
Backing: `unmeasured`. S-0212 claim 6 states this comparison for path computations; no scenario
here isolates the per-tick rule's making the same comparison, with the same deck reads and
exceptions, from claim 19's search (RG18 tests the candidate side only).
Confidence: medium for the comparison and its deck reads through an enabled link; low for its
deck reads through a link that is not enabled or with no link found and for two cells of no
reach region or two off-map cells (as S-0212 claim 4 rates them), and for the exceptions
skipping it. S-0212 claim 6 rates those exceptions medium for path computations; they are rated
lower here because no backing of that claim concerns the per-tick rule. S-0215's revision owns
the scenario that tests it.
