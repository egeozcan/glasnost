---
id: S-0511
title: Bridges (destruction, collapse, falling and repair)
topic: structures
revision: 13
sources-class: [decomp, design]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-11-14
measurements: []
requests: [Q-0676, Q-0677, Q-0678, Q-0710, Q-0712, Q-0713, Q-0714, Q-0715, Q-0716, Q-0717, Q-0718, Q-0719, Q-3046]
supersedes: []
---

# S-0511: Bridges (destruction, collapse, falling and repair)

## Summary

Bridge damage, collapse, falling, hut demolition and engineer repair in skirmish; network games and
campaigns are out of scope. Intact-deck movement belongs to S-0207 claim 5 and S-0209 claims 1, 4,
5 and 15, initial bridge marking to S-0010 claim 25 and S-0211 claims 9 and 15, and drawing to
S-0112 claim 6 and S-0102 claim 15.

Vehicle-transport unloading restrictions belong to S-0507 claims 7 and 10: an unload order is
refused on a bridge or when the transport's own cell or any straight neighbour carries a bridge
deck; passenger exit searches pass over neighbouring cells that carry a deck. These restrictions
concern ordinary unloading, not passenger escape from a destroyed transport.

## Terms

- **Bridge family.** Each bridge piece belongs to one of two families.
  - The *bridge-role family*: tiles of the theater's bridge role (S-0010 claim 25), its
    high-bridge overlay pair and its low-piece palette.
  - The *wooden-bridge-role family*: tiles of the wooden-bridge role, its high-bridge overlay
    pair and its low-piece palette.

  The two families behave identically. Import mapping binds the symbolic piece identities below
  to the imported overlays; no transition depends on our engine's registration order.
- **Tile position *q*.** For a cell whose tile lies in a family's role, *q* = tile number − the
  role's start + 1 (so 1 to 16). The theater names ten positions (Import mapping), which this spec
  uses as follows. A position the theater leaves unset matches no tile.
  - *start ramps:* top-left 1 and 2 for bridges along x, top-right 1 and 2 for bridges along y,
  - *far ramps:* bottom-right 1 and 2 along x, bottom-left 1 and 2 along y,
  - *piers:* middle 1 (*m*₁) along x, middle 2 (*m*₂) along y.

  Sub-tiles are numbered as in S-0010.
- **Axis, start side, far side.** Each family's high overlays are called *Hₓ* and *Hᵧ*, for
  bridges running along cell x and cell y respectively. Its spans run from a start piece toward
  increasing x (east) or increasing y (south), as in S-0010 claim 25. The *start side* of a piece is
  the side toward decreasing x (or y), and the *far side* the side toward increasing x (or y).
  A *line across the bridge* is the set of cells at one position along the axis (one x for a
  bridge along x, one y for a bridge along y); a *line along the bridge* is the set of cells at one
  position across it.
- **Section.** One high-bridge overlay together with the four cells it marks (S-0211 claim 15). The
  overlay's own cell is the *section cell*. The cells along the bridge's line that hold sections
  form a row of sections. A section's *neighbours* are the cells one step from its section cell
  along the axis, toward the start side and toward the far side.
- **Section state.** The overlay data value also chooses its drawn frame (S-0112 claim 6).

  | State | Along x | Along y | Meaning |
  |---|---|---|---|
  | intact | 0 to 3 | 9 to 12 | whole; the value is only a look |
  | chipped on the start side | 4 | 14 | damaged where it meets its start-side neighbour |
  | chipped on the far side | 5 | 13 | damaged where it meets its far-side neighbour |
  | damaged | 6 | 15 | one more successful attempt removes it |
  | open on the start side | 7 | 17 | its start-side neighbour is gone |
  | open on the far side | 8 | 16 | its far-side neighbour is gone |
  | lost | no overlay | no overlay | the section has been removed |

- **Pier.** A tile at positions *m* + *k* of a family's role, *k* = 0 to 4, with *m* = *m*₁ for a
  bridge along x and *m*₂ along y. A pier stands between two spans: S-0010's table B makes such a
  tile the far end of one span and the start of the next.
  - *Pier state k:* 0 intact, 1 chipped on the start side, 2 chipped on the far side, 3 damaged,
    4 collapsed.
  - *Reference cell:* the pier's cell at sub-tile *r* = 4 (bridges along x) or *r* = 2 (along y).
    For a direct attempt at a pier cell of sub-tile *s*, whose placed tile is *w* cells wide, it is
    the cell at sub-tile *r* of that placed tile (column *t* mod *w* along x and row ⌊*t* / *w*⌋
    along y for sub-tile *t*, S-0010): the cell (*r* mod *w*) − (*s* mod *w*) cells from the
    attempted cell along x and ⌊*r* / *w*⌋ − ⌊*s* / *w*⌋ cells from it along y, positive toward
    increasing x or y (claim 6).
  - *Pier column:* the reference cell and its two neighbours across the bridge.

  A pier's state is its tile. For a state replacement by a direct attempt, its affected tile cells
  are the eight-neighbour-connected component containing the reference cell whose cells have
  exactly that cell's pre-change tile number. For one by a notice or a repair, they are the same
  kind of component around the cell the notice arrives at (claim 6) or the repair selects
  (claims 18 and 19) instead. Sub-tile numbers and placed-tile origins do not restrict this
  set. A different tile laid over part of a pier is not changed, and can disconnect cells from
  the affected component. Adjacent copies of the same tile can belong to one component.
  For direct attempts, a tile position in both non-collapsed pier ranges takes the x
  interpretation: test x states 0 to 3 before y states 0 to 3. An x match only at collapsed
  state 4 does not override a non-collapsed y match. Repair's start-piece selection instead
  requires the axis-specific sub-tile in claim 18, and span repair uses its selected axis.
  Claims 6 and 19 own the height effects.
- **Ramp.** A tile at a start-ramp or far-ramp position. Ramps are never removed. They only change
  look.
- **Destroy attempt.** One application of this spec's destruction rules at one cell. It *succeeds*
  when it removes a section, collapses a pier or destroys a low strip, and *fails* otherwise. A
  failed attempt can still change the bridge.
- **Blow-up of a cell.** The collapse effects of claim 9 applied to one cell.
- **Low strip, strip class.** A low bridge is three cells wide. One step along its line, the three
  cells across it form a *strip*, all holding pieces of the same class. Along x the strip is the
  three cells (x, y − 1), (x, y) and (x, y + 1); along y it is the three cells (x − 1, y), (x, y) and
  (x + 1, y). Each family and axis has these symbolic piece identities:

  | Class | Piece identities (ours) |
  |---|---|
  | intact | I₀, I₁, I₂, I₃ (ordinary looks), I₄, I₅ (edge looks) |
  | damaged | D₀, D₁, D₂ |
  | start-side end piece | S (whole), S′ (damaged) |
  | far-side end piece | E (whole), E′ (damaged) |
  | destroyed | X |

  These are roles, not type names or absolute registration positions. Subscripts distinguish
  looks that also affect neighbour re-evaluation (claim 14). The same labels apply independently
  to both axes and families. Pieces are placed at map load by S-0202 claim 10.
- **Hut cell.** The cell containing the hut's actual position: its top-left foundation cell
  (S-0406 claim 1), not its foundation centre or a cell chosen by the approaching engineer.
  The *engineer's entry cell* is the cell containing the engineer's own actual position at the
  entry event. It is not replaced with the hut cell. Claims 16, 18 and 21 use these distinct
  centres even with a multi-cell foundation.
- **Responsible section.** A cell remembers which section last marked it as another section's
  deck cell. A cell marked as a section cell instead selects itself. This is historical
  marking, not a search for nearby overlays (claim 2).
- **Falling, fall rate.** A falling object moves down by its *fall rate* (leptons per update)
  until it reaches the surface. The *bomb mark* is S-0312's term.
- **Hut demolition, repair.** The destruction of a whole bridge from its bridge repair hut (claim
  16), and the rebuilding of a destroyed bridge when an engineer enters the hut (claims 17 to 20).

## Behaviour

### Destroy attempts

1. **Where destroy attempts come from.**
   - *Area damage.* S-0303 claim 18 says when a detonation makes attempts, with its gate, height
     window and chance draw. This spec gives the tests that decide which cells take an attempt,
     the order of those tests, and what each attempt does.
     - Eligible tests take their chance draws in this order: high, bridge-role family; high,
       wooden-bridge-role family; low, wooden-bridge-role family; low, bridge-role family.
       The ion-cannon warhead bypasses these chance draws.
     - *High test of a family:* the detonation's centre cell is a high-bridge cell whose section
       overlay belongs to the family, or the centre cell's tile is one of the family's piers in
       state 0 to 3. The height window of S-0303 applies only when the centre cell is a high-bridge
       cell, so a pier cell without a deck is attempted at any height.
     - *Low test of a family:* the centre cell holds one of the family's low pieces that is not in
       the destroyed class.
     - Every attempt acts on the centre cell (claim 2).
     - The ion-cannon warhead's repeated attempts (S-0303 claim 18) apply to the high tests only. A
       low test makes one attempt whichever warhead it is.
     - A success makes the centre cell untargeted under claim 22. This does not apply the
       effect to the other cells of the removed section.
   - *The trigger action that destroys attached objects* (S-0011 claim 27). After its hits, when
     the raise that fired the trigger carries a cell, the action makes up to 4 attempts at that cell
     and stops at the first that succeeds. The raise of claim 8 carries no cell.
   - *The demolition-blast trigger action, code 63* (S-0524 claim 5). After its five area
     damages, a defined waypoint receives up to 4 direct attempts at its cell, stopping at the
     first success. Bridge effects within those area damages remain subject to the area-damage
     gate.
   - *Hut demolition* (claim 16).
   - *The gate.* Only area damage is gated by the game's bridge-destruction setting. Both trigger
     actions' direct attempts and hut demolition make their attempts whatever the setting. In a skirmish the setting
     is the game option of the setup screens; the map's own setting has no effect (Import mapping).
   - Backing: `measurable` (B1, B9, B11 mixed-test variant, and B10 for gate/trigger/target effects,
     S-0524 scenario C5 for code 63)
   - Confidence: medium (test order and acceptance, the low single attempt, the gate); low (the
     trigger cell)

2. **What one attempt at a cell acts on.** The cell's first matching condition in the table determines the effect; lower rows apply only when no earlier condition matches.

   | Cell condition, in priority order | Effect |
   |---|---|
   | The cell holds a low piece that is not in the destroyed class | The low strip is affected (claim 13). |
   | The cell is a high-bridge cell and a family qualifies as described below | Its responsible section is affected (claim 3). |
   | The cell's tile is a pier of either family in state 0 to 3, and its sub-tile is even along x or at most 4 along y | The pier is affected through its reference cell (claim 6), when that cell holds the attempted cell's own tile; otherwise nothing happens and the attempt fails. Other pier cells without a deck take no effect. |
   | None of the above | Nothing happens and the attempt fails. |
   The responsible section is the cell itself when it retains the section-cell marking,
   even if a later section also marks it. Otherwise it is the last section to mark it as a
   covered cell. Later covering marks replace that remembered responsibility, without removing
   a pre-existing section-cell marking. There is no nearest-overlay or directional tie-break.
   A missing or replaced responsible overlay does not cause a search for another section.
   Family selection prefers the bridge-role family's responsible overlay or a matching
   non-collapsed pier tile at the attempted cell, then the corresponding wooden-family test.
   Without either a recognised responsible overlay or such a pier tile, the attempt fails.
   On a high-marked pier whose responsible overlay has been replaced, the pier can still
   supply the family: the attempt applies claim 3 to the responsible cell's retained section
   state, and can remove that replacement overlay. A state outside the section-state table
   gives no change. This is not a no-deck pier attempt. A missing responsible section altogether
   is malformed input: our engine does nothing and warns, rather than attempting to use an
   absent section.
   - Backing: `measurable` (scenarios B1, B4, B7 and B15)
   - Confidence: high for responsible-section selection; medium for the other rows and the
     retained-state malformed-pier case

### High bridges

3. **A direct attempt on a section.** The section's state decides the outcome:

   | State | New state | Notices sent (claim 4) | Outcome |
   |---|---|---|---|
   | intact or chipped on either side | damaged | damage notice to both neighbours | fails |
   | damaged | lost | loss notice to the far-side neighbour, then to the start-side neighbour | succeeds |
   | open on the start side | lost | loss notice to the far-side neighbour only | succeeds |
   | open on the far side | lost | loss notice to the start-side neighbour only | succeeds |

   An intact section therefore needs two attempts, and a section next to a lost one needs only
   one. Chance draws gate area-damage attempts, not other causes. Claim 5 says what losing a section does.
   - Backing: `measurable` (B1 for states, B14 for notice-side effect order)
   - Confidence: medium

4. **Notices.** A notice goes to one cell, the neighbour on one side of the sender, and arrives on
   the side that faces the sender. A notice changes only a section, a pier or a ramp in that cell:
   - **Damage notice to a section:**
     - intact → chipped on the arriving side,
     - chipped on the other side → damaged,
     - otherwise no change.
   - **Loss notice to a section:**
     - intact, chipped or damaged → open on the arriving side,
     - open on the other side → lost at once, with claim 5's effects, and the same loss notice passes
       on to the next cell beyond it on the side away from the sender,
     - open on the arriving side → no change.
   - **Notice to a pier:** claim 6.
   - **Notice to a ramp:** either kind gives the ramp's whole tile its alternate (damaged) look
     (S-0010). This changes drawing only.

   A section left with lost neighbours on both sides therefore falls at once, and the fall runs on
   along the bridge as far as the sections it isolates.
   - Backing: `measurable` (scenarios B1, B2)
   - Confidence: medium

5. **Losing a section.** When a section is lost:
   - Its overlay is removed from its cell. Its four cells lose their high-bridge marking and gain the
     bridge-body mark, as S-0010 claim 25 ("In play") gives.
   - All four cells receive claim 9's blow-ups. Their deaths and explosion draws are ordered
     by cell: section cell, one step along the marking direction of S-0211 claim 15, two steps
     along it, then one step against it. Objects killed in different cells therefore have
     distinct positions in the death and random-draw record.
   - For a lost section that is the target of a direct attempt, the notices of claim 3 (and any
     losses they cause) come before its own blow-ups.
   - After the attempt's losses come the span check of claim 7 and the connectivity update of
     claim 12.
   - Backing: `measurable` (scenarios B1, B3, B14)
   - Confidence: high (the cells); medium (the order)

6. **Piers.** A pier changes state as follows ("start side" and "far side" are the sides on which a
   notice arrives):

   | State *k* | Direct attempt | Damage notice, start side | Damage notice, far side | Loss notice, start side | Loss notice, far side |
   |---|---|---|---|---|---|
   | 0 intact | 3, damage notice both ways; fails | 1 | 2 | 3 | 3 |
   | 1 chipped start side | 3, damage notice both ways; fails | 1 | 3 | 1 (`quirks.pier_loss_notice_skips_chipped`) | 3 |
   | 2 chipped far side | 3, damage notice both ways; fails | 3 | 2 | 3 | 2 (same quirk) |
   | 3 damaged | collapse; succeeds | 3 | 3 | collapse | collapse |
   | 4 collapsed | not attempted | 4 | 4 | 4 | 4 |

   A direct attempt sends the pier's notices from its reference cell to the neighbours across
   its two sides along the axis.
   - **Direct collapse:** the pier becomes state 4 on its affected tile cells (Terms), and the
     height drop follows that replacement: each of those cells takes the reference cell's
     pre-collapse height level minus 4. Its column receives claim 9's blow-ups at pre-collapse
     heights. Deaths and explosion draws occur in decreasing-cross-coordinate, reference,
     increasing-cross-coordinate order: y across an x bridge, x across a y bridge. These effects
     precede losses caused by the pier's loss notices. Far-side losses precede start-side
     losses. After the tile replacement and both notices, claim 7 searches from two named loss
     cells, start-side neighbour before far-side neighbour.
     These are not guaranteed to select the two former adjacent spans: the collapsed pier is
     no longer an endpoint for either search. The first selected check completes before the
     second, including all band raises. Subsequent routes reflect claim 12's connectivity,
     naming the reference cell.
   - **Collapse from a notice:** the loss propagates onward on the side away from the sender,
     sent from the cell the notice arrived at.
     Collapses further along that direction produce their blow-up effects before this pier does.
     This pier's three blow-ups then occur in centre, decreasing-y, increasing-y order along x,
     or decreasing-x, centre, increasing-x order along y. Its tile and height change as the
     bullet *A received notice uses no reference cell* gives, and it starts no span check or
     connectivity update of its own.
   - **A reference cell that does not hold the pier (Q-3046).** A direct attempt acts on the pier
     only when its reference cell (Terms) holds the attempted cell's own tile, at sub-tile *r*: the
     same family, axis and state. Then every effect above uses that cell, so the pier's state is
     the attempted cell's state. Otherwise the attempt changes nothing, fails and sends no notice,
     and our engine warns. This covers a reference position painted over with another tile, and a
     part of the pier cut off from the reference cell's component (Terms) whose state differs from
     the reference cell's. A cut-off part in the reference cell's state acts on the pier like any
     other pier cell.
     DECIDED (autonomy, 2026-11-13), not a quirk: the original's outcome here (Edge cases,
     *Malformed responsibility and cycles*) needs map data that paints over a pier, and can hang
     the game.
   - **A received notice uses no reference cell.** Where a notice is received, it reads the state
     of the cell it arrives at and replaces that cell's component (Terms), so a cut-off part
     changes only through notices that arrive at its own cells, and the rest of the pier only
     through notices that arrive at theirs. A collapse from a notice takes its blow-up cells and
     its height from a centre cell: the arriving cell itself when its sub-tile is even (along x) or
     below 5 (along y), and otherwise the cell one step from it toward the start side. The three
     cells that blow up are the centre cell and its two neighbours across the bridge, and every
     cell of the replaced component takes the centre cell's height level minus 4. Along y, this
     rule states no relation between the reference cell and the centre cell for an arriving
     sub-tile of 5 or more. The onward notice also goes from the arriving cell, in a cut-off part
     as elsewhere.
   - Backing and confidence, item by item:
     - Direct stages, collapse and height drop: `measurable` (B4, with B14 for collapse-effect
       order and B15 for connected tile cells and span selection). Medium.
     - The notice table: `measurable` (B4). Low.
     - The original's outcomes with a reference cell that does not hold the pier (Edge cases,
       *Malformed responsibility and cycles*): `measurable` (B17). Medium for B17 a and b; low for
       B17 a2 and a3, whose expected results rest on the notice table. Whether those attempts
       fail or succeed, and a3's blow-ups: `unmeasured`, because B17 records neither. Medium.
       The pier's own cells after a3's attempt: `unmeasured`, because B17 a3 does not record
       them. Low.
     - A cut-off part in the reference cell's state acting like any other pier cell: `unmeasured`,
       because B17 b attempts the cut-off part only after the main part has collapsed. Medium.
     - That a received notice uses no reference cell and sends onward from the arriving cell, and
       the centre cell of a collapse from a notice: `unmeasured`, because no test map here sends a
       notice into a cut-off part or outside the reference cell's line along the bridge (a notice
       arrives in its sender's line along the bridge, and which lines the neighbouring pieces send
       from is not settled here). Medium.

7. **The span check: isolated runs fall.** After each successful direct attempt on a high bridge, the
   checked extent is selected by the searches below. A collapsed pier can join two former
   spans into one checked extent rather than keeping their old boundary.
   - **Finding the span.** From the named loss cell, the first neighbour in N, NE, E, SE,
     S, SW, W, NW order carrying a deck or bridge-body mark supplies the bridge. Resolve its
     responsible section, or its body-mark section by claim 16's body rule. From that section
     cell, search toward the start side up to the legal-map boundary, passing over non-bridge
     cells, for the nearest start ramp at sub-tile 8 along x or 12 along y, or non-collapsed
     pier at sub-tile 5 along x or 7 along y. No qualifying neighbour, body resolution or start
     piece means no span check. For direct pier collapse the two named loss cells are the
     reference cell's start-side neighbour and then its far-side neighbour. Both searches use
     the post-collapse tiles and the marks left by the notices, not a saved pre-collapse span.
     A collapsed pier, whether collapsed by this attempt, an earlier attempt or a notice,
     never qualifies as a start piece. There is no exception for the far-side named cell.
     The 29-cell cap applies to the following far-end search, not this start-piece search.
   - The span runs from its start piece (a start ramp at sub-tile 8 or 12, or a pier at sub-tile 5
     or 7) to the nearest far piece within 29 cells along its direction (a far ramp, or a
     non-collapsed pier, at sub-tile 4 or 2). Collapsed piers are passed over at this end too
     (`quirks.collapsed_pier_span_search`). Thus on an ordinary two-span bridge with no other
     pier between its outer ramps, both named searches after direct pier collapse select the
     same outer-ramp-to-outer-ramp extent, provided their marked neighbours resolve to that
     bridge. They do not check just the former start-side span twice. If the outer endpoints
     exceed the cap, neither check has an isolated-run or band-raise effect. Within the cap,
     the second check repeats the same band on the state left by the first. If the first
     check loses *n* isolated sections and at least one gap remains, the two checks together
     raise that combined band *n* + 2 times: *n* + 1 from the first and one from the second.
     Trigger actions that change the bridge can change the second search's result.
   - Consider the cells from the start piece's cell up to, but not including, the far piece's cell.
     Every run of consecutive section cells that includes neither the first of those cells nor the
     last is lost, one section at a time, with claim 5's blow-ups.
   - Runs that include the first or the last cell stay.
   - Deaths and explosion draws of an isolated run occur from its far-side end toward its
     start side. With several isolated runs, the one nearest the start side loses all its
     sections before the next run's effects occur.
   - When no far piece lies within 29 cells (`quirks.bridge_span_cap`), the check does nothing.
   - The sections lost here send no notices and name no cell for connectivity.
   - When the checked span contains a cell without a section, its band receives claim 8's
     bridge-destroyed raises before the first isolated loss, between consecutive isolated
     losses, and after the last. With *n* isolated losses there are *n* + 1 band raises,
     including one when *n* = 0. A span with no missing section raises nothing.
   - Backing: `measurable` (B2, B14 for multiple runs and interleaved band actions,
     B15 for post-collapse endpoints, repeated checks and the combined cap)
   - Confidence: high (collapsed-pier exclusion and post-change searches); medium (the
     isolated-run rule and repeated-check consequence); low (the order of losses, the pass count)

8. **The bridge-destroyed event.** S-0011 claim 20's event code is raised from two places: from claim
   7's passes for a high bridge, and once per destroyed low strip (claim 15).
   - *The raise:* for every tagged cell in a band, the cell's tag is raised with that code, with
     no object and no cell.
   - *The band* has endpoints at the first cell after the span's start piece and at its far
     piece. For a low strip its endpoints are the first and last low pieces of the bridge's run,
     found along the centre row through the strip. Destroyed pieces count as part of the run.
   - *Which cells are raised:* taking the band's cells in order of increasing x (or y), and for
     each of them the cell itself, the next cell across toward increasing y (or x), and the one
     and two cells across toward decreasing y (or x).
   - *The last cell along the band is never processed* (`quirks.bridge_event_band_short`). A band of
     one cell raises nothing.
   - One tag over several of these cells is raised once per cell.
   - Backing: `measurable` (scenario B5)
   - Confidence: medium (low strips, the cells across); low (high-bridge band ends, the pass count)

9. **Blow-up of a cell.** A blow-up has the following occupant and animation effects.
   Ground-layer hit effects precede deck drops; both precede explosion-choice draws. Death
   effects can therefore consume random values before that cell's explosion choices.
   - **Ground layer.** Every object listed in the cell's ground layer (S-0209) receives a hit equal
      to its current health, distance 0, with the game-wide demolition warhead. The hit ignores
      defences, carries the no-escape marker, and has no attacker and no attacking house (S-0302
      claim 1's bridge-section row; S-0302 claims 8 and 9 say which protections still stop it).
      Units under the bridge, ships included, therefore die.
   - **Deck layer.** Every object listed in the cell's deck layer leaves the deck: it moves to the
      ground layer of the same cell at its current position and starts falling with the bomb
      mark (claim 10). This does not issue a stop order: target, destination, queued destinations,
      path and movement-in-progress state are not cleared by the drop. Falling itself prevents
      the ordinary locomotion advance (claim 10).
      A stationary infantry unit's standing-position mark and a stationary vehicle's present
      flag are cleared on deck during this move, and the corresponding ground mark is made
      immediately. There is no deferred transfer after claim 11. Movement reservations are
      cleared at the locomotor's heading point, when it has one, at that point's stored height,
      as well as the ordinary listing change's marks. For drive and ship locomotion that
      stored height follows S-0207 claim 24: ordinary tracks retain their starting height
      through curve chaining, whereas the driving factory exit track retains world height
      0 leptons. Neither is replaced by the object's current height or the deck's height
      at drop. These are the ordinary flag/position
      clearing rules, not ownership-counted reservations (S-0209 claim 1).
      No general deck-record reset follows the attempt, landing or repair. Unrelated marks
      remain until their ordinary clearing event. With just a stationary dropped infantry unit
      or vehicle and no other mark writer, its former deck record is already empty before
      claim 11, stays empty through its fall and is empty after repair. Its ground mark is
      present from the drop, not first from landing; its death clears that ground mark.
   - **Explosions.** When the game-wide bridge-explosion list is not empty, draws are taken from the
      scenario random stream in this order (Formulas F2):
      1. a unit-interval value *u*₁. Nothing more happens unless *u*₁ < 0.95,
      2. *u*₂ and *u*₃, which give the point,
      3. *u*₄. When *u*₄ < 0.5, a debris animation is created at the point, chosen by a ranged draw
         over the game-wide metallic-debris list, with no loop delay and one pass,
      4. an explosion animation at the point: a ranged draw R(1, 5) for its loop delay, then a
         ranged draw over the bridge-explosion list for its type, with one pass.

      Each animation's own creation draws (S-0109, S-0202) come right after its choice draw.
   - Backing: `measurable` (scenarios B3, B6, B11, B14); `unmeasured` for the original's
     empty-list result, because its out-of-list selection has no stable expected outcome
   - Confidence: medium (the kills, the drop, the draw sequence); low (empty-list behaviour)

10. **Falling and landing.** A dropped object falls on each of its own updates (S-0203 claim 4).
    For an ordinary infantry unit or vehicle, the fall and any landing hit precede that turn's
    due behaviour processing (S-0215 claim 5), ordinary locomotion advance and firing check.
    Falling does not suppress the due behaviour processing: it may change orders under the
    behaviour's usual rules. While still falling after the fall step, the unit does not advance
    its ordinary locomotion and cannot fire. A surviving landing removes those falling-specific
    refusals in that same turn. It does not restore a saved deck order or issue a guard order:
    orders are whatever the intervening ordinary behaviour and target-loss rules have left.
    The drop itself preserves them as claim 9 states.
    - Its height changes by its fall rate *v* before *v* is updated (Formulas F3). Any attached
      animation enables the parachute rate recurrence, not only a parachute (S-0522 claim 12).
      The alternative is owned by `quirks.paratrooper_any_animation_slow_fall` in S-0522.
      S-0522 claim 13 owns the absent-parachute release timing: under the default quirk,
      a fresh paratrooper released during its plane's turn uses the parachute recurrence
      once in the release tick, then the no-parachute recurrence from the next tick,
      provided no other animation is attached. With that quirk off it uses the
      no-parachute recurrence from its first update. An unlisted ordinary animation
      name is not an absent type (S-0522 claim 13).
    - Its fall rate starts from the value it already holds. That is 0 for an object that has never
      fallen. An object that has fallen before (by parachute, for example) starts from the rate it
      ended its last fall with (low).
    - When its height above the terrain after the move is 0 or less, the height is set to 0 and the
      fall ends.
    - An object that lands with the bomb mark and positive health then receives a hit equal to its
      current health, distance 0, with the demolition warhead, ignoring defences and carrying the
      no-escape marker, with no attacker and no attacking house. So every object on a lost deck
      dies on landing unless S-0302 stops that hit. Landing does not spend or clear the bomb
      mark, even when the object survives. A later fall that adds no new bomb mark still
      ends with this hit while the retained mark is present. This is one hit per landing,
      not a hit on every later grounded update. A survivor that later dies within S-0312
      claim 11's height window over water still qualifies for that claim's marked death
      effects, even though its marked fall ended earlier; that death rule does not require
      the object to be falling at death.
    - Over a whole drop from a deck (416 leptons above the terrain under it), with no attached
      animation, *v* starting at 0 and the floor of F3 not reached, the object lands on its 30th
      update (Formulas F3).
    - Backing: `measurable` (B3, including its prior-fall, turn-order and retained-mark variants) for ordinary and prior
      parachute drops and retention after survival; `unmeasured` for a later fall without
      a new mark and a later low-water death after survival: B3 specifies the conditional
      comparisons, but the established repeat-drop mechanisms write the mark again and
      therefore cannot distinguish retention from a fresh mark. Named test files cannot
      initialise this runtime history; a read-only recorder can verify it but cannot create
      a separating later transition. The later-water-death comparison likewise lacks a
      legal route from the protected land landing to the required low-water death without
      another mark writer. Also `unmeasured` here for a non-parachute attachment selecting
      the slow recurrence: no established data-only attachment effect is known here to
      remain attached through this deck drop. Renaming a parachute animation would not
      distinguish that condition (S-0522 claim 12 owns the shared rule).
      The absent-parachute release timing is `measurable` through S-0522 scenario P8.
    - Confidence: high (fall arithmetic, landing hit and retained bomb mark); medium (start from a stored rate and absent-parachute release timing)

11. **Units heading onto a lost deck.** After an attempt has made its blow-ups, each blown-up cell
    *c* whose deck-layer occupancy record (S-0209) is not empty is checked once more.
    - Let *P* be the centre of *c* at the height of its terrain floor + 416 leptons.
    - Every object in the deck layer, then in the ground layer, of each of the 24 other cells of
      the 5 × 5 block around *c* is examined.
    - It receives the hit of claim 9's ground layer when its locomotion is heading to *P*.
      This compares movement points, not the unit's final order destination:
      - Walk and hover use their current heading point, when present: its cell must be *c*
        and its stored world height must differ from *P*'s by at most 104 leptons.
      - Drive and ship accept that same heading-point test. They also accept the endpoint cell
        of an unfinished current track when it is *c* and the unit's current world height
        differs from *P*'s by at most 104 leptons. This additional track test uses the unit's
        height, not a newly sampled terrain height at the endpoint. A failed track test does
        not prevent the heading-point test. Both drive/ship tests require a current heading
        point to be present, even when using the unfinished track's endpoint.
      - A heading point keeps the height assigned when it was chosen. The test does not
        resample the cell's new surface after destruction. For drive and ship, S-0207 claim 24
        supplies the retained height from ordinary track start, unchanged by curve chaining.
        The driving factory exit track instead retains world height 0 leptons, irrespective
        of vehicle, terrain or deck height. Its stored-height test uses that zero, but its
        unfinished-track test still uses the vehicle's current height independently. Thus a
        vehicle on an ordinary track travelling under a level bridge on level ground,
        with its track begun at that ground height, fails both deck-height tests: a deck-bearing endpoint does not
        lift its heading point. Ground-layer membership alone is not a refusal, however.
        If the retained starting height is within 104 leptons of *P* but the vehicle has
        since descended beyond that tolerance, the heading-point test can still hit it
        while the current-height track test fails. The heading cell must still be *c*,
        and the nonempty deck-record gate above still applies. S-0208 claim 18 already gives the
        ordinary walker's cell-destination height. An object destination is not excluded:
        it counts when the locomotor has a qualifying heading point or track endpoint, even
        when the final object or cell destination is elsewhere.
      - Drop-pod, fly, jumpjet, rocket, teleport and tunnel locomotion do not count.
        The mech kind is not established here.
    - Backing: `measurable` (scenario B6, including its negative-kind variants) for the stated
      locomotion tests and occupancy condition. Mech is an open question, not a negative claim.
    - Confidence: medium (low for the occupancy condition and negative-kind tests)

12. **Connectivity.** After a successful direct attempt on a high bridge, the cell named for
    S-0212 claim 3 is the lost section's section cell, or a collapsed pier's reference cell. One
    cell is named per attempt, whatever claims 4 and 7 lose with it, and S-0212 claims 3 and 5
    apply. A low-bridge change (claim 15) names no cell. It changes the cells' land types, and
    connectivity is brought up to date (S-0212 claim 5).
    - Backing: `measurable` (scenario B8)
    - Confidence: medium

### Low bridges

13. **A direct attempt on a low strip.** On a three-cell-wide strip, an attempt at any of its
    cross-cells affects the same strip, centred on its middle cross-cell. For malformed widths,
    the selected centre along x is given below for an attempted cell (*x*, *y*). Along y,
    exchange x and y. Membership means a low piece of the selected family.

    | Cross-cell pattern | Selected centre |
    |---|---|
    | (*x*, *y* − 1) is not low | (*x*, *y* + 1) |
    | (*x*, *y* − 1) and (*x*, *y* − 2) are both low | (*x*, *y* − 1) |
    | (*x*, *y* − 1) is low, but (*x*, *y* − 2) is not | (*x*, *y*) |

    The selected centre's class determines the result:

    | Centre class | All three cells of the strip become | Re-evaluated neighbours (claim 14) | Outcome |
    |---|---|---|---|
    | intact | D₀ | both | fails |
    | damaged | destroyed | both, then claim 15 | succeeds |
    | far-side end piece, whole | its damaged position | decreasing x along x; increasing y along y | fails |
    | start-side end piece, whole | its damaged position | increasing x along x; decreasing y along y | fails |
    | any other | no change | none | fails |

    The three cells are written whatever they held before.
    - End pieces can be damaged but never destroyed.
    - An ordinary intact strip needs two attempts, unless claim 14 has already damaged it.
    - Every changed cell is recalculated: its land type is taken from the new overlay type when that
      type overrides the tile's land type, otherwise from the tile.
    - Then the kill check of claim 15 runs on each changed cell, marked "destroyed" only when it
      became destroyed.
    - Backing: `measurable` (B7, including malformed-width variants)
    - Confidence: medium

14. **Neighbour re-evaluation of a low strip.** A re-evaluated neighbour is the strip one step along
    the line on that side. It is re-evaluated only when its centre cell holds a low piece.
    - *Neighbour pattern.* Its two neighbours contribute the integer *s* = *a* + *b* (exact
      integer addition). Along x, *a* comes from increasing x and *b* from decreasing x.
      Along y, *a* instead comes from decreasing y and *b* from increasing y. Contribution
      matches use the re-evaluation axis's bound identities, not an other-axis piece merely
      bearing the same symbolic label:

      | Contribution | Neighbour piece, either axis and family |
      |---|---|
      | *a* = 1 | I₄, D₀, D₂, E′ |
      | *a* = 2 | D₁, X |
      | *b* = 4 | I₅, D₀, D₁, S′ |
      | *b* = 8 | D₂, X |
      | otherwise 0 | any other |

    - *New piece*, for the re-evaluated strip's family and the re-evaluation axis:

      | *s* | 1 | 2 | 4 | 5 | 6 | 8 | 9 | 10 |
      |---|---|---|---|---|---|---|---|---|
      | piece | I₅ | D₂ | I₄ | D₀ | D₂ | D₁ | D₁ | X |

    - *Applying it.* When *s* = 0, nothing changes.
      - For an intact or damaged strip, the new position replaces the strip's position on all
        three cells. If it is the same position, nothing more happens.
      - A whole end piece becomes its damaged position, wherever *s* points.
      - Any other class stays.
      - The changed cells are recalculated and kill-checked as in claim 13.
    - *Consequences.* A strip between a destroyed strip and a damaged or destroyed one becomes
      destroyed itself (*s* = 10) without an attempt. Its destruction is not a claim 15 destruction:
      it raises no event and starts no connectivity update of its own.
    - Re-evaluation does not spread further.
    - Backing: `measurable` (scenario B7)
    - Confidence: medium (low for the exact edge and damaged looks I₄, I₅ and D₀ to D₂,
      which change drawing and also decide *s*)

15. **A destroyed low strip.** When an attempt destroys a strip (claim 13, damaged row), its
    three cells show the destroyed position. Deaths caused by neighbouring strips' re-evaluation
    precede the bridge-destroyed trigger actions, whereas deaths from this strip's own kill checks
    follow those actions. The decreasing-coordinate neighbour's effects precede the
    increasing-coordinate neighbour's effects. A subsequent path computation sees the changed
    connectivity (claim 12).

    Low-strip state changes themselves blow up no cell, create no animation and take no draw.
    Death effects of their killed occupants can still create animations and take draws. Other
    strips stay unless claim 14 changes them; there is no high-bridge isolated-run collapse.
    - **Kill check.** For each changed cell:
      - Each object in its ground layer receives claim 9's ground-layer hit in these cases:
        - when the entry test below gives *blocked*,
        - when the cell became destroyed and the object is of a type with jumpjet locomotion,
        - when the object is an aircraft, even when the change was only to damaged.
      - Then every unit in the ground layer of the other 24 cells of the 5 × 5 block around it
        receives the same hit when its locomotion is heading to that cell at its floor height
        (claim 11's test) and its entry test gives *blocked*.
      **Entry test settings.** Use the object's own-kind entry outcome of S-0209 claims 4
      and 5 on the changed cell, with no approach facing, no reference height level, no source
      cell, and include the current locomotor's cell-admission test. Ordinary drive and walk
      locomotion add no refusal through that last test. This is not a terrain-only test:
      other occupants and overlays still contribute their ordinary outcomes. Only *blocked*
      (outcome 7) causes this hit, not *moving blocker*, *stationary friendly*, *destroyable
      obstacle* or other non-clear outcomes. The mover's own listed presence is passed over
      under the ordinary self-exclusion, including S-0209 claim 5 row VP1's flag exemption.
      No additional exemption for other occupants is made. A repaired low strip's changed
      land type can by itself block a ship and kill it; land type alone is not the general
      definition of this check. The check uses the state left by preceding deaths in the
      same change, not a snapshot before the change.
    - Backing: `measurable` (B5, B7, B6 low-bridge variant, B14 low-order variant and B16)
    - Confidence: medium (low for the jumpjet and aircraft cases)

### Demolition and repair through the hut

16. **Hut demolition.** A bridge repair hut demolishes its bridge in two cases:
    - when it is the target of a timed bomb that detonates (after the bomb's own detonation),
    - when its delayed-kill countdown expires (S-0302 claim 16).

    Neither case applies the gate of claim 1. The expiry clears the countdown and the recorded
    planter, and the hut takes no expiry hit.
    All searches here start from the hut cell defined in Terms, including on a multi-cell
    foundation. Claim 18 instead uses the engineer's entry cell, before the engineer is removed.
    - **The family.** The wooden-bridge-role family is chosen when any cell of the 5 × 5 block
      around the hut's cell has a tile of the wooden-bridge role or holds a wooden low piece. The
      bridge-role family is chosen otherwise (`quirks.bridge_family_by_block`).
    - **A low bridge.** The selected piece has the smallest x in the block, breaking ties by
      smallest y, among low pieces of the chosen family. Its strip centre is defined by claim
      13. Let *w* and *e* be 1 plus the respective counts of consecutive centre-row low pieces
      toward decreasing and increasing axis coordinate. The first affected strip is displaced
      from that centre by −trunc((*w* − *e*) / 2) cell steps, using exact integer counts and
      division truncated toward zero. Up to four consecutive strips are affected, toward
      decreasing coordinate when *e* < *w*, increasing coordinate otherwise, bounded by the
      contiguous low-piece run.
      Each selected non-destroyed strip has three F2 low-form animations before its attempts,
      in increasing cross-coordinate order. It receives up to three attempts, ending at its
      first success. Already-destroyed strips have no animations or effective attempt. Deaths
      and random values of one selected strip precede those of the next. Subsequent ground
      routes reflect the changed connectivity.
    - **A high bridge.** Otherwise the bridge cell is taken from this list, first match:

      | Order | Bridge cell |
      |---|---|
      | 1 | the hut's cell, if it is a high-bridge cell or has the bridge-body mark |
      | 2 | the first such cell found by trying the directions N, NE, E, SE, S, SW, W, NW in turn, and in each direction distances 1, 2, 3 |
      | 3 | none: nothing more happens |

      - A high-bridge cell gives its section cell.
      - For a body-marked cell, let *d* be the across-bridge direction toward increasing y
        on an x bridge, increasing x on a y bridge. Let *t* be the distance to the nearest
        cell without the body mark in that direction. For 1 ≤ *t* ≤ 4 cell steps, the selected
        section cell is displaced by (*t* − 2)*d*. If none exists within four steps, nothing
        happens. This is exact integer displacement without rounding.
      - The selected span starts at the nearest start piece *S* on the start-side ray from
        the section cell within the map: a start ramp, or a pier at sub-tile 4 or 2. Its far
        endpoint is the nearest far piece on the opposite ray from *S*, at sub-tile 4 or 2.
        Demolition makes up to three attempts at *S* and up to three at the cell immediately
        before that endpoint. Start-end attempt effects precede far-end attempt effects.
        Claim 7's isolated-run losses follow these end losses, so the whole ordinary span
        falls. Subsequent routes reflect the changed connectivity.
    - Backing: `measurable` (B9, B12 search variants and B14 demolition variant)
    - Confidence: medium (low for the high-bridge search order)

17. **An engineer enters the hut.** When an infantry unit of an engineer type enters a building whose
    type is a bridge repair hut, the following happen at once, in the step in which it enters.
    Neither the hut's owner nor any alliance is checked.
    - The hut has the entered-by raise of S-0011 claim 18, as for any infantry entry. Its
      trigger actions see the unrepaired bridge.
    - A local-player engineer produces the bridge-repaired announcement when a bridge-repaired
      radar event can be created at its cell. The game-wide repair sound, if set, plays at the
      hut for every player. These sounds precede repair-caused deaths.
    - The bridge has the repaired state of claims 18 to 20. Every infantry unit forgets the hut
      as a target or destination. Infantry in the hut's cells with no destination, or with the
      hut as destination, receive scatter orders (S-0208). Scatter draws follow repair draws.
    - The engineer's destroyed-by-anything raise (S-0011 claim 19) sees the repaired bridge and
      the scatter orders. The engineer leaves play without a combat death or kill credit.

    The engineer is used up even when nothing is repaired, for example when the bridge is intact
    (`quirks.repair_consumes_engineer_always`). No trigger action repairs bridges. The repair
    geometry and piece selection take no scenario-stream draw, but scatter and deaths caused by
    repair use their own rules and may draw.
    - Backing: `measurable` (scenario B12) for entry into a broken bridge's hut; `unmeasured` for
      consuming an engineer at an intact bridge: ordinary input is refused by the cursor,
      and named map/rules files provide no demonstrated way to bypass entry legality or
      initialise an already-committed entry. A read-only recorder cannot force one.
      This is a conditional entry effect, not a claim that such an order can be issued.
      Observable entry ordering is
      `measurable` through B14
    - Confidence: medium

18. **Which bridge the hut repairs.** The family is chosen as in claim 16, from the 5 × 5 block around
    the engineer's entry cell (Terms), its own physical cell at the moment of entry, not a
    substituted hut-foundation cell. The selected bridge obeys these priorities:
    - **A low bridge.** If any cell of the block holds a low piece of that family (any class), the
       first in the order of claim 16 is taken, and the low repair of claim 20 runs from it.
    - **The bridge cell, when no low piece qualifies.** Otherwise the bridge cell is found as in claim 16's table, from the
       engineer's cell, and its section cell is resolved in the same way. If none is found, nothing
       is repaired.
    - **The start piece.** The chosen start piece is the nearest legal cell on the ray from the
       section cell toward the start side, including the section cell, that is one of these:
       - a start ramp at sub-tile 8 (along x) or 12 (along y),
       - a pier in any state at sub-tile 5 (along x) or 7 (along y).

       Intervening non-bridge cells and cells of other bridges do not disqualify the candidate
       (`quirks.repair_walk_unbounded`). Only that nearest qualifying start piece is selected.
    - **Repair extent.** A selected start ramp loses its alternate look and its span has claim
      19's repaired state. A selected pier is restored under claim 19 only if it is collapsed, and
      otherwise keeps its state. Either way, the bridge selected by this claim from the cell two
      steps toward its start side and the pier's own far-side span are repaired. For several
      selected piers, restored states extend toward the start side before the far-side spans
      receive sections. Any occupant or trigger observations during repair therefore see the
      start-side repairs before the far-side ones.
    - Backing: `measurable` (scenario B12), except that a start-piece pier in state 1 to 3 keeps
      its state: `unmeasured`, because B12 repairs no such pier
    - Confidence: medium, for both parts

19. **High-bridge repair.** The repair endpoint is the nearest far ramp at sub-tile 4 along x,
    or 2 along y, within 29 cell steps in the span's direction.
    - Every pier before that endpoint (at sub-tile 4 along x or 2 along y) is restored, and the
      endpoint ramp loses its alternate look. When no endpoint exists within 29 steps
      (`quirks.bridge_span_cap`), the piers within that distance are still restored, but no
      section is placed. Thus a failed long-span repair can visibly restore its piers alone.
    - With an endpoint, every eligible cell from the start cell through the cell before the
      endpoint receives an intact section of the selected family. The ineligible pier cells are
      even sub-tiles along x, or sub-tiles at most 4 along y.
    - Its overlay is the selected family's Hₓ along x or Hᵧ along y; its data value is 0 along
      x and 9 along y. Each placement marks its four cells (S-0211 claim 15) and clears their
      bridge-body mark (S-0010 claim 25).
    - A cell whose slope shape (S-0010 claim 6) is greater than 4 receives no replacement overlay.
      Nothing is blown up.
    - **Restoring a pier.** The pier's tile goes back to state 0 on its affected tile cells
      (Terms), from any state within a span, and only from collapsed at a start piece (claim
      18). If it was collapsed (state 4), three cells each rise by 4 height levels: the selected
      cell within a span, or the cell one step toward the start side from a start piece, and that
      cell's two neighbours across the bridge. Other cells of the pier's tile are not raised
      (`quirks.pier_repair_partial_height`). This rule assumes the start piece's tile cells are
      not painted over; other layouts are not settled here (Edge cases).
    - **Connectivity.** Each restored pier names a cell for S-0212 claim 3: the selected cell
      within a span, or the cell one step toward the start side from a start piece. So do the far
      ramp and the cell one step before it. Then connectivity is updated.
    - **Objects.** No object is harmed or moved by a high-bridge repair.
    - Backing: `measurable` (scenario B12) for collapsed piers, `unmeasured` for restoring a
      span pier from states 1 to 3, because B12 repairs only collapsed piers
    - Confidence: medium (low for the pier height rule)

20. **Low-bridge repair.** From the piece found by claim 18, the centre row is found as in claim 13.
    - The repair affects the whole contiguous centre-row low-piece run containing that piece,
      with no length cap. Piece choices and occupant deaths occur in increasing axis-coordinate
      order, so each repaired strip's random values precede the next one's.
    - Each centre cell whose piece is damaged, destroyed, I₄ or I₅ becomes Iᵣ of its own
      family and axis. The value *r* = 0 to 3 is drawn from the **map-generator stream**
      (S-0202 claim 13; Formulas F4).
    - A damaged end piece becomes its whole position, and any other piece stays.
    - When the centre cell changes, all three cells of its strip get the new position. They are
      recalculated and kill-checked as in claims 13 and 15.
    - A unit standing where a repaired strip now forbids it dies. An example is a ship under the
      bridge.
    - Afterwards connectivity is updated.
    - Backing: `measurable` (B12, including the low variant) for ordinary choices and order,
      `unmeasured` for F4's rejection endpoint, because it needs one particular raw value among
      2³² values and the recorder cannot force that value
    - Confidence: medium (low for the stream)

21. **The repair cursor uses its own test.** Whether the engineer's cursor offers repair over a
    hut (S-0214 rows I3 and IC2) is decided separately from claim 18.
    - **Candidate.** In the hut cell's 5 × 5 block, choose the last qualifying cell in increasing
      y, then increasing x order. A cell qualifies when its tile belongs to either bridge role
      (all 16 positions), or, otherwise, it holds a low piece of either family, destroyed
      included. At one cell the high tile test takes precedence over its low overlay.
      If both high-role ranges contain that tile, the wooden-bridge role supplies its position.
      No candidate means no repair cursor. This does not use claim 16's three-cell rays,
      engineer position, low-first priority or family-by-block choice.
    - **Low candidate.** On its own row, not a re-centred strip row, its axis's contiguous run
      of that family's low pieces must contain a destroyed piece. Test both directions with
      the candidate included. Neither damaged pieces alone nor a nearby disabled link offers
      repair for this candidate.
    - **High candidate.** Let its placed-tile origin be found from its sub-tile and tile width
      as in S-0307 claim 23. Its zero-based position within the bridge role selects a probe
      displacement and travel direction from this fixed table, independent of the theater's
      named ramp and pier positions:

      | Position within role | Probe relative to origin, cells | Direction |
      |---|---|---|
      | 0, 1 | (1, 2) | increasing x |
      | 2 | (0, 2) | decreasing x |
      | 3, 4 | (2, 1) | increasing y |
      | 5 | (2, 0) | decreasing y |
      | 6 to 15 | no probe | no repair cursor |

      **Link-traversal outcomes.** At each probe, eligible links are those, including low links, whose link-distance from the probe is at most 3 cells (S-0212 claim 3). If eligible links exist, the one created earliest is selected; distance does not break ties. If none exists, the cursor does not offer repair and this test creates no links. A selected disabled link offers repair. A selected enabled link also offers repair when neither endpoint is the current traversal cell (`quirks.bridge_cursor_unrelated_link`). Otherwise, the next probe is one cell from the selected link's opposite endpoint, in the direction given by the probe table; the initial traversal cell is the initial probe. Thus an enabled-link chain can lead to a later disabled link, and selection is not limited to one link near the hut.
    Cursor and actual repair may therefore select different bridges in mixed layouts.
    - Backing: `measurable` (B13 and its mixed-candidate and link-chain variants)
    - Confidence: high for candidate order and table; medium for link traversal

22. **Cell target loss after area damage.** On each successful area-damage bridge attempt,
    only references to the detonation's centre cell itself are affected, not references to
    other marked cells of the section, or to units standing there.
    - A combat object's attack target equal to that cell is lost. A mobile object first
      reverts a suspended behaviour, if any, restoring the target and destination put aside
      by its override (S-0215 claim 10). If that still leaves the centre cell as its attack
      target, that target is cleared. Without a suspended behaviour there is no such revert.
      This event neither creates nor refreshes attack-move memory (S-0309 claim 23).
      Destinations, queued destinations and remembered references are not generally erased
      just because they name the cell. A revert can nevertheless change destination and
      behaviour by its own rules; without a revert this event assigns no new behaviour.
      An aircraft in *attack* also restarts that behaviour's attack-run substate when its
      target is lost, without changing the behaviour itself.
    - A projectile's target or recorded firer equal to this cell is cleared too. Ordinary
      projectiles do not have a cell as firer, but this is not a general deletion of a cell's
      other references. Subsequent firing and targetless behaviour follow their ordinary
      rules, not a special immediate guard order.
    - Backing: `measurable` (B10 target-loss variant) for combat target loss and revert.
      The recorded-firer equality is `unmeasured`: projectile creation supplies a firing
      combat object or no firer, not a cell. Named rules and map data can change projectile
      types and targets but cannot assign a cell as their recorded firer; a read-only
      recorder cannot create that state either. Testing the equality itself would require
      a runtime-state write, outside the measurement setup. B10's aircraft and projectile
      variants cover the other consequences.
    - Confidence: high for exact-cell matching and the revert attempt; medium for the
      aircraft substate consequence

## Formulas

**F1. Tile position.** *q* = tile number − role start + 1, an integer. A piece is a pier in state
*k* when *q* = *m* + *k*, 0 ≤ *k* ≤ 4. Exact integer arithmetic, with no rounding.

**F2. Explosion points.** *u* is the unit-interval value of S-0202 claim 6, so each *u* is one ranged
draw R(0, 2147483646). For a blown-up cell (*x*, *y*) at height level ℓ:

> *X* = trunc(256 *x* + 128 + (*u*₂ − 0.5) × 50), *Y* = trunc(256 *y* + 128 + (*u*₃ − 0.5) × 50),
> *Z* = 104 ℓ + 416 leptons.

The products and sums use double-precision floating-point arithmetic. The sum is truncated toward
zero after addition. The point therefore lies within 25 leptons of the cell centre on each axis.

*Low form (hut demolition, claim 16).* Each cell takes, in order:
- two ranged draws R(0, 2147483646) for *X* and *Y*, as above but with *Z* = 104 ℓ,
- a ranged draw R(1, 5) for the animation's loop delay,
- a ranged draw over the bridge-explosion list for its type.

The animation's own creation draws follow. This form has no 0.95 or 0.5 tests and no debris.

**F3. Fall rate.** Let *hₙ* be the integer height above terrain in leptons and *vₙ* the integer fall rate in leptons per update, immediately before update *n*. For a falling object with no attached animation:

> *hₙ₊₁* = *hₙ* + *vₙ*; *vₙ₊₁* = max(trunc(*vₙ* − 1.4), *v*_min).

Here trunc is truncation toward zero of a double-precision value, and *v*_min is the imported no-parachute maximum fall rate, a negative number. For *vₙ* ≤ 0 the rate update is *vₙ₊₁* = *vₙ* − 1, so from *v₀* = 0 the drop after *n* updates is 0 + 1 + … + (*n* − 1) = *n*(*n* − 1)/2 leptons, until *v*_min is reached. The landing test (*hₙ₊₁* ≤ 0) follows the move. While any animation is attached to the falling object, the rate recurrence instead is *vₙ₊₁* = max(*vₙ* − 1, the imported parachute maximum fall rate), using exact integer subtraction and comparison. A parachute is one such animation, not a type restriction. S-0522 claim 12 and `quirks.paratrooper_any_animation_slow_fall` own this attachment condition and its alternative. S-0522 claim 13 owns the absent-parachute release-tick transition described in claim 10 here; absence of a durable parachute does not imply the no-parachute recurrence on that first update.

**F4. Low repair piece.** One raw value *g* of the map-generator stream, read as an unsigned 32-bit
integer:

> *r* = trunc(*g* × 4 × *c*)

Here *c* is the double-precision constant nearest 1 / (2³² − 1). Both multiplications use
double-precision floating-point arithmetic; their final result is truncated toward zero to give the
integer *r*. When *r* > 3 (only for *g* = 2³² − 1) another raw value is taken. Thus each repaired
centre cell uses one accepted value, with any rejected values before it, in walk order.

## Constants

| Name (ours) | Value | Unit | Notes |
|---|---|---|---|
| Deck height | 416 | leptons | 4 height levels; F2, claim 11 |
| Heading tolerance | 104 | leptons | claim 11 |
| Heading search block | 5 × 5 | cells | claims 11 and 15, the centre cell excluded |
| Hut search block | 5 × 5 | cells | claims 16 and 18 |
| Hut bridge-cell search | 3 | cells | claim 16's table, in each of 8 directions |
| Body walk limit | 4 | cells | claim 16 |
| Span limit | 29 | cell steps | claims 7 and 19 |
| Pier height drop and rise | 4 | height levels | claims 6 and 19 |
| Explosion chance threshold | 0.95 | unit-interval value | claim 9 |
| Debris chance threshold | 0.5 | unit-interval value | claim 9 |
| Explosion scatter | 50 | leptons (full width) | F2 |
| Explosion loop delay draw | 1 to 5 | ranged draw | claim 9, F2 |
| Fall-rate step | 1.4 | leptons per update | F3, double precision |
| Trigger and ion-cannon attempts | 4 | attempts | claim 1 |
| Hut demolition attempts | 3 | attempts per strip or end | claim 16 |
| Hut demolition strips | 4 | strips | claim 16 |

## State and ordering

**High-bridge section** (claims 3 and 4):

| State | Event | Result |
|---|---|---|
| intact, chipped | direct attempt | damaged; damage notices to both neighbours; fails |
| damaged | direct attempt | lost; loss notices to both; succeeds |
| open on one side | direct attempt | lost; loss notice to the other side; succeeds |
| intact | damage notice on side *s* | chipped on *s* |
| chipped on the side opposite *s* | damage notice on side *s* | damaged |
| intact, chipped, damaged | loss notice on side *s* | open on *s* |
| open on the side opposite *s* | loss notice on side *s* | lost; the notice passes on |
| any, in a run isolated from both span ends | span check (claim 7) | lost |

**Low strip** (claims 13 and 14):

| Class | Event | Result |
|---|---|---|
| intact | direct attempt | damaged; fails |
| damaged | direct attempt | destroyed; event, connectivity; succeeds |
| whole end piece | direct attempt or re-evaluation | damaged end piece |
| intact or damaged | re-evaluation with *s* = 10 | destroyed (no event) |

**Ordering consequences.**
- All of a destroy attempt's effects happen inside the detonation, action or demolition that makes
  it, before any other object updates. A unit on a lost deck is in the ground layer and falling
  before its next update.
- In a direct attempt on a section, deaths and explosion draws from notice-induced collapses
  precede those of the attempted section itself. Isolated-run losses and their trigger actions
  follow it. Units heading onto a lost deck take their hits only after all these effects and the
  connectivity change, in the order of the cells' blow-ups (claim 11).
- Along a chain of notice-induced collapses, deaths and explosion draws at its farthest end
  precede those nearer the directly attempted section. Explosion draws within each section
  follow claim 5's cell order.
- A ground-layer object dies in its cell's blow-up. A deck object dies only when it lands, some
  updates later (claim 10).

**Cross-spec boundaries.** S-0209 claim 1 cites claim 9 for immediate drop marks and no
later blanket reset. S-0203 claim 4 and S-0215 claim 5 cite claim 10 for the within-turn fall
boundary and due behaviour during falling. S-0215 claim 10 includes claim 22's cell-target-loss
revert cause. S-0214 claim 7 uses claim 21's cursor selection, not claim 18's
repair search. S-0211 claim 15 preserves claim 2's section-cell priority and historical
responsibility. S-0301 claim 9 includes claim 10's falling firing refusal and same-turn
landing boundary. The bridge-hit, retry, trigger, draw and connectivity owners remain as
cited above. Passenger escape from a destroyed transport remains unresolved here.

## Edge cases and quirks

- `quirks.pier_loss_notice_skips_chipped` (default on, the original): a pier chipped on one side
  ignores a loss notice arriving on that same side (claim 6). The usual way a section next to a
  pier is lost — damaged first, then lost — therefore leaves the pier only chipped, not damaged.
  Off ("fixed"): a loss notice turns any pier in state 0 to 2 into state 3.
- `quirks.collapsed_pier_span_search` (proposed, default on, original): claim 7 passes
  over collapsed piers at both endpoints, so collapse can combine former spans and make
  both named searches check the combined extent, subject to its combined length cap.
  Off: a collapsed pier remains eligible at the same start/far sub-tiles as a live pier,
  preserving the former span boundaries for these checks. This changes gameplay, not just drawing.
- `quirks.bridge_span_cap` (default on): a span whose far piece lies 30 or more cell steps from its
  start piece is never checked by claim 7 and cannot be repaired (claim 19). Off: no limit.
- `quirks.bridge_event_band_short` (default on): the last cell along a band is skipped, and a
  one-cell band raises nothing (claim 8). Off: the band is processed to its last cell.
- `quirks.repair_walk_unbounded` (default on): the repair's walk toward the start side passes over
  cells of other bridges and can repair a different bridge further along (claim 18). Off: the walk
  stops at the first cell that is neither a high-bridge cell nor bridge-body marked, and nothing is
  repaired.
- `quirks.bridge_family_by_block` (default on): the family is chosen from the 5 × 5 block, but the
  high-bridge search reaches 3 cells. A wooden high bridge whose tiles all lie more than 2 cells
  from the hut is handled as the other family, and nothing happens: the repair still uses up the
  engineer. Off: the family is taken from the bridge cell that claim 16's table finds.
- `quirks.repair_consumes_engineer_always` (default on): an engineer entering a hut whose bridge
  needs no repair is used up (claim 17). Off: the engineer is refused entry when claim 18 would
  repair nothing.
- `quirks.pier_repair_partial_height` (default on): a collapse lowers every affected tile cell
  (claim 6), but repair raises only the three cells claim 19
  names. Off: repair restores every cell of the pier's tile to the level it had before the
  collapse.
- `quirks.low_strip_regress` (default on): re-evaluation can turn a D₀ strip back into an
  intact edge look I₅ or I₄ when *s* is 1 or 4 respectively (claim 14). This is not known to be reachable from
  stock play, only from pieces placed damaged by a map. Off: re-evaluation never moves a strip from
  the damaged class to the intact class.
- `quirks.low_bridge_wrong_axis_recheck` (proposed, default on, the original): a low piece of the
  other axis lying along the line is re-evaluated with this line's table. Off: a neighbouring
  strip of the other axis is not re-evaluated. This matters only for unusual map data.
- `quirks.bridge_cursor_unrelated_link` (proposed, default on, original): an enabled first nearby link
  that has neither endpoint at the traversal cell offers repair (claim 21). Off: skip links
  without that endpoint and inspect the first qualifying endpoint link instead; no such link
  means no repair cursor. This changes exceptional mixed-bridge maps, not ordinary huts.
- **Malformed responsibility and cycles.** An absent remembered responsible section refuses
  an attempt and warns (claim 2). A direct pier attempt whose reference cell does not hold the
  attempted cell's own tile does nothing and warns (claim 6). The original's outcomes, by map
  layout:
  - The reference position painted with a one-cell clear tile, the pier's other cells in that
    line across the bridge left as placed: the first direct attempt at any of them stops the game
    from responding (B17 a, medium).
  - The reference position painted with a cell of another tile whose sub-tile is *r*: an attempt
    at a pier cell in that line acts on the painted cell as if it were the reference cell, with the
    attempted cell's state. The painted tile's whole eight-neighbour-connected set of identical
    cells becomes the damaged pier tile when the attempted cell is in state 0 to 2, and the attempt
    fails. In state 3 it becomes the collapsed pier tile at the painted cell's level minus 4 after
    blow-ups in the painted cell's column (claim 6's direct collapse, with the painted cell in the
    reference cell's place), and the attempt succeeds. The pier's own cells then change only
    through the notices the painted cell sends, by claim 6's notice rows and its bullet *A
    received notice uses no reference cell* for the cell each notice arrives at (B17 a2 and a3,
    low; in state 3 the pier's own cells low and `unmeasured`, because B17 a3 does not record
    them; the failure or success and the blow-ups medium and `unmeasured`, because B17 records
    neither).
  - A part in state 0 to 2 cut off as in B17 b, the main part collapsed: an attempt at the cut-off
    part turns the main part back to its damaged tile without changing any height, and fails. The
    cells turned back are the component of the reference cell's collapsed tile (Terms, direct
    attempt), so that replacement leaves the cut-off part unchanged. A
    later attempt at the main part collapses it again, 4 height levels lower still (B17 b,
    medium; the failure medium and `unmeasured`, because B17 does not record it).

  Other painted layouts are not settled here. A repeated cursor traversal cell refuses repair and
  warns, rather than allowing a nonterminating search. These are safe malformed-data departures,
  not new configurable bridge settings.
- **Empty lists.** With an empty metallic-debris list and *u*₄ < 0.5, the original reads outside
  the list. Our engine takes the draw and creates no debris animation. The same holds for an empty
  bridge-explosion list in the low form of F2, and the importer warns.

## Import mapping

| Source | Key | Our concept | Form | Absent |
|---|---|---|---|---|
| Rules `[General]` | `BridgeExplosions` | bridge-explosion list (claim 9, F2) | list of animation types | empty: claim 9 makes no draws; the low form still draws |
| Rules `[General]` | `MetallicDebris` | metallic-debris list (claim 9) | list of animation types | empty |
| Rules `[General]` | `NoParachuteMaxFallRate` | *v*_min (F3) | integer, leptons per update | −100 |
| Rules `[General]` | `ParachuteMaxFallRate` | parachute floor (F3) | integer, leptons per update | −3 |
| Rules `[CombatDamage]` | `BridgeStrength` | chance range (S-0303 claim 18) | integer | S-0303 claim 24 |
| Rules `[AudioVisual]` | `RepairBridgeSound` | repair sound (claim 17) | sound name | none: no sound |
| Rules `[CombatDamage]` | `C4Warhead` | demolition warhead (claims 9, 10, 15) | warhead type | no warhead (S-0302 Import mapping) |
| Rules `[MultiplayerDialogSettings]` | `BridgeDestruction` | default of the bridge-destruction game option (claim 1) | boolean | yes |
| Map `[SpecialFlags]` | `DestroyableBridges` | not used in a skirmish: the game option replaces it | boolean | — |
| Rules building type | `BridgeRepairHut` | bridge repair hut (claims 16 to 18) | boolean | no |
| Rules infantry type | `Engineer` | engineer (claim 17) | boolean | no |
| Theater `[General]` | `BridgeSet`, `WoodBridgeSet` | the two roles (S-0010) | tile set number | S-0010 |
| Theater `[General]` | `BridgeTopLeft1`, `BridgeTopLeft2`, `BridgeTopRight1`, `BridgeTopRight2` | start ramps along x, along y | 1-based position *q* | −1: matches no tile |
| Theater `[General]` | `BridgeBottomRight1`, `BridgeBottomRight2`, `BridgeBottomLeft1`, `BridgeBottomLeft2` | far ramps along x, along y | as above | as above |
| Theater `[General]` | `BridgeMiddle1`, `BridgeMiddle2` | *m*₁, *m*₂ (piers along x, along y) | as above | as above |
| Announcements | `EVA_BridgeRepaired` | bridge-repaired announcement (claim 17) | announcement name | none |
| Map terrain blocks and theater tile catalogue | terrain tile number, sub-tile and tile width | connected pier replacement and cursor tile origin (Terms, claim 21) | integer tile/sub-tile identities and catalogue width, S-0012 claim 6 and S-0010 claim 3 | cited specs' empty-cell rules; no separate pier-cell list |
| Map overlay blocks | high-bridge placements | responsible section (claim 2) | placement order and cell marking, S-0009 claim 12 and S-0211 claim 15 | no responsible section without a placement |
| None | none | drop timing, mark clearing, low-change entry settings, heading tolerance, hut search centres and cell-target loss (claims 9 to 11, 15, 16, 18, 22) | fixed behaviour, not imported options | not applicable |
| None | none | retained bomb mark across surviving landings and later falls (claim 10, Q-0677) | runtime state, not an imported landing option | initially unmarked; landing does not clear it |
| None | none | drive/ship heading height used for drop reservations and heading hits (claims 9 and 11, Q-0676) | integer world-height leptons; S-0207 claim 24 supplies assignment and retention | no independent height key |
| None | none | post-collapse start/far endpoint eligibility and repeated combined-extent checks (claims 6 and 7, Q-0678) | fixed searches over runtime tile states; pier identities use the theater rows above | no per-collapse endpoint override |

**Piece-role binding.** Hₓ and Hᵧ are the high-overlay identities whose marking directions
are map north and map west respectively (S-0211 claim 15). The pair identified by the original
restricted-deck exception of S-0211 claim 14 belongs to the wooden-bridge-role family, regardless
of that quirk's current setting; the other pair belongs
to the bridge-role family. These bindings identify imported types, not newly configurable keys.

For each low-bridge family and axis, let *t*₀ be its bridge-piece base type in S-0202 claim 10.
Of the two base types for a given axis, the lower input registration index identifies the
wooden-bridge-role palette and the higher identifies the bridge-role palette. The following
exact integer displacements from *t*₀ bind the symbolic roles. These are engine constants:
recognition and state transitions use fixed input-list positions, not numeric properties read
from a rules file, an art file or a map. The table expresses those fixed relationships relative
to the base types; it does not copy values assigned by stock data. A displacement has units of
overlay registry positions in the input (S-0004 claim 4). Our engine may store the resolved
identities in any order.

| Role | Displacement along x | Displacement along y |
|---|---|---|
| I₀ to I₅ | 0 to 5 respectively | 0 to 5 respectively |
| D₀ to D₂ | 6 to 8 respectively | 6 to 8 respectively |
| E, E′ | 18, 19 | 13, 14 |
| S, S′ | 20, 21 | 15, 16 |
| X | 26 | 18 |

This is the same base-type and end-row binding as S-0202 claim 10, extended to the damage
and destroyed roles. It does not infer bridge identity from art or type spelling. The importer
warns when a required bound type is missing or is not a bridge piece. Backing: `measurable`
(B7's palette variants); confidence: medium.

## Measurement scenarios

All scenarios use stock data except for the explicitly named test-data variants below,
with no engine add-on and no mods (the named test files of a scenario excepted). Named overrides are removed after their sub-test.
The bridge-destruction option is on unless stated, at any fixed, recorded game speed. Except for the ion-cannon warhead, attempts
from area damage need a chance draw (S-0303 claim 18), so scenarios other than B17 never count
ordinary shots. They fire until the bridge's
look changes and record each change, with its tick, in order. The harness should record per tick:
the overlay position and data value of every watched cell, the tile and height level of watched
cells, object deaths with their tick, object heights, trigger raises, and, where a scenario says
so, the number of draws taken from a named random stream. Use one human player and one bystander
computer player kept outside the observed bridge, hut and approach cells, with the glossary's void
rule. Record the chosen speed and replay each deterministic sub-test three times. The measurement method is a repeatable input procedure with observation of game state and random-stream draws. This is a proposal, not a measurement already performed. Void timing and survival tests if the destruction
weapon itself kills a watched unit before the bridge changes, or an unrelated actor affects it.

- **B1 (claims 1 to 5).** *Setup:* a small test map with one straight high bridge along x (7
  sections, with start and far ramps) over water. An artillery unit has a wall-flagged warhead.
  - *Procedure:* fire at the middle section until it is lost, then at one neighbour until it is
    lost.
  - *Observable:* the sequence of states of the middle section and its neighbours.
  - *Expected:* the middle section goes intact → damaged (neighbours chipped on the facing sides)
    → lost (neighbours open on the facing sides). The neighbour then goes straight from open to
    lost, with no damaged stage.
  - *Falsified by:* a section lost from intact with no damaged stage, or a neighbour passing
    through the damaged stage after its partner was lost.
  - *Variants:* repeat along y and for both families, recording the data values of the Terms
    table. Along y, the increasing-y neighbour is chipped at 14 and opened at 17, not 13 and 16.
    A contrary value falsifies the encoding. With a warhead without the wall flag no state changes.
    Repeat the hit at each of the four cross-cells: each must change the same section, or claim 2
    is falsified.
- **B2 (claim 7).** The same bridge, sections numbered 1 to 7 from the start ramp.
  - *Procedure:* lose section 2, then section 6.
  - *Expected:* in the tick section 6 is lost, sections 3, 4 and 5 are lost too, while 1 and 7
    stay. *Variant b:* lose section 1 and then section 3: section 2 is lost with it. *Variant c:*
    a span of 30 or more cells: with sections 2 and 6 lost, nothing more is lost.
  - *Falsified by:* sections 3 to 5 surviving the loss of section 6, sections 1 or 7 lost with
    them, or the long span in variant c losing its middle.
  - *Order and event variant:* use repeatable counting tags across the high-bridge band and
    record section losses and each explosion point with its random-stream values. Expected:
    the isolated run is lost in order 5, 4, 3, with four band raises for the three extra losses.
    Different loss order or raise count falsifies the corresponding part of claim 7. A one-time
    tag cannot measure repeated raises.
- **B3 (claims 9 and 10).** One vehicle and one infantry unit stand on the deck of section 4, a
  ship lies under section 4, and, where the map allows, a vehicle stands on a shore cell under
  the bridge's line.
  - *Procedure:* lose section 4.
  - *Observable:* tick of loss, tick of each death, the height of the deck units on each tick.
  - *Expected:* the ground-layer and ship units die in the tick of loss. The deck units fall,
    the drop from 0 growing by 1 lepton more on each update (F3), and die when they land: the
    30th update for a full 416-lepton drop without an attached animation. A deck vehicle over water shows the wake and splash
    of S-0312 claim 11.
  - *Falsified by:* a ground-layer unit surviving the loss, a deck unit dying before a landing
    update, a deck unit surviving its landing, or a drop profile other than *n*(*n* − 1)/2.
  - *Turn-order and marks variant (Q-0712, Q-0713):* record standing-position and vehicle
    marks, listing layer, actual position, current heading point, destination, path, queued
    destinations, attack target, behaviour and firing events. Give one tank an in-range attack
    target and a second tank and infantry unit move orders before dropping them. Expected:
    the drop changes layer and marks but does not clear their orders or movement state.
    A stationary object's deck mark clears during the drop, before the heading sweep, and
    ground is marked immediately. Due behaviour processing still follows the fall step, but
    a still-falling unit neither advances horizontally nor fires. Use a protected survivor
    on a land cell to test the landing boundary: any otherwise-due behaviour, movement or
    firing uses the landed state in that turn. Repair afterwards and test entry of the old
    deck cell: the stationary dropped unit leaves no phantom deck mark. A deferred transfer,
    suppressed due behaviour, firing in descent, erased order at drop, or a surviving phantom
    mark falsifies the corresponding rule. Record retained target/path state immediately
    at drop, separately from later behaviour changes. Three runs per kind and order.
  - *Retained-mark variant (Q-0677):* give a deck unit temporary invulnerability and
    destroy its deck over land. Record health, falling state, bomb mark and landing hit
    through landing and several grounded updates. Expected: the hit is attempted once,
    the protected unit survives, and the bomb mark stays set afterwards; no further
    landing hit occurs while it remains grounded. Clearing the mark at landing, losing
    it when protection ends, or repeatedly hitting the grounded survivor falsifies this
    rule. Repeat three times. A second bridge collapse is not a distinguishing later-fall
    test because it sets the mark anew. For a conditional later-fall test, record the
    same survivor beginning an ordinary fall that does not write a new bomb mark, and
    verify the absence of such a writer throughout. Expected: another landing hit, now
    fatal without protection; no hit with positive health falsifies the retained-mark
    consequence. A parachute fall is a candidate, but no stock-play route returning the
    same survivor to such a drop is established here. For the later-death comparison,
    require a surviving marked vehicle within 10 leptons over water, no intervening
    mark writer, and protection expired. Kill it with an ordinary hit. Expected: S-0312
    claim 11's wake/splash effects rather than ordinary vehicle death effects. Record
    mark, height, water state and death animations; ordinary effects with those gates
    satisfied falsify the consequence. Until stock fixtures establish these transitions,
    these two conditional comparisons remain unmeasured, not completed oracle runs.
  - *Prior-fall variant:* let a paradropped unit land, move it onto the deck, and destroy the
    deck. Record its fall rate at the end of the parachute drop and its next drop's first
    displacement. Expected: the latter equals the retained rate, and later heights follow F3
    with that initial rate and the imported floor. A reset to zero or a different profile
    falsifies the retained-rate rule. A falling unit updated later in the loss tick can already
    take its first falling update in that tick.
- **B4 (claim 6).** A long high bridge with a pier (a stock map that has one, or a test map).
  - *Procedure:* fire at the pier's reference cell from ground height until it collapses.
  - *Observable:* the pier tile after each change, the heights of the pier's cells, deaths of units
    on the pier column, the states of the sections beside the pier.
  - *Expected:* intact → damaged → collapsed. On collapse the pier column's units die, the pier's
    cells drop 4 height levels, and the sections on both sides become open toward the pier.
  - *Falsified by:* a collapse from intact, units on the column surviving, or no height drop.
  - *Variant (the chipped-pier quirk):* first lose the section on the pier's start side, then
    fire at the pier and count its state changes. Expected: two (the pier is still only chipped,
    so it goes damaged, then collapsed). One change straight to collapsed falsifies the quirk.
    Repeat from the far side, along both axes and for both families. In a named test map place
    each pier state separately and damage or lose its neighbour on each side. The observed tile
    must match claim 6's table, or that entry is falsified. Hit every no-deck sub-tile separately:
    along x odd sub-tiles do nothing; along y sub-tiles 0 through 4 are eligible. A contrary
    response falsifies claim 2.
- **B5 (claims 8 and 15).** A low bridge along x, 6 strips long. Every cell of its run and of the
  rows across carries a one-time tag with the bridge-destroyed code and a counting action.
  - *Procedure:* destroy strip 3.
  - *Expected:* tags on the run from the first piece up to the strip before the last piece fire,
    on the centre row, the row toward y + 1 and the two rows toward y − 1. Tags on the last
    strip's cells and on the row two toward y + 1 do not fire.
  - *Falsified by:* a tag on the last strip firing, or a tag in the expected cells not firing.
  - *Order variant:* use distinct repeatable counting actions on each tagged cell. Expected:
    increasing axis coordinate and, within it, centre, increasing cross-coordinate neighbour,
    decreasing neighbour, then two steps decreasing. A different action order falsifies claim
    8. Give several cells the same repeatable tag: it must fire once per qualifying cell, not
    once per unique tag. A different count falsifies that multiplicity.
  - *Variant:* a high bridge, losing one section: raised cells run from the cell after the start
    ramp through the cell immediately before the far ramp. A tag on that latter cell must fire,
    the far ramp's tag must not. Either opposite result falsifies the high-bridge endpoints.
- **B6 (claim 11).** Two vehicles drive toward section 4's deck, from beside the bridge's line and
  from one section away. A third vehicle drives toward a cell two sections away.
  - *Procedure:* lose section 4 while the first two vehicles are one step away. Record both
    occupancy records, heading points with world heights and unfinished track endpoints
    before and after the loss. Repeat with a stationary deck unit in the watched cell.
    A stationary dropped unit alone leaves no deck mark for the sweep: that is a negative
    control, not a way of satisfying its nonempty-record gate. For a positive test arrange
    a moving unit's reservation in the watched cell while that unit is still outside it,
    and verify that its mark remains through the blow-ups. Void the positive run if the
    recorded deck record is empty at the sweep.
  - *Expected:* when its deck occupancy record is nonempty at claim 11's check, vehicles heading
    onto the cell die in that tick, and the third does not. With an empty record there is no such
    heading hit. A hover vehicle heading there dies too. Repeat with walking infantry and a ship
    approaching a cell at floor height for the low-bridge kill check. The drop of the stationary
    unit and all earlier deaths must precede the heading hits.
  - *Heading variants (Q-0718):* set final destinations two cells past the lost section,
    but time loss during a track or walking step ending in the watched cell. Use separate
    approach points at floor and deck heights and record heights rather than inferring
    them from the final order. Expected: the current heading/unfinished-track tests of
    claim 11 decide, not the final destination. A stored deck-height heading point continues
    to qualify after deck removal, and a floor-height point does not satisfy the deck test.
    An object destination producing the same heading point gives the same result. For low
    changes the floor-height test instead accepts the floor point and rejects the deck point.
    A result depending only on the final cell, the presence of an object destination, or
    a freshly resampled surface falsifies the corresponding rule. Three runs per variant.
  - *Retained starting-height variant (Q-0676):* drive one tank on level ground under
    a high deck toward the watched cell and a second on the deck toward it. Record each
    track's starting world height, retained heading height, current height, heading cell
    and track endpoint, with the nonempty deck-record gate verified separately. The
    ground approach started at the floor fails both deck-height tests; the deck-height
    approach qualifies. Repeat with a descending approach whose retained starting height
    is within 104 leptons of the lost-deck point but whose current height is more than
    104 leptons below it. When its heading cell is the watched cell, the heading test
    still qualifies despite the failed current-height track test. Use recorder-verified
    positions and void a run lacking this separating geometry. Record dropped movers'
    reservation clearing too: the layer follows the retained heading height under claim 9,
    not a substitution of the then-current vehicle height. A floor-start heading lifted
    merely because its target carries a deck, rejection of the separating heading case,
    or clearing based on current height instead of stored height falsifies that rule.
    Repeat three times per fixture, including a chained curve retaining its start height.
  - *Factory-exit control (claims 9 and 11):* use S-0207 D28's elevated stock factory
    setup to record the produced vehicle's stored heading height throughout its driving
    exit track. Expected: 0 leptons even when its starting and current heights are nonzero.
    A copied starting height or a terrain/deck-derived heading height falsifies the exception.
    On a tiny test map arrange the exit apron at the watched bridge cell and time deck loss
    while the vehicle is still outside that cell on its unfinished exit track. Record its
    heading cell, stored height, current world height, track endpoint and the watched
    cell's deck occupancy record at the sweep. Choose *P* above 104 leptons: the zero-height
    heading test fails, but a qualifying current-height track endpoint still causes the hit.
    Compare runs with current height within and outside 104 leptons of *P*. With the other
    gates satisfied, survival in the former or a heading hit in the latter falsifies the
    corresponding branch. Void runs without the intended track, endpoint or nonempty
    record, or with an earlier killing hit. For a mover dropped during that exit, record
    reservation clearing separately: claim 9 uses the stored zero, not its current height.
    Repeat each realized fixture three times at the same recorded speed, with no engine add-on and no mods (the named test files of a scenario excepted), and
    the read-only recorder supplies the points and marks, not altered game state.
  - *Falsified by:* a heading vehicle surviving with the nonempty-record condition satisfied,
    a heading hit despite the empty-record condition, the third vehicle dying, or reversed
    death ordering.
  - *Negative-kind variants:* use separate named test maps for drop-pod, fly, jumpjet,
    rocket, teleport and tunnel locomotion. Where ordinary types cannot realize the approach,
    each map's named rules override assigns the tested locomotion to a test unit, retaining
    the other settings of that unit; remove the override after the run. Order or release it
    toward the watched cell and time collapse while it is in one of the surrounding 24
    cells. Independently retain and record a deck reservation in the watched cell, as in
    the positive control. Record the tested unit's current locomotion kind, cell, health,
    height and movement points, the reservation at the sweep, and all received hits.
    Include an ordinary drive positive control in the same surrounding block. Expected:
    the drive control receives the heading hit, but the tested negative kind receives
    none, regardless of its final destination or movement-point height. A heading hit
    on that negative kind falsifies claim 11; later crash, landing, emergence or ordinary
    weapon damage does not. Void runs with no verified sweep, an earlier hit, a different
    active locomotion kind or the tested unit outside the surrounding block. Repeat three
    times per kind with both axes. A type unable to make ordinary movement points is
    still a valid negative test: those points are not prerequisites for the blanket
    exclusion. Mech remains outside this scenario and outside the asserted table.
- **B7 (claims 2, 13, 14 and 15).** A low bridge along x, 7 strips long.
  - *Procedure:* fire at strip 4 until it is destroyed, then at strip 6 until it is destroyed.
  - *Observable:* overlay positions of strips 3 to 7 after each change, units standing on strips 4
    and 5.
  - *Expected:* strip 4 goes intact → damaged → destroyed, and strips 3 and 5 take damaged looks
    when it is destroyed. When strip 6 is destroyed, strip 5 becomes destroyed in the same tick.
    Units on destroyed strips die. End pieces never become destroyed.
  - *Falsified by:* strip 5 surviving the loss of strip 6, or a strip destroyed with no damaged
    stage.
  - *Variants:* an ion-cannon strike on an intact strip leaves it damaged, never destroyed.
    Repeat along y and for both families, and fire separately at each of the three cross-cells.
    Record the exact overlay positions and compare each neighbour change to claim 14's table.
    In a named test map place the neighbour patterns for every listed value of *s*, including
    already-damaged strips, and attempt the neighbour. Any table mismatch falsifies claim 14.
    Repeat with whole end pieces: along y, damaging the far-side end re-evaluates the neighbour
    toward increasing y, unlike the corresponding x case. A different changed neighbour falsifies
    claim 13. Named malformed-width maps separately realise each centre-selection table row,
    with distinct classes on candidate centres. Expected changes are exactly the three cells
    around the table's selected centre, even if those cells previously held different overlays.
    A different centre or preserved cross-cell overlay falsifies claim 13.
    *Palette-binding variants:* watch each symbolic role separately for both families and axes,
    recording its resolved input type rather than assuming our engine uses the same index.
    Compare the resolved type with the base-relative binding in Import mapping, then apply the
    direct-attempt and neighbour-pattern tests above. A role bound to a different displacement,
    or a transition inconsistent with its symbolic row, falsifies the binding. A named test
    rules file renames the bridge types consistently without moving their input registry
    positions: the same roles and transitions must remain, or the spelling-independent binding
    is falsified. This test does not claim that moving types to arbitrary input positions
    preserves the original's bridge behaviour.
    Place an other-axis
    neighbour to check the original wrong-axis re-evaluation and a damaged strip with *s* = 1
    and 4 to check regression to an intact class. A skipped other-axis neighbour or retained
    damaged class falsifies the corresponding default quirk.
    Record repeatable event counters and deaths on strips destroyed by re-evaluation:
    they must receive no event of their own, and their deaths precede the direct strip's band
    actions. Aircraft in the ground layer die even on a damage-only change, whereas a grounded
    jumpjet is forcibly killed on destruction. Contrary survival falsifies those kill-check rows.
- **B8 (claim 12).** A vehicle ordered across a high bridge after a section is lost.
  - *Expected:* it routes around the bridge, or reports the destination unreachable, from the tick
    of the loss.
  - *Falsified by:* a route planned over the lost section.
  - *Named-cell variant:* construct separate bridges with endpoints within 3 cells of only the
    section cell, only an outer marked cell, or a pier reference cell. After destruction, compare
    cross-bank reachability on each bridge against S-0212 claim 3's named-cell distance rule.
    A change matching an outer marked cell instead of the section or reference cell falsifies
    claim 12. Repeat after repair, isolating the restored pier, far ramp and preceding cell.
    A changed low strip must affect ground routes through its changed land type, not by enabling
    a high-bridge link.
- **B9 (claims 1 and 16).** A bridge repair hut beside a high bridge and another beside a low
  bridge.
  - *Procedure:* (a) a timed bomb on each hut; (b) with the bridge-destruction option off, the same.
  - *Expected:* (a) the whole high span falls. The low bridge loses up to 4 strips from its middle,
    each with explosions. (b) The same, while fire at the bridges changes nothing.
  - *Falsified by:* a high span only partly lost, more than 4 strip centres directly attempted
    by the demolition, or no demolition with the option off. Extra neighbouring losses from
    claim 14 are counted separately.
  - *Expiry variant:* use S-0302 claim 16's delayed-kill setup on a hut. At expiry the bridge
    must be demolished, the countdown cleared and the hut left without an expiry hit. A repaired
    bridge or expiry damage to the hut falsifies claim 16. Record low-demolition animation points,
    delays and random-stream values: each non-destroyed selected strip has three F2 low-form
    animations before its attempts. Different draw values, points or ordering falsify that part.
- **B10 (claim 1, gate).** Skirmish with the bridge-destruction option off, on a map whose own
  setting is on.
  - *Expected:* area damage never changes a bridge. With the option on and the map's setting off,
    it does.
  - *Falsified by:* the map's setting deciding either case.
  - *Trigger variant:* a tagged bridge cell raises an arrival event whose action destroys attached
    objects. With the game option off, the section should still be lost in that action, after up
    to four direct attempts. No change despite a qualified cell raise falsifies claim 1's trigger
    effect. Use a bridge-destroyed raise with the same action as the no-cell control: it must not
    start another cell attempt. A completed raise's centre-cell target must be dropped after a
    successful area-damage attempt in a separate run; a retained attack target falsifies that
    subclaim.
  - *Target-loss variant (claim 22, Q-0719):* simultaneously watch a unit force-firing at
    the centre cell, another moving toward it on the ground, and a third firing at a
    neighbouring cell of the same section. Record current/suspended behaviours, target,
    destination, queued destinations and attack-move memory immediately around success.
    Expected: the first loses its centre-cell attack target; the other two references
    remain. No attack-move memory is created or refreshed. Repeat with a suspended behaviour
    and target restored by an override revert: that behaviour, target and saved destination
    return before any still-equal centre target is cleared. Without a suspended behaviour,
    no new guard behaviour is assigned by success itself. An indiscriminate destination
    clear, neighbouring-target loss, new memory, or missing revert falsifies claim 22.
    Repeat three times; observe later automatic target acquisition separately. Aircraft
    in an attack run aimed at that cell restart the run's substate but retain *attack*.
    A travelling projectile targeted at the centre cell loses that reference; one aimed
    at a neighbouring cell does not. Record the references and aircraft run phase without
    treating persistence of the projectile itself as target retention. Failure to reset
    the aircraft run, or clearing a neighbouring projectile target, falsifies claim 22.
  - *Demolition-action variant:* fire code 63 at a defined waypoint on an intact high section
    and then on an intact low strip, with bridge destruction off. Record the five damage events
    and subsequent bridge states. Expected: direct attempts still remove the section or destroy
    the strip after the damage events, as S-0524 claim 5 and scenario C5 specify. A bridge left
    intact solely because the option is off, or a direct loss before those damages, falsifies
    this part of claim 1.
- **B11 (claim 9, draws).** With the harness counting scenario-stream draws, lose one section in an
  otherwise quiet game.
  - *Expected:* 4 blow-ups. Each takes one unit-interval value and, when it is below 0.95, three
    more, a debris choice when the fourth is below 0.5, then R(1, 5) and the explosion choice,
    each followed by its animation's creation draws.
  - *Falsified by:* a draw count no assignment of these outcomes explains, an explosion point
    inconsistent with F2, or a cell order inconsistent with claim 5. Record values as well as
    counts so animation choices, chance tests and creation draws can be separated.
  - *Mixed-test variant:* a named test map places a non-destroyed low overlay on a pier tile
    without a deck, so both high and low tests qualify. Record every transition and chance draw.
    The first eligible high test acts through claim 2 on the low strip before its low test can
    act again. Repeat with a high overlay of one family on a pier tile of the other, and with
    each low family on each pier family. Assign recorded chance values to transitions in claim
    1's test order. Reversed effective order, or an ion-cannon chance draw, falsifies claim 1.
- **B12 (claims 17 to 20).** An engineer enters the hut of a high bridge with one lost section,
  then a second engineer enters the same hut.
  - *Observable:* the deck after each entry, trigger raises (entered-by on the hut,
    destroyed-by-anything on the engineer), the announcement and the sound.
  - *Expected:* the first engineer restores every eligible section before the far ramp and
    restores collapsed piers encountered before it. Record the cursor and order result for the
    second engineer separately: an intact-bridge refusal is not evidence against the conditional
    entry effects of claim 17.
  - *Falsified by:* a missing eligible section after completed entry, or a first engineer retained.
    Record ground and deck occupants' positions and health during high repair: no repair-caused
    change is expected. Movement, injury or a drop caused by high repair falsifies claim 19.
    A named test map supplies eligible cells of slope shapes 4 and 5: only shape 4 receives an
    overlay. The opposite result falsifies the slope exclusion.
  - *Variant (walk):* two broken high bridges along the same line on the start side of the hut's
    bridge, which is itself broken. One engineer repairs the hut's own bridge only; a second
    bridge repaired falsifies claim 18. In a separate map leave a non-bridge gap on the start-
    side ray before the nearest qualifying start piece of another bridge. That piece must
    still be selected; no repair falsifies the unbounded-selection quirk. With several piers,
    record restored tile heights and section placements: start-side repair effects precede
    far-side placements, as claim 18 states. Opposite ordering falsifies that subclaim.
  - *Low variant:* repaired strips show intact looks, and a ship under the low bridge dies when it
    is repaired. With the harness counting map-generator-stream draws, one value is taken per
    repaired strip, except for F4's rejected values. Compare each accepted value to its resulting
    piece using F4, in increasing axis-coordinate order. A scenario-stream choice draw,
    different piece or reversed draw/death order falsifies claim 20. Repeat with damaged end
    pieces, intact edge looks and intact ordinary looks: the first become whole ends, the
    second receive F4 choices, and the last remain unchanged without a piece-choice draw.
    Any other transition or extra choice draw falsifies the corresponding row.
  - *Pier and cap variants:* collapse a pier and record every tile-cell height before collapse,
    after collapse and after repair. Only the three cells claim 19 names rise by 4 on repair. Any
    other raised cell, or one of those three left unraised, falsifies claim 19. Repeat on a span
    with the far ramp at distance 29 and then 30: the latter places no sections, although piers
    restored before that distance stay restored. Contrary overlay placement falsifies the cap.
  - *Search variants:* named test maps place candidate bridges on several of the hut's eight
    search rays at distances 1 to 3, and low bridges in several cells of its search block. Record
    which bridge changes. The first candidate by claims 16 and 18 must win, with low pieces
    taking precedence. A different repaired or demolished bridge falsifies that selection.
    A wooden high bridge detected only at distance 3, with no wooden tile in the smaller block,
    should not be repaired. Successful repair falsifies the family-selection quirk.
  - *Side effects:* compare friendly, neutral-owned and enemy-owned huts at the same geometry.
    Completed entry must repair equally. Watch other infantry targeting the hut and infantry in
    its footprint: targets are forgotten and eligible stationary infantry scatter. Different
    ownership results, retained hut targets, or missing tag raises falsify claim 17. Record
    the local engineer's announcement and positional repair sound on completed broken-bridge
    entry. A named loose test rules file omits the repair-sound setting at a fresh program
    start: no positional repair sound is expected, and a sound falsifies that absent-key rule.
- **B13 (claim 21).** *Setup:* a three-cell-wide low bridge, its hut and an engineer, owned by
  the human player, with one bystander computer player kept away. *Procedure:* record the cursor
  over the hut when the bridge is intact, after a middle strip becomes damaged, and after it
  becomes destroyed. *Observable:* whether the cursor offers repair. *Expected:* only the destroyed
  state offers it. Offering repair for damage alone, or refusing it for the destroyed run,
  falsifies the low-bridge test. Repeat for both axes and both families. Repeat with one isolated
  high bridge: intact and damaged sections leave its link enabled and offer no repair, whereas a
  lost section disables it and offers repair. A contrary cursor falsifies the ordinary high test.
  - *Mixed-candidate variants (Q-0715, Q-0716):* named test maps put distinct intact and broken
    bridges in the block with their candidate cells ordered differently. Expected: the last
    cell in y-major/x-minor order decides, high or low, not the first low piece, and the hut's
    top-left foundation cell centres the block. A bridge only on a distance-3 ray gives no
    candidate. With a 2 × 2 hut, compare entry from several sides: demolition and cursor use
    the hut cell, repair uses the engineer's actual entry cell. Record all three cells and
    which bridge is changed or offered. A centre-cell substitution or a common selection
    result inconsistent with claims 16, 18 and 21 falsifies those rules.
  - *Link variants:* use high candidates at each of the 16 role positions, with independently
    recorded links and probe cells. Expected: positions 0 to 5 use claim 21's table; the rest
    offer no repair. Construct two links within 3 cells with the earlier-created one farther
    away, then a chain of enabled links ending at a disabled link. The first-created link
    wins, no nearest-link ranking occurs, and the disabled link in the chain offers repair.
    A missing link refuses repair without rebuilding. An earlier unrelated enabled link
    also offers repair under the default quirk, including one near a low link. A contrary
    selection, refusal or spontaneous link creation falsifies the applicable subclaim.
    Record traversal cells to exclude cyclic malformed layouts. Three runs per layout.

- **B14 (claims 3 to 9, 15 and 17, observable ordering).** *Setup:* named small test maps
  with both bridge axes and families, tagged ground occupants in distinct watched cells,
  repeatable counting actions, and stock death effects. Use separate fixtures for a direct
  section loss with a notice-collapse chain, a direct pier collapse and a notice-induced pier
  collapse. *Procedure:* cause each collapse, recording death-trigger action order, explosion
  positions and random-stream values. Repeat three times per fixture. *Expected:* farthest
  notice-chain deaths precede nearer ones, and all precede the directly attempted section's
  deaths. Cross-cell effects match claims 5 and 6, including the distinct direct/notice x-pier
  orders. Ground death effects precede that cell's explosion-choice draws. Direct pier effects
  use pre-collapse heights and precede notice losses, far side before start side. Repeat with
  several isolated runs: the start-side run disappears first, and band actions interleave
  isolated losses as claim 7 specifies. Any reversed death/action order, different point height
  or inconsistent stream-value assignment falsifies the corresponding ordering rule.
  - *Low variant:* tagged occupants on both neighbours and on the directly destroyed strip
    show decreasing-neighbour deaths, increasing-neighbour deaths, band actions, then direct
    strip deaths. Their reversed order falsifies claim 15.
  - *Entry variant:* a low-repair hut and tagged engineer have entered-by and destroyed-by-
    anything counting actions that reveal the watched bridge area. Record sound onset,
    repair-caused occupant deaths, bridge state at each action, and repair/scatter draws.
    Expected: the entered-by action sees the broken bridge, repair sounds precede occupant
    deaths, repair draws precede scatter draws, and the engineer's destruction action sees
    the restored bridge and scatter orders. Any opposite observation falsifies claim 17.
  - *Demolition variant (claim 16):* use even- and odd-length low runs, both nearer-end choices,
    and a run containing already-destroyed strips. Compare selected strip centres and their
    animation points with claim 16's displacement and direction. No animation or effective
    attempt is expected on a selected already-destroyed strip. High demolition affects its
    start end before its far end. Different centres, directions or effect orders falsify
    the respective rules. Repeat from body-marked cells with each distance 1 to 4 to the first
    unmarked cross-cell: the selected section must match claim 16's displacement formula.
    A different span or activity with no unmarked cell within four steps falsifies that rule.

- **B15 (claims 2, 6, 7 and 19; Q-0710 and Q-0717).** Use named test maps with both
  bridge families and axes and the common settings above. Record tile number, sub-tile and
  height of every pier and adjacent tile cell before damage, collapse and repair. Include
  a partially overlaid pier, two touching copies of the same pier-state tile, a diagonal-only
  connection, and an attached different-state tile. Expected: tile replacements follow the
  same-tile eight-neighbour component, not a catalogue rectangle or sub-tile match. Replaced
  map cells and disconnected components stay unchanged, touching same-state copies change
  together, and the different state does not join. Collapse sets every changed cell's height
  to the reference height minus 4; repair raises only the three cells claim 19 names.
  In a named theater-data test make the non-collapsed pier ranges overlap: a direct attempt
  uses x. Also test an x-collapsed/y-live overlap: the live y range wins.
  Tag both former adjacent spans repeatably and arrange gaps without tag actions that
  change bridge state. Collapse their shared pier directly. Record the tile state at each
  search, selected endpoints, section losses and tag raises. Expected: searches occur after
  state 4 is installed and reject that pier at both ends. When the two outer ramps are within
  29 steps, both searches select the same combined extent. For *n* isolated losses during
  the first check, every eligible cell in the combined band receives *n* + 2 raises in total,
  the last from the second check. Use the recorder to distinguish its repeated band from
  two former-span bands. With outer ramps more than 29 steps apart but each former span
  individually within 29, neither check raises a band or loses isolated sections. Repeat
  with a pier already collapsed before a later far-side section loss: it still supplies
  neither endpoint. Accepting state 4, searching before its installation, retaining the
  former pier boundary, a different repeated-band count without intervening state changes,
  or checking the over-cap combined extent falsifies the corresponding rule. A separated start piece beyond a non-bridge
  gap remains eligible up to the map boundary; no eight-neighbour bridge candidate gives no
  span check. Any contrary cell set, axis, band order or bounded start search falsifies the
  corresponding claim. For overlapping deck marks, swap placement order of parallel high
  overlays and record section markings. A section-marked attempted cell selects itself,
  other covered cells select their last marking section. Replacing that overlay does not
  select the next-nearest overlay. On a high-marked pier, repeat after replacing the
  responsible overlay but preserving its state: the pier-selected family can still affect
  that responsible cell and remove the replacement. A different selected section, or refusal
  solely because that replacement is not a high overlay, falsifies claim 2. Three runs
  per fixture. Malformed absent responsibility is a safe departure, not an oracle prediction.
- **B16 (claim 15; Q-0714).** Use a low strip over water with a ship, and a land strip
  holding a vehicle and walking infantry. Change damage states and repair while recording
  exact entry outcomes, current locomotor and other occupants. Include other-occupant cases
  giving *moving blocker*, *stationary friendly* and *blocked*, using named test data or
  forced placement where needed. Expected: only *blocked* causes the entry-refusal hit,
  though the separate aircraft and destruction-jumpjet rows still apply. Self-presence alone
  never supplies that refusal. A ship whose new land type is impassable dies on repair even
  alone. Other-occupant *blocked* is not ignored as a terrain-only interpretation would
  require. Death with only a non-blocked entry outcome, survival with a blocked outcome, or
  refusal caused just by the tested object's self-presence falsifies the corresponding rule.
  Compare states after each preceding occupant death, since a death may free a later
  occupant's entry test. Three runs per case, both families and axes.

- **B17 (claim 6; Q-3046).** Use test maps, each with a high bridge over a pier, one along x
  and one along y. Each map sets some pier cells to cells of another tile, cell by cell, and keeps
  the bridge's other pieces as placed. Attempts are direct attempts (claim 1, a wall-damaging
  warhead) at pier cells that claim 2's pier row covers, as B4 does for the reference cell. The
  firing weapon is named test data: it hits the cell it targets, and its damage amount exceeds
  the game-wide bridge strength, so that each shot at a pier cell in state 0 to 3 makes one
  attempt (S-0303 claim 18).
  - *Setup a (painted-over reference position):* set the reference cell to a one-cell clear tile.
  - *Setup a2:* set the reference cell instead to a cell of another multi-cell tile at that
    tile's own reference sub-tile number (4 along x, 2 along y), and give a block of cells beside
    the bridge, touching that cell, the same tile. Place the pier in state 0. *Setup a3:* as a2,
    with the pier placed in its damaged state (state 3).
  - *Setup b (cut-off part):* in the reference cell's line across the bridge, choose the
    attemptable pier cell farthest from the reference cell toward increasing y (bridge along x) or
    toward increasing x (bridge along y). Set the pier cells that join it to the rest of the pier
    to cells of another tile whose sub-tile number there is greater than the reference sub-tile,
    so that the chosen cell lies in a separate component (Terms). The sub-tile condition makes
    this the layout that Edge cases settles for the original: with painted sub-tiles of *r* or
    less the original's outcome is not settled here. Collapse the main part with two shots at the
    reference cell, so that the cut-off part stays in state 0. Then fire one shot at the chosen
    cell, and after that one at a main-part cell.
  - *Procedure:* in a, a2 and a3, fire one shot at one attemptable pier cell; in b, fire the
    shots b gives. Once each shot has detonated, record the tile and height of every cell where
    the pier's tile was placed (in a3, of the painted cell only) and, in a2 and a3, of every cell
    of the painted tile's connected set of identical cells, and whether the game keeps running
    (a run whose game time has not advanced for 60 seconds of wall time counts as stopped). The
    bridge's other pieces, which notices and span checks also change, are not recorded.
  - *Expected (the original):* a: the game stops. a2: the
    game keeps running, and every cell of the painted tile's connected set of identical cells
    holds the damaged pier tile at an unchanged height. a3: as a2, but with the collapsed pier
    tile at the painted cell's level minus 4. In a2 the pier's own cells end as the notices the
    painted cell sends make them (Edge cases, *Malformed responsibility and cycles*). b: after
    the shot at the cut-off cell, the main part (the component of the reference cell's collapsed
    tile) holds the damaged pier tile at its collapsed heights. After the shot at a main-part
    cell, it holds the collapsed pier tile 4 height levels below its first collapse. Neither of
    these two shots changes any other recorded cell of b.
    *Falsified* by any recorded result, once a shot that has an expected result above has
    detonated, other than that expected result.
  - *Expected (our engine):* every attempt in a, a2, a3 and b's attempt at the cut-off cell
    changes nothing, fails and warns. b's shot at a main-part cell then finds that cell
    collapsed, and nothing happens (claim 2).
  - Runs: in a, a2 and a3, one per setup, axis and attemptable pier cell; in b, one per axis.

## Open questions

- Whether a raise other than an object's arrival carries a cell into the destroy-attached-objects
  action (claim 1). S-0011 owns which raises carry one.
- Whether `quirks.pier_repair_partial_height` changes any stock bridge (claim 19). The affected
  tile-cell set is settled in Terms; no stock-pier height measurement has yet been run.
- Where a passenger leaves a vehicle destroyed on a deck (S-0312 claim 12 points here). This spec has
  not established it.
- Whether mech locomotion counts as heading to a cell (claim 11). B6 proposes negative-kind
  fixtures for the other excluded kinds; none has yet been run.
- A completed intact-bridge hut entry with which to test unconditional engineer consumption
  (claim 17).
- No safe oracle fixture has yet been established for a cyclic malformed cursor-link chain.
  Claim 21 describes ordinary terminating chains; our engine refuses a repeated traversal
  cell with a warning rather than hanging.

## Changes for other specs

- **S-0212:** claim 3's "High-bridge repair names each restored pier's reference cell" becomes
  "High-bridge repair names, for each restored pier, the cell named in S-0511 claim 19's
  Connectivity rule". For a start-piece pier that is the cell one step toward the start side,
  which is the reference cell only for some tile widths.
