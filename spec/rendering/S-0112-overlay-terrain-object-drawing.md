---
id: S-0112
title: Overlay and terrain object drawing
topic: rendering
revision: 12
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: [Q-0012, Q-0021]
supersedes: []
---

# S-0112: Overlay and terrain object drawing

## Summary

Which palette, shade table and brightness rule each overlay, smudge and terrain object is drawn
with, and which sprite and image frame each overlay's body and shadow show (including wall
connection bits and tiberium image runs). Placement, depth and shadow darkening are S-0102's, and
building shade rows is S-0103's and S-0104's. This revision covers skirmish; network games and
campaigns are out of scope.

## Terms

Glossary terms used here include *overlay*, *overlay data value*, *terrain object*, *animated
terrain object*, *palette*, *house colour*, *local player*, *image frame*, *sprite*,
*slope*, *theater*, *tiberium* (the resource type kind), *type list*, *departure*, *importer warning*
and *quirk switch*.

- **Smudge** and its *pieces* are S-0102's terms (Terms there). The *overlay shadow level*
  *ℓ*<sub>s</sub> is S-0102's too: the height level of the cell that holds an overlay, without the
  4 levels a bridge deck adds.
- **Lighting terms of S-0104.** *Shade table*, *shade row*, *brightness*, *scaled ground level*,
  *unscaled ground level U*, *scaled raised level*, *map tint*, *global effect*, *tile shade table*
  of a cell, the *untinted table* (claim 6), the *light-alpha value* and the *light-glow image*
  (Terms there) and *neutral test lighting* are used only as S-0104 defines them. *Deck cell*, *wall overlay* and *ore-spawning terrain object* are S-0104's terms
  (Terms there) and keep that meaning. *Plain terrain object* is a terrain object that is not
  ore-spawning.
- **Depth terms of S-0102.** *Depth bias*, *depth units*, the *flat profile*, *upright* types and
  bodies, *writing depth*, the *repainted area* and its *ranks* are used as S-0102 claims 13 and
  16 use them (claim 8, Constants, OV16).
- **Palette terms of S-0103.** *Unit palette*, *tile palette*, *theater palette*, *anim palette*,
  the theater names and file names of S-0103 claim 2 and Constants, and the *53-shade house-colour
  palette* of a house (S-0103 claim 5) are S-0103's.
- **Overlay list position.** The position of an overlay type in the overlay type list, counted from
  0 in registration order (S-0004), not the number of its key. S-0102 claim 24 and S-0009 claim 12
  count the same way.
- **Overlay flags.** The *tiberium*, *wall*, *crate* and *rubble* flags of an overlay type (Import
  mapping; S-0211 uses the same flags). A *tiberium overlay* is an overlay whose type has the
  tiberium flag (ore and gems). The *vein overlay* is the overlay type at list position 126 (S-0009
  Terms).
- **A veinhole.** The object from which veins grow. It is not an overlay, and its own drawing is not
  covered here.
- **A veinhole marker overlay.** One of the overlay types at list positions 167 and 178. Placing a veinhole
  puts the first in the veinhole's own cell and the second in its eight neighbours, each with
  overlay data value 0 (claim 1 says what this does to a deck cell).
- **Image number.** The integer *v* that a rules layer gives a tiberium to choose its image run
  (claim 10, Import mapping and Constants). It is not an overlay list position: the Constants table
  turns it into one.
- **Rubble cell.** The top-left foundation cell of a destroyed building that left rubble. It
  remembers the building's type (S-0312 claim 20). The building's other foundation cells hold the
  same rubble overlay but are not rubble cells.
- **Image run of a tiberium.** Each tiberium owns a run of consecutive overlay list positions: a
  *flat run* of *F* positions starting at a first position *P*, followed directly by a *sloped run*
  of *S* positions (from *P* + *F*). *P*, *F* and *S* follow from the image numbers the tiberium
  has been given (claim 10, Import mapping and Constants).
- **The tiberium of an overlay.** For a tiberium overlay: the first tiberium, in the order of the
  tiberium list (S-0004), whose flat or sloped run contains the overlay's list position. If none
  does, the first tiberium of the list (claim 4).
- **Frame count *N*.** The number of image frames in a sprite. Body image frames are 0 to
  ⌊*N*/2⌋ − 1 by convention and shadow image frames follow them, but the rules below are stated for
  any image frame index.
- **Start-up resource table.** A shade table of 53 rows built by S-0104's untinted rule (claim 6,
  Formulas "Untinted table") from the **temperate theater palette** (×4 by
  S-0103 claim 1), read once when the program starts.
- **Start-up mine table.** A shade table of 53 rows built by the same untinted rule from the **snow
  unit palette** (×4 by S-0103 claim 1), read once when the program starts. It
  is built from the file's own colours, multiplied by 4 once, so it is never built from the palette
  multiplied by 4 twice that S-0103 describes for a missing unit palette file (S-0103 Edge cases).
  What it is built from when the snow unit palette file itself is missing at program start is an
  open question (Open questions).
- **Connection bits.** For a wall overlay with data value *d*, the four binary digits of
  *d* mod 16, digit *k* (worth 2^*k*, *k* = 0 … 3) standing for one edge neighbour (claim 7).
  ⌊*d*/16⌋ is the wall's damage stage, which the wall-damage rules own (S-0303 claim 16 and
  Formulas F4, S-0210 claim 8).
- **Wall-damage terms.** *Wall hit*, its unit *wall-hit points*, a wall type's *damage-stage
  count* and its *wall strength* are used as the wall-damage rules give them (S-0303 claim 16 and
  Formulas F4; S-0210 claim 8 for crushing).
- **Edge neighbour, diagonal neighbour.** The edge neighbours of cell (*x*, *y*) are the four
  cells in the straight step directions 0, 2, 4 and 6 (glossary *step direction*): map north
  (*x*, *y* − 1), map east (*x* + 1, *y*), map south (*x*, *y* + 1) and map west (*x* − 1, *y*).
  Its diagonal neighbours are the four cells in step directions 1, 3, 5 and 7.
- **The map's overlay list, overlay data stream.** The overlay types and the overlay data values
  a map gives its cells (S-0009 claims 12 and 13; Import mapping). The overlay list is placed in
  stream order: increasing *y* and, within one *y*, increasing *x* (S-0009 claim 12).
- **Within two edge steps.** The cells (*x*′, *y*′) with |*x*′ − *x*| + |*y*′ − *y*| ≤ 2 around
  cell (*x*, *y*): the cell itself, its four edge neighbours, its four diagonal neighbours and the
  four cells two steps away in a straight line.
- **Gate and wall tower types.** The building types the rules name as the first and second Allied
  gate types, the first and second Soviet gate types and the wall tower type (Import mapping).
  A *first* gate type joins walls map east and map west of it, a *second* gate type walls map north
  and map south of it (claim 7). The *top-left foundation cell* of a building is the foundation cell
  with the smallest *x* and the smallest *y*.
- **Wall building.** A building type with the building wall flag that names a wall overlay type
  (Import mapping). Placing one, from the map's building list or by building it in play, puts a wall
  overlay of that type in its cell, initially owned by the placing house, instead of a standing
  building. Map entries must survive S-0009 claim 27; ownership before play follows S-0210
  claim 14 (a) (claim 7).

## Behaviour

### Palettes, shade tables and brightness

1. **Overlay drawing table.** An overlay's cell decides its drawing by the first row of this table
   that it meets. The brightness column names the value of S-0104 claim 13 and Formulas; the row
   that brightness selects follows S-0104 claim 15.

   | Precedence | Cell | Palette and shade table | Brightness | Sprite of the body | Body image frame |
   |---|---|---|---|---|---|
   | 1 | holds a veinhole marker overlay | — | — | not drawn | — |
   | 2 | a deck cell, whatever its overlay type | the cell's tile shade table | the cell's scaled raised level | the cell's overlay type | claim 6 |
   | 3 | tiberium overlay | the start-up resource table | 1000 ‰ | chosen by claim 4 | the overlay data value |
   | 4 | wall overlay | the local player's 53-shade house-colour palette | the cell's unscaled ground level *U* | the cell's overlay type | the overlay data value |
   | 5 | the vein overlay | the start-up resource table | the cell's scaled ground level | the cell's overlay type | the overlay data value |
   | 6 | overlay type with the crate flag | the cell's tile shade table | the cell's scaled ground level | the cell's overlay type | 0 |
   | 7 | overlay type with the rubble flag | the cell's tile shade table | the cell's scaled ground level | claim 5 | claim 5 |
   | 8 | any other overlay | the cell's tile shade table | the cell's scaled ground level | the cell's overlay type | the overlay data value |

   Consequences:
   - **Outside deck and marker cells, ore, gems and veins never take colour.** The start-up resource table is untinted (S-0104
     claim 6). Map tint, coloured light sources and global effects never change their colours.
     Ore and gems ignore the cell's light level: their brightness is always 1000, so they use
     row 26 when the light-alpha value is at its default (S-0104 claim 15), and row 26 reproduces
     the palette exactly, without the −1 of S-0104 claim 7. Shroud cover pieces and light-glow
     images can change their shade row through the per-pixel light-alpha value, despite their
     fixed brightness. S-0110 claims 5 and 6 own those updates: cover pieces act before glows,
     and a glow cannot brighten a pixel whose covered value is 0. Conditional fog updates are
     deferred outside skirmish, where fog is always off. Veins follow the cell's brightness and
     the same per-pixel light-alpha updates.
   - **Walls take neither the tile tints nor their owner's colour.** Walls show the local player's
     house colour, whoever built them and whoever the local player is fighting (`quirks.wall_local_player_colour`).
     A trigger action that changes the map tint, or a global effect, reaches them as it reaches every
     53-shade house-colour palette (S-0104 claims 3 and 17).
   - **Everything drawn with the cell's tile shade table takes the cell's tint**, as terrain does
     (S-0104 claim 3): bridge decks, crates, rubble, and every overlay of row 8, for example a
     fence, track, rock or decoration whose type has none of the four flags. S-0103 claim 8
     and S-0104 claims 3 and 13 delegate overlay palette, tint and brightness precedence to this
     table. The neutral-brightness resource rule applies only outside deck and marker cells.
     Drawn deck overlays instead use the cell's tile shade table and scaled raised level.
   - Rows 3 and 5 depend on the type's flags and list position only, never on the theater.
   - **Deck cells.** The original makes a cell a deck cell only when it places in that cell an
     overlay type at one of the bridge list positions 24, 25, 237 and 238 (the positions of S-0009
     claim 12). A deck cell normally holds that bridge type. A veinhole marker that replaces it
     leaves the cell a deck cell, and row 1 then hides the cell's overlay, so the bridge shows a gap
     there. This happens when the map's overlay list places a veinhole: the markers go wherever the
     list puts the veinhole, whatever its neighbours hold, so a deck cell that the list has filled
     before the veinhole's cell (one of the three neighbours with a smaller *y*, or the one map west
     of it) holds a marker instead of its bridge type. What becomes of a neighbour that the list
     fills after the veinhole's cell is not covered here. Otherwise row 2's precedence over rows 3 to 7
     shows only where the rules give a bridge type one of the four flags.
   - **A type with several flags** follows the first row it meets, as the table orders them. The
     tiberium flag wins over the wall flag, the vein position, the crate flag and the rubble flag,
     the wall flag over the vein position, the crate flag and the rubble flag, the vein position
     over the crate flag and the rubble flag, and the crate flag over the rubble flag. The two marker
     positions of row 1 win over every flag. Outside deck cells and marker cells a tiberium overlay
     always takes its picture from claim 4, whatever its other flags.
   - Backing: `measurable` (OV1 for rows 3 and 5 and the theater independence; OV2 for the
     brightness and tint of rows 2 to 8; OV18 for ore and gem shade-row changes through shroud
     and glow; OV5 for row 4; OV9 for the image frames of rows 6 and
     8; OV8 for row 7; OV10 part A for row 1 and its precedence over row 8, part B for its
     precedence over row 2 and part C for its precedence over rows 3, 4, 6 and 7; OV7 part A
     for row 2's image frames and its precedence over row 8, parts B, C and D for its precedence
     over rows 7, 3 and 4. S-0104's scenario L23 Part C also measures the scaled raised level of
     deck cells. OV15 for the precedence among rows 3 to 7: parts A and B for row 3 over rows 4
     and 7, part C for row 4 over row 7, part D for row 3 over row 5, part E for row 4 over row 5,
     part F for row 5 over row 7, parts G and H for rows 3 and 4 over row 6, and parts I and J for
     row 6 over row 7.)
     `unmeasured` for ignoring a crate's non-zero data value: OV9 cannot establish whether crate
     placement resets it, so only the observed frame-0 result is measurable with that scenario.
     `unmeasured` for row 5 over row 6: a type with the crate flag is never placed by a skirmish
     map (S-0009 claim 12), so vein cells of a vein type given the crate flag could come only from
     vein growth during play, which is not covered here.
     `unmeasured` for row 2's precedence over rows 5 and 6. The vein overlay is the type at
     position 126, which is never a bridge position, so the map's overlay list never puts it in a
     deck cell. The vein rebuild at load (S-0009 claim 14) puts veins only in cells whose overlay
     allows veins, which a bridge type does not unless a rules layer says so, and whether the
     ground under a deck lets a vein pattern pass is not established. A bridge type given the
     crate flag is not placed by a skirmish map at all (S-0009 claim 12), and whether a crate that
     appears during play can land in a deck cell depends on where crates may appear, which is not
     covered here. So no scenario of this spec puts a vein or a crate in a deck cell (Open
     questions).
     OV17 for which positions make deck cells: it swaps flagless types into the four bridge
     positions with a reordered overlay type list in a loose rules copy (named test file) and
     moves the bridge types elsewhere. It samples the four positions and a few others; it cannot
     check every other position, so "only these four" rests on the sample.
   - Confidence: high for rows 3, 4, 5 and 8; medium for rows 1, 2, 6 and 7, for row 1's
     precedence over row 2, for row 2's precedence over rows 3, 4 and 7, for the precedence among
     rows 3 to 7 and for the bridge positions that make deck cells (a sample, OV17); low for row 2's precedence
     over rows 5 and 6.

2. **The start-up tables never change during a session.** The start-up resource table and the
   start-up mine table are built once, when the program starts. Loading a theater or starting
   another game does not rebuild them from any other palette. So outside deck and marker cells,
   ore, gems and veins show the colours of the temperate theater palette in every theater, and
   ore-spawning terrain objects show the
   colours of the snow unit palette in every theater (`quirks.resource_palettes_from_start`).
   Fixed tables do not imply fixed shade rows: shroud cover pieces and light-glow images can
   change ore and gem rows through light-alpha (claim 1; S-0110 claims 5 and 6).
   Fog remains outside skirmish scope.
   - Backing: `measurable` (OV1)
   - Confidence: high

3. **Smudges and terrain objects.**

   | Kind | Palette and shade table | Brightness |
   |---|---|---|
   | smudge piece | the tile shade table of the cell being painted | that cell's scaled ground level |
   | plain terrain object | its cell's tile shade table | its cell's scaled ground level |
   | ore-spawning terrain object | the start-up mine table | its cell's unscaled ground level *U* |

   - A smudge is painted once for every cell it covers (S-0102 claim 25). Each painting uses that
     cell's table and level, so where covered cells differ in light, the last painting of a screen
     pixel decides its colour, as it decides its position. A smudge takes the cell's tint like the
     terrain under it.
   - The terrain object rows restate S-0104 claim 13 and complete it: an ore-spawning terrain
     object takes no tint of any kind (the start-up mine table is untinted) but follows *U*.
   - The image frames of terrain objects and smudges are S-0102's (claims 23 and 25: smudges
     always show image frame 0).
   - Backing: `measurable` (OV1 and OV2 for the ore-spawning row, OV11 for smudges and plain terrain
     objects). `unmeasured` for which painting decides where a smudge's cells differ in light,
     because that depends on the order among terrain tile paintings, which is not known (S-0102
     claim 25).
   - Confidence: high for terrain objects; high for the smudge's table and brightness, medium for
     which painting decides where covered cells differ in light.

### Which sprite and which image frame

4. **Ore and gems take their picture from the tiberium's run, by cell position.** The body of a
   tiberium overlay on cell (*x*, *y*) is not drawn from the cell's own overlay type but from a type
   chosen in the run of the overlay's tiberium (Terms):
   - on a cell of slope shape index 0 (flat, S-0106): the type at list position *P* + ((*x*·*y*) mod *F*),
   - on a cell of slope shape index *s* from 1 to 4: with *k* = ⌊*S*/4⌋, the type at list position
     *P* + *F* + (*s* − 1)·*k* + ((*x*·*y*) mod *k*).

   *x* and *y* are the cell coordinates (integers, never negative), and the product is exact. The
   image frame is the cell's overlay data value (the ore or gem stage). So two flat cells whose
   stored overlay types differ but which belong to the same tiberium, have equal (*x*·*y*) mod *F*
   and have equal data values show the same body, and the overlay type a map stores in an ore cell
   only chooses the tiberium (and the shadow, claim 8).
   On other slope shapes, tiberium overlays are removed by the terrain rules (S-0211 claim 5,
   row 4), so they are never drawn.
   - An overlay with the tiberium flag that lies in no tiberium's runs is drawn as belonging to the
     first tiberium of the list.
   - Backing: `measurable` (OV3 parts A and B for the flat choice and the stored type's
     irrelevance, OV3 part C for the fallback, OV4 for slopes)
   - Confidence: high for the flat and sloped formulas; medium for the fallback to the first
     tiberium

5. **Rubble.** Only a rubble cell draws rubble; the other cells holding the rubble overlay draw
   nothing for it (their shadows follow claim 8). What a rubble cell draws depends on the building
   type it remembers:

   | Remembered building type | Rubble drawn |
   |---|---|
   | does not leave rubble (it cannot be a rubble cell then in normal play, S-0312 claim 20) | nothing |
   | leaves rubble and has a separate rubble sprite (S-0007 claim 11) | image frame 0 of that sprite |
   | leaves rubble, no separate rubble sprite, body sprite of at least 8 image frames (⌊*N*/2⌋ > 3) | image frame 3 of the building's own body sprite |
   | leaves rubble, no separate rubble sprite, body sprite of fewer than 8 image frames | nothing |

   The rubble is drawn with the cell's tile shade table, not with a house-colour palette, so it shows
   the tile palette's colours for its palette indices and takes the cell's tint (claim 1, row 7).
   This claim is claim 1's row 7: in a deck cell or a marker cell, or for a type that also has the
   tiberium, wall or crate flag or the vein position, the earlier row of claim 1 applies instead.
   - Backing: `measurable` (OV8) for the last three rows. `unmeasured` for the first row: a rubble
     cell is made only by a building type that leaves rubble, and a rules layer cannot change the
     type during a game, so the row cannot arise in play. It is stated so that every rubble cell
     has a defined picture.
   - Confidence: medium

6. **Bridge decks vary by position.** In a deck cell the body image frame is the overlay data value
   *d*, except that for *d* = 0 and *d* = 9 it is *d* + *T*(*x* mod 4, *y* mod 4), with the
   engine table *T* of Constants. Neighbouring deck pieces of the same data value therefore
   cycle through four consecutive image frames in a fixed 4 × 4 pattern.
   - Backing: `measurable` (OV7, part E for the data value 9 and a sampled value that does not vary; OV17
     for which cells are deck cells)
     `unmeasured` for every remaining data value other than 0 and 9: OV7 part E samples one,
     and no scenario here tests the exhaustive assertion over all other values.
   - Confidence: medium

7. **Wall connection bits.** A wall overlay's image frame is its overlay data value (claim 1,
   row 4): 16 × damage stage + connection bits. Bit *k* (value 2^*k*, *k* = 0 … 3) stands for the
   edge neighbour in step direction 2*k* (glossary; S-0204 claim 9 gives the same neighbour steps):
   bit 0 for map north, bit 1 for map east, bit 2 for map south and bit 3 for map west. The bit is
   set when that neighbour:
   - holds an overlay of the same overlay type; or,
   - for the overlay types at list positions 0 and 2, holds a building with health above 0 whose
     type is the rules' first Allied gate type and the neighbour lies map east or map west, the
     second Allied gate type and the neighbour lies map north or map south, or the wall tower type
     in any direction; or,
   - for the overlay type at list position 26, holds such a building of the first Soviet gate type
     (map east or map west) or the second Soviet gate type (map north or map south).

   Walls of different overlay types never join.

   **When the bits are recomputed.** A recomputation sets the connection bits of every wall cell
   in a set of cells from that cell's current neighbours, whatever the wall types involved, and
   keeps each cell's damage stage. Cells without a wall are left alone. The events and their sets:

   | Event | Cells recomputed |
   |---|---|
   | a wall overlay is placed in cell *c*: from the map's overlay list, from a wall building, or built in play (a newly placed wall starts with data value 0, so stage 0) | *c* and its four edge neighbours |
   | the wall in cell *c* is sold | *c* and its four edge neighbours |
   | the wall in cell *c* is removed by a wall hit, crushing included (S-0303 claim 16 and F4, S-0210 claim 8) | every cell within two edge steps of *c* |
   | a building of the wall tower type is placed, from the map's building list or built in play, or is destroyed and leaves the map; *c* is its top-left foundation cell | every cell within two edge steps of *c* |
   | a building of a first gate type is placed, or is destroyed and leaves the map; (*x*, *y*) is its top-left foundation cell | the cells (*x* − 1, *y*) and (*x* + 3, *y*), each with its four edge neighbours |
   | a building of a second gate type is placed, or is destroyed and leaves the map; (*x*, *y*) is its top-left foundation cell | the cells (*x*, *y* − 1) and (*x*, *y* + 3), each with its four edge neighbours |

   - A placed gate or tower is in its cells with health above 0 when its recomputation reads the
     neighbours, so the walls it recomputes join it if their type and direction allow (the conditions
     above). A destroyed one has health 0, so the walls its departure recomputes no longer join it.
   - A wall hit that leaves the wall standing changes its stage only. It recomputes no bits, not
     even its own.
   - The departure rows are stated for a building that is destroyed. A gate or tower building that
     leaves the map with health above 0, sold for example, is an open question (Open questions).
   - The gate rows reach the walls at both ends of a foundation three cells long and one cell deep
     along the gate's axis. A wall beside a gate of another shape that is neither one of the two
     cells nor an edge neighbour of one gets no bit toward the gate when the gate is placed, and
     keeps its bit toward the gate after the gate's departure, until another recomputation reaches
     it. In the same way, the tower rows reach every wall beside a tower with a one-cell
     foundation, but a wall beside a larger tower that lies more than two edge steps from its
     top-left foundation cell gets no bit when the tower is placed and keeps its bit after the
     tower's departure (`quirks.gate_tower_fixed_cells`).
   - When a building of the wall tower type is destroyed and leaves the map, every wall, of any
     wall type, that lies in an edge neighbour of its top-left foundation cell also takes a wall hit
     of 200 if it is at stage 0 when its hit is dealt. The hits are dealt in the order map north,
     map east, map south, map west (step directions 0, 2, 4, 6). The four cells are not edge
     neighbours of each other, so the order shows only in which hit takes which draws (S-0303 F4). A destroyed tower has health 0, so it no
     longer joins any wall, and every hit finds the hit wall's connection bits already recomputed
     without the tower. What the hit does is S-0303's (F4). Placing a tower gives no wall hits.
     Two consequences follow from F4:
     - A wall whose only link was the tower has no link when the hit lands. If its type has two
       damage stages, a successful hit removes it, and the removal recomputes its own reach. If
       its type has more, a successful hit leaves it standing at stage 1.
     - A departure hit never sets off F4's neighbour hits. It takes a wall from stage 0 to stage 1.
       Those hits need a type with more than two damage stages (*L* > 2) whose wall has just
       reached stage *L* − 1, the last stage at which it stands (a type with *L* stages stands at
       stages 0 to *L* − 1), so stage 2 or later. A three-stage type gets them at stage 2, not at
       stage 1.
     - Backing of these two consequences: they apply S-0303's F4 to the tower's hits. OV6 cases b
       and n read the first one. The second has no scenario of its own here and is
       `unmeasured`: S-0303's measurements of F4 are its evidence.
   - **Initial state.** At map load the overlay list, with its overlay data stream, is placed
     before the map's buildings, and the buildings are placed in entry order (S-0009 table "Map
     content" and claim 43); each placement recomputes the cells of the event table above. So a wall's
     connection bits at the start of the game are fixed by the last building placement of the map
     that recomputes its cell (the wall building that placed it, if any; a wall building in an edge
     neighbour; a gate whose two cells reach it; a wall tower within two edge steps), and its stage
     is always the one it was placed with:

     | Wall | Last building placement that recomputes its cell | Connection bits at the start |
     |---|---|---|
     | from the overlay list | none | without the overlay data stream: toward its edge neighbours of the same overlay type; with the stream: those of the data value the stream gives, connected or not (S-0009 claim 13) |
     | from the overlay list | a building of the map's building list | from the neighbours as they stand once that building is placed: the overlay list's walls and everything the map's building list has placed up to and including that entry (the walls of its wall buildings among them) |
     | from a surviving wall-building entry of the map's building list (stage 0, initially owned by the building's owner at placement; before play, reassigned by S-0210 claim 14 (a), including to no owner) | its own placement or a later one | as in the row above, for that last placement |

     A human-owned wall-building entry dropped under S-0009 claim 27 places no wall and causes
     no placement recomputation. Dropping it does not remove a wall supplied independently by the
     overlay list. For every surviving wall, whether from the overlay list or a wall building,
     S-0210 claim 14 (a) determines ownership before play, replacing any placement-time owner,
     even with no owner. That reassignment changes neither damage stage nor connection bits.

     Consequences: a wall at either end of a gate three cells long, or beside a one-cell tower,
     starts joined to it, whether it comes from the overlay list or from a wall building listed
     before or after the gate or tower. The connection bits of a stream value in the reach of a
     gate or tower are recomputed before the game starts, and only its stage survives. A wall
     beside a deeper gate or a longer tower, outside the cells of the event table, starts without a bit
     toward it unless a later placement, for example a wall building next to it, recomputes it.
   - What damage does to the stage, the removal of damaged pieces, and the hits on neighbouring
     walls belong to the wall-damage rules (S-0303 claim 16 and F4, S-0210 claim 8). Who may sell a
     wall is S-0213's (claim 12). Placing a gate, tower or wall building where walls already lie
     is not covered here.
   - Backing: `measurable` (S-0210 claim 14, scenario K12 part c, for pre-play ownership
     reassignment and dropped human-owned wall buildings; OV6: map A for the same-type bits, the initial state next to gates,
     recomputation after a placement in play and after a sale; map B for the stream's data values,
     the reach of a wall's placement, the absence of recomputation after a hit that leaves the wall
     standing, the reach of a removal, and the reach of a tower's and a gate's placement (its
     groups T and G); map C for the gate and tower bits by type and direction, the wrong-direction
     and wrong-type cases, the initial state beside a deeper gate (cases i and o), the departure of
     first gates (cases d and k, one fixed cell each), a second gate (cases l and m, both fixed
     cells), a deeper first gate (case i, `quirks.gate_tower_fixed_cells`), a tower with its hits,
     their reach to any wall type and the bits they find (cases b and n), and the reach of a
     tower's departure beyond its edge neighbours (group R); map D for a long tower's far cells at
     placement and departure). `unmeasured` for the order of the tower's four hits: it shows only
     in which draw each hit takes, and the destruction that sets the hits off takes draws of its
     own in the same tick (S-0312). Even a recorder of the random stream cannot tell those draws
     from the hits' draws, so no scenario separates the order.
   - Confidence: high for the same-type bits; medium for the gate and tower exceptions, the
     initial state of overlay-list walls and of wall buildings, the triggers and reaches of
     recomputation and the order of the tower's hits. High for pre-play ownership reassignment
     and dropped human-owned wall buildings.

8. **Overlay shadows.** Every cell that holds an overlay also draws the shadow, if any, that the
   table below gives it, in every game and at every detail level, cells with a veinhole marker
   overlay included.
   S-0102 claim 13, row "shadow image frame of an overlay", owns the shadow's depth bias, flat
   profile and depth writing; claims 16 and 26 own its drawing order and darkening. This claim owns
   its sprite and image frame, which follow the first row of this table that the cell meets, with *N* the frame count of
   that sprite and *d* the overlay data value:

   | Precedence | Cell | Sprite of the shadow | Shadow image frame |
   |---|---|---|---|
   | 1 | overlay type with the crate flag | the cell's overlay type | ⌊*N*/2⌋ |
   | 2 | overlay type with the rubble flag, in a cell that is not a rubble cell | — | none |
   | 3 | rubble overlay in a rubble cell whose building type does not leave rubble | — | none |
   | 4 | rubble overlay in a rubble cell, building with a rubble sprite | that rubble sprite | ⌊*N*/2⌋ |
   | 5 | rubble overlay in a rubble cell, any other building | the building's body sprite | ⌊*N*/2⌋ + 3 (only if ⌊*N*/2⌋ > 3, else none) |
   | 6 | any other cell, deck cells and tiberium overlays included | the cell's **own** overlay type | *d* + ⌊*N*/2⌋ |

   A deck cell has no row of its own here, so a deck cell whose overlay type has the crate or
   rubble flag follows rows 1 to 5, while its body follows claim 1, row 2.

   So the shadow of ore and gems comes from the overlay type stored in the cell, not from the type
   whose picture the body shows (claim 4). On slopes it is not chosen from the sloped run as the
   body is: it is whatever type the cell stores. In a
   deck cell the shadow leaves out the position term of claim 6. Body and shadow can therefore come
   from different sprites or image frames (`quirks.overlay_shadow_from_stored_type`).
   **Depth of every overlay shadow.** Whichever row above chose the sprite, an overlay shadow has
   one depth bias: −2 − 15 × *ℓ*<sub>s</sub> depth units, with the flat profile, and it writes depth
   (S-0102 claim 13, row "shadow image frame of an overlay"). It does not depend on the overlay's kind or flags (ore and
   gems, walls, crates, rubble, bridge decks, the veinhole markers, any other flat or upright type),
   on the row of claim 1 that draws the body, on the slope, or on the 4 levels of a bridge deck. So:
   - the shadow of an upright type does not take the −15 of the upright body, and the shadow of a
     deck cell does not take the deck's 4 levels: it lies 60 depth units farther than the deck's own
     pixels and never darkens them.
   - every overlay body of the repainted area is drawn before every overlay shadow (S-0102 claim 16,
     rank 3), so where a shadow falls on a flat overlay body at the same level the two tie and the
     body shows undarkened.
   - what overlay shadows do where they meet other shadows, bodies and drawings is S-0102 claim 27
     (its overlay-shadow rows), and holds for every shadow of this claim, the marker overlays' too.
   - Backing: `measurable` (OV3 part B and OV7 part A for the stored-type shadow and the deck
     shadow, S-0102 D35 for the depth of ore, upright and deck-cell shadows and OV16 for the depth of
     the shadows of walls, marker cells and rubble, OV10 for the veinhole markers, OV8 for rows 2, 4 and 5, OV9 for row 1, OV7 part B
     for row 2 in a deck cell, OV15 part I for row 1 over row 2 and part J for row 1 over rows 4
     and 5). `unmeasured` for row 3, and so for row 1's precedence over it, since row 3 cannot
     arise in play (claim 5).
     `unmeasured` for a crate's shadow ignoring a non-zero data value: OV9 cannot establish
     whether crate placement resets it; its measurable part tests the resulting shadow frame.
     `unmeasured` for the depth of a crate's shadow: crates cannot be placed on a skirmish map
     (S-0009 claim 12) and appear only in play, so no layout puts one beside a chosen shadow.
   - Confidence: high for the general row and the stored-type sprite, and for the depth of the
     shadows of ore, upright types, walls and deck cells; medium for the depth of the shadows of marker cells,
     rubble and crates, and for the rubble and crate rows, for the precedence of row 1 over rows 2 to 5 and for the veinhole markers' shadows.

9. **An image frame beyond the sprite draws nothing.** When the image frame chosen for a body or a
   shadow is not below the sprite's frame count, or that image frame holds no picture, nothing is
   drawn for it and nothing else changes. For example, an ore cell whose data value is at least the
   frame count of its body sprite shows no ore, and a data value of at least ⌊*N*/2⌋ makes the ore
   show a shadow image frame as its body.
   - Backing: `measurable` (OV12 parts A and B)
   - Confidence: medium

### Image numbers and bad tiberium data

10. **What an image number sets, and what it keeps.** Each time a rules layer gives a tiberium an
    image number *v* (Import mapping), its run changes by the Constants table:
    - every value other than 2 and −1 sets *P*, *F* and *S*,
    - the value 2 sets *P* and *F* only, and *S* keeps the value the tiberium already had,
    - the value −1 changes nothing, like an absent key.

    *S* is therefore always 0 or 8. It is 0 for a tiberium that has never been given a value other
    than 2 and −1, and 8 once it has. A tiberium that has never been given a value other than −1 has
    no run at all (claim 12). The values already there are the tiberium's held values in the sense
    of S-0006 (Terms there). So they can come from a lower layer of the same game or, because
    tiberiums carry over between games (S-0006 claim 10, `quirks.persist_tiberium_overrides`), from
    an earlier game of the session. A fresh program start clears them.

    So a tiberium given image number 2 after a value other than 2 and −1 in the same session has
    the run
    *P* = 27, *F* = 12, *S* = 8 (`quirks.image_two_keeps_sloped_run`). Its overlays on a cell of
    slope shape index *s* from 1 to 4 show, by claim 4, the type at list position
    39 + 2(*s* − 1) + ((*x*·*y*) mod 2), one of positions 39 to 46. Claim 11 does not apply to it.
    These positions also count as its sloped run when the tiberium of an overlay is determined
    (Terms).
    - Backing: `measurable` (OV13 part A for a lower layer of the same game, part B for an earlier
      game, part C for the value −1; OV4 for the runs that values 2, 3 and others set)
    - Confidence: high for the values each image number sets and for −1; medium for the carry-over
      between games, which rests on S-0006 claim 10

11. **A tiberium without a sloped run (departure).** A tiberium whose run has *S* = 0 (claim 10) has
    *k* = 0 in claim 4. In the original, drawing one of its overlays on a cell of slope shape index 1
    to 4 divides by zero, and the game fails. Our engine draws such a cell with the flat choice of
    claim 4 instead. The importer warns for every map cell that starts the game in that state,
    judged with the run in force for that game (claim 10). Ore of that tiberium that comes to lie on
    such a cell later (growth or spread) is drawn the same way.
    - Backing: `measurable` (OV4 part C)
    - Confidence: high

12. **A tiberium without an image number (departure).** A tiberium that has never been given an
    image number other than −1 has no run. In the original, the game fails whenever it determines
    the tiberium of a tiberium overlay (Terms) and no tiberium earlier in the tiberium list than the
    first tiberium without an image number has a run containing the overlay's list position. Our
    engine gives such a tiberium no run, so it owns no overlay, and the importer warns. A tiberium
    overlay whose tiberium (Terms) has no run, because it falls back to a first tiberium without
    one, has its body drawn from its own stored overlay type, with the overlay data value as the
    image frame.
    - Backing: `measurable` (OV14)
    - Confidence: medium

## Formulas

**Body type of a tiberium overlay** (claim 4), integers, *x*, *y* ≥ 0:

> flat: *p* = *P* + ((*x*·*y*) mod *F*)
>
> slope shape *s* from 1 to 4: *k* = ⌊*S*/4⌋, *p* = *P* + *F* + (*s* − 1)·*k* + ((*x*·*y*) mod *k*)

*p* is an overlay list position. *S* is never negative, so the truncation is a floor. The flat
form needs *F* ≥ 1, and *F* is always 12 (claim 10). The sloped form is defined only for *S* ≥ 4,
so that *k* ≥ 1; *S* is 0 or 8 (claim 10), and *S* = 0 is the departure of claim 11.

**Deck body image frame** (claim 6): *f* = *d* + *T*(*x* mod 4, *y* mod 4) when *d* is 0 or 9, else
*f* = *d*. Integers; *x*, *y* ≥ 0 are the cell coordinates, so both remainders lie in 0 … 3. Exact,
no rounding.

**Wall data value** (claim 7): *d* = 16·⌊*d*_old/16⌋ + Σ 2^*k* over the edge neighbours *k* = 0 … 3
that connect. *d*_old is the cell's data value before the recomputation, an integer from 0 to 255,
and the arithmetic is exact integer arithmetic.

**Reach of a recomputation** (claim 7): cell (*x*′, *y*′) is within two edge steps of (*x*, *y*) when
|*x*′ − *x*| + |*y*′ − *y*| ≤ 2. Integers, exact.

**Shadow image frame** (claim 8): *f*_shadow = *f*_base + ⌊*N*/2⌋, *N* the frame count of the
shadow's sprite, integers, division truncated toward zero (*N* ≥ 0). *f*_base is taken from the rows
of claim 8: the overlay data value *d* in the general row (deck cells and tiberium overlays
included), 0 for a crate and for rubble from a separate rubble sprite, and 3 for rubble from a
building's body sprite (drawn only when ⌊*N*/2⌋ > 3). A rubble cell whose building type does not
leave rubble, and the other cells with the rubble overlay, draw no shadow.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| veinhole marker positions | 167, 178 | overlay list position | never drawn (claim 1) |
| vein overlay position | 126 | overlay list position | S-0009 Terms |
| wall positions joining Allied gates and the wall tower | 0, 2 | overlay list position | claim 7 |
| wall position joining Soviet gates | 26 | overlay list position | claim 7 |
| reach of a removal, or of a wall tower's placement or departure | 2 | edge steps | claim 7, Formulas |
| cells a placed or departing gate recomputes around | 1 step back and 3 steps forward along its axis, from the top-left foundation cell | cells | claim 7 |
| wall hit on stage-0 walls next to a departing wall tower | 200 | wall-hit points | claim 7; outcome by S-0303 F4 |
| order of the departing wall tower's hits | map north, map east, map south, map west | step directions 0, 2, 4, 6 | claim 7 |
| data values per wall damage stage | 16 | overlay data value | connection bits = *d* mod 16, stage = ⌊*d*/16⌋ (Terms, claim 7) |
| bridge positions that make deck cells | 24, 25, 237, 238 | overlay list position | claim 1; the same positions as S-0009 claim 12 |
| deck variation table *T* | see the table below | image frames | by (*x* mod 4, *y* mod 4), claim 6 |
| deck data values that vary | 0, 9 | overlay data value | claim 6 |
| rubble image frame of a body sprite | 3 | image frame | claim 5 |
| overlay shadow depth bias | −2 − 15 × *ℓ*<sub>s</sub> | depth units | every overlay kind, flat profile, writes depth; *ℓ*<sub>s</sub> without a deck's 4 levels (claim 8; S-0102 claim 13) |
| ore and gem brightness | 1000 | ‰ | row 26 of 53 when light-alpha is 127, S-0104 |
| start-up table rows | 53 | rows | untinted, S-0104 claim 6 |
| start-up resource palette | temperate theater palette | palette role | read once per session |
| start-up mine palette | snow unit palette | palette role | read once per session |
| tiberium image runs | see the table below | overlay list positions | engine constants (claim 4) |

Deck variation table *T*(*x* mod 4, *y* mod 4), in image frames (claim 6):

| *y* mod 4 \ *x* mod 4 | 0 | 1 | 2 | 3 |
|---|---|---|---|---|
| 0 | 0 | 1 | 2 | 3 |
| 1 | 3 | 2 | 1 | 0 |
| 2 | 2 | 3 | 0 | 1 |
| 3 | 1 | 0 | 3 | 2 |

Image runs of a tiberium by image number *v* (an integer image-number value, unit: image number,
Import mapping). These are positions built into the original, not stock data:

| Image number *v* | *P* | *F* | *S* |
|---|---|---|---|
| 2 | 27 | 12 | not set: keeps its value, 0 or 8 (claim 10) |
| 3 | 127 | 12 | 8 |
| 4 | 147 | 12 | 8 |
| any other value except −1 (0 and 1 included) | 102 | 12 | 8 |
| −1, or the key absent | nothing is set: the run stays as it was, and a tiberium never given another value has none (claims 10 and 12) | | |

*P* is an integer overlay list position; *F* and *S* are integer counts of overlay types in the
flat and sloped runs respectively.

## State and ordering

- The drawing rules of claims 1 to 9 hold in every drawn frame; drawing itself changes no game
  state. Claim 7 also specifies events that change connection bits and cause wall hits, whose
  damage outcomes belong to S-0303. The runs of claim 10 are rules state, set when rules layers are
  read (S-0006).
- A change of light (S-0104 claim 9) shows on the next drawing of the cell for every row that
  reads a light level. Ore and gems never change with the cell's light level, but shroud cover
  pieces and light-glow images can change their shade row through light-alpha (claim 1 and
  S-0110 claims 5 and 6). Each drawn frame uses its own cover and glow result, with initial
  light-alpha 127; earlier drawn frames leave no darkening behind.
- The connection bits of claim 7 change on the tick of the event that recomputes them. They are
  game state (they are overlay data values) and the wall's picture follows them.
- A recomputation reads the neighbours as they are at the moment of its event, not at the end of
  the tick. Its result stands until the next recomputation that reaches the cell. So where two
  events of one tick touch the same cells, the result follows their order in the tick (S-0203).
  For example, a wall is placed next to a cell *n* in the tick in which a gate beside *n* is
  destroyed, and the gate's fixed cells do not reach *n*. Then *n* ends with a bit toward the gate
  if the placement comes before the gate's health reaches 0, and without one if it comes after.

| State | Event | Result |
|---|---|---|
| wall cell, bits *b* | a wall overlay of any type is placed or sold in this cell or in one of its four edge neighbours | bits recomputed from the current neighbours, stage kept (claim 7) |
| wall cell, bits *b* | a wall within two edge steps is removed by a wall hit | bits recomputed, stage kept (claim 7) |
| wall cell, bits *b* | a wall hit leaves this or another wall standing | bits unchanged (claim 7) |
| wall cell within two edge steps of the top-left foundation cell of a wall tower building | that building is placed | bits recomputed with the tower, stage kept (claim 7) |
| wall cell within two edge steps of a wall tower building's top-left foundation cell | that building is destroyed and leaves the map | bits recomputed without the tower, stage kept; if the cell is an edge neighbour of the top-left cell and its wall is at stage 0 when its hit is dealt, it also takes a wall hit of 200, which finds its bits already recomputed; the four hits are dealt in the order map north, east, south, west (claim 7) |
| other wall cell next to a wall tower building | that building is placed, or is destroyed and leaves the map | bits unchanged (claim 7) |
| wall cell at, or edge-adjacent to, one of the two cells of claim 7's gate rows | that gate building is placed, or is destroyed and leaves the map | bits recomputed, with the gate when it is placed and without it when it leaves, stage kept (claim 7) |
| other wall cell next to a gate building | that building is placed, or is destroyed and leaves the map | bits unchanged (claim 7) |
| any session | a theater is loaded or a new game starts | start-up tables unchanged (claim 2) |
| tiberium with run *S* | a rules layer gives it image number 2 | *P* = 27, *F* = 12, *S* unchanged (claim 10) |
| tiberium with any run, or none | a rules layer gives it an image number other than 2 and −1 | *P* and *F* from the Constants table, *S* = 8 (claim 10) |
| tiberium with any run, or none | a rules layer gives it image number −1, or omits the key | unchanged (claim 10) |

## Edge cases and quirks

- **Walls in the local player's colour** (claim 1, row 4). The original draws every wall with the
  local player's 53-shade house-colour palette, although each wall cell records its owner,
  which may differ from the placing house (claim 7; S-0210 claim 14). A wall's colour therefore depends on which house is the local player, not on who built
  it.
  `quirks.wall_local_player_colour` (default on). Off: a wall is drawn with the 53-shade
  house-colour palette of the house recorded as its owner; a wall with no owner keeps the local
  player's palette.
- **Resource and mine palettes fixed at program start** (claim 2). Ore, gems and veins use the
  temperate theater palette, and ore-spawning terrain objects the snow unit palette, whatever the
  theater. `quirks.resource_palettes_from_start` (default on). Off: both tables are rebuilt with the
  untinted rule when a theater is loaded, the resource table from that theater's theater palette
  and the mine table from that theater's unit palette (S-0103 claims 2 and 3).
- **Overlay shadows from the stored type** (claim 8). The shadow of a tiberium overlay comes from
  the cell's own overlay type, and a deck cell's shadow leaves out the position term.
  `quirks.overlay_shadow_from_stored_type` (default on). Off: the shadow comes from the same sprite
  as the body (claim 4's choice for tiberium overlays) and uses the body image frame plus ⌊*N*/2⌋ (with
  the deck position term included).
- **Gate and tower placements and departures reach fixed cells** (claim 7). When a gate building
  is placed or destroyed, only the walls around two cells fixed relative to its top-left
  foundation cell are recomputed, as if every gate were three cells long along its axis and one
  cell deep. A wall beside the gate that is neither one of those two cells nor an edge neighbour
  of one gets no bit toward a newly placed gate and keeps its bit toward a departed one: for
  example a wall beside the third or a later row of a deeper foundation, or beside the far end of
  a gate one cell long. When a wall tower building is placed or destroyed, only the cells within
  two edge steps of its top-left foundation cell are recomputed, as if every wall tower had a
  one-cell foundation. A wall beside a larger tower that lies further away gets no bit toward a
  newly placed tower and keeps its bit toward a departed one.
  `quirks.gate_tower_fixed_cells` (default on). Off: at the placement and at the departure of a
  gate or tower, in addition to the cells of claim 7, every wall cell that is an edge neighbour of
  any foundation cell of that building is recomputed. The switch changes connection bits only:
  the tower's wall hits still go to the edge neighbours of its top-left foundation cell, as in
  claim 7.
- **Shadows of the marker overlays** (claims 1 and 8). The marker overlays' bodies are skipped but their
  shadows are not, and those shadows have the depth of every overlay shadow (claim 8). No switch: it
  follows the general shadow rule.
- **Crates ignore their data value** (claim 1, row 6, and claim 8). No switch.
- **Image number 2 keeps the sloped run** (claim 10). Image number 2 sets the first position and
  the flat run but leaves the sloped run as it was, so whether such a tiberium has sloped pictures
  depends on the values it was given before, in this game or (through tiberium carry-over) an
  earlier one. `quirks.image_two_keeps_sloped_run` (default on). Off: image number 2 sets *S* = 0,
  so such a tiberium always lacks a sloped run and claim 11 applies to it on slopes. The switch is
  independent of `quirks.persist_tiberium_overrides` (S-0006), which decides whether values from
  an earlier game are there to keep.
- Claims 11 and 12 are departures (glossary), not quirks.

## Import mapping

| Source | Key | Parse form | Feeds | Absent |
|---|---|---|---|---|
| rules, overlay type | `Tiberium` | boolean | tiberium overlay: row 3, claim 4 | no |
| rules, overlay type | `Wall` | boolean | wall overlay: row 4, claim 7 | no |
| rules, overlay type | `Crate` | boolean | row 6; shadow row of claim 8 | no |
| rules, overlay type | `IsRubble` | boolean | row 7, claim 5 | no |
| rules, tiberium | `Image` | integer (S-0003) | image number *v*: the run of claim 4 (claim 10, Constants table) | −1 (an explicit −1 acts the same): the run is left as it was (claim 10); a tiberium never given another value has none (claim 12) |
| rules, terrain object type | `SpawnsTiberium` | boolean | ore-spawning terrain object: start-up mine table, *U* (claim 3) | no |
| rules, building type | `LeaveRubble` | boolean | rubble drawn at all (claim 5; creation is S-0312 claim 20) | as S-0312 gives it |
| art, building | `Rubble` | sprite name (S-0007 claim 11) | separate rubble sprite (claim 5) | none: the body sprite's image frame 3 is used |
| rules, `[General]` | `GDIGateOne`, `GDIGateTwo` | building type name | first and second Allied gate type (claim 7) | none |
| rules, `[General]` | `NodGateOne`, `NodGateTwo` | building type name | first and second Soviet gate type (claim 7) | none |
| rules, `[General]` | `WallTower` | building type name | wall tower type (claim 7) | none |
| rules, building type | `Wall` | boolean | the building wall flag of a wall building (Terms) | no |
| rules, building type | `ToOverlay` | overlay type name | the wall overlay type a wall building places (Terms, claim 7) | none |
| map | `[OverlayPack]`, `[OverlayDataPack]` | S-0009 claims 12 and 13 | overlay types and data values | S-0009 |

- The overlay list positions in Constants refer to the overlay type list in registration order
  (S-0004). A mod that reorders the list changes which types they denote.

## Measurement scenarios

**Common setup.** Every scenario uses the glossary's measurement setup, a purpose-made tiny map,
neutral test lighting (S-0104) unless the scenario changes it, the local player in house slot 0
with a known colour, all ground revealed at the start and game speed index 4. Unless a scenario
explicitly tests cover or glow, sample only pixels with light-alpha 127, away from shroud edges
and light-glow images. Fog is always off in skirmish. OV18 overrides the cover artwork, not this
fog setting. Pixel colours are
read from screenshots at 800 × 600 in the 5-6-5 layout and compared with the expected palette
entries after S-0103's 16-bit packing (claim 9). "Sprite frame *f*" means the picture of image frame
*f* of the named sprite, rendered offline from the stock file with the expected palette for
comparison. Use one bystander computer player separated from the observed area, which contains
all named test cells and objects; the glossary's bystander void rule applies. Unrelated mods are
off, and named test files are removed afterwards. Tick-labelled lossless
screen capture and scripted input are the recording tools; any needed harness extension is
proposed to the user, not assumed installed.

In every scenario, any distinguishable mismatch with a stated expected sprite, image frame,
colour, presence or event tick falsifies that part, not only the examples listed under
“Falsified if”. An ambiguous match, a missing required subject, or a failed setup is
inconclusive, never evidence for the claim.

- **OV1, start-up tables in every theater (claims 1 to 3).** Two sessions, one run each (the main session and variant B). Three maps, snow, urban and desert,
  played one after another in one session, after a first game on a temperate map. Each holds a flat
  ore cell at data value 5, a flat gem cell at data value 5, a vein cell, and an ore-spawning
  terrain object. Variant B repeats the session with loose copies of the temperate theater palette
  and the snow unit palette in which the palette entries used by those sprites are replaced by
  distinctive colours, to tell the start-up palettes from look-alike theater palettes.
  - *Observable:* the colours of the ore, gems and veins, and of the ore-spawning object.
  - *Expected:* in every theater, ore and gems show the temperate theater palette's colours exactly
    (row 26, no −1); veins show the same palette at their brightness; the object shows the snow unit
    palette's colours at row 26 of the untinted table. In variant B the replaced colours appear in
    all three theaters.
  - *Falsified if:* any theater shows its own theater, tile or unit palette for these objects, or
    the colours carry S-0104's −1, or variant B's colours appear only in some theaters.
- **OV2, brightness and tint (claims 1 and 3).** One temperate map, four runs: neutral test
  lighting; map ambient 0.43; a red map tint of 0.50 set by the map; and an ion storm. Crates are
  on. Objects: an ore cell, a vein cell, a wall, a vehicle of the local player beside the wall as a
  reference, a fence-like overlay whose type has none of the four flags, an ore-spawning terrain
  object and a plain terrain object, each on its own cell, and a high bridge whose deck cells lie
  over flat ground. Crates cannot be placed by a skirmish map (S-0009 claim 12), so the crates are
  the ones that appear during play; the lighting of all four runs is map-wide, so any crate on
  screen serves.
  - *Observable:* the colours of each object, of the bridge deck and of every crate on screen, and
    of the terrain beside each, from a screenshot taken in each run once the lighting is steady.
  - *Expected:* the ore cell is identical in all four runs. The vein cell and the ore-spawning
    object darken with the ambient but keep their hue under the tint and the storm. The wall darkens
    with the ambient and keeps the house colour under the map's red tint; under the storm its house
    colour changes as the reference vehicle's does (S-0104 claim 3). The fence-like overlay, the
    plain terrain object, the bridge deck and every crate darken with the ambient, turn red under
    the map's tint and take the storm tint, as the terrain beside them does, in the tile palette's
    colours. The deck's brightness is its cell's scaled raised level (S-0104 claim 9 and
    Formulas), not the ground level of the terrain beside it.
  - *Falsified if:* the ore changes in any run; veins or the ore-spawning object take the red tint
    or the storm tint; the wall turns red under the map's tint, does not darken with the ambient, or
    differs from the reference vehicle's house colour under the storm; or the fence-like overlay,
    the plain terrain object, the deck or a crate fails to darken, to turn red or to take the storm
    tint as the terrain beside it does, or shows a house colour. The crate part is inconclusive if
    no crate appears on screen in a run.
  - *Additional light-source run:* neutral test lighting with red building lights covering each
    subject's cell (named test map buildings, as in OV11). One run. Their bodies and glows must
    not overlap sampled pixels. Sample crates only if they appear within a light's radius.
    *Expected:* ore keeps its neutral-run pixels. Veins and the ore-spawning object keep their
    untinted-table hue, although their brightness follows the light. Walls keep their house-colour
    hue while their brightness changes. The fence-like overlay, plain terrain object, bridge deck
    and sampled crates take the same red cell tint as terrain beside them.
    *Falsified if:* ore changes, veins, the ore-spawning object or walls acquire the red tint,
    or any of the tile-table subjects fails to acquire it. The crate part is inconclusive if
    no crate appears within a light's radius.
- **OV3, ore picture by position (claims 4 and 8).** One run per part. Every overlay cell of this and the next
  scenarios is a legal cell (S-0009 claim 12). *Observable:* in every part, which sprite frame each
  cell shows as its body and as its shadow, matched against offline renders. Part A: on a map at least 40 × 40,
  a flat row of 13 consecutive cells (*x*, *y*) with *y* ≡ 1 (mod 12), so that (*x*·*y*) mod 12 =
  *x* mod 12, all holding the overlay type at the first position *P* of one tiberium's run, data
  value 3. *Expected:* cell (*x*, *y*) shows sprite frame 3 of the type at position *P* + (*x* mod
  12), so the first and the last cell of the row look the same and differ from the second. A second
  row with the same *y* mod 12 stores a different type of the same run in every cell: it looks the
  same as the first. *Falsified if* the pictures follow the stored type. Part B: a loose copy of the
  sprite at position *P* + 5 whose shadow image frames are replaced by a full rectangle. The cell of
  the first row with *x* mod 12 = 5 shows the normal shadow of the stored type, not the rectangle; a
  cell whose stored type is *P* + 5 and whose (*x*·*y*) mod 12 is not 5 shows the rectangle shadow
  under a different body. *Falsified if* the rectangle follows the body. Part C (fallback): the
  map's override section gives the tiberium flag to an overlay type that has none of the four flags
  and whose list position lies in no tiberium's run (check the runs against the base file). Flat
  cells of that type at data value 3, at known coordinates. *Expected:* cell (*x*, *y*) shows
  sprite frame 3 of the type at *P*₁ + ((*x*·*y*) mod *F*₁), where *P*₁ and *F*₁ belong to the run of
  the first tiberium of the tiberium list, in the start-up resource table's colours. *Falsified if*
  the cells show the type's own sprite, or the run of another tiberium.
- **OV4, ore on slopes (claims 4, 10 and 11).** One run per part. *Observable:* the body each ramp cell shows,
  matched against offline renders, and whether the game stops. Part A: cells of a tiberium whose
  run has *S* = 8 (its image number is neither 2 nor −1), on legal cells, on ramps of slope shape index 1 to
  4, at positions with *x*·*y* even and odd, data value 2. *Expected:* each shows sprite frame 2 of
  the type at *P* + 12 + 2(*s* − 1) + ((*x*·*y*) mod 2). Part B: the same with a tiberium of image
  number 3, and again with one of image number 4 (expected: the types at 127 + 12 + 2(*s* − 1) +
  ((*x*·*y*) mod 2) and 147 + 12 + 2(*s* − 1) + ((*x*·*y*) mod 2)). Part C: start the program fresh, so that nothing is left from an earlier game. Use a
  tiberium that the base rules give image number 2 and that no layer of the session gives another
  value; check the base file and the map, and if no such tiberium exists, give one image number 2 in
  a loose copy of the rules file (named test file). Place its overlays (positions 27 to 38) on a
  ramp. *Expected:* the original stops when the cell is first drawn. *Falsified if* part A shows a
  flat picture or the wrong slope pair, part B shows types other than those stated, or part C
  draws the cell.
- **OV5, wall colour (claim 1, row 4).** A skirmish map with walls placed as overlays and a wall
  line placed as wall buildings in the map's building list, owned by the opponent's house
  (such buildings become wall overlays when the map loads). Two runs that swap the local player
  between house slots 0 and 1, with different colours. *Observable:* the colours of every wall
  piece, compared with each house's 53-shade house-colour palette. *Expected:* in each run every wall, the
  opponent's included, shows the local player's colour. *Falsified if* any wall shows its owner's
  colour when that differs from the local player's.
- **OV6, wall connection bits (claim 7).** Four maps, one run each. Each map is played from a
  fresh program start, because the override section's game-wide values carry over between games
  (S-0006 claim 10). "Sprite frame *f*" here is the image frame of the wall's own type, so *f* is
  its data value. Where a wall hit may change a stage, only *f* mod 16, the connection bits, is
  read.
  - *Common to the maps.* A skirmish map cannot place a building owned by a human player (S-0009
    claim 27). So the map's gates, towers and wall buildings belong to the computer opponent, the
    only walls that are sold are walls the local player builds during play, and a gate or tower
    leaves the map by being destroyed by units of the local player whose warhead has none of the
    wall flag, the wood flag and the absolute wall-destroyer flag, so that the attack gives no wall
    hits (S-0303 claim 16). The local player's side can build the wall type at list position 2
    (check the base file). The override section names ordinary building types that do not open as
    gates and leave no rubble as the gate and tower types. In maps A, B and C the wall tower type
    has a one-cell foundation, the first Allied gate type a foundation exactly three cells long
    from map west to map east, and the second Allied gate type one exactly three cells long from
    map north to map south (check the base files). Test groups lie at least three cells apart, and
    a group's own buildings stand only where the group places them. The five types named as gate
    and tower types are types that neither side can build during play (check the base files), so
    every building of those types is one the map places. Play uses scripted input. *x*
    and *y* name the top-left foundation cell of the group's building.
  - *Observable:* the sprite frame of every test wall at the start and from the tick of each
    event, matched against offline renders of the wall types' sprites, and whether each test wall
    is still there.
  - *Map A,* without an overlay data stream. A straight line of three walls from the overlay list,
    running map west to map east, of a wall type other than the one at list position 2. W1: a wall
    at list position 2 from the overlay list, one step map west of the top-left foundation cell of
    a building of the first Allied gate type, near the local player's start, with nothing else
    next to it. W2: a wall building that becomes a wall at list position 2, listed after a second
    building of the first Allied gate type and placed one step map west of that building's
    top-left foundation cell. W3: the same, listed before a third building of the first Allied
    gate type. *Expected at the start:* the line joins itself (its ends show 2 and 8, its middle
    10); W1, W2 and W3 all show 2. W1 and W3 get the bit only from the recomputation at their
    gate's placement, W2 already at its own placement. Then, during play:
    - The local player builds a wall at list position 2 one step map north of W1. From that tick
      W1 shows 3 (the new wall and the gate) and the new wall shows 4.
    - The local player builds a plus of walls at list position 2, a centre and four arms, with one
      arm next to an end of the line. Once it is complete, the centre shows 15, each arm 1, 2, 4 or
      8 by its direction toward the centre, and the line is unchanged. Then the local player sells
      the arm map north of the centre. From that tick the centre shows 14 and the other arms are
      unchanged.
  - *Map B,* with an overlay data stream that gives every legal cell data value 0. A plus of walls
    at list position 2 from the overlay list, next to the local player's start, and a straight line
    of seven walls from the overlay list running map west to map east, numbered 1 to 7 from the
    west, of a wall type whose damage-stage count is at least 3 (check the base file). *Expected at
    the start:* every wall outside groups T and G shows sprite frame 0. Then, during play:
    - The local player builds a wall at list position 2 one cell map east of the east arm. From
      that tick the new wall shows 8 and the east arm 10 (the centre to its map west, the new wall
      to its map east). The centre, two cells from the new wall, and the other arms still show 0.
    - A unit of the local player whose damage amount is at least the line type's wall strength and
      whose warhead has the wall flag and reaches only its target cell (S-0303) attacks wall 4 until
      it is gone. While wall 4 stands, every wall of the line keeps *f* mod 16 = 0. From the tick
      it is removed, walls 2 and 6 show *f* mod 16 = 10, wall 3 shows 8, wall 5 shows 2, and walls
      1 and 7 still show 0. This part is inconclusive if a wall other than wall 4 disappears.
    - *Group T (the reach of a tower's placement).* A wall tower building with no wall in any of
      its edge neighbours. Two pairs of walls from the overlay list, each pair of one wall type:
      one wall at (*x*, *y* − 2) with its partner at (*x*, *y* − 3), and one at (*x* − 1, *y* − 1)
      with its partner at (*x* − 2, *y* − 1). Every other cell within three edge steps of the tower
      holds no wall. The stream gives all four the value 0, although each pair joins. *Expected at
      the start:* the wall at (*x*, *y* − 2) shows 1 and the one at (*x* − 1, *y* − 1) shows 8 (two
      edge steps away, so recomputed when the tower is placed); both partners, three edge steps
      away, still show the stream's 0.
    - *Group G (a gate's two cells at its placement).* A building of the first Allied gate type
      with no wall in any of its edge neighbours and none in the cells (*x* − 1, *y*) and
      (*x* + 3, *y*). Two pairs of walls from the overlay list, each pair of one wall type: one
      wall at (*x* − 1, *y* − 1) with its partner at (*x* − 2, *y* − 1), and one at (*x* + 3,
      *y* − 1) with its partner at (*x* + 4, *y* − 1). The stream gives all four the value 0.
      *Expected at the start:* the wall at (*x* − 1, *y* − 1) shows 8 and the one at (*x* + 3,
      *y* − 1) shows 2 (edge neighbours of the gate's two cells); both partners still show 0.
  - *Map C,* without an overlay data stream. The override section names five different building
    types as the first and second Allied gate types, the first and second Soviet gate types and the
    wall tower type. For the first Soviet gate type it names a type whose foundation is at least 3
    cells long from map north to map south (check the base files); the Allied gate and tower types
    are shaped as the common setup says. The override section also gives the wall types at list
    positions 2 and 26 a damage-stage count of 2 and a wall strength of at most 200 (S-0303 F4).
    In map C only the tower's departure gives wall hits, so this changes nothing else there. The
    map places buildings of these types. Each test wall *W* comes from the overlay list and has a
    foundation cell of its building as an edge neighbour. In the cases marked "with *P*", a wall
    *P* lies next to *W* on the side away from the building: a wall building of a wall type other
    than *W*'s, listed after every building of the five types, whose placement recomputes *W*
    (claim 7). The cases:

    | Case | List position of *W* | Building | Direction from *W* to the building | Expected sprite frame of *W* at the start |
    |---|---|---|---|---|
    | a | 0 | second Allied gate, with *P* | map south | 4 |
    | b | 2 | wall tower, with *P* | map north | 1 |
    | c | 0 | wall tower (another building, never destroyed), with *P* | map west | 8 |
    | d | 26 | first Soviet gate, with *P*; *W* one step map west of its top-left foundation cell | map east | 2 |
    | e | 26 | second Soviet gate, with *P* | map south | 4 |
    | f | 2 | second Allied gate, with *P* | map east | 0 (wrong direction) |
    | g | 26 | first Allied gate, with *P* | map west | 0 (gate of the other side) |
    | h | a wall-flag type other than 0, 2 and 26 | first Allied gate, with *P* | map east | 0 |
    | i | 26 | first Soviet gate (a second building), with *P*; *W* one step map west of its foundation cell two steps map south of the top-left one | map east | 2 |
    | j | 2 | first Allied gate, no *P*; *W* at (*x* − 1, *y*) | map east | 2 (recomputed at the gate's placement) |
    | k | 2 | first Allied gate (another building), with *P*; *W* at (*x* + 3, *y*), one step map east of the foundation's east end | map west | 8 |
    | l | 0 | second Allied gate (another building), with *P*; *W* at (*x*, *y* − 1) | map south | 4 |
    | m | 2 | the building of case (l), with *P*; *W* at (*x*, *y* + 3), one step map south of the foundation's south end | map north | 1 |
    | n | 26 | the wall tower of case (b), with *P* | map west | 0 (a type that does not join towers) |
    | o | 26 | first Soviet gate (a third building), no *P*; *W* one step map west of its foundation cell two steps map south of the top-left one | map east | 0 (outside the gate's two cells, so not recomputed at its placement; 2 with `quirks.gate_tower_fixed_cells` off) |

    *Group R (the reach of a tower's departure).* A wall tower building (a third one) with no wall
    in any of its edge neighbours. *A*: a wall at list position 26 from the overlay list at
    (*x*, *y* − 2), two edge steps from the tower and not next to it. A building of the first
    Soviet gate type (a fourth one) stands map east of *A*, with its top-left foundation cell at
    (*x* + 1, *y* − 4), so that *A* lies one step map west of its foundation cell two steps map
    south of the top-left one. Neither of the gate's two cells, (*x*, *y* − 4) and (*x* + 4,
    *y* − 4), reaches *A*, and the gate never covers the tower's cell. *P*: a wall building of
    another wall type at (*x* − 1, *y* − 2), three edge steps from the tower, listed after the gate
    and the tower. *Expected at the start:* *A* shows 2 (the gate to its map east).

    Then, during play, destroy the buildings of cases (d), (i), (k), (l) and (b), one at a time,
    then group R's gate, then group R's tower. From the tick each leaves the map: *W* of cases
    (d), (k), (l) and (m) shows 0; *W* of case (i) still shows 2 (0 with
    `quirks.gate_tower_fixed_cells` off). *W* of cases (b) and (n) is gone from the tick the tower
    leaves: each takes the departing tower's wall hit of 200, which succeeds without a draw, and
    finds no link left, so the hit removes it (claim 7, S-0303 F4). *A* still shows 2 after its
    gate leaves (0 with the switch off), and shows 0 from the tick its tower leaves, because the
    tower's departure recomputes every cell within two edge steps. The parts for cases (d), (i),
    (k), (l) and (m) and for group R are inconclusive if their wall is removed or changes stage
    when its building is destroyed.
  - *Map D (a long tower),* without an overlay data stream. The override section names as the wall
    tower type a building type whose foundation is at least three cells long from map west to map
    east (check the base files). One building of that type, with no wall in any edge neighbour of
    its top-left foundation cell. *W*: a wall at list position 2 from the overlay list, one step map
    east of the foundation's east end, in the row of the top-left cell, so three or more edge steps
    from that cell. *P*: a wall building of another wall type one step map east of *W*, listed
    after the tower. *W*2: a wall at list position 2 from the overlay list, one step map north of
    the third foundation cell of that row, (*x* + 2, *y* − 1), so three edge steps from the top-left
    cell, with no *P*. *Expected at the start:* *W* shows 8 and *W*2 shows 0 (4 with
    `quirks.gate_tower_fixed_cells` off). Destroy the tower. From the tick it leaves the map, *W*
    still shows 8 (0 with the switch off) and *W*2 still shows 0. This part is inconclusive if *W*
    or *W*2 is removed or changes stage.
  - *Falsified if:* any sprite frame shown at the start in map A, B, C or D differs from the one
    stated (in particular W1, W3 or case (j) lacking the gate bit, or case (o) or *W*2 showing
    one); different types join; the bit values do not match the directions; a placement or sale
    in map A does not change the sprite frames on that tick as stated; in map B a wall outside
    groups T and G shows a connection bit at the start, the placement reaches the centre or leaves
    the east arm unchanged, a wall of the line gains a bit while wall 4 stands, or after its
    removal walls 2, 3, 5 and 6 do not show the stated bits or walls 1 and 7 gain one; in group T
    or G a wall that should be recomputed shows 0, or a partner shows a bit; *W* of case (d), (k),
    (l) or (m) keeps its bit, *W* of case (i) loses its bit, or *W* of case (b) or (n) still stands
    after the tower has left the map (at stage 0, no hit landed; at stage 1, with sprite frame 16
    plus its bits, the hit landed while the wall still had a link); in group R *A* loses its bit
    when its gate leaves or keeps it after its tower leaves; in map D *W* loses its bit or *W*2
    gains one.
  - *Ownership companion test:* S-0210 claim 14, scenario K12 part c, specifies the setup,
    recorder, expected ownership and falsifiers for wall-building reassignment, the no-owner
    result and dropped human-owned entries. Run it alongside OV6 for claim 7's ownership
    statements; the connection-bit expectations above are unchanged by reassignment.
- **OV7, bridge decks (claims 1, 6 and 8).** A high bridge four cells long, on legal cells, whose
  deck cells have data value 0, at known coordinates, under a red map tint of 0.50. Six runs.
  - *Observable:* the sprite frames each deck cell shows as its body and its shadow, matched
    against offline renders, and the deck's colours compared with the terrain beside the bridge.
  - *Part A,* stock rules. *Expected:* deck cell (*x*, *y*) shows sprite frame *T*(*x* mod 4,
    *y* mod 4) of the deck type, and its shadow is sprite frame 0 + ⌊*N*/2⌋ in every cell.
  - *Parts B, C and D.* The map's override section gives the overlay types of the deck cells the
    rubble flag (part B), the tiberium flag (part C) or the wall flag (part D). *Expected:* in parts
    B and C the deck's bodies look as in part A. In part B the deck cells cast no shadow (claim 8,
    row 2); in part C their shadows are as in part A. In every part the deck shows the tile
    palette's colours, turned red like the terrain. Part D reads only the colours, because the wall
    flag can change the data values. Each part is inconclusive if the bridge does not appear with
    its deck cells.
  - *Part E, other data values.* Two runs as part A, except that the overlay data stream gives
    every deck cell data value 9 (run E1) or one value *d*′ that is neither 0 nor 9 (run E2).
    Choose *d*′ so that every image frame named here lies among the deck type's body image frames
    (check the stock sprite). *Expected:* in run E1 deck cell (*x*, *y*) shows sprite frame
    9 + *T*(*x* mod 4, *y* mod 4); in run E2 every deck cell shows sprite frame *d*′. *Falsified
    if* run E1 shows the same sprite frame in every deck cell or any frame other than
    9 + *T*(*x* mod 4, *y* mod 4), or the sprite frames of run E2 vary with position. Each run is
    inconclusive if the bridge does not appear with its deck cells, or if a sprite frame named
    here lies beyond the deck type's body image frames.
  - *Falsified if:* in part A every deck cell shows sprite frame 0, or the shadow varies with
    position; in part B the deck cells draw no body, or cast a shadow; in part C a deck cell shows a picture from a
    tiberium's run, or untinted colours of the temperate theater palette; in part D the deck shows
    the local player's house colours; or in any part the deck ignores the tint.
- **OV8, rubble (claims 5 and 8).** One run. Destroy three buildings that leave rubble, under a
  red map tint of 0.50: one with a separate rubble sprite, one without but with at least 8 body
  image frames, and one with fewer.
  - *Observable:* the picture drawn at each building's former foundation cells, body and shadow,
    matched against offline renders, and its colours compared with the ground beside it.
  - *Expected:* sprite frame 0 of the rubble sprite is drawn once, at the first building's
    top-left cell, with sprite frame ⌊*N*/2⌋ of the rubble sprite as its shadow. Sprite frame 3 of
    the second building's body sprite is drawn at its top-left cell, with sprite frame ⌊*N*/2⌋ + 3
    of that sprite as its shadow. Nothing is drawn for the third, and it casts no shadow. No other foundation
    cell of the three shows a rubble picture or a rubble shadow. Each rubble picture is red-tinted
    like the ground and uses the tile palette's colours.
  - *Falsified if:* the rubble shows a house colour, ignores the tint, or is drawn in more than one
    cell; a rubble shadow has another image frame or sprite than stated; the third building casts a
    shadow; or a foundation cell other than the top-left one shows a shadow.
- **OV9, crates and other overlays (claims 1 and 8, rows 6 and 8).** Three runs, one at each
  detail level (low, medium and high, selected in the settings file before game start). A skirmish with
  crates on, on a map whose overlay data stream gives every cell data value 5; and a row of
  overlays of a type without any of the four flags at data values 0 to 3.
  - *Observable:* the body and shadow sprite frames of every crate that appears and of the row,
    matched against offline renders.
  - *Expected:* every crate shows sprite frame 0 with the shadow of sprite frame ⌊*N*/2⌋, and the
    row shows sprite frames 0 to 3 with shadows ⌊*N*/2⌋ to ⌊*N*/2⌋ + 3. The crate check is
    inconclusive if placing a crate resets the data value (the check cannot tell).
  - *Falsified if:* a crate shows any sprite frame other than 0 (sprite frame 5 in particular) or
    a shadow other than sprite frame ⌊*N*/2⌋, or the row shows other sprite frames.
- **OV10, veinhole markers (claims 1 and 8).** Two maps, one run each for parts A and B and four
  runs of part A's map for part C, all with loose sprites
  (named test files) for the two marker overlay types: every body image frame a full rectangle in
  a vivid colour, every shadow image frame a full rectangle. *Observable:* what is drawn in the
  nine marker cells and in the deck cells of part B, from a screenshot.
  - *Part A (open ground).* A map with a veinhole on flat open ground. Check in the base file that
    the two marker types have none of the four flags, so that without row 1 they would be drawn by
    row 8. *Expected:* no vivid
    rectangle in the nine cells beyond the veinhole object itself; each of the nine cells shows
    the rectangle as a shadow (darkened, S-0102 claim 26), from sprite frame ⌊*N*/2⌋ (data value
    0). *Falsified if* a vivid rectangle (a marker body) is drawn, or one of the nine cells shows
    no darkened rectangle.
  - *Part B (markers in deck cells).* A map with a high bridge running map west to map east, at
    least seven deck cells long, and a veinhole in the overlay list at (*x*, *y*), where the cells
    (*x* − 1, *y* − 1), (*x*, *y* − 1) and (*x* + 1, *y* − 1) are deck cells of the bridge away
    from its ends, and no other cell among the veinhole's neighbours is a deck cell. They come
    before the veinhole's cell in stream order, so the veinhole's markers replace their bridge
    overlays (claim 1). The map's overlay data stream gives those three cells
    and every other deck cell the value 0. *Expected:* the three cells show no vivid rectangle and
    no deck picture, so the deck has a gap there; each shows the darkened rectangle from sprite
    frame ⌊*N*/2⌋; the other deck cells show their deck pictures (claim 6). *Falsified if* one of
    the three cells shows a vivid rectangle (the marker drawn as a deck body by claim 1, row 2),
    shows a deck picture, or shows no darkened rectangle. Inconclusive if the rest of the bridge
    does not appear.
  - *Part C (markers with flags).* Part A's map, four runs, each from a fresh program start. The
    map's override section gives both marker types the tiberium flag (run 1), the wall flag (run 2)
    or the rubble flag (run 3), and in run 4 gives the crate flag to the type at position 178 only
    (a veinhole whose own type had the crate flag would not be placed, S-0009 claim 12).
    *Expected:* in every run no body is drawn in the nine cells (in run 4, in the eight cells
    around the veinhole): neither a vivid rectangle nor a picture from a tiberium's run. The
    darkened rectangles appear as in part A in runs 1, 2 and 4, and are missing in run 3 (claim 8,
    row 2). *Falsified if* any body is drawn in those cells, or the shadows differ from those
    stated.
- **OV11, smudges and plain terrain objects (claim 3).** One run. Two crater smudges and two trees: one
  smudge and one tree with all their cells inside a red building light, the others with all their
  cells outside every light. *Observable:* the colours of the smudges and the trees and of the
  ground beside each. *Expected:* the smudge and the tree inside the light take the red tint
  and the brightness of the ground beside them; the ones outside show the neutral colours.
  *Falsified if* a smudge or a tree ignores the light or the tint while the ground beside it shows
  them.
- **OV12, image frames beyond the sprite (claim 9).** One run per part. *Observable:* the body and shadow each test
  cell shows, matched against offline renders, and whether the game stops. Part A: two flat ore cells whose stored type is the
  type at the first position *P* of a tiberium's run, at coordinates with *x*·*y* ≡ 0 (mod 12), so
  that the body is drawn from the stored type as well (claim 4). *N* is the frame count of that
  type's sprite. The first cell has data value *N*, the second ⌊*N*/2⌋. *Expected:* the first cell
  shows nothing, neither body nor shadow (its shadow image frame *N* + ⌊*N*/2⌋ is beyond the
  sprite too). The second shows sprite frame ⌊*N*/2⌋ (a shadow picture) as its body, drawn with
  colours like any body. Its shadow image frame is 2⌊*N*/2⌋: if *N* is even that is *N*, and no
  shadow is drawn; if *N* is odd it is *N* − 1, and that picture darkens the screen. *Falsified if*
  the first cell shows any ore or shadow, the second shows another body, or its shadow does not
  follow the parity of *N* as stated. Part B (an image frame with no picture): a loose copy of that
  sprite (named test file) in which one body image frame *e* is stored with no picture, and a cell
  as above with data value *e*. *Expected:* the game plays on, the cell shows no body, and its
  shadow (sprite frame *e* + ⌊*N*/2⌋) is drawn as usual. *Falsified if* the game stops or the
  shadow is missing.
- **OV13, image numbers and the kept sloped run (claim 10).** Part A: one run. Part B: one
  session of two games. Part C: one test run and one control run. Each part starts with a fresh program start. A map's override section changes a tiberium only with a tiberium list entry keyed with that
  tiberium's existing index (S-0004 claim 10).
  - *Observable:* the body each test cell shows, matched against offline renders, and whether the
    game stops.
  - *Part A, one game.* Choose a tiberium *X* that the base rules give an image number other than 2
    and −1 and that comes, in the tiberium list, before every tiberium that holds the positions 27
    to 38 (check the base file). The map's override section gives *X* image number 2. Cells of the
    types at positions 27 to 38, data value 0: some flat, some on ramps of slope shape index 1 to 4.
    Check first in the stock files that the types at positions 39 to 46 have sprites in the test
    theater; if they have none, only "no stop" is observed. *Expected:* no stop; a flat cell shows
    sprite frame 0 of the type at 27 + ((*x*·*y*) mod 12), and a ramp cell sprite frame 0 of the
    type at 39 + 2(*s* − 1) + ((*x*·*y*) mod 2).
  - *Part B, two games.* *G* is the first tiberium, in the tiberium list, that the base rules give
    image number 2 (check the base file). If no tiberium has image number 2, a loose rules copy as
    in OV4 part C gives it to a tiberium, and *G* is that tiberium. Either way, no tiberium before
    *G* in the list holds positions 27 to 38, so *G* owns them in game 2 (Terms, "the tiberium of an
    overlay"). Game 1: a map whose override section gives *G* image number 3. Game 2, in the same
    session: a map without an override section, with *G*'s overlays (positions 27 to 38) on
    ramps. *Expected:* game 2 does not stop, and the ramp cells show the
    types at positions 39 to 46 as in part A. The control is OV4 part C, which stops.
  - *Part C, the value −1.* A map whose override section gives image number −1 to a tiberium that
    the base rules give image number 2, 3 or 4 (check the base file), with cells on flat ground.
    Read as an ordinary value, −1 would move it to the run at 102 and change the pictures.
    *Expected:* those cells look exactly as in a control run without the override section.
  - *Falsified if:* part A or game 2 of part B stops, or their ramp cells show any positions other
    than 39 to 46; or the cells of part C differ from the control.
- **OV14, a tiberium without an image number (claim 12).** Each run starts with a fresh program
  start, with a loose copy of the rules file (named test file) in which a tiberium *Y* that is not
  first in the tiberium list has no image number key. Run 1: a map with cells of an overlay type
  whose list position lies only in the run of a tiberium listed after *Y* (check the runs against
  the base file). Run 2: the same rules copy, a map whose tiberium cells all lie in the run of the
  first tiberium of the list. Control: run 1's map with the stock rules.
  - *Observable:* whether the game stops, and when, and what the tiberium cells show.
  - *Expected:* run 1 stops no later than the first drawing of such a cell; run 2 and the control
    play on and draw every cell.
  - *Falsified if:* run 1 draws such a cell and plays on, or run 2 stops.
- **OV15, overlay types with several flags (claims 1 and 8).** One run per part, each from a
  fresh program start, because the override section's game-wide values carry over between games
  (S-0006 claim 10). Each part uses a temperate map with a red map tint of 0.50 set by the map, and
  a local player whose house colour is not red. In each part the map's override section adds
  flags to one overlay type, as stated. In parts A to C the test cells lie on flat ground with
  nothing next to them.
  *Observable:* in every part, the body and shadow sprite frames of each test cell, matched
  against offline renders, and their colours compared with the ground beside them and with the
  local player's 53-shade house-colour palette. Each part is inconclusive if its test overlays do
  not appear (an overlay whose type has the wall flag can refuse a cell, and the vein rebuild can
  remove vein cells, S-0009 claim 14).
  - *Part A, tiberium and wall.* The override gives the wall flag to an ore overlay type that lies
    in a tiberium's run at position *P* + 5 (check the base file). Cells of that type from the
    overlay list at coordinates with (*x*·*y*) mod 12 = 0, and an overlay data stream that gives
    them data value 3. *Expected:* each shows sprite frame 3 of the type at *P* (claim 4), in the
    start-up resource table's colours without the red tint, with the shadow of its own type, sprite
    frame 3 + ⌊*N*/2⌋ (claim 1, row 3; claim 8). *Falsified if* a cell shows the sprite of the type
    at *P* + 5, the local player's house colours or the red tint.
  - *Part B, tiberium and rubble.* As part A, but the override gives that ore type the rubble flag
    instead. *Expected:* the bodies as in part A, and no shadow (claim 8, row 2). *Falsified if* a
    cell draws no body or a body other than part A's, or casts a shadow.
  - *Part C, wall and rubble.* The override gives the wall flag and the rubble flag to an overlay
    type with none of the four flags. Cells of that type from the overlay list, with an overlay
    data stream that gives them data value 3. *Expected:* each shows sprite frame 3 of its own type
    in the local player's house colours, without the red tint (claim 1, row 4), and no shadow
    (claim 8, row 2). *Falsified if* a cell draws no body, or shows the tile palette's colours or
    the red tint.
  - *Part D, the vein type and tiberium.* The override gives the tiberium flag to the vein overlay
    type (position 126); check against the base file that position 126 lies in no tiberium's run.
    The map holds vein seeds (S-0009 claim 14) on flat ground. *Expected:* after loading, no vein
    cell shows a picture of the vein type's sprite; each shows a picture from the run of the first
    tiberium of the list (claim 4's fallback), or nothing where its data value is beyond that
    sprite (claim 9), in colours without the red tint. *Falsified if* a vein cell shows a picture
    of the vein type's sprite.
  - *Part E, the vein type and wall.* As part D, but the override gives the vein type the wall
    flag instead. *Expected:* the vein cells show pictures of the vein type's sprite in the local
    player's house colours, without the red tint (claim 1, row 4). Only the colours are read,
    because the wall flag can change the data values. *Falsified if* the vein cells show the colours
    of the temperate theater palette instead.
  - *Part F, the vein type and rubble.* As part D, but the override gives the vein type the rubble
    flag instead. *Expected:* the vein cells look as without the override (claim 1, row 5), and cast
    no shadow (claim 8, row 2). *Falsified if* the vein cells draw no body, or cast a shadow.
  - *Parts G, H and I, the crate type.* A skirmish with crates on. The override adds a flag to the
    overlay type of the crates that appear on land during play (check the base file): the tiberium
    flag (part G), the wall flag (part H) or the rubble flag (part I). Every crate that appears on
    land is observed, and each part runs until at least one has appeared. *Expected:* in part G,
    no crate shows a picture of the crate sprite; each shows a picture from the run of the first
    tiberium of the list (if the crate type lies in no run; check the base file), or nothing where
    its data value is beyond that sprite (claim 9), without the red tint (claim 1, row 3). In part
    H every crate that is drawn uses the local player's house colours, without the red tint (claim
    1, row 4; only colours are read). In part I every crate shows sprite frame 0 of the crate
    sprite, red-tinted like the ground, with the shadow of sprite frame ⌊*N*/2⌋ (claim 1, row 6,
    and claim 8, row 1). *Falsified if:* in part G a crate shows the crate sprite; in part H a crate
    shows the red tint or the tile palette's colours; in part I a crate draws no body or casts no
    shadow.
  - *Part J, the rubble overlay type and the crate flag.* The override gives the crate flag to the
    overlay type that destroyed buildings leave as rubble (check the base file), and a loose sprite
    (named test file) for that type has every body image frame a full rectangle and every shadow
    image frame a full rectangle. Destroy two buildings that leave rubble: one with a separate
    rubble sprite, one without but with at least 8 body image frames. No unit enters the rubble
    cells. *Expected:* every foundation cell of both buildings shows the rectangle as its body,
    red-tinted like the ground (sprite frame 0, claim 1, row 6), and the darkened rectangle from
    sprite frame ⌊*N*/2⌋ as its shadow (claim 8, row 1). Neither the rubble sprite nor image frame 3
    of the second building's body sprite appears. *Falsified if* a rubble picture or a rubble
    shadow appears at a top-left foundation cell, or a foundation cell lacks the rectangle or its
    shadow.

- **OV16, depth of overlay shadows by kind (claim 8; S-0102 claims 13 and 27, which D35 measures
  for ore, an upright type and a deck cell).** Setup and reading as in S-0102 D35: a flat test map at
  level 0 and a copy at level 2; each part compares a ground pixel with the same pixel in a control
  game without the overlay that casts the shadow (value *v* per channel); a part is not measurable
  if its overlay's sprite has no shadow image frames or the shadow does not reach the pixels named.
  Every part puts a tree (a terrain object placed by the map) so that its shadow overlaps the
  overlay shadow on bare ground. One test run and one control run per kind and level, repeated
  at detail low, medium and high (selected in the settings file before game start): 36 runs total
  for three kinds, two height levels, three detail levels and their controls. Record matching
  screenshots once the named objects are present and stationary; destroy the rubble-producing
  building with scripted input before that capture.
  - *Part A (wall).* A wall overlay placed by the map's overlay list. *Expected:* ⌊*v*/4⌋ in the
    overlap, as in D35 (a): the tree's shadow darkens it a second time. With the −15 of an
    upright body the overlay shadow would be farther than the tree's and the overlap would be ⌊*v*/2⌋.
  - *Part B (veinhole marker).* A veinhole as in OV10 part A with loose marker sprites whose shadow
    image frames are full rectangles; a tree whose shadow overlaps a marker cell's rectangle.
    *Expected:* ⌊*v*/4⌋ in the overlap.
  - *Part C (rubble).* A destroyed building that leaves rubble (OV8's first building), the tree beside
    the rubble cell. *Expected:* ⌊*v*/4⌋ in the overlap.
  - *Part D (level 2).* Parts A to C on the level-2 map. *Expected:* the same values (the bias follows
    the cell's level alone).
  *Falsified if* any part gives ⌊*v*/2⌋ in the overlap or another value.
- **OV17, which positions make deck cells (claims 1 and 6).** One run, from a fresh program start,
  on a temperate map with neutral test lighting. A loose copy of the rules file (named test
  file) reorders the overlay type list (S-0004) so that four overlay types *A*, *B*, *C* and *D*
  that have none of the four flags and whose sprites have at least four body image frames
  with distinguishable pictures (check the base file) sit at positions 24, 25, 237 and 238, and the
  four bridge types take the positions that *A* to *D* held in the stock list, *X*₁ to *X*₄. A fifth
  such type *E* keeps its stock position, which is no bridge position, as the control. The map's overlay list is written for the reordered list. Rows
  of four by four cells on flat ground, all at data value 0, so that every combination of *x* mod 4
  and *y* mod 4 occurs: one block per position 24, 25, 237, 238, one block per *X*₁ to *X*₄ and
  one block of *E*.
  - *Observable:* the sprite frame each cell shows as its body, matched against offline renders of
    the sprites, and the cells' colours compared with the terrain beside them.
  - *Expected:* in each of the four blocks at positions 24, 25, 237 and 238, cell (*x*, *y*)
    shows sprite frame *T*(*x* mod 4, *y* mod 4) of the type stored there (claim 6) in the tile
    palette's colours. In each block of a moved bridge type at *X*<sub>*i*</sub> every cell shows
    sprite frame 0. The block of *E* shows sprite frame 0 in every cell.
  - *Falsified if:* a block at one of the four positions shows sprite frame 0 in every cell, a moved
    bridge type shows the 4 × 4 pattern, or the block of *E* shows the pattern. The run is
    inconclusive if the game refuses the reordered list or a block does not appear.

- **OV18, ore and gems under cover and glow (claim 1).** Use the common skirmish setup with
  one bystander computer player separated from the observed ore and gem cells. Unrelated mods
  are off. No harvester enters the observed area; any change of a test cell's
  overlay type or data value voids the run. Choose flat test cells with known body sprites and
  non-transparent body pixels inside their height-0 diamonds, outside shadows and other objects.
  Named test shroud sprite: all 47 pieces are uniformly index *v* inside their cell's diamond
  and index 254 outside it. Make three no-glow runs, *v* = 0, 64 and 127, plus a reference run
  with every piece entirely index 254. Then repeat the three runs with one stock glow-casting
  building whose named test glow image has bands *g* = 0, 127 and 254 across both resource
  pictures. No second glow overlaps the samples. Remove named test files afterwards.
  - *Procedure and tool:* take lossless 800 × 600 screenshots of the stationary test cells in
    each run, game speed index 4. Compare known body-pixel palette indices with packed colours
    of the start-up resource table. One valid run per case, seven runs total.
  - *Observable:* ore and gem pixel colours at each sampled cover value and glow band, and in
    the reference. Record ambiguous palette-row matches as inconclusive.
  - *Expected:* without glow, *a* = *v*; with glow,
    *a* = min(255, ⌊*v*·*g*/127⌋). Each body pixel uses row
    min(52, ⌊26·*a*/127⌋), the S-0104 claim 15 mapping at brightness 1000 ‰ with 53 rows.
    These are non-negative integer calculations with division rounded down. The transparent
    reference uses *a* = 127, row 26. Cover value 0 stays black under every glow band; value
    64 is darkened without glow but brightened by band 254. The resource palette remains
    untinted in all cases.
  - *Falsified if:* an unambiguous sampled pixel differs from its predicted packed colour,
    cover leaves all ore or gem pixels at row 26, a glow restores a cover-value-0 pixel,
    or the result follows a glow before cover rather than a glow scaling the covered value.
    This tests shroud and glow only, not deferred fog updates or multiple-glow ordering.

## Open questions

- **Deck cells beyond the sampled positions.** OV17 samples a few positions besides the four
  bridge positions; that no other position makes deck cells is not checked position by position.
- **Missing start-up palette files.** What happens when the temperate theater palette file or the
  snow unit palette file is missing at program start is not established.
- **Veins and crates in deck cells.** Whether vein growth during play, the vein rebuild at load
  under a bridge type that allows veins, or a crate that appears during play can put an overlay
  in a deck cell is not established. Claim 1 states what such a cell would draw (row 2), but no
  scenario of this spec produces one.
- **Gates and towers that leave the map standing.** Claim 7's departure rows are stated and
  measured for a destroyed building. For a gate or tower that leaves with health above 0 (sold,
  for example), the same cells are recomputed, but the recomputation may still see the building in
  its cells, in which case the walls keep their bits toward it. Not established.
- **Depth of ore and veins on slopes.** On sloped cells ore and veins pass a slope-specific depth
  shape to the depth test. That is a depth matter for S-0102 and is not covered here.
