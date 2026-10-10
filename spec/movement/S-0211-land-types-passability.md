---
id: S-0211
title: Land types and passability
topic: movement
revision: 7
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-10
measurements: []
requests: []
supersedes: []
---

# S-0211: Land types and passability

## Summary

How a cell's land type follows from its tile, overlay and neighbours, the land-type table of speed
multipliers and buildable flags, and the land-type and overlay rules that decide which cells a mobile
object may enter, which movement zones may cross a cell (its passability category) and where
buildings may be placed.

## Terms

*Cell*, *overlay*, *slope*, *height level*, *high bridge*, *bridge deck*,
*movement zone*, *locomotor*, *rules layer*, *rules database*, *map layer*, *engine constant*,
*absent-key default*, *departure*, *importer warning*, *field*, *theater*, *tileset*, *sub-tile*,
*terrain object*, *building*, *registry*, *registration index*, *house*, *game start*, *session*,
*unspecified*, *measurement setup* and *recorder* are used as in the glossary. From other specs, all
drafts of the same batch: *track*, *speed fraction*, *target fraction*, *slowdown distance*, the
*acceleration flag*, the *passive flag* and the braking floor *F*₀.₃ are S-0207's (ground vehicle
movement). *Connectivity* (regions and reach regions), the *gate* (S-0212 claim 6) and the *search layer* of a
bridge cell are S-0212's (pathfinding). *Tile*, *tile number*, the tile's size *w* × *h* in cells, *role*, *role
range* and the *water role* are S-0010's (theater and tileset semantics), and *slope shape* is
S-0106's (slope shapes, terrain surface height and voxel tilt). The *round-toward-zero mode* is S-0205's (number domains).
The crusher flag's effect in play is S-0210's (crushing).

- **Mobile type, mobile object.** An infantry, vehicle or aircraft type, and an object of such a type.
- **Land type.** One of twelve categories of ground. The spec numbers them as follows (the numbers
  are labels and carry no behaviour of their own; scenario L11 uses them only to encode answers):
  0 *clear*, 1 *road*, 2 *water*, 3 *rock*, 4 *wall*, 5 *tiberium* (resource ground, used for ore
  and gems), 6 *beach*, 7 *rough*, 8 *ice*, 9 *railroad*, 10 *tunnel*, 11 *weeds*. Each cell
  has exactly one land type at any time (claims 5 to 8).
- **Locomotion class.** The column of the land-type table that a mobile type uses. This is the term
  S-0006 uses, and this spec owns the list. The spec numbers the classes as follows (labels only):
  0 *foot*, 1 *track*, 2 *wheel*, 3 *hover*, 4 *winged*, 5 *float*, 6 *amphibious*,
  7 *float-on-beach*. A type may also have *no* class (claim 3). The locomotion class is not the same
  thing as the *locomotor* (how an object moves) or the *movement zone* (where the pathfinder lets
  it go). A type has all three, chosen by three separate fields.
- **Land multiplier** *M*(*ℓ*, *c*). A factor for land type *ℓ* and locomotion class *c*. It is
  stored in single precision (claim 2). S-0207 uses it as a speed factor. Here a multiplier of exactly
  zero means the class cannot enter that land (claim 10).
- **Buildable flag** *B*(*ℓ*): whether buildings may be placed on land type *ℓ* (claim 13).
- **Crusher flag.** A vehicle-type field (S-0210 owns what it does in play). This spec uses it only
  to choose a vehicle's default locomotion class (claim 3).
- **Naval flag.** A building-type field that marks buildings placed on water (claim 13).
- **Tile land code.** An integer that the tile data gives for each cell of a tile (S-0010 calls it
  the terrain code). Its meaning is given by a fixed table (claim 6).
- **Slope shape index.** The slope shape that a cell's tile gives that cell, an integer. 0 is flat.
  1 to 4 are the four simple slopes, where one side of the cell is raised by one level. 5 and above
  are the other shapes (corners and steep slopes).
- **Overlay flags.** Fields of an overlay type that this spec uses: its *overlay land type* (a land
  type, or none), the *own land type* flag (when set, the overlay's land type replaces the tile's),
  the *wall* flag, the *tiberium* flag (it is an ore or gem overlay), the *crushable* flag, the
  *rock* flag and the *rubble* flag. The wall flag and the land type *wall* are separate fields: a
  wall overlay normally has both, but claim 5 reads only the land type and claims 10 and 11 read only
  the flag.
- **Laser-fence flag, firestorm-wall flag.** Two building-type fields that mark laser-fence sections
  and firestorm walls (claim 11). A house's *firestorm* is a house state, on or off, that firestorm
  walls depend on. Its rules belong to the structures and superweapon specs.
- **Active laser-fence section.** A building of a laser-fence type that is in neither of its two
  *inactive* states. A newly placed section starts in one of the two, chosen by the orientation of
  the fence. Which events switch a section between states belongs to the structures specs.
- **Occupation pattern.** An integer field of a terrain-object type, given once for the snow theater
  and once for the other theaters (Import mapping). The occupancy specs own what it means for
  occupancy. This spec uses only whether it equals 7.
- **Ground level of a cell.** The cell's height level. A high-bridge cell also has a bridge deck
  4 levels above it.
- **The passability category.** A label from 0 to 7 (carrying no behaviour of its own, like the land-type
  labels) for what a cell lets through, computed from its land
  type, overlay and contents (claim 11): 0 *open*, 1 *crushable*, 2 *wall-like*, 3 *beach*,
  4 *water*, 5 *partial obstacle*, 6 *blocked*, 7 *off-map*.
- **Zone table.** The fixed table that says, for each movement zone and each passability category,
  whether that zone may cross a cell of that category (claim 12).
- **Land recompute.** The recomputation of one cell's land type (claims 5 to 8) together with its
  passability category (claim 11). S-0010 calls the same event *terrain recalculation*. Claim 5
  lists what triggers it.
- **Cliff-back setting.** A game-wide integer field (claim 8).
- **Behind a cliff.** A property of a cell at one land recompute (claim 8).
- **Restricted land type.** An optional vehicle field naming one land type. A vehicle with one may
  only enter cells of that land type, apart from the exceptions of claim 14.
- **Usable area.** The part of the map inside the local area that objects may occupy. It belongs to
  S-0009 and S-0010. This spec uses it only for the off-map category.

## Behaviour

### The land-type table

1. **Twelve land types, eight locomotion classes.** The original has exactly the twelve land types
   and the eight locomotion classes of Terms, fixed in the program. Data adds or removes none. The
   rules data gives one section per land type. A section's absence never removes a land type.
   - Backing: `unmeasured` (that the set is closed has no observable of its own; the effects of each
     land type and class are covered by the claims below, each of which is measurable)
   - Confidence: high

2. **Land multipliers and the buildable flag.** At every game start, each rules layer that
   contains a land type's section sets that land type's whole row:
   - (a) For each of the seven classes foot, track, wheel, hover, float, amphibious and
     float-on-beach, *M* is the class's key value capped at 1.0, as the formula for *M* gives
     (Formulas; the value form is S-0003's, claims 6 and 7). Values not above 1.0 are kept, zero and
     negative values included. A missing key gives 1.0.
   - (b) *M*(*ℓ*, winged) is set to 1.0 whatever the section says: a key for the winged class has no effect.
   - (c) *B*(*ℓ*) is the section's buildable key, a boolean. A missing key gives *no*.

   So a rules layer that gives only one key of a land type's section resets the rest of that row to 1.0
   and *no*. This is the land-type case of S-0006 claim 5, and this claim owns the 1.0 cap and the
   fixed winged value that S-0006 defers here.

   A land type whose section no rules layer has given since program start has the start-up row:
   *M* = 0 for all eight classes (winged included) and *B* = no. Data that gives every land-type
   section in the base rules never shows it. Our engine reproduces the start-up row and gives an
   importer warning for each land-type section that the base rules lack.
   - Backing: `measurable`, by part:

     | Part | Scenarios |
     |---|---|
     | (a) the cap | L1 |
     | (a) negative values kept (they allow entry) | L2, map 3 |
     | (a) a missing key gives 1.0 | L12 part C (and S-0006 scenario D2) |
     | (b) the fixed winged value | L13 |
     | (c) the buildable key and its reset to *no* | L7 |
     | the start-up row | L12 parts A and B |

   - Confidence: high for parts (a) to (c); medium for the start-up row (inferred, not observed)

3. **A mobile type's locomotion class.** A mobile type has one locomotion class, chosen by name from
   the list in Terms (letter case is ignored). Once a type's own section has been read in a rules
   layer, its class in the original is:

   | The section's class key | Class the type held before | Class afterwards |
   |---|---|---|
   | names a class | any | that class |
   | gives a value that names no class (a misspelling, say) | any | *no class*, for a vehicle type too |
   | is empty or missing | a class | unchanged |
   | is empty or missing | *no class* | vehicle type: *track* if its crusher flag is set, *wheel* otherwise; any other type: *no class* |

   Before its first section is read, an infantry type has class *foot*, an aircraft type *winged*
   and a vehicle type *no class* (so the last row gives every vehicle type a class unless its key
   names no class). The last row uses the crusher flag as the type holds it once that rules layer's
   section has been read. The last row applies only while the type has no class, so once it has
   given a vehicle type a class, a later rules layer that changes only the crusher flag leaves that
   class unchanged (third row).

   **Departure.** In the original, the land multiplier of a type with no class is unspecified. Our
   engine replaces the second row: a value that names no class is treated as if the key were absent
   (the last two rows apply), and the importer warns.
   - Backing: `measurable` (scenario L9: part A for the vehicle default, part B for the infantry
     default, part C for a later crusher flag leaving the class unchanged); the aircraft default is `unmeasured`, because the winged class is visible only through
     claim 10's winged row and the winged multiplier, and an aircraft's own locomotor, which this
     spec does not cover, decides where it goes; the unrecognised name is a departure
   - Confidence: high

4. **Name parsing of land types and zones.** An overlay's land type and a vehicle's restricted land
   type are read by name from the twelve land-type names, with letter case ignored. The text that
   Import mapping gives for *none*, or any name not in the list, gives *none*. For a restricted land type, *none* means no
   restriction, so a misspelt restricted land type leaves the vehicle unrestricted (our engine does
   the same, and the importer warns about any name not in the list). A movement zone
   is read by name from the thirteen zone names in Import mapping, with letter case ignored. An
   unrecognised name gives *no zone*.

   **Departure.** In the original, the land type *none* of an overlay type gives an unspecified
   result wherever it is used: as the cell's land type (claim 5, rows 1, 2 and 4) and in category
   rule 2 (claim 11). The original turns only *clear*, not *none*, into *tiberium* for ore and gem
   overlay types (claim 5). Our engine treats an overlay type whose land type is *none* exactly as one
   whose land type is *clear*, that rule included, and the importer warns. So an ore or gem
   overlay type written with *none* gets the land type *tiberium*, like one whose key is absent, and
   every other overlay type written with *none* gets *clear*. Claims 5 and 11 then apply to it
   unchanged. An unrecognised zone name is treated as an absent key, which keeps the zone the type
   held before, and the importer warns.
   - Backing: `measurable` (scenario L17 for the land-type names of restricted vehicles and
     overlays; scenario L18 run D2 for the letter case of zone names); the two departures are
     `unmeasured`, because they replace results that are unspecified in the original
   - Confidence: high

### The land type of a cell

5. **Land type from tile and overlay.** A cell's land type is given by the first row of this table
   whose condition holds. The last column says when the cliff-back rule of claim 8 turns that land
   type into *rock* (*the clear set* there means the land types *clear*, *water*, *beach* and *ice*).

   | Row | The cell | Land type | The cell's tile | Turned into *rock* by claim 8 |
   |---|---|---|---|---|
   | 1 | has an overlay whose land type is *wall* or *railroad*, or whose own-land-type flag is set | the overlay's land type | kept, whatever the tile and sub-tile | always, whatever the land type |
   | 2 | has no tile, or a tile number at or beyond the theater's tile count | the overlay's land type if the cell has an overlay, otherwise *clear* | a tile number at or beyond the count is dropped: the cell holds no tile from then on | when the land type is in the clear set |
   | 3 | has a tile that is unusable for its sub-tile (claim 6) | *clear*, whatever its overlay | dropped, as in row 2 | when the land type is in the clear set (always so for row 3, whose land type is *clear*) |
   | 4 | has an overlay with the tiberium flag (ore or gems) and slope shape index 0 to 4 | the overlay's land type, with *clear* read as *tiberium* | unchanged | when the land type is in the clear set |
   | 5 | none of the above | the land type of the tile land code (claim 6); any other overlay, whatever its land type, leaves the land type to the tile | unchanged | when the land type is in the clear set |

   An ore or gem overlay type whose land type is *clear* (an absent key gives *clear*) is given
   *tiberium* as its land type when its section is read. In our engine this includes one whose land
   type is written as *none* (claim 4's departure). So ore and gem cells normally have land type
   *tiberium*, and row 4's reading of *clear* as *tiberium* makes no further difference.

   A cell whose tile was dropped under row 3 meets row 2 at its next land recompute, so it then
   takes its overlay's land type if it has an overlay. At map load the next recompute is the second
   pass of S-0010 claim 12, so after loading such a cell has its overlay's land type, and row 3's
   *clear* lasts only between the two passes. Claim 8 says when the last column applies. S-0010
   claim 8 states the same rules from the terrain side, and the two must agree.

   **When it is recomputed.** This list is authoritative for every spec (S-0010 calls the same event
   *terrain recalculation* and cites it). A land recompute of a cell happens on these events, and
   the recompute sees the cell's contents after the change:
   - map load: every cell, twice (S-0010 claim 12 gives the two passes). Any whole-map change of the
     terrain at game start recomputes every cell again.
   - an overlay is placed, removed or changed on the cell: ore grows, spreads or is harvested, a
     wall is damaged, sold or destroyed, a bridge is destroyed or repaired, a destroyed building
     leaves rubble, or an animation replaces the overlay.
   - the cell's tile, sub-tile or ground level is changed, for example when a destroyable cliff or a
     bridge is destroyed or repaired. Some of these changes also recompute the neighbouring cells,
     and the specs of those events say which.
   - an infantry unit, vehicle, aircraft, building or terrain object is put into the cell or taken
     out of it while it stands on the ground. For a mobile object this includes every ordinary move
     from one cell to the next, so every cell a unit enters or leaves is recomputed, and for a
     building or terrain object it covers every cell of its footprint.

   Nothing else triggers it: in particular a change of a laser-fence section's state (Terms), of a
   house's firestorm or of the owner of a building does not. The specs that own the events above
   (bridges, walls, cliffs, ore, animations and the structures specs) say when each happens in play,
   but not whether it recomputes the cell: that is stated here.

   - Backing, by part:

     | Part | Backing |
     |---|---|
     | Row 1 | `measurable` (scenarios L4 map 3 type 2, L6 low bridge, L5 run E) |
     | Row 2, with and without an overlay | `measurable` (scenario L16, cells A and B) |
     | Row 3, as seen after loading (with an overlay, the overlay's land type; without one, *clear*) | `measurable` (scenario L16, cells C and D) |
     | Row 3's *clear* for a cell with an overlay, before the next recompute | `unmeasured`: at map load the second pass replaces it before play begins, and no event in play makes a usable tile unusable |
     | Row 4 | `measurable` (scenario L3) |
     | Row 5 | `measurable` (scenarios L4 map 3 type 1, L5, L11, L16 cell E) |
     | Recompute at map load | `measurable` (every scenario that reads a land type after loading) |
     | Recompute when an overlay is removed | `measurable` (scenarios L3 part A and L6) |
     | Recompute when a terrain object is removed | `measurable` (scenario L14) |
     | Recompute when a building is placed or removed | `unmeasured`: a building changes no land type (claim 5 reads only the tile, the overlay and the neighbours' levels) and, as scenario L20 shows for the three kinds of building that can be set up, no category (claim 11), so the recompute has no visible effect in this spec |
     | Recompute when an infantry unit or vehicle moves | `measurable` (S-0010 scenario R1, part E, which uses a named test theater file with the rough-transition role unset: the first unit to enter or leave such a cell makes the recompute visible as a change of the cell's tile; the category cannot show it, because it never depends on units, claim 11) |

   - Confidence: high for rows 1, 4 and 5; medium for rows 2 and 3 (the map's tile numbers and
     sub-tiles must reach the recompute unchanged); high for the recompute at map load, on overlay
     changes, on terrain-object and building changes and on the moves of vehicles and infantry,
     and medium for aircraft on the ground and for whether the list is complete

6. **Tile land codes.** The tile land code *k* maps to a land type by this fixed table:

   | *k* | 0 | 1–4 | 5 | 6 | 7, 8 | 9 | 10 | 11, 12 | 13 | 14 | 15 |
   |---|---|---|---|---|---|---|---|---|---|---|---|
   | land type | clear | ice | tunnel | railroad | rock | water | beach | road | clear | rough | rock |

   A cell's tile is *unusable for its sub-tile* when the tile has no image data at all, when the
   sub-tile number is *w* × *h* or more for that tile, or when the tile has no image data for that
   sub-tile (an empty cell of the tile). Such a cell falls under row 3 of claim 5.

   **Departure.** The original has no entries for codes below 0 or above 15, and a cell with such a
   code gets an unspecified land type. Our engine uses *clear* and the importer warns.
   - Backing: `measurable` (scenario L11 for the codes that occur on flat cells of the test theater,
     scenario L5 run B for code 0 at the foot of a cliff, and scenario L16 cells C and D for the
     unusable sub-tile: a sub-tile number beyond the tile's cells, and an empty cell of a
     multi-cell tile); the out-of-range codes are a departure; a tile with no image data at all is
     `unmeasured` here, because no scenario of this spec produces such a tile (S-0010 claim 7 and its
     scenario C1 cover the tile itself)
   - Confidence: high

7. **Ore and gems on steep slopes are removed.** When a cell with an ore or gem overlay is
   recomputed and its slope shape index is 5 or more, the overlay is removed from the cell, and the
   cell takes the tile's land type. Ore placed by a map on such cells therefore never appears. A
   variant applies to an overlay that has both the tiberium flag and one of the row-1 properties
   of claim 5 (land type *wall* or *railroad*, or the own-land-type flag): it is removed whenever
   the slope shape index is not 0, but the cell keeps the overlay's land type until its next land
   recompute (claim 5 lists the events that trigger one, for example a unit entering or leaving the
   cell).
   - Backing: `measurable` (scenario L3 part A for slope shapes 5 and above, part B for the
     variant's removal); the variant's leftover land type is `unmeasured`, because the next object to
     enter the cell recomputes it, and for overlays placed by the map the second pass at map load
     (S-0010 claim 12) already does, so no mover can observe it reliably
   - Confidence: high for slope shapes 5 and above; medium for the row-1 variant (it needs modified
     overlay data)

8. **Land behind cliffs.** Let *r* be the cliff-back setting and *h* the cell's ground level at the
   moment of the recompute. The cliff-back rule is *active* when *r* mod 256 = 2, where mod gives a
   result from 0 to 255 (Formulas). So 258 and −254 act as 2, while 1, 3 and every other value whose
   remainder modulo 256 is not 2 make the rule inactive. A cell is *behind a cliff* when the rule is
   active and at least one of the six cells (*x*, *y* − 1), (*x* + 1, *y* − 1), (*x* − 1, *y*),
   (*x* − 1, *y* + 1), (*x* + 1, *y* + 1) and (*x* + 2, *y* + 2) has a ground level of *h* + 4 or
   more, compared as integers. Only the ground levels of those six cells count: their land types,
   tiles and objects do not. While the rule is inactive no cell is behind a cliff, so the rule
   changes nothing.

   The rule is part of every land recompute. The land type that a recompute leaves on a cell is the
   one claim 5 gives, except that a cell behind a cliff has *rock* instead wherever claim 5's last
   column says so. The rule changes only the land type, never the tile or the slope shape.
   Claim 5's last column means:
   - **Row 1** (overlay land type *wall* or *railroad*, or own-land-type flag set): always, whatever
     the land type, so a wall, a railroad or a road bridge behind a cliff becomes *rock*.
   - **Rows 2 to 5:** only when the land type is in the clear set (*clear*, *water*, *beach* or
     *ice*). A *road* or *tunnel* cell, a *rough* cell, a *rock* cell or a *tiberium* cell keeps its
     land type. Row 3 always gives *clear*, so a row-3 cell behind a cliff always becomes *rock*,
     and the quirk below does not affect it.

   The tests use the six neighbours' ground levels as they stand when the recompute runs, so a cell
   next to a cliff that is raised or lowered later changes only at its own next recompute (claim 5).

   **Quirk `quirks.cliff_back_case_sets`.** Default (original): row 1 turns into *rock* whatever
   its land type, as above. Off: row 1 uses the clear set like rows 2 to 5, so a wall, railroad
   or road bridge (row 1) behind a cliff keeps its land type unless that land type is in the clear
   set. Rows 2 to 5 behave identically in both settings. S-0010 claim 9 cites this claim and this
   switch and adds nothing.
   - Backing: `measurable` (scenario L5: runs A and B for the rule, run F for the neighbour set, runs C and D for the reduction
     modulo 256, run E for row 1's "always" and row 5's set, which leaves *road* unchanged)
   - Confidence: high for the neighbour set, the threshold, the reduction and the sets of rows 1, 3
     and 5 for *clear* and *road*; medium for *water*, *beach* and *ice*, which no scenario tests,
     and low for neighbours that lie off the map

### Entering a cell and the passability category

9. **Bridges.** A bridge cell's land type follows claim 5 like any other cell's. When a high
   bridge's overlay types have neither the own-land-type flag nor land type *wall* or *railroad*, a
   high-bridge cell keeps the land type of its ground (for example *water*), so a class that may
   enter that land passes under the deck. An object on a bridge deck is not subject to the cell's
   land type: claim 10 does not apply to it. S-0207 says that a vehicle drives on a deck as on
   *road*, and S-0212 treats the deck as its own search layer, so the deck's connectivity does not
   follow the category of the ground cell below. When a low bridge's overlay types have the
   own-land-type flag, row 1 of claim 5 gives a low-bridge cell the overlay's land type (for example
   *road*). When a bridge overlay is destroyed, the cell is recomputed (claim 5) and takes the land
   type of its tile again.
   - Backing: `measurable` (scenario L6)
   - Confidence: high for the land-type rules (they are claim 5's); medium for the deck exemption

10. **Entering a cell, by class.** Apart from occupancy by other objects, zones and heights (which
    belong to the occupancy and pathfinding specs), an object of locomotion class *c* may enter a cell
    of land type *ℓ* by the first of these cases that applies; that case alone decides:
    - *c* is winged: always. So a winged-class object enters a cell with a wall overlay, whatever its
      movement zone.
    - The object will stand on the cell's bridge deck: always, whatever the cell's land type.
    - The cell's overlay type has the wall flag (whatever its land type): only for movement zones
      *destroyer*, *amphibious-destroyer*, *infantry-destroyer* and *crusher-all*, and for *crusher*
      and *amphibious-crusher* when the wall's overlay type is crushable. For those zones, entry is
      decided as for a cell of land type *clear*: it is allowed when *M*(clear, *c*) ≠ 0. Every
      other zone may not enter.
    - Otherwise (no case above applies): when *M*(*ℓ*, *c*) ≠ 0. The comparison is made on the single-precision value, so
      any nonzero value, even a tiny or negative one, allows entry.
    - Backing: `measurable` (scenarios L2, L4 map 3 and L8; scenario L19 maps 2 and 1W for the wall
      case in every movement zone that the zone table lets reach the wall cell; scenario L13 part B
      for the precedence of the winged case over the wall case)
    - Confidence: high

11. **The passability category of a cell.** Each cell's passability category is recomputed at every
    land recompute (claim 5), from the cell as it is after the triggering change. It does not
    depend on the infantry or vehicles in the cell, so their moves, although they trigger a land
    recompute, never change a category. It is given by the first row of this table whose condition
    holds (*ℓ* is the cell's land type, *ℓ*ₒ the land type of the cell's overlay type):

    | Rule | Condition | Category |
    |---|---|---|
    | 1 | the cell lies outside the usable area | 7 *off-map* |
    | 2 | the cell's overlay type is crushable | 1 *crushable* |
    | 2 | the cell's overlay type has the wall flag | 2 *wall-like* |
    | 2 | the cell has an overlay and *M*(*ℓ*ₒ, wheel) = 0 | 6 *blocked* |
    | 2 | the cell's overlay type has the rock flag | 6 *blocked* |
    | 2 | the cell's overlay type has the rubble flag | 0 *open* |
    | 3 | *ℓ* is *water* | 4 *water* |
    | 3 | *ℓ* is *beach* | 3 *beach* |
    | 3 | *M*(*ℓ*, wheel) ≤ 0.01 (single precision compared with the double nearest to 0.01) | 6 *blocked* |
    | 4 | the cell holds a deciding object (rule 4 below) | 2, 5 or 6, as rule 4 gives |
    | 5 | none of the above | 0 *open* |

    So, in a cell inside the usable area, an overlay with the rubble flag that is not crushable, has
    neither the wall nor the rock flag, and whose own land type has a nonzero wheel multiplier puts
    the cell in category 0, even on water, on beach, or on land that rule 3 or 4 would otherwise
    decide.

    **Rule 4** (objects in the cell). Two kinds of object are *deciding objects*, and each gives a
    category:
    - A terrain object (such as a tree), by its type's occupation pattern for the current theater:
      the snow pattern in the snow theater, the temperate pattern in every other theater. The
      pattern 7 gives 2. Any other pattern gives 5.
    - A building of a type with the firestorm-wall flag whose owner's firestorm is on gives 6. In
      the original no event ever turns a house's firestorm on (every house starts with it off), so
      in play a firestorm wall is never a deciding object. This matters only if a later spec gives
      the firestorm a way to turn on.

    When all the deciding objects in a cell give the same category, that is the cell's category. A
    firestorm wall whose owner's firestorm is off, and every other building, an active laser-fence
    section included, has no effect on the category: a cell that rules 1 to 3 leave undecided and
    that holds no deciding object is in category 0 by rule 5, whatever buildings stand in it.

    When deciding objects that give different categories share a cell, the one listed first in the
    cell's object order (the occupancy specs own that order) gives the category. This tie-break is
    our engine's choice, because no observable case is known: that needs a
    terrain object and a firestorm wall whose owner's firestorm is on in one cell, which cannot
    happen in the original, or two terrain objects of different patterns in one cell, which nothing
    is known to cause.

    **Quirk `quirks.passability_laser_fence_lost`.** Default (original): laser-fence sections leave
    the category to rules 4 and 5, as above. Off: an active laser-fence section is a deciding object
    of rule 4 and gives 6, like a firestorm wall whose owner's firestorm is on. Its effect is on
    connectivity (S-0212): with the default, the pathfinder treats a line of active fence sections as
    open ground and plans paths through it.

    The land recompute that a section's placement triggers sees the section in the inactive state
    it starts in. In the original a later switch
    between the active and inactive states is not itself a land recompute trigger. With the default
    this makes no difference, because the state never affects the category. With the quirk off, our
    engine also recomputes the cells of a section whenever it switches state, so the category always
    follows the section's current state.

    Rule 2 applies to the overlay's own land type even when claim 5 ignores it for the cell's land
    type. A cell whose overlay type has land type *rock* but neither the own-land-type flag nor the
    wall, crushable, rock or rubble flag keeps its tile's land type, but its category is 6 when
    *M*(rock, wheel) = 0.

    **Quirk `quirks.passability_wheel_column`.** Rules 2 and 3 judge every class by the *wheel*
    column of the land-type table. Default (original): as above. Off: a cell is put in category 6 by
    rules 2 or 3 only when *M* is 0 (or ≤ 0.01 in rule 3) for every ground class: foot, track,
    wheel, hover, float, amphibious and float-on-beach. Nothing else changes: entry stays decided
    per class by claim 10 (it is already so in every setting), and S-0212's connectivity still
    depends only on the category and the zone table, so it does not become per class. With the
    default (original) behaviour, a mod that makes some land impassable to wheels but passable to
    tracks makes it unreachable for tracked vehicles too, as far as the zone table is concerned.
    With the switch off, such a land is category 0, so a tracked vehicle's path may lead into it and
    it enters, while a wheeled vehicle is planned into it likewise and then refused at its edge by
    claim 10. How the pathfinder then treats the refused step is S-0212's.
    - Backing, by part:

      | Part | Backing |
      |---|---|
      | Rule 1 | `unmeasured`: objects never stand outside the usable area for other reasons (S-0009, S-0010), so no observable separates category 7 from those rules |
      | Rule 2, crushable row | `measurable` (scenario L18 map A; scenario L19 map 1; its precedence over the wall row: scenario L19 map 1W) |
      | Rule 2, wall row | `measurable` (scenario L8 part A, together with claim 10's wall case; scenario L19 map 2) |
      | Rule 2, *M*(*ℓ*ₒ, wheel) = 0 row | `measurable` (scenario L4) |
      | Rule 2, rock row | `measurable` (scenario L18 map B; scenario L19 map 6) |
      | Rule 2, rubble row | `measurable` (scenario L18 map C) |
      | Rule 3, water row | `measurable` (scenarios L6 and L19 map 4) |
      | Rule 3, beach row | `measurable` (scenarios L18 map D and L19 map 3) |
      | Rule 3, the wheel column | `measurable` (scenarios L10 maps 1 and 2, and L12 part A) |
      | Rule 3, the threshold 0.01 and its comparison | `measurable` (scenario L10 maps 3 to 5) |
      | Rule 4, terrain-object rows | `measurable` (pattern 7: scenario L14; other patterns: scenario L19 map 5; both: S-0212 scenario P4, maps D and E) |
      | Rule 4, a cell holding only buildings of the kinds that can be set up (an ordinary building, an inactive laser-fence section, a firestorm wall whose owner's firestorm is off) is in category 0 | `measurable` (scenario L20) |
      | Rule 4, an active laser-fence section, a firestorm wall whose owner's firestorm is on, and the state switch of a section | `unmeasured`: no event of the original turns a house's firestorm on, so a firestorm wall whose owner's firestorm is on cannot be set up; which events make a laser-fence section active belongs to the structures specs, so no scenario of this spec can produce an active section; and with the default a section's state never affects the category, so whether a state switch triggers a recompute has no observable |
      | Rule 4, deciding objects of different categories in one cell (the cell's object order decides) | `unmeasured`: it needs a terrain object and a firestorm wall whose owner's firestorm is on in one cell, which cannot happen in the original, or two terrain objects of different patterns in one cell, which nothing is known to cause |
      | Recompute from the cell after the change | `measurable` (scenario L14) |

    - Confidence: high for rules 1 to 3 and 5, for the result with ordinary buildings, inactive
      laser-fence sections and firestorm walls with the firestorm off, and for the recompute
      events; medium for the terrain-object rows of rule 4, for an active laser-fence section
      having no effect and for the laser-fence state switch not being a trigger; high that no event turns a firestorm on in the original

12. **The zone table.** A movement zone may cross a cell of passability category *p* only where
    the table below says *yes*. *never* is a stronger *no* that the pathfinder treats specially
    (S-0212 owns the difference). S-0212 builds its connectivity from this table.

    | movement zone | 0 open | 1 crushable | 2 wall-like | 3 beach | 4 water | 5 partial | 6 blocked | 7 off-map |
    |---|---|---|---|---|---|---|---|---|
    | normal | yes | no | no | no | no | no | no | never |
    | crusher | yes | yes | no | no | no | no | no | never |
    | destroyer | yes | yes | yes | no | no | no | no | never |
    | amphibious-destroyer | yes | yes | yes | yes | yes | yes | no | never |
    | amphibious-crusher | yes | yes | no | yes | yes | no | no | never |
    | amphibious | yes | no | no | yes | yes | no | no | never |
    | subterranean | yes | yes | yes | no | no | no | yes | never |
    | infantry | yes | no | no | no | no | yes | no | never |
    | infantry-destroyer | yes | yes | yes | no | no | yes | no | never |
    | fly | yes | yes | yes | yes | yes | yes | yes | never |
    | water | no | no | no | no | yes | no | no | never |
    | water-beach | no | no | no | yes | yes | no | no | never |
    | crusher-all | yes | yes | yes | no | no | no | no | never |

    - Backing, by part:

      | Part | Backing |
      |---|---|
      | Columns 0 to 6, all thirteen rows | `measurable` (scenario L19: maps 0 to 6 and 1W for the eleven land zones, maps W0 to W6 for *water* and *water-beach*; each cell of the table is one observation) |
      | Further checks of the rows *normal*, *crusher*, *amphibious* and *infantry* | `measurable` (scenarios L4, L10, L14 and L18) |
      | Column 7 | `unmeasured`: rule 1 of claim 11 has no observable (objects never stand outside the usable area for other reasons), so no scenario can put a band of *off-map* cells between two reachable areas. The difference between *no* and *never* belongs to S-0212 (its claim 2) |

    - Confidence: high for the table's contents; medium for the subterranean and fly rows, where
      scenario L19 also depends on S-0212's region rule, which is of low confidence for movement
      zones that allow *blocked* (S-0212, claim 3); L19 also tells a failed gate from a failed search
      only through S-0212 claim 23, which is of medium confidence; medium for how the pathfinder uses
      the table

### Other uses of land types

13. **Building placement uses the buildable flag.** Apart from the rules of the structures specs
    (adjacency, occupancy, prerequisites), a building may be placed on a cell only if all of these
    hold: the cell is not marked as part of a bridge (a high-bridge cell, or another cell that a
    bridge marks as part of its body; the bridge specs own which cells a bridge marks), its slope
    shape index is 0, it has no overlay (the structures specs own the exceptions, such as walls
    built onto wall cells), and *B*(its land type) is *yes*. A building type with the naval flag
    ignores *B*: it needs instead a cell whose tile number lies in the water role range of S-0010
    (claim 5 and table R: 14 tiles from the water role's start). Where the water role is unset, that
    range covers tile numbers 0 to 12 (S-0010, `quirks.unset_role_range`). In the theater with
    index 5 (the theater index of S-0009's Import mapping, which S-0010 also uses) the original forces
    the water role unset whatever the theater data says (S-0010, claim 4).
    - Backing: `measurable` (scenario L7 for the buildable, flat and no-overlay requirements; L15
      for the naval range); the bridge marking is `unmeasured`: every cell a bridge is known to mark
      also holds a bridge overlay, which the no-overlay requirement already refuses, so no
      observable separates the two
    - Confidence: high for the buildable, flat and no-overlay requirements and for the naval range,
      medium for the bridge marking

14. **Restricted land type.** A vehicle type with a restricted land type may enter a cell only in
    these cases (all other cells count as impassable for it):
    - The cell's land type is the restricted land type.
    - The cell's land type is *tunnel*. On a tile of 3 × 4 or 3 × 5 cells (*w* × *h*), only the
      cell of sub-tile 6 qualifies, and on a tile of 4 × 3 or 5 × 3 cells only the cell of sub-tile
      2. On tiles of other sizes every tunnel cell qualifies.
    - The cell's overlay type has registration index 237 or 238 in the overlay registry, and the
      vehicle is at a height level other than that cell's ground level (that is, on a deck above
      it). The two indices are engine constants, whatever types the data puts there.

    So a restricted vehicle cannot cross a high bridge over ground of another land type unless the
    bridge's cells hold overlays of those two indices. The absent-key default is *none* (no
    restriction).

    **Quirk `quirks.restricted_deck_overlay_indices`.** Default (original): the deck exception
    applies only to cells whose overlay type has registration index 237 or 238. Off: it applies to
    every high-bridge cell when the vehicle is on that cell's bridge deck, whatever the overlay
    type's index, so a restricted vehicle may cross any high bridge.
    - Backing: `measurable` (scenario L8: part A for the restriction, part B for tunnel cells, part C
      for the bridge exception)
    - Confidence: high for the restriction itself; medium for the two exceptions (the tile sizes are
      read as width × height)

## Formulas

> *M*(*ℓ*, *c*) = σ(min(*d*, 1.0)), where *d* is the key's value as S-0003 gives it (claim 6: without
> a percent sign, a single-precision number; claim 7: with one, a double-precision product rounded
> toward zero), and σ(*x*) is *x* rounded to single precision under the round-toward-zero mode
> (S-0205, `quirks.chop_rounding`). The comparison with 1.0 is exact. For a value without a percent
> sign σ changes nothing, because *d* is already a single-precision number. Unit: factor
> (1.0 = 100 %).

> Category rule 3 threshold: *M*(*ℓ*, wheel) ≤ *t*, where *t* is the double-precision number nearest
> to 0.01, and the single-precision value *M* is compared with *t* exactly, with no rounding on
> either side. Domain: floating point. No single-precision number equals that double, so
> "≤" and "<" give the same results. The single-precision number nearest to 0.01 (which a key value
> of 0.01 gives, S-0003 claim 6) lies just below the double, so it is blocked; a comparison made in
> single precision against that same single with "<" would not block it (scenario L10, map 5).
> Unit: factor.

> Category rule 2 and claim 10 zero tests: *M* = 0 exactly, on the single-precision value. Domain:
> floating point (single precision). A comparison with no arithmetic, so no rounding applies. Unit:
> factor.

> Cliff-back test: some listed neighbour *n* has level(*n*) ≥ level(cell) + 4. Integer height levels,
> compared exactly. Unit: height levels.

> Cliff-back rule active ⇔ *r* mod 256 = 2, where *r* is the imported integer setting and mod gives
> a result from 0 to 255. Exact integer arithmetic, so no rounding applies. Unit: setting value.
> (Examples: 2, 258 and −254 make the rule active; 1, 3, 130 and 255 do not.)

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Land types | 12 | land types | Labels in Terms; claim 1 |
| Locomotion classes | 8 | classes | Labels in Terms; claim 1 |
| Land multiplier cap | 1.0 | factor | Claim 2 |
| Winged multiplier | 1.0 | factor | Fixed whenever the section is read; claim 2 |
| Missing multiplier key | 1.0 | factor | Claim 2 |
| Missing buildable key | no | boolean | Claim 2 |
| Start-up row (section never read) | 0 for all classes, no | factor, boolean | Claim 2 |
| Tile land code table | see claim 6 | land type per code | Codes 0–15 |
| Ore-capable slope shapes | 0 to 4 | slope shape index | Claim 5, row 4, and claim 7 |
| Cliff-back height step | 4 | height levels | Claim 8 |
| Cliff-back active value | 2 | setting value | Claim 8; the rule is active when the setting modulo 256 equals it, and does nothing otherwise |
| Cliff-back reduction modulus | 256 | setting value | Claim 8 and Formulas |
| Category 6 threshold (rule 3) | 0.01 | factor | Claim 11 |
| Occupation pattern that gives category 2 | 7 | pattern value | Claim 11, rule 4 |
| Naval placement range | 14 | tiles | Claim 13; the water role range of S-0010 |
| Tunnel tile sizes with one qualifying cell | 3 × 4, 3 × 5 (sub-tile 6); 4 × 3, 5 × 3 (sub-tile 2) | cells, sub-tile number | Claim 14 |
| Overlay registration indices passable on a deck | 237, 238 | registration index | Claim 14 |
| Number of passability categories | 8 | categories | Claims 11 and 12 |
| Movement zones | 13 | zones | Claim 12 |

## State and ordering

| State | Event | Result |
|---|---|---|
| any cell | map load; overlay placed, changed or removed; terrain changed | land recompute: land type (claims 5 to 8) and passability category from the new land type and contents (claim 11) |
| any cell | an infantry unit, vehicle, aircraft, building or terrain object put into or taken out of the cell while on the ground | land recompute, seeing the contents after the change (claim 5) |
| cell holding a terrain object | the terrain object is removed or destroyed | land recompute: rule 4 no longer sees it, so the category is decided without it (claim 11) |
| cell | infantry or vehicle enters or leaves | land recompute, but the category is unchanged: it does not depend on them (claim 11) |
| cell with ore or gems, slope shape ≥ 5 | land recompute | overlay removed, tile land type used (claim 7) |
| cell whose tile was dropped under row 3 | next land recompute | row 2 applies: the overlay's land type if it has an overlay (claim 5) |
| cell with a bridge | bridge destroyed | bridge overlays removed, land recompute (claim 9) |

A map layer's change to a land type's row is in force for that game. Land-type sections are
game-wide sections, so, as for any game-wide section, the change can outlast the game: the row stays
as that map layer set it, in later games of the session too, until a rules layer of a later game
gives the same section again, which re-applies or resets every key of the row (claim 2). If the base
rules give the section, the change therefore ends at the next game start (glossary: *rules
database*; S-0006, claim 10).

## Edge cases and quirks

- `quirks.passability_wheel_column` (claim 11): the category uses the wheel column for every class.
- `quirks.passability_laser_fence_lost` (claim 11): an active laser-fence section has no effect on
  the category, so a fence cell that rules 1 to 3 leave undecided and that holds no deciding object
  of rule 4 is in category 0.
- `quirks.cliff_back_case_sets` (claim 8): behind a cliff, row 1 of claim 5 (wall, railroad and
  own-land-type overlays) becomes *rock* whatever its land type, while the other rows do so only for
  *clear*, *water*, *beach* and *ice* (row 3's land type is always *clear*, so it is unaffected).
- `quirks.restricted_deck_overlay_indices` (claim 14): restricted vehicles may use a bridge deck
  only above overlays of registration indices 237 and 238.
- `quirks.unset_role_range` (owned by S-0010, used by claim 13): with the water role unset, naval
  buildings may be placed on tiles 0 to 12.
- **Firestorm walls are inert in the original.** No event turns a house's firestorm on, so a
  firestorm wall is never a deciding object of rule 4 (claim 11). This is original behaviour, not a
  quirk.
- **Cliff-back setting wraps.** The setting is reduced modulo 256 (claim 8, Formulas), so a value
  such as 258 acts as 2. The importer warns for any value other than 0, 1 and 2.
- **Overlay land types are mostly ignored.** An overlay's land type sets the cell's land type only
  for walls, railroads, overlays with the own-land-type flag, ore or gem overlays, and any overlay on
  a cell without a usable tile (claim 5, row 2, and row 3 from the next land recompute). For every
  other overlay it matters only through category rule 2. This is original behaviour, not a quirk switch.
- **Negative multipliers.** A negative *M* allows entry (claim 10), and it gives a track (S-0207's term) into such a
  cell a negative target fraction (S-0207, claim 9). What follows depends on S-0207's claim 10:
  - **Acceleration flag off:** the speed fraction is clamped to 0 at once, so the vehicle is stuck.
  - **Acceleration flag on, at or beyond the slowdown distance from its destination:** the vehicle
    decelerates towards the negative target at its usual rate, still moving while it slows, until the
    clamp leaves it at 0, where it is stuck. A vehicle marked as crushing (S-0207, claim 10, case 1)
    drops to 0 at once.
  - **Acceleration flag on, within the slowdown distance of its destination:** the braking floor
    (S-0207, claim 10, case 2, `quirks.drive_brake_floor_raises`) sets the speed fraction to at least
    *F*₀.₃ whatever the target, so the vehicle keeps moving at that fraction or faster and is not stuck.
  - **Acceleration flag on, passive flag set:** S-0207's claim 10 leaves the speed fraction
    unchanged, so the vehicle keeps the speed it had.

  The original behaves this way, and our engine reproduces it. The importer warns for any negative
  land multiplier. Scenario L2 uses the acceleration flag off, so only the first case is tested.
- **A zero multiplier on the cell being entered.** When a vehicle begins a track (S-0207's term) into a cell whose
  multiplier for its class is 0 (which claim 10 normally prevents, but which can happen, for example
  when the cell's land type changes after the step was planned), S-0207 gives it a target speed
  fraction of 0.5 instead of 0, so it does not stall.

## Import mapping

| User data | Feeds | Form | Absent |
|---|---|---|---|
| Rules sections `[Clear]`, `[Road]`, `[Water]`, `[Rock]`, `[Wall]`, `[Tiberium]`, `[Beach]`, `[Rough]`, `[Ice]`, `[Railroad]`, `[Tunnel]`, `[Weeds]` (the twelve section names that Q-0007 question 1 asks for; see Questions for the project lead) | the rows of land types 0 to 11 | section names fixed in the engine, case as written | row unchanged by that rules layer; start-up row if never given (claim 2) |
| keys `Foot`, `Track`, `Wheel`, `Hover`, `Float`, `Amphibious`, `FloatBeach` in those sections | *M* for foot, track, wheel, hover, float, amphibious, float-on-beach | decimal or percent (S-0003, claims 6 and 7), capped at 1.0 | 1.0 (reset on override, S-0006 claim 5) |
| a key `Winged` in those sections | nothing: not read (claim 2) | — | *M*(*ℓ*, winged) = 1.0 whenever the section is read |
| key `Buildable` in those sections | *B* | boolean | no |
| vehicle, infantry or aircraft section, key `SpeedType` | locomotion class | name: `Foot`, `Track`, `Wheel`, `Hover`, `Winged`, `Float`, `Amphibious`, `FloatBeach` (case ignored) | keeps value; engine default *foot* for infantry and *winged* for aircraft; vehicles with none get track if `Crusher=yes`, else wheel (claim 3) |
| same sections, key `Crusher` | crusher flag (used for the class default) | boolean | no |
| same sections, key `MovementZone` | movement zone | name: one of the thirteen zone names of claim 12 with the hyphens removed, matched with letter case ignored (so *amphibious-destroyer* is written as one word), except that *subterranean* is spelled with one *r* (*subterannean*) | keeps value; engine default *normal* |
| vehicle section, key `MovementRestrictedTo` | restricted land type | land-type name (case ignored) or `<none>`; any other text gives none (claim 4) | none |
| overlay section, key `Land` | overlay land type | land-type name (case ignored) or `<none>`; any other text gives none (claim 4). Our engine reads none as clear, with an importer warning (claim 4's departure) | clear (then tiberium for ore and gem overlays, claim 5) |
| overlay section, keys `NoUseTileLandType`, `Wall`, `Tiberium`, `Crushable`, `IsARock`, `IsRubble` | own-land-type, wall, tiberium, crushable, rock and rubble flags | boolean | no |
| building section, key `Naval` | naval flag (claim 13) | boolean | no |
| building section, key `LaserFence` | laser-fence flag (claim 11) | boolean | no |
| building section, key `FirestormWall` | firestorm-wall flag (claim 11) | boolean | no |
| terrain-object section, keys `TemperateOccupationBits`, `SnowOccupationBits` | occupation patterns (claim 11, rule 4) | integer | owned by the terrain-object spec |
| `[General]`, key `CliffBackImpassability` | cliff-back setting *r* | integer; only its remainder modulo 256 matters (claim 8, Formulas) | 0 (no effect) |
| Theater tile data, per tile cell | tile land code, slope shape index | public tile format (S-0010) | no image data for a cell: row 3 of claim 5 (claim 6) |
| Theater data file, the water role | the naval placement range (claim 13) | as in S-0010 | unset: tiles 0 to 12 (claim 13) |

## Measurement scenarios

All run under the glossary's *measurement setup*, with stock data, a purpose-made test map and a
*recorder* where positions, cell contents or tick counts are read. "Override" means a rules section
inside the test map (a named test file, removed after the run). The values in overrides are invented
test values, not stock values. Where a scenario overrides a type's section, it gives every field this
spec reads for that type explicitly (locomotion class, crusher flag, movement zone, restricted land
type; for an overlay type its land type and all six flags), unless the scenario says that a key is
left out, so the result does not depend on stock values; fields a scenario does not name take the
values that make them inert (no restriction, every overlay flag off). "A vehicle of class *c*" means a vehicle type with a driving locomotor whose
locomotion class an override sets to *c*. Speeds are read as S-0207 defines them. Unless a scenario
says otherwise, the test map also overrides the clear and road land types with every multiplier
100 % and the buildable key *yes*, so the ground the units use does not depend on stock values.
"A band" is a strip of cells 3 wide that runs from one edge of the usable area to the other, so a
unit can reach the far side only through it, unless the scenario adds a route around it. Every
scenario is deterministic: one run per map and per run letter is enough, unless the scenario says
otherwise.

**L1: multiplier and cap (claim 2).**
- *Setup:* flat map of clear ground. A vehicle of class wheel and zone *normal*, with the
  acceleration flag off. Run 1: an override gives the clear land type's section only a wheel
  multiplier of 50 %. Run 2: 150 %. Run 3: 100 %.
- *Procedure:* order the vehicle along a straight route of 20 cells.
- *Observable:* ticks between its arrival at the centres of cells 5 and 15 (S-0207, scenario D1).
- *Expected:* run 2 equals run 3, and run 1 takes about twice as long (S-0207's formulas give the
  exact value).
- *Falsified if:* run 2 differs from run 3, or run 1 is not slower.

**L2: zero and negative multipliers (claims 2 and 10).**
- *Setup:* clear map with a band of tiles whose tile land code is 14 (rough). An override gives the
  rough land type a track multiplier of 0 and a wheel multiplier of 100 %. A vehicle of class track
  and a vehicle of class wheel on one side, both with movement zone *crusher* (the zone table allows
  category 0 for both) and the acceleration flag off. Map 2: the same, with a route of clear ground
  around the band, longer than the straight route through it. Map 3: as map 2, but the override
  gives rough a track multiplier of −50 %.
- *Procedure:* order both vehicles to a cell beyond the band.
- *Observable:* whether each vehicle's position (recorder) is ever in a band cell, the route it
  takes, and whether it reaches the destination.
- *Expected:* the wheel vehicle crosses the band on every map. Map 1: the track vehicle never enters
  a band cell and stops at the edge. Map 2: it drives around. Map 3: it heads into the band (the
  negative multiplier allows entry) and stalls at the band's edge with a speed fraction of 0
  (S-0207), never reaching the destination.
- *Falsified if:* on map 1 or 2 the track vehicle enters a rough cell; on map 3 it drives around the
  band or crosses it.

**L3: ore land type and steep-slope removal (claims 5 and 7).**
- *Setup, part A:* a map that places ore overlays on a flat cell, a simple-slope cell (slope shape 1
  to 4) and a corner-slope cell (shape 5 or more). An override of the ore overlay type sets its
  tiberium flag, leaves its land key absent and turns its own-land-type, wall, crushable, rock and
  rubble flags off. The test vehicle has class wheel and zone *normal*. Overrides give the tiberium
  land type a wheel multiplier of 50 % and the clear land type one of 100 %. *Part B:* as part A, but the ore overlay type's override
  sets the own-land-type flag.
- *Procedure:* load the map, look at the three cells, then order the test vehicle across the flat
  ore cell and across a clear cell of the same length.
- *Observable:* whether ore is shown and can be harvested on each cell; the vehicle's crossing time
  in ticks over the ore cell and over the clear cell.
- *Expected:* part A: ore on the flat and simple-slope cells, none on the corner-slope cell; the
  vehicle is slowed on the ore cell. Part B: ore on the flat cell only (it is also removed from the
  simple-slope cell).
- *Falsified if:* in part A ore remains on the corner-slope cell, ore is missing from the
  simple-slope cell, or the vehicle is not slowed; in part B ore remains on the simple-slope cell.

**L4: overlay land type ignored, except through the category and row 1 (claims 5, 10, 11 and 12).**
- *Setup:* clear map. Two overlay types that the test map places are overridden. Type 1: land type
  *rock*, own-land-type flag off, and the wall, tiberium, crushable, rock and rubble flags off. Type
  2: the same, but with the own-land-type flag on. A row of cells holds type 1, a second row type 2.
  An override gives the rock land type foot and wheel multipliers of 0. Map 2 (control): the same,
  with the rock land type's foot and wheel multipliers overridden to 100 %. Map 3: the same, with
  the rock land type's foot multiplier overridden to 0 and its wheel multiplier to 100 % (so rule 2
  and rule 3 leave both rows in category 0). An infantry unit of class foot and zone *infantry*, and
  a vehicle of class wheel and zone *normal*.
- *Procedure:* order each unit onto a cell of each row.
- *Observable:* whether each unit enters each row.
- *Expected:* map 1: neither unit enters either row, the type-1 row because of category 6 (rule 2),
  the type-2 row because of land type *rock* (claim 10, and category 6). Map 2: both units enter both
  rows, the type-1 row because its category becomes 0, the type-2 row because its land type becomes
  passable and its category 0. Map 3: both units enter the type-1 row (row 5 of claim 5 leaves the land type to
  the clear tile); on the type-2 row the vehicle enters and the infantry unit is refused (row 1
  gives land type *rock*, and *M*(rock, foot) = 0).
- *Falsified if:* on map 1 a unit enters either row; on map 2 either row stays closed to either
  unit; on map 3 the infantry unit is refused at the type-1 row or enters the type-2 row, or the
  vehicle is refused at either row.

**L5: cliff-back (claims 5, 6 and 8).**
- *Setup:* a map with a straight cliff of 4 levels, using stock cliff tiles. Along the foot of the
  cliff lie, in this order: cells of tile land code 0 (clear); cells of tile land code 11 or 12
  (road); and clear cells holding an overlay of a type whose override gives it land type *road*,
  the own-land-type flag on and the wall, tiberium, crushable, rock and rubble flags off. A vehicle
  of class wheel and zone *normal*. An override gives the rock land type foot and wheel multipliers
  of 0. Runs differ only in the cliff-back setting, given by override: run A 2, run B 0, run C 258,
  run D 1, run E 2, run F 2. Run F uses a second flat clear map with no cliff, in which one
  cell *p* is set to ground level 4 levels above its surroundings (a level written in the map
  file, with the same tile), so it has no neighbour 4 levels up except through *p*.
- *Procedure:* runs A to D: order the vehicle onto each clear foot cell that has one of claim 8's
  neighbours 4 levels up, and onto a clear cell farther from the cliff. Run E: order it onto each
  road foot cell and each overlay foot cell that has one of claim 8's neighbours 4 levels up.
- *Observable:* whether the vehicle enters each cell.
- *Expected:* runs A and C: the clear foot cells are refused, the far cell entered. Runs B and D:
  all entered. Run E: the road foot cells are entered (row 5 leaves *road* unchanged), the overlay
  foot cells are refused (row 1 turns them into *rock* whatever their land type).
- *Run F, procedure and observable:* order the vehicle onto each of the eight cells around *p*, onto
  *p* + (−2, −2), and onto a cell farther away; record whether it enters each.
- *Run F, expected:* it is refused at *p* + (0, 1), *p* + (1, 0), *p* + (−1, 1), *p* + (1, −1),
  *p* + (−1, −1) and *p* + (−2, −2), and enters *p* + (0, −1), *p* + (−1, 0), *p* + (1, 1) and the
  farther cell. A rule over all eight neighbours would also refuse the last three.
- *Falsified if:* run A gives the same result as run B, the far cell is refused in run A, run F
  enters any of the six refused cells or refuses any of the four entered ones, run C
  differs from run A, run D differs from run B, or in run E a road foot cell is refused or an
  overlay foot cell is entered.

**L6: bridges (claims 9 and 11).**
- *Setup:* a river of water tiles crossed by a high bridge and by a low bridge. Overrides: the high
  bridge's overlay types get land type *clear* and the own-land-type flag off; the low bridge's
  overlay types get land type *road* and the own-land-type flag on; both with the wall, tiberium,
  crushable, rock and rubble flags off. The water land type gets float 100 % and wheel 0; the road
  land type float 0 and wheel 100 %. A ship of class float and zone *water*, and a vehicle of class
  wheel and zone *normal*.
- *Procedure:* order the ship along the river under both bridges, and the vehicle across both. Then
  destroy the low bridge and repeat both orders.
- *Observable:* whether each unit passes each bridge.
- *Expected:* the ship passes under the high bridge but not through the low bridge. The vehicle
  crosses both. After the low bridge is destroyed, the vehicle can no longer cross there, and the
  ship can pass.
- *Falsified if:* the ship cannot pass under the high bridge or passes through the intact low
  bridge, the vehicle cannot cross the high bridge or the intact low bridge, or the destroyed low
  bridge stays passable to the vehicle or stays closed to the ship.

**L7: buildable and flat (claims 2 and 13).** This also settles part of S-0006 open question 7.
- *Setup:* skirmish start with a construction yard (a building that produces buildings) on clear ground, flat cells of clear and of road
  land within placement range, a simple-slope cell of clear land within range, and a flat clear
  cell holding an overlay of a type whose override gives it land type *clear* and turns its
  own-land-type, wall, tiberium, crushable, rock and rubble flags off. Run A: an
  override gives the clear land type's section only a foot multiplier of 100 %, so its buildable
  flag resets to *no*, and gives the road land type's section the buildable key *yes*. Run B: an
  override gives both the clear and the road land type the buildable key *yes*.
- *Procedure:* start placing a building and move its foundation over each kind of cell.
- *Observable:* which foundation cells the placement preview marks as allowed.
- *Expected:* run A: flat clear cells refused, flat road cells allowed. Run B: flat clear and road
  cells allowed, the slope cell and the overlay cell refused.
- *Falsified if:* run A allows a flat clear cell or refuses a flat road cell, or run B refuses a
  flat clear or road cell or allows the slope cell or the overlay cell.

**L8: restricted land type and walls (claims 10, 11 and 14).**
- *Setup, part A:* a map with roads and clear ground. A vehicle type of class wheel and zone
  *normal* whose restricted land type an override sets to road. A wall segment across a road, of a wall type whose override gives it land
  type *wall*, the wall flag on, and the crushable, own-land-type, tiberium, rock and rubble flags
  off. A vehicle of zone *normal* and one of zone *destroyer*, both of class wheel and without a
  restricted land type. *Part B:* a vehicle type of class wheel and zone *normal*, restricted to
  railroad by override, on a railroad leading into tunnel tiles of land type *tunnel*, with tiles of
  the sizes listed in claim 14 and of one other size; an override gives the railroad and tunnel land
  types every multiplier 100 %. *Part C:* a vehicle of class wheel and zone *normal*, restricted to
  road, and a high bridge over water whose overlay
  types are not those of registration indices 237 and 238; if the overlay types at those two
  indices form a high bridge in the test theater, a second bridge built from them.
- *Procedure:* part A: order the restricted vehicle onto clear cells and along the road; order the
  two other vehicles through the wall cell. Part B: order the restricted vehicle onto each tunnel
  cell. Part C: order the restricted vehicle across each bridge.
- *Observable:* which cells each vehicle enters (recorder positions).
- *Expected:* part A: the restricted vehicle never leaves road cells; the *normal* vehicle cannot
  cross the wall; the *destroyer* vehicle drives through the wall cell.
  Part B: on tiles of the listed sizes only the cell of the listed sub-tile is entered; on the
  other tile every tunnel cell is. Part C: the first bridge is refused; the second, if present, is
  crossed.
- *Falsified if:* the restricted vehicle enters a clear cell, the *normal* vehicle crosses the wall,
  the *destroyer* vehicle does not drive through the wall cell, part B admits a tunnel cell of
  another sub-tile on a listed tile or refuses a tunnel cell of the other tile, or part C lets the
  vehicle cross the first bridge or refuses the second.

**L9: default locomotion classes (claim 3; named test file).**
- *Setup, all parts:* the class key must be absent for the tested types in **every** applied rules layer,
  because an absent key keeps a class that an earlier rules layer gave (claim 3). The scenario therefore
  uses a named loose copy of the base rules from whose sections of the chosen types the class key is
  removed, used for this scenario only. The test checks that no other applied rules layer gives the class
  key for them. The vehicle default is fixed by the crusher flag that the type holds when the first
  section without a class key is read, which is the loose copy's section, so the loose copy itself
  sets each chosen vehicle type's crusher flag. The test map's overrides of the chosen types give
  every other field this spec reads, but leave out the class key and the crusher flag. Type
  registries are rebuilt at every game start (S-0004, claim 1), so earlier games of the session do
  not matter.
- *Part A:* two such vehicle types, one whose loose-copy section sets the crusher flag and one whose
  section sets it to *no*, both of zone *normal* and with the acceleration flag off. A flat clear
  map. An override gives the clear land type a track multiplier of 50 % and a wheel multiplier of
  100 %. *Part B:* one such infantry type, with zone *infantry*, and an
  infantry type whose override sets class wheel, also with zone *infantry*. A clear map with a band
  of rough tiles (code 14) and a route of clear ground around it. An override gives rough a foot
  multiplier of 0 and every other multiplier 100 % (so the band's category is 0). *Part C:* as
  part A, with a third such vehicle type whose loose-copy section sets the crusher flag to *no* and
  whose map override sets it to *yes* (still without a class key).
- *Procedure:* parts A and C: order the vehicles along parallel straight routes of 20 cells. Part B:
  order both infantry units to a cell beyond the band.
- *Observable:* parts A and C: ticks between arrival at the centres of cells 5 and 15. Part B:
  whether each infantry unit's position is ever in a band cell.
- *Expected:* part A: the crusher vehicle (class *track*) takes about twice as long as the other
  (class *wheel*). Part B: the infantry unit
  without a class key never enters a band cell (class *foot*, *M* = 0): it walks around or stops at
  the edge, as the pathfinding spec decides. The wheel-class infantry unit crosses the band. Part C:
  the third vehicle is as fast as the vehicle without the crusher flag (it keeps class *wheel*,
  because the map layer's crusher flag comes after its class was fixed).
- *Falsified if:* in part A both are equally fast, or the vehicle without the crusher flag is the
  slower; in part B the infantry unit without a class key enters a band cell, or the wheel-class
  unit does not (which would mean infantry do not use claim 10, and part B cannot decide); in part C
  the third vehicle is as slow as the crusher vehicle.

**L10: the wheel column, the threshold and the zone table (claims 11 and 12).**
- *Setup:* as L2, but the override gives rough a wheel multiplier of 0 and a track multiplier of
  100 %. A vehicle of class track with zone *crusher*. Map 2: a route of clear ground around the
  band. Maps 3, 4 and 5: as map 1 (no route around), with the track multiplier still 100 %, but the
  rough land type's wheel multiplier written as a plain decimal without a percent sign: 0.005 on
  map 3, 0.02 on map 4 and 0.01 on map 5. By S-0003 claim 6 these give single-precision values
  just below 0.005, 0.02 and 0.01, all nonzero.
- *Procedure:* order the vehicle to a cell beyond the band.
- *Observable:* whether its position is ever in a band cell, the route it takes, and whether it
  reaches the destination.
- *Expected (default, the original):* map 1: the vehicle does not cross the band (category 6 from
  the wheel column), although its own class could enter rough. Map 2: it drives around. Map 3: it
  does not cross (category 6, although *M*(rough, wheel) ≠ 0). Map 4: it crosses (category 0,
  because the value lies above 0.01). Map 5: it does not cross (the value lies just below the double
  nearest 0.01, Formulas).
- *Falsified if:* the vehicle drives across the band on map 1, 3 or 5, or does not cross it on
  map 4. On map 5, a crossing would also mean that the comparison is made in single precision with
  "<" (Formulas).

**L11: tile land codes (claim 6).**
- *Setup:* in one theater, a flat test map with one strip of several cells for each tile land code
  that occurs on flat tile cells of that theater (the test author reads the codes from the tile
  files), within placement range of a construction yard. Four runs, *i* = 0 to 3: an override gives
  each of the twelve land types its buildable key, *yes* when bit *i* of the land type's label
  (Terms) is 1 and *no* otherwise. Control run: every buildable key *yes*.
- *Procedure:* in each run, start placing a building and move its foundation over each strip.
- *Observable:* for each strip, which runs mark its cells as allowed. The four answers give a 4-bit
  number.
- *Expected:* for each strip, the 4-bit number is the label of the land type that claim 6's table
  gives for its code.
- *Falsified if:* any strip gives another number. A strip that the control run refuses is unusable
  for the test (something other than its land type blocks placement) and is left out.

**L12: the start-up row and the missing-key value (claim 2; named test file).**
- *Setup, parts A and B:* a loose copy of the base rules with the rough land type's section
  removed, used for this scenario only; the test checks that no other applied rules layer gives that
  section. Land-type sections are game-wide and keep their values for the whole session (State and
  ordering), so each of parts A and B runs as the first game after a fresh program start, with the
  loose copy already in place. A flat map with a band of tiles of tile land code 14 (rough) across clear ground, within
  placement range of a construction yard. Part A: a vehicle of class wheel and zone *normal*.
  Part B: a vehicle of class winged and zone *fly*, with the acceleration flag off. *Part C:* the
  same map with a loose copy of the base rules whose rough section holds only the buildable key
  (*yes*), and a vehicle of class track and zone *normal* with the acceleration flag off.
- *Procedure:* part A: order the vehicle across the band, and try to place a building on it.
  Part B: order the vehicle along a straight route that crosses 10 rough cells, and along one of
  10 clear cells. Part C: order the vehicle along a straight route that crosses 10 rough cells and
  10 clear cells, and try to place a building on the band.
- *Observable:* parts A and C: whether placement on the band is allowed; part A: whether the
  vehicle enters the band; parts B and C: ticks for each 10-cell stretch.
- *Expected:* part A: the vehicle does not enter (*M* = 0, and category 6 by rule 3), and placement
  is refused. Part B: the vehicle enters rough (claim 10 lets a winged class enter any cell, and
  zone *fly* may cross category 6) at about half its clear speed, because S-0207 replaces a zero
  multiplier by 0.5. Part C: the vehicle is as fast on rough as on clear (every multiplier of the
  rough row is 1.0), and placement on the band is allowed.
- *Falsified if:* part A's vehicle enters the band or placement is allowed; part B's vehicle is as
  fast on rough as on clear; part C's vehicle is slower on rough than on clear, or placement on the band is refused in
  part C. Part B is
  inconclusive if the order is refused outright, since the pathfinder's regions for zone *fly*
  over *blocked* cells are of low confidence (S-0212, claim 3).

**L13: the fixed winged multiplier and the winged case of entry (claims 2, part (b), and 10).**
- *Part A, setup:* a flat map with a band of rough tiles (code 14) across clear ground. Run A: an override
  gives the rough land type's section only a wheel multiplier of 50 % (so the band's category is
  0). Run B: the same, plus a key spelled like the winged class name with 50 %. A vehicle of class
  winged and a vehicle of class wheel, both of zone *normal* and with the acceleration flag off.
- *Procedure:* order both vehicles along straight routes that cross 10 rough cells and 10 clear
  cells.
- *Observable:* ticks for each 10-cell stretch, per vehicle.
- *Expected:* in both runs the wheel vehicle is slower on rough, and the winged-class vehicle is as
  fast on rough as on clear.
- *Falsified if:* the winged-class vehicle is slower on rough in either run.
- *Part B, setup:* a flat clear map with one wall segment across a corridor, of a wall type whose
  override gives it land type *wall*, the wall flag on, and the crushable, own-land-type, tiberium,
  rock and rubble flags off. Two vehicles of zone *fly* and with the acceleration flag off, one of
  class winged and one of class wheel.
- *Part B, procedure:* order each vehicle to a cell on the far side of the wall.
- *Part B, observable:* whether each vehicle passes through the wall cell (recorder positions).
- *Part B, expected:* the winged-class vehicle passes through the wall cell (claim 10 lets the
  winged case decide before the wall case, and zone *fly* may cross category 2); the wheel-class
  vehicle does not enter the wall cell (zone *fly* is not on the wall list).
- *Part B, falsified if:* the winged-class vehicle is refused at the wall cell, or the wheel-class
  vehicle enters it. If the winged-class vehicle is refused, claim 10's cases are not taken in order
  and the claim is revised.

**L14: recompute when a terrain object is removed (claims 5, 11 and 12).**
- *Setup:* a flat clear map divided by a band of water 3 cells wide, with one land gap one cell wide
  through it. In the gap stands a terrain object whose type's occupation pattern for the theater an
  override sets to 7. A vehicle of class wheel and zone *normal* on one side. Run 1: the vehicle
  belongs to the human-controlled house. Run 2: it belongs to a computer-controlled house and is a
  team member, ordered through its team's script, as in S-0212 scenario P3 run 2.
- *Procedure:* order the vehicle to a cell on the far side. Then destroy the terrain object
  (S-0212 claim 5 brings connectivity up to date in the same tick) and order the vehicle again.
- *Observable:* whether the vehicle moves, and whether it passes through the gap; in run 2 also,
  for the first order, whether the vehicle is still a team member afterwards and whether its house
  gives it a new destination.
- *Expected:* before: the order is refused without movement (the gap cell is category 2, which
  *normal* may not cross, so the far side is in another reach region and the gate of S-0212 claim 6
  fails). In run 2 the vehicle stays a team member and gets no new destination (S-0212 claim 23,
  first row). After: the vehicle drives through the gap.
- *Falsified if:* the vehicle passes the gap or moves towards it while the object stands; in run 2
  it leaves its team or heads for a new destination of its house after the first order (that would
  mean the gate passed and only the occupied cell stopped it, so the category is not one that
  *normal* refuses); or it never passes the gap after the object is destroyed.

**L15: naval placement range (claim 13).**
- *Setup:* run A: in a theater whose water role is set, a map with flat water cells whose tiles
  are the first 14 tiles of the water role, and flat cells of any later tiles whose tile land code is
  9, if the theater has them. A building type available to the player whose naval flag an override
  sets, within placement range of a construction yard. Run B: the same building in the theater with
  index 5 (S-0009's Import mapping; S-0010, claim 4), with flat cells of tile numbers 0 to 12 and 13 or above.
- *Procedure:* start placing the naval building and move its foundation over each kind of cell.
- *Observable:* which foundation cells the placement preview allows.
- *Expected:* run A: only cells of the 14 water-role tiles are allowed. Run B: cells of tiles 0 to
  12 are allowed, and cells of higher tile numbers are refused.
- *Falsified if:* run A allows another tile or refuses one of the 14, or run B differs from the
  expected tile numbers.

**L16: cells without a usable tile (claims 5 and 6).**
- *Setup:* in a theater whose tilesets declare no different authoring tile count (S-0009 claim 8),
  a flat clear map with five cells far enough apart that each can be approached on its own:
  - cell A: tile number equal to the theater's tile count, no overlay.
  - cell B: the same tile number, holding overlay type *T*.
  - cell C: a valid one-cell tile of tile land code 0 (clear) with sub-tile number 1, holding
    overlay type *T*. A second cell, C′, holds overlay type *T* on any multi-cell tile, at a
    sub-tile that is an empty cell of that tile.
  - cell D: a valid one-cell tile of tile land code 14 (rough) with sub-tile number 1, without an
    overlay.
  - control cell E: a valid clear tile at sub-tile 0, holding overlay type *T*.

  An override gives overlay type *T* land type *rough* with the own-land-type, wall, tiberium,
  crushable, rock and rubble flags off. An override gives rough a track multiplier of 0 and every
  other multiplier 100 % (so rule 2 and rule 3 leave each cell in category 0). A vehicle of class
  track and a vehicle of class wheel, both of zone *crusher*.
- *Procedure:* order each vehicle onto each cell in turn.
- *Observable:* whether each vehicle enters each cell.
- *Expected:* the wheel vehicle enters every cell. The track vehicle enters cells A and D (land type
  *clear*: row 2 without an overlay, and row 3 then row 2 without one) and the control cell E (row
  5: *T*'s land type is ignored), and is refused at cells B, C and C′ (land type *rough*: row 2
  with an overlay, and row 3 followed by row 2 at the second pass of map load).
- *Falsified if:* the track vehicle enters cell B, C or C′, or is refused at cell A, D or E; or the
  wheel vehicle is refused anywhere (something other than the land type blocks the cell, and the
  run cannot decide). Under row 5 instead of row 3, cells C and C′ would be entered (their tiles
  read as *clear*: code 0, and an empty tile cell) and cell D refused (code 14, *rough*); so C or C′
  entered, or D refused, means the sub-tile was not found unusable.

**L17: land-type names (claim 4).**
- *Setup:* a map with roads and clear ground, as L8 part A, and a row of clear cells holding
  overlay type *T*. Three vehicle types of class wheel and zone *normal*, whose restricted land type
  an override writes as: type 1, the road name in mixed letter case (for example *rOAd*); type 2,
  the text that Import mapping gives for *none*; type 3, a misspelling of the road name. Overlay type *T*: an override gives it the rock
  name in mixed letter case as its land type, the own-land-type flag on, and the wall, tiberium,
  crushable, rock and rubble flags off. An override gives rock a wheel multiplier of 0. A fourth
  vehicle of class wheel and zone *normal* without a restricted land type.
- *Procedure:* order the three restricted vehicles onto clear cells away from the road; order the
  fourth vehicle onto the row of overlay cells.
- *Observable:* which cells each vehicle enters.
- *Expected:* type 1 never leaves road cells; types 2 and 3 enter clear cells (no restriction); the
  fourth vehicle is refused at the overlay row (row 1 of claim 5 gives land type *rock*).
- *Falsified if:* type 1 enters a clear cell, type 2 or 3 is refused at clear cells, or the fourth
  vehicle enters an overlay cell.

**L18: the rows of rule 2 and rule 3 (claims 4, 11 and 12).**
- *Setup:* four flat clear maps, each with a band and a destination beyond it. Every overlay type
  named is overridden with land type *clear* and every flag off except the one stated.
  - Map A: every band cell holds an overlay of a type with the crushable flag on. A vehicle of class
    wheel and zone *normal*, and one of class wheel and zone *crusher*.
  - Map B: every band cell holds an overlay of a type with the rock flag on. A vehicle of class wheel
    and zone *normal*. Control run: the same overlay type with the rock flag off.
  - Map C: the band is of rough tiles (code 14); an override gives rough a wheel multiplier of 0 and
    a track multiplier of 100 %. Every band cell holds an overlay of a type with the rubble flag on.
    A vehicle of class track and zone *crusher* (L10 is the same map without the overlays).
  - Map D: the band is of beach tiles (code 10); an override gives beach wheel and float-on-beach
    multipliers of 100 %. A vehicle of class wheel and zone *normal*, and one of class wheel and
    zone *amphibious*. Run D2: the same, with the second vehicle's zone written in mixed letter case
    (for example *aMpHiBiOuS*).
- *Procedure:* order every vehicle to the destination.
- *Observable:* whether each vehicle's position is ever in a band cell, and whether it reaches the
  destination.
- *Expected:* map A: the *crusher* vehicle crosses; the *normal* vehicle does not enter the band
  (category 1). Map B: the vehicle does not enter the band (category 6); in the control run it
  crosses. Map C: the vehicle crosses (category 0 from the rubble row, where L10 refuses it). Map D
  and run D2: the *amphibious* vehicle crosses; the *normal* vehicle does not enter the band
  (category 3).
- *Falsified if:* on map A the *normal* vehicle enters the band or the *crusher* vehicle does not
  cross; on map B the vehicle enters the band, or the control vehicle does not cross; on map C the
  vehicle does not cross; on map D or in run D2 the *normal* vehicle enters the band or the
  *amphibious* vehicle does not cross.

**L19: the zone table, cell by cell (claims 10, 11 and 12).**
- *Setup:* eight land-sided maps, 0 to 6 and 1W, and seven water-sided maps, W0 to W6. Each is
  flat and has a near area and a far area separated by a band. On the land-sided maps the near and
  far areas are clear ground; on maps W0 to W6 they are water. The band's cells are built so that
  their passability category is the map's number (1 for map 1W):

  | Maps | Band cells | Category (claim 11) |
  |---|---|---|
  | 0, W0 | clear ground, no overlay | 0, rule 5 |
  | 1, W1 | clear ground, each cell holding an overlay of a type with the crushable flag on | 1, rule 2 |
  | 1W (land zones only) | clear ground, each cell holding an overlay of a type with both the crushable and the wall flag on | 1, rule 2 (the crushable row comes first) |
  | 2, W2 | clear ground, each cell holding an overlay of a type with the wall flag on (not crushable) | 2, rule 2 |
  | 3, W3 | beach tiles (tile land code 10) | 3, rule 3 |
  | 4, W4 | water tiles (tile land code 9); on map W4 the band is water like the areas beside it | 4, rule 3 |
  | 5, W5 | clear ground, each cell holding a terrain object of one type whose occupation pattern for the theater an override sets to a value other than 7 | 5, rule 4 |
  | 6, W6 | clear ground, each cell holding an overlay of a type with the rock flag on | 6, rule 2 |

  Every overlay type named has land type *clear* and every flag off except the one stated.
  Overrides give the clear, beach and water land types every multiplier 100 %, so claim 10 refuses
  a band cell only for its wall flag (maps 1W, 2 and W2), and rule 3 gives no category 6. Units: eleven
  vehicle types with a driving locomotor and class wheel, one for each movement zone other than
  *water* and *water-beach*, which start in the near area of the land-sided maps; and two ship types of
  class float with the locomotor ships use, of zones *water* and *water-beach*, which start on the
  near water of maps W0 to W6. Overrides set each type's movement zone. Every unit belongs to a
  computer-controlled house and is a team member, ordered through its team's script, as in S-0212
  scenario P3 run 2. One run per map and unit; units may share a run when they are placed far
  enough apart that they never meet.
- *Procedure:* order each unit to a cell of the far area.
- *Observable:* whether the unit moves, whether its position is ever in a band cell, whether it
  reaches the destination, the tick its destination is cleared, whether it is still a team member
  afterwards, and whether its house gives it a new destination. Each unit and map gives one of
  these outcomes:
  - **C**, crosses: it reaches the destination.
  - **S**, the gate passes and the search fails: it never reaches the far side of the band, and it
    leaves its team or its house gives it a new destination (S-0212, claim 23, second row).
  - **G**, the gate fails: it takes no step, its destination is cleared in the tick the path is
    first requested, it stays a team member and it gets no new destination (S-0212, claims 6
    and 23, first row).
  - **P**, the gate passes: C or S, whichever the occupancy rules for the terrain objects give
    (the occupancy specs own them; this scenario does not decide between C and S here).
- *Expected:* a *yes* of the zone table gives C, S or P, and a *no* gives G. S appears where
  claim 10 refuses the wall cells to a zone that the table lets reach them (on map 1W, *crusher*
  and *amphibious-crusher* cross because the wall is crushable):

  | movement zone | map 0 | map 1 | map 1W | map 2 | map 3 | map 4 | map 5 | map 6 |
  |---|---|---|---|---|---|---|---|---|
  | normal | C | G | G | G | G | G | G | G |
  | crusher | C | C | C | G | G | G | G | G |
  | destroyer | C | C | C | C | G | G | G | G |
  | amphibious-destroyer | C | C | C | C | C | C | P | G |
  | amphibious-crusher | C | C | C | G | C | C | G | G |
  | amphibious | C | G | G | G | C | C | G | G |
  | subterranean | C | C | S | S | G | G | G | C |
  | infantry | C | G | G | G | G | G | P | G |
  | infantry-destroyer | C | C | C | C | G | G | P | G |
  | fly | C | C | S | S | C | C | P | C |
  | crusher-all | C | C | C | C | G | G | G | G |
  | water (maps W0 to W6) | G | G | — | G | G | C | G | G |
  | water-beach (maps W0 to W6) | G | G | — | G | C | C | G | G |

- *Falsified if:* a cell expected G shows any sign that the gate passed (the unit moves, leaves its
  team or gets a new destination); a cell expected C, S or P shows the G outcome; a C cell does not
  reach the destination; or an S cell crosses the band. A G outcome in the subterranean or fly
  row where C or S is expected falsifies either this table or S-0212's region rule, which is of low
  confidence for those two zones (S-0212, claim 3); S-0212 decides between the two. On map 1W, a G
  outcome for *crusher* or *amphibious-crusher* would mean that the wall row of rule 2 came first
  (category 2). Map 5 also
  checks rule 4's pattern rule: if the terrain objects gave category 2 instead of 5, the
  *destroyer* row would pass the gate and the *infantry* row would not.

**L20: buildings leave the category alone (claims 5 and 11).**
- *Setup:* three land-sided maps, built like map 0 of L19 (flat clear ground, a near and a far area
  separated by a band, band cells with category 0 by rule 5 when empty), except that every cell of
  the band holds a building, so the band is filled with buildings of the one type of the run
  (several cells per building where its foundation covers them, and no gap). Run A: an ordinary
  building type with the laser-fence and firestorm-wall flags off. Run B: a building type whose
  override sets the laser-fence flag; every section is placed by the map, so each starts in the
  inactive state. Run C: a building type whose override sets the firestorm-wall flag, with the
  owner's firestorm off (as every house starts). The buildings belong to a house other than the
  vehicle's. Each type's override gives every field this spec reads for it, as the other scenarios
  do. One vehicle of class wheel and zone *normal*, which starts in the near area, belongs to a
  computer-controlled house and is a team member, ordered through its team's script, as in L19.
- *Procedure:* order the vehicle to a cell of the far area, in each run.
- *Observable:* the outcome letters of L19 (C, S and G), with the same observables.
- *Expected:* S in all three runs: category 0 lets the gate of zone *normal* pass, and the buildings
  that fill the band stop the search, so the vehicle never reaches the far side, and it leaves its
  team or its house gives it a new destination. The result does not depend on the row of the
  zone table beyond category 0 and *normal*'s *no* for every other category. Reference: the same
  vehicle on map 0 of L19 crosses.
- *Falsified if:* the outcome of any run is G: the vehicle takes no step, its destination is
  cleared in the tick the path is first requested, it stays a team member and it gets no new
  destination. That would mean the buildings gave the cells a category other than 0 (every other
  category refuses zone *normal*), so claim 11's rule 4 and rule 5 are revised.

## Open questions

- Which events switch a laser-fence section between its active and inactive states (claim 11) belong
  to the structures specs. It matters only with `quirks.passability_laser_fence_lost` off.
- Which cells a bridge marks as part of its body (claim 13) belongs to the bridge specs.
- Whether the cliff-back rule (claim 8) turns *water*, *beach* and *ice* into *rock* as read
  (scenario L5 tests only *clear*, *road* and row 1), and what ground level a neighbour that lies off
  the map counts with.
- Which game events check *M* ≠ 0 and the absence of an overlay before putting a mobile object on a
  cell (for example a unit leaving a factory) belongs to the occupancy and production specs.
- The difference between *no* and *never* in the zone table belongs to S-0212.

## Questions for the project lead

- This spec gives the twelve land-type section names that Q-0007 question 1 asks for, but not the
  answers to its other questions. `requests` stays empty, because listing Q-0007 would mark the
  whole request answered at hand-off. Should a partial answer be recorded some other way?
- The switch `quirks.restricted_deck_overlay_indices` (claim 14) defaults to the original. Should it
  exist?
