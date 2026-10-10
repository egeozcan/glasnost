---
id: S-0010
title: Theater data, tilesets and the terrain values of a cell
topic: formats
revision: 8
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: [Q-0009, Q-0018]
supersedes: []
---

# S-0010: Theater data, tilesets and the terrain values of a cell

## Summary

How the original reads a theater's tile data (its files, the declaration of tilesets and tile
numbers, tile image files and their variants, and the named tile roles), and what a cell takes from
its tile (slope shape, unusable tiles, the automatic smoothing of ground transitions and ramps, the
choice of a tile variant, and what the tileset flags do in play), and where the picture of a tile
cell and its extra image are drawn on the screen (claims 22 to 24). Not covered here: a cell's land
type, including the effect of overlays and neighbouring cliffs (S-0211), the byte layout of tile
files (public), the meaning of the slope shapes (S-0106), the colour that a drawn pixel takes from
the cell's brightness and the palette (S-0104, S-0103), and how tile images overlap each other
(S-0102).

## Terms

*Theater*, *tileset*, *sub-tile*, *cell*, *top corner*, *height level*, *slope*, *overlay*,
*overlay data value*, *archive*, *resolve*, *loose file*, *rules database*, *game start*, *session*,
*quirk switch*, *departure*, *importer warning*, *unspecified*, *unsynchronised random stream* (with
*draw*), *battlefield view*, *world pixel* and *measurement setup* are used as in the glossary.
*Slope shape* and *surface height* are defined by S-0106. *View pixel* and *view origin* are
S-0101's (claim 5 and its formula F1). The *land
type* rules of a cell (its *land recompute*, the *cliff-back setting*, the *own-land-type flag*) are
S-0211's. *Visiting
order*, *blank cell*, *no-tile marker* (65535), *unset base tile*, the *playable area* (claim 4)
and the *terrain stream* are S-0009's. *Terrain smoothing* is S-0009's name for ground and ramp smoothing (claims 10 and 11); S-0009
defines it as part of every terrain recalculation, at map load (claim 12) and in play, and
this spec owns its rules. The terms below are specific to this spec.

- **Theater data file.** The data file that declares one theater's tile roles and tilesets (claim
  1). Its sections are the *general section* and one *tileset section* per tileset.
- **Tile.** One numbered terrain piece of a theater. A tile covers a rectangle of *w* × *h* cells of
  which some may be empty. Each non-empty cell of a tile is a **tile cell**. A tile cell has an image,
  a *terrain code* (an integer the tile file stores, 0 … 15 in valid data), a *slope code* (the
  slope shape of S-0106, 0 … 20 in valid data) and two radar colours. The tile file format is
  public; this spec says only what these values mean.
- **Tile number.** A tile's index in the theater, counted from 0 across all tilesets in declaration
  order (glossary: tileset). A tileset's *first tile* is the number of its first tile.
- **Variant.** An alternative image file for a tile, found by a letter suffix (claim 3). A tile with
  *n* variants has *n* + 1 *images*, numbered 0 (the plain file) to *n*. All images of a tile share its
  tile number.
- **Role.** A named job that the general section gives to one tileset, such as "the clear tileset"
  or "the cliff tileset" (claim 4). A role's *start* is the first tile of the tileset it names, or
  the *unset* value when it names none.
- **Role range.** The tile numbers *start* … *start* + *n* − 1 that the original treats as having a
  role, for a fixed count *n* per role (Constants, table R).
- **Land type.** As in the glossary (S-0211's Terms): the terrain category of a cell, one of clear, road, water,
  rock, wall, tiberium, beach, rough, ice, railroad, tunnel and weeds. How a cell gets its land type
  from its tile, overlay and neighbours is S-0211's (claims 5 to 8); its movement and building
  effects belong to the movement and building specs.
- **Terrain recalculation.** The step in which the original derives a cell's slope shape and land
  type from its tile, sub-tile and overlay, and smooths its tile (claims 6 to 12, and S-0211 claims 5
  to 8 for the land type). S-0211 calls the same event the *land recompute* and owns the list of
  its triggers (S-0211 claim 5). It runs for every cell twice when a map is loaded (claim 12), and
  again for one cell on each trigger in play: an overlay is placed, removed or changed there, its
  tile, sub-tile or ground level changes (for example a destroyable cliff or a bridge is destroyed
  or repaired), or an object that stands on the ground (an infantry unit, a vehicle, an aircraft on
  the ground, a building or a terrain object) is put into the cell or taken out of it. The last
  trigger includes every ordinary move of a unit from one cell to the next, so every cell a unit
  enters or leaves is recalculated. A whole-map change of the terrain at game start recalculates
  every cell again. S-0211 claim 5 holds the authoritative list.
- **Transition pair.** A pair of roles, a *ground tileset* and a *transition tileset* of 16 tiles,
  used by ground smoothing (claim 10). The four pairs are rough, sand, green and pave.
- **Edge neighbours.** Of cell (x, y): N = (x, y − 1), E = (x + 1, y), S = (x, y + 1),
  W = (x − 1, y), in the map directions of the glossary.
- **General test, checked role.** A role's *general test* is the original's test of whether a tile
  has that role, by the role's range of table R (claim 5). A role is *checked* when its general test
  passes no tile while the role is unset; the test of an *unchecked* role still uses its range,
  counted from the unset start (claim 5, table R).
- **Stray tile number.** A tile number at or beyond the theater's tile count other than the no-tile
  marker (claim 7). A cell holds one in two ways. (a) A cell of the overlay case (below) keeps it at
  every recalculation for as long as it stays of the overlay case: a map can put one there, and
  smoothing can write one there (claim 10). (b) Smoothing can write one into any other cell
  (claims 4 and 10); that cell loses it at its next recalculation, which replaces it by the
  no-tile marker (claim 7). A *negative* tile number is not a stray number: the original's result
  for it is unspecified (claim 7), and claims 15 and 16 give its test separately.
- **Overlay case, no-tile case, tile case.** The three cases into which a cell falls at a terrain
  recalculation, decided by what it holds as the recalculation begins (claim 8, whose table defines
  them).
- **Diamond.** The picture of one tile cell on the ground: the isometric shape 60 pixels wide and 29
  rows tall of claim 22, filled with 900 palette-index pixels from the tile file.
- **Extra image.** The optional rectangle of pixels that a tile cell may carry in addition to its
  diamond, for parts of the terrain that rise above the ground, such as a cliff face (claim 23).
- **Depth data.** The per-pixel depth values that a tile cell's file may carry for its diamond and its
  extra image (S-0102's *tile depth data*).

## Behaviour

### Theater data

1. **Files of a theater.** For a theater the original reads one theater data file and takes its
   tile images from files with the theater's tile suffix (claim 3). Both are fixed per theater
   (Constants, table T). The theater data file is resolved like any data file (S-0005).
   - Backing: `measurable` (scenario F1)
   - Confidence: high

2. **Tileset declaration and tile numbers.** The tileset sections are named with a fixed prefix and
   a four-digit, zero-padded number counted from 0 (Import mapping). The declared tilesets are those
   numbered from 0 up to, but not including, the first number whose section has no tile-count field
   or gives it as −1 (an explicit −1 also ends the list). A section with a higher number has no
   effect, even if it exists. A tileset
   with tile count *C* contributes *C* consecutive tile numbers; a count of 0 is allowed and
   contributes none, and so does a negative count other than −1 (bad data: the importer warns). So
   tile numbers run from 0 to the sum of all counts minus 1, in declaration
   order, and variants never take a number of their own. The authored count of S-0009 (claim 8)
   changes only how map tile numbers are translated, never this numbering.
   - Backing: `measurable` (scenario F1: the numbering by its first part, the end of the list and
     the counts of 0 and below −1 by its end-of-list part)
   - Confidence: high

3. **Tile image files and variants.** The image of tile *k* (1 … *C*, counted within its tileset)
   is the file named by the tileset's file-name field, *k* in decimal with at least two digits
   (zero-padded), a variant letter (none for the plain image), a dot and the theater's tile suffix,
   all written together with no separator before the number or the letter ("NAME07b.TEM"). The
   plain image has no variant letter; variant 1 has the letter "a", variant 2 "b", and so on. The
   file is found like any data file (S-0005), which matches names without regard to letter case, so
   the letter case of the file name, of the variant letter and of the suffix does not matter. A tile's variants are these letters in order, up to but not
   including the first letter for which no file is found, so a tile's variants are always
   consecutive letters. When a file is not found with the theater's own suffix, and the tileset's
   non-height-view field is not 0, the original uses the first file found among those with the
   theater's fallback suffix, the fixed suffix of theater 0 and the fixed suffix of theater 2, in
   that order of preference (table T). The fallback applies to each image file in the same way, the plain one and every variant, so a
   variant found only under a fallback suffix counts as found. A tile whose plain image is found nowhere still takes its tile
   number, but has no image and no tile cells (claim 7). The number is written with at least two
   digits, so tile 100 of a set is written with three. A variant with a gap is not seen: if the
   files "a" and "c" exist but "b" does not, the tile has one variant ("a") and the file "c" is
   never used.
   - Backing: `measurable` (scenario F1; the fallback, its order of preference, its use for a
     variant and the gap rule need named test files)
   - Confidence: high for the naming and the variant order; medium for the fallback, its order of
     preference and the condition on the non-height-view field

4. **Roles.** The general section names a tileset number for each role of table R (Import
   mapping). A role's start is the first tile of that tileset. A role whose key is absent, or
   names a negative tileset number (bad data: the importer warns; a negative number names no
   tileset), or names a tileset number beyond the end of the declaration of claim 2, is unset: its start is −1,
   except the destroyable-cliff role, whose start is −2 when unset. One number past the last
   declared tileset is not unset: a role that names exactly the number of the first missing tileset
   section gets the theater's tile count as its start (`quirks.role_one_past_end`). Such a start
   lies outside the tile list, so a tile that smoothing derives from it (claim 10) is a stray tile
   number (Terms): a cell that is not of the overlay case loses it at its next recalculation (claim 7),
   an overlay-case cell keeps it (claim 8), and the picture of a cell that holds one is unspecified
   (claim 24). For the theater with index 5 the original forces seven roles
   to unset whatever the file says: bridge, cliff, shore, water, water bridge, water cliff and
   wooden bridge. The clear and water roles also give the fill base tiles of S-0009 (claim 6).
   - Backing, by part:

     | Part | Backing |
     |---|---|
     | A set role starts at the first tile of the tileset it names | `measurable` (scenarios L1, R1 part A) |
     | An absent key, a negative number, or a tileset number beyond the first missing one, leaves the role unset (start −1) | `measurable` (scenario R1 parts B, B2 and C) |
     | A role naming the first missing tileset number starts at the tile count | `measurable` (scenario R1 part D, through the four arms, which become no-tile cells; the centre's land type is checked only where the stock land-type data allow, as that part says) |
     | Theater 5 forces the shore and water-bridge roles to unset | `measurable` (scenario R2: part A for the shore role, part B for the water-bridge role) |
     | Theater 5 forces the water role to unset | `measurable` through S-0009 claim 6 and its scenarios, which own the water fill |
     | Theater 5 forces the cliff, water cliff, bridge and wooden bridge roles to unset | `unmeasured` here: this spec uses none of these roles, and their effects belong to specs not yet written |
     | The destroyable-cliff role's unset start is −2 | `measurable` (S-0214's scenario T7, its case with the destroyable-cliff key absent): that role's test accepts its start and the tile after it (table R), so an unset start of −1 would make tile 0 a destroyable-cliff tile, while −2 makes no usable tile one |

   - Confidence: high

5. **Role ranges.** The original treats a tile as having a role when its number lies in the role's
   range of table R, a fixed count of tile numbers from the role's start. While a role is unset,
   the general test of a role marked *checked* in table R passes no tile, and the test of an
   unchecked role still uses its range, counted from the unset start of claim 4: with the unset
   start −1 the range covers tile numbers 0 … *n* − 2 (`quirks.unset_role_range`). In particular, in the
   theater with index 5, whose water and shore roles are forced unset (claim 4), tiles 0 … 12 pass
   the water test and tiles 0 … 40 the shore test. Which game rules use each test belongs to the
   specs of those rules (bridges, naval placement, cliff destruction and so on). This spec uses
   these general tests only for the tunnel entrances of claim 21. Ground and ramp smoothing
   (claims 10 and 11) use ranges with the same counts, except the paved-road range, which has 21
   tiles there (table R), but treat an unset role differently: there its range holds only the tile
   number −1.
   - Backing, by part:

     | Part | Backing |
     |---|---|
     | The ranges that ground and ramp smoothing use: the shore (42 tiles), water-bridge (2), miscellaneous-pave (14), paved-road (21, in the pave test) and median (14) ranges of the neighbour tests, and the ramp (20) and smooth-ramp (12) ranges | `measurable`: the last tile of each range and the tile after it are tested in scenario B1 (parts A to C); scenarios L1, L2, L3, R1 and R2 test the roles' starts and their unset behaviour |
     | The general tests, their unset rule and the counts that only they use (the paved-road count 15 included) | `unmeasured` here, because the rules that use them, the tunnel rules of claim 21 included, belong to specs not yet written |
   - Confidence: high for the counts of the smoothing ranges; medium for the counts of the general
     tests and for which game rules use them

### What a cell takes from its tile

6. **Slope shape from the tile, and the tile cell used.** At terrain recalculation, a cell with a
   usable tile (claim 7) takes the slope shape of the tile cell given by its sub-tile. Its land type
   comes from the tile cell's terrain code, by the table of S-0211 claim 6, subject to the overlay
   and cliff-back rules of S-0211 claims 5 and 8. The tile cell is always that of the tile's plain
   image, whichever image the cell shows (claim 13): the terrain and slope codes of a variant's file
   are never read for the land type or the slope shape, and neither is the variant file for the
   validity test of claim 7. The theater data file's radar-colour fields are not read; the radar
   colours come from the tile cells (their use belongs to the interface specs).
   - Backing: `measurable` (scenario V3 for the plain image, with a named test file; the slope
     shape taken from a tile through S-0211's scenario L3, which places ore on every slope shape,
     and the land type through S-0211's scenarios L5 and L11)
   - Confidence: high for the plain image and the slope shape; the land type is S-0211's

7. **Cells whose tile is not usable.** A cell's tile is *unusable* when its tile number is at or
   beyond the theater's tile count (the no-tile marker included), its tile has no image (claim 3),
   its sub-tile is at or beyond the tile's cell count *w* × *h*, or the tile has no tile cell at
   that sub-tile. After a terrain recalculation, a cell of sub-tile *t* is as follows. *Row 1 of
   S-0211 claim 5* means a cell that holds an overlay whose land type is wall or railroad, or whose
   own-land-type flag is set.

   | The cell as the recalculation begins | Tile and sub-tile | Slope shape |
   |---|---|---|
   | has an overlay of row 1 of S-0211 claim 5, whatever its tile | unchanged, even a tile number at or beyond the tile count (smoothing of claims 10 and 11 may then change the tile, claim 8) | when the tile number is from 0 to the tile count − 1, that of its tile's tile cell at sub-tile *t* mod (*w* × *h*), or flat (0) if the tile has no tile cell there (a tile with no image has no tile cells at all, so its shape is flat); otherwise unchanged |
   | has no such overlay, and an unusable tile | the no-tile marker, sub-tile 0 | flat (0) |
   | has no such overlay, and a usable tile | unchanged (smoothing of claims 10 and 11 may then change the tile) | that of its tile cell at sub-tile *t* |

   The land type of each of these cells is S-0211 claim 5's: row 1 for the first case, rows 2 and 3
   for the second and rows 4 and 5 for the third (the same assignment as the table of claim 8). In
   particular, a cell that holds the no-tile marker is flat and plays as clear unless it holds an
   overlay, and a cell whose tile was dropped here holds the no-tile marker from then on, so its next
   recalculation applies S-0211 claim 5 row 2. This settles S-0009's open question for numbers at
   or beyond the tile count: such a cell plays as flat ground (clear unless it holds an overlay).
   A negative tile number (S-0009 claim 8) gives an unspecified result in the original; our engine
   treats it like the no-tile marker (the departure of S-0009 claim 8), and the importer warns. A
   terrain code outside 0 … 15 is a departure owned by S-0211 claim 6.
   - Backing, by part:

     | Part | Backing |
     |---|---|
     | The no-tile case (tile number at or beyond the tile count, sub-tile beyond the cells, empty tile cell) | `measurable` (scenario C1) |
     | Smoothing of the overlay case | `measurable` (scenario C2 part A) |
     | The overlay case keeps a tile number at or beyond the tile count, and takes its slope shape from *t* mod (*w* × *h*) | `unmeasured`: the overlay hides the ground shape too much to read, and the picture of a cell that holds a tile number at or beyond the tile count is unspecified (claim 24), so the tile number cannot be read from the screen either |
   - Confidence: high (medium for the overlay case)

8. **Which cells are smoothed, and which start animations and tunnel entrances.** What an overlay
   does to a cell's land type, and the removal of ore and gems from steep slopes, are S-0211's
   (claims 5 and 7); this spec does not repeat them. For the terrain, every cell falls into one of
   three cases, decided by what it holds as the recalculation begins:

   | Case | The cell as the recalculation begins | Tile and slope shape | Tile smoothed (claims 10 and 11) | May start a tile animation (claim 19) | May create a tunnel entrance (claim 21) |
   |---|---|---|---|---|---|
   | **overlay case** | holds an overlay of row 1 of S-0211 claim 5 (claim 7) | as claim 7 says | yes | no | no |
   | **no-tile case** | not the overlay case, and its tile is unusable (claim 7, and rows 2 and 3 of S-0211 claim 5) | the no-tile marker; flat | no | no | no |
   | **tile case** | every other cell: a usable tile and no row-1 overlay (rows 4 and 5 of S-0211 claim 5) | slope shape from its tile (claim 6) | yes | yes | yes |

   S-0211 claim 7 may remove the overlay of a tile-case cell in the same recalculation; that changes
   none of the above.
   - Backing: the smoothing of the tile case is `measurable` (scenario L1) and that of the overlay
     case is `measurable` (scenario C2 part A); the overlay case's lack of animations is
     `measurable` (scenario C2 part C, on the animated tile of scenario G4 part A); the overlay
     case's lack of tunnel entrances is `unmeasured`, for the reason of claim 21 (its effect is
     defined by tunnel rules that no spec covers yet, and the entrance is the only thing it
     changes), and the no-tile case's lack of animations follows from there being no tile
     to name one
   - Confidence: high for the tile case and the no-tile case; medium for the overlay case
     (low for its lack of tunnel entrances)

9. **Cells behind cliffs.** The cliff-back rule (the rules data's cliff-back setting, the six
   neighbour cells, the four-level threshold and the switch `quirks.cliff_back_case_sets`) is
   S-0211 claim 8. This spec adds nothing to it and states no behaviour of its own here; the claim
   exists so that claims of other specs that cite S-0010 claim 9 keep their number.
   - Backing: `measurable`, through S-0211 claim 8 and its scenario L5, which own the rule
   - Confidence: high; the confidence of the rule itself is S-0211 claim 8's

### Smoothing

10. **Ground smoothing.** At terrain recalculation, a cell's tile is
    smoothed for each transition pair in turn: rough, sand, green, pave. Each pair works on the tile
    that the previous pair left, so the order shows only where a tile written for one pair is also
    the ground tile or lies in the transition range of a later pair (roles that name overlapping
    tilesets, or the low tile numbers that an unset rough transition role writes, below). For a
    pair with ground tileset start *G* and transition tileset start *L*, a cell whose tile is *G*, or lies in *L* … *L* + 15,
    belongs to the pair. Its tile becomes *G* when all four edge neighbours belong to the pair, and
    *L* + *m* otherwise, where *m* = [N out] + 2·[E out] + 4·[S out] + 8·[W out] and "out" means
    the neighbour does not belong to the pair. Two pairs widen what counts as belonging, for the
    neighbour test only: for the green pair, a neighbour whose tile is in the shore range (42
    tiles from the shore start) or the water-bridge range (2 tiles) counts as belonging; for the pave
    pair, a neighbour in the miscellaneous-pave range (14 tiles), the paved-road range (21 tiles)
    or the median range (14 tiles) counts as belonging. In this smoothing an unset role's range
    holds only the tile number −1 (unlike the general tests of claim 5). The sand, green and pave pairs are skipped when their transition role is unset; the
    rough pair is not, so with its transition role unset its ground cells are still smoothed, and a
    ground cell at the edge of a patch gets the tile −1 + *m*, one of the tile numbers 0 … 14 of
    whatever tilesets come first (`quirks.unset_role_range`). Likewise, when a pair's transition
    role is set but its ground role is unset, a transition cell whose four neighbours all belong
    gets the tile −1, which is unspecified from then on (S-0009 claim 8). In the unset-rough case,
    provided the rough ground tile *G* is not one of the tile numbers 0 … 14, a rewritten cell no
    longer belongs to the pair, so it counts as out for its neighbours: within one
    pass, a ground cell visited after a rewritten neighbour is rewritten too. A cell's N and W
    neighbours are always visited before it (S-0009 claim 3), so the first pass of claim 12 rewrites
    every rough ground cell of the map (at the map's edge too, since a neighbour outside the map
    counts as out, below), and later recalculations, at load or in play, find none left
    to rewrite. When the rough transition role names the first missing tileset (claim 4), the
    transition tiles it writes lie at or beyond the tile count (stray tile numbers, Terms), so
    each such cell that is not of the overlay case becomes the no-tile marker at its next
    recalculation (claim 7): at map load the second pass, and in play whatever triggers the cell's
    next recalculation (Terms), the first unit that enters or leaves it included. A cell of the
    overlay case keeps the number (claim 8) and, being smoothed, may take another number of the
    same kind; it holds one until the overlay is gone and the cell is recalculated. Until
    then the picture of such a cell is unspecified (claim 24). After map load this concerns the
    cells to which the second pass itself gave such a number, such as the centre of scenario R1
    part D, and the cells of the overlay case. Apart from these unset-role cases, a cell that belongs to a pair still belongs to
    it after smoothing, so the result does not depend on the order in which cells are smoothed, and
    a further recalculation, by a unit's move or otherwise, leaves the tile unchanged. In play,
    therefore, a unit's moves change tiles by this claim only in the one-past-the-end case and in
    the case of a set transition role with an unset ground role. The cell's slope shape and land
    type are not re-read from the new tile in the same recalculation. Cells of the
    no-tile case (claim 8) are not smoothed at all (they hold the no-tile marker, which belongs to
    no pair and no ramp range). An edge neighbour outside the map reads as a blank cell (S-0009,
    claim 44): it holds the no-tile marker, so it belongs to no pair, counts as out for every pair
    (the widened neighbour tests included) and is flat for the ramp smoothing of claim 11.
    - Backing: `measurable` (scenario L1 for set roles; S-0009's scenario M25 part A for a
    neighbour outside the map; scenario R1 for an unset or one-past-the-end
      rough transition role, including the erosion within one pass, and its part E for a unit's
      move as the next recalculation; scenario R2 for the green
      pair's widened neighbour test and its dependence on the shore and water-bridge roles,
      scenario B1 parts A and B for the ends of the widened ranges of the green and pave pairs,
      and scenario R3 for the skipped sand, green and pave pairs). The case of a set transition role with an unset ground role is
      `unmeasured`, because the tile −1 it writes is unspecified in the original (claim 7)
    - Confidence: high

11. **Ramp smoothing.** A cell whose tile, as ground smoothing left it, lies in the ramp range
    (20 tiles from the ramp start) or the smooth-ramp range (12 tiles from the smooth-ramp start)
    gets a tile chosen anew from its slope shape *s* (the shape it took in this recalculation, from
    the tile it held when the recalculation began). For *s* = 1 … 4, two edge neighbours are
    tested, *a* with weight 1 and *b* with weight 2, as the table below gives; each counts as *flat*
    when its slope shape is 0, and *m* is the sum of the weights of the flat ones. For *s* = 0 or
    5 … 20, *m* = 0. The new tile is smooth-ramp start + 3(*s* − 1) + *m* − 1 when *m* > 0, and
    ramp start + *s* − 1 when *m* = 0 (Formulas). Because the range test applies to the tile that
    ground smoothing left, the order of the two smoothings shows only where ground smoothing writes
    a tile into a ramp range or out of one (roles that name overlapping tilesets, or the unset-role
    cases of claim 10).

    | *s* | *a* (weight 1) | *b* (weight 2) |
    |---|---|---|
    | 1 | W | E |
    | 2 | N | S |
    | 3 | E | W |
    | 4 | S | N |

    So a ramp tile always shows the ramp-range tile of its own shape, or, for edge slopes that meet
    flat ground at their low or high edge, one of three smoothed tiles. A ramp-range tile of shape 0
    becomes the tile just before the ramp range, a tile of another tileset (`quirks.ramp_shape_zero`).
    Unlike ground smoothing, the result depends on the neighbours' slope shapes at that moment (claim
    12).

    **Unset ramp roles.** As in ground smoothing (claim 10), an unset ramp or smooth-ramp role's
    range holds only the tile number −1, and the formulas use the unset start −1
    (`quirks.unset_role_range`):

    | Ramp role | Smooth-ramp role | Result |
    |---|---|---|
    | unset | unset | no usable tile is ramp-smoothed |
    | set | unset | a ramp-range tile of shape *s* = 1 … 4 with *m* > 0 becomes 3(*s* − 1) + *m* − 2, one of the tile numbers −1 … 10 of whatever tilesets come first (−1 for *s* = 1 and *m* = 1, which is unspecified from then on, claim 7); the other ramp-range tiles get ramp start + *s* − 1 as usual |
    | unset | set | a smooth-ramp-range tile with *m* = 0 (or of shape 0 or 5 … 20) becomes *s* − 2, one of the tile numbers −2 … 18 (a negative result is unspecified from then on, claim 7); with *m* > 0 it gets its smoothed tile as usual |
    - Backing: `measurable` (scenario L2 for set roles; scenario B1 part C for the ends of the two
      ranges; scenario L3 for an unset smooth-ramp or ramp role, for the results that are not
      negative; scenario L4 for a ramp-range tile of shape 0)
    - Confidence: high for the rule and for the unset roles; medium for the edge case of shape 0

12. **Order at map load.** When a map is loaded, terrain recalculation runs twice for every legal
    cell, each time in the visiting order of S-0009 (claim 3). The first pass works on the terrain
    stream, the map's tunnel definitions and the overlays as read (S-0009), and its results are
    already in place when the map's terrain objects and units are placed. The second pass works on
    the map with those objects placed, and its results are in place when the game starts. In the
    first pass a cell's slope shape is still
    flat (0) until the cell itself is recalculated, as for S-0009's blank cell, so ramp smoothing
    (claim 11) sees the N and W neighbours of a cell (already visited, one screen row higher) with
    their real shapes but the E and S neighbours (one screen row lower) as flat. The second pass
    sees every neighbour with the shape that the first pass gave it. Provided each smoothed ramp tile
    carries the same slope shape as the tile it replaced (as the theater data is meant to), the
    tiles after loading are therefore those of a smoothing that sees every neighbour's real shape,
    and the first pass's partial view leaves no trace in them. A cell's land type after loading is the one
    the second pass takes from the tile chosen by the first pass (claims 10 and 11), even where the second
    pass then picks another tile; it keeps that land type until the cell's next recalculation. A recalculation of a cell in play (Terms: among others, every
    unit's move into or out of the cell) sees its neighbours' current shapes.
    - Backing: `measurable` (scenario L2)
    - Confidence: medium (the two passes and their order are certain; that the smoothed ramp tiles
      keep their shapes in stock data is not checked)

### Variants

13. **Choice of the image.** A cell whose tile has *n* ≥ 1 variants shows image *i* = *P* mod
    (*n* + 1), where *P* comes from a pattern indexed by a position (*X*, *Y*):
    - with *n* + 1 ≤ 4 images, the fixed 4 × 4 pattern of table P4 at (*X* mod 4, *Y* mod 4),
    - with *n* + 1 > 4 images, the 8 × 8 pattern of claim 14 at (*X* mod 8, *Y* mod 8).

    For the cell of sub-tile 0, (*X*, *Y*) is the cell's own coordinates. For a cell of another
    sub-tile *t* of a tile whose plain image is *w* cells wide and *h* high (the sizes of the plain
    image, whichever image is shown), *X* = (x − (*t* mod *w*)) / *w* and
    *Y* = (y − ⌊*t* / *w*⌋) / *h*, divided with truncation toward zero, that is, the coordinates of
    the tile's origin cell divided by the tile's size. So for a tile larger than one cell, the
    origin cell and the other cells of the same placed tile can choose different images
    (`quirks.variant_origin_mismatch`). A tile cell that its tile file marks with a certain flag
    takes its image from a per-cell state instead (used by tiles that change in play; not covered:
    which game events set that state belongs to specs not yet written).
    The choice never reaches the cell's land type or slope shape, which come from the plain image
    (claim 6); as far as known it affects only which image is drawn.
    - Backing, by part: the choice for tiles of one cell is `measurable` (scenario V1 part A) and
      for larger tiles `measurable` (V1 part B); the image of a cell without a usable tile is
      `measurable` (scenario C1 part B); the per-cell-state exception is `unmeasured`, because the
      state it reads is set by rules that no spec covers yet
    - Confidence: high for tiles of one cell; medium for larger tiles; low for the per-cell-state
      exception

14. **The 8 × 8 pattern.** The original builds the 8 × 8 pattern the first time in a session that it
    chooses an image for any cell, and keeps it for the rest of the session. At map load it chooses
    an image for every cell whose tile has two or more images and for every cell without a usable
    tile (which shows the clear tile), so the pattern is built while the first such map of the
    session loads, whatever the number of images. The pattern's entry at (*X*, *Y*) has index
    *k* = *X* + 8*Y*, and the 64 entries are the function of the draws from the unsynchronised
    random stream that the formula *8 × 8 pattern* gives (Formulas). Consequences: every entry is
    one of 0 … 7; no entry holds the same value as any of its 8 neighbours in the index-wrapped
    sense of that formula, except where all 64 candidates for an entry clashed; building the
    pattern takes between 64 and 4096 consecutive draws from the unsynchronised stream, all at one
    point of the session; and the pattern depends on how many draws that stream has given before.
    - Backing: `measurable` for the pattern's properties (scenario V2); the draws themselves are
      `measurable` with a recorder that reads the unsynchronised stream (scenario V2 part B), an
      extension of the state recorder that S-0202 (random streams)
      describes, which the project lead has approved as part of the measurement setup
    - Confidence: high

### Tileset flags

15. **Tilesets with the morphable flag, and to-tile buildings.** A building type whose to-tile
    field names a tile (Import mapping) can be placed only on cells whose tile belongs to a tileset with the morphable
    flag. The test passes without looking at any tileset for a cell that holds the no-tile marker, for
    one that holds a stray tile number (Terms), and for one that holds a negative tile number (the
    original tests the number against the range 0 … tile count − 1 and passes any number outside
    it). The tile list is not consulted for such a cell, so the picture it shows (claim 24) does not
    matter to the test. Other conditions of placement are not covered here, nor is what a to-tile
    building does once placed. The original also uses the flag in its random map generator.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The tilesets, and the no-tile marker | `measurable` (scenario G1, parts A and B, with named test data) |
      | A stray tile number | `measurable` (scenario G1 part C, on the centre cell of scenario R1 part D, which holds one until it is recalculated) |
      | A stray tile number of an overlay-case cell | `unmeasured`: whether a building may stand on a cell that holds a row-1 overlay is decided by rules outside this spec, so the tile test cannot be told apart from them |
      | A negative tile number | `unmeasured`: a cell comes to hold one only through results that claims 7, 10 and 11 call unspecified or leave `unmeasured` (map data, the tile −1 of claim 10, the negative results of claim 11's unset-ramp table), so no state to test is practical |

    - Confidence: medium (low for the negative number)

16. **Where new tiberium can appear.** A cell can receive newly grown tiberium (an empty cell becoming
    a tiberium cell) only if, among other conditions that belong to the economy specs, its land type
    is buildable (the land type's buildable field, S-0006), it holds no overlay, its slope shape is 0,
    and its tile belongs to a tileset with the allow-tiberium flag. As in claim 15, the tileset test is
    passed without consulting any tileset by a cell that holds the no-tile marker, a stray tile
    number (Terms) or a negative tile number (the original tests the number against the range
    0 … tile count − 1 and passes any number outside it). A cell that holds an overlay fails the
    overlay condition whatever its tile, so a stray number matters here only for a cell that is not
    of the overlay case, that is, for one that smoothing has just written one to (claims 4 and 10).
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The tilesets, and the no-tile marker | `measurable` (scenario G2, parts A and B) |
      | A stray or negative tile number | `unmeasured`: such a cell exists only as a single cell (or the few cells that claim 10 writes one to) until its next recalculation, and the growth of new ore on one given cell is a rare random event that long runs cannot make observable |

    - Confidence: medium (the condition list is certain; that this test is the one ore growth uses is
      the economy specs' to confirm; low for the stray and negative numbers)

17. **Burrowing.** Inside the playable area of S-0009 (claim 4, tested with the *height-aware* test of that claim, in
    which the cell's height level shifts the edges of the area), a tunnelling unit
    can dig in or come up only on a cell whose tile belongs to a tileset with the allow-burrowing
    flag (absent: allowed), among other conditions: the cell is flat and free of buildings and of
    terrain objects. The same test answers *allowed* for every cell outside that area, whatever its tile; what
    that means in play is for the vehicle specs, which own the full rule.
    - Backing: `measurable` inside the area (scenario G3); the answer outside it is `unmeasured`,
      because a unit cannot be ordered to a cell there and what it would do is the vehicle specs'
    - Confidence: medium

18. **Shadow-casting tilesets.** A tileset casts shadows (S-0104 claim 13) only when both its
    shadow-caster flag is set and its shadow-tile count is not 0. With the flag unset the count has
    no effect; with it set, a count of 0 (or an absent count) disables the shadow (Import mapping).
    Which cells a shadow-casting tile marks is open in S-0104, and so are some limits on which tiles
    of such a tileset cast (Open questions).
    - Backing: `measurable` (scenario G5)
    - Confidence: high

19. **Tile animations.** A tile may name an animation, with a screen displacement (*dx*, *dy*) in
    pixels, a sub-tile it attaches to and a depth adjustment. These keys are not in the tileset
    section: they are read from a separate section of the theater data file whose name is the
    tileset's set name, and when the animation name is empty or absent the other four keys have no
    effect (Import mapping). Only a tile's plain image carries them; variants (claim 3) do not. A
    tile whose attaches-to key is absent never starts its animation. An animation type named there
    that the rules data does not list is created by that reference (S-0004).
    - **When.** At terrain recalculation, a cell of the tile case (claim 8) starts
      an animation when the tile it held as the recalculation began names one and the cell's
      sub-tile equals that tile's attaches-to value. The animation is that tile's, not that of a
      tile that smoothing chooses in the same recalculation (claims 10 and 11). From the second pass
      of claim 12 until the next map load, a cell starts at most one animation: once it has started
      one in that time, a later recalculation starts no second one, however the tile has changed
      since. A cell that has started none can start one at
      any later recalculation in which the tile it holds at the beginning qualifies, for example a
      tile that smoothing left in the cell at an earlier recalculation (scenario G4 part C). After a map load, the
      tile animations of this claim that remain are exactly those that the second pass of claim 12
      starts: at most one per cell, that of the tile the first pass left in the cell. None that the
      first pass started survives loading, even in a cell where the second pass starts none.
    - **Where.** The animation is placed at the world position (X, Y, *z*) of the formula *Tile
      animation position* (Formulas): the cell's centre, moved by the ground displacement that S-0101's
      screen-to-ground formula (F6) gives for the world pixel (*dx*, *dy*), at the height of the
      cell's level (level × 104 leptons). A displacement whose *dx* is at least the battlefield
      view's width in pixels, or whose *dy* is at least its height, moves the animation by nothing
      in the original, so it appears at the cell's centre at the cell's level. That size depends on
      the player's screen, so our engine always applies F6 (a departure for behaviour that depends
      on the host; no switch). Scenario G4 part D tests displacements far beyond any view; the exact
      boundary (at least the view's size, or more than it, and which size when the window is
      resized) is not measured, because the case is a departure that the engine does not reproduce.
    - **Depth.** The animation takes the tile's depth adjustment.
    - **Draws.** A tile animation is created with no loop delay, and its creation takes the draws
      that S-0202 (claim 10, *animation creation draws*) lists, at the moment it starts, during
      map loading too.

    These are S-0103's and S-0104's *terrain animations*; how they play belongs to S-0109 (animation
    playback).
    - Backing: `measurable` (scenario G4, parts A to D: part D for the displacement that is far
      larger than the view; the boundary of that case is `unmeasured`, for the reason above; the
      draws through S-0202's scenario R8, part F)
    - Confidence: high for when an animation starts at map load and for the position formula,
      medium for a start in play after smoothing (part C) and for the displacement far larger than
      the battlefield view (part D), and low for the exact boundary of that case

20. **Fields read without effect in play.** The tileset's allow-to-place, height-view and
    non-height-view fields (apart from claim 3) and its theater-conversion fields are read and have
    no effect on a game as far as found; the set name only names the tiles for the to-tile lookup of
    claim 15 and the section of the tile animations of claim 19; the required-for-random-maps flag
    only affects the random map generator. The general section's ice roles, train-bridge role and
    alternative cliff roles, and the tilesets' radar colours, are not read at all.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The allow-to-place and theater-conversion fields have no effect on building placement or on the pictures of tiles | `measurable` (scenario F1, spot-check part: changed for one tileset, nothing may differ) |
      | The height-view field and the non-height-view field's other uses, the set name's other uses, the random-map flag, and the keys that are not read at all | `unmeasured`: an absence of effect on every part of a game cannot be shown by a finite set of measurements, and no observable exists for the random map generator |
    - Confidence: high for the keys that are not read at all; medium for the fields that are read
      and have no effect, and for the set name and the random-map flag

21. **Tunnel entrances.** A cell of the tile case (claim 8) whose land type is
    tunnel, whose tile is one of the first four tiles of one of the four tunnel roles and which has
    no tunnel entrance yet makes the original create a tunnel entrance for that cell at terrain
    recalculation (a pointer: the tunnel rules have no spec yet). The tunnel roles are not
    checked (claim 5): an unset tunnel role covers tiles 0 … 2, so a tunnel-land cell holding one of
    them also creates an entrance (`quirks.unset_role_range`).
    - Backing: `unmeasured` here (the effect is defined by the tunnel rules)
    - Confidence: medium

### Drawing a tile

22. **The diamond of a cell.** A cell paints the diamond of at most one tile cell (which one, and when none: claim 24).
    The diamond is 60 pixels wide and 29 rows tall, and only the pixels of the shape in the formula
    *Diamond rows* (Formulas) exist: row 0 has 4 pixels, each row down to row 14 has 4 more, row 14
    has 60, and each row below has 4 fewer, down to 4 pixels in row 28. That is 900 pixels. The tile
    file supplies exactly these 900 palette indices, row by row from row 0, left to right within a
    row. Write (*u*, *v*) for the view pixel of the cell's top corner at height 0 (S-0101 claim 5,
    for the world position (256*x*, 256*y*, 0)) and ℓ for the cell's height level, a signed integer.
    The diamond pixel in column *c* (0 … 59) and row *r* (0 … 28) is painted at the view pixel
    (*u* − 30 + *c*, *v* − 15ℓ + *r*) (formula *Tile image placement*). The lift 15ℓ is S-0101
    claim 4's terrain lift and is the only effect of the level on the picture's place. Consequences:
    - Cells of the same level tile the plane without a gap or an overlap: the diamond of the cell
      to the east (its top corner 30 pixels right and 15 rows down) continues the shape exactly.
    - Row 14, the widest, lies 14 rows below the top corner's row (at the cell's level), and
      the diamond's last row is row 28, 28 rows below it. The row 29 that a 30-row cell box would
      have is never painted.
    - Every one of the 900 pixels is painted, palette index 0 included: a diamond has no transparent
      index. The colour each pixel takes from its index (palette, house remap where it applies and
      the cell's brightness) is S-0103's and S-0104's.
    - The tile file's own stored position and size for the cell do not place or shape the diamond.
      Every diamond is placed from its own cell's top corner alone, so the picture of a multi-cell
      tile on the screen comes from where its cells lie on the map, never from the file's layout of
      the cells.
    - The colour of a pixel (S-0103, S-0104) and the depth rule for its pixels (S-0102 claim 9 and
      its formula F3) are not repeated here; this claim fixes only where the pixels go and which of
      them exist.
    - Backing: `measurable` (scenario X1)
    - Confidence: high

23. **The extra image.** A tile cell whose file marks it as having an extra image also carries a
    rectangle of palette indices with its own width and height, and two stored positions: that of the
    tile cell and that of the extra image, both in the coordinates of the tile file's own picture.
    The extra image is drawn with the cell's lift and brightness, over its own cell's diamond, and
    in the order of cells it counts as part of its own cell: where depth does not decide, it covers
    the pictures of the cells painted before its cell and is covered by those of the cells painted
    after it (the order of cells is S-0102's, claim 16). Its top-left pixel lies at the top-left corner of the diamond's 60 × 29 box (column 0, row 0 of claim 22, itself not a painted pixel), (*u* − 30, *v* − 15ℓ), plus
    the difference (extra image position) − (tile cell position), taken per coordinate (formula
    *Tile image placement*). Only that difference matters, not the two positions themselves. It is
    usually negative vertically (the extra image rises above the diamond) and may have either sign
    horizontally; the extra image may reach outside the diamond's 60 columns on either side and below
    the diamond's last row. Palette index 0 is transparent in an extra image: those pixels are not
    painted, every other pixel is painted over whatever is already there, subject to depth. A tile
    cell without the mark paints no extra image.
    - **Depth.** When the tile cell has depth data, the depth data cover the extra image with values
      of their own. A pixel of the extra image takes the depth of S-0102's formula F3, with *e* the
      extra image's own depth value for that pixel and *t* the reference row of the cell's diamond,
      not the row where the extra image's own top lies. It is painted, and its depth stored, only
      where that depth is less than or equal to the stored depth, as for the diamond's pixels
      (S-0102 claim 9). A tile cell without depth data paints its extra image, like its diamond,
      without testing or storing depth. S-0102's claims speak of the tile image as a depth
      writer without naming the extra image; this claim states the rule for it.
    - **Variants.** The image a cell shows (claim 13) supplies its own diamond pixels, extra image
      mark, extra image, positions and depth data from its own file. A variant may therefore carry an
      extra image where the plain image has none, and the reverse. The land type and the slope shape
      still come from the plain image (claim 6).
    - How the images of neighbouring cells overlap, where depth data do not decide it, is S-0102's
      (claim 16 and its depth rules).
    - Backing: `measurable` (scenario X2, parts A to C; the depth rule in part D)
    - Confidence: high for the placement, the transparency and the variants; medium for the depth
      rule (the formula is S-0102's, the extra image's use of it follows from the original's single
      depth shift per tile cell)

24. **Which tile cell is painted, and when nothing is.** Which image a cell shows is claim 13's.
    Let *t* be the cell's sub-tile and *N*<sub>s</sub> the number of tile positions, width times
    height, of the *shown image*. A variant's file has its own size, so *N*<sub>s</sub> can differ from the
    plain image's *w* × *h*. The tile cell that is painted is counted row by row from the
    top-left of the shown image, empty positions included (claim 6).
    - **Tile case (claim 8).** The cell paints the tile cell at index *t* mod *N*<sub>s</sub> of the
      shown image. By claim 7, *t* is below the plain image's *w* × *h*, so the index is *t* itself
      when the shown image is the plain one or any variant with at least as many positions as the
      plain image; a variant with fewer positions wraps the index.
    - **No-tile case.** A cell holding the no-tile marker paints position 0 of the first tile of the
      clear role (S-0009 claim 8, "the theater's clear tile"). The image is the one that claim 13
      gives for that tile with the position (*X*, *Y*) = (x, y), the cell's own coordinates, as for
      sub-tile 0: the 4 × 4 pattern when the clear tile has at most 4 images, the 8 × 8 pattern
      when it has more. The choice of claim 13 is made for the cell even when the clear tile has one image, with no
      visible effect (it still builds the 8 × 8 pattern, claim 14). The cell's height level lifts the picture as claim 22 says.
    - **Overlay case (claim 8), tile number below the tile count.** The cell paints as in the tile
      case: the tile cell at index *t* mod *N*<sub>s</sub>, so a sub-tile at or beyond the cell count
      paints a tile cell of the tile, as the slope shape of claim 7 does.
    - **A stray tile number (Terms).** A cell holds one without being replaced by the no-tile marker
      when it is of the overlay case (a map can put such a number on a cell, and smoothing can write
      one there), or when smoothing has just written one (claims 4 and 10) into another cell that has
      not been recalculated since.
      No tile of that number exists, and the original's picture of such a cell is unspecified (it
      can paint anything or fail). Our engine paints it as for the no-tile case, a departure. The
      importer warns for a map cell that carries such a number together with an overlay of row 1 of
      S-0211 claim 5. A number that smoothing writes in play is not data, so nothing is warned. The
      cell's slope shape and land type are not affected (claims 7 and 8, S-0211).
    - **Nothing painted.** When the shown image has no tile cell at the index, the cell paints
      nothing at all: no diamond, no extra image, no depth. That happens (a) for a cell of the
      overlay case whose tile has no image (claim 3), and (b) when the index lands on an empty
      position of the shown image. For the plain image (b) can occur only in the overlay case,
      because claim 7 guarantees a tile cell in the tile case. For a variant it occurs in either
      case, when the variant's file has an empty position where the plain file has a cell
      (`quirks.variant_cell_missing`). The pixels then
      keep whatever the terrain picture held there, which for a first painting is unspecified; our
      engine leaves them black. The cell's land type and slope shape are those of the plain image
      (claim 6), so the cell plays as ground that is not drawn.
    - **The tile file's height value.** A tile cell's file stores a height value. It is not used when
      a tile is painted, and it does not set a cell's height level when a map is loaded or in play:
      a cell's level comes from the map file (S-0009) and changes only by the play events that
      change it. In the original the value has one use only: when the random map generator
      places a whole tile, it sets the level of each cell the tile covers from these values (the
      generator is not part of Glasnost's skirmish rules and is not specified).
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The tile cell index in the tile case with the plain image, and position 0 for the marker | `measurable` (scenarios X1, C1) |
      | The image of a no-tile cell is claim 13's choice for the clear tile | `measurable` (scenario C1 part B) |
      | A variant's empty position paints nothing | `measurable` (scenario X4 part A; named test file) |
      | A variant with fewer positions wraps the index | `measurable` (scenario X4 part B; named test files) |
      | The same index rule in the overlay case, and nothing painted there for a tile without an image | `unmeasured`: the overlay hides the ground, and a cell that paints nothing shows pixels that are unspecified on a first painting |
      | The height value is unused | `measurable` (scenario X3; named test file), the random map generator part `unmeasured` (not specified) |
      | The picture of a cell that holds a stray tile number | `unmeasured`: the original's result is unspecified, so there is nothing to compare |
    - Confidence: high for the tile case, the no-tile case and the unused height value in drawing.
      Medium for the wrap of a smaller variant, the empty position of a variant and the overlay case.

## Formulas

**Diamond rows (claim 22).** For row *r* = 0 … 28 of the diamond, the first painted column *f*(*r*) and the
number of painted pixels *n*(*r*), integers in pixels:

> *f*(*r*) = 28 − 2*r* and *n*(*r*) = 4(*r* + 1), for *r* ≤ 14
>
> *f*(*r*) = 2(*r* − 14) and *n*(*r*) = 4(29 − *r*), for *r* ≥ 14
>
> Exact integer arithmetic, no rounding.

Both lines give *f* = 0 and *n* = 60 at *r* = 14. The sum of *n*(*r*) over the 29 rows is 900. The
file's 900 pixels fill the rows in this order.

**Tile image placement (claims 22 and 23).** For a cell (*x*, *y*) at height level ℓ, with (*u*, *v*)
the view pixel of the world position (256*x*, 256*y*, 0) (S-0101 claim 5), all integers in pixels:

> diamond pixel (*c*, *r*) → view pixel (*u* − 30 + *c*, *v* − 15ℓ + *r*), for *c* from *f*(*r*) to *f*(*r*) + *n*(*r*) − 1
>
> extra image pixel (*i*, *j*) → view pixel (*u* − 30 + (*X*<sub>e</sub> − *X*<sub>c</sub>) + *i*, *v* − 15ℓ + (*Y*<sub>e</sub> − *Y*<sub>c</sub>) + *j*)

where (*X*<sub>c</sub>, *Y*<sub>c</sub>) and (*X*<sub>e</sub>, *Y*<sub>e</sub>) are the stored positions of the tile cell and of its extra
image, and (*i*, *j*) runs over the extra image's width and height from its top-left pixel. No rounding
enters: everything is an integer sum.

**Ground smoothing mask (claim 10).** For a cell in a transition pair with ground start *G* and
transition start *L*:

> *m* = [N out] + 2·[E out] + 4·[S out] + 8·[W out], where [P] is 1 if P holds and 0 otherwise.
> New tile = *G* if *m* = 0, else *L* + *m*. Integers, exact, no rounding. Unit: tile numbers.

**Ramp smoothing (claim 11).** For a ramp-range or smooth-ramp-range cell of slope shape *s*, with
*a* and *b* the neighbours of weight 1 and 2 in claim 11:

> *m* = [shape(*a*) = 0] + 2·[shape(*b*) = 0] for 1 ≤ *s* ≤ 4, otherwise *m* = 0.
> New tile = smooth-ramp start + 3(*s* − 1) + *m* − 1 if *m* > 0, else ramp start + *s* − 1.
> Integers, exact, no rounding. Unit: tile numbers.

**Variant (claim 13).** For a cell of sub-tile *t* of a tile *w* × *h* with *n* + 1 images:

> (*X*, *Y*) = (x, y) if *t* = 0, else (trunc((x − (*t* mod *w*)) / *w*), trunc((y − ⌊*t*/*w*⌋) / *h*))
>
> *P* = P4(*X* mod 4, *Y* mod 4), the entry of table P4 in column *X* mod 4 and row *Y* mod 4, if
> *n* + 1 ≤ 4; else *P* = P8<sub>*k*</sub>, the entry of the 8 × 8 pattern with
> *k* = (*X* mod 8) + 8·(*Y* mod 8)
>
> image = *P* mod (*n* + 1). Integers; trunc rounds toward zero; mod is the non-negative
> remainder. Unit: image index.

The origin coordinates x − (*t* mod *w*) and y − ⌊*t*/*w*⌋ can be negative for a multi-cell tile
placed so that its origin would lie beyond the map's low x or low y edge. The formula covers that
case as written: the division truncates toward zero, and *X* mod 4, *X* mod 8 (and the same for *Y*)
are non-negative remainders, as in the original.

**Tile animation position (claim 19).** For a cell (x, y) at height level ℓ whose tile has the
displacement (*dx*, *dy*) in pixels:

> X = 256·x + 128 + trunc₀( *a* · (*dx* + 2·*dy*) ),  Y = 256·y + 128 + trunc₀( *a* · (2·*dy* − *dx*) ),
> *z* = 104·ℓ

in leptons, where *a* and trunc₀ are those of S-0101's F6 (the screen-to-ground formula applied to
the world pixel (*dx*, *dy*), with no view origin added). The number domain and rounding are F6's:
the original computes the two products in single-precision floating point under the program-wide
round-toward-zero mode (glossary; its switch `quirks.chop_rounding` belongs to S-0205, number
domains), and F6 shows that the result equals the exact rational value truncated toward zero. In the original the two trunc₀ terms are
both 0 when *dx* ≥ the battlefield view's width or *dy* ≥ its height, in pixels (claim 19); our
engine does not reproduce that case.

**8 × 8 pattern (claim 14).** Let *D*<sub>1</sub>, *D*<sub>2</sub>, … be the consecutive draws
that the unsynchronised stream gives from the moment the pattern is built, and
*c*<sub>i</sub> = *D*<sub>i</sub> mod 8. The neighbours of entry *k* are the entries
(*k* + *d*) mod 64 for *d* in (−9, −8, −7, −1, 1, 7, 8, 9). With *u*<sub>0</sub> = 0, for
*k* = 0 … 63:

> *j*<sub>k</sub> = the smallest *j* in 1 … 64 such that *c*<sub>u<sub>k</sub> + j</sub> ≠ P8<sub>k′</sub>
> for every neighbour *k′* of *k* with *k′* < *k*, or 64 if there is none.
>
> P8<sub>k</sub> = *c*<sub>u<sub>k</sub> + j<sub>k</sub></sub>,  *u*<sub>k+1</sub> = *u*<sub>k</sub> + *j*<sub>k</sub>.
>
> Integers, exact, no rounding; mod is the non-negative remainder. Unit: image indices before the
> reduction of claim 13. The number of draws taken is *u*<sub>64</sub>, between 64 and 4096.

Neighbours with an index above *k* constrain nothing for entry *k*. The neighbours wrap across the pattern's edges like a torus, but along the index, not
along *X* and *Y* separately (so *k* = 7, at the end of a row, and *k* = 8, the start of the next
row, are neighbours, with *d* = 1 seen from *k* = 7).

## Constants

**T. Theater files** (claims 1 and 3). The theater index is that of S-0009's Import mapping. A
mapping from theater index to names; no unit.

| Theater index | Theater data file | Tile suffix | Fallback suffix |
|---|---|---|---|
| 0 | TEMPERATMD.INI | TEM | MMT |
| 1 | SNOWMD.INI | SNO | MMS |
| 2 | URBANMD.INI | URB | MMU |
| 3 | DESERTMD.INI | DES | MMD |
| 4 | URBANNMD.INI | UBN | MMT |
| 5 | LUNARMD.INI | LUN | MML |

After the fallback suffix, the original tries TEM, then URB (claim 3). These names are fixed in the
original's program (engine constants), not stock data.

**R. Roles and role ranges** (claims 4, 5, 10, 11). The columns answer two separate questions.
*Checked* is about the role's general test (claim 5): "yes" means the test passes no tile while the
role is unset, "no" means it still uses the role's range, counted from the unset start, and "—"
means that no general test of the role is known (the role has none that this spec uses).
*Pair skipped when unset* is about ground smoothing (claim 10) and applies to the four transition
roles only: "yes" means the whole pair is skipped while the transition role is unset, "no" means it
is not. An unset ground role never skips a pair (claim 10).

| Role | Range count *n* (tiles) | Checked | Pair skipped when unset | Used here |
|---|---|---|---|---|
| clear | — (start only) | — | — | S-0009 fill |
| water | 14 | no | — | — |
| ramp | 20 | no | — | claim 11 |
| smooth ramp | 12 | no | — | claim 11 |
| cliff | 40 | yes | — | — |
| shore | 42 | no | — | claim 10 |
| water bridge | 2 | no | — | claim 10 |
| miscellaneous pave | 14 | no | — | claim 10 |
| pave (ground) | 16 | no | — | — |
| paved road | 15 (21 in the pave smoothing test) | no | — | claim 10 |
| paved road end | 4 | no | — | — |
| paved road slope | 4 | no | — | — |
| median | 14 | no | — | claim 10 |
| dirt road junction | 11 | no | — | — |
| dirt road curve | 24 | no | — | — |
| dirt road straight | 66 | no | — | — |
| bridge | 16 | yes | — | — |
| wooden bridge | 16 | yes | — | — |
| sand transition | 16 | no | yes | claim 10 |
| green | the green ground start itself, or 16 from the green transition start | no | — | claim 10 |
| destroyable cliff | 2 (the start and the tile after it) | no | — | — |
| tunnel roles (4) | 4 each | no | — | claim 21 |
| rough, sand, green, pave ground | 1 (the start itself) | — | — | claim 10 |
| rough transition | 16 | — | no | claim 10 |
| green transition | 16 | see the green row | yes | claim 10 |
| pave transition | 16 | — | yes | claim 10 |

Note: in the pave smoothing test the miscellaneous-pave range is the 14 tiles *start* … *start* + 13
and the paved-road range the 21 tiles *start* … *start* + 20; the median range is 14 tiles. In the
rows whose *Checked* entry is "yes" or "no", the counts in the table's second column are those of
the original's general tile tests, which other specs will use. In the ground rows and the rough and
pave transition rows, whose *Checked* entry is "—", they are the counts that ground smoothing uses
(claim 10): so the pave ground role has a general test of 16 tiles (row "pave (ground)"), while
ground smoothing counts only its start (row "rough, sand, green, pave ground"). In ground and ramp smoothing (claims 10 and 11) every unset role's range holds
only the tile number −1, whatever the *Checked* column says.

**P4. Fixed variant pattern** (claim 13), by rows *Y* mod 4 = 0 … 3, columns *X* mod 4 = 0 … 3.
P4(*X*, *Y*) is the entry in column *X* and row *Y*. Entries are image indices before the
reduction of claim 13:

| | *X* 0 | 1 | 2 | 3 |
|---|---|---|---|---|
| *Y* 0 | 0 | 1 | 2 | 3 |
| *Y* 1 | 3 | 2 | 1 | 0 |
| *Y* 2 | 2 | 3 | 0 | 1 |
| *Y* 3 | 1 | 0 | 3 | 2 |

**Other constants.**

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Tileset section number width | 4 | decimal digits | Zero-padded, from 0 |
| Tile number within a set in file names and animation keys | at least 2 | decimal digits | From 1; tile 100 has 3 |
| Transition tiles | 16 | tiles | Per pair |
| Transition pairs | 4 | pairs | Rough, sand, green, pave, in that order |
| Variant patterns | 4 × 4 and 8 × 8 | entries | Claims 13 and 14 |
| Pattern draw attempts | 64 | draws per entry, at most | Claim 14 |
| Unset role start | −1 (destroyable cliff −2) | tile number | Claim 4 |
| Diamond size | 60 × 29 | pixels (columns × rows) | Claim 22; 900 pixels |
| Diamond left edge from the top corner | 30 | pixels left | Claim 22 |
| Terrain lift | 15 | pixels per height level | S-0101 claim 4 |
| Transparent index of an extra image | 0 | palette index | Claim 23; the diamond has none |

## State and ordering

- Terrain recalculation of one cell, as observable results: after a recalculation the cell's slope
  shape and land type are those of the tile it held when the recalculation began, after the
  replacements of claim 7 and with the overlay and cliff-back rules of S-0211 claims 5 to 8 applied. The
  tile that ground and ramp smoothing choose in the same recalculation (claims 10 and 11) gives its
  own slope shape and land type only from the cell's next recalculation. A tile animation started
  by the recalculation (claim 19) is likewise that of the tile the cell held when the
  recalculation began, not that of the smoothed tile.
- At map load (claim 12) every legal cell is recalculated twice, each time in the visiting order.
  Observable consequences:

  | Pass | Works on | Its results are in place when | Consequences |
  |---|---|---|---|
  | First | the terrain stream (S-0009 claim 7), the map's tunnel definitions (S-0009, read set) and the overlays | the map's terrain objects and units are placed | ore that a map puts on a cell of shape 5 or more is already gone then (S-0211 claim 7); ramp smoothing sees cells later in the visiting order as flat (claim 12) |
  | Second | the same, with the map's terrain objects and units placed | the game starts | ramp smoothing sees every neighbour's shape from the first pass, which corrects the first pass's view (claim 12, scenario L2); the land types and tile animations after loading are this pass's (claims 12 and 19) |
- The 8 × 8 variant pattern is built at most once per session (claim 14).

## Edge cases and quirks

All switches default to the original behaviour.

- **`quirks.unset_role_range`.** Original: in the general tile tests, an unchecked role that is
  unset still uses its range, counted from the unset start of claim 4, so with the unset start −1
  tiles 0 … *n* − 2 pass its test (claim 5); and in ground
  smoothing, an unset rough transition role still smooths rough ground cells, writing one of the
  tile numbers 0 … 14 into every rough ground cell in the first pass at map load (provided the
  rough ground tile is not one of those numbers, and subject to the map-edge proviso of claim 10; the rewriting
  spreads from the patch edges within that pass), and a pair with an unset ground role writes −1
  into fully surrounded transition cells (claim 10); in ramp smoothing, an unset ramp or smooth-ramp role's range holds the tile −1 and
  its start −1 enters the formulas, so smoothed ramp tiles become low tile numbers of whatever
  tilesets come first (claim 11); an unset tunnel role makes tiles 0 … 2 tunnel entrances
  (claim 21). Fixed: an unset role's range is empty, a transition pair with an unset ground or
  transition role is skipped, and ramp smoothing is skipped when either ramp role is unset.
- **`quirks.role_one_past_end`.** Original: a role naming the first missing tileset number starts at
  the theater's tile count (claim 4). Fixed: it is unset.
- **`quirks.ramp_shape_zero`.** Original: a ramp-range tile of slope shape 0 is replaced by the
  tile just before the ramp range (claim 11). Fixed: it keeps its tile.
- **`quirks.variant_origin_mismatch`.** Original: the origin cell of a multi-cell tile uses its
  own coordinates for the variant choice and the other cells use the origin divided by the tile's
  size (claim 13). Fixed: every cell of a placed tile uses the divided origin coordinates.
- **`quirks.variant_cell_missing`.** Original: a cell whose shown variant has no tile cell at the
  cell's index paints nothing (claim 24). Fixed: it paints the plain image's tile cell instead.
- **The land-type rules** (overlays, tiberium on slopes, cliff-back and the switch
  `quirks.cliff_back_case_sets`) are S-0211's (claims 5 to 8). This spec adds no switch for them.
- **The 8 × 8 pattern and the unsynchronised stream** (claim 14). The pattern depends on the
  unsynchronised stream's history in the session, so in the original two machines of a multiplayer
  game can show different ground variants. S-0202 (claim 11) has our
  engine run the unsynchronised stream on each machine from the game's seed, unsynchronised between
  machines; this spec only fixes where the draws are taken. The pattern reaches no game state,
  because a cell's land type and slope shape come from the tile's plain image whatever image it
  shows (claims 6 and 13). S-0202's claim 11 lists this use (at map load, before play) and records the
  departure that builds the pattern afresh at every map load.
- **A cell that holds a stray tile number** (Terms, claim 24). The original's picture
  of it is unspecified, and ours is the clear tile as for the no-tile case. A departure; the
  importer warns for map data only, as claim 24 says.
- **Bad data** (departures, with importer warnings): a negative tile number counts as the no-tile
  marker (claim 7); a terrain code outside 0 … 15 is S-0211 claim 6's departure; a slope code
  outside 0 … 20 counts as flat (S-0106 owns that departure).
- **Tile animation displacement as large as the view** (claim 19). In the original a displacement
  with *dx* or *dy* at least the battlefield view's width or height in pixels moves the animation by
  nothing. The view size depends on the player's screen, so our engine always applies the formula
  (a departure for behaviour that depends on the host, no switch).

## Import mapping

Theater data file (claim 1), general section `[General]`. Each key below names a tileset number
(integer, S-0003); absent: the role is unset (−1; the destroyable-cliff role −2).

| Key | Role |
|---|---|
| `ClearTile` | clear (also S-0009 fill) |
| `WaterSet` | water (also S-0009 fill) |
| `RampBase` | ramp |
| `RampSmooth` | smooth ramp |
| `RoughTile`, `ClearToRoughLat` | rough ground, rough transition |
| `SandTile`, `ClearToSandLat` | sand ground, sand transition |
| `GreenTile`, `ClearToGreenLat` | green ground, green transition |
| `PaveTile`, `ClearToPaveLat` | pave ground, pave transition |
| `MiscPaveTile`, `PavedRoads`, `PavedRoadEnds`, `PavedRoadSlopes`, `Medians` | miscellaneous pave, paved road, paved road end, paved road slope, median |
| `DirtRoadJunction`, `DirtRoadCurve`, `DirtRoadStraight`, `DirtRoadSlopes` | dirt road roles |
| `CliffSet`, `DestroyableCliffs`, `WaterCliffs`, `WaterCaves`, `CliffRamps` | cliff roles |
| `ShorePieces`, `WaterBridge` | shore, water bridge |
| `BridgeSet`, `WoodBridgeSet` | bridge, wooden bridge |
| `Tunnels`, `TrackTunnels`, `DirtTunnels`, `DirtTrackTunnels` | the four tunnel roles |
| `SlopeSetPieces`, `SlopeSetPieces2`, `MonorailSlopes`, `MMRampBase`, `HeightBase`, `BlackTile`, `RoughGround`, `Rocks`, `WaterfallEast`, `WaterfallWest`, `WaterfallNorth`, `WaterfallSouth` | further roles, read; their uses are not covered here |
| `BridgeTopLeft1`, `BridgeTopLeft2`, `BridgeTopRight1`, `BridgeTopRight2`, `BridgeBottomLeft1`, `BridgeBottomLeft2`, `BridgeBottomRight1`, `BridgeBottomRight2`, `BridgeMiddle1`, `BridgeMiddle2` | tile positions within the bridge tileset (integers, absent −1), for the bridge specs |

Tileset sections `[TileSetNNNN]` (NNNN = four-digit number from 0000):

| Key | Feeds | Form | Absent |
|---|---|---|---|
| `TilesInSet` | tile count *C* (claim 2) | integer | −1: ends the tileset list |
| `LastTilesInSet` | authored tile count (S-0009 claim 8) | integer | no translation |
| `FileName` | file-name stem (claim 3) | text, limit 63 bytes (a longer value is cut, S-0003 claim 1) | TILE |
| `SetName` | tile names for the to-tile lookup, and the name of the tile-animation section (claims 15, 19 and 20) | text, limit 63 bytes (a longer value is cut, S-0003 claim 1) | No Name |
| `MarbleMadness` | height-view field, read, no effect (claim 20) | integer | 65535 |
| `NonMarbleMadness` | non-height-view field: 0 disables the file fallback (claim 3) | integer | 65535 |
| `Morphable` | morphable (claim 15) | boolean | no |
| `AllowTiberium` | allow-tiberium (claim 16) | boolean | no |
| `AllowBurrowing` | allow-burrowing (claim 17) | boolean | yes |
| `ShadowCaster` | shadow caster (claim 18) | boolean | no |
| `ShadowTiles` | shadow-tile count; has an effect only when the shadow-caster flag is set (claim 18) | integer | 0 |
| `AllowToPlace` | read, no effect (claim 20) | boolean | yes |
| `RequiredForRMG` | random map generator only (claim 20) | boolean | no |
| `ToSnowTheater`, `ToTemperateTheater` (the theater-conversion keys) | read, no effect (claim 20) | integer | −1 |
| in the section named by the tileset's `SetName`, not in the tileset section: `TileNNAnim`, `TileNNXOffset`, `TileNNYOffset`, `TileNNAttachesTo`, `TileNNZAdjust` (NN = the tile's number within the set, from 01, written with at least two digits as in claim 3, so tile 100 is written 100; the animation name is text with a limit of 127 bytes) | tile animation: animation type name, *dx* and *dy* in pixels, attaches-to sub-tile, depth adjustment (claim 19) | string, integers | no animation; absent displacements 0, absent attaches-to −1 (the animation never starts), absent depth adjustment 0 |

Keys of other files that this spec uses:

| User data | Key | Feeds | Form | Absent |
|---|---|---|---|---|
| rules, building type | `ToTile` | to-tile field (claim 15): the tile whose name matches; a tile's name is the first 28 bytes of its tileset's set name, a space and its number within the set written with at least two digits (matching rule not established, Open questions) | string | none |
| rules, land-type sections | `Buildable` | the land type's buildable field (claim 16; S-0211 claim 2) | boolean | no |
| theater tile files | each tile cell's terrain code and slope code | claims 6 and 7 (the terrain code's meaning is S-0211 claim 6) | integers | — |
| theater tile files | each tile cell's 900 diamond pixels | palette indices, painted as claim 22 says | bytes | — |
| theater tile files | each tile cell's stored position, extra-image mark, extra-image position, width and height, and extra-image pixels | claim 23; only the difference of the two positions is used | integers, bytes | no extra image |
| theater tile files | each tile cell's depth-data mark and depth values (diamond and extra image) | claims 22, 23 and S-0102 | bytes | no depth data |
| theater tile files | each tile cell's height value | read, no effect on drawing or on a cell's level (claim 24) | integer | — |

## Measurement scenarios

All run under the glossary's measurement setup with stock data on purpose-made test maps (one
theater, stated per scenario, chosen by the measurer from the theaters whose data has the roles the
scenario needs). Where a scenario names test files, they are loose files (S-0005) documented with
the scenario, and everything else stays stock. Screenshots are compared with the stock tile images,
identified by tile number.

**F1. Files and numbering (claims 1 to 3, spot check of 20).**
- *Setup:* a map that places, in a row, the first tile of every tileset of the chosen theater, by
  the tile numbers that claim 2 gives (counting the stock tile counts of that theater's data file).
- *Observable:* screenshot of each placed tile.
- *Expected:* each shows the first tile of its tileset (one of that tile's images, chosen as claim
  13 says).
- *Falsified if:* any cell shows a tile of another tileset (numbering off), in particular after an
  empty tileset.
- *Fallback part (named test files):* a loose copy of the theater data file that adds one tileset
  after the last declared one, with tile count 1 and a new file-name stem that no stock file uses,
  and a loose tile file of that stem and number (a copy of any stock one-cell tile file, renamed)
  that exists only with the theater's fallback suffix. The test map places the new tileset's tile
  (number = the stock tile count). Run 1 leaves the new tileset's non-height-view field absent; run
  2 sets it to 0. *Expected:* run 1 shows the copied tile's image; run 2 shows the clear tile
  (claim 7: the tile has no image). *Falsified if:* run 1 shows the clear tile, or run 2 shows the
  copied image.
- *Gap part (claim 3; named test files):* loose files for one new tile stem, as in the fallback
  part but with the theater's own suffix: the plain image, an "a" variant and a "c" variant, three
  visibly different pictures, no "b" variant; the "a" file is spelled in the other letter case from
  the plain file (S-0005). The map is filled with that tile over an 8 × 8 block. *Observable:*
  which picture each cell shows. *Expected:* the plain picture and the "a" picture appear, by
  table P4 mod 2 (the tile has 2 images), and the "c" picture appears nowhere. *Falsified if:* the
  "c" picture appears, or the "a" picture does not.
- *Preference part (claim 3; named test files; in a theater other than 0 and 2):* loose tile files for
  the new stem and number of the fallback part, now in several copies with visibly different
  pictures, under different suffixes, in five runs, each in a fresh program start and each with the
  new tileset's non-height-view field absent. Run 3: copies with the theater's fallback suffix, the
  suffix of theater 0 and the suffix of theater 2. Run 4: only the last two. Run 5: only the suffix
  of theater 2. Run 6: the plain file with the theater's own suffix and an "a" variant under the
  fallback suffix only (the tile then has 2 images, so the 4 × 4 pattern of table P4 shows both pictures
  on a block of cells, claim 13). *Expected:* run 3 shows the fallback suffix's picture, run 4 the
  suffix of theater 0's, run 5 the suffix of theater 2's, run 6 both the plain and the "a" picture
  (by table P4 mod 2). *Falsified if:* a run shows another picture, or in run 6 the "a" picture
  never shows.
- *End-of-list part (claim 2; named test files):* five loose copies of the theater data file, each
  adding, after the last stock tileset (number *D*), the sections described, and a loose one-cell
  tile file of a new stem that no stock file uses, with the theater's own suffix and a picture that
  differs visibly from the clear tile. The map places tile number *T* (the stock tile count) in a
  block of cells. Each run in a fresh program start. Run 0 (control): section *D* has count 1 and
  that stem. Run 1: section *D* has count −1, section *D* + 1 has count 1 and that stem. Run 2:
  section *D* has no count field, section *D* + 1 as in run 1. Run 3: section *D* has count 0,
  section *D* + 1 as in run 1. Run 4: section *D* has count −2 (bad data), section *D* + 1 as in run 1.
  *Expected:* the new picture on the block in runs 0, 3 and 4 (a count of 0 or −2 gives no tile numbers and
  does not end the list, so the tile of section *D* + 1 is number *T*), and the clear tile in runs 1
  and 2 (the list ended at section *D*, so section *D* + 1 has no effect and *T* is the tile count,
  claim 7). *Falsified if:* the new picture shows in run 1 or 2, or the clear tile shows in run 0, 3 or 4.
- *Spot-check part for claim 20 (named test file):* a loose copy of the theater data file in which
  the clear tileset's allow-to-place flag is set to no and its two theater-conversion fields are
  changed. *Expected:* buildings can still be placed on clear ground, and screenshots match those
  of the unchanged data. *Falsified if:* placement on clear ground is refused or any tile looks
  different.

**C1. Unusable tiles (claim 7).**
- *Setup:* a map with cells holding: a tile number equal to the theater's tile count; a valid
  one-cell tile with sub-tile 1; a valid multi-cell tile at a sub-tile that is empty in its tile file.
- *Observable:* screenshot; and whether a vehicle can drive onto each cell and a building can be
  placed on it.
- *Expected:* each cell shows the clear tile and behaves as clear, flat land.
- *Falsified if:* any of them is impassable, sloped, or shows another tile.
- *Part B, the image of a cell without a tile (claim 24).*
  - *Setup:* two maps, each a block of 8 × 8 cells that hold a tile number equal to the theater's tile
    count (they hold the no-tile marker after loading, claim 7). The first is in a theater whose clear
    tile has 2 to 4 images, the second in one whose clear tile has more than 4 (the measurer checks
    the stock data, as V2 does); each run in a fresh program start.
  - *Observable:* the image of the clear tile each cell shows (by the variant files' pictures).
  - *Expected:* first map, image P4(x mod 4, y mod 4) mod *n*, with *n* the number of images of the
    clear tile (claim 13); second map, the 8 × 8 pattern of claim 14, with period 8 in x and y.
  - *Falsified if:* the first map shows the 8 × 8 pattern or one image everywhere, or the second map
    shows the 4 × 4 pattern.

**C2. The overlay case (claims 7 and 8; named test data).**
- *Part A, smoothing.*
  - *Setup:* scenario L1's plus-shaped patch of five rough ground cells, with an overlay of S-0211
    claim 5's row 1 laid on the N arm (the measurer picks a stock overlay type whose land type is
    railroad or wall, checking the stock data, and uses a map overlay on that cell). The tile
    under the overlay stays visible only if the overlay type is drawn over the ground; the
    measurer chooses one that leaves the ground visible, such as a railroad piece.
  - *Observable:* the tile the N arm shows under the overlay after loading.
  - *Expected:* *L* + 11, as without the overlay (claim 8: the overlay case is smoothed).
  - *Falsified if:* the N arm shows the rough ground tile *G*, or any other tile.
- *Part B (informative):* the overlay case's two other results in claim 7 (the tile number kept at
  or beyond the tile count, the slope shape from *t* mod (*w* × *h*)) are not proposed as
  measurements: see the backing of claim 7.
- *Part C, no tile animation in the overlay case (claim 8; named test data).*
  - *Setup:* scenario G4 part A's map (a stock tile that names an animation, placed once with all
    its cells, on flat ground at level 0), twice: a control as in G4 part A, and a map that differs
    only by a map overlay of S-0211 claim 5's row 1 (a railroad piece, as in part A) on the cell of
    the tile's attaches-to sub-tile. Each run in a fresh program start.
  - *Observable:* screenshot after loading, with the view large enough that the displacement is
    smaller than the view (as in G4 part A); whether the tile's animation appears.
  - *Expected:* the control shows the animation, as G4 part A says; the overlay map shows none (the
    cell is of the overlay case, which starts no animation), while the other cells of the tile are
    unchanged.
  - *Falsified if:* the overlay map shows the animation (if the control shows none, the part is
    inconclusive).

The land-type scenarios that this spec once held (terrain codes, overlays, ore on slopes, cells
behind cliffs) are S-0211's scenarios L3, L5, L11 and L16.

**X1. Placement and shape of the diamond (claims 22 and 24).**
- *Setup:* a loose one-cell tile file of the chosen theater, made for the test (stock names and
  numbers otherwise, replacing one stock one-cell tile file, so the measurer needs no new tileset),
  whose 900 diamond pixels have these indices: the first and last pixel of each row index 1, the
  other pixels of rows 0 to 13 index 2, and the other pixels of rows 14 to 28 index 3, except a block
  of 6 interior pixels in row 14 set to index 0. Neutral test lighting (S-0104), no extra image,
  no depth data (so no depth effects), the stock tile roles unchanged. A map of
  cells at level 0 in a 3 × 3 block, plus a second block at level 3 and a third at level −2 (the map
  gives the levels; S-0009), every cell holding that tile. The camera sits at a view origin that the
  measurer reads by S-0101's measure.
- *Procedure:* screenshot with the whole map in view.
- *Observable:* for the centre cell of each block, the view pixel of every pixel with indices 1, 2, 3
  and the colour at the 6 interior pixels; gaps or overlaps between neighbouring cells of one block.
- *Expected:* pixel (*c*, *r*) of the centre cell lies at (*u* − 30 + *c*, *v* − 15ℓ + *r*), with *u* and
  *v* from S-0101 F1 and claim 5 for that cell's top corner at height 0; row *r* has exactly *n*(*r*)
  pixels starting at column *f*(*r*); no row 29; neighbours of one block leave no gap and show no
  doubly painted pixel; the 6 interior pixels show the colour of palette index 0 under neutral
  lighting, not the ground below them.
- *Falsified if:* any pixel is off by one in either coordinate, a row has another width, a gap or a
  hidden row appears between same-level neighbours, or the index 0 pixels are transparent.

**X2. The extra image (claims 23 and 24; named test files).**
- *Setup:* loose copies of stock one-cell tile files of the chosen theater (replacing three stock
  one-cell tiles, named by the measurer), edited so that the first has an extra image mark and an
  extra image of 20 × 20 pixels, the second one of 80 × 10 pixels, the third one of 20 × 40 pixels,
  each with indices 1 and 2 in a checker pattern and 6 pixels of index 0; the stored positions of
  the tile cell and of the extra image are set so that the difference is, per file, (−20, −20),
  (−10, 15) and (30, −30) (the measurer chooses other differences if the format limits them, and
  records them). No depth data. A fourth tile: a stock one-cell tile with one loose variant "a" that
  has an extra image where the plain file has none (indices as above). Neutral lighting; every
  tile on its own flat 3 × 3 block at level 0, and the fourth tile on the block of scenario V1 so
  that cells with image 1 can be told apart (claim 13).
- *Procedure:* screenshot.
- *Observable:* the top-left pixel of each extra image relative to the top-left corner of the
  diamond's box, (*u* − 30, *v*) (claim 23); whether the index 0 pixels show the diamond or ground below; on which cells of the
  fourth block an extra image appears.
- *Expected, A to C:* (A) the extra image's top-left pixel is at (*u* − 30 + Δ*X*, *v* + Δ*Y*) for the
  stored difference (Δ*X*, Δ*Y*), for all three files, including parts that lie right of column 59 or
  below row 28; (B) index 0 pixels show what is underneath; (C) in the fourth block the extra image
  appears exactly on the cells showing image 1, with nothing on the cells showing the plain image.
- *Falsified if:* an extra image lies elsewhere, an index 0 pixel paints, or the extra image appears
  on a cell showing the plain image or is missing on one showing the variant.
- *Part D (depth, claim 23):* S-0102's scenario D6 over a stock cliff tile cell that has an extra
  image and depth data (the measurer identifies one by the tile file). *Observable:* the pixels of a
  vehicle that are missing where it is behind the extra image's opaque pixels. *Expected:* hidden
  exactly where the extra image's opaque pixels are nearer by the depth of claim 23, and not hidden
  elsewhere. *Falsified if:* the vehicle shows over opaque pixels of the extra image that D6's
  cliff reading says should hide it, or is hidden where only the extra image's transparent pixels lie.

**X3. The tile file's height value (claim 24; named test file).**
- *Setup:* a loose copy of a stock one-cell tile file with only the tile cell's height value changed
  (set to 5, then a second copy to 0 if the stock value was not 0), otherwise identical. Two maps
  that differ only in whether the stock file or the copy is used: a flat 3 × 3 block at level 0 and
  another at level 2, plus a wheeled vehicle that drives across each block.
- *Observable:* both screenshots, pixel by pixel; the vehicle's screen height on each block
  (S-0101 scenario M2).
- *Expected:* identical images; the vehicle's lift on the level 2 block is that of level 2 (15 × 2
  pixels for the terrain, S-0101) whichever file is used.
- *Falsified if:* any pixel or lift differs.

**X4. A variant without the cell (claim 24, `quirks.variant_cell_missing`; named test file).**
- *Setup:* a stock tile of at least two cells in each direction and a loose variant "a" made from its
  plain file with one tile cell emptied (the cell absent in the variant's file). A flat map on which
  the tile is placed at many positions (the 4 × 4 pattern of claim 13 applies by the placed origins),
  so that at least one placed tile shows image 1 (known from the other cells of that tile showing
  the variant's picture). Neutral lighting; a wheeled vehicle ordered across the emptied cell.
- *Observable:* screenshot of the emptied cell of a tile that shows image 1, and of the same cell of
  a tile that shows image 0; whether the vehicle can cross the emptied cell.
- *Expected:* the cell of the image 1 tile paints nothing (its pixels keep whatever the terrain picture
  held there, which claim 24 leaves unspecified for a first painting), not the plain file's picture; the cell of the image 0 tile shows its picture; the vehicle crosses in both
  (the cell's land type is the plain image's).
- *Falsified if:* the plain file's tile cell is painted in the emptied cell, or the cell is
  impassable.
- *Part B, a variant with fewer positions (claim 24; named test files; the scenario above is part A).*
  - *Setup:* a stock tile 2 cells wide and 1 high with no variants (or a loose copy of one), and
    a loose "a" variant for it made from a stock one-cell tile file, so the variant has 1 position and a
    picture that differs visibly from both tile cells of the plain file. The map places the tile
    once with its origin at (2, 0), as in scenario V1 part B, so that cell (3, 0) shows image 1
    (P4(1, 0) = 1). Neutral lighting.
  - *Observable:* the picture in cell (3, 0), sub-tile 1.
  - *Expected:* the variant's picture (index 1 mod 1 = 0 of the variant). Cell (2, 0), the origin cell,
    shows image 0 (P4(2, 0) = 2 mod 2).
  - *Falsified if:* cell (3, 0) shows nothing (its pixels as before), or the plain file's tile cell
    at sub-tile 1.

**L1. Ground smoothing (claim 10).**
- *Setup:* in a theater whose data has the rough pair set, a map with a plus-shaped patch of the
  rough ground tile (five cells) on clear ground, and a 3 × 3 square patch.
- *Observable:* screenshot of the patches.
- *Expected:* the centre of the plus shows the rough ground tile *G*; the arms show transition
  tiles *L* + *m* with *m* = 11 for the N arm (its N, E and W neighbours out), 7 for the E arm,
  14 for the S arm and 13 for the W arm; the square's centre shows *G*, and its edge and corner
  cells show *L* + *m* for their own out neighbours (for example *m* = 9 for the north-west corner
  cell of the square, whose N and W neighbours are out).
- *Falsified if:* any cell shows a different tile from the formula's.

**L2. Ramp smoothing and load order (claims 11 and 12).**
- *Setup:* a map with an isolated shape-1 ramp-range tile between flat cells W and E; a second
  shape-1 cell whose E neighbour is another shape-1 ramp cell one level higher (a two-level climb),
  a shape-3 cell whose W neighbour is a shape-3 ramp cell one level higher.
- *Observable:* screenshot after loading.
- *Expected (smooth-ramp start written Q):* the isolated cell shows Q + 2 (both neighbours flat).
  Eastward climb: the lower cell shows Q + 0 (only its W neighbour is flat), the upper cell Q + 1
  (only its E neighbour is flat). Westward climb (shape 3 tests E with weight 1 and W with
  weight 2): the lower cell shows Q + 6, the upper cell Q + 7. These are the tiles of the second
  load pass, which sees every neighbour's real shape (claim 12).
- *Falsified if:* the isolated cell shows anything but Q + 2. The lower eastward cell showing Q + 2,
  or the upper westward cell Q + 8, would mean that the first pass's view (later cells seen as flat)
  survives loading, so claim 12's second pass, or the shapes of the smoothed tiles, are not as
  claimed.

**L3. Unset ramp roles (claim 11; named test files).**
- *Setup:* a theater whose data sets both ramp roles, with ramp start above 2 and both ramp tilesets
  after the tilesets that hold tiles 0 … 2 (the measurer checks the stock data). Tiles 0 … 2 are
  usable (each has an image and a tile cell at sub-tile 0) and lie in no ground, transition or
  ramp range of claims 10 and 11, so that the second load pass leaves them as they are. For part
  B, the three tiles of the smooth-ramp tileset that a shape-2 cell can be smoothed into (*Q* + 3,
  *Q* + 4 and *Q* + 5, *Q* the smooth-ramp start) have slope shape 2 in their tile files, which
  is claim 12's assumption (the measurer checks the stock files; otherwise the part is
  inconclusive). Part A: a loose
  copy of the theater data file without the smooth-ramp key; the map holds an isolated shape-1
  ramp-range tile (ramp start + 0) between flat W and E neighbours. Part B: a loose copy without the
  ramp key; the map holds a north-to-south line of three cells, each holding a shape-2 tile of the
  smooth-ramp range.
- *Observable:* screenshot after loading; the tile number each observed cell shows.
- *Expected:* Part A: the isolated cell shows tile 1 (3·0 + 3 − 2, both neighbours flat). Part B:
  the middle cell shows tile 0 (*m* = 0, since its N and S neighbours are sloped, so *s* − 2 = 0).
  Each tile is drawn as claims 6 and 7 give for sub-tile 0, which is the clear tile if that tile has
  no tile cell at sub-tile 0.
- *Falsified if:* in part A the cell keeps its ramp tile or shows any other tile; in part B the
  middle cell shows a smooth-ramp-range tile or any tile other than 0.

**L4. A ramp-range tile of slope shape 0 (claim 11, `quirks.ramp_shape_zero`; named test file).**
- *Setup:* a loose copy of the tile file of one ramp-range tile (the first tile of the ramp
  tileset, say) with every slope code set to 0 (the tile file format is public), a one-cell tile
  for this test; the ramp tileset after the tileset that holds the tile just before the ramp range
  (the measurer checks that tile is usable and lies in no ground, transition or ramp range). A map
  with that tile on flat ground.
- *Observable:* the tile the cell shows after loading, compared with the stock tile images.
- *Expected:* the tile just before the ramp range (ramp start − 1), not the ramp-range tile.
- *Falsified if:* the cell keeps its ramp-range tile or shows any other tile.

**R1. Unset and one-past-the-end roles (claims 4, 5 and 10; named test files).**
- *Setup:* a theater whose data sets the rough pair, in which none of the tiles 8 … 14 is one of the
  four ground tiles of claim 10 (rough, sand, green or pave) or lies in a transition range of claim
  10 or in a ramp range, and each of them is usable (it has an image and a tile cell at sub-tile 0,
  so that the second pass keeps it instead of replacing it by the no-tile marker) (the measurer
  checks the stock data). A map with a plus-shaped patch of five rough ground cells (centre (*x*, *y*) and its
  four edge neighbours) on clear ground, with no other rough or transition tiles nearby. By S-0009
  claim 3 the five cells are visited in the order W arm, N arm, centre, S arm, E arm in each pass.
  Five runs, each in a fresh program start:
  - Part A: unchanged data (this is the plus shape of scenario L1).
  - Part B: a loose copy of the theater data file without the rough-transition key.
  - Part B2: a loose copy in which the rough-transition key is present with the value −3 (a
    negative number, claim 4); the expected results are those of part B.
  - Part C: a loose copy in which the rough-transition key names the first missing tileset number
    plus 1.
  - Part D: a loose copy in which the rough-transition key names exactly the first missing tileset
    number, so the rough transition start is the tile count *T*.
- *Observable:* screenshot after loading; the tile number each of the five cells shows. In part D,
  also whether a building can be placed on the centre cell and on an arm cell, judged by the
  placement indicator without placing it, with a building of the human house close enough that
  only the land type decides (entering the cell, or placing a building on it, would recalculate
  it, Terms). This check, and so the centre's number *T* + 9, is observed only if the stock land-type sections
  give rough and clear land different buildable fields; the measurer checks that first, and
  otherwise leaves the centre unobserved (the arms still decide part D).
- *Expected:* Part A: as L1 (centre *G*; arms *L* + 11, *L* + 7, *L* + 14, *L* + 13 for N, E, S, W).
  Parts B, B2 and C (start −1, its range only the tile −1): W arm tile 12 (*m* = 13), N arm tile 10 (*m* = 11), centre
  tile 8 (*m* = 9, because its N and W arms were rewritten earlier in the same pass and count as
  out), S arm tile 14 and E arm tile 14 (*m* = 15, because the rewritten centre counts as out); the
  second pass changes none of them, because none still belongs to the pair. Each is drawn as claims
  6 and 7 give for sub-tile 0. Part D: the first pass writes *T* + 13, *T* + 11, *G*, *T* + 14 and
  *T* + 7 (all still in the transition range); the second pass replaces every arm by the no-tile
  marker (claim 7) and gives the centre *T* + 9, because its N and W arms are already out. The four
  arms show the clear tile (the image of claim 24's no-tile case) and play as clear land. The centre
  holds a number at or beyond the tile count, so its picture is unspecified (claim 24) and is not
  judged; it plays as rough land (its land type comes from *G*, the tile it held when the second pass
  reached it) until its next recalculation (part E), where the placement check can tell.
- *Falsified if:* in parts B, B2 or C any arm shows a rough transition tile, the centre shows *G*, or
  the tile numbers differ from those above; in part D any of the four arms shows a rough
  transition tile, the rough ground tile or anything but the clear tile. Parts B, B2 and C giving different results would falsify
  the rule that an absent key, a negative number and a number beyond the first missing tileset all
  leave the role unset.
- *Part E, a unit's move recalculates the cell (claims 10 and 12, and the triggers in Terms).*
  *Setup:* the data and map of part D, and a wheeled vehicle of the human house parked outside the
  plus. *Procedure:* after loading, the placement check of part D on the centre; then drive the
  vehicle onto the centre and off again; then the same placement check. *Expected:* before the
  drive the centre judges as rough land and the arms as clear land; after it the centre judges as
  clear land too (it now holds the no-tile marker). *Falsified if:* the centre still judges as
  rough land after the drive. The same buildable-field condition as part D applies.

**R2. Roles forced unset in theater 5 (claims 4 and 10; named test files).**
- *Setup:* in the theater with index 5, a loose copy of its theater data file in which the shore
  role and the green ground and green transition roles each name a tileset of that theater (the
  transition role one with at least 16 tiles). A map with a green ground cell *c* whose N
  neighbour holds the first tile of the tileset named by the shore role and whose E, S and W
  neighbours hold the green ground tile. Control: the same test in a theater whose own data sets
  the shore role (with a loose copy that sets the green roles if they are absent).
- *Observable:* the tile number *c* shows after loading.
- *Expected:* in theater 5, *L* + 1 (the shore role is forced unset, so the N neighbour counts as
  out); in the control, *G* (the N neighbour lies in the shore range and counts as belonging).
- *Falsified if:* *c* shows *G* in theater 5, or the control does not show *G*.
- *Part B, water bridge.* The test above is part A. Part B repeats it with the water-bridge role in
  place of the shore role: the loose copy for theater 5 makes the water-bridge role name a tileset
  of that theater, *c*'s N neighbour holds the first tile of the tileset that the water-bridge role
  names, and the control theater's data (or a loose copy of it) sets the water-bridge role.
  *Expected:* in theater 5, *L* + 1; in the control, *G* (the N neighbour lies in the water-bridge
  range and counts as belonging). *Falsified if:* as in part A.

**R3. Pairs skipped when their transition role is unset (claim 10; named test files).**
- *Setup:* a theater whose data sets the sand, green and pave pairs, in which the three ground
  tiles lie in no transition, ramp or other pair's range (the measurer checks the stock data). A
  map with three separated plus-shaped patches (five cells each, on clear ground) of the sand, the
  green and the pave ground tile. Two runs, each in a fresh program start: a control with the
  unchanged data, and a loose copy of the theater data file without the sand, green and pave
  transition keys (the ground keys kept).
- *Observable:* the tile each of the 15 cells shows after loading.
- *Expected:* in the control, each patch is smoothed as in scenario L1 (centre *G*, arms
  *L* + 11, *L* + 7, *L* + 14, *L* + 13 for N, E, S, W). In the loose copy, all 15 cells show their
  ground tile, unchanged (the rough pair, which is not skipped, is scenario R1).
- *Falsified if:* in the loose copy any arm shows a tile other than its ground tile.

**B1. Range ends (claims 5, 10 and 11; named test files where stock data does not fit).** The
tiles used below are usable (each has an image and a tile cell at sub-tile 0), and the measurer
checks the stock data, or makes a loose copy of the theater data file, so that each range-end tile
lies in no ground or transition range of claim 10 and no ramp range of claim 11 except the range
under test.
- *Part A, the widened neighbour tests of the green pair (shore range, 42 tiles; water-bridge range,
  2 tiles).*
  - *Setup:* a theater whose data sets the green pair, the shore role and the water-bridge role. For
    each of the two ranges, with *n* its count and *R* its start, two green ground cells *c*: each
    has green ground tiles *G* on its E, S and W neighbours, and on its N neighbour the tile
    *R* + *n* − 1 (the last tile of the range) for the first cell and *R* + *n* (the tile after
    it) for the second.
  - *Observable:* the tile each *c* shows after loading.
  - *Expected:* *G* for the cell whose N neighbour holds the last tile of the range (it counts as
    belonging); *L* + 1 for the cell whose N neighbour holds the tile after it (it counts as out).
  - *Falsified if:* the last-tile cell shows *L* + 1 (the range is shorter), or the cell with the
    tile after the range shows *G* (the range is longer).
- *Part B, the widened neighbour test of the pave pair (miscellaneous-pave range, 14 tiles,
  paved-road range, 21 tiles, median range, 14 tiles).* As part A, with pave ground cells and the
  pave ground tile on the E, S and W neighbours, for each of the three ranges, with the same
  expected results (*G*, and *L* + 1 for the pave transition start *L*).
- *Part C, the ramp range (20 tiles) and the smooth-ramp range (12 tiles).*
  - *Setup:* a theater whose data sets both ramp roles. For each range, a map with two cells on
    flat ground whose edge neighbours are all flat: one holds the last tile of the range, the
    other the tile after it. The measurer computes with claim 11 (and its second application in
    the second load pass, to the first pass's result) the tile the first cell ends with, from the
    slope shape of its tile cell, and uses the part only for a range where that tile differs from
    the placed one; where the tile after the range lies in the other ramp range, the second cell's
    expected tile is computed from that range's formula instead.
  - *Observable:* the tile each of the two cells shows after loading.
  - *Expected:* the last-tile cell shows the computed tile (not the placed one); the cell with
    the tile after the range keeps it (or shows the other range's computed tile, as said).
  - *Falsified if:* the last-tile cell keeps its placed tile (the range is shorter), or the other
    cell is re-chosen by this range's formula (the range is longer).

**V1. Fixed variant pattern (claim 13).**
- *Part A, one-cell tiles.*
  - *Setup:* a map filled with one one-cell tile that has 2 or 4 images in the chosen theater's
    stock data (identified by its variant files).
  - *Observable:* which image each cell shows, over an 8 × 8 block.
  - *Expected:* image P4(x mod 4, y mod 4) mod (number of images).
  - *Falsified if:* any cell differs.
- *Part B, a tile larger than one cell (`quirks.variant_origin_mismatch`).*
  - *Setup:* a tile 2 cells wide and 1 high with 2 images. Use one from the chosen theater's stock
    data if the measurer finds one. Otherwise use a named test file: a loose "a" variant for a stock
    tile of that size that has no variants, made from another tile file with the same cell layout whose image differs
    visibly. The map places the tile once with its origin at (0, 0) and once at (2, 0).
  - *Observable:* which image each of the four cells shows.
  - *Expected (by the Formulas):* at origin (0, 0), both cells show image 0. At origin (2, 0), the
    origin cell shows image 0 (P4(2, 0) = 2, from its own coordinates) and cell (3, 0) shows image 1
    (*X* = (3 − 1) / 2 = 1, P4(1, 0) = 1), so the two halves of the placed tile differ.
  - *Falsified if:* any cell shows another image, in particular if both halves of the tile at
    (2, 0) show the same image.

**V2. 8 × 8 pattern (claim 14).**
- *Setup:* a map filled with a one-cell tile that has 8 images (for example the stock clear tile of
  a theater that has 8 clear images). Two runs, each in a fresh program start, going straight to
  the map; a third run in the same session after another map of the same kind.
- *Observable:* the image of each cell over a 16 × 16 block.
- *Expected:* the images repeat with period 8 in x and in y; within the 8 × 8 period, no entry
  equals one of its 8 neighbours in the index-wrapped sense of the Formulas, except where 64 draws
  failed (rare); the third run shows the same pattern as the map before it in that session.
- *Falsified if:* the period is not 8, or the pattern changes between two maps of one session.
- *Part B, the draws (claim 14; needs a recorder that reads the unsynchronised stream, an extension
  of S-0202's state recorder, which the project lead has approved).* *Setup:* the map of part A as the first game of a fresh
  program start. *Observable:* the game's seed and the unsynchronised stream's state at the first
  tick, read by the recorder, and the 8 × 8 period of images from part A. *Procedure:* compute the
  unsynchronised stream's raw values from the seed (S-0202 claims 2 and 8), and for *N* = 0, 1, 2, …
  apply the formula *8 × 8 pattern* to the raw values that follow the first *N*. *Expected:* some
  *N* gives the observed period; record the smallest. *Falsified if:* no *N* up to 100000 gives
  the observed period.

**V3. Variants do not change the land type (claims 6 and 13; named test file).**
- *Setup:* a one-cell tile of the chosen theater with one image (no variant files) whose terrain
  code is 0 (clear), and a loose "a" variant for it made from a one-cell water tile file of the
  same theater. A map filled with that tile over a 4 × 4 block, so by table P4 the cells with
  P4 entry 1 or 3 show image 1 (the water picture).
- *Observable:* screenshot; whether a wheeled vehicle can be ordered onto each of the 16 cells.
- *Expected:* the water picture on the 8 cells with odd P4 entries, and the vehicle can enter all
  16 cells (clear land from the plain image).
- *Falsified if:* any cell showing the water picture refuses the vehicle, or plays as water.

**G1. The morphable flag (claim 15).**
- *Part A.* *Setup:* a map override that gives a stock building type a to-tile field naming a stock
  tile, and a test map with a cell of a tileset whose morphable flag is unset and a cell of one
  whose flag is set (the measurer checks the stock data). *Observable:* whether the building can
  be placed on each. *Expected:* only on the morphable cell. *Falsified if:* it can be placed on
  the non-morphable cell.
- *Part B, a cell without a tile.* *Setup:* part A's building, and a map cell holding a tile
  number equal to the theater's tile count (claim 7). *Observable:* whether the building can be
  placed on it. *Expected:* it can. *Falsified if:* it cannot.
- *Part C, a stray tile number (claim 15; named test files).* *Setup:* the data and map of scenario
  R1 part D (the centre of its plus holds a stray tile number from the second load pass until its
  next recalculation) and part A's building, with a building of the human house close enough that
  only the tile and land type decide (as in R1 part D). Two controls on the same map, each a cell
  whose land type is the centre's (rough) that nothing recalculates before the check: one in a tileset whose morphable flag is unset
  and one in a tileset whose flag is set (the measurer picks them in the stock data and adds them
  to the map; if the stock data have no such pair, the part is not run). *Observable:* the
  placement indicator of the building on each of the three cells, judged without placing it and
  before any unit enters or leaves them. *Expected:* the non-morphable control refuses (the tile
  test), the morphable control accepts (so the land type and the other conditions allow the
  building on rough land), and the plus's centre accepts. *Falsified if:* the centre refuses while
  the morphable control accepts. Inconclusive if the morphable control refuses or the
  non-morphable control accepts.

**G2. Allow-tiberium (claim 16).** *Setup:* a map with ore next to large areas of two tilesets, one
with the allow-tiberium flag and one without, both on buildable land, with ore growth on; long runs
at fastest speed. *Observable:* the cells new ore appears on after 20 game minutes, over 5 runs.
*Expected:* new ore only on the allowing tileset. *Falsified if:* any new ore on the other.
*Part B, a cell without a tile.* The same map with a block of cells that hold a tile number equal
to the theater's tile count (clear, buildable, flat land, claim 7) next to the ore. *Expected:* new
ore also appears on those cells in the 5 runs. *Falsified if:* it never does.

**G3. Burrowing (claim 17).** Inside the playable area only (the answer outside it is not measured,
claim 17). *Setup:* a tunnelling unit ordered to a destination cell of a tileset
without the allow-burrowing flag, and to one with it. *Observable:* where it comes up. *Expected:*
not on the first cell (it comes up on the nearest allowed cell). *Falsified if:* it comes up there.

**G4. Tile animation (claim 19).**
- *Part A.* *Setup:* a map with a tile whose stock data names an animation (the measurer picks one
  from the stock theater data), placed once with all its cells, on flat ground at level 0, with a view large enough that
  the displacement is smaller than the view. *Observable:* screenshot; the screen position of the
  animation's anchor relative to the screen position of the attaches-to cell's centre. *Expected:*
  one animation, on the cell of the attaches-to sub-tile, displaced by (*dx*, *dy*) pixels from the
  cell's centre, to within one pixel (the round trip of S-0101's F6 and F1, S-0101 claim 9).
  *Falsified if:* the animation repeats per cell, appears on another cell, or is displaced by more than
  one pixel from (*dx*, *dy*).
- *Part B, the tile before smoothing (named test file).* *Setup:* a loose copy of the theater data
  file that gives the third tile of the smooth-ramp tileset (*Q* + 2) an animation, with any stock
  animation type and attaches-to sub-tile 0, in the section named by that tileset's set name. The
  map is scenario L2's eastward climb. *Observable:* screenshot after loading. *Expected:* the lower
  cell shows *Q* + 0 (L2) and carries the animation, because the second pass started from *Q* + 2,
  the tile the first pass left there. *Falsified if:* the lower cell carries no animation.

- *Part C, a start in play after smoothing (named test file).* *Setup:* a loose copy of the theater
  data file that gives the first tile of the smooth-ramp tileset (*Q* + 0) an animation with
  attaches-to sub-tile 0, and no animation to *Q* + 2, as in part B. The map is scenario L2's
  eastward climb, and a wheeled vehicle of the human house parked beside the lower cell.
  *Procedure:* after loading, drive the vehicle onto the lower cell (the cell is recalculated,
  Terms). *Observable:* screenshot of the lower cell before and after. *Expected:* after loading
  the lower cell shows *Q* + 0 and no animation (the second pass started from *Q* + 2); after the
  vehicle has entered it carries the animation (the recalculation starts from *Q* + 0, and the cell
  has started none). *Falsified if:* the animation never appears.
- *Part D, a displacement far larger than the view (claim 19; named test file).* *Setup:* a loose
  copy of the theater data file that gives one stock one-cell tile an animation (any stock
  animation type) with attaches-to sub-tile 0, in the section named by that tileset's set name.
  Run 1: *dx* = 20000 and *dy* = 0. Run 2: *dx* = 0 and *dy* = 20000 (both far beyond the width and
  height of any screen the measurer uses, which S-0101's measure gives). Control, run 3: *dx* = 40,
  *dy* = 0, which F6 displaces visibly. The map places the tile once on flat ground at level 0,
  well inside the view; each run in a fresh program start. *Observable:* screenshot; the screen
  position of the animation relative to the tile cell's centre. *Expected:* in runs 1 and 2 the
  animation appears at the cell's centre (to within one pixel), not displaced; in run 3 it is
  displaced by (40, 0) pixels as in part A. *Falsified if:* in run 1 or 2 the animation is
  not at the cell's centre (including when it does not appear at all), or in run 3 it is not
  displaced. The boundary of the case (a displacement near the view's size) is not tested.

**G5. Shadow casters need a count (claim 18; named test files).** *Setup:* a theater whose stock data
declares a tileset a shadow caster with a non-zero shadow-tile count (the measurer checks), and
S-0104's scenario part D on one tile of that tileset. Run 1: stock data. Run 2: a loose copy of the
theater data file without that tileset's shadow-tile count (flag kept). Run 3: a loose copy without
the flag (count kept). *Observable:* the vehicle's brightness on each cell, as in S-0104 part D.
*Expected:* some cells darken the vehicle in run 1 (S-0104); none in runs 2 and 3. *Falsified if:*
run 2 or run 3 darkens a cell that run 1 darkens. If run 1 darkens no cell, the part is
inconclusive.

## Open questions

- Q-0009 is answered in full. Its question 1 by claims 1 and 3 and table T, 2 by claim 2, 3 by
  claims 3, 7 and 24, 4 by claims 13 and 14, 5 by claims 6, 7 and 24, 6 by claims 22 to 24, 7 by
  claims 7 and 24 and S-0009 claim 8, 8 by claim 6 and S-0211 claims 5 to 8, and 9 (the spelling of
  the shadow-caster key) by the shadow-caster row of Import mapping.
- What the original's painting does for a cell that holds a tile number at or beyond the tile count
  (claim 24): no tile of that number exists, and no result of the picture is established, so the spec
  leaves it unspecified.
- Whether a tile cell whose file declares a cell height other than the standard 30 rows moves the
  depth shift of S-0102's F3 (the original appears to take it from the file), and the same for the
  width. Stock tile files are not known to differ, so claim 22 fixes 60 × 29 and S-0102 fixes 30.
- Which cells the original paints when it repaints only part of the screen: it picks cells by a
  rectangle that is 60 pixels wide and reaches up to the top of the cell's extra image, so an extra
  image reaching outside it may be cut or missed at the edge of a repainted area. Our engine paints
  whatever its images touch; the difference is a repainting artefact, not a rule, and is not
  specified.
- Whether the radar colours of a cell come from the plain image or from the image it shows (claim
  6 covers only the land type and the slope shape, which are read from the plain image).
- Whether the smoothed ramp tiles of the stock theaters carry the same slope shape as the ramp
  tile they replace (claim 12 assumes so).
- Whether placing the map's terrain objects and units at map load, between the two passes of
  claim 12, recalculates their cells, as putting an object into a cell does in play (Terms).
  Claims 12 and 19 state the results after loading on the reading that it does not.
- What a to-tile building does when placed, and how the to-tile name is matched (claim 15).
- Which game rules use the role tests of table R that this spec does not use (bridges, cliffs and
  their destruction, shores, naval placement, the random map generator).
- Which tiles of a shadow-casting tileset cast shadows (claim 18). The original appears to give
  shadows only to the first 40 tiles of such a tileset and to keep at most five shadow-casting
  tilesets per theater, with an unspecified result for a sixth. Not established well enough to claim.

## Questions for the project lead

- **The glossary's *terrain animation* (and S-0103's Terms).** Both say a terrain animation is
  created "when the map's terrain is set up". By claim 19 a tile animation also starts in play, at
  the recalculation of a cell that has started none, whenever the tile it then holds qualifies (for
  example after a unit's move, scenario G4 part C). The entry should say "when the map's terrain is
  set up, or when a cell is recalculated in play".
- **S-0009 claims 6 and 8 on drawing a tile number at or beyond the tile count.** They say such a
  number is drawn as the theater's clear tile. That holds when the game decides which artwork to
  load, but by claim 24 the original's painting of a cell that still holds such a number is
  unspecified. Our engine paints the clear tile either way, so only the statements about what the
  original does need a revision of S-0009 (a pointer to claim 24 in place of "is drawn as").
