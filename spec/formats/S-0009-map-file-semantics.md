---
id: S-0009
title: Map file semantics (geometry, terrain streams, waypoint markers, start positions, preview)
topic: formats
revision: 7
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0009: Map file semantics (geometry, terrain streams, waypoint markers, start positions, preview)

## Summary

How the original game interprets a map file once it has been split into sections and keys: the
map's footprint and playable area, terrain height, the decoded terrain, overlay and preview values,
the waypoint markers, skirmish start positions, and which parts of a map a skirmish game reads in
which observable order. Byte-level stream encodings, lighting, tile categories, triggers and the
map's rules overrides are outside this spec (Open questions and Questions for the project lead).

## Terms

Terms used as defined in the shared glossary: *cell*, *lepton*, *height level*, *theater*, *house*,
*country*, *skirmish*, *lobby*, *overlay*, *overlay data value*, *tileset*, *sub-tile*, *slope*,
*cell tag*, *type*, *scenario random stream* (with *draw* and *ranged draw*),
*quirk switch*, *measurement setup*, *engine constant* and *absent-key default*. The terms below are
specific to this spec.

- **Cell coordinates (x, y).** Integer cell position on the isometric grid. Increasing x runs
  towards the lower right of the screen, increasing y towards the lower left.
- **Screen row s and screen column d.** For a cell, s = x + y and d = x − y. Cells with equal s
  lie on one horizontal line of the screen: a step of 1 in s is half a cell height (one *half-row*),
  a step of 2 in d is one full cell width.
- **Map width W and map height H.** The third and fourth numbers of the map size entry, in full
  cell widths and full cell heights of the screen-aligned map rectangle.
- **Legal cell.** A cell that exists on this map (claim 1).
- **Local area.** The screen-aligned sub-rectangle of the map in which play takes place, given by
  four numbers (Lx, Ly, Lw, Lh) in the same units as W and H.
- **Usable cell.** A legal cell inside the local area (claim 4).
- **Height level.** As in the glossary. In a map file a cell's height level may be any integer in
  −128 … 127 (claim 7); stock maps use 0 and above.
- **Blank cell.** A legal cell in the state it has before the initial fill: tile number 65535 (the
  no-tile marker), sub-tile 0, height level 0, ice-growth flag clear, flat, clear land, no overlay
  (overlay data value 0) and no objects.
- **Visited cell.** A legal cell that a pass in the visiting order of claim 3 reaches. On every map
  with W ≥ 2 that is every legal cell.
- **Cell number.** A single integer that encodes a cell: n = 1000·y + x.
- **A waypoint (waypoint marker).** One of 702 numbered map markers, each either undefined or naming one cell.
- **Start waypoint markers.** The waypoint markers numbered 0 to 7.
- **Start list.** The ordered list of start cells built for a skirmish game (claim 17). A player's
  *start position* is an index into this list.
- **Home waypoint.** The waypoint the initial camera is placed on; its number is a map setting.
- **Format version.** An integer map setting that selects between legacy and current encodings.
- **Clear base tile, water base tile.** The tile numbers the initial fill writes for a clear fill
  and (before the random variant is added) for a water fill (claim 6).
- **Header block.** A separate map block that describes start markers for the preview (claim 25).
  Its pairs 9 to 16 can also act as start position marks (claim 29).
- **Neutral house, special house.** The two non-player houses that a skirmish game always creates
  after the players.
- **Scenario random stream.** As in the glossary; all draws below are taken from it.
- **Raw value.** One output of the scenario random stream's generator (S-0202, claim 1). Taking a
  raw value advances the stream by one.
- **Range rule.** The rule by which a *ranged draw* over an inclusive range turns raw values into a
  number in that range (S-0202, claim 4). It takes exactly one raw value when the range size is a
  power of two, and may take more than one otherwise, because it rejects raw values that fall
  outside the range.
- **Creation draw.** The one raw value that every creation of an infantry, vehicle, aircraft or
  building object takes from the scenario random stream at the moment of creation, whatever else
  that creation draws (S-0202, claim 10). An object that is never created takes none.
- **Terrain smoothing.** The automatic replacement of terrain tiles at the borders between certain
  tilesets by their matching transition tiles, and of ramp tiles by their smoothed forms. It is part
  of every terrain recalculation: twice for every legal cell while a map loads, and again for a cell
  whenever something in play recalculates it. S-0010 owns it (claims 10 to 12).
- **Vein overlay, vein seed.** The overlay type at list index 126 is the vein overlay (S-0202,
  claim 10). A *vein seed* is a cell that holds the vein overlay with an overlay data value of 48 or
  more when the map's overlay streams have been read (claim 14). The veinhole, the stationary object
  that spreads veins around it in play, comes from a different overlay type; the vein and veinhole
  rules have no spec yet (a pointer only).
- **Ice-growth flag.** A per-cell flag stored in the terrain stream that allows ice to regrow on the
  cell. What it does in play belongs to the economy topic, which has no spec yet (a pointer only).
- **Early phase, late phase.** The two points in loading at which skirmish start positions are
  resolved (claim 20): the early phase before the initial fill, the late phase after all map objects
  are placed.
- **Farthest-sum rule.** The rule by which a player without a chosen start position is given one
  (claim 19).
- **House number.** A house's place in the creation order of claim 19, counting from 0: the
  players first, then the neutral house, then the special house.
- **Observer.** A human participant whom the lobby sets to watch the game instead of playing it.
  The original still creates a house for an observer, in the human part of the house order of
  claim 19, but an observer does not count towards the player count P of claim 17 and receives no
  starting units.
- **Passive house.** A house whose country is marked passive for multiplayer games by the
  country's passive field (Import mapping), as that field stands when the test is made (claims 28
  and 33 say which country data each test sees). A passive house takes no start position (claim 19) and
  is spared by the cleanup of claim 28. Which houses are passive depends only on that field.
- **Decoded stream.** The sequence of values that an encoded map block (terrain, overlay type,
  overlay data or preview) yields once decoded. This spec states the order and meaning of those
  values, not the byte encoding (value widths, compression, block layout), which is outside it
  (Questions for the project lead). A result that would depend on the byte encoding itself, such as
  a stream that ends partway through a value, is unspecified in the original. Our engine treats
  such a partial value as not present, exactly as if the stream had ended just before it, and the
  importer warns (a departure).
- **Crate type.** An overlay type that the rules data marks as a crate: an object that gives a
  random bonus to the unit that picks it up.
- **Cell animation.** An animation that the rules data attaches to an overlay type and that plays
  on each cell holding that overlay. An overlay type with a cell animation counts as displayable
  even when it has no artwork of its own (claim 12).
- **Unset base tile.** The value −1 that a fill base tile takes when no tileset supplies it
  (claim 6). It is an ordinary signed tile number, distinct from the no-tile marker 65535.
- **Stream position.** One of the 512 × 512 places (x, y), 0 ≤ x ≤ 511 and 0 ≤ y ≤ 511, that the
  overlay type and overlay data streams each give one value to (claims 12 and 13). A stream position
  that is a legal cell stands for that cell; the others have no effect.
- **Selected cell.** The cell that a coordinate pair (x, y) read from a terrain record or a
  cell-tag key acts on: the cell whose stream position index 512·y + x equals that of the pair
  (Formulas). For 0 ≤ x ≤ 511 it is (x, y) itself. A pair with x outside 0 … 511 selects another
  cell, for example (527, 10) selects (15, 11) (quirk `quirks.cell_index_alias`). A pair whose
  index lies outside 0 … 262143 selects no cell.
- **Mark.** The integer that each of the start positions 0 … 15 carries during start position
  resolution (claim 29).
- **Base cell.** The cell around which a house's starting units are placed: for a player, its
  start cell (claims 19, 20 and 29).
- **Header pair.** One of the numbered coordinate pairs of the header block (claim 25). An
  *applied pair* is a header pair numbered 9 … min(N_h, 16) that is present in the map, and so
  sets two marks (claim 29).
- **Distance sum.** For a free start position, the sum over all taken positions of the tabulated
  distance between their start cells (claim 19; defined under Formulas).
- **Plain test, height-aware test.** The two variants of the usable-cell test of claim 4: the
  plain test ignores the cell's height, and the height-aware test moves the local area's top and
  bottom edges down by the cell's height level (one more half-row for some slopes).
- **Authoring index space.** The numbering of a theater's tiles that map tile numbers refer to:
  each tileset's tiles are numbered consecutively, using the tileset's authored tile count where it
  declares one and its actual tile count otherwise (claim 8; Formulas).
- **Eligible house.** A house that is neither human-controlled nor passive (claim 28).
- **Empty tag.** A tag with no triggers, attached by a cell-tag entry whose value names no tag the
  map defines. It never sets anything off but occupies the cell like any other tag (claim 31).
- **Stand-in cell.** The one cell, existing on no map, that the original uses wherever a rule
  refers to a cell that does not exist (Edge cases, cell-tag key whose selected cell is not legal).

## Behaviour

### Map footprint and playable area

1. **Legal cells.** A cell (x, y) exists on a map of size W × H exactly when
   W < s ≤ W + 2H and −W < d < W, with s = x + y and d = x − y. The map therefore has 2H screen
   rows, s = W + 1 … W + 2H. Rows alternate between W cells (rows with s − W odd) and W − 1 cells,
   for H·(2W − 1) legal cells in total. The first two numbers of the map size entry (an origin)
   are read and ignored: the diamond always starts at the coordinate origin as above. Cells outside
   this set do not exist. A terrain record or cell-tag entry whose selected cell (Terms) is not a
   legal cell, and an overlay value at a stream position that is not a legal cell, have no effect
   (for cell tags, see also Edge cases). A pair with x outside 0 … 511 can still select a legal
   cell (Terms).
   A waypoint marker that names a non-legal cell is *not* rejected: it still counts as defined (claim 15) and
   can become a start cell (claim 17).
   - Backing: `measurable` (scenario M1; scenario M8b for the waypoint consequence)
   - Confidence: high (medium for the waypoint consequence, which no stock map exercises)

2. **Coordinate range.** Legal cells have 1 ≤ x ≤ W + H − 1 and 1 ≤ y ≤ W + H − 1. The original
   supports cell coordinates 0 … 511 on each axis, so maps with W + H > 512 are outside what it
   supports, and what it does with such a map is unspecified. Our engine treats every cell with a
   coordinate above 511 as non-legal (it does not exist; terrain records and overlay values that
   name it have no effect), and the importer warns (a departure).
   - Backing: `unmeasured` (the original's result is unspecified, so there is nothing to compare
     with. The limit follows from the supported coordinate range.)
   - Confidence: medium

3. **Visiting order.** Whenever the game makes one pass over all legal cells (the fill in claim 6
   uses this), it visits screen rows from top to bottom, s = W + 1 first. Within each row it visits
   cells in increasing x (left to right on screen). The first cell visited is (1, W). On every map
   with W ≥ 2 every legal cell is visited exactly once. Edge case: on a map with W = 1 the legal
   cells are (k, k) for k = 1 … H, one per odd-numbered screen row, and the rows in between hold no
   cell. Such a pass visits only (1, 1); the cells (2, 2) … (H, H) are not visited (quirk
   `quirks.narrow_map_partial_pass`).
   - Backing: `measurable` (scenario M2's visiting-order sub-test and scenario M2b, through the
     fill pattern)
   - Confidence: high (medium for the W = 1 edge case)

4. **Usable cells (local area).** A legal cell is usable when
   W + 2·Ly + h < s ≤ W + 2 + 2·(Ly + Lh) + h and 2·Lx − W < d < 2·(Lx + Lw) − W.
   Here h is 0 for the plain test. For the *height-aware* test, h is the cell's height level ℓ, plus
   one more when the cell is a slope and s < W + 4 + 2·Ly + ℓ.
   The height-aware test moves the top and bottom edges of the area down by one half-row per
   height level. A raised cell near the top edge is drawn higher on screen, so it drops out of the
   area. A raised cell just below the bottom edge is drawn higher, so it comes into the area.
   Unlike the map size, the local area's origin numbers (Lx, Ly) are used.
   - Backing: `measurable` (scenario M3)
   - Confidence: high for the inequalities. Which features use which variant is claim 5.

5. **What the local area limits.** Nearly every gameplay use of the local area applies the
   height-aware test. Cells outside the local area still exist and are drawn (terrain outside it
   is visible beyond the playable border), but play does not extend into them. The claim has two
   parts with different backing:
   - **5a.** The height-aware test limits the extent of the minimap, whether the mouse cursor
     offers an order on a cell, object placement checks, the "find a free nearby cell" search and
     ore growth.
     - Backing: `measurable` (scenarios M3 and M3b)
     - Confidence: medium
   - **5b.** The height-aware test also limits the choice of a map-edge cell for arrivals from
     off-map, the choice of the cells that two superweapons affect when fired (the superweapon
     that turns infantry into mutants and the one that takes control of units), aircraft firing
     positions, cliff destruction and the pathfinding zones. The preview image generated for a map
     uses the plain test. A few features use the plain test in some situations and the
     height-aware test in others; they are listed as an open question.
     - Backing: `unmeasured` here. A scenario for any of these features needs an expected result
       for the feature itself (where an arrival appears, which cells a superweapon affects, which
       route a unit takes, what the generated preview shows), and those results are defined only by
       the topic specs that own the features. Most of them are not written yet, and the one that is
       (S-0212, pathfinding) does not yet say which variant its zones use. Each owning topic spec
       states and measures its variant (Open questions).
     - Confidence: medium

### Terrain and height

6. **Initial fill.** Before the terrain stream is applied, every visited cell (Terms) is set to height level
   *L0* (the map's base level setting, default 0, reduced modulo 256 into the signed range
   −128 … 127) and to sub-tile 0. Its tile depends on the map's fill setting. If the setting is the
   word for water (compared case-insensitively), each visited cell gets the *water base tile* plus a
   ranged draw over 0 … 3. The range 0 … 3 has a power-of-two size, so each such ranged draw takes
   exactly one raw value from the scenario random stream. One draw is taken per visited cell, in the
   visiting order of claim 3: H·(2W − 1) draws when W ≥ 2, and 1 draw when W = 1. For any other
   value, or when the setting is absent, every visited cell gets the *clear base tile* and no random
   value is drawn. On a map with W = 1, the cells the pass does not visit stay blank cells: tile
   65535, sub-tile 0 and height level 0 (not L0), and they take no draw (quirk
   `quirks.narrow_map_partial_pass`). The fill's draws come after the early phase's draws (claim 20).
   The two base tiles are **not** taken from this map's theater. They are the base tiles of the
   theater whose tilesets were loaded most recently before this map's own, in the same program run
   (a previous map's), or tile 0 for both when no theater has been loaded since the program
   started. A theater's clear base tile is the start of its clear role, and its water base tile
   the start of its water role, as S-0010 (claim 4) defines role starts from the theater's
   clear-tile and water-set fields (Import mapping). For the fill this means:
   - A field that names a declared tileset gives the first tile of that tileset.
   - A field that is absent, or names a tileset number beyond the first missing one, gives the
     *unset base tile* −1.
   - A field that names exactly the first missing tileset number gives the theater's tile count
     (S-0010 claim 4, `quirks.role_one_past_end`). That switch therefore changes the fill base tile
     too: with it turned off, such a field gives −1 here as well. A fill tile number at or beyond
     the map's own theater's tile count is drawn as that theater's clear tile, and the first terrain
     recalculation replaces it by the no-tile marker 65535 on most cells (claim 8; S-0010 claims 7
     and 12). When the base tiles come from the map's own theater, which happens when the theater
     loaded before is the same theater, every fill tile built on such a base tile is drawn as the
     clear tile.
   - For the theater with index 5 in the theater list of Import mapping, the water base tile is
     −1 whatever that theater's water-set field says (S-0010, claim 4).

   Tile numbers here are signed integers, and the fill writes the exact sum. With a water base
   tile of −1, a water fill therefore writes the tile numbers −1, 0, 1 and 2. The number −1 is
   not the no-tile marker 65535, and its display is covered by claim 8. Cells covered by the
   terrain stream are unaffected, so all of this shows only on maps that omit cells from the
   terrain stream (quirk `quirks.stale_fill_tiles`).
   - Backing: `measurable` (scenarios M2 and M2b; one draw per visited cell through M2's
     visiting-order sub-test; scenario M9c for where the fill's draws sit in the stream relative to
     the early phase's)
   - Confidence: high for the draw count and order (medium for W = 1). Medium for the stale base
     tiles and the −1 rule (the load order is clear; what a negative tile looks like on screen is
     unspecified, claim 8).

7. **Terrain stream records.** The terrain stream is a sequence of cell records, each carrying
   (x, y, tile number, sub-tile, height level, ice-growth flag). A record naming cell (0, 0), which
   is never legal, marks the end of the stream: records after it have no effect. A record whose
   selected cell (Terms) is not legal has no effect. A record whose selected cell is legal sets all four of that
   cell's values. The height level is set outright, not added to the base level. If two records
   name the same cell, the later one wins. Cells with no record keep the initial fill. A record's
   height level lies in −128 … 127. The stream may also end without a (0, 0) record. Then the
   complete records take effect as above, and the effect of the last record depends on how much of it is
   present:

   | How much of the last record is present | Effect |
   |---|---|
   | Nothing (the data ends between two records) | none; the result is the same as with a (0, 0) record at that point |
   | x only | as if the coordinates were (x, 0), with the rows below. For x = 0 that is the end of the stream; for 1 ≤ x ≤ 511, (x, 0) selects no legal cell, so no cell changes |
   | The coordinates, selecting no legal cell | none |
   | The coordinates, selecting a legal cell, but no tile number | the selected cell's tile becomes the no-tile marker 65535; its other values keep the initial fill |
   | The coordinates, selecting a legal cell, and one or more later complete values | the complete values are applied; the values not present keep the initial fill |
   | Part of a value | unspecified in the original: its result depends on the stream's byte encoding (Terms, decoded stream). Our engine treats the partial value as not present, so the row above that matches what is complete applies; the importer warns |

   No stock map is cut off. Our engine reproduces the cases the table specifies, and the importer
   warns about a terrain stream that ends without a (0, 0) record.
   - Backing: `measurable` (scenario M4, including its height variant for the outright level and a
     negative level, and its selected-cell variant)
   - Confidence: high (medium for duplicates, negative levels and the cut-off records, which no
     stock map exercises)

8. **Tile number translation.** A record's tile number is either the "no tile" marker 65535, which
   is kept, or an index in the *authoring index space*. The authoring space differs from the
   theater's actual tile order only where a tileset declares that maps were authored against a
   different tile count than it now has. The translation is given under Formulas. When no tileset
   of the theater declares such a count, the translation is the identity: tile number m is tile m
   of the theater. The translated number is stored as is, even when it is at or beyond the
   theater's tile count; the first terrain recalculation (S-0010, claims 7 and 12) then replaces
   such a number by the marker 65535 on most cells. When the game decides which tile artwork to
   load:
   - The marker 65535 and numbers at or beyond the theater's tile count count as the theater's
     clear tile.
   - Negative numbers come from the initial fill of claim 6, or from a terrain record whose tile
     number is negative. They are not caught by that rule. The original's result is
     unspecified. Our engine treats a negative tile number like the marker 65535 (a departure),
     and the importer warns about a terrain record with a negative tile number.

   What these numbers mean for play (land type, slope, the cells that keep an out-of-range number)
   belongs to S-0010 (claims 7 and 8).
   - Backing: `measurable` (scenario M4b) for the identity case, and (its second part) for the
     clear-tile rule for numbers at or beyond the tile count. The translation is `measurable`
     with scenario M4c: no stock theater declares a different authored count, so M4c uses a named
     test file, a loose copy of a theater file in which one tileset declares one.
   - Confidence: medium for the translation, low for the art substitution

9. **Height in world units.** One height level is *level height* = 104 leptons of elevation. The
   surface of a flat cell at height level ℓ ≥ 0 lies at ℓ × 104 leptons. Below level 0 it lies one
   lepton higher, at ℓ × 104 + 1 (level −1 at −103, level −2 at −207), because the original forms the
   level's base height in floating point as ℓ × 104 + 0.5 and truncates it toward zero, whatever
   the rounding mode in force (Formulas). S-0106 owns
   the surface height, flat and sloped cells alike (its claims 2 and 3, Formulas F1), and the switch
   for the raised floor below level 0 (`quirks.negative_level_floor_raised`).
   - Backing: `measurable` (scenario M5; S-0106's scenario T1, its negative-level part, for levels
     below 0)
   - Confidence: high (medium below level 0, as in S-0106 claim 3)

10. **Legacy terrain streams.** Older encodings of the terrain stream (four legacy variants) also
    take effect if present. They rank in the order in which Import mapping lists them (the
    unnumbered legacy stream lowest, then legacy streams 2, 3 and 4), and the current stream ranks
    above all of them. Where streams cover the same cell, a higher-ranked stream's records count as
    later records than a lower-ranked stream's (claim 7: the later record wins), so the current
    stream's records win over every legacy stream's. The semantics of the legacy encodings are not
    covered here.
    - Backing: `unmeasured`. A test map would need a valid legacy stream, and the legacy encodings
      are not established by this spec or any other draft, so no such stream can be written from
      the specs. No stock map carries one that could be copied instead. The claim is kept so that an
      importer knows these sections exist and how they rank.
    - Confidence: low (medium for the ranking)

### Overlay

11. **Overlay is read only for format version ≥ 2.** If the map's format version is below 2 (it
    defaults to 0 when absent), the game reads no overlay at all, from either overlay stream.
    - Backing: `measurable` (scenario M6)
    - Confidence: high

12. **Overlay type stream.** The decoded stream holds 512 × 512 values, each an integer
    0 … 255, one per stream position (Terms), in order of increasing y and, within one y, of
    increasing x: counting from 0, the value at index 512·y + x belongs to stream position (x, y).
    The value 255 means "no overlay". Any other value is an index into the rules' overlay type list, which makes the list *order* part of
    the format. An overlay is created on a cell only when all of these hold: the cell is legal, the
    overlay type has artwork or a cell animation, and the game is a campaign mission or the type is
    not a crate type. Crate overlays placed on a map therefore never appear in skirmish or
    multiplayer games. Where the decoded stream is shorter than a position, that position has no
    overlay, and the importer warns about a short stream. Creating the
    overlay applies its normal placement effects (wall linking and similar). Some of them take draws
    from the scenario random stream while the map loads, in the order of the overlay type stream:
    an end marker that completes a low bridge, an overlay type with a cell animation, and the
    recalculation of the cell that every placement causes, which can start the cell's tile
    animation (S-0202, claim 10 and its table "Draws during map loading", states those draws).
    Placing the vein overlay takes no draw of its own here (claim 14 covers veins). For the overlay types
    at list indices 24, 25, 237 and 238, creation leaves the cell's own overlay data value as it
    was before the overlay was created.
    - Backing: `measurable` (scenarios M6, including its short-stream save, and M6c)
    - Confidence: high (the crate exclusion and the four data-preserving indices are medium until
      measured)

13. **Overlay data stream.** Same size and order. Every legal cell's overlay data value (0 … 255) is
    set from its value in the stream, whether or not the cell has an overlay. When the map has an overlay data
    stream, its values are final with respect to overlay creation and its placement effects: no
    data value that they set survives on any legal cell. The vein rebuild of claim 14 comes after the
    stream and changes the data values of vein cells again. Where the
    decoded stream is shorter than a position, the value is 0, and the importer warns. When the map has no overlay data
    stream at all, every cell keeps the data value it had after overlay creation (0 unless
    placement set one).
    - Backing: `measurable` (scenario M6, including its short-stream save, for the stream; scenario
      M6c variant A for the absent stream)
    - Confidence: high

14. **Veins are rebuilt from their seeds.** After the overlay streams have been read and the first
    terrain recalculation pass has run (S-0010, claim 12), every cell that holds the vein overlay
    (Terms) loses it: its overlay becomes none and its data value 0. The cells whose data value was
    48 or more before this, the *vein seeds*, then get veins again. The seeds are taken one after
    another in the reverse of the visiting order of claim 3. A seed that passes the vein placement
    test gets the vein overlay back as the centre of a vein pattern, and its edge neighbours are
    re-patterned, so that a flat seed whose four edge neighbours are flat, clear and empty ends up
    with veins on all five cells. The new data values come from draws on the scenario random
    stream, and S-0202 (claim 10, *vein patterns*) states those draws, their order within one seed and the data
    values they give. The test looks at the cell and its four edge neighbours, and it sees the
    cleared cells, the terrain after the first recalculation pass, every other overlay of the map
    and the veins that earlier seeds have already put back (the test itself belongs to the vein
    rules, which have no spec yet; a pointer only). A seed that fails the test stays empty.
    Consequences: a vein cell whose data value is below 48 keeps a vein after loading only when a
    passing seed next to it re-patterns it; and the vein cells' data values after loading are the
    ones these draws give, not the ones the overlay data stream gave (claim 13). The rebuild creates
    no veinhole; the veinhole object comes from another overlay type (Terms).
    - Backing: `measurable` (scenario M6b for the clearing, the threshold and the re-patterned
      neighbours; S-0202's scenario R8, part B, for the draws, their order within a seed and the
      reverse visiting order across seeds)
    - Confidence: medium (low for which neighbours a passing seed re-patterns when they are not
      flat, clear and empty)

### The waypoint markers

15. **Numbering and encoding.** A map has 702 waypoint markers, numbered 0 … 701. The waypoint number k is defined
    by an entry named with the decimal number k, with no sign and no leading zeros (other spellings
    are not found). Its value is a cell number n, decoded as x = n rem 1000 and y = n quo 1000
    (Formulas). The value 0 means "undefined", so cell (0, 0) cannot be a waypoint; it is never
    legal anyway. Entries named outside 0 … 701 are ignored. The 1000-based decoding applies at
    every format version. (Cell numbers in cell-tag entries switch to a 128-based decoding below
    format version 4; see Formulas, Constants and the cell-tag row of Import mapping.) Each decoded coordinate is reduced modulo 65536 into the
    range −32768 … 32767, so a value whose quotient exceeds 32767 wraps. Our engine reproduces the
    wrap, and the importer warns about such a value.
    A waypoint marker is not checked against the map: one naming a non-legal cell stays defined.
    - Backing: `measurable` (scenario M7, including its wrap map; scenario M17 variant B for the
      1000-based decoding below format version 4; scenario M8b for a non-legal cell)
    - Confidence: high (medium for the wrap)

16. **Home waypoint and initial camera.** The map names a home waypoint number, default 699. If that
    waypoint is undefined after the waypoint markers are read, it is defined at cell (c, c) with
    c = ⌊(W + H) / 2⌋, roughly the map centre. The initial camera centres on the home waypoint cell,
    at (256·x + 128, 256·y + 128) leptons. In skirmish the view then moves to the player's start
    (outside this spec).
    - Backing: `measurable` (scenario M7, first map and its defined-699 variant)
    - Confidence: high for the default and the centre rule, medium for the skirmish camera sequence

### Skirmish start positions

17. **The start list.** Let P be the number of players in the game: human non-observers plus
    computer players. Let K be the number of start waypoint markers defined *consecutively* from waypoint 0
    (the count stops at the first undefined one, at most 8). The game wants N = max(P, K) start
    cells. The start list holds, in increasing waypoint order, the cells of the defined waypoint markers
    among 0 … N − 1. If that gives fewer than N entries, random start cells are added until it has N
    (claim 18). Start position i means the i-th entry of this list, not waypoint i. With a gap in
    the start waypoint markers, positions after the gap shift down and the missing ones become random.
    Example: waypoint markers 0, 1, 3 and 5 are defined and there are 4 players. Then K = 2,
    N = 4, the list is [wp0, wp1, wp3, random], position 2 is waypoint 3 and position 3 is a random
    cell (waypoint 5 lies beyond N − 1 = 3, so it is not used).
    - Backing: `measurable` (scenario M8)
    - Confidence: high

18. **Random start cells.** Each missing entry is found by taking two ranged draws from the scenario
    random stream (each by the stream's range rule, which may consume more than one raw value).
    First u is drawn uniformly from 0 … (W + H − 11), then v uniformly from 10 … (W + H − 11).
    The candidate is x = v + 1, y = u + 11. For W + H ≥ 21 that makes x range over
    11 … W + H − 10 but y over 11 … W + H, an asymmetry (smaller maps: see Formulas). The
    candidate need not be legal. The game then searches outward from it for the nearest cell
    where an 8 × 8 cell area is clear for tracked vehicles (bridges allowed), using the general
    nearby-free-area search (placement rules, which have no spec yet; a pointer only). The entry
    is the cell found for the first candidate whose search succeeds. Each candidate whose search
    fails still uses up its pair of draws, and the original sets no limit on the number of candidates.
    On a map with no such area the original never finishes loading. That is bad data, and our engine
    does not reproduce the endless load (a departure, no quirk switch): after 1000 consecutive failed
    candidates (a fixed limit of our engine, not a constant of the original, so the outcome is
    deterministic) it refuses to start the
    game and reports that no start cell could be found, naming the map. The importer also warns when
    it can already tell, from the map's size and terrain alone, that no 8 × 8 clear area exists. A
    map on which some candidate succeeds is unaffected, and its draws are the original's. In the early phase (claim 20) the search runs on a map of
    blank cells.
    - Backing: `measurable` (scenario M9)
    - Confidence: high for the draws and ranges, medium for the search parameters

19. **Assigning positions to players.** Houses are created in a fixed order: human players in
    increasing order of their chosen colour index (on a tie, the earlier lobby slot first), then
    computer players in lobby-slot order, then the neutral house, then the special house. Players
    who chose a start position get that list entry. Players left on "random" then receive positions
    one at a time in house order:

    | Positions taken when the player's turn comes | Position the player gets |
    |---|---|
    | none | the result of one ranged draw over 0 … N − 1 (which may consume more than one raw value when N is not a power of two) |
    | at least one | the free position whose *distance sum* to all taken positions is largest. A tie, or all sums equal, goes to the lowest-numbered position. The distance sum is defined under Formulas: per pair, a tabulated square root of the squared cell distance, truncated to an integer. That is the truncated Euclidean distance for every pair less than 129 cells apart; from 129 cells on it is sometimes one less |

    Each position, once given, counts as taken for later players. The player's base cell, where its
    starting units are placed, is its start cell. Passive houses (Terms) get no position, whatever
    their place in the house order.
    In the early phase the farthest-sum rule uses the start cells of list 2 (claim 20); the
    late-phase assignments of claims 28 and 29 use list 3.
    Because of the tabulated square root, two positions whose exact distances differ can get equal
    terms, and the tie then goes to the lower-numbered position (quirk
    `quirks.table_root_distance`, owned by S-0204). The first such pair of distances is 128 and 129
    cells.
    - Backing: `measurable` (scenario M10; scenario M10b for the tabulated square root)
    - Confidence: high for the rule and the tabulated square root, medium for the colour-index
      ordering of humans

20. **Start positions are resolved in two phases, from three start lists.** The *early phase*
    runs after the map size, the header block, the local area and the waypoint markers are read, and
    before the initial fill (claim 6). The *late phase* runs after the terrain, the overlay and all
    pre-placed objects exist. Each phase builds start lists by claims 17 and 18, each build a fresh
    list with new random entries and new draws (one game mode places players its own way at the
    early phase; see Open questions):
    - The early phase builds list 1, from which players who chose a position get that entry, and
      then list 2, against whose cells the players left on random are assigned positions (claim 19).
    - The late phase builds list 3. Every player keeps the position index it got in the early
      phase, and its base cell becomes that index's entry in list 3. (Header pairs beyond the
      eighth can change this; see claim 29.)

    Observable consequences:
    - **Draw order.** On the scenario random stream, loading takes its draws in this order:
      list 1's random entries, list 2's random entries, the random-position draws of claims 19 and
      22 (in house order), the initial fill's draws (claim 6); then the draws of the map's content,
      among them **one creation draw for every pre-placed vehicle, aircraft, infantry and structure
      that is created** (claim 27), in the order the table "Start position resolution" (State and
      ordering) gives and S-0202's table "Draws during map loading" details; then list 3's random
      entries, then any late re-assignment draws of claims 28 and 29; then the starting units
      house by house, one creation draw for each, and after the last house one ranged draw over
      0 … 65535 whose value has no effect. The creation draw is the one every creation of such an
      object takes (S-0202, random streams, claim 10), so it is taken for map objects and starting
      units alike.
    - **What the early phase sees.** Every legal cell is a blank cell (Terms): no fill, no terrain,
      no overlay and no objects. Lists 1 and 2 therefore find their random entries by the search of
      claim 18 on a flat, empty map of clear land.
    - **Final cells versus assignment.** Random start cells that players actually start on are
      chosen against the real terrain (list 3). The farthest-sum choice, however, was made against
      list 2's cells. Changing only the map's fill setting (clear or water) changes the stream
      between list 2 and list 3, so it can change the final random start cells but never which
      player holds which position, except for a house that is assigned in the late phase (claims
      28 and 29).
    - Backing: `measurable` (scenarios M9 and M9b, statistical, for the three builds and the
      assignment against list 2; scenario M9c, which reads the seed with the random-state recorder
      of the measurement setup, for the exact draw counts of lists 1 and 2 and of the
      random-position draw, the place of the fill's draws after them, and the place of list 3's
      draws after the fill). The creation draw itself is measurable through S-0202's scenario R5
      (variant b), and the map-content draws through S-0202's scenarios for its table "Draws
      during map loading".
    - Confidence: high for the three builds and the draw order, including the place of the
      creation draws. Medium for the blank-cell terrain seen by the early phase (the cell state is
      clear; how the search treats it belongs to the placement rules, which have no spec yet)

21. **Player-at-position references.** Triggers and team definitions may name the house "player at
    position A" … "player at position H". These mean the house that holds start position 0 … 7 in
    the sense of claim 17. With no such player, the reference is empty. On a map whose header
    count N_h is 9 or more, the "References" rule of claim 29 replaces this one.
    - Backing: `measurable` (scenario M11; scenario M13b variants E and F for maps with N_h ≥ 9)
    - Confidence: medium (how an action resolves the reference is S-0011's, claim 23)

22. **Observers.** A human observer takes no starting units. When its turn comes in the house
    order of claim 19, it is shown a position:

    | Positions taken when the observer's turn comes | Position the observer is shown | Effect on later players |
    |---|---|---|
    | none | the result of one ranged draw over 0 … N − 1. This position counts as taken | the next player of any kind who needs a random position is given this same position, without a draw and without the farthest-sum rule (quirk `quirks.observer_position_handoff`) |
    | at least one | the lowest-numbered taken position | none: the observer takes nothing |

    - Backing: `measurable` (scenario M15)
    - Confidence: low

### Map selection data and preview

23. **Lobby player limit.** The lobby allows as many players as there are defined start waypoint markers
    among 0 … 7, counted in any pattern (gaps allowed). The lobby finds the entries by the same
    spelling rule as the game (claim 15: the plain decimal number, so an entry named "01" is not
    found). It uses a different "undefined" test from the game itself: an entry counts when it is
    present with any value other than −1, so a stored 0 *counts* here although the game treats it
    as undefined. If none counts, the limit
    is the random-map player count setting, or 8 when that is 0 or absent. The game mode may lower
    the limit further. The map's own minimum and maximum player settings are used to label the map
    in the map list (one number when they are equal, a range otherwise). No use of them to limit
    the lobby count is established. Whether the map list filters on them is an open question.
    - Backing: `measurable` (scenario M12; scenario M7 for the spelling rule)
    - Confidence: medium

24. **Preview image.** The preview is an RGB image whose width and height are the third and fourth
    numbers of the preview size entry; the first two are ignored. The decoded preview stream holds
    width × height pixels, each given as three integers 0 … 255 in the order red, green, blue. The
    pixels come row by row from the top row, and left to right within a row. If the stream ends
    before the last pixel, the image is still shown: every pixel whose three values are all
    present shows its decoded colour. What the remaining pixels show is unspecified in the
    original (they keep whatever the image held before); our engine makes them black (red, green
    and blue 0), and the importer warns (a departure). The preview appears in the map
    selection screen and on the loading screen.
    - Backing: `measurable` (scenario M13)
    - Confidence: high for the pixel order. Low for the short-stream result (the original shows a
      shortened preview, but there are signs that it counts the preview as one that failed to
      load, and no observable consequence of that, or of the shortening otherwise, is known).

25. **Preview start markers come from the header, not from the waypoint markers.** The numbered markers the
    selection screen draws over the preview come from a separate header block. It holds a marker
    count N_h, an origin (X0, Y0), an extent (Wh, Hh) and N_h coordinate pairs. Markers are drawn
    only when 1 ≤ N_h ≤ 8, labelled 1 … N_h. Pair i is drawn at the preview position given under
    Formulas. Header pairs 1 … 8 never affect where players start, and the header count affects it
    only through claim 29. A map whose
    header disagrees with its start waypoint markers shows markers in the wrong places. When the
    header block is absent, the count, origin and extent read as −1, so no markers are drawn. An
    extent of 0 (Wh = 0 or Hh = 0) with 1 ≤ N_h ≤ 8 makes the original crash when it draws the
    markers. Our engine draws no preview markers for such a map and changes nothing
    else about it. An importer warns (Edge cases). Exactly the pairs
    1 … N_h are read, whatever the count: a pair numbered above N_h is never read, even when the map
    holds it. With N_h above 8, the pairs from the ninth up to the N_h-th are therefore read, and
    those up to the sixteenth act as start position marks that can change which player holds which
    start position (claim 29, which also covers pairs from the seventeenth on; quirk
    `quirks.header_pairs_as_start_marks`).
    - Backing: `measurable` (scenario M13, including its extent-0 variant; scenario M13b variants C
      and D for which pairs are read)
    - Confidence: high for the marker source and the −1 defaults. Medium for the formula and the
      extent-0 crash.

### What a skirmish game reads

26. **Read set.** In a skirmish game the map contributes the parts in the table under State and
    ordering. It does **not** read the map's per-house definitions (credits, alliances, tech level
    and other house properties). Houses come from the lobby, plus the neutral and special houses.
    A map section whose name is also a country name is still read as a country override under
    S-0006, but only for country properties, never for house properties.
    - Backing: `measurable` (scenario M14)
    - Confidence: medium

27. **Pre-placed objects and their owners.** The map's object lists (vehicles, aircraft, infantry,
    structures) are read, and each object's owner is looked up by exact, case-sensitive match against
    the names of the houses in the game. The neutral and special houses carry the names of their
    countries, which the original finds by fixed names (Import mapping).
    Player houses carry generic placeholder names: one for human players and one for computer
    players (in online games a human house carries the player's online name instead); if several
    players share a placeholder, the first in house order matches. An object whose owner matches no
    house is silently dropped. In particular, objects owned by "player at position X" or by a
    country name other than the neutral and special houses' own names do not appear in skirmish.
    The same applies in multiplayer games. In addition, in a skirmish or multiplayer game a
    pre-placed **structure** whose owner is a human player's house is dropped, while vehicles,
    aircraft and infantry with that owner appear (for multiplayer games this is a departure, Edge
    cases). A dropped object is never created, so it takes no creation draw (claim 20).
    - Backing: `measurable` (scenario M14, including its human-owned structure and vehicle; for
      multiplayer games, scenario M16, where the neutral house's pre-placed objects appear in a
      network game). The online-name rule is `unmeasured`: online games need an online game
      service, which the measurement setup does not include.
    - Confidence: medium

28. **Unused-house cleanup.** If the game has no computer players and fewer human slots (observers
    included) than the rules' maximum player count, a cleanup runs once all map objects are placed,
    before the late phase of claim 20. (One special game mode skips it; it is not identified here,
    see Open questions.) Call a house *eligible* when it is neither human-controlled nor passive.
    The cleanup removes a house from the game exactly when the house is eligible and the house just
    before it in house order (claim 19) was not removed (quirk `quirks.cleanup_skips_next_house`).
    In a run of eligible houses that are next to each other in house order, the first, third, fifth and so on
    are removed and the others are kept. With a removed house go:
    - **its pre-placed objects.**
    - **its triggers:** the trigger instances (S-0011, Terms) whose trigger definition's owner
      country is the house's country and that exist at that moment, which are those of tags created
      by cell tags and by pre-placed objects. A tag created at the end of loading (S-0011, claim
      34), which includes every tag that no cell tag and no pre-placed object names, is created after
      the cleanup and keeps all its trigger instances. Such a trigger has no owner house when no
      remaining house plays its owner country (S-0011, Terms), and what it can still do is S-0011's.
    - **the house itself.** A removed house takes no position in the late phase, receives no starting
      units and owns nothing from then on.

    Passivity for this test is judged by the country data after every rules layer of this game,
    including the map's country overrides (claim 26). In a skirmish or multiplayer game the only
    houses besides the players are the neutral and special houses, which come after every player,
    the neutral house first. Human houses are never removed, so the cleanup can reach only those two
    houses. A map makes either country non-passive with a country override:

    | Country data in force for the neutral country | for the special country | Removed by the cleanup | Kept although not passive |
    |---|---|---|---|
    | passive | passive | none | none |
    | not passive | passive | the neutral house | none |
    | passive | not passive | the special house | none |
    | not passive | not passive | the neutral house | the special house (`quirks.cleanup_skips_next_house`) |

    A neutral or special house that the cleanup has not removed, and whose country is not passive
    in the country data in force, takes part in the late phase. That is the case when the cleanup
    does not run, and for a special house that the cleanup keeps because it removed the neutral
    house. When such a house took no position
    in the early phase (because the country data the early phase saw marked its country passive,
    and that data never includes the map's overrides, claim 33), it is assigned one by claim 19 against
    list 3 after every player, because it comes after them in house order. With both houses in that
    situation, the neutral house is assigned before the special house. The players' start positions
    are therefore unchanged. A neutral or special house that the early phase already saw as
    non-passive is instead assigned a position there, by claim 19 like a player on "random" (after
    every player, in house order), and, unless the cleanup removes it, keeps that position index in
    the late phase like a player. Its start cell is then a base cell like a player's, so it receives
    starting units there, after the cleanup, by the starting-unit rules (no spec yet; a pointer
    only). When every position below N is already taken at its turn, the third unspecified case of
    claim 29 applies to it.
    - Backing: `measurable` (scenario M16: variants A to C for the condition, the removal and the
      late-phase position; variant D for the kept special house; variant E for a trigger of a tag
      created at the end of loading)
    - Confidence: high that a removed house is gone with its objects and that the house after a
      removed one is kept; medium for the condition; low for the trigger removal (including that tags created
      at the end of loading escape it), the use of the map's override, the late-phase position and
      its starting units

### Skirmish start positions: header pairs as start marks

29. **Header pairs beyond the eighth act as start position marks.** Every start position
    0 … 15 carries a *mark*, an integer. The mark −1 means the position is free. A mark equal to
    the house number (Terms) of a non-passive house means that house holds the position. Any other
    mark means the position is taken by no player. Without header pairs beyond the eighth, every
    mark is −1 when the early phase starts, and claims 19 and 20 apply as written. When the header
    count N_h is 9 or more, each header pair i with 9 ≤ i ≤ min(N_h, 16) that is present in the map
    (an *applied pair*) sets the marks of positions 2(i − 9) and 2(i − 9) + 1: its first number
    becomes the mark of the first position and its second number the mark of the second. A pair in
    that range that is absent, and every pair numbered above N_h (which is not read, claim 25),
    leaves those two marks as they are. The pairs are applied twice, with these consequences:
    - **Early phase.** When chosen positions are given, each position's mark is, in order of
      precedence: the house number of the player who chose it, whatever a header pair says,
      otherwise the number its applied pair gives, otherwise −1. A non-passive house that
      holds one or more positions has the highest-numbered of them as its position. A house that
      holds none is assigned by claim 19, in house order, with every position whose mark is not −1
      counting as taken, and the position it is given carries its house number. So the
      random-position draw happens only when every one of positions 0 … N − 1 is marked −1, and the
      farthest-sum rule sums over every position below N whose mark is not −1.
    - **Late phase.** A position covered by an applied pair carries the pair's number again,
      whatever the early phase left there. Every other position keeps its early-phase mark. Each
      non-passive house starts at the list 3 entry of the highest-numbered position it holds. A
      non-passive house that holds no position (a mark overwrote its position, or the house took
      none in the early phase because it counted as passive then, claim 28) is assigned anew by
      claim 19 against list 3, in house order. Its random-position
      draw, if any, comes after list 3's random entries on the scenario random stream. Claim 20's
      "every player keeps the position index it got in the early phase" therefore holds only for
      houses whose position was not overwritten.
    - **References.** "Player at position X" (claim 21) means the house whose house number equals
      the mark of position X, which may be a passive house, and is empty when no house has that
      number.

    | Phase | Marks in force | Non-passive house that holds a position | House that holds none |
    |---|---|---|---|
    | early | chosen positions: the chooser's house number; else applied pairs; else −1 | the highest-numbered position it holds | claim 19 against list 2; every mark other than −1 counts as taken |
    | late | applied pairs again; else the early-phase marks | base cell = list 3 entry of the highest-numbered position it holds | claim 19 against list 3; its draws follow list 3's random entries |

    Examples, with 2 players on "random", 4 start waypoint markers and N_h = 9: pair 9 = (1, 0)
    places house 1 at position 0 and house 0 at position 1 in every game, with no random-position
    draw. With the two players on chosen positions 0 and 1 instead, pair 9 = (−1, −1) frees both
    positions before the late phase, so both players are assigned anew there: the first by a
    random-position draw, the second by the farthest-sum rule.

    Three cases are unspecified in the original: a house that holds a position numbered N or above,
    header pairs from the seventeenth on (read only when N_h ≥ 17), and a house that needs a position by the farthest-sum
    rule when every position below N is already taken (marks can cause this). In the first two cases
    our engine ignores the mark or pair concerned. In the third, the house gets no start position
    and no starting units. These are departures, and the importer warns in all three cases.
    - Backing: `measurable` (scenario M13b: variants A to D for the marks and which pairs are read,
      variant E for the references and for the highest-numbered position a house holds, variant F
      for a mark that names no house)
    - Confidence: medium (low for the references)

### Further rules for map entries

30. **Absent and partial rectangle entries in a map.** The map size, local area and preview size
    entries are four-number entries, read by the rule of S-0003 claim 10 (quirk
    `quirks.absent_rect_is_zero`). For maps that rule means:
    - An absent map size entry gives W = H = 0, so the map has no legal cells.
    - An absent local area entry gives (0, 0, 0, 0), so no cell is usable: no cell offers an order
      and the minimap shows no playable area.
    - An absent preview size entry gives a preview of width and height 0, so no preview is shown.
    - An entry written with an empty value (only spaces, or nothing, after the equals sign)
      creates no entry (S-0001 claim 11), so it behaves exactly like an absent one: an empty map
      size entry leaves no legal cells, an empty local area entry leaves no usable cell, and an empty
      preview size entry shows no preview.
    - The field's own default is used only for numbers that a non-empty value does not supply.
      For the map size that default is (1, 1, 50, 50); for the local area it is (0, 0, W, H), the
      whole map. A value from which no number can be read gives the whole default: a map size
      entry "x" gives a 50 × 50 map, and a local area entry "x" gives the whole map. A value with
      fewer than four numbers keeps the default for the missing trailing numbers: a map size entry
      "0,0,30" gives W = 30 and H = 50, and a local area entry "2,4,26" gives (2, 4, 26, H).
    - For a value with missing trailing numbers, the early phase (claim 20) sees a different local
      area from the rest of the game. In the early phase the missing numbers come from the playable
      area in force before this map was read, not from (0, 0, W, H). Every later use of the local
      area sees the value completed with the default above. Only the early phase's random start
      search (claim 18) can therefore see the difference. Which playable area is in force before the map is read is
      not established (Open questions).
    - Backing: `measurable` (scenario M3c for the local area and the partial map size entry,
      scenario M13 for the absent preview size). The unreadable value is probed on the local area
      only; the map size entry is read by the same rule (S-0003 claim 10). The absent or empty map size entry
      is `unmeasured`: a map without legal cells has no terrain, no start cells and nothing to
      observe, and the original may not survive loading it, so no recordable result is expected.
      What the original does with such a map is unspecified. Our engine imports it as a map with
      no legal cells, does not start a game on it and reports why, and the importer warns (a
      departure). The early-phase read of a partial local area is `unmeasured`:
      its effect depends on the playable area left before the map, which is not established.
    - Confidence: high for the absent, empty and partial local area, medium for the map size and
      the preview, low for the early-phase read

31. **Cell tags.** Each entry of the map's cell-tag section attaches a tag to one cell. The entries
    take effect in entry order, but each entry carries the value that its key has under the rule
    for repeated keys in S-0002 claim 4, not necessarily the value written on its own line: when
    the same key is written twice, both occurrences take effect, each in its own place in entry
    order, with the value of the occurrence that wins by S-0002's rules, and the other value is
    never seen. An entry's key is read leniently as its leading integer: an optional sign
    followed by digits, with anything after them ignored. The result is decoded as a cell number,
    using the base that applies at the map's format version, and each decoded coordinate wraps
    modulo 65536 as for the waypoint markers (Formulas). The entry acts on the selected cell of the
    result (Terms), so a key whose decoded x lies outside 0 … 511 can act on another legal cell.
    The entry's value is a tag name, compared with the names of the map's tags without regard to
    letter case:
    - A value that is one of the two "no tag" markers (Import mapping, compared without regard to
      letter case) attaches nothing.
    - A value that names a tag defined by the map attaches that tag. The tag is created at this
      point if it does not exist yet (S-0011, claim 34, which gives the draws this takes).
    - Any other value, a name that the map does not define, attaches an *empty tag* of that name: a
      tag with no triggers, which never sets anything off but occupies the cell like any other tag.
      Two entries with the same undefined name attach the same empty tag.

    A line with an empty value creates no entry (S-0001 claim 11), so it attaches nothing. A cell
    keeps the first tag attached to it, so a later entry with a *different* key that decodes to the
    same selected cell has no effect, even when the first tag is an empty one. For example, the keys "10015"
    and "010015" are distinct names that both decode to cell (15, 10), and the first of them in
    entry order decides the cell's tag. For two occurrences of one identical key, the cell instead
    gets the winning occurrence's tag, which need not be the first occurrence's. These rules apply
    to every map, user maps included. The importer warns about a key written twice (as S-0002 does
    for every repeated key), about two distinct keys that select the same cell, about a value
    that names no defined tag, about a key whose decoded coordinates wrap, and about a key whose
    decoded x lies outside 0 … 511. A key whose selected cell is not legal is an edge case (Edge
    cases). What an object entering a tagged cell sets off is
    S-0011's (claim 18).
    - Backing: `measurable` (scenario M17; variant B for the 128-based decoding below format
      version 4; variant C for an identical repeated key; variant D for an undefined tag
      name; variant E for the wrap; variant F for a selected cell other than the decoded one)
    - Confidence: high for "first tag wins", medium for the lenient key, the identical repeated
      key, the empty tag of an undefined name, the wrap and the selected cell

32. **Map trigger variables.** The map's variable-name section defines the map's own trigger
    variables, each a named on/off state that triggers can test and change (S-0011, claim 22). Only the first 100
    entries, in entry order, take effect, and each occurrence of a key written twice counts as an
    entry of its own. Each entry carries the value that its key has under S-0002 claim 4, not
    necessarily the value written on its own line, so both occurrences of a key written twice yield
    the winning occurrence's value. Later entries are ignored, even when they name variables 0 … 99. A line
    with an empty value creates no entry (S-0001 claim 11): it is never read and does not count
    towards the first 100 entries. An entry's key, read leniently as its leading integer like a
    cell-tag key (claim 31), is the variable's number. Its value is a comma-separated list: the
    first item is the variable's name, and the second item, when present, is read as a leading
    integer that gives the variable's initial state: set when the integer is not 0, clear when it
    is 0. Items after the second are ignored. Every entry read applies to the variable of its
    number, so when two read entries with distinct keys give the same number (for example the keys
    "5" and "05"), the later one replaces the name, and replaces the initial state when it has a
    second item. (For one identical key written twice, both occurrences give the same value, so
    the order between them does not matter.)

    A variable's name comes only from this map's entries: a variable that no entry names has no
    name. Its state, by contrast, is not reset by reading the map: a variable that no entry names,
    and one whose entry has no second item, keep the state they had before the map was read. The original clears every state when the program
    starts, so in the first game of a program run that state is clear. Whether anything clears the
    states again at a later game start is not established (Open questions).

    Three cases are unspecified in the original. Each such entry still counts as one of the first
    100 entries. Our engine gives each a safe, deterministic result, and the importer warns (a
    departure):
    - a variable number outside 0 … 99: the entry has no effect.
    - a name longer than 39 characters: the name is cut to its first 39 characters, and the
      initial state is read as usual.
    - an entry whose value holds no item at all (only commas): the entry has no effect.

    The importer also warns about a key written twice and about two distinct keys that give the
    same variable number. These rules apply to every map, user maps included.
    - Backing: `measurable` (scenario M18) for the initial states, the 100-entry limit, the
      later-entry rule and the state left by an entry without a second item in the first game of a
      program run. The names themselves are `unmeasured`: they only label variables for map
      authors, and nothing in play shows them.
    - Confidence: medium (low for the state in the first game)

### Skirmish start positions: country data seen by the early phase

33. **Country data seen by the early phase.** In a skirmish or multiplayer game the early phase's
    passivity test (claims 19, 20 and 29) does not see this game's layered rules. Its country data
    is the country data already in force from before this game's rules layering, with the base
    rules' country list, their general section and every country's own section read once more over
    it. This game's full rules layering (base rules, language rules file, game-mode file, S-0006
    claim 1) and the map's rules overrides, country overrides included, take effect only after the
    early phase, and are in force for the cleanup and the late phase (claim 28). Where the extra
    read falls in loading is given by the first row of the start position resolution table (State
    and ordering). Consequences:
    - For a country key that the base rules give, the early phase sees the base rules' value. For
      one they omit, it sees the value already in force, by S-0006 claim 3's rule for absent keys:
      in a later game of the same program run, that can be the value an earlier game's game-mode
      file or map gave it.
    - The early phase never sees this game's game-mode file or the map's country overrides. A map
      that overrides a country's passive field changes passivity only for the cleanup and the late
      phase. When the override makes a player's country passive, that player still takes a
      position in the early phase, so the position counts as taken for the players after it. In
      the late phase the house is passive: it receives no starting units.
    - Conversely, a country that an earlier game's map made passive, and whose passive field the
      base rules do not give, is passive for the early phase of the next game even when that game
      has no such override. A player of that country then takes no position in the early phase. It
      is assigned one in the late phase (claim 29, a non-passive house that holds no position),
      after every house that already holds one.
    - Backing: `measurable` (scenario M16b: variant A for the map override, variant B for the
      carry-over from an earlier game, variant C, with a named test file, for the game-mode file)
    - Confidence: medium

## Formulas

**Cell number decoding.** For an integer n:
x = n rem 1000 and y = n quo 1000. Both are integer operations: quo truncates toward zero and rem
takes the sign of n. A negative n therefore gives negative coordinates, which are never legal. The
encoding used when the game writes a map is n = x + 1000·y.
Legacy cell-tag decoding below format version 4: x = n rem 128, y = n quo 128, same rounding.
Wrap, for waypoint markers (claim 15) and cell-tag keys (claim 31) at either base: each decoded
coordinate c is then replaced by w(c) = ((c + 32768) mod 65536) − 32768, where mod is the
non-negative remainder, so w(c) lies in −32768 … 32767 and equals c whenever c already does.
Integers, exact. Example: n = 65546015 gives x = 15 and y = w(65546) = 10.

**Selected cell (Terms; claims 7 and 31).** For a coordinate pair (x, y) of signed integers in
−32768 … 32767 (a terrain record's coordinates, or a cell-tag key after decoding and wrap), let
i = 512·y + x. If 0 ≤ i ≤ 262143, the selected cell is (i rem 512, i quo 512), which is (x, y)
whenever 0 ≤ x ≤ 511; otherwise there is no selected cell. Integers, exact. Example: (527, 10)
gives i = 5647 and the cell (15, 11). Without `quirks.cell_index_alias`, a pair with x outside
0 … 511 selects no cell.

**Usable cell (claim 4).** With s = x + y, d = x − y, integers throughout:
W + 2·Ly + h < s ≤ W + 2 + 2·(Ly + Lh) + h and 2·Lx − W < d < 2·(Lx + Lw) − W.
For the height-aware test, h = ℓ + δ, where ℓ is the cell's height level and δ = 1 if the cell is a
slope and s < W + 4 + 2·Ly + ℓ, otherwise δ = 0.

**Home waypoint default cell.** c = ⌊(W + H) / 2⌋ (integer, W + H ≥ 0), cell (c, c).

**Random start candidate (claim 18).** u ~ U[0, W + H − 11], then v ~ U[10, W + H − 11], drawn in
that order from the scenario random stream, each with the stream's inclusive-range rule. When an
upper bound is below the lower bound, the range rule takes the two bounds in either order. This
happens for u when W + H < 11 and for v when W + H < 21, so on maps with W + H < 21 the range of v
is W + H − 11 … 10. Candidate cell (v + 1, u + 11), integers.

**Distance sum (claim 19).** For a free position p and the set T of taken positions, with cells
(x_p, y_p) and (x_t, y_t):
S(p) = Σ over t in T of ⌊r(n_t)⌋, with n_t = (x_p − x_t)² + (y_p − y_t)² (an exact integer).
Here r is the *table square root*: r(0) = 0, and for n ≥ 1, with e = ⌊log₂ n⌋ and z = ⌊e / 2⌋,
r(n) = 2^z × ⌊√m′ × 2²³⌋ / 2²³, where m′ = ⌊n × 2^k / 4^z⌋ / 2^k, with k = 13 when e is even and
k = 12 when e is odd.
So n / 4^z, which lies in 1 … 4 (4 excluded), is truncated toward zero to a multiple of 2⁻¹³ (below 2)
or 2⁻¹² (from 2 on), and its square root is truncated toward zero to 24 significant binary digits.
All of this is exact rational arithmetic; ⌊r(n)⌋ is an integer. The chosen position is the
lowest-indexed p with maximal S(p).

Properties, for every squared distance a supported map can produce (claim 2): ⌊r(n)⌋ is never more
than ⌊√n⌋ and never less than ⌊√n⌋ − 1. It equals ⌊√n⌋ for every
n below 16641 = 129², so for every pair less than 129 cells apart. The first n where it is one less
is 16641 itself (a distance of exactly 129 cells gives a term of 128). For larger n the shortfall
occurs at scattered values, not only at perfect squares.

Number domain in the original: the squared distance is formed in double-precision floating point,
the root comes from a table of single-precision values, and each term is truncated toward zero to an
integer, whatever the rounding mode in force. Every intermediate value is exact for any squared distance a
supported map can produce (below 2²⁴), so no rounding mode enters: the result does not depend on the
round-toward-zero mode (glossary; owned by S-0205, number domains), and the formula above
reproduces it exactly in integer arithmetic. It is the same tabulated square root that S-0105 (F1)
uses, restricted to integer inputs, and the same function that S-0204 (Terms, *table square root*,
claim 5 and F1) uses for distances between objects. Without `quirks.table_root_distance` (S-0204) the
term is ⌊√n⌋, the largest integer whose square is at most n.

**Tile number translation (claim 8).** The theater's tilesets, in declaration order, define a list of
pairs (t_i, δ_i), one for each tileset whose authored tile count A differs from its actual tile count
C. Here t_i is the tileset's first tile number in the authoring space plus A, and δ_i = C − A. The
authoring space numbers each tileset's tiles consecutively using A where it is given, and C
otherwise. With n pairs, for a map tile number m ≠ 65535 let
j = the largest k in 0 … n such that t_i ≤ m for every i ≤ k (so j = 0 when t_1 > m or there are
no pairs).
Then
m' = m + (δ_1 + δ_2 + … + δ_j), which is m itself when j = 0.
Integers only. m = 65535 is kept.

**Preview marker position (claim 25).** For header pair (xi, yi), preview drawn at size (pw, ph) with
its top-left at (left, top) in screen pixels:
px = trunc(trunc((xi − X0)·1000 / Wh) · pw / 1000) + left − 9
py = trunc(trunc((yi − Y0)·1000 / Hh) · ph / 1000) + top − 6.
All integer; trunc rounds toward zero (this differs from ⌊ ⌋ when xi < X0 or yi < Y0). The inner
quotient is the pair's displacement from the origin in per-mille (‰) of the marker extent, and the outer
one turns it into pixels of the drawn preview. The −9/−6 terms place the marker graphic's corner.

**Floor height (claim 9).** For a flat cell of height level ℓ (an integer), the floor height is
104 × ℓ leptons when ℓ ≥ 0 and 104 × ℓ + 1 leptons when ℓ < 0. In the original this is 104 × ℓ + 0.5,
formed in floating point and truncated toward zero to an integer; the formula above gives the same
integers exactly. The truncation is toward zero whatever the rounding mode in force
(`quirks.chop_rounding` of S-0205 does not apply to it). S-0106 (Formulas F1) owns it, with the slope
rise added.

**Fill tile (claim 6).** For a visited cell, tile = B + r, where B is the applicable base tile
(a signed integer, possibly −1) and r is 0 for a clear fill or the value of the cell's ranged draw
over 0 … 3 for a water fill. Exact signed integer sum, no rounding.

**Header pair marks (claim 29).** Header pair i, for 9 ≤ i ≤ min(N_h, 16), sets the marks of start
positions 2(i − 9) and 2(i − 9) + 1. Integers; a mark is taken exactly as written in the pair.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| waypoint count | 702 | waypoint markers | numbered 0 … 701 |
| start waypoint markers | 8 | waypoint markers | waypoint markers 0 … 7 |
| default home waypoint | 699 | waypoint number | when the map names none |
| cell number base | 1000 | cell-number units per unit of y | n = x + 1000·y |
| legacy cell-tag base | 128 | cell-number units per unit of y | below format version 4 |
| minimum format version for overlay | 2 | format version | overlay streams ignored below this |
| format version that switches cell-tag decoding | 4 | format version | |
| default format version | 0 | format version | when absent |
| supported coordinate range per axis | 512 | cells | coordinates 0 … 511 |
| coordinate wrap | 65536 | cells | each decoded coordinate of a waypoint marker or cell-tag key is reduced modulo 65536 into −32768 … 32767 (claims 15 and 31) |
| overlay stream size | 512 × 512 | values (stream positions) | the value at index 512·y + x belongs to stream position (x, y) |
| no-overlay value | 255 | overlay type stream value | |
| no-tile marker | 65535 | tile number | in the terrain stream's tile number |
| level height | 104 | leptons per height level | engine constant; see S-0101 |
| water fill variants | 4 | tiles | water base tile + 0 … 3 |
| fill base tiles before any theater is loaded | 0 | tile number | clear and water base tile alike; see quirk `quirks.stale_fill_tiles` |
| overlay indices whose data value survives creation | 24, 25, 237, 238 | overlay list index | creation leaves the cell's data value unchanged (claim 12) |
| vein overlay index | 126 | overlay list index | claim 14; S-0202 claim 10 |
| vein seed threshold | 48 | overlay data value | data ≥ 48 → vein seed (claim 14) |
| random start margin | 10 | cells | see the random start candidate formula |
| random start clear area | 8 × 8 | cells | tracked-vehicle passability, bridges allowed |
| preview marker scale | 1000 | ‰ of the marker extent | per-mille step of a header pair's displacement (Formulas, preview marker position) |
| preview marker corner shift | (9, 6) | pixels | subtracted from the marker position: −9 horizontally, −6 vertically (Formulas) |
| lobby fallback player count | 8 | players | no start waypoint markers, no random-map count |
| map-list default minimum players | 2 | players | loose map files only |
| map-list default maximum players | 4 | players | loose map files only |
| map size default for numbers the entry does not supply | (1, 1, 50, 50) | cell widths × cell heights (W × H = 50 × 50) | used only for numbers a non-empty value lacks (claim 30); origin numbers ignored; an absent or empty entry gives zeros, see quirk `quirks.absent_rect_is_zero` |
| blank cell tile | 65535 | tile number | the no-tile marker; see Terms |
| unset base tile | −1 | tile number | a base tile with no tileset to supply it (claim 6) |
| theater whose water base tile is always unset | 5 | theater index (Import mapping) | fixed rule of the program (claim 6) |
| start positions that carry a mark | 16 | start positions | positions 0 … 15 (claim 29) |
| free mark | −1 | mark value | claim 29 |
| header pairs that act as marks | 9 … min(N_h, 16) | header pair number | pairs above N_h are not read; pairs 17 and beyond: unspecified (claim 29) |
| variable-name entries read | 100 | entries | the first 100 in entry order (claim 32) |
| map trigger variables | 100 | variables | numbered 0 … 99 (claim 32) |
| longest specified variable name | 39 | characters | longer names: unspecified in the original; our engine cuts them to 39 characters and the importer warns (claim 32) |
| water fill draw range | 0 … 3 | tile numbers | added to the water base tile; one raw value per ranged draw |

## State and ordering

**Skirmish read set and its observable order.** Each row is something the map supplies. The rows
are grouped by topic. Neither the groups nor the order of the rows say anything about when a part
is read; only the last column gives ordering, and only as consequences a player or a measurement
can see.

| Map part (role) | Read in skirmish? | Effect | Observable ordering consequence |
|---|---|---|---|
| ***Footprint and theater*** | | | |
| Format version | yes | selects overlay reading and legacy decodings | — |
| Map size and local area | yes | legal cells, usable cells | — |
| Theater | yes | tilesets, palettes | the fill uses the tiles of the theater loaded before this map (claim 6, quirk `quirks.stale_fill_tiles`). Unknown names: see quirks |
| Lighting | yes | see S-0104 | — |
| ***Terrain and overlay*** | | | |
| Terrain stream (current and legacy) | yes | tiles, heights | overrides the initial fill. The last record for a cell wins |
| Overlay type and data streams | only at format version ≥ 2 | overlays | the data stream's values replace every data value that overlay creation set (claim 13); the vein rebuild of claim 14 comes after both streams |
| Tube (tunnel) definitions | yes | tunnels | — |
| Terrain objects (trees and similar) | yes | terrain objects | — |
| Smudges | yes | smudges | — |
| ***Markers and start positions*** | | | |
| The waypoint markers | yes | waypoint cells, home waypoint, start list | the early phase already uses them (claim 20) |
| Header block (marker data) | yes | preview markers; pairs 9 … min(N_h, 16) are start position marks (claim 29) | its marks are in force for the early phase and are applied again for the late phase (claim 29) |
| ***Houses and pre-placed objects*** | | | |
| Vehicles, aircraft, infantry, structures | yes | pre-placed objects (claim 27) | the late phase's list 3 is built on a map that already holds all of them (claim 20) |
| Per-house definitions | **no** | — | houses come from the lobby |
| Player name setting | no | campaign only | — |
| ***Rules, settings and triggers*** | | | |
| Rules override sections | yes | see S-0006 | pre-placed objects already follow the overrides |
| Map-wide settings (growth switches, free radar, crates, initial time and similar) | yes | map-wide game settings | — |
| Team, script, task-force, trigger, tag and AI-trigger definitions | yes | triggers and tags: S-0011; teams, scripts, task forces and AI triggers: the team system, which has no spec (a pointer only) | — |
| Cell tags | yes | attaches tags to cells (claim 31) | for one cell, the first entry wins, even with an empty tag; a key written twice yields its winning occurrence's value both times (claim 31) |
| Variable names | yes, first 100 entries | map trigger variables: names and initial states (claim 32) | — |
| ***Map selection*** | | | |
| Preview size and stream | map selection screen only | preview image | — |
| Map list name, digest, minimum and maximum players, game modes, official flag | map selection screen only | listing | — |

**Start position resolution.** The rows are the five points of loading at which start positions
and base cells are decided, or which take draws between those decisions, in the order in which
they occur. For each point the table gives what is already in force there, what is decided there, and the draws it takes from the scenario random
stream. Every draw of a row comes after the draws of the rows above it, and within a row the draws
come in the order listed. Nothing else in the table fixes an order: an implementation must
reproduce the draw order and the consequences that claims 20, 28, 29 and 33 state.

| Point in loading | Already in force | What is decided | Draws, in stream order |
|---|---|---|---|
| Early phase (claim 20) | this map's size, header block, local area and waypoint markers; the houses, in the order of claim 19; the country data of claim 33 (the data in force before this game's rules layering, with the base rules' extra read over it); every legal cell a blank cell | the marks of the applied header pairs (claim 29); players who chose a position get that entry of list 1, and a chosen position's mark is its chooser's house number whatever a header pair says; houses on "random" get positions by claim 19 against the cells of list 2 | list 1's random entries; list 2's random entries; the random-position draws of claims 19 and 22, in house order |
| Initial fill (claim 6) | the early phase's decisions | the fill of every visited cell | one per visited cell, for a water fill only |
| Map content (claims 7 to 14, 27 and 31) | the fill | the cell tags, the terrain, the overlay, the vein rebuild, the terrain objects, the pre-placed objects and the smudges | the draws that S-0202 (random streams, its table "Draws during map loading") lists between the initial fill and the late phase, in its order. Among them, in this relative order: the tags created by cell tags (S-0011, map triggers, claim 34); the overlay placement draws of claim 12 and the tile animations of the first terrain recalculation pass (S-0010, claim 12); the vein pattern draws of claim 14; then the draws of the pre-placed objects: all vehicles' draws, then all aircraft's, then all infantry's, then all structures', and within one kind in entry order; and last the tile animations of the second terrain recalculation pass. Every pre-placed object that is created takes one creation draw (S-0202 claim 10), and it comes before the draws of that object's tag (S-0011 claim 34) and of its placement. An object that claim 27 drops takes no draw. An object that is created but cannot be placed does not appear in the game, yet its creation draw has been taken |
| Late phase (claim 20) | the terrain, the overlay and every pre-placed object; this game's full rules layering with the map's overrides (claim 33); the unused-house cleanup of claim 28, already applied when its condition holds | the marks of the applied header pairs, applied again (claim 29); every player keeps its position index, and its base cell is that index's entry in list 3, which is built on the real terrain; a non-passive house without a position (its position was overwritten by a mark, or it was passive in the early phase, claim 28) is assigned anew by claim 19 against list 3; a house that the cleanup removed takes no part (claim 28) | list 3's random entries; the re-assignment draws of claims 28 and 29 |
| Starting units | every base cell | units appear around each base cell (starting-unit rules, no spec yet; a pointer only) | house by house, in house order (claim 19): that house's starting units, each taking one creation draw (S-0202 claim 10), plus whatever else the starting-unit rules draw for that house; then, after the last house, one ranged draw over 0 … 65535 (one raw value) whose value has no effect. Later loading steps (crates, the end-of-loading tags of S-0011 claim 34) draw after these |

## Edge cases and quirks

Every quirk switch below defaults to the original (YR) behaviour described first; the *fixed* or
*alternative* behaviour is what the switch selects when turned off.

- **`quirks.narrow_map_partial_pass`.** On a map with W = 1, a pass in the visiting order reaches
  only cell (1, 1) (claim 3). The initial fill therefore leaves the cells (2, 2) … (H, H) as blank
  cells at height level 0 and takes a single draw (claim 6). *Fixed behaviour:* visit every legal
  cell, so every cell gets the fill and one draw is taken per legal cell. This changes the random
  stream on water-filled W = 1 maps.
- **`quirks.absent_rect_is_zero`.** If the map size, local area or preview size entry is absent,
  or written with an empty value (which creates no entry, S-0001 claim 11), it reads as
  (0, 0, 0, 0), not as the field's default. An absent or empty map size leaves a map with no legal
  cells. An absent or empty local area leaves no usable cells. An absent or empty preview size gives
  an empty preview. The default (1, 1, 50, 50 for the size, the whole map for the local area) is
  used only for the numbers that a non-empty value does not supply: a value with no readable
  number ("x") gives the whole default, and missing trailing numbers ("10,10") keep the default
  for the missing ones (claim 30). *Fixed behaviour:* an absent or empty entry gives the field's
  default.
- **`quirks.start_waypoint_gap`.** The start list skips undefined start waypoint markers, and the count K
  stops at the first gap (claim 17), so a gap shifts positions and creates random starts even when
  enough start waypoint markers exist further on. *Fixed behaviour:* position i is waypoint i whenever
  waypoint i is defined, and only undefined waypoint markers become random.
- **`quirks.lobby_counts_zero_waypoint`.** The lobby counts a start waypoint stored as 0 as defined,
  but the game treats it as undefined and gives that slot a random start (claims 15, 23). *Fixed
  behaviour:* the lobby uses the same "undefined" test as the game.
- **`quirks.preview_markers_from_header`.** Preview markers are taken from the header block, which
  can disagree with the real start waypoint markers (claim 25). *Fixed behaviour:* draw markers at the start
  waypoint cells, projected onto the preview.
- **`quirks.random_start_asymmetric_range`.** The random start candidate's y range runs 10 cells
  further than its x range and can leave the map (claim 18). *Fixed behaviour:* draw both
  coordinates from 11 … W + H − 10. This changes which cells come out. Each attempt still takes two
  ranged draws, but the number of attempts can change, because a different candidate may need more
  or fewer retries to find a clear area.
- **`quirks.double_start_resolution`.** Start positions are resolved in two phases from three start
  lists, and the farthest-sum choice is made against the early phase's list 2, built on blank cells
  (claim 20). The extra draws shift the scenario random stream. *Fixed behaviour:* build one list,
  after the terrain is loaded, and assign every position against it. This changes the random stream,
  so it is incompatible with replaying original games.
- **`quirks.skirmish_drops_country_owned_objects`.** In skirmish, pre-placed objects whose owner is a
  country name (other than the neutral and special houses' names) or "player at position X" vanish. Objects owned by the generic player placeholder
  names go to the first matching player (claim 27). *Fixed behaviour (mod option):* resolve "player
  at position X" to that player and country names to the first player of that country.
- **`quirks.no_overlay_below_format_2`.** A map without a format version (or below 2) loses all its
  overlay: no ore, veins, walls, bridges or veinholes (claim 11). *Fixed behaviour:* read the overlay streams
  regardless of the format version.
- **`quirks.skirmish_drops_map_crates`.** Crate overlays placed in a map never appear in skirmish
  or multiplayer games (claim 12). This looks deliberate (random crates replace them). *Alternative:* keep them.
- **Water fill consumes random draws.** A map whose fill setting is water advances the scenario
  random stream by one draw per visited cell during loading (claim 6). This is not a quirk switch: it
  is part of the stream's history and must be reproduced for replay compatibility.
- **`quirks.stale_fill_tiles`.** The initial fill uses the clear and water base tiles of the theater
  loaded last in the program run, or tile 0 before any theater was loaded, instead of this map's
  theater (claim 6). On a fresh start a water-filled map without a terrain stream shows tiles 0 … 3
  of its theater, not water. After a theater-5 map it gets the tile numbers −1 … 2 (claim 8 covers
  the negative one). *Fixed behaviour:* the fill uses the base tiles of this map's own theater,
  with the unset rules of claim 6 still applying. The number of random draws is the same
  either way.
- **`quirks.header_pairs_as_start_marks`.** With a header marker count N_h above 8, all N_h pairs are
  read, and header pairs 9 … min(N_h, 16) act as start position marks (claim 29). They can fix which house holds a position,
  block a position for every player, and move a player away from the position it chose. Pairs
  from the seventeenth on are ignored by our engine (claim 29). *Fixed behaviour:* read at most 8
  pairs, so every mark starts free and claims 19 and 20 apply as written. This changes which
  player holds which position on such maps, and how many draws the assignment takes. An importer
  warns about a header count above 8 either way.
- **`quirks.observer_position_handoff`.** When an observer is reached before any position is
  taken, the position it is shown is handed to the next player who needs a random position,
  without a draw (claim 22). *Fixed behaviour:* the position shown to the observer is not reserved
  and counts as free for the players. The next player is assigned by claim 19 as if no observer
  were present, taking its own draw. This changes the scenario random stream and the positions in
  games with observers.
- **`quirks.role_one_past_end`** (shared with S-0010, claim 4). A theater's clear-tile or
  water-set field that names exactly the first missing tileset number gives that theater's tile
  count as its clear or water base tile (claim 6). A fill that uses this theater's base tiles
  therefore writes the tile count, plus the ranged draw for a water fill (Formulas, fill tile),
  and claim 8 decides what such a number shows. Theater 5's
  water base tile is −1 either way (claim 6). *Fixed behaviour:* such a field gives the unset base
  tile −1, as a field that names a tileset number beyond the first missing one does. The number of
  random draws is the same either way.
- **`quirks.cell_index_alias`.** A terrain record or cell-tag key whose x coordinate lies outside
  0 … 511 acts on the cell with the same stream position index 512·y + x, which can be a legal cell
  other than the one the coordinates name (Terms, *selected cell*; claims 7 and 31). *Fixed
  behaviour:* such a pair selects no cell, so the record or entry has no effect. The importer warns
  about such a pair either way. Stock maps hold no such pair.
- **`quirks.table_root_distance` (owned by S-0204).** This spec has no switch of its own for the
  table square root. The farthest-sum rule follows S-0204's switch, which also governs the distances
  between objects. With the switch on (the original), each pair is measured with the table square
  root of Formulas, which falls one short of the truncated Euclidean distance for some pairs 129 or
  more cells apart (claim 19). Ties it creates go to the lower-numbered position. With it off, each
  term is ⌊√n⌋, the exact truncated distance. For start positions this changes which player holds
  which position only on maps with start positions at least 129 cells apart, and never the number
  of draws.
- **`quirks.negative_level_floor_raised` (owned by S-0106).** This spec has no switch of its own
  for the floor height below level 0. With the switch on (the original), the surface of a flat
  cell at a negative height level ℓ lies at 104 × ℓ + 1 leptons, one lepton above 104 × ℓ (claim 9,
  Formulas). With it off, it lies at 104 × ℓ leptons at every level. S-0106 owns the switch and
  its effect on sloped cells. Stock maps use no negative levels.
- **`quirks.cleanup_skips_next_house`.** The unused-house cleanup keeps every house whose
  predecessor in house order it removed, even when that house is neither human-controlled nor
  passive (claim 28). In a skirmish or multiplayer game this matters only
  when both the neutral and the special country are non-passive in the country data in force: the
  neutral house is removed and the special house stays, keeps its objects and triggers, and takes
  part in the late phase. *Fixed behaviour:* every eligible house is removed, whatever happened to
  the house before it, so both houses are removed.
  No draw depends on the switch directly; a kept special house can take a late-phase position and
  starting units, with their draws (claims 20 and 28).
- **Start waypoint on a non-legal cell.** Such a waypoint is still defined and becomes a start
  cell (claims 1, 15 and 17), and the player's base cell is that cell. Where its starting units
  then appear is not established here and belongs to the starting-unit rules, which have no spec yet. The importer warns.
- **Human-owned pre-placed structures in multiplayer games.** The original drops a pre-placed
  structure whose owner is a human player's house only on the computer of the player who holds
  that house, and creates it on every other computer, so the computers disagree about which
  structures exist (and about the creation draws, claim 20). Our engine drops such a structure on
  every computer, which is the skirmish rule of claim 27 applied to every human player's house (a
  departure; not a quirk switch).
- **Cell-tag key whose selected cell is not legal.** This covers a key with no selected cell and
  a key whose selected cell is not a legal cell (Terms). In the original such an entry is not
  simply ignored. Under the first-tag rule of claim 31, all such entries of a map act as if they
  named one and the same *stand-in* cell, a cell that exists on no map. The first of them in entry
  order whose value attaches a tag creates that tag at this point, with the draws of S-0011
  (claim 34), and counts as one of the tag's carriers. Every later entry of this kind has no
  effect.
  The original's result is nevertheless unspecified. Every other rule of the original that refers
  to a cell that does not exist uses the same stand-in. So two things depend on state outside this
  map: whether the stand-in is still free when a later game of the same program run reads its cell
  tags, and whether anything in play can raise a tag through it. Our engine ignores every such entry: it creates no tag there,
  attaches nothing and counts no carrier, so the tag is created where S-0011 (claim 34) would create
  it without the entry. The importer warns (a departure).
- **Unknown theater name.** A theater name that is not one of the six known names is accepted as
  "no theater": the map then has no valid theater, and what the original does next is
  unspecified. Our engine treats the name as absent, so the map gets the default theater
  (Import mapping), and the importer warns (a departure).
- **Home waypoint number outside 0 … 701.** Such a number is never defined, so the centre rule of
  claim 16 applies to it, with an unspecified result in the original. Our engine treats the number
  as absent, so the home waypoint is 699 and claim 16 applies to that waypoint, and the importer
  warns (a departure).
- **Overlay index beyond the overlay type list.** The original's result is unspecified. Our engine
  treats the value like 255, so that position has no overlay (its overlay data value still comes
  from claim 13), and the importer warns (a departure).
- **Negative height levels** are representable in a terrain record (claim 7). Scenario M4 checks
  level −1. Stock maps use only levels of 0 and above. Our engine reproduces them, and the importer
  warns about a negative level.
- **Header extent of 0.** A header block with 1 ≤ N_h ≤ 8 and a marker extent Wh or Hh of 0 makes
  the original crash when the map selection screen draws the preview markers (claim 25). This is
  not a quirk switch, since no engine reproduces a crash. Our engine draws no preview markers for
  such a map and treats the rest of the map normally (preview image, start positions and
  everything else unchanged). An importer warns.

## Import mapping

The first table lists keys in the map file; a second table lists keys of other files that this
spec relies on. Section names are given in brackets. Parse forms follow S-0003. Defaults are
the original's absent-key behaviour.

| Key | Concept | Parse form | Default when absent |
|---|---|---|---|
| [Map] Size | W = third number, H = fourth. The first two are ignored | four comma-separated integers | (0,0,0,0), see `quirks.absent_rect_is_zero`; an empty value is the same as absent. Numbers a non-empty value does not supply come from 1,1,50,50 (claim 30) |
| [Map] LocalSize | local area (Lx, Ly, Lw, Lh) | four integers | (0,0,0,0) (quirk); an empty value is the same as absent. Numbers a non-empty value does not supply come from 0,0,W,H (claim 30) |
| [Map] Theater | theater: TEMPERATE, SNOW, URBAN, DESERT, NEWURBAN, LUNAR (case-insensitive) → theaters 0 … 5 | string | TEMPERATE. An empty value also gives TEMPERATE. Unknown names: our engine uses TEMPERATE and the importer warns (Edge cases) |
| [Map] Level | base height level L0 of the initial fill | integer | 0 |
| [Map] Fill | initial fill. Only the value Water (case-insensitive) differs from the default | string | Clear |
| [IsoMapPack5] | current terrain stream (records of claim 7) | encoded block (encoding outside this spec) | none: initial fill only |
| [IsoMapPack], [IsoMapPack2] … [IsoMapPack4] | legacy terrain streams | encoded blocks | none |
| [OverlayPack] | overlay type stream | encoded block | none |
| [OverlayDataPack] | overlay data stream | encoded block | none: data values stay as overlay creation left them |
| [Waypoints] k | waypoint k, k = 0 … 701, key spelled as a plain decimal | cell number (integer) | undefined (also when the value is 0) |
| [Basic] HomeCell | home waypoint number | integer | 699. A number outside 0 … 701: our engine uses 699 and the importer warns (Edge cases) |
| [Basic] AltHomeCell | alternative home waypoint (campaign use) | integer | 699 |
| [Basic] NewINIFormat | format version | integer | 0 |
| [CellTags] n | tag on cell n (claim 31). The key is a cell number, parsed leniently as a leading integer, decoded 1000-based at format version 4 and above, 128-based below it, wrapped modulo 65536 per coordinate, and acts on its selected cell (Terms, Formulas). Entries take effect in entry order, each with the value its key has under S-0002 claim 4. A cell keeps the first tag given to it. Tag names compare without regard to letter case. The value <none> or none attaches nothing; a value naming no defined tag attaches an empty tag that still occupies the cell; a line with an empty value creates no entry (S-0001 claim 11). The importer warns about a repeated key, about distinct keys that select the same cell, about a decoded x outside 0 … 511, about an undefined tag name, about a wrapped coordinate and about a non-legal cell (Edge cases) | string (tag name) | no tag on any cell |
| [Header] NumberStartingPoints | preview marker count N_h | integer | −1 (no markers) |
| [Header] StartX, StartY | marker origin X0, Y0 | integer | −1 |
| [Header] Width, Height | marker extent Wh, Hh | integer | −1. An explicit 0 makes the original crash when markers are drawn; our engine then draws no markers and an importer warns (Edge cases) |
| [Header] Waypoint1 … WaypointN | marker pairs (xi, yi), read for i = 1 … N_h only (N = N_h, the marker count); pairs 9 … min(N_h, 16) are start position marks (claim 29) | two integers, by S-0003 claim 9 (a component that cannot be read takes the value in force, and the importer warns) | pairs 1 … 8: (0,0). Pairs 9 … 16: the marks stay as they are. Pairs above N_h: not read |
| [Header] NumCoopHumanStartSpots | read; no skirmish effect established | integer | 4 |
| [Preview] Size | preview width = third number, height = fourth | four integers | (0,0,0,0) (quirk); an empty value is the same as absent |
| [PreviewPack] | preview pixels (claim 24) | encoded block | no preview |
| [Basic] Name | map list name | string | "No Name" (loose map files) |
| [Basic] MinPlayers, MaxPlayers | shown in the map list only | integer | 2 and 4 (loose map files) |
| [Basic] GameMode | game modes the map is listed under | string list | empty |
| [Basic] Official | official-map flag | boolean | false |
| [Digest] | map digest for multiplayer comparison | string | "No Digest" |
| [RandomMap] NumPlayers | lobby fallback player count (claim 23) | integer | 0 → 8 |
| [Basic] TiberiumGrowthEnabled, VeinGrowthEnabled, IceGrowthEnabled | growth switches (economy topic, no spec yet) | boolean | true, true, true |
| [Basic] TiberiumDeathToVisceroid | visceroid switch | boolean | true |
| [Basic] FreeRadar | radar without a radar building | boolean | false |
| [Basic] TruckCrate, TrainCrate | crate switches | boolean | false, false |
| [Basic] FillSilos | read; effect not established here | boolean | false |
| [Basic] MultiplayerOnly | read; effect not established here | boolean | false |
| [Basic] InitTime | read; meaning not established here | integer | 10000 (unit not established, since the meaning is not) |
| [Basic] Percent | read; meaning not established here | integer | 0 (unit not established, since the meaning is not) |
| [VariableNames] | map trigger variables (claim 32): the first 100 entries; key = variable number, read as a leading integer; value = name, then optionally the initial state. The importer warns about a repeated key, about distinct keys that give the same number, and about the three cases of claim 32 that our engine replaces | comma-separated list: string, integer (not 0 = set) | no variables. No second item: the state is left as it was (clear in the first game of a program run, claim 32) |
| [Units], [Aircraft], [Infantry], [Structures], [Terrain], [Smudge], [Tubes] | pre-placed content (claim 27; their formats have no spec yet) | lists | none |
| [Houses] and per-house sections | not read in skirmish | — | — |
| [Basic] Player | not read in skirmish | — | — |

Keys from other files that this spec relies on (theater file and rules file):

| Key | Concept | Parse form | Default when absent |
|---|---|---|---|
| theater file [General] ClearTile | tileset whose first tile is the clear base tile (claim 6) | integer tileset number | none: clear base tile −1 |
| theater file [General] WaterSet | tileset whose first tile is the water base tile (claim 6); ignored for theater 5 | integer tileset number | none: water base tile −1 |
| theater file [TileSetNNNN] TilesInSet, LastTilesInSet | actual tile count C and authored tile count A of a tileset (Formulas, tile translation) | integer | A absent: no translation pair for that tileset |
| rules overlay type: Crate | crate type (claim 12) | boolean | not a crate type |
| rules overlay type: its cell-animation key | cell animation (claim 12) | animation type name | none |
| rules country: MultiplayPassive | passive house (claims 19, 28) | boolean | false when the country is created; when a later read or rules layer omits the key, the value already in force is kept (S-0006 claim 3) |

House names matched by pre-placed objects in skirmish (claim 27): the neutral and special houses
answer to their country names (Neutral, Special). The original finds those two countries by
these fixed names, so they are engine constants, not stock data. Human players (outside online games) answer to the placeholder
<human player> and computer players to Computer. The player-at-position names used by triggers and
teams are <Player @ A> … <Player @ H>, matched case-sensitively. These names, and the map-list
defaults "No Name" and "No Digest", and the two "no tag" markers <none> and none of the cell-tag
values (claim 31), are fixed text inside the original's program (engine constants), not game text
from the data files.

## Measurement scenarios

All scenarios run under the measurement setup (glossary): an unmodified install with no engine
add-ons, stock data, and tiny purpose-made test maps (e.g. a 20 × 20 flat clear map) with format
version 4, unless stated otherwise. No loose file overrides shipped data, except a named test file
that a scenario lists (M4c, M16b variant C), which is removed after that scenario's runs. A
presentation-only display compatibility layer may be used, and the measurement record names it
generically, by what it does. Positions are read from the game's own coordinate display or
by placing a marker unit. "Runs" counts game launches. Deterministic results need one run per
variant. Map sizes are given as the third and fourth numbers of the size entry (W × H), with the
first two numbers 0.

**Default players and map settings.** Unless a scenario says otherwise:
- It is a skirmish game with 1 human and 1 computer player on fixed positions 0 and 1. Its map
  defines start waypoint markers 0 and 1 on legal cells, at least 3 cells apart, and no other start
  waypoint markers. No random start cell is then needed. This matters on the smallest maps
  (M1, M2, M2b), where an 8 × 8 clear area cannot fit and a random start would never finish
  loading (claim 18).
- Every scenario keeps its participants, players and observers alike, within the lobby limit of
  claim 23. Each scenario that departs from the default states its start waypoint markers.
- Scenarios that read the minimap (M1, M3, M3c) set the map's free-radar switch, so the minimap is
  shown without a radar building.
- Scenarios that read tiles, heights or overlays away from the players' starting units give the
  map a trigger that reveals the whole map to the human player at the start. M11, M13b variants
  E and F, M16, M17 and M18 use a reveal as their observable and have no such trigger.

**Statistical sub-tests.** The scenario random stream starts from a different state in every
run, and the measurement setup never fixes that state (glossary, *measurement setup*,
statistical runs). A result that depends on it is therefore measured over many runs, and the
sub-test states the number of runs and the statistic that confirms or falsifies the claim. Such
sub-tests are marked *(statistical)*. Where a sub-test can avoid the dependence altogether, by
checking a relation that holds for every starting state, it does so and needs one run. Exact draw
counts and where a group of draws sits in the stream change no distribution, so no statistical
sub-test can show them. They are measured instead with the random-state recorder of the
measurement setup (glossary), which reads the running original's seed without changing it, so
that the stream can be computed by S-0202 and compared with what the game shows (scenario M9c).

- **M1, legal cells (claim 1).** Map size W × H = 10 × 6 (size entry 0,0,10,6), format version 4.
  In the overlay stream, put an ore overlay at (x, y) = (1, 10) (s = 11, d = −9, legal) and one at
  (1, 9) (s = 10, not legal). 1 run. *Expected:* the first ore cell exists, the second does not. A
  minimap cell count can confirm the H·(2W − 1) = 6 · 19 = 114 legal cells. *Falsified if* ore
  appears on the non-legal cell, the legal one is missing, or the count is not 114.
- **M2, visiting order and water fill (claims 3, 6).** A 10 × 6 map with the fill set to water and
  no terrain stream, plus the same map with a clear fill. Its theater is neither the snow theater
  nor theater 5. Before running, check in the theater data that the water base tile of the snow
  theater (claim 6) has a different tile number from the water base tile of the water map's
  theater; choose the water map's theater so that they differ, otherwise the after-snow sub-test
  cannot tell the stale tiles from the map's own. 2 fresh game launches of the water map.
  *Expected:* on each launch every legal cell shows one of tiles 0 … 3 of the map's
  theater (not water, because no theater was loaded before it in that program run), and the
  pattern differs between launches. Then, without restarting the game, play a snow map and then the
  water map again (1 run). *Expected:* the second time, the fill tiles are the snow theater's water
  tile numbers (plus 0 … 3) read in the water map's theater. *Falsified if* the first launch shows
  water, a cell shows a tile outside the four, or the fill does not change after the snow map.
  *Theater-5 sub-test:* without restarting, play a map of
  theater 5 (Import mapping), then the water map again (1 run). *Expected:* no cell shows the water
  tiles seen after the snow map. The cells hold tile numbers −1 … 2, so they show tiles 0, 1 or 2 of
  the water map's theater, or the −1 tile, whose display is unspecified (claim 8); the run may
  therefore show damaged graphics or end abnormally. *Falsified if* water tiles appear.
  *Distribution (statistical):* 40 fresh launches of the water map, recording the draw value r = 0 … 3
  (tile number minus the base tile) of every legal cell, 4560 values in all. *Expected:* the four
  values are equally frequent, and the pair (value of a cell, value of the next cell in the
  visiting order of claim 3) is uniform over its 16 values. *Falsified if* a chi-square test at
  the 1 % level rejects either uniformity.
  *Visiting order (one run, no dependence on the starting state):* a 15 × 15 map (435 legal
  cells) with the fill set to water and no terrain stream, played as the first game of a fresh
  program run, so that the draw values show as tiles 0 … 3. Record every cell's value and number the
  cells 0 … 434 in the visiting order of claim 3. Each raw value of the stream is the bitwise
  exclusive-or of the raw values 250 and 147 places before it, and every bit position, the lowest
  two included, obeys that relation on its own (S-0202, claim 2); a ranged draw over 0 … 3 keeps the
  two lowest bits of exactly one raw value (S-0202, claim 4). *Expected:*
  for every k from 250 to 434, the value of cell k equals the bitwise exclusive-or of the values
  of cells k − 250 and k − 147. *Falsified if* any of these 185 equations fails (a wrong visiting
  order, or a cell that takes other than one draw, breaks about three in four of them).
  *Inconclusive if* the four tiles cannot be told apart on screen.
- **M2b, narrow map (claims 3, 6, quirk `quirks.narrow_map_partial_pass`).** A map of size 1 × 5 (legal
  cells (1, 1) … (5, 5)), base level 2, fill set to water, no terrain stream. It is played after
  another water-filled map of the same theater, a theater other than theater 5, in the same
  program run, so that the fill shows real water tiles (claim 6). 1 run. *Expected:* cell (1, 1) is at height level 2 and
  shows a water tile. Cells (2, 2) … (5, 5) are at height level 0 and show no water tile (they hold
  the no-tile marker, which shows as the theater's clear tile by claim 8). The height levels are
  the deciding observable; the tiles confirm them. *Falsified if* cell (1, 1) is not at
  level 2, or any of (2, 2) … (5, 5) is at level 2 or shows a water tile. That the pass then takes
  exactly 1 draw is not observable without the starting state; it follows from the unfilled cells
  and from one draw per visited cell, which M2's visiting-order sub-test checks. Start waypoint markers 0 and 1 are at (1, 1) and (5, 5). If the game does not
  finish loading (the starting units have almost no room), the run is inconclusive. *Second
  variant (claim 6, modulo and case):* the same map with base level 258 and the fill value
  spelled "wAtEr", played under the same conditions, 1 run. *Expected:* the same result as the
  first variant: cell (1, 1) at height level 2 (258 reduced modulo 256) and showing a water tile.
  *Falsified if* cell (1, 1) is at any other level or shows no water tile.
- **M3, usable area (claims 4, 5).** Map 30 × 30 with local area (2, 4, 26, 22). For each of the
  four inequalities, probe one cell just inside and one just outside: does the cursor offer a move
  order, and where is the minimap border? Repeat with a level-4 plateau touching the top and bottom
  edges. 2 runs (flat, plateau). *Expected:* edges exactly as in claim 4, moved down by 4 half-rows
  on the plateau. *Falsified if* any probe cell lands on the other side of its edge.
- **M3b, other users of the height-aware test (claim 5).** M3's plateau map, with these additions.
  A pre-placed construction vehicle (the vehicle type of the human player's side that deploys into
  a construction yard) is owned by the human placeholder name; claim 27 keeps pre-placed vehicles
  of human players, while it would drop a pre-placed structure. It stands on the plateau a few cells
  below the moved top edge, far enough inside that the deployed yard lies wholly inside the area.
  An ore field on the plateau has an edge that touches the moved top edge. The human's start
  waypoint marker is on the opposite side of the map, far from the top edge, and the computer
  player's is in another corner. The human player gets the lobby's highest starting-credits
  choice. 1 run of about 15 minutes at a fixed game speed. Deploy the vehicle, build the structures
  that a vehicle factory requires, then the factory next to the yard on the side of the top edge.
  Probes:
  - try to place a structure whose foundation includes a plateau cell just outside the moved top
    edge, and one just inside, both within the distance from the base at which placement is
    otherwise allowed.
  - build vehicles until the cells around the factory are full, and record where new vehicles
    are put.
  - record which cells the ore spreads to.

  *Expected:* placement is refused on the outside cell and allowed on the inside one; no new
  vehicle and no new ore appears on a cell outside the moved edge. *Falsified if* any probe acts on
  a cell outside the moved edge, or refuses a cell inside it for this reason. *Inconclusive if* the
  vehicle does not deploy, or the computer player's attacks reach the base before the probes are
  done.
- **M3c, absent, empty and partial rectangles (claim 30).** M3's flat map in four variants of its
  local area entry: removed; written with an empty value; "x" (no readable number); and "2,4,26"
  (no fourth number). 4 runs. *Expected:* with the entry removed, and equally with the empty
  value, no cell offers a move order and the minimap shows no playable area. The "x" entry
  behaves like (0, 0, 30, 30), the whole map. The partial entry behaves like (2, 4, 26, 30), with
  M3's probes moved to the new bottom edge. A fifth variant has the map size entry "0,0,30" (no
  fourth number, so W = 30 and H = 50) and the local area entry "0,0,30,50", and ore placed at
  cell (50, 50) (s = 100, d = 0), which is legal when H = 50 but not when H = 30 (s would have to
  be at most 90). Its start waypoint markers are those of M3's map, which are legal at either
  height. 1 run. *Expected:* the ore appears. *Falsified if* the removed or the empty entry leaves
  any cell usable, the "x" entry differs from the whole map, the partial entry's bottom edge is
  not at Lh = 30, or the ore is missing.
- **M4, terrain records (claim 7).** Base level 1. A terrain stream holding two records for one cell
  (different tiles), a record for a non-legal cell, and a (0, 0) record followed by one more record
  for a legal cell. 1 run. *Expected:* the later duplicate's tile shows, the non-legal record changes
  nothing, and the record after (0, 0) has no effect. A second map whose stream has no terminator and
  ends right after the tile number of its last record (a legal cell, record height level 3), 1 run.
  *Expected:* that cell shows the record's tile at height level 1 (the fill's level), and the map
  loads. *Falsified if* the first duplicate shows, the record after the terminator is applied, the
  cut-off cell is at level 3 or lacks the record's tile, or the second map fails to load.
  A further cut-off variant uses a water fill. It is played after another water-filled map of the
  same theater, a theater other than theater 5, in the same program run, so that the fill shows real water tiles (claim 6). Its
  stream ends right after the complete coordinates of its last record (a legal cell, record tile a
  clear land tile, record height level 3). 1 run. *Expected:* the cell shows the theater's clear
  tile (the no-tile marker, claim 8), not water, at height level 1. *Falsified if* the cut-off cell
  shows water or sits at height level 3. Three last cut-off variants each hold complete records
  for a few legal cells and no terminator. The first ends right after its last complete record
  (between two records), the second right after the x of one more record (an x between 1 and
  511), the third right after the complete coordinates of one more record that names a non-legal
  cell with x between 0 and 511. 1 run each. *Expected:*
  in each, the map loads, the complete records are applied, and no other cell differs from the
  initial fill.
  *Falsified if* the map fails to load, a complete record is not applied, or any other cell
  changes. Streams that end partway through a value are unspecified (claim 7) and are not tested.
  *Selected-cell variant (claim 7, quirk `quirks.cell_index_alias`):* a 20 × 20 map whose terrain
  stream holds one complete record with coordinates (527, 10) and the tile of a clearly different
  tileset from the rest of the map (for example a pavement tile on a map of clear ground), then a
  (0, 0) record. 1 run. *Expected:* cell (15, 11) shows that tile, and no other cell changes.
  *Falsified if* (15, 11) keeps the fill tile, or the tile appears anywhere else.
  *Height variant:* base level 1. The terrain stream holds complete records, each with a flat
  tile, for a legal cell A at record height level 3 and a legal cell B at record height level −1,
  and ends with a (0, 0) record. A legal cell D has no record, so it keeps the fill's level 1.
  A, B and D are at least 3 cells apart. A pre-placed vehicle owned by the human placeholder name
  (claim 27) stands on each of A, B and D. 1 run. Observable: the on-screen vertical distance of the
  vehicles on A and on B from the vehicle on D, in pixels. Let L(z) be the height lift in pixels
  that S-0101 gives for a world height of z leptons, with z the floor height of the cell's level
  (claim 9, Formulas): 104 × level at level 0 and above, 104 × level + 1 below it. *Expected:*
  the vehicle on A is L(312) − L(104) pixels above the one on D, and the vehicle on B is
  L(104) − L(−103) pixels below it. *Falsified if* the vehicle on A is L(416) − L(104) pixels above
  D (the record's level added to the base level) or at any other distance than expected, or the
  vehicle on B is not below the one on D (for example drawn at level 255, 0 or 1).
- **M4b, untranslated tile numbers (claim 8, identity case).** Any stock theater (none declares a
  different authored count). A terrain stream with records whose tile numbers are the first tile
  of several different tilesets. 1 run. *Expected:* each cell shows the first tile of the
  corresponding tileset, as counted in the theater's own tile order. *Falsified if* any cell shows
  a tile from another position. The translation itself is probed by M4c. *Second part
  (claim 8, low confidence):* one more record whose tile number is at least the theater's tile
  count (count the tiles in the theater data) and below 65535. *Expected:* that cell shows the
  theater's clear tile. *Falsified if* it shows any other tile, damaged graphics, or the game
  ends abnormally.
- **M4c, tile number translation (claim 8), with a named test file.** Named test file: a loose
  copy of the theater file of one stock theater (not theater 5), changed in one way only: one
  tileset T, not the first and not the last, gets an authored tile count A = C − 2, where C is its
  actual tile count (count it in the theater data; choose a T with C ≥ 3). The file is removed after
  the runs. Let F be T's first tile number in the theater's own order, so T's first tile in the
  authoring space is also F (no earlier tileset declares a count), and the translation pair is
  t = F + A, δ = 2 (Formulas). A 20 × 20 map of that theater with a terrain stream holding records
  for three legal cells, at least 3 cells apart: cell P with tile number F + A − 1 (the last tile of T
  in the authoring space), cell Q with tile number F + A, and cell R with tile number F − 1 (the last
  tile of the tileset before T). 1 run with the test file, then 1 run of the same map without it.
  *Expected:* with the test file, P shows tile F + A − 1 of the theater (T's (A − 1)-th tile counting
  from 0), Q shows tile F + A + 2 of the theater (the first tile of the tileset after T), and R shows
  tile F − 1 in both runs. Without the test file, Q shows tile F + A (a tile of T). *Falsified if*,
  with the test file, Q shows tile F + A or any tile other than F + A + 2, P shows any tile other
  than F + A − 1, or R differs between the runs.
- **M5, level height (claim 9).** Flat cells at levels 0 and 4, side by side, with one unit standing
  on each. 1 run. Observable: the on-screen vertical distance between the two units, in pixels.
  *Expected:* the distance equals the screen projection of 416 leptons of world height under
  S-0101. *Falsified if* the distance matches any other whole number of
  leptons per level.
- **M6, overlay (claims 11–13).** A map with ore, a wall line and a crate overlay, saved once with
  the format version set to 4 and once with the key removed. Skirmish, 2 runs. In the version-4
  save, one ore cell's value in the data stream gives a different density from its neighbours.
  *Expected:* with 4, ore and walls appear, the crate does not, and that ore cell shows the data
  stream's density. Without the key, no overlay at all appears. *Falsified if* overlay appears
  without the key, the crate appears, or the ore density ignores the data stream.
  - *Format versions 1 and 2:* two more saves, 1 run each. *Expected:* version 1 behaves like the
    removed key (no overlay), version 2 like version 4. *Falsified if* overlay appears at
    version 1 or is missing at version 2.
  - *Short-stream save:* the version-4 map, with ore cells also on the last rows of stream
    positions (largest y). Its overlay type stream is encoded so that it ends before those
    positions. Its overlay data stream is encoded so that it ends before one ore cell P whose
    density was set to a non-zero value. The wall line, the crate and every other ore cell lie
    before the end of the data stream in stream order, except the cells past the end of the type
    stream and P itself, so P is the last overlay before the end of the type stream. 1 run.
    *Expected:* the cells past the end of the type stream have no ore, every overlay before the
    end of the data stream is as in the full map, and P shows data value 0 (the lowest density).
    *Falsified if* ore appears past the end of the type stream, an overlay before the end of the
    data stream differs from the full map, or P shows its non-zero density.
- **M6b, vein rebuild (claim 14).** Format version 4. Two vein overlays (index 126) on flat clear
  cells whose four edge neighbours are flat, clear and empty, at least four cells apart in x or in
  y: site A with data value 47 and site B with data value 48. Before running, check in the rules
  data that the vein overlay type has artwork (claim 12). 1 run; read the cells on screen after a
  reveal. *Expected:* A and its neighbours hold no overlay. B and its four edge neighbours all hold
  the vein overlay, B showing a pattern centre. *Falsified if* A keeps a vein, B lacks one, or any
  edge neighbour of B lacks one. The draws, their order within a seed and the reverse visiting
  order across seeds are S-0202's scenario R8, part B.
- **M6c, data values that survive creation (claim 12).** Format version 4. Overlay list indices
  count positions in the overlay type list in registration order from 0 (S-0004), not the numbers
  of the list's keys. Place the overlay types at indices 24, 25, 237 and 238 on four separate
  cells, plus a straight wall line as a control. Variant A has no overlay data stream. Variant B
  is identical but has an overlay data stream that gives every cell the value 0. 1 run each.
  *Expected:* each of the four indexed cells looks the same in both variants, because creation
  leaves its data value at the blank cell's 0. The wall line differs between the variants: in A
  the pieces are joined, because placement sets their data values, and in B they show the pieces
  for data value 0. The control shows that the comparison can detect a data value set by creation.
  *Falsified if* any indexed cell differs between A and B. If the wall control shows no
  difference, the run is inconclusive, not a pass. The test cannot detect a type whose creation
  would itself set 0.
- **M7, waypoint encoding and home (claims 15, 16).** A map with waypoint 699 undefined and no home
  setting. 1 run. *Expected:* the initial view, before it moves to the player, is centred on cell
  (⌊(W+H)/2⌋, ⌊(W+H)/2⌋). A variant defines waypoint 699 at a legal cell at least 5 cells from that
  centre, still with no home setting, 1 run. *Expected:* the initial view is centred on waypoint
  699's cell. *Falsified if* it starts at the centre or elsewhere. *Wrap map:* a 20 × 20 map whose
  start waypoint marker 1 holds the cell number 65546015 (quotient 65546, which reduced modulo
  65536 is 10; remainder 15), 1 run. *Expected:* the player at position 1 starts at cell (15, 10).
  *Falsified if* that player starts anywhere else. A second map, 30 × 30 and flat clear, has start waypoint markers 0 and 2
  defined normally and an entry spelled "01" whose value names a third legal cell; it has no other
  start waypoint markers. Open it in the lobby first. *Expected:* the lobby allows exactly 2
  players, because the "01" entry is not found (claim 23). Then 3 runs with 2 players (1 human,
  1 computer) on fixed positions 0 and 1. Start waypoint 1 is undefined, so K = 1, N = 2 and the start
  list is [wp0, random]. *Expected:* position 0 starts at waypoint 0's cell in every run, and
  position 1 at a cell that varies between runs and is neither the "01" cell nor waypoint 2's.
  *Falsified if* the view starts elsewhere, the lobby allows 3 players or only 1, or position 1
  ever starts at the "01" cell or at waypoint 2's cell.
- **M8, start list with a gap (claim 17).** A 40 × 40 flat clear map with start waypoint markers
  0, 1, 3 and 5 defined and no others, so the lobby limit is 4 (claim 23). 4 players (1 human,
  3 computers) with fixed positions 0 … 3. K = 2 and N = 4, so the start list is [wp0, wp1, wp3,
  random]. 3 runs. *Expected:* position 2 starts at waypoint 3's cell in every run. Position 3
  starts at a cell that varies between runs and is never waypoint 5's cell. *Falsified if*
  position 3 starts at waypoint 3 or waypoint 5, or at the same cell in every run, or position 2
  starts anywhere but waypoint 3.
- **M8b, start waypoint on a non-legal cell (claims 1, 15, 17).** A 20 × 20 map with start
  waypoint markers 0, 2 and 3 on legal cells and waypoint 1 on a non-legal cell, for example cell
  (1, 1) (s = 2, not more than W). 3 players (1 human, 2 computers) on fixed positions 0, 2 and 3,
  so that no player is meant to start on the non-legal cell. 3 runs. *Expected:* waypoint 1 counts
  as defined, so K = 4 and the start list is the four waypoint cells. In every run the players at
  positions 2 and 3 start at waypoint 2's and waypoint 3's cells. *Falsified if* position 2 or 3
  starts anywhere else in any run (with waypoint 1 undefined, position 2 would get a random cell).
  If the game does not finish loading, the run is inconclusive.
- **M9, random starts (claims 18, 20).** A 40 × 40 map with no start waypoint markers, partly
  covered by water and cliffs, 2 players (1 human, 1 computer) on fixed positions, 20 runs. Record
  the start cells. *Expected:* every start lies in an 8 × 8 area clear for tracked
  vehicles on the real terrain. The map has no random-map player count. *Expected (lobby):* the
  lobby allows 8 players (claim 23 fallback). A variant of the map with a random-map player count
  of 3, lobby only. *Expected:* the lobby allows 3 players. *Falsified if* either lobby allows
  another number. A second map with no 8 × 8 clear area, 1 run, in the original. *Expected:* loading
  never finishes (observe for a fixed time limit, then stop the game; the run records the hang, not
  a result). *Falsified if* any start lacks the clear area, or the second map loads. Our engine on
  the same map is checked separately: it must refuse to start with the report of claim 18.
  *Candidate ranges (statistical):* a 40 × 40 flat clear map with no start waypoint markers, 2
  players (1 human, 1 computer) on fixed positions, 100 runs, recording both start cells (200
  cells). Compare them with 200 start cells from a reimplementation of claim 18 on the same map,
  over randomly chosen starting states. On this map the search of claim 18 moves only candidates
  near or beyond the edge of the legal area; the reimplementation moves each such candidate to the
  nearest legal cell with a clear 8 × 8 area. *Expected:* the counts in each of the four bands
  x + y ≤ 60, 61 … 80, 81 … 100 and above 100, and in each of the bands y − x below −10,
  −10 … 9 and 10 or more, agree with the reimplementation's. The asymmetric y range (claim 18) puts more
  starts at large y than at large x. *Falsified if* a chi-square test at the 1 % level rejects the
  agreement for either set of bands.
- **M9b, assignment before the fill (claim 20), statistical.** A 40 × 40 flat clear map with start
  waypoint markers 0, 5 and 6 and no others, so the lobby allows 3 players (claim 23) while K = 1,
  N = 3 and the start list is [wp0, random, random] (claim 17). Start waypoint marker 0 lies near one corner of
  the legal area. 3 players: the human on fixed position 0 and 2 computers on "random". 40 runs
  with the clear fill and 40 with the water fill (a terrain stream covers every cell, so the fill
  is invisible). In each run record which computer starts farther from waypoint 0's cell. The
  first computer is assigned the farther of positions 1 and 2 by list 2's cells, and both
  computers start on list 3's cells, which are drawn independently of list 2's. *Expected:* in
  each set of 40 runs the first computer starts farther from waypoint 0 in roughly half the runs
  (between 10 and 30 of 40). *Falsified if* in either set it starts farther in 36 or more runs (the
  assignment then used the final cells), or in 4 or fewer. The exact draw counts of lists 1 and 2
  are not observable this way (claim 20).
- **M9c, draw counts and order at load (claims 6, 18, 19 and 20), with the random-state recorder.**
  A 40 × 40 map, format version 4, local area the whole map, with the fill set to water. A terrain
  stream gives a flat clear tile to every legal cell except those of the first four screen rows
  (s = 41 … 44), which keep the fill: 158 cells, the first 158 in the visiting order of claim 3,
  each showing r = 0 … 3 above the water base tile. Every run is the first game of a fresh program
  run, so the water base tile is 0 (claim 6) and each of those cells shows tile r of the map's
  theater. The map has no overlay, no objects, no cell tags and
  no animated tiles, so no map-content draw lies between the fill and list 3. A trigger reveals the
  map (default settings); nothing carries its tag, so the tag is created at the end of loading and
  its draws come after list 3 (S-0011, claim 34). 2 players, 1 human and 1 computer, both on "random". With the recorder,
  read the seed at the first tick, compute the scenario random stream from it (S-0202, claims 1 to
  3), and let k be the smallest number of raw values before the fill's for which the fill draws of
  the 158 cells, by S-0202's range rule over 0 … 3, equal the shown values r in visiting order.
  - Variant A: start waypoint markers 0 and 1 at legal clear cells, at least 20 cells apart and
    10 cells from water. N = K = 2, so lists 1 and 2 have no random entries, the human takes the
    random-position draw over 0 … 1 and the computer is placed by the farthest-sum rule. 5 runs.
    *Expected:* k = 1 in every run (S-0202's table "Draws during map loading" lists nothing before
    the early phase), and the human starts at waypoint 0 exactly when the raw value just before
    the fill's gives 0 by the range rule. (A k that is the same in every run but larger than 1
    falsifies only the absence of earlier draws; variant B then uses that k in place of 1.)
  - Variant B: the same map with no start waypoint markers, so lists 1 and 2 each have 2 random
    entries (claim 18) and N = 2. 10 runs. *Expected:* k equals 1 plus the raw values that the 4
    random entries take by claim 18 (two ranged draws each, by S-0202's range rule; on the blank
    cells of the early phase the first candidate pair of each entry is kept), all computed from the
    recorded seed, and the human starts at the position that the raw value just before the fill's
    gives, not the one that the first raw value after the seeding would give. Further, list 3's entries are drawn from the raw values that directly follow
    the fill's 3160 draws: in every run where a computed list 3 candidate cell is a legal clear
    cell at least 10 cells from water and from the map's edge, the player holding that position
    has its starting units within 8 cells of it.
  *Falsified if* k differs from its expected value in any run, the human's position in A disagrees
  with the draw, or in B a player's starting units lie more than
  8 cells from a list 3 candidate that meets the condition. *Inconclusive if,* in B, k exceeds its
  expected value by the raw values of whole further candidate pairs: the early-phase search then
  rejected a candidate, which is a question for the placement rules, not for the draw order. Also
  *inconclusive if* the computed stream
  fails to match the fill at every k (S-0202's recurrence is then at fault, which its scenario R1
  checks).
- **M10, farthest-sum (claim 19).** 4 start waypoint markers at known cells, 3 players (1 human,
  2 computers) all on "random", 50 runs. House order is the human, then the computers in lobby-slot
  order. *Expected:* the human's position is roughly uniform over the 4 (each between 5 and 20 of
  the 50 runs). The first computer is always at the position farthest from the human. The second
  computer has the largest distance sum, with ties going to the lower position. Colour-order
  sub-test: a two-human network game (2 humans, 1 computer, all on "random"), 10 runs, then the same
  with the two humans' colour indices swapped, 10 runs. *Expected:* the human with the lower colour
  index takes the uniformly drawn position in every run, so the roles of the two humans swap with
  the colours. *Falsified if* any run breaks the farthest-sum rule, a position is never or almost
  always drawn first, or swapping colours does not swap the roles.
- **M10b, tabulated square root (claim 19, quirk `quirks.table_root_distance` of S-0204).** A 140 × 140
  flat clear map with start waypoint markers 0 at (80, 150), 1 at (171, 241) and 2 at (209, 150),
  all legal. 2 players: the human on fixed position 0 and 1 computer on "random", so N = 3 and no
  random start cell is needed. Squared distances from waypoint 0: 2 × 91² = 16562 to waypoint 1
  (exact distance about 128.7) and 129² = 16641 to waypoint 2 (exactly 129). Both terms are 128
  (Formulas), so the tie goes to position 1. 3 runs. *Expected:* the computer starts at waypoint
  1's cell in every run. *Falsified if* it starts at waypoint 2's cell (exact distances). Control:
  waypoint 2 moved to (210, 150), squared distance 130² = 16900, term 130, 1 run. *Expected:* the
  computer starts at waypoint 2's cell. *Inconclusive if* the control gives waypoint 1.
- **M11, player at position (claim 21).** A trigger that reveals the map to "player at position B",
  2 players with fixed positions 0 and 1, then the same with the positions exchanged. 2 runs.
  *Expected:* the player at position 1 gets the reveal both times. *Falsified if* the reveal follows
  the player rather than the position.
- **M12, lobby limit (claims 15, 23).** A 40 × 40 flat clear map with start waypoint markers 0, 1,
  2 and 5, with waypoint 2 stored as 0. 1 run of the lobby plus 1 game with 4 players (1 human,
  3 computers) on fixed positions 0 … 3. *Expected:* the lobby allows 4 players, and in game
  position 2 starts at a random cell. *Falsified if* the lobby allows a different number, or
  position 2 starts at cell (0, 0) or at waypoint 5. *Map-list label:* two loose map files, one
  with minimum and maximum player settings both 3, one with 2 and 5, lobby only. *Expected:* the
  map list labels the first with a single number and the second with a range. *Falsified if*
  either label has the other form.
- **M13, preview (claims 24, 25).** A map whose header block lists 2 markers at positions different
  from its 2 start waypoint markers. 1 run. *Expected:* the preview markers are drawn at the header
  positions and players start at the waypoint cells. The same map with the preview stream shortened
  so that the last row is missing, 1 run. *Expected:* the preview is still shown, with all rows but
  the last as encoded. The same map without a header block, 1 run. *Expected:* no markers. The
  same map with its preview size entry removed (the preview stream kept), 1 run. *Expected:* no
  preview image is shown in the map selection screen (claim 30). *Falsified if* the markers follow
  the waypoint cells, the shortened preview disappears entirely or is shown complete, markers
  appear without a header block, or a preview image appears without a preview size entry.
  *Extent-0 variant:* the first map with N_h = 2 and the marker extent Wh set to 0 (Import mapping), 1 run.
  Select the map in the map selection screen. *Expected:* the original crashes when the preview
  markers would be drawn. *Falsified if* the screen shows the preview with or without markers
  and the program keeps running.
- **M13b, header pairs as start position marks (claim 29).** A map with 4 start waypoint markers
  whose pairwise distances are all different, a header block with N_h = 9, and 2 players: 1 human
  (house 0) and 1 computer (house 1). No preview markers are drawn, because N_h > 8.
  - Variant A: both players on "random", pair 9 = (1, 0). 5 runs. *Expected:* in every run the
    computer starts at waypoint 0's cell and the human at waypoint 1's cell.
  - Variant B: the human chooses position 0 and the computer position 1, pair 9 = (−1, −1).
    10 runs. *Expected:* the human's start varies over the 4 waypoint cells (at least 2 different
    ones in 10 runs), and the computer always starts at the waypoint cell farthest from the human's.
  - Control: variant B with N_h = 8. 2 runs. *Expected:* the human starts at waypoint 0 and the
    computer at waypoint 1.
  - Variant C, pairs above N_h are not read: N_h = 9, pair 9 absent, pair 10 = (1, 0) present,
    both players on "random". 10 runs. *Expected:* pair 10 has no effect. The human's start varies
    over the 4 waypoint cells (at least 2 different ones in 10 runs), and the computer always
    starts at the waypoint cell farthest from the human's.
  - Variant D: variant C with N_h = 10. 5 runs. *Expected:* in every run the computer starts at
    waypoint 2's cell and the human at waypoint 3's cell.
  - Variant E, references and the highest-numbered position held: N_h = 9, pair 9 = (0, 0),
    both players on "random", and a trigger that reveals the whole map to "player at position A"
    once a short time has elapsed. Before running, check that the distance sums (Formulas) of
    waypoint markers 2 and 3 to waypoint markers 0 and 1 differ; call W* the cell of the one with the
    larger sum. 3 runs.
    *Expected:* in every run the human (house 0, which holds positions 0 and 1 through their marks)
    starts at waypoint 1's cell, the computer starts at W*, and the human's map is revealed after
    the delay, because position A's mark is the human's house number although the human does not
    start there.
  - Variant F, a mark that names no house: N_h = 9, pair 9 = (5, 5) (no house has number 5: the
    houses are 0 and 1 for the players, 2 and 3 for the neutral and special houses), both players
    on "random", and the same reveal trigger as variant E. Under the same pre-check, 3 runs.
    *Expected:* in every run the human starts at W* and the computer at the other of the
    cells of waypoint markers 2 and 3; no player starts at waypoint 0's or waypoint 1's cell, and the map is never revealed,
    because the reference is empty.

  *Falsified if* variant A ever differs from its expectation, variant B always starts the players
  at waypoint markers 0 and 1 or puts the computer elsewhere than farthest from the human, the
  control moves either player, variant C starts the computer at waypoint 2 and the human at
  waypoint 3 in every run (pair 10 read although N_h = 9), the human's start in variant C never
  varies or the computer there is ever elsewhere than farthest from the human, variant D ever
  differs from its expectation, variant E gives no reveal (the reference then follows where a
  player starts, not the mark) or starts the human at waypoint 0's cell (the lowest-numbered
  position held instead of the highest), or variant F ever starts a player at waypoint 0 or 1, ever
  gives a reveal, or starts the players differently from its expectation. *Inconclusive if* the
  reveal trigger fails in a control map where "player at position A" is the human's own fixed
  position 0 (N_h = 8, 1 run): the trigger is then at fault.
- **M14, skirmish read set (claims 26, 27).** A map with start waypoint markers 0, 1 and 2 and
  3 players on fixed positions 0, 1 and 2: 1 human and 2 computers with different colours. The map
  has a per-house section giving the player's country extra credits, and pre-placed units owned by
  the neutral house, by a country name, by player at position A, and by the computer placeholder
  name, plus one pre-placed structure and one pre-placed vehicle owned by the human placeholder
  name (owner names as in Import mapping). 1 run. *Expected:* the player starts with the lobby
  credits, the neutral house's unit appears, the country-owned and player-at-position units do
  not, the computer-owned unit appears in the colour of the computer player that comes first
  in lobby-slot order, and the human-owned vehicle appears in the player's colour while the
  human-owned structure does not. *Falsified if* the credits change, the country-owned or
  player-at-position unit appears, the computer-owned unit belongs to the other computer player
  or is missing, the human-owned vehicle is missing, or the human-owned structure appears.
- **M15, observers (claim 22).** 2 computer players on "random" plus one human observer, on a map
  with 4 start waypoint markers, 10 runs. Record the position the observer is first shown and each
  player's start. *Expected:* the first computer player in house order always starts at the
  position the observer was first shown. *Falsified if* it ever starts elsewhere.
  Second branch: a network game with 1 human player on fixed position 2, 1 human observer and
  1 computer player on "random", on a map with 4 start waypoint markers whose pairwise distances
  are all different, 5 runs. *Expected:* the observer is first shown position 2 (the only taken
  position) in every run, and the computer always starts at the waypoint farthest from waypoint 2,
  as if no observer were present. *Falsified if* the observer is first shown another position, or
  the computer starts anywhere else in any run.
- **M16, unused-house cleanup (claim 28).** Before running, check that the rules data's maximum
  player count is above 2, and that the rules data marks both the neutral country and the special
  country passive. If any check fails, the variants below do not separate the cases and the
  scenario has to be redesigned.
  Map: 30 × 30, local area the whole map, format version 4. Start waypoint markers 0 … 3 at cells
  (20, 20), (40, 40), (40, 20) and (25, 35). Their distance sums (Formulas) to waypoint markers 0 and 1
  are 20 + 20 = 40 for waypoint 2 and 15 + 15 = 30 for waypoint 3. Owned by the neutral house (by
  its country name, Import mapping): a pre-placed structure at cell (35, 30) and a pre-placed
  vehicle at cell (36, 30), plus a trigger owned by the neutral country that reveals the whole map
  to every player once a short time has elapsed. That trigger's tag is attached through a cell tag
  to a legal cell that no unit enters, for example a cliff cell, so that the tag exists when the
  cleanup runs (claim 28; a tag that nothing carries would be created after the cleanup and keep
  its trigger, variant E). The map has an override section for the neutral country that sets its
  passive field (Import mapping) to false. The special country is not overridden, except in
  variant D. Players on fixed positions 0 and 1. Each variant runs once, in a fresh game launch,
  so that no country data from an earlier game is in force.
  - Variant A: a network game with 2 humans and no computer player. *Expected:* once the map has
    loaded, neither the structure at (35, 30) nor the vehicle at (36, 30) is there, the map is never
    revealed, the humans start at waypoint 0's and waypoint 1's cells, and the neutral house
    receives no starting units anywhere (the cleanup removed it).
  - Variant B, condition control: variant A's map with 2 humans and 1 computer player on fixed
    position 2. *Expected:* the structure and the vehicle are present, the map is revealed after
    the delay, the players start at waypoint markers 0, 1 and 2, and starting units of the neutral house
    appear around waypoint 3's cell, the only free position.
  - Variant C, passivity control: variant A's game without the override section. *Expected:* the
    structure and the vehicle are present, the map is revealed after the delay, and the neutral
    house gets no starting units.
  - Variant D, both houses non-passive (`quirks.cleanup_skips_next_house`): variant A's game, with
    a second override section, for the special country, that sets its passive field to false too.
    The map also gives the special house (by its country name) a pre-placed structure at cell
    (30, 50) and a pre-placed vehicle at cell (31, 50), and a second trigger, owned by the special
    country, that reveals the whole map to every player once a short time has elapsed, like the
    neutral one. Its tag is attached through a cell tag to another legal cell that no unit enters.
    Variant A shows that the neutral trigger is removed, so a reveal here comes from the special
    trigger. *Expected:* the neutral house's structure and vehicle are gone, the special house's
    structure and vehicle are there, the map is revealed after the delay, and starting units of the special house appear around waypoint 2's cell (it is the only house
    that needs a position in the late phase). *Falsified if* the special house's objects are missing
    or the map stays hidden (the cleanup removed the special house too: the fixed behaviour), or the
    neutral house's objects are present.
  - Variant E, a trigger created at the end of loading: variant A's game, with the cell-tag entry
    removed, so that nothing carries the neutral trigger's tag and the tag is created at the end of
    loading. *Expected:* the structure and the vehicle are gone as in A, but the map is revealed
    after the delay: the trigger survives the cleanup although its owner house is gone. *Falsified
    if* the map stays hidden while it is revealed in C. *Inconclusive if* S-0011 makes the reveal
    action depend on the trigger's owner house; the variant then needs an action that does not.

  *Falsified if* the structure or the vehicle is present in variant A, either is missing in
  variant B while both are present in C, a player in A or B starts elsewhere than at the waypoint
  of its chosen position, the map is revealed in A although it is revealed in C, the neutral
  house receives starting units in A, or in B around any waypoint but 3. Starting units of the
  neutral house that are missing in B falsify the starting-units sentence of claim 28 only.
  *Inconclusive if* the pre-placed objects are missing in C as well (the removal then has another
  cause), or the map is not revealed in C (the trigger is at fault, so the trigger sub-tests of A
  and E say nothing). In A, B and C the special country stays passive (pre-check), so in B the
  neutral house is the only house besides the players that needs a position, and it finds one free
  position; in D the special house is the only one, and it finds two. No house needs a position
  after it, so the unspecified all-taken case of claim 29 cannot arise.
- **M16b, country data seen by the early phase (claim 33).** M16's map and start waypoint
  markers, without the neutral country's override section. 3 players: 1 human of country C0 on
  fixed position 0 (waypoint 0), and 2 computers on "random": the first in lobby-slot order of
  country C1, the second of country C2, with C0, C1 and C2 three different countries. By the
  farthest-sum rule, the first computer to be assigned takes waypoint 1 (distance 28 from
  waypoint 0, against 20 for waypoint 2 and 15 for waypoint 3), and the second then takes
  waypoint 2 (sum 40, against 30 for waypoint 3). There is a computer player, so the cleanup of
  claim 28 does not run. Before running, check that the base rules data do not mark C1 passive.
  - Variant A, map override: the map has an override section for C1 that sets its passive field
    (Import mapping) to true. Fresh program start, 1 run. *Expected:* the second computer's
    starting units appear around waypoint 2's cell, and the first computer receives no starting
    units.
  - Variant B, carry-over: run only if the base rules data give no passive field in C1's section
    (otherwise this variant cannot separate the cases and is skipped). Without restarting the
    program after variant A, play the map again without the override section, with the same
    players and countries. 1 run. *Expected:* the first computer starts at waypoint 2's cell and
    the second at waypoint 1's cell.
  - Control: variant B's map and players as the first game after a fresh program start. 1 run.
    *Expected:* the first computer starts at waypoint 1's cell and the second at waypoint 2's.
  - Variant C, game-mode file, with a named test file: run only if the stock game-mode file of
    the mode played gives no passive field for C1. Named test file: a loose copy of that game-mode
    file with one change, a section for C1 that sets its passive field to true; it is removed after
    the run. M16b's map without any override section, the same players and countries, fresh
    program start, 1 run. *Expected:* as in variant A: the second computer's starting units appear
    around waypoint 2's cell, and the first computer receives no starting units.

  *Falsified if* in variant A the second computer starts at waypoint 1's cell (the early phase saw
  the override) or the first computer receives starting units, or if variant B gives the
  control's positions, or if variant C starts the second computer at waypoint 1's cell (the early
  phase saw the game-mode file) or gives the first computer starting units. *Inconclusive if* the control differs from its expectation (the
  farthest-sum setup is then at fault).
- **M17, cell tags (claim 31).** A 20 × 20 map, format version 4, with three tags T1, T2 and T3.
  Each tag's trigger fires when an object enters its cell and has its own visible effect, for
  example revealing a different corner of the map. The cell-tag section lists, in this order:
  key 10015 with value T1, key 010015 with value T2 (both decode to cell (15, 10)), key 12012x
  with value T3 (cell (12, 12)), and a line with key 14014 and an empty value (cell (14, 14)),
  which creates no entry (S-0001 claim 11). 1 run: move one unit onto each of the three cells in
  turn. *Expected:* (15, 10) sets off T1 only, (12, 12) sets off T3, and (14, 14) sets off
  nothing. *Falsified if* T2 ever fires, T3 does not fire, or anything fires at (14, 14).
  *Variant B (claims 15 and 31 below format version 4):* the same map at format version 3 with a
  cell-tag section of two entries: key 1295 with value T1 (128-based: cell (15, 10); 1000-based:
  cell (295, 1), not legal) and key 12012 with value T3 (1000-based: cell (12, 12); 128-based: cell
  (108, 93), not legal). Start waypoint markers 0 and 1 hold the cell numbers 15008 and 8015,
  which name cells (8, 15) and (15, 8) when read 1000-based but other cells when read 128-based.
  1 run: move one unit onto (15, 10) and then onto (12, 12). *Expected:* the players start at
  (8, 15) and (15, 8), (15, 10) sets off T1, and (12, 12) sets off nothing. *Falsified if* a
  player starts elsewhere, T3 fires, or T1 does not fire.
  *Variant C (claim 31, identical repeated key):* the first map, with a cell-tag section holding
  the key 10015 twice, first with value T1 and then with value T2, plus filler entries with
  distinct keys, each with the value none, so that they attach nothing (claim 31). Before running,
  choose the fillers so that, by the hash-order rule in S-0002's Formulas, the *second* occurrence
  of 10015 wins. 1 run: move one unit onto (15, 10). *Expected:* T2 fires and T1 does not.
  *Falsified if* T1 fires, or both fire. A control map with the same fillers but the two values of
  10015 in the opposite order, 1 run, *expected:* T1 fires and T2 does not.
  *Variant D (claim 31, undefined tag name):* the first map, with a cell-tag section of two
  entries: key 10015 with a value that names no tag the map defines, then key 010015 with value
  T2. 1 run: move one unit onto (15, 10). *Expected:* nothing fires, because the empty tag of the
  first entry occupies the cell. *Falsified if* T2 fires. A control map whose first entry has the
  value none instead, 1 run, *expected:* T2 fires.
  *Variant E (claim 31, wrap):* the first map, with a cell-tag section of one entry: key 65546015
  with value T1 (y = 65546 wraps to 10, so the cell is (15, 10); Formulas). 1 run: move one unit
  onto (15, 10). *Expected:* T1 fires. *Falsified if* it does not.
  *Variant F (claim 31, selected cell; quirk `quirks.cell_index_alias`):* the first map, with a
  cell-tag section of one entry: key 10527 with value T1 (decoded (527, 10), selected cell
  (15, 11), Formulas). 1 run: move one unit onto (15, 11). *Expected:* T1 fires. *Falsified if* it does not.
- **M18, map trigger variables (claim 32).** Every variable number used is in 0 … 99. Three
  triggers each reveal a different corner of the map when their variable is set: on variable 0,
  variable 1 and variable 5. Keys "10x1", "10x2" and so on are distinct keys that all read
  leniently as 10, so they fill entries without touching the probed variables.
  - Variant A, 101 entries in this order: key "0" with value "A,1"; key "1" with value "B,0"; key
    "5" with value "X,0"; 97 filler entries with keys "10x1" … "10x97", each with value "C,0"; and
    last, as the 101st entry, key "5x" with value "X,1". *Expected:* only variable 0's corner is
    revealed at the start.
  - Variant B, the control: variant A without its last filler, so the key "5x" entry is the 100th.
    *Expected:* variable 0's and variable 5's corners are revealed, variable 1's is not.

  - Variant C, no second item: variant B with the value of key "1" changed to "B" (no second
    item), played as the first game after a fresh program start. *Expected:* as in variant B:
    variable 1's corner is not revealed, because its state stays clear.
  - Variant D, empty lines: variant B with a line holding key "7" and an empty value inserted just
    before the key "5x" entry. That line creates no entry, so "5x" is still the 100th entry.
    *Expected:* as in variant B.

  1 run each, each in a fresh program start. *Falsified if* variant A reveals variable 1's or
  variable 5's corner or not variable 0's, variant B or D does not reveal variable 5's corner or
  reveals variable 1's, or variant C reveals variable 1's corner.

## Open questions

- Outside this spec, with pointers only: lighting (S-0104); tile categories and what tiles mean
  for play (S-0010); triggers and tags (S-0011); and the rules sections a map may override
  (S-0006). The byte encodings are a question for the project lead (below).
- The game mode that skips the unused-house cleanup of claim 28 is not identified.
- What a house that the cleanup removes (claim 28) leaves behind in the start position marks of
  claim 29 is not established. Two cases: whether a house that took a position in the early phase
  leaves its house number as that position's mark, and whether the houses after a removed one
  keep their house numbers for the late phase's mark test. Both arise only when the early phase
  already saw the neutral or the special country as non-passive (claim 33) or on maps with header
  pairs beyond the eighth.
- Which playable area is in force before a map's local area is first read (claim 30, early-phase
  read of a partial local area), in the first game of a program run and in later ones.
- Whether anything clears the map trigger variables' states at a game start after the first one
  of a program run (claim 32). A probe: in one program run, play a map that sets a variable, then
  a map whose entry for that variable has no second item, and see whether a trigger on it fires.
- A few features use the plain test of claim 4 in some situations and the height-aware test in
  others. Among them are the minimap's test of whether a whole rectangle lies inside the playable
  area, part of hover-vehicle movement, the computer players' choice of a target for a superweapon,
  and the growth of veins around a veinhole; further uses of this kind may exist and are not
  identified. Each topic spec should state which variant it uses, and in which situations. The same holds for the claim 5 features that
  M3 and M3b do not probe: map-edge arrivals, the cells the two superweapons of claim 5 affect, aircraft firing positions, cliff
  destruction, pathfinding zones and the generated preview.
- Whether anything in play can raise a tag that a cell-tag entry with a non-legal cell attached to
  the stand-in cell, and whether that stand-in is free again at the start of a later
  game of the same program run (Edge cases, cell-tag key whose selected cell is not legal).
- The legacy terrain streams (claim 10) and the legacy object-list encodings below format version 4
  are not specified. They matter only for very old maps.
- The game mode that places players its own way in the early phase instead of building list 2 (one
  stock mode uses its own placement) is not identified. It needs a per-mode check.
- Maps wider or taller than 63 cells might behave differently from smaller maps in some way
  not covered here. No such difference is known. Comparing a 63 × 63 map with a 64 × 64 map that
  is otherwise the same could look for one.
- The observer rules (claim 22) are low confidence.
- Whether the map list filters maps on their minimum and maximum player settings (claim 23).
- The meaning of the initial-time and percent settings, and the effect of the fill-silos and
  multiplayer-only switches, are not established here.
- The full list of draws in the map-content row of the table "Start position resolution" belongs
  to S-0202 (its table "Draws during map loading"); what each pre-placed object's placement and
  each starting unit's placement draw belongs to the placement and starting-unit rules, which have
  no spec yet. Whether further loading steps draw is not established. The random-state recorder
  (scenario M9c) can count the draws of a whole load once the placement and starting-unit rules
  state theirs; until then an unexplained surplus cannot be assigned to a step.
- Each of the first 50 variable-name entries (claim 32) seems also to give its whole value text
  (name and state together), as a name only, to the variable of the entry's number in a second set
  of named variables, possibly the game's global trigger variables. Only names would change, so
  nothing in play shows it. It is a pointer for a later revision of S-0011 (whose subset does not
  cover global trigger variables), which should establish what that second set is.

## Questions for the project lead

- The byte encodings of the terrain, overlay and preview blocks (value widths, compression, block
  layout) are outside this spec, which fixes only the order and meaning of the decoded values
  (Terms, decoded stream). They need a formats spec of their own, which is not yet planned: should
  one be added to the plan?
