---
id: S-0406
title: Building placement (foundation cells, the cell test, adjacency, building off allies, shroud, walls and gates, upgrades)
topic: structures
revision: 13
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-14
measurements: []
requests: [Q-0133, Q-0130, Q-0220, Q-0277, Q-0282, Q-0798, Q-3087]
supersedes: []
---

# S-0406: Building placement (foundation cells, the cell test, adjacency, building off allies, shroud, walls and gates, upgrades)

## Summary

Where a house may put a finished building: which cells a building covers, the test every one of
those cells must pass, the distance to the house's own base (adjacency) and to allied bases, the
shroud condition, what walls, wall towers and gates do to the cells they are placed on, how a wall
fills the gap to another wall, how an upgrade is placed onto a building, and when a placed or
deployed building's construction begins and ends. Skirmish only. Campaign
behaviour and network games are out of scope. Deploying a vehicle into a building uses the cell test
of this spec and is S-0415's.

**Paired hand-off confirmations.** S-0404 claim 10 distinguishes upgrade orders from
ordinary placement: upgrades bypass adjacency but retain explored-ground legality, as claim 12
here states. S-0215 claim 11 retains claim 17's construction ready-mark and second-start
contract. Its factory-release rule agrees with S-0417 claim 2: release and unloading neither
set nor clear that mark. Those factory rules do not change the construction timings here.
S-0214 claims 21–22 own the ordinary arrow used during placement; claim 19 here owns the
separate placement display. S-0411 claims 10, 12 and 17 distinguish placement's announcement
from the game-wide refusal sound and delegate placement's text/no-text answer to claim 20
here. Confirm these matching companion passages at joint sign-off.

## Terms

Shared terms used here are defined in the glossary: *house*, *building*, *type*, *field*, *tick*,
*foundation* (with its *top-left cell*), *cell*, *lepton*, *legal cell*, *usable cell*, *overlay*,
*overlay data value*, *occupancy record*, *land type*, *land multiplier*, *locomotion class*,
*shroud*, *construction state*, *on the map*, *in play*, *local player*, *AI house*,
*computer-controlled*, *human-controlled*, *registry*, *registration index*, *absent-key default*,
*importer warning*, *quirk switch*, *departure*, *place order*, *adjacency*, *cell test*, *upgrade*,
*recorder*, *bystander computer player*. S-0404's *item*, *line*, *done* and *could make* (Terms) are
used as S-0404 defines them, and S-0408's *silent wall sale* (Terms) and unsellable flag (claim 3,
Import mapping) as S-0408 defines them. S-0010's *tileset*, *morphable flag*, *tile number*,
*water role* and *slope shape* (claims 4 to 6 and 15) and S-0211's buildable flag *B* and
*high-bridge cell* (Terms, claims 2 and 15) are used as those specs define them, and so are
S-0215's *ready mark* (Terms) and per-tick starts (claim 11). A place order
names the top-left cell of the foundation. In addition:

- **Placement refusal announcement.** The cannot-deploy-here announcement requested for a
  refused placement (S-0411 claims 10 and 12; S-0701). It is not the game-wide refusal sound.
  A request does not guarantee audible playback: announcement eligibility and suppression
  remain S-0701's.
- **Cell test.** The test of claim 3 that a cell passes or fails for a building type being placed,
  together with the wall exceptions of claim 8. S-0415 (claim 3) uses it for deploying.
- **Filling building.** A building that the wall fill of claim 11 creates in a gap cell.
- **Foundation cells.** The cells a building covers for placement and occupancy: its foundation's
  cell list (claim 2), each cell taken relative to the top-left cell. A building is "in" a cell
  only when the cell is one of its foundation cells.
- **Foundation rectangle.** The *W* × *H* rectangle of cells whose top-left cell is the foundation's
  top-left cell, *W* along map *x* and *H* along map *y*, with the foundation's size from S-0108's
  vocabulary (Import mapping, foundation vocabulary). It never includes a bib row.
- **Base-normal building.** A building whose type has the base-normal flag (Import mapping). Only
  base-normal buildings of the house itself support adjacency (claim 4).
- **Ally-eligible building.** A building whose type has the ally-eligible flag (Import mapping).
- **Adjacency *A*.** Written *A* below: the building type's adjacency field, an integer in cells
  (Import mapping).
- **Build-off-ally option.** S-0412's build-off-ally setting (a lobby check box whose default comes
  from the rules data): it lets ally-eligible buildings of allied houses support adjacency
  (claim 4).
- **Wall type.** A building type with the wall flag of a building type (Import mapping, building
  type section; not the overlay type's flag below). It never stays a building: it
  turns into its wall overlay (claim 10).
- **Wall overlay of a type.** The overlay type a building type names in its to-overlay field
  (Import mapping), whether or not that overlay type has the wall flag of an overlay type. In this
  spec *wall overlay* tied to a building type ("the type's wall overlay", "its wall overlay", "a
  wall type with no wall overlay") always means this to-overlay target. S-0210 (Terms) uses *wall
  overlay* alone for any overlay type with the wall flag; this spec writes that sense as "an
  overlay type with the wall flag". A *wall* in a cell is the overlay that cell holds, named by its
  overlay type or by the building type whose wall overlay it is.
- **Wall flag of an overlay type.** The flag that marks an overlay type as a wall: what S-0210
  (Terms, *wall overlay*; Import mapping) defines and owns, absent-key default no. This spec reads
  it only in claim 11 (Import mapping).
- **Wall owner.** The house recorded as owning the wall in a cell (S-0210, claim 14).
- **Builder of a wall overlay.** The first building type in the registry (registration order of
  S-0004) whose wall overlay is that overlay type, whatever the other types that name the same
  overlay. S-0408 (claim 12) calls it "the building type that builds that wall"; the unsellable
  flag it refers to is the builder's.
- **Damaged wall.** A wall in a cell whose overlay data value is 16 or more.
- **Wall tower type, gate types.** The building types that the rules data names, game-wide, as the
  wall tower and as the four gate types (two of a first pair, two of a second pair; Import mapping).
  A *gate-flag type* is any building type with the gate flag; it need not be one of the named gate
  types.
- **Upgrade type.** A building type whose powers-up field names another building type (Import
  mapping). S-0405 (Terms) calls such a building an upgrade.
- **Upgrade positions.** The three places, numbered 1 to 3, in which a building records the
  upgrade types it holds, at most one type per position (S-0405, Terms, uses the same three
  positions). A position can be empty while a higher one is filled (claim 12). A building's *upgrade
  count* is a separate number: how much room it has used (claim 12).
- **Cursor shroud result.** A yes-or-no value that placement mode (S-0411, claim 10) keeps for
  human input: the result of claim 5's condition for the whole foundation rectangle (9 cells for the
  8-cell 3 × 3 word) that would lie under the cursor, computed whenever the cell under the cursor
  changes while placement mode is active (claim 6).

## Behaviour

### What a placement needs

1. **Placement after completion.** A building or defence becomes placeable when its item is done
   (S-0404, claim 7), and it stays done for as long as needed: completion never places it. A place
   order is applied only when the item of the house's line of that kind is done and a factory
   building of the house could make its type with the tech-tree condition, power and docks left out
   (S-0404, Terms): its build limit (S-0407, claim 9) is not asked again here. Without
   such a factory building the order does nothing: the item stays done and no sound plays (until
   the last factory building of the kind is removed for good, when S-0404, claim 8 cancels and
   refunds the item; a building being sold or queued to be sold already counts as unable to make). Otherwise
   the building is put on the map at the centre of the named cell, (256*x* + 128, 256*y* + 128)
   leptons (F4), by the rules of this spec:
   - if the placement succeeds, the building enters play in that tick, its construction state begins
     (S-0108) and the line ends (S-0404, claim 7),
   - if it fails, nothing is placed, the placement refusal announcement is requested when the
     house is the local player's, and placement mode resumes with the same building
     (S-0411, claim 10). No game-wide refusal sound is requested.
   The placed building stands at height 0 above the terrain: its world height is the surface
   height of S-0106 (claim 2) at the centre of the named cell, which on a flat cell is 104 leptons
   times the cell's height level (glossary, *height level*). This is the same placement as a
   deployed building's (S-0415, claim 6). The placement point and this height are the building's
   position in leptons, not its reference point (glossary, *reference point*: the centre of the
   foundation).
   - Backing: `measurable` (scenario PL1; run 4 for the placement point of F4 and the world height)
   - Confidence: high (success and failure outcomes, the factory condition, the placement
     point); medium (the world height)

2. **Foundation cells.** A building's foundation cells are every cell of its foundation rectangle,
   with two exceptions: the 8-cell 3 × 3 foundation word of S-0108's vocabulary (Import mapping)
   lacks the cell 2 cells along map *x* and 1 cell along map *y* from the top-left cell (the same 8
   cells as S-0302, claim 13), and the zero-size word has no cells (its foundation rectangle is
   0 × 0). A building type with no foundation cells passes every cell condition of claim 3, since it
   has no cell to test, and claim 5's shroud condition has no cell to judge either. Claim 4 treats
   its rectangle as 0 × 0 (F1).
   - Backing: `measurable` (scenario PL9, parts a and b for the missing cell, part c for the
     zero-size word's cell conditions and its adjacency ring, part d for its shroud condition.
     PL6 e distinguishes the rectangle's shroud test from the eight-cell occupancy list)
   - Confidence: high (the rectangle and the missing cell); medium (no cell conditions for the
     zero-size word); low (its adjacency ring, and no shroud condition)

3. **The cell test.** A building type with the place-anywhere flag (Import mapping) may be placed
   whatever its cells hold: none of the conditions (a) to (d) below is asked of it (compare claim
   11). The place-anywhere flag takes precedence over a to-tile field: a type with both is placed
   by the place-anywhere rule, and the to-tile rule below is not asked of it (its to-tile field
   still exempts it from claim 5's shroud condition). For every other type without a to-tile field
   (Import mapping), placement needs every
   foundation cell to pass all of the following (the wall exceptions of claim 8 aside). A type with
   a to-tile field follows the to-tile rule at the end of this claim instead.
   - (a) **Usable.** The cell is a usable cell in the height-aware form (S-0009, claims 4 and 5a).
   - (b) **Nothing in the way.** No aircraft is listed in the cell's ground layer (S-0209, Terms,
     *listed*): the condition makes no test of the aircraft's height or state. S-0209 claim 13
     owns listing: an aircraft in ordinary flight is not listed in either cell layer, while a
     landed aircraft is listed in the ground layer. Thus ordinary flight over an otherwise empty
     foundation cell does not block placement, but a landed aircraft does. No other object occupies the cell (an
     infantry unit, a vehicle or a building), it holds no terrain object, and the ground layer's
     occupancy record (S-0209) shows no standing position taken and no vehicle present. Whatever the
     record shows for buildings and other objects does not matter; buildings are caught by the
     object condition. Two kinds of type have an object condition of their own instead:
     - A gate-flag type, or a type with the laser-fence-post flag (Import mapping): an object in
       the cell does not refuse when it is a building of the placing house whose type has the
       laser-fence flag; any other object still refuses, and so does the occupancy record as above.
       Nothing removes that laser-fence building when a post type is placed over it (claim 9
       removes laser-fence buildings only for gate-flag types). Whether the two then stand in the
       same cell, or the placement is refused later, is not established (Open questions 1).
     - A type with the laser-fence flag (Import mapping): only a building or a terrain object in
       the cell refuses. Aircraft, infantry units and vehicles in the cell, and the occupancy
       record, do not. This rule applies whatever the type's gate or post flags say.
   - (c) **No overlay.** The cell holds no overlay. Ore and gem cells therefore refuse placement
     (S-0401 owns the resources), as do walls of any house, other overlay types, bridge parts and
     crates. Claim 8 gives the wall exceptions: a cell that meets one of them must still pass (a)
     and (b), and is exempt from (c) and (d). A type with the laser-fence flag has one more
     exception: a cell whose overlay is ore, gems, or the overlay type with registration index 126
     in the overlay registry passes (c) and (d) together when it is not marked as part of a bridge
     and its slope shape is 0; the buildable flag, the naval range and the float multiplier of (d)
     are not asked. Every other overlay still refuses it, except that claim 8's wall exceptions
     apply to a laser-fence type as to any type: a cell that meets one of them is exempt from (c)
     and (d) for it too.
   - (d) **Ground.** For a type that is not water-bound, all of these:
     - the cell is not marked as part of a bridge: it is not a high-bridge cell (S-0211, claim
       15) and does not carry the bridge-body mark of S-0010 claim 25; that mark's band can include
       cells without overlays,
     - its slope shape (S-0010, claim 6) is 0, and
     - for a type without the naval flag, the buildable flag *B* of the cell's land type (S-0211,
       Terms and claim 2 c) is *yes*, or
     - for a type with the naval flag, *B* is not asked: the cell's tile number lies in the water
       role range of S-0010 (claim 5 and table R), the 14 tiles from the water role's start. Where
       the water role is unset, that range covers tile numbers 0 to 12 (S-0010,
       `quirks.unset_role_range`), and in the theater with index 5 (the theater index of S-0009's
       Import mapping) the original forces the water role unset whatever the theater data says
       (S-0010, claim 4).

     For a water-bound type (claim 21) instead: the land multiplier of the cell's land type
     for the *float* locomotion class (S-0211) is not 0. The bridge, slope, buildable-flag and
     tile-range conditions do not apply to a water-bound type, whatever its naval flag says. So a
     water-bound type may stand on a sloped cell, or on a cell that a high bridge marks without any
     overlay in it (S-0211, claim 15), when the land type's float multiplier is not 0. A cell that
     holds a bridge overlay is refused by (c), whatever (d) says of it. This spec owns where a
     building may stand; S-0211 owns the land types, *B* and the multipliers it reads.
   - **To-tile rule.** A type with a to-tile field needs only one of its foundation cells to pass,
     not every cell. That cell must pass (a), (c) and (d) as for any type, the exceptions of (c)
     included: a cell that meets one of claim 8's wall exceptions is exempt from both (c) and (d),
     and for a to-tile type with the laser-fence flag the laser-fence overlay exception of (c) also
     applies. In place of (b) it must hold no building, and its tile must pass the tileset test of
     S-0010 (claim 15: a tile of a tileset with the morphable flag, or a tile number outside the
     tile range, which always passes). Infantry units, vehicles, aircraft, terrain objects and the
     occupancy record in that cell do not matter. The other foundation cells are not tested. A
     type with no foundation cells passes, as claim 2 says. A to-tile type that also has the
     laser-fence, gate or laser-fence-post flag keeps the one-cell rule, but in that cell its own
     object condition of (b) replaces the no-building and tileset conditions.
   The building being placed never counts as an object in its own foundation cells, so it does not
   block itself. A unit of the placing house standing in a foundation cell blocks the placement like any
   other unit; it is not moved away.
   - Backing: `measurable`, through these scenario parts:
     - PL3 parts a to n and r for the ordinary rule (a, b, c and d, the gate and post object
       condition), part o for the height-aware usable test, part p for an aircraft in flight,
       part q for the fence building under a post type, parts a, e, i, j and s for the
       place-anywhere flag,
     - PL3 part t for the to-tile rule, part w for the to-tile rule combined with the laser-fence,
       gate and post flags, part x for the precedence of the place-anywhere flag over a to-tile
       field,
     - PL3 part u for the laser-fence object condition and its overlay exception (ore and gems,
       the overlay with registration index 126, another overlay type refusing, the slope, the
       bridge mark, and the buildable flag, naval range and float multiplier not asked), part v
       for claim 8's wall exceptions under the to-tile and laser-fence rules,
     - PL4 parts a to f for the water-bound and naval ground rules; S-0211's scenarios L15 and L21
       also observe the naval range and the high-bridge cells; L21 part d observes the
       bridge-body mark's placement refusal on cells without overlays.

     The occupancy-record condition of (b) on a cell with no object in it is `unmeasured`:
     no way is known to set up a record whose object has gone.
     If the test map cannot hold ore in a cell that a high bridge marks (PL3 part *u*10), the bridge
     mark of the laser-fence overlay exception is `unmeasured` too: every other bridge-marked cell
     known either holds a bridge overlay, which refuses whatever the mark says, or holds no overlay,
     where the exception is not asked, so no other setup separates the mark.
   - Confidence: high (a, c, the ore and unit refusals, d for types that are not water-bound apart
     from the bridge cells), medium (high-bridge cells and other bridge body cells in d, b's
     occupancy-record part, a landed aircraft refusing and ordinary flight not blocking placement,
     the laser-fence-post and gate
     exception, d for water-bound types, including the skipped slope, bridge and tile-range
     conditions, the to-tile rule, the precedence of the place-anywhere flag), low (the
     laser-fence building under a post type, the laser-fence rule, which no
     stock type uses, the to-tile rule combined with the laser-fence, gate and post flags, and the
     wall exceptions for to-tile and laser-fence types)

### Conditions checked when the order is given

4. **Adjacency.** A building can be placed only when some cell within *A* + 1 cells of its
   foundation rectangle holds a supporting building, where *A* is the type's adjacency. Precisely:
   the cells looked at are those of the rectangle that extends the foundation rectangle by *A* + 1
   cells on each of its four sides, less the foundation rectangle itself; so a gap of at most *A*
   empty cells may lie between the new building and its support, along either map axis or both. A
   building in one of those cells supports the placement when
   - it is owned by the placing house and is base-normal, or
   - the build-off-ally option is on, its owner allies the placing house (S-0412, Terms: the
     one-way relation, read in the direction from the building's owner to the placing house), and
     it is ally-eligible. A house counts as allying itself, so with the option on its own
     ally-eligible buildings support it too, base-normal or not.
   Any building in the cell counts, whatever its state: in its construction state, being sold, or
   damaged. Adjacency is a condition of the place order only: a vehicle deploying into a building
   (S-0415) uses the cell test of claim 3 but not this condition. The foundation rectangle, not the
   cell list of claim 2, decides which cells are looked at. A building is in a cell only when it covers the cell
   (Terms, *foundation cells*), so the missing cell of the 8-cell 3 × 3 foundation holds no
   supporter. Since that cell lies in the middle of an edge, any ring that contains it also contains
   a covered cell of the same building: this cannot change an outcome, and no scenario tests it. The
   absent-key default of *A* is 3 cells.
   - Backing: `measurable` (scenarios PL2, parts a to e, part e for a negative *A* (F1), and PL5,
     parts a to e; PL5's part e separates the two directions of the one-way relation, because only
     the placing house allies the owner). The missing-cell supporter detail is `unmeasured`:
     every ring containing that cell also contains a covered cell of the same building, so no
     placement outcome distinguishes the detail.
   - Confidence: high (the ring of *A* + 1 cells, own base-normal buildings); medium (allies, the
     option's default, the states that count, the absent-key defaults, the direction of the
     relation)

5. **Explored ground.** Every cell of the foundation rectangle must be free of the placing house's
   shroud (S-0409, claims 1 and 12, owns the shroud: the explored state of the placing house). A
   type with a to-tile field (Import mapping) is exempt, whatever its foundation cells hold.
   - Backing: `measurable` (scenario PL6, parts a to c and e for the shroud condition and part d for the
     exemption, with a test type that has a to-tile field; part d compares two runs, so it does not
     need the to-tile rule of claim 3 to be known)
   - Confidence: high (the shroud condition); low (the to-tile exemption: no stock type with a
     to-tile field is offered, so no observation of stock play backs it)

6. **When claims 4 and 5 are judged.** Claims 4 and 5 are conditions on giving the order. Applying
   the place order (claim 1) does not check them again: a supporting building destroyed, or a cell
   shrouded again, between the issue of the order and its application does not stop the placement.
   - *Human input in the original.* When the player clicks to place, an order is issued only if both
     hold; otherwise the placement refusal announcement is requested and no order is issued.
     No game-wide refusal sound is requested. Adjacency is judged at the
     click, with the building, the cell under the cursor and the buildings as they are then. The
     shroud condition is not: the click uses the cursor shroud result, which is recomputed only
     when the cell under the cursor changes while placement mode is active. If a foundation cell is
     revealed, or shrouded again, while the cursor stays on the same cell, the click still
     uses the old result. The cursor shroud result when placement mode begins, before the first
     change of cell, is not established.
   - *Our engine.* With `quirks.placement_shroud_at_cursor` (default on, the original's behaviour),
     the shroud condition of an order from human input is the cursor shroud result as just
     described. With the switch off, and always for an AI house (which has no cursor), claims 4 and 5
     are legality conditions of the place order, judged when the order is issued, and are not
     judged again when it is applied. This follows the AI-house ruling (glossary, *AI house*: AI
     orders pass the same legality rules as a person's input). The adjacency timing and the human
     shroud timing with the switch on are the original's; judging an AI house's order at issue is
     the AI-house ruling, not a departure of this spec.
   - Backing: `measurable` (scenarios PL6 for the shroud timing at the click, PL7 part a for
     adjacency lost after the click and PL7 part b for a cell shrouded again after the click).
     If PL7 b cannot establish its required event timing, the no-recheck shroud subclaim is
     `unmeasured`: the stale cursor result would otherwise mask a possible application-time
     check. The mode-start cursor result is `unmeasured` and not specified here: no scenario
     establishes it before the first cursor-cell change.
     The AI issue-time rule is `unmeasured`: it follows the project AI-house ruling and has no
     counterpart reachable through the original's computer-player input.
   - Confidence: high (adjacency at the click, no check when the order is applied); medium (the
     shroud result held from the last change of cell)

7. **Which cell a click names.** In placement mode the clicked cell is the centre of the foundation:
   the place order names the cell (*x* − ⌊(*W* − 1)/2⌋, *y* − ⌊(*H* − 1)/2⌋) for a click on cell
   (*x*, *y*), where *W* × *H* is the size of the bounding rectangle of the foundation cells (F2).
   A building with no foundation cells has a bounding rectangle of 1 × 1, so the click names the
   top-left cell itself. This is an input rule; S-0411 (claim 10) owns placement mode.
   - Backing: `measurable` (scenarios PL1 and PL9, part c)
   - Confidence: medium (the displacement); low (the 1 × 1 size for no cells)

### Walls, wall towers and gates

8. **Wall exceptions to the cell test.** A cell that holds an overlay is exempt from conditions (c)
   and (d) of the cell test (claim 3), though it must still pass (a) and (b), when its wall owner is
   the placing house and the cell's overlay type (by its registration index in the overlay registry)
   and the building type being placed form one of these pairs:

   | Overlay type of the cell | Building types exempt on it |
   |---|---|
   | registration index 0 or 2 | a type whose wall overlay is that overlay type, on a damaged wall only; the wall tower type; each of the first pair of gate types |
   | registration index 26 | a type whose wall overlay is that overlay type, on a damaged wall only; each of the second pair of gate types |

   So a wall can be rebuilt on the house's own damaged wall of the same kind, and a wall tower or a
   named gate can stand on the house's own walls; every other overlay, and every wall of another
   house, still refuses placement.
   - Backing: `measurable` (scenario PL8: parts b, c and d for registration indices 0 or 2 and
     the wall tower type, part e for the first pair of gate types, part j for index 26 and the
     second pair)
   - Confidence: medium

9. **What placement does to walls underneath.** When a building is put on the map, what it removes
   underneath depends only on its type. Every type is in exactly one row of this table:

   | Type | Cells | Removed there |
   |---|---|---|
   | The wall tower type | its top-left cell | the wall, when its wall owner is the placing house and its overlay type has registration index 0 or 2 |
   | Not the wall tower type; has a wall overlay | its top-left cell | the wall, when its wall owner is the placing house and its overlay type is the type's wall overlay |
   | Neither of the above; a gate-flag type | each of its foundation cells | the wall, when its wall owner is the placing house and its overlay type has registration index 0, 2 or 26; and any building of the placing house with the laser-fence flag (removed from play without refund) |
   | Any other type | none | nothing |

   The row follows from the type, not from what its cells hold, so the rows do not combine: a type
   of the first row removes nothing when its top-left cell holds no such wall, even when it also
   has a wall overlay or the gate flag. So a gate-flag type that is the wall tower type, or that has
   a wall overlay, can remove the wall in its top-left cell, and no wall or laser-fence building in
   its other foundation cells. The first two rows look only at the top-left cell, whatever the
   foundation's size.
   Each wall removal is the silent wall sale of S-0408 (claim 12) and has that claim's conditions:
   the wall is removed only when the placing house is human-controlled (in our engine any player
   house, glossary *AI house*) and the wall's builder (Terms) exists and does not have the
   unsellable flag; otherwise the wall stays in its cell, and the placement goes on without
   removing it (the cell test of claim 8 has already passed). No refund, no sound. The removal of
   a laser-fence building is not a wall sale and has no such conditions.
   - Backing: `measurable` (scenario PL8: parts c, e and j for the removals, part f for the
     builder's conditions, part k for which type is the builder, parts l and m for which row applies
     to a type that has several of these properties, part n for the laser-fence buildings); the part about a placing house that is not
     human-controlled is `unmeasured` in the original: no input makes a computer-controlled house
     place a building, so no scenario can reach it, and in our engine it follows from the
     AI-house ruling
   - Confidence: medium (walls, the builder, which row applies); low (the laser-fence removal,
     which no stock type exercises)

10. **A wall type becomes its overlay.** When a building of a wall type passes the cell test, no
    building enters play and none remains: its wall overlay is created in the cell instead, with the
    placing house as its wall owner. On a damaged own wall (claim 8), when
    claim 9's removal of the old wall takes place, the cell ends with an undamaged wall.
    When claim 9's conditions fail (the builder of the wall type's overlay, Terms, has the
    unsellable flag, the only one of claim 9's conditions that can fail for a player house in our
    engine), the damaged wall is not removed. The placement still passes the cell test, no building
    of the wall type remains and the item is spent. The original's final cell contents are not
    specified here: retaining the old damaged wall is a hypothesis, not a normative original rule.
    **Pending resolution, our engine retains that wall, its owner and its damage state unchanged.**
    This is an explicit provisional implementation rule, not a claim of verified equivalence and
    not a permanent departure. PL8 f tests the hypothesis with an outcome-based falsifier.
    A wall type that has no wall overlay (the to-overlay field absent) has no defined behaviour in
    the original: once its placement passes the cell test, the original fails (a fault, not a
    refusal). This is a departure with no switch: the importer
    warns, and a type with the wall flag and no wall overlay is imported as an ordinary building
    (the wall flag is ignored), so claims 10 and 11 do not apply to it.
    - Backing: `measurable` (scenario PL8, parts b, f and j for successful repair and acceptance
      despite failed removal; every part that places a wall type for the conversion).
      The provisional retention rule is `unmeasured`: it is an interim rule of our engine, not
      established original behaviour; PL8 f is a discriminating investigation, not backing for a
      normative original cell-content claim. `unmeasured` for the
      wall type with no wall overlay: the original does not give a result a scenario could record
      (it faults), so the departure has nothing to be compared with
    - Confidence: high (the conversion and the explicit interim engine rule); medium (successful
      repair); low (the original cell-content hypothesis after failed removal; the fault of the
      original for a wall type with no wall overlay)

11. **Wall fill.** When a building whose wall overlay (Terms) is an overlay type with the wall flag
    of an overlay type (Terms), and whose type is not a firestorm-wall type, is placed through a place order, its gaps are filled in the same tick (the
    types that this applies to are normally the wall types). Let *L* =
    ⌊*G* / 256⌋ cells, where *G* is the type's guard range in leptons (Import mapping), rounded
    toward −∞ (F3). In each of the four map directions, let *c*₁, *c*₂, … be the cells leading away
    from the placed cell, *c*₁ its neighbour; only *c*₁ to *c*<sub>*L*</sub> of that direction count.
    The direction is filled when the nearest of *c*₁ to *c*<sub>*L*</sub> that holds the type's wall
    overlay with the placing house as its wall owner is some *c*ₖ, and every cell *c*₁ to *c*ₖ₋₁
    before it passes the cell test for the type: those *k* − 1 cells each receive a new wall of the
    same type. For a wall type each of them is placed by claim 10 and so becomes the type's wall overlay in
    its cell.
    For a type that is not a wall type (the building type itself has no wall flag, while its wall
    overlay has the wall flag of an overlay type), each filling building is created and put on the
    map as a building of that type in its gap cell, passing no further test than the cell test
    just described; it follows claim 9 like any building put on the map, and whether it shows a
    construction state is not established (Open questions 1). Otherwise nothing is filled in that direction, either because no
    such wall lies within *c*₁ to *c*<sub>*L*</sub>, or because a cell before the nearest one fails
    the cell test. The cell test here is the one of claim 3 without the place-anywhere exemption: a
    fill cell of a type with the place-anywhere flag still has to pass (a) to (d). A damaged wall
    counts too: the nearest cell that holds the type's wall overlay with the placing house as its
    wall owner is that direction's *c*ₖ, whether or not its wall is damaged. That cell is not put
    through the cell test, so the fill neither repairs nor replaces it, and cells beyond it do not
    affect the fill. The four directions do not interact:
    they share no cell, and the walls of one direction never change what another direction finds.
    So gaps of up to *L* − 1 cells to another wall of the same kind and owner are closed, and with
    *L* ≤ 1 nothing is filled. The filling walls cost nothing. A wall placed by the fill does not fill
    further. Each filling wall counts as a building created under S-0202 (claim 10), even when it
    becomes the type's wall overlay in its cell, and takes one creation draw. The number of draws taken is therefore the
    number of walls added by the fill; which direction took which draw cannot be observed, so the
    directions may be handled in any order.
    Firestorm-wall types (Import mapping) have a fill rule of their own, which no stock type
    uses and which this spec leaves open.
    - Backing: `measurable` (scenario PL8: parts a, g, h and i for the directions, the limit, the
      stop at a failing cell and the draws; part o for the cell test without the place-anywhere
      exemption; part p for an own damaged wall as the nearest wall; part q for a filling wall
      not filling further; part r for a type that is not a wall type; part s for a wall overlay
      without the wall flag of an overlay type, which fills nothing)
    - Confidence: high (directions, the fill, no cost); medium (the limit *L* − 1, the stop at a
      failing cell, the nearest own wall deciding, the no-exemption detail, the damaged wall as
      nearest, no chained fill); low (the draw count, the filling buildings of a type that is not a
      wall type)

### Upgrades

12. **Placing an upgrade.** An upgrade type is not placed by the cell test. Its place order succeeds
    when the named cell holds a building of the placing house whose type is the one the upgrade names
    and which has room for it: with the upgrade's level field absent (−1), the building's upgrade
    count is below its type's upgrade count (Import mapping); with a level field from 1 to 3, the
    building's upgrade count is 0. Any other level value refuses. On success the upgrade is added to
    that building and no building of the upgrade type enters play. Its effects:
    - the building's upgrade count becomes *c*′ of F5: the count raised by 1 (level field absent)
      or by the level field *n*, capped at its type's upgrade count except on a building of the
      wall tower type, whose count is not capped,
    - the building's health becomes its type's strength (full health), whatever its damage,
    - the upgrade type is recorded in upgrade position *c*′ when 1 ≤ *c*′ ≤ 3. A *c*′ of 0 or above
      3 records nothing in the three positions, in the original as in our engine (F5); what else the
      original does with it is the departure stated after this list. So a level field *n*
      above 1 on a building with no upgrade records one upgrade, in position *n* (or in the position
      of the capped count), and the positions below it stay empty; every upgrade recorded counts in
      the power of S-0405 and in the prerequisite check of S-0407 (claim 8 c), and the part of the
      level field that is not recorded only uses up room,
    - the capped and the uncapped count differ only when the level field *n* is above the target
      type's upgrade count *U*: then a building of the wall tower type reaches the count *n* and
      records the upgrade in position *n*, where any other building reaches *U* and records it in
      position *U* (nothing when *U* is 0). So a level-field upgrade on a wall-tower building whose
      type has upgrade count 0 is recorded and counts, and every later upgrade on that building
      refuses (its count is no longer 0, and is at least *U*),
    - the house's power and the types offered to it take the upgrade into account, at the time
      S-0405 (claim 7) and S-0407 (claim 12) give, and whether the house has radar is decided again
      under S-0410's rules (claim 1; what that can change is S-0410's).
    *Departure (bad data).* When *c*′ is 0 or above 3, the original records the upgrade type in no
    position, so it adds no power and meets no prerequisite, but it also alters some other part of
    the building's state; which part, and what that changes, is not established, and for the larger
    counts it may make the original fail. Our engine records nothing and changes
    nothing else: the upgrade only uses up room and restores health, as the list above says. *c*′
    above 3 needs a target type whose upgrade count is above 3 (a level field reaches at most 3, and
    a level-less upgrade needs room below the count), and the importer warns about every such
    target type; *c*′ of 0 is the level-field case of `quirks.upgrade_level_without_room` (Edge
    cases and quirks), whose importer warning covers it.
    A place order of an upgrade type that names such a building passes claim 4 whatever its ring
    holds, for human input and for an AI house alike; claim 5's shroud condition is still asked of
    the upgrade type's foundation rectangle at the named cell. This follows S-0404 claim 10's
    legality rule: an upgrade place order is exempt from adjacency but remains subject to explored
    ground; ordinary building place orders retain both conditions. The upgrade target's owner,
    type and room must still satisfy this claim when the order is applied: unlike adjacency and
    explored ground, those target conditions are checked again. A target lost, captured or filled
    between issue and application can therefore refuse the upgrade. S-0408 claim 5 owns which
    upgrade a sale removes: it sells the position named by the count, not an arbitrarily selected
    upgrade. In particular, selling the first of three contiguous upgrades is not an available
    ordinary sale operation.

    **Upgrade precedence (Q-0798 question 3).** The numbered positions, not a separate
    installation history, determine observable precedence. Empty positions are not compacted
    by installation or sale. A successful installation replaces any type already recorded in
    position *c*′; it does not search for the lowest empty position. The following distinction
    applies to S-0308 claim 12's weapon resolution and S-0215 claim 32's attack gate:

    | Use | Eligible upgrades and order |
    |---|---|
    | Weapon provider for a requested slot, count *c* from 1 to 3 | Positions 1 through *c*, in increasing position number, skipping empty positions and types without an ordinary weapon in that slot. The first provider wins. Positions above *c* do not participate even if still occupied. If none supplies the slot, S-0308 claim 12's occupant and host fallbacks apply. |
    | Weapon provider, count 0 | No upgrade participates, even if positions retain types after a sale. |
    | Attack gate | Applies only when the count is nonzero and position 1 is occupied. It tests that position's type only. An empty position 1 bypasses this gate: it does not substitute the first occupied higher position. S-0215 claim 32 owns the ordinary firing path used when the gate does not apply. |
    | Count above 3 | There is no fourth valid upgrade position. The gate still refers only to position 1. For weapon resolution our engine considers positions 1 to 3 only, in that order; it ignores the excess count. The original's result after those three fail to supply the requested slot is unspecified under the bad-data departure below. |

    Thus a level-2 installation on an empty target with room records its type in position 2:
    it can supply weapons, but it does not activate the attack gate. Adding a level-less
    upgrade next records it in position 3; the provider order is 2 then 3, while the gate
    remains bypassed. Selling that last upgrade leaves position 2 first among weapon
    providers. Selling the level-2 upgrade itself without the turret flag leaves count 1
    and no recorded provider; another level-less installation then occupies position 2
    again, not position 1.

    A sale of a turret-flag upgrade can instead leave count 0 and occupied lower positions
    (S-0408 claim 5). Those retained types provide no weapons while the count is 0. A new
    level-less installation at count 0 records position 1 and activates that position's gate.
    Higher retained positions remain ineligible until the count reaches them, and a later
    installation at such a position replaces its retained type. A wall tower uses exactly
    the same precedence rules: its uncapped count does not add valid positions or make
    the last installed excess upgrade the first provider.

    For counts above 3 the original may fail while resolving a slot not supplied by the
    three valid positions; no stable additional provider order is specified. Restricting
    our engine's search to the three valid positions, with the ordinary fallbacks if none
    supplies the slot, extends this claim's existing bad-data departure and importer warning.
    There is no new imported ordering field. Q-0798 questions 1 and 2 concern firing attempts
    and delayed shots, not installation: their owners are S-0301 claims 9 and 14 and
    S-0215 claim 32.
    - Backing: `measurable` (scenario PL10: parts a and b for the room rule, part c for a type
      without room, part d for the position, the full repair and the power and prerequisite
      effects, part e for the adjacency replacement, part f for the shroud condition, part g for
      level values that refuse, part h for a count above 3 that records no power and no position,
      part i for the uncapped count of the wall tower type, PL16 for weapon-provider precedence,
      the position-1-only gate, sale retention and replacement, part j for the application-time
      target recheck, unmeasured if its required event timing cannot be established); `unmeasured` for the original's other
      effects of a *c*′ of 0 or above 3 (no game concept is known that this other alteration of
      the building's state changes, so there is no observable to propose, the larger counts risk a fault, and
      our engine's departure does not depend on the answer). The original's weapon-resolution
      outcome beyond the three valid positions is also `unmeasured`: it depends on unrelated
      state and may fault; PL16 d proposes a diagnostic run, not a deterministic original result.
      The bounded search in our engine is `unmeasured` design backing for the safe departure.
      Also `unmeasured` for
      deciding again whether the house has radar (every input of S-0410's claim 1
      that an upgrade can change is a power change, which S-0410's claim 2 already lists as
      requesting that decision; an upgrade that changes none leaves the answer as it was, so
      no observation separates a decision made because of the upgrade from none)
    - Confidence: medium (the level rule, the full repair, the position, the replacement of the
      adjacency condition, the wall tower exception, no position recorded for a *c*′ of 0 or above
      3); high (position order, count-limited weapon providers, position-1-only attack gate,
      no compaction and replacement of a retained type); low (several levels at once, deciding
      radar again, the shroud condition for an upgrade, any original outcome beyond three positions)

### Other houses and the original's computer player

13. **AI houses place like players.** In our engine an AI house places buildings only through place
    orders under claims 1 to 12 (glossary, *AI house*). Informative, for AI designers (the
    original's computer player is out of scope): it places its buildings by its own base-planning
    rules, which apply neither the adjacency nor the shroud condition (they are input checks of the
    local player). Reproducing its base layouts is a matter of choosing cells that pass these
    claims.
    - Backing: `unmeasured`: it is a rule of our engine; the original's computer placement is out of
      scope
    - Confidence: high

### Construction completion and initial facing

14. **Construction processing.** While *construction* is current, every processing chooses
    a delay of 1 tick, with no timing draw from the scenario random stream. Subject to
    S-0215 claim 2's general processing gates, it is therefore processed on consecutive ticks.
    Its first processing starts the construction animation range again at stage 0, even when
    placement has already begun that range. The rate of the behaviour does not decide this
    delay. Random draws caused by the effects of a completed building, such as a free unit
    (S-0402 claim 18), are separate from this no-draw timing rule.
    - Backing: `measurable` (PL11 a and b)
    - Confidence: high (the delay, no timing draw and stage restart)

15. **Handover after build-up.** Once the first *construction* processing has occurred,
    a later processing that finds the build-up completion report ends construction in that
    same tick. It requests the *idle* body animation state, completes the building's opening
    effects (S-0108 claim 11, including the conditional free unit of S-0402 claim 18), and
    queues *guard*. That queued *guard* becomes current at the building's second per-tick
    start of the same tick (S-0215 claim 11), and is first processed on the next tick.
    The completion does not itself clear the building's target. A target inherited at deploy
    therefore survives this handover, unless another rule has removed or replaced it.
    An armed building may then start *attack* from *guard* under S-0215 claim 32.

    For an uninterrupted build-up with at least 2 stages and a positive stage delay,
    the processing on the tick the last stage is reached already sees its completion report:
    there is no additional tick between that report and handover. In terms of the first
    construction processing tick *C*, the handover is at *C* + (*n* − 1) × *d_b* ticks,
    using S-0108 claim 5's actual construction range (F6). A gate uses the same handover
    rule and its gate range, not the ordinary sprite-half count. The special theater-refresh
    range of that claim also remains applicable.

    A zero-delay range reports completion at the next building turn despite its frozen
    stage. Thus a type with no build-up sprite, whose default range has one stage and zero
    delay, hands over at the processing after its first construction processing: *C* + 1
    ticks in uninterrupted play. It does not wait forever for a sprite that does not exist.
    F6 counts from *C*; claim 17 and F7 give *C* and the handover from the placement or deploy
    tick. Positive-delay one-stage ranges and non-positive stage counts never report
    completion (claim 18).
    - Backing: `measurable` (PL11 a to d and f). Processing gates that interrupt a building turn
      are `unmeasured` here: PL11 supplies uninterrupted runs only; their legality and timing
      remain owned by S-0215 claim 2 rather than a new interruption rule in this spec.
    - Confidence: high (same-tick report handover, queued guard, target retained and opening
      effects); medium (F6 and the no-sprite case)

16. **Initial building facing.** A place order puts an ordinary building on the map with
    facing 0 facing units, map north. This is the facing used by S-0408 claim 8 for passenger
    release; for a turreted building its turret facing also starts at 0. A building has no
    independently turning body: this same facing is its turret facing when it has a turret
    (S-0206, Terms). It is not the barrel elevation used by rendering.
    The click names a cell, not a direction, and the facing of the factory building does not
    change this result. Later aiming follows S-0215 and S-0310. There is no imported
    placement-facing field.

    A successful vehicle deploy gives the building the same initial facing 0, including its
    turret, rather than the vehicle's final body facing or the building type's deploy facing.
    S-0415 claim 6 already supplies the insertion-facing result requested by Q-0220 question 1
    and owns deploy completion; this claim makes the turret consequence explicit for question 2.
    It describes entry, not a guarantee against later aiming at an inherited target.
    For question 3, a deploy facing outside 0 … 255 never permits deploy completion:
    S-0415 F1's rounded vehicle facing always lies in 0 … 255 and cannot equal that imported
    integer. No building appears and there is therefore no deployed-building facing to reduce
    modulo 65536. A place order still gives facing 0 regardless of that field.
    - Backing: `measurable` (PL12 for placement; PL13 for deployment, turret facing and the
      out-of-range case; S-0415 D15 already proposes the ordinary deploy-facing observation)
    - Confidence: high

### When construction begins and whether it ends

17. **From entry to the first construction processing.** A building that enters play under
    construction (claim 1, a restored replacement under S-0524 claim 15, or a deploy under
    S-0415 claim 6) begins its construction state at
    entry: its stage is 0 and its stage timing counts from the entry tick (S-0108 claim 5). At
    entry it has no current behaviour and *construction* queued. A rule that asks for its
    behaviour while none is current sees the queued *construction*. The queued *construction*
    becomes current only at one of the building's per-tick starts (S-0215 claim 11), and the
    first start of a tick does not apply while the construction state lasts. It therefore becomes
    current at the second per-tick start of the first tick on which the building's ready mark
    (S-0215, Terms) is set, and its first processing *C* is on the next tick.

    The construction state sets the ready mark on a building turn on which (a) the stage advances
    onto the last stage of the range, stage *a* + *c* − 1 of S-0108 claim 5, or (b) the range's
    delay is 0, so that its stage does not advance. A stage that runs past the end and wraps
    (S-0108 claim 6) does not set it. A deploy also sets the ready mark when the building enters
    play. A place order and a restored replacement instead leave the mark clear at entry,
    even without a build-up sprite. Healing an existing building through restoration does not
    restart construction or change its mark (S-0524 claim 15). The results, for uninterrupted
    play (F7):
    - **Placed building, at least 2 stages and a positive delay.** The place order is applied in
      the orders part of tick *T* (S-0203 claim 7), so the building's first turn is on *T* + 1. The
      build-up runs once from entry and lands on its last stage at *R* = *T* + (*n* − 1) × *d_b*,
      which sets the ready mark, and *construction* becomes current in that tick. On *C* = *R* + 1
      the restart of claim 14 shows stage 0 again, and the build-up runs a second time to the
      handover of claim 15 at *H* = *C* + (*n* − 1) × *d_b* = *T* + 2(*n* − 1) × *d_b* + 1. The
      build-up animation is therefore seen twice, and the last stage of the first run is shown
      on tick *R* only (`quirks.placed_buildup_twice`).
    - **Placed building, zero-delay range** (including a type with no build-up sprite): the mark
      is set at the first turn, *T* + 1, so *C* = *T* + 2 and *H* = *T* + 3.
    - **Restored replacement (S-0524 claim 15).** The same range-reporting and restart rules
      apply as for a placed building, including two build-up runs for at least 2 stages and a
      positive delay. Let *U* be its actual first eligible building turn, not an assumed tick
      after the restoration action. For a zero-delay range, the mark is first set at *U*,
      *C* = *U* + 1 and *H* = *U* + 2. For a positive-delay multi-stage range, let *R* be the
      turn that first lands on the last stage: *C* = *R* + 1 and *H* follows F6. The order-tick
      expressions above do not fix *U* or *R* for restoration. Claim 18's non-completion rule
      applies to restored replacements as to placed buildings.
    - **Deployed building:** the mark is already set at entry, so *construction* becomes current
      at the second per-tick start of the building's first turn *U*, whatever its range, and
      *C* = *U* + 1. The handover follows F6. Which tick *U* is, relative to the deploy, is
      S-0415's (with S-0203 claims 4 and 10). The one or two ticks of build-up shown before *C*
      are replaced by claim 14's restart, so a deployed building shows one build-up run.
    - A gate uses its gate range (*n* = *G* + 1), and a theater-refreshed range its own count and
      delay (S-0108 claim 5), under the same rules.

    None of these steps takes a draw from the scenario random stream. A tick on which the
    building's update does not run (S-0215 claim 2) delays the processing steps and the
    ready-mark turns. Stage timing still runs on the tick number (S-0203 claim 9).
    - Backing: `measurable` (PL11 a to d and f; S-0524 claim 15 for restored replacements,
      with its proposed restoration readiness scenario)
    - Confidence: medium (the second build-up run and resulting *R*, *C* and *H* for placed
      buildings and restored replacements); high (no draw; the clear entry mark for placement
      and restoration, including missing art; the deploy's mark at entry and *C* = *U* + 1)

18. **Ranges that never report completion.** The ready mark of claim 17 (a) needs the stage to
    land on the last stage, and the stage is advanced before it is compared. A range with a
    positive delay and a count of 1 goes from stage 0 to stage 1, past its end, and wraps back to
    0 in the same turn, so it is never on its last stage after an advance. A count of 0 or less
    behaves the same way. A wrapped delay is never 0 (S-0108 claim 6), so rule (b) never applies
    either. Such a building never completes its construction:
    - a placed one never makes *construction* current: it keeps it queued and shows stage 0 for
      as long as it stands,
    - a deployed one makes *construction* current but never hands over.

    In both cases it gets no opening effects, never counts as constructed (S-0408 claim 3 a) and
    never takes *guard*. The ranges that do this are, by S-0108 claim 5:
    - a build-up sprite of 2 or 3 image frames (*n* = 1) when the build-up time gives a
      positive delay, or one of 0 or 1 image frames (*n* = 0, *d_b* = 1),
    - a gate with *G* = 0 when the build-up time gives a positive delay,
    - a theater-refreshed range with *N_b* = 1 or *N_b* = 0.

    No stock data reaches this case. `quirks.single_stage_buildup_never_completes` (default on).
    The importer warns about such types in both states of the switch. This switch is shared
    with selling: S-0408 claim 17 owns the same failure to complete for a positive-delay
    build-down with 1 stage or fewer, and the resulting absence of a sale refund or removal.
    - Backing: `measurable` (PL11 e; S-0408 S10 for the selling effect)
    - Confidence: medium

### Placement display and refusal messages

19. **Placement hover is not an action cursor (Q-0277).** While building placement mode is
    active and the pointer is inside the battlefield, the pointer uses S-0214's ordinary
    arrow, even over a blocked cell or an eligible upgrade target. It does not become a
    deploy, no-deploy, capture or universal refusal cursor. Its frame and top-left hot spot
    are S-0214 claims 21–22's. The candidate foundation is a separate map-aligned display,
    anchored by claim 7, not an image attached to the arrow's hot spot. Moving outside the
    battlefield hides the active foundation display; sidebar and edge-scroll cursor rules
    remain their owning specs'. No building sprite preview is drawn in ordinary skirmish
    placement mode.

    For an ordinary type without a to-tile field, each marked foundation cell uses the
    placement image's frame 0 when that cell is currently explored, passes the cell test
    without the place-anywhere exemption, and the adjacency result saved at the last
    cursor-cell change is positive. Otherwise it uses frame 2 on a flat cell or frame 3 on
    a sloped cell. These are integer image-frame indices, not colours supplied by this spec.
    The display can therefore disagree with the eventual click: adjacency is checked again
    at the click, while the click's shroud result can be stale under claim 6. A place-anywhere
    type can also display a refused cell that its actual placement accepts.

    An upgrade's eligible display instead needs explored ground in that cell and a building
    satisfying claim 12's ownership, target-type and room conditions. Adjacency does not
    decide that display. An eligible upgrade cell uses frame 0; an ineligible one uses
    frame 2 or 3 as above. Neither the pointer nor the display grants permission to bypass
    the issue-time shroud condition or the application-time target check.
    - Backing: `measurable` (PL14)
    - Confidence: high (ordinary arrow and separate display); medium (frame choice,
      saved-adjacency versus live cell checks, upgrade display and no sprite preview)

20. **Placement refusal adds no battlefield text (Q-0282).** A click refused by claim 6,
    including an upgrade click that fails its issue-time conditions, and a place order
    refused when applied under claim 1 add no transient text message to the battlefield
    message list. Their feedback is the placement refusal announcement and the retained
    done item/placement mode. An order ignored for lack of a qualifying factory adds no
    text and requests no sound. No placement-refusal string, screen position, capacity or
    expiry timer is therefore needed; pausing has no placement-message timer to stop.
    Independently generated trigger or other messages are not suppressed by this rule.
    - Backing: `measurable` (PL15)
    - Confidence: high

21. **Which rules layers make a building type water-bound (Q-3087).** A building type's
    water-bound flag (claim 3 d) is decided again in every rules layer whose section for the type
    gives its water-bound field or its locomotion-class field (Import mapping). A layer that gives
    neither, or has no section for the type, keeps the flag (S-0006, claims 3 and 7). In a layer
    that gives one of them:
    - (a) when the layer gives the water-bound field, its value is the flag, whatever class the
      same layer gives,
    - (b) otherwise the flag is on when the class names *float*, in any letter case, and off for
      any other class name of S-0211 (*float-on-beach* and *amphibious* included) and for text that
      names no class. That last case is the original's behaviour for a building type (PL17 part
      g). S-0211 claim 3's departure, which treats such text as absent for a mobile type, does not
      apply to a building type's class field.

    So after all layers the flag comes from the last layer that gives either field, and within that
    layer from the water-bound field when it gives one. A later layer that gives only the class
    overrides an explicit value of an earlier layer: an earlier *yes* followed by a later class
    *wheel* leaves the type not water-bound, and an earlier *no* followed by a later class *float*
    makes it water-bound. A type to which no layer gives either field is not water-bound (but see
    open question 7). An entry with no value, of either field, is absent (S-0006, claim 3; PL17
    parts l and m), and a water-bound value that S-0003 (claim 2) does not recognise counts as not
    given, so (b) or keeping applies (PL17 part k).

    Within this spec's placement rules, a building type's locomotion-class field acts only through
    this flag (open question 8 lists a use outside them that was not traced).
    - Backing: `measurable` (PL17) for parts (a) and (b), the order across layers and the empty and
      unrecognised entries; `unmeasured`
      for the absence of other placement effects: no single run observes every use of the field.
    - Confidence: high (parts (a) and (b), the order across layers and the empty and unrecognised
      entries, PL17); medium (no other
      placement effect)

## Formulas

> **F1. Adjacency ring.** For a foundation rectangle with top-left cell (*x*₀, *y*₀) and size *W* ×
> *H* (0 × 0 for the zero-size word), and adjacency *A*, with *d* = *A* + 1: the cells looked at are
> all (*x*, *y*) with *x*₀ − *d* ≤ *x* ≤ *x*₀ + *W* − 1 + *d* and *y*₀ − *d* ≤ *y* ≤ *y*₀ + *H* − 1 + *d*,
> except those with *x*₀ ≤ *x* ≤ *x*₀ + *W* − 1 and *y*₀ ≤ *y* ≤ *y*₀ + *H* − 1. Integers, in cells; exact,
> no rounding. Any negative *A* empties the ring (for *A* = −1 the ring and the hole coincide; below
> that the rectangle lies inside the hole), so the condition always fails.

> **F2. Click cell.** top-left = (*x* − ⌊(*W* − 1)/2⌋, *y* − ⌊(*H* − 1)/2⌋), with *W* × *H* the size of
> the bounding rectangle of the foundation cells, taken as 1 × 1 when there is no cell. Integers, in
> cells. *W* and *H* ≥ 1, so the division rounds toward zero and toward −∞ alike.

> **F3. Wall fill limit.** *L* = ⌊*G* / 256⌋, *G* in leptons (integer; the guard range converted as
> S-0215 states), rounding toward −∞; *L* in cells. Filled cells per direction: *k* − 1 with *k* ≤ *L*.

> **F4. Placement point.** (256*x* + 128, 256*y* + 128) leptons for top-left cell (*x*, *y*), at the
> world height of claim 1 (104 × ℓ leptons on a flat cell at height level ℓ). Integers, in
> leptons, exact, no rounding.

> **F5. Upgrade count and position.** *c*′ = min(*c* + *i*, *U*), or *c*′ = *c* + *i* on a building
> of the wall tower type, where *c* is the building's upgrade count before the addition, *i* is 1
> when the upgrade's level field is absent and the level field *n* otherwise, and *U* is the target
> type's upgrade count. The upgrade type is recorded in upgrade position *c*′ when 1 ≤ *c*′ ≤ 3; when
> *c*′ is 0 or above 3 it is not recorded (the original's other alteration of the building's
> state in that case is a departure, claim 12). Integers, in upgrade-count units; exact, no rounding.

> **F6. Handover relative to construction processing.** *H* = *C* + (*n* − 1) × *d_b*
> for *n* ≥ 2 stages and *d_b* ≥ 1 tick per stage. For a zero-delay range, *H* = *C* + 1.
> *C* is the first construction processing tick, not the placement tick. Integers in ticks,
> exact multiplication and addition, no rounding. S-0108 owns the floating-point calculation
> and rounding that produce *d_b*. These expressions assume no interrupted building turns
> or processing gates during the interval, and no intervening behaviour or animation change.

> **F7. Construction times from entry.** For a building placed in the orders part of tick *T*,
> with *n* ≥ 2 stages and *d_b* ≥ 1: *R* = *T* + (*n* − 1) × *d_b*, *C* = *R* + 1 and
> *H* = *T* + 2(*n* − 1) × *d_b* + 1. Placed, zero-delay range: *C* = *T* + 2, *H* = *T* + 3.
> Deployed, with *U* the building's first turn: *C* = *U* + 1, and *H* by F6. With
> `quirks.placed_buildup_twice` off, a placed building is treated like a deployed one with
> *U* = *T* + 1: *C* = *T* + 2 and *H* by F6. Integers in ticks, exact, no rounding; the same
> assumptions as F6. Restored replacements use claim 17's actual eligible turns, not the
> orders-part assumption above: zero delay gives *C* = *U* + 1, *H* = *U* + 2; a positive-delay
> multi-stage range gives *C* = *R* + 1, *H* by F6. With the shared switch off, restoration
> gives *C* = *U* + 1 and *H* by F6. Here *U* is its first eligible building turn and *R* its
> first last-stage report turn. All are integer ticks with exact addition, no rounding.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Adjacency absent-key default | 3 | cells | claim 4 |
| Base-normal flag absent-key default | yes | boolean | claim 4 |
| Ally-eligible flag absent-key default | no | boolean | claim 4 |
| Build-off-ally option absent-key default | on | boolean | claim 4 |
| Place-anywhere flag absent-key default | no | boolean | claim 3 |
| Damaged wall threshold | 16 | overlay data value | claim 8 |
| Registration indices of the walls of the first gate pair and the wall tower | 0 and 2 | registration index | claims 8 and 9 |
| Registration index of the walls of the second gate pair | 26 | registration index | claims 8 and 9 |
| Overlay registration index a laser-fence type may stand on | 126 | registration index | claim 3 c |
| Naval placement range | 14 | tiles from the water role's start | claim 3 d; S-0010's water role |
| Naval placement range with the water role unset | 0 to 12 | tile numbers | claim 3 d; S-0010 |
| Missing cell of the 8-cell 3 × 3 word | (2, 1) | cells from the top-left cell | claim 2 |
| Upgrade positions of a building | 3 | positions | claim 12 |
| Level field values that place on an empty building | 1 to 3 | levels | claim 12 |
| Creation draws per filling wall | 1 | draws of S-0202 | claim 11 |
| Construction processing delay | 1 | tick | claim 14, no timing draw |
| From the ready mark that makes *construction* current to the first construction processing | 1 | tick | claim 17 |
| Build-up runs of a placed building with at least 2 stages and a positive delay | 2 | runs | claim 17, `quirks.placed_buildup_twice` |
| Stage counts at or below which a positive-delay construction range never reports | 1 | stages | claim 18 |
| Placement facing | 0 | facing units | claim 16, map north |

## State and ordering

| State | Event | Result |
|---|---|---|
| Item done, placement mode on (human input) | click | For ordinary placement, adjacency (claim 4) and the cursor shroud result (claims 5 and 6; with `quirks.placement_shroud_at_cursor` off, explored ground judged now) decide whether an order is issued. An upgrade uses claim 12's target condition instead of adjacency, retaining the shroud condition; on failure the placement refusal announcement is requested and no order is issued. Placement mode is offered only while the item is done (S-0411) |
| Item done (an AI house) | place order issued | Ordinary placement judges adjacency and explored ground at issue (claim 6). An upgrade uses claim 12's target condition instead of adjacency, retaining explored ground |
| Place order issued | order applied (order application of S-0213) | The order is applied only when the item is done and a factory building could make the type; without such a factory building it does nothing and no sound plays (claim 1). When it is applied, the cell test (claims 3 and 8), or for an upgrade type the upgrade rule (claim 12), decides the outcome; claims 4 and 5 are not judged again. If that test fails, nothing is placed and, for a local player's house, the placement refusal announcement is requested, without the game-wide refusal sound |
| Placed or restored under construction, *construction* queued, no current behaviour | building turn lands on the last build-up stage, or any turn of a zero-delay range | Ready mark set; *construction* becomes current at the second per-tick start of that tick; first construction processing on the next tick, restarting at stage 0 (claims 14 and 17) |
| Deployed under construction, ready mark set at entry | the building's first turn | *construction* becomes current at its second per-tick start; first processing on the next tick (claim 17) |
| Under construction, positive-delay range of 1 stage or fewer | any turn | No ready mark from the range; construction never completes (claim 18) |
| *construction*, after its first processing | last build-up stage reached, or zero-delay completion report | Construction completes, idle body requested and guard becomes current in this tick (claim 15) |
| Newly completed, guard current | next tick's processing | Guard follows S-0215 claim 32, with the target kept across construction completion |
| Order applied, every test passed | building put on the map | The item’s line ends in that tick. After placement, each cell has the contents specified by claims 9–11: applicable walls are removed, a wall-type placement leaves its wall overlay, and any applicable wall fill is present. When removal under a damaged own wall succeeds, that cell holds an undamaged wall. |

Observable consequences:
- A building is in play from the tick its place order is applied; the adjacency and prerequisites
  it supports take effect from then on, while it is still in its construction state (S-0407, claim
  8, for prerequisites).
- Two otherwise valid ordinary place orders applied in the same tick for overlapping cells:
  the first applied succeeds and the second fails the cell test, since the first building
  already occupies the cells. The order of
  application is S-0213's. Confidence medium. Backing: `unmeasured`: scripted input cannot issue
  two place orders in one tick (a house has one item done per line and placement mode takes one
  click at a time), and the outcome follows from claim 1 (the building is in play at once) and
  claim 3 (b) (a building in a cell refuses), both testable by scenarios PL1 and PL3.
  This consequence is limited to ordinary types subject to the all-cells, no-building condition:
  place-anywhere types, laser-fence replacements, to-tile types and upgrades follow their own
  rules and are not covered by the two-order refusal statement.

## Edge cases and quirks

- **Wall exception skips the ground test.** A cell meeting a wall exception (claim 8) passes without
  the ground conditions of claim 3 (d), so a wall tower or gate stands on its own wall even on a
  slope or unbuildable land. `quirks.wall_exception_skips_ground` (default on). Off: claim 3 (d) is
  still asked of such cells, as (d) applies to the type being placed. With the switch off, a wall
  cell that fails (d) for that type refuses the repair, tower or gate: for a type that is neither
  naval nor water-bound, a cell marked as part of a bridge, a sloped cell or a cell whose land type
  has the buildable flag off; for a naval type that is not water-bound, a bridge-marked or sloped
  cell or a tile outside the naval range; for a water-bound type, a land type whose float
  multiplier is 0. So the result depends on the rules data's land type for walls (S-0211 claim 5 reads the cell's land
  type, which for a wall cell is normally the land type *wall*; whether that land type is buildable
  is data). Check the default
  data before using the off state.
- **Shroud condition from the cursor.** For human input, the click uses the cursor shroud result
  (claim 6), so a foundation cell that is revealed or shrouded again while the cursor stays on
  the same cell does not change the click's outcome until the cursor changes cell.
  `quirks.placement_shroud_at_cursor` (default on). Off: the shroud condition is judged when the
  order is issued, as for an AI house. The on default follows the glossary's quirk-switch
  convention: reproduce the original by default.
- **Own ally-eligible buildings with the option on** support adjacency without the base-normal flag
  (claim 4). This follows from the house counting as its own ally; no switch.
- **Water-bound types skip the bridge, slope, buildable-flag and tile-range conditions** (claim 3 d,
  scenario PL4, parts c to f). No switch: it is the rule of the water-bound flag, which tests the
  float multiplier instead.
- **A later layer's class overrides an earlier explicit water-bound value** (claim 21, PL17 parts
  d and e). No switch: the flag follows the last layer that gives either field.
- **A placed building's build-up runs twice** (claim 17). The place order leaves *construction*
  queued until the build-up's first run reports its last stage, and the first construction
  processing then restarts the build-up at stage 0, so the whole build-up of a placed building
  with at least 2 stages and a positive delay takes 2(*n* − 1) × *d_b* + 1 ticks from the order
  tick (F7). A deployed building runs it once. `quirks.placed_buildup_twice` (default on). Off: a
  place order sets the ready mark at entry, as a deploy does. *construction* then becomes current
  at the first turn, *T* + 1, *C* = *T* + 2, and the build-up is seen once, ending at
  *T* + 2 + (*n* − 1) × *d_b* (F7). The same switch covers restored replacements
  (S-0524 claim 15): on leaves their mark clear at entry, including absent build-up art.
  Off sets it at entry and uses the deployed-style start. For restoration, use the actual first
  eligible building turn *U*, not *T* + 1: with the switch off, *C* = *U* + 1 and handover
  follows F6. With it on, claim 17's range report determines the start. Every other rule of
  claims 14, 15 and 18 is unchanged.
- **Single-stage build-up never completes** (claim 18). A positive-delay construction range of
  one stage or fewer never reports completion, so the building stays under construction for good.
  `quirks.single_stage_buildup_never_completes` (default on). Off: such a range reports
  completion on the building turn on which its stage first runs past the end and wraps, that is
  *d_b* ticks after the range starts. A placed building then reaches *construction* at
  *T* + *d_b* and hands over *d_b* ticks after *C*, and a deployed one hands over at
  *C* + *d_b*. Ranges with at least 2 stages are unchanged. The importer warns about such types
  in both states (Import mapping). The same switch governs selling (S-0408 claim 17):
  with it on, a positive-delay build-down of one stage or fewer never completes, so the
  building remains and no sale refund is paid, even though sale releases have already
  occurred. With it off, completion is reported on the first wrap, giving an ordinary
  sale's payment and removal at *D* + *d_b*, where *D* is the build-down-start tick.
  This is exact integer arithmetic in ticks, assuming uninterrupted building turns and
  selling processing (S-0408 F4). Zero-delay ranges are unchanged; S-0408 S10 measures
  the selling effect. Construction timings above are unchanged.
- **Upgrade beyond the third position** (claim 12, F5). A target type whose upgrade count is above
  3 lets level-less upgrades reach *c*′ above 3. The original records them in no position and
  alters some other part of the building's state, with effects not established; our engine records
  nothing. Departure (bad data), no
  switch; the importer warns about such target types (Import mapping, upgrade count).
- **Unknown or absent foundation data.** S-0108 owns the foundation word; a word it does not know
  gives 1 × 1.
- **Negative adjacency** (F1) makes placement impossible for that type. The importer warns. No
  switch: it is what the ring of F1 gives for such a value.
- **Level-field upgrade on a type without upgrade room** (claim 12). A level field from 1 to 3 asks
  only that the building holds no upgrade yet, not that its type's upgrade count is above 0. On a
  building whose type has upgrade count 0 the place order therefore succeeds: the upgrade is
  consumed and the building is restored to full health. Unless the building is of the wall tower
  type, its upgrade count stays 0 and no upgrade is recorded, so the upgrade changes neither power,
  nor prerequisites, nor the room for later upgrades (scenario PL10 part c); the original's other
  alteration of the building's state is claim 12's departure, and our engine reproduces no other
  effect.
  On a building of the wall tower type the count is not capped (F5): the upgrade is recorded in
  position *n* and counts like any recorded upgrade (scenario PL10 part i).
  `quirks.upgrade_level_without_room` (default on). Off: the place order refuses when the target
  type's upgrade count is 0 and the target is not of the wall tower type, the one case in which
  the upgrade would record nothing; the wall tower case and every other case of claim 12
  (for example a level-3 upgrade on an empty building whose type's upgrade count is 2, recorded at
  the capped position 2) are unchanged. The importer warns about such pairs. Backing: `measurable`
  for the default (scenario PL10 parts c and i); the off state exists on our engine only.

## Import mapping

| Key (file, section) | Concept | Form | Absent |
|---|---|---|---|
| Placement display image `PLACE.SHP` | candidate-cell frames 0, 2 and 3 (claim 19) | sprite image, resolved by S-0005 | missing-image behaviour is not established |
| Cursor image and palette | ordinary arrow during placement (claim 19) | owned by S-0214 Import mapping | S-0214 |
| None | no transient placement-refusal text (claim 20) | no string-table entry or duration is imported for this event | not applicable |
| `Adjacent` (rules, building type) | adjacency *A* (claim 4) | integer, cells | 3 |
| `BaseNormal` (rules, building type) | base-normal flag (claim 4) | boolean | yes |
| `EligibileForAllyBuilding` (rules, building type; the original's spelling) | ally-eligible flag (claim 4) | boolean | no |
| `BuildOffAlly` (rules, `MultiplayerDialogSettings`; S-0412 owns it) | default of the build-off-ally option (claim 4) | boolean | yes |
| `PlaceAnywhere` (rules, building type) | place-anywhere flag (claim 3) | boolean | no |
| `Foundation` (art; S-0108) | foundation word and size (claim 2); the word `3x3Refinery` is the 8-cell 3 × 3 word | word (S-0108) | `1x1` |
| `WaterBound` (rules, building type) | water-bound flag, tested with the class *float* (claim 3 d), decided per layer (claim 21) | boolean (S-0003, claim 2); an unrecognised value counts as not given | in a layer that omits it: set from that layer's `SpeedType` when the layer gives one, otherwise kept (claim 21); no when no layer gives either key |
| `SpeedType` (rules, building type) | the water-bound flag in a layer that omits `WaterBound` (claim 21): on for `Float`, off for any other text | name, matched as in S-0211's `SpeedType` row (case ignored) | no effect |
| `Naval` (rules, building type) | naval flag (claim 3 d) | boolean | no |
| `Wall` (rules, building type) | wall type (claim 10); ignored, with an importer warning, when the type has no `ToOverlay` | boolean | no |
| `ToOverlay` (rules, building type) | wall overlay of the type (claims 8 to 11) | overlay type name | none |
| `Wall` (rules, overlay type; S-0210 owns it) | wall flag of an overlay type (claim 11) | boolean | no |
| `Gate` (rules, building type) | gate-flag type (claims 3 b and 9) | boolean | no |
| `LaserFencePost`, `LaserFence` (rules, building type) | laser-fence-post flag, laser-fence flag (claims 3 b and 9) | boolean | no |
| `WallTower` (rules, `General`) | wall tower type (claims 8 and 9); its used upgrade count is uncapped but still has only three valid upgrade positions and the position-1-only gate (claim 12) | building type name | none |
| `GDIGateOne`, `GDIGateTwo` (rules, `General`) | first pair of gate types (claim 8) | building type name | none |
| `NodGateOne`, `NodGateTwo` (rules, `General`) | second pair of gate types (claim 8) | building type name | none |
| `GuardRange` (rules, building type) | wall fill length *G* (claim 11) | cells in decimal form, converted to leptons as S-0215 (Import mapping) states | 0 (no fill) |
| `FirestormWall` (rules, building type) | firestorm-wall type (claim 11, left open) | boolean | no |
| `PowersUpBuilding` (rules, building type) | upgrade type and its target type (claim 12) | building type name | none |
| `PowersUpToLevel` (rules, building type) | upgrade level field and resulting recorded position (claim 12, F5); skipped positions stay empty and do not move the attack gate to a higher position | integer | −1 |
| `Upgrades` (rules, building type) | upgrade capacity of the target type (claim 12); the current used count also bounds weapon-provider eligibility, independently of occupied positions. A capacity above 3 on a type that some upgrade type names gets an importer warning; our engine never resolves weapons beyond the three valid positions (claim 12, departure) | integer | 0 |
| `ToTile` (rules, building type; read once the theater's tiles are loaded) | to-tile field (claims 3 and 5) | tile name | none |
| `Morphable` (theater data, tileset; S-0010 owns it) | the tileset test of the to-tile rule (claim 3) | boolean | no |
| `Buildable` (rules, land type section; S-0211 owns it) | buildable flag *B* (claim 3 d) | boolean | no |
| The water role (theater data; S-0010 owns it) | start of the naval placement range (claim 3 d) | as in S-0010 | unset: tiles 0 to 12 |
| `BuildupTime` (rules, `General`; S-0108 owns it) | build-up stage delay used by claim 15 | decimal, game minutes, converted and rounded by S-0108 F1 | S-0108 Import mapping |
| `Gate` (rules, building type), `GateStages` (art, building's art section; S-0108 owns the stage count) | gate construction range used by claim 15 | boolean and integer stage count | no and 9 stages respectively |
| `Rate` (rules, `Construction`; S-0215 owns its parsing) | ignored by the construction processing delay of claim 14 | decimal, game minutes | held value as in S-0215; no effect on this 1-tick delay |
| `DeploysInto` (rules, vehicle type; S-0415 owns it) | building type created by a successful deploy (claim 16) | building type name | none, not a deploying vehicle |
| `DeployFacing` (rules, building type; S-0415 owns it) | integer deploy facing in 1/256-turn steps, used for vehicle alignment, not initial building facing (claim 16); outside 0 … 255 prevents deployment and gets an importer warning | integer | 128 |
| `Turret` (rules, building type; S-0310 owns it) | building's facing is its turret's facing when this flag is set (claim 16) | boolean | no |

The build-up sprite resolves by S-0007's build-up file role. No build-up sprite gives
S-0108's default zero-delay construction range, not a placement-facing default. The importer warns about a building
type whose construction range, by S-0108 claim 5, has a positive delay and 1 stage or fewer: a
build-up sprite of 0 or 1 image frames, one of 2 or 3 image frames with a positive build-up
delay, or a gate whose gate stage count is 0 with a positive build-up delay (claim 18). No rules,
art or map key supplies a direction to a place order (claim 16).


## Measurement scenarios

Measurement setup (glossary): a tiny purpose-made skirmish map, one human house and one allied and
one enemy bystander computer player (glossary; the allied one is also the owner of the allied test
buildings of PL5, which also uses a second enemy one), named test rules files in the map's override sections, scripted input for exact
clicks, game speed index 2, 800 × 600 pixels, no engine add-on and no mods (the named test files of a scenario excepted),
a recorder noting each building's type, owner and top-left cell per tick (and, for PL1
run 4, its position in leptons, world height included). Placement
legality uses no random draw, so every run is deterministic given the same input; each scenario is
run **3 times** with identical input and must give the same result each time (a run is repeated,
and the repeat counts as one of the 3, when the void rule of *bystander computer player* voids it).
Each scenario names its observed area, the cells in which the void rule applies.

**PL1. Place order outcomes and the click cell.**
- Observed area: the cells of the clicked foundations and of the house's only factory building for
  buildings (and,
  in run 3, the two cells holding the limited test building).
- Setup: run 1, a done 3 × 3 test building on the building line (one item at a time, in its own
  run); run 1b, a done 2 × 2 test defence on the defence line; clicks on cells chosen so the named
  top-left cell is known by F2; one click on a blocked cell (a tree in a foundation cell). Run 2, the
  house's only factory building for buildings is sold while the building is done, with the click placed during
  the sale (after the sell order, before that building leaves the map); the credits and the line are
  read in the tick after it has left the map. Run 3, a test building type whose build limit is
  1 and a second building of that type given to the house by test trigger while the first is done.
  Run 4, a done 1 × 1 test building placed once on a flat cell at height level 0 and once on a flat
  cell at height level 2 (both buildable, next to the base), and a done 1 × 1 test type with the
  place-anywhere flag placed once on a sloped cell at height level ℓ whose surface height at the
  centre (S-0106, claim 2) differs from 104 × ℓ leptons, with the recorder reading the placed
  building's position in leptons, world height included, in the tick it first appears.
- Observable: the building's top-left cell; the tick it first appears in the recorder and its construction state in that tick (run 1); the placement refusal announcement request and playback separately; whether placement mode resumes; whether the item is still done; credits; in run 4 the building's position (*x*, *y*) in leptons and its world height.
- Expected: the building appears in the tick its order is applied, in its construction state; top-left cells by F2 (the 2 × 2 building's top-left is the clicked cell); the blocked
  click requests the placement refusal announcement without the game-wide refusal sound and
  resumes placement mode; in run 2 the click during the
  sale does nothing (no factory building could make the type: S-0404, Terms, a building being sold
  is excluded), the item stays done and no sound plays, and in the tick that factory building leaves the map the
  item is cancelled with the refund of S-0404 claim 8 and the line is empty; run 3 places the done
  building although the limit is reached; in run 4 the position is (256*x* + 128, 256*y* + 128)
  leptons for the top-left cell (*x*, *y*) (F4), with world height 0 on the level-0 cell and 208
  leptons (2 × 104) on the level-2 cell, and on the sloped cell the surface height of S-0106
  (claim 2) at the cell centre.
- Falsified if: in run 4 the position differs from F4, or the world height on the level-2 cell is
  not 208 leptons (for instance 0, which would put the building at world height 0 whatever the
  cell's height level), or the world height on the sloped cell is not S-0106's surface height at
  the centre (for instance 104 × ℓ, the flat height of its level); another cell is used; the building appears in a later tick or is not in its construction state when it appears; a failed placement ends placement mode, requests no placement refusal announcement or
  requests the game-wide refusal sound; in run 2 the click during the sale places the building or plays a sound, or the item is
  not cancelled and refunded when the factory building leaves the map; or run 3's placement is refused.

**PL2. Adjacency distance.**
- Observed area: the new building's cells, its ring of *A* + 1 cells and the supporting building.
- Setup: a test building type with adjacency *A* = 2 (test value); the house's factory building
  for buildings (claim 1) stands outside every ring that this scenario tests, and the house's only
  building within those rings is a base-normal 1 × 1 test building at a known cell. Place attempts with gaps of 2 and 3 empty cells
  along map *x*, and diagonally with gaps of 2 along both axes; a second run with *A* = 0.
  Parts: (a) the placements above; (b) the supporting building in its construction state (the
  next placement made while it is still being built), being sold (placement made during the sale,
  before it leaves the map) and damaged to a few health points, each with a gap of *A*; (c) a test
  type without the adjacency key, gaps of 3 and 4; (d) a supporting test building without the
  base-normal key, and, with the option on, a nearby allied building without the ally-eligible key,
  each with a gap of 0; (e) a test type with adjacency *A* = −1 (test value), placed touching the
  base-normal test building (gap 0).
- Observable: whether each placement succeeds.
- Expected: gaps up to *A* succeed (2, and diagonal 2 × 2), a gap of *A* + 1 refuses; with *A* = 0
  only touching placements (including diagonal touching) succeed. In (b) all three states support.
  In (c) gap 3 succeeds and gap 4 refuses; in (d) the building without the base-normal key supports, the
  allied building without the ally-eligible key does not. In (e) the placement refuses although it
  touches its support (F1: the ring is empty).
- Falsified if: a gap of *A* + 1 succeeds (ring of *A* + 2) or a gap of *A* refuses (ring of *A*), or
  in (b) a state does not support; in (c) another gap limit appears; in (d) the outcomes are
  reversed; in (e) the placement succeeds.

**PL3. Ore, units and objects in the way.**
- Observed area: the candidate foundation cells, each a 1 × 1 test building's cell next to the base
  (two cells for the 2 × 1 types of parts t and w).
- Setup: candidate foundation cells holding (a) one ore cell, (b) one gem cell, (c) an own infantry
  unit, (d) an own parked vehicle, (e) a tree, (f) a crate, (g) nothing, (h) a landed aircraft,
  (i) a bridge cell, (j) a sloped cell, (k) a cell whose land type has the buildable flag off
  (test rules); a test type with the place-anywhere flag on cells (a), (e), (i) and (j); a test gate
  type, and a test type with the laser-fence-post flag, on a cell holding (l) an own building with
  the laser-fence flag, (m) an enemy one, (n) an own building without it; (o) a legal cell outside
  the local area, and a cell at the local area's border whose height level makes the plain and the
  height-aware tests of S-0009 (scenario M3) differ; (p) a test aircraft of the house in ordinary
  flight over the otherwise empty candidate cell at both the click and the application of the
  place order, then, in a separate run as the control, the same aircraft landed on the cell as in (h); (q) after the post-type placement of (l)
  succeeds, the recorder's list of buildings in the cell; (r) a cell holding an overlay type that
  is not ore, gems, a wall or a bridge (test rules name one), (s) the same cell with the
  place-anywhere type. (t) Named test theater data as in S-0010 scenario G1: one tileset with the
  morphable flag and one without, every tested cell flat and of a land type
  with the buildable flag on; a 1 × 1 test type with a to-tile field placed on (*t*1) a
  morphable cell holding an own infantry unit, (*t*2) a cell of the tileset without the flag,
  (*t*3) a morphable cell holding ore, (*t*4) a morphable cell holding an own building; and a 2 × 1
  test type with a to-tile field placed (*t*5) with one cell morphable and the other not. (u) A
  test type with the laser-fence flag placed on (*u*1) a cell holding an own infantry unit, (*u*2)
  a tree, (*u*3) an ore cell and, separately, a gem cell, (*u*4) a sloped ore cell, (*u*5) a cell holding an own building,
  (*u*6) a flat cell without overlay whose land type has the buildable flag off (test rules), and
  (*u*7) a flat ore cell whose land type has the buildable flag off; the cells of (*u*1) to
  (*u*5) are of a land type with the buildable flag on. For the rest of part u, the test rules
  fix the overlay registry (S-0004 registration order) so that registration index 126 is a test
  overlay type that is not ore, gems, a wall or a bridge, and part r's overlay type has another
  index: the laser-fence type placed on (*u*8) a flat cell holding the index-126 overlay whose land
  type has the buildable flag off, (*u*9) a flat cell holding part r's overlay type, (*u*10) a flat
  ore cell that a high bridge marks without a bridge overlay in it (the cell two steps
  beyond the bridge overlay along its marking direction, S-0211 claim 15, as in PL4; ore is put
  there by the test map's overlay data); and (*u*11) a test type with the laser-fence and
  water-bound flags and (*u*12) a test type with the laser-fence and naval flags, each placed on a
  flat ore cell whose land type has a float multiplier of 0 (test rules) and whose tile lies
  outside the water role range. If the test map cannot hold ore in the cell of (*u*10) (the ore
  is gone, or the cell is no longer marked, when the game starts), (*u*10) is not run. (v) In a separate game with part t's test
  theater data, test rules that name as the wall tower type (*v*1) a 1 × 1 test type with a to-tile field, placed on a morphable
  cell holding an own wall of an overlay type with registration index 0 or 2, and, in another run,
  (*v*2) a 1 × 1 test type with the laser-fence flag, placed on such an own wall on a flat
  cell; each also placed, as the control, on an enemy wall of the same overlay type. (w) With part
  t's test theater data: a 1 × 1 test type with a to-tile field and the laser-fence flag placed on
  (*w*1) an empty cell of the tileset without the morphable flag and (*w*2) a morphable cell
  holding a tree; a 1 × 1 test type with a to-tile field and the gate flag, and one with a to-tile
  field and the laser-fence-post flag, each placed on (*w*3) an empty cell of the tileset without
  the morphable flag and (*w*4) a morphable cell holding an own infantry unit; and a 2 × 1 test type
  with a to-tile field and the laser-fence flag placed (*w*5) with a tree in one cell and the other
  cell empty and morphable. Every cell of part w is flat, holds no overlay and is of a land type
  with the buildable flag on. (x) With part t's test theater data: a 1 × 1 test type with both the
  place-anywhere flag and a to-tile field placed on a cell of the tileset without the morphable
  flag holding ore, a cell on which the to-tile rule refuses twice over (tileset and overlay).
- Observable: placement success (for parts t to x, whether the place order succeeds: the item is
  spent and no placement refusal announcement is requested).
- Expected: (a) to (f), (h) to (k) and (r) refuse; (g) succeeds; (o) both refuse when S-0009's
  height-aware test says the cell is not usable, and succeed otherwise; (p) placement succeeds
  during ordinary flight but fails when the aircraft has landed (S-0209 claim 13); (q) is
  recorded whatever it shows (the fence building present or gone), as claim 3 (b)
  leaves it open; (s) succeeds; the place-anywhere type succeeds on all four of its cells; the gate type and
  the post type succeed on (l) and refuse on (m) and (n); (*t*1) and (*t*5) succeed, (*t*2),
  (*t*3) and (*t*4) refuse; (*u*1), (*u*3) (on the ore and on the gem cell), (*u*7), (*u*8), (*u*11) and (*u*12) succeed, (*u*2),
  (*u*4), (*u*5), (*u*6), (*u*9) and (*u*10) refuse; (*v*1) and (*v*2) succeed on the own wall and
  refuse on the enemy wall; (*w*1), (*w*3) and (*w*5) succeed, (*w*2) and (*w*4) refuse (with the
  gate type and with the post type alike), each the opposite of what a plain to-tile type gives
  in parts *t*1 and *t*2; (x) succeeds.
- Falsified if: any of (a) to (f), (h) to (k) and (r) succeeds, (g) or (s) refuses, (o) follows the
  plain test where the two tests differ, the landed aircraft in (p) lets the placement succeed, the
  flying aircraft in (p) blocks placement despite every other placement condition being satisfied, the place-anywhere type refuses anywhere, or the gate or post type
  succeeds on (m) or (n) or refuses on (l); (*t*1) refuses (units would then count for a to-tile
  type), (*t*5) refuses (every cell would then be tested), or (*t*2), (*t*3) or (*t*4) succeeds,
  and in part u (*u*1), (*u*3) on either cell, (*u*7), (*u*8), (*u*11) or (*u*12) refuses (the buildable flag,
  the naval range or the float multiplier would then be asked, or index 126 not excepted), or
  (*u*2), (*u*4), (*u*5), (*u*6), (*u*9) or (*u*10) succeeds (another overlay type, or a
  bridge-marked cell, would then pass); in part v a type refuses on the own wall (the wall
  exceptions would then not apply to it) or succeeds on the enemy wall; in part w (*w*1) or (*w*3)
  refuses (the tileset test would then still be asked), (*w*2) or (*w*4) succeeds (the to-tile
  object rule would then apply instead of the type's own), or (*w*5) refuses (every cell would
  then be tested); (x) refuses (the to-tile rule would then take precedence).

**PL4. Water-bound and naval types.**
- Observed area: the cells named in the setup, one 1 × 1 test building per placement.
- Setup: a test water-bound type, a test naval type that is not water-bound, and a test type that
  is both water-bound and naval; named test cells: water cells, a shore cell whose land type has a
  float multiplier of zero (test rules), a water cell whose tile is outside the water range, a
  sloped land cell and a flat land cell whose land type has the buildable flag off (test rules give
  both land types a float multiplier that is not zero), and a cell that a high bridge marks without
  any overlay in it (the cell two steps beyond the bridge overlay along its marking direction,
  S-0211 claim 15), whose land type is water. Parts: (a) the water-bound type on a water cell, on
  the shore cell and on the out-of-range water cell; (b) the naval type on an in-range water cell
  and on the out-of-range water cell; (c) the water-bound type on the flat land cell with the
  buildable flag off; (d) the water-bound type on the sloped land cell; (e) the water-bound type on
  the high-bridge-marked cell, and, as the control, a type that is neither water-bound nor naval
  on the same cell; (f) the type that is both water-bound and naval on the out-of-range water cell.
- Observable: placement success per cell.
- Expected: (a) succeeds on the water cell and on the out-of-range cell, refuses on the shore
  cell; (b) succeeds in range, refuses out of range (the tile range of claim 3 d); (c), (d)
  and (e) succeed for the water-bound type, and the control in (e) refuses (S-0211 claim 15 and
  scenario L21 there); (f) succeeds (the float multiplier decides, not the tile range).
- Falsified if: the water-bound type succeeds on the shore cell in (a); it follows the tile range
  or the buildable flag in (a) or (c),
  refuses on the sloped cell in (d), or refuses on the high-bridge-marked cell in (e); the naval
  type does not follow the tile range in (b) (for instance it follows the float multiplier); or
  (f) refuses out of range.

**PL5. Building off allies.**
- Observed area: the ring cells of the new building and the one building in range.
- Setup: the only building in range is (a) an allied ally-eligible building, (b) an allied
  base-normal building that is not ally-eligible, (c) an enemy ally-eligible building, (d) an own
  ally-eligible building that is not base-normal, (e) an ally-eligible building of a second enemy
  bystander computer player to whom the human house has given an alliance order (S-0414, claim 12,
  test rules that allow the order, as S-0414 scenario V9 uses; the second enemy keeps the order
  from being the last peace that claim refuses), so that the human house allies the owner and the
  owner does not ally the human house. Runs with the lobby check box ticked and unticked; a run
  with a loose test copy of the base rules without the default key, the program started after
  placing it, and the check box left as the lobby offers it (as S-0412 scenario K1 does); a run
  with a map override section that names the default key with value off, the check box left as the
  lobby offers it.
- Observable: placement success; the state the lobby offers the check box in.
- Expected: option on: (a) and (d) succeed, (b), (c) and (e) refuse; option off: all five refuse; with
  the key absent the lobby offers the box ticked, and the placements behave as with it ticked; with
  the map override the box is still offered ticked (S-0412, claim 1).
- Falsified if: (b), (c) or (e) supports placement (in (e) the relation is then read from the placing
  house towards the owner instead), (a) or (d) refuses with the option on, any placement
  succeeds with the option off, the absent-key default offered is off, or the map override changes
  the setting.

**PL6. Shroud, and when it is judged.**
- Observed area: the foundation cells of the test building and the cells that the test trigger
  reveals or shrouds again.
- Setup: a map with shroud; a test building next to the base whose foundation has one cell shrouded
  for the house. Parts: (a) cursor moved onto the foundation, click; then the cell revealed by test
  trigger, the cursor moved one cell away and back, click; (b) the cursor placed on the foundation
  with the cell shrouded and left still, the cell revealed by test trigger, then click without
  moving; (c) a foundation fully explored, the cursor placed and left still, one foundation cell
  shrouded again by test trigger, then click without moving; (d) a test type with a to-tile field and a foundation cell shrouded for the house, compared with the
  same placement when every cell is explored, the cursor moved onto the foundation each time, and
  the same pair of placements with a test type that has both a to-tile field and the place-anywhere
  flag (claim 3: its to-tile field still exempts it). (e) A test type with the 8-cell 3 × 3
  foundation word: only its missing cell (2, 1) is shrouded, all covered cells explored, with
  every other placement condition satisfied. Move the cursor onto the foundation before clicking.
  The original has no off state of the switch of claim 6 to run; a replay on our engine runs both states.
- Observable: placement refusal announcement request and playback separately, whether an order
  is issued and the building placed.
- Expected: (a) the first click refuses, the second places; (b) the click refuses (the old result); (c) the click places (the old result); after moving the cursor one cell away and back, (b) places
  and (c) refuses. On our engine with the switch off, (b) places and (c) refuses at once.
  (d) for each of the two types, the outcome with the shrouded cell is the same as with every cell
  explored (the shroud is not what decides it); the type with both flags succeeds both times.
  (e) refuses: the shroud test includes the missing cell of the foundation rectangle.
- Falsified if: in (a) the first click places or the second refuses; (b) places, or (c) refuses,
  which would put the judgement at the click; after the cursor moves one cell away and back, (b)
  refuses or (c) places; or (d) the two outcomes of a type differ, or the type with both flags
  refuses; or (e) places despite the rectangle's shrouded missing cell.

**PL7. Conditions lost between click and application.**
- Observed area: the foundation and the supporting building's cells; for part b, the foundation
  cells.
- Setup: (a) the only supporting building is killed by test trigger in the same tick as the
  scripted click, ordered so the click is taken before the kill; (b) the foundation fully explored
  and the cursor left still, one foundation cell shrouded again by test trigger in the same tick as
  the click, ordered after the click and before the order is applied (S-0213, claim 1: orders
  issued in a tick are applied at its end). If scripted input and test triggers cannot place
  the trigger between the click and the end of the tick, part b is not run, and the shroud half of claim 6's application
  rule stays unmeasured.
- Observable: whether the building is placed.
- Expected: (a) and (b) placed (adjacency and shroud were judged at the click and are not judged
  again when the order is applied).
- Falsified if: the placement fails for adjacency in (a), or for the cell shrouded again in (b).

**PL8. Walls, towers, gates and fill.**
- Observed area: the cells along the four rays from the placed wall, up to 5 cells away, and the
  cells named in each part.
- Setup: named test rules fix the overlay registry. Wall type *W* has as its wall overlay the
  overlay type *O* at registration index 0 or 2, with a guard range of 4 cells (*L* = 4) and
  *W* is the first building type in the registry that names *O* (so it is *O*'s builder). Wall type
  *W*26 has as its wall overlay the overlay type at registration index 26 (guard range 4 cells).
  *O* and the index-26 overlay type both have the wall flag of an overlay type (set by the test
  rules, since its absent-key default is no).
  The test rules name a building type *G*1 as one of the first pair of gate types and a building
  type *G*2, which has the gate flag, as one of the second pair. Own walls are made by placing *W* and *W*26. Credits, the
  sell sound and the random-state recorder's draw count are read throughout. Parts:
  - (a) place *W* 3 cells from an own wall in line (gap 2) and, separately, 5 cells away (gap 4).
  - (b) damage an own wall of *O*, then place *W* on it.
  - (c) place the wall tower type on an own wall of *O*. Repeat on a sloped cell and on a
    flat cell with test rules making its wall land type unbuildable. The tower is neither naval
    nor water-bound; all conditions other than the ground condition are satisfied.
  - (d) place the wall tower type on an enemy wall.
  - (e) *G*1, which has the gate flag, with a 2 × 1 foundation, across two own walls of *O*.
  - (f) as (b) with the unsellable flag on *W* (the builder of *O*).
  - (g) place *W* 4 cells from an own wall (gap 3).
  - (h) place *W* 3 cells from an own wall with a tree in the cell between them, and, separately,
    3 cells from an own wall with an enemy wall in the cell next to the placed cell.
  - (i) own walls 3 cells away in all four directions at once, compared with a placement that
    fills nothing. Use the read-only random-state recorder to count draws in each placement tick
    separately, excluding unrelated draws. Record each run's seed and prior stream state.
    Matching seeds across runs are not required and are not forced.
  - (j) *G*2 with a 2 × 1 foundation across two own walls of the index-26 overlay (*j*1); *G*2 on
    an own wall of *O* (*j*2); *G*1 on an own wall of the index-26 overlay (*j*3); *W*26 on an
    own damaged wall of the index-26 overlay (*j*4) and on an own undamaged one (*j*5).
  - (k) a second building type *W*b registered after *W* that also names *O*: first the unsellable
    flag on *W*b only, with *W* placed on a damaged own wall of *O*; then the unsellable flag on
    *W* only, with *W*b placed on a damaged own wall of *O*.
  - (l) a type that is the wall tower type and also has the gate flag, with a 2 × 1 foundation,
    across two own walls of *O*.
  - (m) a further first-pair gate type with the gate flag, with *O* as its wall overlay (and no
    wall flag), with a 2 × 1 foundation, across two own walls of *O*.
  - (n) *G*1 on a cell that holds an own building with the laser-fence flag.
  - (o) a type like *W* with the place-anywhere flag, with an own wall 3 cells away and a tree in
    the cell between them.
  - (p) own damaged wall of *O* 2 cells from the placed cell with an own undamaged one 4 cells away,
    in line (*p*1); own damaged wall 3 cells away and nothing behind it (*p*2).
  - (q) an own wall 4 cells east of the placed cell, and an own wall 3 cells north of the cell next
    to the placed cell on its east side, in the same column as that cell.
  - (r) a building type that is not a wall type, with *O* as its wall overlay, a guard range of 4
    cells and no wall flag, placed 3 cells from an own wall of *O* in line (gap 2).
  - (s) a wall type *W*n whose wall overlay is a test overlay type *O*n without the wall flag of
    an overlay type, at a registration index other than 0, 2, 26 and 126, with a guard range of 4
    cells; an own wall of *O*n is made by placing *W*n, then *W*n is placed 3 cells from it in
    line (gap 2).
- Observable: overlay per cell and its owner; credits; the overlay's damage state; for part r the
  recorder's buildings in the gap cells and their construction state; the sell sound,
  the draws taken in the placement tick, and in part n whether the laser-fence building is still in
  play afterwards.
- Expected: (a) gap 2 filled with 2 walls at no extra cost, gap 4 not filled (*L* − 1 = 3); (b)
  accepted, the wall becomes undamaged; (c) accepted, the wall removed without refund and without
  sound; (d) refused; (e) both walls removed without refund or sound; (f) accepted, no wall-type
  building remains, the item is spent, and no refund or sell sound occurs. The cell-content
  hypothesis is that the original keeps the same wall type, owner and damage state; record each
  before and after placement. Our engine must retain all three under claim 10's interim rule.
  A repaired, replaced or missing wall in the original falsifies the retention hypothesis and
  requires revising that interim rule before claiming equivalence. (g)
  filled with 3 walls; (h) no fill in either case; (i) all four directions filled (8 walls) and 8
  more draws than the placement that fills nothing, whatever order the recorder sees; (*j*1)
  accepted, both walls removed without refund; (*j*2) and (*j*3) refused; (*j*4) accepted and the
  wall becomes undamaged; (*j*5) refused; (k) with the flag on *W*b only the wall of *O* is
  removed and ends undamaged; with the flag on *W* only the removal does not happen (the builder
  is *W*, the first type that names *O*), and the cell holds what (f) records; (l) accepted, the
  wall in the top-left cell removed, the wall in the other cell still there; (m) accepted, the same
  result as (l); (n) accepted, the laser-fence building gone, no refund; (o) no fill; (*p*1) one
  wall added in the cell next to the placed cell, the damaged wall still damaged, the cell between
  the damaged wall and the undamaged one still empty; (*p*2) 2 walls added, the damaged wall still
  damaged; (q) 3 walls added in the east line and no wall in the two cells between the east
  neighbour and the northern wall, 3 more draws than a placement that fills nothing; (r)
  hypothesis: the two gap cells each hold a building of the type (no overlay of *O* is created), with 2 more
  draws than a placement that fills nothing; whether those buildings show a construction state is
  recorded; (s) the placed *W*n becomes an overlay of *O*n in its cell (claim 10), the two gap cells
  stay empty, and the draws taken equal those of a placement that fills nothing.
- Falsified if: in (r) the gap cells stay empty or hold walls of *O*; in (s) a gap cell receives a
  wall or a building, or more draws are taken than by a placement that fills nothing; gap 4 fills, gap 3 does not fill, a fill goes past a tree or an enemy wall, fill
  walls cost credits, (d) succeeds, a refund or a sell sound appears, or (i) shows a draw count other
  than the number of walls added; (a) fills gap 2 with other than 2 walls, (*p*2) adds other than 2
  walls, or (q) adds other than 3 walls in the east line or takes other than 3 more draws; (b),
  (c), (e) or (f) refuses, (f) leaves a wall-type building or an unspent item, (f) changes the
  wall's type, owner or damage state or removes it (falsifying the retention hypothesis), the wall
  in (b) stays damaged, or a wall in (c) or (e) is not removed; (*j*2), (*j*3) or (*j*5)
  succeeds, (*j*1) or (*j*4) refuses, (*j*1) leaves either underlying wall, or (*j*4) leaves
  the wall damaged; in
  (k) the removal follows the flag of *W*b instead of *W*, or the final cell contents differ
  from the corresponding (b) or (f) control; in (l) or (m) the wall in the other cell
  is removed too (the rules combine), or in (m) no wall is removed; in (n) the laser-fence building
  is still in play; in (o) the fill crosses the tree; in (*p*1) the damaged wall is repaired or
  replaced, or more than one wall is added; in (q) a wall appears between the east neighbour and the
  northern wall.

**PL9. The 8-cell 3 × 3 foundation and the zero-size word.**
- Observed area: the 3 × 3 block of cells from the intended top-left cell (parts a and b); the
  click cell and the ring (parts c and d).
- Setup: (a) a test type with the 8-cell 3 × 3 foundation word and a tree in the cell (2, 1) from
  the intended top-left cell; (b) the tree in (1, 1) instead; (c) a test type with the zero-size
  word and adjacency 3, clicked on a cell holding a tree, with the only supporting building in
  turn at 4 cells to the −*x* side of the clicked cell and at 4 cells to its +*x* side (same
  row); (d) the same type clicked on a cell that is shrouded for the house (the cursor moved onto it
  first, as in PL6), with the supporting building at 4 cells to the −*x* side.
- Observable: placement success; the top-left cell of a placed building.
- Expected: (a) succeeds, (b) refuses; (c) the placement ignores the tree (no cells to test), is
  named by the clicked cell itself (F2 with 1 × 1), and succeeds with the support at 4 cells to the
  −*x* side and refuses with it at 4 cells to the +*x* side (F1 with *W* = *H* = 0 reaches
  *d* = 4 cells to the −*x* side and 3 to the +*x* side); (d) succeeds (no foundation cell, so no
  shroud to judge).
- Falsified if: (a) refuses; (b) succeeds; (c) the tree refuses, the top-left cell is not the clicked cell, or both
  supports succeed or both refuse (a 1 × 1 ring); (d) refuses.

**PL10. Upgrades.**
- Observed area: the target building and its cells (for parts e and f, also the cells around it).
- Setup: (a) a target test building with upgrade count 2, damaged to half health before each
  placement, and a test upgrade type with no level field placed three times on it; (b) a target
  with upgrade count 3 and a level-1 test upgrade type placed twice; (c) a target type with upgrade
  count 0, damaged, and a level-1 upgrade type, which also gives a test power output and is a
  prerequisite of a test building; (d) a target with upgrade count 3 and no upgrade, a level-2 test
  upgrade type U2 with test power output P2, then a level-less upgrade type U3 with test power
  output P3 placed on it twice; a test building whose prerequisite is U2; (e) a lone target
  building with no other building of the house within the ring of the upgrade type's adjacency
  (test value 0), a level-less upgrade type, and, as the control, the same upgrade type clicked on
  an empty cell in the same position; (f) a target test type that reveals nothing around itself
  (test rules), its cell shrouded for the house by test trigger, the cursor moved onto the cell
  before each click: once with the cell shrouded and once with it explored (revealed by test
  trigger before the cursor moves); if test triggers cannot keep the cell shrouded, this part is not
  run and the shroud half of the claim stays unmeasured; (g) a target with upgrade count 2 and
  test upgrade types whose level fields are 0, 4 and −2, each clicked once on a target with no
  upgrade; (h) a target with upgrade count 4 (test value; kept at 4 so that no run reaches the larger
  counts that may make the original fail, claim 12), damaged before each placement, and a
  level-less test upgrade type with test power output P4 placed on it four times; (i) test rules
  that name a target test type with no upgrade-count key (count 0) as the wall tower type, the
  target damaged, and a level-1 test upgrade type with test power output P1 that is also the
  prerequisite of a test building, placed on it twice. (j) An eligible target at the click,
  destroyed by a test trigger after the click but before application, as PL7's event timing
  requires. Record the issue and application boundaries separately; if that timing cannot
  be established, do not run (j) and leave the timing subclaim unmeasured.
- Observable: placement success, the building's health after each placement, the house's power
  output (S-0405) and whether the test building is offered (S-0407); in (e) and (f) whether the
  upgrade is placed and the placement refusal announcement request and playback separately.
- Expected: (a) two successes then a refusal, the building at full health after each success; (b)
  the first succeeds, the second refuses; (c) the placement succeeds, the building is healed, and
  the power output does not change and the prerequisite is not met; (d) U2 succeeds, the output
  rises by P2 (once, not twice) and the test building is offered; U3 then succeeds and the output
  rises by P3; the second U3 refuses; (e) the upgrade is placed on the target, and the click on
  the empty cell refuses; (f) the click on the shrouded cell refuses, the click on the explored
  cell places the upgrade; (g) all three refuse; (h) all four succeed and each restores full
  health, the output rises by P4 with each of the first three and not with the fourth (*c*′ = 4 is
  not recorded, F5); (i) the first placement succeeds, restores full health, raises the output by
  P1 and makes the test building offered (the uncapped count 1 records the upgrade in position 1),
  and the second refuses (the count is no longer 0). (j) The order is issued but the upgrade
  refuses at application, retaining the done item and requesting the placement refusal
  announcement; it does not attach to a missing target.
- Falsified if: the counts differ, the health stays at half, (c) refuses, changes the output or
  meets the test prerequisite, or (d) raises the output by twice P2, leaves the test building
  unoffered, accepts the second U3 or fails to add P3 for the first U3; (e) the upgrade is
  refused for adjacency; (f) the click on the shrouded cell places the upgrade, or the explored one
  refuses; (g) any of the three places an upgrade; (h) the fourth placement refuses or raises the
  output; (j) the order succeeds after the eligible target is destroyed before application, or
  (i) the first placement leaves the output unchanged or the test building not offered
  (the count was capped, as in part c), or the second placement succeeds.

**PL11. Construction start, cadence and handover (Q-0130).**
- Setup: the common skirmish measurement setup, game speed index 2, no engine add-on and no mods (the named test files of a scenario excepted). Observed area: the building's
  foundation and any inherited target. Use an unarmed test building with a known multi-stage
  positive-delay build-up (*n* ≥ 4 and *d_b* ≥ 3, so that the two runs are easy to tell apart),
  plus a turreted copy with no enemy in sight. Test data give two substantially different
  behaviour rates. Parts: (a) ordinary build-up, placed; (b) a gate with a known gate range,
  placed; (c) a type whose build-up file does not resolve, placed; (d) an armed copy made by a
  vehicle deploy, inheriting a live target; (e) a test type whose build-up sprite has 2 image
  frames and a positive build-up delay, placed, and a deployable copy of it. Repeat (e) with
  named test build-up files of 0, 1 and 3 image frames, with a gate whose gate-stage field is 0,
  and with theater-flag copies whose theater-refreshed files have 0 or 1 image frames
  (S-0108 claim 5). Verify each resulting range's count and delay before recording its timing.
  (f) the type of (a) made by a vehicle deploy, and a theater-flag copy whose refreshed range
  has at least 2 stages and positive delay, both placed and deployed, to test that range under F7.
- Procedure: place or deploy with scripted input and note the tick *T* of the place order (or
  the deploy tick). Record every tick from *T* through the handover and the first following
  behaviour processing with the permitted read-only recorder; in (e) record for 20 × *d_b*
  ticks, then try to sell. Repeat 3 times per part and rate, with recorded seeds.
- Observable: current and queued behaviour, processing timer, body stage and animation state,
  construction-completed state, target, sell availability and scenario random-stream state, per
  tick. Record *R* (first tick on which the last build-up stage is shown), *C* (first
  construction processing, seen as the stage returning to 0 or the behaviour's first processing)
  and *H* separately from *T*, and the deployed building's first turn *U*. An unarmed copy
  separates timing draws from post-completion targeting or free-unit draws.
- Expected: no current behaviour and *construction* queued from *T* until *R*. For (a) and (b):
  *R* = *T* + (*n* − 1) × *d_b*, *construction* current at the end of tick *R*, stage 0 again on
  *C* = *R* + 1, consecutive construction processings with delay 1 whatever the imported
  behaviour rate and with no timing draw, and handover at *H* = *T* + 2(*n* − 1) × *d_b* + 1, with
  *guard* current and the idle body by the end of that turn, not one tick later (F6, F7). For
  (c): *C* = *T* + 2 and *H* = *T* + 3. For deployed (d) and (f): *construction* current at
  the end of the building's first turn *U*, *C* = *U* + 1 with stage 0,
  *H* = *C* + (*n* − 1) × *d_b*. The placed theater-refreshed copy in (f) follows (a)'s F7
  timings using its actual refreshed range, not the ordinary sprite-half count or delay.
  In (d) construction completion keeps the inherited target, and the next guard processing may
  start attack by S-0215 claim 32. For (e): the placed building keeps *construction* queued and
  shows stage 0 throughout, and the deployed one has *construction* current but never hands
  over. Neither is offered *sell* by S-0408's construction condition, and neither gets opening effects.
- Falsified by: in (a) or (b), a handover at *T* + (*n* − 1) × *d_b* or near it (one build-up
  run, as `quirks.placed_buildup_twice` off would give, *T* + 2 + (*n* − 1) × *d_b*), no return to
  stage 0 after *R*, or a *C* other than *R* + 1; any rate-dependent construction delay or timing
  draw; a handover tick differing from F6 or F7 in their stated domain; in (c) a no-sprite
  building never completing; in (d) a target cleared specifically by handover; in (e) any
  completion.

**PL12. Placement facing (Q-0133).**
- Setup: the common skirmish setup, speed index 2, no engine add-on and no mods (the named test files of a scenario excepted). Observed area:
  a legal foundation next to the base. Use a placed absorbing test building and a placed
  turreted test defence, with no enemy in sight. Give pre-placed factory buildings differing
  map facings in separate runs.
- Procedure: place each done item and record its facing in the very tick it enters play,
  before any later aiming. Complete the absorbing building, put infantry into it and sell it
  without a target or other facing-changing event. Repeat 3 times for each factory facing.
- Observable: building facing at insertion, turret facing at insertion, and released passenger
  facings at the sale, using the read-only recorder. The released passenger observation is
  a secondary check using S-0408 claim 8, not a substitute for the initial-facing record.
- Expected: initial placement facing and initial turret facing 0 facing units regardless of
  factory facing or clicked cell. Unchanged facing gives passengers facing 0 at release.
- Falsified by a different initial facing or a dependence on factory facing or click position.
  A later aiming turn alone does not falsify the initial-facing claim. Repeat the scripted placements and document the observed facings as described above.

**PL13. Deployed-building facing and invalid deploy directions (Q-0220).**
- Setup: the common skirmish measurement setup, game speed index 2, 800 × 600 pixels,
  no engine add-on and no mods (the named test files of a scenario excepted). Observed area: a clear foundation
  and the deploying vehicle's cell, with no enemy in range and no inherited target. A test
  vehicle deploys into an ordinary building, then in separate runs into a turreted copy.
  Map overrides set deploy facing to 64, 192, −1 and 256, each in a separate variant.
- Procedure: give the vehicle a deploy order with scripted input. For the valid values,
  start already aligned, then repeat starting opposite the required direction. For the invalid
  values, record at least 20 deploy processings, identified by the processing timer as in
  S-0415 D1. Repeat 3 valid runs per variant. A run losing deploy readiness or gaining a
  destination during the observation is void.
- Observable: vehicle body facing, building appearance tick, and building facing and turret
  facing at entry, using the permitted read-only recorder. For invalid values also record
  continued deploy attempts without building appearance. Do not infer an initial facing
  from a turret's later turn or a fixed body sprite.
- Expected: both valid values give building facing 0 facing units and turret facing 0,
  whether the vehicle was initially aligned or turned. Both invalid values produce no
  building despite clear ground and repeated ready attempts. As a control, placing the same
  building types through place orders gives facing 0 for all four field values.
- Falsified by: inheritance of the vehicle's facing, any other initial turret facing, a valid
  value changing the building's entry facing, or an invalid value producing a building.
  Later aiming alone is not a falsifier.

**PL14. Placement pointer and separate foundation display.**
- Setup: common skirmish setup, speed 2, 800 × 600, no engine add-on and no mods (the named test files of a scenario excepted). Observed area: an explored foundation beside the base, a blocked
  foundation, a sloped cell, a shrouded cell and a lone upgrade target. Use a done ordinary
  building, a place-anywhere copy and an upgrade with room on its target. A named test
  placement image with distinguishable frames 0, 2 and 3 identifies frame choice without
  relying on stock colours. Keep all other test files stock.
- Procedure: hover without clicking in each area, then move outside the battlefield and
  back. Record pointer-inclusive video. For the ordinary type, remove its sole adjacency
  supporter by test trigger while the cursor stays still, then click. Separately reveal a
  shrouded cell while the cursor stays still, then click. Test the upgrade on a lone eligible
  target, a full target, an enemy target and empty ground. Repeat each case three times.
- Observable: arrow frame and hot spot, foundation anchor, placement-image frames per
  marked cell, absence of a building sprite preview, and click/order outcome.
- Expected: the ordinary arrow in every battlefield case and a separate foundation display
  anchored by F2. Explored, clear, adjacent ordinary cells show frame 0; blocked flat cells
  show frame 2 and blocked sloped cells frame 3. Losing support without cursor motion keeps
  the saved positive display condition but the click refuses for adjacency. Revealing a cell
  without cursor motion can make its display eligible while the click retains the old shroud
  refusal. The place-anywhere copy can display refusal yet place successfully. Only the own
  compatible upgrade target with room displays frame 0, even without adjacency. The active
  display disappears outside the battlefield.
- Falsified by a battlefield deploy/refusal/capture cursor replacing the arrow, a sprite
  preview, a differing anchor/frame, a live adjacency display refresh with no cursor-cell
  change, or an upgrade display requiring adjacency. Record cursor and cell results
  separately: a correct arrow alone does not confirm the foundation display.

**PL15. Placement's Q-0282 text/no-text answer.**
- Setup: common skirmish setup, speed 2, 800 × 600, no engine add-on and no mods (the named test files of a scenario excepted). Observed area: the candidate foundation, its only supporting building,
  the upgrade target and the battlefield message area. Use a done building and, in separate
  runs, a done upgrade. No independent messages may occur during the observation.
- Procedure: refuse clicks for missing adjacency and unexplored ground; apply a place order
  onto a cell blocked by a tree; refuse an upgrade on a full target; and ignore a placement
  while the last qualifying factory is being sold, as PL1 run 2. Record from before each
  click until 200 ticks afterward, opening and closing the pause/options menu midway.
  Run each case three times with pointer-inclusive video, audio and a read-only recorder of
  announcement requests. In a separate control, show a test trigger's message to verify that
  the capture includes the message area; do not overlap it with a refusal.
- Observable: announcement requests separately from audible playback, message-list entries
  and the retained item/placement mode. Record whether any game-wide refusal sound is
  requested.
- Expected: refusals request only the placement refusal announcement, retain the item and
  add no text entry. The factory-condition ignore requests no sound and adds no text.
  The control message is visible. Pausing creates no placement text or expiry behaviour.
- Falsified by a refusal-created text entry, a game-wide refusal sound request for placement,
  or any sound/text request on the factory-condition ignore. Suppressed announcement playback
  alone is not a falsifier of a request.

**PL16. Upgrade positions are not installation-history precedence (claim 12).**

- Setup: a tiny flat skirmish map, game speed index 2, stock data except named map overrides
  for a test armed host and upgrades. One human house and one isolated bystander computer
  player; no engine add-on and no mods (the named test files of a scenario excepted). Use ordinary non-SAM hosts, with no delayed firing, no
  occupants and stationary enemy targets. Give each upgrade distinguishable ordinary primary
  and secondary weapons, observable by projectile or damage. A passing upgrade has identical
  primary and secondary weapon types; a failing upgrade has an ordinary single-shot primary
  and no secondary. Give the host its own distinguishable weapons. All upgrades in the sale
  tests lack the turret flag except where explicitly stated. Three fresh runs per part:
  record ticks, count, recorded types by position, resolved weapon identity, firing attempts
  and successful shots separately using the read-only recorder and video. Any bystander
  interference voids the run.
- Procedure and expected results:
  - (a) Give a host capacity 3. Install passing level-2 upgrade A, then a level-less upgrade B
    with a distinguishable secondary and no primary. Position 1 stays empty, A occupies 2,
    B occupies 3. Weapon providers prefer A where it has a weapon; in a fresh variant where
    A lacks a secondary, B supplies it. Even when A is the failing type, it does not impose
    the attack gate: position 1 is empty. Sell once and install C: only B is sold and C
    replaces position 3; A retains precedence. Attempting to sell A directly while B remains
    is not an available ordinary sale. Then sell to remove A and record count 1 with empty
    positions; install a level-less D and observe position 2, not 1.
  - (b) Install A, B and turret-flag C contiguously on a fresh capacity-3 host, then sell C.
    Count becomes 0 while A and B remain recorded. Weapon lookup uses the host, not A or B.
    Install a failing level-less D: it replaces A in position 1 and its gate suppresses firing,
    even though B remains recorded in position 2. Install passing E next: E replaces B in
    position 2, rather than being appended after the retained types. Use a fresh passing-D
    control to distinguish E's secondary from B's when D has no secondary but a multi-shot
    primary. Resolved secondary becomes E's, while the gate still tests D.
  - (c) Name the host type as the wall tower in the test map, give it capacity 4 and install
    four level-less upgrades. The first three supply both requested slots. The fourth raises
    the count and repairs the host but is not a valid provider; positions 1 to 3 and their
    provider priority do not change. The gate still tests position 1. Repeat with a level-2
    first installation and empty position 1: the gate remains bypassed.
  - (d) Diagnostic only, in a disposable run: repeat the count-4 tower test with none of the
    three recorded types supplying the requested slot. Record host fallback, refusal, fault
    or any other result without assuming one. Our engine must use the ordinary fallback,
    with no fourth provider. An original result is evidence for further investigation,
    not confirmation of a stable excess-position rule.
- Falsifiers: compaction after sale, filling a lower hole rather than position *c*′, a weapon
  supplied by a retained position above the count, a gate imposed by position 2 with position 1
  empty, or changed precedence from the fourth installation in part c. Part d has no claimed
  deterministic original outcome to falsify; it investigates the departure's unsafe case.
- Tool: scripted normal input plus a read-only recorder. No game-state writes or harness
  changes are required.

**PL17. The water-bound flag across rules layers (claim 21).**
- Observed area: two named cells, one 1 × 1 test building per placement.
- Setup: a buildable 1 × 1 test building type with neither the naval flag nor the place-anywhere
  flag. Test rules give the water land type a float multiplier that is not zero and the buildable
  flag off, and the clear land type a float multiplier of zero and the buildable flag on. Named
  cells: a flat water cell and a flat clear cell, both explored and within adjacency range of a
  building of the placing house. Each part writes the type's two fields as listed, in the test rules
  file (the base layer) and in the test map's section for the type (the map layer); a field not
  listed is absent from that layer. Parts:
  - (a) base: class *float*.
  - (b) base: class *float*; map: class *wheel*.
  - (c) base: neither field; map: class *float*.
  - (d) base: water-bound *yes*; map: class *wheel*.
  - (e) base: water-bound *no*; map: class *float*.
  - (f) base: class *float* and water-bound *no*.
  - (g) base: class *float*; map: a class text that names no class.
  - (h) base: class *float*; map: a section for the type that gives one unrelated field only.
  - (i) base: class *float* written in lower case.
  - (j) base: class *float-on-beach*.
  - (k) base: water-bound *yes*; map: class *wheel* and a water-bound text that S-0003 claim 2 does
    not recognise.
  - (l) base: water-bound *yes*; map: class *wheel* and a water-bound entry with no value.
  - (m) base: class *float*; map: a class entry with no value and no other field.
- Observable: placement success on each named cell, one fresh game per part and cell.
- Expected: water-bound in parts a, c, e, h, i and m: the water cell succeeds and the clear cell
  refuses. Not water-bound in parts b, d, f, g, j, k and l: the water cell refuses and the clear
  cell succeeds.
- Falsified if: any part gives the other part group's pair of results, for instance (d) succeeds on
  the water cell (the earlier explicit value would then stand), or (g) succeeds on the water cell
  (text that names no class would then keep the earlier class), or (k) or (l) succeeds on the
  water cell (the unrecognised or empty value would then keep the earlier *yes*), or (m) refuses
  on the water cell (the empty class entry would then count as text that names no class).
- Tool: the bench with a test rules file and a test map; scripted placement orders.

## Open questions

1. The firestorm-wall fill rule (claim 11), the fate of an own laser-fence building under a placed
   laser-fence-post type (claim 3 b; scenario PL3 q) and whether the filling buildings of a type
   that is not a wall type show a construction state (claim 11, scenario PL8 r) are not
   described. No stock type is known to have the firestorm-wall or laser-fence flags.
2. Whether the occupancy-record part of claim 3 (b) can refuse a cell that has no object in it (for
   example a position still recorded by an infantry unit that has just left) was not established.
3. The cursor shroud result when placement mode begins, before the cursor first changes cell
   (claim 6): the original may hold the value of an earlier placement. No scenario is proposed
   until S-0411 fixes how placement mode starts.
4. What the original's cell holds after a failed removal under a damaged wall (claim 10,
   scenario PL8 f). The retention hypothesis is non-normative; pending resolution our engine
   keeps the existing wall, owner and damage state unchanged. A contrary PL8 f result requires
   revising that interim rule.
5. Between entry and the tick *construction* becomes current (claim 17), a building has no
   current behaviour and *construction* queued. Under S-0215's queueing rule (claim 7) an order
   or rule that queues another behaviour in that window would replace the queued *construction*,
   and the building might then never complete. Whether any order or rule can reach a building in
   that window was not established. Sale removal and refund timing (Q-0130 question 4) are
   S-0408's.
6. Placement display details for to-tile types, gate display extents, wall-fill preview
   and a missing placement display image are not specified by claim 19. The ordinary arrow
   relationship is established for placement mode independently of these display details.
7. A building type that reads no section in any rules layer. Claim 21 treats it as not
   water-bound; this is our engine's choice, a departure with no switch. The original's ground test
   for such a type is not established: the reading is that it tests the *foot* locomotion class's
   land multiplier in place of claim 3 (d)'s conditions (confidence low, `unmeasured`: whether any
   data can place such a type, or deploy into one, was not established).
8. Whether any building rule passes a building type's class to S-0212 claim 25's nearby search
   was not traced, and no such use is identified. This is the open part of Q-3087 question 5.

## Changes for other specs

- S-0308 claim 12 and Terms: replace installation-history ordering with claim 12 here's
  count-limited increasing-position provider order, skipping empty positions; count 0 ignores
  retained upgrades. Counts above 3 use the three-position bad-data departure. The attack gate
  is position-1-only, not the first nonempty provider.
- S-0215 claim 32: the upgrade gate requires nonzero count and occupied position 1; an empty
  position 1 bypasses it rather than testing a higher installed upgrade. Use claim 12 here
  for retained positions and excess counts. Extend B36 with PL16's level-skip control.
- S-0404 claim 10: its place-order row's no-recheck sentence refers only to adjacency and
  explored ground. Upgrade target ownership, compatibility and room are checked again at
  application under claim 12 here; make that distinction explicit at joint hand-off.
- S-0006 claim 5, *Keys that reset to another key's value*: add the water-bound flag of a building
  type. In a layer whose section for the type gives the locomotion-class field but not the
  water-bound field, the flag is set from that class, replacing the value an earlier layer gave,
  even an explicit one. S-0406 claim 21 owns the rule.
