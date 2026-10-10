---
id: S-0204
title: Coordinates, distance and facing
topic: simulation
revision: 5
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0204: Coordinates, distance and facing

## Summary

How the original places objects in the world (positions in leptons, cells, heights), how it measures
the distance between two objects, how it turns the direction from one point to another into a
facing, and how coarse facings (*n* steps of a turn, *n* a power of two from 2 to 256) are derived
from a facing.
Distances and directions use tabulated functions, so they differ slightly from exact Euclidean
geometry; this spec gives the exact results. Turning over time is S-0206.

## Terms

Glossary terms are used as defined there, in particular *lepton*, *cell*, *world height*, *height
level*, *facing*, *foundation*, *top corner*, *tick* and *round-toward-zero mode*.

- **Position.** An object's location in the world: three integers (*x*, *y*, *z*) in leptons. *x*
  and *y* are the map axes of the glossary's *cell* entry; *z* is the world height. Each component
  is an integer from −2³¹ to 2³¹ − 1.
- **Reference point.** The point of an object that distance and direction measurements use (claim
  4). For a building it is the centre of its foundation; for every other object it is its position.
- **Bib flag, bib row.** The *bib flag* is a yes/no field of a building type (Import mapping),
  off unless the data sets it. It is a different field from the art setting that names a
  building's *bib sprite* (glossary, S-0102): the flag can be set without a bib sprite and the
  reverse. For a building whose type has the bib flag set, the original can treat the foundation
  as one row of cells longer along the map *y* axis (towards map south): *W* × (*H* + 1) cells
  instead of *W* × *H*. That extra row is the *bib row*. The original counts it only when a rule
  asks for the foundation including the bib. Distance, direction and the weapon range test never
  ask for it (claims 4, 6 and 7), so in this spec *W* × *H* is always the foundation without
  any bib row, whatever the flag. Which other rules, if any, count the bib row is for the
  structure specs to say.
- **Planar distance, spatial distance.** The two distance measures between objects (claim 6):
  planar ignores *z*, spatial includes it.
- **Table square root** *r*(*n*) of a non-negative integer *n*. The tabulated square root of
  Formulas F1. It is the same function that S-0105 (F1) calls the table square root and S-0009
  (claim 19) uses, restricted to integer inputs.
- **Direction** of a point *Q* from a point *P*. The facing that the original computes for "the
  way from *P* to *Q*" (claim 8, Formulas F3). It is close to, but not always equal to, the exact
  angle.
- **Signed facing difference** *δ*(*a*, *b*) of two facings. The value *b* − *a* reduced to the range
  −32768 … 32767 (Formulas F5). A positive value is a clockwise turn, a negative value a
  counter-clockwise one, and a half turn counts as −32768 (counter-clockwise).
- **n-step facing** of a facing *f*, for *n* a power of two from 2 to 256. *f* rounded to the
  nearest of *n* equally spaced directions, numbered 0 … *n* − 1 clockwise from map north (Formulas
  F4). The 8-step facing is also called the *8-direction*.
- **Truncation to double precision, trunc₅₃.** Rounding a real number toward zero to the nearest
  number with 53 significant binary digits. **Truncation to single precision, trunc₂₄.** The same
  with 24 significant binary digits. Both describe the original's floating-point results under the
  round-toward-zero mode (S-0205, number domains). More generally, trunc_*b* rounds toward zero to
  *b* significant binary digits; F1 uses trunc₁₄.

## Behaviour

### Positions and cells

1. **Positions are integer leptons.** Every object position the game rules use is an integer
   triple of leptons. A cell edge is 256 leptons and one height level is 104 leptons of world height
   (glossary; S-0101). Cell coordinates are integers from −32768 to 32767.
   - Backing: `unmeasured` (the integer domain is internal; its consequences are tested through
     claims 6 and 8)
   - Confidence: high

2. **Position to cell.** The cell that contains a position (*x*, *y*, *z*) is (trunc(*x* / 256),
   trunc(*y* / 256)), with division truncated toward zero. This agrees with S-0101 (F7). For the
   non-negative positions of every legal cell it equals ⌊*x* / 256⌋. A negative component (a point
   off the map's north-west edges) is truncated toward zero, so the positions −255 … 255 along an
   axis all map to cell coordinate 0.
   - Backing: `unmeasured` for negative positions (no object can be observed there); the
     non-negative case is S-0101's
   - Confidence: high

3. **Cell to position.** The centre of cell (*x*, *y*) is (256*x* + 128, 256*y* + 128) and its top
   corner is (256*x*, 256*y*) (glossary). A building's own position is the centre of the top-left
   cell of its foundation (S-0101, claim 6).
   - Backing: `unmeasured` (stated by the glossary and S-0101)
   - Confidence: high

4. **Reference point of a building.** For distance and direction (claims 6 to 8), a building is
   measured from the centre of its foundation, not from its own position: for a foundation of
   *W* × *H* cells the reference point is its position plus (128 × (*W* − 1), 128 × (*H* − 1), 0)
   leptons, which is its foundation's top corner plus (128*W*, 128*H*). *W* × *H* is the foundation
   without a bib row (Terms), even when the building type has the bib flag set. Every other object is
   measured from its position.
   - Backing: `measurable` (scenario C2: runs d to g for the reference point, runs h and i for the
     bib row; for a 1 × 1 building the reference point is its position)
   - Confidence: high

### Distance

5. **The table square root.** Distances between objects are taken with the table square root of
   Formulas F1, not the exact square root. The squared distance is formed exactly (F2; it is an integer
   below 2⁵³ for every pair of map points). The result truncated to an integer, ⌊*r*(*n*)⌋, is never more than ⌊√*n*⌋. Its
   shortfall *s*(*n*) = ⌊√*n*⌋ − ⌊*r*(*n*)⌋ grows with the distance, because the table keeps only
   14 significant binary digits of *n* (relative error of *r* below 2⁻¹⁴):
   - *s* = 0 for every *n* below 16641 (distances below 129 leptons).
   - *s* ≤ 1 for every *n* below 536987929 (distances below 23173 leptons, about 90.5 cells); for
     distances of 2 to 10 cells about 4 % of values are one lepton short, and from 10 to 50 cells
     about 20 %.
   - *s* first reaches 2 at *n* = 536987929 and first reaches 3 at *n* = 2147673649 (46343
     leptons, about 181 cells); in general *s* < √*n* / 16384 + 1, so up to about 12 leptons at
     the far end of the largest maps.

   Positions on cell centres give squared distances *n* = 65536 × *m*, with *m* a sum of two
   squares of cell differences, and *r*(65536 × *m*) = 256 × *r*(*m*) exactly. For *m* below 16384
   (pairs of cell centres less than 128 cells apart) the result equals ⌊√*n*⌋. From *m* = 16393 on
   it is often short, and the first *m* that is 2 leptons or more short is 32770: of the 82489
   non-zero values of *m* that cells up to 511 apart can give, 23995 are exact, 22923 are one lepton short and the rest
   are 2 to 8 leptons short. Our engine reproduces the table square root by default
   (`quirks.table_root_distance`). That one switch covers every use of the table square root that
   measures a distance: the object distances of this spec and the farthest-sum start-position rule
   of S-0009, which measures each pair of start positions with the same table.
   - Backing: `measurable` (scenario C3, when a placement at exactly 129 leptons exists; if none does,
     the shortfall is `unmeasured` and rests on F1) for the existence of the shortfall; the exact values are
     `unmeasured` (a shortfall of a few leptons at an arbitrary distance cannot be isolated in
     play) and rest on the table, which is reproduced exactly by F1
   - Confidence: high

6. **Distance between two objects.** The distance from object *A* to object *B* is
   *D*(*A*, *B*) = max(0, ⌊*r*(*n*)⌋ − *β*(*B*)) leptons, where *n* is the squared difference of
   their reference points (claim 4). *n* = Δ*x*² + Δ*y*² for the planar distance and
   Δ*x*² + Δ*y*² + Δ*z*² for the spatial distance. *β*(*B*) = 64 × (*W* + *H*) leptons when *B* is a
   building with a *W* × *H* foundation (so 128 for 1 × 1, 256 for 2 × 2, 384 for 3 × 3), and 0
   otherwise. *D* is defined between two objects that exist; what a rule does when it has no
   object to measure towards is owned by that rule.
   - Backing: `measurable` for the reference point and the size of *β* with a building as *B*
     (scenario C2: the range test of claim 7 asks whether ⌊*r*(*n*)⌋ ≤ range + *β*(*B*), which for a
     non-negative range is the same test as *D*(*A*, *B*) ≤ range; it shows the reference point and
     *β* of the shared formula in that test, while the rules that use the value of *D* itself measure
     it through their own specs) and for the root (C3).
     `unmeasured` for the clamp max(0, …) and for *D* with the building as the *first* object.
     The clamp matters only for an object *A* closer to a building's reference point than *β*, that
     is, standing on or near the building's foundation, and only for a rule that uses
     the value of *D* rather than comparing it with a non-negative threshold. No such rule is
     covered by a spec yet (the area-guard leash of S-0215, for example, compares *D* with a
     non-negative leash length). The order of the two objects is fixed by each rule that uses *D*,
     so the one-sided form is measured through those rules by their own specs. The range test's
     one-sidedness (claim 7) is measured by C2 runs j and k.
   - Confidence: high for the formula; medium for the cases in which a rule uses the planar or the
     spatial form (claim 7)

7. **Which rules use which distance.** Every game rule known to measure between two objects uses
   *D* of claim 6 or the same table square root on the difference of two positions. Most object
   rules (guarding an area, the spread of a detonation, team and targeting decisions) use the spatial
   distance. The planar form is rare. A weapon's range test uses the table square root on the
   difference between a source position (which the firing rule supplies) and the target's
   reference point (claim 4), and the target is in range when that truncated root is at most the
   range. For a weapon whose projectile does not fly in an arc, it lengthens the range by *β* of a
   building *target* (again without a bib row) instead of shortening the distance. A building
   *attacker* gets no lengthening from its own foundation, and a weapon whose projectile flies in an
   arc gets no lengthening for a building target either: the plain truncated root is compared
   with the range. The combat specs own which form it uses, its other range adjustments (some depend on the
   projectile and on height) and when it applies. No game rule was found that uses an integer
   approximation such as "longer side plus half the shorter side" (Open questions). Each spec that
   owns a rule names the form it uses.
   - Backing: `measurable` for the range test's lengthening by *β* of a building target, for the
     "at most" comparison, for the absence of lengthening from a building attacker (runs j and k) and
     for the absence of lengthening for an arcing projectile (runs l to n), all in scenario C2.
     `unmeasured` for the inventory itself (a list of the original's rules; each owning spec
     measures its own rule)
   - Confidence: high for the range test's lengthening and comparison; medium for the building
     attacker (its source position is supplied by the firing rule, which the combat specs own), for
     the arcing projectile (the arc and height adjustments of the combat specs are kept out of the
     runs, and the chosen weapon may supply another source position) and for the inventory

   **Quirk (one-sided building reduction).** *D*(*A*, *B*) is reduced only when the *second* object
   *B* is a building. So the distance from a unit to a building is shorter than the distance from the
   building to the unit, and two buildings measure each other only from the second one's edge.
   Which object a rule puts second is owned by the spec of that rule. Our engine keeps this
   (`quirks.one_sided_building_distance`).

### Direction

8. **Direction from one point to another.** The direction of *Q* from *P* is computed by Formulas
   F3. In words: the ratio of the two map-axis differences selects an entry of a 4097-entry
   arctangent table at a fixed step of the tangent, and the resulting angle is scaled to facing
   units with a slightly short scale and truncated. Consequences that every implementation must
   reproduce:
   - (a) The four axis directions come out as 0 (map north), 16383 (map east), 32767 (map south) and
     49153 (map west), not 0, 16384, 32768 and 49152.
   - (b) Directions are quantised by the table step, so some facings are never the direction of
     any point. Near map east and map west the step is about 254 facing units: every point with
     |Δ*y*| < 0.0244 × |Δ*x*| has the same direction as the axis on its side. That is 16383 on
     both sides of map east, but 49153 just north of map west and 49150 just south of it. Near map
     north and map south neighbouring table entries are very close in angle, but the table ends at
     a tangent of about 100, which leaves a gap of about 104 facing units beside each of these two
     axes. Every point with |Δ*y*| more than about 100 × |Δ*x*| has the
     direction of the axis on its side: 0 on both sides of map north, but 32767 just east of map
     south and 32766 just west of it. The nearest directions beside each axis, and the facings
     that no direction ever takes (both sides of the axis together, the axis itself included), are:

     | Axis | Nearest directions off the axis | Facings no direction takes |
     |---|---|---|
     | map north (0) | 104 to the east, 65432 to the west | 1 … 103 and 65433 … 65535 |
     | map east (16383) | 16128 to the north, 16638 to the south | 16129 … 16382 and 16384 … 16637 |
     | map south (32767 east side, 32766 west side) | 32662 to the east, 32871 to the west | 32663 … 32765 and 32768 … 32870 |
     | map west (49153 north side, 49150 south side) | 49408 to the north, 48895 to the south | 48896 … 49149, 49151, 49152 and 49154 … 49407 |
   - (c) The diagonal directions (|Δ*x*| = |Δ*y*|) are 8315 (north-east), 24451 (south-east),
     41082 (south-west) and 57221 (north-west), about 123 facing units off the exact eighths.

   It is used whenever a rule needs the facing towards another object or point, for example the
   turret or body facing towards a target (S-0206, with the reference points of claim 4) and the
   walking direction of infantry (S-0208, F7, which gives the same scale). Our engine
   reproduces it by default (`quirks.table_direction`).
   - Backing: `measurable` (scenario C1) for parts (a), (b) and (c), with a recorder that reads the
     turret facing (runs a, c and e for the axis values of parts (a) and (b), runs b and f for the
     table quantisation, run d for part (c)). The never-taken facings of part (b) follow from F3
     and are tested through it: a resting facing among them in any C1 run falsifies F3. The 50
     table exceptions of F3 are `measurable` through one of them, index 1024 (C1 run g). The other
     49 are `unmeasured`: each needs two objects whose map-axis differences have a ratio of about
     10:1 or steeper and fall in a narrow band. Of all pairs of cell centres up to 60 cells apart,
     only those 1 cell apart along the map *x* axis and 25 along the map *y* axis, or 2 and
     50, reach an exception index, and it is always 1024. They rest on the table itself.
   - Confidence: high for F3 and parts (a) to (c); medium for "whenever a rule needs" (some rules
     may compute directions differently, Open questions)

9. **8-directions and neighbour cells.** Moving one step in 8-direction *d* changes the cell by
   the neighbour step of the table in Constants (0 = map north, clockwise). The facing that belongs
   to 8-direction *d* is exactly 8192 × *d* (a driving vehicle turns to it before a step, S-0206
   claim 12). How a rule chooses the 8-direction towards a more distant cell is not settled here:
   it is owned by the spec of that rule (for example the path steps of S-0207 and S-0212).
   - Backing: `unmeasured` here: each rule that uses a neighbour step is measured by its own spec
     (for example the step directions of S-0207)
   - Confidence: high for the neighbour steps and the facing 8192 × *d*

### Facings

10. **Facings and coarse facings.** The original stores every facing as an integer 0 … 65535 with
    the zero and sense of the glossary (claim G1), at the full resolution of one facing unit: no
    rule was found that rounds a stored facing to a coarser step. A coarse facing is derived
    when it is needed by rounding to the nearest step, a half step rounding up (towards the
    larger step number), with wrap-around (Formulas F4). This rule gives S-0105's 32-step facing
    and the 8-direction of claim 9. It is not the only coarse facing: a spec that defines its own
    grouping of facings owns it. In particular the eight infantry facing groups of S-0107 (claim 3,
    F2) are not F4 with *n* = 8: they are built from the 32-step facing in sectors skewed 1024
    facing units counter-clockwise of the compass points (`quirks.infantry_facing_sector_skew`,
    S-0107). A facing given as a value *v* from 0 to 255, as in a map file, is
    *v* × 256 facing units (glossary, claim G1).
    - Backing: `measurable` (S-0206 scenario R2 tests the full resolution of a map facing through
      turn times; this answers the glossary's open question on claim G1)
    - Confidence: high

11. **Signed facing difference.** Whenever the original compares or interpolates two facings it
    uses the signed facing difference of Formulas F5. Of the two ways round, the shorter is taken. A
    difference of exactly half a turn is −32768, so a half turn is made counter-clockwise (S-0206).
    - Backing: `measurable` (S-0206 scenario R1)
    - Confidence: high

## Formulas

All quantities are exact integers unless the formula says otherwise.

**F1. Table square root of an integer** (claims 5 and 6). *r*(0) = 0. For *n* ≥ 1 let
*z* = ⌊⌊log₂ *n*⌋ / 2⌋, the largest integer with 4^*z* ≤ *n*, so that *n* / 4^*z* lies from 1
(included) to 4 (excluded). Then

> *r*(*n*) = 2^*z* × trunc₂₄(√(trunc₁₄(*n* / 4^*z*))).

In words: *n* / 4^*z* is truncated toward zero to 14 significant binary digits, and its exact root
to 24 significant binary digits. Both truncations are exact rational operations: trunc₁₄(*n* / 4^*z*)
is a multiple of 2⁻¹³ (of 2⁻¹² when *n* / 4^*z* ≥ 2), and trunc₂₄ of a root between 1 and 2 is the
largest multiple of 2⁻²³ whose square is at most the radicand, which an integer square root gives
exactly. The distance term is ⌊*r*(*n*)⌋. This is the same
function as S-0009's (claim 19) and S-0105's (F1) for an integer input, and it already includes the
truncating conversion of *n* to single precision that the original applies to *n* (S-0205, number
domains). Units: *n* in square leptons, *r* in leptons.

Dependence on the rounding mode (S-0205, `quirks.chop_rounding`; this spec defines no switch of its
own for it): the only part of F1 that the mode affects is the conversion of *n* to single
precision, which is exact for *n* below 2²⁴ (distances below 4096 leptons). With the switch off, a
larger *n* gives ⌊*r*(*n*′)⌋ instead of ⌊*r*(*n*)⌋, where *n*′ is *n* rounded to a single-precision value by the rule of S-0205 for that mode (ties
included), and F1's truncation to 14 significant digits applies to *n*′. The result
differs for roughly 1 in 2000 squared distances between 16 and 512 cells, by one step of the
truncation to 14 significant digits.

**F2. Distance between objects** (claim 6). With reference points *p*_A and *p*_B (claim 4):

> *n* = (*x*_B − *x*_A)² + (*y*_B − *y*_A)² [+ (*z*_B − *z*_A)² for the spatial form],
> *D*(*A*, *B*) = max(0, ⌊*r*(*n*)⌋ − *β*(*B*)), *β*(*B*) = 64 × (*W*_B + *H*_B) for a building, else 0.

Integer leptons. In the original the squares are summed in double precision, which is exact for
every *n* below 2⁵³.

**F3. Direction** (claim 8). For points *P* and *Q* (positions or reference points, in leptons) let
*X* = *x*_Q − *x*_P and *Y* = *y*_P − *y*_Q (note the sign: *Y* grows towards map north). The
constants (see Constants) are *σ* = 3276705 / 2²⁷ (table step), *h* = 13176795 / 2²³
(single-precision π/2), *c*₁ = 884279719003555 / 2⁴⁹ (double-precision π/2),
*π*_d = 884279719003555 / 2⁴⁸ (double-precision π) and *c*₂ = −5733986146512111 / 2³⁹ (the
double-precision value nearest −32767/π, about −10430.060040584269).

*Table index* *i*, defined for *X* ≠ 0:

> *i* = ⌊2²⁷ × |*Y*| / (3276705 × |*X*|)⌋ (exact integer division).

*Table angle* *A* = *T*(*i*) for *i* ≤ 4096, and *A* = *h* for *i* > 4096. For the purpose of F3,
*T*(*i*) may be taken as arctan(*i* × *σ*) truncated toward zero to single precision,
trunc₂₄(arctan(*i σ*)), with the correction noted under "The original's table and the exceptions"
below. This is a surrogate that gives the original's facings exactly; it is not the original's
table itself (see below).

*Angle* *θ* (radians, counter-clockwise from map east as on a map with *Y* up), by the signs of *X*
and *Y*:

| | *Y* > 0 | *Y* = 0 | *Y* < 0 |
|---|---|---|---|
| *X* > 0 | *A* | *A* | −*A* |
| *X* = 0 | *h* | 0 | −*h* |
| *X* < 0 | trunc₅₃(*π*_d − *A*) | trunc₅₃(*π*_d − *A*) | −trunc₅₃(*π*_d − *A*) |

*Facing*:

> *φ* = trunc(trunc₅₃(trunc₅₃(*θ* − *c*₁) × *c*₂)) mod 65536,

with trunc toward zero to an integer and mod the non-negative remainder. Unit: facing units.

Number domain in the original: every floating-point result of F3 is rounded toward zero (the
round-toward-zero mode; S-0205, number domains). *X* and *Y* are single-precision values (exact
here); the quotient *Y* / *X* and the quotient of that by *σ* are double-precision values; the
conversion of the latter to the integer *i* truncates. The integer formula for *i* gives the same
index for every |*Y*| below 2²⁴ leptons, so it covers every pair of map points. trunc₂₄ is exact
for every table entry: no value arctan(*i σ*) lies within 10⁻⁴ of a single-precision step of a
boundary, so any correctly rounded double-precision arctangent gives the same *T*(*i*).

**The original's table and the exceptions.** The original's 4097 table angles are single-precision
values close to, but not equal to, trunc₂₄(arctan(*i σ*)): 4067 of them are smaller, none is
larger, and the largest gap is about 1.55 × 10⁻⁶ radians (13 single-precision steps near π/2). No
closed formula for them is known. For direction facings the gap matters only at the 50 indices
*i* listed below: there the facing *φ* comes out one facing unit nearer to map east (when *X* > 0)
or map west (when *X* < 0) than the surrogate formula gives, in all four quadrants. For every
other index, the surrogate gives the original's facing exactly. Any other use of the raw angle
(outside this spec) needs the original's table values, not the surrogate.

> 441, 486, 789, 810, 842, 993, 1024, 1115, 1148, 1183, 1203, 1405, 1419, 1443, 1504, 1542, 1582,
> 1618, 1810, 1907, 2015, 2044, 2094, 2136, 2202, 2225, 2400, 2513, 2605, 2621, 2670, 2704, 2721,
> 2848, 3030, 3096, 3312, 3364, 3418, 3502, 3531, 3683, 3715, 3748, 3781, 3849, 3884, 3956, 3993,
> 4069.

Dependence on the rounding mode: none. F3 gives the same facing for every index and quadrant with
`quirks.chop_rounding` on or off (S-0205), given the same table values.

Test values (Δ*x* = *x*_Q − *x*_P, Δ*y* = *y*_Q − *y*_P, so *Y* = −Δ*y*):

| Δ*x* | Δ*y* | *i* | *φ* |
|---|---|---|---|
| 0 | −1 | — | 0 |
| 1 | 0 | 0 | 16383 |
| 0 | 1 | — | 32767 |
| −1 | 0 | 0 | 49153 |
| −1000 | 1 | 0 | 49150 |
| −1 | 200 | 8192 | 32766 |
| 1 | −1 | 40 | 8315 |
| 1 | 1 | 40 | 24451 |
| −1 | 1 | 40 | 41082 |
| −1 | −1 | 40 | 57221 |
| 2560 | −256 | 4 | 15368 |
| 256 | −2560 | 409 | 1041 |
| 300 | −200 | 27 | 10304 |
| −700 | 1300 | 76 | 37922 |
| 1 | −100 | 4096 | 104 |
| 1 | −200 | 8192 | 0 |
| 9 | −97 | 441 | 966 (the formula without the exception gives 965) |

**F4. n-step facing** (claim 10). For *n* = 2^*b* (1 ≤ *b* ≤ 8) and a facing *f*:

> *f*ₙ = ⌊*f* × *n* / 65536 + ½⌋ mod *n*.

Exact rational arithmetic: *f* × *n* / 65536 is rounded to the nearest integer with halves rounded up
(towards the larger step number), and the step *n* wraps to 0. Unit: steps of 65536 / *n* facing
units. For *n* = 8: ⌊*f* / 8192 + ½⌋ mod 8; for *n* = 32: ⌊*f* / 2048 + ½⌋ mod 32, which gives the
same values as S-0105's formula; for *n* = 256: ⌊*f* / 256 + ½⌋ mod 256. Back from a step:
facing = *f*ₙ × 65536 / *n*, an exact integer.

**F5. Signed facing difference** (claim 11).

> *δ*(*a*, *b*) = ((*b* − *a* + 32768) mod 65536) − 32768.

Integers, mod the non-negative remainder. Range −32768 … 32767.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Cell edge | 256 | leptons | Glossary |
| Height level | 104 | leptons of world height | Glossary, S-0101 |
| Building distance reduction | 64 | leptons per foundation cell of width plus height | Claim 6 |
| Building reference shift | 128 | leptons per foundation cell beyond the first, along each axis | Claim 4 |
| Table root exact range | 16641 | square leptons (n below it) | Claim 5 |
| Arctangent table entries | 4097 | entries (index 0 … 4096) | F3 |
| Arctangent table step *σ* | 3276705 / 2²⁷ ≈ 0.024413354694843292 | tangent units per index | F3 |
| Single-precision π/2 *h* | 13176795 / 2²³ ≈ 1.5707963705062866 | radians | F3 |
| Double-precision π/2 *c*₁ | 884279719003555 / 2⁴⁹ ≈ 1.5707963267948966 | radians | F3 |
| Double-precision π *π*_d | 884279719003555 / 2⁴⁸ ≈ 3.141592653589793 | radians | F3 |
| Direction scale *c*₂ | −5733986146512111 / 2³⁹ ≈ −10430.060040584269 | facing units per radian | F3; nearest double to −32767/π |
| Axis directions | 0, 16383, 32767, 49153 | facing units | Claim 8 |
| 8-direction step | 8192 | facing units | Claim 9 |
| Neighbour steps of 8-directions 0 … 7 | (0, −1), (1, −1), (1, 0), (1, 1), (0, 1), (−1, 1), (−1, 0), (−1, −1) | cells (Δx, Δy) | Claim 9; 0 = map north, clockwise |

## State and ordering

No state of its own. Distances and directions are computed from the positions as they are at the
moment a rule asks for them; the order of the rules within a tick is S-0203's.

## Edge cases and quirks

- **`quirks.table_root_distance`** (default: on, the original). Distances use the table square
  root (claim 5, F1). Off: ⌊√*n*⌋, the exact integer square root. The switch is owned here and
  covers both the object distances and the farthest-sum start-position rule of S-0009, which
  has no switch of its own.
- **`quirks.table_direction`** (default: on, the original). Directions follow F3 exactly: the
  tabulated arctangent with its index step and the 50 table exceptions, the single-precision π/2 on
  the north–south axis, and the scale −32767/π with truncation, so that the axis directions are 0,
  16383, 32767 and 49153 and the diagonals are about 123 units off (claim 8). Off: the direction is
  the exact angle, φ = round(arctan2(*X*, *Y*) × 65536 / (2π)) mod 65536, with arctan2(*X*, *Y*) the angle
  clockwise from map north and halves rounded up, so the axis directions are 0, 16384, 32768 and
  49152 and the diagonals 8192, 24576, 40960 and 57344.
  The switch also covers the reverse scale, from a facing back to an angle, wherever a rule uses
  the same short scale: the walking step of infantry (S-0208 F3), which turns a facing *f* into
  the angle θ = (*f̂* − 16383) × (−2π / 65534), with *f̂* = *δ*(0, *f*) (F5), the facing as a value
  from −32768 to 32767. Off, that angle is θ = (*f̂* − 16384) × (−2π / 65536), with the floating-point
  rules S-0208 F3 gives. One switch covers both directions of the scale, so that a facing made by
  F3 and turned back into an angle lands on the axis it came from with the switch on or off; a
  mixed setting would put exact facings one facing unit off.
- **`quirks.one_sided_building_distance`** (default: on, the original). The building reduction
  *β* applies only to the second object of *D*(*A*, *B*) (claim 7). Off: *β*(*A*) + *β*(*B*) is
  subtracted, so the distance between two objects is the same in both orders. The weapon range
  test's lengthening by a building target's *β* (claim 7) is not a use of *D*; the combat specs
  own it, and this switch does not change it.
- **Negative positions.** Positions west or north of the map's top corner truncate toward cell 0
  (claim 2). This is the original's behaviour, and our engine keeps it; no switch, since no legal
  cell is affected.
- **Half turns.** The half-turn convention of claim 11 is not a quirk switch; S-0206 describes its
  consequence for turning.

## Import mapping

This spec reads no data key of its own. It uses the foundation size of building types, which their
own specs import. One key is listed so that implementers know this spec ignores it:

| File, section | Key | Concept | Parse form | Absent |
|---|---|---|---|---|
| rules file, a building type's section | `Bib` | the bib flag (Terms) | boolean (S-0003) | no (flag off) |

Distance, direction and the range test ignore the flag (claims 4, 6 and 7). The structure specs own
what else it does.

## Measurement scenarios

All scenarios run under the *measurement setup* of the glossary. Where a scenario needs a turn rate
or a weapon range that stock data does not have, it names a test rules file (a loose copy of the
rules file with only the fields the scenario names changed), removed after the run. Turn-time
arithmetic is S-0206's (F1 there).

**C1. Directions through the turret facing (claim 8).**
- *Setup:* a purpose-made flat map, one human player, one computer player far away. A recorder reads
  the attacker's turret facing every tick. A named test rules file gives a turn rate of 1 to one stock
  vehicle type that has a turret (S-0206 Import mapping), sets the range field of that vehicle's
  weapon to 26 cells (the conversion of the range field belongs to the combat specs) and makes no
  other change. The weapon is chosen to have no minimum range. That vehicle is
  pre-placed by the map with facing value 0 on cell (*x*, *y*), and its turret facing is checked to
  be 0 at the start. An unarmed enemy vehicle owned by the computer player, which the map leaves
  without orders, is pre-placed on a cell for each run:
  - run a: (*x* + 10, *y*), due map east,
  - run b: (*x* + 10, *y* − 1), ten cells east and one north (Δ*x* = 2560, Δ*y* = −256),
  - run c: (*x*, *y* + 10), due map south,
  - run d: (*x* + 3, *y* − 3), on the north-east diagonal,
  - run e: (*x* − 10, *y*), due map west,
  - run f: (*x* − 10, *y* + 1), ten cells west and one south (Δ*x* = −2560, Δ*y* = 256),
  - run g: (*x* + 1, *y* − 25), one cell east and 25 north (Δ*x* = 256, Δ*y* = −6400, table index
    1024, one of the 50 exceptions of F3).
  The farthest target (run g) is about 25.02 cells away, inside the 26-cell range, so the vehicle
  does not move. The measurement setup reveals the map, so every target is visible.
- *Procedure:* order the attack and record until the turret facing stops changing.
- *Observable:* the turret facing at rest, the number of ticks over which it changes, and the
  sense in which it changes.
- *Runs:* 1 per target position (no random draw is involved).
- *Expected:* resting turret facings 16383, 15368, 32767, 8315, 49153, 48135 and 417 (F3), reached
  clockwise in runs a to d and g in 63, 60, 127, 32 and 1 ticks (S-0206, F1), and
  counter-clockwise in runs e and f in 63 and 67 ticks (δ(0, 49153) = −16383, δ(0, 48135) =
  −17401).
- *Falsified if:* any resting facing differs from the expected one; in particular 16384, 15344,
  32768, 8192, 49152 or 48112 would be the exact angles. In run g, 416 (F3 without the exception)
  falsifies the exception list. Run g does not separate F3 from the exact angle, which also
  rounds to 417. Any resting facing among the facings
  that claim 8(b) says no direction takes falsifies the table quantisation. A run c turned
  counter-clockwise would mean a direction of 32768 together with the half-turn convention of
  claim 11. A vehicle that moves invalidates the run (repeat runs a to f with the target nearer,
  keeping the same direction). If an attack cannot be ordered or carried out at run g's distance,
  the exception stays `unmeasured` for that reason.

**C2. Building reference point, building lengthening, arcing projectile and bib row (claims 4, 6 and 7).**
- *Setup:* a purpose-made flat map, one human player and one computer player far away. A named test
  rules file sets the range field of one stock vehicle type's weapon to 5 cells, which is 1280
  leptons (the conversion of the range field belongs to the combat specs; runs a and b check it),
  and, for run h only, sets the bib flag of the run's target building type (Import mapping). It
  makes no other change. The vehicle type has a turret. Its weapon has no minimum range, and its
  projectile does not fly in an arc and has no range change from height. The attacker is pre-placed
  by the map on cell (*x*, *y*) as an ordinary ground vehicle on open, flat ground, not inside a
  bunker, a transport or a building. Every target is on the same flat ground. These choices keep
  out the range adjustments that the combat specs own. One target per run is owned by the computer
  player and pre-placed by the map, with nothing else of that player within 20 cells. A target
  building is unarmed and is given by its top-left foundation cell. Every building type used has the
  bib flag off in stock data, and only run h's test rules file sets it. A non-building target is an
  unarmed vehicle. The map leaves every target without orders.
- *Procedure:* order the attacker to attack the target and watch it until it fires or moves, for at
  most 300 ticks.
- *Observable:* whether the attacker fires its first shot without leaving cell (*x*, *y*) ("fires"),
  or moves before it fires ("moves").
- *Runs:* 1 per row of the table (no random draw is involved).
- *Expected:* the "Expected" column. Δ is the difference from the attacker's position to the
  target's reference point (claim 4), in leptons, and the limit is 1280 + *β* (claim 6). The
  attacker fires exactly when ⌊*r*(Δ*x*² + Δ*y*²)⌋ ≤ limit. At these distances ⌊*r*⌋ equals the
  exact truncated root, so the table root does not affect the outcome. The last two columns give
  the outcome under two wrong hypotheses: "no lengthening" (no *β*), and "top-left" (measured from
  the building's top-left cell centre, with *β*). Run h's third hypothesis, "bib row counted" (the
  foundation taken as *W* × (*H* + 1), which moves the reference point 128 leptons towards map
  south and adds 64 to *β*), gives Δ = (1408, −512), ⌊*r*⌋ = 1498 and limit 1600, so "fires".

  | Run | Target (top-left cell) | Δ (*x*, *y*) | ⌊*r*⌋ | Limit | Expected | No lengthening | Top-left |
  |---|---|---|---|---|---|---|---|
  | a | vehicle on (*x* + 5, *y*) | (1280, 0) | 1280 | 1280 | fires | fires | fires |
  | b | vehicle on (*x* + 5, *y* + 1) | (1280, 256) | 1305 | 1280 | moves | moves | moves |
  | c | 1 × 1 building, (*x* + 5, *y* + 1) | (1280, 256) | 1305 | 1408 | fires | moves | fires |
  | d | 2 × 2 building, (*x* + 5, *y*) | (1408, 128) | 1413 | 1536 | fires | moves | fires |
  | e | 2 × 2 building, (*x* + 6, *y*) | (1664, 128) | 1668 | 1536 | moves | moves | fires |
  | f | 3 × 3 building, (*x* + 5, *y* − 1) | (1536, 0) | 1536 | 1664 | fires | moves | fires |
  | g | 3 × 3 building, (*x* + 6, *y* − 1) | (1792, 0) | 1792 | 1664 | moves | moves | fires |
  | h | 2 × 2 building with the bib flag set, (*x* + 5, *y* − 3) | (1408, −640) | 1546 | 1536 | moves | moves | fires |
  | i | the same building type as run h, flag not set, (*x* + 5, *y* − 3) | (1408, −640) | 1546 | 1536 | moves | moves | fires |

  Runs j and k test the building attacker of claim 7. The attacker is a stock defensive building
  with a 1 × 1 foundation, a turret and a weapon with the properties above, pre-placed on cell
  (*x*, *y*) for the human player, who has enough power for it to fire. The test rules file sets
  that weapon's range field to 5 cells. The target is an unarmed enemy vehicle, left without orders by
  the map, on (*x* + 5, *y*) in run j and on (*x* + 5, *y* + 1) in run k. The observable is
  whether the building fires within 300 ticks of the target's placement or of an attack order.
  Expected: run j fires (1280 ≤ 1280) and run k does not (1305 > 1280).

  Runs l to n test the arcing projectile of claim 7. The attacker is a stock vehicle type with a
  turret whose weapon has a projectile that flies in an arc, found by the same choice as above. The
  test rules file sets that weapon's range field to 5 cells and, if the weapon has a minimum range,
  sets that field to 0 (both conversions belong to the combat specs). The attacker, the flat ground,
  the placement and the observable are as in runs a to i. The targets are unarmed:
  run l a 1 × 1 building on (*x* + 5, *y* + 1) (Δ = (1280, 256), ⌊*r*⌋ = 1305, limit 1280 without
  lengthening and 1408 with it); run m a 2 × 2 building on (*x* + 5, *y*) (Δ = (1408, 128), ⌊*r*⌋ =
  1413, limit 1280 without lengthening and 1536 with it); run n, a control, a vehicle on (*x* + 5, *y*)
  (Δ = (1280, 0), 1280 ≤ 1280). Expected: runs l and m "moves" and run n "fires". Without a
  lengthening, the first two are out of range; with it, both would fire.
- *Falsified if:*
  - runs c, d and f give "moves": the range is not lengthened by *β*.
  - runs e and g give "fires" (with d and f also "fires"): the building is measured from its top-left
    cell centre, not from its foundation's centre.
  - run h gives "fires" while run i gives "moves": the bib row is counted.
  - run k gives "fires": a building attacker's range is lengthened by its own foundation.
  - run l or m gives "fires": an arcing projectile is lengthened by *β* of a building target too.
    If run n gives "moves", the setup (range conversion, source position or the weapon's own
    adjustments) differs from what runs l and m assume, and they cannot be read. If the game has no
    stock weapon with an arcing projectile that a test rules file can give these properties, the
    arcing part of claim 7 stays `unmeasured` for that reason.
  If run a gives "moves" or run b "fires", the range-field conversion, the source position or the
  "at most" comparison differs from what this scenario assumes, and runs c to n cannot be read. For
  run a the distance equals the range exactly, so "moves" there with "fires" at (*x* + 4, *y*) points
  to a strict comparison.
- *Note:* the source position of the range test is supplied by the firing rule (claim 7). For a
  ground vehicle the scenario assumes it is the vehicle's position, and for a 1 × 1 building its
  reference point. Runs a, b, j and k check this assumption on targets that are not buildings.

**C3. Table root shortfall (claim 5).**
- *Setup:* a purpose-made flat map; one stock infantry type of the human player and one enemy
  infantry of the computer player. A named test rules file sets the attacker's weapon range field
  to half a cell, which is 128 leptons (the conversion of the range field belongs to the combat
  specs), and changes nothing else. Infantry stand at sub-cell positions, so their positions need not
  be cell centres. The two soldiers are placed (with the sub-cell positions of the infantry specs) so
  that their positions differ by exactly 129 leptons along one map axis and 0 along the other, on
  flat ground at the same height.
- *Procedure:* order the attack while the attacker is held in place (guard order, no movement).
- *Observable:* whether it fires without moving.
- *Runs:* 1 per axis (x and y).
- *Expected:* it fires: the squared distance is 16641 and ⌊*r*(16641)⌋ = 128, which is within a
  range of 128 leptons.
- *Falsified if:* it does not fire, while it does fire in a control run at a separation of 128
  leptons (the exact distance 129 is out of range).
- *Note:* the scenario assumes that the range test's source position, which the firing rule supplies
  (claim 7), is the attacker's own position. The control run at a 128-lepton separation checks
  only that the attacker can fire at that range at all. It does not rule out a source position a
  lepton or two away from the attacker's position. C2 runs a and b check the assumption for
  vehicles, not for infantry. The scenario also depends on infantry sub-cell positions and on the
  range-field conversion, which other specs own. If no placement gives an exact 129-lepton
  separation, the claim stays measurable only in principle, and the table root rests on F1 alone.

## Open questions

- Whether any game rule measures distance with an integer approximation (for example the longer
  side plus half the shorter side) instead of the table square root. None was found, but the search
  was not exhaustive.
- Whether every rule that needs a direction uses F3. A few rules (for example some projectile and
  locomotor paths) were not traced and may compute an angle differently; the specs that own them
  should say.
- Which rules use the planar and which the spatial distance, for each rule: owned by the specs of
  those rules (claim 7). The weapon range test in particular belongs to the combat specs.
- How rules choose the 8-direction from one cell towards a distant cell (claim 9). The one rule
  found that does this from the direction of claim 8 belongs to the computer player's team
  logic and reports a rotated direction number, so it was not generalised; the path and movement
  specs should say what their rules do.

## Questions for the project lead

- Should our engine ship the 4097 arctangent table values as engine-constant data instead of the
  surrogate and exception list of F3? The surrogate with the 50 exceptions gives identical
  facings, but other users of the same angle (turret and projectile aiming, matrices and other
  locomotor rules, outside this spec) take the raw angle, and there the surrogate differs from the
  original's value at 4067 of the 4097 entries. Those specs will need the table values
  themselves, which no closed formula is known to reproduce.
