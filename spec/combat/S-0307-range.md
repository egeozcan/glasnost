---
id: S-0307
title: Weapon range test
topic: combat
revision: 1
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
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
projectiles fly once fired is S-0306.

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
- **Wall-damaging warhead.** A warhead with the wall-damage property (Import mapping). It is the only
  property of a warhead that this spec reads (claim 11).
- **Open-topped transport.** A transport type whose passengers fire from inside it. Its passengers
  and its own weapon are each a firer.
- **Garrisoned building.** As in S-0301 (Terms): an occupiable building that may fire while
  occupied and has at least one occupant.
- **Firer's weapon range** ρ(*A*, *i*). The reach that other rules (flak scatter, S-0306 claim 4, and
  the scan and guard reach of S-0309) read for firer *A*'s weapon in slot *i* (claim 13). It is not
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
    candidate position as source point, without the substitutions of claim 8. The target-scan and
    order rules that measure distance in other ways are owned by their specs (S-0309, S-0214).
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
    the target-scan and guard rules of S-0309 (its F1). The range test of claims 1 to 11 does
    **not** use it: it uses the weapon's own range field, so an open-topped transport whose
    passengers have short weapons still passes the range test at its own weapon's range, while
    those other rules see the shorter value. The passengers' own range tests use their own weapons
    and the bonus of claim 5.
    - Backing: `measurable` (scenario RG9 for the range test's non-use; the scatter and scan
      consequences through S-0306 scenario P3 and S-0309's scenarios)
    - Confidence: high for the rule as read from the original; medium that no other rule reads it

### Degenerate questions

14. **No target, no weapon.** The range test as the firing check asks it answers *in range* when
    there is no target (the firing check's other answers decide, S-0301), and *out of range* when
    there is a target but the firer has no weapon in the slot. In both cases no other claim of this
    spec changes the answer.
    - Backing: `unmeasured` (no scenario of this spec can ask the question without a target or
      without a weapon)
    - Confidence: medium

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

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
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
  projectile subject to elevation exists while *I* = 0. A negative *I* is not zeroed: it gives the
  original's integer division, truncating toward zero (*k* = trunc(Δ*h* / *I*) ≤ 0, so *E*₂ is
  zero or negative and *E*₁ the table root of F2's hypotenuse), and is reproduced, with the same
  warning.
- **Negative elevation cap (bad data, reproduced).** The cap applies even when Δ*h* = 0 (*k* = 0,
  *f* = 0), so a cap *c* < 0 gives *E*₂ = 256 × trunc(*c*) < 0 and *E*₁ = ⌊*r*(*E*₂²)⌋, which is |*E*₂|
  while |trunc(*c*)| ≤ 127 (S-0204 claim 5), for every pair of floor-bound objects with a
  subject-to-elevation projectile, including a firer on lower ground. The importer warns.
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
| rules, weapon section | `Range` | weapon range *R*₀ (claims 1, 2) | lepton form of claim 1 | 0 |
| rules, weapon section | `MinimumRange` | minimum range *m* | lepton form of claim 1 | 0 |
| rules, weapon section | `CellRangefinding` | cell rangefinding property (claim 8) | boolean (S-0003) | no |
| rules, weapon section | `Projectile` | the projectile type whose properties claims 6 and 9 to 11 use | type name | — |
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
- Run b (garrison): a stock occupiable building with a 2 × 3 foundation (⌊min/2⌋ = 1) and one
  occupant whose weapon can fire at ground units, occupied-weapon range set to 3 (limit 1024).
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

## Open questions

- The elevation level (Terms) uses a per-cell "carries a bridge deck" mark that is not the same
  mark as the "carries a high bridge" test of claims 7 and 10. Which cells of a bridge carry it
  (every deck cell, or only some, such as the cells that also lift the deck in drawing) is not
  known; RG3 run d with the attacker on different cells along the bridge would show it.
- Whether any rule besides the flak scatter and the scan and guard reach reads the firer's weapon
  range of claim 13 (no other rule that reads it is known).
- Whether any rule besides the firing check and the approach-position search calls the range test
  (claim 12).
