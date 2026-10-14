---
id: S-0111
title: In-game overlays (health bars, pips, selection marks, insignia)
topic: rendering
revision: 9
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
measurements: []
requests: [Q-0471]
supersedes: []
---

# S-0111: In-game overlays (health bars, pips, selection marks, insignia)

## Summary

The status extras that S-0102 paints over every vehicle, infantry unit, aircraft and building after
the five render layers: where they are anchored, when the health bar shows, how many health segments
are filled and in which colour, the selection marks, the pip scale (passengers, spawned aircraft,
occupants, mind control, resource load, ammunition), the self-heal marker, the control-group number,
the veterancy insignia and the repair wrench. Which objects have status extras and the painting order
belong to S-0102.

## Terms

*Tick*, *drawn frame*, *image frame*, *sprite*, *world pixel*, *battlefield view*, *local player*,
*house*, *disguise*, *foundation* (W × H cells in the glossary; written W × D here, because *H* is
the health), *game speed* (index *s*), *quirk switch*, *departure* and *importer warning* are used as
in the glossary. *Status extras* is S-0102's term. The projection
F1 and the height lift *L* (F2) are those of S-0101; the health ratio and its rounding are those of
S-0108 (Formulas F3).

| Term | Meaning |
|---|---|
| overlay (in *overlay sprite*, *overlay image*, *overlay palette*) | Part of the status extras of this spec, and the sprites and palette of Import mapping that draw them. Not the glossary's *overlay*, the cell-level map object kind. |
| status anchor | The screen pixel of an object's centre at its world height: its world pixel by the projection of S-0101 including the height lift, minus the view origin. Every displacement below is in screen pixels from it, *x* to the right and *y* downwards. |
| projected displacement P(ξ, η, ζ) | The screen displacement that S-0101 F1 and F2 give when applied to the world displacement (ξ, η, ζ) in leptons (not to an absolute position): P = (trunc((30ξ − 30η) / 256), trunc((15ξ + 15η) / 256) − *L*(ζ)), integers, truncated toward zero. A building overlay is placed at the anchor plus a projected displacement. This is not the same as projecting the absolute corner and subtracting the anchor's world pixel, which can differ by a pixel through the truncations. |
| building box | For a building of foundation W × D cells (W along the map *x* axis, D along *y*): the box whose footprint is 256·W by 256·D leptons centred on the building's centre point, and which rises from the centre's height to *V* leptons above it. *V* is the building's art height field (Import mapping) times the height of one level, 104 leptons. With α = 128·W and β = 128·D (half the footprint, in leptons), its left corner lies (−α, β) leptons from the centre point and its top corner (−α, −β). |
| health ratio *h* | Current health *H* divided by the type's full strength *S*, the quotient rounded toward zero to binary64, as in S-0108 F3 (the round-toward-zero mode, planned S-0205). |
| yellow ratio, red ratio | The two game-wide condition ratios that S-0108 imports (Import mapping), binary64 values. |
| health colour | green, yellow or red (claim 5). |
| selected | Chosen by the local player as part of the current selection. |
| hovered | Under the local player's mouse pointer while not selected. |
| pip sprite, second pip sprite, bracket sprite, wrench sprite | The overlay sprites of Import mapping. Image-frame numbers below refer to them. |
| pip scale | The row of small pips drawn with the health bar for an owner the local player may inspect (claim 9). |
| pip-scale kind | The type's imported pip-scale field: none, ammunition, resource, passengers, power or mind control. |
| row length *R* | The number of pips a type's pip-scale kind gives (Constants T2). |
| centred image | An image drawn with the centre of its sprite canvas at the given point (the canvas width and height halved and truncated). Every overlay image here is centred. |
| gunner flag | A type flag (Import mapping) marking a transport whose first passenger mans its weapon; it only shifts the passenger row here (claim 12). |
| absorbing building | A building type flagged to absorb infantry or to absorb vehicles (Import mapping); the passengers it holds fill its row differently (claim 12). |
| veterancy value *v* | A unit's experience as a single-precision number: below 0, from 0 to below 1 (rookie), from 1 to below 2 (veteran), 2 or more (elite). The combat specs own how it grows. |

## Behaviour

1. **Anchor and culling.** All status extras of an object are placed from its status anchor. They
   are drawn only when the anchor lies within the battlefield view widened by 360 pixels on the left
   and the right and by 180 pixels at the top and the bottom. Fog of war does not hide them.
   - Backing: `measurable` (scenario OV1)
   - Confidence: high

2. **Shading of overlay images.** Selection brackets, the veterancy insignia, the repair wrench and
   the attached-bomb sprite are shaded by the light-alpha values of S-0104 claim 15 at neutral
   brightness, so the shroud and fog of S-0110 (shroud rendering) darken them. Health segments and
   pips are not shaded: they keep their palette colours under shroud and at night.
   - Backing: `measurable` (scenario OV2)
   - Confidence: medium

3. **When the health bar shows.** The health bar of an object, of any owner, is drawn while it is
   selected, and while it is hovered. A hovered unit that is disguised to the local player shows a
   health bar only as the disguise rules allow (the special-abilities specs). The health bar is drawn
   at every health from full down to the last point.
   - Backing: `measurable` (scenario OV3) for selection and hovering; the disguise case is a pointer
     to the special-abilities specs, which will carry its backing
   - Confidence: high for selection and hovering; low for the disguise case

4. **Filled segments.** A health bar has *n* segments (claims 6 and 7). For a full strength above 0,
   the number of filled segments is *f* = min(*n*, max(1, trunc(*h* × *n*))), with the product
   rounded toward zero in binary64 (Formulas F1); claim 5 gives *f* for a full strength of 0 or
   below. So one segment stays filled at any health, and *f* = *n* only at full health (or
   above). In exact terms *f* = min(*n*, max(1, ⌊*H*·*n* / *S*⌋ − *c*)), *c* as in F1: one segment
   fewer than the exact product when *H*·*n* / *S* is a whole number but *H* / *S* has no exact binary
   representation.
   - Backing: `measurable` (scenario OV4)
   - Confidence: high

5. **Health colour.** The colour follows from where *h* lies relative to the two ratios. The three
   rows below exclude each other and cover every *h*; *h* is compared exactly with the binary64
   ratios.

   | *h* | Vehicles, aircraft and infantry | Buildings |
   |---|---|---|
   | *h* ≤ red ratio, or *h* undefined | red | red while *H* > 0; green at *H* = 0 |
   | red ratio < *h* ≤ yellow ratio | yellow | yellow |
   | *h* > red ratio and *h* > yellow ratio | green | green |

   The two columns agree for every *H* > 0. They differ only at *H* = 0, where a building shows green
   (with a red ratio of 0 or more). With both ratios at their absent-key default 0.5, or with a red
   ratio above the yellow ratio, a bar is never yellow.
   - **Full strength of 0 or below** (bad data; the combat specs own whether such an object can
     stand). Then *f* = 1 in every case, whether the product *h* × *n* is negative, zero, infinite
     or undefined. The formula of claim 4 does not apply here: read exactly, an infinite product
     would give *n*. The colour is the table's with this *h*: for *S* < 0, *h* = *H* / *S* ≤ 0 as computed; for
     *S* = 0 and *H* > 0, *h* is +∞, above both ratios (green); for *S* = 0 and *H* = 0, *h* is
     undefined and falls in the first row (red for vehicles, aircraft and infantry, green for a
     building, since *H* = 0). Our engine reproduces this and the importer warns about a type whose
     full strength is 0 or below.
   - Backing: `measurable` (scenario OV4) for *H* > 0; `unmeasured` for full strength 0 or below,
     because whether an object with such a strength can stand on the map belongs to the combat specs
     and no scenario of this spec can make one; `unmeasured` for a building at health 0,
     because a building whose health reaches 0 is destroyed (the combat specs), so no stable state
     shows its bar at health 0
   - Confidence: high; medium for the full-strength case (derived, not observed)

6. **Health bar of vehicles, aircraft and infantry.** With δ the type's selection-bracket displacement
   (Import mapping):

   | Kind | Segments *n* | Segment *i* (from 0) at | Bracket image (selected only) |
   |---|---|---|---|
   | vehicle or aircraft | 17 | (−15 + 2*i*, δ − 25) | bracket sprite image frame 0 at (1, δ − 26) |
   | infantry | 8 | (−5 + 2*i*, δ − 24) | bracket sprite image frame 1 at (11, δ − 25) |

   Only the filled segments are drawn, each as a centred image of the pip sprite: image frame 16 for
   green, 17 for yellow and 18 for red. Empty segments draw nothing, so the bracket image shows through
   as the empty part. When the object is hovered but not selected, the segments are drawn without the
   bracket.
   - Backing: `measurable` (scenario OV3)
   - Confidence: high

7. **Health bar of buildings.** A building's health bar is a diagonal line of *n* pips of the pip
   sprite, each pip 4 pixels to the left of and 2 pixels below the one before, so the line runs down
   to the left, parallel to the building box's upper-left edge (from its top corner to its left
   corner). With *Q* = P(−α, β, *V*), the projected displacement of the box's left corner on its top
   face, and *Q*′ = P(−α, −β, *V*), that of its top corner on the top face:
   - *n* = trunc((*Q*<sub>y</sub> − *Q*′<sub>y</sub>) / 2), integers. For whole-cell foundations
     this is *n* = trunc((trunc(15(D − W) / 2) − trunc(−15(W + D) / 2)) / 2); the height cancels.
     It equals ⌊15·D / 2⌋ except when D is even, W + D is odd and D > W, where it is one less (for
     example 14 for a 1 × 2 foundation, 29 for 1 × 4 and 3 × 4).
   - *Q* = (−15(W + D), trunc(15(D − W) / 2) − *L*(*V*)).
   - Pip *i* (from 0, top right first) is centred at *Q* + (4(*n* − *i*) + 3, 2(*i* − *n*) + 4), so
     the last pip lies 7 pixels right of and 2 pixels below *Q*.
   - The *f* filled pips come first from the top right, in image frame 1 (green), 2 (yellow) or 4
     (red); the remaining *n* − *f* pips are drawn in image frame 0 (empty).
   - Backing: `measurable` (scenario OV5)
   - Confidence: high for the pip image frames, the step, the filled-first order, *n* and the placement
     relative to *Q*; medium for the vertical part of *Q* (the level height of 104 leptons that
     scales the art height field)

8. **Selection marks of buildings.** A selected building gets corner brackets instead of a bracket
   image: thin lines of palette index 15 of the overlay palette (index 12 when the building's height
   above the terrain is below −4 leptons), drawn in the projection of the building box. At a corner of
   the box, a line runs along each edge that meets there, for a quarter of that edge's length, rounded
   toward zero in leptons. Vehicles, aircraft and infantry have no corner brackets.
   - Backing: `measurable` (scenario OV5) for the lines and index 15; `unmeasured` for index 12,
     because no known game state gives a building a height above the terrain below −4 leptons (a
     building stands on the terrain of its cell), so no scenario can show it
   - Confidence: medium for the quarter-length rule, the box and the colour; low for which corners
     are drawn

9. **Who sees the pip scale.** The pip scale is drawn together with the health bar (claim 3) when:
   - the owner is the local player's house or allied with it, or
   - the local player's house has spied on the object (the special-abilities specs own spying), or
   - for a building only, the type is marked to draw its pips for all, or can be occupied by
     infantry. The draw-for-all mark has no effect on vehicles, aircraft and infantry.

   Otherwise only the health bar is drawn.
   - Backing: `measurable` (scenario OV6)
   - Confidence: high

10. **Pip-scale placement.** The pip scale starts at a point *o*: for vehicles, aircraft and infantry
    *o* = (−10, +10) from the anchor; for a building *o* = P(−α, β, 0), the projected displacement
    of the building box's left corner at the base of the box, which for whole-cell foundations is
    (−15(W + D), trunc(15(D − W) / 2)). Pip *j* (from 0) of a row is drawn as a centred image at:

    | Kind | Pip *j* at | Sprite |
    |---|---|---|
    | vehicle or aircraft | *o* + (−5 + 4*j*, 0) | second pip sprite |
    | infantry | *o* + (6 + 4*j*, 0) | second pip sprite |
    | building | *o* + (6 + 4*j*, −1 + 2*j*) | pip sprite |

    Ammunition pips (claim 15) are drawn 3 pixels higher than this row and always come from the
    second pip sprite, for buildings too. The self-heal marker (claim 16) always comes from the pip
    sprite.
    - Backing: `measurable` (scenario OV6)
    - Confidence: high

11. **What the pip scale shows.** The pip scale shows each row of the table below that applies to
    the object. Every row begins at pip 0, so two rows on one object overlap; where they do, a row
    covers every row above it in the table.

    | Row | Shown for | Pips |
    |---|---|---|
    | spawned aircraft | a type that spawns aircraft, whatever else applies | one per spawn slot: image frame 1 for each aircraft currently docked or ready, 0 for the rest |
    | occupants | a building type that shows its occupant pips and holds at least one occupant slot, whatever else applies | one per slot: the occupying infantry type's occupant pip image frame, or image frame 6 for an empty slot |
    | passengers | a type with a passenger capacity above 0, whatever its pip-scale kind | claim 12 |
    | by pip-scale kind | a type with a passenger capacity of 0 or below | mind control: claim 13; resource: claim 14; ammunition: claim 15; none and power: no row |

    So the passenger row and the row by pip-scale kind exclude each other, while the spawned-aircraft
    and occupant rows can appear with either: a type that spawns aircraft, has the ammunition kind and
    no passenger capacity shows the spawn row covered by the ammunition row where they overlap.

    The self-heal marker (claim 16) and the control-group number (claim 17) belong to the pip scale
    as well: they cover the rows where they overlap them, and the group number covers the self-heal
    marker where those two overlap. A building whose pip-scale kind is resource shows no pip scale at
    all: no row, no self-heal marker and no group number (`quirks.building_resource_pips_hidden`).
    Filling or exceeding the passenger row length does not hide the passenger row, the self-heal
    marker or the group number: claim 12 clips the passenger pips to the row, and claims 9, 16 and
    17 still decide their ordinary visibility.
    - Backing: `measurable` (scenarios OV6, OV7); overflow visibility is a departure tested against
      this spec, not the original (scenario OV7-D)
    - Confidence: high for the rows, the covering, the passenger rule and the resource cut-off,
      high for the lead-decided overflow visibility

12. **Passenger pips.** The row length *R* is the one that the type's pip-scale kind gives (Constants
    T2): the passenger capacity for the passengers kind, but 0 for the kind none, the ammunition
    capacity for the ammunition kind, and so on. Let the *m* passengers be numbered *k* = 1 … *m* in
    the order they entered, *z*<sub>k</sub> being passenger *k*'s size field (binary64).
    - **Retained pip values.** An infantry type's passenger pip colour and occupant pip image frame
      keep the held value whenever a rules layer contains its section but omits the corresponding
      key, including in the base rules; before any reading their defaults are image frames 1 and 7
      respectively (S-0003 claim 16; S-0006 claim 5). This fixed display-only departure has no quirk
      switch and is independent of `quirks.override_resets_keys`: it changes no passenger capacity,
      loading legality or other gameplay rule.
    - **Total size.** When every size is 0 or more, the total size is *Z* = Σ trunc(*z*<sub>k</sub>),
      the sum of the sizes each truncated toward zero, whatever the order of entry. A negative size
      can make *Z* depend on that order: in general *Z* = *Z*<sub>1</sub>, where *Z*<sub>m+1</sub> = 0
      and *Z*<sub>k</sub> = trunc(*Z*<sub>k+1</sub> + *z*<sub>k</sub>) for *k* = *m* … 1, each sum in
      binary64 under the round-toward-zero mode (planned S-0205). For sizes of 0
      or more this equals Σ trunc(*z*<sub>k</sub>), because that mode never rounds a sum up. For
      example, a first passenger of size −0.5 followed by one of size 2.5 gives *Z* = 1, not 2.
    - **Row bounds.** Only pips with integer indices 0 ≤ *j* < *R* are drawn, whether *Z* is below,
      equal to or above *R*. Thus *Z* > *R* clips the passenger blocks without suppressing the
      self-heal marker or group number (departure); *R* = 0 gives no passenger pips, but those
      markers remain eligible under their ordinary visibility rules.
    - **Blocks.** Each passenger *k* covers a block of *b*<sub>k</sub> consecutive pips:
      *b*<sub>k</sub> = 1 + max(0, trunc(*z*<sub>k</sub> − 1)) for an infantry or vehicle passenger
      (*z*<sub>k</sub> − 1 rounded toward zero in binary64, then truncated, which gives the truncation
      of the exact value), and *b*<sub>k</sub> = 1 for any other passenger. The blocks are stacked
      downwards from pip *Z* − 1 in order of entry, the most recent passenger's block highest:
      passenger *k*'s block runs from pip *s*<sub>k</sub> = *Z* − (*b*<sub>k</sub> + *b*<sub>k+1</sub>
      + … + *b*<sub>m</sub>) to pip *s*<sub>k</sub> + *b*<sub>k</sub> − 1. Its lowest pip
      *s*<sub>k</sub> shows its colour pip: its type's passenger pip colour (Import mapping) for
      infantry, image frame 5 for a vehicle, image frame 1 for any other passenger. Its other pips show
      image frame 3. Every pip of the row outside the blocks shows image frame 0.
    - **Reading the row.** For an infantry or vehicle passenger of size 1 or more, *b*<sub>k</sub> =
      trunc(*z*<sub>k</sub>). When every passenger is such a passenger, the blocks fill pips 0 to
      *Z* − 1 exactly, and read from pip 0 the row shows the passengers in the order they entered,
      each one's colour pip followed by its extra pips, and then the empty pips. When the blocks sum
      to less than *Z* (a passenger of another kind whose truncated size is above 1), the lowest
      *Z* − Σ*b*<sub>k</sub> pips stay empty, below the first passenger's block. Only infantry and
      vehicles are known to enter transports; the other kind is given for completeness. When the
      blocks sum to more than *Z* (a size below 1), a block reaches below pip 0 and the original's
      result is *unspecified*; our engine shows no pip below pip 0, and the importer warns about a
      passenger-capable type with a size below 1.
    - **Gunner gap.** On a transport type with the gunner flag, pip 0 stays at its place of claim 10
      and each pip *j* from 1 to *R* − 1 is drawn at the place that claim 10 gives pip *j* + 2, which
      leaves a gap of two pip positions after pip 0. Pip 0 is drawn even when *R* = 0; its image frame
      is then *unspecified* in the original, and our engine draws no pip (the importer warns about a
      gunner type whose row length is 0).
    - **Absorbing buildings.** In an absorbing building the row fills upwards from pip 0 instead, one
      pip per passenger, with the most recent passenger at pip 0 and the first one highest: an
      infantry passenger gives its pip colour, a vehicle image frame 4, anything else image frame 1.
      The remaining pips show image frame 0. Its one-pip-per-passenger layout replaces the
      total-size block layout above. If such a building
      holds more passengers than *R*, the original's result is *unspecified*; our engine shows only
      the *R* most recent passengers.
    - Backing: `measurable` (scenario OV7); fixed pip retention is a display-only departure tested
      against this spec in OV7-R, not an original-game expectation
    - Confidence: high for fixed pip retention, the row length, total size, blocks, image frames and order.
      Medium for the gunner gap and the absorbing buildings
    - Overflow clipping backing: departure, tested against this spec in OV7-D, not the original,
      confidence high for the lead-decided rule

13. **Mind-control pips.** The row has *M* pips, *M* being the damage of the type's primary weapon
    (the number of units it can control, the special-abilities specs). Counting from pip 0, as many
    pips as the unit currently controls show image frame 1, and the rest of the row image frame 0.
    A unit whose type may control
    without limit and that currently controls more than *M* (overloading, the special-abilities specs)
    shows one more pip at index *M*, just after the row. That pip shows image frame 3 for 10 ticks
    from each overload damage pulse: in the drawn frames that follow the tick of the pulse and each of
    the next 9 ticks. From the 10th tick after the pulse it shows image frame 4, until the next pulse.
    The 10 ticks are not normalised for game speed.
    - A type with the mind-control kind and no primary weapon is bad data: the original fails as soon
      as such an object's pip scale is to be drawn, since the row length *M* it needs does not exist.
      Our engine takes *M* = 0 for it, so the row has no pips and an overload pip, if any, stands at
      index 0; the importer warns about a type with the mind-control pip-scale kind and no primary
      weapon (departure). The same *M* = 0 is the row length that claim 12 uses for such a type with
      a passenger capacity.
    - Backing: `measurable` (scenario OV8); the bad-data case is `unmeasured`, because the original
      fails instead of showing a result
    - Confidence: high for the row and for the failure without a primary weapon; medium for the
      overload pip and its 10 ticks

14. **Resource pips.** The row has 3 pips for an infantry type and 5 for any other type. An object's
    load holds one amount per resource for the resources at registration indices 0 to 3 (glossary
    *registration index*, the tiberium registry; the economy specs own how the amounts change), each
    amount a single-precision number. The load is split in two:
    - *g* is the amount at registration index 1, widened exactly to binary64, and
    - *q* is the sum of the amounts at indices 0, 2 and 3, formed in binary64 under the
      round-toward-zero mode (planned S-0205) as (*a*<sub>3</sub> +
      *a*<sub>2</sub>) + *a*<sub>0</sub>, where *a*<sub>i</sub> is the amount at index *i*. This sum
      is exact, so its order and its rounding cannot matter, whenever the largest non-zero amount
      is less than 2<sup>27</sup> times the smallest non-zero amount: the binary exponents of the amounts then
      differ by at most 27, and the exact sum of three single-precision numbers needs at most 24
      significant bits for one amount, plus the exponent spread, plus 2 bits of carry, that is at
      most 24 + 27 + 2 = 53 bits, which binary64 holds (and the partial sum
      *a*<sub>3</sub> + *a*<sub>2</sub> needs fewer). Only amounts further apart than that can make
      the order and the rounding of *q* visible.

    With *C* the type's storage capacity and *R* the row length:
    - *n*<sub>g</sub> = trunc(*g* / *C* × *R* + 0.5) and *n*<sub>q</sub> = trunc(*q* / *C* × *R* +
      0.5) pips, in binary64 with each operation rounded toward zero (Formulas F2).
    - The first *n*<sub>g</sub> pips use image frame 5, the next *n*<sub>q</sub> image frame 2, and
      the rest image frame 0; pips beyond *R* are not drawn.

    So each part rounds half up separately, and the row can show one pip more or one pip fewer than
    the whole load would round to.
    - Backing: `measurable` (scenario OV9) for the row, the split and the rounding; `unmeasured` for
      the order of the sum *q*: another order or rounding changes *q* by at most one unit in its last
      binary64 place, which moves *n*<sub>q</sub> only when *q* / *C* × *R* + 0.5 lies within that
      margin of a whole number, and no scenario can arrange that reliably
    - Confidence: high

15. **Ammunition pips.** The row length is the type's ammunition capacity *A*, and *a* is the current
    ammunition. With no wrap value (the type's pip wrap *w* = 0), pip *j* is drawn in image frame 13
    while *j* < *a* and is not drawn otherwise. With *w* > 0 only *w* pips are drawn, and pip *j* shows
    image frame 15 + λ, where λ is the largest integer from 0 to ⌊*A* / *w*⌋ − 1 with λ·*w* + *j*
    < *a*, or image frame 14 when there is none. So each further layer of wrapped ammunition moves a pip
    to the next image frame.
    - Backing: `measurable` (scenario OV10)
    - Confidence: high

16. **Self-heal marker.** A vehicle or infantry unit with a pip scale shows a self-heal marker when its
    owner has the healing ability that applies to it: infantry, and vehicles marked organic, use the
    infantry healing ability (the owner holds a building that heals infantry); other vehicles use the
    unit repair ability (the specs of those buildings own the abilities). Aircraft and buildings never
    show the marker.
    - Its image frame follows the ability: 13 for infantry healing (organic vehicles included), 20 for
      unit repair.
    - Its place follows the kind: (33, −32) from *o* for a vehicle and (25, −35) for infantry (38 and
      19 pixels right of the first pip of the row of claim 10).
    - It is drawn every drawn frame while the ability exists. During the first 6 ticks of every period
      of *T* ticks, counted from tick 0 of the game (*t* mod *T* ≤ 5), and only while the unit is
      damaged, it is drawn as a shadow (darkening) instead of as an image. *T* is the game-wide
      self-heal interval for infantry when the infantry healing ability applies, and for units
      otherwise. The blink follows ticks and is not normalised for game speed.
    - A negative *T*, which is bad data, blinks with period |*T*| in the original: the marker is
      drawn as a shadow while *t* mod |*T*| ≤ 5. Our engine does the same and the importer warns. With *T* =
      0 the original fails; our engine then never blinks the marker and the importer warns
      (departure).
    - Backing: `measurable` (scenario OV11)
    - Confidence: high for the conditions, image frames, places and the blink period; medium for the
      darkened drawing during the blink

17. **Control-group number.** A unit or building in one of the local player's ten control groups
    shows the digit (*γ* + 1) mod 10 for group *γ* (0 to 9), so the first group shows 1 and the
    tenth shows 0. It is drawn with the pip scale, as text placed at the point (−4, −39) from the
    pip-scale origin *o* of claim 10 for infantry and (−4, −36) for others, so for a vehicle at
    (−14, −26) and for infantry at (−14, −29) from the status anchor. It is drawn in the owner's house
    colour, inside a small border of the same colour.
    - Backing: `measurable` (scenario OV12)
    - Confidence: high for the digit and the placement point relative to *o*; medium for the colour
      and the border; how the text is aligned to that point and the font are not established (Open
      questions)

18. **Veterancy insignia.** Every unit and building whose veterancy value is outside the rookie band
    shows an insignia, whether or not it is selected and whoever owns it: veteran image frame 14, elite
    image frame 15, below 0 image frame 19, all from the pip sprite, as a centred image at (5, 2) from
    the anchor for infantry and (10, 6) for everything else. The insignia is not drawn while the unit
    is disguised to the local player, and not while it is cloaked and hidden from the local player.
    For vehicles, infantry and aircraft, a cloaked unit stays visible to the local player, insignia included, when:
    - the local player owns it, or
    - its cell is revealed by the local player's sensors (the special-abilities specs own cloaking
      and sensors), or
    - in a skirmish or multiplayer game, its owner and the local player's house are allied with each
      other.

    While such a unit is cloaking or uncloaking (between visible and cloaked), the insignia is drawn.
    When a cloaked building hides its insignia is not established (Open questions).
    - Backing: `measurable` (scenario OV13)
    - Confidence: high for the image frames, bands and position; medium for the disguise and cloak cases

19. **Repair wrench.** A building under repair shows the wrench sprite, unless its cell is under
    shroud. Its cell is the cell of the building's position, the top-left cell of its foundation
    (S-0101 claim 6), not the cell under its centre. The shroud test is the point test of S-0110
    (shroud rendering) claim 8 applied to the centre of that cell at the cell's centre height: for
    cell (*x*, *y*) the point (256*x* + 128, 256*y* + 128, *Z*<sub>c</sub>) in leptons, where
    *Z*<sub>c</sub> is S-0106's surface height (claims 2 and 3) at in-cell position (128, 128). It
    is not the level height ℓ × 104, and the two differ:
    - on a valley or steep cell at level ℓ ≥ 0, *Z*<sub>c</sub> = ℓ × 104 + 104 leptons, so the
      test's count of whole height levels is *u* = ℓ + 1, not ℓ, and claim 8 picks its cells for
      ℓ + 1 levels (for example, at level 0 the pair of cells for *u* = 1 instead of the cell itself),
    - on an edge slope or saddle cell *Z*<sub>c</sub> = ℓ × 104 + 52 and on a flat or corner-slope
      cell ℓ × 104, so *u* = ℓ for ℓ ≥ 0,
    - below level 0, *Z*<sub>c</sub> is 1 lepton higher than with the level height (S-0106 claim 3),
      for example −103 on a flat cell at level −1.

    **Centre height below 0.** S-0110 claim 8 owns the point test for both signs of the centre
    height, including truncation toward zero and `quirks.shroud_negative_odd_pair` (default:
    on, the original). The wrench uses that shared test and its quirk alternatives, not a separate
    negative-height rule.

    The wrench's image frame at tick *t* is ⌊6·(*t* mod *p*) / (*p* − 1)⌋, where *p* = max(2,
    ⌊N(14, *s*) / 4⌋) ticks and N is the game-speed normalisation of S-0107 (F3). So the wrench
    cycles through image frames 0 to 6 (all seven only when *p* ≥ 7; with a shorter period some are
    skipped) in a period that lasts about the same real time at every game speed. Its period is the pulsing marker's period that the glossary (claim G3 and its Open
    questions) leaves to the rendering specs. It is drawn only, and affects no game state. With
    `quirks.speed_normalised_delays` off, *p* = max(2, ⌊14 / 4⌋) = 3 ticks at every speed. A unit
    carrying an attached bomb shows the bomb sprite the same way when its cell (the cell of its
    position) is not under shroud by the same test (the special-abilities specs own its image
    frames).
    - Backing: `measurable` (scenario OV14)
    - Confidence: high for the period, the image frames, the cell tested and its centre height,
      medium for the shroud condition as seen on screen and for the case below level 0

## Formulas

**F1. Filled health segments.** *H* and *S* are the integer health and full strength, *S* > 0 (claim
5 gives *f* for *S* ≤ 0). Write ⟨*e*⟩ for the binary64 result of the single operation *e*, rounded
toward zero (the round-toward-zero mode, planned S-0205):

> *f* = min(*n*, max(1, trunc(⟨⟨*H* / *S*⟩ × *n*⟩))),  where ⟨*H* / *S*⟩ is the health ratio *h*

With a significand of 53 bits or more the result does not depend on the width (53 and 64 bits give
the same *f*): rounding toward zero never lifts *h* or ⟨*h* × *n*⟩ above the exact value, and the
error, below 2<sup>−51</sup>·*H*·*n* / *S*, is smaller than the distance 1/*S* from a non-whole
*H*·*n* / *S* to the next whole number whenever *H*·*n* < 2<sup>51</sup>, which holds for every
health that fits a 32-bit integer and every *n* of this spec. (A 24-bit significand would not do:
its error, up to about 2<sup>−22</sup>·*n*, reaches 1/*S* for full strengths above roughly
2<sup>22</sup> / *n*.) So for 0 ≤ *H* ≤ *S*

> *f* = min(*n*, max(1, ⌊*H*·*n* / *S*⌋ − *c*)),  *c* = 1 when *H*·*n* / *S* is a whole number and
> *H* / *S*, in lowest terms, has a denominator that is not a power of 2; *c* = 0 otherwise

For example *H*/*S* = 3/17 with *n* = 17 gives *f* = 2, where ⌊*H*·*n* / *S*⌋ = 3; every ratio *k*/17
with 2 ≤ *k* ≤ 16 shows one segment fewer than the exact product (*k* = 1 is lifted back to 1 by the
lower bound). With round-to-nearest, *f* would be 3. For *n* = 8 such a case cannot occur (8*H*/*S*
whole means a power-of-2 denominator). For the building values of claim 7 it can: for example *n* =
15 at *H*/*S* = 1/3. *n* is 17 or 8 (claim 6) or the building value of claim 7.

**F2. Resource pips.** *g*, *q* and *C* as in claim 14, *R* the row length. In binary64,
round-toward-zero mode:

> *n*<sub>ℓ</sub> = trunc((ℓ / *C*) × *R* + 0.5) for ℓ = *g* and for ℓ = *q*, each operation
> rounded toward zero

*g* is a single-precision amount widened exactly to binary64; *q* is the binary64 sum of claim 14,
(*a*<sub>3</sub> + *a*<sub>2</sub>) + *a*<sub>0</sub> with each addition rounded toward zero, of three
single-precision amounts; *C* and *R* are integers, exact in binary64. *C* = 0 gives an infinite or
undefined quotient in the original, whose result is *unspecified*; our engine draws no resource pips
then, and the importer warns about a type with a resource pip scale and no storage capacity.

**F3. Wrapped ammunition** (integers): pip *j* of *w* shows image frame 15 + max λ over 0 ≤ λ ≤
⌊*A* / *w*⌋ − 1 with λ·*w* + *j* < *a*, or 14 when no λ qualifies.

**F4. Wrench image frame** (integers): *p* = max(2, ⌊N(14, *s*) / 4⌋), image frame = ⌊6 (*t* mod
*p*) / (*p* − 1)⌋, where *t* is the number of ticks since the game started. N(14, *s*) = ⌊112 / (*s* + 1)⌋ by
S-0107 F3, so *p* is 28, 14, 9, 7, 5, 4 and 4 for *s* = 0 … 6.

**F5. Building health-bar geometry** (integers, pixels, truncation toward zero). For a foundation
W × D, α = 128·W and β = 128·D leptons, and the projected displacements of Terms:

> *Q* = P(−α, β, *V*) = (−15(W + D), trunc(15(D − W) / 2) − *L*(*V*))
>
> *Q*′ = P(−α, −β, *V*) = (15(D − W), trunc(−15(W + D) / 2) − *L*(*V*))
>
> *n* = trunc((*Q*<sub>y</sub> − *Q*′<sub>y</sub>) / 2) = trunc((trunc(15(D − W) / 2) − trunc(−15(W + D) / 2)) / 2)
>
> *o* = P(−α, β, 0) = (−15(W + D), trunc(15(D − W) / 2))

*Q* places the building health pips (claim 7) and *o* the building pip scale (claim 10).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Health segments, vehicle or aircraft | 17 | segments | claim 6 |
| Health segments, infantry | 8 | segments | claim 6 |
| Segment spacing | 2 | pixels | claim 6 |
| Building health pip step | (−4, +2) | pixels | claim 7 |
| Pip spacing, pip scale | 4 | pixels | claim 10 |
| Culling margin | 360 × 180 | pixels | claim 1 |
| Height of one level | 104 | leptons | building box, claim 7 |
| Resource pips | 3 (infantry), 5 (others) | pips | claim 14 |
| Power pip-scale length | 10 | pips | no row of its own is drawn (claim 11); a type with a passenger capacity uses it as its passenger row length (claim 12) |
| Overload pip highlight | 10 | ticks | claim 13; not normalised for game speed |
| Self-heal blink | 6 | ticks per period | claim 16 |
| Wrench base delay | 14 | ticks before normalisation | claim 19 |
| Wrench image frames | 7 | image frames (0 … 6) | claim 19 |
| Building bracket depth threshold | 4 | leptons below the terrain (height above the terrain below −4) | claim 8 |
| Selection colour | 15 (12 below the depth threshold) | palette index | claim 8 |

**T1. Image frames of the pip sprite used here.**

| Image frame | Use |
|---|---|
| 0 | empty pip (building health, passengers, resource, spawns, mind control) |
| 1 … 5, 7 … 12 | passenger pip colours (Import mapping); 1 green building health, full spawn or mind-control pip and "other" passenger, 2 yellow building health and resource, 4 red building health and vehicle in an absorbing building, 5 resource at registration index 1 (claim 14) and vehicle passenger |
| 3 | extra passenger space; overloaded mind control just after a pulse |
| 4 | overloaded mind control otherwise |
| 6 | empty occupant slot |
| 13 | ammunition, no wrap (second pip sprite); infantry-healing self-heal marker (pip sprite) |
| 14 … | wrapped ammunition, 14 empty and 15 + layer (second pip sprite); 14 veteran and 15 elite insignia (pip sprite) |
| 16, 17, 18 | green, yellow, red health segment |
| 19 | insignia below rookie |
| 20 | unit-repair self-heal marker |

Which of the two pip sprites holds an image frame depends on the object kind and the pip kind (claim 10).
The same number can mean different pictures in the two.

**T2. Row length by pip-scale kind** (used by claims 12 to 15).

| Pip-scale kind | Row length |
|---|---|
| none | 0 |
| ammunition | the type's ammunition capacity |
| resource | 3 for an infantry type, 5 for any other type |
| passengers | the type's passenger capacity |
| power | 10 (no row of its own is drawn, claim 11; the passenger row of claim 12 uses this length) |
| mind control | the damage of the type's primary weapon; 0 when the type has none (departure, claim 13) |

## State and ordering

| Object state | Health bar | Pip scale | Insignia |
|---|---|---|---|
| selected | yes, with bracket (units) or corner brackets (buildings) | if claim 9 allows | if claim 18 allows |
| hovered | yes, no bracket | if claim 9 allows | if claim 18 allows |
| neither | no | no | if claim 18 allows |

Within one object, where they overlap, the selection marks and the health bar cover the insignia,
and the pip scale covers the health bar.

The overload pip (claim 13) and the self-heal blink (claim 16) change with ticks: every drawn frame
between two ticks shows the same state.

## Edge cases and quirks

- **`quirks.building_resource_pips_hidden`** (default: on). A building with the resource pip-scale
  kind shows no pip scale at all (claim 11). Off: it shows the resource row of claim 14 with the
  building row of claim 10, and its group number as usual (buildings show no self-heal marker,
  claim 16).
- **Pip-value retention** (display-only departure, no switch; claim 12). The original shifts the
  passenger pip colour and occupant pip image frame on successive reads of an infantry section
  that omits the corresponding key, a display-only bug. Our engine retains the held values under
  S-0003 claim 16 and S-0006 claim 5; omission alone produces no importer warning.
- **The game-wide enemy-health flag does nothing.** The original reads it, but no overlay changes
  with its value. Our engine imports it and ignores it, with no switch, since there is no behaviour
  to reproduce.
- **Passenger row overflow** (departure, no switch). The original hides the passenger row,
  self-heal marker and group number when the passengers' total size exceeds the row length, a
  display-only bug whose departure is confirmed by the project lead. Claims 11 and 12 require
  clipped passenger pips and ordinary marker visibility instead, without changing passenger
  capacity or loading legality. The importer warns about a type with a passenger capacity whose
  pip-scale kind is not passengers.
- **Resource pips round half up per resource** (claim 14), so the row can show one pip more or one
  pip fewer than the whole load would round to. No switch: it is what the original shows.
- **Building at health 0 shows green** (claim 5). No switch: no stable state shows it.
- **Mind-control kind without a primary weapon** (claim 13): the original fails; our engine takes a
  row length of 0 and the importer warns (departure, no switch).
- **Full strength of 0 or below** (claim 5): one filled segment and the colours given there,
  reproduced; the importer warns. No switch: the result is deterministic and harmless.
- **Bad passenger data** (claim 12): a size below 1, a gunner type with row length 0, and more
  passengers in an absorbing building than its row length give *unspecified* results in the
  original; our engine's results are given in claim 12, with importer warnings where the data can
  be checked at import.
- **Self-heal interval of 0 or below** (claim 16): reproduced for negative values, safe result for
  0 (departure, no switch), importer warning for both.
- **Round-toward-zero mode** (no switch here). F1 and F2 depend on it; the switch
  `quirks.chop_rounding` belongs to S-0205 (number domains, planned), and F1 and F2 are among the
  results it covers.
- **`quirks.speed_normalised_delays`** (default: on, shared with S-0107, S-0108
  and S-0109 (animation playback, planned)). Claim 19.

## Import mapping

| Section and key | Form | Feeds | Absent |
|---|---|---|---|
| rules `[AudioVisual]`, `ConditionYellow` | decimal or percentage ratio (S-0003) | yellow ratio (claim 5) | 0.5 (as S-0108) |
| rules `[AudioVisual]`, `ConditionRed` | decimal or percentage ratio | red ratio (claim 5) | 0.5 (as S-0108) |
| rules `[AudioVisual]`, `EnemyHealth` | boolean | nothing (Edge cases) | — |
| rules `[General]`, `SelfHealInfantryFrames` | integer, ticks | *T* for infantry healing (claim 16) | 1000 |
| rules `[General]`, `SelfHealUnitFrames` | integer, ticks | *T* for unit repair (claim 16) | 1000 |
| type section, `PixelSelectionBracketDelta` | integer, pixels | δ (claim 6) | 0 |
| type section, `PipScale` | named value: `none`, `ammo`, `tiberium`, `passengers`, `power`, `mindcontrol` (S-0003 claim 16) | pip-scale kind (claim 11) | none |
| type section, `PipsDrawForAll` | boolean | claim 9 (buildings only) | no |
| type section, `PipWrap` | integer | *w* (claim 15) | 0 |
| type section, `Ammo` | integer | *A* (claim 15) | as the combat specs give |
| type section, `Passengers` | integer | passenger capacity (claims 11, 12) | as the transport specs give |
| type section, `Size` | decimal | *z* (claim 12) | as the transport specs give |
| type section, `Storage` | integer | *C* (claim 14) | as the economy specs give |
| type section, `SpawnsNumber` | integer | spawn slots (claim 11) | 0 |
| type section, `Organic` | boolean | healing ability used (claim 16) | as the combat specs give |
| type section, `Primary` | weapon name | *M* is that weapon's damage (claim 13) | none (then *M* = 0 and an importer warning for the mind-control kind) |
| type section, `Gunner` | boolean | gunner gap in the passenger row (claim 12) | no |
| infantry section, `Pip` | named value (vocabulary below) | passenger pip colour (claim 12) | keep held value; initially 1 |
| infantry section, `OccupyPip` | named value (vocabulary below) | occupant pip image frame (claims 11, 12) | keep held value; initially 7 |
| building section, `CanBeOccupied` | boolean | claim 9 | no |
| building section, `ShowOccupantPips` | boolean | occupant row (claim 11) | yes |
| building section, `MaxNumberOccupants` | integer | occupant slots (claim 11) | 0 |
| building section, `InfantryAbsorb` | boolean | absorbing building (claim 12) | no |
| building section, `UnitAbsorb` | boolean | absorbing building (claim 12) | no |
| building's art section, `Height` | integer, height levels | *V* of the building box (claims 7, 8) | 2 |

Vocabulary of `Pip` and `OccupyPip`, compared ignoring case (S-0003 claim 16), with the image frame
each word selects: `green` 1, `yellow` 2, `white` 3, `red` 4, `blue` 5, `persongreen` 7,
`personyellow` 8, `personwhite` 9, `personred` 10, `personblue` 11, `personpurple` 12.
A word that matches none gives 1, with an importer warning (S-0003 claim 16). An absent read
retains the held value, initially 1 (`Pip`) and 7 (`OccupyPip`), as claim 12 requires.

Overlay sprites, resolved by S-0005: `PIPS.SHP` (pip sprite), `PIPS2.SHP` (second pip sprite),
`PIPBRD.SHP` (bracket sprite), `WRENCH.SHP` (wrench sprite). Pips and brackets use the overlay palette
`PALETTE.PAL`; the wrench and the bomb sprite use `MOUSE.PAL`. A missing sprite draws nothing; the
importer warns.

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*, a tiny flat purpose-made map, one human player and
one computer player, neutral lighting, a fixed screen resolution, screenshots compared pixel by pixel
with the overlay sprites and palettes from the stock archives. Health is set by attacking with a stock
weapon and read with a recorder that reads the running game's state; the recorder also gives each
object's centre, from which the status anchor follows by S-0101. Where a scenario needs data that
stock types do not have, it uses a test map whose override sections change only the named fields,
documented with the scenario. Unless a scenario says otherwise, one run is enough: no random draw is
involved.

- **OV1 (claim 1).** Scroll so that a selected vehicle's anchor moves out of the view in steps of 10
  pixels. Expected: its bracket and bar stay drawn while the anchor is within 360 pixels left or
  right (180 above or below) of the view, and vanish beyond. Falsified if they vanish earlier or stay
  later.
- **OV2 (claim 2).** A selected veteran transport of the player, with one passenger, at the edge of
  the shroud and in fog; a damaged building of the player being repaired, partly in fog; a unit of
  the player carrying an attached bomb (if a stock weapon attaches one), in fog. Then the same
  scene in a test map with night lighting. Expected: the bracket, the insignia, the wrench and the
  bomb sprite are darkened in fog and under shroud as S-0110 (shroud rendering) F1 gives, and drawn at
  neutral brightness whatever the map lighting; the health segments and the passenger pips keep their
  palette colours in fog, under shroud and at night. Falsified if the segments or pips darken, or the
  bracket, insignia, wrench or bomb sprite does not darken in fog, or any of them follows the night
  lighting.
- **OV3 (claims 3, 6).** A stock vehicle, infantry unit and aircraft of the player and of the computer
  player, each at several healths. Select and hover each. Expected: the segment rows of claim 6 at the
  displacements given (δ read from the type's field), the bracket only when selected, enemy bars on
  hover. Falsified if any displacement or image frame differs.
- **OV4 (claims 4, 5).** A test map gives one stock vehicle type a strength of 160 and sets the
  yellow and red ratios to 0.5 and 0.25 (exact in binary64), and gives a second stock vehicle type a
  strength of 170. The first vehicle is taken through healths 81, 80, 41 and 40. Expected: green,
  yellow, yellow, red (each colour switches at equality). A building of the player given the test
  strength 160 is taken through the same healths. Expected: the same colours. The second vehicle is
  taken through healths 170 down to 1. Expected: *f* as F1 gives at every health, in particular 2
  filled segments at health 30 (*h* = 3/17), not 3, and 1 segment at health 1. Falsified if a colour
  changes one point late or early, or *f* differs (3 segments at health 30 would mean
  round-to-nearest).
- **OV5 (claims 7, 8).** Buildings of the player with foundations 1 × 1, 1 × 2, 1 × 4, 2 × 2, 3 × 4
  and 4 × 4 (stock buildings where these exist, otherwise test-map foundations), each selected.
  Expected: 7, 14, 29, 15, 29 and 30 health pips, stepping (−4, +2), filled first in the health
  colour then empty image frame 0, the last pip 7 pixels right of and 2 below *Q* of claim 7 (for a 2 × 2
  foundation with an art height field of *e* levels, *Q* = (−60, −15*e*) from the anchor); corner brackets of
  quarter edge length in index 15. Falsified if a
  pip count differs (15 for 1 × 2 would mean ⌊15·D / 2⌋), the step or *Q* differs, or the brackets
  have another length or colour.
- **OV6 (claims 9, 10, 11).** Own and enemy transports with passengers, an own and an enemy
  occupiable building, an own building of a stock type with the resource pip-scale kind (the
  "refinery" below) and an own harvester; then a test type that spawns aircraft,
  has the ammunition kind and no passenger capacity. Expected: pip scales on own objects and on the
  enemy occupiable building only. For the vehicles and infantry, the placements of claim 10 from *o* =
  (−10, +10). For a 2 × 2 occupiable building, *o* = (−60, 0) from the anchor and occupant pip *j*
  at (−54 + 4*j*, −1 + 2*j*); for a 3 × 2 one, *o* = (−75, −7). The refinery shows none. The test
  spawner shows the spawn row with the ammunition row covering it. Falsified if an enemy transport
  shows pips without being spied on, the refinery shows pips, a building row is placed elsewhere, or
  the test spawner lacks either row. Then a test map whose override sections give one stock enemy
  building type that a spy can infiltrate a passenger capacity of 3 and the passengers pip-scale kind
  (so that its pip scale is a row of three empty pips, image frame 0); a spy of the player infiltrates
  one such building and a second one is left alone. Expected: the empty row on the infiltrated
  building after the spy enters and not before, and none on the other. The same test map also gives
  a second enemy building type the same capacity and kind and marks it, and one stock enemy
  transport type, to draw their pips
  for all. Expected: the three empty pips on that building without any spy, and no pips on the empty
  enemy transport (its row of empty pips stays hidden). Last, an own unit of a stock type with the
  ammunition pip-scale kind and no passenger capacity, fully loaded. Expected: its ammunition pips 3
  pixels above the row places of claim 10, from the second pip sprite. Falsified if the infiltrated
  building shows no pip scale or the untouched one shows it, the draw-for-all mark shows pips on the
  transport or not on the building, or the ammunition pips sit at another height.
- **OV7 (claims 11, 12).** A stock transport of the player, of a type without the gunner flag, in control group 1, loaded in a known
  order with infantry of different pip colours and, if one exists, a vehicle passenger of size 2.
  Expected: read from pip 0, the passengers in loading order, each one's colour pip (image frame 5
  for the vehicle) followed by its image-frame-3 pips, then empty pips; the colours of T1 and Import mapping. Then a
  test map override section gives the transport type the pip-scale kind none. Expected while empty:
  its group number and no pips; the loaded departure case is tested separately in OV7-D.
  Then a test transport type with the gunner flag, the passengers pip-scale kind and a
  passenger capacity of 3 (so *R* = 3), with two
  infantry aboard: expected pip 0 at its claim 10 place, then two empty pip places, then pips 1 and 2.
  Then an absorbing building of the player that absorbs infantry, entered by two infantry of
  different pip colours: expected the second one's colour at pip 0, the first one's at pip 1, the
  rest image frame 0. Then measure the original's omitted-value bug: a named loose test copy
  of the base rules gives a stock infantry type passenger pip image frame 1 and occupant pip image
  frame 7. Compare a control map with no section for the type to a test map whose override section
  changes one unrelated field and omits both pip keys. The skirmish's mode file has no section for
  the type, so the map layer is the only read without the keys. At game speed 2, load one such
  infantry into the transport and, separately, occupy an own occupiable building with another of
  that type. Capture both selected hosts' pips, one run per map. Expected original result: frames
  1 and 7 on the control, 2 and 9 on the test map. Repeat with base passenger pip image frame 11:
  expected 11 on the control and 1 on the test map. These are oracle expectations for the bug,
  not our engine's corrected result. Falsified if the transport fill runs from the last pip, the
  image frames differ, the empty transport loses its group number under the kind none, the gunner
  row has no gap or a gap of another width, the absorbing building shows the first passenger at
  pip 0, or either omitted pip value remains unchanged in the original.
- **OV7-R (claim 12; departure conformance, not an original-game measurement).** Replay OV7's
  control and omission maps in our engine at game speed 2, with no AI orders in the observed area.
  Capture the selected transport and occupied building after the same loading and occupation
  orders, one run per map. Expected: frames 1 and 7 on both maps, and passenger frame 11 on both
  maps in the repeat. Repeat with `quirks.override_resets_keys` off: the result is unchanged.
  Finally omit both pip keys from the named test base rules as well: expected the initial frames
  1 and 7 on both maps. Any shift after omission, or a warning caused solely by omission, fails
  fixed retention.
- **OV7-D (claims 11, 12; departure conformance, not an original-game measurement).** Replay the
  common tiny-map setup in our engine at game speed 3. Use a non-gunner transport in control group
  1, its owner holding a unit-repair building, and three size-1 infantry passengers with distinct
  pip colours. Named test-map overrides give the transport passenger capacity 3, the ammunition
  pip-scale kind and ammunition capacity 2, hence *R* = 2. Select or hover the transport and capture
  screenshots after loading 0, 1, 2 and 3 passengers in a known order. Expected: always two row
  pips, filled in entry order then empty; at two passengers both pips remain visible, and at three
  the first two passengers' colour pips remain while the third lies beyond the row and is clipped.
  The self-heal marker and group digit remain visible in all four states. Repeat with the pip-scale
  kind none (*R* = 0): no passenger pips at any load, but the marker and digit remain visible.
  Deselect and move the pointer away: all pip-scale elements disappear. Repeat the hovered loaded
  case with an unspied enemy transport: its passenger pips remain hidden under claim 9, regardless
  of fullness. Record one screenshot per state; falsified by a missing in-row pip, any pip outside
  the row, suppressed eligible markers, or pip-scale elements visible against claims 3 or 9.
- **OV8 (claim 13).** A stock mind-controlling unit controlling 0, 1 and its maximum of units; then a
  stock unit whose type may control without limit, made to control more than its maximum, with test
  data setting the delay between overload damage pulses well above 10 ticks. Recorded per tick at
  game speeds 0 and 6; a pulse's tick is the tick in which the controlling unit loses health to the
  overload. Expected: image-frame-1 pips equal to the count; the extra pip at index *M* only while
  over the maximum, in image frame 3 in the 10 consecutive ticks starting with each pulse's tick and
  in image frame 4 otherwise, at both speeds. Falsified if image frame 3 lasts 9 or 11 ticks, or its length changes with
  game speed.
- **OV9 (claim 14).** A stock harvester (5 pips) loaded in steps with a mix of the resources at
  registration indices 0 and 1 (the recorder reads each resource's registration index). Expected: the
  counts of F2. A quarter of capacity in each resource gives trunc(1.25 + 0.5) = 1 image-frame-5 pip
  (index 1) and 1 image-frame-2 pip (index 0), 2 in all, where the whole load would round to 3; an eighth of capacity in each gives
  trunc(0.625 + 0.5) = 1 of each, 2 in all, where the whole load would round to 1 (both fractions
  are exact in binary64, so the round-toward-zero mode does not move them). If the map holds a
  resource of registration index 2 or 3 that the harvester can collect, a load of it adds to the
  image-frame-2 pips together with index 0. Falsified if the counts follow the total load instead, or
  an index 2 or 3 amount shows in image frame 5.
- **OV10 (claim 15).** A stock unit with ammunition and no wrap, and one with a pip wrap (test data in a
  test map override section if no stock type has one), fired until empty. Expected, at each
  ammunition count *a*: for the unit without wrap, image frame 13 for every pip *j* < *a* and no pip
  drawn from *j* = *a* on; for the unit with wrap *w*, exactly *w* pips in the image frames of F3.
  Falsified if any pip shows another image frame, or another number of pips is drawn.
- **OV11 (claim 16).** An own damaged infantry unit while the player owns an infantry-healing building,
  recorded every drawn frame for 3 periods; then a damaged own non-organic vehicle while the player
  owns a unit-repair building; then a test vehicle type marked organic, damaged, while the player owns
  only an infantry-healing building; then, with a test rules file that sets the infantry self-heal
  interval to a negative value −*T*₀, the infantry unit again. Expected: image frame 13 at (25, −35)
  from *o* for the infantry unit, image frame 20 at (33, −32) for the vehicle, image frame 13 at
  (33, −32) for the organic vehicle; the marker drawn as a shadow during ticks with *t* mod *T* ≤ 5 and as an image
  otherwise, with period *T*₀ under the negative interval; undamaged, always as an image; no marker
  on aircraft. Falsified if an image frame, place or the blink timing differs.
- **OV12 (claim 17).** A vehicle and an infantry unit put in groups 1, 5 and 10 of the keyboard.
  Expected: digits 1, 5 and 0 in the owner's house colour, placed from the point (−14, −26) from the
  vehicle's anchor and (−14, −29) from the infantry unit's; the screenshots also show how the text
  aligns to that point (Open questions). Falsified if the digit differs, or the text moves with
  anything other than that point.
- **OV13 (claim 18).** A small map with water. A test map whose unit entries give veterancy values of
  −0.5, 0.5, 1.5 and 2.5 levels (through the veterancy field of the map's unit entries, in hundredths:
  −50, 50, 150, 250) to units of both players; a disguised enemy unit; a cloaked own naval unit, a
  cloaked enemy naval unit away from the player's sensors, and the same enemy unit within range of a
  player's unit with sensors; the disguised and cloaked units are given 150 (veteran). Expected:
  image frame 19 for −0.5, none for 0.5, image frame 14 for 1.5, image frame 15 for 2.5, at (10, 6)
  or (5, 2), shown unselected and for the enemy; hidden for the
  disguised enemy; shown on the cloaked own unit; hidden on the cloaked enemy away from sensors and
  shown within sensor range. Then, in a skirmish with the computer player allied to the player, a
  cloaked allied naval unit away from the player's sensors: expected its insignia shown. And the own
  naval unit recorded every drawn frame while it cloaks and uncloaks: expected the insignia drawn
  throughout. Last, a single-player test mission (not skirmish or multiplayer) in which a computer
  house allied to the player owns a cloaked naval unit away from the player's sensors: expected its
  insignia hidden. Falsified if any image frame or band differs, the own cloaked unit hides its insignia,
  the cloaked enemy away from sensors shows it, the allied cloaked unit hides it in skirmish or
  shows it in the single-player mission, or the insignia
  vanishes during cloaking or uncloaking.
- **OV14 (claim 19).** A damaged own building being repaired, recorded per tick at game speeds 0, 3 and
  6. Expected: periods of 28, 7 and 4 ticks, image frames ⌊6 (*t* mod *p*)/(*p* − 1)⌋. Then the
  cell tested. The wrench is shaded (claim 2), so under a shroud piece it is black whatever the
  test gives; each case below is therefore a pair of states, read from screenshots together with
  the shroud pieces of S-0110: (a) the screen area under the wrench uncovered and the cell that
  claim 19 tests unexplored, expected no wrench; (b) that cell explored as well, expected the
  wrench. Only (a) tells the tested cell apart from the shading. A test map places a damaged 3 × 3
  building of the computer player, which repairs it, in unexplored territory, and a scout of the
  player explores towards it from the side away from the building's top-left cell, until the
  building's middle is uncovered and its top-left cell is not, then on until that cell is explored.
  The cases, with (*c*<sub>x</sub>, *c*<sub>y</sub>) the top-left cell:
  - flat ground at level 0: the tested cell is (*c*<sub>x</sub>, *c*<sub>y</sub>),
  - flat ground at level 2 on a plateau: (*c*<sub>x</sub> − 1, *c*<sub>y</sub> − 1), not the
    top-left cell itself,
  - if the map file can place the building with its top-left cell on a valley slope at level 0:
    the pair (*c*<sub>x</sub> − 1, *c*<sub>y</sub> − 1) and (*c*<sub>x</sub>, *c*<sub>y</sub>)
    (*u* = 1), so in (a) both are unexplored and in (b) one of them is explored, where the level
    height would make the top-left cell alone decide,
  - flat ground at level −2: the pair (*c*<sub>x</sub> − 1, *c*<sub>y</sub> − 1) and
    (*c*<sub>x</sub>, *c*<sub>y</sub>), and in a third state, with only cell (*c*<sub>x</sub> + 1,
    *c*<sub>y</sub> + 1) of the three explored and the area under the wrench uncovered, expected no
    wrench (the level height would test that cell).

  Then a unit carrying an attached bomb (if a stock weapon attaches one), in view and with its cell
  under shroud: expected the bomb sprite in view and none under shroud. Falsified if the period does
  not follow F4, a state (a) shows the wrench, a state (b) does not, or the bomb sprite ignores the
  shroud.

## Open questions

- The level height of 104 leptons used for the building box (claim 7) is believed, not confirmed
  for the overlay; scenario OV5 should pin it through *Q*.
- Which corners of a building's box get selection brackets (claim 8) is not established.
- Which picture each image-frame number holds in each pip sprite should be read from the stock sprites at
  measurement time; this spec fixes only the numbers.
- How a disguised unit's health bar behaves on hover (claim 3) belongs to the special-abilities specs.
- When a cloaked building's insignia is hidden from the local player (claim 18) is not established.
  The rule given there is for vehicles, infantry and aircraft.
- How the group number's text is aligned to its placement point, its font and the exact size of its
  border (claim 17) are not established; the colour is medium confidence.
- The extra-information overlay that some units draw beside the pip scale is not covered, nor is the
  talk bubble that can be shown over one object at a time (it uses its own sprite, drawn at the
  status anchor without shading; what shows it is not traced).

## Cross-spec consistency

- Q-0471 is fully answered by claim 12 and Import mapping: the former
  `quirks.pip_colour_default_shift` is withdrawn. Pip-value retention is mandatory, not a
  rendering option; OV7-R checks the corrected result. S-0003 claim 16 and S-0006 claim 5 agree.
- Hand off with S-0003 claim 16 and S-0006 claim 5 aligned to claim 12's fixed pip-value retention,
  initial image frames 1 and 7, and no pip-default switch. OV7 observes the original bug.
  OV7-R checks the correction in our engine.
- Hand off with S-0110 claim 8 owning the point test for both signs of height and
  `quirks.shroud_negative_odd_pair`. Claim 19 supplies the tested centre surface height and wrench
  observable, not an independent negative-height extension; OV14 retains the original expectations.
