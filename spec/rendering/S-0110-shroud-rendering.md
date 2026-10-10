---
id: S-0110
title: Shroud and fog rendering
topic: rendering
revision: 4
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0110: Shroud and fog rendering

## Summary

How the shroud and the fog of war are drawn for the local player: which cover piece each cell gets
from the states of the cell and its eight neighbours, which sprite the pieces come from, where a
piece is placed on the screen, how pieces of neighbouring cells combine, how a piece darkens the
drawing under it by setting the light-alpha values of S-0104 that such drawing reads, and the test
by which a world point counts as lying under the shroud. When a cell becomes explored or seen
belongs to the visibility specs; this spec reads those states.

## Terms

*Cell*, *legal cell*, *top corner*, *height level*, *world pixel*, *battlefield view*, *drawn
frame*, *image frame*, *sprite*, *shroud*, *fog of war*, *local player*, *house*,
*measurement setup*, *recorder* and *quirk switch* are used as in the glossary. The projection, the
object height lift and the terrain height lift (15 pixels per height level, S-0101 claim 4) are those
of S-0101. *Light-alpha value*, *light-glow image*, *shade row* and the brightness mapping
are those of S-0104 (claim 15). The *repair wrench* is that of S-0111 (claim 19).

| Term | Meaning |
|---|---|
| explored cell | A cell whose shroud the local player has removed. The visibility specs own when a cell becomes explored and when it is covered again. |
| seen cell | With fog of war on, a cell that the visibility specs count as seen by the local player, i.e. not under fog. The visibility specs own when a cell is seen and when it stops being seen. |
| map neighbours | The eight cells around cell (*x*, *y*), named by glossary map directions: the side neighbours N = (*x*, *y* − 1), E = (*x* + 1, *y*), S = (*x*, *y* + 1), W = (*x* − 1, *y*), and the corner neighbours NW = (*x* − 1, *y* − 1), NE = (*x* + 1, *y* − 1), SE = (*x* + 1, *y* + 1), SW = (*x* − 1, *y* + 1). On the screen N is up-right, E down-right, S down-left, W up-left, NW straight up, NE right, SE straight down and SW left. |
| covered neighbour | For the shroud: a map neighbour that is not explored. For the fog: a map neighbour that is not seen. A map neighbour that is not a legal cell is always covered (claim 1). |
| cover piece | An image-frame index from 0 to 46 chosen per cell for one cover. 0 is "no cover" and 15 is "fully covered". |
| shroud cover, fog cover | The two covers drawn for each cell (claims 1 and 4). |
| shroud sprite, fog sprite | The two sprites the pieces come from (claim 3; Import mapping gives their file names). |
| canvas | The 60 × 30 pixel screen rectangle in which the pieces of one cell may act (claim 7, F2). |
| shrouded point | A world point that the point test of claim 8 places under the shroud. |
| gap generator | The stock building type that SH1 uses to make explored cells unexplored again (Edge cases). Its rule belongs to the visibility specs. |
| cell's diamond | The 60 × 30 pixel diamond inscribed in a cell's canvas, with its top corner at the middle of the canvas's top edge: the screen area of the cell's ground at height 0. |
| test shroud sprite | The named test file of the Measurement scenarios that replaces the shroud sprite in SH1's capture sub-test B, SH6 and SH8. |

## Behaviour

1. **Shroud piece of a cell.** In every drawn frame, every legal cell whose canvas meets the
   battlefield view gets a shroud piece, chosen from the explored states of the cell and its map
   neighbours at the time of that drawn frame:
   - a cell that is not explored gets piece 15 (fully covered),
   - an explored cell gets the piece of claim 2 for its set of covered neighbours.

   A map neighbour that is not a legal cell (S-0009 claim 1) counts as covered: a cell that is not
   legal is never explored or seen. This holds whether its coordinates lie inside or outside the
   range the map size spans, so it covers the non-legal cells beside every border cell of the map's
   diamond as well as cells beyond the coordinate range. The piece
   depends only on these current states, not on the cell's history: a cell that is covered again
   after being explored gets piece 15 like a cell that was never explored. The result is the same
   as if the whole battlefield view were redrawn from the current states in every drawn frame; an
   implementation may redraw less, as long as what is shown stays the same (claim 9).
   - Backing: `measurable` (scenario SH1 with its re-cover and border sub-tests; SH8 for a
     neighbour beyond the coordinate range)
   - Confidence: high; medium for non-legal neighbours (that no rule ever marks a non-legal cell
     explored or seen) and for the re-covered case

2. **Piece for a set of covered neighbours.** The piece depends only on the *reduced set*: the
   covered side neighbours, together with each covered corner neighbour neither of whose two
   adjacent side neighbours (N and W for NW, N and E for NE, S and E for SE, S and W for SW) is
   covered. For a reduced set:
   - when it holds side neighbours only, the piece is 1·[N] + 2·[E] + 4·[S] + 8·[W], where [X] is 1
     if X is in the set and 0 otherwise; so the empty set gives 0 and all four sides give 15,
   - otherwise the piece is the entry of table T1 (Constants).

   The 256 possible sets reduce to 47 distinct sets, and the 47 pieces are all different.
   - Backing: `measurable` (scenario SH1)
   - Confidence: high

3. **Which sprite.** With fog of war off, shroud pieces are image frames of the shroud sprite. With
   fog of war on, both covers take their pieces from the fog sprite, the shroud cover included.
   - Backing: `measurable` (scenario SH2)
   - Confidence: high

4. **Fog piece of a cell.** With fog of war on, every cell that gets a shroud piece in a drawn frame
   (claim 1) also gets a fog piece, chosen exactly as in claims 1 and 2 but from the seen states: a
   cell that is not seen gets 15, a seen cell gets the piece for its set of neighbours that are not
   seen. Like the shroud piece it depends only on the current states, so a cell that stops being
   seen gets 15. The fog piece is not drawn once the local player's house has been defeated; then
   only the shroud cover remains.
   - Backing: `measurable` (scenario SH2)
   - Confidence: high for the piece; medium for the defeated case

5. **Pieces set light-alpha values.** A piece is not painted onto the screen. Its pixels change the
   light-alpha values *a* of the screen pixels under it. Drawing that is shaded by light-alpha values
   through S-0104 claim 15 uses the resulting value, so the cover darkens it: the terrain tiles and
   the objects shaded that way. Drawing that does not read light-alpha values is not darkened by the
   cover and keeps its palette colours under shroud and fog; S-0111 claim 2 names such drawing among
   the overlay images (health segments and pips). Which other kinds of drawing read the values
   belongs to the specs of that drawing. Each pixel of the piece's image frame, with palette index
   *v*, turns *a* into *a*′:
   - **shroud piece:** *a*′ = *v* for every *v* except 254; index 254 leaves *a* unchanged,
   - **fog piece**, acting on the value the same cell's shroud piece produced: *a*′ = max(0, *a* +
     *v* − 127) for *v* < 128; indices of 128 and above leave *a* unchanged.

   So a shroud pixel of index 0 makes the pixels below it black (shade row 0), index 127 leaves them
   at the default light-alpha value, and a fog pixel of index *v* darkens by 127 − *v* steps. Pixels
   outside the image frame's own rectangle are not touched. (Formulas F1.)
   - Backing: `measurable` (scenario SH3)
   - Confidence: high for the shroud rule, the fog formula and the darkening of terrain tiles,
     medium for which kinds of object drawing read the values (S-0104 claim 15 has the same caveat)

6. **Light-glow images over the covers.** The light-glow images act on the values that the pieces
   of all cells produced in the same drawn frame: each pixel *g* of a glow image turns *a* into
   *a*′ = min(255, ⌊*a*·*g* / 127⌋). Every pixel of the rectangle of the glow image's image frame
   that lies in the battlefield view takes part; no index is skipped, so a glow image is expected to use 127 wherever
   it should leave the light unchanged. A glow therefore scales the covered value and cannot lighten
   a pixel whose shroud value is 0: a lamp does not show through full shroud, but it does brighten
   fog.
   - Backing: `measurable` (scenario SH4)
   - Confidence: high for the formula and for its acting on the values the covers produced; medium
     for the order among several glows (Open questions)

7. **Pieces lie on the height-0 grid.** The piece of cell (*x*, *y*) is placed as if the cell were
   at world height 0, whatever its terrain height. The top-left corner of the cell's canvas is at the
   world pixel of the cell's top corner at height 0 (S-0101), moved 30 pixels left; the canvas is one
   cell wide and one cell high on the screen (60 × 30 pixels), and pixels of the image frame that
   fall outside the canvas are not applied. So on raised ground the cover of a cell appears below
   the cell's drawn terrain, by the terrain height lift of its height level (S-0101 claim 4), and
   covers what is drawn
   there, which can be other cells and the lower parts of objects.
   - Backing: `measurable` (scenario SH5)
   - Confidence: high

8. **Shrouded points.** Consistent with claim 7, the original decides whether a world point (*X*,
   *Y*, *Z*), integers in leptons with *Z* ≥ 0, lies under the shroud for the local player by
   projecting it down the screen onto the height-0 grid. With *u* = ⌊*Z* / 104⌋, the number of whole
   height levels, and (*c*<sub>x</sub>, *c*<sub>y</sub>) = (trunc(*X* / 256), trunc(*Y* / 256)),
   divisions on integers, truncated toward zero:
   - for even *u*, the point is shrouded when cell (*c*<sub>x</sub> − *u*/2, *c*<sub>y</sub> − *u*/2)
     is not explored,
   - for odd *u*, it is shrouded when neither cell (*c*<sub>x</sub> − (*u* + 1)/2, *c*<sub>y</sub> −
     (*u* + 1)/2) nor cell (*c*<sub>x</sub> − (*u* − 1)/2, *c*<sub>y</sub> − (*u* − 1)/2) is explored.

   A candidate cell whose coordinates both lie in 0 … 511 and that is not a legal cell counts as
   not explored. A candidate with a coordinate outside 0 … 511 is read as another cell, or as no
   cell (Edge cases, `quirks.shroud_point_cell_wrap`). For odd *u* the point is drawn half-way between
   the height-0 positions of the two candidate cells, so it counts as uncovered when either is. Which
   rules use this test belongs to their own specs; S-0111's repair wrench uses it, and scenarios SH6
   and SH8 observe the test through it.
   - Backing: `measurable` (scenario SH6; SH8 for a non-legal candidate cell inside 0 … 511 and for
     a candidate outside 0 … 511)
   - Confidence: high for the formula; medium for non-legal candidate cells and for candidates
     outside 0 … 511. Heights below 0 are not covered by this claim; S-0111 claim 19 states the
     test for the centre heights below 0 that its wrench uses.

9. **When pieces change.** Every drawn frame shows the pieces for the states at the time of that
   drawn frame. When the explored or seen state of a cell changes, the pieces of that cell and of
   its map neighbours show the new state from the first drawn frame after the change; no drawn
   frame shows the old state after it. Pieces follow drawn frames, not ticks. Two consequences can
   be seen within a single drawn frame:
   - the pieces of all cells in it agree with one set of states: a cell never shows its new state
     while a neighbour's piece still shows the old one, or the other way round.
   - they agree with other drawing in the same drawn frame that reads the same states, such as the
     repair wrench of S-0111 claim 19, which uses the point test of claim 8.

   An implementation may cache pieces, as long as no drawn frame shows a stale one.
   - Backing: `measurable` (scenario SH1, its drawn-frame capture sub-tests)
   - Confidence: medium

10. **Neighbouring canvases.** The canvases of cells (*x*, *y*) and (*x* + 1, *y*), and of (*x*, *y*)
    and (*x*, *y* + 1), overlap in a 30 × 15 pixel rectangle (F2); canvases of cells whose *x* + *y*
    differ by 0 or by 2 or more do not overlap. Where two canvases overlap, the result depends on the shroud
    pixel of the cell with the larger *x* + *y* (the lower one on the screen): wherever it is not
    254, it sets the value, whatever the upper cell's shroud and fog pieces produced, and the lower
    cell's fog pixel acts on that value; where it is 254, the lower cell's fog pixel acts on the
    upper cell's result (Observable order in State and ordering). So a fogged cell's darkening does not show
    where a lower neighbour's shroud piece has pixels other than 254. In the stock sprites no two
    neighbouring pieces change the same pixel, so this is visible only with modded sprites.
    - Backing: `measurable` (scenario SH7)
    - Confidence: high

## Formulas

**F1. Light-alpha update by one piece pixel** (integers). *a* is the screen pixel's current
light-alpha value and *v* the piece's palette index at that pixel.

> shroud: *a*′ = *v* if *v* ≠ 254, else *a*′ = *a*
>
> fog: *a*′ = max(0, *a* + *v* − 127) if *v* < 128, else *a*′ = *a*
>
> glow (claim 6): *a*′ = min(255, ⌊*a*·*g* / 127⌋) for a glow pixel *g*

All operands are non-negative integers, so the floor is integer division. The resulting *a* feeds
the shade row ⌊*a*·ρ·(*N* − 1) / 32258⌋ of S-0104 claim 15, with ρ and *N* as defined there.

**F2. Canvas placement** (integers, world pixels). With *P*(*x*, *y*) the world pixel of the top
corner of cell (*x*, *y*) at world height 0 (S-0101), the cell's canvas spans

> *q*<sub>x</sub> from *P*<sub>x</sub> − 30 to *P*<sub>x</sub> + 29, *q*<sub>y</sub> from
> *P*<sub>y</sub> to *P*<sub>y</sub> + 29

and the image frame is drawn at its own position inside that canvas. Since *P*(*x* + 1, *y*) =
*P*(*x*, *y*) + (30, 15) and *P*(*x*, *y* + 1) = *P*(*x*, *y*) + (−30, 15) (S-0101), the lower half
of each canvas is shared with the upper halves of the canvases of (*x* + 1, *y*) and (*x*, *y* + 1).

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Cover pieces | 47 | image frames (0 … 46) | claim 2 |
| No-cover piece | 0 | image frame | claim 1 |
| Full-cover piece | 15 | image frame | claim 1 |
| Shroud transparent index | 254 | palette index | claim 5 |
| Fog neutral index | 127 | palette index | claim 5; also the default light-alpha value of S-0104 |
| Fog ignored indices | 128 … 255 | palette index | claim 5 |
| Glow scale | 127 | glow value per factor 1.0 | claim 6 |
| Canvas | 60 × 30 | pixels | claim 7 |
| Canvas shift | 30 | pixels, to the left of the top corner | claim 7 |
| Height level | 104 | leptons | claim 8 (glossary) |
| Cell size | 256 | leptons | claim 8 (glossary) |
| Point-test coordinate range | 0 … 511 | cell coordinates, on each axis | claim 8; the wrap of Edge cases uses 512 |

**T1. Pieces for reduced sets that contain a corner neighbour.** (Sets with side neighbours only use
the sum of claim 2.)

| Reduced set of covered neighbours | Piece |
|---|---|
| N, SE | 16 |
| E, SW | 17 |
| N, E, SW | 18 |
| S, NW | 19 |
| E, S, NW | 20 |
| W, NE | 21 |
| N, W, SE | 22 |
| S, W, NE | 23 |
| N, SW | 24 |
| E, NW | 25 |
| S, NE | 26 |
| W, SE | 27 |
| N, SE, SW | 28 |
| E, NW, SW | 29 |
| S, NW, NE | 30 |
| W, NE, SE | 31 |
| NW | 32 |
| NE | 33 |
| SE | 34 |
| SW | 35 |
| NW, NE | 36 |
| NE, SE | 37 |
| SE, SW | 38 |
| NW, SW | 39 |
| NW, NE, SE | 40 |
| NE, SE, SW | 41 |
| NW, SE, SW | 42 |
| NW, NE, SW | 43 |
| NW, SE | 44 |
| NE, SW | 45 |
| NW, NE, SE, SW | 46 |

## State and ordering

**Per cell, per drawn frame.** The two covers follow their own states independently. Whether a cell
can be seen without being explored belongs to the visibility specs; the pieces need no assumption
about it.

| Cover | Drawn when | State of the cell | Piece | Sprite |
|---|---|---|---|---|
| shroud | always | not explored | 15 | shroud sprite (fog off) or fog sprite (fog on) |
| shroud | always | explored | claim 2 over unexplored neighbours | as above |
| fog | fog of war on and local player not defeated | not seen | 15 | fog sprite |
| fog | fog of war on and local player not defeated | seen | claim 2 over unseen neighbours | fog sprite |

**Observable order.** For one screen pixel, let *C*<sub>1</sub>, *C*<sub>2</sub> be the cells whose
canvases contain it (one or two, claim 10), with *x* + *y* of *C*<sub>2</sub> larger. Drawing at that
pixel that reads light-alpha values (claim 5) is shaded with the light-alpha value

> *a* = *G*<sub>*k*</sub>(… *G*<sub>1</sub>(*F*<sub>2</sub>(*S*<sub>2</sub>(*F*<sub>1</sub>(*S*<sub>1</sub>(*a*<sub>0</sub>))))) …)

where *a*<sub>0</sub> is the pixel's light-alpha value before the covers, *S*<sub>*i*</sub> and
*F*<sub>*i*</sub> are the shroud and fog rules of F1 for the pixel of *C*<sub>*i*</sub>'s shroud and
fog piece there, and *G*<sub>1</sub> … *G*<sub>*k*</sub> the glow rule of F1 for each light-glow
image over the pixel (their order is an open question). A rule whose piece does not reach the pixel,
or is not drawn, leaves the value unchanged. Consequences: a fog piece darkens on top of the shroud
piece of the same cell; a lower cell's shroud piece can replace an upper cell's fog darkening
(claim 10); and a light glow scales the result of all covers.

## Edge cases and quirks

- **The cover ignores terrain height** (claims 7 and 8), so on hills it is displaced from the cells
  it belongs to. `quirks.shroud_on_flat_grid` (default: on, the original). Off: each canvas is
  placed at the top corner of its cell at the cell's own terrain height, and the point test of
  claim 8 uses the cell under the point (*c*<sub>x</sub>, *c*<sub>y</sub>) directly.
- **Fog of war changes the shroud sprite** (claim 3). This is intended behaviour, not a quirk: the
  fog sprite is the cover set designed for games with fog of war, and it serves both covers there.
  Our engine does the same and has no switch.
- **Non-legal neighbours count as covered** (claim 1). Every cell on the border of the map's
  diamond of legal cells has map neighbours that are not legal, so when it is explored it gets edge
  pieces towards them, and these never go away. Whether those border cells lie inside the local area
  (S-0009), and so can be seen, depends on the map. This is the original's behaviour and our engine
  reproduces it; it needs no switch.
- **Cell coordinates outside 0 … 511 in the point test** (claim 8). The original reads a candidate
  cell (*x*, *y*) of the point test as the cell (*x* mod 512, *y* + ⌊*x* / 512⌋) when
  *y* + ⌊*x* / 512⌋ lies in 0 … 511, and as no cell (not explored) otherwise. All values are
  integers; *x* mod 512 lies in 0 … 511, and ⌊ ⌋ rounds toward −∞. So a
  candidate with *x* < 0 is read as (*x* + 512, *y* − 1), and one with *x* > 511 as (*x* − 512,
  *y* + 1). This can read a legal cell only on maps whose legal cells reach coordinates near 511
  (W + H close to 512, S-0009 claim 2), for points high above cells near the map's edge.
  `quirks.shroud_point_cell_wrap` (default: on, the original). Off: a candidate with a coordinate
  outside 0 … 511 is not legal and counts as not explored. The map neighbours of claim 1 never read
  a legal cell this way: legal cells have coordinates 1 … 511, and a neighbour with a coordinate of
  0 or 512 is read as a cell that is not legal, or as no cell. Example: on a map of size
  258 × 254 (W + H = 512), a point 4 height levels above the centre of the corner cell (1, 258)
  has the candidate (−1, 256), which is read as the legal cell (511, 255) at the opposite end of
  the map. Confidence: medium. Scenario SH8.
- **Overlapping pieces** (claim 10). The lower cell's (larger *x* + *y*) shroud piece wins even over
  the upper cell's fog darkening. This is the original's composition rule, not a quirk, and our engine reproduces it.
  With sprites whose pixels stay inside their own cell's diamond, as the stock ones do, the order
  has no visible effect.
- **Re-covered cells.** A cell that becomes unexplored again (for example under an enemy gap
  generator) or stops being seen gets piece 15 for that cover from the next drawn frame, and its
  neighbours get edge pieces towards it, as for any other covered cell (claims 1, 4 and 9).

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| file `SHROUD.SHP` (resolved by S-0005) | shroud sprite, image frames 0 … 46 (claim 3) | sprite | no shroud pieces drawn; the importer warns |
| file `FOG.SHP` (resolved by S-0005) | fog sprite, image frames 0 … 46 (claims 3, 4) | sprite | no pieces drawn with fog of war on; the importer warns |
| game setting fog of war (lobby or map; the visibility specs own it) | claims 3 and 4 | boolean | off |

A missing sprite is bad data: the original draws no cover from it. Our engine does the same and the
importer warns. An image frame index that the sprite does not have draws nothing.

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*, a tiny purpose-made map of one theater with flat
ground except where stated, one human player and one computer player far away unless a scenario states otherwise, no lighting effects
(ambient neutral). Screenshots at a fixed screen resolution; pixels are compared with the image frames
of the shroud and fog sprites read from the stock archives, and with the shade rows of S-0104.

**Test shroud sprite** (a named test file: a loose shroud sprite that wins over the stock one, removed
after the run; used only with fog of war off). Image frames 0 … 46 each cover the 60 × 30 rectangle.
Every image frame is index 254 everywhere, except image frame 15, which is index 64 inside the
cell's diamond and 254 outside it. Under it, by claim 5 and F1, every unexplored cell's diamond is shaded with light-alpha
value 64 (clearly darker, but not black), and nothing else is darkened. Two things follow. Whether a
cell is explored can be read from its diamond alone. And drawing over unexplored cells, such as the
repair wrench, stays visible, so whether the wrench is drawn no longer depends on the cover over it.

**Number of runs.** Drawing is deterministic for given states, and every expected result below is read
against the states recorded in the same screenshot or captured drawn frame. The random seed and the
computer player's choices can change which states occur, but not whether a run passes. So each
scenario needs one valid run per case, unless it says otherwise. A run is void when the stated
precondition does not occur (for example, the computer player does not start a repair); a void run
is repeated, up to 5 attempts per case, and a case with no valid run after that stays unmeasured.

**Drawn-frame capture** (SH1's capture sub-tests). A *recorder* that captures whole drawn frames and holds
every one of them: the display path waits for the monitor's refresh, and a video capture records every
refresh. The record names the display path generically. A drawn frame shown over several refreshes
appears several times; that is harmless, since each check below is made within one captured image.

- **SH1 (claims 1, 2, 9).** Fog of war off. Local area covering the whole map (S-0009 claim 4), so
  that units can reach the border of the map's diamond.
  - **Main test.** One stock infantry unit of the player stands still in the middle of the map. After
    it has revealed its surroundings, screenshot the area. Then move it two cells and screenshot again.
  - **Re-cover sub-test.** The computer player owns a powered stock gap generator next to ground that
    the human player's unit has explored. The unit then leaves the area. Once the area has been
    covered again, screenshot it.
  - **Border sub-test.** The unit walks to a cell on the border of the map's diamond (S-0009 claim 1)
    and back to the middle of the map. Screenshot the border cells while the unit is there and after
    it has left.
  - **Capture sub-test A (neighbour agreement).** During the main test's move, capture every drawn
    frame (Drawn-frame capture).
  - **Capture sub-test B (agreement with the wrench).** With the test shroud sprite, a stock building
    of the computer player with a one-cell foundation stands on flat ground at level 0 in unexplored
    ground, placed damaged (map data) and repaired by the computer player (it has credits). The
    human player's unit walks towards the building until the building's cell (*c*) is explored, then
    on past it. Capture every drawn frame of the walk. The run is void if the repair has not started
    before (*c*) is explored. 5 valid runs, since each run gives one change of (*c*).
  - Observable: for each cell near the edge of the explored area, whether it is unexplored (dark) and,
    for explored cells, which image frame of the shroud sprite its darkening matches. In sub-test B,
    for each captured drawn frame: whether the diamond of (*c*) is darkened (image frame 15 of the test
    shroud sprite) and whether the wrench is drawn. A captured drawn frame counts as showing the wrench when it holds
    the pixels of one of the wrench sprite's image frames at the wrench's place (S-0111 claim 19).
    If one of those image frames has no pixels, a missing wrench in one captured drawn frame is no
    evidence on its own, and only the first half of sub-test B's falsifier (a wrench over a
    darkened diamond) applies.
  - Expected: every explored cell shows the piece that claim 2 gives for its unexplored neighbours as
    read from the same screenshot, and unexplored cells show piece 15. In the re-cover sub-test, the
    covered area shows piece 15 inside, and the explored cells around it show edge pieces towards it.
    In the border sub-test, every explored border cell shows the piece that claim 2 gives when its
    non-legal neighbours count as covered, so it keeps edge pieces towards the outside of the map
    while the unit is there and after it has left. In sub-test A, every captured drawn frame passes
    the main test's check on its own. In sub-test B, every captured drawn frame shows the wrench
    exactly when the diamond of (*c*) is not darkened (claim 8 tests (*c*) itself at level 0).
  - Falsified if: any explored cell shows another piece; the re-cover sub-test's area shows pieces
    that no assignment of explored states makes consistent with claims 1 and 2 (for example a
    piece-0 cell beside a piece-15 cell); an explored border cell shows no edge towards a non-legal
    neighbour, or loses it; a single captured drawn frame in sub-test A fails the check; or a single
    captured drawn frame in sub-test B shows the wrench over a darkened diamond of (*c*), or no
    wrench over an undarkened one.
- **SH2 (claims 3, 4).** As SH1's main test with fog of war on: the unit walks away so that part of
  the explored area becomes unseen. **Defeat sub-test:** the human player's only object is one
  infantry unit, and it walks into the range of a stock defensive building of the computer player
  placed by map data, which destroys it. The defeat sub-test is void if the game leaves the
  battlefield view before a screenshot can be taken after the defeat.
  - Observable: as in SH1, for both covers: which image frame, and of which sprite, the shroud
    darkening and the fog darkening of each cell near the edges match. After the defeat, whether
    the fog darkening remains.
  - Expected: shroud and fog pieces both match image frames of the fog sprite; fogged cells show the
    fog piece over the shroud piece as F1 combines them; every fog piece is the one claim 2 gives
    for the fogged (dark-grey) neighbours read from the same screenshot; and after the defeat the
    fog darkening vanishes and the shroud darkening stays.
  - Falsified if: the shroud pieces match the shroud sprite with fog on, a fog piece disagrees with
    claim 2 for the fogged cells around it, or the fog darkening remains after defeat.
- **SH3 (claim 5).** One run with fog of war off and one with it on. A stock vehicle of a neutral
  house (one that reveals nothing for the human player) stands on flat ground, placed by map data
  with no orders; a run in which it moves before the screenshot is void. The human player's
  infantry unit stands so that the edge of its revealed area runs through the vehicle's cell: that
  cell is explored and its map neighbour N is not, so the cell gets a shroud edge piece (claim 2)
  and the vehicle is drawn partly over that piece. In the fog-on run the infantry unit then steps
  back until the vehicle's cell is explored but no longer seen, so that a fog piece 15 acts on top
  of the shroud edge piece. A reference screenshot shows the same vehicle type, with the same
  facing, on uncovered ground.
  - Pixels: from the image frames of the pieces the cell gets (read from the sprite that claim 3
    names), pick at least
    three terrain pixels and three vehicle pixels inside the piece's rectangle: one under a piece
    pixel of index 254 (shroud) or 128 or above (fog), as a control, and the others under pixels of
    different indices below 127.
  - Observable: the shade row of each chosen pixel, found by matching its colour against the shade
    rows of S-0104 for the palette index the reference screenshot shows at that place.
  - Expected: each pixel shows the shade row that S-0104 claim 15 gives for *a* from F1 (*a* = *v* for
    the shroud, and the fog combination where both apply); the control pixels keep their uncovered
    shade row.
  - Falsified if: the observed shade rows differ by more than one row from F1 and claim 15.
- **SH4 (claim 6).** A stock building that casts a light glow (a lamp post or similar), placed so that
  its glow overlaps the edge of the shroud; with fog of war on, a second lamp in a fogged area.
  - Observable: the shade rows of fully shrouded pixels inside the glow, and of fogged pixels inside
    and outside the glow.
  - Expected: pixels under full shroud stay black inside the glow; fogged pixels inside the glow are
    brighter than fogged pixels outside it, by the factor *g*/127 (truncated).
  - Falsified if: the glow lightens fully shrouded pixels, or fogged pixels inside the glow differ
    from F1 by more than one shade row.
- **SH5 (claim 7).** A map with a plateau 4 height levels high next to flat ground; the player's
  unit explores the flat ground only, up to the foot of the plateau.
  - Observable: the screen position of the dark diamond of each unexplored plateau cell relative to
    the cell's drawn terrain, and whether a unit later walking onto an explored plateau cell is
    darkened.
  - Expected: the cover of each unexplored plateau cell appears 60 pixels (4 × 15, the terrain
    height lift of S-0101 claim 4) below its drawn terrain, on the screen position the cell would
    have at height 0. A unit on the plateau is darkened exactly where the pixels it is drawn on lie
    under a cover piece, whatever the state of the cell it stands on.
  - Falsified if: the cover follows the plateau's terrain.
- **SH6 (claim 8).** Fog of war off, with the test shroud sprite. A test map with plateaus 3 and 4
  height levels high and flat ground around them, with ramps so that infantry can reach every side.
  On a flat cell of each plateau's top, and on the flat ground as a control, stands a stock building
  of the computer player with a one-cell foundation, placed damaged (map data), which the computer
  player repairs (it has credits; a run in which it does not start repairing is void). The building
  is at cell (*c*<sub>x</sub>, *c*<sub>y</sub>); its wrench uses the centre of that cell at the
  cell's terrain height (S-0111 claim 19), so *u* is the plateau's height in levels. Write (*c* −
  *k*) for cell (*c*<sub>x</sub> − *k*, *c*<sub>y</sub> − *k*). The human player has two infantry
  units. Unit B approaches the building from map south-east (the side of the larger *x* and *y*)
  and unit A from map north-west, each one cell per step along the diagonal through the building,
  and each stops at the step at which its revealed area first takes in its target cell, so that a
  gap of unexplored cells stays between them. The cases, each with one valid run:
  - 3-level plateau. Case 3a: B's target is (*c*), A stays away. Case 3b: B's target is (*c* − 1),
    A stays away. Case 3c: B's target is (*c*) and A's target is (*c* − 2), leaving (*c* − 1)
    unexplored.
  - 4-level plateau. Case 4a: B's target is (*c* − 1), A stays away. Case 4b: B's target is
    (*c* − 1) and A's target is (*c* − 3), leaving (*c* − 2) unexplored. Case 4c: B's target is
    (*c* − 2).
  - Flat control. B's target is (*c*).

  If a step explores more cells than intended, the run still counts: the expected result follows
  the states actually recorded. In every case the building's own cell (*c*) is explored, so the
  building is drawn whatever S-0111 decides for objects in unexplored cells.
  - Observable: after each run, whether each cell (*c* − *k*), *k* = 0 … 3, is explored (read from
    whether its diamond is darkened under the test shroud sprite), and whether the building shows
    the repair wrench (S-0111).
  - Expected (all states with (*c*) explored):

    | Plateau | (*c* − 1) | (*c* − 2) | (*c* − 3) | Wrench |
    |---|---|---|---|---|
    | 3 levels | not explored | not explored | any | not shown |
    | 3 levels | explored | not explored | any | shown |
    | 3 levels | not explored | explored | any | shown |
    | 4 levels | explored | not explored | not explored | not shown |
    | 4 levels | explored | not explored | explored | not shown |
    | 4 levels | any | explored | any | shown |
    | flat | any | any | any | shown |

  - Falsified if: any row differs. In particular, the wrench following the building's own cell on a
    plateau (shown in the "not shown" rows), depending on only one of the two candidate cells on the
    3-level plateau, or depending on (*c* − 1) or (*c* − 3) on the 4-level plateau.
- **SH7 (claim 10).** Fog of war on, flat map, with a named test file: a loose fog sprite whose image
  frames 0 … 46 each fill the whole 60 × 30 rectangle, with index 127 everywhere in every image frame
  except image frame 15, which is index 64 everywhere. The human player's unit explores an area and then
  walks away, so that a fogged area borders a seen area. Screenshot the border. The test file is
  removed after the run.
  - Under the test sprite every shroud piece of an explored cell sets 127, a fog piece 15 darkens by
    63 steps and every other fog piece changes nothing. Each screen pixel lies in the upper half of
    exactly one cell's canvas (on flat ground, the upper half of the bounding box of that cell's
    tile) and in the lower half of one other, upper cell's canvas.
  - Observable: the darkened terrain pixels near the border, compared with the grid of canvas upper
    halves, each 60 × 15 pixels and made of a left and a right 30 × 15 half.
  - Expected: each pixel takes the result of the lower cell, the one whose upper half holds it, so
    the darkened area is a union of whole upper halves (those of the fogged cells), darkened by 63
    steps.
  - Falsified if: some upper half is darkened in only its left or only its right 30 × 15 half. That
    is what the upper cells' fog darkening surviving under the lower cell's shroud piece would show.
- **SH8 (claims 1 and 8 at the edge of the coordinate range, `quirks.shroud_point_cell_wrap`).**
  Fog of war off, with the test shroud sprite. A test map of size W × H = 258 × 254, so that
  W + H = 512 and the cell (511, 255) is legal (S-0009 claims 1 and 2:
  *x* + *y* = 766 = W + 2H and *x* − *y* = 256 < W). Ground at level 0, except a plateau 4 levels
  high, flat on top, at the corner of the map around cell (1, 258). The local area is
  (−1, −2, W + 2, H + 2), so that the plateau's cells pass the height-aware test of S-0009 claim 4
  and the view can scroll above the top of the map (S-0101 claim 8). On the plateau stand two stock
  buildings of the computer player with one-cell foundations, placed damaged (map data), which the
  computer player repairs (it has credits): building P at (1, 258) and building Q at (2, 259). One
  infantry unit of the human player stands on the plateau beside them, so that the cells of P and Q
  are explored. For a point 4 levels above the centre of a cell, claim 8 tests the cell two steps
  back on both axes:
  - P's candidate is (−1, 256). With the switch on (the original) it is read as the legal cell
    (511, 255) (Edge cases); with it off it counts as not explored.
  - Q's candidate is (0, 257), inside 0 … 511 but not a legal cell, so it counts as not explored
    whatever the switch.

  Both wrenches are drawn near the top edge of the map; under the test shroud sprite no cover can
  hide them. Three runs, each needing one valid run: case A, case B, and case B again with the stock
  shroud sprite. In case A no unit of the human player is near (511, 255). In case B a second
  infantry unit of the human player starts beside (511, 255) (map data), so that cell is explored.
  The two cases make both states of (511, 255) likely, but the expected result follows the state
  observed in the run: the reveal of the unit at the corner might itself reach (511, 255) (Open
  questions). A run is void if the original does not create both buildings, the computer player
  does not start repairing both of them, or the view cannot be scrolled to show both wrench places.
  - Observable: whether P and Q show the repair wrench; whether (511, 255) is explored (its diamond
    under the test shroud sprite); and, from the stock-sprite run, the piece of cell (511, 255) and
    of the corner cell (1, 258) in a screenshot.
  - Expected: in every run, P shows the wrench exactly when (511, 255) is explored, and Q never
    shows it. In the
    stock-sprite run, (511, 255) and (1, 258) each have the piece that claim 2 gives when their
    neighbours that are not legal count as covered (claim 1); the neighbours of (511, 255) that
    count include the three neighbours with *x* = 512, beyond the coordinate range.
  - Falsified if: P's wrench disagrees with the observed state of (511, 255) (no wrench while that
    cell is explored is what the switch turned off would give); Q shows the wrench in any run; or
    either corner cell
    shows no edge towards a neighbour outside the map.

## Open questions

- Several glow images over one area are applied one after another in a fixed order (the order the
  light sources were created, low confidence). Because each step truncates, the order can change a
  value by one step; S-0104 should say which order it uses.
- Which cells stop being seen when the unit walks away (SH2) is the visibility specs' to say; the
  original may keep some cells seen after the units that saw them have left. The pieces follow the
  seen states whatever they are, so SH2 reads the fogged cells from the screenshot itself.
- Whether the tactical radar draws the shroud with the same pieces is not covered; it belongs to the
  interface specs.
- Whether a unit's reveal near the corner of a map with W + H = 512 explores cells across the
  coordinate wrap of the Edge cases (for example (511, 255) from a unit near (1, 258)) belongs to
  the visibility specs. SH8 reads the state of (511, 255) from the screen, so its result does not
  depend on the answer.

## Questions for the project lead

- S-0104 says in its Terms and in claim 15 that light-alpha values other than 127 come only from
  light-glow images. Claim 5 here shows that the shroud and fog pieces set them too, through the same
  shading. Either S-0104 gets a revision of that sentence (its claim 15 formula is unchanged), or
  this spec is read as extending it; the project lead decides. Claim 6 also answers S-0104's open
  question of how light-glow images write the values on the battlefield view, after the covers.
  Whether glows are written the same way anywhere else is S-0104's to say.
