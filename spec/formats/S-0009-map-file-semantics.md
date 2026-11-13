---
id: S-0009
title: Map file semantics (geometry, terrain streams, waypoint markers, start positions, preview)
topic: formats
revision: 45
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-13
measurements: []
requests: [Q-0010, Q-0016, Q-0018, Q-0023, Q-0045, Q-0046, Q-0419, Q-1805, Q-3064]
supersedes: []
---

# S-0009: Map file semantics (geometry, terrain streams, waypoint markers, start positions, preview)

## Summary

How the original game interprets a map file once it has been split into sections and keys: the
map's footprint and playable area, terrain height, the decoded terrain, overlay and preview values,
the waypoint markers, skirmish start positions, which parts of a map a skirmish game reads in
which observable order, the fields of the map's object lists with the state in which each
pre-placed object starts, the cell tags and map trigger variables, what a cell outside the map
reads as, and the starting units each house receives. Byte-level stream encodings, lighting, tile categories, triggers and the
map's rules overrides are outside this spec (Open questions).

Gameplay requirements specify skirmish only. Claims 47 and 48 additionally specify the house
identity import needed for map inspection without skirmish reassignment, complementing
S-0103 claim 14. This limited import probe does not establish campaign gameplay support.
Network games and campaigns are out of scope, except what a network lobby needs (Q-1805):
the house order of several human players and observers (claim 19) and the position an
observer is shown (claim 22). Claim 19's rules for ties, for the order among observers, for a
network game's random colours and for the place of observers in the house order are design
rules of our engine; the original's place for observers is open (Open questions).

S-0012 claim 9, Formulas F3 and scenario R2 own partial terrain values. Claims 45 and 46
cover standard skirmish starting units; the all-base-units family follows S-0412 claim 13,
including its different placement and draw rules. Sign-off checks these scopes against
S-0412's revision in the same hand-off.

**Resolved S-0012 clarification.** S-0012 revision 2 claim 9, F3 and R2 supply the
requested corrections. Joint sign-off confirms that partial coordinates use available bytes
of a zero coordinate pair only after a complete record, while partial tile numbers after
complete first-record coordinates follow F3 and have the R2 first-record control. Claim 7
and M4 already agree. The historical revision-12 conflict is resolved: the corrections were
already present in approved S-0009 revision 14 and retained in approved revision 15.

**Joint hand-off checks.** Claim 29 and the header-pair Import mapping agree with S-0003
claim 9: unread components are unspecified in the original; only our engine completes them
from the corresponding values in force, with an importer warning. Missing or empty pairs
retain the whole value in force. S-0412 claim 14 and its ordering table already allow the game-start
tile-animation draws of S-0202 claim 14 after secret-lab choices; those choices are not
necessarily the last draws before tick 0. S-0407 claim 14 already agrees with this order and
no longer requests a correction to S-0412. Joint sign-off confirms these current passages,
S-0414's end-test and end-delay rules used by S-0412 claim 11, and the standard versus
all-base-units scope and shared placement-search reuse described above. Claims 28 and 45
also agree with S-0414 claim 1: cleanup removes the house itself, not merely its objects,
so it receives no starting units and takes no defeat test. No further S-0009 scope change
is required. The loading-order table agrees with S-0202 claim 14, including cleanup after
the second pass and before the late phase, and game-start recalculation after crates,
secret labs and end-of-loading tags even with no starting units or an early stop.
Recalculation is not a final-draw boundary: resource-glint setup can draw afterwards
(S-0202 claim 14). S-0202 claim 15 supplies downstream particle and smudge creation-draw
rules at map load and during play, including immediate middle events, with R14 proposed
rather than measured. Subsequent particle updates remain outside that claim's scope.
Joint sign-off checks S-0011 claim 36 and its load-time draw-order table together with
S-0202 claims 14–15, S-0412 claim 14, S-0109 claims 13–14 and 25, S-0401 claim 20,
and this spec's claim 28 and loading-order table. Neither these checks nor this table
treats creation-time particle or smudge effects as taking zero draws. Production-start
draw assignment belongs to S-0404 claim 11, not this loading-order table.
Claim 40 already follows S-0405 claim 18's conditional initial power-switch rule; the map
flag's encoding remains here, and S-0108 claim 13 owns the slot-animation consequences.
Claim 40 follows S-0215 claim 1: set-up queues *guard*, the grand opening during
placement sets readiness, and *guard* becomes current at the building's first per-tick
start. Confirm its placement-time slot effects with S-0108 claim 11 and the set-up
exclusion of the free unit with S-0402 claim 18 at joint hand-off.

**S-0412 claim 7 (Q-1805).** No change is needed. Its statement that no observable shows the
order in which players are resolved holds within its scope, a skirmish with one human. In a
network game, which human on "random" gets which colour slot decides that human's place in
claim 19's house order and so the start positions. Our engine draws the slots of every player
on "random", human or computer, in increasing lobby-slot order (claim 19, design). No
observable against the original fixes that order, because how the original draws them in a
network game is open.

## Terms

Terms used as defined in the shared glossary: *cell*, *lepton*, *height level*, *theater*, *house*,
*country*, *skirmish*, *lobby*, *overlay*, *overlay data value*, *tileset*, *sub-tile*, *slope*,
*cell tag*, *type*, *scenario random stream* (with *draw* and *ranged draw*), *behaviour*,
*follower car*, *leader* and *car chain*, *quirk switch*, *measurement setup*, *engine constant*,
*absent-key default*, *AI house* and *step direction*. The terms below are specific to this spec.

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
- **Visited cell.** A legal cell that the visiting order of claim 3 contains. On every map with
  W ≥ 2 that is every legal cell.
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
  number in that range (S-0202, claim 4). It takes no raw value when the range holds a single
  number, exactly one raw value when the range size is a power of two from 2 on, and may take more
  than one otherwise, because it rejects raw values that fall outside the range.
- **R(a, b).** As in S-0202 (Terms, *ranged draw R(a, b)*): one ranged draw over the inclusive
  range a … b on the scenario random stream, by the range rule. For example R(0, 7) gives one of
  0 … 7 and takes exactly one raw value, and R(0, 1) gives 0 or 1 and takes exactly one raw value.
- **Percent draw.** As in S-0202 (Terms and claim 6): the ranged draw R(0, 99), compared with an
  integer percentage. Its range holds 100 numbers, which is not a power of two, so by the range
  rule one percent draw may take more than one raw value. "A percent draw below 50" holds for 50
  of its 100 equally likely results, 0 … 49.
- **Creation draw.** The one raw value that every creation of an infantry, vehicle, aircraft or
  building object takes from the scenario random stream at the moment of creation, whatever else
  that creation draws (S-0202, claim 10). An object that is never created takes none.
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
  players first (an observer's house among them, placed as claim 19 states), then the neutral
  house, then the special house.
- **Observer.** A human participant whom the lobby sets to watch the game instead of playing it.
  The original still creates a house for an observer. Our engine places it as claim 19 states,
  and the original's place is open. An observer does not count towards the player
  count P of claim 17 and receives no starting units.
- **Passive house.** A house whose country is marked passive for multiplayer games by the
  country's passive field (Import mapping), as that field stands when the test is made (claims 28
  and 33 say which country data each test sees). A passive house takes no start position (claim 19) and
  is spared by the cleanup of claim 28. Which houses are passive depends only on that field.
- **Decoded stream.** The sequence of values that an encoded map block (terrain, overlay type,
  overlay data or preview) yields once decoded. This spec states the order and meaning of those
  values, not the byte encoding (value widths, compression, block layout), which is outside it
  (S-0012). Partial terrain values follow S-0012 claim 9 and Formulas F3, including its
  `quirks.partial_terrain_values` switch, as claim 7 below describes. An incomplete first coordinate pair
  is separately unspecified in the original; our engine applies no record from that block
  and the importer warns. Partial preview pixels follow claim 24.
- **Crate type.** An overlay type that the rules data marks as a crate: an object that gives a
  random bonus to the unit that picks it up.
- **Cell animation.** An animation that the rules data attaches to an overlay type (Import
  mapping) and that starts each time an overlay of that type is placed normally on a cell, at a position
  near that cell (claim 12). Low-bridge end markers and their direct end-row and piece writes
  do not start cell animations (S-0202 claim 14). An overlay type with a cell animation counts as displayable even when
  it has no artwork of its own (claim 12).
- **Cell corner.** The point of a cell with the smallest x and y, at (256·x, 256·y) leptons: the
  cell's top corner on the screen. A point's *place within its cell* is (X mod 256, Y mod 256),
  its lepton coordinates reduced to its own cell.
- **Ground occupant.** An infantry unit, vehicle, aircraft or building that a cell holds on its
  ground layer, not on a bridge deck. A building is a ground occupant of every cell of its
  foundation. Terrain objects, overlays, smudges and animations are not occupants.
- **Terrain recalculation.** As in S-0010 (Terms, and claim 12 for the passes at map load and
  the game-start pass): the step that derives a cell's slope shape and land type from its tile,
  height and neighbours.
- **Tile animation.** As in S-0202 (Terms) and S-0010 (claim 19): the animation that a cell's tile
  names and that the cell starts when it is recalculated.
- **End marker, end row, bridge piece.** As in S-0202 (Terms and claim 10): the overlay types that
  complete a low bridge, the row of cells that an end marker writes, and the cells between two
  end rows.
- **Unset base tile.** The value −1 that a fill base tile takes when no tileset supplies it
  (claim 6). It is an ordinary signed tile number, distinct from the no-tile marker 65535.
- **Stream position.** One of the 512 × 512 places (x, y), 0 ≤ x ≤ 511 and 0 ≤ y ≤ 511, that the
  overlay type and overlay data streams each give one value to (claims 12 and 13). A stream position
  that is a legal cell stands for that cell; the others have no effect.
- **Selected cell.** The cell that a coordinate pair (x, y) read from a terrain record or a
  cell-tag key acts on: the cell whose stream position index 512·y + x equals that of the pair
  (Formulas). For 0 ≤ x ≤ 511 and 0 ≤ y ≤ 511 it is (x, y) itself. A pair with x outside 0 … 511 selects another
  cell, for example (527, 10) selects (15, 11) (quirk `quirks.cell_index_alias`). A pair whose
  index lies outside 0 … 262143 selects no cell.
- **Mark.** The integer that each of the start positions 0 … 15 carries during start position
  resolution (claim 29).
- **Base cell.** The cell around which a house's starting units are placed: for a player, and for
  a neutral or special house that holds a start position (claim 28), its start cell (claims 19,
  20 and 29).
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
- **Health value, facing value.** The integers of a pre-placed object's entry from which its
  starting health (claim 37) and starting facing (claim 39) are derived. They are not the health
  and facing themselves.
- **Required field.** A field of an object entry that claim 35 lists as required. The original does
  not survive an entry that matches an owner and a type but lacks one (claim 36).
- **Upgrade.** A building placed onto another building, its host, to change it. Here only the
  upgrades a map entry names matter (claim 40); what an upgrade does belongs to the building specs.
- **Flight height.** The height above the terrain at which an aircraft type flies, a field of its
  type (aircraft specs, none yet; a pointer only).
- **Stand-in cell.** The one cell, existing on no map, that the original uses wherever a rule
  refers to a cell that does not exist (claim 44; Edge cases, cell-tag key whose selected cell is
  not legal).
- **Start area.** The 8 × 8 block of cells that a random start cell needs clear, with the examined
  cell as its corner (claim 18).
- **Placed vehicles.** The vehicles that the map's vehicle list has put on the map, in entry order,
  numbered from 0 (claim 41).
- **Follow values.** The follow-index fields of the map's vehicle entries that give one, in entry
  order, numbered from 0, counting only the entries claim 41 names.
- **Tag.** As in S-0011 (Terms, *tag* and *tag definition*): the runtime object of one of the
  map's tag definitions, which carries that definition's triggers. A *tag name* in a map entry
  names a tag definition (claims 31 and 41).
- **Starting units.** The units a house receives around its base cell when a skirmish or
  multiplayer game starts, after the map is loaded (claim 20): its base unit, if any, and the
  units bought with its budget (claims 45 and 46).
- **Base unit.** In standard skirmish mode, the one vehicle a house receives first, at its
  base cell, when the game's bases setting is on (claim 45). The all-base-units family
  instead grants the vehicles specified by S-0412 claim 13.
- **Budget, start pools.** The budget is the amount of credits' worth of starting units every
  house receives besides its base unit. The budget pool is the set of types whose costs set it,
  and a house's start pools are its vehicle pool and its infantry pool, from which its units are
  drawn (claim 45).
- **Placement search.** The search of claim 46 for a cell next to a base cell on which a starting
  unit can be put.
- **Game's tech level.** One integer for the whole program run, with which the budget pool of
  claim 45 compares each type's tech level. The original copies it once, when the program starts,
  from the base rules' multiplayer tech level (Import mapping), which is 10 when the base rules
  omit it. Nothing later reaches that copy: not the language rules file, the game-mode file or the
  map's rules overrides, and not a later game of the same program run. S-0006 (claim 10) restores
  the *rules value* of that key at every game start, but the game's tech level is a copy taken at
  program start and does not follow the rules value. The original's skirmish setup screen offers
  no tech-level choice, and no other way of changing it in a skirmish game is known. Our engine
  does the same.
- **House's tech level.** One integer per house, with which that house's start pools (claim 45)
  compare each type's tech level. Every house created from the lobby, whether a human player's,
  an observer's or a computer player's, takes the game's tech level when it is created. The
  neutral and special houses have tech level 1, a fixed value of the program, since the map's
  per-house definitions are not read in skirmish (claim 26). The two tech levels therefore differ
  only for those two houses, and that matters only when one of them receives starting units
  (claim 28).
- **Campaign mission.** A single-player game played on a mission of the game's campaigns, as
  opposed to a skirmish. Campaign missions are outside this spec except where a rule differs
  between them and skirmish: crate overlays (claim 12), the player name setting (State and
  ordering, read set) and the alternative home waypoint (Import mapping).

## Behaviour

### Map footprint and playable area

1. **Legal cells.** A cell (x, y) exists on a map of size W × H exactly when
   W < s ≤ W + 2H and −W < d < W, with s = x + y and d = x − y. The map therefore has 2H screen
   rows, s = W + 1 … W + 2H. Rows alternate between W cells (rows with s − W odd) and W − 1 cells,
   for H·(2W − 1) legal cells in total. The first two numbers of the map size entry (an origin)
   are read and ignored: the diamond always starts at the coordinate origin as above. Cells outside
   this set do not exist. A terrain record whose selected cell (Terms) is not a legal cell, and an
   overlay value at a stream position that is not a legal cell, have no effect. A cell-tag entry
   whose selected cell is not a legal cell is not simply ignored by the original, whose result for
   it is unspecified; our engine ignores it (Edge cases). A pair with x outside 0 … 511 can still
   select a legal cell (Terms).
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

3. **Visiting order.** The *visiting sequence* is ordered by screen row from top to bottom
   (increasing s, starting at s = W + 1), and within one screen row by increasing x (left to right
   on screen). Its first cell is (1, W). The rules of this spec that take cells one after another
   use this sequence, and it is observable through them: the initial fill takes its draws cell by
   cell in this sequence (claim 6), and the vein rebuild takes its seeds in reverse sequence
   (claim 14). On every map with W ≥ 2 the sequence contains every legal cell exactly once. Edge case:
   on a map with W = 1 the legal cells are (k, k) for k = 1 … H, one per odd-numbered screen row,
   and the rows in between hold no cell. There the visiting sequence contains only (1, 1); the cells
   (2, 2) … (H, H) are not in it, so the rules that use the sequence leave them out (quirk
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
   These are the local area's current values, not permanently its imported values: a trigger
   action can replace all four during play (S-0524 claim 8).
   - Backing: `measurable` (scenario M3) for the flat and plateau inequalities,
     S-0524 claim 8 and its scenario C7 for runtime replacement.
     The extra half-row for a slope is `unmeasured` here: M3 uses flat plateau cells and
     supplies no slope-edge probe that distinguishes that adjustment.
   - Confidence: high for the inequalities. Which features use which variant is claim 5.

5. **What the local area limits.** Nearly every gameplay use of the local area applies the
   height-aware test. Cells outside the local area still exist and are drawn (terrain outside it
   is visible beyond the playable border), but play does not extend into them. Subsequent
   usable-area tests use a replacement local area when a trigger changes it (S-0524 claim 8).
   Effects on buildings excluded by a shrink remain open there. The claim has two parts
   with different backing:
   - **5a.** The height-aware test limits the extent of the minimap, whether the mouse cursor
     offers an order on a cell, object placement checks, the "find a free nearby cell" search and
     ore growth.
     - Backing: `measurable` (scenarios M3 and M3b)
     - Confidence: medium
   - **5b.** The height-aware test also governs the choice of a map-edge cell for arrivals from
     off-map. For paradrop planes and aircraft retreat, S-0522 claims 4 and 5 own the cell
     selection and house-edge choice: the selected edge cell is outside the height-aware usable
     area, not inside it. Retreat behaviour is S-0501 claim 34.
     The height-aware test also limits the choice of the cells that two superweapons affect when fired (the superweapon
     that turns infantry into mutants and the one that takes control of units), aircraft firing
     positions, cliff destruction and the pathfinding zones. The preview image generated for a map
     uses the plain test. A few features use the plain test in some situations and the
     height-aware test in others; they are listed as an open question.
     - Backing: `measurable` for paradrop edge selection through S-0522 claims 4 and 5,
       scenario P3; retreat behaviour inherits S-0501 claim 34's backing.
       The remaining features are `unmeasured` here. A scenario for any of them needs an expected result
       for the feature itself (where an arrival appears, which cells a superweapon affects, which
       route a unit takes, what the generated preview shows), and those results are defined only by
       the topic specs that own the features. Most of them are not written yet, and the one that is
       (S-0212, pathfinding) does not yet say which variant its zones use. Each owning topic spec
       states and measures its variant (Open questions).
     - Confidence: medium

### Terrain and height

6. **Initial fill.** The initial fill gives every visited cell (Terms) height level *L0* (the map's
   base level setting, default 0, reduced modulo 256 into the signed range −128 … 127) and
   sub-tile 0. Terrain records (claim 7) replace the values they set, so a value of the fill
   remains wherever no record sets it. A visited cell's fill tile depends on the map's fill
   setting. If the setting is the word for water (compared case-insensitively), each visited cell
   gets the *water base tile* plus a ranged draw over 0 … 3. The range 0 … 3 has a power-of-two size, so each such ranged draw takes
   exactly one raw value from the scenario random stream. One draw is taken per visited cell, in the
   visiting order of claim 3: H·(2W − 1) draws when W ≥ 2, and 1 draw when W = 1. For any other
   value, or when the setting is absent, every visited cell gets the *clear base tile* and no random
   value is drawn. On a map with W = 1, the legal cells that are not visited cells stay blank cells: tile
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
     the map's own theater's tile count is replaced by the no-tile marker 65535 on most cells by the
     first terrain recalculation (claim 8; S-0010 claims 7 and 12); what is painted for such a
     number is S-0010 claim 24's. When the base tiles come from the map's own theater, which happens
     when the theater loaded before is the same theater, every fill tile built on such a base tile
     is of that kind.
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
   present. A partial coordinate pair has the table's deterministic effect only after an
   earlier complete record. Partial tile numbers and missing one-byte values have their
   stated effects even in the first record when its coordinate pair is complete.
   A nonempty packed block supplying fewer than four expanded bytes for its first
   coordinate pair has unspecified coordinates in the original; our engine applies no record
   from that block and warns, without a switch, as S-0012 claim 9 specifies. An absent block simply keeps
   the initial fill.

   | How much of the last record is present | Effect |
   |---|---|
   | Nothing (the data ends between two records) | none; the result is the same as with a (0, 0) record at that point |
   | x only | as if the coordinates were (x, 0), with the rows below. For x = 0 that is the end of the stream; for 1 ≤ x ≤ 511, (x, 0) selects no legal cell, so no cell changes |
   | The coordinates, selecting no legal cell | none |
   | The coordinates, selecting a legal cell, but no tile number | the selected cell's tile becomes the no-tile marker 65535; its other values keep the values already in force |
   | The coordinates, selecting a legal cell, and one or more later complete values | the complete values are applied; the values not present keep the values already in force |
   | Part of a coordinate pair, after an earlier complete record | Available low-order bytes replace those of a zero coordinate pair (S-0012 claim 9). Three coordinate bytes can select a legal cell, whose tile then becomes 65535 and whose other values remain in force. Our engine reproduces this by default and warns |
   | Complete coordinates selecting a legal cell, but only part of its tile number | Available low-order bytes replace those of the preset tile number 65535 (S-0012 Formulas F3); the resulting tile number is translated normally. Our engine reproduces this by default and warns |

   Missing sub-tile, height and ice-growth values retain the selected cell's values already
   in force, including values from an earlier duplicate record, not necessarily the fill.

   No stock map is cut off. Our engine reproduces the cases the table specifies, and the importer
   warns about a terrain stream that ends without a (0, 0) record.
   - Backing: `measurable` (scenario M4, including its height variant for the outright level and a
     negative level, and its selected-cell variant)
   - Confidence: high (medium for duplicates, negative levels and the cut-off records, which no
     stock map exercises). The partial-value refinements carry S-0012 claim 9's medium
     confidence. Partial coordinates, zero/two/three tile bytes and missing sub-tile/height
     are `measurable` through M4 and S-0012 R2. The exact one-tile-byte result, missing ice-growth
     retention and unspecified first coordinates are `unmeasured` for the reasons in S-0012
     claim 9; failure recording does not establish a deterministic original result.

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
     clear tile. That is the choice of artwork only. What is painted for a cell that holds such a
     number is S-0010 claim 24's (for a number at or beyond the tile count the original's picture
     is unspecified there, and our engine paints the clear tile).
   - Negative numbers come from the initial fill of claim 6, or from a terrain record whose tile
     number is negative. They are not caught by that rule. The original's result is
     unspecified. Our engine treats a negative tile number like the marker 65535 (a departure),
     and the importer warns about a terrain record with a negative tile number.

   What these numbers mean for play (land type, slope, the cells that keep an out-of-range number)
   belongs to S-0010 (claims 7 and 8).
   - Backing: `measurable` (scenario M4b) for the identity case. The picture for a number at or
     beyond the tile count is `unmeasured`: S-0010 claim 24 leaves the original's picture
     unspecified, so M4b's second part records it without a conformance expectation.
     The translation is `measurable`
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
    defaults to 0 when absent), the game reads no overlay at all, from either overlay stream (quirk
    `quirks.no_overlay_below_format_2`).
    - Backing: `measurable` (scenario M6)
    - Confidence: high

12. **Overlay type stream.** The decoded stream holds 512 × 512 values, each an integer
    0 … 255, one per stream position (Terms), in order of increasing y and, within one y, of
    increasing x: counting from 0, the value at index 512·y + x belongs to stream position (x, y).
    The value 255 means "no overlay". Any other value is an index into the rules' overlay type list, which makes the list *order* part of
    the format. An overlay is created on a cell only when all of these hold: the cell is legal, the
    overlay type has artwork or a cell animation, the game is a campaign mission or the type is
    not a crate type, and the cell's slope shape (S-0106) as it stands at that moment is at most 4
    or the overlay type is the one at list index 178. A cell's slope shape is 0 until its first
    terrain recalculation (S-0010, claim 12), so while the overlay stream is read the slope
    condition can only refuse an overlay on a cell that an earlier placement has already
    recalculated, which only a low bridge's end row or pieces do (S-0202 claim 10). In play it
    applies to every overlay placed. Crate overlays placed on a map therefore never appear in skirmish or
    multiplayer games (quirk `quirks.skirmish_drops_map_crates`). Where the decoded stream is shorter than a position, that position has no
    overlay, and the importer warns about a short stream. Creating the
    overlay applies its normal placement effects (wall linking and similar). Some of them take draws
    from the scenario random stream while the map loads, in the order of the overlay type stream:
    an end marker that completes a low bridge, an overlay type with a cell animation, and the
    recalculation of the cell that every placement causes, which can start the cell's tile
    animation (S-0202, claim 10 and its table "Draws during map loading", states those draws and
    the low-bridge rules they depend on).
    Placing the vein overlay takes no draw of its own here (claim 14 covers veins). For the overlay types
    at list indices 24, 25, 237 and 238, creation leaves the cell's own overlay data value as it
    was before the overlay was created.
    **Cell animations.** Each time an overlay whose type has a cell animation (Terms; Import
    mapping) is placed normally on a cell (x, y), while the map loads and in play alike, one animation of
    that type starts. Low-bridge end markers, their end-row writes and their bridge-piece writes
    start no cell animation, even when the marker or written type names one (S-0202 claim 14).
    Their cell recalculations can still start tile animations: an end-row write takes any
    such tile-animation draws without cell-animation draws, and a piece's raw type-choice value
    precedes its recalculation's tile-animation draws and the next piece's raw value. For
    ordinary placement, the cell animation is as follows:
    - **Where.** At the world position (256·x + 512, 256·y + 512, *z*) leptons: the cell's centre
      moved by 384 leptons along +x and by 384 along +y, which is the cell corner (Terms) of cell
      (x + 2, y + 2). On the screen that is the overlay cell's own column, one and a half cell
      heights lower. The height *z* is the terrain surface height (S-0106) at the centre of the
      overlay's cell (x, y), not at the animation's position.
    - **How it is created.** An ordinary creation in the sense of S-0109, with no parameter of its
      own beyond its position. It has no loop delay, so its start event (S-0109 claim 13) happens
      at once, during map loading included. It has no repeat factor (S-0109 claim 7 with *m* = 1),
      its creation does not ask for reversed play (the type's own reverse setting still applies,
      S-0109 claim 8), and it is not a trigger-played animation, so its start event has its start sound and its resource
      chain reaction. Every effect of the start event, and of the middle event that it produces
      when μ = 0 (S-0109 claim 14), takes place at the position above: the chain reaction acts on
      cell (x + 2, y + 2), not on the overlay's cell. What those effects are, and their draws,
      belong to S-0109 and to the specs of those effects.
    - **Order.** The animation's creation draws (S-0202 claim 10, *animation creation draws*)
      come before the draws of any tile animation that the recalculation of the overlay's cell,
      caused by the same placement, starts. The overlay is already in its cell, and that
      placement's recalculation of the cell has not yet taken place, when the animation is
      created.
    - Backing: `measurable` (scenarios M6, including its short-stream save, and M6c; scenario M6d
      for the slope condition; scenario M6e for the cell animation's position, its start event,
      the chain reaction's cell, and (part D) its draws coming before those of the tile animation
      of the same placement; S-0202's scenario R8, part F, for the creation draws themselves,
      and R12 for the low-bridge exclusion and its tile-animation draw order)
    The list-index-178 slope exception is `unmeasured` here: M6d does not place that type
    on a cell already recalculated to a slope above 4. The cell animation's elevated z,
    repeat factor, reverse request and start event with μ = 0 are `unmeasured` here: M6e
    uses level 0 and does not distinguish those creation parameters; their consequences
    require additional animation setups. The observable position, sound and chain-reaction
    claims keep M6e's backing.
    - Confidence: high (the crate exclusion and the four data-preserving indices are medium until
      measured; medium for the slope condition and its exception; medium for the cell animation's
      position and creation; high for the low-bridge cell-animation exclusion)

13. **Overlay data stream.** Same size and order. Every legal cell's overlay data value (0 … 255) is
    set from its value in the stream, whether or not the cell has an overlay. When the map has an overlay data
    stream, its values are final with respect to overlay creation and its placement effects: no
    data value that they set survives on any legal cell. On every cell that the vein rebuild of
    claim 14 clears or re-patterns, the data value that the rebuild leaves replaces the stream's. Where the
    decoded stream is shorter than a position, the value is 0, and the importer warns. When the map has no overlay data
    stream at all, every cell keeps the data value it had after overlay creation (0 unless
    placement set one).

    An immediate crater request from a cell animation or a tile animation started by overlay
    placement acts on the resource levels then present, before this data overwrite (S-0202
    claim 15). The overwrite replaces the data value even if the request emptied that cell,
    but cannot restore an overlay type that the request removed. A tile animation first
    created by the first whole-map terrain recalculation instead acts on the imported data.
    The second and game-start recalculation passes act later still. Thus the animation's
    creation group determines which resource state its immediate crater request sees, not
    merely the fact that it starts during loading. Its particle and smudge creation draws
    follow S-0202 claim 15, not an assumed zero-draw rule.
    - Backing: `measurable` (scenario M6, including its short-stream save, for the stream; scenario
      M6c variant A for the absent stream; S-0202 R14 C for crater timing, retained data and
      removed overlay types)
    - Confidence: high

14. **Veins are rebuilt from their seeds.** After the overlay streams have been read and the first
    terrain recalculation pass has run (S-0010, claim 12), every cell that holds the vein overlay
    (Terms) loses it: its overlay becomes none and its data value 0. The cells whose data value was
    48 or more before this, the *vein seeds*, then get veins again. The seeds are ordered by the
    reverse of the visiting order of claim 3, and the order shows in two ways: the seeds' draws come
    in that order, and the placement test of a seed sees the veins put back for every seed before it
    in that order (the seeds later in the visiting order) and none for the seeds after it. A seed that passes the vein placement
    test gets the vein overlay back as the centre of a vein pattern, and its edge neighbours are
    re-patterned, so that a flat seed whose four edge neighbours are flat, clear and empty ends up
    with veins on all five cells. The new data values come from draws on the scenario random
    stream, and S-0202 (claim 10, *vein patterns*) states those draws, their order within one seed and the data
    values they give. The test looks at the cell and its four edge neighbours, and it sees the
    cleared cells, the terrain after the first recalculation pass, every other overlay of the map
    and the veins put back for the seeds before it in that order. The test, the neighbours a passing
    seed re-patterns and the connection pattern that sets their data are S-0202's (claim 10, *vein
    patterns*). A seed that fails the test stays empty.
    Consequences: a vein cell whose data value is below 48 keeps a vein after loading only when a
    passing seed next to it re-patterns it; and the vein cells' data values after loading are the
    ones these draws give, not the ones the overlay data stream gave (claim 13). The rebuild creates
    no veinhole; the veinhole object comes from another overlay type (Terms).
    - Backing: `measurable` (scenario M6b for the clearing, the threshold and the re-patterned
      neighbours; S-0202's scenario R8, part B, for the draws, their order within a seed and the
      reverse visiting order across seeds)
    - Confidence: medium (the test and the re-patterned neighbours carry S-0202's confidence)

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
    the start waypoint markers, positions after the gap shift down and the missing ones become random
    (quirk `quirks.start_waypoint_gap`).
    Example: waypoint markers 0, 1, 3 and 5 are defined and there are 4 players. Then K = 2,
    N = 4, the list is [wp0, wp1, wp3, random], position 2 is waypoint 3 and position 3 is a random
    cell (waypoint 5 lies beyond N − 1 = 3, so it is not used).
    - Backing: `measurable` (scenario M8)
    - Confidence: high

18. **Random start cells.** Each missing entry is found by taking two ranged draws from the scenario
    random stream (each by the stream's range rule, which may consume more than one raw value).
    First u is drawn uniformly from 0 … (W + H − 11), then v uniformly from 10 … (W + H − 11).
    The candidate is x = v + 1, y = u + 11. For W + H ≥ 21 that makes x range over
    11 … W + H − 10 but y over 11 … W + H, an asymmetry (quirk
    `quirks.random_start_asymmetric_range`; smaller maps: see Formulas). The
    candidate need not be legal. The game then looks for the entry with the nearby-cell search of
    S-0212 (claim 25), centred on the candidate, for locomotion class *track* and movement zone
    *normal*, and with none of that search's optional parts: no reach region, no deck mode, no
    height condition, no further test and no reference cell. A cell qualifies, in place of that claim's single-cell conditions,
    when it is a usable cell (the height-aware test of claim 4) and every cell of its *start
    area* is clear. The start area of cell (x, y) is the 8 × 8 block of cells (x + i, y + j) for
    i, j = 0 … 7: it has the examined cell as its corner and extends towards increasing x and y. A
    cell of the start area is clear when it passes the class rule (S-0211 claim 10) for class
    *track* with movement zone *normal*, a bridge cell being judged on its deck, and its
    occupancy record (S-0209) is empty, the deck's for a bridge cell. So bridge cells are allowed,
    a wall overlay blocks, and no other overlay plays a part. Only the corner is tested against
    the usable area. The other 63 cells may lie outside it, and a cell outside the map is read as
    claim 44 says (in our engine always clear). The ring order, the scan order, the limit of 24
    collected cells, the visibility test and the ring cut-off are those of S-0212 claim 25. With no
    reference cell, that claim takes the preferred cell at position (*t* mod *n*) of the
    collection, with *t* the tick number. The tick number is 0 throughout loading, so the result
    is the first preferred cell in collection order: the first collected cell that passes the
    visibility test, or the first collected cell when none does. Among the qualifying cells of
    the first ring that has one, the scan order decides, not the distance to the candidate. The
    search takes no draw. When no cell qualifies, the candidate fails.
    The entry is the cell found for the first candidate whose search succeeds. Each candidate
    whose search fails still uses up its pair of draws, and the original sets no limit on the
    number of candidates.
    On a map with no such area the original never finishes loading. That is bad data, and our engine
    does not reproduce the endless load (a departure, no quirk switch): after 1000 consecutive failed
    candidates (a fixed limit of our engine, not a constant of the original, so the outcome is
    deterministic) it refuses to start the
    game and reports that no start cell could be found, naming the map. The importer also warns when
    it can already tell, from the map's size and terrain alone, that no 8 × 8 clear area exists. A
    map whose search succeeds before that limit is unaffected, and its draws are the original's.
    A map that could succeed only after the limit may instead be refused.
    In the early phase (claim 20) every legal cell is a blank cell, so every start area is clear.
    The entry is then the candidate itself when it is a usable cell (ring 0, which S-0212 lists
    twice), and otherwise the first usable cell, in scan order, of the first ring that has one,
    since on a flat map every cell passes the visibility test.
    - Backing: `measurable` (scenario M9; scenario M9d for the start area's corner and direction,
      the scan-order choice and the settings of the search)
    - Confidence: high for the draws and ranges, the search settings, the start area and the tick
      number 0; medium for the early-phase consequence, which leans on the visibility test of
      S-0212 claim 25 on a flat map

19. **Assigning positions to players.** Houses are created in a fixed order: human players in
    increasing order of their colour slot, then computer players in lobby-slot order, then the
    neutral house, then the special house. This holds for any number of human players, and every
    non-observer human player's house comes before every computer player's. Observers come between
    the non-observer human players and the computer players. That place is our engine's rule
    (design); the original's place for observers is open (Open questions).
    - A human whose colour was left on "random" is placed by the colour slot S-0412 claim 7 gave
      it, exactly as if it had chosen that slot: the colour draws come before the house order is
      fixed. S-0412 claim 7 places the draws for a
      skirmish game. In a network game the original resolves "random" colours at the same point
      of the game start. Which peer makes those draws, and in which
      order, is open (Open questions).
      - Backing for the draws coming first in a network game: `unmeasured`, because no measurement
        setup yet runs a network game on the stock executable; M10 run 3 would back it then, since
        a game of several humans is a network game. Confidence: low (unmeasured).
    - The colour draws never give a player a slot that another player holds (S-0412 claim 7), so
      they cannot produce a tie between two humans. Whether two humans can hold the same chosen
      colour in the original is open (Open questions). In our engine such a tie goes to the
      earlier lobby slot (design).
    - Observers hold no colour slot (S-0412 claim 7). They come after every human player with a
      slot and before every computer player. Among themselves they come in lobby-slot order in
      our engine, the same on every peer (design). The original's order among several observers
      is not known (Open questions).
    - In our engine, S-0412 claim 7's colour-slot rule also gives the slots of a network game's
      players on "random": every such player, human or computer, draws in increasing lobby-slot
      order (design), so every peer gets the same slots. How the original draws them in a network
      game is open (Open questions), so this order cannot yet be checked against the original.

    Players who chose a start position get that list entry. Players left on "random" then receive
    positions one at a time in house order:

    | Positions taken when the player's turn comes | Position the player gets |
    |---|---|
    | none | the result of one ranged draw over 0 … N − 1 (which takes no raw value when N = 1 and may consume more than one when N is not a power of two; Terms, range rule) |
    | at least one | the free position whose *distance sum* to all taken positions is largest. A tie, or all sums equal, goes to the lowest-numbered position. The distance sum is defined under Formulas: per pair, a tabulated square root of the squared cell distance, truncated to an integer. For two legal cells that is the truncated Euclidean distance for every pair less than 129 cells apart; from 129 cells on it is sometimes one less. A start cell far outside the map follows the wider rule below |

    Each position, once given, counts as taken for later players. The player's base cell, where its
    starting units are placed, is its start cell. Passive houses (Terms) get no position, whatever
    their place in the house order.
    In the early phase the farthest-sum rule uses the start cells of list 2 (claim 20); the
    late-phase assignments of claims 28 and 29 use list 3.
    Because of the tabulated square root, two positions whose exact distances differ can get equal
    terms, and the tie then goes to the lower-numbered position (quirk
    `quirks.table_root_distance`, owned by S-0204). The first such pair of distances is 128 and 129
    cells.
    **Start cells far outside the map.** A start waypoint marker can name a cell far outside the
    map (claims 1 and 15), with a coordinate anywhere in −32768 … 32767. For a pair that involves
    such a cell, three more rules of Formulas (*Distance sum*) apply, and our engine reproduces the
    first two. First, each coordinate difference is reduced modulo 65536 into −32768 … 32767
    before it is squared, so a difference of 32775 counts as −32761. Second, each term, once
    truncated to an integer, is reduced the same way, so a term of 32768 or more counts as a
    negative number. Third, when every free position's sum is then below 0, the original's choice
    is unspecified. Our engine gives the lowest-numbered free position, and the importer's warning
    about a start waypoint on a non-legal cell covers the case (Edge cases; a departure).
    - Backing: `measurable` (scenario M10; scenario M10b for the tabulated square root; scenario
      M10c for the two reductions of a pair with a start cell far outside the map). The case in
      which every sum is below 0 is `unmeasured`: the original's result is unspecified, so there is
      nothing to compare with.
    - Backing for the house order of several humans and observers: `unmeasured` until a
      measurement setup can run a game of several human players on the stock executable. M10's
      colour-order sub-test is the scenario proposed to back it then: the colour-slot order of
      humans (runs 1 and 2), the place of a human on "random" given its slot (run 3) and the
      place of one observer (run 4, which would bear on the original's place for observers,
      open). Our engine's tie order, observer order, observer place and network colour draws are
      our engine's choice (design); the original's behaviour for those four is open (Open
      questions).
    - Confidence: high for the rule and the tabulated square root; medium for the two
      reductions; low for the colour-slot ordering of humans and the place of a human on
      "random" (unmeasured), though each is a single rule with no exception. The place of
      observers is our engine's rule and gets no rating against the original, whose place is
      open.

20. **Start positions are resolved in two phases, from three start lists.** The *early phase*
    runs after the map size, the header block, the local area and the waypoint markers are read, and
    before the initial fill (claim 6). The *late phase* runs after the terrain, the overlay and all
    pre-placed objects exist. Each phase builds start lists by claims 17 and 18, each build a fresh
    list with new random entries and new draws (quirk `quirks.double_start_resolution`; one game mode places players its own way at the
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
      house by house, with the draws of claim 46, and after the last house one ranged draw over
      0 … 65535 whose value has no effect (not taken when generation stops early, claim 45). The creation draw is the one every creation of such an
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
      clear, and claim 18 gives what the search makes of it)

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

    - Backing: `measurable` (scenario M15's skirmish branch). The branch with an already
      taken position is `unmeasured`: its discriminating setup needs a human player and a
      separate human observer. M10's colour-order sub-test, run 4, would back it once a
      measurement setup can run a game of several human players on the stock executable.
    - Confidence: low

### Map selection data and preview

23. **Lobby player limit.** The lobby allows as many players as there are defined start waypoint markers
    among 0 … 7, counted in any pattern (gaps allowed). The lobby finds the entries by the same
    spelling rule as the game (claim 15: the plain decimal number, so an entry named "01" is not
    found). It uses a different "undefined" test from the game itself: an entry counts when it is
    present with any value other than −1, so a stored 0 *counts* here although the game treats it
    as undefined (quirk `quirks.lobby_counts_zero_waypoint`). If none counts, the limit
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
    - Backing: `measurable` (scenario M13: its channel-order map for the red, green and blue order
      and the row order, its shortened stream for the short-stream result)
    - Confidence: high for the pixel order. Low for the short-stream result: the original treats
      the reading of a shortened stream as having failed, yet it still shows the decoded pixels,
      and nothing observable is known to follow from that failure.

25. **Preview start markers come from the header, not from the waypoint markers.** The numbered markers the
    selection screen draws over the preview come from a separate header block. It holds a marker
    count N_h, an origin (X0, Y0), an extent (Wh, Hh) and N_h coordinate pairs. Markers are drawn
    only when 1 ≤ N_h ≤ 8, labelled 1 … N_h. Pair i is drawn at the preview position given under
    Formulas. Header pairs 1 … 8 never affect where players start, and the header count affects it
    only through claim 29. A map whose
    header disagrees with its start waypoint markers shows markers in the wrong places (quirk
    `quirks.preview_markers_from_header`, whose switched-off behaviour is claim 25a). When the
    header block is absent, the count, origin and extent read as −1, so no markers are drawn. An
    extent of 0 (Wh = 0 or Hh = 0) with 1 ≤ N_h ≤ 8 makes the original crash when it draws the
    markers. With the switch on, our engine draws no preview markers for such a map and changes nothing
    else about it. An importer warns (Edge cases). Exactly the pairs
    1 … N_h are read, whatever the count: a pair numbered above N_h is never read, even when the map
    holds it. With N_h above 8, the pairs from the ninth up to the N_h-th are therefore read, and
    those up to the sixteenth act as start position marks that can change which player holds which
    start position (claim 29, which also covers pairs from the seventeenth on; quirk
    `quirks.header_pairs_as_start_marks`).
    - Backing: `measurable` (scenario M13, including its extent-0 variant and its wrap variant
      for the integer width of the marker formula; scenario M13b variants C and D for which pairs
      are read)
    - Confidence: high for the marker source and the −1 defaults. Medium for the formula and the
      extent-0 crash.
    - **25a. Markers with the switch off.** With `quirks.preview_markers_from_header` off, the
      header block has no effect on the preview markers. Its effects on start positions (claim 29)
      are unchanged. The markers show the start list of claim 17 instead, as built for a game of L
      players, where L is the lobby's player limit counted from the map (claim 23), and with
      `quirks.start_waypoint_gap` as set (so N = max(L, K) in claim 17's terms). For each start
      position p in 0 … N − 1 whose entry is
      the cell of a defined waypoint marker, and that cell is a legal cell (claim 1), a marker
      labelled p + 1 is drawn at the position given under Formulas (preview marker position,
      waypoint form). The preview image is taken to cover the whole map footprint: the origin and
      extent of that formula are those of the legal cells. A position whose entry is random gets no
      marker. A start waypoint marker stored as 0, which the lobby counts
      (`quirks.lobby_counts_zero_waypoint`), is undefined for the start list, so it never has a
      marker of its own. A
      position whose entry is a defined waypoint marker naming a non-legal cell is a real start
      (claim 1), but its cell lies outside the footprint and so off the preview image, and it gets
      no marker either. The marker graphic, its label and its corner shift (9, 6) are those of
      claim 25. When no position qualifies, no markers are drawn. Only which positions have a
      waypoint entry matters: drawing the markers finds no random start cell and takes no draw.
      - Same markers as with the switch on: take a map whose defined start waypoint markers among
        0 … 7 are exactly 0 … L − 1, all on legal cells, and whose header block has N_h = L, the
        origin and extent of the waypoint form and, for pair i, the projection (u, v) of
        waypoint i − 1's cell (Formulas). Both settings of the switch draw the same L markers there.
        On such a map the start list's waypoint entries do not depend on the number of players,
        so the markers are right for every player count. On a map with a gap in its start
        waypoint markers they are right for a game of L players.
      - Backing: `unmeasured` (a design choice of our engine for the switched-off state; the
        original has no such state). Scenario M13's agreement variant measures, with the switch
        on, that a header written by the waypoint form puts each marker over its waypoint's place
        in a footprint-sized preview image.
      - Confidence: high (fixed by this claim). Medium for the agreement variant's prediction.

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
    country name other than the neutral and special houses' own names do not appear in skirmish
    (quirk `quirks.skirmish_drops_country_owned_objects`).
    The same applies in multiplayer games. In addition, in a skirmish or multiplayer game a
    pre-placed **structure** whose owner is a human player's house is dropped, while vehicles,
    aircraft and infantry with that owner appear (for multiplayer games this is a departure, Edge
    cases). A dropped object is never created, so it takes no creation draw (claim 20).
    - Backing: `measurable` (scenario M14, including its human-owned structure and
      vehicle; scenario M26 variant G for the dropped object taking no creation draw).
      Network-specific owner rules are `unmeasured` here because network games are out of
      scope. The online-name rule is `unmeasured`: online games need an online game
      service, which the measurement setup does not include.
    - Confidence: medium; low for the multiplayer-game statements and the online-name rule, which
      are `unmeasured` network-game claims

28. **Unused-house cleanup.** If the game has no computer players and fewer human slots (observers
    included) than the rules' maximum player count (Import mapping: an integer taken from the base
    rules only, S-0006 claim 2, and 8 when they omit it), a cleanup runs after all map objects are placed
    and after the second whole-map terrain recalculation (S-0010 claim 12), before the late
    phase of claim 20. (One special game mode skips it; it is not identified here,
    see Open questions.) In our engine the computer players are the AI houses (glossary, *AI
    house*) that fill the lobby's computer slots, so the cleanup runs only in a game with no AI
    house. A game whose players are all AI houses, which the decision of the project lead of
    2026-10-11 allows (S-0414), has computer players and so no cleanup. Call a house
    *eligible* when it is neither human-controlled nor passive.
    The cleanup removes a house from the game exactly when the house is eligible and the house just
    before it in house order (claim 19) was not removed (quirk `quirks.cleanup_skips_next_house`).
    In a run of eligible houses that are next to each other in house order, the first, third, fifth and so on
    are removed and the others are kept. With a removed house go:
    - **its pre-placed objects.** Removing a ground occupant recalculates each cell it occupied,
      including a building's foundation cells. Under S-0010 claim 19 a missing tile animation
      may start at that removal, taking S-0202 claim 10's applicable creation and immediate
      start-event draws during the cleanup, before the late phase (S-0202 claim 14).
      A cell with a tile-animation start already recorded since the second pass starts no
      new one and takes no new creation draws. Removing an object that occupies no ground
      cell does not imply a ground-cell recalculation.
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
    starting units there, after the cleanup, by claims 45 and 46, with its own tech level 1 (Terms,
    house's tech level); when no type qualifies for its country, claim 45's early stop applies at
    it. When every position below N is already taken at its turn, the third unspecified case of
    claim 29 applies to it.
    - Backing: `measurable` (scenario M16's skirmish variants B and G for a computer
      player preventing cleanup, the neutral house's late-phase position and its tech level 1).
      The removal condition without computer players, alternating removals, trigger removal,
      special-house starting units and the maximum-player-count boundary are `unmeasured`
      here: M16's older discriminating variants require network games, now deferred, and no
      equivalent in-scope setup is established. Cleanup's position after the second pass,
      removal-cell recalculations and conditional tile-animation draws are likewise
      `unmeasured` here, in agreement with S-0202 claim 14: its R11 is now deferred
      because the former proposal reused M16 variant A's network setup, and no
      discriminating no-computer-player skirmish fixture is established. That AI houses count as computer players
      is our engine's reading of the condition (design), not an oracle observation.
    - Confidence: high that a removed house is gone with its objects; medium for the condition's
      computer-player branch (M16 variants B and G); low for the removal condition without computer
      players, alternating removals, the special house's starting units and the
      maximum-player-count boundary, for cleanup after the second pass and before the late phase,
      and for removal-cell recalculations and their conditional tile-animation draws, each
      `unmeasured` because its setup is a network game; low for the trigger removal (including that
      tags created at the end of loading escape it), the use of the map's override, the late-phase
      position and its starting units

### Skirmish start positions: header pairs as start marks

29. **Header pairs beyond the eighth act as start position marks.** Every start position
    0 … 15 carries a *mark*, an integer. The mark −1 means the position is free. A mark equal to
    the house number (Terms) of a non-passive house means that house holds the position. Any other
    mark means the position is taken by no player. Without header pairs beyond the eighth, every
    mark is −1 when the early phase starts, and claims 19 and 20 apply as written. When the header
    count N_h is 9 or more, each header pair i with 9 ≤ i ≤ min(N_h, 16) that is present in the map
    (an *applied pair*) supplies the marks of positions 2(i − 9) and 2(i − 9) + 1: a readable
    first or second number becomes the mark of its corresponding position. If a component
    cannot be read, the original's mark for that component is unspecified; our engine keeps
    that position's mark as it stood before the pair was applied and the importer warns
    (S-0003 claim 9). A pair in that range that is absent, and every pair numbered above N_h
    (which is not read, claim 25), leaves those two marks as they are. A missing or empty pair
    keeps the whole pair's value in force. Unread-component completion is a departure.
    For pairs 9 to 16 the fallback components are the two marks at that phase, so early-
    and late-phase completion can differ.
    The pairs are applied twice, with these consequences:
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

    Three further cases are unspecified in the original: a house that holds a position numbered N or above,
    header pairs from the seventeenth on (read only when N_h ≥ 17), and a house that needs a position by the farthest-sum
    rule when every position below N is already taken (marks can cause this). In the first two cases
    our engine ignores the mark or pair concerned. In the third, the house gets no start position
    and no starting units. These are departures, and the importer warns in all three cases.
    - Backing: `measurable` (scenario M13b: variants A to D for the marks and which pairs are read,
      variant E for the references and for the highest-numbered position a house holds, variant F
      for a mark that names no house). Unread-component results in the original are
      `unmeasured`: unrelated internal state gives no reproducible value to predict.
      M13b variant G checks our engine's deterministic completion and warnings only.
    - Confidence: medium (low for the references; high for the unread-component departure)

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
      not established (Open questions). Until it is, our engine lets the early phase see the
      completed value as well, exactly like every later use, and the importer warns about a
      local area entry with fewer than four numbers (an interim rule, revised when the open
      question is settled).
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
    states again at a later game start is not established (Open questions). Until it is, our
    engine clears every variable state at every game start, so every game behaves like the first
    game of a program run (an interim rule, revised when the open question is settled).

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

    So the early phase can miss passive fields that this game supplies, and can see passive fields
    that an earlier game of the same program run left in force (quirk
    `quirks.early_phase_stale_country_data`).
    - Backing: `measurable` (scenario M16b: variant A for the map override, variant B for the
      carry-over from an earlier game, variant C, with a named test file, for the game-mode file)
    - Confidence: medium

### Pre-placed objects: entry fields and initial state

Claims 34 to 43 give the field positions of the map's object lists and the state in which each
pre-placed object starts. The owner rules are claim 27's. Each object enters play when its entry is
read, in the placement order of claim 43, while the game's set-up at game start is in progress
(S-0108 claim 6). Field positions are numbered from 1. Import mapping lists the sections.

34. **Splitting an entry.** Every entry of the vehicle, infantry, aircraft and building lists, and
    of the smudge list, is one comma-separated value. The entry's key is only a name and has no
    effect. The entries are read in entry order, and each carries the value its key has under
    S-0002 claim 4 (as for cell tags, claim 31). Only the first 127 characters of the value are
    read, so fields that start later are absent. The value is cut at every comma. A run of commas
    with nothing between them, or a comma at the start, yields no field at all, so an empty field
    does not count and every later field moves one place forward
    (quirk `quirks.object_empty_fields_collapse`). Spaces belong to the field they are in. A numeric
    field is read leniently as its leading integer, like a cell-tag key (claim 31): its value is
    the integer written by its longest beginning of the form *spaces (possibly none), an optional
    sign, one or more digits*, and anything after that beginning is ignored. The result is a signed
    32-bit integer: a number outside −2147483648 … 2147483647 wraps as S-0003 claim 4 says for the
    decimal form. A field that does not begin in that form (no digit right after its leading
    spaces and sign) reads as 0. A name field is compared as written, spaces included.
    - Backing: `measurable` (scenario M19, variant D for the empty field and the 127-character
      limit)
    - Confidence: high (medium for the 127-character limit)

35. **Field positions.** At format version 4 and above, the fields are:

    | Field | Vehicles | Aircraft | Infantry | Buildings |
    |---|---|---|---|---|
    | 1 | owner | owner | owner | owner |
    | 2 | type | type | type | type |
    | 3 | health value (claim 37) | health value | health value | health value |
    | 4, 5 | cell x, cell y | cell x, cell y | cell x, cell y | cell x, cell y of the foundation's top-left cell (claim 38) |
    | 6 | facing value (claim 39) | facing value | standing position number (S-0208 claim 5) | facing value |
    | 7 | behaviour name | behaviour name | behaviour name | tag name |
    | 8 | tag name | tag name | facing value | computer-sell flag |
    | 9 | veterancy value | veterancy value | tag name | computer-rebuild flag |
    | 10 | group number | group number | veterancy value | switched-on flag |
    | 11 | bridge flag | recruit-no flag | group number | upgrade count |
    | 12 | follow index | recruit-yes flag | bridge flag | searchlight value |
    | 13 | recruit-no flag | — | recruit-no flag | first upgrade name |
    | 14 | recruit-yes flag | — | recruit-yes flag | second upgrade name |
    | 15 | — | — | — | third upgrade name |
    | 16 | — | — | — | computer-repair flag |
    | 17 | — | — | — | nominal flag |

    Below format version 4, fields 4 and 5 are replaced by one field, a cell number decoded with
    base 128 (Formulas, object cell), and every later field moves one place forward. A flag is set
    when its integer is not 0. Fields after the last one in the table are ignored.

    The fields up to and including field 6 (field 5 below format version 4) are *required*. Every
    later field is optional: when it is absent, the object takes the default of Import mapping. A
    behaviour name and a tag name are matched by S-0215 and S-0011 (claim 41). The veterancy value
    is S-0311's (claim 11 there).
    - Backing: `measurable` (scenario M19: variants A to F for the positions of owner, type,
      health, cell, facing and standing position, variant G for the behaviour name, variant H for
      the tag name, variant I for the veterancy value; scenario M20 for format versions below 4).
      The positions of the bridge flag, the switched-on flag, the upgrade fields and the follow
      index are measured through scenarios M23 (variant A), M21 (variants C and D) and M24. The
      positions of the group number, the two recruit flags, the computer-sell, computer-rebuild
      and computer-repair flags, the nominal flag and the searchlight value are `unmeasured`: no
      rule of this spec acts on them, and the topic that would show their effect has no spec yet or
      belongs to another spec (claims 40 and 41; the repair flag's effect is S-0314's), so a
      misplaced field would show nothing
    - Confidence: high for fields 1 to 6 and for the veterancy, tag and behaviour positions; medium
      for the other optional fields

36. **Type lookup and short entries.** The type field is compared, without regard to letter case,
    with the identifiers of the types of that list's kind only: vehicle types for the vehicle list,
    aircraft types for the aircraft list, and so on. A name that is not a type of that kind (an
    infantry type named in the vehicle list, a misspelt name, an empty type field) drops the entry
    silently, like an unmatched owner (claim 27), and a dropped entry takes no creation draw. An
    entry that matches an owner and a type but lacks a required field is unspecified in the
    original, which does not survive it. Our engine drops the entry and the importer warns (a
    departure). The dropped entry takes the draws that the original has taken for it when it fails:
    one creation draw for a vehicle, aircraft or infantry entry, none for a building entry.
    - Backing: `measurable` (scenario M19, variant C for the wrong-kind and letter-case
      cases; scenario M26 variant G for the dropped entry taking no creation draw). The
      short entry is `unmeasured`: the original does not survive it, so it yields no recordable
      result.
    - Confidence: high for the lookup, medium for the short entry

37. **Starting health.** Let *S* be the type's strength (its full hit points) and *H* the entry's
    health value. The health value is on a scale where 256 means full strength. For a building, a
    value above 256 is first replaced by 256. The object starts with health *h*, given by Formulas
    (object health): *H* × *S* / 256, truncated toward zero, then raised to full strength *S*
    whenever it comes out above *S* − 3 (quirk `quirks.near_full_health_rounds_up`). The truncated
    result is a signed 32-bit integer: a truncated quotient outside −2147483648 … 2147483647 wraps
    (Formulas). Only a health value far outside 0 … 256 reaches that, and for a building only a very
    negative one. Our engine reproduces the wrap. So a health value never gives more than full strength. A value
    of 256 or more gives full strength for a building always, and for the other kinds whenever the
    quotient does not wrap. A value a little below 256 can give full strength too: with *S* = 100,
    a value of 254 gives 100, not 99. The low end depends on the kind (a wrapped quotient can be
    negative):
    - An infantry unit whose result is below 1 starts with 1.
    - A vehicle or aircraft whose result is 0 starts with 1. A negative result is unspecified in
      the original.
    - A building takes the result as it is, even 0 or negative, which is unspecified in the
      original.

    In the unspecified cases our engine starts the object with 1 and the importer warns (a
    departure). The ratio *h* / *S* is the health ratio that S-0108 (claims 2 and 12) compares with
    the condition ratios, so a building whose health value puts it at or below the damaged
    threshold is drawn damaged from the first tick.
    - Backing: `measurable` (scenario M19, variants A and B; variant J for the wrap and for the
      building's replacement of a value above 256)
    - Confidence: high for the scale, the truncation and the near-full rule; medium for the low
      end, the wrap and the building's replacement

38. **Position.** Let (x, y) be the entry's cell, after the coordinate wrap of Formulas (object
    cell).
    - A vehicle, an aircraft or a building stands on the centre of the cell (x, y), at world
      position (256·x + 128, 256·y + 128) leptons. For a building, (x, y) is the top-left cell of
      its foundation: its bounding rectangle covers the cells (x + i, y + j) with
      0 ≤ i < w and 0 ≤ j < h, where w × h is its size. Its occupied foundation cells are
      S-0406 claim 2's, including that claim's missing-cell and zero-size exceptions.
      The image anchor of S-0108 claim 8 lies at the top corner of cell (x, y). That same screen point is the building's draw point in the sense of S-0105
      (Terms): its voxel turret's turret anchor is that point moved by the type's turret-animation
      displacements.
    - An infantry unit stands at the standing position whose number is field 6, within cell (x, y)
      (S-0208 claim 5 and its table P). Its world position is
      (256·x + u, 256·y + v) leptons, where (u, v) is that position's row of S-0208 table P.
    - A vehicle stands on the ground of its cell, or on the bridge deck (glossary) when its bridge
      flag is set. The height of an infantry unit is S-0208's (claim 5). An aircraft stands landed
      on the ground of its cell when the cell is usable by the height-aware test (claim 4), and is
      placed in the air at its type's flight height above the terrain otherwise.
    - At the game's set-up, a pre-placed vehicle or aircraft is placed whatever already occupies
      its cell: two vehicle entries on one cell put both vehicles there. Where infantry may stand
      is S-0208's and S-0209's.
    - An entry whose cell is not a legal cell (claim 1) is unspecified in the original. Our engine
      takes the creation draw, drops the object and the importer warns (a departure).
    - Backing: `measurable` (scenario M19 for the cell and the standing position, scenario M21 for
      the building's top-left cell and variant E for its draw point, scenario M23 for heights and
      the shared cell)
    - Confidence: high for the cell centre and the standing position; medium for the building's
      top-left cell and draw point, the heights and the shared cell; low for the aircraft placed
      in the air

39. **Facing.** A pre-placed vehicle, aircraft or building starts with body facing 256 × (*v* mod
    256) facing units, where *v* is its facing value and mod is the non-negative remainder (glossary
    claim G1 states this for 0 ≤ *v* ≤ 255; other values reduce the same way). A vehicle's turret
    starts at its body facing (S-0206 claim 5, placement), and so does an aircraft's. A building's
    turret facing, the one facing a building turns (S-0206, S-0310), starts at the same value, so a
    voxel turret is drawn at that facing on the first tick (S-0105) and a sprite turret aims with it.
    An infantry unit takes its facing value (field 8) the same way, and faces 0 when the field is
    absent.
    - Backing: `measurable` (scenario M21 for the building turret, scenario M19 for the other kinds)
    - Confidence: high for vehicles, medium for buildings, aircraft and infantry

40. **Initial state of a pre-placed building.** A building placed by the map is complete at once:
    it never enters the *construction* state and shows no build-up. Its placement sets its body
    state to *idle* at once, at stage *a* of the idle range, with the delay that S-0108 claim 6
    gives an immediate change (it is made during the set-up), and the set-up queues *guard*
    (S-0215 claim 1). Its grand opening runs during placement, after *guard* is queued,
    and sets the ready mark. At the end of set-up its current behaviour is still *none*.
    At its first per-tick start *guard* becomes current and is processed in that same turn,
    whether the building is armed or unarmed, turreted or not, or upgraded (S-0215 claim 1).

    Placement starts the set-up *idle* and *active-1* … *active-4* slot animations of
    S-0108 claim 11. It also has that claim's grand-opening slot starts: the ordinary
    non-refinery starts, the refinery's fill-level slot, the superweapon slot when applicable,
    and a sprite turret's *turret* slot. A refinery retains the set-up idle animation even
    though its grand opening does not itself start idle. These starts use the health already
    established by claim 37, subject to S-0108 claim 12's slot-condition swaps and claim 11's
    sprite-turret exception. Thus a damaged building with a sprite turret can show normal
    slot animations without regaining health. Subsequent slot changes follow S-0108,
    including claim 13 for power effects and claim 15 for the refinery fill. A pre-placed
    building receives no free unit: the grand opening occurs, but its free-unit grant is
    suppressed during set-up (S-0402 claim 18).

    The building fields that change the start:
    - **Switched-on flag** (default set): when it is 0, the building starts switched off only
      when its type has a base drain above 0 or the needs-power flag. Otherwise it stays
      switched on, including a pure producer with neither condition. Firestorm-wall types
      are not excluded from this startup rule, unlike later power orders. A set or absent
      flag leaves the building switched on (S-0405 claim 18 and scenario W13; later orders
      follow its claim 10; S-0108 claim 13 owns the slot-animation consequences).
    - **Upgrades.** With upgrade count *c* (default 0), each of the first min(*c*, 3) upgrade name
      fields that names a building type is placed as an upgrade of this building: an object of that
      type, for the same owner, at the same cell and facing, right after the building. A name that
      matches no building type adds nothing. A count above 3 is unspecified in the original. Our
      engine uses 3 and the importer warns (a departure). An upgrade name field that is absent
      does not mean "none": it keeps the type that the same field named in the most recent earlier
      entry of the building list that gave that field, and none when no earlier entry gave it
      (quirk `quirks.upgrade_fields_carry_over`). An entry dropped for its owner or type (claims 27
      and 36) gives no field. The upgrades start at full health.
    - **Searchlight value** (default 1): for a building whose type has a searchlight, the
      searchlight's behaviour. What the values mean belongs to the lighting topic (no spec covers
      searchlights yet, a pointer only).

    The computer-repair flag is read by automatic building repair (S-0314 claim 13), subject
    to the house skill-level gate of S-0314 claim 8 and the further conditions of its claims
    12 and 15. Human-controlled houses, and AI houses in our engine, need no repair flag.
    An uncaptured building of the neutral or special house needs it. The computer-sell and
    computer-rebuild flags steer the original's computer players, whose base logic is outside
    the specs. Automatic skirmish sales ignore the computer-sell flag (S-0314 claim 14):
    clearing it does not prevent a sale under that rule. The *neutral* and *special* houses
    (glossary, *AI house*, *Player houses only*) keep the original's rules for owners that are not
    human-controlled. Their house skill level is 0, as for a human-controlled house, because the
    program's setup gives a level above 0 only to houses of players, so S-0314's gate treats
    their buildings as it treats a human-controlled house's: with the repair-and-sell threshold
    above 0, the repair flag has no effect on a building of either house, whatever its value; with
    a threshold of 0 or below, S-0314 claims 12 to 15 own the outcome for every house. Of the two
    other flags, no rule of play is known that acts on a building of the neutral or special house
    (a passive house also runs no computer logic); whether a non-passive neutral or special house
    (claim 28) runs the computer players' base logic on its buildings is not established
    (Open questions). Our engine reads all three flags and keeps them as map data; it applies the
    repair flag only through S-0314 and acts on the sell and rebuild flags in no house, because
    AI houses follow the unit rules of human-controlled houses (decision of the project lead,
    2026-10-09, refined 2026-10-10 for the two non-player houses). What the nominal flag does in
    the original is not established here, and no rule of play is known to depend on it. An AI
    house may read the three computer flags and choose to reproduce the original's treatment of
    the building through ordinary orders.
    - Backing: `measurable` (scenario M21: variant A for the complete idle start, the slot starts
      and the damaged variant, variant B for the sprite turret slot, variant C for an eligible
      switched-off start, variant D for upgrades and the carried-over field; S-0405 W13 for
      the startup power-state conditions and firestorm-wall cases; S-0215 B36 for the
      queued/current distinction, ready mark and first per-tick start; M21 F for the
      refinery and superweapon slots and the absence of a free unit; S-0314 H13 and H15 for the repair flag's effect and
      automatic-start pacing; S-0314 H14 for automatic sales ignoring the sell flag).
      The sell and rebuild flags' effects through the original's computer players' base
      logic are `unmeasured`: no scenario of this spec observes that out-of-scope logic.
      S-0215 B36 remains proposed, not a recorded measurement. Firing alone does not
      separate *guard* from other firing behaviours; the recorder must distinguish
      current from queued behaviour.
    - Confidence: high for the complete start in the idle state and first per-tick start of
      *guard*; medium for the placement-time slots and absence of a free unit,
      the switched-on flag and the upgrades; low for the carried-over upgrade field.
      High for the repair-flag condition and automatic sales ignoring the sell flag
      (S-0314 claims 13 to 15); medium for the other computer-flag effects.

41. **Other object fields.**
    - *Behaviour name* (vehicles, aircraft, infantry): the behaviour the object starts with, by
      the names of S-0215 (Import mapping there). An absent field, or a name that matches none,
      means *none*.
    - *Tag name*: the tag the object carries, matched as S-0011 claim 2 says. The tag is created
      at this object's entry if it does not exist yet (S-0011 claim 34). An absent field attaches
      nothing.
    - *Veterancy value*: S-0311 claim 11. Absent: the object keeps the experience its type and
      country give it.
    - *Group number* (vehicles, aircraft, infantry): read and kept. Its effect is not established
      (Open questions).
    - *Follow index* (vehicles): makes vehicles *follower cars* of a *leader* (glossary). This
      spec gives only which vehicle becomes a follower car of which. How a follower car is sent
      after its leader is S-0212's (claim 26), and how it copies its leader's speed fraction is
      S-0207's (claim 10). A vehicle's leader can be any placed vehicle of the list, wherever its
      entry stands. The links are worked out from two numbered sequences (Terms), both counted
      from 0:
      - The *follow values*: one for each vehicle entry that matches an owner and a vehicle type
        (claims 27 and 36) and gives a follow-index field, in entry order. An entry dropped for
        its owner or its type gives none. An entry without the field gives none either; after the
        collapse of claim 34 that is an entry that ends before field 12 (field 11 below format
        version 4). An entry whose vehicle is created but not placed still gives its value; in
        our engine that is an entry whose cell is not legal (claim 38).
      - The *placed vehicles*: all the vehicles that the whole vehicle list puts on the map, in
        entry order, and no others. Starting units (claim 20) are never among them, and the
        vehicles of a house that the cleanup of claim 28 later removes are among them.

      The placed vehicle numbered i takes the follow value numbered i. A value n with
      0 ≤ n < V, where V is the number of placed vehicles, makes it a follower car of the placed
      vehicle numbered n, which becomes its leader. Any other value (negative, or V or more) gives
      it no leader. Consequences:
      - When every placed vehicle's entry gives the field and every entry that gives it is placed,
        each vehicle takes the value of its own entry. Maps written with every field are of this
        kind.
      - When an earlier entry lacks the field, the later values move forward onto earlier
        vehicles: the vehicle numbered i takes the value of the (i + 1)-th entry that gives one,
        which need not be its own (quirk `quirks.follow_index_by_place`). For example, with three
        placed vehicles whose entries give no field, n = 2 and n = −1, the first vehicle becomes a
        follower car of the third, the second vehicle has no leader, and the third vehicle's
        value is the unspecified case below.
      - n names a placed vehicle, not an entry: entries dropped for their owner or type are not
        counted.
      - The rule makes no exception for n = i, so a vehicle can become its own leader, and links
        can form a loop. S-0212 claim 26 sends cars only after a leader that is not itself a
        follower car, so a vehicle that is its own leader, and every car whose leader lies in a
        loop, is never sent after anything.

      A placed vehicle numbered at or beyond the number of follow values takes a value that is
      unspecified in the original. That happens whenever some placed vehicle's entry lacks the
      field and no unplaced vehicle's value makes up for it, and then it affects the last placed
      vehicles. Our engine gives such a vehicle no leader, and the importer warns (a departure).
      The importer also warns about a vehicle list in which some entries give the field and others
      do not, about a vehicle that is its own leader and about a loop of links. What becomes of a
      follower car whose leader the cleanup of claim 28 removes is not established (Open
      questions). Until it is, our engine removes the link, so that car has no leader (an interim
      rule).
    - *Recruit-no and recruit-yes flags*: in the original they mark whether its computer players
      may take the object into their teams. In our engine the recruitable property of S-0215
      (claim 20) has no effect (glossary, *AI house*), so our engine keeps these flags as map data
      and no engine rule acts on them. An AI house (glossary) may read them and choose to
      reproduce the original's use of them through ordinary orders.
    - Backing: `measurable` (scenario M19: variant G for the behaviour name's field and a name that
      matches, and for an unmatched name not being read as that behaviour; variant H for the tag name; variant I for the veterancy value and its absent
      case; scenario M24 for the follow index, observed through S-0212 claim 26). Group number and the recruit flags are `unmeasured` here:
      their observables belong to specs that do not exist yet or to behaviour glasnost does not
      reproduce. That an unmatched behaviour name gives *none*, rather than some other behaviour
      that is ready to change, is `unmeasured`: a mobile actor in *none* makes the placement
      fall-back of S-0215 and rests in a behaviour that follows orders, so it shows the same as any
      such behaviour, and no scenario of this spec can tell them apart. The loop and the vehicle
      that is its own leader are `unmeasured`: they follow from
      the link rule and S-0212 claim 26, and a car that is never sent shows nothing.
    - Confidence: high for behaviour, tag and veterancy; medium for the follow index (which value
      a vehicle takes, the counting of n). The car whose leader is removed is an open question, not
      a claim.

42. **Terrain objects and smudges.**
    - **Terrain objects.** Each entry of the terrain-object list places one terrain object. The
      entry's key is a cell number, read as a leading integer and decoded with base 1000 at format
      version 4 and above, base 128 below it, with the coordinate wrap (Formulas, object cell). The
      entry's value is the terrain object's type name, compared without regard to letter case. The
      two "no tag" markers of claim 31 (Import mapping) and an empty value place nothing. A name that is not yet a terrain
      object type creates one (S-0004 claim 3, creation outside the rules layers), which is then
      placed. The object stands on the centre of its cell, at (256·x + 128, 256·y + 128) leptons,
      with its type's full strength as its starting health. A terrain type's strength, including
      the game-wide tree strength it takes when no rules layer gives it one, is S-0102's (claim 23,
      *Strength of a terrain type*). A terrain object type that the entry's name creates reads no
      rules section, because every rules layer has been read by then (S-0004 claim 3), so its
      strength stays unset: such an object starts with health −1 and shows image frame 1 (S-0102
      claim 23). A map's rules override section for an existing terrain type is a rules layer, so
      a strength it gives reaches every object of that type that the map places (the overrides are
      read before the objects). Terrain objects are not among the kinds that take a creation
      draw (Terms). A cell that is not legal is unspecified; our engine places nothing there and
      the importer warns (a departure).
    - **Smudges.** Each entry of the smudge list has the fields type, cell x, cell y and piece
      number, read as in claim 34. The cell is always two fields, whatever the format version,
      with the coordinate wrap. An entry whose type field names no smudge type (compared without
      regard to letter case) or that has no x field places nothing. An entry whose piece number is
      present and not 0 places nothing: a multi-cell smudge is written once, at its origin, with
      piece 0. Otherwise the smudge covers the w × h cells (x + i, y + j), 0 ≤ i < w and
      0 ≤ j < h, where w and h are its type's width and height, and it is placed without any test
      of whether those cells may take a smudge. An entry with an x field but no y field is
      unspecified in the original; our engine places nothing and the importer warns (a departure).
    - Backing: `measurable` (scenario M22: variant A for the terrain objects, variant B for the
      smudge fields and covered cells, variant C for the untested placement, variant D for the
      starting health through the image frame). One part is `unmeasured`: the terrain object type
      created from an unknown name, because such a type has no artwork and no rules data, so
      nothing on screen shows it or its health −1, and whether it blocks its cell belongs to the
      terrain-object rules, which have no spec yet
    - Confidence: high for the terrain-object key and value and the starting health; medium for
      the smudge fields, the covered cells and the untested placement, and for the health −1 of an
      object whose type the map creates

43. **Placement order.** The map's objects are placed kind by kind: terrain objects, then
    vehicles, then aircraft, then infantry, then buildings (each building's upgrades directly after
    it), then smudges, each kind in entry order. This is the order of S-0203 claim 3 and of the
    draw table under State and ordering. It is also the order in which the objects are first added
    to the drawing order, so wherever drawing keeps an order of insertion (S-0102 claim 6: equal
    sort keys keep their relative order), objects of one kind follow entry order and the kinds
    follow the order above.
    - Backing: `measurable` (scenario M23, variant C)
    - Confidence: high for the placement order, medium for its use as the insertion order of
      drawing

### Cells outside the map

44. **What a cell outside the map reads as.** Wherever a rule reads a cell that is not a legal
    cell (claim 1), it reads the stand-in cell (Terms). The cases that matter at map load are an
    edge neighbour across the map's edge in a terrain recalculation (ground and ramp smoothing,
    S-0010 claim 10, and the cliff-back rule, S-0211 claim 8), a cell of a start area (claim 18),
    an edge neighbour in the vein placement test (S-0202 claim 10) and an end-row or piece cell of
    a low bridge (S-0202 claim 10). In our engine the stand-in always reads as a blank cell
    (Terms): the no-tile marker 65535, sub-tile 0, height level 0, flat (slope shape 0), clear
    land, no overlay (overlay data 0) and no objects. So in smoothing an off-map neighbour holds
    the no-tile marker, which lies in no tile range, and is flat; in the cliff-back rule its ground
    level is 0, so it puts a cell behind a cliff only when that cell's own level is −4 or lower; a
    start area's off-map cells are clear; and the vein placement test passes an off-map neighbour.
    In the original the stand-in is one shared cell whose state rules can also change, and no
    rule is known that resets it. At map load the only known change comes from a low bridge whose
    end row or pieces reach outside the map: each such cell gives the stand-in that end-row or
    piece overlay and recalculates it (S-0202 claim 10), and every later read of any off-map
    cell, in the same game or in a later game of the same program run, sees that state. Our
    engine never changes a cell outside the map, so every read sees the blank cell (a departure,
    no switch). The
    importer warns when an end row or a bridge piece of a map's low bridge would fall outside the
    map.
    - Backing: `measurable` for the blank-cell reading (scenario M25: smoothing at the map's edge
      and the cliff-back rule at level −4). `unmeasured` for the original's changes to the shared
      stand-in: no scenario here arranges an off-map low-bridge write followed by a read
      whose visible result distinguishes the changed stand-in from a blank one. Our engine's
      departure is not itself evidence about the original
    - Confidence: high that every off-map read goes to one stand-in that starts as a blank cell,
      medium that low bridges are the only change at map load (changes in play were not looked for)

### Starting units

45. **Which starting units a house receives.** In standard skirmish mode, after the late phase (claim 20), each house in house order (claim 19) that is neither passive nor an observer's receives starting units around its base cell (Terms). A house that the cleanup of claim 28 removed is no longer in the game and receives none; passivity here is judged by the country data in force for the late phase (claim 33). The all-base-units family is an exception: S-0412 claim 13 gives its starting-unit rules, which replace the standard-mode rules here.
    - **Base unit.** When the game's bases setting (a setup value, Import mapping) is on, the house
      first receives one vehicle: the first type in the rules' base-unit list whose owner list
      includes the house's country. The base unit is not paid from the budget. When no type of
      the list qualifies, or the base unit cannot be placed (claim 46), the step stops early
      (below).
    - **Budget.** Every house gets the same budget *B* = trunc((trunc(*n* / 2) + *C*) / *n*) × *U*
      (Formulas, *Starting-unit budget*; trunc rounds toward zero), where *U* is the game's
      unit-count setting (a setup value) and *C* the sum of the costs of the *n* types of the budget pool (*n* taken as 1 when
      the pool is empty). The budget pool holds every vehicle type and every infantry type that is
      allowed to start in multiplayer games, whose owner list includes the country of at least
      one house that is not passive, and whose tech level (the type's field, Import mapping) is at
      most the game's tech level (Terms), with the vehicle types of the base-unit list left out.
    - **Start pools.** A house's vehicle pool holds, in type registration order (S-0004), every
      vehicle type that is allowed to start in multiplayer games, whose owner list includes the
      house's own country, whose tech level is at most the house's tech level (Terms) and which is
      not in the base-unit list. Its infantry pool holds the infantry types that meet the same
      conditions, the base-unit list aside. Tech levels are compared as signed integers, so a type
      whose tech level is negative always meets the tech-level condition. For a player's house the
      two tech levels are the same value. A neutral or special house that receives starting units
      (claim 28) compares with its tech level 1, so its pools hold only types of tech level 1 or
      lower whose owner lists include its country; when both are empty and *B* > 0, the early
      stop below applies at that house.
    - **Buying.** The house buys budget units one at a time, for as long as its remaining budget
      *R* (in credits, *B* before the first purchase) is above 0 and fewer than 20 of its
      placements have failed. A positive *R* left when buying ends is given to the house as
      credits. Each purchase follows this table:

      | Part of one purchase | Rule |
      |---|---|
      | Pool | the vehicle pool when *R* > trunc(*B* / 3) and the vehicle pool is not empty; the infantry pool otherwise |
      | Type | the pool's entry at the index given by one ranged draw R(0, pool size − 1) |
      | Creation | the unit is created (its creation draw) and then placed by claim 46 |
      | Placement succeeds | *R* falls by the type's cost |
      | Placement fails | the unit is removed again, its creation draw stays taken, and the failure counts towards the 20 |

    - **Early stop.** The whole starting-unit step stops at a house in three cases: the bases
      setting is on and no type of the base-unit list qualifies for the house; its base unit
      cannot be placed (claim 46); or *B* > 0 and its vehicle pool and infantry pool are both
      empty. That house and every house after it then receive no budget units (the houses before
      it keep theirs; in the third case this house keeps its base unit, and in the second case
      its base unit is removed again after taking its creation draw), and the final ranged draw of
      claim 20 is not taken. Our engine does the same.
    - **Behaviour.** In the original the starting behaviour depends on whether the house is
      human-controlled: a starting unit of a human-controlled house starts in the *guard*
      behaviour and one of any other house in *area guard* (S-0215). The other houses are the
      computer players' and the *neutral* and *special* houses (a house reaches this step only if
      it is not passive, claim 28), which are not human-controlled. Our engine starts the starting
      units of every player house, human or AI, in *guard* (AI houses follow the unit rules of
      human-controlled houses, decision of the project lead, 2026-10-09; glossary, *AI house*); an
      AI house may give them *area guard* through ordinary orders (S-0213). The units of a
      neutral or special house keep the original's behaviour: *area guard* (glossary, *AI house*,
      *Player houses only*, decision of the project lead, 2026-10-10). Their experience at
      creation is S-0311's.
    - Backing: `measurable` (scenario M26 for the pools, the budget, the vehicle-or-infantry rule
      and the type draws; variant B for the base unit and the bases setting; variant E for the
      failure limit and the credits left over; variant F, with a named test file, for the early
      stop when no type of the base-unit list qualifies; variant H for the game's tech level, its
      source, its absent-key default, the signed comparison and the house's tech level of a
      player; variant I for the early stop on empty start pools; scenario M16 variants B and G for
      the neutral house's tech level 1 and its *area guard* start).
      Budget arithmetic that wraps is `unmeasured` here: M26's positive stock costs and
      small unit count do not reach the signed-integer boundary; no scenario here separates
      a wrapped budget from an unbounded one.
      The special house's tech level and starting behaviour are `unmeasured` here: M16 D,
      the setup that separates them, is a deferred network game. The second early-stop
      case, a base unit that cannot be placed, is
      `unmeasured`: it arises only on a base cell that is not a legal cell, where the original's
      result is unspecified and our engine's rule is a departure (claim 46). It stops the step in
      the same way as the measured cases.
    - Confidence: high for the budget formula, the pools, the one-third rule, the failure limit,
      the type draw and the early stop, and that a player's house has the game's tech level; medium
      for the left-over credits, for the setup values' path from the setup screen (Import mapping),
      for the game's tech level being copied once at program start and changed by nothing later,
      and for the neutral house's tech level 1; low for the special house's tech level and starting
      behaviour, `unmeasured` because the setup that separates them is a network game

46. **Where starting units are placed, and their draws.** In standard skirmish mode, all placements of this step happen while the map is still loading. Putting an object on a cell then ignores the cell's land type and occupants (the entry test that play applies is skipped during loading). So putting a unit directly on its base cell, or on a cell that the placement search below has accepted, is not known to fail in any established case. Apart from a base cell that is not a legal cell (the base-unit bullet below), a placement fails only when the search finds no cell that accepts the unit. The all-base-units family is an exception: its placement rules are S-0412 claim 13's, not the placement rules below.
    - **The base unit** is put directly at the centre of its base cell, with body facing 0
      (S-0204), with no search and no draw. Only if that direct placement fails does the placement
      search below run for it, with first radius 1. If the search fails as well, the base unit is
      removed and the step stops early (claim 45). On a legal base cell the direct placement
      succeeds in every established case, so the base unit's search and its draws do not occur.
      A base cell that is not a legal cell (a start waypoint marker on a non-legal cell, claims 1
      and 17) is unspecified in the original (Open questions). Our engine treats the direct
      placement on such a cell as failed: the search with first radius 1 runs, with its draws, and
      when it fails too, the base unit is removed and the step stops early at this house, so this
      house and every later house receive no budget units and the final draw of claim 20 is not
      taken (claim 45). The importer warns (a departure; Edge cases).
    - **Every other starting unit** is placed by the placement search around the base cell *b*
      with first radius 4.
    - **The placement search** with first radius *r*₀, for the base unit and every other starting
      unit alike, uses this acceptance test: a cell *accepts* the unit when it is a usable cell
      (the height-aware test of claim 4) and, in addition, either has no ground occupant (Terms)
      or, for an infantry unit, has an infantry unit as its *nearest ground occupant*. The
      nearest ground occupant is the one whose reference point (S-0204 claim 4: its position, or
      the centre of a building's foundation), taken by its place within its own cell (Terms), lies
      nearest to the cell corner: the one with the smallest distance ⌊*r*(*a*² + *b*²)⌋, where
      (*a*, *b*) is that place, *r* the table square root (S-0204 claim 5 and its Formulas F1) and the
      height plays no part (Formulas, *nearest ground occupant*). It is not measured to the cell's
      centre. On a tie, the occupant that came into the cell last wins among infantry, vehicles
      and aircraft, and any of them wins over a building; between buildings, the one that came
      into the cell first wins. Objects on a bridge deck are never ground occupants, so on a
      bridge cell only what stands on the ground under the deck counts. A cell that is not a legal
      cell is never usable, so it never accepts a unit, not even as the first element *b* of the
      ordering below.

      **Placement candidates and observable draws.** Let the first radius be 1 for a base unit whose direct placement has failed, and 4 for every other starting unit. At each radius through 31, the search has a first set of eight candidates and a second set of eight candidates. The first set lies one candidate in each of the eight step directions, at that radius from the base cell; the direction sequence begins at one ranged draw from 0 to 7 for that radius. The second set uses the same directions and radius, with each candidate independently displaced by either zero or one cell along each axis. For each such candidate, the x displacement is drawn first and its direction is chosen by a percent draw; the y displacement and its direction follow. A percent result below 50 moves toward higher coordinates, and every coordinate is clamped to 1 through W + H − 1.

      Candidate precedence is the order of the base cell, followed at each radius from the first radius through 31 by the eight first-set candidates in direction order and then the eight second-set candidates in direction order. The base cell can accept the unit only at its initial position in this order; a later candidate that repeats the base cell cannot accept it. Placement uses the earliest candidate in this precedence order that passes the acceptance test. The observable draw trace ends with the draws associated with that candidate, including draws for any ineligible repeated occurrence of the base cell; no draws for later candidates are taken. If no candidate through radius 31 passes, the full candidate sequence's associated draws are taken and placement fails. Implementations may test candidates as they are generated or store them first, provided the chosen cell and random-stream position are identical.

      The two candidate sets and the coordinate clamping are given by the formulas under **Placement search candidate**. The acceptance test above applies to both sets. The chosen unit stands at the candidate cell's centre at its terrain surface height, on the ground below any bridge deck, facing 0.

    - **Draws of a placement itself.** Putting a vehicle of a type with either visceroid flag
      (Import mapping) on the map takes one draw R(0, 29) at that moment, right after the draws
      of its search, if any. A placement that fails puts nothing on the map and takes no such
      draw. Putting an infantry unit on the map takes no draw while the map loads: it stands at
      the centre standing position of its cell (S-0208), whatever infantry already stand there,
      so several starting infantry units can share one cell's centre position
      (`quirks.starting_infantry_share_centre`, Edge cases).
    - **Draw order.** House by house, in house order: the base unit's creation draw, then the
      draws of its placement search (only when its direct placement fails), then its visceroid
      draw, if any; then, for each unit bought, its type draw, its creation draw, the draws of its
      placement search and its visceroid draw, if any. After the last house comes the draw
      R(0, 65535) of claim 20. Crediting a left-over budget takes no draw.
    - Backing: `measurable` (scenario M26: the cells, the standing positions and the count of raw
      values taken; variant C for the land type being ignored while the map loads; variant D for
      radii beyond the first and the second pass, which pre-placed objects around the base cell
      force; variant E for a placement that fails; variant J for the nearest ground occupant
      measured to the cell corner and its tie rule; variant K for deck objects not counting and
      the unit standing under the deck). Clamping and the draw boundary at a later candidate
      equal to the base cell are `unmeasured` here: M26's base cells are away from the clamp
      edges and its forced-failure variant never reaches an accepting edge candidate. Ties
      involving aircraft or buildings are also `unmeasured` here: M26 J3 only separates a
      tied infantry unit and vehicle; the wider rule rests on S-0209 claim 13's listing order.
      The visceroid placement draw is `unmeasured` here: M26 does not supply a starting
      vehicle with either visceroid flag, so it cannot distinguish that extra ranged draw.
      The base unit's search is `unmeasured`: on a
      legal base cell no established case makes the direct placement fail, and on a base cell
      that is not legal the original's result is unspecified (above), so no setup gives a result
      of the original to compare with.
    - Confidence: high for the search's order, radii, clamping and draws, and for the cell
      corner as the point the nearest ground occupant is measured to (revision 12 corrects the
      earlier "nearest to its centre"); medium for the rest of the acceptance rule (which objects
      are ground occupants, a building in every cell of its foundation, the tie rule, the deck),
      for the placement always succeeding while the map loads, and for the visceroid draw being
      the only draw of a vehicle's placement

### House identity for map inspection

47. **A listed house's country (Q-0419).** For map inspection without skirmish reassignment,
    each entry of the map's house list names a house and its own map section, not its country.
    Only the first 19 bytes of the list value are used, then leading and trailing whitespace
    are trimmed. The section's country selector determines the house's country even when the
    house name happens to equal a different country's name. An absent or empty selector uses
    the country at registration index 0, not the country named like the house.
    - A nonempty selector uses its first 127 bytes, trimmed after the cut. It matches a
      country's type name or its imported name field without regard to ASCII letter case.
      Countries are considered in registration order, so the earliest country matching either
      name wins. Numbers are names, never registration indices.
    - An ordinary nonempty name matching no country creates a new country at the end of the
      country registry, with that name limited to 24 bytes as in S-0004 claim 5. It uses engine
      defaults, not the properties of an unrelated country or a later read of its own section.
      In particular its inherited colour is the initial colour choice described by
      S-0103 claim 14. The house's own colour selector can still replace that inheritance under
      the same claim. A missing selector and an unknown nonempty selector therefore differ.
    - Reserved random-country and player-at-position selectors are not ordinary unknown names.
      They do not select a valid country for this map-house import in the original, whose
      result is unspecified. Our importer treats them as an absent selector and warns.
      A country registry with no country at index 0 likewise cannot supply the absent-selector
      fallback: our importer leaves that house unresolved and warns, rather than inventing
      a country or colour.
    - Backing: `measurable` (M27 A–C for names, defaults and creation). Reserved-selector and
      empty-registry original results are `unmeasured`: they supply no defined original
      country or colour to compare with.
    - Confidence: high for the selector, matching and defaults; medium for unknown-country
      inheritance and malformed-selector handling

48. **Owner names are house names, not country fallback selectors (Q-0419).** In the same
    map-inspection import, a nonempty house list creates only the listed houses. An object's
    owner must match one of their imported house names exactly, including letter case, as in
    claim 27. Matching a country but no house does not create an owner or select that country's
    colour. An owner matching neither is also unresolved: the original drops its object, so
    there is no original remap colour for it. A viewer that retains such an entry for inspection
    must distinguish it as unresolved rather than claim its provisional colour is inherited
    from the original.
    - With no house-list entries, the original instead creates one house per registered
      country, in country order. Such a house takes the first 20 bytes of its country's type
      name as its house name and inherits that country's colour, subject to its own section's
      colour override under S-0103 claim 14. Thus an exact country-name owner can resolve in
      this empty-list case, but not merely because it is a country name. It must match an
      actually created house. An empty-valued list line is no entry (S-0001 claim 11).
    - These rules do not create additional skirmish houses. Claim 26 still excludes the map's
      house definitions there; claim 27 determines owner retention, and S-0103 claim 10
      supplies the retained skirmish houses' colours, including the neutral and special houses.
    - Backing: `measurable` (M27 D–E, including listed, country-only and unknown owners)
    - Confidence: high for house-only matching and empty-list fallback; medium for pixels
      before measurement

## Formulas

**Starting-unit budget** (claim 45). With *n* ≥ 1 the number of types in the budget pool (1 when
it is empty), *C* the sum of their costs and *U* the unit-count setting, all signed 32-bit integers:

> *B* = trunc((trunc(*n* / 2) + *C*) / *n*) × *U*

where trunc rounds toward zero. Every intermediate sum, the quotient and the product are signed 32-bit
integers, and a result outside −2147483648 … 2147483647 wraps (S-0205 claim 1, *wrap*); so does the
remaining budget *R* when a cost is subtracted from it. Stock data stays far inside the range, and the
importer warns when the costs of the budget pool, or *B*, would leave it. *B* is the same for every house. A unit comes from the vehicle pool
while *R* > trunc(*B* / 3) (and the vehicle pool is not empty). Example: a pool of 4 types costing
in all 2002 credits gives trunc((2 + 2002) / 4) = 501 per unit of *U*.

**Placement search candidate** (claim 46). For base cell *b* = (*b*ₓ, *b*ᵧ), radius *r* and step
direction *e* with unit step (*s*ₓ, *s*ᵧ) (glossary *step direction*, both components −1, 0 or 1),
the first-pass candidate is (clamp(*b*ₓ + *r*·*s*ₓ), clamp(*b*ᵧ + *r*·*s*ᵧ)) with clamp(*v*) =
min(max(*v*, 1), W + H − 1). The second-pass candidate replaces its x by clamp(x + *a*ₓ) when
*p*ₓ < 50 and by clamp(x − *a*ₓ) otherwise, and its y likewise with *a*ᵧ and *p*ᵧ. Integers, exact.

**Nearest ground occupant** (claim 46). For a ground occupant whose reference point is (X, Y, Z)
leptons (S-0204 claim 4), *a* = X mod 256 and *b* = Y mod 256 (least non-negative residues; on a
legal cell X and Y are positive), and its distance is δ = ⌊*r*(*a*² + *b*²)⌋ with *r* the table
square root of S-0204 (Formulas F1); *a*² + *b*² is an exact integer of at most 130050. Z plays no
part. Number domain in the original: the root comes from the table of single-precision values of
the distance sum below. Its argument here is an integer below 2²⁴, so, as stated there, the
narrowing to single precision is exact and ⌊*r*(*a*² + *b*²)⌋ does not depend on the rounding mode
in force. The nearest ground occupant has the smallest δ; ties as claim 46 says. Examples: an object
at its cell's centre has *a* = *b* = 128 and δ = ⌊*r*(32768)⌋, at most 181 (S-0204 claim 5); a 2 × 2
building's reference point is a cell corner, so it has δ = 0 in each of its four cells and is the
nearest ground occupant there unless another occupant also has δ = 0.

**Cell number decoding.** For an integer n, a signed 32-bit integer (a waypoint value read as an
integer by S-0003 claims 3 and 4, or a cell-tag or terrain-object key read as its leading decimal
integer, which wraps in the same way):
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

**Distance sum (claim 19).** For a free position p and the set T of taken positions, with start
cells (x_p, y_p) and (x_t, y_t), integers in −32768 … 32767 (claim 15):
S(p) = Σ over t in T of v_t, with v_t = w(⌊r(n_t)⌋), n_t = Δx² + Δy², Δx = w(x_p − x_t) and
Δy = w(y_p − y_t).
Here w is the wrap of the cell number decoding above (reduction modulo 65536 into
−32768 … 32767), so n_t is an exact integer in 0 … 2³¹. And r is the *table square root*: r(0) = 0,
and for n ≥ 1, with e = ⌊log₂ n⌋ and z = ⌊e / 2⌋,
r(n) = 2^z × ⌊√m′ × 2²³⌋ / 2²³, where m′ = ⌊n × 2^k / 4^z⌋ / 2^k, with k = 13 when e is even and
k = 12 when e is odd.
So n / 4^z, which lies in 1 … 4 (4 excluded), is truncated toward zero to a multiple of 2⁻¹³ (below 2)
or 2⁻¹² (from 2 on), and its square root is truncated toward zero to 24 significant binary digits.
All of this is exact rational arithmetic; ⌊r(n)⌋ is an integer. S(p) is a sum of fewer than 8
terms (N is at most 8, claim 17), each in −32768 … 32767, formed exactly. The chosen position is the lowest-indexed p with maximal
S(p) when that maximum is 0 or more. When every free position's S(p) is below 0, the original's
choice is unspecified, and our engine chooses the lowest-indexed free position (claim 19).

Pairs of legal cells. Both cells of a pair are legal cells in every case except a start waypoint
marker on a non-legal cell, since a random start cell is a usable cell (claim 18). On a supported
map their coordinates lie in 1 … 511 (claim 2), so |Δx| and |Δy| are at most 510, neither wrap
changes anything, every term is ⌊r(n_t)⌋ ≥ 0, and n_t ≤ 2 × 510² = 520200, below 2²⁴. For every n below 2²⁴, ⌊r(n)⌋ is never more
than ⌊√n⌋ and never less than ⌊√n⌋ − 1. It equals ⌊√n⌋ for every n below 16641 = 129², so for
every pair less than 129 cells apart. The first n where it is one less is 16641 itself (a distance
of exactly 129 cells gives a term of 128). For larger n the shortfall occurs at scattered values,
not only at perfect squares.

Start cells far outside the map. A start waypoint marker on a non-legal cell (claims 1 and 15) can
give a coordinate difference of 32768 or more in absolute value, which the first wrap reduces (a
difference of 32775 gives −32761), and n_t up to 2³¹. From 2²⁴ on, ⌊r(n)⌋ can be more than one
less than ⌊√n⌋, and a term ⌊r(n_t)⌋ of 32768 or more becomes negative through the second wrap
(32769 gives −32767). Example: start cells (20, 20) and (−500, −32747) give Δx = 520 in absolute
value, Δy = −32767, n = 1073946689, ⌊r(n)⌋ = 32769 and the term −32767.

Number domain in the original: the coordinate differences and the terms are integers reduced as
above; the squared distance is formed in double-precision floating point, which is exact because
it is below 2⁵³; it is then narrowed to single precision, the root comes from a table of
single-precision values, and the result is truncated toward zero to an integer, whatever the
rounding mode in force. Under the round-toward-zero mode (glossary; S-0205 claim 5) the narrowing
truncates, and r above already includes that truncation (S-0204, F1), so the formula reproduces
the original exactly in integer arithmetic for every n. For n below 2²⁴, which covers every pair of
legal cells, the narrowing is exact and the result does not depend on the rounding mode. For n of
2²⁴ or more, which only a start cell far outside the map gives, the result with
`quirks.chop_rounding` (S-0205) off follows the rule that S-0204's F1 gives for that mode; this spec
defines no switch of its own for it. It is the same tabulated square root that S-0105 (F1) uses,
restricted to integer inputs, and the same function that S-0204 (Terms, *table square root*, claim 5
and F1) uses for distances between objects. Without `quirks.table_root_distance` (S-0204) the term
is w(⌊√n⌋), where ⌊√n⌋ is the largest integer whose square is at most n.

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
All values are signed 32-bit integers (the header values as read by S-0003 claims 3 and 9, the preview
size and position in screen pixels). Every difference, product and sum wraps modulo 2³² into
−2147483648 … 2147483647 (S-0205 claim 1, *wrap*), and each quotient is truncated toward zero
(S-0205 claim 2; this differs from ⌊ ⌋ when xi < X0 or yi < Y0). A divisor of 0 is the crash of
claim 25, and the one quotient that overflows, −2147483648 divided by −1, is S-0205 claim 3's.
Apart from a divisor of 0, where our engine draws no markers (claim 25), our engine reproduces all
of this. The
inner quotient is the pair's displacement from the origin in per-mille (‰) of the marker extent,
and the outer one turns it into pixels of the drawn preview. The −9/−6 terms place the marker
graphic's corner. Example of the wrap: X0 = −2147484, Wh = 1000 and xi = 2147984 give
xi − X0 = 4295468, whose product with 1000 wraps to 500704, so the inner quotient is 500, as for a
pair halfway across the extent.

**Preview marker position, waypoint form (claim 25a).** For a marked legal cell (x, y) on a map of
size W × H, with pw, ph, left and top as in the formula above:
u = ⌊(x − y) / 2⌋ and v = ⌊(x + y + 1) / 2⌋
(u grows to the right on screen and v downwards; one step of u is two pixels of a footprint-sized
preview image and one step of v is one pixel),
u0 = ⌊(1 − W) / 2⌋ and v0 = ⌊(W + 2) / 2⌋ (the least u and the least v of any legal cell),
U = W − 1 and V = ⌊(W + 2H + 1) / 2⌋ − ⌊(W + 2) / 2⌋ (the spans of u and v over the legal cells),
px = ⌊⌊(u − u0)·1000 / U⌋ · pw / 1000⌋ + left − 9
py = ⌊⌊(v − v0)·1000 / V⌋ · ph / 1000⌋ + top − 6.
Integers only. For a legal cell every operand is non-negative and no product leaves the signed
32-bit range, so each ⌊ ⌋ equals the truncation of the formula above and nothing wraps. When U = 0
or V = 0 (W = 1, or H = 1 with W even), no markers are drawn. This is the formula above applied to a
header with origin (u0, v0), extent (U, V) and pair (u, v). Adding one constant to X0 and to the
first number of every pair, or to Y0 and to the second number of every pair, changes no marker.
Example: W = H = 40 gives u0 = −20, v0 = 21, U = 39 and V = 39. The cell (30, 25) gives (u, v) = (2, 28)
and the cell (25, 50) gives (−13, 38).

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

**Object health (claim 37).** For a type strength *S* (hit points) and a health value *H*, both
signed 32-bit integers (claim 34), with *H* first replaced by min(*H*, 256) for a building:
h₀ = w₃₂(trunc(RZ₅₃(*H* × *S*) / 256)), and h = *S* when h₀ > *S* − 3, otherwise h = h₀.
trunc rounds toward zero. RZ₅₃ rounds the exact product toward zero to 53 significant binary digits
(S-0205 claim 5); it changes nothing while |*H* × *S*| < 2⁵³, and while |*H* × *S*| < 2⁶¹ the
result is the same as trunc(*H* × *S* / 256). w₃₂ is the wrap of S-0205 (claims 1 and 7): the
integer in −2147483648 … 2147483647 congruent to its argument modulo 2³², which changes nothing
while the quotient lies in that range. *S* − 3 is a signed 32-bit integer too. Then the low end of
claim 37 applies: for infantry h < 1 gives 1, for vehicles and aircraft h = 0 gives 1. Without
`quirks.near_full_health_rounds_up`, h = min(h₀, *S*).
Number domain in the original: the product is formed in floating point under the round-toward-zero
mode, the division by 256 is exact, and the conversion to an integer is S-0205 claim 7's common
form: truncation toward zero, then the wrap (the quotient is always below 2⁵⁴ in absolute value,
so the conversion's zero result for 2⁶³ and above never arises). With `quirks.chop_rounding`
(S-0205) off, the product is rounded by S-0205's rule for that mode instead of RZ₅₃, which can
change the result only when |*H* × *S*| ≥ 2⁵³; this spec defines no switch of its own for it.
Examples: *S* = 300 and *H* = 128 give h₀ = 150 and h = 150; *S* = 300 and *H* = 255 give
h₀ = 298 > 297, so h = 300. A vehicle with *S* = 1000 and *H* = 1099511756 gives
*H* × *S* / 256 = 4294967796.875 and h₀ = w₃₂(4294967796) = 500, so h = 500, half strength; a
building with the same *H* and *S* has *H* replaced by 256 and starts at full strength.

**Object cell (claims 35, 38 and 42).** At format version 4 and above, an object entry's x and y
are its two cell fields, and a terrain-object key n gives x = n rem 1000 and y = n quo 1000. Below
format version 4, an object entry's single cell field and a terrain-object key n give
x = n rem 128 and y = n quo 128. quo truncates toward zero and rem takes the sign of n, as in the
cell number decoding above. A smudge's x and y are always two fields. Each coordinate c is then
replaced by w(c) = ((c + 32768) mod 65536) − 32768, the wrap of the cell number decoding above.
Integers, exact. The world position of the cell's centre is (256·x + 128, 256·y + 128) leptons,
and an infantry unit's is (256·x + u, 256·y + v) with (u, v) from S-0208 table P.

**Facing of a pre-placed object (claim 39).** facing = 256 × (*v* mod 256) facing units, mod the
non-negative remainder. Integers, exact.

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
| coordinate wrap | 65536 | cells | each decoded coordinate of a waypoint marker, cell-tag key, object entry, terrain-object key or smudge entry is reduced modulo 65536 into −32768 … 32767 (claims 15, 31, 38 and 42; Formulas), and so are the coordinate differences and the terms of the distance sum (claim 19; Formulas) |
| overlay stream size | 512 × 512 | values (stream positions) | the value at index 512·y + x belongs to stream position (x, y) |
| no-overlay value | 255 | overlay type stream value | |
| no-tile marker | 65535 | tile number | in the terrain stream's tile number |
| level height | 104 | leptons per height level | engine constant; see S-0101 |
| cell animation displacement | 384 | leptons, along +x and along +y | claim 12: from the centre of the overlay's cell to the cell animation's position; engine constant |
| water fill variants | 4 | tiles | water base tile + 0 … 3 |
| fill base tiles before any theater is loaded | 0 | tile number | clear and water base tile alike; see quirk `quirks.stale_fill_tiles` |
| overlay indices whose data value survives creation | 24, 25, 237, 238 | overlay list index | creation leaves the cell's data value unchanged (claim 12) |
| vein overlay index | 126 | overlay list index | claim 14; S-0202 claim 10 |
| vein seed threshold | 48 | overlay data value | data ≥ 48 → vein seed (claim 14) |
| largest slope shape that takes an overlay | 4 | slope shape (S-0106) | an overlay is created on a cell whose slope shape is above 4 only for the overlay type at list index 178 (claim 12) |
| overlay index exempt from the slope condition | 178 | overlay list index | claim 12 |
| random start margin | 10 | cells | see the random start candidate formula |
| random start clear area | 8 × 8 | cells | the start area: corner cell and the cells towards increasing x and y (claim 18) |
| tick number during loading | 0 | ticks | the choice by tick number of the nearby-cell search picks the first preferred cell (claim 18) |
| starting-unit failure limit | 20 | failed placements per house | claim 45 |
| game's tech level when the base rules omit it | 10 | tech level (the integer scale of the types' tech-level field) | absent-key default (Terms, game's tech level; Import mapping) |
| tech level of the neutral and special houses | 1 | tech level | fixed in the program (Terms, house's tech level) |
| maximum player count when the base rules omit it | 8 | players | absent-key default (claim 28; Import mapping) |
| vehicle share of the budget | everything above trunc(*B* / 3) | credits | units come from the infantry pool once *R* ≤ trunc(*B* / 3) (claim 45) |
| placement search radii | first radius … 31 | cells | first radius 4 for budget units, 1 for a base unit whose direct placement fails (claim 46) |
| placement search passes | 2 per radius, 8 candidates each | candidates | the second pass adds the displacement draws (claim 46) |
| displacement per axis | 0 or 1 | cells | R(0, 1), direction by a percent draw against 50 (claim 46) |
| displacement direction threshold | 50 | percent | a percent draw below 50 moves the coordinate towards higher values (claim 46; Terms, percent draw) |
| radius-start draw | 0 … 7 | step directions | one ranged draw *k*ᵣ = R(0, 7) per radius reached; candidate *F*(*r*, *j*) lies along step direction (*k*ᵣ + *j*) mod 8 (claim 46) |
| final draw after the starting units | 0 … 65535 | draw values (dimensionless) | one ranged draw after the last house, one raw value, its value unused; not taken when the step stops early (claims 20 and 45) |
| visceroid placement draw | 0 … 29 | draw values (dimensionless) | one per placement of a vehicle of a visceroid type (claim 46) |
| purchases without a fall in the budget before our engine stops a house | 1000 | consecutive purchases | a limit of our engine, not a constant of the original (Edge cases) |
| failed random start candidates before our engine gives up | 1000 | candidates (pairs of ranged draws) | a limit of our engine, not a constant of the original, which sets no limit (claim 18) |
| preview marker scale | 1000 | ‰ of the marker extent | per-mille step of a marker's displacement (Formulas, both preview marker position entries) |
| preview marker corner shift | (9, 6) | pixels | subtracted from the marker position: −9 horizontally, −6 vertically, the same with the switch on or off (Formulas) |
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
| health value of full strength | 256 | health-value units | claim 37; a building's larger values are replaced by 256; for the other kinds larger values give full strength while the quotient of Formulas (object health) does not wrap |
| near-full margin | 3 | hit points | a starting health above *S* − 3 becomes *S* (claim 37) |
| object entry characters read | 127 | characters | of each object or smudge entry's value (claim 34) |
| legacy object cell base | 128 | cell-number units per unit of y | object cell fields and terrain-object keys below format version 4 |
| upgrade name fields | 3 | fields | per building entry (claim 40) |
| facing value scale | 256 | facing units per unit of the facing value | claim 39 |
| infantry facing when absent | 0 | facing units | claim 39 |

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
| Theater | yes | tilesets, palettes | the fill uses the tiles of the theater loaded before this map (claim 6, quirk `quirks.stale_fill_tiles`). Unknown names: see Edge cases |
| Lighting | yes | see S-0104 | — |
| ***Terrain and overlay*** | | | |
| Terrain stream (current and legacy) | yes | tiles, heights | overrides the initial fill. The last record for a cell wins |
| Overlay type and data streams | only at format version ≥ 2 | overlays | the data stream's values replace every data value that overlay creation set (claim 13); on every cell that the vein rebuild clears or re-patterns, the overlay and data value it leaves replace the streams' (claim 14) |
| Tube (tunnel) definitions | yes | tunnels | — |
| Terrain objects (trees and similar) | yes | terrain objects (claim 42) | placed before every vehicle, aircraft, infantry unit and building (claim 43) |
| Smudges | yes | smudges (claim 42) | placed after every building (claim 43) |
| ***Markers and start positions*** | | | |
| The waypoint markers | yes | waypoint cells, home waypoint, start list | the early phase already uses them (claim 20) |
| Header block (marker data) | yes | preview markers with `quirks.preview_markers_from_header` on; pairs 9 … min(N_h, 16) are start position marks (claim 29) | its marks are in force for the early phase and are applied again for the late phase (claim 29) |
| ***Houses and pre-placed objects*** | | | |
| Vehicles, aircraft, infantry, structures | yes | pre-placed objects (claims 27 and 34 to 41) | the late phase's list 3 is built on a map that already holds all of them (claim 20) |
| Per-house definitions | **no** | — | houses come from the lobby |
| Player name setting | no | campaign only | — |
| ***Rules, settings and triggers*** | | | |
| Rules override sections | yes | see S-0006 | pre-placed objects already follow the overrides |
| Map-wide settings (growth switches, free radar, crates and similar) | yes | map-wide game settings; the initial-time and percent settings are read and kept with no known effect (Import mapping) | — |
| Team, script, task-force, trigger, tag and AI-trigger definitions | yes | triggers and tags: S-0011; teams, scripts, task forces and AI triggers: the team system, which has no spec (a pointer only) | — |
| Cell tags | yes | attaches tags to cells (claim 31) | for one cell, the first entry wins, even with an empty tag; a key written twice yields its winning occurrence's value both times (claim 31) |
| Variable names | yes, first 100 entries | map trigger variables: names and initial states (claim 32) | — |
| ***Map selection*** | | | |
| Preview size and stream | map selection screen only | preview image | — |
| Map list name, digest, minimum and maximum players, game modes, official flag | map selection screen only | listing | — |

**Start position resolution.** The rows are the points of loading at which start positions
and base cells are decided, or which take draws between those decisions, in the order in which
they occur. For each point the table gives what is already in force there, what is decided there, and the draws it takes from the scenario random
stream. Every draw of a row comes after the draws of the rows above it, and within a row the draws
come in the order listed. Nothing else in the table fixes an order: an implementation must
reproduce the draw order and the consequences that claims 20, 28, 29 and 33 state.
Animation creation and immediate start-event draws in these rows include any immediate
middle-event particle and smudge creation draws under S-0202 claim 15. That claim
supplies their composition; subsequent particle updates are outside its scope.

| Point in loading | Already in force | What is decided | Draws, in stream order |
|---|---|---|---|
| Early phase (claim 20) | this map's size, header block, local area and waypoint markers; the houses, in the order of claim 19; the country data of claim 33 (the data in force before this game's rules layering, with the base rules' extra read over it); every legal cell a blank cell | the marks of the applied header pairs (claim 29); players who chose a position get that entry of list 1, and a chosen position's mark is its chooser's house number whatever a header pair says; houses on "random" get positions by claim 19 against the cells of list 2 | list 1's random entries; list 2's random entries; the random-position draws of claims 19 and 22, in house order |
| Initial fill (claim 6) | the early phase's decisions | the fill of every visited cell | one per visited cell, for a water fill only |
| Map content (claims 7 to 14, 27, 31 and 34 to 43) | the fill | the cell tags, the terrain, the overlay, the vein rebuild, the terrain objects, the pre-placed objects and the smudges | the draws that S-0202 (random streams, its table "Draws during map loading") lists between the initial fill and the late phase, in its order. Among them, in this relative order: the tags created by cell tags (S-0011, map triggers, claim 34); the overlay placement draws of claim 12 and the tile animations of the first terrain recalculation pass (S-0010, claim 12); the vein pattern draws of claim 14; then the draws of the pre-placed objects: all vehicles' draws, then all aircraft's, then all infantry's, then all structures', and within one kind in entry order; and last the tile animations of the second terrain recalculation pass. Every pre-placed object that is created takes one creation draw (S-0202 claim 10), and it comes before the draws of that object's tag (S-0011 claim 34) and of its placement. An object that claim 27 drops takes no draw. An object that is created but cannot be placed does not appear in the game, yet its creation draw has been taken |
| Unused-house cleanup (claim 28), when its condition holds | the second terrain recalculation pass is complete | eligible houses and their pre-placed objects and existing triggers are removed | conditional tile-animation creation and immediate start-event draws from removal-cell recalculations, before the late phase (S-0202 claims 10 and 14) |
| Late phase (claim 20) | the terrain, the overlay and every surviving pre-placed object; this game's full rules layering with the map's overrides (claim 33); the unused-house cleanup of claim 28, already applied when its condition holds | the marks of the applied header pairs, applied again (claim 29); every player keeps its position index, and its base cell is that index's entry in list 3, which is built on the real terrain; a non-passive house without a position (its position was overwritten by a mark, or it was passive in the early phase, claim 28) is assigned anew by claim 19 against list 3; a house that the cleanup removed takes no part (claim 28) | list 3's random entries; the re-assignment draws of claims 28 and 29 |
| Starting units | every base cell | each house's base unit and budget units, placed around its base cell (claims 45 and 46) | house by house, in house order (claim 19), the draws of claim 46: the base unit's creation draw, the draws of its placement search (only when its direct placement fails) and any visceroid draw, then for each unit bought its type draw, creation draw, placement search draws and any visceroid draw; then, after the last house, one ranged draw over 0 … 65535 (one raw value) whose value has no effect (not taken when the step stops early, claim 45). Later loading steps draw after these: initial crates, conditional secret-lab choices, end-of-loading tags, then the game-start tile animations (S-0202 claim 14). This order holds when the starting-unit step stops early or places no units |
| Game-start terrain recalculation | initial crates, secret-lab choices and end-of-loading tags are complete | every legal cell is recalculated before tick 0, even when no starting units were placed | in visiting order, the applicable creation and immediate start-event draws of missing tile animations (S-0010 claims 12 and 19; S-0202 claims 10 and 14). A cell whose start was recorded since the second pass takes no new creation draws |
| Later resource-glint setup | game-start terrain recalculation is complete | resource-glint setup as specified by S-0202 claim 14 | its conditional choices and applicable immediate animation draws, before tick 0 (S-0202 claim 14 and R13). The recalculation row is not an exhaustive final-draw boundary |

## Edge cases and quirks

Every quirk switch below defaults to the original (YR) behaviour described first; the *fixed* or
*alternative* behaviour is what the switch selects when turned off.

- **`quirks.narrow_map_partial_pass`.** On a map with W = 1, the visiting order contains only
  cell (1, 1) (claim 3). The initial fill therefore leaves the cells (2, 2) … (H, H) as blank
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
  waypoint i is defined, and only undefined waypoint markers become random. N stays max(P, K), with
  K still counted consecutively from waypoint 0 as in claim 17, so the list has the same length and
  the same number of random entries as with the switch on; only which positions they fill changes.
  The random entries are found in increasing position order, each by claim 18 with its draws. In
  claim 17's example (waypoint markers 0, 1, 3 and 5, 4 players) the list becomes [wp0, wp1,
  random, wp3]. A defined start waypoint marker numbered N or higher is still not used. (A design
  choice of our engine for the switched-off state; the original has no such state.)
- **`quirks.lobby_counts_zero_waypoint`.** The lobby counts a start waypoint stored as 0 as defined,
  but the game treats it as undefined and gives that slot a random start (claims 15, 23). *Fixed
  behaviour:* the lobby uses the same "undefined" test as the game.
- **`quirks.preview_markers_from_header`.** Preview markers are taken from the header block, which
  can disagree with the real start waypoint markers (claim 25). *Fixed behaviour:* draw a marker for
  each start position whose entry is a waypoint cell, at that cell projected onto the preview
  (claim 25a and Formulas, preview marker position, waypoint form).
- **`quirks.random_start_asymmetric_range`.** The random start candidate's y range runs 10 cells
  further than its x range and can leave the map (claim 18). *Fixed behaviour:* draw both
  coordinates from 11 … W + H − 10, in the original's order: the first ranged draw
  R(11, W + H − 10) gives y and the second, over the same range, gives x, and the candidate is
  (x, y). Each is one ranged draw by the range rule (Terms), and when W + H < 21 the upper bound
  lies below the lower one and the range rule takes the two bounds in either order, as in Formulas
  (*Random start candidate*). This changes which cells come out. Each attempt still takes two
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
- **`quirks.partial_terrain_values` (owned by S-0012, default original).** Partial coordinate
  pairs after an earlier complete terrain record, and partial tile values whenever complete
  coordinates select a legal cell, have the low-byte replacement effects of claim 7 and
  S-0012 Formulas F3. With the switch off, partial multi-byte values count as absent:
  a partial coordinate pair ends the stream and a partial tile number gives 65535.
  Complete one-byte values still take effect. The importer warns either way. An incomplete
  first coordinate pair is the separate safe departure of claim 7, not governed by this switch.
- **`quirks.cell_index_alias`.** A terrain record or cell-tag key whose x coordinate lies outside
  0 … 511 acts on the cell with the same stream position index 512·y + x, which can be a legal cell
  other than the one the coordinates name (Terms, *selected cell*; claims 7 and 31). *Fixed
  behaviour:* such a pair selects no cell, so the record or entry has no effect. The importer warns
  about such a pair either way. Stock maps hold no such pair.
- **`quirks.table_root_distance` (owned by S-0204).** This spec has no switch of its own for the
  table square root. The farthest-sum rule follows S-0204's switch, which also governs the distances
  between objects. With the switch on (the original), each pair is measured with the table square
  root of Formulas, which falls one short of the truncated Euclidean distance for some pairs of
  legal cells 129 or more cells apart (claim 19), and can fall further short for a pair with a
  start cell far outside the map. Ties it creates go to the lower-numbered position. With it off,
  each term is ⌊√n⌋, the exact truncated distance, with the same reductions modulo 65536 of the
  differences and of the term (Formulas). For start positions this changes which player holds
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
  cell (claims 1, 15 and 17), and the player's base cell is that cell. The placement search of
  claim 46 tries only usable cells, so the budget units appear on usable cells within its radii,
  or fail. What the original does with the base unit, whose direct placement aims at the
  non-legal cell, is unspecified (Open questions). Our engine treats that direct placement as
  failed, so the base unit's placement search runs, and when it fails as well the step stops
  early (claims 45 and 46; a departure). Such a start cell can also lie far outside the map, and
  then the distance sums of claim 19 follow the wider rule of Formulas, with one unspecified case
  that our engine replaces (a departure). The importer warns about every start waypoint marker
  on a non-legal cell.
- **`quirks.starting_infantry_share_centre`** (default on, the original). A starting infantry unit
  placed into a cell that already holds infantry stands at that cell's centre position together
  with them (claim 46), with no draw. *Alternative:* it gets the standing position that S-0208
  claim 3 gives for a desired point of class centre, with that claim's draw, and a cell with no
  free position for it does not accept it. The draws of the step then differ from the original's.
  In the alternative, the placement search's acceptance test (claim 46) gains one condition for an
  infantry unit: S-0208 claim 3 must give the unit a position in the cell, judged on the cell's
  ground occupancy record. That is the case unless the first row of that claim's table applies
  (vehicle-present or object-present without an open gate) or the cell's right, left and bottom
  positions are all taken. Every row of S-0208's table O for class centre holds those same three
  positions, so whether a position exists does not depend on the draw, and testing a cell takes
  no draw. The draw R(0, 3) of S-0208 claim 3 is taken once, on the cell that accepts the unit,
  right after the draws of the search element that reached that cell and before the next unit's
  draws; the unit then stands at the position that draw gives. (A design choice of our engine for the
  switched-off state.)
- **Empty infantry pool when infantry is due** (departure, no switch). When a house's remaining
  budget is at or below trunc(*B* / 3), or its vehicle pool is empty, the original takes the next
  unit from the infantry pool even when that pool is empty, with an unspecified result. Our engine
  takes it from the vehicle pool instead (the ranged draw is then over the vehicle pool), and the
  importer warns when a country that a house can have gets an empty infantry pool and a non-empty
  vehicle pool.
- **Budget that does not fall** (departure, no switch). A type of cost 0 or less does not lower
  the remaining budget, so a pool of such types can keep a house buying forever in the original.
  Our engine stops a house's buying after 1000 consecutive successful purchases that did not lower
  its remaining budget (a limit of our engine), and the importer warns about every type in a start
  pool whose cost is 0 or less.
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
  not a quirk switch, since no engine reproduces a crash. With `quirks.preview_markers_from_header`
  on, our engine draws no preview markers for such a map and treats the rest of the map normally (preview image, start positions and
  everything else unchanged). An importer warns.
- **`quirks.object_empty_fields_collapse`.** In an object or smudge entry, an empty field (two
  commas in a row, or a leading comma) is not a field: every later field moves one place forward,
  so "A,B,256,,10,12" reads 10 as the cell x (claim 34). With the switch off, an empty field keeps
  its place: an empty optional field takes its default, and an empty required field drops the entry
  as a short entry does (claim 36), with an importer warning.
- **`quirks.near_full_health_rounds_up`.** A pre-placed object whose computed starting health is
  above *S* − 3 starts at full strength (claim 37). With the switch off, it keeps the computed
  value, never more than *S*.
- **`quirks.upgrade_fields_carry_over`.** An absent upgrade name field of a building entry keeps
  the type that the same field named in an earlier building entry (claim 40), so a building can
  receive an upgrade its own entry does not name. With the switch off, an absent field means no
  upgrade.
- **`quirks.follow_index_by_place`.** The placed vehicle numbered i takes the follow value
  numbered i, not the value of its own entry (claim 41). When an entry lacks the follow-index
  field, the later values therefore move forward onto earlier vehicles, and a vehicle can become a
  follower car that its own entry did not link. With the switch off, each placed vehicle takes the
  value of its own entry, a vehicle whose entry gives no field has no leader, and the value of an
  entry whose vehicle is not placed is discarded. In both cases n names the placed vehicle
  numbered n. The switch changes nothing on a map whose vehicle entries all give the field and are
  all placed, and it never changes the draws.
- **`quirks.early_phase_stale_country_data`.** The early phase judges passivity by country data
  from before this game's rules layering, with only the base rules' country data read again over
  it (claim 33). A map's or game-mode file's passive field therefore does not reach the early
  phase, and a value that an earlier game of the same program run left in force can. With the
  switch off, the early phase judges passivity by the country data that the cleanup and the late
  phase see: this game's full rules layering with the map's overrides. A player whose country
  this game makes passive then takes no position in the early phase, and nothing from an earlier
  game reaches it. This can change which house holds which position and how many draws the
  assignment takes. With `quirks.double_start_resolution` off there is no early phase, so this
  switch then has no effect.
- **Pre-placed objects: unspecified entries.** These are unspecified in the original, and our
  engine replaces each by a safe result with an importer warning (departures): an entry that lacks
  a required field (claim 36: dropped, with the creation draw that claim gives); a starting health below 1 for
  a vehicle or aircraft, or a building's starting health of 0 or less (claim 37: health 1); an
  object entry whose cell is not legal (claim 38: creation draw taken, object dropped); an upgrade
  count above 3 (claim 40: 3 used); a placed vehicle numbered at or beyond the number of follow
  values (claim 41: no leader); a terrain object on a cell that is not legal and a smudge entry
  with an x field but no y field (claim 42: nothing placed).

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
| [Header] NumberStartingPoints | preview marker count N_h (markers with the switch on, claim 25) | integer | −1 (no markers) |
| [Header] StartX, StartY | marker origin X0, Y0 (markers with the switch on, claim 25) | integer | −1 |
| [Header] Width, Height | marker extent Wh, Hh (markers with the switch on, claim 25) | integer | −1. An explicit 0 makes the original crash when markers are drawn; our engine then draws no header markers and an importer warns (Edge cases) |
| [Header] Waypoint1 … WaypointN | marker pairs (xi, yi) (markers with the switch on, claim 25), read for i = 1 … N_h only (N = N_h, the marker count); pairs 9 … min(N_h, 16) are start position marks (claim 29) | two integers, by S-0003 claim 9 (an unreadable component is unspecified in the original; our engine keeps the corresponding value in force, and the importer warns) | pairs 1 … 8: (0,0). Pairs 9 … 16: readable components set their corresponding marks; unreadable components are unspecified in the original and keep their prior marks in our engine. Missing or empty pairs keep both marks. Pairs above N_h: not read |
| [Header] NumCoopHumanStartSpots | read; no skirmish effect established, so it has no unit here | integer | 4 |
| [Preview] Size | preview width = third number, height = fourth | four integers | (0,0,0,0) (quirk); an empty value is the same as absent |
| [PreviewPack] | preview pixels (claim 24) | encoded block | no preview |
| [Basic] Name | map list name | string | "No Name" (loose map files) |
| [Basic] MinPlayers, MaxPlayers | shown in the map list only | integer | 2 and 4 (loose map files) |
| [Basic] GameMode | game modes the map is listed under | string list | empty |
| [Basic] Official | official-map flag | boolean | false |
| [Basic] Theme | map theme: read and kept as map data; S-0703 (claim 11 and its Import mapping) gives what it means and how it is parsed | string, with the text limit S-0703 gives (127 bytes) | no map theme |
| [Digest] | map digest for multiplayer comparison | string | "No Digest" |
| [RandomMap] NumPlayers | lobby fallback player count (claim 23) | integer | 0 → 8 |
| [Basic] TiberiumGrowthEnabled, VeinGrowthEnabled, IceGrowthEnabled | growth switches (economy topic, no spec yet) | boolean | true, true, true |
| [Basic] TiberiumDeathToVisceroid | visceroid switch | boolean | true |
| [Basic] FreeRadar | radar without a radar building | boolean | false |
| [Basic] TruckCrate, TrainCrate | crate switches | boolean | false, false |
| [Basic] FillSilos | read; effect not established here | boolean | false |
| [Basic] MultiplayerOnly | read; effect not established here | boolean | false |
| [Basic] InitTime | read and kept as map data; no rule of play is known to use it, so it has no unit here | integer | 10000 |
| [Basic] Percent | read and kept as map data; no rule of play is known to use it, so it has no unit here | integer | 0 |
| [VariableNames] | map trigger variables (claim 32): the first 100 entries; key = variable number, read as a leading integer; value = name, then optionally the initial state. The importer warns about a repeated key, about distinct keys that give the same number, and about the three cases of claim 32 that our engine replaces | comma-separated list: string, integer (not 0 = set) | no variables. No second item: the state is left as it was (clear in the first game of a program run, claim 32) |
| [Units] (any key) | pre-placed vehicles, one per entry: owner, type, health value, cell x, cell y, facing value, behaviour name, tag name, veterancy value, group number, bridge flag, follow index, recruit-no flag, recruit-yes flag (claims 34 to 41; below format version 4 one cell number replaces x and y) | comma-separated list, empty fields collapse (claim 34); integers read as leading integers; flags set when not 0 | no vehicles. Fields 1 to 6 required (claim 36). Absent optional fields: behaviour none, no tag, veterancy from type and country (S-0311), group number not set, bridge flag clear, no follow value (which, by `quirks.follow_index_by_place`, does not by itself mean that this vehicle has no leader; claim 41), both recruit flags clear |
| [Aircraft] (any key) | pre-placed aircraft: owner, type, health value, cell x, cell y, facing value, behaviour name, tag name, veterancy value, group number, recruit-no flag, recruit-yes flag | as [Units] | no aircraft. Fields 1 to 6 required; absent optional fields as for [Units] |
| [Infantry] (any key) | pre-placed infantry: owner, type, health value, cell x, cell y, standing position number (S-0208 claim 5), behaviour name, facing value, tag name, veterancy value, group number, bridge flag, recruit-no flag, recruit-yes flag | as [Units] | no infantry. Fields 1 to 6 required. Facing value absent: 0. Bridge flag absent: S-0208 claim 5's height. Other optional fields as for [Units] |
| [Structures] (any key) | pre-placed buildings: owner, type, health value, cell x, cell y (top-left foundation cell), facing value, tag name, computer-sell flag, computer-rebuild flag, switched-on flag, upgrade count, searchlight value, three upgrade names, computer-repair flag, nominal flag (claims 34 to 40) | as [Units] | no buildings. Fields 1 to 6 required. Absent: no tag, computer-sell, computer-rebuild, computer-repair and nominal flags clear, switched-on flag set, upgrade count 0, searchlight value 1, upgrade names carried over from the previous entry that gave them, otherwise none (`quirks.upgrade_fields_carry_over`) |
| [Terrain] n | terrain object at cell n (claim 42): key decoded 1000-based at format version 4 and above, 128-based below it, wrapped per coordinate; value = terrain object type name, without regard to letter case; <none>, none and an empty value place nothing; an unknown name creates the type (S-0004) | key: cell number (leading integer); value: string | no terrain objects |
| [Smudge] (any key) | smudge: type name, cell x, cell y, piece number (claim 42); placed only when the piece number is absent or 0; covers the type's width × height cells from (x, y) | comma-separated list as [Units]; cell always two fields | no smudges |
| [Tubes] | tube (tunnel) definitions; their format has no spec yet | lists | none |
| [Houses] (any key) | map-inspection house names and their own section names (claims 47–48); not read in skirmish | string value: first 19 bytes, then trim; entries in section order, with duplicate-key lookup as S-0002 claim 4 | no listed houses: map inspection creates one house per country, using the first 20 bytes of its type name |
| map house's own section: Country | country of a listed house (claim 47); not read in skirmish | first 127 bytes, then trim; ASCII-case-insensitive whole-name match to country type name or Name field; earliest matching country wins; numeric text is a name | absent or empty: country index 0; unknown nonempty name creates a default country with a 24-byte type name |
| rules country: Name | alternative country name accepted by the house's country selector (claim 47) | imported name string, as S-0004 claim 9 | initially its type name |
| map house's own section: Country = <random> or <Player @ A> … <Player @ H> | reserved selectors invalid for this inspection import (claim 47) | random token ignores ASCII case; player-at-position tokens match exact case | our importer uses the absent-selector fallback and warns |
| country and map house's own section: Color | inherited country colour and optional house replacement, respectively | S-0103 claim 14 and its Import mapping own the name lookup, 31-character limit and defaults | S-0103 claim 14; no colour fallback for an unresolved object owner (claim 48) |
| [Basic] Player | not read in skirmish | — | — |

Keys from other files that this spec relies on (theater file and rules file):

| Key | Concept | Parse form | Default when absent |
|---|---|---|---|
| theater file [General] ClearTile | tileset whose first tile is the clear base tile (claim 6) | integer tileset number | none: clear base tile −1 |
| theater file [General] WaterSet | tileset whose first tile is the water base tile (claim 6); ignored for theater 5 | integer tileset number | none: water base tile −1 |
| theater file [TileSetNNNN] TilesInSet, LastTilesInSet | actual tile count C and authored tile count A of a tileset (Formulas, tile translation) | integer | A absent: no translation pair for that tileset |
| rules overlay type: Crate | crate type (claim 12) | boolean | not a crate type |
| rules overlay type: CellAnim | the overlay type's cell animation (claims 12 and its *Cell animations* part; Terms). The importer's "has artwork or a cell animation" test of claim 12 reads this same value. Low-bridge markers and their direct end-row and piece writes do not start it (S-0202 claim 14). The name is compared without regard to letter case; a name that is not yet an animation type creates one (S-0004 claim 13 and its Import mapping). The values <none> and none (any letter case) mean no cell animation and clear a value an earlier layer gave | animation type name (string) | none when the type is created; a layer that omits the key or gives it an empty value keeps the value in force (S-0006 claim 3) |
| rules terrain type: Strength | the terrain type's strength, and so a map-placed terrain object's starting health (claim 42); how it is read and filled is S-0102's (claim 23) | integer, health points | unset; filled from the game-wide tree strength (S-0102 claim 23) |
| rules [General] TreeStrength | the game-wide tree strength that a terrain type without a strength takes (claim 42; S-0102 claim 23) | integer, health points | 25 (S-0102) |
| rules country: MultiplayPassive | passive house (claims 19, 28) | boolean | false when the country is created; when a later read or rules layer omits the key, the value already in force is kept (S-0006 claim 3) |
| rules smudge type: Width, Height | the w × h cells a smudge covers (claim 42) | integers | 1 and 1 |
| rules [General] BaseUnit | the base-unit list (claim 45): its first type that a house's country owns is the house's base unit; its types are left out of the budget pool and the vehicle pools | list of vehicle type names | empty: no base unit |
| rules vehicle or infantry type: AllowedToStartInMultiplayer | whether the type may be in the budget pool and a start pool (claim 45) | boolean | yes |
| rules vehicle or infantry type: Owner | the owner list: the countries that own the type (claim 45) | list of country names | empty: no country |
| rules vehicle or infantry type: TechLevel | the type's tech level, compared as a signed integer with the game's tech level and the house's tech level (Terms; claim 45) | integer | 255, so that the type qualifies only where the tech level it is compared with is 255 or more (medium confidence) |
| rules [MultiplayerDialogSettings] TechLevel | the game's tech level (Terms; claim 45): the base rules' value, copied once when the program starts. Later layers (the language rules file, the game-mode file, the map's overrides) and later games change only the rules value (S-0006 claim 10), not the copy | integer | 10 |
| rules [Maximums] Players | the maximum player count of the unused-house cleanup (claim 28), taken from the base rules only (S-0006 claim 2) | integer | 8 |
| rules vehicle or infantry type: Cost | the cost summed into the budget and taken from the remaining budget (claim 45) | integer | 0 |
| rules vehicle type: SmallVisceroid, LargeVisceroid | either one on: the placement draw R(0, 29) of claim 46 | boolean each | no |
| rules [MultiplayerDialogSettings] UnitCount, Bases | the defaults that the setup screen offers for the unit-count setting *U* and the bases setting (claim 45); the game uses the setup's values | integer; boolean | not established |

House names matched by pre-placed objects in skirmish (claim 27): the neutral and special houses
answer to their country names (Neutral, Special). The original finds those two countries by
these fixed names, so they are engine constants, not stock data. Human players (outside online games) answer to the placeholder
<human player> and computer players to Computer. The player-at-position names used by triggers and
teams are <Player @ A> … <Player @ H>, matched case-sensitively. These names, and the map-list
defaults "No Name" and "No Digest", and the two "no tag" markers <none> and none of the cell-tag
values (claim 31), which also mean "no terrain object" in the terrain-object list (claim 42), are fixed text inside the original's program (engine constants), not game text
from the data files.

## Measurement scenarios

All scenarios run under the measurement setup (glossary): an unmodified install with no engine
add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted), stock data, and tiny purpose-made test maps
(e.g. a 20 × 20 flat clear map) with format
version 4, unless stated otherwise. No loose file overrides shipped data, except a named test file
that a scenario lists (M4c, M16 variant F2, M16b variant C, M26 variants F and H, and M19
variant J and M21 variant E when they use one),
which is removed after that scenario's runs. A
presentation-only display compatibility layer may be used, and the measurement record names it
generically, by what it does. Positions are read from the game's own coordinate display or
by placing a marker unit. "Runs" counts game launches. Deterministic results need one run per
variant. Map sizes are given as the third and fourth numbers of the size entry (W × H), with the
first two numbers 0.

**Default players and map settings.** Unless a scenario says otherwise:
- Game speed is index 2 throughout each run; observations use tick numbers, not wall time.
  Detail level is high. Screen recordings and first-tick saves record visible outcomes.
  Scenarios naming the random-state recorder additionally record the scenario seed and stream
  position. An inconclusive run supplies no backing and must be redesigned or repeated.
- Any mismatch with a deterministic expected result below falsifies that tested part, even
  when its individual falsifier lists only representative alternatives. Statistical tests
  use their stated rejection criterion rather than a single coincidental result.
- It is a skirmish game with 1 human and 1 computer player on fixed positions 0 and 1. Its map
  defines start waypoint markers 0 and 1 on legal cells, at least 3 cells apart, and no other start
  waypoint markers. No random start cell is then needed. This matters on the smallest maps
  (M1, M2, M2b), where an 8 × 8 clear area cannot fit and a random start would never finish
  loading (claim 18).
- Every scenario keeps its participants, players and observers alike, within the lobby limit of
  claim 23. Each scenario that departs from the default states its start waypoint markers.
- The map's local area entry covers the whole map, (0, 0, W, H), so that claim 30's absent entry
  (no usable cell) does not apply. A scenario that gives its own local area or removes the entry
  says so (M3, M3b, M3c, M23 variant B, M26 variant E).
- Scenarios that read the minimap (M1, M3, M3c) set the map's free-radar switch, so the minimap is
  shown without a radar building.
- Scenarios that read tiles, heights or overlays away from the players' starting units give the
  map a trigger that reveals the whole map to the human player at the start. M11, M13b variants
  E and F, M16, M17, M18 and M19 variant H use a reveal as their observable and have no such
  trigger.

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
  pattern may differ between launches; an identical pattern is not itself a falsifier.
  Then, without restarting the game, play a snow map and then the
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
  the no-tile marker, which paints as S-0010 claim 24 says). The height levels are
  the deciding observable; the tiles confirm them. *Falsified if* cell (1, 1) is not at
  level 2, or any of (2, 2) … (5, 5) is at level 2 or shows a water tile. That the fill then takes
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
  changes. *Partial-value variants:* use S-0012 scenario R2's complete-chunk variants,
  each after an earlier complete record: three coordinate bytes with legal x and y below 256,
  and one, two or three tile bytes. Record the selected cell's tile and height at the first
  view. Include an earlier duplicate with a visibly distinctive sub-tile and height, then
  cut the later record before either value: each omitted value must retain the earlier
  duplicate's value rather than restore the fill. Also repeat the two-/three-tile-byte
  variants as the first record with complete coordinates, using valid distinctive tiles.
  *Expected:* the coordinate and tile results of S-0012 claim 9 and F3, translated by
  claim 8, including those first-record tile variants. *Falsified if* a selected legal cell,
  a valid two-/three-byte tile control, or a retained sub-tile/height differs. The exact
  one-tile-byte number and missing ice-growth retention remain unmeasured; an unspecified
  out-of-range picture is not a falsifier. For zero through three expanded bytes before
  the first complete coordinate pair, predict no original coordinates; our engine must
  apply no record and warn. Replay on our engine with the partial-value switch off:
  partial coordinates stop the stream and partial tiles use 65535, while complete
  one-byte values still apply. Failure of these warnings or alternate outcomes falsifies
  the engine requirement, not an original-game claim.
  *Selected-cell variant (claim 7, quirk `quirks.cell_index_alias`):* a 20 × 20 map whose terrain
  stream holds one complete record with coordinates (527, 10) and the tile of a clearly different
  tileset from the rest of the map (for example a pavement tile on a map of clear ground), then a
  (0, 0) record. 1 run. *Expected:* cell (15, 11) shows that tile, and no other cell changes.
  *Falsified if* (15, 11) keeps the fill tile, or the tile appears anywhere else.
  *Height variant:* base level 1. The terrain stream holds complete records, each with a flat
  tile, for a legal cell A at record height level 3 and a legal cell B at record height level −1,
  and ends with a (0, 0) record. A legal cell D has no record, so it keeps the fill's level 1.
  A, B and D are at least 3 cells apart and have equal screen row x + y, so their
  level-0 centre projections have the same vertical coordinate. Use the same vehicle type
  and facing for all three. A pre-placed vehicle owned by the human placeholder name
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
  a tile from another position. The translation itself is probed by M4c. *Exploratory second
  part (claim 8):* one more record whose tile number is at least the theater's tile count
  (count the tiles in the theater data) and below 65535. Record the picture or abnormal end.
  The original has no specified picture for that cell (S-0010 claim 24), so this part has no
  conformance expectation or falsifier; it does not back a `measurable` claim. Our engine's
  separate check expects the clear tile.
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
- **M5, level height (claim 9).** Flat cells at levels 0 and 4, with equal x + y and
  separated horizontally on screen, with one unit of the same type and facing standing on
  each. 1 run. Observable: the vertical distance between matching image anchors, in pixels.
  *Expected:* the distance is L(416) − L(0), using S-0101's object height lift.
  *Falsified if* it differs. A screen-pixel match does not distinguish every possible
  lepton height that quantises to those same pixels; S-0106's scenario T1 provides the
  finer surface-height checks.
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
- **M6d, slope condition for overlays (claim 12).** Format version 4, a 30 × 30 map. *T* is an
  overlay type with artwork that is neither a resource (its land type is not *tiberium*) nor a
  wall, has no cell animation and is not at list index 178; check this in the rules data first,
  because a recalculation removes a resource overlay from a sloped cell (S-0211 claim 7) and a
  wall has placement checks of its own. Part A: the terrain stream gives a row of cells tiles
  and heights that produce slope shapes 0 to 20 after recalculation (S-0106), and the overlay
  type stream puts *T* on each of them. Part B: an end
  marker of position 124 on a flat cell (*x*, *y*) whose east neighbour (*x* + 1, *y*) has a slope
  shape above 4, and *T* on (*x* + 1, *y*), which comes after the marker in the overlay type
  stream. Part C, the control: the same as part B with a slope shape of 1 at (*x* + 1, *y*).
  1 run, read from a save at the first tick.
  *Expected:* in part A every cell holds *T*, because no cell had been recalculated when its
  overlay was placed, so each slope shape still read 0. In part B, (*x* + 1, *y*) still holds the
  end-row type 96 of S-0202 claim 10: the end row's recalculation gave it its slope shape, and *T*
  was refused. In part C it holds *T*, which replaced the end-row overlay. *Falsified if* a part-A
  cell lacks *T*, or the part-B cell holds *T* while the part-C cell does. *Inconclusive if* the
  part-C cell keeps type 96 (then a placement does not replace an end-row overlay, and part B
  shows nothing).
- **M6e, cell animation position and start event (claim 12, *Cell animations*).** Format version 4,
  a 30 × 30 flat clear map at level 0, revealed to the human player. *T* is an overlay type as in
  M6d (with artwork, not a resource, not a wall, no cell animation in the rules data, not a crate
  type) and *A* a stock animation type with a long, clearly visible first image frame and no
  follow-up type. The map's override sections (S-0006) give *T* the cell animation *A* (Import
  mapping) and give *A* the resource chain-reaction setting of S-0109 (claim 13; its Import
  mapping). The overlay type stream puts *T* on cell (12, 12) in parts A to C. Part A: one ore
  cell at (14, 14) and no other resource within 6 cells. Part B, the control: one ore cell at
  (12, 15) instead, which touches neither (12, 12) nor (14, 14), and none at (14, 14). Part C: as
  part A without *A*'s chain-reaction setting. 1 run per part, read from a save at the first tick
  and watched on screen.
  *Expected:* in every part the animation appears in the screen column of cell (12, 12), one and
  a half cell heights below that cell's centre (at the top corner of cell (14, 14)), and its start
  sound plays as the game starts. In part A the ore at (14, 14) is gone at the first tick; in
  parts B and C the ore cell is still there. *Falsified if* the animation appears centred on
  cell (12, 12), part A's ore remains while part C's also remains, or part B's ore at (12, 15) is
  gone (the chain reaction acted on another cell than (14, 14)). *Inconclusive if* part C's ore is
  gone too (then something other than the chain reaction removed it).
  Part D, order of draws, with the random-state recorder: *A* without the chain-reaction setting
  but with a random rate pair that gives a range of 3 values (S-0109 claim 3, so its stage-delay
  draw may take more than one raw value), and *T* placed on the attaches-to cell of a stock tile
  whose tile animation (S-0010 claim 19) also has such a pair, of another range, and no other
  start-event effect. Nothing else on the map draws in the overlay type stream's part of the
  load. 3 runs. From the recorded seed, predict the load's draws twice: with the cell
  animation's draw first and with the tile animation's draw first. Each prediction gives the cell
  animation's stage delay and the stream position at the first tick (the tile animation itself
  may be removed later in the load, S-0010 claim 19, but its draw stays taken). Read the cell
  animation's first image-frame change from saves at consecutive ticks, and the stream position
  with the recorder. *Expected:* the prediction with the cell animation first holds. *Falsified if* only the other prediction holds. *Inconclusive if*
  the two predictions coincide (choose other seeds or ranges).
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
  position 1 at the random cell predicted for list 3 from the recorded seed by claims 18
  and 20 (use the random-state recorder). A random cell may coincide with either named cell.
  That coincidence alone says nothing. *Falsified if* the view starts elsewhere, the lobby
  allows 3 players or only 1, or position 1 differs from the seed-based prediction.
- **M8, start list with a gap (claim 17).** A 40 × 40 flat clear map with start waypoint markers
  0, 1, 3 and 5 defined and no others, so the lobby limit is 4 (claim 23). 4 players (1 human,
  3 computers) with fixed positions 0 … 3. K = 2 and N = 4, so the start list is [wp0, wp1, wp3,
  random]. 3 runs with the random-state recorder. *Expected:* position 2 starts at waypoint
  3's cell in every run. Position 3 starts at the random cell predicted for list 3 from the
  recorded seed by claims 18 and 20. *Falsified if* position 3 differs from that prediction,
  or position 2 starts anywhere but waypoint 3. Coincidence of a random cell with a waypoint
  cell, or repetition across runs, is not itself a falsifier.
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
  near or beyond the edge of the legal area; the comparison uses claim 18's ring collection,
  scan order and visibility preference, with tick number 0, not a nearest-cell approximation. *Expected:* the counts in each of the four bands
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
- **M9d, the start-cell search (claim 18), with the random-state recorder.** A 40 × 40 map, all
  water except one land block of 10 × 8 clear, flat cells (10 along x, 8 along y) at least 12
  cells from every edge, with no start waypoint markers, one human and one computer player, the
  bases setting on. Lists 1 and 2 are found on blank cells, so the late phase's list 3 is the one
  that sees the block. The only corners whose start area is clear are the three cells
  (*x*₀, *y*₀), (*x*₀ + 1, *y*₀) and (*x*₀ + 2, *y*₀) of the block's corner of lowest x and y.
  20 runs. From the recorded seed compute every list-3 candidate and, by claim 18, the ring and
  the cell that the scan order picks (or the candidate's failure, with its pair of draws).
  *Expected:* each player's base unit stands on one of the three corner cells, the one predicted
  for its list-3 entry, in every run. Under a start area centred on the examined cell, the
  base units would stand about four cells further into the block. Under a choice by distance, the
  picked corner would differ from the scan-order pick in the runs where the two rules disagree
  (expected in several of 20 runs). *Falsified if* a base unit stands elsewhere than the predicted
  corner.
- **M10, farthest-sum (claim 19).** 4 start waypoint markers at known cells, 3 players (1 human,
  2 computers) all on "random", 50 runs. House order is the human, then the computers in lobby-slot
  order. *Expected:* the human's position is roughly uniform over the 4 (each between 5 and 20 of
  the 50 runs). The first computer is always at the position farthest from the human. The second
  computer has the largest distance sum, with ties going to the lower position.
  *Colour-order sub-test (claim 19's house order of several humans; run 4 also claim 22's taken branch).* It needs a game of several
  human players on the stock executable; no measurement setup runs one yet. Every start is on
  "random". Runs 1 to 3 use 4 start waypoint markers in one row, 10 cells apart, numbered 0 to 3
  from the row's west end. Every run records each player's start position.
  *Telling runs.* The first player in the house order takes the drawn position, and every later
  position follows from it by the farthest-sum rule, so for each candidate order of the humans
  the analysis computes the positions that order would give from its first player's observed
  position. A run is telling when only one candidate order reproduces every observed position.
  The other runs back nothing. On this row, a draw of marker 1 or 2 makes run 1 telling (half
  of all draws), and only a draw of marker 2 makes runs 2 and 3 telling (a quarter).
  - Run 1: 2 humans in lobby slots 1 and 2 with chosen colour slots 5 and 2, and 1 computer,
    10 runs, then the same with the two humans' colour slots swapped (2 and 5), 10 runs.
    *Expected:* in every telling run the human with the lower colour slot is
    first, so the roles of the two humans swap with the colours. Ten runs leave no telling run
    about once in a thousand series.
  - Run 2: 3 humans in lobby slots 1, 2 and 3 with chosen colour slots 5, 1 and 3, and 1
    computer, 20 runs. *Expected:* every run is reproduced by the house order of the humans of
    lobby slots 2, 3 and 1, then the computer, and in every telling run only by that order.
    Twenty runs leave no telling run about three times in a thousand series.
  - Run 3: as run 2, but the humans of lobby slots 1 and 2 choose colour slots 1 and 6 and the
    human of lobby slot 3 leaves its colour on "random", and the computer is given colour slot 7,
    40 runs, recording the slot it is given.
    *Expected:* every run is reproduced by the order in which that human stands among the other
    two by the slot it was given, as if it had chosen it, and in every telling run only by that
    order. The alternatives are placing that human first, after every chosen colour, or by its
    lobby slot; from lobby slot 3, outside the other two humans' lobby slots, the last is the
    same order as after every chosen colour. A given slot between the other two humans' slots,
    2 to 5, puts it between them, which tells this rule from all three alternatives. A given
    slot of 0 puts it first, which tells the rule only from the last two. Slot 7 is the computer's,
    so it cannot be given (S-0412 claim 7). A run backs the rule only against the
    alternatives it tells apart. If the given slot is equally likely to be any of the five
    free slots (0 and 2 to 5), so from 2 to 5 in four draws of five, and with a telling position
    draw in one of four, 40 runs leave no run telling all three apart about 13 times in
    100 000 series.
  - Run 4: as run 2 with one more human set to observe, in a lobby slot after the computer's,
    20 runs, on a map with 5 start waypoint
    markers in one row, numbered 0 to 4 at 30, 0, 10, 20 and 40 cells from the row's west end,
    recording the position the observer is shown at the start. *Expected:* by claim 22, the
    observer is shown the lowest-numbered of the three humans' positions. An observer whose turn
    came first would be shown the first human's position, which differs from that whenever the
    draw is not of marker 0 (four draws in five). One whose turn came after the computer would
    be shown the lowest-numbered of all four taken positions, which differs whenever the draw
    is of marker 2 or 3 (two draws in five), since the computer then holds marker 0. An order
    by lobby slot would put the observer after the computer, which is this last case. Twenty runs
    leave no run telling the second case apart in fewer than one series in ten thousand.
  *Falsified if* any run breaks the farthest-sum rule, a position is never or almost always drawn
  first, a run of runs 1 to 3 is not reproduced by the predicted order (for a human on
  "random", the order its given slot predicts), or the observer is shown a position other than
  the lowest-numbered of the three humans' positions. That last result falsifies either claim
  19's place for observers, against the two rival places run 4 tells apart (first in the house
  order, or after the computer, which an order by lobby slot also gives), or claim 22's rule for
  an observer whose turn comes after a position is taken; run 4 alone does not say which.
- **M10b, tabulated square root (claim 19, quirk `quirks.table_root_distance` of S-0204).** A 140 × 140
  flat clear map with start waypoint markers 0 at (80, 150), 1 at (171, 241) and 2 at (209, 150),
  all legal. 2 players: the human on fixed position 0 and 1 computer on "random", so N = 3 and no
  random start cell is needed. Squared distances from waypoint 0: 2 × 91² = 16562 to waypoint 1
  (exact distance about 128.7) and 129² = 16641 to waypoint 2 (exactly 129). Both terms are 128
  (Formulas), so the tie goes to position 1. 3 runs. *Expected:* the computer starts at waypoint
  1's cell in every run. *Falsified if* it starts at waypoint 2's cell (exact distances). Control:
  waypoint 2 moved to (210, 150), squared distance 130² = 16900, term 130, 1 run. *Expected:* the
  computer starts at waypoint 2's cell. *Inconclusive if* the control gives waypoint 1.
- **M10c, start cells far outside the map (claim 19, Formulas, *distance sum*).** A 20 × 20 flat
  clear map with the bases setting off, so that no base unit is placed on a non-legal cell (claim
  46). 2 players, 1 human and 1 computer: the player on fixed position 0 and the player on
  "random" are named per variant. Start waypoint markers 0, 1 and 2 are defined and no others, so
  K = N = 3 and no random start cell is needed. Read from a save at the first tick.
  *Inconclusive if* the game does not finish loading, or ends before the first tick can be saved.
  - Variant A, the reduction of a difference: waypoint 0 holds the cell number −32760005, which
    names the non-legal cell (−5, −32760) (Formulas, cell number decoding); waypoint 1 holds 15025,
    cell (25, 15), and waypoint 2 holds 35025, cell (25, 35), both legal (s = 40 and 60). From
    waypoint 0, the y differences 32775 and 32795 reduce to −32761 and −32741, so the terms are
    32760 for waypoint 1 and 32740 for waypoint 2. The computer is on fixed position 0, the cell
    far outside the map, and the human on "random", so the human is placed by the farthest-sum
    rule and its view and units are on a legal cell. 3 runs. *Expected:* the human's starting
    units stand around waypoint 1's cell in every run. *Falsified if* they stand around waypoint
    2's cell in any run (the differences were not reduced).
  - Variant B, the reduction of a term: waypoint 0 holds 20020, cell (20, 20); waypoint 1 holds
    −32747500, the non-legal cell (−500, −32747); waypoint 2 holds 25030, cell (30, 25). From
    waypoint 0, waypoint 1 gives Δx = −520 and Δy = −32767, n = 1073946689 and ⌊r(n)⌋ = 32769,
    which reduces to the term −32767; waypoint 2 gives the term 11. The human is on fixed
    position 0 and the computer on "random". 3 runs. *Expected:* the
    computer's starting units stand around waypoint 2's cell in every run. *Falsified if* the
    computer is placed at waypoint 1 in any run, which shows as no computer units around
    waypoint 2's cell (the term was not reduced).
- **M11, player at position (claim 21).** A trigger that reveals the map to "player at position B",
  2 players with fixed positions 0 and 1, then the same with the positions exchanged. 2 runs.
  *Expected:* the player at position 1 gets the reveal both times. *Falsified if* the reveal follows
  the player rather than the position.
- **M12, lobby limit (claims 15, 23).** A 40 × 40 flat clear map with start waypoint markers 0, 1,
  2 and 5, with waypoint 2 stored as 0. 1 run of the lobby plus 1 game with 4 players (1 human,
  3 computers) on fixed positions 0 … 3. *Expected:* the lobby allows 4 players, and in game
  the start list is [wp0, wp1, random, random], and position 2 starts at the list-3 cell
  predicted from the recorded seed by claims 18 and 20. Use the random-state recorder.
  *Falsified if* the lobby allows a different number or position 2 differs from that
  prediction. A random cell coinciding with waypoint 5 is not itself a falsifier. *Map-list label:* two loose map files, one
  with minimum and maximum player settings both 3, one with 2 and 5, lobby only. *Expected:* the
  map list labels the first with a single number and the second with a range. *Falsified if*
  either label has the other form.
- **M13, preview (claims 24, 25 and 25a).** A map whose header block lists 2 markers at positions different
  from its 2 start waypoint markers. 1 run. *Expected:* the preview markers are drawn at the header
  positions and players start at the waypoint cells. The same map with the preview stream shortened
  so that the last row is missing, 1 run. *Expected:* the preview is still shown, with all rows but
  the last as encoded. The same map without a header block, 1 run. *Expected:* no markers. The
  same map with its preview size entry removed (the preview stream kept), 1 run. *Expected:* no
  preview image is shown in the map selection screen (claim 30). *Falsified if* the markers follow
  the waypoint cells, the shortened preview disappears entirely or a complete decoded row
  differs from its encoded colours, markers appear without a header block, or a preview
  image appears without a preview size entry. The omitted row has no expected original
  colour; it may coincidentally look complete and cannot falsify the claim.
  *Channel-order map:* the first map with a preview of 32 × 16 pixels whose decoded values give
  the left half of the top 8 rows the colour (255, 0, 0), the right half of those rows (0, 255, 0)
  and the bottom 8 rows (0, 0, 255), each written in stream order as three values, 1 run.
  *Expected:* in the map selection screen the preview's top left quarter is red, its top right
  quarter green and its bottom half blue. *Falsified if* red and blue are exchanged (the values
  read in the order blue, green, red), or the green part appears in the bottom half or the blue
  part runs from top to bottom on one side (rows and columns exchanged).
  *Wrap variant (Formulas, preview marker position):* the first map with N_h = 1, marker origin
  X0 = −2147484 and Y0 = 0, marker extent Wh = Hh = 1000, and pair 1 = (2147984, 500), 1 run,
  then a control with X0 = 0 and pair 1 = (500, 500), the rest unchanged, 1 run. *Expected:*
  marker 1 is drawn at the same place in both, near the centre of the preview (inner quotients 500 and
  500). *Falsified if* the two markers are drawn at different places, or the wrap variant draws no
  marker on the preview while the control does (the product then did not wrap).
  *Agreement variant (claim 25a):* a map of size 40 × 40 with start waypoint markers 0 and 1 only,
  at cells (30, 25) and (25, 50), and a preview of 78 × 39 pixels, all black except one white pixel
  for each waypoint marker, at column 2·(u − u0) and row v − v0 of its projection (Formulas,
  waypoint form): column 44, row 7 and column 14, row 17. Its header block has N_h = 2, origin
  (−20, 21), extent (39, 39) and pairs (2, 28) and (−13, 38). 1 run, map selection screen.
  *Expected:* marker 1 is drawn near waypoint 0's white pixel and marker 2 near waypoint 1's, each
  with the same displacement from its pixel, within 2 pixels. *Falsified if* the two displacements differ by
  more than 2 pixels in either direction, or a marker is nearer the other marker's pixel.
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
    10 runs with the random-state recorder. *Expected:* the human starts at the position
    predicted by its late-phase random-position draw (claim 29), and the computer always
    starts at the waypoint cell farthest from the human's. Repeated positions are allowed.
  - Control: variant B with N_h = 8. 2 runs. *Expected:* the human starts at waypoint 0 and the
    computer at waypoint 1.
  - Variant C, pairs above N_h are not read: N_h = 9, pair 9 absent, pair 10 = (1, 0) present,
    both players on "random". 10 runs with the random-state recorder. *Expected:* pair 10
    has no effect. The human starts at the position predicted by its early-phase draw, and
    the computer always starts at the waypoint cell farthest from the human's.
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

  *Falsified if* variant A ever differs from its expectation, variant B's human start
  differs from its seed-based prediction or its computer is not farthest from the human,
  the control moves either player, variant C's human start differs from its seed-based
  prediction or its computer is not farthest from the human, variant D ever
  differs from its expectation, variant E gives no reveal (the reference then follows where a
  player starts, not the mark) or starts the human at waypoint 0's cell (the lowest-numbered
  position held instead of the highest), or variant F ever starts a player at waypoint 0 or 1, ever
  gives a reveal, or starts the players differently from its expectation. *Inconclusive if* the
  reveal trigger fails in a control map where "player at position A" is the human's own fixed
  position 0 (N_h = 8, 1 run): the trigger is then at fault.
  - Variant G, deterministic completion (our engine only): use variant B with pair 9's
    nonempty value giving only −1. In the early phase its unread second component uses the
    mark in force, −1 before chosen positions are assigned. In the late phase it uses the
    mark left by the early phase, house number 1. Thus only house 0 needs re-assignment
    in the late phase. Record the marks, base cells, draw trace and importer warning in one
    run, then compare with a complete-pair control computed from the marks at each phase.
    Also test a value with no readable first component: both marks stay in force at each
    read and a warning appears. A mismatch with these per-phase expectations or a missing
    warning falsifies our engine's departure. These runs predict no original result for
    unread components.
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
- **M15, observers (claim 22; bears on the original's place of a single observer, open in claim
  19: not after both computer players).** Runnable only if the original's skirmish lobby can
  start a game with an observer, which S-0414 leaves open; if it cannot, M15 is void. 2 computer
  players on "random" plus one human observer, on a map with 4 start waypoint markers at the corners of a rectangle, so that each position's farthest one is the
  opposite corner, 10 runs. Record the position the observer is first shown and each
  player's start. *Expected:* the first computer player in house order always starts at the
  position the observer was first shown. *Falsified if* it ever starts elsewhere. If the observer
  came after the computer players, it would be shown the lower-numbered of their two starts, and
  the first computer player would start there in about half the runs, so 10 passing runs leave
  about 1 chance in 1000 of missing that order. An observer between the two computer players
  would be shown the first one's start, as one before both would be, so M15 cannot detect that
  place. A failure cannot tell the observer's place from claim 22's rule.
  The branch for an observer whose turn comes after a position is taken is M10's colour-order
  sub-test, run 4.
- **M16, unused-house cleanup (claim 28).** Only variants B and G below are proposed
  skirmish measurements. The older network variants A, C, D, E and F, including D's second
  part, are deferred context and supply no backing in this revision. Before running, check that the base rules' maximum
  player count (Import mapping; 8 when they omit it) is above 2, and that the rules data marks both
  the neutral country and the special country passive. If any check fails, the variants below do
  not separate the cases and the scenario has to be redesigned. Also choose, from the rules data,
  a vehicle type *V*ₙ and an infantry type *I*ₙ that are in the budget pool of claim 45 for the
  players' countries and whose tech levels are above 1, and check that no other vehicle or
  infantry type that is allowed to start in multiplayer games has tech level 2 or lower and the
  neutral or special country in its owner list.
  The bases setting is off in every variant: with it on, a non-passive neutral or special house
  would need a base-unit type that its country owns, and the early stop of claim 45 would hide
  its starting units.
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
  variant D. The map also has type override sections (S-0006) for *V*ₙ and *I*ₙ. Each gives the
  type's owner list from the rules data with the neutral country added (in variant D the special
  country as well), and the tech level 1. The players' pools keep both types, since the game's
  tech level is above 1 (the pre-check found types above 1 in the budget pool), and a neutral
  house that receives starting units has exactly *V*ₙ as its vehicle pool and *I*ₙ as its
  infantry pool (claim 45, with its tech level 1). Players on fixed positions 0 and 1. Each
  variant runs once, in a fresh game launch, so that no country data from an earlier game is in
  force.
  - Variant A: a network game with 2 humans and no computer player. *Expected:* once the map has
    loaded, neither the structure at (35, 30) nor the vehicle at (36, 30) is there, the map is never
    revealed, the humans start at waypoint 0's and waypoint 1's cells, and the neutral house
    receives no starting units anywhere (the cleanup removed it).
  - Variant B, skirmish condition control: the map above with 1 human at fixed position 0
    and 1 computer player at fixed position 1. *Expected:* the structure and the vehicle are
    present, the map is revealed after the delay, the players start at waypoint markers 0 and 1,
    and starting units of the neutral house, all of type *V*ₙ or *I*ₙ, appear around waypoint
    2's cell (its distance sum 40 exceeds waypoint 3's sum 30),
    each in the *area guard* behaviour (claim 45; read from a save, as in M26).
    *Falsified if* an object is missing, the reveal fails, a player or the neutral house starts
    at another waypoint, an unexpected neutral starting type appears, or its starting behaviour
    differs. In-scope control B0 uses the same skirmish without the neutral country's override:
    the pre-placed objects and reveal remain, but no neutral starting units appear. If B0
    lacks an object or reveal, that corresponding part is inconclusive rather than backed
    by the deferred network control C.
  - Variant C, passivity control: variant A's game without the neutral country's override section
    (the type override sections stay). *Expected:* the
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
    structure and vehicle are there, the map is revealed after the delay, and starting units of
    the special house, all of type *V*ₙ or *I*ₙ, appear around waypoint 2's cell (it is the only
    house that needs a position in the late phase), each in the *area guard* behaviour.
    *Falsified if* the special house's units start in another behaviour, or the special house's objects are
    missing or the map stays hidden (the cleanup removed the special house too: the fixed
    behaviour), or the neutral house's objects are present. Second part, the special house's
    tech level: variant D with the two type override sections giving the tech level 2 instead of
    1. *Expected:* as in variant D, except that the special house receives no starting units
    anywhere (empty pools at its tech level 1, claim 45). *Falsified if* starting units of the
    special house appear while they appear in variant D.
  - Variant E, a trigger created at the end of loading: variant A's game, with the cell-tag entry
    removed, so that nothing carries the neutral trigger's tag and the tag is created at the end of
    loading. *Expected:* the structure and the vehicle are gone as in A, but the map is revealed
    after the delay: the trigger survives the cleanup although its owner house is gone. *Falsified
    if* the map stays hidden while it is revealed in C. *Inconclusive if* S-0011 makes the reveal
    action depend on the trigger's owner house; the variant then needs an action that does not.
  - Variant F, the maximum player count. (F1) Variant A's game, with a rules override section in
    the map that sets the maximum player count (Import mapping) to 2. *Expected:* as in variant A,
    because the count comes from the base rules only. (F2) Variant A's game with a named test
    file: a loose copy of the rules file with one change, the maximum player count set to 2; the
    program is started with it in place, and it is removed after the run. *Expected:* the cleanup
    does not run, because 2 human slots are not fewer than 2: the structure and the vehicle are
    present, the map is revealed after the delay, and the neutral house takes the late-phase
    position by the farthest-sum rule (claim 19: waypoint 2, sum 40, against 30 for waypoint 3),
    so its starting units appear around waypoint 2's cell. *Falsified if* the structure or the
    vehicle is present in F1 (the map reached the count), or either is missing in F2 (the count
    does not come from that key, or the cleanup also runs when the slots equal the count).
  - Variant G, the neutral house's tech level: variant B, with the two type override sections
    giving the tech level 2 instead of 1. *Expected:* everything as in variant B, except that the
    neutral house receives no starting units anywhere: its pools are empty, because its tech level
    is 1 (claim 45, early stop at its turn). *Falsified if* starting units of the neutral house
    appear (its tech level would then be 2 or more) while they appear in B. Together with B, which
    shows that types of tech level 1 qualify, this fixes the neutral house's tech level at 1.

  *Falsified if* the structure or the vehicle is present in variant A, either is missing in
  variant B while both are present in C, a player in A or B starts elsewhere than at the waypoint
  of its chosen position, the map is revealed in A although it is revealed in C, the neutral
  house receives starting units in A, or in B around any waypoint but 2, or of a type other than
  *V*ₙ and *I*ₙ. Starting units of the neutral house that are missing in B falsify only the
  starting-units sentence of claim 28 or the neutral house's tech level 1 (Terms), not the
  cleanup; variant G then says nothing. Starting units of the neutral house that appear in B in
  another behaviour than *area guard* falsify only the behaviour rule of claim 45.
  *Inconclusive if* the pre-placed objects are missing in C as well (the removal then has another
  cause), or the map is not revealed in C (the trigger is at fault, so the trigger sub-tests of A
  and E say nothing). In A, B, C, F and G the special country stays passive (pre-check), so in B
  and G the neutral house is the only house besides the players that needs a position, and it
  finds two free positions; in F2 it is the only one and finds two, and in D the special house is
  the only one and finds two. No house needs a position
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
- **M19, object entry fields and starting health (claims 34 to 39, 41).** A 20 × 20 flat clear map,
  format version 4, revealed to the human player except in variant H. Pre-placed objects owned by
  the computer placeholder name unless a variant says otherwise, each on its own cell at known
  coordinates. Health is read from the health bar (S-0111) and, where the bar's
  resolution is not enough, by counting the shots of a known weapon that destroy the object; each
  type's strength *S* is known from the rules data. 1 run per variant.
  - (A) One vehicle of each of three stock types with different strengths, each of at least 5
    (below that, the near-full rule also raises the value 128), at health values 256,
    255, 254, 128, 64, 1 and 0. *Expected:* the health of Formulas (object health): 256, 255 and
    254 give full strength whenever trunc(*H* × *S* / 256) > *S* − 3; 128 gives exactly
    ⌊*S*/2⌋ (truncated); 1 and 0 give 1 when *S* < 256. *Falsified if* any value differs by more
    than the display's resolution, or a near-full value shows below full.
  - (B) The same health values for an infantry unit, plus a negative value (−10) for it, and the
    values 256, 254, 128 and 2 for a building type with *S* of at least 256. *Expected:* infantry
    at 0 and −10 start with 1, and the building's health follows Formulas (object health). The
    building's low end (a result of 0 or less) is unspecified and not probed. *Falsified if* the
    infantry unit starts with 0 or below, or a building value gives another health.
  - (C) Type fields written in lower case, an infantry type named in the vehicle list, and a
    misspelt type name. *Expected:* the lower-case entries appear, the other two do not.
    *Falsified if* a lower-case entry is missing or a wrong-kind entry appears.
  - (D) An entry of the form owner, type, 256, an empty field, 10, 12, 64, and an entry padded
    with spaces after its seventh field so that its tag name field starts at character 130.
    *Expected:* the empty field collapses, so the first vehicle stands at cell (10, 12) facing
    east (facing value 64); the padded entry's tag is not attached. *Falsified if* the first vehicle
    is missing or stands anywhere but (10, 12), or faces otherwise (the empty field counted), or the
    padded entry carries its tag.
  - (E) Facing values 0, 64, 128, 192, 320 and −64 on six vehicles, facing values 64 and 192 on
    two aircraft of a type that stands landed on open ground, and infantry facing values 0, 64 and
    absent. *Expected:* vehicle body facings 0, 16384, 32768, 49152, 16384 and 49152 facing units
    (map north, east, south, west by glossary claim G1), turrets matching their bodies; the
    aircraft face east and west; the infantry face north, east and north. *Falsified if* 320 or
    −64 face anywhere else, an aircraft or an infantry unit faces otherwise, or a turret differs
    from its body on the first tick.
  - (F) Infantry at standing position numbers 0 to 4 in five cells. *Expected:* the positions of
    S-0208 table P (scenario W2 there covers the details). *Falsified if* any stands elsewhere.
  - (G) Behaviour name (claims 35 and 41). Objects owned by the human placeholder name (claim 27
    keeps them), each with every field up to the behaviour name and no later field, on open
    ground at least 4 cells from every other object: an unarmed vehicle, an infantry unit and an
    aircraft of a type that stands landed on open ground, each once with the behaviour name
    written as S-0215's import name of *sticky* and once with a name that matches no behaviour
    (the same name with its last letter changed). No enemy comes near them. Give each, in turn, a
    move order to a cell 6 cells away. *Expected:* the three objects placed in *sticky* never
    leave their cells, because an actor in *sticky* is never ready to change, so the move order
    stays queued (S-0215 claim 25); the three objects with the unmatched name start the move and
    reach the cell, because the unmatched name is not read as *sticky* (by claim 41 it means
    *none*, and the placement fall-back of S-0215 then gives a resting behaviour that a move order
    can follow; this variant cannot tell *none* from another such behaviour). *Falsified if* a *sticky* object
    starts the move, or an object with the unmatched name does not. A *sticky* object that moves
    falsifies the field position (field 7) or the name matching.
  - (H) Tag name (claims 35 and 41). The map is not revealed. Four objects owned by the computer
    placeholder name stand within 6 cells of the human's start: a vehicle (tag in field 8), an
    aircraft standing landed (field 8), an infantry unit (field 9) and a building (field 7). Each
    carries its own tag, whose trigger is owned by the human's country and has the single event
    destroyed-by-anything (S-0011 claim 19) and the action that reveals the area around its own
    waypoint (S-0011 claim 26). The four waypoint markers are numbered 10 to 13, so they are not
    start waypoint markers and leave the start list and the lobby limit unchanged, and they lie
    in four different shrouded corners, each farther from the human's start than
    the human's units can see. A fifth object, a vehicle with no tag field, is the control. The
    human destroys the control first, then the four tagged objects one by one, with its starting
    units. *Expected:* nothing is revealed when the control is destroyed; each tagged object's
    destruction reveals the area around its own waypoint, and only that one. *Falsified if* a
    corner is revealed before its object is destroyed, or stays hidden after it. *Inconclusive*
    for an object that the computer player moves out of the human's reach before it is
    destroyed.
  - (I) Veterancy value (claims 35 and 41). The computer player's country is set in the lobby to
    a fixed country chosen before running. Before running, choose for that country a vehicle type
    and an infantry type that are in none of that country's veteran lists (S-0311
    claim 11; check in the rules data). Vehicles of that type with veterancy values 99, 100 and
    200 in field 9, and infantry units of that type with values 99, 100 and 200 in field 10.
    *Expected:* by S-0311 (claim 11 and F5, experience *n* × 0.01, so 0.99, exactly 1 and exactly
    2), the objects with 99 show no veterancy insignia, those with 100 the veteran insignia and
    those with 200 the elite insignia (S-0111 claim 18). *Falsified if* any of the six shows
    another insignia. *Absent field:* if the rules data give the computer's country a vehicle type
    in its veteran list for vehicles, one vehicle of that type whose entry ends before field 9.
    *Expected:* the veteran insignia (the experience its type and country give it). *Falsified
    if* it shows none.
  - (J) Wrap of the starting health, and a building's replacement of a value above 256 (claim 37).
    Choose a vehicle type and a building type whose strengths *S* are above 600 (check in the rules
    data). If no stock type of a kind qualifies, use a named test file: a loose copy of the rules
    file in which one type of each kind that lacks a qualifying stock type has strength 1000 (both
    changes in the one copy when both kinds need it), removed after the run. For each
    of the two types write the health value *H* = ⌈256 × (2³² + ⌊*S* / 2⌋) / *S*⌉ of its own
    *S*, which lies below 2³¹ because *S* is above 512. *Expected:* the vehicle starts at the
    health that Formulas (object health) gives, about half strength (*H* × *S* / 256 lies just
    above 2³² + ⌊*S* / 2⌋, which wraps to about *S* / 2); the building starts at full strength,
    because its *H* is replaced by 256. *Falsified if* the vehicle starts at full strength (no
    wrap) or the building at about half strength (no replacement).
- **M20, format versions below 4 (claim 35).** M19 variant A's map saved at format version 3, with
  each object's two cell fields replaced by one cell number n = x + 128·y. *Expected:* every
  object at the same cell as in M19, with the same health and facing. *Falsified if* any object is
  displaced or its later fields are misread (wrong health or facing).
- **M21, initial state of pre-placed buildings (claims 38, 39, 40).** A 20 × 20 flat clear map
  revealed to the human player. Buildings owned by the computer placeholder name. Screenshots of
  the first visible tick and of tick 30. 1 run per variant.
  - (A) A stock building with a build-up sprite and distinct idle and active slot animations, at
    health 256, and a second copy at a health value below the damaged threshold. *Expected:* no
    build-up on either; both in the idle body state from the first tick, the idle and active
    animations already running. For a type without a sprite turret the second copy is damaged
    in body and animations. *Falsified if* a build-up plays, or that damaged copy shows
    undamaged slot animations.
  - (B) A stock building with a sprite turret, at facing values 0 and 128, with no enemy in range,
    plus a damaged copy. *Expected:* the turret animation is present from placement, facing
    as claim 39 specifies. Under S-0108 claim 11's original sprite-turret exception the
    damaged copy's running slot animations show normal variants, but its body and health
    remain damaged. *Falsified if* the turret is absent at placement or starting it restores
    health. A copy built in play is the completion-time control.
  - (C) A stock building with powered slot animations and either a positive base drain or the
    needs-power flag, once with the switched-on flag 0 and once with it absent. *Expected:*
    the first starts switched off (its powered animations frozen as for a building switched
    off in play), the second runs. *Falsified if* both behave alike. S-0405 W13 supplies the
    pure-producer and firestorm-wall controls for the startup conditions.
  - (D) A stock building that accepts upgrades, with upgrade count 1 and one valid upgrade name,
    then a second building entry with upgrade count 1 and no upgrade name field. *Expected:* both
    buildings show the upgrade (the second through `quirks.upgrade_fields_carry_over`). *Falsified
    if* the first lacks it, or the second lacks it while the first has it.
  - (E) A stock building with a voxel turret and a 2 × 2 foundation at cell (12, 12), facing
    value 64, with turret-animation displacements of 0 (the stock values or a named test file).
    The foundation cells (12, 12), (13, 12), (12, 13) and (13, 13) are legal (s = 24 … 26).
    *Expected:* its foundation occupies (12, 12) to (13, 13) (an infantry unit cannot stand on any
    of those cells), and its turret is centred on the projection of cell (12, 12)'s top corner and
    points east. *Falsified if* the foundation is shifted, or the turret is centred elsewhere or
    faces another way.
  - (F) Pre-place a stock refinery whose type grants a free unit, and a stock superweapon
    building with visible charge-slot artwork. Disable starting units and keep a distant
    bystander computer player alive. Use speed index 2, 800 × 600 pixels, stock data, no engine add-on and
    no mods (the named test files of a scenario and the purpose-made test maps excepted). With a read-only recorder, capture slot names, health and nearby unit counts
    at the end of loading and before and after each building's first turn. *Expected:* the
    refinery has both its set-up idle slot and its fill-level slot, but grants no free unit.
    The superweapon building has the charge slot selected by S-0108 claim 11, subject to
    its power-state rules. *Falsified if* the fill slot is missing at the end of loading,
    the refinery grants a free unit, or the charge slot disagrees with that selection when
    power rules do not suppress it. S-0215 claim 1's scenario B36 separately distinguishes
    current and queued *guard* at these boundaries for armed, unarmed and upgraded hosts.
- **M22, terrain objects and smudges (claim 42).** A 20 × 20 flat clear map revealed to the
  human player. 1 run per variant.
  - (A) Terrain-object entries with keys 12012 and 10014 naming a stock tree type in lower case,
    one with value none, and one at format version 3 with key 10 + 128·12 = 1546 on a separate map.
    All three cells are legal (s = 24, 24 and 22), and the key 10014 has x ≠ y, so it tests which
    part of the key is x. *Expected:* trees at (12, 12), (14, 10) and, on the second map,
    (10, 12), and nothing for none.
    *Falsified if* a tree is missing or misplaced, or the none entry places anything.
  - (B) A stock smudge type of width and height 2 at cell (12, 12) with piece number 0, and a
    second entry at (15, 12) with piece number 1. All the cells involved are legal (s = 24 … 27).
    *Expected:* the first covers (12, 12) to (13, 13); the second places nothing. *Falsified if*
    the first covers other cells or the second appears.
  - (C) The same smudge type at cell (12, 12) with piece number 0, on a map where (13, 12) and
    (13, 13) are water cells and the cells around them are clear ground. *Expected:* the smudge
    covers all four cells, the water cells included, since its placement tests no cell (claim 42).
    *Falsified if* it is missing or covers only the clear cells.
  - (D) Starting health of a terrain object (claim 42). Two maps, each with one terrain-object
    entry for a stock tree type *K* that is not animated and has two image frames, at (12, 12).
    The first map's override section gives *K* the strength 1; the second gives it the strength 5.
    *Expected:* the tree shows image frame 1 on the first map and image frame 0 on the second
    (S-0102 claim 23: frame 1 below a health of 2), at the first tick. *Falsified if* both show the
    same image frame. S-0102's scenario D27 tests the game-wide tree strength and its quirk.
- **M23, heights, shared cells and placement order (claims 38, 43).** A 20 × 20 flat clear map
  with one high bridge, revealed to the human player. 1 run per variant.
  - (A) A vehicle on a bridge cell with the bridge flag 1 and one with the flag 0. *Expected:* the
    first stands on the deck, the second under it. *Falsified if* both stand at one height.
  - (B) In this variant the map's local area is (2, 4, 16, 12), so a legal cell is usable when
    28 < s ≤ 54 and −16 < d < 16 (claim 4). Start waypoint markers 0 and 1 are at (16, 26)
    (s = 42, d = −10) and (28, 18) (s = 46, d = 10), both inside it. Two vehicle entries on the
    usable cell (20, 20), an aircraft entry on the usable cell (22, 18), and one on the legal cell
    (12, 12), which lies outside the local area (s = 24). *Expected:* both vehicles appear on the
    shared cell; the first aircraft stands landed, the second is in the air. *Falsified if* a vehicle is missing or
    the aircraft heights are the other way round. *Height-aware sub-test:* on M3's plateau map, one
    aircraft entry on a level-4 plateau cell inside the local area by the plain test but outside the
    moved top edge (claim 4), and one on a plateau cell inside the moved edge, 1 run. *Expected:*
    the first is in the air, the second stands landed. *Falsified if* the first stands landed (the
    plain test decided).
  - (C) Two vehicles of one type with equal sort keys (same screen row, the later entry to the
    right), and an infantry unit and a vehicle on cells whose sort keys are equal. *Expected:* the
    drawing over a shared pixel follows entry order within one kind and vehicles before infantry
    across kinds (S-0102 claim 6). *Falsified if* the overlap is drawn the other way round before
    any object has moved.

- **M24, follow index (claim 41).** A 30 × 30 flat clear map, format version 4, revealed to the
  human player. Every vehicle is of one stock ground vehicle type that drives (glossary,
  *locomotor*) and is not marked as a train, is owned by the human placeholder name (claim 27),
  faces map north and stands on open clear ground at least 4 cells from every other object. The
  observable is S-0212 claim 26: when a leader that is not itself a follower car is ordered to move,
  its follower cars are given destinations 1 and 2 steps back from its destination and drive there,
  and every other vehicle stays where it is. Each order sends the named vehicle 10 cells along +x,
  so the cars' destinations lie along −x from it. 1 run per variant; record which vehicles move and
  where they stop.
  - (A) Control: three entries P, Q and R in this order, with follow-index fields −1, 0 and 3
    (V = 3). Order P. *Expected:* Q drives to the cell 1 step back from P's destination; R stays,
    because 3 is not below V.
  - (B) Values move forward: four entries in this order. A gives fields 1 to 11 only, so it has no
    follow-index field; B gives the value 2, C and D the value −1. The follow values are 2, −1 and
    −1, and the placed vehicles A, B, C and D (V = 4), so A takes 2 and becomes a follower car of C,
    B and C have no leader, and D's value is unspecified. D stands at least 15 cells from the
    others, and what D does is not recorded. Order C. *Expected:* A drives to a cell 1 or 2 steps
    back from C's destination (2 when D's unspecified value happens to make D a car of C too), and
    B stays where it is.
  - (C) n counts placed vehicles: three entries in this order. The first has an owner name that
    matches no house in the game, so it is dropped (claim 27); then P with value −1 and Q with value
    0. Order P. *Expected:* Q drives to the cell 1 step back from P's destination, because 0 names P,
    the first placed vehicle, not the dropped entry.

  *Falsified if* in variant A R moves; in variant B B drives after C while A stays (each vehicle took
  its own entry's value); in variant B neither A nor B moves while variant A passed; or in variant C
  Q stays while variant A passed (n counted the dropped entry). *Inconclusive if* Q does not move in
  variant A: either the link or S-0212 claim 26 is then at fault, and variants B and C say nothing.
- **M25, cells outside the map (claim 44).** Two maps, read from a save at the first tick.
  - (A) *Smoothing at the edge.* A 20 × 20 map of a theater whose rough transition role is set
    (S-0010 claim 4): a patch of rough ground cells that runs up to the map's edge on one side and
    is surrounded by clear ground on the others. *Expected:* each rough cell on the edge gets the
    transition tile that S-0010 claim 10 gives when every neighbour outside the map holds the
    no-tile marker (so does not belong to the rough pair) and is flat, the same as for a clear
    neighbour. *Falsified if* an edge cell gets the tile it would get with its off-map neighbours
    counted as rough.
  - (B) *Cliff-back at the edge.* A 20 × 20 map whose rules data sets the cliff-back setting so
    that the rule of S-0211 claim 8 is active, with a cell of level −4 on the map's edge whose on-map neighbours are all at
    level −4, and a second such cell at level −3. *Expected:* the level −4 cell's land type is
    *rock* (an off-map neighbour at level 0 is 4 levels higher), the level −3 cell's is not.
    *Falsified if* either differs. Check the levels with S-0009 scenario M4's method first, since
    negative levels are not used by stock maps.
- **M26, starting units (claims 45 and 46), with the random-state recorder.** A 40 × 40 flat clear
  map with start waypoint markers 0 and 1 far apart and well inside the usable area, one human
  and one computer player of different countries, a small unit-count setting, the bases setting
  on, no pre-placed objects and no crates. Before running, read from the rules data the
  base-unit list, for every vehicle and infantry type its multiplayer-start flag, owner list,
  tech level and cost, and the game's tech level *t* (the base rules' multiplayer tech level,
  Import mapping; 10 when they omit it). Both players' houses have tech level *t* (Terms, house's
  tech level). Build the budget pool and both houses' start pools by claim 45 (these values stay
  in the measurement record). 5 runs, read from a save at the first tick.
  From the recorded seed, replay the load's draws up to the starting units (scenario M9c) and
  then predict, house by house, every type draw, the remaining budget, every creation draw and
  every placement search with its draws, and the final ranged draw.
  Predict balances after starting-unit buying and then apply S-0412 claim 3's original
  extra-credit rule to the computer player's balance at its fixed lobby difficulty. That
  later addition is not left-over budget and does not apply to the human player.
  *Expected:* each house has its base unit on its base cell and exactly the predicted budget
  units, of the predicted types, on the predicted cells (infantry at their cells' centre
  positions, several sharing a cell where predicted), the human's in *guard* and the computer's
  in *area guard* (the original's behaviour; our engine gives both players' houses *guard*, claim
  45), the
  predicted credits, and the stream position at the first tick equals the prediction.
  *Falsified if* a type, a cell, the number of units or the stream position differs.
  - Variant B: the bases setting off. *Expected:* no base unit, and the first budget unit of each
    house stands on its base cell, with no search draw. *Falsified if* any house has a base unit,
    a house's first budget unit stands elsewhere than on its base cell, or the stream position at
    the first tick fits a search draw for that unit.
  - Variant C: the same map with water on every cell within 5 cells (Chebyshev distance) of base
    cell 0 except the base cell itself. *Expected:* house 0's budget units stand on the cells the
    placement search predicts, water cells at radius 4 and 5 included, because placing ignores the
    land type while the map loads (claim 46). *Falsified if* house 0's units avoid the water and
    stand only at radius 6 or more, or stand on cells the search does not predict.
  - Variant D, radii beyond the first and the second pass: the same map with pre-placed objects
    owned by the neutral house (by its country name, claim 27; vehicles, or buildings where a
    foundation covers several cells with one entry) on every legal cell at Chebyshev distance 3, 4
    or 5 from base cell 0. Every candidate of radius 4, in either pass, and of radius 5's first
    pass then lies on an occupied cell. 3 runs. The pre-placed objects take map-content draws
    (S-0202's table "Draws during map loading"), so the place of the starting units' first draw
    is found as in M9c: the smallest number of raw values after the recorded seed for which the
    prediction matches what the save shows. *Expected:* house 0's first budget unit stands on the
    cell that the prediction gives, at Chebyshev distance 6 (a second-pass candidate of radius 5,
    or a first-pass candidate of radius 6 when no second-pass candidate of radius 5 is free), and
    every later unit where the prediction puts it. *Falsified if* any of house 0's budget units
    stands on a cell other than the predicted one, or at distance 5 or less. *Inconclusive if* no
    number of raw values makes the prediction match (an unexplained map-content draw).
  - Variant E, failure limit and left-over credits: the same map with the local area made 2
    widths wide and 1 height tall (Lw = 2, Lh = 1), placed so that it holds base cell 0. By the
    usable-cell test of claim 4 its usable cells then lie within Chebyshev distance 2 of base
    cell 0, while every candidate of the placement search other than the base cell lies at
    distance 3 or more, and base cell 0 holds the base unit. Base cell 1 lies outside the local
    area. So every purchase of either house fails, the remaining budget stays *B*, above
    trunc(*B* / 3), and only vehicles are bought. 3 runs. *Expected:* each house has its base unit
    and no budget unit; the stream holds exactly 20 failed purchases per house (each with its type
    draw, its creation draw and every draw of the placement search, radii 4 to 31, both passes),
    and each house's pre-bonus balance is the credits setting plus *B*. For the computer
    player's first-tick balance, additionally apply the original's extra-credit formula of
    S-0412 claim 3 to that pre-bonus balance, with its fixed lobby difficulty.
    *Falsified if* a budget unit appears, the stream position at the first tick fits 19 or 21
    failures of a house better than 20, or a house's credits differ from that prediction. *Inconclusive if* the game
    does not finish loading with such a local area.
  - Variant F, early stop, with a named test file: a loose copy of the rules file with one
    change, a base-unit list (Import mapping) that holds only types whose owner lists include the
    human's country and not the computer's (for example only the construction vehicle of the
    human's side, with the computer of a country of another side); it is removed after the runs.
    The pools are built from the test file's data, so the construction vehicle types left out of
    the list can be in them. 3 runs. *Expected:* the human (house 0) receives its base unit and
    its budget units as predicted; the computer (house 1) receives no unit at all. Its
    pre-bonus balance is the credits setting (no left-over budget); its first-tick balance
    additionally includes S-0412 claim 3's original extra credits at its fixed lobby difficulty.
    The stream position at the first tick equals the
    prediction without the final draw R(0, 65535) of claim 20. *Falsified if* the computer
    receives any starting unit, its balance differs from that prediction, or the stream
    position fits the prediction with the final draw.
  - Variant G, dropped entries take no draw (claims 27 and 36): M26's map with five vehicle
    entries that are dropped (three whose owner names no house in the game, two whose type is an
    infantry type) and one vehicle entry owned by the neutral house (by its country name) of a
    type without either visceroid flag, which is created, all at least 12 cells from both base
    cells. 3 runs. *Expected:* the stream position at the first tick equals the prediction in which
    the neutral vehicle takes one creation draw and the dropped entries none. *Falsified if* it
    equals instead the prediction in which each dropped entry takes one creation draw (five raw
    values more before the starting units). *Inconclusive if* it equals neither prediction (an
    unexplained draw, which the main runs then have to locate).
  - Variant H, the game's tech level (Terms; claim 45). Choose a vehicle type *X* that is in the
    budget pool and in the human's vehicle pool. Every sub-variant gives the map a type override
    section (S-0006) for *X* that sets its tech level as stated. Each is predicted from the
    recorded seed twice, with every pool built at a game's tech level, and so a player's house
    tech level, of *t* and of *t* + 1, and with *X* at its overridden level; 1 run each.
    (H1) *X* at *t* + 1. *Expected:* the prediction at *t* holds (it leaves *X* out) and no unit of
    type *X* appears. Control: *X* at *t*, where the prediction at *t* includes *X*. *Expected:* it
    holds. (H2) As H1, with *t* + 1 also given to the game's tech level key by one later layer at a
    time: (a) a rules override section of the map; (b) instead, a named test file, a loose copy of
    the game-mode file of the mode played with that key set to *t* + 1; (c) instead, a named test
    file, a language rules file that holds only that key, with a valid digest formed as S-0005
    (claim 21b) gives, so that S-0006 (claim 1) applies it. Each is played as the second game of
    its program run, after a game of M26's main map. *Expected:* the prediction at *t* holds in
    all three. (H3) As H1, with a named test file: a loose copy of the rules file whose game's
    tech level key is *t* + 1, in place when the program starts. *Expected:* the prediction at
    *t* + 1 holds. (H4) A named test file: a loose copy of the rules file without the game's tech
    level key, in place when the program starts. Two runs, with *X* at 10 and at 11, each
    predicted with every pool built at a game's tech level of 10, and once more with the pools
    built at the level that puts *X* on the other side (9 for the first run, 11 for the second).
    *Expected:* the predictions at 10 hold in both runs. (H5) *X* at −1. *Expected:* the
    prediction at *t*, which includes *X*, holds (signed comparison).
    *Falsified if* in H1 the prediction at *t* fails or a unit of type *X* appears, or its control
    fails; in any part of H2 the prediction at *t* + 1 holds instead of the one at *t* (a later
    layer reached the game's tech level); in H3 the prediction at *t* holds instead of the one at
    *t* + 1 (the copy does not come from the base rules); in H4 either run fits its other
    prediction (the absent-key default is not 10); or in H5 the prediction without *X* holds (an
    unsigned comparison). *Inconclusive if* the two predictions of a sub-variant coincide (choose
    another *X*).
  - Variant I, early stop for a house with empty start pools (claim 45): M26's map with a third
    start waypoint marker, number 2, far from the other two and well inside the usable area, and
    an override section for the neutral country that sets its passive field (Import mapping) to
    false. The bases setting is off. Before running, check in the rules data that no vehicle or
    infantry type that is allowed to start in multiplayer games has tech level 1 or lower and the
    neutral country in its owner list, and that *B* > 0. The early phase saw the neutral country
    passive (claim 33), so the neutral house is assigned the one free position, 2, in the late
    phase, with no draw (claims 28 and 29). There is a computer player, so the cleanup of claim 28
    does not run. 3 runs. *Expected:* both players receive their predicted units; the neutral
    house receives none, its pools being empty at its tech level 1; and the stream position at
    the first tick equals the prediction without the final draw R(0, 65535) of claim 20.
    *Falsified if* a starting unit of the neutral house appears, or the stream position fits the
    prediction with the final draw.
  - Variant J, the nearest ground occupant (claim 46): M26's map with the bases setting off and,
    on base cell 0, two objects of the neutral house (by its country name, claim 27): a vehicle,
    which stands at the cell's centre (claim 38), and an infantry unit. Every placement search of
    house 0 starts at base cell 0, so the cell's acceptance decides whether house 0's infantry
    units stand there. Before running, check from the prediction (with the base cell refused)
    that house 0 buys at least one infantry unit. 3 runs per part.
    (J1) The infantry unit's standing position number is 1, the top position (S-0208 claims 5
    and table P), whose place in the cell is (64, 64). Measured to the cell corner it is nearer
    than the vehicle (⌊*r*(8192)⌋ against ⌊*r*(32768)⌋), so it is the nearest ground occupant.
    *Expected:* every infantry unit of house 0 stands on base cell 0, at its centre position, and
    no vehicle of house 0 does. (J2, control) Standing position number 4, the bottom position
    (192, 192), farther from the corner than the vehicle. *Expected:* no starting unit of house 0
    stands on base cell 0. (J3, tie) Standing position number 0, the centre: both objects have
    the same distance, and the infantry unit came into the cell after the vehicle (claim 43), so
    it wins the tie. *Expected:* as in J1.
    *Falsified if* in J1 no infantry unit of house 0 stands on base cell 0 (the reading "nearest to
    the centre", by which the vehicle at the centre would be nearest), or in J3 none does (the tie
    going to the occupant that came first), or in J2 any does. The stream position at the first
    tick equals the prediction made with the rule of claim 46 in every part.
  - Variant K, bridge cells (claim 46): M26's map with a high bridge whose cells under the deck
    are clear ground (check in the theater data that such a bridge exists; otherwise the variant
    is not run), with start waypoint marker 0 on a bridge cell of it. The bases setting is off.
    (K1) A vehicle of the neutral house stands on the deck at base cell 0 (bridge flag 1, claim
    38). *Expected:* house 0's first budget unit stands on base cell 0, on the ground under the
    deck, with no search draw. (K2, control) The same vehicle with bridge flag 0, on the ground.
    *Expected:* house 0's first budget unit is a vehicle and stands on the cell that the
    placement search predicts, not on base cell 0. 3 runs each. *Falsified if* in K1 the first
    budget unit stands anywhere but on the ground of base cell 0, or in K2 it stands on base
    cell 0.

- **M27, map-house identity and unresolved owners (claims 47–48; Q-0419).**
  - *Setup:* S-0103 scenario PM's isolated single-player map-entry probe, not a campaign-play
    test or a skirmish. Tiny purpose-made map, separated visible stock infantry and ordinary
    buildings, neutral test lighting, speed index 2, 800×600 pixels, no engine add-on and no mods (the named test files of a scenario and the purpose-made test maps excepted).
    Named test data: PM's loose base-rules copy with three countries C0, C1 and C2 assigned
    distinct existing colour definitions, C0 at registration index 0. Keep the colour registry
    valid. Remove the test files afterwards. Each variant is one fresh program run.
  - *Procedure A, identity:* list short house names different from every country name, give
    their sections selectors for C1, mixed-case C1 and C2, and leave their house colour
    selectors absent. Also list a house named exactly like C2 whose country selector is C1.
    Give C1 a unique imported alternative name and test a house selecting that alternative.
    *Expected:* both spellings, the alternative name and the C2-named house inherit C1's
    ramp; only the C2-selected house inherits C2's. A house's own name does not choose its
    country. With an alternative name shared by C1 and C2, C1 wins by country order.
  - *Procedure B, defaults:* use absent, empty, unknown short nonempty and undefined numeric
    country selectors. Make C0's imported colour distinct from the first definition.
    *Expected:* absent and empty inherit C0, whereas unknown and numeric selectors create
    default countries and inherit the initial definition of S-0103 claim 14. Repeat with
    a rules country whose type name is that numeric text: it then resolves by name. Give a
    house an explicit valid colour selector as PM's control: it replaces either inheritance.
  - *Procedure C, length:* list a 19-byte house name followed by extra letters, supplying
    different country selectors in the full-name section and the 19-byte section. Use a
    short house with a country selector of more than 127 bytes whose first 127 bytes become
    C1 after trailing spaces are trimmed; the omitted suffix must not affect lookup.
    *Expected:* the long house uses its 19-byte section; the cut-and-trim selector selects C1.
    These are named test map data, not stock-name-length assumptions.
  - *Procedure D, nonempty list:* list one test house only. Place otherwise valid objects owned
    by that house, by an unlisted country name, by a wholly unknown name and by a case-changed
    spelling of the listed house. *Expected:* only the exact listed-house objects survive.
    There is no automatic country fallback and no grey original object for the others.
  - *Procedure E, empty list:* remove all house-list entries, also testing a list containing
    only an empty-valued line. Place objects with exact short country type names, changed-case
    names and unknown names. *Expected:* exact country-named objects now resolve to the
    automatically created houses and inherit their countries' ramps, subject to a valid own
    colour override; changed-case and unknown owners remain absent.
  - *Observable:* record which objects remain and the identified remap pixels of each retained
    infantry and building, compared with S-0103 claim 6's ramps under test lighting. Record
    imported owner/country identities if a read-only recorder is available, especially for
    the new-country controls. *Falsified if:* another country or ramp is selected, numeric
    text is an index, the length cuts differ, or an unlisted owner survives in D. If PM's direct
    map-entry harness is unavailable, propose it to the user; do not modify the executable or
    substitute skirmish loading, which intentionally discards these definitions.

## Open questions

Q-0419's house identity, country-only owners and unresolved-owner questions are answered by
claims 47–48. S-0103 claim 14 already owns inherited and overridden colour selection. A viewer's
optional diagnostic colour for unresolved entries is presentation policy, not an original colour.

- (Q-1805, claim 19.) In the original, for a game of several humans: whether two humans can
  hold the same chosen colour and, if so, which of them comes first in the house order; the
  order among several observers; the place of observers in the house order (M10 run 4 would
  bear on it, but cannot tell a failure of it from a failure of claim 22's rule; M15, if the
  original's skirmish lobby can start a game with an observer at all, which S-0414 leaves open,
  could only rule out a single observer after both computer players, again together with claim
  22's rule); and how a network game's random colours are drawn, that is
  which peer makes the draws and in which order. No
  proposed run separates these, so no rule is claimed for the original; our engine's rules for
  them are design.

- Outside this spec, with pointers only: lighting (S-0104); tile categories and what tiles mean
  for play (S-0010); triggers and tags (S-0011); and the rules sections a map may override
  (S-0006). The byte encodings are S-0012's.
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
- The legacy terrain streams (claim 10) are not specified. They matter only for very old maps.
  (The object lists below format version 4 differ only in their cell field, claim 35.)
- Pre-placed objects (claims 34 to 43): what a pre-placed building does when its foundation is
  already occupied at the set-up; what the group number of claim 41 does; what becomes of a follower
  car whose leader the cleanup of claim 28 removes (claim 41 gives our engine's interim rule); and
  how an aircraft type flagged as a spawned missile is placed
  (claim 38 covers other aircraft) are not established. Our engine places such a missile like any
  other aircraft and the importer warns.
- What the original does with a base unit whose base cell is not a legal cell (claim 46): whether
  its direct placement there succeeds, putting the unit at a position outside the map, or fails
  and starts the placement search. Our engine treats it as failed (claim 46).
- The game mode that places players its own way in the early phase instead of building list 2 (one
  stock mode uses its own placement) is not identified. It needs a per-mode check.
- Maps wider or taller than 63 cells might behave differently from smaller maps in some way
  not covered here. No such difference is known. Comparing a 63 × 63 map with a 64 × 64 map that
  is otherwise the same could look for one.
- The observer rules (claim 22) are low confidence.
- Whether the map list filters maps on their minimum and maximum player settings (claim 23).
- The initial-time and percent settings are read and kept, but no rule of play is known to use
  them in a skirmish game (Import mapping); whether a campaign mission uses them is outside this
  spec. The effect of the fill-silos and multiplayer-only switches is not established here.
- The full list of draws in the map-content row of the table "Start position resolution" belongs
  to S-0202 (its table "Draws during map loading"); what each pre-placed object's placement draws
  belongs to the placement rules, which have no spec yet, apart from what S-0202's table states.
  The starting units' draws are claim 46's. Whether further loading steps draw is not established.
  The random-state recorder (scenario M9c) can count the draws of a whole load; an unexplained
  surplus cannot yet be assigned to a step.
- Which game modes other than the standard skirmish change the starting units (claim 45) is not
  established; claims 45 and 46 describe the standard mode.
- Whether rules in play change the stand-in cell of claim 44 (for example an effect aimed at a
  cell outside the map) was not looked for. Our engine changes nothing there in any case.
- Does the variable-name section (claim 32) also name the global trigger variables, so that a
  global variable's name comes from the entry with the same number? Only names would change, so
  nothing in play seems to show it. It is a pointer for a later revision of S-0011 (whose subset
  does not cover global trigger variables).
- Whether a non-passive neutral or special house (claim 28) runs the original's computer-player
  base logic, which would act on the computer-sell and computer-rebuild flags of its buildings
  (claim 40), is not established. A probe: scenario M16 variant B's map with a damaged pre-placed
  building of the neutral house that has the sell flag set, watched for a sale.
