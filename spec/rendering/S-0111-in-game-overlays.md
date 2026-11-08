---
id: S-0111
title: In-game overlays (health bars, pips, selection marks, insignia, name tip)
topic: rendering
revision: 57
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-08
measurements: []
requests: [Q-0471, Q-1622, Q-2040, Q-2071, Q-2072, Q-2673, Q-2687, Q-2736, Q-2791]
supersedes: []
---

# S-0111: In-game overlays (health bars, pips, selection marks, insignia, name tip)

## Summary

The status extras that S-0102 paints over every vehicle, infantry unit, aircraft and building after
the five render layers: where they are anchored, when the health bar shows, how many health segments
are filled and in which colour, the selection marks, the pip scale (passengers, spawned aircraft,
occupants, mind control, resource load, ammunition), the self-heal marker, the control-group number,
the veterancy insignia and the repair wrench, with the cursor palette that the wrench and the
bomb icon are drawn through and what happens without it (claim 19a). Which objects have status extras and the painting order
belong to S-0102. Skirmish only; campaigns and network games are out of scope. Conditional fog
behaviour is deferred context only, since S-0409 claim 19 forces fog off in skirmish.

It also gives the name tip (claims 20 to 26): the small text box that the interface shows over the
battlefield view when the pointer rests on an object, with its timing, the option that turns it
off, the objects that get one, its text, its placement, its look and when it goes. The display
names it shows are found as S-0004 claim 17 gives. The sidebar's own tips are S-0411's (claims
20 to 22).

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
| health ratio *h* | Current health *H* divided by the type's full strength *S*, the quotient rounded toward zero to binary64, as in S-0108 F3 (the round-toward-zero mode, S-0205). |
| yellow ratio, red ratio | The two game-wide condition ratios that S-0108 imports (Import mapping), binary64 values. |
| health colour | green, yellow or red (claim 5). |
| selected | Chosen by the local player as part of the current selection. |
| hovered | Under the local player's mouse pointer while not selected. |
| pointer object | The object that the local player's pointer is on, found as for *hovered* and for clicks, whether selected or not; none when the pointer is on no object. The pointer cell is the cell under the pointer found the same way. |
| name tip | The interface's text box for the pointer object (claims 20 to 26). Presentation only: it changes no game state. |
| rest, wait | The pointer *rests* while it does not move. The *wait* is the real time a rest must last before the name tip shows (claim 20). |
| display name | A type's text for the player, resolved from its display-name label as S-0004 claim 17 gives. Never missing: a type with no label has empty text. |
| turret-tip vehicle | A vehicle whose type has a turret count above 0 and the turret-tip flag (Import mapping). |
| pip sprite, second pip sprite, bracket sprite, wrench sprite | The overlay sprites of Import mapping. Image-frame numbers below refer to them. |
| pip scale | The row of small pips drawn with the health bar for an owner the local player may inspect (claim 9). |
| pip-scale kind | The type's imported pip-scale field: none, ammunition, resource, passengers, power or mind control. |
| row length *R* | The number of pips a type's pip-scale kind gives (Constants T2). |
| centred image | An image drawn with the centre of its sprite canvas at the given point (the canvas width and height halved and truncated). Every overlay image here is centred. |
| gunner flag | A type flag (Import mapping) marking a transport whose first passenger mans its weapon; it only shifts the passenger row here (claim 12). |
| absorbing building | A building type flagged to absorb infantry or to absorb vehicles (Import mapping); the passengers it holds fill its row differently (claim 12). |
| veterancy value *v* | A unit's experience as a single-precision number: below 0, from 0 to below 1 (rookie), from 1 to below 2 (veteran), 2 or more (elite). The combat specs own how it grows. |

## Behaviour

1. **Anchor and culling.** Status extras of an object are placed from its status anchor, except
   the bomb icon, which uses the carrier's draw point (claim 19). They are drawn only when the anchor lies within the battlefield view widened by 360 pixels on the left
   and the right and by 180 pixels at the top and the bottom. Fog of war does not hide them.
   - Backing: `measurable` (scenario OV1) for anchoring and culling; `unmeasured` for fog
     independence, since fog is always off in skirmish (S-0409 claim 19)
   - Confidence: high

2. **Shading of overlay images.** Selection brackets, the veterancy insignia, the repair wrench and
   the attached-bomb sprite are shaded by the light-alpha values of S-0104 claim 15 at neutral
   brightness, so the shroud and fog of S-0110 (shroud rendering) darken them. Health segments and
   pips are not shaded: they keep their palette colours under shroud and at night. The
   control-group number and its box are not shaded either (claim 17, scenario OV12 c).
   - Backing: `measurable` (scenario OV2) for shroud and lighting; `unmeasured` for fog
     shading, since fog is always off in skirmish (S-0409 claim 19). The fog rule is deferred
     context for later modes, not a reachable skirmish test. A sprite's shroud shading remains
     `unmeasured` if OV2 cannot expose an eligible nontransparent pixel beneath a shroud piece
   - Confidence: medium

3. **When the health bar shows.** The health bar of an object, of any owner, is drawn while it is
   selected, and while it is hovered. A hovered unit that *appears disguised* to the local player's
   house (S-0516 claim 35) shows a health bar only when its disguise has a house, rather than none
   (S-0516 claim 36). This restriction does not apply to a selected unit, or to a hovered unit that
   does not appear disguised. It uses *appears disguised*, not whether the unit is drawn as its
   disguise (S-0516 claim 37). The health bar is drawn at every health from full down to the last point.
   - Backing: `measurable` (scenario OV3)
   - Confidence: high

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
   - for a building only, the type is marked to draw its pips for all, or can be occupied by
     infantry. The draw-for-all mark has no effect on vehicles, aircraft and infantry.

   Otherwise only the health bar is drawn. Infiltration does not reveal an enemy object's pip
   scale (S-0510 claim 8). These rules cover skirmish only. This corrects this spec's previously
   approved claim 9, which offered spying as a pip-visibility condition; that condition is not
   reachable in stock skirmish. It is a correction of the described original behaviour, not a
   departure or a new quirk switch.
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
    | spawned aircraft | a type that spawns aircraft, whatever else applies | one per spawn slot of the type: the first *f* pips image frame 1 and the rest image frame 0, where *f* is the number of its slots that are docked or reloading |
    | occupants | a building type that shows its occupant pips and holds at least one occupant slot, whatever else applies | one per slot: pip *k* shows the occupant at list position *k* by its infantry type's occupant pip image frame, or image frame 6 when no occupant holds that position |
    | passengers | a type with a passenger capacity above 0, whatever its pip-scale kind | claim 12 |
    | by pip-scale kind | a type with a passenger capacity of 0 or below | mind control: claim 13; resource: claim 14; ammunition: claim 15; none and power: no row |

    **Spawned-aircraft row (Q-2072).** In S-0506 claim 4's slot states, a slot counts toward *f*
    while it is *docked* or *reloading*: its object is inside the spawner. A slot that is *on the
    rail*, *preparing* (a missile waiting on its launcher included), *attacking*, *returning* (on
    its way back) or *empty* does not. The full pips are packed from pip 0: they do not stand at
    their slots' own indices, so a launched slot leaves no gap. A slot that becomes docked or
    reloading again adds a full pip at the end of the full ones. So a launched aircraft's pip is
    empty for the whole time it is out, and its death, at crash start or later, changes no pip.

    **Occupant row (Q-2072).** The occupants are in S-0508's list order: pip 0 shows the first
    occupant, the one inside longest, and each later entrant takes the next pip (S-0508 claim 5).
    Occupants of different types each keep their own image frame at their own position. The only
    loss of one occupant while others stay that S-0508 claim 19 lists is its defeat death. The
    original keeps the dead place in the list (S-0508 claim 20), so no later pip moves, and what
    the dead place's pip shows is *unspecified*. Our engine removes the place (S-0508 claim 23),
    and every later occupant moves down one pip: this closing up is our engine's rule, not a
    claim about the original.

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
    - Backing: `measurable` (scenarios OV6, OV7) for the tested rows and spawn/ammunition overlap.
      `measurable` (scenario OV15) for the spawn states that count, the packing and the occupant
      order. `unmeasured` for the pip of a dead occupant place and for the later pips' staying in
      place in the original: S-0508 claim 20 gives no safe original result once a place is dead,
      so no scenario records it. The closing up is our engine's rule, checked on our engine only
      (scenario OV15 d).
      `unmeasured` for other multi-row overlaps and resource-building suppression of the group
      number: these scenarios do not establish those combinations. Overflow visibility is a
      departure tested against this spec, not the original (scenario OV7-D)
    - Confidence: high for the rows, the covering, the passenger rule and the resource cut-off,
      high for the lead-decided overflow visibility. High for the spawn states that count and the
      packing, and for the occupant order and positions while every occupant lives (OV15 a to c).
      Medium for the original's later pips staying in place after a defeat death (S-0508 claim
      20, unmeasured). The dead place's pip in the original is unspecified, not rated. The
      engine's closing up is a design rule (OV15 d)

12. **Passenger pips.** The row length *R* is the one that the type's pip-scale kind gives (Constants
    T2): the passenger capacity for the passengers kind, but 0 for the kind none, the ammunition
    capacity for the ammunition kind, and so on. A type whose pip-scale word is outside the
    vocabulary has the kind none (Import mapping), so its row length is 0 too.
    Let the *m* passengers be numbered *k* = 1 … *m* in
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
      binary64 under the round-toward-zero mode (S-0205). For sizes of 0
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
      against this spec in OV7-R, not an original-game expectation. `unmeasured` for fractional
      and negative passenger sizes, passengers other than infantry or vehicles, a zero-length
      gunner row and absorbing-building overflow: OV7 supplies no reachable controlled fixture
      for those cases, so their safe results and warnings are not oracle measurements.
      Passenger rows whose kind is power, resource or mind control are also `unmeasured`: OV7
      does not supply those mixed-capability hosts. Vehicle passenger blocks remain `unmeasured`
      if OV7's conditional vehicle fixture cannot be loaded; absorbing-building vehicle pips are
      `unmeasured`, because OV7 tests only infantry absorption
    - Confidence: high for fixed pip retention, the row length, total size, blocks, image frames and order.
      Medium for the gunner gap and the absorbing buildings
    - Overflow clipping backing: departure, tested against this spec in OV7-D, not the original,
      confidence high for the lead-decided rule

13. **Mind-control pips.** The row has *M* pips, *M* being the damage of the type's primary weapon
    (its control capacity, S-0601 claim 1). Counting from pip 0, as many
    pips as the unit currently controls show image frame 1, and the rest of the row image frame 0.
    A unit whose type may control
    without limit and that currently controls more than *M* (S-0601 claims 1 and 16)
    shows one more pip at index *M*, just after the row. That pip shows image frame 3 for 10 ticks
    from each damaging overload pulse (S-0601 claim 16): in the drawn frames that follow the tick of the pulse and each of
    the next 9 ticks. From the 10th tick after the pulse it shows image frame 4, until the next pulse.
    The 10 ticks are not normalised for game speed. Link lines and control rings are separate
    presentation effects owned by S-0601 claims 17 and 18, respectively.
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
      round-toward-zero mode (S-0205) as (*a*<sub>3</sub> +
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
    - Backing: `measurable` (scenario OV9) for the vehicle row, the split and the rounding.
      `unmeasured` for the infantry row and zero-storage safe result: OV9 provides neither a
      collecting infantry fixture nor a zero-storage load. `unmeasured` for
      resource indices 2 and 3 if OV9 cannot collect them: a skipped resource fixture is not
      confirmation. `unmeasured` for the order of the sum *q*: another order or rounding changes
      *q* by at most one unit in its last binary64 place, which moves *n*<sub>q</sub> only when *q* / *C* × *R* + 0.5 lies within that
      margin of a whole number, and no scenario can arrange that reliably
    - Confidence: high

15. **Ammunition pips.** The row length is the type's ammunition capacity *A*, and *a* is the current
    ammunition. With no wrap value (the type's pip wrap *w* = 0), pip *j* is drawn in image frame 13
    while *j* < *a* and is not drawn otherwise. With *w* > 0 only *w* pips are drawn, and pip *j* shows
    image frame 15 + λ, where λ is the largest integer from 0 to ⌊*A* / *w*⌋ − 1 with λ·*w* + *j*
    < *a*, or image frame 14 when there is none. So each further layer of wrapped ammunition moves a pip
    to the next image frame.
    - **Negative wrap (Q-2071).** A pip wrap below 0 is kept as read, and the type shows no
      ammunition pip at all, whatever its capacity *A* and its current ammunition *a*. Only the
      ammunition row is affected: the group number keeps its own rules.
      The importer warns about a negative pip wrap.
    - Backing: `measurable` (scenario OV10; OV10 b for the negative wrap). The importer warning is
      `unmeasured`: the original cannot confirm our diagnostics
    - Confidence: high for the rows with no wrap and with a wrap above 0. Medium for the negative
      wrap until OV10 b runs

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
    - Backing: `measurable` (scenario OV11) for the positive and negative periods and marker
      presentation; `unmeasured` for a zero period and its safe replacement, since OV11 avoids
      the failing original case and supplies no departure-conformance run
    - Confidence: high for the conditions, image frames, places and the blink period; medium for the
      darkened drawing during the blink

17. **Control-group number.** A unit or building in one of the local player's ten control groups
    shows the digit (*γ* + 1) mod 10 for group *γ* (0 to 9), so the first group shows 1 and the
    tenth shows 0. It is drawn with the pip scale at the placement point *d*: (−4, −39) from the
    pip-scale origin *o* of claim 10 for infantry and (−4, −36) for others, buildings included, so
    *d* is (−14, −26) from the status anchor for a vehicle and (−14, −29) for infantry. Claim 11
    gives the cases that show no number.

    **Glyph (Q-2040 questions 1 and 2).** The digit is one glyph of the game bitmap font: the same
    font, glyph widths and masks as the status text of S-0213 claim 25, with the file layout of
    S-0014. There is no separate set of fixed-size digit images. Each digit keeps its own imported
    width *G*, so digits of different widths give boxes of different widths. The glyph is
    left-aligned with its top stored row at *d*, mask pixel (*i*, *r*) at *d* + (*i*, *r*) (F6). No
    centring, baseline shift or shadow is applied.

    **Glyph rows.** Every stored row of the digit is eligible to paint, 0 ≤ *r* < *H*<sub>f</sub>,
    even when the line pitch *P*<sub>f</sub> is smaller than the bitmap height *H*<sub>f</sub>.
    This differs from the status text, whose rows S-0213 claim 25 limits to fewer than both its
    bitmap height and its pitch. The original sets no pitch limit for the digit and paints every
    stored row, so there is no extra row to correct, and our engine draws the same rows. The
    number is a single line, so no following line can overlap it.

    **Missing glyph.** A digit whose text unit has no mapped glyph uses S-0213 claim 25's
    generated fallback: the complement of the capital X's mask, with X's stored width as *G*, so
    the box takes the width that X gives. Our engine paints the fallback's set pixels in the house
    colour, as for any digit. The original paints them in the house colour with the packed
    foreground's even-numbered bits inverted, the cosmetic defect of S-0213 claim 25, while the
    border keeps the house colour. This display-only bug has no quirk switch.

    **Box (question 3).** Behind the glyph is a box whose size comes from the digit's text
    advance *G* + 1 (S-0213 claim 25) and the font's line pitch *P*<sub>f</sub>. Its top-left
    pixel is (−3, +2) from *d*, and its outer size is (*G* + 6) × (*P*<sub>f</sub> − 4) pixels
    (F6). The box is first filled with opaque black. Then its outermost ring of pixels, one pixel
    wide, is painted in the house colour, forming a closed rectangle around the digit alone. The
    glyph's set mask pixels are painted in the house colour last. Clear mask pixels leave the
    black fill, or the scene outside the box, unchanged. Set pixels in glyph rows above or below
    the box are painted over the scene. The gaps between glyph and border therefore follow from
    the font's data, not from a fixed margin. When *P*<sub>f</sub> − 4 ≤ 0 the digit is still
    drawn, and our engine draws no box. That box rule is a design rule: the original's pixels
    for such a font are recorded, not predicted.

    **Colour and shading (questions 4 and 5).** The digit and the border share one colour: the
    owner's house display RGB of S-0103 claim 15, at display precision. It is not a separate ramp
    entry or shade row. The number is drawn unshaded. Lighting, including the map tint, light source
    tints and the global screen effects of S-0104 claim 2, the shroud of S-0110 and a tint from the
    colour-adjustment table on the object itself, such as the invulnerability effect's, change
    neither the digit, the border nor the black fill. Every object has an owning house in skirmish,
    so the colour always comes from that house and no ownerless case arises.

    **Order (question 6).** The number and its box are drawn last among the object's own status
    extras: after the insignia, selection marks, health bar, pip rows and self-heal marker. They
    cover those extras where they overlap. They are not drawn in a separate pass above every
    object. Each object's status extras, the number included, are complete before the next
    object's begin, in the object order of S-0102 claims 1, 6 and 7. A later object's status
    extras therefore cover an earlier object's number and box. An earlier object's status extras
    lie under a later object's opaque box.

    **View edges and a missing font.** At the battlefield view's edges our engine cuts the
    finished box and digit like any other overlay image. The original cuts the box to the view
    before outlining it. A box cut by the view's top, right or bottom edge therefore shows a
    border-coloured line along that edge, a display-only bug with no quirk switch. At the left
    edge the original's border falls one pixel outside the view, so no line shows there and the
    visible part matches the cut image. Without a usable game font (S-0014 claim 6) our engine
    draws neither digit nor box. S-0213 claim 25's missing-asset diagnostic covers the font.
    - Backing: `measurable` (scenario OV12) for the digit, placement, glyph, alignment, box, colour
      and both orders (parts a, b, d and e). Also `measurable` for the unshaded number under
      lighting, the shroud and the invulnerability tint (part c), the original's
      lines at the four view edges (part f), and the stored rows, the rows outside the box, the
      digit drawn when *P*<sub>f</sub> − 4 ≤ 0 and the fallback glyph (part g). The covering of the
      object's own extras stays `unmeasured` if OV12 a exposes no own pixel under the box. The
      shroud result stays `unmeasured` if OV12 c can place no box pixel over a shrouded cell. The
      screen effects' result is `unmeasured`: no OV12 part captures one. The
      cut edge, the fallback's house colour, the absent box for *P*<sub>f</sub> − 4 ≤ 0 and the
      missing-font result are design rules, not oracle predictions
    - Confidence: high for the digit, the placement point, the font, the left alignment, the box
      geometry and fill, the shared colour sample and the absence of shading under lighting. Medium
      for both orders, the shroud, screen effects and object tints leaving the number unchanged, the
      stored rows painting without a pitch limit, the rows outside the box, the digit still drawn
      when *P*<sub>f</sub> − 4 ≤ 0, the fallback glyph and its colour defect, and the original's
      line along the top, right and bottom view edges with none along the left

18. **Veterancy insignia.** Every unit and building whose veterancy value is outside the rookie band
    shows an insignia, whether or not it is selected and whoever owns it: veteran image frame 14, elite
    image frame 15, below 0 image frame 19, all from the pip sprite, as a centred image at (5, 2) from
    the anchor for infantry and (10, 6) for everything else. The insignia is not drawn while the unit
    *appears disguised* to the local player's house (S-0516 claims 35–36), even if it is currently
    drawn as its real type. Nor is it drawn when cloak presentation hides the object from that viewer
    (S-0516 claim 28). A fully cloaked object remains visible, insignia included unless the disguise
    restriction applies, when the local player owns it, senses its cell or is mutually allied with
    its owner. Sensor coverage is S-0516 claims 21–23's, not shroud sight.

    A building at fade levels 11 to 15 uses that same fully cloaked presentation rule for its insignia,
    even before its cloak state becomes fully cloaked. At fade levels 0 to 10 its insignia is not
    suppressed by cloak. A fading vehicle, infantry unit or aircraft retains its insignia except in
    the non-owner hidden band of S-0516 claim 28; disguise suppression still applies. These claims
    cover skirmish only; campaigns and network games are out of scope.
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
    carrying an attached bomb shows the bomb sprite only when the bomb is visible to the local
    player (S-0519 claim 10) and its cell (the cell of its position) is not under shroud by the
    same test. Its image frames follow S-0519 Formulas F3, not the wrench's animation.
    The bomb icon is centred on the carrier's draw point, not on the status anchor; for a building
    that is the top corner of its foundation (S-0108 claim 8). It is the first of the carrier's
    status extras, so the health bar and selection marker paint over it. S-0519 claim 24 gives its
    image, palette and brightness. For vehicles, infantry and aircraft the draw point coincides
    with the status anchor (S-0519 claim 24). The wrench and the bomb icon are drawn through the
    cursor palette; claim 19a gives it, and what is drawn when it is missing.
    - Backing: `measurable` (scenario OV14; S-0519 scenario T25 for bomb placement and overlap)
    - Confidence: high for the period, the image frames, the cell tested and its centre height,
      high for bomb placement and first place among the carrier's extras; medium for the shroud
      condition as seen on screen and for the case below level 0
    - Coverage limit: the slope-height and below-zero cases are `unmeasured` until OV14's
      conditional placements are demonstrated reachable. Attached-bomb shroud visibility is
      `unmeasured` if OV14 cannot expose a visible bomb sprite and its tested cell's shroud states.
      A skipped placement is not confirmation. The non-default wrench period is an engine
      alternative tested against this spec in OV14-D, not an original-game measurement

19a. **Cursor palette (Q-1622).** The cursor palette is one palette file, resolved by S-0005 like any
    other file and kept like the other kept files of S-0005 claim 22, so its lifetime follows
    `quirks.art_cache_never_invalidated`. With the switch on (YR) it is read once when the program
    starts, before any game, and kept across games for the whole session (across a change of
    theater or side, see below). With the switch off it is resolved
    and read afresh at the start of each game, like every kept file. Each file component *c*
    becomes the palette component (4·*c*) mod 256, the rule S-0103 claim 1 gives for the palettes it
    lists, so a component above 63 wraps (64 gives 0). The cursor palette follows
    `quirks.palette_component_wrap` as those palettes do: with the switch off, such a component
    becomes 252 (S-0103). Drawn pixels then take S-0103 claim 9's
    output step like any other palette's. With `quirks.art_cache_never_invalidated` on, its colours stay
    the same across games within one session, also across a change of theater or side (`unmeasured`:
    S-0005 claim 22 discards the interface palettes whenever the side group is mounted, and no run
    here replaces the file across such a change, so a re-read only there is not ruled out); with that switch off, they stay the
    same within one game, whatever its theater and sides, and a file replaced between games is
    followed. Neither the overlay palette nor the unit palette in force supplies them (OV14
    palette part a). On a stock install the file is found inside the cache archive nested in the
    base game's main archive (S-0005 claim 10; medium, pending OV14 palette part 0). If the cursor
    palette cannot be found when it is read, the original fails, and never draws a wrench or a bomb
    icon. Our engine (departure, no switch): with no cursor palette, the
    wrench and the bomb icon are not drawn, and the importer warns once, naming the file.
    - Backing: `measurable` (scenario OV14, palette part a) for the palette the two sprites are
      drawn through; `measurable` (OV14 palette parts c and e) for the conversion of S-0103 claim 1,
      including the wrap above 63, and the switch-off value is our engine's fixed behaviour of
      S-0103's switch, checked on our engine by part e's engine fixture; `measurable` (OV14 palette part c) for colours that stay those of the
      file present when the program started, across games, with the switch on; part b replaces no
      file and backs none of this, and part c does not change theater or side between its games, so
      that a replaced file is also not followed across a theater or side change is `unmeasured`; and
      the switch-off lifetime is our engine's fixed behaviour of S-0005 claim 22, checked on our
      engine by part c's engine fixture; `measurable` (OV14
      palette part d) for the moment of the read, at start-up; `measurable` (OV14 palette part 0)
      for the file being found on a stock install; `unmeasured` for the missing file, because
      removing it would change the stock archives; the
      importer warning is `unmeasured`, as for this spec's other importer warnings.
    - Confidence: medium for the palette used and its conversion, because OV14 palette parts a
      and e identify them through predictions that include claim 2's light-alpha shading (itself
      medium); high for the switch-off value, a design rule shared with S-0103's palettes; medium for the failure without
      the file (`unmeasured`, and its moment is that of the read, itself medium below),
      medium for the colours staying the same across games within one session with the switch on
      (the original's behaviour), pending OV14 palette part c; medium, `unmeasured`, for the same
      across a theater or side change; medium for the file being found on a stock install, pending
      OV14 palette part 0; high for the switch-off lifetime, a design rule shared with every
      kept file; medium for the read happening at start-up rather than at first use, pending OV14
      palette part d, which tells the two apart only as far as S-0005 claim 1's test for a loose
      file at every open holds (itself medium); medium for the importer warning.

20. **Name tip: when it shows (Q-2673, Q-2687).** The name tip shows only while the player's tooltip
    option is on (Import mapping). It is on unless the player's settings turn it off, and both
    options menus change it. With the option off, no name tip ever shows.
    - **Wait.** The tip shows once the pointer has rested on the battlefield view for 1000 ms of
      real time. Every pointer movement in the game's window, over the sidebar too, of any distance
      and whether or not the pointer object changes, hides a tip that is showing and starts the wait
      again. The one exception is over the sidebar's build-icon slots (S-0411 claim 20): the
      movement that brings the pointer onto them is an ordinary one, but once one drawn frame has
      passed with the pointer on a slot (medium, `unmeasured`, Backing), each later movement that
      ends on a slot shows that slot's sidebar tip at once, and one that ends on an empty slot hides
      the tip. Pressing or releasing the left, right or middle mouse button hides the tip and stops
      the wait. No name tip then shows until the pointer has moved and rested again (over the slots,
      S-0411 claim 20 gives when a sidebar tip shows again). The mouse wheel affects neither. The pointer
      leaving the game's window is not an event of its own: the last movement inside the window
      restarts the wait as usual, and a wait that ends with the pointer outside the window shows
      nothing and uses up the rest's choice.
    - **Real time, not ticks.** The wait and the lifetime below are real time. Game speed does not
      change them. Our engine keeps this real-time timing: the tip feeds no game state.
    - **Where the pointer rests.** When the wait ends, a name tip is chosen only if the pointer is
      then over the battlefield view. Over the sidebar, the radar included, no name tip shows (the
      sidebar's own tips are S-0411's, claims 20 to 22), and outside the game's window nothing
      shows. Drawing on the battlefield view, such as the status message lines of S-0213 claim 25,
      is not a place of its own: the tip is chosen for the pointer object under it, as anywhere else
      on the view.
    - **One choice per rest.** When the wait ends, the text is chosen from the state at that moment
      (claims 21 to 23). If there is no text, the text is empty, or the wait ends while the tip is
      blocked (below), nothing shows. Nothing then shows until the pointer moves and rests again,
      even if an object later comes under the resting pointer or the block ends. While the tip
      shows, claim 26 chooses its text again.
    - **Lifetime.** A tip stays for 10000 ms of real time after it shows, and then hides. A name tip
      does not show again until the pointer moves and rests again (over the slots, S-0411 claim 20
      gives the sidebar tip's lifetime). A text chosen again while the tip
      shows (claim 26) does not start the lifetime again.
    - **Blocked.** The tip is blocked while the mouse cursor is hidden, while the player's input is
      locked, while the battlefield view is moving, as during a scroll, and while a left or right
      mouse button that was pressed over the battlefield view is held, as while a selection box is
      dragged. That block ends at the button's release, in the route mode too (whether the original
      ends it there after an ordinary left click in the route mode released over a highlighted or
      refused route point is an Open question). A held middle button does not block it. In the route
      mode (S-0213 claim 16), a held right button blocks the tip as it does outside the route mode.
      So does a held left button whose press is an ordinary click there, inside a point's click area
      or outside every one (S-0213 claim 16b): one that drags a band box, one that adds a point or
      selects at its release, and one that the route mode refuses only at its release, for route
      state or the route-holder limit (S-0213 claim 16d). A held left button does not block the tip
      when the route mode refused its press at the press, for kind (S-0213 claim 16d) or by the
      point-click test (S-0213 claim 16b), or when its press took a point, a join or a loop close
      (S-0213 claim 16b), or, with no object selected, took an own point that the click then selects
      (S-0213 claims 16b and 16e). Such a press starts no selection box, and after a movement with
      its button still held, the tip shows once the pointer has rested for 1000 ms, unless another
      block holds. A press that takes a point differs from a click that adds a point: the latter is
      an ordinary click and blocks the tip. An order block (S-0201 Terms) locks the input and hides
      the cursor: in a skirmish, the block of a game's first ticks (S-0201 claim 14) and a block
      that the map's trigger actions make, a chrono flash's included (S-0201 claim 11). No other
      ordinary skirmish situation is known to lock the input or hide the cursor while the pointer is
      over the battlefield view. These blocks are the name tip's. Over the sidebar only the hidden
      cursor blocks the sidebar's own tips (S-0411 claim 20): an order block blocks them through
      the cursor it hides, and a locked input alone does not.
    - **Options menu.** Opening the in-game options menu (S-0213 claim 23) hides a tip that shows
      and stops the wait. While the menu is open, no wait runs and no tip shows. When it closes, the
      tooltip option in force applies again, and no name tip shows until the pointer moves and
      rests again: a tip hidden by the menu does not come back (over the slots, S-0411 claim 20
      gives the sidebar tip's).
    - Backing: `measurable` (scenario NT1 for the wait, its restart, the buttons, the held buttons,
      the wheel, the lifetime, game speed, the one choice per rest; NT2 for the option, its settings
      key, both options menus and the menu's opening; NT8 for a scroll and a held left button; in
      the route mode, NT1 (h) runs 1 and 2 for an ordinary left press and a held right button, run 3
      for a click refused at its release, runs 4 and 5 for presses refused at the press and run 6
      for a press that takes a point). `unmeasured` for a press in the route mode with no object
      selected that takes an own point: every NT1 (h) run presses with an object selected.
      `unmeasured` for a held left button that drags a band box in the route mode: no NT1 (h) run
      moves the pointer with the left button held by more than S-0214 claim 18's band-box distance
      and rests with the button held, and run 3's band box only sets up its selection. `unmeasured`
      for the end of the block at the button's release in the route mode: no NT1 (h) run moves the
      pointer and rests after the release, and the release itself hides the tip and stops the wait,
      so a block that stays after the release gives the same record. `unmeasured` for the end of the
      block after an ordinary left click in the route mode released over a highlighted or refused
      route point: no NT1 (h) run releases such a click over a point, and that end is an Open
      question. `unmeasured` for a locked input and a hidden cursor: the block of a game's first
      ticks lasts 4 ticks, into which no set-up here can time the end of a 1000 ms real-time wait,
      and a test map's block would have to be timed against that wait, which no NT part does.
      `unmeasured` for the pointer outside the window: whether it can leave the window depends on
      the display mode, which no set-up here fixes. `unmeasured` for a rest over a status message
      line: a set-up here can produce such a line at a chosen time only by S-0213's route refusals
      (its scenario O21), whose fixtures this spec does not take over. `unmeasured` for the menu's
      closing bringing no tip back: closing it takes a press on the menu or a key that S-0213 claim
      23 does not give, and a press hides the tip anyway. `unmeasured` for a rest over the sidebar
      giving no name tip and for a movement over the sidebar, outside its build-icon slots, starting
      the wait again: the battlefield view does not include the sidebar (glossary), so no run can
      put an object a tip could be chosen for under a pointer that rests on the sidebar, and a rest
      there gives no name tip with or without the rule, while a movement from the sidebar back onto
      the view starts the wait anyway.
      `measurable` (S-0411 scenario S16 parts a and b) for the exception over the build-icon slots,
      as S-0411 claim 20's Backing gives it, and `unmeasured` for its one drawn frame, as there. The
      blocks over the sidebar are backed as S-0411 claim 20's Backing gives them (`unmeasured` for
      the order block and for the options menu over the sidebar).
      `unmeasured` for a text chosen again not starting the lifetime again: no set-up here changes a
      showing tip's text at a chosen time (claim 26). `unmeasured` for a block that ends while the
      pointer still rests: no NT set-up holds the pointer still through a block that ends without a
      scroll, such as a block of the map's trigger actions or a chrono flash (S-0201 claim 11), and
      a scroll moves the battlefield under the pointer, so no set-up here keeps a known object under
      it. `unmeasured` for a held left button whose press is an ordinary click inside a point's
      click area in the route mode: that needs a point that is not highlighted though the
      point-click test passes it, such as another house's, which only that house's own route-mode
      clicks make, and no NT set-up has a second player giving orders.
    - Confidence: high for the 1000 ms wait, the 10000 ms lifetime, the restart on any movement over
      the battlefield view, the three buttons, the option and game speed not changing the wait or
      the lifetime. Medium, `measurable` (NT1, NT2 and NT8), for the wheel, the moving view, the one
      choice per rest, the options menu's opening and the held buttons, in the route mode too while
      the button is held. The route-mode cases of the `unmeasured` list below are not part of that
      medium. Low, `unmeasured` (Backing), for the end of the block after an ordinary left click in
      the route mode released over a highlighted or refused point (Open questions), which that
      medium does not cover. Medium, `unmeasured` (Backing), for the hidden cursor, the locked
      input, a block that ends while the pointer rests, a rest over a message line, the pointer
      outside the window, a rest over the sidebar giving no name tip, the window's edge not being a
      movement, a movement over the sidebar outside its build-icon slots starting the wait again,
      the lifetime not starting again, the menu's closing bringing no tip back, a held left button
      whose press is an ordinary click inside a point's click area in the route mode, a held left
      button that drags a band box in the route mode, a press in the route mode with no object
      selected that takes an own point and the end of the block at the button's release in the route
      mode, apart from the low case above.
      Medium, `measurable` (S-0411 scenario S16 parts a and b), for the exception of a movement over
      the build-icon slots. Medium, `unmeasured`, for its one drawn frame. For the blocks over the
      sidebar, S-0411 claim 20's levels: medium, `unmeasured` for the order block and the options
      menu there.

21. **Name tip: which pointer objects get one.** When the wait ends, the tip's text is:
    - the shroud label (Import mapping) when the pointer cell is not explored by the local
      player's house (S-0409), whatever the cell holds
    - otherwise nothing when there is no pointer object. A resource overlay, a wall or other
      overlay, or bare ground under the pointer gives no tip
    - the name of claim 22 for a vehicle, infantry unit, aircraft or building that the local
      player owns. This includes an own cloaked object
    - the name of claim 22 for a vehicle, infantry unit, aircraft or building of any other house
      (allied, enemy, civilian or neutral), with two exceptions:
      - nothing when it is fully cloaked (S-0516 Terms) and not sensed by the local player's
        house (S-0516 Terms), also when its owner is allied. A cloaking or uncloaking object
        gets its name
      - nothing when its type has the invisible flag (Import mapping)
    - for any other pointer object, such as a terrain object: the display name of the resource
      kind of the pointer cell's resource overlay (the kind S-0401 gives that overlay), and
      nothing when the pointer cell has no resource overlay.

    An occupied building shows its own name, never an occupant's. An object's health, owner and
    veterancy never appear in the tip.
    - Backing: `measurable` (scenario NT3). `unmeasured` for a cloaking or uncloaking object getting
      its name: the real-time wait would have to end inside the cloak's transition, which no NT3
      part can time
    - Confidence: high for own and other houses' vehicles, infantry, aircraft and buildings, for the
      fully cloaked rule on a fully cloaked unit, and for the invisible flag on an enemy vehicle and
      an enemy building, a building type's invisible-in-game key included; medium for the invisible
      flag on infantry and aircraft of other houses and for own objects of a type with the flag
      getting their name, which NT3 does not run, for a cloaking or uncloaking object getting its
      name, which no NT3 part times, for the shroud label and for the other pointer objects, since
      which of them can be the pointer object is S-0214's matter

22. **Name tip: the name by kind.** The name of claim 21 is:
    - for a building or an aircraft: its type's display name. A disguise does not change it
    - for an infantry unit:
      - the technician label (Import mapping) when the unit carries the technician mark (below)
      - otherwise the display name of its disguise, when it is disguised and its owner does not
        ally the local player's house
      - otherwise its type's display name.

      This disguise test is not S-0516 claim 35's: the local player's detector coverage and the
      disguise's house do not enter it. So enemy infantry inside the local player's detector
      coverage still shows its disguise's name, and an own or allied one shows its own.
    - for a vehicle that appears disguised to the local player's house (S-0516 claim 35) and has
      a disguise: nothing when the disguise has no house, as for the hover health bar (claim 3),
      and the display name of its disguise otherwise. A disguise's display name is that of the
      type it copies, which can be a terrain object's
    - for any other turret-tip vehicle: the text of claim 23
    - for any other vehicle: its type's display name.

    **Technician mark.** An infantry unit can carry a mark that makes its name the technician
    label. Only a unit whose type has the nominal flag (Import mapping) gets it, and only when a
    building releases the unit, for example as a survivor. Which releases set it is not settled
    (Open questions). A unit without the mark never shows the technician label.
    - Backing: `measurable` (scenario NT4 for the disguise rules, a disguised building and aircraft
      included; NT3 for the type names; NT9 for the technician mark). `unmeasured` for the mark's
      effect on the name if NT9 gives no survivor the label: no other release with the mark set is
      known
    - Confidence: high for the type names of buildings, aircraft and vehicles; medium for a disguise
      not changing the name of a building or an aircraft, for both disguise rules and for a vehicle
      disguise without a house; low for when the technician mark is set, medium for its effect on
      the name

23. **Name tip of a turret-tip vehicle.** The text is the modifier, one space, and the vehicle's own
    type's display name, never its disguise's. The modifier depends on the vehicle's selected turret
    index *t* (S-0105 claim 43) and on *P*, the passenger at position 1 of its passenger order
    (S-0507 claim 3): the one that boarded most recently among those aboard. *P* need not be the
    passenger whose entry set the weapon number (S-0308 claim 6), and it changes whenever the
    passenger at position 1 changes. S-0105 claim 43 gives *t* for a multi-turret type. A turret-tip
    vehicle whose type has rotating-barrel weapon stages (S-0105 Import mapping) gets *t* by the
    same rules for a gunner type: it is set from the turret table entry for the weapon number when
    the vehicle is created and at every gunner hand-over (S-0308 claim 6), and is *none* for a type
    that is not a gunner type. A change of weapon stage never changes *t*, and such a
    rotating-barrel vehicle whose type also has the charge-turret flag keeps *t* at *none* for its
    whole life.
    - With no passenger, or a *P* that is not infantry, the modifier is label A.
    - Otherwise, with *u* standing for *P*'s type having the own-name flag (Import mapping):

      | *t* | Modifier |
      |---|---|
      | 0 | label A |
      | 1 | *P*'s type's display name if *u*, label B otherwise |
      | 2 | label C |
      | 3 | *P*'s type's display name |
      | any other value, *none* included | *P*'s type's display name if *u*, label A otherwise |

    Labels A, B and C are three fixed string-table entries (Import mapping), resolved as S-0004
    claim 17 gives. The modifier follows the index, not the weapon number: a vehicle whose index
    differs from its weapon number shows the index's row. An empty modifier still leaves the
    space, and so does an empty type name. The text then starts or ends with a space and still
    shows.
    - Backing: `measurable` (scenario NT5, every row and flag variant, (f) for a rotating-barrel
      type, gunner or not, with or without the charge-turret flag, (g) for *P* with two passengers
      aboard, (h) for a disguised subject)
    - Confidence: high for the table, for label A with no passenger and for the format; medium for
      the type name never being the disguise's in the format, which only NT5 (h) runs, *P* being the
      passenger at position 1 when more than one is aboard, label A with a *P* that is not infantry,
      which NT5 (e) reaches only if its variant lets such a unit board, for *t* of a rotating-barrel
      type, and for the empty-name cases

24. **Name tip: size and placement.** The tip's text is laid out in the game font (S-0014). A line's
    width in pixels is the sum, over its text units (S-0014 Terms), of each text unit's glyph width
    (S-0014 claims 3 and 4) plus 1, except that a tab moves the line's width up to the next multiple
    of 64. A line break (a carriage return or a line feed, a carriage return followed by a line feed
    counting as one) starts a new line. Lines are also wrapped to the wrap width *W* − 8, where *W*
    is the battlefield view's width in pixels:
    - **Wrapping.** When a text unit would make its line wider than the wrap width and it is not the
      first unit on its line, the line is broken. A space is a break point only when at least one
      unit comes before it on its line. If the line has a break point before that unit, it breaks at
      the last one: the line ends just before that space, the space is dropped (neither drawn nor
      counted in the line's width), and the next line starts with the unit after it. Otherwise the
      line ends just before the unit, which starts the next line. A unit that is first on its line
      stays on it even when it alone is wider than the wrap width, so every line holds at least one
      unit. So a line whose only space is its first unit, as when a text starts with a space (claim
      23), keeps that space and breaks just before the unit that passes the wrap width. The box then
      has no empty line, and its text starts at the usual origin (claim 25), the space taking its
      width at the start of the first line. Only the space at the break is dropped, and only spaces
      of the current line count: a line break forgets every earlier space.
    - **Size.** *w*<sub>t</sub> is the width of the widest line and *h*<sub>t</sub> is the number of
      lines times the font's line pitch *P*<sub>f</sub> (S-0014 claim 1). The box is *w* =
      *w*<sub>t</sub> + 8 pixels wide and *h* = *h*<sub>t</sub> + 4 pixels high.

    Let (*x*, *y*) be the pointer's screen position when the tip shows, and let the battlefield view
    span *X*<sub>0</sub> to *X*<sub>1</sub> horizontally and *Y*<sub>0</sub> to *Y*<sub>1</sub>
    vertically (its right and bottom edges exclusive). The box's top-left corner is (*l*, *m*):
    - *l* = *x*, or *X*<sub>1</sub> − *w* when *x* + *w* > *X*<sub>1</sub>, so the box keeps its
      right edge inside the view
    - *m* = *y* + 16, or *y* − *h* + 1 when *y* + 16 + *h* > *Y*<sub>1</sub>, so a tip near the
      bottom edge flips above the pointer, its last row on the pointer's row *y*.

    There is no correction at the left or top edge: the view cuts a box that reaches past them. With
    the wrap width above, only a line holding a single unit wider than the wrap width can make a box
    wider than the view. The box is drawn with the size it is placed with (Edge cases give the
    original's display bugs here). While the tip shows, it keeps the pointer position it was placed
    for, and a text chosen again (claim 26) is laid out and placed again by the same rules.
    - Backing: `measurable` (scenario NT6, which measures the drawn box against the probe texts'
      layout, the absence of a top correction through its many-line probe, the break at the last
      space with its dropped space through its three-word probe, and the original's placing, which
      gives claim 24's box at the right edge and at every pointer row that NT6 runs, Edge cases).
      `unmeasured` for the absence of a left correction, the break of a line with no space and a
      unit wider than the wrap width: in the original only a box that reaches past the view's left
      edge shows them, and there its lines are cut (Edge cases). `unmeasured` for a text laid out
      and placed again while the tip shows: no set-up here changes a showing tip's text at a chosen
      time (claim 26). The wrap width *W* − 8, a line break forgetting earlier spaces, no break at a
      space that is first on its line and our engine's placing with the drawn size at the one
      pointer row where the original's flip differs (Edge cases) are presentation design rules, not
      oracle predictions.
    - Confidence: high for the distance of 16 pixels below the pointer, the left edge at the
      pointer's column when the box is not shifted, and the four design rules above; medium for the
      box's margins of 8 and 4 pixels, the text layout (*w*<sub>t</sub> and *h*<sub>t</sub>), the
      flip and the right-edge shift, which rest on the box's size, the line breaks, the wrapping
      rules, the absence of a left or top correction, which NT6 reaches only at the top, and the
      layout and placing again of a text chosen again

25. **Name tip: look and layer.** The box is filled with opaque black and framed by a line one pixel
    wide along its four edges. The frame and the text have one colour, the tip colour: RGB (164,
    210, 255) when the local player's side index (S-0005 claim 13a) is 0, and RGB (255, 255, 0) for
    any other side index. It takes S-0103 claim 9's output step like any other drawn colour. The tip
    is drawn on top of everything else on the battlefield view, status extras included, and never
    over the sidebar (claim 24 keeps it inside the view). A building's name tip is not S-0102's
    information tip of a building, which is drawn right after the ground layer (S-0102 claim 2): the
    name tip is drawn after the whole battlefield view, the information tips included. The text's
    origin, from which S-0014 claims 3 and 4 draw its first line, is 4 pixels right of the box's
    left edge and 2 pixels below its top edge, so 3 columns of black lie between the frame and the
    text's origin on the left and 1 row above it. Each further line starts *P*<sub>f</sub> lower at
    the same column. The tip colour and the black fill are drawn as they are: the cell's light, a
    lighting tint and shroud (S-0104, S-0409) change neither, since the tip is drawn over the
    battlefield with no shade table.
    - Backing: `measurable` (scenario NT7)
    - Confidence: high for the black fill, the frame, the one colour of frame and text, and the two
      RGB values; medium for the side mapping, the layer, the colours under a lighting tint and over
      shroud, and the text's origin inside the frame

26. **Name tip: when it goes or changes.** A tip hides when the pointer moves, when a left, right or
    middle mouse button is pressed or released, when its lifetime ends (claim 20), when the tooltip
    option is turned off and when the in-game options menu opens (claim 20). While it shows, its
    text is chosen again in every drawn frame, by claims 21 to 23 for the place where the pointer
    rests and with claim 20's blocking conditions:
    - When that gives no text or an empty text, or the tip is blocked, the tip hides, and nothing
      shows until the pointer moves and rests again (Edge cases for the original). So it hides when
      its object dies or leaves the pointer's place and nothing with a text is left there, when an
      object of another house fully cloaks there unsensed (claim 21), and when the view scrolls
      under the resting pointer.
    - When it gives another text, that text replaces the shown one, and the box is laid out and
      placed again for it (claim 24). So another object left under the pointer when the first one
      dies or leaves gives its own name, and a change of disguise gives the new disguise's name
      (claim 22).
    - Otherwise the tip stays as it is. A change of the selection, a change of owner that leaves
      claim 22's name the same and an own object cloaking change nothing.

    Nothing outside claim 20's blocking conditions stops the tip from showing: a building placement,
    a superweapon aim and any order cursor all leave it able to show.
    - Backing: `measurable` (scenario NT1 (c) for the pointer and (g) for a press or release while
      the tip shows; NT2 for the options menu; NT8: part a for death and departure, b for a
      selection change, f for an own cloak, g for a scroll while the tip shows, d for the placement,
      the aim and an order cursor). The choice in every drawn frame, as against a slower cadence, is
      `unmeasured`: NT8 bounds only how soon the tip hides after its cause, and no part records the
      frame in which it hides. `unmeasured` for another text replacing the shown one, for a change
      of owner or disguise and for another house's object cloaking while the tip shows: none can be
      timed into the tip's lifetime without a press, which hides the tip, or another house's action.
      `unmeasured` for the option turned off while the tip shows: turning it off takes a press in
      the options menu, which itself hides the tip.
    - Confidence: high for the pointer, the buttons and the lifetime; medium for the choice in every
      drawn frame and what it hides, replaces or leaves, for the option turned off while the tip
      shows, the options menu, and for the placement, the aim and the order cursors

## Formulas

**F1. Filled health segments.** *H* and *S* are the integer health and full strength, *S* > 0 (claim
5 gives *f* for *S* ≤ 0). Write ⟨*e*⟩ for the binary64 result of the single operation *e*, rounded
toward zero (the round-toward-zero mode, S-0205):

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

**F6. Control-group number** (integers, pixels, *x* to the right and *y* downwards; claim 17).
*d* is the placement point, *G* the digit glyph's imported width, *H*<sub>f</sub> the font's
bitmap height and *P*<sub>f</sub> its line pitch:

> glyph mask pixel (*i*, *r*), 0 ≤ *i* < *G*, 0 ≤ *r* < *H*<sub>f</sub>, at
> (*d*<sub>x</sub> + *i*, *d*<sub>y</sub> + *r*)
>
> box: *d*<sub>x</sub> − 3 ≤ *x* ≤ *d*<sub>x</sub> + *G* + 2 and
> *d*<sub>y</sub> + 2 ≤ *y* ≤ *d*<sub>y</sub> + *P*<sub>f</sub> − 3
>
> border: the box pixels with *x* = *d*<sub>x</sub> − 3, *x* = *d*<sub>x</sub> + *G* + 2,
> *y* = *d*<sub>y</sub> + 2 or *y* = *d*<sub>y</sub> + *P*<sub>f</sub> − 3

The other box pixels are black. A set mask pixel shows the house colour whether it falls on the
border, the black interior or the scene. The rows are not limited by *P*<sub>f</sub>: when
*P*<sub>f</sub> < *H*<sub>f</sub>, the rows *r* ≥ *P*<sub>f</sub> still paint. When
*P*<sub>f</sub> − 4 ≤ 0 the box is empty in our engine and only the glyph is drawn. Nothing is
rounded.

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
| Group-number box corner | (−3, +2) from the placement point | pixels | claim 17, F6 |
| Group-number box size | (*G* + 6) × (*P*<sub>f</sub> − 4) | pixels | claim 17, F6 |
| Group-number box fill | black, opaque | colour | claim 17 |
| Name-tip wait | 1000 | ms of real time | claim 20 |
| Name-tip lifetime | 10000 | ms of real time | claim 20 |
| Name-tip distance below the pointer | 16 | pixels | claim 24 |
| Name-tip margins | 8 (width), 4 (height) | pixels added to the laid-out text | claim 24 |
| Name-tip wrap allowance | 8 | pixels taken from the view's width for the wrap width | claim 24 |
| Name-tip text origin | 4 (right), 2 (down) | pixels from the box's top-left corner | claim 25 |
| Name-tip colour, side index 0 | (164, 210, 255) | RGB | claim 25 |
| Name-tip colour, other side indices | (255, 255, 0) | RGB | claim 25 |
| Name-tip text limit | 256 | text units (S-0014) | Edge cases |

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
| hovered | if claim 3 allows, no bracket | if claims 3 and 9 allow | if claim 18 allows |
| neither | no | no | if claim 18 allows |

Within one object, where they overlap, the selection marks and the health bar cover the insignia,
and the pip scale covers the health bar. The control-group number and its box cover all of them.
Across objects, each object's status extras are complete before the next object's begin, in
S-0102's object order (claim 17).

The overload pip (claim 13) and the self-heal blink (claim 16) change with ticks: every drawn frame
between two ticks shows the same state.

**Name tip (claims 20 to 26).** The tip has four states: off, idle, waiting and showing. All
timing is real time.

| State | Event | Next state | Effect |
|---|---|---|---|
| any | tooltip option turned off, or in-game options menu opened | off | tip hidden, none shows while off |
| off, in-game options menu not open | tooltip option turned on | idle | none |
| off | in-game options menu closed with the option on | idle | none |
| idle, waiting or showing | pointer moves | waiting | tip hidden; the 1000 ms wait starts again |
| waiting or showing | left, right or middle button pressed or released | idle | tip hidden; no wait runs |
| waiting | wait ends over the battlefield view, claim 20 does not block, text not empty | showing | text chosen and box placed (claims 21 to 24); 10000 ms lifetime starts |
| waiting | wait ends otherwise | idle | nothing shows |
| showing | drawn frame, claim 20 does not block, text not empty | showing | text chosen and box placed again (claims 21 to 24, 26); lifetime unchanged |
| showing | drawn frame otherwise | idle | tip hidden |
| showing | lifetime ends | idle | tip hidden |

The state table restates claims 20 and 26. Over the sidebar's build-icon slots, S-0411 claim 20's
sidebar tip applies instead of this table (claim 20's exception). While the in-game options menu is
open the state stays off, whatever the option: turning the option on in that menu takes effect when
the menu closes (claim 20). Backing: `measurable` (scenarios NT1, NT2 and NT8), and `unmeasured`
where claims 20 and 26 give a part as `unmeasured`, with their reasons. Confidence: high for the
pointer, button and lifetime rows, turning the option on, and the 1000 ms wait of the two wait-end
rows; medium for the blocking conditions, the place of the rest and the one choice per rest that
those two rows restate from claim 20, for the text and box that the showing rows choose by claims 21
to 24, for the two drawn-frame rows (claim 26), for the options menu, and for turning the option off
while a tip shows.

The overlap order between insignia, health bar and pip scale is `unmeasured`, confidence medium:
no scenario below isolates their overlapping nontransparent pixels. The visibility table restates
claims 3, 9 and 18. OV8 and OV11 test tick transitions; invariance over multiple drawn frames
within one tick is `unmeasured`, confidence high, since they supply no frozen-cycle comparison.

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
  to reproduce. Backing: `unmeasured`, confidence high: no scenario here varies the flag while
  holding the scene constant.
- **Passenger row overflow** (departure, no switch). The original hides the passenger row,
  self-heal marker and group number when the passengers' total size exceeds the row length, a
  display-only bug whose departure is confirmed by the project lead. Claims 11 and 12 require
  clipped passenger pips and ordinary marker visibility instead, without changing passenger
  capacity or loading legality. The importer warns about a type with a passenger capacity whose
  pip-scale kind is not passengers.
- **Negative pip wrap** (claim 15, Q-2071): no ammunition pips, with an importer warning
  (medium until OV10 b runs). No switch: the result is deterministic and harmless, and it only
  follows from bad data.
- **Unknown pip-scale word** (Import mapping, Q-2071): the kind none, with an importer warning
  (medium until OV10 c runs). No switch.
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
- **Missing cursor palette** (departure, no switch; claim 19a, Q-1622). The original fails
  when it reads the cursor palette and finds none (`unmeasured`, medium). Our engine draws neither the wrench nor the
  bomb icon and the importer warns once, naming the file. A missing overlay sprite likewise
  draws nothing, with an importer warning (Import mapping).
- **Round-toward-zero mode** (no switch here). F1 and F2 depend on it; the switch
  `quirks.chop_rounding` belongs to S-0205 (number domains), and F1 and F2 are among the
  results it covers.
- **`quirks.speed_normalised_delays`** (default: on, shared with S-0107, S-0108
  and S-0109 (animation playback)). Claim 19.
- **Name-tip text of 256 text units or more** (departure, no switch; claims 20 and 24). The limit
  counts text units (S-0014 Terms), not characters. In the original, what such a text shows is
  undefined: it can show text units that are not in the name. Our engine shows the first 256 text
  units. A text of up to 255 text units shows whole in both. Backing: `unmeasured`, since the
  original's result is undefined. Confidence: medium.
- **Name-tip box at the bottom edge** (display-only, no switch; claim 24). In the original, at the
  one pointer row *h* + 15 pixels above the view's bottom edge, where *h* is the box's drawn height,
  the box does not flip above the pointer, and its last row falls outside the view and is cut. For a
  box no wider than the view whose text has not been chosen again with a smaller size (the next
  bullet and the last bullet give those cases), the original's box stands where claim 24 places it
  at every other pointer position, at the right edge included. Our engine flips the box at that row
  as claim 24 gives.
- **Name-tip box wider than the view** (display-only, no switch; claim 24). In the original a line
  wraps only when it would be wider than *W*, not *W* − 8, so a text with a line wider than *W* − 8
  gives a box wider than the view. That box stands with its right edge on the view's last column and
  is cut at the view's left edge, and near the bottom edge its flip can come at other pointer rows
  than claim 24 gives. Our engine wraps at *W* − 8.
- **Name-tip wrap after a line break** (display-only, no switch; claim 24). In the original a wrap
  can go back to the last space before an earlier line break, which draws part of the text twice and
  makes the box taller. Our engine forgets every space at a line break.
- **Name-tip wrap at a leading space** (display-only, no switch; claim 24). In the original a line
  whose only space is its first unit breaks at that space when a later unit passes the wrap width,
  which leaves an empty first line, so the box is one line pitch taller and the text starts one line
  lower. Our engine does not break at a space that is first on its line.
- **Name-tip box after a smaller text** (display-only, no switch; claims 24 and 26). In the
  original, when a text chosen again while the tip shows is narrower or has fewer lines than an
  earlier text of the same showing, the box can stand further left or higher than claim 24 places
  it, as the earlier, larger box would, until the tip hides. Our engine places each text by its own
  size.
- Backing for the five bullets above: `measurable` (scenario NT6) for the drawn size and the
  original's placing at the right edge and at every pointer row NT6 runs. `unmeasured` for the
  original's results in each bullet, for a reason of set-up: no NT6 run reaches the one row of the
  first bullet (its runs near the bottom edge keep the pointer within 20 pixels of that edge, while
  that row lies more than 20 pixels above it for every box, its many-line run flips under both
  rules, and its other runs at the view's centre end far above the bottom edge); no NT probe text
  has a line wider than *W* − 8 for the second; no NT probe text has a line break followed by a line
  that passes the wrap width for the third; no NT run wraps a text that starts with a space (an
  empty modifier or type name, claim 23; NT5 gives such texts but does not wrap them) for the
  fourth; and no set-up here changes a showing tip's text at a chosen time (claim 26) for the fifth.
  Confidence: medium.
- **A name tip back under a resting pointer** (display-only, no switch; claims 20 and 26). In the
  original, when a showing name tip has hidden because its text became empty or none, or because
  the cursor was hidden, the input locked or the view moving (claim 26), and the pointer still
  rests there when the hidden tip's lifetime would have ended, the text is chosen again at that
  moment and the tip shows if it then gives one and no block holds, as for S-0411's icon tips
  (S-0411 Edge cases). A press or release of a mouse button ends that lifetime, so a tip it hid,
  or one hidden while a button is held, does not come back this way. Our engine shows nothing
  until the pointer moves and rests again. Backing: `unmeasured` for the name tip, for a reason
  of set-up: no NT run brings an object with a text under a resting pointer after its tip has
  hidden, and no NT run ends a block with the pointer still on a known object (a scroll moves the
  view under it); S-0411 scenario S16 part i arranges the emptied case for an icon tip.
  Confidence: medium.
- **The movement that leaves the build-icon slots** (display-only, no switch; claim 20). In the
  original the first movement off the slots is handled without a wait, so a name tip of an object
  on the battlefield view where it ends shows at once; S-0411 Edge cases give it. Our engine
  starts the wait, as for any movement. Backing: `measurable` (S-0411 scenario S16 part i) for a
  fast movement from an icon onto a vehicle; `unmeasured` for a movement that ends elsewhere,
  since part i reads no other. Confidence: medium.
- **Two disguise tests** (no switch; claim 22). The tip uses S-0516 claim 35's test for vehicles
  only. Infantry uses the simpler test of claim 22, as the original does. An enemy spy inside the
  local player's detector coverage therefore still shows its disguise's name.

## Import mapping

| Section and key | Form | Feeds | Absent |
|---|---|---|---|
| rules `[AudioVisual]`, `ConditionYellow` | decimal or percentage ratio (S-0003) | yellow ratio (claim 5) | 0.5 (as S-0108) |
| rules `[AudioVisual]`, `ConditionRed` | decimal or percentage ratio | red ratio (claim 5) | 0.5 (as S-0108) |
| rules `[AudioVisual]`, `EnemyHealth` | boolean | nothing (Edge cases) | — |
| rules `[General]`, `SelfHealInfantryFrames` | integer, ticks | *T* for infantry healing (claim 16) | 1000 |
| rules `[General]`, `SelfHealUnitFrames` | integer, ticks | *T* for unit repair (claim 16) | 1000 |
| type section, `PixelSelectionBracketDelta` | integer, pixels | δ (claim 6) | 0 |
| type section, `PipScale` | named value: `none`, `ammo`, `tiberium`, `passengers`, `power`, `mindcontrol` (S-0003 claim 16, vocabulary below) | pip-scale kind (claim 11) | none |
| type section, `PipsDrawForAll` | boolean | claim 9 (buildings only) | no |
| type section, `PipWrap` | integer, negative values kept (claim 15) | *w* (claim 15) | 0 |
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
| game font file `GAME.FNT`, resolved as in S-0213's Import mapping | binary layout of S-0014 claims 1 to 3 | digit glyph masks, widths *G*, bitmap height *H*<sub>f</sub> and line pitch *P*<sub>f</sub> (claim 17, F6) | a missing or rejected font (S-0014 claim 6): no group number in our engine, with S-0213 claim 25's diagnostic; no original result is predicted |
| settings file `[Options]`, `ToolTips` | boolean (S-0003) | tooltip option (claims 20 and 26) | yes |
| type section, `Invisible` | boolean | invisible flag (claim 21) | no; a building type whose `InvisibleInGame` is yes also gets the flag |
| vehicle section, `HasTurretTooltips` | boolean | turret-tip flag (Terms, claim 23) | no |
| infantry section, `UseOwnName` | boolean | own-name flag (claim 23) | no |
| infantry section, `Nominal` | boolean | nominal flag, needed for the technician mark (claim 22) | no |
| type section, `UIName` | display-name label (S-0004 claim 17) | display name (claims 21 to 23) | S-0004 claim 17 |
| string table, `TXT_SHADOW` | fixed entry identifier, not literal game text | shroud label (claim 21) | S-0004 claim 17: a missing entry shows its placeholder |
| string table, `TXT_TECHNICIAN` | fixed entry identifier, not literal game text | technician label (claim 22) | as above |
| string table, `Tip:Rocket` | fixed entry identifier, not literal game text | label A (claim 23) | as above |
| string table, `Tip:MachineGun` | fixed entry identifier, not literal game text | label B (claim 23) | as above |
| string table, `Tip:Repair` | fixed entry identifier, not literal game text | label C (claim 23) | as above |

Vocabulary of `PipScale`, compared ignoring case (Q-2071): `ammo`, `tiberium`, `passengers`,
`power` and `mindcontrol` select their kinds. Any other word, `none` included, gives the kind
none: no row of its own and row length 0 in claim 12, the same as an absent key with no earlier
value. Its fixed fallback is none, not the held value (S-0003 claim 16). Our engine warns about a
word other than `none` that matches none of them. Backing: `measurable` (scenario OV10 c) for the
fallback, `unmeasured` for the warning. Confidence: high for the five listed words and `none`,
medium for the fallback of any other word until OV10 c runs (S-0003 claim 16 rates its per-field
fallbacks medium).

Vocabulary of `Pip` and `OccupyPip`, compared ignoring case (S-0003 claim 16), with the image frame
each word selects: `green` 1, `yellow` 2, `white` 3, `red` 4, `blue` 5, `persongreen` 7,
`personyellow` 8, `personwhite` 9, `personred` 10, `personblue` 11, `personpurple` 12.
A word that matches none gives 1, with an importer warning (S-0003 claim 16). An absent read
retains the held value, initially 1 (`Pip`) and 7 (`OccupyPip`), as claim 12 requires.

Overlay sprites, resolved by S-0005: `PIPS.SHP` (pip sprite), `PIPS2.SHP` (second pip sprite),
`PIPBRD.SHP` (bracket sprite), `WRENCH.SHP` (wrench sprite). Pips and brackets use the overlay palette
`PALETTE.PAL`; the wrench and the bomb sprite use the cursor palette `MOUSEPAL.PAL`, the same
palette as the cursor images of S-0214 (Q-1622).

| Sprite file | Use | Palette file |
|---|---|---|
| `BOMBCURS.SHP` | bomb icon (S-0519 claim 24) | `MOUSEPAL.PAL` |

**Cursor palette (Q-1622).** Claim 19a gives the cursor palette's rule: the file, its
conversion, its lifetime and the missing-file case.

A missing sprite draws nothing; the
importer warns. A missing cursor palette has its own rule (claim 19a). This missing-asset rule is `unmeasured`, confidence medium: no scenario here
removes the overlay assets and checks both the absent drawing and the warning. Other importer
warnings in this spec are likewise `unmeasured`, confidence medium, unless explicitly checked by
a departure-conformance scenario: the original cannot confirm our importer's diagnostics.

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*, a tiny flat purpose-made map, one human player and
one bystander computer player (the glossary's void rule applies in the observed area around the
test objects), neutral lighting, game-speed index 2 and 800 × 600 pixels unless stated otherwise,
no mods except the named test data, screenshots compared pixel by pixel
with the overlay sprites and palettes from the stock archives. No external program may alter game
state or generate orders beyond the scenario's prescribed mouse and keyboard input. Read-only
recorders and the glossary's scripted-input allowance remain permitted; the original's computer
player remains active and subject to the bystander void rule. Health is set by attacking with a stock
weapon and read with a recorder that reads the running game's state; the recorder also gives each
object's centre, from which the status anchor follows by S-0101. Where a scenario needs data that
stock types do not have, it uses a test map whose override sections change only the named fields,
documented with the scenario. Unless a scenario says otherwise, one run is enough: no random draw is
involved.

- **OV1 (claim 1).** Use a named test bracket sprite with a centred canvas extending at least
  400 pixels horizontally and 220 pixels vertically from its centre, with visible test marks
  throughout. This makes the bracket still intersect the view when its anchor crosses the culling
  margin. Scroll a selected vehicle out of the view, first in steps of 10 pixels and then in
  single-pixel steps at each boundary; read its anchor and view origin with the recorder.
  Expected: the intersecting test marks remain drawn while the anchor is within 360 pixels left or
  right (180 above or below) of the view, including equality, and vanish beyond. Falsified if
  disappearance occurs at another anchor position. Repeat at each of the four view edges. Ordinary
  stock brackets alone cannot expose this offscreen culling margin.
- **OV2 (claim 2).** A selected veteran transport of the player, with one passenger, at the edge of
  the shroud; a damaged building of the player being repaired at that edge; a unit of the player
  carrying an attached bomb, attached by a stock bomb-planting weapon, at that edge. Position the
  objects so visible overlay pixels cross the shroud rim while the wrench and bomb pass claim 19's
  point test; log that test's explored cells. Compare the same scene in named test maps with neutral
  and night lighting. Expected: the bracket, insignia, wrench and bomb pixels darken under shroud
  as S-0110 F1 gives and otherwise use neutral brightness regardless of map lighting; health
  segments and passenger pips keep their palette colours. Falsified if the segments or pips darken,
  the other sprites ignore shroud at sampled covered pixels, or any overlay follows night lighting.
  If no covered, nontransparent pixel of an eligible sprite can be exposed, that sprite's shroud
  result remains unmeasured, not passed. No fog-on run is planned: S-0409 claim 19 forces fog off.
- **OV3 (claims 3, 6).** A stock vehicle, infantry unit and aircraft of the player and of the computer
  player, each at several healths. Select each owned object, then deselect it and hover it.
  Hover each enemy object. Expected: the segment rows of claim 6 at the
  displacements given (δ read from the type's field), the bracket only when selected, enemy bars on
  hover. Falsified if any displacement or image frame differs. Also hover an enemy spy disguised
  as infantry of the player's house and an enemy disguise-when-still vehicle disguised as terrain,
  with neither inside the player's disguise-detector coverage. Expected: a bar on the spy, whose
  disguise has a house, and none on the vehicle, whose disguise has none. Repeat after detector
  coverage reaches the vehicle: expected its hover bar. Falsified by the opposite visibility.
  Use game-speed index 2 and the common setup's restrictions on mods and external orders, screen
  capture and a read-only recorder of disguise state and house; repeat each case three times.
- **OV4 (claims 4, 5).** A test map gives one stock vehicle type a strength of 160 and sets the
  yellow and red ratios to 0.5 and 0.25 (exact in binary64), and gives a second stock vehicle type a
  strength of 170. The first vehicle is taken through healths 81, 80, 41 and 40. Expected: green,
  yellow, yellow, red (each colour switches at equality). A building of the player given the test
  strength 160 is taken through the same healths. Expected: the same colours. The second vehicle is
  taken through healths 170 down to 1. Expected: *f* as F1 gives at every health, in particular 2
  filled segments at health 30 (*h* = 3/17), not 3, and 1 segment at health 1. Falsified if a colour
  changes one point late or early, or *f* differs (3 segments at health 30 would mean
  round-to-nearest). Repeat the sweep with a test infantry type of strength 160: expected the
  eight-segment F1 result. Repeat the colour checks with both condition ratios omitted from a
  named loose test base-rules copy, ensuring no later layer supplies them: expected no yellow.
  Then set the red ratio to 0.75 and yellow to 0.25: expected red through equality at 0.75 and
  green above it, never yellow. Any yellow in these last two runs falsifies claim 5.
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
  shows pips, the refinery shows pips, a building row is placed elsewhere, or
  the test spawner lacks either row. Then a test map whose override sections give one stock enemy
  building type that a spy can infiltrate a passenger capacity of 3 and the passengers pip-scale kind
  (so that its pip scale is a row of three empty pips, image frame 0); a spy of the player infiltrates
  one such building and a second one is left alone. Neither type is occupiable or marked to draw
  its pips for all. Hover both before and after entry. Expected: neither building shows a pip scale
  before or after infiltration (S-0510 claim 8). The same test map also gives
  a second enemy building type the same capacity and kind and marks it, and one stock enemy
  transport type, to draw their pips
  for all. Expected: the three empty pips on that building without any spy, and no pips on the empty
  enemy transport (its row of empty pips stays hidden). Last, an own unit of a stock type with the
  ammunition pip-scale kind and no passenger capacity, fully loaded. Expected: its ammunition pips 3
  pixels above the row places of claim 10, from the second pip sprite. Falsified if either the infiltrated
  building or the untouched one shows a pip scale, the draw-for-all mark shows pips on the
  transport or not on the building, or the ammunition pips sit at another height. Repeat the
  ordinary passenger and ammunition checks with a mutually allied bystander house: expected its
  hovered transport's pips as for an owned transport. Hidden allied pips falsify claim 9.
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
  case with an enemy transport: its passenger pips remain hidden under claim 9, regardless
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
  - **OV10 b (claim 15, Q-2071 question 2).** A test map override section gives a test copy of an
    ammunition unit type the ammunition kind and a pip wrap of −2, with no passenger capacity. An
    own unit of it in control group 1, selected, recorded at full ammunition, after one shot and
    when empty. Expected: no ammunition pip at any count, with the group number still drawn.
    Falsified if any ammunition pip is drawn, or the pips of a wrap of 0 or of 2 appear.
  - **OV10 c (claims 11, 12, Import mapping, Q-2071 question 1).** Test rules data gives a test
    copy of a stock transport type without the gunner flag the pip-scale word `passengers`, its
    passenger capacity unchanged, and a test copy of an ammunition unit type the word `ammo` and
    no passenger capacity. A test map override section then gives both copies the word
    `sideways`; a control game uses the same rules data without that override. In each game, own
    units of both copies are selected, as in OV10 b, and recorded with the transport empty, then
    loaded with one infantry passenger. Expected: in the control, the transport's passenger row
    and the second unit's ammunition row; in the test game, no pip row on either unit, loaded or
    not. Falsified if the test game shows the transport's passenger row of any length or the
    second unit's ammunition row (the held kind kept), or if the control shows neither row (the
    run cannot tell the fallback from the held kind).
- **OV15 (claim 11, Q-2072).** Pip scales recorded once per tick in screenshots or by the
  harness's overlay read-out, with each slot's S-0506 state from the same tick.
  - **OV15 a (spawned aircraft).** S-0506's R14 setup: an own ordinary carrier with its full
    complement and an enemy target in range. Record while one aircraft launches, attacks, returns,
    is taken in and reloads, until it is docked again; then while two are out at once and the
    first of them comes back. Expected: the full pips equal the number of slots docked or
    reloading in that tick, packed from pip 0: while one aircraft is out, the last pip of the row
    is empty, not the pip at the launched slot's index; the pip refills on the tick its slot
    becomes reloading. Falsified if a returning or attacking slot shows a full pip, a reloading
    slot an empty one, or an empty pip appears before a full one.
  - **OV15 b (missile spawner).** S-0506's R2 and R5 setups: an own missile spawner while its
    missile prepares on its launcher, is released and its slot regenerates. Expected: that slot
    stops counting as full from the tick it leaves docked (preparing) and stays uncounted on the
    rail and while empty, then counts again from the tick the new missile is docked. Falsified if
    a slot whose missile prepares still counts as full.
  - **OV15 c (occupants).** An own occupiable building with at least two slots, test data in a
    test map override section if needed: two test copies of a stock garrisoning infantry type
    with different occupant pip words. Enter type A then type B, record, evacuate; enter B then
    A, record. Expected: pip 0 shows the first entrant's image frame and pip 1 the second's in
    both orders, the other slots image frame 6. Falsified if the image frames follow type or
    entry recency instead.
  - **OV15 d (occupant close-up, our engine only).** S-0508 scenario G18's engine fixture with
    three identified occupants U, V and W, entered in that order, of test types with different
    occupant pip words, in a building with at least four slots. Kill V while inside, by a defeat
    hit and, in a second fixture, by an independently supplied death event. Record the pips in
    the update of the death and the next. Expected: pip 0 shows U's image frame, pip 1 W's and
    every other slot image frame 6. Repeat with W dying: pips 0 and 1 show U and V. Falsified if
    a dead place keeps a pip, or W's pip stays at position 2. No original run is given: S-0508
    claim 20 gives no safe original result once a place is dead.
- **OV11 (claim 16).** An own damaged infantry unit while the player owns an infantry-healing building,
  recorded every drawn frame for 3 periods; then a damaged own non-organic vehicle while the player
  owns a unit-repair building; then a test vehicle type marked organic, damaged, while the player owns
  only an infantry-healing building. Also select a damaged aircraft and a damaged building while
  the player owns both healing buildings: expected no self-heal marker on either. Then, with a
  test rules file that sets the infantry self-heal
  interval to a negative value −*T*₀, the infantry unit again. Expected: image frame 13 at (25, −35)
  from *o* for the infantry unit, image frame 20 at (33, −32) for the vehicle, image frame 13 at
  (33, −32) for the organic vehicle; the marker drawn as a shadow during ticks with *t* mod *T* ≤ 5 and as an image
  otherwise, with period *T*₀ under the negative interval. Repeat each unit case at full health:
  expected always an image, never a shadow. Falsified if an image frame, place or the blink timing
  differs, a full-health marker blinks, or an aircraft or building shows the marker.
- **OV12 (claim 17, Q-2040).** Close-up screenshots at 100 % scale, one screen pixel per image
  pixel, compared pixel by pixel. Read each object's centre and sort key with the recorder. Read
  the digit glyphs' masks and widths *G*, the bitmap height *H*<sub>f</sub> and the line pitch
  *P*<sub>f</sub> from the stock game font file (S-0014), and predict each number's pixels by F6.
  - *a. Digit, glyph and box.* A selected vehicle and a selected infantry unit of the player, put
    in groups 1, 5 and 10 of the keyboard. The player's house takes a red remap colour; repeat the
    game with a blue one. Expected: digits 1, 5 and 0, each glyph left-aligned with its top stored
    row at the point (−14, −26) from the vehicle's anchor and (−14, −29) from the infantry unit's.
    Each box has the corner and size of F6, a black interior and a one-pixel border ring. The box
    behind digit 1 differs in width from those behind 5 and 0 by exactly their glyph-width
    difference, and boxes of equal-width digits match. Falsified by a centred or baseline-aligned
    glyph, a box of another size or position, a border thicker than one pixel, an interior that
    shows the scene, or a pixel changed outside the box and the glyph mask (a shadow, for
    instance). Where the object's own selection bracket or health bar falls under its box, the
    box and digit show. Falsified if those pixels show through the box.
  - *b. Colour sample.* In both games of part a, every set mask pixel and every border pixel
    equals the house display RGB of S-0103 claim 15 at display precision. Falsified if the digit
    and the border differ, or if either matches another ramp entry, another shade row or the
    definition's stored HSV bytes read as RGB, instead of claim 15's sample.
  - *c. Shading.* Repeat part a's vehicle on a cell of a named test map with neutral lighting, of
    one with night lighting and of one under a coloured light source. Then, as in OV2, place it at
    the shroud's edge so that box and digit pixels lie over shrouded cells. Capture it once more
    while the vehicle is under the invulnerability effect of the player's invulnerability
    superweapon (an iron curtain). Expected: identical number pixels in
    every capture. Falsified if any digit, border or fill pixel changes. If no box pixel can be
    placed over a shrouded cell, the shroud result remains unmeasured, not passed.
  - *d. Building.* A selected building of the player assigned to a control group. Expected: part
    a's rule from the point (−4, −36) from its building pip-scale origin *o* of claim 10. A
    different digit, placement or box falsifies it.
  - *e. Order across objects.* Two selected grouped vehicles on flat cells one cell apart along the
    map's *y* axis: a screen displacement of (−30, +15) pixels by S-0101 and sort keys differing by
    256, so S-0102 claim 6 draws the second one later. Then five selected grouped infantry units in
    one cell, choosing pairs whose sort keys differ. Expected: where a later object's status-extra
    pixels and an earlier object's box or digit coincide, the later object's pixels show. Where a
    later object's box covers an earlier object's health bar or pips, the box shows. Falsified if
    every number appears above all objects' status extras, or if an earlier object's extras show
    through a later box. If the two vehicles' extras do not overlap with the stock types'
    selection-bracket displacement, that pair checks only that each number is complete, and the
    infantry cell carries the ordering test.
  - *f. View edge.* Scroll so that the vehicle's box crosses the view's top edge, then its right,
    bottom and left edges in turn. Expected in the original: at the top, right and bottom edges,
    a border-coloured line along the view edge where it cuts the box. At the left edge, no such
    line: the visible part matches the cut image. Falsified if a top, right or bottom cut shows
    no line, or the left cut shows one. Our engine is checked against claim 17's cut image at all
    four edges, with no line along any edge.
  - *g. Test fonts.* Named test copies of the game font file, each valid under S-0014 claim 6,
    with S-0014 scenario F2's setup and restore rule. Repeat part a's vehicle in group 1 with
    each copy, and read *G*, *H*<sub>f</sub>, *P*<sub>f</sub> and the masks from that copy.
    1. *P*<sub>f</sub> = *H*<sub>f</sub> + 1, and digit 1's glyph has set pixels in its top two
       stored rows and in its bottom stored row. Expected: those pixels in the house colour on
       the scene above and below the box. Falsified if they are missing, black or cut at the box.
    2. *P*<sub>f</sub> < *H*<sub>f</sub>, and digit 1's glyph has set pixels in the rows from
       *P*<sub>f</sub> to *H*<sub>f</sub> − 1. Expected: those rows painted, with the box of F6
       for that pitch. Falsified if the digit ends at row *P*<sub>f</sub> − 1 or *P*<sub>f</sub>,
       as status text would.
    3. *P*<sub>f</sub> = 4. Expected: digit 1's set pixels drawn in the house colour. Record any
       other changed pixels around them: no original result for the box is predicted.
       Falsified if the digit is missing. Our engine is checked to draw no box.
    4. Text unit 49 (the digit 1) unmapped, and the capital X mapped with an asymmetric mask and a
       width different from digit 1's. Expected: the complement of X's mask within X's width and
       the bitmap height, in a box of X's width plus 6. In the original, the fallback's pixels
       show S-0213 claim 25's packed-colour defect while the border shows the house colour. Our
       engine paints them in the house colour. Falsified by another fallback shape or width, or a
       missing digit.
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
  throughout for the imported stage count's non-hidden bands (S-0516 claim 28).
  Add veteran buildings inside a test-map cloak field, owned by the player, a mutually allied house
  and an enemy. Record fade level and insignia every tick, with and without player sensor coverage.
  Expected: the enemy building without coverage loses its insignia at level 11 and regains it at
  level 10 on uncloaking; the owned, mutually allied and sensed buildings retain it at all levels.
  Also record a veteran disguised enemy vehicle during a fake blink that shows its real image
  (S-0516 claim 39): while it still appears disguised, its insignia remains hidden. Once detector
  coverage prevents it appearing disguised, the insignia shows. Use game-speed index 2, 800 × 600
  pixels, the common setup's restrictions on external orders, and no mods apart from the named
  test-map cloak-field and veterancy fixture. Use tick-stamped screen capture and a read-only
  recorder; repeat these added sub-tests three times.
  Falsified if any image frame or band differs, the own cloaked unit hides its insignia,
  the cloaked enemy away from sensors shows it, the allied cloaked unit hides it in skirmish,
  a building's threshold differs, or the insignia follows the disguise image rather than the
  appears-disguised test.
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

  Then use a unit carrying an attached bomb planted by the local player's house, with the
  sprite's location in view and free of shroud shading. After S-0519 claim 10's visibility
  recomputation, compare its tested cell unexplored and explored: expected no bomb sprite in
  the former state and the bomb sprite in the latter. For an independent visibility control,
  have an opponent plant a bomb on an own unit in explored territory, with no local bomb
  detector in range, then move a local detector into range. After each visibility recomputation,
  expect no icon without detection and an icon with detection, with image frames following
  S-0519 Formulas F3. Record plant ticks, detector positions and tick-stamped screenshots.
  Falsified if the period does not follow F4, a state (a) shows the wrench, a state (b) does not,
  or the bomb sprite ignores either the shroud or bomb visibility, or uses different image
  frames. An attached-bomb sub-test that cannot reach both explored and unexplored tested-cell
  states while keeping eligible bomb pixels in view is unmeasured, not passed.
  Palette (Q-1622). Part 0 (the file present): before the other parts, list the entries of
  the stock install's archives as S-0005 claims 10 and 12 mount them. Expected: the cursor
  palette is found in the cache archive nested in the base game's main archive, and no loose
  copy is in the game folder. If it is not found anywhere, parts a to e are void and the
  missing-file rule of claim 19a applies to the stock install. Part a: in state (b) on flat ground at level 0, under neutral light and with
  no shroud piece over the wrench, capture the wrench's pixels and those of a visible attached
  bomb. For each candidate palette (the cursor palette, the overlay palette and the unit palette
  in force), predict each pixel by decoding the shown frame through that palette and then
  applying the conversions that apply to the shown pixel: the multiply by 4 of S-0103 claim 1,
  the surface's component cut of S-0103 claim 9, and the light-alpha shading of claim 2 at the
  captured light.
  Judge only pixel indices at which the cursor-palette prediction, after these conversions,
  differs from the prediction under each other candidate; if no such index occurs in the shown frames, the run is inconclusive. Expected:
  each judged pixel matches the cursor-palette prediction. A match only under another candidate
  falsifies the palette row. A judged pixel that matches no candidate makes the run
  inconclusive for the palette row: it points at the conversions, not at the palette. Part b: run part a in two games of one session, one on a map of
  each of two theaters, the local player of a different side in each; expected the same
  cursor-palette prediction in both. Part c: start the program with a loose test cursor palette
  file in the game folder (S-0005 precedence; an invented palette whose colours differ from the
  stock one at the wrench's indices, still distinct after the conversions), play part a, then
  return to the menu, replace the loose file with a second invented palette whose colours, after
  the conversions, differ from the first test palette's at every judged index, and play part a
  again in the same session. Expected: both games match the first test palette. A match with
  the second palette in the second game falsifies colours that stay those of the file present
  when the program started. Engine fixture: the same steps on our engine with
  `quirks.art_cache_never_invalidated` off give the first test palette in the first game and
  the second in the second; a match with the first palette in the second game falsifies the
  switch-off lifetime. Part d (whether a loose palette added after the main menu appears
  affects the first game): start the program with no loose cursor palette file, so that the stock
  palette from the archive is the one present at start-up; at the menu, before any game, add
  the first test palette as a loose file (S-0005 claim 1 tests for a loose file at every open),
  then play part a. Expected: the wrench and bomb pixels match the stock cursor palette's
  prediction. A match with the test palette falsifies the start-up read. A match with the stock palette supports the
  start-up read only as far as S-0005 claim 1's test for a loose file at every open holds.
  Every test palette of parts c and d keeps its components at or below 63. Part e (the wrap):
  start the program with a loose test cursor palette whose components at the wrench's indices
  lie from 64 to 79, and play part a under neutral light. For each of these components the
  wrapped value (4·*c*) mod 256 lies from 0 to 60 and the cut value min(4·*c*, 255) is 255
  (S-0103's switch-off value 252 gives the same outputs after S-0103 claim 9's cut), so
  after S-0103 claim 9's cut they differ at both 5- and 6-bit outputs (at most 7 against 31 in
  5 bits, at most 15 against 63 in 6 bits); a component held at 63 gives 252, which matches
  the cut value only after S-0103 claim 9's output step.
  As in part a, judge only the indices whose wrapped and cut predictions still differ after
  every conversion, the shading included; if no such index occurs in the shown frames, the run
  is inconclusive. Expected: each judged pixel matches the prediction with each component *c*
  taken as (4·*c*) mod 256. A match with 4·*c* cut at 255, or with *c* held at 63, falsifies
  the wrap. A judged pixel that matches neither prediction makes the run inconclusive.
  Engine fixture: the same steps on our engine with `quirks.palette_component_wrap` off give
  each judged pixel the cut prediction, with *c* above 63 taken as 252; a match with the
  wrapped prediction falsifies the switch-off rule for the cursor palette.
- **OV14-D (claim 19; engine-alternative conformance, not an original-game measurement).** Replay the
  repaired-building timing test in our engine with `quirks.speed_normalised_delays` off, at game
  speeds 0, 3 and 6. Record the tick and wrench image frame for 30 ticks per speed. Expected:
  period 3 ticks and image frames 0, 3, 6 at tick residues 0, 1, 2, respectively, at every speed.
  Falsified by any different image frame or period.

**Name-tip scenarios (claims 20 to 26, Q-2673).** Common setup as above, with the tooltip option on
unless stated. A presentation-only probe string table is a loose copy of the stock string table
(resolved by S-0005) that keeps every entry and replaces only the texts of the entries a scenario
names, each by a distinct short probe text. The pointer is moved by the glossary's scripted-input
allowance. The screen is captured at 60 frames a second with the capture time of each frame. A wait
is measured from the last pointer movement to the first frame that shows the tip. A tip can first
show only in a drawn frame, and the game draws one each cycle (S-0201 claim 6), 16 × *s* ms apart at
a game-speed index *s* from 1 to 6 with a host that keeps up (S-0201 claim 2). Expected waits and
lifetimes therefore allow one such cycle at the index in force plus two capture frame times (34 ms)
of capture and timer granularity.

- **NT1 (claim 20).** One own vehicle on open ground. (a) Move the pointer onto it and rest for 15
  s. Expected: the tip shows after 1000 ms and hides 10000 ms after it showed, and does not show
  again during the rest. (b) As (a), with a 1-pixel pointer movement 600 ms into the rest that stays
  on the vehicle. Expected: the tip shows 1000 ms after the second movement, not after the first.
  (c) As (b), with a movement while the tip shows. Expected: the tip hides in that frame and shows
  again 1000 ms later. (d) As (a), with a press and release of the left, then in further runs the
  right and the middle button 600 ms into the rest, and in a fourth run a wheel turn. Expected: no
  tip during the rest after a button, and in the wheel run the tip 1000 ms after the rest began. (e)
  As (a) at game-speed indices 1 and 6. Expected: the same 1000 ms and 10000 ms each time. (f) Rest
  the pointer for 1500 ms on open ground with no object, then drive an own vehicle under the still
  resting pointer and keep it there for 3 s. Expected: no tip until the pointer moves. (g) As (a),
  with a press of the left button 3 s into the rest, while the tip shows, and its release 1 s later,
  then in further runs the right and the middle button. In three more runs, press the button with
  the pointer on open ground beside the vehicle, move the pointer onto the vehicle with the button
  held, by less than S-0214 claim 27's drag threshold so that a held right button starts no scroll
  drag (S-0101 claim 16), and rest for 3 s, then release the button and keep resting for 3 s.
  Expected: the tip hides in the frame of the press in the first three runs. In the other three, no
  tip shows with the left or the right button, while it is held or after its release, and with the
  middle button the tip shows 1000 ms after the last movement and hides in the frame of the release.
  In no run does it show again during the rest. (h) Each run starts a new game, with a second own
  vehicle W and an own building added to the set-up, and the route-mode key (S-0213 claim 16) is
  held from before the run's first click until after the last release. Runs 1 and 2 are made as the
  runs of (g) that press beside the vehicle, with the vehicle selected, so that the pointer moves
  onto the vehicle with the button held and rests there for 3 s before the release. With the left
  button that movement is by no more than S-0214 claim 18's band-box distance, so that the release
  is an ordinary click, and with the right button by less than S-0214 claim 27's drag threshold: run
  1 with the left button, run 2 with the right. In run 3, W is selected by a click on it and given a
  point by a click on open ground two cells from the vehicle, on the side away from the place of run
  1's press, so that that press is outside every point's click area (S-0213 claim 16b), both
  vehicles are then selected by a band box drawn with the route-mode key still held, and run 1 is
  repeated, except that after the rest the pointer moves back to the place of the press, on open
  ground beside the vehicle, rests there for 3 s and the button is released there, so that the click
  is refused only at its release, for route state (S-0213 claim 16d). In runs 1 to 3 no release is
  over a route point. Runs 4 to 6 use the left button, and the pointer may move by any distance
  while it is held, since none of their presses starts a band box or a scroll. In run 4 the building
  is selected and the press is made on open ground beside the vehicle, so that it is refused for
  kind (S-0213 claim 16d). In runs 5 and 6, W is first given a point as in run 3. In run 5, with W
  still selected, the press is made on that point, which the point-click test refuses (S-0213 claim
  16b). In run 6 the vehicle is selected by a click on it and the press is made on that point, which
  is highlighted, so that the press takes it (S-0213 claim 16b). In runs 4 to 6 the pointer then
  moves onto the vehicle and rests for 3 s, and the button is released there. Expected: in runs 1 to
  3 no tip shows while the button is held, run 3's rest after the move back included, or after its
  release, and runs 3 to 5 each give their refusal message. In runs 4 to 6 the tip shows 1000 ms
  after the last movement with the button held and hides in the frame of the release. In no run does
  it show again during the rest. Falsified in any part by a wait or lifetime outside the margin.
  Falsified in (b) by a wait that follows the first movement, in (d) by a tip after a button or a
  tip 1000 ms after the wheel turn, in (e) by a time that changes with game speed, and in (f) by a
  tip. Falsified in (g) by a tip that stays after the press or the release, a tip with the left or
  right button held, or no tip with the middle button held. Falsified in (h) by a tip in runs 1 to
  3, a refusal message missing in runs 3 to 5, no tip with the button held in runs 4 to 6, or a tip
  that stays after the release in runs 4 to 6.
- **NT2 (claim 20, option).** As NT1 (a) with the tooltip option turned off in the in-game options
  menu, then turned on again. Expected: no tip during a 15 s rest while off, and the tip of NT1 (a)
  after it is on again. Repeat with the option turned off in the main menu's options menu before the
  game starts, and once with the settings file's tooltip key set to no before the program starts,
  each followed by a 15 s rest. Expected: no tip in either. Then, with the option on, the vehicle
  and the pointer placed so that the tip's box lies where the open options menu leaves the
  battlefield image visible, and NT1 (a)'s tip showing, open the in-game options menu with the
  Escape key (S-0213 claim 23) without moving the pointer, and in another run open it the same way
  300 ms into the rest and keep the pointer still for 3 s. Expected: no tip in any frame that shows
  the open menu. Falsified by any tip while off, or by a tip in a frame that shows the open menu. If
  the open menu leaves no such place visible, these two runs are inconclusive.
- **NT3 (claim 21).** A probe string table gives each display-name label used here a distinct probe
  text, and the shroud label one too. Rest the pointer for 3 s on each of: an own, an allied and an
  enemy vehicle, infantry unit, aircraft (landed, then flying) and building; a civilian building; an
  own building occupied by own infantry; a tree on clear ground and a tree on an ore cell; an ore
  cell with no object; a wall; an unexplored cell with nothing in it and one holding an enemy
  building. Then with a test map that gives one enemy vehicle type the invisible flag, rest on such
  a vehicle, and with a test map that gives one enemy building type the invisible-in-game key and
  not the invisible key, rest on such a building. With S-0516's cloak fixtures, rest on an own fully
  cloaked unit, and on an enemy and an allied fully cloaked unit without and then with the local
  player's sensor coverage on its cell. Observable: the tip's text, or no tip. Expected: the type's
  probe text for every unit and building except: none for the invisible-flag vehicle, the
  invisible-in-game building and for the fully cloaked units of other houses without sensor
  coverage; the building's own text for the occupied building; the ore type's probe text for the
  tree on ore; none for the tree on clear ground, the ore cell without object and the wall; the
  shroud label's probe text on both unexplored cells. Falsified by any other text, a tip where none
  is expected, or none where one is expected. Health, owner or rank in any text also falsifies claim
  21.
- **NT4 (claim 22, disguise).** With the disguise fixtures of S-0516's scenarios (CS15), and a probe
  string table as in NT3: (a) an enemy infantry unit disguised as an infantry type of the local
  player, outside and then inside the local player's powered detector coverage; (b) an own disguised
  infantry unit; (c) an enemy vehicle disguised as a terrain object, with no disguise house; (d) an
  enemy vehicle with a disguise that has a house, if the fixture can make one; (e) with S-0516
  scenario H's fixtures, a disguised enemy building and a disguised enemy aircraft. Observable: the
  tip's text. Expected: (a) the disguise type's probe text both times; (b) its own type's probe
  text; (c) no tip; (d) the disguise's probe text; (e) each one's own type's probe text. Falsified
  by the own type's text in (a), especially inside detector coverage, by a tip in (c), or by a
  disguise's text in (e). (d) is inconclusive if the fixture cannot give a vehicle a disguise with a
  house.
- **NT5 (claim 23).** A probe string table gives labels A, B and C and the display-name labels of
  the vehicle and passenger types used distinct probe texts. Use S-0105 scenario MS-33's subject
  with a test map that gives it the turret-tip flag. Its selected turret index follows the passenger
  that enters it empty (S-0308 claim 6, S-0105 claim 43), so each index is reached through the MS-33
  passenger type that gives it: index 2 with the mode-0 passenger, 0 with mode 1, 1 with mode 2, 5
  (outside the table's numbered rows) with mode 3, and *none* with mode 17, whose weapon number has
  no turret table entry. A named rules variant of the subject maps weapon number 4 to turret index
  3, with a further passenger type of mode index 4, so that index 3 is reached as well. (a) Rest on
  the subject empty. (b) to (d) For each of these passenger types, in three named test map variants,
  (b) the type has the own-name flag, (c) in its place, a test copy of it added to the infantry list
  of the base rules copy (S-0004), with the same mode index, the own-name flag and no display-name
  field, so that it never has a label and its display name is empty (S-0004 claim 17), (d) the type
  has neither: load it as the only passenger and rest on the subject. (e) A named rules variant that
  lets the subject carry a small vehicle: load that vehicle as its first passenger and rest.
  Observable: the tip's text. Expected: label A in (a); the row of claim 23's table for the index in
  (b) to (d), with one space before the vehicle's probe text, and a leading space in (c) at indices
  1, 3, 5 and *none*; label A in (e). Falsified by a modifier that follows the weapon number where
  it differs from the index, or by any other row. (e) is inconclusive if the vehicle cannot board.
  (f) A named rules variant gives the subject rotating-barrel weapon stages: with the mode-2
  passenger of (d) aboard, rest on it, then let it fire at the enemy target until its weapon stage
  rises, and rest again. Expected: label B both times, with index 1 from the hand-over. Repeat (f)
  in two further named variants of it: one that clears the subject's gunner flag, and one that keeps
  the gunner flag and adds the charge-turret flag. Expected: label A in both, before and after the
  stage rises, *t* being *none*. Falsified by another modifier, or by a modifier that changes with
  the stage. (g) In a fourth named test map variant, the mode-1 passenger type has the own-name flag
  and the mode-2 type has neither. Load the mode-2 passenger into the empty subject, so that the
  index is 1, then the mode-1 passenger, and rest on the subject. Expected: the mode-1 type's probe
  text as the modifier (row 1 with the flag, *P* being the later passenger). Falsified by label B,
  which would follow the passenger that entered it empty. (h) A named rules variant gives the
  subject the ability of S-0516 to take a disguise. With the mode-2 passenger of (d) aboard, let an
  own subject take its disguise, then rest on it, first with the local player owning it and then, in
  another run, with an enemy of the local player owning it and no detector of the local player near.
  Expected: label B, one space and the subject's own probe text for the own subject; for the enemy
  one, the text claim 22 gives for a vehicle that appears disguised. Falsified by the disguise's
  name in the own subject's tip.
- **NT6 (claim 24).** One own vehicle with a probe display name, at the view's centre, then within
  20 pixels of its right edge, then of its bottom edge, then of the bottom-right corner. Rest on it
  and measure the box's corners and the pointer position in screen pixels. Expected: the box at
  (*x*, *y* + 16) in the centre; right edge on the view's last column near the right edge; last row
  on the pointer's row *y* near the bottom edge; both near the corner. Repeat with a probe text
  holding a line break and one holding a tab, at the centre. Expected: for the line-break probe two
  lines and the box 2 *P*<sub>f</sub> + 4 high, and for the tab probe one line and the box
  *P*<sub>f</sub> + 4 high; each box *w*<sub>t</sub> + 8 wide by claim 24's line width, with the
  glyph widths of the stock font. Then give the vehicle a probe text of enough lines that its box is
  more than 24 pixels taller than half the view's height, put the vehicle at the view's centre and
  rest on it with the pointer within 8 pixels of the view's centre row, so that the box both flips
  and reaches above the view's top edge. Expected: the box flipped, its last row on the pointer's
  row *y* and its top above the view's top edge, so that only its lower part shows. Then give the
  vehicle a probe text of three equal words joined by single spaces, each word one letter repeated,
  where *W* is the view's width in pixels measured from the capture, at 800 × 600, so that the line
  width (claim 24, from the stock font's glyph widths) of the first two words with the space between
  them and the space after them is less than *W* − 8 and that of the whole text is more than *W*,
  with a letter wide enough that the whole text has at most 255 text units (Edge cases), and rest on
  it at the view's centre. Expected: two lines, the first two words with the space between them on
  the first and the third word, with no space before it, on the second, and the box 8 pixels wider
  than the first line, shifted to the view's right edge. So under claim 24, and under the original's
  wrap at *W* (Edge cases), the first unit that passes the wrap width lies inside the third word,
  the last break point before it is the second space, and the third word alone, being no wider than
  the first two, fits on its line. A break at the first space would put the first word alone on the
  first line. No run here reaches the one pointer row where the original's flip differs from claim
  24's (Edge cases), and at the right edge the original's box stands where claim 24 places it, so
  every expectation here holds for the original. Falsified by another distance below the pointer, a
  many-line box moved down to the view's top edge, flip or shift, another box size than claim 24's
  layout gives, a box that leaves the view on the right or the bottom, or in the three-word probe
  the first word alone on the first line, another break or a space at the start of the second line.
- **NT7 (claim 25).** As NT6 at the centre, playing a country of side index 0, then 1, then 2.
  Observable: the box's pixels. Expected: black inside, a one-pixel frame and the text in RGB (164,
  210, 255) for side index 0 and RGB (255, 255, 0) for 1 and 2, after S-0103 claim 9's output step,
  and the tip over every other battlefield drawing. Repeat at the centre with a named map lighting
  variant whose ambient light is tinted, and with the tip's box partly over shroud. Also record
  where the text's first glyph is drawn. Expected also: the first glyph drawn from an origin 4
  pixels right of the box's left edge and 2 pixels below its top edge (S-0014 claim 3 gives which of
  its pixels are set). Then rest on an own building, with an own selected ground vehicle just below
  the pointer so that the tip's box covers its health bar and with an own aircraft flying across the
  box. Expected: the name tip over the health bar, the aircraft and any information tip of the
  building (S-0102 Terms), since it is not an information tip. Falsified by another fill, frame
  width or colour, a colour that changes with the tint or over shroud, a battlefield drawing over
  the tip, or another text origin.
- **NT8 (claim 26).** (a) While the tip of an enemy vehicle shows, destroy the vehicle; in another
  run let it drive away. Expected: no later than 2 s after the vehicle is destroyed, and in the
  other run by the time the vehicle is 2 cells or more from the pointer's cell, the tip has hidden
  or shows the name of what is then under the pointer (claim 26), and once hidden it does not show
  again during the rest, except at the moment the hidden tip's lifetime would have ended (10000 ms
  after it showed) when an object with a text is then under the pointer (Edge cases, *A name tip
  back under a resting pointer*). (b) Select and deselect another unit with the keyboard while a tip shows.
  Expected: no change. (c) Hold the left button, drag a selection box and rest, still holding, for 3
  s with an own vehicle under the pointer, then release the button and rest for 3 s more. Expected:
  no tip while the button is held or after its release. (d) Rest with a building placement and then
  a superweapon aim active over an own vehicle, and then with an own unit selected and the pointer,
  showing the attack cursor, on an enemy vehicle. Expected: the tip after 1000 ms in each. (e) Fill
  a strip of the map with own vehicles packed side by side, start a keyboard scroll along it and
  rest the pointer on the view during it, so that an own vehicle is under the pointer when each wait
  inside the scroll ends. Expected: no tip at the end of a wait that falls inside the scroll. (f)
  With S-0516's cloak fixtures, let an own unit that can cloak cloak while its tip shows. Expected:
  no change. (g) While the tip of an own building shows, scroll the view with the keyboard for 1 s
  and then keep the pointer still for 5 s. Expected: the tip hidden within 200 ms after the view
  first moves, and no tip for the rest of the rest, except as in (a) at the moment the hidden tip's
  lifetime would have ended. Falsified by a tip that hides or changes in (b)
  or (f), a tip that still shows the vehicle's or the building's name after the times given in (a)
  or (g), a tip that shows again during the same rest after it hid in (a) or (g), other than at the
  end of the hidden tip's lifetime with an object with a text then under the pointer, a tip in (c) or
  (e), or no tip in (d).
- **NT9 (claim 22, technician mark).** With a probe string table for the technician label, destroy
  and, in another run, sell an own and then a captured building whose survivors include a type with
  the nominal flag (set by test map if needed). Rest on each survivor. Observable: each survivor's
  tip text. Expected: every survivor that shows the technician label's probe text has the nominal
  flag, and a survivor type without the flag shows its own text in every run. Falsified by the
  technician label on a survivor without the flag. NT9 is inconclusive for the mark's effect on the
  name when no survivor shows the technician label's probe text. Which releases give the label is
  recorded for the Open question and confirms no claim.

## Open questions

- The level height of 104 leptons used for the building box (claim 7) is believed, not confirmed
  for the overlay; scenario OV5 should pin it through *Q*.
- Which corners of a building's box get selection brackets (claim 8) is not established.
- Which picture each image-frame number holds in each pip sprite should be read from the stock sprites at
  measurement time; this spec fixes only the numbers.
- The extra-information overlay that some units draw beside the pip scale is not covered, nor is the
  talk bubble that can be shown over one object at a time (it uses its own sprite, drawn at the
  status anchor without shading; what shows it is not traced).
- Name tip (claim 20): whether any sequence of skirmish play other than S-0201's order blocks locks
  the input or hides the cursor, and whether the held-button block ends at the release when a left
  press in the route mode that was an ordinary click is released over a point that is highlighted or
  that the point-click test refuses (S-0213 claim 16b): the tip may then stay blocked after the
  release until a later release of the left or right button over the battlefield view. Confidence:
  medium for the first, low for the second.
- Name tip (claim 22): which releases from a building set the technician mark. It is set for a
  survivor of a building in some building state, and when a building puts a unit of the nominal
  type on the map in some other way. Neither condition is identified. Scenario NT9 explores it.
  Confidence: low.

## Cross-spec consistency

- Hand off with S-0506 scenario R5's crash-start check (Q-2072). Under claim 11 only docked and
  reloading slots show a full pip, so a launched aircraft's pip is already empty and does not
  change when the aircraft is killed. R5 must read the slot's change at crash start from the
  slot state that S-0506 claim 9 gives, not from the pips. S-0506's rules do not change.
- Claim 17 (Q-2040) uses the game font of S-0014 and S-0213 claim 25 for a second purpose. Its
  glyph widths, masks, fallback image and transparency are S-0014 claims 3 to 5, and its advance
  is S-0213 claim 25's. Claim 17 owns the number's foreground, box and placement. S-0014 claim 6's
  whole-font rejection omits the number as it omits status text. Claim 17's colour is S-0103
  claim 15's house display RGB, as consumed by S-0410 claim 8, without changing it.
- Hand off with S-0014's summary and claim 6, and with S-0213 claim 25, noting the font's
  second use: claim 6's rejection also omits the group number, and the digit paints every
  stored row with no pitch limit, unlike status text. Neither spec's rules change.
- Hand off with S-0102 claim 1 stating that status extras are painted object by object, each
  object's complete before the next object's begin, in the layer order of claim 1 and the
  in-layer order of claims 6 and 7. Claim 17's order across objects and its scenario OV12 e rely
  on that statement. S-0102 owns the order itself.
- This draft replaces this spec's previously approved claim 9 and OV6 infiltration expectation:
  spying is not a reachable pip-reveal condition in stock skirmish. The corrected claim 9 agrees
  with approved S-0510 claim 8 and its scenario I7. Hand-off replaces the old S-0111 text; no
  companion correction to S-0510 is required. OV6 explicitly discriminates the old expectation
  (pips appear after infiltration) from the corrected one (they remain hidden).
- Q-0471 is fully answered by claim 12 and Import mapping: the former
  `quirks.pip_colour_default_shift` is withdrawn. Pip-value retention is mandatory, not a
  rendering option; OV7-R checks the corrected result. S-0003 claim 16 and S-0006 claim 5 agree.
- Hand off with S-0003 claim 16 and S-0006 claim 5 aligned to claim 12's fixed pip-value retention,
  initial image frames 1 and 7, and no pip-default switch. OV7 observes the original bug.
  OV7-R checks the correction in our engine.
- Hand off with S-0110 claim 8 owning the point test for both signs of height and
  `quirks.shroud_negative_odd_pair`. Claim 19 supplies the tested centre surface height and wrench
  observable, not an independent negative-height extension; OV14 retains the original expectations.
- Hand off with S-0214's cursor-palette row aligned to claim 19a (Q-1622): the same file, read once at start-up with
  `quirks.art_cache_never_invalidated` on and afresh at each game start with it off (S-0005
  claim 22); without it the original fails
  when it reads it, and our engine draws neither the wrench nor the bomb icon and warns once.
- Hand off with S-0103 claim 1's list of palettes loaded from palette files extended by the
  cursor palette (the same multiply by 4 in 8-bit arithmetic, Q-1622). The edge case
  `quirks.palette_component_wrap` applies to the cursor palette as to the others: off, a
  component above 63 becomes 252.
- Hand off with a row in S-0005 claim 22's table for the cursor palette: a kept file that
  follows `quirks.art_cache_never_invalidated` like every other kept file. With the switch on
  (YR) it is read once when the program starts and kept across games for the whole session.
  That a replaced file is also not followed across a change of theater or side is `unmeasured`
  (S-0111 claim 19a). With
  it off, it is resolved afresh at the start of each game. No exception to the switch's row
  is needed. Confidence: medium for the switch-on row across games (the original's behaviour),
  medium and `unmeasured` across a change of theater or side, high for the switch-off row (a design
  rule); scenario S-0111 OV14 palette parts c and d.
- Hand off with S-0105's Open questions entry on the hover tip name: claim 23 here answers it. A
  turret-tip vehicle's tip follows its selected turret index (S-0105 claim 43), not its weapon
  number. S-0105's rules do not change.
- Hand off with S-0516 claim 36's bullet for the hover name: claim 22 here uses claim 35's test for
  vehicles only. Infantry uses a simpler test, with no detector coverage and no disguise house, so
  that bullet should name vehicles only and point to claim 22 for infantry. It should also say that
  claim 35's rule for a disguised building or aircraft does not reach the hover name: claim 22 here
  shows a building's or aircraft's own type name whatever its disguise. S-0516's rules do not
  change.
- Hand off with S-0004 scenario L2: its view-height condition "more than 2 × (23 × *P*<sub>f</sub> +
  3) pixels tall" uses claim 24's former height margin of 3 pixels. With the margin of 4 pixels it
  should read 2 × (23 × *P*<sub>f</sub> + 4), so that the view is still more than twice as tall as
  (xii)'s box. L2's expectations and S-0004's rules do not change.
- Claims 21 to 23 rely on S-0004 claim 17 for every display name and fixed label they show,
  including the placeholder for a missing entry. This spec states no lookup rule of its own.
- Hand off with S-0006 claim 13 and S-0411's Import mapping: S-0006's placeholder for a missing
  label and the missing-entry diagnostic of S-0411's fixed labels are S-0004 claim 17's placeholder,
  and both should cite it. S-0411 claims 20 to 22 give the sidebar's tips and cite claims 20 and
  24 to 26 here for their timing, layout and look. Neither spec's rules change.
- Hand off with S-0102's Terms entry *information tip* and its claim 2: a building's name tip
  (claims 20 to 25 here) is not an information tip. It is drawn after the whole battlefield view,
  over the air and top layers, every status extra and any information tip, while the information
  tips keep S-0102 claim 2's place right after the ground layer. S-0102's Terms entry can say that
  the text a building shows while the pointer rests on it in its name tip is this spec's name tip,
  not an information tip. S-0102's rules do not change. Confidence: medium. Backing: `measurable`
  (NT7's building run).
