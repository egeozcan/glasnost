---
id: S-0110
title: Shroud and fog rendering
topic: rendering
revision: 8
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-14
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
belongs to S-0409; this spec reads those states. Skirmish is the scope: fog of war is always off
there (S-0409 claim 19; S-0412 claim 1). Conditional fog rules below document deferred behaviour
for later modes, not a reachable skirmish setting; network games and campaigns are out of scope.

## Terms

*Cell*, *legal cell*, *top corner*, *height level*, *world pixel*, *battlefield view*, *drawn
frame*, *image frame*, *sprite*, *shroud*, *fog of war*, *local player*, *house*,
*measurement setup*, *recorder* and *quirk switch* are used as in the glossary. The projection, the
object height lift and the terrain height lift (15 pixels per height level, S-0101 claim 4) are those
of S-0101. *Shade row* and the brightness mapping are those of S-0104 (claim 15). For battlefield
rendering, *light-alpha value* means a per-screen-pixel integer from 0 to 255, initially 127
in each drawn frame, modified by cover pieces and light-glow images as specified here.
A *light-glow image* has S-0104's meaning; claim 6 here owns its battlefield pixel updates.
S-0104's Terms and claim 15 agree with these cover and glow updates and own the shade-row
formula. A cover can change the value without any glow. The *repair wrench* is that of
S-0111 (claim 19).

| Term | Meaning |
|---|---|
| explored cell | A cell whose shroud the local player has removed. S-0409 owns when a cell becomes explored and when it is covered again. |
| seen cell | A cell with S-0409's seen state for the local player. With fog on this means not under fog; in skirmish it is bookkeeping only. S-0409 owns its changes. |
| map neighbours | The eight cells around cell (*x*, *y*), named by glossary map directions: the side neighbours N = (*x*, *y* − 1), E = (*x* + 1, *y*), S = (*x*, *y* + 1), W = (*x* − 1, *y*), and the corner neighbours NW = (*x* − 1, *y* − 1), NE = (*x* + 1, *y* − 1), SE = (*x* + 1, *y* + 1), SW = (*x* − 1, *y* + 1). On the screen N is up-right, E down-right, S down-left, W up-left, NW straight up, NE right, SE straight down and SW left. |
| covered neighbour | For the shroud: a map neighbour that is not explored. For the fog: a map neighbour that is not seen. A map neighbour that is not a legal cell counts as covered for shroud unless the shared explored state of non-legal cells is explored for the local player's house (S-0409, claim 14). It always counts as covered for fog, since a non-legal cell is never seen. |
| cover piece | An image-frame index from 0 to 46 chosen per cell for one cover. 0 designates an uncovered cell with no covered neighbours (explored for shroud, seen for fog); 15 designates a fully covered state. These are state labels, not promises about the imported artwork: even piece 0 can change pixels under claim 5. |
| shroud cover, fog cover | The two covers drawn for each cell (claims 1 and 4). |
| shroud sprite, fog sprite | The two sprites the pieces come from (claim 3; Import mapping gives their file names). |
| canvas | The 60 × 30 pixel screen rectangle in which the pieces of one cell may act (claim 7, F2). |
| shrouded point | A world point that the point test of claim 8 places under the shroud. |
| candidate cells | The one cell for even *u*, or the two cells for odd *u*, whose explored states claim 8 tests. *u* is the point's count of whole height levels, not a radar column. |
| gap generator | A gap-generating object as defined by S-0409; SH1 uses a stock building of that kind. |
| cell's diamond | The 60 × 30 pixel diamond inscribed in a cell's canvas, with its top corner at the middle of the canvas's top edge: the screen area of the cell's ground at height 0. |
| test shroud sprite | The diamond-only named test file of the Measurement scenarios that replaces the shroud sprite in SH1's capture sub-test B, SH5, SH6 and SH8, and supplies SH7's state-reading reference. Other scenarios explicitly define their own variants. |

## Behaviour

1. **Shroud piece of a cell.** In every drawn frame, every legal cell whose canvas meets the
   battlefield view gets a shroud piece, chosen from the explored states of the cell and its map
   neighbours at the time of that drawn frame:
   - a cell that is not explored gets piece 15 (fully covered),
   - an explored cell gets the piece of claim 2 for its set of covered neighbours.

   A map neighbour that is not a legal cell (S-0009 claim 1) counts as covered unless the shared
   explored state of non-legal cells is explored for the local player's house (S-0409, claim 14).
   A non-legal cell is never seen. The shared explored state is an exception to the usual rule
   that non-legal cells are not explored; it can become explored through the gap-field behaviour
   of S-0409 claim 14, under its `quirks.offmap_cells_shared` (default original). This applies
   whether the neighbour's coordinates lie inside or outside the range the map size spans.
   The piece depends only on the current explored and seen states, not on the cell's history: a cell that is covered again
   after being explored gets piece 15 like a cell that was never explored. The result is the same
   as if the whole battlefield view were redrawn from the current states in every drawn frame; an
   implementation may redraw less, as long as what is shown stays the same (claim 9).
   - Backing: `measurable` (scenario SH1 with its re-cover and border sub-tests; SH8 for a
     neighbour beyond the coordinate range; SH8 part Q2 for the shared-state exception)
   - Confidence: high for ordinary legal cells; medium for non-legal neighbours and the
     re-covered case; low for the shared-state exception

2. **Piece for a set of covered neighbours.** The piece depends only on the *reduced set*: the
   covered side neighbours, together with each covered corner neighbour neither of whose two
   adjacent side neighbours (N and W for NW, N and E for NE, S and E for SE, S and W for SW) is
   covered. For a reduced set:
   - when it holds side neighbours only, the piece is 1·[N] + 2·[E] + 4·[S] + 8·[W], where [X] is 1
     if X is in the set and 0 otherwise; so the empty set gives 0 and all four sides give 15,
   - otherwise the piece is the entry of table T1 (Constants).

   The sum uses exact integer arithmetic, with no rounding. The 256 possible sets reduce to
   47 distinct sets, and the 47 piece indices are all different; this does not imply different
   artwork if the user's sprite reuses images.
   - Backing: `measurable` for neighbour sets reached in SH1; `unmeasured` for the exhaustive
     256-set mapping, since SH1 does not arrange every set and no stock-data setup for independently
     setting each neighbour's state is identified. The reduced-set count follows mathematically
     from the reduction rule.
   - Confidence: high

3. **Which sprite.** With fog of war off, shroud pieces are image frames of the shroud sprite. With
   fog of war on, both covers take their pieces from the fog sprite, the shroud cover included.
   - Backing: `measurable` for fog-off sprite selection (SH1 and SH2); `unmeasured` for fog-on
     selection, since no skirmish reaches fog on and later modes are out of scope.
   - Confidence: high

4. **Fog piece of a cell.** With fog of war on, every cell that gets a shroud piece in a drawn frame
   (claim 1) also gets a fog piece, chosen exactly as in claims 1 and 2 but from the seen states: a
   cell that is not seen gets 15, a seen cell gets the piece for its set of neighbours that are not
   seen. Like the shroud piece it depends only on the current states, so a cell that stops being
   seen gets 15. The fog piece is not drawn once the local player's house has been defeated; then
   only the shroud cover remains.
   - Backing: `unmeasured`: every part requires fog on, which no skirmish reaches; later modes
     are out of scope. SH2 checks that no fog piece appears in skirmish, not this conditional rule.
   - Confidence: high for the piece; medium for the defeated case

5. **Pieces set light-alpha values.** A piece is not painted onto the screen. Its pixels change the
   light-alpha values *a* of the screen pixels under it. Before any cover or glow acts in a drawn
   frame the value is 127; earlier drawn frames do not leave darkening behind. Drawing that is shaded by light-alpha values
   through S-0104 claim 15 uses the resulting value, so ordinary cover art darkens it: the terrain tiles and
   the objects shaded that way. Drawing that does not read light-alpha values is not darkened by the
   cover and keeps its palette colours under shroud and fog; S-0111 claim 2 names such drawing among
   the overlay images (health segments and pips). Which other kinds of drawing read the values
   belongs to the specs of that drawing. Each pixel of the piece's image frame, with palette index
   *v*, turns *a* into *a*′:
   - **shroud piece:** *a*′ = *v* for every *v* except 254; index 254 leaves *a* unchanged,
   - **fog piece**, acting on the value the same cell's shroud piece produced: *a*′ = max(0, *a* +
     *v* − 127) for *v* < 128; indices of 128 and above leave *a* unchanged.

   So a shroud pixel of index 0 makes the pixels below it black (shade row 0), index 127 leaves them
   at the default light-alpha value by setting it to 127, and a fog pixel of index *v* darkens by
   127 − *v* steps when *v* < 128. Pixels
   outside the image frame's own rectangle are not touched. (Formulas F1.)
   - Backing: `measurable` for shroud pixel updates, untouched pixels outside the image rectangle,
     and terrain/vehicle shading (SH3, including its rectangle sub-test); SH1 capture sub-test B
     tests restoration of the initial value where newly explored cells' pieces become transparent,
     `unmeasured` for fog pixel updates, since fog on is unreachable in skirmish. Which other
     drawing reads the values is a claim of its own drawing spec, not of SH3.
   - Confidence: high for the shroud rule, the fog formula and terrain shading,
     medium for vehicle shading

6. **Light-glow images over the covers.** The light-glow images act on the values that the pieces
   of all cells produced in the same drawn frame: each pixel *g* of a glow image turns *a* into
   *a*′ = min(255, ⌊*a*·*g* / 127⌋). Every pixel of the rectangle of the glow image's image frame
   that lies in the battlefield view takes part; no index is skipped, so a glow image is expected to use 127 wherever
   it should leave the light unchanged. A glow therefore scales the covered value and cannot lighten
   a pixel whose shroud value is 0: that pixel stays black under a lamp. A non-zero covered value
   can be brightened, including one produced by fog.
   - Backing: `measurable` for the formula, image rectangle and action over shroud (SH4),
     `unmeasured` for action over fog, since fog on is unreachable in skirmish. The order among
     several glows is not fixed here (Open questions).
   - Confidence: high

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
   not explored unless the shared explored state of non-legal cells is explored for the local
   player's house (S-0409, claim 14; claim 1 here). A candidate with a coordinate outside 0 … 511 is read as another cell, or as no
   cell (Edge cases, `quirks.shroud_point_cell_wrap`). For odd *u* the candidates are adjacent
   along the map south-east diagonal, and either being explored makes the point uncovered. Which
   rules use this test belongs to their own specs; S-0111's repair wrench uses it, and scenarios SH6
   and SH8 observe the test through it.

   For a radar pixel, S-0410 F3 first supplies the cell shown at that pixel; it does not supply
   these candidate cells. S-0410 claim 6 applies this point test at that cell's centre, at its
   surface height (S-0106 claims 2 and 3). Thus (*c*<sub>x</sub>, *c*<sub>y</sub>) here is the
   cell from F3, and *u* comes from that centre's height in leptons, not necessarily its terrain
   height level. S-0410 calls this count *z* and uses *u* for a radar column. Its radar scenarios
   R3 and R4 own the pixel mapping and its shroud consequence.
   - Backing: `measurable` for cell-centre heights used by SH6 and SH8 (including SH8 Q2),
     `unmeasured` for arbitrary off-centre points and fractional height levels, since these
     wrench scenarios test only cell centres at flat whole-level heights and no controllable
     observable for the remaining inputs is identified.
   - Confidence: high for the formula; medium for non-legal candidate cells and coordinate wrap,
     low for the shared-state exception. Heights below 0 are not covered by this claim; S-0111 claim 19 states the
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
   - Backing: `measurable` for explored-state updates (SH1 drawn-frame capture sub-tests),
     `unmeasured` for seen-state updates of fog pieces, since fog on is unreachable in skirmish
   - Confidence: medium

10. **Neighbouring canvases.** The canvases of cells (*x*, *y*) and (*x* + 1, *y*), and of (*x*, *y*)
    and (*x*, *y* + 1), overlap in a 30 × 15 pixel rectangle (F2); canvases of cells whose *x* + *y*
    differ by 0 or by 2 or more do not overlap. Where two canvases overlap, the result depends on the shroud
    pixel of the cell with the larger *x* + *y* (the lower one on the screen): wherever it is not
    254, it sets the value, whatever the upper cell's shroud and fog pieces produced, and the lower
    cell's fog pixel acts on that value; where it is 254, the lower cell's fog pixel acts on the
    upper cell's result (Observable order in State and ordering). So a fogged cell's darkening does not show
    where a lower neighbour's shroud piece has pixels other than 254. The overlap rule can be
    distinguished with named test sprites whose active pixels extend outside the cell's diamond.
    - Backing: `measurable` for canvas geometry and shroud replacement/transparent preservation
      (SH7); `unmeasured` for interleaved fog composition, since fog on is unreachable in skirmish
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
the shade row min(*N* − 1, ⌊*a*·ρ·(*N* − 1) / 32258⌋) of S-0104 claim 15, with ρ and *N* as defined there.

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
| Clear-neighbour piece | 0 | image frame | claims 1 and 2; state label, not guaranteed transparency |
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

**Observable order.** For one screen pixel, let *C*<sub>1</sub>, *C*<sub>2</sub> name the legal cells
whose canvases contain it (at most two, claim 10), with *x* + *y* of *C*<sub>2</sub> larger when
both exist. With only one cell, name it *C*<sub>1</sub>; outside all legal canvases neither exists. Drawing at that
pixel that reads light-alpha values (claim 5) is shaded with the light-alpha value

> *a* = (*G*<sub>*k*</sub> ∘ … ∘ *G*<sub>1</sub> ∘ *F*<sub>2</sub> ∘ *S*<sub>2</sub> ∘ *F*<sub>1</sub> ∘ *S*<sub>1</sub>)(*a*<sub>0</sub>)

where *a*<sub>0</sub> = 127 is the pixel's initial light-alpha value in that drawn frame, *S*<sub>*i*</sub> and
*F*<sub>*i*</sub> are the shroud and fog rules of F1 for the pixel of *C*<sub>*i*</sub>'s shroud and
fog piece there, and *G*<sub>1</sub> … *G*<sub>*k*</sub> the glow rule of F1 for each light-glow
image over the pixel (their order is an open question). A rule for a nonexistent cell, or whose
piece does not reach the pixel or is not drawn, leaves the value unchanged. Consequences: a fog piece darkens on top of the shroud
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
- **Non-legal neighbours count as covered unless their shared state is explored** (claim 1,
  S-0409 claim 14). Every cell on the border of the map's diamond of legal cells has map
  neighbours that are not legal, so when it is explored it gets edge pieces towards them unless
  their shared explored state is explored for the local player's house. A hostile gap field
  can make that state explored when it stops while the local player's house has an active spy
  satellite and no other hostile field covers a non-legal cell. Later gap changes can cover
  the shared state again. Whole-map reveal and re-cover do not reset it directly, but their
  gap-field removal and restoration participate in its changes as S-0409 claim 21 specifies.
  In particular, exploration persists through a re-cover with no hostile field covering
  non-legal cells; it is not immune to later gap changes. This is governed by
  `quirks.offmap_cells_shared` (owned by S-0409, default on, original behaviour).
  Off: non-legal cells always count as unexplored. Claims 1 and 8 read the same shared
  state; SH8 part Q2 tests both consequences. Whether those border cells lie inside the local area
  (S-0009), and so can be seen, depends on the map. This is the original's behaviour and our engine
  reproduces it; there is no additional switch beyond S-0409's shared-state switch.
- **Cell coordinates outside 0 … 511 in the point test** (claim 8). The original reads a candidate
  cell (*x*, *y*) of the point test as the cell (*x* mod 512, *y* + ⌊*x* / 512⌋) when
  *y* + ⌊*x* / 512⌋ lies in 0 … 511, and otherwise as the shared off-map state (normally
  not explored; S-0409 owns the exception). All values are
  integers; *x* mod 512 lies in 0 … 511, and ⌊ ⌋ rounds toward −∞. So a
  candidate with −512 ≤ *x* < 0 is read as (*x* + 512, *y* − 1), and one with
  512 ≤ *x* < 1024 as (*x* − 512, *y* + 1); the general formula applies beyond those ranges. This can read a legal cell only on maps whose legal cells reach coordinates near 511
  (W + H close to 512, S-0009 claim 2), for points high above cells near the map's edge.
  `quirks.shroud_point_cell_wrap` (default: on, the original). Off: a candidate with a coordinate
  outside 0 … 511 counts as not explored even if the shared off-map state is explored; this
  rejection takes precedence over `quirks.offmap_cells_shared`. The map neighbours of claim 1 never read
  a legal cell this way: legal cells have coordinates 1 … 511, and a neighbour with a coordinate of
  0 or 512 is read as a cell that is not legal, or as no cell; such a result reads the shared
  off-map explored state of S-0409. Example: on a map of size
  258 × 254 (W + H = 512), a point 4 height levels above the centre of the corner cell (1, 258)
  has the candidate (−1, 256), which is read as the legal cell (511, 255) at the opposite end of
  the map. Confidence: medium. Scenario SH8.
- **Overlapping pieces** (claim 10). The lower cell's (larger *x* + *y*) shroud piece wins even over
  the upper cell's fog darkening. This is the original's composition rule, not a quirk, and our engine reproduces it.
  With sprites whose active pixels stay inside their own cell's diamond, neighbouring pieces
  do not change the same pixel and the order has no visible effect.
- **Re-covered cells.** A cell that becomes unexplored again (for example under an enemy gap
  generator) or stops being seen gets piece 15 for that cover from the next drawn frame, and its
  neighbours get edge pieces towards it, as for any other covered cell (claims 1, 4 and 9).

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| file `SHROUD.SHP` (resolved by S-0005) | shroud sprite, image frames 0 … 46 (claim 3) | sprite | no shroud pieces drawn; the importer warns |
| file `FOG.SHP` (resolved by S-0005) | fog sprite, image frames 0 … 46 (claims 3, 4) | sprite | no pieces drawn with fog of war on; the importer warns |
| fog-of-war setting (S-0409 claim 19; S-0412 claim 1) | conditional rules of claims 3 and 4 | boolean | always off in skirmish, regardless of lobby defaults or map data |

A missing sprite is bad data: the original draws no cover from it. Our engine does the same and the
importer warns. An image frame index that the sprite does not have draws nothing.
- Backing: `measurable` (SH9 for the missing shroud sprite and unavailable frame),
  `unmeasured` for the missing fog sprite, since fog on is unreachable in skirmish. The warning
  is an importer requirement, not an observation of the original.
- Confidence: medium

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*, a tiny purpose-made map of one theater with flat
ground except where stated, one human player and one bystander computer player far away unless a
scenario states otherwise, no lighting effects (ambient neutral), game speed index 2, 800×600.
Director and mods are off; named test data below are the only overrides and are removed afterwards.
A run is void if an unnamed opponent object enters the observed area or acts on a test object.
Fog is off in every run; SH2 also tests that attempts to enable it do not survive skirmish start.
Screenshots are compared with the selected shroud sprite's image frames and S-0104 shade rows.
Use whole-drawn-frame capture for changing-state tests and screenshots for stationary cases.
Wrench cases require repair to remain active throughout the observations; completed repair voids
that observation, not the shroud test. Record the retry count and every unreached case.

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
  - **Border sub-test.** No spy satellite is active and no earlier satellite-assisted gap removal
    has occurred, so the shared off-map state remains unexplored (SH8 Q2 tests the exception).
    The unit walks to a cell on the border of the map's diamond (S-0009 claim 1)
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
- **SH2 (claim 3, skirmish scope).** Repeat SH1's main test and walk away from the explored area.
  Two runs: first with a named test rules file setting the lobby fog default on, then with a test
  map setting its fog flag as well (the imported settings of S-0409 and S-0412). Wait at least
  twice the imported fog period after leaving. This is also S-0409 scenario V12.
  - Observable: shroud edge pieces and the explored ground left behind, in screenshots.
  - Expected: shroud uses the shroud sprite, and explored ground is not darkened by fog in either
    run; starting the skirmish has forced fog off.
  - Falsified if: shroud uses the fog sprite or fog darkening appears. These runs cannot test the
    deferred fog-on rules of claims 3 to 5 or the fog-on defeat gate of claim 4.
- **SH3 (claim 5).** Fog off. A neutral-house stock vehicle is placed on flat explored ground,
  with no orders. Capture a reference at its fixed facing. Named test shroud sprite: every piece
  has the same 60 × 30 image, index *v* throughout the cell's diamond and 254 outside it. Make
  one run per *v* = 0 … 255, with the same vehicle and ground; no glow. Runs in which the vehicle
  moves or changes facing are void. A control run uses 254 everywhere.
  - Observable: terrain and vehicle pixel colours, matched to the reference's palette indices
    and S-0104 shade rows. Choose pixels well inside the diamond; ambiguous colour matches are
    excluded, not treated as confirmation.
  - Expected: *v* ≠ 254 sets *a* = *v*, including index 0; *v* = 254 preserves the reference
    shading. Compare each value's capture with a fully transparent reference of the same scene. Each chosen pixel has the capped shade row of F1 and S-0104 claim 15. Pixels outside
    the image rectangle preserve their reference shading.
  - Falsified if: a chosen pixel has a colour inconsistent with that shade row, index 254 changes
    it, or the effect extends beyond the image rectangle. If palette quantisation makes two
    candidate rows identical, that pixel does not distinguish them; record that limitation.
    The fog formula is not tested, since skirmish cannot enable fog.
  - **Rectangle sub-test.** A separate named sprite gives every piece a 10 × 6 pixel image
    rectangle centred within the 60 × 30 canvas, uniformly index 64; the remainder of the canvas
    is outside that image rectangle, not transparent pixels within it. Compare with a fully
    transparent reference of the same stationary scene. Choose terrain pixels immediately inside
    and outside all four image edges, away from other cells' active rectangles. One valid run.
    Expected: inside pixels use *a* = 64; outside pixels keep their reference shading.
    Falsified if: any unambiguous inside pixel fails that shade-row check, or any outside pixel
    changes. Record the image rectangle's placement with the capture.
- **SH4 (claim 6).** Fog off. Place one stock glow-casting building so its glow reaches into
  explored and unexplored ground. Named test files replace its light-glow image with a rectangle
  divided into bands of values *g* = 0, 64, 127, 254 and 255, and replace every shroud piece with
  SH3's diamond image. One run per shroud value *v* = 0, 64 and 127. No second glow overlaps it.
  The glow rectangle must cross both canvas halves and extend outside the building's own cell.
  A reference capture for each *v* has the glow absent (named test map suppresses that image
  only); a second reference uses the fully transparent shroud sprite as well, to identify the
  terrain palette indices before covering.
  - Observable: shade rows at known image-band pixels and at pixels just outside its rectangle,
    using both references and S-0104; record ambiguous colour matches as inconclusive.
  - Expected: inside the whole glow rectangle the row follows F1 with
    *a*′ = min(255, ⌊*v*·*g*/127⌋), not a glow followed by shroud replacement. No index is skipped:
    *g* = 0 produces black, 127 preserves the covered value, and 254/255 scale it, with saturation.
    *v* = 0 stays black for all bands. Outside the rectangle the reference shading remains.
  - Falsified if: any unambiguous chosen pixel disagrees with the capped shade row, an image band
    fails to act in a canvas half or outside the source cell, or pixels beyond the rectangle change.
    Multiple-glow order and the deferred fog consequence are not tested.
- **SH5 (claim 7).** With the diamond-only test shroud sprite, a map with a plateau 4 height
  levels high next to flat ground; the player's
  unit explores the flat ground only, up to the foot of the plateau.
  - Observable: the screen position of the dark diamond of each unexplored plateau cell relative to
    the cell's drawn terrain, and whether a unit later walking onto an explored plateau cell is
    darkened.
  - Expected: the cover of each unexplored plateau cell appears 60 pixels (4 × 15, the terrain
    height lift of S-0101 claim 4) below its drawn terrain, on the screen position the cell would
    have at height 0. A unit on the plateau is darkened exactly where the pixels it is drawn on lie
    under a cover piece, whatever the state of the cell it stands on.
  - Falsified if: the displacement differs from 60 pixels, the cover follows the plateau's
    terrain, or the unit's shading disagrees with the cover at its drawn pixels.
  - Canvas-clip sub-test: a named test shroud sprite gives every frame a 90 × 60 image extending
    beyond its 60 × 30 canvas, with a distinctive dark band outside the canvas and index 254
    inside. Expected: no dark band is applied; a screenshot matches the transparent-sprite
    reference. Falsified if: any outside-band darkening appears. One valid run.
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

  The expected result follows the states actually recorded, not the intended movement. A case
  is valid only when its recorded explored states reach the corresponding row below; extra
  reveals that erase that distinction void the case. In every valid case the building's own
  cell (*c*) is explored, so the building is drawn whatever S-0111 decides for objects in
  unexplored cells. Each row must be reached, including both single-candidate rows at 3 levels,
  an unreached row stays unmeasured after the five-attempt limit.
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
- **SH7 (claim 10).** Fog off, flat map. Named test shroud sprite: frames 0 … 46 fill the entire
  60 × 30 rectangle with 127 except frame 15, which uses 64. A human infantry unit reveals an
  area; screenshot its boundary. Read explored states separately using the diamond-only test
  shroud sprite on an otherwise identical map and stationary units; a changed boundary voids
  the comparison. Choose overlaps where the upper and lower cells have different cover values.
  - Observable: terrain shades in the 30 × 15 overlap rectangles, matched against S-0104.
  - Expected: every overlap takes the lower cell's value, not the upper cell's; same-row canvases
    never overlap. Thus each cell's upper 60 × 15 half uniformly takes its value (64 or 127).
  - Transparency sub-test: repeat with the upper 15 rows of every test image changed to 254,
    leaving its lower half as before. Expected: each overlap now preserves the upper cell's
    lower-half value. One valid run for each sprite variant is required.
  - Falsified if: a selected overlap shows the upper value in the first run, the lower value
    in the transparency run, another shade in either, or the overlap size differs from 30 × 15.
    This tests shroud-only ordering; fog interleaving remains unmeasured.
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
  - **Q (non-legal candidate).** Q's candidate is (0, 257), inside 0 … 511 but not a legal cell,
    so it counts as not explored
    whatever the wrap switch, before the shared-state sub-test Q2.

  Both wrenches are drawn near the top edge of the map; under the test shroud sprite no cover can
  hide them. Three runs, each needing one valid run: case A, case B, and case B again with the stock
  shroud sprite. In case A no unit of the human player is near (511, 255). In case B a second
  infantry unit of the human player starts beside (511, 255) (map data), so that cell is explored.
  The two cases make both states of (511, 255) likely, but the expected result follows the state
  observed in the run. S-0409 claim 5 rejects non-legal sight targets, so the point-test wrap
  does not itself reveal (511, 255). Cases A and B must record different explored states there,
  otherwise the wrap discrimination is inconclusive. A run is void if the original does not create both buildings, the computer player
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
    shows no edge towards a neighbour outside the map, before the shared-state exception below.
  - **Q2 (shared off-map state; claims 1 and 8; S-0409 V8 part H).** Repeat the Q observation
    while the human player owns an active stock spy satellite and sufficient power. A hostile
    stock gap generator has its field crossing the map border, with no other hostile gap field
    covering non-legal cells; destroy its power supply so the field stops. Q must still be under
    repair during the before/after captures; confirm repair by increasing health, and void a run
    where it has ended. Before the field stops, explored border cells away from the field have
    edge pieces towards non-legal neighbours and Q has no wrench. After it stops, with
    `quirks.offmap_cells_shared` on, those edges disappear and Q shows its wrench: its candidate
    (0, 257) now reads the shared explored state. A purpose-made map trigger then performs a
    whole-map re-cover (S-0011; S-0409 claim 15); keep the infantry beside Q so its own cell is
    explored again, and repeat the capture. No hostile field may cover non-legal cells during
    this re-cover. Expected: both effects persist in this field-free case. Then restore the
    generator's power so that the same hostile border field starts again, with no intervening
    gap changes. Expected: the border edges return and Q's wrench disappears, even though
    Q's own cell stays explored. Whole-map operations with running border fields follow
    S-0409 claim 21 and are tested by its V18, not by this field-free re-cover.
    With the switch off in our engine, the edges remain and Q has no wrench throughout.
    One valid run; falsified if either effect disagrees at any of these captures.
    Use a stock-sprite capture for edge pieces and the test shroud sprite for the wrench.
    Changing the sprite between captures requires a fresh run with the same preconditions.
- **SH9 (Import mapping, missing shroud data).** Named test archives reproduce the mounted stock
  contents except that the shroud sprite is omitted; ensure no other mounted copy supplies it
  (S-0005). On SH1's map, capture before and after the infantry explores. Control: the same map
  with the stock sprite restored. A second named test shroud sprite has only frames 0 … 14,
  each a recognisable dark diamond; screenshot an unexplored area that requires piece 15.
  - Observable: terrain darkening in screenshots; one valid run per case.
  - Expected: no shroud darkening in the absent-file case, and no piece-15 darkening in the
    short-sprite case, while available frames still darken explored boundary cells. Restoring
    the stock sprite restores the ordinary cover.
  - Falsified if: a missing file or unavailable frame instead draws cover or prevents the game
    from reaching the battlefield view. Fog data absence is not tested in skirmish.

## Open questions

- Multiple-glow order is not specified. Creation order is a low-confidence hypothesis,
  `unmeasured` here: SH4 uses a single glow, and no discriminating multi-glow capture is arranged.
  Per-image truncation and saturation can make the order visible, so commutativity must not be
  assumed. S-0104 and this spec need a common ordering rule after a distinguishing measurement.
- Which cells stop being seen with fog on belongs to S-0409's deferred fog rule (claim 19).
  The conditional rendering rules follow seen states whatever they are, but no current scenario
  observes them in skirmish.
- The radar's shroud display is owned by S-0410 claim 6 and F3: it uses claim 8's point test,
  not these cover pieces.
- The exhaustive piece table and arbitrary-point inputs of claim 8 remain unmeasured in the
  parts their backings name. Measurements must not report SH1 or SH6 as confirming those parts.

## Cross-spec consistency

- S-0409 owns explored/seen states, the off-map shared-state exception and fog being off in
  skirmish. Claims 1 and 8 here read that exception; they do not override it. Persistence
  follows S-0409 claim 21, including changes caused by whole-map gap-field removal and
  restoration; later gap changes can restore covered border pieces and shrouded point results.
  Its sight reveal
  tests legality before acting, so the point-test wrap does not imply wrapped sight reveals.
- S-0101 owns projection and the 15-pixel terrain lift; S-0106 owns centre surface height.
  SH6 and SH8 use flat cells so their centre heights really are whole levels. S-0111 claim 19
  supplies the wrench observable and retains the negative-height extension outside claim 8.
  S-0410 claim 6 and F3 supply radar's initial cell, not the candidate cells of this spec.
- Claims 5 and 6 here own battlefield cover and glow updates; S-0104's Terms and claim 15
  agree and own the shade-row formula, including its final cap. This is not an additional
  lighting channel or a departure from the original. Claim 6 supplies the battlefield glow
  update, not an assertion about other displays. Sign-off must retain that agreement.
- S-0011 claims 26 and 36 apply their trigger reveals to every player house, human or AI,
  consistently with S-0409 claim 1 and the glossary's AI-house ruling. Rendering here reads
  only the local player's house's resulting states; it adds no human-only reveal restriction.
- S-0414's original skirmish ends at the human player's defeat, unlike our engine's play-to-last-side
  departure. No scenario here requires the original to play on after defeat. The fog-on defeat
  rule remains unmeasured and deferred, rather than borrowing our engine's end condition.
