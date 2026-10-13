---
id: S-0307
title: Weapon range test
topic: combat
revision: 4
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: []
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
walking infantry's stopping and destination outcomes. Exceptional searches and jumpjet stopping
remain open below.
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
- **Airborne, floor-bound.** An object on the map is *airborne* when its height above the
  surface is at least 208 leptons (two height levels), and *floor-bound* otherwise. These are
  the original's two tests used below; an object not on the map is neither.
- **Measured point** *M*_T. The point of the target that the range test measures to (claim 7). It
  is not S-0306's target point: it has no aim shift, and its height may be replaced.
- **Source point** *S*. The point of the firer that the range test measures from (claim 8).
- **Subject to elevation.** A projectile type property (Import mapping). It switches on the
  elevation bonus of claim 6.
- **Cell rangefinding.** A weapon property (Import mapping). It moves the source point to the
  firer's cell (claim 8 a).
- **Close-range property.** A boolean type field (Import mapping). Claims 15 and 16 cover only
  types without this property; the approach rules for types with it remain open.
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
- **Elevation level of a cell** λ(*c*). The cell's height level, plus 4 when the cell is marked as
  carrying a bridge deck: the same 4-level lift that S-0102 (claims 13 and 15) gives such a cell's
  depth and its deck.
  It depends on the cell only, not on whether the object stands on the deck or under it.

## Behaviour

### The range field

1. **Weapon range in leptons.** The weapon's range field is a decimal number of cells. The
   importer reads it with the decimal form of S-0003 (claim 6, single precision), multiplies it by
   256 in double precision and truncates toward zero: *R*₀ = trunc(RZ₅₃(*x* × 256)) leptons. A
   value read as exactly −1 is treated as absent and keeps the value already in force (the
   absent-key default is 0). The minimum-range field and the type's air range bonus field use the
   same form. The domain is a field magnitude below 8 388 608 cells (|*R*₀| below 2³¹); beyond it
   the value is not covered.
   - Backing: `measurable` (scenario RG1, runs a, b and d)
   - Confidence: high

2. **Unlimited range.** A weapon whose *R*₀ is exactly −512 (a range field of −2) passes the range
   test for every target, without any other part of this spec being applied (no minimum range,
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
   - Backing: `measurable` (scenario RG2, runs d and e, each addition alone)
   - Confidence: high for each addition; medium for the combination

6. **Elevation bonus.** When the weapon's projectile is subject to elevation and both the firer
   and the target are floor-bound, a bonus *E* that grows with the height-level difference is
   added. It differs by projectile kind: *E*₁ (F2) for a projectile that is not an arcing
   projectile, *E*₂ (F2) for an arcing projectile. Without the subject-to-elevation property, or
   when either object is airborne or off the map, *E* = 0. When the firer's cell has an elevation
   level lower than or equal to the target's, Δ*h* = 0 and F2 gives *E* = 0 for every cap *c* ≥ 0
   (a negative cap is an edge case below). The levels compared are elevation levels
   (Terms), so a firer standing under a high bridge counts as 4 levels higher than the ground around
   it, and a target under a bridge as 4 levels higher too (quirk `quirks.elevation_deck_cell_level`).
   - Backing: `measurable` (scenario RG3; run d for the bridge-cell levels)
   - Confidence: high for the formulas; medium for the bridge-cell levels (which cells carry the deck
     mark is not known; see Open questions) and the absent-section case (see Edge cases). The
     overflow behaviour of F2 is bad data and not measured (Edge cases).

### What is measured

7. **Measured point.** *M*_T is the target's reference point (S-0204 claim 4: the centre of a
   building's foundation, without any bib row). When the target is floor-bound, the world height
   of *M*_T is replaced by the terrain height at *M*_T (on a slope, the height of the sloped surface
   at that point, S-0106), plus 416 leptons when the cell under *M*_T carries a high bridge. So a
   floor-bound target counts as standing on the ground surface,
   and a floor-bound target in a high-bridge cell counts as standing on the bridge deck even when
   it is under the bridge (quirk `quirks.range_target_on_deck`).
   - Backing: `measurable` (scenario RG4, runs a and b)
   - Confidence: high; medium for the sloped-surface case (not measured)

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
   - Backing: `measurable` (scenario RG5 covers (a), RG2 run f covers (b))
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
     away horizontally and *M*_T.z − *S*.z higher. Since *v* is computed from *L*, reachability
     fails only for targets far above the source.
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
    Claims 15 and 16 specify covered cases of choosing such a candidate. S-0215 claim 37 uses
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
    *A*'s type is open-topped, ρ is the smallest of that value and the range fields of the primary
    weapons of *A*'s current passengers; a passenger with no primary weapon is skipped, and a
    transport without passengers uses its own value. The result is read by the rules that scale
    with the firer's reach: the flak scatter of S-0306 (claim 4, its *R*_W) and the reach used by
    the target-scan and guard rules of S-0309 (its F1), and the approach step and search
    (S-0215 F10 and F4 below). The range test of claims 1 to 11 does
    **not** use it: it uses the weapon's own range field, so an open-topped transport whose
    passengers have short weapons still passes the range test at its own weapon's range, while
    those other rules see the shorter value. The passengers' own range tests use their own weapons
    and the bonus of claim 5.
    - Backing: `measurable` (scenario RG9 for the range test's non-use; the scatter and scan
      consequences through S-0306 scenario P3 and S-0309's scenarios)
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
    and belongs to that region. This is not the actor's neighbouring-cell entry outcome: the
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
    claim are legal and inside the usable area.

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
    | its type has the hunter-seeker property, or it is vehicle-thief infantry | destination becomes the target object itself, and the target is kept |
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
    draw. When there were candidates, the centre of the failure search is not specified here
    (Open questions).
    - Backing: `measurable` (RG12)
    - Confidence: high for keeping or dropping the target and the object-destination exceptions,
      medium for the empty-radius search parameters

18. **Balloon-hover vehicle with a vertical projectile (Q-0072 question 2).** When a vehicle with
    the balloon-hover property takes the approach step and the weapon chosen for its target has
    a vertical projectile, its destination becomes the target object itself, and the target is
    kept. It does not search for a firing cell, even when it already has a destination or the
    target is already in range. The weapon and its projectile must both exist. This replaces
    the common approach step of S-0215 claim 37, subject to that claim's earlier crushing
    exception. It takes no random draw. Without either property this exception does not apply.
    - Backing: `measurable` (RG13)
    - Confidence: high

## Formulas

All integers are leptons unless stated. trunc is truncation toward zero. The original computes
every floating-point step in the round-toward-zero mode (S-0205, `quirks.chop_rounding`); this spec
defines no switch of its own for it. Squares of integer differences are summed in double precision
(exact here) and passed to the table square root of S-0204 F1 (`quirks.table_root_distance`).
Every other sum and product of integers (*R*, the limits, F2) is reduced to the signed 32-bit range
with the wrap function

> *w*(*n*) = the integer congruent to *n* modulo 2³² that lies in [−2³¹, 2³¹ − 1].

*w* changes a value only outside that range, so it matters only for bad data (Edge cases).

**F1. Effective range** (claims 2 to 6).

When *R*₀ = −512 the target is in range and no other formula of this spec applies (claim 2).
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

The margins use the nearest double-precision representations of the decimals shown. Let *f* be
S-0204's direction from the target's reference point *T* to the attacker's reference point.
The bearing number *b* is S-0204's 256-step facing of *f*, a whole number from 0 to 255. At each
radius the bearing differences, in 1/256-turn units, have this preference order:

> 0, +1, −1, +2, −2, +3, −3, +4, −4, +5, −5, +6, −6, +8, −8,
> +16, −16, +24, −24, +32, −32, +48, −48, +64, −64.

For each difference δ, *p* is the signed representative of 256(*b* + δ) modulo 65536, in
[−32768, 32767]. With *c* = −0.00009587672516830327 radians per facing unit, held in double
precision, and the table trigonometry of S-0105 F1:

> θ = RZ₅₃((*p* − 16383) × *c*)
> x = trunc(RZ₅₃(T.x + RZ₅₃(cosₜ(θ) × radius)))
> y = trunc(RZ₅₃(T.y − RZ₅₃(sinₜ(θ) × radius)))
> candidate cell = (trunc(x / 256), trunc(y / 256))
> candidate point = (256 × cell.x + 128, 256 × cell.y + 128, 0).

The products and sums in the position formula use double precision in round-toward-zero mode.
Integer conversions truncate toward zero. Repeated candidate cells are not removed from the
preference order. The integer radius arithmetic uses *w* as above. This claim covers valid
map coordinates and radii without wrap; malformed-data outcomes of this search are still open.

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

- **`quirks.garrison_range_drops_air_bonus`** (default on, the original). A garrisoned building's
  range replaces the weapon range together with its air bonus, so the air bonus is lost (claim 4).
  Off: the air bonus is added to the garrison range.
- **`quirks.range_target_on_deck`** (default on). A floor-bound target in a high-bridge cell is
  measured at deck height even below the bridge (claim 7). Off: a floor-bound target is measured
  at the height of the surface it is on (the deck only when it is on the bridge deck, the terrain
  otherwise).
- **`quirks.elevation_deck_cell_level`** (default on). The elevation bonus compares elevation
  levels, which add 4 for every cell marked as carrying a bridge deck, whether the object stands on
  the deck or under it (claim 6). So a firer under a bridge gains a bonus against targets on the
  open ground beside it, and a target under a bridge reduces the firer's bonus. Off: the 4 levels
  are added only for an object that is on the bridge deck.
- **Elevation increment 0 (departure, bad data).** With *I* = 0 (its absent-key default, also when
  the elevation section is missing) the original stops with a program error, an integer division
  by zero (S-0205, claim 3), whenever the range test is asked for two floor-bound objects with a
  projectile subject to elevation, whatever their height difference, Δ*h* = 0 included. Our engine takes *E*₁ = *E*₂ = 0 when *I* = 0, and the importer warns when a
  projectile subject to elevation exists while *I* = 0. A negative *I* is not zeroed: it gives the original's integer division, truncating toward zero (*k* = trunc(Δ*h* / *I*) ≤ 0). *E*₂ and *E*₁ then follow F2, including the configured increment bonus *b*, cap *c* and wrap; their signs are not fixed for arbitrary imported values. This behaviour is reproduced, with the same warning.
- **Negative elevation cap (bad data, reproduced).** The cap applies even when Δ*h* = 0 (*k* = 0 and *f* = 0), so *f*′ = *c* for *c* < 0. The resulting values are still calculated by F2, including its signed-32-bit wrap: *t* is obtained by the stated saturation rule, *E*₂ = *w*(256*t*), *σ* = *w*(*E*₂² + (104 Δ*h*)²), and *E*₁ = ⌊*r*(|*σ*|)⌋. When the intermediate products do not wrap and |trunc(*c*)| ≤ 127, *E*₂ = 256 × trunc(*c*) and, at Δ*h* = 0, *E*₁ = |*E*₂| (S-0204 claim 5). This applies to every pair of floor-bound objects with a subject-to-elevation projectile, including a firer on lower ground. The importer warns.
- **Negative range fields**, other than the −1 sentinel, whose *R*₀ is not −512, give a negative
  *R*₀ (a field slightly beyond −2, such as −2.001, still truncates to −512 and is unlimited). F1
  and claim 9 apply to it unchanged, so the target is out of range unless the other terms of the
  limit reach the distance; reproduced, the importer warns.
- **Overflow in F2 and the limits (bad data, reproduced, not measured).** The bonus term of F2 can
  overflow: with |*t*| ≥ 182 the sum wraps (F2 gives the result), and the limits of claim 9 wrap when
  *R* + *E* leaves the 32-bit range. A large positive result needs a cap of 182 or more together
  with a *k* × *b* that reaches it (for example *I* = 1, *b* = 200, *c* = 200, Δ*h* = 1). A cap of
  −182 or less wraps for every floor-bound pair with a subject-to-elevation projectile, even at
  Δ*h* = 0. The importer warns when such a projectile exists and |trunc(*c*)| ≥ 148 (a warning that
  can come without an actual overflow).

## Import mapping

| File, section | Key | Concept | Form | Absent |
|---|---|---|---|---|
| rules, weapon section | `Range` | weapon range *R*₀ (claims 1, 2); also the approach radius through ρ in claims 13 and 15 | lepton form of claim 1 | 0 |
| rules, weapon section | `MinimumRange` | minimum range *m* | lepton form of claim 1 | 0 |
| rules, section of an infantry or vehicle type | `CloseRange` | close-range property, excluded from claims 15 and 16 | boolean (S-0003) | no |
| rules, infantry or vehicle type section | `SpeedType` | locomotion class for approach-cell qualification and the failure nearby search | name, as S-0211 Import mapping | S-0211 claim 3: infantry foot; vehicle track when its crusher flag is yes, otherwise wheel |
| rules, infantry or vehicle type section | `MovementZone` | own movement zone and reach region for claims 15 to 17 | name, as S-0211 claim 12 and Import mapping | normal |
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

All scenarios run under the glossary's *measurement setup*, with a named test rules file (a loose
copy of the rules file changing only the fields named) removed after each run. The observable is
the one of S-0204 scenario C2: an attacker ordered to attack a pre-placed, unarmed enemy target
either fires its first shot without leaving its cell ("fires") or moves first ("moves"), watched
for up to 300 ticks; a building attacker that cannot move either fires or does not. Targets belong
to a computer player that the map leaves without orders. Unless stated otherwise the attacker is a
stock turreted vehicle on cell (*x*, *y*) of flat open ground at height level 0, its weapon has no
minimum range and a projectile that is neither arcing nor subject to elevation, cliffs or walls,
and targets stand on cell centres. One run per row; no random draw decides the outcome.

**RG1. Range field and comparisons (claims 1, 2, 9).** Target vehicle on (*x* + 5, *y*) (1280
leptons).
- Run a: range field 5 (1280). Expected: fires.
- Run b: range field 4.99 (single precision 4.98999…, × 256 truncated: 1277). Expected: moves.
- Run b′: range field 4.9999 (1279.97…: 1279 truncated, 1280 rounded). Expected: moves.
- Run c: range field −2, target on (*x* + 40, *y*). Expected: fires.
- Run d (the −1 sentinel): the test rules file gives the weapon range field 5; the test map's own
  override section for the same weapon gives −1. Target on (*x* + 5, *y*). Expected: fires (the
  value 1280 of the earlier layer is kept). Control: the override section gives 4 instead: moves.
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
- Run d (bridge-cell levels): flat level-0 map with a high bridge; the straight weapon subject to
  elevation (same test file) on the ground *under* the bridge, a target on open ground 5 cells
  away across the bridge's axis (spatial distance 1280 with no height difference). Expected:
  fires (λ_A = 4, *E*₁ = ⌊*r*(256² + 416²)⌋, limit 1512); control with the attacker on open ground
  at the same distance: moves (limit 1024 < 1280).
- Falsified if: the straight weapon fails at (5, 1) or the arcing one fires there, the control
  fires at 4, or run d does not fire under the bridge.

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
- Falsified if: it fires at 1 cell.

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
field 6 (1536) carrying one infantry passenger whose primary weapon has range field 3 (768); no
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
Director and mods are off. Use named test data to give one ordinary ground infantry type and one
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
- Falsifier: a cell destination or no destination with both properties and a target, an object
  destination under the common search in either control, or a search-specific random draw.

**RG14. Absent elevation section and zero increment (claim 6, Edge cases).** Use named rules test data with the elevation section absent in every layer and a projectile whose subject-to-elevation property is on. Place a floor-bound firer and target on the map and issue an attack that invokes the range test. Record whether the original reports a program error/stops or completes the test. Expected: the original stops with the integer-division-by-zero program error described in Edge cases. Falsified if it completes the range test without that error. Repeat with an explicit zero increment if the section-absence case is not equivalent. This measurement observes the original's failure; it does not validate the safe result of our engine's departure.

**RG15. Signed-32-bit overflow in F2 (claim 6, F2).** Use a named test map with a firer one height level above a floor-bound target, and named rules test data setting *I* = 1 height level, *b* = 200 cells per step, *c* = 200 cells, and weapon range *R*₀ = 100 leptons, with other range adjustments disabled. Put the target at a recorded spatial distance of 45000 leptons; arrange a clear line of fire and an eligible target. The stated F2 gives *k* = 1, *E*₂ = 51200 leptons and a wrapped *σ* of −1673516480 square leptons, so the comparison uses *E*₁ = ⌊*r*(1673516480)⌋ = 40908 leptons and a limit of 41008 leptons. Expected: the target is out of range and the attacker moves before firing. Falsified if it fires within range under these inputs. A control with a cap/increment combination that does not overflow must produce the result predicted by that control's F2 value. Record the input values and first-fire/first-movement outcome.

## Open questions

- **Q-0072 question 2, remaining coverage.** Claims 15 to 18 do not yet cover cell targets,
  close-range types and their infantry standing-position choice, raised terrain or bridges,
  jumpjet searches, or the deploying search beyond S-0215's adjusted reach. The exact failure-search
  centre after candidates have been tried is open: it must not be assumed to be the target's cell.
  Alternative cells outside the usable area are also not covered. These paths need further
  verification, so Q-0072 is not yet in this spec's requests list. Crushing destinations are
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

- The elevation level (Terms) uses a per-cell "carries a bridge deck" mark that is not the same
  mark as the "carries a high bridge" test of claims 7 and 10. Which cells of a bridge carry it
  (every deck cell, or only some, such as the cells that also lift the deck in drawing) is not
  known; RG3 run d with the attacker on different cells along the bridge would show it.
- Whether any rule besides the flak scatter, scan and guard reach, the approach step and the
  open-topped infantry stop of S-0208 claim 17 reads the firer's weapon range of claim 13.
- Whether any rule besides the firing check, the approach-position search and the movement
  users named in claim 12 calls the range test.
