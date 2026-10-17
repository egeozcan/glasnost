---
id: S-0101
title: Isometric projection, height lift, view position and scroll limits
topic: rendering
revision: 13
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-17
measurements: []
requests: [Q-0279, Q-0450, Q-0460, Q-0570]
supersedes: []
---

# S-0101: Isometric projection, height lift, view position and scroll limits

## Summary

How a world position (leptons, with height) becomes a pixel position in the battlefield view, how
terrain, overlays and objects are lifted by height, how the view position is chosen and limited by the map's
local area, and how a screen pixel is turned back into a ground position or a cell for the cursor.
Projection and picking calculations state their number domains and rounding. Edge, right-drag and keyboard
scrolling specify their displacement, acceleration and relation to game speed and freezes.
This spec covers skirmish; network games and campaigns are out of scope.

## Terms

Terms used as defined in the shared glossary: *cell*, *lepton*, *height level*, *world height*,
*local area*, *map size*, *object*, *render layer*, *building*, *aircraft*, *infantry*, *vehicle*,
*drawn frame*, *tick*, *quirk switch*, *engine constant*, *measurement setup*, *backing*,
*confidence*, *round-toward-zero mode*. The terms below are specific to this spec.

| Term | Meaning |
|---|---|
| trunc₀ | Truncation toward zero: trunc₀(2.7) = 2, trunc₀(−2.7) = −2. |
| rz<sub>d</sub>(*v*) | *v* rounded toward zero to a double-precision value (F6, F9, F10). |
| rz<sub>s</sub>(*v*) | *v* rounded toward zero to a single-precision value (F6). |
| world pixel (*q<sub>x</sub>*, *q<sub>y</sub>*) | Integer pixel position in an unbounded plane that contains the whole map, *q<sub>x</sub>* growing to the right and *q<sub>y</sub>* downwards. The battlefield view shows a window of it. |
| battlefield view | The part of the game screen that shows the map, without the sidebar and the strip above it. |
| view size (*W*, *H*) | Width and height of the battlefield view in screen pixels, both positive integers. |
| view pixel (*v<sub>x</sub>*, *v<sub>y</sub>*) | A pixel position relative to the battlefield view's top-left pixel, which is (0, 0). Pixels inside the view have 0 ≤ *v<sub>x</sub>* < *W* and 0 ≤ *v<sub>y</sub>* < *H*. |
| view centre | The world pixel the battlefield view is centred on. It is the view's only position state. |
| view origin *O* | The world pixel shown at view pixel (0, 0). World pixel = view pixel + *O*. |
| height lift *L(z)* | Number of pixels a point at world height *z* is drawn above its ground projection. |
| drawn extent | The set of view pixels covered by an object's current image and shadow before view clipping, including image placement and height lift; selection and status markers are not included. |
| flat-ground cell under a pixel | The cell that F7 gives for a view pixel or world pixel: the cell whose ground would be drawn there if it were at height level 0, whatever its actual height. |
| probe rows | For a cursor at view pixel (*x*, *y*), the 180 view pixels (*x*, *y* + 1) … (*x*, *y* + 180) directly below it in its column (claim 10, F8). |
| ground box | The box in world leptons, given by F4a, that claim 6 tests an object's ground position against. |
| extent test | The view-intersection test of F4b for an object's drawn extent. |
| animated terrain object, terrain object that is not animated | The two groups of terrain objects distinguished by S-0102. Terrain objects that are not animated are painted in the cell pass; animated terrain objects are drawn as objects. S-0102 claim 23 specifies which terrain objects belong to each group. The on-screen tests for animated terrain objects are given in claim 6. |
| cell pass, object pass | The two drawing passes that S-0102 distinguishes, in S-0102's wording. Which images each pass draws is S-0102's to say. Claim 6 names the ones this spec relies on. |
| bib (of a building) | A part of a building's image, in S-0102's wording. S-0102 defines it. This spec says only that the cell pass paints it (claim 6). |
| *M<sub>w</sub>* | The map width of the map size (S-0009 writes it *W*; here *W* is the view width). |
| (*V<sub>x</sub>*, *V<sub>y</sub>*, *V<sub>w</sub>*, *V<sub>h</sub>*) | The four numbers of the local area: left, top, width, height. As for the map size, horizontal numbers count cell widths (60 world pixels each) and vertical numbers count cell heights (30 world pixels each). |
| scroll index *r* | The player's scroll-speed setting, 0 fastest to 6 slowest. Distinct from game speed. |
| edge momentum *a* | Non-negative integer controlling edge-scroll acceleration and run-on (F9), initially 0 at game start. Not a world velocity. |
| client pixel | Pixel measured from the game window's client top-left, including the sidebar and the strip above the battlefield. Client width and height are *B*, *C*. |

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
3. **Object height lift.** An object at integer world height *z* leptons is drawn *L(z)* pixels
   above its ground projection, using the exact rational scale and rounding in F2.
   This is the drawing lift only; it does not change simulation sight centres, which use
   the distinct floating-point simulation lift defined by S-0409 claim 5 and F2.
   This is a departure confirmed by the project lead: the original's floating-point scale,
   height-dependent correction and truncation visibly misalign objects with whole-level terrain.
   - Backing: `measurable` (M2; departure portions tested against this spec, not the original)
   - Confidence: high
4. **Terrain height lift.** Terrain tiles are lifted by exactly 15 pixels per height level
   (integer, no rounding). Objects at whole-level heights use exactly the same lift, including
   negative levels. Between levels, objects on slopes, climbing, falling or flying use F2.
   - Backing: `measurable` (M2; object alignment is a departure tested against this spec)
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
   - **Drawn extent (vehicles, infantry, aircraft, projectiles, and every kind not listed below).**
     The object's drawn extent must intersect the view (F4b), regardless of where its projected
     position lies. Thus an image or shadow enters and leaves smoothly as its pixels cross a view
     edge. This tests drawing only for objects already in the player's visible area; it never
     bypasses shroud or fog. Animations, animated terrain objects and the parts of a building drawn
     as objects skip this additional test: for them the ground box is the only on-screen test.
     Selection and status markers remain outside this extent rule.
   - **Not decided by these tests.** A building's body, shadow and bib, and terrain objects that are
     not animated, are painted in the cell pass (S-0102), so they are drawn whenever any part of
     them is inside the view, wherever the building's position lies relative to the band or the box.
     The parts of a building drawn as objects are those S-0102 places in the object pass, such as
     voxel turrets.

   An object that fails a test is not drawn at all, together with its shadow. For a building this
   applies only to its parts drawn as objects, not to what the cell pass paints.
   This is a departure confirmed by the project lead: the original tests the position against a
   band extending 360 pixels horizontally and 180 vertically beyond the view, causing visible
   images and shadows to pop when their object's position crosses that band.
   - Backing, by part of the claim:

     | Part | Backing |
     |---|---|
     | Extent intersection for aircraft | `measurable` (M4a, departure test against this spec) |
     | Extent intersection for infantry | `measurable` (M4c, departure test if its precondition holds) |
     | Extent intersection for vehicles | `measurable` (M4d, departure test if its precondition holds) |
     | Extent intersection for projectiles | `measurable` (M4e, departure test against this spec) |
     | A building's body is drawn whatever its position relative to the band | `measurable` (scenario M4b, if its precondition holds; otherwise unconfirmed) |
     | Cell-pass building shadows, bibs and non-animated terrain objects are not culled by the band or ground box | `unmeasured`: M4b observes the body only; no scenario here makes these other images reach into the view from beyond either bound |
     | Building parts drawn as objects are not culled by the original position band | `measurable` (scenario M4b, only if the chosen building has such a part inside the view in run 1; otherwise unconfirmed) |
     | Building object parts, animations and animated terrain objects skip the extent test | `unmeasured`: M4b distinguishes band culling, not an extent test; no scenario here observes drawing work for a wholly off-screen extent |
     | Other object kinds use the extent test | `unmeasured` (reason below) |
     | The ground box | `unmeasured` (reason below) |

     The extent-test rows above specify our engine's departure. The unchanged kind exemptions and
     ground box describe the original. Reasons for their unmeasured parts: the old band makes a
     visible difference only for an image, or a shadow, that reaches more
     than 180 pixels above or below its position, or more than 360 pixels to one side. The
     measurement setup cannot place an animation at a chosen position, and no stock terrain object
     or animation is known to have an image that large. The other kinds (particles, voxel
     animations and similar effects) are placed by the game itself, never by the player, and are
     drawn close to their position. The ground box's edges lie hundreds of pixels outside the view at
     any common view size, so no stock image positioned outside the box can reach into the view. An
     unmodified install cannot report culling decisions directly.
   - Confidence: high for extent intersection and the kind exemptions. High that a shadow is
     drawn as part of its object (M4a and M4c to M4e rely on it). Low that a hovering jumpjet's shadow is drawn on the ground below it; the
     preconditions of M4c and M4d check this, and it decides only whether those scenarios can test
     the extent rule. Medium for the exact box (F4a).
7. **View origin from view centre.** The game positions the view by its centre. The view origin is
   the view centre minus half the view size, each half truncated (F3). Centring the view on an object
   uses the world pixel of the object's position including its height lift (F1), so an object on high
   ground or in the air appears at the view's centre, not the ground point below it.
   - Backing: `measurable` (M3c; our engine's airborne lift uses claim 3's departure)
   - Confidence: high
8. **Scroll limits.** Every time the view centre is set, by scrolling, by centring on an object or at
   the start of the game, each component of the requested centre is limited by F5, independently of
   the other component. F5 uses the current local area, including a replacement made during play
   by S-0524 claim 8. Subsequent scroll and centring requests therefore use the changed limits,
   not the starting rectangle. The radar rectangle follows the same runtime change under
   S-0524 claim 8; radar presentation is not otherwise specified here.
   On an axis whose maximum is at least its minimum, F5 gives:

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
   When the view is oversized on an axis (maximum < minimum), the local area is centred on that
   axis independently of the requested position, using F5's fixed centre; the other axis retains
   its ordinary limits if it is not oversized.
   This is a departure confirmed by the project lead: the original chooses contradictory minimum
   or maximum bounds according to the request, making an oversized view jump.
   On each oversized axis, battlefield pixels beyond that axis's interval of F5's local-area
   pixel rectangle are black in every drawn frame: no terrain, objects, shroud or light appears
   there, regardless of whole-map reveal or the shared explored state of non-legal cells.
   With both axes oversized, the black region is the union of those horizontal and vertical
   bands. The vertical bands use the same rectangle, without shifting it to match the fixed
   centre. Selection overlays, the cursor, messages and the sidebar are unaffected and may
   appear over the black. Non-oversized axes gain no additional clipping rule.
   This is presentation only and changes no simulation state. It takes precedence over
   S-0110 claim 5's light-alpha drawing and claim 8's shroud point test for battlefield imagery
   in those bands, without changing the point test for other uses.
   - Backing: `measurable` (M3a for normal limits; M3d for runtime changes; M3b tests centring and black bands on our
     engine). Original-side backing for the black-band rule: not applicable, our departure
     accompanying the fixed oversized centre, not a claim about the original
   - Confidence: high for the formulas and the oversized-view result. Medium for the mapping of the
     map file's numbers onto the rectangle roles, which follows S-0009 and is not yet measured (M3a
     would confirm it).
9. **Screen to ground.** Turning a view pixel back into a ground position uses F6 on its world
   pixel. The result is at world height 0 and ignores terrain height. Its cell, by F7, is the
   flat-ground cell under the pixel, which is the cell under the cursor wherever no probe row
   qualifies (claim 10).
   - F6 is not an exact inverse of F1. Projecting its result back with F1, for |*q<sub>x</sub>*| ≤ 1200 and 0 ≤ *q<sub>y</sub>* ≤ 2400, gives the
     original pixel or one pixel above it, and in either case either the same column or one column
     closer to the screen column of the X = Y diagonal. Within that range it never lands below the
     pixel or farther from the diagonal. This follows from F1 and F6 and holds for every world
     pixel in the stated range. See quirk `quirks.screen_to_ground_not_exact_inverse`.
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
    - Backing: `measurable` (scenario M6; the level-0 case by M5, negative levels by M6c)
    - Confidence: high
11. **Overlay height lift outside bridge-deck cells.** Overlays (walls, fences, ore and gem
    fields, crates and the like) and their shadows are lifted on the screen like terrain tiles:
    by exactly 15 pixels per height level of their own cell (integer, no rounding), never by *L(z)*.
    - **Bridge decks are outside this claim.** Their placement and lift belong to S-0102
      claims 15 and 24. This spec supplies no separate bridge-deck screen-lift formula. Their
      depth and the world height of objects on them remain as specified by S-0102.
    - Consequence outside bridge-deck cells: an overlay stays aligned with its cell's tile at
      every level. Under claims 3–4, an object standing at that cell's whole-level height also
      stays aligned with both.
    - This claim fixes only how the lift depends on the cell's height level, not the image's
      fixed placement on its cell. The lift is added to the placement specified by S-0102 claim 24.
    - Backing: `measurable` for wall and ore images and any shadows they draw (scenario M2e).
      Other overlay kinds and their shadows: `unmeasured`, because M2e samples only walls and
      ore, and no scenario here separately tests their placement across height levels. Bridge
      decks are not asserted by this claim.
    - Confidence: high.

12. **Screen-edge scrolling.** In ordinary skirmish play, without a mouse button held or a
    planning interaction, the pointer activates edge scrolling only at client column 0 or
    *B* − 1, or client row 0 or *C* − 1 (and beyond those bounds if the pointer can reach there).
    Columns 1 and *B* − 2 and rows 1 and *C* − 2 do not activate it. These are the full client
    edges, not the battlefield/sidebar boundary. F9 gives the direction: at 800×600 the four
    corners scroll diagonally and the midpoint of each edge scrolls along that screen axis.
    Direction is retained while the pointer leaves the edge and momentum decays, so leaving
    the edge need not stop scrolling immediately. At an edge the direction is sampled again
    even when the pointer moves to a different edge without an intervening stop.
    - Backing: `measurable` (M7a, M7c)
    - Confidence: high
13. **Edge-scroll displacement and acceleration.** Each unfrozen cycle with edge scrolling
    active requests the displacement in F9, once per cycle, not once per display refresh and
    not multiplied by elapsed time. The two components of a diagonal request have the same
    magnitude as a cardinal request: diagonal distance is not normalised. F5 limits the
    resulting centre. Displacement depends on the scroll index and the imported edge-scroll
    multiplier, not on the game-speed index. Acceleration and decay advance by at most one
    stage per cycle, and only when the 16 ms pacing clock has advanced since the preceding
    stage change. Thus a slow cycle does not skip stages to catch up. At game speeds 1 to 6
    on a host that keeps up, an uninterrupted edge hold advances one stage per cycle until
    its steady speed; index 0 may repeat stages because its cycles can be shorter than 16 ms.
    More cycles per second mean faster scrolling per second even at steady speed.
    - Backing: `measurable` (M7b, M7c, M7e)
    - Confidence: high
14. **Arrow-key scrolling.** With no modifier held, each held arrow requests 21 pixels per
    unfrozen cycle along its screen axis. It does not accelerate and ignores both the scroll
    index and the edge-scroll multiplier. Two perpendicular arrows request (±21, ±21) pixels,
    opposed arrows cancel on their axis. Mouse and arrow requests add before the next displayed
    view position, subject to F5. Key down/up events take effect when accepted as input, not
    necessarily in the cycle in which the player physically presses/releases them (S-0213
    claim 1). As specified by S-0213 claim 23, these are fallback controls only when no saved
    command binding conflicts. S-0213 claim 23 owns acceptance of arrow presses and releases during an
    order block, including `quirks.blocked_arrow_events`. A modifier-assisted scroll is
    outside this claim.
    - Backing: `measurable` (M7d, M7e; S-0213 claim 23, scenario O22 for blocked input)
    - Confidence: high
15. **Scrolling while paused or frozen.** Neither edge nor arrow scrolling continues during a
    stand-still for the options menu or lost focus, or during a frozen cycle. Drawn frames during
    a freeze do not imply scroll updates. Arrow scrolling also stops during a freeze if an arrow
    was already held before it began. A freeze does not itself clear edge momentum or held-arrow
    state: when unfrozen cycles resume, scrolling can resume without starting acceleration over.
    Edge scrolling is suppressed during an order block. Under S-0213 claim 23's default
    input rule, arrow presses and releases made during the block are discarded: previously
    accepted contributions remain unchanged and continue scrolling in unfrozen cycles.
    Unblocking does not synthesize a press for an arrow first pressed during the block,
    even if it remains physically held. Events already accepted into waiting inputs before
    the block are not cancelled by this rule. S-0213 claim 23 also owns the alternative
    input rule under `quirks.blocked_arrow_events`; neither setting permits scrolling during
    frozen cycles or stand-stills. This is presentation behaviour, not an additional
    game-speed path to game state. S-0201 claims 8 to 11 specify the stand-still/freeze distinction.
    - Backing: `measurable` (M7f; S-0213 claim 23, scenario O22 for blocked input)
    - Confidence: high for scrolling suppression and retained state; medium for its visible
      resumption, which depends on accepted input and the pointer's position
16. **Right-drag scrolling.** S-0214 claim 27 owns activation, the anchor at the accepted press,
    and release classification. Once that gesture is active and scrolling is permitted, each
    unfrozen input cycle requests F10's displacement. It has no edge momentum or elapsed-time
    multiplier and ignores the imported edge-scroll multiplier. Method 0 keeps the pointer at its
    displaced location and continuously scrolls towards that displacement from the anchor.
    Method 1 scrolls in the same direction but returns the pointer to the anchor whenever either
    computed component is nonzero. Method 2 reverses both directions and returns the pointer
    likewise, so dragging right moves the viewed map imagery right rather than moving the view
    centre right. Recentring occurs even when F5 prevents actual motion. Near the client edges,
    outward components receive F10's minimum and fourfold boost. Diagonals retain their separate
    full components. Each unfrozen input cycle samples the pointer once for this update; in a
    running skirmish this is once per tick and per drawn frame, not per host mouse event or monitor
    refresh. Game speed changes cycles per second, not F10's pixels per cycle. The drag itself
    does not hide the pointer; claim 17 specifies its image. Order blocks, frozen cycles and
    stand-stills suppress drag updates as they do edge updates; the gesture's release
    classification remains S-0214 claim 27's.
    - Backing: `measurable` (M8) for reachable cases. Full-client edge substitutions and
      boosts whose anchors cannot be reached are `unmeasured`: M8 records reachability,
      but normal input cannot place an anchor outside the battlefield view.
    - Confidence: high for displacement, method direction and recentering; medium for edge
      boosts in reachable boundary cases
17. **Right-drag cursor (Q-0450).** Each permitted active drag update displays a separate,
    static drag-scroll image, not an edge-scroll image of S-0213 claim 24. Its hot spot is
    the full cursor canvas centre, (⌊*w*/2⌋, ⌊*h*/2⌋) pixels for canvas width *w* and height *h*.
    It uses the full-size image, not a radar miniature. Before applying that update's drag
    displacement, test whether a one-pixel view-centre request in each of the four screen
    directions can move the view under claim 8. The refused directions select the image
    below. Image selection does not depend on the pointer's displacement, drag method or
    scrolling magnitude; a latched drag whose magnitude rounds to zero still selects it.

    | Refused screen directions | Cursor image frame (zero-based) |
    |---|---|
    | none | 385 |
    | Up | 386 |
    | Up and Right | 387 |
    | Right | 388 |
    | Right and Down | 389 |
    | Down | 390 |
    | Down and Left | 391 |
    | Left | 392 |
    | Left and Up | 393 |
    | any other combination | 385 |

    Frames belong to the cursor image imported by S-0214 claim 22. The gesture adds no
    pointer-hiding rule; hiding caused by another interaction, such as an order block,
    remains outside this claim.
    - Backing: `measurable` (M8, including cursor-image observations)
    - Confidence: high for the images, centred hot spot and ordinary limits; medium for
      combinations reached only with an oversized view, which use claim 8's departure

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

**F2. Object drawing height lift** (*z* in leptons, a signed integer; output in pixels):

> *L(z)* = ⌊15·*z*/104 + ½⌋ = ⌊(30·*z* + 104)/208⌋

Exact rational arithmetic, rounded to the nearest integer with ties towards positive infinity,
including negative heights. There is no floating-point rounding site in this departure.
For every integer level *h*, *L*(104·*h*) = 15·*h*.
This formula applies only to drawing, not to the simulation lift for sight centres
(S-0409 claim 5 and F2); that lift retains the original floating-point arithmetic.
Check values: *L*(−104) = −15, *L*(−4) = −1, *L*(0) = 0, *L*(400) = 58,
*L*(416) = 60, *L*(1000) = 144, *L*(2080) = 300, *L*(3500) = 505.

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

**F4b. Drawn extent intersection** (integer view-pixel coordinates, departure).
Let *E* be the object's drawn extent and *R* the view's pixel set:

> *R* = pixels (*x*, *y*) with 0 ≤ *x* < *W* and 0 ≤ *y* < *H*
>
> passes ⇔ *E* ∩ *R* is not empty

A shadow belongs to *E* even if the object's body lies wholly outside *R*. The ground box of
F4a and the visibility rules remain separate prerequisites. This criterion does not prescribe
how an implementation calculates the extent.

**F5. Scroll limits for the view centre** (integers, all in world pixels):

> x<sub>min</sub> = trunc₀(*W*/2) + 30·(2·*V<sub>x</sub>* − *M<sub>w</sub>*)
>
> x<sub>max</sub> = x<sub>min</sub> + 60·*V<sub>w</sub>* − *W*
>
> y<sub>min</sub> = 15·(*M<sub>w</sub>* − 5 + 2·*V<sub>y</sub>*) + trunc₀(*H*/2)
>
> y<sub>max</sub> = y<sub>min</sub> + 30·*V<sub>h</sub>* + 135 − *H*

For a non-oversized axis (*M* ≥ *m*), with requested value *c*, minimum *m* and maximum *M*,
the result is

> limit(*c*) = *m* for *c* < *m*,  limit(*c*) = min(*c*, *M*) for *c* ≥ *m*.

The result is (limit(centre<sub>x</sub>) with x<sub>min</sub>, x<sub>max</sub>; limit(centre<sub>y</sub>)
with y<sub>min</sub>, y<sub>max</sub>) on non-oversized axes. The two axes do not affect each
other; each oversized axis independently uses the replacement below.
When *M* < *m*, the result instead ignores *c* and uses the local area's geometric centre:

> horizontal centre = 30·(2·*V<sub>x</sub>* − *M<sub>w</sub>* + *V<sub>w</sub>*)
>
> vertical centre = 15·(*M<sub>w</sub>* + 2·*V<sub>y</sub>* + *V<sub>h</sub>*)

These are exact integers. The vertical centre is the midpoint of the local area's rows,
not the midpoint of the asymmetric scroll margins. The oversized condition on the vertical
axis remains 30·*V<sub>h</sub>* + 135 < *H*, consistent with its ordinary scroll range.

With the centre at x<sub>min</sub>, the view's left edge is at world column 60·*V<sub>x</sub>* −
30·*M<sub>w</sub>* (the local area's left edge). At x<sub>max</sub>, the view's right edge is 60·*V<sub>w</sub>*
columns further right (the local area's right edge). At y<sub>min</sub>, the view's top edge is at
world row 15·(*M<sub>w</sub>* + 2·*V<sub>y</sub>*) − 75. At y<sub>max</sub>, its bottom edge is
30·*V<sub>h</sub>* + 135 rows below that.

The local-area pixel rectangle used by claim 8 has integer world-pixel bounds:

> left = 30·(2·*V<sub>x</sub>* − *M<sub>w</sub>*), right = left + 60·*V<sub>w</sub>*
>
> top = 15·(*M<sub>w</sub>* − 5 + 2·*V<sub>y</sub>*), bottom = top + 30·*V<sub>h</sub>* + 135

Left and top are inclusive; right and bottom are exclusive. Arithmetic is exact integer
arithmetic, with no rounding. Test a view pixel using its world pixel (F3).
This rectangle includes the ordinary vertical scroll margins; it is not shifted by the
oversized vertical centre, which lies 7.5 pixels above the rectangle's geometric midpoint.

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
  (glossary). Its switch, `quirks.chop_rounding`, belongs to S-0205 (number domains), and
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

**F9. Edge-scroll direction and magnitude** (claims 12 and 13).

Let (*p*, *q*) be the pointer's client pixel. When it is on an edge, form a snapped client point
(*s*, *t*): horizontally choose 0 for *p* < α·*B*, *B* − 1 for *p* > (1 − α)·*B*, otherwise
trunc₀(*B*/2). Vertically choose 0 for *q* < β·*C*, *C* − 1 for *q* > (1 − β)·*C*, otherwise
trunc₀(*C*/2). Here α = 0.16 and β = 0.21, each represented by the nearest double-precision
value. The comparisons use double-precision products rounded toward zero. Compute the bearing
*f* by S-0204 F3, including its table corrections and shared quirk, with differences
Δx = *s* − trunc₀(*B*/2), Δy = *t* − trunc₀(*C*/2) in pixels rather than leptons.
This use interprets zero bearing as screen up, not map north. Its coarse direction is

> *u* = ⌊(*f* + 128)/256⌋ mod 256,   *d* = ⌊(*u* + 16)/32⌋ mod 8.

All coarse conversions are exact integer arithmetic. The direction vectors for *d* = 0 … 7 are
respectively (0, −1), (1, −1), (1, 0), (1, 1), (0, 1), (−1, 1), (−1, 0), (−1, −1).

The magnitude table *T* has these integer values, in pixels before the imported multiplier:

| Table index *i* | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|---|
| *T(i)* | 448 | 384 | 320 | 256 | 192 | 128 | 64 | 32 | 16 |

For *r* from 0 to 6 and edge momentum *a*, the cycle's effective momentum is
*aₑ* = min(*a*, 7 − *r*). While the pointer is on an edge, or *a* > 0 with a retained direction,

> *i* = 8 − *aₑ*,   *m* = trunc₀(rz<sub>d</sub>(*T(i)* × μ)),   requested displacement = *m* × direction vector.

μ is the imported edge-scroll multiplier, a double-precision dimensionless value. The original
uses floating-point multiplication rounded toward zero, then converts to an integer by truncation
toward zero (S-0205). The formula covers finite non-negative μ whose result fits a signed
32-bit integer. Momentum arithmetic is integer, with no rounding. The magnitude is applied on
both axes for a diagonal, without a division by √2.

At a reachable edge, the next momentum is *aₑ* + 1 if the pacing clock has advanced by at least
one unit (16 ms) since the last momentum change, otherwise *aₑ*. The change restarts that
one-unit interval. If the one-pixel request in that direction is blocked by F5, no edge request
or acceleration is made. Away from an edge, a nonzero momentum still makes the request above,
but the next momentum is max(0, *aₑ* − 1) when that interval expires, otherwise *aₑ*.
Decay also restarts the interval and still occurs when the retained direction is blocked.
There is no request away from an edge with momentum 0. A freeze performs neither transition.

A fresh, uninterrupted edge hold at paced speeds therefore uses table indices
8, 7, …, *r* + 1, then repeats *r* + 1. Its steady magnitude is
trunc₀(rz<sub>d</sub>(*T(r + 1)* × μ)) pixels per cycle. The original's clock is
⌊host milliseconds / 16⌋, as in S-0201; this clock affects local camera acceleration only.

**F10. Right-drag displacement** (claim 16). The anchor (*bₓ*, *bᵧ*) and sampled pointer
(*pₓ*, *pᵧ*) are integer view pixels; their raw difference is (*δₓ*, *δᵧ*).
For a zero horizontal difference, substitute −1 at *bₓ* = 0 or +1 at *bₓ* ≥ *B* − 1.
For a zero vertical difference, substitute −1 at *bᵧ* = 0 or +1 at *bᵧ* ≥ *C*.
Otherwise keep the difference. The use of full-client dimensions with view-relative anchors
is intentional and describes the original's reachable edge boosts.

For either axis let *b* be that anchor component, *δ* its substituted difference, and *N*
its full-client dimension (*B* or *C*). Its boosted difference is

> *e* = −4·max(5, |*δ*|) when *b* < 10 and *δ* < 0
>
> *e* = 4·max(5, *δ*) when *b* > *N* − 10 and *δ* > 0
>
> *e* = *δ* otherwise.

All these operations are exact integers. With scroll index *r* from 0 to 6, define

> *g* = rz<sub>d</sub>(*K*/(*r* + 1))
>
> magnitude on an axis = |trunc₀(rz<sub>d</sub>(*e*·*g*))| pixels per cycle.

*K* = 1 for method 0 and 12 for methods 1 and 2. The direction is the sign of *e* for
methods 0 and 1, and its opposite for method 2. A zero component requests zero. The original
uses double precision, rounding each division and multiplication towards zero before the
final truncation (S-0205 claim 1); methods 1 and 2 recenter when the sum of the two magnitudes
is positive, before clipping the view request by F5. There is no diagonal normalisation.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| leptons per cell | 256 | leptons | Along each map axis |
| cell centre displacement | 128 | leptons | Added to 256 × cell index |
| leptons per height level | 104 | leptons | Fixed |
| cell diamond | 60 × 30 | pixels | Width × height of one cell's footprint |
| terrain lift per level | 15 | pixels | Exact integer, terrain tiles only |
| object lift scale | 15/104 | pixels per lepton | Exact rational, departure (F2) |
| drag edge region | 10 | pixels | Strict comparisons in F10 |
| drag minimum outward displacement | 5 | pixels | Before the fourfold boost |
| drag edge boost | 4 | multiplier | Applied before method scaling |
| ground box anchor points | (trunc₀(*W*/2), −*H*), (2*W*, trunc₀(*H*/2)), (−*W*, trunc₀(*H*/2)) | pixels from the view origin | F4a. The box spans twice the distance from the first point to the other two |
| scroll top margin | 75 | pixels | The "− 5" term of y<sub>min</sub> (5 × 15) |
| scroll bottom extra | 135 | pixels | Added to the vertical range |
| inverse coefficient *a* | 4 473 959 / 1 048 576 (4.26669979095459) | leptons per pixel | Exact value of a single-precision constant |
| cursor probe depth | 180 | pixels | Probes at 1 … 180 rows below the cursor. The largest qualifying one decides (claim 10). Applies while `quirks.cursor_probe_depth_180` is on |

## State and ordering

The view's position has one state value, the view centre; edge momentum and its retained
scroll direction are separate presentation state (F9). Every request to move it (scrolling, centring on an
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
- **`quirks.screen_to_ground_not_exact_inverse`** (default: on). F6 with the constant *a* and
  truncation. Alternative: an exact inverse that returns a point whose projection is the input pixel.
  With the quirk off, the cell under a pixel is still given by F7's pixel formula, so only the
  lepton position returned changes. No observable difference in play is known
  (claim 9).
- **Rounding mode.** F6 and the scroll calculations in F9 and F10 use floating point during play.
  They rely on the
  program-wide *round-toward-zero mode* (glossary), whose single switch `quirks.chop_rounding`
  belongs to S-0205 (number domains). This spec defines no switch of its own for it. The
  height lift F2 is exact and independent of this mode. With the mode on, F6 equals its exact formula; rounding to
  nearest would move about 0.14 % of its lepton results by one lepton and no cell (F6).
- **`quirks.cursor_probe_depth_180`** (default: on). The cell under the cursor is found from at most
  180 probe rows below the cursor (claim 10), so for a cell at height level 13 or above the pick
  no longer follows its drawn lifted diamond. Alternative: probe at least 15 × (the highest height
  level on the map) rows, so that a cell at any level *h* behaves as lifted by exactly 15*h* pixels
  and the pick follows every drawn lifted diamond. On a map whose cells are all at level 12 or
  below, both settings give the same cell for every cursor pixel. Scenario M6b tests the default.
  The result its falsifier names (a lift of 210 px at level 14) is the result with the quirk off.
- **Drawing and camera departures.** Claims 3, 6 and 8 fix presentation-only defects outright,
  with no quirk switch. Negative whole-level heights align with terrain just as positive ones do.
- **Negative ground positions.** F7 truncates toward zero, so world positions from −255 to −1 leptons
  on an axis are reported in cell 0 of that axis, not cell −1. Such positions lie outside every map.

## Import mapping

| Source | Key | Feeds | Parse | Default when absent |
|---|---|---|---|---|
| map file, `[Map]` | `Size` = left, top, width, height | map size. Only the width *M<sub>w</sub>* enters F5 | four integers | as in S-0009 |
| map file, `[Map]` | `LocalSize` = left, top, width, height | local area *V<sub>x</sub>*, *V<sub>y</sub>*, *V<sub>w</sub>*, *V<sub>h</sub>* in F5 | four integers | as in S-0009 |
| map file, cell data | cell height | height level of each cell (terrain lift, cursor probe) | integer per cell | 0 |
| No user key | none | oversized-axis black bands (claim 8) use the local-area pixel rectangle derived from the map geometry above | presentation rule, not imported data | always applied on each oversized axis, independent of reveal and shared off-map state |
| No user key | none | exact object drawing lift (claim 3, F2); simulation sight-centre lift is separately defined by S-0409 claim 5 and F2 | engine arithmetic, not imported data | drawing always uses F2 |
| settings file, options section | scroll-speed control | scroll index *r* | integer, ordinary control range 0 … 6, fastest … slowest | 3 |
| rules, `[AudioVisual]` | `ScrollMultiplier` | edge-scroll multiplier μ | floating-point number or percent, as S-0003 | 1.0; later layers retain the value in force |
| settings file, options section | scrolling-method preference | right-drag method (claim 16, F10) | integer: 0 continuous, 1 same-direction recentered, 2 reverse-direction recentered | 0 |

The scroll-speed control's positions *p* = 0 … 6 (slowest … fastest) set *r* = 6 − *p*.
There is no rules key for the one-pixel edge, the magnitude table or the unmodified arrow speed.
The speed and multiplier defaults and parsing are covered by claims 13 and 14 and M7b/M7d/M7g.

F3 and F5 use the same view size *W*, *H*, as they do in normal play. *W* and *H* come from the
player's screen resolution minus the sidebar and the strip above the view. A later user-interface
spec, not yet written and without a number, will give them. They are not rules data.

## Measurement scenarios

All scenarios use the glossary's measurement setup: an unmodified install with no loose override
files and no engine add-ons, and stock data apart from the purpose-made test maps named here. Any
game speed, 800×600 unless stated. Each skirmish has the local human-controlled player and one
bystander computer player, kept outside the observed map area under the glossary's void rule.
Run with no engine add-on and no mods (the named test files of a scenario and the
purpose-made test maps excepted). Keep the test objects visible and unshrouded throughout each
observation; fog and shroud are not culling outcomes of claim 6. The observable is a screenshot or a recording of the battlefield
view. Departure portions explicitly run on our engine and are tested against this spec rather
than the original; all other portions measure the original. "Pixel position" is read relative to
the view's top-left pixel. Rendering and the cursor pick
are deterministic, so **one run of each condition is enough unless its scenario specifies repeats**.
A second run otherwise serves to rule out a reading mistake.

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
    with the same building on level 0. Departure test on our engine: expect unchanged
    image-to-tile placement at levels 20 and 34 compared with level 0. Any pixel difference
    falsifies F2. This tests this spec, not the original. Record an original fixture rejection
    separately; (c) remains.
  - (c) Setup: flat map, level 0. A jumpjet hovering in place over the same cell with the same facing
    in five runs. Its jumpjet type's jumpjet cruise height (the height above the terrain at which it
    flies and hovers, so on this map also its world height while hovering) is set, through a test-map
    override section, to 400, 416, 1000, 2080 and 3500 leptons in the five runs. If the type's data makes it bob up and down while hovering,
    turn that off through the same override section; otherwise read its image at the middle of the
    up-and-down motion. Observable: the vertical distance between its hovering images in the 416 run
    and in each of the other four runs, matched by image pixels. Hovering images are compared only
    with hovering images, because a jumpjet type's unit is drawn differently while it stands on the
    ground. Departure test on our engine, with *L*(416) = 60: expect the 400 run 2 px lower,
    and the 1000, 2080 and 3500 runs 84, 240 and 445 px higher. Any other distance falsifies F2.
    This tests this spec, not the original.
  - (d) Levels below 0 (claim 4). The same as (b), with plateaus sunk to levels −1 and −7, if
    the game accepts a map with such heights. Departure test on our engine: expect unchanged
    image-to-tile placement at levels −1 and −7 compared with level 0. Any difference falsifies
    F2. This tests this spec, not the original. Record an original fixture rejection separately.
  - (e) Overlays (claim 11). Setup: the map of (b), plateaus at levels 0, 20 and 34, with a wall
    segment and a cell of ore placed on a plateau-top cell of each level, next to the building of
    (b). Observable: for each overlay, the vertical distance between its image and its own cell's
    tile, matched by image pixels, compared with the same distance at level 0. The same is read
    for each overlay's shadow, if the chosen types draw one. Expected: no
    difference at level 20 or 34 (the overlay lifts by exactly 15 pixels per level). Any
    overlay or shadow displacement relative to its tile falsifies claim 11. On our engine also
    require unchanged building placement; this part tests the departure against this spec. No bridge-deck screen-lift expectation is proposed
    here: that behaviour is owned by S-0102 claim 15, not claim 11.
  - (f) Exact rounding, engine conformance only (claim 3). Use a revealed scene fixture on
    our engine with identical stationary point images at known integer world heights, a fixed
    origin and ground projection, and no bobbing or placement shift. Test heights −53, −52,
    −51, −4, 0, 51, 52 and 53 leptons. Read vertical displacement from the height-0 image.
    Expected lifts are respectively −8, −7, −7, −1, 0, 7, 8 and 8 pixels. A different
    displacement falsifies F2, including its negative rounding and positive-infinity tie rule.
    One run; this is a design-rule test, not an original-game measurement.
  - (g) Drawing versus sight-centre lift (Q-0460, claim 3). Use S-0409 claim 5's
    scenario V5 part E: fresh skirmishes with a stationary sight source on broad level-19
    and level-20 plateaus, with height blocking and sight bonuses suppressed as that
    scenario specifies. On our engine also record a fixed image feature relative to its
    cell's tile and compare with the same type and facing at level 0. Expected: unchanged
    image-to-tile placement at both levels under drawing F2, but the explored footprint
    uses S-0409 claim 5's simulation centre, shifted nine cells north-west at both levels.
    An image displacement falsifies drawing F2; a ten-cell centre shift at level 20
    falsifies the separation from simulation. One run per level; original-game sight
    observations and their controls remain those of S-0409, not a drawing conformance test.
- **M3 — view position and scroll limits** (claims 7, 8).
  - (a) Normal case. Setup: a purpose-made map whose local area is smaller than the map size on all
    sides and larger than the view. Scroll hard to each of the four edges and take a screenshot.
    Observable: which cells are visible at each edge (left-most column of cells, top-most row),
    converted to world pixels with F1. Expected: the view edges given after F5. Falsified if any
    edge differs by more than 1 px from the prediction. Start of the game: on the same map, put the
    local player's start position in the local area's top-left corner cell and take a screenshot in
    the first drawn frame of the game. Expected: the view does not extend beyond the limits from F5,
    and its left and top edges are at those limits if the game centres the view on or near the start
    position. Falsified if the view extends beyond either limit. At each edge, keep scrolling
    outward for ten more drawn frames, then reverse direction for ten drawn frames, recording the
    terrain's pixel displacement in every drawn frame. Expected: the first inward scroll request
    moves the view, just as in a control reversal made away from an edge. Falsified if the edge
    run needs extra inward requests before movement begins (the State and ordering consequence).
  - (b) Oversized view. Setup: a purpose-made map whose local area width satisfies *W*/120 <
    *V<sub>w</sub>* < *W*/60, so that the view is wider than the local area but x<sub>min</sub> lies
    inside it. Its vertical scroll range, 30·*V<sub>h</sub>* + 135 pixels, is taller than the view,
    so the vertical axis is limited normally. Place unit P in the left quarter of the local area and
    unit Q at its right edge, both on level 0 and well inside it vertically. Centre the view on P, take a screenshot, then centre it
    on Q and take another. Observable: the world column of the view's left and right edges in each
    screenshot. Departure test on our engine: expect the same horizontal origin in both,
    F5's geometric local-area centre minus trunc₀(*W*/2). Request-dependent movement or another
    origin falsifies claim 8. Vertical requests retain normal limits.
    Repeat with an area wider than the view but with 0 < 30·*V<sub>h</sub>* + 135 < *H*.
    Centre on units in different rows: expect a fixed vertical origin, F5's local-area centre
    minus trunc₀(*H*/2), while horizontal requests retain normal limits. Also test a view
    oversized on both axes: every request leaves both origins fixed. One run per shape.
    Any axis coupling, jump or different fixed centre falsifies claim 8. Oversized outcomes
    test this spec, not the original.
    For each shape, also capture the battlefield pixel colours on our engine at speed 2,
    using stock terrain and object types with no engine add-on and no mods (the named test
    files of a scenario and the purpose-made test maps excepted). Make the map size
    larger than its local area, with terrain and visible objects whose images would extend
    into the bands without clipping. Test unrevealed play and whole-map reveal (S-0409
    claim 15), and both explored and unexplored shared off-map states using S-0110
    claim 8's scenario SH8 part Q2 setup. Capture after scrolling and centring requests.
    Record the world origin and compare every band pixel, excluding UI and selection
    overlays, with F5's exact integer bounds: all must be black, including corners when
    both axes are oversized. Probe the first pixel outside and the last pixel inside each
    interval; choose visible terrain inside as a control that ordinary drawing continues.
    The vertical boundary must use F5's rectangle rather than a rectangle shifted to
    symmetrically surround the fixed centre. Move the cursor and a selection overlay over
    a band and show a status message: these retain their ordinary presentation there.
    Repeat with neither axis oversized as a control: no new black band is imposed.
    One run per shape and reveal/shared-state condition. Any terrain, object, shroud or
    light pixel in a band, a shifted vertical boundary, clipping on a non-oversized axis,
    or suppression of the UI overlays by this rule falsifies claim 8. Tool: frame capture
    and pixel comparison on our engine; original-side backing is not applicable because
    this is our departure, not an original-game measurement.
  - (c) Centring with height. Setup: a large map. A unit standing on a 4-level plateau in the middle
    of the local area, far from every scroll limit, and a unit standing at level 0 likewise.
    Use the same unit type and facing in both places. Centre the view on each in turn. Observable:
    the view pixel of the same fixed feature of each unit's image. Expected: the same view pixel
    in both cases, with the plateau terrain shifted. Falsified
    if the two view pixels differ. The likeliest failure is the plateau unit appearing 60 px higher
    than the level-0 unit, which is where it would be if centring used the ground point below it.
    Airborne variant: on flat ground, use the same jumpjet type, facing and cell in two runs,
    hovering without bobbing at 416 and 2080 leptons, set by a named test-map override section as
    in M2c. Centre on it once it is stationary. Expected: the same image feature lands at the
    same view pixel in both runs. Falsified if it differs, notably by the 240-pixel lift difference on our engine (claim 3's departure)
    that ground-only centring would leave. The cell is far from all scroll limits.
  - (d) Runtime local-area change (claim 8). Use S-0524 claim 8's scenario C7, at speed 2,
    with both the initial and replacement areas larger than the view on each axis. After the
    resize action fires, scroll to all four limits and centre on units near the new edges.
    Record tile pixels to recover the view origin and capture the radar border before and
    after the action. Expected: subsequent requests obey F5 with the replacement area's
    four integers, and the radar uses that same area. Repeat with a shrink of an empty strip.
    One run per enlargement/shrink condition, no engine add-on and no mods (the named test
    files of a scenario and the purpose-made test maps excepted), using scripted input
    and frame capture. A limit still derived from the initial area, or a radar rectangle that
    remains unchanged, falsifies the runtime rule. This does not assert whether resizing alone
    immediately repositions a stationary view before another scroll or centring request.
- **M4 — drawn extent** (claim 6). Parts a and c–e test the departure against this spec,
  not the original; part b retains the original's independently needed building exemption.
  - (a) Setup: flat map. An aircraft flying with a world height *z* of at least 1700 leptons, so that
    *L(z)* ≥ 245 px, set through a test-map override section if needed. It flies straight down the
    screen (map south-east) into the view from above the top edge, with the view fixed. Record every
    drawn frame. Observable: the drawn frame in which its shadow first appears and the shadow's row in
    that drawn frame. Expected on our engine: the shadow's first pixels slide in at the top
    edge as soon as its extent intersects the view, allowing one cycle's movement. It is not
    withheld until the body's position reaches row −180. Delayed appearance or a missing
    shadow with any of its pixels inside the view falsifies F4b.
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
    *L(z)* = 300 px on our engine while it flies. Precondition: with the unit hovering inside
    the view, its shadow is drawn on the ground below its image. If it is not, record that: the scenario
    then cannot test the infantry shadow departure, and that part of claim 6 stays unconfirmed. Order the
    unit to fly straight down the screen into the view from above the top edge, and record every
    drawn frame. Observable: the drawn frame in which its shadow first appears and the shadow's row
    in it. Expected on our engine: the shadow slides in as soon as its extent intersects the
    top edge, allowing one cycle's movement. A shadow withheld until the body's position
    reaches row −180, or missing with pixels inside the view, falsifies F4b.
  - (d) Vehicles. The same as (c), with a vehicle type made a jumpjet type through the same override
    section: its imported jumpjet flag set, its imported locomotor field set to the jumpjet
    locomotor (the flag alone does not change the locomotor), and its jumpjet cruise height set to
    2080 leptons. The same
    precondition, observable, expected result and falsifier apply, for vehicles.
  - (e) Projectiles. Setup: flat map, level 0. An aircraft type that attacks by dropping
    projectiles that fall to the ground, flying at a world height from 2080 to 3300 leptons (set
    through a test-map override section if needed), so that each projectile starts its fall with
    300 ≤ *L(z)* ≤ 476 px on our engine. The projectile's image is a sprite, not a voxel model, and its art leaves
    its shadow on (the absent-key default). Two runs with the same map, target, orders and timing.
    Only the fixed view differs. In run 1 the view is placed so that the target's cell is drawn about
    20 px below the view's top edge, in run 2 about 300 px below it. Record every drawn frame.
    Observable: the number of drawn frames from the first drawn frame that shows the falling
    projectile's shadow to the first drawn frame that shows its impact (the impact animation is on
    the ground, so it is visible in both runs). Expected on our engine, with the shadow's
    whole ground placement inside both views: equal counts within one drawn frame, including
    the early fall when the projectile's body is above row −180 in run 1. A shorter run-1
    count due to the body's position falsifies F4b. If run 2 shows no shadow, the test is
    inconclusive.
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
  - (c) Non-positive levels. Use a bridge-free flat test map at level −1, then at level −7,
    with legal, usable cells and a fixed origin read from tile pixels as in M5. If the original
    rejects either fixture, that level's outcome is `unmeasured` because the setup cannot be
    reached. Probe a 7×7 pixel square around a cell's drawn top corner and around its unlifted
    top corner. Expected: every pixel selects F7's flat-ground cell, ignoring the negative
    height, not the cell whose raised-downward tile covers that pixel. Any different selected
    cell falsifies claim 10's non-positive-level rule. One run per accepted level.

- **M7 — edge and keyboard scrolling** (claims 12 to 15). Setup: a large flat purpose-made
  skirmish map with a revealed, distinctive tile grid throughout the observation area; begin
  far from scroll limits. Use 800×600, speed 2 unless varied, one isolated bystander computer
  player, no engine add-on and no mods (the named test files of a scenario and the
  purpose-made test maps excepted). Scripted input and a screen recording identify every drawn
  frame and the view origin from tile pixels. Re-centre before each sub-run and leave the pointer
  inside the client for at least 16 cycles to exhaust momentum. One run per condition, with a
  repeat only to resolve capture ambiguity. Count game-drawn frames, not monitor refreshes.
  - (a) Test each edge midpoint at the outermost pixel, then at one and four pixels inward.
    Hold for 12 cycles. Test the four client corners and the battlefield/sidebar seam likewise.
    Expected: only the outermost full-client edge starts movement; corners give both components,
    midpoint edges only one. Any motion starting one pixel inward after momentum has expired,
    activation at the internal seam, or a missing corner component falsifies claim 12.
    Repeat an outermost-edge hold with each mouse button held, without activating a right drag,
    and during a planning interaction. Exhaust momentum before each run. Expected: no ordinary
    edge request; motion attributable to edge scrolling falsifies claim 12's exclusions.
  - (b) For each scroll index 0 … 6, hold an edge midpoint and record the first 16 displacements.
    Expected: F9's table sequence, scaled by μ, ending at its stated steady value. Repeat with
    a named test-map override setting μ to 0.1 (invented test value), then μ to 1.0; the
    displacement must use truncation, not nearest rounding. For example *T(7)* × 0.1 gives
    3 pixels and *T(8)* × 0.1 gives 1 pixel. A constant speed from the first cycle, a different
    peak, a different rounding result, or a diagonal component smaller than its cardinal
    counterpart falsifies claim 13. Repeat each corner to check both components.
  - (c) At steady speed, move the pointer just inside the edge. Expected: movement continues
    in the retained direction while F9 decays to zero. Move it to a different edge while still
    accelerating: direction changes without resetting momentum. Hold outward at a scroll limit,
    then reverse: no accumulated outward displacement must be unwound. Record each origin.
    immediate stopping on edge exit, a momentum reset on the new edge, or a delay caused by
    accumulated out-of-bounds displacement falsifies the respective rule.
  - (d) Hold each arrow for 12 accepted cycles, pointer in the interior. Repeat all scroll
    indices and the two μ test maps from (b), with no modifiers. Expected: 21 pixels per
    cycle throughout. Hold two perpendicular arrows, then opposed arrows; expect (21, 21)
    in the appropriate signs and zero on the opposed axis. Combine an arrow with a steady
    edge hold: expect the sum. Any setting dependence, acceleration, diagonal normalisation
    or failure of addition/cancellation falsifies claim 14. Ignore initial/final cycles whose
    key events have not yet been accepted.
  - (e) Repeat a steady edge hold and an arrow hold at game speeds 1 and 6; include speed 0
    with host-clock timestamps if capture can identify each game-drawn frame. Expected:
    identical steady per-cycle magnitudes, different per-second distances. At index 0, stages
    may repeat within one 16 ms clock interval but never advance twice in that interval.
    Any steady per-cycle speed change with game speed or elapsed-time-scaled jump falsifies
    the corresponding claim. If index 0 cannot be captured reliably, leave that clock-phase
    consequence unconfirmed rather than substituting monitor frames for cycles.
  - (f) Add a named elapsed-time trigger starting a positive timed freeze as in S-0201 T6.
    Begin a steady edge hold, then in another run hold an arrow before the freeze; do not
    release the arrow until after it ends. Expected: origin unchanged in all frozen drawn
    frames, resuming on unfrozen cycles, with retained momentum rather than a new ramp.
    Repeat across options-menu and lost-focus stand-stills: no motion while stopped and
    no backlog jump. A frozen-frame origin change or a fresh acceleration ramp without
    an intervening pointer/input change falsifies claim 15. Separately start a chrono flash
    while holding an edge, and in another run while holding an arrow: expect edge suppression
    during its order block but continued held-arrow scrolling until its frozen cycles begin.
    Extend these runs with S-0213 claim 23's scenario O22: a new press during the block
    contributes nothing even after unblock while still held, whereas release or release/repress
    during the block leaves a previously accepted contribution unchanged. An accepted release
    after unblock ends that contribution; a fresh accepted press then starts it. Use O22's
    separate fresh runs, initial-block variant, recorder, event-boundary checks and three repeats
    per case. Read displacement as 21 pixels per unfrozen cycle per accepted arrow, with
    addition/cancellation and F5 limits as in claim 14; frozen cycles still give zero.
    Movement from a discarded new press, automatic start at unblock, or stopping from a
    discarded release falsifies the default input rule. O22 also supplies the working unblocked
    control. These trigger fixtures use S-0201 claim 11's mapping, not a new pause input
    invented for this spec.
  - (g) Remove the saved scroll index for a fresh program start, then read the scroll-speed
    control and replay (b): expect index 3 and its displacement sequence. For a named loose
    test rules file with the edge multiplier omitted, remove that key from every applied
    layer: expect μ = 1.0. Restore all test files afterwards. Another default falsifies
    the import row. This tests defaults, not a stock-data value.

- **M8 — right-drag displacement, preferences and cursor** (claims 16 and 17). Use M7's large revealed flat
  map, view away from limits, 800×600, game speed 2, scripted input and frame capture. Choose
  an interior anchor and begin a drag with a sampled crossing under S-0214 claim 27.
  For each method 0, 1 and 2 and scroll index 0 … 6, record pointer and view origin every
  unfrozen cycle. Use unequal raw components such as (17, 11) pixels. Expect F10: method 0
  repeats displacement while the pointer stays away from the anchor; methods 1 and 2 return
  it to the anchor on a nonzero request, method 2 reversing both directions. Feed a fresh
  identical displacement on each later sample to test recentered repetition. Repeat with
  M7's μ fixtures and speeds 1 and 6: per-cycle amounts are unchanged. One run per condition.
  Start anchors at view columns 0, 9 and 10 and test outward displacement after activation, then
  test corresponding reachable vertical edges and client right/bottom regions, recording
  full-client dimensions and view-relative coordinates rather than assuming all regions are
  reachable. Expect F10's strict regions, minimum 5 and fourfold boost. Once latched, a zero
  raw displacement at anchor column 0 gives the substituted outward component.
  Repeat at a view limit: centre stays bounded but methods 1 and 2 still recenter on a nonzero
  computed request and release retains selection under S-0214 claim 27. Repeat a drag through
  M7f's freeze, order block and stand-stills: no updates while suppressed. A different magnitude,
  diagonal normalisation, multiplier dependence, reversed method 0/1, failure to reverse method 2,
  recentering on a zero request or suppression failure falsifies the respective rule. Finally
  remove the saved method at fresh start: expect method 0; any other method falsifies the
  import default. Unreachable client-edge substitutions and boosts are `unmeasured`:
  view-relative anchors may not reach the full-client thresholds in the chosen layout.
  Also record the cursor image and its hot spot relative to the sampled pointer, using
  the imported cursor frames as references. Test the view interior, each limit and each
  corner, with every drag method and with a zero-magnitude sample after latching. Expect
  claim 17's static full-size frames and centred hot spot, independent of requested drag
  direction, with no hiding caused by the drag. A frame from S-0213 claim 24's edge set,
  direction-dependent image, animation, non-centred hot spot or drag-induced hiding
  falsifies claim 17. The image reflects the limits before that cycle's displacement.
  A request newly reaching a limit changes the image on the next update. One run per
  condition. On our engine add claim 8's horizontally, vertically and doubly oversized
  fixtures and expect frame 385 for the other refusal combinations; these are departure
  conformance tests, not original-game measurements.

## Open questions

- The cell under the cursor when probed cells are part of a bridge (claim 10, scope) is not
  specified here. The original treats a bridge deck as lifted above its cells' height levels and can
  select a neighbouring bridge cell, under conditions that depend on the bridge's direction and its
  neighbouring cells. A later bridge spec should specify this, with a scenario like M6 run across a
  bridge.
- The view size *W*, *H* for each resolution belongs to a later user-interface spec, not yet
  written.
- Whether any game mode lifts the scroll limits. None is known in skirmish play.
- Whether the screen band also decides anything else drawn for a building, such as its selection
  outline, its health bar and other status markers. The original appears to draw these only while
  the building's position is in the band, but no scenario checks it yet. The building's body is
  drawn regardless (claim 6).
- Scroll indices outside 0 … 6, negative or overflowing edge multipliers, modifier-assisted
  arrow speeds and right-drag method indices outside 0 … 2 are not specified here. Q-0279's
  ordinary edge and unmodified-arrow cases are answered; claim 16 adds ordinary right-drag
  methods. No dedicated measurement exercises edge momentum on an oversized axis, which
  cannot change its fixed centre under claim 8.
