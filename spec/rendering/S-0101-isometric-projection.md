---
id: S-0101
title: Isometric projection, height lift, view position and scroll limits
topic: rendering
revision: 3
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-09
measurements: []
requests: []
supersedes: []
---

# S-0101: Isometric projection, height lift, view position and scroll limits

## Summary

How a world position (leptons, with height) becomes a pixel position in the battlefield view, how
terrain, overlays and objects are lifted by height, how the view position is chosen and limited by the map's
local area, and how a screen pixel is turned back into a ground position or a cell for the cursor.
Every integer step is exact; the two calculations the original does in floating point are named,
with their rounding direction, and each is given an exact equivalent.

## Terms

Terms used as defined in the shared glossary: *cell*, *lepton*, *height level*, *world height*,
*local area*, *map size*, *object*, *render layer*, *building*, *aircraft*, *infantry*, *vehicle*,
*drawn frame*, *tick*, *quirk switch*, *engine constant*, *measurement setup*, *backing*,
*confidence*, *round-toward-zero mode*. The terms below are specific to this spec.

| Term | Meaning |
|---|---|
| trunc₀ | Truncation toward zero: trunc₀(2.7) = 2, trunc₀(−2.7) = −2. |
| rz<sub>d</sub>(*v*) | *v* rounded toward zero to a double-precision value (F2, F6). |
| rz<sub>s</sub>(*v*) | *v* rounded toward zero to a single-precision value (F6). |
| world pixel (*q<sub>x</sub>*, *q<sub>y</sub>*) | Integer pixel position in an unbounded plane that contains the whole map, *q<sub>x</sub>* growing to the right and *q<sub>y</sub>* downwards. The battlefield view shows a window of it. |
| battlefield view | The part of the game screen that shows the map, without the sidebar and the strip above it. |
| view size (*W*, *H*) | Width and height of the battlefield view in screen pixels, both positive integers. |
| view pixel (*v<sub>x</sub>*, *v<sub>y</sub>*) | A pixel position relative to the battlefield view's top-left pixel, which is (0, 0). Pixels inside the view have 0 ≤ *v<sub>x</sub>* < *W* and 0 ≤ *v<sub>y</sub>* < *H*. |
| view centre | The world pixel the battlefield view is centred on. It is the view's only position state. |
| view origin *O* | The world pixel shown at view pixel (0, 0). World pixel = view pixel + *O*. |
| height lift *L(z)* | Number of pixels a point at world height *z* is drawn above its ground projection. |
| exact scale | 15 pixels per 104 leptons of height, rounded half up: ⌊15·*z*/104 + ½⌋ pixels for a world height of *z* leptons. Used only as a comparison for *L(z)*. |
| flat-ground cell under a pixel | The cell that F7 gives for a view pixel or world pixel: the cell whose ground would be drawn there if it were at height level 0, whatever its actual height. |
| probe rows | For a cursor at view pixel (*x*, *y*), the 180 view pixels (*x*, *y* + 1) … (*x*, *y* + 180) directly below it in its column (claim 10, F8). |
| ground box | The box in world leptons, given by F4a, that claim 6 tests an object's ground position against. |
| screen band | The band of view pixels, given by F4b, that claim 6 tests the view pixel of an object's position against. |
| animated terrain object, terrain object that is not animated | The two groups of terrain objects that S-0102 distinguishes, in S-0102's wording. Terrain objects that are not animated are painted in the cell pass (S-0102). Animated ones are drawn as objects, subject to the on-screen tests of claim 6 (the ground box only). Which terrain objects fall in each group is S-0102's to say, and S-0102 does not yet say it (Open questions). Until it does, the terrain-object parts of claim 6 apply to whichever objects that rule puts in each group. |
| cell pass, object pass | The two drawing passes that S-0102 distinguishes, in S-0102's wording. Which images each pass draws is S-0102's to say. Claim 6 names the ones this spec relies on. |
| bib (of a building) | A part of a building's image, in S-0102's wording. S-0102 defines it. This spec says only that the cell pass paints it (claim 6). |
| *M<sub>w</sub>* | The map width of the map size (S-0009 writes it *W*; here *W* is the view width). |
| (*V<sub>x</sub>*, *V<sub>y</sub>*, *V<sub>w</sub>*, *V<sub>h</sub>*) | The four numbers of the local area: left, top, width, height. As for the map size, horizontal numbers count cell widths (60 world pixels each) and vertical numbers count cell heights (30 world pixels each). |

## Behaviour

1. **Ground projection.** A world position (X, Y) in leptons at world height *z* is drawn at the world
   pixel given by F1. Moving one cell along +X moves the point 30 pixels right and 15 pixels down.
   One cell along +Y moves it 30 pixels left and 15 pixels down. A cell therefore occupies a diamond
   60 pixels wide and 30 pixels tall.
   - Backing: `measurable` (scenario M1a)
   - Confidence: high
2. **Rounding toward zero, not down.** Both divisions by 256 in F1 truncate toward zero. The world
   pixel column *q<sub>x</sub>* = 0 is the screen column of the X = Y diagonal. Positions with
   −256/30 < X − Y < 256/30 all land on column 0, a band twice as wide as every other column's.
   Away from the diagonal each column covers 256/30 leptons of X − Y. The projection is therefore not
   translation-symmetric across the diagonal. See quirk `quirks.projection_truncates_toward_zero`.
   - Backing: `measurable` (scenario M1b)
   - Confidence: high
3. **Object height lift.** An object at world height *z* (leptons) is drawn *L(z)* pixels higher than
   its ground projection, with *L* given by F2. *L* is not exactly 15 pixels per 104 leptons: it is
   computed in floating point with a slightly-too-small scale, a +1 correction from 728 leptons
   upwards, and truncation toward zero. For heights that are whole multiples of 104 leptons it equals
   15 × level for levels 0 to 19. From level 20 (2080 leptons) upwards it falls below that: 299
   instead of 300 at level 20, 503 instead of 505 at 3500 leptons. Compared with the exact scale
   (Terms), as follows from F2 and checked for every integer height in each range:
   - from 0 to 2079 leptons, *L(z)* differs from it by at most one pixel, in either direction, and
     only at heights that are not whole multiples of 104 leptons.
   - from 2080 leptons upwards, *L(z)* is never above it and is often below it: one pixel below at
     2080 leptons, equal at 2081 leptons, at most two pixels below anywhere up to 3500 leptons, and
     the largest shortfall over a range of heights grows with the height.
   - below 0 leptons, *L(z)* is never below it and is often above it, that is closer to zero,
     because the final step truncates toward zero: at most 4 pixels above from −3000 to −1 leptons,
     and up to 11 pixels above at −13 312 leptons (level −128).

   See quirk `quirks.height_lift_float_scale`.
   - Backing: `measurable` (scenario M2, at the sample heights it uses; the range bounds above
     follow from F2 and hold whenever it does)
   - Confidence: high
4. **Terrain height lift.** Terrain tiles are lifted by exactly 15 pixels per height level (integer,
   no rounding). Objects use *L(z)* instead. For whole levels the two agree up to level 19. An object
   standing on a cell at level 20 or above is drawn one or more pixels lower than the tile's surface:
   one pixel at level 20, two at level 34. An object standing on a cell below level 0 is drawn
   higher than the tile's surface: one pixel at levels −1 to −6, two at levels −7 to −19, and up to
   11 pixels at level −128. Between whole levels (objects on slopes, climbing or falling) the
   comparison with the exact scale in claim 3 applies: at most one pixel either way from 0 to
   2079 leptons. From 2080 leptons of world height upwards, flying objects are never drawn higher
   than the exact scale would put them, and at many heights they are drawn lower: one pixel lower
   at 2080 leptons, while at 2081 leptons the two agree.
   - Backing: `measurable` (scenario M2; levels below 0 by M2d)
   - Confidence: high
5. **Screen position of a point.** The view pixel of a world point is its world pixel (F1) minus the
   view origin. All quantities are integers.
   - Backing: `measurable` (scenario M1a)
   - Confidence: high
6. **On-screen tests for objects.** In every drawn frame, an object in any render layer is drawn only
   if it passes the tests that apply to its kind:
   - **Ground box (every kind).** The ground position (X, Y) of the object must lie inside a box in
     world leptons derived from the view origin (F4a). For a building the position tested is the
     centre of the top-left cell of its foundation, for an animation its current position, and for
     every other object its own position. The test ignores height. On screen the box is a diamond
     whose top corner is *H* pixels above the view's top edge, at the view's centre column. Its upper
     edges fall one pixel for every two pixels sideways, so at the view's left and right edges they
     lie about *H* − *W*/4 pixels above the view. Its lower edges lie at least about *H* + 1.25·*W* pixels
     below the view's bottom edge, and its side corners lie more than 3·*H* pixels beyond each side
     edge.
   - **Screen band (vehicles, infantry, aircraft, projectiles, and every kind not listed below).**
     The view pixel of the object's position, including its height lift, must lie within 360 pixels
     left or right of the view and 180 pixels above or below it, bounds inclusive (F4b). Animations,
     animated terrain objects and the parts of a building drawn as objects skip this test: for them
     the ground box is the only on-screen test. Whether the band decides the markers drawn with a
     selected building is an open question.
   - **Not decided by these tests.** A building's body, shadow and bib, and terrain objects that are
     not animated, are painted in the cell pass (S-0102), so they are drawn whenever any part of
     them is inside the view, wherever the building's position lies relative to the band or the box.
     The parts of a building drawn as objects are those S-0102 places in the object pass, such as
     voxel turrets.

   An object that fails a test is not drawn at all, together with everything drawn as part of it,
   its shadow included. For a building this applies only to its parts drawn as objects, not to what
   the cell pass paints. An image whose position is outside the band is therefore not drawn even if
   part of it, or its shadow, would be visible. See quirk `quirks.screen_band_tests_position_only`.
   - Backing, by part of the claim:

     | Part | Backing |
     |---|---|
     | The band's bounds, and that aircraft use it | `measurable` (scenario M4a) |
     | Infantry use it | `measurable` (scenario M4c, if its precondition holds; otherwise unconfirmed) |
     | Vehicles use it | `measurable` (scenario M4d, if its precondition holds; otherwise unconfirmed) |
     | Projectiles use it | `measurable` (scenario M4e) |
     | A building's body is drawn whatever its position relative to the band | `measurable` (scenario M4b, if its precondition holds; otherwise unconfirmed) |
     | The parts of a building drawn as objects skip it | `measurable` (scenario M4b, only if the chosen building has such a part inside the view in run 1; otherwise unconfirmed) |
     | Animations and animated terrain objects skip it | `unmeasured` (reason below) |
     | Every other kind uses it | `unmeasured` (reason below) |
     | The ground box | `unmeasured` (reason below) |

     Reasons. The band makes a visible difference only for an image, or a shadow, that reaches more
     than 180 pixels above or below its position, or more than 360 pixels to one side. The
     measurement setup cannot place an animation at a chosen position, and no stock terrain object
     or animation is known to have an image that large. The other kinds (particles, voxel
     animations and similar effects) are placed by the game itself, never by the player, and are
     drawn close to their position. The ground box's edges lie hundreds of pixels outside the view at
     any common view size, so no stock image positioned outside the box can reach into the view. An
     unmodified install cannot report culling decisions directly.
   - Confidence: high for the band, its bounds, which kinds use it and which skip it. High that a
     shadow is drawn as part of its object and shares its band test (scenarios M4a and M4c to M4e
     rely on it). Low that a hovering jumpjet's shadow is drawn on the ground below it; the
     preconditions of M4c and M4d check this, and it decides only whether those scenarios can test
     the band. Medium for the exact box (F4a).
7. **View origin from view centre.** The game positions the view by its centre. The view origin is
   the view centre minus half the view size, each half truncated (F3). Centring the view on an object
   uses the world pixel of the object's position including its height lift (F1), so an object on high
   ground or in the air appears at the view's centre, not the ground point below it.
   - Backing: `measurable` (scenario M3c)
   - Confidence: high
8. **Scroll limits.** Every time the view centre is set, by scrolling, by centring on an object or at
   the start of the game, each component of the requested centre is limited by F5, independently of
   the other component. On each axis, with the minimum and maximum of F5, the result is:

   | Requested value | Result |
   |---|---|
   | below the minimum | the minimum |
   | at or above the minimum, and above the maximum | the maximum |
   | at or above the minimum, and not above the maximum | the requested value |

   When the local area is at least as large as the view on that axis (maximum ≥ minimum), this is an
   ordinary clamp. Horizontally, the view's left and right edges can reach the local area's left and
   right edges and no further. Vertically, the view's top edge can go 75 pixels above the top of the
   local area's rows, and its bottom edge 60 pixels below their bottom, so the vertical scroll range
   is the local area's height plus 135 pixels. Both margins are fixed. A scroll that would leave this
   range stops at its edge.
   When the view is larger than the local area on that axis (maximum < minimum), a request below the
   minimum gives the minimum and **every other request gives the maximum**. Horizontally that means:
   centring on a point left of the minimum aligns the view's left edge with the local area's left
   edge, and centring on any other point aligns the view's right edge with the local area's right
   edge. See quirk `quirks.oversized_view_clamp`.
   - Backing: `measurable` (scenario M3a, M3b)
   - Confidence: high for the formulas and the oversized-view result. Medium for the mapping of the
     map file's numbers onto the rectangle roles, which follows S-0009 and is not yet measured (M3a
     would confirm it).
9. **Screen to ground.** Turning a view pixel back into a ground position uses F6 on its world
   pixel. The result is at world height 0 and ignores terrain height. Its cell, by F7, is the
   flat-ground cell under the pixel, which is the cell under the cursor wherever no probe row
   qualifies (claim 10).
   - F6 is not an exact inverse of F1. Projecting its result back with F1 gives the original pixel,
     or one pixel above it, and in either case either the same column or one column closer to the
     screen column of the X = Y diagonal. It never lands below the pixel or farther from the diagonal.
     This follows from F1 and F6 and was checked for every world pixel with |*q<sub>x</sub>*| ≤ 1200
     and 0 ≤ *q<sub>y</sub>* ≤ 2400. See quirk `quirks.screen_to_ground_not_exact_inverse`.
   - Backing, by part of the claim:

     | Part | Backing |
     |---|---|
     | The cell partition of level-0 ground under the cursor (F7's pixel formula) | `measurable` (scenario M5) |
     | The lepton values of F6 inside a cell, and the round-trip bound above | `unmeasured` (reason below) |

     Reason. F6's point lies on the ground area of the input pixel or of a neighbouring pixel, so
     it is within one pixel of where an exact inverse would put it, and no part of the original is known to show the player a cursor position finer
     than a cell: orders and the placement outline are given per cell (claim 10), and the only
     other known use of F6, the ground box (F4a), is itself `unmeasured`. The round-trip bound is a
     consequence of F1 and F6 and holds whenever both do.
   - Confidence: high for the cell partition. High for the lepton values and the round-trip bound,
     which follow from the engine constant *a* and the stated rounding.
10. **Cell under the cursor on raised terrain.** For a cursor at view pixel (*x*, *y*), look at its
    probe rows, the 180 rows directly below it in its column. A probe row qualifies when the
    flat-ground cell under it, raised by 15 pixels per height level of that cell, reaches the cursor
    row. The cursor's cell is the flat-ground cell under the qualifying row farthest below the
    cursor. If no row qualifies, it is the flat-ground cell under the cursor pixel itself, whatever
    its height. F8 states this exactly. Only each cell's height level enters the rule, not the image
    of its tile.
    - Where every probed cell is at level 0 or below, no probe row qualifies, so the result there is
      always the flat-ground cell under the cursor pixel (F7).
    - Because only 180 rows are probed, the rule behaves as if a cell at level *h* > 0 were lifted by
      min(15*h*, 180) pixels: its flat rows moved up by that amount select it, unless a cell lower on
      the same screen column also reaches those rows. For levels 1 to 12 that is exactly its drawn
      lifted diamond. For level 13 and above the pick stops following the drawing: the cell is
      selected by the rows 180 above its flat rows, and the top 15*h* − 180 rows of its drawn diamond
      select cells behind it (for example the cell drawn above it on a level plateau). See quirk
      `quirks.cursor_probe_depth_180`.
    - Consequence at a cliff that faces the camera: the rows between the front plateau cell's lifted
      diamond and the low ground, where the cliff face is drawn, select that front plateau cell, not
      the low cell in front of the cliff. The low cell is selected only by its own flat rows.
    - Scope: this rule holds when none of the probed cells is part of a bridge. Near bridge cells the
      result can differ, and this spec does not specify it (see Open questions).
    - Backing: `measurable` (scenario M6; the level-0 case by M5)
    - Confidence: high
11. **Overlay height lift.** Overlays (walls, fences, ore and gem fields, crates, bridge parts and
    the like) and their shadows are lifted on the screen like terrain tiles: by exactly 15 pixels
    per height level of their own cell (integer, no rounding), never by *L(z)*.
    - A bridge deck follows the same rule. On the screen it is lifted only by the height level of
      its cell. The 4 extra levels at which the deck lies (glossary *bridge deck*; S-0102, claim 15)
      enter its depth (S-0102) and the world height of objects on it, but they are not an extra
      60-pixel lift of its image. The raise that the player sees therefore comes from the deck's
      image itself.
    - Consequence: an overlay stays aligned with its cell's tile at every level. On a cell at level
      20 or above, an object standing on the same cell is drawn one or more pixels lower than both
      (claim 4).
    - This claim fixes only how the lift depends on the cell's height level. Where on its cell an
      overlay's image is placed is not specified here: some overlay kinds and some deck images are
      drawn a few pixels higher than others, by fixed amounts that do not depend on the level (Open
      questions). The lift is added to that placement.
    - Backing, by part of the claim:

      | Part | Backing |
      |---|---|
      | Overlays, their shadows and bridge decks lift by 15 pixels per level of their cell, not by *L(z)* | `measurable` (scenario M2e; shadows only if the chosen overlay types draw one) |
      | A bridge deck gets no extra lift for its 4 levels | `unmeasured` (reason below) |

      Reason. With the placement on the cell left open, a fixed extra lift of the same 60 pixels for
      every deck would look the same as a deck image drawn 60 pixels higher. The part says only that
      an implementation must not add the 4 levels to the screen lift on top of an image that already
      shows the raise. It can be checked only together with a placement rule.
    - Confidence: high for the per-level lift of overlays, their shadows and decks. High that the
      deck's 4 levels do not enter its screen lift.

## Formulas

**F1. Projection** (integers. Inputs: X, Y, *z* in leptons. Output: world pixel):

> *q<sub>x</sub>* = trunc₀( (30·X − 30·Y) / 256 )
>
> *q<sub>y</sub>* = trunc₀( (15·X + 15·Y) / 256 ) − *L(z)*

The numerators are exact integers. They fit in 32 bits for every position on a map. Example: cell
(10, 4) has its centre at (2688, 1152), which projects to (180, 225) at height 0. A cell (x, y) has
its top corner (the corner topmost on the screen), world position (256x, 256y), at world pixel
(30(x − y), 15(x + y)). Cell corners and cell centres give numerators that are whole multiples of
256, so their projections have no fractional part to truncate.

**F2. Object height lift** (*z* in leptons, a signed integer. The original evaluates this in double
precision):

> *L(z)* = trunc₀( ( *z* · *k* + [*z* ≥ 728] ) + 0.5 )

- *k* = 0.14350360082660085 pixels per lepton, a double-precision engine constant (see Constants).
- [*z* ≥ 728] is 1 when *z* ≥ 728, else 0.
- Number domain and rounding: IEEE double precision, with a 53-bit significand. Write
  rz<sub>d</sub>(*v*) for *v* rounded **toward zero** to a double-precision value, as the original
  rounds throughout play (glossary *round-toward-zero mode*). In full, F2 is

  > *L(z)* = trunc₀( rz<sub>d</sub>( rz<sub>d</sub>( rz<sub>d</sub>(*z* · *k*) + [*z* ≥ 728] ) + 0.5 ) )

  The product and each of the two sums is rounded, and the grouping shown is part of the rule:
  0.5 is added to the rounded sum of the product and [*z* ≥ 728]. The final conversion truncates
  toward zero, so negative heights move toward 0: *L*(−104) = −14, not −15.
- The rounding mode of the three double-precision steps does not change *L(z)* at any height a map
  can produce: rounding to nearest (ties to even) at the product and at both sums, followed by the
  same final truncation, gives the same *L(z)* as rounding toward zero for every integer *z* from
  −300 000 to 300 000 leptons, checked with exact arithmetic for each step. Cell heights lie
  within levels −128 to 127 (S-0009), that is −13 312 to 13 208 leptons. An implementation may
  therefore use either mode at those three steps. The final conversion must truncate toward zero.
  The program-wide mode and its switch `quirks.chop_rounding` belong to S-0205 (number domains,
  planned). This spec defines no switch for it, and at every height a map can produce *L(z)* is
  the same whichever way that switch is set.
- *k* is (60 / *D*) × *s*, each step rounded to double precision, with 60 in pixels. Here
  *D* = 362.0386657714844 leptons is the single-precision value nearest to 256·√2 (exact:
  362.0386719675…), and *s* = 0.865897536277771 (dimensionless) is the slightly inexact single-precision value the original uses for sin 60° (exact: 0.8660254…).
  Both roundings give the same *k*. *k* is slightly below 15/104 mainly because of the formula
  itself: with the exact 256·√2 and sin 60°, (60 / (256·√2)) × sin 60° = 30·√(3/2) / 256 ≈
  0.1435248 pixels per lepton, already below 15/104 ≈ 0.1442308. Together, the single-precision *D*
  and *s* lower it only slightly further, to *k*.

Check values: *L*(0) = 0, *L*(3) = 0, *L*(4) = 1, *L*(103) = 15, *L*(104) = 15, *L*(727) = 104,
*L*(728) = 105, *L*(1456) = 210, *L*(1500) = 216, *L*(2000) = 288, *L*(2080) = 299 (not 300),
*L*(3000) = 432, *L*(3500) = 503, *L*(−1) = *L*(−3) = *L*(−4) = 0.

An implementation may tabulate *L* over the height range it supports, or evaluate F2 in double
precision with *k* as given, rounding either toward zero or to nearest at the three inner steps.

**F3. View origin** (integers. Centre and origin in world pixels, *W* and *H* in screen pixels):

> *O<sub>x</sub>* = centre<sub>x</sub> − trunc₀(*W* / 2),  *O<sub>y</sub>* = centre<sub>y</sub> − trunc₀(*H* / 2)

**F4a. Ground box** (integers. *G*(*q<sub>x</sub>*, *q<sub>y</sub>*) is F6 applied to a world pixel,
which may lie outside the view. *O* is the view origin in world pixels, *W*, *H* the view size in
screen pixels. X, Y and the X and Y components of *G* are in leptons):

> (X<sub>A</sub>, Y<sub>A</sub>) = *G*(*O<sub>x</sub>* + trunc₀(*W*/2), *O<sub>y</sub>* − *H*)
>
> X<sub>B</sub> = X component of *G*(*O<sub>x</sub>* + 2*W*, *O<sub>y</sub>* + trunc₀(*H*/2)),  Y<sub>C</sub> = Y component of *G*(*O<sub>x</sub>* − *W*, *O<sub>y</sub>* + trunc₀(*H*/2))
>
> inside ⇔ X<sub>A</sub> ≤ X < X<sub>A</sub> + 2·(X<sub>B</sub> − X<sub>A</sub>)  and  Y<sub>A</sub> ≤ Y < Y<sub>A</sub> + 2·(Y<sub>C</sub> − Y<sub>A</sub>)

The box follows the current origin in every drawn frame.

**F4b. Screen band** (integers. (*s<sub>x</sub>*, *s<sub>y</sub>*) is the view pixel of the
object's position: F1 including *L(z)*, minus *O*):

> −360 ≤ *s<sub>x</sub>* ≤ *W* + 360  and  −180 ≤ *s<sub>y</sub>* ≤ *H* + 180

**F5. Scroll limits for the view centre** (integers, all in world pixels):

> x<sub>min</sub> = trunc₀(*W*/2) + 30·(2·*V<sub>x</sub>* − *M<sub>w</sub>*)
>
> x<sub>max</sub> = x<sub>min</sub> + 60·*V<sub>w</sub>* − *W*
>
> y<sub>min</sub> = 15·(*M<sub>w</sub>* − 5 + 2·*V<sub>y</sub>*) + trunc₀(*H*/2)
>
> y<sub>max</sub> = y<sub>min</sub> + 30·*V<sub>h</sub>* + 135 − *H*

For one axis, with requested value *c*, minimum *m* and maximum *M*, the result is

> limit(*c*) = *m* for *c* < *m*,  limit(*c*) = min(*c*, *M*) for *c* ≥ *m*.

The result is (limit(centre<sub>x</sub>) with x<sub>min</sub>, x<sub>max</sub>; limit(centre<sub>y</sub>)
with y<sub>min</sub>, y<sub>max</sub>). The two axes do not affect each other. When *M* ≥ *m*, limit
is the ordinary clamp. When *M* < *m*, limit(*c*) = *m* for *c* < *m* and *M* for every *c* ≥ *m*.

With the centre at x<sub>min</sub>, the view's left edge is at world column 60·*V<sub>x</sub>* −
30·*M<sub>w</sub>* (the local area's left edge). At x<sub>max</sub>, the view's right edge is 60·*V<sub>w</sub>*
columns further right (the local area's right edge). At y<sub>min</sub>, the view's top edge is at
world row 15·(*M<sub>w</sub>* + 2·*V<sub>y</sub>*) − 75. At y<sub>max</sub>, its bottom edge is
30·*V<sub>h</sub>* + 135 rows below that.

**F6. Screen to ground** (world pixel (*q<sub>x</sub>*, *q<sub>y</sub>*) = view pixel + *O*):

> X = trunc₀( *a* · (*q<sub>x</sub>* + 2·*q<sub>y</sub>*) ),  Y = trunc₀( *a* · (2·*q<sub>y</sub>* − *q<sub>x</sub>*) ),  *z* = 0
>
> *a* = 4 473 959 / 1 048 576 = 4.26669979095459 leptons per pixel (exactly)

- *a* is nominally 128/30 but slightly larger. The second coefficient, nominally 128/15, is exactly
  2*a*, which is why the formula needs only *a*.
- Number domain: the formula is exact rational arithmetic followed by truncation toward zero. An
  integer implementation computes trunc₀(4 473 959 · *u* / 1 048 576) with a 64-bit product, where
  *u* = *q<sub>x</sub>* + 2·*q<sub>y</sub>* for X and *u* = 2·*q<sub>y</sub>* − *q<sub>x</sub>* for Y.
- The original computes it in floating point, with *a*, 2*a* and the pixel coordinates as
  single-precision values. Write rz<sub>d</sub> and rz<sub>s</sub> for rounding toward zero to
  double and to single precision. Its value is

  > X = trunc₀( rz<sub>s</sub>( rz<sub>d</sub>( *a*·*q<sub>x</sub>* + 2*a*·*q<sub>y</sub>* ) ) ),  Y = trunc₀( rz<sub>s</sub>( rz<sub>d</sub>( 2*a*·*q<sub>y</sub>* − *a*·*q<sub>x</sub>* ) ) )

  in which both products are exact. Rounding toward zero can never carry a value across an integer
  (every integer below 2²⁴ is representable), so the result equals the exact formula above.
- The rounding toward zero at those two steps is the program-wide *round-toward-zero mode*
  (glossary). Its switch, `quirks.chop_rounding`, belongs to S-0205 (number domains, planned), and
  this spec defines no switch for it. With that switch on (the default), F6 is the exact formula
  above. If that switch is off and the two steps round to nearest instead, some results change by
  one lepton: about 0.14 % of them in a sample of world pixels with |*q<sub>x</sub>*| ≤ 1500 and
  300 ≤ *q<sub>y</sub>* ≤ 3300, none of which changed the cell (F7).

**F7. Cell of a ground position and of a world pixel** (integers. X, Y in leptons, results in cell
indices):

> cell of (X, Y) = ( trunc₀(X / 256), trunc₀(Y / 256) )

This also truncates toward zero, so positions from −255 to 255 leptons on an axis all fall in cell
index 0 on that axis. Combined with F6, the flat-ground cell under world pixel (*q<sub>x</sub>*,
*q<sub>y</sub>*) is exactly

> ( trunc₀( (*q<sub>x</sub>* + 2·*q<sub>y</sub>*) / 60 ), trunc₀( (2·*q<sub>y</sub>* − *q<sub>x</sub>*) / 60 ) )

whenever |*q<sub>x</sub>* + 2·*q<sub>y</sub>*| and |2·*q<sub>y</sub>* − *q<sub>x</sub>*| are at most
128 818 (cell indices up to 2146). This was checked for every integer in that range. Beyond it the
slightly-too-large *a* starts to move boundaries. Consequence for cells whose two indices are both
at least 1: the pixels of cell (x, y) are those with 60x ≤ *q<sub>x</sub>* + 2·*q<sub>y</sub>* <
60(x + 1) and 60y ≤ 2·*q<sub>y</sub>* − *q<sub>x</sub>* < 60(y + 1). The cell's top corner pixel and
the pixels on its two upper edges belong to it. The pixels on its two lower edges belong to the
neighbouring cells. Index 0 on an axis is twice as wide: it takes every value of the corresponding
sum from −59 to 59, because the truncation is toward zero (see Negative ground positions).

**F8. Cell under the cursor** (integers. *c*(*r*) is the flat-ground cell of F7 for view pixel
(*x*, *r*), that is world pixel (*x* + *O<sub>x</sub>*, *r* + *O<sub>y</sub>*). *h*(*c*) is the
height level of cell *c*):

> *Q* = the set of rows *r* with *y* + 1 ≤ *r* ≤ *y* + 180 and *r* − 15·*h*(*c*(*r*)) ≤ *y*
>
> cursor cell = *c*(max *Q*) if *Q* is not empty, *c*(*y*) if *Q* is empty

Exact integer arithmetic, so no rounding applies beyond F7's. 180 is the cursor probe depth
(Constants) while `quirks.cursor_probe_depth_180` is on.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| leptons per cell | 256 | leptons | Along each map axis |
| cell centre displacement | 128 | leptons | Added to 256 × cell index |
| leptons per height level | 104 | leptons | Fixed |
| cell diamond | 60 × 30 | pixels | Width × height of one cell's footprint |
| terrain lift per level | 15 | pixels | Exact integer, terrain tiles only |
| object lift scale *k* | 0.14350360082660085 | pixels per lepton | Double precision. 15/104 would be 0.1442307… |
| lift correction threshold | 728 | leptons | 7 levels. Adds 1 before truncation |
| on-screen band | 360 horizontal, 180 vertical | pixels | Margins beyond the view on each side (F4b) |
| ground box anchor points | (trunc₀(*W*/2), −*H*), (2*W*, trunc₀(*H*/2)), (−*W*, trunc₀(*H*/2)) | pixels from the view origin | F4a. The box spans twice the distance from the first point to the other two |
| scroll top margin | 75 | pixels | The "− 5" term of y<sub>min</sub> (5 × 15) |
| scroll bottom extra | 135 | pixels | Added to the vertical range |
| inverse coefficient *a* | 4 473 959 / 1 048 576 (4.26669979095459) | leptons per pixel | Exact value of a single-precision constant |
| cursor probe depth | 180 | pixels | Probes at 1 … 180 rows below the cursor. The largest qualifying one decides (claim 10). Applies while `quirks.cursor_probe_depth_180` is on |

## State and ordering

The view has one state value, the view centre. Every request to move it (scrolling, centring on an
object, start of game) stores the limited centre of F5, never the requested one. The origin of every
drawn frame is derived from that stored centre by F3. Observable consequence: after scrolling hard
against an edge, scrolling back moves the view at once. No extra scrolling is needed to "unwind" the
part of the request that was cut off.

| State | Event | Result |
|---|---|---|
| view centre *c* | scroll or centre request to *c′* | The centre becomes *c′* limited by F5. The origin is F3 of that centre. The battlefield is redrawn from the new origin |

## Edge cases and quirks

- **`quirks.projection_truncates_toward_zero`** (default: on). The divisions by 256 in F1 truncate
  toward zero, so column 0 covers twice the usual range of positions (claim 2). Row 0 would
  too, but its negative half holds only positions outside every map. Alternative: floor
  division, which makes the projection exactly translation-invariant by whole pixels. With the quirk
  off, an object at a position with X < Y moves one pixel left wherever 30·(X − Y)/256 is not a
  whole number. Terrain tiles do not move, because cell corners and centres project without a
  fractional part (F1).
- **`quirks.height_lift_float_scale`** (default: on). Object height lift uses F2. Alternative: exact
  *L(z)* = ⌊15·*z*/104 + ½⌋ (the exact scale), which keeps objects standing on whole levels aligned
  with the terrain at every level. With the quirk on, flying units and units on very high terrain
  sit up to a few pixels lower than the exact scale would put them (for example 299 instead of 300
  pixels at 20 levels), and units below level 0 sit higher than it (claims 3 and 4).
- **`quirks.screen_to_ground_not_exact_inverse`** (default: on). F6 with the constant *a* and
  truncation. Alternative: an exact inverse that returns a point whose projection is the input pixel.
  With the quirk off, the cell under a pixel is still given by F7's pixel formula, so only the
  lepton position returned changes. No observable difference in play is known
  (claim 9).
- **Rounding mode.** F2 and F6 are the only floating-point steps evaluated during play (the
  derivation of *k* in F2 gives the same *k* in either mode). Both rely on the
  program-wide *round-toward-zero mode* (glossary), whose single switch `quirks.chop_rounding`
  belongs to S-0205 (number domains, planned). This spec defines no switch of its own for it. The
  mode does not change *L(z)* at any height a map can produce (F2). With the mode on, F6 equals its exact formula; rounding to
  nearest would move about 0.14 % of its lepton results by one lepton and no cell (F6).
- **`quirks.oversized_view_clamp`** (default: on). When the view is larger than the local area on
  one axis, F5 gives the minimum for requests below it and the maximum for every other request
  (claim 8). Alternative: centre the local area in the view on that axis, so that every request
  gives the same position.
- **`quirks.cursor_probe_depth_180`** (default: on). The cell under the cursor is found from at most
  180 probe rows below the cursor (claim 10), so for a cell at height level 13 or above the pick
  no longer follows its drawn lifted diamond. Alternative: probe at least 15 × (the highest height
  level on the map) rows, so that a cell at any level *h* behaves as lifted by exactly 15*h* pixels
  and the pick follows every drawn lifted diamond. On a map whose cells are all at level 12 or
  below, both settings give the same cell for every cursor pixel. Scenario M6b tests the default.
  The result its falsifier names (a lift of 210 px at level 14) is the result with the quirk off.
- **`quirks.screen_band_tests_position_only`** (default: on). The screen band (F4b) tests only the
  view pixel of the object's position, so an object of a band-tested kind, and its shadow, pop into
  view when that position enters the band, even if part of the image or shadow would have been
  visible earlier (scenarios M4a, M4c to M4e). Alternative: draw an object of a band-tested kind
  whenever any part of its image or its shadow would lie inside the view. The ground box still
  applies. With the quirk off, a high-flying object's shadow slides in from the view's edge instead
  of appearing suddenly.
- **Negative heights.** F2 truncates toward zero. A world height is negative wherever an object is
  below height level 0, for example on a cell that a map sets below level 0 (a map may use levels
  down to −128). From −4 leptons downwards, *L(z)* is one pixel smaller in magnitude than rounding the same
  bracketed value of F2 down would give (*L*(−104) = −14, not −15). From −1 to −3 leptons both give
  0. Compared with the exact scale, objects below level 0 are drawn higher than their terrain
  (claim 4). Covered by `quirks.height_lift_float_scale`.
- **Negative ground positions.** F7 truncates toward zero, so world positions from −255 to −1 leptons
  on an axis are reported in cell 0 of that axis, not cell −1. Such positions lie outside every map.

## Import mapping

| Source | Key | Feeds | Parse | Default when absent |
|---|---|---|---|---|
| map file, `[Map]` | `Size` = left, top, width, height | map size. Only the width *M<sub>w</sub>* enters F5 | four integers | as in S-0009 |
| map file, `[Map]` | `LocalSize` = left, top, width, height | local area *V<sub>x</sub>*, *V<sub>y</sub>*, *V<sub>w</sub>*, *V<sub>h</sub>* in F5 | four integers | as in S-0009 |
| map file, cell data | cell height | height level of each cell (terrain lift, cursor probe) | integer per cell | 0 |

F3 and F5 use the same view size *W*, *H*, as they do in normal play. *W* and *H* come from the
player's screen resolution minus the sidebar and the strip above the view. A later user-interface
spec, not yet written and without a number, will give them. They are not rules data.

## Measurement scenarios

All scenarios use the glossary's measurement setup: an unmodified install with no loose override
files and no engine add-ons, and stock data apart from the purpose-made test maps named here. Any
game speed, 800×600 unless stated. The observable is a screenshot or a recording of the battlefield
view. "Pixel position" is read relative to the view's top-left pixel. Rendering and the cursor pick
are deterministic, so **one run of each scenario is enough**. A second run is useful only to rule out
a reading mistake.

- **M1a — ground projection** (claims 1, 5). Setup: a flat purpose-made map, all level 0, clear
  terrain. Place a building with a 1×1 foundation, or another object drawn at its cell centre, in
  cells (20, 10), (10, 20), (15, 15), (16, 15) and (15, 16). Centre the view on (15, 15).
  Observable: pixel displacement between the images. Expected: (16, 15) is 30 px right of and 15 px
  below (15, 15). (15, 16) is 30 px left of and 15 px below it. (20, 10) is exactly 600 px right of
  (10, 20) and on the same row. Falsified if any displacement differs from these by one pixel or
  more.
- **M1b — truncation toward zero** (claim 2). Cell centres and infantry spots lie on a 128-lepton
  grid in X − Y, where F1 has no fractional part, so they cannot show the effect. Setup: the M1a map.
  A test vehicle whose speed is set, through a test-map override section, so low that it moves at
  most about 1.5 leptons per tick. Driving along a screen row, X − Y changes by √2 times its speed
  per tick, so each ordinary column (256/30 ≈ 8.5 leptons of X − Y) is held for at least about 4
  ticks. Order it to drive along one screen row across the X = Y diagonal, from cell (10, 20) to
  cell (20, 10). Record every tick (or every drawn frame, noting the tick of each) while it crosses
  the diagonal. Observable: the screen column of one fixed feature of its image in each tick, and
  how many ticks each column is held. Expected: ordinary columns are held for about the same number
  of ticks. The column of the diagonal (world column 0) is held about twice as long as its
  neighbours. Falsified if the diagonal column's hold time is less than 1.5 times the average of its
  two neighbours', or if a column next to it is held twice as long instead.
- **M2 — height lift** (claims 3, 4, 11).
  - (a) Setup: a purpose-made map with plateaus at levels 0 … 14, each plateau top made of the same
    flat tile, and the same building with a 1×1 foundation placed on a plateau-top cell of each level.
    Observables, both read from matching image pixels: (i) the vertical distance between the top
    corner of a plateau-top tile at level *n* and the row the same cell's top corner would have at
    level 0, which is F1 of the cell's top corner, read from the level-0 tile grid; (ii) the
    vertical distance between the building's image and its own cell's tile, compared with the same
    distance at level 0. Expected: (i) 15·*n* exactly; (ii) no difference, because *L*(104·*n*) =
    15·*n* for these levels. Falsified if any level gives a tile lift other than 15·*n*, or a
    building displacement that differs from its level-0 displacement.
  - (b) Setup: a purpose-made map with plateaus at levels 20 and 34, if the game accepts a map with
    such heights. Observable: the building's image position relative to its own cell's tile, compared
    with the same building on level 0. Expected: 1 px lower at level 20, 2 px lower at level 34.
    Falsified if either displacement differs from that, for example if the image sits at the same place
    relative to the tile at both levels (the likeliest failure). If the game rejects the map, record
    that; (c) remains.
  - (c) Setup: flat map, level 0. A jumpjet hovering in place over the same cell with the same facing
    in five runs. Its jumpjet type's jumpjet cruise height (the height above the terrain at which it
    flies and hovers, so on this map also its world height while hovering) is set, through a test-map
    override section, to 400, 416, 1000, 2080 and 3500 leptons in the five runs. If the type's data makes it bob up and down while hovering,
    turn that off through the same override section; otherwise read its image at the middle of the
    up-and-down motion. Observable: the vertical distance between its hovering images in the 416 run
    and in each of the other four runs, matched by image pixels. Hovering images are compared only
    with hovering images, because a jumpjet type's unit is drawn differently while it stands on the
    ground. Expected, with *L*(416) = 60: the 400 run 3 px lower (*L*(400) = 57), the 1000 run 85 px
    higher (*L*(1000) = 145), the 2080 run 239 px higher and the 3500 run 443 px higher. The exact
    scale would give 2, 84, 240 and 445. Falsified if any distance differs from 3, 85, 239 or 443.
  - (d) Levels below 0 (claim 4). The same as (b), with plateaus sunk to levels −1 and −7, if
    the game accepts a map with such heights. Expected: the building's image is 1 px higher relative
    to its cell's tile at level −1 and 2 px higher at level −7, compared with level 0. Falsified if
    either displacement differs from that. If the game rejects the map, record that, and the claim
    for levels below 0 stays unconfirmed.
  - (e) Overlays (claim 11). Setup: the map of (b), plateaus at levels 0, 20 and 34, with a wall
    segment and a cell of ore placed on a plateau-top cell of each level, next to the building of
    (b). Observable: for each overlay, the vertical distance between its image and its own cell's
    tile, matched by image pixels, compared with the same distance at level 0. The same is read
    for each overlay's shadow, if the chosen types draw one. Expected: no
    difference at level 20 or 34 (the overlay lifts by exactly 15 pixels per level), while the
    building of (b) is 1 px lower at level 20 and 2 px lower at level 34. Falsified if either
    overlay or shadow sits 1 px lower at level 20 or 2 px lower at level 34 (a lift by *L(z)*), or differs
    from its level-0 distance in any other way. Bridge decks: a second purpose-made map with two
    high bridges of the same type and length, one over ground at level 0 and one over ground at
    level 20, if the game accepts such a map. Observable: on each bridge, the vertical distance
    between one fixed pixel of a deck image and the row its cell's top corner would have at level
    0, which is F1 of that corner, read from the level-0 tile grid as in (a). Expected: the
    distance on the level-20 bridge is exactly 300 px (15 × 20) larger than on the level-0 bridge.
    Falsified if it is 299 px larger (a lift by *L(z)*) or differs from 300 in any other way. If the game rejects either map,
    record that; the matching part of claim 11 stays unconfirmed.
- **M3 — view position and scroll limits** (claims 7, 8).
  - (a) Normal case. Setup: a purpose-made map whose local area is smaller than the map size on all
    sides and larger than the view. Scroll hard to each of the four edges and take a screenshot.
    Observable: which cells are visible at each edge (left-most column of cells, top-most row),
    converted to world pixels with F1. Expected: the view edges given after F5. Falsified if any
    edge differs by more than 1 px from the prediction. Start of the game: on the same map, put the
    local player's start position in the local area's top-left corner cell and take a screenshot in
    the first drawn frame of the game. Expected: the view does not extend beyond the limits from F5,
    and its left and top edges are at those limits if the game centres the view on or near the start
    position. Falsified if the view extends beyond either limit.
  - (b) Oversized view. Setup: a purpose-made map whose local area width satisfies *W*/120 <
    *V<sub>w</sub>* < *W*/60, so that the view is wider than the local area but x<sub>min</sub> lies
    inside it. Its vertical scroll range, 30·*V<sub>h</sub>* + 135 pixels, is taller than the view,
    so the vertical axis is limited normally. Place unit P in the left quarter of the local area and
    unit Q at its right edge, both on level 0 and well inside it vertically. Centre the view on P, take a screenshot, then centre it
    on Q and take another. Observable: the world column of the view's left and right edges in each
    screenshot. Expected: centred on P, the view's left edge is at the local area's left edge.
    Centred on Q, the view's right edge is at the local area's right edge, so the view moves left,
    away from Q. Falsified if centring on P does not put the view's left edge at the local area's
    left edge, or if centring on Q leaves the view's left edge at the local area's left edge.
  - (c) Centring with height. Setup: a large map. A unit standing on a 4-level plateau in the middle
    of the local area, far from every scroll limit, and a unit standing at level 0 likewise.
    Centre the view on each in turn. Observable: the view pixel of a fixed feature of each unit's
    image. Expected: the same view pixel in both cases, with the plateau terrain shifted. Falsified
    if the two view pixels differ. The likeliest failure is the plateau unit appearing 60 px higher
    than the level-0 unit, which is where it would be if centring used the ground point below it.
- **M4 — on-screen band** (claim 6).
  - (a) Setup: flat map. An aircraft flying with a world height *z* of at least 1700 leptons, so that
    *L(z)* ≥ 245 px, set through a test-map override section if needed. It flies straight down the
    screen (map south-east) into the view from above the top edge, with the view fixed. Record every
    drawn frame. Observable: the drawn frame in which its shadow first appears and the shadow's row in
    that drawn frame. Expected: the shadow appears suddenly, about *L(z)* − 180 px below the top edge,
    in the drawn frame in which the aircraft's lifted position reaches row −180. It does not slide in from the top
    edge. Falsified if the shadow's first row differs from *L(z)* − 180 by more than the number of
    rows the aircraft moves in one drawn frame. The likeliest failure is a shadow that first appears
    at the top edge and slides in. The aircraft's shadow is drawn as part of the aircraft, so it
    shares the aircraft's band test.
  - (b) Buildings. Setup: a flat purpose-made map, level 0, view fixed. One building, not selected,
    of the tallest building type available, placed in the middle of the local area, far from every
    scroll limit. Its tested position is the centre of its foundation's top-left cell, 15 px below
    that cell's top corner on the screen (F1). Precondition: with the whole building inside the
    view, measure *d*, the number of rows from its tested position up to the topmost pixel of its
    body. The scenario needs *d* ≥ 200. If *d* < 200, record *d*: the scenario then cannot tell the
    outcomes apart, and both building rows of claim 6's backing table stay unconfirmed. Two runs
    with the same map and building, in which only the fixed view differs. Fix the view at a known
    position: centre it on a chosen cell, and read the origin from the cells visible in a
    screenshot, as in M5. Then choose the building's cell so that, in run 1, its tested position
    lies 181 to *d* − 1 rows below the view's bottom edge (view rows *H* + 181 … *H* + *d* − 1).
    Moving the building one cell along X or Y moves its position 15 rows, and the window is at least
    19 rows tall, so some cell always fits. Part of the body then lies inside the view, but the
    position lies outside the band. In run 2, the control, the tested position lies about 100 rows
    below the bottom edge. In both runs the tested position lies far inside the ground box, whose
    lower edges are more than *H* + 1.25·*W* pixels below the view. Observable, from a screenshot of
    each run: whether the part of the body that lies inside the view is drawn; and, if the building
    has a part drawn as an object (S-0102), such as a voxel turret, that lies inside the view in
    run 1, whether that part is drawn. Expected: the body is drawn in both runs, because it is
    painted in the cell pass and no band test applies to it. Such an object part is drawn in both
    runs too, because building parts skip the band. Falsified if run 1 shows none of the body while
    run 2 shows it, or if such a part is missing in run 1 while drawn in run 2: either is what a
    band test on the building's position would give. Only the body and those parts are observed,
    and the building is kept unselected so that no selection or status markers are drawn: whether
    the band decides those is an open question.
  - (c) Infantry. Setup: flat map, level 0, view fixed. A unit of a jumpjet infantry type whose
    jumpjet cruise height is set, through a test-map override section, to 2080 leptons, so that
    *L(z)* = 299 px while it flies. Precondition: with the unit hovering inside the view, its shadow
    is drawn on the ground about 299 px below its image. If it is not, record that: the scenario
    then cannot show the band for infantry, and that part of claim 6 stays unconfirmed. Order the
    unit to fly straight down the screen into the view from above the top edge, and record every
    drawn frame. Observable: the drawn frame in which its shadow first appears and the shadow's row
    in it. Expected: the shadow appears suddenly about 119 px (*L(z)* − 180) below the top edge, in
    the drawn frame in which the unit's lifted position reaches row −180. Falsified if the shadow's
    first row lies above row 119 by more than the number of rows the unit moves in one drawn frame,
    in particular if the shadow first appears at the top edge and slides in.
  - (d) Vehicles. The same as (c), with a vehicle type made a jumpjet type through the same override
    section: its imported jumpjet flag set, its imported locomotor field set to the jumpjet
    locomotor (the flag alone does not change the locomotor), and its jumpjet cruise height set to
    2080 leptons. The same
    precondition, observable, expected result and falsifier apply, for vehicles.
  - (e) Projectiles. Setup: flat map, level 0. An aircraft type that attacks by dropping
    projectiles that fall to the ground, flying at a world height from 2080 to 3300 leptons (set
    through a test-map override section if needed), so that each projectile starts its fall with
    299 ≤ *L(z)* ≤ 475 px. The projectile's image is a sprite, not a voxel model, and its art leaves
    its shadow on (the absent-key default). Two runs with the same map, target, orders and timing.
    Only the fixed view differs. In run 1 the view is placed so that the target's cell is drawn about
    20 px below the view's top edge, in run 2 about 300 px below it. Record every drawn frame.
    Observable: the number of drawn frames from the first drawn frame that shows the falling
    projectile's shadow to the first drawn frame that shows its impact (the impact animation is on
    the ground, and animations skip the band, so it is visible in both runs). Expected: in run 2 the
    shadow is drawn for the whole fall, because the projectile's lifted position is at row
    300 − *L(z)* ≥ −180 for every *z* ≤ 3340 leptons. In run 1 it appears only once 20 − *L(z)* ≥
    −180, that is once *L(z)* ≤ 200 px (*z* ≤ 1390 leptons), so the run 1 count is clearly smaller.
    The aircraft and its own shadow are also missing in run 1 while they are above row −180.
    Falsified if the run 1 count is within one drawn frame of the run 2 count. If run 2 shows no
    shadow at all, the scenario says nothing about projectiles.
- **M5 — screen to ground** (claim 9, its cell partition; F7). Setup: flat map, level 0. Scroll to the top-left
  scroll limit so that the view origin is known from F3 and F5 (which M3a checks), or read the origin
  from the cells visible in a screenshot. Put the cursor on exact view pixels,
  for example with a program that moves the mouse pointer (this is player input, not a recorder:
  it supplies only the pointer position a player would), around the top corner of one cell whose two indices are
  both at least 1: the 7×7 pixels
  centred on it, and the pixels on its four edges one pixel inside and one pixel outside each edge.
  Observable: the reported cell for each pixel, read as the cell outline shown while placing a
  building with a 1×1 foundation, or as the cell a vehicle drives to after a move order at that
  pixel. On ground where every cell is at level 0, the cell under the cursor is the flat-ground
  cell under the cursor pixel (claim 10), so this observes F7's pixel formula directly.
  Expected: the cell given by F7's pixel formula. In particular the corner pixel and the
  upper-edge pixels belong to the cell, and the lower-edge pixels to its neighbours. Falsified if any
  pixel's cell differs from the prediction.
- **M6 — cursor on raised terrain** (claim 10). (a) Setup: a purpose-made map, no bridges. Every
  cell (x, y) with x + y ≤ 2*p* is at height level 4 and every other cell at level 0. Choose *p* so
  that every cell (*n*, *n*) with *p* − 2 ≤ *n* ≤ *p* + 3 lies in the local area, and so that world
  column 0 lies inside the local area horizontally (60·*V<sub>x</sub>* − 30·*M<sub>w</sub>* ≤ 0 <
  60·(*V<sub>x</sub>* + *V<sub>w</sub>*) − 30·*M<sub>w</sub>*). This is a straight cliff four levels
  high, facing the camera, along a screen row. Only the cells' height levels matter, not which tiles
  are drawn. Fix the view and know its origin as in M5. Put the cursor, as in M5, on every pixel of
  world column *q<sub>x</sub>* = 0 (the column through the top corners of the cells (*n*, *n*)), from
  world row 30*p* − 120 down to 30*p* + 89. Observable: the reported cell per pixel. Expected, by
  world row *q<sub>y</sub>* (view row = *q<sub>y</sub>* − *O<sub>y</sub>*):

  | World rows | Cell | Where on the screen |
  |---|---|---|
  | 30*n* − 60 … 30*n* − 31, for *n* < *p* | (*n*, *n*) | plateau top, each cell's diamond lifted by 60 px |
  | 30*p* − 60 … 30*p* − 31 | (*p*, *p*) | the front plateau cell's lifted diamond |
  | 30*p* − 30 … 30*p* + 29 | (*p*, *p*) | the cliff face: the 60 rows between that lifted diamond and the low ground |
  | 30*n* … 30*n* + 29, for *n* > *p* | (*n*, *n*) | low ground, unlifted |

  Worked example of the arithmetic only (not a map to build, since these cells lie outside any real
  local area): with *p* = 10, rows 180–209 select (8, 8), 210–239 select (9, 9), 240–329 select
  (10, 10) and 330–359 select (11, 11). Falsified if any pixel's cell differs from the table. The likeliest
  failure is the cliff-face rows selecting (*p* + 1, *p* + 1), the low cell in front.
  - (b) Probe depth. The same map with the plateau at level 14 instead of 4 (if the game rejects
    that height, record it). Cursor on world column 0 from row 30*p* − 240 down to 30*p* + 89.
    Expected: rows 30*n* − 180 … 30*n* − 151 select (*n*, *n*) for *n* < *p*, rows 30*p* − 180 …
    30*p* + 29 select (*p*, *p*), and rows 30*n* … 30*n* + 29 select (*n*, *n*) for *n* > *p*. The
    drawn diamond of (*n*, *n*) starts 30 rows higher, at 30*n* − 210, so its top 30 rows select
    (*n* − 1, *n* − 1). Falsified if any pixel's cell differs, in particular if the pick follows the
    drawn diamonds (a lift of 210 px).

## Open questions

- The cell under the cursor when probed cells are part of a bridge (claim 10, scope) is not
  specified here. The original treats a bridge deck as lifted above its cells' height levels and can
  select a neighbouring bridge cell, under conditions that depend on the bridge's direction and its
  neighbouring cells. A later bridge spec should specify this, with a scenario like M6 run across a
  bridge.
- The view size *W*, *H* for each resolution belongs to a later user-interface spec, not yet
  written.
- Which terrain objects are animated, and so drawn as objects under the ground box of claim 6, and
  which are painted in the cell pass. The Terms leave this to S-0102, whose own open questions say
  it is not yet fully mapped. Until S-0102 states it, the terrain-object parts of claim 6 have no
  defined membership.
- Where on its cell an overlay's image, and its shadow, is placed (claim 11). Some overlay kinds,
  among them ore and gem fields, walls and crates, and bridge-deck cells are drawn a few pixels
  higher than other overlays, and some deck shadows a few pixels to the side, by fixed amounts that do not depend on the height level, and some of
  these shifts depend on the overlay type's position in the overlay type list. A later overlay or
  bridge spec should state the placement in full.
- Whether any game mode lifts the scroll limits. None is known in skirmish play.
- Whether the screen band also decides anything else drawn for a building, such as its selection
  outline, its health bar and other status markers. The original appears to draw these only while
  the building's position is in the band, but no scenario checks it yet. The building's body is
  drawn regardless (claim 6).
- What happens to scrolling (edge or keyboard scrolling, as opposed to centring requests) when the
  view is larger than the local area. Claim 8 gives the result of each request. How scrolling
  turns into requests in that case has not been studied.
