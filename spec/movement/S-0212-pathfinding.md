---
id: S-0212
title: Pathfinding (observable behaviour)
topic: movement
revision: 98
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-09
measurements: []
requests: [Q-0017, Q-0041, Q-0042, Q-0043, Q-0088, Q-0238, Q-0371, Q-0634, Q-1280, Q-1751, Q-1771,
Q-1870, Q-1975, Q-2896]
supersedes: []
---

# S-0212: Pathfinding (observable behaviour)

## Summary

How a ground or naval unit of the original chooses the cells it will drive or walk through on the
way to a destination: which cells count as connected at all, the reachability test made before
any search, the cost of each step, which cells a house's threat records mark as threatened, the
search and its limits, the two straightening passes applied
to a found path, how many steps are kept, when a path is computed again, what happens when no path
exists, and when the destination is replaced by a nearby cell. How a unit then moves along the kept
steps (speed, turning, collisions, blocked-step reactions) belongs to the movement specs. The spec
covers skirmish games. Network games and campaigns are out of scope for now, and the few mentions
of them describe the original only.

**Cross-spec agreements.** Claim 25's infantry teleport fallback parameters agree with S-0208 claim
19, and its failed-approach row includes close-range searches covered by S-0307 claim 17.
Claim 25's non-infantry teleport search removes the reach-region restriction when the
ordered destination lies in no reach region, agreeing with S-0207 claim 20 b and D22 run 10.
The aircraft-factory edge-entry row and release-tick selection agree with S-0417 claim 14.
Claim 25's click-search conditions agree with S-0214 claim 11: direct destinations bypass
search, and a free mover's no-move search has no reach-region restriction, including inside
the usable area. P16 distinguishes move orders from rally-point searches.
Claim 27 remains a summary of S-0209 claim 5, which owns the full vehicle entry-outcome table.
Claims 20–22 include the ship refresh, chase-cut and retry rules of S-0504 claim 9.
S-0504 claim 9 agrees with the spatial table-root distance of claim 20
and S-0204 claim 5, truncated toward zero to integer leptons, for both refresh and chase
cutting. There is no ship-specific exact-root exception. The shared
`quirks.table_root_distance` alternative applies as stated under Formulas.
Claim 25a completes the aircraft landing footprint search used by S-0501 claim 41.
S-0501 claim 41 owns building and fallback-object selection, including equal-distance
fallback objects, and the unchecked no-result landing destination. Those caller rules are
separate from the nearby search's candidate tie rule. The new table T5 row and the enter-processing tick for the waiting absorber approach (S-0605 claim 4) answer the request in the S-0212 entry of S-0605's cross-reference list, which S-0605 should then drop.
Q-0238 is answered here for the failure tick, behaviour queue, final nearby-search parameters
and absence of additional search draws. S-0215 claim 38 and F11–F12 supply the upstream
centre geometry, base-record refresh, no-base usable-area correction and radius-then-direction
draws. Table T5 includes that refresh's separate nearby search.

Claim 16a answers Q-1751 (a house's threat records and their rebuild, in which an *AI house*
counts as computer-controlled). Claim 13 answers Q-1771 (every building's exemption rectangle is
in the corridor exemption, taken away when the building leaves the map, a to-tile building
included). For Q-1870, claim 24's repeated substitution on high ground and a harvester's
departure from a docking cell are stated under Edge cases.

### Evidence scope

The detailed route rules below are a behavioural hypothesis, not a record of measured paths:
there are no M-records for this spec. In particular, a matching whole route does not establish
all of the search decisions that could have produced it. Much of the remaining unsupported
detail falls into three **unmeasured rule families**, listed below; the list is not complete, and
every claim states its own backing and reasons, so that no aggregate check is mistaken for
independent evidence:

| Family | Detail still unmeasured | Why it is retained |
|---|---|---|
| Search selection | first-recorded route retention and its margin (11), original tie differences (12a), corridor progress, free-part and closure predicates, individual sector and connection choices (13 to 13b), flattening precedence and straight-run V's (17), threat and post-tunnel transformations (18) | These jointly predict a stored path, but no proposed scenario separates each remaining predicate from the others. They are unmeasured route hypotheses, not settled observations of intermediate work. Their exact definitions keep the reference predictions falsifiable as a family. |
| Unreachable or oversized cases | comparisons involving off-map cells (2, 4), unobservable connectivity changes and same-tick update timing (3, 5), unknown-zone gate exception (6), unusual train start state (7), exact gate and remaining train cost entries (8), permanent marks (9), same-cell computation and search limits (12, 13), absent-link bridge reads and estimate legs (3, 4, 13b, 24), loaner and retreat conditions (14), duplicate/hidden-centre and hidden-ring limits (25) | Normal input cannot force the relevant state, or no discriminating fixture is known. Limits and undefined-result departures still need deterministic answers in our engine. The local backing states which parts are original hypotheses and which are design rules. |
| Remaining caller/state distinctions | lepton-scale refresh boundary and table-root distinction, ship retained-step and ship/hover/walker chase-cut isolation (20), post-substitution gate and jumpjet estimate zone (14, 24), mode-2 path-length conditions and independently existing marks (19), remaining locomotors' failure retention (23), untested no-move/object-order parameters (25) | The scenarios do not isolate these states from other reactions. They remain explicitly unmeasured; ordinary caller observations are no backing for them. |

Scenarios P20 to P25 add separate probes of reachable height, nearby-search, blocker-chain,
cost and timing rules. Their qualification conditions are checked against observable state.
A missing qualifying run is inconclusive, never a confirmation. The unmeasured families above
are not promoted by a reference implementation agreeing with itself.

## Terms

*Cell*, *lepton*, *tick*, *facing*, *height level*, *world height*, *slope*, *bridge*, *high
bridge*, *bridge deck*, *overlay*, *building*, *house*, *human-controlled*, *local area*, *legal
cell*, *movement zone*, *locomotor*, *game minute*, *map size*, *round-toward-zero mode*,
*departure*, *AI house*, *team member*, *importer warning*, *override section*, *trigger action*,
*type kind*, *quirk switch*, *step direction* (written *d*) and *follower car* (with its *leader*
and *car chain*) are used as in the glossary.

Terms from other specs, cited by claim. All of them are approved. S-0003 (value forms), S-0006
(rules layering) and S-0009 (map file semantics, which defines the *waypoint marker*) are cited by
claim, and S-0104 (lighting) for the *ion storm*, a global effect (table T5). The others give:
- From S-0201 (game speed and tick rate): the **tick number** of a tick (its Terms and claim 1).
- From S-0204 (coordinates and facing): the **table square root** *r*(*n*) of an integer
  *n* (its Formulas F1), the **8-direction** of a facing (its claim 10 and Formulas F4: the facing
  rounded to the nearest eighth of a turn, a half step rounding up, which is a step direction), the
  **direction** of one point
  from another (its claim 8 and Formulas F3), and the roundings trunc₂₄ and trunc₅₃ (truncation
  toward zero to single and to double precision).
- From S-0205 (number domains): every floating-point result of the original is formed with a 53-bit
  significand under the round-toward-zero mode (its claims 4 and 5). This spec defines no rounding
  switch of its own.
- From S-0206 (turn rates): the *body facing* of a unit.
- From S-0106 (terrain surface height): the **surface height** *H* at a point, in integer
  leptons, excluding a bridge deck (its Terms and claim 2).
- From S-0207 (ground vehicle movement) and S-0208 (infantry movement): the **type speed** *V* of
  a vehicle type (S-0207 claim 1, in progress units per tick) and the **base speed** *b* of an
  infantry type (S-0208 F4, in leptons per tick). Both are the integer 0 to 255 that the type's
  speed field gives through the same clamp and scaling, with the same rule for the value −1. This
  spec calls either integer the type's **speed number** and compares speed numbers as plain
  integers, whatever their unit (claim 19).
- From S-0211 (land types and passability): *land type*, *locomotion class*, *land multiplier*
  *M*(*ℓ*, *c*), the **passability category** of a cell (claim 11) with its eight names *open*,
  *crushable*, *wall-like*, *beach*, *water*, *partial obstacle*, *blocked* and *off-map*, the
  **zone table** (claim 12) with its three answers *yes*, *no* and *never*, the *usable area*, a
  house's *firestorm*, and the rule of its claim 10 for which cells an object of a locomotion class
  may enter with a given movement zone (here: a cell is **enterable by class**).
- From S-0208 (infantry movement): the *scatter* of an infantry unit, which uses the nearby-cell
  search of claim 25 (S-0208 claim 13), and the *screen-cover check* of its Terms, which is the
  visibility test of claim 25.
- From S-0210 (crushing): who is a *crusher* (its claim 1: the type's crusher flag or the crusher
  veterancy ability).
- From S-0209 (occupancy and crowding): the **entry outcome** a unit gets for a neighbouring cell,
  with its eight names *clear*, *cloaked blocker*, *moving blocker*, *closed friendly gate*, *friendly
  obstacle*, *enemy obstacle*, *stationary friendly* and *blocked*, in that order of severity; the
  *occupancy record* of a cell layer, and a vehicle that is *between steps* or *at a chain point*
  (its Terms); the *close-enough distance* and the *blockage
  delay*, which S-0209 imports (its Formulas F2 and
  Import mapping); the *path retry timer*, whose length this spec gives (claim 21). Which situation
  gives which outcome belongs to S-0209: claim 4 for an infantry mover, claim 5 for a vehicle
  mover. Claim 27 here summarises the rows of S-0209 claim 5 that this spec relies on; where the
  summary and S-0209 differ, S-0209 is meant. To keep the two
  *blocked*s apart, this spec always writes "outcome *blocked*" or "category *blocked*".
- From S-0215 (behaviours): *behaviour* and the behaviour names *move*, *guard*, *area guard* and
  *retreat*, to *queue* a behaviour, to *fall back to rest*; *destination*.
- From S-0503 claims 15–16: a hovering unit's **movement destination**, the locomotor's
  three-dimensional destination point, distinct from its **order-level destination**, the
  destination seen by behaviours and factory report tests. An accepted stop can clear the
  latter while retaining the former.
- From S-0213 (human orders): the *order* and the *rally point* order of its claim 20, which this
  spec cites as users of the nearby-cell search (claim 25). No rule of S-0213 is used otherwise: how
  a unit is ordered, and when an order applies, belong to S-0213 and S-0214.
- From S-0010 (theater and tileset semantics): the **destroyable-cliff role** of a theater's tiles
  (its claim 4 and table R), the *bridge* and *wooden bridge* roles of the same table (Terms,
  *bridge ends*), the burrowing test of its claim 17, which is the further test of claim 25, and
  the **bridge spans** of its claim 25 (its Terms: *bridge span*, *high span*, *low span*, each with
  a *start cell* and a *far end*), which are this spec's bridge links (claim 3).
- From S-0009 (map file semantics): the *visiting order* of the map's cells (its claim 3) and the
  order of the overlay type stream (its claim 12: increasing y, and within one y increasing x).
- From S-0402 (harvest cycle): the *harvester*, *refinery*, *queueing cell* and *free unit*
  (its Terms), used by the nearby-cell searches of its claims 9, 13 and 18 (table T5).
- From S-0214 (order targeting rules): the *free mover* of its Terms, which decides the no-move
  rows of table T5, and the *destroyable-cliff cell*, a cell whose tile is the first tile of that
  role or the tile after it (its Terms).

Terms specific to this spec:

- **Terrain object:** an object of the terrain-object type kind, such as a tree, standing in a cell.
- **Destroyable cliff:** a destroyable-cliff cell (S-0214 Terms): its terrain tile is the first
  tile of its theater's destroyable-cliff role (S-0010 claim 4) or the tile after it. Both tiles
  can collapse when hit, which changes the cell's terrain.
- **Wall section:** one cell's piece of a wall: an overlay whose overlay type has the wall flag
  (S-0211 Terms), placed by the map or built by a house in play. A wall line is a row of them.
- **Vein-hole field:** the vein overlay that a vein-hole object spreads around itself and that grows
  and shrinks during play. Stock data is not known to use it.
- **Chebyshev distance** between cells (x₁, y₁) and (x₂, y₂): max(|x₁ − x₂|, |y₁ − y₂|), in cells.
- **Bridge cell:** a cell that carries a high bridge's bridge deck: a high-bridge cell of S-0211
  claim 15.
- **Deck-entry mark:** a yes-or-no mark of a cell, kept by the cells that an intact high-bridge
  overlay marks (S-0211 claim 15, with its marking direction *d*). Of the four cells that overlay
  marks, it sets the mark on the cell one step against *d*, on its own cell and on the cell one step
  along *d*, and clears it on the cell two steps along *d*. When several overlays mark one cell, the
  one placed last, in the order of the overlay type stream (S-0009 claim 12), decides. A destroyed
  high bridge clears the mark on the cells whose high-bridge marking it clears (S-0211 claim 15 d),
  and a rebuilt one sets it again as at map load. So of a high bridge's deck, normally 4 cells wide
  across its axis, 3 rows carry the mark. In this spec only the height rule of claim 10 reads it.
- **Bridge axis** of a bridge cell: along cell x or along cell y. It is set on all four cells an
  intact high-bridge overlay marks: along cell x when the overlay's marking direction is step
  direction 0 (indices 24 and 237 of S-0211 claim 15, which mark cells along y), along cell y when
  it is step direction 6 (indices 25 and 238). When several overlays mark one cell, the one placed
  last decides, as for the deck-entry mark. At game start the band of a low span (S-0010 claim 25)
  also sets it on each of its four cells, bridge cells included: along cell x for a span running
  east, along cell y for one running south; where bands of several spans cover one cell, the span
  whose start cell comes first in the visiting order decides. The axis is read by the bridge ends
  and the anchors (F8).
- **Bridge link:** a link (claim 3) of a bridge. Its two **end cells** are a bridge span's start cell
  and far end (S-0010 claim 25), and its **line** is the row or column of cells from one to the
  other, along cell x for a span running east and along cell y for one running south. A cell lies
  *within* *k* *cells of the link* when it lies between the two end cells along the line (both end
  cells included) and at most *k* cells to the side of the line. The link's *first end* is the
  span's start cell, its *second end* the far end.
- **Search layer:** a bridge cell is two places for the search: its ground and its deck. The deck
  layer sits 4 height levels above the cell's height level. Every other cell has only the ground
  layer.
- **Region:** a maximal set of cells of one passability category, joined through the neighbour
  rule of claim 3.
- **Reach region** of a movement zone *Z*: a maximal set of cells reachable from one another for
  *Z* according to claims 2 to 5. Two cells in different reach regions of *Z* are treated as
  mutually unreachable for every unit of movement zone *Z*. Not the same thing as the glossary's
  *movement zone*, which is the imported category.
- **Block** of level *L* (*L* = 0, 1, 2): an aligned square of 2^(*L*+1) × 2^(*L*+1) cells, starting
  at cell coordinates that are multiples of the block size. Level 0 is the finest.
- **Sector** of level *L*: a maximal set of cells of one region inside one block of level *L*,
  joined through neighbouring cells whose height levels differ by at most 1 (claim 13b). Each
  sector of level *L* < 2 lies inside exactly one sector of level *L* + 1, its **parent**.
- **Connection:** the relation between two sectors of one level that the coarse search may step
  across (claim 13b). A connection is either *straight* or *diagonal* (claim 13b); entering a
  sector across a diagonal connection costs slightly more (F9).
- **Bridge ends** of a bridge cell *X*: walking from *X* along its bridge, one cell at a time, in
  each of the two directions of *X*'s bridge axis (Terms),
  the first cell that is not a bridge cell, provided its tile is one of the 16 tiles of the bridge
  role or of the wooden-bridge role of its theater (S-0010 claims 4 and 5, table R) and its land type
  (S-0211) is not rock. A direction whose first such cell fails this condition has no bridge end.
  For a bridge along x the *first* bridge end is the east one, for a bridge along y the south one.
- **Coarse end cell** of the start or of the destination: the cell from which, or to which, the
  coarse search runs (claim 13b).
- **Anchor** of a bridge cell for a sector: the cell near one of its bridge ends from which a leg of
  the coarse estimate is measured (F8).
- **Pending:** the state of a search entry that has been recorded and not yet continued from. In the
  cell search (claims 11 and 12) the entry is a (cell, search layer) pair. In the coarse search
  (claim 13) it is a route to a sector, and one sector can have more than one pending route.
- **Expansion:** continuing a search from a pending entry. The entry's candidates are considered
  (claim 7 for the cell search, claim 13 for the coarse search) and the entry stops being pending.
  An entry is *expanded* when this happens.
- **Drive locomotor, walk locomotor:** the locomotor of driving vehicles and the locomotor of
  walking infantry (glossary *locomotor*).
- **Corridor exemption:** a ground-layer cell is exempt when one of its eight neighbours holds a
  vehicle, an infantry unit, an aircraft, a terrain object or a wall section, on either layer, or
  when it lies in the exemption rectangle of a building on the map (claim 13).
- **Exemption rectangle:** a building's foundation rectangle (S-0406 claim 2) grown by one cell on
  every side: for a foundation rectangle of *w* × *h* cells, the (*w* + 2) × (*h* + 2) cells whose
  top-left cell lies one cell lower in map x and one lower in map y than the rectangle's top-left
  cell (claim 13).
- **Corridor:** at each level, the chain of sectors, from the start's sector to the destination's,
  that the coarse search of claim 13 selects.
- **Cost unit:** the unit of every path cost in this spec, in the cell search and in the coarse
  search alike. A straight step onto a cell with outcome *clear* costs about 1 cost unit (F1).
- **Search mode** *m*: 0 for a fresh path, 1 for a re-path after the next step was found blocked
  by a moving blocker, 2 for a re-path after the blockage lasted the blockage delay (claim 22).
- **Stored path:** the list of at most 24 step directions that a unit keeps and follows
  (claim 20).
- **Heading point:** the point, in leptons, that a unit's locomotor is currently driving towards:
  normally the position of its destination, and while chasing a unit the point that the movement
  specs give.
- **Tunnel:** an underground passage that a map defines in its tunnel list (Import mapping), from a
  **tunnel entrance** cell to a **tunnel exit** cell. A **tunnel step** jumps from a tunnel entrance
  to its exit (claim 9). Not the same thing as the land type *tunnel* of S-0211. Stock maps are not
  known to define tunnels.
- **Congestion mark:** a mark a cell can carry that makes steps onto it cost four times as much
  (claim 9) and keeps it out of straightened stretches (claims 17 and 18).
- **Threat block:** an aligned square of 4 × 4 cells; the threat block of cell (*x*, *y*) is
  (⌊*x*/4⌋, ⌊*y*/4⌋). It is the same square as a block of level 1.
- **Threat record:** a non-negative integer that a house keeps for each threat block, built from
  the threat amounts of objects on the map (claim 16a).
- **Threat amount** of an object: the integer it adds to threat records (claim 16a).
- **Threat value** of a cell for a house: the house's threat record for the threat block holding
  the cell (claim 16a). It is used through the threat coefficient, in the straightening passes
  (claims 17 and 18).
- **Coarse threat** *t*(*q*, *s*, *L*): the integer threat that a house's threat records give for a
  step of the coarse search from sector *q* to sector *s* of level *L* (F5).
- **Threat coefficient** *k* of a unit: its type's threat-avoidance field (Import mapping), or
  exactly 1.0 when the unit is a team member whose team type is marked to avoid threats (never the
  case for a unit of an *AI house*, which has no teams, Departure D5). The field is
  a single-precision decimal (S-0003 claim 6) held as a double.
- **Team type, script step:** the original's computer-team system: a team type describes a team, and
  a team carries out the steps of a script one at a time. The team system itself is out of scope
  (S-0215). This spec uses only the two conditions it names.
- **Nearby-cell search:** the original's search for a substitute cell near a given cell
  (claim 25).
- **Loaner:** a unit only lent to its house for a while, which leaves the map again afterwards
  (claim 14).
- **Terms defined where they are used:** *candidate precedence* (claim 7); *blocker chain* and a unit
  *heading into* a cell (claim 8); *recorded-pair margin* (claim 11); *progress* and *progress
  cell* (claim 13); *free part*, *borders*, *probe cell*, *whole* and *split* (claim 13a); *lane
  cells* (claim 13b); *run*, *V* and *final* step (claim 17); *stretch*, *diagonal-first* and
  *straight-first* shapes, *removed step*, *leg*, *turns back*, *reference distance*, *record*,
  *triggers*, *new maximum* and *closing stretch* (claim 18); *cell ahead* and *qualifies* for a
  re-path (claim 19); *deck mode*, *simplified movement zone*, *scan order*, *row cell*, *column
  cell*, a cell that *qualifies*, *height condition*, *further test*, *collected cells* and
  *visibility test* (claim 25); *close case* (claim 27).

## Behaviour

### Connectivity

1. **Connectivity uses the passability category.** Regions and reach regions are built from the
   passability category of S-0211 claim 11 and the zone table of S-0211 claim 12. Two consequences
   matter here: units never change a cell's category, so they never split regions, and buildings
   leave their cells *open* (a house's firestorm is never on in the original, so firestorm wall
   sections never count, S-0211 claim 11), so a ring of buildings does not separate the cells
   inside it from those outside (Edge cases).
   - Backing: `measurable` (scenarios P3, P4)
   - Confidence: high for the consequences as stated. The category rules carry S-0211's confidence

2. ***No* and *never*.** For pathfinding, a zone-table answer of *no* and one of *never* both keep the
   category out of the movement zone's reach regions (claim 4) and out of its corridors (claim 13).
   They differ in one way only: the *never* column is the category *off-map*, and off-map cells form
   no region at all (claims 3 and 4).
   - Backing: `measurable` for the shared effect of *no* and *never* on a destination (scenarios
     P3 and P11 run 3). `unmeasured` for the distinction between them: it shows only when an off-map
     cell is compared with another off-map cell or with a cell of no reach region, and claim 4 says
     why neither comparison can be brought about in play
   - Confidence: high

3. **Regions and their joins.** Two neighbouring cells of the same passability category, neither of
   them *off-map*, belong to the same region when their height levels differ by at most 1. Two
   neighbours in step direction 2 or 6 (along cell x) of the same category also join when their
   height levels differ by 2 or 3 (`quirks.region_join_height_asymmetry`). Off-map cells belong to
   no region. Two regions are *adjacent* when they contain neighbouring cells whose height levels
   differ by at most 1. They are also adjacent, whatever the heights, when one of the two regions
   has the category *blocked* and they contain neighbouring cells. Apart from these two exceptions
   (the join along cell x, and the adjacency of a region of category *blocked*), neighbouring cells
   whose height levels differ by 2 or more are never joined or adjacent through that pair. Two
   neighbouring cells that are both of category *blocked* still join only under the height limits
   above. A **link** (a bridge connection or a tunnel connection, while it is enabled) makes the
   regions of its two end cells adjacent.

   *Bridge links.* The bridge links are the bridge spans that S-0010 claim 25 finds at game start,
   one link per span, created in the visiting order of their start cells (S-0009 claim 3). A link
   is enabled at game start when its span is high and not enabled when it is low. So the two end cells
   of an intact high bridge's link lie in one reach region of every movement zone that can use
   both of their regions, whatever lies under the bridge. In play, a successful direct attempt
   on a high bridge names one cell: the lost section's section cell or a collapsed pier's
   reference cell, regardless of any additional losses caused by that attempt (S-0511 claim 12).
   High-bridge repair names each restored pier's reference cell, the far ramp and the cell one
   step before that ramp (S-0511 claim 19). Low-bridge destruction and repair name no cell
   (S-0511 claims 12, 15 and 20). For each named cell, the bridge links change as follows:

   | Bridge links within 3 cells (Terms) of the named cell | Result |
   |---|---|
   | at least one | each of them stops being enabled for a destruction attempt, and becomes enabled for a repair |
   | none | every bridge link is replaced by the links of the spans that S-0010 claim 25's rules for the spans and for high and low find on the map as it stands at that moment. Each new link within 3 cells of the named cell stops being enabled or becomes enabled as in the row above. When no new link lies within 3 cells, no link changes |

   S-0511 claims 12 and 19 own these named-cell rules. Claim 5 brings connectivity up to
   date after bridge changes, including low-bridge changes that name no cell.

   The relation is symmetric: a step of 2 or 3 levels along cell x joins the two cells whichever of
   them is higher. The original builds regions by labelling the map in a fixed order, and in rare
   configurations its result differs from this rule: a region can stay separate from a neighbour
   that the rule makes adjacent. The smallest known case is 2 × 2 cells: (x, y) *crushable* at level
   *h*, (x + 1, y) *open* at *h* + 2, (x, y + 1) *open* at *h*, (x + 1, y + 1) *open* at *h* + 2, with
   every other neighbour of the four in a category the movement zone cannot use. The rule makes all
   four one reach region for a movement zone that allows *open* and *crushable*. The original leaves
   (x, y) on its own. The adjacency of a region of category *blocked* across a height difference of
   2 or more is affected in the same way: the original records it in some labelling orders and not
   in others. Departure D4 (accepted by the project lead, 2026-10-10) makes our engine follow the
   rule.
   - Backing: `measurable` (scenario P4, maps F, F3, F5, I and H; map K for a high bridge's link,
     and P8 for a link that stops being enabled; S-0511 scenarios B8 and B12 for the destruction
     and repair named-cell rules). `unmeasured` for the new search of spans in play:
     stock play gives no way to make that lookup find no link
   - Confidence: high for the height limits 1 and 3. Medium that the rule matches the original's
     reach regions on stock maps for movement zones that do not allow the category *blocked*. Low
     for the zones that allow it (subterranean and fly), where the original's labelling departs from
     the rule far more often. Medium for which bridge links exist and that a high span's link is
     enabled at game start. Low for the changes in play, whose triggering cells belong to the
     bridge specs

4. **Reach regions.** For a movement zone *Z*, each region gets the zone table's answer for its
   category. Regions whose answer is *yes* and that are adjacent to one another, directly or through
   a chain of *yes* regions, form one reach region of *Z*. A region whose answer is not *yes* belongs
   to no reach region of *Z*: every cell in it counts as unreachable for *Z*. In the comparison of
   claim 6, however, any two such cells count as lying in the same reach region, and so do any two
   off-map cells. An off-map cell, a cell of no reach region and a cell of a reach region all count
   as lying in different reach regions. Each movement zone has its own reach regions, all built from
   the same regions.

   *A bridge cell read for its deck.* Where this spec reads a bridge cell's reach region *for its
   deck* (claims 6, 14 and 25 say when), the cell stands for the deck and gets the reach region of
   another cell, found through the first bridge link, in creation order (claim 3), that the bridge
   cell lies within 1 cell of (Terms):

   | Link | Reach region of the deck |
   |---|---|
   | found, enabled | that of the link's first end |
   | found, not enabled | going from the bridge cell one cell at a time east along cell x when the link's line runs along cell x, or south along cell y when it runs along cell y, take the first cell that is not a bridge cell: that of the link's second end when that cell's tile is one of the 16 tiles of the bridge or wooden-bridge role of its theater and its land type is not rock, otherwise that of the link's first end |
   | none found | a reach region of its own: such a deck counts as lying in the same reach region as any other deck read with no link found, and in a different one from every other cell, off-map cells and cells of no reach region included |

   Every other read of a bridge cell, and every read of another cell, takes the cell's own reach
   region as above, which for a bridge cell is that of its ground.
   - Backing: `measurable` for reach regions and their differences between cells inside the usable
     area (scenarios P3, P4), for a destination that is off-map or in no reach region while the
     unit stands in a reach region (P11 run 3), and for a deck read through an enabled link (P4 map
     K run 2). `measurable` for two cells of no reach region counting as the same region in target
     acquisition (S-0309 claims 7 and 11, scenario A11 part j, runs 3 and 5): map-placed infantry
     on cliff cells allow this comparison without a move order placing them there.
     `unmeasured` for comparisons involving off-map cells. Two off-map cells are compared only
     when the unit's own cell is outside the usable area, and then the second exception of claim 6
     skips the comparison. This also prevents the gate from comparing an off-map starting cell
     with a destination of no reach region. `unmeasured` for a deck read through a
     link that is not enabled or with no link found: the first needs a bridge cell within 1 cell of
     a link that is not enabled, such as a low span laid beside a high bridge, or a high bridge whose
     link a destruction has disabled while some of its deck stands, which belongs to the bridge
     specs; the second needs a deck cell more than 1 cell to the side of every link's line, which
     depends on where a map's end tiles (S-0010 table B) and overlays lie, and no scenario of this
     spec places one at will
   - Confidence: medium (the two shared comparisons: low; the deck read: medium for an enabled link,
     low for the other two rows)

5. **When connectivity is recomputed.** After each of the following changes of the map content, a
   path computation sees the reach regions and the corridors that the new map content gives, even
   one made later in the same tick as the change. Entries that change no category leave
   connectivity as it was:
   - a wall section built, damaged, destroyed or sold,
   - a bridge destroyed or repaired, whether by damage, by an engineer (an infantry unit that enters a bridge repair hut to rebuild the bridge) or by a trigger action,
   - a destroyable cliff collapsing,
   - a vein-hole field growing or shrinking,
   - a terrain object destroyed,
   - a building destroyed,
   - a trigger action that changes the map's local area, which moves cells into and out of the
     category *off-map*.

   No other event of play is known to change a cell's category. A destroyed building changes none:
   a house's firestorm never becomes active in the original (S-0211 claim 11), so with S-0211's
   defaults no building, firestorm wall sections included, ever decides a category. With
   `quirks.passability_laser_fence_lost` off (S-0211), a laser-fence section placed, becoming active
   or becoming inactive changes a category, and connectivity is then brought up to date after that
   change too.
   - Backing: `measurable` for a wall section built (scenario P15 run (wb)), for walls, terrain
     objects and destroyable cliffs destroyed and bridges repaired (P15), for a change of the local
     area (P15 run (r)) and for bridges destroyed (P8), each through "a later path computation sees
     the change". `unmeasured` for the other entries: a damaged wall stays a wall, a destroyed
     building changes no category, stock data has no vein holes, and whether a player can sell a
     wall section at all belongs to the order specs (S-0213), so none of them is known to change a
     path in stock play. `unmeasured` for "later in the same tick": the proposed runs do not
     qualify a destruction followed by a path computation on that same tick. Scripted input can
     time an order, but does not itself establish that the resulting computation follows the
     destruction rather than precedes it. No qualifying fixture is supplied here, so the
     same-tick guarantee is not independently backed by the later-order observations
   - Confidence: medium for the list (high for walls built and destroyed, bridges and the local
     area). Low for whether the result always equals a full recomputation for the new map content
     (Open questions)

### The reachability gate

6. **No search across reach regions.** Every path computation compares the reach region
   (claim 4) of the unit's cell with that of the destination cell as ordered, for the unit's
   movement zone. The unit's cell is read for its deck (claim 4) when the unit is on a bridge deck,
   and the destination cell when it is a bridge cell (for an aircraft as well). When they differ, no search is made: the unit gets no path, and
   the gate-failure row of claim 23 applies. Only a destination that passes the comparison can be
   replaced by a nearby cell (claim 24); one that fails it never is. The comparison is
   skipped (the destination counts as reachable) in three cases, the *exceptions* of this claim:
   first, the unit's type has no movement zone (Import mapping, departure D1); second, the unit's
   cell is outside the usable area but a legal cell; third, the unit is in the state that disables
   the corridor (claim 14), its cell is inside the usable area, and the destination is a legal cell
   outside it.
   - Backing: `measurable` (scenarios P3, P7 run 7, P11 run 1 for the second exception, P11 run 2
     for the third). `unmeasured` for the first exception, which only data outside the vocabulary
     reaches, and which departure D1 replaces in our engine
   - Confidence: high for the gate itself and its place before substitution. Medium for the
     exceptions

### The search

7. **Search shape.** The path is found by a best-first search over (cell, search layer) pairs,
   from the unit's cell to the destination cell. **Candidate eligibility and route selection.**
   Eligible adjacent-cell candidates have step directions 0 through 7 and an entry outcome other
   than *blocked*. With a corridor, a ground-layer candidate is eligible only when its level-0
   sector is on the corridor or it has the corridor exemption (claim 13); bridge-deck candidates
   are not restricted by the corridor. At a tunnel entrance the tunnel step is also a candidate,
   subject to the same outcome and corridor eligibility conditions.
   Among candidates associated with one expanded pair, route-selection precedence is direction
   order 0 through 7, followed by the tunnel step. This **candidate precedence** determines which
   route reaches a pair first (claim 11), equal-score route choices (claim 12a) and the progress
   cell (claim 13), without prescribing how candidates are stored or enumerated.
   For each considered candidate the search records a cost *g* (F1) and an estimate *f* = *g* + *h*
   (F2). The pair with the smallest *f* is expanded next (ordering and ties: claim 12). The search
   ends with a path when the destination pair is about to be expanded. The start pair is the unit's
   cell on the search layer the unit is on. The destination pair is the destination cell on its
   ground layer, except that when the destination is a bridge cell the destination is its deck (for
   every unit that is not an aircraft). A candidate step onto a bridge cell is on that cell's ground
   layer when the height level of the pair being expanded is within 1 of the cell's height level,
   and on its deck otherwise.

   Each pair has a height level: its cell's height level, plus 4 on a deck. The start pair is on the
   deck when the unit is on a bridge deck and is not an aircraft. For a type marked as a train
   standing on a bridge cell, let *H*₀ be the start pair's height level by these rules and *z* the
   train's world height in leptons. The start pair's height level is then

   > *H* = *H*₀ + 4 when |trunc(*z* / 104) − *H*₀| > 2, and *H* = *H*₀ otherwise

   where 104 leptons is the world height of one height level (glossary), trunc converts the
   quotient toward zero to an integer, and every other operation is exact integer arithmetic. So
   a train at deck height that is not marked as on the deck still starts on the deck. The entry outcome of a candidate (S-0209) is asked for a step from
   the cell of the pair being expanded, in the candidate's step direction, with the height level of
   that pair as the mover's reference height level. The height rule of claim 10 is part of that
   outcome.

   The estimate *h* is the table square root (S-0204) of the squared straight-line distance in
   cells (F2), while a straight and a diagonal step cost about the same (F1). *h* therefore
   can overestimate remaining route cost: a clear diagonal step covers √2 cells of straight-line
   distance for about 1 cost unit. The search is therefore not guaranteed to find the cheapest
   path. On the open-ground routes described below it favours closing straight-line distance
   with diagonal steps before straight ones. Two consequences:
   - On open ground searched without a corridor (claim 14), the path found takes all its diagonal
     steps first and then all its straight steps, and its number of steps is the Chebyshev
     distance from start to destination. This holds for destinations up to at least 14 cells
     away along each axis. In those searches no two pending pairs have exactly the same *f* when
     one is expanded, so the tie order of claim 12a plays no part. Neither straightening pass
     changes such a path: it has no right-angle V of diagonal runs (claim 17), and it is already
     a shortest path, so claim 18 finds no stretch to shorten.
   - Around obstacles the search can return a path with more steps than the shortest one. The
     straightening passes (claims 17 and 18) can remove some of the extra steps before the path
     is stored.
   - Backing: `measurable` for the open-ground shape (P17), ground/deck step choices
     (P4 map K2), and eligible blocked outcomes through route choice (P13). `unmeasured` for the
     train start-height adjustment: no scenario places a train at deck height without its deck
     state, so its adjustment cannot be isolated. Candidate precedence and the destination's
     selection at minimum *f* have aggregate backing only (P1's secondary check); their
     individual effects are `unmeasured`, because the driven path also depends on corridor
     choice and straightening. `unmeasured` for the extra
     steps around obstacles on their own: what the unit drives is the path after claims 17 and 18,
     which can remove them, so the search's own extra steps show only together with the
     straightening passes (scenario P9, train run 4, and the secondary check of P1)
   - Confidence: high for the cost and estimate formulas. Medium for the open-ground shape, which
     follows from F1, F2 and T4. Medium for the obstacle consequence

8. **Step cost by entry outcome.** The base cost *b* of a step depends on the entry outcome for its
   target cell (table T2): *clear* 1, *cloaked blocker* 1000, *moving blocker* per the next
   paragraph, *closed friendly gate* 1, *friendly obstacle* 60, *enemy obstacle* 20, *stationary
   friendly* 8. Steps with outcome *blocked* are not considered. For a type marked as a train, every
   outcome other than *blocked* costs 1.

   For *moving blocker*: in search mode 2, *b* = 1000. In search mode 1, *b* = 4. In search mode 0,
   *b* depends on the **blocker chain** of the target cell: a sequence of at most 10 cells
   *c*₁, *c*₂, …, with *c*₁ the target cell. Each cell of the chain is represented by one object, the
   first object the cell lists on the step's search layer, in the listing order of S-0209 claim 13. Here
   a *unit* is a vehicle, an infantry unit or an aircraft. Whether the unit counts as moving in the
   sense of claim 27 plays no part. When the object of *c*ᵢ is a unit heading into a further cell,
   and *i* < 10, that further cell is *c*ᵢ₊₁. Otherwise *c*ᵢ is the chain's last cell, and *b* follows
   from it:

   | Last cell *c*ᵢ of the chain | *b* |
   |---|---|
   | holds no object | 1 |
   | its object is not a unit (for example a building or a terrain object) | 4 |
   | its object is a unit with zero speed and an empty stored path (any *i*, 10 included) | 1, as if the cell were empty |
   | *i* = 10, and its object is a unit heading into a further cell | 4 |

   A unit is heading into the cell one step from its own cell in the 8-direction of its current body
   facing if its current speed is not zero, and otherwise into the cell of the first step of its
   stored path. For a hovering unit, this current-speed test means its double-precision
   speed fraction ρ is nonzero, not that its hover speed or displacement is positive
   (S-0503 claim 14). Even while turning with zero hover speed, nonzero ρ sends the
   chain along the body facing, not the movement heading. With ρ = 0, the first stored
   step supplies the direction, or none when the path is empty.
   A unit with zero speed and an empty stored path is heading nowhere. So a parked
   unit, standing still with no stored path, ends the chain as an empty cell would (*b* = 1), while a
   unit that stands still but keeps a stored path passes the chain on to its next cell. For a
   further cell of the chain, the object is taken from the cell's deck when it is a bridge cell and
   the unit heading into it either is on a bridge deck or stands in a cell whose height level is at
   least 3 above that further cell's height level, and from its ground otherwise.

   The land multipliers play no part in the cost: a step onto slow terrain costs the same as a step
   onto a road. A land type whose multiplier for the unit's locomotion class is exactly 0 is
   impassable for that unit, but that comes from the entry outcome (*blocked*), not from the cost.
   - Backing: `measurable` (scenario P13 for the values of T2, for the chain ending on an empty cell
     and for the chain ending on a parked unit, P2 for the absence of terrain costs, P1 for the
     whole). P23 tests the ten-cell cap, non-unit chain termination and bridge-layer
     selection using scripted input and qualifying snapshots. S-0503 scenario H11 tests
     the hover's ρ-based heading choice separately from actual displacement.
     P24 tests the closed-friendly-gate
     low base cost and trains' uniform treatment of stationary-friendly and enemy-obstacle
     outcomes. Exact gate cost and the other train outcome costs remain `unmeasured` individually:
     P24 separates the stated low-cost choices from the ordinary larger penalties but does not
     determine every table entry or distinguish 1 from every nearby cost. The search-mode costs
     are tested in aggregate by P6 and P14, not by independent cost-threshold tests.
   - Confidence: high for the table. Medium for the blocker chain

9. **Congestion marks and tunnel steps.** A step onto a cell that carries the congestion mark costs four
   times its base cost (F1). No rule of the original that sets the mark on a cell for longer than
   one search has been found, so the marks that occur in play are the temporary ones of claim 19,
   and in particular a tunnel's entrance carries no mark. A tunnel step costs the Chebyshev distance
   between the entrance and the exit cell, with no direction term and no factor. Stock data is not
   known to contain tunnels.
   - Backing: `measurable` with a test map containing a tunnel (P9) for the tunnel step
     and unmarked entrance, and the fourfold cost through claim 19 (P14). `unmeasured`
     for the absence of permanent marks: these runs test their own searches, not every possible
     event that could set a mark
   - Confidence: medium for the costs. Low that no rule sets the mark permanently

10. **Height rule for a step.** A step from a pair at cell *F* (claim 7) to a neighbouring cell *T*
    compares two height levels: *T*'s height level, and *F*'s height level, which for a bridge cell
    *F* is the level of its ground even when the pair is its deck. With Δ = level(*T*) − level(*F*):

    | Δ | The step is allowed when | Layer of *T* entered (claim 7) |
    |---|---|---|
    | 0 | the pair is not a deck; or *F* is a bridge cell, the pair is its deck, and *T* is a bridge cell with the deck-entry mark (Terms) | *T*'s ground from a ground pair; *T*'s deck from a deck pair |
    | +1 or −1 | the lower of the two cells is a slope (a cell whose slope shape index, S-0211 Terms, is not 0), whichever of the two the step starts from, from a ground pair and from a deck pair alike, with no deck-entry mark needed | as claim 7 gives: from a deck pair, *T*'s deck when *T* is a bridge cell (a deck laid over sloping ground), *T*'s ground otherwise (`quirks.deck_side_step_ground_level`) |
    | +4 | *F* is a bridge cell and the pair is its deck: the step off a deck onto ground at deck level | *T*'s ground |
    | −4 | *T* is a bridge cell with the deck-entry mark, from either layer of *F*: the step onto a deck from ground at deck level | *T*'s deck |
    | 2, 3, 5 or more (either sign) | never | — |

    A step that this rule refuses has outcome *blocked* whatever occupies *T*. Only the two height
    levels, whether the lower cell is a slope, which layer the pair is on and the deck-entry mark
    count: the slope's shape and which of its sides is raised play no part, and neither do the
    tiles at a bridge's ends. In particular:
    - A step of 4 levels off a deck needs nothing of *T*: any cell at the deck's level next to a
      deck pair can be entered from it, sideways as well as at the bridge's ends. A step of 4 levels
      onto a deck needs the deck-entry mark. So on level ground the deck of the row without the
      mark (Terms) is never entered by a step, neither from the rest of the deck nor from ground at
      deck level; only the ground under it is.
    - From a deck pair a step to a cell at the level of the ground under the deck is refused unless
      that cell is a bridge cell with the deck-entry mark. So a unit on a deck never steps sideways
      onto the ground 4 levels below it, and a deck-to-deck step between cells of equal ground level
      needs the mark. A deck-to-deck step between cells whose ground levels differ by 1 needs no
      mark, only the slope condition.
    - From a ground pair of a bridge cell (under the deck), the steps are those between ground
      cells: Δ = 0 is always allowed, so a unit drives under a bridge as over open ground.

    Away from bridges every step of a route therefore changes the height level by at most 1, and
    each step up by one level needs a slope as its lower cell. So a route that climbs from level *h*
    to level *h* + *n* passes through a slope cell at each of the levels *h*, *h* + 1, …,
    *h* + *n* − 1, and a single slope cell never leads onto ground 2 levels higher.

    Because a deck pair is judged by its cell's ground level, a unit on a deck may step sideways
    onto a cell that is not a bridge cell, 1 level above or below the ground under the deck, when
    the slope condition holds for the ground levels: it lands on that cell's ground and so drops 3
    levels (Δ = +1) or 5 levels (Δ = −1) in one step (`quirks.deck_side_step_ground_level`, Edge
    cases).
    - Backing: `measurable` (P4 maps A, C, C2, C3, I, F3 and K; K2 for level-ground deck rows).
      P20 separately tests deck-side steps with ground-level difference ±1 and deck-to-deck
      steps over sloping ground, including the marked/unmarked distinction
    - Confidence: high for the table and for the cells that carry the deck-entry mark. Medium that
      no other condition of the original enters a step's height test in the search

11. **Recorded route for a cell and layer.** During one search, each (cell, search layer) pair
    has the cost and predecessor of the first route to reach it, with precedence given by claims
    7 and 12. A later route does not replace them, even if cheaper (`quirks.path_no_reopen`, on
    by default). Once expanded, the pair is not eligible for another expansion in that search.
    Consequently, the resulting path need not be the cheapest path.
    **Recorded-pair margin.** One further rule applies to an already recorded pair, and it shows
    only in one corner of claim 15: when the destination pair has already been recorded and a later
    neighbour finds its outcome *blocked*, the rule of claim 15 applies only if the destination's
    recorded cost is at least that neighbour's cost plus 1.009 (F1).
    - Backing: `unmeasured`. The recorded costs are internal. The rule changes path shape only where
      a cheaper second route to an already recorded cell exists, and the estimate's own bias
      (claim 7) changes path shape in the same places, and no practical map separating the two
      has been established here. The
      secondary check of scenario P1 compares whole paths with claims 7 to 18 taken together, this
      claim included, but it is not a test of this claim alone.
    - Confidence: high

12. **Expansion order, limits and failure.**
    - Route-selection result: among pending pairs with the smallest *f*, a tie is resolved in favour of the pair recorded first (claim 12a, departure D7). This fixes the route consequences of a tie; it does not require an implementation to expand or store entries in any particular internal order.
    - Expansion limit: a search expands at most 65527 pairs. A search that reaches the limit fails.
    - A search that reaches the destination after exactly 10000 expansions also fails
      (`quirks.path_search_10000_failure`). The count includes the end of claim 15: a search that
      stops beside a blocked destination after exactly 10000 expansions fails as well.
    - Recorded-pair limit: a search records at most 65535 pairs besides the start pair, and one that
      would record more fails (departure D2).
    - A found path must contain at least one step. When the destination cell is the unit's own cell
      on the same search layer, no path results.
    - Backing: `measurable` for the ordering key through route choice (P13). The one-step minimum
      shows through P7 run 5's failed search beside a blocked destination. `unmeasured` for the
      same-cell, same-layer failure: no scenario forces a path computation rather than an
      arrival response to an order onto the unit's own cell. `unmeasured` for the 10000 rule and
      limits: their counts are internal and no scenario here isolates a threshold from other
      search failures
    - Confidence: high for the ordering key, the limits and the 10000 rule

    12a. **Order among equal entries.** Entries of different cost keep the ordering of claims 12
    and 13. Among pending entries of exactly the same cost, our engine takes the one recorded first:
    - *Cell search.* The compared cost is *f*, using exact comparison of the stored
      single-precision values (F1, F2). At equal *f*, the route associated with the entry recorded
      earliest wins. The start entry precedes every other entry. Entries reached from an earlier
      expansion precede those first reached from a later expansion; among candidates associated
      with one expanded entry, claim 7's candidate precedence applies. This specifies which route
      is selected, not how entries are stored or enumerated.
    - *Coarse search.* The compared cost is the stored single-precision route cost, compared
      exactly (F9). At equal cost the earliest-recorded route wins. The start-sector route
      precedes every other route. Routes first reached from an earlier expansion precede those
      first reached from a later expansion. Among routes associated with one expansion,
      precedence is by the entered sector's **first cell**: its least cell in increasing y,
      then increasing x order. Two sectors of one level never share a cell, so this preference is
      strict. A cheaper replacement route (claim 13) has its own precedence at the expansion
      that reaches it, not the precedence of the route it undercuts.

    This order is a rule of our engine, not the original's (departure D7). The original breaks the
    same ties by an order of its own, which this spec does not reproduce. That order depends on
    how the original keeps its pending entries, and in the coarse search also on the order in
    which its labelling of the map found each sector's connections. So at exact ties our engine's route can differ from the original's. In the cell search such ties
    are rare (T4). In the coarse search they are common on open ground, where many corridors cost
    the same (claim 13).
    - Backing: `unmeasured` for the original's differing tie order. The order specified for our
      engine is a design rule, checked by a reference implementation, not an oracle hypothesis. That the original's order differs from it shows only through
      whole routes at exact ties, and which pending entries tie depends on the whole search, so no
      practical map isolates it. The secondary checks of scenarios P1 and P12 report the runs whose
      reference computation meets an exact tie
    - Confidence: high that the original's order differs from this one in some ties. For example,
      when the smallest *f* among the pending pairs is shared by exactly two candidates that the
      expansion just made has recorded, the original continues from the later-recorded of the two
      first, while this order continues from the earlier one

13. **The corridor (coarse search).** When the corridor is used (claim 14), the cell search is
    confined to a chain of sectors found first:
    - Each level has its own corridor, a chain of sectors from the sector holding the start's coarse
      end cell to the sector holding the destination's coarse end cell (claim 13b: normally the
      start cell and the destination cell themselves). A sector can be on a level's corridor only when the
      zone table answers *yes* for the unit's movement zone and the sector's category, and, below
      level 2, only when its parent is on the corridor of the level above or its category is
      *crushable*. So, apart from *crushable* sectors, the level-2 corridor bounds the level-1
      corridor, and that bounds the level-0 corridor.
    - The search at each level is a cheapest-first search without an estimate. The cost of a route
      to a sector is the cost of the route to the sector before it plus the entry cost of the new
      sector: the per-category constant of table T3 for the new sector's category, plus the threat
      term *T* (F5) when the unit's threat coefficient *k* exceeds 0.00001 (F7), plus 0.001 when
      the route enters the new sector across a diagonal connection (claim 13b). The threat term is always 0 at level 0, so threat steers the
      corridor only through levels 1 and 2. Number domain and rounding: F9. Unlike the cell search,
      the coarse search lowers a sector's recorded cost whenever it finds a strictly cheaper route,
      so it ends with each sector at the cost of the cheapest route found to it. Each route found
      to a sector has this effect:

      | Sector | Route found | Result |
      |---|---|---|
      | not yet reached | any, fewer than 9999 routes pending | the route's cost becomes the sector's recorded cost, and the route is pending |
      | reached | strictly cheaper than the recorded cost, fewer than 9999 routes pending | the route's cost becomes the recorded cost and the route is pending. The route it undercut stays pending |
      | reached | not strictly cheaper | no effect |
      | not yet reached | any, 9999 routes pending | the route's cost becomes the sector's recorded cost, but the route is never pending and never continued from |
      | reached | strictly cheaper than the recorded cost, 9999 routes pending | the route's cost becomes the recorded cost, so a later route to that sector must be cheaper still to be taken, but the route is never pending and never continued from |

      A route recorded at the limit follows `quirks.coarse_limit_records_cost`. The pending routes, undercut ones included, count towards the limit of 9999 pending routes at
      any moment, so one sector can account for more than one of them. Each pending route is
      continued from once, at its own cost, when it comes up, and an undercut route is no
      exception. While the limit has not been reached, continuing from an undercut route changes
      no sector's cost, since the cheaper route to the same sector came up first. Connections
      closed by the retry below are not used.
    - **End of a level's search.** A level's search ends when the pending route taken next (the
      cheapest, ties by claim 12a) is a route to the destination's sector: that route's chain of
      sectors, from the start's sector to the destination's, is the level's corridor, and that route
      is not continued from. A route to the destination's sector that is recorded but not yet taken
      does not end the search, so a cheaper route to that sector found before it is taken still wins.
      When several routes to the destination's sector are pending, the first one taken decides. A
      route recorded while 9999 routes are pending is never taken, so it never ends the search. The
      level's search fails when no route is pending.
    - The movement zone used here is the unit's own, except that an infantry type that is also a
      jumpjet type (Import mapping) always uses movement zone *infantry* for the coarse search of
      a path computation, while the reach regions of claims 6 and 14 and the free part of claim 13a
      still use its own movement zone. The estimate *E* of claim 24 is not a path computation: it
      runs the coarse search for the unit's own movement zone, a jumpjet infantry type included.
      It adds the threat term exactly as the coarse search of a path computation does (claim 24).
    - When start and destination lie in the same sector at a level, that level's corridor is that
      one sector.
    - The corridor of each level is only the chain of sectors on the one route the coarse search
      chose, not every sector it explored. At level 0 that is a chain of blocks of 2 × 2 cells.
      Ground-layer cells whose level-0 sector is off the corridor are not considered, except cells
      with the **corridor exemption**: a cell one of whose eight neighbours holds a vehicle, an
      infantry unit, an aircraft, a terrain object or a wall section (placed by the map or built in
      play), and a cell in the exemption rectangle (Terms) of a building on the map. The exemption belongs to the cell, not to a layer: an object on a neighbour's bridge deck
      gives it as one on that neighbour's ground does. A unit counts in the cell it was last
      recorded in, from the moment it enters play. A vehicle's or infantry unit's recorded cell
      follows it as it moves (Open questions). An aircraft gives the exemption around the cell
      where it is placed on the map and, after each landing, around the cell it landed in; the
      exemption does not follow it in flight. When it lands, the exemption moves from around its
      recorded cell to around the landing cell, except at the first landing of an aircraft placed
      in the air: that one has no recorded cell, so the exemption around its placement cell stays
      for as long as the map lasts. At that first landing the exemption is still added around the
      landing cell, as at every landing; only its removal around a recorded cell is missing.
      When an aircraft leaves the map, the exemption is taken away around its recorded cell
      (`quirks.aircraft_threat_stays_put`, claim 16a). One placed in the air that leaves the map
      before its first landing has never had a recorded cell: the exemption around its placement
      cell stays, and the removal is made around the cell whose two coordinates are both 0, as
      claim 16a's threat removal is. None of that cell's neighbours is a legal cell on a map at
      least 2 cells wide (S-0009 claim 1), so that removal changes no cell a path can use.
      Every building gives the exemption to every cell of its exemption rectangle, whatever its
      owner, gates included: a building on a foundation rectangle of *w* × *h* cells covers
      (*w* + 2) × (*h* + 2) cells. The rectangle is used, not the list of foundation cells, so the
      8-cell 3 × 3 foundation (S-0406 claim 2) covers the full 5 × 5 cells; the zero-size foundation
      covers 2 × 2 cells: the building's own cell and its neighbours one lower in map x, one lower
      in map y and one lower in both. A building gives it from the moment it is put on the map,
      whether the map places it at game start or a house places it in play (so throughout its
      build-up), until it leaves the map; a capture or any other change of owner leaves it as it is.
      When a building leaves the map the exemption is taken away from its rectangle, also for a
      building whose type has a to-tile field (S-0406 Import mapping). The original leaves such a
      building's rectangle exempt for as long as the map lasts; that is a bug, fixed here with no
      switch, as the project lead decided on 2026-10-24. A wall section gives
      the exemption only to its eight neighbours, never to its own cell, both when the map places
      it and when it is built from a wall type: such a type becomes its wall overlay without any
      building being placed (S-0406 claim 10), so it has no exemption rectangle. Exemptions from
      several sources add up: a cell stays exempt until every source that gave it the exemption
      has taken it away. The coarse searches see building cells as *open* (claim 1), while a wall
      section's cell is *wall-like* and a terrain object's cell *wall-like* or *partial obstacle*
      (S-0211 claim 11), which the zone table closes to movement zone *normal* and, apart from
      *partial obstacle*, to *infantry*. So a corridor often runs straight through a building or a
      row of buildings, and seldom through walls or trees; the buildings' exemption rectangles let
      the cell search inside such a corridor follow the cells round the buildings' edge, off the
      corridor, and rejoin it on the far side (Edge cases and quirks: a unit routed through its own
      base). Bridge-deck cells
      are never excluded this way. Consequently every ground-layer cell of a path *found* with a corridor lies
      in that chain or has the exemption (straightening, claims 17 and 18, may later move cells off
      it). In particular the first step of a path is normally not held to the corridor, because the
      moving unit itself gives every neighbour of the cell it is recorded in the exemption.
      On open ground, where most sectors cost the same, the route is decided largely by the extra
      cost of diagonal connections (claim 13b) and by the order in which the coarse search takes
      routes of equal cost (claim 12a).
    - **Progress.** Number the sectors of the level-0 corridor *s*₀ (the sector of the start's coarse
      end cell) to *s*ₙ (that of the destination's coarse end cell). Let *p* be the greatest index for which there is a sequence of recorded cells
      *e*₀, …, *e*ₚ such that *e*₀ is the start cell and, for each *i* from 1 through *p*, *e*ᵢ is
      the first candidate in sector *s*ᵢ recorded after *e*ᵢ₋₁ (claim 11). A candidate refused as
      outcome *blocked*, or one whose pair was already recorded, cannot be an *e*ᵢ. The
      **progress** is *p* and the **progress cell** is *e*ₚ: the first recorded cell in the furthest
      corridor sector reached by such a sequence.
    - **Retry.** A failed cell search inside a corridor has, at each level *L*, the effect given by
      this table, where *S* is the level-*L* sector that holds the progress cell and *whole* and
      *split* are defined in claim 13a:

      | *S* | Level-*L* corridor | Effect |
      |---|---|---|
      | whole | any | the connections from *S* to every sector that the free part of *S* does not border are closed |
      | split | at least 2 sectors, *S* on it | the connection from *S* to the next corridor sector (from the previous one to *S* when *S* is the last) is closed, and so is the connection from the earlier of those two sectors to every sector adjacent to both |
      | split | fewer than 2 sectors, or *S* not on it | the corridor is abandoned |

      Closed connections stay closed for the rest of the path computation. The searches of one path
      computation with a corridor follow this table:

      | State of the path computation | Event | Result |
      |---|---|---|
      | no coarse search made yet | the coarse search fails at any level | one cell search without a corridor, whose result is final |
      | no coarse search made yet | the coarse search succeeds at every level | a cell search inside the corridor |
      | cell search inside a corridor | it finds a path | that path is the result |
      | cell search inside a corridor | it fails, and it was the 5th cell search of the path computation | the path computation fails; no further search is made, whatever the retry table would give |
      | cell search inside a corridor | it fails, and fewer than 5 cell searches have been made | the retry table above applies at each level |
      | retry made, corridor abandoned at some level | — | one cell search without a corridor, whose result is final |
      | retry made, no level abandoned | — | a new coarse search, which does not use closed connections |
      | new coarse search | it fails at any level | with `quirks.corridor_retry_no_fallback` on (the default, original): the path computation fails. With it off: one more cell search without a corridor, whose result is final |
      | new coarse search | it succeeds at every level | a cell search inside the new corridor |

      So a path computation makes at most 5 cell searches, and the search without a corridor that
      follows an abandoned corridor counts towards the 5. The extra search of the table's last-but-one
      row (the switch off) counts too: a retry is made only after fewer than 5 cell searches, so it
      is at most the 5th. Like every cell search it ends as failed at the expansion limit of the
      search (`quirks.path_search_10000_failure`), and it is deterministic (no draw). When the first coarse search fails, the
      single search without a corridor is the only cell search made, so the limit plays no part.
    - Backing: `measurable` through path shape and failure (scenarios P1, P10, P12; failure after
      retries only through P12 run T, since the building maps of P12 make no retry). `unmeasured`
      for the internal ordering of the coarse search, its end rule and for the limit of 9999 pending routes, which
      needs thousands of sector routes pending at once. No proposed fixture reaches that
      threshold with a route result separated from the other search failures. That a unit goes
      round a row of buildings that open ground surrounds, instead of giving up, is `measurable`
      (P31, P12 and P12 run S). That it does so because of the exemption rectangles of buildings,
      on the first cell search, is `unmeasured`: without the rectangles the first cell search
      inside the corridor would fail, and a retry within the limit of 5 could then find a corridor
      round the row's ends and give the same walk. None of those scenarios records the number of
      cell searches or which cells were exempt, so the walk does not tell the rectangles from a
      successful retry. `unmeasured` for
      the exemption that units, aircraft, terrain objects and wall sections give: whether a found
      path needs such an exempt cell off the chain depends on which chain the coarse search picks,
      and on open ground that comes down to the order of equal routes (claim 12a), which a test map
      cannot set apart from the rest of the search, so no practical map makes that exemption decide
      a route by itself. P1 (a) allows for it, and the secondary checks of P1 and P12 compare whole
      routes with it included, but neither is a test of it alone. `unmeasured` for the removal of a to-tile
      building's exemption: once the building has gone, its rectangle holds nothing, so an exemption left there could decide a
      route only as the exemption of units and terrain objects does, by letting a found path use a
      cell off the chain, which no practical map isolates (above)
    - Confidence: medium for the overall scheme, the cost formula, the progress rule, the retry's
      two branches and the corridor exemption of units, aircraft, terrain objects and wall
      sections, including that the exemption ignores layers and that aircraft count (both rest on
      the corridor exemption's `unmeasured` backing above). Medium that a unit goes round a row of
      buildings that open ground surrounds instead of giving up, until P31, P12 and P12 run S have
      run (they tell this from giving up). Medium (`unmeasured`, above) that this comes from the
      corridor exemption that a building gives the cells round its foundation: the same scenarios
      would pass with any exempt band that reaches round the row's ends, or with a successful retry.
      Medium (`unmeasured`, above) that the unit walks out on the first cell search. Medium for the
      details of the exemption rectangle of buildings: its size and position, that every building
      placed on the map gets it at placement (by the map or by a house), that it is taken away when
      the building leaves the map, that a wall type gives none, and that a change of owner leaves
      it. They are `unmeasured`: those scenarios judge only whether the unit walks out, which any
      exempt band reaching round the row's ends, or a successful retry, would give, and none of them
      records which cells were exempt or how many cell searches were made. Medium for the threat
      term being 0 at level 0, and for the limit of 5 cell searches until P12 run T has run. Medium
      (`unmeasured`, above) for the end of a level's search, which is the coarse search's end rule.
      Medium for the coarse threat at levels 1 and 2 (F5; which records hold threat is claim 16a).
      Low for the per-category constants' effect, and for when a moving vehicle's or infantry unit's
      recorded cell changes; medium for an aircraft's exemption cells around a cell it was placed on
      the ground in or landed in. Low for the exemption around the placement cell of an aircraft
      placed in the air, which stays after its first landing or after it leaves the map unlanded, as
      claim 16a and S-0009 claim 38 rate the aircraft placed in the air. Low, for the same
      reason, for the removal around the cell whose coordinates are both 0 when such an aircraft
      leaves the map unlanded (`unmeasured`: it changes no legal cell, so no run can observe it).
      The sectors and
      connections carry the confidence of claim 13b
    13a. **Free part, whole and split.** The *free part* of *S* is the set of cells of *S* that
    can be reached from the progress cell by steps in the eight directions that stay inside *S*,
    through cells whose entry outcome for the unit (entering from the step's direction, at the
    cell's own height level) is *clear* or whose category the zone table does not answer *yes* for
    the unit's movement zone; the height rule of claim 10 plays no part
    (`quirks.corridor_free_part_ignores_height`). The free part *borders* a sector other than *S* when a cell of that
    sector next to the free part meets the same outcome-or-category condition. Whether *S* is
    *split* is decided at one *probe cell*: with the progress cell at (*x*, *y*) and *B* the block
    size of level *L*, the probe cell is (*x* + (*x* mod *B*), *y* + (*y* mod *B*)), in exact integer
    arithmetic on cell coordinates. *S* counts as split when the probe cell is inside the usable
    area, belongs to *S*, and the free part does not cover all *B* × *B* cells of *S*'s block.
    Otherwise *S* counts as whole (`quirks.corridor_split_probe`).

    The movement zone of the outcome-or-category condition is the unit's own, also for an infantry
    type with the jumpjet flag, whose corridor was found for movement zone *infantry* (claim 13).
    Under this condition a cell whose category the unit's own movement zone cannot use belongs to
    the free part whatever its entry outcome, and so does a cell with outcome *clear*. Only a cell
    of a usable category with an outcome other than *clear* stops the free part
    (`quirks.corridor_free_part_unusable_cells`). With that switch off, a cell belongs to the free
    part only when its category is usable for the unit's own movement zone *and* its entry outcome
    is *clear*, and the same stricter condition decides which neighbouring sectors the free part
    borders. The two switches are independent: `quirks.corridor_split_probe` changes only the split
    test, and `quirks.corridor_free_part_unusable_cells` changes only which cells the free part and
    its borders contain.

    Where the category part can show: every cell of *S* has *S*'s category, because a sector lies
    inside one region. For a unit whose corridor used its own movement zone, that category is
    usable, because the level-0 sector of the progress cell is on the corridor and the sectors of
    every level that hold that cell lie in the same region. So for such a unit the category part
    acts only on cells of neighbouring sectors whose category the movement zone cannot use, and no
    corridor ever enters those sectors, so whether their connections to *S* close changes no later
    coarse search. The category part, and the switch, can change a retry only for an infantry type
    with the jumpjet flag whose own movement zone cannot use a category that movement zone
    *infantry* can.
    - Backing: `measurable` only for aggregate retry success/failure through P12 run T (the strip
      and ledge maps; the building maps of P12 make no retry). The free-part
      predicate, probe coordinates, whole/split classification and individual closure choices
      are `unmeasured` separately: run T observes routes and failures, not these judgements, and
      its outcomes also depend on corridor ties. `unmeasured` for the effect of
      `quirks.corridor_free_part_unusable_cells`, which shows only for a jumpjet infantry type as
      described above whose corridor search fails, a case no scenario of this spec sets up
    - Confidence: high for the outcome-or-category condition as stated, apart from the height
      clause, and for the movement zone it uses. Medium that the height rule of claim 10 plays no
      part, as the entry for `quirks.corridor_free_part_ignores_height` rates it: run T shows only
      the give-up at a ledge, not the free part itself. Medium for the probe cell and for the
      reasoning on where the category part can show. The effect of both on the original's routes
      has not been checked

    13b. **Sectors, connections and coarse end cells.**
    - *Sectors.* At each level, the cells of one region that lie in one block form one sector or
      more: two of them are in the same sector when a chain of neighbouring cells (eight
      directions) of that region inside that block joins them, each two consecutive cells of the
      chain differing by at most 1 height level. The join of claim 3 along cell x across 2 or 3
      levels does not apply here, so where a region holds together only through such a step, its
      two sides are different sectors, and no connection joins them directly (below). Unless other
      sectors connect them, the coarse search then fails between them, and the table of claim 13
      applies. Off-map cells belong to no sector. A
      sector's category is its region's, which all its cells share. By this rule a sector lies
      inside one sector of the next level, its parent.
    - *Connections.* Two sectors of one level are connected when they contain two neighbouring cells
      (eight directions), both inside the usable area, whose height levels differ by at most 1,
      whatever the two sectors' categories and regions. Unlike the adjacency of regions in claim 3,
      a sector of category *blocked* has no exception from the height limit. A connection is
      *diagonal* when every such pair of cells between the two sectors is a diagonal pair whose two
      cells lie in different block columns of that level, so that the pair crosses the left or right
      edge of a block on the slant, and *straight* otherwise. On open ground, where each block is one
      sector, a block's sector therefore has straight connections to the four blocks beside it and
      diagonal ones to the four blocks at its corners.
    - *Links.* An enabled link (claim 3) adds connections at every level, all of them straight, with
      no height or usable-area condition: between the sectors of its two end cells and, for a
      bridge, also between the sectors of the cell beside one end cell and the cell beside the
      other end cell on the same side of the bridge, for each of the two sides across the bridge's
      axis. The tile of the link's **first end alone** decides the displacement at both ends.
      the second end's tile and the link's line do not decide it. Let *i* be the first end's
      tile's place, 0 to 15, in its bridge or wooden-bridge role (S-0010 table B). The two
      side connections use these displacements, in integer cells:

      | First end's *i* (either role) | First side displacement | Second side displacement |
      |---|---|---|
      | 0, 1, 6, 7, 8, 9, 10 | (0, −1), map north | (0, +1), map south |
      | 3, 4, 11, 12, 13, 14, 15 | (+1, 0), map east | (−1, 0), map west |
      | 2, 5 | (−1, −1), map north-west | (+1, +1), map south-east |

      Each side connects the sector of the first end plus its displacement to the sector of
      the second end plus **the same displacement**. All additions are exact integer cell
      arithmetic, without rounding. Thus a diagonal first-end tile uses diagonal side cells
      at both ends even when the second end has an ordinary tile. For ordinary span starts
      of S-0010 table B these are the two neighbours across the span's line; places 2 and 5
      do not start a span under that table, but this rule fixes their side cells too. For a tunnel, which cells besides its two end cells are joined is not established.
      In our engine two sectors share at most one connection: when a link joins two sectors that are
      already connected, their connection is straight. The original can keep the two side by side,
      which gives the same route costs (departure D7).
    - *Coarse end cells.* The coarse search runs from the start's coarse end cell to the
      destination's. The start's is the start cell, except when the unit is on a bridge deck; the
      destination's is the destination cell, except when that is a bridge cell. In those two cases
      the coarse end cell is a cell at an end of the bridge, on the same lane as the cell *X*
      concerned, given by the table below. The coarse estimate of claim 24 finds its coarse end
      cells in the same way, with the found cell in place of the start, which counts as on a bridge
      deck when it is a bridge cell. The corridor condition of claim 14 reads the coarse end cells.

      The bridge's link is the first bridge link, in creation order (claim 3: the visiting order of
      the spans' start cells), that *X* lies within 2 cells of (Terms). With *o* the distance,
      signed, by which *X* lies to the side of the link's first end cell across that line, the two
      **lane cells** are the link's two end cells, each moved by *o* across the line. *Y* is the
      nearest cell that is not a bridge cell, going from *X* east along cell x for a link along
      cell x, or south along cell y for a link along cell y.

      | Link of *X* | Condition | Coarse end cell |
      |---|---|---|
      | found, enabled | — | the lane cell nearer to *X*, by the table square root (S-0204) of the squared distance in cells truncated to a whole number; the lane cell at the link's second end when both distances are equal |
      | found, not enabled | *Y* meets the condition of a bridge end (Terms: its tile is one of the 16 tiles of the bridge or wooden-bridge role of its theater, and its land type is not rock) | the lane cell at the link's second end |
      | found, not enabled | *Y* does not meet that condition | the lane cell at the link's first end |
      | none found | at least one of *X*'s two bridge ends lies inside the usable area | the nearer of those of *X*'s bridge ends that lie inside the usable area; the first bridge end on equal distance |
      | none found | neither bridge end lies inside the usable area | the original reads undefined data; our engine uses *X* itself (departure D6) |

    - *The original's labelling.* The original builds sectors and connections by labelling the map
      in a fixed order, as it builds regions (claim 3), and its result can differ from the rule
      above in three ways: a sector can be split where only a diagonal pair of its cells joins two
      parts whose other neighbours differ by 2 or more height levels; a connection that rests on
      such a diagonal pair alone can be missing; and a connection can be diagonal in the original
      although the two sectors also have a straight pair, when its labelling met a diagonal pair
      across a block's left or right edge first. The first two need neighbouring cells 2 or more
      levels apart. The third is common wherever a region's border meets a block's left or right
      edge, but it changes only the extra cost of 0.001. On ground whose neighbouring cells differ
      by at most 1 level, the original's sectors and connections are those of the rule. Our engine
      follows the rule (departure D4, accepted).
    - Backing: `measurable` only for aggregate route results through P1 and P12 and their
      secondary checks. Individual sector membership and connection classification are
      `unmeasured`: no scenario isolates them from coarse ties, retries and straightening.
      The reference checks assess them together. `unmeasured` for the coarse end cells on a bridge: a corridor that starts
      in the wrong sector changes only which corridor is chosen, and a coarse search that fails
      falls back on a search without a corridor that still finds a path, so no route isolates them.
      `unmeasured` for the bridge side connections (Q-0088): they affect only corridor choice,
      inseparable from the coarse-search ties and labelling departures. The displacement table
      and use of the first end's tile have high confidence
    - Confidence: medium for the sectors, the connections and their two kinds as stated, and for
      the agreement with the original on ground without steps of 2 levels or more. Low for the
      links' side cells and for the coarse end cells on a bridge

14. **When the corridor is used.** The corridor is used only when all of these hold: the unit
    has entered the usable area (a unit that arrives from outside it
    counts as outside until it has first entered); the coarse end cells of the start and of the
    destination (claim 13b) are inside the usable area; and the unit is not in the corridor-disabling state. The unit is in
    that state when it has entered the usable area and any of these holds: its type is a train; it
    is a loaner; its current behaviour is *retreat*; or it is a team member whose team is carrying
    out a "move to a waypoint marker" script step whose waypoint marker lies outside the usable
    area, the team being in its moving state for that step and its script not yet finished. Other
    script steps that move a team, such as "move to a cell", do not put a unit in this state. So a
    train never uses the corridor: before it has first entered the usable area the first condition
    fails, and afterwards it is in the corridor-disabling state. When
    the corridor is used and the reach regions of start and destination (the destination after
    claim 24) differ, read as in claim 6, the path computation fails at once. That failure counts as a failed
    search with the gate passed, so the search-failure rows of claim 23 apply. Without the corridor,
    the cell search runs over the whole map even when the reach regions differ.
    - Backing: `measurable` (P11 run 1 for a unit that has not yet entered the usable area,
      run 2 for the team condition; P12 run T with the train type for trains). `unmeasured` for the
      post-substitution reach-region comparison: no scenario changes a passed gate to different
      reach regions while preserving corridor eligibility. `unmeasured`
      for the loaner and *retreat* conditions: no order or map setting of the measurement setup is
      known to make a unit a loaner or to put it on *retreat* at a chosen moment
    - Confidence: medium. Low for the loaner and team conditions, and for what the team's moving
      state requires

15. **A blocked destination is approached.** When the destination cell's entry outcome is *blocked*
    as the search reaches a neighbour of it, and that neighbour's search layer is within 1 height
    level of the destination's search layer, the search ends there: the path leads to the neighbour
    of the destination, not into it. This does not apply to vehicles whose type is marked passive.
    Because a found path must have at least one step (claim 12), a unit that already stands next to
    its blocked destination gets no path from this rule: the search fails, and claim 23 applies.
    The unit is within 1 cell, so it gets row 3 (it does not give up), except when the destination
    is a bridge cell and the unit is not on a bridge deck, which gives row 2.
    - Backing: `measurable` (scenario P7, runs 5 and 6)
    - Confidence: medium (the passive exception: low)

16. **Train restriction.** For a type marked as a train, the three neighbours of the start cell that
    lie behind the train (the step directions *d* with (*d* − *f*₈) mod 8, taken from 0 to 7, equal
    to 3, 4 or 5, where *f*₈ is the 8-direction of its current body facing) cannot be entered for
    the whole search: no path passes through them, and a destination among them is never reached, so
    the search fails. The exclusion applies on one search layer of each of the three cells, the
    ground layer unless the start's search layer lies at least 2 height levels above that cell's
    height level. So the path the search finds never starts with a reversing step. The exclusion
    holds for the search only: the straightening passes (claims 17 and 18) judge the three cells by
    their entry outcome like any other cell, so either pass can give the stored path a first step
    into one of them. A train ordered to a cell a few cells straight behind it can therefore start
    by stepping diagonally backwards (scenario P9, train run 2, `quirks.train_exclusion_search_only`).
    - Backing: `measurable` only with test data (scenario P9: the order onto a cell behind the
      train for the exclusion, the orders to cells further behind it for the first step after
      straightening). Stock data is not known to use trains
    - Confidence: medium (low for which search layer is excluded on a bridge)

### Threat records

16a. **Threat records of a house (Q-1751).** Every house keeps a threat record for each threat
    block of the map (Terms). The threat value of a cell (claims 17 and 18) and the coarse threat
    (claim 13, F5) are read from these records, always from those of the moving unit's owner. The
    records describe objects, not danger: they are kept the same way for human-controlled and
    computer-controlled houses, apart from one test of the rebuild (below). They are read only for
    a unit whose threat coefficient *k* exceeds 0.00001 (claim 13, F7) or, in the straightening
    tests, multiplied by *k*.

    *What poses a threat.* Each vehicle, infantry unit, aircraft and building on the map has a
    threat amount, an integer:

    | Object | Threat amount |
    |---|---|
    | A building with at least one occupant (S-0508) | the number of occupants × the per-occupant threat (Import mapping) |
    | A building with no occupant that holds a bunkered vehicle (S-0606) | the threat field of the bunkered vehicle's type |
    | Any other building, vehicle, infantry unit or aircraft | the threat field of its own type (Import mapping) |

    Nothing else enters the amount: not the object's weapons, range, damage, health, veterancy or
    cloaking, not the passengers of a transport, and not whether the recording house can see the
    object or has ever seen it. An object off the map (glossary *on the map*), such as a passenger
    or a garrison occupant, contributes nothing of its own.

    *Whose records.* An object's amount goes into the records of every house other than its owner
    that does not ally its owner. The recording house's own side of the relation decides (glossary
    *alliance*): when only house A allies house B, B's objects stay out of A's records, while A's
    objects are in B's. So, under the incremental rules below, a house's records never hold its own
    objects or those of houses it allies.

    *Where.* An amount *a* is spread over 3 × 3 threat blocks centred on the object's threat block:
    *a* in that block, ⌊*a*/2⌋ in each of the four blocks sharing an edge with it and ⌊*a*/4⌋ in each
    of the four diagonal ones. A building's block is the one holding its top-left foundation cell,
    not its centre, so the area of a large building lies towards its top-left side; a vehicle's or
    infantry unit's block is the one holding the cell it was last recorded in (claim 13), and an
    aircraft's block follows the aircraft rule below. The spread footprint is therefore 12 × 12
    cells, reaching between 4 and 7 cells from that cell along each axis, whatever the object's
    weapon range. So, for Q-1751 question 1, an object adds to the records only within its 12 × 12
    footprint, from 4 to 7 cells away along each axis. Where a route bends follows from the coarse
    search and the straightening rules over those records, and depends on the unit's coefficient
    *k* times the amount in each block it crosses (the full amount in the object's own block, a
    half or a quarter in the blocks round it), through
    the threat value of claims 17 and 18 and the coarse threat of F5. Taking a contribution away
    subtracts the same three amounts, and a record that would fall below 0 becomes 0 instead.

    *When.* The records change at once, in the same tick, at these events:

    | Event | Change |
    |---|---|
    | An object is placed on the map (built, deployed, unloaded, leaving a building, or placed at the start) | its current amount is added at its block |
    | An object leaves the map (sold, entering a transport or a building, or a destroyed object when it is taken off the map, which for some destroyed objects is only after a death sequence: glossary *on the map*) | the amount last added for it is taken away where it was added (for an aircraft, see the aircraft rule below) |
    | A vehicle or infantry unit is recorded in a cell of another threat block | the amount last added is taken away at the old block, and its current amount is added at the new block |
    | An object's owner changes | the amount last added is taken away from the records of the houses that record the old owner's objects, and, if the object is on the map, its current amount is added to the records of the houses that record the new owner's objects |
    | An infantry unit enters a building as an occupant, or a building's occupants leave or are killed | the building's amount last added is taken away and its current amount is added, at its block |
    | A house makes or breaks an alliance | that house's records are rebuilt (below) |

    Moving within one threat block changes nothing. There is no periodic refresh. A building's
    amount follows a change of bunkered vehicle only at the building's next event of the table.
    The amount last added for an object is one figure, the same for every house that records it:
    the amount added at its latest event of the table other than a rebuild. A rebuild (below) adds
    the object's current amount but leaves that figure unchanged. So after a change of bunkered
    vehicle and a rebuild of only one recording house, a later event takes the older figure away
    from that house too, which then keeps the difference or loses more than was added (floor 0).

    *Aircraft.* An aircraft's threat does not follow it in flight. Its amount is added at the block
    of the cell where it is placed on the map, whether it is placed on the ground (for example at
    an airfield) or in the air. Placed on the ground, it is recorded in that cell; placed in the
    air, it has no recorded cell yet. Its movement changes no record, however far it flies. Each
    time it lands, it is recorded in the cell it lands in, and again no record changes. At an
    owner change its amount is taken away and added again at the block of its recorded cell or, if
    it has never had one, at the block of the cell whose two coordinates are both 0. When it leaves
    the map, its amount is taken away at the block of its recorded cell at that moment or, if it
    has never had one, at the block of that same cell, the one whose two coordinates are both 0.
    So an aircraft that has landed in another block, or that was placed in the air, leaves its
    amount at its placement block ("stale threat", until the house's next rebuild) and takes the
    amount away at a block where it was never added, lowering what other objects put there (floor
    0). In a rebuild an aircraft counts at the block of its recorded cell; one that has never had a
    recorded cell is skipped (`quirks.aircraft_threat_stays_put`).

    *Rebuild.* When a house makes or breaks an alliance (S-0412 claim 8, S-0414), its records are
    set to 0 and filled again from every object on the map that is not destroyed and whose current
    amount is above 0, spread as above:
    - A vehicle, infantry unit or aircraft that has a recorded cell and is not the house's own
      counts at its block when the house is human-controlled and does not ally the object's owner,
      and always when the house is computer-controlled, allies included. An *AI house* counts as
      computer-controlled here (decided at the delegated sign-off, 2026-10-25). This house test
      is the one exception to the glossary's rule that an AI house has the rules of a
      human-controlled house (glossary *AI house*); Departure D5 still applies to its units.
    - Every building counts at its block, whoever owns it: the house's own and its allies'
      buildings included.

    The same holds however the alliance is made or broken: by the teams of game start, by a
    player's alliance order, or by a map trigger's action. A map trigger's action that makes or
    ends an alliance acts on both houses at once (S-0209, the backing of its alliance views), so
    it rebuilds the records of both houses.

    The incremental rules never change a house's records for its own or its allies' objects, so
    those extra contributions stay ("stale threat") after the object moves, is destroyed or leaves
    the map, until the house's next rebuild (`quirks.threat_rebuild_counts_friends`). A house that
    makes the alliances of game start (S-0412 claim 8) is rebuilt then, and those alliances are
    made after the map's objects and the starting units are placed. So such a house starts with
    stale threat from every building that the map places for it or for a house it allies, and, if
    it is computer-controlled, from its allies' starting units. A game-start rebuild omits every
    aircraft that the map places in the air (it has no recorded cell), so such an aircraft is not
    in the rebuilt house's records until a later rebuild after it has landed, unless an owner
    change adds it first (below).

    *Effect on paths.* A unit with a large enough coefficient is steered away from threatened blocks
    by the coarse search: at levels 1 and 2 it adds the threat term to the entry cost of a sector
    (F5), so a long route can take the other side of a lake or a base. The straightening passes test
    threat too (claims 17 and 18). A path can still pass threatened cells: the level-0 corridor and
    the cell search ignore threat, cells with the corridor exemption (claim 13) are not held to the
    corridor, and the right-angle flattening reads only one cell's threat value
    (`quirks.flatten_threat_start_cell`).
    - Backing: `measurable` by scenario P30, with the threat records read by the read-only recorder
      and each part's route check, and by P10 for the aggregate route effect, for:
      - the threat blocks, the spread, the first and third rows of the amount table and the
        removal and occupant rows of the event table (P30)
      - the placement row for objects placed in play: the infantry units placed on leaving a
        building (part f2) and on unloading from a transport (part i)
      - the building's top-left cell (part a2)
      - a computer-controlled house's rebuild counting an allied vehicle (part c)
      - what the amount leaves out, and entering and leaving a transport (part i)
      - entering and leaving a building (part f2)
      - the rebuild skipping an aircraft placed in the air (part h2, which places one in the air
        by its placement cell, S-0009 claim 38).

      `unmeasured` for the following rules, which no part of P30 isolates:
      - the placement row for the objects that the map places: the alliances of game start
        (S-0412 claim 8) can rebuild the reading house's records after those placements and give
        the same records, so parts a, a2, b, d, e, h and i show the same pattern whether or not
        the placement added it, and no part places a building, a vehicle or an aircraft in play
        and reads an enemy house's records then
      - where an aircraft's amount is taken away, and its effect on other objects' threat at that
        block (as Quirk switches, `quirks.aircraft_threat_stays_put`, says)
      - a computer-controlled house's counting of its allies' infantry units and aircraft in a
        rebuild: part c tests that count with an allied vehicle only
      - for an aircraft placed in the air, its owner change and its removal at the block of the
        cell whose two coordinates are both 0, and its corridor exemption around the placement
        cell (claim 13)
      - the owner-change row: an owner change inside the recorded window needs a capturing or
        controlling unit there too, whose own amount changes the same records in the same ticks
      - the bunkered-vehicle amount, its refresh only at the building's next event and the one
        figure last added that a rebuild leaves unchanged: the
        amount shows only after a later event of the table, such as a rebuild, and a part
        would have to follow a bunker, its vehicle and that event in one window, with the
        vehicle's own amount changing the same records
      - the negative threat field (Edge cases and quirks, `quirks.negative_threat_field`): stock
        data is not known to use one, and the floor at 0 makes its effect depend on the order of
        placements around it
      - the one-sided alliance: in P30, H and A ally each other, so no part tells a test of the
        recording house's own side of the relation from a test of both sides.
    - Confidence: medium, until the part of P30 that tells each rule from its alternatives has
      run, for the threat blocks, the spread, the amounts of the first and third rows of the amount
      table, what the amount leaves out (the sentence after the table), the placement row for
      objects placed in play, the removal and occupant rows of the event table (entering and
      leaving a transport or a building included), and which houses record an object, the
      one-sided alliance excepted. Medium, as an `unmeasured` rule with its stated reason, for the
      placement row for the objects that the map places. Medium for the building's
      top-left cell, the aircraft rule for an aircraft that is not placed in the air, the rebuild
      with its house test and its place at game start, the owner-change row, the bunkered-vehicle
      amount with its delayed refresh and the one figure last added, the negative threat field
      and the one-sided alliance. Low for
      the aircraft rule for an aircraft placed in the air (where its amount is added, its omission
      from a rebuild until it has landed, its owner change, its removal at the block of the cell
      whose coordinates are both 0, and the corridor exemption round its placement cell), as S-0009
      claim 38 rates the aircraft placed in the air: part h2 rests on that placement. Low for the
      timing of a moving unit's change of block: it follows the recorded cell of claim 13, whose
      moment of change is rated low there.

### Straightening

17. **Right-angle turns are flattened.** After a path is found, its right-angle turns are flattened
    once. The rule is given as the path that results.

    *Runs and V's.* A **run** is a stretch of consecutive equal steps that cannot be extended by an
    equal step before or after it, except that it never includes a final step (below). A **V** is
    a run of *n* steps in direction *d₁* followed directly by a run of *m* steps in a direction *d₂*
    at a right angle to it ((*d₂* − *d₁*) mod 8, taken from 0 to 7, equal to 2 or 6), where the
    first run is one that can open a V. Every run of diagonal steps can open a V. A run of straight
    steps can open a V only in the cases listed under *V's of straight runs* below. Tunnel steps
    are never part of a V.

    *Flattening one V.* The last *j* steps of the first run and the first *j* steps of the second
    run (2*j* steps) are replaced by 2*j* steps in a direction *d₃* that depends only on the two
    directions *d₁* and *d₂*, in either order. For two diagonal runs *d₃* is the straight direction
    between them: 7 and 1 give 0, 1 and 3 give 2, 3 and 5 give 4, 5 and 7 give 6. For two straight
    runs (below): 0 and 2 give 1, 2 and 4 give 3, 4 and 6 give 5, and 6 and 0 give 0, not the 7
    between them. *j* is the largest value from 1 to min(*n*, *m*) for which every cell the 2*j* new
    steps pass through has entry outcome *clear*, carries no congestion mark (claims 9 and 19), and has
    threat value × *k* below 1.0 (F7), where the threat value is read at the cell where the new
    steps start, the same cell for every cell checked (`quirks.flatten_threat_start_cell`). If no
    *j* qualifies, the V is *not flattened* and stays as it is. For two diagonal runs the new steps
    end on the same cell as the steps they replace, so the rest of the path is unchanged.

    *Which V's are flattened.* For the path after the preceding judgement, the table gives the
    resulting final steps and the portion eligible for later transformations. When multiple V's
    are eligible, the resulting path is the one obtained by transforming the V whose first run
    begins earliest among the non-final steps. This precedence applies whether or not the V's
    overlap. The table describes path results, not an implementation's scan procedure.

    | Judgement result | Steps made final | Remaining path considered by later judgements |
    |---|---|---|
    | V flattened | Every step before the replacement's new steps, including the *n* − *j* steps retained from its first run | The path after those final steps, beginning at the first new step |
    | V not flattened | Every step through the end of its first run | The path after those final steps, beginning at the start of its second run |

    A final step belongs to no later V and never joins a later run, even when its direction equals
    that run's steps. Everything after the final steps is judged as it then stands. This gives the
    following consequences:
    - When the whole first run is replaced (*j* = *n*), the step before the old first run is final
      and never forms a V with the new steps, even when it is a straight step at a right angle to
      them (example (b)).
    - Two stretches of equal steps that end up side by side count as one run only when the earlier
      one is not final. Thus new straight steps of a flattened V do not join equal straight steps
      before them, which are final (example (c)), but do join equal straight steps directly after
      them (example (d)).
    - After a flattened V, the *n* − *j* retained steps are final and belong to no later
      transformation. The eligible suffix begins with the 2*j* new steps; these and any equal
      steps directly after them form its first run. Whether that run can open a V depends on its direction
      and how it arose: a run of diagonal steps always can, while a run of straight steps can only
      in the cases listed under *V's of straight runs*. After a flattened V of diagonal runs, the
      new steps are straight; the remaining *m* − *j* diagonal steps of the second run form their
      own run after them and can open a further V. After an unflattened V, the eligible suffix
      starts with its second run, which can open a further V. After a flattened V of
      straight runs, the rules under *V's of straight runs* apply.
    - Where two V's would share a run, the one nearer the path's start is judged first; the later
      V sees only what the earlier judgement left of the shared run (examples (a) and (c)).

    *V's of straight runs* (`quirks.flatten_straight_run_v`). A run of straight steps can open a V
    in three cases: it begins with the new straight steps of a flattened V of diagonal runs (with
    any equal straight steps directly after them, by the consequence above); it is the second run
    of a V of straight runs that was not flattened; or it begins with the new steps of a flattened
    V of straight runs whose directions are 6 and 0, because those steps lead north (below). No
    other run of straight steps opens a V, so straight runs that the search produced never do,
    except in those ways. A V of straight runs is judged and replaced by the same rule, with 2*j*
    new steps. For the pairs 0 and 2, 2 and 4, and 4 and 6 these are diagonal and do not end where
    the replaced steps end: *j* steps in each of two straight directions move the path *j* cells
    along each axis, the 2*j* diagonal steps move it 2*j* cells along each. For the pair 6 and 0,
    *d₃* is 0 (north), not 7: the 2*j* steps all lead north. In every case each later step of the
    path is shifted by the same amount, and the stored path does not end on the cell the search
    reached.

    After a V of straight runs is replaced, the eligible suffix likewise begins at the first new
    step, with the *n* − *j* kept steps final. Its possible V shapes depend on the pair:
    - Pairs 0 and 2, 2 and 4, 4 and 6 (new steps diagonal). The 2*j* new steps are a run of
      diagonal steps, which can open a V, but its only possible partner is the step after the
      second run, so only when *j* = *m* and that step is at a right angle to the new steps. When
      *j* < *m*, the rest of the second run (straight) comes after the new steps at 45° to them. It
      is a run of its own, and it never opens a V.
    - Pair 6 and 0 in the order 6 then 0 (new steps north). The rest of the second run is north as
      well and joins the new steps, so the run judged next has *j* + *m* steps. It can open a V
      with a straight step at a right angle after it.
    - Pair 0 and 6 in the order 0 then 6 (new steps north). The rest of the second run is west, at a
      right angle to the new steps, so the new steps and the rest form a V with *n* = 2*j* and
      *m* − *j* steps, judged by the same rule.
    - The last V judged is the one whose second run ends the path. Nothing that its new steps form
      afterwards with the rest of that run is judged, so the case 0 then 6 at the end of a path
      leaves the rest of the run as it is.

    *Worked examples* (directions as in Terms; every cell clear, no congestion mark, no threat). Example
    (a) and (b) hold under any setting of `quirks.flatten_straight_run_v`, examples (c) to (f) with
    it on:
    - (a) The search finds 1, 3, 3, 3, 1, which ends at (*x* + 5, *y* + 1). The first V is (1 | 3,
      3, 3) with *n* = 1, *m* = 3, *j* = 1, giving 2, 2, 3, 3, 1. The rest of its second run, 3, 3,
      opens the V (3, 3 | 1), *j* = 1, giving 2, 2, 3, 2, 2, which ends on the same cell.
    - (b) 1, 3, 3, 5 ends at (*x* + 2, *y* + 2). The V (1 | 3, 3) gives 2, 2, 3, 5. The 2, 2 are
      followed by a diagonal step, so the next V is (3 | 5). It has *j* = *n* = 1 and gives
      2, 2, 4, 4, which makes the 2, 2 before its new steps final. So they do not form a V of
      straight runs with the 4, 4 that now follow them, although east and south are at a right
      angle.
    - (c) 1, 3, 1, 3, 4, 4, 4 ends at (*x* + 4, *y* + 3). The two V's (1 | 3) give 2, 2, 2, 2, 4, 4,
      4. The first pair 2, 2 is final, so the run that opens the next V is the second pair only:
      the V (2, 2 | 4, 4, 4) has *n* = 2 and *j* = 2, and gives 2, 2, 3, 3, 3, 3, 4, which ends at
      (*x* + 6, *y* + 5). Had the four 2's counted as one run, *j* would be 3.
    - (d) 1, 3, 2, 4, 4, 4 ends at (*x* + 3, *y* + 3). The V (1 | 3) gives 2, 2, 2, 4, 4, 4: the
      search's own step 2 joins the new pair, so the V (2, 2, 2 | 4, 4, 4) has *n* = 3 and *j* = 3,
      and gives six steps 3, which end at (*x* + 6, *y* + 6).
    - (e) 5, 7, 0 ends at (*x* − 2, *y* − 1). The V (5 | 7) gives 6, 6, 0, and the V (6, 6 | 0) has
      *j* = 1 and *d₃* = 0 (directions 6 and 0), giving 6, 0, 0, which ends at (*x* − 1, *y* − 2).
    - (f) 7, 1, 6 ends at (*x* − 1, *y* − 2). The V (7 | 1) gives 0, 0, 6. The run 0, 0 is followed by
      the west step, a V of straight runs with *n* = 2, *m* = 1, *j* = 1 and *d₃* = 0, which makes the
      first 0 final, keeps it and gives 0, 0, 0. The path now ends at (*x*, *y* − 3).
    - With `quirks.flatten_straight_run_v` off, examples (c) to (f) stop after their V's of
      diagonal runs: (c) gives 2, 2, 2, 2, 4, 4, 4, (d) gives 2, 2, 2, 4, 4, 4, (e) gives 6, 6, 0 and
      (f) gives 0, 0, 6, each ending where the search's path ends.
    - Backing: `measurable` for the flattening of V's of diagonal runs on clear ground after a
      path's 20th step, where claim 18 does not act (scenario P1, expectation (b), long runs).
      `unmeasured` for the threat test and `quirks.flatten_threat_start_cell`, for the order of
      judging and for V's of straight runs: a V arises only where obstacles or the corridor bend the
      path, and on open ground which corridor is chosen depends on the order of equal routes (claim
      12a), so no practical map can place a threatened cell, two V's that share a run, or a V of
      straight runs at a known point of a found path. The examples serve a reference implementation
      and the secondary check of P1
    - Confidence: medium. Medium also for V's of straight runs and their displacement, which no
      measurement backs

18. **Early detours are shortened.** This pass works on the path as claim 17 has left it, so the
    two passes together can give a path that neither pass alone would give. It examines the first
    20 places of that path (below) and replaces some *stretches* (runs of consecutive steps) by shorter or
    reordered ones with the same two ends. The rule is given by definitions; worked examples follow.

    *Shapes and acceptance.* A stretch from cell *s* to cell *e* is replaced by one of two shapes
    with the same ends and with as many steps as the Chebyshev distance between them: all diagonal
    steps first and then all straight steps (*diagonal-first*), or all straight steps first and then
    all diagonal steps (*straight-first*). The diagonal-first shape is used when it is acceptable,
    and otherwise the straight-first shape when that one is; the diagonal-first shape is not tried
    when the ends lie on one straight line, nor the straight-first shape when they lie on one
    diagonal line. When neither is acceptable, or when *s* and *e* are the same cell, the stretch
    stays as it is. A shape is acceptable only if every cell it enters has entry outcome *clear*
    and carries no congestion mark, and if its count of entered cells with threat value × *k* ≥
    0.01 (F7), each cell's own threat value being used, stays within a limit. The limit is 0, except
    for the closing stretch (below), for which it is 3. Threat is counted only when *k* exceeds
    0.00001. The replacement can have as many steps as the stretch: then the stretch only changes
    the order of its steps. The new steps take the first places of the stretch, and its remaining
    places become *removed steps*, which are dropped from the path when the pass ends.

    *What the pass judges on.* Places are the original step positions in the path received from
    claim 17, counted from its start. Only the first 20 places are eligible, including places
    whose steps a replacement removes. The legs, the reference distances and the triggers below are judged on the path
    as this pass received it; the choice of each stretch and the acceptance test see the path as
    the earlier replacements of this pass have left it. A replacement keeps both ends of its
    stretch, so every later step still starts and ends on the cells it did.

    *Legs.* The examined steps fall into *legs*. The first leg begins at the start cell. A step
    *turns back* when, added to the displacement of its leg so far (from the leg's first cell), it
    makes the absolute value of the x or the y component smaller. A step that turns back begins a
    new leg at the cell it starts from, and is then the first step of that new leg; the first step
    of a leg never turns back.

    *Reference distances and trigger predicate.* Each examined step has a **reference distance**
    and a **record**. A step **triggers** exactly when its reference distance is less than or
    equal to its record; its trigger corresponds to the stretch ending on that step's end cell,
    defined below.
    - In the first two legs, the reference distance is the Chebyshev distance from the path's start
      cell to the step's end cell. The record is the maximum of zero and the reference distances
      of all earlier examined steps in those two legs.
    - In the third and later legs, let *p* be the current leg's first cell, *q* the preceding leg's
      first cell and *e* the step's end cell. With `quirks.detour_mirrored_reference` on, the
      reference distance is the Chebyshev distance from *q* to 2*p* − *e*; with it off, it is the
      distance from *q* to *e*. The record is the maximum of the Chebyshev distance from *q* to
      *p* and the reference distances of earlier examined steps in the current leg. Coordinates
      and distances here use exact integer arithmetic, in cells.

    *The stretch of a trigger.* A stretch is a run of consecutive places of the path. The stretch
    of a trigger ends with the trigger step's place, on its end cell *e*. Its *earliest place* is,
    in the first and second legs, the path's first place, and from the third leg on, the place of
    the first step of the leg before the current one. Number the steps that are not removed, from
    the earliest place to the stretch's last place (the trigger step's place, or for the closing
    stretch the last examined place) in path order, *σ*₁, …, *σ*ₜ, and let *d*ᵢ be the
    Chebyshev distance from *e* to the cell where *σ*ᵢ starts. *σ*ᵢ is a *new maximum* when *d*ᵢ
    is larger than every *d*ⱼ with *i* < *j* ≤ *t*, so *σ*ₜ always is one. Let *u* be the largest
    *i* for which *σ*ᵢ is not a new maximum. When *u* exists and some *σ*ᵢ with *i* < *u* is a new
    maximum, let *v* be the largest such *i*: the stretch begins at the place right after
    *σ*ᵥ's place. Otherwise it begins at the earliest place. Either way the removed steps in it
    stay places of the stretch, also those before its first step that is not removed, so the
    replacement's new steps can take a place that a removed step held. The stretch runs from the
    cell where its first step that is not removed starts to *e* (from *e* to *e*, so it stays as it
    is, when every place of it is removed).

    *The closing stretch.* When the examined part has ended (after its 20th place or at the end of
    the path) and at least one step turned back, let *p* be the first cell of the last leg, *e* the
    cell where the examined part ends and *m* the number of places from the last leg's first step to
    the end of the examined part, removed steps included. When the Chebyshev distance between *p*
    and *e* is smaller than *m* − 1, the stretch ending on *e* is replaced as for a trigger, its
    earliest place being the place of the last leg's first step, its last place the last examined
    place, and its threat limit 3.

    *Tunnel steps.* No leg or reference-distance record spans a tunnel step. The eligible
    suffix after it has the next step's place as its earliest place and the same transformation
    rules as a new path. Its assumed start is the cell north of the tunnel entrance, rather than
    the tunnel exit (`quirks.detour_after_tunnel_position`; with
    the switch off, at the tunnel exit). A tunnel step is never part of a stretch.

    *Worked examples* (directions as in Terms, steps counted from 1, every cell entered by a shape
    clear, no congestion mark, no threat, switches at their defaults):
    **Example (a).** Input directions 1, 3, 4, 5, 4, 4 from (*x*, *y*) end at
    (*x* + 1, *y* + 4). Cell (*x* + 1, *y*) is not clear, so claim 17 leaves this input
    unchanged (P9 train run 4). Its leg boundaries and trigger predicates are:

    | Original place | Leg | End-cell reference distance | Record | Trigger |
    |---|---|---|---|---|
    | 1 | first | 1 | 0 | no |
    | 2 | second, starting at (*x* + 1, *y* − 1), where y displacement turns back from 1 to 0 | 2 | 1 | no |
    | 3 | second | 2 | 2 | yes |
    | 4 | third, starting at *p* = (*x* + 2, *y* + 1), where x displacement turns back; *q* = (*x* + 1, *y* − 1) | 2, to the mirrored end (*x* + 3, *y*) | 2, the distance from *q* to *p* | yes |
    | 5 and 6 | third | no larger than the record | 2 | yes |

    Distances and coordinates are integers in cells. The resulting stretches are:

    | Trigger place | Stretch selection | Replacement result |
    |---|---|---|
    | 3 | *d* values 2, 2, 1; only *σ*₁ is not a new maximum, with no earlier step, so the earliest place is 1 | 1, 3, 4 becomes diagonal-first 3, 2 and one removed place |
    | 4 | earliest place 2; its non-removed steps are 2 at place 2 and 5 at place 4, with *d* values 1, 1 | 2, removed place, 5 becomes 4 |
    | 5 and 6 | the same trigger predicate | the stretch stays straight |

    Output: 3, 4, 4, 4. The closing stretch has *m* = 3 places and distance 3 cells, so
    remains unchanged. With `quirks.detour_mirrored_reference` off, the third leg's distances
    from *q* are 3, 4, 5 cells, there are no further triggers, and output is 3, 2, 5, 4, 4.

    **Example (b).** Input directions 2, 4, 4, 5, 4, 4, 4 from (*x*, *y*) are P9 train
    run 2's path after claim 17. Their predicates and result are:

    | Original place | Reference distance, in cells | Record, in cells | Stretch result |
    |---|---|---|---|
    | 1 | 1 | 0 | no trigger |
    | 2, ending at (*x* + 1, *y* + 1) | 1 | 1 | trigger; *d* values 1, 1 select earliest place 1; directions 2, 4 become 3 |
    | 3 | 2 | 1 | no trigger |
    | 4 to 7 | 3, 4, 5, 6 | each below its reference distance | no trigger |

    Place 4 turns back along x, so the second leg begins at (*x* + 1, *y* + 2). Its
    closing stretch has *m* = 4 places and distance 4 cells, so remains unchanged.
    Output: 3, 4, 5, 4, 4, 4.

    - Backing: `measurable` with test data (scenario P9, train runs 2 to 4, whose expected paths
      follow from claims 7 to 18 with no exact tie), for the stretch selection, the mirrored
      reference and the shortening at the start of a path. The secondary check of P1 compares whole
      paths with claims 7 to 18 taken together. `unmeasured` for the threat limits and closing
      stretch: the train runs contain no threatening cells and their closing stretches are left
      unchanged. `unmeasured` for transformations after tunnel steps: P9 uses a tunnel but does
      not isolate a post-exit detour, and no scenario here distinguishes the assumed start cells
    - Confidence: medium for the selection and the acceptance rule, and for the order of the two
      shapes

### Re-paths and congestion

19. **Re-paths avoid congestion.** In search modes 1 and 2, cells around the blockage carry the
    congestion mark (so *τ* = 4 in F1) for the duration of that one search. The *cell ahead* is the cell
    one step from the unit in the 8-direction of its body facing. A unit in the cell ahead
    *qualifies* when, in mode 2, it is any vehicle or infantry unit, or, in mode 1, it is of a
    different type from the blocked unit, its type has a lower speed number (Terms: S-0207 claim 1
    for a vehicle type, S-0208 F4 for an infantry type) than the blocked unit's type, and it is
    inside the usable area. The two speed numbers are compared as integers 0 to 255, whichever of
    the two formulas gave each, so a vehicle and an infantry unit are compared directly. In
    addition, in both modes, a vehicle qualifies only if its stored path has at least two steps,
    and an infantry unit only if it has at least three (that mode 2 has these conditions too is
    this spec's assumption, not established, Open questions). Each cell is then counted once for
    each of these that names it:
    - (i) the stored path of each qualifying unit: the cells that its remaining stored steps (up to
      24) lead through, starting from the cell it is moving into, that cell itself not included,
    - (ii) the occupied cells of the 5 × 5 block centred on the cell ahead, except the unit's own
      cell (one count each),
    - (iii) the cell ahead itself (one count).

    In mode 1, when (i) counts no cell, nothing is counted and the search runs as mode 0 instead.
    Otherwise, during the search a cell carries the congestion mark exactly when one, but not both, of
    these holds: it carried the mark before the search, or its count is odd
    (`quirks.path_congestion_toggle`). After the search every cell carries the mark it carried
    before.

    So a cell counted twice is not made costly. In particular the cell ahead, which is normally
    occupied (that is why the unit is blocked), is counted once in (ii) and once in (iii). It
    therefore ends up *not* made costly, unless the stored path of a qualifying unit counts it a
    third time.
    - Backing: `measurable` (P14 for mode-1 stored-path marks and parity, P6 for timing).
      P25(c) tests mode-2 qualification independently of type and speed. The exact path-length
      cutoffs remain `unmeasured`: no run holds otherwise identical blockage state while varying
      remaining path length across the two-step and three-step boundaries. `unmeasured` for restoration of an
      independently pre-existing mark: no scenario establishes one before the search
    - Confidence: medium for the rule. Low for the exact unit test of mode 1 and the path-length
      conditions

### Path storage and timing

20. **Stored path and when it is recomputed.** A unit keeps at most 24 steps of a found path. A new
    search starts from the cell the unit is in and replaces the stored path. The unit computes a new
    path:
    - with the driving or ship locomotor, when it begins the last step of its stored path while its
      heading point is more than 512 leptons away (F6c). This search is made at once, in search
      mode 0, without waiting for the retry delay of claim 21. The walking and hovering
      locomotors make no such recomputation,
    - when it has no stored steps left and has not arrived (a fresh search, claim 21),
    - when its destination changes,
    - when its next step is blocked (claim 22 for driving, ship and hovering locomotors,
      S-0209 claims 7 to 10 for the walk locomotor),
    - and, with the driving, ship or hovering locomotor, in the chase case below.

    For driving and ship locomotion, search eligibility and results do not define whether
    a locomotor destination point is present. An accepted movement instruction installs
    that point before any search succeeds, even when its cell equals the unit's own cell.
    A wait, turn, retry delay or failed search that retains the movement goal keeps the
    point present. S-0207 claim 25 owns its assignment and clearing; neither search success
    nor goal-cell inequality substitutes for that presence test.
    Separately, a driving or ship vehicle with no active track removes an order-level
    cell destination when it occupies that cell, or falls back to rest when it has
    approach destinations (S-0207 claim 26). For a cell destination this is locomotion cleanup, not search
    failure or give-up: it needs no successful search, exact point alignment or due
    behaviour processing. For a destination that names a building, vehicle, infantry unit or
    aircraft, mere cell equality does not clear it; a vehicle at rest in guard instead needs its
    position to equal its locomotor destination point exactly (S-0207 claim 26).
    Final-track completion clears the locomotor point, not the order-level goal.
    Absent an intervening effect, the next no-track update clears that cell goal.

    For a hovering unit, the destination used by its movement completion and path searches
    in claims 20–23 is its movement destination, not its order-level destination. A search
    targets the movement destination's cell even when an accepted stop has cleared the
    order-level destination. S-0503 claim 16 owns when stop retains that point and the
    stored path; clearing the order-level destination alone does not require a new path.

    When a path is recomputed as the last stored step begins, the unit still takes that old step,
    and the new path's first step is dropped: the unit continues with the new path's second step
    from the cell the old step reached (`quirks.path_refresh_keeps_old_step`). When the new path's
    first step is the old step, as is usual, this has no visible effect. Otherwise every later step
    is taken from a cell one step away from the cell it was planned from, until a step is refused or
    the path is computed again. For a train type, the step in progress is kept on purpose: the search
    starts where that step leads and its result is stored after it, still up to 24 steps in total,
    so the quirk does not arise.

    When a unit with the driving, ship or hovering locomotor has a vehicle or an infantry unit rather than a cell as its
    destination, the stored path
    is cut to ⌊*D* / 256⌋ steps when that is less than 24, where *D* is the distance in leptons from
    the unit to its heading point (F6c). When *D* is under 256 the stored path is emptied at once, so
    a fresh search follows (subject to the path retry timer). The unit therefore computes a new path
    after about as many steps as there were cells between it and its heading point when it last
    computed one. Where the heading point lies while chasing belongs to the movement specs. When the
    chased unit moves slowly it stays close to the chased unit's position.
    The walk locomotor cuts no stored path for a chased unit either.
    - Backing: `measurable` (scenario P5, and P9's train refresh run for the train rule; P5 run 1i
      for the absence of the refresh in the walk locomotor; S-0504 N11 for ship refresh
      timing; S-0207 D29 for driving and ship destination presence before search and
      during retained-goal waits, including same-cell assignment; S-0207 D30 for
      no-track cell-goal cleanup distinguished from search failure; S-0207 D31 for a destination that names a building, vehicle, infantry unit or aircraft and for the guard exception; S-0503 H12 and H15
      for the hover movement/order-level destination split).
      The exact lepton-scale refresh boundary and the distinction between the
      table root and an exact Euclidean root are `unmeasured` for both driving and ships:
      P5 and S-0504 N11 place the heading point well beyond the threshold, not at a
      qualifying separation where the two roots change the refresh decision. They test
      refresh timing, not that rounding distinction. The ship retained-old-step effect
      and chase cut are `unmeasured`: no
      qualifying stock setup separates them from unchanged replacement routes or the
      chased unit's own cell-entry reactions (S-0504 claim 9). The hovering chase cut is
      `unmeasured`: S-0207 scenario D21 explains why its effect cannot be isolated from the
      hovering unit's other step-point reactions. `unmeasured` for the absence of the
      chase cut in the walk locomotor: a walker that follows a moving unit meets it within a few
      cells as a moving blocker or a stationary friendly, and S-0209's reactions then make new
      paths of their own, so the absence of the cut cannot be isolated
    - Confidence: high for the 24-step limit. Medium for the 512-lepton trigger, the chase cut, the
      train rule and their absence in the walk locomotor. High for destination presence
      independent of search success and goal-cell inequality (S-0207 claim 25). High for the cell/object distinction and the cleanup conditions of the object and guard rows, as S-0207 claim 26 gives them.
      Low for the dropped first step

21. **Retry delay.** The retry delay is *R* = trunc(*P* × 900) ticks (F3), where *P* is the
path-retry delay field in game minutes. The path retry timer of S-0209 counts it down. A unit
computes no new path while its retry delay runs, with these exceptions: the immediate recomputation
of claim 20 and immediate fresh searches of S-0207 claim 15 (driving locomotor), S-0504 claim 9
(ship locomotor) and S-0503 claim 12 (hovering locomotor), the computation a follower car makes when
its leader sends it after itself (claim 26), and the immediate new path of a walker that meets an
obstacle (S-0209 claim 7, walk locomotor). For the driving and walk locomotors, a running retry
delay ends when the unit's destination is changed to a different one or cleared, a delay started
earlier in the same tick included. For the walk locomotor it also ends when the walker is given
the destination it already holds, the same cell or the same object, by an order, a scatter or any
other rule, at every assignment that S-0209 claim 10 says is made, a delay started earlier in the
same tick included (Q-2896). So the first search for a new destination is not held back, and
no delay remains after a failure that clears the destination (claim 23). The offers of a refinery's
docking cell that S-0402 claim 10a gives a harvester are left to that claim (Open questions). For
the hovering locomotor, a person's stop, which clears the order-level destination, makes the timer
eligible (S-0503 claim 16), but every fresh search starts *R* again (below), so after a failed fresh
search or a failed gate the delay runs from that tick, even where the failure clears the order-level
destination (claim 23, rows 2 and 3, and scenario P19 for the failed gate). Whether a changed
order-level destination ends a running delay is not established, and this spec leaves it to S-0503
(Open questions). S-0503 owns the movement destination a hovering unit keeps and the searches made
for it. The delay after a follower car's computation is claim 26's, and for a car with the driving
or walk locomotor its new destination also ends a delay that ran before. For the driving and ship
locomotors:
    - A fresh search (no stored steps left) waits for the delay to be over. After a fresh search
      that fails, or whose gate fails (claim 6), the delay *R* runs from the tick of that search.
      For the driving locomotor it runs only until the destination is cleared: a failed gate and the
      failures of claim 23's second and third rows clear it in the same update (for a follower car,
      at the timing of claim 26), so no delay remains (above). After a fresh search that succeeds,
      no delay runs.
    - A re-path for a blocked next step (claim 22) also waits for the delay. After such a re-path
      the delay *R* starts again, whether the re-path succeeded or its search failed, except that
      for the driving locomotor a failed search that clears the destination ends it again (above).
      After a re-path whose gate failed, no delay runs.

    For the walk locomotor, when its searches are made is S-0209's (claims 7 and 10), and the delay
    after each is as follows. A fresh search, and the immediate new path of S-0209 claim 7, behave
    as a fresh search of the drive locomotor: no delay after a found path, *R* after a failed search
    unless the failure clears the destination, and no delay after a failed gate, which clears it
    (claim 23). After a re-path for a moving blocker the delay *R* starts again whether it found a
    path or its search failed, unless the failure clears the destination. A failed gate clears it
    there too, so no delay remains. A walker that keeps its destination after a failed search (claim
    23, third row) waits for *R* before it searches for that destination again, unless an
    assignment ends the delay first (above). For the hovering
    locomotor, a fresh search with no stored steps left waits for *R* to expire. Every such fresh
    search starts *R* again from its tick, whether it finds a path, fails in the search or fails at
    the gate. In particular, a successful fresh search does not remove the delay as it does for the
    driving and ship locomotors. The immediate first-check fresh searches of S-0503 claim 12 bypass
    a running timer but likewise start the full *R* from the actual search tick on success, search
    failure or gate failure. This covers first-check *cloaked blocker*, *blocked*, *friendly
    obstacle*, *enemy obstacle* and *stationary friendly*. Later ordinary retries without a heading
    point wait for that new timer, while another first-check refusal can bypass it again. An
    accepted person's stop clears the ordinary retry delay, so a hover retaining its movement
    destination is immediately eligible for a later ordinary search; the absence of an order-level
    destination does not otherwise waive the timer gate. Eligibility is not itself a search, and
    only an actual fresh search restarts *R* (S-0503 claim 16). S-0207 claim 19 gives the waits this
    produces when a hovering unit discards its path while its target is in range.
    - Backing: `measurable` (S-0207 scenario D21 run 4 for the retained-destination hovering retry
      wait after success, scenario P19 for its failed-search and failed-gate cases, P18's hover
      variant and S-0503 H9 for immediate-search timer restarts, S-0503 H12 and H15 for the two
      destinations and stop-time retry eligibility, scenarios P6, P7 run 5, P25(b) for the follower
      car's delay after its computation; for the walk locomotor, S-0209's scenarios C2 (c, d) and C5
      for the delay between re-paths and C19 for the immediate new path). That no delay remains
      after a walker's fresh search whose gate fails, which clears its destination, is `measurable`
      by P25(a), whose failed-gate control reads the timer at the end of that tick. After a walker's
      re-path whose gate fails it is `unmeasured`, as claim 23 rates that re-path. S-0417 scenario
      U10 part g gives its walker an order at its release that fails the gate, but its run does not
      compare the walk to the walker's next destination with the stock retry delay, so the walk
      without a wait is `unmeasured` there (S-0417 claim 17). That changing a walker's destination
      to a different cell ends a delay still running from an earlier tick is `measurable` by P25(e)
      run 2 and by S-0208 scenario W17 part m, whose scatter order to a walker waiting for a moving
      blocker, with a delay from its last re-path still running, is followed by a change of position
      in the tick of the order only when the new destination ends that delay (S-0208 claim 6). That
      giving a walker the cell it already holds as its destination, by an order in a later tick than
      the search that started the delay, ends that delay is `measurable` by P25(e) run 1. The same
      end for the same object, by a scatter or another rule, and for a delay started earlier in the
      same tick, is `unmeasured`: P25(e) gives only a move order to a cell, in a later tick. That
      clearing the destination, or changing a driving unit's, ends a delay still running from an
      earlier tick, a driving follower car's included (claim 26), is `unmeasured`: in U10 g no delay
      runs when the new destination is given, and no scenario here clears a unit's destination, or
      changes a driving unit's, while such a delay runs. P25(a, b) additionally reads the retry
      timer immediately after a walker's failed search/gate and a follower car's computation. It
      does not infer that timer from a later search decided by another reaction. S-0504 N11 tests
      the ship's successful fresh-search clearing, failed-search delay, moving-blocker restart and
      first-check timer exceptions.
    - Confidence: medium (for the walk and hovering locomotors: medium for the delays as stated, low
      for the exact tick of the hovering search after a wait, tested by S-0207 D21 run 4). No delay
      after a walker's failed gate: medium, until P25(a) is run (`unmeasured` after a re-path). The
      end of a running delay at a cleared or changed destination: medium for the driving and walk
      locomotors, a driving follower car's included. The end of a walker's running delay at the
      destination it already holds given again (Q-2896): medium. The carve-outs: low
      (`unmeasured`) for the ship locomotor and the locomotors other than the driving, walk and
      hovering locomotors waiting at a changed destination; a harvester's docking offers as S-0402
      claim 10a rates them, and the hovering locomotor's as S-0503 rates it

22. **Blocked next step (driving, ship and hovering locomotors).** For a unit with the
    driving, ship or hovering locomotor, when the entry
    outcome for the next stored step is *moving blocker*, the unit starts a blockage timer of *B*
    ticks (the blockage delay) the first time this happens for the current blockage. A blockage ends,
    and the timer starts afresh the next time, once the unit has taken a step again or its
    destination has been replaced; S-0209 claim 10 gives the same rule for a walker, for which only
    a step ends it. Each time the
    retry delay of claim 21 has passed, it computes a new path in search mode 1 while the blockage
    timer runs, and in search mode 2 once it has run out. A unit with a stored path but a next step
    of another outcome, other than *blocked*, reacts as S-0209 and the movement specs say (opening a
    gate, attacking an obstacle, waiting). For a driving or ship unit, at the first check of a track's first cell,
    an outcome *blocked* clears the stored path and makes a fresh search in mode 0 on the
    same tick, regardless of the path retry timer. A found path gets the repeated check
    of S-0207 claim 15 on that tick. This is the same immediate-search timing as that claim's
    first check of an obstacle or stationary friend, not the delayed fresh search used after
    a cloaked blocker. A repeated check with outcome *blocked* gives up the move. Checks
    of a curve's second cell follow S-0207 claim 15's separate table, not this first-cell rule.
    S-0504 claim 9 applies these driving first-check rules to ships: a cloaked blocker's
    fresh search waits for the retry timer, whereas a stationary friendly, friendly obstacle
    or enemy obstacle causes an immediate fresh search and same-tick repeated check of a
    found path. Ships share the moving-blocker timer and modes above, not hover's
    immediate cloaked-blocker search below.

    A hovering unit follows the same moving-blocker timer and search-mode rule above
    (S-0503 claim 12). Its first check with outcome *blocked* or *cloaked blocker* discards the
    stored path and makes a fresh mode-0 search immediately, regardless of the path retry
    timer; a found path receives a repeated check on that tick. Unlike driving, the hover's
    cloaked-blocker search does not wait for the timer, and a repeated *cloaked blocker* keeps
    the destination and waits rather than giving up the move. A repeated *blocked* gives up
    the move. S-0503 claim 12 owns the full hovering entry-outcome table.

    The walk locomotor's reactions to every outcome of its next step are S-0209's (claims 7 to 10).
    Its re-paths for a moving blocker (S-0209 claim 10) use search modes 1 and 2 as above, and the
    immediate new path it makes after an obstacle (S-0209 claim 7) is a search in mode 0.
    - Backing: `measurable` (scenarios P6, P14; S-0207 scenario D17 part d3 for the immediate
      fresh search on outcome *blocked*, also specified here as P18; S-0503 scenario H9 and
      P18's hover variant for hovering reactions; S-0504 N11 for ship moving-blocker
      timing and first-check timer exceptions; P25(d) for separate step-taken and
      destination-replaced endings of a blockage)
    - Confidence: medium (locomotors other than driving, ship, hovering and walking are not established).
      Low for what ends a blockage,
      pending the discriminating timer observations of P25(d)

23. **When no path is found.** What happens depends on why no path was found:

    | Case | Result |
    |---|---|
    | The gate of claim 6 fails | The path computation ends at once with no path. The unit does not leave its team, no behaviour is queued and its house gives it no new destination. A unit with the driving, ship or walk locomotor has its destination cleared, so it stops where it is. For the walk locomotor this holds after a fresh search and after a re-path for a moving blocker, and its movement in progress ends in the same movement update. For the driving and walk locomotors the clearing ends the retry delay, so none remains; for the ship locomotor the delay runs after a fresh search (claim 21). What its *move* behaviour does without a destination belongs to S-0215 (it falls back to rest). No other row of this claim applies. |
    | The search fails (including the immediate failure of claim 14), and the unit is more than 1 cell (Chebyshev) from the destination, or is not on a bridge deck while the destination is a bridge cell | The unit stops (its current motion ends) and the retry delay *R* starts, which the clearing of its destination below ends again for the driving and walk locomotors (claim 21). It gives up: it leaves its team, its destination is cleared, and it queues *guard* if it has a human-controlled owner or *area guard* if it has a computer-controlled owner. A unit with a computer-controlled owner in a skirmish is given the house-selected destination of claim 23a, provided that choice yields a cell; otherwise it gets none. These two computer-controlled rules (*area guard*, the new destination) describe the original only. In our engine the rule for a human-controlled owner holds for every player-house unit, those of an *AI house* included (glossary *AI house*): such a unit queues *guard* and its house gives it no destination. Units of the neutral and special houses keep the original's non-human-owner rule; those houses are not AI houses. This is Departure D5, with no switch. Our AI may give the same unit a destination as an ordinary order. |
    | The search fails in any other case (within 1 cell) | The unit stops and the retry delay *R* starts. It does not give up: it stays in its team and no behaviour is queued. A unit with the drive locomotor also has its destination cleared, so it ends up standing where it is without a destination and with no retry delay (claim 21). For locomotors other than driving, ship and hovering, the destination is kept, and the unit tries again once its behaviour starts it moving towards it (S-0215), which the retry delay holds back. A unit with the ship or hovering locomotor also has its destination cleared, like a unit with the driving locomotor, but its retry delay *R* keeps running (claim 21). |

    The same clearing ends every failed search of a unit with the driving, ship or hovering
    locomotor, so in rows 2 and 3 such a unit is left without a destination, unless its house has
    just given it a new one (row 2, in the original only). Being within the close-enough distance
    of the destination makes no difference to any row. For the walk locomotor, the stop of rows 2
    and 3 ends its movement in progress in the same movement update, whether or not the unit keeps
    its destination.

    For a hovering unit, stopping and clearing these points do not cancel a positive
    travel distance already computed on an arrival tick. In either failed-search
    distance case above, a surviving on-map hover can still take that cached final
    step along its old movement heading (S-0503 claim 7), including after a first-check
    refusal caused an immediate search that failed. Zeroing its hover speed and ρ
    during that refusal does not recompute the cached distance. A resulting refusal
    can suppress travel under S-0503 claims 7 and 12; search failure alone cannot.
    This permits no later travel ticks once the move has ended.

    Search failure is not the same as refusal of a hovering heading-point choice.
    S-0503 claim 15 owns the arrival consequences: a deferred search or a search failing
    during a non-refusing choice restores the vehicle-present flag at the actual position
    and produces a cell-arrival update. Failure in the initial arrival path update that
    clears the movement destination and heading point can instead prevent that choice:
    without a contacted object in the present cell, there is no restoration or update.
    Same-cell contact can still permit choice without a movement destination. These
    outcomes remain subject to that claim's hold and cached-travel qualifications.
    After an accepted stop, an equal retained movement destination and heading point can
    still complete with a non-refusing no-point choice and an update; unequal points
    follow the no-movement-destination arrival cases (S-0503 claim 16).
    - Backing: `measurable` (P3, P4, P7 runs 5, 5b and 6, P8, P11; P7 run 7b for the walk
      locomotor's fresh search in the gate row; P19 for ship and hovering units in row 3; S-0503 H3
      and H9 for cached arrival-tick travel, H12 and H15 for retained movement destinations and the
      arrival-search outcome distinctions). `unmeasured` for destination retention and restart in
      row 3 with locomotors other than driving, ship and hovering: no scenario tests those
      locomotors after this failure, and the rule remains a low-confidence assumption (Open
      questions). `unmeasured` for the walk locomotor's re-path in the gate row: that row needs a
      destination that becomes unreachable while a moving blocker holds the walker back, in the tick
      window before its re-path, and no scenario of this spec times that change. It is rated medium
      because the original clears the destination at a failed gate after a re-path as it does after
      a fresh search. `unmeasured` for the end of the walk locomotor's movement in progress in rows
      2 and 3: no scenario of this spec observes it, and S-0417 claim 17 relies on it. That the
      clearing ends the retry delay for the driving and walk locomotors rests on claim 21 and has
      its backing: P27's Failure run reads the timer after a driving harvester's failure in row 2,
      P25(a) after a walker's fresh search in the gate row, and the rest is `unmeasured`. The
      neutral/special distinction is our engine's scope rule from the glossary, not an additional
      oracle claim
    - Confidence: medium for driving, ship and hovering, and for the walk locomotor in the gate row.
      Low for the remaining locomotors in row 3, and for the end of the walk locomotor's movement in
      progress in rows 2 and 3. Medium for the clearing ending the retry delay, as claim 21 rates it

23a. **House-selected destination after give-up (Q-0238).** For both the neutral
    and special houses, row 2 assigns the house-selected destination on the same tick as
    the failed search, not on a later processing of *area guard*. This failure reaction
    queues only *area guard*: it neither queues *move* nor forces a behaviour to start.
    The ordinary start and processing rules remain S-0215's. The two houses use the
    same choice rules; neither has an automatic exemption just because it is a non-player
    house or lacks a base or start cell.

    The destination is a nearby-cell result around a centre supplied by the house's
    choice, not necessarily that centre itself. Table T5 specifies this final search.
    No result leaves the destination absent. S-0215 claim 38 owns the centre choice:
    infantry and vehicles use the central choice regardless of their weapons, with no
    outer-choice index draw. The recorded base centre is preferred to the recorded
    base-start cell; absent both, displacement uses the zero world point, not the unit's
    position. One ranged radius draw precedes one raw scenario-stream direction draw,
    even when the house has no base or start cell. That claim defines the radius bound
    and direction bits; rejection can make the ranged draw consume several raw values
    (S-0202). The final nearby-cell search takes no additional draw. S-0215 claim 38
    and F11–F12 define centre geometry, base-record maintenance and correction into
    the usable area, including the no-base case. The base-record refresh uses its own
    nearby search in T5; it is distinct from this final search and also takes no draw.
    Player-house units do not take either choice under Departure D5.
    - Backing: `measurable` (P26 for timing, queue, final-search parameters and presence
      of scenario draws; S-0215 claim 38 and B37 for centre preference and radius-then-direction
      draws and F11–F12's geometry and record maintenance; claim 25 and P16 run (u)
      for the nearby search's no-draw rule). Precedence and retention of a separately
      set base centre, and a start-assigned cell before any building refresh, inherit
      S-0215 claim 38's `unmeasured` backing: B37 supplies no repeatable ordinary
      setup or map action setting either record for a neutral or special house.
      A price-factor change alone not refreshing the records also inherits that claim's
      `unmeasured` backing; no scenario isolates it from a building event. These limits
      do not qualify the no-base correction, building refresh or draw rules.
      The player-house exclusion is the glossary's scope rule
    - Confidence: high for same-tick assignment, the queue and presence of a ranged draw.
      Medium for the final-search parameters

### Destination substitution

24. **Replacing a blocked destination.** After the gate of claim 6 has passed, and before the search
    (not for trains):
    - When the destination cell's entry outcome is *stationary friendly* and the unit's distance to
      the destination, *D*ᵤ (F6a), is larger than the close-enough distance, the original runs the
      nearby-cell search (claim 25) around the destination. *D*ᵤ is measured from the unit's position
      to the centre of the destination cell taken at world height 0, so the unit's own world height
      counts in full (`quirks.substitution_distance_flat_destination`). It adopts the cell found when
      two conditions hold. First, the distance between the centres of that cell and the destination
      cell (F6b) is smaller than *D*ᵤ. Second, a coarse estimate *E* of the route length from that
      cell to the destination is at most their Chebyshev distance plus 6 cells. *E* comes from the
      coarse search of claim 13 alone, run between the coarse end cells of claim 13b for the unit's
      own movement zone, with no exception for
      an infantry type with the jumpjet flag (which uses movement zone *infantry* only in the
      coarse search of a path computation, claim 13). The one exception is a unit of movement zone
      *subterranean* whose destination passes the further test of a cell named in claim 25: *E* is
      then found for movement zone *normal*.
      *E* = max(Chebyshev distance, 2*n* − 2 + legs) cells, where *n* is the number of sectors on the
      level-0 corridor found and the legs are given by F8, except in the first case of F8 (both cells
      on the same bridge), where *E* is the Chebyshev distance alone. When the coarse search fails,
      the cell is not adopted. This substitution allowance is 6 cells, not the 8-cell detour
      allowance of the obstructed approach-position choice in S-0307 claim 16.
    - **Threat in the estimate (Q-1975).** The coarse search of the estimate adds the threat term
      *T* (F5) to a sector's entry cost exactly as the coarse search of a path computation does:
      from the threat records of the unit's owner (claim 16a), with the unit's own threat
      coefficient *k* (the type's field, or exactly 1.0 for a member of a team marked to avoid
      threats), whenever *k* exceeds 0.00001 (F7). The subterranean case changes only the movement
      zone: the coefficient and the house are still the unit's own. *T* is 0 at level 0 (F5), so
      threat changes *E* only through the level-1 and level-2 corridors, which bound the level-0
      corridor (claim 13). A unit with a coefficient above the limit can therefore get a longer
      level-0 corridor from the found cell, so a larger *E*, and the found cell can be refused where
      a unit with coefficient 0 in the same place adopts it.
    - When the destination cell's outcome is *blocked* and the cell holds a building, the original
      always adopts the cell found by the nearby-cell search, with no distance or estimate test. When
      that search finds no cell, the original's new destination is undefined. Our engine keeps the
      destination unchanged (departure D3).
    - In both cases the substitute comes from the nearby-cell search of claim 25 around the
      destination cell, with the unit's own cell as reference cell, the reach region and the
      simplified movement zone of claim 25, and the height condition of claim 25. By that condition
      the destination counts 4 levels higher only when it is a bridge cell *and* the unit stands on a
      bridge deck. So for a unit on the ground ordered onto a bridge cell, the substitute is sought
      near the ground under the deck, although the search itself aims at the deck (claim 7), and it
      can never be given a deck cell as its substitute: it gets a bank cell or none
      (`quirks.substitution_centre_ground_level`). With that switch off, in this use of the
      nearby-cell search the centre's level includes the deck whenever the destination is a bridge
      cell, for every unit that is not an aircraft, as the search's destination pair does (claim 7):
      a unit on the ground ordered onto an occupied deck cell can then be given a free deck cell
      nearby. The switch does not change any other use of the search (claim 25, table T5). No simplified
      movement zone may enter a cell with a wall section (S-0211
      claim 10), so a wall cell is never adopted here, even for a unit whose own movement zone passes
      walls, except by a unit of locomotion class *winged*, for which claim 25 skips the class rule.
    - **Repeated substitution on high ground (Q-1870).** Once a substitute is adopted it is the
      unit's destination, and later path computations apply the ordinary substitution rules to that
      new destination: while the substitute stays free they substitute nothing. So a single order
      onto an occupied cell substitutes once while its substitute stays free, and the unit stops at
      the substitute. A substitute that becomes occupied later is an occupied destination in its
      own right, and the ordinary rules apply to it. Repeated substitution around the original
      occupied cell requires that a caller give the unit that cell again, as S-0402 claim 10 row 2
      does at each processing of *enter*. *D*ᵤ counts the unit's own world height in
      full, so for a unit whose *D*ᵤ stays above the close-enough distance wherever it stands next
      to the destination, each such path computation runs the nearby-cell search while the occupant
      stays. This holds, for one, whenever the unit's world height alone exceeds the close-enough
      distance, which a base raised by enough height levels of 104 leptons gives. For a unit
      whose occupancy record holds it, its own cell does not qualify in the search unless the
      unit is of locomotion class *winged* (claim 25), so each substitute is another cell. S-0208
      claim 13 gives the cases in which a unit is not marked in its own cell, and this inference
      does not cover them. A unit that comes
      within the close-enough distance gets its route onto the occupied cell instead, and S-0207
      claim 15 decides what it does there. S-0402 claim 10a gives the consequence for a harvester
      at an occupied docking cell. With `quirks.substitution_distance_flat_destination` off, the
      true distance decides, so height alone does not keep the substitution going.
    - Backing: `measurable` (scenario P7, run 3b for the height rule, run 10 for the destination's
      level on a bridge and so for `quirks.substitution_centre_ground_level`, run 9 for wall cells,
      runs 8 and 8b for the estimate test; scenario P30 part j for the threat term in the
      estimate). `unmeasured` for the subterranean destination's
      further-test gating and resulting estimate zone: P16 (t) tests the no-move caller instead,
      not this substitution caller, and so for the coefficient and house of that case's
      estimate, which are the same as in every other case. `unmeasured` for the movement zone of
      the estimate *E*
      of a jumpjet infantry type, which no scenario sets up, and for the legs of F8, which apply only
      when the found cell or the destination is a bridge cell and change *E* by a few cells, an
      effect no scenario of this spec isolates. `measurable` (S-0402 scenario H20, parts e and
      f) for the repeated substitution on high ground
    - Confidence: medium for the triggers, the distance test and the height rule, and for the
      repeated substitution on high ground (derived from these rules, not observed). High for the
      wall-cell consequence. Medium for the estimate away from bridges. High that the estimate's
      coarse search adds the threat term with the unit's coefficient and house, except in the
      subterranean case. Low for the legs of F8 and the coarse end cells on a bridge (claim 13b)

25. **The nearby-cell search.** Around a centre cell (*c*ₓ, *c*ᵧ), the original examines square
    rings of Chebyshev radius *r* = 0, 1, 2, … cells, up to a largest radius of 31 cells, or of
    *w* + *h* − 1 cells when *w* + *h* is less than 32, where *w* and *h* are the map width and map
    height of the map size (glossary), added as plain numbers in the map size's own units. The
    cells of ring *r* have a fixed **scan order**. A cell (*x*, *y*) of the ring is a *row cell*
    when |*y* − *c*ᵧ| = *r*, and a *column cell* otherwise (then |*x* − *c*ₓ| = *r*). Every row cell
    comes before every column cell. Row cells are ordered by *x*, and on equal *x* by *y*. Column
    cells are ordered by *y*, and on equal *y* by *x*. Ring 0 is the centre cell, which appears twice
    in the scan order (`quirks.nearby_centre_twice`): if it qualifies, it is collected twice and
    counts twice towards the limit of 24 collected cells.

    The uses in table T5 examine single cells. Claim 25a adds the aircraft landing
    footprint search. S-0402 claim 24 owns the additional
    slave-miner deployment-site use: candidates are rectangular footprints, not single
    cells. It supplies the footprint qualification, caller settings and destination shift
    after selection; the ring order, visibility preference, collection limit and
    reference-cell distance tie rule below still apply. Do not apply T5's single-cell
    qualification literally to that use.

    Each single-cell use of the search fixes a centre, a locomotion class, a
    movement zone and whether the use is in **deck mode**, and optionally a reach region, the height
    condition, the further test, a no-overlay condition and a reference cell. A use may also
    exclude bridge cells. Deck mode has the three effects given below:
    a bridge cell's reach region is read for its deck, a bridge-cell centre's level includes the
    deck (claim 24's switch aside, below), and the ring cut-off of the collection is the first ring that contains any collected
    cell rather than the first that contains one passing the visibility test. Table T5
    lists the single-cell users and what each use fixes. Its cell/object order rows apply only
    when S-0214 claim 11 selects a search: that claim's direct-destination cases bypass
    this search, even when the clicked cell is occupied. A free mover's *no-move*
    search on a cell or object has no reach-region restriction, including inside the
    usable area; for object clicks use T5's stated free-mover conditions, which do not
    depend on an ion storm. Rally-point clicks use their own search row, not the
    move-order direct-destination conditions.
    The **simplified movement zone** of claim 24 maps crushing and destroying variants to their base
    zone: *normal*, *crusher*, *destroyer* and *crusher-all* map to *normal*; *amphibious*,
    *amphibious-crusher* and *amphibious-destroyer* map to *amphibious*; *infantry* and
    *infantry-destroyer* map to *infantry*; the others map to themselves.

    **T5. Single-cell users of the nearby-cell search.** "Search layer" is the ordered unit's own (claim 7).
    "Own zone" is the movement zone of the ordered unit's type.

    | User | Centre | Locomotion class | Movement zone | Reach region | In deck mode when | Height condition | Further test | Reference cell |
    |---|---|---|---|---|---|---|---|---|
    | Destination substitution (claim 24) | the destination | the unit's | the simplified movement zone | that of the unit's cell, read for its deck when the unit is on a bridge deck (claim 4), for the simplified zone | the unit is on a bridge deck | yes | for a unit of movement zone *subterranean*, as below | the unit's cell |
    | Failed ordinary approach (S-0307 claim 17), ground infantry or vehicle on terrain without bridges, excluding jumpjet and special crushing, deploying or balloon-hover searches | the failure centre of S-0307 claim 17, including after candidate trials | the actor's | own zone | that of the actor's ground cell for its own zone | never | yes | no | none |
    | jumpjet movement-point adjustment, non-balloon-hover vehicle (S-0502 claims 16 and 23) | the ordered point's cell | *hover* | *fly* | none | never | no | no | none |
    | jumpjet movement-point adjustment, infantry or balloon-hover vehicle (S-0502 claims 16 and 23) | the ordered point's cell | *hover* | *fly* | none | the centre carries a high bridge | no | no | none |
    | jumpjet stopped-movement reroute, including failed landing and effective order-level destination clears (S-0502 claims 17, 23 and 24) | the unit's current cell | the type's own | own zone, not simplified | none | never | no | no | none |
    | Animated landing-first undeploy (S-0502 claim 23) | the vehicle's current cell | the type's own, *winged* counted as *track* | own zone, not simplified | that of the centre cell for that zone, read for its deck when the vehicle is on a bridge deck; none when that read gives no reach region | the vehicle is on a bridge deck | no | no | none |
    | Teleporting landing of a non-infantry object (S-0207 claim 20 b) | the destination cell | the object's | own zone, not simplified | that of the destination cell for its own zone, read for its deck in deck mode (claim 4); none when that read gives no reach region | the destination is a bridge cell and the object's world height is more than 312 leptons above surface height *H* there (exact integer comparison) | no | no | none |
    | Infantry teleport object-destination fallback (S-0208 claim 19), after landing-point refusal | the cell of the destination object's heading point (its next movement point, or its position if it has none) | the infantry type's | own zone, not simplified | that of the infantry unit's heading-point cell for its own zone, read for its deck when the unit is on a bridge deck; its teleporting locomotor's heading point is its position | the centre carries a high bridge | yes | no | none |
    | Starting-building restoration displacement (S-0524 claim 13) | the displaced occupant's old cell | the occupant type's | *normal* | none | never | no | no | the occupant's old cell |
    | Trigger crate placement (S-0524 claim 4) | the waypoint-slot cell | *float* when the centre's land type is *water*, *track* otherwise | *normal* | none | never | no | no | none |
    | Random crate placement (S-0520 claim 3) | the candidate cell supplied by S-0520 claim 3 | *float* when the centre's land type is *water*, *track* otherwise | *normal* | none | never | no | no | none |
    | Unit crate placement fallback (S-0520 claim 13) | the crate cell | the awarded unit type's | *normal* | none | never | no | no | none |
    | Transport shore approach (S-0507 claim 8) | the transport's cell | *wheel* | *normal* | none | never | no | no | none |
    | Vehicle passenger exit (S-0507 claim 10) | the serving neighbour chosen by the exit rounds | the passenger's | *normal* | none | never | no | no | none |
    | Bunker exit (S-0606 claim 12) | the bunker's top-left foundation cell + (−1, +1) cells | the vehicle's | *normal* | none | never | no | no | none |
    | Paradrop target adjustment (S-0522 claim 1) | the target cell | *foot* | *normal* | none | never | no | no | none |
    | Air-strike aircraft placement (S-0519 claim 17) | the strike's arrival cell | *winged* | *normal* | none | never | no | no | none |
    | Parasite exit (S-0614 claim 13) | the host's cell | the host's, *winged* counted as *track* | the host's own | that of the centre cell for that zone, read for its deck in deck mode | the host is on a bridge deck | no | no | none |
    | Scatter (S-0208 claim 13) | as S-0208 claim 13 says | the unit's | *normal* | none | the unit is on a bridge deck | yes | no | none |
    | Move order onto a cell, or no-move order by a unit that is not a free mover (S-0214 claim 11), when that claim uses the search, except the next row | the clicked cell | the unit's | own zone, *subterranean* counted as *normal*, and *fly* counted as *infantry* while an ion storm (a global effect, S-0104) is active | that of the unit's cell, read for its deck when the unit is on a bridge deck (claim 4), for that zone | the centre is a bridge cell | yes | no | none |
    | No-move order onto a cell (S-0214 claim 11) by a free mover (S-0214 Terms: a unit that is in the air, is of movement zone *subterranean* whatever its state, is of a jumpjet infantry type with no ion storm active, or is an aircraft) | the clicked cell | the unit's | as in the row above | none | the centre is a bridge cell | yes | for a unit of movement zone *subterranean* that is below the ground surface when the search is made (below) | none |
    | Move order onto an object (S-0214 claim 11), when that claim uses the search, except the next row | the object's cell | the unit's | own zone, *subterranean* counted as *normal* and *fly* as *infantry*, whether or not an ion storm is active | that of the unit's cell, read for its deck when the unit is on a bridge deck (claim 4), for that zone | the centre is a bridge cell | yes | no | none |
    | No-move order onto an object (S-0214 claim 11) by a unit that is in the air, is of movement zone *subterranean* whatever its state, is of a jumpjet infantry type (whether or not an ion storm is active) or is an aircraft | the object's cell | the unit's | as in the row above | none | the centre is a bridge cell | yes | as in the no-move row for a cell | none |
    | Rally point (S-0213 claim 20) | the clicked cell | *foot*; *amphibious* for a naval factory; *winged* for an aircraft factory | *normal*; *amphibious-crusher* for a naval factory; *fly* for an aircraft factory | that of the building's own cell for that zone | the centre is a bridge cell | no | no | none |
    | House base-record refresh (S-0215 claim 38) | the weighted mean cell of S-0215 F12 | *foot* | *normal* | none | always | no | no | none |
    | House-selected destination after give-up (claim 23a), original non-human-owner rule retained for neutral and special houses | the corrected centre supplied by S-0215 claim 38 and F11–F12 | *track* | *normal* | that of the unit's ground cell for *normal* | never | no | no | none |
    | Aircraft factory edge entry without a rally point (S-0417 claim 14) | the cell of the factory's reference point | the aircraft's, *winged* counted as *track* | the aircraft's own | that of the centre cell for that zone, read for its deck in deck mode | the aircraft is on a bridge deck | no | no | none |
    | Harvester queueing (S-0402 claim 9) | the queueing cell of S-0402 | *wheel* | *normal* | none | never | no | no | the refinery's top-left cell |
    | Idle harvester step-off (S-0402 claim 13) | the cell of the building's reference point | the harvester's, *winged* counted as *track* | the harvester's own | that of the centre cell for that zone, read for its deck in deck mode | the harvester is on a bridge deck | no | no | none |
    | Waiting absorber approach (S-0605 claim 4) | the cell of the building's reference point | the actor's, *winged* counted as *track* | the actor's own | that of the centre cell for that zone, read for its deck in deck mode | the actor is on a bridge deck | no | no | none |
    | Free unit, first search (S-0402 claim 18) | the building's top-left cell | *wheel* | the free unit type's own | that of the centre cell's ground for that zone | never | yes | no | the building's top-left cell |
    | Free unit, second search (S-0402 claim 18) | as in the preceding row | *wheel* | as in the preceding row | as in the preceding row | never | yes | no | the building's top-left cell |

    The restoration-displacement, trigger-crate, random-crate, unit-crate fallback,
    transport-shore, vehicle-passenger-exit, bunker-exit, paradrop and air-strike rows
    allow bridge cells and ask for no no-overlay condition. S-0606 claim 12 owns
    bunker-exit release and its outcome when that search finds no cell: the vehicle's
    destination is the placeholder map cell (0, 0), as for the transport-shore row
    (S-0507 claim 8). S-0606 gives this outcome `unmeasured` backing and low confidence,
    unlike the bunker-exit caller settings in the claim on search parameters. For trigger and random crates, selection
    is not placement: a selected cell outside the height-aware usable area or holding
    any overlay is refused without another nearby search around that centre. S-0524
    claim 4 owns trigger placement; S-0520 claim 3 owns random placement's subsequent
    candidate attempts. The unit-crate fallback search follows a separate direct
    placement attempt at the crate cell, accepting only the awarded unit's *clear*
    entry outcome (S-0520 claim 13). That attempt is not ring-zero qualification:
    it does not require the search's empty occupancy record. The fallback search's
    selected cell receives the same direct placement test once; refusal causes no
    further search or placement attempt. S-0520 claim 13 owns those placement tests
    and the money outcome when the fallback search or placement fails.
    The paradrop row is used only when the target is a water-role tile (S-0522 claim 1).
    Its result replaces the target only when there is a result and it is not itself a
    water-role tile; otherwise the original target remains. This post-selection refusal
    does not cause another search.

    Restoration displacement uses a 1 × 1-cell footprint. The old-cell reference
    selects by reference distance among preferred collected cells, not globally over
    all nearby cells. S-0524 claim 13 owns reverse lifting-order placement and its
    ordinary placement fall-back: earlier re-placed occupants affect later searches.
    Placement uses the selected cell's terrain-surface centre without a bridge lift,
    facing 0. With no result, placement is nevertheless attempted at the blank
    stand-in at cell (0, 0), under S-0009 claim 44; failure of that placement, or of
    placement at a found cell, removes the occupant. This does not make the stand-in
    a qualifying search result. Failed building placement's return of lifted occupants
    is a separate caller rule of S-0524 claim 13, not this nearby search.

    Both transport rows use a 1 × 1-cell footprint. S-0507 claims 8 and 10 own their
    unchecked no-result handling: although the search finds no cell, each caller uses
    the placeholder at map cell (0, 0). Shore approach assigns it as destination and
    waits 10 ticks, without trying an unloading neighbour in that processing. Vehicle
    passenger exit attempts placement there, not at the serving neighbour's centre.
    Its later movement destination remains the exit-round cell, even when a successful
    search selected a different placement cell. These caller rules do not make (0, 0)
    a qualifying result or prove reachability or placement success.

    The parasite-exit row gives the original host-mobility search. S-0614 claim 13 owns
    when that search is needed and its `quirks.parasite_exit_search_host_mobility`
    alternative, which uses the parasite's class and zone instead.

    All four jumpjet rows use a 1 × 1-cell footprint, no further test, no no-overlay
    condition and no reference cell. Only the non-balloon-hover vehicle movement-point
    row excludes bridge cells; the other three allow them. The animated-undeploy row
    selects the initial destination before the subsequent movement-point adjustment.
    S-0502 claim 23 supplies the settings and S-0517 claim 6 owns its assignment.
    Searches retain the unit's own occupancy: a nonempty searched record rejects the
    candidate even when its only mark belongs to that unit, except for the *winged*
    class's omission of occupancy checks below. Grounded self-occupancy is not lifted
    for movement-point adjustment. Infantry standing-position adjustment happens after
    cell selection and supplies no exemption from this whole-record test.
    The stopped-movement row is not limited to failed landing or player stop: it also
    applies to effective order-level destination clears during deploy or behaviour
    processing while the jumpjet retains a movement commitment. S-0502 claim 24 owns
    which assignments are effective, including suppression of redundant assignments.
    Without a movement commitment there is no reroute. These settings have high
    confidence and are `measurable` (S-0502 scenarios J16, J19 and J22).

    The failed-approach row, the non-balloon-hover jumpjet movement-point row and both
    free-unit searches exclude bridge cells. The teleporting
    landing row and the infantry teleport fallback row allow them and ask for no no-overlay condition. Only the first free-unit search requires
    the **no-overlay condition**: the cell must hold no overlay of any kind, resources included.
    No other row of T5 asks for that condition.

    In the non-infantry teleporting landing row, a destination in no reach region removes
    only the reach-region restriction. It does not fail the search automatically or require
    candidates to lie in no reach region themselves. Usable-area, class and occupancy
    tests still apply, and collection order, visibility preference and the landing-point
    fixation tick select the result as below. Thus an unenterable water destination can
    yield a free bank cell; no qualifying cell within the radius gives no result.

    A cell **qualifies** when all of these hold:
    - it is inside the usable area,
    - when the user asks for the no-overlay condition, it holds no overlay of any kind,
    - when the user excludes bridge cells, it is not a bridge cell,
    - when the user gives a reach region, the cell lies in it (claim 4). A bridge cell's reach
      region is read for its deck when the use is in deck mode, and for its ground otherwise,
    - it is enterable by class (S-0211 claim 10) for the user's locomotion class with the user's
      movement zone, a bridge cell being judged as entered on its deck, so that its land type is not
      consulted,
    - its occupancy record (S-0209) is empty as a whole: it records no standing position as taken,
      and no vehicle, no building and no object as present. The record read is the
      deck's for a bridge cell and the ground's for every other cell. So a cell holding any infantry unit,
      vehicle or building on the cell layer whose record is read, or an aircraft on the ground there, never qualifies,
    - when the user asks for the **height condition**: the cell's level and the centre's level
      differ by at most 1 height level. A cell's level is its height level, plus 4 when it is a
      bridge cell. The centre's level is its height level, plus 4 only when the centre is a bridge
      cell and the use is in deck mode (in claim 24's use, `quirks.substitution_centre_ground_level`
      off changes this to plus 4 whenever the centre is a bridge cell; no other use is changed).
      Bridge cells may qualify except in the ordinary approach-failure, non-balloon-hover
      jumpjet movement-point and both free-unit rows of T5, which exclude them
      independently of locomotion class,
    - when the user asks for it, it passes the *further test* of a cell, which is the burrowing test
      of S-0010 claim 17: the test of whether a tunnelling unit may dig in or come up on the cell.
      S-0010 claims 17 and 25 give the whole test, among its conditions the tileset's
      allow-burrowing flag, a flat cell, and the bridge cells and bridge-body marks (the band of a
      low span and the cells of a destroyed high bridge) that refuse it. This spec adds no condition
      of its own. Two users ask for it, in different ways:
      - Claim 24 asks for it only for a unit of movement zone *subterranean*, and only when the
        destination itself passes the same test. The destination's own result decides both: when
        the destination passes, every candidate cell must pass too and the estimate *E* of claim 24 uses movement zone
        *normal*; when it fails, neither applies.
      - The no-move rows of table T5 ask for it when the unit is of movement zone *subterranean* and
        is below the ground surface at the moment the search is made: its world height is less than
        the world height of the ground at its position, counting a bridge deck's height when the
        unit is on one. That is the case while it travels underground and while it is partway into
        digging in or coming up. Here the clicked cell's own result does not decide whether the test applies: every candidate cell,
        the centre included, must pass, whether or not the clicked cell passes. A subterranean unit
        on the surface, and every other free mover, gets no further test.

    For locomotion class *winged*, only the usable-area condition, any no-overlay condition,
    the height condition, the further test and any user's bridge exclusion apply: the reach region, the class rule and the occupancy record are not consulted. In
    claim 24 the centre is the destination, whose occupant gave the outcome that started the search,
    so it normally does not qualify, and the double entry of the centre matters only where the
    centre can be empty.

    **Collected sequence and result.** The ordered candidate sequence consists of qualifying
    cells in rings of increasing Chebyshev radius up to the largest radius, in the scan order
    defined above within each ring. The **collected cells** form the prefix ending at the earlier
    of its 24th cell, if reached, or the end of the first ring containing a qualifying cell that
    passes the **visibility test**. In deck mode that stopping ring is instead the first ring
    containing any qualifying cell. If the 24-cell limit falls within a ring, only its prefix is
    included; otherwise the entire stopping ring is included and no later ring contributes.

    The visibility test is S-0208's screen-cover check (its Terms), including its arithmetic and
    the high-bridge raise of cells in front. It judges the candidate at its own ground height
    level, including for a bridge cell in deck mode: a passing cell's centre is not hidden on
    screen by higher terrain in front. If any collected cells pass visibility, only those cells
    are preferred; otherwise all collected cells are preferred, retaining sequence order.

    With a reference cell, the result is the preferred cell with minimum table-square-root
    distance *r*(Δx² + Δy²) in cells (S-0204), using exact integer cell differences and sum of
    squares. At these distances the table root orders cells exactly as true distance does.
    Ties go to the earliest position in the collected sequence. Without a reference cell, the
    result is the preferred cell at zero-based position *N* mod *n*, in exact integer arithmetic,
    where *n* is the preferred-sequence length. A centre collected twice occupies two positions.
    *N* is the tick number (S-0201) applicable to that search: the scatter-request tick for
    scatter, click-handling tick for move orders and rally points (S-0213 claim 20 and S-0214
    claim 11), failure tick for ordinary approach failure, landing-point fixation tick for
    teleporting landing (S-0207 claim 20 b) and infantry teleport fallback (S-0208 claim 19),
    action tick for trigger crate placement (S-0524 claim 4), placement-attempt tick for
    random crate placement (S-0520 claim 3), pickup tick for unit-crate placement fallback
    (S-0520 claim 13), unload-processing tick for transport shore approach and vehicle
    passenger exit (S-0507 claims 8 and 10) and for bunker exit (S-0606 claim 12), launch tick for paradrop
    target adjustment (S-0522 claim 1), start tick for air-strike aircraft placement
    (S-0519 claim 17), exit-search tick for parasite exit (S-0614 claim 13), processing tick for idle
    harvester step-off (S-0402 claim 13), *enter*-processing tick for the waiting absorber approach
    (S-0605 claim 4), release tick for
    aircraft factory edge entry without a rally point (S-0417 claim 14), and failed-search
    tick for the house-selected destination after give-up (claim 23a). The base-record
    refresh uses the tick of its building entry, removal or owner-change event (S-0215
    claim 38), not a subsequent path-failure tick.

    S-0307 claim 16 owns qualification of obstructed approach positions before failure, including
    its two alternative coarse-estimate tests; these are not nearby-cell-search qualification
    and do not change the failure row's parameters. If no cell qualifies, there is no result.
    The search takes no random-stream draw.
    - Backing: `measurable` for the ring rule, the scan order and the distance choice (scenario P7,
      runs 1, 2, 4, 8 and 8b), for the occupancy condition (P7 runs 1, 2, 8 and 8b, P16), for the
      class rule with walls (P7 run 9, P16), for the harvester queueing and idle step-off
      parameters (S-0402 scenario H11, part e for a proposed no-result search), for the waiting-absorber approach parameters, which are the step-off row's settings (S-0605 scenario A2c, search variants (i) to (iii)), for the
      aircraft factory edge-entry parameters on ordinary ground (S-0417 U13 c), for the
      parasite-exit host-mobility parameters (S-0614 scenario P4), for trigger-crate
      search settings and post-selection refusal (S-0524 scenario C4), for restoration
      displacement settings, reference choice and unchecked no-result placement
      (S-0524 scenario C14), for random-crate
      search settings and post-selection refusal (S-0520 scenario CR1), for unit-crate
      fallback settings, the separate direct-placement test and the single
      post-selection placement attempt (S-0520 scenario CR9), for paradrop
      target-adjustment settings and retention of the original target after refusal
      (S-0522 scenario P1), for air-strike placement settings and the shared placement
      cell selected at the start tick (S-0519 scenario T17), for the
      final nearby search after give-up (P26), for base-record-refresh parameters
      (S-0215 B37, with P26's refresh comparison), and for both free-unit searches and the no-overlay
      condition (S-0402 scenario H13 parts b and c), for the rectangular deployment-site
      use delegated to S-0402 claim 24 (its scenario H18), for the ordinary approach-failure parameters
      and centre after candidate trials, including close-range failure (S-0307 scenarios RG12,
      RG17 and RG23 f), for the infantry teleport fallback parameters (S-0208 scenario W11), for the teleporting
      landing parameters on flat ground (S-0207 scenario D22 run 3), for its omitted
      reach-region restriction when the destination has no reach region (S-0207 D22 run 10),
      and for the choice without a reference cell (P16, runs
      (s), (u) and (r), which also test the parameters of table T5 for a scatter, a move order onto
      an unreachable cell and a rally point), and for the further test of the no-move row for a
      cell, with the state that decides it (P16 run (t), test data). The no-move row for a unit
      that is not a free mover has the move-order row's parameters and is tested by run (u2) below.
      `unmeasured` for the
      transport shore approach and its unchecked no-result handling (S-0507 claim 8):
      S-0507 T9 has not yet established its ordinary-order/map witness. Vehicle-passenger
      exit settings and tick-modulo choice are `measurable` (S-0507 T13); its unchecked
      no-result placement remains `unmeasured`, because no ordinary-input witness
      separates it from the preceding exit-entry refusals (S-0507 claim 10).
      `unmeasured` for the bunker-exit parameters (S-0606 claim 12): no player-input route to a bunkered vehicle's
      *unload* has been established. Confidence in those caller settings is high, but their
      reachability is low. `unmeasured` for the other
      parameters of the no-move rows and for the rows of orders onto an object: they differ from the
      move-order row only in parameters that need a flying or jumpjet unit next to occupied cells.
      P22(a) tests the teleporting landing's strict deck-mode threshold on a high bridge.
      `unmeasured` for the 24-cell limit and the stop rule in deck mode. Both differ from the plain
      ring rule only when every qualifying cell of the inner rings fails the visibility test, which
      needs terrain built to hide whole rings (S-0208's scenario W7, part f, tests the check itself
      on uneven ground). `unmeasured` for the double entry of the centre: in claim 24 the centre normally does
      not qualify, and a scatter's centre qualifies only in the case S-0208 claim 13 names (the
      empty destination cell of a walking unit), while a scatter without a threat point is accepted
      only while the unit's locomotor is not moving (S-0208 claim 11), so that case cannot be set up
      at will. In the uses of table T5 for orders, a qualifying centre on flat ground ends the
      collection with ring 0, so the result is the centre at either position.
      P22(b, c) tests the failure row's and second free-unit search's bridge exclusions on
      raised terrain, where the height condition alone allows the candidate. The first free-unit
      search's independent bridge exclusion remains `unmeasured`: a bridge overlay already
      fails its no-overlay condition, so these two refusals cannot be distinguished in that caller.
      P21(b) tests the winged class's omitted occupancy and reach-region checks. Its omitted class check stays `unmeasured`:
      the winged class's fixed land multipliers give no independent zero-multiplier control.
      P21(a) tests the 31/32-cell radius boundary and the no-result case. The smaller-map
      radius bound stays `unmeasured`: no fixture here distinguishes it from usable-area refusal.
      The aircraft factory row's deck-mode case remains `unmeasured`: U13 c does not
      establish an aircraft entering from the edge while marked as on a bridge deck.
      Visibility preference
      across rings remains `unmeasured`: no proposed fixture hides whole qualifying inner rings
      while controlling the visibility of outer-ring cells.
      The no-random-draw rule is `measurable` through P16 run (u)'s random-state check
    - Confidence: high for the occupancy condition and the class rule. Medium for the ring rule, the
      scan order, the 24-cell limit, the distance choice, the choice without a reference cell and
      the parameters of table T5. Medium that the visibility test is S-0208's screen-cover check
      and that the further test is S-0010's burrowing test, each applied as stated; their content
      carries the confidence of those specs. High for the condition under which the no-move rows of
      T5 ask for the further test, and that they test no destination first

25a. **Aircraft landing footprints and candidate ties (Q-0634).** The building-centred
    landing search of S-0501 claim 41 uses claim 25's rings, scan order, collection limit
    and visibility preference. A candidate (*x*, *y*) is the top-left cell of the nine-cell
    rectangle (*x* + *i*, *y* + *j*), for integer *i* and *j* from 0 through 2, inclusive.
    These are exact integer additions in cells, not a square centred on the candidate.

    The top-left candidate must pass the height-aware usable-area test. Each of the nine
    cells must pass all of these tests:
    - its ground reach region for movement zone *normal* must match the building cell's
      ground reach region, unless that reference has no reach region, which removes this
      restriction rather than requiring cells of no reach region,
    - its occupancy record must be empty as a whole, with no ignored infantry or vehicle
      positions; read the deck record on a bridge cell and the ground record otherwise,
    - it must be enterable by locomotion class *foot* and movement zone *normal* under
      S-0211 claim 10, treating a bridge cell as entered on its deck. A wall overlay
      refuses the cell, and a non-bridge cell needs a nonzero foot land multiplier.

    There is no additional usable-area test on the other eight cells, no common-height or
    slope requirement, no burrowing test, no blanket overlay exclusion and no extra
    building-placement or terrain-object test beyond those class, region and occupancy
    tests. Bridges are allowed; no deck mode means reach-region comparisons use ground
    even though bridge occupancy and class checks use the deck. Non-wall overlays do not
    independently refuse a cell. Visibility is tested at the top-left candidate's ground
    level as in claim 25, not at the middle or at every cell of the rectangle.

    The result is the selected top-left cell (*x*, *y*) itself: displacement is (0, 0)
    cells. It is not shifted to (*x* + 1, *y* + 1), unlike S-0402 claim 24's deployment
    site. The 3 × 3 search has no reference cell and does not rank candidates by distance
    to the aircraft or building. With *n* preferred cells, its result is position *N* mod
    *n* in claim 25's preferred sequence, where *N* is the landing-search tick number and
    the remainder is exact integer arithmetic. Equal geometric distances introduce no
    separate tie test. For the aircraft's 1 × 1 own-cell replacement search, S-0501 claim
    41 supplies a reference cell: claim 25's minimum table-root distance applies, with
    equal distances resolved by the earliest collected position, not tick modulo. The
    1 × 1 other-object fallback search has no reference and uses tick modulo instead.
    These 1 × 1 searches also return the selected cell unchanged. When no candidate
    qualifies, the no-result value is map cell coordinates (0, 0), not a successful
    selection of a usable cell. S-0501 claim 41 owns the unchecked use of that value by
    the own-cell replacement caller and its quirk alternative. None of these searches
    draws from a random stream.

    S-0501 claims 34, 37 and 41 already answer the caller settings, no-building fallback
    and edge-selection parts of Q-0634. Comparing buildings or equally distant fallback
    objects belongs to S-0501 claim 41, not this footprint-selection rule.
    - Backing: `measurable` (P28; S-0501 scenario AC25 for the unchecked no-result value)
    - Confidence: high for the nine-cell tests, unchanged result, no-result value and absence of draws.
      Medium for the shared collection, visibility and tie rules, as in claim 25

### Linked vehicles

26. **Follower cars are sent after the leader.** When a leading vehicle that is not itself a
follower car obtains a path with at least one step, each car of its car chain is given, in that
tick, a destination: the first car the cell one step back from the leader's destination, against the
direction of the last step of the whole path found (not only of its stored part), and each later car
the cell one step further back in the same direction. A car whose current destination already lies
in that cell is left alone. Any other car queues *move* with that destination and computes its path
at once under this spec, with its own reachability gate and search. It does so even while its own
retry delay runs (an exception of claim 21). For a car with the driving or walk locomotor the new
destination also ends that delay (claim 21), and afterwards its retry delay is none after a found
path or a failed gate, and the delay *R* from that tick after a failed search, until its destination
is cleared (claim 21). For a car of any other locomotor, afterwards its retry delay is none after a
found path, the delay *R* from that tick after a failed search, and unchanged after a failed gate,
and whether its new destination ends a delay that ran before is not established (Open questions).
What else follows a failure is claim 23, except that when exactly a driving or walking car's
destination is cleared after such a failure is not established (Open questions).
    - Backing: `measurable` only with test data (scenario P9, follower runs 1 and 2). Stock data is
      not known to link cars. That a running delay does not hold the car back is `unmeasured` here:
      a driving car's own failed order clears its destination, which ends the delay, so no simple
      setup leaves one running when the leader sends it, and no scenario here links cars of another
      locomotor. For a walking car, the rules stated for the driving car follow from claim 21 and
      are `unmeasured` here for the same reason. P25(b) tests a driving car's retry delay
      immediately after success, failed search and failed gate, without relying on its
      still-uncertain later failure handling.
    - Confidence: medium (low for any car's retry delay afterwards; the rest of a walking car's
      rules, and a running delay not holding a car back, medium, `unmeasured`)

### Entry outcomes for a vehicle mover

27. **The rows this spec relies on.** For a ground vehicle asking to enter a cell, the objects on
    the cell's search layer give these outcomes. "Allied" includes the vehicle's own house.
    S-0209 claim 5 gives the full table and owns these outcomes. This claim summarises it, and where
    the two differ, S-0209 claim 5 is meant.
    - An allied vehicle, infantry unit or aircraft that does not count as moving (it has no
      destination, is not turning and its locomotor is not moving) gives *stationary friendly*.
    - An allied unit that counts as moving gives *blocked* in one close case. The close case needs
      all three of these: with *f*ₒ the occupant's body
      facing and *f*ₘ the mover's, the 8-direction of the facing (*f*ₒ + 32767) mod 65536 equals the
      8-direction of *f*ₘ (the two face roughly opposite ways, `quirks.opposite_facing_reversal`), the spatial distance between their
      positions (F6c) is at most 511 leptons, and the 8-direction of the direction of the occupant
      from the mover (S-0204) equals the 8-direction of the mover's body facing (the occupant lies
      ahead).
    - Outside the close case, an allied unit that counts as moving gives *moving blocker* when it is
      a vehicle (or, with low confidence, an aircraft) that is between steps, or any unit that is at a chain point (both terms
      of S-0209). Any other allied unit that counts as moving, among them every moving allied
      infantry unit and a vehicle in the middle of a step that is not at a chain point, gives no outcome of its own. The outcome
      then comes from the cell's other objects and its occupancy record (S-0209 claim 5, part 5e):
      for example *moving blocker* through the cell's vehicle-present flag or through an allied
      infantry house, and *clear* when nothing gives a value.
    - An allied building not counted as moving gives *blocked* only when no earlier
      exception in S-0209 claim 5's occupant table applies. Those exceptions are not
      all mission targets: they include buildings invisible in play, non-blocking repair
      or bunker cells, the bib edge and inactive firestorm walls. Gates, contact,
      absorbing and grinding buildings and behaviour relations follow that table's
      rows VP3 and VP5 to VP18, with their stated conditions and precedence.
    - For a wall section the vehicle does not crush, S-0209 rows VO4 to VO6 give
      *blocked* when the vehicle has no primary weapon, or its primary weapon's warhead
      can neither damage walls nor damage wood while the overlay armour is wood.
      Either wall-damaging ability or that wood case instead gives *friendly obstacle*
      for an allied wall section and *enemy obstacle* for an enemy one. A vehicle crushes a wall section
      when the wall's overlay type is crushable and the vehicle is a crusher (by its type's crusher
      flag or by the crusher veterancy ability, S-0210 claim 1), or when its movement zone is
      *crusher-all*, whatever the overlay type. It then gets *friendly obstacle* for an allied wall
      section, and an enemy wall section adds no outcome.
    - Occupant overrides follow S-0209 claim 5, using the listing order of its claim 13.
      A *blocked* overlay or occupant before the first overriding occupant defeats that
      override. Otherwise the override decides: less severe earlier values and all later
      occupants, even a later *blocked*, do not change it under the default
      `quirks.occupancy_override_hides_later_blockers`. Its alternative is owned by S-0209.
      With no override, take the most severe contents value in S-0209's order, then
      apply that claim's occupancy-record table. Terrain checks (height steps, land
      multipliers, the usable area) come from claim 10, S-0211 and S-0209; an occupant
      override does not cancel a prior terrain refusal.
    - Backing: `measurable` (scenario P7 runs 1 to 4, the held blocker of P6, P14 runs 1 to 4 for a
      unit ahead that faces the same way, P14 run 5 for the close case itself, and P9's train
      refresh run for an allied wall section and a vehicle without a weapon). For moving allied
      units without an outcome of their own, S-0209 claim 5 (its scenario C20 (b) for infantry); the
      split between moving units with and without an outcome of their own has the backing of S-0209
      rows VP22 and VP23. Building exceptions, overrides and the wood-wall case use
      S-0209's C23, C24, C18 and C20 (j4), respectively
    - Confidence: medium for the stationary-vehicle row, close case, moving allied units,
      override precedence and contents severity. The building exceptions retain S-0209's
      row-specific low or medium confidence; medium for the wood-wall case. Low for
      the remaining crusher and weapon distinctions not isolated by this spec's scenarios

27a. **A harvester without contact on a docking cell (Q-0371).** For an ordinary
    driving harvester, a refinery's docking cell (S-0402 claim 10) is not a reserved
    movement cell merely because another harvester holds contact with that refinery.
    Entry still uses S-0209 claim 5 and claim 27 here. In particular, the bib-edge
    exception of S-0209 claim 5 does not require contact; the non-blocking-building
    exception for a contacted building does. An empty docking cell outside the
    foundation has no building blocker. A docking cell within the foundation can
    therefore be enterable without contact through the bib-edge exception, but an
    ordinary blocking foundation cell is not made enterable by its docking role.
    Other occupants and terrain can still refuse entry. Merely reaching the cell
    establishes neither contact nor unloading (S-0402 claims 10 and 10a).

    Once there without contact, the harvester may remain at rest until an order, its standing
    behaviour or a general scatter rule moves it. Pathfinding does not evict it just because another
    harvester reserves the dock. A contacting harvester driving onto the cell can send it a scatter
    request; that is S-0207 claim 15's movement rule, not an eviction by pathfinding (S-0402 claim
    10a; Q-1870). A field destination supplied by S-0402 claim 3a uses the ordinary route rules: the
    departure cell's docking role adds no forbidden exit direction and does not require dock
    ownership. Destination changes and movement start use claims 20 and 21: a new destination ends a
    running retry delay. The harvester need not be able to re-enter its departure cell for a route
    out of it to be found.

    | Departure-route result | Driving harvester's result |
    |---|---|
    | Route found | Follows the stored route under S-0207 claim 15; later blocked steps use that claim and claim 22 here, not a harvest-specific timeout. |
    | Reachability gate fails | Destination cleared; no behaviour queued by pathfinding; the clearing leaves no retry delay (claims 21 and 23). |
    | Search fails, destination more than 1 cell away in Chebyshev distance, or the bridge condition of claim 23 holds | Destination cleared and *guard* queued for every player-house harvester in our engine; the clearing leaves no retry delay. Claim 23 owns the original's non-human-owner exception. |
    | Search fails otherwise | Destination cleared, which leaves no retry delay, and pathfinding queues no behaviour (claim 23). |

    A blocker leaving does not itself restore a cleared field destination or resume *harvest*.
    Another movement attempt requires a destination supplied by an order or by behaviour
    processing. If *harvest* remains current, its next seeking processing may supply one under
    S-0402 claim 3a; if it has ended, that seeking rule does not restart it. A reoffered
    destination is searched without a wait, since a new destination ends a running retry delay
    (claim 21). A contacting harvester's repeated docking offers are a separate case, owned by
    S-0402 claim 10a, not evidence of automatic retries by this non-contact occupant.
    - Backing: `measurable` (P27 for entry and resting persistence; S-0402 H20 for the harvest and
      docking exchange, its part e for the rest after a stop order). The failure reactions in the
      table are claim 23's and carry its backing. `unmeasured` for the rest after a give-up from the
      docking cell: no setup is known that is sure to give a departure from the docking cell that
      gives up (S-0402 claim 3a). `unmeasured` for a reoffered destination searched without a wait,
      as claim 21 rates a new destination's end of a running delay
    - Confidence: high for no docking-specific route restriction; the failure reactions as claim 23
      rates them; medium for entry and resting persistence pending P27; medium for the rest after a
      stop order pending S-0402 H20 part e, and for the rest after a give-up; the contacting
      harvester's scatter request and its results as S-0402 claim 10a rates them (low); medium
      (`unmeasured`) for a reoffered destination searched without a wait

## Formulas

All floating-point results below are formed with a 53-bit significand under the round-toward-zero
mode (S-0205): a sum or product of two values is trunc₅₃ of the exact result, and storing a result in
single precision is trunc₂₄.

**Switches of S-0204.** `quirks.table_root_distance` covers the lepton distances F6a to F6c, and
so the distance tests of claims 20, 24 and 27: with the switch off each distance is ⌊√*n*⌋, as
S-0204 says. It does not cover the estimate *h* of F2 or the nearest-cell choice of claim 25, which
use the table square root whatever the switch. S-0204's alternative is an integer root, and in
cells an integer root would change these two rules instead of refining a rounding: an integer *h*
makes many candidates tie and gives the search another shape, and in claim 25 a straight
neighbour (distance 1) would tie with a diagonal one (√2, floored to 1). Claim 25 orders its cells
in the same way under the table root as under an exact, unfloored root. `quirks.table_direction`
covers the direction of the occupant from the mover in claim 27's close case: with the switch off
that direction is S-0204's exact angle, and the 8-direction is taken of that angle. Taking the
8-direction of a facing (S-0204 F4) has no switch.

**F1. Step cost.** For a step in direction *d* onto a cell with entry outcome *v*:

> *c* = trunc₂₄(*b*(*v*) × *τ* + *ε*(*d*))
>
> *g*(new) = trunc₂₄(*g*(current) + *c*)

where *b*(*v*) is the base cost of claim 8 (table T2), *τ* = 4 when the target cell carries the
congestion mark (claims 9 and 19) and 1 otherwise, and *ε*(*d*) is the direction term of table T4, a
single-precision value. *b* × *τ* is an exact small integer. Unit: cost units. For a tunnel step
*c* = the Chebyshev distance between entrance and exit, exactly, with no *ε*. The start pair has
*g* = 0. The comparisons of *f* values (claim 12) are exact comparisons of the stored
single-precision values.
Recorded-pair margin (claim 11): with *g*ₑ the cost of the pair being expanded and *g*ᵣ the
destination's recorded cost (both single precision), the rule of claim 15 for an already recorded
destination applies when *g*ᵣ ≥ trunc₅₃(*g*ₑ + μ), where μ is the binary64 value nearest 1.009. The
comparison is exact.

**F2. Estimate.** For a candidate cell (*x*, *y*) and destination (*x*ₜ, *y*ₜ):

> *h* = *r*((*x* − *x*ₜ)² + (*y* − *y*ₜ)²), in cells
>
> *f* = trunc₂₄(*g* + *h*)

*r* is the table square root of S-0204 (its Formulas F1), applied to the exact integer sum of
squares, here in square cells. Its result is a single-precision value that never exceeds the true
square root, and the sum *g* + *h* of two single-precision values is exact before the narrowing.
The search layer does not enter *h*.

**F3. Retry delay.**

> *R* = trunc(trunc₅₃(*P* × 900)) ticks

with *P* the path-retry delay field in game minutes and 900 ticks per game minute. A value read
from data is a single-precision decimal (S-0003 claim 6) held as a double. The absent-key default is
the binary64 value nearest 0.016. trunc converts toward zero to an integer.

**F4. Close-enough distance.** *C*, in leptons, is the close-enough distance of S-0209 (its
Formulas F2 and Import mapping, which also give the absent-key default and the handling of the
value −1). This spec only uses it (claim 24). A unit that is a team member uses a distance given by
its team's current script step instead, which belongs to the team specs.

**F5. Threat term (coarse search).**

> *T* = trunc(trunc₅₃(*t* × *k*))

*k* is the threat coefficient (a double: the single-precision field value widened, or exactly 1.0),
and *t* = *t*(*q*, *s*, *L*) is the coarse threat (Terms) of the unit's house for the step from sector
*q* to sector *s* at level *L*, an integer:
- Level 0: *t* = 0 always.
- Level 1: *t* is the house's threat record (claim 16a) for the threat block holding the entered
  sector *s*. A level-1 sector lies inside one block of level 1, which is one threat block (Terms).
  *q* plays no part.
- Level 2: a sector of level 2 lies inside one block of 8 × 8 cells, whose four **quarters** are
  threat blocks: north-west, north-east, south-west and south-east. The **seed block** of a sector
  is the threat block holding its first cell in row order (the cell with the smallest *y*, and
  among those the smallest *x*). When *q* and *s* have the same seed block, *t* is that block's
  record. Otherwise let *d* be the direction from the seed block of *q* to that of *s*, one of eight,
  from the signs of the differences of their block coordinates (*y* grows to map south). Then
  *t* = ⌊(*a* + *b*) / 2⌋, where *a* is the smaller record of the quarters of *q*'s 8 × 8 block on
  its side towards *d*, and *b* the smaller record of the quarters of *s*'s 8 × 8 block on its side
  towards the opposite direction:

  | *d* | Quarters of *q* | Quarters of *s* |
  |---|---|---|
  | north | north-west, north-east | south-west, south-east |
  | north-east | north-east | south-west |
  | east | north-east, south-east | north-west, south-west |
  | south-east | south-east | north-west |
  | south | south-west, south-east | north-west, north-east |
  | south-west | south-west | north-east |
  | west | north-west, south-west | north-east, south-east |
  | north-west | north-west | south-east |

  So a step between level-2 sectors is charged for the threat on the facing sides of the two
  8 × 8 blocks. The same rule holds when *q* and *s* lie in one 8 × 8 block with different seed
  blocks: *d* still comes from their seed blocks, and *a* and *b* are then read from opposite sides
  of that one block (for *d* east, *a* is the smaller of its north-east and south-east records and
  *b* the smaller of its north-west and south-west records). For a straight direction, *a* and *b* are each the smaller of two records, so
  threat on only one of the two facing quarters of a block adds nothing for that block.

*T* is an integer. The term is used only when *k* > 0.00001 (F7). Confidence: medium that *t* is 0
at level 0 and for levels 1 and 2. Their aggregate effect is `measurable` (scenario P10 for the
route, P30 part a for the records). The level-2 choice of quarters is `unmeasured`: one object's
spread always reaches neighbouring quarters as well, and no proposed fixture separates threat on one
facing quarter from threat on two.

**F6. Lepton distances.** Each is ⌊*r*(*n*)⌋ leptons, with *r* the table square root of S-0204
(its Formulas F1) and *n* the exact integer sum of squared coordinate differences in leptons:
- **F6a. Unit to destination** (claim 24): *n* = Δ*x*² + Δ*y*² + *z*², from the unit's position to
  the destination cell's centre (256*x* + 128, 256*y* + 128), with *z* the unit's world height. The
  destination point is taken at world height 0
  (`quirks.substitution_distance_flat_destination`).
- **F6b. Cell centre to cell centre** (claim 24): *n* = Δ*x*² + Δ*y*².
- **F6c. Unit to point** (claims 20 and 27): *n* = Δ*x*² + Δ*y*² + Δ*z*², between the unit's position
  and its heading point (claim 20) or the other unit's position (claim 27). This same
  ⌊*r*(*n*)⌋ distance applies to ship refresh and chase cutting: it is not an exact
  Euclidean square root followed by integer truncation. The original forms the squares
  and sum in double-precision floating point under S-0205's round-toward-zero mode.
  For signed 32-bit differences and a sum below 2⁵³ square leptons, those operations are
  exact, as in S-0204 claim 6. That is the arithmetic domain of this formula, not a
  guarantee for arbitrary representable world-height separations.

*D* / 256 in claim 20 is an integer division of the non-negative integer *D*.

**F7. Threat tests.** The cell threat tests of claims 17 and 18 form trunc₅₃(*t* × *k*) for the
integer threat value *t* of a cell and compare it exactly with a binary64 constant: < 1.0 in claim
17, ≥ the binary64 value nearest 0.01 in claim 18. The coefficient test "*k* exceeds 0.00001" of
claims 13 and 18 compares *k* exactly with the binary64 value nearest 0.00001.

**F8. Legs of the coarse estimate (claim 24).** The coarse search of the estimate runs from the
found cell's coarse end cell to the destination's (claim 13b). Let *s*₁, …, *s*ₙ be the sectors of
its level-0 corridor, *s*₁ holding the found cell's coarse end cell and *s*ₙ the destination's.
- When the found cell and the destination are both bridge cells of one bridge (for each of them,
  the first bridge link in creation order that it lies within 3 cells of, Terms, is the same link),
  *E* is just their Chebyshev distance, and no legs apply.
- Otherwise, when the destination is a bridge cell, a *destination leg* is added: the Chebyshev
  distance from the destination to its anchor for *s*ₙ₋₁ when *n* ≥ 4, and for *s*ₙ when *n* < 4.
  When the anchor for *s*ₙ₋₁ does not exist, the anchor for *s*ₙ is used; when that does not exist
  either, the destination's coarse end cell takes its place.
- When the found cell is a bridge cell, a *found-cell leg* is added in the same way: the Chebyshev
  distance from the found cell to its anchor for *s*₂ when *n* ≥ 4 and for *s*₁ when *n* < 4, then
  its anchor for *s*₁, then its coarse end cell.

The **anchor** of a bridge cell *X* for a sector *S* is the first of these six cells that lies inside
the usable area and whose level-0 sector is *S*: *X*'s first bridge end (Terms), the two cells beside
it across the bridge's axis, *X*'s second bridge end, and the two cells beside that. For a bridge
along cell x the order is: east end, the cell south of it, the cell north of it, west end, the cell
south of it, the cell north of it. For a bridge along cell y: south end, the cell west of it, the cell
east of it, north end, the cell west of it, the cell east of it. A missing bridge end contributes no
cells. When none of the six qualifies, the anchor does not exist.

Integers, in cells.

**F9. Coarse route cost (claim 13).** For a route reaching sector *s* from sector *q*:

> cost(*s*) = trunc₂₄(trunc₅₃(trunc₅₃(*κ*(*s*) + cost(*q*)) + *T*) + *λ*)

*κ*(*s*) is the T3 constant for *s*'s category, cost(*q*) the stored single-precision cost of the
route to *q* (0 for the start sector), *T* the integer threat term of F5 (0 when the coefficient test
fails), and *λ* the binary64 value nearest 0.001 when the route enters *s* across a diagonal connection
(claim 13b), otherwise 0. Unit:
cost units. A sector already reached gets the new route only when the new stored cost is strictly
smaller than its stored cost (an exact comparison). Under the round-toward-zero mode the diagonal
term adds less than 0.001 once a cost reaches 2¹³ cost units, and nothing once it reaches 2¹⁴. Costs that
large arise only with large threat terms.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Stored path length | 24 | steps | claim 20 |
| Path refresh distance | 512 | leptons | claim 20, recomputation as the last stored step begins |
| Expansion limit | 65527 | expanded pairs | claim 12 |
| Failing expansion count | 10000 | expanded pairs | `quirks.path_search_10000_failure` |
| Recorded pairs per search | 65535 | pairs besides the start | claim 12, departure D2 |
| Coarse routes pending at once | 9999 | sector routes | claim 13, `quirks.coarse_limit_records_cost` |
| Path length beyond which results are undefined | 1999 | steps | original only: departure D2 removes the limit in our engine |
| Corridor length beyond which results are undefined | 500 | sectors per level | departure D2: our engine treats a longer chain as a failed coarse search |
| Cell searches per path | 5 | searches | claim 13 |
| Recorded-pair margin | 1.009 | cost units | claim 11 (corner case only), F1 |
| Blocker chain | 10 | cells (one object each) | claim 8 |
| Blocker chain deck rule | 3 | height levels | claim 8: a further cell is read on its deck when the unit heading into it stands at least this much higher |
| Deck layer height | 4 | height levels | Terms, *search layer*; claims 10 and 25 |
| Train exclusion deck rule | 2 | height levels | claim 16: the deck is excluded when the start's search layer is at least this much higher |
| Stored steps of a qualifying unit | 2 (vehicle), 3 (infantry unit) | steps, at least | claim 19 |
| Congestion-mark cost factor | 4 | factor (dimensionless) | *τ* in F1; claims 9, 19 |
| Corridor levels | 3 | levels | block sizes 2, 4 and 8 cells |
| Region join limit | 1 | height levels | claim 3 |
| Region join limit along cell x | 3 | height levels | claim 3, `quirks.region_join_height_asymmetry` |
| Nearby search largest radius | 31 | cells | claim 25, less on maps with *w* + *h* < 32 |
| Nearby search candidates | 24 | cells | claim 25 |
| Substitution detour allowance | 6 | cells | claim 24 |
| Straightening scan | 20 | steps | claim 18 |
| Threat cells allowed in the closing stretch | 3 | cells | claim 18 |
| Threat block | 4 × 4 | cells | Terms, claim 16a |
| Threat spread | 1, ½, ¼ (each share rounded down) | fraction of the amount, for the object's block, each edge neighbour and each diagonal neighbour | claim 16a |
| Congestion block | 5 × 5 | cells | claim 19 |
| Close-case distance | 511 | leptons | claim 27 |
| Close-case facing reversal | 32767 | facing units | claim 27, `quirks.opposite_facing_reversal` |
| Threat thresholds | 1.0, 0.01, 0.00001 | 1.0 and 0.01: threat value × coefficient. 0.00001: coefficient (all dimensionless) | claims 13, 17, 18, F7 |
| Diagonal connection cost | 0.001 | cost units | claims 13 and 13b, F9 |
| Lane reach of a bridge link | 2 | cells to the side of the link's line | coarse end cells, claim 13b |
| Lane reach for the same-bridge test | 3 | cells to the side of the link's line | F8 |
| Lane reach for a deck's reach region | 1 | cells to the side of the link's line | claim 4 |
| Lane reach for a link changed in play | 3 | cells to the side of the link's line | claim 3 |
| Height step onto or off a deck | 4 | height levels | claim 10 |
| Train start-layer tolerance | 2 | height levels | claim 7: the train's start pair is raised to the deck beyond it |
| Bridge-role tiles that count as a bridge end | 16 | tiles per role | Terms, *bridge ends* |
| Ticks per game minute | 900 | ticks per game minute | glossary |
| Leptons per cell | 256 | leptons | glossary |

**T2. Base step cost by entry outcome** (cost units).

| Outcome | clear | cloaked blocker | moving blocker | closed friendly gate | friendly obstacle | enemy obstacle | stationary friendly | blocked |
|---|---|---|---|---|---|---|---|---|
| *b* | 1 | 1000 | claim 8 (1, 4 or 1000) | 1 | 60 | 20 | 8 | not considered |

**T3. Sector entry cost by passability category** (coarse search, claim 13; cost units).

| Category | open | crushable | wall-like | beach | water | partial obstacle | blocked | off-map |
|---|---|---|---|---|---|---|---|---|
| cost | 1 | 0 | 0 | 1 | 1 | 0 | 1 | 1 |

**T4. Direction term** *ε*(*d*), the single-precision values nearest these decimals, in cost units.

| *d* | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
|---|---|---|---|---|---|---|---|---|
| *ε* | 0.001 | 0.005 | 0.002 | 0.006 | 0.003 | 0.007 | 0.004 | 0.008 |

Straight steps carry the smaller terms, so among routes of equal step count and outcomes the one
with fewer diagonal steps is cheaper, and among straight steps map north is cheapest, then east,
south and west.

## State and ordering

| State of the unit's path | Event | Result |
|---|---|---|
| no stored steps, retry delay over | needs to move | path computation: the gate of claim 6 judges the destination as ordered, and only a destination that passes it can be substituted (claim 24). On success the steps are stored; no retry delay follows for driving, ship or walking, but the hovering locomotor starts R again (claim 21). On failure claim 23 |
| no stored steps, retry delay running | needs to move | waits (a follower car sent by its leader computes at once instead, claim 26) |
| stored path | next step *clear* | follows it (movement specs) |
| stored path, driving or ship locomotor | next step of another outcome, neither *moving blocker* nor *blocked* | reacts as S-0209 and the movement specs say: opens a gate, attacks an obstacle or waits (claim 22) |
| stored path, more than one step | the next step begins | the step is taken |
| stored path, one step left, driving or ship locomotor | that step begins, heading point more than 512 leptons away | immediate recomputation in mode 0 (claim 20), the old step is still taken |
| stored path, driving or ship locomotor | next step *moving blocker* | blockage timer starts if not running. When the retry delay is over, re-path in mode 1, or 2 once the blockage timer has run out |
| stored path, driving or ship locomotor | first check of the track's first cell: outcome *blocked* | stored path cleared, fresh search on this tick regardless of the retry timer; a found path gets a repeated check on this tick (claim 22) |
| stored path, walk locomotor | next step of any outcome other than *clear* | reacts as S-0209 claims 7 to 10 say, with the delays of claim 21 |
| any | destination changes | stored path cleared, fresh search. For the driving and walk locomotors, a change to a different destination ends a running retry delay, so the search is not held back (claim 21); for the walk locomotor the destination it already holds, given again, ends it too (claim 21), except for a harvester's offer of its docking cell (S-0402 claim 10a). For the ship locomotor and the locomotors other than the driving, walk and hovering locomotors, the search waits for a running delay like any fresh search (claims 20 and 21; a follower car sent by its leader computes at once instead, claim 26), at low confidence (`unmeasured`, Open questions). For the hovering locomotor, S-0503 gives whether it waits |

Ordering consequences:
- A search made after the only enabled bridge connection is destroyed, even later in the same
  tick, already sees the banks as separated (claim 5). An intact alternative connection can
  still join them.
- The gate uses the destination as ordered, so an ordered destination in another reach region is
  dropped even when a cell near it would be reachable (claims 6 and 24).
- Straightening (claims 17 and 18) uses the entry outcomes at the moment of the search, including
  the temporary congestion marks of claim 19, so a path found in mode 1 or 2 is straightened around
  the marked cells.
- The search reads entry outcomes as they are at the moment of the search. Units that move later in
  the same tick are not anticipated except by the blocker chain of claim 8.

## Edge cases and quirks

- **`quirks.path_search_10000_failure`** (default: on). A search that reaches the destination after
  exactly 10000 expansions fails. Off: it succeeds as at any other count.
- **`quirks.path_no_reopen`** (default: on). A (cell, search layer) pair keeps the cost and
  predecessor of the first expansion that recorded it, and never becomes pending again (claim 11). Off:
  when an expansion reaches a recorded pair more cheaply, the pair's cost and predecessor are
  replaced and it becomes pending again, as in a standard best-first search. The 1.009 margin corner of
  claim 11 applies only when the switch is on. With the switch off, claim 15 treats an already
  recorded destination like one not yet recorded: a later neighbour that finds the destination's
  outcome *blocked*, within 1 height level of it, ends the search there, with no comparison of
  costs.
- **`quirks.region_join_height_asymmetry`** (default: on). Neighbouring cells of one category along
  cell x join one region when their height levels differ by up to 3, while in every other direction
  they join only up to a difference of 1 (claim 3). The original therefore treats some
  cliff-separated cells as connected, and a unit ordered across such a cliff searches and fails
  instead of being stopped by the gate of claim 6. Off: the limit is 1 in every direction.
- **`quirks.flatten_threat_start_cell`** (default: on). The right-angle flattening of claim 17
  reads the threat value at the new steps' first cell for every cell it checks. Off: it reads the
  threat value of each checked cell.
- **`quirks.flatten_straight_run_v`** (default: on). In claim 17, the new straight steps of a
  flattened V of diagonal runs, with equal straight steps directly after them, the second run
  of a V of straight runs that was not flattened, and the new steps of a flattened V of straight
  runs of the pair 6 and 0 can open a V of straight runs. Such a V is replaced by 2*j* steps, which
  are diagonal except for the pair 6 and 0, displace every later step of the path by *j* cells
  along each axis, and the pair 6 and 0 gives direction 0 instead of 7. The stored path then does not end on the
  cell the search reached. Off: no run of straight steps opens a V, so only V's of diagonal runs are
  flattened and the path always ends where the search ended.
- **`quirks.detour_mirrored_reference`** (default: on). From the third leg of claim 18 on, the
  reference distance of a step is measured from the previous leg's first cell *q* to the step's end
  cell mirrored through the current leg's first cell, so triggers there bear no fixed relation to
  how far the path has really got from *q*: stretches can be replaced that are not detours (they
  are only reordered into one of the two shapes), and detours can be missed. Off: the reference
  distance is the Chebyshev distance from *q* to the step's end cell.
- **`quirks.detour_after_tunnel_position`** (default: on). After a tunnel step, claim 18 starts
  afresh as if the rest of the path began at the cell north of the tunnel entrance, so the cells it
  checks and the stretches it replaces after a tunnel are those of a path shifted away from where
  the unit will really be. Off: it starts afresh at the tunnel exit.
- **`quirks.path_congestion_toggle`** (default: on). The congestion marks of claim 19 follow the
  parity of each cell's count, so a cell counted twice is left unmarked, and a cell that already
  carried the congestion mark loses it during the search when its count is odd. Off: during the search a
  cell carries the mark when it carried it before or its count is at least 1.
- **`quirks.corridor_retry_no_fallback`** (default: on). When the new coarse search that follows a
  corridor retry (claim 13) fails at any level, except the chain-length failure of departure D2 (Edge cases and quirks), the path computation fails, and a unit can stop at
  once at a short strip of ground it cannot enter that the coarse searches count as open (Edge
  cases below, P12 run T). Off: one more
  cell search without a corridor is made, with the same limits as the other cell searches, and the
  computation fails only if that fails too.
- **A to-tile building's exemption is taken away (fixed, no switch).** Every building, a type with
  a to-tile field (Import mapping) included, loses the corridor exemption of its rectangle when it
  leaves the map (claim 13). In the original a to-tile building's rectangle stays exempt for the rest
  of the game, so a later corridor search can use those cells; the project lead ruled this a plain
  bug, fixed with no switch, on 2026-10-24. Backing: `unmeasured`, for the reason given in claim 13's
  backing. Confidence: medium.
- **`quirks.corridor_free_part_ignores_height`** (default: on). The free part of claim 13a, and the
  sectors it borders, judge each cell by its entry outcome at its own height level only, not by the
  height rule of claim 10, so a ledge (a line of flat cells one level above flat ground, no slope
  between) does not stop the free part; a retry at a ledge then closes nothing, and the unit gives
  up after 5 identical cell searches (Edge cases below, P12 run T). Off: a cell whose entry from
  the step's start the height rule refuses does not belong to the free part and does not count for
  its borders. Independent of `quirks.corridor_split_probe` and
  `quirks.corridor_free_part_unusable_cells`. Backing: `measurable` (P12 run T, ledge maps) for the
  give-up; the free part and the corridor are `unmeasured` separately (claim 13a). Confidence:
  medium that a ledge stops no free part, so no retry changes the corridor; medium that the 7- and
  9-ledges of run T fail at every shift; low for the shifts at which the 3- and 5-ledges fail (as
  the ledge entry under Edge cases rates them).
- **`quirks.corridor_split_probe`** (default: on). Whether the progress sector counts as split
  (claim 13a) is decided at the probe cell, which need not lie in that sector at all, and a sector
  that does not fill its block counts as split whenever the probe cell is in it. Off: *S* counts as
  split when some cell of *S* inside the usable area is not in the free part, and as whole
  otherwise.
- **`quirks.corridor_free_part_unusable_cells`** (default: on). In a corridor retry (claim 13a), a
  cell whose category the unit's own movement zone cannot use belongs to the free part, and counts
  for the free part's borders, whatever its entry outcome. Off: only a cell of a usable category
  with entry outcome *clear* belongs to the free part or counts for its borders. Independent of
  `quirks.corridor_split_probe`. Claim 13a says why the difference can show only for an infantry
  type with the jumpjet flag.
- **`quirks.substitution_centre_ground_level`** (default: on). In destination substitution
  (claim 24), the nearby-cell search judges heights around the ground of a bridge-cell destination
  unless the unit stands on a bridge deck, although the search aims at the deck (claim 7). So a
  unit on the ground ordered onto an occupied deck cell gets a bank cell or no substitute, never a
  deck cell. Off: in claim 24's use the centre's level includes the deck whenever the destination
  is a bridge cell, for every unit that is not an aircraft, so free deck cells next to the
  destination can be adopted. The other uses of the search (claim 25, table T5) are unchanged either way.
- **`quirks.substitution_distance_flat_destination`** (default: on). The unit's distance to its
  destination in claim 24 (F6a) measures to the destination cell's centre at world height 0, so a
  unit on high ground counts as farther away than it is: it compares that larger distance with the
  close-enough distance, and a nearby cell needs only to be closer to the destination than that
  larger distance. Off: the destination point is the cell's centre at the cell's ground world
  height, and the distance is the true spatial distance.
- **`quirks.path_refresh_keeps_old_step`** (default: on). When a path is recomputed as the last stored
  step begins (claim 20), the unit takes the old step and drops the new path's first step, so the
  rest of the new path is followed from a cell one step away from where it was planned whenever the
  two first steps differ. Off: the unit takes the new path's first step instead of the old one.
- **`quirks.nearby_centre_twice`** (default: on). The nearby-cell search of claim 25 examines its
  centre cell twice, so a qualifying centre is collected twice: it takes two of the 24 places and,
  when the rule gives no reference cell, two of the *n* positions of the choice by tick number.
  Off: the centre is examined and collected once. No known use of the search shows the difference
  in practice (claim 25).
- **`quirks.opposite_facing_reversal`** (default: on). The close case of claim 27 compares the
  occupant's facing turned by 32767 facing units, not by the exact half turn of 32768, so facings
  that differ by exactly 32768 can miss the test by one unit at the boundary of an 8-direction.
  Off: the occupant's facing is turned by 32768.
- **`quirks.coarse_limit_records_cost`** (default: on). In claim 13, a route found while 9999
  routes are pending still sets the sector's recorded cost, although it is never pending, so later
  routes to that sector must beat it. Off: such a route leaves the sector's recorded cost unchanged.
- **`quirks.deck_side_step_ground_level`** (default: on). Claim 10 judges a step from a deck pair
  by the height level of the ground under the deck, so the slope rule for a difference of 1 lets a
  unit on a deck step sideways onto a cell that is not a bridge cell, 1 level above or below that
  ground, when the lower of the two cells is a slope: it lands on that cell's ground, 3 levels
  (difference +1) or 5 levels (difference −1) below the deck. Off: from a deck pair, a difference
  of 1 onto a cell that is not a bridge cell is refused. A step of 1 onto another bridge cell's
  deck (a deck over sloping ground) is allowed either way.
- **`quirks.train_exclusion_search_only`** (default: on). Claim 16's exclusion of the three cells
  behind a train holds for the search only, so straightening can give the stored path a reversing
  first step. Off: the straightening passes also treat the three cells as not enterable, and make
  no change that would put the path's first step into one of them.
- **Quirks of the passability category** belong to S-0211: `quirks.passability_wheel_column` (the
  category judges every movement zone by the wheel column of the land multipliers) and
  `quirks.passability_laser_fence_lost` (laser-fence sections leave the category alone). Through
  claim 1 both change regions and reach regions here too.
- **Buildings do not separate regions** (claim 1). A destination enclosed by buildings passes the
  gate and costs a full search, up to 5 cell searches, before it fails. This is original behaviour.
- **A unit routed through its own base** (claims 1, 12, 13; Q-1771). The coarse searches see
  through buildings, so the first corridor from a factory's exit often runs through the factory
  itself and through the rows of buildings beyond it. The cell search inside that corridor is not
  stopped by them: every cell of each building's exemption rectangle is exempt (claim 13), and the
  rectangles of a group of buildings join into one band of cells round the group. Where the
  corridor meets a building or a row of buildings with open ground round it, the search follows
  that band round the row's end and rejoins the corridor on the far side. A unit standing at a
  factory's exit and ordered across its own base therefore keeps its destination and gets a route
  in the tick of the order, instead of having its destination cleared (claim 23), and reaches the
  destination on the first cell search; no retry is made, and the limit of 5 cell searches plays no
  part. The same holds for an infantry unit and a vehicle, and for the buildings of an allied
  house, whose cells give the unit the same outcome as its own house's (claim 27). The exemption
  itself is given whatever a building's owner (claim 13), but a building of a house the unit's
  house does not ally gives an armed unit *enemy obstacle* rather than *blocked* (S-0209 claim 5),
  which this answer does not cover. This is stated for groups whose detour stays far inside the
  expansion limit of claim 12: the layouts of P31 and the walls of P12 need a few hundred
  expansions at most. A band round a far larger group, or one cut by something the cell search
  cannot cross and that gives no exemption (a ledge or a strip of ground the unit cannot enter,
  below), can still make the first search fail, and the retry rules of claim 13 then decide. Which
  end of a row the unit goes round is decided by the cell search's own rules inside the corridor:
  claim 12's step costs, though the search need not find the end with the cheaper route (claim 7),
  and at an exact tie the order of equal routes (claim 12a, whose design order the original does
  not follow in every tie). P31 records the side taken. **Answer to Q-1771** (not yet observed).
  (1) The original walks out: the unit at the factory's exit, and the unit of the request's 3 × 2
  block and row of ten, find the way on the first cell search inside the first corridor, round the
  buildings on the cells of their exemption rectangles, and never reach the limit of 5. It goes
  round
  the end that the rules above decide. (2) The corridor exemption includes the exemption
  rectangles of buildings (claim 13). The rest of claim 13 applies to this case as stated: the limit of 5 cell searches, no corridor-free search after the 5th
  failure, the coarse searches seeing building cells as *open*, and the retry table. That rests on
  claim 13's own ratings, not on P31 or P12, which do not count cell searches.
  (3) So no switch is needed for this case. Backing: `measurable` (P31; P12 for a long wall of
  buildings) that the unit walks out; `unmeasured` that it does so on the first cell search, round
  the buildings' exemption rectangles (claim 13's Backing). Confidence: medium that the layouts of
  P31 and P12 are passed on the first search (that each layout's corridor meets the band as
  described was traced by hand, without computing its exact cells); item 2 as claim 13 rates each
  rule (medium for the limit of 5, for the retry's branches and for "on the first cell
  search"); for the side, medium (the cell search's rules and claim 12's cost formula), and at an
  exact tie the original's side is `unmeasured`, as in claim 12a.
- **A harvester leaving a refinery's docking cell across its base** (claims 1, 10, 13, 23,
  27a; Q-1870). This is the entry above with a harvester as the unit and the docking cell
  (S-0402 Terms) as the exit. No rule of the path computation depends on the harvester flag,
  and the field leg after unloading is an ordinary destination: the remembered cell, or the
  cell the ring search gives (S-0402 claims 3 and 6). For a foundation at least 3 cells wide
  the docking cell lies in the refinery's own exemption rectangle, and so does the ring of
  cells round the foundation in every case, so the cell search leaves the docking cell on that
  ring, goes round the refinery and follows the band of the other buildings' rectangles. The
  exemption does not look at height. Raised ground cuts the band only where the cell search
  cannot step from one of its cells to the next (claim 10), as at a ledge, or where a strip of
  ground the unit cannot enter crosses it (next entries). Ground that no unit can enter at
  all is closed to the coarse searches too, so the corridor itself
  goes round it. **Answer to Q-1870** (not yet observed). (1) As for Q-1771, the original
  finds the way on the first cell search inside the first corridor, round the refinery and the
  buildings beyond it. There is no higher limit, no search without a corridor and no other
  corridor: claim 13 stands, and neither `quirks.corridor_retry_no_fallback` nor
  `quirks.corridor_split_probe` is needed for this. Only where a ledge or such a strip cuts
  every way round does the first search fail; claim 13's retries then decide, and a give-up
  follows claim 23. (2) After a give-up, a harvester of a player house other than a chrono
  miner stands on the docking cell in *guard* and stays there. No refinery rule, harvest rule
  or retry delay moves it, and it does not try the field again until an order moves it, or a
  scatter request that it accepts (claim 27a; S-0402 claims 3a and 13; S-0215 claim 11a). The
  original's computer-controlled owner, whose harvester gets *area guard* with a
  house-selected destination (claim 23a) and then *harvest* again from *area guard* (S-0215
  claim 26), applies in our engine only to the harvesters of the neutral and special houses
  (claim 23, Departure D5). (3) A second harvester that comes loaded to that refinery is
  S-0402 claim 10a. Where no substitute is adopted at an offer, as within the close-enough
  distance, it can make the resting one move by a scatter request. While claim 24 adopts a
  substitute at each of its offers, as it can on a base raised high enough, it does not dock.
  Backing: `measurable` (P31 run D for item 1; for
  item 2, P27 and S-0402 H20 part e for the same rest after a stop order; S-0402 H20 parts e
  and f for item 3). `unmeasured` for item 2's rest after a give-up: no setup is known that is
  sure to give a departure from the docking cell that gives up (S-0402 claim 3a), and without
  one a rest cannot be told from a retry that never comes. `unmeasured` that item 1 happens on
  the first cell search, as for Q-1771. Confidence: medium that the path computation has no rule
  of its own for harvesters; medium for item 1; medium for item 2, both for the rest after a
  stop order (pending S-0402 H20 part e) and for the rest after a give-up. For item 3, low for
  the scatter request and its results, as S-0402 claim 10a rates them, and medium that a harvester
  given a substitute at each offer on a raised base does not dock, as S-0402 claim 10a rates the
  substituted offers on a raised base.
- **A short line of buildings across the route, with open ground round it** (claims 1, 13; Q-1280).
  As for a unit in its own base (above), the first corridor runs through the line, and the cell
  search inside it walks round the line's end on the cells of the buildings' exemption rectangles.
  So the first cell search finds the way round at every alignment against the sector grid, for
  the straight lines of 3, 5, 7 and 9 1 × 1 buildings that run S of P12 tests; the retries of
  claim 13 are not reached. For far longer lines the limit stated above applies. A line of wall
  sections behaves alike for a different reason: its cells are *wall-like*, which the zone table
  closes to movement zones *normal* and *infantry*, so the corridor itself goes round the line,
  and the wall's own exemption covers the cells next to it. So neither a line of buildings nor a
  line of walls shows the failure of the next entry. **Answer to Q-1280** (not yet observed). (1)
  A vehicle 20 cells from a straight north–south line of seven 1 × 1 buildings on open ground,
  ordered to a cell just behind the line, drives round the line at once at every alignment, and
  so it does at lengths 3, 5 and 9. (2) An infantry unit behaves alike, and so does the approach
  of a recycling building just behind the line (S-0603, S-0605), since all use this path
  computation. (3) The corridor exemption includes every building's exemption rectangle (claim 13),
  so the lines of 7 and 9 buildings are passed at every alignment, as the shorter ones are. This
  has not been observed: it follows from the exemption alone, and P12 run S has not run.
  Backing: `measurable` (P12 run S) for items 1 and 2, `unmeasured` for item 3, as claim 13's
  Backing says (the walk does not tell the rectangles from a successful retry). Confidence:
  medium for items 1 to 3.
- **A short strip of ground the unit cannot enter, across the route, with open ground round it**
  (claims 1, 8, 13, 13a). An obstacle that the coarse searches count as passable, that the cell
  search cannot cross, that gives no exemption and whose cells stop the free part of claim 13a
  makes the first cell search inside the corridor fail. A strip of a land type whose multiplier
  for the unit's locomotion class is 0 while its wheel multiplier is above 0.01 is one: the
  category reads only the wheel column (S-0211 claim 11, its `quirks.passability_wheel_column`),
  so the strip is *open* to the gate and to the coarse searches, while its cells have entry
  outcome *blocked* for the unit (claim 8) and nothing on them gives the exemption. Whether stock
  data has such a land type for some locomotion class is not checked here; run T of P12 makes one
  with test data. The way round is then found
  only if the retry changes a corridor in a way that lets a new coarse search route round it. In
  the retry table, a level at which the progress cell's sector is *whole* and whose free part
  borders every sector connected to it closes no connection at that level. A new coarse search
  uses only the connections of its own level and the corridor of the level above, so that level's
  corridor is then found again unchanged (closures at other levels do not alter it). A sector
  below level 2 can be on a corridor only when its parent is, or its category is *crushable*
  (claim 13). When the strip lies inside one level-2 sector (blocks of 8 cells) whose free part
  borders every sector connected to it, and the probe of claim 13a reads that sector as whole, the
  level-2 corridor stays the same through every retry. A new coarse search can then fail at level
  1 because the remaining way round lies outside that corridor, and the path computation fails
  (claim 13's last-but-one row); the unit does not move and its destination is cleared (claim 23).
  This follows from the rules of claims 13 and 13a as written, and run T of P12 tests it on the
  original. This failure has gameplay impact (a unit refuses an order although an open way round
  exists), so it gets a switch, **`quirks.corridor_retry_no_fallback`** (default: on, the
  original). Off: when the new coarse search after a retry fails, one more cell search is made
  without a corridor, bounded by the same limits as the other cell searches (it counts towards the
  5 of claim 13 and ends as failed at the expansion limit) and deterministic; the path computation
  fails only if that search fails too. With it off (our engine's alternative, which the original
  does not have), an open way round a short strip is expected at every alignment. This is an
  engine check, not a claim about the original, so no oracle measurement backs it. With it on,
  whether a given strip fails depends on its alignment with the sector grid and on the probe cell.
  With `quirks.corridor_split_probe` on (the default), the probe reads the level-2 sector as split
  only when the probe cell lies in that sector and the free part does not fill its block; a block
  that holds strip cells never has a full free part, so the reading is split exactly when the
  probe cell falls in the sector and inside the usable area, and then the connection to the next
  corridor sector is closed and a new level-2 corridor may go round. With
  `quirks.corridor_split_probe` off, a sector reads as split whenever any usable cell of it lies
  outside the free part, so every block that holds a strip cell reads as split whatever the probe
  cell, and the failure is expected to occur at fewer alignments. The rule alone does not settle
  the lengths: its failing case is stated for a strip inside one level-2 sector, and a strip of 9
  cannot lie inside one. So the expectations that a strip of 3 or 5 is passed, and that strips of
  7 and 9 fail at some alignment, are measurement expectations, not consequences of the rule. This
  spec does not predict which of run T's shifts is that alignment (it depends on the progress cell,
  which depends on the order of candidate cells, claim 11); run T (with the switch on) records the
  result for every shift so the failing shifts can be named, and the claim stays at "some shift
  fails". The same path computation serves infantry, vehicles and the approach of a recycling or
  absorbing building (S-0603, S-0605). Backing: `measurable` (P12 run T) for lengths 3 and 5 being
  passed and for the existence of failing 7- and 9-strip placements; `unmeasured` until run T has
  been run for which shifts fail, because the progress cell is not predicted. Confidence: high that
  the rules as written produce the failure whenever the level-2 corridor is unchanged, low for the
  exact strip lengths and alignments.
  A ledge differs: a straight line of flat cells one height level above the flat ground on both
  sides, with no slope between them, is also open to the coarse searches (cells one level apart
  join, claims 3 and 13b) and refused by the cell search (claim 10), but the free part of claim 13a
  judges only a cell's entry outcome at its own height level, not the height rule, so the ledge's
  cells do not stop it. A retry then reads the progress sector as whole with a free part that
  borders every connected sector, closes nothing at any level, and each new coarse search finds the
  same corridor again. So when the first corridor crosses the ledge without a cell round its end,
  the same cell search fails 5 times and the path computation fails at the 5th, whatever
  `quirks.corridor_retry_no_fallback` is set to. This has the same gameplay impact as the strip (a
  unit refuses an order although an open way round exists), so it gets a switch,
  **`quirks.corridor_free_part_ignores_height`** (default: on, the original). Off: the free part
  and its borders (claim 13a) also apply the height rule of claim 10 to each step, so a cell that
  the step into it would be refused by that rule does not belong to the free part; a ledge then
  stops the free part as a strip does, and the retries can close connections and lead a new coarse
  search round it, under the same rules and switches as for a strip. It is independent of
  `quirks.corridor_split_probe` and `quirks.corridor_free_part_unusable_cells`. Backing:
  `measurable` (P12 run T, ledge maps) for the original's give-up; the free part and the corridor
  are `unmeasured` separately (claim 13a). Confidence: medium that no retry changes the corridor in
  that case; medium that the 7- and 9-ledges of run T fail at every shift; low for the shifts at
  which the 3- and 5-ledges fail.
- **Departure D1: unknown movement zone.** In the original a movement-zone value outside the
  vocabulary leaves the type with no movement zone. The gate then always passes, and the search
  reads the reach region of a non-existent movement zone, which is undefined. Our engine follows
  S-0211 claim 4, which owns the parsing of zone names: such a value is treated as an absent key,
  so the type keeps the movement zone it held before (from an earlier rules layer, or the
  absent-key default *normal*), and the importer warns. In our engine every type therefore has a
  movement zone, and the first exception of claim 6 never applies.
- **Departure D2: search size limits.** In the original, a search that records more than 65535
  pairs besides the start pair, or finds a path of more than 1999 steps, has undefined results. Our
  engine ends a search as failed when it would record a 65536th pair besides the start pair, and
  keeps a found path of any length (only its first 24 steps are stored anyway). Likewise a coarse
  search (claim 13) whose chosen chain has more than 500 sectors at some level has undefined results
  in the original. Our engine treats such a coarse search as failed, and the table of claim 13 then
  applies: after a first coarse search the cell search runs without a corridor, after a repeated one
  the path computation fails, whatever `quirks.corridor_retry_no_fallback` is set to, because a departure takes no switch; confidence is low that such chains occur on legal map sizes.
- **Departure D3: building destination with no substitute.** When the destination cell holds a
  building, its outcome is *blocked*, and the nearby-cell search of claim 24 finds no cell, the
  original replaces the unit's destination by one that names no cell, and what follows is
  undefined. Our engine keeps the destination unchanged, so the search approaches the building
  under claim 15. This needs no importer warning, because the cause is the game situation, not the
  data.
- **Departure D4 (accepted): labelling-order cases of regions and sectors.** In rare height
  configurations the original's regions differ from the rule of claim 3 (smallest known case in
  claim 3). Its sectors and connections can differ from the rule of claim 13b in the three ways that
  claim lists: two of them need neighbouring cells 2 or more height levels apart, while the third,
  a connection that the original makes diagonal although a straight pair exists, is common but
  changes only the extra cost of 0.001 in the coarse search. Each difference comes from the order
  in which the original labels the map, and reproducing it would mean reproducing that labelling
  exactly. Our engine follows the rules of claims 3 and 13b. The project lead accepted this
  departure (decision of 2026-10-10). Route costs and every formula stay exact.
- **Departure D6: coarse end cell of a bridge cell with no answer.** When the coarse end cell of
  claim 13b is sought for a bridge cell that belongs to no bridge link and neither of whose bridge
  ends lies inside the usable area, the original reads undefined data. Our engine uses the bridge
  cell itself as the coarse end cell. The case is not known to arise in stock play, and no importer
  warning is needed, because the cause is the map situation.
- **Departure D5: rules for computer-controlled owners only.** This is a design choice of scope,
  the kind of departure the glossary entry *departure* allows, and it follows the project lead's
  decision recorded under the glossary entry *AI house*. The following original rules are not
  reproduced for AI player houses (the neutral and special houses are outside that departure):
  - after giving up (claim 23 row 2) the unit queues *area guard*, where a human-controlled
    owner's unit queues *guard*,
  - after giving up in a skirmish or multiplayer game, its house gives it a new destination, and
  - the threat coefficient of a unit that is a team member whose team type is marked to avoid
    threats is exactly 1.0 (Terms).

  Our engine reproduces none of them for an *AI house*, which has the unit rules of a
  human-controlled house and no teams (glossary *AI house*, *team member*). A unit of an *AI house*
  queues *guard*, gets no destination from its house and always uses its own coefficient. The neutral and special
  houses are not AI houses and keep the original's non-human-owner rules (glossary *AI house*).
  There is no switch. Our AI may reproduce the original's result through ordinary orders. The measurement
  scenarios that use a computer-controlled house measure the original.
- **Departure D7 (accepted): order among exactly equal entries.** Claim 12a gives our engine's
  order among pending entries of exactly the same cost: the entry recorded first goes first, and in
  the coarse search a route's new routes are recorded in the order of the sectors' first cells. The
  original breaks the same ties by an order of its own, which depends on how it keeps its pending
  entries and on the order in which its labelling of the map finds connections (claim 13b).
  Reproducing it would fix the data structures of both searches and would need the original's
  labelling order, which this spec does not establish (departure D4). Our engine does not reproduce
  it, and there is no switch. Routes can therefore differ from the original's where the search meets
  an exact tie: rarely in the cell search (T4), often in the coarse search on open ground. In the
  same way our engine keeps one connection per pair of sectors (claim 13b), where the original can
  hold a diagonal connection and a link's straight one side by side. That changes no route cost,
  only the count of pending routes towards the limit of 9999 (claim 13). There is no quirk switch,
  because a switch would need the original's order stated as behaviour, which this spec cannot do.
  The project lead accepted this departure (decision of 2026-10-10). Route costs and every formula
  stay exact.
- **Threat with a zero coefficient.** A unit with threat coefficient 0 ignores threat in every
  claim: the coarse threat term is skipped and every threat test of claims 17 and 18 passes.
- **`quirks.threat_rebuild_counts_friends`** (default: on). The rebuild of a house's threat records
  after it makes or breaks an alliance (claim 16a) counts every building on the map, its own and
  its allies' included, and, for a computer-controlled house, the vehicles, infantry units and
  aircraft of its allies. No later event takes those contributions away, so they stay in the
  records ("stale threat") until the next rebuild, even after the objects have moved, been
  destroyed or left the map. A unit with a threat coefficient can therefore avoid its own base or
  a spot where an allied unit once stood. Off: the rebuild excludes the house's own objects and objects of houses it allies.
  Backing: `measurable` (P30 part c and its variant c2 for the counted allied vehicle and
  building at game start, part g for the house's own building counted at a rebuild in play and
  kept after it leaves the map); `unmeasured` for the allies' infantry units and aircraft (claim
  16a gives the reason). Confidence: medium, until those parts have run.
- **`quirks.negative_threat_field`** (default: on). A type whose threat field is negative takes its amount away from the
  records when it is placed and adds it back when it is removed, with the floor at 0 of claim 16a,
  so it can lower another object's threat for as long as it stands near it, and can leave a record
  higher than before after it leaves (the floor lost part of the subtraction). The rebuild skips
  it (only amounts above 0 count). Stock data is not known to use negative values, but a mod may
  set one, and the record left higher after such an object leaves is a defect a mod may want to
  turn off. Off: a negative threat field is read as 0, so an object of such a type has no
  threat amount and changes no record. Backing: `unmeasured` (claim 16a gives the reason).
  Confidence: medium.
- **`quirks.aircraft_threat_stays_put`** (default: on). An aircraft's threat stays at the block
  where it was placed on the map while it flies and after it lands elsewhere, and is taken away
  at the block of its last landing cell, or of the cell whose coordinates are both 0 for one that
  was placed in the air and never landed (claim 16a). Its placement block keeps stale threat, and
  the block where the amount is taken away loses threat that other objects put there. The
  corridor exemption that an aircraft placed in the air gives around its placement cell stays
  after its first landing in the same way (claim 13). Off: an aircraft placed in the air is
  recorded in its placement cell, and when it lands in a cell of another block its amount moves
  to that block as a vehicle's does, so its amount and its exemption are always taken away where
  they were added. Backing: `measurable` (P30 parts h1 and h2) for where the amount stays; `unmeasured` for
  where it is taken away, for the lowering of other objects' threat at that block and for the
  corridor exemption, which P30 part h neither sets up nor records. Confidence: medium for an
  aircraft placed on the ground; low for everything the switch says about an aircraft placed in
  the air (where its amount stays and is taken away, and its corridor exemption), as claim 16a
  and S-0009 claim 38 rate the aircraft placed in the air.

## Import mapping

Keys are read from the rules file and its rules layers (S-0006). Booleans, decimals, integers and
named values follow S-0003. An absent key keeps the held value (S-0006 claim 3). The absent-key
defaults are the values before any data is read. Type fields start from them in every game, because
every type is created anew at each game start. The game-wide keys below are set to their
defaults only once, when the program starts, so the default holds in the first game of a session,
and in a later game an absent key keeps the value an earlier game's rules layers left
(S-0006 claim 3). The keys behind the passability category (land types, overlays, terrain objects,
firestorm walls and laser fences) and the locomotion class are S-0211's.

**Vehicle, infantry and aircraft type keys** (in the type's section)

| Key | Form | Absent-key default | Use here |
|---|---|---|---|
| `MovementZone` | named value as in S-0211's import mapping (text cut to 31 characters, letter case ignored, the spelling `subterannean`) | `normal` | movement zone (claims 4, 6, 13, 25), including the aircraft factory edge-entry row. The final house-selected search instead always uses *normal* (claim 23a). A value outside the vocabulary: S-0211 claim 4, departure D1 |
| `SpeedType` | named locomotion class (S-0211 Import mapping) | S-0211 claim 3: *foot* for infantry, *winged* for aircraft, *track* for crusher vehicles and *wheel* for other vehicles | aircraft factory edge-entry search uses the aircraft's class, *winged* counted as *track*. The final house-selected search instead always uses *track* (claim 23a) |
| `ThreatAvoidanceCoefficient` | decimal, single precision (S-0003 claim 6), held as a double | 0.0 | threat coefficient *k* (claims 13, 17, 18), copied to the unit when it enters play |
| `ThreatPosed` (building types too) | integer | 0 | the type's threat field: threat amount of claim 16a |
| `Speed` | integer, converted to the speed number 0 to 255 (Terms): by S-0207 claim 1 for a vehicle type, by S-0208 F4 for an infantry type, each with its rule for the value −1 | speed number 0 | speed comparison of claim 19 |
| `IsTrain` (vehicle types) | boolean | false | marks the type as a train: train rules (claims 7, 8, 14, 16, 20, 24). Stock data is not known to set it |
| `JumpJet` | boolean | false | an infantry type with this flag uses movement zone *infantry* for the coarse search of a path computation (claim 13), whatever its own movement zone. The estimate of claim 24 uses its own movement zone (Open questions) |
| `Passive` (vehicle types only) | boolean | false | exception of claim 15 |

**Aircraft landing search inputs** (claim 25a; caller selection belongs to S-0501 claim 41)

| Input | Form and absent-key rule | Use here |
|---|---|---|
| Search dimensions, locomotion class and movement zone | executable constants, no INI key or absent-key case | building search is 3 × 3 cells, *foot* and *normal*, regardless of the aircraft type's `SpeedType` or `MovementZone`; its replacement/fallback searches are 1 × 1 |
| Land multipliers, wall-overlay flags and map cells | imported forms and absent-key defaults belong to S-0211 Import mapping and S-0009 claims 4 and 5 | nine-cell class checks, reach regions and top-left height-aware usable-area test; no building foundation or placement flags are imported for this search |

**Team type key**

| Key | Form | Absent-key default | Use here |
|---|---|---|---|
| `AvoidThreats` | boolean | false | describes the original only: when true, the original's team members use *k* = 1.0. Our engine does not read it for an *AI house* (Departure D5) |

**Game-wide keys**

| Section | Key | Form | Absent-key default | Use here |
|---|---|---|---|---|
| `[AI]` | `PathDelay` | decimal, game minutes, single precision held as a double | the binary64 value nearest 0.016 | *P* (F3, claim 21) |
| `[General]` | `ThreatPerOccupant` | integer | 5 (the original's built-in default, used only when no rules layer sets the key; `unmeasured`, as no scenario here leaves the key out, medium) | per-occupant threat of an occupied building (claim 16a) |

The close-enough distance *C* (claim 24) and the blockage delay *B* (claim 22) are imported as
S-0209's Import mapping says.

**Bridge data** (read by other specs; listed for the uses here)

| Data | Owner | Use here |
|---|---|---|
| `BridgeSet`, `WoodBridgeSet` (the bridge and wooden-bridge roles of the theater) | S-0010 Import mapping, including its absent-key defaults; the integer tile number minus the role's first tile number gives the role-relative place that selects both ends' side cells (claim 13b, Q-0088) | the bridge spans of S-0010 claim 25, which are the bridge links (claim 3); the deck read of claim 4 (not-enabled row); the bridge ends (Terms) and anchors (F8) |
| map overlays at the high-bridge overlay indices 24, 25, 237 and 238 (overlay type stream, S-0009 claim 12) | S-0211 claim 15 | bridge cells, the deck-entry mark and the bridge axis (Terms), and so claims 4, 7 and 10 |

**Map tunnel list**

| Map section | Entry | Use here |
|---|---|---|
| `[Tubes]` | one entry per tunnel: entrance cell x and y, a facing, exit cell x and y, then a list of directions | tunnel entrance and tunnel exit (claims 7 and 9). The direction list does not enter pathfinding. The facing enters only which further cells a tunnel's link joins in the coarse search (claim 13b, not established) |

## Measurement scenarios

**Common setup.** The glossary's *measurement setup*: unmodified install, stock data, no engine add-on and no mods (the named test files of a scenario excepted), a purpose-made test map unless stated,
skirmish, game speed 2 unless stated otherwise (results are in ticks and cells). One
human-controlled house with the test units and one bystander computer player isolated from the
scenario's observed area. The observed area comprises the test routes, objects and search
candidates. Void and repeat a run if an unnamed bystander object enters that area or acts on a
test object, or if the bystander is defeated before observation ends. Where a scenario names
computer-controlled test units or teams, those are additional named test objects; their rules
are the original's, not those of our engine's *AI houses* (Edge cases and quirks).
Record, for each tick, each test unit's cell and destination,
its current behaviour, whether it is a team member, and the tick count. Also record each test
unit's stored path. Where a scenario changes a rules value it does so
through the test map's override section (a named test file), named in the scenario and removed after
the run. A land-type override gives every multiplier key the scenario needs, because the rest of that
row resets (S-0006 claim 5). No stock value is assumed: expected numbers are computed from the fields'
values at measurement time. A computer-controlled team member gets its order through its team's
script, the team being made by a trigger action that creates a team. Use one run per variant unless stated. Claim 25's nearby-cell search takes no random draws.
P16 run (u) checks this separately from other activity in the tick.

**Shared no-reach-region comparison (claim 4).** S-0309 claims 7 and 11, scenario A11 part j,
runs 3 and 5, supply the setup, target recorder, expected picks and falsification for this rule:
a map-placed infantry searcher on a cliff cell of no reach region picks the enemy on another
such cell, not the enemy on open ground. This tests the shared comparison through target
acquisition, not a path computation's gate or subsequent route search.

- **P1 (claims 13 and 17; secondary check: claims 7 to 18): path shape on open ground.**
  - Setup: a flat map of clear land at one height level, no overlays or objects, 40 × 40 cells. One
    wheeled vehicle without a weapon (a test override removes its weapon if its type has one), of a
    type not marked as a crusher and not of movement zone *crusher-all* (a test override changes
    either if needed), and placed by the map with no veterancy gained, so without the crusher
    veterancy ability (S-0210 claim 1). So a wall section
    gives it outcome *blocked* (claim 27). It is started in turn at cells (10, 10),
    (11, 10), (10, 11) and (11, 11) (all four parities of x and y) facing map north.
  - Procedure: from each start, order it to the cells 5 and 2, 2 and 5, 7 and 7, 15 and 1, and 1 and
    15 cells away (in +x and +y), one order per run, starting afresh each time. Repeat with a single
    wall line placed across the direct route (a 7-cell wall perpendicular to the line from start to
    goal, centred on it). Long runs, which serve (b): on a 64 × 64 map of the same kind, from the
    same four starts, order it to the cells 22 and 8 cells away and to the cells 8 and 22 cells
    away, once on open ground and once with a 7-cell wall line across the direct route,
    perpendicular to it and centred 19 cells from the start, so that the route bends around the wall
    after its 20th step.
  - Observable: the sequence of cells entered, the ticks of the unit's path computations, and the
    tick of any refused step.
  - Expected, independent of tie order: (a) the cells entered, except those of straightened
    stretches and those with the corridor exemption (the first cell entered, and in the wall runs
    the cells next to a wall section), lie in a chain of aligned 2 × 2 blocks that connects the
    start block to the goal block (claim 13). Any other cell outside every such chain may occur only
    inside a run of off-chain cells
    whose two neighbours on the route are in the chain and which forms a straight line (a flattened
    V, claim 17) or, within the first 20 steps, a diagonal-then-straight or straight-then-diagonal
    shape (claim 18). (b) No right-angle V of diagonal runs whose two runs both lie after the 20th
    stored step of a path computation remains whose straight replacement would be all clear (claim
    17), unless its second run came from flattening a V of straight runs (its first run was then
    already final). Within the first 20 steps (b) says nothing, because claim 18 works there after
    claim 17 and can reorder or shorten stretches (scenario P9, train run 4, tests its selection). A run in which the unit refuses a step, or computes
    a new path before it arrives other than as it begins the 24th step of a stored path (claim 20),
    has not followed a path that ends at the goal. On this static map there are two known causes: a
    flattened V of straight runs, which shifts the rest of the stored path (claim 17,
    `quirks.flatten_straight_run_v`), and, in a long run whose path is refreshed at its 24th step,
    the dropped first step of the new path (claim 20, `quirks.path_refresh_keeps_old_step`). Such a
    run is left out of (a) and (b), reported, and goes to the secondary
    check. (c) The routes from starts of different parity differ in at least one
    goal case, as confinement to aligned blocks predicts.
  - Secondary check, not a falsifier of any single claim: compare each run's cell sequence with the
    one a reference implementation of claims 7 to 18 with T2 to T4 gives, and have the reference
    implementation log every exact tie it meets between pending entries. A run whose reference
    computation meets no such tie should match exactly, and a mismatch there sends claims 7 to 18
    back for review as a whole. A run with a tie is reported but not judged, because our engine's
    tie order (claim 12a, departure D7) is not the original's.
  - Falsified if: (a) or (b) fails in any run, or the four parities always give identical routes up
    to translation.
- **P2 (claim 8): terrain speed does not steer the path.**
  - Setup: a flat map. Start and goal 20 cells apart on a straight line of rough land. A road runs
    parallel, 3 cells to the side, joined to start and goal. No override.
  - Procedure: order a wheeled vehicle from start to goal.
  - Observable: cells entered.
  - Expected: the vehicle drives straight across the rough land, ignoring the faster road.
  - Falsified if: it uses the road.
  - Void if: at measurement time the test vehicle's land multiplier for rough land is not lower than
    its multiplier for road (S-0211), judged from the fields' values, since the road is then not the
    faster route and the run cannot tell terrain-aware costs from this claim.
- **P3 (claims 1, 2, 4, 6, 23): unreachable destination on an island.**
  - Setup: a land vehicle on the mainland. A small island separated by water, no bridge. Run 1: the
    vehicle belongs to the human-controlled house. Run 2: it belongs to a computer-controlled house
    and is a team member.
  - Procedure: order the vehicle to a cell on the island (through its team's script in run 2, and
    in run 1 through the same script if the interface refuses the order).
  - Observable: whether the vehicle moves at all, the tick its destination is cleared, whether it
    is still a team member afterwards, whether it is given a new destination in the following
    ticks.
  - Expected: no step is taken. The destination is cleared in the tick the path is first requested.
    In run 2 the vehicle stays a team member and gets no new destination from its house. Its
    behaviour afterwards follows S-0215 (falling back to rest) and is not part of this test.
  - Falsified if: the vehicle drives to the shore first, waits before its destination is cleared,
    or (run 2) leaves its team or heads for a new destination of its house, which would mean the
    search-failure rows of claim 23 ran instead of the gate.
- **P4 (claims 1, 3, 4, 6, 10, 15, 23): plateaus, slopes, a building pocket and bridge decks.**
  - Setup: map A: a goal cell on a plateau 2 height levels above the surrounding ground with cliff
    edges and no slope. Map B: the same goal at ground level, enclosed by a ring of the house's own
    buildings with no gap. Run B1 with a vehicle of the human-controlled house, run B2 with a vehicle of a
    computer-controlled house that is a team member. Map C: a goal on a plateau 1 height level above
    the surrounding ground, with cliff edges except for one slope cell at the ground's height level
    whose raised side meets the plateau, so that it leads up onto it. Map C2: as A, with a ramp of
    two slope cells in a row leading up, the first at the ground's height level and the second one
    level higher, each raised on the side towards the plateau. Map C3: as C, but the slope cell
    has the plateau's height level instead of the ground's, so that it stands 1 level above the
    ground in front of it, and the plateau has no other slope. Its orientation is that of map C, and
    by claim 10 only its height level and its being a slope count. Maps C, C2 and C3 are made with the
    map's height and tile data, the slope tiles of a land type the vehicle can enter, and in map C3
    the vehicle is a team member of a computer-controlled house, as in run B2.
    Map D (snow theater) and map E (another theater): the goal enclosed by a ring of one purpose-made terrain-object test type. Test overrides set the type's two theater-specific occupation-pattern fields to controlled values, one equal to 7 and the other not equal to 7 (S-0211 claim 11 and Import mapping). The maps otherwise use identical geometry; no stock terrain-object type or stock occupation-pattern value is used. Order for one infantry unit and one vehicle. Map F: two goal plateaus of clear land 2 height levels above
    the clear land next to them, with no cliff tiles in between (the map's height data alone raises
    them) and no slope. The first plateau is a strip whose only edges with the low ground run across
    cell x (its neighbours at x − 1 and x + 1 are low), the second a strip whose only such edges run
    across cell y. All their other sides are land of a type the vehicle cannot enter. Maps I, F3
    and F5: as map F, with the two plateaus 1, 3 and 5 height levels above the low ground. In maps
    F, I, F3 and F5 the vehicle is a team member of a computer-controlled house, as in run B2, so
    that a failed search shows as a team exit and a failed gate does not. Map K: a river crossed by
    an intact high bridge, the goal on the far bank, the vehicle on the near bank, and no other
    crossing; run 2 has the goal on a free deck cell over the middle of the river. Map K2: a high
    bridge laid as a map's bridge normally is (end tiles of the bridge role and high-bridge overlays
    of index 24 or 237, so that its axis runs along cell x and its north deck row is the one
    without the deck-entry mark, Terms), over dry flat clear ground of height level *h*, the deck at
    *h* + 4. Along the western half of the bridge's south side lies a strip of clear land at height
    level *h* + 4 next to the south deck row, joined to the west bank; beside the eastern half of the
    south deck row the ground stays at *h*. Along the whole north side lies another such strip next
    to the north deck row. The bank cells at each end are at *h* + 4 with a ramp of slope cells down
    to *h*. Runs: (a) the vehicle on the middle deck row over the eastern half, ordered to the ground
    cell at level *h* 2 cells south of the south deck row, in the eastern half; (b) the vehicle on the south strip, ordered onto the south deck
    cell next to it; (c) the vehicle on the north strip, ordered onto the north deck cell that lies
    beside the strip cell 3 cells east of it; (d) the vehicle on the middle deck row over the western half, ordered onto the south strip cell
    beside the south deck row. Map H: the
    2 × 2 configuration of claim 3, made with a crushable overlay on the cell (x, y), every other
    neighbour of the four cells being land of a category the test vehicle's movement zone cannot use
    (for example category *blocked*). A crusher vehicle (movement zone allowing *open* and
    *crushable*) stands at (x, y + 1).
  - Procedure: order a vehicle to the goal in each map (both goals in map F, one per run). In map
    H, order the crusher vehicle onto (x, y).
  - Observable: whether the vehicle moves, how far it gets, the ticks until its destination is
    cleared, team membership and new destinations as in P3.
  - Expected: A: no movement, cleared at once (different reach regions). B: the reach regions agree
    (buildings are *open*), so the search runs and fails. The vehicle stops and gives up when more
    than 1 cell away. In run B2 it leaves its team and its house gives it a new destination, which
    P3 run 2 does not show. C and C2: the vehicle drives up the slope or the ramp to the goal (in
    C2 each step between ramp cells, and from the upper ramp cell onto the plateau, is a difference
    of 1 whose lower cell is a slope). C3: the gate passes (a difference of 1 joins), but the only
    step up, from the ground onto the slope cell, has a flat lower cell, so it is refused (claim
    10): the search fails, the vehicle never reaches the plateau, and it leaves its team. D/E: the infantry unit
    reaches the goal exactly when the theater's occupation pattern is not the full one. F: the
    plateau whose edge runs across cell x passes the gate (claim 3's x rule), so the vehicle
    searches, may drive towards the edge, fails, and leaves its team. The other is refused at once
    without movement and stays in its team. I: both plateaus pass the gate (a difference of 1 joins
    in every direction), and both searches fail because a step of 1 level needs a slope (claim 10):
    team exit in both runs. F3: as F (a difference of 3 still joins along cell x, and the step is
    refused). F5: both refused at once, no team exit. K: the vehicle drives over the bridge, onto and
    off its deck (steps of 4 levels at the bridge's ends), and reaches the goal; in run 2 the gate
    passes (the deck is read through the enabled link as the bank's reach region, claim 4) and the
    vehicle stops on the deck cell. K2 (claim 10): (a) the vehicle never steps from the deck straight
    down onto a cell at level *h*, although such cells lie beside the deck: it leaves the deck
    sideways onto a strip (a step of 4 levels off a deck needs nothing of the cell) or at a bridge
    end, and reaches the goal by a ramp; (b) it steps
    sideways onto the south deck cell (marked) at once; (c) it never enters the north deck cell. Two outcomes
    are expected, each telling something: it drives along the north strip and stops on the strip
    cell beside the deck cell (the unmarked row can be entered neither from the strip nor from the
    deck, so the search ends beside the destination, claim 15), or its destination is cleared at
    once with no step (the deck read of that row found no link within 1 cell, claim 4, row "none
    found"); (d) it steps sideways off
    the deck onto the strip.
    H (original, low confidence): the order is dropped at once by the gate, although the rule of
    claim 3 would let the vehicle drive onto the cell. This result supports departure D4's
    description of the original.
  - Falsified if: A behaves like B, B is refused at once like A, B2 shows no team exit or new
    destination, C or C2 fails, the vehicle of C3 reaches the
    plateau (which would mean a slope as the higher cell also allows the step) or is refused without
    a team exit, D/E disagree with S-0211's occupation rule, the two plateaus of F behave
    alike, a vehicle reaches a plateau in map I, F3 or F5, either plateau of I is refused without a
    team exit, the two plateaus of F3 behave alike, either plateau of F5 gives a team exit, or the
    vehicle of K does not reach the goal or is refused at once in run 2. K2: the vehicle of (a)
    steps from the deck onto a cell at level *h*; (b) does not enter the south deck cell from the
    strip; (c) enters the north deck cell; (d) does not step off sideways. K2, except run (c), is void if a run is refused at once by
    the gate because a deck read (the vehicle's on the deck, or a destination deck cell) found no
    link within 1 cell (claim 4), or if the recorder shows that the north deck row carries the mark after all (the
    bridge was laid with overlays of another index). For map H, the vehicle driving onto (x, y) falsifies the
    corner-case description of claim 3 (and makes D4 unnecessary).
- **P5 (claim 20): path refresh, stale paths and chase re-paths.**
  - Setup: map 1: a flat corridor map, a 1-cell-wide road between impassable land, 60 cells long,
    then open ground. The road crosses a river on a high bridge whose near end lies 30 cells from the
    road's start (run 1) or 15 cells (run 2). Map 2: flat open clear ground, 40 cells long along map
    north, with a test vehicle at the south end on column *x* and three friendly vehicles waiting
    off to the side. Map 3 (run 3): flat open clear ground, 100 cells long along map north and 3
    cells wide, with the test vehicle at the south end on column *x* and, 30 cells ahead of it on the
    same column, a friendly vehicle of a type whose speed number is lower than the test vehicle's.
  - Procedure: run 1 (map 1): order a vehicle from the road's start to the far open ground. At the
    moment the vehicle has taken 5 steps, destroy the bridge (a trigger action that detonates a weapon
    at a waypoint marker on it, as in P8). Run 1i: as run 1 with an infantry unit instead of the
    vehicle (walk locomotor). Run 2: as run 1, with the bridge 15 cells ahead. Run 3
    (map 3): order the test vehicle to move to the friendly vehicle (an order whose destination is
    that vehicle) and order the friendly vehicle to the north end of the map, so that it drives
    slowly away along the column, and record the ticks of path recomputation
    (direction changes, or a change of the recorded stored path) and the separation of the two
    vehicles at each one. Run 4 (map 2): order the test vehicle 30 cells straight north. Right after
    the order, drive the three friendly vehicles onto the cells (*x* − 1, *y*₂₅), (*x*, *y*₂₅) and
    (*x* + 1, *y*₂₅), where *y*₂₅ is the row 25 cells north of the start, so that they stand there
    before the test vehicle has taken 20 steps. The first path (computed at the order) cannot see
    them, and its 24 stored steps end on row 24.
  - Observable: runs 1 and 2: the cell at which the vehicle stops and the tick its destination is
    cleared. Run 1i: the first cell at which the infantry unit stops. Run 3: the number of steps between successive recomputations, and the separation in
    cells at the earlier one. Run 4: the cell the test vehicle enters from row 23, and the one
    after it.
  - Expected: run 2: it drives on until its stored path, which still crosses the bridge, reaches the
    gap. There the next step is refused, the fresh search fails the gate and the destination is
    cleared. Run 1: the bridge lies beyond its 24 stored steps. It computes a new path at once as it
    begins its 24th step, from the cell reached after 23 steps (the heading point is far more than
    512 leptons away). That computation fails the gate, and the destination is cleared there. The
    vehicle may still finish the step it had begun. What decides is that the destination is cleared
    before the vehicle reaches the cell 24 steps from the start. Run 1i: the walker makes no
    recomputation as its last stored step begins (claim 20), so it walks all 24 stored steps and
    first stops on the cell 24 steps from the start, where its fresh search fails the gate.
    Run 3: the number of steps between
    recomputations is ⌊separation in leptons / 256⌋ at the earlier recomputation, within 2 steps
    (the chased vehicle moves a little meanwhile), and never more than 24. Run 4 (low
    confidence): the recomputation from row 23 finds a new path whose first step is diagonal, to
    (*x* + 1, row 24), the cheapest way past the three vehicles (the mirror-image detour has the same
    length and loses on the direction terms). The vehicle still takes the old straight step to
    (*x*, row 24), and then the new path's second step, diagonal from there towards
    (*x* + 1, row 25), which holds a friendly vehicle.
  - Falsified if: in run 1 the destination is cleared before the 23rd step is done, or only once
    the vehicle has reached the cell 24 steps from the start (which would mean the path is recomputed
    only when it is used up), or in run 1i the infantry unit first stops before the cell 24 steps
    from the start (which would mean the walk locomotor recomputes early too), or in run 2 before the vehicle reaches the gap. In run 3 the steps
    between recomputations are 24 whatever the separation, or differ from ⌊separation in leptons /
    256⌋ by more than 2 steps. In run 4 the vehicle moves from row 23 directly to
    (*x* + 1, row 24) and on to (*x* + 2, row 25), which would mean the new path's first step is
    taken (`quirks.path_refresh_keeps_old_step` off).
- **P6 (claims 19, 21, 22, 27): retry delay and blockage modes.**
  - Setup: test override in the test map's override section for the game-wide AI fields (Import
    mapping here and in S-0209): the path-retry delay set to a value *P* with trunc(*P* × 900) = 90
    ticks, and the blockage-delay field (an integer number of ticks, S-0209 Import mapping) set to
    *B* = 300. A 1-cell-wide corridor with a side loop (a longer
    route around) of 12 cells. A friendly vehicle held in the corridor, and a test vehicle ordered
    through it from behind.
  - Procedure: record until the test vehicle takes the loop. Repeat 10 times with the held vehicle
    swapped for a slowly moving one that shuttles back and forth (statistical runs, since the timing
    of the encounters varies).
  - Observable: ticks from first blockage to each recomputation (visible as a change of the stored
    route or of the step direction), and the tick the loop is taken.
  - Expected (main case, the shuttling blocker): recomputations at least 90 ticks apart. The loop
    is not taken before 300 ticks after the first blockage while the blocker keeps counting as
    moving. Over the runs in which the blocker counts as moving until the loop is taken, the median
    time to take the loop lies between 300 and 390 ticks. Held blocker (low
    confidence, since a vehicle that stands still without a destination gives *stationary
    friendly*, whose handling belongs to S-0209 and the movement specs): the loop is taken at the
    first recomputation.
  - Falsified if: two recomputations come less than 90 ticks apart, or the loop is taken before
    the blockage delay in the shuttling case, or, over the shuttling runs in which the blocker
    counted as moving until the loop was taken (recorder), the median time to take the loop lies
    outside 300 to 390 ticks. For the held blocker (low confidence), falsified if
    the loop is not taken at the first recomputation, which would show that a vehicle without a
    destination is not handled as in the main case.
- **P7 (claims 6, 15, 21, 23, 24, 25, 27): destination substitution and blocked destinations.**
  - Setup: a flat map with a 2 × 2 building of the human house, a vehicle 10 cells away. For runs 2
    to 4, a friendly vehicle stands on a guard behaviour with no destination, so a vehicle mover gets
    *stationary friendly* for its cell (claim 27). For runs 5 and 6, a test override gives one land
    type (say rough) a track multiplier of 0 and a wheel multiplier of 1.0, and a single cell of that
    land type lies in clear land. The mover is a tracked vehicle. For runs 7 and 7b, a small island
    holds a building of the human house and clear cells. For run 7b, an infantry unit of the
    human house stands on clear mainland ground 10 cells from the island's shore, as the only
    member of a team that a trigger creates at the start, whose script's one step is a move to a
    clear cell of the island. For runs 8 and 8b, a separate flat
    map: a straight barrier along
    row *y*₀ from *x* − 15 to *x* + 15, one cell thick, with *x* even and *y*₀ odd. The barrier cells
    must have a passability category that the mover's movement zone cannot use (for example
    water), not merely a land multiplier of 0, because the coarse search reads categories only
    (claim 13). Open clear ground lies on both sides and around both ends. The destination cell
    *D* = (*x*, *y*₀ + 1) lies beside the barrier's middle. Six friendly vehicles on guard with no
    destination stand on *D* and on the five cells of *D*'s ring 1 that are not barrier cells:
    (*x* ± 1, *y*₀ + 1) and (*x* − 1 … *x* + 1, *y*₀ + 2). The mover stands at (*x*, *y*₀ − 10), on the
    other side. A test override sets the close-enough field to 1.375 cells (352 leptons). In run 8b
    the barrier has a one-cell gap of clear land at (*x* + 2, *y*₀). For run 9, a separate flat map
    of clear land: a friendly vehicle on guard with no destination stands on *D* = (*x*, *y*), three
    wall sections of the human house stand on (*x* − 1, *y* + 1), (*x*, *y* + 1) and
    (*x* + 1, *y* + 1), and the mover, whose movement zone a test override sets to *crusher-all*,
    stands at (*x*, *y* + 10); the close-enough field is set to 1.375 cells as for run 8. For run 10,
    a river crossed by a high bridge at least 12 cells long, with a friendly vehicle on guard with
    no destination standing on the deck of the bridge cell *Q* over the middle of the river, and the
    close-enough field set to 1.375 cells. Orders onto an occupied cell are plain move orders to
    that cell (S-0213).
  - Procedure: run 1: order the vehicle onto a building cell. Run 2: onto the cell of the stationary
    friendly vehicle, 10 cells away. Run 3: as run 2 but from 2 cells away. Run 3b: as run 3, with a
    test override setting the close-enough field to 2.125 cells (544 leptons), once with both
    vehicles on flat ground at height level 0 and once with both on a wide flat plateau at height
    level 3. Run 4: as run 3, with a test override setting the close-enough field to half a cell (128
    leptons). Run 5: order the tracked vehicle onto the single cell of the overridden land type, from
    10 cells away. Run 5b: as run 5, with a test override setting the close-enough field to half a
    cell (128 leptons). Run 6: as run 5, with a further override marking the tracked vehicle's type
    passive. Run 7: order a vehicle on the mainland onto the island building's cell. Run 7b:
    order an infantry unit on the mainland onto a clear cell of the island, through a team script,
    because an interface order would have the island cell replaced by a mainland cell before any
    path computation (S-0214 claim 11). Runs 8, 8b and
    9: order the mover onto *D*. Run 10: order a wheeled vehicle onto *Q*, once from the ground of
    the near bank 10 cells from *Q* along the bridge's axis, and once (control) from the deck of the
    same bridge, 5 cells from *Q*.
  - Observable: the cell where the vehicle stops, the tick its destination is cleared, team and
    behaviour changes as in P3. In runs 8, 8b, 9 and 10, the recorded destination in the ticks after
    the order. In run 7b, the recorded destination and whether the unit has movement in progress,
    in each tick after the order.
  - Expected: run 1: a cell next to the building, the qualifying ring-1 cell nearest to the
    vehicle's start (on equal distance, the first in the ring order of claim 25). The building's
    other cells are in ring 1 too but do not qualify, since their occupancy record is not empty
    (claim 25). Run 2: an
    unoccupied cell next to the occupied cell, nearest to the vehicle. Run 3: no substitution
    (within the close-enough distance). The vehicle stops next to the occupied cell. Run 3b: at
    height level 0 the distance is 512 leptons, below 544, so there is no substitution. On the
    plateau the vehicle's world height of 312 leptons counts in full (F6a), the distance becomes 599
    leptons (the table square root of 359488), above 544, and the vehicle is sent to an unoccupied
    ring-1 cell. Run 4: substitution takes place (the unit is now farther than the close-enough
    distance, and a ring-1 cell is closer to the destination than the unit is): the vehicle stops on
    the unoccupied ring-1 cell nearest to it. Run 5: no substitution (the cell holds no building).
    The vehicle stops on a neighbour of the cell (claim 15). Its next search follows at once (a
    successful search leaves no delay), is made from beside the cell and fails. The vehicle stays
    there, stays in any team, queues no behaviour, and its destination is cleared in the tick of
    that failed search (claim 23, row 3). Run 5b: the same, although the vehicle now stands farther
    from the cell's centre than the close-enough distance. Run 6: the passive vehicle gets no path
    (claim 15 does not apply, the search exhausts the reach region and fails), takes no step, and
    gives up (claim 23). Run 7: no step. The destination is cleared at once by the gate, with no
    substitution (claims 6 and 24). Run 7b: the infantry unit takes no step. In its first movement
    update after the order its destination is cleared and its movement in progress ends. It stays
    in its team and no behaviour is queued (claim 23, the gate row). Runs 8 and 8b: the
    unit's distance to *D* is 11 cells, far above
    the close-enough distance, so the nearby-cell search runs. Ring 1 holds no qualifying cell
    (barrier or occupied), so the scan ends with ring 2, and its qualifying cell nearest to the
    mover is *F* = (*x*, *y*₀ − 1), on the mover's side, 512 leptons from *D* (closer than the
    unit). The Chebyshev distance from *F* to *D* is 2, so *F* is adopted only if *E* ≤ 8. Run 8:
    the only route from *F* to *D* leads around a barrier end, through at least 17 level-0 sectors,
    so *E* ≥ 32 (or the coarse search fails, if the upper levels confine it), and *F* is not
    adopted: the recorded destination stays *D* and the mover drives off towards a barrier end.
    (Once it has passed a barrier end, a later path computation may adopt a ring-2 cell on *D*'s
    side, with a short route. That is not part of this test.) Run 8b: with the parities of the
    setup, the route through the gap passes 3 level-0 sectors, and with any alignment at most 5, so
    *E* ≤ max(2, 2 × 5 − 2) = 8: the recorded destination becomes *F* in the tick of the order's
    path computation, and the mover stops on *F*. Run 9: the mover's distance to *D* is 10 cells, so
    the nearby-cell search runs, with the simplified movement zone *normal*, which may not enter a
    wall cell. Ring 1 then holds five qualifying cells. The nearest to the mover are (*x* − 1, *y*)
    and (*x* + 1, *y*), at equal distance, and (*x* − 1, *y*) comes first in scan order. It is 256
    leptons from *D*, and its estimate is at most 2 cells. So the recorded destination becomes
    (*x* − 1, *y*) in the tick of the order's path computation. Run 10 (low confidence for the bank
    cell chosen): from the ground, the destination's level is the height level of *Q* without the
    deck, so no bridge cell qualifies (each counts 4 levels higher) and the river's water cells are
    not enterable by class. The recorded destination either stays *Q* or becomes a cell of a bank,
    never a bridge cell. Control, from the deck: the destination's level includes the deck, the
    deck cells next to *Q* qualify, and the recorded destination becomes the deck cell next to *Q*
    on the mover's side (256 leptons from *Q*, estimate 1 cell by F8).
  - Falsified if: in runs 1 to 4 the vehicle stops elsewhere, or a different ring is used. Run 3b
    shows the same result on the plateau as at level 0 (which would mean the destination's height is
    taken into account). Run 4 shows no substitution (which would mean the found cell must lie
    within the close-enough distance of the destination, not merely closer than the unit). Run 5 or
    5b enters the cell, is given a substitute, or keeps its destination and goes on searching (which
    would mean the drive locomotor does not clear it), or run 5b keeps it while run 5 clears it
    (which would mean the close-enough distance matters). Run 6 moves towards the cell. Run 7
    drives to a cell near the island, which would mean substitution came before the gate. Run 7b
    walks towards the island, or keeps its destination or its movement in progress after that
    update (which would mean the walk locomotor does not clear it at the gate). Run 8
    changes the destination to *F*, or to any other cell on the mover's side of the barrier, before
    the mover has passed a barrier end (which would mean there is no route-length test, or a much
    looser one). Run 8b keeps *D* as its destination (which would mean the test refuses a short
    route, or the scan picks a different cell). Run 9 changes the destination to one of the wall
    cells (which would mean the class rule uses the mover's own movement zone), or to a cell other
    than (*x* − 1, *y*). Run 10 from the ground changes the destination to a bridge cell (which would
    mean the destination's level is its deck, as in the search: the behaviour of
    `quirks.substitution_centre_ground_level` off), or the control does not change it to the deck
    cell next to *Q* on the mover's side.
  - Void if: runs 2 and 3 set no close-enough override, so their expectations rest on the
    close-enough distance *C* at measurement time (S-0209 Import mapping, judged from the field's
    value). The unit's distance *D*ᵤ to the destination (F6a, at height level 0) is 512 leptons in
    run 3 (2 cells) and 2560 leptons in run 2 (10 cells). Run 3 is void if *C* is below 512 leptons,
    and run 2 is void if *C* is not below 2560 leptons.
- **P8 (claims 3, 5, 6, 23): bridge destroyed.**
  - Setup: a river with one high bridge. A vehicle on the near bank ordered to a far-bank cell.
  - Procedure: destroy the bridge (a trigger action that detonates a weapon at a waypoint marker on
    the bridge) while the vehicle approaches, at least 30 cells from the bridge.
  - Observable: the tick the vehicle's destination is cleared, relative to the tick of the bridge's
    destruction, and the cells entered after the destruction.
  - Expected: at its next path computation after the destruction (at the latest when its stored
    path of at most 24 steps runs out), the vehicle fails the gate: its destination is cleared at
    once and it takes no step onto the bridge cells. As in P3, it does not leave a team and its house
    gives it no new destination.
  - Falsified if: it keeps driving to the river bank and waits there, or it searches and gives up
    with the team exit of claim 23.
- **P9 (claims 9, 16, 18, 20, 26, 27): tunnels, trains and follower cars (test data).**
  - Setup: named test map containing a tunnel in its tunnel list, a test vehicle type marked as a
    train (override section), and a leading vehicle with two follower cars linked in the map's
    vehicle section.
  - Procedure: order a vehicle to a cell beyond the tunnel. Train runs 1 to 3: the train type stands
    at (*x*, *y*) on open clear ground with no other object within 8 cells, facing map north. Order
    it onto (*x*, *y* + 1), the cell straight behind it (run 1), onto (*x*, *y* + 6) (run 2) and
    onto (*x* + 2, *y* + 6) (run 3), placed afresh for each run. Train run 4: as runs 1 to 3, except
    for wall sections of the human house on (*x* − 2, *y*), (*x* − 1, *y*), (*x* + 1, *y*), (*x*,
    *y* + 2) and (*x*, *y* + 3); order the train onto (*x* + 1, *y* + 4). Order the leading vehicle
    to a cell 10 cells away in a straight line along +x, and in a second run to a cell 10 cells
    along +x and 4 along +y. Train refresh run: on the map of P5 run 4 without the three friendly
    vehicles, with a construction yard (a building that produces buildings) of the human house
    beside the route, order the train type 30 cells straight north from (*x*, *y*), facing north.
    Before it has taken 20 steps, build three wall sections of the human house on (*x* − 1, *y*₂₅),
    (*x*, *y*₂₅) and (*x* + 1, *y*₂₅) (*y*₂₅ as in P5). Repeat the train refresh run with an
    ordinary vehicle type, not a train, as control. Both the train type and the control type are
    vehicle types without a weapon (a test override removes it if needed), neither marked as a
    crusher nor of movement zone *crusher-all*, so that the wall sections give both of them outcome
    *blocked* (claim 27).
  - Observable: whether the tunnel is used, the cells the train enters in train runs 1 to 4 and the
    tick its destination is cleared in run 1, the destinations and final cells of the two follower
    cars. Train refresh run: the cells entered from row 23 on, and whether the stored path is
    emptied by a refused step (recorder).
  - Map layout for the tunnel: an overland route of *n* clear straight steps, and a tunnel route of
    *a* clear straight steps to the entrance, a tunnel of Chebyshev length *t* and *e* clear straight
    steps after the exit, chosen so that *n* − (*a* + *t* + *e*) = 2.
  - Expected: the tunnel is used. With an unmarked entrance the tunnel route costs *a* + *t* + *e*,
    2 cost units less than the overland route (direction terms aside, at most 0.008 per step). Were
    the entrance marked, the step onto it would cost 4 instead of 1 and the overland route would win
    by 1. Train run 1: the destination is one of the three cells the search may not enter (claim
    16), so the search fails although the cell is clear: the train takes no step, and its
    destination is cleared in the tick of the order's path computation (claim 23, row 3). Train run
    2: the search finds the step directions 2, 3, 5, 5, 4, 4, 4 (Terms: 2 is east, 3 south-east, 4
    south, 5 south-west), 7 steps for a Chebyshev distance of 6. Claim 17 flattens the pair 3, 5 to
    4, 4, and claim 18 shortens the opening 2, 4 to a single step 3. The train enters 6 cells: (*x*
    + 1, *y* + 1), (*x* + 1, *y* + 2), (*x*, *y* + 3), (*x*, *y* + 4), (*x*, *y* + 5), (*x*, *y* +
    6). Train run 3: the search finds 2, 3, 4, 4, 4, 4, 4, and claim 18 shortens the opening 2, 3, 4
    to 3, 3. The train enters 6 cells: (*x* + 1, *y* + 1), (*x* + 2, *y* + 2), then (*x* + 2, *y* +
    3) to (*x* + 2, *y* + 6). Train run 4: the search finds 1, 3, 4, 5, 4, 4; claim 17 leaves it,
    because flattening the pair 1, 3 would enter the wall cell (*x* + 1, *y*); and claim 18 gives 3,
    4, 4, 4 (claim 18, example (a)). The train enters 4 cells: (*x* + 1, *y* + 1) to (*x* + 1, *y* +
    4). In runs 2 to 4 the first cell entered lies behind the train (claim 16: the exclusion holds
    for the search only). No exact tie of *f* arises in these searches. The cars end on the cells 1
    and 2 steps back from the leader's destination against the direction of the last step of the
    leader's path: along −x in the first run, and in the second run against whatever the last step
    of the found path is (which a reference implementation of claims 7 to 18 gives for the map).
    Train refresh run: the walls give outcome *blocked* (claim 27), the only outcome a train's
    search avoids (claim 8). The control behaves as in P5 run 4: as it begins the step from row 23
    to (*x*, row 24) it computes a new path from row 23, whose first step is diagonal to (*x* + 1,
    row 24). It still takes the old step and then the new path's second step, diagonal towards (*x*
    + 1, row 25), a wall cell: that step is refused, its stored path is emptied and a fresh search
    follows (claim 22). The train computes its new path from (*x*, row 24), where the step in
    progress leads, and keeps that step: from (*x*, row 24) it steps sideways and passes the walls
    without a refused step.
  - Falsified if: the vehicle drives overland (which would mean the entrance, or the tunnel step,
    costs more than claim 9 says), the train of run 1 reaches its destination by any route (claim
    16), the train of run 2, 3 or 4 enters other cells than expected (7 or more cells in run 2 or 3
    would mean that claim 18 does not shorten a detour at the start of a path, or that a stored
    first step cannot enter the cells the search excluded; in run 4, the 5 cells (*x* + 1, *y* + 1),
    (*x* + 2, *y* + 1), (*x* + 1, *y* + 2), (*x* + 1, *y* + 3), (*x* + 1, *y* + 4) would contradict
    `quirks.detour_mirrored_reference`, and any other sequence the stretch selection of claim 18),
    or the cars' cells differ from the expectation. In the train refresh run, the train's stored
    path is emptied by a refused step into a wall cell (which would mean a train drops a step like
    other vehicles). If the control shows no refused step either,
    `quirks.path_refresh_keeps_old_step` is contradicted (P5 run 4) and the train refresh run says
    nothing about trains.
- **P10 (claim 13): threat avoidance.**
  - Setup: test override giving a vehicle type a threat coefficient of 1.0, and an otherwise equal
    type with 0, both of the human house. Two routes from the start to a goal, separated by
    impassable land along their whole length and running at least 16 cells apart, so that they pass
    through different level-2 sectors (the threat term is 0 at level 0, F5, so threat steers the
    corridor only through levels 1 and 2). Route A is shorter than route B by at least 8 cells and
    passes through the nine threat blocks around a building of a computer-controlled house whose
    type has a positive threat field (claim 16a). Route B passes through no threat block that any
    object of that house threatens. The recorder reads the human house's threat records for the
    blocks along both routes.
  - Procedure: order each type to the goal 10 times, starting afresh each time.
  - Observable: which route each takes, judged by the first cells it enters past the fork (so a
    vehicle destroyed on the way still counts), and the threat the house's records hold along each
    route.
  - Expected: the coefficient-0 type takes route A in every run (claims 7 to 12; threat plays no
    part for it, F7). The coefficient-1.0 type takes route B in every run, provided the recorder
    shows a threat along route A that, with *k* = 1.0, outweighs the extra sector costs of route B
    (F5, F9). The human house's records hold the building's threat from the start, whether or not
    the house has seen it (claim 16a; P30 part a tests that separately).
  - Falsified if: the coefficient-0 type takes route B in any run, or the coefficient-1.0 type takes
    route A in any run while the recorder shows such a threat along route A. A run in which the
    recorder shows no threat along route A is void.
- **P11 (claims 2, 4, 6, 14, 23): the gate near the edge of the usable area.**
  - Setup: a map whose local area leaves a strip of legal cells outside it along one edge. Vehicles
    of a computer-controlled house in a skirmish, each a team member. Run 1: one vehicle placed by
    the map on a legal cell in that strip, an island as in P3 inside the usable area, and a control
    vehicle of the same house inside the usable area. Run 2: a vehicle inside the usable area whose
    team's script has a "move to a waypoint marker" step, the waypoint marker lying on a small
    island in the outside strip, separated from the vehicle by water, and a control vehicle whose
    team moves it to a waypoint marker on an island inside the usable area. Run 3: a vehicle inside
    the usable area whose team's script has a "move to a cell" step (the cell given in the form the
    team specs define), once to a clear land cell in the outside strip that open land connects to
    the vehicle, and once to a water cell inside the usable area.
  - Procedure: run 1: order both vehicles, through their team's script, to a cell on the island.
    Runs 2 and 3: let the teams carry out their move step.
  - Observable: whether each vehicle leaves its team and is given a new destination by its house, and
    the cells each enters. Run 2: in addition, whether the recorder shows the team in its moving
    state when the vehicle's path is computed.
  - Expected: both control vehicles fail the gate (as in P3 run 2: no team exit, no new
    destination). Run 1: the outside vehicle skips the gate (claim 6, second exception), searches
    without a corridor (claim 14), fails, and gives up: it leaves its team, and its house gives it a
    new destination. Run 2: the first vehicle is in the corridor-disabling state, and its destination
    is a legal cell outside the usable area, so the gate is skipped (third exception). It searches
    without a corridor, fails, and gives up in the same way (low confidence). Run 3: the "move to a
    cell" step does not disable the corridor, so the gate compares. An off-map destination (the
    strip cell) and a destination of no reach region (the water cell) both lie in a different reach
    region from the vehicle's cell (claims 2 and 4): each order is dropped at once, the vehicle
    takes no step, and it stays in its team, although the strip cell can be reached over open land.
  - Falsified if: the outside vehicle of run 1 or the first vehicle of run 2 behaves like its
    control vehicle (destination cleared at once, no team exit), or a control vehicle leaves its
    team. A run-2 result like the control's while the recorder does not show the team in its moving
    state is void, not a falsification. In run 3 the vehicle drives towards the strip cell, or
    searches and gives up with a team exit, for either destination.
- **P12 (claims 13 with 13a, 14): corridors through buildings, and retries.**
  - Setup: open clear ground, enclosed so that the only way from one half to the other leads
    through or around a straight wall of buildings of the allied house A (see *Building owners*
    below), 2 cells thick, across the direct route, with a single one-cell gap at one end. A vehicle 10 cells before the wall's
    middle, goal 10 cells behind it. Maps G12, G24, G48 and G96: wall lengths of 12, 24, 48 and 96
    cells, the gap always at the same end.
  - Procedure: order the vehicle to the goal in each map. Then repeat each map with the test train
    type of P9 (test data), facing the wall.
  - Observable: whether the vehicle reaches the goal or gives up without a route, and the cells
    entered.
  - Expected: buildings leave the cells *open* (claim 1), so the first corridor runs through the
    wall, and the cell search inside it follows the cells of the buildings' exemption rectangles
    (claim 13) along the wall's face to the gap, through it, and back into the corridor on the far
    side. So the vehicle reaches the goal through the gap on the first cell search at every
    length of the sweep, the longest detour (G96) staying far inside the expansion limit (claim
    12). The result for each map is the same in every run. The train never uses the corridor
    (claim 14) and searches the whole map, so it reaches the goal through the gap in every map too.
    Secondary check: the outcome and route in each map match what an implementation of claims 3 to 18
    with 13a and 13b gives for the same map, when that implementation meets no exact tie between
    pending entries in its coarse or cell searches. A map whose computation meets such a tie is
    reported but not judged (claim 12a, departure D7).
  - Falsified if: the vehicle gives up at any length (which would mean that building cells are not
    exempt, or that the expansion limit is reached), or the result of a map changes between runs,
    or the train gives up in a map. In these maps a train and a corridor-bound vehicle
    both reach the goal, so they do not tell the two apart; run T does.
  - **Building owners (P12, its runs and P31).** A skirmish or multiplayer map cannot give a human
    player's house a building (S-0009 claim 27). Every building these scenarios place by the map
    therefore belongs to A, a computer-controlled house on the human house H's lobby team, so that
    A and H start allied (S-0412 claim 8). A takes the earlier lobby slot of the two computer players, so it comes first in house order
    (S-0009 claim 19), so the map's objects owned by the computer players' placeholder name go to A (S-0009
    claim 27); a second computer-controlled house E on no team plays the opponent. The start
    positions of A and E are far from the test area, and a run in which a unit of A or E enters
    the test area is repeated. The map's buildings are of a named test file's type with no weapon
    and a threat field of 0 and the foundation stated, so they add nothing to any house's threat
    records (claim 16a), just as H's own buildings add nothing to H's. For the cells and the
    exemption, A's buildings are H's equals: an allied building gives the same outcome as an own
    one (claim 27), and the exemption is given whatever the owner (claim 13). The moving units are
    H's (the map may place H's vehicles and infantry, S-0009 claim 27). A building that must be
    H's own, such as a factory or the recycling building of run S, H builds during the run from
    its base unit (the game's bases setting on, S-0009 claim 45), where H's placement rules allow
    it, and the positions used are named in the run's record.
  - **Run S (short line, Q-1280).** Setup: open clear ground; a straight north–south line of 1 × 1
    buildings of A, lengths 3, 5, 7 and 9,
    each map with the line's middle placed at eight shifts of 0 to 7 cells along the
    line against the level-2 grid. A vehicle 20 cells west of the line's middle, goal 2 cells east
    of it, and the same for one non-jumpjet infantry unit and for the recycle order to a recycling building of H that the scenario sets with a 2 × 2 cell footprint
    and that H builds just behind the line. Observable: whether the unit moves at all or its destination is cleared in
    the tick of the order (claim 23), and the cells entered if it does. Expected: every placement
    of every length is passed: in the tick of the order the unit keeps its destination and gets a
    route (its first step follows on the next tick, S-0209), and it goes round one end of
    the line (which end is recorded, not predicted), and the outcome is identical for the vehicle
    and infantry on the same map. Falsified if: any placement fails at once, or the vehicle and
    the infantry unit differ on one map.
  - **Run T (a strip the unit cannot enter; `quirks.corridor_retry_no_fallback`).** Test data: in
    the test map's override section, give one land type other than *clear* a multiplier of 0 for
    locomotion class *track* and a wheel multiplier of 1 (the land type and its full multiplier row
    are named in the run's record); that land type appears nowhere on the map except in the strip. Setup as run S, with a straight north–south strip of that land type, lengths 3,
    5, 7 and 9 cells and one cell wide, in place of the line of buildings, the same eight shifts,
    and a test vehicle of locomotion class *track* and movement zone *normal* (test data if no
    such vehicle type exists) as the only unit. Ledge maps: the same lengths and shifts with a
    ledge in place of the strip: a line of flat cells one height level above the flat ground on
    both sides, with no slope cell between them; the stock vehicle of run S. Train runs: repeat
    each strip map with the test train type of P9, given locomotion class *track* in the override
    section. Observable: as run S.
    Expected (switch on, the original): strips of 3 and 5 are passed; at least one placement each of
    the 7- and 9-strips fails at once (which shifts fail is recorded; see the strip entry in Edge
    cases and quirks). Ledge maps (switch `quirks.corridor_free_part_ignores_height` on, the
    original): the first corridor of this straight route runs along the line's middle, so the unit
    gives up after its 5 cell searches (claim 23) at every shift of the 7- and 9-ledges (medium
    confidence), and is expected to at most shifts of the 3- and 5-ledges (low confidence); the
    result is recorded per shift. Train runs: the train never uses the corridor (claim 14) and
    searches the whole map, so it reaches the goal round the strip at every placement, including
    those where the tracked vehicle gives up. Falsified if: every
    placement of the 7- and 9-strips is passed (the original then makes a corridor-free search
    after a failed repeated coarse search, or re-searches level 2; the switch's default would then
    be wrong), or any placement of the 3- or 5-strip fails, or a placement of the 7- or 9-ledge is
    passed (a retry would then change the corridor at a ledge, or the free part would follow the
    height rule), or the train gives up at a placement (trains would then use the corridor).
- **P13 (claims 7, 8, 12): base costs and the blocker chain, by route choice.**
  - Setup: a test map whose local area leaves a strip of legal cells outside it, as in P11, so that
    the test unit, placed by the map in that strip, searches without a corridor (claim 14) and its
    route choice depends on the cell costs alone. The test unit is an armed infantry unit of the
    human house, so that S-0209 claim 4 decides each outcome. From its cell two 1-cell-wide corridors
    of clear land, walled by impassable land, lead to one merge cell *M*, from which a single cell
    leads on to the goal. Corridor A consists of straight steps only, in one direction. Corridor B is
    longer than A by Δ steps. The two cells next to *M* on the corridors lie mirror-symmetrically
    with respect to the line from *M* to the goal, so that they have the same estimate. One object
    stands on a cell in the middle of corridor A. Sub-runs by object: (s) an allied vehicle standing
    still without a destination (*stationary friendly*, *b* = 8), (e) an unarmed enemy vehicle of a
    computer-controlled house (*enemy obstacle*, *b* = 20), (f) a wall section of the human house,
    with a test override giving the test infantry type a weapon that can damage walls (*friendly
    obstacle*, *b* = 60), (c) an enemy vehicle that the human house cannot see because it is
    cloaked, made cloaked by a test override (*cloaked blocker*, *b* = 1000), (m1) an allied vehicle
    given a move order along corridor A away from the start 15 ticks before the test order, the
    cells ahead of it empty (chain ending on an empty cell, *b* = 1), (m4) as m1, but a second allied
    vehicle, parked (no destination, no stored path), stands in the next cell ahead of the moving
    one along corridor A, so that the moving one heads into the parked one's cell (chain ending on a
    parked unit, *b* = 1 as for an empty cell).
  - Procedure: order the test unit to the goal. Repeat each sub-run for the Δ values below, one
    map per value.
  - Observable: which corridor the test unit enters first. In sub-runs m1 and m4, also whether the
    moving vehicle still has a destination in the tick of the test unit's path computation (a
    sub-run in which it has lost it is void, since its cell would then give *stationary friendly*).
  - Expected: with the object, corridor A costs about *b* − 1 cost units more than its length, so A
    is taken when Δ exceeds *b* − 1 by a margin, and B when Δ falls short of it by a margin. Margins
    of 3 cost units cover the direction terms and the estimate. (s): B at Δ = 4, A at Δ = 10. (e): B at
    Δ = 16, A at Δ = 22. (f): B at Δ = 56, A at Δ = 62. (c): B at Δ = 60. The chain sub-runs have a
    margin of 1 cost unit only (low confidence): (m1): A at Δ = 2. (m4): A at Δ = 2. The switch
    point for each object can also be computed exactly from claims 7 to 12 for the map.
  - Falsified if: any of these choices is reversed. For (c), A being taken at Δ = 60 means the
    cloaked blocker costs far less than claim 8 says. For (m4), B being taken at Δ = 2 means a
    parked unit ends the chain at cost 4, as a non-unit object does.
- **P14 (claims 9, 19, 22, 27): which cells a re-path makes costly.**
  - Setup: two parallel 1-cell-wide lanes of clear land, L and R, each 30 cells long, separated by
    impassable land and joined at both ends: at the south end by a short junction to the start area,
    at the north end to the goal. A slow vehicle *S* of one type and a fast test vehicle *T* of
    another type with a higher type speed (S-0207), both of the human house. A test override sets the
    path-retry delay field to a value *P* with trunc(*P* × 900) = 90 ticks (F3) and the blockage-delay
    field (an integer number of ticks, S-0209 Import mapping) to 3000, so that every re-path of the
    scenario is made in search mode 1. Run 1: *S* is ordered to the goal through lane L (its
    stored path runs along L), and *T* follows it into L, 3 cells behind. Run 2: the mirror image, both
    in lane R. Run 3 (parity, low confidence): as run 1, but the blocker in the cell ahead of *T* is a
    group of two slow allied infantry units of a type whose speed number (claim 19) is lower than
    *T*'s, walking together along L with
    the same stored path, and run 4 the same with three such infantry units. Run 5 (close case):
    lane R is made 2 cells longer than L by a kink, and *S* is replaced by a vehicle *O* of *T*'s
    own type. *T* is ordered from the start area to the goal and, in the same tick, *O* from the goal
    to the start area, so that both take lane L and meet head-on in it.
  - Procedure: runs 1 to 4: let *T* catch up with the slower unit or units ahead until its next step
    gives *moving blocker* and it re-paths in search mode 1 after the retry delay. Run 5: let the two
    vehicles meet.
  - Observable: the cells *T* enters after the re-path, and whether it backs out to the junction to
    take the other lane. Run 5: in addition, the tick in which *T* and *O* first stand in
    neighbouring cells, and the tick in which *T*'s stored path is emptied (recorder).
  - Expected: runs 1 and 2: the re-path makes every cell of *S*'s remaining stored path in the lane
    (up to 24 cells) cost four times as much, so *T* backs out and takes the other lane whenever that
    is cheaper than following *S* (a reference implementation of claims 7 to 19 gives the choice for
    the exact positions). Without the marks *T* would stay behind *S*. The choice mirrors with the
    lanes. Run 3: each lane cell of the common stored path is counted twice, so it is not made
    costly, and *T* stays behind the infantry. Run 4: counted three times, made costly, and *T* takes
    the other lane as in run 1. *T* never gets *blocked* from the unit ahead, since the two face the
    same way (claim 27). Run 5: once the two stand in neighbouring cells, facing each other and less
    than 512 leptons apart, each gets outcome *blocked* for the other's cell (claim 27). *T*'s stored
    path is emptied in the tick its next step is refused (claim 22), and its fresh search treats
    *O*'s cell as impassable, so *T* backs out of L towards the junction within about the retry
    delay of 90 ticks (or, should its searches fail under claim 13, gives up under claim 23; its
    stored path is emptied either way). Were *O*'s cell *moving blocker* instead, it would cost *T*
    at most 4 cost units (claim 8: *O* is of *T*'s own type, so mode 1 runs as mode 0 by claim 19),
    far less than backing out to take the longer lane R, and *T* would wait facing *O* until the
    blockage delay of 3000 ticks ran out.
  - Falsified if: in runs 1 and 2 *T* stays behind *S* although the reference implementation with
    claim 19 gives the other lane, or *T* takes the same lane in both runs. In run 3 *T* changes lanes,
    or in run 4 it stays behind the infantry although it changed lanes in run 1 (the parity rule
    predicts the opposite in each case). In run 5 *T* keeps its stored path and stays in L facing *O* for more
    than 300 ticks after they first stand in neighbouring cells.
- **P15 (claim 5): content changes that update connectivity.**
  - Setup: for each change, a goal pocket sealed off from a test vehicle only by the object
    concerned, everything else around the pocket impassable: (w) a wall section in a wall line,
    (t) a tree with the full occupation pattern of its theater (category *wall-like*, S-0211) in a
    ring of such trees, (k) a destroyable cliff cell between the pocket and the low ground, with a
    test override setting the game-wide cliff-collapse chance field so that a hit cliff always
    collapses, (b) a bridge whose repair joins the pocket (a far bank), repaired by an engineer (as in claim 5)
    of the human house. In every run of P15 the test vehicle's movement zone is *normal*, for which the
    zone table does not answer *yes* for the category *wall-like*.
  - Procedure: order the vehicle to the pocket. Then remove the object: by a trigger action that
    detonates a weapon at a waypoint marker on it in (w), (t) and (k), the weapon strong enough to
    destroy it, and by the repair in (b). Wait 30 ticks and order the vehicle to the pocket again.
  - Observable: whether the first order is dropped at once, whether the second leads into the
    pocket.
  - Expected: the first order fails the gate (the pocket is another reach region) and the
    destination is cleared at once. The second order passes the gate and the vehicle drives into the
    pocket. In (k) the collapsed cliff must leave a passable cell, which the recorder confirms.
  - Falsified if: the second order is still dropped at once while the recorder shows the object
    gone, which would mean that change does not update connectivity.
  - Run (wb), a wall section built: the goal pocket is joined to the test vehicle's area only
    through a one-cell gap of clear land, everything else around it impassable, and a construction
    yard (a building that produces buildings) of the human house stands nearby. The test vehicle is a
    team member of a computer-controlled house and gets its orders through its team's script, as in
    P3 run 2. First order it into the
    pocket and back out. Then build a wall section of the human house in the gap, wait 30 ticks, and
    order it into the pocket again. Expected: the first order leads into the pocket. The second is
    dropped at once by the gate: no step, no team exit. Falsified if the second order leads to a
    search, shown by a team exit (claim 23) or by the vehicle driving to the wall, which would mean
    building a wall section does not update connectivity.
  - Run (r), the local area changed: the map of P11, with a trigger action that changes the local
    area at a set tick so that it then covers the outside strip. A
    vehicle inside the usable area, a team member as in P11 run 3, whose team's script has a "move to
    a cell" step to a clear land cell of the strip that open land connects to the vehicle. Run it
    once with the step carried out before the change and once 30 ticks after it. Expected: before
    the change the order is dropped at once, no step and no team exit (as in P11 run 3). After the
    change the vehicle drives to the cell. Falsified if after the change the order is still dropped
    at once while the recorder shows the cell inside the new local area.
- **P16 (claim 25): which cells the nearby-cell search collects, without a reference cell.**
  - Run (s), a scatter. Setup: as part (c) of S-0208's scenario W7: an idle civilian of a computer-controlled house, of the unarmed
    type that scenario uses, standing in *guard* at (*x*, *y*) on open flat ground, made to scatter on its own
    by one hit from a unit of the human house at least 4 cells away, so that the scatter has no
    threat point and uses the nearby-cell search around the civilian's own cell with no reference
    cell (S-0208 claim 13). In addition, wall sections of the human house stand on (*x*, *y* − 1)
    and (*x* + 1, *y*), and on
    (*x* − 1, *y* + 1), an unarmed vehicle of the human house (a test override removes its weapon
    if its type has one) standing still with no destination. No unit other than these is within 2
    cells.
  - Procedure: repeat 10 times, each time with the hit in a different tick.
  - Observable: the tick number *N* of the tick of the scatter's search, read by the recorder, and
    the civilian's new destination.
  - Expected: ring 1 holds five qualifying cells: the two wall cells are not enterable by class for
    movement zone *normal*, the vehicle's cell and the civilian's own cell have a non-empty
    occupancy record. In scan order they are (*x* − 1, *y* − 1), (*x*, *y* + 1), (*x* + 1, *y* − 1),
    (*x* + 1, *y* + 1) and (*x* − 1, *y*). All pass the visibility test on flat ground, so the
    destination is the cell at position (*N* mod 5) of that list, counting from 0.
  - Falsified if: the destination is a wall cell or the vehicle's cell in any run, or it differs
    from the cell at position (*N* mod 5). The run is void if S-0208's own scenario W7 (c) does not
    show the choice by tick number on open ground.
  - Run (u), a move order onto an unreachable cell (table T5, move-order row). Setup: open flat clear
    ground, and a test override that sets the wheel multiplier of one land type (say rough) to 0, so
    that its cells have the category *blocked* (S-0211 claim 11). The only cell of that land type is
    (*x*, *y*), inside the usable area. An unarmed wheeled vehicle of the human house (a test override removes its weapon if
    its type has one) stands at (*x* − 6, *y*), no other object is within 3 cells of (*x*, *y*), and
    the area is not shrouded for the human house.
    Procedure: click a move order for the vehicle onto (*x*, *y*), 10 times, each time with the
    vehicle placed afresh and the click in a different tick. Observable: the tick number *N* in
    force when the click is handled (recorder), and the vehicle's destination after the order.
    Direct-destination control: repeat once without the land-type override, on unoccupied,
    revealed clear ground inside the usable area and in the vehicle's reach region.
    Expected: the clicked cell itself is the destination and no nearby search occurs,
    independent of *N*. A substituted destination or a recorded nearby search falsifies
    that control. Restore the override for the ten search runs.
    Expected: (*x*, *y*) fails the reach-region comparison of S-0214 claim 11, so that claim uses the search. Ring 1
    holds eight qualifying cells, in scan order (*x* − 1, *y* − 1), (*x* − 1, *y* + 1),
    (*x*, *y* − 1), (*x*, *y* + 1), (*x* + 1, *y* − 1), (*x* + 1, *y* + 1), (*x* − 1, *y*) and
    (*x* + 1, *y*). All pass the visibility test, so the destination is the cell at position
    (*N* mod 8) of that list. Falsified if: the destination differs from that cell, or it is
    (*x* − 1, *y*), the cell nearest to the vehicle, in every run whatever *N* (which would mean the
    unit's cell serves as reference cell).
    Random-state check: additionally bracket the click-handling nearby-cell search with the
    read-only random-state recorder, recording every stream state before and after it. Expected:
    no stream advances during that search; any advance falsifies the no-draw claim. Obtain one
    qualifying bracket among these ten runs. If the recorder cannot separate the search from
    other activity in the tick, report that check inconclusive; a whole-tick state difference
    alone is not a falsifier of the search's draw count.
  - Run (u2), a no-move order by a unit that is not a free mover onto (*x*, *y*) (table T5, first row). Setup: as run (u), with the same wheeled vehicle (a wheeled vehicle is not a free mover as the draft's Terms define the term) and the same land-type override. Procedure: as run (u), with a no-move order in place of the move order, 10 times, each time with the vehicle placed afresh and the click in a different tick. Observable: the tick number *N* in force when the order is handled (recorder), and the vehicle's destination after the order. Expected: the destination is the cell at position (*N* mod 8) of run (u)'s ring list, in every run. Falsified if: the clicked cell itself is the destination in any run, or the destination differs from that cell. Void if the click does not give the vehicle the no-move action.
  - Run (r), a rally point (table T5, rally-point row). Setup: open flat clear ground with a
    barracks (a building that produces infantry) of the human house whose nearest cell is at least 3 cells from (*x*, *y*). An unarmed
    vehicle of the human house stands still with no destination on (*x*, *y*), wall sections of the
    human house stand on (*x*, *y* − 1) and (*x* + 1, *y*), and a second such vehicle on
    (*x* − 1, *y* + 1). Procedure: set the barracks' rally point by a click on (*x*, *y*), 10 times,
    each in a different tick. Observable: *N* as in run (u), and the barracks' guard spot after the
    click (S-0213 claim 20). This is a rally-point search, not a unit move order: the
    direct-destination bypass of S-0214 claim 11 does not apply. Expected: the same five
    cells as in run (s), in the same order, so the
    guard spot is the cell at position (*N* mod 5). Falsified if: it is a wall cell or a vehicle's
    cell, or it differs from that cell.
  - Run (t), the further test of a no-move order (table T5, no-move row for a cell; test data).
    Setup: a test map whose usable area leaves a strip of legal cells outside it on its west side,
    the usable area starting at column *x*₀; all ground at one height level *h*, not shrouded for the
    human house. The three cells (*x*₀, *y* − 1), (*x*₀, *y*) and (*x*₀, *y* + 1) are slope cells of
    height level *h* (a slope tile of clear land, made with the map's height and tile data); every
    other cell within 3 cells is flat clear land, with no object. A test override gives one vehicle
    type of the human house movement zone *subterranean* and the tunnelling locomotor; one unit of that type stands 8 cells east of
    (*x*₀, *y*), on a tileset that allows burrowing. Procedure: click a no-move order for the unit
    onto the strip cell (*x*₀ − 1, *y*), as S-0214's scenario T7 step (a2) does, 10 times: 5 times
    while the unit stands on the surface, and 5 times while it travels underground (after an order
    that makes it dig in, while the recorder shows its world height below the ground's), each click
    in a different tick and with the unit placed afresh. Observable: the tick number *N* in force
    when the click is handled, the unit's height above the ground at that moment, and its
    destination after the order. Expected: the clicked cell lies outside the usable area, so it
    never qualifies. On the surface no further test applies, ring 1 holds the three slope cells,
    and the destination is the one at position (*N* mod 3) in scan order: (*x*₀, *y* − 1),
    (*x*₀, *y* + 1), (*x*₀, *y*). Underground, the three slope cells fail the further test (S-0010
    claim 17: not flat), so ring 1 contributes nothing and the destination is a flat cell of ring 2
    inside the usable area, the one at position (*N* mod 7) of the scan order (*x*₀, *y* − 2),
    (*x*₀, *y* + 2), (*x*₀ + 1, *y* − 2), (*x*₀ + 1, *y* + 2), (*x*₀ + 1, *y* − 1), (*x*₀ + 1, *y*),
    (*x*₀ + 1, *y* + 1). Falsified if: an underground click gives a slope
    cell, or a surface click gives a ring-2 cell (which would mean the test does not depend on the
    unit's state), or the clicked cell's own failure decides it (no destination at all underground,
    which would mean the clicked cell is tested as claim 24 tests the destination). Void if the clicks do not give
    the no-move action (S-0214 claim 11 then takes another branch), or if no test override can make
    a unit travel underground in the setup.
  - For runs (u), (r) and (t): if the recorder cannot fix which tick number is in force when the click
    is handled, one shift *o* of 0 or 1 is allowed, the same for all 10 clicks of a run: the
    expected cell is then the one at position ((*N* + *o*) mod *n*). The run falsifies the claim if
    some click fits neither shift, or if its clicks need different shifts.

- **P17 (claim 7): diagonal steps first on open ground without a corridor.**
  - Setup: a test map whose local area covers only a small part of the map, as in P13, leaving
    outside it a block of legal cells of flat clear land at one height level, at least 30 × 30
    cells, with no overlays or objects. The vehicle of P1, placed by the map in the middle of that
    block, at (*x*, *y*), facing map north. Having never entered the usable area, it searches
    without a corridor (claim 14), and the gate is skipped (claim 6).
  - Procedure: order it to the cells displaced by (+10, +4), (+4, −10), (−12, +5), (−9, −2),
    (+7, +7) and (+12, 0) from (*x*, *y*), one order per run, with the vehicle placed afresh each
    time. If the interface refuses an order to a cell outside the usable area, give it through a
    team's script instead, as in P11, with the vehicle a team member of a computer-controlled
    house. Each destination lies within 12 cells, so the whole path is stored at once (claim 20).
  - Observable: the sequence of cells entered.
  - Expected, independent of tie order: with displacement (Δ*x*, Δ*y*), the vehicle enters
    max(|Δ*x*|, |Δ*y*|) cells. The first min(|Δ*x*|, |Δ*y*|) steps are diagonal, towards the
    destination, and the rest straight. For (+10, +4) the cells are (*x* + *i*, *y* + *i*) for
    *i* = 1 to 4, then (*x* + *i*, *y* + 4) for *i* = 5 to 10. No exact tie of *f* arises in
    these searches. Neither straightening pass changes these paths: they contain no right-angle V
    of diagonal runs, and they are already shortest paths.
  - Falsified if: in any run a straight step comes before a diagonal one, or the vehicle enters
    more cells than the Chebyshev distance. That would mean the search does not favour the
    candidate that closes the straight-line distance fastest (claim 7), or that its estimate or
    step costs differ from F1, F2 and T4.

**P18. Immediate fresh search after a blocked first step (claim 22).** Use S-0207 D17
part d3's narrow clear-ground corridor, unarmed human-owned mover and enemy blocker, with
one bystander computer player isolated from the observed corridor. Game speed 2,
no engine add-on and no mods (the named test files of a scenario excepted), except the named test map and retry-delay override. Set the path-retry delay
to 1 game minute. Introduce the blocker by the map's supported reinforcement trigger after
the mover has a stored path, before its check of the blocked cell. For each tick, record the
stored steps, destination, track state, search occurrences and retry-timer state. The run qualifies
only if the refused first check occurs with the retry timer still running and the old step
still stored. Obtain one qualifying run; otherwise report inconclusive. Expected: a mode-0
search occurs on the refusal tick despite the timer. A found path is checked again that tick.
A failed search follows claim 23. Falsifier: no search on the refusal tick and a later search
only after the timer expires, or a found path receives no repeated check on that tick although
the mover faces its new first step. Use the stated setup and record the stated observables. Void the run if the bystander enters the corridor or acts on either test unit.

**Hover variant (claim 22).** Repeat with a human-owned stock hovering type and record
its heading point instead of track state. Obtain one qualifying first-check *blocked* run
and one first-check *cloaked blocker* run while the retry timer is running. For the latter,
place a stock cloak-capable enemy blocker after the path is chosen, as in S-0503 H9(c).
Expected in both: an immediate mode-0 search and, when a path is found, a repeated check
that tick. Only a trace with a successful search followed by the intended repeated outcome
qualifies to test destination retention: repeated *cloaked blocker* retains the destination
and waits; repeated *blocked* clears it. Failed searches instead follow claim 23.
Record the retry timer immediately after each actual immediate search: expect the full
*R* from this tick on success, search failure and gate failure, even when an older timer
was running. Include the obstacle and stationary-friendly first-check cases of S-0503
claim 12. Later ordinary no-heading-point retries must wait for the new timer, unless
another first-check refusal bypasses it. A delayed first search, a different timer value,
or the opposite destination result in a qualifying repeated check falsifies the
corresponding hover rule. Repeat S-0503 H9(e)'s moving-allied-blocker test
with the same recorder: expect mode 1 while the blockage timer runs and mode 2 once it
expires, each held by the retry delay. A different mode or timer boundary falsifies the
moving-blocker extension. Report inconclusive for each subtest without a qualifying trace.
Retain the same game speed, settings and bystander void rule.

**P19. Failed search beside the destination: ship and hovering (claims 21 and 23).**
Use a tiny level map with an isolated bystander computer player, game speed 2,
no engine add-on and no mods (the named test files of a scenario excepted), this test map and named overrides included. Use a human-owned stock hovering type
on clear ground, and a human-owned stock ship type in a separate open-water area. In each
area place one destination tile of a distinct land type that is connected to the mover's
reach region. An override makes that tile's land multiplier zero for the mover's locomotion
class, without changing its passability category or reach region. Set the retry delay to
0.1 game minutes, giving *R* = 90 ticks. Give a move order to the tile from 10 cells away.
Record search ticks, destination, position, stored steps, retry timer and queued behaviour
for each tick. Obtain one qualifying run per locomotor: the mover must
reach a neighbouring cell and then fail a search within 1 cell of the destination, with no
destination substitution or gate failure. Expected: on that failed-search tick the mover
loses its destination and starts *R*, without queuing a behaviour. For hovering, the fresh
search that fails waits for *R* counted from the preceding successful fresh search. Falsified
if either mover keeps its destination after the qualifying failure, or if the hovering fresh
search occurs before that delay expires. A run with another unit entering the observed area,
a substitution, or no qualifying search is void or inconclusive, not a falsifier. Use the stated
setup and record the stated observables. A separate hovering run
orders the mover to a cell on an isolated island in another reach region while its timer is
expired. Expected: no step, a gate failure, destination cleared, and *R* starting on that tick.
Any step towards the island, a retained destination, or a different timer value falsifies
this expected gate-failure result. The failed-gate timer specifically tests claim 21.
For cached travel, add S-0503 H9's arrival-time failed-search runs in both distance
cases of claim 23, including an immediate search after first-check refusal. Retain
only traces with a positive distance computed before arrival and no subsequent refusal
suppressing travel. Expected: clearing destination and heading point does not cancel
that tick's F3–F4 travel of S-0503, despite speed fields being zeroed during refusal.
No later travel occurs after the move ends. Suppression solely because the search
failed, or later travel without a new move, falsifies this rule; no qualifying arrival
is inconclusive.
For the destination distinction, use S-0503 H12 and H15's stop and arrival fixtures.
Record both destinations, heading point, stored path, search ticks, retry timer,
vehicle-present flags, contact and cell-arrival updates. Expected: accepted stop clears
its order-level destination and ordinary retry delay, but equal full three-dimensional
movement destination and heading point survive that request. Any subsequent search
uses the retained movement destination's cell and starts *R*. Qualify arrival traces
separately for failure before choice, failure inside non-refusing choice, and timer-deferred
choice. Expect the flag/update outcomes of S-0503 claim 15, not a uniform failed-search
reaction; H15's same-cell-contact and held cases must be distinguished. Loss of equal
points at stop, a search aimed at the absent order-level destination, or contrary
flag/update outcomes in a qualifying trace falsifies the corresponding rule. No
qualifying trace is inconclusive.

**P20. Deck steps over sloping ground (claim 10).** Use P4 K2's high bridge over dry
clear ground, with the same unit and recorder. Named test maps change only the height and tile
of one side cell *T* next to a marked south deck cell *F*. The unit starts on *F*'s deck.
Provide a distant ramp route joining the bank to *T*, so the gate passes independently of the
side step. In separate maps, *T* is not a bridge cell and its ground is one level above the
ground under *F*, with *F*'s ground a slope; or one level below it, with *T* a slope. Controls
make the lower ground cell flat. Order onto *T* once in each map. Record the first stored step
on the computation tick, before movement reactions, and the cells entered afterwards.
Expected: the slope variants store the direct side step, onto *T*'s ground, a drop of 3 or 5
height levels respectively; the flat controls do not store that step. A direct step in a flat
control, or no direct step in a slope variant with a passed gate and clear occupants, falsifies
the corresponding row. Repeat with *T* a bridge cell, its ground one level higher or lower,
and with its deck-entry mark separately present and absent. Expected: the slope condition
allows the direct step onto *T*'s deck with either mark value; the flat controls refuse it.
Any mark-dependent refusal of an otherwise qualifying slope step falsifies the mark exemption.
One qualifying run per variant. A failed gate or unexpected occupant voids a run; do not infer
the search's rule from a later refused locomotor step.

**P21. Nearby radius, no result, and winged checks (claim 25).**
- (a) Use a flat map large enough for a square of radius 34 cells centred on *C*, entirely
  inside the usable area. An ordinary unarmed wheeled unit stands on clear ground outside that
  square. Water occupies the square, except one clear candidate *Q* at (*c*ₓ + 31, *c*ᵧ),
  connected to the mover's ground by a clear lane running east from *Q*. No other clear cell
  lies within radius 31. Reveal the area. Click a move onto water cell *C* as in P16(u),
  once on this map and once on a variant with *Q* and its lane beginning at radius 32.
  Record the order's destination on the click-handling tick. Expected: the first map gives
  *Q*; the second finds no nearby cell and produces no move order (S-0214 claim 11).
  A missing *Q* in the first map, or a new move destination in the second, falsifies the
  radius/no-result rule. Verify the click reaches the search rather than being refused by
  shroud or targeting. A different targeting action voids the run. This tests the 31/32-cell
  boundary, not the smaller-map-width boundary or hidden-ring limit.
- (b) Place an aircraft factory on one flat plateau and a parked human-owned vehicle on *C*
  on another plateau, separated by impassable cells and height differences greater than 3
  levels, with no bridge or tunnel. The two cells must be in different reach regions of zone
  *fly* under claim 4; verify this from the test geometry. Set the factory's rally point to
  *C* once, with *C* revealed and usable. Record its resulting guard spot. Expected: it is
  *C*, despite the occupied cell and different reach region, because the winged search omits
  both checks and has no height condition. A different guard spot falsifies that combined
  omission. Controls replace the factory by an infantry-producing building, first with *C*
  occupied on the building's own plateau and then with *C* empty on the separated plateau.
  Neither control may choose *C*. A control choosing it voids the contrast pending review of
  the caller. These observations do not isolate the winged class-rule omission.

**P22. Bridge conditions of nearby-search callers (claim 25).**
- (a) Named test map: a high bridge over an isolated dry-ground pocket, with raised banks
  joined by its enabled link. The ground pocket and banks have different normal-zone reach
  regions. A blocked landing centre *C* on the deck has two free ring-1 candidates: ground
  cell *A* in the pocket and non-bridge bank cell *B* at deck height. All other ring-1 cells
  have nonempty occupancy records. Bridge cells other than *C* in that ring are occupied on
  their decks. Both *A* and *B* pass visibility and class tests. Use a non-infantry teleporting
  type with zone normal and wheel class. Choose a departure point on level ground at world
  height exactly 312 leptons above *C*'s terrain surface, and a second point at 364 leptons
  above it (for example a slope on a higher departure plateau). Give an ordinary teleport
  order to occupied *C*, one run from each departure. Record departure height at landing-point
  selection and the chosen landing cell. Expected: at 312, deck mode is off, *C*'s ground
  reach region allows only *A*; at 364, it is on, the enabled link's deck region allows only
  *B*. A choice of *B* at exactly 312, or *A* above 312, falsifies the strict threshold.
  A run qualifies only if the original landing is refused and the recorder confirms the two
  candidate regions and empty records; otherwise it is inconclusive. The observation concerns
  landing-point selection, not the later jump height.
- (b) Use S-0307 RG12's empty-radius attack failure on a revealed ground cell *C* at level *h*.
  A free neighbouring bridge cell *B* has underlying clear ground at *h* − 4 and deck at *h*.
  Distant ramps join its ground to the attacker's ground reach region. Arrange every other
  ring-1 cell to fail occupancy or class qualification, and one free non-bridge cell *Q* on
  ring 2 at level *h*. Keep *C* occupied by the target. Record the destination on the failure
  tick. Expected: *Q*, never *B*, although *B* passes the height and reach-region conditions.
  A destination *B* falsifies the separate bridge exclusion. A different failure centre,
  occupied *B*, or a failed reach comparison voids the run. One qualifying run.
- (c) Repeat the height/region contrast for S-0402 H13's free-unit creation, using the building's
  top-left cell as centre. Make the only otherwise qualifying ring-1 cell a free bridge deck
  at the centre's level, with its ground connected to the building's ground reach region by
  distant ramps. Put one free non-bridge cell on ring 2; neither candidate has an overlay
  other than the bridge's own overlay. Observe the created unit's cell. Expected: the
  non-bridge cell, through either the first or second search, never the bridge. A bridge
  placement falsifies exclusion in at least the search that selected it. To test the second
  search alone, add a resource overlay to the non-bridge candidate: it then fails the first
  search's no-overlay condition but passes the second. Obtain one qualifying run of each
  variant. This does not separately identify whether the first search also rejected a bridge
  by its no-overlay condition; it establishes that the second excludes it independently.

**P23. Blocker-chain termination and layer (claim 8).** Use P13's two narrow corridors,
but a driving mover placed outside the usable area, with the first search made before it enters,
so no corridor is used. Script move inputs for blockers; take a read-only snapshot of positions,
speeds, facings and stored paths on the mover's search tick. Record the mover's new stored path
before it encounters any blocker. One qualifying run per contrast below. The two corridors
have the same merge/goal geometry as P13 and their length difference is 2 steps. Before the
oracle run, compute their stored-path predictions under claims 7 to 18, once with base cost 1
and once with base cost 4 for the first blocker. Use a fixture only if these predictions choose
different initial corridors, neither prediction meets an exact tie, and straightening preserves
the distinction. Expected: the recorded path chooses the cost-1 corridor prediction when the
chain ends empty, and the cost-4 prediction in the capped/non-unit cases. A choice matching the
opposite prediction falsifies the indicated chain rule; a path matching neither rejects the
combined search hypothesis and cannot be attributed to the chain alone.
- Cap contrast: ten units in consecutive cells, all heading onward, with the next cell empty,
  versus nine such units followed by an empty cell. Expected base costs: 4 and 1 respectively.
  Slow named type-speed overrides widen the snapshot window. A snapshot with any zero-speed,
  empty-path unit before the cap, or any different first-listed object, is void. Scripted input
  need not hit a predetermined tick: repeat up to 100 attempts to obtain one snapshot of each
  case, or report inconclusive. This replaces the assumption that the ten-unit state must be
  produced reliably on every attempt.
- Non-unit contrast: the first moving blocker heads into an allied building cell versus an
  empty cell, with the same positions and first blocker speed/path. Expected base costs: 4
  and 1. Qualify while the blocker still heads into that cell, before its own refusal changes
  the state. Up to 100 attempts per contrast, otherwise inconclusive.
- Layer contrast: the blocker heads onto a bridge cell whose ground holds a parked unit and
  whose deck holds a moving unit that heads into an allied building's cell on the bank.
  Provide the first blocker first on the deck or at least 3 height levels above that bridge
  cell's ground, and then on ground less than 3 levels above it. Expected: the deck-selected
  chain continues through the deck unit and terminates at the bank building, cost 4; the
  ground-selected chain terminates at the parked unit, cost 1. The snapshot must establish
  each layer's first-listed object, both units' headings and the height condition. If legal
  placement cannot produce such distinct lists, that contrast is inconclusive and supplies
  no evidence for the rule.

**P24. Gate cost and train uniform costs (claim 8).** Use the no-corridor two-route geometry
of P23, clear except one test object in corridor A. Record the first stored path immediately
after computation, not the later collision or gate-opening reaction. One run per variant.
An ordinary outside-area mover crosses a closed allied gate in A, while B is 2 steps longer.
Expected: its stored path uses A, as for an empty gate cell, at base cost 1. A B-choice when
the exact no-tie reference prediction selects A falsifies that prediction; it does not by
itself estimate the gate's cost to unit precision. For the train cost contrast, use P9's test
train type, facing down A so its three excluded cells lie behind both choices. Put a parked
allied vehicle in A. B is 4 steps longer. Expected: the train stores A at base cost 1, while
an otherwise identical non-train outside-area control stores B at base cost 8. Repeat with an
unarmed enemy object giving enemy-obstacle outcome and B 16 steps longer: train A, control B.
Check both predictions before the run using claims 7 to 18, with no exact tie. A train choosing
B, or a control choosing A, falsifies the corresponding uniform-cost contrast. Different
outcomes, a corridor in the control, or train exclusions intersecting either planned route
void the contrast. These variants test the gate's low cost and uniform treatment of two
non-clear outcomes, not every possible train outcome independently; the untested train rows
retain aggregate backing only.

**P25. Timers and qualification read directly (claims 19, 21, 22, 26).** Common setup and
recorder as P6, including scripted input and tick-by-tick retry/blockage timer readings.
The timer's remaining ticks are the observable: a later movement reaction is not its proxy.
- (a) Place a walker next to a tile with zero foot multiplier but a passable category and no
  building, so a passed-gate search has no positive-length path under claim 15. Give the move
  through a team script if targeting substitutes the destination. Control: a destination on an
  isolated island, for a failed gate. Set retry delay to 0.1 game minutes, giving 90 ticks. One
  qualifying search of each kind. Expected: after the passed-gate search the walker keeps its
  destination and its timer reads 90 ticks from that computation tick. After the failed gate its
  destination is cleared and its timer has run out at the end of that tick (claim 21). A different
  duration or start tick, or a running timer after the failed gate, falsifies claim 21's
  walker-failure rule. A targeting substitution or no computation voids the run.
- (b) P9's leader and follower, with retry delay 90 ticks. Choose the leader's last-step direction
  and destination so the first follower's derived destination is separately: a free reachable cell,
  a blocked cell beside the car that passes the gate but yields no positive-length path, or an
  isolated reachable-by-leader cell in a different reach region from the car. Record its timer and
  destination immediately before and after the computation triggered by the leader; one qualifying
  run per case. Expected: success leaves no delay and gate failure leaves no delay. Search failure
  leaves 90 ticks from that tick, or no delay where the recorder shows the car's destination already
  cleared at the reading after the computation, since claim 26 leaves the timing of that clearing
  open. Any other timer transition falsifies claim 26's rule. A leader with no found path, an
  unchanged follower destination, or no follower computation voids the run. No assumption about when
  its destination later clears is made. Named type overrides may give the leader an amphibious zone
  while its car has a normal zone for the failed-gate fixture, or different locomotion classes for
  the passed-gate blocked-tile fixture. Qualify the derived destination and actual gate result from
  the recorded state, not from these type differences alone.
- (c) Repeat P14's lane fixture with blockage delay shortened enough to expire while the
  blocker still has at least four stored steps. Use a same-type blocker in one run and a
  faster different-type blocker in another, neither qualifying in mode 1. Record a search
  after the blockage timer expires, its mode, the blocker path and the mover's new path.
  Expected: mode 2 counts both blockers' remaining paths regardless of type/speed. Use only
  positions for which the marked/unmarked reference predictions choose different lanes with
  no exact tie. The mode-2 path follows the marked prediction. An unmarked-prediction path
  falsifies the type/speed independence; no distinguishing prediction is inconclusive.
- (d) Separate blockage endings. On P6's map record a first wait until its blockage timer is
  close to expiring. In one run move the blocker aside long enough for the mover to take one
  step, then introduce another moving blocker; in another replace the mover's destination
  while it has taken no step, and let that move meet a moving blocker. Observe the blockage
  timer at the next wait. Expected: both start a new full blockage delay from that wait,
  rather than inheriting the previous remainder. A carried-over remainder falsifies the
  corresponding ending condition of claim 22. One qualifying run each. A successful intervening
  step in the replacement-only run, or no such step in the step-only run, voids the contrast.
- (e) A destination given while the walker's delay runs (claim 21; Q-2896). Part (a)'s
  passed-gate walker, with its retry delay of 90 ticks, and a second tile like the first next to
  the walker. Let *c* be the tick of the walker's failed search, after which it keeps its
  destination. In tick *c* + 10 give it a move order by the same means as in part (a): in run 1
  to the cell it already holds as its destination, in run 2 to the second tile. One qualifying run
  each. Record the timer at the end of every tick and every path computation from *c* to *c* + 100.
  Expected in both runs: at the end of tick *c* + 10 the timer has run out, or reads 90 ticks from
  a path computation made in that tick, and the walker's next path computation after *c* comes
  before tick *c* + 90. A timer still counting from tick *c* at the end of tick *c* + 10 (its
  reading 10 ticks below its reading at the end of *c*), or no path computation between *c* + 10
  and *c* + 89, falsifies the end of the delay for that run. No
  computation at *c*, a targeting substitution, or a walker that leaves its cell voids the run.

**P26. Neutral and special give-up (claim 23a; Q-0238).**
- Setup: purpose-made skirmish map, game-speed index 2, 800 × 600, 
  no engine add-on and no mods (the named test files of a scenario excepted), stock types only. Place one ordinary unarmed driving vehicle owned by the neutral
  house, then repeat with the special house. A map-defined team of that house supplies
  a move-to-waypoint destination through its script, as in P11; the team is used only
  to exercise the original, not as an implementation requirement. Enclose that waypoint
  more than one cell away in impassable buildings: their cells
  retain the open passability category, so connectivity does not itself refuse the order.
  Include one bystander computer player isolated from the test area. Use normal map
  team processing, not a write into the running game's state, to produce the move.
  For the human-owned control, supply the same destination through an ordinary move order.
  Score only a run whose recorder confirms a passed reachability gate and an actual
  search failure satisfying row 2, with no intervening order or second failure.
- Procedure: record the failed-search tick, current and queued behaviours, destination,
  map occupancy and random-stream state immediately around the failure reaction and
  on the next two ticks. For the final-search test also record the house-supplied centre.
  That single cell is an input to this test, not evidence that its upstream selection
  rule is known. Predict the final result from T5 and claim 25 at the recorded failure
  tick. Compare a same-geometry run owned by a human-controlled house. Use the
  read-only recorder and scripted input; do not alter the measurement harness here.
- Expected: both non-player houses queue *area guard*, with no *move* queued or forced
  by the failure reaction. A found final-search cell is assigned on the failure tick.
  No result leaves no destination. It matches the *track*, *normal*, ground-reach-region
  search with no height condition and selection position *N* mod *n*. The centre
  choice advances the scenario stream, even without a base or start cell. This part
  tests only the presence of centre-choice draws, not their total or the final search's
  separate no-draw rule; P16 run (u) tests that shared nearby-search rule.
  The human-owned control queues *guard* and receives no house-selected destination.
- Falsifiers: a destination assigned only on a later tick, an additional queued or forced
  behaviour attributable to this reaction, a mismatch with the final-search prediction,
  or no scenario draw in an isolated non-player-house choice. Unrelated draws void
  the draw comparison, not confirm it. A gate refusal is not this test.
- Runs: obtain three qualifying failures per non-player house and three human controls,
  allowing at most twenty attempts per group. Include a no-result final search only
  if recorded occupancy establishes that no candidate qualifies through the full radius.
  Otherwise record that branch as untested. No upstream centre-choice claim is confirmed
  by this conditional final-search test.
- Base-record-refresh comparison (claim 25, T5): use S-0215 B37's owned-building
  variants with the same stock-data settings and read-only recorder. For each recorded
  entry, removal or owner-change event, compute the mean cell using S-0215 F12 and
  predict the new base-start cell with T5 at that event's tick. Record the resulting
  cell and scenario-stream state around the refresh, separately from a later path
  failure. Expect the *foot*, *normal*, unrestricted-reach-region search in deck mode,
  with no height, no-overlay or burrowing condition, bridges allowed and no reference
  cell, taking no draw. A qualifying candidate at another height distinguishes this
  search from height-constrained callers; a candidate outside a supplied region
  distinguishes it from the final give-up search. A different result from the T5
  prediction or an isolated refresh draw falsifies the rule. Obtain one qualifying
  event of each kind; unrelated draws or missing contrasts leave that branch untested.

### P27: Non-contact docking-cell entry and departure (claim 27a)

- Setup: a tiny revealed skirmish map with a stock refinery, two ordinary driving
  harvesters A and B of the human player, a nearby resource field and one bystander
  computer player kept away. Use speed 2, stock data, no engine add-on and no mods (the named test files of a scenario excepted).
  Record tick, position, destination, current and queued behaviour, contact, tether,
  stored route and retry timer with a read-only recorder and scripted input.
- Entry and rest: while B holds the refinery contact, order A to its docking cell
  after B has left that cell. Qualify only runs where the cell is empty, entry is
  permitted by S-0209 claim 5 and A holds neither contact nor tether. Stop A there
  and observe for 300 ticks without another order. Record every scatter request
  and exclude runs in which one moves A. Expected: A reaches and remains on the
  cell without unloading or acquiring B's contact. A departure with no order,
  behaviour destination or general scatter cause falsifies the persistence rule.
  Refused entry when the ordinary entry table permits it falsifies the absence
  of an additional docking restriction. If no qualifying entry exists, the test
  is inconclusive, not evidence that every docking cell is enterable.
- Departure: in three qualifying runs, issue harvest with a clear route to the resource field.
  Expected: A gets a field destination and leaves without taking B's contact. Its first route
  attempt is not held back by a running retry timer (claims 21 and 27a). No run here starts that
  attempt while a timer runs, so the timer is not judged. A refused departure solely because B holds
  the dock falsifies the claim.
- Failure: repeat with a purpose-made map placing the resource field more than 1 cell away in the
  same reach region but behind an unbroken building barrier. Qualify only runs in which A receives
  that field destination, no nearby substitution changes it and the search actually fails.
  Expected on the failure tick: destination cleared, no retry delay left and *guard* queued.
  Remove the barrier after *guard* becomes current, then observe for 300 ticks without an
  order. Nothing restarts the route or harvest in that time. Give a fresh harvest order; a new
  destination can now produce a route. Retaining the failed destination, omitting queued *guard*,
  or restoring that destination without an order falsifies the stated failure reaction. Record
  three qualifying runs; inability to obtain the failed-search fixture leaves this part
  inconclusive. Gate failure and failures within 1 cell retain the discriminating tests of P4 and
  P19, not this distant-search expectation.

### P28: Aircraft landing footprints and ties (claim 25a)

- **Setup:** purpose-made skirmish maps with stock rules and art, no engine add-on and no mods (the named test files of a scenario excepted),
  speed index 2, short game off. Use an ordinary aircraft and a full own-house pad whose
  building behaviour requests landing nearby when it cannot accept another aircraft.
  Check that S-0501 claim 41's building search is used, that the winning building's
  weighted distance is at least 256 leptons (avoiding the own-cell replacement), and no
  successful pad choice supersedes it. Use the read-only recorder at 800 × 600,
  one slot, eight runs each with the seed the game chose (S-0202 claim 8), and scripted input. Record the search tick, chosen
  destination cell, terrain, bridge state, reach-region membership derived from the map,
  occupancy and random-stream state. A run not invoking that request is inconclusive.
- **Footprint and displacement:** surround the building with obstacles so only one
  qualifying top-left cell exists in its first qualifying ring. Leave its positive-x,
  positive-y 3 × 3 rectangle clear. Expect that top-left cell, not its middle. In paired
  maps place one stock infantry unit, parked vehicle or building in each of the other
  eight rectangle cells in turn, keeping the top-left clear. Expect refusal and a separate
  known clear rectangle selected. Repeat with a wall and with water outside the building's
  normal ground reach region in one rectangle cell. Retaining that candidate falsifies
  the nine-cell requirement.
- **Which tests apply:** compare empty rectangles containing a stock resource overlay,
  a passable slope and different reachable ground levels against a clear control. Qualify
  each run by checking the nine class, region and occupancy conditions. Expect no independent
  refusal for overlay, slope or height difference. On a bridge fixture keep the nine ground
  cells in the building's normal reach region. Compare ground occupancy under a free deck
  with deck occupancy: only deck occupancy refuses a bridge cell. A ground-region mismatch
  still refuses it despite a connected deck. Contrary selection with all other conditions
  controlled falsifies the corresponding test; failed fixture qualification is inconclusive.
- **Usable-area extent:** provide a rectangle whose top-left is height-aware usable but
  another cell is not, only where all nine still pass region, class and occupancy conditions.
  Expect acceptance without a separate rectangle-boundary refusal. If no such stock fixture
  can be established, rejection is inconclusive, not evidence for a further usable-area test.
- **No-reference selection:** create several visible qualifying top-left candidates in one
  stopping ring, with unequal distances to the aircraft and a pair equally distant from the
  building. Keep occupancy stable and induce requests on enough tick residues to cover the
  preferred-sequence length. Expect position *N* mod *n* in the scan order, unchanged by
  aircraft distance and with no random-state advance attributable to the search. A nearest-only
  result or selection draw falsifies this rule.
- **Reference ties:** separately trigger S-0501 claim 41's own-cell 1 × 1 replacement with
  its centre occupied and two equally near, visible, qualifying cells in the first ring.
  Expect the earlier scan-order cell on every tick residue, with zero displacement and no
  search draw. Reverse the geometry and repeat. Tick-modulo alternation here, or a shift by
  one cell on either axis with either footprint size, falsifies the claim.

**Flight caller checks (claim 25).** Use S-0502 scenario J16's complete setup,
procedure, recorder and 20-seed comparison for the four jumpjet rows of T5. Compare
selection at the recorded search tick with the ring, visibility and collection rules
here. Include bridge exclusion versus allowance, deck mode, overlay-bearing candidates,
height differences and reach-region boundaries. Include a grounded ordinary vehicle's
own-cell search with its own occupancy as the only mark: that cell must be rejected,
not temporarily cleared for selection. Record animated undeploy's initial selection
separately from its subsequent movement-point adjustment. Use S-0502 J19 and J22's
setups and qualifying observations for effective destination clears from stop, deploy
and behaviour processing, distinguishing suppressed repeated assignments and idle
units from actual reroutes. Predict each actual reroute with the stopped-movement
row at its search tick. A selected cell inconsistent with its T5 settings or these
shared selection rules, or treating a qualifying effective clear as exempt from this
row, falsifies the claim.

**Transport caller checks (claim 25).** Use S-0507 T13's tiny flat skirmish map,
stock-data setup with no engine add-on and no mods (the named test files of a scenario excepted), game-speed index 2 and 800 × 600.
Unload a vehicle passenger through ordinary orders, varying the processing tick with
candidate cells held fixed. Record the serving neighbour, passenger placement cell,
subsequent movement destination and random-stream state read-only. Predict placement
from the vehicle-passenger-exit row and the preferred sequence at that tick. Expected:
tick-modulo selection, no search draw, and the exit-round movement destination retained
independently of placement. A different placement, search draw or substituted movement
destination falsifies that part. If no ordinary-order fixture yields multiple preferred
candidates, the selection comparison is inconclusive. S-0507 T9 supplies the proposed
shore witness; shore approach and both unchecked no-result cases retain the unmeasured
qualifications of claim 25, rather than being confirmed by ordinary successful exits.

**P29. Restoration displacement and no-track cleanup (claims 20 and 25).**
Use tiny skirmish maps with stock types and data, no engine add-on and no mods (the named test files of a scenario excepted), game speed
index 2, and read-only tick/state recording. For displacement, follow S-0524 C14:
sell a starting building, park ground occupants on its former foundation, and trigger
restoration. Predict each search from the occupant's old cell and T5, recording
placement cells and facing. Expect reference-distance choice within the collected
sequence, facing 0, and earlier re-placements affecting later searches. In a variant
with every candidate blocked through the radius cap, expect a stand-in placement
attempt and removal if it fails, not indefinite off-map retention. Another search
class, reference, reach restriction or no-result outcome falsifies that subclaim.
For cleanup, follow S-0207 D30 with same-cell move and accepted same-cell building
entry controls. Expect the cell goal removed at the first no-track locomotion update,
without requiring a search, while cell equality alone retains the building goal.
Record final-track arrival separately: absent intervening effects, cell-goal removal
is at the next no-track update, not completion itself. Search-dependent cleanup or
unconditional same-cell building-goal removal falsifies that distinction. These are
proposed recorder runs, not completed measurements.

### P30: Threat records and the cells a path avoids (claims 16a and 24, Q-1751, Q-1975)

- **Setup:** a purpose-made flat skirmish map of open clear ground, stock rules and art, no engine
  add-on and no mods (the named test files of this scenario excepted), speed index 2, short game
  off, shroud on, 800 × 600. Houses: a human house H, a computer-controlled house C on another lobby
  team, and a computer-controlled house A on H's lobby team (so H and A ally each other from the
  start, S-0412 claim 8). A named test file, a rules file supplied through the file system for
  these runs and removed afterwards (glossary *measurement setup*, named test files), adds:
  - a vehicle type V1 with threat coefficient 1.0 and a type V0 equal to it in every other field
    with coefficient 0,
  - a building type B with no weapon, a 1 × 1 foundation and a threat field *a* of at least 400,
    and a type B3 equal to it except for a 3 × 3 foundation,
  - a defence type D with a weapon whose range covers the route and a threat field of 0,
  - a vehicle type E with a threat field *a* and no weapon, and
  - an aircraft type F with a threat field *a* and no weapon that needs no airfield and lands at
    the end of a move,
  - copies of E for part i: E-c, which cloaks; E-w, with a weapon whose range reaches well beyond
    the 3 × 3 blocks; E-d, with a weapon of the same range as E-w's and ten times its damage; and
    a transport type T with a threat field *a*, no weapon and room for two
    infantry units,
  - an infantry type N with no weapon and a threat field *b* of at least 100 and different from
    *a*, for parts f2 and i.

  The start and goal cells lie 40 cells apart on one row, with the test object beside the middle
  of that row: every test building stands with its whole foundation off the row, on the cells
  next to it, so that no building is on the route and none can make a cell of the row *blocked*
  for V0 or V1 (claim 27). Record the run's threat-block coordinates. Every house's start position and starting
  units lie at least 40 cells from the test object, each part places only the test objects it
  names, and the map places no other object within that distance, so nothing else reaches the
  recorded window. The read-only recorder of the
  measurement setup records, every tick, the cell each vehicle and aircraft is recorded in, whether each test
  object is on the map, and the threat records of H, A and C for the 7 × 7 threat blocks around the
  test object (the block of a building's top-left foundation cell, claim 16a). Five runs of each
  part and of each type, each with the seed the game chose (S-0202 claim 8; no expectation here
  depends on a random draw), each order given once at tick 60 by scripted input.
- **Lobby order:** a skirmish map gives every object it places for the computer players'
  placeholder to the computer player first in house order, and gives the human house no
  building (S-0009 claim 27); the earlier lobby slot of the two computer players comes first in
  house order (S-0009 claim 19). The lobby order is therefore set per part:
  - in parts a, a2, b, d, h and j, C takes the earlier lobby slot, so the map's computer-owned
    objects (B, B3, D and F) go to C.
  - variant c2 has runs of its own, in which A takes the earlier lobby slot, so its map-placed
    B goes to A, and the map places no object for C.
  - parts c, e, f, g and i place no computer-owned object by the map, and the lobby order does
    not matter to them.

  The run's record names the order used. A run in which a map-placed object belongs to a house
  other than the one its part names is void.
- **Route checks:** the threat records and recorded cells are tallies that only the read-only
  recorder shows. So every part that changes H's records also has a route check that needs no
  reading of them: a V1 and a V0 of H, ordered from the start cell to the goal cell at the tick
  the part names, with V0 as the control. V1 *bends* if it leaves the row to pass around most of
  the nine blocks around the named block, as in part a (a few cells inside them can remain on its
  path). V1 *drives straight* if it keeps V0's row there. Route checks test the aggregate path effect,
  not single threat-record values; a record reading and a route check that disagree are reported
  separately. Reading a house's threat records with the read-only
  recorder is already part of the approved scenario P10, which reads them along both of its
  routes; P30 reads the same records, and also the cell each vehicle and aircraft is recorded in.
- **Part a (unseen enemy building):** B of C placed by the map beside the row, never seen by H.
  Expected: from tick 0 H's records hold *a* at B's block, ⌊*a*/2⌋ at its four edge neighbours,
  ⌊*a*/4⌋ at its four diagonal neighbours and 0 elsewhere in the window. Falsified if the records
  differ from this pattern, or stay 0 until H sees B. Route check (qualitative, P10 is the route
  test): V0 drives straight along the row, while V1 leaves the row to pass around most of the nine
  blocks; a few cells inside them can remain on V1's path (claim 16a: level 0, the corridor
  exemption, straightening). The records of a computer-controlled house are checked by part e
  (C's records). The records are the discriminating observation, and the route check stays
  qualitative: it falsifies nothing by itself.
- **Part a2 (large building):** B3 of C in B's place, placed with its top-left foundation cell in
  the last cell column of a threat block along the row, so that its centre cell lies in the next
  block along the row (the run's threat-block coordinates show which). Expected: the pattern of
  part a centred on the block of B3's top-left cell. Falsified if the pattern is centred on the
  block of its centre cell. Route check: V1's route is recorded, with no prediction of where its
  bend starts; the records are the discriminating observation.
- **Part b (armed, no threat field):** D of C in B's place. Expected: H's records stay 0 and both
  types drive straight. Falsified if V1 bends around D's range or the records hold any threat.
- **Part c (allied object):** in the runs of part e, A's records too. E is placed by the map, so it
  has a recorded cell when the alliances of game start rebuild A's records, and A is
  computer-controlled, so that rebuild counts E, the vehicle of its ally H (claim 16a). Expected:
  from tick 0 A's records hold the pattern of part a around the block of E's start cell, and they
  keep it there, unchanged, while E moves (A allies H, so no incremental rule changes A's records
  for E), while C's records follow E as part e says. Falsified if A's records at tick 0 do not hold
  that pattern (a rebuild that left allied units out would give 0 there), or if they change while
  E moves. Variant c2, B of A placed by the map in runs of its own (*Lobby order*): the
  alliances of game start
  rebuild H's records after the map's objects are placed, so from tick 0 H's records hold the
  pattern of part a around B (stale threat, `quirks.threat_rebuild_counts_friends`), and it stays.
  Falsified if H's records are 0 there at tick 0. Route check (qualitative, as in part a): V1
  ordered at tick 60 bends around B of A, H's ally's building.
- **Part d (removal):** B of C destroyed by a map trigger at tick 30. Expected: H's records return
  to 0 in the tick the recorder first shows B off the map (after any destruction sequence), and V1
  ordered after that tick drives straight. Falsified if a record of the window stays above 0 after
  that tick, or drops to 0 while B is still on the map.
- **Part e (moving threat):** E of H, ordered at tick 30 to drive across the row of blocks; read C's
  records. Expected: the pattern of part a around the block of E's recorded cell, moving in the
  same tick that the recorded cell enters a cell of another threat block, and unchanged while it
  moves within one block. Falsified if C's records lag the recorded cell or change within a block.
- **Part f (occupied building):** a stock civilian garrison building of the civilian house
  (S-0508) with a threat field of 0 in the test file, entered by 1, then 2, then 3 infantry units of H of a test
  infantry type whose threat field is also 0 in the test file (so the units add nothing of their
  own before or after they enter); read C's records. At its first turn after the first entry
  the building changes owner to H (S-0508 claim 7), and at its first turn after the last occupant
  leaves it returns to the civilian house (S-0508 claim 8). These are owner changes, the owner-change
  row of claim 16a: the building's amount is taken away from the records of every house that
  records the old owner's objects and added to those of every house that records the new owner's.
  C allies neither H, which is on another lobby team, nor the civilian house (S-0412 claim 8: no
  house is made to ally a house that plays a country of the civilian side), so C records both owners' objects and the two changes
  take away and add the same amount at the same block in the same tick: C's records do not change
  at either owner change. Expected: the building's block holds *n* × the per-occupant threat value of the rules
  file in use (and the spread of claim 16a) from the tick each occupant enters, and 0 after they
  leave. Falsified if the record ignores the occupants or follows another factor, or changes in
  the tick of an owner change. Variant f2: the same, with infantry units of type N (threat field
  *b*), each standing still in a block of its own beside the row before the part's order.
  Expected: C's records hold the sum of the patterns of each unit's own amount *b* and of the
  building's amount; in the tick a unit enters, its own pattern is gone and the building's amount
  grows by the per-occupant value; in the tick each unit is placed on leaving the building, its
  pattern *b* is added at the block of the cell it is placed in, and the building's amount falls.
  Falsified if a unit's own pattern stays after it enters, or is not added when it leaves.
- **Part g (rebuild in play):** the test file gives H's construction-yard type a threat field
  *a*. Scripted input drives H's starting construction vehicle to B's place and deploys it there,
  with the yard's whole foundation beside the row, and the yard is the test object of this part.
  Expected: H's records are not changed by the deployment. At tick 300 a map trigger's
  alliance-making action makes H and C allies; S-0209 (the backing of its alliance views) gives
  such actions as acting on both houses at once, so the run does not use the player's alliance
  order of S-0414 claim 12. Expected: from that tick H's records hold the pattern of part a around H's own
  construction yard (`quirks.threat_rebuild_counts_friends`); H sells the yard at tick 400 and the
  pattern stays after the yard has left the map. With the switch off the records would stay 0.
  Falsified if H's records change at the deployment, stay 0 after the trigger, or lose the
  pattern after the sale without a further alliance change. A trigger that makes no alliance
  change voids the run. Route check (qualitative, as in part a): V1 ordered at tick 250 drives
  straight past the yard, and V1 ordered at tick 450 bends around the cells where the yard stood,
  although nothing stands there then.
- **Part h (aircraft):** F of C is placed by the map; a map trigger destroys it at tick 400.
  Read H's records, and record whether F starts on the ground or in the air. Two runs, which
  choose the case by F's placement cell (S-0009 claim 38, as its scenario M23 part B does):
  - **h1, on the ground.** F's entry is at the test object's place, a cell usable by the
    height-aware test (S-0009 claim 4), so F starts landed. At tick 60 a map trigger orders it to
    a cell 12 cells along the row (three threat blocks away, inside the window), where it lands.
    Expected: the pattern of part a around F's placement block from tick
    0, unchanged while F flies, after it lands and after it has left the map; the block of its
    landing cell stays 0 throughout (`quirks.aircraft_threat_stays_put`). With the switch off the
    pattern would move to the landing block when F lands and be gone once F has left the map.
    Falsified if H's records follow F in flight, move when it lands, or lose the pattern at the
    placement block when F leaves the map. Route check: V1 ordered at tick 450, after F has left
    the map, bends around F's placement block and drives straight past the block of its landing
    cell; with the switch off it would drive straight past both.
  - **h2, in the air.** h2 has a map of its own, laid out as P30's map but with these
    coordinates, so that F's placement cell lies outside the local area while the start, the goal,
    the row and the test object's place lie inside it. The map is 100 × 100 cells, so its legal
    cells have 100 < *s* ≤ 300 and −100 < *d* < 100, with *s* = *x* + *y* and *d* = *x* − *y*
    (S-0009 claim 1). Its local area is (1, 50, 98, 49), so on its flat ground a legal cell is
    usable when 200 < *s* ≤ 300 and −98 < *d* < 98 (S-0009 claim 4). The test object's place is
    (111, 111) (*s* = 222, *d* = 0), in threat block (27, 27). The row is *y* = 112, from the start
    cell (91, 112) to the goal cell (131, 112), so its cells have 203 ≤ *s* ≤ 243 and −21 ≤ *d* ≤ 19,
    all usable. F's entry is on cell (99, 99) (*s* = 198, *d* = 0), a legal cell outside the local
    area, so F starts in the air (S-0009 claim 38, as in its scenario M23 part B). That cell's
    threat block (24, 24) is three blocks from the test object's block on each axis, a corner of
    the recorded 7 × 7 window. Every house's start position lies in the usable area, at least 40
    cells from the test object, as in the other parts. At tick 60 a map trigger orders it to land on the test object's place. Expected: H's
    window, F's placement block included, holds 0 throughout, because the rebuilds of game start
    skip an aircraft with no recorded cell and nothing later adds F's amount (claim 16a). With
    the switch off F's amount would be in the records at its placement block from tick 0 until
    it lands, and at the landing block from then until it leaves the map. Falsified if any record
    of the window rises above 0, or if F starts landed (the run is then void). Route check: V1
    ordered at tick 200, while F stands landed, drives straight past the landing block.
- **Part i (what the amount leaves out; transport):** C's records, one object of H per run, each
  placed by the map at the test object's place: E placed at a quarter of its full strength; E
  placed at the highest veterancy level; E-c, which cloaks and which C never sees; E-w; E-d; and T.
  Expected: for each of E, E-c, E-w and E-d, the pattern of part a with amount *a*, from tick 0, the same
  in every run. For T, two units of type N of H are placed by the map in blocks of their own and
  ordered at tick 60 to enter T, and T is ordered to unload at tick 400. Expected: the pattern of
  *a* around T throughout, the pattern of *b* around each unit until the tick it is taken off the
  map on entering T (at whichever block its recorded cell is in, claim 13), nothing for the units
  while they are inside, and the pattern of *b* added at the block of each unit's placement cell in
  the tick it is unloaded. Falsified if any variant of E gives an amount other than *a*, if T's
  amount changes with its passengers, or if a unit's pattern stays while it is inside T. No route
  check: the part reads only C's records (part e gives the route effect of a vehicle's amount).
- **Part j (threat in the substitution estimate, claim 24, Q-1975):** a separate flat map with
  scenario P7's barrier for run 8b (the one-cell gap, the six guarding vehicles on *D* and its
  ring, the close-enough field of 1.375 cells), the guarding vehicles of H, but with the
  barrier running from *x* − 30 to *x* + 30. B of C is placed by the map on the mover's side, beside
  the gap but off the cells of the gap's route, so that the gap lies inside B's nine threat blocks
  and both barrier ends lie outside them. The mover is V1 in one set of runs and V0 in the other,
  at P7's start cell, ordered to *D* at tick 60.
  - Control j0 (no substitution): the six guarding vehicles are left out, and V1 and V0 each start
    on *F* and are ordered to *D*. The part is valid only if V0 drives through the gap and V1 drives
    round a barrier end, which shows that B's threat moves V1's corridor at this layout.
  - Expected: V0 is given *F* as its destination in the tick of the order's path computation and
    stops on *F*, as in P7 run 8b. V1 keeps *D* as its destination and drives off towards a barrier
    end, because its estimate from *F* follows the corridor round the end (*E* > 8).
  - Falsified if, in a valid part, V1 is given *F* or any other cell on the mover's side of the
    barrier before it has passed a barrier end: the estimate would then ignore the threat term.

### P31: Out of a factory exit across the unit's own base (claims 1, 13; Q-1771, Q-1870)

- **Setup:** a purpose-made flat skirmish map of open clear ground, stock rules and art, no engine
  add-on and no mods (the named test file of this scenario excepted), speed index 2, short game
  off, 800 × 600, a human house H, the allied computer-controlled house A and the opponent E, as
  in P12's *Building owners*. Every building below that the map places belongs to A and is of
  that entry's test type with a 1 × 1 foundation (no weapon, threat field 0). A 3 × 2 block of such buildings (3 cells west to east, 2 north to south). A row of ten
  such buildings, 4 cells north of the block's north row, its west end 5 cells west of the block's
  west column and its east end 2 cells east of the block's east column, with open ground round
  both ends of the row and round the block. One non-jumpjet infantry unit of H on the cell south of
  the block's middle building. 64 maps: the whole layout shifted *e* cells east and *n* cells
  north against the sector grid (claim 13b), one map for each pair (*e*, *n*) with *e* and *n*
  each from 0 to 7, so that every alignment of the layout against the 8 × 8 blocks occurs once.
  Each map is run again with a vehicle of
  movement zone *normal* in place of the infantry unit.
- **Procedure:** order the unit to the cell 16 cells north and 3 cells west of its own. One run per
  map.
- **Observable:** whether, in the tick of the order, the unit keeps its destination and gets a
  route, or its destination is cleared and it stays (claim 23); the cells it enters; the tick it reaches the destination; which end of
  the row it goes round.
- **Expected:** at every shift and for both units, the unit keeps its destination and gets a route
  in the tick of the order, and reaches the destination round one end of the row (which end is
  recorded, not predicted). By claim 13 this happens on its first cell search, which the scenario
  does not record.
- **Falsified if:** at any shift the destination is cleared and the unit stays (the buildings'
  exemption rectangles would then not let the cell search round the row, and the limit of 5 cell
  searches of claim 13 would decide).
- **Run F (factory):** an infantry factory of H, which H builds on open ground during the run
  (P12's *Building owners*), and three rows of 1 × 1 buildings placed by the map for A north of it, each row 6 to 10 buildings long (lengths and positions named in the run's
  record), the rows 3 cells apart, the nearest 3 cells north of the factory, open ground round the
  ends of every row. Produce one infantry unit; once it stands on the factory's exit cell, order
  it to a cell 15 cells north of that cell. Repeat with a vehicle factory and a vehicle of
  movement zone *normal*. Own-base repeat: H builds the three rows itself, from a stock 1 × 1 type
  of its side or the test type made buildable by H in the named test file, each building where H's
  placement rules allow it (the positions named in the run's record). Expected: as above, the unit
  keeps its destination, gets a route and arrives. Falsified if it gives up.
- **Run D (docking cell; Q-1870):** H builds a stock refinery on open level ground during the
  run (P12's *Building owners*), and its free harvester is the unit. A resource field lies 15 to 20
  cells west of the refinery, and between them the map places for A two north–south rows of 1 × 1
  buildings, each 6 to 10 buildings long, with open ground round both ends (lengths and positions
  named in the run's record). Let the harvester load, return, unload and leave. Run D2: the same
  layout with the refinery's site and the ground at least 2 cells round it raised on a plateau
  (its height levels named in the run's record), whose only ramp down lies north of the plateau,
  off the straight line to the field, and whose other edges are cliff ground that no unit can
  enter, with no ledge and no strip open to the coarse searches. A D2 map whose edge turns out
  to hold a ledge or such a strip makes the run inconclusive, not falsifying. Observable: from
  the harvester's first seeking processing after unloading ends (S-0402 claim 11) on, whether it
  has a field destination and keeps it with a route; the cells it enters; whether it reaches the
  field. Expected in both: it leaves the docking cell round the refinery and reaches the field
  without giving up (Edge cases, the harvester leaving a docking cell). Falsified if its destination
  is cleared and it stays on the docking cell in *guard*. If a run does give up, record the
  harvester for 2000 ticks more: S-0402 claims 3a and 13 expect it to stay on the docking cell in
  *guard*, with no destination and no new route.
- **Control (trees):** the layout of the 64 maps with a terrain object (a tree) in place of
  each 1 × 1 building. Expected: the unit moves and arrives, as with buildings, but by a
  different mechanism: a tree's cell is *wall-like* or *partial obstacle* (S-0211 claim 11), which
  the coarse searches close to movement zone *normal* (and, when *wall-like*, to *infantry*), so
  the corridor itself goes round, and a tree's eight neighbours have the exemption (claim 13).
  Compare outcomes only, not cells. The control does not test the exemption rectangle of
  buildings and does not show the failure after 5 cell searches; a stop here falsifies the tree
  rules of claim 13, not those of buildings.

## Open questions

- Measurement coverage remains incomplete for the path-length cutoffs of claim 19, the closing
  stretch and threat limits of claim 18, and hidden-ring selection boundaries of claim 25. Their backings explicitly remain
  unmeasured; existing aggregate route checks do not establish them independently.
- Sectors and connections (claim 13b): which further cells a tunnel's link joins, and how often the
  labelling-order cases of claim 13b occur on stock maps, are not established. The secondary checks
  of scenarios P1 and P12 are the practical check.
- For the corridor exemption (claim 13): exactly when a moving unit's recorded cell changes (on
  entering a new cell or at some other moment of its move) is only partly established. The same
  moment decides when a moving vehicle's or infantry unit's threat moves to another threat block
  (claim 16a; P30 part e records both). For an aircraft the moments are stated (claims 13 and
  16a, rated medium): placement on the ground and landing.
- When exactly the destination of a follower car with the driving or walk locomotor is cleared after
  its computation of claim 26 fails (at once, or only at the car's next path computation) is not
  established.
- Whether, after a local change (claim 5), the original's result can ever differ from a full
  recomputation for the new map content (for example when the change splits a region) is not
  settled.
- Whether the region corner cases of claim 3 occur on stock maps, and whether a configuration other
  than the 2 × 2 case of claim 3 is common enough to matter, is not settled. Scenario P4 map H tests
  the smallest case.
- Whether any rule sets the congestion mark of a cell permanently (Terms, claim 9). None was found.
- Which units are loaners (claim 14) in stock play, and the exact team state that goes with the
  "move to a waypoint marker" condition, are only partly established.
- For a unit of an infantry type with the jumpjet flag, whether its current locomotor affects its
  path computation in any way is not established. It does not change the movement zone that the
  coarse search of a path computation uses (claim 13). The estimate *E* of claim 24 uses the
  unit's own movement zone, not *infantry*; no scenario checks this.
- The mechanical-walker locomotor also computes paths. Whether it uses claim 22's blockage
  modes and claim 20's immediate refresh like the driving locomotor is not established.
  Ship refresh, chase-cut and retry rules are included in claims 20–22, and S-0504 claim 9
  agrees with them.
  The lepton-scale refresh boundary and table-root distinction remain unmeasured for
  driving and ships: neither P5 nor S-0504 N11 supplies a discriminating boundary setup.
  Hovering chase-cut, fresh-search delay and blocked-step rules are
  included there too; unlike driving and ships, hovering has no last-step refresh.
- Whether the path-length conditions of claim 19 (a vehicle with at least two stored steps, an infantry
  unit with at least three) apply in search mode 2 as well as in mode 1 is not established. This spec
  applies them in both.
- For locomotors other than driving, ship and hovering, whether a unit of claim 23's third row
  keeps its destination, and when it starts moving towards it again (S-0215), is not established.
- For the ship and hovering locomotors and every locomotor other than the driving and walk
  locomotors, whether clearing a unit's destination, or changing it to a different one, ends a
  running retry delay (claim 21) is not established. For the hovering locomotor the destination
  meant is the order-level one, and a person's stop is excepted (S-0503 claim 16). This spec keeps
  the delay running for them, and for the hovering locomotor leaves it to S-0503. Whether a
  harvester's offer of its docking cell ends a running delay is not established either, and S-0402
  claim 10a keeps the delay running. For every locomotor other than the walk locomotor, whether
  giving a unit the destination it already holds ends the delay is not established; this spec
  keeps the delay running. For the walk locomotor claim 21 states that it ends it (Q-2896).
- Which alignments of a straight strip of ground the unit cannot enter fail, and whether lengths
  3, 5, 7 and 9 of such a strip, and of a ledge, behave as P12 run T expects (see the strip entry in
  Edge cases and quirks): not established until run T is run. That lines of buildings (P12 run S,
  Q-1280) and the layouts of P31 (Q-1771) are passed at every shift follows from the exemption
  rectangles of claim 13 but has not been observed.

## Changes for other specs

- **S-0309, Import mapping, per-occupant threat.** S-0309's import row for threat values joins a
  type's threat field and the game-wide per-occupant threat and gives both an absent-key default
  of 0. That default is right for the type's threat field but not for the per-occupant threat:
  when no rules layer sets the per-occupant key, the original uses 5, a built-in default rather than a
  value of any data file (stock data sets the key, so the constant shows
  only with a rules file that leaves it out). S-0309 should give the per-occupant key its own row
  with that absent-key default, as this spec's Import mapping does; the per-occupant threat of
  claim 16a uses it.
- **S-0603, claim 3, the approach behind a short line of buildings.** Claim 3 says: "A unit
  ordered to a recycling building that stands behind a short line of buildings, with open ground
  round the line, can fail at once: the path computation fails, the unit does not start a move
  and its destination is cleared", citing this spec's claims 13, 13a and 23, its edge case "short
  line of buildings" and scenario P12 run S. This spec's claim 13 gives every
  building's exemption rectangle the corridor exemption, so the first cell search goes round such
  a line at every length and shift that P12 run S tests, and the approach is expected to succeed
  (Edge cases and quirks, the answer to Q-1280, item 2). S-0603 should drop that
  failure from claim 3 and from claim 4's proviso ("This presumes the approach is not cut short as
  described in claim 3"), drop "that a short line makes the approach fail at once" from claim 3's
  medium list and the low rating for which lengths and alignments fail, and turn the separate
  approach-failure test of scenario G9 (the setup of P12 run S) into a test that the unit keeps
  its destination and gets a route in the tick of the order at every length and shift. Its Open questions entry on which lengths
  and alignments make the approach fail, tracked by Q-1280, is then answered for straight lines
  of 1 × 1 buildings. A failing case remains for a short strip of ground the unit cannot enter that
  the coarse searches count as open (Edge cases and quirks; P12 run T), which S-0603 may cite in
  its place.
- **S-0605, its S-0212 cross-reference.** S-0605's list of cross-references says that the
  approach to the building "can fail where S-0212's short-line case (Q-1280) makes that
  computation fail: a short line of buildings across the route with open ground round it". Under
  this spec's claim 13, a short line of buildings does not make it fail (as for S-0603 above).
  S-0605 should name instead a short strip of ground the actor cannot enter that the coarse
  searches count as open, with open ground round it (Edge cases and quirks; P12 run T).
- **S-0412, claim 14, start alliances.** Claim 14 says that nothing observable depends on when the
  start alliances of claim 8 are made. Threat records are an exception: those alliances are made
  after the map's objects and the starting units are placed, and each one rebuilds the allying
  house's threat records (claim 16a), so a house starts with stale threat from the buildings the
  map places for it and its allies and, if it is computer-controlled, from its allies' starting
  units. S-0412 should name this exception and point to claim 16a. Scenario P30
  variant c2 checks it.

- **Glossary, *AI house*.** The entry says that an AI house has the same unit rules as a
  human-controlled house. Claim 16a's rebuild makes one exception: in the house test that decides
  whether the house's threat records count its allies' vehicles, infantry and aircraft, an AI
  house counts as computer-controlled. The entry should name this exception and point to
  claim 16a; Departure D5, for the units' own coefficient, is unchanged.

- **S-0208, claim 6.** Its "what that does to a running retry delay is S-0212 claim 21's case of
  a destination given again, which that spec leaves open (its Open questions)" can read that the
  scatter, whose cell is already the walker's destination, ends a running retry delay as one to a
  different cell does (S-0212 claim 21), so the search in the extra locomotion is not held back
  either. Level and backing: medium, `unmeasured` for a scatter (S-0212 claim 21).
- **S-0417, claim 17 (*No path*) and Open question 16.** "Whether giving it the destination it
  already holds ends the delay is left open, as S-0212 leaves it" (claim 17, and its summary
  sentence) can read that it ends it (S-0212 claim 21), so a delay that runs when a walking unit
  is given a destination, the one it holds included, never holds that destination unsearched. Open question 16's premise ("takes it as not
  ending it") no longer holds, and the case can close with that pointer. Level and backing:
  medium, `measurable` by S-0212 scenario P25(e) run 1 for an order to the same cell, as S-0212
  claim 21 rates it.
- **S-0209, claim 10.** "That end is `unmeasured` here: scenario C5 reads only the blockage
  delay" can add that S-0212 scenario P25(e) run 1 reads the end of the retry delay for an order
  to the walker's present destination cell. Level and backing: medium, `measurable` (S-0212
  P25(e) run 1).

## Questions for the project lead

- Decided (project lead, 2026-10-10): departure D7 is accepted. Exact ties between pending entries
  go to the entry recorded first (claim 12a); route costs and every formula stay exact.
- Decided (project lead, 2026-10-10): departure D4 is accepted. Regions, sectors and connections
  are built by the rules of claims 3 and 13b, not by the original's labelling order.
- Decided (delegated sign-off, 2026-10-10): the rules for tunnels and trains (claims 7, 9, 14, 16,
  20 and 26) stay in this spec.
- Decided (delegated sign-off, 2026-10-10): the five switches for behaviours that read like defects
  are accepted with their names and corrected alternatives, default original:
  `quirks.opposite_facing_reversal`, `quirks.coarse_limit_records_cost`,
  `quirks.train_exclusion_search_only`, `quirks.detour_mirrored_reference` and
  `quirks.detour_after_tunnel_position`.
- Decided (delegated sign-off, 2026-10-10): S-0209 owns the entry outcomes. Claim 27 agrees
  with S-0209 claim 5 for allied units that count as moving, and points to it for the full table.

- Decided (spec approval, 2026-10-19): `quirks.corridor_retry_no_fallback` is
  accepted, a switch that turns the original's failure after a repeated coarse search into one
  extra cell search, default original.
- For information (Q-1771): buildings give the corridor exemption to their exemption rectangle
  (claim 13). So a line or wall of buildings with open ground round it is passed on the first
  cell search at every length and shift that P12 and its run S test, and does not make the unit
  give up (the Q-1280 entry in Edge cases and quirks). The case that
  `quirks.corridor_retry_no_fallback` changes is a strip of ground the unit cannot enter that the
  coarse searches count as open (P12 run T). A ledge across the route makes the unit give up
  after 5 cell searches whatever that switch says (Edge cases and quirks). The approach of a
  recycling or absorbing building behind a line of buildings follows the same rule, so S-0603
  claim 3 and scenario G9, and S-0605's cross-reference, are listed under Changes for other
  specs.
- Decided (delegated sign-off, 2026-10-25): `quirks.threat_rebuild_counts_friends`,
  `quirks.aircraft_threat_stays_put`, `quirks.negative_threat_field` and
  `quirks.corridor_free_part_ignores_height` are accepted, default original (claims 13a and 16a,
  Edge cases and quirks).
- Decided (delegated sign-off, 2026-10-25): an *AI house* counts as computer-controlled in the
  rebuild's house test (claim 16a). Departure D5 is not extended to it; under D5 an AI house's
  units still use their own coefficient.
- Decided (project lead, 2026-10-24): a to-tile building's exemption rectangle left exempt after it
  leaves the map is a plain bug, fixed with no switch: claim 13, Edge cases and quirks.
