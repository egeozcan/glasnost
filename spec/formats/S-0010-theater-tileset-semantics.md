---
id: S-0010
title: Theater data, tilesets and the terrain values of a cell
topic: formats
revision: 21
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-12
measurements: []
requests: [Q-0009, Q-0018, Q-0041, Q-0046, Q-0055, Q-3006]
supersedes: []
---

# S-0010: Theater data, tilesets and the terrain values of a cell

## Summary

How the original reads a theater's tile data (its files, the declaration of tilesets and tile
numbers, tile image files and their variants, and the named tile roles), and what a cell takes from
its tile (slope shape, unusable tiles, the automatic smoothing of ground transitions and ramps, the
choice of a tile variant, and what the tileset flags do in play), and where the picture of a tile
cell and its extra image are drawn on the screen (claims 22 to 24), and the bridge spans that the
tiles of the two bridge roles define at game start, with the cells they mark (claim 25). Not covered here: a cell's land
type, including the effect of overlays and neighbouring cliffs (S-0211), the byte layout of tile
files (public), the meaning of the slope shapes (S-0106), the colour that a drawn pixel takes from
the cell's brightness and the palette (S-0104, S-0103), and how tile images overlap each other
(S-0102). This revision covers skirmish games only; network games, campaigns and the random map
generator are out of scope. Claims 13 and 24 also specify unconditional corrections for two
terrain-picture artefacts, with the original's behaviour retained for oracle comparisons.

## Terms

*Theater*, *tileset*, *sub-tile*, *cell*, *top corner*, *height level*, *slope*, *overlay*,
*overlay data value*, *archive*, *resolve*, *loose file*, *rules database*, *game start*, *session*,
*quirk switch*, *departure*, *importer warning*, *unspecified*, *unsynchronised random stream* (with
*draw*), *battlefield view*, *world pixel* and *measurement setup* are used as in the glossary.
*Resource glint* is S-0202's animation selected for a resource cell during game-start setup,
not a tile animation. *Cell animation* is the overlay-attached animation of S-0009 and S-0202.
*Slope shape* and *surface height* are defined by S-0106. *View pixel* and *view origin* are
S-0101's (claim 5 and its formula F1). The *land
type* rules of a cell (its *land recompute*, the *cliff-back setting*, the *own-land-type flag*) are
S-0211's. *Visiting
order*, *blank cell*, *no-tile marker* (65535), *unset base tile*, the *playable area* (claim 4)
and the *terrain stream* are S-0009's. S-0012 claim 6 owns the terrain stream's byte encoding,
including the tile number and sub-tile values in each cell record. *Terrain smoothing* is S-0009's name for ground and ramp smoothing (claims 10 and 11); S-0009
defines it as part of every terrain recalculation, at map load (claim 12) and in play, and
this spec owns its rules. The terms below are specific to this spec.

- **Theater data file.** The data file that declares one theater's tile roles and tilesets (claim
  1). Its sections are the *general section* and one *tileset section* per tileset.
- **Tile.** One numbered terrain piece of a theater. A tile covers a rectangle of *w* × *h* cells of
  which some may be empty. Each non-empty cell of a tile is a **tile cell**. A tile cell has an image,
  a *terrain code* (an integer the tile file stores, 0 … 15 in valid data), a *slope code* (the
  slope shape of S-0106, 0 … 20 in valid data) and two radar colours: the **first tile radar
  colour** and the **second tile radar colour**, each a red, green and blue triple of integer
  channel values 0 … 255. These names distinguish the two triples in their file order, not two
  radar points. Their use is claim 20's and S-0410 claim 9's. The tile file format is
  public; this spec says only what these values mean.
- **Alternate-image mark, alternate-image state.** The mark is a yes-or-no value of a tile cell in
  its tile file: one bit of the tile cell's flag value, the value whose bit of value 1 is the
  extra-image mark and whose bit of value 2 is the depth-data mark (Import mapping). The mark is set
  exactly when the bit of value 4 of that flag value is set, whatever its other bits hold. The state
  is a yes-or-no value of a map cell. Together they choose an image without the variant pattern
  (claim 13).
- **Bridge span, high span, low span.** A stretch of map cells between two end cells whose tiles
  belong to the bridge role or the wooden-bridge role, found at game start and sometimes found
  again in play (claim 25). A span
  is *high* when every cell between its ends that is not a bridge-role tile is a high-bridge cell
  (S-0211 claim 15), and *low* otherwise.
- **Bridge-body mark.** A yes-or-no mark of a map cell, besides the high-bridge marking of S-0211
  claim 15, that says the cell belongs to a bridge's structure. Low spans set it at game start
  (claim 25); the destruction of a high bridge sets it too (claim 25). It is clear on every cell
  after map load.
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
  its triggers (S-0211 claim 5). It runs for every cell twice when a map is loaded and once more at
  game start (claim 12), and
  again for one cell on each trigger in play: an overlay is placed, removed or changed there, its
  tile, sub-tile or ground level changes (for example a destroyable cliff or a bridge is destroyed
  or repaired), or an object that stands on the ground (an infantry unit, a vehicle, an aircraft on
  the ground, a building or a terrain object) is put into the cell or taken out of it. The last
  trigger includes every ordinary move of a unit from one cell to the next, so every cell a unit
  enters or leaves is recalculated. Resizing the local area in play also recalculates every
  legal cell (S-0524 claim 8), under the same terrain and animation rules, not the second
  load pass's animation replacement rule. The *game-start pass* of claim 12 is the whole-map
  recalculation at game start. S-0211 claim 5 holds the authoritative list.
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

Unless a part has an explicit confidence of its own, the confidence stated under its numbered
claim applies to that part too, including table rows and absent-key defaults. A backing delegated
to another claim inherits that claim's confidence as well as its scenario. A defined correction
in our engine is a design rule, not a claim of an oracle measurement.

### Theater data

1. **Files of a theater.** For a theater the original reads one theater data file and takes its
   tile images from files with the theater's tile suffix (claim 3). Both are fixed per theater
   (Constants, table T). The theater data file is resolved like any data file (S-0005).
   - Backing: `measurable` (scenario F1, its theater part: the data file name, the tile suffix and
     the fallback suffix of each of the six theaters, one run per theater)
   - Confidence: high

2. **Tileset declaration and tile numbers.** The tileset sections are named with a fixed prefix and
   a four-digit, zero-padded number counted from 0 (Import mapping). The declared tilesets are those
   numbered from 0 up to, but not including, the first number whose section has no tile-count field
   or gives it as −1 (an explicit −1 also ends the list). A section with a higher number has no
   effect, even if it exists. A tileset
   with tile count *C* contributes *C* consecutive tile numbers; a count of 0 is allowed and
   contributes none, and so does a negative count other than −1 (bad data: the importer warns). So
   tile numbers run from 0 to the sum of all positive counts minus 1, in declaration
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
     variant, the gap rule and the digits of tile numbers 1 and 100 need named test files, in its
     fallback, preference, gap and digits parts)
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
   (claim 24). With `quirks.theater_five_forced_unset_roles` on (the default), the original forces seven roles to unset in the theater with index 5, whatever the file says: bridge, cliff, shore, water, water bridge, water cliff and wooden bridge. With this switch off, these roles use their declared values under the ordinary role rules. The clear and water roles also give the fill base tiles of S-0009 (claim 6).
   - Backing, by part:

     | Part | Backing |
     |---|---|
     | A set role starts at the first tile of the tileset it names | `measurable` (scenarios L1, R1 part A) |
     | An absent key, a negative number, or a tileset number beyond the first missing one, leaves the role unset (start −1) | `measurable` (scenario R1 parts B, B2 and C) |
     | A role naming the first missing tileset produces stray tiles that later become no-tile cells | `measurable` (scenario R1 parts D, D2 and E); the exact numeric start equal to the tile count is `unmeasured`, because no proposed observable distinguishes it from another start beyond that count |
     | Theater 5 forces the shore and water-bridge roles to unset | `measurable` (scenario R2: part A for the shore role, part B for the water-bridge role) |
     | Theater 5 forces the water role to unset | `measurable` through S-0009 claim 6 and its scenarios, which own the water fill |
     | Theater 5 forces the cliff, water cliff, bridge and wooden bridge roles to unset | `unmeasured` here: no scenario part isolates these four overrides; the bridge scenarios use set roles in another theater, and cliff effects are outside this spec |
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
     | The general water test and its unset range | `measurable` through S-0406 claim 3 d and scenario PL4, and S-0211 scenario L15 |
     | The other general tests, their unset rule and the counts that only they use (the paved-road count 15 included) | `unmeasured` here: no scenario part isolates these general tests; the tunnel-entrance creation rule of claim 21 has no scenario either |
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
   - Backing, by part:

     | Part | Backing |
     |---|---|
     | The land type and slope shape come from the plain image | `measurable` (scenario V3, with a named test file) |
     | Variants do not affect usable-tile validation | `unmeasured` beyond the valid-plain/missing-variant case of X4: no scenario part supplies an invalid plain tile cell paired with a valid variant cell |
     | The slope shape taken from the tile cell of the sub-tile, and the land type | `measurable` through S-0211's scenario L3, which places ore on every slope shape, and its scenarios L5 and L11 |
     | The radar colours come from the tile cells, not from the theater data file | Claim 20 owns this rule and its backing; S-0410 owns the resulting radar pixels |

   - Confidence: high for plain-image validation and the slope shape. Land-type confidence is
     S-0211's, and radar-colour confidence is claim 20's

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
   particular, a non-overlay-case cell that holds the no-tile marker is flat and plays as clear
   unless an overlay or S-0211's cliff-back rule changes its land type, and a cell whose tile was dropped here holds the no-tile marker from then on, so its next
   recalculation applies S-0211 claim 5 row 2. This settles S-0009's open question for numbers at
   or beyond the tile count in the non-overlay case: such a cell plays as flat ground, with
   its land type subject to S-0211's overlay and cliff-back rules.
   A negative tile number (S-0009 claim 8) gives an unspecified result in the original; our engine
   treats it like the no-tile marker (the departure of S-0009 claim 8), and the importer warns. A
   terrain code outside 0 … 15 is a departure owned by S-0211 claim 6.
   - Backing, by part:

     | Part | Backing |
     |---|---|
     | The no-tile case (tile number at or beyond the tile count, sub-tile beyond the cells, empty tile cell) | `measurable` (scenario C1); a tile with no image is tested in scenario F1's fallback part, run 2 |
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
   none of the above. Resizing the local area in play recalculates every legal cell, not only
   cells gained or lost by the resize (S-0524 claim 8). Each cell still follows its case above.
   The event does not unconditionally restart tile animations (claim 19).
   - Backing, by part:

     | Part | Backing |
     |---|---|
     | Local-area resize and its whole-map terrain recalculation | S-0524 claim 8 owns the trigger, confidence and backing; scenario G4 part G here tests preservation of an existing tile animation, not the exact extent of the recalculation |
     | The tile case is smoothed | `measurable` (scenario L1) |
     | The overlay case is smoothed | `measurable` (scenario C2 part A) |
     | The overlay case starts no tile animation | `measurable` (scenario C2 part C, on the animated tile of scenario G4 part A) |
     | The no-tile case is not smoothed | `measurable` (scenario C1: a smoothed cell would not show the clear tile) |
     | The no-tile case starts no tile animation | `unmeasured`: no proposed scenario gives the displayed clear tile an animation while the cell holds the marker, so the scenarios do not distinguish eligibility from merely lacking an animation name |
     | Tunnel-entrance eligibility in all three cases | `unmeasured`, for the reason of claim 21: no scenario part isolates the creation or absence of an entrance |

   - Confidence: high for the tile case and the no-tile case; medium for the overlay case
     (low for its lack of tunnel entrances)

9. **Cells behind cliffs.** The cliff-back rule (the rules data's cliff-back setting, the six
   neighbour cells, the four-level threshold and the switch `quirks.cliff_back_case_sets`) is
   S-0211 claim 8. This spec adds nothing to it and states no behaviour of its own here; the claim
   exists so that claims of other specs that cite S-0010 claim 9 keep their number.
   - Backing: `measurable`, through S-0211 claim 8 and its scenario L5, which own the rule
   - Confidence: high; the confidence of the rule itself is S-0211 claim 8's

### Smoothing

10. **Ground smoothing.** The mask rule below specifies non-overlapping transition pairs,
    with the unset-role exceptions stated below. For overlapping roles this claim specifies only
    the freshly loaded fixture of scenario R4: a five-cell rough-ground plus on clear ground,
    with the sand and rough transition roles sharing the range *L* … *L* + 15, distinct rough
    and sand ground tiles, and no other pair or ramp range touching those tiles. Its loaded
    centre is the rough ground tile *G* and all four arms are *L* + 15. This result is fixed for
    that fixture, not a choice left to an implementation's update order. Other overlapping-role
    configurations and other pre-load terrain histories are outside this claim.
    For a pair with ground tileset start *G* and transition tileset start *L*, a cell whose tile is *G*, or lies in *L* … *L* + 15,
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
    provided the rough ground tile *G* is not one of tile numbers 0 … 14 and no other role
    interferes, the first load recalculation leaves no rough ground cell unchanged. The predicted
    plus-shaped patch outcomes are those in scenario R1 parts B and B2, and later recalculations
    do not rewrite those cells again. This includes rough ground cells at the map edge, where an
    outside neighbour counts as out (S-0009 claim 44). These are tile-assignment outcomes, not a
    required internal cell-processing order. When the rough transition role names the first missing tileset (claim 4), the
    transition tiles it writes lie at or beyond the tile count (stray tile numbers, Terms), so
    each such cell that is not of the overlay case becomes the no-tile marker at its next
    recalculation (claim 7): at map load the second pass or the game-start pass of claim 12, and in
    play whatever triggers the cell's next recalculation (Terms), the first unit that enters or
    leaves it included. A cell of the
    overlay case keeps the number (claim 8) and, being smoothed, may take another number of the
    same kind; it holds one until the overlay is gone and the cell is recalculated. Until
    then the picture of such a cell is unspecified (claim 24). At the first tick this concerns the
    cells to which the game-start pass itself gave such a number, and the cells of the overlay
    case. Each whole-map pass turns the stray cells it finds into no-tile cells, which count as out
    for their neighbours, so the rewriting goes one layer deeper into a large patch at each pass: a
    plus of five cells holds no stray number at the first tick, while a 4 × 4 block keeps one in
    its third cell along the diagonal (scenario R1, parts D and D2). Apart from these unset-role cases and from roles that name overlapping tilesets
    (the next paragraph), a cell that belongs to a pair still belongs to it after smoothing, so the
    result does not depend on the order in which cells are smoothed, and a further recalculation,
    by a unit's move or otherwise, leaves the tile unchanged. In play, therefore, a unit's moves
    change tiles by this claim only in the one-past-the-end case, in the case of a set transition
    role with an unset ground role, and where roles name overlapping tilesets.
    The overlap fixture specified above is created by R4's named theater data file; it is not
    asserted to occur in stock data. This claim imposes no general ordering among overlapping
    pairs or among recalculated cells. The cell's slope shape and land
    type are not re-read from the new tile in the same recalculation. Cells of the
    no-tile case (claim 8) are not smoothed at all (they hold the no-tile marker, which belongs to
    no pair and no ramp range). An edge neighbour outside the map reads as a blank cell (S-0009,
    claim 44): it holds the no-tile marker, so it belongs to no pair, counts as out for every pair
    (the widened neighbour tests included) and is flat for the ramp smoothing of claim 11.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The mask rule for set roles | `measurable` (scenario L1) |
      | A neighbour outside the map | `measurable` (S-0009's scenario M25 part A) |
      | An unset or one-past-the-end rough transition role's stated final tile and land-type outcomes | `measurable` (scenario R1; its part E for a unit's move as the next recalculation) |
      | Exact stray tile numbers and intermediate erosion states during loading | `unmeasured`: stray pictures are unspecified, and no scenario observes the intermediate assignments separately |
      | The green pair's widened neighbour test and its dependence on the shore and water-bridge roles | `measurable` (scenario R2) |
      | The ends of the widened ranges of the green and pave pairs | `measurable` (scenario B1 parts A and B) |
      | The skipped sand, green and pave pairs | `measurable` (scenario R3) |
      | The stated overlapping-tilesets outcome | `measurable` (scenario R4, with a loose copy of the theater data file); no general rule for other overlaps is claimed |
      | A set transition role with an unset ground role | `unmeasured`: the tile −1 it writes is unspecified in the original (claim 7) |

    - Confidence: high; medium for the overlapping-tilesets outcome, which has not been observed

11. **Ramp smoothing.** A cell whose tile, as ground smoothing left it, lies in the ramp range
    (20 tiles from the ramp start) or the smooth-ramp range (12 tiles from the smooth-ramp start)
    gets a tile chosen anew from its slope shape *s* (the shape it took in this recalculation, from
    the tile it held when the recalculation began). For *s* = 1 … 4, two edge neighbours are
    tested, *a* with weight 1 and *b* with weight 2, as the table below gives; each counts as *flat*
    when its slope shape is 0, and *m* is the sum of the weights of the flat ones. For *s* = 0 or
    5 … 20, *m* = 0. The new tile is smooth-ramp start + 3(*s* − 1) + *m* − 1 when *m* > 0, and
    ramp start + *s* − 1 when *m* = 0 (Formulas, ramp smoothing; the ranges are table R's, claim 5). Because the range test applies to the tile that
    ground smoothing left (claim 10), the order of the two smoothings shows only where ground smoothing writes
    a tile into a ramp range or out of one (roles that name overlapping tilesets, or the unset-role
    cases of claim 10).

    | *s* | *a* (weight 1) | *b* (weight 2) |
    |---|---|---|
    | 1 | W | E |
    | 2 | N | S |
    | 3 | E | W |
    | 4 | S | N |

    So a ramp tile always shows the ramp-range tile of its own shape, or, for edge slopes that meet
    flat ground at their low or high edge, one of three smoothed tiles. A tile of the ramp range or of the smooth-ramp range whose slope shape is 0
    becomes the tile just before the ramp range (ramp start − 1), a tile of another tileset
    (`quirks.ramp_shape_zero`).
    Unlike ordinary non-overlapping ground smoothing, ramp outcomes depend on the neighbours'
    slope shapes at the recalculation event. Claim 12 and scenario L2 state the loaded outcomes,
    without prescribing an internal cell-processing order.

    **Unset ramp roles.** As in ground smoothing (claim 10), an unset ramp or smooth-ramp role's
    range holds only the tile number −1, and the formulas use the unset start −1
    (`quirks.unset_role_range`):

    | Ramp role | Smooth-ramp role | Result |
    |---|---|---|
    | unset | unset | no usable tile is ramp-smoothed |
    | set | unset | a ramp-range tile of shape *s* = 1 … 4 with *m* > 0 becomes 3(*s* − 1) + *m* − 2, one of the tile numbers −1 … 10 of whatever tilesets come first (−1 for *s* = 1 and *m* = 1, which is unspecified from then on, claim 7); the other ramp-range tiles get ramp start + *s* − 1 as usual |
    | unset | set | a smooth-ramp-range tile with *m* = 0 (or of shape 0 or 5 … 20) becomes *s* − 2, one of the tile numbers −2 … 18 (a negative result is unspecified from then on, claim 7); with *m* > 0 it gets its smoothed tile as usual |
    - Backing: `measurable` (scenario L2 for set roles; scenario B1 part C for the ends of the two
      ranges; scenario L3 for the stated non-negative unset-role outcomes; scenario L4 for a
      ramp-range tile of shape 0 (part A) and a smooth-ramp-range tile of shape 0 (part B)).
      Other unset-role results are `unmeasured`: no scenario part isolates them, and negative
      tile results are unspecified in the original.
    - Confidence: high for the rule and for the unset roles; medium for the edge case of shape 0

12. **Terrain results at map-load milestones.** By the time the map's terrain objects and units
    have been placed, every legal cell has undergone one terrain recalculation using the terrain
    stream, tunnel definitions and overlays read from the map (S-0009). By the time the game-start
    recalculation begins, every legal cell has undergone a second recalculation with those objects
    present. Scenario L2 predicts a loaded ramp result in which each neighbour has the shape
    established by the first recalculation: the earlier partial-neighbour result does not remain
    visible, provided the smoothed ramp tiles retain the replaced tile's slope shape. This
    assumption has not been checked for stock data. After the second recalculation, a cell's land
    type reflects the tile held when that recalculation began, normally the tile chosen by the
    first recalculation (claims 10 and 11), even if the second recalculation chose another tile.
    This intermediate land type lasts until the cell is recalculated again, no later than the
    game-start recalculation. The skirmish setup's placement of starting units reads this
    intermediate terrain; the backing for that interval remains `unmeasured` as given below.
    The unused-house cleanup follows the second whole-map recalculation and precedes the late
    start-position phase (S-0009 claim 28). Removing grounded objects can recalculate their
    occupied cells, including a building's foundation cells, and start a missing tile animation
    there; its draws belong to cleanup, not to either whole-map pass (S-0202 claim 14).

    **Game-start results.** After starting-unit setup, initial crates, secret-lab choices and
    end-of-loading tags (S-0412 claim 14), but before the first tick, every legal cell has
    undergone one further terrain recalculation. This position is unchanged when starting-unit
    setup places no units or stops early. At the first tick, a cell's land type reflects the tile held when this
    recalculation began—the tile chosen by the second load recalculation if nothing recalculated
    the cell in between—and remains so until the cell's next recalculation. The tile chosen by the
    game-start recalculation supplies its land type only from then on. Cleanup, starting-unit
    placement and animation effects can cause intervening recalculations; claims 7 and 8 and S-0211 apply
    to each event, including the overlay and unusable-tile exceptions. A stray tile number written
    by the second load recalculation is the no-tile marker by the first tick unless the cell is
    of the overlay case (claim 7). This recalculation may start tile animations (claim 19), and
    bridge spans (claim 25) are found after these game-start results are established. In play, a
    recalculation uses the neighbours' shapes that are current at that event. Resource-glint
    setup follows the game-start recalculation and can create further animations and take
    further scenario-stream draws before tick 0 (S-0202 claim 14). Glints are not tile
    animations; this pass is not an exhaustive final-draw boundary for loading.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The final loaded ramp outcome, with shape-preserving tiles | `measurable` (scenario L2); the earlier partial-neighbour state itself is `unmeasured`, because no screenshot during loading shows it separately |
      | The land type between the second pass and the game-start pass | `unmeasured`: only the skirmish setup's placement of starting units reads it, and the tiles of the two passes differ only in rare smoothing cases that no starting position is known to meet |
      | The game-start terrain and animation outcomes | `measurable` (scenario R1 part D, whose plus centre plays as clear land at the first tick, and scenario G4 part C, whose animation is present at the first frame); the exact count of whole-map recalculations is `unmeasured`, because these final outcomes do not distinguish extra recalculations once the terrain is stable |
      | At the first tick, without an intervening recalculation, the land type reflects the second pass's tile | `measurable` (scenario R1 parts D2 and E, through the cell that holds a stray tile number at the first tick and still plays as the land of the tile the game-start pass found there, until its next recalculation) |
      | Cleanup's position and conditional tile-animation starts | S-0202 claim 14 owns the rule and its `unmeasured` backing: no discriminating no-computer-player skirmish fixture is established |
      | Game-start pass after crates, secret labs and end-of-loading tags, including no-unit and early-stop cases | `measurable` through S-0202 scenario R10 variants A and B |
      | Subsequent resource-glint setup | S-0202 claim 14 owns the rule, confidence and backing, with scenario R13 |
      | The two load milestones and the first pass's results being in place before the map's terrain objects and units are placed | `unmeasured`: the proposed screenshots do not count recalculations or isolate object-placement effects during loading (the open question below asks whether placing map objects recalculates their cells) |

    - Confidence: medium (the three passes and their order are certain; that the smoothed ramp tiles
      keep their shapes in stock data is not checked); low for the placement order

### Variants

13. **Choice of the image.** A cell whose tile has *n* ≥ 1 variants shows image *i* = *P* mod
    (*n* + 1), where *P* comes from a pattern indexed by a position (*X*, *Y*):
    - with *n* + 1 ≤ 4 images, the fixed 4 × 4 pattern of table P4 at (*X* mod 4, *Y* mod 4),
    - with *n* + 1 > 4 images, the 8 × 8 pattern of claim 14 at (*X* mod 8, *Y* mod 8).

    For every sub-tile *t*, including 0, of a tile whose plain image is *w* cells wide and *h*
    high (the sizes of the plain image, whichever image is shown), *X* =
    (x − (*t* mod *w*)) / *w* and *Y* = (y − ⌊*t* / *w*⌋) / *h*, divided with truncation toward
    zero: the coordinates of the tile's origin cell divided by the tile's size. Thus every cell
    of one placed tile uses the same pattern position. This is an unconditional drawing
    correction, with no quirk switch. The original instead uses the cell's own coordinates for
    sub-tile 0, which can make that cell show a different variant from the rest of its tile.
    A tile cell whose file carries the *alternate-image mark* (a
    yes-or-no mark in the tile file, taken from the plain image's tile cell at the cell's
    sub-tile) does not use the pattern: it shows image 1 when the map cell's own *alternate-image
    state* is set and image 0 (the plain image) when it is clear. The state is a yes-or-no value that
    each map cell holds, clear at map load; it is used by tiles that change in play. The bridge
    damage and repair events of S-0511 claims 4 and 19 are examples of alternate-look changes.
    This spec fixes only what the cell shows for each value of the state. For a tile with only one image the mark has no effect: the
    cell shows the plain image.
    - **Where the mark is read.** Only the plain image's file counts: the mark is that of the plain
      image's tile cell at index *t* mod (*w* × *h*) of the plain image (the bit of value 4 of its
      flag value, Terms). The tile cells of the variant files are never consulted for it, whatever
      they carry. Where the plain image has no tile cell at that index, the cell has no mark and
      uses the pattern. A cell that holds the no-tile marker shows the clear tile by the pattern
      (claim 24); no mark is read for it.
    - **Image count.** The count *n* + 1 that selects the pattern is the number of images of the
      tile, the plain image included (claim 3): a tile with one to three variants (two to four
      images) uses table P4, and a tile with four or more variants (five or more images) uses the
      8 × 8 pattern of claim 14, never table P4.

    The choice never reaches the cell's land type or slope shape, which come from the plain image
    (claim 6). It decides which image is drawn and also which tile cell's radar colours the radar map
    takes for the cell (S-0410, the tile colour of the image variant the cell shows).
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The choice for tiles of one cell | `measurable` (scenario V1 part A) |
      | The original's mismatched choice for larger tiles | `measurable` (scenario V1 part B); our engine's uniform position is a high-confidence design definition tested by that part's engine comparison |
      | The choice never reaches the land type or slope shape | `measurable` (scenario V3) |
      | The image of a cell without a usable tile | `measurable` (scenario C1 part B) |
      | The origin coordinates of a tile placed beyond the map's low edges (a negative origin, with truncating division) | `unmeasured`: a cell that lies so near the map's edge is outside the playable area of S-0009 and shows nothing a measurement can compare; the formula only states the arithmetic |
      | A marked tile cell shows the plain image after map load (the state clear at load), the mark read from the plain image's file only, and the bit that carries it | `measurable` (scenario V4, named test files) |
      | Image 1 while the state is set | `unmeasured` here: V4 tests the clear state only; bridge damage and repair change tile looks under S-0511 claims 4 and 19, but no scenario part here isolates a marked tile cell in the set state |
      | The mark having no effect on a one-image tile | `unmeasured`: a one-image tile shows the same picture with either image-selection rule |

    - Confidence: high for tiles of one cell and the defined correction, medium for the original's
      larger-tile choice, low for the negative origin,
      high for the bit that carries the mark and for its being read from the plain image's file
      only, and low for what the set state shows

14. **The 8 × 8 pattern.** The original builds the 8 × 8 pattern at the first image choice of claim 13
    in a session, whatever the number of images of the tile it is made for, and keeps it for the
    rest of the session. The choice is made for every cell whose tile has two or more images and for
    every cell without a usable tile (which shows the clear tile), once for each at the end of every
    map load and again whenever such a cell is drawn. So the pattern is built while the first map of
    the session loads, unless that map has no such cell: a map whose cells all hold one-image tiles
    (only a map built for the purpose) builds it at the first later moment that such a cell is
    drawn or loaded, and then the draws fall in the middle of play. The pattern's entry at (*X*, *Y*) has index
    *k* = *X* + 8*Y*, and the 64 entries are the function of the draws from the unsynchronised
    random stream that the formula *8 × 8 pattern* gives (Formulas). Consequences: every entry is
    one of 0 … 7; no entry holds the same value as any of its 8 neighbours in the index-wrapped
    sense of that formula, except where all 64 candidates for an entry clashed; building the
    pattern takes between 64 and 4096 consecutive draws from the unsynchronised stream, all at one
    point of the session; and the pattern depends on how many draws that stream has given before.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The pattern's properties, and its being kept for the session | `measurable` (scenario V2 part A) |
      | The consecutive draw segment and its candidate/retry formula | `measurable` with a recorder that reads the unsynchronised stream (scenario V2 part B), an extension of the state recorder that S-0202 describes, approved as part of the measurement setup |
      | The precise first-image-choice trigger during loading | `unmeasured`: V2 part B reads the final stream state and image pattern, not the exact event at which the pattern was built |
      | A first map without any such cell: the pattern is built at a later moment | `unmeasured`: the recorder of part B reads the stream at the first tick of a game, not at the moment of a later draw, and real maps always hold clear cells with variants |

    - Confidence: high; low for the later-moment case

### Tileset flags

15. **Tilesets with the morphable flag, and to-tile buildings.** The tileset condition of
    S-0406 claim 3's to-tile rule passes for a tested cell whose tile belongs to a tileset with the
    morphable flag. S-0406 owns when this condition is asked and the requirement for only one
    qualifying foundation cell, not every cell. The test passes, whatever the tilesets say, for every cell whose tile number lies outside
    0 … tile count − 1: one that holds the no-tile marker, one that holds a stray tile number
    (Terms) and one that holds a negative tile number. So for such a cell the picture it shows
    (claim 24) does not matter to the test. Other conditions of placement belong to S-0406
    claim 3's to-tile rule; what a to-tile building does once placed is not covered here.
    The morphable flag also constrains where destruction smudges can be placed; S-0312 claim 26
    owns that rule, its footprint test and its backing. This is a separate use of the flag, not
    the to-tile placement test above.
    The original also uses the flag in its random map generator.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The tilesets, and the no-tile marker | `measurable` (scenario G1, parts A and B, with named test data) |
      | A stray tile number | `measurable` (scenario G1 part C, on the cell of scenario R1 part D2 that holds one at the first tick until it is recalculated) |
      | A stray tile number of an overlay-case cell | `unmeasured`: whether a building may stand on a cell that holds a row-1 overlay is decided by rules outside this spec, so the tile test cannot be told apart from them |
      | A negative tile number | `unmeasured`: a cell comes to hold one only through results that claims 7, 10 and 11 call unspecified or leave `unmeasured` (map data, the tile −1 of claim 10, the negative results of claim 11's unset-ramp table), so no state to test is practical |

    - Confidence: medium (low for the negative number)

16. **Where new tiberium can appear.** A cell can receive newly appearing tiberium (an empty cell becoming
    a tiberium cell) only if, among other conditions that belong to the economy specs, its land type
    is buildable (the land type's buildable field, S-0006), it holds no overlay, its slope shape is 0,
    and its tile belongs to a tileset with the allow-tiberium flag. As in claim 15, the tileset
    condition holds, whatever the tilesets say, for every cell whose tile number lies outside
    0 … tile count − 1: one that holds the no-tile marker, a stray tile number (Terms) or a
    negative tile number. A cell that holds an overlay fails the
    overlay condition whatever its tile, so a stray number matters here only for a cell that is not
    of the overlay case, that is, for one that smoothing has just written one to (claims 4 and 10).
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The tilesets, and the no-tile marker | `measurable` (scenario G2, parts A and B) |
      | The other conditions listed (buildable land type, no overlay, slope shape 0) | `measurable` (scenario G2 part C); S-0401 claim 10 owns the full germinability test |
      | A stray tile number | `unmeasured`: no proposed fixture preserves R1 part D2's transient stray cell next to a mine through loading; placing the extra terrain object can itself affect the recalculations that determine the survivor |
      | A negative tile number | `unmeasured`: the original's result on such a tile is unspecified (claim 7), so no stable germination fixture is established |

    - Confidence: medium for tileset eligibility and the other conditions (S-0401 claim 10 owns
      the full germinability test); low for the stray and negative numbers

17. **Burrowing.** The *burrowing test* of a cell decides whether a tunnelling unit may dig in or
    come up on it. A cell outside the playable area of S-0009 (claim 4, tested with the
    *height-aware* test of that claim, in which the cell's height level shifts the edges of the
    area) passes, whatever it holds. A cell inside that area passes exactly when all of these hold:
    - Its tile belongs to a tileset with the allow-burrowing flag (absent: allowed). As in claims 15
      and 16, this condition holds, whatever the tilesets say, for every cell whose tile number lies
      outside 0 … tile count − 1: the no-tile marker, a stray tile number (Terms) and a negative
      tile number.
    - It is flat: its slope shape index (S-0106) is 0.
    - It is not a high-bridge cell (S-0211 claim 15) and does not carry the bridge-body mark (Terms,
      claim 25). So the cells of a low bridge's span and of a band beside it fail (claim 25), while
      the cells holding a bridge's two end tiles fail only when one of the other conditions refuses
      them (for example when their tile is sloped) or another span's band covers them.
    - Its ground layer holds no building and no terrain object. Buildings and terrain objects on a
      bridge deck do not count, nor do units of any kind.

    Nothing else enters the test: not the cell's land type, not its overlay (a wall, ore or a bridge
    overlay itself), not the cell's height level beyond the area test, and not the side that owns a
    building. The vehicle specs own what a tunnelling unit does with the answer, and S-0212 uses the
    same test as the *further test* of its nearby-cell search (its claim 25).
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | A cell of a tileset without the flag refuses, one with it accepts (inside the area) | `measurable` (scenario G3, parts A and B) |
      | An absent key allows | `measurable` (scenario G3 part C, with a loose copy of the theater data file) |
      | A cell without a usable tile number passes the tileset condition | `measurable` (scenario G3 part F, the no-tile marker); for a stray or negative number `unmeasured`, for the reasons given in claim 16 |
      | Sloped cells, high-bridge cells and cells with a building or terrain object on the ground layer refuse | `measurable` (scenario G3 part D) |
      | The cells of a low span and its band refuse, the end cells and the cells just outside the band accept | `measurable` (scenario G3 part E) |
      | The bridge-body mark that a destroyed high bridge sets | `unmeasured` here: when a bridge is destroyed, and which cells its destruction touches, belong to the bridge specs (claim 25) |
      | The ore overlay and rough land type play no part | `measurable` (scenario G3 part D's two controls) |
      | Walls, other overlays and land types, deck-layer objects, units and owner-independent building refusal | `unmeasured`: G3 part D does not isolate these cases from other destination and movement rules |
      | The answer outside the area | `unmeasured`: a unit cannot be ordered to a cell there, and what it would do is the vehicle specs' |

    - Confidence: high for the list of conditions (it is complete as read); medium for how the
      vehicle specs use it

18. **Shadow-casting tilesets.** A tileset casts shadows (S-0104 claim 13) only when both its
    shadow-caster flag is set and its shadow-tile count is not 0. With the flag unset the count has
    no effect; with it set, a count of 0 (or an absent count) disables the shadow (Import mapping).
    Which cells each tile of such a tileset marks, by the tile's position within the tileset, is
    S-0104 claim 19; tiles at some positions mark none.
    - Backing: `measurable` (scenario G5, which runs S-0104's scenario L15 part D)
    - Confidence: high

19. **Tile animations.** A tile may name an animation, with a screen displacement (*dx*, *dy*) in
    pixels, a sub-tile it attaches to and a depth adjustment. These keys are not in the tileset
    section: they are read from a separate section of the theater data file whose name is the
    tileset's set name, and when the animation name is empty or absent the other four keys have no
    effect (Import mapping). Only a tile's plain image carries them; variants (claim 3) do not. A
    tile whose attaches-to key is absent never starts its animation. An animation type named there
    that the rules data does not list is created by that reference (S-0004).
    The once-only rules below exclude animations whose type has the vein-animation flag.
    Their removal may permit another start, an unresolved case listed in Open questions.
    - **When.** At terrain recalculation, a cell of the tile case (claim 8) starts
      an animation when the tile it held as the recalculation began names one and the cell's
      sub-tile equals that tile's attaches-to value. The animation is that tile's, not that of a
      tile that smoothing chooses in the same recalculation (claims 10 and 11). From the second pass
      of claim 12 until the next map load, a cell starts at most one animation: once it has started
      one in that time, a later recalculation starts no second one, however the tile has changed
      since. A cell that has started none can start one at
      any later recalculation in which the tile it holds at the beginning qualifies, for example a
      tile that smoothing left in the cell at an earlier recalculation. Immediately after the
      second pass, the tile animations of this claim that remain are exactly those that this
      pass starts: at most one per cell, that of the tile the first pass left in the cell. None
      that the first pass started survives loading, even in a cell where the second pass starts
      none. Subsequent cleanup or placement recalculations can start a missing animation under
      the same rule (claim 12; S-0202 claim 14). The game-start pass then starts one in each
      cell that has started none since the second pass and whose tile at that event qualifies.
      Without an intervening recalculation, this is the tile the second pass left (scenario G4
      part C). At the first tick a cell has started at most one tile animation since the second
      pass, whether in that pass, an intervening recalculation or the game-start pass.
      A local-area resize in play (S-0524 claim 8) preserves this once-only state: an existing
      tile animation is not removed or restarted by the resize. A cell that has started none
      may start one only when the ordinary eligibility conditions above hold.
      A tile animation started while the map loads is subject to the same creation-time rule as
      any other animation: if its middle image frame μ is 0 (μ is ⌊*N* / 2⌋ of the sprite that
      S-0109 claim 25 names), its middle event happens at creation, not at its first update
      (S-0109 claim 14).
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
      map loading too. In the game-start pass these creations occur in visiting order,
      after starting units, crates, secret labs and end-of-loading tags (claim 12; S-0202
      claim 14 and scenario R10). Low-bridge markers and their direct end-row and piece
      writes start no overlay cell animations, but their cell recalculations can start tile
      animations. S-0202 claims 10 and 14 own these exclusions and the piece-choice draw
      preceding its tile-animation draws, with scenarios R8 H and R12.
    - **How it is created.** When a qualifying terrain recalculation starts a tile animation, its start event occurs immediately, including during map loading. The animation runs the number of passes specified by its type's loop count, without an additional repeat factor. It does not start from a trigger. Its start event therefore has its start sound and resource-chain reaction. Every effect of the start event, and the middle event that it produces when μ = 0 (S-0109 claim 14), occurs at the position given under **Where**: the chain reaction acts on the cell containing that position, which may differ from the tile cell when the displacement crosses a cell edge, and the height test of S-0109 claim 14 uses that position's height above the terrain. The start events of animations started in the first pass of claim 12 also occur, and their effects remain after those animations are removed. S-0109 and the specs for those effects own their details and draws. S-0202 claim 15 owns immediate crater timing: a tile animation started by overlay placement's recalculation acts before the overlay data stream is read, whereas one first started in the first whole-map pass acts on the imported data, and the second and game-start passes act later still. The data stream overwrites overlay data but does not restore an overlay type exhausted by an earlier crater request.

    These are S-0103's and S-0104's *terrain animations*; how they play belongs to S-0109 (animation
    playback).
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | An animation starts on the attaches-to cell of a stock tile, at the displaced position | `measurable` (scenario G4 part A) |
      | The animation is that of the tile held when the second pass began, not of the smoothed tile | `measurable` (scenario G4 part B) |
      | The game-start pass starts the animation of the tile the second pass left, in a cell that has started none | `measurable` (scenario G4 part C, run 1) |
      | A start in play, after smoothing, in a cell that has started none | `unmeasured`: after the game-start pass, the tiles that smoothing writes in play in the known cases are stray tile numbers or tiles a cell already held, so no setup is known in which a cell comes to hold, in play, a tile that names an animation |
      | Local-area resize does not remove or restart an existing tile animation | `measurable` (scenario G4 part G); a previously missing animation starting at resize remains `unmeasured`, for the same missing-fixture reason as a start in play above |
      | None that the first pass started survives loading, and at most one animation per cell after loading | `measurable` (scenario G4 part E) |
      | A cell that has started one starts no second one in later recalculations | `measurable` at the game-start pass (scenario G4 part C, run 2); in play `unmeasured`, for the reason of the row on starts in play |
      | A displacement far larger than the view | `measurable` (scenario G4 part D) |
      | The boundary of that case | `unmeasured`: G4 part D tests only displacements far beyond the view, not either boundary |
      | The depth adjustment and animation fields' absent-key and plain-image-only rules | `unmeasured`: no scenario part varies these independently; G4's examples use explicit attaches-to values and standard depth |
      | The creation draws | `measurable` through S-0202's scenario R8, part F |
      | Conditional cleanup starts and low-bridge exclusions of cell animations, not tile animations | S-0202 claim 14 owns the rules, confidence and backing: cleanup is `unmeasured`; bridge exclusions are `measurable` through R12, with R8 H for a piece's tile animation |
      | The creation parameters: the start event at creation, at the displaced position, with the chain reaction in the cell under that position; passes from the type's loop count | `measurable` (scenario G4 part F, named test data) |
      | The start events of first-pass animations keep their effects | `measurable` (scenario G4 part F, run 2) |
      | Immediate crater timing relative to the overlay data stream and recalculation passes | S-0202 claim 15 owns the rule, confidence and `measurable` backing (its scenario R14 part C) |
      | A tile animation's middle event at creation, when its middle image frame μ is 0, during map loading | S-0109 claim 14 owns the rule, its confidence and its backing |

    - Confidence: high for when an animation starts at map load and at the game-start pass, for the
      position formula and for the creation parameters, medium for a start in play after smoothing (part C) and for the displacement far larger than
      the battlefield view (part D), and low for the exact boundary of that case and for the rule
      of at most one animation per cell in play

20. **Fields read without effect in play.** The tileset's allow-to-place, height-view and
    non-height-view fields (apart from claim 3) and its theater-conversion fields are read and have
    no effect on a game as far as found; the set name only names the tiles for the to-tile lookup of
    claim 15 and the section of the tile animations of claim 19; the required-for-random-maps flag
    only affects the random map generator. The general section's ice roles, train-bridge role and
    alternative cliff roles, and the tilesets' radar colours, are not read at all.
    The tile cells' two colours are different data: their first tile radar colour supplies the
    radar's tile colour, from the image variant the cell shows (claim 13), while their second
    tile radar colour has no effect on the radar. S-0410 claim 9 owns the brightness scaling,
    halving and fallback colour; the two stored colours do not give the cell's two radar points
    different colours.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | The first tile radar colour is used, the second is not, and the variant supplies it | `measurable` through S-0410 scenario R6 parts A and D, with named test tile colours; use the ordinary pattern of claim 13 to select variants, not an unspecified way of setting the alternate-image state |
      | The allow-to-place and theater-conversion fields have no effect on building placement or on the pictures of tiles | `measurable` (scenario F1, spot-check part: changed for one tileset, nothing may differ) |
      | The height-view field and the non-height-view field's other uses, the set name's other uses, the random-map flag, and the keys that are not read at all | `unmeasured`: an absence of effect on every part of a game cannot be shown by a finite set of measurements, and no observable exists for the random map generator |
    - Confidence: high for the keys that are not read at all; medium for the fields that are read
      and have no effect, for the tile radar colours' use, and for the set name and the random-map flag

21. **Tunnel entrances.** A cell of the tile case (claim 8) whose land type is
    tunnel, whose tile is one of the first four tiles of one of the four tunnel roles and which has
    no tunnel entrance yet gains a tunnel entrance at terrain recalculation. S-0212 owns tunnel
    links and their pathfinding use; this claim concerns entrance creation only. The tunnel roles are not
    checked (claim 5): an unset tunnel role covers tiles 0 … 2, so a tunnel-land cell holding one of
    them also creates an entrance (`quirks.unset_role_range`).
    - Backing: `unmeasured`: no scenario part isolates entrance creation or its unset-role case
      from tunnel definitions and pathfinding outcomes
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
      rule (the formula is S-0102's; that the extra image takes its diamond's reference row is
      inferred, not observed, and scenario X2 part D would confirm it)

24. **Which tile cell is painted, and when nothing is.** Which image a cell shows is claim 13's.
    Let *t* be the cell's sub-tile and *N*<sub>s</sub> the number of tile positions, width times
    height, of the *shown image*. A variant's file has its own size, so *N*<sub>s</sub> can differ from the
    plain image's *w* × *h*. The tile cell that is painted is counted row by row from the
    top-left of the shown image, empty positions included (claim 6).
    - **Tile case (claim 8).** Subject to the missing-cell correction below, the cell paints the
      tile cell at index *t* mod *N*<sub>s</sub> of the shown image. By claim 7, *t* is below the plain image's *w* × *h*, so the index is *t* itself
      when the shown image is the plain one or any variant with at least as many positions as the
      plain image; a variant with fewer positions wraps the index.
    - **No-tile case.** A cell holding the no-tile marker paints position 0 of the first tile of the
      clear role (S-0009 claim 8, "the theater's clear tile"). The image is the one that claim 13
      gives for that tile with the position (*X*, *Y*) = (x, y), the cell's own coordinates, as for
      sub-tile 0: the 4 × 4 pattern when the clear tile has at most 4 images, the 8 × 8 pattern
      when it has more. The choice of claim 13 is made for the cell even when the clear tile has one image, with no
      visible effect (it still builds the 8 × 8 pattern, claim 14). The cell's height level lifts the picture as claim 22 says.
    - **Overlay case (claim 8), tile number below the tile count.** The cell paints as in the tile
      case, including the missing-cell correction: the tile cell at index *t* mod *N*<sub>s</sub>, so a sub-tile at or beyond the cell count
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
    - **Missing tile cells.** When the selected variant has no tile cell at its selected index,
      our engine paints the plain image's tile cell at *t* mod (*w* × *h*) instead, with that
      tile cell's diamond, extra image and depth data. This is an unconditional drawing
      correction with no quirk switch; it changes no land type or slope shape. It corrects the
      terrain picture only: the selected variant still supplies the radar colour, or S-0410
      claim 9's dark-grey fallback when its tile cell has none. The original
      paints nothing for a missing variant cell, leaving stale terrain pixels (unspecified on
      the first painting). If the plain image too has no tile cell, or the overlay-case tile has
      no image at all, our engine paints no diamond, extra image or depth there and leaves the
      uncovered terrain background black. The original likewise paints nothing in these
      plain-image cases; such a case occurs only with an overlay (claim 7).
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
      | The original paints nothing at a variant's empty position | `measurable` (scenario X4 part A; named test file); plain-image fallback in our engine is a high-confidence design definition tested by that part's engine comparison |
      | A variant with fewer positions wraps the index | `measurable` (scenario X4 part B; named test files) |
      | The same index rule in the overlay case, and nothing painted there for a tile without an image | `unmeasured`: the overlay hides the ground, and a cell that paints nothing shows pixels that are unspecified on a first painting |
      | The height value is unused | `measurable` (scenario X3; named test file), the random map generator part `unmeasured` (not specified) |
      | The picture of a cell that holds a stray tile number | `unmeasured`: the original's result is unspecified, so there is nothing to compare |
    - Confidence: high for the tile case, the no-tile case, the unused height value in drawing
      and the defined missing-cell correction. Medium for the original's wrap of a smaller
      variant, empty position of a variant and overlay case.

### Bridge spans

25. **Bridge spans and the bridge-body mark.** Once at game start (glossary), right after the
    game-start pass of claim 12 and before the first tick, the original finds the map's bridge spans (Terms) from the
    tiles of the bridge role and of the wooden-bridge role (claim 4, table R; 16 tiles each), and
    gives the bridge-body mark to the cells of every low span. Which spans exist and which cells
    receive the body mark do not depend on the order in which cells are looked at.
    - **Start cell.** A cell whose tile is tile *i* of the bridge role or of the wooden-bridge role
      (*i* = tile number − the role's start, 0 … 15) and whose sub-tile equals the start sub-tile of
      table B for *i*. Its span runs in the direction table B gives: east (towards *x* + 1) or south
      (towards *y* + 1). A tile in the bridge role's range is judged by the bridge role's *i* alone,
      even when it also lies in the wooden-bridge role's range. An unset role has no tiles here: unlike
      the general tests of claim 5, these tests never count from the unset start, so
      `quirks.unset_role_range` does not reach them.
    - **Far end.** The far end is the nearest cell beyond the start cell in its direction (along
      the start cell's row for east, along its column for south) whose tile is tile *j* of the
      bridge role, or (for a tile outside the bridge role's range) tile *j* of the wooden-bridge
      role, with the end sub-tile of table B for *j*. The two roles may mix: a span can start on one
      role's tile and end on the other's. When a cell that is not a legal cell of the map (S-0009
      claim 1) comes before such a cell in that direction, or there is no such cell, the start cell
      has no span. Several spans may share cells, and the far end of one span can be the start cell
      of another.
    - **High or low.** The cells strictly between the two ends whose tiles lie in neither role's
      range decide. When every one of them is a high-bridge cell (S-0211 claim 15) as it stands
      at that moment, the span is high; when at least one is not, the span is low. A span without
      such a cell is high. Between-cells whose tiles lie in one of the two ranges play no part.
    - **Band of a low span.** For a low span, each cell strictly between the two ends that is not a
      high-bridge cell sets the bridge-body mark on four cells across the span: for a span running
      east, the cell itself, the two cells beyond it towards *y* − 1 and the one cell beyond it
      towards *y* + 1 (same *x*); for a span running south, the cell itself, the two cells towards
      *x* − 1 and the one cell towards *x* + 1 (same *y*). A mark that would fall on a cell that is
      not a legal cell has no effect. The two end cells themselves get the mark only when the band
      of another span covers them. A high span sets no mark. Each of the four band cells also
      gets the bridge axis of S-0212's Terms: along cell x for a span running east, along cell y
      for one running south, including any high-bridge cell the band covers. Where several
      spans' bands cover a cell, the span whose start cell comes first in S-0009's visiting order
      decides its axis.
    - **Bridge links.** These spans, with their two end cells, are also the bridge links that S-0212
      uses (its claim 3 and claim 13b), a high span being a link that is enabled at game start and
      a low span one that is not. The tunnel links of S-0212 are not spans.
    - **In play.** For each high-bridge overlay destroyed, all four cells of S-0211 claim 15
      receive the bridge-body mark as their high-bridge marking is cleared: the cell one step
      against the marking direction, the overlay's own cell, and the cells one and two steps
      along that direction. Repair clears the bridge-body mark on those same four cells and
      restores their high-bridge marking. S-0511 claims 5 and 19 own which cells are unmarked
      and re-marked during destruction and repair; their connectivity changes and the cells
      named for them are S-0511 claims 12 and 19. These pointers settle neither passenger
      escape placement nor exceptional repair-cursor selection, which remain open in S-0511.
      When a destruction or repair finds no bridge link within 3 cells of
      its named cell, the spans are found again from the map's current tiles and high-bridge
      markings by the start, far-end and high-or-low rules above. This replaces the bridge links
      as S-0212 claim 3 states, but does not set the low-span bands again: neither their body
      marks nor their axes are reapplied by that search. S-0212's Terms own the distance test
      meant by within 3 cells.
    - Backing, by part:

      | Part | Confidence | Backing |
      |---|---|---|
      | Start cells, qualifying far ends and directions by table B, and the band of a low span | high | `measurable` (scenario G3 part E and its table-coverage repeats, named test maps) |
      | High and low spans: a high span marks no band | medium | `measurable` (scenario G3 part E, run 2) |
      | Nearest qualifying far end when several compete | high | `unmeasured`: the repeats provide only one qualifying far end per tested start |
      | A span without any non-role between-cell is high | medium | `unmeasured`: G3 part E always has non-role between-cells, so it does not isolate this empty-between case |
      | The spans are S-0212's bridge links, and a high span is an enabled one | medium | `unmeasured` here: this spec proposes no path-search probe that distinguishes link state from ordinary passability; S-0212 claim 3 owns their use |
      | Bridge-role priority over an overlapping wooden-bridge role | medium | `unmeasured`: the table-coverage repeats keep the two role ranges separate |
      | Mixed-role spans, a direction that leaves the map before a far end, and an unset role | medium | `unmeasured`: no scenario part proposes these configurations; the repeats use separate set roles with matching ends inside the map |
      | The body mark set on all four cells by destruction and cleared by repair | medium | `unmeasured` here: S-0511 claims 5 and 19 own the affected cells; burrowing refuses both an intact deck and its destroyed body, so that test does not distinguish these marks |
      | The band cells' bridge axis | medium | `unmeasured` here: no proposed axis-reading pathfinding probe isolates it from other bridge rules; S-0212 claim 13b owns the axis's use |
      | Earliest-start priority of overlapping bands | medium | `unmeasured`: no scenario part creates overlapping bands with different axes and isolates an axis-dependent outcome |
      | Body membership independent of span enumeration, end cells covered only by another band's marks, and bands clipped to legal cells | high | `unmeasured` beyond G3 part E's isolated interior span: no scenario part tests shared ends or clipping at the map boundary |
      | Spans found again in play without reapplying low-span bands | medium | `unmeasured`: as S-0212 claim 3 states, stock play gives no known way to make the destruction or repair lookup find no link |

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

> (*X*, *Y*) = (trunc((x − (*t* mod *w*)) / *w*), trunc((y − ⌊*t*/*w*⌋) / *h*)) for every *t*, including 0
>
> *P* = P4(*X* mod 4, *Y* mod 4), the entry of table P4 in column *X* mod 4 and row *Y* mod 4, if
> *n* + 1 ≤ 4; else *P* = P8<sub>*k*</sub>, the entry of the 8 × 8 pattern with
> *k* = (*X* mod 8) + 8·(*Y* mod 8)
>
> image = *P* mod (*n* + 1). Integers; trunc rounds toward zero; mod is the non-negative
> remainder. Unit: image index.
>
> Exception (claim 13): a cell whose plain image's tile cell at index *t* mod (*w* × *h*) carries the
> alternate-image mark shows image 1 when its alternate-image state is set and image 0 when it is
> clear, with no pattern (for *n* ≥ 1).

The origin coordinates x − (*t* mod *w*) and y − ⌊*t*/*w*⌋ can be negative for a multi-cell tile
placed so that its origin would lie beyond the map's low x or low y edge. The formula covers that
case as written: the division truncates toward zero, and *X* mod 4, *X* mod 8 (and the same for *Y*)
are non-negative remainders. The original uses this arithmetic only for *t* ≠ 0 and instead
uses (*x*, *y*) for *t* = 0 (claim 13's drawing correction).

**Tile animation position (claim 19).** For a cell (x, y) at height level ℓ whose tile has the
displacement (*dx*, *dy*) in pixels:

> X = 256·x + 128 + trunc₀( *a* · (*dx* + 2·*dy*) ),  Y = 256·y + 128 + trunc₀( *a* · (2·*dy* − *dx*) ),
> *z* = 104·ℓ

in leptons, where *a* and trunc₀ are those of S-0101's F6 (the screen-to-ground formula applied to
the world pixel (*dx*, *dy*), with no view origin added). The number domain and rounding are F6's:
the original uses single-precision inputs and coefficients, exact products, a double-precision
sum rounded toward zero, then a single-precision result rounded toward zero before truncating to
an integer, as F6 states. The program-wide round-toward-zero mode is the glossary's; its switch
`quirks.chop_rounding` belongs to S-0205, number domains. F6 shows that the result equals the exact
rational value truncated toward zero. In the original the two trunc₀ terms are
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

**B. Bridge span tiles** (claim 25). *i* is the tile's place in its role's range (tile number − the
role's start), the same for the bridge role and the wooden-bridge role. "—" means the tile is never
a start cell, or never a far end. Sub-tiles are as in the glossary (index *t* with column *t* mod *w* and row ⌊*t* / *w*⌋,
as in claim 13); east is towards *x* + 1, south towards *y* + 1. Unit: sub-tile indices.

| *i* | Start sub-tile | Direction of the span | End sub-tile |
|---|---|---|---|
| 0, 1 | 7 | east | — |
| 2 | — | — | 4 |
| 3, 4 | 7 | south | — |
| 5 | — | — | 2 |
| 6 … 10 | 4 | east | 4 |
| 11 … 15 | 2 | south | 2 |

**Other constants.**

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Tileset section number width | 4 | decimal digits | Zero-padded, from 0 |
| Tile number within a set in file names and animation keys | at least 2 | decimal digits | From 1; tile 100 has 3 |
| Transition tiles | 16 | tiles | Per pair |
| Transition pairs | 4 | pairs | Rough, sand, green, pave; claim 10 fixes only R4's loaded overlap outcome, not general overlaps |
| Variant patterns | 4 × 4 and 8 × 8 | entries | Claims 13 and 14 |
| Largest image count that uses table P4 | 4 | images, the plain image included | Claim 13 |
| Alternate-image mark | bit of value 4 | flag value of a tile cell | Claim 13, Terms |
| Band of a low bridge span | 4 | cells across the span: 2 to the low side, the cell, 1 to the high side | Claim 25 |
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
- The map-load and game-start milestones of claim 12 have these consequences; they do not
  prescribe an internal cell-processing order. Rows concerning placement milestones have
  claim 12's `unmeasured` backing:

  | Pass | Works on | Its results are in place when | Consequences |
  |---|---|---|---|
  | First | the terrain stream (S-0009 claim 7), the map's tunnel definitions (S-0009, read set) and the overlays | the map's terrain objects and units are placed | ore that a map puts on a cell of shape 5 or more is already gone then (S-0211 claim 7); an intermediate ramp result need not be the final loaded result (claim 12) |
  | Second | the same, with the map's terrain objects and units placed | unused-house cleanup begins | ramp smoothing sees every neighbour's shape from the first pass, which corrects the first pass's view (claim 12, scenario L2); this pass replaces earlier tile animations; cleanup or placement recalculations may later start missing ones (claims 12 and 19) |
  | Game start | terrain after starting-unit setup, initial crates, secret-lab choices and end-of-loading tags, even with no starting units or early-stop placement | the first tick | absent intervening recalculations, land types reflect the second pass's tiles; non-overlay-case stray tile numbers become the no-tile marker; tile animations start in visiting order in cells that have started none since the second pass (claims 10, 12 and 19); the bridge spans of claim 25 are found right after it; subsequent resource-glint setup can take further draws before tick 0 (S-0202 claim 14) |
- The 8 × 8 variant pattern is built at most once per session in the original (claim 14).
  Our engine rebuilds it per map load, the documented departure owned by S-0202 claim 11.
- The bridge spans and the bridge-body marks of low spans are set once at game start, right after
  the game-start pass, from the high-bridge marking and the tiles it leaves (claim 25). The
  burrowing test (claim 17) reads them from the first tick on.

## Edge cases and quirks

All switches default to the original behaviour.

- **`quirks.theater_five_forced_unset_roles`.** Default (original): in the theater with index 5, the bridge, cliff, shore, water, water bridge, water cliff and wooden bridge roles are forced unset whatever the file says (claim 4). Off: those roles use their declared values under the ordinary role rules.

- **`quirks.unset_role_range`.** Original: in the general tile tests, an unchecked role that is
  unset still uses its range, counted from the unset start of claim 4, so with the unset start −1
  tiles 0 … *n* − 2 pass its test (claim 5); and in ground
  smoothing, an unset rough transition role still smooths rough ground cells, writing one of the
  tile numbers 0 … 14 into every rough ground cell in the first pass at map load (provided the
  rough ground tile is not one of those numbers, and subject to the map-edge proviso of claim 10; the outcome
  leaves no unchanged rough ground cell after that pass), and a pair with an unset ground role writes −1
  into fully surrounded transition cells (claim 10); in ramp smoothing, an unset ramp or smooth-ramp role's range holds the tile −1 and
  its start −1 enters the formulas, so smoothed ramp tiles become low tile numbers of whatever
  tilesets come first (claim 11); an unset tunnel role makes tiles 0 … 2 tunnel entrances
  (claim 21). Fixed: an unset role's range is empty, a transition pair with an unset ground or
  transition role is skipped, and ramp smoothing is skipped when either ramp role is unset.
- **`quirks.role_one_past_end`.** Original: a role naming the first missing tileset number starts at
  the theater's tile count (claim 4). Fixed: it is unset.
- **`quirks.ramp_shape_zero`.** Original: a tile of the ramp range or of the smooth-ramp range whose
  slope shape is 0 is replaced by the tile just before the ramp range (claim 11). Fixed: it keeps
  its tile.
- **Uniform multi-cell variants and missing-variant fallback.** Claims 13 and 24 apply these
  drawing corrections unconditionally. They affect terrain and radar pictures, not game state,
  and have no quirk switches.
- **The land-type rules** (overlays, tiberium on slopes, cliff-back and the switch
  `quirks.cliff_back_case_sets`) are S-0211's (claims 5 to 8). This spec adds no switch for them.
- **The 8 × 8 pattern and the unsynchronised stream** (claim 14). The pattern depends on the
  unsynchronised stream's history in the session, so in the original two machines of a multiplayer
  game can show different ground variants. Network games are out of scope for now; S-0202 (claim
  11) has our engine run the unsynchronised stream from the game's seed; this spec only fixes where
  the draws are taken. The pattern reaches no game state,
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
| `BridgeSet`, `WoodBridgeSet` | bridge, wooden bridge (also the bridge spans of claim 25) |
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
| rules, building type | `ToTile` | to-tile field (claim 15): the tile whose name matches; a tile's name is built from its tileset's set name and its number within the set (the form of the name, a limit on the length of the set name and the matching rule are not established, Open questions) | string | none |
| rules, land-type sections | `Buildable` | the land type's buildable field (claim 16; S-0211 claim 2) | boolean | no |
| theater tile files | each tile cell's terrain code and slope code | claims 6 and 7 (the terrain code's meaning is S-0211 claim 6) | integers | — |
| theater tile files | each tile cell's 900 diamond pixels | palette indices, painted as claim 22 says | bytes | — |
| theater tile files | each tile cell's stored position, extra-image mark, extra-image position, width and height, and extra-image pixels | claim 23; only the difference of the two positions is used | integers, bytes | no extra image |
| theater tile files | each tile cell's depth-data mark and depth values (diamond and extra image) | claims 22, 23 and S-0102 | bytes | no depth data |
| theater tile files | each tile cell's flag value: its bit of value 1 is the extra-image mark, its bit of value 2 the depth-data mark, and its bit of value 4 the alternate-image mark; the other bits are ignored | claims 13 (alternate-image mark, read from the plain image's file only), 22 and 23 | integer, bits | no mark |
| theater tile files | each tile cell's height value | read, no effect on drawing or on a cell's level (claim 24) | integer | — |

## Measurement scenarios

All run under the glossary's measurement setup, with no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted),
with stock data on purpose-made test maps (one
theater, stated per scenario, chosen by the measurer from the theaters whose data has the roles the
scenario needs). Where a scenario names test files, they are loose files (S-0005) documented with
the scenario, and everything else stays stock. Screenshots are compared with the stock tile images,
identified by tile number. Unless a scenario says otherwise, use a skirmish with one human house
and one bystander computer player, game speed index 2, crates off, and one run per stated setup or
sub-run in a fresh program start. Keep starting units and all other objects outside the observed
test cells until the stated procedure calls for them. Reveal and view the test area without moving
a unit through it before the first screenshot. The observed area is the named patches, blocks or
test cells; the glossary's bystander void rule applies. A stock-data prerequisite that cannot be
met makes that part inconclusive, not a falsification. Record screenshots or video and any stated
recorder values. The procedure is to load, observe the first visible result, and then perform only
the orders or waits stated in that scenario. No harness changes are made by this spec.

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
  pictures, under different suffixes, in four runs, each in a fresh program start and each with the
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
- *Digits part (claim 3, and the animation keys of claim 19; named test files):* a loose copy of the
  theater data file that adds one tileset after the last declared one, with tile count 100 and a
  new file-name stem that no stock file uses, and loose tile files (copies of a stock one-cell tile
  file with the theater's own suffix, edited to three visibly different pictures) named for tile 1
  with two digits ("01"), for tile 99 with two digits and for tile 100 with three digits ("100").
  No other file of the stem exists, so every other tile of the set has no image. Tiles 1 and 100
  also get an animation in the section named by the set name, with attaches-to sub-tile 0, under
  the tile-animation keys of Import mapping with the tile's number written "01" and "100". The map places tiles 1, 99 and 100 of the set (numbers
  *T* + 0, *T* + 98 and *T* + 99, *T* the stock tile count) on separate cells. *Expected:* each of the
  three cells shows its own picture, and the animations appear on the cells of tiles 1 and 100 and
  not on tile 99's. *Falsified if:* any cell shows a picture other than its named file's picture,
  either expected animation is missing, or an animation appears on tile 99.
- *Theater part (claim 1, table T; named test files).* For each theater index 0 … 5, one run in
  a map of that theater: a loose copy of that theater's data file (under the name that table T gives
  for it) that adds one tileset after the last declared one, with tile count 1 and a new file-name
  stem, and a loose one-cell tile file of that stem and number 1 whose picture differs visibly from
  the clear tile. Two sub-runs per theater, each in a fresh program start: (a) the tile file has the
  theater's own suffix from table T; (b) the tile file has only the theater's fallback suffix from
  table T, the tileset's non-height-view field being absent. The map places the new tileset's tile
  (number = the stock tile count). *Expected:* in all twelve runs the new picture shows (so the data
  file name, the tile suffix and the fallback suffix of table T are those of every theater).
  *Falsified if:* any run shows the clear tile (the loose data file was not read, or the suffix is
  other than the table's).
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
    the stock data, as V2 does). If no stock clear tile has 2 to 4 images, the first map uses a
    named loose theater data file assigning the clear role to a tileset whose first tile has that
    many images; otherwise all data stay stock. Each run is in a fresh program start.
  - *Observable:* the image of the clear tile each cell shows (by the variant files' pictures).
  - *Expected:* first map, image P4(x mod 4, y mod 4) mod *n*, with *n* the number of images of the
    clear tile (claim 13); second map, the 8 × 8 pattern of claim 14, with period 8 in x and y.
  - *Falsified if:* any first-map cell differs from the formula, or the second map fails to
    repeat its images after 8 cells in either direction. V2 part B tests the second map's exact
    random pattern; a coincidental shorter period alone is not a falsifier.

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
  of 6 interior pixels in row 14 set to index 0. A named loose tile palette makes indices 0, 1,
  2 and 3 visibly distinct, with index 0 also distinct from the unpainted background, so a skipped
  zero pixel cannot pass the transparency test by chance. Neutral test lighting (S-0104), no extra image,
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

**X4. A variant without the cell (claim 24; named test file).**
- *Setup:* a stock tile of at least two cells in each direction and a loose variant "a" made from its
  plain file with one nonzero-sub-tile cell emptied (the cell absent in the variant's file). A flat map on which
  the tile is placed at many positions (the 4 × 4 pattern of claim 13 applies by the placed origins),
  so that at least one emptied cell selects image 1 (computed from claim 13 and checked against
  another non-origin cell of the same placed tile showing the variant's picture). Neutral lighting; a wheeled vehicle ordered across the emptied cell.
- *Observable:* screenshot of the emptied cell of a tile that shows image 1, and of the same cell of
  a tile that shows image 0; whether the vehicle can cross the emptied cell.
- *Expected in the original:* the cell of the image 1 tile paints nothing (its pixels keep whatever the terrain picture
  held there, which claim 24 leaves unspecified for a first painting), not the plain file's picture; the cell of the image 0 tile shows its picture; the vehicle crosses in both
  (the cell's land type is the plain image's).
- *Falsified in the original if:* the plain file's tile cell is painted in the emptied cell,
  or the cell is impassable. To distinguish no paint from an accidentally matching retained
  image, repeat after scrolling the area out of view and back; if the terrain pixels still
  cannot distinguish the alternatives, the no-paint part is inconclusive.
- *Engine comparison:* the emptied cell instead paints the plain file's complete tile cell,
  including its extra image and depth if present, while remaining passable. Missing pixels or
  any use of the empty variant's absent data fail the correction. There is no switch setting
  that restores the original hole.
- *Part B, a variant with fewer positions (claim 24; named test files; the scenario above is part A).*
  - *Setup:* a stock tile 2 cells wide and 1 high with no variants (or a loose copy of one), and
    a loose "a" variant for it made from a stock one-cell tile file, so the variant has 1 position and a
    picture that differs visibly from both tile cells of the plain file. The map places the tile
    once with its origin at (8*a* + 2, 8*b*), for positive integers *a* and *b* chosen so both
    cells lie well inside the playable area. As in V1 part B, the second cell shows image 1:
    its divided origin is (4*a* + 1, 8*b*), so the P4 entry is 1. Neutral lighting.
  - *Observable:* the picture in the second cell, sub-tile 1.
  - *Expected in the original:* the variant's picture in the second cell (index 1 mod 1 = 0
    of the variant). The origin cell shows image 0 (its P4 entry is 2, reduced modulo 2).
  - *Falsified in the original if:* the second cell shows nothing or the plain file's tile
    cell at sub-tile 1, or the origin does not show image 0.
  - *Engine comparison:* both cells use the divided origin and show image 1. A different
    image or an unpainted cell fails the correction.

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
- *Falsified if:* any of the five cells shows a tile other than its stated expected tile.
  In particular, Q + 2 on the lower eastward cell or Q + 8 on the upper westward cell is the
  competing partial-neighbour outcome, rather than the final full-neighbour outcome. Before
  testing, verify that all selected smoothed tiles preserve their slope shapes and that no
  ground-pair role overlaps either ramp range; otherwise this part is inconclusive.

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

**L4. A tile of slope shape 0 in a ramp range (claim 11, `quirks.ramp_shape_zero`; named test files). The setup below is part A, the ramp range; part B follows.**
- *Setup:* a loose copy of the tile file of one ramp-range tile (the first tile of the ramp
  tileset, say) with every slope code set to 0 (the tile file format is public), a one-cell tile
  for this test; the ramp tileset after the tileset that holds the tile just before the ramp range
  (the measurer checks that tile is usable and lies in no ground, transition or ramp range). A map
  with that tile on flat ground.
- *Observable:* the tile the cell shows after loading, compared with the stock tile images.
- *Expected:* the tile just before the ramp range (ramp start − 1), not the ramp-range tile.
- *Falsified if:* the cell keeps its ramp-range tile or shows any other tile.
- *Part B, the smooth-ramp range.* As part A, with a loose copy of the tile file of the first tile of
  the smooth-ramp tileset (every slope code 0, a one-cell tile), and a map with that tile on flat
  ground; the measurer checks that the tile just before the *ramp* range is usable and lies in no
  ground, transition or ramp range. *Expected:* the tile just before the ramp range (ramp start − 1),
  not the smooth-ramp tile. *Falsified if:* the cell keeps its smooth-ramp-range tile or shows any
  other tile.

**R1. Unset and one-past-the-end roles (claims 4, 5 and 10; named test files).**
- *Setup:* a theater whose data sets the rough pair, in which none of the tiles 8 … 14 is one of the
  four ground tiles of claim 10 (rough, sand, green or pave) or lies in a transition range of claim
  10 or in a ramp range, and each of them is usable (it has an image and a tile cell at sub-tile 0,
  so that the second pass keeps it instead of replacing it by the no-tile marker) (the measurer
  checks the stock data). A map with a plus-shaped patch of five rough ground cells (centre (*x*, *y*) and its
  four edge neighbours) on clear ground, with no other rough or transition tiles nearby. Check
  that the rough ground tile is flat and has rough land type. Seven runs (A, B, B2, C, D, D2 and E), each in a fresh program start:
  - Part A: unchanged data (this is the plus shape of scenario L1).
  - Part B: a loose copy of the theater data file without the rough-transition key.
  - Part B2: a loose copy in which the rough-transition key is present with the value −3 (a
    negative number, claim 4); the expected results are those of part B.
  - Part C: a loose copy in which the rough-transition key names the first missing tileset number
    plus 1.
  - Part D: a loose copy in which the rough-transition key names exactly the first missing tileset
    number, so the rough transition start is the tile count *T*.
  - Part D2: part D's data, with the plus replaced by a 4 × 4 block of rough ground cells, from
    (*x*₀, *y*₀) to (*x*₀ + 3, *y*₀ + 3), on clear ground.
- *Observable:* screenshot at the first frame of the game; the tile number each of the five cells
  (in part D2, each cell of the block) shows. In parts D and D2, also whether a building can be placed on the centre cell (in D2 on the
  cell (*x*₀ + 2, *y*₀ + 2)) and on an arm cell (in D2 on another cell of the block), judged by the
  placement indicator without placing it, with a building of the human house close enough that
  only the land type decides (entering the cell, or placing a building on it, would recalculate
  it, Terms). Use an ordinary one-cell building, with all other placement conditions satisfied.
  If the stock land-type sections do not distinguish rough from clear by buildability, a named
  map override sets clear land buildable and rough land not buildable for D, D2 and E only. Check
  two untouched control cells of those land types: the placement indicator must accept clear
  and refuse rough. Leave the one-cell test building's to-tile, place-anywhere, naval, laser-fence,
  gate and fence-post flags absent or disabled so S-0406's ordinary land rule decides. If no such one-cell placement test can be set up, the land-type parts are
  inconclusive; the pictures still test the no-tile outcomes.
- *Expected:* Part A: as L1 (centre *G*; arms *L* + 11, *L* + 7, *L* + 14, *L* + 13 for N, E, S, W).
  Parts B, B2 and C: W arm tile 12, N arm tile 10, centre tile 8, S arm tile 14 and E arm tile 14.
  None remains the rough ground tile. Each is drawn as claims 6 and 7 give for sub-tile 0.
  Part D: at the first frame all five cells show the clear tile (claim 24's no-tile picture) and
  play as clear land, the centre included. These final outcomes, not the internal traversal or
  the intermediate tile assignments during loading, are judged. Part D2: at the first frame every cell of the block shows the clear tile and
  plays as clear land, except (*x*₀ + 2, *y*₀ + 2), which holds *T* + 15 (each whole-map pass
  turns one more layer into the no-tile marker, claim 10): its picture is unspecified (claim 24)
  and is not judged, and it plays as rough land (its land type comes from *G*, the tile it held
  when the game-start pass reached it) until its next recalculation (part E), where the placement
  check can tell.
- *Falsified if:* in parts B, B2 or C any arm shows a rough transition tile, the centre shows *G*, or
  the tile numbers differ from those above; in part D any of the four arms shows a rough
  transition tile, the rough ground tile or anything but the clear tile, or the centre judges as
  rough land at the first frame; in part D2 the cell
  (*x*₀ + 2, *y*₀ + 2) judges as clear land at the first frame, or another cell of the block judges
  as rough land. Parts B, B2 and C giving different results would falsify
  the rule that an absent key, a negative number and a number beyond the first missing tileset all
  leave the role unset.
- *Part E, a unit's move recalculates the cell (claims 10 and 12, and the triggers in Terms).*
  *Setup:* the data and map of part D2, and a wheeled vehicle of the human house parked outside the
  block. *Procedure:* at the first frame, the placement check of part D2 on the cell
  (*x*₀ + 2, *y*₀ + 2); then drive the vehicle onto that cell and off again; then the same placement
  check. *Expected:* before the drive that cell judges as rough land and the rest of the block as
  clear land; after it the cell judges as clear land too (it now holds the no-tile marker).
  *Falsified if:* the cell still judges as rough land after the drive. The same buildable-field
  condition as part D applies.

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
- *Falsified if:* *c* shows anything other than *L* + 1 in theater 5, or the control does not
  show *G*.
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
- *Falsified if:* any control cell differs from its mask formula, or in the loose copy any
  cell shows a tile other than its ground tile.

**R4. Loaded outcome of one overlapping-role fixture (claim 10; named test file).**
- *Setup:* a theater whose data sets the rough pair and the sand transition role (the measurer
  checks the stock data), and a loose copy of its theater data file in which the sand transition
  role names the same tileset as the rough transition role (so the rough transition tiles *L* … *L* + 15
  are also the sand transition range). Keep the sand ground role set and distinct from the rough
  ground tile. The measurer checks that both ground tiles lie outside *L* … *L* + 15 and all
  other pair and ramp ranges; the clear background belongs to neither pair. Neither the green
  nor pave pair nor either ramp range may contain *G* or *L* … *L* + 15 in the loose copy. A map with scenario L1's plus-shaped patch of five rough ground cells
  on clear ground, away from other rough or sand tiles. Two runs, each in a fresh program start: a
  control with the unchanged data (this is scenario L1), and the loose copy.
- *Observable:* screenshot after loading; the tile each of the five cells shows.
- *Expected:* control: centre *G*; arms *L* + 11, *L* + 7, *L* + 14, *L* + 13 for N, E, S, W. Loose copy:
  centre *G*, and all four arms *L* + 15. This is the sand-pair outcome for an arm whose four
  neighbours are outside that pair: the centre's rough ground tile is not in the sand range.
  The control's arm tiles are the competing rough-pair outcome; the screenshot distinguishes
  which result prevails, without requiring either implementation to perform a particular
  sequence of internal updates.
- *Falsified if:* in the loose copy any arm shows another tile than *L* + 15 (in particular a tile of
  the control), or the centre shows another tile than *G*.

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
  - *Falsified if:* either cell differs from its stated expected tile. In particular, *L* + 1
    on the last-tile cell suggests a shorter range, and *G* on the cell beyond it a longer range.
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
  - *Falsified if:* either cell differs from its computed expected tile. Keeping the placed
    last tile suggests a shorter range; re-choosing the tile beyond it by this range's formula
    suggests a longer range. Include the game-start recalculation in the prediction, and use
    shape-preserving tiles so the two-load-pass result is stable by the first screenshot.

**V1. Fixed variant pattern (claim 13).**
- *Part A, one-cell tiles.*
  - *Setup:* a map filled with one one-cell tile that has 2 or 4 images in the chosen theater's
    stock data (identified by its variant files).
  - *Observable:* which image each cell shows, over an 8 × 8 block.
  - *Expected:* image P4(x mod 4, y mod 4) mod (number of images).
  - *Falsified if:* any cell differs.
  - *What the pattern looks like over a large area (for comparison with an engine's screenshots).*
    The pattern is anchored to map cell (0, 0) and has no random part. Write *s* = x + y (a screen
    row of cells: S-0101's F1 puts cells with equal *s* side by side on one horizontal line) and
    *d* = x − y (a screen column: cells with equal *d* lie one above the other). Table P4 equals
    *s* mod 4 on the cells with *s* even and *d* mod 4 on the cells with *s* odd. So with 4 images,
    every other horizontal row of cells shows a single image along its whole length (image 0 on the
    rows with *s* ≡ 0 mod 4, image 2 on those with *s* ≡ 2), and the cells between them form
    vertical columns of a single image (image 1 in the columns with *d* ≡ 1 mod 4, image 3 in those
    with *d* ≡ 3). With 2 images whole rows alternate: image 0 on every row with *s* even and image 1
    on every row with *s* odd, which reads on the screen as horizontal stripes one cell high (a
    checkerboard in map coordinates). With 3 images the image is (*s* mod 4) mod 3 or (*d* mod 4) mod 3 by the
    same split. On the isometric screen the bands are therefore horizontal (rows of constant *s*)
    and vertical (columns of constant *d*); in a top-down drawing of the map grid, with x across and
    y down, the same lines of constant x + y and constant x − y appear as diagonal stripes. A tile
    with more than 4 images, such as V2's example, does not use this table at all (claims 13 and
    14); V2 describes its typical appearance, not a prohibition on every chance alignment.
- *Part B, a tile larger than one cell (claim 13's drawing correction).*
  - *Setup:* a tile 2 cells wide and 1 high with 2 images. Use one from the chosen theater's stock
    data if the measurer finds one. Otherwise use a named test file: a loose "a" variant for a stock
    tile of that size that has no variants, made from another tile file with the same cell layout whose image differs
    visibly. The map places the tile with origins (8*a*, 8*b*) and (8*a* + 2, 8*b*), for positive
    integers *a* and *b* chosen so all four cells lie well inside the playable area.
  - *Observable:* which image each of the four cells shows.
  - *Expected in the original:* at the first origin both cells show image 0. At the second
    origin, the origin cell shows image 0 (its P4 entry is 2, reduced modulo 2), and the second
    cell shows image 1 (its divided origin is (4*a* + 1, 8*b*), whose P4 entry is 1), so the two
    halves of that placed tile differ.
  - *Falsified in the original if:* any cell shows another image, in particular if both halves
    of the second placed tile show the same image.
  - *Engine comparison:* both cells at the first origin show image 0, both at the second show
    image 1. Different variants within either placed tile fail the correction. One-cell tiles
    retain part A's original pattern.

**V2. 8 × 8 pattern (claim 14).**
- *Setup:* a map filled with a one-cell tile with more than 4 images, identified from its files.
  Record its imported image count. For part B use named test files with exactly 8 visibly distinct
  images if no qualifying tile with that count is available: the 8 is a test choice that exposes
  every possible pattern entry, not a stock-data assertion. Two runs, each in a fresh program start, going straight to
  the map; a third run in the same session after another map of the same kind.
- *Observable:* the image of each cell over a 16 × 16 block.
- *Expected:* the images repeat with period 8 in x and in y. With exactly 8 images, each shown
  image exposes one pattern entry, which differs from its 8 index-wrapped neighbours except
  where 64 candidate draws failed; with fewer images the final modulo reduction can make
  neighbours equal without a failed candidate sequence. The third run shows the same pattern
  as the map before it in that session.
- *Falsified if:* the images fail to repeat after 8 cells on either axis, or the pattern changes
  between two maps of one session. An accidental shorter period is not a falsifier. Part B
  decides the candidate/retry rule; adjacent equal values alone cannot falsify it without knowing
  whether all 64 candidates clashed.
- *What the pattern looks like over a large area.* The 8 × 8 period is anchored to map cell (0, 0)
  and repeats in x and in y. With 8 images, the images inside a period look scattered: no image
  repeats on a cell's 8 index-wrapped neighbours apart from rare failed tries, so a large
  area typically has no long stripes or bands, but chance alignments are not forbidden. The
  guaranteed repeat is every 8 cells along x and along y (on the screen, every 8 cells along each of the two diagonal axes of the
  ground).
- *Part B, the draws (claim 14; needs a recorder that reads the unsynchronised stream, an extension
  of S-0202's state recorder, which the project lead has approved).* *Setup:* the map of part A
  as the first game of a fresh program start, one run. *Observable:* the game's seed and the
  complete unsynchronised stream state at the first tick, read by the recorder, and the 8 × 8
  period of images from part A. *Procedure:* reconstruct the stream from that seed (S-0202
  claims 2 and 8), finding the smallest total draw count *K* up to 100000 that matches the full
  recorded state. For each proposed start count *N* from 0 to *K*, compare the period with the
  formula *8 × 8 pattern* applied to the raw values after those first *N* draws. A match is
  admissible only if its draw segment ends no later than *K*. *Expected:* an admissible *N*
  reproduces every entry and takes *u*₆₄ draws, 64 … 4096 as the formula gives; record the
  smallest admissible *N* and its draw count. *Falsified if:* no admissible *N* reproduces the
  whole period, or a claimed match requires draws after the recorded state. If no *K* up to
  100000 matches the complete state, the part is inconclusive at that search limit, rather than
  a falsification of the tile formula. This tests a contiguous segment, not its precise trigger
  among loading events.

**V3. Variants do not change the land type or slope shape (claims 6 and 13; named test files).**
- *Setup:* a one-cell tile of the chosen theater with one image (no variant files) whose terrain
  code is 0 (clear), and a loose "a" variant for it made from a one-cell water tile file of the
  same theater. A map filled with that tile over a 4 × 4 block, so by table P4 the cells with
  P4 entry 1 or 3 show image 1 (the water picture).
- *Observable:* screenshot; whether a wheeled vehicle can be ordered onto each of the 16 cells.
- *Expected:* the water picture on the 8 cells with odd P4 entries, and the vehicle can enter all
  16 cells (clear land from the plain image).
- *Falsified if:* any cell showing the water picture refuses the vehicle, or plays as water.
- *Slope part:* a second run uses an otherwise identical variant whose slope code is 1, while
  the plain file remains slope 0 and the tile lies outside all smoothing ranges. Drive the
  vehicle across every cell and record its height at the cell centre as in S-0106 scenario T1,
  comparing to control cells showing the plain image. *Expected:* all cells have the flat
  surface height of the plain file. *Falsified if:* a variant-shown cell gives the shape-1
  surface height or any height different from the flat control.

**V4. The alternate-image mark (claim 13, Q-0055; named test files).**
- *Setup:* a stock one-cell tile of the chosen theater with 2 to 4 images (the measurer identifies
  one by its variant files) whose variant "a" looks clearly different from the plain image. A map
  fills an 8 × 8 block with that tile, so that by table P4 some cells would show image 1. Run 1:
  the stock files. Run 2: a loose copy of the tile's plain file in which the tile cell's flag value
  has its bit of value 4 set, nothing else changed. Run 3: the stock plain file and a loose copy of
  variant "a" with that bit set. Run 4: as run 2, but the bit of value 4 left clear and the bit of
  value 8 set. Each run in a fresh program start.
- *Observable:* screenshot after loading; which image each cell of the block shows.
- *Expected:* runs 1, 3 and 4 show the P4 pattern of V1 part A; run 2 shows the plain image on
  every cell of the block (the mark is set, the state is clear at load).
- *Falsified if:* run 2 shows a variant on any cell (the mark is another bit, or the state is not
  clear at load), run 3 shows the plain image everywhere (the mark is read from the variant file),
  or any cell in runs 1, 3 or 4 differs from the P4 prediction.

**G1. The morphable flag (claim 15).**
- *Part A.* *Setup:* a map override that gives a stock building type a to-tile field naming a stock
  tile, and a test map with a cell of a tileset whose morphable flag is unset and a cell of one
  whose flag is set (the measurer checks the stock data). *Observable:* whether the building can
  be placed on each. *Expected:* only on the morphable cell. *Falsified if:* it can be placed on
  the non-morphable cell or cannot be placed on the morphable cell while the other placement
  conditions are satisfied.
- *Part B, a cell without a tile.* *Setup:* part A's building, and a map cell holding a tile
  number equal to the theater's tile count (claim 7). *Observable:* whether the building can be
  placed on it. *Expected:* it can. *Falsified if:* it cannot.
- *Part C, a stray tile number (claim 15; named test files).* *Setup:* the data and map of scenario
  R1 part D2 (its cell (*x*₀ + 2, *y*₀ + 2) holds a stray tile number from the game-start pass of
  claim 12 until its next recalculation) and part A's building, with a building of the human house close enough that
  only the tile and land type decide (as in R1 part D2). Two controls on the same map, each a cell
  whose land type is that cell's (rough) that nothing recalculates before the check: one in a tileset whose morphable flag is unset
  and one in a tileset whose flag is set (the measurer picks them in the stock data and adds them
  to the map; if the stock data have no such pair, the part is not run). *Observable:* the
  placement indicator of the building on each of the three cells, judged without placing it and
  before any unit enters or leaves them. *Expected:* the non-morphable control refuses (the tile
  test), the morphable control accepts (so the land type and the other conditions allow the
  building on rough land), and the cell (*x*₀ + 2, *y*₀ + 2) accepts. *Falsified if:* that cell
  refuses while the morphable control accepts. Inconclusive if the morphable control refuses or the
  non-morphable control accepts.

**G2. Allow-tiberium and the no-tile exception (claim 16; named test data).**
- *Setup:* a small flat map with a stock animated ore mine well inside the playable area. Of its
  eight neighbours, seven hold walls and the eighth is the target. The mine's own cell holds no
  resource. The target has overlay data value 0, no overlay, no object or bridge mark, and a
  buildable land type. No unit enters these cells. Use a named loose theater data file to change
  only the tested tileset flag between runs; keep the mine's stock animation and chance fields.
  A named map override may make the target's land type buildable. Turn ordinary resource growth
  and spreading off so that only the mine can supply new resource (S-0401 claim 15). Choose
  target tiles outside every ground and ramp smoothing range and verify the target is empty
  at the first tick.
- *Part A:* two fresh runs with the same valid flat target tile, its allow-tiberium flag yes and
  no respectively. *Part B:* two more fresh runs with the clear tileset's flag no: one target
  holds the ordinary clear tile, the other a tile number equal to the theater tile count, which
  becomes the no-tile marker after loading (claim 7).
- *Procedure:* video the mine and target with tick numbers until the mine completes its first
  animation, at most 18000 ticks per run. Identify the completion and spread opportunity by
  S-0401 claim 15's sprite-frame sequence, not merely by elapsed time or resource appearance.
  Record the target immediately before and after that completion. A run without an identifiable
  completion is inconclusive and repeated, not evidence against germination. With seven
  neighbours blocked, the target is the only possible germinable neighbour, regardless of the
  spread step's random starting direction.
- *Observable:* whether the target gains resource at that completed animation. *Expected:*
  part A yes-flag target gains resource and no-flag target stays empty; part B ordinary clear
  target stays empty and no-tile target gains resource. *Falsified if:* any of these four
  outcomes differs at a verified completion while all other conditions are satisfied. Thus a
  failure to generate a random mine animation cannot masquerade as a failed tile condition.
- *Part C, the other three conditions of claim 16:* repeat part A's yes-flag fixture in three
  fresh runs changing only the target condition: a named test tile with slope shape 1 outside
  every smoothing range, its land type made non-buildable, or a wall overlay placed on it.
  Each stays empty at a verified completion, while the unchanged flat/buildable/empty target
  of part A receives resource. Resource on a changed target falsifies that condition. This
  tests representatives of the conditions, not every overlay or land type.

**G3. Burrowing (claim 17; named test file for part C).** Inside the playable area only (the answer
outside it is not measured, claim 17). The destination cells are flat, free of buildings and terrain
objects, and out of the way of every other condition of the vehicle specs, so that only the tile's
tileset decides.
- *Part A, a tileset without the flag.* *Setup:* a tunnelling unit ordered to dig in on, and come up
  at, a cell of a tileset whose allow-burrowing flag is set to no in the stock data (the measurer
  checks the stock data; a loose copy of the theater data file may set the flag to no for a tileset).
  *Observable:* the cells where it digs in and comes up. *Expected:* not on that cell.
  *Falsified if:* it digs in or comes up there.
- *Part B, a tileset with the flag.* The same with a cell of a tileset whose flag is set to yes.
  *Expected:* it digs in and comes up on exactly that cell. *Falsified if:* it does not (then part A
  is inconclusive).
- *Part C, the absent key.* *Setup:* part B with a loose copy of the theater data file in which the
  allow-burrowing key is removed from the tileset of part A. *Expected:* the unit digs in and comes up on
  that cell as in part B (absent means allowed). *Falsified if:* it does not.
- *The tunnelling unit (named test data, every part).* Stock rules may hold no vehicle type with the
  tunnelling locomotion; the measurer then gives one stock vehicle type that locomotion through a
  rules override (the vehicle specs' import key), and checks the stock data first.
- *Part D, the other conditions (claim 17).* *Setup:* a test map on flat ground of a tileset with
  the flag set, holding, each well apart: a sloped cell; a high-bridge cell whose ground the unit
  may enter (S-0211 scenario L21's map); a cell with a building of the computer house on the
  ground layer; a cell with a terrain object (a tree); and two controls, a cell with ore and a
  cell of rough land type. *Observable:* for each cell, whether the unit comes up on exactly that
  cell when ordered there. *Expected:* it does not come up on the sloped, high-bridge, building or
  terrain-object cell, and it does come up on both controls (the overlay and the land type play no
  part). *Falsified if:* it comes up on one of the first four, or refuses a control.
- *Part E, the band of a low span (claims 17 and 25).* *Setup:* a test map, flat clear ground of a
  tileset with the flag set: a start tile of the wooden-bridge role from table B (for example
  *i* = 6, whose start cell is its sub-tile 4) and, 10 cells further east on the start cell's row, a
  tile of the same role placed so that one of its end sub-tiles lies there (for example *j* = 6,
  with its sub-tile 4 there), with clear cells between them and no overlays. Write (*x*₀, *y*₀) for
  the start cell. No cell between the ends is a high-bridge cell, so the span is low. The measurer
  checks that column *x*₀ + 5 holds plain clear cells on rows *y*₀ − 3 … *y*₀ + 2 (the two tiles
  are narrow enough). *Observable:* whether the unit comes up on each cell (*x*₀ + 5, *y*) for
  *y* = *y*₀ − 3 … *y*₀ + 2, and on the start cell and the far-end cell where these are flat.
  *Expected:* it does not come up on rows *y*₀ − 2, *y*₀ − 1, *y*₀ and *y*₀ + 1; it does come up on
  rows *y*₀ − 3 and *y*₀ + 2, and on the end cells when they are flat. A control run without the
  far-end tile (no span) lets the unit come up on all six rows. *Falsified if:* it comes up on one
  of the four band rows, or refuses row *y*₀ − 3 or *y*₀ + 2. If the control refuses a row, the
  test map is not neutral and the run is inconclusive. Run 2, a high span: the same ends, with a
  high bridge laid between them (high-bridge overlays of S-0211 claim 15 placed so that every
  cell between the ends on row *y*₀ is a high-bridge cell). *Expected:* the unit comes up on rows
  *y*₀ − 3, *y*₀ − 2 and *y*₀ + 2 of column *x*₀ + 5 wherever no high-bridge marking reaches them
  (the high span marks no band), and not on the high-bridge cells. *Falsified if:* it refuses a
  cell of column *x*₀ + 5 that no high bridge marks.
- *Table-coverage repeats of part E (claim 25):* repeat the low-span band test in separate maps
  for every start tile and start sub-tile listed in table B, in its listed direction, and for
  every listed far-end tile and end sub-tile, pairing each with a compatible start. Both roles
  are tested separately without overlapping role ranges. Keep the between-cells flat clear and
  choose a midpoint far enough from both tile footprints for the six-cell cross-band probe.
  Controls change only the tested start or end sub-tile to a valid sub-tile not listed for it,
  and include tiles that table B says never start or end a span. *Expected:* the listed pairs
  refuse exactly the four band cells and accept the two outside cells; the nonqualifying
  controls have no band and accept all six. *Falsified if:* a listed pair has another band or no
  band, a direction differs, or a nonqualifying control has a band. Inconclusive if an unintended
  end tile creates another span, or a control fails the ordinary burrowing conditions.
- *Part F, a cell without a tile (claim 17).* *Setup:* part B's map with a cell holding a tile
  number equal to the theater's tile count (it holds the no-tile marker after loading, claim 7),
  and a loose copy of the theater data file in which the clear tileset's allow-burrowing flag is
  set to no. *Expected:* the unit comes up on that cell (the tileset condition holds for a cell
  outside 0 … tile count − 1), and not on a cell of the clear tileset. *Falsified if:* it refuses
  the no-tile cell while it would come up on it with the flag set.

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

- *Part C, the game-start pass (claims 12 and 19; named test files).* *Setup:* the map is scenario
  L2's eastward climb. Run 1: a loose copy of the theater data file that gives the first tile of
  the smooth-ramp tileset (*Q* + 0) an animation of a stock animation type *A* with attaches-to
  sub-tile 0, and no animation to *Q* + 2. Run 2: the same, and *Q* + 2 gets an animation of another
  stock animation type *B* that looks clearly different, with attaches-to sub-tile 0. Each run in a
  fresh program start. *Observable:* screenshot of the lower cell at the first frame of the game,
  before any unit moves. *Expected:* the lower cell shows *Q* + 0 in both runs. Run 1: it carries
  *A*'s animation already at the first frame (the second pass started from *Q* + 2, which names
  none; the game-start pass started from *Q* + 0). Run 2: it carries *B*'s animation only (the second
  pass started *B*'s, so the game-start pass starts no second one). *Falsified if:* in run 1 the
  cell carries no animation at the first frame (then the game-start pass does not exist, or does
  not start animations), or in run 2 it carries *A*'s animation, alone or with *B*'s.

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

- *Part E, the first pass's animation does not survive (claim 19; named test files).* *Setup:*
  scenario L2's isolated cell (the placed tile *B*, the ramp tile of shape 1, which the first and
  the second pass both smooth into *Q* + 2, *Q* the smooth-ramp start). A loose copy of the theater
  data file that gives *B* an animation of one stock animation type with attaches-to sub-tile 0
  and displacement (0, 0), in the section named by *B*'s tileset's set name. Run 1: *Q* + 2 gets no
  animation. Run 2: *Q* + 2 gets an animation of another stock animation type that looks different,
  with attaches-to sub-tile 0 and a displacement of (40, 0). Each run in a fresh program start.
  *Observable:* screenshot after loading; which animations appear on the cell, by their types and
  positions. *Expected:* run 1 shows no animation on the cell (the first pass started *B*'s, and
  it did not survive); run 2 shows only the animation of *Q* + 2, at the displaced position, and
  not *B*'s. *Falsified if:* run 1 shows *B*'s animation, or run 2 shows *B*'s animation (alone
  or together with the other), or run 2 lacks the expected animation of *Q* + 2.
- *Part F, how a tile animation is created (claim 19, Q-0046; named test data).* *Setup:* a rules
  override with a test animation type that has the resource chain reaction (the S-0109 import key)
  and a loop count of 2, and a loose copy of the theater data file that gives one stock one-cell
  tile that animation with attaches-to sub-tile 0 and displacement (30, 15) pixels, which F6 turns
  into one cell along +x. Run 1: the map places that tile once on flat ground at (*x*, *y*), with
  ore in (*x*, *y*) only in sub-run A and in (*x* + 1, *y*) only in sub-run B, with no other
  resource nearby. This separates the directly affected cell from later propagation. Run 2: part E's run 1 (*B*'s animation started by the
  first pass only), with *B*'s animation set to the test type and displacement (0, 0), and ore in
  the cell. *Procedure and observable:* first record which cells hold ore at the first visible
  result, before orders or resource updates; then let the animation run and count its passes in
  video with tick numbers. The resource observation and the playback count are separate. *Expected:* run 1: the ore of (*x* + 1, *y*) is
  gone in sub-run B and that of (*x*, *y*) remains in sub-run A (the start event acts at once,
  in the cell under the displaced position), and the animation plays 2 passes (its type's loop count, no repeat factor),
  then ends or switches to its follow-up type. Run 2: the ore in the cell is gone although no
  animation remains (the first pass's start event kept its effect). *Falsified if:* in run 1 the
  ore in sub-run A is gone, ore in sub-run B remains, or the number of passes differs; or in run 2 the ore remains.

- *Part G, local-area resize preserves an existing animation (claims 8 and 19; named test
  data).* *Setup:* part A's map and tile, with its attached animation set to a non-vein test
  type using a stock sprite, enough passes to remain visible throughout the test, and no
  damaging or terrain-changing effects. Add the local-area resize action of S-0524 claim 8
  at tick 30, enlarging the area while leaving the observed tile well inside both rectangles.
  Use game speed 6, one human house and one bystander computer player kept away; no engine
  add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted).
  Run once with the action and once without it.
  *Procedure and observable:* capture video with tick numbers around tick 30, tracking the
  animation's image frame and pass count, not its screen position if the view moves.
  *Expected:* its playback continues across the resize as in the control, without an extra
  copy or a return to the starting frame. *Falsified if:* resize removes, duplicates or
  restarts the animation. This does not establish whether a previously missing animation
  can start or how many cells were recalculated.

**G5. Shadow casters need a count (claim 18; named test files).** *Setup:* S-0104's scenario L15
part D, whose named test copy of the theater data file declares one tileset a shadow caster with a
shadow-tile count of 1. Run 1: that copy. Run 2: that copy without the tileset's shadow-tile count
(flag kept). Run 3: that copy without the flag (count kept). Run 4: that copy with the flag set and
the count explicitly 0. *Observable:* the vehicle's brightness on each cell, as in S-0104's
scenario L15 part D.
*Expected:* in run 1 the cells that S-0104 claim 19 marks darken the vehicle; in runs 2 to 4 no
cell does. *Falsified if:* run 2, 3 or 4 darkens a cell, or run 1 darkens none.

## Open questions

- Q-0018 asks four things of this spec's area, and two reach it. Its question 4 (the draws of a tile
  animation that starts during loading) is answered by claim 19 (when and for which tile an
  animation starts, and that its creation takes the draws of S-0202 claim 10) with S-0202. Its
  question 5 (what a neighbour outside the map counts as in ground and ramp smoothing) is answered by
  claim 10: a blank cell, which belongs to no pair and is flat for ramp smoothing (the cliff-back
  rule is S-0211's). Its questions 1 to 3 and 6 are answered by S-0009, S-0202, S-0203 and S-0211,
  and this spec states nothing for them.
- Q-0055 is answered in full by claim 13 (the bit of value 4 of the tile cell's flag value, read
  from the plain image's file only), the Terms, the Import mapping and scenario V4.
- Q-0041: its question 2 (which cells besides high-bridge cells fail the burrowing test, and the
  other conditions) is answered by claims 17 and 25 and scenario G3 parts D to F. Its question 1
  (the no-move rows of S-0212's table T5) is S-0212's.
- Q-0046: this spec answers how a tile animation is created (claim 19: no loop delay, no repeat
  factor, not trigger-played, its start event at once at the displaced position, including for
  the animations of the first pass). Its question 1 (the overlay type's cell-animation key) is
  S-0009's, and questions 2a to 2c (what the start event's chain reaction, particles, scorch marks
  and craters draw, and which sprite gives μ) belong to S-0109, S-0202 and the specs of those
  effects.
- Whether a tile animation whose type has the vein-animation flag lets its cell start a second one
  after it ends: in the original, removing such an animation in play clears, on the cell under its
  position, the record that the cell has started one (claim 19). No stock tile animation is known
  to have that flag; claim 19 does not cover the case.
- Bridge destruction and repair are owned by S-0511 claims 5 and 19. Claim 25 states the
  body-mark and rediscovery outcomes for a given affected overlay and lookup result; the
  bridge-link lookup is S-0212 claim 3's.
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
- Which cells the original paints when it repaints only part of the screen. An extra image that
  reaches far to the side of its cell's diamond may be cut or missed at the edge of a repainted
  area, and how far it may reach upward and downward without being missed is not established. Our
  engine paints whatever its images touch; the difference is a repainting artefact, not a rule,
  and is not specified.
- Settled in revision 10: the radar colours of a cell come from the image it shows, not from the
  plain image (claim 13; S-0410 already says so).
- Whether the smoothed ramp tiles of the stock theaters carry the same slope shape as the ramp
  tile they replace (claim 12 assumes so).
- General overlapping-role smoothing outside R4's freshly loaded fixture remains outside
  claim 10. No history-dependent or pair-priority contract for those other cases is established.
- Whether placing the map's terrain objects and units at map load, between the two passes of
  claim 12, recalculates their cells, as putting an object into a cell does in play (Terms).
  Claims 12 and 19 state the results after loading on the reading that it does not.
- What a to-tile building does when placed, and how the to-tile name is matched (claim 15). The
  original appears to build a tile's name from the set name (only its first 28 characters, a
  longer set name being cut), a space and the tile's number within the set written with at least two
  digits; the comparison with the building's to-tile value is not established (for example
  whether letter case matters).
- Which game rules use the role tests of table R that this spec does not use (bridges, cliffs and
  their destruction, shores, naval placement, the random map generator).

## Cross-spec changes to confirm

Named change to confirm: the approved specs whose measurement-setup sentence uses the older wording of the condition are reworded to this spec's wording ("no engine add-on and no mods, the named test files of a scenario and the purpose-made test maps excepted") in the lead's round-114 sweep (lead decision 2026-10-17); this spec makes no change to them. The statements below confirm unchanged claims: S-0211 claim 5 includes the game-start recalculation and the relevant move trigger; S-0202 claim 14 states the game-start animation-draw ordering; S-0410 scenario R6 part D uses S-0010 claim 13's ordinary variant pattern; S-0103 defines terrain animations to include those started when terrain is set up or a cell is recalculated in play; and S-0009 claims 6 and 8 defer the picture rule to S-0010 claim 24.

## Questions for the project lead

None is open.

- Decided (lead delegated, 2026-10-10): the glossary's *terrain animation* entry says the animation is
  created when the map's terrain is set up or when a cell is recalculated in play (claim 19).
- Decided (lead delegated, 2026-10-10): S-0009 claims 6 and 8 point to claim 24 of this spec in place of
  "is drawn as"; our engine paints the clear tile either way.
