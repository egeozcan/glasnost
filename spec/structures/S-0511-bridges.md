---
id: S-0511
title: Bridges (destruction, collapse, falling and repair)
topic: structures
revision: 3
sources-class: [decomp]
status: approved
approved-by: Claude (delegated by project lead)
approved-on: 2026-10-15
measurements: []
requests: []
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
  - *Reference cell:* the pier's cell at sub-tile 4 (bridges along x) or 2 (along y).
  - *Pier column:* the reference cell and its two neighbours across the bridge.

  A pier's state is its tile. Changing the state replaces the tile on every cell of the pier.
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
     - When an attempt from area damage succeeds, every object whose target is the centre cell
       itself drops that target.
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
   - Confidence: medium (test order and acceptance, the low single attempt, the gate); low (target
     dropping, the trigger cell)

2. **What one attempt at a cell acts on.** The cell's first matching condition in the table determines the effect; lower rows apply only when no earlier condition matches.

   | Cell condition, in priority order | Effect |
   |---|---|
   | The cell holds a low piece that is not in the destroyed class | The low strip is affected (claim 13). |
   | The cell is a high-bridge cell | The section whose overlay marks it is affected (claim 3). |
   | The cell's tile is a pier of either family in state 0 to 3, and its sub-tile is even along x or at most 4 along y | The pier is affected (claim 6). Other pier cells without a deck take no effect. |
   | None of the above | Nothing happens and the attempt fails. |
   - Backing: `measurable` (scenarios B1, B4, B7)
   - Confidence: medium

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

   A pier's notices go from its reference cell to the neighbours across its two sides along the
   axis.
   - **Direct collapse:** the pier becomes state 4, and every cell of its tile has the
     reference cell's pre-collapse height level minus 4. Its column receives claim 9's
     blow-ups at pre-collapse heights. Deaths and explosion draws occur in decreasing-cross-
     coordinate, reference, increasing-cross-coordinate order: y across an x bridge, x across
     a y bridge. These effects precede losses caused by the pier's loss notices. Far-side
     losses precede start-side losses. Both adjacent spans have claim 7's isolated-run result,
     and subsequent routes reflect claim 12's connectivity, naming the reference cell.
   - **Collapse from a notice:** the loss propagates onward on the side away from the sender.
     Collapses further along that direction produce their blow-up effects before this pier does.
     This pier's three blow-ups then occur in centre, decreasing-y, increasing-y order along x,
     or decreasing-x, centre, increasing-x order along y. Its tile and height change as above,
     but it starts no span check or connectivity update of its own.
   - Backing: `measurable` (B4 for direct stages, collapse, height drop and notice states,
     B14 for collapse-effect order)
   - Confidence: medium (direct stages, collapse, height drop); low (the notice table)

7. **The span check: isolated runs fall.** After each successful direct attempt on a high bridge, the
   original checks the span that held the lost section, or the spans on both sides of a collapsed
   pier.
   - The span runs from its start piece (a start ramp at sub-tile 8 or 12, or a pier at sub-tile 5
     or 7) to the nearest far piece within 29 cells along its direction (a far ramp, or a pier, at
     sub-tile 4 or 2).
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
   - Backing: `measurable` (B2, B14 for multiple runs and interleaved band actions)
   - Confidence: medium (the isolated-run rule); low (the order of losses, the pass count)

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
      ground layer of the same cell at its current position, its locomotion stops, and it starts
      falling with the bomb mark (claim 10).
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

10. **Falling and landing.** A dropped object falls on each of its own updates (S-0203).
    - Its height changes by its fall rate *v* before *v* is updated (Formulas F3). Any attached
      animation enables the parachute rate recurrence, not only a parachute (S-0522 claim 12).
      The alternative is owned by `quirks.paratrooper_any_animation_slow_fall` in S-0522.
    - Its fall rate starts from the value it already holds. That is 0 for an object that has never
      fallen. An object that has fallen before (by parachute, for example) starts from the rate it
      ended its last fall with (low).
    - When its height above the terrain after the move is 0 or less, the height is set to 0 and the
      fall ends.
    - An object that lands with the bomb mark and positive health then receives a hit equal to its
      current health, distance 0, with the demolition warhead, ignoring defences and carrying the
      no-escape marker, with no attacker and no attacking house. So every object on a lost deck
      dies on landing unless S-0302 stops that hit. A vehicle landing in water dies under S-0312
      claim 11.
    - Over a whole drop from a deck (416 leptons above the terrain under it), with no attached
      animation, *v* starting at 0 and the floor of F3 not reached, the object lands on its 30th
      update (Formulas F3).
    - Backing: `measurable` (B3, including its prior-fall variant) for ordinary and prior
      parachute drops; `unmeasured` here for a non-parachute attachment selecting the slow
      recurrence, because B3 establishes no fixture that keeps such an attachment during a
      deck drop (S-0522 claim 12 owns that shared condition)
    - Confidence: high (fall arithmetic, landing kill); medium (start from a stored rate)

11. **Units heading onto a lost deck.** After an attempt has made its blow-ups, each blown-up cell
    *c* whose deck-layer occupancy record (S-0209) is not empty is checked once more.
    - Let *P* be the centre of *c* at the height of its terrain floor + 416 leptons.
    - Every object in the deck layer, then in the ground layer, of each of the 24 other cells of
      the 5 × 5 block around *c* is examined.
    - It receives the hit of claim 9's ground layer when its locomotion is heading to *P*: its
      current destination lies in *c* within 104 leptons of *P*'s height.
      - For drive and ship locomotion, the destination is the end of the track it is following,
        if any, else its destination.
      - Walk and hover locomotion use their destination.
      - Drop-pod, fly, jumpjet, rocket, teleport and tunnel locomotion do not count.
        The mech kind is not established here.
    - Backing: `measurable` (scenario B6) for drive, walk, hover and ship locomotion and the
      occupancy condition; `unmeasured` for the negative tests of the other kinds, because no
      scenario yet brings each to the tested destination, and for the untraced mech kind
    - Confidence: medium (low for the occupancy condition and untested kinds)

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
        - when it could not enter the cell as it now is (the entry test of S-0209 for its own kind),
        - when the cell became destroyed and the object is of a type with jumpjet locomotion,
        - when the object is an aircraft, even when the change was only to damaged.
      - Then every unit in the ground layer of the other 24 cells of the 5 × 5 block around it
        receives the same hit when its locomotion is heading to that cell at its floor height
        (claim 11's test) and it could not enter that cell.
    - Backing: `measurable` (B5, B7, B6 low-bridge variant and B14 low-order variant)
    - Confidence: medium (low for the jumpjet and aircraft cases)

### Demolition and repair through the hut

16. **Hut demolition.** A bridge repair hut demolishes its bridge in two cases:
    - when it is the target of a timed bomb that detonates (after the bomb's own detonation),
    - when its delayed-kill countdown expires (S-0302 claim 16).

    Neither case applies the gate of claim 1. The expiry clears the countdown and the recorded
    planter, and the hut takes no expiry hit.
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
      consuming an engineer at an intact bridge, because the repair cursor can refuse the order
      and no fixture yet establishes how to complete that entry. Observable entry ordering is
      `measurable` through B14
    - Confidence: medium

18. **Which bridge the hut repairs.** The family is chosen as in claim 16, from the 5 × 5 block around
    the engineer's cell. The selected bridge obeys these priorities:
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
    - **Repair extent.** A selected start ramp loses its alternate look and its span has
      claim 19's repaired state. A selected pier is restored under claim 19, together with the
      bridge selected by this claim from the cell two steps toward its start side and its own
      far-side span. For several selected piers, restored states extend toward the start side
      before the far-side spans receive sections. Any occupant or trigger observations during
      repair therefore see the start-side repairs before the far-side ones.
    - Backing: `measurable` (scenario B12)
    - Confidence: medium

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
    - **Restoring a pier.** The pier's tile goes back to state 0 on all its cells, whatever state it
      was in. If it was collapsed (state 4), the reference cell and its two neighbours across the
      bridge each rise by 4 height levels. Other cells of the pier's tile are not raised
      (`quirks.pier_repair_partial_height`).
    - **Connectivity.** Each restored pier, and the far ramp, name a cell for S-0212 claim 3. So
      does the cell one step before the far ramp. Then connectivity is updated.
    - **Objects.** No object is harmed or moved by a high-bridge repair.
    - Backing: `measurable` (scenario B12)
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

21. **The repair cursor uses its own test.** Whether the engineer's cursor offers a repair over a hut
    (S-0214 rows I3 and IC2) is decided by a test separate from claim 18. That test asks whether a
    bridge within the hut's 5 × 5 block is destroyed:
    - for a low bridge, whether its run holds a destroyed piece,
    - for an ordinary, unambiguous high bridge, whether its nearby bridge link is not enabled.
      Selection among several nearby bridges and exceptional link configurations remains open.

    The different selection tests may make cursor and repair disagree, but no such layout is
    established here.
    - Backing: `measurable` (scenario B13) for the ordinary low- and high-bridge cursor tests.
      `unmeasured` for a disagreement between cursor and repair, because no discriminating layout
      has yet been established
    - Confidence: low

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

Here trunc is truncation toward zero of a double-precision value, and *v*_min is the imported no-parachute maximum fall rate, a negative number. For *vₙ* ≤ 0 the rate update is *vₙ₊₁* = *vₙ* − 1, so from *v₀* = 0 the drop after *n* updates is 0 + 1 + … + (*n* − 1) = *n*(*n* − 1)/2 leptons, until *v*_min is reached. The landing test (*hₙ₊₁* ≤ 0) follows the move. While any animation is attached to the falling object, the rate recurrence instead is *vₙ₊₁* = max(*vₙ* − 1, the imported parachute maximum fall rate), using exact integer subtraction and comparison. A parachute is one such animation, not a type restriction. S-0522 claim 12 and `quirks.paratrooper_any_animation_slow_fall` own this attachment condition and its alternative.

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

**Cross-spec changes required at joint sign-off.** S-0302 claim 16 must describe hut expiry
as demolition, not repair, and claim 1's bridge hits can cite claims 9 to 11 and 15 here.
S-0303 claim 18 must restrict ion-cannon retries to high tests, use the skirmish option gate,
and adopt claim 1's membership, height and draw-order boundaries. S-0202 claim 13 must include
claim 20's in-play map-generator draws. S-0011 claims 20 and 27 must adopt claims 8 and 1's
band and cell-attempt rules. S-0212 claim 3 must use claims 12 and 19's named cells. The
remaining existing bridge pointers in S-0010 claim 25, S-0211 claim 15, S-0214 rows I3 and IC2,
and S-0312 claims 11 and 12 should point to the respective rules here without claiming that
passenger placement or exceptional cursor selection has been resolved.

## Edge cases and quirks

- `quirks.pier_loss_notice_skips_chipped` (default on, the original): a pier chipped on one side
  ignores a loss notice arriving on that same side (claim 6). The usual way a section next to a
  pier is lost — damaged first, then lost — therefore leaves the pier only chipped, not damaged.
  Off ("fixed"): a loss notice turns any pier in state 0 to 2 into state 3.
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
- `quirks.pier_repair_partial_height` (default on): a collapse lowers every cell of the pier's tile
  to the reference cell's level − 4, but repair raises only the pier column. Off: repair restores
  every cell of the pier's tile to the level it had before the collapse.
- `quirks.low_strip_regress` (default on): re-evaluation can turn a D₀ strip back into an
  intact edge look I₅ or I₄ when *s* is 1 or 4 respectively (claim 14). This is not known to be reachable from
  stock play, only from pieces placed damaged by a map. Off: re-evaluation never moves a strip from
  the damaged class to the intact class.
- `quirks.low_bridge_wrong_axis_recheck` (proposed, default on, the original): a low piece of the
  other axis lying along the line is re-evaluated with this line's table. Off: a neighbouring
  strip of the other axis is not re-evaluated. This matters only for unusual map data.
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

**Piece-role binding.** Hₓ and Hᵧ are the high-overlay identities whose marking directions
are map north and map west respectively (S-0211 claim 15). The pair identified by the original
restricted-deck exception of S-0211 claim 14 belongs to the wooden-bridge-role family, regardless
of that quirk's current setting; the other pair belongs
to the bridge-role family. These bindings identify imported types, not newly configurable keys.

For each low-bridge family and axis, let *t*₀ be its bridge-piece base type in S-0202 claim 10.
Of the two base types for a given axis, the lower input registration index identifies the
wooden-bridge-role palette and the higher identifies the bridge-role palette. The following exact integer displacements from *t*₀ bind the symbolic
roles. They describe the fixed bridge palette's transition relationships, not stock rule values.
A displacement counts overlay registry positions in the input (S-0004 claim 4); our engine may
store the resolved identities in any order.

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

All scenarios use stock data, with the director and mods off, the bridge-destruction
option on unless stated, and any fixed game speed. Except for the ion-cannon warhead, attempts
from area damage need a chance draw (S-0303 claim 18), so scenarios never count ordinary shots. They fire until the bridge's
look changes and record each change, with its tick, in order. The harness should record per tick:
the overlay position and data value of every watched cell, the tile and height level of watched
cells, object deaths with their tick, object heights, trigger raises, and, where a scenario says
so, the number of draws taken from a named random stream. Use one human player and one bystander
computer player kept outside the observed bridge, hut and approach cells, with the glossary's void
rule. Record the chosen speed and replay each deterministic sub-test three times. The proposed tools
are scripted input with video capture and a read-only state and random-stream recorder. These are
proposals, not measurements already performed. Void timing and survival tests if the destruction
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
  - *Procedure:* keep a stationary deck unit in the watched cell, and lose section 4 while
    the first two vehicles are one step away. Record occupancy and movement destinations before
    and after the loss. Repeat with the watched cell initially empty.
  - *Expected:* when its deck occupancy record is nonempty at claim 11's check, vehicles heading
    onto the cell die in that tick, and the third does not. With an empty record there is no such
    heading hit. A hover vehicle heading there dies too. Repeat with walking infantry and a ship
    approaching a cell at floor height for the low-bridge kill check. The drop of the stationary
    unit and all earlier deaths must precede the heading hits.
  - *Falsified by:* a heading vehicle surviving with the nonempty-record condition satisfied,
    a heading hit despite the empty-record condition, the third vehicle dying, or reversed
    death ordering. The other locomotion kinds remain outside this scenario's backing.
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
    after collapse and after repair. Only the three column cells rise by 4 on repair. Any other
    raised cell, or an unchanged column cell, falsifies claim 19. Repeat on a span with the far
    ramp at distance 29 and then 30: the latter places no sections, although piers restored before
    that distance stay restored. Contrary overlay placement falsifies the cap.
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

## Open questions

- Whether a raise other than an object's arrival carries a cell into the destroy-attached-objects
  action (claim 1). S-0011 owns which raises carry one.
- The exact cells of a pier's tile, and so whether `quirks.pier_repair_partial_height` changes any
  stock bridge (claim 19).
- Where a passenger leaves a vehicle destroyed on a deck (S-0312 claim 12 points here). This spec has
  not established it.
- Whether mech locomotion counts as heading to a cell (claim 11), and a discriminating fixture
  for each of its other untested locomotion kinds.
- Exceptional or ambiguous cursor-link selection, and a completed intact-bridge hut entry with
  which to test unconditional engineer consumption (claims 17 and 21).
- The cell of a hut used by claims 16 and 18 when its foundation has several cells (low).
